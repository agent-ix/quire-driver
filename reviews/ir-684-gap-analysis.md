---
id: SR-3121
title: "IR-684 gap-analysis of FR-003 caller-selected limits"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver PR #52 frozen head (commit id recorded only in the Linear marker); FR-003; TC-010; src/limits.rs; quire-cli/src/cli.rs; quire-cli/tests/binary_process.rs; ticket IR-684"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-003
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-010
    type: reviews
---
# IR-684: gap-analysis

## Summary

Ticket: IR-684. PR: quire-driver#52. Planless gap analysis (plan completion: not assessed) over FR-003 and TC-010, using `quire matrix --format tsv` on driver main and on the PR head (quire 0.36.1, engine 0.50.1).

FR-003 matrix, main to PR head:

| Criterion | main | PR head |
| --- | --- | --- |
| FR-003-AC-1 | tagged | tagged (adds four limits.rs tests) |
| FR-003-AC-2 | tagged | tagged |
| FR-003-AC-3 | tagged | tagged (line shifts only) |
| FR-003-AC-4 | method-without-symbol | method-without-symbol, now with two limits.rs symbols |
| FR-003-AC-5 | untagged | tagged |
| FR-003-AC-6 | untagged | tagged |
| FR-003-AC-7 | untagged | tagged |
| FR-003-AC-8 | untagged | tagged |
| FR-003-AC-9 | untagged | untagged (planned, QSL-656) |
| FR-003-AC-10 | untagged | untagged (planned, QSL-656) |

Outside FR-003, the only matrix changes are binary_process.rs and cli.rs line shifts for FR-002-AC-6, FR-002-AC-8 and FR-004-AC-2. No criterion lost a binder.

The tests.md adaptation list from the merged spec PR was followed. `src/limits.rs` 232-256 became `qsl_parser_refuses_unknown_malformed_and_duplicate_operands`, asserting QSL `UsageCause`s. `src/limits.rs` 258-269 (bare `work_units=11` applied to accounting) was replaced by `recognized_but_unavailable_setting_has_operation_specific_error`. `binary_process.rs` 211-231 moved from bare `work_units` to `s3.work_units`. `binary_process.rs` 257-276 became the unknown/malformed/duplicate loop, with the `s1.input_bytes=invalid` case now tagged AC-7 (with AC-2 and AC-8 on the same loop). No production code lacks an owning requirement: `LimitOverrides`/`LimitError` are FR-003, and the cli.rs edits are FR-003 call-site changes. No stubs, tautologies or coverage inflation were found. `every_exposed_setting_updates_only_its_qsl_bound` derives its cases from QSL `bounds()`, not from a driver list.

## Examined scope

```yaml
{
  "scope": [
    {"id": "FR-003-AC-1", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "Setting one QSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values."},
    {"id": "FR-003-AC-2", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A setting name not resolved by QSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand."},
    {"id": "FR-003-AC-3", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "An input that reaches a caller-lowered bound returns typed `LimitExceeded` with the configured value and field name and QSL's incomplete category; the process status equals QSL's category exit function. Raising the same bound to fit the input allows the operation to complete."},
    {"id": "FR-003-AC-4", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "The driver uses QSL `parse_operands`, checks each resolved setting against the selected operation's `SettingLimits::bounds()`, and applies an exposed setting with `CallerLimits::set_bound`; it maintains no duplicate grammar, alias table, or name-to-field mapping."},
    {"id": "FR-003-AC-5", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A setting parsed by QSL but absent from the selected operation's `SettingLimits::bounds()` is refused before any stage with code `invalid-request` and a distinct diagnostic naming the setting as unavailable for that operation."},
    {"id": "FR-003-AC-6", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "Applying an accepted setting through QSL `CallerLimits::set_bound` changes only the matching bound; every other operation limit retains its prior value."},
    {"id": "FR-003-AC-7", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A malformed `--limit` operand is refused before any stage with code `invalid-request` and identifies that operand."},
    {"id": "FR-003-AC-8", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A duplicate `--limit` setting is refused before any stage with code `invalid-request` and identifies the duplicate setting."},
    {"id": "FR-003-AC-9", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "After QSL-656 lands, the driver accepts the accounting setting names defined by QSL FR-255 through QSL's operand parser under the canonical `accounting.<counter>` namespace, without adding aliases."},
    {"id": "FR-003-AC-10", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "After QSL-656, when the selected operation's `SettingLimits::bounds()` includes a caller accounting setting, the driver applies that value through QSL `CallerLimits::set_bound` and passes it to the operation."},
    {"id": "TC-010", "path": "spec/test-cases/TC-010-caller-selected-limits.md", "role": "examined", "excerpt": "3. Pass the former driver-only name `normalization.work_units`, a malformed operand, a duplicate setting and a former bare accounting name such as `work_units`. Observe each refusal before any stage with code `invalid-request`"}
  ]
}
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-010 step 3's former bare accounting name (`work_units`) has no test, though tests.md marks FR-003-AC-2 done; a re-added driver alias for a bare accounting name would pass every test | quire-cli/tests/binary_process.rs:257-284; src/limits.rs:117-142; spec/tests.md:57 |

### FND-001

TC-010 step 3 and its Expected Results ("Unknown names (including former bare accounting names) ... are refused") require a former bare accounting name such as `work_units` to be refused as an ordinary unknown setting. The old `src/limits.rs` 258-269 test applied `work_units=11`; it was replaced by the AC-5 unavailable test rather than adapted into a bare-name refusal. Neither the unit test nor the binary loop passes `work_units` (or any of the ten former bare names). QSL's parser refuses it today, but the driver-level guarantee the spec names (no alias re-added in `LimitOverrides::parse` or `RunOptions::parse`) has no killing test. Fix: add `"work_units=1"` to the binary loop and to `qsl_parser_refuses_unknown_malformed_and_duplicate_operands`, expecting `UsageCause::UnknownSetting`.

## Verdict

**PASS with one medium finding.** AC-1 through AC-8 now have tagged binders, AC-9/AC-10 correctly stay untagged and planned, and the adaptation list was followed. The bare-accounting-name case is the one TC-010 step not executed.

## Validation and limitations

Matrix computed with the installed `quire matrix` over separate main and PR-head worktrees. Ambient module warnings (DuplicateArchetype, DuplicateInverseEdge, inline-data-schema) appear identically on both sides. No tests were executed (no cargo, by brief). Custody: the fix-round coder or lead copies this file to `reviews/ir-684-gap-analysis.md`.

## Dispositions

Round 1, the fix round-1 commit on PR #52 (commit id recorded only in the Linear dispositions marker). The matrix after the fix round has the same per-criterion statuses as before it (82 tagged, 49 untagged, 2 method-without-symbol repo-wide); FR-003-AC-2 gains one binder, AC-5 one more, and AC-9/AC-10 stay untagged.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fix round 1: `work_units=1` is in the binary refusal loop and in `qsl_parser_refuses_unknown_malformed_and_duplicate_operands` expecting `UsageCause::UnknownSetting`; a re-added bare alias now fails |
