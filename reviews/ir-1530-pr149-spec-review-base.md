---
id: SR-17940486709564847047960
title: "IR-1530 PR #149 spec-review"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #149 at 9aca0720, git diff origin/main...HEAD; IR-1530; spec/functional/FR-005-register-first-party-providers.md, spec/test-cases/TC-012-first-party-provider-interface.md, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-005
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-012
    type: reviews
---

## Summary

Ticket: IR-1530. Base spec-review of the one new AC, the TC-012 step and scope
change, and the spec/tests.md row. `quire validate` on FR-005 and TC-012 exits
0. Placement in FR-005 is consistent: AC-20 and AC-21 already hold the linked
Kani provider's execution behaviour, and the Verification cell names the tests,
TC-012 and CG FR-056-AC-15. TC-012's scope line was widened to AC-22 and its
expected results gained the matching line. Sub-analyses (EARS, integrity) were
not run: one AC was added, no requirement statement or relationship edge changed.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | TC-012 step 14 and the spec/tests.md row misstate what the native-route check observes. They say the native-route harness "reaches CG's execution (here, its `Execution` refusal for the missing launcher)"; the test mutates the generated identity, so CG refuses `HarnessProvenanceMismatch` before reading the crate or the launcher. Likewise "a launch attempt would be an `Execution` refusal" holds only loosely: with the nonexistent crate, CG refuses `HarnessNotInCrate` before any launch. State the actual refusal the step expects. | spec/test-cases/TC-012-first-party-provider-interface.md:87-96, spec/tests.md:78 |
| FND-002 | low | The new spec/tests.md row for FR-005-AC-22 is inserted between the AC-8 and AC-20..AC-21 rows, out of AC order. | spec/tests.md:78-79 |

## Verdict

Spec changes are well placed and traceable; fix FND-001 together with the
native-route test (SR-17940486709564847047958 FND-002). If the fix round gates
the scalar native path (SR-17940486709564847047958 FND-001), its AC belongs in
FR-009 next to FR-009-AC-7/AC-13, with its own TC-016 step.
