---
id: SR-14841
title: EARS analysis of IR-1106 function playback intake
type: SpecReview
analysis: ears-conformance
scope: agent-ix/quire-driver; spec/functional/FR-009-settle-execute-and-replay-results.md;
  spec/test-cases/TC-016-execute-replay-and-terminal-records.md; ticket IR-1106
review_set: subset
---
# SR-14841: EARS analysis

## Summary

Reviewed frozen PR #106 head 61362388da62f9393043c5ca94588b452f6378e1. No applicable AssuranceProfile was found under this repository's spec tree.

## Scope

- examined: FR-009-AC-20 — For a falsified function item with a complete valid transcript at the byte and value ceilings, the driver calls CG `parse_assertion_playback` once on exactly the retained `KaniRunOutcome::Falsified.counterexample`, with the selected item's retained source identity and context and the smaller caller/proving-run ceilings in each dimension. It passes the resulting typed `Playback`, exact proving-run identity and same admitted package to CG function replay. A reproducing replay remains subject to FR-009-AC-2 and CG terminal mapping.
- examined: FR-009-AC-21 — A full transcript that exceeds the byte ceiling, has more values than the value ceiling, or has no unique valid assertion yields CG's corresponding typed `PlaybackRefusal` from that single parse, even if a shortened assertion block would fit. The driver neither uses optional diagnostics nor invokes function replay, creates settlement, or creates a terminal for the refused item.
- examined: FR-009-AC-22 — A typed upstream refusal before playback intake is preserved before any parse; a successful parse followed by CG's typed identity, decode, or replay refusal is preserved under CG's contract and cannot settle as `refuted`. A parser refusal is not recast as replay parity or a driver join fault. No refused path creates a replay settlement or terminal for the item.
- examined: TC-016:function-playback-intake — The function playback intake scenarios remain planned until CG PR #621's typed
`Playback` entry is available to the driver and IR-610 wires E9 production
replay. Drive a falsified function item through the production provider result
with its complete retained `KaniRunOutcome::Falsified.counterexample`, exact
selected proving-run identity and admitted package. Set finite, distinct
caller and proving-run byte and value ceilings; record the parser call and CG
function replay invocation. The oracle is one
`parse_assertion_playback` call over the exact retained transcript with the
selected item's r
- context_only: FR-009:behavior — If parsing succeeds but CG function replay returns a typed refusal or evidence failure, the driver SHALL preserve that result under CG's replay and terminal contracts without treating it as reproduction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS within the changed spec scope.
