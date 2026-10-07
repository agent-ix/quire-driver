---
id: SR-050
title: "evidence analysis of IR-672 adapter specification"
type: SpecReview
analysis: evidence
scope: "FR-006-AC-3..4, FR-013-AC-14..15, TC-005, TC-028"
review_set: all
---

## Summary

TC-028 names platform-specific outcomes for normal shutdown and Linux/macOS SIGINT/SIGTERM. Windows Job Object evidence remains planned under IR-674 and requires a native Windows runner; current Ubuntu-only CI is not such evidence. TC-005 supplies static dependency evidence.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Preserve separate Linux/macOS signal receipts and Windows Job Object receipts; a generic cross-platform pass would not prove the platform-specific requirements, and Windows evidence is not presently available. | TC-028 |

## Analysis Notes

- Signal cases cover plugin startup, active request, and shutdown waiting for exit.
- Windows setup-failure evidence is inspection-only until a failure seam and native Windows runner are available.
- The planned lifecycle fixture must expose descendant liveness and provider-request markers for the asserted outcomes.
