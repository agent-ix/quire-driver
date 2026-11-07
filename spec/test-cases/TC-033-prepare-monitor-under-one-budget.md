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

1. Produce a plan root for the 20-bit 1 MHz deadline claim and a genuine
   observation progress view. Select the lock, root, clause, subject,
   occurrence, profile, target and time dimension independently, and prepare
   with an ample budget; confirm the driver makes no strict read. Compare every `Layout`
   field with the IR layout context, the receipt and QSL's and the owner's
   values. Drop the plan root, package and request and build twice
   (FR-019-AC-1).
2. Record the successful totals and, for each phase and resource, the
   cumulative consumption before the phase plus its first charge. Prepare at
   the exact totals, then for every phase once with work and once with
   storage limited to that cumulative value minus one (FR-019-AC-2).
3. Deny an `admit_authority`, `identity_rows` and adapter charge; omit the
   progress view; change the claim and observation selection; build the
   authority row from one authority and read it against the package of a
   second admission of the same root bytes; make `try_new` refuse
   (FR-019-AC-3, FR-019-AC-4).
4. Place defects in two phases at once; set cancellation before each phase
   (FR-019-AC-5).
5. Prepare a plan whose entry reaches a map; change only `PlanId` metadata,
   then only an entry body (FR-019-AC-6).
6. Compare consumption before and after `Program::try_new`; force an RT
   construction refusal (FR-019-AC-7).
7. Inspect the public API and source (FR-019-AC-8).
8. Convert every Layout value from a genuine plan; give a time dimension
   unequal to the admitted one; give an unconvertible key (FR-019-AC-9).
9. Inspect the source mapping of an absent `tick_unit_dimension()` and of
   disagreeing entry result types (FR-019-AC-10).

Tag the tests `#[trace("TC-033", "<criterion id>")]`. Steps 1 to 6 run in
`quire-integration` with released QSL, IR, observation and RT; a hand-built
plan alone receives no credit.

## Expected Results

- Step 1: every field is byte-equal; both builds are independent.
- Step 2: exact totals succeed; each work and storage one-below run refuses
  in its phase with the first unpaid kind, amount and locus and earlier
  consumption retained.
- Step 3: each QSL or IR refusal arrives unchanged with its stated category;
  the second emission refuses `ForeignEmission`.
- Step 4: the earlier phase wins; cancellation returns `cancelled` with no
  program.
- Step 5: the map's key and value types are in the environment; `PlanId`
  alone does not change `Layout`, a body change changes `Layout.root`.
- Step 6: consumption is unchanged; the refusal is `program`.
- Step 7: one budget parameter on each entry and none of the excluded items.
- Step 8: conversions succeed; the mismatched dimension returns
  `target_mismatch`; the key failure returns `projection` with the native
  cause.
- Step 9: both conditions map to `internal`, with no fault-injection seam or
  public closure constructor.
