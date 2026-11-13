---
id: SR-17940486709564847047749
title: "IR-1494 PR #148 FR-020 gap analysis"
type: SpecReview
analysis: gap-analysis
date: 2026-11-12
scope: "agent-ix/quire-driver PR #148 at 5438a5ac; IR-1494; FR-020-AC-1..AC-15 and TC-034 against src/drive.rs, src/refusal.rs, tests/drive.rs and the locked CG 7fb4f53c, IR 270410bb, QSL 4b7b6003"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-034
    type: reviews
---

## Summary

Ticket: IR-1494. Planless gap analysis. Plan completion: not assessed. `quire matrix` at 5438a5ac: FR-020-AC-1..9 and AC-11..15 `tagged`; AC-10 `method-without-symbol` (retired, Inspection), consistent with CG FR-022-AC-38 retired at 7fb4f53c. AC-9: CG 7fb4f53c defines exactly eight `RequirementRecordMismatch` variants and the test lists all eight. AC-11 and AC-13 run against real CG. IR-1494's acceptance ("hands CG the typed record per occurrence; a missing record is a typed refusal; an end-to-end test") is met by AC-1..AC-4 and AC-11/AC-13 over real CG.

The two items the PR keeps Planned are not blocked at the dependency versions this PR locks:

- Guard-node half of FR-020-AC-14 says "QSL's `PathGuard` and `RequirementRecord` have no public constructor". IR-1522 (Done) added `PathGuard::new` and `RequirementRecord::with_path_condition` (behind QSL's `test-support` feature) in QSL commit 268402763, which is an ancestor of the locked QSL 4b7b6003. The QSL review of that PR names the driver step: a `qsl-semantics` dev-dependency with `features = ["test-support"]` (git, `branch = "main"`).
- `KaniReaderCatalogUnavailable` mapping: "IR `CatalogReadError` has no public constructor". IR-1523 (Done) added `CatalogReadError::new`; its merge commit is 270410bb, exactly the IR revision this PR locks.

## Verdict

Not mergeable: both Planned items are deliverable now and their owner tickets were filed to unblock this ticket, so leaving them Planned (with a rationale that is false at the locked versions) parks two criteria with no remaining owner. Deliver both in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-020-AC-14 guard-node case stays Planned "QSL-owned" because "PathGuard and RequirementRecord have no public constructor", but the locked QSL 4b7b6003 contains IR-1522's `PathGuard::new` and `RequirementRecord::with_path_condition` (test-support). The case is testable now; no ticket owns it any more. Add the dev-dependency, the test, and flip the FR-020/TC-034/tests.md text. | spec/functional/FR-020-project-requirement-records-into-codegen-input.md:194, spec/test-cases/TC-034-project-requirement-records-into-codegen.md:16-17, spec/tests.md:127 |
| FND-002 | high | The `KaniReaderCatalogUnavailable` code mapping stays untested "because IR `CatalogReadError` has no public constructor", but the locked IR 270410bb is the IR-1523 merge that adds `CatalogReadError::new`. A mutant changing that arm's code survives today. Add the mapping test and drop the Planned note and the stale comment. | src/refusal.rs:999, src/refusal.rs:2065, spec/tests.md:127 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | The new guard-node test checks that the fault names "the record's QSL occurrence key" only as `assert_ne!(occurrence.node(), missing())`; a fault naming a different record's key, or any key other than the guard node, still passes. Assert the occurrence equals one of the rewritten value-validity record keys (or the first in map order). | tests/drive.rs:3818-3822 |

## Dispositions

Round 1, head 32113a0e. Reviewer mutants: M5 `.take(0)` on the record path-condition projection in src/drive.rs fails 6 drive tests including `a_guard_whose_node_the_package_lacks_refuses_before_negotiation` (tests/drive.rs:3813); M6 splitting the catalog arm to `Code::InvalidRequest` in src/refusal.rs fails `kani_reader_catalog_unavailable_is_a_runtime_invariant_fault`. Nothing is left Planned for FR-020 (FR-020, TC-034, tests.md); `quire matrix` tags AC-14 at tests/drive.rs:3773.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Guard-node case tested by `a_guard_whose_node_the_package_lacks_refuses_before_negotiation` with QSL `PathGuard::new` / `with_path_condition` via a `qsl-semantics` dev-dependency (git, `branch = "main"`, `features = ["test-support"]`, no pin); Cargo.lock unchanged; tests/dependency_edges.rs allow-list updated and passing. |
| FND-002 | fixed | `KaniReaderCatalogUnavailable` case built with `CatalogReadError::new` in `kani_oracle_cases`, own test `kani_reader_catalog_unavailable_is_a_runtime_invariant_fault`; the stale comment is removed. |
