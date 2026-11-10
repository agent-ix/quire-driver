---
id: TC-034
title: "Project QSL requirement records into CG's typed input"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: verifies
---
# TC-034: Project QSL requirement records into CG's typed input

## Description

Verify that the driver hands CG each routed item's QSL requirement and the
package's requirement records exactly, and refuses typed when it cannot.

Scope: FR-020-AC-1 through FR-020-AC-15. Planned until CG PR #772 (IR-304
slice (a)) merges; AC-11 and AC-13 wait on IR-304 slice (b).
FR-001-AC-17 keeps its own evidence in TC-017; this case reuses that vector
for FR-020-AC-6 only.

## Test Procedure

1. Check a QSL unit with one node carrying two occurrences with different
   result bounds and guards (the built `if p then g(x + 1) else h(x + 1)`
   unit of AC-3), one with a conversion context and two guards, a map built with `FromIterator` whose
   one occurrence holds a `value-validity` and an `operation-contract`
   record, a bound `operation-contract` item with no
   `value-validity` record at its node, and the
   IR-304 guarded package including `if p then x + 1 else x + 1`. Capture
   what the driver passes to a recording CG seam (AC-1 to AC-3, AC-5 to
   AC-7, AC-12, AC-15).
2. In an in-module test in `src/drive.rs` (`drive()` takes no
   `RequirementMap`), call the private `lower_driver_request`, negotiation
   and `generate` with a `RequirementMap` argument missing the routed
   item's `value-validity` record (AC-4), and with a record whose
   guard node the admitted package does not hold, and with a record whose
   key has an unmapped role (AC-14). Assert the fault
   fields, code, stage, kind and that the recording seam saw no generation
   call.
3. Reuse the FR-010-AC-4 partition vector, once as is and once with B's
   record removed from the map (AC-6).
4. Tamper one record's result-bound node, run with real CG, and compare the
   returned refusal (AC-8). Feed a fake CG error of each slice (a) mismatch
   variant (AC-9) and of `GuardContextMissing` (AC-13).
5. Run the guarded package through real CG with the Kani provider fixture
   (AC-10 in slice (a), AC-11 after slice (b)).

## Mutants

Each mutant must make the named assertion fail; none is equivalent to the
correct driver.

| Mutant | Caught by |
|---|---|
| Build the occurrence from the node id with role and ordinal dropped | AC-1 occurrence equality; AC-3 (two items collapse) |
| Reverse the path condition, or flip a `holds` value | AC-1 ordered-guard equality |
| Drop the conversion context or fill it from the node | AC-1 `None` and `Some` cases |
| Build records only for routed items | AC-2 refused item's record; AC-6 B's record |
| Project every capability's record, so a later one overwrites at a shared occurrence | AC-2 (no other-capability entry); AC-5 (entry fields equal the `value-validity` record) |
| Look the item's record up by occurrence alone | AC-5 (the ambiguous key returns no record, so the run faults) |
| Attach a requirement to every bound item, whatever its capability | AC-15 (`None` expected at the `operation-contract` index) |
| Fault at projection time for any bound item without a record | AC-15 (the run must complete with `InvalidRequest` settled); AC-4 (the fault must come only after negotiation, for the routed item) |
| Skip the absent-record check for a routed item, or check items that are not routed | AC-4 (fault expected, no CG call); AC-15 (no fault expected) |
| Fill an absent record from the item's own fields | AC-4 fault and no-generation-call assertions |
| Skip a record that fails to resolve and continue | AC-14 fault and no-generation-call assertions |
| Attach a requirement at B's index, or fault on B's missing record | AC-6 `None` at index 1; completion with B's record removed |
| Place guard roots before the conversion root, in reverse, or advance driver indices | AC-7 position order and next index |
| Emit a shared guard root once for both items | AC-12 (the else span lacks the root) |
| Replace the QSL result bound with the IR selected type | AC-8 (CG then reports no mismatch) |
| Map a CG record refusal to `invalid_request` | AC-9, AC-13 wire code |
| Report a CG record refusal at the Negotiation stage | AC-9, AC-13 stage |
| Report a CG record refusal as outcome kind refusal | AC-9, AC-13 outcome kind |
| Rewrite a CG refusal's request index, occurrence or mismatch | AC-9, AC-13 field equality |
| Strip an unlowered guard | AC-10, AC-11 (no guard-free harness) |

## Expected Results

- The projection equals QSL's `value-validity` values field by field; an
  absent or unresolvable record is a `runtime_invariant` Generation fault
  before CG is called; CG's record refusals pass through unchanged.
