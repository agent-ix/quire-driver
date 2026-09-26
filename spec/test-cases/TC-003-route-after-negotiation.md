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

The requested items are not caller input: they are written from the
compiled unit's own checked requirement records
(`CheckedGraph::requirements()`, QSL-266), one per scalar operation
application, so every item in a unit settles the same capability kind,
`value-validity`. Building a mixed settlement set therefore rewrites CG's
real negotiation answer through the test's own upstream seam.

1. With `kani` registered for `value-validity`, drive
   `function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }`, which
   writes one requirement record (the `-`); it settles `supported`, and
   `route` is called once, after the negotiation, with one disposition,
   routed to `kani`, kind Kani.
2. With `kani` registered for `value-validity`, drive a request naming an
   unregistered backend (`missing`); the one item settles `invalid-request`
   (`unknown-backend`, naming `missing`), and `route` is never called.
3. With `kani` registered for `operation-contract` only, drive the same
   unit as step 1; its `value-validity` item settles `unsupported`, and
   `route` is never called.
4. With `kani` registered for `value-validity`, drive a unit declaring an
   integer `=` alongside the unary `-` of step 1 (see TC-006 step 6's
   `EQ_UNIT`), which writes two requirement records (the `=` at index 0,
   the `-` at index 1), rewriting the negotiation answer so item 0 (the
   `=`) settles `unsupported`, leaving only item 1 (the `-`) `supported`.
5. Drive `function f using v(): Boolean pure { true }`, which writes no
   requirement record.

Tests: `tests/drive.rs`,
`tc_001_tc_006_steps_run_in_order_through_generation` (step 1),
`tc_001_a_named_unregistered_backend_settles_invalid_request` (step 2),
`tc_003_no_supported_item_means_no_route_call` (step 3),
`tc_003_route_and_generate_skip_a_negotiation_unsupported_item_by_request_index`
(step 4) and `tc_003_a_unit_with_no_scalar_applications_routes_nothing`
(step 5).

## Expected Results

- Step 4: `route` and CG generation each see one item, the `-` at request
  index 1 -- not at list position 0, which the unsupported item 0 leaves
  it as. `route` is called with one disposition (`supported: 1,
  dispositions: 1`), and generation is called over one item. Both the
  routed and the generated item carry request index 1, not 0: keying by
  list position instead of `settlement.request_index` would put 0 there
  instead, and fail this test.
- Step 5: `route` is never called, generation is never called, and the run
  still completes (an empty request negotiates zero items).
