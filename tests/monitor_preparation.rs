// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real producer input to the driver's independent selection boundary.

use std::{path::PathBuf, process::Command, sync::LazyLock};

use qsl_analyze::monitor_root::{emit_plan_root, PlanRootRequest};
use qsl_foundation::SourceIdentity;
use qsl_package::Emission;
use qsl_replay::{spine::SpineLimits, MonitorTarget, TimingObligationSettings};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedOccurrenceRole, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedRational, CheckedSelectionRole, CheckedTerm,
    SelectedExactIdentities, SelectedFactRows,
};
use quire_driver::monitor::{
    admit_monitor_selection, project_monitor_initializer, MonitorPreparationError,
    MonitorPreparationErrorCode, MonitorPreparationLimits, MonitorPreparationRequest,
    MonitorSelection, MonitorSelectionField,
};
use quire_driver::monitor_refusal::TargetPremise;
use quire_exact::{Cancel, CancelCause, Integer, Rational};

struct Fixture {
    package: Box<CheckedPackageV2>,
    emission: Emission,
    buffers: Vec<(CheckedNodeId, Box<str>)>,
}

static FIXTURE: LazyLock<Fixture> = LazyLock::new(fixture);

fn fixture() -> Fixture {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output().expect("resolved dependency metadata");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let manifest = metadata["packages"].as_array().unwrap().iter()
        .find(|package| package["name"] == "qsl-replay").unwrap()["manifest_path"].as_str().unwrap();
    let path = PathBuf::from(manifest);
    let document = std::fs::read(path.parent().unwrap().parent().unwrap()
        .join("examples/config-version/model.semantic-ir.json")).unwrap();
    let document_value: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let identity = document_value["package"]["identity"].as_str().unwrap();
    let packages = package_input([document.as_slice()]).unwrap();
    let text = format!(r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "{identity}";
dimension Time;
unit second : Time = rational(1, 1);
unit ms : Time = rational(1, 1000) * second;
time Selected dense unit ms;
temporal SelectedMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}}
"#);
    let source = SourceIdentity::new("agent-ix", "monitor-preparation");
    let name = LibraryName::new("agent-ix/monitor-preparation").unwrap();
    let target = MonitorTarget {
        counter: "counter".to_owned(), unit: "ms".to_owned(), width_bits: 16,
        uncertainty: Rational::from_integer(Integer::zero()),
        max_event_rate: Some(Rational::from_integer(Integer::from(2_u64))),
        max_reading_gap: 100, max_activations: 8,
    };
    let root = emit_plan_root(&PlanRootRequest {
        source: &source, path: "monitor-preparation.native", bytes: text.as_bytes(),
        packages: &packages, package_name: &name, limits: SpineLimits::default(),
        clause: "SelectedMonitor", target: &target, settings: &TimingObligationSettings::default(),
    }, &Cancel::new()).expect("actual final PlanRoot producer");
    let mut evidence = root.emission.evidence().clone();
    evidence.insert_domain_package_document(identity, document).unwrap();
    let package = match read_checked_package(root.emission.package().bytes(), CheckedPackageReadLimits {
        bytes: 8 * 1024 * 1024, nodes: 100_000, edges: 1_000_000,
        occurrences: 1_000_000, diagnostics: 1_000, work: 100_000_000,
    }, &evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => panic!("strict final package admission: {other:?}"),
    };
    // Capacity values are selected from the producer-owned plan rather than
    // recomputed by the driver. Retain its independently produced values and
    // use the final emission's exact subformula identities.
    let plan = package.monitor_plans().next().unwrap();
    let buffers = plan.buffers().map(|buffer| (buffer.subformula().clone(), buffer.capacity().into())).collect();
    Fixture { package, emission: root.emission, buffers }
}

fn with_request(test: impl FnOnce(&mut MonitorPreparationRequest<'_>)) {
    let fixture = &*FIXTURE;
    let package = fixture.package.as_ref();
    let plan = package.monitor_plans().next().unwrap();
    let source = package.source_map().iter().find(|row|
        row.node_id == *plan.clause() && row.role == CheckedOccurrenceRole::Claim).unwrap();
    let CheckedTerm::Application { arguments, .. } = package.term(plan.clause()).unwrap() else { panic!("clause application"); };
    let CheckedTerm::Reference { target: subject } = &arguments[0] else { panic!("original over parameter"); };
    let profile = package.lock().profile_selections.iter().find(|profile|
        profile.role == CheckedSelectionRole::TemporalProfile && profile.definition.identity.ends_with("temporal.timed/v1")).unwrap();
    let clock = package.temporal_bindings().unwrap().iter().find(|row|
        row.declaration == *plan.clause() && row.occurrence == source.ordinal).unwrap();
    let tick = fixture.emission.selected_source_unit_rows().iter().find(|row|
        row.qualified_declaration.last().is_some_and(|name| name.as_ref() == "ms")).unwrap();
    let equality = fixture.emission.equality_witness(package);
    let rate = CheckedRational { numerator: "2".into(), denominator: "1".into() };
    let uncertainty = CheckedRational { numerator: "0".into(), denominator: "1".into() };
    let mut request = MonitorPreparationRequest {
        package, selection: MonitorSelection::new(package, plan.node_id()),
        package_lock: package.lock(), root_key: plan.structural_key(),
        entry_keys: [plan.init().structural_key(), plan.on_event().structural_key(),
            plan.on_watermark().structural_key(), plan.on_gap().structural_key()],
        clause: plan.clause(), subject, occurrence: source, profile, clock_binding: clock,
        counter: "counter", tick_unit: &tick.structural_node, counter_width_bits: 16,
        max_reading_gap: 100, max_event_rate: &rate, uncertainty: &uncertainty,
        buffers: &fixture.buffers, activation_capacity: "8",
        witnesses: SelectedFactRows {
            declarations: fixture.emission.selected_exact_declaration_rows(),
            fields: fixture.emission.selected_exact_field_rows(),
            source_units: fixture.emission.selected_source_unit_rows(),
            equality_applications: equality.applications(), equality_facts: equality.facts(),
        },
        limits: MonitorPreparationLimits { work: 100_000_000, storage: 8 * 1024 * 1024 },
    };
    test(&mut request);
}

#[test]
fn selection_binds_original_package_keys_subject_occurrence_and_target() {
    with_request(|request| {
        let admitted = admit_monitor_selection(request, &Cancel::new()).expect("genuine matching independent selection");
        assert_eq!(admitted.plan().node_id(), request.selection_root());
        assert_eq!(admitted.plan().structural_key(), request.root_key);
        assert!(admitted.work() > 0);
    });
}

#[test]
fn equal_value_foreign_instance_and_independent_identity_edits_refuse() {
    with_request(|request| {
        let cloned = request.package.clone();
        request.package = &cloned;
        let error = admit_monitor_selection(request, &Cancel::new()).err().expect("foreign instance stops");
        assert!(matches!(error, MonitorPreparationError::Selection { field: MonitorSelectionField::Package, .. }));
    });
    with_request(|request| {
        request.subject = request.package.monitor_plans().next().unwrap().state();
        let error = admit_monitor_selection(request, &Cancel::new()).err().expect("foreign subject stops");
        assert!(matches!(error, MonitorPreparationError::Selection { field: MonitorSelectionField::Subject, .. }));
    });
    with_request(|request| {
        let mut occurrence = request.occurrence.clone();
        occurrence.regions[0].start += 1;
        request.occurrence = &occurrence;
        let error = admit_monitor_selection(request, &Cancel::new()).err().expect("changed full occurrence stops");
        assert!(matches!(error, MonitorPreparationError::Selection { field: MonitorSelectionField::Occurrence, .. }));
    });
}

#[test]
fn target_mismatch_keeps_the_helpers_value_endpoint_unchanged() {
    with_request(|request| {
        let uncertainty = CheckedRational { numerator: "1".into(), denominator: "2".into() };
        request.uncertainty = &uncertainty;
        let error = admit_monitor_selection(request, &Cancel::new()).err().expect("independent changed uncertainty stops");
        let MonitorPreparationError::TargetMismatch(mismatch) = error else { panic!("exact target mismatch"); };
        assert_eq!(mismatch.premise, TargetPremise::Uncertainty);
        assert_eq!(mismatch.locus.node, request.selection_root().clone());
        assert_eq!(mismatch.locus.body_path, [14, 0]);
        assert_eq!(mismatch.locus.structural_key, *request.root_key);
        assert_eq!(mismatch.locus.occurrence, None);
    });
}

#[test]
fn operating_work_and_original_cancellation_are_not_selection_refusals() {
    with_request(|request| {
        request.limits.work = 0;
        assert!(matches!(admit_monitor_selection(request, &Cancel::new()),
            Err(MonitorPreparationError::Work { limit: 0, consumed: 0, next: 1 })));
        let cancel = Cancel::new(); cancel.cancel(CancelCause::Deadline);
        assert!(matches!(admit_monitor_selection(request, &cancel),
            Err(MonitorPreparationError::Cancelled(CancelCause::Deadline))));
    });
}

#[test]
fn initializer_projection_preserves_the_actual_native_source_type_stop() {
    with_request(|request| {
        let admitted = admit_monitor_selection(request, &Cancel::new()).unwrap();
        let error = project_monitor_initializer(&admitted, &Cancel::new()).expect_err("current source record projection is genuinely unavailable");
        assert_eq!(error.code(), MonitorPreparationErrorCode::Projection);
        let MonitorPreparationError::Projection(native) = error else { unreachable!() };
        let init = admitted.plan().init();
        let CheckedTerm::Aggregate { members } = init.body() else { panic!("admitted entry aggregate"); };
        let CheckedTerm::Binding { value, .. } = &members[1] else { panic!("admitted entry body"); };
        let CheckedTerm::Reference { target } = value.as_ref() else { panic!("original initializer expression"); };
        let identities = SelectedExactIdentities::from_stage_rows(request.package,
            request.witnesses.declarations.iter().cloned(), request.witnesses.fields.iter().cloned()).unwrap();
        let original = request.package.project_exact_type(target, &identities, 1_000_000).expect_err("native unavailable source type");
        assert_eq!(native, original);
        assert_eq!(*native.requested, *target);
    });
}
