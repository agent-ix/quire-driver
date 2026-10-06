---
id: TM-001
title: "Driver command and execution test matrix"
type: TestMatrix
---
# TM-001: Driver binary, provider and execution test matrix

## Overview

This authored verification inventory integrates FR-001 and SP-D1 through
SP-D9 (FR-002 through FR-010) from IR-527, FR-011 from IR-616 and FR-012 for
the downstream cache component, and FR-014 for the CLI package boundary. It
covers FR-001 through FR-012 and FR-014, and TC-001 through TC-022 and TC-028.
Discrete FR
and TC artifacts remain authoritative;
this matrix is their index, not a computed implementation-coverage report.

`✅` means the existing source contains the test implementations named by
TC-001 through TC-008; it does not claim a fresh test run or compatibility
with current upstream APIs. `🚧` means verification is planned. This spec-only
change runs artifact validation, not source tests or the G5b gate.

## Verification Ownership

| Scope | Verification owner and location |
|---|---|
| FR-001 / TC-001..TC-008 | Driver orchestration tests in `tests/drive.rs`, `tests/dependency_edges.rs` and `src/refusal.rs`, as named by the TC artifacts; upstream API adaptation is IR-606. |
| FR-001-AC-9..10, FR-011 / TC-019 | Driver library cancellation matrix and source-provision lifetime tests; API/lifetime review. |
| FR-002..FR-009 / TC-009..TC-016, TC-018 | QSL-backed execute and available proof-terminal serializers have unit coverage; CLI process and full prove/replay composition tests remain planned. |
| Plugin wire, cache policy, execution semantics and AOT/interpreter parity | QSL ADR-029 owner contracts and suites. Driver TC-013, TC-014 and TC-015 check composition boundaries; quire-cache behavior is verified by TC-020..TC-022 under FR-012. |
| FR-010 / TC-017 | Planned QSL-owned cross-repository test in `quire-integration`, QSL-514; QSL AR2 is the prerequisite. |
| G5b acceptance evidence / IR-528 | `quire-integration` only: CLI/library parity across the eight outcome kinds with identical outcome bytes and exit codes; provider/plugin installation; cache hit/miss/never-store; AOT/interpreter parity; absence of a QSL user binary; and prove execution, replay and FR-331 records. |

The driver supplies the one user-facing CLI. CG FR-005's CLI-facing
conformance is exercised through that binary; CG keeps its local generator
and atomic publication tests. CG's cross-backend parity and IR-337's
outcome-per-backend and bound-suite coverage feed the G5b evidence rather
than a second driver or CG gate. The driver and QSL suites supply local
verification; they do not substitute for IR-528 acceptance evidence.

## Requirements Traceability

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1..FR-001-AC-11 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007, TC-008, TC-019 | ✅ Existing test sources; TC-005 gains the transitive cache-boundary assertion |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-009, TC-018 | 🚧 QSL-backed execute result serialization is unit-tested in `src/execute.rs`; machine CLI dispatch/writing and process exit integration remain planned. |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3 | TC-010 | 🚧 Planned |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-011 | 🚧 Planned |
| FR-005 | FR-005-AC-1..FR-005-AC-5 | TC-012 | 🚧 The Linked Kani Provider slice covers QSL manifest admission/lookup, including the Linked-origin half of FR-005-AC-1 in `src/registry.rs`, Kani preflight refusals, explicit `ProofCeilings` propagation and mismatch refusal, typed CG evidence fields, and CG terminal mapping. The QSL-to-CG origin projection is tested in `src/drive.rs`. Provider limits/results are Kani-only here; cancellation is checked between items, while an active CG harness runs to its identity's wall-clock ceiling. Full routed invocation and actual execution-result forwarding remain planned. Process-origin plugin registration and process backend-kind mapping remain planned under plugin-host work. |
| FR-006 | FR-006-AC-1, FR-006-AC-2 | TC-013 | 🚧 Planned |
| FR-007 | FR-007-AC-1..FR-007-AC-6 | TC-014 | 🚧 AC-1, AC-2 and AC-6 remain planned until provider dispatch is integrated; the helper tests cover cache-call semantics AC-3..AC-5 |
| FR-008 | FR-008-AC-1, FR-008-AC-2 | TC-015 | 🚧 Planned |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | TC-016 | 🚧 Execute and available proved-provider terminals serialize through QSL `OutcomeDocument`; falsified scalar results without replay remain typed errors. Full replay mapping and per-item accounting remain planned. |
| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 | TC-017 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
| FR-011 | FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-011-AC-4, FR-011-AC-5 | TC-019 | ✅ Cancellation matrix and API mapping tests present; CLI adapter remains out of scope |
| FR-012 | FR-012-AC-1..FR-012-AC-5 | TC-020 | ✅ Key members, target shape, excluded metadata projection and meaning-input changes are tested with `Trace:` tags |
| FR-012 | FR-012-AC-6..FR-012-AC-8 | TC-021 | ✅ Matrix binds AC-6, AC-7, and AC-8 to the typed hit/miss, excluded-value, and deterministic incomplete-value test at `quire-cache/src/lib.rs:942` |
| FR-012 | FR-012-AC-9..FR-012-AC-12 | TC-022 | ✅ Entry bounds, temp cleanup, rename errors, ownership-filtered eviction, concurrent writes and eviction-callsite NotFound are tested with `Trace:` tags |
| FR-012 | FR-012-AC-13 | TC-005 | ✅ `tests/dependency_edges.rs::tc_005_driver_edges_match_the_adr_011_allow_list` carries a `Trace:` tag |
| FR-014 | FR-014-AC-1 | TC-028 | 🚧 Sole `quire-cli` ownership of the Cargo `quire` binary target; package-graph test planned |
| FR-014 | FR-014-AC-2 | TC-028 | 🚧 Direct `quire-driver` and `quire-cache` dependencies; package-graph test planned |
| FR-014 | FR-014-AC-3 | TC-005 | 🚧 `quire-cli` workspace default-member status; metadata assertion planned |
| FR-014 | FR-014-AC-4 | TC-009, TC-018, TC-028 | 🚧 CLI contract and process-boundary tests and fixtures owned by `quire-cli`; test-location assertions planned |
| FR-014 | FR-014-AC-5 | TC-028 | 🚧 Parser and binary entry point locations in `quire-cli`; layout assertion planned |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-001 | The steps run in ADR-011 T-13 order | Integration | FR-001-AC-4 | ✅ Tests present |
| TC-002 | An unknown BackendId refuses the run before negotiation | Integration | FR-001-AC-2 | ✅ Tests present |
| TC-003 | route runs after negotiation, for supported items only | Integration | FR-001-AC-3 | ✅ Tests present |
| TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 | Integration | FR-001-AC-1, FR-001-AC-11, FR-012-AC-13, FR-014-AC-3 | ✅ Driver closure assertion present; downstream edge and default-member assertion are planned |
| TC-006 | E5 admits the emitted package and CG generates for the routed items | Integration | FR-001-AC-6, FR-001-AC-7, FR-001-AC-8 | ✅ Tests present |
| TC-007 | Generation refusals are typed | Integration | FR-001-AC-5, FR-001-AC-6 | ✅ Tests present |
| TC-008 | A missing import or an unknown header profile refuses at compile | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-009 | Exercise CLI verbs, outcome documents and exit codes | Integration | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5, FR-014-AC-4 | 🚧 Planned in `quire-cli/tests/cli_contract.rs`; fixtures belong to `quire-cli` |
| TC-010 | Apply explicit lifecycle limits and preserve limit outcomes | Integration | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3 | ✅ Tests present |
| TC-011 | Select only the requested execution engine | Integration | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | 🚧 Planned |
| TC-012 | Register and invoke a first-party provider through Provider | Integration | FR-005-AC-1..FR-005-AC-5 | 🚧 `src/registry.rs` covers distinct lookup, Linked-origin FR-005-AC-1, and identical-manifest idempotence without order-dependent selection; `tests/drive.rs` tests the exhaustive Linked/Process projection and caller ceilings in the generated harness identity; `src/provider.rs` covers cancellation, backend mismatch, missing limits/harness, execution-ceiling mismatch, typed ceiling/memory/symbolic-argument evidence carriers and CG terminal mappings; `tests/drive.rs` exercises the public `Provider::run` path with a selected item. Real Kani execution/result forwarding and Process-origin plugin registration remain planned. |
| TC-013 | Dispatch process-provider results through the driver | Integration | FR-006-AC-1, FR-006-AC-2 | 🚧 Planned |
| TC-014 | Exercise the cache-aware provider-call helper through a recording cache interface | Unit | FR-007-AC-1..FR-007-AC-6 | 🚧 Helper tests cover cache-call semantics AC-3..AC-5; admission-to-dispatch integration AC-1, AC-2 and AC-6 remains planned |
| TC-015 | Invoke AOT with current checked request inputs | Integration | FR-008-AC-1, FR-008-AC-2 | 🚧 Planned |
| TC-016 | Settle one terminal record per item after replay | Integration | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | 🚧 Execute and available proved-provider outputs use QSL's canonical serializer; falsified scalar mapping remains unguessed. Scalar replay and full per-item accounting remain planned. |
| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
| TC-018 | Preserve the QSL outcome at the binary process boundary | Integration | FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-014-AC-4 | 🚧 Planned in `quire-cli/tests/binary_process.rs`; fixtures belong to `quire-cli` |
| TC-019 | Propagate library cancellation and retain bounded checked-unit source | Integration | FR-001-AC-9, FR-001-AC-10, FR-011-AC-1..FR-011-AC-5 | ✅ `tests/drive.rs::tc_019_cancellation_preserves_every_stage_and_cause`; `tests/drive.rs::tc_019_e5_receives_source_provision_and_checked_unit_retains_bounded_root` |
| TC-020 | Derive cache identity from meaning inputs and manifest content | Unit | FR-012-AC-1..FR-012-AC-5 | 🚧 Key, target, and excluded-field projection tests are present; `Trace:` tags identify covered criteria |
| TC-021 | Reject invalid cache entries and excluded values | Unit | FR-012-AC-6..FR-012-AC-8 | ✅ Quire matrix binds AC-6, AC-7, and AC-8 to `quire-cache/src/lib.rs:942`; typed invalid-entry readers also bind AC-6 |
| TC-022 | Keep cache writes atomic and within the caller's size limit | Unit | FR-012-AC-9..FR-012-AC-12 | ✅ Size, rename boundary and rename NotFound, temp cleanup, ownership-filtered eviction, concurrency and removal-race tests carry `Trace:` tags |
| TC-028 | Keep the CLI package separate from the driver and compose the cache | Integration | FR-014-AC-1, FR-014-AC-2, FR-014-AC-4, FR-014-AC-5 | 🚧 Planned package metadata and CLI ownership test |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Verify cache calls carry eligible requests and admitted manifest identity and preserve typed hits | `quire-driver` cache interface with a recording implementation | component | TC-014 |
| Verify content identity, typed reads, exclusion policy and bounded atomic storage | Downstream `quire-cache` package and caller-named temporary directory | component | TC-020, TC-021, TC-022 |
