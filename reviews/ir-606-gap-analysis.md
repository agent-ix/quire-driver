---
id: SR-022
title: "Gap analysis of quire-driver PR #14 against FR-001 and TC-004"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@8ca0d9a66a845472a2adabd1a6d39801cad9e93c; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-004-command-level-refusals.md; src/upstream.rs; src/refusal.rs; src/drive.rs; src/handoff.rs; src/registry.rs; tests/drive.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-004, type: references }
---
# Gap analysis of quire-driver PR #14 against FR-001 and TC-004

## Summary

Ticket: IR-606. This repository-driven audit was scoped by the dispatching
brief to the diff `origin/main...HEAD` at 8ca0d9a. It included the
test-oracle-strength (semantic) review. Plan completion was not assessed.
All eight FR-001 criteria carry trace tags. However, FR-001-AC-5's
compile-limit branch lost its only test in this diff, and the regression
that removal hid is real (SR-021 FND-001). The one new test checks a
`DriverFault::Spine` value it builds itself, so it never runs the new
`driver_refusal` mapping.

## Verdict

**FAIL**: one `high` finding. FR-001-AC-5 shows as tagged and green, but
its limit clause is violated and nothing tests it.

Clean in scope: no stale trace tags (`quire coverage` reports no
`untracked_symbols`). `tc_008_a_missing_import_refuses_at_compile_intake` and
`tc_008_an_unknown_header_profile_refuses_with_its_cause` both run the
real `QslCg` lifecycle and assert stage, full `code/cause` and kind. The
changes to their fixtures match QSL's new header grammar, which has no
`version`/`digest` on `profile`/`import`. The parse-failure step of
`tc_004_command_level_refusals_are_typed` still asserts the stage, the code,
the kind and the call log. FR-001-AC-4 (step order) holds:
`tc_001_tc_006_steps_run_in_order_through_generation` still drives the
`Recorder` through the new `compile`, and that `compile` delegates to the
real `QslCg`.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | The diff deletes the only compile-stage limit test (TC-004 step 2) and adds no replacement, so FR-001-AC-5's limit clause is untested and its `code/cause` regression shipped undetected | tests/drive.rs:1371, FR-001-AC-5, TC-004 | correct-requirement-no-evidence |
| FND-002 | medium | No test executes `driver_refusal`. `tc_004_a_typed_spine_fault_is_a_compile_fault` builds `DriverFault::Spine { stage: Check }` itself, so the per-operation stage labels and the Limit/Fault/Cancelled split in upstream.rs are unverified | src/upstream.rs:30, src/refusal.rs:399 | correct-requirement-no-evidence |
| FND-003 | medium | The three new `RoutedGenerationError` arms (`DuplicateHarness`, `KaniRecordCountMismatch`, `KaniDuplicatePositionOutOfRange`) map to `runtime_invariant`/Fault, and their messages are new, but no test asserts their code, kind or stage | src/refusal.rs:234, src/refusal.rs:323 | correct-requirement-no-evidence |

### FND-001 detail

Nesting depth is no longer a QSL checking limit (ADR-030 D-1;
`CheckingLimits::new` now takes only `nodes`), so the old `not not not true`
at depth 3 step cannot be written any more. Deleting it is justified. Not
replacing it is not: bounds that can still be reached are easy to drive
through the public `SpineLimits` fields with no new dependency. Both probes
below ran in a throwaway copy of the tree:

- `limits.source.source_bytes = 8` gives `Compile(Source)`, `Limit`, `stage_limit_exceeded`
- `limits.checking = limits.checking.with_work_budget(1)` gives `Compile(Check)`, `Limit`, `stage_limit_exceeded`

Both show the `code/cause` regression that SR-021 FND-001 records. TC-004
still lists step 2 and its expected result
(`stage_limit_exceeded`/`nesting-depth-exceeded`, at the check stage, kind
limit) and names `tc_004_command_level_refusals_are_typed` as its test.

### FND-002 detail

Mutations no test catches:

- swap `SpineStage::Intake` for `SpineStage::Source` at src/upstream.rs:109
- label the `check` failure `SpineStage::Emit` at src/upstream.rs:119

QSL faults cannot be reached from caller input. A unit test in
`upstream.rs` that feeds `StageFailure::Fault`, `StageFailure::Cancelled`
and `StageFailure::Limit` straight into `driver_refusal` would bind the
mapping. The new test's `stage()` assertion passes through the value the
test set itself; only its `code()`/`kind()` assertions test real mapping,
through the existing `Self::Fault(_)` arm.

## Coverage

- Reconciliation: quoin matrix (quoin 0.28.1, quire 0.36.1 engine 0.50.1)
- Plan completion: not assessed
- Criteria: tagged 8 · untagged 29 · tagged-by-ignored-test 0 · method-without-symbol 0. All 8 tagged are FR-001's. The 29 untagged are FR-002…FR-010 (5, 3, 4, 3, 2, 2, 2, 5, 3). This diff does not touch them, and the dispatching brief puts them out of scope, so they do not drive this verdict.
- Evidence: no run evidence (the store has never been fed for this repo)
- `quire coverage` diagnostics: one existing `tag-on-non-binding-symbol` (`FR-075` on `drive` at src/drive.rs:93, unchanged by this diff) and the module's archetype-matches-nothing notices. Both are out of scope.
- Untraced behaviors / stubs: 0 in the diff. `DriverFault::Spine` falls under FR-001's "an upstream answer that breaks the driver's invariants is an internal fault".
- Semantic review: ran over 1 requirement (FR-001; AC-4, AC-5, AC-7 oracles read), as the test-oracle-strength pass the brief asked for.
