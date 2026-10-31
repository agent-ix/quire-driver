---
id: TC-007
title: "Generation refusals are typed"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-007: Generation refusals are typed

## Description

Verify step 7's one caller-input refusal, `MissingKindContext`.
`DriverFault::UnknownNode` (a routed item naming a node the checked
package does not hold) is a separate, internal-fault case: see TC-004
(`tc_004_a_fault_is_an_internal_fault_with_runtime_invariant`,
`src/refusal.rs`). It is not a caller refusal, and no caller input can
build one -- every item's node comes from the same compiled package's own
`CheckedGraph::requirements()` (QSL-266), and `checked` is IR's own
admission of the bytes that same package emitted, so the lookup this generator does cannot miss on any request
the driver itself builds.

Scope: FR-001-AC-5, FR-001-AC-6.

## Test Procedure

1. With `kani` registered for `value-validity`, drive
   `function inc using v(x: Int[0, 9]): Int[0, 10] pure { x + 1 }`, which
   writes one requirement record, the `+`, routed to `kani`, with no Kani
   context.

Test: `tests/drive.rs`, `tc_007_generation_refusals_are_typed`.

## Expected Results

- Step 1 refuses with CG's `MissingKindContext` for Kani, at the generation
  step, code `invalid-request`, outcome kind refusal, after one generation
  call over one item.
