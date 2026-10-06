// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for IR-608 run engine selection.

use std::process::Command;

use qsl_foundation::diagnostic::Category;

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
