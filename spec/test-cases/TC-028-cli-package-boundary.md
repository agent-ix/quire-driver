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

The process-provider lifecycle checks require the process backend mapping from
CG IR-629. Until that code is available, the package-graph and ownership checks
remain independently verifiable and the lifecycle checks remain planned.

Scope: FR-006-AC-1, FR-006-AC-3..FR-006-AC-5, FR-013-AC-11,
FR-013-AC-14..FR-013-AC-15,
FR-014-AC-1, FR-014-AC-2, FR-014-AC-4 and FR-014-AC-5.

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
7. Using the process-provider invocation from TC-013 and a fixture that starts
   a descendant which remains alive, let a plugin request finish normally.
   Confirm the CLI closes the session and the process tree exits before the
   CLI exits; if graceful shutdown expires, confirm forced cleanup completes
   within one second.
8. On Linux and macOS, for each plugin-session state (waiting to send hello,
   running a request, and waiting for process exit after shutdown), run one
   invocation that receives SIGINT and one that receives SIGTERM. Confirm the
   CLI requests QSL cancellation, waits for settlement or cancellation-grace
   expiry, terminates any remaining process tree, and exits only after cleanup
   completes within the FR-013 bound.
9. On Windows, start the same fixture under the CLI and terminate the CLI
   process while plugin descendants remain. Confirm the host assigns the
   plugin to its owning Job Object before allowing plugin code to execute,
   configures `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, and terminates all
   remaining job members on handle close.
10. Inject Job Object creation failure and child-assignment failure separately.
    Confirm each returns a typed host-start failure, no provider request is
    dispatched, no plugin hello marker is written, and any spawned child is
    terminated before CLI exit.

The signal and host-exit guarantees cover normal command termination and
handled SIGINT/SIGTERM; they do not cover an uncatchable host kill on Unix or a
descendant that deliberately leaves the host's process group.

Test locations: `quire-cli/tests/package_boundary.rs` for package ownership
and graph assertions; `quire-cli/tests/plugin_lifecycle.rs` for adapter
session cleanup and signal/host-exit behavior. Both remain planned until IR-672
code is implemented.

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
- Normal CLI termination and handled SIGINT/SIGTERM close active plugin trees
  within the defined bound; Windows Job Object handle closure terminates all
  remaining job members.
- Windows Job Object creation/assignment failures refuse plugin startup and
  do not leave a child process or dispatch a request.
