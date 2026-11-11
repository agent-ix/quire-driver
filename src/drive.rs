// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`drive`]: the driver's steps, in ADR-011 T-13 order.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use qsl_foundation::source::provenance::OccurrenceKey;
use qsl_foundation::{BackendId, SourceIdentity};
use qsl_package::{EmittedPackage, ExportRefusal};
use qsl_replay::spine::{
    AdmittedModels, BindOutcome, Call, CheckedUnit, ScalarOccurrence, SpineLimits,
};
use qsl_route::request::{items_from_requirements, ExtentClassification, RequirementItem};
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{Candidate, CandidateOutcome, Mode, Registry};
use qsl_semantics::check::{Capability, RequirementMap};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_contract_codegen as cg;
use quire_contract_codegen::{
    BackendKind, ContractItemSettlement, KaniInstallation, KaniScalarNativeRequest,
    RoutedGeneration,
};
use quire_contract_model::{CheckedNodeId, CheckedPackageV2};
use quire_exact::Cancel;

use crate::handoff::handoff;
use crate::{
    AdmittedRegistry, DriverFault, DriverRefusal, RequirementRecordUnresolvedCause, Upstream,
};

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
    /// The checked package's name; QSL's check names the package it builds.
    pub package_name: &'a str,
    /// FR-056's package input: domain package documents by model identity.
    pub packages: &'a qsl_semantics::model::intake::PackageInput,
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
    /// Only QSL export-refused items, keyed by their original driver index.
    pub item_refusals: Vec<ItemRefusal>,
    /// The S4 in-process package.
    package: CheckedUnit,
    /// The E4 wire output: the v2 bytes.
    emitted: EmittedPackage,
    /// E5: the emitted bytes as IR's reader admitted them.
    checked: Arc<CheckedPackageV2>,
    /// CG's settlement of each export-bound item, in driver request order.
    pub settlements: Vec<ContractItemSettlement>,
    /// All QSL-produced request items, including export-refused items. Only
    /// bound entries construct the CG envelope or FR-331 process request.
    pub request_items: Vec<RequirementItem>,
    /// Immutable original request inventory for replay association.
    replay_items: Vec<RequirementItem>,
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
    pub(crate) fn replay_item(&self, index: usize) -> Option<&RequirementItem> {
        self.replay_items.get(index)
    }

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

/// One QSL export refusal, preserving its occurrence and driver index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemRefusal {
    /// Original index among all driver request items.
    pub request_index: usize,
    /// QSL's exact requested occurrence.
    pub occurrence: OccurrenceKey,
    /// QSL's unchanged typed refusal.
    pub refusal: ExportRefusal,
}

/// The export's one decision per original driver position.
struct ExportPartition {
    bound_indices: Vec<usize>,
    item_refusals: Vec<ItemRefusal>,
}

impl ExportPartition {
    fn includes(&self, request_index: usize) -> bool {
        self.bound_indices.binary_search(&request_index).is_ok()
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
        input.package_name,
        input.packages,
        input.limits,
        &input.cancel,
    )?;
    let emitted = emitted_unit.package().clone();
    // E5: IR admits the emitted bytes.
    let checked = Arc::new(handoff(
        upstream,
        &models,
        &checked_unit,
        &emitted_unit,
        input.limits.imports,
        input.limits.package.emit(),
    )?);
    // E4's source provision is needed through E5 for diagnostics, then the
    // returned CheckedUnit remains the driver's bounded source owner.
    drop(emitted_unit);

    // FR-075: one item per requirement record, in occurrence-key order.
    let items = items_from_requirements(
        checked_unit.package().graph().requirements(),
        registry,
        named,
    );
    let partition = if upstream.has_abstraction_relation(checked_unit.package()) {
        let keys = items
            .iter()
            .map(|item| item.occurrence().clone())
            .collect::<Vec<_>>();
        let results = upstream
            .export_bindings(checked_unit.package(), &keys)
            .map_err(|refusal| match refusal {
                ExportRefusal::UnknownItem { item } => {
                    DriverRefusal::Fault(DriverFault::ExportUnknownItem { item })
                }
                refusal @ (ExportRefusal::Unbound { .. } | ExportRefusal::Fault(_)) => {
                    DriverRefusal::BindingsExport(refusal)
                }
            })?;
        if results.len() != items.len() {
            return Err(DriverRefusal::Fault(DriverFault::ExportCount {
                requested: items.len(),
                returned: results.len(),
            }));
        }
        let mut partition = ExportPartition {
            bound_indices: Vec::with_capacity(items.len()),
            item_refusals: Vec::new(),
        };
        for (request_index, result) in results.into_iter().enumerate() {
            match result {
                Ok(bound_item) if bound_item.item == keys[request_index] => {
                    partition.bound_indices.push(request_index);
                }
                Ok(_) => {
                    return Err(DriverRefusal::Fault(DriverFault::ExportUnknownItem {
                        item: keys[request_index].clone(),
                    }))
                }
                Err(ExportRefusal::UnknownItem { item }) => {
                    return Err(DriverRefusal::Fault(DriverFault::ExportUnknownItem {
                        item,
                    }));
                }
                Err(refusal @ ExportRefusal::Fault(_)) => {
                    return Err(DriverRefusal::BindingsExport(refusal));
                }
                Err(ExportRefusal::Unbound { item, .. }) if item != keys[request_index] => {
                    return Err(DriverRefusal::Fault(DriverFault::ExportUnknownItem {
                        item,
                    }));
                }
                Err(refusal @ ExportRefusal::Unbound { .. }) => {
                    partition.item_refusals.push(ItemRefusal {
                        request_index,
                        occurrence: keys[request_index].clone(),
                        refusal,
                    })
                }
            }
        }
        partition
    } else {
        ExportPartition {
            bound_indices: (0..items.len()).collect(),
            item_refusals: Vec::new(),
        }
    };
    let lowered = lower_driver_request(
        &checked,
        &items,
        &partition,
        &upstream.requirement_records(checked_unit.package()),
    )?;

    // T-13 order: the `route` candidate step, then every registered
    // BackendId through CG's one conversion, then E7.
    let manifests = registry.manifest_snapshot();
    let prepared = envelope(
        registry, manifests, &checked, &items, &partition, named, &lowered,
    )?;
    let kinds = convert_backends(upstream, registry, &prepared.generation_descriptors)?;
    let settlements = upstream
        .negotiate(
            &checked,
            &lowered.contract_package,
            &lowered.positions,
            &prepared.envelope,
        )
        .map_err(|error| DriverRefusal::from(*error))?;
    let in_order = settlements.len() == partition.bound_indices.len()
        && settlements
            .iter()
            .map(|settlement| settlement.request_index)
            .eq(partition.bound_indices.iter().copied());
    if !in_order {
        return Err(DriverRefusal::Fault(DriverFault::SettlementCount {
            requested: partition.bound_indices.len(),
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
        item_refusals: partition.item_refusals,
        settlements,
        replay_items: items.clone(),
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
            application: item.node().clone(),
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
    let mut generation_items = Vec::with_capacity(routed.len());
    for item in routed {
        let request_index = item.request_index;
        let node = items
            .get(request_index)
            .ok_or(DriverRefusal::Fault(DriverFault::RoutingTarget))?
            .node();
        let node_id = resolve_checked_node(checked_node_keys(checked), node, request_index)?;
        let descriptor = descriptors
            .iter()
            .find(|descriptor| descriptor.identity == item.candidate.id().as_str())
            .ok_or(DriverRefusal::Fault(DriverFault::AdmittedManifestMissing))?;
        let generation_item = cg::RoutedGenerationItem::from_descriptor(
            request_index,
            node_id.clone(),
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
        let generation_item = if item.kind == cg::BackendKind::Kani {
            let requirement = lowered
                .requirements
                .get(request_index)
                .cloned()
                .flatten()
                .ok_or_else(|| {
                    DriverRefusal::Fault(DriverFault::RequirementRecordAbsent {
                        request_index,
                        occurrence: items[request_index].occurrence().clone(),
                    })
                })?;
            generation_item.with_requirement(requirement)
        } else {
            generation_item
        };
        generation_items.push(generation_item);
    }
    let contexts = cg::GenerationContexts {
        kani: generation.kani.map(|kani| cg::KaniGenerationContext {
            ceilings: kani.ceilings,
            subject_path: kani.subject_path,
            unwind: kani.unwind,
            requirement_records: &lowered.records,
        }),
    };
    upstream
        .generate(
            checked,
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
    /// FR-020: the requirement for each request index, `None` at an excluded
    /// position and at a bound item with no `value-validity` record.
    requirements: Vec<Option<cg::RoutedRequirement>>,
    /// FR-020: one entry for every `value-validity` record of the package.
    records: cg::RequirementRecords,
}

/// The projection of QSL's requirement records into CG's typed input
/// (FR-020).
struct RequirementProjection {
    requirements: Vec<Option<cg::RoutedRequirement>>,
    records: cg::RequirementRecords,
}

/// Why a QSL node does not resolve in the admitted package.
enum NodeLookup {
    Unknown,
    Ambiguous,
}

/// Every V2 node of the admitted package paired with its structural key.
fn checked_node_keys(
    checked: &CheckedPackageV2,
) -> impl Iterator<Item = (&CheckedNodeId, &CheckedNodeId)> {
    checked
        .resolved_graph()
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            checked
                .structural_key_at(index)
                .map(|key| (&node.node_id, key))
        })
}

/// The one node walk: the V2 node whose structural key is `node`.
fn select_node<'a>(
    nodes: impl Iterator<Item = (&'a CheckedNodeId, &'a CheckedNodeId)>,
    node: &quire_exact::NodeKey,
) -> Result<&'a CheckedNodeId, NodeLookup> {
    let mut matches = nodes.filter_map(|(id, key)| structural_key_matches(key, node).then_some(id));
    let selected = matches.next().ok_or(NodeLookup::Unknown)?;
    if matches.next().is_some() {
        return Err(NodeLookup::Ambiguous);
    }
    Ok(selected)
}

/// The V2 node of `node` in the admitted package, by the walk FR-001 uses.
fn lookup_checked_node<'a>(
    checked: &'a CheckedPackageV2,
    node: &quire_exact::NodeKey,
) -> Result<&'a CheckedNodeId, NodeLookup> {
    select_node(checked_node_keys(checked), node)
}

/// IR's occurrence role for a QSL role spelling.
fn occurrence_role(role: &str) -> Option<quire_contract_model::CheckedOccurrenceRole> {
    use quire_contract_model::CheckedOccurrenceRole as Role;
    match role {
        "declaration" => Some(Role::Declaration),
        "type" => Some(Role::Type),
        "expression" => Some(Role::Expression),
        "anchor" => Some(Role::Anchor),
        "claim" => Some(Role::Claim),
        "generated" => Some(Role::Generated),
        _ => None,
    }
}

/// A package record's QSL occurrence key as CG's `RequirementOccurrence`,
/// in the admitted package's V2 frame. A record has no request index, so a
/// failure is the record-level fault naming the record's `record` key.
fn record_occurrence(
    checked: &CheckedPackageV2,
    record: &OccurrenceKey,
    occurrence: &OccurrenceKey,
) -> Result<cg::RequirementOccurrence, DriverRefusal> {
    let unresolved = |cause| {
        DriverRefusal::Fault(DriverFault::RequirementRecordUnresolved {
            occurrence: record.clone(),
            cause,
        })
    };
    let node_id = lookup_checked_node(checked, occurrence.node())
        .map_err(|lookup| {
            unresolved(match lookup {
                NodeLookup::Unknown => RequirementRecordUnresolvedCause::UnknownNode {
                    node: occurrence.node().clone(),
                },
                NodeLookup::Ambiguous => RequirementRecordUnresolvedCause::AmbiguousNode {
                    node: occurrence.node().clone(),
                },
            })
        })?
        .clone();
    let role = occurrence_role(occurrence.origin().role().as_str()).ok_or_else(|| {
        unresolved(RequirementRecordUnresolvedCause::UnknownOccurrenceRole {
            role: occurrence.origin().role().as_str().to_owned(),
        })
    })?;
    Ok(cg::RequirementOccurrence {
        node_id,
        role,
        ordinal: occurrence.origin().ordinal(),
    })
}

/// A package record's result-bound node in the admitted package's V2 frame.
fn record_node(
    checked: &CheckedPackageV2,
    record: &OccurrenceKey,
    node: &quire_exact::NodeKey,
) -> Result<CheckedNodeId, DriverRefusal> {
    lookup_checked_node(checked, node)
        .cloned()
        .map_err(|lookup| {
            DriverRefusal::Fault(DriverFault::RequirementRecordUnresolved {
                occurrence: record.clone(),
                cause: match lookup {
                    NodeLookup::Unknown => {
                        RequirementRecordUnresolvedCause::UnknownNode { node: node.clone() }
                    }
                    NodeLookup::Ambiguous => {
                        RequirementRecordUnresolvedCause::AmbiguousNode { node: node.clone() }
                    }
                },
            })
        })
}

/// FR-020: project the QSL requirement of every export-bound
/// `value-validity` item that has a record, and the package's whole set of
/// `value-validity` records, from the one `map`. Only the `value-validity`
/// capability projects, because CG keys its records by occurrence alone and
/// two capabilities can share one. An item of another capability, and an
/// item with no record, projects `None` and raises nothing here; the absent
/// record of a routed item is refused when the generation items are built.
/// Items resolve first (request-indexed faults), then the package records.
fn project_requirements(
    checked: &CheckedPackageV2,
    items: &[RequirementItem],
    partition: &ExportPartition,
    map: &RequirementMap,
) -> Result<RequirementProjection, DriverRefusal> {
    let mut requirements = Vec::with_capacity(items.len());
    for (request_index, item) in items.iter().enumerate() {
        if !partition.includes(request_index)
            || map
                .get_kind(item.occurrence(), Capability::ValueValidity)
                .is_none()
        {
            requirements.push(None);
            continue;
        }
        let occurrence = |key: &OccurrenceKey| {
            contract_occurrence(checked, key, request_index)
                .map(|key| cg::RequirementOccurrence::from(&key))
        };
        let result_bound = resolve_checked_node(
            checked_node_keys(checked),
            item.result_bound(),
            request_index,
        )?
        .clone();
        let path_condition = item
            .path_condition()
            .iter()
            .map(|guard| {
                Ok(cg::RequirementGuard {
                    occurrence: occurrence(guard.occurrence())?,
                    holds: guard.holds(),
                })
            })
            .collect::<Result<Vec<_>, DriverRefusal>>()?;
        requirements.push(Some(cg::RoutedRequirement {
            occurrence: occurrence(item.occurrence())?,
            result_bound,
            conversion_context: item.conversion_context().map(occurrence).transpose()?,
            path_condition,
        }));
    }
    let mut records = BTreeMap::new();
    for (key, record) in map.iter() {
        if record.requirements().kind() != Capability::ValueValidity {
            continue;
        }
        let occurrence = record_occurrence(checked, key, key)?;
        let conversion = record
            .conversion_context()
            .map(|conversion| record_occurrence(checked, key, conversion))
            .transpose()?;
        let path_condition = record
            .path_condition()
            .iter()
            .map(|guard| {
                Ok(cg::RequirementGuard {
                    occurrence: record_occurrence(checked, key, guard.occurrence())?,
                    holds: guard.holds(),
                })
            })
            .collect::<Result<Vec<_>, DriverRefusal>>()?;
        records.insert(
            occurrence,
            cg::RequirementRecordInput {
                result_bound: record_node(checked, key, &record.result_bound().node())?,
                conversion_context: conversion,
                path_condition,
            },
        );
    }
    Ok(RequirementProjection {
        requirements,
        records: cg::RequirementRecords::new(records),
    })
}

/// Lower each admitted driver item and its QSL-selected result context once.
/// The same package and positions bind CG negotiation preflight and generation.
fn lower_driver_request(
    checked: &CheckedPackageV2,
    items: &[RequirementItem],
    partition: &ExportPartition,
    map: &RequirementMap,
) -> Result<LoweredDriverRequest, DriverRefusal> {
    // FR-001 first: every item's own application, occurrence and conversion
    // resolve (request-indexed faults) before any package record is read.
    let mut frames = Vec::with_capacity(items.len());
    for (request_index, item) in items.iter().enumerate() {
        let application =
            resolve_checked_node(checked_node_keys(checked), item.node(), request_index)?.clone();
        if !partition.includes(request_index) {
            frames.push((application, None));
            continue;
        }
        let application_occurrence =
            contract_occurrence(checked, item.occurrence(), request_index)?;
        let conversion = item
            .conversion_context()
            .map(|occurrence| contract_occurrence(checked, occurrence, request_index))
            .transpose()?;
        frames.push((application, Some((application_occurrence, conversion))));
    }
    let projection = project_requirements(checked, items, partition, map)?;
    let mut roots = Vec::new();
    let mut positions = Vec::new();
    let mut result_selections = Vec::with_capacity(items.len());
    for (request_index, (application, selection)) in frames.into_iter().enumerate() {
        let Some((application_occurrence, conversion)) = selection else {
            roots.push(application.clone());
            positions.push(cg::ContractRequestPosition::Excluded {
                node_id: application,
            });
            result_selections.push(cg::ContractDriverResultSelection::NotApplicable);
            continue;
        };
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
        // FR-020: one guard root per path guard, outermost first, inside this
        // item's own context span; a guard shared by two items is rooted in each.
        if let Some(requirement) = &projection.requirements[request_index] {
            for guard in &requirement.path_condition {
                roots.push(guard.occurrence.node_id.clone());
                positions.push(cg::ContractRequestPosition::ContextRoot {
                    node_id: guard.occurrence.node_id.clone(),
                });
            }
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
        requirements: projection.requirements,
        records: projection.records,
    })
}

fn contract_occurrence(
    checked: &CheckedPackageV2,
    occurrence: &qsl_foundation::source::provenance::OccurrenceKey,
    request_index: usize,
) -> Result<quire_contract_model::ContractOccurrenceKeyV1, DriverRefusal> {
    let node_id =
        resolve_checked_node(checked_node_keys(checked), occurrence.node(), request_index)?.clone();
    let role = occurrence_role(occurrence.origin().role().as_str()).ok_or_else(|| {
        DriverRefusal::Fault(DriverFault::UnknownOccurrenceRole {
            role: occurrence.origin().role().as_str().to_owned(),
        })
    })?;
    Ok(quire_contract_model::ContractOccurrenceKeyV1 {
        node_id,
        role,
        ordinal: occurrence.origin().ordinal(),
    })
}

#[cfg(test)]
mod scalar_receipt_tests {
    use super::{driver_entries_match, qsl_limit_fields_match};
    use quire_contract_codegen::Artifact;

    #[test]
    fn every_named_limit_is_checked_against_the_retained_qsl_value() {
        fn different_in_range(value: u64) -> u64 {
            if value == u64::MAX {
                value - 1
            } else {
                value + 1
            }
        }

        let retained = qsl_replay::spine::default_accounting(1_000_000);
        assert!(qsl_limit_fields_match(&retained, &retained));
        for field in 0..10 {
            let mut changed = retained;
            match field {
                0 => changed.integer_bits = different_in_range(changed.integer_bits),
                1 => changed.decimal_digits = different_in_range(changed.decimal_digits),
                2 => changed.scale_expansion = different_in_range(changed.scale_expansion),
                3 => changed.text_input_bytes = different_in_range(changed.text_input_bytes),
                4 => changed.text_scalars = different_in_range(changed.text_scalars),
                5 => changed.normalized_scalars = different_in_range(changed.normalized_scalars),
                6 => changed.unit_edges = different_in_range(changed.unit_edges),
                7 => changed.value_occurrences = different_in_range(changed.value_occurrences),
                8 => changed.work_units = different_in_range(changed.work_units),
                9 => changed.result_units = different_in_range(changed.result_units),
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

// The native runner's memory enforcement requires Linux procfs.
#[cfg(all(test, target_os = "linux"))]
mod native_receipt_integration_tests {
    use super::*;
    use qsl_replay::spine::{default_accounting, CallArgument};
    use quire_contract_codegen::{KaniInstallation, ProofCeilings};
    use quire_exact::{Integer, Value};
    use std::io::Write as _;
    use std::num::NonZeroU64;
    use std::path::PathBuf;

    struct NativeTargetDirectory {
        path: PathBuf,
        lock_path: PathBuf,
    }

    impl NativeTargetDirectory {
        fn new() -> Self {
            let executable = std::env::current_exe().expect("the test executable has a path");
            let target_root = executable
                .parent()
                .and_then(Path::parent)
                .and_then(Path::parent)
                .expect("the test executable resides under Cargo's target directory");
            let path = target_root.join("ir773-native-target");
            std::fs::create_dir_all(&path)
                .expect("the test owns its stable native target directory");

            // Keep Kani's compiled dependencies for sequential focused runs, but never let
            // concurrent runs mutate the same target. A hard-killed run leaves the marker;
            // remove it only after verifying that no run still owns this target.
            let lock_path = target_root.join("ir773-native-target.lock");
            let mut lock = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
                .expect("no other native receipt test owns the stable Kani target");
            writeln!(lock, "pid={}", std::process::id())
                .expect("the native target ownership marker is writable");
            Self { path, lock_path }
        }
    }

    impl Drop for NativeTargetDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.lock_path);
        }
    }

    /// This Kani-lane test obtains a genuine successful receipt through the
    /// production driver call, then checks the driver's L0 comparison against
    /// a distinct retained context.
    /// Trace: FR-009-AC-12, TC-016.
    #[test]
    #[ignore = "kani lane: paired proof/native execution uses the installed Kani backend"]
    fn successful_native_receipt_is_authenticated_against_retained_limits() {
        let source = format!(
            "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
             function inc using v(x: Int[4, 4]): Int[5, 5] pure {{ x + 1 }}\n",
            "quire.value.complete/v1"
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
            backend: Default::default(),
        };
        let environment = ObjectEnvironment::default();
        let installation = KaniInstallation::discover().expect("cargo-kani is installed");
        let target_directory = NativeTargetDirectory::new();
        let registry = crate::build_registry(&[crate::ProviderManifest {
            identity: "kani".to_owned(),
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
                    memory_bytes: NonZeroU64::new(16 * 1024 * 1024 * 1024).expect("positive limit"),
                    wall_clock: std::time::Duration::from_secs(600),
                },
                subject_path: "crate::subject",
                unwind: 3,
            }),
            scalar_execution: Some(ScalarExecutionInput {
                call: &call,
                replay_limits: qsl_replay::ReplayLimits::default(),
                environment: &environment,
                native: Some(ScalarNativeInput {
                    installation: &installation,
                    target_directory: &target_directory.path,
                }),
            }),
        };
        let driven = drive(
            &crate::upstream::QslCg,
            CompileInput {
                source: SourceIdentity::new("agent-ix", "ir-773"),
                package_name: "native-receipt",
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
        let native_value = receipt
            .native_outcome()
            .clone()
            .completed()
            .expect("the native outcome is completed");
        assert_eq!(native_value.to_string(), "5");
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
        different_original_limits.result_units -= 1;
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
    checked: &Arc<CheckedPackageV2>,
    items: &[RequirementItem],
    partition: &ExportPartition,
    named: Option<&BackendId>,
    lowered: &LoweredDriverRequest,
) -> Result<PreparedEnvelope, DriverRefusal> {
    let manifest = registry
        .descriptors()
        .map(|descriptor| {
            let retained = manifests
                .iter()
                .find(|manifest| manifest.identity == descriptor.id().as_str())
                .ok_or(DriverRefusal::Fault(DriverFault::AdmittedManifestMissing))?;
            project_descriptor(descriptor, retained.bounds.as_ref())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut request_items = Vec::with_capacity(items.len());
    let mut candidate_sets = Vec::with_capacity(items.len());
    for (request_index, item) in items.iter().enumerate() {
        if !partition.includes(request_index) {
            candidate_sets.push(Vec::new());
            continue;
        }
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
    checked: &Arc<CheckedPackageV2>,
    node: &quire_exact::NodeKey,
    request_index: usize,
    lowered: &LoweredDriverRequest,
) -> Result<cg::KaniForm, DriverRefusal> {
    let selected = resolve_checked_node(checked_node_keys(checked), node, request_index)?;
    Ok(cg::KaniForm::V2ScalarClaim {
        package: Arc::clone(checked),
        contract: Box::new(lowered.contract_package.clone()),
        positions: lowered.positions.clone(),
        request_index,
        node_id: selected.clone(),
    })
}

fn resolve_checked_node<'a>(
    nodes: impl Iterator<Item = (&'a CheckedNodeId, &'a CheckedNodeId)>,
    node: &quire_exact::NodeKey,
    request_index: usize,
) -> Result<&'a CheckedNodeId, DriverRefusal> {
    select_node(nodes, node).map_err(|lookup| {
        DriverRefusal::Fault(match lookup {
            NodeLookup::Unknown => DriverFault::UnknownNode {
                request_index,
                node: node.clone(),
            },
            NodeLookup::Ambiguous => DriverFault::AmbiguousNode {
                request_index,
                node: node.clone(),
            },
        })
    })
}

/// Compare the owners' borrowed structural forms without constructing another source graph.
fn structural_key_matches(id: &CheckedNodeId, key: &quire_exact::NodeKey) -> bool {
    use quire_contract_model::CheckedNodePathSegment;
    use quire_exact::PathSegmentView;

    let mut pending = vec![(id, key.view())];
    while let Some((id, key)) = pending.pop() {
        if id.package.as_ref() != key.package() || id.ordinal != key.ordinal() {
            return false;
        }
        let mut path = key.path();
        for segment in &id.path {
            match segment {
                CheckedNodePathSegment::Name(name) => {
                    let Some(PathSegmentView::Text(text)) = path.next() else {
                        return false;
                    };
                    if name.as_ref() != text {
                        return false;
                    }
                }
                CheckedNodePathSegment::Node(node) => {
                    let Some(PathSegmentView::Node(child)) = path.next() else {
                        return false;
                    };
                    pending.push((node.as_ref(), child));
                }
                CheckedNodePathSegment::Index(_)
                | CheckedNodePathSegment::Operand(_)
                | CheckedNodePathSegment::Dependency { .. } => return false,
            }
        }
        if path.next().is_some() {
            return false;
        }
    }
    true
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
    use ix_trace_rs::trace;
    use quire_contract_model::CheckedNodeId;
    use quire_exact::NodeKey;
    use quire_exact::PathSegment;

    use super::resolve_checked_node;
    use crate::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};

    fn key(name: &str, ordinal: u64) -> NodeKey {
        NodeKey::new(
            "pkg".to_owned(),
            vec![PathSegment::Text(name.to_owned())],
            ordinal,
        )
        .expect("a fixture key is admissible")
    }

    fn checked(key: &NodeKey) -> CheckedNodeId {
        serde_json::from_value(
            qsl_foundation::node_wire::node_key_to_json(key).expect("the key encodes"),
        )
        .expect("the wire form is a checked node identity")
    }

    /// Trace: FR-001-AC-12, TC-006.
    #[test]
    fn selected_node_key_resolves_to_its_exact_checked_identity() {
        let wanted = key("a", 0);
        let nodes = [checked(&key("a", 1)), checked(&wanted)];

        let resolved = resolve_checked_node(nodes.iter().map(|id| (id, id)), &wanted, 3)
            .expect("the admitted package holds one exact match");

        assert_eq!(resolved, &nodes[1]);
    }

    #[trace("TC-006", "FR-001-AC-12")]
    #[test]
    fn indexed_wire_id_is_selected_by_its_resolved_operand_key() {
        use quire_contract_model::CheckedNodePathSegment;
        use std::sync::Arc;

        let child = key("operand", 0);
        let other_child = key("operand", 1);
        let wanted = NodeKey::new("pkg".to_owned(), vec![PathSegment::Node(child.clone())], 0)
            .expect("a nested source key is valid");
        let resolved = |child: &NodeKey| CheckedNodeId {
            package: "pkg".into(),
            path: vec![CheckedNodePathSegment::Node(Arc::new(checked(child)))],
            ordinal: 0,
        };
        let wire = |index| CheckedNodeId {
            package: "pkg".into(),
            path: vec![CheckedNodePathSegment::Index(index)],
            ordinal: 0,
        };
        let ids = [wire(3), wire(8)];
        let keys = [resolved(&other_child), resolved(&child)];
        let selected = resolve_checked_node(ids.iter().zip(keys.iter()), &wanted, 0)
            .expect("resolved operands select the admitted graph-local ID");
        assert!(std::ptr::eq(selected, &ids[1]));
        assert_ne!(
            selected, &keys[1],
            "wire identity remains in the admitted frame"
        );
    }

    /// Trace: FR-001-AC-12, TC-006.
    #[test]
    fn missing_node_key_returns_a_typed_internal_fault() {
        let wanted = key("a", 0);
        let nodes: [CheckedNodeId; 0] = [];

        let refusal = resolve_checked_node(nodes.iter().map(|id| (id, id)), &wanted, 4)
            .expect_err("a missing node cannot be guessed");

        assert!(matches!(
            &refusal,
            DriverRefusal::Fault(DriverFault::UnknownNode {
                request_index: 4,
                node,
            }) if node == &wanted
        ));
        assert_eq!(refusal.stage(), DriverStage::Generation);
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(refusal.code(), "runtime_invariant");
    }

    /// Trace: FR-001-AC-12, TC-006.
    #[test]
    fn an_identity_present_twice_refuses_as_ambiguous() {
        let wanted = key("a", 0);
        let nodes = [checked(&wanted), checked(&wanted)];

        let refusal = resolve_checked_node(nodes.iter().map(|id| (id, id)), &wanted, 5)
            .expect_err("two checked identities with one key are ambiguous");

        assert!(matches!(
            &refusal,
            DriverRefusal::Fault(DriverFault::AmbiguousNode {
                request_index: 5,
                node,
            }) if node == &wanted
        ));
        assert_eq!(refusal.stage(), DriverStage::Generation);
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(refusal.code(), "runtime_invariant");
    }
}

/// The routing step over the `supported` settlements only.
fn route_supported(
    upstream: &impl Upstream,
    settlements: &[ContractItemSettlement],
    candidate_sets: &[Vec<Candidate>],
    kinds: &BTreeMap<BackendId, BackendKind>,
) -> Result<Vec<Routed>, DriverRefusal> {
    let mut indexes = Vec::new();
    let mut dispositions = Vec::new();
    for settlement in settlements {
        let Some(cg::ItemSettlement {
            disposition: cg::Disposition::Supported { backend },
            ..
        }) = &settlement.backend
        else {
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
    use qsl_route::{BackendDescriptor, Candidate, ProviderOrigin, Registry};
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
            [(Some("value-validity"), Some("bounded"))],
            qsl_route::ManifestMembers {
                domains: Some(&domains),
                ..Default::default()
            },
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
