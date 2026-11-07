---
id: SR-2026110705000823
title: Failure-domain analysis of FR-019 identity, budget and refusal paths
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-driver; spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md; spec/test-cases/TC-033-prepare-monitor-under-one-budget.md; spec/functional/FR-015-prepare-an-admitted-monitor-program.md; spec/functional/FR-016-retain-monitor-refusal-loci.md; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md; spec/tests.md; PR quire-driver#129 head 3e3077679fe7019066785b98bf5472eec2c419dc; ticket IR-721
review_set: subset
date: '2026-11-07'
---
# SR-2026110705000823: Failure-domain analysis of FR-019 identity, budget and refusal paths

## Summary

Ticket: IR-721. Reviewed quire-driver#129 at head 3e3077679fe7019066785b98bf5472eec2c419dc against RT origin/main (a483bf5), IR origin/main (e35b912), QSL origin/main (597381d), sibling PRs IR #945 (cc5034a) and RT #209 (d82a03d), open QSL PRs #2477 and #2489, and the driver WIP branch code/ir721-driver-closure (48ccd11). Review set: subset (base, integrity, dependency, failure-domain). No AssuranceProfile in this repository requires an enforced run. Lens: failure-domain.

## Scope

- examined: FR-019 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — When a caller prepares one QSL tick-monitor plan for the runtime, the driver
- examined: FR-019:request (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — `MonitorPreparationRequest` SHALL have exactly these public fields:
- examined: FR-019:consumed-qsl-interface (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — The driver SHALL consume exactly these QSL capabilities. Each returns
- examined: FR-019:outputs (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — On success the driver SHALL return `Program::try_new(parts)`'s owner. The
- examined: FR-019:phases (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — The driver SHALL run these phases in order, calling `budget.resume()` at the
- examined: FR-019:layout (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | `Layout` field | Value |
- examined: FR-019:refusal-order (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — The first failing phase SHALL determine the refusal. Within a phase, the
- examined: FR-019-AC-1 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-1 | From a genuine QSL plan root for the 20-bit 1 MHz deadline claim, `prepare_monitor` returns a Program whose `Layout` fields equal, byte for byte, the IR layout context, the receipt byt
- examined: FR-019-AC-2 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-2 | One budget measured over a successful preparation succeeds at its exact work and storage limits. For each of P1 through P6, a limit one unit below that phase's first charge returns `wo
- examined: FR-019-AC-3 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-3 | A QSL adapter or `identity_rows` denial and a QSL authority handoff refusal (absent order premise, absent association, second emission) are returned unchanged inside `MonitorPreparatio
- examined: FR-019-AC-4 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-4 | Each IR authority refusal code reaches the caller unchanged, classified `selection`; a handoff from a second emission of the same source returns `ForeignEmission`. | Test |
- examined: FR-019-AC-5 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-5 | With defects placed in two phases at once, preparation returns the earlier phase's refusal; cancellation set before each phase returns `cancelled`, no program and the consumption admit
- examined: FR-019-AC-6 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-6 | A reached map collection lowers into the canonical `TypeEnvironment` with its key and value types; changing only QSL's `PlanId` metadata leaves `Layout` unchanged, while changing an en
- examined: FR-019-AC-7 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-7 | The budget's consumption is unchanged by `Program::try_new`; an RT construction refusal is returned as `program` with its original `ProgramRefusal`. | Test |
- examined: FR-019-AC-8 (spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) — | FR-019-AC-8 | Source and API inspection finds one `&mut NativeJoinBudget` parameter on `prepare_monitor`, `lower_scalar_functions` and `copy_monitor_locus`; no `MonitorPreparationLimits`, private le
- examined: TC-033 (spec/test-cases/TC-033-prepare-monitor-under-one-budget.md) — 1. Produce a plan root for the 20-bit 1 MHz deadline claim with a genuine
- examined: FR-015:inputs-budget (spec/functional/FR-015-prepare-an-admitted-monitor-program.md) — - One caller-owned finite `NativeJoinBudget` and caller cancellation under
- examined: FR-015:public-entry (spec/functional/FR-015-prepare-an-admitted-monitor-program.md) — The public entry's request/result shape SHALL be:
- examined: FR-015:final-subject-scratch (spec/functional/FR-015-prepare-an-admitted-monitor-program.md) — Before direct producer comparisons or origin conversion, the driver SHALL
- examined: FR-015:lifetime-budget (spec/functional/FR-015-prepare-an-admitted-monitor-program.md) — The bounded locus-copy component of
- examined: FR-015-AC-8 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md) — | FR-015-AC-8 | Planned until IR-721 lands. Work/storage exhaustion, cancellation, upstream nominal/equality refusal and RT ProgramRefusal preserve distinct typed causes/loci, return no partial progra
- examined: FR-015:dependencies-qsl (spec/functional/FR-015-prepare-an-admitted-monitor-program.md) — - [QSL FR-251](ix://agent-ix/quire-spec-language/FR-251) owns derivation and
- examined: FR-016:bounded-copy-budget (spec/functional/FR-016-retain-monitor-refusal-loci.md) — The component charges the caller's one preparation `NativeJoinBudget` under
- examined: FR-016:storage-ledger (spec/functional/FR-016-retain-monitor-refusal-loci.md) — Before allocating new vector or text capacity, the component SHALL reserve
- examined: FR-016:copy-error (spec/functional/FR-016-retain-monitor-refusal-loci.md) — `MonitorLocusCopyError` SHALL distinguish `Work { limit, consumed, next }`,
- examined: FR-016-AC-10 (spec/functional/FR-016-retain-monitor-refusal-loci.md) — | FR-016-AC-10 | Independently measured work and peak logical storage permit exact-limit success; one-under work/storage and pre-cancellation return their distinct original typed operating causes with
- examined: FR-016-AC-11 (spec/functional/FR-016-retain-monitor-refusal-loci.md) — | FR-016-AC-11 | Every allocating copy/scratch reservation is fallible and precharged; reservation failure preserves TryReserveError, excess text capacity returns Capacity before conversion, and clean
- examined: FR-017:lower-monitor-signature (spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md) — ```rust
- examined: FR-017:helper-budget (spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md) — The public helper SHALL charge the caller's `NativeJoinBudget` from its current consumption, under FR-019's charging rules.
- examined: FR-017:continue-allowance (spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md) — ### Continue one operating allowance
- examined: FR-017-AC-3 (spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md) — | FR-017-AC-3 | Full lowering preserves admission's already consumed work under the same total limit, native read success work, charged storage and original cancellation. An exact/one-under operating 
- examined: FR-017-AC-5 (spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md) — | FR-017-AC-5 | The public lower_scalar_functions helper accepts the closed initial scalar set and returns owned native RT root addresses/functions/expressions under its independent allowance. Request
- examined: spec/tests.md:scope (spec/tests.md) — This authored verification inventory integrates FR-001 and SP-D1 through

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | P4 uses 'the package's `admission_binding()`' and P5 calls the closure read without saying on which instance. It could be request.package (the independently selected CheckedPackageV2) or the instance strictly read from plan_root's emission in P1. FR-015:155-157 compares the two only by complete package equality, so they can be distinct instances. FR-361 ForeignEmission refuses 'a different package instance'. Using request.package's binding with a plan from it never ties the authority to QSL's emission, and using the strict-read instance bypasses the independent selection. State which instance supplies the plan ref and the AdmissionBinding, and how request.package relates to it. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:119-120 | missing-requirement |
| FND-002 | medium | copy_monitor_locus now charges the shared budget, but MonitorLocusCopyError still carries `Work { limit, consumed, next }` / `Storage { limit, consumed, next }` from the removed private ledger, and no NativeJoinLocusSource is given for its charges. FR-019-AC-2 requires the budget's first unpaid kind, amount and phase locus. Specify the error's relation to `first_unpaid()` and `first_unpaid_locus()` (e.g. PreparationOutput(k) with form 'monitor_locus') so that a denial inside the diagnostic path reports the same data. | spec/functional/FR-016-retain-monitor-refusal-loci.md:225-228 | missing-requirement |

## Verdict

NOT MERGEABLE. The phase refusal order, the cancellation polling between phases (it matches IR AD-010), cumulative no-refund consumption, the end of the budget at Program::try_new (it matches RT #209 FR-276-AC-13) and exact-limit admit / one-under refuse in AC-2 are sound. Two identity and denial paths are unspecified.
