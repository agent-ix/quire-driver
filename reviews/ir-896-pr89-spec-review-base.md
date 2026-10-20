---
id: SR-13547
title: "IR-896 PR 89 spec-review/base review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver; spec/tests.md; FR-001-AC-22; TC-004"
review_set: subset
---

## Summary

Reviewed the changed evidence matrix against FR-001-AC-22 and TC-004 at ed1f04c9b3d24d3467ce277613976968b3b61feb. The new complete status exceeds the evidence named in the row.

## Verdict

FAIL

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The new ✅ status reports FR-001-AC-22 complete although its FR-272 diagnostic and FR-274 location/locus obligations are not evidenced. | spec/tests.md:48 |

## Coverage

Reviewed both changed spec/tests.md rows, the owning criterion and TC-004 procedure/expected results. No requirement statement changed in this PR.

Examined units:
- `FR-001-AC-22` (examined) in `spec/functional/FR-001-orchestrate-the-driver-steps.md`: When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`, `document_pointer`, `expected` and `actual`. The diagnostic carries the same code and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location, and carries `locus` as a `CheckedNodeId` payload when present; every returned code has an FR-271 row.
- `TC-004` (examined) in `spec/test-cases/TC-004-command-level-refusals.md`: Step 10's refused result becomes an FR-272 refusal diagnostic at `package_read`, with CG code and cause unchanged, category `refusal`, the `path` as the FR-274 `json_path` location, and a present `locus` retained as typed `CheckedNodeId`.
- `spec/tests.md:48` (examined) in `spec/tests.md`: | FR-001 | FR-001-AC-22 | TC-004 | ✅ `src/refusal.rs` tests retain all eight typed refusal fields in both present and absent forms, map the code/cause/category/stage, cover the five permitted incomplete limits and the `Diagnostics` invariant fault, and retain each CG projection unchanged. `src/drive.rs` converts the CG negotiation error through `DriverRefusal::from`. FR-460/FR-461 reader-limit registration remains deferred. |
- `spec/tests.md:122` (examined) in `spec/tests.md`: | TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5, FR-001-AC-22 | ✅ command-level refusal tests and focused CG reader-projection unit tests present |

## Evidence

The row cites src/refusal.rs projection tests and src/drive.rs conversion. Neither emits the FR-272 diagnostic specified by FR-001-AC-22 and TC-004, nor proves path becomes json_path and locus remains a typed CheckedNodeId payload. Mark this AC partial until that evidence exists, or add a failing diagnostic-path test and implementation before retaining ✅.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 1fa0e7f, the ✅ row now cites the actual typed diagnostic constructor and tests for FR-274 JsonPath and typed locus; the previously unsupported completion claim is backed. |
