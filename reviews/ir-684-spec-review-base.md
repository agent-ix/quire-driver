---
id: SR-3122
title: "IR-684 spec-review of the FR-003, TC-010 and tests.md edits"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #52 frozen head (commit id recorded only in the Linear marker); spec/functional/FR-003-apply-caller-selected-limits.md; spec/test-cases/TC-010-caller-selected-limits.md; spec/tests.md; ticket IR-684"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-003
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-010
    type: reviews
---
# IR-684: spec-review (base)

## Summary

Ticket: IR-684. PR: quire-driver#52. This review is scoped to the spec edits in the code PR, which go slightly beyond status rows: one normative FR-003 bullet is removed and one TC-010 step is reworded. They were compared with the merged FR-003 and TC-010 text on driver main.

FR-003: the diff is exactly the four-line deletion of the `CallerLimits::from_operands` Behavior bullet; no other prose, AC, relationship or dependency changed. The deletion is correct. The code never uses `from_operands`, and the bullet's "only when QSL's operation-specific bound application path is unavailable" escape is gone, so the parse, bounds-check, `set_bound` sequence is the only normative path. TC-010: step 3 now calls `normalization.work_units` "the former driver-only name" instead of "a name accepted by the current driver parser", which is accurate after the driver parser was deleted. The bare-name `work_units` case is kept, and no expected result changed. tests.md: the FR-003 rows are split per AC, AC-9/AC-10 stay 🚧 planned on QSL-656, and the TC-010 summary stays 🚧. `quire validate --scope . <three changed docs>` exits 0 with only the ambient module warnings.

## Examined scope

```yaml
{
  "scope": [
    {"id": "FR-003", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "- The driver SHALL construct `CallerLimits` for refusal reporting with `CallerLimits::from_operands` only when QSL's operation-specific bound application path is unavailable; this constructor does not replace the parse, bounds-check, then `set_bound` sequence above. (removed)"},
    {"id": "FR-003-AC-5", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A setting parsed by QSL but absent from the selected operation's `SettingLimits::bounds()` is refused before any stage with code `invalid-request` and a distinct diagnostic naming the setting as unavailable for that operation. After QSL-656, call-accounting settings exposed by those bounds are available for application and are not refused as unavailable."},
    {"id": "FR-003-AC-3", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and QSL's incomplete category; the process status equals QSL's category exit function. Raising the same bound to fit the input allows the operation to complete."},
    {"id": "TC-010", "path": "spec/test-cases/TC-010-caller-selected-limits.md", "role": "examined", "excerpt": "3. Pass the former driver-only name `normalization.work_units`, a malformed operand, a duplicate setting and a former bare accounting name such as `work_units`."},
    {"id": "spec/tests.md", "path": "spec/tests.md", "role": "examined", "excerpt": "FR-003 rows for AC-1/AC-3, AC-2/AC-7/AC-8, AC-4, AC-5, AC-6, AC-9/AC-10 and the TC-010 summary row."}
  ]
}
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | tests.md marks FR-003-AC-5 fully done, but AC-5's second sentence (accounting settings not refused after QSL-656) is planned and untestable today | spec/tests.md:59 |
| FND-002 | low | The tests.md AC-1/AC-3 row says `binary_process.rs` checks typed metadata through the driver wrapper; that evidence is in `src/execute.rs` | spec/tests.md:56 |

### FND-001

FR-003-AC-5 has two clauses. The unavailable-setting refusal is implemented and tested. "After QSL-656, call-accounting settings exposed by those bounds are available for application and are not refused as unavailable" cannot be exercised until QSL-656 lands. The row reads ✅ with no qualifier, while the AC-9/AC-10 row is 🚧. A reader of the matrix takes AC-5 as complete. Fix: mark the row partial, or note that the post-QSL-656 clause is planned with AC-9/AC-10.

### FND-002

`caller_source_limit_preserves_configured_bound_and_field` (configured bound and `setting()`) lives in `src/execute.rs`. `binary_process.rs` checks only exit status. The row should name `src/execute.rs` for the typed-metadata evidence.

## Verdict

**PASS with two low findings.** The only normative change, removing the `from_operands` bullet, is accurate and confined. The TC-010 rewording is accurate. Status rows match the matrix except for the two attribution and qualification nits above.

## Validation and limitations

Sub-analyses beyond base were not run: the edit is one bullet deletion, one reworded step and status rows, with no new requirement statement, relationship or domain object. Custody: the fix-round coder or lead copies this file to `reviews/ir-684-spec-review-base.md`.

## Dispositions

Round 1, the fix round-1 commit on PR #52 (commit id recorded only in the Linear dispositions marker). `quire validate --scope .` on FR-003, TC-010 and tests.md exits 0 with only the ambient module warnings. The new TC-010 sentence on refusal precedence is accurate to the code (parse in RunOptions::parse precedes apply_to in run()).

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fix round 1: the tests.md AC-5 row is now 🚧 Partial and names the accounting clause as planned until QSL-656 |
| FND-002 | fixed | fix round 1: the tests.md AC-1/AC-3 row now cites `src/execute.rs::caller_source_limit_preserves_configured_bound_and_field` for typed metadata and keeps binary_process.rs for status and raised-bound completion |
