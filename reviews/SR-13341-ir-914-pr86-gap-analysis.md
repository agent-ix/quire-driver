---
id: SR-13341
title: "IR-914 quire-driver PR 86 gap analysis: bound-only CG request"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@7c43963fc9bc844071ae4285a6f1cf9a07acca34; FR-010-AC-1..AC-5, FR-001-AC-15..AC-21, FR-009-AC-16, TC-017 vs spec/tests.md and quoin matrix; src/drive.rs, src/refusal.rs, src/upstream.rs, tests/drive.rs"
review_set: subset
---

## Summary

Ticket: IR-914. PR agent-ix/quire-driver#86. This is a planless gap analysis; plan completion was
not assessed. The reviewer ran `quoin matrix --json` (quoin 0.28.3) at the head.

| Criterion | Status | Tagging test |
| --- | --- | --- |
| FR-010-AC-4 | tagged | `export_partition_withholds_b_and_preserves_a_and_c_indices` |
| FR-001-AC-17 | tagged | `export_partition_withholds_b_and_preserves_a_and_c_indices` |
| FR-009-AC-16 | tagged | `export_partition_withholds_b_and_preserves_a_and_c_indices` |
| FR-010-AC-5 | tagged | `all_export_refusals_send_nothing_to_cg` |
| FR-001-AC-18 | tagged | `all_export_refusals_send_nothing_to_cg` |
| FR-001-AC-19 | tagged | `export_run_refusal_and_unknown_driver_key_keep_their_distinct_outcomes` |
| FR-001-AC-20 | tagged | `export_run_refusal_and_unknown_driver_key_keep_their_distinct_outcomes` |
| FR-001-AC-21 | tagged | `relationless_package_skips_export_and_has_no_item_refusals` |
| FR-001-AC-16 | tagged | `refusal::tests::excluded_generation_index_is_an_internal_join_fault` (one arm only) |
| FR-010-AC-1 | untagged | none |
| FR-010-AC-2 | untagged | none |
| FR-010-AC-3 | untagged | none |
| FR-001-AC-15 | untagged | none |

Every tag is a `#[trace(...)]` attribute, and every tagged test passes in the reviewer's full
gate run (see SR-13340).

The production code added in the diff has owning requirements:

- The export partition and refusal arms are owned by FR-010 and FR-001.
- The `FairnessKindsMember` / `TypeInvariant` arms and the extra `None` descriptor argument adapt
  to the bumped QSL API. They change no driver behaviour.

The main-path rules are pinned by killed mutants (SR-13340). The findings below are new branches
that no test can fail, criterion clauses the tagged tests do not assert, and a stale trace table.

## Examined criteria

- FR-010-AC-4 (examined): "the CG request carries A and C and not B, and B holds an excluded position that keeps its driver index". The envelope count of 2, settlements `[0, 2]` (outer and nested), and B's refusal at index 1 are asserted. The excluded position itself is not asserted (FND-003).
- FR-010-AC-5 (examined): "When the export refuses every requested item, the driver sends CG a request with zero items." The test asserts the opposite, that no CG call is made (SR-13340 FND-001).
- FR-001-AC-15 (examined): position vector with an excluded position keeping `node_id` and driver index. Untagged and unasserted (FND-003).
- FR-001-AC-17, FR-001-AC-18, FR-001-AC-21 (examined): asserted. Clean apart from the FR-001-AC-18 interplay with SR-13340 FND-001.
- FR-001-AC-19 (examined): "refuses the run with the export's typed refusal, and the refusal carries the bindings-export step". The stage and kind are asserted; the refusal payload is not (FND-001, mutant M9).
- FR-001-AC-20 (examined): `runtime_invariant`, Fault and Handoff are asserted. Clean.
- FR-009-AC-16 (examined): terminal records for A and C and none for B. Not asserted (FND-002).
- FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 (examined): exercised by the partition test but untagged (FND-003).
- spec/tests.md rows for FR-010, FR-001-AC-17..21, FR-009-AC-16 and TC-017 (examined): stale (FND-004).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Six new refusal branches have no test that can fail; reviewer mutants M6 to M11 all survive the full `quire-driver` suite. (M6) A per-item `ExportRefusal::Fault` should refuse the whole run as `BindingsExport`; deleting that arm, so the fault becomes an item refusal, survives. (M7) The `ExportCount` length check, when removed, survives. (M8) A bound result whose `item` differs from the driver key, when accepted as bound, survives. (M9) The FR-001-AC-19 whole-export payload: replacing the export's refusal with a different `ExportRefusal` survives, because the test asserts only stage and kind, not "the export's typed refusal". (M10) The `PackageNegotiation` code, when changed, survives. (M11) The `ContractPackageReadIncomplete` kind, changed from `Limit`, survives. The `Unbound`-with-a-different-key arm and the `EnvelopeItemCount` → `SettlementCount` mapping are also untested. Fix: add a fixture case per arm to `tests/drive.rs` or to the refusal unit tests, and assert `refusal` equal to the fixture's exact `ExportRefusal` in the AC-19 test. | src/drive.rs:157-208, src/refusal.rs:210-237, src/refusal.rs:323-327, src/refusal.rs:405, tests/drive.rs:1022-1050 |
| FND-002 | medium | FR-009-AC-16 ("the outcome holds one terminal record for each of A and C and none for B") is tagged on the partition test, but that test asserts no terminal record at all. Its only generation assertion is `.all(\|item\| item.request_index != 1)`, which also passes when generation is empty. A reviewer probe showed the generation is `[0, 2]` today, so the behaviour holds, but no assertion pins it. TC-017 asks for a fixture provider that returns a terminal result per CG-request item, and for checking the records for A and C. Fix: assert the generation request indexes equal `[0, 2]`, and build terminal records (or the FR-331 process request through `RequestStage::from_driven`) for the fixture and assert indexes `[0, 2]` with none for 1. | tests/drive.rs:937-991 |
| FND-003 | medium | FR-001-AC-15, a criterion this PR's IR-914 scope changes, is untagged. Its excluded-position clause is never asserted directly: an `Excluded { node_id }` at driver index 1 carrying B's exact node, absent from the envelope. Mutants that break it are caught only indirectly, through CG's correspondence refusal. FR-010-AC-1, AC-2 and AC-3 are also untagged, although the partition test exercises exactly-the-subset negotiation, bound-only generation and the unchanged `Unbound` refusal. Fix: in the partition test, assert the lowered positions (`DriverItem`, `Excluded { node_id == B's node }`, `DriverItem`) through the recorder, and tag FR-001-AC-15 and FR-010-AC-1..3 where they are actually asserted. | tests/drive.rs:937-991, src/drive.rs:376-383 |
| FND-004 | low | The spec/tests.md trace table is stale for this PR. The FR-010 row, the FR-001-AC-17..AC-21 row, the FR-009-AC-16 row and the TC-017 row (and line 30) still read "Planned in quire-integration; QSL AR2 prerequisite", but driver-level tagged tests for these criteria now exist in `tests/drive.rs`. Fix: update the status cells to name the driver tests and keep quire-integration#16 as the cross-repo composition. | spec/tests.md:30, spec/tests.md:47, spec/tests.md:87, spec/tests.md:88, spec/tests.md:134 |

## Verdict

**Gaps remain in branch and clause evidence; the main partition paths are covered.**

- FND-001 leaves six new refusal arms unpinned. That includes the AC-19 payload clause.
- FND-002 and FND-003 are clause-level gaps on criteria the PR tags or changes.
- FND-004 is bookkeeping.

All four are fix-in-PR items.

## Dispositions

Review pass only; no disposition round has run yet. Disposition rows are appended here.
