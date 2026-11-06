// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real producer input to the driver's independent selection boundary.

use std::{
    process::Command,
    sync::{Arc, LazyLock},
};

use ix_trace_rs::trace;
use quire_contract_model::{
    CheckedCallableReadErrorCode, CheckedMonitorClock, CheckedMonitorClockReadError,
    CheckedMonitorClockReadErrorCode, CheckedNodeId, CheckedNodePathSegment, CheckedOccurrence,
    CheckedOccurrenceRole, CheckedRational, CheckedTerm, ClockConfiguration,
    ExactTypeProjectionErrorCode, NativeJoinBudget, SelectedExactIdentities, TemporalBinding,
};
use quire_driver::monitor::{
    admit_monitor_clock_selection, admit_monitor_selection, project_monitor_initializer,
    MonitorPreparationError, MonitorPreparationErrorCode, MonitorPreparationRequest,
    MonitorSelection, MonitorSelectionField,
};
use quire_driver::monitor_refusal::TargetPremise;
use quire_exact::{Cancel, CancelCause};

/// Native PlanRoot support shared by monitor integration suites.
#[path = "common/monitor.rs"]
pub mod monitor_fixture;
use monitor_fixture::{Fixture, SAME_CLAUSE_TWIN};

static FIXTURE: LazyLock<Fixture> =
    LazyLock::new(|| monitor_fixture::fixture("SelectedMonitor", "counter", SAME_CLAUSE_TWIN));

fn with_request(test: impl FnOnce(MonitorPreparationRequest<'_>)) {
    let fixture = &*FIXTURE;
    let plan = fixture.package.monitor_plans().next().unwrap();
    // Legacy partial selection chooses its authored fixture claim by name.
    // Final-subject qualification uses the original native subject instead.
    let ordinal = fixture
        .package
        .temporal_clause_names()
        .unwrap()
        .iter()
        .find(|row| row.declaration == *plan.clause() && row.name.as_ref() == "SelectedMonitor")
        .unwrap()
        .occurrence;
    monitor_fixture::with_request(fixture, ordinal, test);
}

#[test]
fn selection_binds_original_package_keys_subject_occurrence_and_target() {
    with_request(|request| {
        let admitted = admit_monitor_selection(&request, &Cancel::new())
            .expect("genuine matching independent selection");
        assert_eq!(admitted.plan().node_id(), request.selection_root());
        assert_eq!(admitted.plan().structural_key(), request.root_key);
        assert!(std::ptr::eq(admitted.request(), &request));
        assert!(std::ptr::eq(admitted.package(), request.package));
        assert!(admitted.work() > 0);
    });
}

#[test]
fn equal_value_foreign_instance_and_independent_identity_edits_refuse() {
    with_request(|request| {
        let cloned = request.package.clone();
        let request = MonitorPreparationRequest {
            package: &cloned,
            ..request
        };
        let error = admit_monitor_selection(&request, &Cancel::new())
            .err()
            .expect("foreign instance stops");
        assert!(matches!(
            error,
            MonitorPreparationError::Selection {
                field: MonitorSelectionField::Package,
                ..
            }
        ));
    });
    with_request(|mut request| {
        request.subject = request.package.monitor_plans().next().unwrap().state();
        let error = admit_monitor_selection(&request, &Cancel::new())
            .err()
            .expect("foreign subject stops");
        assert!(matches!(
            error,
            MonitorPreparationError::Selection {
                field: MonitorSelectionField::Subject,
                ..
            }
        ));
    });
    with_request(|request| {
        let mut occurrence = request.occurrence.clone();
        occurrence.regions[0].start += 1;
        let request = MonitorPreparationRequest {
            occurrence: &occurrence,
            ..request
        };
        let error = admit_monitor_selection(&request, &Cancel::new())
            .err()
            .expect("changed full occurrence stops");
        assert!(matches!(
            error,
            MonitorPreparationError::Selection {
                field: MonitorSelectionField::Occurrence,
                ..
            }
        ));
    });
}

#[test]
fn target_mismatch_keeps_the_helpers_value_endpoint_unchanged() {
    with_request(|request| {
        let uncertainty = CheckedRational {
            numerator: "1".into(),
            denominator: "2".into(),
        };
        let request = MonitorPreparationRequest {
            uncertainty: &uncertainty,
            ..request
        };
        let error = admit_monitor_selection(&request, &Cancel::new())
            .err()
            .expect("independent changed uncertainty stops");
        assert_eq!(error.code().as_str(), "target_mismatch");
        assert_eq!(
            MonitorPreparationErrorCode::from_code("target_mismatch"),
            Some(error.code())
        );
        let MonitorPreparationError::TargetMismatch(mismatch) = error else {
            panic!("exact target mismatch");
        };
        assert_eq!(mismatch.premise, TargetPremise::Uncertainty);
        assert_eq!(mismatch.locus.node, request.selection_root().clone());
        assert_eq!(mismatch.locus.body_path, [14, 0]);
        assert_eq!(mismatch.locus.structural_key, *request.root_key);
        assert_eq!(mismatch.locus.occurrence, None);
    });
}

#[test]
fn operating_work_and_original_cancellation_are_not_selection_refusals() {
    with_request(|mut request| {
        let required = admit_monitor_selection(&request, &Cancel::new())
            .unwrap()
            .work();
        request.limits.work = required;
        assert_eq!(
            admit_monitor_selection(&request, &Cancel::new())
                .unwrap()
                .work(),
            required
        );
        request.limits.work = required - 1;
        let error = admit_monitor_selection(&request, &Cancel::new())
            .err()
            .expect("one below required work stops");
        assert_eq!(error.code().as_str(), "work");
        assert!(MonitorPreparationErrorCode::all().contains(&error.code()));
        let MonitorPreparationError::Work {
            limit,
            consumed,
            next,
        } = error
        else {
            panic!("typed operating work stop");
        };
        assert_eq!(limit, required - 1);
        assert!(consumed <= limit && next > limit - consumed);
        request.limits.work = 0;
        assert!(matches!(
            admit_monitor_selection(&request, &Cancel::new()),
            Err(MonitorPreparationError::Work {
                limit: 0,
                consumed: 0,
                next: 1
            })
        ));
        let cancel = Cancel::new();
        cancel.cancel(CancelCause::Deadline);
        assert!(matches!(
            admit_monitor_selection(&request, &cancel),
            Err(MonitorPreparationError::Cancelled(CancelCause::Deadline))
        ));
    });
}

#[test]
fn every_independent_target_premise_preserves_its_native_value_locus() {
    for (premise, expected_path) in [
        (TargetPremise::Counter, vec![12, 0]),
        (TargetPremise::CounterWidth, vec![8, 0]),
        (TargetPremise::TickUnit, vec![13, 0]),
        (TargetPremise::MaxReadingGap, vec![9, 0]),
        (TargetPremise::MaxEventRate, vec![10, 0]),
        (TargetPremise::Uncertainty, vec![14, 0]),
        (TargetPremise::BufferCapacity, vec![11, 0, 0, 1, 0]),
        (TargetPremise::ActivationCapacity, vec![16, 0]),
    ] {
        with_request(|request| {
            let changed_rate = CheckedRational {
                numerator: "3".into(),
                denominator: "1".into(),
            };
            let changed_uncertainty = CheckedRational {
                numerator: "1".into(),
                denominator: "2".into(),
            };
            let mut changed_buffers = request.buffers.to_vec();
            assert!(!changed_buffers.is_empty());
            changed_buffers[0].1 = "999".into();
            let edited = match premise {
                TargetPremise::Counter => MonitorPreparationRequest {
                    counter: "other-counter",
                    ..request
                },
                TargetPremise::CounterWidth => MonitorPreparationRequest {
                    counter_width_bits: 8,
                    ..request
                },
                TargetPremise::TickUnit => MonitorPreparationRequest {
                    tick_unit: request.clause,
                    ..request
                },
                TargetPremise::MaxReadingGap => MonitorPreparationRequest {
                    max_reading_gap: 101,
                    ..request
                },
                TargetPremise::MaxEventRate => MonitorPreparationRequest {
                    max_event_rate: &changed_rate,
                    ..request
                },
                TargetPremise::Uncertainty => MonitorPreparationRequest {
                    uncertainty: &changed_uncertainty,
                    ..request
                },
                TargetPremise::BufferCapacity => MonitorPreparationRequest {
                    buffers: &changed_buffers,
                    ..request
                },
                TargetPremise::ActivationCapacity => MonitorPreparationRequest {
                    activation_capacity: "0",
                    ..request
                },
            };
            let error = admit_monitor_selection(&edited, &Cancel::new())
                .err()
                .expect("independent changed premise stops");
            let MonitorPreparationError::TargetMismatch(mismatch) = error else {
                panic!("exact target mismatch for {premise:?}");
            };
            assert_eq!(mismatch.premise, premise);
            assert_eq!(mismatch.locus.node, *edited.selection_root());
            assert_eq!(mismatch.locus.structural_key, *edited.root_key);
            assert_eq!(mismatch.locus.body_path, expected_path);
            assert_eq!(mismatch.locus.occurrence, None);
        });
    }
}

#[test]
fn initializer_projection_preserves_the_actual_native_source_type_stop() {
    with_request(|request| {
        let admitted = admit_monitor_selection(&request, &Cancel::new()).unwrap();
        let init = admitted.plan().init();
        let CheckedTerm::Aggregate { members } = init.body() else {
            panic!("admitted entry aggregate");
        };
        let CheckedTerm::Binding { value, .. } = &members[1] else {
            panic!("admitted entry body");
        };
        let CheckedTerm::Reference { target } = value.as_ref() else {
            panic!("original initializer expression");
        };
        let mut budget = NativeJoinBudget::new(request.limits.work, request.limits.storage);
        let identities = SelectedExactIdentities::from_stage_rows(
            request.package,
            request.witnesses.declarations.iter().cloned(),
            request.witnesses.fields.iter().cloned(),
            &mut budget,
        )
        .unwrap();
        let error = project_monitor_initializer(&admitted, &identities, &Cancel::new())
            .expect_err("current source record projection is genuinely unavailable");
        assert_eq!(error.code(), MonitorPreparationErrorCode::Projection);
        assert_eq!(error.code().as_str(), "projection");
        let MonitorPreparationError::Projection(native) = error else {
            unreachable!()
        };
        let original = request
            .package
            .project_exact_type(target, &identities, 1_000_000)
            .expect_err("native unavailable source type");
        assert_eq!(native, original);
        assert_eq!(*native.requested, *target);
    });
}

#[derive(Clone, Copy)]
enum IdentityEdit {
    Root,
    Lock,
    RootKey,
    Entry(usize),
    Clause,
    Profile,
    BufferOwner,
    BufferCount,
}

#[test]
fn independent_identity_and_buffer_association_edits_refuse_at_the_exact_field() {
    for edit in [
        IdentityEdit::Root,
        IdentityEdit::Lock,
        IdentityEdit::RootKey,
        IdentityEdit::Entry(0),
        IdentityEdit::Entry(1),
        IdentityEdit::Entry(2),
        IdentityEdit::Entry(3),
        IdentityEdit::Clause,
        IdentityEdit::Profile,
        IdentityEdit::BufferOwner,
        IdentityEdit::BufferCount,
    ] {
        with_request(|request| {
            let plan = request.package.monitor_plans().next().unwrap();
            let original_key = plan.structural_key().clone();
            let mut absent = plan.node_id().clone();
            absent.ordinal += 1;
            let mut changed_lock = request.package_lock.clone();
            changed_lock
                .required_features
                .push("independently-unselected-feature".into());
            let mut changed_profile = request.profile.clone();
            changed_profile.definition.identity = "independently-unselected-profile".into();
            let mut changed_buffers = request.buffers.to_vec();
            assert!(!changed_buffers.is_empty());
            let original_buffer = changed_buffers[0].0.clone();
            let original_entries = [
                plan.init(),
                plan.on_event(),
                plan.on_watermark(),
                plan.on_gap(),
            ];
            let mut entry_keys = request.entry_keys;
            let (edited, expected_field, expected_node) = match edit {
                IdentityEdit::Root => (
                    MonitorPreparationRequest {
                        selection: MonitorSelection::new(request.package, &absent),
                        ..request
                    },
                    MonitorSelectionField::Root,
                    &absent,
                ),
                IdentityEdit::Lock => (
                    MonitorPreparationRequest {
                        package_lock: &changed_lock,
                        ..request
                    },
                    MonitorSelectionField::PackageLock,
                    plan.node_id(),
                ),
                IdentityEdit::RootKey => (
                    MonitorPreparationRequest {
                        root_key: &absent,
                        ..request
                    },
                    MonitorSelectionField::RootKey,
                    plan.node_id(),
                ),
                IdentityEdit::Entry(index) => {
                    entry_keys[index] = plan.structural_key();
                    (
                        MonitorPreparationRequest {
                            entry_keys,
                            ..request
                        },
                        MonitorSelectionField::EntryKey,
                        original_entries[index].node_id(),
                    )
                }
                IdentityEdit::Clause => (
                    MonitorPreparationRequest {
                        clause: plan.state(),
                        ..request
                    },
                    MonitorSelectionField::Clause,
                    plan.node_id(),
                ),
                IdentityEdit::Profile => (
                    MonitorPreparationRequest {
                        profile: &changed_profile,
                        ..request
                    },
                    MonitorSelectionField::Profile,
                    plan.clause(),
                ),
                IdentityEdit::BufferOwner => {
                    changed_buffers[0].0 = plan.state().clone();
                    (
                        MonitorPreparationRequest {
                            buffers: &changed_buffers,
                            ..request
                        },
                        MonitorSelectionField::BufferAssociation,
                        &original_buffer,
                    )
                }
                IdentityEdit::BufferCount => {
                    changed_buffers.clear();
                    (
                        MonitorPreparationRequest {
                            buffers: &changed_buffers,
                            ..request
                        },
                        MonitorSelectionField::BufferAssociation,
                        plan.node_id(),
                    )
                }
            };
            let error = admit_monitor_selection(&edited, &Cancel::new())
                .err()
                .expect("the changed independent identity must refuse");
            assert_eq!(error.code().as_str(), "selection");
            assert_eq!(
                MonitorPreparationErrorCode::from_code("selection"),
                Some(error.code())
            );
            assert!(MonitorPreparationErrorCode::all().contains(&error.code()));
            assert_eq!(MonitorPreparationErrorCode::from_code("unrecognized"), None);
            let MonitorPreparationError::Selection { field, node } = error else {
                panic!("typed selection refusal, not a target capacity error");
            };
            assert_eq!(field, expected_field);
            assert_eq!(node.as_ref(), expected_node);
            assert_eq!(
                edited
                    .package
                    .monitor_plans()
                    .next()
                    .unwrap()
                    .structural_key(),
                &original_key
            );
        });
    }
}

#[test]
fn identity_worklist_storage_stops_before_exceeding_zero_or_one_slot() {
    let slot = u64::try_from(std::mem::size_of::<&CheckedNodeId>()).unwrap();
    for (limit, expected_consumed) in [(0, 0), (slot, slot)] {
        with_request(|mut request| {
            request.limits.storage = limit;
            let error = admit_monitor_selection(&request, &Cancel::new())
                .err()
                .expect("identity scratch must respect the finite worklist capacity");
            assert_eq!(error.code().as_str(), "storage");
            assert_eq!(
                MonitorPreparationErrorCode::from_code("storage"),
                Some(error.code())
            );
            let MonitorPreparationError::Storage {
                limit: observed,
                consumed,
                next,
            } = error
            else {
                panic!("typed scratch capacity stop");
            };
            assert_eq!(observed, limit);
            assert_eq!(consumed, expected_consumed);
            assert!(next > limit - consumed);
        });
    }
}

#[test]
fn deep_caller_keys_return_typed_stops_on_a_small_stack() {
    const CHILD: &str = "QUIRE_DRIVER_MONITOR_KEY_STACK_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "deep_caller_keys_return_typed_stops_on_a_small_stack",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .expect("isolated stack-safety regression process");
        assert!(
            output.status.success(),
            "small-stack child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    LazyLock::force(&FIXTURE); // Producer/compiler setup is not run on the small stack.
    with_request(|request| {
        let mut deep = CheckedNodeId {
            package: "independent-key".into(),
            path: vec![CheckedNodePathSegment::Name("leaf".into())],
            ordinal: 0,
        };
        for _ in 0..8192 {
            deep = CheckedNodeId {
                package: "independent-key".into(),
                path: vec![CheckedNodePathSegment::Node(Arc::new(deep))],
                ordinal: 0,
            };
        }
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(64 * 1024)
                .spawn_scoped(scope, move || {
                    let edited = MonitorPreparationRequest {
                        root_key: &deep,
                        ..request
                    };
                    let error = admit_monitor_selection(&edited, &Cancel::new())
                        .err()
                        .expect("a deep independent expected key must return a mismatch");
                    let MonitorPreparationError::Selection { field, node } = error else {
                        panic!("typed mismatch, not a process abort");
                    };
                    assert_eq!(field, MonitorSelectionField::RootKey);
                    assert_eq!(node.as_ref(), edited.selection_root());
                    let mut limited = edited;
                    limited.limits.work = 4096;
                    let error = admit_monitor_selection(&limited, &Cancel::new())
                        .err()
                        .expect("finite work must stop the same deep request");
                    assert_eq!(error.code(), MonitorPreparationErrorCode::Work);
                })
                .expect("small-stack thread")
                .join()
                .expect("typed stop without host stack overflow");
        });
    });
}

#[test]
fn initializer_read_resumes_the_consumed_admission_allowance() {
    with_request(|mut request| {
        let setup_limits = request.limits;
        let required = admit_monitor_selection(&request, &Cancel::new())
            .unwrap()
            .work();
        request.limits.work = required;
        let admitted = admit_monitor_selection(&request, &Cancel::new())
            .expect("exact admission allowance succeeds");
        let mut budget = NativeJoinBudget::new(setup_limits.work, setup_limits.storage);
        let identities = SelectedExactIdentities::from_stage_rows(
            request.package,
            request.witnesses.declarations.iter().cloned(),
            request.witnesses.fields.iter().cloned(),
            &mut budget,
        )
        .unwrap();
        let error = project_monitor_initializer(&admitted, &identities, &Cancel::new())
            .expect_err("no fresh work allowance is granted to projection");
        assert!(
            matches!(error, MonitorPreparationError::Work { limit, consumed, next: 1 }
            if limit == required && consumed == required)
        );
    });
}

#[test]
fn actual_native_read_work_and_projection_causes_remain_distinct() {
    with_request(|request| {
        let plan = request.package.monitor_plans().next().unwrap();
        let native = request
            .package
            .source_callable(plan.init().node_id(), 0)
            .expect_err("the native callable allowance fails before the read");
        assert_eq!(native.code, CheckedCallableReadErrorCode::WorkLimit);
        assert_eq!(native.locus, *plan.init().node_id());
        assert_eq!(native.work, 1);
        let wrapped = MonitorPreparationError::from(native.clone());
        assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Work);
        let MonitorPreparationError::CallableRead(preserved) = wrapped else {
            panic!("native callable payload variant");
        };
        assert_eq!(*preserved, native);

        let mut absent = plan.init().node_id().clone();
        absent.ordinal += 1;
        let native = request
            .package
            .source_callable(&absent, 10)
            .expect_err("an unknown callable is not a work stop");
        assert_eq!(native.code, CheckedCallableReadErrorCode::UnknownNode);
        let wrapped = MonitorPreparationError::from(native.clone());
        assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Projection);
        let MonitorPreparationError::CallableRead(preserved) = wrapped else {
            panic!("native callable payload variant");
        };
        assert_eq!(*preserved, native);

        let CheckedTerm::Aggregate { members } = plan.init().body() else {
            panic!("admitted init body");
        };
        let CheckedTerm::Binding { value, .. } = &members[1] else {
            panic!("init expression binding");
        };
        let CheckedTerm::Reference { target } = value.as_ref() else {
            panic!("init expression reference");
        };
        let mut budget = NativeJoinBudget::new(request.limits.work, request.limits.storage);
        let identities = SelectedExactIdentities::from_stage_rows(
            request.package,
            request.witnesses.declarations.iter().cloned(),
            request.witnesses.fields.iter().cloned(),
            &mut budget,
        )
        .unwrap();
        let native = request
            .package
            .project_exact_type(target, &identities, 0)
            .expect_err("the native type allowance fails before its read");
        assert_eq!(native.code, ExactTypeProjectionErrorCode::ExhaustedWork);
        assert_eq!(*native.requested, *target);
        assert_eq!(native.work, 0);
        let wrapped = MonitorPreparationError::from(native.clone());
        assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Work);
        let MonitorPreparationError::Projection(preserved) = wrapped else {
            panic!("native type payload variant");
        };
        assert_eq!(preserved, native);
    });
}

#[test]
fn native_operation_and_key_invariant_failures_retain_their_payloads() {
    // Direct decoding does not grant the native operation admission proof.
    let operation: quire_contract_model::CheckedApplicationOperation = serde_json::from_value(
        serde_json::json!({"identity": "quire.op.integer.add", "mode": null,
            "laws": [], "member": null, "constraints": [], "leaves": []}),
    )
    .unwrap();
    let native = operation
        .read()
        .expect_err("an unadmitted operation cannot expose its views");
    let wrapped = MonitorPreparationError::from(native);
    assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Internal);
    let MonitorPreparationError::OperationRead(preserved) = wrapped else {
        panic!("native operation-read payload");
    };
    assert_eq!(preserved, native);

    // Invalid shape from the native fallible copier is an internal fault, not
    // an ordinary independently selected target inequality or temporal result.
    let native = quire_exact::NodeKey::try_new("package".to_owned(), Vec::new(), 0)
        .expect_err("an empty original key path has no canonical shape");
    assert_eq!(native, quire_exact::InvalidNodeKey::EmptyPath);
    let wrapped = MonitorPreparationError::from(native);
    assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Internal);
    let MonitorPreparationError::Key(preserved) = wrapped else {
        panic!("native key payload");
    };
    assert_eq!(preserved, native);
}

#[trace("FR-015-AC-13")]
#[test]
fn native_missing_clock_payload_still_maps_to_selection() {
    with_request(|request| {
        let native = CheckedMonitorClockReadError {
            code: CheckedMonitorClockReadErrorCode::MissingClock,
            at: Box::new(request.clause.clone()),
            occurrence: CheckedOccurrence {
                role: request.occurrence.role.clone(),
                ordinal: request.occurrence.ordinal,
            },
            work: 8,
            attempted_work: 0,
        };
        let wrapped = MonitorPreparationError::ClockRead(Box::new(native.clone()));
        assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Selection);
        let MonitorPreparationError::ClockRead(actual) = wrapped else {
            panic!("native missing-clock payload")
        };
        assert_eq!(*actual, native);
    });
}

#[trace("FR-015-AC-13", "FR-015-AC-14")]
#[test]
fn clock_admission_reads_native_model_time_without_optional_expectation() {
    with_request(|mut request| {
        request.clock_binding = None;
        let partial = admit_monitor_selection(&request, &Cancel::new()).unwrap();
        assert_eq!(partial.plan().clause(), request.clause);
        let native = partial
            .plan()
            .clock_for_claim(
                request.clause,
                &CheckedOccurrence {
                    role: request.occurrence.role.clone(),
                    ordinal: request.occurrence.ordinal,
                },
                u64::MAX,
            )
            .unwrap();
        assert_eq!(native.clock_name(), "model-time");
        assert_eq!(native.claim_name(), "SelectedMonitor");
        let CheckedMonitorClock::ModelTime { unit, .. } = native.clock() else {
            panic!("native model-time clock")
        };
        assert_eq!(unit, request.tick_unit);
        let admitted = admit_monitor_clock_selection(&request, &Cancel::new()).unwrap();
        assert_eq!(admitted.clock(), &native);
    });
}

#[trace("FR-015-AC-14")]
#[test]
fn timed_model_clock_refuses_an_independent_extra_configured_row() {
    with_request(|request| {
        assert!(request.package.temporal_bindings().unwrap_or_default().is_empty());
        let extra = TemporalBinding {
            declaration: request.clause.clone(),
            occurrence: request.occurrence.ordinal,
            clock_name: "model-time".into(),
            clock: ClockConfiguration::TimestampedEvent {
                timestamp_unit: "ms".into(),
            },
        };
        let request = MonitorPreparationRequest {
            clock_binding: Some(&extra),
            ..request
        };
        let error = admit_monitor_clock_selection(&request, &Cancel::new())
            .err()
            .expect("extra Timed clock row cannot mint a capability");
        assert_eq!(error.code(), MonitorPreparationErrorCode::Selection);
        let MonitorPreparationError::Selection { field, .. } = error else {
            panic!("typed independent clock-row refusal")
        };
        assert_eq!(field, MonitorSelectionField::Clock);
    });
}

#[trace("FR-015-AC-13")]
#[test]
fn clock_admission_retains_native_wrong_clause_role_and_ordinal_payloads() {
    for case in 0..3 {
        with_request(|request| {
            let plan = request.package.monitor_plans().next().unwrap();
            let mut occurrence = request.occurrence.clone();
            let clause = if case == 0 {
                plan.state()
            } else {
                request.clause
            };
            let expected = match case {
                0 => CheckedMonitorClockReadErrorCode::WrongClause,
                1 => {
                    occurrence.role = CheckedOccurrenceRole::Declaration;
                    CheckedMonitorClockReadErrorCode::WrongRole
                }
                2 => {
                    occurrence.ordinal = u64::MAX;
                    CheckedMonitorClockReadErrorCode::UnknownOccurrence
                }
                _ => unreachable!(),
            };
            let request = MonitorPreparationRequest {
                clause,
                occurrence: &occurrence,
                clock_binding: None,
                ..request
            };
            let native = plan
                .clock_for_claim(
                    clause,
                    &CheckedOccurrence {
                        role: occurrence.role.clone(),
                        ordinal: occurrence.ordinal,
                    },
                    u64::MAX,
                )
                .unwrap_err();
            let error = admit_monitor_clock_selection(&request, &Cancel::new())
                .err()
                .unwrap();
            assert_eq!(error.code(), MonitorPreparationErrorCode::Selection);
            let MonitorPreparationError::ClockRead(actual) = error else {
                panic!("native clock refusal")
            };
            assert_eq!(actual.code, expected);
            assert_eq!(*actual, native);
        });
    }
}

#[trace("FR-015-AC-13", "FR-015-AC-14")]
#[test]
fn same_clause_twin_claims_keep_their_exact_native_model_time_occurrences() {
    with_request(|request| {
        let names = request.package.temporal_clause_names().unwrap();
        let twin = names
            .iter()
            .find(|row| row.name.as_ref() == "TwinMonitor")
            .unwrap();
        assert_eq!(&twin.declaration, request.clause, "native shared clause");
        assert_ne!(twin.occurrence, request.occurrence.ordinal);
        let occurrence = request
            .package
            .source_map()
            .iter()
            .find(|row| {
                row.node_id == twin.declaration
                    && row.role == CheckedOccurrenceRole::Claim
                    && row.ordinal == twin.occurrence
            })
            .unwrap();
        let request = MonitorPreparationRequest {
            occurrence,
            clock_binding: None,
            ..request
        };
        let native = request
            .package
            .monitor_plans()
            .next()
            .unwrap()
            .clock_for_claim(
                request.clause,
                &CheckedOccurrence {
                    role: CheckedOccurrenceRole::Claim,
                    ordinal: twin.occurrence,
                },
                u64::MAX,
            )
            .unwrap();
        let clock = admit_monitor_clock_selection(&request, &Cancel::new()).unwrap();
        assert_eq!(clock.clock(), &native);
        assert_eq!(native.claim_name(), "TwinMonitor");
        assert_eq!(native.clock_name(), "model-time");
        assert_eq!(native.occurrence().ordinal, twin.occurrence);
        let CheckedMonitorClock::ModelTime { unit, .. } = native.clock() else {
            panic!("native twin model-time clock")
        };
        assert_eq!(unit, request.tick_unit);
    });
}

#[trace("FR-015-AC-13", "FR-015-AC-14")]
#[test]
fn clock_read_gets_remaining_work_with_exact_and_one_under_native_allowances() {
    with_request(|mut request| {
        request.clock_binding = None;
        let prefix = admit_monitor_selection(&request, &Cancel::new())
            .unwrap()
            .work();
        let plan = request.package.monitor_plans().next().unwrap();
        let occurrence = CheckedOccurrence {
            role: request.occurrence.role.clone(),
            ordinal: request.occurrence.ordinal,
        };
        let semantic = plan
            .clock_for_claim(request.clause, &occurrence, u64::MAX)
            .unwrap();
        let full = admit_monitor_clock_selection(&request, &Cancel::new()).unwrap();
        let prelude = full.selection().work() - semantic.work();
        assert!(prelude >= prefix);
        for allowance in [0, semantic.work() - 1] {
            request.limits.work = prelude + allowance;
            let error = admit_monitor_clock_selection(&request, &Cancel::new())
                .err()
                .unwrap();
            assert_eq!(error.code(), MonitorPreparationErrorCode::Work);
            let MonitorPreparationError::ClockRead(actual) = error else {
                panic!("native allowance stop")
            };
            let expected = plan
                .clock_for_claim(request.clause, &occurrence, allowance)
                .unwrap_err();
            assert_eq!(*actual, expected);
            assert_eq!(actual.work, allowance);
            assert_eq!(actual.attempted_work, 1);
        }
        request.limits.work = prelude + semantic.work();
        let exact = admit_monitor_clock_selection(&request, &Cancel::new()).unwrap();
        assert_eq!(exact.clock(), &semantic);
    });
}

#[trace("FR-015-AC-13")]
#[test]
fn clock_admission_preserves_pre_cancelled_deadline_and_original_inputs() {
    with_request(|mut request| {
        request.clock_binding = None;
        let original_source = request.occurrence.clone();
        let original_source_map = request.package.source_map().to_vec();
        let original_root = request.selection_root().clone();
        let original_clause = request.clause.clone();
        let original_subject = request.subject.clone();
        let original_profile = request.profile.clone();
        let original_limits = request.limits;
        let original_package = request.package;
        // Establish that this authentic fixture otherwise admits model time.
        let admitted = admit_monitor_clock_selection(&request, &Cancel::new()).unwrap();
        assert_eq!(admitted.clock().clock_name(), "model-time");
        let cancel = Cancel::new();
        cancel.cancel(CancelCause::Deadline);
        let error = admit_monitor_clock_selection(&request, &cancel)
            .err()
            .expect("pre-cancellation returns no clock-admitted capability");
        assert_eq!(error.code(), MonitorPreparationErrorCode::Cancelled);
        assert!(matches!(
            error,
            MonitorPreparationError::Cancelled(CancelCause::Deadline)
        ));
        assert_eq!(cancel.cause(), Some(CancelCause::Deadline));
        assert!(std::ptr::eq(request.package, original_package));
        assert_eq!(request.selection_root(), &original_root);
        assert_eq!(request.clause, &original_clause);
        assert_eq!(request.subject, &original_subject);
        assert_eq!(request.profile, &original_profile);
        assert_eq!(request.limits, original_limits);
        assert!(request.clock_binding.is_none());
        assert_eq!(request.occurrence, &original_source);
        assert_eq!(request.package.source_map(), original_source_map);
    });
}
