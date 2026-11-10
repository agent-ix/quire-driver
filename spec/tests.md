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
for the CLI package boundary, and planned FR-015 for admitted monitor program
preparation. It covers FR-001 through FR-019 and TC-001 through TC-033.
Discrete FR and TC artifacts remain authoritative;
this matrix is their index, not a computed implementation-coverage report.

`✅` means the named verification is implemented in the listed artifact;
`🚧` means some or all verification remains planned, as described in the row.
These markers do not claim a fresh test run or compatibility with current
upstream APIs.

## Verification Ownership

| Scope | Verification owner and location |
|---|---|
| FR-001 / TC-001..TC-008 | Driver orchestration tests in `tests/drive.rs`, `tests/dependency_edges.rs` and `src/refusal.rs`, as named by the TC artifacts; TC-006 includes the IR-757 KaniForm projection and occurrence-context regression. |
| FR-001-AC-9..10, FR-011 / TC-019 | Driver library cancellation matrix and source-provision lifetime tests; API/lifetime review. |
| FR-002..FR-009 / TC-009..TC-016, TC-018 | QSL-backed execute and available proof-terminal serializers have unit coverage; `quire-cli` process tests cover selected-engine and limit exit behavior; full outcome-contract and prove/replay composition tests remain planned. |
| Plugin wire, cache policy, execution semantics and AOT/interpreter parity | QSL ADR-029 owns the protocol and policy. Driver `quire-plugin-host` conformance is specified by TC-023..TC-026 under FR-013; implementation tests cover the listed session, typed-failure, cancellation/deadline, stderr and cleanup cases, with remaining cases identified in the matrix rows. TC-013, TC-014 and TC-015 specify composition boundaries; `quire-cache` behavior is verified by TC-020..TC-022 under FR-012. |
| FR-010 / TC-017 | Planned QSL-owned cross-repository test in `quire-integration`, QSL-514; QSL AR2 is the prerequisite. |
| FR-015 / TC-029 | Planned QSL producer, strict IR admission, complete IR FR-357 generated-source projection, driver lowering and RT build/execution evidence. The twelve complete-preparation FR-015 criteria remain planned and untagged; TC-029 supplies the procedure, not execution credit. |
| G5b acceptance evidence / IR-528 | `quire-integration` only: CLI/library parity across the eight outcome kinds with identical outcome bytes and exit codes; provider/plugin installation; cache hit/miss/never-store; AOT/interpreter parity; absence of a QSL user binary; and prove execution, replay and FR-331 records. End-to-end evidence is required on every OS declared supported at gate freeze; one platform's result does not establish acceptance on another. |

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
| FR-001 | FR-001-AC-1..FR-001-AC-16 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007, TC-008, TC-019 | 🚧 Partial: `tests/drive.rs::tc_006_rr16_preserves_each_occurrence_conversion_context` covers typed-form projection and distinct occurrence-context roots. Production QSL Call binding, request-index sidecar retention, and forwarding through generation (AC-13..AC-16) remain planned under IR-879. Configured Process-provider invocation is exercised under TC-013 and TC-028. |
| FR-001 | FR-001-AC-17..FR-001-AC-21 | TC-017 | 🚧 `tests/drive.rs` covers the export partition, whole-export refusal, unknown driver item and relation-less skip; cross-repository composition remains in `quire-integration` PR 16. |
| FR-001 | FR-001-AC-22 | TC-004 | ✅ `src/refusal.rs` tests retain all eight typed refusal fields in both present and absent forms. `DriverRefusal::from` constructs a typed FR-272 diagnostic with the code, cause, category and `package_read` stage; it maps the CG path to an FR-274 `JsonPath` location and retains the `CheckedNodeId` locus. Tests cover the five permitted incomplete limits, the `Diagnostics` invariant fault, and each unchanged CG projection. `src/drive.rs` converts the CG negotiation error through that constructor. FR-460/FR-461 reader-limit registration remains deferred. |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-5 | TC-009, TC-018, TC-028 | 🚧 Partial: `quire-cli/tests/cli_contract.rs` compares `compile`, `check`, `parse`, and `format` machine documents with QSL-built outcomes; `compile` uses QSL's parse/select/check/package path. Its working-directory trap test verifies `check` reads the explicitly named source without reading malformed `quire.toml` (AC-5). Other verbs remain explicitly unsupported. `binary_process.rs` checks successful outcome bytes, `invalid-request` and `io-error` documents against QSL-built documents, plus named root-argv refusals. Check-stage semantic refusal cases and `ExecutionFailure` documents remain planned; Linux/macOS signal behavior is tracked separately under AC-7. |
| FR-002 | FR-002-AC-4 | TC-009, TC-018 | 🚧 `TC-018` step 1 inspects the production call through `ix_cli_kit::exit::caller_exit_code`; the wrapper unit test checks QSL's category results, and the kit's exhaustive `u8` contract is tested in ix-cli-kit. Full CLI outcome composition remains planned. |
| FR-002 | FR-002-AC-6 | TC-009, TC-018 | ✅ `quire-cli/tests/binary_process.rs::reserved_exit_code_uses_internal_failure_fallback` exercises the test-feature seam in text and machine modes; status `30`, empty stdout, and stderr naming reserved `5` and fallback `30` are asserted. |
| FR-002 | FR-002-AC-7 | TC-028 | 🚧 Linux/macOS cleanup and signal re-raise are exercised through the adapter; full CLI dispatch integration remains planned. |
| FR-002 | FR-002-AC-8 | TC-009, TC-018 | 🚧 Partial: diagnostics use `ix_cli_kit::streams::Record::write` and machine results use `write_primary_stdout`; unit and process-level tests cover failed stderr writes without replacing refusal/fallback statuses. |
| FR-002 | FR-002-AC-14..FR-002-AC-16 | TC-018 | 🚧 Partial: `quire-cli/tests/binary_process.rs` verifies the refusal exit and absence of `OutputFailure` for a FIFO `BrokenPipe` on Unix, plus status `30` and an `OutputFailure` diagnostic for Linux `/dev/full`. Windows broken-pipe evidence and non-Linux non-broken stdout-I/O-error evidence remain open. |
| FR-002 | FR-002-AC-9 | TC-009, TC-018 | 🚧 Partial: binary-process tests compare successful outcomes and request-refusal/I/O-error documents with QSL-built canonical documents. The four TC-018 test-adapter cases (refused-range, profile-gated, cancelled and fault) remain planned; `ExecutionFailure` documents await QSL-653. |
| FR-002 | FR-002-AC-10 | TC-009 | 🚧 No config reader exists. If configuration is added, it must use ix-cli-kit `config`; unnamed configuration reads remain prohibited. |
| FR-002 | FR-002-AC-11 | TC-009 | ✅ `quire-cli/tests/cli_contract.rs::version_command_reports_the_quire_cli_package_version` checks `--version` against the `quire-cli` Cargo package version. |
| FR-002 | FR-002-AC-12 | TC-009 | ✅ `quire-cli/tests/cli_contract.rs::built_binary_version_agrees_with_cargo_package_version` checks the built binary with `ix_cli_kit::version::Agreement::command`. |
| FR-002 | FR-002-AC-13 | TC-009 | ✅ `quire-cli/tests/binary_process.rs::unavailable_text_renderers_return_typed_unsupported_refusals` checks default and explicit text mode for parse/check/compile/run; QSL-600/QSL-601 provide the pending renderer. |
| FR-003 | FR-003-AC-1, FR-003-AC-3 | TC-010 | ✅ `src/limits.rs` applies each exposed value through QSL bounds and `CallerLimits::set_bound`; `src/execute.rs::caller_source_limit_preserves_configured_bound_and_field` checks typed metadata through the driver wrapper; `binary_process.rs` checks incomplete stage-limit status and completion after raising the bound. |
| FR-003 | FR-003-AC-2, FR-003-AC-7, FR-003-AC-8 | TC-010 | ✅ `src/limits.rs` asserts QSL parser refusal causes; `binary_process.rs::qsl_rejects_unknown_malformed_and_duplicate_limits_before_source_read` checks operands and pre-stage refusal. |
| FR-003 | FR-003-AC-4 | TC-010 | ✅ Inspection: `src/limits.rs` uses QSL `parse_operands`, selected-operation `bounds()`, and `CallerLimits::set_bound`; the driver grammar/table is removed. |
| FR-003 | FR-003-AC-5 | TC-010 | ✅ `src/limits.rs::recognized_but_unavailable_setting_has_operation_specific_error` distinguishes a QSL-recognized setting unavailable to `run` from an unknown name. |
| FR-003 | FR-003-AC-6 | TC-010 | ✅ `src/limits.rs::every_exposed_setting_updates_only_its_qsl_bound` checks each exposed setting changes only its own bound. |
| FR-003 | FR-003-AC-9, FR-003-AC-10 | TC-010 | ✅ `src/limits.rs::each_named_accounting_setting_updates_only_call_accounting` checks all QSL accounting names update only `Call.accounting`; `quire-cli/tests/binary_process.rs::named_accounting_work_units_limit_reaches_qsl_call_accounting` verifies `accounting.work_units=0` reaches QSL and returns the typed incomplete outcome. |
| FR-004 | FR-004-AC-1..FR-004-AC-10 | TC-011 | 🚧 Partial: CLI parser and binary-process tests cover interpreter selection, unavailable-engine refusal, and unknown engine values. `caller_object_context_controls_lookup_result`, `absent_undefined_lookup_matches_direct_qsl`, and `unresolved_population_identity_matches_direct_qsl_refusal` compare driver results/documents with direct QSL using the same object environments. `caller_object_environment_is_forwarded_unchanged` proves reference forwarding, and the CLI explicitly supplies its empty-environment policy. The private `execute_using` callback can characterize exact typed QSL `RunRefusal::UnsupportedResult` and preparation-refusal forwarding. Its invocation count cannot detect direct engine calls outside that callback. Full AC-4 evidence remains blocked on a second production engine or upstream-supported shared dispatch boundary that observes every engine call; the test must assert exact failure, one selected invocation, and zero alternative invocations. |
| FR-005 | FR-005-AC-1..FR-005-AC-4 | TC-012 | 🚧 The Linked Kani Provider slice covers QSL manifest admission/lookup, caller-supplied Linked origin, distinct lookup, and identical-manifest idempotence, plus Kani preflight refusals, explicit `ProofCeilings` propagation and mismatch refusal, typed CG evidence fields, and CG terminal mapping. Process-origin registration/duplicate behavior and full routed invocation remain planned. |
| FR-005 | FR-005-AC-5 | TC-012, TC-028 | 🚧 Partial: `tests/drive.rs` and the FR-331 host fixture cover caller-supplied Process origin, one hello admission and registration into the same QSL registry with its retained DTO. TC-012 now verifies CG's origin-aware kind conversion; the configured composition test exercises production dispatch. |
| FR-005 | FR-005-AC-12..FR-005-AC-15 | TC-013 | 🚧 Partial: `quire-cli/src/process_provider.rs::fr331_hello_is_admitted_once_and_retained_for_registry_snapshot` checks one QSL hello admission, registration into the same registry, and lossless carrier retention. |
| FR-005 | FR-005-AC-16 | TC-012 | ✅ `src/registry.rs` checks canonical linked bytes, absent optional members, byte-for-byte preservation of supplied wire, and typed refusal of empty advertisements; `quire-cli/src/backend_provider.rs::request_stage_refuses_missing_manifest_wire_without_reconstruction` checks the adapter refuses instead of reconstructing missing bytes. |
| FR-005 | FR-005-AC-18..FR-005-AC-19 | TC-013 | 🚧 Partial: the process hello adapter passes domains to QSL and preserves its typed `invalid-domains` refusal; absent/unknown kind and mode refusals also remain QSL-typed. Evidence-kind admission is QSL-owned and awaits QSL-654; fairness, obligation-form and reduction values are checked as FR-331 wire shape by the driver and interpreted at negotiation. |
| FR-005 | FR-005-AC-17 | TC-012 | ✅ `src/drive.rs::backend_kind_conversion_uses_admitted_origin` verifies the driver calls CG's descriptor conversion, preserves Process origin for the same identity, and still refuses an unknown Linked identity. |
| FR-005 | FR-005-AC-6 | TC-012, TC-013 | 🚧 Partial: `src/provider.rs::shared_provider_carrier_preserves_non_kani_data_and_failures` verifies generic success/failure data, request indexes and backend identity; the host adapter test verifies a non-Kani carrier survives conversion. End-to-end routed invocation remains planned under IR-672. |
| FR-005 | FR-005-AC-7 | TC-013 | 🚧 Planned under IR-672 and QSL-655: retain a plugin counterexample as a non-terminal candidate until QSL S6a replay reproduces it; do not emit `Refuted` before replay. |
| FR-005 | FR-005-AC-8 | TC-012 | 🚧 Partial: TC-012 exercises CG's authoritative `run_terminal_value` mapping; the driver passes the original evidence and optional settlement, and implements no second mapping. |
| FR-005 | FR-005-AC-20..FR-005-AC-21 | TC-012 | ✅ `src/provider.rs::kani_request_forwards_only_the_explicit_home_directory` forwards a known path and keeps unknown `None` with a decoy child-process `HOME`; `provider_result_preserves_optional_cg_diagnostics_and_failure_cause` retains CG's optional fields unchanged. |
| FR-005 | FR-005-AC-9..FR-005-AC-11 | TC-012, TC-013 | 🚧 Partial: generic non-Kani success/failure carriers are tested under IR-677. Plugin trusted `proved` mapping waits on QSL-490 and QSL-655; replay-only claims become tool failures only after QSL-655 amends ADR-029 PL-7. |
| FR-006 | FR-006-AC-1 | TC-013, TC-028 | 🚧 Partial: `quire-cli/src/cli.rs::configured_run_invokes_only_the_selected_process_and_retains_its_typed_result` starts two real FR-331 fixtures, admits both into one registry, runs QSL/CG composition and sends one selected request through the production host adapter. Full field-projection coverage and later-start cleanup remain incomplete. |
| FR-006 | FR-006-AC-8 | TC-013, TC-028 | ✅ `configured_run_invokes_only_the_selected_process_and_retains_its_typed_result` verifies the selected provider's configured `deadline_ms` is serialized unchanged. |
| FR-006 | FR-006-AC-24 | TC-028 | ✅ `quire-cli/tests/binary_process.rs::provider_config_file_and_schema_errors_emit_one_invalid_request_document` checks missing/unreadable files, malformed JSON, missing and wrongly typed fields, missing/zero/negative/fractional/null/unsafe deadlines, null optional budgets, and unknown fields at all object levels against one `Operation::Execute`/`Code::InvalidRequest` document before provider start. |
| FR-006 | FR-006-AC-25 | TC-028 | 🚧 Partial: `configured_run_invokes_only_the_selected_process_and_retains_its_typed_result` starts and admits both providers before negotiation and dispatches only the selected provider. Later-start/hello failure cleanup with preservation of the original typed host failure remains untested. |
| FR-006 | FR-006-AC-26 | TC-028 | ✅ `quire-cli/tests/binary_process.rs::duplicate_provider_identities_refuse_before_negotiation_and_close_sessions` checks machine `InvalidRequest`, no request markers, and shutdown of both sessions for QSL's idempotent repeated admission path. The composition-level test also checks typed refusal and cleanup diagnostics. |
| FR-006 | FR-006-AC-2 | TC-013 | 🚧 The process adapter preserves generic result carriers; no end-to-end settlement claim is made from an injected boundary. Production routing and typed-result retention are exercised by the configured composition; settlement remains pending QSL result-mapping APIs, including QSL-655 for AC-14..AC-23. |
| FR-006 | FR-006-AC-3 | TC-028 | 🚧 Partial: `configured_run_invokes_only_the_selected_process_and_retains_its_typed_result` and the production signal subprocess cover normal and signal-driven shutdown; the production normal-exit one-second forced-cleanup bound is not asserted. Windows native evidence remains planned. |
| FR-006 | FR-006-AC-4 | TC-028 | 🚧 Partial: `quire-cli/tests/binary_process.rs::configured_process_sigterm_cleans_active_session_before_reraise` exercises production SIGTERM during a running request, verifies bounded cleanup, empty stdout and signal termination. SIGINT and production start/shutdown-wait states remain untested; Windows console control is outside the guarantee. |
| FR-006 | FR-006-AC-5 | TC-013 | 🚧 Partial: `src/registry.rs::linked_manifest_snapshot_is_independent_of_registration_order` checks that the retained registered-candidate snapshot is sorted independently of registration order; production composition retains and serializes that snapshot, while linked-wire population remains IR-688. |
| FR-006 | FR-006-AC-6..FR-006-AC-13 | TC-013, TC-028 | 🚧 Partial: the production fixture checks the selected request's index, kind, claim, backend, candidate set, extent and unchanged deadline. Exact package bytes/id, forbidden top-level members, temporal proof bounds and proof-bound `kind` remain unverified; QSL proof-bound `kind` support awaits QSL-654. |
| FR-006 | FR-006-AC-14 | TC-013 | 🚧 Unclaimed: typed `declined` result mapping remains pending QSL-655; no terminal mapping is inferred. |
| FR-006 | FR-006-AC-15 | TC-013, TC-028 | 🚧 Partial: the production fixture verifies a `Tested` result remains typed in `CommandResult` and is not fabricated as a QSL terminal; the shared settlement path remains pending QSL-655 and IR-677. |
| FR-006 | FR-006-AC-16..FR-006-AC-23 | TC-013 | 🚧 Unclaimed: diagnostic conversion and settlement remain pending QSL-655 and the required QSL result-mapping APIs; no terminal mappings are inferred. |
| FR-007 | FR-007-AC-1..FR-007-AC-6 | TC-014 | 🚧 AC-1, AC-2 and AC-6 remain planned until provider dispatch is integrated; the helper tests cover cache-call semantics AC-3..AC-5 |
| FR-008 | FR-008-AC-1, FR-008-AC-2 | TC-015 | 🚧 Planned |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-10 | TC-016 | 🚧 Execute and available proved-provider terminals serialize through QSL `OutcomeDocument`; per-item terminal coverage applies to requests reaching settlement. Predicate/function replay and scalar parity use their distinct QSL/CG contracts; scalar parity never settles `refuted`. Full replay mapping and per-item accounting remain planned. |
| FR-009 | FR-009-AC-6, FR-009-AC-7, FR-009-AC-8, FR-009-AC-9, FR-009-AC-11, FR-009-AC-12, FR-009-AC-13 | TC-006, TC-016 | 🚧 QSL FR-367 and IR FR-049 APIs are available; production call-to-occurrence binding and the request-index sidecar through generation remain planned under IR-879 and are exercised by TC-006. CG request/meter/returned `ScalarLimits` attestation remains planned under IR-857. Refusal passthrough follows IR-847. Native receipt, replay conversion, limits mismatch, report-claim conversion, and per-item settlement remain planned under IR-773/IR-631/IR-610 and their CG API dependencies. Claim mismatch must use CG's typed converter when available; scalar parity never settles as `refuted`. |
| FR-001, FR-009 | FR-001-AC-23..FR-001-AC-24, FR-009-AC-23..FR-009-AC-24 | TC-016 | 🚧 Planned until IR-1121 supplies CG’s current-V2 function harness and proving-run identity, then IR-610 retains the exact QSL source/dependency lock, selected declaration and occurrence, admitted package, generated identity and provider evidence for E9. Missing or mismatched joins fault before provider/replay; upstream typed refusals keep their owner. |
| FR-009 | FR-009-AC-17..FR-009-AC-19 | TC-016 | 🚧 Planned: the driver projects QSL's exact typed `OperationSite.frame_operation` value to the CG item joined by admitted package identity and request index. QSL FR-121 defines the pair and owner resolution; QSpec FR-323 defines replay-target/request shape. AC-18 preserves QSL typed refusals; AC-19 reserves `runtime_invariant` for a driver-detected inconsistent join. Process/state-machine operations and state-clause projection remain out of scope. |
| FR-009 | FR-009-AC-14..FR-009-AC-15 | TC-016 | ✅ `src/provider.rs::falsified_scalar_does_not_guess_a_replay_settlement` passes the optional settlement to CG's map; `cg_terminal_mapping_reads_original_failure_cause_not_diagnostics` verifies the actual failure cause drives the map and optional diagnostics do not classify it. |
| FR-009 | FR-009-AC-16 | TC-017 | 🚧 `tests/drive.rs` checks the bound-only generation indexes; terminal-record attribution remains in the TC-017 composition. |
| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, FR-010-AC-5 | TC-017 | 🚧 `tests/drive.rs` checks the partition and all-refused paths; `quire-integration` PR 16 checks the cross-repository bound-only path. |
| FR-011 | FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-011-AC-4, FR-011-AC-5 | TC-019 | ✅ Cancellation matrix and API mapping tests present; CLI adapter remains out of scope |
| FR-012 | FR-012-AC-1..FR-012-AC-5 | TC-020 | ✅ Key members, target shape, excluded metadata projection and meaning-input changes are tested with `Trace:` tags |
| FR-012 | FR-012-AC-6..FR-012-AC-8 | TC-021 | ✅ Matrix binds AC-6, AC-7, and AC-8 to the typed hit/miss, excluded-value, and deterministic incomplete-value test at `quire-cache/src/lib.rs:942` |
| FR-012 | FR-012-AC-9..FR-012-AC-12 | TC-022 | ✅ Entry bounds, temp cleanup, rename errors, ownership-filtered eviction, concurrent writes and eviction-callsite NotFound are tested with `Trace:` tags |
| FR-012 | FR-012-AC-13 | TC-005 | ✅ `tests/dependency_edges.rs::tc_005_driver_edges_match_the_adr_011_allow_list` carries a `Trace:` tag |
| FR-013 | FR-013-AC-1..FR-013-AC-3 | TC-023 | 🚧 Partial: tests refuse bare names and cover hello admission, rejected protocol, and zero, one and multiple requests; the explicit no-request-frame assertion remains planned. |
| FR-013 | FR-013-AC-4, FR-013-AC-7 | TC-024 | 🚧 Partial: tests cover invalid typed result, wrong request id, fault, non-zero exit, truncated hello, non-canonical/oversized frames and unexpected extra frame; malformed JSON and truncated-body cases remain planned. |
| FR-013 | FR-013-AC-5..FR-013-AC-6 | TC-025 | 🚧 Partial: valid settlement and fault during cancellation grace, grace-expiry descendant cleanup within one second, and blocked-write deadline are tested. |
| FR-013 | FR-013-AC-9..FR-013-AC-10 | TC-026 | 🚧 Partial: stderr truncation and EOF exit-status preservation, descendant cleanup and bounded reader joins are tested. |
| FR-013 | FR-013-AC-11 | TC-005, TC-028 | 🚧 `tests/dependency_edges.rs` and `quire-cli/tests/package_boundary.rs` already assert the direct quire-cli-to-host edge and host-free driver closure; full TC-028 adapter composition remains planned for IR-672. |
| FR-013 | FR-013-AC-12 | TC-027 | 🚧 Makefile `test` already runs `cargo fetch --locked`; clean-cache fetch, offline metadata and full-test evidence remain planned |
| FR-013 | FR-013-AC-13 | TC-027 | ✅ Python 3 prerequisite test emits an actionable installation message |
| FR-013 | FR-013-AC-14 | TC-028 | 🚧 Deferred to IR-674; this PR has no kill-on-close implementation or test. Native Windows evidence is required when the follow-up is completed. |
| FR-013 | FR-013-AC-15 | TC-028 | 🚧 Planned analysis/inspection of `quire-plugin-host/src/lib.rs`; runtime failure evidence requires a distinguishable or injectable setup-failure seam and native Windows runner. |
| FR-014 | FR-014-AC-1 | TC-028 | ✅ `quire-cli/tests/package_boundary.rs::cargo_target_and_dependencies_belong_to_quire_cli` asserts the sole `quire` binary target is in `quire-cli`. |
| FR-014 | FR-014-AC-2 | TC-005, TC-028 | ✅ `tests/dependency_edges.rs` asserts the direct CLI edges and that the driver closure excludes the kit; `quire-cli/tests/package_boundary.rs` checks the direct kit edge. |
| FR-014 | FR-014-AC-3 | TC-005 | ✅ TC-005 asserts `quire-cli` workspace default-member status |
| FR-014 | FR-014-AC-4 | TC-009, TC-018, TC-028 | ✅ `quire-cli/tests/package_boundary.rs::parser_and_cli_tests_live_in_the_cli_package` asserts CLI test and fixture ownership. CLI behavior remains tracked separately by TC-009/TC-018. |
| FR-014 | FR-014-AC-5 | TC-028 | ✅ `quire-cli/tests/package_boundary.rs::parser_and_cli_tests_live_in_the_cli_package` asserts parser and binary entry point locations. |
| FR-014 | FR-014-AC-6 | TC-028 | 🚧 Partial: `quire-cli/src/cli.rs` parses one explicit `--provider-config <path>` and rejects missing or repeated options; the binary test covers a valid empty `providers` list, and production binary tests exercise configured dispatch, duplicate refusal and active-request SIGTERM cleanup. Strict invalid-file and deadline outcomes are covered under FR-006-AC-24. Omitted configuration bypasses the provider composition. |
| FR-015 | FR-015-AC-1..FR-015-AC-12 | TC-029 | 🚧 Planned: producer/read/lower/RT evidence in TC-029 has not executed. QSL-1631 source-owned witnesses, complete IR FR-357 projection and driver preparation remain open; the first witnessed IR subset does not complete a `TypeEnvironment` or RT `ProgramParts`. No criterion is accepted by this inventory row. |
| FR-016 | FR-016-AC-1..FR-016-AC-12 | TC-030 | 🚧 Planned for the one-budget accounting of FR-016-AC-10/11; locus-retention evidence as recorded in TC-030. |
| FR-017 | FR-017-AC-1..FR-017-AC-8 | TC-031 | 🚧 Planned: full lowering on the caller budget; scalar helper evidence as recorded in TC-031. |
| FR-018 | FR-018-AC-1..FR-018-AC-5 | TC-032 | 🚧 Evidence as recorded in TC-032. |
| FR-019 | FR-019-AC-1..FR-019-AC-10 | TC-033 | 🚧 Planned: one-budget preparation from a genuine plan root; producer criteria run in `quire-integration`. |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-001 | The steps run in ADR-011 T-13 order | Integration | FR-001-AC-4 | ✅ Tests present |
| TC-002 | An unknown BackendId refuses the run before negotiation | Integration | FR-001-AC-2 | ✅ Tests present |
| TC-003 | route runs after negotiation, for supported items only | Integration | FR-001-AC-3 | ✅ Tests present |
| TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5, FR-001-AC-22 | ✅ command-level refusal tests and focused CG reader-projection unit tests present |
| TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 | Integration | FR-001-AC-1, FR-001-AC-11, FR-012-AC-13, FR-013-AC-11, FR-014-AC-2, FR-014-AC-3 | ✅ Driver/cache edges, host-free and kit-free driver closure, direct CLI-to-host/kit edges and CLI default membership are asserted. |
| TC-006 | E5 admits the emitted package and CG generates for the routed items | Integration | FR-001-AC-6, FR-001-AC-7, FR-001-AC-8, FR-001-AC-12..FR-001-AC-16 | 🚧 Existing tests verify typed-form projection and two same-node occurrences’ distinct conversion roots. The production QSL Call/binder-to-CG composition, ordered operand sidecars, full request-position vector, and typed absence/mismatch cases remain planned under IR-879. This generation-path regression is distinct from TC-016 native receipt authentication and replay. |
| TC-007 | Generation refusals are typed | Integration | FR-001-AC-5, FR-001-AC-6 | ✅ Tests present |
| TC-008 | A missing import or an unknown header profile refuses at compile | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-009 | Exercise CLI verbs, outcome documents and exit codes | Integration | FR-002-AC-1..FR-002-AC-6, FR-002-AC-8..FR-002-AC-13, FR-014-AC-4 | 🚧 Partial: `quire-cli/tests/cli_contract.rs` checks QSL-equivalent machine documents for `compile`, `check`, `parse`, and `format`; text mode for parse/check/compile/run fails closed with typed unsupported results until QSL-601 supplies the renderer. Other verbs remain explicitly unsupported. The working-directory trap covers AC-5. Stream write failures, check-stage semantic refusal cases, and `ExecutionFailure` documents remain planned. No configuration reader exists; AC-10 remains unimplemented. Fixtures belong to `quire-cli`. |
| TC-010 | Apply explicit lifecycle limits and preserve limit outcomes | Integration | FR-003-AC-1..FR-003-AC-10 | ✅ QSL parser/application, stage-bound independence, unavailable-stage-setting refusal, unknown/malformed/duplicate refusals, typed stage-limit results and raised-bound completion are tested. QSL's `accounting.<counter>` setting names route through `AccountingLimits`, while the separate bare-name adverse case retains QSL's unknown-setting refusal; focused unit and process tests assert stage limits remain unchanged and QSL returns an incomplete accounting-limit outcome. |
| TC-011 | Select only the requested execution engine | Integration | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4, FR-004-AC-5, FR-004-AC-6, FR-004-AC-7, FR-004-AC-8, FR-004-AC-9, FR-004-AC-10 | 🚧 `src/execute.rs` verifies interpreter selection, unchanged environment-reference forwarding, direct-QSL parity for populated/empty object contexts, the QSL-defined absent-lookup `Undefined` result, unresolved-population refusal parity, unavailable-engine refusal and preparation-refusal parity. Callback-injected cases can verify exact `RunRefusal::UnsupportedResult` and typed preparation-refusal forwarding, but callback counts cannot detect a direct alternate invocation. Full AC-4 remains Partial until a second production engine or upstream-supported shared boundary makes every invocation observable; the future oracle must assert exact results, one selected invocation, zero alternative invocations, and fail if any engine call bypasses the boundary. |
| TC-012 | Register and invoke a first-party provider through Provider | Integration | FR-005-AC-1..FR-005-AC-11, FR-005-AC-16 | 🚧 `src/registry.rs` covers distinct lookup, Linked-origin FR-005-AC-1, identical-manifest idempotence without order-dependent selection, canonical linked-manifest bytes with absent optional members omitted, byte-for-byte retention of supplied wire, and refusal of empty advertisements; `quire-cli/src/backend_provider.rs` verifies the adapter refuses `wire: None` instead of reconstructing. `tests/drive.rs` tests the Linked/Process projection and caller ceilings in the generated harness identity; `src/provider.rs` covers cancellation, backend mismatch, missing limits/harness, execution-ceiling mismatch, typed evidence carriers and CG terminal mappings; `tests/drive.rs` exercises public `Provider::run`. Generic non-Kani carriers are tested; real Kani execution/result forwarding remains planned. Configured process-provider invocation is exercised under TC-013 and TC-028. |
| TC-013 | Dispatch process-provider results through the driver | Integration | FR-005-AC-12..FR-005-AC-19, FR-006-AC-1..FR-006-AC-2, FR-006-AC-5..FR-006-AC-23 | 🚧 Partial: the production fixture covers call-aware selected-item dispatch, typed result retention, request projection, hello admission/registry retention, QSL-owned domain admission/refusals, and generic result carriers. Evidence-kind admission/refusals and proof-bound `kind` emission await QSL-654; fairness, obligation-form and reduction values are enforced as FR-331 wire shape and used at negotiation. Settlement remains pending QSL result-mapping APIs. The 13 Kani rows from QSpec FR-331-AC-20, FR-272 diagnostics, and replay/proved settlement remain held. Replay-only claims remain unhandled until QSL-655 amends PL-7. |
| TC-014 | Exercise the cache-aware provider-call helper through a recording cache interface | Unit | FR-007-AC-1..FR-007-AC-6 | 🚧 Helper tests cover cache-call semantics AC-3..AC-5; admission-to-dispatch integration AC-1, AC-2 and AC-6 remains planned |
| TC-015 | Invoke AOT with current checked request inputs | Integration | FR-008-AC-1, FR-008-AC-2 | 🚧 Planned |
| TC-016 | Settle one terminal record per item after replay | Integration | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-6, FR-009-AC-7, FR-009-AC-8, FR-009-AC-9, FR-009-AC-10, FR-009-AC-11, FR-009-AC-12, FR-009-AC-13, FR-009-AC-17, FR-009-AC-18, FR-009-AC-19, FR-009-AC-20..FR-009-AC-24, FR-001-AC-23..FR-001-AC-24 | 🚧 QSL/IR binding APIs exist; TC-006/IR-879 owns the still-planned production generation composition. TC-016 specifies native receipt authentication, replay, typed conversion, terminal settlement, and frame-target projection; these remain planned under IR-773/IR-631/IR-610 and the QSL/CG API dependencies; function proof production also waits on IR-1121. Predicate/function reproduction, frame projection, and scalar-parity conversion remain distinct; scalar parity never settles `refuted`. |
| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, FR-010-AC-5, FR-001-AC-17..FR-001-AC-21, FR-009-AC-16 | 🚧 `tests/drive.rs` exercises the driver partition, typed refusal and index mapping; `quire-integration` PR 16 exercises QSL/driver/CG composition. The full specified terminal outcome remains open. |
| TC-018 | Preserve the QSL outcome at the binary process boundary | Integration | FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-6, FR-002-AC-8, FR-002-AC-9, FR-002-AC-14, FR-002-AC-15, FR-002-AC-16, FR-014-AC-4 | 🚧 Partial: binary-process tests cover the code-5 fallback, machine-outcome bytes, request/I/O refusals, stream diagnostics, the FIFO broken-pipe case on Unix, and the Linux `/dev/full` case. The four test-adapter cases (refused-range, profile-gated, cancelled and fault), full locus-rendered diagnostic comparison, Windows broken-pipe coverage, non-Linux `/dev/full` equivalent, and `ExecutionFailure` documents (QSL-653) remain planned. Fixtures belong to `quire-cli`. |
| TC-019 | Propagate library cancellation and retain bounded checked-unit source | Integration | FR-001-AC-9, FR-001-AC-10, FR-011-AC-1..FR-011-AC-5 | ✅ Cancellation/source-lifetime tests are present with explicit comma-separated Trace IDs for FR-001-AC-9, FR-001-AC-10, and FR-011-AC-1..FR-011-AC-4. |
| TC-020 | Derive cache identity from meaning inputs | Unit | FR-012-AC-1..FR-012-AC-5 | 🚧 Key, target, and excluded-field projection tests are present; `Trace:` tags identify covered criteria |
| TC-021 | Reject invalid cache entries and excluded values | Unit | FR-012-AC-6..FR-012-AC-8 | ✅ Quire matrix binds AC-6, AC-7, and AC-8 to `quire-cache/src/lib.rs:942`; typed invalid-entry readers also bind AC-6 |
| TC-022 | Keep cache writes atomic and within the caller's size limit | Unit | FR-012-AC-9..FR-012-AC-12 | ✅ Size, rename boundary and rename NotFound, temp cleanup, ownership-filtered eviction, concurrency and removal-race tests carry `Trace:` tags |
| TC-023 | Start a plugin session and admit hello before requests | Integration | FR-013-AC-1..FR-013-AC-3 | 🚧 Partial: executable-name refusal, hello admission, rejected protocol, and zero/one/two requests are tested; the explicit no-request-frame assertion remains planned. |
| TC-024 | Enforce plugin framing and preserve typed failures | Integration | FR-013-AC-4, FR-013-AC-7 | 🚧 Partial: invalid result, wrong request id, plugin fault, non-zero exit, truncated hello, non-canonical/oversized frames, and unexpected extra frame are tested; malformed JSON and truncated-body cases remain planned. |
| TC-025 | Preserve plugin settlement during cancellation and bound deadlines | Integration | FR-013-AC-5..FR-013-AC-6 | 🚧 Partial: cancellation settlement/fault, grace-expiry descendant cleanup within one second, and blocked-write deadline tests exist. |
| TC-026 | Bound plugin stderr and preserve cleanup | Integration | FR-013-AC-9..FR-013-AC-10 | 🚧 Partial: stderr truncation and EOF exit-status preservation, descendant cleanup and bounded reader joins are tested. |
| TC-027 | Run plugin-host tests and dependency checks from a clean cache | Integration | FR-013-AC-12, FR-013-AC-13 | 🚧 Python prerequisite message is tested; clean-cache fetch, offline metadata, and full gate evidence remain planned. |
| TC-028 | Keep the CLI package separate from the driver and compose cache and process host | Integration | FR-002-AC-3, FR-002-AC-7, FR-006-AC-1, FR-006-AC-3..FR-006-AC-4, FR-006-AC-24..FR-006-AC-27, FR-013-AC-11, FR-013-AC-14..FR-013-AC-15, FR-014-AC-1, FR-014-AC-2, FR-014-AC-4..FR-014-AC-6 | 🚧 Partial: `quire-cli` tests cover explicit-option parsing, the empty configuration, strict invalid-file/deadline outcomes, selected-provider composition with typed-result retention, production duplicate-identity refusal, and active-request SIGTERM cleanup. Later-start/hello failure cleanup remains untested. Package-boundary tests cover target ownership, layout and the direct CLI-to-host edge. Adapter lifecycle/signal tests are partial; AC-14 kill-on-close is deferred to IR-674 and has no implementation or test in this PR; it requires native Windows evidence when resumed. |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Verify cache calls carry eligible requests and preserve typed hits | `quire-driver` cache interface with a recording implementation | component | TC-014 |
| Verify content identity, typed reads, exclusion policy and bounded atomic storage | Downstream `quire-cache` package and caller-named temporary directory | component | TC-020, TC-021, TC-022 |
| Verify plugin sessions, protocol, typed outcomes, cancellation, deadlines, stderr and cleanup | `quire-plugin-host` with deterministic process fixtures | component | TC-023, TC-024, TC-025, TC-026, TC-027 |
| Verify CLI process-provider composition and cleanup on exit and handled signals | `quire-cli` adapter and `quire-plugin-host` with a descendant-spawning fixture | component | TC-028 |
| Verify admitted monitor program preparation and execution from real emitted entries | QSL producer, strict IR reader, driver preparation and RT program | integration | TC-029 (planned) |

The separate clock-admission component has narrow FR-015-AC-13
traces for native clause/role/ordinal and work-refusal payloads, plus
`MissingClock` payload classification. An admitted non-Timed no-clock driver
entry is still needed for the no-token `MissingClock` obligation.
The original partial selection controls retain their unclocked meaning.
Contract CG PR #749 removes the previous production compilation blocker.
Actual production component execution passes the quire-integration `monitor_preparation` and the driver
`monitor_lowering` suites. The clock-admission controls
exercise reached FR-015-AC-13 native refusal payloads, exact requested
occurrences and remaining allowance, plus original pre-cancellation with no
capability or mutation. This is component evidence, not a final gate or a
complete preparation result.
The authentic Timed PlanRoot now provides a native model-time clock receipt,
extra independent configured-row refusal and successful cumulative work
accounting under FR-015-AC-14. Configured-profile row comparison and full
criterion qualification remain Planned.

This component does not qualify FR-015-AC-1 through FR-015-AC-12 or whole
TC-029 execution. QSL PR #2395 supplies the final subject-occurrence owner
lookup; its driver consumer has separate FR-015-AC-15 controls. Admitted
order/progress association, authentic final source-union producer identities,
complete generated record closure and full Program preparation remain missing
facts. Contract IR PR #905 supplies the native union seam and Contract IR
PR #908 supplies the selected clock read. QSL PR #2385 supplies final
source-record/member rows and Contract IR PR #910 supplies native record
admission/projection; their driver join and complete closure qualification
remain separate work. The clock component retains the direct native refusal
when those available record rows are not joined.

The native final-subject API retains the genuine `MonitorSubject` and final
`Emission`, strictly authenticates the complete native package, and consumes
the original-key/ordinal owner declaration lookup. Native acquisition, full
package equality and the owner's unbounded linear lookup are outside the
driver ledger; declaration `len()` is not a work receipt. The driver prefix
is preserved through the mapped token's actual native clock continuation.
This component reads the healthy native Timed model-time source but supplies no
complete preparation qualification. Source-record driver joining and full
closure remain separate.

Production quire-integration `monitor_final_subject` executes eight FR-015-AC-15
controls: two-clause owner mapping and native model-time source reading;
another declaration that maps in the same emission; absent lookup and edited
node/role/name/package; full native package and instance mismatches; independent
clause/full occurrence/subject changes; native acquisition payloads/limits; and
original cancellation with exact/one-under driver prefix continuation. The
preparation controls also exercise native model-time admission. These receipts
qualify final-subject selection and its bounded native source read, not whole
TC-029 preparation.

The additional early-identity boundary control requires zero-work refusal
before wrong producer package/node/role/ordinal selection. Exact package-name
material and both native keys are charged before comparison. Native borrowed
key unit controls cover nested UTF-8 material, exact/one-under work and scratch
release on work, storage and cancellation stops. Both integration suites use
one shared native producer/request builder with explicit source shapes.
