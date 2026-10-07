---
id: SR-044
title: "scope-boundary analysis of IR-672 adapter specification"
type: SpecReview
analysis: scope-boundary
scope: "FR-005, FR-006, FR-013, AD-001, TC-028"
review_set: all
---

## Summary

Responsibility is divided among QSL, the driver, the CLI adapter, and the process host: QSL owns registry vocabulary and cancellation; the driver owns its seam and proof settlement; `quire-cli` composes the host and handles CLI lifecycle; the host owns process containment and protocol.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The stated exclusions bound the signal guarantee to handled signals and descendants in the managed process group; callers should not read it as protection from uncatchable host termination. | FR-006, TC-028 |

## Analysis Notes

- The driver package remains host-free and does not define process origin or CG backend mapping.
- Windows Job Object creation, assignment, and kill-on-close are host responsibilities; CLI closes the owning session on exit.
