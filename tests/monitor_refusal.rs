// SPDX-License-Identifier: AGPL-3.0-or-later
//! Functional diagnostic derivation from QSL emission and strict IR admission.

use std::{path::PathBuf, process::Command, sync::LazyLock};

use ix_trace_rs::trace;
use qsl_foundation::SourceIdentity;
use qsl_package::{emit_checked, emit_monitor_plan_over, Emission, MonitorPlanWire, V2ReadLimits};
use qsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedNodeKind, CheckedOccurrence, CheckedOccurrenceRole,
    CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedTerm,
};
use quire_driver::monitor_refusal::{
    target_mismatch_at, unsupported_at, MonitorLocusErrorCode, TargetPremise,
};
use quire_exact::{Cancel, Integer, Rational};

// These ordinary functions only supply admitted diagnostic data. They are
// neither a temporal algorithm nor evidence of monitor preparation/build.
const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "agent-ix/config-version";
dimension Time;
unit second : Time = rational(1, 1);
unit ms : Time = rational(1, 1000) * second;
time Selected dense unit ms;
temporal Diagnostic using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}
record DActivation { subformula: Integer; origin_tick: Integer; status: Integer; }
record DState { now: Integer; history_start: Integer; buffer_0: Sequence<Integer>; activations: Sequence<DActivation>; event_position: Integer; }
record DInputs { flag: Boolean; }
record DScope { flag: Boolean; }
record DInstance { subject: String; subformula: Integer; origin_tick: Integer; }
record DGap { gap: Integer; max_reading_gap: Integer; }
record DLimit { limit: String; amount: Integer; setting: String; }
union DOutcome { Failed(DGap), False, Incomplete(DLimit), Indeterminate, Pending, True }
record DResult { instance: DInstance; outcome: DOutcome; }
type DResults = Sequence<DResult>;
tuple DStep(DState, DResults);
function emptyResults using v(): DResults pure { sequence[] }
function fixtureInit using v(): DState pure {
    let initial = DState { now: 0, history_start: 0 - 1, buffer_0: sequence[], activations: sequence[], event_position: 0 } in initial
}
function fixtureEvent using v(s: DState, delta: Integer, i: DInputs): DStep pure { let pair = DStep(s, emptyResults()) in pair }
function fixtureWatermark using v(s: DState, delta: Integer, progress: DScope): DStep pure { let pair = DStep(s, emptyResults()) in pair }
function fixtureGap using v(s: DState, delta: Integer): DStep pure { let pair = DStep(s, emptyResults()) in pair }
function plus using v(a: Integer, b: Integer): Integer pure { a + b }
function remainder using v(a: Integer, b: Integer): Integer pure { a mod b }
function narrowed using v(a: Float64[nearest-even]): Float32[toward-negative] pure { convert<Float32[toward-negative]>(a) }
"#;

struct Fixture {
    package: Box<CheckedPackageV2>,
}
static FIXTURE: LazyLock<Fixture> = LazyLock::new(fixture);

fn model_document() -> Vec<u8> {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let manifest = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == "qsl-replay")
        .unwrap()["manifest_path"]
        .as_str()
        .unwrap();
    let path = PathBuf::from(manifest);
    std::fs::read(
        path.parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/config-version/model.semantic-ir.json"),
    )
    .unwrap()
}

fn admit(emission: &Emission, evidence: &CheckedPackageEvidence) -> Box<CheckedPackageV2> {
    match read_checked_package(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        evidence,
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => {
            let wire: serde_json::Value =
                serde_json::from_slice(emission.package().bytes()).unwrap();
            let nodes = wire["semantic_graph"]["nodes"].as_array().unwrap();
            let root = nodes
                .iter()
                .find(|node| node["semantic_form"] == "monitor_plan")
                .unwrap();
            let binding = |name: &str| {
                root["body"]["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|member| member["name"] == name)
                    .unwrap()["value"]["target"]
                    .clone()
            };
            let state = binding("state");
            let init = binding("init");
            let init = nodes.iter().find(|node| node["node_id"] == init).unwrap();
            let result = nodes
                .iter()
                .find(|node| node["node_id"] == init["semantic_type"])
                .unwrap();
            panic!(
                "strict admission: {other:?}; state={state}; init result={} form={}",
                init["semantic_type"], result["semantic_form"]
            );
        }
    }
}

fn fixture() -> Fixture {
    let document = model_document();
    let model: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let identity = model["package"]["identity"].as_str().unwrap();
    let text = SOURCE.replace("agent-ix/config-version", identity);
    let packages = package_input([document.as_slice()]).unwrap();
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let parsed = parse(
        &ParseRequest {
            source: &SourceIdentity::new("agent-ix", "monitor-diagnostic"),
            path: "monitor-diagnostic.native",
            bytes: text.as_bytes(),
        },
        limits.source,
        &cancel,
    )
    .unwrap()
    .into_value();
    let selected = select(&parsed, &packages, limits.model, &cancel)
        .unwrap()
        .into_value();
    let checked = check(
        &parsed,
        "agent-ix/monitor-diagnostic".to_owned(),
        &selected,
        &DependencyInput::default(),
        limits,
        &cancel,
    )
    .unwrap()
    .into_value();
    let base = emit_checked(checked.package(), limits.package.emit()).unwrap();
    let mut evidence = base.evidence().clone();
    evidence
        .insert_domain_package_document(identity, document)
        .unwrap();
    let clause = &checked.package().graph().temporal_clauses()[0];
    let operators = clause
        .clause()
        .operators()
        .iter()
        .filter(|operator| operator.interval.is_some())
        .collect::<Vec<_>>();
    assert_eq!(operators.len(), 1);
    let buffers = [(operators[0].node.index(), 7)];
    let emission = emit_monitor_plan_over(
        &base,
        LibraryName::new("agent-ix/monitor-diagnostic").unwrap(),
        &evidence,
        &MonitorPlanWire {
            clause: &clause.identity(),
            formula: clause.clause().formula(),
            counter: "diagnostic-counter",
            counter_width_bits: 16,
            max_reading_gap: 100,
            max_event_rate: &Rational::from_integer(Integer::from(2_u64)),
            uncertainty: &Rational::new(Integer::one(), Integer::from(2_u64)).unwrap(),
            buffers: &buffers,
            tick_unit: "ms",
            activation_capacity: 3,
            state: "DState",
            inputs: "DInputs",
            scope: "DScope",
            result: "DResult",
            entries: [
                "fixtureInit",
                "fixtureEvent",
                "fixtureWatermark",
                "fixtureGap",
            ],
            renamed_records: &["DActivation", "DInstance", "DLimit"],
            renames: &[("origin_tick", "origin"), ("amount", "value")],
        },
        V2ReadLimits::bounded(1 << 24),
        limits.package.emit(),
    )
    .expect("the public QSL writer emits the checked diagnostic root");
    let package = admit(&emission, &evidence);
    assert_eq!(package.monitor_plans().len(), 1);
    Fixture { package }
}

fn operation<'a>(
    package: &'a CheckedPackageV2,
    identity: &str,
) -> (&'a CheckedNodeId, CheckedNodeKind) {
    let matches = package.graph().nodes.iter().zip(package.node_kinds()).filter(|(node, _)| matches!(package.term(&node.node_id), Some(CheckedTerm::Application { operation, .. }) if operation.identity() == Some(identity))).collect::<Vec<_>>();
    let [(node, kind)] = matches.as_slice() else {
        panic!("one exact operation {identity}");
    };
    (&node.node_id, **kind)
}

// Independent path resolution uses the actual containing terms, never a
// constructor output or a diagnostic's operation record as the expected value.
fn selected<'a>(
    package: &'a CheckedPackageV2,
    node: &CheckedNodeId,
    path: &[usize],
) -> &'a CheckedTerm {
    path.iter()
        .fold(package.term(node).unwrap(), |term, index| match term {
            CheckedTerm::Aggregate { members } => &members[*index],
            CheckedTerm::Application { arguments, .. } => &arguments[*index],
            CheckedTerm::Binding { value, .. } => {
                assert_eq!(*index, 0);
                value.as_ref()
            }
            _ => panic!("the independent expected path ends before {index}"),
        })
}

#[trace("TC-030", "FR-016-AC-1", "FR-016-AC-3", "FR-016-AC-6", "FR-016-AC-7")]
#[test]
fn every_target_premise_names_its_actual_admitted_value() {
    let package = &FIXTURE.package;
    let before = serde_json::to_value((package.graph(), package.source_map())).unwrap();
    let plan = package.monitor_plans().next().unwrap();
    let root = plan.node_id();
    let generated = package
        .graph()
        .nodes
        .iter()
        .find(|node| &node.node_id == root)
        .unwrap();
    assert_eq!(
        generated.occurrences,
        vec![CheckedOccurrence {
            role: CheckedOccurrenceRole::Generated,
            ordinal: 0
        }]
    );
    for (premise, name, slot, buffer) in [
        (TargetPremise::Counter, "counter", 12, None),
        (TargetPremise::CounterWidth, "counter_width_bits", 8, None),
        (TargetPremise::TickUnit, "tick_unit", 13, None),
        (TargetPremise::MaxReadingGap, "max_reading_gap", 9, None),
        (TargetPremise::MaxEventRate, "max_event_rate", 10, None),
        (TargetPremise::Uncertainty, "uncertainty", 14, None),
        (TargetPremise::BufferCapacity, "capacity", 11, Some(0)),
        (
            TargetPremise::ActivationCapacity,
            "activation_capacity",
            16,
            None,
        ),
    ] {
        let actual = target_mismatch_at(package, root, premise, buffer, None).unwrap();
        let explicit = target_mismatch_at(
            package,
            root,
            premise,
            buffer,
            Some(&generated.occurrences[0]),
        )
        .unwrap();
        assert_eq!(
            explicit.locus.occurrence,
            Some(generated.occurrences[0].clone())
        );
        let path = if buffer.is_some() {
            vec![slot, 0, 0, 1, 0]
        } else {
            vec![slot, 0]
        };
        assert_eq!(actual.premise, premise);
        assert_eq!(actual.locus.node, *root);
        assert_eq!(actual.locus.structural_key, *plan.structural_key());
        assert_eq!(actual.locus.body_path, path, "{premise:?}");
        assert_eq!(actual.locus.occurrence, None);
        let CheckedTerm::Binding {
            name: actual_name,
            value,
        } = selected(package, root, &path[..path.len() - 1])
        else {
            panic!("the value's original binding");
        };
        assert_eq!(actual_name.as_ref(), name);
        assert_eq!(selected(package, root, &path), value.as_ref());
    }
    let cap =
        target_mismatch_at(package, root, TargetPremise::BufferCapacity, Some(0), None).unwrap();
    let CheckedTerm::Reference { target } = selected(package, root, &[11, 0, 0, 0, 0]) else {
        panic!("original buffer owner");
    };
    assert_eq!(target, plan.buffers().next().unwrap().subformula());
    assert_eq!(cap.locus.body_path, vec![11, 0, 0, 1, 0]);
    assert_eq!(
        before,
        serde_json::to_value((package.graph(), package.source_map())).unwrap()
    );
}

#[trace("TC-030", "FR-016-AC-2", "FR-016-AC-5", "FR-016-AC-7")]
#[test]
fn operations_and_native_child_paths_retain_the_original_terms() {
    let package = &FIXTURE.package;
    for identity in [
        "quire.op.integer.add",
        "quire.op.integer.mod",
        "quire.op.ieee.to_float32",
    ] {
        let (node, kind) = operation(package, identity);
        let CheckedTerm::Application {
            operation: expected,
            arguments,
            ..
        } = package.term(node).unwrap()
        else {
            panic!("actual application");
        };
        if identity == "quire.op.ieee.to_float32" {
            assert!(
                expected.mode().unwrap().is_some(),
                "the real rounding conversion carries its owner mode"
            );
        }
        if identity == "quire.op.ieee.to_float32" {
            assert!(!expected.law_definitions().unwrap().is_empty());
            assert!(expected.member_declaration().unwrap().is_some());
        }
        let actual = unsupported_at(package, node, &[], kind, None).unwrap();
        assert_eq!(actual.operation.as_ref(), Some(expected));
        assert_eq!(actual.kind, kind);
        assert!(
            !arguments.is_empty(),
            "the actual application has child references"
        );
        for (index, argument) in arguments.iter().enumerate() {
            let CheckedTerm::Reference { target } = argument else {
                panic!("the actual source operation uses an admitted child reference, not a nested application");
            };
            let child = unsupported_at(package, node, &[index], kind, None).unwrap();
            assert_eq!(child.operation, None, "the nonempty path names a reference");
            assert_eq!(child.locus.body_path, vec![index]);
            assert_eq!(
                unsupported_at(package, node, &[index, 0], kind, None)
                    .unwrap_err()
                    .code,
                MonitorLocusErrorCode::InvalidBodyPath
            );
            let target_kind = package
                .graph()
                .nodes
                .iter()
                .zip(package.node_kinds())
                .find(|(original, _)| &original.node_id == target)
                .unwrap()
                .1;
            let reached = unsupported_at(package, target, &[], *target_kind, None).unwrap();
            assert_eq!(reached.kind, *target_kind);
            assert_eq!(reached.locus.node, *target);
            assert!(reached.locus.body_path.is_empty());
        }
        assert_eq!(
            unsupported_at(package, node, &[arguments.len()], kind, None)
                .unwrap_err()
                .code,
            MonitorLocusErrorCode::InvalidBodyPath
        );
    }
    let plan = package.monitor_plans().next().unwrap();
    let kind = package
        .graph()
        .nodes
        .iter()
        .zip(package.node_kinds())
        .find(|(node, _)| &node.node_id == plan.init().node_id())
        .unwrap()
        .1;
    let init = plan.init().node_id();
    assert!(matches!(
        selected(package, init, &[1]),
        CheckedTerm::Binding { .. }
    ));
    assert!(matches!(
        selected(package, init, &[1, 0]),
        CheckedTerm::Reference { .. }
    ));
    assert!(matches!(
        selected(package, init, &[0, 0]),
        CheckedTerm::Aggregate { .. }
    ));
    for path in [vec![1], vec![1, 0], vec![0, 0]] {
        let value = unsupported_at(package, init, &path, *kind, None).unwrap();
        assert_eq!(
            value.operation, None,
            "the real nonempty path selects a nonapplication"
        );
        assert_eq!(value.locus.body_path, path);
    }
    for path in [&[1, 1][..], &[1, 0, 0][..]] {
        assert_eq!(
            unsupported_at(package, plan.init().node_id(), path, *kind, None)
                .unwrap_err()
                .code,
            MonitorLocusErrorCode::InvalidBodyPath
        );
    }
}

#[trace("TC-030", "FR-016-AC-4", "FR-016-AC-6", "FR-016-AC-7")]
#[test]
fn invalid_indices_occurrences_and_selectors_keep_distinct_typed_failures() {
    let package = &FIXTURE.package;
    let root = package.monitor_plans().next().unwrap().node_id();
    for (premise, index) in [
        (TargetPremise::BufferCapacity, None),
        (TargetPremise::BufferCapacity, Some(1)),
        (TargetPremise::Counter, Some(0)),
    ] {
        assert_eq!(
            target_mismatch_at(package, root, premise, index, None)
                .unwrap_err()
                .code,
            MonitorLocusErrorCode::InvalidPremiseIndex
        );
    }
    let (node, kind) = operation(package, "quire.op.integer.mod");
    assert_eq!(
        target_mismatch_at(package, node, TargetPremise::Counter, None, None)
            .unwrap_err()
            .code,
        MonitorLocusErrorCode::NotMonitorRoot
    );
    let occurrence = package
        .graph()
        .nodes
        .iter()
        .find(|original| &original.node_id == node)
        .unwrap()
        .occurrences
        .first()
        .expect("a real source operation occurrence");
    let original = unsupported_at(package, node, &[], kind, Some(occurrence)).unwrap();
    assert_eq!(original.locus.occurrence.as_ref(), Some(occurrence));
    let root_foreign = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: 0,
    };
    assert_eq!(
        target_mismatch_at(
            package,
            root,
            TargetPremise::Uncertainty,
            None,
            Some(&root_foreign)
        )
        .unwrap_err()
        .code,
        MonitorLocusErrorCode::OccurrenceMismatch
    );
    let foreign = CheckedOccurrence {
        role: CheckedOccurrenceRole::Expression,
        ordinal: u64::MAX,
    };
    assert_eq!(
        unsupported_at(package, node, &[], kind, Some(&foreign))
            .unwrap_err()
            .code,
        MonitorLocusErrorCode::OccurrenceMismatch
    );
    assert_eq!(
        target_mismatch_at(
            package,
            root,
            TargetPremise::Counter,
            None,
            Some(occurrence)
        )
        .unwrap_err()
        .code,
        MonitorLocusErrorCode::OccurrenceMismatch
    );
    let mut unknown = node.clone();
    unknown.package = "not-the-selected-package".into();
    for error in [
        unsupported_at(package, &unknown, &[], kind, None).unwrap_err(),
        target_mismatch_at(package, &unknown, TargetPremise::Uncertainty, None, None).unwrap_err(),
    ] {
        assert_eq!(error.code, MonitorLocusErrorCode::UnknownNode);
        assert_eq!(error.node, unknown);
    }
}

#[trace("TC-030", "FR-016-AC-1", "FR-016-AC-5", "FR-016-AC-6")]
#[test]
fn owned_diagnostics_outlive_the_package_and_occurrence_buffers() {
    let (actual, expected_key, expected_operation, expected_occurrence) = {
        let owned = fixture();
        let (node, kind) = operation(&owned.package, "quire.op.integer.mod");
        let source_node = owned
            .package
            .graph()
            .nodes
            .iter()
            .find(|original| &original.node_id == node)
            .unwrap();
        let occurrence = source_node.occurrences[0].clone();
        let CheckedTerm::Application { operation, .. } = owned.package.term(node).unwrap() else {
            panic!("selected operation");
        };
        let actual = unsupported_at(&owned.package, node, &[], kind, Some(&occurrence)).unwrap();
        (
            actual,
            owned.package.structural_key(node).unwrap().clone(),
            operation.clone(),
            occurrence,
        )
    };
    assert_eq!(actual.locus.structural_key, expected_key);
    assert_eq!(actual.operation, Some(expected_operation));
    assert_eq!(actual.locus.occurrence, Some(expected_occurrence));
}

#[test]
#[trace("FR-016-AC-9")]
fn bounded_copy_preserves_actual_resolved_target_axes() {
    use quire_driver::{monitor::MonitorPreparationLimits, monitor_refusal::copy_monitor_locus};
    let package = &FIXTURE.package;
    let plan = package.monitor_plans().next().unwrap();
    let actual = target_mismatch_at(
        package,
        plan.node_id(),
        TargetPremise::BufferCapacity,
        Some(0),
        None,
    )
    .unwrap();
    let copied = copy_monitor_locus(
        &actual.locus.node,
        &actual.locus.structural_key,
        &actual.locus.body_path,
        actual.locus.occurrence.as_ref(),
        MonitorPreparationLimits {
            work: u64::MAX,
            storage: u64::MAX,
        },
        &Cancel::new(),
    )
    .unwrap();
    assert_eq!(copied.locus, actual.locus);
    assert_eq!(copied.locus.body_path.last(), Some(&0));
    assert_eq!(copied.locus.structural_key, *plan.structural_key());
}
