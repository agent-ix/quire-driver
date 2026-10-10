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
alone remains below it. For CPU accounting, exercise both a descendant that
exits and is reaped between monitor polls and a descendant that leaves the
plugin process group before consuming CPU and exiting. Keep the plugin alive
through each measurement. On a runtime with a complete accounting boundary,
verify that both descendants' CPU contributes to the request total. On a
runtime without that capability, verify `ProcessBudgetError::Unavailable`
occurs before the request frame is sent. Include a monitor test double that
cannot observe the complete process tree, plus fixtures that write stderr
beyond its session retention bound, close stdout and remain alive, and exit
before the expected hello frame. For every budget failure, retain the host
result and fixture process identifiers; after the host returns, check that
neither the plugin nor its child remains alive and that all host I/O workers
have terminated.

## Expected Results

- The host samples resource use at intervals no longer than 20 ms. CPU totals
  include descendants that exit between polls and descendants that leave the
  plugin process group; process-group sampling alone is not accepted as
  complete accounting. Memory sampling is best-effort, so a transient peak can
  exceed the configured value before the next measurement.
- When the runtime cannot establish complete CPU accounting or observe the
  full memory tree, the host returns `ProcessBudgetError::Unavailable` before
  sending any request frame.
- Stderr retention does not exceed the configured session limit, excess bytes
  are drained, and captured stderr is available on success and failure.
- The host returns process-exit or typed failure information without a busy
  loop; the process tree is terminated and all I/O workers finish within one
  second after termination begins.
- Mutation checks removing the CPU-budget branch, process-tree termination,
  stderr truncation, or worker joining each cause this test to fail.
