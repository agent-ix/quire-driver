---
id: SR-2026110414560040
title: IR-977 spec-review-integrity review
type: SpecReview
analysis: integrity
scope: agent-ix/quire-driver; IR-977; spec/functional/FR-009-settle-execute-and-replay-results.md;
  spec/test-cases/TC-016-execute-replay-and-terminal-records.md; src/frame_replay.rs;
  src/drive.rs; src/refusal.rs; src/lib.rs; tests/frame_replay.rs; Cargo.lock
review_set: subset
---

## Summary

Reviewed frozen PR #121 for IR-977. The new preparation contract is consistent with AC-17..19 and existing FR-001 runtime-invariant ownership. TC-016 exercises each bounded obligation. Package and full occurrence identity are explicit; no fallback guessing or ambiguous duplicate authority is introduced. Existing relationships are unchanged. Full settlement remains explicitly future work.

## Verdict

**PASS** for the changed surface; no actionable defect found. Final aggregate gate remains the code lead's responsibility.

## Coverage

No applicable AssuranceProfile exists in this repository. No reviewer build, Kani or proof execution ran. Inspected the five-test, scoped Clippy, format, changed-spec and projection-removal receipts. Dependency lock adoption changes only merged-main resolutions, without manifest pins.

scope:
- id: FR-009-AC-17
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For an operation-frame replay selected from an admitted package, the driver
    projects QSL's exact typed `OperationSite.frame_operation` value to the CG selected
    item associated with the same admitted package identity and request index, without
    changing the pair or deriving it from CG selection or caller spelling. QSL FR-121
    owns pair shape and declaring-object resolution; QSpec FR-323 owns replay-target
    and packet/request shape. This driver criterion covers only object-type operations
    exposed by QSL's current call-site API; process/state-machine operations and state-clause
    projection are outsid
- id: FR-009-AC-18
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: When QSL call-site selection returns a typed refusal for a frame operation,
    the driver preserves that refusal unchanged and stops before CG frame-target construction,
    replay, settlement, or terminal creation; it does not recast the refusal as a
    driver join fault.
- id: FR-009-AC-19
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: When QSL call-site selection succeeds but the driver detects that its retained
    association of the QSL result to the CG selected item is missing or has a package
    identity, request index or complete occurrence that differs from the original
    run or the CG item, the driver returns FR-001's typed `runtime_invariant` fault
    and stops before CG frame-target construction, replay, settlement, or terminal
    creation. A typed refusal actually returned by CG is preserved under CG's contract
    and is not this driver-detected join fault.
- id: FR-009-frame-preparation
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: The production `Driven::prepare_frame_replay` entry SHALL use the existing
    CG `FrameReplayInputs` envelope and the original `Driven` request inventory.
- id: TC-016-frame-projection
  path: spec/test-cases/TC-016-execute-replay-and-terminal-records.md
  role: examined
  excerpt: For the frame-target projection scenario, call production `Driven::prepare_frame_replay`
    with the real QSL producer and public CG consumer.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
