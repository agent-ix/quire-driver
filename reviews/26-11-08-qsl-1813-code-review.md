---
id: SR-6011
title: "code review (with Rust lane) of quire-driver PR 131 (QSL-1813) frontend CLI process tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver PR 131 (QSL-1813), head 42371f61a65189590869a1788bd38d084e38ac3b; quire-cli/tests/frontend_cli.rs; read against quire-cli/src/cli.rs, ix-cli-kit streams.rs and quire-cli/tests/binary_process.rs"
review_set: subset
---
# SR-6011: code review of the QSL-1813 frontend CLI process tests

## Summary

Ticket: QSL-1813. The diff adds one file, `quire-cli/tests/frontend_cli.rs`
(259 lines, tests only). The Rust lane (rust-review, rust-style loaded) is
folded into this file. Cargo was not run (shared build lane); the coder
reports tests and clippy clean.

Measured against `quire-cli/src/cli.rs`:

- AC-3: the arity refusals come from the `*Options::parse` functions, which
  run before `read_source`. The `!stderr.contains("could not read")` check
  matches the real read-failure message (`could not read source ...`), so the
  test fails if a refusal moved after the read. ix-cli-kit renders a human
  diagnostic as the message only, so `stderr.contains(verb)` checks the
  message and nothing else.
- AC-5: each invalid case reaches a refusal in `parse_invocation` or
  `*Options::parse`, and the valid case asserts exit 0 and non-empty stdout.
- AC-7: `/dev/full` returns ENOSPC to every writer, root included. The test
  is gated to Linux. It fails loudly (`expect`) where the device is missing.
  It does not pass silently.
- AC-8: the child's Rust runtime ignores SIGPIPE, so a write gets EPIPE and
  `write_primary_output` keeps the category. The test would fail on a
  signal death (`code() == None`). But the ordering is racy (FND-001).
- AC-9: the relative and absolute runs compare stdout, stderr and the exit
  code, so the test fails if the path leaks into the output.
- No new fixtures, and no spec text was copied. The test uses the existing
  `ir608-run.qsl` and `parse-invalid.qsl`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-8 closed-pipe test drops the stdout reader only after spawn; if the child writes first (pipe buffer absorbs it) EPIPE is never hit and the test passes without exercising the broken-pipe path | quire-cli/tests/frontend_cli.rs:198-205 |
| FND-002 | medium | AC-6-tagged test asserts that a non-UTF-8 parse/format/compile path operand refuses, the opposite of FR-287-AC-6's second clause (a non-UTF-8 file path parses); it pins the contrary behaviour under that AC's trace | quire-cli/tests/frontend_cli.rs:131-137 |
| FND-003 | low | REFUSAL and TOOL_FAILURE hardcode 20 and 30 while the same file derives codes from Category::exit_code elsewhere | quire-cli/tests/frontend_cli.rs:17-18 |
| FND-004 | low | AC-9 temp directory has no drop guard, so a failing assertion leaks it; binary_process.rs already has a TestDirectory guard pattern | quire-cli/tests/frontend_cli.rs:227-258 |

## Verdict

Changes requested. FND-001 and FND-002 must be fixed before merge.

- **FND-001:** make the closed reader deterministic. Use `std::io::pipe()`
  (MSRV 1.98.1 allows it), drop the reader before `spawn`, and pass the
  writer as stdout. Or reuse the FIFO-source ordering from
  `binary_process.rs:1039`.
- **FND-002:** drop the three path-operand cases from the AC-6 test, or move
  them out from under the AC-6 trace. The PR body already states that the
  non-UTF-8 path clause is unmet.

AC-3, AC-5, AC-7 and the AC-9 path check are real tests that can fail.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | New run usage line advertises `[--engine interpreter]` although ExecutionEngine::parse also accepts aot and jit, and omits the accepted `--provider-config <path>` option, so the usage text misstates run's options | quire-cli/src/cli.rs:557 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | ae57cdb: std::io::pipe() reader dropped before the child is spawned; the first write always meets a closed pipe (frontend_cli.rs:213-221) |
| FND-002 | fixed | ae57cdb: the three non-UTF-8 path-operand cases were removed from the AC-6 test; PR body states the path clause as unmet |
| FND-003 | fixed | ae57cdb: refusal_code() and tool_failure_code() derive from Category::Refusal and Category::InternalFailure |
| FND-004 | fixed | ae57cdb: local TestDirectory Drop guard removes the directory after a failed assertion |
| FND-005 | still-open | new in this round; the run usage line must list `--engine interpreter|aot|jit` and `--provider-config <path>` |
