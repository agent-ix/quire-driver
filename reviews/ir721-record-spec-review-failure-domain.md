---
id: SR-2026110414560050
title: IR-721 PR123 source-record spec-review/failure-domain
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-driver; PR123; source-record admission changed seam
review_set: subset
---

## Summary

Ticket IR-721 (nonclosing). Independent static review of the original frozen PR123 source. Reviewer made no code edits. No applicable AssuranceProfile was found.

## Verdict

CONDITIONAL: no static defect found in this method; production qualification remains BLOCKED/UNRUN.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Strict reader failures stop before row mapping, native join errors produce no owner, package/identities cannot be rebound through public mutation, and complete rows outlive emission destruction. Cycles/recursive fields are owner-handled and native projections have independent work limits. Zero join and projection controls are authored. Arbitrary source-row names/opaque keys are not authenticated by IR structural validation; sealed QSL origin is the actual authority. No callbacks or caller evaluators are introduced.

Plan completion: not assessed

## Verification limits

Production tests are UNRUN: the focused production build failed in upstream CG replay/function.rs:1211 with E0004 (UnknownImport, UnknownImportedFunction, NotImportable), before driver compilation. The supplied log is /tmp/ir721-record-focused.log. No Cargo/build/test/gate/census/Kani was run by reviewer. Root retains the sole final scope gate. No runtime/full IR-721 AC1/clock/Program acceptance is claimed. Optional gap semantic fanout was not invoked; ordinary code/spec faithfulness was reviewed. Evidence advisor analysis skipped because the dispatch excludes broad advisor/census work. No enforced spec-review policy applies.

## Examined units

```yaml
scope:
- id: FR-018-AC-1
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Genuine ordinary source records emitted by public QSL and strictly admitted
    by IR retain all complete joined native rows and opaque record keys through driver
    admission/projection, including required, optional and Option fields and recursive/shared
    types where the owners admit them.
- id: FR-018-AC-2
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Native structural/source/owner/member-order/presence/type mismatch and
    duplicate controls return the original native error payload and work; independent
    join exhaustion returns no admitted owner. Counterfactual native-row controls
    establish structural checks only and never producer authority.
- id: FR-018-AC-3
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: The public API accepts genuine immutable Emission authority and binds its
    rows to its own strict reader output; caller-created row structs cannot enter
    production admission. Native reader refusal/incompleteness remain distinct.
- id: FR-018-AC-4
  path: spec/functional/FR-018-admit-source-record-witnesses.md
  role: examined
  excerpt: Projection uses the retained package and native identities with its independent
    limit, retaining original native type and complete source_record_view without
    schema reconstruction or invented member identities.
- id: src/monitor/source_record.rs
  path: src/monitor/source_record.rs
  role: examined
  excerpt: Complete changed file and its role in authentic source-record admission.
- id: tests/source_record_admission.rs
  path: tests/source_record_admission.rs
  role: examined
  excerpt: Complete changed file and its role in authentic source-record admission.
- id: src/monitor/mod.rs
  path: src/monitor/mod.rs
  role: examined
  excerpt: Complete changed file and its role in authentic source-record admission.
- id: Cargo.lock
  path: Cargo.lock
  role: examined
  excerpt: Complete changed file and its role in authentic source-record admission.
- id: spec/test-cases/TC-032-source-record-admission.md
  path: spec/test-cases/TC-032-source-record-admission.md
  role: examined
  excerpt: Complete changed file and its role in authentic source-record admission.
```

## Latest verdict

Round 1: statically clear within this method’s changed source-record seam; no open or new findings. Production qualification remains BLOCKED/UNRUN by the unchanged upstream CG exhaustive-match failure. No reviewer Cargo/build/test/gate ran. Root retains the sole final gate after the upstream repair and actual focused checks. IR-721 remains nonclosing. This does not establish production PASS or merge readiness.
