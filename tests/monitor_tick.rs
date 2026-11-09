// SPDX-License-Identifier: AGPL-3.0-or-later
//! TC-200 step 1: the prepared tick monitor on a 20-bit, 1 MHz counter.
//!
//! The expected outcomes are the TC-200 expected results, not the output of
//! any other run.

use std::rc::Rc;

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{PlanAuthoritySelection, PlanAuthorityValidationCaps};
use qsl_replay::MonitorTarget;
use quire_contract_model::CheckedNodeId;
use quire_contract_runtime::monitor::{
    build, BuildPlan, Event, InputContext, Monitor, Program, Reading, StepBudget, Watermark,
};
use quire_driver::monitor::{prepare_monitor, MonitorPreparationLimits};
use quire_exact::{Cancel, FieldValue, Integer, Rational, ScalarLimits, Value};

/// Native PlanRoot support shared by monitor integration suites.
#[path = "common/monitor.rs"]
pub mod monitor_fixture;
/// Genuine observation-owner progress admission.
#[path = "common/observation.rs"]
pub mod observation;

const CLAUSE: &str = r#"temporal C using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    always (holds(candidate.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(candidate.versionNumber = 3))
}"#;

const LIMITS: MonitorPreparationLimits = MonitorPreparationLimits {
    work: 100_000_000,
    storage: 100_000_000,
};

/// The microsecond tick unit of the 1 MHz counter.
const MICROSECONDS: &str = "unit us : Time = rational(1, 1000000) * second;";

fn ratio(numerator: u64) -> Rational {
    Rational::from_integer(Integer::from(numerator))
}

/// Prepare the TC-200 clause for a 20-bit, 1 MHz counter with the given
/// clock uncertainty in microseconds.
fn prepare(uncertainty: u64) -> Rc<Program> {
    let target = MonitorTarget {
        counter: "counter".to_owned(),
        unit: "us".to_owned(),
        width_bits: 20,
        uncertainty: ratio(uncertainty),
        max_event_rate: Some(ratio(2)),
        // Wide enough for the 3001 us reading gap of the scenario.
        max_reading_gap: 10_000,
        max_activations: 8,
    };
    let fixture = monitor_fixture::fixture_for("C", "counter", CLAUSE, MICROSECONDS, "us", &target);
    // The base Time dimension comes from the strictly admitted package's
    // own dimension declaration, not from the closure under test.
    let time: CheckedNodeId = {
        let mut dimensions = fixture
            .package
            .resolved_graph()
            .nodes
            .iter()
            .filter(|node| &*node.semantic_form == "dimension");
        let node = dimensions.next().expect("the declared Time dimension");
        assert!(dimensions.next().is_none(), "one declared dimension");
        node.node_id.clone()
    };
    let root = fixture.into_root();
    let (progress, subject) = observation::admitted(Some(vec![observation::row(&root)]));
    let source = root.checked_unit().source().identity().clone();
    let claim = root.plan().subject().clone();
    let clock = observation::clock();
    let observation_source = observation::source();
    prepare_monitor(
        root,
        PlanAuthoritySelection {
            source: &source,
            claim: &claim,
            observation_subject: &subject,
            observation_clock: &clock,
            observation_source: &observation_source,
        },
        Some(progress),
        PlanAuthorityValidationCaps {
            read: monitor_fixture::read_limits(),
            order: quire_canonical::Limits::new(1 << 20),
        },
        &time,
        LIMITS,
        &Cancel::new(),
    )
    .expect("the clause prepares")
}

fn budget() -> StepBudget {
    let big = 1_000_000;
    StepBudget {
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
    }
}

/// One obligation's identity and settled state.
#[derive(Debug, Eq, PartialEq)]
struct Settled {
    subformula: Integer,
    origin_tick: Integer,
    outcome: String,
}

/// The value of the named field of a composite, by its declared name.
fn field<'v>(program: &Program, value: &'v Value, name: &str) -> &'v Value {
    let Value::Composite(composite) = value else {
        panic!("a composite value");
    };
    let declaration = program
        .types()
        .composite(composite.declaration().clone())
        .expect("a declared composite");
    let quire_semantic_value::declaration::CompositeShape::Record(fields) = declaration.shape()
    else {
        panic!("a record");
    };
    let position = fields
        .iter()
        .position(|declared| declared.name() == name)
        .unwrap_or_else(|| {
            panic!(
                "field {name} in {:?}",
                fields
                    .iter()
                    .map(|f| f.name().to_owned())
                    .collect::<Vec<_>>()
            )
        });
    let FieldValue::Present(present) = &composite.slots()[position] else {
        panic!("field {name} is present");
    };
    present
}

fn settled(program: &Program, results: &Value) -> Vec<Settled> {
    let Value::Collection(results) = results else {
        panic!("results are a sequence");
    };
    results
        .elements()
        .iter()
        .map(|result| {
            let instance = field(program, result, "instance");
            let Value::Integer(subformula) = field(program, instance, "subformula") else {
                panic!("integer subformula");
            };
            let Value::Integer(origin_tick) = field(program, instance, "origin") else {
                panic!("integer origin");
            };
            let Value::Union(outcome) = field(program, result, "outcome") else {
                panic!("an outcome union");
            };
            Settled {
                subformula: subformula.clone(),
                origin_tick: origin_tick.clone(),
                outcome: outcome.member().identifier().to_owned(),
            }
        })
        .collect()
}

fn context(program: &Program) -> InputContext<'_> {
    let layout = program.layout();
    InputContext {
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
    }
}

struct Run {
    program: Rc<Program>,
    monitor: Monitor,
}

impl Run {
    fn new(program: &Rc<Program>) -> Self {
        let monitor = build(BuildPlan {
            program: Rc::clone(program),
            budget: budget(),
        })
        .expect("initialization yields a monitor");
        Self {
            program: Rc::clone(program),
            monitor,
        }
    }

    /// An event at counter reading `tick` observing the requested atoms.
    fn event(&mut self, tick: u64, req: bool, ack: bool) -> Vec<Settled> {
        let program = Rc::clone(&self.program);
        let layout = program.layout();
        let captures = program
            .types()
            .record(
                layout.inputs.key.clone(),
                vec![
                    ("a2", FieldValue::Present(Value::Boolean(req))),
                    ("a4", FieldValue::Present(Value::Boolean(ack))),
                ],
            )
            .expect("captures");
        let step = self
            .monitor
            .event(Event {
                reading: Reading {
                    context: context(&program),
                    value: tick,
                },
                inputs: &captures,
            })
            .expect("admitted event");
        settled(&program, step.results())
    }

    /// An admitted progress assertion covering ticks through `boundary`.
    fn watermark(&mut self, boundary: u64) -> Vec<Settled> {
        let program = Rc::clone(&self.program);
        let layout = program.layout();
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
        let boundary = Integer::from(boundary);
        let step = self
            .monitor
            .watermark(Watermark {
                context: context(&program),
                progress_context: &layout.progress_context,
                assertion: b"admitted progress assertion",
                inclusive_boundary: &boundary,
                scope: &scope,
            })
            .expect("admitted watermark");
        settled(&program, step.results())
    }
}

/// A settled obligation as TC-200 names them: the root `always` instance is
/// subformula 0 and the `eventually` interval is subformula 3, both from the
/// request at tick 0.
fn settled_at_zero(subformula: u64, outcome: &str) -> Settled {
    Settled {
        subformula: Integer::from(subformula),
        origin_tick: Integer::zero(),
        outcome: outcome.to_owned(),
    }
}

/// Request at tick 0, first acknowledgement at `ack` ticks.
fn after_ack(uncertainty: u64, ack: u64) -> Vec<Settled> {
    let program = prepare(uncertainty);
    let mut run = Run::new(&program);
    assert!(
        run.event(0, true, false).is_empty(),
        "the request alone settles nothing"
    );
    // TC-200 names the obligations of the request at tick 0; the later
    // event starts its own, unrelated instances.
    let mut results = run.event(ack, false, true);
    results.retain(|result| result.origin_tick == Integer::zero());
    results
}

#[trace("TC-200", "FR-276")]
#[test]
fn ack_at_2998_is_definitely_true() {
    assert_eq!(after_ack(0, 2998), vec![settled_at_zero(3, "True")]);
}

#[trace("TC-200", "FR-276")]
#[test]
fn ack_at_3000_is_indeterminate() {
    assert_eq!(
        after_ack(0, 3000),
        vec![
            settled_at_zero(0, "Indeterminate"),
            settled_at_zero(3, "Indeterminate")
        ]
    );
}

#[trace("TC-200", "FR-276")]
#[test]
fn ack_at_3001_is_definitely_false() {
    assert_eq!(
        after_ack(0, 3001),
        vec![settled_at_zero(0, "False"), settled_at_zero(3, "False")]
    );
}

#[trace("TC-200", "FR-276")]
#[test]
fn ack_at_2998_with_two_microseconds_of_uncertainty_is_indeterminate() {
    assert_eq!(
        after_ack(2, 2998),
        vec![
            settled_at_zero(0, "Indeterminate"),
            settled_at_zero(3, "Indeterminate")
        ]
    );
}

#[trace("TC-200", "FR-276")]
#[test]
fn pending_stays_distinct_from_indeterminate_while_the_window_is_open() {
    let program = prepare(0);
    let mut run = Run::new(&program);
    run.event(0, true, false);
    let pending = run.watermark(2998);
    assert_eq!(
        pending,
        vec![settled_at_zero(0, "Pending"), settled_at_zero(3, "Pending")]
    );
    assert!(pending
        .iter()
        .all(|result| result.outcome != "Indeterminate"));
}
