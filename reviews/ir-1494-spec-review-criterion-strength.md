---
id: SR-14859
title: "IR-1494 requirement-record projection spec-review/criterion-strength review"
type: SpecReview
analysis: criterion-strength
review_set: subset
scope: "agent-ix/quire-driver; IR-1494; PR #146; spec/functional/FR-020, spec/test-cases/TC-034, FR-001, FR-010, spec/spec.md, spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-driver/FR-020, type: references }
  - { target: ix://agent-ix/quire-driver/TC-034, type: references }
---
## Summary

Ticket: IR-1494. Reviewer spec-review/criterion-strength pass over PR #146 (head 5b0e7a2), spec-only: new FR-020 and TC-034 plus FR-001/FR-010 cross-references and index updates. Verified against CG FR-022 on quire-contract-codegen origin/main, CG PR #772 (code/ir-304-slice-a-record-input), QSL origin/main (qsl-route request.rs, qsl-semantics claims.rs and claims/map.rs) and the driver code (src/drive.rs, src/refusal.rs, src/upstream.rs, Cargo.toml, quire-cli/Cargo.toml).

## Verdict

Run by hand: Jev is not installed on this host. AC-1, AC-2, AC-7, AC-8 and AC-9 controls can fail via their named assertions (checked against TC-034's mutant table). AC-5 has no reachable control, AC-10 is a disjunction that passes in either slice, AC-3 needs a fixture the spec does not name, and AC-6's B half is vacuous.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-5's 'two distinct QSL keys -> one occurrence' is unreachable; its mutant control cannot fail | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-002 | medium | AC-10 disjunction passes in either slice; occurrence-keyed harness is a slice (b) field | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-003 | low | AC-3 fixture (one node, two occurrences with different result bounds) is not named and may not exist | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-004 | low | AC-6's 'no requirement for B' and its mutant are vacuous: B is never a routed item | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

## Reviewed Units

```json
{
  "clean": false,
  "scope": [
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
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "high",
      "confidence": "high",
      "check": "untestable-ac",
      "artifact_id": "FR-020-AC-5",
      "related_ids": [
        "FR-020",
        "TC-034"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "AC-5; TC-034 step 2 and mutant 'Overwrite on a duplicate key'",
      "excerpt": "When two distinct QSL keys project to one `RequirementOccurrence`, the run refuses with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index; stage, wire code and kind are those of FR-020-AC-4 and CG generation is not called.",
      "finding": "Two DISTINCT QSL keys cannot project to one RequirementOccurrence: resolve_checked_node uses structural_key_matches, a full structural equality (src/drive.rs:1288-1347), and already faults AmbiguousNode on one key->two ids; the role map is 1:1 over six spellings; the ordinal is copied. No real or seam-built input reaches AC-5 unless the node resolver itself is injected, which nothing specifies, so the 'Overwrite on a duplicate key' mutant has no control that can fail. The real collision is one key with records of two capabilities (base FND-001). Re-state AC-5 on that case, or drop RecordDuplicate.",
      "method": "spec-review/criterion-strength"
    },
    {
      "fnd": "FND-002",
      "severity": "medium",
      "confidence": "high",
      "check": "ambiguous",
      "artifact_id": "FR-020-AC-10",
      "related_ids": [
        "FR-020-AC-9"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "AC-10",
      "excerpt": "In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard.",
      "finding": "AC-10 is an either/or that passes with either outcome in either slice: after slice (b), a regression back to unsupported/path_guard_not_lowered still passes. 'Generates a harness keyed by its occurrence' is not assertable in slice (a): ScalarObligationIdentity.requirement_occurrence is a slice (b) field (CG FR-022 Scoped evidence status; FR-022-AC-31). Split by slice: slice (a) - g generates a harness, h settles path_guard_not_lowered with exactly guard_lowering_not_implemented naming its first guard; slice (b) - h's harness has the guard Assume first after the symbolic inputs and requirement_occurrence equals the item's occurrence.",
      "method": "spec-review/criterion-strength"
    },
    {
      "fnd": "FND-003",
      "severity": "low",
      "confidence": "medium",
      "check": "untestable-ac",
      "artifact_id": "FR-020-AC-3",
      "related_ids": [
        "TC-034"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "AC-3; TC-034 step 1",
      "excerpt": "Given one application node with two occurrences that differ in ordinal, result bound and path condition, the two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries, and each item's requirement equals the entry at its own key.",
      "finding": "AC-3 needs one application node whose two occurrences differ in result bound as well as ordinal and path condition. The canonical one-node/two-occurrence source (CG FR-022-AC-27 `if p then x + 1 else x + 1`) gives both occurrences the same result bound, and TC-034 names no source that yields different bounds. Name the QSL source, or drop 'result bound' from the conjunction.",
      "method": "spec-review/criterion-strength"
    },
    {
      "fnd": "FND-004",
      "severity": "low",
      "confidence": "high",
      "check": "soundness",
      "artifact_id": "FR-020-AC-6",
      "related_ids": [
        "TC-034",
        "FR-010-AC-4"
      ],
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "lines": "AC-6; mutant 'attach a requirement to B'",
      "excerpt": "For the FR-010-AC-4 vector, CG receives a `RoutedRequirement` for A and for C and none for B; `RequirementRecords` still holds B's record; B's absent requirement is not a fault, including when B's record is also removed from the view; and the run completes as FR-001-AC-17 states.",
      "finding": "'CG receives ... none for B' and the mutant 'attach a requirement to B' are vacuous: B is an Excluded position, never a RoutedGenerationItem (src/drive.rs lower_driver_request; TC-017 asserts generation items [0, 2]), so no requirement can be attached to it and the mutant is not expressible. Keep the B-record and 'B record removed' halves; drop the vacuous half and mutant.",
      "method": "spec-review/criterion-strength"
    }
  ]
}
```

## New findings (disposition pass 1)

Re-review of head ab47adb (commits 5e98dc5, ab47adb) against the review-pass findings.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | (Recorded late) AC-6 asserted FR-001-AC-17's completion, whose evidence is owned by TC-017 | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |
| FND-006 | low | (Recorded late) TC-034 had no mutant for AC-9's stage and outcome-kind assertions | spec/test-cases/TC-034-project-requirement-records-into-codegen.md |
| FND-007 | medium | AC-5's cited QSL step-relation fixture holds no value-validity record; its precondition has no named real source | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

```json
[
  {
    "fnd": "FND-005",
    "severity": "medium",
    "confidence": "high",
    "check": "soundness",
    "artifact_id": "FR-020-AC-6",
    "related_ids": [
      "FR-001-AC-17",
      "TC-017"
    ],
    "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
    "lines": "AC-6",
    "excerpt": "For the FR-010-AC-4 vector, CG receives a `RoutedRequirement` for A and for C and none for B; `RequirementRecords` still holds B's record; B's absent requirement is not a fault, including when B's record is also removed from the view; and the run completes as FR-001-AC-17 states.",
    "finding": "Reported in the review-pass handback but omitted from this SR file and its Linear comment at review time; recorded here so the record matches what the author acted on. AC-6 asserted 'the run completes as FR-001-AC-17 states', which is FR-001's criterion evidenced by TC-017, not TC-034."
  },
  {
    "fnd": "FND-006",
    "severity": "low",
    "confidence": "high",
    "check": "coverage",
    "artifact_id": "TC-034",
    "related_ids": [
      "FR-020-AC-9"
    ],
    "path": "spec/test-cases/TC-034-project-requirement-records-into-codegen.md",
    "lines": "Mutants",
    "excerpt": "| Map a CG record refusal to `invalid_request` or rewrite its fields | AC-9 code, kind and field equality |",
    "finding": "Reported in the review-pass handback but omitted from this SR file and its Linear comment at review time; recorded here. The only AC-9 mutant changed the wire code, so no mutant forced AC-9's stage or outcome-kind assertions to fail."
  },
  {
    "fnd": "FND-007",
    "severity": "medium",
    "confidence": "high",
    "check": "untestable-ac",
    "artifact_id": "FR-020-AC-5",
    "related_ids": [
      "TC-034"
    ],
    "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
    "lines": "AC-5; TC-034 step 1; mutant 'Look the item's record up by occurrence alone'",
    "excerpt": "Given a unit whose one occurrence holds both a `value-validity` record and a record of another capability (as QSL's step-relation fixture places a temporal-satisfaction and an operation-contract record on one occurrence), `RequirementRecords` holds exactly the `value-validity` record's fields at that key, the item's requirement equals them, and no fault is raised.",
    "finding": "QSL's step-relation fixture (qsl-package/src/emit/tests/step_relation.rs:129-154) asserts requirements().len() == 2: a TemporalSatisfaction and an OperationContract record on one claim occurrence, and no value-validity record. AC-5's precondition (one occurrence holding a value-validity record AND another capability) is therefore not met by the cited fixture, and no QSL source is shown that produces it (value-validity records sit on scalar application expression occurrences). Without the shared case, RequirementMap::get returns the sole value-validity record and the 'look up by occurrence alone' mutant is equivalent on every real input. Say AC-5 builds the map through the projection's map argument (RequirementMap is FromIterator, so a test can add another-capability record at the value-validity key), or cite a real source."
  }
]
```

## New findings (disposition pass 2)

Re-review of head ddf8761 (commit ddf8761 on ab47adb).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | AC-16 cannot occur in the driver flow: a NotApplicable item settles InvalidRequest at negotiation and never reaches generate_routed | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

```json
[
  {
    "fnd": "FND-008",
    "severity": "medium",
    "confidence": "high",
    "check": "untestable-ac",
    "artifact_id": "FR-020-AC-16",
    "related_ids": [
      "FR-020-AC-15",
      "FR-020"
    ],
    "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
    "lines": "AC-16; Behavior 'If a NotApplicable item of another capability sits at a node'; TC-034 mutant 'Drop the value-validity record'",
    "excerpt": "Given a bound `operation-contract` Kani item at a node that also holds a `value-validity` record, CG refuses the call as `RequirementRecord` with `OccurrenceNotSelected`, and the driver returns it unchanged in `DriverRefusal::Generation` with stage Generation, wire code `runtime_invariant` and outcome kind internal fault, with the record still in the map.",
    "finding": "In the driver every Kani item is a V2ScalarClaim; CG's preflight refuses a NotApplicable V2ScalarClaim as Invalid(ScalarResultMissing) -> Disposition::InvalidRequest, so the item is not routed. generate_routed's record checks (including OccurrenceNotSelected) iterate only routed items (generate.rs:416-419), so the AC-16 refusal and the 'drop the value-validity record' mutant's control cannot arise from a driver run. This is not a CG defect and needs no CG ticket; drop AC-16 and its Behavior bullet (see base FND-003)."
  }
]
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head ab47adb: Given a unit whose one occurrence holds both a `value-validity` record and a record of another capability (as QSL's step-relation fixture places a temporal-satisfaction and an operation-contract record on one occurrence), `RequirementRecords` holds exactly the `value-validity` record's fields at that key, the item's requirement equals them, and no fault is raised. |
| FND-002 | fixed | Round 1, head ab47adb: Slice (a) only: over the IR-304 guarded package, the unguarded item `g` generates a harness, and each guarded item (`f` then item, `h`) settles `unsupported` with `path_guard_not_lowered` and reason `guard_lowering_not_implemented`; no guarded item has a harness. This criterion retires when slice (b) lands, as CG FR-022-AC-38 does. / After IR-304 slice (b): over the same package the `h` item generates a harness whose guard `Assume` follows the symbolic inputs, and its identity's `requirement_occurrence` equals the item's occurrence; an item with a guard CG cannot lower settles `unsupported`... |
| FND-003 | still-open | RR16_UNIT (tests/drive.rs:208) already narrows its two x + 1 occurrences to Int[0, 10] and Int[0, 20], so 'extended so its two narrow conversions target different bounded types' changes nothing; but it has no guard, so both path conditions are empty and AC-3's 'differ in ... path condition' is still unmet. Add a guard to the fixture (e.g. route the two calls under if-branches) or drop 'path condition' from AC-3. |
| FND-004 | fixed | Round 1, head ab47adb: For the FR-010-AC-4 vector, the projection returns a requirement for A (index 0) and C (index 2) and `None` at B's index 1, B's position is excluded, `RequirementRecords` still holds B's record, the run completes without a `DriverRefusal`, and the same holds with B's record removed from the map. FR-001-AC-17 keeps its own evidence in TC-017; this criterion asserts only the projection and the absence of a fault. |
| FND-005 | fixed | Round 1, head ab47adb: FR-001-AC-17 keeps its own evidence in TC-017; this criterion asserts only the projection and the absence of a fault. |
| FND-006 | fixed | Round 1, head ab47adb: / Report a CG record refusal at the Negotiation stage / AC-9, AC-13 stage / / / Report a CG record refusal as outcome kind refusal / AC-9, AC-13 outcome kind / / / Rewrite a CG refusal's request index, occurrence or mismatch / AC-9, AC-13 field equality / |
| FND-003 | fixed | Round 2, head ddf8761: For a built unit in which one `x + 1` node's then and else occurrences are guarded by `p` and `not p` and converted to `Int[0, 10]` and `Int[0, 20]` (`function f using v(p: Boolean, x: Int[0, 9]): Boolean pure { if p then g(x + 1) else h(x + 1) }` with `g` over `Int[0, 10]` and `h` over `Int[0, 20]`, built on the RR16_UNIT shape), the two occurrences differ in ordinal, result bound and path condition. The two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries with different result bounds and path conditions, and each item's requirement equals the... |
| FND-007 | fixed | Round 2, head ddf8761: Given a `RequirementMap` built with `FromIterator` that holds, at one occurrence, both a `value-validity` record and an `operation-contract` record with a different result bound, `RequirementRecords` holds exactly the `value-validity` record's fields at that key, the `value-validity` item's requirement equals them, and no fault is raised. A lookup by occurrence alone finds the key ambiguous and returns no record, so that lookup raises `RequirementRecordAbsent` here. |
| FND-008 | fixed | Round 3, head e5bb946: AC-16, its Behavior bullet and the 'drop the value-validity record' mutant are removed; FR-020 now ends at AC-15: For a bound item of another capability (a model-reading `operation-contract` item) whose occurrence has no `value-validity` record in the map, negotiation settles it `InvalidRequest` as before, it is absent from the generation items, the projection returns `None` at its index, no fault is raised, and a `value-validity` item in the same request still receives its requirement. |
