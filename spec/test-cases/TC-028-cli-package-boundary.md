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

CG IR-629's typed process-backend mapping is available. Production
process-provider invocation and configuration forwarding remain IR-672
implementation work. Package-graph, adapter-fixture lifecycle, and cleanup
checks can run independently of production dispatch.

Scope: FR-002-AC-3, FR-002-AC-7, FR-006-AC-1, FR-006-AC-3..FR-006-AC-4,
FR-006-AC-8,
FR-006-AC-24..FR-006-AC-27, FR-013-AC-11,
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
   strict JSON file containing at least two entries for distinct fixture
   executables. Each fixture SHALL record its executable identity, exact argv,
   its `deadline_ms`, and hello-ready marker. Give each entry a distinct valid
   `deadline_ms` and distinct, non-default frame, total, stderr-retention, and
   cancellation-grace values, plus explicit optional
   process-memory and CPU-time values. Hold the fixture hellos at a test barrier
   and verify both hellos are admitted before the run begins backend
   negotiation. Use the real QSL/CG route to select the second configured
   provider. Verify only that provider receives the routed request, and verify
   the process-provider dispatch receives that entry's exact executable path,
   argv, `ProcessProviderLimits`, and `deadline_ms`; the first provider
   receives no request. Verify the selected provider's FR-331 request contains
   that entry's exact `deadline_ms`, unchanged.
   Include a separate entry with `argv: []` and verify the executable receives
   no extra arguments. The admitted hello is the sole source of each provider
   identity; the config contains no identity field. This end-to-end routed case
   remains planned until IR-672 implements production dispatch.
7. Run `quire run` without `--provider-config` and verify no process provider
   is registered or started. An explicit empty `providers` array has the same
   result. Verify the command does not read a default file, environment
   variable, or search path for provider configuration.
8. For each invalid configuration, run `quire run
   quire-cli/tests/fixtures/ir608-run.qsl yes --format json
   --provider-config <path>` and verify no provider process starts. Cover a
   missing or unreadable file; malformed JSON; missing or wrongly typed
   required fields; missing, zero, negative, fractional, null, wrongly typed,
   or greater-than-9,007,199,254,740,991 `deadline_ms`; unknown fields at the
   top-level, provider-entry and `limits` objects; and `null` or another wrongly
   typed value for either optional process budget. Add parser-level boundary
   cases that accept `deadline_ms` values 1 and 9,007,199,254,740,991; keep
   end-to-end dispatch fixtures' deadlines large enough for the fixture run.
   Verify each process exits with the existing QSL
   refusal category and writes exactly one `quire-outcome/1` for
   `Operation::Execute` with `Code::InvalidRequest` to stdout. Compare the
   document with the existing CLI input-refusal result, including its
   diagnostic; assert no configuration-specific code or cause is introduced.
9. Configure two distinct executables whose admitted hellos advertise the same
   identity. Verify QSL admission is invoked normally and its typed registration
   refusal, if any, is preserved. If QSL treats identical repeated admission as
   idempotent, verify the CLI returns its existing `InvalidRequest` input
   refusal instead. In either case, verify negotiation does not start, no
   request is sent, and all started sessions are closed.
10. Start the first configured fixture successfully, then make the later
    provider fail to start; repeat with a later provider that starts but returns
    an invalid hello. For each case, verify negotiation does not start, the
    earlier active session is closed and its process tree exits, and the
    original typed host failure is preserved without replacement.
11. Inspect the `quire-cli` process-provider adapter wiring. Confirm it receives
   the once-admitted descriptor from the host hello admitter, passes it to the
   driver's existing QSL registry without re-admitting it, and builds no
   second registry. Confirm it connects the descriptor to the process-provider
   seam. This admission-boundary check uses an adapter fixture; production
   provider invocation remains IR-672 implementation work.
12. On Linux and macOS, using an adapter fixture independent of CG routing and
   a plugin fixture that starts a descendant which remains alive, let a request finish
   normally. Confirm the CLI closes the session and the process tree exits
   before the CLI exits; if `RunLimits.cancellation_grace` expires first,
   confirm forced cleanup completes within one second. Windows forced cleanup
   uses the existing `terminate_tree`/`terminate_job` path and is verified in
   step 14; only automatic kill-on-close remains planned under IR-674.
13. On Linux and macOS only, for each plugin-session state, run one invocation
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
14. On Windows, verify forced plugin-tree cleanup through the host's existing
   `terminate_tree`/`terminate_job` path using a native Windows runner. The
   separate automatic kill-on-close mechanism is planned under IR-674; when
   implemented, verify Job Object assignment before plugin execution and
   descendant termination on owning-handle close. Record this as
   Windows-native evidence; do not count Linux or macOS results as Windows
   evidence.
15. Inspect the Windows host startup/error path in
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
Linux/macOS runners provide the signal evidence in step 13.

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
  malformed/unknown configuration or an invalid `deadline_ms` fails before
  spawn; each configured provider's executable, arguments, limits and
  `deadline_ms` reach the selected process-provider dispatch unchanged; and
  omitting the option (or providing an empty `providers` array) registers and
  starts no process providers, with no implicit deadline. End-to-end routed
  dispatch remains planned under IR-672. Invalid config in machine mode uses
  the existing
  `Operation::Execute` `Code::InvalidRequest` input-refusal document and
  refusal exit category, with no new code or cause. Optional CPU/RSS fields
  omitted from `limits` mean no such budget; explicit `null` and wrongly typed
  values are rejected before spawn.
- With two configured providers, both hellos are admitted before negotiation;
  the second provider can be selected and receives only its routed request,
  with its exact executable, argv, limits and `deadline_ms`. The hello
  descriptor remains the sole identity source.
- Distinct sessions with duplicate admitted identities cause no negotiation
  and all started sessions close. QSL registration refusals pass through
  unchanged; if QSL accepts identical registration idempotently, the CLI uses
  its existing `InvalidRequest` input refusal to fail closed.
- A later provider start or hello failure prevents negotiation, closes earlier
  active sessions and preserves the original typed host failure.
- The host hello admitter returns a once-admitted descriptor; the CLI passes it
  to the driver's existing registry without re-admission or a second registry.
- Normal CLI termination requests graceful shutdown on all supported adapter
  platforms; Linux/macOS handled SIGINT/SIGTERM close active plugin trees
  within the defined bound.
- Windows forced cleanup uses the merged host's explicit termination path;
  verify it on a native Windows runner. Automatic kill-on-close remains
  planned under IR-674, and setup-failure runtime evidence requires a native
  Windows runner and a usable failure seam.
