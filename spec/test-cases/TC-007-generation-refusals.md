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

Verify the two input refusals of step 7.

Scope: FR-001-AC-5, FR-001-AC-6.

## Test Procedure

1. With `kani` registered, drive one `value-validity` item naming the `+`
   node of the TC-006 unit, with no Kani context.
2. Drive an unsupported item, then a `value-validity` item naming a node
   the package does not hold, with no Kani context.

Test: `tests/drive.rs`, `tc_007_generation_refusals_are_typed`.

## Expected Results

- Step 1 refuses with CG's `MissingKindContext` for Kani, at the generation
  step, code `invalid-request`, outcome kind refusal, after one generation
  call over one item.
- Step 2 refuses with `UnknownNode` at request index 1 naming that node, at
  the generation step, code `dangling_reference`, outcome kind refusal, and
  CG generation is never called.
