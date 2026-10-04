---
id: TC-014
title: "Pass manifest content to the QSL cache"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-007
    type: verifies
---
# TC-014: Pass manifest content to the QSL cache

## Description

Verify the driver's cache call includes the selected provider's manifest
bytes as content input, independently of any recorded revision or version
pin.

Scope: FR-007-AC-1 through FR-007-AC-2.

## Test Procedure

Use a recording cache interface with one eligible typed request and a
provider manifest. Invoke the driver once, then change one manifest byte
without changing the provider's recorded revision or version pin and invoke
it again. Inspect the inputs received by the cache interface.

Test location: planned driver cache-boundary tests.

## Expected Results

- Each cache invocation receives the typed request and exact manifest bytes
  selected for that invocation.
- The second invocation receives the changed bytes while the recorded pin is
  unchanged; the driver has not replaced manifest content with pin metadata.
- Cache key construction, miss/hit behavior, validation and storage follow
  QSL ADR-029 CA-1 through CA-5 and are verified by QSL-owned tests.
