---
id: SR-012
title: "Gap analysis of QSL-1 PR #7 against FR-001 and TC-006"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@9c1a9bdee9aaef95c6e95fdd829cd55f94dd31c1; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-001..TC-008; tests/drive.rs; tests/dependency_edges.rs; README.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-006, type: references }
---
# Gap analysis of QSL-1 PR #7 against FR-001 and TC-006

## Summary

Ticket: QSL-1. PR: quire-driver#7, reviewed at 9c1a9bd.

FR-001's acceptance criteria are unchanged by this PR. It changes FR-001's
Behavior prose, adds TC-006 step 9, and moves two intent tests from ignored
to active. Every AC is backed. The gap is in TC-006. Step 9's expected result
says a shape is unreachable, and a QSL unit that does reach it has no step or
test.

## Verdict

**PASS with a medium gap.** No criterion is missing a test and there are no
stubs. One TC expected result states a false invariant, and the real
reachable case is untested.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-006 step 9's expected result concludes that CG's `ResultBoundUnreachable` "is unreached from any unit this driver's own tests compile", on the grounds that QSL and CG "ask the same question over the same integers". Its test only shows that QSL refuses one shape: `x + 100` directly in the return context. No TC step or test covers a QSL-compiled unit that does reach the refusal. One exists: `function f using v(x: Int[0, 9]): Int[0, 10] pure { if x > 20 then x + 100 else 0 }` compiles, because QSL does not discharge the coercion in a dead branch. Its `+`, routed to Kani, settles `Unsupported`/`ResultBoundUnreachable` with `[100, 109]` against `[0, 10]` (measured). Failure scenario: AC-6's pass-through of this refusal, including no harness and the typed fields, is never asserted, and the spec tells readers the case cannot happen. Code-side detail is in SR-011 FND-001. | spec/test-cases/TC-006-generation-through-cg.md:41-42,89-101; tests/drive.rs:781-814 |

## Coverage

- Reconciliation: `quire coverage --scope . --json` found 0 unbacked rows,
  0 status lies and 0 untracked symbols.
- AC to TC to test:
  - AC-1: TC-005.
  - AC-2: TC-002.
  - AC-3: TC-003.
  - AC-4: TC-001.
  - AC-5: TC-004, TC-007 and TC-008.
  - AC-6: TC-006 and TC-007. This covers `tc_006_x_plus_1_settles_supported`,
    `tc_006_x_plus_y_settles_supported_with_independent_argument_bounds` (both
    now active), the two pass-through tests, and the new
    `tc_006_x_plus_100_...`.
  - AC-7: TC-004 and TC-006.
  - AC-8: TC-006.
- Stubs: none. `src/` is unchanged by this PR.
- Intent-test oracle strength: node id and argument ranges are pinned, but the
  operation identity and the result bound are not (SR-011 FND-004).

## Dispositions

Re-checked at 670c56e91c5d3cac64a07315f3a3ee2d8d876ae7.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 670c56e. TC-006 has a new step 10 (`if x > 20 then x + 100 else 0`) with an expected result. `tc_006_x_plus_100_in_a_dead_branch_settles_result_bound_unreachable` (tests/drive.rs:864) asserts three things. It asserts that the driver's output equals a direct `generate_routed` call, and that no harness is produced. It also asserts, by full equality, the typed disposition `Unsupported { ResultBoundUnreachable { "quire.op.integer.add", 0, 10, "100", "109" } }`. The test carries `/// FR-001-AC-6 (TC-006)`. Step 9's expected result no longer claims the refusal is unreachable. |
