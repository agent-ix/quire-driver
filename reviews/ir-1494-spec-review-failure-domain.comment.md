<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14858 method=spec-review/failure-domain lang=spec pr=quire-driver#146 model=claude-opus-5-5 run=87752af5-6144-4b92-9817-ee41c4ca9989 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | untestable-ac | RecordMissing/AC-4 reachable only via an undefined 'requirements view' seam; cause name collides with CG's RecordMissing |
| FND-002 | medium | soundness | No fault/stage/index for a package record (not tied to a routed item) that fails to resolve |
| FND-003 | medium | coverage | Shared guard node across two items' spans is unspecified; AC-7's single-item vector lets a dedup mutant pass |

Verdict: Needs changes. Frames, the no-substitution rule and CG passthrough are covered. Three failure modes are unspecified or rest on an undefined seam.

+++ [reviewer data]

```yaml
clean: false
scope:
- id: FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: 'One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item''s `RoutedRequirement`, and the package''s `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), "Per-occurrence requirement ...'
- id: FR-020-AC-4
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: When the requirement records hold no entry for a routed item's occurrence, the run refuses with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record.
- id: FR-020-AC-5
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: When two distinct QSL keys project to one `RequirementOccurrence`, the run refuses with the `RequirementRecord` fault, cause `RecordDuplicate`, naming the request index; stage, wire code and kind are those of FR-020-AC-4 and CG generation is not called.
- id: FR-020-AC-7
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them.
- id: FR-020#behavior-missing
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- If the package''s requirement records hold no entry for a routed item''s occurrence, then the driver SHALL refuse the run with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and the item''s occurrence, before calling CG generation.'
- id: FR-020#behavior-records
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- The driver SHALL build `RequirementRecords` from every record of the compiled package''s own `requirements()`, projecting each record''s `ResultBound::node()`, conversion occurrence and path condition exactly as for an item. It SHALL NOT build a record from an item, from the checked graph, or from any source other than QSL''s record at that key.'
- id: FR-020#behavior-guards
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  role: examined
  excerpt: '- The driver SHALL add one `ContextRoot` position for each path guard, after any conversion root and in outermost-first order, inside that item''s contiguous context span, as CG FR-022 "Path conditions in the scalar harness" requires. Context roots SHALL NOT advance driver indices (FR-001).'
- id: TC-034#procedure
  path: spec/test-cases/TC-034-project-requirement-records-into-codegen.md
  role: examined
  excerpt: 1. Check a QSL unit with one node carrying two occurrences (different ordinal, result bound and path condition), one with a conversion context and two guards, and the IR-304 guarded package. Capture what the driver passes to a recording CG seam (AC-1 to AC-3, AC-6, AC-7). 2. Drive the projection function over a requirements view with the item's key removed (AC-4), and over a view in which two keys map to one occurrence (AC-5). Assert the fault fields, code, stage, kind and that the recording seam saw no generation call. 3. Reuse the FR-010-AC-4 partition vector, once as is and once with B's...
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  method: spec-review/failure-domain
  check: untestable-ac
  artifact_id: FR-020-AC-4
  related_ids:
  - FR-020-AC-5
  - TC-034
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: Behavior RecordMissing bullet; AC-4; TC-034 step 2
  excerpt: When the requirement records hold no entry for a routed item's occurrence, the run refuses with the `RequirementRecord` fault, cause `RecordMissing`, naming the request index and occurrence, with stage Generation, wire code `runtime_invariant` and outcome kind internal fault; CG generation is not called and the result carries no settlement built from a substitute record.
  finding: 'Items and records both come from the same CheckedGraph::requirements() (items_from_requirements), so in production a routed item always has its record: AC-4 is reachable only through the ''projection function'' over a ''requirements view'' that TC-034 step 2 names and FR-020 never defines. The repo''s seam is the Upstream trait (src/upstream.rs: tests wrap QslCg and still run the real functions); it cannot hand the driver a record set separate from the compiled package (QSL exposes no public record mutator outside cfg test-support). The projection''s signature (items and records as separate inputs, unit-tested directly) is therefore a hidden coder decision; state it in FR-020, or drop the driver pre-check and rely on CG''s RecordMissing passthrough (AC-9). Also the driver cause name RecordMissing collides with CG''s RequirementRecordMismatch::RecordMissing, giving two refusal shapes (Fault vs Generation) for one condition.'
- fnd: FND-002
  severity: medium
  confidence: medium
  method: spec-review/failure-domain
  check: soundness
  artifact_id: FR-020
  related_ids:
  - FR-020-AC-2
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: Behavior 'build RequirementRecords'
  excerpt: 'One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item''s `RoutedRequirement`, and the package''s `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), "Per-occurrence requirement ...'
  finding: Projecting every package record resolves each record's result-bound, conversion and guard NodeKeys in the admitted package, including records of excluded or unrouted items. A record whose node does not resolve, or resolves ambiguously, or whose guard role is unknown, hits UnknownNode/AmbiguousNode/UnknownOccurrenceRole, which carry a request_index that a package record does not have. FR-020 states no fault, stage or index for a record that is not any routed item's, nor whether such a record stops the run.
- fnd: FND-003
  severity: medium
  confidence: medium
  method: spec-review/failure-domain
  check: coverage
  artifact_id: FR-020-AC-7
  related_ids:
  - FR-020
  path: spec/functional/FR-020-project-requirement-records-into-codegen-input.md
  lines: Behavior ContextRoot bullet; AC-7
  excerpt: For an item with two path guards and a conversion context, the request positions hold the conversion `ContextRoot`, then one `ContextRoot` per guard in outermost-first order, all inside the item's context span, and the next item's driver index is unchanged by them.
  finding: The canonical guarded vector (CG FR-022-AC-27, `if p then x + 1 else x + 1`) routes two items that share one guard node p (for `if p` the guard node is the parameter node). FR-020 neither says a shared guard node is repeated as a ContextRoot in each item's span nor that each guard node joins the lowering roots (lower_driver_request pushes roots and positions together). AC-7 uses one item, so a mutant that emits a shared guard root once passes AC-7 while CG's per-span check (GuardContextMissing, slice (b)) or the FR-039 position check refuses. Add the two-item shared-guard case to AC-7.
bindings: []
```

+++
