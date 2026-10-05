// SPDX-License-Identifier: AGPL-3.0-or-later
//! The registry value (ADR-012 §7.1): built by the driver from provider
//! manifests and passed to every consumer as an argument.

use qsl_route::{
    BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin, Registry,
};
use std::ops::Deref;

use crate::{DriverRefusal, Provider};

/// The FR-331 provider-manifest fields the driver registers a backend from.
///
/// The `advertises` pairs are the manifest's `(kind, mode)` labels exactly
/// as it states them; `route` admits them (`BackendDescriptor::admit`), so an
/// absent or unknown label refuses the registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderManifest {
    /// The backend identity (`backend.identity`).
    pub identity: String,
    /// The digest of this manifest (`backend.manifest_digest`).
    pub manifest_digest: ManifestDigest,
    /// Each advertised `(kind, mode)` label pair; `None` is an absent label.
    pub advertises: Vec<(Option<String>, Option<String>)>,
}

/// A registry accepted by the driver's complete manifest-admission pass.
///
/// The wrapper exposes only shared access, so callers cannot add a conflicting
/// descriptor after admission and then create cache requests from stale state.
#[derive(Debug)]
pub struct AdmittedRegistry {
    registry: Registry,
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

    /// The unique linked provider for `backend`, if present.
    ///
    /// Returns `None` when there is no matching provider or when multiple
    /// implementations share an identical idempotent manifest. The latter
    /// identity is admitted by QSL, but this lookup does not choose an
    /// implementation by input order.
    #[must_use]
    pub fn provider(&self, backend: &BackendId) -> Option<&dyn Provider> {
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

impl Deref for AdmittedRegistry {
    type Target = Registry;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

/// Build the registry from `manifests`.
///
/// Every manifest is admitted and registered. When any registration is
/// refused (an absent or unknown kind or mode, or a duplicate backend
/// identity), the run is refused with every refusal, ordered by backend
/// identity then manifest digest.
///
/// `Registry::register`'s own `Err` is what that one call caused or
/// observed, not a diff against the registry's prior state, so it can
/// repeat a conflict an earlier call in this same loop already reported.
/// Rather than accumulate those (and dedup them by hand), this collects
/// only the vocabulary refusals `BackendDescriptor::admit` produces during
/// the loop, then reads every duplicate-backend conflict once, after the
/// loop, from `Registry::refusals` -- the registry's own stable, deduped
/// account of every identity conflict it currently holds.
pub fn build_registry(manifests: &[ProviderManifest]) -> Result<AdmittedRegistry, DriverRefusal> {
    let mut registry = Registry::new();
    let mut refusals = Vec::new();
    for manifest in manifests {
        let backend_id = BackendId::from_wire(&manifest.identity)
            .map_err(DriverRefusal::EmptyBackendIdentity)?;
        let backend = Candidate::new(backend_id);
        let admitted = BackendDescriptor::admit(
            backend,
            ProviderOrigin::Linked,
            manifest.manifest_digest,
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
        );
        match admitted {
            Ok(descriptor) => {
                // The conflict is read back, deduped, from `refusals()`
                // below; a successful registration needs nothing here.
                let _ = registry.register(descriptor);
            }
            Err(refusal) => refusals.push(refusal),
        }
    }
    refusals.extend(registry.refusals());
    if refusals.is_empty() {
        Ok(AdmittedRegistry { registry })
    } else {
        refusals.sort();
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
    use qsl_route::{BackendId, ManifestDigest, ProviderOrigin};

    use crate::{KaniProvider, Provider, ProviderManifest};

    use super::build_linked_providers;

    fn provider(identity: &str, digest: u8) -> KaniProvider {
        KaniProvider::new(ProviderManifest {
            identity: identity.to_owned(),
            manifest_digest: ManifestDigest::from_digest([digest; 32]),
            advertises: Vec::new(),
        })
    }

    /// Trace: FR-005-AC-3.
    #[test]
    fn linked_manifests_admit_distinct_identities_and_resolve_provider_lookup() {
        let first = provider("agent-ix/provider/first", 1);
        let second = provider("agent-ix/provider/second", 2);
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

    /// Trace: FR-005-AC-1, FR-005-AC-4.
    #[test]
    fn identical_linked_manifests_are_idempotent_without_order_dependent_lookup() {
        let first = provider("agent-ix/provider/same", 1);
        let second = provider("agent-ix/provider/same", 1);
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
