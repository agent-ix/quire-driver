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

Scope: FR-020-AC-1 through FR-020-AC-11. Planned until CG PR #772 (IR-304
slice (a)) merges.

## Test Procedure

1. Check a QSL unit with one node carrying two occurrences (different
   ordinal, result bound and path condition), one with a conversion context
   and two guards, and the IR-304 guarded package. Capture what the driver
   passes to a recording CG seam (AC-1 to AC-3, AC-6, AC-7).
2. Drive the projection function over a requirements view with the item's
   key removed (AC-4), and over a view in which two keys map to one
   occurrence (AC-5). Assert the fault fields, code, stage, kind and that the
   recording seam saw no generation call.
3. Reuse the FR-010-AC-4 partition vector, once as is and once with B's record removed from the view (AC-6).
4. Tamper one record's result-bound node, run with real CG, and compare the
   returned refusal (AC-8). Feed a fake CG error of each mismatch variant
   (AC-9).
5. Run the end-to-end package through real CG with the Kani provider fixture
   (AC-10). Inspect `Cargo.toml` (AC-11).

## Mutants

Each mutant must make the named assertion fail; none is equivalent to the
correct driver.

| Mutant | Caught by |
|---|---|
| Build the occurrence from the node id with role and ordinal dropped | AC-1 occurrence equality; AC-3 (two items collapse) |
| Reverse the path condition, or flip a `holds` value | AC-1 ordered-guard equality |
| Drop the conversion context or fill it from the node | AC-1 `None` and `Some` cases |
| Build records only for routed items | AC-2 refused item's record; AC-6 B's record |
| Fill a missing record from the item's own fields | AC-4 fault and no-generation-call assertions |
| Overwrite on a duplicate key instead of refusing | AC-5 fault and no-generation-call assertions |
| Refuse on any missing record, or attach a requirement to B | AC-6 completion with B's record removed; B's absent requirement |
| Place guard roots before the conversion root, in reverse, or advance driver indices | AC-7 position order and next index |
| Replace the QSL result bound with the IR selected type | AC-8 (CG then reports no mismatch) |
| Map a CG record refusal to `invalid_request` or rewrite its fields | AC-9 code, kind and field equality |
| Strip an unlowered guard | AC-10 (no guard-free harness) |
| Add a `rev` to the CG dependency | AC-11 inspection |

## Expected Results

- The projection equals QSL's values field by field; a missing or duplicated
  record is a `runtime_invariant` Generation fault before CG is called; CG's
  record refusals pass through unchanged.
