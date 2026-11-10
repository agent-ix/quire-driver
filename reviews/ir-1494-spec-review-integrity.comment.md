<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14855 method=spec-review/integrity lang=spec pr=quire-driver#146 model=claude-opus-5-5 run=87752af5-6144-4b92-9817-ee41c4ca9989 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | low | other | spec.md section 5 count left at FR-014/TC-028 while tests.md was updated to FR-020/TC-034 |

Verdict: Integrity is otherwise sound: ids are unique, FR-020/TC-034 are added to spec.md relationships and index and to tests.md (both tables); FR-001 and FR-010 cross-references resolve; quire validate passes for every touched file and quire matrix lists FR-020-AC-1..11 (untagged as expected for a spec-only change; AC-11 reports method-without-symbol because it is Inspection). One stale index line remains.

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: 'One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item''s `RoutedRequirement`, and the package''s `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), "Per-occurrence requirement ...'
- id: TC-034
  path: spec/test-cases/TC-034-project-requirement-records-into-codegen.md
  role: examined
  excerpt: Each mutant must make the named assertion fail; none is equivalent to the correct driver. | Mutant | Caught by | |---|---| | Build the occurrence from the node id with role and ordinal dropped | AC-1 occurrence equality; AC-3 (two items collapse) | | Reverse the path condition, or flip a `holds` value | AC-1 ordered-guard equality | | Drop the conversion context or fill it from the node | AC-1 `None` and `Some` cases | | Build records only for routed items | AC-2 refused item's record; AC-6 B's record | | Fill a missing record from the item's own fields | AC-4 fault and no-generation-call a...
- id: spec.md#5
  path: spec/spec.md
  role: examined
  excerpt: '[TM-001](tests.md) integrates FR-001 through FR-014, and TC-001 through TC-028.'
- id: TM-001
  path: spec/tests.md
  role: examined
  excerpt: It covers FR-001 through FR-020 and TC-001 through TC-034.
findings:
- fnd: FND-001
  severity: low
  confidence: high
  method: spec-review/integrity
  check: other
  artifact_id: spec.md#5
  related_ids:
  - TM-001
  path: spec/spec.md
  lines: 258-259
  excerpt: '[TM-001](tests.md) integrates FR-001 through FR-014, and TC-001 through TC-028.'
  finding: 'spec.md section 5 still says TM-001 integrates ''FR-001 through FR-014, and TC-001 through TC-028'' while this PR updates the same count in tests.md to FR-020/TC-034. The PR edits spec.md''s index for FR-020/TC-034, so the count belongs in this PR: change it to FR-001 through FR-020 and TC-001 through TC-034 (matching tests.md).'
bindings: []
```

+++
