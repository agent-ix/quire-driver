---
id: SR-20261104030007
title: "IR-1333 PR 116 gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; spec/functional/FR-001-orchestrate-the-driver-steps.md; src/refusal.rs; tests/drive.rs; Cargo.lock"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
---
# IR-1333 PR 116 gap analysis

## Summary

Ticket: IR-1333. This diff-scoped check reconciled the changed `UnitMismatch` mapping with FR-001-AC-22, its computed trace binders, the test body, and the production mapping. The new behavior has an owning criterion and a test that executes the production conversion. Plan completion: not assessed.

## Verdict

**PASS for the changed behavior.** `quoin matrix --repo . --json` reports FR-001-AC-22 tagged to `reader_timed_and_semantic_refusals_keep_distinct_typed_payloads`. Its assertions compare the produced refusal and diagnostic against literal expected values, including `unit-mismatch` and `ill_typed`. The workspace compile defect is recorded in the code-review artifact and prevents a PR-wide PASS.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Coverage

- Examined `FR-001-AC-22`: "When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`, `document_pointer`, `expected` and `actual`. The diagnostic carries the same code and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location, and carries `locus` as a `CheckedNodeId` payload when present; every returned code has an FR-271 row."
- The changed test binding is correct. Existing companion tests cover remaining projection fields and incomplete limits; no new stub or reverse ownership gap was introduced.
- Repository-wide matrix output has unrelated pre-existing untagged criteria, including FR-001-AC-23 and AC-24. This review judges the changed AC-22 path only.
- Plan completion: not assessed. Semantic review of unrelated requirements: skipped.
