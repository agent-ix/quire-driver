---
id: SR-5800
title: code-review of PR58 macOS fail-closed budgets
type: SpecReview
analysis: code-review
scope: agent-ix/quire-driver@252a3f70606603da761559b0ee58c0aecac3551f; quire-plugin-host/src/lib.rs,
  quire-plugin-host/tests/fixtures/failure-plugin.py; FR-013-AC-8 (IR-671)
review_set: subset
---

## Summary

Ticket: IR-671. Exact frozen diff reviewed against c9dac5c162dd6c72154171846b83397cbb378bce. Code-review includes the Rust-review lane. One medium test-readiness race; no production refusal defect found.

## Verdict

**CONDITIONAL** — Fix FND-001 before handoff; runtime reproduction was not performed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Escaped-descendant readiness is published before its JSON is complete: if the helper is descheduled after opening the final ready path but before closing its buffered JSON writer, the parent sees existence and emits hello; the Rust test then reads empty or partial JSON and fails before checking budget refusal. Publish a completed identity atomically or signal readiness only after closing it. | quire-plugin-host/tests/fixtures/failure-plugin.py:70; quire-plugin-host/src/lib.rs:2499 |

## Coverage

Review is limited to the PR diff. Rust idiom checklist applied before analysis and again as formal pre-handoff review; repository AGENTS.md/CLAUDE.md, safety/lint configuration and portable rust-style defaults read. No applicable AssuranceProfile found. No normative spec or plan diff: spec-review submethods do not apply.

Mac CPU and memory tests invoke the real PluginSession request path, require typed Unavailable and no writer dispatch; the escape fixture uses start_new_session and compares measured process-group identities. Removing the refusal causes expect_err or typed-cause assertions to fail. Existing test instrumentation observes entry into write dispatch; it does not alter behavior. Bounded marker polling checks a condition, rather than assuming readiness after a fixed sleep.

begin_request fails before ID advancement, serialization, and writer dispatch. Original cancellation/deadline precedence, abort diagnostics and cleanup/error custody are unchanged. Budget-free hello and requests retain the early return. Linux accounting and other-platform refusal logic are unchanged. macOS enforcement and complete escaped-tree termination are not implemented or claimed by this slice.

No builds, test executions or probes were run, per dispatch. Linux final full gate pending with lead. macOS candidate make ci was reported by a peer; no receipt was available in reviewer custody, so it is not reviewer-verified execution evidence.

ID collision check found no SR-5800/5801 in reviewed repository or existing scratch; reservation evidence came from dispatch, not ticket authorization. This is best available local evidence, not global proof.

## Examined scope

```yaml
scope:
- id: FR-013-AC-8
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Per-request memory and CPU-time budgets aggregate usage across the plugin
    process tree; a sampled budget breach terminates descendants and reports the corresponding
    failure. Memory sampling is best-effort between polls. If complete-tree measurement
    is unavailable, the host returns `ProcessBudgetError::Unavailable` before sending
    the request.
- id: FR-013-AC-4
  path: spec/functional/FR-013-host-process-plugins.md
  role: context_only
  excerpt: An invalid typed result, wrong request identity, plugin fault, or non-zero
    exit is returned as its specific failure and is not converted to cancellation
    or timeout.
- id: FR-013-AC-6
  path: spec/functional/FR-013-host-process-plugins.md
  role: context_only
  excerpt: A deadline expiring while a plugin does not read a request frame returns
    `TimedOut` within the 20-millisecond polling interval plus the one-second cleanup
    bound.
- id: FR-013-AC-9
  path: spec/functional/FR-013-host-process-plugins.md
  role: context_only
  excerpt: Stderr beyond the session retention limit is drained but not retained;
    stderr is available on failures, and reader/writer workers are joined within one
    second after termination begins.
- id: ProcessMonitor::begin_request
  path: quire-plugin-host/src/lib.rs
  role: examined
  excerpt: Err(ProcessBudgetError::Unavailable)
- id: ProcessMonitor::check
  path: quire-plugin-host/src/lib.rs
  role: examined
  excerpt: Err(ProcessBudgetError::Unavailable)
bindings:
- test_id: tests::begin_request_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-8
  trace: correct
- test_id: tests::macos_cpu_budget_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-8
  trace: correct
- test_id: tests::macos_memory_budget_refuses_before_dispatch_for_escaped_descendant
  ac_id: FR-013-AC-8
  trace: correct
```

## New findings (disposition pass 1)

Reviewed: agent-ix/quire-driver@5c4c1167fe94ffde209e70099e2ee4d3e3063b30; narrow fix delta in quire-plugin-host/src/lib.rs and quire-plugin-host/tests/fixtures/failure-plugin.py. Original scope and findings above remain unchanged.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | Partial-write visibility relies on a 200 ms sleep: the helper publishes .writing then automatically finishes and replaces ready, so if the Rust test is descheduled longer than that window, its !ready.exists() assertion fails even with correct atomic publication. Hold the helper at a bounded release handshake until the absence assertion completes. | quire-plugin-host/src/lib.rs:2516; quire-plugin-host/tests/fixtures/failure-plugin.py:78 |
| FND-003 | medium | Adding the writing argument shifted the request marker to sys.argv[6], but the escaped-descendant request handler still writes sys.argv[5], now the done path. The asserted request marker remains absent even after receipt, and the cleanup signal can be forged by request dispatch. Bind the handler to the actual request-marker argument and independently check cleanup. | quire-plugin-host/tests/fixtures/failure-plugin.py:129; quire-plugin-host/src/lib.rs:2494 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 5c4c1167fe94ffde209e70099e2ee4d3e3063b30 — same-directory temporary is closed before atomic os.replace exposes ready |

## Disposition round 1 evidence

Original FND-001 is fixed by source inspection. Round-one result remains CONDITIONAL: FND-002 and FND-003 need correction in the same PR. The production fail-closed branches are unchanged in this fix delta. No full-PR approval or full-gate claim. Final gate UNRUN; mergeability is not approved by this review. No new gap artifact or reviewer allocated.

Owner-attributed, reviewer-unverified native macOS receipts: focused test reported one pass/22 filtered; fresh-source/fresh-target direct-final-write mutant reported exit 101 at partial-ready-visibility assertion; restored source reported one pass. An earlier stale-test-binary mutant pass was DISCARDED and supplies no evidence. Native log paths reported by owner on Draco: /private/tmp/ir671-pr58-focused-head-5c4c1167.log, /private/tmp/ir671-pr58-mutant-direct-write-fresh-target.log, /private/tmp/ir671-pr58-restored-pass-fresh-target.log. Reviewer did not access these logs or run tests/probes. Runtime receipts do not remove the source-derived scheduling race.

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  fix_sha: 5c4c1167fe94ffde209e70099e2ee4d3e3063b30
  after_excerpt: "temporary = sys.argv[1] + '.tmp-' + str(os.getpid())\nwith open(temporary,\
    \ 'wb') as marker:\n    marker.write(payload[:8])\n    marker.flush()\n    os.fsync(marker.fileno())\n\
    \    with open(sys.argv[2], 'wb') as progress:\n        progress.write(b'partial')\n\
    \    time.sleep(0.2)\n    marker.write(payload[8:])\n    marker.flush()\n    os.fsync(marker.fileno())\n\
    os.replace(temporary, sys.argv[1])"
new_findings:
- fnd: FND-002
  severity: medium
  confidence: high
  method: code-review
  check: code-bug
  artifact_id: FR-013-AC-8
  related_ids: []
  path: quire-plugin-host/src/lib.rs
  lines: 2512-2519
  excerpt: Per-request memory and CPU-time budgets aggregate usage across the plugin
    process tree; a sampled budget breach terminates descendants and reports the corresponding
    failure. Memory sampling is best-effort between polls. If complete-tree measurement
    is unavailable, the host returns `ProcessBudgetError::Unavailable` before sending
    the request.
  finding: 'Partial-write visibility relies on a 200 ms sleep: the helper publishes
    .writing then automatically finishes and replaces ready, so if the Rust test is
    descheduled longer than that window, its !ready.exists() assertion fails even
    with correct atomic publication. Hold the helper at a bounded release handshake
    until the absence assertion completes.'
- fnd: FND-003
  severity: medium
  confidence: high
  method: code-review
  check: test-intent
  artifact_id: FR-013-AC-8
  related_ids: []
  path: quire-plugin-host/tests/fixtures/failure-plugin.py
  lines: 128-130
  excerpt: Per-request memory and CPU-time budgets aggregate usage across the plugin
    process tree; a sampled budget breach terminates descendants and reports the corresponding
    failure. Memory sampling is best-effort between polls. If complete-tree measurement
    is unavailable, the host returns `ProcessBudgetError::Unavailable` before sending
    the request.
  finding: Adding the writing argument shifted the request marker to sys.argv[6],
    but the escaped-descendant request handler still writes sys.argv[5], now the done
    path. The asserted request marker remains absent even after receipt, and the cleanup
    signal can be forged by request dispatch. Bind the handler to the actual request-marker
    argument and independently check cleanup.
bindings:
- test_id: tests::begin_request_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-8
  trace: correct
- test_id: tests::macos_cpu_budget_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-8
  trace: correct
- test_id: tests::macos_memory_budget_refuses_before_dispatch_for_escaped_descendant
  ac_id: FR-013-AC-8
  trace: correct
```

## New findings (disposition pass 2)

Reviewed: agent-ix/quire-driver@45f3ec950a0e73226160431a2ca8ddc0b3673ab1; narrow fix delta in the same host test/fixture paths. No production refusal changes.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The cleanup guard remains armed after both successful tests remove their release and stop paths. On function exit its Drop writes both paths again, leaving two temporary marker files per test run. Drop or disarm the guard before removing the files so the success path leaves no markers. | quire-plugin-host/src/lib.rs:1804; quire-plugin-host/src/lib.rs:2566; quire-plugin-host/src/lib.rs:2650 |

## Dispositions (round 2)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-002 | fixed | 45f3ec950a0e73226160431a2ca8ddc0b3673ab1 — bounded release handshake holds partial JSON until the absence assertion completes |
| FND-003 | fixed | 45f3ec950a0e73226160431a2ca8ddc0b3673ab1 — request-marker argv7 is distinct from done argv6; a real budget-free request asserts marker presence and no forged done |

## Disposition round 2 evidence

FND-001 remains fixed from round 1. FND-002/FND-003 are fixed by independent source inspection. New FND-004 remains open; disposition verdict CONDITIONAL. No whole-PR approval or merge approval. Full final gate UNRUN; mergeability reported false by dispatch. New behavioral regression executes real request dispatch and requires request-marker presence before signaling stop, so reverting argv7 to argv6 falsifies an actual observation. Handshake timeout exits without publishing ready; fixed sleep publication is removed. Guard now releases a waiting helper on unwind, but its success-path ordering leaks marker files.

Owner-attributed, reviewer-unverified receipts: two genuine mutants reported exit101 (release wait removal fails absence assertion; request-marker reversion to argv6 fails marker assertion); restored escaped test reported1/1, host library24/24, fmt/diff0. No full CI reported. No runtime execution or logs read by reviewer; earlier stale-binary mutant remains discarded.

```yaml
dispositions:
- fnd: FND-002
  outcome: fixed
  fix_sha: 45f3ec950a0e73226160431a2ca8ddc0b3673ab1
  after_excerpt: "    deadline = time.monotonic() + 5\n    while not os.path.exists(sys.argv[3])\
    \ and time.monotonic() < deadline:\n        time.sleep(0.005)\n    if not os.path.exists(sys.argv[3]):\n\
    \        raise SystemExit(6)\n    marker.write(payload[8:])\n    marker.flush()\n\
    \    os.fsync(marker.fileno())\nos.replace(temporary, sys.argv[1])"
- fnd: FND-003
  outcome: fixed
  fix_sha: 45f3ec950a0e73226160431a2ca8ddc0b3673ab1
  after_excerpt: "request = read_frame()\nif mode == \"escaped-descendant-memory\"\
    :\n    with open(sys.argv[7], \"wb\") as marker:\n        marker.write(b\"request\"\
    )"
new_findings:
- fnd: FND-004
  severity: low
  confidence: high
  method: code-review
  check: code-bug
  artifact_id: StopEscapedFixture::drop
  related_ids:
  - FR-013-AC-8
  path: quire-plugin-host/src/lib.rs
  lines: 1802-1805,2564-2570,2648-2654
  excerpt: "let _ = std::fs::write(&self.release, b\"release\");\n            let\
    \ _ = std::fs::write(&self.stop, b\"stop\");"
  finding: The cleanup guard remains armed after both successful tests remove their
    release and stop paths. On function exit its Drop writes both paths again, leaving
    two temporary marker files per test run. Drop or disarm the guard before removing
    the files so the success path leaves no markers.
bindings:
- test_id: tests::escaped_fixture_request_marker_is_distinct_from_cleanup_marker
  ac_id: FR-013-AC-8
  trace: correct
```

## New findings (disposition pass 3)

Reviewed: agent-ix/quire-driver@c956b08f7f0045f723fa4d2bceade7aa7208b270; base and merge-base independently verified as 4ffeab192cff070a3905364badf32d21fb18f8d2. Narrow guard fix plus normal merge resolution in host test/fixture paths only. PR diff has two files and no normative spec diff against the new base.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | After main redefines AC-8 as complete CPU accounting, the macOS CPU refusal, memory refusal, and request-marker fixture tests still actively Trace AC-8. They establish no complete CPU accounting. Bind CPU pre-dispatch Unavailable to AC-17 and memory pre-dispatch Unavailable to AC-18; the budget-free marker helper should make no unsupported AC-8 claim (use provenance or a valid owning criterion). | quire-plugin-host/src/lib.rs:2444; quire-plugin-host/src/lib.rs:2490; quire-plugin-host/src/lib.rs:2584; spec/functional/FR-013-host-process-plugins.md:154 |

## Dispositions (round 3)

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-004 | fixed | e7b18ac9d7bae024f6682ca9ca1e6934a6c0f979 — both success paths disarm before deleting markers; Drop writes only while armed |

## Disposition round 3 evidence

FND-001..003 retain their fixed outcomes; atomic publication, bounded release handshake and request-marker argv7 remain intact. FND-004 fixed by source inspection. New merge-scope FND-005 remains open; verdict CONDITIONAL. Main Unix-wide memory Unavailable test with AC18 is retained, including Linux-only injected measurement failure. Combined Linux memory AC8/AC16 trace is retained, and macOS fixtures are present. No runtime or complete-accounting enforcement is claimed for macOS.

Focused guard test exercises disarmed and armed Drop with absent/present marker assertions; removing either disarm guard or armed writes falsifies the corresponding observation. Both real fixture success paths disarm before removing files. No new build, test, probe, full gate, configuration/auth repair, reviewer or source edit. Owner-reported fmt0, guard2passed, macOS1passed, diff0 are attributed and reviewer-unverified, not reviewer execution. Full final gate remains UNRUN; remote mergeability reported MERGEABLE by root connector does not provide gate evidence or merge approval. SR5801 retains its historical original-head diff-scope verdict; no new whole-repository assurance claim.

```yaml
dispositions:
- fnd: FND-004
  outcome: fixed
  fix_sha: e7b18ac9d7bae024f6682ca9ca1e6934a6c0f979
  after_excerpt: "impl StopEscapedFixture {\n        fn disarm(&mut self) {\n    \
    \        self.armed = false;\n        }\n    }\n\n    impl Drop for StopEscapedFixture\
    \ {\n        fn drop(&mut self) {\n            if !self.armed {\n            \
    \    return;\n            }\n            let _ = std::fs::write(&self.release,\
    \ b\"release\");\n            let _ = std::fs::write(&self.stop, b\"stop\");\n\
    \        }\n    }"
new_findings:
- fnd: FND-005
  severity: medium
  confidence: high
  method: code-review
  check: trace
  artifact_id: FR-013-AC-8
  related_ids:
  - FR-013-AC-17
  - FR-013-AC-18
  path: quire-plugin-host/src/lib.rs
  lines: 2444,2490,2584
  excerpt: Per-request CPU-time accounting includes the plugin and every descendant
    from request start, including descendants that exit between polls or leave the
    plugin process group; a sampled subset is not complete accounting.
  finding: After main redefines AC-8 as complete CPU accounting, the macOS CPU refusal,
    memory refusal, and request-marker fixture tests still actively Trace AC-8. They
    establish no complete CPU accounting. Bind CPU pre-dispatch Unavailable to AC-17
    and memory pre-dispatch Unavailable to AC-18; the budget-free marker helper should
    make no unsupported AC-8 claim (use provenance or a valid owning criterion).
  test_id: tests::macos_cpu_budget_returns_unavailable_before_dispatch
  claimed_ac: FR-013-AC-8
  correct_ac: FR-013-AC-17
scope:
- id: FR-013-AC-8
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Per-request CPU-time accounting includes the plugin and every descendant
    from request start, including descendants that exit between polls or leave the
    plugin process group; a sampled subset is not complete accounting.
- id: FR-013-AC-17
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Before dispatching a CPU-budgeted request, the host establishes complete
    CPU accounting and bounded termination for that same boundary, or returns `ProcessBudgetError::Unavailable`
    without sending the request.
- id: FR-013-AC-18
  path: spec/functional/FR-013-host-process-plugins.md
  role: examined
  excerpt: Before dispatching a memory-budgeted request, the host establishes observation
    and bounded termination for the plugin and its complete live descendant set, or
    returns `ProcessBudgetError::Unavailable` without sending the request.
bindings:
- test_id: tests::macos_cpu_budget_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-8
  trace: wrong
  correct_ac: FR-013-AC-17
- test_id: tests::macos_memory_budget_refuses_before_dispatch_for_escaped_descendant
  ac_id: FR-013-AC-8
  trace: wrong
  correct_ac: FR-013-AC-18
- test_id: tests::escaped_fixture_request_marker_is_distinct_from_cleanup_marker
  ac_id: FR-013-AC-8
  trace: wrong
  correct_ac: null
- test_id: tests::begin_request_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-18
  trace: correct
- test_id: tests::process_memory_budget_includes_descendant_usage
  ac_id: FR-013-AC-16
  trace: correct
```

## Dispositions (round 4)

Reviewed: agent-ix/quire-driver@62dc73ccac325acd95634f1465d97c7e10ebbd6c; base/merge-base 4ffeab192cff070a3905364badf32d21fb18f8d2 independently verified through SSH and git ancestry. Delta against previous reviewed head c956b08f7f0045f723fa4d2bceade7aa7208b270 consists exactly of three Trace line corrections in quire-plugin-host/src/lib.rs.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-005 | fixed | 62dc73ccac325acd95634f1465d97c7e10ebbd6c — CPU refusal traces AC17; memory refusal traces AC18; marker helper cites TC026 without unsupported AC8 |

## Disposition round 4 evidence

**PASS for outstanding review findings at this exact head.** FND-001..004 retain their fixed outcomes. No new findings. Assertions, fixture logic, success-path guard disarm, bounded handshake, atomic publication, request-marker mapping, main Unix-wide unavailable/AC18 and combined Linux AC16 tags remain unchanged by this trace-only delta. CPU and memory refusal assertions match committed AC17/AC18; marker helper claims only TC026 fixture provenance.

This is a narrow disposition verdict, not whole-PR runtime/gate acceptance or merge approval. No reviewer builds, tests, probes, heavy runs, lock acquisition, source changes, config/auth repair or new reviewer. Owner reports three focused tests passed one each and fmt/diff0; attributed and reviewer-unverified, no native runtime credit. Final full gate UNRUN. SR5801 remains the historical original-head artifact; no new gap or whole-repository assurance claim.

```yaml
dispositions:
- fnd: FND-005
  outcome: fixed
  fix_sha: 62dc73ccac325acd95634f1465d97c7e10ebbd6c
  after_excerpt: '/// Trace: FR-013-AC-17, TC-026; reject unsupported macOS CPU accounting
    before dispatch.

    /// Trace: FR-013-AC-18, TC-026; fail closed when a descendant escapes the sampled
    process group.

    /// Trace: TC-026; verify the escaped fixture''s request marker names its own
    path.'
new_findings: []
bindings:
- test_id: tests::macos_cpu_budget_returns_unavailable_before_dispatch
  ac_id: FR-013-AC-17
  trace: correct
- test_id: tests::macos_memory_budget_refuses_before_dispatch_for_escaped_descendant
  ac_id: FR-013-AC-18
  trace: correct
helper_trace:
  test_id: tests::escaped_fixture_request_marker_is_distinct_from_cleanup_marker
  tc_id: TC-026
  unsupported_ac8_claim_removed: true
```
