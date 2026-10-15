// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`drive`]: the driver's steps, in ADR-011 T-13 order.

use std::collections::BTreeMap;

use qsl_foundation::{BackendId, SourceIdentity};
use qsl_package::EmittedPackage;
use qsl_replay::spine::{CheckedUnit, SpineLimits};
use qsl_route::request::{items_from_requirements, ExtentClassification, RequirementItem};
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{Candidate, CandidateOutcome, Mode, Registry};
use quire_contract_codegen as cg;
use quire_contract_codegen::{BackendKind, ItemSettlement, RoutedGeneration};
use quire_contract_codegen::{ContractDriverResultSelection, ContractRequestPosition};
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageV2, ContractOccurrenceKeyV1,
    ContractPackageV1,
};
use quire_exact::Cancel;

use crate::handoff::handoff;
use crate::{DriverFault, DriverRefusal, Upstream};

/// The unit to compile, and the inputs QSL's typed lifecycle takes with it.
#[derive(Clone, Debug)]
pub struct CompileInput<'a> {
    /// The unit's source identity.
    pub source: SourceIdentity,
    /// The path the unit is displayed as; never opened.
    pub path: &'a str,
    /// The unit's complete-V1 source bytes.
    pub bytes: &'a [u8],
    /// FR-056's package input: domain package documents by `sha256-jcs`
    /// digest.
    pub packages: &'a BTreeMap<[u8; 32], Vec<u8>>,
    /// The S1 to S3 stage limits.
    pub limits: SpineLimits,
    /// The caller-owned QSL cancellation handle. Pass a clone when the caller
    /// must retain the ability to cancel this operation from another thread.
    pub cancel: Cancel,
}

/// One item settled `supported` and routed: the input CG generation takes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Routed {
    /// The item's request index.
    pub request_index: usize,
    /// The routed backend, as `route` returned it.
    pub candidate: Candidate,
    /// That backend's CG kind, from the pre-negotiation conversion.
    pub kind: BackendKind,
}

impl Routed {
    /// The routed `BackendId`.
    pub fn backend(&self) -> &BackendId {
        self.candidate.id()
    }
}

/// Step 7's inputs: one optional context per CG backend kind. A routed
/// item whose kind has no context refuses the run
/// ([`DriverRefusal::Generation`]).
#[derive(Clone, Copy, Debug, Default)]
pub struct GenerationInput<'a> {
    /// The Kani context.
    pub kani: Option<KaniInput<'a>>,
}

/// The Kani generation context, including CG FR-028's proof ceilings.
#[derive(Clone, Copy, Debug)]
pub struct KaniInput<'a> {
    /// Caller-supplied resource ceilings embedded in every generated harness.
    pub ceilings: cg::ProofCeilings,
    /// Rust path of the customer subject.
    pub subject_path: &'a str,
    /// Loop unwind bound.
    pub unwind: u32,
}

/// A completed run.
#[derive(Debug)]
pub struct Driven {
    /// The S4 in-process package.
    pub package: CheckedUnit,
    /// The E4 wire output: v2 bytes and `package_id`.
    pub emitted: EmittedPackage,
    /// E5: the emitted bytes as IR's reader admitted them, at the emitted
    /// `package_id`.
    pub checked: CheckedPackageV2,
    /// CG's settlement of every requested item, in request order.
    pub settlements: Vec<ItemSettlement>,
    /// Every item settled `supported`, with its routed backend, in request
    /// order.
    pub routed: Vec<Routed>,
    /// CG's generation output for the routed items, in ascending request
    /// index. Empty when nothing was routed. Its `claim_map` is the FR-014
    /// claim map CG derived for the Kani group (IR-294), `Some` whenever one
    /// ran; a caller writes it as `claim-map.json`.
    pub generation: RoutedGeneration,
}

/// Run one request through the driver's steps (see the crate docs).
///
/// `registry` is the value [`crate::build_registry`] built. The items to
/// negotiate are not supplied by the caller: they are written from the
/// compiled unit's own checked requirement records
/// (`CheckedPackage::graph().requirements()`, FR-062, FR-075), one item per
/// record in occurrence-key order (`items_from_requirements`). `named` is
/// the backend every item's candidate set is built against, when the
/// request names one; `generation` holds the context of each CG backend
/// kind an item may be routed to.
pub fn drive(
    upstream: &impl Upstream,
    input: CompileInput<'_>,
    registry: &crate::AdmittedRegistry,
    named: Option<&BackendId>,
    generation: GenerationInput<'_>,
) -> Result<Driven, DriverRefusal> {
    // Steps 1 and 2: S1 to S4, then the E4 emitter.
    let (checked_unit, emitted_unit) = upstream.compile(
        input.source.clone(),
        input.path,
        input.bytes,
        input.packages,
        input.limits,
        &input.cancel,
    )?;
    let emitted = emitted_unit.package().clone();
    // E5: IR admits the emitted bytes at the emitted `package_id`.
    let checked = handoff(upstream, &input, &emitted_unit)?;
    // E4's source provision is needed through E5 for diagnostics, then the
    // returned CheckedUnit remains the driver's bounded source owner.
    drop(emitted_unit);

    // FR-075: one item per requirement record, in occurrence-key order.
    let items = items_from_requirements(
        checked_unit.package().graph().requirements(),
        registry,
        named,
    );

    // T-13 order: the `route` candidate step, then every registered
    // BackendId through CG's one conversion, then E7.
    let lowering = lower_requests(&checked, &items)?;
    let (envelope, candidate_sets, descriptors) = envelope(
        registry,
        registry.manifests(),
        &items,
        named,
        Some(&lowering),
    )?;
    let kinds = convert_backends(upstream, registry)?;
    let settlements = upstream
        .negotiate(&envelope)
        .map_err(DriverRefusal::Envelope)?;
    let in_order = settlements.len() == items.len()
        && settlements
            .iter()
            .enumerate()
            .all(|(index, settlement)| settlement.request_index == index);
    if !in_order {
        return Err(DriverRefusal::Fault(DriverFault::SettlementCount {
            requested: items.len(),
            settled: settlements.len(),
        }));
    }

    // The routing step: the `supported` items, and only those.
    let routed = route_supported(upstream, &settlements, &candidate_sets, &kinds)?;

    // Step 7: CG generation for the routed items.
    let generation = generate(upstream, &routed, generation, &lowering, &descriptors)?;

    Ok(Driven {
        package: checked_unit,
        emitted,
        checked,
        settlements,
        routed,
        generation,
    })
}

/// CG generation over `routed`. Each item's node is resolved by lookup in
/// the admitted package, never converted from its wire spelling. Nothing
/// routed means no call.
fn generate(
    upstream: &impl Upstream,
    routed: &[Routed],
    generation: GenerationInput<'_>,
    lowering: &LoweredRequests,
    descriptors: &BTreeMap<String, cg::BackendDescriptor>,
) -> Result<RoutedGeneration, DriverRefusal> {
    if routed.is_empty() {
        return Ok(RoutedGeneration::default());
    }
    let mut generation_items = Vec::with_capacity(routed.len());
    for item in routed {
        let request_index = item.request_index;
        let descriptor = descriptors
            .get(item.candidate.id().as_str())
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        let position_index = lowering
            .driver_position_indexes
            .get(request_index)
            .copied()
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        let position = lowering
            .positions
            .get(position_index)
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        let mut routed_item = cg::RoutedGenerationItem::from_descriptor(
            request_index,
            position.node_id().clone(),
            cg_candidate(&item.candidate),
            descriptor,
        )
        .map_err(|_| DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        if let ContractRequestPosition::DriverItem {
            result_selection, ..
        } = position
        {
            routed_item =
                routed_item.with_scalar_selection(&lowering.contract, result_selection.clone());
        }
        generation_items.push(routed_item);
    }
    let contexts = cg::GenerationContexts {
        kani: generation.kani.map(|kani| cg::KaniGenerationContext {
            ceilings: kani.ceilings,
            subject_path: kani.subject_path,
            unwind: kani.unwind,
        }),
    };
    upstream
        .generate(
            &lowering.contract,
            &lowering.positions,
            &generation_items,
            &contexts,
        )
        .map_err(DriverRefusal::Generation)
}

struct LoweredRequests {
    checked: CheckedPackageV2,
    contract: ContractPackageV1,
    positions: Vec<ContractRequestPosition>,
    request_nodes: Vec<CheckedNodeId>,
    driver_position_indexes: Vec<usize>,
}

fn lower_requests(
    checked: &CheckedPackageV2,
    items: &[RequirementItem],
) -> Result<LoweredRequests, DriverRefusal> {
    let by_digest: BTreeMap<&str, &CheckedNodeId> = checked
        .graph()
        .nodes
        .iter()
        .map(|node| (node.node_id.digest.as_ref(), &node.node_id))
        .collect();
    let mut roots = Vec::with_capacity(items.len() * 2);
    let mut positions = Vec::with_capacity(items.len() * 2);
    let mut request_nodes = Vec::with_capacity(items.len());
    let mut driver_position_indexes = Vec::with_capacity(items.len());
    for item in items {
        let application = by_digest
            .get(item.node().to_string().as_str())
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        let app_occurrence = checked_occurrence(checked, item.occurrence(), application)
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        roots.push((*application).clone());
        let conversion = item
            .conversion_context()
            .map(|occurrence| {
                let node = by_digest
                    .get(occurrence.node().to_string().as_str())
                    .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
                let key = checked_occurrence(checked, occurrence, node)
                    .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
                Ok::<_, DriverRefusal>(((*node).clone(), key))
            })
            .transpose()?;
        let result_selection = if item.kind().to_wire() == "value-validity" {
            ContractDriverResultSelection::OwnResult {
                application: app_occurrence,
                conversion: conversion
                    .as_ref()
                    .map(|(_, conversion)| conversion.clone()),
            }
        } else {
            ContractDriverResultSelection::NotApplicable
        };
        driver_position_indexes.push(positions.len());
        positions.push(ContractRequestPosition::DriverItem {
            node_id: (*application).clone(),
            result_selection,
        });
        request_nodes.push((*application).clone());
        if let Some((conversion_node, _)) = conversion {
            roots.push(conversion_node.clone());
            positions.push(ContractRequestPosition::ContextRoot {
                node_id: conversion_node,
            });
        }
    }
    let lowered = checked.lower(&roots, &cg::routed_contract_lowering_profile());
    let contract = lowered
        .package
        .wire()
        .cloned()
        .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
    if contract.wire().package.source.package_id != *checked.package_id() {
        return Err(DriverRefusal::Fault(DriverFault::RoutingTarget));
    }
    Ok(LoweredRequests {
        checked: checked.clone(),
        contract,
        positions,
        request_nodes,
        driver_position_indexes,
    })
}

fn checked_occurrence(
    checked: &CheckedPackageV2,
    occurrence: &qsl_foundation::source::provenance::OccurrenceKey,
    node_id: &CheckedNodeId,
) -> Option<ContractOccurrenceKeyV1> {
    let role = match occurrence.origin().role().as_str() {
        "declaration" => CheckedOccurrenceRole::Declaration,
        "type" => CheckedOccurrenceRole::Type,
        "expression" => CheckedOccurrenceRole::Expression,
        "anchor" => CheckedOccurrenceRole::Anchor,
        "claim" => CheckedOccurrenceRole::Claim,
        "generated" => CheckedOccurrenceRole::Generated,
        _ => return None,
    };
    let ordinal = occurrence.origin().ordinal();
    checked
        .source_map()
        .iter()
        .any(|entry| &entry.node_id == node_id && entry.role == role && entry.ordinal == ordinal)
        .then(|| ContractOccurrenceKeyV1 {
            node_id: node_id.clone(),
            role,
            ordinal,
        })
}

/// MD-2: convert every registered `BackendId` to its CG kind, or refuse
/// naming every unconverted id in bytewise order.
fn convert_backends(
    upstream: &impl Upstream,
    registry: &Registry,
) -> Result<BTreeMap<BackendId, BackendKind>, DriverRefusal> {
    let mut kinds = BTreeMap::new();
    let mut unconverted = Vec::new();
    for id in registry.backends() {
        match upstream.backend_kind(id) {
            Some(kind) => {
                kinds.insert(id.clone(), kind);
            }
            None => unconverted.push(id.clone()),
        }
    }
    if unconverted.is_empty() {
        Ok(kinds)
    } else {
        unconverted.sort_by(|a, b| a.as_str().as_bytes().cmp(b.as_str().as_bytes()));
        Err(DriverRefusal::UnknownBackend {
            backends: unconverted,
        })
    }
}

/// The FR-331 envelope CG negotiates over, and each item's candidates as
/// `items_from_requirements` computed them against the registry for `named`.
/// Types cross as their wire spellings (ADR-013 T-7).
type EnvelopeProjection = (
    cg::BackendProviderEnvelope,
    Vec<Vec<Candidate>>,
    BTreeMap<String, cg::BackendDescriptor>,
);

fn envelope(
    registry: &Registry,
    manifests: &[crate::ProviderManifest],
    items: &[RequirementItem],
    named: Option<&BackendId>,
    lowering: Option<&LoweredRequests>,
) -> Result<EnvelopeProjection, DriverRefusal> {
    let manifest = registry
        .descriptors()
        .map(|descriptor| {
            let retained = manifests
                .iter()
                .find(|manifest| {
                    manifest.identity == descriptor.id().as_str()
                        && manifest.manifest_digest == descriptor.manifest_digest()
                })
                .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
            project_descriptor(descriptor, retained)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let descriptors = manifest
        .iter()
        .cloned()
        .map(|descriptor| (descriptor.identity.clone(), descriptor))
        .collect();

    let mut request_items = Vec::with_capacity(items.len());
    let mut candidate_sets = Vec::with_capacity(items.len());
    for (request_index, item) in items.iter().enumerate() {
        let lowering = lowering.ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
        let label = item.kind().to_wire();
        let kind = cg::CapabilityKind::from_label(label).map_or_else(
            || cg::RequestedKind::Unknown(label.to_owned()),
            cg::RequestedKind::Known,
        );
        let (candidates, set) = match item.candidates() {
            CandidateOutcome::Candidates(set) => (
                cg::Candidates::Set(set.candidates().iter().map(cg_candidate).collect()),
                set.candidates().to_vec(),
            ),
            CandidateOutcome::UnknownBackend(id) => (
                cg::Candidates::UnknownBackend(id.as_str().to_owned()),
                Vec::new(),
            ),
        };
        candidate_sets.push(set);
        let kani_form = if item.kind().to_wire() == "value-validity" {
            cg::KaniForm::V2ScalarClaim {
                package: Box::new(lowering.checked.clone()),
                contract: Box::new(lowering.contract.clone()),
                positions: lowering.positions.clone(),
                request_index,
                node_id: lowering
                    .request_nodes
                    .get(request_index)
                    .cloned()
                    .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?,
            }
        } else {
            cg::KaniForm::OutsideSet {
                package: Box::new(lowering.checked.clone()),
                node_id: lowering
                    .request_nodes
                    .get(request_index)
                    .cloned()
                    .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?,
            }
        };
        request_items.push(cg::RequestItem {
            kind,
            extent: Some(cg_extent(item)),
            named_backend: named.map(|id| id.as_str().to_owned()),
            candidates,
            kani_form,
        });
    }

    Ok((
        cg::BackendProviderEnvelope {
            contract_version: cg::BACKEND_PROVIDER_CONTRACT.to_owned(),
            capability_vocabulary: cg::CAPABILITY_VOCABULARY.to_owned(),
            manifest,
            items: request_items,
        },
        candidate_sets,
        descriptors,
    ))
}

/// Project one QSL registry descriptor to CG's FR-331 envelope descriptor.
///
/// QSL layer R owns provider origin; this conversion copies its closed
/// vocabulary exhaustively without inferring it from the backend identity or
/// provider bytes (QSL ADR-029 PV-1, CG FR-019-AC-15).
fn project_descriptor(
    descriptor: &qsl_route::BackendDescriptor,
    retained: &crate::ProviderManifest,
) -> Result<cg::BackendDescriptor, DriverRefusal> {
    let mut advertised = Vec::new();
    for (kind, mode) in descriptor.advertises() {
        let label = kind.to_wire();
        let kind = cg::CapabilityKind::from_label(label).ok_or_else(|| {
            DriverRefusal::Fault(DriverFault::VocabularyMismatch {
                label: label.to_owned(),
            })
        })?;
        advertised.push((kind, cg_mode(mode)));
    }
    advertised.sort();

    let origin = match descriptor.origin() {
        qsl_route::ProviderOrigin::Linked => cg::ProviderOrigin::Linked,
        qsl_route::ProviderOrigin::Process => cg::ProviderOrigin::Process,
    };

    let wire = retained
        .wire
        .as_deref()
        .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
    let fields: RetainedManifestFields = serde_json::from_slice(wire)
        .map_err(|_| DriverRefusal::Fault(DriverFault::RoutingTarget))?;

    Ok(cg::BackendDescriptor {
        identity: descriptor.id().as_str().to_owned(),
        advertised,
        origin,
        domains: descriptor.domains().cloned(),
        bounds: fields.bounds,
    })
}

#[derive(serde::Deserialize)]
struct RetainedManifestFields {
    #[serde(default, deserialize_with = "deserialize_present")]
    bounds: Option<BTreeMap<String, u64>>,
}

fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// The routing step over the `supported` settlements only.
fn route_supported(
    upstream: &impl Upstream,
    settlements: &[ItemSettlement],
    candidate_sets: &[Vec<Candidate>],
    kinds: &BTreeMap<BackendId, BackendKind>,
) -> Result<Vec<Routed>, DriverRefusal> {
    let mut indexes = Vec::new();
    let mut dispositions = Vec::new();
    for settlement in settlements {
        let cg::Disposition::Supported { backend } = &settlement.disposition else {
            continue;
        };
        let index = settlement.request_index;
        let candidate = candidate_sets
            .get(index)
            .and_then(|set| set.iter().find(|c| c.id().as_str() == backend))
            .ok_or_else(|| {
                DriverRefusal::Fault(DriverFault::SupportedOutsideCandidates {
                    request_index: index,
                    backend: backend.clone(),
                })
            })?;
        indexes.push(index);
        dispositions.push(RouteDisposition::Supported(candidate.clone()));
    }
    if dispositions.is_empty() {
        return Ok(Vec::new());
    }
    let targets = upstream.route(&dispositions);
    if targets.len() != dispositions.len() {
        return Err(DriverRefusal::Fault(DriverFault::RoutingTarget));
    }
    indexes
        .into_iter()
        .zip(targets)
        .map(|(request_index, target)| {
            let candidate = target.ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
            let kind = kinds
                .get(candidate.id())
                .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?
                .clone();
            Ok(Routed {
                request_index,
                candidate: candidate.clone(),
                kind,
            })
        })
        .collect()
}

fn cg_candidate(candidate: &Candidate) -> cg::Candidate {
    cg::Candidate {
        identity: candidate.id().as_str().to_owned(),
    }
}

fn cg_mode(mode: Mode) -> cg::Mode {
    match mode {
        Mode::Bounded => cg::Mode::Bounded,
        Mode::Unbounded => cg::Mode::Unbounded,
    }
}

fn cg_extent(item: &RequirementItem) -> cg::ExtentClassification {
    let extent = item.extent();
    // `items_from_requirements` creates initial request items only. It does not
    // call QSL's `RequestWriter::bounded_item`, so no substituted proof-bound
    // values exist to project here; the QSL `unbounded` view supplies the exact
    // domain set and substituted bounds remain empty.
    let domains = item
        .unbounded()
        .iter()
        .map(|(domain, kind)| cg::ExtentDomain {
            domain: domain.clone(),
            kind: *kind,
        })
        .collect();
    match extent {
        ExtentClassification::Bounded => cg::ExtentClassification {
            extent: cg::Mode::Bounded,
            finite_bound_available: false,
            domains,
            bounds: Vec::new(),
        },
        ExtentClassification::Unbounded {
            finite_bound_available,
        } => cg::ExtentClassification {
            extent: cg::Mode::Unbounded,
            finite_bound_available,
            domains,
            bounds: Vec::new(),
        },
    }
}

#[cfg(test)]
mod origin_projection_tests {
    use qsl_foundation::BackendId;
    use qsl_route::{
        descriptor_from_manifest, BackendDescriptor, Candidate, ManifestDigest, ProviderOrigin,
        Registry,
    };

    use quire_contract_codegen as cg;

    use super::{envelope, project_descriptor};
    use crate::ProviderManifest;
    use std::collections::BTreeMap;
    use std::sync::Arc;

    fn descriptor(origin: ProviderOrigin, identity: &str) -> BackendDescriptor {
        let candidate =
            Candidate::new(BackendId::from_wire(identity).expect("identity is nonempty"));
        descriptor_from_manifest(
            candidate,
            origin,
            ManifestDigest::from_digest([7; 32]),
            [(Some("value-validity"), Some("bounded"))],
            Some(&["integer"]),
        )
        .expect("QSL accepts the typed descriptor")
    }

    fn registry(origin: ProviderOrigin) -> Registry {
        let mut registry = Registry::new();
        registry
            .register(descriptor(origin, "example-provider"))
            .expect("one descriptor registers");
        registry
    }

    fn snapshot(descriptor: &BackendDescriptor) -> ProviderManifest {
        ProviderManifest {
            identity: descriptor.id().as_str().to_owned(),
            manifest_digest: descriptor.manifest_digest(),
            advertises: descriptor
                .advertises()
                .map(|(kind, mode)| {
                    (
                        Some(kind.to_wire().to_owned()),
                        Some(mode.to_wire().to_owned()),
                    )
                })
                .collect(),
            domains: descriptor.domains().map(|domains| {
                domains
                    .iter()
                    .map(|kind| kind.to_wire().to_owned())
                    .collect()
            }),
            wire: Some(Arc::from(
                serde_json::to_vec(&serde_json::json!({
                    "backend": descriptor.id().as_str(),
                    "advertises": descriptor.advertises().map(|(kind, mode)| serde_json::json!({
                        "kind": kind.to_wire(), "mode": mode.to_wire()
                    })).collect::<Vec<_>>(),
                    "domains": descriptor.domains().map(|domains| domains.iter().map(|kind| kind.to_wire()).collect::<Vec<_>>())
                }))
                .expect("test descriptor encodes"),
            )),
        }
    }

    /// Trace: FR-005-AC-5, TC-012.
    #[test]
    fn qsl_origins_project_exhaustively_without_changing_descriptor_members() {
        let linked_registry = registry(ProviderOrigin::Linked);
        let process_registry = registry(ProviderOrigin::Process);
        let linked = project_descriptor(
            linked_registry
                .descriptors()
                .next()
                .expect("one linked descriptor"),
            &snapshot(
                linked_registry
                    .descriptors()
                    .next()
                    .expect("linked descriptor"),
            ),
        )
        .expect("linked projection succeeds");
        let process = project_descriptor(
            process_registry
                .descriptors()
                .next()
                .expect("one process descriptor"),
            &snapshot(
                process_registry
                    .descriptors()
                    .next()
                    .expect("process descriptor"),
            ),
        )
        .expect("process projection succeeds");

        assert_eq!(linked.identity, "example-provider");
        assert_eq!(linked.identity, process.identity);
        assert_eq!(linked.advertised, process.advertised);
        assert_eq!(
            linked.advertised,
            vec![(cg::CapabilityKind::ValueValidity, cg::Mode::Bounded)]
        );
        assert_eq!(linked.origin, cg::ProviderOrigin::Linked);
        assert_eq!(process.origin, cg::ProviderOrigin::Process);
        assert_eq!(linked.bounds, None);
    }

    /// Trace: FR-019-AC-15. The lossless FR-331 descriptor distinguishes a missing bounds
    /// member from a present empty object; projection preserves both shapes.
    #[test]
    fn descriptor_projection_preserves_absent_and_present_empty_bounds() {
        let registry = registry(ProviderOrigin::Linked);
        let descriptor = registry
            .descriptors()
            .next()
            .expect("registered descriptor");
        let absent = project_descriptor(descriptor, &snapshot(descriptor)).expect("absent bounds");
        let mut present_empty = snapshot(descriptor);
        let mut wire: serde_json::Value =
            serde_json::from_slice(present_empty.wire.as_deref().expect("retained wire"))
                .expect("test wire parses");
        wire.as_object_mut()
            .expect("object")
            .insert("bounds".to_owned(), serde_json::json!({}));
        present_empty.wire = Some(Arc::from(
            serde_json::to_vec(&wire).expect("test wire encodes"),
        ));
        let present = project_descriptor(descriptor, &present_empty).expect("empty bounds");
        assert_eq!(absent.bounds, None);
        assert_eq!(present.bounds, Some(BTreeMap::new()));
    }

    /// Trace: FR-005-AC-5, TC-012; every registry descriptor reaches CG once.
    #[test]
    fn envelope_projects_each_registered_descriptor_once_with_its_origin() {
        let expected = [
            (
                "example/linked",
                ProviderOrigin::Linked,
                cg::ProviderOrigin::Linked,
            ),
            (
                "example/process",
                ProviderOrigin::Process,
                cg::ProviderOrigin::Process,
            ),
        ];
        let mut registry = Registry::new();
        let mut snapshots = Vec::new();
        for (identity, qsl_origin, _) in expected {
            let descriptor = descriptor(qsl_origin, identity);
            snapshots.push(snapshot(&descriptor));
            registry
                .register(descriptor)
                .expect("distinct descriptors register");
        }

        let (envelope, _, _) =
            envelope(&registry, &snapshots, &[], None, None).expect("envelope projects");
        assert_eq!(envelope.manifest.len(), registry.descriptors().count());
        let mut expected_identities: Vec<String> = expected
            .iter()
            .map(|(identity, _, _)| (*identity).to_owned())
            .collect();
        expected_identities.sort();
        let mut projected_identities: Vec<String> = envelope
            .manifest
            .iter()
            .map(|projected| projected.identity.clone())
            .collect();
        projected_identities.sort();
        assert_eq!(projected_identities, expected_identities);
        for (identity, _, origin) in &expected {
            let projected: Vec<_> = envelope
                .manifest
                .iter()
                .filter(|projected| projected.identity == *identity)
                .collect();
            assert_eq!(projected.len(), 1, "each identity appears once");
            assert_eq!(projected[0].origin, *origin);
        }
    }
}
