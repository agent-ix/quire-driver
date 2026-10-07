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

Scope: FR-002-AC-1 through FR-002-AC-5 and FR-014-AC-4.

## Test Procedure

1. Invoke every listed verb with its valid format option, including `inspect`
   with `dot`.
2. Invoke an unknown verb, option, profile and provider; capture stdout,
   stderr, exit status and whether a lifecycle stage ran.
3. Exercise representative outcomes for every O-16 category, including
   `undefined`, a pending monitor clause, an inconclusive proof and a
   multi-item outcome containing multiple exit codes. Compare each process
   status to QSL's category exit function.
4. For the exit-only kit slice, verify the exit boundary calls
   `ix_cli_kit::exit::caller_exit_code` with QSL's `Category::exit_code`
   result. Exercise the kit function for every `u8`: compare its returned
   `ExitCode` to `ExitCode::from(code)` with `PartialEq` for every value other
   than `5`, including `0`, `1` through `4`, `6`, `10`, `20`, `21`, `22`,
   `30` and `255`. For `5`, assert the error's `rejected_code` and
   `reserved_code` fields both equal the specified values.
   Exercise the driver's reserved-code fallback through its test seam: it
   emits a diagnostic and returns QSL `Category::InternalFailure` code `30`.
   Adoption of the kit's `streams`, `json`, `config` and `version` modules is
   required but remains open under IR-607; this step does not claim those
   modules are adopted.
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
- Exit conversion uses `ix_cli_kit::exit::caller_exit_code` without mapping
  QSL categories through the kit's typed `Outcome` values. Input `5` is
  rejected with its reserved and rejected values; if the driver receives
  that mapping result, it reports the invariant failure and exits `30`.
- The parser remains owned by `quire-cli`. The other shared kit modules remain
  required under IR-607 and are not claimed as adopted by this exit-only slice.
- JSON mode emits one QSL-defined `quire-outcome/1` value equal to the typed
  library outcome. A stream write failure returns the internal-failure
  category through QSL's exit function; a broken pipe ends quietly with the
  outcome's code.
- An unnamed configuration file is not read.
