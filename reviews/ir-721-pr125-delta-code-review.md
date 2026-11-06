---
id: SR-2026110414560058
title: IR-721 PR125 rebased delta code review
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; PR #125 rebased delta at 8a2ad60b1fdf312db84857cb5465a4a657a234cb; src/monitor/{error,lowering,projection,selection,source_record}.rs; tests/{monitor_preparation,monitor_final_subject,monitor_lowering,source_record_admission}.rs; FR-015-AC-14 and FR-018-AC-5 context"
review_set: subset
---

## Summary

Ticket IR-721. Focused independent review of the material code delta since the previously reviewed head 07d28ef: native join budget API migration, public source-record projection budget, native storage error classification, and Timed independent clock-row refusal. The prior static review reported clean; this document does not repeat unrelated PR review. No applicable AssuranceProfile was identified in the previously reviewed scope.

## Verdict

PASS for the reviewed delta. The driver gives the native reads remaining work and storage, charges successful clock and type-projection work once, preserves native refusal payloads, and maps native storage exhaustion to the stable storage code. The public source-record projection accepts the caller's mutable native budget. The authentic Timed fixture refuses an extra independent configured row as Clock selection. No new defect found. Merge remains subject to the lead's final aggregate gate on the reviewed head.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | implementation-bug-despite-evidence |

## Coverage

Rust lane of code-review examined the changed resource, public API, error mapping, clock comparison, and focused test paths. FR-015-AC-14 and FR-018-AC-5 were read as behavior context. Gap-analysis and spec-review were not repeated because this assignment is the material delta after the prior static review. The coder reported focused AC14/refusal, monitor_preparation 19/19, lowering, source-record, and final-subject tests passing; no reviewer tests, builds, or gates ran.

## Examined units

```yaml
scope:
  - id: FR-015-AC-14
    path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
    role: examined
    excerpt: "A healthy clock-admission entry exposes the native `clock_for_claim` receipt for its independently selected final clause/Claim, charges successful receipt work exactly once to the remaining cumulative allowance, and compares every field of any optional independent clock row."
  - id: FR-018-AC-5
    path: spec/functional/FR-018-admit-source-record-witnesses.md
    role: examined
    excerpt: "An authentic emission joins successfully at its measured native work and storage charge. One unit below either dimension returns no owner and preserves the original native exhausted-work or exhausted-storage payload."
  - id: src/monitor/error.rs
    path: src/monitor/error.rs
    role: examined
    excerpt: "ExactTypeProjectionErrorCode::ExhaustedStorage => MonitorPreparationErrorCode::Storage"
  - id: src/monitor/selection.rs
    path: src/monitor/selection.rs
    role: examined
    excerpt: "let mut native_budget = NativeJoinBudget::new(budget.remaining(), budget.remaining_storage());"
  - id: src/monitor/source_record.rs
    path: src/monitor/source_record.rs
    role: examined
    excerpt: "pub fn project(&self, node: &CheckedNodeId, budget: &mut NativeJoinBudget) -> Result<ExactTypeProjection, ExactTypeProjectionError>"
  - id: src/monitor/projection.rs
    path: src/monitor/projection.rs
    role: examined
    excerpt: "project_exact_type(target, identities, &mut native_budget)"
  - id: src/monitor/lowering.rs
    path: src/monitor/lowering.rs
    role: examined
    excerpt: "project_exact_type(node, identities, &mut native_budget)"
  - id: tests/monitor_preparation.rs
    path: tests/monitor_preparation.rs
    role: examined
    excerpt: "fn timed_model_clock_refuses_an_independent_extra_configured_row()"
  - id: tests/source_record_admission.rs
    path: tests/source_record_admission.rs
    role: examined
    excerpt: "fn authentic_join_charges_both_required_allowances_and_preserves_storage_refusal()"
  - id: tests/monitor_final_subject.rs
    path: tests/monitor_final_subject.rs
    role: examined
    excerpt: "fn original_subjects_select_their_own_final_claims_and_native_model_time()"
  - id: tests/monitor_lowering.rs
    path: tests/monitor_lowering.rs
    role: examined
    excerpt: "let mut native_budget = NativeJoinBudget::new(u64::MAX, u64::MAX);"
```
