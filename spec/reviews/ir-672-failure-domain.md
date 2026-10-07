---
id: SR-047
title: "failure-domain analysis of IR-672 adapter specification"
type: SpecReview
analysis: failure-domain
scope: "FR-005, FR-006, FR-013 Windows process lifecycle, TC-028"
review_set: all
---

## Summary

The spec defines normal close, Linux/macOS handled-signal cancellation, and bounded cleanup. Windows Job Object behavior is explicitly planned under IR-674 because the merged host's standard-library path lacks kill-on-close; no current Windows cleanup claim is made.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Windows kill-on-close is not available in the merged host path; retain it as planned work under IR-674 and require native Windows evidence before claiming support. | FR-013-AC-14, TC-028 |

## Analysis Notes

- Windows startup failure observation is static analysis until a distinguishable or injectable seam and native Windows runner exist.
- Linux/macOS signal handling covers sessions starting, running, and shutting down; cleanup has a one-second bound after forced termination begins.
- Process-to-CG mapping is deferred to CG IR-629 and is not inferred here.
