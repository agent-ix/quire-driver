---
id: SR-14861
title: "IR-1494 PR #147 gap-analysis (FR-020 ACs vs tests)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver PR #147 at 2d9b9220; IR-1494, IR-1504, IR-1511; spec/functional/FR-020-project-requirement-records-into-codegen-input.md, spec/test-cases/TC-034-project-requirement-records-into-codegen.md, spec/tests.md, spec/spec.md, src/drive.rs, src/refusal.rs, src/upstream.rs, tests/drive.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-034
    type: reviews
---

## Summary

Ticket: IR-1494. Planless gap analysis of FR-020's 15 acceptance criteria against `quire matrix` (quoin 0.29.1) and the tagged tests at PR head 2d9b9220. The PR changes no spec file. Matrix: AC-1, 2, 3, 4, 6, 7, 8, 9, 10, 12, 14 and 15 are tagged; AC-5, AC-11 and AC-13 are untagged. Plan completion: not assessed.

Per-AC reading (tests run: drive 44/44, refusal lib 32/32; mutation results are in SR-14860):
AC-2, AC-3, AC-6, AC-12, AC-15 and AC-4 are asserted with the driver's output compared field by field against expectations built from QSL's own records. AC-8 tampers real records (swaps the two `x + 1` records) and checks CG's `ResultBoundMismatch` comes back unchanged. AC-9 covers all seven slice (a) mismatches (CG main dab4861f has exactly those seven; `GuardContextMissing` is absent) with code, stage, kind, field preservation and distinct messages. AC-10 uses a local guarded unit, not the IR-304 package by name, and asserts the slice (a) settlement exactly. AC-11 and AC-13 are legitimately Planned: they need IR-304 slice (b), which CG main does not contain. AC-1 and AC-7 are tagged, but their defining condition (an item with two path guards) is absent from every fixture (FND-001). AC-5 is untested, but the blocker the author gives does not hold (FND-002). AC-14 covers the key-role and key-node causes, not the guard-node example the AC leads with; that half is plausibly blocked because QSL's `PathGuard` and `RequirementRecord` have no public constructor and a compiled package's guard nodes always resolve (FND-004). No spec row claims Implemented without a test, because the PR leaves every FR-020 status at Planned. That status is now stale, and the remaining gaps are recorded nowhere in the spec (FND-003).

## Verdict

**NOT mergeable.** FND-001 (high) leaves AC-1/AC-7's ordered two-guard requirement unexercised; FND-002 leaves an AC uncovered behind a blocker that does not apply; FND-003 leaves the traceability record stating the opposite of the code's state.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-7 (and AC-1) require an item with two path guards in outermost-first order; the tagged test's fixture has one guard per item (guard_root_total == 2 over two items), so the order is never checked and the reverse-order mutant survives | tests/drive.rs:3427-3523 |
| FND-002 | medium | AC-5 is untested and not blocked: RequirementMap implements FromIterator and records are Clone, so the operation-contract record of MODEL_READING_UNIT can be re-keyed onto the value-validity item's occurrence through the existing requirement_records seam (the technique rekey() already uses) | tests/drive.rs:3568-3587 |
| FND-003 | medium | Spec status not updated: tests.md FR-020/TC-034 rows, spec.md index, FR-020 Description/Dependencies and AC-9/AC-10 rows still say Planned until CG PR #772 / types not merged, and AC-5, AC-14's guard-node half and the slice (b) dependency of AC-11/AC-13 are not recorded as the remaining gaps | spec/tests.md:127 |
| FND-004 | low | AC-14's guard-node case (its first example) is not exercised: the test re-keys a record's key to an unknown node; the guard-node half is plausibly unconstructible but is not documented as uncovered | tests/drive.rs:3593-3660 |
| FND-005 | low | TC-034 step 2 prescribes an in-module test that calls the private lower_driver_request with a RequirementMap; the PR instead adds a public defaulted trait method Upstream::requirement_records as the seam, which FR-020/TC-034 do not name | src/upstream.rs:120-124 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| FR-020-AC-1 | examined | For a routed item whose QSL record has a conversion context and a two-guard path condition (one guard requiring holds = true, one holds = false), the RoutedRequirement passed to CG carries that item's occurrence, the V2 node of its result bound, the conversion occurrence, and the two guards in QSL's order with the same holds values. For an item with no conversion context it carries None. |
| FR-020-AC-2 | examined | RequirementRecords on the Kani context holds one entry per value-validity record of the package's requirements(), including a record whose item the export refused, and no entry for a record of another capability. |
| FR-020-AC-3 | examined | One x + 1 node's then and else occurrences guarded by p and not p and converted to Int[0, 10] and Int[0, 20]; the two routed items carry different RoutedRequirement values. |
| FR-020-AC-4 | examined | No value-validity record for a Kani-routed item: refuses after negotiation with RequirementRecordAbsent; CG generation is not called. |
| FR-020-AC-5 | examined | A RequirementMap built with FromIterator holding a value-validity and an operation-contract record at one occurrence: RequirementRecords holds exactly the value-validity record's fields, no fault. |
| FR-020-AC-6 | examined | FR-010-AC-4 vector: requirements for A and C, None at B, RequirementRecords still holds B's record, completes also with B's record removed. |
| FR-020-AC-7 | examined | For an item with two path guards and a conversion context, the conversion ContextRoot, then one ContextRoot per guard in outermost-first order, inside the item's span. |
| FR-020-AC-8 | examined | A record whose result bound disagrees: CG's ResultBoundMismatch returned unchanged as a Generation runtime_invariant fault. |
| FR-020-AC-9 | examined | Each of the seven slice (a) RequirementRecordMismatch variants returned unchanged, Generation, runtime_invariant, Fault. |
| FR-020-AC-10 | examined | Slice (a): unguarded item generates a harness; guarded items settle path_guard_not_lowered / guard_lowering_not_implemented with no harness. |
| FR-020-AC-11 | examined | After IR-304 slice (b): guard Assume and requirement_occurrence identity. |
| FR-020-AC-12 | examined | if p then x + 1 else x + 1: a ContextRoot for p in each item's span, not emitted once. |
| FR-020-AC-13 | examined | After IR-304 slice (b): GuardContextMissing returned unchanged. |
| FR-020-AC-14 | examined | A record with a guard occurrence whose node the admitted package does not hold refuses RequirementRecordUnresolved with UnknownNode; an unmapped key role gives UnknownOccurrenceRole; raised before generation. |
| FR-020-AC-15 | examined | A bound operation-contract item projects None, adds no guard root, raises no fault; a value-validity item still receives its requirement. |
| TC-034 | context_only | Test procedure and mutant table for FR-020-AC-1..15. |
