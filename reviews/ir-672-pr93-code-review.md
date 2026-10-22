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

## Dispositions

Historical dispositions restored verbatim in meaning from the existing tracker records.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | let (cleanup_diagnostics, cleanup_failures) = close_configured_providers(active); return Err(Box::new(ConfiguredRunFailure {     call: Some(call),     document,     kind: ConfiguredRunErrorKind::DuplicateIdentity { backend },     cleanup_diagnostics,     cleanup_failures, })); |
| FND-002 | fixed | let configured = crate::signals::run(move \|signals\| {     run_configured_providers(ConfiguredRunRequest {         path: &options.path,         bytes: &bytes,         source,         limits,         call,         packages,         dependencies,         config: provider_config,         signals,     }) }); |

## Current revalidation

PASS — prior findings remain closed. Rust review lane: inspected supported-cohort source adjustments, typed refusal projection, statistical fixture, configured call correlation, duplicate-session refusal, signal context and session cleanup. Original duplicate-identity and signal findings remain fixed. No new code finding. All git-source repositories have one resolved revision; runtime remains on the approved retained revision. Aggregate gate is outside this static review.

Reviewed scope: the frozen final PR diff and supported-cohort/rebase delta, as identified by the tracker comment.

## Revalidated scope excerpts

- `FR-006-AC-1` (role: examined): When negotiation selects a process provider, the driver invokes its process-provider seam with the configuration entry associated with that provider's once-admitted hello identity, one `EmittedPackage` value, that entry's caller-supplied deadline, the registered-candidate manifest snapshot and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry.
- `FR-006-AC-3` (role: examined): Before normal CLI command termination, the adapter requests graceful shutdown of every active plugin session; on Linux and macOS, if `RunLimits.cancellation_grace` expires first, the process tree terminates and cleanup completes within one second. The merged Windows host force-terminates the Job Object through `terminate_tree`/`terminate_job`; native Windows evidence remains planned. Kill-on-close is separately planned under IR-674.
- `FR-006-AC-4` (role: examined): On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal. No QSL cancellation cause is claimed; with no active session, the platform's default signal disposition is retained.
- `FR-006-AC-8` (role: examined): The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged.
- `FR-006-AC-10` (role: examined): The `quire-cli` adapter serializes each item's request index, claim locus and kind from the corresponding retained QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation.
- `FR-006-AC-11` (role: examined): The `quire-cli` adapter serializes each item's selected backend from CG's routed output and its candidate set from the corresponding retained QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation.
- `FR-006-AC-12` (role: examined): The `quire-cli` adapter serializes each item's extent and bounds from the corresponding retained QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation.
- `FR-006-AC-13` (role: examined): The `quire-cli` adapter serializes each item's temporal proof bounds under `items[].temporal.subject.proof_bounds`.
- `FR-006-AC-15` (role: examined): A validated result without a specified QSL terminal mapping remains typed and unmapped in the driver's shared settlement path.
- `FR-006-AC-24` (role: examined): The `--provider-config` file accepts only the documented strict JSON shape; missing/unreadable files, malformed JSON, missing or wrongly typed fields, a `deadline_ms` that is missing, non-integer, outside 1..=9,007,199,254,740,991, `null` or wrongly typed optional process budgets, and unknown fields at any object level are rejected before any provider starts. In machine mode, these failures use the existing `Operation::Execute` `Code::InvalidRequest` input-refusal result and serialize that typed outcome once.
- `FR-006-AC-25` (role: examined): The `quire-cli` composition starts and admits every configured provider before negotiation, and uses each admitted hello descriptor as the sole provider identity for subsequent selection and dispatch.
- `FR-006-AC-26` (role: examined): If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a QSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when QSL treats repeated identical admission as idempotent.
- `FR-006-AC-27` (role: examined): If a configured provider fails to start or admit its hello, the composition begins no negotiation, closes all earlier active sessions, and preserves the original typed host failure.
