---
id: SR-14841
title: spec-review-integrity — IR-1277 canonical driver APIs
type: SpecReview
analysis: integrity
scope: agent-ix/quire-driver; IR-1277; spec/functional/FR-001-orchestrate-the-driver-steps.md
review_set: subset
---

## Summary

Behavior prose and AC-15 agree on borrowing the original admitted source and sharing its retained owner. Existing position-vector ordering, excluded items, separate context roots and driver-owned QSL sidecars remain intact. No contradiction with the typed mismatch/invariant distinction was found; no ID, section or relationship is removed.

## Verdict

PASS.

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
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Identity confusion, package/frame ownership, absence versus exclusion, refusal stages and observable negative controls were checked in the changed scope. No new domain object or dependency relationship requires a separate object/dependency analysis.
