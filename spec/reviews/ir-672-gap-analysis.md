---
id: SR-14851
title: "IR-672 PR 143 process-provider gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; PR #143 diff relative to origin/peter/ir671-windows-kill-on-drop; quire-cli/src/cli.rs; Cargo.lock; FR-006-AC-1..4"
review_set: subset
---

## Summary

Compared the changed CLI integration with FR-006 process-provider and lifecycle obligations. `quoin matrix` identified FR-006-AC-1..4 as untagged; AC-1/2 are Planned and AC-3/4 are Test obligations. Manual source-to-criterion review confirms the new listener wrapper is not connected to a production process-provider session. The checked-in CG lock is also behind current main's Process backend support. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The driver path has no production dispatch from `cli::run` to `HostProcessProvider`; `run` ignores `SignalContext` and invokes only the interpreter execution path. Consequently FR-006-AC-1 process invocation and AC-3/4 plugin shutdown and signal cleanup remain unimplemented by this PR. `process_provider.rs::start` activates cancellation only after it is actually dispatched. | quire-cli/src/cli.rs:355,479; quire-cli/src/process_provider.rs:106-115; FR-006-AC-1,3,4 |
| FND-002 | medium | `Cargo.lock` pins CG at `53de0b343f5bf96ec4235ffbe96243e44649e7cc`, where `BackendKind` has only Kani. Current CG main `9d66b882` in `src/routed/capability.rs` includes `BackendKind::Process(BackendId)`, process descriptor mapping, and `negotiate_arm` dispatch. Bump this dependency so the driver can consume that existing support; no CG implementation is missing on current main. | Cargo.lock:1461-1463; CG src/routed/capability.rs; FR-006-AC-1 |

## Verdict

FAIL. FR-006-AC-1 and FR-006-AC-3..4 are not satisfied by the production CLI path, and the locked CG revision lacks the Process backend support already present on current main. IR-629 is Done; the remaining dependency task is a CG lock bump, and no missing dispatch function or arm remains in current CG main.

## Coverage

- FR-006-AC-1: checked process-provider invocation, caller-named executable, package/deadline/manifest/items handoff against the production `cli::run` path.
- FR-006-AC-2: checked the absence of a production process-provider route through which typed results could reach shared proof settlement.
- FR-006-AC-3: checked plugin-session graceful shutdown and forced tree cleanup reachability from CLI termination.
- FR-006-AC-4: checked SIGINT/SIGTERM listener to caller-owned cancellation handle and process-tree cleanup reachability.

`quoin matrix` showed all four ACs untagged in the computed matrix. Quoin 0.29.4 has no `gap-analysis` subcommand, so this is the skill's manual scoped chain check over the matrix output, changed source, adapter implementation, and FR-006 criteria. No unrelated matrix gaps are reported.
