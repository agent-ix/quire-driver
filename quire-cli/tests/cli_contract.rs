// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for the initial command shell.
#![allow(
    clippy::expect_used,
    reason = "process launch failures should fail this integration test"
)]

use std::process::Command;

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
