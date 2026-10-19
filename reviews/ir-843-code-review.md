---
id: SR-5842
title: "Code and Rust review of IR-843 Kani evidence forwarding"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@1c9f16196592633ef8e74269c367674ae2def9be; src/provider.rs; Cargo.lock; spec/functional/FR-005-register-first-party-providers.md; spec/functional/FR-009-settle-execute-and-replay-results.md; spec/test-cases/TC-012-first-party-provider-interface.md; spec/test-cases/TC-016-execute-replay-and-terminal-records.md; spec/tests.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-005, type: reviews }
  - { target: ix://agent-ix/quire-driver/FR-009, type: reviews }
---
# Code and Rust review of IR-843 Kani evidence forwarding

## Summary

Ticket: IR-843. PR: quire-driver#82, reviewed at the exact frozen head `1c9f16196592633ef8e74269c367674ae2def9be` against base `58b5b2721e9746330ad31d9f2d8c8a79a729f9e5`. The implementation forwards explicit home context and original CG evidence to CG's APIs, but the evidence-retention test is fixture-only and does not kill a production-path projection mutant.

## Scope

- **FR-005-AC-20** (`spec/functional/FR-005-register-first-party-providers.md`): The driver forwards `KaniExecutionLimits.home_directory` unchanged to CG's `KaniExecutionRequest`; if it is `None`, the driver passes `None` even when the process environment has a `HOME` value. The driver does not infer the path from other directories or from the selected Kani installation.
- **FR-005-AC-21** (`spec/functional/FR-005-register-first-party-providers.md`): The typed Kani result retains CG's `KaniExecutionEvidence` unchanged: a `NoVerdict` diagnostic record and a present `failure_cause` remain equal to CG's values, while absent optional fields remain absent. The driver does not parse diagnostics or derive a failure cause from them.
- **FR-009-AC-14** (`spec/functional/FR-009-settle-execute-and-replay-results.md`): For a Kani result, the driver passes the original CG `KaniExecutionEvidence` and optional replay settlement to CG `run_terminal_value`; it preserves the resulting typed terminal without adding a driver mapping. The map receives the evidence's actual optional `failure_cause`; the driver invents or infers no cause.
- **FR-009-AC-15** (`spec/functional/FR-009-settle-execute-and-replay-results.md`): Changing or omitting optional CG diagnostics while holding the outcome, success-check count, failure cause, and replay settlement fixed does not change the terminal produced by CG `run_terminal_value`; the driver does not parse diagnostic content to classify or map a result.

## Verdict

**CONDITIONAL** — one medium test-intent gap: the test cited for FR-005-AC-21 only inspects evidence it constructed itself, so it would pass if the execution path discarded optional fields.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FR-005-AC-21 test proves only that its fixture retains fields it just assigned; it does not exercise CG execution or the provider execution path, so stripping diagnostics or failure_cause from returned evidence would leave it green. | src/provider.rs:823-844; spec/functional/FR-005-register-first-party-providers.md:171 |

## Code Review

- Reviewed the exact PR diff only. The only production Rust change forwards `KaniExecutionLimits.home_directory` to `KaniExecutionRequest` and sends the retained evidence reference plus optional settlement directly to CG `run_terminal_value`; it does not consult `HOME`, infer context, project evidence, parse diagnostics, or infer failure causes.
- The new home-context test uses a child with decoy `HOME` and checks both explicit `Some(path)` and unknown `None`. The terminal-map test verifies CG maps the original `OutOfMemory` cause, changing/omitting diagnostics leaves the terminal equal, absent cause plus non-classifying text remains `Failed`, and a falsification with `ReplaySettlement::Reproduced` becomes `Refuted` through CG.
- FR-005-AC-21's `provider_result_preserves_optional_cg_diagnostics_and_failure_cause` constructs a `ProviderItemResult` via the fixture helper, assigns the evidence fields there, and reads them back. A mutation in the actual execution-result path that clears either optional field would not touch this test. Add an assertion through the provider execution result path or an injectable execution seam so the test observes CG-produced evidence.
- Test tags bind correctly to FR-005-AC-20/21 and FR-009-AC-14/15. Static matrix reports all four criteria tagged, one binder each.
- The PR body is `Part of IR-843` with no close keyword. `Cargo.lock` changes exactly one revision for each changed upstream dependency cohort listed in the PR body; the CG source is `4fb3402242b6d5913a9beff76779df5aef73ada9`, and there are no unrelated changed floating revisions in this diff.
- PR #81 is merged (head `49936212b0bde683c9e35c25788cc652504bb354`). Its changed contracts appear on this review head. Current CG main is `4fb3402242b6d5913a9beff76779df5aef73ada9`; `KaniExecutionRequest.home_directory` is optional, `KaniExecutionEvidence` retains optional `diagnostics` and `failure_cause`, and `run_terminal_value` accepts `&KaniExecutionEvidence` plus `Option<ReplaySettlement>`.

## Rust Review

- Repo conventions were read from `CLAUDE.md` and `AGENTS.md`. No new panic/unsafe surface or public API debt was found in the diff. The test's subprocess path is deterministic and the environment override is explicit.
- Relevant mutant checks by inspection: removing explicit home forwarding is killed by the `Some` equality; falling back to ambient `HOME` for `None` is killed by the child test; omitting the replay settlement is killed by the falsification test; dropping the actual failure cause is killed by the OOM terminal expectation; diagnostic-presence-dependent mapping is killed by the with/without comparison. A production evidence-projection mutant is not killed by the fixture-only FR-005-AC-21 test (FND-001).

## Gate Results

No tests or aggregate `make ci` were run, per review brief. The static `quire matrix` reports the four changed criteria tagged. The repository-wide strict matrix exits 1 on 52 other untagged criteria and 3 `method-without-symbol` criteria, outside this exact PR scope.
