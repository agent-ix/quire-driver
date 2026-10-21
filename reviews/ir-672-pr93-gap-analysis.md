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
