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

Verify workspace membership, executable ownership and the CLI dependency edges.

Scope: FR-014-AC-1, FR-014-AC-2, FR-014-AC-4 and FR-014-AC-5.

## Test Procedure

1. Run `cargo metadata --locked --offline --format-version 1` and identify the
   workspace members and package targets for `quire-cli` and `quire-driver`.
2. Inspect normal dependency edges from `quire-cli`; confirm they include
   `quire-driver` and `quire-cache`.
3. Confirm exactly one workspace package provides the `quire` binary target
   and that package is `quire-cli`.
4. Inspect the package test layout. Confirm CLI contract and binary
   process-boundary tests and their fixtures are under the `quire-cli` package.
5. Inspect the package source layout. Confirm the argument parser is
   `quire-cli/src/cli.rs` and the binary entry point is `quire-cli/src/main.rs`.

Test location: `quire-cli/tests/package_boundary.rs`, added with this
requirement.

## Expected Results

- `quire-cli` and `quire-driver` are distinct workspace packages.
- Only `quire-cli` provides the `quire` binary target.
- `quire-cli` directly depends on `quire-driver` and `quire-cache`.
- The CLI tests and their fixtures have the specified `quire-cli` ownership.
- The parser and binary entry point have the specified `quire-cli` locations.
