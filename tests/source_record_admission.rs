// SPDX-License-Identifier: AGPL-3.0-or-later
//! Authentic ordinary-source record admission, independently of monitor closure.

use ix_trace_rs::trace;
use qsl_foundation::{node_wire::node_key_to_json, SourceIdentity};
use qsl_package::{emit_checked, Emission};
use qsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};
use qsl_semantics::model::intake::package_input;
use quire_contract_model::{
    CheckedNodeId, CheckedPackageReadLimits, ExactTypeProjectionErrorCode, SelectedExactIdentities,
};
use quire_driver::monitor::source_record::{
    admit_source_records, SourceRecordAdmissionError, SourceRecordAdmissionErrorCode,
};
use quire_exact::{Cancel, Presence, ValueType};

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

#[trace("FR-018-AC-1", "FR-018-AC-3", "FR-018-AC-4")]
#[test]
fn authentic_complete_rows_and_parameter_projections_preserve_opaque_types() {
    let (emission, tagged, tree) = produce("record-positive");
    let admitted =
        admit_source_records(&emission, CheckedPackageReadLimits::bounded(), 100_000).unwrap();
    assert_eq!(admitted.rows().count(), 3);
    for original in emission.selected_source_record_rows() {
        let row = admitted
            .rows()
            .find(|r| r.structural_node == original.structural_node)
            .unwrap();
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
        let projected = admitted.project(parameter, 100_000).unwrap();
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
        let expected = admitted
            .package()
            .project_exact_type(parameter, &SelectedExactIdentities::empty(), 100_000)
            .unwrap_err();
        assert!(matches!(
            expected.code,
            ExactTypeProjectionErrorCode::UnavailableIdentityWitness
        ));
        let expected = admitted.project(parameter, 0).unwrap_err();
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
    let admitted =
        admit_source_records(&emission, CheckedPackageReadLimits::bounded(), 100_000).unwrap();
    let native = SelectedExactIdentities::empty()
        .with_source_record_rows(admitted.package(), admitted.rows().cloned(), 0)
        .unwrap_err();
    let error = admit_source_records(&emission, CheckedPackageReadLimits::bounded(), 0)
        .err()
        .unwrap();
    assert_eq!(error.code(), SourceRecordAdmissionErrorCode::Join);
    let SourceRecordAdmissionError::Join(actual) = error else {
        panic!("native join disposition")
    };
    assert_eq!(actual, native);
}

/// These are counterfactual native-row structural controls, never authentic
/// producer inputs to the public driver API.
#[trace("FR-018-AC-2")]
#[test]
fn native_structural_controls_reject_corrupt_claims_without_granting_authority() {
    let (emission, _, _) = produce("record-controls");
    let admitted =
        admit_source_records(&emission, CheckedPackageReadLimits::bounded(), 100_000).unwrap();
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
        let error = SelectedExactIdentities::empty()
            .with_source_record_rows(admitted.package(), [row], 100_000)
            .unwrap_err();
        assert_eq!(
            error.code,
            ExactTypeProjectionErrorCode::MismatchedIdentityWitness
        );
        assert!(error.work > 0);
    }
    let duplicate = SelectedExactIdentities::empty()
        .with_source_record_rows(
            admitted.package(),
            [original.clone(), original.clone()],
            100_000,
        )
        .unwrap_err();
    assert_eq!(
        duplicate.code,
        ExactTypeProjectionErrorCode::ConflictingIdentityWitness
    );
    let (foreign_emission, _, _) = produce("record-foreign");
    let foreign = admit_source_records(
        &foreign_emission,
        CheckedPackageReadLimits::bounded(),
        100_000,
    )
    .unwrap();
    assert!(SelectedExactIdentities::empty()
        .with_source_record_rows(admitted.package(), foreign.rows().cloned(), 100_000)
        .is_err());
}

#[trace("FR-018-AC-3")]
#[test]
fn reader_incompleteness_retains_its_original_native_payload() {
    let (emission, _, _) = produce("record-read-limit");
    let limits = CheckedPackageReadLimits { bytes: 0, ..CheckedPackageReadLimits::bounded() };
    let native = quire_contract_model::read_checked_package(
        emission.package().bytes(), limits, emission.evidence());
    let quire_contract_model::CheckedPackageDispatchResult::Incomplete(expected) = native else {
        panic!("zero bytes must stop native reading")
    };
    let actual = admit_source_records(&emission, limits, 100_000).err().unwrap();
    assert_eq!(actual.code(), SourceRecordAdmissionErrorCode::ReadIncomplete);
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
    let admitted = admit_source_records(&emission, CheckedPackageReadLimits::bounded(), 100_000).unwrap();
    let original = admitted.rows().find(|r| r.qualified_declaration == [Box::from("Tagged")]).unwrap();
    let point = admitted.rows().find(|r| r.qualified_declaration == [Box::from("Point")]).unwrap();
    let mut counterfactual = original.clone();
    counterfactual.record_key = point.record_key.clone();
    counterfactual.members[0].value_type = ValueType::Boolean;
    counterfactual.members[0].original_member_identity = "producer-owned-original-name".into();
    let structural_only = SelectedExactIdentities::empty().with_source_record_rows(
        admitted.package(), [counterfactual.clone()], 100_000).unwrap();
    assert_eq!(structural_only.source_record_rows().next(), Some(&counterfactual));
    assert_ne!(counterfactual.record_key, original.record_key);
    assert_ne!(counterfactual.members[0].value_type, original.members[0].value_type);
    let authentic = admit_source_records(&emission, CheckedPackageReadLimits::bounded(), 100_000).unwrap();
    assert_eq!(authentic.rows().find(|r| r.structural_node == original.structural_node), Some(original));
}
