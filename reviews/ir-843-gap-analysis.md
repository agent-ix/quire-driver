---
id: SR-5843
title: "Gap analysis of IR-843 changed requirements, tests, and code"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@1c9f16196592633ef8e74269c367674ae2def9be; changed files src/provider.rs, Cargo.lock, spec/functional/FR-005-register-first-party-providers.md, spec/functional/FR-009-settle-execute-and-replay-results.md, spec/test-cases/TC-012-first-party-provider-interface.md, spec/test-cases/TC-016-execute-replay-and-terminal-records.md, spec/tests.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-005, type: references }
  - { target: ix://agent-ix/quire-driver/FR-009, type: references }
---
# Gap analysis of IR-843 changed requirements, tests, and code

## Summary

Ticket: IR-843. This repository-driven, planless audit is limited to the exact PR diff and its four acceptance criteria. All four criteria have tagged tests and the implementation uses the upstream contracts, but FR-005-AC-21's tagged test does not exercise the production evidence-retention path; plan completion was not assessed.

## Scope

- **FR-005-AC-20** (`spec/functional/FR-005-register-first-party-providers.md`): The driver forwards `KaniExecutionLimits.home_directory` unchanged to CG's `KaniExecutionRequest`; if it is `None`, the driver passes `None` even when the process environment has a `HOME` value. The driver does not infer the path from other directories or from the selected Kani installation.
- **FR-005-AC-21** (`spec/functional/FR-005-register-first-party-providers.md`): The typed Kani result retains CG's `KaniExecutionEvidence` unchanged: a `NoVerdict` diagnostic record and a present `failure_cause` remain equal to CG's values, while absent optional fields remain absent. The driver does not parse diagnostics or derive a failure cause from them.
- **FR-009-AC-14** (`spec/functional/FR-009-settle-execute-and-replay-results.md`): For a Kani result, the driver passes the original CG `KaniExecutionEvidence` and optional replay settlement to CG `run_terminal_value`; it preserves the resulting typed terminal without adding a driver mapping. The map receives the evidence's actual optional `failure_cause`; the driver invents or infers no cause.
- **FR-009-AC-15** (`spec/functional/FR-009-settle-execute-and-replay-results.md`): Changing or omitting optional CG diagnostics while holding the outcome, success-check count, failure cause, and replay settlement fixed does not change the terminal produced by CG `run_terminal_value`; the driver does not parse diagnostic content to classify or map a result.

## Verdict

**CONDITIONAL** — one medium evidence-quality gap in the test for FR-005-AC-21; no untagged changed criterion or reverse code-to-requirement gap was found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-005-AC-21 test constructs its own evidence and reads the same fixture back, so it does not establish that the execution path retains CG diagnostics and failure_cause unchanged. | src/provider.rs:823-844; FR-005-AC-21 |

## Coverage

- Reconciliation: quoin matrix (quoin 0.28.3, quire 0.36.2) — four changed criteria tagged; no run evidence available. The exact-tree matrix has one binder per changed criterion.
- Plan completion: not assessed
- Criteria: tagged 4 · untagged 0 · tagged-by-ignored-test 0 · method-without-symbol 0 (changed-criterion subset)
- Evidence: bound 0 · stale 0 · suspect 0 · undischarged 4 (no run evidence)
- Untraced behaviors / stubs: 1 changed criterion has a fixture-only test that does not observe the production evidence-preservation path.
- Semantic review: skipped (not requested)

The repository-wide static matrix has 52 untagged and 3 method-without-symbol criteria outside the exact PR diff; those are not included in this PR-scoped verdict. `quire matrix --strict` therefore exits 1 repository-wide, while all four changed criteria are tagged.
