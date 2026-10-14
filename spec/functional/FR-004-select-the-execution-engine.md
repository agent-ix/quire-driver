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
  - target: ix://agent-ix/quire-spec-language/FR-089
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-306
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-002
    type: depends_on
---
# FR-004: Select the execution engine without substitution

## Description

For an execution request, the driver SHALL select the engine named by
`--engine interpreter|aot|jit`; when the option is absent, it SHALL select
`interpreter`. The driver SHALL run only the selected engine. It SHALL refuse
before any lifecycle stage when the selected engine is not linked into the
binary, and SHALL NOT substitute another engine.

The public execution request SHALL carry a caller-owned QSL
`ObjectEnvironment`. The selected interpreter SHALL pass that same environment
unchanged to QSL `spine::run`. If the called function requires object context
that is absent from the supplied environment, the driver SHALL return QSL's
typed refusal unchanged; it SHALL NOT supply a default environment or derive
context from display text or package bytes.

## Inputs

- An execution request and optional `--engine` value.
- The caller-owned typed QSL `ObjectEnvironment` for the execution request.
- The execution engines linked into the binary.

## Outputs

- The selected engine's typed execution outcome, or a pre-stage refusal.
- QSL's unchanged typed refusal when execution requires object context absent
  from the supplied environment.

## Behavior

- If a caller supplies a value outside `interpreter`, `aot` and `jit`, then
  the CLI SHALL reject it as an unknown argument with QSL's refusal category.
- If a caller selects a recognized engine not linked into the binary, then
  the CLI SHALL refuse before any stage as unsupported, naming the engine.
- The driver SHALL invoke only the selected engine, including after a
  preparation failure or unsupported result.
- The driver execution boundary SHALL require the caller-owned
  `ObjectEnvironment`; it SHALL pass the same value unchanged to QSL
  `spine::run` for the selected interpreter.
- If a function requires object context absent from the supplied environment,
  the driver SHALL return QSL's existing typed refusal unchanged.
- The driver SHALL NOT create a default environment.
- The driver SHALL NOT infer object context from display text or package
  bytes.
- The driver SHALL NOT replace QSL's typed refusal with a different outcome.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | An execution request without `--engine` runs the interpreter and returns its result. | Test (TC-011) |
| FR-004-AC-2 | A value outside the closed engine set is refused before any stage with QSL's refusal category and names the supplied value. | Test (TC-011) |
| FR-004-AC-3 | A recognized engine absent from the binary is refused before any stage with category unsupported; no other engine runs, and process status equals QSL's category exit function. | Test (TC-011) |
| FR-004-AC-4 | When the selected engine returns unsupported or a typed preparation failure, that result is returned and no other engine is invoked. | Test (TC-011) |
| FR-004-AC-5 | The public execution request carries the caller-owned typed QSL `ObjectEnvironment`, and the selected interpreter passes that same value unchanged to QSL `spine::run`. | Test |
| FR-004-AC-6 | Given a function whose result depends on object context, a nonempty caller environment produces the QSL result for that context and differs from the result with an empty environment; the driver derives no context and supplies no default. | Test |
| FR-004-AC-7 | Given a function that requires object context absent from the caller-supplied environment, the driver returns QSL's existing typed refusal unchanged and does not substitute another outcome. | Test |

## Dependencies

- QSL ADR-029 EB-8 and FR-296; QSpec FR-306.
- QSL `qsl-semantics::model::object_environment::ObjectEnvironment` is the
  typed evaluation context. QSL owns its contents and semantics; the driver
  carries the caller's value to the run boundary.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point and exit function.
- [TC-011](../test-cases/TC-011-execution-engine-selection.md).
