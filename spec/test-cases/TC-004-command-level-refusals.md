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
2. Build a registry from two manifests of `kani` with different digests.
3. Build a registry from a manifest advertising `not-a-kind`.

Test: `tests/drive.rs`, `tc_004_command_level_refusals_are_typed`.

## Expected Results

- Step 1 refuses at a compile stage, outcome kind refusal, and only the
  compile call was made.
- Step 2 refuses at the registry with `invalid_capability`/`duplicate-backend`.
- Step 3 refuses with `invalid_capability`/`unknown-kind`.
