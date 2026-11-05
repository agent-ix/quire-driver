---
id: SR-20261104030006
title: "IR-1333 PR 116 code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; Cargo.lock; src/drive.rs; src/execute.rs; src/provider.rs; src/refusal.rs; tests/drive.rs; quire-cli/src/cli.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
---
# IR-1333 PR 116 code review

## Summary

Ticket: IR-1333. Frozen head: `5150b44bd7717224b76148344177109c27e6e89c`. Reviewed `origin/main...HEAD` with the Rust review lane. The new `UnitMismatch` arm emits `unit-mismatch`; the FR-001-AC-22 test checks the driver's code, cause, stage, projection and diagnostic, and the upstream typed cause's serialization. All changed fixture calls retain their first two authority and identity arguments. The lockfile retains branch-main first-party dependencies. No CI workflow changed and no applicable AssuranceProfile was found.

## Verdict

**FAIL.** The current QSL API in the changed lockfile makes `quire-cli` fail to compile because its existing `SourceIdentity::new` calls still pass four arguments. The driver-only checks cannot establish a passing workspace gate.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The QSL lockfile update selects the two-argument `SourceIdentity::new`, but 18 `quire-cli` call sites still pass four arguments; the workspace CLI target cannot compile. | Cargo.lock:1277; quire-cli/src/cli.rs:731 | implementation-bug-despite-evidence |

## Coverage

- Examined: the changed lockfile, all five changed Rust files, the seven changed fixture calls, `FR-001-AC-22`, and the affected CLI constructor call sites.
- `FR-001-AC-22` excerpt: "When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`, `document_pointer`, `expected` and `actual`. The diagnostic carries the same code and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location, and carries `locus` as a `CheckedNodeId` payload when present; every returned code has an FR-271 row."
- The changed test has a meaningful oracle for the driver's new mapping: changing the new match arm away from `unit-mismatch` fails its `package_read_cause` and diagnostic assertions. The serialization assertion checks the upstream enum's serde spelling; it does not substitute for the driver assertions.
- Review was scoped to the PR diff. Aggregate gates were not run in this reviewer pass.
