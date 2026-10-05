---
id: SR-031
title: "failure-domain review of IR-616 cancellation specification"
type: SpecReview
analysis: failure-domain
scope: "FR-011, TC-019, FR-001-AC-9..10, AD-001 cancellation/source lifetime sections"
review_set: all
---

## Summary

The requirement assigns cancellation initiation to the caller, polling to QSL, and propagation to the driver, and it defines a post-package cancellation boundary. The remaining failure-domain issue is the caller-visible error channel: the current driver result type does not carry QSL cancellation as an incomplete typed result.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Specify a public result variant/type for QSL cancellation, stage, and cause so a shared-handle cancellation cannot fall into the existing `DriverRefusal::Fault` path. | FR-001, FR-011-AC-2..3 |

## Analysis Notes

- The caller owns a cloneable handle and can cancel from another thread. QSL owns polling and the `incomplete` category; the driver says it does not poll independently.
- The scope ends at QSL `package`: cancellation after package completion does not interrupt routing or CG generation. The CLI signal/deadline trigger is explicitly outside this slice.
- TC-019 covers already-cancelled and in-flight cases, requested and deadline causes, no partial package, and no completed `Driven` result.
- Source lifetime remains a distinct boundary question: `CheckedUnit` is retained inside `Driven` and exposes its own source/library objects, even though the separate `EmittedUnit` is dropped.
