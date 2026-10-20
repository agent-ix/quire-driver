// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`drive`]: the driver's steps, in ADR-011 T-13 order.

use std::collections::BTreeMap;
use std::path::Path;

use qsl_foundation::{BackendId, SourceIdentity};
use qsl_package::EmittedPackage;
use qsl_replay::spine::{
    AdmittedModels, BindOutcome, Call, CheckedUnit, ScalarOccurrence, SpineLimits,
};
use qsl_route::request::{items_from_requirements, ExtentClassification, RequirementItem};
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{Candidate, CandidateOutcome, Mode, Registry};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_contract_codegen as cg;
use quire_contract_codegen::{
    BackendKind, ItemSettlement, KaniInstallation, KaniScalarNativeRequest, RoutedGeneration,
};
use quire_contract_model::{CheckedNodeId, CheckedPackageV2};
use quire_exact::Cancel;

use crate::handoff::handoff;
use crate::{AdmittedRegistry, DriverFault, DriverRefusal, Upstream};

macro_rules! native_limit_fields_match {
    ($left:expr, $right:expr) => {{
        let left = $left;
        let right = $right;
        left.integer_bits == right.integer_bits
            && left.decimal_digits == right.decimal_digits
            && left.scale_expansion == right.scale_expansion
            && left.text_input_bytes == right.text_input_bytes
            && left.text_scalars == right.text_scalars
            && left.normalized_scalars == right.normalized_scalars
            && left.unit_edges == right.unit_edges
            && left.value_occurrences == right.value_occurrences
            && left.work_units == right.work_units
            && left.result_units == right.result_units
    }};
}

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
    /// Optional caller input that binds selected scalar Kani harnesses to
    /// one unchanged QSL call and executes each generated artifact natively.
    pub scalar_execution: Option<ScalarExecutionInput<'a>>,
}

/// Caller-owned values required to bind and execute a routed scalar harness.
#[derive(Clone, Copy, Debug)]
pub struct ScalarExecutionInput<'a> {
    /// The exact QSL call selected by the caller.
    pub call: &'a Call,
    /// Caller-selected QSL replay limits retained for the later replay step.
    pub replay_limits: qsl_replay::ReplayLimits,
    /// The same object environment supplied to the QSL call.
    pub environment: &'a ObjectEnvironment,
    /// Native runner settings, when this call also requests same-artifact
    /// proof and execution. Omitting these retains the bound production
    /// context for a later runner step.
    pub native: Option<ScalarNativeInput<'a>>,
}

/// Inputs for CG's same-artifact native scalar runner.
#[derive(Clone, Copy, Debug)]
pub struct ScalarNativeInput<'a> {
    /// Installed bounded Kani launcher used for the generated harness.
    pub installation: &'a KaniInstallation,
    /// Caller-owned target directory; CG stages the executed package privately.
    pub target_directory: &'a Path,
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
    package: CheckedUnit,
    /// The E4 wire output: v2 bytes and `package_id`.
    emitted: EmittedPackage,
    /// E5: the emitted bytes as IR's reader admitted them, at the emitted
    /// `package_id`.
    checked: CheckedPackageV2,
    /// CG's settlement of every requested item, in request order.
    pub settlements: Vec<ItemSettlement>,
    /// The exact QSL-produced request items used to construct the CG envelope.
    /// The CLI serializes these same values into the FR-331 process request.
    pub request_items: Vec<RequirementItem>,
    /// Every item settled `supported`, with its routed backend, in request
    /// order.
    pub routed: Vec<Routed>,
    /// CG's generation output for the routed items, in ascending request
    /// index. Empty when nothing was routed. Its `claim_map` is the FR-014
    /// claim map CG derived for the Kani group (IR-294), `Some` whenever one
    /// ran; a caller writes it as `claim-map.json`.
    generation: RoutedGeneration,
    /// Each native scalar execution, keyed to the exact routed request item.
    scalar_executions: Vec<ScalarExecution>,
}

impl Driven {
    /// The compiled in-process QSL package.
    pub fn package(&self) -> &CheckedUnit {
        &self.package
    }

    /// The emitted package bytes and identity.
    pub fn emitted(&self) -> &EmittedPackage {
        &self.emitted
    }

    /// The admitted emitted package.
    pub fn checked(&self) -> &CheckedPackageV2 {
        &self.checked
    }

    /// The immutable generation result produced from the admitted package.
    pub fn generation(&self) -> &RoutedGeneration {
        &self.generation
    }

    /// Native scalar executions retained for later replay composition.
    ///
    /// The collection is read-only so callers cannot detach or replace a
    /// verified execution context after the driver has authenticated it.
    pub fn scalar_executions(&self) -> &[ScalarExecution] {
        &self.scalar_executions
    }
}

/// The call, occurrence binding, and CG's same-artifact native receipt for a
/// routed scalar item. This is retained for a later replay consumer; it does
/// not settle or replay the outcome.
pub struct ScalarExecution {
    /// The driver's original request index.
    request_index: usize,
    /// The exact QSL call cloned from the caller's input.
    call: Call,
    /// The object environment used by the same QSL binding call.
    environment: ObjectEnvironment,
    /// The caller's original cancellation handle, shared with QSL and CG.
    cancel: Cancel,
    /// Exact QSL replay limits retained alongside the native observation.
    replay_limits: qsl_replay::ReplayLimits,
    /// The QSL requirement occurrence selected for this routed item.
    occurrence: ScalarOccurrence,
    /// Concrete QSL-bound operands in the generated harness argument order.
    operands: qsl_replay::spine::BoundScalarOperands,
    /// CG's opaque proof/native receipt for the executed immutable snapshot.
    receipt: Option<cg::KaniScalarNativeReceipt>,
}

impl ScalarExecution {
    /// Original request index associated with this execution.
    pub fn request_index(&self) -> usize {
        self.request_index
    }

    /// The exact caller call retained during binding and native execution.
    pub fn call(&self) -> &Call {
        &self.call
    }

    /// Object environment retained during binding.
    pub fn environment(&self) -> &ObjectEnvironment {
        &self.environment
    }

    /// Shared cancellation handle used for the request and native execution.
    pub fn cancel(&self) -> &Cancel {
        &self.cancel
    }

    /// Caller replay limits retained with this execution.
    pub fn replay_limits(&self) -> qsl_replay::ReplayLimits {
        self.replay_limits
    }

    /// Exact occurrence selected for this request item.
    pub fn occurrence(&self) -> &ScalarOccurrence {
        &self.occurrence
    }

    /// QSL-bound operands in generated harness order.
    pub fn operands(&self) -> &qsl_replay::spine::BoundScalarOperands {
        &self.operands
    }

    /// CG's opaque receipt, when native execution was requested.
    pub fn receipt(&self) -> Option<&cg::KaniScalarNativeReceipt> {
        self.receipt.as_ref()
    }
}

impl std::fmt::Debug for ScalarExecution {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScalarExecution")
            .field("request_index", &self.request_index)
            .field("call", &self.call)
            .field("environment", &self.environment)
            .field("replay_limits", &self.replay_limits)
            .field("occurrence", &self.occurrence)
            .field("operands", &self.operands)
            .field("receipt", &self.receipt)
            .finish_non_exhaustive()
    }
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
    registry: &AdmittedRegistry,
    named: Option<&BackendId>,
    generation_input: GenerationInput<'_>,
) -> Result<Driven, DriverRefusal> {
    // Steps 1 and 2: S1 to S4, then the E4 emitter.
    let (checked_unit, models, emitted_unit) = upstream.compile(
        input.source.clone(),
        input.path,
        input.bytes,
        input.packages,
        input.limits,
        &input.cancel,
    )?;
    let emitted = emitted_unit.package().clone();
    // E5: IR admits the emitted bytes at the emitted `package_id`.
    let checked = handoff(
        upstream,
        &models,
        &checked_unit,
        &emitted_unit,
        input.limits.imports,
    )?;
    // E4's source provision is needed through E5 for diagnostics, then the
    // returned CheckedUnit remains the driver's bounded source owner.
    drop(emitted_unit);

    // FR-075: one item per requirement record, in occurrence-key order.
    let items = items_from_requirements(
        checked_unit.package().graph().requirements(),
        registry,
        named,
    );
    let lowered = lower_driver_request(&checked, &items)?;

    // T-13 order: the `route` candidate step, then every registered
    // BackendId through CG's one conversion, then E7.
    let manifests = registry.manifest_snapshot();
    let prepared = envelope(registry, manifests, &checked, &items, named, &lowered)?;
    let kinds = convert_backends(upstream, registry, &prepared.generation_descriptors)?;
    let settlements = upstream
        .negotiate(&prepared.envelope)
        .map_err(DriverRefusal::from)?;
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
    let routed = route_supported(upstream, &settlements, &prepared.candidate_sets, &kinds)?;

    // Step 7: CG generation for the routed items.
    let generation = generate(
        upstream,
        &checked,
        &prepared.generation_descriptors,
        &items,
        &routed,
        generation_input,
        &lowered,
    )?;

    let scalar_executions = if let Some(scalar_input) = generation_input.scalar_execution {
        execute_scalar_items(
            &generation,
            &checked_unit,
            &models,
            &checked,
            &input.cancel,
            &items,
            scalar_input,
        )?
    } else {
        Vec::new()
    };

    Ok(Driven {
        package: checked_unit,
        emitted,
        checked,
        settlements,
        request_items: items,
        routed,
        generation,
        scalar_executions,
    })
}

fn execute_scalar_items(
    generation: &RoutedGeneration,
    checked_unit: &CheckedUnit,
    models: &AdmittedModels,
    admitted: &CheckedPackageV2,
    cancel: &Cancel,
    items: &[RequirementItem],
    scalar: ScalarExecutionInput<'_>,
) -> Result<Vec<ScalarExecution>, DriverRefusal> {
    let mut executions = Vec::new();
    for generated in &generation.items {
        let cg::KindOutput::Kani {
            harness: Some(harness),
            ..
        } = &generated.output
        else {
            continue;
        };
        let item = items
            .iter()
            .find(|item| item.index().get() == generated.request_index)
            .ok_or_else(|| DriverRefusal::Fault(DriverFault::NativeReceiptMismatch))?;
        let occurrence = ScalarOccurrence {
            function: scalar.call.function.clone(),
            application: item.node(),
            occurrence: item.occurrence().clone(),
        };
        let bound = match qsl_replay::spine::bind_scalar_operands(
            admitted,
            models,
            checked_unit,
            scalar.call,
            &occurrence,
            scalar.environment,
            cancel,
        )
        .map_err(DriverRefusal::ScalarCall)?
        {
            BindOutcome::Bound(bound) => bound,
            BindOutcome::Refused(refusal) => {
                return Err(DriverRefusal::ScalarBinding(refusal));
            }
        };
        if !bound_operands_match_identity(&bound, &harness.identity.arguments) {
            return Err(DriverRefusal::Fault(DriverFault::NativeReceiptMismatch));
        }
        let operand_values = bound.values();
        let receipt = if let Some(native) = scalar.native {
            let artifacts = generation
                .oracle_artifacts
                .as_deref()
                .ok_or_else(|| DriverRefusal::Fault(DriverFault::NativeReceiptMismatch))?;
            let receipt = cg::execute_kani_scalar_native(&KaniScalarNativeRequest {
                cancel,
                installation: native.installation,
                harness,
                oracle_artifacts: artifacts,
                operands: &operand_values,
                scalar_limits: native_scalar_limits(scalar.call.accounting),
                target_directory: native.target_directory,
            })
            .map_err(DriverRefusal::ScalarNative)?;
            verify_native_receipt(
                &receipt,
                artifacts,
                harness,
                &operand_values,
                scalar.call.accounting,
            )?;
            Some(receipt)
        } else {
            None
        };
        executions.push(ScalarExecution {
            request_index: generated.request_index,
            call: scalar.call.clone(),
            environment: scalar.environment.clone(),
            cancel: cancel.clone(),
            replay_limits: scalar.replay_limits,
            occurrence,
            operands: bound.clone(),
            receipt,
        });
    }
    Ok(executions)
}

fn bound_operands_match_identity(
    bound: &qsl_replay::spine::BoundScalarOperands,
    arguments: &[cg::ScalarObligationArgument],
) -> bool {
    bound.operands().len() == arguments.len()
        && bound.operands().iter().zip(arguments).enumerate().all(
            |(ordinal, (operand, argument))| {
                operand.ordinal == ordinal as u64
                    && i128::from(argument.minimum) == operand.range.lower
                    && i128::from(argument.maximum) == operand.range.upper
            },
        )
}

fn verify_native_receipt(
    receipt: &cg::KaniScalarNativeReceipt,
    artifacts: &[cg::Artifact],
    harness: &cg::KaniScalarObligationHarness,
    operands: &[i64],
    retained_limits: qsl_replay::ScalarLimits,
) -> Result<(), DriverRefusal> {
    if !native_limits_match(receipt, &retained_limits) {
        return Err(DriverRefusal::Fault(
            DriverFault::NativeExecutionLimitsMismatch,
        ));
    }
    if receipt.harness_identity() != &harness.identity
        || receipt.operands() != operands
        || !receipt_driver_bytes_match(receipt.staged_entries(), artifacts, harness)
        || !receipt_digest_matches(receipt)
    {
        return Err(DriverRefusal::Fault(DriverFault::NativeReceiptMismatch));
    }
    Ok(())
}

fn native_limits_match(
    receipt: &cg::KaniScalarNativeReceipt,
    retained: &qsl_replay::ScalarLimits,
) -> bool {
    let returned = receipt.scalar_limits();
    let meter = receipt.meter_observation().limits();
    native_limit_fields_match!(returned, retained) && native_limit_fields_match!(meter, retained)
}

/// Project the caller's QSL limits into CG's public request type while retaining
/// every named counter. CG is adding this type's root re-export on main.
fn native_scalar_limits(limits: qsl_replay::ScalarLimits) -> quire_contract_codegen::ScalarLimits {
    quire_contract_codegen::ScalarLimits {
        integer_bits: limits.integer_bits,
        decimal_digits: limits.decimal_digits,
        scale_expansion: limits.scale_expansion,
        text_input_bytes: limits.text_input_bytes,
        text_scalars: limits.text_scalars,
        normalized_scalars: limits.normalized_scalars,
        unit_edges: limits.unit_edges,
        value_occurrences: limits.value_occurrences,
        work_units: limits.work_units,
        result_units: limits.result_units,
    }
}

#[cfg(test)]
fn qsl_limit_fields_match(
    left: &qsl_replay::ScalarLimits,
    right: &qsl_replay::ScalarLimits,
) -> bool {
    native_limit_fields_match!(left, right)
}

fn receipt_driver_bytes_match(
    staged: &[(String, Vec<u8>)],
    artifacts: &[cg::Artifact],
    harness: &cg::KaniScalarObligationHarness,
) -> bool {
    driver_entries_match(
        staged,
        artifacts,
        &harness.rust.contents,
        &harness.record.path,
        &harness.record.contents,
    )
}

fn driver_entries_match(
    staged: &[(String, Vec<u8>)],
    artifacts: &[cg::Artifact],
    harness_rust: &str,
    harness_record_path: &str,
    harness_record: &str,
) -> bool {
    let find = |path: &str| {
        staged
            .iter()
            .find(|entry| entry.0 == path)
            .map(|entry| &entry.1)
    };
    if find("src/lib.rs").is_none_or(|bytes| bytes != harness_rust.as_bytes())
        || find(harness_record_path).is_none_or(|bytes| bytes != harness_record.as_bytes())
    {
        return false;
    }
    artifacts.iter().all(|artifact| {
        let staged_path = if artifact.path == "src/lib.rs" {
            "src/oracle-artifacts-lib.rs"
        } else {
            &artifact.path
        };
        find(staged_path).is_some_and(|bytes| bytes == artifact.contents.as_bytes())
    })
}

fn receipt_digest_matches(receipt: &cg::KaniScalarNativeReceipt) -> bool {
    recompute_proof_content_digest(receipt.staged_entries()).is_some_and(|(identity, digest)| {
        receipt.proof_content_identity().as_bytes() == &identity
            && receipt.proof_content_digest() == &digest
    })
}

fn recompute_proof_content_digest(
    entries: &[(String, Vec<u8>)],
) -> Option<([u8; 32], qsl_replay::DigestRecord)> {
    use quire_canonical::FixedShape;

    #[derive(serde::Serialize, FixedShape)]
    struct CanonicalArtifact {
        path: String,
        contents: String,
    }
    #[derive(serde::Serialize, FixedShape)]
    struct ScalarProofPreimage {
        schema: String,
        artifacts: Vec<CanonicalArtifact>,
    }

    if entries.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return None;
    }
    let artifacts = entries
        .iter()
        .map(|(path, bytes)| {
            String::from_utf8(bytes.clone()).map(|contents| CanonicalArtifact {
                path: path.clone(),
                contents,
            })
        })
        .collect::<Result<Vec<_>, _>>();
    let Ok(artifacts) = artifacts else {
        return None;
    };
    let preimage = ScalarProofPreimage {
        schema: "agent-ix/cg-kani-scalar-proof-content/v2".to_owned(),
        artifacts,
    };
    let digest =
        quire_canonical::sha256(&preimage, quire_canonical::Limits::new(256 * 1024 * 1024)).ok()?;
    let identity = *digest.as_bytes();
    let expected = qsl_replay::DigestRecord::mint(qsl_replay::DigestDomain::Sha256Jcs, identity);
    Some((identity, expected))
}

/// CG generation over `routed`. Each item's node is resolved by lookup in
/// the admitted package, never converted from its wire spelling. Nothing
/// routed means no call.
fn generate(
    upstream: &impl Upstream,
    checked: &CheckedPackageV2,
    descriptors: &[cg::BackendDescriptor],
    items: &[RequirementItem],
    routed: &[Routed],
    generation: GenerationInput<'_>,
    lowered: &LoweredDriverRequest,
) -> Result<RoutedGeneration, DriverRefusal> {
    if routed.is_empty() {
        return Ok(RoutedGeneration::default());
    }
    let nodes: BTreeMap<&str, &CheckedNodeId> = checked
        .graph()
        .nodes
        .iter()
        .map(|node| (node.node_id.digest.as_ref(), &node.node_id))
        .collect();
    let mut generation_items = Vec::with_capacity(routed.len());
    for item in routed {
        let request_index = item.request_index;
        let node = items
            .get(request_index)
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?
            .node();
        let node_id = nodes
            .get(node.to_string().as_str())
            .ok_or(DriverRefusal::Fault(DriverFault::UnknownNode {
                request_index,
                node,
            }))?;
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.identity == item.candidate.id().as_str())
            .ok_or(DriverRefusal::Fault(DriverFault::AdmittedManifestMissing))?;
        let generation_item = cg::RoutedGenerationItem::from_descriptor(
            request_index,
            (*node_id).clone(),
            cg_candidate(&item.candidate),
            descriptor,
        )
        .map_err(|_| DriverRefusal::Fault(DriverFault::RoutingTarget))?
        .with_scalar_selection(
            &lowered.contract_package,
            lowered
                .result_selections
                .get(request_index)
                .cloned()
                .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?,
        );
        generation_items.push(generation_item);
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
            &lowered.contract_package,
            &lowered.positions,
            &generation_items,
            &contexts,
        )
        .map_err(DriverRefusal::Generation)
}

struct LoweredDriverRequest {
    contract_package: quire_contract_model::ContractPackageV1,
    positions: Vec<cg::ContractRequestPosition>,
    result_selections: Vec<cg::ContractDriverResultSelection>,
}

/// Lower each admitted driver item and its QSL-selected result context once.
/// The same package and positions bind CG negotiation preflight and generation.
fn lower_driver_request(
    checked: &CheckedPackageV2,
    items: &[RequirementItem],
) -> Result<LoweredDriverRequest, DriverRefusal> {
    let mut roots = Vec::new();
    let mut positions = Vec::new();
    let mut result_selections = Vec::with_capacity(items.len());
    for (request_index, item) in items.iter().enumerate() {
        let application = resolve_checked_node(
            checked.graph().nodes.iter().map(|node| &node.node_id),
            item.node(),
            request_index,
        )?
        .clone();
        let application_occurrence =
            contract_occurrence(checked, item.occurrence(), request_index)?;
        let conversion = item
            .conversion_context()
            .map(|occurrence| contract_occurrence(checked, occurrence, request_index))
            .transpose()?;
        let result_selection = cg::ContractDriverResultSelection::OwnResult {
            application: application_occurrence,
            conversion: conversion.clone(),
        };
        roots.push(application.clone());
        positions.push(cg::ContractRequestPosition::DriverItem {
            node_id: application,
            result_selection: result_selection.clone(),
        });
        result_selections.push(result_selection);
        if let Some(conversion) = conversion {
            roots.push(conversion.node_id.clone());
            positions.push(cg::ContractRequestPosition::ContextRoot {
                node_id: conversion.node_id,
            });
        }
    }
    let lowered = checked.lower(&roots, &cg::routed_contract_lowering_profile());
    let Some(contract_package) = lowered.package.wire().cloned() else {
        if let Some(quire_contract_model::CompleteLoweringRecordV2::Failed {
            limit_kind,
            limit,
            consumed,
            ..
        }) = lowered.records.iter().find(|record| {
            matches!(
                record,
                quire_contract_model::CompleteLoweringRecordV2::Failed { .. }
            )
        }) {
            return Err(DriverRefusal::LoweringLimit {
                limit_kind: *limit_kind,
                limit: *limit,
                consumed: *consumed,
            });
        }
        return Err(DriverRefusal::Fault(
            DriverFault::ContractLoweringIncomplete,
        ));
    };
    Ok(LoweredDriverRequest {
        contract_package,
        positions,
        result_selections,
    })
}

fn contract_occurrence(
    checked: &CheckedPackageV2,
    occurrence: &qsl_foundation::source::provenance::OccurrenceKey,
    request_index: usize,
) -> Result<quire_contract_model::ContractOccurrenceKeyV1, DriverRefusal> {
    let node_id = resolve_checked_node(
        checked.graph().nodes.iter().map(|node| &node.node_id),
        occurrence.node(),
        request_index,
    )?
    .clone();
    let role = match occurrence.origin().role().as_str() {
        "declaration" => quire_contract_model::CheckedOccurrenceRole::Declaration,
        "type" => quire_contract_model::CheckedOccurrenceRole::Type,
        "expression" => quire_contract_model::CheckedOccurrenceRole::Expression,
        "anchor" => quire_contract_model::CheckedOccurrenceRole::Anchor,
        "claim" => quire_contract_model::CheckedOccurrenceRole::Claim,
        "generated" => quire_contract_model::CheckedOccurrenceRole::Generated,
        other => {
            return Err(DriverRefusal::Fault(DriverFault::UnknownOccurrenceRole {
                role: other.to_owned(),
            }));
        }
    };
    Ok(quire_contract_model::ContractOccurrenceKeyV1 {
        node_id,
        role,
        ordinal: occurrence.origin().ordinal(),
    })
}

#[cfg(test)]
mod scalar_receipt_tests {
    use super::{driver_entries_match, qsl_limit_fields_match, recompute_proof_content_digest};
    use quire_contract_codegen::Artifact;

    #[test]
    fn proof_digest_uses_sorted_complete_staged_entries_and_fr357_domain() {
        let entries = [
            ("Cargo.lock", "lock\n"),
            ("Cargo.toml", "manifest"),
            ("claim-map.json", "{}"),
            ("kani-obligations/m.json", "record"),
            ("src/bin/ir791_native.rs", "adapter"),
            ("src/lib.rs", "harness"),
            ("src/oracle-artifacts-lib.rs", "oracle"),
        ]
        .into_iter()
        .map(|(path, contents)| (path.to_owned(), contents.as_bytes().to_vec()))
        .collect::<Vec<_>>();
        let (identity, record) = recompute_proof_content_digest(&entries)
            .expect("UTF-8 staged entries produce the canonical record");
        assert_eq!(
            identity,
            [
                0x57, 0x1d, 0x61, 0x99, 0x41, 0x4a, 0xfa, 0xdc, 0x06, 0x57, 0x44, 0xe9, 0xc4, 0x7d,
                0xf0, 0x6f, 0xbe, 0xb1, 0x8e, 0xe3, 0x6c, 0xca, 0x99, 0x5b, 0xa0, 0x29, 0x30, 0x9a,
                0xdd, 0x15, 0xd3, 0x56,
            ]
        );
        assert_eq!(
            record,
            qsl_replay::DigestRecord::mint(qsl_replay::DigestDomain::Sha256Jcs, identity)
        );
        assert!(recompute_proof_content_digest(&[
            ("z".to_owned(), vec![]),
            ("a".to_owned(), vec![]),
        ])
        .is_none());
        assert!(recompute_proof_content_digest(&[("a".to_owned(), vec![0xff])]).is_none());
        let mut changed = entries.clone();
        changed[0].1.push(b'!');
        let (changed_identity, _) = recompute_proof_content_digest(&changed)
            .expect("mutated staged bytes remain valid UTF-8");
        assert_ne!(
            identity, changed_identity,
            "every staged entry is identity-bound"
        );
    }

    #[test]
    fn every_named_limit_is_checked_against_the_retained_qsl_value() {
        let retained = qsl_replay::spine::default_accounting(1_000_000);
        assert!(qsl_limit_fields_match(&retained, &retained));
        for field in 0..10 {
            let mut changed = retained;
            match field {
                0 => changed.integer_bits += 1,
                1 => changed.decimal_digits += 1,
                2 => changed.scale_expansion += 1,
                3 => changed.text_input_bytes += 1,
                4 => changed.text_scalars += 1,
                5 => changed.normalized_scalars += 1,
                6 => changed.unit_edges += 1,
                7 => changed.value_occurrences += 1,
                8 => changed.work_units += 1,
                9 => changed.result_units += 1,
                _ => unreachable!("loop ranges over the ten named limits"),
            }
            assert!(!qsl_limit_fields_match(&changed, &retained));
        }
    }

    #[test]
    fn native_receipt_driver_entries_must_match_retained_bytes() {
        let artifacts = [
            Artifact::new("Cargo.toml", "[package]"),
            Artifact::new("src/lib.rs", "oracle source"),
        ];
        let staged = vec![
            ("Cargo.toml".to_owned(), b"[package]".to_vec()),
            (
                "kani-obligations/example.json".to_owned(),
                b"record".to_vec(),
            ),
            ("src/lib.rs".to_owned(), b"harness source".to_vec()),
            (
                "src/oracle-artifacts-lib.rs".to_owned(),
                b"oracle source".to_vec(),
            ),
        ];
        assert!(driver_entries_match(
            &staged,
            &artifacts,
            "harness source",
            "kani-obligations/example.json",
            "record",
        ));

        let mut changed = staged.clone();
        changed[3].1.push(b'!');
        assert!(!driver_entries_match(
            &changed,
            &artifacts,
            "harness source",
            "kani-obligations/example.json",
            "record",
        ));
    }
}

#[cfg(test)]
mod native_receipt_integration_tests {
    use super::*;
    use qsl_replay::spine::{default_accounting, CallArgument};
    use qsl_route::ManifestDigest;
    use qsl_semantics::value::{CatalogRole, DefinitionLock};
    use quire_contract_codegen::{KaniInstallation, ProofCeilings};
    use quire_exact::{Integer, Value};
    use std::num::NonZeroU64;
    use std::path::PathBuf;

    struct NativeTargetDirectory(PathBuf);

    impl NativeTargetDirectory {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("the system clock follows the Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ir773-native-target-{}-{nonce}",
                std::process::id()
            ));
            std::fs::create_dir(&path).expect("the test owns a fresh native target directory");
            Self(path)
        }
    }

    impl Drop for NativeTargetDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// This Kani-lane test obtains a genuine successful receipt through the
    /// production driver call, then checks the driver's L0 comparison against
    /// a distinct retained context.
    /// Trace: FR-009-AC-12, TC-016.
    #[test]
    #[ignore = "kani lane: paired proof/native execution uses the installed Kani backend"]
    fn successful_native_receipt_is_authenticated_against_retained_limits() {
        let lock = DefinitionLock::pinned();
        let root = lock.entry(CatalogRole::Root);
        let source = format!(
            "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
             function inc using v(x: Int[0, 9]): Int[0, 10] pure {{ x + 1 }}\n",
            root.identity
        );
        let packages = BTreeMap::new();
        let cancel = Cancel::new();
        let call = Call {
            function: "inc".to_owned(),
            arguments: vec![CallArgument {
                parameter: "x".to_owned(),
                value: Value::Integer(Integer::from(4_i64)),
            }],
            accounting: default_accounting(1_000_000),
            supplied: Default::default(),
        };
        let environment = ObjectEnvironment::default();
        let installation = KaniInstallation::discover().expect("cargo-kani is installed");
        let target_directory = NativeTargetDirectory::new();
        let registry = crate::build_registry(&[crate::ProviderManifest {
            identity: "kani".to_owned(),
            manifest_digest: ManifestDigest::from_digest([7; 32]),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("the Kani provider registers");
        let generation = GenerationInput {
            kani: Some(KaniInput {
                ceilings: ProofCeilings {
                    memory_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("positive limit"),
                    wall_clock: std::time::Duration::from_secs(60),
                },
                subject_path: "crate::subject",
                unwind: 1,
            }),
            scalar_execution: Some(ScalarExecutionInput {
                call: &call,
                replay_limits: qsl_replay::ReplayLimits::default(),
                environment: &environment,
                native: Some(ScalarNativeInput {
                    installation: &installation,
                    target_directory: &target_directory.0,
                }),
            }),
        };
        let driven = drive(
            &crate::upstream::QslCg,
            CompileInput {
                source: SourceIdentity::new("agent-ix", "ir-773", "native-receipt", "1"),
                path: "native-receipt.native",
                bytes: source.as_bytes(),
                packages: &packages,
                limits: SpineLimits::default(),
                cancel: cancel.clone(),
            },
            &registry,
            None,
            generation,
        )
        .expect("native proof and receipt succeed");

        let execution = driven
            .scalar_executions()
            .first()
            .expect("the routed scalar was executed");
        let receipt = execution.receipt().expect("the native receipt is retained");
        assert_eq!(receipt.operands(), [4, 1]);
        assert!(native_limits_match(receipt, &call.accounting));
        assert!(receipt.native_outcome().clone().completed().is_some());
        assert!(receipt_digest_matches(receipt));
        let mut changed_artifacts = driven
            .generation()
            .oracle_artifacts
            .as_ref()
            .expect("the routed generation retains its oracle artifacts")
            .clone();
        let changed = changed_artifacts
            .first_mut()
            .expect("the generated oracle package is non-empty");
        changed.contents.push('!');
        assert!(!receipt_driver_bytes_match(
            receipt.staged_entries(),
            &changed_artifacts,
            match &driven.generation().items[0].output {
                cg::KindOutput::Kani {
                    harness: Some(harness),
                    ..
                } => harness,
                _ => panic!("the routed scalar generated a Kani harness"),
            }
        ));

        let mut different_original_limits = call.accounting;
        different_original_limits.result_units += 1;
        let harness = match &driven.generation().items[0].output {
            cg::KindOutput::Kani {
                harness: Some(harness),
                ..
            } => harness,
            _ => panic!("the routed scalar generated a Kani harness"),
        };
        let artifacts = driven
            .generation()
            .oracle_artifacts
            .as_deref()
            .expect("the routed generation retains its oracle artifacts");
        let refusal = verify_native_receipt(
            receipt,
            artifacts,
            harness,
            execution.operands().values().as_slice(),
            different_original_limits,
        )
        .expect_err("a successful L1 receipt cannot be accepted against retained L0");
        assert_eq!(refusal.stage(), crate::DriverStage::Replay);
        assert_eq!(refusal.code(), "runtime_invariant");
        assert_eq!(refusal.kind(), crate::OutcomeKind::Fault);
        assert!(matches!(
            refusal,
            DriverRefusal::Fault(DriverFault::NativeExecutionLimitsMismatch)
        ));
    }
}

/// Convert every QSL-registered provider's projected descriptor to its CG
/// kind, preserving QSL's admitted provider origin.
fn convert_backends(
    upstream: &impl Upstream,
    registry: &Registry,
    descriptors: &[cg::BackendDescriptor],
) -> Result<BTreeMap<BackendId, BackendKind>, DriverRefusal> {
    let mut kinds = BTreeMap::new();
    let mut unconverted = Vec::new();
    for qsl_descriptor in registry.descriptors() {
        let id = qsl_descriptor.id();
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.identity == id.as_str())
            .ok_or(DriverRefusal::Fault(DriverFault::AdmittedManifestMissing))?;
        match upstream.backend_kind(descriptor) {
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
struct PreparedEnvelope {
    envelope: cg::BackendProviderEnvelope,
    candidate_sets: Vec<Vec<Candidate>>,
    generation_descriptors: Vec<cg::BackendDescriptor>,
}

fn envelope(
    registry: &Registry,
    manifests: &[crate::ProviderManifest],
    checked: &CheckedPackageV2,
    items: &[RequirementItem],
    named: Option<&BackendId>,
    lowered: &LoweredDriverRequest,
) -> Result<PreparedEnvelope, DriverRefusal> {
    let manifest = registry
        .descriptors()
        .map(|descriptor| {
            let retained = manifests
                .iter()
                .find(|manifest| {
                    manifest.identity == descriptor.id().as_str()
                        && manifest.manifest_digest == descriptor.manifest_digest()
                })
                .ok_or(DriverRefusal::Fault(DriverFault::AdmittedManifestMissing))?;
            project_descriptor(descriptor, retained.bounds.as_ref())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut request_items = Vec::with_capacity(items.len());
    let mut candidate_sets = Vec::with_capacity(items.len());
    for (request_index, item) in items.iter().enumerate() {
        let extent = Some(cg_extent(item));
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
        let kani_form = project_kani_form(checked, item.node(), request_index, lowered)?;
        request_items.push(cg::RequestItem {
            kind,
            extent,
            named_backend: named.map(|id| id.as_str().to_owned()),
            candidates,
            kani_form,
        });
    }

    let generation_descriptors = manifest.clone();
    Ok(PreparedEnvelope {
        envelope: cg::BackendProviderEnvelope {
            contract_version: cg::BACKEND_PROVIDER_CONTRACT.to_owned(),
            capability_vocabulary: cg::CAPABILITY_VOCABULARY.to_owned(),
            manifest,
            items: request_items,
        },
        candidate_sets,
        generation_descriptors,
    })
}

/// Project the admitted checked node selected by QSL into CG's typed Kani
/// source form. The form is source evidence, never a driver classification.
fn project_kani_form(
    checked: &CheckedPackageV2,
    node: qsl_foundation::digest::WireNodeId,
    request_index: usize,
    lowered: &LoweredDriverRequest,
) -> Result<cg::KaniForm, DriverRefusal> {
    let selected = resolve_checked_node(
        checked
            .graph()
            .nodes
            .iter()
            .map(|candidate| &candidate.node_id),
        node,
        request_index,
    )?;
    Ok(cg::KaniForm::V2ScalarClaim {
        package: Box::new(checked.clone()),
        contract: Box::new(lowered.contract_package.clone()),
        positions: lowered.positions.clone(),
        request_index,
        node_id: selected.clone(),
    })
}

fn resolve_checked_node<'a>(
    nodes: impl Iterator<Item = &'a CheckedNodeId>,
    node: qsl_foundation::digest::WireNodeId,
    request_index: usize,
) -> Result<&'a CheckedNodeId, DriverRefusal> {
    let digest = node.to_string();
    let mut matches = nodes.filter(|candidate| candidate.digest.as_ref() == digest);
    let Some(selected) = matches.next() else {
        return Err(DriverRefusal::Fault(DriverFault::UnknownNode {
            request_index,
            node,
        }));
    };
    if matches.next().is_some() {
        return Err(DriverRefusal::Fault(DriverFault::AmbiguousNode {
            request_index,
            node,
        }));
    }
    Ok(selected)
}

/// Project one QSL registry descriptor to CG's FR-331 envelope descriptor.
///
/// QSL layer R owns provider origin; this conversion copies its closed
/// vocabulary exhaustively without inferring it from the backend identity or
/// provider bytes (QSL ADR-029 PV-1, CG FR-019-AC-15).
fn project_descriptor(
    descriptor: &qsl_route::BackendDescriptor,
    bounds: Option<&BTreeMap<String, u64>>,
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

    Ok(cg::BackendDescriptor {
        identity: descriptor.id().as_str().to_owned(),
        advertised,
        origin,
        domains: descriptor.domains().cloned(),
        bounds: bounds.cloned(),
    })
}

#[cfg(test)]
mod kani_form_projection_tests {
    use qsl_foundation::digest::WireNodeId;
    use quire_contract_model::CheckedNodeId;

    use super::resolve_checked_node;
    use crate::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};

    fn checked_node(domain: &str, wire: &WireNodeId) -> CheckedNodeId {
        CheckedNodeId {
            domain: domain.into(),
            digest: wire.to_string().into(),
        }
    }

    /// Trace: FR-001-AC-12, TC-006.
    #[test]
    fn selected_wire_node_resolves_to_its_exact_checked_identity() {
        let wire = WireNodeId::from_digest([7; 32]);
        let expected = checked_node("quire.checked-semantic-node/v1", &wire);
        let unrelated = checked_node("other-domain", &WireNodeId::from_digest([8; 32]));
        let nodes = [unrelated, expected.clone()];

        let resolved = resolve_checked_node(nodes.iter(), wire, 3)
            .expect("the admitted package holds one exact match");

        assert_eq!(resolved, &expected);
    }

    /// Trace: FR-001-AC-12, TC-006.
    #[test]
    fn missing_wire_node_returns_a_typed_internal_fault() {
        let wire = WireNodeId::from_digest([9; 32]);
        let nodes: [CheckedNodeId; 0] = [];

        let refusal = resolve_checked_node(nodes.iter(), wire, 4)
            .expect_err("a missing node cannot be guessed from its display");

        assert!(matches!(
            refusal,
            DriverRefusal::Fault(DriverFault::UnknownNode {
                request_index: 4,
                node,
            }) if node == wire
        ));
        assert_eq!(refusal.stage(), DriverStage::Generation);
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(refusal.code(), "runtime_invariant");
    }

    /// Trace: FR-001-AC-12, TC-006.
    #[test]
    fn duplicate_wire_digest_with_distinct_domains_refuses_as_ambiguous() {
        let wire = WireNodeId::from_digest([10; 32]);
        let nodes = [
            checked_node("domain-a", &wire),
            checked_node("domain-b", &wire),
        ];

        let refusal = resolve_checked_node(nodes.iter(), wire, 5)
            .expect_err("a digest shared by two checked identities is ambiguous");

        assert!(matches!(
            refusal,
            DriverRefusal::Fault(DriverFault::AmbiguousNode {
                request_index: 5,
                node,
            }) if node == wire
        ));
        assert_eq!(refusal.stage(), DriverStage::Generation);
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(refusal.code(), "runtime_invariant");
    }
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
    use qsl_route::{BackendDescriptor, Candidate, ManifestDigest, ProviderOrigin, Registry};
    use std::collections::BTreeMap;

    use quire_contract_codegen as cg;

    use super::project_descriptor;

    fn descriptor(origin: ProviderOrigin, identity: &str) -> BackendDescriptor {
        let candidate =
            Candidate::new(BackendId::from_wire(identity).expect("identity is nonempty"));
        let domains = ["integer"];
        qsl_route::descriptor_from_manifest(
            candidate,
            origin,
            ManifestDigest::from_digest([7; 32]),
            [(Some("value-validity"), Some("bounded"))],
            Some(&domains),
            None,
        )
        .expect("QSL accepts the typed descriptor")
    }

    fn registry(origin: ProviderOrigin, identity: &str) -> Registry {
        let mut registry = Registry::new();
        registry
            .register(descriptor(origin, identity))
            .expect("one descriptor registers");
        registry
    }

    /// Trace: FR-005-AC-5, TC-012.
    #[test]
    fn qsl_origins_project_exhaustively_without_changing_descriptor_members() {
        let linked_registry = registry(ProviderOrigin::Linked, "example-provider");
        let process_registry = registry(ProviderOrigin::Process, "example-provider");
        let linked_bounds = BTreeMap::from([("max_states".to_owned(), 8)]);
        let linked = project_descriptor(
            linked_registry
                .descriptors()
                .next()
                .expect("one linked descriptor"),
            Some(&linked_bounds),
        )
        .expect("linked projection succeeds");
        let process = project_descriptor(
            process_registry
                .descriptors()
                .next()
                .expect("one process descriptor"),
            None,
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
        assert_eq!(
            linked.bounds,
            Some(BTreeMap::from([("max_states".to_owned(), 8)]))
        );
        assert_eq!(process.bounds, None);
    }

    /// Trace: FR-005-AC-17, TC-013.
    #[test]
    fn backend_kind_uses_qsl_origin_for_the_same_identity() {
        let linked_registry = registry(ProviderOrigin::Linked, "kani");
        let process_registry = registry(ProviderOrigin::Process, "kani");
        let project = |registry: &Registry| {
            project_descriptor(
                registry
                    .descriptors()
                    .next()
                    .expect("one registered descriptor"),
                None,
            )
            .expect("QSL descriptor projects into CG")
        };
        let linked_descriptor = project(&linked_registry);
        let process_descriptor = project(&process_registry);
        let id = BackendId::from_wire("kani").expect("identity is valid");

        let linked = super::convert_backends(
            &crate::upstream::QslCg,
            &linked_registry,
            &[linked_descriptor],
        )
        .expect("linked Kani maps to its built-in kind");
        let process = super::convert_backends(
            &crate::upstream::QslCg,
            &process_registry,
            &[process_descriptor],
        )
        .expect("process origin maps to CG's process kind");

        assert_eq!(linked.get(&id), Some(&cg::BackendKind::Kani));
        assert_eq!(
            process.get(&id),
            Some(&cg::BackendKind::Process(cg::BackendId("kani".to_owned())))
        );
    }
}
