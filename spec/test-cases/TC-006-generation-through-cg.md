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

Scope: FR-001-AC-6 through FR-001-AC-8, and FR-001-AC-12.

## Test Procedure

The requested items are not caller input: they are written from the
compiled unit's own checked requirement records
(`CheckedGraph::requirements()`, QSL-266), one per scalar operation
application, in occurrence-key order.

1. Drive `function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }`
   with an empty registry, and take its admitted package. The unit writes
   one requirement record, the `-`.
2. With `kani` registered for `value-validity`, drive the same unit, with
   a Kani context carrying no claim map (IR-294: CG derives it from the
   package and the routed node alone). Its one item routes to `kani` at
   request index 0.
3. Call CG `generate_routed` directly over the same package with the
   routed item (index 0, the `-` node, the `kani` wire pair, kind Kani) and
   the same context.
4. Drive `function inc using v(x: Int[0, 9]): Int[0, 10] pure { x + 1 }`,
   which writes one requirement record, the `+`, routed to `kani`, and
   compare the driver's generation output with a direct `generate_routed`
   call over the same routed item.
5. Drive a unit with a `Text[1, 5; nfc]` parameter and one with a
   `Rational[0, 9; 1, 9]` parameter, with an empty registry (neither
   declares a scalar operation application).
6. Drive a unit with an integer `=` (`quire.op.integer.eq`, a node CG's
   derivation names no operator family for) alongside an integer unary `-`
   (`quire.op.integer.negate`, which it does derive): two requirement
   records, the `=` at index 0 and the `-` at index 1, both routed to
   `kani`.
7. Drive step 4's unit again, and check its `+` item's disposition and
   harness directly, rather than only by pass-through equality with CG.
8. Drive `function add using v(x: Int[0, 9], y: Int[10, 20]): Int[10, 29]
   pure { x + y }`, routing the `+` node to `kani`.
9. Drive `function unreachable_result using v(x: Int[0, 9]): Int[0, 10]
   pure { x + 100 }`, with an empty registry.
10. Drive `function unreachable_branch using v(x: Int[0, 9]): Int[0, 10]
    pure { if x > 20 then x + 100 else 0 }`. The unit's `if` guard
    (`x > 20`) is itself a scalar comparison, so this writes two
    requirement records: the `+` at index 0 and the guard `>` at index 1.
    Neither item has a selected conversion context, so both settle as
    unsupported before routing or generation.
11. Drive the ordinary unit with caller-supplied FR-056 domain package
    documents and capture the typed evidence passed to IR's E5 reader.
    Compare the reader's evidence with the supplied digest and complete
    document bytes, plus the supported root feature.
12. Drive an admitted checked package containing a bounded integer form
    that CG FR-019 currently marks implemented, and a separate admitted
    package containing a valid rational scalar form selected from CG
    FR-019/TC-030's planned status register. At the driver-to-CG negotiation
    seam, record each request item and forward the envelope unchanged to CG.
    Compare the typed IR tag/form structurally with the corresponding
    requirement and node in the exact admitted checked package, matched by
    occurrence.
13. Using the same checked rational form, exercise three candidate controls
    with candidate sets produced by QSL from their registered descriptors,
    not authored by the test: (a) no candidate; (b) two valid candidates and
    no named backend; and (c) one Kani candidate that passes candidate
    validation but advertises `bounded` only for an unbounded item whose
    `finite_bound_available` is true. Forward each typed item unchanged to
    CG's public negotiation operation. Do not construct a caller-supplied CG
    lowerability/refusal value or derive the form from display/debug text.
14. Drive the RR-16 source with `g(p: Int[0, 10])`, `h(q: Int[0, 20])`, and
    `f(x: Int[0, 9]) { g(x + 1) and h(x + 1) }`, with Kani registered.
    Compare each QSL requirement's exact application occurrence and direct
    narrow occurrence with the driver's alternating `[add, narrow]` roots;
    verify the shared add node produces two routed items and the two narrow
    roots remain attached to their original occurrences through CG generation.

Tests: `tests/drive.rs`,
`tc_001_tc_006_steps_run_in_order_through_generation` (steps 1 to 3),
`tc_006_x_plus_1_generation_passes_through_unchanged` (step 4),
`tc_006_e5_admits_text_and_rational_units` (step 5),
`tc_006_an_underivable_node_settles_unsupported_and_its_sibling_is_unaffected`
(step 6), `tc_006_x_plus_1_settles_supported` (step 7),
`tc_006_x_plus_y_settles_supported_with_independent_argument_bounds` and
`tc_006_x_plus_y_generation_passes_through_unchanged` (step 8),
`tc_006_x_plus_100_in_the_return_context_is_refused_at_check` (step 9), and
`tc_006_x_plus_100_in_a_dead_branch_settles_result_bound_unreachable`
(step 10), and `tc_006_handoff_passes_domain_package_evidence_unchanged`
(step 11), and `tc_006_rr16_preserves_each_occurrence_conversion_context`
(step 14).

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
- Step 4's generation output equals the direct `generate_routed` call over
  the same routed item, whatever CG settles it.
- Step 5: E5 admits both packages at their emitted `package_id`. A float
  profile unit is not covered: no float parameter form this QSL revision
  admits was found.
- Step 6: both items route and generate; nothing is rejected, and the
  generation output is keyed by request index, ascending: `items[0]` is
  the `=` at index 0, `items[1]` is the `-` at index 1. The `=` item's
  record is `Unsupported` with reason `no_derivable_claim` naming the node
  and CG's `OperationNotDerivable` refusal, and carries no harness. The
  `-` item's disposition and whole harness equal a direct `generate_routed`
  call over the `-` item alone: the underivable `=` item does not affect
  its sibling. The derived claim map holds both nodes: the `=` node's
  claim is `Refused` with an `Underived` operation provenance (nobody
  declared it), the `-` node's is `Generated`, and the whole map
  serializes.
- Step 7's `+` item settles `Supported`, ranging its one argument over
  `x`'s own `[0, 9]` domain: the literal `1` is a reference to its own
  plain-typed `value` node, which IR-302 reads as a literal operand (its
  body decides it, not its type), owning no bound and constrained to
  exactly its own value.
- Step 8's `+` item settles `Supported`, ranging its two arguments over
  `x`'s `[0, 9]` and `y`'s `[10, 20]` domains independently: the `+`
  node's own type is the plain scalar type, and only the declared
  context's narrowing `conversion` consumes it, so IR-302 reads the
  conversion's own `[10, 29]` bound as the `+` node's result bound.
- Step 9's unit does not compile: QSL's own checker proves `x + 100`'s
  every exact result (`[100, 109]`) is disjoint from the declared `[0,
  10]` result bound and refuses at the check stage
  (`undefined_expression`/`unproved-range`), before CG is ever called.
  CG's own `ResultBoundUnreachable` refusal names the identical
  disjoint-interval fact (confirmed against CG's own fixture test,
  `quire-contract-codegen`'s
  `tc_033_qsl_shaped_literal_and_consumer_refusals_have_no_harness`). This
  is one shape, not a general guarantee that the refusal is unreachable:
  step 10 reaches it.
- Step 10's unit compiles: QSL's `Facts::walk` proves `x > 20` is never
  true over `x: Int[0, 9]` and never discharges the dead branch's
  coercion, so the checked package still emits. Neither item has a selected
  conversion context. The add settles as
  `Unsupported`/`ResultBoundUnreachable` (`quire.op.integer.add`, result
  `[0, 10]`, reachable `[100, 109]`); the guard settles as
  `Unsupported`/`OperationNotRendered`. Neither item is routed or generated.
- Step 11's E5 evidence contains every caller-supplied domain-package
  document under the original digest and with the original bytes, and still
  contains the supported root feature. Dropping or changing that evidence
  changes the typed value observed by the reader and fails the assertion.
- Steps 12 and 13's captured form equals the typed form in the matched
  admitted checked requirement/node. The bounded implemented form is settled
  by CG according to FR-019. In step 13(a), CG settles the empty set as
  `unsupported` with `unsupported_projection` /
  `unsupported-requested-capability`. In step 13(b), CG settles the
  ambiguous set as `invalid-request` with
  `invalid_capability`/`ambiguous-backend`, naming both candidates in
  candidate order. In step 13(c), candidate validation selects the one Kani
  candidate, then the valid planned rational form settles with the exact typed
  refusal in its TC-030 Planned row before the unbounded/bounded-only
  `requires-bound` result. Altering, dropping or replacing the checked form,
  editing a QSL-produced candidate set, or deciding lowerability/refusal in
  the driver fails the assertion.
- Step 14's application and conversion occurrence keys match QSL's two
  requirement records exactly. Lowering positions are `[DriverItem(add,
  narrow-1), ContextRoot(narrow-1), DriverItem(add, narrow-2),
  ContextRoot(narrow-2)]`; the application occurrences and narrow roots
  differ, the add node is shared, and CG returns exactly the two routed items
  without swapping their contexts.
