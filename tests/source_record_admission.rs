// SPDX-License-Identifier: AGPL-3.0-or-later
//! Authentic ordinary-source record admission, independently of monitor closure.

use ix_trace_rs::trace;
use qsl_foundation::{node_wire::node_key_to_json, SourceIdentity};
use qsl_package::{emit_checked, Emission};
use qsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};
use qsl_semantics::model::intake::package_input;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, ExactTypeProjectionError,
    ExactTypeProjectionErrorCode, NativeJoinBudget, SelectedExactIdentities,
};
use quire_driver::monitor::source_record::{
    admit_source_records, AdmittedSourceRecords, SourceRecordAdmissionError,
    SourceRecordAdmissionErrorCode,
};
use quire_driver::monitor::{
    MonitorPreparationError, MonitorPreparationErrorCode, MonitorPreparationLimits,
};
use quire_exact::{Cancel, Presence, ValueType};

const JOIN_LIMITS: MonitorPreparationLimits = MonitorPreparationLimits {
    work: 100_000,
    storage: 1_000_000,
};

fn produce(identity: &str) -> (Emission, CheckedNodeId, CheckedNodeId) {
    let source = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
record Point { x: Integer; y: Integer; }
record Tagged { p: Point; note: Boolean?; maybe: Option<Integer>; }
record Tree { weight: Integer; kids: Sequence<Tree>[0, 3]; rest: Option<Tree>; }
function tagged using v(t: Tagged): Boolean pure { true }
function tree using v(t: Tree): Boolean pure { true }
"#;
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let parsed = parse(
        &ParseRequest {
            source: &SourceIdentity::new("agent-ix", identity),
            path: "records.native",
            bytes: source.as_bytes(),
        },
        limits.source,
        &cancel,
    )
    .unwrap()
    .into_value();
    let packages = package_input(std::iter::empty::<&[u8]>()).unwrap();
    let selected = select(&parsed, &packages, limits.model, &cancel)
        .unwrap()
        .into_value();
    let checked = check(
        &parsed,
        format!("agent-ix/{identity}"),
        &selected,
        &DependencyInput::default(),
        limits,
        &cancel,
    )
    .unwrap()
    .into_value();
    let id = |name| {
        let original = checked.package().graph().function_identity(name).unwrap();
        serde_json::from_value(node_key_to_json(&original).unwrap()).unwrap()
    };
    let tagged = id("tagged");
    let tree = id("tree");
    (emit_checked(checked.package()).unwrap(), tagged, tree)
}

fn assert_complete_producer_rows(admitted: &AdmittedSourceRecords, emission: &Emission) {
    let originals = emission.selected_source_record_rows();
    assert!(
        !originals.is_empty(),
        "authentic producer supplies source records"
    );
    assert_eq!(admitted.rows().count(), originals.len());
    for original in emission.selected_source_record_rows() {
        let row = admitted
            .rows()
            .find(|r| r.structural_node == original.structural_node)
            .unwrap();
        assert_eq!(row.structural_node, original.structural_node);
        assert_eq!(row.source_authority, original.source_authority);
        assert_eq!(row.source_identity, original.source_identity);
        assert_eq!(row.qualified_declaration, original.qualified_declaration);
        assert_eq!(row.record_key, original.record_key);
        assert_eq!(row.members.len(), original.members.len());
        for (actual, original) in row.members.iter().zip(&original.members) {
            assert_eq!(
                actual.original_member_identity,
                original.original_member_identity
            );
            assert_eq!(actual.admitted_name, original.admitted_name);
            assert_eq!(actual.presence, original.presence);
            assert_eq!(actual.type_node, original.type_node);
            assert_eq!(actual.value_type, original.value_type);
        }
    }
}

#[trace("FR-018-AC-1", "FR-018-AC-3", "FR-018-AC-4")]
#[test]
fn authentic_complete_rows_and_parameter_projections_preserve_opaque_types() {
    let (emission, tagged, tree) = produce("record-positive");
    let admitted = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    assert_eq!(admitted.rows().count(), 3);
    assert_complete_producer_rows(&admitted, &emission);
    let row = admitted
        .rows()
        .find(|r| r.qualified_declaration == [Box::from("Tagged")])
        .unwrap();
    assert_eq!(row.members[1].presence, Presence::Optional);
    assert_eq!(row.members[1].value_type, ValueType::Boolean);
    assert_eq!(row.members[2].presence, Presence::Required);
    assert_eq!(
        row.members[2].value_type,
        ValueType::Option(std::sync::Arc::new(ValueType::Integer))
    );
    for (function, name) in [(tagged, "Tagged"), (tree, "Tree")] {
        let header = admitted
            .package()
            .source_callable(&function, 100_000)
            .unwrap();
        let parameter = &header.callable().unwrap().parameters[0];
        let mut budget = NativeJoinBudget::new(100_000, JOIN_LIMITS.storage);
        let projected = admitted.project(parameter, &mut budget).unwrap();
        let row = admitted
            .rows()
            .find(|r| r.qualified_declaration == [Box::from(name)])
            .unwrap();
        assert_eq!(
            projected.value_type(),
            &ValueType::Composite(row.record_key.clone())
        );
        assert_eq!(projected.source_record_view(), Some(row));
        assert!(projected.work() > 0);
        let mut budget = NativeJoinBudget::new(100_000, u64::MAX);
        let expected = admitted
            .package()
            .project_exact_type(parameter, &SelectedExactIdentities::empty(), &mut budget)
            .unwrap_err();
        assert!(matches!(
            expected.code,
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness
        ));
        let mut budget = NativeJoinBudget::new(0, JOIN_LIMITS.storage);
        let expected = admitted.project(parameter, &mut budget).unwrap_err();
        assert_eq!(expected.code, ExactTypeProjectionErrorCode::ExhaustedWork);
    }
    drop(emission);
    assert_eq!(
        admitted.rows().count(),
        3,
        "owned native rows outlive producer setup"
    );
}

#[trace("FR-018-AC-2")]
#[test]
fn join_exhaustion_preserves_the_exact_native_failure() {
    let (emission, _, _) = produce("record-budget");
    let admitted = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    let mut budget = NativeJoinBudget::new(0, JOIN_LIMITS.storage);
    let native = SelectedExactIdentities::empty()
        .with_source_record_rows(admitted.package(), admitted.rows().cloned(), &mut budget)
        .unwrap_err();
    let error = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        MonitorPreparationLimits {
            work: 0,
            ..JOIN_LIMITS
        },
    )
    .err()
    .unwrap();
    assert_eq!(error.code(), SourceRecordAdmissionErrorCode::Join);
    let SourceRecordAdmissionError::Join(actual) = error else {
        panic!("native join disposition")
    };
    assert_eq!(actual, native);
}

#[trace("FR-018-AC-5")]
#[test]
fn authentic_join_charges_both_required_allowances_and_preserves_storage_refusal() {
    let (emission, _, _) = produce("record-native-budget");
    let admitted = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    let mut measured = NativeJoinBudget::new(u64::MAX, u64::MAX);
    SelectedExactIdentities::empty()
        .with_source_record_rows(admitted.package(), admitted.rows().cloned(), &mut measured)
        .unwrap();
    let charge = measured.last_success_delta().unwrap();
    assert!(charge.work > 0);
    assert!(charge.storage > 0);
    let exact = MonitorPreparationLimits {
        work: charge.work,
        storage: charge.storage,
    };
    let exact_admitted = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        exact,
    )
    .unwrap();
    assert_eq!(exact_admitted.rows().count(), admitted.rows().count());
    for (limits, refusal, category) in [
        (
            MonitorPreparationLimits {
                work: charge.work - 1,
                ..exact
            },
            ExactTypeProjectionErrorCode::ExhaustedWork,
            MonitorPreparationErrorCode::Work,
        ),
        (
            MonitorPreparationLimits {
                storage: charge.storage - 1,
                ..exact
            },
            ExactTypeProjectionErrorCode::ExhaustedStorage,
            MonitorPreparationErrorCode::Storage,
        ),
    ] {
        let mut native_budget = NativeJoinBudget::new(limits.work, limits.storage);
        let expected = SelectedExactIdentities::empty()
            .with_source_record_rows(
                admitted.package(),
                admitted.rows().cloned(),
                &mut native_budget,
            )
            .unwrap_err();
        assert_eq!(expected.code, refusal);
        let actual = admit_source_records(
            &emission,
            emission.evidence(),
            CheckedPackageReadLimits::bounded(),
            limits,
        )
        .err()
        .unwrap();
        let SourceRecordAdmissionError::Join(actual) = actual else {
            panic!("native resource refusal must retain its payload")
        };
        assert_eq!(actual, expected);
        assert_eq!(MonitorPreparationError::Projection(actual).code(), category);
    }
    let incomplete = ExactTypeProjectionError {
        code: ExactTypeProjectionErrorCode::IncompleteIdentityWitness,
        requested: Some(Box::new(
            admitted.rows().next().unwrap().structural_node.clone(),
        )),
        at: None,
        form: Some("selected_identity".into()),
        work: charge.work,
        denied: None,
    };
    let wrapped = MonitorPreparationError::Projection(incomplete.clone());
    assert_eq!(wrapped.code(), MonitorPreparationErrorCode::Projection);
    let MonitorPreparationError::Projection(preserved) = wrapped else {
        unreachable!()
    };
    assert_eq!(preserved, incomplete);
}

/// These are counterfactual native-row structural controls, never authentic
/// producer inputs to the public driver API.
#[trace("FR-018-AC-2")]
#[test]
fn native_structural_controls_reject_corrupt_claims_without_granting_authority() {
    let (emission, _, _) = produce("record-controls");
    let admitted = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    let original = admitted.rows().find(|r| r.members.len() == 3).unwrap();
    let mut controls = Vec::new();
    let mut row = original.clone();
    row.source_identity = "foreign".into();
    controls.push(row);
    let mut row = original.clone();
    row.qualified_declaration = vec!["foreign".into()];
    controls.push(row);
    let mut row = original.clone();
    row.members.pop();
    controls.push(row);
    let mut row = original.clone();
    row.members.swap(0, 1);
    controls.push(row);
    let mut row = original.clone();
    row.members[0].presence = Presence::Optional;
    controls.push(row);
    let mut row = original.clone();
    row.members[0].type_node = original.structural_node.clone();
    controls.push(row);
    let mut row = original.clone();
    row.members[0].original_member_identity = "".into();
    controls.push(row);
    let mut row = original.clone();
    row.members[0].admitted_name = "foreign".into();
    controls.push(row);
    for row in controls {
        let mut budget = NativeJoinBudget::new(JOIN_LIMITS.work, JOIN_LIMITS.storage);
        let error = SelectedExactIdentities::empty()
            .with_source_record_rows(admitted.package(), [row], &mut budget)
            .unwrap_err();
        assert_eq!(
            error.code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        assert!(error.work > 0);
    }
    let mut budget = NativeJoinBudget::new(JOIN_LIMITS.work, JOIN_LIMITS.storage);
    let duplicate = SelectedExactIdentities::empty()
        .with_source_record_rows(
            admitted.package(),
            [original.clone(), original.clone()],
            &mut budget,
        )
        .unwrap_err();
    assert_eq!(
        duplicate.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    let (foreign_emission, _, _) = produce("record-foreign");
    let foreign = admit_source_records(
        &foreign_emission,
        foreign_emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    let mut budget = NativeJoinBudget::new(JOIN_LIMITS.work, JOIN_LIMITS.storage);
    assert!(SelectedExactIdentities::empty()
        .with_source_record_rows(admitted.package(), foreign.rows().cloned(), &mut budget)
        .is_err());
}

#[trace("FR-018-AC-3")]
#[test]
fn reader_incompleteness_retains_its_original_native_payload() {
    let (emission, _, _) = produce("record-read-limit");
    let limits = CheckedPackageReadLimits {
        bytes: 0,
        ..CheckedPackageReadLimits::bounded()
    };
    let native = quire_contract_model::read_checked_package(
        emission.package().bytes(),
        limits,
        emission.evidence(),
    );
    let quire_contract_model::CheckedPackageDispatchResult::Incomplete(expected) = native else {
        panic!("zero bytes must stop native reading")
    };
    let actual = admit_source_records(&emission, emission.evidence(), limits, JOIN_LIMITS)
        .err()
        .unwrap();
    assert_eq!(
        actual.code(),
        SourceRecordAdmissionErrorCode::ReadIncomplete
    );
    let SourceRecordAdmissionError::ReadIncomplete(actual) = actual else {
        panic!("reader incompleteness remains distinct")
    };
    assert_eq!(actual, expected);
}

/// IR cannot authenticate producer-owned opaque values. Only the immutable
/// Emission, rather than acceptance of a caller-created row, supplies authority.
#[trace("FR-018-AC-3")]
#[test]
fn native_opaque_claim_acceptance_does_not_replace_emission_authority() {
    let (emission, _, _) = produce("record-opaque-control");
    let admitted = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    let original = admitted
        .rows()
        .find(|r| r.qualified_declaration == [Box::from("Tagged")])
        .unwrap();
    let point = admitted
        .rows()
        .find(|r| r.qualified_declaration == [Box::from("Point")])
        .unwrap();
    let mut counterfactual = original.clone();
    counterfactual.record_key = point.record_key.clone();
    counterfactual.members[0].value_type = ValueType::Boolean;
    counterfactual.members[0].original_member_identity = "producer-owned-original-name".into();
    let mut budget = NativeJoinBudget::new(JOIN_LIMITS.work, JOIN_LIMITS.storage);
    let structural_only = SelectedExactIdentities::empty()
        .with_source_record_rows(admitted.package(), [counterfactual.clone()], &mut budget)
        .unwrap();
    assert_eq!(
        structural_only.source_record_rows().next(),
        Some(&counterfactual)
    );
    assert_ne!(counterfactual.record_key, original.record_key);
    assert_ne!(
        counterfactual.members[0].value_type,
        original.members[0].value_type
    );
    let authentic = admit_source_records(
        &emission,
        emission.evidence(),
        CheckedPackageReadLimits::bounded(),
        JOIN_LIMITS,
    )
    .unwrap();
    assert_eq!(
        authentic
            .rows()
            .find(|r| r.structural_node == original.structural_node),
        Some(original)
    );
}

#[trace("FR-018-AC-1", "FR-018-AC-3")]
#[test]
fn final_real_plan_root_emission_retains_its_source_record_rows() {
    use qsl_analyze::monitor_root::{emit_plan_root, PlanRootRequest};
    use qsl_replay::{MonitorTarget, TimingObligationSettings};
    use qsl_semantics::library::LibraryName;
    use quire_exact::{Integer, Rational};

    let metadata = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let metadata: serde_json::Value = serde_json::from_slice(&metadata.stdout).unwrap();
    let manifest = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == "qsl-replay")
        .unwrap()["manifest_path"]
        .as_str()
        .unwrap();
    let path = std::path::PathBuf::from(manifest);
    let document = std::fs::read(
        path.parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples/config-version/model.semantic-ir.json"),
    )
    .unwrap();
    let parsed_document: serde_json::Value = serde_json::from_slice(&document).unwrap();
    let identity = parsed_document["package"]["identity"].as_str().unwrap();
    let packages = package_input([document.as_slice()]).unwrap();
    let source = format!(
        r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
profile timed = "quire.temporal.timed/v1";
profile complete = "quire.temporal.complete/v1";
model Selected = "{identity}";
dimension Time;
unit second : Time = rational(1, 1);
unit ms : Time = rational(1, 1000) * second;
time Selected dense unit ms;
record Point {{ x: Integer; y: Integer; }}
record Tagged {{ p: Point; note: Boolean?; maybe: Option<Integer>; }}
function recordParameter using v(t: Tagged): Boolean pure {{ true }}
temporal SelectedMonitor using timed over (candidate: Selected::ConfigVersion) clock "model-time" on origin {{
    eventually[0 ms, 4 ms] holds(candidate.versionNumber >= 0)
}}
"#
    );
    let target = MonitorTarget {
        counter: "counter".to_owned(),
        unit: "ms".to_owned(),
        width_bits: 16,
        uncertainty: Rational::from_integer(Integer::zero()),
        max_event_rate: Some(Rational::from_integer(Integer::from(2_u64))),
        max_reading_gap: 100,
        max_activations: 8,
    };
    let root = emit_plan_root(
        &PlanRootRequest {
            source: &SourceIdentity::new("agent-ix", "record-plan-root"),
            path: "record-plan-root.native",
            bytes: source.as_bytes(),
            packages: &packages,
            package_name: &LibraryName::new("agent-ix/record-plan-root").unwrap(),
            limits: SpineLimits::default(),
            clause: "SelectedMonitor",
            target: &target,
            settings: &TimingObligationSettings::default(),
        },
        &Cancel::new(),
    )
    .expect("genuine public final PlanRoot producer");
    let read_limits = CheckedPackageReadLimits {
        bytes: 8 * 1024 * 1024,
        nodes: 100_000,
        edges: 1_000_000,
        occurrences: 1_000_000,
        diagnostics: 1_000,
        work: 100_000_000,
    };
    let native = quire_contract_model::read_checked_package(
        root.emission.package().bytes(),
        read_limits,
        root.emission.evidence(),
    );
    let quire_contract_model::CheckedPackageDispatchResult::Refused(expected) = native else {
        panic!("original model documents are required by the strict owner reader")
    };
    let error = admit_source_records(
        &root.emission,
        root.emission.evidence(),
        read_limits,
        JOIN_LIMITS,
    )
    .err()
    .unwrap();
    assert_eq!(error.code(), SourceRecordAdmissionErrorCode::ReadRefused);
    let SourceRecordAdmissionError::ReadRefused(actual) = error else {
        panic!("native reader refusal remains distinct")
    };
    assert_eq!(*actual, expected);
    let mut evidence = root.emission.evidence().clone();
    evidence
        .insert_domain_package_document(identity, document)
        .unwrap();
    let admitted = admit_source_records(
        &root.emission,
        &evidence,
        CheckedPackageReadLimits {
            bytes: 8 * 1024 * 1024,
            nodes: 100_000,
            edges: 1_000_000,
            occurrences: 1_000_000,
            diagnostics: 1_000,
            work: 100_000_000,
        },
        JOIN_LIMITS,
    )
    .unwrap_or_else(|error| {
        let quire_contract_model::CheckedPackageDispatchResult::AdmittedV2(package) =
            quire_contract_model::read_checked_package(
                root.emission.package().bytes(), read_limits, &evidence)
        else {
            panic!("the original final emission must first strictly admit")
        };
        let SourceRecordAdmissionError::Join(native) = &error else {
            panic!("authentic final record admission failed: {error:?}")
        };
        let row = root.emission.selected_source_record_rows().iter()
            .find(|row| Some(&row.structural_node) == native.requested.as_deref());
        panic!("authentic final record join failed: {error:?}; original producer row: {row:?}; original admitted body: {:?}",
            native.requested.as_deref().and_then(|id| package.term(id)));
    });
    assert_eq!(admitted.package().monitor_plans().count(), 1);
    assert_complete_producer_rows(&admitted, &root.emission);
    for row in admitted.rows() {
        assert!(admitted.package().node_kind(&row.structural_node).is_some());
        for member in &row.members {
            assert!(admitted.package().node_kind(&member.type_node).is_some());
        }
    }
    for (original_name, admitted_name) in [("origin_tick", "origin"), ("amount", "value")] {
        let (producer, index) = root
            .emission
            .selected_source_record_rows()
            .iter()
            .find_map(|row| {
                row.members
                    .iter()
                    .position(|member| {
                        member.original_member_identity.as_ref() == original_name
                            && member.admitted_name.as_ref() == admitted_name
                    })
                    .map(|index| (row, index))
            })
            .expect("genuine generated row retains its original identity and final binding rename");
        let row = admitted
            .rows()
            .find(|row| row.structural_node == producer.structural_node)
            .unwrap();
        assert_eq!(
            row.members[index].original_member_identity.as_ref(),
            original_name
        );
        assert_eq!(row.members[index].admitted_name.as_ref(), admitted_name);
        assert_eq!(
            row.members[index].type_node,
            producer.members[index].type_node
        );
        assert_eq!(
            row.members[index].value_type,
            producer.members[index].value_type
        );
    }
}

#[trace("FR-018-AC-3")]
#[test]
fn source_record_error_catalog_has_exact_distinct_round_trip_spellings() {
    use SourceRecordAdmissionErrorCode::{Join, ReadIncomplete, ReadRefused};

    let expected = [
        (ReadRefused, "read_refused"),
        (ReadIncomplete, "read_incomplete"),
        (Join, "join"),
    ];
    assert_eq!(
        SourceRecordAdmissionErrorCode::all(),
        &[ReadRefused, ReadIncomplete, Join]
    );
    let mut spellings = std::collections::BTreeSet::new();
    for (code, spelling) in expected {
        assert_eq!(code.as_str(), spelling);
        assert_eq!(
            SourceRecordAdmissionErrorCode::from_code(spelling),
            Some(code)
        );
        assert!(
            spellings.insert(code.as_str()),
            "distinct catalog spellings"
        );
        assert_eq!(
            SourceRecordAdmissionErrorCode::from_code(&spelling.to_uppercase()),
            None
        );
    }
    for invalid in [
        "",
        "unknown",
        "ReadRefused",
        "read-refused",
        " join",
        "join ",
        "read_incomplete\n",
    ] {
        assert_eq!(SourceRecordAdmissionErrorCode::from_code(invalid), None);
    }
}
