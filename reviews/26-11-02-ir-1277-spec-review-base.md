---
id: SR-14840
title: spec-review-base — IR-1277 canonical driver APIs
type: SpecReview
analysis: base
scope: agent-ix/quire-driver; IR-1277; spec/functional/FR-001-orchestrate-the-driver-steps.md
review_set: subset
---

## Summary

FR-001 and AC-15 now explicitly name the original E5-admitted source alongside the unchanged lowered contract, positions, routed items and contexts. The ownership condition is observable through pointer-identity controls; occurrence and refusal obligations retain their original meaning. No new artifact IDs, relationships, pin, digest ledger or copied owner specification was added.

## Verdict

PASS for changed specification.

## Examined scope

```yaml
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
- id: FR-001-AC-13
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'The driver''s internal scalar-generation composition receives the caller''s
    unchanged QSL `Call`, `ObjectEnvironment`, and `Cancel` with the request''s compile
    input. QSL binding and IR admission use the package produced by that same request:
    the exact emitted bytes are admitted once and are not reconstructed or re-admitted.
    The generated result remains associated with that call and admitted package in
    the driver library.'
- id: FR-001-AC-14
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: For each routed scalar item selected for native proof, the driver joins
    the QSL item and CG driver item by request index and exact `OccurrenceKey`, then
    uses QSL `bind_scalar_operands` on that call's package and exact function/application/occurrence.
    Bound operands remain in QSL order, matching the CG scalar identity's argument
    order, with their argument source and ordinal; `ScalarLimits` come from the same
    call's accounting. Two occurrences on one node remain distinct. No positional
    zip or node-only occurrence selection is permitted.
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
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Reviewed the changed contract and nearby constraints, not repository-wide completion. Structural validation passes for 82 documents. Existing computed-matrix and strict-coverage debt is recorded by the separate gap analysis; this PASS grants no whole-repository coverage claim.
