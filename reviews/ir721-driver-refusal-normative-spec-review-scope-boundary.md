---
id: SR-2026110414560031
title: IR-721 PR118 normative spec-review/scope-boundary
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-driver; PR118 normative; spec/functional/FR-016-retain-monitor-refusal-loci.md;
  spec/test-cases/TC-030-monitor-refusal-loci.md
review_set: subset
---

## Summary

IR-721 driver draft PR118; normative specification-first review at frozen 3c68a705172924059ca99c540baab8a18292ba8b. Only FR-016 and TC-030 are added; no source/exports exist yet. Agreed interface contract and FR-015 ownership were read. No applicable AssuranceProfile found. This review authorizes no source implementation until its normative findings are resolved.

## Verdict

PASS within the normative changed scope. Implementation awaits resolution of the base-review finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Interface boundary: monitor_refusal owns diagnostic derivation/retention (cross-cutting error diagnostics). Contract IR owns strict admission, structural keys, terms, monitor-root views and occurrence membership; these are assumed owner contracts, while future TC030 validates this consumer preserves them. FR015 owns independent target comparison, support decisions, request/witness holders and outer error wrapping; RT owns Program/build/execution. No helper may grant admission, choose a fallback operation, follow references, prepare runtime owners or reinterpret temporal outcomes. Native supplied containing-node kind remains a documented caller precondition rather than a second metadata classifier. The interface endpoint ambiguity is recorded in base review. No source file custody expansion is proposed.

## Verification limits

No Cargo, test, gate, global advisor/census/matrix, solver or Kani execution. Scoped Quire 0.38.3 validation of the two specifications exits 0; engine summary has no EARS statement diagnostic and two ac:vague-response lexical warnings on “support” within explicit no-support-policy clauses. These clauses are concrete prohibitions, so the lexical hits are not semantic defects. The full report and engine summary are retained outside the branch. Structural validation does not establish implemented evidence. FR-016 and TC-030 stay Planned and supply no full FR-015/FR-276 preparation or temporal acceptance.

## Examined units

```yaml
scope:
- id: FR-016-AC-1
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: For an actual strictly admitted node, the locus retains the supplied selector
    and exactly its package-resolved structural key as distinct values, plus the original
    path and optional occurrence; its owned fields remain readable after package/input
    destruction.
- id: FR-016-AC-2
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Aggregate/application source-order children and binding child zero resolve
    their actual typed subterms. An unavailable child, nonzero binding child or continuation
    past a reference returns InvalidBodyPath; selecting the referenced original node
    starts a separate path.
- id: FR-016-AC-3
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: On a genuinely admitted monitor root, Uncertainty resolves the actual named
    uncertainty value path and BufferCapacity resolves the chosen capacity path/owner.
    Every TargetPremise chooses its own named binding without comparing or changing
    its value.
- id: FR-016-AC-4
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: BufferCapacity without an index, an out-of-range buffer index, and an index
    supplied for another premise each return InvalidPremiseIndex, with no target payload.
    A known ordinary node used as a root returns NotMonitorRoot; an unresolved node
    returns UnknownNode.
- id: FR-016-AC-5
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: A selected actual application yields its entire original CheckedApplicationOperation
    and the caller-resolved containing-node kind. A nonapplication terminal yields
    None. Changing the selected inline operation or path changes the observation;
    no support-policy verdict is inferred.
- id: FR-016-AC-6
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: An actual admitted role/ordinal is retained exactly; a nonmember occurrence
    returns OccurrenceMismatch. A genuine generated root without an occurrence succeeds
    with None without selecting a source-map row.
- id: FR-016-AC-7
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Construction retains typed error distinctions and returns no partial diagnostic
    on failure. Neither constructor changes the admitted package, target values, operation
    records or source correspondence.
- id: FR-016-AC-8
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: The diagnostic module uses native IR identities/terms and native Vec child
    paths, introduces no plan/target/expression DTO or evaluator, and performs no
    parse, initialization, runtime build, support-policy decision, whole-graph index
    collection or identity recomputation.
- id: FR-016-Description
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: 'The driver SHALL expose diagnostic constructors that retain the original

    admitted locus of a target-premise mismatch or an unsupported ordinary

    operation. These constructors serve the outer preparation failures of

    [FR-015](FR-015-prepare-an-admitted-monitor-program.md); they do not prepare a

    program, compare independent target selections, or decide runtime support.

    Their results are diagnostic values, not admission capabilities or plan IDs.'
- id: FR-016-Inputs
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: "The public `monitor_refusal` module SHALL expose these entries:\n\n```rust\n\
    target_mismatch_at(\n    package: &CheckedPackageV2,\n    root: &CheckedNodeId,\n\
    \    premise: TargetPremise,\n    buffer_index: Option<usize>,\n    occurrence:\
    \ Option<&CheckedOccurrence>,\n) -> Result<TargetMismatch, MonitorLocusError>\n\
    \nunsupported_at(\n    package: &CheckedPackageV2,\n    node: &CheckedNodeId,\n\
    \    body_path: &[usize],\n    kind: CheckedNodeKind,\n    occurrence: Option<&CheckedOccurrence>,\n\
    ) -> Result<UnsupportedOperation, MonitorLocusError>\n```\n\nAll `Checked*` types\
    \ are the canonical Contract IR types. The caller"
- id: FR-016-Outputs
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: '`TargetMismatch` SHALL retain its typed `premise` and owned `locus`.

    `UnsupportedOperation` SHALL retain its owned `locus`, original caller-resolved

    `CheckedNodeKind`, and `operation: Option<CheckedApplicationOperation>`.


    `MonitorRefusalLocus` SHALL retain these distinct original axes:


    - `node`: the supplied original admitted `CheckedNodeId` selector;

    - `structural_key`: the exact key resolved by `package.structural_key(node)`;

    - `body_path`: the native `Vec<usize>` child path within that node; and

    - `occurrence`: the supplied exact `Option<CheckedOccurrence>`.


    The helper SHALL clone the re'
- id: FR-016-Behavior
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: '### Resolve the original locus


    When the supplied node cannot resolve through the selected package, the helper

    SHALL return `UnknownNode` before attempting body traversal. A graph-local

    selector and its resolved structural key SHALL remain separate fields.


    For a nonempty `body_path`, the helper SHALL visit `Aggregate.members` and

    `Application.arguments` by source-order child index, and `Binding.value` by

    child index zero. An unavailable child or a continuation into any other term

    SHALL return `InvalidBodyPath`. A `Reference` ends the current node/path;

    following it requires the caller to sele'
- id: FR-016-Qualification Status
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: 'Planned until the public constructors and

    [TC-030](../test-cases/TC-030-monitor-refusal-loci.md) have executed. Passing
    this

    requirement establishes diagnostic derivation and retention only. It supplies

    no healthy preparation, target-admission decision, RT construction, temporal

    execution or whole FR-276 acceptance credit.'
- id: FR-016-Dependencies
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: "- [FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns independent\n\
    \  selection comparison, runtime-support policy, the preparation request and\n\
    \  outer failure categories that consume these diagnostics.\n- [IR FR-038](ix://agent-ix/quire-contract-ir/FR-038)\
    \ owns strict admission,\n  canonical checked terms, structural-key resolution\
    \ and retained occurrences.\n- [TC-030](../test-cases/TC-030-monitor-refusal-loci.md)\
    \ verifies the diagnostic\n  constructors without granting full monitor qualification."
- id: TC-030-Description
  path: spec/test-cases/TC-030-monitor-refusal-loci.md
  role: examined
  excerpt: 'Verify the functional diagnostic constructors of

    [FR-016](../functional/FR-016-retain-monitor-refusal-loci.md) against actual

    strictly admitted checked nodes and body terms. The test does not accept a

    payload merely because an enum can be constructed or displayed.'
- id: TC-030-Test Procedure
  path: spec/test-cases/TC-030-monitor-refusal-loci.md
  role: examined
  excerpt: "1. Obtain ordinary checked application nodes through the existing public\
    \ QSL\n   compiler and strict IR reader. Obtain a complete valid monitor root\
    \ with\n   a buffer from the public producer/reader path or an existing owner\
    \ fixture\n   consumed by reference. Retain original selectors, structural keys,\
    \ typed\n   terms and actual occurrence lists. Do not copy an upstream fixture\
    \ corpus,\n   fabricate a semantic ID or use a fake preparation entry.\n2. Derive\
    \ the root's uncertainty locus and each permitted target-premise\n   location;\
    \ derive a buffer capacity location from its actual row. Independently"
- id: TC-030-Expected Results
  path: spec/test-cases/TC-030-monitor-refusal-loci.md
  role: examined
  excerpt: 'Every applicable FR-016 criterion returns the exact derived original locus,

    original typed operation data or typed constructor error. A wrong path,

    foreign occurrence, reminted identity or operation inferred from a label fails

    the test. Status is Planned until execution; this case establishes no full

    FR-015 preparation, FR-276 construction or temporal-execution acceptance.'
- id: IR721-agreed-contract
  path: /tmp/ir721-refusal-contract.md
  role: context_only
  excerpt: 'Agreed path convention: native Vec<usize> as used by RT Expression.body_path.
    Aggregate.members and Application.arguments use source-order child indices; Binding.value
    uses child0.'
```
