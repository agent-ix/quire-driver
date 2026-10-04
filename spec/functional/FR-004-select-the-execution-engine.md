---
id: FR-004
title: "Select the execution engine without substitution"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-296
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-279
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-306
    type: depends_on
---
# FR-004: Select the execution engine without substitution

## Description

For an execution request, the driver SHALL select the engine named by
`--engine interpreter|aot|jit`; when the option is absent, it SHALL select
`interpreter`. The driver SHALL run only the selected engine. It SHALL refuse
before any lifecycle stage when the selected engine is not linked into the
binary, and SHALL NOT substitute another engine.

## Inputs

- An execution request and optional `--engine` value.
- The execution engines linked into the binary.

## Outputs

- The selected engine's typed execution outcome, or a pre-stage refusal.

## Behavior

- If a caller supplies a value outside `interpreter`, `aot` and `jit`, then
  the CLI SHALL reject it as an unknown argument with exit status 20.
- If a caller selects a recognized engine not linked into the binary, then
  the CLI SHALL refuse before any stage as unsupported, naming the engine and
  using exit status 21.
- The driver SHALL invoke only the selected engine, including after a
  preparation failure or unsupported result.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | An execution request without `--engine` runs the interpreter and returns its result. | Test (TC-011) |
| FR-004-AC-2 | A value outside the closed engine set is refused before any stage with exit 20 and names the supplied value. | Test (TC-011) |
| FR-004-AC-3 | A recognized engine absent from the binary is refused before any stage with category unsupported and exit 21; no other engine runs. | Test (TC-011) |
| FR-004-AC-4 | When the selected engine returns unsupported or a typed preparation failure, that result is returned and no other engine is invoked. | Test (TC-011) |

## Dependencies

- QSL ADR-029 EB-8 and FR-296; QSpec FR-306.
- [TC-011](../test-cases/TC-011-execution-engine-selection.md).
