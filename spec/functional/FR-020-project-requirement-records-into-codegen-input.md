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

This requirement specifies code that depends on CG types not yet merged
(IR-304 slice (a), CG code PR #772). Its criteria are Planned until that PR
lands; the type names below are those CG FR-022 uses today and follow CG if
review renames them.

## Inputs

- The QSL `RequirementItem` at each request index
  (`qsl_route::request::RequirementItem`): `occurrence()`, `result_bound()`
  (a QSL `NodeKey`), `conversion_context()` and `path_condition()`
  (ordered `PathGuard`s, each an `OccurrenceKey` and `holds`).
- The compiled package's checked requirement records
  (`CheckedGraph::requirements()`, one per scalar application occurrence,
  keyed by `OccurrenceKey`): each record's `ResultBound::node()`,
  `conversion_context` and `path_condition`.
- The E5-admitted `CheckedPackageV2`, whose source frame is the one every
  projected node id is expressed in.
- FR-010's partition: which items the export bound and which it refused.

## Outputs

- For each routed Kani item selected with `OwnResult`, a CG
  `RoutedRequirement { occurrence, result_bound, conversion_context,
  path_condition }`, attached with `RoutedGenerationItem::with_requirement`.
- On `KaniGenerationContext.requirement_records`, one CG `RequirementRecords`
  holding one `RequirementRecordInput { result_bound, conversion_context,
  path_condition }` for every record of the package, keyed by
  `RequirementOccurrence { node_id, role, ordinal }`.
- Or a `DriverRefusal::Fault(DriverFault::RequirementRecord { request_index,
  occurrence, cause })` where `cause` is `RecordMissing` or
  `RecordDuplicate`. It has stage Generation, wire code `runtime_invariant`
  and outcome kind internal fault. The code is already in the vocabulary
  FR-001 uses for a missing or duplicate retained association; the driver
  adds no catalog code.

## Behavior

- The driver SHALL project `RequirementOccurrence` from a QSL
  `OccurrenceKey` as follows: `node_id` is the key's `NodeKey` resolved to
  the admitted package's V2 `CheckedNodeId` by the same lookup FR-001 uses
  for an item's node; `role` and `ordinal` are the key's role and ordinal,
  by value. An unknown QSL role is the existing `UnknownOccurrenceRole`
  fault.
- The driver SHALL NOT derive an occurrence from a node id alone, and the
  driver SHALL NOT carry a hash or digest of a key.
- The driver SHALL build each `RoutedRequirement` from the same
  `RequirementItem` that produced that request index: `occurrence` from the
  item's occurrence; `result_bound` from the item's result-bound `NodeKey`;
  `conversion_context` from the item's conversion occurrence (`None` only
  when the item has none); and `path_condition` from the item's guards, one
  `RequirementGuard { occurrence, holds }` per `PathGuard`, in QSL's
  outermost-first order with each `holds` value unchanged.
- The driver SHALL build `RequirementRecords` from every record of the
  compiled package's own `requirements()`, projecting each record's
  `ResultBound::node()`, conversion occurrence and path condition exactly as
  for an item. It SHALL NOT build a record from an item, from the checked
  graph, or from any source other than QSL's record at that key.
- **Frames.** The driver SHALL express every projected node id in the V2
  source frame of the admitted package, as it does for the item's node. QSL's
  `ResultBound::node()` is the type node QSL checked; the driver SHALL pass
  it through that mapping and SHALL NOT read the IR result context's
  `selected_type`, translate it to the represented V1 frame, or compare it
  with the QSL bound. CG makes that comparison, and the driver SHALL preserve
  a disagreement as CG's `ResultBoundMismatch`.
- The driver SHALL attach a `RoutedRequirement` to every routed Kani item
  that FR-001 selects with `OwnResult`, which is every item the export bound
  (FR-010). It SHALL attach none to an export-refused item, whose position
  is excluded, and SHALL treat the absence of a record for such an item as
  no fault.
- The driver SHALL add one `ContextRoot` position for each path guard,
  after any conversion root and in outermost-first order, inside that item's
  contiguous context span, as CG FR-022 "Path conditions in the scalar
  harness" requires. Context roots SHALL NOT advance driver indices
  (FR-001).
- If the package's requirement records hold no entry for a routed item's
  occurrence, then the driver SHALL refuse the run with the
  `RequirementRecord` fault, cause `RecordMissing`, naming the request index
  and the item's occurrence, before calling CG generation.
- If two distinct QSL keys project to one `RequirementOccurrence`, then the
  driver SHALL refuse the run with the `RequirementRecord` fault, cause
  `RecordDuplicate`, naming the request index of the first item that reaches
  the repeated occurrence, before calling CG generation.
- The driver SHALL NOT fabricate, default, repair or substitute a record or
  requirement in any case above, SHALL NOT choose another record by node id
  or by position, and SHALL NOT call CG generation with a partial set.
- A CG `RoutedGenerationError::RequirementRecord { request_index,
  occurrence, mismatch }` (any `RequirementRecordMismatch`: `RequirementMissing`,
  `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`,
  `ResultBoundMismatch`, `ConversionContextMismatch`,
  `PathConditionMismatch` or `GuardContextMissing`) means the driver's
  projection disagreed with the authenticated selection. The driver SHALL
  preserve it unchanged in `DriverRefusal::Generation`, with stage
  Generation, wire code `runtime_invariant` and outcome kind internal fault.
- The driver SHALL preserve CG's per-item settlement of an item whose guard
  CG does not lower (`path_guard_not_lowered`, and in slice (a)
  `GuardLoweringNotImplemented`) unchanged; it SHALL NOT remove a guard to
  avoid that settlement.
- The driver's dependency on CG SHALL name CG by branch, as it names
  `qsl-route`, with no `rev`, commit hash or digest. Taking the IR-304
  types is a plain dependency update of that branch reference, not a
  carve-out.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-020-AC-1 | For a routed item whose QSL record has a conversion context and a two-guard path condition (one guard requiring `holds = true`, one `holds = false`), the `RoutedRequirement` passed to CG carries that item's occurrence (node, role and ordinal), the V2 node of its result bound, the conversion occurrence, and the two guards in QSL's order with the same `holds` values. For an item with no conversion context it carries `None`. | Test (TC-034) |
| FR-020-AC-2 | `RequirementRecords` on the Kani context holds one entry per record of the package's `requirements()`, including a record whose item the export refused, each with its own result bound, conversion occurrence and ordered path condition. | Test (TC-034) |
| FR-020-AC-3 | Given one application node with two occurrences that differ in ordinal, result bound and path condition, the two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries, and each item's requirement equals the entry at its own key. | Test (TC-034) |
| FR-020-AC-4 | When the requirement records hold no entry for a routed item's occurrence, the run refuses with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record. | Test (TC-034) |
| FR-020-AC-5 | When two distinct QSL keys project to one `RequirementOccurrence`, the run refuses with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index; stage, wire code and kind are those of FR-020-AC-4 and CG generation is not called. | Test (TC-034) |
| FR-020-AC-6 | For the FR-010-AC-4 vector, CG receives a `RoutedRequirement` for A and for C and none for B; `RequirementRecords` still holds B's record; B's absent requirement is not a fault, including when B's record is also removed from the view; and the run completes as FR-001-AC-17 states. | Test (TC-034) |
| FR-020-AC-7 | For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them. | Test (TC-034) |
| FR-020-AC-8 | Given a record whose QSL result-bound node differs from the type the IR result context selects, the driver passes the QSL node unchanged; CG refuses with `RequirementRecord` and `ResultBoundMismatch`, and the driver returns it unchanged in `DriverRefusal::Generation` with stage Generation, wire code `runtime_invariant` and kind internal fault. | Test (TC-034) |
| FR-020-AC-9 | For each `RequirementRecordMismatch` variant CG names (`RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`, `PathConditionMismatch`, `GuardContextMissing`), a CG refusal carrying it is returned as `DriverRefusal::Generation` with the request index, occurrence and mismatch unchanged, stage Generation, wire code `runtime_invariant` and outcome kind internal fault. | Test (TC-034) |
| FR-020-AC-10 | In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard. | Test (TC-034; Planned until CG PR #772, IR-304 slice (a), lands; guard `Assume` branch also needs IR-304 slice (b)) |
| FR-020-AC-11 | The driver's `Cargo.toml` entry for `quire-contract-codegen` names a branch like the `qsl-route` entry and carries no `rev`, and `Cargo.lock` content is the only record of the resolved commit. | Inspection (TC-034) |

## Dependencies

- [CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022) owns
  `RequirementOccurrence`, `RequirementGuard`, `RequirementRecordInput`,
  `RequirementRecords`, `RoutedRequirement`, `RoutedGenerationItem::with_requirement`,
  `KaniGenerationContext.requirement_records` and
  `RoutedGenerationError::RequirementRecord`. The spec is merged; the code is
  IR-304 slice (a), CG PR #772 (`code/ir-304-slice-a-record-input`), not
  merged. Until it merges the driver cannot compile against these types and
  AC-1 through AC-10 stay Planned.
- IR-304 slice (b) lowers guards to `Assume` statements. The projection does
  not depend on it; the guard-`Assume` branch of AC-10 does.
- IR-1494 is the driver ticket for this requirement. The driver code and
  tests are a later, separate change.
- QSL-266, QSL-700 and QSL-904 own the per-occurrence requirement record and
  its `RequirementItem` fields.
- [FR-001](FR-001-orchestrate-the-driver-steps.md) owns the position vector,
  node lookup and runtime-invariant vocabulary;
  [FR-010](FR-010-send-only-bound-items-to-codegen.md) owns which items are
  bound.
- [TC-034](../test-cases/TC-034-project-requirement-records-into-codegen.md).
