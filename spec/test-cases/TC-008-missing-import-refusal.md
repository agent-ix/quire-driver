---
id: TC-008
title: "A missing import refuses at compile intake"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-008: A missing import refuses at compile intake

## Description

Verify that the driver compiles with an empty `DependencyInput` (ADR-015
D-1), so a unit whose `import` names a library nothing supplies refuses at
the intake stage, and that `DriverRefusal::code()` renders that refusal's
`code/cause`, not just its bare code.

Scope: FR-001-AC-5.

## Test Procedure

1. Drive a unit that `import`s a library identity, with no library supplied
   for it (the driver never supplies one).

Tests: `tests/drive.rs`, `tc_008_a_missing_import_refuses_at_compile_intake`.

## Expected Results

- The run refuses at `DriverStage::Compile(SpineStage::Intake)`.
- The refusal's code is `missing_import/missing-selection`, not the bare
  `missing_import`.
- The outcome kind is a refusal, not a limit or a fault.
