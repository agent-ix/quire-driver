---
id: SR-14801
title: "Base checklist — IR-607 stdout-failure spec slice"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver; FR-002 AC-14..AC-16, TC-018 process-boundary procedures and expected results, FR-002/TC-018 tests.md rows"
review_set: subset
---

## Summary

Reviewed the new acceptance criteria and their process-level test oracle for ID, testability, consistency, dependency, and matrix status. The new criteria are sequential, use integration tests, remain untagged/planned, and tests.md reports that honestly; findings concern the FIFO test setup and one assertion that exceeds its mapped criterion.

## Verdict

**CONDITIONAL** — the FIFO procedure needs bounded rendezvous/cleanup, and the broken-pipe test asserts a diagnostic policy not stated by AC-14.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FIFO writer rendezvous has no deadline or cleanup rule; if the child exits before opening the FIFO, the parent can block indefinitely and leave the child running. | spec/test-cases/TC-018-binary-process-boundary.md:98-101 |
| FND-002 | medium | TC-018 requires no `OutputFailure` diagnostic on BrokenPipe, but FR-002-AC-14 only forbids selecting an output-failure category; the test oracle imposes an additional observable behavior. | spec/test-cases/TC-018-binary-process-boundary.md:101-104; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:136 |
