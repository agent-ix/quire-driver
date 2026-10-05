---
id: TC-019
title: "Propagate library cancellation and release the emitted source provision"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-011
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-019: Propagate library cancellation and release the emitted source provision

## Description

Verify the caller-facing library cancellation API, single-handle threading,
QSL-owned result mapping and E4 source-provision lifetime. This case does not
exercise process signals or timeouts; those belong to a separate CLI adapter.

Scope: FR-001-AC-9 through FR-001-AC-10, FR-011-AC-1 through FR-011-AC-5.

## Test Procedure

1. In a driver library integration test, create a caller-owned QSL `Cancel`
   handle and a test upstream that instruments the handle passed to `parse`,
   `select`, `check` and `package`. The test upstream uses the real QSL
   lifecycle entry points and delegates every stage's normal input to them.
2. For an already-cancelled handle with cause `Requested`, invoke the driver
   operation and require QSL's typed cancellation failure from the first
   stage, no E4 output, and no completed `Driven` value. Require that the
   caller's cancellation is observed by that stage and that no later stage
   is called.
3. For an in-flight cancellation, use a test work-meter observer to cancel
   a clone from a second thread after the selected stage begins charging.
   Run once with `Requested` and once with `Deadline`. Require QSL's typed
   cancellation failure with the supplied cause, no partial package and no
   completed `Driven` result. Verify cancellation through a clone of the
   caller's handle is observed by the active QSL stage and every preceding
   stage received that same shared cancellation state.
4. For a completed request, instrument E5 evidence consumption and the
   returned value. Require that the source provision is available to the E5
   check, then assert the returned `Driven` API carries the checked unit,
   emitted package bytes, admitted package and downstream values but no
   `EmittedUnit` or source-provision field. The test must not copy source
   bytes into `Driven` as a substitute for the released provision.
5. Independently pass the returned QSL cancellation outcome category to
   `Category::exit_code`; require QSL's CB-4 result for `incomplete`.
   Confirm the driver propagates `StageFailure::Cancelled(CancelCause)` as
   incomplete instead of sending it through the refusal-or-fault conversion
   for an unshared handle. Inspect the driver library and binary boundary to
   confirm no driver category conversion. Do not add or infer signal/timeout
   behavior.

## Expected Results

- The library exposes caller-controlled cancellation and threads one handle
  through QSL stages; cancellation stops the run and propagates QSL's typed
  failure with no partial E4 package.
- QSL owns the `incomplete` category and its CB-4 exit mapping; the driver
  adds no mapping.
- E5 consumes the E4 source provision before the driver releases it. The
  returned `Driven` result retains no `EmittedUnit` or source provision.
- No CLI signal or timeout behavior is claimed by this test.

Test location: planned driver library integration tests and API/lifetime
review.
