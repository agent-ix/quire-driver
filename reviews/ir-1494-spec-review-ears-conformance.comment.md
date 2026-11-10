<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14856 method=spec-review/ears-conformance lang=spec pr=quire-driver#146 model=claude-opus-5-5 run=87752af5-6144-4b92-9817-ee41c4ca9989 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |

Verdict: PASS for grammar. Every Behavior statement is ubiquitous ('The driver SHALL ...') or unwanted-behaviour ('If ..., then the driver SHALL ...') with the driver as subject; quire validate reports no ears warning on FR-020 (the two grammar warnings are on unchanged FR-019). Content defects in these statements are recorded under base, dependency, failure-domain and criterion-strength; the policy sentence in the dependency bullet is covered by the dependency review.

+++ [reviewer data]

```yaml
clean: true
scope:
- id: FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: 'One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item''s `RoutedRequirement`, and the package''s `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), "Per-occurrence requirement ...'
- id: FR-020#behavior-occurrence
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- The driver SHALL project `RequirementOccurrence` from a QSL `OccurrenceKey` as follows: `node_id` is the key''s `NodeKey` resolved to the admitted package''s V2 `CheckedNodeId` by the same lookup FR-001 uses for an item''s node; `role` and `ordinal` are the key''s role and ordinal, by value. An unknown QSL role is the existing `UnknownOccurrenceRole` fault.'
- id: FR-020#behavior-records
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- The driver SHALL build `RequirementRecords` from every record of the compiled package''s own `requirements()`, projecting each record''s `ResultBound::node()`, conversion occurrence and path condition exactly as for an item. It SHALL NOT build a record from an item, from the checked graph, or from any source other than QSL''s record at that key.'
- id: FR-020#behavior-guards
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- The driver SHALL add one `ContextRoot` position for each path guard, after any conversion root and in outermost-first order, inside that item''s contiguous context span, as CG FR-022 "Path conditions in the scalar harness" requires. Context roots SHALL NOT advance driver indices (FR-001).'
- id: FR-020#behavior-missing
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- If the package''s requirement records hold no entry for a routed item''s occurrence, then the driver SHALL refuse the run with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and the item''s occurrence, before calling CG generation.'
- id: FR-020#behavior-duplicate
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- If two distinct QSL keys project to one `RequirementOccurrence`, then the driver SHALL refuse the run with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index of the first item that reaches the repeated occurrence, before calling CG generation.'
- id: FR-020#behavior-cg-refusal
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- A CG `RoutedGenerationError::RequirementRecord { request_index, occurrence, mismatch }` (any `RequirementRecordMismatch`: `RequirementMissing`, `OccurrenceNotSelected`, `RecordMissing`, `ApplicationMismatch`, `ResultBoundMismatch`, `ConversionContextMismatch`, `PathConditionMismatch` or `GuardContextMissing`) means the driver''s projection disagreed with the authenticated selection. The driver SHALL preserve it unchanged in `DriverRefusal::Generation`, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault.'
findings: []
bindings: []
```

+++
