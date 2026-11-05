// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original-source setup preflight, not executable lowering or runtime qualification.

use ix_trace_rs::trace;
use qsl_foundation::{node_wire::node_key_to_json, SourceIdentity};
use qsl_package::emit_checked;
use qsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};
use qsl_semantics::model::intake::package_input;
use quire_contract_model::{
    read_checked_package, CheckedCallableIdentity, CheckedNodeId, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, SelectedExactIdentities,
};
use quire_exact::{Cancel, ValueType};

/// These native views are acquired in explicit setup, outside any assembly
/// allowance. No successful callable work receipt or full preparation is claimed.
#[trace("TC-031")]
#[test]
fn actual_source_functions_share_handles_without_sharing_function_identity() {
    const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function first using v(a: Integer, b: Integer): Integer pure { a + b }
function second using v(a: Integer, b: Integer): Integer pure { a + b }
"#;
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let parsed = parse(
        &ParseRequest {
            source: &SourceIdentity::new("agent-ix", "lowering-alias-preflight"),
            path: "lowering-alias.native",
            bytes: SOURCE.as_bytes(),
        },
        limits.source,
        &cancel,
    )
    .expect("the original ordinary source parses")
    .into_value();
    let packages = package_input(std::iter::empty::<&[u8]>()).unwrap();
    let selected = select(&parsed, &packages, limits.model, &cancel)
        .unwrap()
        .into_value();
    let checked = check(
        &parsed,
        "agent-ix/lowering-alias-preflight".to_owned(),
        &selected,
        &DependencyInput::default(),
        limits,
        &cancel,
    )
    .expect("the real checker admits both functions")
    .into_value();
    let originals = ["first", "second"].map(|name| {
        let identity = checked
            .package()
            .graph()
            .function_identity(name)
            .expect("a named original function identity");
        let wire = node_key_to_json(identity).expect("the owner encodes its genuine key");
        serde_json::from_value::<CheckedNodeId>(wire).expect("IR reads that original key shape")
    });
    let emission = emit_checked(checked.package()).expect("the real QSL producer emits");
    let package = match read_checked_package(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        emission.evidence(),
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => panic!("strict IR admission of the original emission: {other:?}"),
    };
    let first = package
        .source_callable(&originals[0], 10_000)
        .unwrap()
        .expect("first original source callable");
    let second = package
        .source_callable(&originals[1], 10_000)
        .unwrap()
        .expect("second original source callable");
    assert_eq!(
        first.identity,
        CheckedCallableIdentity::Function(originals[0].clone())
    );
    assert_eq!(
        second.identity,
        CheckedCallableIdentity::Function(originals[1].clone())
    );
    assert_ne!(first.identity, second.identity);
    assert_ne!(
        package.structural_key(&originals[0]),
        package.structural_key(&originals[1])
    );
    assert_eq!(first.parameters.len(), 2);
    assert_eq!(
        first.parameters, second.parameters,
        "genuine same-package parameter handle sharing"
    );
    assert_eq!(
        first.body, second.body,
        "genuine identical expression handle under distinct functions"
    );
    let identities = SelectedExactIdentities::from_stage_rows(
        &package,
        emission.selected_exact_declaration_rows().iter().cloned(),
        emission.selected_exact_field_rows().iter().cloned(),
    )
    .expect("the native owner joins the actual producer's primitive setup rows");
    for node in first
        .parameters
        .iter()
        .chain([&first.body, &originals[0], &originals[1]])
    {
        let projected = package
            .project_exact_type(node, &identities, 10_000)
            .expect("native projection of each actual primitive semantic type");
        assert_eq!(projected.node(), node);
        assert_eq!(projected.value_type(), &ValueType::Integer);
        assert!(
            projected.work() > 0,
            "native type read carries its own work receipt"
        );
    }
}
