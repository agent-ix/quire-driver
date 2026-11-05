---
id: FR-016
title: "Retain typed monitor target and unsupported-operation refusal loci"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/FR-015
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: depends_on
---
# FR-016: Retain typed monitor target and unsupported-operation refusal loci

## Description

The driver SHALL expose diagnostic constructors that retain the original
admitted locus of a target-premise mismatch or an unsupported ordinary
operation. These constructors serve the outer preparation failures of
[FR-015](FR-015-prepare-an-admitted-monitor-program.md); they do not prepare a
program, compare independent target selections, or decide runtime support.
Their results are diagnostic values, not admission capabilities or plan IDs.

## Inputs

The public `monitor_refusal` module SHALL expose these entries:

```rust
target_mismatch_at(
    package: &CheckedPackageV2,
    root: &CheckedNodeId,
    premise: TargetPremise,
    buffer_index: Option<usize>,
    occurrence: Option<&CheckedOccurrence>,
) -> Result<TargetMismatch, MonitorLocusError>

unsupported_at(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
    body_path: &[usize],
    kind: CheckedNodeKind,
    occurrence: Option<&CheckedOccurrence>,
) -> Result<UnsupportedOperation, MonitorLocusError>
```

All `Checked*` types are the canonical Contract IR types. The caller supplies
`kind` from the admitted containing node already resolved by its bounded
traversal. The helper SHALL retain that value without a second graph-wide
metadata lookup or another supported-operation classifier. For a nonempty
child path, `kind` remains the containing node's kind. The current strict IR
contract permits `Application` only at a node body root: nested applications
are owner-refused. Therefore full application records are observed at empty
paths; admitted nonempty paths select nonapplication terms and reference
boundaries, not an inline application.

`TargetPremise` SHALL distinguish `Counter`, `CounterWidth`, `TickUnit`,
`MaxReadingGap`, `MaxEventRate`, `Uncertainty`, `BufferCapacity` and
`ActivationCapacity`. Only `BufferCapacity` takes a buffer index; every other
premise requires `None`. This enum names diagnostic fields, not a replacement
target or an encoding of selected/admitted target values.

## Outputs

`TargetMismatch` SHALL retain its typed `premise` and owned `locus`.
`UnsupportedOperation` SHALL retain its owned `locus`, original caller-resolved
`CheckedNodeKind`, and `operation: Option<CheckedApplicationOperation>`.

`MonitorRefusalLocus` SHALL retain these distinct original axes:

- `node`: the supplied original admitted `CheckedNodeId` selector;
- `structural_key`: the exact key resolved by `package.structural_key(node)`;
- `body_path`: the native `Vec<usize>` child path within that node; and
- `occurrence`: the supplied exact `Option<CheckedOccurrence>`.

The helper SHALL clone the resolved key rather than recomputing, synthesizing
or replacing an identity. The owned payload SHALL remain readable after the
admitted package and constructor input buffers are dropped.

`MonitorLocusError` SHALL expose a typed `MonitorLocusErrorCode` distinguishing
`UnknownNode`, `NotMonitorRoot`, `InvalidBodyPath`, `OccurrenceMismatch` and
`InvalidPremiseIndex`. Diagnostic construction failure SHALL return no partial
payload. Its caller decides whether that failure arose from invalid selection
or a broken lowering invariant; the helper SHALL NOT relabel it as an input
unsupported operation or a runtime/temporal outcome.

## Behavior

### Resolve the original locus

When the supplied node cannot resolve through the selected package, the helper
SHALL return `UnknownNode` before attempting body traversal. A graph-local
selector and its resolved structural key SHALL remain separate fields.

For a nonempty `body_path`, the helper SHALL visit `Aggregate.members` and
`Application.arguments` by source-order child index, and `Binding.value` by
child index zero. An unavailable child or a continuation into any other term
SHALL return `InvalidBodyPath`. A `Reference` ends the current node/path;
following it requires the caller to select that original referenced node and
restart with an empty path. The helper SHALL NOT follow references, reorder
children or translate paths to a second location vocabulary.

When an occurrence is supplied, the helper SHALL require the exact role and
ordinal in that node's admitted occurrence list. It SHALL return
`OccurrenceMismatch` for a nonmember. When `None` is supplied, the helper SHALL
retain `None`, including for genuine generated monitor roots whose generated
occurrences were not selected by the caller. No first source-map row or fabricated ordinal supplies an absent
occurrence; source-map absence alone does not invalidate an admitted occurrence.

### Derive a target-premise location

The target constructor SHALL resolve `root` as an actual monitor root of the
selected admitted package. A known node outside that set SHALL return
`NotMonitorRoot`.

The constructor SHALL derive the path to the value of the selected premise's
actual named binding in the admitted root body. Every target-premise path SHALL
include the final `Binding.value` child index zero. `BufferCapacity` SHALL end
at the value of the selected buffer row's `capacity` binding, including that
binding's final child zero, without substituting a numeric subformula index
for its original owner. For the current admitted root shape, uncertainty is
`[14, 0]` and buffer capacity is `[11, 0, buffer_index, 1, 0]`; the constructor
derives these paths from the actual named bindings rather than treating an
example path as an independently authoritative field index. Missing, excessive or inappropriate
buffer indices SHALL return `InvalidPremiseIndex`. A missing named child or
an incompatible term path SHALL return `InvalidBodyPath` rather than inventing
a default premise. The constructor SHALL leave all premise values unchanged;
independent target equality and first-mismatch policy belong to its caller.

### Retain an unsupported operation

The unsupported constructor SHALL resolve the exact typed subterm at the
requested node/path. For an admitted root `Application` selected by an empty path, it SHALL clone the complete original
`CheckedApplicationOperation`, preserving identity, laws, modes and every
other retained field. For another terminal term, it SHALL retain `None` for
the application record. It SHALL neither infer an operation from display text
nor decide that an operation is unsupported. Its caller makes the support
policy decision before constructing this diagnostic payload.

### Bounded copying of an already resolved locus

The driver SHALL expose this independent component in `monitor_refusal`:

```rust
copy_monitor_locus(
    node: &CheckedNodeId,
    structural_key: &CheckedNodeId,
    body_path: &[usize],
    occurrence: Option<&CheckedOccurrence>,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<CopiedMonitorLocus, MonitorLocusCopyError>
```

`CopiedMonitorLocus` SHALL contain the existing `MonitorRefusalLocus` as
`locus`, cumulative `work: u64` and retained `storage: u64`. It is an accounting
receipt for this component, not a second location or semantic representation.
The caller supplies the two original axes after resolving their meaning;
this entry SHALL copy them without a package lookup, equality test, structural
identity recomputation, occurrence-membership check or path derivation. In
particular it SHALL preserve the entire supplied native path, including the
final binding child zero when supplied by target derivation. Its success
asserts copying only, not that a node/key/path/occurrence combination is valid.

The component SHALL construct the canonical `CheckedNodeId` fields directly:
fallibly copy each root's package text, path vector and `Name` text, retain
`Index` and ordinal unchanged, and share each existing `Node(Arc<CheckedNodeId>)`
by cloning that Arc. It SHALL NOT invoke the native allocating node/path Clone,
serialize a key, allocate a replacement Arc, or duplicate an Arc's subtree.
The occurrence SHALL retain its native role and ordinal without allocation.
The native `Vec<usize>` path SHALL be copied with fallible capacity reservation.
After success, the locus SHALL remain readable after every supplied buffer and
its source package have been dropped.

Before inspecting or copying each represented record, container, segment tag,
integer scalar or UTF-8 byte, the component SHALL charge work: one unit per
record/container/tag, eight per integer scalar, and one per UTF-8 byte. Each Arc
edge SHALL also incur one unit before sharing. The same schedule applies to
borrowed descendants reached for storage measurement. Traversal SHALL be
iterative and interruptible, with no semantic depth limit, whole-package scan
or uncharged scratch collection. Cancellation SHALL be polled before every
charge, including an initial zero charge and allocation reservations. The scratch
frontier SHALL grow geometrically by requesting twice its full current capacity
(or one slot from empty), using fallible exact reservation. Before any growth
reservation or storage-ledger mutation it SHALL additionally charge one work
unit per live reference that the reservation may relocate. This relocation
charge is separate from represented-material work; denied work SHALL leave
the frontier allocation, capacity, contents and storage ledger unchanged.

Before allocating new vector or text capacity, the component SHALL reserve
its logical bytes against the storage limit. Scratch and retained allocations
SHALL share one live ledger. Vector charge SHALL include capacity times native
element size; boxed text charge SHALL include its UTF-8 length. Before retaining
an existing Arc edge, the component SHALL charge its reachable native backing
storage: each descendant node allocation, package text, path capacity and Name
text. Shared descendants SHALL be counted per incoming edge, conservatively,
without a deduplication index; the two root copies are accounted independently.
The Arc control-block allocator overhead, like allocator bookkeeping for other
allocations, is outside logical storage. Counting shared backing storage SHALL
not trigger any copy of that storage. Cleanup SHALL release scratch and failed
partial output without allocating or polling cancellation. The success receipt
SHALL exclude released scratch; the peak live ledger SHALL never exceed the
allowance for published output. An allocator's excess reported capacity SHALL
be charged before that capacity is retained or used.

Text copying SHALL use fallible reservation and a conversion to native
`Box<str>` that performs no further allocation. An implementation using
`String::into_boxed_str` SHALL first require capacity equal to length; it SHALL
refuse excess capacity rather than invoke an infallible shrink allocation.
This explicit portability refusal is `Capacity { requested, actual }`, not an
input-semantic refusal. No allocating Clone, infallible reserve/collect or
post-clone charge SHALL substitute for the bounded construction path.

`MonitorLocusCopyErrorCode` SHALL expose stable Copy categories `Work`,
`Storage`, `Allocation`, `Capacity` and `Cancelled`, with exact lowercase
`as_str`, complete `all` and exact `from_code` catalog accessors. Each failure
SHALL expose its category through a nonallocating `code` accessor without
formatting or copying the native cause. Unknown spellings SHALL return None.

`MonitorLocusCopyError` SHALL distinguish `Work { limit, consumed, next }`,
`Storage { limit, consumed, next }`, `Allocation(TryReserveError)`,
`Capacity { requested, actual }` and `Cancelled(CancelCause)`. Arithmetic
capacity/charge overflow SHALL return Storage before allocation, using
`next: u64::MAX`; it SHALL NOT wrap or truncate. Allocation and cancellation
SHALL retain their original native causes. These errors SHALL contain no owned
node/key/path/operation or formatted error text whose construction would itself
require copying a diagnostic. Every failure SHALL return no partial locus and
leave all supplied material unchanged.

### Unqualified native ownership boundaries

The bounded copy entry SHALL NOT call `target_mismatch_at` or `unsupported_at`
as a hidden prepass. Those existing constructors perform package resolution,
path derivation, membership scans and owned error construction outside this
component's accounting. They remain functional diagnostic evidence only.

Complete bounded diagnostic construction remains Planned. The IR/model owner
must supply an allocation/work/storage-aware native copy or sharing API for
`CheckedApplicationOperation`: its private retained record, execution point
and admitted views cannot be reconstructed by this driver, and its public
infallible Clone is not a bounded-copy API. Native callable/type-projection
failure retention likewise needs the corresponding IR/model owner APIs.
The driver owns later budget threading into target derivation, selection,
unsupported and projection failure assembly and outer error boxing; neither
this helper nor its receipt qualifies those paths. Caller-input acquisition
remains caller-owned and unaccounted by this component. The complete preparer,
TypeEnvironment, Layout and RT Program handoff remain Planned under
[FR-015](FR-015-prepare-an-admitted-monitor-program.md) and
[FR-017](FR-017-lower-and-retain-admitted-monitor-programs.md), with IR/model
owning opaque native copies and RT owning its Program construction. No
replacement operation DTO, copied semantic vocabulary or compatibility layer
SHALL bypass a missing native owner API.

### Composition boundary

The outer `MonitorPreparationError` remains owned by
[FR-015](FR-015-prepare-an-admitted-monitor-program.md). Preparation SHALL retain
these native payloads in its selection/unsupported categories without parsing
or replacing them with human-readable text. This requirement introduces no
request/witness wrapper, executable plan DTO, type environment or evaluator.
The constructors SHALL execute no source parser, initializer, transition or RT
build. They SHALL NOT collect or sort a whole-package graph index, retain a
second graph, or recompute content identities to construct a diagnostic.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-016-AC-1 | For an actual strictly admitted node, the locus retains the supplied selector and exactly its package-resolved structural key as distinct values, plus the original path and optional occurrence; its owned fields remain readable after package/input destruction. | Test |
| FR-016-AC-2 | Aggregate/application source-order children and binding child zero resolve their actual typed subterms. An unavailable child, nonzero binding child or continuation past a reference returns InvalidBodyPath; selecting the referenced original node starts a separate path. | Test |
| FR-016-AC-3 | On a genuinely admitted monitor root, Every TargetPremise resolves the value of its own actual named binding, including the final Binding.value child zero, without comparing or changing that value. Uncertainty resolves its actual uncertainty value path; BufferCapacity resolves the chosen capacity value path, including its final child zero and original row owner. | Test |
| FR-016-AC-4 | BufferCapacity without an index, an out-of-range buffer index, and an index supplied for another premise each return InvalidPremiseIndex, with no target payload. A known ordinary node used as a root returns NotMonitorRoot; an unresolved node returns UnknownNode. | Test |
| FR-016-AC-5 | Each selected actual root application at an empty path yields its entire original CheckedApplicationOperation and caller-resolved containing-node kind. Selecting a different root operation retains its different original record. Admitted nonempty paths select actual nonapplication terms and yield None; reference continuations fail rather than fabricate nested application coverage. Nested Applications are forbidden by the current owner admission contract. No support-policy verdict is inferred. | Test |
| FR-016-AC-6 | An actual admitted role/ordinal is retained exactly; a nonmember occurrence returns OccurrenceMismatch. A genuine generated root succeeds with caller-supplied None even when it retains a Generated occurrence, without selecting a source-map row or inventing a selected occurrence. | Test |
| FR-016-AC-7 | Construction retains typed error distinctions and returns no partial diagnostic on failure. Neither constructor changes the admitted package, target values, operation records or source correspondence. | Test |
| FR-016-AC-8 | The diagnostic module uses native IR identities/terms and native Vec child paths, introduces no plan/target/expression DTO or evaluator, and performs no parse, initialization, runtime build, support-policy decision, whole-graph index collection or identity recomputation. | Inspection |
| FR-016-AC-9 | The bounded copy entry returns the existing canonical locus with both original node axes, native path and exact optional occurrence; after source destruction, shared nested Arc keys and copied text/path fields remain usable. It performs no resolution or validity claim. | Test |
| FR-016-AC-10 | Independently measured work and peak logical storage permit exact-limit success; one-under work/storage and pre-cancellation return their distinct original typed operating causes with no locus. Nested/shared keys, UTF-8 text and long native paths are included; arithmetic overflow refuses before allocation. | Test |
| FR-016-AC-11 | Every allocating copy/scratch reservation is fallible and precharged; reservation failure preserves TryReserveError, excess text capacity returns Capacity before conversion, and cleanup releases partial output without allocating. No allocating native clone or infallible boxed-text shrink hides behind a successful receipt. | Test |
| FR-016-AC-12 | Source inspection confirms opaque operation/native failure clones, existing diagnostic derivation and error boxing are outside bounded-copy coverage; no bounded whole-preparation claim or replacement native operation representation is introduced. | Inspection |

## Qualification Status

The independent `copy_monitor_locus` component implements AC-9 through AC-11;
focused tests cover native ownership, exact independently calculated work/storage,
operating refusals, deep/shared and wide-frontier keys, precharged geometric
relocation, stable error categories and private reservation boundaries. AC-12
has source-inspection evidence only. The copy path calls neither existing
constructor, retains Arc backing without cloning its subtree, and constructs
boxed text only after the exact-capacity guard. Its receipt covers this component
only; bounded diagnostic integration remains Planned.

The four original focused `monitor_refusal` tests execute the diagnostic-constructor
controls of AC-1 through AC-7 through public QSL checking/emission and strict
IR admission. They cover all eight target-premise value paths and buffer
ownership, complete original root-application data, admitted nonapplication
child paths and reference boundaries,
actual Generated/Expression occurrence distinctions, caller-selected None,
typed failures, unchanged inputs and owned payloads after package destruction.
AC-8 has source-inspection evidence only: the diagnostic module calls no parser,
initializer or RT build, introduces no semantic DTO or support-policy classifier,
and constructs no whole-graph index or replacement identity. No Test trace
claims that inspection criterion.

This evidence establishes diagnostic derivation and retention only. Healthy
preparation, target-admission decisions, RT construction, temporal execution
and whole FR-276 acceptance remain unqualified.

## Dependencies

- [FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns independent
  selection comparison, runtime-support policy, the preparation request and
  outer failure categories that consume these diagnostics.
- [IR FR-038](ix://agent-ix/quire-contract-ir/FR-038) owns strict admission,
  canonical checked terms, structural-key resolution and retained occurrences.
- [TC-030](../test-cases/TC-030-monitor-refusal-loci.md) verifies the diagnostic
  constructors without granting full monitor qualification.
