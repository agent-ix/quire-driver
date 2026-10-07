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

Verify that QSL parses CLI `--limit` operands, applies exposed settings to the
selected operation, omitted values keep their defaults, and a reached limit
remains a typed incomplete outcome. Accounting counters become caller-settable
as `accounting.<counter>` when QSL-656 lands; before then the every-bound goal
is not met and former bare accounting names are refused.

Scope: FR-003-AC-1 through FR-003-AC-10.

## Test Procedure

1. Inspect the parser and application path: `--limit` operands go through
   QSL `qsl_foundation::setting::parse_operands` and
   `qsl_replay::CallerLimits::from_operands`. Confirm the driver has no
   duplicate grammar, duplicate-detection, overflow or name-to-field rules.
2. Read the selected operation's QSL `SettingLimits::bounds()`. For every
   returned setting, pass its exact QSL name with a distinct valid value
   through `--limit`; observe that only that exposed bound changes at the
   operation boundary. Confirm a recognized setting absent from `bounds()` is
   refused with `invalid-request` and an operation-unavailable diagnostic,
   distinct from an unknown-name refusal.
3. Pass `normalization.work_units`, a name accepted by the current driver
   parser but not resolved by QSL, a malformed operand, a duplicate setting
   and a former bare accounting name such as `work_units`. Observe each
   refusal before any stage with code `invalid-request`; unknown-name,
   malformed, duplicate and accounting-name diagnostics identify their
   respective cause. After QSL-656 lands, pass every exact
   `accounting.<counter>` name and verify it is accepted; no bare-name
   compatibility alias is accepted.
4. On a 32-bit target, pass a value larger than `usize::MAX` and verify an
   `invalid-request` refusal without clamping. Run an input that
   exceeds a chosen limit. Through the driver's typed
   execution wrapper, inspect the returned `LimitExceeded` configured bound
   and `setting()`; then repeat with the limit raised to fit the input.

Test location: `src/limits.rs`, `src/execute.rs`, `quire-cli/src/cli.rs`, and
`quire-cli/tests/binary_process.rs`.

## Expected Results

- The explicit value replaces only its field; omitted fields retain defaults.
- Every setting exposed by the operation is accepted under its exact name
  through QSL's operand APIs and changes only the matching bound.
- Unknown names, malformed operands, duplicates and former bare accounting
  names are refused before any stage with code `invalid-request`.
- A QSL-recognized setting absent from the operation's bounds is refused with
  `invalid-request` and an operation-unavailable diagnostic, not the
  unknown-name diagnostic.
- No spelling or alias absent from QSL's parser is accepted; the former bare
  accounting names are not accepted before or after QSL-656, and namespaced
  `accounting.<counter>` operands are enabled when QSL-656 lands.
- A value too large for the platform-sized bound is refused with
  `invalid-request`, without clamping.
- The low limit returns QSL's typed `LimitExceeded`; the driver wrapper
  preserves its configured bound and `setting()` and has category
  `Incomplete`. The process status is selected by QSL's category exit
  function. Raising the source-byte bound to the fixture's byte length allows
  the operation to complete.
