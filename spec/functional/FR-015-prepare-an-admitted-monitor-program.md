---
id: FR-015
title: "Prepare one admitted monitor program for the runtime"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-353
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: depends_on
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-251
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-487
    type: depends_on
---
# FR-015: Prepare one admitted monitor program for the runtime

## Description

Planned until IR-721 lands. Given an independently selected, strictly admitted
monitor plan, the driver SHALL lower its complete ordinary executable closure
once into the runtime-owned immutable program. The driver SHALL preserve the
admitted selection, native identities, types, provenance and exact premises.
Preparation SHALL execute no initializer or transition.

QSL-1630 and QSL-1631 supply body-sensitive plan identity and source enum
rows. Contract IR PR #905 delivered the IR-1323 native union
admission/projection seam. QSL PR #2385 supplies final source-record/member
rows and Contract IR PR #910 supplies native source-record admission and
projection. The clock component does not join those record rows or establish
complete generated record closure. The authentic final source-union handoff
(QSL-1723), complete closure and runtime lowering remain prerequisites for
healthy complete preparation. Native clock refusal evidence does not qualify
a prepared program.

This library boundary is downstream of QSL production and IR admission. It
implements the consumer handoff of [IR FR-038](ix://agent-ix/quire-contract-ir/FR-038)
and [RT FR-276](ix://agent-ix/quire-contract-runtime/FR-276), rather than another
temporal checker or a command-line monitor feature. Its implementation and
producer-to-runtime qualification are not delivered by this specification.

## Inputs

The driver SHALL expose a public `prepare_monitor` library entry with these
logically separate inputs:

- A borrow of the selected `CheckedPackageV2` and a selector for one admitted
  monitor root in that package. The entry SHALL obtain its
  `CheckedMonitorPlanRef` from that package; a view or matching label from
  another package SHALL NOT supply any of its contents.
- The independently selected clause and authentic occurrence, subject,
  timestamped-event profile, target premises, admitted clock/order and progress
  association. Selection SHALL NOT be inferred from a display label or the
  transition bodies.
- Original producer witnesses needed by the existing public IR joins, retained
  from the same checked acquisition and corresponding to final emitted
  identities and occurrences. A caller-created identity row alone SHALL NOT
  establish producer authority.
- Explicit operating work/storage allowances and caller cancellation under
  [FR-003](FR-003-apply-caller-selected-limits.md) and
  [FR-011](FR-011-cancel-a-driver-library-operation.md).

A driver request may group borrows to these canonical values. It SHALL NOT
introduce a copied plan representation, second expression/type vocabulary,
caller-supplied executable callback or unchecked replacement environment.
No authored source, parser, formula renderer or QSL evaluator is an input to
preparation. This specification invents no missing QSL witness carrier and
claims no already implemented handoff for one.

### Concrete clock admission within the partial selection boundary

`admit_monitor_clock_selection` SHALL call the selected IR plan's `clock_for_claim`
with the independently supplied final clause and full `CheckedOccurrence`,
under the remaining work allowance. Every successful clock selection SHALL retain
that native `CheckedMonitorClockRead` in a distinct `AdmittedMonitorClockSelection`
and expose it as a borrow; missing clock
association SHALL refuse, including when `clock_binding` is absent. The
optional independent `TemporalBinding` is an additional expectation: when
present its declaration, ordinal, clock name and closed clock configuration
SHALL match the native read. Its absence waives only that comparison and
SHALL NOT waive native clock admission or choose an occurrence or clock name.

The driver SHALL retain every native clock refusal payload unchanged,
including code, `at`, occurrence, consumed work and attempted work. Native
`WorkLimit` SHALL map to the outer stable `work` code; `WrongClause`,
`WrongRole`, `UnknownOccurrence` and `MissingClock` SHALL map to `selection`.
A successful native receipt SHALL be charged exactly once to the same
cumulative preparation allowance. All existing package/root, subject,
profile, target and witness boundaries remain required.

The existing `admit_monitor_selection` remains an explicitly partial
package/root/occurrence/target selection. Its `AdmittedMonitorSelection` does
not claim clock admission. The separate clock entry SHALL first validate the
shared package/root/key, subject, profile and target prefix, then invoke the
native clock reader with the independent clause and full role/ordinal.
Native `WrongClause`, `WrongRole`, `UnknownOccurrence` and `MissingClock`
refusals SHALL retain their first causes. Before constructing a successful
clock-admitted capability, the driver SHALL validate full source-map
authenticity and any optional independent clock expectation. Source-map
inspection may consume work before the native read; its equality check SHALL
NOT replace the native semantic refusal. The exposed inner selection SHALL
include the clock work prefix so subsequent projection cannot reset that
allowance.

QSL's current Timed PlanRoot has a model-time binding and no temporal clock
row. IR reads that admitted model-time source directly, including its exact
unit. Configured clock rows remain a separate association for non-Timed
profiles. The no-token `MissingClock` case needs a non-Timed admitted monitor
plan without such a row; the current Timed producer does not exercise it.

This partial clock component establishes no order or progress association,
complete generated record closure, full `Program`, or producer-to-runtime
qualification. QSL PR #2395 (QSL-1728) now supplies the owner lookup from
an original `MonitorSubject` clause key and Claim ordinal to its final emitted
checked declaration. Driver consumption of that mapping is defined below;
the lookup does not supply a Timed clock configuration.

### Native final-subject admission

The driver SHALL expose this partial native-subject entry without a production
`qsl-analyze` dependency or a copied plan representation:

```rust
admit_monitor_final_subject(
    request: &MonitorPreparationRequest<'_>,
    subject: &qsl_replay::MonitorSubject,
    emission: &qsl_package::Emission,
    evidence: &quire_contract_model::CheckedPackageEvidence,
    read_limits: quire_contract_model::CheckedPackageReadLimits,
    cancel: &quire_exact::Cancel,
) -> Result<AdmittedMonitorFinalSubject<'_>, MonitorPreparationError>
```

The caller SHALL retain the genuine plan's native `subject()` and its final
`Emission`. The driver SHALL strictly read that emission with native evidence
and explicit native read limits, then compare the complete native admitted
package with the independently selected request package using IR's complete
package equality. Equal package names, display labels or partial tables SHALL
NOT establish this association. Native read refusals and incomplete outcomes
SHALL retain their unchanged typed payloads; the outer codes SHALL be
`selection` and `work`, respectively.

The entry SHALL require the subject's occurrence node to equal its clause
node, its package to match the emission's native package name, and its full
origin to denote a Claim. It SHALL execute
`emission.temporal_binding_rows().declaration(subject.clause_node(),
subject.occurrence().origin().ordinal())`; `None` SHALL return a typed mapping
absence retaining the original native subject. A declaration elsewhere in the
emission SHALL NOT substitute for the selected root's clause. The mapped
final declaration and native Claim role/ordinal SHALL match the independent
request's clause and full authentic source-map row. The admitted name row at
that exact declaration/ordinal SHALL match the subject's retained name; a
name SHALL never select a row. Existing package-instance/root/keys, subject,
profile, target and full source-map validations SHALL remain required.

The successful private-field token SHALL retain native subject/emission
borrows and the verified partial selection. Its `admit_clock(cancel)` method
SHALL resume the same driver work prefix through the existing native clock
admission path; it SHALL NOT reset the allowance or derive a clock config.
Mapping success alone SHALL establish no complete plan identity, formula or
target provenance beyond the independent request checks, no clock receipt,
no source-record closure, and no executable Program.

Native strict acquisition, complete native package equality and the owner's
linear declaration lookup are explicitly outside the driver's material-work
ledger. The owner lookup has no bounded-read receipt: `TemporalBindingRows`
`len()` counts clock rows, including zero for a mapped Timed subject, and
SHALL NOT be used as a declaration-work count. Native read limits govern the
acquisition; cancellation SHALL be checked before acquisition and before continuing
after successful synchronous native steps. Native failure payloads SHALL
return unchanged. Driver identity/name comparisons, partial selection and resumed
clock admission SHALL consume their actual existing ledger prefix. This
component SHALL NOT claim full preparation work/storage qualification.

Before direct producer comparisons or origin conversion, the driver SHALL
inspect both package-name byte strings, the complete occurrence and clause
native NodeKeys, and the origin role/ordinal under its material ledger.
Native NodeKeys SHALL be walked iteratively through the owner's borrowed
`view()`/path-segment views, without owned path extraction, Serde or identity
copies. Work SHALL charge each native key/container/tag, eight units per
ordinal and each UTF-8 byte under the existing material schedule. Traversal
scratch SHALL share the existing storage ledger and be released on work or
cancellation failure. These direct comparisons are not part of the native
acquisition/equality/owner-lookup exclusions. With a valid same emission,
zero driver work SHALL refuse before comparing an edited producer identity.

### Executed narrow clock refusal evidence

The production `tests/monitor_preparation.rs` controls exercise FR-015-AC-13:
wrong clause, role and ordinal preserve the original native payload; the
native `MissingClock` payload retains its outer `selection` classification;
same-clause twin claims retain their exact requested occurrences; one-under
native allowances preserve the original work refusal; and pre-cancelled
`Deadline` returns no clock capability and leaves the original inputs intact.
The authentic Timed PlanRoot now admits a native model-time clock, including
its exact claim name, clock name and unit. A driver entry test with an admitted
non-Timed plan lacking a clock row is still needed to qualify the no-token
`MissingClock` obligation. Contract CG PR #749 removes the previous production
compilation blocker. Complete generated record closure, order/progress and
full `Program` preparation remain Planned.

### Public entry shape and unresolved input handoff

The public entry's request/result shape SHALL be:

```rust
prepare_monitor(
    request: MonitorPreparationRequest<'_>,
    cancel: &quire_exact::Cancel,
) -> Result<Rc<quire_contract_runtime::monitor::Program>, MonitorPreparationError>
```

`MonitorPreparationRequest` SHALL group borrows of the selected admitted
package/root, the independent canonical selection and original admitted
associations, same-final-emission witness rows, and finite operating
work/storage allowances. It SHALL NOT accept `ProgramParts`, a caller-built
`TypeEnvironment`, replacement bodies or a callback supplying semantics.
The package SHALL resolve its selected root and four entries itself; a foreign
borrowed view SHALL NOT replace that resolution.

This freezes the entry/result shape and failure distinctions, not a concrete
request field list or an implemented Rust type. The request's concrete public
fields SHALL be frozen only after the owning interfaces supply authentic
final source record/member and union identities and admitted clock/order/
progress associations. `PlanRoot` currently contains no final `AdmittedContext`;
its selected model rows cannot establish generated source record authority.
Raw caller bytes or a new local association wrapper SHALL NOT be treated as
proof that an order/progress association was independently admitted. No
speculative carrier for those unavailable inputs is introduced here.

## Outputs

On success, preparation SHALL return the runtime's retained `Rc<monitor::Program>`
constructed through `Program::try_new` from consumed `ProgramParts`. The owner
SHALL be usable in the existing RT `BuildPlan` with explicit `StepBudget`.
Preparing the owner SHALL NOT call `monitor::build`.

On failure, preparation SHALL return a typed error retaining the first failing
selection or source locus and the relevant native cause. No partial program
SHALL escape. The error SHALL distinguish:

- missing or mismatched package, root, clause, occurrence, subject, target or
  admitted association;
- an admitted ordinary form or operation unsupported by RT's representation;
- a missing, foreign, conflicting or otherwise refused upstream identity
  witness or admitted nominal/equality read;
- an operating work/storage stop or cancellation; and
- the runtime's original `ProgramRefusal` during construction.

`MonitorPreparationError` SHALL expose distinct typed categories for selection,
unsupported operation, upstream projection, work, storage, cancellation and
runtime program construction. A broken lowering invariant SHALL retain the
distinct driver internal-fault category. Native node/body-path or occurrence
loci and upstream causes SHALL remain available without parsing human-readable
error text. These are failure categories, not temporal outcomes or a complete
new catalog of upstream causes.

If the driver breaks an invariant of its lowering contract, the error SHALL
remain a driver internal fault with its native construction/read cause, rather
than refusal of already admitted input. Expected operating failures SHALL NOT
be relabelled as driver faults.

A reader refusal before this entry SHALL remain an IR admission refusal.
Preparation failure SHALL NOT become a temporal `False`, `Incomplete`,
reading-gap `Failed`, runtime step stop or executor invariant failure. Runtime
build/execution retain those owners' distinct failure and outcome contracts.

## Behavior

### Bind the independent selection

Before constructing the runtime owner, the driver SHALL match the root's
admitted clause and selected authentic occurrence to the independent selection.
The driver SHALL bind the same package instance, resolved root and four entry
content keys, subject, profile, target counter/unit/width/gap/rate/uncertainty/
capacities, clock binding, admitted order and progress context. Equal labels,
numeric graph indices or source `PlanId` metadata SHALL NOT supply that match.
The driver SHALL NOT create another persistent plan identity.

The driver SHALL retain the selected package lock and original admitted clock,
order and progress value encodings in the corresponding existing RT `Layout`
fields. It SHALL NOT replace them with a local generation number, new identity
record or default association. Canonical structural keys SHALL retain their
complete package, path segments, nested node segments and ordinal. A name
string SHALL NOT replace an original `NodeKey`, `MemberId`, `VariantId` or
`UnitId`.

### Lower the retained closure

Starting at the four admitted entries, the driver SHALL resolve the reachable
ordinary function/expression and type/declaration closure. Each reachable
function and expression SHALL have one runtime arena address. Repeated uses
SHALL retain the original source key and inline body path; arena indices are
addresses only. For graph-local references, the driver SHALL resolve the
selected package's `structural_key` before converting to the canonical shared
Exact key; an index selector or raw JSON spelling SHALL NOT be treated as that
key. Recursive calls SHALL remain resolved ordinary call edges,
rather than host recursion or an invented semantic depth cap.

The driver SHALL lower admitted ordinary operations to the corresponding
closed RT `Operation` forms. Parameter order, lexical ownership, lazy branch
selection, evaluation order, declaration/member/variant identities and nominal
presence SHALL remain unchanged. A missing RT operation SHALL cause typed
unsupported lowering before initialization. It SHALL NOT cause a callback,
substituted temporal algorithm or partial executable program. The driver
SHALL NOT interpret temporal formula operators to supply missing body meaning.

The driver SHALL materialize the needed admitted native type/declaration
closure in canonical QSV `TypeEnvironment`, using the original Exact shared
values/types and identities. Source-owned records, tuples and unions SHALL
retain declaration order, closed/open shape, presence and payload types. Collections SHALL retain their
structural Exact element, kind and cardinality types without inventing a
collection declaration identity. The driver SHALL NOT treat a selected-domain
projection as a complete projection of generated source types, or invent an identity when the required
projection/witness is unavailable.

[IR FR-357](ix://agent-ix/quire-contract-ir/FR-357) owns the complete selected
generated-source type/declaration projection needed for this environment.
Its first bounded witnessed subset, exercised by merged integration PR #47,
reads explicitly selected model declaration/field and equality facts only;
it does not traverse the monitor root or establish the complete closure. That subset alone SHALL NOT be used to construct a complete
`TypeEnvironment` or RT `ProgramParts`.

For equality, the driver SHALL obtain opaque `AdmittedEqualityDecision` through
IR `admitted_equality` with matching original producer application, fact and
source-unit rows. It SHALL obtain `CheckedEquality` through
`CheckedEquality::from_admitted_decision`. It SHALL NOT choose a schedule,
recheck a decision, reconstruct producer identities, filter enum facts or
select a new unit conversion path. The hidden QSV reader seal/mint SHALL NOT
be called by the driver or re-exported by this interface. Existing IR/QSV
permitted mechanical materialization and its original typed arithmetic/resource
failures remain authoritative.

The driver SHALL move the canonical environment and complete owned arenas into
`ProgramParts` and `Program::try_new`. It SHALL NOT retain a second executable
DTO or borrow source/compiler/reader preparation from the returned program.
The admitted source closure, rather than a reconstructed formula, SHALL
supply the four entry bodies.

### Retain exact plan and operating premises

Preparation SHALL consume QSpec FR-487's current seventeen-binding root through
the admitted view. State SHALL retain `now`, `history_start`, ordered
`buffer_<n>` fields, `activations`, then exact Integer `event_position`.
An admitted activation status SHALL remain opaque to plan selection: its
actual type and ordinary uses SHALL be lowered without a second hard-coded
status-record schema.

Capacities SHALL retain their exact admitted values, including zero activation
capacity. No host-width narrowing, default capacity, repeated capacity formula,
extra bound tick-interval table or semantic size/depth cap SHALL be introduced.
The original tick unit, exact rational rate/uncertainty, counter width and
maximum modular reading gap SHALL remain in `Layout`. Runtime derives deltas
and executes entries; preparation SHALL NOT supply replacement tick updates
or gap outcomes.

Lowering SHALL visit relevant reachable nodes and edges with an iterative
worklist and charge their identity/value material against the caller allowance.
It SHALL NOT sort or repeatedly index the entire package before selecting the
relevant closure. An operating stop or cancellation SHALL return no partial
program and leave the admitted input and selection unchanged. Operating
allowances SHALL NOT alter semantic capacities or become temporal verdicts.

### Lifetime and dependency boundary

After successful preparation, dropping source bytes, QSL checked objects,
producer witness holders and admitted-package preparation SHALL leave the
program usable. Runtime `build` SHALL create fresh independent state for each
build under RT FR-276; preparation SHALL share no live state or rebinding path.

The production dependency direction SHALL be driver to IR/QSV/Exact and RT.
The cycle-free IR model SHALL NOT depend on RT or the driver. RT SHALL remain
a leaf without IR, driver or QSL dependencies. Preparation SHALL perform no
QSL compile/recompile, source parsing, evaluator call, CG generation or external
effect.

The implemented independent bounded locus-copy component in
[FR-016](FR-016-retain-monitor-refusal-loci.md) has an independent allowance and
qualifies only already-resolved canonical locus retention. The driver owns
later threading of remaining preparation work/storage into diagnostic
selection, target-path derivation, failure assembly and outer error boxing;
these integrations remain Planned. IR/model owns allocation-aware copying of
opaque operation and native projection/callable failure payloads. Caller-input
acquisition and complete preparation/Program storage remain Planned; a locus
copy receipt SHALL NOT reset or certify the complete preparation allowance.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-015-AC-1 | Planned until IR-721 lands. A fresh QSL plan-root package is strictly admitted and its independently selected root, clause, authentic occurrence, subject and target prepare one RT owner. Changing each selection independently, including another package instance with equal labels/indices, gives typed selection refusal before init and no owner. | Test |
| FR-015-AC-2 | Planned until IR-721 lands. Layout retains the admitted root/four-entry content keys, lock, clock/order/progress associations, exact tick premises and capacities. Independent edits are either retained exactly after valid admission or refused at the owning selection boundary; labels, metadata PlanId, defaults and truncated keys never supply a match. | Test |
| FR-015-AC-3 | Planned until IR-721 lands. A genuine generated closure containing shared expressions, lexical bindings, lazy branches, recursive calls, records, tuples, sequences/folds, union payloads and native comparisons lowers with original identities/order and one address per expression/function. Its actual admitted bodies execute; no handwritten temporal body substitutes. | Test |
| FR-015-AC-4 | Planned until IR-721 lands. Generated source records, tuples, collections and unions reach the canonical TypeEnvironment with complete shapes, presence and original member/variant identities. Missing/foreign identity or projection gives the native typed refusal and no invented type or partial owner. | Test |
| FR-015-AC-5 | Planned until IR-721 lands. Reachable equalities join same-acquisition producer witnesses against final admitted identities and adopt CheckedEquality without rechecking. Missing, conflicting, foreign and stale-after-rewrite rows refuse with exact native cause/locus; no seal/mint or fallback schedule path exists in the driver. | Test |
| FR-015-AC-6 | Planned until IR-721 lands. A seventeen-binding plan retains the ordered State fields, buffers, exact event_position and admitted opaque activation-status type. Zero and above-host-width capacities retain their values without a local cap, bound interval table or status-schema restriction. | Test |
| FR-015-AC-7 | Planned until IR-721 lands. A reachable admitted operation absent from RT gives typed unsupported lowering at that operation before init; an unrelated unsupported node outside the closure does not block the plan. Neither invokes a callback, evaluator, parser, compiler or replacement temporal algorithm. | Test |
| FR-015-AC-8 | Planned until IR-721 lands. Work/storage exhaustion, cancellation, upstream nominal/equality refusal and RT ProgramRefusal preserve distinct typed causes/loci, return no partial program and leave inputs unchanged. A broken lowering invariant is a driver fault rather than input refusal; expected operating failure is not a fault. No preparation failure becomes a temporal verdict, StepResourceStop or ExecutorInvariantFailure. | Test |
| FR-015-AC-9 | Planned until IR-721 lands. After producer/reader preparation is dropped, two builds of one prepared owner have independent fresh state. Actual admitted entries execute with explicit StepBudget; baseline event, watermark, poll and gap controls preserve RT's event-position/baseline rules and exact outcomes. Preparation itself invokes no entry or build. | Test |
| FR-015-AC-10 | Planned until IR-721 lands. Real producer/read/lower/RT execution is compared with QSL's independent tick reference for RT FR-276's boundary, uncertainty, buffer21/22, wrap/gap, changed identity/order, activation and typed-stop controls. QSpec vectors are read by reference. Construction diagnostics and a single pilot plan receive no complete FR-276 execution credit. | Test |
| FR-015-AC-11 | Planned until IR-721 lands. A large selected closure amid an unrelated long graph visits only relevant reachable nodes/edges and identity/value material under the caller allowance, without a whole-package sort or repeated full indexing. Equal budgets give equal typed work stops without a semantic depth/size ceiling. | Test |
| FR-015-AC-12 | Planned until IR-721 lands. Public consumer/API and dependency controls establish one retained RT Program, no source input/callback, no driver seal/mint export or copied plan/expression/value family, and no IR/QSL/driver backedge from RT or RT edge from the cycle-free IR model. | Test |
| FR-015-AC-13 | The separate clock-admission entry preserves native `WrongClause`, `WrongRole`, `UnknownOccurrence`, `MissingClock` and `WorkLimit` refusal payloads, including exact locus, occurrence, consumed work and attempted work. Semantic clock refusals map to outer `selection`; native work refusal maps to `work`. A missing clock returns no clock-admitted token even when the optional clock expectation is absent. Pre-cancellation returns the original cancellation cause, no clock-admitted token and unchanged inputs. Existing partial selection remains explicitly unclocked. | Test |
| FR-015-AC-14 | A healthy clock-admission entry exposes the native `clock_for_claim` receipt for its independently selected final clause/Claim, charges successful receipt work exactly once to the remaining cumulative allowance, and compares every field of any optional independent clock row. The exposed inner selection retains that work prefix for later projection. Authentic Timed model-time receipt and cumulative work are exercised; optional independent row comparison and complete criterion qualification remain Planned. This establishes no final PlanRoot subject-occurrence mapping, order/progress association or complete program. | Test |
| FR-015-AC-15 | A real PlanRoot's original native subject and final emission admit one final-subject token only after strict native emission/package equality, owner declaration lookup, exact root/clause/Claim, full source-map, native name and existing independent selection checks. Two-clause controls select their own final declarations; absent lookup, foreign subject/emission/package, role/node/ordinal and independent request discrepancies give typed refusal. Native read limits preserve exact payloads; cancellation returns the original cause and no token. Driver work is retained across the token's actual native model-time clock path. Native acquisition/equality/linear lookup remain explicitly outside the driver ledger. No complete preparation credit is claimed. | Test |

## Dependencies

- [IR FR-038](ix://agent-ix/quire-contract-ir/FR-038) owns admission, borrowed
  views, independent selection and root/four-entry identity.
  [QSpec FR-487](ix://agent-ix/quire-specification/FR-487) and
  [QSpec FR-322](ix://agent-ix/quire-specification/FR-322) own the current
  carriers. Those artifacts and vectors are read by reference, not copied.
- [QSL FR-251](ix://agent-ix/quire-spec-language/FR-251) owns derivation and
  entry semantics. Its public producer is
  `qsl_analyze::monitor_root::emit_plan_root`. Merged QSL-1463 retains and
  rekeys model declaration/field, equality producer/fact and selected source-unit
  witness rows in the final `PlanRoot.emission`. QSL-1630 now binds atom,
  guard and helper bodies to plan identity; QSL-1631 now retains and rekeys
  source enum declaration/member rows. Collections are structural and require
  no declaration identity row. Contract IR PR #905 supplies source union identity admission/projection;
  the matching authentic final producer handoff (QSL-1723) remains required;
  QSL PR #2385 now retains and rekeys source-record/member rows, with native
  admission/projection supplied by Contract IR PR #910. Joining those rows
  in the driver and qualifying complete generated record closure remain
  separate work. Selected model rows and source enum rows do not substitute
  for either source-owned witness channel. Final admitted clock/order/progress associations must also
  come from their owning admitted interfaces. No new QSL carrier is defined
  here, and the concrete preparation request field list remains unfrozen until
  those inputs exist.
- [IR FR-353](ix://agent-ix/quire-contract-ir/FR-353) owns the no-recheck equality
  read; [IR FR-354](ix://agent-ix/quire-contract-ir/FR-354) owns nominal reads.
  [IR FR-357](ix://agent-ix/quire-contract-ir/FR-357) owns the complete selected
  generated-source type/declaration projection. Its first witnessed subset is
  insufficient for a complete `TypeEnvironment` or RT `ProgramParts`; complete
  projection and driver lowering remain planned IR-721 work.
- [RT FR-276](ix://agent-ix/quire-contract-runtime/FR-276) owns Program/build,
  bounded state and executor outcomes. The tick-aware reference is
  `qsl_eval::temporal::TickMonitorReference`.
- [TC-029](../test-cases/TC-029-admitted-monitor-program-preparation.md) defines
  planned evidence. IR-721 owns preparation; IR-722 owns execution
  qualification. Their obligations remain open until that evidence exists.
