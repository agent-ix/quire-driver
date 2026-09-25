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

Verify that the driver calls compile, the backend conversion, negotiation
and routing in that order, against the real QSL and CG functions.

Scope: FR-001-AC-4.

## Test Procedure

1. Build a registry holding `kani`, advertising `value-validity` bounded.
2. Drive a complete-V1 unit with three items (see TC-003) through a
   recorder that logs each upstream call and then makes it.

Test: `tests/drive.rs`, `tc_001_tc_003_steps_run_in_order_and_route_only_supported_items`.

## Expected Results

- The call log is: compile, the conversion of `kani`, one negotiation of
  three items, one routing call.
- The emitted v2 bytes are not empty.
