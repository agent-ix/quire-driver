---
id: FR-017
title: "Lower and retain one admitted native monitor program"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-016
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-354
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-357
    type: depends_on
  - target: ix://agent-ix/quire-contract-runtime/FR-276
    type: references
---
# FR-017: Lower and retain one admitted native monitor program

## Description

The monitor lowerer SHALL materialize the selected admitted executable closure
into the existing runtime representation without replacing source identities,
introducing another executable vocabulary or inferring missing authority.
[FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns admission, the
preparation request and the outer entry/error; this requirement owns native
lowering and retention after that admission. Lowering executes no initializer
or runtime build.

## Inputs

The agreed full lowering handoff is:

```rust
pub fn lower_monitor(
    selection: AdmittedMonitorSelection<'_>,
    cancel: &quire_exact::Cancel,
) -> Result<std::rc::Rc<quire_contract_runtime::monitor::Program>, MonitorPreparationError>;
```

The lowerer SHALL consume the existing opaque selection by value. It SHALL
obtain the original package, four-entry plan view, independent premises,
canonical witness borrows and operating limits through the selection's
read-only interface. It SHALL NOT accept a second request/plan DTO, a caller
`ProgramParts`, a replacement environment or an executable callback.

The selection establishes only the identity/premise matches that admission
actually checked. It SHALL NOT substitute for authentic source record/member,
union, native type, clock/order/progress or complete closure authority that an
owning interface has not supplied. Native owner input types and their actual
public read APIs govern that handoff; this requirement invents no missing
carrier or speculative getter name.

## Outputs

Full success SHALL be the original runtime-owned `Rc<monitor::Program>` created
by moving canonical `TypeEnvironment`, `Layout`, functions and expressions into
`ProgramParts` and `Program::try_new`. No partial owner SHALL escape a failure.
After success, the program SHALL retain no source/package/request/witness or
temporary preparation borrow.

Private component work may materialize existing RT `FunctionId` root addresses,
`Vec<Function>` and `Vec<Expression>` directly. Such staging consists only of
native RT owners, not another plan/type/expression DTO. It SHALL NOT be exposed
as a separate public scalar preparation API or counted as a complete Program.
The full entry SHALL NOT be published as a constant-unavailable placeholder
while its required authority-bearing inputs remain absent.

## Behavior

### Retain original identities and lexical ownership

The lowerer SHALL resolve original graph-local selectors through the selected
package's structural-key interface before converting them with the owning
native key codec. It SHALL retain complete package, nested key segments and
ordinal; arena positions are addresses only. It SHALL NOT mint source IDs,
replace a content key by a numeric index or infer nominal members from a name.

The lowerer SHALL obtain callable headers, parameter order, body handles,
native types and declaration/equality facts through their actual owner reads.
Ordinary source functions and authenticated anonymous monitor entries SHALL
retain their respective native owner interfaces. A source-only callable read
returning no source declaration SHALL NOT be interpreted as authority for an
anonymous monitor entry. No shadow callable reader or whole-package callable
table supplies a missing selective read.

Function and expression edges SHALL retain source order, native operand types,
parameter positions and actual owning `FunctionId` context. The selected
ordinary operation SHALL map to its corresponding closed RT `Operation`, not
a computed result or a reimplemented evaluator. Calls SHALL reference retained
callee addresses; recursive calls SHALL remain graph edges rather than host
recursion or an invented semantic depth limit.

The lowerer SHALL retain the original expression key and native child path.
The current strict IR boundary admits Application only at a node body root;
nested Applications SHALL NOT be fabricated as lowering fixtures. Nonempty
paths retain actual admitted nonapplication terms/reference boundaries under
[FR-016](FR-016-retain-monitor-refusal-loci.md).

### Preserve scoped sharing and package identity

Memoization SHALL preserve the owning invocation context of argument/local
dependent expressions. A shared original node does not authorize an argument
of one function to become an argument of another. Context-free sharing SHALL
retain the one original source key without duplicate unowned executable
locations. The lowerer SHALL visit relevant selected nodes/edges iteratively,
without collecting a second whole-package graph or repeatedly scanning unrelated
callables before the selected closure.

Fresh lowering outputs SHALL resolve addresses only within their own native
arenas. Equal numeric function/expression addresses under another package or
subject SHALL NOT establish source identity. Full monitor Layout SHALL retain
the original clause, subject, occurrence and identity-bearing root/buffer
associations; a numeric subformula index SHALL NOT alias another subject.
Scalar arena tests alone SHALL NOT claim that full temporal association check.

### Continue one operating allowance

Full lowering SHALL resume admission's consumed work under the same total
caller limit and original cancellation handle. It SHALL NOT reset a fresh full
allowance after admission. It SHALL charge actual reached identity/value
material, native owner-reported projection work and retained arena storage.
Native reader success work SHALL come from its owning result, not a guessed
formula, a returned-row count or charging all remaining allowance as a receipt.

Work, storage and cancellation stops SHALL return no partial published owner.
Storage reservations SHALL be checked before exposing owned arenas; allocation
failure SHALL retain its native cause. Operating limits SHALL NOT change target
premises, semantic capacities, source types or evaluation meaning.

Component tests may obtain genuine native callable/type/node views in explicit
setup before exercising the private assembly phase. Their phase allowance
covers that phase only. Setup outside it SHALL NOT be reported as accounted
full preparation work, nor shall a component result satisfy the resumed-total
budget criterion. The production handoff still requires the actual selective
native reads and their reported work.

### Preserve native dispositions and authority

The lowerer SHALL retain original IR read/projection failures, located
`UnsupportedOperation`, operating work/storage stops, original cancellation
cause and native RT `ProgramRefusal` as distinct outer failure shapes. A broken
lowering invariant SHALL remain a driver fault rather than refusing an already
admitted semantic input. New native outer error payloads remain owned by
[FR-015](FR-015-prepare-an-admitted-monitor-program.md).

A missing nominal/type/association channel SHALL NOT produce an invented
`TypeEnvironment`, member identity, empty/default association or successful
Program. It remains an unqualified owner boundary until the real input and
read exist. Private primitive component success does not discharge those
complete-program obligations.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-017-AC-1 | Full lowering consumes the actual opaque admitted selection and produces the existing RT Program from the complete four-entry native closure and authentic type/association inputs. No replacement request, callback, fake environment or constant-refusal preparation stub supplies the handoff. | Test |
| FR-017-AC-2 | After complete success, dropping all source, package, witness, request and preparation objects leaves the RT-owned program usable. Two actual runtime builds retain independent initialized state without rebinding or a borrowed compiler owner. | Test |
| FR-017-AC-3 | Full lowering preserves admission's already consumed work under the same total limit, native read success work, charged storage and original cancellation. An exact/one-under operating stop returns its original typed cause and no partial owner; no guessed reader receipt or semantic-capacity change supplies success. | Test |
| FR-017-AC-4 | Native IR refusal, unsupported source operation, operating stop, cancellation, runtime ProgramRefusal and driver invariant failure remain distinct, with original source node/path and native payload. No failure becomes a temporal outcome or a partial successful program. | Test |
| FR-017-AC-5 | A private component consumes genuine QSL-produced/strictly admitted ordinary Integer/Boolean callable facts and materializes native RT root addresses/functions/expressions. Its selected arithmetic/Boolean/call operations, argument positions, native projected types and complete source keys match the actual input; no source parse or evaluator supplies lowering meaning. This is component evidence only. | Test |
| FR-017-AC-6 | Within one genuine checked package, two functions sharing actual parameter/query handles retain distinct native Argument FunctionId owners and positions. Context-qualified memoization retains the identical original source keys without merging the owning invocations or rewriting keys. Native scalar outputs remain inspectable after setup destruction; separately lowered packages also retain their own qualified keys and scoped addresses. This supplies no temporal subformula-association credit. | Test |
| FR-017-AC-7 | The complete monitor program retains original clause/subject/occurrence/root/buffer ownership. Equal numeric subformula indices under different genuine subjects do not alias in selection or retained Layout; no scalar-only sample substitutes for this control. | Test |
| FR-017-AC-8 | Lowering consumes native typed owner facts and emits only RT-native executable types, with no copied evaluator, transition implementation, second IR reader/plan vocabulary, hidden identity mint, whole-package index, repeated unrelated callable scan or semantic depth cap. | Inspection |

## Qualification Status

All criteria remain Planned until their exact controls execute. The first
intended component checks cover ordinary source scalar arena materialization
and retention only. They invoke no `emit_plan_root`, fabricated monitor
selection, complete `Layout`, replacement `TypeEnvironment` or RT build.
Complete Program ownership, resumed whole-preparer accounting, authenticated
anonymous-entry/type closure and subject/subformula association evidence remain
separate Planned controls. No component observation closes IR-721 AC2.

## Dependencies

- [FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns admission, token
  getters, shared budget/error API, manifest dependencies and orchestration.
- [FR-016](FR-016-retain-monitor-refusal-loci.md) owns native diagnostic payloads.
- [IR FR-354](ix://agent-ix/quire-contract-ir/FR-354) owns authentic callable and
  runtime-context reads; selective success-work and authenticated anonymous
  entry reads must be delivered by that owner rather than inferred here.
- [IR FR-357](ix://agent-ix/quire-contract-ir/FR-357) owns complete generated-source
  closure authority. Existing selected facts alone establish no complete
  nominal environment or admitted order/progress association.
- [RT FR-276](ix://agent-ix/quire-contract-runtime/FR-276) owns native program
  validation, initialization and execution.
- [TC-031](../test-cases/TC-031-monitor-native-lowering.md) separates scalar
  component evidence from complete monitor qualification.
