---
id: TC-028
title: "Keep the CLI package separate from the driver and compose cache and host"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: verifies
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

Scope: FR-002-AC-3, FR-002-AC-7, FR-006-AC-1, FR-006-AC-3..FR-006-AC-4,
FR-006-AC-24, FR-013-AC-11,
FR-013-AC-14..FR-013-AC-15,
FR-014-AC-1, FR-014-AC-2, FR-014-AC-4..FR-014-AC-6.

## Test Procedure

1. Run `cargo metadata --locked --offline --format-version 1` and identify the
   workspace members and package targets for `quire-cli` and `quire-driver`.
2. Inspect normal dependency edges from `quire-cli`; confirm they include
   `quire-driver`, `quire-cache`, `quire-plugin-host` and `ix-cli-kit`. Inspect
   the driver dependency closure and confirm it excludes both
   `quire-plugin-host` and `ix-cli-kit`.
3. Confirm exactly one workspace package provides the `quire` binary target
   and that package is `quire-cli`.
4. Inspect the package test layout. Confirm CLI contract and binary
   process-boundary tests and their fixtures are under the `quire-cli` package.
5. Inspect the parser definition and entry point. Confirm the CLI parsing
   definitions are implemented in `quire-cli/src/cli.rs`, the binary entry
   point is `quire-cli/src/main.rs`, and the `quire-driver` package's
   `src/lib.rs` does not export a `cli` module.
6. Exercise the production `quire run --provider-config <path>` command with a
   temporary strict JSON file containing a `providers` array and one entry with
   `executable`, `argv`, and `limits`. Give the entry explicit
   `max_frame_bytes`, `max_total_bytes`, `max_stderr_bytes`, and
   `cancellation_grace_ms` values, plus optional
   `max_process_memory_bytes` and `max_process_cpu_time_ms` values. Use a real
   fixture executable whose hello manifest supplies its provider identity.
   Verify the once-admitted hello identity is the identity used for registry
   selection and the selected process-provider dispatch uses that executable,
   exact argument vector (including an empty vector in a separate case), and
   the exact `ProcessProviderLimits`; the fixture receives the routed request.
   This end-to-end routed case remains planned until CG IR-629 supplies the
   process-backend mapping. Also verify a missing/unreadable config file,
   malformed JSON, absent or wrongly typed required fields, and unknown fields
   at each object level fail before a provider process starts. Verify omitted
   optional CPU or memory fields mean `None`, without an implicit budget. Run
   Verify the config has no caller-supplied identity field: the admitted hello
   descriptor supplies the provider identity used for selection. Run
   `quire run` without `--provider-config` and verify no process provider is
   registered or started; an explicit empty `providers` array has the same
   result. The command SHALL NOT read a default file, environment variable, or
   search path for provider configuration. The configuration SHALL NOT carry a
   second provider identity that could disagree with the admitted hello.
7. Inspect the `quire-cli` process-provider adapter wiring. Confirm it receives
   the once-admitted descriptor from the host hello admitter, passes it to the
   driver's existing QSL registry without re-admitting it, and builds no
   second registry. Confirm it connects the descriptor to the process-provider
   seam. End-to-end provider routing remains blocked on CG IR-629; this
   admission-boundary check uses an adapter fixture.
8. On Linux and macOS, using an adapter fixture independent of CG routing and
   a plugin fixture that starts a descendant which remains alive, let a request finish
   normally. Confirm the CLI closes the session and the process tree exits
   before the CLI exits; if `RunLimits.cancellation_grace` expires first,
   confirm forced cleanup completes within one second. Windows forced cleanup
   uses the existing `terminate_tree`/`terminate_job` path and is verified in
   step 10; only automatic kill-on-close remains planned under IR-674.
9. On Linux and macOS only, for each plugin-session state, run one invocation
   that receives SIGINT and one that receives SIGTERM: wait for a `started`
   marker before hello is written, wait for a request-running marker, and wait
   for a shutdown-waiting marker after the shutdown frame. Confirm the CLI
   requests cancellation, waits for settlement or
   `RunLimits.cancellation_grace` expiry, terminates any remaining process
   tree, restores the default disposition and re-raises the received signal.
   Confirm the parent observes signal termination for that exact signal (not
   an integer exit code), stdout contains no `quire-outcome/1` document, and no
   QSL cause is claimed. With no active session, confirm the CLI retains the
   platform's default signal disposition. Windows console control events are
   outside this signal guarantee.
10. On Windows, verify forced plugin-tree cleanup through the host's existing
   `terminate_tree`/`terminate_job` path using a native Windows runner. The
   separate automatic kill-on-close mechanism is planned under IR-674; when
   implemented, verify Job Object assignment before plugin execution and
   descendant termination on owning-handle close. Record this as
   Windows-native evidence; do not count Linux or macOS results as Windows
   evidence.
11. Inspect the Windows host startup/error path in
    `quire-plugin-host/src/lib.rs` for Job Object creation and assignment
    failures. Record whether failures are distinguishable as typed start
    errors before provider dispatch and child cleanup. Runtime fault-injection
    coverage remains planned until an injectable or distinguishable failure
    seam exists; use native Windows evidence when available.

The signal guarantee covers Linux/macOS handled SIGINT/SIGTERM only; it does
not cover Windows console control events, an uncatchable host kill on Unix, or
a plugin descendant that deliberately leaves the host's process group.

Test locations: `tests/dependency_edges.rs` and
`quire-cli/tests/package_boundary.rs` already assert the CLI-to-host edge.
The package-boundary test currently asserts
`!dependencies.contains("ix-cli-kit")`, the opposite of the new CLI-to-kit
requirement, so its FR-014-AC-2 Trace tag is removed until the IR-607 code slice
inverts that assertion, verifies the positive CLI-to-kit edge, and asserts the
driver closure excludes the kit. The planned
`quire-cli/tests/plugin_lifecycle.rs` will cover adapter session cleanup and
signals. Windows-specific evidence is planned for a native Windows runner; no
Windows runtime coverage is claimed from the current Ubuntu-only CI. Native
Linux/macOS runners provide the signal evidence in step 9.

## Expected Results

- `quire-cli` and `quire-driver` are distinct workspace packages.
- Only `quire-cli` provides the `quire` binary target.
- `quire-cli` directly depends on `quire-driver`, `quire-cache`,
  `quire-plugin-host` and `ix-cli-kit`; the existing tests assert the
  CLI-to-host edge. The IR-607 code slice inverts the existing negative kit
  assertion and verifies that the driver closure has no normal or build
  dependency on the host or ix-cli-kit.
- The CLI tests and their fixtures have the specified `quire-cli` ownership.
- The parser definitions and binary entry point have the specified
  `quire-cli` locations, and the `quire-driver` package's `src/lib.rs` does
  not export a `cli` module.
- `quire run --provider-config <path>` reads only the named strict JSON file;
  malformed/unknown configuration fails before spawn, configured executable,
  arguments and limits reach the selected process-provider dispatch unchanged,
  and omitting the option (or providing an empty `providers` array) registers
  and starts no process providers. End-to-end routed dispatch remains planned
  until CG IR-629 provides the process-backend mapping.
- The host hello admitter returns a once-admitted descriptor; the CLI passes it
  to the driver's existing registry without re-admission or a second registry.
- Normal CLI termination requests graceful shutdown on all supported adapter
  platforms; Linux/macOS handled SIGINT/SIGTERM close active plugin trees
  within the defined bound.
- Windows forced cleanup uses the merged host's explicit termination path;
  verify it on a native Windows runner. Automatic kill-on-close remains
  planned under IR-674, and setup-failure runtime evidence requires a native
  Windows runner and a usable failure seam.
