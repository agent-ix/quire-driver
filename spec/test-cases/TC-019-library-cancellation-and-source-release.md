---
id: TC-019
title: "Propagate library cancellation and retain bounded checked-unit source"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-011
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-019: Propagate library cancellation and retain bounded checked-unit source

## Description

Verify the caller-facing library cancellation API, per-stage propagation,
QSL-owned result mapping, and the retained source lifetime after E5. This
case does not exercise process signals or timeouts; those belong to a
separate CLI adapter.

Scope: FR-001-AC-9 through FR-001-AC-10, FR-011-AC-1 through FR-011-AC-5.

## Test Procedure

1. In a driver library integration test, create a caller-owned QSL `Cancel`
   handle and a test upstream that instruments the handle passed to `parse`,
   `select`, `check` and `package`. Use the real QSL lifecycle entry points
   and delegate each stage's normal input to them.
2. For an already-cancelled handle with cause `Requested`, invoke the driver
   operation and require `DriverRefusal::Cancelled` for `SpineStage::Source`
   with cause `Requested`, no E4 output, and no completed `Driven` value. Require that the
   caller's cancellation is observed by that stage and that no later stage
   is called.
3. For each lifecycle stage (`parse`, `select`, `check`, `package`), use its
   test work-meter observer to cancel a clone from a second thread after that
   stage begins charging. Run each stage once with `Requested` and once with
   `Deadline`. Require the driver's public cancellation variant to name the
   stage under test and preserve the supplied cause. Require no E4 package,
   no completed `Driven` value, and no later stage call. This is an eight-row
   stage-by-cause matrix; do not substitute a single selected stage.
4. For a completed request, instrument E5 evidence consumption and inspect
   the returned `Driven.package`. Require E5 to receive the E4
   `EmittedUnit` source provision, and require the returned `CheckedUnit` to
   retain its root source and library-source accessors for diagnostic loci.
   Under the current empty `DependencyInput`, require no resolved library
   source and root-source bytes no greater than the request's
   `SpineLimits.source.source_bytes`. Confirm the returned value contains no
   `EmittedUnit` source provision, so the E4 copy is released after E5.
5. Independently obtain the QSL category for
   `StageFailure::Cancelled(cause)` and pass it to `Category::exit_code`;
   require QSL's CB-4 result for `incomplete`. Confirm the driver returns
   `DriverRefusal::Cancelled { stage, cause }` instead of sending
   `StageFailure::Cancelled(CancelCause)` through the refusal-or-fault
   conversion for an unshared handle. Inspect the binary boundary to confirm
   it uses QSL `Category::exit_code` without a second mapping. Do not add or
   infer signal/timeout behavior.

## Expected Results

- The library exposes caller-controlled cancellation and threads one handle
  through each QSL stage; cancellation returns `DriverRefusal::Cancelled`
  with stage and cause, with no partial E4 package.
- QSL owns the `incomplete` category and its CB-4 exit mapping; the driver
  adds no mapping.
- E5 consumes the E4 source provision before the driver releases it. The
  returned `Driven.package` retains the bounded `CheckedUnit` source; under
  the current empty dependency input it contains only the root source.
- No CLI signal or timeout behavior is claimed by this test.

Test location: `tests/drive.rs::tc_019_cancellation_preserves_every_stage_and_cause`
and `tests/drive.rs::tc_019_e5_receives_source_provision_and_checked_unit_retains_bounded_root`;
the public API and ownership boundary are also subject to review.
