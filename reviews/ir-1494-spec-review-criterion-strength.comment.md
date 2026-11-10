<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14859 method=spec-review/criterion-strength lang=spec pr=quire-driver#146 model=claude-opus-5-5 run=87752af5-6144-4b92-9817-ee41c4ca9989 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | untestable-ac | AC-5's 'two distinct QSL keys -> one occurrence' is unreachable; its mutant control cannot fail |
| FND-002 | medium | ambiguous | AC-10 disjunction passes in either slice; occurrence-keyed harness is a slice (b) field |
| FND-003 | low | untestable-ac | AC-3 fixture (one node, two occurrences with different result bounds) is not named and may not exist |
| FND-004 | low | soundness | AC-6's 'no requirement for B' and its mutant are vacuous: B is never a routed item |

Verdict: Run by hand: Jev is not installed on this host. AC-1, AC-2, AC-7, AC-8 and AC-9 controls can fail via their named assertions (checked against TC-034's mutant table). AC-5 has no reachable control, AC-10 is a disjunction that passes in either slice, AC-3 needs a fixture the spec does not name, and AC-6's B half is vacuous.

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-020-AC-1
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For a routed item whose QSL record has a conversion context and a two-guard path condition (one guard requiring `holds = true`, one `holds = false`), the `RoutedRequirement` passed to CG carries that item's occurrence (node, role and ordinal), the V2 node of its result bound, the conversion occurrence, and the two guards in QSL's order with the same `holds` values. For an item with no conversion context it carries `None`.
- id: FR-020-AC-2
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '`RequirementRecords` on the Kani context holds one entry per record of the package''s `requirements()`, including a record whose item the export refused, each with its own result bound, conversion occurrence and ordered path condition.'
- id: FR-020-AC-3
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: Given one application node with two occurrences that differ in ordinal, result bound and path condition, the two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries, and each item's requirement equals the entry at its own key.
- id: FR-020-AC-4
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: When the requirement records hold no entry for a routed item's occurrence, the run refuses with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record.
- id: FR-020-AC-5
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: When two distinct QSL keys project to one `RequirementOccurrence`, the run refuses with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index; stage, wire code and kind are those of FR-020-AC-4 and CG generation is not called.
- id: FR-020-AC-6
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For the FR-010-AC-4 vector, CG receives a `RoutedRequirement` for A and for C and none for B; `RequirementRecords` still holds B's record; B's absent requirement is not a fault, including when B's record is also removed from the view; and the run completes as FR-001-AC-17 states.
- id: FR-020-AC-7
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them.
- id: FR-020-AC-8
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: Given a record whose QSL result-bound node differs from the type the IR result context selects, the driver passes the QSL node unchanged; CG refuses with `RequirementRecord` and `ResultBoundMismatch`, and the driver returns it unchanged in `DriverRefusal::Generation` with stage Generation, wire code `runtime_invariant` and kind internal fault.
- id: FR-020-AC-9
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For each `RequirementRecordMismatch` variant CG names (`RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`, `PathConditionMismatch`, `GuardContextMissing`), a CG refusal carrying it is returned as `DriverRefusal::Generation` with the request index, occurrence and mismatch unchanged, stage Generation, wire code `runtime_invariant` and outcome kind internal fault.
- id: FR-020-AC-10
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard.
- id: FR-020-AC-11
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: The driver's `Cargo.toml` entry for `quire-contract-codegen` names a branch like the `qsl-route` entry and carries no `rev`, and `Cargo.lock` content is the only record of the resolved commit.
- id: TC-034
  path: spec/test-cases/TC-034-project-requirement-records-into-codegen.md
  role: examined
  excerpt: Each mutant must make the named assertion fail; none is equivalent to the correct driver. | Mutant | Caught by | |---|---| | Build the occurrence from the node id with role and ordinal dropped | AC-1 occurrence equality; AC-3 (two items collapse) | | Reverse the path condition, or flip a `holds` value | AC-1 ordered-guard equality | | Drop the conversion context or fill it from the node | AC-1 `None` and `Some` cases | | Build records only for routed items | AC-2 refused item's record; AC-6 B's record | | Fill a missing record from the item's own fields | AC-4 fault and no-generation-call a...
findings:
- fnd: FND-001
  severity: high
  confidence: high
  method: spec-review/criterion-strength
  check: untestable-ac
  artifact_id: FR-020-AC-5
  related_ids:
  - FR-020
  - TC-034
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: AC-5; TC-034 step 2 and mutant 'Overwrite on a duplicate key'
  excerpt: When two distinct QSL keys project to one `RequirementOccurrence`, the run refuses with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index; stage, wire code and kind are those of FR-020-AC-4 and CG generation is not called.
  finding: 'Two DISTINCT QSL keys cannot project to one RequirementOccurrence: resolve_checked_node uses structural_key_matches, a full structural equality (src/drive.rs:1288-1347), and already faults AmbiguousNode on one key->two ids; the role map is 1:1 over six spellings; the ordinal is copied. No real or seam-built input reaches AC-5 unless the node resolver itself is injected, which nothing specifies, so the ''Overwrite on a duplicate key'' mutant has no control that can fail. The real collision is one key with records of two capabilities (base FND-001). Re-state AC-5 on that case, or drop RecordDuplicate.'
- fnd: FND-002
  severity: medium
  confidence: high
  method: spec-review/criterion-strength
  check: ambiguous
  artifact_id: FR-020-AC-10
  related_ids:
  - FR-020-AC-9
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: AC-10
  excerpt: In an end-to-end run over a QSL-checked package holding an unguarded function and a function guarded by `if x > 5`, the unguarded item generates a harness keyed by its occurrence; the guarded item either generates a harness whose first statement after the symbolic inputs is the guard `Assume` (after IR-304 slice (b)) or settles `unsupported` with `path_guard_not_lowered` (slice (a) reports `GuardLoweringNotImplemented`), and no harness for it omits the guard.
  finding: 'AC-10 is an either/or that passes with either outcome in either slice: after slice (b), a regression back to unsupported/path_guard_not_lowered still passes. ''Generates a harness keyed by its occurrence'' is not assertable in slice (a): ScalarObligationIdentity.requirement_occurrence is a slice (b) field (CG FR-022 Scoped evidence status; FR-022-AC-31). Split by slice: slice (a) - g generates a harness, h settles path_guard_not_lowered with exactly guard_lowering_not_implemented naming its first guard; slice (b) - h''s harness has the guard Assume first after the symbolic inputs and requirement_occurrence equals the item''s occurrence.'
- fnd: FND-003
  severity: low
  confidence: medium
  method: spec-review/criterion-strength
  check: untestable-ac
  artifact_id: FR-020-AC-3
  related_ids:
  - TC-034
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: AC-3; TC-034 step 1
  excerpt: Given one application node with two occurrences that differ in ordinal, result bound and path condition, the two routed items carry different `RoutedRequirement` values, `RequirementRecords` holds two entries, and each item's requirement equals the entry at its own key.
  finding: AC-3 needs one application node whose two occurrences differ in result bound as well as ordinal and path condition. The canonical one-node/two-occurrence source (CG FR-022-AC-27 `if p then x + 1 else x + 1`) gives both occurrences the same result bound, and TC-034 names no source that yields different bounds. Name the QSL source, or drop 'result bound' from the conjunction.
- fnd: FND-004
  severity: low
  confidence: high
  method: spec-review/criterion-strength
  check: soundness
  artifact_id: FR-020-AC-6
  related_ids:
  - TC-034
  - FR-010-AC-4
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: AC-6; mutant 'attach a requirement to B'
  excerpt: For the FR-010-AC-4 vector, CG receives a `RoutedRequirement` for A and for C and none for B; `RequirementRecords` still holds B's record; B's absent requirement is not a fault, including when B's record is also removed from the view; and the run completes as FR-001-AC-17 states.
  finding: '''CG receives ... none for B'' and the mutant ''attach a requirement to B'' are vacuous: B is an Excluded position, never a RoutedGenerationItem (src/drive.rs lower_driver_request; TC-017 asserts generation items [0, 2]), so no requirement can be attached to it and the mutant is not expressible. Keep the B-record and ''B record removed'' halves; drop the vacuous half and mutant.'
bindings: []
```

+++
