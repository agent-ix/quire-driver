---
id: SR-14792
title: "IR-672 PR 93 spec-review base"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #93; spec/tests.md; spec/test-cases/TC-013-plugin-host-protocol-and-failures.md; spec/test-cases/TC-028-cli-package-boundary.md"
review_set: subset
---

## Summary

Reviewed whether the changed `spec/tests.md` coverage descriptions match the current tests and the linked test-case procedures. The matrix rows accurately describe the new partial evidence, but the test-case prose still says production routed dispatch remains planned under IR-672, contradicting the newly reported production test.

## Verdict

CONDITIONAL

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-006 and its test cases still describe production dispatch/config coverage as planned under IR-672 although the changed matrix reports the production tests. | spec/functional/FR-006-host-provider-plugins.md:195, spec/functional/FR-006-host-provider-plugins.md:202, spec/functional/FR-006-host-provider-plugins.md:218, spec/test-cases/TC-013-plugin-host-protocol-and-failures.md:36, spec/test-cases/TC-013-plugin-host-protocol-and-failures.md:140, spec/test-cases/TC-028-cli-package-boundary.md:66, spec/tests.md:152 |

## Coverage

Compared the changed FR-006/FR-014 coverage rows and TC-013/TC-028 summaries in `spec/tests.md` with the FR-006 verification cells, trace-tagged production test, and current test-case procedures. The rows in `spec/tests.md` call the production dispatch partial and list its remaining duplicate-identity and partial-start cleanup gaps. However, FR-006-AC-1, AC-8 and AC-24 still mark production dispatch/deadline/config validation planned; TC-013 procedure step 1 and its overview, and TC-028 procedure step 6 / expected results, still state end-to-end dispatch remains planned under IR-672. Update those stale statements to distinguish implemented dispatch from pending settlement and residual coverage.

Examined units:
- `spec/tests.md` FR-006-AC-1, AC-8, AC-15, AC-24..27 rows (examined): New test evidence and remaining coverage.
- `spec/tests.md` TC-013 and TC-028 rows (examined): Updated test summaries.
- `FR-006-AC-1`, `FR-006-AC-8`, and `FR-006-AC-24` (examined): Requirement verification cells still show planned status.
- `TC-013` (examined): The test procedure's statement of driver dispatch and current implementation status.
- `TC-028` (examined): The test procedure's statement of production configuration and routed dispatch status.

## Dispositions

Historical dispositions restored verbatim in meaning from the existing tracker records.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The narrow spec/tests.md edits do not resolve the original stale FR-006 verification cells and TC-013/TC-028 procedure statements. |
| FND-002 | still-open | Before the next wording correction, FR-001 said production Process dispatch remained planned under IR-672 and TC-012 said full process-provider invocation remained planned. |
| FND-002 | fixed | Configured Process-provider invocation is exercised under TC-013 and TC-028. Configured process-provider invocation is exercised under TC-013 and TC-028. |
| FND-001 | fixed | CG IR-629's typed process-backend mapping is available. Step 6 exercises the production `--provider-config` path and selected process-provider invocation: the configured-run composition test drives the selected route through the host adapter, and binary-process tests cover configuration parsing and the production entrypoint. Package-graph checks and adapter-fixture lifecycle checks provide separate evidence. Remaining limits are later-start/hello failure cleanup and production signal coverage for the starting and shutdown-waiting states; the two-provider selection assertion is currently at the configured-run composition boundary rather than one binary-process test. |

## Current revalidation

PASS — prior findings remain closed. The rebase keeps the merged object-environment parity evidence and removes only the contradictory trailing planned sentence in TC-011. FR-006 and TC-013/TC-028 consistently describe implemented configured dispatch, production SIGTERM coverage, and remaining settlement/lifecycle/projection gaps. Original status-truthfulness findings remain fixed. No requirement, AC identifier, dependency or behavioral obligation was introduced by the rebase delta.

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

## Gate-fix narrow revalidation

PASS: narrow gate-fix review. TC-006 and its test now identify the guard at request index 0 and the add at index 1 in this checked package. Pinned QSL items_from_requirements enumerates the BTreeMap of OccurrenceKey records; driver validates settlement request order. Both exact typed disposition oracles remain intact: integer.gt OperationNotRendered with the same derived integer range, and integer.add ResultBoundUnreachable with result [0,10] and reachable [100,109]. Neither item is routed/generated. The fix changes no production code, failure category, assertion strength or settlement behavior. No new finding. No builds/tests/gates run by reviewer.

Examined: `TC-006` step 10/expected result (tests and spec agree on ascending occurrence-key request order); `FR-001-AC-6` (generation only after routing, keyed by request index); `tests/drive.rs::tc_006_x_plus_100_in_a_dead_branch_settles_result_bound_unreachable` (exact typed refusals and empty routed/generated sets preserved).

## Approved lock-only revalidation

PASS: narrow approved lock-only delta. Only Cargo.lock changes: all nine QSL packages resolve 485ccaa3bd289ae94baa74e96a976adce99fd0d8 and quire-exact resolves 72b254219fd65a058b8cf93cefadda1c4c249fca. CG 2c1db3e3, IR d0737774, FCD ac62202a, quire-rs fd1cdd9d, QSV 1a38c06f, runtime 3cc88c48 and ix-trace-rs 9378a677 are retained. TOML inspection verifies one revision per first-party repository. No source/spec delta; prior findings remain fixed. Scope is the explicitly approved QSL/exact update, with no claim that an unpublished future CG/FCD cohort has landed. No builds/tests/gates run by reviewer.

Examined: Cargo.lock source identities and package records; prior source/spec dispositions remain valid because neither source nor spec changed.
