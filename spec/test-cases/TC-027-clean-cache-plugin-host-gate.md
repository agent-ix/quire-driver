---
id: TC-027
title: "Run plugin-host tests and dependency checks from a clean cache"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
---
# TC-027: Run plugin-host tests and dependency checks from a clean cache

## Description

Verify the repository test target provisions all locked platform dependencies
before TC-005 performs offline Cargo metadata inspection.

Scope: FR-013-AC-12 through FR-013-AC-13.

## Test Procedure

In an isolated Cargo home with no registry or git package cache, run `make
test` using the repository lockfile and normal network access. Observe the
fetch step, then the offline metadata check and complete test suite. Repeat
without network after the fetch has populated the isolated cache.

## Expected Results

- The first run fetches every lockfile platform dependency before offline
  metadata inspection, including dependencies not built for the host target.
- The dependency-edge check and full `make test` pass. The second run passes
  without network because the prior locked fetch populated the cache.
- If the plugin fixture runtime is absent, the host test suite fails with an
  actionable message naming the missing runtime and installation instructions.
