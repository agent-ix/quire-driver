---
id: TC-012
title: "Register and invoke a first-party provider through Provider"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-005
    type: verifies
---
# TC-012: Register and invoke a first-party provider through Provider

## Description

Verify the linked Kani Provider slice: manifest admission and lookup, Kani
preflight refusals, CG-owned terminal mapping, and the generic result carrier
used by non-Kani providers. Full routed invocation and process-provider
terminal integration remain planned.

Scope: FR-005-AC-1 through FR-005-AC-11, FR-005-AC-16 through FR-005-AC-17,
and FR-005-AC-20 through FR-005-AC-22.

## Test Procedure

1. Register two providers with distinct identities and verify the QSL
   registry admits both and the driver resolves each identity to its provider.
2. Register identical linked manifests in both input orders. Verify QSL
   admits one descriptor and driver lookup does not select one implementation
   according to input order.
3. Exercise Kani item preflight with cancellation, backend mismatch, missing
   execution limits, and missing harness; verify each retains its typed
   failure.
4. Call the public `Provider::run` with a selected generated Kani item but no
   execution limits; verify it returns one typed missing-limits result.
5. Generate a Kani item with explicit `ProofCeilings`; verify the harness
   identity carries those exact memory and wall-clock values. Verify a typed
   `ProviderItemResult` retains CG's execution ceilings, memory observation
   and symbolic-argument bounds. Supply a different memory or wall-clock
   ceiling and verify a typed mismatch refusal occurs before CG execution.
6. Exercise Kani request construction with an explicitly supplied absolute
   launcher home directory and verify the same path reaches CG's execution
   request. Repeat with the home directory unknown while a child test process
   has a decoy `HOME`; verify the CG request receives `None`, not the
   environment value or a path inferred from another directory. The focused
   assertion is `src/provider.rs::kani_request_forwards_only_the_explicit_home_directory`.
7. Pass CG evidence with a `NoVerdict` diagnostic record through the typed
   Kani result carrier and verify its structured fields are unchanged. Repeat
   with diagnostics absent and verify they remain absent. Verify the driver
   does not parse diagnostic text or create a `KaniFailureCause`; see
   `src/provider.rs::provider_result_preserves_optional_cg_diagnostics_and_failure_cause`.
8. Construct generic provider results with non-Kani success and failure types.
   Verify each retains its request index and backend identity. Exercise the
   host adapter conversion with a non-Kani result carrier and verify its typed
   result reaches the caller unchanged.
9. Pass a cover-unsatisfied outcome and a timeout outcome through CG's
   `run_terminal_value`; verify they map to `Proved { success_checks: 0 }`
   and `Incomplete(TimedOut)` respectively.
10. The origin-aware QSL-to-CG descriptor projection is tested in
   `src/drive.rs`: build separate QSL registries with otherwise equal `kani`
   descriptors, one `Linked` and one `Process`, then verify their CG
   descriptor projections preserve identity, advertised pairs and the supplied
   origin. Then verify the driver's pre-negotiation conversion calls CG's
   `BackendKind::from_descriptor`: the same unknown identity is refused when
   Linked and becomes `Process(identity)` when Process. The driver does not
   construct that kind or infer it from identity. The Linked-origin half of
   FR-005-AC-1 is tested in `src/registry.rs`. Process-origin plugin hello
   registration through the `quire-cli` process-provider adapter remains
   planned.

11. Verify the Kani output's terminal is the one produced by CG's
   `run_terminal_value` from the original evidence and optional replay
   settlement. Include `NoVerdict` with and without diagnostics and
   `FailedWithoutCounterexample` with absent and present causes. Confirm the
   driver forwards only CG's actual optional cause and diagnostics do not
   change the terminal. `src/provider.rs::cg_terminal_mapping_reads_original_failure_cause_not_diagnostics`
   checks cause forwarding and diagnostic neutrality, while
   `src/provider.rs::falsified_scalar_does_not_guess_a_replay_settlement`
   checks the optional replay settlement reaches CG's map.
12. Planned under QSL-655: keep a plugin counterexample in the shared carrier
    as a non-terminal candidate until QSL S6a replay reproduces it; do not emit
    `Refuted` before replay.
13. Build a linked provider registry from a manifest whose `wire` is absent.
    Verify `build_registry` retains canonical FR-331 bytes, includes present
    optional members and omits absent ones. Pass a retained manifest with
    `wire: None` directly to the FR-331 request adapter and verify it refuses
    rather than reconstructing the descriptor. Verify a supplied `wire` value
    is retained byte-for-byte, and an empty advertised-capability list is
    refused before the registry can emit a descriptor.
14. Submit a harness the driver generated on the general `rt::Integer` route
    (`x + 1` over `Int[0, 9]`) to the provider's production run path with a
    nonexistent launcher and crate. Verify the item's result is
    `KaniProviderFailure::NotTractable` carrying CG's
    `UnsupportedObligation::V2ClauseNotTractable` with the harness's node and
    operation identity, 4 bits, the general route and the shipped measured row,
    and that no launch was attempted (a launch attempt would be an `Execution`
    refusal). Verify the same harness on the native route is not refused by
    that row and reaches CG's execution (here, its `Execution` refusal for the
    missing launcher). FR-005-AC-22.

Test location: `src/registry.rs`, `src/drive.rs`, `src/provider.rs`,
`quire-cli/src/backend_provider.rs`, `quire-cli/src/process_provider.rs`, and
`tests/drive.rs`; full routed
invocation and process-provider terminal integration remain planned.

## Expected Results

- Distinct manifest identities become corresponding registered backends, and
  lookup resolves each uniquely registered linked provider.
- Repeating an identical linked manifest is QSL-idempotent. If multiple
  linked implementations share that identity, lookup does not choose one
  based on input order.
- Kani preflight preserves cancellation and backend mismatch and reports
  missing execution limits and harnesses as typed failures.
- The public `Provider::run` processes each selected item and returns its
  typed missing-limits refusal when execution is not configured.
- Caller-provided `ProofCeilings` appear unchanged in the generated harness
  identity; an execution request with differing ceilings is refused before
  execution. CG's evidence fields remain available through the typed provider
  result. CG FR-028 defines the wall-clock ceiling in the harness
  identity as the execution timeout; the driver supplies no second timeout.
- An explicitly supplied `KaniExecutionLimits.home_directory` reaches CG's
  request unchanged. An unknown home directory remains `None` even when a
  decoy process `HOME` is set. CG's optional diagnostics and failure cause
  remain unchanged in typed evidence; the driver reads neither diagnostic text
  nor ambient environment to choose a failure cause or terminal.
- CG maps cover-unsatisfied to `Proved { success_checks: 0 }` and timeout to
  `Incomplete(TimedOut)`; the driver does not reimplement that mapping.
- A general-route scalar harness is refused with CG's typed measured row
  before any Kani process starts; a native-route harness is not refused by it.
- Kani execution limits and terminal mapping remain Kani-specific. The shared
  result carrier preserves generic non-Kani success and failure data, request
  indexes and backend identities. Plugin `proved` results remain unmapped
  pending QSL-490 and QSL-655; the driver does not fabricate Kani
  `success_checks`. Plugin claims of replay-only causes remain tool failures.
  Cancellation is checked between routed items. The current CG execution API
  accepts no cancellation handle, so an already-running harness continues
  until its identity's `wall_clock` ceiling.
- Process-origin plugin admission remains planned with plugin-host work. The
  test covers projection of the supplied origin into CG's descriptor and CG's
  `BackendKind::from_descriptor` classifies the descriptor from that origin.
  The driver does not construct CG process kinds from provider manifest bytes.
- Linked registry admission retains canonical FR-331 bytes. Present optional
  members survive and absent members remain absent; the FR-331 request adapter
  refuses `wire: None` rather than reconstructing a manifest.
