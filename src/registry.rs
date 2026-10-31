// SPDX-License-Identifier: AGPL-3.0-or-later
//! The registry value (ADR-012 §7.1): built by the driver from provider
//! manifests and passed to every consumer as an argument.

use qsl_foundation::BackendId;
use qsl_route::{
    descriptor_from_manifest, BackendDescriptor, Candidate, ManifestMembers, ProviderOrigin,
    RegistrationRefusal, Registry,
};
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::{DriverRefusal, Provider};

#[derive(serde::Serialize, quire_canonical::FixedShape)]
struct LinkedManifestWire<'a> {
    backend: &'a str,
    advertises: Vec<AdvertisedPairWire<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domains: Option<&'a Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bounds: Option<&'a BTreeMap<String, u64>>,
}

#[derive(serde::Serialize, quire_canonical::FixedShape)]
struct AdvertisedPairWire<'a> {
    kind: &'a str,
    mode: &'a str,
}

/// The FR-331 provider-manifest fields the driver registers a backend from.
///
/// This type and [`build_registry`] are for compile-time linked providers.
/// Plugin hello manifests carry `Process` origin and must use the origin-aware
/// registration path instead (ADR-029 PV-1).
///
/// The `advertises` pairs are the manifest's `(kind, mode)` labels exactly
/// as it states them; QSL admits them through `descriptor_from_manifest`, so an
/// absent or unknown label refuses the registration.
/// Optional `domains` labels are passed to that same QSL admission call; QSL
/// owns boundable-kind and mode/domain consistency refusals.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderManifest {
    /// The backend identity (`backend.identity`).
    pub identity: String,
    /// Each advertised `(kind, mode)` label pair; `None` is an absent label.
    pub advertises: Vec<(Option<String>, Option<String>)>,
    /// FR-331 domains retained for linked projection or process admission.
    pub domains: Option<Vec<String>>,
    /// FR-331 run-limit defaults retained for linked projection or process admission.
    pub bounds: Option<BTreeMap<String, u64>>,
    /// RFC 8785 bytes of the complete retained FR-331 descriptor.
    pub wire: Option<Arc<[u8]>>,
}

/// A registry accepted by the driver's complete manifest-admission pass.
///
/// The wrapper exposes shared access by default. Process descriptors that have
/// already crossed QSL admission may be appended explicitly through
/// [`LinkedProviders::register_admitted`].
#[derive(Debug)]
pub struct AdmittedRegistry {
    registry: Registry,
    manifests: Vec<ProviderManifest>,
}

impl AdmittedRegistry {
    /// Retained FR-331 descriptors paired with each admitted QSL descriptor.
    #[must_use]
    pub fn manifest_snapshot(&self) -> &[ProviderManifest] {
        &self.manifests
    }
}

/// The QSL registry and linked providers admitted from the same manifest set.
///
/// Provider lookup is by the opaque backend identity, the same value the
/// registry routes. Lookup returns `None` when no implementation matches or
/// when identical idempotent manifests name multiple implementations; the
/// latter is ambiguous even though QSL admits and routes the identity.
pub struct LinkedProviders<'a> {
    admitted: AdmittedRegistry,
    providers: Vec<(String, &'a dyn Provider)>,
}

impl LinkedProviders<'_> {
    /// The immutable registry built from these providers' manifests.
    #[must_use]
    pub fn registry(&self) -> &AdmittedRegistry {
        &self.admitted
    }

    /// The retained FR-331 descriptor snapshot in registry order.
    #[must_use]
    pub fn manifest_snapshot(&self) -> Vec<ProviderManifest> {
        let mut manifests = self
            .admitted
            .manifests
            .iter()
            .filter(|manifest| {
                self.admitted
                    .registry
                    .descriptors()
                    .any(|descriptor| descriptor.id().as_str() == manifest.identity)
            })
            .cloned()
            .collect::<Vec<_>>();
        manifests.sort_by(|left, right| left.identity.cmp(&right.identity));
        manifests
    }

    /// Register caller-admitted descriptors and their lossless FR-331 DTOs in
    /// the same registry that owns linked providers. Every registered
    /// descriptor has one retained manifest snapshot source. QSL retains
    /// authority over duplicate refusal; this method does not re-admit.
    pub fn register_admitted(
        &mut self,
        admissions: impl IntoIterator<Item = (BackendDescriptor, ProviderManifest)>,
    ) -> Result<(), DriverRefusal> {
        let admissions = admissions.into_iter().collect::<Vec<_>>();
        let registration_result = register_into_registry(
            &mut self.admitted.registry,
            admissions.iter().map(|(descriptor, _)| descriptor.clone()),
        );
        for (_, manifest) in admissions {
            if !self
                .admitted
                .manifests
                .iter()
                .any(|retained| retained.identity == manifest.identity)
            {
                self.admitted.manifests.push(manifest);
            }
        }
        self.admitted.manifests.retain(|manifest| {
            self.admitted
                .registry
                .descriptors()
                .any(|descriptor| descriptor.id().as_str() == manifest.identity)
        });
        registration_result
    }

    /// The unique linked provider for `backend`, if present.
    ///
    /// Returns `None` when there is no matching provider or when multiple
    /// implementations share an identical idempotent manifest. The latter
    /// identity is admitted by QSL, but this lookup does not choose an
    /// implementation by input order.
    #[must_use]
    pub fn provider(&self, backend: &BackendId) -> Option<&dyn Provider> {
        if !self
            .admitted
            .descriptors()
            .any(|descriptor| descriptor.id() == backend)
        {
            return None;
        }
        // A repeated identical manifest is QSL-idempotent, but it does not
        // identify which of multiple linked implementations should execute.
        let mut matching = self
            .providers
            .iter()
            .filter(|(identity, _)| identity == backend.as_str());
        let provider = matching.next()?.1;
        matching.next().is_none().then_some(provider)
    }
}

impl AdmittedRegistry {
    /// Lossless manifest snapshots retained for descriptors admitted into this registry.
    #[must_use]
    pub fn manifests(&self) -> &[ProviderManifest] {
        &self.manifests
    }
}

impl Deref for AdmittedRegistry {
    type Target = Registry;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

/// Build the registry from `manifests`.
///
/// Every manifest is admitted with `ProviderOrigin::Linked` and registered.
/// Do not use this function for plugin hello manifests: those require
/// `ProviderOrigin::Process` under ADR-029 PV-1. When any registration is
/// refused (an absent or unknown kind or mode, invalid domains, or a duplicate
/// backend identity), the run is refused with every refusal, ordered by
/// backend identity then identity.
///
/// `Registry::register`'s own `Err` is what that one call caused or
/// observed, not a diff against the registry's prior state, so it can
/// repeat a conflict an earlier call in this same loop already reported.
/// Rather than accumulate those (and dedup them by hand), this collects
/// only the vocabulary refusals `descriptor_from_manifest` produces during
/// the loop, then reads every duplicate-backend conflict once, after the
/// loop, from `Registry::refusals` -- the registry's own stable, deduped
/// account of every identity conflict it currently holds.
pub fn build_registry(manifests: &[ProviderManifest]) -> Result<AdmittedRegistry, DriverRefusal> {
    let mut registry = Registry::new();
    let mut refusals = Vec::new();
    let mut admitted_manifests = Vec::new();
    for manifest in manifests {
        let backend_id = BackendId::from_wire(&manifest.identity)
            .map_err(DriverRefusal::EmptyBackendIdentity)?;
        if manifest.advertises.is_empty() {
            return Err(DriverRefusal::EmptyLinkedManifestAdvertisements);
        }
        let backend = Candidate::new(backend_id);
        let domain_labels = manifest
            .domains
            .as_deref()
            .map(|domains| domains.iter().map(String::as_str).collect::<Vec<_>>());
        let admitted = descriptor_from_manifest(
            backend,
            ProviderOrigin::Linked,
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
            ManifestMembers {
                domains: domain_labels.as_deref(),
                ..ManifestMembers::default()
            },
        );
        match admitted {
            Ok(descriptor) => {
                let wire = linked_manifest_wire(manifest)?;
                // The conflict is read back, deduped, from `refusals()`
                // below; a successful registration needs nothing here.
                let _ = registry.register(descriptor);
                if !admitted_manifests
                    .iter()
                    .any(|retained: &ProviderManifest| retained.identity == manifest.identity)
                {
                    let mut retained = manifest.clone();
                    retained.wire = Some(wire);
                    admitted_manifests.push(retained);
                }
            }
            Err(refusal) => refusals.push(refusal),
        }
    }
    refusals.extend(registry.refusals());
    if refusals.is_empty() {
        Ok(AdmittedRegistry {
            registry,
            manifests: admitted_manifests,
        })
    } else {
        refusals.sort();
        Err(DriverRefusal::Registration(refusals))
    }
}

fn linked_manifest_wire(manifest: &ProviderManifest) -> Result<Arc<[u8]>, DriverRefusal> {
    if let Some(wire) = &manifest.wire {
        return Ok(Arc::clone(wire));
    }

    let advertises = manifest
        .advertises
        .iter()
        .map(|(kind, mode)| match (kind.as_deref(), mode.as_deref()) {
            (Some(kind), Some(mode)) => Ok(AdvertisedPairWire { kind, mode }),
            _ => Err(DriverRefusal::Fault(
                crate::DriverFault::AdmittedManifestMissingLabel,
            )),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let descriptor = LinkedManifestWire {
        backend: &manifest.identity,
        advertises,
        domains: manifest.domains.as_ref(),
        bounds: manifest.bounds.as_ref(),
    };
    let bytes = quire_canonical::to_vec(
        &descriptor,
        // FR-331 defines no linked-manifest byte ceiling. Linked manifests
        // are supplied by in-process providers; do not add a smaller driver
        // limit than the canonical encoder's representable byte range.
        quire_canonical::Limits::new(u64::MAX),
    )
    .map_err(DriverRefusal::ManifestEncoding)?;
    Ok(Arc::from(bytes))
}

/// Register descriptors admitted by the caller (for example, the CLI's
/// process-host hello admitter) without performing descriptor admission a
/// second time. The caller supplies each descriptor's origin; this function
/// preserves it and delegates duplicate-backend policy to QSL's registry.
fn register_into_registry(
    registry: &mut Registry,
    descriptors: impl IntoIterator<Item = BackendDescriptor>,
) -> Result<(), DriverRefusal> {
    for descriptor in descriptors {
        let _ = registry.register(descriptor);
    }
    let refusals: Vec<RegistrationRefusal> = registry.refusals().collect();
    if refusals.is_empty() {
        Ok(())
    } else {
        Err(DriverRefusal::Registration(refusals))
    }
}

/// Admit compile-time providers and retain the identity-to-provider mapping
/// used after negotiation and routing.
pub fn build_linked_providers<'a>(
    providers: &[&'a dyn Provider],
) -> Result<LinkedProviders<'a>, DriverRefusal> {
    let manifests = providers
        .iter()
        .map(|provider| provider.manifest())
        .collect::<Vec<_>>();
    let admitted = build_registry(&manifests)?;
    let linked = providers
        .iter()
        .zip(manifests)
        .map(|(provider, manifest)| (manifest.identity, *provider))
        .collect();
    Ok(LinkedProviders {
        admitted,
        providers: linked,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use qsl_foundation::BackendId;
    use qsl_route::{descriptor_from_manifest, Candidate, ProviderOrigin};

    use crate::{KaniProvider, Provider, ProviderManifest};

    use super::build_linked_providers;

    fn provider(identity: &str) -> KaniProvider {
        KaniProvider::new(ProviderManifest {
            identity: identity.to_owned(),
            advertises: vec![(
                Some("operation-contract".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        })
    }

    /// Trace: FR-006-AC-5.
    #[test]
    fn linked_manifest_snapshot_is_independent_of_registration_order() {
        let first = provider("agent-ix/provider/first");
        let second = provider("agent-ix/provider/second");
        let admitted = build_linked_providers(&[&first, &second]).expect("both providers admit");
        assert!(admitted
            .registry()
            .descriptors()
            .all(|descriptor| descriptor.domains().is_some_and(|domains| {
                domains.contains(&qsl_foundation::bound::DomainKind::Integer)
            })));
        let forward = admitted.manifest_snapshot();
        let reverse = build_linked_providers(&[&second, &first])
            .expect("both providers admit")
            .manifest_snapshot();

        assert_eq!(forward, reverse);
        assert!(forward
            .windows(2)
            .all(|pair| pair[0].identity < pair[1].identity));
    }

    /// Trace: FR-005-AC-16.
    #[test]
    fn linked_registry_populates_canonical_manifest_wire_and_preserves_absence() {
        let absent_optional = KaniProvider::new(ProviderManifest {
            identity: "agent-ix/provider/without-optional".to_owned(),
            advertises: vec![(
                Some("operation-contract".to_owned()),
                Some("unbounded".to_owned()),
            )],
            domains: None,
            bounds: None,
            wire: None,
        });
        let present_optional = KaniProvider::new(ProviderManifest {
            identity: "agent-ix/provider/with-optional".to_owned(),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: Some(BTreeMap::from([("max_states".to_owned(), 8)])),
            wire: None,
        });

        let manifests = build_linked_providers(&[&absent_optional, &present_optional])
            .expect("QSL admits both linked manifests")
            .manifest_snapshot();
        let absent = manifests
            .iter()
            .find(|manifest| manifest.identity == "agent-ix/provider/without-optional")
            .expect("absent-optional manifest is retained");
        let present = manifests
            .iter()
            .find(|manifest| manifest.identity == "agent-ix/provider/with-optional")
            .expect("present-optional manifest is retained");

        assert_eq!(
            absent.wire.as_deref(),
            Some(&b"{\"advertises\":[{\"kind\":\"operation-contract\",\"mode\":\"unbounded\"}],\"backend\":\"agent-ix/provider/without-optional\"}"[..])
        );
        assert_eq!(
            present.wire.as_deref(),
            Some(&b"{\"advertises\":[{\"kind\":\"value-validity\",\"mode\":\"bounded\"}],\"backend\":\"agent-ix/provider/with-optional\",\"bounds\":{\"max_states\":8},\"domains\":[\"integer\"]}"[..])
        );
    }

    /// Trace: FR-005-AC-16.
    #[test]
    fn linked_registry_preserves_caller_supplied_wire_byte_for_byte() {
        let wire: std::sync::Arc<[u8]> = std::sync::Arc::from(
            &b"{\"advertises\":[{\"kind\":\"value-validity\",\"mode\":\"bounded\"}],\"backend\":\"agent-ix/provider/full-wire\",\"domains\":[\"integer\"],\"options\":{\"proof\":\"exact\"}}"[..],
        );
        let provider = KaniProvider::new(ProviderManifest {
            identity: "agent-ix/provider/full-wire".to_owned(),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: Some(std::sync::Arc::clone(&wire)),
        });

        let snapshot = build_linked_providers(&[&provider])
            .expect("QSL admits the linked descriptor")
            .manifest_snapshot();
        assert_eq!(snapshot[0].wire.as_deref(), Some(wire.as_ref()));
    }

    /// Trace: FR-005-AC-16.
    #[test]
    fn linked_manifest_encoding_does_not_add_a_one_megabyte_limit() {
        let identity = format!("agent-ix/provider/{}", "x".repeat(1_048_577));
        let provider = KaniProvider::new(ProviderManifest {
            identity,
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        });

        let snapshot = build_linked_providers(&[&provider])
            .expect("FR-331 has no one-megabyte linked-manifest ceiling")
            .manifest_snapshot();
        let wire = snapshot[0]
            .wire
            .as_deref()
            .expect("registry populated canonical wire");
        assert!(wire.len() > 1_048_576);
    }

    /// Trace: FR-005-AC-16.
    #[test]
    fn linked_registry_refuses_empty_advertisements_before_wire_emission() {
        let manifest = ProviderManifest {
            identity: "agent-ix/provider/empty-advertisements".to_owned(),
            advertises: Vec::new(),
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        };

        let refusal = super::build_registry(&[manifest])
            .expect_err("FR-331 requires at least one advertised capability");
        assert!(matches!(
            refusal,
            crate::DriverRefusal::EmptyLinkedManifestAdvertisements
        ));
        assert_eq!(refusal.code(), "invalid_capability");
        assert_eq!(
            refusal.qsl_category(),
            Some(quire_semantic_value::catalog::Category::Refusal)
        );
        assert_eq!(refusal.stage(), crate::DriverStage::Registry);
        assert_eq!(refusal.kind(), crate::OutcomeKind::Refused);
    }

    #[ix_trace_rs::trace("TC-004", "FR-001-AC-5")]
    #[test]
    fn linked_probabilistic_manifest_retains_the_absent_evidence_kind_refusal() {
        let manifest = ProviderManifest {
            identity: "agent-ix/provider/probabilistic".to_owned(),
            advertises: vec![(
                Some("probabilistic-satisfaction".to_owned()),
                Some("unbounded".to_owned()),
            )],
            domains: None,
            bounds: None,
            wire: None,
        };
        let refusal = super::build_registry(&[manifest])
            .expect_err("the driver's manifest has no evidence-kinds member to supply");
        assert_eq!(refusal.code(), "invalid_capability/unknown-evidence-kind");
        assert_eq!(refusal.kind(), crate::OutcomeKind::Refused);
        assert_eq!(refusal.stage(), crate::DriverStage::Registry);
        assert!(
            matches!(refusal, crate::DriverRefusal::Registration(ref errors)
            if errors.len() == 1 && errors[0].cause() == &qsl_route::RegistrationCause::UnknownEvidenceKind(
                qsl_route::EvidenceKindsDefect::Absent
            ))
        );
    }

    /// A bounded advertisement without QSL's required domain labels preserves
    /// the typed FR-290 admission cause instead of inventing a default domain.
    #[test]
    fn linked_registry_preserves_absent_domains_refusal() {
        let manifest = ProviderManifest {
            identity: "agent-ix/provider/missing-domains".to_owned(),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: None,
            bounds: None,
            wire: None,
        };
        let crate::DriverRefusal::Registration(refusals) =
            super::build_registry(&[manifest]).expect_err("QSL requires bounded domains")
        else {
            panic!("QSL admission refusal remains typed");
        };
        assert_eq!(refusals.len(), 1);
        assert_eq!(
            refusals[0].cause(),
            &qsl_route::RegistrationCause::InvalidDomains(qsl_route::DomainsDefect::Absent)
        );
    }

    /// Trace: FR-005-AC-5.
    /// Caller-admitted Process origin is preserved by registration.
    #[test]
    fn caller_admitted_process_descriptor_joins_the_existing_registry() {
        let linked_provider = provider("agent-ix/provider/linked");
        let mut linked =
            build_linked_providers(&[&linked_provider]).expect("linked provider admits");
        let descriptor = descriptor_from_manifest(
            Candidate::new(
                BackendId::from_wire("agent-ix/plugin/test").expect("nonempty identity"),
            ),
            ProviderOrigin::Process,
            [(Some("operation-contract"), Some("bounded"))],
            qsl_route::ManifestMembers {
                domains: Some(&["integer"]),
                ..Default::default()
            },
        )
        .expect("the host hello admitter returns a QSL-admitted descriptor");
        let id = BackendId::from_wire("agent-ix/plugin/test").expect("nonempty identity");
        linked
            .register_admitted([(
                descriptor,
                ProviderManifest {
                    identity: "agent-ix/plugin/test".to_owned(),
                    advertises: vec![(
                        Some("operation-contract".to_owned()),
                        Some("bounded".to_owned()),
                    )],
                    domains: Some(vec!["integer".to_owned()]),
                    bounds: None,
                    wire: None,
                },
            )])
            .expect("descriptor registers into the existing registry");
        let registered = linked
            .registry()
            .descriptors()
            .find(|descriptor| descriptor.id() == &id)
            .expect("registered descriptor");
        assert_eq!(registered.origin(), ProviderOrigin::Process);
        assert_eq!(linked.registry().backends().count(), 2);
        let snapshot = linked.manifest_snapshot();
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot
            .iter()
            .any(|manifest| manifest.identity == "agent-ix/plugin/test"));
    }

    /// Trace: FR-005-AC-4.
    /// Unequal linked/process descriptors withdraw their shared identity and
    /// retain each distinct identity in QSL's refusal.
    #[test]
    fn linked_process_conflict_returns_qsl_refusals_and_withdraws_identity() {
        let identity = "agent-ix/provider/shared";
        let linked_provider = provider(identity);
        let other_linked = provider("agent-ix/provider/linked");
        let mut linked = build_linked_providers(&[&linked_provider, &other_linked])
            .expect("linked providers admit");
        let process = descriptor_from_manifest(
            Candidate::new(BackendId::from_wire(identity).expect("nonempty identity")),
            ProviderOrigin::Process,
            [(Some("operation-contract"), Some("bounded"))],
            qsl_route::ManifestMembers {
                domains: Some(&["integer"]),
                ..Default::default()
            },
        )
        .expect("process descriptor admits");

        let refusal = linked
            .register_admitted([
                (
                    process,
                    ProviderManifest {
                        identity: identity.to_owned(),
                        advertises: vec![(
                            Some("operation-contract".to_owned()),
                            Some("bounded".to_owned()),
                        )],
                        domains: Some(vec!["integer".to_owned()]),
                        bounds: None,
                        wire: None,
                    },
                ),
                (
                    descriptor_from_manifest(
                        Candidate::new(
                            BackendId::from_wire("agent-ix/provider/later")
                                .expect("nonempty identity"),
                        ),
                        ProviderOrigin::Process,
                        [(Some("operation-contract"), Some("bounded"))],
                        qsl_route::ManifestMembers {
                            domains: Some(&["integer"]),
                            ..Default::default()
                        },
                    )
                    .expect("later descriptor admits"),
                    ProviderManifest {
                        identity: "agent-ix/provider/later".to_owned(),
                        advertises: vec![(
                            Some("operation-contract".to_owned()),
                            Some("bounded".to_owned()),
                        )],
                        domains: Some(vec!["integer".to_owned()]),
                        bounds: None,
                        wire: None,
                    },
                ),
            ])
            .expect_err("different origin and manifest conflict");
        let refusals = match refusal {
            crate::DriverRefusal::Registration(refusals) => refusals,
            _ => Vec::new(),
        };
        assert_eq!(refusals.len(), 2);
        assert_eq!(refusals[0].identity().as_str(), identity);
        assert_eq!(refusals[1].identity().as_str(), identity);
        let id = BackendId::from_wire(identity).expect("nonempty identity");
        assert!(linked
            .registry()
            .descriptors()
            .all(|descriptor| descriptor.id() != &id));
        assert!(linked.provider(&id).is_none());
        let snapshot = linked.manifest_snapshot();
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot
            .iter()
            .any(|manifest| manifest.identity == "agent-ix/provider/linked"));
        assert!(snapshot
            .iter()
            .any(|manifest| manifest.identity == "agent-ix/provider/later"));
        assert!(snapshot
            .iter()
            .all(|manifest| manifest.identity != identity));
    }

    /// Trace: FR-005-AC-4.
    /// Repeating an identical caller-admitted descriptor is idempotent under
    /// the QSL registry.
    #[test]
    fn repeated_process_descriptor_is_idempotent() {
        let linked_provider = provider("agent-ix/provider/linked");
        let mut linked =
            build_linked_providers(&[&linked_provider]).expect("linked provider admits");
        let process = descriptor_from_manifest(
            Candidate::new(
                BackendId::from_wire("agent-ix/provider/process").expect("nonempty identity"),
            ),
            ProviderOrigin::Process,
            [(Some("operation-contract"), Some("bounded"))],
            qsl_route::ManifestMembers {
                domains: Some(&["integer"]),
                ..Default::default()
            },
        )
        .expect("process descriptor admits");

        linked
            .register_admitted([
                (
                    process.clone(),
                    ProviderManifest {
                        identity: "agent-ix/provider/process".to_owned(),
                        advertises: vec![(
                            Some("operation-contract".to_owned()),
                            Some("bounded".to_owned()),
                        )],
                        domains: Some(vec!["integer".to_owned()]),
                        bounds: None,
                        wire: None,
                    },
                ),
                (
                    process,
                    ProviderManifest {
                        identity: "agent-ix/provider/process".to_owned(),
                        advertises: vec![(
                            Some("operation-contract".to_owned()),
                            Some("bounded".to_owned()),
                        )],
                        domains: Some(vec!["integer".to_owned()]),
                        bounds: None,
                        wire: None,
                    },
                ),
            ])
            .expect("identical descriptor repeat is idempotent");
        assert_eq!(linked.registry().backends().count(), 2);
        assert_eq!(linked.registry().refusals().count(), 0);
    }

    /// Trace: FR-005-AC-1, FR-005-AC-3.
    #[test]
    fn linked_manifests_admit_distinct_identities_and_resolve_provider_lookup() {
        let first = provider("agent-ix/provider/first");
        let second = provider("agent-ix/provider/second");
        let providers: [&dyn Provider; 2] = [&first, &second];
        let linked = build_linked_providers(&providers).expect("both manifests admit");

        let first_id = BackendId::from_wire("agent-ix/provider/first").expect("nonempty id");
        let second_id = BackendId::from_wire("agent-ix/provider/second").expect("nonempty id");
        assert_eq!(
            linked
                .provider(&first_id)
                .map(|provider| provider.manifest().identity),
            Some("agent-ix/provider/first".to_owned())
        );
        assert_eq!(
            linked
                .provider(&second_id)
                .map(|provider| provider.manifest().identity),
            Some("agent-ix/provider/second".to_owned())
        );
        assert_eq!(linked.registry().backends().count(), 2);
        assert!(linked
            .registry()
            .descriptors()
            .all(|descriptor| descriptor.origin() == ProviderOrigin::Linked));
    }

    /// Trace: FR-005-AC-4.
    #[test]
    fn identical_linked_manifests_are_idempotent_without_order_dependent_lookup() {
        let first = provider("agent-ix/provider/same");
        let second = provider("agent-ix/provider/same");
        let first_order: [&dyn Provider; 2] = [&first, &second];
        let reverse_order: [&dyn Provider; 2] = [&second, &first];

        let first_linked =
            build_linked_providers(&first_order).expect("equal manifests are idempotent");
        let reverse_linked =
            build_linked_providers(&reverse_order).expect("equal manifests are idempotent");
        let id = BackendId::from_wire("agent-ix/provider/same").expect("nonempty id");

        assert_eq!(first_linked.registry().backends().count(), 1);
        assert_eq!(reverse_linked.registry().backends().count(), 1);
        assert!(first_linked.provider(&id).is_none());
        assert!(reverse_linked.provider(&id).is_none());
    }
}
