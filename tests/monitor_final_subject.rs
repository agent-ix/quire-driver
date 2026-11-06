// SPDX-License-Identifier: AGPL-3.0-or-later
//! Authentic final-subject mapping and native refusal controls.

use std::sync::LazyLock;

use ix_trace_rs::trace;
use qsl_semantics::library::LibraryName;
use quire_contract_model::{
    read_checked_package, CheckedOccurrenceRole, CheckedPackageDispatchResult, CheckedTerm,
};
use quire_driver::monitor::{
    admit_monitor_final_subject, MonitorPreparationError, MonitorPreparationErrorCode,
    MonitorPreparationRequest, MonitorSelectionField,
};
use quire_exact::{Cancel, CancelCause};

/// Native PlanRoot support shared by monitor integration suites.
#[path = "common/monitor.rs"]
pub mod monitor_fixture;
use monitor_fixture::{fixture, read_limits, Fixture, DISTINCT_CLAUSE};

static FIRST: LazyLock<Fixture> =
    LazyLock::new(|| fixture("SelectedMonitor", "counter", DISTINCT_CLAUSE));
static SECOND: LazyLock<Fixture> =
    LazyLock::new(|| fixture("OtherMonitor", "counter", DISTINCT_CLAUSE));
static FOREIGN: LazyLock<Fixture> =
    LazyLock::new(|| fixture("SelectedMonitor", "other-counter", DISTINCT_CLAUSE));

fn with_request(fixture: &Fixture, test: impl FnOnce(MonitorPreparationRequest<'_>)) {
    let ordinal = fixture.subject.occurrence().origin().ordinal();
    monitor_fixture::with_request(fixture, ordinal, test);
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
fn original_subjects_select_their_own_final_claims_and_native_model_time() {
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
            let CheckedTerm::Application { arguments, .. } = request.package.term(owner).unwrap()
            else {
                panic!("admitted Timed clause")
            };
            let clock_name = match arguments.get(1).unwrap() {
                CheckedTerm::Binding { value, .. } => value.as_ref(),
                direct => direct,
            };
            let CheckedTerm::Literal { value, .. } = clock_name else {
                panic!("admitted model-time name")
            };
            assert_eq!(value.as_str(), Some("model-time"));
            assert!(token.selection().work() > 0);
            assert_eq!(
                fixture.emission.temporal_binding_rows().len(),
                0,
                "Timed clock row count is not declaration mapping count"
            );
            let clock = request
                .package
                .model_time_for_clause(owner, u64::MAX)
                .unwrap()
                .expect("native model-time source");
            assert_eq!(clock.clause(), owner);
            assert_eq!(clock.unit(), request.tick_unit);
            let admitted = token.admit_clock(&Cancel::new()).unwrap();
            assert_eq!(admitted.clock().clock_name(), "model-time");
            assert_eq!(admitted.clock().claim_name(), fixture.subject.clause_name());
            assert!(admitted.selection().work() > token.selection().work());
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

#[trace("FR-015-AC-15")]
#[test]
fn producer_material_is_charged_before_early_identity_mismatches() {
    use qsl_foundation::source::provenance::OccurrenceKey;
    use qsl_replay::MonitorSubject;
    use quire_exact::{NodeKey, Origin, PathSegment, Role};
    for case in 0..4 {
        with_request(&FIRST, |mut request| {
            let original = &FIRST.subject;
            let package = if case == 0 {
                LibraryName::new("agent-ix/foreign").unwrap()
            } else {
                original.package().clone()
            };
            let node = if case == 1 {
                SECOND.subject.clause_node()
            } else {
                original.clause_node()
            };
            let role = if case == 2 { "declaration" } else { "claim" };
            let ordinal = if case == 3 {
                u64::MAX
            } else {
                original.occurrence().origin().ordinal()
            };
            let subject = MonitorSubject::new(
                package,
                original.clause_node().clone(),
                OccurrenceKey::new(node.clone(), Origin::new(Role::new(role), ordinal)),
                original.clause_name(),
            );
            request.limits.work = 0;
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
            assert!(matches!(
                error,
                MonitorPreparationError::Work {
                    limit: 0,
                    consumed: 0,
                    next: 1
                }
            ));
        });
    }
    with_request(&FIRST, |mut request| {
        let original = &FIRST.subject;
        let subject = MonitorSubject::new(
            LibraryName::new("agent-ix/foreign").unwrap(),
            original.clause_node().clone(),
            original.occurrence().clone(),
            original.clause_name(),
        );
        let names = 2
            + u64::try_from(subject.package().as_str().len()).unwrap()
            + u64::try_from(FIRST.emission.package().name().len()).unwrap();
        request.limits.work = names - 1;
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
        assert_eq!(error.code(), MonitorPreparationErrorCode::Work);
        request.limits.work = names;
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
        assert_field(error, MonitorSelectionField::ProducerSubject);
    });
    with_request(&FIRST, |mut request| {
        let original = &FIRST.subject;
        let node =
            NodeKey::try_new("p".to_owned(), vec![PathSegment::Text("x".to_owned())], 0).unwrap();
        let subject = MonitorSubject::new(
            original.package().clone(),
            original.clause_node().clone(),
            OccurrenceKey::new(node, original.occurrence().origin().clone()),
            original.clause_name(),
        );
        let names = 2 + 2 * u64::try_from(original.package().as_str().len()).unwrap();
        // This simple native key costs 15 material units. The second key must
        // be charged before comparing the unequal keys, so its push is denied.
        request.limits.work = names + 15;
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
        assert!(
            matches!(error, MonitorPreparationError::Work { consumed, next: 1, .. } if consumed == names + 15)
        );
    });
}
