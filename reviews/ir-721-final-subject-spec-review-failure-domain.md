---
id: SR-2026110414560049
title: IR-721 final subject spec-review/failure-domain
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-driver; IR-721; changed monitor final-subject seam, spec and
  tests
review_set: subset
relationships:
- target: ix://agent-ix/quire-driver/FR-015
  type: reviews
---

## Summary

Ticket: IR-721. Independent frozen PR #124 review. No applicable AssuranceProfile exists under spec/. Strict acquisition has explicit native limits and unchanged refusal/incomplete payloads. Original-subject package/node/role, absent lookup, same-emission other declaration, full independent source row and cancellation failure domains are specified. No callback or new graph traversal was introduced; the owner linear scan remains an explicit unmetered synchronous limitation.

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
