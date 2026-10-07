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

When a caller supplies one or more `--limit` options, the driver SHALL apply
those named values to the matching fields of the typed limits passed to the
selected lifecycle operation. When a caller omits a limit field, the
operation SHALL use that field's published QSL default. Every QSL-enforced
bound SHALL be caller-configurable. When an operation reaches a bound, it SHALL
return QSL's typed limit outcome and report no success.

## Inputs

- Zero or more `--limit` options naming a QSL limits field and its value.
- The lifecycle operation's published default limits.

## Outputs

- The operation's result under the effective limits, or its typed
  `LimitExceeded` outcome when a bound is reached.

## Behavior

- If a caller supplies an unknown limit field, malformed value or duplicate,
  then the CLI SHALL reject it before the operation begins with QSL's refusal
  category.
- The driver SHALL resolve each `--limit` name with QSL's
  `qsl_foundation::Setting::from_name`
  and SHALL use the resulting typed setting, rather than a driver-owned name
  alias or name-to-field table, when applying the value.
- The driver SHALL use the resulting typed setting with the selected
  operation's QSL `qsl_foundation::SettingLimits::set_bound` to select the
  bound to change.
  If QSL recognizes a setting but the selected operation does not expose it,
  the CLI SHALL reject the option before the operation begins with QSL's
  refusal category.
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
| FR-003-AC-1 | Setting one valid `--limit` overrides only its named field; every omitted field retains its published default, and the effective limits passed to the operation equal those values. | Test (TC-010) |
| FR-003-AC-2 | A `--limit` name that `qsl_foundation::Setting::from_name` does not resolve, a malformed value or a duplicate `--limit` is refused before any stage with QSL's refusal category and names the option value. | Test (TC-010) |
| FR-003-AC-3 | An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and QSL's incomplete category; the process status equals QSL's category exit function. Raising the same bound to fit the input allows the operation to complete. | Test (TC-010) |
| FR-003-AC-4 | Each accepted `--limit` name is an exact name resolved by `qsl_foundation::Setting::from_name`; the driver accepts no spelling or alias that QSL does not resolve. | Test and Inspection (TC-010) |
| FR-003-AC-5 | A setting recognized by QSL but not exposed by the selected operation is refused before any stage with QSL's refusal category and names the option value. | Test (TC-010) |
| FR-003-AC-6 | Applying an accepted setting changes only the bound selected by QSL `qsl_foundation::SettingLimits::set_bound`; every other operation limit retains its prior value. | Test and Inspection (TC-010) |

## Dependencies

- QSL ADR-029 LC-4, FR-255 and FR-277; QSpec FR-300's caller-supplied
  limits.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point and exit function.
- [TC-010](../test-cases/TC-010-caller-selected-limits.md).
