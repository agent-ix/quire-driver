---
id: TC-026
title: "Enforce plugin process-tree budgets and bounded cleanup"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
---
# TC-026: Enforce plugin process-tree budgets and bounded cleanup

## Description

Verify resource accounting, stderr handling, process-tree termination, and
worker cleanup for the host lifecycle.

Scope: FR-013-AC-8 through FR-013-AC-10.

## Test Procedure

Use synchronized fixtures that spawn a child process, exceed the memory
budget, exceed the CPU-time budget, write stderr beyond its retention bound,
close stdout and remain alive, and exit before the expected hello frame. For
each failure, retain the host result and fixture process identifiers; after the
host returns, check that neither the plugin nor its child remains alive and
that all host I/O workers have terminated.

## Expected Results

- Memory and CPU breaches identify the relevant budget and terminate both
  plugin and descendants without changing the failure classification.
- Stderr retention does not exceed the configured limit, excess bytes are
  drained, and captured stderr is available on success and failure.
- The host returns process-exit or typed failure information without a busy
  loop, and all I/O workers finish within the documented cleanup bound.
- Mutation checks removing the CPU-budget branch, process-tree termination,
  stderr truncation, or worker joining each cause this test to fail.
