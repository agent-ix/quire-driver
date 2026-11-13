---
id: SR-17940486709564847047750
title: "IR-1494 PR #148 FR-020 spec review"
type: SpecReview
analysis: base
date: 2026-11-12
scope: "agent-ix/quire-driver PR #148 at 5438a5ac; IR-1494; spec/functional/FR-020, spec/test-cases/TC-034, spec/tests.md, spec/spec.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-034
    type: reviews
---

## Summary

Ticket: IR-1494. Base spec review of the FR-020 status flips. `quire validate 'spec/**/*.md'`: the two pre-existing FR-019 EARS warnings only. AC-9 now names all eight CG variants (matches CG 7fb4f53c). AC-10 is retired with strike-through and Inspection, mirroring CG FR-022-AC-38. The TC-034 mutant table adds the guard-root and `GuardContextMissing` mapping rows, both of which the reviewer reproduced as killed. Dependencies text matches CG main. The stale-Planned items are recorded in the gap-analysis SR.

## Verdict

Two wording defects; neither blocks on its own, but FND-001 should be fixed with the gap-analysis findings in this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-020-AC-11 names its vectors as `if y > 5 then y + 1 else 0` ("then and else") and `if z + 1 > 5 then z + 1 else 0`, but with `else 0` there is no else item, so "negated for the else item" cannot hold; the test uses `else y + 1` and `else z + 1`. Make the AC name the vectors the test runs. | spec/functional/FR-020-project-requirement-records-into-codegen-input.md:191, tests/drive.rs:4121-4124 |
| FND-002 | low | FR-020-AC-13 says "a driver request that omits a guard root yields this refusal over real CG", but the test keeps the driver's positions and rewrites the item's requirement (through the test seam) to name a guard rooted only in another item's span. Say that, as the PR body does. | spec/functional/FR-020-project-requirement-records-into-codegen-input.md:193, tests/drive.rs:4199-4276 |

## Dispositions

Round 1, head 32113a0e. `quire validate 'spec/**/*.md'`: the two pre-existing FR-019 warnings only.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-11 now names `if y > 5 then y + 1 else y + 1` (then and else item) and `if z + 1 > 5 then z + 1 else z + 1`, matching the test. |
| FND-002 | fixed | AC-13 now says real CG given an item whose requirement and record name a guard rooted only in another item's span refuses the whole call. |
