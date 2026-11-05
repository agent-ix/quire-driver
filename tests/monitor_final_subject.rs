// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real producer input to the driver's independent selection boundary.

use std::{path::PathBuf, process::Command, sync::LazyLock};

use ix_trace_rs::trace;
use qsl_analyze::monitor_root::{emit_plan_root, PlanRootRequest};
use qsl_foundation::SourceIdentity;
use qsl_package::Emission;
use qsl_replay::{spine::SpineLimits, MonitorTarget, TimingObligationSettings};
use qsl_semantics::{library::LibraryName, model::intake::package_input};
use quire_contract_model::{
    read_checked_package, CheckedMonitorClockReadErrorCode, CheckedNodeId, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedPackageDispatchResult, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedRational, CheckedSelectionRole, CheckedTerm, SelectedFactRows,
};
use quire_driver::monitor::{
    admit_monitor_final_subject, MonitorPreparationError, MonitorPreparationErrorCode,
    MonitorPreparationLimits, MonitorPreparationRequest, MonitorSelection, MonitorSelectionField,
};
use quire_exact::{Cancel, CancelCause, Integer, Rational};

struct Fixture {
    package: Box<CheckedPackageV2>,
    subject: qsl_replay::MonitorSubject,
    evidence: quire_contract_model::CheckedPackageEvidence,
    counter: &'static str,
    emission: Emission,
    buffers: Vec<(CheckedNodeId, Box<str>)>,
}

static FIRST: LazyLock<Fixture> = LazyLock::new(|| fixture("SelectedMonitor", "counter"));
static SECOND: LazyLock<Fixture> = LazyLock::new(|| fixture("OtherMonitor", "counter"));
static FOREIGN: LazyLock<Fixture> = LazyLock::new(|| fixture("SelectedMonitor", "other-counter"));

fn fixture(selected: &str, counter: &'static str) -> Fixture {
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
temporal OtherMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    eventually[0 ms, 3 ms] holds(candidate.versionNumber >= 1)
}}
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
    let package = match read_checked_package(
        root.emission.package().bytes(),
        CheckedPackageReadLimits {
            bytes: 8 * 1024 * 1024,
            nodes: 100_000,
            edges: 1_000_000,
            occurrences: 1_000_000,
            diagnostics: 1_000,
            work: 100_000_000,
        },
        &evidence,
    ) {
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

fn with_request(fixture: &Fixture, test: impl FnOnce(MonitorPreparationRequest<'_>)) {
    let package = fixture.package.as_ref();
    let plan = package.monitor_plans().next().unwrap();
    // The sole root is independently selected. The ordinal comes from the
    // original native subject, not a row position or name search.
    assert_eq!(package.monitor_plans().len(), 1);
    let ordinal = fixture.subject.occurrence().origin().ordinal();
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

fn read_limits() -> CheckedPackageReadLimits {
    CheckedPackageReadLimits {
        bytes: 8 * 1024 * 1024,
        nodes: 100_000,
        edges: 1_000_000,
        occurrences: 1_000_000,
        diagnostics: 1_000,
        work: 100_000_000,
    }
}

fn assert_field(error: MonitorPreparationError, expected: MonitorSelectionField) {
    assert_eq!(error.code(), MonitorPreparationErrorCode::Selection);
    let MonitorPreparationError::Selection { field, .. } = error else {
        panic!("typed independent mismatch")
    };
    assert_eq!(field, expected);
}

#[trace("FR-015-AC-15")]
#[test]
fn original_subjects_select_their_own_final_claims_and_reach_native_missing_clock() {
    let mut declarations = Vec::new();
    for fixture in [&*FIRST, &*SECOND] {
        with_request(fixture, |request| {
            let token = admit_monitor_final_subject(
                &request,
                &fixture.subject,
                &fixture.emission,
                &fixture.evidence,
                read_limits(),
                &Cancel::new(),
            )
            .unwrap();
            let original = fixture.subject.occurrence();
            let owner = fixture
                .emission
                .temporal_binding_rows()
                .declaration(fixture.subject.clause_node(), original.origin().ordinal())
                .unwrap();
            assert_eq!(token.declaration(), owner);
            assert_eq!(token.declaration(), request.clause);
            assert_eq!(token.selection().plan().clause(), owner);
            assert!(std::ptr::eq(token.subject(), &fixture.subject));
            assert!(std::ptr::eq(token.emission(), &fixture.emission));
            assert_eq!(original.node(), fixture.subject.clause_node());
            assert_eq!(request.occurrence.role, CheckedOccurrenceRole::Claim);
            assert_eq!(request.occurrence.ordinal, original.origin().ordinal());
            let name = request
                .package
                .temporal_clause_names()
                .unwrap()
                .iter()
                .find(|row| {
                    row.declaration == *owner && row.occurrence == original.origin().ordinal()
                })
                .unwrap();
            assert_eq!(name.name.as_ref(), fixture.subject.clause_name());
            assert!(token.selection().work() > 0);
            assert_eq!(
                fixture.emission.temporal_binding_rows().len(),
                0,
                "Timed clock row count is not declaration mapping count"
            );
            let error = token.admit_clock(&Cancel::new()).err().unwrap();
            let MonitorPreparationError::ClockRead(actual) = error else {
                panic!("native clock handoff")
            };
            let expected = token
                .selection()
                .plan()
                .clock_for_claim(
                    owner,
                    &CheckedOccurrence {
                        role: CheckedOccurrenceRole::Claim,
                        ordinal: original.origin().ordinal(),
                    },
                    u64::MAX,
                )
                .unwrap_err();
            assert_eq!(*actual, expected);
            assert_eq!(actual.code, CheckedMonitorClockReadErrorCode::MissingClock);
            declarations.push(owner.clone());
        });
    }
    assert_ne!(declarations[0], declarations[1]);
}

#[trace("FR-015-AC-15")]
#[test]
fn another_subject_mapping_in_the_same_emission_cannot_replace_the_selected_clause() {
    with_request(&FIRST, |request| {
        let other = &SECOND.subject;
        assert!(FIRST
            .emission
            .temporal_binding_rows()
            .declaration(other.clause_node(), other.occurrence().origin().ordinal())
            .is_some());
        let error = admit_monitor_final_subject(
            &request,
            other,
            &FIRST.emission,
            &FIRST.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .err()
        .unwrap();
        assert_field(error, MonitorSelectionField::FinalDeclaration);
    });
}

#[trace("FR-015-AC-15")]
#[test]
fn missing_lookup_and_subject_node_role_name_package_edits_are_typed() {
    use qsl_foundation::source::provenance::OccurrenceKey;
    use qsl_replay::MonitorSubject;
    use quire_exact::{Origin, Role};
    for case in 0..5 {
        with_request(&FIRST, |request| {
            let original = &FIRST.subject;
            let ordinal = if case == 0 {
                u64::MAX
            } else {
                original.occurrence().origin().ordinal()
            };
            let role = if case == 1 { "declaration" } else { "claim" };
            let occurrence_node = if case == 2 {
                SECOND.subject.clause_node()
            } else {
                original.clause_node()
            };
            let package = if case == 4 {
                LibraryName::new("agent-ix/foreign").unwrap()
            } else {
                original.package().clone()
            };
            let name = if case == 3 {
                "OtherMonitor"
            } else {
                original.clause_name()
            };
            let subject = MonitorSubject::new(
                package,
                original.clause_node().clone(),
                OccurrenceKey::new(
                    occurrence_node.clone(),
                    Origin::new(Role::new(role), ordinal),
                ),
                name,
            );
            let error = admit_monitor_final_subject(
                &request,
                &subject,
                &FIRST.emission,
                &FIRST.evidence,
                read_limits(),
                &Cancel::new(),
            )
            .err()
            .unwrap();
            if case == 0 {
                assert_eq!(error.code(), MonitorPreparationErrorCode::Selection);
                let MonitorPreparationError::MissingFinalSubject(retained) = error else {
                    panic!("original missing mapping")
                };
                assert_eq!(*retained, subject);
            } else {
                assert_field(
                    error,
                    if case == 3 {
                        MonitorSelectionField::ClaimName
                    } else {
                        MonitorSelectionField::ProducerSubject
                    },
                );
            }
        });
    }
}

#[trace("FR-015-AC-15")]
#[test]
fn complete_native_package_authentication_rejects_foreign_emission_and_keeps_instance_boundary() {
    with_request(&FIRST, |request| {
        assert_eq!(
            FIRST.emission.package().name(),
            FOREIGN.emission.package().name()
        );
        assert_ne!(FIRST.package, FOREIGN.package);
        let error = admit_monitor_final_subject(
            &request,
            &FIRST.subject,
            &FOREIGN.emission,
            &FOREIGN.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .err()
        .unwrap();
        assert_field(error, MonitorSelectionField::EmissionPackage);
    });
    with_request(&FOREIGN, |request| {
        let error = admit_monitor_final_subject(
            &request,
            &FIRST.subject,
            &FIRST.emission,
            &FIRST.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .err()
        .unwrap();
        assert_field(error, MonitorSelectionField::EmissionPackage);
    });
    with_request(&FIRST, |request| {
        let cloned = request.package.clone();
        let request = MonitorPreparationRequest {
            package: &cloned,
            ..request
        };
        let error = admit_monitor_final_subject(
            &request,
            &FIRST.subject,
            &FIRST.emission,
            &FIRST.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .err()
        .unwrap();
        assert_field(error, MonitorSelectionField::Package);
    });
}

#[trace("FR-015-AC-15")]
#[test]
fn independent_clause_full_occurrence_and_existing_subject_must_still_match() {
    for case in 0..6 {
        with_request(&FIRST, |request| {
            let mut row = request.occurrence.clone();
            let mut request = request;
            let expected = match case {
                0 => {
                    request.clause = SECOND.package.monitor_plans().next().unwrap().clause();
                    MonitorSelectionField::FinalDeclaration
                }
                1 => {
                    row.ordinal = u64::MAX;
                    MonitorSelectionField::FinalOccurrence
                }
                2 => {
                    row.role = CheckedOccurrenceRole::Declaration;
                    MonitorSelectionField::FinalOccurrence
                }
                3 => {
                    row.node_id = request.subject.clone();
                    MonitorSelectionField::FinalOccurrence
                }
                4 => {
                    row.regions[0].start += 1;
                    MonitorSelectionField::Occurrence
                }
                5 => {
                    request.subject = request.clause;
                    MonitorSelectionField::Subject
                }
                _ => unreachable!(),
            };
            request.occurrence = &row;
            let error = admit_monitor_final_subject(
                &request,
                &FIRST.subject,
                &FIRST.emission,
                &FIRST.evidence,
                read_limits(),
                &Cancel::new(),
            )
            .err()
            .unwrap();
            assert_field(error, expected);
        });
    }
}

#[trace("FR-015-AC-15")]
#[test]
fn native_acquisition_refusals_and_limits_keep_their_complete_original_payloads() {
    with_request(&FIRST, |request| {
        let evidence = quire_contract_model::CheckedPackageEvidence::new();
        let CheckedPackageDispatchResult::Refused(expected) =
            read_checked_package(FIRST.emission.package().bytes(), read_limits(), &evidence)
        else {
            panic!("missing native evidence")
        };
        let error = admit_monitor_final_subject(
            &request,
            &FIRST.subject,
            &FIRST.emission,
            &evidence,
            read_limits(),
            &Cancel::new(),
        )
        .err()
        .unwrap();
        assert_eq!(error.code(), MonitorPreparationErrorCode::Selection);
        let MonitorPreparationError::EmissionRefused(actual) = error else {
            panic!("native refused acquisition")
        };
        assert_eq!(*actual, expected);
    });
    for bytes in [true, false] {
        with_request(&FIRST, |request| {
            let mut limits = read_limits();
            if bytes {
                limits.bytes = 0;
            } else {
                limits.work = 0;
            }
            let CheckedPackageDispatchResult::Incomplete(expected) =
                read_checked_package(FIRST.emission.package().bytes(), limits, &FIRST.evidence)
            else {
                panic!("native acquisition limit")
            };
            let error = admit_monitor_final_subject(
                &request,
                &FIRST.subject,
                &FIRST.emission,
                &FIRST.evidence,
                limits,
                &Cancel::new(),
            )
            .err()
            .unwrap();
            assert_eq!(error.code(), MonitorPreparationErrorCode::Work);
            let MonitorPreparationError::EmissionIncomplete(actual) = error else {
                panic!("native incomplete acquisition")
            };
            assert_eq!(*actual, expected);
        });
    }
}

#[trace("FR-015-AC-15")]
#[test]
fn final_subject_cancellation_work_and_clock_continuation_preserve_the_original_prefix() {
    with_request(&FIRST, |mut request| {
        let source = request.occurrence.clone();
        let cancel = Cancel::new();
        cancel.cancel(CancelCause::Deadline);
        assert!(matches!(
            admit_monitor_final_subject(
                &request,
                &FIRST.subject,
                &FIRST.emission,
                &FIRST.evidence,
                read_limits(),
                &cancel
            ),
            Err(MonitorPreparationError::Cancelled(CancelCause::Deadline))
        ));
        assert_eq!(request.occurrence, &source);
        let required = admit_monitor_final_subject(
            &request,
            &FIRST.subject,
            &FIRST.emission,
            &FIRST.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .unwrap()
        .selection()
        .work();
        request.limits.work = required;
        let exact = admit_monitor_final_subject(
            &request,
            &FIRST.subject,
            &FIRST.emission,
            &FIRST.evidence,
            read_limits(),
            &Cancel::new(),
        )
        .unwrap();
        assert_eq!(exact.selection().work(), required);
        let error = exact.admit_clock(&Cancel::new()).err().unwrap();
        assert!(
            matches!(error, MonitorPreparationError::Work { consumed, next, .. } if consumed == required && next > 0)
        );
        for limit in [required - 1, 0] {
            request.limits.work = limit;
            let error = admit_monitor_final_subject(
                &request,
                &FIRST.subject,
                &FIRST.emission,
                &FIRST.evidence,
                read_limits(),
                &Cancel::new(),
            )
            .err()
            .unwrap();
            assert_eq!(error.code(), MonitorPreparationErrorCode::Work);
        }
        assert_eq!(request.occurrence, &source);
    });
}
