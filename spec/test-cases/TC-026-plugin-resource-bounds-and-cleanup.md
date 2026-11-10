---
id: TC-026
title: "Bound plugin stderr and preserve cleanup"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
---
# TC-026: Bound plugin stderr and preserve cleanup

## Description

Verify stderr retention and worker cleanup for the host lifecycle.

Scope: FR-013-AC-9 through FR-013-AC-10.

## Test Procedure

Use process fixtures that write stderr beyond the session retention limit,
close stdout while the process remains alive, and exit before the expected
hello frame. Exercise cleanup on each failure path. Retain the host result and
fixture process identifiers; after the host returns, check that the plugin
process tree has terminated and that all host I/O workers have stopped.

## Expected Results

- Stderr retention does not exceed the configured session limit, excess bytes
  are drained, and captured stderr is available on success and failure.
- After stdout closes, waiting for process exit does not busy-wait and preserves
  an available child exit status.
- On failure, the process tree is terminated and all I/O workers stop within
  one second after termination begins.
