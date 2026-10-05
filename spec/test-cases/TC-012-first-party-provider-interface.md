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
5. Pass a cover-unsatisfied outcome and a timeout outcome through CG's
   `run_terminal_value`; verify they map to `Proved { success_checks: 0 }`
   and `Incomplete(TimedOut)` respectively.
6. The Linked-origin half of FR-005-AC-1 is tested in `src/registry.rs`:
   verify the compile-time provider's QSL descriptor has origin `Linked`.
   Process-origin plugin hello registration and FR-005-AC-5 remain planned
   with plugin-host work and CG IR-633; when that integration lands, verify
   the origin-aware CG projection preserves the registry descriptor's origin.

Test location: `src/registry.rs`, `src/provider.rs`, and `tests/drive.rs`; full
routed invocation and process-provider integration remain planned.

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
- CG maps cover-unsatisfied to `Proved { success_checks: 0 }` and timeout to
  `Incomplete(TimedOut)`; the driver does not reimplement that mapping.
- Provider limits and result carriers are Kani-only for this slice.
  Cancellation is checked between routed items. The current CG execution API
  accepts no cancellation handle, so an already-running harness continues
  until its configured timeout.
- Process-origin admission and CG origin interpretation remain planned with
  plugin-host work and CG IR-633. The driver does not construct CG process
  kinds from provider manifest bytes.
