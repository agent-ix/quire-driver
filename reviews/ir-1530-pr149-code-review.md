---
id: SR-17940486709564847047958
title: "IR-1530 PR #149 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver PR #149 at 9aca0720, git diff origin/main...HEAD; IR-1530; src/provider.rs, src/refusal.rs, Cargo.lock (plus src/drive.rs, src/upstream.rs, src/registry.rs, quire-cli/src read as context for Kani launch sites)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-005
    type: reviews
---

## Summary

Ticket: IR-1530. Code-review with the rust-review lane folded in. The PR makes
`KaniProvider::run_items` (the body of `Provider::run`) load
`TractabilityTable::shipped_general_route_scalar()` and pass it to CG's
`execute_kani_obligation_decided`; it adds `KaniProviderFailure::{NotTractable,
TractabilityUnavailable}`, maps CG's new `KaniObligationError::TractabilityTable`
in `src/refusal.rs`, bumps Cargo.lock to CG b4e2a6bd (#821 merge) and two tests.

Launch-site sweep of the workspace (`src`, `quire-cli/src`): Kani is launched
from exactly two driver call sites, `src/provider.rs` (`execute_kani_obligation_decided`,
now gated) and `src/drive.rs:539` (`cg::execute_kani_scalar_native`, not gated).
`src/upstream.rs:191-228` is QSL parse/check/package and CG `generate_routed`
generation; it launches nothing (CG keeps emitting the harness by design and
refuses only at run time), so it is not a bypass. No other `execute_kani*` call
exists; quire-cli launches no Kani and builds `build_linked_providers(&[])`, so
neither driver call site has a non-test caller in this workspace: both are
public library API of equal standing.

Checked clean: the refusal is mapped arm-for-arm without reclassification;
`refusal.rs` maps the table-load fault to `RuntimeInvariant` with an
internal-fault message; tests carry `#[trace("TC-012", "FR-005-AC-22")]`.
Cargo.lock: every bumped source is `branch=main` (no rev pins, Cargo.toml
untouched); qsl/ir/model/exact moved to current main as a side effect of the CG
update and CG b4e2a6bd adds `libc`; acceptable, not churn. `cargo test --lib
provider::` passes 17/17 at 9aca0720.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | Ungated Kani launch: `execute_scalar_items` calls `cg::execute_kani_scalar_native` (which runs `execute_kani_obligation_library` on `KaniExecutableHarness::Scalar`) on a harness from `generate_routed`, i.e. always the general `rt::Integer` route, with no tractability decision. `drive` and `ScalarNativeInput` are public (`src/lib.rs:70`). The repo's own kani-lane test `successful_native_receipt_is_authenticated_against_retained_limits` launches exactly the measured not-tractable form (general route, `quire.op.integer.add`, `Int[4, 4]`, <=4 bits). IR-1530's planner AC (refusal where Kani is launched) is therefore not met. Fix is driver-only: before `execute_kani_scalar_native`, call CG's exported `refuse_if_not_tractable(&KaniExecutableHarness::Scalar(harness), &table)` with the shipped table, return a new typed `DriverRefusal` arm carrying CG's refusal unchanged, add an FR-009 AC + test, and re-point or re-expect the drive.rs:1231 kani-lane test (it will now be refused). A CG `execute_kani_scalar_native_decided` would be the alternative, but is not needed. | src/drive.rs:534-548, src/drive.rs:1231-1298, src/lib.rs:70, CG src/kani/run/scalar.rs:435-470, CG src/lib.rs:210 |
| FND-002 | medium | `native_route_scalar_harness_still_reaches_execution` passes for a reason other than its claim. It mutates `harness.identity.integer_representation` after generation; CG's `require_in_crate` checks `has_valid_provenance()` (`identity == generated_identity`) first, so the `Execution(_)` it matches is `HarnessProvenanceMismatch` (a tampered harness), not a launch-path refusal "for the missing launcher". It proves only that the decider returned `Run` for a Native identity. Either assert `Execution(HarnessProvenanceMismatch { .. })` and say so, or build a genuine native harness through CG's exported `generate_routed_native` and assert the refusal that follows (`HarnessNotInCrate` with the nonexistent crate). | src/provider.rs:977-1003, CG src/kani/run/execute.rs:646-654, CG src/kani/identity.rs:811-815 |
| FND-003 | low | When the embedded table fails to load, `run_items` returns `TractabilityUnavailable` for every item before the per-item preflight, so a cancelled request, a backend mismatch, missing limits or a harness-less item report a table fault instead of their own typed failure; the table is also re-parsed on every call. A per-item typed failure rather than a panic is the right shape; load lazily at the point of execution (or once) so preflight failures keep precedence. | src/provider.rs:303-318, src/provider.rs:440-463 |
| FND-004 | low | A `NotTractable` item has no terminal: `terminal_record()` errs and `ProviderResults::outcome_bytes()` fails the whole batch, so one refused item hides every other item's outcome. This matches existing `Execution` refusals and FR-005-AC-22's no-reclassification rule, and CG ships no terminal map for `DecidedRunRefusal`; record it as a known gap rather than mapping it in the driver. | src/provider.rs:84-90, src/provider.rs:241-256 |

## Verdict

Not mergeable as the IR-1530 closer at 9aca0720. The provider-path wiring is
correct and minimal and the general-route test is real (driver-generated
harness, typed row equality, distinguishes the pre-launch refusal from any
`Execution` refusal). FND-001 leaves a second public Kani launch site ungated,
which is what IR-1530's closing condition is about; it is fixable in this PR
with driver code only. FND-002 should be fixed in the same round. FND-003 and
FND-004 are optional. A test through `Provider::run` instead of `run_items` is
not required: `run` is a one-line delegation that ignores `package`.
