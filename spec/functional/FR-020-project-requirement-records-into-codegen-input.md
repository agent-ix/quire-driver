---
id: FR-020
title: "Project each QSL requirement record into CG's typed routed-generation input"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-driver/FR-001
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-010
    type: depends_on
  - target: ix://agent-ix/quire-contract-codegen/FR-022
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: depends_on
---
# FR-020: Project each QSL requirement record into CG's typed routed-generation input

## Description

One application node can carry several scalar occurrences, each with its own
result bound and path condition (QSL-266, QSL-700, QSL-904). The node id
alone cannot say which claim an item makes. For each routed Kani item the
driver SHALL hand CG the QSL record of that exact occurrence: the item's
`RoutedRequirement`, and the package's `RequirementRecords` on the Kani
generation context. A record that cannot be projected exactly is a typed
driver fault, never a guess. CG owns the types and every comparison
([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), "Per-occurrence
requirement records", "Frames" and "Where the checks run"); the driver owns
the projection and the refusal when it cannot make it.

The CG types are merged (IR-304 slice (a), on CG main) and the driver
implements this requirement against them. AC-11 and AC-13 wait on IR-304
slice (b), which CG has not landed; the guard-node half of AC-14 has no
test (see the AC-14 row).

## Inputs

- The QSL `RequirementItem` at each request index
  (`qsl_route::request::RequirementItem`): `occurrence()`, `result_bound()`
  (a QSL `NodeKey`), `conversion_context()` and `path_condition()`
  (ordered `PathGuard`s, each an `OccurrenceKey` and `holds`).
- The compiled package's checked requirement records
  (`CheckedGraph::requirements()`, a `RequirementMap` keyed by
  `OccurrenceKey` and `Capability`: one record per occurrence and
  capability, and different capabilities may share one occurrence, for
  example a temporal-satisfaction and an operation-contract record, QSL
  `RequirementMap`). Each record has a `ResultBound::node()`, a
  `conversion_context` and a `path_condition`. CG's `RequirementRecords`
  is keyed by occurrence alone, so only one capability projects: the
  `value-validity` records, which are the scalar claims a `value-validity`
  Kani item makes. CG's `V2ScalarClaim` preflight requires an
  `Expression`-role `OwnResult` occurrence, and QSL keys the
  `operation-contract` and `temporal-satisfaction` records at `Claim`-role
  occurrences, so an item of those capabilities settles `InvalidRequest` at
  negotiation as it does today and is not routed to generation.
- The E5-admitted `CheckedPackageV2`, whose source frame is the one every
  projected node id is expressed in.
- FR-010's partition: which items the export bound and which it refused.

## Outputs

- For each export-bound `value-validity` item that has a record, selected
  with `OwnResult`, a CG
  `RoutedRequirement { occurrence, result_bound, conversion_context,
  path_condition }` is projected, and attached with
  `RoutedGenerationItem::with_requirement` to the generation item of each
  such item that negotiation routes to Kani.
- On `KaniGenerationContext.requirement_records`, one CG `RequirementRecords`
  holding one `RequirementRecordInput { result_bound, conversion_context,
  path_condition }` for every `value-validity` record of the package, keyed
  by `RequirementOccurrence { node_id, role, ordinal }`.
- The requirements and the records come from one driver projection function, run in
  `lower_driver_request` over the export-bound items before negotiation,
  because the guard `ContextRoot` positions are lowering roots that bind
  CG's negotiation preflight. It takes the items, the
  package's `RequirementMap` and the admitted package as arguments and
  returns the requirement for each request index (`None` at an excluded
  position or at a bound item with no record) with the `RequirementRecords`, or a fault. Taking the map as an
  argument is what lets a test supply a map with a record removed.
- Or a `DriverRefusal::Fault` of one of two new `DriverFault` variants:
  `RequirementRecordAbsent { request_index, occurrence }` and
  `RequirementRecordUnresolved { occurrence, cause }`, where `cause` is a
  new `RequirementRecordUnresolvedCause` with the variants `UnknownNode
  { node }`, `AmbiguousNode { node }` and `UnknownOccurrenceRole { role }`.
  The existing `DriverFault` node and role variants carry a request index
  that a package record lacks, so the new cause type carries the same
  payload without it. Each has stage
  Generation, wire code `runtime_invariant` and outcome kind internal fault.
  The code is already in the vocabulary FR-001 uses for a missing retained
  association; the driver adds no catalog code.

## Behavior

- The driver SHALL project `RequirementOccurrence` from a QSL
  `OccurrenceKey` as follows: `node_id` is the key's `NodeKey` resolved to
  the admitted package's V2 `CheckedNodeId` by the same lookup FR-001 uses
  for an item's node; `role` and `ordinal` are the key's role and ordinal,
  by value. FR-001 resolves an item's own occurrence first, with its
  request-indexed faults, so the projection never sees an item's occurrence
  unresolved; a node or role of a package record that does not project is
  the `RequirementRecordUnresolved` fault with the matching cause.
- The driver SHALL NOT derive an occurrence from a node id alone, and the
  driver SHALL NOT carry a hash or digest of a key.
- The driver SHALL build each `RoutedRequirement` from the same
  `RequirementItem` that produced that request index: `occurrence` from the
  item's occurrence; `result_bound` from the item's result-bound `NodeKey`;
  `conversion_context` from the item's conversion occurrence (`None` only
  when the item has none); and `path_condition` from the item's guards, one
  `RequirementGuard { occurrence, holds }` per `PathGuard`, in QSL's
  outermost-first order with each `holds` value unchanged.
- The driver SHALL build `RequirementRecords` from every `value-validity`
  record of the compiled package's own `requirements()`, projecting each
  record's `ResultBound::node()`, conversion occurrence and path condition
  exactly as for an item. It SHALL NOT project a record of another
  capability, because two capabilities can share one occurrence and CG's map
  holds one record per occurrence. It SHALL NOT build a record from an
  item, from the checked graph, or from any source other than QSL's record.
- The driver SHALL find a `value-validity` item's record at the item's
  occurrence and the `value-validity` capability (`get_kind`), never by
  occurrence alone.
- **Frames.** The driver SHALL express every projected node id in the V2
  source frame of the admitted package, as it does for the item's node. QSL's
  `ResultBound::node()` is the type node QSL checked; the driver SHALL pass
  it through that mapping and SHALL NOT read the IR result context's
  `selected_type`, translate it to the represented V1 frame, or compare it
  with the QSL bound. CG makes that comparison, and the driver SHALL preserve
  a disagreement as CG's `ResultBoundMismatch`.
- The projection SHALL build a `RoutedRequirement` for each export-bound
  item (FR-010), selected with `OwnResult` as FR-001 states (FR-001's
  selection is unchanged), whose occurrence has a `value-validity` record.
- The projection SHALL return `None` at the request index of any other
  item: an export-refused item (an excluded position), or a bound item with
  no `value-validity` record, such as one of another capability.
- The projection SHALL raise no fault for a `None`.
- The projection SHALL add no guard root for an item whose projection is
  `None`.
- The driver SHALL attach the projected `RoutedRequirement` to each
  generation item for the subset negotiation routes to Kani, and SHALL NOT
  attach one to an item that is not routed.
- The driver SHALL add one `ContextRoot` position for each path guard,
  after any conversion root and in outermost-first order, inside that item's
  contiguous context span, as CG FR-022 "Path conditions in the scalar
  harness" requires. Context roots SHALL NOT advance driver indices
  (FR-001). Where two items share a guard node (the then and else
  occurrences of `if p then x + 1 else x + 1` are both guarded by `p`), the
  driver SHALL add that root in each item's own span.
- The driver SHALL NOT emit a shared guard root once for several items,
  because CG checks a guard against its own item's span.
- If an item negotiation routes to Kani has a `None` projection, then the
  driver SHALL refuse the run with the `RequirementRecordAbsent` fault,
  naming the request index and the item's occurrence, when it builds the
  generation items and before calling CG generation. The fault fires only
  for a routed item, so a bound item that settles `InvalidRequest` at
  negotiation never faults.
- If a `value-validity` record of the package fails to project (a node of
  its key, result bound, conversion occurrence or a guard occurrence is
  unknown or ambiguous in the admitted package, or a role is unknown), then
  the driver SHALL refuse the run with the `RequirementRecordUnresolved`
  fault, naming the record's QSL occurrence key (a package record has no
  request index) and carrying the matching `RequirementRecordUnresolvedCause`. It SHALL NOT omit the record and continue, whether or not an
  item routes to it.
- The driver SHALL NOT fabricate, default, repair or substitute a record or
  requirement in any case above, SHALL NOT choose another record by node id
  or by position, and SHALL NOT call CG generation with a partial set.
- A CG `RoutedGenerationError::RequirementRecord { request_index,
  occurrence, mismatch }` (any `RequirementRecordMismatch`:
  `RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`,
  `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`
  or `PathConditionMismatch` from IR-304 slice (a), and `GuardContextMissing`
  from IR-304 slice (b)) means the driver's
  projection disagreed with the authenticated selection. The driver SHALL
  preserve it unchanged in `DriverRefusal::Generation`, with stage
  Generation, wire code `runtime_invariant` and outcome kind internal fault.
- The driver SHALL preserve CG's per-item settlement of an item whose guard
  CG does not lower (`path_guard_not_lowered`, and in slice (a)
  `GuardLoweringNotImplemented`) unchanged; it SHALL NOT remove a guard to
  avoid that settlement.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-020-AC-1 | For a routed item whose QSL record has a conversion context and a two-guard path condition (one guard requiring `holds = true`, one `holds = false`), the `RoutedRequirement` passed to CG carries that item's occurrence (node, role and ordinal), the V2 node of its result bound, the conversion occurrence, and the two guards in QSL's order with the same `holds` values. For an item with no conversion context it carries `None`. | Test (TC-034) |
| FR-020-AC-2 | `RequirementRecords` on the Kani context holds one entry per `value-validity` record of the package's `requirements()`, including a record whose item the export refused, each with its own result bound, conversion occurrence and ordered path condition, and holds no entry for a record of another capability. | Test (TC-034) |
| FR-020-AC-3 | For a built unit in which one `x + 1` node's then and else occurrences are guarded by `p` and `not p` and converted to `Int[0, 10]` and `Int[0, 20]` (`function f using v(p: Boolean, x: Int[0, 9]): Boolean pure { if p then g(x + 1) else h(x + 1) }` with `g` over `Int[0, 10]` and `h` over `Int[0, 20]`, built on the RR16_UNIT shape), the two occurrences differ in ordinal, result bound and path condition. The two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries with different result bounds and path conditions, and each item's requirement equals the entry at its own key. If QSL emits two nodes for the two `x + 1`, the test fails and the unit is adjusted until one node carries both occurrences. | Test (TC-034) |
| FR-020-AC-4 | With a `RequirementMap` that holds no `value-validity` record for the occurrence of an item negotiation routes to Kani, the run completes the lowering and negotiation steps and then refuses, when the generation items are built, with `DriverFault::RequirementRecordAbsent`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record. | Test (TC-034) |
| FR-020-AC-5 | Given a `RequirementMap` built with `FromIterator` that holds, at one occurrence, both a `value-validity` record and an `operation-contract` record with a different result bound, `RequirementRecords` holds exactly the `value-validity` record's fields at that key, the `value-validity` item's requirement equals them, and no fault is raised. A lookup by occurrence alone finds the key ambiguous and returns no record, so that lookup raises `RequirementRecordAbsent` here. | Test (TC-034) |
| FR-020-AC-6 | For the FR-010-AC-4 vector, the projection returns a requirement for A (index 0) and C (index 2) and `None` at B's index 1, B's position is excluded, `RequirementRecords` still holds B's record, the run completes without a `DriverRefusal`, and the same holds with B's record removed from the map. FR-001-AC-17 keeps its own evidence in TC-017; this criterion asserts only the projection and the absence of a fault. | Test (TC-034) |
| FR-020-AC-7 | For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them. | Test (TC-034) |
| FR-020-AC-8 | Given a record whose QSL result-bound node differs from the type the IR result context selects, the driver passes the QSL node unchanged; CG refuses with `RequirementRecord` and `ResultBoundMismatch`, and the driver returns it unchanged in `DriverRefusal::Generation` with stage Generation, wire code `runtime_invariant` and kind internal fault. | Test (TC-034) |
| FR-020-AC-9 | For each of the seven `RequirementRecordMismatch` variants IR-304 slice (a) defines (`RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`, `PathConditionMismatch`), a CG refusal carrying it is returned as `DriverRefusal::Generation` with the request index, occurrence and mismatch unchanged, stage Generation, wire code `runtime_invariant` and outcome kind internal fault. | Test (TC-034) |
| FR-020-AC-10 | Slice (a) only: over the IR-304 guarded package, the unguarded item `g` generates a harness, and each guarded item (`f` then item, `h`) settles `unsupported` with `path_guard_not_lowered` and reason `guard_lowering_not_implemented`; no guarded item has a harness. This criterion retires when slice (b) lands, as CG FR-022-AC-38 does. | Test (TC-034) |
| FR-020-AC-11 | After IR-304 slice (b): over the same package the `h` item generates a harness whose guard `Assume` follows the symbolic inputs, and its identity's `requirement_occurrence` equals the item's occurrence; an item with a guard CG cannot lower settles `unsupported` with `path_guard_not_lowered` and no harness omits a guard. | Test (TC-034; Planned until IR-304 slice (b) lands) |
| FR-020-AC-12 | For `if p then x + 1 else x + 1`, whose then and else items are both guarded by the node of `p`, the positions hold a `ContextRoot` for `p` in the then item's span and another in the else item's span, and the root is not emitted once for both. | Test (TC-034) |
| FR-020-AC-13 | After IR-304 slice (b): a CG refusal `RequirementRecord` with `GuardContextMissing` is returned as `DriverRefusal::Generation` with its request index, occurrence and guard unchanged, stage Generation, wire code `runtime_invariant` and outcome kind internal fault. | Test (TC-034; Planned until IR-304 slice (b) lands) |
| FR-020-AC-14 | When a `value-validity` record of the package has a guard occurrence whose node the admitted package does not hold, the run refuses with `DriverFault::RequirementRecordUnresolved` naming the record's QSL occurrence key with cause `UnknownNode`; when a record's key has a role outside IR's occurrence vocabulary, the cause is `UnknownOccurrenceRole`. Each has stage Generation, wire code `runtime_invariant` and outcome kind internal fault, is raised even when no item routes to that record, and CG generation is not called. The `AmbiguousNode` cause is produced by the same node lookup FR-001 uses and cannot be built through IR admission, so no criterion constructs it. The guard-node case (the first example) is Planned: QSL's `PathGuard` and `RequirementRecord` have no public constructor, so no test can give a record a guard whose node the package lacks; the test covers the record-key node and role instead. | Test (TC-034; guard-node case Planned, QSL-owned) |
| FR-020-AC-15 | For a bound item of another capability (a model-reading `operation-contract` item) whose occurrence has no `value-validity` record in the map, negotiation settles it `InvalidRequest` as before, it is absent from the generation items, the projection returns `None` at its index and adds no guard root for it, no `RequirementRecordAbsent` fault is raised at any step, and a `value-validity` item in the same request still receives its requirement. | Test (TC-034) |

## Dependencies

- [CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022) owns
  `RequirementOccurrence`, `RequirementGuard`, `RequirementRecordInput`,
  `RequirementRecords`, `RoutedRequirement`, `RoutedGenerationItem::with_requirement`,
  `KaniGenerationContext.requirement_records` and
  `RoutedGenerationError::RequirementRecord`. The spec and the slice (a) code are
  merged on CG main, and the driver builds against them.
- IR-304 slice (b) (ticket IR-304, reopened as its owner) lowers guards to
  `Assume` statements and adds `GuardContextMissing` and the
  `requirement_occurrence` identity field. The projection does not depend on
  it; AC-11 and AC-13 do. CG main has not landed slice (b).
- Other capabilities are unchanged: their items settle at negotiation as
  today and this requirement adds nothing for them.
- IR-1504 owns how the driver tracks CG (both manifests) and the dependency
  reference; this requirement sets no pin policy and adds no hash, digest or
  pin.
- IR-1494 is the driver ticket for this requirement. The driver code and
  tests are a later, separate change.
- QSL-266, QSL-700 and QSL-904 own the per-occurrence requirement record and
  its `RequirementItem` fields.
- [FR-001](FR-001-orchestrate-the-driver-steps.md) owns the position vector,
  node lookup and runtime-invariant vocabulary;
  [FR-010](FR-010-send-only-bound-items-to-codegen.md) owns which items are
  bound.
- [TC-034](../test-cases/TC-034-project-requirement-records-into-codegen.md).
