---
id: SR-122001
title: IR-721 clock admission code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-driver; PR122 nine-file clock-admission delta
review_set: subset
---

## Summary

Independent static review of original frozen PR122 source. Ticket: IR-721. Read AGENTS/CLAUDE, code-review Rust lane and rust-style references. No applicable AssuranceProfile found. No reviewer builds, tests, Cargo, global coverage census, advisors or final gate. Inspected supplied component harness: 16 tests pass and Clippy passes; this harness is diagnostic only. Actual production compile is BLOCKED by upstream codegen E0004 for three QSL CallSiteRefusal variants in /tmp/ir721-clock-compile2.log. Production validation is UNRUN; no merge-ready or full Program claim.

## Verdict

CONDITIONAL: one missing cancellation regression.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new clock-admission entry has no cancellation regression: the existing cancellation test invokes only the partial admit_monitor_selection API. Add an authentic-fixture pre-cancel control for admit_monitor_clock_selection that preserves Cancelled(Deadline), returns no capability and leaves the request/source row unchanged. | tests/monitor_preparation.rs:352-394,869-1043 |

## Coverage

Traced package-instance pointer identity, root/key/entry/subject/profile/target prefix; exact supplied clause/full role+ordinal native read; full source-map equality before token; all four optional clock fields; native unchanged payload and stable existing Copy outer code catalog; single success-work charge and projection resume; private borrow-only receipt. No copied native representation or clock fallback. Native error allocation/storage certification is excluded. Positive receipt/counterexample/success-work cells are Planned and unexecuted.

## Examined units

```yaml
scope:
- id: FR-015-AC-1
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. A fresh QSL plan-root package is strictly admitted
    and its independently selected root, clause, authentic occurrence, subject and
    target prepare one RT owner. Changing each selection independently, including
    another package instance with equal labels/indices, gives typed selection refusal
    before init and no owner.
- id: FR-015-AC-2
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. Layout retains the admitted root/four-entry
    content keys, lock, clock/order/progress associations, exact tick premises and
    capacities. Independent edits are either retained exactly after valid admission
    or refused at the owning selection boundary; labels, metadata PlanId, defaults
    and truncated keys never supply a match.
- id: FR-015-AC-3
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. A genuine generated closure containing shared
    expressions, lexical bindings, lazy branches, recursive calls, records, tuples,
    sequences/folds, union payloads and native comparisons lowers with original identities/order
    and one address per expression/function. Its actual admitted bodies execute; no
    handwritten temporal body substitutes.
- id: FR-015-AC-4
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. Generated source records, tuples, collections
    and unions reach the canonical TypeEnvironment with complete shapes, presence
    and original member/variant identities. Missing/foreign identity or projection
    gives the native typed refusal and no invented type or partial owner.
- id: FR-015-AC-5
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. Reachable equalities join same-acquisition
    producer witnesses against final admitted identities and adopt CheckedEquality
    without rechecking. Missing, conflicting, foreign and stale-after-rewrite rows
    refuse with exact native cause/locus; no seal/mint or fallback schedule path exists
    in the driver.
- id: FR-015-AC-6
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. A seventeen-binding plan retains the ordered
    State fields, buffers, exact event_position and admitted opaque activation-status
    type. Zero and above-host-width capacities retain their values without a local
    cap, bound interval table or status-schema restriction.
- id: FR-015-AC-7
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. A reachable admitted operation absent from
    RT gives typed unsupported lowering at that operation before init; an unrelated
    unsupported node outside the closure does not block the plan. Neither invokes
    a callback, evaluator, parser, compiler or replacement temporal algorithm.
- id: FR-015-AC-8
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. Work/storage exhaustion, cancellation, upstream
    nominal/equality refusal and RT ProgramRefusal preserve distinct typed causes/loci,
    return no partial program and leave inputs unchanged. A broken lowering invariant
    is a driver fault rather than input refusal; expected operating failure is not
    a fault. No preparation failure becomes a temporal verdict, StepResourceStop or
    ExecutorInvariantFailure.
- id: FR-015-AC-9
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. After producer/reader preparation is dropped,
    two builds of one prepared owner have independent fresh state. Actual admitted
    entries execute with explicit StepBudget; baseline event, watermark, poll and
    gap controls preserve RT's event-position/baseline rules and exact outcomes. Preparation
    itself invokes no entry or build.
- id: FR-015-AC-10
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. Real producer/read/lower/RT execution is compared
    with QSL's independent tick reference for RT FR-276's boundary, uncertainty, buffer21/22,
    wrap/gap, changed identity/order, activation and typed-stop controls. QSpec vectors
    are read by reference. Construction diagnostics and a single pilot plan receive
    no complete FR-276 execution credit.
- id: FR-015-AC-11
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. A large selected closure amid an unrelated
    long graph visits only relevant reachable nodes/edges and identity/value material
    under the caller allowance, without a whole-package sort or repeated full indexing.
    Equal budgets give equal typed work stops without a semantic depth/size ceiling.
- id: FR-015-AC-12
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: context_only
  excerpt: Planned until IR-721 lands. Public consumer/API and dependency controls
    establish one retained RT Program, no source input/callback, no driver seal/mint
    export or copied plan/expression/value family, and no IR/QSL/driver backedge from
    RT or RT edge from the cycle-free IR model.
- id: FR-015-AC-13
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: The separate clock-admission entry preserves native `WrongClause`, `WrongRole`,
    `UnknownOccurrence`, `MissingClock` and `WorkLimit` refusal payloads, including
    exact locus, occurrence, consumed work and attempted work. Semantic clock refusals
    map to outer `selection`; native work refusal maps to `work`. A missing clock
    returns no clock-admitted token even when the optional clock expectation is absent.
    Existing partial selection remains explicitly unclocked.
- id: FR-015-AC-14
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned pending authentic timed-profile clock emission and IR admission.
    A healthy clock-admission entry exposes the native `clock_for_claim` receipt for
    its independently selected final clause/Claim, charges successful receipt work
    exactly once to the remaining cumulative allowance, and compares every field of
    any optional independent clock row. The exposed inner selection retains that work
    prefix for later projection. This establishes no final PlanRoot subject-occurrence
    mapping, order/progress association or complete program.
- id: FR-015-clock-ordering
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: The separate clock entry SHALL first preserve all of that partial boundary's
    validations and then obtain the native clock receipt.
- id: TC-029-clock
  path: spec/test-cases/TC-029-admitted-monitor-program-preparation.md
  role: examined
  excerpt: FR-015-AC-13 covers only the native refusal controls above. The healthy
    receipt, optional expectation mismatch and successful cumulative work controls
    belong to FR-015-AC-14 and remain Planned without trace credit until the authentic
    timed clock producer handoff exists and those controls execute.
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Authentic fixture first proves MissingClock without cancellation; the new API is then called with pre-cancelled Deadline and asserts original typed cause, no capability, unchanged package/source map and independent inputs. |

### Fixed excerpt

```rust
let error = admit_monitor_clock_selection(&request, &cancel)
            .err()
            .expect("pre-cancellation returns no clock-admitted capability");
        assert_eq!(error.code(), MonitorPreparationErrorCode::Cancelled);
        assert!(matches!(
            error,
            MonitorPreparationError::Cancelled(CancelCause::Deadline)
        ));
```

## Round 1 disposition checkpoint

Rechecked round 1 fixed PR122 source against the original reviewed delta. Both SR-122001/FND-001 and SR-122004/FND-001 are fixed by inspected source/spec changes. No new findings in this method's changed scope. Original findings and original verdict above remain historical and unedited. Current static findings are clear; this is not production PASS or merge readiness. No reviewer builds, tests, Cargo, matrix, advisors, EARS engine or final gate. Supplied /tmp/ir721-clock-review-cancel-test.log reports the one new cancellation test passes only in the supplementary source harness. Production compilation was not rerun and remains BLOCKED on the codegen owner; authentic positive Timed clock emission/admission and FR015-AC14 remain Planned. Previous limitations and complete-program exclusions remain unchanged.
