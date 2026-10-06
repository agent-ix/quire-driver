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
preflight refusals, and CG-owned terminal mapping. Full routed invocation,
typed-result forwarding, and process-provider integration remain planned.

Scope: FR-005-AC-1 through FR-005-AC-5.

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
6. Pass a cover-unsatisfied outcome and a timeout outcome through CG's
   `run_terminal_value`; verify they map to `Proved { success_checks: 0 }`
   and `Incomplete(TimedOut)` respectively.
7. The origin-aware QSL-to-CG descriptor projection is tested in
   `src/drive.rs`: build separate QSL registries with otherwise equal `kani`
   descriptors, one `Linked` and one `Process`, then verify their CG
   projections preserve identity and advertised pairs while mapping each
   origin to its corresponding CG variant. The Linked-origin half of
   FR-005-AC-1 is tested in `src/registry.rs`. Process-origin plugin hello
   registration through the `quire-cli` process-provider adapter remains
   planned. CG IR-629 gates process backend-kind mapping; the origin projection
   itself is covered by the driver test.

Test location: `src/registry.rs`, `src/drive.rs`, `src/provider.rs`, and
`tests/drive.rs`; full routed invocation and process-provider integration
remain planned.

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
- CG maps cover-unsatisfied to `Proved { success_checks: 0 }` and timeout to
  `Incomplete(TimedOut)`; the driver does not reimplement that mapping.
- Provider limits and result carriers are Kani-only for this slice.
  Cancellation is checked between routed items. The current CG execution API
  accepts no cancellation handle, so an already-running harness continues
  until its identity's `wall_clock` ceiling.
- Process-origin plugin admission remains planned with plugin-host work. CG
  origin interpretation is covered by the projection test; the driver does
  not construct CG process kinds from provider manifest bytes.
