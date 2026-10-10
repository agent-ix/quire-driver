---
id: SR-5801
title: gap-analysis of PR58 macOS fail-closed budgets
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-driver@252a3f70606603da761559b0ee58c0aecac3551f; quire-plugin-host/src/lib.rs,
  quire-plugin-host/tests/fixtures/failure-plugin.py; FR-013-AC-8 (IR-671)
review_set: subset
---

## Summary

Ticket: IR-671. Diff-scoped traceability, reverse ownership and stub review of the macOS budget-refusal slice. FR-013-AC-8 is tagged and owns both changed monitor branches and the two behavioral tests.

## Verdict

**PASS** — No diff-scoped trace or ownership gap; not an execution or whole-repository gate verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Review is limited to the PR diff. Rust idiom checklist applied before analysis and again as formal pre-handoff review; repository AGENTS.md/CLAUDE.md, safety/lint configuration and portable rust-style defaults read. No applicable AssuranceProfile found. No normative spec or plan diff: spec-review submethods do not apply.

Mac CPU and memory tests invoke the real PluginSession request path, require typed Unavailable and no writer dispatch; the escape fixture uses start_new_session and compares measured process-group identities. Removing the refusal causes expect_err or typed-cause assertions to fail. Existing test instrumentation observes entry into write dispatch; it does not alter behavior. Bounded marker polling checks a condition, rather than assuming readiness after a fixed sleep.

begin_request fails before ID advancement, serialization, and writer dispatch. Original cancellation/deadline precedence, abort diagnostics and cleanup/error custody are unchanged. Budget-free hello and requests retain the early return. Linux accounting and other-platform refusal logic are unchanged. macOS enforcement and complete escaped-tree termination are not implemented or claimed by this slice.

No builds, test executions or probes were run, per dispatch. Linux final full gate pending with lead. macOS candidate make ci was reported by a peer; no receipt was available in reviewer custody, so it is not reviewer-verified execution evidence.

ID collision check found no SR-5800/5801 in reviewed repository or existing scratch; reservation evidence came from dispatch, not ticket authorization. This is best available local evidence, not global proof.

Plan completion: not assessed

Reconciliation: quire matrix (quire 0.36.2, engine 0.50.2), no run evidence read. Computed whole-repository matrix: 133 criteria, 82 tagged, 49 untagged, 2 method-without-symbol. These are context counts, not a whole-repository PASS: the dispatch restricts review to changed behavior. The one examined AC, FR-013-AC-8, is tagged, including both new macOS tests. Out-of-diff gaps are not findings in this pass. Reverse inventory: two changed monitor branches and two new tests, all owned by AC8; zero new untraced behaviors, source stubs or hollow tests. Optional semantic gap-analysis not run; code-review independently checks assertion falsifiability.

Matrix emits preexisting DuplicateArchetype/InverseEdge first-wins diagnostics. Coverage reports absent inspection/interface/NFR/StR/suite archetypes, out-of-diff nonbinding FR075 and uncatalogued verification cells; AC8 is present with Test method and concrete binders. No stale/untracked tag appears on the changed tests. The separate SR-5800 test-publication race remains open; this traceability PASS does not clear it.

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
