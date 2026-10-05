---
id: SR-029
title: "IR-607 PR #21 code, Rust, and gap review"
type: SpecReview
analysis: code-review + rust-review + gap-analysis
scope: "agent-ix/quire-driver#21@004206a79e23714dfa11347b6136925bccf52bfa; changed Rust files only"
review_set: subset
---

# IR-607 PR #21 review

## Summary

Reviewed only PR #21 at `004206a79e23714dfa11347b6136925bccf52bfa` in the code, Rust, and gap-analysis lanes. No CI was run. The central conversion is correct: `process_exit_code` is the sole `Category` to `ExitCode` conversion and directly calls `Category::exit_code`; there are no other process-exit conversions or signal/timeout handlers. The `OutcomeSink` boundary carries serialized outcome JSON as opaque text and does not implement kit mechanics or re-encode the outcome. Its success and failure tests preserve QSL categories as intended. The exhaustive `Category::ALL` test would fail if the process mapping were mutated to constant `1`.

## Verdict

**CONDITIONAL** — two findings remain in the argument parser and its future dispatch seam.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | major | Operation arguments are consumed and discarded instead of reaching a handler. | `src/bin/quire.rs:84-105,114-120` |
| FND-002 | moderate | The parser duplicates ADR-029's verb and format rules without citing the authority. | `src/bin/quire.rs:10-48,82-109` |

### FND-001 — operation arguments are discarded

The parser comment says operation-specific options "pass through untouched for later handlers," but `Invocation::Operation` stores only `verb` and `format`; the loop advances over every remaining argument, and `main` ignores both stored fields. Consequently `--limit` and other operation arguments cannot be dispatched later, and arbitrary unknown options are silently accepted before the command returns `Unsupported`. Preserve the raw operation arguments in the invocation and pass them to the handler seam (or explicitly reject options that are not parsed) so this boundary does not lose caller input.

### FND-002 — ADR-029 rules are restated

`VERBS`/`Verb::parse` encode the full verb list, and `OutputFormat`/`parse_invocation` encode the accepted formats and the `dot` restriction. These are local copies of the authoritative ADR-029 command grammar, but the changed code does not cite ADR-029. Keep ADR-029 as the named authority at the parser boundary and avoid presenting a second authoritative grammar; add source references where the local dispatch selectors are necessarily represented.

## Reviewed checks

- `process_exit_code` is the only conversion from QSL `Category` to `std::process::ExitCode`, and calls `category.exit_code()` directly.
- `main` returns `ExitCode`; no `process::exit`, direct numeric status mapping, signal handler, or timeout path was introduced.
- `OutcomeSink` accepts the serialized outcome unchanged. It provides an output seam without importing or copying ix-cli-kit mechanics.
- The library unit test iterates all eight QSL categories, including those whose exit codes are 20, 21, 22, and 30; a constant-1 mutant fails this assertion. Integration tests directly assert refusal 20 and unsupported 21.
- Rust structure is small and safe; no ownership, unsafe, panic-path, or error-propagation defect was found within the scoped files.
- The binary is an initial skeleton: known commands currently return `Unsupported`, and help/version currently return success without rendering text. Full command execution and output remain open ticket work, consistent with the PR's documented dependency seam.

## Scope and limitations

Examined `src/bin/quire.rs`, `src/cli.rs`, `src/lib.rs`, and `tests/cli_skeleton.rs` in the PR diff against `3da83aafc9ab8b714b4ebb22e50919aac7480471`. Read FR-002 and TC-018 as context. No files were changed other than this review artifact; no tests or CI were run.

## Follow-up on d387759

Reviewed the same PR scope at `d387759796408170dec389339bc19e12980c0d77` as round 2 of this review.

- **FND-001 resolved at the parser/dispatch seam:** the invocation now retains the original operation `OsString`s and `main` passes them to `dispatch`; a unit test covers `--limit`, its value, and a path argument.
- **FND-002 resolved:** parser and selector documentation now cite QSL ADR-029 as owner/authority.
- **Residual FND-001 boundary gap:** `main` still collects only the first 64 argv entries with `.take(64)`, silently dropping any remaining arguments before the parser and dispatch seam. This contradicts the dispatch seam's “every original argument” contract. Remove the truncation or reject overflow explicitly before dispatch.

The original exit mapping, opaque outcome seam, absence of signal/timeout handling, and constant-1 mutant coverage remain unchanged and valid. **Follow-up verdict: CONDITIONAL** on the silent argv truncation. CI was not run.

## Final follow-up on 6a51eaef

Reviewed the same scope at `6a51eaef51b8c3ec7ab9bf88e89ff0394e565294` as round 3. `main` now collects all operation arguments with `args_os().skip(1).collect()`; the raw `OsString` values remain retained in `Invocation::Operation` and are passed through `dispatch`. The overflow gap is resolved. The ADR-029 authority citations remain in place. FND-001 and FND-002 are resolved, and the original exit mapping, opaque outcome seam, no-signal/no-timeout check, and constant-1 mutant coverage remain valid. **Final verdict: PASS.** No CI was run.
