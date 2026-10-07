---
id: SR-2760
title: "code review (with Rust lane) of quire-driver PR 43 (IR-681) signal test diagnostics"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver PR 43 (IR-681), reviewed revision named in the tracker marker; CLAUDE.md, quire-cli/src/signals.rs (module docs and mod tests); read against quire-cli/tests/fixtures/signal-plugin.py and quire-plugin-host/src/lib.rs (plugin stdio setup)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: reviews
---
# SR-2760: code review of the IR-681 signal test diagnostics

## Summary

Ticket: IR-681. The diff has two files. `CLAUDE.md` gains a two-line note
after the `make` command block. `quire-cli/src/signals.rs` gains a module-doc
note (lines 7-9), and every other change is inside `mod tests`. No production
code changed. The Rust lane (rust-review) is folded into this file. Cargo was
not run: a macOS `make ci` is reported passed and a Linux normal-permission
gate is queued.

Measured:

- **Only tests and docs.** The non-test hunks are `CLAUDE.md:31-32` and the
  `//!` lines at `signals.rs:7-9`. Both are documentation.
- **No assertion lost strength.** Each before/after pair was compared. The
  readiness `assert!(try_wait().is_none())` became `if let Some(status) ...
  panic!`, and `assert!(marker.is_file())` became `if !marker.is_file() {
  kill; wait; panic! }`. Both keep the same pass/fail condition. The signal
  equality assertions (`signals.rs:447`, `:672`), the prompt-cleanup bound
  (`:673-676`), the stdout sentinel scans, the `!outcome.exists()` checks,
  `cleaned.is_file()`, `request_cancelled.exists()` and the two reap
  assertions after the 2200 ms sleep are unchanged except for their messages.
- **No timing constant changed.** These are still 3 s, 4 s, 5 s, 1500 ms,
  2200 ms, the 5 ms poll and the fixture's 2/10 s sleeps. No mutant becomes
  newly survivable.
- **Diagnostics at every failure path.** All of these now include `ExitStatus`
  Debug, `code()`, `signal()` (via `ExitStatusExt`), and lossy stdout and
  stderr: an early exit before ready, a readiness timeout, a bounded-wait
  timeout, and every post-exit assertion. On the two timeout paths, the child
  is killed and waited before its pipes are read. The readiness-timeout path
  used to panic without killing the child. It now kills it, which fixes a
  leaked child that could run for up to 10 s. Test 2 used to read with
  `read_to_string`, which could panic on non-UTF-8 output and hide the
  diagnostic. It now reads lossily.
- **No hang on read.** `read_child_output` reads only after `wait`/`try_wait`
  returns. The plugin is spawned with stdin, stdout and stderr all piped
  (`quire-plugin-host/src/lib.rs:388-390`). So neither the plugin nor its
  fixture descendant holds the test child's fd 1/2, and EOF arrives when the
  direct child exits, even if a process-group kill failed. Pipe back-pressure
  cannot deadlock past the bounded wait, because a child blocked on a full pipe
  is killed at the deadline.
- **Rust idioms.** The helpers are `#[cfg(any(target_os = "linux", target_os
  = "macos"))]`-gated like the tests that use them. `expect` and `panic!` sit
  under the module's existing reasoned `#[allow(clippy::expect_used,
  clippy::panic, reason = ...)]`. There is no new `allow` and no `unwrap` in
  non-test code. Helper names are clear.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The readiness-timeout panic messages say the opposite of what happened. The old `assert!` messages were expectations, and they were copied verbatim into an unconditional `panic!`. A sandboxed readiness timeout therefore prints `active child became ready: child status=...` and `hello fixture reached its blocking point; child status=...` when the child did not become ready. That misleads the person diagnosing exactly the failure this PR exists for. Reword them, for example `"{mode} child did not become ready within 3 s: ..."`. | quire-cli/src/signals.rs:413-416, quire-cli/src/signals.rs:645-648 |
| FND-002 | low | The plugin lifecycle test's post-exit assertions do not name the stage. The test loops over six stages. The signal-equality assertion `assert_eq!(status.signal(), Some(expected), "{failure}")` is the one that fails in the sandbox (`None` vs `Some(SIGTERM)`), and its message has no `{stage}`. The same is true of the sentinel, outcome-document and request-handle messages. The child's libtest stdout names the test, not the stage. So the failure report still cannot tell `hello` from `shutdown-int`. Test 1 prefixes `{mode}:` (line 447). Do the same here. | quire-cli/src/signals.rs:672, quire-cli/src/signals.rs:677-691 |
| FND-003 | low | The prompt-cleanup timing check has moved. `started.elapsed() < 1500 ms` is now evaluated after `read_child_output` (line 670), so the measured interval includes reading both pipes to EOF. Before this PR it was taken right after the child's status. The cost is negligible today, because no grandchild inherits the child's stdout/stderr. But the 1500 ms bound now depends on pipe EOF, and IR-681 states that timing behaviour is unchanged. If a future fixture inherits stdio, a slow descendant would fail the timing assertion with a misleading "must cancel promptly" message. Fix: capture `let elapsed = started.elapsed();` as soon as `status` is obtained and assert on that. | quire-cli/src/signals.rs:670-676 |
| FND-004 | low | The docs state the requirement but not how to recognise a sandbox failure or what to do about it. Neither note names the affected tests. Neither describes the observed signature: the child's `signal()` is `None` where `SIGTERM`/`SIGINT` was expected, or the bounded wait is exceeded. Neither says to rerun under normal (unsandboxed) permissions rather than treating the failure as a regression. A reader who hits `signal=None` will not connect it to this note. The note does not overclaim the unmeasured mechanism, which is correct. | CLAUDE.md:31-32, quire-cli/src/signals.rs:7-9 |

## Verdict

The PR stays within IR-681: only tests and docs change. No assertion, oracle
or timing constant is weakened, and every failure path now reports the child's
status, code, signal, stdout and stderr without risk of a hang. All four
findings are low. Two message fixes (FND-001, FND-002), one moved line
(FND-003) and one extra doc sentence (FND-004) would finish the job in one
small fix round. Nothing here blocks correctness.
