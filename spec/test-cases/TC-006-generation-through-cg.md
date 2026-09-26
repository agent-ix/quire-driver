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

Scope: FR-001-AC-6, FR-001-AC-7, FR-001-AC-8.

## Test Procedure

1. Drive `function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }`
   with no items, and take its admitted package.
2. With `kani` registered for `value-validity`, drive the TC-003 items,
   the first naming the unary `-` node, with a Kani context carrying no
   claim map (IR-294: CG derives it from the package and the routed node
   alone).
3. Call CG `generate_routed` directly over the same package with the
   routed item (index 0, the `-` node, the `kani` wire pair, kind Kani) and
   the same context with `KaniToolPins::pinned()`.
4. Drive three items over `function inc using v(x: Int[0, 9]): Int[0, 10]
   pure { x + 1 }`: the `conversion` node, an unsupported item and the `+`
   node.
5. Drive a unit with a `Text[1, 5; nfc]` parameter and one with a
   `Rational[0, 9; 1, 9]` parameter, with no items.
6. Drive a unit with an integer `=` (`quire.op.integer.eq`, a node CG's
   derivation names no operator family for) alongside an integer unary `-`
   (`quire.op.integer.negate`, which it does derive), both routed to
   `kani`.
7. Drive the `+` node of step 4's unit alone, routed to `kani`.
8. Drive `function add using v(x: Int[0, 9], y: Int[10, 20]): Int[10, 29]
   pure { x + y }`, routing the `+` node to `kani`.

Tests: `tests/drive.rs`,
`tc_001_tc_003_tc_006_steps_run_in_order_through_generation`,
`tc_006_generation_output_ascends_by_request_index`,
`tc_006_e5_admits_text_and_rational_units`,
`tc_006_an_underivable_node_settles_unsupported_and_its_sibling_is_unaffected`,
`tc_006_x_plus_1_generation_passes_through_unchanged`,
`tc_006_x_plus_y_generation_passes_through_unchanged`,
`tc_006_x_plus_1_settles_supported` (ignored, IR-302) and
`tc_006_x_plus_y_settles_supported_with_independent_argument_bounds`
(ignored, IR-302).

## Expected Results

- The admitted package's `package_id` digest is the emitted `package_id`,
  and it equals step 1's package.
- The driver's generation output equals step 3's, with one item at request
  index 0 and the `kani` wire pair. Its record is `Supported` with a scalar
  harness ranging its one argument over `z`'s `[0, 9]` domain: QSL-261 is
  resolved by IR-296 and IR-297; IR-298 then resolves the result role from
  what the node itself names, and a single-operand node like this one has
  only one reachable candidate, so no own-bound ambiguity check ever runs.
  The generation output's `claim_map` is `Some` and holds the `-` node's
  claim as `Generated`; the whole claim map serializes.
- Step 4 routes indexes 0 and 2, generates once over two items, and the
  output's request indexes are `[0, 2]`.
- Step 5: E5 admits both packages at their emitted `package_id`. A float
  profile unit is not covered: no float parameter form this QSL revision
  admits was found.
- Step 6: both items route and generate; nothing is rejected. The `=`
  item's record is `Unsupported` with reason `no_derivable_claim` naming
  the node and CG's `OperationNotDerivable` refusal, and carries no
  harness. The `-` item's record and harness equal those of a run driven
  with only the `-` item: the underivable item does not affect its
  sibling. The derived claim map holds both nodes: the `=` node's claim is
  `Refused` with an `Underived` operation provenance (nobody declared it),
  the `-` node's is `Generated`, and the whole map serializes.
- Steps 7 and 8's intended outcome, once IR-302 is fixed, is `Supported`:
  step 7's `+` item ranges its one argument over `x`'s own `[0, 9]` domain
  (the literal `1` needs no bound of its own); step 8's `+` item ranges its
  two arguments over `x`'s `[0, 9]` and `y`'s `[10, 20]` domains
  independently. Today, through real QSL output, CG's IR-298 rule settles
  step 7 `RequiresBound` (naming the literal's own unbounded type: QSL
  represents every literal as a reference to its own plain-typed value
  node, never inline) and step 8 `Unsupported` (`AmbiguousBound`: `x` and
  `y` each own a bound and nothing else is reachable from the `+` node
  itself, so removing both leaves none) -- this is CG's own regression,
  tracked as IR-302, not the driver's expected behavior. The driver's own
  tests assert the pass-through invariant unconditionally (the driver's
  output equals a direct `generate_routed` call, whatever CG settles) and
  keep the intended-`Supported` assertions `#[ignore]`d until IR-302
  lands.
