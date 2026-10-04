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

Verify that provider execution and QSL replay feed one authoritative terminal
record per requested item.

Scope: FR-009-AC-1 through FR-009-AC-5.

## Test Procedure

Run one multi-item proof request with a checked package emitted during that
request. Include successful and unsupported items, a Kani counterexample
reproduced by replay, a counterexample that fails replay parity, a replay
refusal, a replay fault and a budget-incomplete item. Compare each terminal
record with CG's FR-331 mapping of the paired provider and replay results.

Test location: planned prove/replay integration tests.

## Expected Results

- The provider receives the emitted package from the same request.
- A counterexample is `refuted` only when replay reproduces it; parity
  disagreement is `inconclusive`, replay refusal keeps its cause, and replay
  fault is an internal failure.
- CG's mapping is used without a second driver mapping.
- The outcome has exactly one terminal record for every requested item,
  including unsupported and incomplete items.
