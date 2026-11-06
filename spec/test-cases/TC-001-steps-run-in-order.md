---
id: TC-001
title: "The steps run in ADR-011 T-13 order"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-001: The steps run in ADR-011 T-13 order

## Description

Verify that the driver calls compile, the E5 read, the Linked-backend
conversion, negotiation, routing and generation in that order, against the
real QSL and CG functions.

Scope: FR-001-AC-4 and FR-001-AC-25.

## Test Procedure

1. Build a registry holding `kani`, advertising `value-validity` bounded.
2. Drive `function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }`,
   which writes one requirement record (see TC-003), with a Kani context
   (see TC-006), through a recorder that logs each upstream call and then
   makes it.
3. Drive a fixture with a caller-selected `emit.output_bytes` ceiling equal to
   its actual canonical E4 byte length and confirm E5 admits it. Lower the
   ceiling by one byte and confirm QSL refuses at E4 before E5. Inspect the
   E5 call to `admission_evidence` to confirm it receives that same caller
   `SpineLimits.package.emit()` value.

Test: `tests/drive.rs`, `tc_001_tc_006_steps_run_in_order_through_generation`.

## Expected Results

- The call log is: compile, the E5 read, the conversion of Linked `kani`, one
  negotiation of one item, one routing call, one generation call over
  one item. (The candidate step calls no upstream
  function the recorder sees.)
- The emitted v2 bytes are not empty.
- The caller-selected exact byte ceiling admits the emitted package at E5;
  a lower ceiling refuses emission and never reaches E5. The E5 evidence
  call uses the same caller-owned package limit as E4.
