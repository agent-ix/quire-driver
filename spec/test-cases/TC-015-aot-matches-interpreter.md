---
id: TC-015
title: "AOT and interpreter return equal evaluations"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-008
    type: verifies
---
# TC-015: AOT and interpreter return equal evaluations

## Description

Verify that the AOT backend accepts only the current checked package and
returns the interpreter's result for the same checked entry and accounting
limits.

Scope: FR-008-AC-1 through FR-008-AC-5.

## Test Procedure

Run a parity corpus through the interpreter and AOT, including completed,
undefined, refused and incomplete evaluations, source-location and loss
records, an accounting-limit boundary, and an entry the AOT backend cannot
prepare. Build with a named toolchain. Attempt to provide unchecked bytes and
make the selected executable unavailable after preparation.

Test location: planned AOT conformance tests.

## Expected Results

- AOT lowers and builds from the checked package produced for the request and
  communicates through the canonical frame protocol.
- The AOT and interpreter results match in outcome, typed cause, location,
  loss records and accounting limit.
- Unchecked bytes are rejected. A limit remains a typed limit and an
  unsupported entry remains unsupported; neither silently falls back.
- The generated executable performs no semantic I/O or ambient-input reads.
