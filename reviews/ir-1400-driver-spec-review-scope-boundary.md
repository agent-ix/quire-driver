---
id: SR-2026110705000829
title: spec-review/scope-boundary of driver current owner API compatibility
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-driver; IR-1400; Cargo.lock, spec/functional/FR-001-orchestrate-the-driver-steps.md,
  spec/functional/FR-016-retain-monitor-refusal-loci.md, src/drive.rs, src/handoff.rs,
  src/monitor/budget/tests.rs, src/monitor/error.rs, src/monitor/final_subject.rs,
  src/monitor/lowering/key.rs, src/monitor/material.rs, src/monitor/source_record.rs,
  src/monitor_refusal/copy.rs, src/refusal.rs, tests/drive.rs
review_set: subset
---

## Summary

Ticket: IR-1400. Reviewed the frozen PR 130 current-owner compatibility diff only. No defects found.

## Verdict

**PASS** within the authorized blocker scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Analysis

FR-001 is driver core orchestration plus cross-cutting refusal projection; FR-016 is driver infrastructure copying/accounting. IR owns contextual resolution, document-held identity and catalog reading; QSL owns compilation Extension code/cause; CG owns its catalog EnvelopeRefusal path/message. Driver consumes these APIs and retains their typed data. Native collision and projection tests verify the consumer boundary. IR catalog-failure creation remains assumed owner behavior because it exposes no supported external construction seam; driver propagation is inspected exhaustively. No new owner responsibility or dependency edge is assigned by these clarifications.

## Coverage

Supplied evidence inspected: /tmp/IR-1400-driver-check.log (library check), /tmp/IR-1400-driver-clippy.log (library and drive target), /tmp/IR-1400-driver-focused.log (five unit tests and one native regression), /tmp/IR-1400-driver-red.log (old lookup fails), /tmp/IR-1400-driver-fmt-check.log and /tmp/IR-1400-driver-spec.log. Reviewer independently ran the computed matrix and validation of the two edited documents. Cargo, aggregate, full workspace, cargo-deny, deep and Kani gates were not rerun per dispatch; the lead retains the final gate. This PASS is limited to the current owner API blocker, not integration #89 acceptance or IR-721 closure.

Plan completion: not assessed

No applicable AssuranceProfile exists under spec/. Optional repository-wide semantic gap analysis was not run. Integrity, dependency, object, app, risk and evidence lenses are not selected: no structure, relationship, domain object, UI, risk contract or verification-method change.

## Examined scope

```yaml
scope:
- id: FR-001-AC-5
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Each command-level refusal carries its step, catalog code and outcome kind.
    A `Compile` refusal's code renders `code/cause` when the refusing stage names
    a cause (e.g. `missing_import`/`missing-selection`), not just the bare code, including
    QSL's native extension refusal cause.
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
- id: FR-016-AC-10
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: A budget whose work and storage limits equal the consumption before the
    call plus the call's measured cumulative work and storage (scratch included) succeeds,
    and the receipt's `work` and `storage` equal those measured amounts; one unit
    below either returns `Work` or `Storage` whose payload equals the budget's first
    unpaid kind, amount and `PreparationOutput` locus, with no locus copied; pre-cancellation
    returns the original cancellation cause. Nested/shared keys, UTF-8 text and long
    native paths are included; arithmetic overflow refuses before allocation.
- id: FR-016-AC-11
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Every allocating copy/scratch reservation is fallible and precharged; reservation
    failure preserves TryReserveError, excess text capacity returns Capacity before
    conversion, and cleanup drops partial output without allocating or refunding a
    charge. No allocating native clone or infallible boxed-text shrink hides behind
    a successful receipt.
- id: FR-001-lookup
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: "  structural identity (package, path, ordinal) equals the item's `NodeKey`,\
    \ found by lookup. Lookup resolves node-held operand slots through their actual\
    \ holding rows in the admitted package; equal raw slot bytes alone do not select\
    \ a graph node. Driver requests carry the selected document-held ID unchanged.\
    \ Every item's node\n  comes from the same compiled package's own"
- id: FR-016-copy
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: 'The component SHALL construct the canonical `CheckedNodeId` fields directly:

    fallibly copy each root''s package text, path vector and `Name` text, retain

    `Index`, `Operand` and ordinal unchanged, and share each existing `Node(Arc<CheckedNodeId>)`

    by cloning that Arc. It SHALL NOT invoke the native allocating node/path Clone,

    serialize a key, allocate a replacement Arc, or duplicate an Arc''s subtree.

    The occurrence SHALL retain its native role and ordinal without allocation.

    The native `Vec<usize>` path SHALL be copied with fallible capacity reservation.

    After success, the locus SHALL remain rea'
- id: FR-016-work
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: 'Before inspecting or copying each represented record, container, segment
    tag,

    integer scalar or UTF-8 byte, the component SHALL charge work: one unit per

    record/container/tag, eight per integer scalar (including each `Operand` slot),
    and one per UTF-8 byte. Each Arc

    edge SHALL also incur one unit before sharing. Traversal SHALL be

    iterative and interruptible, with no semantic depth limit, whole-package scan

    or uncharged scratch collection. Cancellation SHALL be polled before every'
```
