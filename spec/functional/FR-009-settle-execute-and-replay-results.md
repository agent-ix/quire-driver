---
id: FR-009
title: "Settle executions after replaying counterexamples"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-009: Settle executions after replaying counterexamples

## Description

When the caller requests a proof, the driver SHALL run the generated provider
request and replay each counterexample through QSL's replay operation. For each
proof request that reaches per-item settlement, the driver SHALL produce
exactly one FR-331 terminal record for each requested item. A request-level
typed refusal before per-item settlement does not receive a per-item
terminal-record guarantee. The driver SHALL preserve the independent result
and cause from execution and replay when settling an item. For routed scalar
replay, the driver SHALL authenticate the native observation against the exact
generated artifact that was proved and SHALL preserve the original claim and
replay context through QSL evaluation and report handling.

## Inputs

- The checked and emitted package from the current request.
- The request items, provider results, generated artifacts and cancellation
  handle.
- The QSL replay result for each provider counterexample.
- For a routed scalar replay, one retained context containing the exact
  generated artifact and emitted package, scalar identity and occurrence,
  ordered operands, original `ScalarLimits`, and the original QSL claim and
  replay context.
- The actual typed native outcome produced by executing that retained artifact
  for the retained scalar inputs, and QSL's report for the original claim.

## Outputs

- One FR-331 terminal record per requested item, written in the
  `quire-outcome/1` outcome document, when the request reaches per-item
  settlement.
- The upstream typed refusal without a per-item terminal record when the
  request is refused before per-item settlement.

## Behavior

- The driver SHALL pass the generated request and its exact emitted package
  to the selected provider.
- The driver SHALL preserve the provider's typed result.
- For predicate/function counterexamples, when QSL replay reproduces the
  counterexample, the driver SHALL settle that item as `refuted`.
- For predicate/function counterexamples, when replay disagrees or completes
  without reproducing the counterexample, the driver SHALL settle that item as
  `inconclusive` with the corresponding replay-parity cause.
- The driver SHALL preserve QSL FR-357's typed scalar operator-parity report
  and apply CG FR-032's typed settlement conversion; `Agrees`, `Diverged`,
  `GeneratedFault`, `RefusedInput`, `Incomplete`, and other typed refusals
  retain the meanings those contracts define.
- The driver SHALL NOT settle a scalar operator-parity result as `refuted`.
- When QSL replay refuses, the driver SHALL preserve its typed replay cause.
- When QSL replay faults, the driver SHALL settle the item as a failed
  internal result and SHALL NOT report a reproduced counterexample.
- For a routed scalar replay, the driver SHALL retain one context that binds
  the exact generated artifact and emitted package to the scalar identity,
  occurrence, ordered operands, original `ScalarLimits`, and the exact claim
  and replay context sent to QSL. It SHALL NOT replace any member with a
  caller-supplied classification or a regenerated artifact.
- The driver SHALL execute that same retained artifact with the retained
  scalar inputs and original limits, retain its actual typed native outcome,
  and verify QSL's observation digest against that outcome while binding the
  observation to the artifact's canonical proof-content identity before QSL
  evaluates the observation.
- If the required retained context or an authentic observation is absent or
  mismatched, the driver SHALL preserve the upstream typed completion refusal
  and SHALL NOT invoke QSL replay evaluation or create a replay settlement or
  terminal record. The exact refusal variant remains governed by the upstream
  typed API; this requirement allocates no new refusal mapping.
- Before using a QSL replay report for CG conversion or terminal settlement,
  the driver SHALL compare `report.claim()` with the exact original claim sent
  to QSL for every report outcome. If the report is absent or its claim differs,
  the driver SHALL NOT invoke CG conversion or create a settlement or terminal
  record, and SHALL preserve any upstream typed refusal without inventing a
  driver, CG, or QSL mapping.
- CG's FR-331 terminal mapping SHALL combine the Kani outcome and replay
  result.
- The driver SHALL NOT define a second terminal mapping.
- When a proof request reaches per-item settlement, the driver SHALL write
  one terminal record per requested item, including unsupported and incomplete
  items.
- For a request that reaches per-item settlement, the driver SHALL NOT
  duplicate or omit an item record.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | The proof provider receives the generated request and the exact checked-package bytes emitted by the same request. | Test |
| FR-009-AC-2 | Predicate/function replay that reproduces the provider counterexample settles as `refuted`. | Test |
| FR-009-AC-3 | A replay refusal retains its typed cause and a replay fault is an internal failure; neither is converted to `refuted`. | Test |
| FR-009-AC-4 | The terminal result is composed by CG's FR-331 mapping from both the provider result and replay result; the driver introduces no second mapping. | Partial (TC-016 proves QSL serialization for an available CG terminal; replay-combined settlement remains planned) |
| FR-009-AC-5 | When a proof request reaches per-item settlement, the outcome contains exactly one FR-331 terminal record for every requested item, including unsupported and incomplete items; a request-level typed refusal before per-item settlement is governed by its refusal contract and has no per-item terminal-record guarantee. | Partial (TC-016 serializes available provider terminals through QSL; full per-item replay accounting remains planned) |
| FR-009-AC-6 | For routed scalar replay, the driver retains one context binding the exact proved generated artifact and emitted package to the scalar identity, occurrence, ordered operands, original `ScalarLimits`, and exact QSL claim and replay context; it executes that artifact with those inputs and limits and retains the actual typed native outcome. | Planned (TC-016; IR-610 producer slice) |
| FR-009-AC-7 | The driver verifies QSL's native-observation digest against the actual typed outcome it obtained by executing the exact proved artifact with the retained operands and original limits, and binds that observation to the artifact's canonical proof-content identity before QSL evaluates it; missing or mismatched context or observation preserves the upstream typed completion refusal and produces no QSL replay evaluation, settlement, or terminal record. | Planned (TC-016; IR-610 producer slice and reviewed CG API) |
| FR-009-AC-8 | Before acting on any QSL replay report, the driver compares `report.claim()` with the exact claim sent for that replay; an absent report or claim mismatch produces no CG conversion, settlement, or terminal record and preserves the upstream typed refusal without adding a driver, CG, or QSL mapping. | Planned (TC-016; IR-610 producer/consumer slice and reviewed CG API) |
| FR-009-AC-9 | Each scalar operator-parity report outcome is converted only through CG FR-032's typed settlement conversion of the QSL FR-357 report; `Agrees`, `Diverged`, `GeneratedFault`, `RefusedInput`, `Incomplete`, and other typed refusal outcomes retain the meanings those contracts define. | Planned (TC-016; IR-610 consumer slice and reviewed CG API) |
| FR-009-AC-10 | Predicate/function replay that disagrees or completes without reproducing a provider counterexample settles as `inconclusive` with its replay-parity cause. | Planned (TC-016; full replay path) |
| FR-009-AC-11 | No scalar operator-parity report settles as `refuted`. | Planned (TC-016; IR-610 consumer slice and reviewed CG API) |

## Dependencies

- QSL ADR-029 LC-1, CB-4 and library operation `prove`; ADR-013 O-16 and
  C-09; QSL FR-357 observation-digest and report-claim contract; CG FR-032
  native-observation and scalar-replay binding contract; CG's terminal-record
  mapping. The driver producer and consumer API remains planned until the
  reviewed CG KaniForm API is available.
- [TC-016](../test-cases/TC-016-execute-replay-and-terminal-records.md).
