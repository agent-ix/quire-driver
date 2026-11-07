---
id: FR-009
title: "Settle executions after replaying counterexamples"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-contract-codegen/FR-017
    type: depends_on
  - target: ix://agent-ix/quire-contract-codegen/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-323
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-121
    type: depends_on
  - target: ix://agent-ix/quire-contract-codegen/FR-024
    type: depends_on
---
# FR-009: Settle executions after replaying counterexamples

## Description

When the caller requests a proof, the driver SHALL run the generated provider
request and replay each counterexample through QSL's replay operation. For each
proof request that reaches per-item settlement, the driver SHALL produce
exactly one FR-331 terminal record for each item in the CG request. A request-level
typed refusal before per-item settlement does not receive a per-item
terminal-record guarantee. For an item the export refused, the driver SHALL write no terminal record
and SHALL keep its refusal in FR-001's per-item refusal member. The driver SHALL preserve the independent result
and cause from execution and replay when settling an item. For routed scalar
replay, the driver SHALL authenticate the native observation against the exact
generated artifact that was proved and SHALL preserve the original claim and
replay context through QSL evaluation and report handling.

## Inputs

- The checked and emitted package from the current request.
- The request items, provider results, generated artifacts and cancellation
  handle.
- The QSL replay result for each provider counterexample.
- For a selected function proof item, the one driver-retained E9 context
  described by FR-001-AC-23: request index, exact QSL function declaration and
  application occurrence, admitted package identity, source and dependency lock,
  CG generated item and `KaniObligationIdentity`, proving-run accounting and
  stage limits, declared domains, replay limits, and caller/proving-run decode
  limits.
- For a falsified function item, the complete `counterexample` transcript retained
  in its original CG `KaniExecutionEvidence`, the same selected proving-run
  identity and admitted package, and the applicable caller and proving-run
  witness-decode byte and value limits.
- For an operation-frame replay, QSL's validated typed `frame_operation`
  selection containing the declaring object type and operation identifier,
  paired with the selected CG item by the same admitted package and request
  index.
- For a routed scalar replay, one retained context containing the exact
  generated artifact and emitted package, scalar identity and occurrence,
  ordered operands, original `ScalarLimits` and `ReplayLimits`, and the
  original QSL claim and replay context.
- For a routed scalar native execution, the selected `ExecutionRequest.call`,
  the exact QSL requirement occurrence and checked scalar application, and a
  typed mapping that binds that call's named arguments to the CG scalar
  identity's ordered arguments. The original scalar limits are the accounting
  limits on that same call.
- The actual typed native outcome produced by executing that retained artifact
  for the retained scalar inputs, and QSL's report for the original claim.

## Outputs

- One FR-331 terminal record per item in the CG request, written in the
  `quire-outcome/1` outcome document, when the request reaches per-item
  settlement.
- For each export-refused item, no terminal record; its refusal stays in
  `Driven`'s per-item refusal member (FR-001), and it does not appear in the
  `quire-outcome/1` document that holds the records of the items that settled.
- The upstream typed refusal without a per-item terminal record when the
  request is refused before per-item settlement.

## Behavior

- The driver SHALL pass the generated request and its exact emitted package
  to the selected provider.
- For routed scalar native execution, the driver SHALL obtain the concrete
  operand vector through a typed mapping that binds one selected
  `ExecutionRequest.call` and its named `CallArgument` values to one exact QSL
  requirement occurrence and checked scalar application, and orders the
  resulting values according to the corresponding CG
  `ScalarObligationIdentity.arguments`. The driver SHALL retain that call,
  occurrence, application and mapping as one context. It SHALL use the
  `ScalarLimits` from that same call's accounting as the original `ScalarLimits`
  for the context. It SHALL NOT pair QSL call arguments with CG arguments by
  vector position, infer an occurrence from an expression node alone, or
  substitute limits from another call or context. A mapping or binding that is
  absent or ambiguous is missing retained context under FR-009-AC-7; it cannot
  produce an accepted native receipt or proceed to QSL replay evaluation,
  settlement, or terminal creation. QSL's operand binder and the IR declared-
  name accessor are available; the driver's production composition boundary
  that supplies the same call and selected occurrence to that mapping remains
  planned under FR-001-AC-13 through FR-001-AC-16 and IR-879.
- The driver SHALL preserve the provider's typed result.
- For a Kani result, the driver SHALL pass the same CG
  `KaniExecutionEvidence` and optional replay settlement to CG's
  `run_terminal_value`. The driver SHALL NOT synthesize a
  `KaniFailureCause`, infer one from diagnostics or other text, or alter the
  evidence before this call. CG's function reads the outcome, success-check
  count and optional failure cause from that evidence.
- When executing a selected function proof item, the driver SHALL use its
  FR-001-AC-23 retained context to invoke the selected provider with a request
  containing only that matching generated item. The driver SHALL retain the
  invocation-to-result association privately with its request index, backend,
  occurrence, admitted package and full CG proving-run identity. It SHALL
  require exactly one returned `ProviderItemResult` with the same request index
  and backend, then compare the successful `KaniExecutionEvidence` fields that
  identify the invoked harness and run (including harness path, proof ceilings,
  contract kind, symbolic argument bounds, solver and unwind) with the selected
  generated harness and its identity. The driver SHALL NOT assume that the
  provider result carries an occurrence, package identity or full
  `KaniObligationIdentity`; those come only from the retained invocation
  context. It SHALL NOT accept execution evidence from another item or use a
  fallback generated harness. When the provider returns a typed refusal, the
  driver SHALL preserve it unchanged. When the provider succeeds but its
  observable result fields or retained association mismatch, the driver SHALL
  return a `runtime_invariant` fault before playback intake.
- When function replay is requested for a falsified item, the driver SHALL call CG's
  `parse_assertion_playback` exactly once on the complete retained
  `KaniRunOutcome::Falsified.counterexample` transcript, with the selected
  item's retained source identity and context and the smaller applicable caller
  and proving-run ceiling in each of `max_bytes` and `max_values`.
- The driver SHALL pass the resulting typed `Playback` to CG's function replay
  entry with the exact proving-run identity and admitted package for that
  selected item.
- The driver SHALL NOT parse optional diagnostics, substitute a selected
  assertion block or truncated transcript, reparse the playback, or rebuild
  the identity or package from caller text. CG's parser owns unique-assertion selection and
  full-transcript length accounting; CG replay independently checks the parsed
  playback against the proving-run limits and identity.
- When a typed upstream refusal occurs before function playback intake, the
  driver SHALL preserve it before parsing, without replay settlement or a
  terminal record for that item.
- If the bounded parser refuses the transcript, the driver SHALL preserve its
  typed `PlaybackRefusal` without invoking function replay or creating replay
  settlement or a terminal record for that item.
- If parsing succeeds but CG function replay returns a typed refusal or
  evidence failure, the driver SHALL pass its typed CG `ReplaySettlement`
  conversion with the original execution evidence to `run_terminal_value` and
  preserve CG's resulting terminal under the governing contracts without
  treating the result as reproduction.
- The driver SHALL NOT infer an intake refusal from optional diagnostics.
- For predicate/function counterexamples, when QSL replay reproduces the
  counterexample, the driver SHALL settle that item as `refuted`.
- For predicate/function counterexamples, when replay disagrees or completes
  without reproducing the counterexample, the driver SHALL settle that item as
  `inconclusive` with the corresponding replay-parity cause.
- The driver SHALL preserve QSL FR-357's typed scalar operator-parity report
  and apply CG FR-032's typed settlement conversion; `Agrees`, `Diverged`,
  `GeneratedFault`, `RefusedInput`, `Incomplete`, and other typed refusals
  retain the meanings those contracts define.
- The driver SHALL NOT settle a scalar operator-parity result as `refuted`.
- For an operation-frame counterexample, the driver SHALL project the exact
  QSL `OperationSite.frame_operation` value to the CG selected item associated
  with the same admitted package identity and request index. It SHALL preserve
  the typed pair unchanged. QSL FR-121 owns the pair's shape and declaring
  object resolution; QSpec FR-323 owns the replay-target and packet/request
  shape. The driver does not derive the pair from CG's function selection or
  caller-supplied operation spelling.
- The production `Driven::prepare_frame_replay` entry SHALL use the existing
  CG `FrameReplayInputs` envelope and the original `Driven` request inventory.
  The caller SHALL leave `selected_target` absent; a supplied target is a
  driver `runtime_invariant` fault. The driver SHALL select QSL's operation
  using the envelope's source, domain documents and dependency selections,
  preserve a call-site refusal unchanged, then compare the emitted bytes with
  the original admitted run. It SHALL authenticate the selected package,
  request index and complete frame occurrence against that run's retained
  request item before constructing CG's target. Missing items and package,
  index, node, role or ordinal mismatches are driver `runtime_invariant`
  faults. It SHALL populate the target only from the successful QSL site and
  invoke public CG `FrameReplay::new`, preserving CG errors unchanged. The
  returned preparation permits CG replay; it does not create settlement or
  terminal records and does not add a request or routed item.
- The driver SHALL NOT infer a function target, select a different package or
  request index, or add another routed item for an operation-frame replay.
- When QSL call-site selection returns a typed refusal, the driver SHALL
  preserve that refusal unchanged and stop before CG frame-target construction,
  replay, settlement, or terminal creation. It SHALL NOT recast that upstream
  refusal as a driver join fault.
- When QSL call-site selection succeeds but the driver's retained association
  between that result and its CG selected item contains a package identity or
  request index that differs from the corresponding value on the CG item,
  the driver SHALL return FR-001's typed `runtime_invariant` fault and stop
  before CG frame-target construction, replay, settlement, or terminal
  creation. A typed refusal actually returned by CG is preserved under its
  owning contract; it is not the driver's join-fault condition.
- This driver mapping covers object-type operations selected through QSL's
  current call-site API. Process/state-machine operations are not selectable
  through that API. State-clause target projection is outside this requirement
  pending QSL-973 and its applicable CG contract.
- When QSL replay refuses, the driver SHALL preserve its typed replay cause.
- When QSL replay faults, the driver SHALL settle the item as a failed
  internal result and SHALL NOT report a reproduced counterexample.
- For a routed scalar replay, the driver SHALL retain one context that binds
  the exact generated artifact and emitted package to the scalar identity,
  occurrence, ordered operands, original `ScalarLimits` and `ReplayLimits`,
  and the exact claim and replay context sent to QSL. It SHALL send that same
  retained `ReplayLimits` in the QSL replay request. It SHALL NOT replace any
  member with a caller-supplied classification or a regenerated artifact.
- The driver SHALL execute that same retained artifact with the retained
  scalar inputs and original `ScalarLimits`.
- The driver SHALL retain its actual typed native outcome as a separate claim
  field.
- The opaque CG receipt SHALL bind the exposed entries to the immutable
  snapshot CG executed.
- The driver SHALL compare driver-owned staged entries with its retained
  originals.
- For CG-generated staged entries, the driver SHALL rely on the opaque CG
  receipt's attestation.
- The driver SHALL carry the native outcome separately as the claim.
- The driver SHALL keep retained `ReplayLimits` bound to the exact replay
  request context; they do not change native execution limits.
- When the driver receives a native receipt, the driver SHALL compare the
  receipt's attested `ScalarLimits` with the original `ScalarLimits` in its
  private retained context. The receipt's attestation covers the limit supplied
  to the native request, used by the meter, and returned by the receipt, as
  required by the CG producer contract.
- The driver SHALL return
  `DriverRefusal::Fault(DriverFault::NativeExecutionLimitsMismatch)` at the
  new `DriverStage::Replay` stage, with code `runtime_invariant` and outcome
  kind `Fault`, when a successful CG receipt consistently attests request,
  metered, and returned limits `L1` but the private retained context contains
  different original limits `L0`.
- The driver SHALL stop before QSL replay evaluation when required retained
  context or an authentic observation is absent or mismatched.
- The driver SHALL create no replay settlement or terminal record for missing
  or mismatched retained context or observation.
- When an upstream operation returns a typed completion refusal, the
  driver SHALL preserve that refusal unchanged.
- For scalar parity, the driver SHALL pass each QSL report and its exact sent
  claim through CG's binding converter; that converter compares
  `report.claim()` for every outcome before accepting the report for settlement.
- If the report is absent or its claim differs, CG FR-032's converter returns
  a typed binding refusal with no settlement. The current CG main exports no
  public scalar binding converter, so this path remains planned until that API
  exists.
- The driver SHALL surface CG's typed binding refusal unchanged.
- The driver SHALL NOT create a terminal record for that refused report.
- The driver SHALL NOT add a local refusal mapping.
- CG's FR-331 terminal mapping SHALL combine the Kani outcome and replay
  result.
- The driver SHALL NOT define a second terminal mapping.
- When the authoritative terminal value is `ToolFailed`, the driver SHALL
  construct QSL's tool-failed terminal record, preserving the typed failure
  (including byte-limit counts), request index, and backend identity.
- When a proof request reaches per-item settlement, the driver SHALL write
  one terminal record per item in the CG request, including unsupported and incomplete
  items.
- For a request that reaches per-item settlement, the driver SHALL NOT
  duplicate or omit an item record.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-009-AC-1 | The proof provider receives the generated request and the exact checked-package bytes emitted by the same request. | Test |
| FR-009-AC-2 | Predicate/function replay that reproduces the provider counterexample settles as `refuted`. | Test |
| FR-009-AC-3 | A replay refusal retains its typed cause and a replay fault is an internal failure; neither is converted to `refuted`. | Test |
| FR-009-AC-4 | The terminal result is composed by CG's FR-331 mapping from both the provider result and replay result; the driver introduces no second mapping. | Partial (TC-016 proves QSL serialization for an available CG terminal; replay-combined settlement remains planned) |
| FR-009-AC-5 | When a proof request reaches per-item settlement, the outcome contains exactly one FR-331 terminal record for every item in the CG request, including unsupported and incomplete items; a request-level typed refusal before per-item settlement is governed by its refusal contract and has no per-item terminal-record guarantee. | Partial (TC-016 serializes available provider terminals through QSL; full per-item replay accounting remains planned) |
| FR-009-AC-6 | For routed scalar replay, the driver retains one context binding the exact proved generated artifact and emitted package to the scalar identity, occurrence, ordered operands, original `ScalarLimits` and `ReplayLimits`, and exact QSL claim and replay context; it sends the retained `ReplayLimits` in that QSL request, executes the artifact with the retained scalar inputs and `ScalarLimits`, and retains the actual typed native outcome. | Planned (TC-016; IR-773 producer; production call-to-generation boundary FR-001-AC-13 through FR-001-AC-16 / IR-879) |
| FR-009-AC-7 | The actual typed native outcome is carried separately as the claim. Before QSL evaluates it, the driver relies on the following checks. The opaque CG receipt binds the exposed entries, including CG-generated runner files, to the immutable snapshot CG executed; the driver checks its driver-owned entries against retained originals. The driver also compares the receipt's attested request/meter/returned `ScalarLimits` with its privately retained original limits, using the CG meter-limits attestation contract owned by IR-857. Missing or mismatched context, artifact identity, or observation prevents QSL replay evaluation, settlement, and terminal creation. An upstream typed completion refusal is preserved unchanged under the refusal classification contract specified by IR-847. | Planned (TC-016; IR-773 producer; production call-to-generation boundary FR-001-AC-13 through FR-001-AC-16 / IR-879; IR-847 refusal classification, IR-857 request/meter/returned `ScalarLimits` attestation, IR-631 replay-conversion consumer, IR-610 E9 integration) |
| FR-009-AC-8 | For scalar parity, the CG-owned binding converter compares `report.claim()` with the exact claim sent for every report outcome; an absent report or mismatch yields CG FR-032's typed binding refusal with no settlement, which the driver surfaces unchanged and never turns into a terminal. This path remains planned until CG exports the public scalar binding converter; the driver adds no refusal mapping. | Planned (TC-016; IR-610 consumer slice and public CG converter API) |
| FR-009-AC-9 | Each scalar operator-parity report outcome is converted only through CG FR-032's typed settlement conversion of the QSL FR-357 report; `Agrees`, `Diverged`, `GeneratedFault`, `RefusedInput`, `Incomplete`, and other typed refusal outcomes retain the meanings those contracts define. | Planned (TC-016; IR-610 consumer slice and reviewed CG API) |
| FR-009-AC-10 | Predicate/function replay that disagrees or completes without reproducing a provider counterexample settles as `inconclusive` with its replay-parity cause. | Planned (TC-016; full replay path) |
| FR-009-AC-11 | No scalar operator-parity report settles as `refuted`. | Planned (TC-016; IR-610 consumer slice and reviewed CG API) |
| FR-009-AC-12 | If CG returns a successful native-execution receipt that consistently attests request, metered, and returned `ScalarLimits` `L1` while the private retained context contains different original limits `L0`, the driver returns `DriverRefusal::Fault(DriverFault::NativeExecutionLimitsMismatch)` at the new `DriverStage::Replay` stage, with code `runtime_invariant` and outcome kind `Fault`, before QSL replay evaluation; no replay evaluation, settlement, or terminal record is created. An upstream typed completion refusal, when returned, is preserved unchanged. | Planned (TC-016; IR-773 S6b producer) |
| FR-009-AC-13 | For routed scalar native execution, the driver obtains concrete operands from a typed mapping of one selected `ExecutionRequest.call` to one exact QSL requirement occurrence and checked scalar application, with values ordered by the matching CG `ScalarObligationIdentity.arguments`; the original `ScalarLimits` come from that same call's accounting. Positional zipping, expression-node-only occurrence matching, and limits from another call are not valid mappings. If the mapping or exact binding is absent or ambiguous, it is missing retained context under FR-009-AC-7 and cannot produce an accepted native receipt or proceed to QSL replay evaluation, settlement, or terminal creation. | Planned (TC-016; IR-773 producer; production call-to-generation boundary FR-001-AC-13 through FR-001-AC-16 / IR-879; QSL FR-367 and IR-832 APIs are available) |
| FR-009-AC-14 | For a Kani result, the driver passes the original CG `KaniExecutionEvidence` and optional replay settlement to CG `run_terminal_value`; it preserves the resulting typed terminal without adding a driver mapping. The map receives the evidence's actual optional `failure_cause`; the driver invents or infers no cause. | Test (`src/provider.rs::falsified_scalar_does_not_guess_a_replay_settlement`, `src/provider.rs::cg_terminal_mapping_reads_original_failure_cause_not_diagnostics`; TC-016) |
| FR-009-AC-15 | Changing or omitting optional CG diagnostics while holding the outcome, success-check count, failure cause, and replay settlement fixed does not change the terminal produced by CG `run_terminal_value`; the driver does not parse diagnostic content to classify or map a result. | Test (`src/provider.rs::cg_terminal_mapping_reads_original_failure_cause_not_diagnostics`; TC-016; CG FR-017-AC-36 and FR-029) |
| FR-009-AC-16 | For the FR-010-AC-4 vector and a proof request that reaches per-item settlement, the outcome holds one terminal record for each of A and C and none for B, whose refusal appears only in `Driven`'s per-item refusal member. | Test (TC-017) |
| FR-009-AC-17 | For an operation-frame replay selected from an admitted package, the driver projects QSL's exact typed `OperationSite.frame_operation` value to the CG selected item associated with the same admitted package identity and request index, without changing the pair or deriving it from CG selection or caller spelling. QSL FR-121 owns pair shape and declaring-object resolution; QSpec FR-323 owns replay-target and packet/request shape. This driver criterion covers only object-type operations exposed by QSL's current call-site API; process/state-machine operations and state-clause projection are outside scope. | Test (`tests/frame_replay.rs::frame_projection_preserves_inherited_and_redefined_declaring_owner`; TC-016; bounded preparation only) |
| FR-009-AC-18 | When QSL call-site selection returns a typed refusal for a frame operation, the driver preserves that refusal unchanged and stops before CG frame-target construction, replay, settlement, or terminal creation; it does not recast the refusal as a driver join fault. | Test (`tests/frame_replay.rs::frame_projection_preserves_qsl_refusal_before_any_join_or_codegen_check`; TC-016) |
| FR-009-AC-19 | When QSL call-site selection succeeds but the driver detects that its retained association of the QSL result to the CG selected item is missing or has a package identity, request index or complete occurrence that differs from the original run or the CG item, the driver returns FR-001's typed `runtime_invariant` fault and stops before CG frame-target construction, replay, settlement, or terminal creation. A typed refusal actually returned by CG is preserved under CG's contract and is not this driver-detected join fault. | Test (`tests/frame_replay.rs::frame_projection_rejects_missing_changed_and_caller_populated_associations`, `public_reporting_inventory_cannot_rebind_retained_frame_request`, `frame_projection_preserves_native_codegen_failure_and_admitted_package_equality`; TC-016; FR-001) |
| FR-009-AC-20 | For a falsified function item with a complete valid transcript at the byte and value ceilings, the driver calls CG `parse_assertion_playback` once on exactly the retained `KaniRunOutcome::Falsified.counterexample`, with the selected item's retained source identity and context and the smaller caller/proving-run ceilings in each dimension. It passes the resulting typed `Playback`, exact proving-run identity and same admitted package to CG function replay. A reproducing replay remains subject to FR-009-AC-2 and CG terminal mapping. | Planned (TC-016; CG #621 typed function replay API merged; IR-610 E9 integration) |
| FR-009-AC-21 | A full transcript that exceeds the byte ceiling, has more values than the value ceiling, or has no unique valid assertion yields CG's corresponding typed `PlaybackRefusal` from that single parse, even if a shortened assertion block would fit. The driver neither uses optional diagnostics nor invokes function replay, creates settlement, or creates a terminal for the refused item. | Planned (TC-016; CG #621 typed function replay API merged; IR-610 E9 integration) |
| FR-009-AC-22 | A typed upstream refusal before playback intake is preserved before any parse, settlement, or terminal creation. A parser refusal is preserved without replay settlement or terminal creation and is not recast as replay parity or a driver join fault. After a successful parse, a CG typed identity, decode, or replay refusal or evidence failure follows CG's `ReplaySettlement` conversion and `run_terminal_value` mapping with the original execution evidence; the driver preserves the resulting typed terminal and never settles that result as `refuted`. | Planned (TC-016; CG #621 typed function replay API merged; IR-610 E9 integration) |
| FR-009-AC-23 | For two function proof items sharing a checked node but naming distinct exact application occurrences, the driver invokes the selected provider once per matching generated item, retains each typed result with its own request index and CG `KaniObligationIdentity`, and supplies the corresponding retained source, dependency lock, admitted package, accounting/stage/domain/replay inputs and smaller caller/proving-run decode limits to function playback and replay. Neither result or context is selected by node alone. | Planned (TC-016; IR-610 E9 function producer; IR-1121 CG current-V2 function proof producer) |
| FR-009-AC-24 | The driver invokes the selected provider with exactly one generated function item and privately retains that invocation’s request index, backend, occurrence, admitted package and full CG proving-run identity. A typed provider refusal is preserved before playback intake. A returned item count other than one, a returned request index or backend different from the invocation, or successful `KaniExecutionEvidence` whose harness path, proof ceilings, contract kind, symbolic argument bounds, solver or unwind differ from the selected generated harness and identity returns a driver `runtime_invariant` fault before playback intake, replay, settlement or terminal creation. The provider result is not assumed to carry occurrence, package or full identity; those are read from the retained invocation context, never another item’s context. | Planned (TC-016; IR-610 E9 function producer; IR-1121 CG current-V2 function proof producer) |
| FR-009-AC-25 | A `ToolFailed` terminal value becomes QSL's tool-failed terminal record with the same typed failure, byte-limit counts when present, request index, and backend identity. | Test (TC-016) |

## Dependencies

- IR-1121 must first supply a current-V2 generated function harness and typed proving-run `KaniObligationIdentity`; until then the driver cannot execute or join an authentic function proof item.
- CG #621 / IR-1085 provides the merged typed `Playback` function replay entry
  and bounded `parse_assertion_playback` API. Driver integration in
  FR-009-AC-20 through FR-009-AC-24 remains planned under IR-610 E9 and
  IR-1121; no production function replay path is claimed here.
- QSL FR-323 owns the three replay-target alternatives and the frame-request
  omission of arguments and occurrence key; QSL FR-121 / QSL #1215 owns the
  validated typed operation call-site result and its declaring object for
  inherited and redefined object-type operations. CG FR-024 / IR-958 owns the
  selected-item packet projection and its typed failure behavior. The public
  CG selected-item and selected-target input is available.
  FR-009-AC-17 through FR-009-AC-19 are implemented through the bounded
  production preparation entry; full frame settlement remains outside it.
  QSL-973 owns the separate state-clause target API.
- QSL ADR-029 LC-1, CB-4 and library operation `prove`; ADR-013 O-16 and
  C-09; QSL report-claim contract; CG FR-032
  native-observation and scalar-replay binding contract; CG FR-017 owns Kani
  execution evidence and FR-029 owns its terminal mapping. QSL FR-367 and
  IR-832 own the named `CallArgument`-to-selected-
  scalar-operand binding and typed declared-name accessor; FR-001-AC-13 through
  FR-001-AC-16 specify the driver's production composition that supplies the
  same call and exact occurrence to them. IR-773 owns the planned
  same-artifact native-observation producer; IR-631 consumes its receipt for
  replay conversion; IR-610 owns the later E9 integration. These criteria
  remain planned until the production boundary and required reviewed CG APIs
  are implemented. The producer does not create replay settlement or terminal
  records.
- [TC-016](../test-cases/TC-016-execute-replay-and-terminal-records.md).
