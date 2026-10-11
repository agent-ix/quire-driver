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
for every requested item, including the routed scalar observation producer
and report-claim guard, remains planned.

Scope: FR-009-AC-1 through FR-009-AC-8.

## Test Procedure

Run a verified Kani result through `ProviderResults::outcome_bytes` and compare
the bytes with QSL's `OutcomeDocument::settled` over
`OutcomeItem::from_terminal`. Verify that a falsified scalar without replay
remains a typed error and produces no guessed outcome.

The scalar producer/replay scenarios below remain planned until the driver has
the reviewed CG KaniForm API and the IR-610 producer/consumer path. The
qualification must retain the exact generated artifact/package, scalar
identity/occurrence, ordered operands, original `ScalarLimits`, and original
QSL claim/replay context as one execution context; it must execute and
authenticate that same artifact's actual typed native outcome before QSL
evaluation. It must reject missing or mismatched evidence without invoking QSL
or creating settlement, then verify `report.claim()` against the exact claim
sent before any CG conversion for every report outcome. A caller-supplied
outcome/digest echo or regenerated artifact is not a valid positive case.

Test location: `src/provider.rs`; full prove/replay integration remains
planned.

## Expected Results

- Available proved-provider outcomes are serialized by QSL's
  `OutcomeDocument`; the driver defines no JSON document shape.
- A falsified scalar without QSL replay does not produce a terminal document.
- A counterexample is `refuted` only when replay reproduces it; parity
  disagreement is `inconclusive`, replay refusal keeps its cause, and replay
  fault is an internal failure.
- CG's mapping is used without a second driver mapping.
- The outcome has exactly one terminal record for every requested item,
  including unsupported and incomplete items.
- For missing or mismatched artifact/context/native evidence, the upstream
  typed completion refusal is preserved and QSL evaluation, settlement, and
  terminal creation do not occur.
- A missing replay report or a report whose `claim()` differs from the exact
  claim sent does not reach CG conversion or terminal settlement; the upstream
  typed refusal is preserved without a new driver/CG/QSL mapping.
