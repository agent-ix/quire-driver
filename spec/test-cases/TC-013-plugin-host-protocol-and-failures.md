---
id: TC-013
title: "Reject malformed plugin exchanges and preserve item-local outcomes"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-006
    type: verifies
---
# TC-013: Reject malformed plugin exchanges and preserve item-local outcomes

## Description

Verify the plugin host's explicit executable boundary, canonical framing,
manifest admission, per-run budgets and typed item-local failure behavior.

Scope: FR-006-AC-1 through FR-006-AC-6.

## Test Procedure

Run a fixture plugin that accepts a valid `hello` frame, receives a routed
request and returns a typed result. Repeat with one fault at a time: an
unlisted executable path, a non-canonical frame, a wrong protocol identity,
a wrong request id, an oversized frame, a total-byte overrun, a crash, a
non-zero exit, a missing terminal record, an unrouted-item result, an
unadvertised-kind result, a deadline, and cancellation past the grace period.
Use a request that also routes an unrelated item to a second provider. Include
one reproduced counterexample and one provider `proved` result.

Test location: planned plugin-host integration tests.

## Expected Results

- The host starts only the explicitly named executable and never loads a
  dynamic library or discovers a plugin.
- Only the canonical hello manifest is admitted, and the resulting backend
  identity comes from that manifest.
- Each malformed or over-budget exchange is rejected before an invalid
  result can become a verdict.
- Plugin failures settle only the items routed to that plugin as `failed`;
  timeout and cancellation settle open items `incomplete`. The unrelated
  provider's item is unchanged.
- The counterexample is `refuted` only after replay reproduces it. The plugin
  proof is marked `trusted` and names the plugin identity.
