---
id: SR-14858
title: "IR-1494 requirement-record projection spec-review/failure-domain review"
type: SpecReview
analysis: failure-domain
review_set: subset
scope: "agent-ix/quire-driver; IR-1494; PR #146; spec/functional/FR-020, spec/test-cases/TC-034, FR-001, FR-010, spec/spec.md, spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-driver/FR-020, type: references }
  - { target: ix://agent-ix/quire-driver/TC-034, type: references }
---
## Summary

Ticket: IR-1494. Reviewer spec-review/failure-domain pass over PR #146 (head 5b0e7a2), spec-only: new FR-020 and TC-034 plus FR-001/FR-010 cross-references and index updates. Verified against CG FR-022 on quire-contract-codegen origin/main, CG PR #772 (code/ir-304-slice-a-record-input), QSL origin/main (qsl-route request.rs, qsl-semantics claims.rs and claims/map.rs) and the driver code (src/drive.rs, src/refusal.rs, src/upstream.rs, Cargo.toml, quire-cli/Cargo.toml).

## Verdict

Needs changes. Frames, the no-substitution rule and CG passthrough are covered. Three failure modes are unspecified or rest on an undefined seam.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | RecordMissing/AC-4 reachable only via an undefined 'requirements view' seam; cause name collides with CG's RecordMissing | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-002 | medium | No fault/stage/index for a package record (not tied to a routed item) that fails to resolve | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-003 | medium | Shared guard node across two items' spans is unspecified; AC-7's single-item vector lets a dedup mutant pass | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

## Reviewed Units

```json
{
  "clean": false,
  "scope": [
    {
      "id": "FR-020",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item's `RoutedRequirement`, and the package's `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), \"Per-occurrence requirement ..."
    },
    {
      "id": "FR-020-AC-4",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "When the requirement records hold no entry for a routed item's occurrence, the run refuses with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record."
    },
    {
      "id": "FR-020-AC-5",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "When two distinct QSL keys project to one `RequirementOccurrence`, the run refuses with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index; stage, wire code and kind are those of FR-020-AC-4 and CG generation is not called."
    },
    {
      "id": "FR-020-AC-7",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them."
    },
    {
      "id": "FR-020#behavior-missing",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "- If the package's requirement records hold no entry for a routed item's occurrence, then the driver SHALL refuse the run with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and the item's occurrence, before calling CG generation."
    },
    {
      "id": "FR-020#behavior-records",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "- The driver SHALL build `RequirementRecords` from every record of the compiled package's own `requirements()`, projecting each record's `ResultBound::node()`, conversion occurrence and path condition exactly as for an item. It SHALL NOT build a record from an item, from the checked graph, or from any source other than QSL's record at that key."
    },
    {
      "id": "FR-020#behavior-guards",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "- The driver SHALL add one `ContextRoot` position for each path guard, after any conversion root and in outermost-first order, inside that item's contiguous context span, as CG FR-022 \"Path conditions in the scalar harness\" requires. Context roots SHALL NOT advance driver indices (FR-001)."
    },
    {
      "id": "TC-034#procedure",
      "path": "spec/test-cases/TC-034-project-requirement-records-into-codegen.md",
      "role": "examined",
      "excerpt": "1. Check a QSL unit with one node carrying two occurrences (different ordinal, result bound and path condition), one with a conversion context and two guards, and the IR-304 guarded package. Capture what the driver passes to a recording CG seam (AC-1 to AC-3, AC-6, AC-7). 2. Drive the projection function over a requirements view with the item's key removed (AC-4), and over a view in which two keys map to one occurrence (AC-5). Assert the fault fields, code, stage, kind and that the recording seam saw no generation call. 3. Reuse the FR-010-AC-4 partition vector, once as is and once with B's..."
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "medium",
      "confidence": "high",
      "check": "untestable-ac",
      "artifact_id": "FR-020-AC-4",
      "related_ids": [
        "FR-020-AC-5",
        "TC-034"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "Behavior RecordMissing bullet; AC-4; TC-034 step 2",
      "excerpt": "When the requirement records hold no entry for a routed item's occurrence, the run refuses with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record.",
      "finding": "Items and records both come from the same CheckedGraph::requirements() (items_from_requirements), so in production a routed item always has its record: AC-4 is reachable only through the 'projection function' over a 'requirements view' that TC-034 step 2 names and FR-020 never defines. The repo's seam is the Upstream trait (src/upstream.rs: tests wrap QslCg and still run the real functions); it cannot hand the driver a record set separate from the compiled package (QSL exposes no public record mutator outside cfg test-support). The projection's signature (items and records as separate inputs, unit-tested directly) is therefore a hidden coder decision; state it in FR-020, or drop the driver pre-check and rely on CG's RecordMissing passthrough (AC-9). Also the driver cause name RecordMissing collides with CG's RequirementRecordMismatch::RecordMissing, giving two refusal shapes (Fault vs Generation) for one condition.",
      "method": "spec-review/failure-domain"
    },
    {
      "fnd": "FND-002",
      "severity": "medium",
      "confidence": "medium",
      "check": "soundness",
      "artifact_id": "FR-020",
      "related_ids": [
        "FR-020-AC-2"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "Behavior 'build RequirementRecords'",
      "excerpt": "One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item's `RoutedRequirement`, and the package's `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), \"Per-occurrence requirement ...",
      "finding": "Projecting every package record resolves each record's result-bound, conversion and guard NodeKeys in the admitted package, including records of excluded or unrouted items. A record whose node does not resolve, or resolves ambiguously, or whose guard role is unknown, hits UnknownNode/AmbiguousNode/UnknownOccurrenceRole, which carry a request_index that a package record does not have. FR-020 states no fault, stage or index for a record that is not any routed item's, nor whether such a record stops the run.",
      "method": "spec-review/failure-domain"
    },
    {
      "fnd": "FND-003",
      "severity": "medium",
      "confidence": "medium",
      "check": "coverage",
      "artifact_id": "FR-020-AC-7",
      "related_ids": [
        "FR-020"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "Behavior ContextRoot bullet; AC-7",
      "excerpt": "For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them.",
      "finding": "The canonical guarded vector (CG FR-022-AC-27, `if p then x + 1 else x + 1`) routes two items that share one guard node p (for `if p` the guard node is the parameter node). FR-020 neither says a shared guard node is repeated as a ContextRoot in each item's span nor that each guard node joins the lowering roots (lower_driver_request pushes roots and positions together). AC-7 uses one item, so a mutant that emits a shared guard root once passes AC-7 while CG's per-span check (GuardContextMissing, slice (b)) or the FR-039 position check refuses. Add the two-item shared-guard case to AC-7.",
      "method": "spec-review/failure-domain"
    }
  ]
}
```

## New findings (disposition pass 1)

Re-review of head ab47adb (commits 5e98dc5, ab47adb) against the review-pass findings.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | RequirementRecordUnresolved cannot carry the existing causes 'unchanged'; unknown role has two faults; cause type unnamed | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

```json
[
  {
    "fnd": "FND-004",
    "severity": "medium",
    "confidence": "high",
    "check": "soundness",
    "artifact_id": "FR-020-AC-14",
    "related_ids": [
      "FR-020"
    ],
    "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
    "lines": "Behavior first bullet and 'fails to project' bullet; Outputs; AC-14",
    "excerpt": "When a `value-validity` record of the package has a result-bound node, conversion occurrence or guard occurrence the admitted package does not resolve (or ambiguously resolves), the run refuses with `DriverFault::RequirementRecordUnresolved`, naming the record's QSL occurrence key and carrying the unknown-node or ambiguous-node cause unchanged, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault, even when no item routes to that record; CG generation is not called.",
    "finding": "'carrying the unknown-node, ambiguous-node or unknown-role cause unchanged' cannot hold: DriverFault::UnknownNode and AmbiguousNode carry request_index (src/refusal.rs:205-221), which FR-020 itself says a package record lacks, so the cause must be a new type that the spec does not name or shape. The first Behavior bullet still sends an unknown QSL role to the existing UnknownOccurrenceRole fault while the unresolved bullet sends a record's unknown role to RequirementRecordUnresolved, so one condition has two faults; AC-14 covers only the node causes. Name the cause enum and its variants, say which fault an unknown role in a record's guard or conversion takes, and note that the ambiguous-node case is not test-constructible through IR admission (or say how it is)."
  }
]
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head ab47adb: - Both come from one driver projection function that takes the items, the package's `RequirementMap` and the admitted package as arguments and returns the requirement for each request index (`None` at an excluded position) with the `RequirementRecords`, or a fault. Taking the map as an argument is what lets a test supply a map with a record removed. |
| FND-002 | fixed | Round 1, head ab47adb: - If a `value-validity` record of the package fails to project (a node of its result bound, conversion occurrence or a guard occurrence is unknown or ambiguous in the admitted package, or a role is unknown), then the driver SHALL refuse the run with the `RequirementRecordUnresolved` fault, naming the record's QSL occurrence key (a package record has no request index) and carrying the unknown-node, ambiguous-node or unknown-role cause unchanged. It SHALL NOT omit the record and continue, whether or not an item routes to it. |
| FND-003 | fixed | Round 1, head ab47adb: For `if p then x + 1 else x + 1`, whose then and else items are both guarded by the node of `p`, the positions hold a `ContextRoot` for `p` in the then item's span and another in the else item's span, and the root is not emitted once for both. |
| FND-004 | fixed | Round 2, head ddf8761: - Or a `DriverRefusal::Fault` of one of two new `DriverFault` variants: `RequirementRecordAbsent { request_index, occurrence }` and `RequirementRecordUnresolved { occurrence, cause }`, where `cause` is a new `RequirementRecordUnresolvedCause` with the variants `UnknownNode { node }`, `AmbiguousNode { node }` and `UnknownOccurrenceRole { role }`. The existing `DriverFault` node and role variants carry a request index that a package record lacks, so the new cause type carries the same payload without it. Each has stage Generation, wire code `runtime_invariant` and outcome kind internal fault.... / When a `value-validity` record of the package has a guard occurrence whose node the admitted package does not hold, the run refuses with `DriverFault::RequirementRecordUnresolved` naming the record's QSL occurrence key with cause `UnknownNode`; when a record's key has a role outside IR's occurrence vocabulary, the cause is `UnknownOccurrenceRole`. Each has stage Generation, wire code `runtime_invariant` and outcome kind internal fault, is raised even when no item routes to that record, and CG generation is not called. The `AmbiguousNode` cause is produced by the same node lookup FR-001 uses... |
