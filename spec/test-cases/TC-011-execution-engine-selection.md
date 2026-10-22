---
id: TC-011
title: "Select only the requested execution engine"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-004
    type: verifies
---
# TC-011: Select only the requested execution engine

## Description

Verify the `--engine` default, closed set, unavailable-engine refusal and
no-substitution behavior. Also verify that the selected interpreter receives
the caller-owned `ObjectEnvironment` unchanged for public library requests,
preserves QSL's actual outcome when object context is absent, including both
QSL-defined `Undefined` and typed-refusal cases, and explicitly applies the
current CLI's empty-environment caller policy.

Scope: FR-004-AC-1 through FR-004-AC-10.

## Test Procedure

1. Run an execution request without `--engine` and observe which backend runs.
2. Run with an unknown engine name.
3. Run with a recognized engine that is not linked into the binary.
4. Run the linked interpreter with a source-byte bound that causes a typed
   preparation refusal. Compare the driver's exact typed refusal with a direct
   `qsl_replay::spine::run` using the same source, call and limits.
5. Exercise the current private callback through the same selection and
   dispatch function called by public `execute` with two designated failures:
   QSL `RunRefusal::UnsupportedResult` and a typed preparation refusal. For
   each case, assert the exact refusal and QSL document are returned and the
   callback ran exactly once. This callback count alone SHALL NOT be reported
   as proof of no substitution.
6. Inspect `src/execute.rs`: confirm the callback type returns only QSL run
   success or `RunRefusal`, including `RunRefusal::UnsupportedResult` whose
   category is `Unsupported`. Distinguish this selected-engine result from
   `ExecutionFailure::UnsupportedEngine`, which is produced before callback
   invocation when an engine is not linked into the binary.
7. Full AC-4 verification remains blocked until a second production engine
   or upstream-supported shared dispatch contract makes every reachable
   engine invocation observable through the same test boundary. At that
   boundary, inject `RunRefusal::UnsupportedResult` and a typed preparation
   failure from the selected engine; assert exact result preservation, one
   selected-engine invocation, and zero invocations of every alternative
   engine. An added direct engine call outside the counted boundary SHALL make
   this test fail; if the seam cannot enforce that property, AC-4 remains
   Partial.
8. Inspect `src/execute.rs` and `quire-cli/src/cli.rs`: confirm public
   `execute` calls the same private dispatch function with QSL
   `spine::run`, and neither the public API nor CLI, configuration, or
   environment can select the test callback.
9. Inspect `quire-cli/src/cli.rs`: confirm the CLI invocation exposes no
   object-context input and its run handler explicitly supplies an empty
   `ObjectEnvironment` to the public execution request.
10. Through the public library execution request, run a function whose result
   depends on an object supplied in a nonempty caller-owned
   `ObjectEnvironment`; run the same call with an empty environment and
   compare both driver outcomes with direct QSL runs using those exact
   environments.
11. Run a function with a population argument whose identity has no binding in
   the caller-supplied environment; compare the driver's typed refusal with
   direct QSL, as specified by QSL FR-089-AC-4.
12. Run a `lookup<T>(p, r) absent undefined` query whose bound population has
   no matching key; compare the driver's `Undefined` outcome with direct QSL,
   as specified by QSL FR-090.

Test locations: `src/execute.rs`, `quire-cli/src/cli.rs`, and
`quire-cli/tests/binary_process.rs`.

## Expected Results

- The omitted option selects the interpreter.
- An unknown name is refused before any stage with QSL's refusal category.
- A recognized but unavailable engine is refused before any stage with exit
  status selected by QSL's category exit function and category unsupported.
- The CLI run handler explicitly supplies an empty environment under its
  caller policy; the public library request requires its caller to provide an
  environment, and the nonempty public-library case is tested separately.
- Callback-injected tests can verify exact QSL failure preservation through
  the current shared dispatch function for both `RunRefusal::UnsupportedResult`
  and a typed preparation refusal. Their callback counts do not establish
  that an alternate engine was never invoked.
- Full no-substitution verification remains Partial until a second production
  engine or upstream-supported shared dispatch boundary can represent the
  selected-engine result and make all engine invocations observable; the
  oracle checks exact failure, one selected invocation, and zero alternative
  invocations, and fails if any invocation bypasses it.
- Inspection confirms the callback is private, public `execute` supplies QSL
  `spine::run`, and no API/CLI/configuration/environment/default surface
  selects a callback.
- The nonempty caller environment reaches QSL unchanged: the driver's result
  matches the direct QSL result for that environment and differs from the
  direct QSL result using an empty environment.
- When a population argument names an identity absent from the environment,
  the driver returns the same typed QSL refusal as the direct run.
- When an `absent undefined` lookup has no matching key, the driver returns
  QSL's `Undefined` outcome unchanged. The driver does not replace QSL's
  operation-specific outcome with a refusal or another outcome.
- The driver does not create a default environment or derive context from
  display text or package bytes.
