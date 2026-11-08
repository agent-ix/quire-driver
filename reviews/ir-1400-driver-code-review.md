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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | Monitor fixtures still access private PlanRoot fields and call equality_witness without its required NativeJoinBudget or Result handling, so the current-owner all-targets check cannot compile the test targets. | tests/common/monitor.rs:122 |

The lead's first final-gate step exposed nine compiler errors (E0616, E0061 and E0599) in the shared fixture; /tmp/IR-1400-driver-final-4eb6ac104.log records the failures. No later gate stage ran. This was missed by the original library/drive-target review evidence.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | The fixture retains the genuine PlanRoot, borrows its plan subject and original emission through owner getters, and handles equality_witness with a finite NativeJoinBudget and native Result. Final-subject callers retain pointer-identity assertions. |
| FND-003 | fixed | Source-record fixtures borrow original PlanRoot emission through its getter; the non-resource IncompleteIdentityWitness has configured_bound None. The current error-code catalog expectation includes CatalogUnavailable. Missing-evidence refusal is explicit; positive admission retains original producer evidence. |
| FND-004 | fixed | Occurrence membership now checks the resolved graph in the same document-held identity domain as the already validated term and structural key; exact role/ordinal membership remains required. |
| FND-005 | fixed | Select the same declaration-less graph node through resolved_graph(), retaining its document-held ID for native lowering and retaining native failure records in setup diagnostics. |
| FND-006 | fixed | The three driver helper selectors use resolved_graph document-held IDs for the same selected expression/operation/bound nodes, leaving all assertions unchanged. |
| FND-007 | fixed | The frame fixture matches the original QSL clause to IR structural_key_at(index) through the owner StructuralKeys adapter, then retains that row's resolved document-held ID. |

## Disposition pass 1 evidence and current verdict

**PASS — no open findings.** Inspected only tests/common/monitor.rs and tests/monitor_final_subject.rs changes between the pre-fix candidate and the fixture-fix candidate. The other five review artifacts remain unchanged. The original Findings table is preserved. FND-001 is solely the original clean placeholder and has no disposition.

Fixture::subject borrows root.plan().subject(); Fixture::emission borrows root.emission(). No cloned or reconstructed authoritative subject/emission replaces the genuine producer pair. Tests retain std::ptr::eq checks against those same borrowed originals. Other final-subject edits mechanically replace field access with getters; intentional malformed-subject tests retain their prior construction and assertions.

Equality witness acquisition receives its own finite 100,000,000 work allowance, 8 MiB storage allowance and 8 MiB read-storage allowance before the driver's independent preparation request. These are explicit fixture acquisition ceilings, not expected driver consumption or production defaults. Native failure panics the fixture with context; it is not suppressed or replaced by empty rows. Existing driver budget/cancellation assertions are unchanged. No source, dependency, specification or review artifact changed in the inspected fix diff.

Inspected supplied /tmp/IR-1400-driver-fixture-control.log: exact monitor-preparation selection control passed (one test); /tmp/IR-1400-driver-fixture-clippy.log: two-target scoped Clippy passed; /tmp/IR-1400-driver-fixture-fmt-check.log: formatting completed with existing nightly-format-option warnings. No Cargo or aggregate gate rerun by reviewer. Full final-subject behavioral suite and the remaining aggregate stages are not claimed executed by these controls; lead retains the final gate.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | high | Source-record fixtures access private PlanRoot emission fields and omit ExactTypeProjectionError.configured_bound, producing eleven compiler errors against current owner APIs. | tests/source_record_admission.rs:274 |
| FND-004 | high | Monitor occurrence membership compares raw node-held IDs with validated document-held IDs, rejecting a genuine member occurrence as OccurrenceMismatch when the native node contains Operand slots. | src/monitor_refusal.rs:152 |

FND-003 is demonstrated by /tmp/IR-1400-driver-continuation-3cb6e0a40.log (E0063 and E0616). The missing field belongs to ExactTypeProjectionError, not Dependency. FND-004 follows the native ID-domain mismatch: term and structural_key accept the document-held ID, while graph() returns raw Operand slots for the same node; raw equality therefore rejects its actual occurrence.

## Disposition pass 2 evidence and current verdict

**PASS — no open findings.** FND-002 remains fixed from round 1; FND-003 and FND-004 are fixed in this round. FND-001 remains the original clean placeholder. Original finding text and previous dispositions are preserved.

Inspected only the two requested batches between the pre-fix candidate and fixture/occurrence-fix candidate: tests/source_record_admission.rs, src/monitor_refusal.rs and tests/monitor_refusal.rs. Production diff is a single switch from graph() to resolved_graph(). No new identity minting, raw-slot reconstruction, foreign package selection, owner authority clone, fallback, cause conversion or work/storage accounting change is present.

Source-record fixture negative control supplies an explicit empty CheckedPackageEvidence both to the native owner reader and to the driver, asserting equality of the retained refusal. Its positive control uses the genuine PlanRoot emission/evidence with the original domain document. Getter changes retain the original root and borrowed emission. The manually constructed IncompleteIdentityWitness remains a semantic failure with no configured resource bound; existing native resource-refusal assertions and first-denial payload comparisons are retained. Error-catalog exact cardinality and spelling checks remain strict, now including the operational catalog condition added by this PR.

Monitor occurrence tests find document-held IDs in resolved_graph and assert that the authentic corresponding raw ID differs and contains Operand slots. They verify owner contextual resolution equals the structural key, then exercise genuine occurrence success. Wrong-role/root and foreign-ordinal controls still require OccurrenceMismatch. Neither occurrence membership nor term/structural-key validation is bypassed. This directly checks FR-016-AC-6/7; the existing trace binding remains correct, so no separate gap-artifact change is needed.

Supplied evidence inspected: /tmp/IR-1400-driver-fixture-all-compile-green.log (driver package test-target check passed); /tmp/IR-1400-driver-fixture2-control.log (exact native source-record positive/negative control passed); /tmp/IR-1400-driver-occurrence-control.log (exact native membership/negative control passed); /tmp/IR-1400-driver-fixture2-clippy.log and /tmp/IR-1400-driver-occurrence-clippy.log (scoped checks passed); corresponding fmt-check logs (existing nightly-option warnings only). Reviewer did not rerun Cargo or aggregate checks. CLI/workspace compilation and remaining lead gate stages are pending; no aggregate acceptance is claimed. Other five SR artifacts remain unchanged.

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | high | Provider REAL_ADMISSION fixture passes a raw Operand-slot node ID to document-held lowering, so native admission succeeds but lowering returns no wire package and poisons the shared fixture, failing two provider preflight controls. | src/provider.rs:505 |

## Disposition pass 3 evidence and current verdict

**PASS — no open findings.** FND-002 through FND-004 retain their fixed dispositions; FND-005 is fixed. FND-001 remains the original clean placeholder.

Inspected only src/provider.rs changes between the requested pre-fix and fixed candidates. Both changed expressions are under cfg(test) mod tests. The original compile_package source, E5 reader, evidence, bounded limits, declaration-less-node predicate and node order are unchanged. IR resolved_graph returns the same positional nodes with document-held IDs; the fixture selects the same original node rather than inventing or cloning authority from another package. checked.lower receives that document-held ID and its unchanged routed_contract_lowering_profile. Native feature_accounting and source-map occurrences are still read from the real admitted lower output. No lowered receipt, accounting value, resource ceiling or first-denial result is fabricated or changed.

The native red log shows successful E5 admission followed by Failed lowering records containing Operand(0), rather than an admission refusal. The first provider test panics on absent wire output and the second fails due to the poisoned LazyLock. Switching only the identity view restores native lowering. Enhanced setup diagnostics expose lowered.records on failure; they do not recover from the failure or bypass any assertion. Existing cancellation/backend-mismatch and missing-limits/missing-harness negative assertions are unchanged.

Evidence inspected: /tmp/IR-1400-driver-continuation2-db5cc1809.log (driver library 98 passed, two failed, one ignored); /tmp/IR-1400-driver-provider-native-red.log (native lowering failure with raw Operand ID); /tmp/IR-1400-driver-provider-cancel-control.log and /tmp/IR-1400-driver-provider-limits-control.log (each affected exact control passes independently); /tmp/IR-1400-driver-provider-clippy.log (passed); /tmp/IR-1400-driver-provider-fmt-check.log (existing nightly-option warnings). No actual Kani engine was run or claimed. Reviewer reran no Cargo or aggregate gate. Remaining integration/plugin-host/doc/control stages belong to the lead; this verdict does not imply those stages passed.

Other five review artifacts remain unchanged. Existing FR-005-AC-2 provider preflight evidence bindings remain applicable; no new behavior or criterion was introduced by this fixture correction.

## New findings (disposition pass 4)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | Driver acceptance helpers use raw node-held Operand IDs as expected CG document-held selections and native forms, causing nine assertion failures after the production identity correction. | tests/drive.rs:868 |
| FND-007 | high | Frame-replay fixture decodes raw node-held IDs through WireGraph without holding-row resolution, failing with OperandSlot(0) before all five native frame tests can reach their assertions. | tests/frame_replay.rs:129 |

## Disposition pass 4 evidence and current verdict

**PASS — no open findings.** FND-002 through FND-005 retain fixed dispositions; FND-006 and FND-007 are fixed. FND-001 remains the original clean placeholder.

Inspected only tests/drive.rs and tests/frame_replay.rs between the supplied frozen candidates. Driver helper predicates for expression semantic form, operation identity and integer-range bounds remain identical; only the owner's graph view changes to document-held IDs. No expected positive generation output, native-form equality assertion, request ordering, unsupported-sibling outcome or bound assertion was weakened.

The frame fixture retains the original QSL call_site clause. For each admitted positional row it reads structural_key_at(index), converts that complete structural key using the QSL owner's StructuralKeys adapter and compares it with clause.site.node. It retains the matching resolved row's exact document-held ID for CG StateFrameRequest. The old raw-slot WireGraph conversion is removed. It does not pick by raw slot equality, infer a new semantic identity, replace the clause with another item, or reconstruct an authority from text. The original request occurrence/index join, generated harness identity, inheritance/declaring-owner expectations and admitted-package comparisons remain unchanged. Diff inspection confirms no edits to the tests asserting QSL refusal precedence, missing/changed/caller-populated associations or public-inventory rebinding rejection.

Reviewed all fourteen /tmp/IR-1400-driver-round4-receipts.json entries against their actual logs /tmp/IR-1400-driver-r4-01.log through /tmp/IR-1400-driver-r4-14.log. Every entry has exit 0, its named exact test reports ok, and each log reports one passed and zero failed. These are nine driver controls plus five frame-replay controls, covering positive generation/native forms and unchanged refusal/association/rebinding negatives. /tmp/IR-1400-driver-unrun-tail-6da5bfe2e.log records the earlier nine drive assertion failures and five OperandSlot setup failures, plus the successful remaining integration/plugin-host/doc/adapter runs. /tmp/IR-1400-driver-fixture4-clippy.log passes; /tmp/IR-1400-driver-fixture4-fmt-check.log carries only existing nightly-option warnings.

No production, dependency, specification or original review artifact is changed by this fix diff. No actual Kani execution, new budget assumption, exception or test seam is introduced. Reviewer ran no Cargo, aggregate or broad test repeat. All fourteen reported failures now have inspected exact green controls; lead owns the complete gate checklist and merge decision. Other five SRs remain unchanged: this fixture-only correction changes no acceptance contract or trace binding.
