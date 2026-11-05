// SPDX-License-Identifier: AGPL-3.0-or-later
//! Production frame preparation with real QSL selection and public CG construction.

use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::time::Duration;

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{default_accounting, DependencyInput, SpineLimits};
use qsl_replay::{
    call_site, ClaimedChange, ClauseName, DocumentRef, Identifier, OperationName, ReplayLimits,
    SelectedObject,
};
use quire_contract_codegen::{
    self as cg, FrameReplayInputs, LockedSource, ProvidedDocument, ReplayInputs, SelectedFrameItem,
    WitnessDecodeLimits,
};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, FrameReplayFailure, GenerationInput, QslCg,
};
use quire_exact::{Cancel, Origin, Role};
use serde_json::json;

fn source() -> SourceIdentity {
    SourceIdentity::new("driver", "frame")
}

fn operation(name: &str) -> OperationName {
    OperationName {
        model: Identifier::new("M").unwrap(),
        object: Identifier::new("Child").unwrap(),
        operation: Identifier::new(name).unwrap(),
    }
}

struct Fixture {
    unit: String,
    domain: Vec<u8>,
    driven: Driven,
    harness: cg::StateFrameIdentity,
    site: qsl_replay::OperationSite,
    index: usize,
    transcript: String,
}

impl Fixture {
    fn new(redefined: bool) -> Self {
        let origin = |identity: &str| {
            json!({"generated": {"generatorIdentity": "ix://driver/model",
            "generatorVersion": "1.0.0", "inputIdentities": [identity]}})
        };
        let op = |owner: &str| {
            json!({"identity": format!("ix://driver/model/{owner}/touch"),
            "name": "touch", "params": [], "pre": [], "post": [],
            "origin": {"source": {"sourceIdentity": "ix://driver/model", "path": "model",
                "startLine": 1, "startColumn": 1}},
            "frame": {"modifies": [], "creates": [], "deletes": []}})
        };
        let ty = |name: &str, parents, operations| {
            json!({"identity": format!("ix://driver/model/{name}"),
            "displayName": name, "kind": {"module": "driver/model", "name": "entity"},
            "roles": [], "origin": origin(name), "constraints": [], "extensions": [],
            "unknownPolicy": "reject", "supertypes": parents, "fields": [], "operations": operations})
        };
        let mut child_op = op("Child");
        child_op["redefines"] = json!("ix://driver/model/Base/touch");
        let domain = json!({"contractVersion": "2.0.0", "canonicalization": {"algorithm": "rfc8785-v1"},
            "source": {"identity": "ix://driver/model", "version": "1.0.0", "dialect": "spec-bundle"},
            "package": {"identity": "driver/model", "version": "1.0.0", "mappingVersions": [], "profileVersions": []},
            "occurrences": [], "extensions": [], "constructs": [{"kind": {"module": "driver/model", "name": "entity"},
                "moduleVersion": "1.0.0", "construct": {"identity": "entity", "shape": "record", "members": {},
                    "meaning": "quire.meaning.model.object-type/v1"}}],
            "types": [ty("Base", json!([]), json!([op("Base")])),
                ty("Child", json!(["ix://driver/model/Base"]), if redefined {json!([child_op])} else {json!([])})]})
            .to_string().into_bytes();
        let unit = "language \"ix:native\" edition \"1-draft\";\nprofile v = \"quire.value.complete/v1\";\nmodel M = \"driver/model\";\npost Holds using v on M::Child::touch { true }\n".to_owned();
        let packages = qsl_semantics::model::intake::package_input([domain.as_slice()]).unwrap();
        let driven = drive(
            &QslCg,
            CompileInput {
                source: source(),
                path: "frame",
                bytes: unit.as_bytes(),
                package_name: "frame",
                packages: &packages,
                limits: SpineLimits::default(),
                cancel: Cancel::new(),
            },
            &build_registry(&[]).unwrap(),
            None,
            GenerationInput::default(),
        )
        .unwrap();
        let located = call_site(
            source(),
            "frame".into(),
            "frame",
            unit.as_bytes(),
            [domain.as_slice()],
            &DependencyInput::default(),
            &operation("touch"),
        )
        .unwrap();
        let clause = call_site(
            source(),
            "frame".into(),
            "frame",
            unit.as_bytes(),
            [domain.as_slice()],
            &DependencyInput::default(),
            &ClauseName(Identifier::new("Holds").unwrap()),
        )
        .unwrap();
        let graph = qsl_replay::WireGraph::new(
            driven
                .checked()
                .graph()
                .nodes
                .iter()
                .map(|node| &node.node_id),
        )
        .unwrap();
        let clause_node = driven
            .checked()
            .graph()
            .nodes
            .iter()
            .map(|node| &node.node_id)
            .find(|id| graph.key(id).ok().as_ref() == Some(&clause.site.node))
            .unwrap();
        let harness = cg::generate_state_frame_obligations(&cg::StateFrameRequest {
            ceilings: cg::ProofCeilings {
                memory_bytes: NonZeroU64::new(1024 * 1024).unwrap(),
                wall_clock: Duration::from_secs(1),
            },
            package: driven.checked(),
            clause: clause_node,
            state_path: "crate::State",
            state_fields: &[],
            subject_path: "crate::touch",
            unwind: 2,
        })
        .unwrap()
        .frame
        .identity;
        let index = driven
            .request_items
            .iter()
            .find(|item| item.occurrence() == &located.site.frame_occurrence)
            .unwrap()
            .index()
            .get();
        let transcript = format!("/// Test generated for harness `{}`\n/// Check for `assertion`: \"frame\"\n#[test]\nfn kani_concrete_playback_check_1() {{\n    let concrete_vals: Vec<Vec<u8>> = vec![\n    ];\n    kani::concrete_playback_run(concrete_vals, check);\n}}\n", harness.harness_path());
        Self {
            unit,
            domain,
            driven,
            harness,
            site: located.site,
            index,
            transcript,
        }
    }

    fn inputs(&self) -> FrameReplayInputs<'_> {
        let limits = WitnessDecodeLimits {
            bytes: 100_000,
            values: 100,
        };
        let document = |value: serde_json::Value| ProvidedDocument {
            bytes: value.to_string().into_bytes(),
        };
        FrameReplayInputs {
            run: ReplayInputs {
                decode_limits: limits,
                package_name: "frame".into(),
                source: LockedSource {
                    authority: "driver".into(),
                    identity: "frame".into(),
                    bytes: self.unit.as_bytes().to_vec(),
                },
                dependencies: vec![],
                accounting_limits: default_accounting(1_000_000),
                stage_limits: BTreeMap::new(),
                declared_domains: vec![],
                replay_limits: ReplayLimits::default(),
            },
            packages: vec![ProvidedDocument {
                bytes: self.domain.clone(),
            }],
            state_documents: vec![
                document(
                    json!({"identity": {"authority": "driver", "identity": "invocation"},
                "self": {"population": "objects", "key": "one"}, "pre": {"identity": {"authority": "driver", "identity": "pre"}}}),
                ),
                document(
                    json!({"identity": {"authority": "driver", "identity": "pre"}, "populations": [{"population": "objects", "complete": true,
                    "objects": [{"key": "one", "type": "ix://driver/model/Child", "fields": {}}]}]}),
                ),
            ],
            operation: operation("touch"),
            selected_item: SelectedFrameItem {
                package: self.driven.checked(),
                request_index: self.index,
                occurrence: &self.site.frame_occurrence,
            },
            selected_target: None,
            invocation: DocumentRef {
                authority: "driver".into(),
                identity: "invocation".into(),
            },
            change: ClaimedChange::FieldWrite {
                object: SelectedObject {
                    population: "objects".into(),
                    key: "one".into(),
                },
                field: "other".into(),
            },
            harness: self.harness.clone(),
            playback: cg::parse_assertion_playback(
                &self.transcript,
                self.harness.harness_symbol.as_str(),
                self.harness.module_symbol.as_str(),
                limits.bytes,
                limits.values,
            )
            .unwrap(),
            requested_decode_limits: limits,
        }
    }
}

/// Trace: FR-009-AC-17, TC-016.
#[test]
fn frame_projection_preserves_inherited_and_redefined_declaring_owner() {
    for (redefined, owner) in [(false, "M::Base"), (true, "M::Child")] {
        let fixture = Fixture::new(redefined);
        let count = (
            fixture.driven.request_items.len(),
            fixture.driven.routed.len(),
        );
        let prepared = fixture
            .driven
            .prepare_frame_replay(fixture.inputs())
            .unwrap();
        assert_eq!(fixture.site.frame_operation.object.to_string(), owner);
        assert_eq!(
            prepared.packet.replay_target.frame_operation,
            Some(fixture.site.frame_operation.clone())
        );
        assert!(prepared.packet.replay_target.function.is_none());
        assert!(prepared.packet.replay_target.state_clause.is_none());
        assert_eq!(
            prepared.wire.replay_target.members.frame_operation,
            Some(fixture.site.frame_operation.clone())
        );
        assert!(prepared.wire.replay_target.members.function.is_none());
        assert!(prepared.wire.replay_target.members.state_clause.is_none());
        assert!(!prepared.wire.replay_target.arguments);
        assert!(prepared.wire.replay_target.occurrence_key.is_none());
        assert_eq!(
            count,
            (
                fixture.driven.request_items.len(),
                fixture.driven.routed.len()
            )
        );
    }
}

fn assert_join_fault(result: Result<cg::FrameReplay, FrameReplayFailure>) {
    let FrameReplayFailure::Driver(error) = result.expect_err("selection must fail closed") else {
        panic!("expected driver association fault");
    };
    assert_eq!(error.code(), "runtime_invariant");
    assert_eq!(error.stage(), quire_driver::DriverStage::Replay);
    assert!(matches!(
        error,
        quire_driver::DriverRefusal::Fault(quire_driver::DriverFault::FrameReplayAssociation)
    ));
}

/// Trace: FR-009-AC-19, TC-016.
#[test]
fn frame_projection_rejects_missing_changed_and_caller_populated_associations() {
    let fixture = Fixture::new(false);
    let other = Fixture::new(true);
    let mut inputs = fixture.inputs();
    inputs.selected_item.package = other.driven.checked();
    assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
    let mut inputs = fixture.inputs();
    inputs.selected_item.request_index = usize::MAX;
    assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
    let mut inputs = fixture.inputs();
    inputs.selected_target = Some(cg::SelectedFrameTarget {
        operation: fixture.site.frame_operation.clone(),
        package_id: fixture.harness.package.clone(),
        request_index: fixture.index,
        occurrence: fixture.site.frame_occurrence.clone(),
    });
    assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
    for item in &fixture.driven.request_items {
        if item.occurrence() == &fixture.site.frame_occurrence {
            continue;
        }
        let mut inputs = fixture.inputs();
        inputs.selected_item.request_index = item.index().get();
        assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
        let mut inputs = fixture.inputs();
        inputs.selected_item.occurrence = item.occurrence();
        assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
    }
    for (role, ordinal) in [("claim", 0), ("generated", 1)] {
        let changed = qsl_replay::OccurrenceKey::new(
            fixture.site.frame_occurrence.node().clone(),
            Origin::new(Role::new(role), ordinal),
        );
        let mut inputs = fixture.inputs();
        inputs.selected_item.occurrence = &changed;
        assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
    }
    let mut inputs = fixture.inputs();
    inputs.run.source.bytes.extend_from_slice(b"\n");
    assert_join_fault(fixture.driven.prepare_frame_replay(inputs));
}

/// Trace: FR-009-AC-18, TC-016.
#[test]
fn frame_projection_preserves_qsl_refusal_before_any_join_or_codegen_check() {
    let fixture = Fixture::new(false);
    let expected = call_site(
        source(),
        "frame".into(),
        "frame",
        fixture.unit.as_bytes(),
        [fixture.domain.as_slice()],
        &DependencyInput::default(),
        &operation("missing"),
    )
    .unwrap_err();
    let mut inputs = fixture.inputs();
    inputs.operation = operation("missing");
    inputs.selected_item.request_index = usize::MAX;
    inputs.harness.state_fields.push("invalid-for-cg".into());
    let FrameReplayFailure::CallSite(actual) =
        fixture.driven.prepare_frame_replay(inputs).unwrap_err()
    else {
        panic!("QSL refusal must precede driver and CG checks");
    };
    assert_eq!(format!(actual), format!(expected));
    assert!(matches!(
        *actual,
        qsl_replay::CallSiteRefusal::UnknownOperation { .. }
    ));
}

/// Trace: FR-009-AC-19, TC-016.
#[test]
fn public_reporting_inventory_cannot_rebind_retained_frame_request() {
    let mut fixture = Fixture::new(false);
    fixture.driven.request_items.reverse();
    fixture.driven.routed.clear();
    fixture.driven.settlements.clear();
    fixture
        .driven
        .prepare_frame_replay(fixture.inputs())
        .expect("authentic original selection survives reporting edits");
    fixture.driven.request_items.clear();
    fixture
        .driven
        .prepare_frame_replay(fixture.inputs())
        .expect("private request inventory remains authoritative");
}
