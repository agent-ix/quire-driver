---
id: SR-046
title: "base checklist review of IR-672 adapter specification"
type: SpecReview
analysis: base
scope: "FR-005, FR-006, FR-013-AC-11/14/15, TC-005, TC-028, AD-001, spec/tests.md"
review_set: all
---

## Summary

The adapter ownership and shutdown requirements are stated across the functional requirements, test cases, matrix, and architecture record. The driver remains independent of the process host, while `quire-cli` owns composition and lifecycle handling. Process-to-CG mapping remains an explicit external dependency, and Windows kill-on-close is marked planned under IR-674 because the merged host cannot provide it through its standard-library path.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The merged host cannot meet Windows kill-on-close through its current standard-library Job Object path; FR-013-AC-14 and the architecture row now mark the mechanism planned under IR-674. | FR-013-AC-14, AD-001 |

## Checklist Notes

- Identifiers are unique and traceable; FR-006-AC-3..4 and FR-013-AC-14..15 have TC-028 coverage.
- TC-005's host-free driver closure is already tested; its direct CLI-to-host edge assertion remains planned for the code slice. TC-028 covers existing package layout plus planned adapter behavior.
- TC-028 gates only end-to-end provider routing on CG IR-629; package and fixture lifecycle checks are independent.
- Unix uncatchable host termination and descendants escaping the host process group are stated exclusions.
- Quire validation exits 0 with 0 grammar findings; it emits seven ambient catalog notices (DuplicateArchetype x5, inline-data-schema, DuplicateInverseEdge).
