---
id: TC-003
title: "route runs after negotiation, for supported items only"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-003: route runs after negotiation, for supported items only

## Description

Verify that routing follows negotiation and sees only `supported` items.

Scope: FR-001-AC-3.

## Test Procedure

1. With `kani` registered for `value-validity`, drive three bounded items:
   `value-validity` naming no backend, `operation-contract` naming no
   backend, and `value-validity` naming `missing`.
2. Drive one `operation-contract` item alone.

Tests: `tests/drive.rs`,
`tc_001_tc_003_tc_006_steps_run_in_order_through_generation` and
`tc_003_no_supported_item_means_no_route_call`.

## Expected Results

- Step 1 settles `supported`, `unsupported` and `invalid-request`
  (`unknown-backend`, naming `missing`). `route` is called once, after the
  negotiation, with one disposition, and item 0 is routed to `kani`, kind
  Kani.
- Step 2 settles `unsupported`, and `route` is never called.
