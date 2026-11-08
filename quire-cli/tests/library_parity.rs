// SPDX-License-Identifier: AGPL-3.0-or-later
//! The `quire` binary and the in-process library path agree for one
//! Value-family source: same disposition, identity, diagnostics and exit code.

use std::collections::BTreeMap;
use std::process::{Command, Output};

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, default_accounting, package, parse, select, Call, CallArgument, DependencyInput,
    PackageLimits, ParseRequest, SpineLimits, DEFAULT_WORK_UNITS,
};
use qsl_replay::OutcomeDocument;
use quire_driver::{
    execute, ExecutionEngine, ExecutionFailure, ExecutionRequest, ObjectEnvironment,
};
use quire_exact::{Cancel, Integer, Value};
use quire_semantic_value::catalog::Category;
use quire_semantic_value::supplied_stop::SuppliedLimits;

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");

const VALUE_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    type Digit = Int[0, 9];\n\
    function p using v(x: Digit): Boolean pure { x < 10 }\n\
    function q using v(x: Digit): Boolean pure { x < 5 }\n";

const MALFORMED_SOURCE: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\";\n\
    function f using v(: Boolean pure { true }\n";

/// Removes its directory when dropped, including after a failed assertion.
struct TestDirectory(std::path::PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Write `contents` to a fresh source file and return its directory guard and path.
fn source_file(label: &str, contents: &str) -> (TestDirectory, String) {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("quire-parity-{label}-{}", std::process::id())),
    );
    std::fs::create_dir_all(&directory.0).expect("create source directory");
    let path = directory.0.join("source.qsl");
    std::fs::write(&path, contents).expect("write source");
    let path = path.to_str().expect("UTF-8 temp path").to_owned();
    (directory, path)
}

/// What the binary must print: the kit's canonical encoding of the library
/// document plus a newline, with the exit code of the document's category.
fn expected(document: &OutcomeDocument) -> (Vec<u8>, i32) {
    let mut encoded =
        ix_cli_kit::json::encode_canonical(document, false).expect("encode library document");
    encoded.push('\n');
    (
        encoded.into_bytes(),
        i32::from(document.category().exit_code()),
    )
}

fn library_check(path: &str, bytes: &[u8]) -> OutcomeDocument {
    let source = SourceIdentity::new("quire", path);
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = BTreeMap::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &packages, limits.model, &cancel).and_then(|models| {
            check(
                parsed.value(),
                path.to_owned(),
                models.value(),
                &DependencyInput::default(),
                limits,
                &cancel,
            )
        })
    });
    OutcomeDocument::from_check(&result)
}

fn library_package(path: &str, bytes: &[u8]) -> OutcomeDocument {
    let source = SourceIdentity::new("quire", path);
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = BTreeMap::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &packages, limits.model, &cancel).and_then(|models| {
            check(
                parsed.value(),
                path.to_owned(),
                models.value(),
                &DependencyInput::default(),
                limits,
                &cancel,
            )
            .and_then(|checked| package(checked.value(), PackageLimits::default(), &cancel))
        })
    });
    OutcomeDocument::from_package(&result)
}

fn library_execute(
    path: &str,
    bytes: &[u8],
    function: &str,
    arguments: Vec<CallArgument>,
) -> OutcomeDocument {
    let packages = BTreeMap::new();
    let dependencies = DependencyInput::default();
    let request = ExecutionRequest {
        source: SourceIdentity::new("quire", path),
        package_name: path,
        path,
        bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits: SpineLimits::default(),
        call: Call {
            function: function.to_owned(),
            arguments,
            accounting: default_accounting(DEFAULT_WORK_UNITS),
            supplied: SuppliedLimits::default(),
        },
        object_environment: &ObjectEnvironment::default(),
        cancel: Cancel::new(),
        engine: ExecutionEngine::Interpreter,
    };
    match execute(request) {
        Ok(result) => result.document,
        Err(ExecutionFailure::Run { document, .. }) => *document,
        Err(other) => panic!("unexpected library failure: {other}"),
    }
}

fn digit_argument(x: i64) -> Vec<CallArgument> {
    vec![CallArgument {
        parameter: "x".to_owned(),
        value: Value::Integer(Integer::from(x)),
    }]
}

fn quire(arguments: &[&str]) -> Output {
    Command::new(QUIRE)
        .args(arguments)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run quire")
}

fn assert_parity(output: &Output, document: &OutcomeDocument, context: &str) {
    let (bytes, code) = expected(document);
    assert_eq!(output.status.code(), Some(code), "{context}: exit code");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&bytes),
        "{context}: document"
    );
}

/// Trace: FR-299-AC-1, FR-299-AC-2; `quire check`, `compile` and `run` print
/// the library outcome for the same source, byte for byte, with the library's
/// exit code, and a repeat run prints equal bytes.
#[test]
fn binary_outcomes_equal_the_library_outcomes_for_one_value_source() {
    let (_directory, path) = source_file("value", VALUE_SOURCE);
    let bytes = VALUE_SOURCE.as_bytes();
    let success = i32::from(Category::Success.exit_code());

    let checked = library_check(&path, bytes);
    assert_eq!(i32::from(checked.category().exit_code()), success);
    let output = quire(&["check", &path]);
    assert_parity(&output, &checked, "check");
    assert_eq!(output.stdout, quire(&["check", &path]).stdout);

    let packaged = library_package(&path, bytes);
    assert_eq!(i32::from(packaged.category().exit_code()), success);
    let output = quire(&["compile", &path]);
    assert_parity(&output, &packaged, "compile");
    assert_eq!(output.stdout, quire(&["compile", &path]).stdout);

    let executed = library_execute(&path, bytes, "p", digit_argument(3));
    assert_eq!(i32::from(executed.category().exit_code()), success);
    let output = quire(&["run", &path, "p", "x=3"]);
    assert_parity(&output, &executed, "run p x=3");
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("run prints one JSON document");
    assert_eq!(document["result"]["value"]["value"], true);
    assert_eq!(output.stdout, quire(&["run", &path, "p", "x=3"]).stdout);

    // The second function of the source takes the same path.
    let executed = library_execute(&path, bytes, "q", digit_argument(7));
    assert_parity(&quire(&["run", &path, "q", "x=7"]), &executed, "run q x=7");
}

/// Trace: FR-299-AC-1; a refused source gives the library's refusal document,
/// identity and exit code through `check`, `compile` and `run`.
#[test]
fn binary_refusals_equal_the_library_refusals_for_a_malformed_source() {
    let (_directory, path) = source_file("malformed", MALFORMED_SOURCE);
    let bytes = MALFORMED_SOURCE.as_bytes();

    let checked = library_check(&path, bytes);
    assert_ne!(
        i32::from(checked.category().exit_code()),
        i32::from(Category::Success.exit_code())
    );
    assert_parity(&quire(&["check", &path]), &checked, "check");
    assert_parity(
        &quire(&["compile", &path]),
        &library_package(&path, bytes),
        "compile",
    );
    assert_parity(
        &quire(&["run", &path, "f"]),
        &library_execute(&path, bytes, "f", Vec::new()),
        "run",
    );
}
