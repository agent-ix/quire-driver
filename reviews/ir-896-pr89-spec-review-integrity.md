---
id: SR-13548
title: "IR-896 PR 89 spec-review/integrity review"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-driver; spec/tests.md; FR-001-AC-22; TC-004"
review_set: subset
---

## Summary

Checked the two changed matrix rows for structural consistency and identifier integrity at ed1f04c9b3d24d3467ce277613976968b3b61feb. The matrix links the right criterion and test case; the evidence-status defect is recorded in the base review.

## Verdict

PASS

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

FR-001-AC-22 and TC-004 were examined. No requirement text or relationships changed; no duplicate/missing ID or contradictory mapping was introduced.

Examined units:
- `FR-001-AC-22` (context_only) in `spec/functional/FR-001-orchestrate-the-driver-steps.md`: When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`, `document_pointer`, `expected` and `actual`. The diagnostic carries the same code and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location, and carries `locus` as a `CheckedNodeId` payload when present; every returned code has an FR-271 row.
- `spec/tests.md:48` (examined) in `spec/tests.md`: | FR-001 | FR-001-AC-22 | TC-004 | ✅ `src/refusal.rs` tests retain all eight typed refusal fields in both present and absent forms, map the code/cause/category/stage, cover the five permitted incomplete limits and the `Diagnostics` invariant fault, and retain each CG projection unchanged. `src/drive.rs` converts the CG negotiation error through `DriverRefusal::from`. FR-460/FR-461 reader-limit registration remains deferred. |
- `spec/tests.md:122` (examined) in `spec/tests.md`: | TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5, FR-001-AC-22 | ✅ command-level refusal tests and focused CG reader-projection unit tests present |
