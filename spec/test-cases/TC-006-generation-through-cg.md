---
id: TC-006
title: "E5 admits the emitted package and CG generates for the routed items"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-006: E5 admits the emitted package and CG generates for the routed items

## Description

Verify the E5 hand-off and step 7 against the real QSL, IR and CG
functions. Kani is not run; generation only produces its output.

Scope: FR-001-AC-6, FR-001-AC-7.

## Test Procedure

1. Drive `function inc using v(x: Int[0, 9]): Int[0, 10] pure { x + 1 }`
   with no items, and take its admitted package. Build the FR-014 claim map
   of its `binary` (`+`) node with CG `generate_exact_scalar_oracles`, over
   the parameter's own `[0, 9]` integer domain.
2. With `kani` registered for `value-validity`, drive the TC-003 items,
   the first naming the `+` node, with a Kani context over that claim map.
3. Call CG `generate_routed` directly over the same package with the
   routed item (index 0, the `+` node, the `kani` wire pair, kind Kani) and
   the same context with `KaniToolPins::pinned()`.
4. Drive three items: the `conversion` node, an unsupported item and the
   `+` node.
5. Drive a unit with a `Text[1, 5; nfc]` parameter and one with a
   `Rational[0, 9; 1, 9]` parameter, with no items.

Tests: `tests/drive.rs`,
`tc_001_tc_003_tc_006_steps_run_in_order_through_generation` and
`tc_006_generation_output_ascends_by_request_index` and
`tc_006_e5_admits_text_and_rational_units`.

## Expected Results

- The admitted package's `package_id` digest is the emitted `package_id`,
  and it equals step 1's package.
- The driver's generation output equals step 3's, with one item at request
  index 0 and the `kani` wire pair. Its record is `Supported` with a scalar
  harness: QSL-261 is resolved by IR-296 and IR-297. A unit with two bounded
  parameters still settles `AmbiguousBound` (IR-298, IR-299), a known limit
  this test case does not exercise.
- Step 4 routes indexes 0 and 2, generates once over two items, and the
  output's request indexes are `[0, 2]`.
- Step 5: E5 admits both packages at their emitted `package_id`. A float
  profile unit is not covered: no float parameter form this QSL revision
  admits was found.
