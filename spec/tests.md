---
id: TM-001
title: "Driver command and execution test matrix"
type: TestMatrix
---
# TM-001: Driver binary, provider and execution test matrix

## Overview

This authored verification inventory integrates FR-001 and SP-D1 through
SP-D9 (FR-002 through FR-010) from IR-527, FR-011 from IR-616, FR-012 for
the downstream cache component, FR-013 for the process plugin host, and FR-014
for the CLI package boundary. It covers FR-001 through FR-014 and TC-001
through TC-028. Discrete FR and TC artifacts remain authoritative;
this matrix is their index, not a computed implementation-coverage report.

`✅` means the named verification is implemented in the listed artifact;
`🚧` means some or all verification remains planned, as described in the row.
These markers do not claim a fresh test run or compatibility with current
upstream APIs.

## Verification Ownership

| Scope | Verification owner and location |
|---|---|
| FR-001 / TC-001..TC-008 | Driver orchestration tests in `tests/drive.rs`, `tests/dependency_edges.rs` and `src/refusal.rs`, as named by the TC artifacts; upstream API adaptation is IR-606. |
| FR-001-AC-9..10, FR-011 / TC-019 | Driver library cancellation matrix and source-provision lifetime tests; API/lifetime review. |
| FR-002..FR-009 / TC-009..TC-016, TC-018 | QSL-backed execute and available proof-terminal serializers have unit coverage; `quire-cli` process tests cover selected-engine and limit exit behavior; full outcome-contract and prove/replay composition tests remain planned. |
| Plugin wire, cache policy, execution semantics and AOT/interpreter parity | QSL ADR-029 owns the protocol and policy. Driver `quire-plugin-host` conformance is specified by TC-023..TC-026 under FR-013; implementation tests cover the listed session, typed-failure, cancellation/deadline and budget cases, with remaining cases identified in the matrix rows. TC-013, TC-014 and TC-015 specify composition boundaries; `quire-cache` behavior is verified by TC-020..TC-022 under FR-012. |
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
| FR-001 | FR-001-AC-1..FR-001-AC-11 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007, TC-008, TC-019 | 🚧 Partial: `tests/dependency_edges.rs` checks the driver/cache/host graph boundaries and `tests/drive.rs` carries explicit FR-001-AC-9/AC-10 Trace IDs; full cross-repository lifecycle evidence remains planned. |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-5 | TC-009, TC-018, TC-028 | 🚧 Partial: `quire-cli/tests/binary_process.rs` checks successful outcome bytes, `invalid-request` and `io-error` documents against QSL-built documents, and named root-argv refusals. A `cli.rs` unit test checks that a QSL diagnostic in a refusal document is written to the stream. Typed check-stage adapter cases remain planned; `ExecutionFailure` documents await QSL-653 and other verbs remain unimplemented. Linux/macOS signal behavior is tracked separately under AC-7. |
| FR-002 | FR-002-AC-4 | TC-009, TC-018 | 🚧 `TC-018` step 1 inspects the production call through `ix_cli_kit::exit::caller_exit_code`; the wrapper unit test checks QSL's category results, and the kit's exhaustive `u8` contract is tested in ix-cli-kit. Full CLI outcome composition remains planned. |
| FR-002 | FR-002-AC-6 | TC-009, TC-018 | ✅ `quire-cli/tests/binary_process.rs::reserved_exit_code_uses_internal_failure_fallback` exercises the test-feature seam in text and machine modes; status `30`, empty stdout, and stderr naming reserved `5` and fallback `30` are asserted. |
| FR-002 | FR-002-AC-7 | TC-028 | 🚧 Linux/macOS cleanup and signal re-raise are exercised through the adapter; full CLI dispatch integration remains planned. |
| FR-002 | FR-002-AC-8 | TC-009, TC-018 | 🚧 Partial: diagnostics use `ix_cli_kit::streams::Record::write` and machine results use `write_primary_stdout`; unit and process-level tests cover failed stderr writes without replacing refusal/fallback statuses. Process-level stdout failure and broken-pipe coverage remains planned. |
| FR-002 | FR-002-AC-9 | TC-009, TC-018 | 🚧 Partial: binary-process tests compare successful outcomes and request-refusal/I/O-error documents with QSL-built canonical documents. The four TC-018 test-adapter cases (refused-range, profile-gated, cancelled and fault) remain planned; `ExecutionFailure` documents await QSL-653. |
| FR-002 | FR-002-AC-10 | TC-009 | 🚧 No config reader exists. If configuration is added, it must use ix-cli-kit `config`; unnamed configuration reads remain prohibited. |
| FR-002 | FR-002-AC-11 | TC-009 | ✅ `quire-cli/tests/cli_contract.rs::version_command_reports_the_quire_cli_package_version` checks `--version` against the `quire-cli` Cargo package version. |
| FR-002 | FR-002-AC-12 | TC-009 | ✅ `quire-cli/tests/cli_contract.rs::built_binary_version_agrees_with_cargo_package_version` checks the built binary with `ix_cli_kit::version::Agreement::command`. |
| FR-003 | FR-003-AC-1, FR-003-AC-3 | TC-010 | 🚧 Existing tests cover an `s1.input_bytes` override, typed stage-limit metadata through the driver wrapper, exit category and raised-limit completion. QSL operand parsing/application must replace the driver parser; the ten accounting bounds remain unavailable until QSL-656. |
| FR-003 | FR-003-AC-2, FR-003-AC-7, FR-003-AC-8 | TC-010 | 🚧 Existing parser tests cover malformed values, unknown fields and duplicates, but do not exercise QSL's `parse_operands` / `CallerLimits::from_operands`; the case `normalization.work_units` accepted by the driver but rejected by QSL is planned. |
| FR-003 | FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-10 | TC-010 | 🚧 QSL API use, operation `bounds()` discrimination, operation-unavailable diagnostics, typed application and platform-overflow refusal remain planned for IR-684. |
| FR-003 | FR-003-AC-9 | TC-010 | 🚧 Ten `accounting.<counter>` settings depend on QSL-656; former bare counter names are not accepted before or after namespaced support lands. |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-011 | 🚧 Planned: current tests compare the linked interpreter's typed preparation refusal with direct QSL output, but cross-engine no-substitution for a selected-engine failure awaits a second engine or injectable seam. CLI parser and binary-process tests cover engine selection and unavailable-engine refusal. |
| FR-005 | FR-005-AC-1..FR-005-AC-4 | TC-012 | 🚧 The Linked Kani Provider slice covers QSL manifest admission/lookup, caller-supplied Linked origin, distinct lookup, and identical-manifest idempotence, plus Kani preflight refusals, explicit `ProofCeilings` propagation and mismatch refusal, typed CG evidence fields, and CG terminal mapping. Process-origin registration/duplicate behavior and full routed invocation remain planned. |
| FR-005 | FR-005-AC-5 | TC-012, TC-028 | 🚧 Partial: `tests/drive.rs` covers projection of a caller-supplied Process origin; process hello admission and registration through the one QSL registry remain planned for IR-672, and CG process backend-kind mapping remains gated on IR-629. |
| FR-006 | FR-006-AC-1, FR-006-AC-2 | TC-013, TC-028 | 🚧 Driver seam and CLI host composition tests planned; FR-006-AC-2 remains planned pending IR-677 ProviderResults generalization |
| FR-006 | FR-006-AC-3..FR-006-AC-4 | TC-028 | 🚧 Normal CLI exit and Linux/macOS handled SIGINT/SIGTERM cleanup remain planned for the IR-672 adapter. The existing Windows forced cleanup path (`terminate_tree`/`terminate_job`) awaits native evidence in TC-028 step 9; Windows console-control cleanup is not claimed. |
| FR-007 | FR-007-AC-1..FR-007-AC-6 | TC-014 | 🚧 AC-1, AC-2 and AC-6 remain planned until provider dispatch is integrated; the helper tests cover cache-call semantics AC-3..AC-5 |
| FR-008 | FR-008-AC-1, FR-008-AC-2 | TC-015 | 🚧 Planned |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | TC-016 | 🚧 Execute and available proved-provider terminals serialize through QSL `OutcomeDocument`; falsified scalar results without replay remain typed errors. Full replay mapping and per-item accounting remain planned. |
| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 | TC-017 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
| FR-011 | FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-011-AC-4, FR-011-AC-5 | TC-019 | ✅ Cancellation matrix and API mapping tests present; CLI adapter remains out of scope |
| FR-012 | FR-012-AC-1..FR-012-AC-5 | TC-020 | ✅ Key members, target shape, excluded metadata projection and meaning-input changes are tested with `Trace:` tags |
| FR-012 | FR-012-AC-6..FR-012-AC-8 | TC-021 | ✅ Matrix binds AC-6, AC-7, and AC-8 to the typed hit/miss, excluded-value, and deterministic incomplete-value test at `quire-cache/src/lib.rs:942` |
| FR-012 | FR-012-AC-9..FR-012-AC-12 | TC-022 | ✅ Entry bounds, temp cleanup, rename errors, ownership-filtered eviction, concurrent writes and eviction-callsite NotFound are tested with `Trace:` tags |
| FR-012 | FR-012-AC-13 | TC-005 | ✅ `tests/dependency_edges.rs::tc_005_driver_edges_match_the_adr_011_allow_list` carries a `Trace:` tag |
| FR-013 | FR-013-AC-1..FR-013-AC-3 | TC-023 | 🚧 Partial: tests refuse bare names and cover hello admission, rejected protocol, and zero, one and multiple requests; the explicit no-request-frame assertion remains planned. |
| FR-013 | FR-013-AC-4, FR-013-AC-7 | TC-024 | 🚧 Partial: tests cover invalid typed result, wrong request id, fault, non-zero exit, truncated hello, non-canonical/oversized frames and unexpected extra frame; malformed JSON and truncated-body cases remain planned. |
| FR-013 | FR-013-AC-5..FR-013-AC-6 | TC-025 | 🚧 Partial: valid settlement and fault during cancellation grace, grace-expiry descendant cleanup within one second, and blocked-write deadline are tested. |
| FR-013 | FR-013-AC-8..FR-013-AC-10 | TC-026 | 🚧 Partial: Linux CPU and memory aggregation across sampled process-group members, multithreaded process accounting, stderr truncation, EOF exit-status preservation, descendant cleanup and bounded reader joins are tested. A fork/exit smoke test exercises concurrent churn while sampling but does not force a particular stat-read interleaving. macOS and Windows budget enforcement remains planned in IR-671; requests with those budgets currently fail closed as Unavailable. CPU usage by members that exit between samples remains a measured limitation under IR-668. Windows Job Object cleanup is planned under IR-674 and lacks native Windows evidence. |
| FR-013 | FR-013-AC-11 | TC-005, TC-028 | 🚧 `tests/dependency_edges.rs` and `quire-cli/tests/package_boundary.rs` already assert the direct quire-cli-to-host edge and host-free driver closure; full TC-028 adapter composition remains planned for IR-672. |
| FR-013 | FR-013-AC-14 | TC-028 | 🚧 Automatic Windows Job Object kill-on-close remains planned under IR-674 and requires native Windows evidence; this row covers only kill-on-close, not the separate forced-cleanup path. |
| FR-013 | FR-013-AC-15 | TC-028 | 🚧 Planned analysis/inspection of `quire-plugin-host/src/lib.rs`; runtime failure evidence requires a distinguishable or injectable setup-failure seam and native Windows runner. |
| FR-013 | FR-013-AC-12 | TC-027 | 🚧 Makefile `test` already runs `cargo fetch --locked`; clean-cache fetch, offline metadata and full-test evidence remain planned |
| FR-013 | FR-013-AC-13 | TC-027 | ✅ Python 3 prerequisite test emits an actionable installation message |
| FR-014 | FR-014-AC-1 | TC-028 | ✅ `quire-cli/tests/package_boundary.rs::cargo_target_and_dependencies_belong_to_quire_cli` asserts the sole `quire` binary target is in `quire-cli`. |
| FR-014 | FR-014-AC-2 | TC-005, TC-028 | ✅ `tests/dependency_edges.rs` asserts the direct CLI edges and that the driver closure excludes the kit; `quire-cli/tests/package_boundary.rs` checks the direct kit edge. |
| FR-014 | FR-014-AC-3 | TC-005 | ✅ TC-005 asserts `quire-cli` workspace default-member status |
| FR-014 | FR-014-AC-4 | TC-009, TC-018, TC-028 | ✅ `quire-cli/tests/package_boundary.rs::parser_and_cli_tests_live_in_the_cli_package` asserts CLI test and fixture ownership. CLI behavior remains tracked separately by TC-009/TC-018. |
| FR-014 | FR-014-AC-5 | TC-028 | ✅ `quire-cli/tests/package_boundary.rs::parser_and_cli_tests_live_in_the_cli_package` asserts parser and binary entry point locations. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-001 | The steps run in ADR-011 T-13 order | Integration | FR-001-AC-4 | ✅ Tests present |
| TC-002 | An unknown BackendId refuses the run before negotiation | Integration | FR-001-AC-2 | ✅ Tests present |
| TC-003 | route runs after negotiation, for supported items only | Integration | FR-001-AC-3 | ✅ Tests present |
| TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 | Integration | FR-001-AC-1, FR-001-AC-11, FR-012-AC-13, FR-013-AC-11, FR-014-AC-2, FR-014-AC-3 | ✅ Driver/cache edges, host-free and kit-free driver closure, direct CLI-to-host/kit edges and CLI default membership are asserted. |
| TC-006 | E5 admits the emitted package and CG generates for the routed items | Integration | FR-001-AC-6, FR-001-AC-7, FR-001-AC-8 | ✅ Tests present |
| TC-007 | Generation refusals are typed | Integration | FR-001-AC-5, FR-001-AC-6 | ✅ Tests present |
| TC-008 | A missing import or an unknown header profile refuses at compile | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-009 | Exercise CLI verbs, outcome documents and exit codes | Integration | FR-002-AC-1..FR-002-AC-6, FR-002-AC-8..FR-002-AC-12, FR-014-AC-4 | 🚧 Partial: `quire-cli/tests/binary_process.rs` covers the code-5 fallback, successful machine outcome bytes, `invalid-request` and `io-error` documents, and named root-argv refusals; `quire-cli/tests/cli_contract.rs` covers package version output and built-binary Agreement. Stream write failures, TC-018 check-stage adapter cases, `ExecutionFailure` documents (QSL-653), configuration, and remaining verbs are planned. Fixtures belong to `quire-cli`. |
| TC-010 | Apply explicit lifecycle limits and preserve limit outcomes | Integration | FR-003-AC-1..FR-003-AC-10 | 🚧 Existing tests cover an `s1.input_bytes` override and typed limit outcomes. QSL operand parsing, accepted-name agreement (including rejection of `normalization.work_units`), operation-bound discrimination, distinct refusal diagnostics, platform overflow and ten namespaced accounting settings (QSL-656) remain planned for IR-684. |
| TC-011 | Select only the requested execution engine | Integration | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | 🚧 Planned: `src/execute.rs` verifies interpreter selection, unavailable-engine refusal and exact typed preparation-refusal parity with direct QSL; CLI tests cover option parsing and process status. Cross-engine no-substitution for a selected-engine failure awaits a second engine or injectable seam. |
| TC-012 | Register and invoke a first-party provider through Provider | Integration | FR-005-AC-1..FR-005-AC-5 | 🚧 `src/registry.rs` covers distinct lookup, Linked-origin FR-005-AC-1, and identical-manifest idempotence without order-dependent selection; `tests/drive.rs` tests the exhaustive Linked/Process projection and caller ceilings in the generated harness identity; `src/provider.rs` covers cancellation, backend mismatch, missing limits/harness, execution-ceiling mismatch, typed ceiling/memory/symbolic-argument evidence carriers and CG terminal mappings; `tests/drive.rs` exercises the public `Provider::run` path with a selected item. Real Kani execution/result forwarding and Process-origin plugin registration remain planned. |
| TC-013 | Dispatch process-provider results through the driver | Integration | FR-006-AC-1, FR-006-AC-2 | 🚧 Planned driver seam and settlement composition |
| TC-014 | Exercise the cache-aware provider-call helper through a recording cache interface | Unit | FR-007-AC-1..FR-007-AC-6 | 🚧 Helper tests cover cache-call semantics AC-3..AC-5; admission-to-dispatch integration AC-1, AC-2 and AC-6 remains planned |
| TC-015 | Invoke AOT with current checked request inputs | Integration | FR-008-AC-1, FR-008-AC-2 | 🚧 Planned |
| TC-016 | Settle one terminal record per item after replay | Integration | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | 🚧 Execute and available proved-provider outputs use QSL's canonical serializer; falsified scalar mapping remains unguessed. Scalar replay and full per-item accounting remain planned. |
| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
| TC-018 | Preserve the QSL outcome at the binary process boundary | Integration | FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-6, FR-002-AC-8, FR-002-AC-9, FR-014-AC-4 | 🚧 Partial: binary-process tests cover the code-5 fallback, machine-outcome bytes, request/I/O refusals, and stream diagnostics. The four test-adapter cases (refused-range, profile-gated, cancelled and fault), full locus-rendered diagnostic comparison, stdout failure/broken-pipe cases, and `ExecutionFailure` documents (QSL-653) remain planned. Fixtures belong to `quire-cli`. |
| TC-019 | Propagate library cancellation and retain bounded checked-unit source | Integration | FR-001-AC-9, FR-001-AC-10, FR-011-AC-1..FR-011-AC-5 | ✅ Cancellation/source-lifetime tests are present with explicit comma-separated Trace IDs for FR-001-AC-9, FR-001-AC-10, and FR-011-AC-1..FR-011-AC-4. |
| TC-020 | Derive cache identity from meaning inputs and manifest content | Unit | FR-012-AC-1..FR-012-AC-5 | 🚧 Key, target, and excluded-field projection tests are present; `Trace:` tags identify covered criteria |
| TC-021 | Reject invalid cache entries and excluded values | Unit | FR-012-AC-6..FR-012-AC-8 | ✅ Quire matrix binds AC-6, AC-7, and AC-8 to `quire-cache/src/lib.rs:942`; typed invalid-entry readers also bind AC-6 |
| TC-022 | Keep cache writes atomic and within the caller's size limit | Unit | FR-012-AC-9..FR-012-AC-12 | ✅ Size, rename boundary and rename NotFound, temp cleanup, ownership-filtered eviction, concurrency and removal-race tests carry `Trace:` tags |
| TC-023 | Start a plugin session and admit hello before requests | Integration | FR-013-AC-1..FR-013-AC-3 | 🚧 Partial: executable-name refusal, hello admission, rejected protocol, and zero/one/two requests are tested; the explicit no-request-frame assertion remains planned. |
| TC-024 | Enforce plugin framing and preserve typed failures | Integration | FR-013-AC-4, FR-013-AC-7 | 🚧 Partial: invalid result, wrong request id, plugin fault, non-zero exit, truncated hello, non-canonical/oversized frames, and unexpected extra frame are tested; malformed JSON and truncated-body cases remain planned. |
| TC-025 | Preserve plugin settlement during cancellation and bound deadlines | Integration | FR-013-AC-5..FR-013-AC-6 | 🚧 Partial: cancellation settlement/fault, grace-expiry descendant cleanup within one second, and blocked-write deadline tests exist. |
| TC-026 | Enforce plugin process-tree budgets and bounded cleanup | Integration | FR-013-AC-8..FR-013-AC-10 | 🚧 Partial: Linux CPU/memory aggregation across sampled group members, a fork/exit churn smoke test, unavailable measurement, multithreaded accounting, stderr truncation, EOF status, descendant cleanup, and bounded joins are tested; the smoke test does not force a particular stat-read interleaving. macOS and Windows budget enforcement is planned in IR-671 and currently fails closed as Unavailable. CPU between-sample accounting is measured as partial under IR-668; Windows Job Object cleanup is planned under IR-674. |
| TC-027 | Run plugin-host tests and dependency checks from a clean cache | Integration | FR-013-AC-12, FR-013-AC-13 | 🚧 Python prerequisite message is tested; clean-cache fetch, offline metadata, and full gate evidence remain planned. |
| TC-028 | Keep the CLI package separate from the driver and compose cache and process host | Integration | FR-002-AC-3, FR-002-AC-7, FR-006-AC-1, FR-006-AC-3..FR-006-AC-4, FR-013-AC-11, FR-013-AC-14..FR-013-AC-15, FR-014-AC-1, FR-014-AC-2, FR-014-AC-4, FR-014-AC-5 | 🚧 Existing package-boundary tests cover target ownership, layout and the direct CLI-to-host edge; the kit edge assertion is inverted, and adapter lifecycle/signal tests are partial. Windows forced cleanup requires native Windows evidence; automatic kill-on-close remains planned under IR-674. |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Verify cache calls carry eligible requests and admitted manifest identity and preserve typed hits | `quire-driver` cache interface with a recording implementation | component | TC-014 |
| Verify content identity, typed reads, exclusion policy and bounded atomic storage | Downstream `quire-cache` package and caller-named temporary directory | component | TC-020, TC-021, TC-022 |
| Verify plugin sessions, protocol, typed outcomes, cancellation, deadlines, budgets and cleanup | `quire-plugin-host` with deterministic process fixtures | component | TC-023, TC-024, TC-025, TC-026, TC-027 |
| Verify CLI process-provider composition and cleanup on exit and handled signals | `quire-cli` adapter and `quire-plugin-host` with a descendant-spawning fixture | component | TC-028 |
