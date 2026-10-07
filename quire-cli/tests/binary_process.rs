// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for IR-608 run engine selection.

use std::process::Command;

use qsl_foundation::diagnostic::Category;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    default_accounting, Call, DependencyInput, SpineLimits, DEFAULT_WORK_UNITS,
};
use qsl_replay::OutcomeDocument;
use quire_driver::{execute, ExecutionEngine, ExecutionRequest};

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");

/// Trace: FR-004-AC-1, omitted engine selects the linked interpreter.
#[test]
fn run_without_engine_executes_with_the_interpreter() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes"])
        .output()
        .expect("run quire");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-9; JSON is the unchanged QSL outcome encoded by the kit.
#[test]
fn machine_run_writes_the_canonical_qsl_outcome() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--format", "json"])
        .output()
        .expect("run quire in machine mode");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let packages = std::collections::BTreeMap::new();
    let dependencies = DependencyInput::default();
    let source_bytes = std::fs::read(SOURCE).expect("read source fixture");
    let request = ExecutionRequest {
        source: SourceIdentity::new("quire", SOURCE, "filesystem", "current"),
        path: SOURCE,
        bytes: &source_bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits: SpineLimits::default(),
        call: Call {
            function: "yes".to_owned(),
            arguments: Vec::new(),
            accounting: default_accounting(DEFAULT_WORK_UNITS),
        },
        engine: ExecutionEngine::Interpreter,
    };
    let result = execute(request).expect("obtain typed QSL result");
    let expected =
        ix_cli_kit::json::encode_canonical(&OutcomeDocument::from_call(&result.outcome), false)
            .expect("encode typed outcome through kit");
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    let qsl_bytes = result
        .outcome_bytes()
        .expect("encode typed outcome through QSL");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse child outcome"),
        serde_json::from_slice::<serde_json::Value>(&qsl_bytes).expect("parse QSL outcome bytes")
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-8, FR-002-AC-9; pre-QSL request refusals use QSL's catalog code.
#[test]
fn malformed_run_limit_emits_one_qsl_outcome_and_streams_diagnostic() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "s1.input_bytes=invalid",
            "--format",
            "json",
        ])
        .output()
        .expect("run quire with malformed limit");
    assert_eq!(output.status.code(), Some(20));
    let document = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .expect("one JSON outcome document");
    assert_eq!(document["format"], "quire-outcome/1");
    assert_eq!(document["operation"], "execute");
    assert_eq!(document["category"], "refusal");
    assert_eq!(document["diagnostics"][0]["code"], "invalid_runtime_input");
    assert!(String::from_utf8_lossy(&output.stderr).contains("s1.input_bytes=invalid"));
}

/// Trace: FR-004-AC-3, unavailable engines refuse before reading a source.
#[test]
fn unavailable_engines_are_not_substituted() {
    for engine in ["aot", "jit"] {
        let output = Command::new(QUIRE)
            .args(["run", "missing-source.qsl", "yes", "--engine", engine])
            .output()
            .expect("run quire");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Unsupported.exit_code())),
            "{engine}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(engine),
            "typed unsupported refusal names {engine}"
        );
    }
}

/// Trace: FR-004-AC-2, a value outside the engine set is refused.
#[test]
fn an_unknown_engine_is_a_refusal() {
    let output = Command::new(QUIRE)
        .args(["run", "missing-source.qsl", "yes", "--engine", "gpu"])
        .output()
        .expect("run quire");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("gpu"));
}

/// Trace: FR-003-AC-1, caller work limits reach QSL's interpreter call.
#[test]
fn runtime_work_limit_changes_the_call_outcome() {
    let exhausted = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", "work_units=0"])
        .output()
        .expect("run with exhausted work limit");
    assert_eq!(
        exhausted.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let sufficient = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", "work_units=1000"])
        .output()
        .expect("run with sufficient work limit");
    assert_eq!(
        sufficient.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-003-AC-3 / TC-010, a lifecycle stage limit uses QSL's exit code.
#[test]
fn lifecycle_stage_limit_uses_incomplete_exit_code() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", "s1.input_bytes=1"])
        .output()
        .expect("run with an exhausted source-byte limit");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let source_bytes = std::fs::read(SOURCE).expect("read fixture to set exact source bound");
    let bound = format!("s1.input_bytes={}", source_bytes.len());
    let raised = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", &bound])
        .output()
        .expect("run with the source bound raised to the fixture length");
    assert_eq!(
        raised.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-003-AC-2, malformed caller limits are rejected before file access.
#[test]
fn malformed_limit_is_named_and_refused_before_source_read() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "s1.input_bytes=invalid",
        ])
        .output()
        .expect("run with malformed limit");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("s1.input_bytes=invalid"));
}

/// Trace: FR-002-AC-6, the reserved QSL result falls back without stdout output.
#[cfg(feature = "tc018-test-adapter")]
#[test]
fn reserved_exit_code_uses_internal_failure_fallback() {
    for format in ["text", "json"] {
        let output = Command::new(QUIRE)
            .args(["check", "--format", format, "missing-source.qsl"])
            .env("TC018_CASE", "reserved-exit")
            .output()
            .expect("run the test-built quire binary with a valid verb");
        assert_eq!(output.status.code(), Some(30), "{format}");
        assert!(output.stdout.is_empty(), "{format}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("reserved code 5"), "{format}: {stderr}");
        assert!(stderr.contains("fallback code 30"), "{format}: {stderr}");
        assert_eq!(
            stderr.as_ref(),
            "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30.\n",
            "{format}"
        );
    }
}

/// The normal binary ignores TC-018's selector even when the caller sets it.
#[cfg(not(feature = "tc018-test-adapter"))]
#[test]
fn default_binary_does_not_enable_the_test_exit_seam() {
    let output = Command::new(QUIRE)
        .args(["check", "--format", "json", "missing-source.qsl"])
        .env("TC018_CASE", "reserved-exit")
        .output()
        .expect("run the default-feature quire binary");
    assert_eq!(output.status.code(), Some(21));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
