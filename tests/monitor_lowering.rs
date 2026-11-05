// SPDX-License-Identifier: AGPL-3.0-or-later
//! Real ordinary-source scalar lowering; no monitor Program or runtime construction claim.

use ix_trace_rs::trace;
use qsl_foundation::{node_wire::node_key_to_json, SourceIdentity};
use qsl_package::emit_checked;
use qsl_replay::spine::{check, parse, select, DependencyInput, ParseRequest, SpineLimits};
use qsl_semantics::model::intake::package_input;
use quire_contract_model::{
    read_checked_package, CheckedCallableIdentity, CheckedNodeId, CheckedPackageDispatchResult,
    CheckedPackageReadLimits, SelectedExactIdentities,
};
use quire_contract_runtime::monitor::{FunctionId, Operation};
use quire_driver::monitor::{
    lowering::lower_scalar_functions, MonitorPreparationError, MonitorPreparationLimits,
};
use quire_exact::{Cancel, CancelCause, OrderingOperator, ValueType};

/// These native views are acquired in explicit setup, outside any assembly
/// allowance. No successful callable work receipt or full preparation is claimed.
#[trace("TC-031")]
#[test]
fn actual_source_functions_share_handles_without_sharing_function_identity() {
    const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function combineOne using v(a: Integer, b: Integer): Integer pure { a + b }
function combineTwo using v(a: Integer, b: Integer): Integer pure { a + b }
"#;
    let Fixture {
        package,
        identities,
        originals,
    } = source_fixture(
        SOURCE,
        "agent-ix/lowering-alias-preflight",
        &["combineOne", "combineTwo"],
    );
    let first = package
        .source_callable(&originals[0], 10_000)
        .unwrap()
        .callable()
        .cloned()
        .expect("first original source callable");
    let second = package
        .source_callable(&originals[1], 10_000)
        .unwrap()
        .callable()
        .cloned()
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

struct Fixture {
    package: Box<quire_contract_model::CheckedPackageV2>,
    identities: SelectedExactIdentities,
    originals: Vec<CheckedNodeId>,
}

fn source_fixture(source: &str, package_name: &str, names: &[&str]) -> Fixture {
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let parsed = parse(
        &ParseRequest {
            source: &SourceIdentity::new("agent-ix", "lowering-alias-preflight"),
            path: "lowering-alias.native",
            bytes: source.as_bytes(),
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
        package_name.to_owned(),
        &selected,
        &DependencyInput::default(),
        limits,
        &cancel,
    )
    .expect("the real checker admits both functions")
    .into_value();
    let originals = names
        .iter()
        .map(|name| {
            let identity = checked
                .package()
                .graph()
                .function_identity(name)
                .expect("a named original function identity");
            let wire = node_key_to_json(&identity).expect("the owner encodes its genuine key");
            serde_json::from_value::<CheckedNodeId>(wire).expect("IR reads that original key shape")
        })
        .collect::<Vec<_>>();
    let emission = emit_checked(checked.package()).expect("the real QSL producer emits");
    let package = match read_checked_package(
        emission.package().bytes(),
        CheckedPackageReadLimits::bounded(),
        emission.evidence(),
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        other => panic!("strict IR admission of the original emission: {other:?}"),
    };
    let identities = SelectedExactIdentities::from_stage_rows(
        &package,
        emission.selected_exact_declaration_rows().iter().cloned(),
        emission.selected_exact_field_rows().iter().cloned(),
    )
    .expect("the native owner joins the actual producer's primitive setup rows");
    Fixture {
        package,
        identities,
        originals,
    }
}

const SCALAR_SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function sumValue using v(a: Integer, b: Integer): Integer pure { a + b }
function subValue using v(a: Integer, b: Integer): Integer pure { a - b }
function mulValue using v(a: Integer, b: Integer): Integer pure { a * b }
function negValue using v(a: Integer): Integer pure { -a }
function lessValue using v(a: Integer, b: Integer): Boolean pure { a < b }
function leValue using v(a: Integer, b: Integer): Boolean pure { a <= b }
function greaterValue using v(a: Integer, b: Integer): Boolean pure { a > b }
function geValue using v(a: Integer, b: Integer): Boolean pure { a >= b }
function notValue using v(a: Boolean): Boolean pure { not a }
function andValue using v(a: Boolean, b: Boolean): Boolean pure { a and b }
function orValue using v(a: Boolean, b: Boolean): Boolean pure { a or b }
function sameSum using v(a: Integer, b: Integer): Integer pure { a + b }
function nestedValue using v(a: Integer, b: Integer): Integer pure { (a + b) * (a + b) }
function identityValue using v(a: Integer): Integer pure { a }
"#;

const SCALAR_NAMES: &[&str] = &[
    "sumValue",
    "subValue",
    "mulValue",
    "negValue",
    "lessValue",
    "leValue",
    "greaterValue",
    "geValue",
    "notValue",
    "andValue",
    "orValue",
    "sameSum",
    "nestedValue",
];

fn allowances() -> MonitorPreparationLimits {
    MonitorPreparationLimits {
        work: 10_000_000,
        storage: 10_000_000,
    }
}

fn original_key(
    package: &quire_contract_model::CheckedPackageV2,
    id: &CheckedNodeId,
) -> quire_exact::NodeKey {
    // This small trusted fixture's expected key uses the existing owner codec;
    // production conversion never serializes a recursive JSON tree.
    let wire = serde_json::to_value(package.structural_key(id).unwrap()).unwrap();
    qsl_foundation::node_wire::node_key_from_json(&wire).unwrap()
}

#[trace("FR-017-AC-5", "FR-017-AC-6")]
#[test]
fn public_scalar_helper_retains_exact_operations_root_order_and_function_owners() {
    let Fixture {
        package,
        identities,
        originals,
    } = source_fixture(SCALAR_SOURCE, "agent-ix/scalar-lowering", SCALAR_NAMES);
    let cancel = Cancel::new();
    let mut requested = originals.iter().rev().cloned().collect::<Vec<_>>();
    requested.push(originals[0].clone());
    let (roots, functions, expressions) =
        lower_scalar_functions(&package, &requested, &identities, allowances(), &cancel).unwrap();
    assert_eq!(roots.len(), requested.len());
    assert_eq!(functions.len(), originals.len());
    for (index, selected) in requested.iter().enumerate() {
        assert_eq!(
            functions[roots[index].0].key,
            original_key(&package, selected)
        );
    }
    assert_eq!(roots[originals.len() - 1], roots[originals.len()]);
    let id_for = |at: usize| roots[originals.len() - 1 - at];
    for (at, name) in SCALAR_NAMES.iter().enumerate() {
        let function = id_for(at);
        let native = &functions[function.0];
        let parameter_type = if matches!(*name, "notValue" | "andValue" | "orValue") {
            ValueType::Boolean
        } else {
            ValueType::Integer
        };
        let parameter_count = if matches!(*name, "negValue" | "notValue" | "identityValue") {
            1
        } else {
            2
        };
        assert_eq!(native.parameters, vec![parameter_type; parameter_count]);
        let expected_result = if matches!(
            *name,
            "lessValue"
                | "leValue"
                | "greaterValue"
                | "geValue"
                | "notValue"
                | "andValue"
                | "orValue"
        ) {
            ValueType::Boolean
        } else {
            ValueType::Integer
        };
        assert_eq!(native.result, expected_result);
        let expression = &expressions[native.body.0];
        assert_eq!(expression.value_type, expected_result);
        let read = package.source_callable(&originals[at], 100_000).unwrap();
        let callable = read.callable().unwrap();
        assert_eq!(expression.key, original_key(&package, &callable.body));
        assert!(expression.body_path.is_empty());
        let binary = match &expression.operation {
            Operation::IntegerAdd { left, right } if matches!(*name, "sumValue" | "sameSum") => {
                Some((*left, *right))
            }
            Operation::IntegerSubtract { left, right } if *name == "subValue" => {
                Some((*left, *right))
            }
            Operation::IntegerMultiply { left, right }
                if matches!(*name, "mulValue" | "nestedValue") =>
            {
                Some((*left, *right))
            }
            Operation::BooleanAnd { left, right } if *name == "andValue" => Some((*left, *right)),
            Operation::BooleanOr { left, right } if *name == "orValue" => Some((*left, *right)),
            Operation::IntegerOrder {
                operator,
                left,
                right,
            } => {
                let expected = match *name {
                    "lessValue" => OrderingOperator::Less,
                    "leValue" => OrderingOperator::LessOrEqual,
                    "greaterValue" => OrderingOperator::Greater,
                    "geValue" => OrderingOperator::GreaterOrEqual,
                    _ => panic!("unexpected native ordering for {name}"),
                };
                assert_eq!(*operator, expected);
                Some((*left, *right))
            }
            Operation::IntegerNegate { value } if *name == "negValue" => {
                assert_argument(&expressions, *value, function, 0);
                None
            }
            Operation::BooleanNot { value } if *name == "notValue" => {
                assert_argument(&expressions, *value, function, 0);
                None
            }
            Operation::Argument {
                function: actual,
                position,
            } if *name == "identityValue" => {
                assert_eq!(*actual, function);
                assert_eq!(*position, 0);
                None
            }
            _ => panic!("wrong native operation for {name}"),
        };
        if let Some((left, right)) = binary {
            if *name == "nestedValue" {
                assert_eq!(
                    left, right,
                    "repeated genuine subexpression shares within one owner"
                );
                assert!(matches!(
                    expressions[left.0].operation,
                    Operation::IntegerAdd { .. }
                ));
            } else {
                assert_argument(&expressions, left, function, 0);
                assert_argument(&expressions, right, function, 1);
            }
        }
    }
    let sum_read = package.source_callable(&originals[0], 100_000).unwrap();
    let same_read = package.source_callable(&originals[11], 100_000).unwrap();
    let sum = sum_read.callable().unwrap();
    let same = same_read.callable().unwrap();
    assert_eq!(sum.parameters, same.parameters);
    assert_eq!(sum.body, same.body);
    let left = &expressions[functions[id_for(0).0].body.0];
    let right = &expressions[functions[id_for(11).0].body.0];
    assert_eq!(left.key, right.key);
    assert_ne!(functions[id_for(0).0].body, functions[id_for(11).0].body);
    for (position, parameter) in sum.parameters.iter().enumerate() {
        let key = original_key(&package, parameter);
        for owner in [id_for(0), id_for(11)] {
            assert!(expressions.iter().any(|e| e.key == key && matches!(e.operation, Operation::Argument { function, position: at } if function == owner && at == position)));
        }
    }
    let owned_keys = functions.iter().map(|f| f.key.clone()).collect::<Vec<_>>();
    drop(sum_read);
    drop(same_read);
    drop(package);
    drop(identities);
    drop(originals);
    drop(requested);
    for (function, expected) in functions.iter().zip(owned_keys) {
        assert_eq!(function.key, expected);
    }
    for (index, function) in functions.iter().enumerate() {
        assert!(function.body.0 < expressions.len());
        assert!(matches!(
            function.result,
            ValueType::Integer | ValueType::Boolean
        ));
        // The native address remains scoped to these returned arenas.
        assert!(roots.contains(&FunctionId(index)));
    }
}

fn assert_argument(
    expressions: &[quire_contract_runtime::monitor::Expression],
    id: quire_contract_runtime::monitor::ExpressionId,
    owner: FunctionId,
    position: usize,
) {
    assert!(
        matches!(expressions[id.0].operation, Operation::Argument { function, position: actual } if function == owner && actual == position)
    );
}

#[trace("FR-017-AC-5")]
#[test]
fn excluded_source_forms_stop_at_original_operation_without_partial_tuple() {
    const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function modValue using v(a: Integer, b: Integer): Integer pure { a mod b }
function literalValue using v(): Integer pure { 0 }
function helperValue using v(a: Integer): Integer pure { a }
function callValue using v(a: Integer): Integer pure { helperValue(a) }
function ifValue using v(a: Integer, b: Integer, flag: Boolean): Integer pure { if flag then a else b }
function implyValue using v(a: Boolean, b: Boolean): Boolean pure { a implies b }
function letValue using v(a: Integer): Integer pure { let x = a in x + a }
"#;
    let fixture = source_fixture(
        SOURCE,
        "agent-ix/scalar-excluded",
        &[
            "modValue",
            "literalValue",
            "callValue",
            "ifValue",
            "implyValue",
            "letValue",
        ],
    );
    for (index, root) in fixture.originals.iter().enumerate() {
        if index == 1 {
            let native = fixture.package.source_callable(root, 100_000).unwrap_err();
            assert_eq!(
                native.code,
                quire_contract_model::CheckedCallableReadErrorCode::MalformedBody
            );
            assert_eq!(native.locus, *root);
            let failure = lower_scalar_functions(
                &fixture.package,
                std::slice::from_ref(root),
                &fixture.identities,
                allowances(),
                &Cancel::new(),
            );
            let Err(MonitorPreparationError::CallableRead(actual)) = failure else {
                panic!("native reader refusal must be retained without a tuple");
            };
            assert_eq!(*actual, native);
            continue;
        }
        let read = fixture
            .package
            .source_callable(root, 100_000)
            .expect("nonliteral excluded bodies reach the operation boundary");
        let callable = read.callable().unwrap();
        let failure = lower_scalar_functions(
            &fixture.package,
            std::slice::from_ref(root),
            &fixture.identities,
            allowances(),
            &Cancel::new(),
        );
        let failure = match failure {
            Ok(_) => panic!("excluded source unexpectedly produced native arenas"),
            Err(failure) => failure,
        };
        let MonitorPreparationError::Unsupported(payload) = failure else {
            panic!("expected located Unsupported, got {failure:?}");
        };
        assert_eq!(payload.locus.node, callable.body);
        assert_eq!(
            payload.locus.structural_key,
            *fixture.package.structural_key(&callable.body).unwrap()
        );
        assert!(payload.locus.body_path.is_empty());
        assert!(payload.locus.occurrence.is_none());
        if let Some(quire_contract_model::CheckedTerm::Application { operation, .. }) =
            fixture.package.term(&callable.body)
        {
            assert_eq!(payload.operation.as_ref(), Some(operation));
        } else {
            assert!(payload.operation.is_none());
        }
    }
    let healthy = source_fixture(
        SCALAR_SOURCE,
        "agent-ix/scalar-healthy-after-refusal",
        SCALAR_NAMES,
    );
    assert!(lower_scalar_functions(
        &healthy.package,
        &healthy.originals[..1],
        &healthy.identities,
        allowances(),
        &Cancel::new()
    )
    .is_ok());
}

#[trace("FR-017-AC-5")]
#[test]
fn independent_component_stops_and_package_scoping_remain_native() {
    let fixture = source_fixture(SCALAR_SOURCE, "agent-ix/scalar-budgets", SCALAR_NAMES);
    let roots = &fixture.originals[..1];
    let call = |limits, cancel: &Cancel| {
        lower_scalar_functions(&fixture.package, roots, &fixture.identities, limits, cancel)
    };
    assert!(matches!(
        call(
            MonitorPreparationLimits {
                work: 0,
                ..allowances()
            },
            &Cancel::new()
        ),
        Err(MonitorPreparationError::Work { limit: 0, .. })
    ));
    assert!(matches!(
        call(
            MonitorPreparationLimits {
                storage: 0,
                ..allowances()
            },
            &Cancel::new()
        ),
        Err(MonitorPreparationError::Storage { limit: 0, .. })
    ));
    let cancel = Cancel::new();
    cancel.cancel(CancelCause::Deadline);
    assert!(matches!(
        call(allowances(), &cancel),
        Err(MonitorPreparationError::Cancelled(CancelCause::Deadline))
    ));
    let first = call(allowances(), &Cancel::new()).unwrap();
    let other = source_fixture(
        SCALAR_SOURCE,
        "agent-ix/scalar-different-package",
        SCALAR_NAMES,
    );
    let second = lower_scalar_functions(
        &other.package,
        &other.originals[..1],
        &other.identities,
        allowances(),
        &Cancel::new(),
    )
    .unwrap();
    assert_eq!(first.0, second.0);
    assert_ne!(first.1[0].key, second.1[0].key);
    assert_ne!(
        first.2[first.1[0].body.0].key,
        second.2[second.1[0].body.0].key
    );
    let changed_source = SCALAR_SOURCE.replacen("{ a + b }", "{ a - b }", 1);
    let changed = source_fixture(&changed_source, "agent-ix/scalar-budgets", SCALAR_NAMES);
    let changed_output = lower_scalar_functions(
        &changed.package,
        &changed.originals[..1],
        &changed.identities,
        allowances(),
        &Cancel::new(),
    )
    .unwrap();
    assert!(matches!(
        first.2[first.1[0].body.0].operation,
        Operation::IntegerAdd { .. }
    ));
    assert!(matches!(
        changed_output.2[changed_output.1[0].body.0].operation,
        Operation::IntegerSubtract { .. }
    ));
    assert_ne!(
        first.2[first.1[0].body.0].key,
        changed_output.2[changed_output.1[0].body.0].key
    );
    let empty = lower_scalar_functions(
        &fixture.package,
        &[],
        &fixture.identities,
        allowances(),
        &Cancel::new(),
    )
    .unwrap();
    assert!(empty.0.is_empty() && empty.1.is_empty() && empty.2.is_empty());
}

#[trace("FR-017-AC-5")]
#[test]
fn parameter_only_native_reader_boundary_keeps_its_original_failure() {
    let fixture = source_fixture(
        SCALAR_SOURCE,
        "agent-ix/scalar-parameter-only",
        &["identityValue"],
    );
    let root = &fixture.originals[0];
    let native = fixture.package.source_callable(root, 100_000).unwrap_err();
    assert_eq!(
        native.code,
        quire_contract_model::CheckedCallableReadErrorCode::MalformedBody
    );
    assert_eq!(native.locus, *root);
    let result = lower_scalar_functions(
        &fixture.package,
        &fixture.originals,
        &fixture.identities,
        allowances(),
        &Cancel::new(),
    );
    let Err(MonitorPreparationError::CallableRead(actual)) = result else {
        panic!("parameter-only read failure must remain native");
    };
    assert_eq!(*actual, native);
}

#[trace("FR-017-AC-5")]
#[test]
fn admitted_bounded_parameter_is_not_widened_to_supported_integer() {
    const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function boundedValue using v(a: Int[0, 9], b: Int[0, 9]): Integer pure { a + b }
"#;
    let fixture = source_fixture(SOURCE, "agent-ix/scalar-bounded", &["boundedValue"]);
    let read = fixture
        .package
        .source_callable(&fixture.originals[0], 100_000)
        .unwrap();
    let parameter = &read.callable().unwrap().parameters[0];
    let projection = fixture
        .package
        .project_exact_type(parameter, &fixture.identities, 100_000)
        .unwrap();
    assert!(matches!(projection.value_type(), ValueType::Int(_)));
    let result = lower_scalar_functions(
        &fixture.package,
        &fixture.originals,
        &fixture.identities,
        allowances(),
        &Cancel::new(),
    );
    let Err(MonitorPreparationError::Unsupported(payload)) = result else {
        panic!("bounded parameter must remain unsupported, never widened");
    };
    assert_eq!(payload.locus.node, *parameter);
    assert_eq!(
        payload.locus.structural_key,
        *fixture.package.structural_key(parameter).unwrap()
    );
}

#[trace("FR-017-AC-5")]
#[test]
fn measured_component_storage_includes_native_constructor_reservations() {
    let fixture = source_fixture(
        SCALAR_SOURCE,
        "agent-ix/scalar-storage-boundary",
        SCALAR_NAMES,
    );
    let invoke = |storage| {
        lower_scalar_functions(
            &fixture.package,
            &fixture.originals[..1],
            &fixture.identities,
            MonitorPreparationLimits {
                storage,
                ..allowances()
            },
            &Cancel::new(),
        )
    };
    let mut lower = 0;
    let mut upper = allowances().storage;
    assert!(invoke(upper).is_ok());
    // Bounded integer bisection over this one real assembly, not an estimate
    // of native private layouts or an assertion about full input acquisition.
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        match invoke(middle) {
            Ok(_) => upper = middle,
            Err(MonitorPreparationError::Storage {
                limit,
                consumed,
                next,
            }) => {
                assert_eq!(limit, middle);
                assert!(next > limit - consumed);
                lower = middle + 1;
            }
            Err(other) => panic!("unexpected component storage stop: {other:?}"),
        }
    }
    assert!(lower > 0);
    let Err(MonitorPreparationError::Storage {
        limit,
        consumed,
        next,
    }) = invoke(lower - 1)
    else {
        panic!("one-under measured component storage must return no tuple");
    };
    assert_eq!(limit, lower - 1);
    assert!(next > limit - consumed);
    let (roots, functions, expressions) = invoke(lower).unwrap();
    assert_eq!(roots, vec![FunctionId(0)]);
    assert!(matches!(
        expressions[functions[0].body.0].operation,
        Operation::IntegerAdd { .. }
    ));
}

#[trace("FR-017-AC-5", "FR-017-AC-6")]
#[test]
fn heterogeneous_signature_retains_unused_types_and_nonzero_argument_position() {
    const SOURCE: &str = r#"language "ix:native" edition "1-draft";
profile v = "quire.value.complete/v1";
function mixedValue using v(unused: Integer, flag: Boolean, retainedNumber: Integer): Boolean pure { not flag }
"#;
    let fixture = source_fixture(SOURCE, "agent-ix/scalar-heterogeneous", &["mixedValue"]);
    let read = fixture
        .package
        .source_callable(&fixture.originals[0], 100_000)
        .unwrap();
    let callable = read.callable().unwrap();
    let expected = vec![ValueType::Integer, ValueType::Boolean, ValueType::Integer];
    assert_eq!(callable.parameters.len(), expected.len());
    for (parameter, expected) in callable.parameters.iter().zip(&expected) {
        let native = fixture
            .package
            .project_exact_type(parameter, &fixture.identities, 100_000)
            .unwrap();
        assert_eq!(native.value_type(), expected);
    }
    let (roots, functions, expressions) = lower_scalar_functions(
        &fixture.package,
        &fixture.originals,
        &fixture.identities,
        allowances(),
        &Cancel::new(),
    )
    .unwrap();
    assert_eq!(roots, vec![FunctionId(0)]);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].parameters, expected);
    assert_eq!(functions[0].result, ValueType::Boolean);
    let body = &expressions[functions[0].body.0];
    assert_eq!(body.value_type, ValueType::Boolean);
    assert_eq!(body.key, original_key(&fixture.package, &callable.body));
    let Operation::BooleanNot { value } = body.operation else {
        panic!("expected actual BooleanNot body");
    };
    assert_argument(&expressions, value, FunctionId(0), 1);
    assert_eq!(expressions[value.0].value_type, ValueType::Boolean);
    assert_eq!(
        expressions[value.0].key,
        original_key(&fixture.package, &callable.parameters[1])
    );
    assert!(!expressions.iter().any(|e| matches!(
        e.operation,
        Operation::Argument {
            position: 0 | 2,
            ..
        }
    )));
}
