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
operation SHALL use that setting's published QSL default. After QSL-656
exposes accounting settings, every QSL-enforced bound SHALL be
caller-configurable through the canonical QSL setting interface. The driver
supports 64-bit hosts. On a 32-bit host, QSL owns its documented bound
application behavior, including any clamp; the driver adds no platform
overflow rule. When an operation reaches a bound, it SHALL return QSL's typed
limit outcome and report no success.

## Inputs

- Zero or more `--limit` options containing a QSL setting operand and value.
- The lifecycle operation's published default limits.

## Outputs

- The operation's result under the effective limits, or its typed
  `LimitExceeded` outcome when a bound is reached.

## Behavior

- When the CLI receives `--limit` operands, the driver SHALL call QSL
  `qsl_replay::CallerLimits::from_operands`, which uses
  `qsl_foundation::setting::parse_operands`. QSL owns setting-name grammar,
  duplicate detection, value parsing and overflow handling. The driver SHALL
  NOT reimplement these rules or add aliases.
- While QSL-656 remains unmerged, the CLI SHALL accept only setting names
  recognized by the current QSL operand parser; accounting settings are not
  caller-settable during this interim.
- When QSL-656 lands, the driver SHALL use the accounting setting names
  defined by QSL FR-255 under `accounting.<counter>` without adding aliases.
- When applying a parsed setting to an operation, the driver SHALL check
  whether that setting appears in the selected operation's QSL
  `SettingLimits::bounds()`.
- If a recognized setting is absent from the selected operation's bounds,
  then the CLI SHALL refuse it before any stage with code `invalid-request`
  and a diagnostic that distinguishes an operation-unavailable setting from
  an unknown name.
- When a parsed setting is exposed by the selected operation, the driver SHALL
  apply it with QSL's `CallerLimits::set_bound` API after parsing through
  `CallerLimits::from_operands`.
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
| FR-003-AC-1 | Setting one QSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values. | Test (TC-010) |
| FR-003-AC-2 | A setting name not resolved by QSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand. | Test (TC-010) |
| FR-003-AC-3 | An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and QSL's incomplete category; the process status equals QSL's category exit function. Raising the same bound to fit the input allows the operation to complete. | Test (TC-010) |
| FR-003-AC-4 | The driver calls QSL `CallerLimits::from_operands` and maintains no duplicate grammar, alias table, or name-to-field mapping. | Inspection (TC-010) |
| FR-003-AC-5 | A setting parsed by QSL but absent from the selected operation's `SettingLimits::bounds()` is refused before any stage with code `invalid-request` and a distinct diagnostic naming the setting as unavailable for that operation. | Test (TC-010) |
| FR-003-AC-6 | Applying an accepted setting through QSL `CallerLimits::set_bound` changes only the matching bound; every other operation limit retains its prior value. | Test and Inspection (TC-010) |
| FR-003-AC-7 | A malformed `--limit` operand is refused before any stage with code `invalid-request` and identifies that operand. | Test (TC-010) |
| FR-003-AC-8 | A duplicate `--limit` setting is refused before any stage with code `invalid-request` and identifies the duplicate setting. | Test (TC-010) |
| FR-003-AC-9 | After QSL-656 lands, QSL resolves the accounting setting names defined by QSL FR-255 under the canonical `accounting.<counter>` namespace. | Test (TC-010) |
| FR-003-AC-10 | When QSL-656 lands and the selected operation's `SettingLimits::bounds()` includes a caller accounting setting, the driver applies that value through QSL `CallerLimits::set_bound` and passes it to the operation. | Test (TC-010) |

## Dependencies

- QSL ADR-029 LC-4, FR-255 and FR-277; QSpec FR-300's caller-supplied
  limits. QSL-656 adds the ten namespaced accounting settings required to
  meet the every-bound goal.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point and exit function.
- [TC-010](../test-cases/TC-010-caller-selected-limits.md).
