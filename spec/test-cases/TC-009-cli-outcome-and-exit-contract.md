---
id: TC-009
title: "Exercise CLI verbs, outcome documents and exit codes"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: verifies
---
# TC-009: Exercise CLI verbs, outcome documents and exit codes

## Description

Verify that the driver CLI exposes the specified verbs, preserves typed
outcomes in `quire-outcome/1` documents and uses QSL's single exit function.

Scope: FR-002-AC-1 through FR-002-AC-4.

## Test Procedure

1. Invoke every listed verb with its valid format option, including `inspect`
   with `dot`.
2. Invoke an unknown verb, option, profile and provider; capture stdout,
   stderr, exit status and whether a lifecycle stage ran.
3. Exercise representative outcomes for every O-16 category, including
   `undefined`, a pending monitor clause, an inconclusive proof and a
   multi-item outcome containing multiple exit codes. Compare each process
   status to QSL's category exit function.
4. Run one operation in JSON mode and compare the document with the typed
   library outcome. Repeat with stdout/stderr write failure and a broken pipe.
5. Run from a directory containing an unnamed configuration file and verify
   that the CLI does not read it.

Test location: planned driver CLI integration tests.

## Expected Results

- Valid verbs and format options reach their corresponding operations.
- Invalid arguments are refused before any stage with exit 20 and name the
  refused argument.
- Each category and multi-item result exits with the exact code selected by
  QSL's single exit function; no second mapping changes the result.
- JSON mode emits exactly one `quire-outcome/1` document preserving the
  operation's typed values. A stream write failure exits 30; a broken pipe
  ends quietly with the outcome's code.
- An unnamed configuration file is not read.
