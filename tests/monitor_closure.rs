// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plan-authority admission and selected type closure from a genuine PlanRoot.

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{PlanAuthoritySelection, PlanAuthorityValidationCaps};
use quire_driver::monitor::{
    collect_monitor_closure_rows, MonitorPreparationErrorCode, MonitorPreparationLimits,
};
use quire_exact::Cancel;

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

#[trace("TC-033", "FR-017-AC-1")]
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

#[trace("TC-033", "FR-019-AC-1")]
#[test]
fn the_step_one_plan_derives_its_layout_from_the_closure() {
    let cancel = Cancel::new();
    let mut rows = collect_clause("C", STEP_ONE, LIMITS, &cancel).expect("genuine root admits");
    let layout = derive(&mut rows, &cancel, false).expect("layout derives");
    assert!(!layout.package_lock.is_empty());
    assert!(!layout.clock_binding.is_empty());
    assert!(!layout.admitted_order.is_empty());
    assert!(layout.max_reading_gap > 0);
    assert_eq!(layout.buffers.len(), 1);
}

#[trace("TC-033", "FR-019-AC-4")]
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
    let time = collect_clause(selected, additional, LIMITS, cancel)
        .expect("genuine root admits")
        .read_closure(cancel, |_, closure, _| {
            closure.model_types().dimensions()[0]
                .structural_node
                .clone()
        })
        .expect("closure reads");
    let fixture = monitor_fixture::fixture(selected, "counter", additional);
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

#[trace("TC-033", "FR-019-AC-1")]
#[test]
fn the_step_one_plan_prepares_a_checked_program() {
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel);
    if let Err(error) = &program {
        panic!("prepare failed: {error:?} / {error}");
    }
}

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
    let quire_exact::FieldValue::Present(Value::Union(outcome)) = &first.slots()[1] else {
        panic!("an outcome union");
    };
    outcome.member().identifier().to_owned()
}

/// The request at tick 0 with no acknowledgement: the `[0, 3]` window is open
/// through tick 2 and closed, unanswered, once tick 3 is covered.
#[trace("TC-033", "FR-019-AC-1")]
#[test]
fn the_prepared_step_one_program_runs_in_the_runtime() {
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel).expect("prepares");
    assert_eq!(outcome_after(&program, 2), "Pending");
    assert_eq!(outcome_after(&program, 3), "False");
    assert_eq!(outcome_after(&program, 4), "False");
}

#[trace("TC-033", "FR-017-AC-1")]
#[test]
fn the_prepared_program_declares_its_tick_unit_in_the_unit_table() {
    let cancel = Cancel::new();
    let program = prepare("C", STEP_ONE, &cancel).expect("prepares");
    assert!(program
        .types()
        .units()
        .nominal_node(&program.layout().tick_unit)
        .is_some());
}
