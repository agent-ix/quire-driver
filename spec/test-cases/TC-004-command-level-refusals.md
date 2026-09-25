---
id: TC-004
title: "Command-level refusals are typed"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-004: Command-level refusals are typed

## Description

Verify the step, catalog code and outcome kind of the command-level
refusals.

Scope: FR-001-AC-5.

## Test Procedure

1. Drive a unit that does not parse.
2. Drive a unit whose body `not not not true` is four deep, with a checking
   depth limit of 3.
3. Build a registry from two manifests of `kani` with different digests.
4. Build a registry from a manifest advertising `not-a-kind`.
5. Take a `DriverRefusal::Fault` for a `supported` settlement outside its
   candidates, and one for a vocabulary mismatch.

Tests: `tests/drive.rs`, `tc_004_command_level_refusals_are_typed`;
`src/refusal.rs`, `tc_004_a_fault_is_an_internal_fault_with_runtime_invariant`.

## Expected Results

- Step 1 refuses at the source stage with `invalid_syntax`, outcome kind
  refusal, and only the compile call was made.
- Step 2 refuses at the check stage with `stage_limit_exceeded`, outcome
  kind limit.
- Step 3 refuses at the registry with `invalid_capability`/`duplicate-backend`.
- Step 4 refuses with `invalid_capability`/`unknown-kind`.
- Step 5's faults have outcome kind fault and code `runtime_invariant`, at
  the negotiation and conversion steps.
