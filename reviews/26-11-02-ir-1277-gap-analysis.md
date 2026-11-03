---
id: SR-14843
title: gap-analysis — IR-1277 canonical driver APIs
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-driver; IR-1277; spec/functional/FR-001-orchestrate-the-driver-steps.md,
  src/drive.rs
review_set: subset
---

## Summary

The new indexed-wire regression executes natively but its fully qualified trace attribute is only an evidence_near_miss in the declared scanner, so it contributes no verifies binding. Other changed controls bind the source-retention, refusal and occurrence criteria and exercise real QSL/IR/CG paths. No stub, synthetic Complete result, copied owner evaluator or coverage narrowing was found in the changed implementation.

## Verdict

CONDITIONAL for changed scope; repository matrix FAIL.

## Examined scope

```yaml
- id: FR-001-AC-5
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Each command-level refusal carries its step, catalog code and outcome kind.
    A `Compile` refusal's code renders `code/cause` when the refusing stage names
    a cause (e.g. `missing_import`/`missing-selection`), not just the bare code.
- id: FR-001-AC-12
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: For each request item, the driver's CG negotiation input carries the typed
    IR tag/form projected from the admitted checked requirement and node named by
    that same item's QSL requirement record; generation preserves that item's exact
    application occurrence and direct conversion-context occurrence as ordered lowering
    roots, including when several items share one application node. No caller-computed
    lowerability/refusal or display/debug-derived form substitutes for the checked
    form.
- id: FR-001-AC-15
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: CG receives the original E5-admitted `CheckedPackageV2` by reference alongside
    its existing admitted `ContractPackageV1`, complete FR-330 position vector, routed
    items, and generation contexts. The driver retains and shares the original admitted
    source owner without reconstruction or deep copying. The position vector retains
    one position per original request item, in driver-index order (a `DriverItem`
    for each bound item, an excluded position, keeping the item's `node_id` and counted
    in driver indices, for each export-refused item; excluded positions are absent
    from the FR-331 envelope), and a
- id: FR-001-AC-16
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'No selected scalar item means no scalar binding or native execution and
    no fallback selection. QSL''s typed missing/ambiguous call or occurrence refusal
    is preserved unchanged. A missing/ambiguous required conversion context preserves
    its owning QSL/IR/CG typed refusal; `None` is valid only when the exact QSL item
    has no conversion context. An admitted-package/request-position/routed-item mismatch
    preserves the owning IR/CG typed refusal, or is a driver `runtime_invariant` fault
    when the driver detects an inconsistent join. Each case stops before native execution
    or replay and invents no call, '
- id: FR-001-AC-22
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver
    emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG
    projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`,
    `document_pointer`, `expected` and `actual`. The diagnostic carries the same code
    and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location,
    and carries `locus` as a `CheckedNodeId` payload when present; every returned
    code has an FR-271 row. When CG returns `ContractPackageReadIncomplete`, the driver
    emits `stage_limit_exc'
- id: indexed_wire_id_is_selected_by_its_resolved_operand_key
  path: src/drive.rs
  role: examined
  excerpt: '#[ix_trace_rs::trace("TC-006", "FR-001-AC-12")]'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new indexed-wire regression uses a qualified trace attribute that the declared scanner treats only as an evidence near miss; import trace and use #[trace(...)] so the control binds FR-001-AC-12. | src/drive.rs:1392-1394 |

## Coverage

Plan completion: not assessed

Computed repository matrix: 136 tagged, 47 untagged, 4 method-without-symbol, 1 tagged-by-ignored-test. Repository strict coverage: 41 unbacked and 14 contradicted statuses. These broader existing obligations were not changed or accepted by this migration; no repository-wide PASS is granted. Scope is the changed driver API/identity/refusal behavior, with the missed binding reported above. Optional semantic gap fan-out was not selected. No matrix was hand-edited.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The test module imports the trace macro and uses its supported unqualified attribute. Inverse trace lookup now reports canonical verifies claims for FR-001-AC-12 and TC-006, with no near-miss citations. |

## Disposition verdict

PASS for the changed trace binding; no findings remain open in this review. The original repository-wide matrix and strict-coverage debt remain outside this migration and are not approved as complete. Downstream monitor qualification remains subject to the parent lane's producer-failure investigation and final gate; this disposition grants no RT executor or producer qualification.
