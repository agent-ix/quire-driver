---
id: TC-016
title: "Settle one terminal record per item after replay"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-009
    type: verifies
---
# TC-016: Settle one terminal record per item after replay

## Description

Verify the available proved-provider terminal path uses QSL's canonical
`OutcomeItem` and `OutcomeDocument` serializer. Full replay-driven settlement
for every requested item remains planned.

Scope: FR-009-AC-1 through FR-009-AC-5.

## Test Procedure

Run a verified Kani result through `ProviderResults::outcome_bytes` and compare
the bytes with QSL's `OutcomeDocument::settled` over
`OutcomeItem::from_terminal`. Run an execute result through
`ExecutionResult::outcome_bytes` and compare it with QSL's
`OutcomeDocument::from_call` serialization. Verify that a falsified scalar
without replay remains a typed error and produces no guessed outcome.

The multi-item replay scenarios below remain planned until the driver has the
CG replay adapter and complete per-item settlement path.

Test location: `src/provider.rs` and `src/execute.rs`; full prove/replay
integration remains planned.

## Expected Results

- The available execute and proved-provider outcomes are serialized by QSL's
  `OutcomeDocument`; the driver defines no JSON document shape.
- A falsified scalar without QSL replay does not produce a terminal document.
- A counterexample is `refuted` only when replay reproduces it; parity
  disagreement is `inconclusive`, replay refusal keeps its cause, and replay
  fault is an internal failure.
- CG's mapping is used without a second driver mapping.
- The outcome has exactly one terminal record for every requested item,
  including unsupported and incomplete items.
