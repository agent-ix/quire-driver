---
id: SR-6007
title: "STD-200 spec-review/integrity of FR-010, TC-017 and tests.md traceability"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-driver PR #84 @488bd1d; spec/tests.md, spec/test-cases/TC-017-only-bound-items-reach-codegen.md, spec/functional/FR-010-send-only-bound-items-to-codegen.md; ticket STD-200"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/TC-017
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-010
    type: reviews
---
# STD-200: spec-review/integrity

## Summary

Ticket: STD-200. This review checks that the diff's index and traceability edits are consistent. In scope are the FR-010 row (line 86) and TC-017 row (line 132) of spec/tests.md, TC-017's Scope line and vector label, and FR-010's AC table and Dependencies. `make spec` (`quire validate 'spec/**/*.md'`) exits 0, with only pre-existing module warnings (DuplicateArchetype, DuplicateInverseEdge, semantic.inline-data-schema). `quoin matrix` exits 0 and lists FR-010-AC-1..AC-3 only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | spec/tests.md (the FR-010 and TC-017 rows) and TC-017 (Scope and the "Partition vector (FR-010-AC-4)" label) trace to FR-010-AC-4, which FR-010 does not define. Every index entry this PR adds points at a criterion that does not exist. | spec/tests.md:86; spec/tests.md:132; spec/test-cases/TC-017-only-bound-items-reach-codegen.md:17,26 |

## Examined scope

```yaml
scope:
  - {id: tests.md-FR-010-row, path: spec/tests.md, role: examined, excerpt: "| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4 | TC-017 | Planned in quire-integration; QSL AR2 prerequisite |"}
  - {id: tests.md-TC-017-row, path: spec/tests.md, role: examined, excerpt: "| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4 | Planned in quire-integration; QSL AR2 prerequisite |"}
  - {id: TC-017, path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md, role: examined, excerpt: "Scope: FR-010-AC-1 through FR-010-AC-4."}
  - {id: FR-010, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: context_only, excerpt: "Acceptance Criteria: FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 (no AC-4)."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: spec-review/integrity
    check: trace
    artifact_id: tests.md-FR-010-row
    related_ids: [tests.md-TC-017-row, TC-017, FR-010]
    path: spec/tests.md
    lines: "86"
    excerpt: |-
      | FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4 | TC-017 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
    finding: "spec/tests.md (the FR-010 and TC-017 rows) and TC-017 (Scope and the 'Partition vector (FR-010-AC-4)' label) trace to FR-010-AC-4, which FR-010 does not define. Every index entry this PR adds points at a criterion that does not exist."
    test_id: TC-017
    claimed_ac: FR-010-AC-4
    correct_ac: "none (FR-010-AC-4 undefined)"
bindings:
  - {test_id: TC-017, ac_id: FR-010-AC-4, trace: wrong}
```

## Verdict

Not mergeable until FR-010-AC-4 is added (base SR-6005 FND-001). Once it is, these index rows become correct as written. Tooling limitation: neither `quire validate` nor `quoin matrix` flags an index or TC reference to an undefined AC id, so a green `make spec` does not show that the traceability is sound.
