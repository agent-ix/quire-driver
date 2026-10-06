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

Use synchronized fixtures that spawn a child process and make the combined
plugin-plus-child resident memory exceed the request limit while each process
alone remains below it. Repeat for summed CPU-time deltas. Include a monitor
test double that cannot observe the complete process tree, plus fixtures that
write stderr beyond its session retention bound, close stdout and remain alive,
and exit before the expected hello frame. For every budget failure, retain the
host result and fixture process identifiers; after the host returns, check
that neither the plugin nor its child remains alive and that all host I/O
workers have terminated.

## Expected Results

- The host samples the complete process tree at intervals no longer than 20 ms.
  A sampled aggregate memory or CPU-time budget breach identifies its typed
  failure and terminates the plugin and descendants. The sample is best-effort:
  a transient peak between polls may exceed the configured value before the
  next measurement.
- When the monitor cannot observe the complete process tree, the host returns
  `ProcessBudgetError::Unavailable` before sending any request frame.
- Stderr retention does not exceed the configured session limit, excess bytes
  are drained, and captured stderr is available on success and failure.
- The host returns process-exit or typed failure information without a busy
  loop; the process tree is terminated and all I/O workers finish within one
  second after termination begins.
- Mutation checks removing the CPU-budget branch, process-tree termination,
  stderr truncation, or worker joining each cause this test to fail.
