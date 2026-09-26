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

Verify that the driver calls compile, the E5 read, the backend conversion,
negotiation, routing and generation in that order, against the real QSL and CG functions.

Scope: FR-001-AC-4.

## Test Procedure

1. Build a registry holding `kani`, advertising `value-validity` bounded.
2. Drive a complete-V1 unit with three items (see TC-003) and a Kani
   context (see TC-006) through a recorder that logs each upstream call
   and then makes it.

Test: `tests/drive.rs`, `tc_001_tc_003_tc_006_steps_run_in_order_through_generation`.

## Expected Results

- The call log is: compile, the E5 read, the conversion of `kani`, one
  negotiation of three items, one routing call, one generation call over
  one item. (The candidate step calls no upstream
  function the recorder sees.)
- The emitted v2 bytes are not empty.
