---
id: SR-3120
title: "IR-684 code-review of the QSL setting API limit path"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver PR #52 frozen head (commit id recorded only in the Linear marker); src/limits.rs; quire-cli/src/cli.rs; quire-cli/tests/binary_process.rs; ticket IR-684"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-003
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-010
    type: reviews
---
# IR-684: code-review (rust-review folded in)

## Summary

Ticket: IR-684. PR: quire-driver#52 (draft). Scope: the PR diff against driver main, six files. The rust-review lane is folded into this file. QSL APIs were read at the driver's lockfile pin of quire-spec-language: `qsl-foundation/src/setting.rs` (`Setting`, `from_name`, `SettingLimits`, `UsageRefusal`, `UsageCause`, `parse_operands`), `qsl-replay/src/limits.rs` (`CallerLimits`, `from_operands`, `SettingLimits for CallerLimits`) and `qsl-replay/src/spine.rs` (`SettingLimits for SpineLimits`). Cargo was not run, by brief; the author reports a passing macOS `make ci` on the frozen head, and the Linux gate follows.

The driver's own limit table is deleted, not bypassed: `is_supported`, the 27-arm name-to-field `match`, `as_usize`/`PlatformValueOverflow`, `ValueOutOfRange`, `MalformedOperand`, `DuplicateSetting` and the bare accounting names are gone, and a repo-wide search over Rust, Markdown, TOML and the Makefile finds no leftover grammar, alias, name table or overflow constant (only historical review artifacts mention the old names). `LimitOverrides::parse` is a one-line delegation to `qsl_foundation::setting::parse_operands`. `apply_to` reads `SpineLimits::bounds()`, refuses any parsed setting absent from it with a distinct `OperationUnavailable` error naming the setting, and applies the rest through `CallerLimits::set_bound` on a copy, writing back only after every operand passed, so application is all-or-nothing. `CallerLimits::from_operands` is not used. Both refusal paths reach `CommandResult::input_refusal`, which emits `Code::InvalidRequest` (Refusal category, exit 20) before `read_source`, so no stage runs. Production code has no `unwrap`, `expect`, panic or `unsafe` on user input. No Cargo.toml or Cargo.lock change, no compatibility layer, no new hash, digest or commit id.

## Examined scope

```yaml
{
  "scope": [
    {"id": "FR-003-AC-1", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "Setting one QSL-resolved `--limit` operand overrides only its exposed bound; omitted settings retain their published defaults, and effective limits passed to the operation equal those values."},
    {"id": "FR-003-AC-2", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A setting name not resolved by QSL's operand parser is refused before any stage with code `invalid-request` and names the offending operand."},
    {"id": "FR-003-AC-4", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "The driver uses QSL `parse_operands`, checks each resolved setting against the selected operation's `SettingLimits::bounds()`, and applies an exposed setting with `CallerLimits::set_bound`; it maintains no duplicate grammar, alias table, or name-to-field mapping."},
    {"id": "FR-003-AC-5", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A setting parsed by QSL but absent from the selected operation's `SettingLimits::bounds()` is refused before any stage with code `invalid-request` and a distinct diagnostic naming the setting as unavailable for that operation."},
    {"id": "FR-003-AC-6", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "Applying an accepted setting through QSL `CallerLimits::set_bound` changes only the matching bound; every other operation limit retains its prior value."},
    {"id": "FR-003-AC-7", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A malformed `--limit` operand is refused before any stage with code `invalid-request` and identifies that operand."},
    {"id": "FR-003-AC-8", "path": "spec/functional/FR-003-apply-caller-selected-limits.md", "role": "examined", "excerpt": "A duplicate `--limit` setting is refused before any stage with code `invalid-request` and identifies the duplicate setting."},
    {"id": "src/limits.rs", "path": "src/limits.rs", "role": "examined", "excerpt": "LimitOverrides::parse delegates to parse_operands; apply_to checks SpineLimits::bounds() then CallerLimits::set_bound on a copy."},
    {"id": "quire-cli/src/cli.rs", "path": "quire-cli/src/cli.rs", "role": "examined", "excerpt": "RunOptions::parse maps UsageRefusal to input_refusal; run() maps LimitError to input_refusal before read_source."},
    {"id": "quire-cli/tests/binary_process.rs", "path": "quire-cli/tests/binary_process.rs", "role": "examined", "excerpt": "qsl_rejects_unknown_malformed_and_duplicate_limits_before_source_read; recognized_setting_unavailable_for_run_is_named_before_source_read; checking_work_limit_changes_the_call_outcome."},
    {"id": "qsl-foundation setting.rs", "path": "qsl-foundation/src/setting.rs", "role": "context_only", "excerpt": "parse_operands: UsageRefusal naming the first operand that is not <name>=<value>, names no setting, holds a non-u64 value, or repeats a setting."},
    {"id": "qsl-replay limits.rs", "path": "qsl-replay/src/limits.rs", "role": "context_only", "excerpt": "CallerLimits::set_bound tries spine, library, admission, observation, explore, replay in turn; from_operands applies every parsed setting."}
  ]
}
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test asserts code `invalid-request` on the operation-unavailable path; a mutant at cli.rs:369 that emits a Refusal-category failure with no catalog code survives | quire-cli/src/cli.rs:368-370; quire-cli/tests/binary_process.rs:286-311 |
| FND-002 | low | All-or-nothing application is implemented but untested; a mutant applying each setting straight into `limits` before a later unavailable operand survives | src/limits.rs:29-46 |
| FND-003 | low | Refusal order with several bad operands is deterministic but neither tested nor documented: every parser refusal wins over an operation-unavailable refusal, whatever the operand order | quire-cli/src/cli.rs:291; quire-cli/src/cli.rs:368 |
| FND-004 | low | The `!caller_limits.set_bound(..)` arm is unreachable for a setting in `bounds()`; if QSL ever broke that invariant it would be reported to the caller as an operation-unavailable refusal | src/limits.rs:40-41 |
| FND-005 | low | AC-6 "retains its prior value" is only tested from `SpineLimits::default()`; a mutant rebuilding `spine` from defaults instead of `*limits` survives | src/limits.rs:35-37; src/limits.rs:83-112 |

### FND-001

FR-003-AC-5 requires code `invalid-request`. `recognized_setting_unavailable_for_run_is_named_before_source_read` runs in text mode and checks only exit 20, empty stdout and the stderr wording. In text mode the outcome document is never written (`emit_result` returns early unless `--format json`), so replacing `CommandResult::input_refusal(..)` at cli.rs:369 with `CommandResult::failure(Category::Refusal, ..)` (no catalog code) or with any other Refusal-category code passes every test. The parser path is pinned by `malformed_run_limit_emits_one_qsl_outcome_and_streams_diagnostic` in JSON mode; the unavailable path has no equivalent. Fix: run the unavailable case with `--format json` and compare stdout with `expected_document_bytes(Code::InvalidRequest, ..)`.

### FND-002

`apply_to` copies `*limits` into `CallerLimits` and writes back only after the loop, which is correct. No test passes an exposed setting followed by an unavailable one (for example `s1.input_bytes=5` then `explore.states=1`) and checks that `limits` is unchanged after `Err`. Today the CLI drops `limits` on error, so the effect is invisible at the process boundary, but `apply_to` is public library API (`src/lib.rs:68`), and a refactor to `limits.set_bound` inside the loop would leave a half-applied value with no failing test.

### FND-003

`RunOptions::parse` runs `parse_operands` over all operands before `run()` calls `apply_to`. So `--limit explore.states=1 --limit bogus=1` reports the unknown `bogus=1`, not the earlier unavailable `explore.states`. That is deterministic and reasonable, but no test pins it and TC-010 does not state it, so a reorder would change which operand is named without a failing test.

### FND-004

`available` is built from `limits.bounds()`, and QSL guarantees that no two limits types report the same setting, so `CallerLimits::set_bound` always returns `true` via `spine` for an available setting. The second disjunct cannot fire. If it did, a QSL contract break would surface as a caller-facing "unavailable for the `run` operation" refusal. This is acceptable interim code. A `debug_assert!` or a comment stating the invariant would make the intent explicit.

### FND-005

FR-003-AC-6 says every other limit "retains its prior value". All `apply_to` tests start from `SpineLimits::default()`, so `CallerLimits { spine: SpineLimits::default(), .. }` (discarding the caller's incoming values, the `from_operands` shape) is indistinguishable. A test from a non-default baseline would close this.

## Verdict

**PASS with findings.** The implementation matches FR-003's interim scope exactly. It parses with QSL `parse_operands`, checks against the run operation's `SettingLimits::bounds()` (`SpineLimits`), applies with `CallerLimits::set_bound`, refuses all-or-nothing before any stage, and keeps no driver grammar, alias, name table or overflow rule. The tests kill the important mutants: applying through `from_operands`, skipping the bounds check (the AC-5 unit test expects `Err` for `explore.states`, which `CallerLimits` would otherwise accept into `explore`), accepting `normalization.work_units`, and passing default limits to the operation. FND-001 is the only medium finding. It is a one-test fix and does not block on correctness.

## Validation and limitations

Static review only: no cargo, by brief. The test results are the author's macOS `make ci` report on the frozen head, not reproduced here; the Linux `make ci` under normal permissions is still required. Quire validation of this artifact is at its explicit scratch path. Custody: the fix-round coder or lead copies this file to `reviews/ir-684-code-review.md`; the reviewer does not commit it.

## Dispositions

Round 1, the fix round-1 commit on PR #52 (commit id recorded only in the Linear dispositions marker). Verified statically against the actual delta; no cargo was run.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | fix round 1: `recognized_setting_unavailable_for_run_is_named_before_source_read` now runs `--format json` and asserts stdout equals `expected_document_bytes(Code::InvalidRequest, ..)`; a no-code or other-code mutant at cli.rs:369 now fails |
| FND-002 | fixed | fix round 1: `unavailable_setting_leaves_all_prior_bounds_unchanged` applies `s1.input_bytes=5` then `explore.states=11` and asserts `limits == before`; in-place application now fails |
| FND-003 | fixed | fix round 1: `invalid_operand_precedes_recognized_but_unavailable_setting` (explore.states before work_units) asserts the unknown operand is named and no unavailable text appears; TC-010 step 3 states the precedence |
| FND-004 | fixed | fix round 1: the branch is split; `debug_assert!(updated, ..)` documents the bounds()/set_bound invariant and release builds still refuse rather than panic; it cannot fire on valid input because `available` comes from the same `SpineLimits::bounds()` that `CallerLimits::set_bound` reaches first |
| FND-005 | fixed | fix round 1: `one_limit_override_preserves_other_defaults` now seeds `s1.tokens=17` before applying and asserts it is retained; a rebuild-from-defaults mutant now fails |
