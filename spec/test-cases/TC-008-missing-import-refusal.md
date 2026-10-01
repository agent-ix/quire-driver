---
id: TC-008
title: "A missing import or an unknown header profile refuses at compile"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-008: A missing import or an unknown header profile refuses at compile

## Description

Verify that the driver compiles with an empty `DependencyInput` (ADR-015
D-1), so a unit whose `import` names a library nothing supplies refuses at
the intake stage, and that `DriverRefusal::code()` renders that refusal's
`code/cause`, not just its bare code.

It also verifies that a header `profile` selecting an identity in no catalog
refuses at the assembly stage as `CompileRefusal::Profile` (QSL FR-110), and
that its code renders the first refusal's cause.

Scope: FR-001-AC-5.

## Test Procedure

1. Drive a unit that `import`s a library identity, with no library supplied
   for it (the driver never supplies one).
2. Drive a unit whose header `profile` selects `test:unknown-profile`.

Tests: `tests/drive.rs`, `tc_008_a_missing_import_refuses_at_compile_intake`
and `tc_008_an_unknown_header_profile_refuses_with_its_cause`.

## Expected Results

- The run refuses at `DriverStage::Compile(SpineStage::Intake)`.
- The refusal's code is `missing_import/missing-selection`, not the bare
  `missing_import`.
- The outcome kind is a refusal, not a limit or a fault.
- For step 2, the refusal is `CompileRefusal::Profile` at
  `DriverStage::Compile(SpineStage::Assembly)`, its code is
  `unknown_profile/unsupported-selection`, and the outcome kind is a
  refusal.
