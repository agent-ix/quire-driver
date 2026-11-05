---
id: SR-2026110414560046
title: IR-721 PR123 source-record spec-review/base
type: SpecReview
analysis: base
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

FR-018 and TC-032 define observable admission/projection behavior and distinct native outcomes, link US-001 and FR-015, and accurately retain Planned qualification. All four criteria have authored trace bindings. No new hash/pin/tracking mechanism is specified; native work receipts are discussed only as unavailable accounting, not demanded as file tracking.

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

## Continuation disposition and authentic positive evidence (round 3)

Round 3 continuation: no open or new static driver finding; FND-001 remains fixed. The merged QSL-1741 owner repair is adopted through the normal dependency branch, and the production source-record transport module is unchanged. The historical final producer-rename blocker is resolved for the exercised genuine PlanRoot fixture.

The initial unchanged positive reached successful native admission and then failed only its stale fixed two-row expectation: actual complete final producer/joined set was thirty. The test now requires a nonempty producer set and exact joined/producer count equality, and shares the complete row comparison with the ordinary-source positive. This strengthens the oracle rather than excluding generated records. Every original producer row is found by structural identity, all structural/source/declaration/opaque-key fields are compared, member counts are equal, and ordered original identities, admitted names, presence, type IDs and opaque Exact values are compared. Every admitted final record/member type ID resolves through the public native node_kind accessor. Both genuine generated origin_tick-to-origin and amount-to-value renames are required while original member identities remain unchanged. No caller row, driver rewrite/filter, patched emitted bytes or fabricated producer witness establishes the positive.

The supplied focused production log records all seven source_record_admission tests passing, zero failed/ignored/filtered. This includes the final public QSL PlanRoot emission, original-domain native evidence insertion, strict reading, actual driver admission and complete native joined rows. Ordinary independent callable parameter projections, native error payloads, structural-only counterfactuals, opaque authority distinction and stable error catalog also pass. Supplied formatting and changed FR-018/TC-032 validation logs complete without errors, with existing tooling advisories only. Documentation accurately reports the successful source-record slice while keeping whole-monitor qualification partial. Earlier monitor preparation/dependency-edge successes are historical receipts, not newly rerun evidence in this continuation.

Receipts examined: /tmp/ir721-record-qsl1741-positive.log (intermediate stale-count failure), /tmp/ir721-record-qsl1741-focused.log (final seven-test success), /tmp/ir721-record-qsl1741-fmt-check.log and /tmp/ir721-record-qsl1741-spec.log. No build/test/gate/census was run by reviewer. Root retains the ONE final scoped gate after artifact custody on the exact candidate. This review clears the source-record delta and confirms its supplied authentic positive evidence, not the final gate. Full IR-721 AC1, healthy Timed clock receipt, Program construction and cumulative preparation work/storage qualification remain Planned. Native join still provides no success-work receipt; no cumulative receipt is invented. QSL-1746/QSL-1747 remain untouched. IR-721 remains nonclosing.
