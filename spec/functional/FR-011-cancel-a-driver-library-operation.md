---
id: FR-011
title: "Cancel a driver library operation through a caller-owned handle"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-driver/FR-001
    type: references
  - target: ix://agent-ix/quire-driver/TC-019
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-276
    type: depends_on
---
# FR-011: Cancel a driver library operation through a caller-owned handle

## Description

The driver library SHALL accept a caller-owned, cloneable QSL `Cancel` handle
for its compile-and-drive operation and pass that same handle to each QSL
lifecycle stage it invokes: `parse`, `select`, `check` and `package`. The
caller may cancel through any clone, including from another thread while one
of those stages is running. The driver SHALL propagate QSL's typed cancelled
failure to its caller without converting it to a driver-authored category.
This cancellation guarantee covers those QSL calls; it does not interrupt
the driver's later routing or CG generation after `package` has completed.

The caller-visible category remains QSL-owned: a cancelled operation is
`incomplete` under ADR-029 CB-4. The binary obtains its process code from
QSL's one `Category::exit_code` function, as FR-002 specifies; this
requirement defines no exit table or mapping.

This requirement specifies the library API first so realtime and embedded
callers can cancel without a process signal or timer. A CLI signal/deadline
adapter is a separate thin frontend layer: it may own signal handling or a
timer and cancel the same handle with the corresponding QSL cause. That CLI
adapter is planned separately; this requirement claims no signal handler,
timeout, timer, or signal-to-cause mapping.

After E5 has consumed the E4 `EmittedUnit` source provision as evidence, the
driver SHALL release that `EmittedUnit` and its source provision. `Driven`
retains the checked unit, emitted package, admitted package and downstream
results already specified by FR-001; it does not retain or expose the source
provision. A caller that needs its original source bytes retains its own
input. This avoids keeping a second source provision alive with the returned
run result.

## Inputs

- The normal driver request and limits specified by FR-001.
- A caller-owned, cloneable QSL `Cancel` handle.

## Outputs

- The normal completed `Driven` result when all stages complete.
- The typed driver compile failure that preserves QSL's cancelled stage and
  cause when cancellation stops a lifecycle stage; no completed `Driven`
  result or partial E4 package is returned for that failed call.

## Behavior

- The driver SHALL neither create a private replacement handle nor mask a
  cancellation failure from QSL.
- The driver SHALL propagate `StageFailure::Cancelled(CancelCause)` as the
  caller-visible `incomplete` result. It SHALL NOT pass this failure through
  the QSL refusal-or-fault conversion that treats an unshared cancellation
  handle as an internal fault.
- QSL polls cancellation at its stage/work charges and stops within one
  charge; the driver does not poll independently or decide when to cancel.
- After QSL `package` completes, a later cancellation does not stop routing
  or CG generation under this requirement.
- The library does not read a clock. A caller or separate CLI frontend that
  owns a deadline requests cancellation using QSL's `Deadline` cause.
- The driver SHALL use the E4 source provision only through the E5 hand-off,
  then release `EmittedUnit`; retaining caller-owned `CompileInput` bytes is
  the caller's responsibility.
- The CLI's signal and timeout adapter remains planned and outside this
  library requirement's implementation scope.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-011-AC-1 | The public driver library accepts one caller-owned `Cancel` token and passes that same token to `parse`, `select`, `check` and `package` in order. | Test |
| FR-011-AC-2 | When the caller cancels the token during a QSL lifecycle stage, the driver returns the typed cancellation failure with its cause; it returns no `Driven` result or partial package. | Test |
| FR-011-AC-3 | The cancellation's caller-visible category is QSL's `incomplete`, and the binary obtains the process code only through QSL `Category::exit_code`; the driver adds no conversion. | Test |
| FR-011-AC-4 | After E5 hand-off, the returned `Driven` value does not retain or expose `EmittedUnit` or its source provision; the QSL E5 evidence check still receives that provision before it is released. | Test |
| FR-011-AC-5 | CLI signal and deadline triggers are outside this specification slice; a later frontend adapter uses the caller's `Cancel` token to request cancellation. | Inspection |

## Dependencies

- QSL ADR-029 LC-3 cancellation and CB-4 outcome mapping.
- QSL FR-275 operation request shape, FR-276 cancellation, and FR-285
  category exit function.
- Driver FR-001 compile sequence and E5 hand-off.

## Status

Specified; implementation and CLI signal/deadline integration remain
planned.
