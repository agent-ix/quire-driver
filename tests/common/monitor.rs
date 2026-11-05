// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared authentic timed PlanRoot production, admission and request support.

use std::{path::PathBuf, process::Command};

use qsl_analyze::monitor_root::{emit_plan_root, PlanRootRequest};
use qsl_foundation::SourceIdentity;
use qsl_package::Emission;
use qsl_replay::{spine::SpineLimits, MonitorTarget, TimingObligationSettings};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedOccurrenceRole, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, CheckedPackageV2, CheckedRational, CheckedSelectionRole, CheckedTerm,
    SelectedFactRows,
};
use quire_driver::monitor::{
    MonitorPreparationLimits, MonitorPreparationRequest, MonitorSelection,
};
use quire_exact::{Cancel, Integer, Rational};

/// A second authored claim with the legacy identical clause body.
pub const SAME_CLAUSE_TWIN: &str = r#"temporal TwinMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}"#;

/// A distinct second clause for original-subject mapping controls.
pub const DISTINCT_CLAUSE: &str = r#"temporal OtherMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {
    eventually[0 ms, 3 ms] holds(candidate.versionNumber >= 1)
}"#;

/// Native producer output, strict admission and original witness holders.
pub struct Fixture {
    /// Strictly admitted native package.
    pub package: Box<CheckedPackageV2>,
    /// Original pre-writer native subject.
    pub subject: qsl_replay::MonitorSubject,
    /// Native evidence extended with the selected domain document.
    pub evidence: quire_contract_model::CheckedPackageEvidence,
    /// Independently supplied hardware counter expectation.
    pub counter: &'static str,
    /// Original final native emission.
    pub emission: Emission,
    /// Exact producer capacities joined to final admitted subformula IDs.
    pub buffers: Vec<(CheckedNodeId, Box<str>)>,
}

/// Produce and strictly admit a selected clause with the requested companion source.
pub fn fixture(selected: &str, counter: &'static str, additional_clause: &str) -> Fixture {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--locked", "--format-version", "1", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("resolved dependency metadata");
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
        .find(|package| package["name"] == "qsl-replay")
        .unwrap()["manifest_path"]
        .as_str()
        .unwrap();
    let path = PathBuf::from(manifest);
    let document = std::fs::read(
        path.parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/config-version/model.semantic-ir.json"),
    )
    .unwrap();
    let document_value: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let identity = document_value["package"]["identity"].as_str().unwrap();
    let packages = package_input([document.as_slice()]).unwrap();
    let text = format!(
        r#"language "ix:native" edition "1-draft";
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
{additional_clause}
"#
    );
    let source = SourceIdentity::new("agent-ix", "monitor-preparation");
    let name = LibraryName::new("agent-ix/monitor-preparation").unwrap();
    let target = MonitorTarget {
        counter: counter.to_owned(),
        unit: "ms".to_owned(),
        width_bits: 16,
        uncertainty: Rational::from_integer(Integer::zero()),
        max_event_rate: Some(Rational::from_integer(Integer::from(2_u64))),
        max_reading_gap: 100,
        max_activations: 8,
    };
    let root = emit_plan_root(
        &PlanRootRequest {
            source: &source,
            path: "monitor-preparation.native",
            bytes: text.as_bytes(),
            packages: &packages,
            package_name: &name,
            limits: SpineLimits::default(),
            clause: selected,
            target: &target,
            settings: &TimingObligationSettings::default(),
        },
        &Cancel::new(),
    )
    .expect("actual final PlanRoot producer");
    let mut evidence = root.emission.evidence().clone();
    evidence
        .insert_domain_package_document(identity, document)
        .unwrap();
    let package =
        match read_checked_package(root.emission.package().bytes(), read_limits(), &evidence) {
            CheckedPackageDispatchResult::AdmittedV2(package) => package,
            other => panic!("strict final package admission: {other:?}"),
        };
    // Capacity values are selected from the producer-owned plan rather than
    // recomputed by the driver. Retain its independently produced values and
    // use the final emission's exact subformula identities.
    let plan = package.monitor_plans().next().unwrap();
    assert_eq!(plan.buffers().len(), root.plan.buffers().len());
    let buffers = plan
        .buffers()
        .zip(root.plan.buffers())
        .map(|(buffer, (_, capacity))| {
            (
                buffer.subformula().clone(),
                capacity.to_string().into_boxed_str(),
            )
        })
        .collect();
    Fixture {
        package,
        subject: root.plan.subject().clone(),
        evidence,
        counter,
        emission: root.emission,
        buffers,
    }
}

/// Construct independent inputs using the caller-selected original Claim ordinal.
pub fn with_request(
    fixture: &Fixture,
    ordinal: u64,
    test: impl FnOnce(MonitorPreparationRequest<'_>),
) {
    let package = fixture.package.as_ref();
    let plan = package.monitor_plans().next().unwrap();
    assert_eq!(package.monitor_plans().len(), 1);
    let source = package
        .source_map()
        .iter()
        .find(|row| {
            row.node_id == *plan.clause()
                && row.role == CheckedOccurrenceRole::Claim
                && row.ordinal == ordinal
        })
        .unwrap();
    let CheckedTerm::Application { arguments, .. } = package.term(plan.clause()).unwrap() else {
        panic!("clause application");
    };
    let CheckedTerm::Reference { target: subject } = &arguments[0] else {
        panic!("original over parameter");
    };
    let profile = package
        .lock()
        .profile_selections
        .iter()
        .find(|profile| {
            profile.role == CheckedSelectionRole::TemporalProfile
                && profile.definition.identity.ends_with("temporal.timed/v1")
        })
        .unwrap();
    let clock = package
        .temporal_bindings()
        .unwrap_or_default()
        .iter()
        .find(|row| row.declaration == *plan.clause() && row.occurrence == source.ordinal);
    let tick = fixture
        .emission
        .selected_source_unit_rows()
        .iter()
        .find(|row| {
            row.qualified_declaration
                .last()
                .is_some_and(|name| name.as_ref() == "ms")
        })
        .unwrap();
    let equality = fixture.emission.equality_witness(package);
    let rate = CheckedRational {
        numerator: "2".into(),
        denominator: "1".into(),
    };
    let uncertainty = CheckedRational {
        numerator: "0".into(),
        denominator: "1".into(),
    };
    let request = MonitorPreparationRequest {
        package,
        selection: MonitorSelection::new(package, plan.node_id()),
        package_lock: package.lock(),
        root_key: plan.structural_key(),
        entry_keys: [
            plan.init().structural_key(),
            plan.on_event().structural_key(),
            plan.on_watermark().structural_key(),
            plan.on_gap().structural_key(),
        ],
        clause: plan.clause(),
        subject,
        occurrence: source,
        profile,
        clock_binding: clock,
        counter: fixture.counter,
        tick_unit: &tick.structural_node,
        counter_width_bits: 16,
        max_reading_gap: 100,
        max_event_rate: &rate,
        uncertainty: &uncertainty,
        buffers: &fixture.buffers,
        activation_capacity: "8",
        witnesses: SelectedFactRows {
            declarations: fixture.emission.selected_exact_declaration_rows(),
            fields: fixture.emission.selected_exact_field_rows(),
            source_units: fixture.emission.selected_source_unit_rows(),
            equality_applications: equality.applications(),
            equality_facts: equality.facts(),
        },
        limits: MonitorPreparationLimits {
            work: 100_000_000,
            storage: 8 * 1024 * 1024,
        },
    };
    test(request);
}

/// Explicit native read allowances shared by fixture construction and controls.
pub fn read_limits() -> CheckedPackageReadLimits {
    CheckedPackageReadLimits {
        bytes: 8 * 1024 * 1024,
        nodes: 100_000,
        edges: 1_000_000,
        occurrences: 1_000_000,
        diagnostics: 1_000,
        work: 100_000_000,
    }
}
