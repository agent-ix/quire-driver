---
id: SR-010
title: "Gap analysis of QSL-1 against FR-001 and its test cases"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@46ea01228ce32ac661ba4be4097992be8ed0c6ef; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-001..TC-008; tests/drive.rs; tests/dependency_edges.rs; src/refusal.rs; src/registry.rs; src/handoff.rs; src/upstream.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-004, type: references }
  - { target: ix://agent-ix/quire-driver/TC-005, type: references }
  - { target: ix://agent-ix/quire-driver/TC-008, type: references }
---
# Gap analysis of QSL-1 against FR-001 and its test cases

## Summary

Ticket: QSL-1. PR: quire-driver#6, reviewed at 46ea012. This repo has no
plan bundle, so this checks FR-001's acceptance criteria against the TCs and
tagged tests, plus changed code against its owning requirement.

Every behavior the PR changes has an owner:

- the empty `DependencyInput` belongs to FR-001 Description step 1, AC-5,
  TC-008 and `tc_008_a_missing_import_refuses_at_compile_intake`;
- the diagnostics-catalog accessor belongs to FR-001 Status and AC-7
  through the E5 tests;
- `serde_json` as a dev-dependency belongs to AC-1, TC-005 and
  `tc_005_driver_edges_match_the_adr_011_allow_list`;
- the pass-through and IR-302 intent tests belong to AC-6 and TC-006
  steps 7 and 8.

The AC-5 wording against the Check-stage cause is recorded once, as SR-009
FND-001. It is not repeated here.

## Verdict

**PASS with low gaps.** No criterion is missing a test, and there are no
stubs, but two spec and trace gaps remain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | TC-004 has not been updated for the registry dedup behavior the PR adds and tests. Step 3's procedure and expected result still describe only the two-manifest case and the `duplicate-backend` code. `tc_004_command_level_refusals_are_typed` now also asserts the full refusal vector and an A/B/A case (one refusal per distinct digest). No FR-001 text states one refusal per distinct digest either. Failure scenario: the test enforces a behavior no spec states, so someone who "simplifies" the test back to a code check has no spec row to violate. | spec/test-cases/TC-004-command-level-refusals.md:23,43; tests/drive.rs:959-991; src/registry.rs:27-40 |
| FND-002 | low | `quire coverage --scope . --json` reports FR-001-AC-1 unbacked (7 of 8 FR-001 criteria backed). TC-005's `FR-001-AC-1 (TC-005)` tag sits in the `//!` module doc, and `tc_005_driver_edges_match_the_adr_011_allow_list` has no `///` tag. The gap predates this PR, but this PR rewrites AC-1 and edits that test. Failure scenario: the coverage rollup reports the dependency-edge criterion as untested even though a real test backs it, so a coverage gate on this repo would fail or be waived. | tests/dependency_edges.rs:2,114-115; spec/functional/FR-001-orchestrate-the-driver-steps.md:109 |

## Coverage

- Reconciliation: `quire coverage --scope . --json` found 0 unbacked matrix
  rows, 0 status lies and 0 untracked symbols. The FR-001 acceptance-criterion
  group is 7 of 8 backed; the unbacked one is AC-1 (FND-002).
- AC to TC to test:
  - AC-1: TC-005, tc_005 (the tag placement issue is FND-002).
  - AC-2: TC-002.
  - AC-3: TC-003.
  - AC-4: TC-001.
  - AC-5: TC-004, TC-007 and TC-008, the last new with its test.
  - AC-6: TC-006 and TC-007, plus the two new pass-through tests and two
    ignored intent tests.
  - AC-7: TC-004 and TC-006.
  - AC-8: TC-006.
- Every test name that TC-006 and TC-008 cite exists in tests/drive.rs.
- FR-001 Description step 1 states the empty `DependencyInput` (prior item 9).
- FR-001 and TC-006 name IR-302 in one Status paragraph and one Expected
  Results bullet respectively.
- Stubs: none in src/ or tests/. Changed behavior with no owning
  requirement: none.
- Semantic check:
  - The IR-302 intent tests assert the TC-006 steps 7 and 8 intended
    `Supported` outcome, with bounds [(0,9),(0,9)] and [(0,9),(10,20)].
  - The pass-through tests assert full `RoutedGeneration` equality with a
    direct `generate_routed` call. A mutation that clears `claim_map` fails
    both (see SR-009).
