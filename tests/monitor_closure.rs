// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plan-authority admission and selected type closure from a genuine PlanRoot.

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{PlanAuthoritySelection, PlanAuthorityValidationCaps};
use quire_driver::monitor::{
    collect_monitor_closure_rows, MonitorPreparationErrorCode, MonitorPreparationLimits,
};
use quire_exact::{Cancel, Integer, Rational};

/// Native PlanRoot support shared by monitor integration suites.
#[path = "common/monitor.rs"]
pub mod monitor_fixture;
/// Genuine observation-owner progress admission.
#[path = "common/observation.rs"]
pub mod observation;

fn caps() -> PlanAuthorityValidationCaps {
    PlanAuthorityValidationCaps {
        read: monitor_fixture::read_limits(),
        order: quire_canonical::Limits::new(1 << 20),
    }
}

/// The TC-200 step-1 clause, selected by name over the shared fixture unit.
const STEP_ONE: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    always (holds(candidate.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}"#;

fn collect(
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<quire_driver::monitor::MonitorClosureRows, quire_driver::monitor::MonitorPreparationError>
{
    collect_clause("SelectedMonitor", "", limits, cancel)
}

fn collect_clause(
    selected: &str,
    additional: &str,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<quire_driver::monitor::MonitorClosureRows, quire_driver::monitor::MonitorPreparationError>
{
    let fixture = monitor_fixture::fixture(selected, "counter", additional);
    let root = fixture.into_root();
    let (progress, subject) = observation::admitted(Some(vec![observation::row(&root)]));
    let source = root.checked_unit().source().identity().clone();
    let claim = root.plan().subject().clone();
    let clock = observation::clock();
    let observation_source = observation::source();
    collect_monitor_closure_rows(
        root,
        PlanAuthoritySelection {
            source: &source,
            claim: &claim,
            observation_subject: &subject,
            observation_clock: &clock,
            observation_source: &observation_source,
        },
        Some(progress),
        caps(),
        limits,
        cancel,
    )
}

const LIMITS: MonitorPreparationLimits = MonitorPreparationLimits {
    work: 100_000_000,
    storage: 100_000_000,
};

#[trace("TC-033", "FR-019-AC-1")]
#[test]
fn genuine_plan_root_admits_authority_and_reads_the_selected_closure() {
    let cancel = Cancel::new();
    let mut rows = collect(LIMITS, &cancel).expect("genuine root admits");
    let before = rows.budget().consumed();
    assert!(before.work > 0);
    let work = rows
        .read_closure(&cancel, |authority, closure, _facts| {
            assert_eq!(
                closure.authority().order_value(),
                authority.admitted_order()
            );
            assert_eq!(
                closure.authority().progress_association(),
                authority.progress_context()
            );
            assert!(!closure.exact_types().is_empty());
            closure.work()
        })
        .expect("closure reads");
    assert!(work > 0);
    assert!(rows.budget().consumed().work > before.work);
}

#[trace("TC-033", "FR-019-AC-3")]
#[test]
fn a_starved_budget_stops_with_a_work_or_storage_code() {
    let cancel = Cancel::new();
    let error = collect(
        MonitorPreparationLimits {
            work: 1,
            storage: 1,
        },
        &cancel,
    )
    .err()
    .expect("starved");
    assert!(matches!(
        error.code(),
        MonitorPreparationErrorCode::Work | MonitorPreparationErrorCode::Storage
    ));
}

#[trace("TC-033", "FR-019-AC-5")]
#[test]
fn cancellation_before_admission_returns_cancelled() {
    let cancel = Cancel::new();
    cancel.cancel(quire_exact::CancelCause::Requested);
    let error = collect(LIMITS, &cancel).err().expect("cancelled");
    assert_eq!(error.code(), MonitorPreparationErrorCode::Cancelled);
}

#[trace("TC-033", "FR-019-AC-6")]
#[test]
fn the_step_one_monitor_declarations_admit_into_a_type_environment() {
    let cancel = Cancel::new();
    let mut rows = collect_clause("C", STEP_ONE, LIMITS, &cancel).expect("genuine root admits");
    let (environment, expected) = rows
        .read_closure(&cancel, |_, closure, facts| {
            let environment =
                quire_driver::monitor::build_type_environment(closure, facts, LIMITS, &cancel);
            let expected =
                facts.source_records.len() + facts.source_tuples.len() + facts.source_unions.len();
            (environment, expected)
        })
        .expect("closure reads");
    let environment = environment.expect("QSL's declarations admit as the retained environment");
    assert!(expected > 0);
    assert_eq!(environment.composites().count(), expected);
}

fn derive(
    rows: &mut quire_driver::monitor::MonitorClosureRows,
    cancel: &Cancel,
    wrong_dimension: bool,
) -> Result<quire_contract_runtime::monitor::Layout, quire_driver::monitor::MonitorPreparationError>
{
    use quire_contract_runtime::monitor::{Entries, FunctionId};
    rows.read_closure(cancel, |authority, closure, _| {
        let package = authority.admitted_package();
        let dimensions = closure.model_types().dimensions();
        assert_eq!(
            dimensions.len(),
            1,
            "the fixture declares one Time dimension"
        );
        let expected = if wrong_dimension {
            closure.plan().state()
        } else {
            &dimensions[0].structural_node
        };
        let entries = Entries {
            init: FunctionId(0),
            on_event: FunctionId(1),
            on_watermark: FunctionId(2),
            on_gap: FunctionId(3),
        };
        quire_driver::monitor::derive_layout(
            package, authority, closure, entries, expected, LIMITS, cancel,
        )
    })?
}

#[trace("TC-033", "FR-019-AC-9")]
#[test]
fn the_step_one_plan_derives_its_layout_from_the_closure() {
    let cancel = Cancel::new();
    let mut rows = collect_clause("C", STEP_ONE, LIMITS, &cancel).expect("genuine root admits");
    let layout = derive(&mut rows, &cancel, false).expect("layout derives");
    assert!(!layout.package_lock.is_empty());
    assert!(!layout.clock_binding.is_empty());
    assert!(!layout.admitted_order.is_empty());
    // The fixture's independently authored target premises.
    assert_eq!(layout.counter, "counter");
    assert_eq!(layout.counter_width_bits, 16);
    assert_eq!(layout.max_reading_gap, 100);
    assert_eq!(layout.activation_capacity, Integer::from(8_u64));
    assert_eq!(
        layout.max_event_rate,
        Rational::from_integer(Integer::from(2_u64))
    );
    assert_eq!(layout.uncertainty, Rational::from_integer(Integer::zero()));
    assert_eq!(layout.buffers.len(), 1);
    assert!(layout.buffers[0].capacity > Integer::zero());
}

#[trace("TC-033", "FR-019-AC-9")]
#[test]
fn a_tick_unit_of_another_dimension_is_a_target_mismatch() {
    let cancel = Cancel::new();
    let mut rows = collect_clause("C", STEP_ONE, LIMITS, &cancel).expect("genuine root admits");
    let error = derive(&mut rows, &cancel, true).err().expect("mismatch");
    assert_eq!(error.code(), MonitorPreparationErrorCode::TargetMismatch);
}

fn prepare(
    selected: &str,
    additional: &str,
    cancel: &Cancel,
) -> Result<
    std::rc::Rc<quire_contract_runtime::monitor::Program>,
    quire_driver::monitor::MonitorPreparationError,
> {
    let fixture = monitor_fixture::fixture(selected, "counter", additional);
    // Independent of the closure under test: the strictly admitted package's
    // own declared Time dimension.
    let time = fixture
        .package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| &*node.semantic_form == "dimension")
        .expect("the declared Time dimension")
        .node_id
        .clone();
    let root = fixture.into_root();
    let (progress, subject) = observation::admitted(Some(vec![observation::row(&root)]));
    let source = root.checked_unit().source().identity().clone();
    let claim = root.plan().subject().clone();
    let clock = observation::clock();
    let observation_source = observation::source();
    quire_driver::monitor::prepare_monitor(
        root,
        PlanAuthoritySelection {
            source: &source,
            claim: &claim,
            observation_subject: &subject,
            observation_clock: &clock,
            observation_source: &observation_source,
        },
        Some(progress),
        caps(),
        &time,
        LIMITS,
        cancel,
    )
}

#[trace("TC-033", "FR-017-AC-1")]
#[test]
fn the_step_one_plan_prepares_a_program_with_four_typed_entries() {
    use quire_exact::ValueType;
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel)
        .unwrap_or_else(|error| panic!("prepare failed: {error:?} / {error}"));
    let layout = program.layout();
    let entry = |id| program.function(id).expect("a retained entry function");
    let (init, event, watermark, gap) = (
        entry(layout.entries.init),
        entry(layout.entries.on_event),
        entry(layout.entries.on_watermark),
        entry(layout.entries.on_gap),
    );
    assert!(init.parameters.is_empty());
    assert_eq!(init.result, layout.state.value_type);
    assert_eq!(
        event.parameters,
        vec![
            layout.state.value_type.clone(),
            ValueType::Integer,
            layout.inputs.value_type.clone()
        ]
    );
    assert_eq!(
        watermark.parameters,
        vec![
            layout.state.value_type.clone(),
            ValueType::Integer,
            layout.scope.value_type.clone()
        ]
    );
    assert_eq!(
        gap.parameters,
        vec![layout.state.value_type.clone(), ValueType::Integer]
    );
    for step in [event, watermark, gap] {
        assert_eq!(step.result, layout.step_result.value_type);
    }
    // The four entries are distinct functions with distinct source keys.
    let mut keys = vec![&init.key, &event.key, &watermark.key, &gap.key];
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), 4);
}

/// Smoke test only: the complete TC-200 step-1 oracle is `monitor_tick.rs`.
///
/// The first result's outcome name after one request event at tick 0 and a
/// progress assertion covering ticks through `boundary`.
fn outcome_after(
    program: &std::rc::Rc<quire_contract_runtime::monitor::Program>,
    boundary: u64,
) -> String {
    use quire_contract_runtime::monitor::{
        build, BuildPlan, Event, InputContext, Reading, StepBudget, Watermark,
    };
    use quire_exact::{FieldValue, Integer, ScalarLimits, Value};
    let layout = program.layout();
    let context = InputContext {
        package_lock: &layout.package_lock,
        clause: &layout.clause,
        subject: &layout.subject,
        occurrence: &layout.occurrence,
        clock_binding: &layout.clock_binding,
        counter: &layout.counter,
        tick_unit: &layout.tick_unit,
        counter_width_bits: layout.counter_width_bits,
        profile: &layout.profile,
        admitted_order: &layout.admitted_order,
    };
    let big = 1_000_000;
    let budget = StepBudget {
        work: 100_000_000,
        scalar: ScalarLimits {
            integer_bits: big,
            decimal_digits: big,
            scale_expansion: big,
            text_input_bytes: big,
            text_scalars: big,
            normalized_scalars: big,
            unit_edges: big,
            value_occurrences: big,
            work_units: big,
            result_units: big,
        },
        stack_frames: 100_000,
        scratch_elements: 1_000_000,
        scratch_bytes: 100_000_000,
    };
    let mut monitor = build(BuildPlan {
        program: std::rc::Rc::clone(program),
        budget,
    })
    .expect("init runs");
    let request = program
        .types()
        .record(
            layout.inputs.key.clone(),
            vec![
                ("a2", FieldValue::Present(Value::Boolean(true))),
                ("a4", FieldValue::Present(Value::Boolean(false))),
            ],
        )
        .expect("captures");
    let scope = program
        .types()
        .record(
            layout.scope.key.clone(),
            vec![(
                "subject",
                FieldValue::Present(Value::Integer(Integer::zero())),
            )],
        )
        .expect("scope");
    monitor
        .event(Event {
            reading: Reading { context, value: 0 },
            inputs: &request,
        })
        .expect("request event");
    let boundary = Integer::from(boundary);
    let step = monitor
        .watermark(Watermark {
            context,
            progress_context: &layout.progress_context,
            assertion: b"admitted assertion",
            inclusive_boundary: &boundary,
            scope: &scope,
        })
        .expect("watermark");
    let Value::Collection(results) = step.results() else {
        panic!("results are a sequence");
    };
    let Value::Composite(first) = &results.elements()[0] else {
        panic!("a result record");
    };
    let quire_semantic_value::declaration::CompositeShape::Record(fields) = program
        .types()
        .composite(first.declaration().clone())
        .expect("the declared result record")
        .shape()
    else {
        panic!("a record");
    };
    let position = fields
        .iter()
        .position(|field| field.name() == "outcome")
        .expect("an outcome field");
    let quire_exact::FieldValue::Present(Value::Union(outcome)) = &first.slots()[position] else {
        panic!("an outcome union");
    };
    outcome.member().identifier().to_owned()
}

/// The request at tick 0 with no acknowledgement: the `[0, 3]` window is open
/// through tick 2 and closed, unanswered, once tick 3 is covered.
#[trace("TC-033", "FR-017-AC-1")]
#[test]
fn smoke_the_prepared_step_one_program_runs_in_the_runtime() {
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel).expect("prepares");
    assert_eq!(outcome_after(&program, 2), "Pending");
    assert_eq!(outcome_after(&program, 3), "False");
    assert_eq!(outcome_after(&program, 4), "False");
}

#[trace("TC-033", "FR-019-AC-1")]
#[test]
fn the_prepared_program_declares_its_tick_unit_in_the_unit_table() {
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel).expect("prepares");
    use quire_semantic_value::unit::NominalUnitForm;
    let layout = program.layout();
    let node = program
        .types()
        .units()
        .nominal_node(&layout.tick_unit)
        .expect("the tick unit is declared in the unit table");
    let NominalUnitForm::Unit { dimension, .. } = &node.form else {
        panic!("the tick unit is a unit node");
    };
    assert_eq!(dimension, &layout.time_dimension);
}

/// The environment's own ceilings and reservation denials are work and
/// storage failures; only a declaration refusal is a projection fault.
#[trace("TC-033", "FR-017-AC-4")]
#[test]
fn environment_ceilings_and_denials_map_to_work_and_storage_codes() {
    use quire_driver::monitor::MonitorPreparationError as Error;
    use quire_semantic_value::declaration::{
        EnvironmentFailure, EnvironmentLimit, EnvironmentLimitKind, StorageRequest, StorageUnit,
    };
    let code = |failure| Error::TypeEnvironment(Box::new(failure)).code();
    let limit = EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 1, 2);
    let request = StorageRequest::new(8, StorageUnit::Bytes);
    assert_eq!(
        code(EnvironmentFailure::Limit(limit)),
        MonitorPreparationErrorCode::Work
    );
    assert_eq!(
        code(EnvironmentFailure::Allocation(request)),
        MonitorPreparationErrorCode::Storage
    );
    assert_eq!(
        code(EnvironmentFailure::Capacity(request)),
        MonitorPreparationErrorCode::Storage
    );
}

/// Lowering relies on QSL declaring every source-record member required.
#[trace("TC-033", "FR-017-AC-1")]
#[test]
fn the_producers_source_records_declare_every_member_required() {
    let cancel = Cancel::new();
    let mut rows = collect_clause("C", STEP_ONE, LIMITS, &cancel).expect("genuine root admits");
    rows.read_closure(&cancel, |_, _, facts| {
        assert!(!facts.source_records.is_empty());
        for record in facts.source_records {
            for member in &record.members {
                assert_eq!(
                    member.presence,
                    quire_exact::Presence::Required,
                    "{} declares {} optional",
                    record.qualified_declaration.join("."),
                    member.admitted_name
                );
            }
        }
    })
    .expect("closure reads");
}

/// A clause conjoining `terms` atoms, so the generated step functions nest
/// one level per subformula.
fn deep_chain_clause(terms: usize) -> String {
    let atoms = vec!["holds(candidate.versionNumber = 2)"; terms].join(" and ");
    format!(
        r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    always ({atoms} implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}}"#
    )
}

/// Lower the clause's entries under `storage` bytes and report the outcome.
fn lower_under(
    clause: &str,
    storage: u64,
) -> Result<usize, quire_driver::monitor::MonitorPreparationError> {
    let cancel = Cancel::new();
    // The package read needs more than the lowering allowance under test.
    let ample = MonitorPreparationLimits {
        work: 1 << 40,
        storage: 1 << 34,
    };
    let mut rows = collect_clause("C", clause, ample, &cancel).expect("genuine root admits");
    rows.read_closure(&cancel, |authority, closure, facts| {
        quire_driver::monitor::lowering::lower_entries(
            authority.admitted_package(),
            closure,
            facts,
            MonitorPreparationLimits {
                work: 1 << 40,
                storage,
            },
            &cancel,
        )
        .map(|bodies| bodies.expressions.len())
    })
    .expect("closure reads")
}

/// Under one allowance a one-atom clause lowers and a forty-atom clause stops
/// with a typed storage code; a larger allowance lowers it. Recursion depth
/// reserves storage per level, so growth ends in this stop, not a stack overflow.
#[trace("TC-033", "FR-017-AC-3")]
#[test]
fn a_deeper_clause_stops_at_the_storage_allowance_not_the_stack() {
    let allowance = 8 << 20;
    lower_under(&deep_chain_clause(1), allowance).expect("a shallow clause lowers");
    let error = lower_under(&deep_chain_clause(40), allowance)
        .err()
        .expect("the deeper clause needs more than the allowance");
    assert_eq!(error.code(), MonitorPreparationErrorCode::Storage);
    lower_under(&deep_chain_clause(40), 64 << 20).expect("an ample allowance lowers");
}

/// The binder-bearing constructs of the real plan pass `Program::try_new`:
/// sibling and nested lets, folds, maps and filters, each binder in its own slot.
#[trace("TC-033", "FR-017-AC-1")]
#[test]
fn binders_lower_to_distinct_slots_and_validate() {
    use quire_contract_runtime::monitor::{ExpressionId, Operation};
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel).expect("prepares and validates");
    let mut lets = Vec::new();
    let (mut folds, mut maps, mut filters, mut nested) = (0, 0, 0, 0);
    let mut index = 0;
    while let Some(expression) = program.expression(ExpressionId(index)) {
        match &expression.operation {
            Operation::Let { binding, body, .. } => {
                lets.push(*binding);
                if matches!(
                    program.expression(*body).map(|inner| &inner.operation),
                    Some(Operation::Let { .. })
                ) {
                    nested += 1;
                }
            }
            Operation::SequenceFold { .. } => folds += 1,
            Operation::SequenceMap { .. } => maps += 1,
            Operation::SequenceFilter { .. } => filters += 1,
            _ => {}
        }
        index += 1;
    }
    assert!(nested > 0, "a let directly nested in a let");
    assert!(folds > 0 && maps > 0 && filters > 0);
    // Sibling lets of one function bind different slots.
    let mut slots: Vec<_> = lets.iter().map(|l| (l.function.0, l.position)).collect();
    let total = slots.len();
    slots.sort_unstable();
    slots.dedup();
    assert_eq!(slots.len(), total, "no binder slot is declared twice");
    assert!(
        slots.windows(2).any(|pair| pair[0].0 == pair[1].0),
        "some function has sibling or nested lets"
    );
}
