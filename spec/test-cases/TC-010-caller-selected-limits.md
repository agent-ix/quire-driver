---
id: TC-010
title: "Apply explicit lifecycle limits and preserve limit outcomes"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-003
    type: verifies
---
# TC-010: Apply explicit lifecycle limits and preserve limit outcomes

## Description

Verify that CLI `--limit` values override the named operation limits, omitted
values keep their defaults, and a reached limit remains a typed incomplete
outcome.

Scope: FR-003-AC-1 through FR-003-AC-3.

## Test Procedure

1. Run an operation with one valid limit override and observe the effective
   limits at the operation boundary.
2. Run with an unknown field, malformed value and duplicate field.
3. Run an input that exceeds a chosen limit. Through the driver's typed
   execution wrapper, inspect the returned `LimitExceeded` configured bound
   and `setting()`; then repeat with the limit raised to fit the input.

Test location: `src/limits.rs`, `src/execute.rs`, `quire-cli/src/cli.rs`, and
`quire-cli/tests/binary_process.rs`.

## Expected Results

- The explicit value replaces only its field; omitted fields retain defaults.
- Invalid options are refused before any stage with QSL's refusal category.
- The low limit returns QSL's typed `LimitExceeded`; the driver wrapper
  preserves its configured bound and `setting()` and has category
  `Incomplete`. The process status is selected by QSL's category exit
  function. Raising the source-byte bound to the fixture's byte length allows
  the operation to complete.
