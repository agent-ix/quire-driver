---
id: TC-014
title: "Exercise driver calls through a recording cache interface"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-007
    type: verifies
---
# TC-014: Exercise driver calls through a recording cache interface

## Description

Verify that the driver sends the exact eligible request and manifest content to
its injected cache interface and preserves typed hit and miss results.

Scope: FR-007-AC-1 through FR-007-AC-2.

## Test Procedure

Inject a recording cache interface and a provider spy. On the first invocation,
return a miss and have the provider return a typed result. Inspect the recorded
operation, package identity, typed request, provider identity, manifest bytes
and value passed for recording. On a second invocation, return a typed hit and
inspect the driver's result and provider call count. On a third invocation,
change one manifest byte while keeping the provider identity and any recorded
revision or version pin unchanged; return a miss and inspect the new recorded
bytes.

## Expected Results

- The first cache call receives every named typed input and the exact admitted
  manifest bytes; the provider's typed result is passed for recording and
  returned unchanged.
- The hit result is returned unchanged and the provider spy's call count does
  not increase.
- The third cache call receives the changed manifest bytes, not a revision or
  version pin in their place.
- The recording interface is supplied by the test; the `quire-driver`
  package has no dependency on the concrete `quire-cache` package.
