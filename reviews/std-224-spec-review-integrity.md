---
id: SR-6010
title: "STD-224 spec-review/integrity of AC numbering and tests.md traceability"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-driver PR #85 @970b890; spec/tests.md, spec/test-cases/TC-017-only-bound-items-reach-codegen.md, spec/functional/FR-001-orchestrate-the-driver-steps.md, spec/functional/FR-009-settle-execute-and-replay-results.md, spec/functional/FR-010-send-only-bound-items-to-codegen.md; ticket STD-224"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/TC-017
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-001
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-009
    type: reviews
---
# STD-224: spec-review/spec-integrity-analysis

## Summary

Ticket: STD-224. This review checks AC numbering, relationships and traceability. FR-001-AC-17 follows AC-16 and FR-009-AC-16 follows AC-15, with no gaps or reuse. TC-017 adds `verifies` edges to FR-001 and FR-009, and FR-010 adds `depends_on` edges to them. TC-017's Scope line and the spec/tests.md coverage and TC rows name exactly the new ACs. `make spec` (`quire validate 'spec/**/*.md'`, quire 0.36.2 / engine 0.50.2) exits 0, with only the pre-existing module warnings (DuplicateArchetype, DuplicateInverseEdge, semantic.inline-data-schema). `quoin matrix --json` (quoin 0.28.3) exits 0 and resolves FR-001-AC-17, FR-009-AC-16 and FR-010-AC-3..AC-5 as method Test, 'untagged' and 'no run evidence', which is expected for planned quire-integration tests.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | spec/tests.md is updated in the coverage table but not elsewhere. The test-location row still reads 'FR-010 / TC-017' although TC-017 now also verifies FR-001-AC-17 and FR-009-AC-16. The new FR-001-AC-17 row sits at line 86 among the FR-009 rows, away from the FR-001 row (line 46, AC-1..AC-16). Move it beside the FR-001 row and extend the location row. | spec/tests.md:30,46,86 |
| FND-002 | low | FR-001-AC-17 bundles two scenarios (the A/B/C vector and the all-refused run) in one criterion, so one result can fail for two reasons. The A/B/C vector is now asserted three times (FR-001-AC-17, FR-010-AC-4's new clause, FR-009-AC-16), each with a different slice. Splitting the all-refused half into its own AC (beside FR-010-AC-5) would keep the criteria atomic. | spec/functional/FR-001-orchestrate-the-driver-steps.md:249 |
| FND-003 | low | The commit drops FR-009's trailing newline (diff shows '\ No newline at end of file' on the TC-016 Dependencies line). This is an unrelated byte change; restore the newline. | spec/functional/FR-009-settle-execute-and-replay-results.md:205 |

## Examined scope

```yaml
clean: false
scope:
- id: tests-FR-001-AC-17
  path: spec/tests.md
  role: examined
  excerpt: '| FR-001 | FR-001-AC-17 | TC-017 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |'
- id: tests-FR-009-AC-16
  path: spec/tests.md
  role: examined
  excerpt: '| FR-009 | FR-009-AC-16 | TC-017 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |'
- id: tests-TC-017
  path: spec/tests.md
  role: examined
  excerpt: '| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, FR-010-AC-5, FR-001-AC-17, FR-009-AC-16 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |'
- id: tests-location
  path: spec/tests.md
  role: examined
  excerpt: '| FR-010 / TC-017 | Planned QSL-owned cross-repository test in `quire-integration`, QSL-514; QSL AR2 is the prerequisite. |'
- id: TC-017-scope
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: examined
  excerpt: 'Scope: FR-010-AC-1 through FR-010-AC-5, FR-001-AC-17, FR-009-AC-16.'
- id: FR-001-AC-17
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Given a package whose export returns a bound result for A and C and a refusal for B, the run completes, `Driven` holds the settlement of A and C and B's refusal in the per-item refusal member, and no `DriverRefusal` is returned; with every item refused, `Driven` holds no settled item and one refusal per requested item.
- id: FR-009-AC-16
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: Given a partly bound package, the outcome holds one terminal record for each item in the CG request and none for an export-refused item, whose refusal remains in `Driven`'s per-item refusal member.
- id: FR-010-AC-4
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: Given an export that returns a bound result for a clause item A, a refusal for an extent-domain item B over an unbound population, and a bound result with an empty binding set for an item C that references no model element, the CG request carries A and C and not B, and B's refusal is in the per-item refusal member beside the settlement of A and C.
- id: FR-009-eof
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: '- [TC-016](../test-cases/TC-016-execute-replay-and-terminal-records.md).  (no trailing newline)'
- id: tests-FR-001-main
  path: spec/tests.md
  role: context_only
  excerpt: '| FR-001 | FR-001-AC-1..FR-001-AC-16 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007, TC-008, TC-019 | 🚧 Partial: ...'
findings:
- fnd: FND-001
  severity: low
  confidence: high
  method: spec-review/spec-integrity-analysis
  check: trace
  artifact_id: tests-location
  related_ids:
  - tests-FR-001-AC-17
  - tests-FR-001-main
  path: spec/tests.md
  lines: 30,46,86
  excerpt: '| FR-010 / TC-017 | Planned QSL-owned cross-repository test in `quire-integration`, QSL-514; QSL AR2 is the prerequisite. |'
  finding: spec/tests.md is updated in the coverage table but not elsewhere. The test-location row still reads 'FR-010 / TC-017' although TC-017 now also verifies FR-001-AC-17 and FR-009-AC-16. The new FR-001-AC-17 row sits at line 86 among the FR-009 rows, away from the FR-001 row (line 46, AC-1..AC-16). Move it beside the FR-001 row and extend the location row.
- fnd: FND-002
  severity: low
  confidence: medium
  method: spec-review/spec-integrity-analysis
  check: compound
  artifact_id: FR-001-AC-17
  related_ids:
  - FR-010-AC-4
  - FR-009-AC-16
  - FR-010-AC-5
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  lines: '249'
  excerpt: Given a package whose export returns a bound result for A and C and a refusal for B, the run completes, `Driven` holds the settlement of A and C and B's refusal in the per-item refusal member, and no `DriverRefusal` is returned; with every item refused, `Driven` holds no settled item and one refusal per requested item.
  finding: FR-001-AC-17 bundles two scenarios (the A/B/C vector and the all-refused run) in one criterion, so one result can fail for two reasons. The A/B/C vector is now asserted three times (FR-001-AC-17, FR-010-AC-4's new clause, FR-009-AC-16), each with a different slice. Splitting the all-refused half into its own AC (beside FR-010-AC-5) would keep the criteria atomic.
- fnd: FND-003
  severity: low
  confidence: high
  method: spec-review/spec-integrity-analysis
  check: other
  artifact_id: FR-009-eof
  related_ids: []
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  lines: '205'
  excerpt: '- [TC-016](../test-cases/TC-016-execute-replay-and-terminal-records.md).  (no trailing newline)'
  finding: The commit drops FR-009's trailing newline (diff shows '\ No newline at end of file' on the TC-016 Dependencies line). This is an unrelated byte change; restore the newline.
bindings:
- test_id: TC-017
  ac_id: FR-001-AC-17
  trace: correct
- test_id: TC-017
  ac_id: FR-009-AC-16
  trace: correct
- test_id: TC-017
  ac_id: FR-010-AC-4
  trace: correct
```

## Recommendations

- Move the FR-001-AC-17 row next to FR-001's, extend the location row, split AC-17's all-refused half, restore FR-009's trailing newline.

## Verdict

Mergeable on integrity grounds: numbering and the matrix resolve; three low findings.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The new FR-001-AC-19 bundles two independent scenarios (a whole-export refusal and an unknown driver-built key) with different outcome kinds. This is the same atomicity pattern FND-002 split out of AC-17. It also names no step for the refusal case. Split it into two criteria. | spec/functional/FR-001-orchestrate-the-driver-steps.md:275 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix 86fe419: / FR-001 / FR-001-AC-17..FR-001-AC-19 / TC-017 / ... (now directly after the FR-001 AC-1..AC-16 row)  /  / FR-010, FR-001-AC-17..FR-001-AC-19, FR-009-AC-16 / TC-017 / Planned QSL-owned cross-repository test ... / |
| FND-002 | fixed | Fix 86fe419: / FR-001-AC-17 / For the FR-010-AC-4 vector, the run completes, ... / / / FR-001-AC-18 / When the export refuses every requested item, `Driven` holds no settled item and one refusal in the per-item refusal member for each requested item. / ; FR-010-AC-4: 'This is the vector FR-001-AC-17 and FR-009-AC-16 cite.' |
| FND-003 | fixed | Fix 86fe419: - [TC-016](../test-cases/TC-016-execute-replay-and-terminal-records.md).  (trailing newline restored) |
| FND-004 | fixed | Fix 01ca369: / FR-001-AC-19 / An export that fails as a whole refuses the run with the export's typed refusal, and the refusal carries the bindings-export step. / / / FR-001-AC-20 / An occurrence key the driver built that the export reports as naming no requirement record is a `runtime_invariant` internal fault. / / / FR-001-AC-21 / When the package declares no abstraction relation, the driver does not call the export, every item goes to CG, and the per-item refusal member is empty. / |
