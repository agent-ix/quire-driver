<!-- reviewer repo=agent-ix/quire-driver visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-14860 method=code-review lang=rust pr=quire-driver#147 model=claude-opus-5-5 run=c649c842-70a7-46db-92d9-c4e448adac7c -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | high | coverage | Path-condition / guard-root order untested: every fixture has one guard per item; reversing the order (M1) survives all 44 drive tests |
| FND-002 | low | test-intent | KaniReaderCatalogUnavailable mapping unpinned (M4 survives); comment claims a test that does not exist |
| FND-003 | low | test-intent | direct_generation discards the test-built item and feeds CG the driver's own items and records |
| FND-004 | low | other | Cargo.lock mixes two QSL main commits (48a12edb for qsl-analyze/command/route, 91ada09a for the rest) |
| FND-005 | low | soundness | Record-level RequirementRecordUnresolved can preempt FR-001's request-indexed item faults (projection runs first) |
| FND-006 | low | other | lookup_checked_node duplicates resolve_checked_node; resolved-graph iterator repeated 4x |

Verdict: NOT mergeable (FND-001). Tests: drive 44/44, refusal lib 32/32 at 2d9b9220. Mutations: M1 reverse path condition SURVIVES; M2 drop conversion root -> 22 fail incl. guarded_occurrences_carry_their_own_requirement_and_guard_roots; M3 swap KaniReaderRefused/KaniUnsupportedBody codes -> kani_reader_refused_is_a_runtime_invariant_fault + kani_unsupported_body_is_an_invalid_request fail; M4 CatalogUnavailable -> invalid-request SURVIVES; M5 guard roots before conversion root -> guarded_occurrences_... fails. Clean: exhaustive arms (no new wildcards), distinct messages, no new src panics/unwraps, branch = "main" in both manifests, no pins/shims, trace tags present, additive documented public API. RR16_UNIT edit is not a weakening; 'invalid-request' matches the catalog wire spelling (pre-existing src/refusal.rs:1368).

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: src/drive.rs#project_requirements, path: src/drive.rs, role: examined, excerpt: "FR-020: project the QSL requirement of every export-bound value-validity item that has a record, and the package's whole set of value-validity records, from the one map."}
  - {id: src/drive.rs#lower_driver_request, path: src/drive.rs, role: examined, excerpt: "FR-020: one guard root per path guard, outermost first, inside this item's own context span; a guard shared by two items is rooted in each."}
  - {id: src/drive.rs#generate, path: src/drive.rs, role: examined, excerpt: "A Kani-routed item whose lowered requirement is None refuses DriverFault::RequirementRecordAbsent { request_index, occurrence }."}
  - {id: src/refusal.rs#oracle_code, path: src/refusal.rs, role: examined, excerpt: "KaniReaderRefused | KaniReaderCatalogUnavailable => RuntimeInvariant; KaniReaderIncomplete => Diagnostics ? RuntimeInvariant : StageLimitExceeded; KaniOwnerPathRefused => owner_path_code; KaniUnsupportedBody | KaniNestedCallBody => InvalidRequest"}
  - {id: src/refusal.rs#generation_code, path: src/refusal.rs, role: examined, excerpt: "RoutedGenerationError::RequirementRecord { .. } => Code::RuntimeInvariant"}
  - {id: src/upstream.rs#requirement_records, path: src/upstream.rs, role: examined, excerpt: "fn requirement_records(&self, package: &CheckedPackage) -> RequirementMap { package.graph().requirements().clone() }"}
  - {id: Cargo.toml, path: Cargo.toml, role: examined, excerpt: "quire-contract-codegen = { git = \"https://github.com/agent-ix/quire-contract-codegen\", branch = \"main\" }"}
  - {id: tests/drive.rs#RR16_UNIT, path: tests/drive.rs, role: examined, excerpt: "function f using v(x: Int[0, 9]): Boolean pure { k(g(x + 1), h(x + 1)) }"}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: code-review
    check: coverage
    artifact_id: tests/drive.rs#guarded_occurrences_carry_their_own_requirement_and_guard_roots
    related_ids: [FR-020-AC-1, FR-020-AC-7, src/drive.rs#lower_driver_request]
    path: tests/drive.rs
    lines: "3427-3523"
    excerpt: |-
      assert_eq!(
          guard_root_total, 2,
          "the guard node is rooted once per guarded item"
      );
    finding: "Path-condition and guard-root order is untested: every fixture has one guard per item, so reversing the path condition in the projection and the guard roots (M1) survives all 44 drive tests."
  - fnd: FND-002
    severity: low
    confidence: high
    method: code-review
    check: test-intent
    artifact_id: src/refusal.rs#kani_oracle_cases
    related_ids: [src/refusal.rs#oracle_code]
    path: src/refusal.rs
    lines: "2065-2066"
    excerpt: |-
      /// `KaniReaderCatalogUnavailable` wraps IR's `CatalogReadError`, which has
      /// private fields and no constructor; its own test inspects the source.
    finding: "KaniReaderCatalogUnavailable's mapping is unpinned (M4 to invalid-request survives) and the case-table comment claims its own test inspects the source, but no such test exists."
  - fnd: FND-003
    severity: low
    confidence: medium
    method: code-review
    check: test-intent
    artifact_id: tests/drive.rs#direct_generation
    related_ids: []
    path: tests/drive.rs
    lines: "1308-1359"
    excerpt: |-
      let item = item.with_scalar_selection(&package, selection);
      let driver_item = passed
          .iter()
          .find(|driver| driver.request_index == item.request_index)
          .expect("the driver passed the expected request index");
      driver_item.clone()
    finding: "direct_generation builds the expected item, then discards it and feeds CG the driver's own passed item and records, so the *_passes_through_unchanged tests no longer check the routed item against an independent oracle."
  - fnd: FND-004
    severity: low
    confidence: high
    method: code-review
    check: other
    artifact_id: Cargo.lock
    related_ids: []
    path: Cargo.lock
    lines: "-"
    excerpt: |-
      qsl-route source = "git+https://github.com/agent-ix/quire-spec-language?branch=main#48a12edb628236a3bf2e866155a3c36fb763d1d1"
      qsl-semantics source = "git+https://github.com/agent-ix/quire-spec-language?branch=main#91ada09a785432803ee980fe2b9bf4b0f1c5a2b7"
    finding: "Cargo.lock now holds QSL crates from two commits of the same main branch (qsl-analyze, qsl-command, qsl-route at 48a12edb; the other seven at 91ada09a, two commits later); origin/main had one QSL commit."
  - fnd: FND-005
    severity: low
    confidence: medium
    method: code-review
    check: soundness
    artifact_id: src/drive.rs#lower_driver_request
    related_ids: [FR-020]
    path: src/drive.rs
    lines: "1001"
    excerpt: |-
      let projection = project_requirements(checked, items, partition, map)?;
    finding: "project_requirements runs before the per-item loop, so a record at an item's own occurrence that fails lookup raises RequirementRecordUnresolved (no request index) before FR-001's request-indexed UnknownNode/AmbiguousNode/UnknownOccurrenceRole; FR-020 says the item's occurrence resolves first."
  - fnd: FND-006
    severity: low
    confidence: high
    method: code-review
    check: other
    artifact_id: src/drive.rs#lookup_checked_node
    related_ids: [src/drive.rs#resolve_checked_node]
    path: src/drive.rs
    lines: "802-820"
    excerpt: |-
      fn lookup_checked_node<'a>(
          checked: &'a CheckedPackageV2,
          node: &quire_exact::NodeKey,
      ) -> Result<&'a CheckedNodeId, NodeLookup> {
    finding: "lookup_checked_node duplicates resolve_checked_node (same structural_key_matches walk, different error type), and the resolved-graph iterator expression is now repeated four times."
bindings:
  - {test_id: guarded_occurrences_carry_their_own_requirement_and_guard_roots, ac_id: FR-020-AC-7, trace: correct}
  - {test_id: every_requirement_record_mismatch_is_a_distinct_generation_fault, ac_id: FR-020-AC-9, trace: correct}
  - {test_id: kani_reader_refused_is_a_runtime_invariant_fault, ac_id: FR-001-AC-24, trace: correct}
```

+++
