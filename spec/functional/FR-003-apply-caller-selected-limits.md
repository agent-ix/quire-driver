---
id: FR-003
title: "Apply caller-selected lifecycle limits"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-277
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-255
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-275
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-300
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-002
    type: depends_on
---
# FR-003: Apply caller-selected lifecycle limits

## Description

When a caller supplies one or more `--limit` options, the driver SHALL use
QSL's operand parser and typed caller limits to apply those values to the
selected lifecycle operation. When a caller omits a limit setting, the
operation SHALL use that setting's published QSL default. After QSL-656's
accounting-setting support lands, every QSL-enforced bound SHALL be
caller-configurable. Until then, the ten accounting counters are not
configurable through `--limit`, and the every-bound goal is not met. When an
operation reaches a bound, it SHALL return QSL's typed limit outcome and
report no success.

## Inputs

- Zero or more `--limit` options containing a QSL setting operand and value.
- The lifecycle operation's published default limits.

## Outputs

- The operation's result under the effective limits, or its typed
  `LimitExceeded` outcome when a bound is reached.

## Behavior

- When the CLI receives `--limit` operands, the driver SHALL pass them to QSL
  `qsl_foundation::setting::parse_operands` and
  `qsl_replay::CallerLimits::from_operands`. QSL owns setting-name grammar,
  duplicate detection, value parsing and overflow handling. The driver SHALL
  NOT reimplement these rules or add aliases.
- While QSL-656 remains unmerged, the CLI SHALL accept only setting names
  recognized by the current QSL operand parser; accounting settings are not
  caller-settable during this interim.
- When a caller supplies one of the ten accounting counters by its former bare
  name before QSL-656 lands, the CLI SHALL refuse it before the operation
  begins.
- When QSL-656 lands, the driver SHALL accept each exact
  `accounting.<counter>` name from QSL's `quire.value.accounting/v1` settings
  without adding a bare-name alias. The ten counters are `work_units`,
  `integer_bits`, `decimal_digits`, `scale_expansion`, `text_input_bytes`,
  `text_scalars`, `normalized_scalars`, `unit_edges`, `value_occurrences` and
  `result_units`.
- When applying a parsed setting to an operation, the driver SHALL check
  whether that setting appears in the selected operation's QSL
  `SettingLimits::bounds()`.
- If a recognized setting is absent from the selected operation's bounds,
  then the CLI SHALL refuse it before any stage with code `invalid-request`
  and a diagnostic that distinguishes an operation-unavailable setting from
  an unknown name.
- When a parsed setting is exposed by the selected operation, the driver SHALL
  use QSL's typed setting application API.
- When a parsed value cannot fit the selected operation's platform-sized
  bound, the CLI SHALL refuse it before any stage with code `invalid-request`
  and preserve the caller's original value in the diagnostic; the operation
  receives no clamped value.
- The driver SHALL pass the computed limits values to the selected operation.
- The driver SHALL preserve each explicit caller value when building the
  computed limits values.
- A reached limit SHALL retain QSL's typed cause, limit kind, configured
  value and field identity in the machine outcome.
- A limit outcome SHALL retain QSL's incomplete category and use QSL's
  category exit function.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-003-AC-1 | Setting one QSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values. The ten accounting bounds remain outside this criterion until QSL-656 lands. | Test (TC-010) |
| FR-003-AC-2 | A setting name not resolved by QSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand. | Planned (TC-010; current driver accepts `normalization.work_units`, which QSL does not resolve) |
| FR-003-AC-3 | An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and QSL's incomplete category; the process status equals QSL's category exit function. Raising the same bound to fit the input allows the operation to complete. | Test (TC-010) |
| FR-003-AC-4 | The driver passes `--limit` operands to QSL `parse_operands` and `CallerLimits::from_operands` and maintains no duplicate grammar, alias table, or name-to-field mapping. | Inspection (TC-010) |
| FR-003-AC-5 | A setting parsed by QSL but absent from the selected operation's `SettingLimits::bounds()` is refused before any stage with code `invalid-request` and a distinct diagnostic naming the setting as unavailable for that operation. | Test (TC-010) |
| FR-003-AC-6 | Applying an accepted setting changes only the matching bound through QSL's typed setting application API; every other operation limit retains its prior value. | Test and Inspection (TC-010) |
| FR-003-AC-7 | A malformed `--limit` operand is refused before any stage with code `invalid-request` and identifies that operand. | Test (TC-010) |
| FR-003-AC-8 | A duplicate `--limit` setting is refused before any stage with code `invalid-request` and identifies the duplicate setting. | Test (TC-010) |
| FR-003-AC-9 | After QSL-656 lands, `work_units`, `integer_bits`, `decimal_digits`, `scale_expansion`, `text_input_bytes`, `text_scalars`, `normalized_scalars`, `unit_edges`, `value_occurrences` and `result_units` are caller-settable through their exact `accounting.<counter>` names; former bare names remain refused. | Planned (TC-010; QSL-656) |
| FR-003-AC-10 | On a target where a requested value exceeds `usize::MAX`, the CLI refuses it before any stage with code `invalid-request` and does not clamp it. | Test (TC-010; 32-bit target) |

## Dependencies

- QSL ADR-029 LC-4, FR-255 and FR-277; QSpec FR-300's caller-supplied
  limits. QSL-656 adds the ten namespaced accounting settings required to
  meet the every-bound goal.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point and exit function.
- [TC-010](../test-cases/TC-010-caller-selected-limits.md).
