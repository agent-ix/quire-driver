---
id: SR-14791
title: "IR-672 PR 93 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver PR #93; FR-006-AC-1..27; spec/tests.md; quire-cli/src/cli.rs; quire-cli/src/process_provider.rs; quire-cli/src/provider_config.rs"
review_set: subset
---

## Summary

Planless, diff-focused manual gap analysis of the configured process-provider path, its FR-006 trace bindings, and the updated test matrix. `quoin gap-analysis` is not an installed command in the resolved Quoin CLI; the static matrix was computed with `quire matrix`. It shows intentional unclaimed settlement/lifecycle criteria as well as an untagged duplicate-identity case that the production code does not fail closed on.

## Verdict

FAIL

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-006-AC-26 has no trace-tagged test, and production proceeds when duplicate sessions have identical idempotent admissions, then dispatches to both. | spec/functional/FR-006-host-provider-plugins.md:220, spec/tests.md:152, quire-cli/src/cli.rs:881-887 |
| FND-002 | high | FR-006-AC-4 is untagged and the production command path does not activate signal handling, so a handled signal can bypass session cleanup. | spec/functional/FR-006-host-provider-plugins.md:196, quire-cli/src/cli.rs:1262-1267 |

## Coverage

Computed the static matrix with `quire matrix --scope . --format json`. FR-006-AC-1, AC-8, AC-10..11, AC-13, AC-15 and AC-24 are tagged by tests; AC-2..4, AC-9, AC-12, AC-14, AC-16..23 and AC-25..27 are untagged. Many untagged items concern explicitly pending settlement and lifecycle work; this review does not classify them as defects solely for being untagged. AC-4 and AC-26 intersect production behavior in this diff and remain both untested and violated. Plan completion: not assessed.

Manual acceptance-criteria-to-tests check was used because `quoin gap-analysis` is unavailable. New tests substantiate strict JSON refusal, one selected-provider dispatch, selected deadline forwarding, typed result retention and diagnostic retention. They do not exercise duplicate identities, later-start cleanup, omitted-config process behavior, or signal cleanup. The changed `spec/tests.md` describes these residual gaps as partial/planned. No test or full gate was run.

Examined units:
- `FR-006-AC-1` (examined): Dispatch only the negotiated items for the selected process provider.
- `FR-006-AC-8` (examined): The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged.
- `FR-006-AC-15` (examined): Retain typed results without fabricating QSL terminals.
- `FR-006-AC-24` (examined): Reject invalid strict configuration before starting a provider.
- `FR-006-AC-25` (examined): Start and admit every provider before negotiation.
- `FR-006-AC-26` (examined): Duplicate admitted identities stop before negotiation and close all sessions, with InvalidRequest on idempotent QSL admission.
- `FR-006-AC-27` (examined): A later start/hello failure stops negotiation, closes earlier sessions and preserves the typed host failure.
- `FR-006-AC-4` (examined): Handled SIGINT/SIGTERM cancels and cleans up active sessions before restoring and re-raising the signal.
- `TC-013` (context_only): Driver/provider request, dispatch and settlement test coverage.
- `TC-028` (context_only): CLI configuration, production dispatch, duplicate admission, cleanup and lifecycle coverage.
- `spec/tests.md` FR-006 rows and TC-013/TC-028 summaries (examined): Reported evidence and remaining test gaps.

## Evidence

The computed matrix has no binder for FR-006-AC-26. The available production path calls `register_admitted` without distinguishing multiple sessions that report one identical descriptor; the registry deliberately treats identical registrations as idempotent. Later, the dispatch loop visits each session and filters only by the repeated identity, causing both to receive the selected request. The two-provider test uses distinct identities and therefore cannot detect this case.

## Dispositions

Historical dispositions restored verbatim in meaning from the existing tracker records.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | let (cleanup_diagnostics, cleanup_failures) = close_configured_providers(active); return Err(Box::new(ConfiguredRunFailure {     call: Some(call),     document,     kind: ConfiguredRunErrorKind::DuplicateIdentity { backend },     cleanup_diagnostics,     cleanup_failures, })); |
| FND-002 | fixed | let configured = crate::signals::run(move \|signals\| {     run_configured_providers(ConfiguredRunRequest {         path: &options.path,         bytes: &bytes,         source,         limits,         call,         packages,         dependencies,         config: provider_config,         signals,     }) }); |

## Current revalidation

PASS — prior findings remain closed. Scoped matrix regenerated with quire matrix --scope . --format json. AC-4 and AC-26 now have production test bindings; selected-provider dispatch, deadline, typed unmapped results and duplicate-session cleanup have concrete assertions. AC-2/14..23 settlement and AC-27 later-start cleanup remain explicitly planned; AC-3/4/12/13 evidence remains partial. This is scoped delta assurance, not repository-wide completeness. Plan completion: not assessed. No builds/tests were run.

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
