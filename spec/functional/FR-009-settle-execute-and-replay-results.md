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
request and replay each counterexample through QSL's replay operation. The
driver SHALL produce exactly one FR-331 terminal record for each requested
item. The driver SHALL preserve the independent result and cause from
execution and replay when settling the item.

## Inputs

- The checked and emitted package from the current request.
- The request items, provider results, generated artifacts and cancellation
  handle.
- The QSL replay result for each provider counterexample.

## Outputs

- One FR-331 terminal record per requested item, written in the
  `quire-outcome/1` outcome document.

## Behavior

- The driver SHALL pass the generated request and its exact emitted package
  to the selected provider.
- The driver SHALL preserve the provider's typed result.
- When QSL replay reproduces a provider counterexample, the driver SHALL
  settle that item as `refuted`.
- When replay disagrees or completes without reproducing the counterexample,
  the driver SHALL settle that item as `inconclusive` with the corresponding
  replay-parity cause.
- When QSL replay refuses, the driver SHALL preserve its typed replay cause.
- When QSL replay faults, the driver SHALL settle the item as a failed
  internal result and SHALL NOT report a reproduced counterexample.
- CG's FR-331 terminal mapping SHALL combine the Kani outcome and replay
  result.
- The driver SHALL NOT define a second terminal mapping.
- The driver SHALL write one terminal record per requested item, including
  unsupported and incomplete items.
- The driver SHALL NOT duplicate or omit an item record.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | The proof provider receives the generated request and the exact checked-package bytes emitted by the same request. | Test |
| FR-009-AC-2 | A provider counterexample settles `refuted` only when QSL replay reproduces it; a disagreement or completed non-reproduction settles `inconclusive` with a replay-parity cause. | Test |
| FR-009-AC-3 | A replay refusal retains its typed cause and a replay fault is an internal failure; neither is converted to `refuted`. | Test |
| FR-009-AC-4 | The terminal result is composed by CG's FR-331 mapping from both the provider result and replay result; the driver introduces no second mapping. | Partial (TC-016 proves QSL serialization for an available CG terminal; replay-combined settlement remains planned) |
| FR-009-AC-5 | The outcome contains exactly one FR-331 terminal record for every requested item, including unsupported and incomplete items. | Partial (TC-016 serializes available provider terminals through QSL; full per-item replay accounting remains planned) |

## Dependencies

- QSL ADR-029 LC-1, CB-4 and library operation `prove`; ADR-013 O-16 and
  C-09; CG's terminal-record mapping.
- [TC-016](../test-cases/TC-016-execute-replay-and-terminal-records.md).
