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

The agreed preparer-independent public helper is
`quire_driver::monitor::lowering::lower_scalar_functions`:

```rust
pub fn lower_scalar_functions(
    package: &CheckedPackageV2,
    roots: &[CheckedNodeId],
    identities: &SelectedExactIdentities,
    limits: MonitorPreparationLimits,
    cancel: &quire_exact::Cancel,
) -> Result<
    (Vec<quire_contract_runtime::monitor::FunctionId>,
     Vec<quire_contract_runtime::monitor::Function>,
     Vec<quire_contract_runtime::monitor::Expression>),
    MonitorPreparationError,
>;
```

Its tuple SHALL contain only existing owned RT requested-root addresses,
functions and expressions, with no source/package/preparation borrow or second
plan/type/expression DTO. Root addresses SHALL preserve requested-root order;
repeated roots SHALL resolve to the same FunctionId rather than duplicate the
function. Empty roots SHALL return empty native arrays, subject to cancellation
and operating checks. Failure SHALL return no partial tuple. This helper is
not a complete Program or a monitor preparation entry.

The public helper SHALL start its explicit independent operating allowance.
The full by-value `lower_monitor` SHALL reuse the same private worklist under
admission's inherited budget; it SHALL NOT invoke the public helper with a
fresh allowance that discards admission work. P8 owns shared module registration,
exports, budget/error/request interfaces, manifest and lock changes.
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
a computed result or a reimplemented evaluator. Where the complete lowerer
supports calls, they SHALL reference retained callee addresses; recursive calls SHALL remain graph edges rather than host
recursion or an invented semantic depth limit.

The lowerer SHALL retain the original expression key and native child path.
A transparent Reference node with its own genuine structural key SHALL retain
that key and its native projected type on a native Expression carrying the
already lowered closed scalar operation and original operand addresses/owner.
It SHALL NOT erase that original location, introduce a wrapper operation or
compute a replacement value. Only the declared nonallocating scalar variants
may be copied this way; future allocating variants require explicit support.
The current strict IR boundary admits Application only at a node body root;
nested Applications SHALL NOT be fabricated as lowering fixtures. Nonempty
paths retain actual admitted nonapplication terms/reference boundaries under
[FR-016](FR-016-retain-monitor-refusal-loci.md).

### Closed initial scalar capability

The standalone helper SHALL accept only ordinary source functions whose native
projected parameter, result and reached executable types are exactly
`quire_exact::ValueType::Integer` or `ValueType::Boolean`. Bounded numeric,
nominal, composite and other types are outside this initial capability.
Functions SHALL have no context or frame. Their executable bodies may be a
reference to one of that callable's authenticated ordered parameters, a
reference to another supported executable node, or a root Application from
the table below. Ordinary source callable reads SHALL retain authenticated
Value(Parameter) and Value(Literal) body roots as supplied by the native reader.
A direct Value(Parameter) body SHALL materialize its declared argument position,
original body key and native scalar type. A Value(Literal) body is readable
input but remains outside this helper's closed capability: it SHALL reach the
driver's located Unsupported result with no operation record. This ordinary
source capability SHALL NOT widen the separate monitor-entry Expression-only
read or qualify a complete monitor entry. A parameter reference SHALL become native
`Argument { function, position }` from callable membership and declaration
order, never by interpreting its name/level metadata as executable syntax.
References to parameters from another callable SHALL NOT acquire local ownership.

Every supported Application SHALL have the exact listed native operator class,
arity and operand/result types. Its reader-admitted `operation.read()` SHALL
have mode None, member None, execution_point None, and empty laws, constraints
and leaves. All operands SHALL be original ordered reference terms to supported
executable nodes or parameters. No broad operator family implies support.

| Native operation identity | Operator / arity | Operand types -> result | Native RT operation |
| --- | --- | --- | --- |
| `quire.op.integer.add` | Binary / 2 | Integer, Integer -> Integer | IntegerAdd |
| `quire.op.integer.sub` | Binary / 2 | Integer, Integer -> Integer | IntegerSubtract |
| `quire.op.integer.mul` | Binary / 2 | Integer, Integer -> Integer | IntegerMultiply |
| `quire.op.integer.negate` | Unary / 1 | Integer -> Integer | IntegerNegate |
| `quire.op.integer.lt` | Binary / 2 | Integer, Integer -> Boolean | IntegerOrder, OrderingOperator::Less |
| `quire.op.integer.le` | Binary / 2 | Integer, Integer -> Boolean | IntegerOrder, OrderingOperator::LessOrEqual |
| `quire.op.integer.gt` | Binary / 2 | Integer, Integer -> Boolean | IntegerOrder, OrderingOperator::Greater |
| `quire.op.integer.ge` | Binary / 2 | Integer, Integer -> Boolean | IntegerOrder, OrderingOperator::GreaterOrEqual |
| `quire.op.boolean.not` | Unary / 1 | Boolean -> Boolean | BooleanNot |
| `quire.op.boolean.and` | Binary / 2 | Boolean, Boolean -> Boolean | BooleanAnd |
| `quire.op.boolean.or` | Binary / 2 | Boolean, Boolean -> Boolean | BooleanOr |

This is a closed initial set, not an illustrative list. Literal, If, Let,
ordinary source Call, equality, integer division/remainder/modulo, Boolean
implies, dependency references, general aggregate construction and every other
form/identity/metadata shape SHALL return located typed UnsupportedOperation
when admitted but outside this set. The parameter metadata aggregate is read
only through authenticated callable membership, not lowered as an aggregate.
A native selective callable/type/operation read failure SHALL retain its
original native payload rather than becoming Unsupported. An admitted root
with no supported ordinary source callable SHALL receive a located Unsupported
result, not an invented callable. No generic integer operation, implied Boolean
expansion, evaluator or nominal environment supplies missing support.

The helper SHALL discover only requested functions and their reached supported
expression dependencies iteratively, in requested-root and source-operand
order, bounded by actual caller work/storage and cancellation. Ordinary calls
are initially unsupported: no callee graph is traversed or recursive callable
body expanded; a reached Call produces Unsupported at that original locus.
Repeated acyclic expression references SHALL share within their correct owner
context. A native reader's cycle refusal SHALL remain native; an impossible
pending-expression cycle after admission SHALL be a typed driver invariant
fault, not host recursion or a fabricated successful arena. Supporting calls
later requires an explicit capability amendment and reserved native function
addresses before body discovery, not an implicit expansion of this set.

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
Driver-owned vector, text and worklist storage reservations SHALL be checked
before exposing owned arenas; allocation failure SHALL retain its native cause.
The scoped implementation accounts for measurable driver-owned capacity and
native key-constructor internal reservations through the owner's
`NodeKey::try_new_with` callback. Constructor Policy failures SHALL retain the
original budget/cancellation error unchanged; Construction failures SHALL retain
native InvalidNodeKey. Caller-owned input acquisition/allocation and native
diagnostic payload cloning do not receive complete retained-storage
qualification from these constructor cells. Exact whole-preparation storage
controls remain Planned; native constructor coverage SHALL NOT upgrade them.
Operating limits SHALL NOT change target
premises, semantic capacities, source types or evaluation meaning.

Source-fixture preflight may obtain genuine native callable/type/node views
in explicit setup without claiming lowering accounting. The public scalar
helper SHALL account for its own selective native reads and assembly under
its independent allowance, using actual owner-reported read work. Pre-acquired
setup views SHALL NOT substitute for those production reads or their receipts.
Neither source preflight nor standalone helper success satisfies the inherited
whole-preparation budget criterion.

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
read exist. Standalone primitive helper success does not discharge those
complete-program obligations.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-017-AC-1 | Full lowering consumes the actual opaque admitted selection and produces the existing RT Program from the complete four-entry native closure and authentic type/association inputs. No replacement request, callback, fake environment or constant-refusal preparation stub supplies the handoff. | Test |
| FR-017-AC-2 | After complete success, dropping all source, package, witness, request and preparation objects leaves the RT-owned program usable. Two actual runtime builds retain independent initialized state without rebinding or a borrowed compiler owner. | Test |
| FR-017-AC-3 | Full lowering preserves admission's already consumed work under the same total limit, native read success work, charged storage and original cancellation. An exact/one-under operating stop returns its original typed cause and no partial owner; no guessed reader receipt or semantic-capacity change supplies success. | Test |
| FR-017-AC-4 | Native IR refusal, unsupported source operation, operating stop, cancellation, runtime ProgramRefusal and driver invariant failure remain distinct, with original source node/path and native payload. No failure becomes a temporal outcome or a partial successful program. | Test |
| FR-017-AC-5 | The public lower_scalar_functions helper accepts the closed initial scalar set and returns owned native RT root addresses/functions/expressions under its independent allowance. Requested order and repeated-root FunctionId sharing, exact supported operation metadata/arity/types, parameter ownership and unsupported/native-reader dispositions match the declared contract. No source parse or evaluator supplies meaning; this is component evidence only. | Test |
| FR-017-AC-6 | Within one genuine checked package, two functions sharing actual parameter handles retain distinct native Argument FunctionId owners and positions. Context-qualified memoization retains the identical original source keys without merging the owning invocations or rewriting keys. Native scalar outputs remain inspectable after setup destruction; separately lowered packages also retain their own qualified keys and scoped addresses. This supplies no temporal subformula-association credit. | Test |
| FR-017-AC-7 | The complete monitor program retains original clause/subject/occurrence/root/buffer ownership. Equal numeric subformula indices under different genuine subjects do not alias in selection or retained Layout; no scalar-only sample substitutes for this control. | Test |
| FR-017-AC-8 | Lowering consumes native typed owner facts and emits only RT-native executable types, with no copied evaluator, transition implementation, second IR reader/plan vocabulary, hidden identity mint, whole-package index, repeated unrelated callable scan or semantic depth cap. | Inspection |

## Qualification Status

Qualification is partial. Executed scalar-component evidence covers the eleven
plain operations, exact parameter/result types, root order/repetition, native
source keys, per-function shared-handle ownership, nested sharing, retained
arenas after setup drop, different package keys, declared unsupported operation
loci, bounded-type refusal and independent operating stops. Genuine source
parameter-only and literal-only ordinary callable bodies now have native
Value-root reads. Executed refreshed controls now retain parameter-only native
Argument ownership, scalar type and original body key after setup destruction;
read a literal before driver Unsupported at its original Value node with no
operation; and preserve the complete native UnknownNode payload/work/locus for
an actual selector from another strictly admitted package. Complete
monitor-entry qualification remains Planned; its Expression-only rule is unchanged. Full criteria remain Planned until
all their exact controls execute. The first
intended public-helper checks cover the closed ordinary source scalar
capability and native arena retention only. The existing original-source
preflight establishes genuine shared parameter/body handles, not helper success. They invoke no `emit_plan_root`, fabricated monitor
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
