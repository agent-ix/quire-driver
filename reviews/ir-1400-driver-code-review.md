---
id: SR-2026110705000824
title: code-review of driver current owner API compatibility
type: SpecReview
analysis: code-review
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

The four selectors in generate, lower_driver_request, contract_occurrence and project_kani_form enumerate resolved_graph().nodes and obtain structural_key_at(index). Upstream's resolved_graph and document_held_id expose the same wire semantic_graph node IDs; no raw-slot lookup, foreign-package alias, key minting or identity parsing is added. Existing resolve_checked_node still rejects zero or multiple structural matches. Numeric Index and Operand cannot match a native structural key.

All three IR dispatch boundaries move CatalogReadError into typed envelopes without formatting or conversion. CG's original EnvelopeRefusal retains path/message, returns runtime_invariant/InternalFailure/Fault at PackageRead and produces no malformed-package/limit diagnostic. QSL Extension retains its native refusal and uses its typed code/cause. Operand copying charges tag plus eight numeric work units before preserving the slot; shared traversal retains the same Arc and ledger path. No new allocation occurs in that arm; existing cancellation, storage and winning-failure order remain. The monitor key converter rejects raw numeric axes without minting a structural key.

Searched changed helpers, surrounding modules and owner APIs for duplicated logic or catalog vocabulary. These are consumer matches over owner enums, with no vendored schema, new shim, new gate, hidden clone, unsafe code, narrowed CI lane or changed limit. Cargo.lock refresh retains normal main dependencies; no manifest pin or new product tracking mechanism is introduced. Broad existing hash cleanup is outside the user's explicit frozen scope.

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
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: '# This file is automatically @generated by Cargo.

    # It is not intended for manual editing.

    version = 4


    [[package]]

    name = "agent-ix-extraction-frontend"

    version = "0.2.1"

    source = "git+https://github'
- id: src/drive.rs
  path: src/drive.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! [`drive`]: the driver''s steps, in ADR-011 T-13 order.


    use std::collections::BTreeMap;

    use std::path::Path;

    use std::sync::Arc;


    use qsl_foundation::s'
- id: src/handoff.rs
  path: src/handoff.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! E5 (ADR-011 \xA72.1):\
    \ the E4 v2 bytes handed to IR's checked-package reader,\n//! The complete E4\n\
    //! value remains available through admission for bounde"
- id: src/monitor/budget/tests.rs
  path: src/monitor/budget/tests.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Live storage accounting\
    \ across retained allocations and key scratch.\n\nuse std::sync::{\n    atomic::{AtomicUsize,\
    \ Ordering},\n    Arc, OnceLock,\n};\n\nuse"
- id: src/monitor/error.rs
  path: src/monitor/error.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Typed monitor preparation\
    \ stops, separate from runtime outcomes.\n\nuse quire_contract_model::{\n    CheckedCallableReadError,\
    \ CheckedCallableReadErrorCo"
- id: src/monitor/final_subject.rs
  path: src/monitor/final_subject.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Bind an original QSL subject through its owner''s final declaration lookup.


    use qsl_package::Emission;

    use qsl_replay::MonitorSubject;

    use quire_contr'
- id: src/monitor/lowering/key.rs
  path: src/monitor/lowering/key.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Iterative conversion of original structural fields through the native key
    constructor.


    use quire_contract_model::{CheckedNodeId, CheckedNodePathSegme'
- id: src/monitor/material.rs
  path: src/monitor/material.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Charged borrowed material\
    \ walks; structural keys never use recursive Serde.\n\nuse quire_contract_model::{\n\
    \    CheckedArtifactRef, CheckedNodeId, Checke"
- id: src/monitor/source_record.rs
  path: src/monitor/source_record.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! Authentic QSL source-record\
    \ witnesses joined to their own strict IR admission.\n\nuse qsl_package::Emission;\n\
    use quire_contract_model::{\n    read_checke"
- id: src/monitor_refusal/copy.rs
  path: src/monitor_refusal/copy.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! Bounded copying of supplied native diagnostic axes, without resolution.


    use std::{collections::TryReserveError, mem::size_of, sync::Arc};


    use quire_'
- id: src/refusal.rs
  path: src/refusal.rs
  role: examined
  excerpt: "// SPDX-License-Identifier: AGPL-3.0-or-later\n//! The driver's command-level\
    \ refusals (ADR-011 \xA72.3, \xA75; ADR-012 \xA77.2).\n//! AGE-394 (the lifecycle\
    \ and CLI design) renders them; this module only types\n"
- id: tests/drive.rs
  path: tests/drive.rs
  role: examined
  excerpt: '// SPDX-License-Identifier: AGPL-3.0-or-later

    //! FR-001 acceptance tests: the driver''s step order, the MD-2 refusal,

    //! routing after negotiation, the E5 hand-off and CG generation. Every call goes '
```
