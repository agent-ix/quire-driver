---
id: SR-032
title: "integrity review of IR-616 cancellation specification"
type: SpecReview
analysis: integrity
scope: "FR-011, TC-019, FR-001 changed sections, AD-001, spec index and TM-001"
review_set: all
---

## Summary

The new trace links and matrix rows are consistent, with TC-019 covering the new acceptance criteria. FR-011 groups two independently observable behaviors and leaves its public cancellation result representation underspecified relative to FR-001's existing result contract.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Separate or explicitly justify the independent caller-cancellation behavior and post-E5 source-lifetime behavior currently bundled in FR-011. | FR-011, TC-019 |
| FND-002 | medium | State how the library's public result represents a cancelled stage and cause; FR-011 promises QSL's typed incomplete failure while FR-001 currently exposes `Result<Driven, DriverRefusal>`, whose cancellation path is a fault. | FR-001, FR-011 |

## Analysis Notes

- Traceability is complete for the changed artifacts: FR-011 is indexed; TC-019 verifies FR-011 and FR-001-AC-9/10; AD-001 and TM-001 reference both.
- Cancellation and source retention can be tested and implemented independently. The requirement title describes cancellation, while AC-4 adds a separate lifetime contract.
- The current public driver model is concrete context for the missing result-channel contract: `DriverRefusal` has `Compile` and `Fault` paths, and `QslCg::compile` routes `StageFailure` through QSL `refusal_or_fault`, which maps `Cancelled` to an internal fault.
