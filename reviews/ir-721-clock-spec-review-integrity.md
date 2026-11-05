---
id: SR-122004
title: IR-721 clock admission spec-review-integrity review
type: SpecReview
analysis: integrity
scope: agent-ix/quire-driver; PR122 nine-file clock-admission delta
review_set: subset
---

## Summary

Independent static review of original frozen PR122 source. Ticket: IR-721. Read AGENTS/CLAUDE, code-review Rust lane and rust-style references. No applicable AssuranceProfile found. No reviewer builds, tests, Cargo, global coverage census, advisors or final gate. Inspected supplied component harness: 16 tests pass and Clippy passes; this harness is diagnostic only. Actual production compile is BLOCKED by upstream codegen E0004 for three QSL CallSiteRefusal variants in /tmp/ir721-clock-compile2.log. Production validation is UNRUN; no merge-ready or full Program claim.

## Verdict

CONDITIONAL: contradictory ordering obligation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The required partial-first ordering contradicts native first-cause preservation: partial selection rejects wrong clause/role/ordinal before clock_for_claim, whereas the clock API intentionally returns native WrongClause/WrongRole/UnknownOccurrence. Specify the shared validation prefix, native read, then source-map and optional-expectation validation before successful capability construction. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:102-105 |

## Coverage

Read native wrong-clause/role/ordinal checks against partial guards. Capability success still checks full source-map authenticity; the finding is contradictory normative refusal ordering, not forged success. AC13 and AC14 are independently scoped and externally observable.

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
| FND-001 | fixed | The rewritten obligation names the shared validation prefix, native semantic first causes, and source-map/optional expectation checks before successful capability construction; it permits inspection before the read without replacing native semantic refusal. |

### Fixed excerpt

```rust
The separate clock entry SHALL first validate the
shared package/root/key, subject, profile and target prefix, then invoke the
native clock reader with the independent clause and full role/ordinal.
Native `WrongClause`, `WrongRole`, `UnknownOccurrence` and `MissingClock`
refusals SHALL retain their first causes. Before constructing a successful
clock-admitted capability, the driver SHALL validate full source-map
authenticity and any optional independent clock expectation.
```

## Round 1 disposition checkpoint

Rechecked round 1 fixed PR122 source against the original reviewed delta. Both SR-122001/FND-001 and SR-122004/FND-001 are fixed by inspected source/spec changes. No new findings in this method's changed scope. Original findings and original verdict above remain historical and unedited. Current static findings are clear; this is not production PASS or merge readiness. No reviewer builds, tests, Cargo, matrix, advisors, EARS engine or final gate. Supplied /tmp/ir721-clock-review-cancel-test.log reports the one new cancellation test passes only in the supplementary source harness. Production compilation was not rerun and remains BLOCKED on the codegen owner; authentic positive Timed clock emission/admission and FR015-AC14 remain Planned. Previous limitations and complete-program exclusions remain unchanged.

## Round 2 production evidence and dependency checkpoint

Same reviewer continuation for PR122 after normal main dependency adoption and production component execution. The original review and round 1 UNRUN/BLOCKED observations above describe their historical checkpoints; the following facts supersede that evidence/dependency status only. Both original findings remain fixed. No new findings in this method's changed scope. Current scoped review: CLEAR for the partial clock refusal component, pending the lead's one final scoped gate after artifact custody. This review is not that gate and does not qualify healthy clock admission or a complete program.

Inspected the complete delta since the fixed static source: no driver production code, tests, Cargo.toml, Makefile or CI behavior changed. Cargo.lock adopts normal branch=main CG repair, coherent QSL packages from one main cohort, and coherent IR/model main packages. The installed CG source explicitly maps the three new imported call-site refusals through their original setup refusal codes; CG PR749 is merged. No pin, dependency checkout patch, synthetic clock row, relaxed assertion or gate change was introduced.

Actual production quire-driver receipts inspected: /tmp/ir721-clock-production-focused-tests.log reports 17 monitor_preparation and 8 monitor_lowering tests passed with zero failed/ignored; /tmp/ir721-clock-production-monitor-units.log reports 9 selected library units passed, 82 filtered out; /tmp/ir721-clock-production-clippy.log completes successfully for the production crate and selected tests; supplied warning-denied invocation is reported by the lead. /tmp/ir721-clock-production-fmt.log and /tmp/ir721-clock-production-spec.log contain no failure; stable rustfmt configuration and installed module warnings remain visible. This is production component evidence, no longer only the supplementary harness. No reviewer builds, tests, Cargo, matrix, advisors, EARS engine or final gate were run.

Only FR015-AC13 refusal controls are qualified: native wrong clause/role/ordinal and MissingClock retain complete payloads; same-clause twins retain requested ordinals; zero/one-under/exact native work allowance retains first stops; pre-cancelled Deadline returns its original cause, no capability and unchanged inputs. FR015-AC14 healthy native receipt, optional independent clock comparison and successful cumulative work remain Planned. QSL1728 final PlanRoot subject mapping, admitted order/progress associations and full Program preparation remain Planned. QSL PR2385 and IR PR910 are merged and supply source-record/member rows and native record projection; driver joins and complete generated record closure are separate unqualified work. Native union seam is delivered; authentic final source-union producer handoff remains separate. The current status prose accurately separates these available producer/IR facts from driver qualification.

### Current examined units

```yaml
scope:
- id: FR-015-AC-13
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Pre-cancellation returns the original cancellation cause, no clock-admitted
    token and unchanged inputs.
- id: FR-015-AC-14
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: Planned pending authentic timed-profile clock emission and IR admission.
- id: TC-029-clock-production
  path: spec/test-cases/TC-029-admitted-monitor-program-preparation.md
  role: examined
  excerpt: 'After adopting Contract CG PR #749, the production driver runs all 17
    `monitor_preparation` tests and all 8 `monitor_lowering` tests successfully.'
- id: clock-status
  path: spec/tests.md
  role: examined
  excerpt: This is component evidence, not a final gate or a complete preparation
    result.
```
