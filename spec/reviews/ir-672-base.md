---
id: SR-038
title: "base checklist review of IR-672 adapter specification"
type: SpecReview
analysis: base
scope: "FR-005, FR-006, FR-013-AC-11/14/15, TC-005, TC-028, AD-001, spec/tests.md"
review_set: all
---

## Summary

The adapter ownership and shutdown requirements are stated across the functional requirements, test cases, matrix, and architecture record. The driver remains independent of the process host, while `quire-cli` owns composition and lifecycle handling. The process-to-CG mapping remains an explicit external dependency.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Windows Job Object assignment must precede plugin execution; this is now stated in FR-013 and verified by TC-028. | FR-013-AC-14..15, TC-028 |

## Checklist Notes

- Identifiers are unique and traceable; FR-006-AC-3..5 and FR-013-AC-14..15 have TC-028 coverage.
- TC-005 covers the direct CLI-to-host dependency and host-free driver closure; TC-028 covers adapter behavior.
- TC-028 distinguishes package-graph checks from lifecycle checks gated on CG IR-629.
- Unix uncatchable host termination and descendants escaping the host process group are stated exclusions.
- Quire validation passed after the requirement and test-case edits.
