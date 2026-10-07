// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for CLI run options and QSL results.

use std::process::Command;
#[cfg(unix)]
use std::process::{ExitStatus, Stdio};

use qsl_foundation::diagnostic::{Category, Code};
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    default_accounting, Call, DependencyInput, SpineLimits, DEFAULT_WORK_UNITS,
};
use qsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};
use quire_driver::{execute, ExecutionEngine, ExecutionRequest};

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");

#[cfg(unix)]
fn status_with_unwritable_stderr(arguments: &[&str], reserved_exit: bool) -> ExitStatus {
    let mut command = Command::new(QUIRE);
    command.args(arguments).stdout(Stdio::null());
    if reserved_exit {
        command.env("TC018_CASE", "reserved-exit");
    }
    if let Ok(full) = std::fs::OpenOptions::new().write(true).open("/dev/full") {
        return command
            .stderr(Stdio::from(full))
            .status()
            .expect("run quire with /dev/full as stderr");
    }

    let mut child = command
        .stderr(Stdio::piped())
        .spawn()
        .expect("run quire with a pipe for stderr");
    drop(child.stderr.take());
    child.wait().expect("wait for quire after closing stderr")
}

fn expected_document_bytes(code: Code, message: &str) -> Vec<u8> {
    let document =
        OutcomeDocument::new(Operation::Execute, None, code.category()).with_diagnostics(vec![
            OutcomeDiagnostic::new(None, code.as_str(), None, message),
        ]);
    let mut bytes =
        ix_cli_kit::json::encode_canonical(&document, false).expect("encode expected QSL document");
    bytes.push('\n');
    bytes.into_bytes()
}

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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("s1.input_bytes=invalid"));
    assert!(stderr.ends_with('\n'));
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-8, FR-002-AC-9; source I/O errors retain QSL's native code.
#[test]
fn missing_source_emits_qsl_io_error_document_and_diagnostic() {
    let output = Command::new(QUIRE)
        .args(["run", "missing-source.qsl", "yes", "--format", "json"])
        .output()
        .expect("run quire with a missing source");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("could not read source missing-source.qsl:"));
    assert!(stderr.ends_with('\n'));
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::IoError, stderr.trim_end())
    );
}

/// Trace: FR-002-AC-1, FR-002-AC-8; root argv refusals name the rejected argument.
#[test]
fn root_argv_refusal_names_invalid_format_without_result_output() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--format", "dot"])
        .output()
        .expect("run quire with an invalid root format");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("dot"));
}

/// Trace: FR-002-AC-1, FR-002-AC-8; run-only options are named when refused on another verb.
#[test]
fn non_run_verb_refuses_and_names_run_only_option() {
    for option in ["--engine", "--limit"] {
        let output = Command::new(QUIRE)
            .args(["check", option, "gpu"])
            .output()
            .expect("run quire with a run-only option on check");
        assert_eq!(output.status.code(), Some(20), "{option}");
        assert!(output.stdout.is_empty(), "{option}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(option),
            "{option}"
        );
    }
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

/// Trace: FR-003-AC-1, the exposed checking work bound reaches QSL unchanged.
#[test]
fn checking_work_limit_changes_the_call_outcome() {
    let exhausted = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", "s3.work_units=0"])
        .output()
        .expect("run with exhausted work limit");
    assert_eq!(
        exhausted.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let sufficient = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", "s3.work_units=1000"])
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

/// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8.
#[test]
fn qsl_rejects_unknown_malformed_and_duplicate_limits_before_source_read() {
    for operand in [
        "normalization.work_units=1",
        "s1.input_bytes=invalid",
        "s1.input_bytes=1", // duplicated below using the same option twice
    ] {
        let mut command = Command::new(QUIRE);
        command.args(["run", "missing-source.qsl", "yes", "--limit", operand]);
        if operand == "s1.input_bytes=1" {
            command.args(["--limit", operand]);
        }
        let output = command.output().expect("run with invalid QSL limit");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Refusal.exit_code())),
            "{operand}"
        );
        assert!(output.stdout.is_empty(), "{operand}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(operand), "{operand}: {stderr}");
        assert!(
            !stderr.contains("could not read source"),
            "{operand}: {stderr}"
        );
    }
}

/// Trace: FR-003-AC-5, QSL-recognized settings absent from run bounds are distinct refusals.
#[test]
fn recognized_setting_unavailable_for_run_is_named_before_source_read() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "explore.states=2",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a setting unavailable to run");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("explore.states"), "{stderr}");
    assert!(
        stderr.contains("unavailable for the `run` operation"),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read source"), "{stderr}");
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-003-AC-2, parser refusals take precedence over operation bounds.
#[test]
fn invalid_operand_precedes_recognized_but_unavailable_setting() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "explore.states=2",
            "--limit",
            "normalization.work_units=1",
        ])
        .output()
        .expect("run with an unknown operand and unavailable setting");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("normalization.work_units"), "{stderr}");
    assert!(
        !stderr.contains("unavailable for the `run` operation"),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read source"), "{stderr}");
}

/// Trace: FR-002-AC-8; a full stderr device does not replace a refusal exit with a panic.
#[cfg(unix)]
#[test]
fn refusal_keeps_its_exit_code_when_stderr_is_unwritable() {
    let status = status_with_unwritable_stderr(&["run", SOURCE, "yes", "--format", "dot"], false);
    assert_eq!(status.code(), Some(20));
}

/// Trace: FR-002-AC-6, the reserved QSL result falls back without stdout output.
#[cfg(feature = "tc018-test-adapter")]
#[test]
fn reserved_exit_code_uses_internal_failure_fallback() {
    let version = Command::new(QUIRE)
        .arg("--version")
        .env("TC018_CASE", "reserved-exit")
        .output()
        .expect("run the test-built quire binary with --version");
    assert_eq!(version.status.code(), Some(30));
    assert!(version.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&version.stderr),
        "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30.\n"
    );

    for format in ["text", "json"] {
        let output = Command::new(QUIRE)
            .args(["run", SOURCE, "yes", "--format", format])
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

    #[cfg(unix)]
    {
        let status =
            status_with_unwritable_stderr(&["run", SOURCE, "yes", "--format", "json"], true);
        assert_eq!(status.code(), Some(30));
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
