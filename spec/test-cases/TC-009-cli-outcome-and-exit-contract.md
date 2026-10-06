---
id: TC-009
title: "Exercise CLI verbs, outcome documents and exit codes"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-014
    type: verifies
---
# TC-009: Exercise CLI verbs, outcome documents and exit codes

## Description

Verify that the driver CLI exposes the specified verbs, preserves typed
outcomes in `quire-outcome/1` documents and uses QSL's single exit function.

Scope: FR-002-AC-1 through FR-002-AC-5.

## Test Procedure

1. Invoke every listed verb with its valid format option, including `inspect`
   with `dot`.
2. Invoke an unknown verb, option, profile and provider; capture stdout,
   stderr, exit status and whether a lifecycle stage ran.
3. Exercise representative outcomes for every O-16 category, including
   `undefined`, a pending monitor clause, an inconclusive proof and a
   multi-item outcome containing multiple exit codes. Compare each process
   status to QSL's category exit function.
4. Verify shared command/output, configuration and version mechanics use
   ix-cli-kit. Inspect the exit boundary with an outcome category that the
   kit's typed `Outcome` cannot represent and confirm the QSL-returned code is
   passed through unchanged.
5. Run one operation in JSON mode and compare the serialized value with the
   typed library outcome. Repeat with stdout/stderr write failure and a broken
   pipe.
6. Run from a directory containing an unnamed configuration file and verify
   that the CLI does not read it.

Test location: `quire-cli/tests/cli_contract.rs`; CLI-specific test fixtures
reside under `quire-cli/tests/fixtures/`. The parser and command construction
under test are owned by `quire-cli/src/cli.rs`.

## Expected Results

- Valid verbs and format options reach their corresponding operations.
- Invalid arguments are refused before any stage with QSL's refusal category
  and name the refused argument.
- Each category and multi-item result exits with the exact code selected by
  QSL's single exit function; no second mapping changes the result.
- Shared CLI mechanics use ix-cli-kit without mapping QSL categories through
  the kit's typed `Outcome` values.
- JSON mode emits one QSL-defined `quire-outcome/1` value equal to the typed
  library outcome. A stream write failure returns the internal-failure
  category through QSL's exit function; a broken pipe ends quietly with the
  outcome's code.
- An unnamed configuration file is not read.
