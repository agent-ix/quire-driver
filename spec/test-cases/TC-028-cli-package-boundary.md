---
id: TC-028
title: "Keep the CLI package separate from the driver and compose the cache"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-014
    type: verifies
---
# TC-028: Keep the CLI package separate from the driver and compose the cache

## Description

Verify workspace membership, executable ownership and the CLI dependency edges
without adding the downstream cache to the driver library closure.

Scope: FR-014-AC-1 through FR-014-AC-3.

## Test Procedure

1. Run `cargo metadata --locked --offline --format-version 1` and identify the
   workspace members and package targets for `quire-cli` and `quire-driver`.
2. Inspect normal dependency edges from `quire-cli`; confirm they include
   `ix-cli-kit`, `quire-driver` and `quire-cache`.
3. Traverse the complete normal/build dependency closure from `quire-driver`.
   Check that no package in the closure is `quire-cache`.
4. Confirm `quire-driver` has no `quire` binary target and `quire-cli` has the
   sole `quire` binary target.

Test location: `tests/dependency_edges.rs`, added with this requirement.

## Expected Results

- Both packages are workspace members and are distinct package identities.
- The `quire` binary target belongs to `quire-cli` only.
- `quire-cli` directly depends on `ix-cli-kit`, `quire-driver` and
  `quire-cache`.
- The full normal/build dependency closure from `quire-driver` excludes
  `quire-cache`.
