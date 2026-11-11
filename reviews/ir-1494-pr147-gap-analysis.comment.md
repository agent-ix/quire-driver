<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14861 method=gap-analysis lang=rust pr=quire-driver#147 model=claude-opus-5-5 run=c649c842-70a7-46db-92d9-c4e448adac7c -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | coverage | AC-7/AC-1 two-guard ordered condition absent from every fixture; reverse-order mutant survives |
| FND-002 | medium | coverage | AC-5 untested, and the stated blocker does not hold: FromIterator + Clone records let a test re-key the operation-contract record onto the value-validity occurrence via the existing seam |
| FND-003 | medium | coverage | Spec status stale: FR-020/TC-034 still Planned until CG #772 everywhere; remaining gaps (AC-5, AC-14 guard-node half, AC-11/13 slice (b)) undocumented |
| FND-004 | low | coverage | AC-14 guard-node example not exercised (key re-keyed instead); plausibly unconstructible, but undocumented |
| FND-005 | low | other | TC-034 step 2's in-module test replaced by a public Upstream::requirement_records seam that the spec does not name |

Verdict: NOT mergeable. Matrix (quire matrix, quoin 0.29.1): AC-1..4, 6..10, 12, 14, 15 tagged; AC-5, 11, 13 untagged. Rulings on declared gaps: AC-11/AC-13 legitimately Planned (CG main dab4861f has the seven slice (a) mismatches, no GuardContextMissing); KaniReaderCatalogUnavailable legitimately unconstructible (CatalogReadError fields pub(super)); AC-14 guard-node half plausibly blocked (PathGuard/RequirementRecord have no public constructor); AC-5 NOT blocked. No spec row claims Implemented without a test; but none of these gaps is documented because the PR leaves the spec untouched.

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-020-AC-1, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For a routed item whose QSL record has a conversion context and a two-guard path condition (one guard requiring holds = true, one holds = false), the RoutedRequirement passed to CG carries that item's occurrence (node, role and ordinal), the V2 node of its result bound, the conversion occurrence, and the two guards in QSL's order with the same holds values. For an item with no conversion context it carries None."}
  - {id: FR-020-AC-2, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "RequirementRecords on the Kani context holds one entry per value-validity record of the package's requirements(), including a record whose item the export refused, each with its own result bound, conversion occurrence and ordered path condition, and holds no entry for a record of another capability."}
  - {id: FR-020-AC-3, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For a built unit in which one x + 1 node's then and else occurrences are guarded by p and not p and converted to Int[0, 10] and Int[0, 20], the two occurrences differ in ordinal, result bound and path condition; the two routed items carry different RoutedRequirement values."}
  - {id: FR-020-AC-4, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "With a RequirementMap that holds no value-validity record for the occurrence of an item negotiation routes to Kani, the run completes lowering and negotiation and then refuses with DriverFault::RequirementRecordAbsent; CG generation is not called."}
  - {id: FR-020-AC-5, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "Given a RequirementMap built with FromIterator that holds, at one occurrence, both a value-validity record and an operation-contract record with a different result bound, RequirementRecords holds exactly the value-validity record's fields at that key, the value-validity item's requirement equals them, and no fault is raised."}
  - {id: FR-020-AC-6, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For the FR-010-AC-4 vector, the projection returns a requirement for A and C and None at B's index 1, RequirementRecords still holds B's record, the run completes, and the same holds with B's record removed."}
  - {id: FR-020-AC-7, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For an item with two path guards and a conversion context, the request positions hold the conversion ContextRoot, then one ContextRoot per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them."}
  - {id: FR-020-AC-8, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "Given a record whose QSL result-bound node differs from the type the IR result context selects, CG refuses with RequirementRecord and ResultBoundMismatch, returned unchanged in DriverRefusal::Generation."}
  - {id: FR-020-AC-9, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For each of the seven RequirementRecordMismatch variants IR-304 slice (a) defines, a CG refusal carrying it is returned as DriverRefusal::Generation with the request index, occurrence and mismatch unchanged."}
  - {id: FR-020-AC-10, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "Slice (a) only: the unguarded item g generates a harness, and each guarded item settles unsupported with path_guard_not_lowered and reason guard_lowering_not_implemented; no guarded item has a harness."}
  - {id: FR-020-AC-11, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "After IR-304 slice (b): the h item generates a harness whose guard Assume follows the symbolic inputs, and its identity's requirement_occurrence equals the item's occurrence."}
  - {id: FR-020-AC-12, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For if p then x + 1 else x + 1, the positions hold a ContextRoot for p in the then item's span and another in the else item's span, and the root is not emitted once for both."}
  - {id: FR-020-AC-13, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "After IR-304 slice (b): a CG refusal RequirementRecord with GuardContextMissing is returned as DriverRefusal::Generation unchanged."}
  - {id: FR-020-AC-14, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "When a value-validity record of the package has a guard occurrence whose node the admitted package does not hold, the run refuses with DriverFault::RequirementRecordUnresolved naming the record's QSL occurrence key with cause UnknownNode; when a record's key has a role outside IR's occurrence vocabulary, the cause is UnknownOccurrenceRole."}
  - {id: FR-020-AC-15, path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md, role: examined, excerpt: "For a bound operation-contract item with no value-validity record, negotiation settles it InvalidRequest, the projection returns None and adds no guard root, no RequirementRecordAbsent fault is raised, and a value-validity item in the same request still receives its requirement."}
  - {id: TC-034, path: spec/test-cases/TC-034-project-requirement-records-into-codegen.md, role: context_only, excerpt: "Verify that the driver hands CG each routed item's QSL requirement and the package's requirement records exactly, and refuses typed when it cannot."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-020-AC-7
    related_ids: [FR-020-AC-1, TC-034]
    path: tests/drive.rs
    lines: "3427-3523"
    excerpt: |-
      For an item with two path guards and a conversion context, the request positions hold the conversion ContextRoot, then one ContextRoot per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them.
    finding: "AC-7 (and AC-1) require an item with two path guards in outermost-first order; the tagged test's fixture has one guard per item (guard_root_total == 2 over two items), so the order is never checked and the reverse-order mutant survives."
  - fnd: FND-002
    severity: medium
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-020-AC-5
    related_ids: [TC-034]
    path: tests/drive.rs
    lines: "3568-3587"
    excerpt: |-
      Given a RequirementMap built with FromIterator that holds, at one occurrence, both a value-validity record and an operation-contract record with a different result bound, RequirementRecords holds exactly the value-validity record's fields at that key, the value-validity item's requirement equals them, and no fault is raised.
    finding: "AC-5 is untested and not blocked: RequirementMap implements FromIterator and records are Clone, so the operation-contract record of MODEL_READING_UNIT can be re-keyed onto the value-validity item's occurrence through the existing requirement_records seam (the technique rekey() already uses)."
  - fnd: FND-003
    severity: medium
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-020
    related_ids: [TC-034, FR-020-AC-5, FR-020-AC-9, FR-020-AC-10, FR-020-AC-11, FR-020-AC-13, FR-020-AC-14]
    path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
    lines: "Description; spec/tests.md:127"
    excerpt: |-
      This requirement specifies code that depends on CG types not yet merged (IR-304 slice (a), CG code PR #772). Its criteria are Planned until that PR lands; the type names below are those CG FR-022 uses today and follow CG if review renames them.
    finding: "Spec status not updated: tests.md FR-020/TC-034 rows, spec.md index, FR-020 Description/Dependencies and AC-9/AC-10 rows still say Planned until CG PR #772 / types not merged, and AC-5, AC-14's guard-node half and the slice (b) dependency of AC-11/AC-13 are not recorded as the remaining gaps."
  - fnd: FND-004
    severity: low
    confidence: medium
    method: gap-analysis
    check: coverage
    artifact_id: FR-020-AC-14
    related_ids: []
    path: tests/drive.rs
    lines: "3593-3660"
    excerpt: |-
      When a value-validity record of the package has a guard occurrence whose node the admitted package does not hold, the run refuses with DriverFault::RequirementRecordUnresolved naming the record's QSL occurrence key with cause UnknownNode; when a record's key has a role outside IR's occurrence vocabulary, the cause is UnknownOccurrenceRole.
    finding: "AC-14's guard-node case (its first example) is not exercised: the test re-keys a record's key to an unknown node; the guard-node half is plausibly unconstructible but is not documented as uncovered."
  - fnd: FND-005
    severity: low
    confidence: medium
    method: gap-analysis
    check: other
    artifact_id: TC-034
    related_ids: [FR-020]
    path: src/upstream.rs
    lines: "120-124"
    excerpt: |-
      In an in-module test in src/drive.rs (drive() takes no RequirementMap), call the private lower_driver_request, negotiation and generate with a RequirementMap argument missing the routed item's value-validity record (AC-4), and with a record whose guard node the admitted package does not hold, and with a record whose key has an unmapped role (AC-14).
    finding: "TC-034 step 2 prescribes an in-module test that calls the private lower_driver_request with a RequirementMap; the PR instead adds a public defaulted trait method Upstream::requirement_records as the seam, which FR-020/TC-034 do not name."
bindings:
  - {test_id: guarded_occurrences_carry_their_own_requirement_and_guard_roots, ac_id: FR-020-AC-1, trace: correct}
  - {test_id: guarded_occurrences_carry_their_own_requirement_and_guard_roots, ac_id: FR-020-AC-2, trace: correct}
  - {test_id: guarded_occurrences_carry_their_own_requirement_and_guard_roots, ac_id: FR-020-AC-3, trace: correct}
  - {test_id: guarded_occurrences_carry_their_own_requirement_and_guard_roots, ac_id: FR-020-AC-7, trace: correct}
  - {test_id: guarded_occurrences_carry_their_own_requirement_and_guard_roots, ac_id: FR-020-AC-12, trace: correct}
  - {test_id: a_routed_item_without_a_record_refuses_before_generation, ac_id: FR-020-AC-4, trace: correct}
  - {test_id: requirement_record_faults_are_generation_runtime_invariants, ac_id: FR-020-AC-4, trace: correct}
  - {test_id: the_export_partition_vector_projects_a_and_c_and_not_b, ac_id: FR-020-AC-6, trace: correct}
  - {test_id: a_disagreeing_result_bound_is_cg_s_refusal_returned_unchanged, ac_id: FR-020-AC-8, trace: correct}
  - {test_id: every_requirement_record_mismatch_is_a_distinct_generation_fault, ac_id: FR-020-AC-9, trace: correct}
  - {test_id: a_guarded_item_settles_path_guard_not_lowered_and_has_no_harness, ac_id: FR-020-AC-10, trace: correct}
  - {test_id: an_unprojectable_record_refuses_before_negotiation, ac_id: FR-020-AC-14, trace: correct}
  - {test_id: requirement_record_faults_are_generation_runtime_invariants, ac_id: FR-020-AC-14, trace: correct}
  - {test_id: a_bound_operation_contract_item_projects_none_and_raises_no_fault, ac_id: FR-020-AC-15, trace: correct}
```

+++
