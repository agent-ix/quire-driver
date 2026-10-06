// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-014 (TC-028): keep the CLI executable and its frontend-owned files in
//! the separate `quire-cli` workspace package.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "metadata and layout mismatches are the test failure report"
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("quire-cli has a workspace root")
        .to_path_buf()
}

fn metadata() -> Value {
    let root = workspace_root();
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--offline",
            "--no-deps",
        ])
        .current_dir(&root)
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata is JSON")
}

fn direct_normal_dependencies(package: &Value) -> BTreeSet<&str> {
    package["dependencies"]
        .as_array()
        .expect("package dependencies")
        .iter()
        .filter(|dependency| dependency["kind"].is_null())
        .map(|dependency| dependency["name"].as_str().expect("dependency name"))
        .collect()
}

/// Trace: FR-014-AC-1/AC-2, Cargo owns one quire target in quire-cli.
#[test]
fn cargo_target_and_dependencies_belong_to_quire_cli() {
    let metadata = metadata();
    let packages = metadata["packages"].as_array().expect("workspace packages");
    let cli_packages: Vec<&Value> = packages
        .iter()
        .filter(|package| package["name"] == "quire-cli")
        .collect();
    assert_eq!(cli_packages.len(), 1, "one quire-cli workspace package");
    let cli = cli_packages[0];

    let quire_targets: Vec<(&Value, &Value)> = packages
        .iter()
        .flat_map(|package| {
            package["targets"]
                .as_array()
                .expect("package targets")
                .iter()
                .filter(move |target| {
                    target["name"] == "quire"
                        && target["kind"]
                            .as_array()
                            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
                })
                .map(move |target| (package, target))
        })
        .collect();
    assert_eq!(quire_targets.len(), 1, "one Cargo binary named quire");
    assert_eq!(quire_targets[0].0["name"], "quire-cli");

    let dependencies = direct_normal_dependencies(cli);
    assert!(dependencies.contains("quire-driver"));
    assert!(dependencies.contains("quire-cache"));
    assert!(!dependencies.contains("ix-cli-kit"));
}

/// Trace: FR-014-AC-4/AC-5, the parser, binary entry point, CLI tests and
/// fixtures are owned by quire-cli, and the library exports no CLI module.
#[test]
fn parser_and_cli_tests_live_in_the_cli_package() {
    let root = workspace_root();
    let cli_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parser_path = cli_root.join("src/cli.rs");
    let parser = fs::read_to_string(&parser_path).expect("read CLI parser source");
    assert!(parser.contains("fn parse_invocation("));
    assert!(parser.contains("enum Verb"));
    assert!(parser.contains("enum OutputFormat"));

    let main_path = cli_root.join("src/main.rs");
    let main = fs::read_to_string(&main_path).expect("read CLI entry point");
    assert!(main.contains("cli::entrypoint()"));
    assert!(!main.contains("fn parse_invocation("));

    let driver_lib = fs::read_to_string(root.join("src/lib.rs")).expect("read driver library");
    assert!(!driver_lib.contains("pub mod cli;"));

    for relative in ["tests/cli_contract.rs", "tests/binary_process.rs"] {
        assert!(
            cli_root.join(relative).is_file(),
            "missing quire-cli/{relative}"
        );
        assert!(
            !root.join(relative).exists(),
            "CLI test remains in the quire-driver package: {relative}"
        );
    }
    let fixture = "tests/fixtures/ir608-run.qsl";
    assert!(
        cli_root.join(fixture).is_file(),
        "missing quire-cli/{fixture}"
    );
    assert!(
        !root.join(fixture).exists(),
        "CLI fixture remains in quire-driver"
    );
}
