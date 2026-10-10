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
remains a typed incomplete outcome. Until QSL-656 / QSL #665 lands, pinned QSL
does not recognize the bare accounting counter names as caller-settable; when
it lands, QSL's `parse_operands` owns name resolution and returns typed
`SettingName` values. The driver checks stage settings against the selected
operation's bounds and applies them with `CallerLimits::set_bound`. It applies
accounting settings with `AccountingLimits::set_bound` to `Call.accounting`;
accounting settings do not pass through stage bounds. The driver adds no
namespace or aliases. The driver supports 64-bit hosts; it adds no platform
overflow rule, and QSL owns bound application behavior on 32-bit hosts.

Scope: FR-003-AC-1 through FR-003-AC-10.

## Test Procedure

1. Inspect the parser and application path: the driver calls
   `qsl_foundation::setting::parse_operands` and branches on the returned
   `SettingName`. It checks only stage settings against operation bounds,
   applies those with `CallerLimits::set_bound`, and applies accounting
   settings with `AccountingLimits::set_bound` to `Call.accounting`. Confirm
   the driver has no duplicate grammar, duplicate-detection or name-to-field
   rules.
2. Read the selected operation's QSL `SettingLimits::bounds()`. For every
   returned stage setting, pass its exact QSL name with a distinct valid value
   through `--limit`; observe that only that exposed bound changes at the
   operation boundary. Confirm a recognized stage setting absent from
   `bounds()` is refused with `invalid-request` and an operation-unavailable
   diagnostic, distinct from an unknown-name refusal.
3. Pass the former driver-only name `normalization.work_units`, a malformed
   operand and a duplicate setting. Observe each refusal before any stage with
   code `invalid-request`; unknown-name, malformed and duplicate refusals
   identify their respective causes. When multiple operands are invalid,
   confirm QSL parser refusals are reported before checking whether recognized
   settings are available to the operation. Until QSL-656 / QSL #665 lands,
   pinned QSL treats bare accounting names such as `work_units` as unknown.
   After it lands, pass the bare names defined by QSL FR-255 and verify QSL's
   operand API resolves them to `SettingName::Accounting` values.
4. Apply a bare accounting operand such as `work_units=0`. Inspect that QSL
   `AccountingLimits::set_bound` changes `Call.accounting` while the stage
   limits remain unchanged, then run an input that exhausts that accounting
   budget and verify QSL reports the accounting limit. Separately, run an
   input that exceeds a chosen stage limit. Through the driver's
   typed execution wrapper, inspect the returned `LimitExceeded` configured
   bound and `setting()`; then repeat with the limit raised to fit the input.

Test location: `src/limits.rs`, `src/execute.rs`, `quire-cli/src/cli.rs`, and
`quire-cli/tests/binary_process.rs`.

## Expected Results

- The explicit value replaces only its field; omitted fields retain defaults.
- Every exposed stage setting is accepted under its exact name through QSL's
  operand API and changes only the matching stage bound.
- A QSL-resolved accounting setting changes `Call.accounting` through QSL's
  `AccountingLimits`; it leaves stage limits unchanged and reaches `spine::run`.
- Names not resolved by the pinned QSL parser, malformed operands and
  duplicates are refused before any stage with code `invalid-request`.
- A QSL-recognized setting absent from the operation's bounds is refused with
  `invalid-request` and an operation-unavailable diagnostic, not the
  unknown-name diagnostic.
- The driver adds no name parser, namespace or aliases. QSL-656 / QSL #665
  enables QSL's bare accounting names defined by FR-255.
- QSL owns platform-sized bound application; the driver adds no overflow rule.
- The low limit returns QSL's typed `LimitExceeded`; the driver wrapper
  preserves its configured bound and `setting()` and has category
  `Incomplete`. The process status is selected by QSL's category exit
  function. Raising the source-byte bound to the fixture's byte length allows
  the operation to complete.
