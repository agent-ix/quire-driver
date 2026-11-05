---
id: SR-2026110414560048
title: IR-721 final subject spec-review/integrity
type: SpecReview
analysis: integrity
scope: agent-ix/quire-driver; IR-721; changed monitor final-subject seam, spec and
  tests
review_set: subset
relationships:
- target: ix://agent-ix/quire-driver/FR-015
  type: reviews
---

## Summary

Ticket: IR-721. Independent frozen PR #124 review. No applicable AssuranceProfile exists under spec/. New AC-15 is subordinate to FR-015 and TC-029. Exact mapping success is separated from AC-14 healthy clock admission, order/progress, closure and Program preparation. The documented acquisition/equality/owner-lookup exclusion is internally explicit; implementation accounting differs as recorded in code-review.

## Verdict

PASS — No defect found in this method’s changed-scope checks.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | implementation-bug-despite-evidence |

## Coverage

Reconciliation: quire matrix --scope . --format json; no run evidence store read. FR-015-AC-15: one tagged criterion, seven active test binders. Plan completion: not assessed. No reviewer builds, tests, Kani, pin checks or repository census were run, per dispatch. Coder logs examined: seven final-subject tests, seventeen preparation tests, eight lowering tests and nine focused unit tests passed; supplied formatter, Clippy and spec logs are supporting evidence, not independent execution. Reviewer spec validation passed. Dependency/object lenses were skipped because no relationship edge or domain-object definition changed; criterion-strength/Jev was not selected.

## Examined units

```yaml
scope:
- id: FR-015-AC-15
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: A real PlanRoot's original native subject and final emission admit one
    final-subject token only after strict native emission/package equality, owner
    declaration lookup, exact root/clause/Claim, full source-map, native name and
    existing independent selection checks.
- id: FR-015/native-ledger
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Driver identity/name comparisons, partial selection and resumed clock admission
    SHALL consume their actual existing ledger prefix.
- id: FR-015/native-payloads
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Native read refusals and incomplete outcomes SHALL retain their unchanged
    typed payloads; the outer codes SHALL be `selection` and `work`, respectively.
- id: FR-015-AC-14
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned pending authentic timed-profile clock emission and IR admission.
- id: src/monitor/final_subject.rs
  path: src/monitor/final_subject.rs
  role: examined
  excerpt: pub fn admit_monitor_final_subject<'a>(
- id: src/monitor/selection.rs
  path: src/monitor/selection.rs
  role: examined
  excerpt: pub(super) fn resume_clock<'a>(
- id: src/monitor/error.rs
  path: src/monitor/error.rs
  role: examined
  excerpt: '    EmissionIncomplete(Box<CheckedPackageIncomplete>),'
- id: src/monitor/mod.rs
  path: src/monitor/mod.rs
  role: examined
  excerpt: pub use final_subject::{admit_monitor_final_subject, AdmittedMonitorFinalSubject};
- id: tests/monitor_final_subject.rs
  path: tests/monitor_final_subject.rs
  role: examined
  excerpt: fn original_subjects_select_their_own_final_claims_and_reach_native_missing_clock()
    {
- id: spec/test-cases/TC-029-admitted-monitor-program-preparation.md
  path: spec/test-cases/TC-029-admitted-monitor-program-preparation.md
  role: examined
  excerpt: '### Final subject mapping component'
- id: spec/spec.md
  path: spec/spec.md
  role: examined
  excerpt: remain separate obligations. The partial native-subject API
- id: spec/tests.md
  path: spec/tests.md
  role: examined
  excerpt: The native final-subject API retains the genuine `MonitorSubject` and final
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: name = "qsl-package"
```

## Round 1 disposition checkpoint

Current verdict: PASS for the changed seam; both original code findings are fixed and no new defect was found. Original findings and initial verdict above are retained as history. The shared tests/common/monitor.rs now owns producer/document/admission/request construction for both suites; explicit companion source preserves the legacy same-clause twins and distinct mapping clauses. Direct package names, both complete native NodeKeys and native origin role/ordinal are charged before their corresponding comparisons or conversion. The borrowed iterative native walker accounts key/container/tag, ordinal and UTF-8 material and releases scratch on typed failure. Integration tests cover zero-work precedence, package exact/one-under boundaries and denial before comparison when the second native key cannot be inspected; unit tests cover nested UTF-8 exact/one-under work, storage denial and original cancellation cleanup.

The generic Budget.push_key retains the existing one-unit logical push schedule and actual-capacity storage accounting. Broad frontiers may cause repeated Vec reservations; the changed contract specifies identity/material traversal units and scratch storage, not allocator relocation CPU units or an aggregate runtime bound. No new relocation finding is asserted. The separate diagnostic-copy relocation schedule does not silently redefine this ledger. This conclusion grants no full preparation work/storage qualification.

Supplied production logs show eight final-subject, seventeen preparation, eight lowering and ten focused unit tests passed. Supplied Clippy, formatter and specification checks passed. These are coder execution receipts inspected by this independent reviewer; no reviewer builds or gates ran. The source mapping, native payload classification, full selected-root/source/name checks and genuine MissingClock continuation remain unchanged. AC-14 and whole Program preparation remain Planned.

### Round 1 examined units

```yaml
scope:
- id: FR-015/native-precomparison
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Native NodeKeys SHALL be walked iteratively through the owner's borrowed
    `view()`/path-segment views, without owned path extraction, Serde or identity
    copies.
- id: src/monitor/material.rs
  path: src/monitor/material.rs
  role: examined
  excerpt: impl Material for NodeKey {
- id: src/monitor/budget.rs
  path: src/monitor/budget.rs
  role: examined
  excerpt: '    pub(super) fn push_key<T>('
- id: tests/common/monitor.rs
  path: tests/common/monitor.rs
  role: examined
  excerpt: 'pub fn fixture(selected: &str, counter: &''static str, additional_clause:
    &str) -> Fixture {'
- id: tests/monitor_final_subject.rs/producer-material
  path: tests/monitor_final_subject.rs
  role: examined
  excerpt: fn producer_material_is_charged_before_early_identity_mismatches() {
- id: src/monitor/budget/tests.rs/native-material
  path: src/monitor/budget/tests.rs
  role: examined
  excerpt: fn native_key_borrowed_material_charges_nested_utf8_and_releases_all_scratch()
    {
```
