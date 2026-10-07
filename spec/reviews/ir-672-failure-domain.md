---
id: SR-039
title: "failure-domain analysis of IR-672 adapter specification"
type: SpecReview
analysis: failure-domain
scope: "FR-005, FR-006, FR-013 Windows process lifecycle, TC-028"
review_set: all
---

## Summary

The spec defines normal close, handled-signal cancellation, bounded cleanup, and Windows Job Object failure behavior. It distinguishes host-start refusal from provider execution and scopes guarantees around uncatchable host termination and escaped Unix process groups.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Windows containment depends on assigning the child before plugin execution; FR-013 and TC-028 now make that ordering observable. | FR-013, TC-028 |

## Analysis Notes

- A Job Object creation or assignment failure returns a typed start failure and dispatches no provider request.
- Signal handling covers sessions starting, running, and shutting down; cleanup has a one-second bound after forced termination begins.
- Process-to-CG mapping is deferred to CG IR-629 and is not inferred here.
