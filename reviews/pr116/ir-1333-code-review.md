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

## Dispositions

Round 1 reviewed fix commit `fa9697e89f2e6fbcea23307caab5c8e2b1db33e8`. All 18 CLI constructor calls now pass QSL's two labels; comparison of the deleted and added calls confirms that every authority and identity argument is unchanged. The TC-018 observation oracle now expects exactly QSL's two serialized fields. No new finding arose from the fix diff.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The 18 CLI call sites use the two-argument constructor; `quire-cli/src/cli.rs:731` is now `SourceIdentity::new("quire", &options.path)`. A focused `cargo check -p quire-cli --tests --locked` passes on the fix head. |

## Disposition pass 2

Reviewed the one-line test oracle change at `0fcf062ff4dab6191b01622608ea97dd8d85890b`. The locked `quire-semantic-value` revision `30c2493163cf35dd00d6dcaad63a7cbe9b2f5290` declares `Category::ALL` in this order: Success, Violation, Undefined, Refusal, Unsupported, Incomplete, Pending, Indeterminate, Inconclusive, InternalFailure. Its exhaustive `exit_code` match returns `[0, 10, 10, 20, 21, 22, 0, 11, 22, 30]` in that order, matching the new explicit test array. The test still checks the array length before zipping and compares the CLI's `process_exit_code` to each literal; a wrong mapping can fail the oracle. No new finding arose. FND-001 remains fixed, so no new disposition row was added.
