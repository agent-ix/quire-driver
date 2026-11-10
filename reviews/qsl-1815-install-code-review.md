---
id: SR-6016
title: "Code review of locked Git installation documentation"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; QSL-1815; PR 139; README.md Getting started; Cargo.toml, Cargo.lock and quire-cli/Cargo.toml as supporting context"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-driver/FR-014", type: references}
  - {target: "ix://agent-ix/quire-driver/FR-002", type: references}
---

## Summary

Independent static review of README-only PR 139 at frozen head 0f4c30e292716670d7c2b11888e564a450eec03c. The Git installation command, local-checkout alternative and version smoke command match the workspace's package/binary ownership. No static defect was found; the actual clean-install and installed-binary smoke evidence remains pending.

## Verdict

PASS for documentation correctness only. The Git command explicitly selects the real `quire-cli` workspace package, which provides the `quire` binary; the local path names that package; both commands request the committed lockfile. Root and CLI manifests currently agree on Rust 1.98.1. The version check uses the implemented version path rather than the known silent-help path, whose implementation belongs to a separate change.

Merge readiness still requires the lead's successful installation and installed-binary checks below. This review is not a successful clean-install verdict or QSL-1815 completion claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

No applicable AssuranceProfile was found. Only code-review applies: the diff changes README prose and commands, not Rust/manifests, production code, requirements or plans. Rust review, gap-analysis, spec-review and a repository-wide computed matrix were therefore not run. No Cargo invocation, installation, test, broad gate, source edit or PR-branch edit was performed by this reviewer.

Examined documentation units:

- README-install-git, examined, README.md: `cargo install --git https://github.com/agent-ix/quire-driver --locked quire-cli`
- README-install-checkout, examined, README.md: `git clone https://github.com/agent-ix/quire-driver`, `cd quire-driver`, `cargo install --path quire-cli --locked`.
- README-install-smoke, examined, README.md: `Check the install with quire --version.`
- FR-014-AC-1, examined, spec/functional/FR-014-separate-the-user-cli-package.md: "Exactly one workspace package, `quire-cli`, provides the `quire` binary target." Package and binary declarations were inspected; existing behavior is not newly implemented by this PR.
- FR-002-AC-11, context_only, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: "When invoked with `--version`, the CLI reports the semantic version declared by the `quire-cli` Cargo package."
- FR-002-AC-12, context_only, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md: "The `quire-cli` built-binary version-agreement check uses `ix_cli_kit::version::Agreement::command` to compare `CARGO_PKG_VERSION` with the output of the built `quire --version` command and requires agreement."

Static support: root Cargo.lock is tracked; `quire-cli` is present in the workspace and lockfile; its sole declared binary is `quire`; `publish = false` remains unchanged. The CLI version handler writes `CARGO_PKG_VERSION` and uses the normal output/error path. `git diff --check` passed and the frozen worktree was clean.

The official [Cargo install command reference](https://doc.rust-lang.org/cargo/commands/cargo-install.html) confirms Git URL plus package selection, path installation and locked dependency resolution. No upstream prose or code was copied into the product.

## Pending installation evidence

The original worker's installation log has no completion status and is not successful-install evidence. The lead owns the resumed operation using the same isolated Cargo home and target; its log is `/tmp/qsl1815-install-smoke/install-resume.log` and lock timing is `/tmp/qsl1815-install-smoke/resume.time`. This reviewer does not inherit its tool session or process handles.

Before merge readiness, retain: installation exit 0; exact fetched driver commit and Cargo.lock/toolchain identity; the installed `quire --version` stdout, stderr and successful exit with package-version agreement; and the README `yes.qsl` JSON run's successful exit and expected true result. Record the known silent-help behavior separately rather than treating it as informative-help acceptance. If the install or smoke fails, route the concrete failure for correction; this static PASS does not waive it.
