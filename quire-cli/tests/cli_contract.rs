// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for the initial command shell.
#![allow(
    clippy::expect_used,
    reason = "process launch failures should fail this integration test"
)]

use std::path::Path;
use std::process::Command;

use ix_cli_kit::version::Agreement;
use qsl_foundation::diagnostic::Category;

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");

/// Trace: IR-607, one QSL exit function for unknown verbs.
#[test]
fn unknown_verb_uses_qsl_refusal_code() {
    let output = Command::new(QUIRE)
        .arg("not-a-quire-verb")
        .output()
        .expect("run unknown command");
    assert_eq!(output.status.code(), Some(20));
}

/// Trace: IR-607, recognized verbs reach the operation dispatch seam.
#[test]
fn known_verb_returns_qsl_unsupported_code_until_handler_exists() {
    let output = Command::new(QUIRE)
        .arg("check")
        .output()
        .expect("run check skeleton");
    assert_eq!(output.status.code(), Some(21));
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
