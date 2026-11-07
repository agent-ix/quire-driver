---
id: SR-2026110705000820
title: Base spec review of quire-driver#129 tick-monitor driver spec
type: SpecReview
analysis: base
scope: agent-ix/quire-driver; spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md; spec/test-cases/TC-033-prepare-monitor-under-one-budget.md; spec/functional/FR-015-prepare-an-admitted-monitor-program.md; spec/functional/FR-016-retain-monitor-refusal-loci.md; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md; spec/tests.md; PR quire-driver#129 head 3e3077679fe7019066785b98bf5472eec2c419dc; ticket IR-721
review_set: subset
date: '2026-11-07'
---
# SR-2026110705000820: Base spec review of quire-driver#129 tick-monitor driver spec

## Summary

Ticket: IR-721. Reviewed quire-driver#129 at head 3e3077679fe7019066785b98bf5472eec2c419dc against RT origin/main (a483bf5), IR origin/main (e35b912), QSL origin/main (597381d), sibling PRs IR #945 (cc5034a) and RT #209 (d82a03d), open QSL PRs #2477 and #2489, and the driver WIP branch code/ir721-driver-closure (48ccd11). Review set: subset (base, integrity, dependency, failure-domain). No AssuranceProfile in this repository requires an enforced run. Lens: base.

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
| FND-001 | high | FR-015 still says MonitorPreparationRequest groups 'finite operating work/storage allowances', which contradicts FR-019:71 ('The request SHALL NOT carry operating limits') and the separate &mut NativeJoinBudget parameter. A coder following FR-015 adds limits to the request; one following FR-019 must not. Remove the allowance clause from FR-015:234-237. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:234-237 | wrong-requirement |
| FND-002 | high | FR-016-AC-10 still requires 'peak logical storage permit exact-limit success', and FR-016:205-208 still says the receipt 'SHALL exclude released scratch' and 'the peak live ledger SHALL never exceed the allowance'. The new gloss at FR-016:142-145 redefines 'release/peak' as the cumulative counter instead of rewriting these lines. Under a budget that never releases, the exact limit is cumulative storage including scratch. So a budget equal to the measured peak, which AC-10 says must succeed, now refuses, and CopiedMonitorLocus.storage ('retained storage', FR-016:160-161) has no defined value. The AC's meaning changed without its text changing. Rewrite AC-10 and lines 160-161 and 205-208 for cumulative accounting, with exact-cumulative admit and one-under refuse. | spec/functional/FR-016-retain-monitor-refusal-loci.md:283 | wrong-requirement |
| FND-003 | medium | FR-017-AC-5 still says lower_scalar_functions returns its outputs 'under its independent allowance', and FR-017:243-244 still says 'Its independent receipt'. FR-017:87 now says the helper charges the caller's NativeJoinBudget from its current consumption. A test written to AC-5 starts a fresh allowance, which FR-019-AC-8 forbids. Reword AC-5 to the caller's budget, without narrowing its other clauses. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:279 | wrong-requirement |
| FND-004 | medium | FR-017's full lowering entry is still `lower_monitor(selection: AdmittedMonitorSelection<'_>, cancel)` with no budget parameter, and FR-017:40-43 obtains 'operating limits through the selection'. The lead's decision is that the budget is passed as a separate &mut, and FR-019-AC-8 lists only prepare_monitor, lower_scalar_functions and copy_monitor_locus. Either lower_monitor gets a `budget: &mut NativeJoinBudget` parameter and is added to AC-8, or the spec states how the selection borrows it. As written the selection cannot own a &mut taken by prepare_monitor. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:33-37 | missing-requirement |
| FND-005 | medium | FR-015:201-202 still requires 'Traversal scratch SHALL share the existing storage ledger and be released on work or cancellation failure'. This contradicts FR-019:123-124 (storage is never released or refunded) and FR-019-AC-8 ('no ... storage release'). Restate it as a charge to the one budget with no refund. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:199-202 | wrong-requirement |
| FND-006 | medium | FR-015-AC-8 requires that operating stops 'leave inputs unchanged'. The budget is now a &mut input that keeps admitted consumption after a refusal (FR-019-AC-5: 'the consumption admitted so far'). Read literally, AC-8 fails on every refusal. Exempt the budget's cumulative consumption explicitly, so that AC-8 is not narrowed. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:411 | wrong-requirement |
| FND-007 | medium | FR-015's Dependencies still say 'the concrete preparation request field list remains unfrozen until those inputs exist', while FR-019:55 now fixes 'exactly these public fields'. Delete the stale sentence. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:440-443 | wrong-requirement |
| FND-008 | low | spec/tests.md:14 now claims to cover 'FR-001 through FR-019 and TC-001 through TC-033', but the inventory has no rows for FR-016, FR-017, FR-018 or TC-030 to TC-032. Narrow the sentence or add the rows. | spec/tests.md:14 | wrong-requirement |

## Verdict

NOT MERGEABLE. The new FR-019 and TC-033 are structurally valid (quire validate passes; only EARS warnings) and introduce no pins, digests, compatibility path, vendoring or ticket citations. No existing FR-015/016/017 acceptance-criterion row was deleted or reworded, but the rework leaves normative text and two AC rows that contradict the one-budget, no-release model, and one AC (FR-016-AC-10) silently changes meaning.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-009 | low | FR-016:182-183 still says 'The same schedule applies to borrowed descendants reached for storage measurement', and lines 189 and 192 still say 'storage-ledger'. The rewritten line 198 says that sharing an Arc edge 'SHALL NOT copy or measure its subtree'. The leftover sentence implies work charges for a subtree walk that no longer happens. Delete it and say 'budget' instead of 'storage ledger'. | spec/functional/FR-016-retain-monitor-refusal-loci.md:181-183 | wrong-requirement |

## Dispositions

Round 1, verified against fix commit e12108e8b1d802ade55e28056f8fc3db350e66df.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |
| FND-007 | fixed | - |
| FND-008 | fixed | - |
| FND-009 | fixed | - |
