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
  - target: ix://agent-ix/quire-spec-language/FR-090
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
unchanged to QSL `spine::run`. The driver SHALL return QSL's actual typed
execution outcome unchanged when evaluation uses the supplied environment.
QSL outcomes for absent context depend on the operation: an absent lookup may
produce `Undefined` under QSL FR-090, while an unresolved population identity
produces QSL's typed refusal under FR-089. The driver SHALL NOT supply a
missing value for the public library request or derive context from display
text or package bytes. Since the current CLI invocation exposes no
object-context input, its handler SHALL explicitly supply an empty
`ObjectEnvironment` as that caller's policy.

## Inputs

- An execution request and optional `--engine` value.
- The caller-owned typed QSL `ObjectEnvironment` for a public library request;
  the current CLI supplies an empty environment under its explicit caller
  policy because it exposes no object-context input.
- The execution engines linked into the binary.

## Outputs

- The selected engine's typed execution outcome, or a pre-stage refusal.
- QSL's unchanged typed execution outcome, including its defined `Undefined`
  and typed-refusal cases.

## Behavior

- If a caller supplies a value outside `interpreter`, `aot` and `jit`, then
  the CLI SHALL reject it as an unknown argument with QSL's refusal category.
- If a caller selects a recognized engine not linked into the binary, then
  the CLI SHALL refuse before any stage as unsupported, naming the engine.
- The driver SHALL invoke only the selected engine, including after a
  preparation failure or unsupported result.
- When a test verifies no substitution after a selected engine returns a typed
  failure, it SHALL inject only the engine-invocation callback into the same
  private selection and dispatch function called by the public `execute`
  function. The test SHALL observe the callback invocation count and the exact
  typed failure returned by that function; a separate selector, a constructed
  `ExecutionFailure`, or a helper that does not traverse this shared dispatch
  function SHALL NOT count as evidence for AC-4.
- When the public `execute` function runs in production, the driver SHALL
  supply QSL `spine::run` as the engine-invocation callback.
- When tests inject a callback to verify AC-4, the driver SHALL keep that
  injection private and SHALL NOT expose it through the public API, CLI,
  configuration, environment, or a production default.
- The driver execution boundary SHALL require the caller-owned
  `ObjectEnvironment`; it SHALL pass the same value unchanged to QSL
  `spine::run` for the selected interpreter.
- The public library request SHALL not synthesize or default its required
  `ObjectEnvironment`. The current CLI handler SHALL explicitly pass an empty
  `ObjectEnvironment` under its caller policy because the CLI exposes no
  object-context input.
- The driver SHALL preserve QSL's actual typed outcome when an object lookup
  is absent, including QSL's `Undefined` outcome where FR-090 specifies it.
- If a population argument names an identity with no binding in the supplied
  environment, the driver SHALL return QSL's existing typed refusal unchanged,
  as specified by FR-089-AC-4.
- A public library request SHALL NOT default its required environment.
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
| FR-004-AC-6 | Given a function whose result depends on object context, a nonempty caller environment produces the QSL result for that context and differs from the result with an empty environment; the public library request derives no context or default. | Test |
| FR-004-AC-7 | Given a QSL-defined absent object lookup whose QSL result is `Undefined`, the driver returns that `Undefined` outcome unchanged and does not replace it with a refusal or another outcome. | Test |
| FR-004-AC-8 | Given a population argument whose identity has no binding in the caller-supplied environment, the driver returns QSL's existing typed refusal unchanged, as required by QSL FR-089-AC-4. | Test |
| FR-004-AC-9 | The current CLI handler explicitly supplies an empty `ObjectEnvironment` as its caller policy because its invocation surface has no object-context input; public library callers still must supply the required environment themselves. | Inspection (TC-011) |
| FR-004-AC-10 | The no-substitution test seam injects only the engine invocation callback into the private selection and dispatch function shared with public `execute`; the test verifies the exact typed failure and one invocation, while the public API, CLI, configuration, environment, and production default expose no callback selector. | Inspection (TC-011) |

## Dependencies

- QSL ADR-029 EB-8 and FR-296; QSpec FR-306.
- QSL `qsl-semantics::model::object_environment::ObjectEnvironment` is the
  typed evaluation context. QSL owns its contents and semantics; the driver
  carries the caller's value to the run boundary.
- QSL FR-089-AC-4 defines typed refusal for an unresolved population identity;
  QSL FR-090 defines `Undefined` for the absent-lookup mode that requests it.
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point and exit function.
- [TC-011](../test-cases/TC-011-execution-engine-selection.md).
