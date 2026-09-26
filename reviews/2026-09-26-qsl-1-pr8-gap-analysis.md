---
id: SR-014
title: "Gap analysis of QSL-1 PR #8 against FR-001 and its test cases"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@5e8003c588dd6b4ef4a210b3e2820d0fddfc5914; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-001..TC-008; tests/drive.rs; tests/dependency_edges.rs; src/drive.rs; src/refusal.rs; README.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-001, type: references }
  - { target: ix://agent-ix/quire-driver/TC-003, type: references }
  - { target: ix://agent-ix/quire-driver/TC-006, type: references }
  - { target: ix://agent-ix/quire-driver/TC-007, type: references }
---
# Gap analysis of QSL-1 PR #8 against FR-001 and its test cases

## Summary

Ticket: QSL-1. PR: quire-driver#8, reviewed at 5e8003c.

The done-when for QSL-266 and QSL-1 is met in the code. The driver drops
`RequestedItem`, and every driver test routes the items from the unit's own
requirement records, with no caller input. Where old expected values
survive, they are unchanged.

The PR updated FR-001's Inputs and its "requirement records" paragraph, and
the README. It did not update the test cases. TC-001, TC-003, TC-006 and
TC-007 still describe caller-supplied items. They name two tests that no
longer exist and give expected results the tests no longer check. Three
FR-001 acceptance-criterion clauses lost their oracle in the rewrite.

## Verdict

**CHANGES REQUESTED (medium).** Every AC still has at least one backing
test, and there are no stubs. Several AC clauses are no longer backed, and
four TCs are stale.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-001, TC-003, TC-006 and TC-007 are stale. TC-001 step 2 and its expected results say "three items" and "one negotiation of three items", but the test negotiates one item. TC-003 step 1 expects supported, unsupported and invalid-request from three caller items, and step 2 drives "one `operation-contract` item alone". Neither is what the tests do now. TC-006 steps 1, 2, 4, 5, 7, 9 and 10 say "with no items", "the TC-003 items", "three items ... the `conversion` node" and "routing its `+` node". Step 10 omits the guard `>` item, which is now routed. TC-007 step 2 expects `UnknownNode`, which the test no longer drives. Named tests that no longer exist: `tc_001_tc_003_tc_006_steps_run_in_order_through_generation` (TC-001:25, TC-003:25, TC-006:47) and `tc_006_generation_output_ascends_by_request_index` (TC-006:48). No TC lists `tc_001_a_named_unregistered_backend_settles_invalid_request`, which tags TC-004, or `tc_003_a_unit_with_no_scalar_applications_routes_nothing`. `quire validate` and `quire coverage` do not catch this: coverage reports 0 unbacked rows. Failure scenario: a reader of TC-003 believes the mixed supported/unsupported/invalid-request run is still tested, but no test covers it (SR-013 FND-001). | spec/test-cases/TC-001-steps-run-in-order.md:18-33; spec/test-cases/TC-003-route-after-negotiation.md:17-33; spec/test-cases/TC-006-generation-through-cg.md:21-56; spec/test-cases/TC-007-generation-refusals.md:17-33 |
| FND-002 | medium | The unknown-node clauses have no test. FR-001-AC-6 ("an unknown node refuses the run") and FR-001's Behavior ("A routed item whose node the package does not hold refuses the run with `dangling_reference` before CG is called") lost their only test when the dangling-node case was dropped. No caller input can reach them now. Failure scenario: the spec promises a caller refusal that cannot happen, while the reachable case, an E4/E5 inconsistency, is misclassified and untested. Restate the clause as an internal fault and test the fault mapping (SR-013 FND-002). | spec/functional/FR-001-orchestrate-the-driver-steps.md:82-84,125; spec/test-cases/TC-007-generation-refusals.md:28-33 |
| FND-003 | medium | Two clauses lost their oracle. FR-001-AC-3 "only for `supported` items" and FR-001-AC-6 "its output is keyed by request index, ascending" have no test with a mixed or gapped settlement set. Every routed run is all-supported and contiguous, and the EQ test no longer asserts positions. FR-001-AC-8's "does not affect its routed siblings' records or harnesses" lost its differential. Failure scenario: a position-for-request-index regression in routing or generation, or a sibling leak into a harness, passes every test. The code-side detail and a constructible fix through the `Recorder` seam and a direct `generate_routed` call are in SR-013 FND-001 and FND-003. | spec/functional/FR-001-orchestrate-the-driver-steps.md:122,125-127; tests/drive.rs:1328-1381 |
| FND-004 | low | The new `named` input has one negative test (unregistered, which settles `invalid-request`) and no positive one. A request that names a registered backend, which narrows every item's candidate set to that backend and puts it on each wire item's `named_backend`, is not driven. Failure scenario: `named` is dropped from the envelope's `named_backend`, or from the `items_from_requirements` call, while it is set on the other, and every test stays green, because every other test passes `None`. | src/drive.rs:129,133,297; tests/drive.rs:620-660 |
| FND-005 | low | `tc_003_a_unit_with_no_scalar_applications_routes_nothing` is traced only to QSL's `FR-075`, and `quire coverage` reports it as this repo's one untracked symbol (0 at PR #7). It verifies FR-001-AC-3 (no `route` call) and AC-6 (no generation call). Tag it with those, and list it in TC-003. | tests/drive.rs:1041-1072 |

## Coverage

- Reconciliation at 5e8003c: `quire coverage --scope . --json` found 0
  unbacked rows, 0 status lies and 1 untracked symbol (FND-005). Totals: 8
  of 8 criteria backed.
- AC to TC to test:
  - AC-1: TC-005, `tc_005_driver_edges_match_the_adr_011_allow_list`.
    Unchanged, passes at the new pin.
  - AC-2: TC-002, which now drives `UNIT` with no items. The MD-2 conversion
    still runs over every registered backend.
  - AC-3: TC-003. `tc_003_no_supported_item_means_no_route_call` and the
    step-order test are backed. The "only supported among mixed" clause is
    unbacked (FND-003).
  - AC-4: TC-001, `tc_001_tc_006_steps_run_in_order_through_generation`. The
    TC text is stale (FND-001).
  - AC-5: TC-004, TC-007 and TC-008.
  - AC-6: TC-006 and TC-007. The pass-through and IR-302 tests are backed.
    The ascending and unknown-node clauses are unbacked (FND-002, FND-003).
  - AC-7: TC-004 and TC-006.
  - AC-8: TC-006. Claim-map and no-derivable-claim are backed. The sibling
    differential is lost (FND-003).
- Stubs: none. `src/` has no `todo!`/`unimplemented!`.
- Code without an owning requirement: none new. `items_from_requirements`
  is QSL FR-075, and FR-001's Inputs now cite it.

## Dispositions

Re-checked by the reviewer at fc265183ce5962a5697a2a5364f127a190cf1cda
(fix commit debdda4). The coder's Dispositions section in the committed
copy of this file was written by the coder. It is superseded by this
section.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed debdda4 | TC-001, TC-003, TC-004, TC-006 and TC-007 now describe routing from `requirements()`. They name only test functions that exist; every name was checked against tests/drive.rs and src/refusal.rs. TC-003 steps 1-5 map one-to-one to tests, and TC-006 step 10 covers the guard `>` item. |
| FND-002 | fixed debdda4 | FR-001's Behavior, AC-6 and the fault bullet, TC-004 step 5 and TC-007 now state the unknown-node case as an internal fault. `src/refusal.rs` covers the mapping. |
| FND-003 | fixed (partly) debdda4 | AC-3 "only supported" is backed by the mixed test, and mutant MA is caught. AC-8's sibling differential is restored, by full equality with a direct `generate_routed` call. AC-6 "ascending" is backed by positional asserts in the EQ test. AC-6's "each with ... its admitted `CheckedNodeId`" has no oracle when positions and request indexes diverge: mutant MB survives. See SR-013 FND-006. |
| FND-004 | fixed (partly), rest open as FND-006 | The envelope use of `named` is now backed by `tc_004_a_named_registered_backend_that_does_not_advertise_the_kind_settles_unsupported`, and mutant M2 is caught. The coder's deviation is accepted on its premise and rejected on its conclusion. The premise holds: CG edade59 `BackendKind::ALL = [Self::Kani]`. The claim that the unregistered test covers the `items_from_requirements` use is false: mutant M1 survives. A `Recorder` envelope capture does detect it. See FND-006 below and SR-013 FND-007. |
| FND-005 | fixed debdda4 | The no-scalar test is tagged `FR-001-AC-3 and FR-001-AC-6 (TC-003)` and is listed as TC-003 step 5. |

### New findings (disposition pass)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | The `items_from_requirements` use of `named` (the candidate-set narrowing) has no test. Mutant M1 (`None` in place of `named`) leaves all 19 drive tests green, because CG re-derives the same `unknown-backend` cause from the envelope's `named_backend`. Failure scenario: the driver sends CG a candidate set that ignores the named backend, and nothing notices until CG gains a second backend kind. Fix: capture the envelope in `Recorder::negotiate` and assert `items[0].candidates == Candidates::UnknownBackend("missing")` for `named = Some("missing")`. Also correct the test doc comment's false claim (SR-013 FND-007). | src/drive.rs:129; tests/drive.rs:705-735 |

### Round 2

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-006 | fixed b113d4c | `Recorder::negotiate` now captures the negotiation envelope. `tc_001_a_named_unregistered_backend_settles_invalid_request` drives `named = Some("missing")` and asserts `envelope.items[0].candidates == Candidates::UnknownBackend("missing")`, which only `items_from_requirements` threading `named` through `Registry::candidates` can produce. Mutant M1 (`None` in place of `named` there) now fails this test with `Candidates::Set([kani])` instead. |
| FND-003 | fixed b113d4c (fully) | AC-6's "admitted `CheckedNodeId`" half closed by SR-013 FND-006's fix (the mixed test now asserts the generated node's identity via a direct `generate_routed` comparison). |
| FND-004 | fixed b113d4c (fully) | The candidate-narrowing half of `named` closed by FND-006's fix above; the envelope half was already backed at debdda4 (mutant M2 caught). |

Re-run at b113d4c26b3f3707d7e99df783888d218928b221 with
`scratchpad/rv-drv8/mutants.sh` against a detached worktree at that commit;
each mutant applied, tested with `cargo test -q --test drive`, restored
after (confirmed clean via `git status --porcelain`). All four now fail
(killed): MA and MB by
`tc_003_route_and_generate_skip_a_negotiation_unsupported_item_by_request_index`,
M1 by `tc_001_a_named_unregistered_backend_settles_invalid_request`, M2 by
`tc_004_a_named_registered_backend_that_does_not_advertise_the_kind_settles_unsupported`.

Disposition-pass gate: `make ci` at b113d4c, log
`/tmp/claude-1000/-home-peter-dev/09ab4eba-c376-42ef-aaec-be2e14ce86e4/scratchpad/qsl-1-routes-ci-r2.log`,
first line `b113d4c26b3f3707d7e99df783888d218928b221`, last line `exit=0`.
`tests/drive.rs` shows 19 passed (unchanged). No new findings.
