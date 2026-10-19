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
for every requested item, including the routed scalar observation producer
and report-claim guard, remains planned.

Scope: FR-009-AC-1 through FR-009-AC-15.

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

When IR-773 adds the native producer after the production mapping exists,
send the retained `ReplayLimits` unchanged in the QSL replay request and execute
the artifact with the retained scalar inputs and `ScalarLimits`. Recompute QSL FR-357's artifact-only `sha256-jcs`
`DigestRecord` from every staged `(path, bytes)` entry exposed by the CG
receipt, then compare it with the receipt's attested proof-content identity.
Use the same record version, digest domain, and mapping defined by the merged
IR-846 contract for
both values; do not compare records until that mapping is available. Preserve
upstream typed completion refusals according to the merged IR-847 refusal
classification contract. Request/meter/returned `ScalarLimits` attestation is
owned by the CG API tracked in IR-857; until it is available, this comparison
and AC-7 remain planned.
Check driver-owned staged entries against retained originals; the opaque CG
receipt attests that CG-generated runner entries are part of the immutable
snapshot CG executed. The digest preimage excludes operands, limits, and the
native outcome, which is carried separately as the claim. The oracle compares
the retained and sent `ReplayLimits` for equality. It must reject missing or
mismatched evidence without invoking QSL or creating settlement. The
report-claim mismatch case remains planned until CG exports its FR-032 public
scalar binding converter: then pass every report outcome through that
converter, verify it compares `report.claim()` with the
exact claim sent, and assert that an absent report or mismatch returns CG's
typed binding refusal without settlement or terminal creation. The driver
surfaces that refusal unchanged and creates no local mapping. A
caller-supplied outcome/digest echo or regenerated artifact is not a valid
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

## Expected Results

- Available proved-provider outcomes are serialized by QSL's
  `OutcomeDocument`; the driver defines no JSON document shape.
- A falsified scalar without QSL replay does not produce a terminal document.
- For predicate/function counterexamples, `refuted` requires QSL replay
  reproduction; disagreement or completed non-reproduction is `inconclusive`
  with the replay-parity cause, a replay refusal keeps its typed cause, and a
  replay fault is an internal failure.
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
  per requested item, including unsupported and incomplete items. A
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
