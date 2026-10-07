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

Only end-to-end process-provider routing requires the process backend mapping
from CG IR-629. Package-graph, adapter-fixture lifecycle, and cleanup checks do
not require that mapping and can run independently of it.

Scope: FR-006-AC-1, FR-006-AC-3..FR-006-AC-4, FR-013-AC-11,
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
6. Inspect the `quire-cli` process-provider adapter wiring. Confirm it receives
   the once-admitted descriptor from the host hello admitter, passes it to the
   driver's existing QSL registry without re-admitting it, and builds no
   second registry. Confirm it connects the descriptor to the process-provider
   seam. End-to-end provider routing remains blocked on CG IR-629; this
   admission-boundary check uses an adapter fixture.
7. On Linux and macOS, using an adapter fixture independent of CG routing and a plugin fixture
   that starts a descendant which remains alive, let a request finish
   normally. Confirm the CLI closes the session and the process tree exits
   before the CLI exits; if `RunLimits.cancellation_grace` expires first,
   confirm forced cleanup completes within one second. Windows forced
   process-tree cleanup remains planned under IR-674.
8. On Linux and macOS only, for each plugin-session state, run one invocation
   that receives SIGINT and one that receives SIGTERM: wait for a `started`
   marker before hello is written, wait for a request-running marker, and wait
   for a shutdown-waiting marker after the shutdown frame. Confirm the CLI
   requests cancellation, waits for settlement or
   `RunLimits.cancellation_grace` expiry, terminates any remaining process
   tree, exits with the conventional signal-terminated status, and makes no
   QSL cause claim. With no active session, confirm the CLI retains the
   platform's default signal disposition. Windows console control events are
   outside this signal guarantee.
9. Windows Job Object behavior is planned under IR-674 and has no current
   implementation claim. When a supported mechanism and native Windows runner
   are available, verify assignment before plugin execution and descendant
   termination on owning-handle close. Record this as Windows-native evidence;
   do not count Linux or macOS results as Windows evidence.
10. Inspect the Windows host startup/error path in
    `quire-plugin-host/src/lib.rs` for Job Object creation and assignment
    failures. Record whether failures are distinguishable as typed start
    errors before provider dispatch and child cleanup. Runtime fault-injection
    coverage remains planned until an injectable or distinguishable failure
    seam exists; use native Windows evidence when available.

The signal guarantee covers Linux/macOS handled SIGINT/SIGTERM only; it does
not cover Windows console control events, an uncatchable host kill on Unix, or
a plugin descendant that deliberately leaves the host's process group.

Test locations: existing `quire-cli/tests/package_boundary.rs` covers package
ownership and layout; its FR-014-AC-2 Trace tag currently overstates coverage
because it checks driver/cache but not the CLI-to-host edge. Remove that tag and
add the missing edge assertion in the IR-672 code slice. The planned
`quire-cli/tests/plugin_lifecycle.rs` will cover adapter session cleanup and
signals. Windows-specific evidence is planned for a native Windows runner; no
Windows runtime coverage is claimed from the current Ubuntu-only CI. Native
Linux/macOS runners provide the signal evidence in step 8.

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
- The host hello admitter returns a once-admitted descriptor; the CLI passes it
  to the driver's existing registry without re-admission or a second registry.
- Normal CLI termination requests graceful shutdown on all supported adapter
  platforms; Linux/macOS handled SIGINT/SIGTERM close active plugin trees
  within the defined bound.
- Windows Job Object behavior remains planned under IR-674; step 10 currently
  specifies static inspection only, with runtime evidence requiring a native
  Windows runner and a usable setup-failure seam.
