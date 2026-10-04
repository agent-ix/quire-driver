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
no-substitution behavior.

Scope: FR-004-AC-1 through FR-004-AC-4.

## Test Procedure

1. Run an execution request without `--engine` and observe which backend runs.
2. Run with an unknown engine name.
3. Run with a recognized engine that is not linked into the binary.
4. Use a test engine that returns unsupported and one that returns a typed
   preparation failure; observe all engine invocations and the final outcome.

Test location: planned driver CLI integration tests.

## Expected Results

- The omitted option selects the interpreter.
- An unknown name is refused before any stage with exit 20.
- A recognized but unavailable engine is refused before any stage with exit
  21 and category unsupported.
- A selected engine's unsupported or preparation result is returned without
  invoking another engine.
