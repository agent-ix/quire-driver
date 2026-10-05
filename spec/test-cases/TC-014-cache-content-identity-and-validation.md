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

Scope: FR-007-AC-1 through FR-007-AC-4.

## Test Procedure

Inject a recording `Cache` and provider spy. Return a miss, then a provider
result; inspect lookup's operation, package identity, canonical request,
`BackendId`, admitted `ManifestDigest`, generation target/options and the exact
typed value passed to `record`. Next return a valid typed hit and inspect the
unchanged result and provider call count. Change manifest content while keeping
identity and recorded pin fixed; provide the corresponding newly admitted
`ManifestDigest` and verify it reaches the interface. Exercise a typed cache
I/O error and excluded/oversize record outcomes.

## Expected Results

- Lookup receives all FR-007-AC-1 members including generate-only target and
  options; miss calls the provider exactly once and records/returns its same
  typed result.
- A valid typed hit is returned unchanged and does not increment provider calls;
  an always-miss fake fails this assertion.
- The changed manifest is represented by its newly admitted `ManifestDigest`,
  regardless of unchanged identity or pin.
- Typed I/O faults are not converted to misses; `NeverStore` and `NotStored`
  preserve the provider result.
- The recording interface is supplied by the test; the `quire-driver`
  package has no dependency on the concrete `quire-cache` package.
