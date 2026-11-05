---
id: SR-2026110414560047
title: IR-721 PR123 source-record spec-review/ears-conformance
type: SpecReview
analysis: ears-conformance
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

Reviewed Description and Behavior obligations. Named driver/function/projection actors and concrete read/map/join/project responses are clear. Ubiquitous obligations fit this single admission operation; exclusions and qualification prose do not invent additional SHALL triggers. No ambiguous event-versus-state obligation was found.

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

## Continuation delta and production evidence (round 2)

Round 2 continuation: static driver source-record transport remains clear; FND-001 remains fixed and no new driver defect was found. Actual production qualification is PARTIAL/BLOCKED, not UNRUN: source_record_admission ran seven tests with six passing and the genuine final PlanRoot positive failing. monitor_preparation passed all twelve tests and dependency_edges passed its one test. The supplied scoped Clippy log completed successfully; root also reports formatting and changed-spec validation passed. Reviewer ran no builds, tests, gates or census.

The positive fails with the original native MismatchedIdentityWitness at Activation, requested/at preserved, work 197. The diagnostic prints the unmodified QSL row with original_member_identity=origin_tick and admitted_name=origin_tick; strict IR's actual binding is origin with the identical Integer type node. Owner source confirms final wire record bindings are renamed, while source-row carry rekeys structural/member type IDs and leaves admitted_name unchanged. Native IR correctly compares the actual binding name against admitted_name. The driver still copies all fields directly and correctly returns the native failure; it does not filter, rewrite, reseal, or return a partial owner. The unchanged healthy positive is retained as a failure, not converted to negative acceptance.

The test delta uses the public native node_kind accessor and adds a failure-only diagnostic that rereads the same original bytes/evidence and prints the original row/body before panicking. It adds no acceptance fallback or new production path. FR-018/TC-032 now honestly distinguish executed ordinary-source/control qualification from blocked final PlanRoot qualification. The old CG compile blocker is historical; the current blocker is the genuine QSL final binding-name witness defect.

Receipts examined: /tmp/ir721-record-production-tests.log, /tmp/ir721-record-production-clippy.log and /tmp/ir721-record-plan-root-owner-gap.log. No full cumulative preparation-work/storage qualification, native join success-work receipt, full IR-721 AC1, clock or Program completion is inferred. Final gate remains held for the producer repair and actual focused checks; root alone owns that gate. IR-721 remains nonclosing. Static clearance does not establish production PASS or merge readiness.
