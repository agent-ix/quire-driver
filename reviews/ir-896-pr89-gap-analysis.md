---
id: SR-13546
title: "IR-896 PR 89 gap-analysis review"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; src/refusal.rs; src/drive.rs; spec/tests.md; FR-001-AC-22"
review_set: subset
---

## Summary

Planless gap analysis of the PR diff at ed1f04c9b3d24d3467ce277613976968b3b61feb and the FR-001-AC-22 binding chain. The criterion is tagged, but the diagnostic emission and location/locus mapping remain unbacked.

## Verdict

FAIL

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-001-AC-22 has no test or production path proving emitted FR-272 diagnostic location and typed locus; the matrix row marks it complete. | spec/tests.md:48 |

## Coverage

Computed quire matrix --strict shows FR-001-AC-22 tagged by the two src/refusal.rs unit tests; 41 other repository criteria are untagged and one is tagged only by ignored tests, outside this PR scope. Plan completion: not assessed.

Examined units:
- `FR-001-AC-22` (examined) in `spec/functional/FR-001-orchestrate-the-driver-steps.md`: When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`, `document_pointer`, `expected` and `actual`. The diagnostic carries the same code and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location, and carries `locus` as a `CheckedNodeId` payload when present; every returned code has an FR-271 row.
- `TC-004` (examined) in `spec/test-cases/TC-004-command-level-refusals.md`: At the CG package-reader boundary, return `ContractPackageReadRefused` results covering a returned code with a cause and one without a cause, plus present and absent optional payload members.
- `spec/tests.md:48` (examined) in `spec/tests.md`: | FR-001 | FR-001-AC-22 | TC-004 | ✅ `src/refusal.rs` tests retain all eight typed refusal fields in both present and absent forms, map the code/cause/category/stage, cover the five permitted incomplete limits and the `Diagnostics` invariant fault, and retain each CG projection unchanged. `src/drive.rs` converts the CG negotiation error through `DriverRefusal::from`. FR-460/FR-461 reader-limit registration remains deferred. |

## Evidence

The new unit test checks DriverRefusal getters and retained projections. Source inspection found no call site constructing an FR-272 diagnostic from DriverRefusal or assigning path to FR-274 json_path and locus to a typed payload. A change that drops those diagnostic fields leaves the tagged tests green, while the new matrix row reports full completion.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | At 1fa0e7f, `DriverRefusal::from` constructs `PackageReadDiagnostic` at the CG boundary; tests assert its code, cause, category, stage, JsonPath, typed locus, and limit payload. The exact projection remains retained. Focused tests pass. |
