// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`drive`]: the driver's steps, in ADR-011 T-13 order.

use std::collections::BTreeMap;

use qsl_foundation::digest::WireNodeId;
use qsl_foundation::SourceIdentity;
use qsl_package::{CheckedPackage, EmittedPackage};
use qsl_replay::spine::SpineLimits;
use qsl_route::request::{ExtentClassification, RequestWriter};
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{BackendId, Candidate, CandidateOutcome, ManifestDigest, Mode, Registry};
use qsl_semantics::family::Requirements;
use quire_contract_codegen as cg;
use quire_contract_codegen::{BackendKind, ItemSettlement};

use crate::{DriverFault, DriverRefusal, Upstream};

/// The unit to compile, and the inputs spine `compile` takes with it.
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
}

/// One item to negotiate: a checked item's requirement record (ADR-011
/// §2.1 E7, ADR-012 §13.5) and the backend the request names, if any.
///
/// `qsl-package` does not yet expose the per-item requirement records of a
/// `CheckedPackage`, so the caller supplies them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestedItem {
    /// The checked node the item requests a claim over.
    pub node: WireNodeId,
    /// The item's capability kind and extent.
    pub requirements: Requirements,
    /// The backend the request names, if any.
    pub backend: Option<BackendId>,
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

/// A completed run.
#[derive(Debug)]
pub struct Driven {
    /// The S4 in-process package.
    pub package: CheckedPackage,
    /// The E4 wire output: v2 bytes and `package_id`.
    pub emitted: EmittedPackage,
    /// CG's settlement of every requested item, in request order.
    pub settlements: Vec<ItemSettlement>,
    /// Every item settled `supported`, with its routed backend, in request
    /// order.
    pub routed: Vec<Routed>,
}

/// Run one request through the driver's steps (see the crate docs).
///
/// `registry` is the value [`crate::build_registry`] built; `items` are the
/// items to negotiate, in request order.
pub fn drive(
    upstream: &impl Upstream,
    input: CompileInput<'_>,
    registry: &Registry,
    items: &[RequestedItem],
) -> Result<Driven, DriverRefusal> {
    // Steps 1 and 2: S1 to S4, then the E4 emitter.
    let compiled = upstream
        .compile(
            input.source,
            input.path,
            input.bytes,
            input.packages,
            input.limits,
        )
        .map_err(DriverRefusal::Compile)?;

    // T-13 order: the `route` candidate step, then every registered
    // BackendId through CG's one conversion, then E7.
    let (envelope, candidate_sets) = envelope(registry, items)?;
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

    Ok(Driven {
        package: compiled.package,
        emitted: compiled.emitted,
        settlements,
        routed,
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
/// `route` computed them. Types cross as their wire spellings (ADR-013 T-7).
fn envelope(
    registry: &Registry,
    items: &[RequestedItem],
) -> Result<(cg::BackendProviderEnvelope, Vec<Vec<Candidate>>), DriverRefusal> {
    let mut manifest = Vec::new();
    for descriptor in registry.descriptors() {
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
        manifest.push(cg::BackendDescriptor {
            identity: descriptor.id().as_str().to_owned(),
            manifest_digest: digest_wire(descriptor.candidate().manifest_digest()),
            advertised,
            pinned_tool: descriptor.tool().as_str().to_owned(),
        });
    }

    let mut writer = RequestWriter::new();
    let mut request_items = Vec::with_capacity(items.len());
    let mut candidate_sets = Vec::with_capacity(items.len());
    for item in items {
        let index = writer.item(item.node, &item.requirements);
        let extent = writer
            .items()
            .get(index.get())
            .map(|written| cg_extent(written.extent()));
        let label = item.requirements.kind().to_wire();
        let kind = cg::CapabilityKind::from_label(label).map_or_else(
            || cg::RequestedKind::Unknown(label.to_owned()),
            cg::RequestedKind::Known,
        );
        let (candidates, set) =
            match registry.candidates(item.requirements.kind(), item.backend.as_ref()) {
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
        request_items.push(cg::RequestItem {
            kind,
            extent,
            named_backend: item.backend.as_ref().map(|id| id.as_str().to_owned()),
            candidates,
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
    ))
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
            let kind = *kinds
                .get(candidate.id())
                .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?;
            Ok(Routed {
                request_index,
                candidate: candidate.clone(),
                kind,
            })
        })
        .collect()
}

/// A manifest digest's wire spelling: `<domain>:<hex>`.
fn digest_wire(digest: ManifestDigest) -> String {
    format!("{}:{}", digest.domain().as_str(), digest.hex())
}

fn cg_candidate(candidate: &Candidate) -> cg::Candidate {
    cg::Candidate {
        identity: candidate.id().as_str().to_owned(),
        manifest_digest: digest_wire(candidate.manifest_digest()),
    }
}

fn cg_mode(mode: Mode) -> cg::Mode {
    match mode {
        Mode::Bounded => cg::Mode::Bounded,
        Mode::Unbounded => cg::Mode::Unbounded,
    }
}

fn cg_extent(extent: ExtentClassification) -> cg::ExtentClassification {
    match extent {
        ExtentClassification::Bounded => cg::ExtentClassification {
            extent: cg::Mode::Bounded,
            finite_bound_available: false,
        },
        ExtentClassification::Unbounded {
            finite_bound_available,
        } => cg::ExtentClassification {
            extent: cg::Mode::Unbounded,
            finite_bound_available,
        },
    }
}
