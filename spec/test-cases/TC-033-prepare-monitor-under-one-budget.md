---
id: TC-033
title: "Prepare a monitor program from a plan root under one budget"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-019
    type: verifies
---
# TC-033: Prepare a monitor program from a plan root under one budget

## Description

Verify that one genuine QSL plan root, its independent selection and one
`NativeJoinBudget` yield one runtime Program with an exact `Layout`, or one
typed first refusal, with every phase charging the same budget.

## Test Procedure

1. Produce a plan root for the 20-bit 1 MHz deadline claim with a genuine
   observation progress association. Strictly read its emission, select the
   package, root, clause, subject, occurrence, profile and target
   independently, and prepare with an ample budget. Compare every `Layout`
   field with the IR layout context, the receipt and QSL's and the owner's
   values. Drop the plan root, package and request and build twice
   (FR-019-AC-1).
2. Record the successful consumption and the first charge of each phase.
   Prepare at the exact limits, then once per phase with a limit one below
   that phase's first charge (FR-019-AC-2).
3. Deny a QSL adapter and `identity_rows` charge; remove the order premise and
   the association; take the handoff from a second emission of the same
   source (FR-019-AC-3, FR-019-AC-4).
4. Place defects in two phases at once; set cancellation before each phase
   (FR-019-AC-5).
5. Prepare a plan whose entry reaches a map; change only `PlanId` metadata,
   then only an entry body (FR-019-AC-6).
6. Compare consumption before and after `Program::try_new`; force an RT
   construction refusal (FR-019-AC-7).
7. Inspect the public API and source (FR-019-AC-8).

Tag the tests `#[trace("TC-033", "<criterion id>")]`. Steps 1 to 6 run in
`quire-integration` with released QSL, IR, observation and RT; a hand-built
plan alone receives no credit.

## Expected Results

- Step 1: every field is byte-equal; both builds are independent.
- Step 2: exact limits succeed; each one-below run refuses in its phase with
  the first unpaid kind, amount and locus and earlier consumption retained.
- Step 3: each QSL or IR refusal arrives unchanged with its stated category;
  the second emission refuses `ForeignEmission`.
- Step 4: the earlier phase wins; cancellation returns `cancelled` with no
  program.
- Step 5: the map's key and value types are in the environment; `PlanId`
  alone does not change `Layout`, a body change changes `Layout.root`.
- Step 6: consumption is unchanged; the refusal is `program`.
- Step 7: one budget parameter on each entry and none of the excluded items.
