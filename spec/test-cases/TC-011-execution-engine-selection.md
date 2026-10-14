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
the caller-owned `ObjectEnvironment` unchanged and preserves QSL's typed
refusal when required object context is missing.

Scope: FR-004-AC-1 through FR-004-AC-7.

## Test Procedure

1. Run an execution request without `--engine` and observe which backend runs.
2. Run with an unknown engine name.
3. Run with a recognized engine that is not linked into the binary.
4. Run the linked interpreter with a source-byte bound that causes a typed
   preparation refusal. Compare the driver's exact typed refusal with a direct
   `qsl_replay::spine::run` using the same source, call and limits. Cross-engine
   non-substitution remains unverified until a second engine or injectable
   engine seam is available.
5. Through the public execution request, run a function whose result depends
   on an object supplied in a nonempty caller-owned `ObjectEnvironment`; run
   the same call with an empty environment and compare both driver outcomes
   with direct QSL runs using those exact environments.
6. Run a function that requires object context with that context absent from
   the caller-supplied environment; compare the driver's typed refusal with
   direct QSL and verify the driver does not substitute a default or derive
   context from display text or package bytes.

Test locations: `src/execute.rs`, `quire-cli/src/cli.rs`, and
`quire-cli/tests/binary_process.rs`.

## Expected Results

- The omitted option selects the interpreter.
- An unknown name is refused before any stage with QSL's refusal category.
- A recognized but unavailable engine is refused before any stage with exit
  status selected by QSL's category exit function and category unsupported.
- The linked interpreter's typed preparation refusal matches direct QSL
  output. No cross-engine invocation claim is made until a second engine or
  injectable seam is available.
- The nonempty caller environment reaches QSL unchanged: the driver's result
  matches the direct QSL result for that environment and differs from the
  direct QSL result using an empty environment.
- When required object context is absent, the driver returns the same typed
  QSL refusal as the direct run with the same environment; no default or
  derived context changes the outcome.
