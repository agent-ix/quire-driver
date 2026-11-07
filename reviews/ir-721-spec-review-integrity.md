---
id: SR-2026110705000821
title: Integrity analysis of FR-019 Layout derivation, phases and classification
type: SpecReview
analysis: integrity
scope: agent-ix/quire-driver; spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md; spec/test-cases/TC-033-prepare-monitor-under-one-budget.md; spec/functional/FR-015-prepare-an-admitted-monitor-program.md; spec/functional/FR-016-retain-monitor-refusal-loci.md; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md; spec/tests.md; PR quire-driver#129 head 3e3077679fe7019066785b98bf5472eec2c419dc; ticket IR-721
review_set: subset
date: '2026-11-07'
---
# SR-2026110705000821: Integrity analysis of FR-019 Layout derivation, phases and classification

## Summary

Ticket: IR-721. Reviewed quire-driver#129 at head 3e3077679fe7019066785b98bf5472eec2c419dc against RT origin/main (a483bf5), IR origin/main (e35b912), QSL origin/main (597381d), sibling PRs IR #945 (cc5034a) and RT #209 (d82a03d), open QSL PRs #2477 and #2489, and the driver WIP branch code/ir721-driver-closure (48ccd11). Review set: subset (base, integrity, dependency, failure-domain). No AssuranceProfile in this repository requires an enforced run. Lens: integrity.

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
| FND-001 | high | `step_result` has no derivable source. RT Layout.step_result is a TypeBinding{key: NodeKey, value_type} for 'the exact two-position (State, Results) entry result type'. IR CheckedMonitorPlanRef (monitor_plan.rs:562-574) exposes only state(), inputs(), scope() and results(). The type node is reachable only as an entry's result_type() (e.g. on_event().result_type()). FR-019:142 says 'the plan's admitted type nodes' and names no accessor or agreement check across the three step entries. Name the source and the refusal when the entries disagree. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:142 | missing-requirement |
| FND-002 | high | `time_dimension` is listed among 'the admitted plan premises, after the FR-015 comparison with the independent target', but IR CheckedMonitorPlanRef has no time-dimension accessor, the request (FR-019:69) carries no time_dimension, and FR-015's comparison covers counter/unit/width/gap/rate/uncertainty/capacities only. RT documents it as an 'Independently resolved base Time dimension key'. Specify where the key comes from (e.g. the tick unit's admitted dimension in the closure) and how it is independently checked. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:144 | missing-requirement |
| FND-003 | medium | The premise and type rows require conversions that the spec does not state: CheckedNodeId to NodeKey (root, clause, tick_unit, every TypeBinding.key, Buffer.owner), CheckedRational to Rational, the canonical decimal &str activation_capacity and buffer capacities to quire_exact::Integer, and CheckedMonitorBufferRef to Buffer{owner, capacity}. FR-019:147 forbids re-encoding, yet decoding decimal text is required. No phase charge or error classification is given for a conversion failure such as InvalidNodeKey. State each conversion, its P6 charge and its error category. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:144-147 | missing-requirement |
| FND-004 | medium | 'A limit one unit below that phase's first charge' can be read as an absolute limit, which refuses in P1 for every phase, or as consumption before the phase plus that phase's first charge minus one, which is the intended form. 'returns work or storage' also lets a phase be tested on one kind only. State the cumulative form and require both a work and a storage one-under control for each phase that charges both. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:160 | wrong-requirement |
| FND-005 | medium | The MonitorPreparationErrorCode mapping lists selection, work, storage, projection, program and cancelled only. It drops FR-015's distinct unsupported-operation and driver internal-fault categories, P1's FR-015 selection refusals, the P4 SelectedMonitorAuthorityRowError, and Layout conversion failures. The mapping should either be complete or defer explicitly to FR-015's categories for the unlisted causes. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:101-105 | missing-requirement |
| FND-006 | low | 'Each byte and text value SHALL be copied exactly once, under P6' conflicts with P4 (FR-019:119), which copies the order and association bytes into SelectedMonitorAuthorityRow, and with IR FR-361's receipt copy. Say 'copied into Layout once, under P6'. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:146 | wrong-requirement |

## Verdict

NOT MERGEABLE. I checked the Layout table field by field against RT origin/main `monitor::Layout` (src/monitor/program.rs:59-117) and against IR FR-361's layout context (#945). root, clause, package_lock, subject, occurrence, profile, clock_binding, admitted_order, progress_context, entries, counter, counter_width_bits, max_reading_gap, max_event_rate and uncertainty map to a real source. Two fields have no derivable source, and several required representation conversions are unstated.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-007 | medium | time_dimension's source is now 'the admitted dimension of the tick unit in the closure's unit facts', but no closure accessor is named. The IR closure is reached-only (FR-357-AC-13), and it is seeded from State, Inputs, Scope, Results and the entries (FR-357-AC-1). plan.tick_unit() is a premise, not a reached type, so its unit and dimension fact may be absent from the closure. FR-019 gives no source and no refusal for that case. Name the accessor (e.g. the SelectedTypeDeclarations unit row for tick_unit, or IR's identity dimension read). Either require IR to seed tick_unit, or classify its absence (projection or target_mismatch) and cover it in AC-9. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:191 | missing-requirement |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-008 | medium | FR-019-AC-9 is verified by Test and requires 'an injected absent tick_unit_dimension() and injected disagreeing entry result types each return internal', but nothing can inject either one. SelectedMonitorTypeClosure has no public constructor, and IR guarantees both values for an admitted plan; IR FR-361-AC-5 and strict admission fix one tuple result type. No genuine or fixture plan can produce these states without a driver test seam, and none is specified. TC-033 step 8 (lines 42-44 and 64-66) omits the absent-dimension control. Either specify the seam, for example a private fault-injection hook in the driver's Layout assembly, or verify these two internal mappings by Inspection and keep the Test clauses for target_mismatch and InvalidNodeKey. Then align TC-033 step 8. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:219 | wrong-requirement |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-009 | medium | FR-019-AC-10 requires source inspection to show that 'an absent tick_unit_dimension()' maps to internal. FR-019:126 and :194 say the same thing ('An absent value is internal'). IR #945 at a2239e2 specifies `SelectedMonitorTypeClosure::tick_unit_dimension() -> &CheckedNodeId` (FR-357-AC-18), which is a non-Option return, so the value cannot be absent. No driver source can contain that mapping, and the clause either passes vacuously or cannot pass. Delete the absent-dimension clause from AC-10, from the classification row at :126 and from the Layout row at :194, and from TC-033 step 9 (lines 44-45). Keep the disagreeing-result-types internal mapping. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:221 | wrong-requirement |

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
