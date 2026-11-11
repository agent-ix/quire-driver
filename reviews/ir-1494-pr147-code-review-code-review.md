---
id: SR-14860
title: "IR-1494 PR #147 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver PR #147 at 2d9b9220; IR-1494, IR-1504, IR-1511; Cargo.toml, quire-cli/Cargo.toml, Cargo.lock, src/drive.rs, src/refusal.rs, src/lib.rs, src/provider.rs, src/upstream.rs, tests/drive.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: reviews
---

## Summary

Ticket: IR-1494 (IR-1504 and IR-1511 folded in as duplicates). The PR moves both manifests from CG `tag = "v0.1.0"` to `branch = "main"` (lock: CG a15be8eb -> dab4861f), adapts to CG main (MemoryMechanism dropped with no shim, `proof_phase: None`, `source: Some("fixture")`), and implements the FR-020 projection: `project_requirements` (src/drive.rs) builds each item's `RoutedRequirement` and the package `RequirementRecords` from `value-validity` records only (`get_kind`), `lower_driver_request` adds the conversion root then one guard root per path guard in each item's span, `generate` refuses a Kani-routed item with no requirement with `RequirementRecordAbsent` after negotiation, and a package record that does not project refuses `RequirementRecordUnresolved` inside `lower_driver_request`, before negotiation. CG's `RoutedGenerationError::RequirementRecord` maps to `runtime_invariant`/Generation/Fault; six new Kani oracle errors are mapped by exhaustive arms (`oracle_code`, `owner_path_code` over all 13 `CheckedFunctionErrorCode` values), each with its own message. The rust-review lane is folded into this file.

Checked and clean: no new wildcard arm in `generation_code`, `oracle_code` or `owner_path_code`; the seven mismatch messages are distinct (the Debug of the mismatch); no new `unwrap`/`expect`/`panic!` in non-test `src`; no rev/tag/hash pin in either manifest; trace attributes present on every new test (`TC-034`/FR-020 ACs, `TC-007`/FR-001-AC-24); public API changes are additive (`RequirementRecordUnresolvedCause` export with doc comments, defaulted `Upstream::requirement_records`). The RR16_UNIT edit (`g(x + 1) and h(x + 1)` -> `k(g(x + 1), h(x + 1))`) is not a weakening: RR-16's subject is one node at two occurrences with different conversion ranges, which the new unit keeps; `and` would now guard the right operand and move it to `path_guard_not_lowered`, and guards are covered by the new GUARDED_UNIT tests. The `invalid-request` expectations are honest: that is the catalog's wire spelling, already used by the pre-existing test at src/refusal.rs:1368.

Executed (own worktree, `flock /tmp/cargo-slot.lock`, `QUIRE_CVC5_BIN` set): `cargo test --locked -p quire-driver --test drive` 44/44 pass; `cargo test --locked -p quire-driver --lib refusal::` 32/32 pass. Mutation controls (each reverted, tree clean after): M1 reverse the path condition in both the projection and the guard roots: SURVIVES all 44 drive tests. M2 drop the conversion ContextRoot: 22 drive tests fail, including `guarded_occurrences_carry_their_own_requirement_and_guard_roots`. M3 swap codes (KaniReaderRefused -> invalid-request, KaniUnsupportedBody -> runtime_invariant): `kani_reader_refused_is_a_runtime_invariant_fault` and `kani_unsupported_body_is_an_invalid_request` fail. M4 KaniReaderCatalogUnavailable -> invalid-request: SURVIVES. M5 guard roots before the conversion root: `guarded_occurrences_carry_their_own_requirement_and_guard_roots` fails ("span of request 0"). M6 NotApplicable scalar selection on generation items: 21 fail. No Kani, no workspace run.

## Verdict

**NOT mergeable.** The projection, fault placement and error mapping are correct and the error surface is well pinned, but the guard ordering that FR-020-AC-1/AC-7 require is not tested: every fixture has one guard per item, so M1 (reverse the path condition, a mutant TC-034 lists as caught) survives. FND-001 must be fixed; the rest are low.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Path-condition and guard-root order is untested: every fixture has one guard per item, so reversing the path condition in the projection and the guard roots (M1) survives all 44 drive tests | tests/drive.rs:3427-3523 |
| FND-002 | low | KaniReaderCatalogUnavailable's mapping is unpinned (M4 to invalid-request survives) and the case-table comment claims "its own test inspects the source", but no such test exists | src/refusal.rs:2065-2066 |
| FND-003 | low | direct_generation builds the expected item, then discards it and feeds CG the driver's own passed item and records, so the *_passes_through_unchanged tests no longer check the routed item against an independent oracle | tests/drive.rs:1308-1359 |
| FND-004 | low | Cargo.lock now holds QSL crates from two commits of the same main branch (qsl-analyze, qsl-command, qsl-route at 48a12edb; the other seven at 91ada09a, two commits later); origin/main had one QSL commit | Cargo.lock |
| FND-005 | low | project_requirements runs before the per-item loop, so a record at an item's own occurrence that fails lookup raises RequirementRecordUnresolved (no request index) before FR-001's request-indexed UnknownNode/AmbiguousNode/UnknownOccurrenceRole; FR-020 says the item's occurrence resolves first | src/drive.rs:1001 |
| FND-006 | low | lookup_checked_node duplicates resolve_checked_node (same structural_key_matches walk, different error type), and the resolved-graph iterator expression is now repeated four times | src/drive.rs:802-820 |

## Scope

| Unit | Role | Excerpt |
| --- | --- | --- |
| project_requirements | examined | Projects the QSL requirement of every export-bound value-validity item that has a record, and the package's whole set of value-validity records, from the one map. |
| lower_driver_request guard roots | examined | One guard root per path guard, outermost first, inside this item's own context span; a guard shared by two items is rooted in each. |
| generate RequirementRecordAbsent | examined | A Kani-routed item whose projection is None refuses with RequirementRecordAbsent naming request index and occurrence. |
| refusal.rs generation_code / oracle_code / owner_path_code | examined | RequirementRecord -> RuntimeInvariant; KaniReader*/KaniOwnerPathRefused/KaniUnsupportedBody/KaniNestedCallBody mapped by exhaustive arms. |
| DriverFault::RequirementRecordAbsent / RequirementRecordUnresolved, RequirementRecordUnresolvedCause | examined | Two new faults, stage Generation, runtime_invariant, Fault. |
| Upstream::requirement_records | examined | Defaulted trait method returning package.graph().requirements().clone(). |
| Cargo.toml, quire-cli/Cargo.toml, Cargo.lock | examined | quire-contract-codegen branch = "main" in both manifests. |
| src/provider.rs, tests/drive.rs MemoryObservation fixtures | examined | source: Some("fixture"), proof_phase: None. |
| tests/drive.rs RR16_UNIT | examined | function f using v(x: Int[0, 9]): Boolean pure { k(g(x + 1), h(x + 1)) } |
| tests/drive.rs direct_generation | examined | Returns the driver's passed item for each expected request index. |

## Dispositions

Round 1, head 856d2e3a (one fix commit on 2d9b9220; based on main d6b6b31, `git merge-tree` against origin/main clean). Checked against `git diff 2d9b9220..856d2e3a` and the code. Focused job in own worktree: drive 46/46, refusal lib 32/32. M1 (reverse the path condition in projection and roots) now fails `two_nested_guards_keep_outermost_first_order_in_the_requirement_and_roots` (the run refuses at `drive_unit(...).expect`, tests/drive.rs:3535); M7 (look the item's record up with `get` instead of `get_kind`) fails `two_capabilities_at_one_occurrence_project_only_the_value_validity_record` (tests/drive.rs:3864).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | NESTED_GUARDED_UNIT (`if p then (if q then true else g(x + 1)) else true`) and `two_nested_guards_keep_outermost_first_order_in_the_requirement_and_roots` assert holds `[true, false]`, distinct outer and inner guard nodes, the item and record set against QSL-derived expectations, and the span `[conversion, outer, inner]`; M1 now fails it. |
| FND-002 | fixed | The case-table comment now says CatalogReadError has `pub(super)` fields and no public constructor and that the RuntimeInvariant mapping is not pinned by any test; tests.md records the mapping as Planned. No false test claim remains. |
| FND-003 | fixed | `direct_generation` takes `driven`, builds each expected item with `with_scalar_selection` and attaches `expected_requirement` from QSL's own `get_kind(ValueValidity)` record, and builds `requirement_records` with `expected_record_set` from `driven`; nothing is read from `generation_routed` or `generation_records`, so the oracle is independent of what the driver passed. |
| FND-004 | fixed | Cargo.lock: all ten QSL crates at 91ada09a; the lock diff is the three source lines of qsl-analyze, qsl-command and qsl-route. |
| FND-005 | fixed | `lower_driver_request` first resolves every item's application, occurrence and conversion (request-indexed faults) into `frames`, then calls `project_requirements`, which projects items before package records. This matches FR-020's "FR-001 resolves an item's own occurrence first". No test pins the order; accepted, because an item whose own occurrence fails lookup cannot be built from a compiled package, so the competing fault is unreachable from a test. |
| FND-006 | fixed | `checked_node_keys` (one iterator) and `select_node` (one walk) now back both `lookup_checked_node` and `resolve_checked_node`; the four repeated iterator expressions are gone. |
