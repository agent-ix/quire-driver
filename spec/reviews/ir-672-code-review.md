---
id: SR-14850
title: "IR-672 PR 143 Rust code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; PR #143 diff relative to origin/peter/ir671-windows-kill-on-drop; quire-cli/src/cli.rs; Cargo.lock; FR-006-AC-1..4"
review_set: subset
---

## Summary

Reviewed PR #143 head `962c31d78df9290d569902e55fc91475f4aa9070` relative to `origin/peter/ir671-windows-kill-on-drop`, excluding the inherited PR #68 diff. The changed CLI path starts a signal listener, but the run handler ignores its `SignalContext` and still has no production process-provider dispatch. The lockfile also pins CG before its Process backend dispatch support.

## Verdict

FAIL. The production run path does not activate signal cancellation for plugin sessions, and the checked-in CG lock entry predates the process backend arm required by the process-provider route.

## Reviewed units

- `quire-cli/src/cli.rs:355,479`: examined. `run` ignores its signal context; `entrypoint` starts the signal listener for Run.
- `quire-cli/src/process_provider.rs:56-130`: examined as integration context. The adapter activates the signal context in `start`, but the production run path does not instantiate or dispatch to it.
- `Cargo.lock:1461-1463`: examined. The locked CG revision is `53de0b343f5bf96ec4235ffbe96243e44649e7cc`.
- `spec/functional/FR-006-host-provider-plugins.md:143-146`: examined for the affected acceptance criteria.
- Current CG `origin/main:src/routed/capability.rs`: examined for dependency support. Current main (`9d66b882`) defines `BackendKind::Process(BackendId)`, maps `ProviderOrigin::Process` in `from_descriptor`, and includes exhaustive `negotiate_arm` dispatch. The lockfile revision's same file contains only `BackendKind::Kani`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The process-provider path is locked to CG revision `53de0b3`, whose `BackendKind` has only `Kani`; it cannot use current Process descriptor routing. Current CG main `9d66b882` already supplies `BackendKind::Process`, `from_descriptor` mapping, and `negotiate_arm` dispatch. Update the lock to the supported CG main revision as part of this PR. | Cargo.lock:1461-1463; CG src/routed/capability.rs; FR-006-AC-1 |
| FND-002 | high | `entrypoint` wraps Run in `signals::run`, but `run` accepts `_signals` and executes the interpreter through `execute`; it never constructs or dispatches to `HostProcessProvider`. Thus no production plugin session registers a cancellation handle, so handled SIGINT/SIGTERM cannot cancel and clean up a plugin process tree through this path. Wire the production process-provider dispatch and exercise the CLI path. | quire-cli/src/cli.rs:355,479; quire-cli/src/process_provider.rs:113-115; FR-006-AC-3..4 |

## Coverage

Rust lane reviewed API use, lifecycle wiring, error handling, concurrency boundaries and whether tests exercise the production integration. No tests, gates or builds were run in this review; coder-reported focused checks were taken as reported. The inherited PR #68 diff was excluded.
