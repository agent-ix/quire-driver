---
id: TC-028
title: "Keep the CLI package separate from the driver and compose cache and host"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-006
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-014
    type: verifies
---
# TC-028: Keep the CLI package separate from the driver and compose cache and host

## Description

Verify workspace membership, executable ownership and the CLI dependency edges,
including that the CLI composes the process host while the driver core excludes
it.

Scope: FR-006-AC-1, FR-013-AC-11, FR-014-AC-1, FR-014-AC-2, FR-014-AC-4
and FR-014-AC-5.

## Test Procedure

1. Run `cargo metadata --locked --offline --format-version 1` and identify the
   workspace members and package targets for `quire-cli` and `quire-driver`.
2. Inspect normal dependency edges from `quire-cli`; confirm they include
   `quire-driver`, `quire-cache` and `quire-plugin-host`. Inspect the driver
   dependency closure and confirm it excludes `quire-plugin-host`.
3. Confirm exactly one workspace package provides the `quire` binary target
   and that package is `quire-cli`.
4. Inspect the package test layout. Confirm CLI contract and binary
   process-boundary tests and their fixtures are under the `quire-cli` package.
5. Inspect the parser definition and entry point. Confirm the CLI parsing
   definitions are implemented in `quire-cli/src/cli.rs`, the binary entry
   point is `quire-cli/src/main.rs`, and the `quire-driver` package's
   `src/lib.rs` does not export a `cli` module.
6. Inspect the `quire-cli` process-provider adapter wiring. Confirm it obtains
   the plugin hello manifest, supplies it for admission to the driver's single
   QSL registry, and connects the admitted process provider to the driver's
   process-provider seam.

Test location: `quire-cli/tests/package_boundary.rs`, added with this
requirement.

## Expected Results

- `quire-cli` and `quire-driver` are distinct workspace packages.
- Only `quire-cli` provides the `quire` binary target.
- `quire-cli` directly depends on `quire-driver`, `quire-cache` and
  `quire-plugin-host`; `quire-driver` has no normal or build dependency on the
  host.
- The CLI tests and their fixtures have the specified `quire-cli` ownership.
- The parser definitions and binary entry point have the specified
  `quire-cli` locations, and the `quire-driver` package's `src/lib.rs` does
  not export a `cli` module.
- The `quire-cli` adapter supplies the plugin hello manifest for admission to
  the driver's registry and connects the admitted provider to the seam.
