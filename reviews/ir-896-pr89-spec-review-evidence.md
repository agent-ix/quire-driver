---
id: SR-13549
title: "IR-896 PR 89 spec-review/evidence review"
type: SpecReview
analysis: evidence
scope: "agent-ix/quire-driver; spec/tests.md; FR-001-AC-22; TC-004"
review_set: subset
---

## Summary

Checked FR-001-AC-22 verification method against quoin advise and inspected the cited evidence at ed1f04c9b3d24d3467ce277613976968b3b61feb. Test is an applicable method; the diagnostic emission portion has no producing test.

## Verdict

CONDITIONAL

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The evidence row names only projection and conversion checks for a criterion whose verification also requires observable diagnostic output. | spec/tests.md:48 |

## Coverage

quoin advise --json reports authored Test with no mismatch; it recommends example/unit and integration test evidence among its matches. The projection unit tests address field retention, while the TC-004 diagnostic result is unobserved.

Examined units:
- `FR-001-AC-22` (examined) in `spec/functional/FR-001-orchestrate-the-driver-steps.md`: When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`, `document_pointer`, `expected` and `actual`. The diagnostic carries the same code and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location, and carries `locus` as a `CheckedNodeId` payload when present; every returned code has an FR-271 row.
- `TC-004` (examined) in `spec/test-cases/TC-004-command-level-refusals.md`: Step 10's refused result becomes an FR-272 refusal diagnostic at `package_read`, with CG code and cause unchanged, category `refusal`, the `path` as the FR-274 `json_path` location, and a present `locus` retained as typed `CheckedNodeId`.
- `spec/tests.md:48` (examined) in `spec/tests.md`: | FR-001 | FR-001-AC-22 | TC-004 | ✅ `src/refusal.rs` tests retain all eight typed refusal fields in both present and absent forms, map the code/cause/category/stage, cover the five permitted incomplete limits and the `Diagnostics` invariant fault, and retain each CG projection unchanged. `src/drive.rs` converts the CG negotiation error through `DriverRefusal::from`. FR-460/FR-461 reader-limit registration remains deferred. |

## Evidence

The authored Test method is appropriate, but the current cited tests stop at DriverRefusal. TC-004 expects an FR-272 diagnostic with FR-274 json_path and typed locus; no test observes that result. Add integration evidence or mark the criterion partial.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 1fa0e7f, the production `DriverRefusal::from` constructs the diagnostic and the focused tests observe each mapped diagnostic field, including location and locus. The Test method is now backed for this slice. |
