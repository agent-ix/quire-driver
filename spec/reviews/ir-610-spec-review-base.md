---
id: SR-14844
title: base review of IR-610 E9 function producer
type: SpecReview
analysis: base
scope: agent-ix/quire-driver; spec/functional/FR-001-orchestrate-the-driver-steps.md;
  spec/functional/FR-009-settle-execute-and-replay-results.md; spec/test-cases/TC-016-execute-replay-and-terminal-records.md;
  spec/tests.md; ticket IR-610
review_set: subset
---
# SR-14844: base review

## Summary

Reviewed PR #107 frozen head b9600ff65d138053915329e6a0735cc821f46ef2. No applicable AssuranceProfile exists in this repository. The current-V2 CG harness dependency is explicitly Planned.

## Scope

- examined: FR-001-AC-23 — Two generated function items that select the same checked node at different application occurrences retain two distinct contexts keyed by their original request indices. Each context contains the QSL-selected function declaration and exact occurrence, its producing source and dependency lock, the same E5-admitted package identity, and the matching CG `KaniObligationIdentity`; CG `ReplayInputs` use those retained source, lock, accounting, stage, domain and replay inputs; playback intake uses the smaller caller/proving-run byte and value decode limits. No identity or source is reconstructed from
- examined: FR-001-AC-24 — A QSL selection, IR admission or CG generation refusal is preserved as that owner’s typed refusal. After all three succeed, a missing, duplicate or mismatched request-index, occurrence, package, declaration or generated-identity association returns a driver `runtime_invariant` fault before provider execution or replay, without selecting another item or fallback context.
- examined: FR-009-AC-20 — For a falsified function item with a complete valid transcript at the byte and value ceilings, the driver calls CG `parse_assertion_playback` once on exactly the retained `KaniRunOutcome::Falsified.counterexample`, with the selected item's retained source identity and context and the smaller caller/proving-run ceilings in each dimension. It passes the resulting typed `Playback`, exact proving-run identity and same admitted package to CG function replay. A reproducing replay remains subject to FR-009-AC-2 and CG terminal mapping.
- examined: FR-009-AC-21 — A full transcript that exceeds the byte ceiling, has more values than the value ceiling, or has no unique valid assertion yields CG's corresponding typed `PlaybackRefusal` from that single parse, even if a shortened assertion block would fit. The driver neither uses optional diagnostics nor invokes function replay, creates settlement, or creates a terminal for the refused item.
- examined: FR-009-AC-22 — A typed upstream refusal before playback intake is preserved before any parse, settlement, or terminal creation. A parser refusal is preserved without replay settlement or terminal creation and is not recast as replay parity or a driver join fault. After a successful parse, a CG typed identity, decode, or replay refusal or evidence failure follows CG's `ReplaySettlement` conversion and `run_terminal_value` mapping with the original execution evidence; the driver preserves the resulting typed terminal and never settles that result as `refuted`.
- examined: FR-009-AC-23 — For two function proof items sharing a checked node but naming distinct exact application occurrences, the driver invokes the selected provider once per matching generated item, retains each typed result with its own request index and CG `KaniObligationIdentity`, and supplies the corresponding retained source, dependency lock, admitted package, accounting/stage/domain/replay inputs and smaller caller/proving-run decode limits to function playback and replay. Neither result or context is selected by node alone.
- examined: FR-009-AC-24 — A typed provider execution refusal is preserved before playback intake. If a provider result is successful but the driver detects that its request index, occurrence, package or CG proving-run identity differs from the retained selected context, it returns a driver `runtime_invariant` fault before playback intake, replay, settlement or terminal creation; no other item’s context is substituted.
- examined: TC-016:function-proof-producer — The function proof producer and join scenarios remain planned until IR-1121 supplies CG’s current-V2 generated function harness and proving-run identity.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Successful provider-result identity comparison has no observable carrier | spec/functional/FR-009-settle-execute-and-replay-results.md:273 |

## Verdict

CONDITIONAL: clarify the post-execution result identity oracle.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-009-AC-24 now checks the one-item ProviderRequest, private invocation slot, observable result index/backend and evidence fields; TC-016 mutates those actual fields. |
