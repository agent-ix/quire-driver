---
id: SR-049
title: "dependency analysis of IR-672 adapter specification"
type: SpecReview
analysis: dependency
scope: "FR-005, FR-006, FR-013, FR-014, TC-005, TC-028"
review_set: all
---

## Summary

The adapter depends on the host implementation and CLI package boundary. CG IR-629 is a named prerequisite for process-provider invocation, while package graph checks remain independently testable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Process-provider lifecycle tests cannot execute end to end until CG IR-629 provides the process BackendKind mapping; TC-028 explicitly marks these checks planned. | FR-006, TC-028 |

## Analysis Notes

- `quire-cli` composes `quire-plugin-host`; `quire-driver` excludes that dependency.
- QSL owns origin vocabulary and registry admission. The driver passes descriptors through without constructing CG backend kinds.
- Windows Job Object mechanism is owned by IR-674; signal initiation and active-session closure are CLI-adapter responsibilities.
