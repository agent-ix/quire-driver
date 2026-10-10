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

Scope: FR-013-AC-8 through FR-013-AC-10 and FR-013-AC-16 through FR-013-AC-19.

## Test Procedure

Use synchronized fixtures that spawn a child process and make the combined
plugin-plus-child resident memory exceed the request limit while each process
alone remains below it. For CPU accounting, run two cases: one child exits and
is reaped between monitor polls; in the other, the child leaves the plugin
process group and remains alive at breach. In each case, synchronize plugin-only
and child-only CPU workloads. Set the budget above the CPU consumed by either
workload alone and below their combined CPU use. Keep the plugin alive until
the child has completed its assigned CPU workload in the exited-child case;
keep both processes alive until the combined budget breach in the escaped-child
case. Assert the typed CPU-budget failure in both cases. On a runtime that
cannot establish complete accounting and bounded termination for the full
boundary, assert `ProcessBudgetError::Unavailable` before any request frame is
sent, and assert that neither workload starts. Include a monitor test double
that cannot observe and terminate the complete current process tree, plus
fixtures that write stderr beyond its session retention bound, close stdout and
remain alive, and exit before the expected hello frame. For every budget
failure, retain the host result and fixture process identifiers; after the host
returns, check that neither the plugin nor any descendant, including the
escaped child, remains alive and that all host I/O workers have terminated.

## Expected Results

- The host samples resource use at intervals no longer than 20 ms. On a runtime
  supporting the complete boundary, both synchronized CPU cases return the
  typed CPU-budget failure although the plugin-only and child-only usage are
  each below the cap. This makes the exited-between-polls and escaped
  descendant contributions observable without exposing an internal CPU total.
- When the runtime cannot establish complete CPU accounting together with
  bounded termination of that same boundary, it returns
  `ProcessBudgetError::Unavailable` before sending any request frame. The same
  fail-closed result applies when the complete live memory tree cannot be
  observed and boundedly terminated.
- Memory is sampled as the current process-tree resident-byte sum at each poll;
  a transient peak between polls may exceed the configured value.
- A measured budget breach returns its typed CPU or memory failure and
  terminates every member of the established resource boundary within one
  second after termination begins. The escaped descendant is alive when the
  budget trips and is confirmed gone when the host returns.
- Stderr retention does not exceed the configured session limit, excess bytes
  are drained, and captured stderr is available on success and failure.
- The host returns process-exit or typed failure information without a busy
  loop; the process tree is terminated and all I/O workers finish within one
  second after termination begins.
- Mutation checks removing the CPU-budget branch, process-tree termination,
  stderr truncation, or worker joining each cause this test to fail.
