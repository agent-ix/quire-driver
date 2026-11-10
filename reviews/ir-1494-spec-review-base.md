---
id: SR-14854
title: "IR-1494 requirement-record projection spec-review/base review"
type: SpecReview
analysis: base
review_set: subset
scope: "agent-ix/quire-driver; IR-1494; PR #146; spec/functional/FR-020, spec/test-cases/TC-034, FR-001, FR-010, spec/spec.md, spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-driver/FR-020, type: references }
  - { target: ix://agent-ix/quire-driver/TC-034, type: references }
---
## Summary

Ticket: IR-1494. Reviewer spec-review/base pass over PR #146 (head 5b0e7a2), spec-only: new FR-020 and TC-034 plus FR-001/FR-010 cross-references and index updates. Verified against CG FR-022 on quire-contract-codegen origin/main, CG PR #772 (code/ir-304-slice-a-record-input), QSL origin/main (qsl-route request.rs, qsl-semantics claims.rs and claims/map.rs) and the driver code (src/drive.rs, src/refusal.rs, src/upstream.rs, Cargo.toml, quire-cli/Cargo.toml).

## Verdict

FAIL - not mergeable. The projection direction, names (RequirementItem accessors, ResultBound::node(), RoutedRequirement, RequirementRecords, with_requirement, KaniGenerationContext.requirement_records, RoutedGenerationError::RequirementRecord) and the V2-frame rule match QSL origin/main and CG FR-022 / PR #772. Reusing runtime_invariant at stage Generation for both the driver fault and CG's RequirementRecord passthrough is consistent with FR-001-AC-24 and the existing generation_code mapping (ExcludedDriverIndex, UnauthenticatedOperationView). Two defects block: the record source is mis-described (QSL keys records by occurrence AND capability), and AC-9 claims an eighth CG variant that slice (a) does not ship.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | 'Every record of requirements()' ignores QSL's per-capability records at one occurrence; projection collides on valid packages | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-002 | medium | AC-9 requires GuardContextMissing, which CG adds only in slice (b); the slice (a) gate is mis-stated | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

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
      "id": "FR-020-AC-1",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "For a routed item whose QSL record has a conversion context and a two-guard path condition (one guard requiring `holds = true`, one `holds = false`), the `RoutedRequirement` passed to CG carries that item's occurrence (node, role and ordinal), the V2 node of its result bound, the conversion occurrence, and the two guards in QSL's order with the same `holds` values. For an item with no conversion context it carries `None`."
    },
    {
      "id": "FR-020-AC-2",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "`RequirementRecords` on the Kani context holds one entry per record of the package's `requirements()`, including a record whose item the export refused, each with its own result bound, conversion occurrence and ordered path condition."
    },
    {
      "id": "FR-020-AC-3",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "Given one application node with two occurrences that differ in ordinal, result bound and path condition, the two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries, and each item's requirement equals the entry at its own key."
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
      "id": "FR-020-AC-6",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "For the FR-010-AC-4 vector, CG receives a `RoutedRequirement` for A and for C and none for B; `RequirementRecords` still holds B's record; B's absent requirement is not a fault, including when B's record is also removed from the view; and the run completes as FR-001-AC-17 states."
    },
    {
      "id": "FR-020-AC-7",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them."
    },
    {
      "id": "FR-020-AC-8",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "Given a record whose QSL result-bound node differs from the type the IR result context selects, the driver passes the QSL node unchanged; CG refuses with `RequirementRecord` and `ResultBoundMismatch`, and the driver returns it unchanged in `DriverRefusal::Generation` with stage Generation, wire code `runtime_invariant` and kind internal fault."
    },
    {
      "id": "FR-020-AC-9",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "For each `RequirementRecordMismatch` variant CG names (`RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`, `PathConditionMismatch`, `GuardContextMissing`), a CG refusal carrying it is returned as `DriverRefusal::Generation` with the request index, occurrence and mismatch unchanged, stage Generation, wire code `runtime_invariant` and outcome kind internal fault."
    },
    {
      "id": "FR-020-AC-10",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard."
    },
    {
      "id": "FR-020-AC-11",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "The driver's `Cargo.toml` entry for `quire-contract-codegen` names a branch like the `qsl-route` entry and carries no `rev`, and `Cargo.lock` content is the only record of the resolved commit."
    },
    {
      "id": "TC-034",
      "path": "spec/test-cases/TC-034-project-requirement-records-into-codegen.md",
      "role": "examined",
      "excerpt": "Each mutant must make the named assertion fail; none is equivalent to the correct driver. | Mutant | Caught by | |---|---| | Build the occurrence from the node id with role and ordinal dropped | AC-1 occurrence equality; AC-3 (two items collapse) | | Reverse the path condition, or flip a `holds` value | AC-1 ordered-guard equality | | Drop the conversion context or fill it from the node | AC-1 `None` and `Some` cases | | Build records only for routed items | AC-2 refused item's record; AC-6 B's record | | Fill a missing record from the item's own fields | AC-4 fault and no-generation-call a..."
    },
    {
      "id": "FR-022",
      "path": "ix://agent-ix/quire-contract-codegen/FR-022",
      "role": "context_only",
      "excerpt": "RequirementRecords wraps a BTreeMap<RequirementOccurrence, RequirementRecordInput> for the whole checked package. It is built with RequirementRecords::new(map), so one key has at most one record by type."
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "high",
      "confidence": "high",
      "check": "soundness",
      "artifact_id": "FR-020-AC-2",
      "related_ids": [
        "FR-020",
        "FR-020-AC-5"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "Inputs; Behavior 'build RequirementRecords'; AC-2",
      "excerpt": "`RequirementRecords` on the Kani context holds one entry per record of the package's `requirements()`, including a record whose item the export refused, each with its own result bound, conversion occurrence and ordered path condition.",
      "finding": "QSL's CheckedGraph::requirements() is a RequirementMap keyed by (OccurrenceKey, Capability), not one record per scalar occurrence: qsl-semantics/src/check/claims/map.rs:151 'Different capabilities may share the occurrence', and real packages do it (step relations hold TemporalSatisfaction and OperationContract records at one claim occurrence, qsl-package/src/emit/tests/step_relation.rs:141,150). Projecting 'every record' into CG's BTreeMap<RequirementOccurrence, RequirementRecordInput> makes ONE QSL key yield two entries, so the driver either refuses valid packages with RecordDuplicate or silently overwrites. FR-020 must say which records project (e.g. only the record of the routed item's capability via get_kind, or only scalar value records) and fix the Inputs line 'one per scalar application occurrence'.",
      "method": "spec-review/base"
    },
    {
      "fnd": "FND-002",
      "severity": "medium",
      "confidence": "high",
      "check": "soundness",
      "artifact_id": "FR-020-AC-9",
      "related_ids": [
        "FR-020",
        "FR-020-AC-10"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "AC-9; Behavior CG-refusal bullet; Dependencies",
      "excerpt": "For each `RequirementRecordMismatch` variant CG names (`RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`, `PathConditionMismatch`, `GuardContextMissing`), a CG refusal carrying it is returned as `DriverRefusal::Generation` with the request index, occurrence and mismatch unchanged, stage Generation, wire code `runtime_invariant` and outcome kind internal fault.",
      "finding": "AC-9 and the Behavior bullet list eight RequirementRecordMismatch variants including GuardContextMissing, but CG ships GuardContextMissing only in IR-304 slice (b): CG FR-022 'Scoped evidence status' gives slice (a) 'every RequirementRecordMismatch variant except GuardContextMissing', and PR #772 src/core/requirement.rs has seven variants. Dependencies say AC-1..AC-10 are Planned only until slice (a), so AC-9's eighth case cannot compile at the stated gate. Mark GuardContextMissing as slice (b) in AC-9, the Behavior bullet, Dependencies and the tests.md/TC-034 status rows.",
      "method": "spec-review/base"
    }
  ]
}
```
