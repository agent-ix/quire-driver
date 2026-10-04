---
id: TC-015
title: "Invoke AOT with current checked request inputs"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-008
    type: verifies
---
# TC-015: Invoke AOT with current checked request inputs

## Description

Verify that the driver dispatches an AOT-selected request to QSL's AOT
backend with the checked package and settings from that request and preserves
the backend's typed result.

Scope: FR-008-AC-1 through FR-008-AC-2.

## Test Procedure

Use a recording AOT backend and submit a request with a checked package,
selected entries, accounting and preparation limits, cancellation handle and
named toolchain. Have the backend return a typed evaluation. Repeat with a
typed preparation failure, and with the prepared executable unavailable
before execution. In the last case the backend shall return a typed fault.
Record all backend calls.

Test location: planned driver/AOT boundary tests.

## Expected Results

- The AOT backend receives only the checked package and entries produced for
  the current request, with its exact limits, cancellation handle and named
  toolchain.
- The driver's result equals the backend's typed evaluation or typed
  preparation failure, including the typed fault when the prepared
  executable is unavailable.
- The driver makes no interpreter or alternate-engine call after either
  failure.
- AOT/interpreter semantic parity is verified by the QSL ADR-029 EB-5
  conformance evidence, not re-specified by this driver-boundary test.
