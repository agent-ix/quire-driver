---
id: SR-013
title: "Code and Rust review of QSL-1 PR #8: drop RequestedItem, route from CheckedGraph::requirements()"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@5e8003c588dd6b4ef4a210b3e2820d0fddfc5914; Cargo.toml; Cargo.lock; README.md; spec/functional/FR-001-orchestrate-the-driver-steps.md; src/drive.rs; src/lib.rs; src/refusal.rs; tests/drive.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-007, type: references }
---
# Code and Rust review of QSL-1 PR #8: drop RequestedItem, route from CheckedGraph::requirements()

## Summary

Ticket: QSL-1. PR: quire-driver#8, reviewed at 5e8003c.

The PR moves the QSL layer crates from a553b7c to 267be557 (QSL #459,
QSL-266). It removes `RequestedItem` from the public API and replaces
`drive()`'s `items: &[RequestedItem]` parameter with `named:
Option<&BackendId>`. The items are now written by
`qsl_route::request::items_from_requirements` from
`CheckedPackage::graph().requirements()`. The README and FR-001 are updated,
and `tests/drive.rs` is rewritten so that no test supplies an item.

The production change is small and correct. `envelope()` reads each item's
kind, extent and candidate outcome from `RequirementItem`, and stamps `named`
on every request item. `named` is the same value `items_from_requirements`
built the candidate outcomes against, so the two cannot disagree. The public
API change is clean: there is no shim, no deprecated alias and no leftover
`RequestedItem` reference outside prose.

The pin bump is correct. Measured over `Cargo.lock` by (name, source): the
only change is the five direct QSL layer crates and their QSL
dependencies moving from a553b7c to 267be557. CG stays at edade59, IR's
model and root crates stay at 48ab5dc, and the transitive 9395be4, daa0178,
7b1ec7a and b52c263 copies are unchanged. The other lock-diff lines are
reordering.

The problems are in test coverage. The rewrite lost three oracles the old
tests had, and it left the `UnknownNode` refusal as a public caller-input
refusal that no caller input can now produce.

Method: I read the diff, QSL 267be557's `qsl-route/src/request.rs` and
`qsl-semantics/src/check/family.rs`, and compared every changed test with
`git show origin/main:tests/drive.rs`. I then ran a temporary probe test in
the review worktree. It was not committed and was deleted after the run. The
probe drove the dead-branch unit, a `x > 20`-only unit and two
unbounded-parameter units through `drive` with `kani` registered.

## Verdict

**CHANGES REQUESTED (medium).** The code, the pin bump, the README and the
new FR-001 text are sound, and `make ci` is green. Three medium findings
concern lost oracles and a misclassified refusal. None of them is a runtime
bug today.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No test now routes a mixed or gapped settlement set. The old combined test settled supported, unsupported and invalid-request in one run and asserted `Route { supported: 1, dispositions: 1 }`. The old ascending test asserted routed and generated indexes `[0, 2]`. After the split, every routed run is all-supported with contiguous indexes: `[0]` in the negate and IR-302 tests, and `[0, 1]` in the EQ and dead-branch tests. Every other run is none-supported. The EQ test also replaced the positional `items[0].request_index == 0` and `items[1].request_index == 1` checks with `find`, so no test asserts ascending order. Failure scenario: `route_supported` or `generate` is changed to use the routed position instead of `settlement.request_index`, or to pass a non-supported settlement to `route`, and every test stays green. This answers coder deviations 1 and 3: the split lost the AC-3 "only supported" oracle, and ascending order across settled and unrouted items has no oracle. A mixed set can no longer come from real input. I measured every record as `value-validity`, and `Int` without a bound does not parse ("expected `[`"), so every item in a unit gets the same candidate outcome. It can still be built through the test seam. Have `Recorder::negotiate` rewrite the real CG answer for the EQ unit so that item 0 settles `unsupported`, then assert routed `[1]`, `Route { supported: 1, dispositions: 1 }`, `Generate { items: 1 }` and generated indexes `[1]` naming the `-` node. Also restore the positional index assertions in the EQ test. | src/drive.rs:138-151,186-191,322-360; tests/drive.rs:1328-1349 |
| FND-002 | medium | `DriverRefusal::UnknownNode` is now unreachable from caller input, but it is still a public input refusal (`OutcomeKind::Refused`, `dangling_reference`). Every item's node now comes from `compiled.package.graph().requirements()`, and `checked` is IR's admission of the bytes that same package emitted at the same `package_id`. The lookup at drive.rs:192-197 can fail only if E4 emitted, or E5 admitted, a graph without a node QSL's own requirement record names. FR-001's Behavior calls that "an upstream answer that breaks the driver's invariants", which is an internal fault. Failure scenario: an E4/E5 regression drops a node, and the driver reports it to the caller as the caller's own `dangling_reference` refusal instead of a `runtime_invariant` fault. The guard is not dead code. It is the only thing between that regression and a wrong CG call. It should become an internal fault, not be removed, because removing it would need an `expect`/panic. Move it to a `DriverFault` variant (for example `DriverFault::UnknownNode { request_index, node }`: stage Generation, code `runtime_invariant`, kind Fault), and cover the mapping with a unit test in `src/refusal.rs` like `tc_004_a_fault_is_an_internal_fault_with_runtime_invariant`. FR-001:82-84, FR-001-AC-6 and TC-007 step 2 still describe the old caller refusal (see SR-014). This answers coder deviation 2. | src/drive.rs:192-197; src/refusal.rs:38-47,155,175,199; spec/functional/FR-001-orchestrate-the-driver-steps.md:82-84,125 |
| FND-003 | medium | The EQ test dropped its "sibling unaffected" differential, although the PR body says the underivable-sibling test "keep[s] [its] old assertions". The old test compared the `-` item's `record.disposition` and its whole `harness` with a run that routed the `-` alone. The new test checks only `Supported { .. }`, `harness.identity.node_id` and the argument bounds `[(0, 9)]`. FR-001-AC-8 requires that an underivable node "does not affect its routed siblings' records or harnesses". Failure scenario: the `=` item's refusal leaks into its sibling's rendered harness, for example its claim-map entry, `harness.rust` contents or result-bound literals, and the test stays green. The driver can no longer route the `-` alone, but CG can. Call `generate_routed(&checked, &[<the - item alone, request_index 1>], ...)` directly and assert that `record.disposition` and `harness` are equal to the combined run's `-` item. | tests/drive.rs:1360-1381 |
| FND-004 | low | The dead-branch test now routes the `x > 20` guard as request index 1, but it asserts nothing about that item except through the pass-through equality with CG. Measured at 5e8003c: `Unsupported { OperationNotRendered { operation_identity: "quire.op.integer.gt", derived_domains: [IntegerRange { bound: <x's bound node>, lower: "0", upper: "9" }] } }`, with no harness. The same result comes from a unit with only `x > 20`. Failure scenario: CG starts rendering `gt`, or settles it differently, and the test and its doc stay silent. The pass-through check compares against CG itself. This answers coder deviation 4: yes, assert it. The correct expectation is that disposition by full equality, with `harness.is_none()`. The node should be resolved from `gt` rather than hard-coded. | tests/drive.rs:878-893,917,950-968 |
| FND-005 | low | Leftovers in the tests. `expression` and `expression_by_identity` still build and return a `WireNodeId` that every caller now discards (`_add_wire`, `_neg_wire`, `_eq_wire`, `_gt_wire`, 9 sites). Return only the `CheckedNodeId`, and drop the `WireNodeId` import. `INT_UNIT`'s doc still says the unit has "two nodes a Kani item can name", but its `conversion` node is not a requirement record and cannot be an item now. | tests/drive.rs:16,43-44,366-394,492,672,710,756,805,902-903,1311-1312 |

## Rust review

Lane: rust-review over `src/drive.rs`, `src/lib.rs` and `tests/drive.rs`. The
repo idioms are in `CLAUDE.md`, `clippy.toml` and the `[lints]` table
(`unsafe_code = "forbid"`, `clippy::all = "deny"`).

- Public API: `RequestedItem` is removed from `drive.rs` and from the `lib.rs`
  re-export, with no alias and no compat path. `drive`'s `named:
  Option<&BackendId>` is the idiomatic borrowed optional. No new public type
  leaks a QSL type. `RequirementItem` stays private to `drive.rs`.
- `envelope()` no longer rebuilds a `RequestWriter` to classify the extent.
  `RequirementItem::extent()` is `ExtentClassification::of(...)`, the same
  classification `RequestWriter::item` computes, so the wire extent is
  unchanged. `Some(cg_extent(..))` is now unconditional. That is correct: the
  old `Option` came only from the writer's index lookup.
- `named` semantics: `named` is now one value per request, where each item
  used to carry its own. It is passed to `items_from_requirements` (candidate
  outcome) and to `envelope` (`named_backend` on each wire item) from the
  same binding in `drive`, so the two always agree.
  `tc_001_a_named_unregistered_backend_settles_invalid_request` covers the
  unregistered case. No test covers a named backend that is registered, which
  narrows the candidate set (SR-014 FND-004).
- No new `unwrap`/`expect`/panic in `src/`. `.ok_or(DriverRefusal::...)`
  builds cheap values eagerly, which is fine. No `unsafe`, no new `#[allow]`.
  The eager `.ok_or(DriverRefusal::UnknownNode { .. })` is the
  misclassification in FND-002, not an idiom problem.
- Tests: expected values carried over from `origin/main:tests/drive.rs` are
  unchanged where they survive. That covers `bounds == [(0, 9)]`, the
  `missing`/`unknown-backend` cause, the IR-302 `x + 1` and `x + y`
  assertions, the `ResultBoundUnreachable { 0, 10, "100", "109" }`
  disposition, the eq `NoDerivableClaim`/`Underived` claim, and the typed
  `MissingKindContext` refusal. The call-log counts changed only where the
  item count did (`Negotiate { items: 1 }`). No expected value was
  regenerated from the new code. The losses are the oracles removed in
  FND-001 and FND-003.
- The `Cargo.toml` revision comment correctly marks itself informational.

## Gate results

- Reviewer log:
  `/tmp/claude-1000/-home-peter-dev/09ab4eba-c376-42ef-aaec-be2e14ce86e4/scratchpad/rv-drv8/make-ci.log`.
  `make ci` ran at 5e8003c. The first line is
  `5e8003c588dd6b4ef4a210b3e2820d0fddfc5914`, and the last line is `exit=0`.
  fmt-check, clippy `-D warnings`, test, deny, audit-unsafe, docs and spec
  were all green. Test counts: 2 unit, 1 dependency-edges and 17 in
  `tests/drive.rs`, all passing with 0 ignored.
- The probe ran as a temporary test in the review worktree and was deleted
  after the run. It printed the dispositions cited in FND-001 and FND-004.

## Dispositions

Re-checked at debdda46387c04a4d7a8ef6aca80abdec0d7d000 (fix round, one commit
on top of 5e8003c).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | debdda4. `Recorder::negotiate` now rewrites the real CG negotiation answer for `EQ_UNIT`, forcing item 0 (the `=`) `unsupported`, in the new `tc_003_route_and_generate_skip_a_negotiation_unsupported_item_by_request_index`. It asserts `routed`/generated indexes `[1]`, `Route { supported: 1, dispositions: 1 }` and `Generate { items: 1 }`, and its own doc explains the positional check: the routed and generated item sits at list position 0 but carries request index 1. I re-ran this with `route_supported` changed to key by loop position instead of `settlement.request_index` (a local, uncommitted edit, reverted after) and confirmed the test goes red on that regression alone. The underivable-sibling test also restores positional (`items[0]`/`items[1]`) asserts, covering AC-6's ascending-order clause with two genuinely `supported`, routed items. |
| FND-002 | fixed | debdda4. `DriverRefusal::UnknownNode` moved to `DriverFault::UnknownNode` (stage Generation, code `runtime_invariant`, kind Fault), covered by a new case in `src/refusal.rs`'s `tc_004_a_fault_is_an_internal_fault_with_runtime_invariant`. FR-001:82-91 and AC-6 now describe it as an internal fault reachable only through an E4/E5 inconsistency; TC-007 states the same and points at the `refusal.rs` test rather than a caller-input case. |
| FND-003 | fixed | debdda4. The underivable-sibling test now calls `generate_routed` directly over the `-` item alone (`request_index: 1`, the same node/backend/kind) and asserts the whole routed item (`RoutedItem`, disposition and harness together) equals the combined run's `-` item, not just a few destructured fields. |
| FND-004 | fixed | debdda4. The dead-branch test asserts the guard `>` item's full disposition: `Unsupported { OperationNotRendered { operation_identity: "quire.op.integer.gt", derived_domains: [IntegerRange { bound: <x's own [0,9] node>, lower: "0", upper: "9" }] } }`, with `harness.is_none()`. The bound node is resolved structurally (a new `integer_range_bound` helper matching the node's own `min`/`max` literals), not hard-coded. I independently re-measured this against a fresh single-declaration `x > 20` unit (no dead branch, no `+`) via a temporary probe test (not committed): it settles the identical disposition, confirming CG's own rule (`OperationNotRendered`: IR-confirmed operation, no Kani renderer for any family but `IntegerArithmetic`) rather than an artifact of the dead-branch shape. |
| FND-005 | fixed | debdda4. `expression`/`expression_by_identity` now return `CheckedNodeId` alone; every call site (9) dropped its unused `_*_wire` binding, and the `WireNodeId` import is gone. `INT_UNIT`'s doc now says the narrowing `conversion` is excluded from `is_scalar_application` by name and is not itself a requirement record. |

Disposition-pass gate at debdda4: `/tmp/claude-1000/-home-peter-dev/09ab4eba-c376-42ef-aaec-be2e14ce86e4/scratchpad/qsl-1-routes-ci-r1.log`, first line `debdda46387c04a4d7a8ef6aca80abdec0d7d000`, last line `exit=0`. `tests/drive.rs` shows 19 passed (was 17), `src/refusal.rs` shows 2 passed (was 2, one case added inside the existing test), 0 ignored throughout. No new findings.
