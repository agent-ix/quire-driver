---
id: SR-14794
title: "IR-672 PR 93 evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-driver; FR-006, TC-013, TC-028, spec/tests.md; scoped final-delta review"
review_set: subset
---

## Summary

Ticket: IR-672. Ran quoin advise --repo . --json and inspected scoped FR-006 recommendations: no mismatch or inconclusive recommendation. Production configuration, selected invocation, duplicate identity and active-request SIGTERM checks provide test evidence; broader lifecycle, complete bounds and settlement remain partial/planned. Test bindings and assertion scope were read statically; no test execution or aggregate qualification is claimed.

## Verdict

PASS for the examined change; final aggregate qualification remains the code lead's gate.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

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

TC-013 and TC-028 (role: examined): production configured invocation coverage and explicitly pending settlement/lifecycle evidence.

spec/tests.md TC-011 (role: examined): preserves direct-QSL object-context parity evidence and cross-engine partial status after rebase.

No applicable AssuranceProfile was found. No source edits, builds, tests or aggregate gates were run.
