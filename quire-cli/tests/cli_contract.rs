// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for the initial command shell.
#![allow(
    clippy::expect_used,
    reason = "process launch failures should fail this integration test"
)]

use std::path::Path;
use std::process::Command;

use ix_cli_kit::version::Agreement;
use qsl_cst::format::{format_with_limits, FormatLimits, FormatRefusal};
use qsl_foundation::diagnostic::{Code, Staged};
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, package, parse, select, DependencyInput, PackageLimits, ParseRequest, SpineLimits,
};
use qsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};
use quire_exact::Cancel;
use quire_semantic_value::catalog::Category;

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");
const INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/parse-invalid.qsl"
);
const CHECK_INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/check-invalid.qsl"
);

fn parse_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read parse fixture");
    let source = SourceIdentity::new("quire", path, "filesystem", "current");
    let limits = SpineLimits::default();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &Cancel::new(),
    );
    OutcomeDocument::from_parse(&result)
        .to_bytes()
        .expect("QSL serializes its parse outcome")
}

fn check_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read check fixture");
    let source = SourceIdentity::new("quire", path, "filesystem", "current");
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &Default::default(), limits.model, &cancel).and_then(|models| {
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
        .to_bytes()
        .expect("QSL serializes its check outcome")
}

fn package_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read package fixture");
    let source = SourceIdentity::new("quire", path, "filesystem", "current");
    let limits = SpineLimits::default();
    let dependencies = DependencyInput::default();
    let packages = std::collections::BTreeMap::new();
    let cancel = Cancel::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes: &bytes,
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
                &dependencies,
                limits,
                &cancel,
            )
            .and_then(|checked| package(checked.value(), PackageLimits::default(), &cancel))
        })
    });
    OutcomeDocument::from_package(&result)
        .to_bytes()
        .expect("QSL serializes its package outcome")
}

fn format_outcome_bytes(path: &'static str) -> Vec<u8> {
    let bytes = std::fs::read(path).expect("read format fixture");
    let source = SourceIdentity::new("quire", path, "filesystem", "current");
    let limits = qsl_cst::Limits::default();
    let cancel = Cancel::new();
    let result = qsl_cst::parse_with_cancel(source, path, &bytes, limits, &cancel)
        .map_err(FormatRefusal::DiagnosedSource)
        .and_then(|parsed| format_with_limits(&parsed, FormatLimits::default()).map(Staged::new));
    OutcomeDocument::from_format(&result)
        .to_bytes()
        .expect("QSL serializes its format outcome")
}

fn argument_refusal_bytes(operation: Operation, message: &str) -> Vec<u8> {
    OutcomeDocument::new(operation, None, Code::InvalidRequest.category())
        .with_diagnostics(vec![OutcomeDiagnostic::new(
            None,
            Code::InvalidRequest.as_str(),
            None,
            message,
        )])
        .to_bytes()
        .expect("QSL serializes the argument refusal")
}

fn assert_qsl_document(cli_stdout: &[u8], qsl_bytes: &[u8]) {
    assert_eq!(
        cli_stdout.iter().filter(|&&byte| byte == b'\n').count(),
        1,
        "machine output has exactly one trailing line terminator"
    );
    let cli_document = cli_stdout
        .strip_suffix(b"\n")
        .expect("machine output ends with one line terminator");
    let cli_value: serde_json::Value =
        serde_json::from_slice(cli_document).expect("CLI emits one JSON document");
    let qsl_value: serde_json::Value =
        serde_json::from_slice(qsl_bytes).expect("QSL bytes encode one JSON document");
    assert_eq!(cli_value, qsl_value);
}

/// Trace: IR-607, one QSL exit function for unknown verbs.
#[test]
fn unknown_verb_uses_qsl_refusal_code() {
    let output = Command::new(QUIRE)
        .arg("not-a-quire-verb")
        .output()
        .expect("run unknown command");
    assert_eq!(output.status.code(), Some(20));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check dispatches through QSL's typed check operation.
#[test]
fn check_emits_qsl_check_outcome_document() {
    let output = Command::new(QUIRE)
        .args(["check", SOURCE, "--format", "json"])
        .output()
        .expect("run check command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_qsl_document(&output.stdout, &check_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-5; the CLI reads only the input path supplied on argv.
#[test]
fn check_does_not_read_an_unnamed_config_from_the_working_directory() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "quire-cli-no-implicit-config-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated working directory");
    struct RemoveDirectory(std::path::PathBuf);
    impl Drop for RemoveDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = RemoveDirectory(directory.clone());

    // A config reader would reject this malformed file. The command receives
    // its source explicitly and must not inspect ambient working-directory data.
    std::fs::write(directory.join("quire.toml"), "[malformed\n")
        .expect("write trap configuration file");

    let output = Command::new(QUIRE)
        .args(["check", SOURCE, "--format", "json"])
        .current_dir(&directory)
        .output()
        .expect("run check from isolated working directory");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_qsl_document(&output.stdout, &check_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check refusals retain QSL's typed outcome.
#[test]
fn check_refusal_emits_qsl_diagnostic_cause_and_locus() {
    let output = Command::new(QUIRE)
        .args(["check", INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run check refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");
    assert_qsl_document(&output.stdout, &check_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check source I/O failures use QSL's typed code and serializer.
#[test]
fn check_missing_source_emits_qsl_io_error_document() {
    let missing = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/missing-check-source.qsl"
    );
    let output = Command::new(QUIRE)
        .args(["check", missing, "--format", "json"])
        .output()
        .expect("run check with a missing source");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let message = stderr.trim_end();
    assert!(message.starts_with(&format!("could not read source {missing}:")));
    assert!(stderr.ends_with('\n'));

    let expected = OutcomeDocument::new(Operation::Check, None, Code::IoError.category())
        .with_diagnostics(vec![OutcomeDiagnostic::new(
            None,
            Code::IoError.as_str(),
            None,
            message,
        )])
        .to_bytes()
        .expect("QSL serializes the check I/O outcome");
    assert_qsl_document(&output.stdout, &expected);
}

/// Trace: FR-002-AC-1, FR-002-AC-3; compile dispatches through QSL's package operation.
#[test]
fn compile_emits_qsl_package_outcome_document() {
    let output = Command::new(QUIRE)
        .args(["compile", SOURCE, "--format", "json"])
        .output()
        .expect("run compile command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_qsl_document(&output.stdout, &package_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; compile refusals retain QSL's package outcome.
#[test]
fn compile_refusal_emits_qsl_package_outcome_document() {
    let output = Command::new(QUIRE)
        .args(["compile", INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run compile refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");
    assert_qsl_document(&output.stdout, &package_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; check semantic refusals retain QSL's checked outcome.
#[test]
fn check_semantic_refusal_emits_qsl_check_outcome_document() {
    let output = Command::new(QUIRE)
        .args(["check", CHECK_INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run check semantic refusal");
    assert_eq!(output.status.code(), Some(20));
    assert!(!output.stderr.is_empty());
    assert_qsl_document(&output.stdout, &check_outcome_bytes(CHECK_INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; argument refusals retain the selected QSL operation.
#[test]
fn implemented_verb_argument_refusals_use_the_selected_operation() {
    let cases = [
        (
            "parse",
            Operation::Parse,
            "unknown parse option `--unknown`",
        ),
        (
            "check",
            Operation::Check,
            "unknown check option `--unknown`",
        ),
        (
            "compile",
            Operation::Package,
            "unknown compile option `--unknown`",
        ),
        (
            "format",
            Operation::Format,
            "unknown format option `--unknown`",
        ),
    ];

    for (verb, operation, message) in cases {
        let output = Command::new(QUIRE)
            .args([verb, "--unknown", "--format", "json"])
            .output()
            .expect("run command with an unknown operation option");
        assert_eq!(output.status.code(), Some(20), "{verb} exit status");
        assert_eq!(
            output.stderr,
            format!("{message}\n").as_bytes(),
            "{verb} stderr"
        );
        assert_qsl_document(&output.stdout, &argument_refusal_bytes(operation, message));
    }
}

/// Trace: FR-002-AC-1, FR-002-AC-3; parse serializes QSL's typed parse result.
#[test]
fn parse_emits_qsl_parse_outcome_document() {
    let output = Command::new(QUIRE)
        .args(["parse", SOURCE, "--format", "json"])
        .output()
        .expect("run parse command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_qsl_document(&output.stdout, &parse_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; a parse refusal retains QSL's cause and locus.
#[test]
fn parse_refusal_emits_qsl_diagnostic_cause_and_locus() {
    let output = Command::new(QUIRE)
        .args(["parse", INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run parse refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");

    assert_qsl_document(&output.stdout, &parse_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; text mode writes QSL's formatted UTF-8 value.
#[test]
fn format_writes_qsl_formatted_text_without_editing_the_source() {
    let source = std::fs::read(SOURCE).expect("read format fixture");
    let parsed = qsl_cst::parse_with_cancel(
        SourceIdentity::new("quire", SOURCE, "filesystem", "current"),
        SOURCE,
        &source,
        qsl_cst::Limits::default(),
        &Cancel::new(),
    )
    .expect("fixture is an admissible parsed source");
    let expected = format_with_limits(&parsed, FormatLimits::default())
        .expect("fixture formats under the default limit");

    let output = Command::new(QUIRE)
        .args(["format", SOURCE])
        .output()
        .expect("run format command");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(
        std::fs::read(SOURCE).expect("source remains readable"),
        source
    );
}

/// Trace: FR-002-AC-1, FR-002-AC-3; format machine output is QSL's typed outcome.
#[test]
fn format_machine_mode_emits_qsl_format_outcome_document() {
    let output = Command::new(QUIRE)
        .args(["format", SOURCE, "--format", "json"])
        .output()
        .expect("run format in machine mode");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    assert_qsl_document(&output.stdout, &format_outcome_bytes(SOURCE));
}

/// Trace: FR-002-AC-1, FR-002-AC-3; formatter refusals retain QSL diagnostics.
#[test]
fn format_refusal_emits_qsl_diagnostic_cause_and_locus() {
    let output = Command::new(QUIRE)
        .args(["format", INVALID_SOURCE, "--format", "json"])
        .output()
        .expect("run format refusal");
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(output.stderr, b"expected `)`\n");

    assert_qsl_document(&output.stdout, &format_outcome_bytes(INVALID_SOURCE));
}

/// Trace: FR-002-AC-11, --version reports the Cargo package semantic version.
#[test]
fn version_command_reports_the_quire_cli_package_version() {
    let output = Command::new(QUIRE)
        .arg("--version")
        .output()
        .expect("run quire --version");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
    assert_eq!(
        output.stdout,
        format!("{}\n", env!("CARGO_PKG_VERSION")).as_bytes()
    );
    assert!(output.stderr.is_empty());
}

/// Trace: FR-002-AC-12, the built binary agrees with its Cargo package version.
#[test]
fn built_binary_version_agrees_with_cargo_package_version() {
    let agreement = Agreement::new()
        .surface("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"))
        .command("quire --version", Path::new(QUIRE), &["--version"]);
    let result = agreement.check();
    assert!(result.is_ok(), "version agreement failed: {result:?}");
}
