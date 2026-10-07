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

Scope: FR-003-AC-1 through FR-003-AC-5.

## Test Procedure

1. Inspect the parser and application path: names are resolved with QSL
   `Setting::from_name`, typed settings are passed to QSL
   `SettingLimits::set_bound`, and the driver has no parallel name-to-field
   table.
2. Read the selected operation's QSL `SettingLimits::bounds`. For every
   returned setting, pass its exact `Setting::name()` with a distinct valid
   value through `--limit`; observe that only that setting's bound changes
   at the operation boundary.
3. Pass a name not resolved by QSL `Setting::from_name`, a QSL setting not
   exposed by the selected operation, a malformed value and a duplicate
   setting. Observe each refusal before any stage.
4. Run an input that exceeds a chosen limit. Through the driver's typed
   execution wrapper, inspect the returned `LimitExceeded` configured bound
   and `setting()`; then repeat with the limit raised to fit the input.

Test location: `src/limits.rs`, `src/execute.rs`, `quire-cli/src/cli.rs`, and
`quire-cli/tests/binary_process.rs`.

## Expected Results

- The explicit value replaces only its field; omitted fields retain defaults.
- Every setting exposed by the operation is accepted under its exact QSL
  `Setting::name()` spelling and changes only the matching bound through
  QSL `SettingLimits::set_bound`.
- Unrecognized names, recognized settings not exposed by the operation,
  malformed values and duplicates are refused before any stage with QSL's
  refusal category; no driver-only alias is accepted.
- The low limit returns QSL's typed `LimitExceeded`; the driver wrapper
  preserves its configured bound and `setting()` and has category
  `Incomplete`. The process status is selected by QSL's category exit
  function. Raising the source-byte bound to the fixture's byte length allows
  the operation to complete.
