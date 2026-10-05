---
id: TC-025
title: "Preserve plugin settlement during cancellation and bound deadlines"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
---
# TC-025: Preserve plugin settlement during cancellation and bound deadlines

## Description

Verify PL-6 cancellation grace and PL-7 deadline behavior while plugin I/O is
pending.

Scope: FR-013-AC-5 and FR-013-AC-6.

## Test Procedure

Use a fixture that receives cancel and returns a valid, correlated incomplete
settlement within grace; a second fixture that never settles; and a third that
completes hello but never reads a request frame. Trigger cancellation in the
first two cases. Give the third case a 500 ms deadline, a request larger than
the pipe capacity, and a bounded termination grace. Use synchronization
handshakes rather than sleeps to arrange fixture states; measure deadline
return against the configured grace and documented polling interval.

## Expected Results

- The first case returns the plugin's validated settlement unchanged.
- The second case terminates the process tree after grace and returns the
  cancellation outcome.
- The blocked-write case returns `TimedOut` within the configured bound and
  does not wait for the plugin to read or exit voluntarily.
- Malformed frames or plugin faults received during grace retain their typed
  failure instead of becoming cancellation or timeout.
- Mutation checks removing the valid-settlement path, grace expiry, or
  cancellation-aware write each cause this test to fail.
