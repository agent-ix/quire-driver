---
id: SR-14790
title: "IR-672 PR 93 code-review review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #93; quire-cli/src/cli.rs; src/registry.rs; FR-006-AC-26"
review_set: subset
---

## Summary

Reviewed the full PR diff with the Rust review lane folded into code review. The configured-provider path composes one QSL call and one drive, correlates call items by package, request index and occurrence, retains typed results without terminal mapping, and enforces strict provider configuration. One duplicate-identity case remains unsafe.

## Verdict

FAIL

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Identical admitted identities are idempotently registered, after which every matching session receives the selected request instead of failing closed. | quire-cli/src/cli.rs:881-887, quire-cli/src/cli.rs:927-957, src/registry.rs:119-139 |
| FND-002 | high | The production entrypoint never starts the signal listener or registers active sessions, so SIGINT/SIGTERM can terminate the CLI while leaving the isolated plugin process group running. | quire-cli/src/cli.rs:1262-1267, quire-cli/src/cli.rs:1131-1143 |

## Coverage

Inspected the changed Rust source and tests, configuration deserialization, host lifecycle, routing projection, response retention, result projection, lockfile composition, repo conventions, and relevant FR-006 criteria. No source edits or test runs were made. The current PR head was reviewed; local PR checks report no checks. Rust formatting and broader gates were not run in this reviewer pass.

Examined units:
- `FR-006-AC-1` (examined): When backend negotiation selects an out-of-process provider, the driver SHALL dispatch only that provider's negotiated items through its driver-owned process-provider seam, using the executable path named by the caller.
- `FR-006-AC-8` (examined): The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged.
- `FR-006-AC-15` (examined): A result without a current QSL terminal mapping remains typed and unmapped.
- `FR-006-AC-26` (examined): If distinct configured process sessions return the same identity from their admitted hellos, the composition begins no negotiation and closes all sessions; if QSL treats identical repeated admission as idempotent, return the existing CLI InvalidRequest input refusal.
- `FR-006-AC-27` (examined): A configured provider start or hello failure begins no negotiation, closes earlier sessions and preserves the original typed host failure.
- `FR-006-AC-4` (examined): On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is active requests cancellation and cleans up the process tree before restoring the default disposition and re-raising the signal.

## Evidence

`src/registry.rs` documents and tests identical process descriptor admission as idempotent. The configured run passes all active descriptors to `register_admitted` and only handles an error; it has no pre-registration check for repeated identities. When registration is idempotent, `drive` proceeds. The dispatch loop independently iterates every active session and selects work by equality with the routed backend identity, so both sessions with that identity receive the same request. This violates FR-006-AC-26 and can send the selected request to multiple executables.

The CLI's `entrypoint` passes a default `SignalContext`; `dispatch` ignores its signal argument and `run` passes another default context to the configured composition. Nothing calls `signals::run` in the production command path, so `SignalContext::activate` is never reached. The host creates each plugin in an isolated process group. A handled signal therefore takes the default CLI action without invoking bounded session shutdown, leaving that process group outside normal cleanup.
