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
metadata lookup or another supported-operation classifier. For an inline
term, `kind` remains the containing node's kind; the optional application
record identifies the inline operation independently.

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
retain `None`, including for genuine generated monitor roots without source
occurrences. No first source-map row or fabricated ordinal supplies an absent
occurrence; source-map absence alone does not invalidate an admitted occurrence.

### Derive a target-premise location

The target constructor SHALL resolve `root` as an actual monitor root of the
selected admitted package. A known node outside that set SHALL return
`NotMonitorRoot`.

The constructor SHALL derive the path to the selected premise's actual named
binding in the admitted root body. `BufferCapacity` SHALL derive the path to
the selected buffer row's `capacity` binding without substituting a numeric
subformula index for its original owner. Missing, excessive or inappropriate
buffer indices SHALL return `InvalidPremiseIndex`. A missing named child or
an incompatible term path SHALL return `InvalidBodyPath` rather than inventing
a default premise. The constructor SHALL leave all premise values unchanged;
independent target equality and first-mismatch policy belong to its caller.

### Retain an unsupported operation

The unsupported constructor SHALL resolve the exact typed subterm at the
requested node/path. For an `Application`, it SHALL clone the complete original
`CheckedApplicationOperation`, preserving identity, laws, modes and every
other retained field. For another terminal term, it SHALL retain `None` for
the application record. It SHALL neither infer an operation from display text
nor decide that an operation is unsupported. Its caller makes the support
policy decision before constructing this diagnostic payload.

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
| FR-016-AC-3 | On a genuinely admitted monitor root, Uncertainty resolves the actual named uncertainty value path and BufferCapacity resolves the chosen capacity path/owner. Every TargetPremise chooses its own named binding without comparing or changing its value. | Test |
| FR-016-AC-4 | BufferCapacity without an index, an out-of-range buffer index, and an index supplied for another premise each return InvalidPremiseIndex, with no target payload. A known ordinary node used as a root returns NotMonitorRoot; an unresolved node returns UnknownNode. | Test |
| FR-016-AC-5 | A selected actual application yields its entire original CheckedApplicationOperation and the caller-resolved containing-node kind. A nonapplication terminal yields None. Changing the selected inline operation or path changes the observation; no support-policy verdict is inferred. | Test |
| FR-016-AC-6 | An actual admitted role/ordinal is retained exactly; a nonmember occurrence returns OccurrenceMismatch. A genuine generated root without an occurrence succeeds with None without selecting a source-map row. | Test |
| FR-016-AC-7 | Construction retains typed error distinctions and returns no partial diagnostic on failure. Neither constructor changes the admitted package, target values, operation records or source correspondence. | Test |
| FR-016-AC-8 | The diagnostic module uses native IR identities/terms and native Vec child paths, introduces no plan/target/expression DTO or evaluator, and performs no parse, initialization, runtime build, support-policy decision, whole-graph index collection or identity recomputation. | Inspection |

## Qualification Status

Planned until the public constructors and
[TC-030](../test-cases/TC-030-monitor-refusal-loci.md) have executed. Passing this
requirement establishes diagnostic derivation and retention only. It supplies
no healthy preparation, target-admission decision, RT construction, temporal
execution or whole FR-276 acceptance credit.

## Dependencies

- [FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns independent
  selection comparison, runtime-support policy, the preparation request and
  outer failure categories that consume these diagnostics.
- [IR FR-038](ix://agent-ix/quire-contract-ir/FR-038) owns strict admission,
  canonical checked terms, structural-key resolution and retained occurrences.
- [TC-030](../test-cases/TC-030-monitor-refusal-loci.md) verifies the diagnostic
  constructors without granting full monitor qualification.
