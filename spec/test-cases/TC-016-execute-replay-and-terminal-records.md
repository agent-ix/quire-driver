---
id: TC-016
title: "Settle one terminal record per item after replay"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-009
    type: verifies
---
# TC-016: Settle one terminal record per item after replay

## Description

Verify the available proved-provider terminal path uses QSL's canonical
`OutcomeItem` and `OutcomeDocument` serializer. Full replay-driven settlement
for every item in the CG request, including the routed scalar observation producer
and report-claim guard, remains planned.

Scope: FR-009-AC-1 through FR-009-AC-24 and FR-009-AC-26 and FR-001-AC-23/24.

## Test Procedure

Run a verified Kani result through `ProviderResults::outcome_bytes` and compare
the bytes with QSL's `OutcomeDocument::settled` over
`OutcomeItem::from_terminal`. Verify that a falsified scalar without replay
remains a typed error and produces no guessed outcome.

For CG terminal forwarding, pass the original `KaniExecutionEvidence` and
optional replay settlement to `run_terminal_value`. Cover `NoVerdict` evidence
with and without diagnostics and `FailedWithoutCounterexample` evidence with
each actual optional failure cause state. Verify the typed result follows CG's
map, diagnostics do not change it, and the driver neither synthesizes nor
infers a cause. `src/provider.rs::cg_terminal_mapping_reads_original_failure_cause_not_diagnostics`
asserts that CG's retained `OutOfMemory` cause selects the terminal while
different or absent diagnostic text does not. The preserved-evidence test is
`src/provider.rs::provider_result_preserves_optional_cg_diagnostics_and_failure_cause`.
`src/provider.rs::falsified_scalar_does_not_guess_a_replay_settlement` verifies
the optional replay settlement is passed to CG's map and no settlement is
guessed when it is absent.

FR-009-AC-26: drive a general-route scalar (`x + 1` over `Int[0, 9]`) through
`drive` with `ScalarNativeInput` naming a nonexistent launcher. Verify the
result is `DriverRefusal::ScalarNotTractable` carrying CG's
`V2ClauseNotTractable` for the general route (a launch attempt would instead be
`ScalarNative`). Drive the same source through a generation that uses CG's
`generate_routed_native` and verify the paired runner is reached
(`ScalarNative`, the launcher is missing). `src/drive.rs::scalar_native_tractability_tests`.

The scalar producer/replay scenarios below remain planned. IR-879 owns the
production driver composition that binds a real QSL call to its exact selected
requirement occurrence, retains the ordered operands and context beside the CG
output, and supplies CG's existing admitted package and complete request
positions. TC-006 verifies that prerequisite through the production generation
path; it does not qualify native execution or a receipt. IR-773 owns the
same-artifact producer; IR-631 consumes its receipt for replay conversion, and
IR-610 owns later E9 integration. The producer does not perform replay
conversion, settlement, or terminal creation. Its later qualification must
retain the exact generated artifact/package, scalar identity/occurrence,
ordered operands, original `ScalarLimits` and `ReplayLimits`, and original QSL
claim/replay context as one execution context. No positional zip of QSL and CG
vectors, inferred occurrence, caller-authored operand list, or limits from a
different call is a valid positive case. Missing, ambiguous, or mismatched
call/occurrence binding must not yield an accepted receipt or reach QSL replay,
settlement, or terminal creation. The production composition and the producer
qualification remain planned until each owning slice is implemented.

The function proof producer and join scenarios remain planned until IR-1121
supplies CG’s current-V2 generated function harness and proving-run identity.
From one QSL source and dependency lock, select two function proof items that
name the same checked node at different application occurrences. Record each
original request index, QSL-selected function declaration and exact occurrence,
E5-admitted package identity, CG generated item and `KaniObligationIdentity`.
Invoke the selected provider once per matching item with a one-item
`ProviderRequest`, retaining a private invocation slot that binds its request
index and backend to the occurrence, admitted package and full CG identity.
Require exactly one `ProviderItemResult` with that index and backend. Compare
the successful `KaniExecutionEvidence` harness path, ceilings, contract kind,
symbolic argument bounds, solver and unwind with the selected generated harness
and identity; do not read occurrence, package or full identity from the provider
result, which carries none of them. Verify CG `ReplayInputs`
carry the original source, dependency lock, accounting and stage limits, declared
domains and replay limits; check playback intake uses the smaller caller/proving-run
byte and value decode limits, and playback/replay receives the same item’s package and
identity. Swap the two retained occurrences or identities while leaving the
node equal; assert a driver `runtime_invariant` fault before provider execution.
Remove one context, duplicate a request index, or mismatch the retained package,
declaration or generated identity; assert the same pre-execution fault. After a
valid invocation, return zero or two items, a wrong request index or backend,
or one successful evidence record with another item’s harness path or different
ceilings, kind, argument bounds, solver or unwind. Assert a driver
`runtime_invariant` fault before playback intake, no fallback selection, and
no settlement or terminal. This mutation changes actual provider-result fields
or the private invocation slot, never imaginary occurrence/package/identity
fields on `ProviderItemResult`. Independently make QSL selection, IR admission,
CG generation and the provider return their own typed refusals; assert each is
preserved unchanged at its owning stage, with no driver join fault substituted.
Do not mint an identity from caller text, a diagnostic or a Kani transcript.

The function playback intake scenarios remain planned until IR-1121 supplies
the current-V2 proof producer and IR-610 wires E9 production replay. CG PR #621
has merged the typed `Playback` entry. Drive a falsified function item through the production provider result
with its complete retained `KaniRunOutcome::Falsified.counterexample`, exact
selected proving-run identity and admitted package. Set finite, distinct
caller and proving-run byte and value ceilings; record the parser call and CG
function replay invocation. The oracle is one
`parse_assertion_playback` call over the exact retained transcript with the
selected item's retained `source_id` and context and the smaller ceiling in
each dimension, followed by one replay call receiving the same parsed
`Playback`, proving-run identity and package. A valid reproducing example
must reach FR-009-AC-2's replay/settlement path and CG's terminal mapping.

For adverse intake, add irrelevant text outside the sole assertion block so
the complete transcript exceeds the byte ceiling while the block fits; assert
CG's byte-limit `PlaybackRefusal` and no replay, settlement or terminal. At a
byte ceiling that admits the full text, exceed the value ceiling and assert
CG's value-limit refusal. Supply zero or multiple valid assertion blocks and
assert CG's corresponding typed refusal. In each case, check that optional
diagnostics cannot supply a replacement transcript or change the result.
First return a typed upstream refusal before intake and assert no parser call;
then parse a valid transcript with a mismatched proving-run identity or
package and assert CG's typed replay failure takes precedence over any
reproduction. For that failure and for a successful parse followed by a typed
CG decode or replay refusal, assert that the driver passes CG's corresponding
`ReplaySettlement` and the original execution evidence to
`run_terminal_value`, then preserves CG's typed terminal. Compare each result
with CG's direct conversion and terminal mapping; none is `refuted`, and the
driver does not recast these paths as parity or join faults. The parser's
retained full-source length and CG replay's independent limit and identity
checks are part of the oracle. These checks do not claim the current
generation-only `drive()` path performs replay.

When IR-773 adds the native producer after the production mapping exists,
send the retained `ReplayLimits` unchanged in the QSL replay request and execute
the artifact with the retained scalar inputs and `ScalarLimits`.
Preserve
upstream typed completion refusals according to the merged IR-847 refusal
classification contract. Request/meter/returned `ScalarLimits` attestation is
owned by the CG API tracked in IR-857; until it is available, this comparison
and AC-7 remain planned.
Check driver-owned staged entries against retained originals; the opaque CG
receipt attests that CG-generated runner entries are part of the immutable
snapshot CG executed. The native outcome is carried separately as the claim. The oracle compares
the retained and sent `ReplayLimits` for equality. It must reject missing or
mismatched evidence without invoking QSL or creating settlement. The
report-claim mismatch case remains planned until CG exports its FR-032 public
scalar binding converter: then pass every report outcome through that
converter, verify it compares `report.claim()` with the
exact claim sent, and assert that an absent report or mismatch returns CG's
typed binding refusal without settlement or terminal creation. The driver
surfaces that refusal unchanged and creates no local mapping. A
caller-supplied outcome echo or regenerated artifact is not a valid
positive case.

For the scalar-limit provenance mismatch case, retain original limits `L0`,
then deliberately send the native request with different limits `L1`. Use a
receipt whose CG attestation says the request supplied `L1`, the meter used
`L1`, and the receipt returns `L1`; this is a valid receipt for the request,
not a fabricated or mutated receipt. Verify that the driver compares `L1` with
its private `L0`, rejects the completion, and does not invoke QSL replay,
settle an item, or create a terminal. For this successful receipt, assert
`DriverRefusal::Fault(DriverFault::NativeExecutionLimitsMismatch)`, stage
`DriverStage::Replay`, code `runtime_invariant`, and outcome kind `Fault`. Also
verify that an upstream typed completion refusal, when actually returned, is
preserved unchanged. The successful-receipt mismatch test remains planned
until the driver implements this contract; it must not manufacture an
upstream refusal.

The production QSL-call-to-CG-generation prerequisite is covered by TC-006.
Test location for Kani request/evidence forwarding and terminal mapping:
`src/provider.rs`. Test location for native receipt and settlement qualification: `src/provider.rs`;
full prove/replay integration remains planned.

For the frame-target projection scenario, call production
`Driven::prepare_frame_replay` with the real QSL producer and public CG
consumer. Reject a prepopulated target, missing request item, changed package,
index and complete occurrence (including node, role and ordinal) before CG
construction. Compare the prepared CG packet and request targets and assert
that request arguments, request occurrence key and function target are absent,
and the original request/routed inventory is unchanged. Use object-type operation fixtures
covering an inherited operation and a redefined operation. Obtain the
`OperationSite.frame_operation` values from QSL's validated call-site result
and compare each value unchanged with the target placed on the CG selected item
joined by the same admitted package identity and request index. QSL FR-121 is
the oracle for how the declaring object is resolved; this driver check asserts
only lossless projection. QSpec FR-323 is the oracle for packet/request shape.
Do not infer the target from CG function selection or caller spelling, and do
not add a routed item.

For the negative partition, first make QSL call-site selection return its typed
refusal and assert the driver returns that same refusal before CG frame-target
construction or replay. Separately, provide a successful QSL result whose driver-retained association
(package identity or request index) has a value that differs from the
corresponding value on the CG selected item;
assert the driver returns FR-001's typed `runtime_invariant` fault before CG
frame-target construction or replay. A typed refusal returned by CG itself is
propagated unchanged and is not treated as a driver join mismatch. Both cases
produce no settlement or terminal. `tests/frame_replay.rs` verifies these
preparation scenarios with real QSL compilation and CG-generated frame
harness identities. Full frame settlement remains outside this check.

## Expected Results

- For a frame counterexample, the CG selected item's target equals the exact
  `OperationSite.frame_operation` value returned by QSL for the same admitted
  package identity and request index. QSL FR-121 defines the declaring-owner
  resolution and QSpec FR-323 defines the packet/request shape; this driver
  projection adds no target inference or routed item.
- A typed QSL call-site refusal is preserved unchanged. A successful QSL result
  that the driver detects cannot be joined to the CG item by equal package
  identity and request index returns FR-001's typed `runtime_invariant` fault.
  A CG-returned typed refusal remains under CG's refusal contract. Neither path
  proceeds to replay, settlement, or terminal creation.
- Available proved-provider outcomes are serialized by QSL's
  `OutcomeDocument`; the driver defines no JSON document shape.
- A falsified scalar without QSL replay does not produce a terminal document.
- For predicate/function counterexamples, `refuted` requires QSL replay
  reproduction; disagreement or completed non-reproduction is `inconclusive`
  with the replay-parity cause, a replay refusal keeps its typed cause, and a
  replay fault is an internal failure.
- A falsified function result reaches replay only after one bounded CG parse
  of its complete retained transcript, with the exact selected identity and
  package. A typed upstream or CG parser refusal before replay produces no
  replay settlement or terminal for the refused item. A CG replay refusal or
  evidence failure after successful parsing follows CG's typed settlement and
  terminal mapping with the original execution evidence; its terminal is not
  `refuted`.
- Scalar operator-parity reports preserve QSL FR-357's typed `Agrees`,
  `Diverged`, `GeneratedFault`, `RefusedInput`, `Incomplete`, or refusal
  outcome through CG FR-032's typed settlement conversion. No scalar
  operator-parity report settles as `refuted`.
- CG's mapping is used without a second driver mapping.
- The driver's terminal call forwards the original CG evidence and optional
  replay settlement; a failure cause is read only from that evidence. Optional
  diagnostics are retained in the typed evidence but never classify or map a
  terminal.
- A request that reaches per-item settlement has exactly one terminal record
  per item in the CG request, including unsupported and incomplete items. A
  request-level typed refusal before settlement has no per-item terminal
  record guarantee.
- Missing or mismatched retained context or authentic native-observation
  evidence prevents QSL replay evaluation, settlement, and terminal creation.
  Preserve an upstream typed completion refusal unchanged only when the
  upstream operation actually returns one.
- When a successful CG receipt consistently attests request, metered, and
  returned native limits `L1`, but the driver's privately retained original
  limits are `L0` and differ from `L1`, the driver rejects the completion
  before QSL replay, settlement, or terminal creation. It returns
  `DriverRefusal::Fault(DriverFault::NativeExecutionLimitsMismatch)` at
  `DriverStage::Replay`, with code `runtime_invariant` and outcome kind `Fault`;
  it does not manufacture or attribute this driver-owned fault to an upstream
  typed refusal.
- After the public CG FR-032 binding converter exists, an absent replay report
  or a report whose `claim()` differs from the exact claim sent returns CG's
  typed binding refusal without replay settlement or terminal creation; the
  driver surfaces that refusal unchanged, with no local mapping. Until that
  API exists, this case remains planned and the driver has no refusal path to
  claim.

- A provider `ToolFailed` value retains its exact typed tool failure, including
  configured and observed byte counts, request index, and backend identity in
  QSL's terminal record (FR-009-AC-25).
