---
id: SR-2026110705000822
title: Dependency analysis of FR-019 consumed QSL, IR and RT interfaces
type: SpecReview
analysis: dependency
scope: agent-ix/quire-driver; spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md; spec/test-cases/TC-033-prepare-monitor-under-one-budget.md; spec/functional/FR-015-prepare-an-admitted-monitor-program.md; spec/functional/FR-016-retain-monitor-refusal-loci.md; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md; spec/tests.md; PR quire-driver#129 head 3e3077679fe7019066785b98bf5472eec2c419dc; ticket IR-721
review_set: subset
date: '2026-11-07'
---
# SR-2026110705000822: Dependency analysis of FR-019 consumed QSL, IR and RT interfaces

## Summary

Ticket: IR-721. Reviewed quire-driver#129 at head 3e3077679fe7019066785b98bf5472eec2c419dc against RT origin/main (a483bf5), IR origin/main (e35b912), QSL origin/main (597381d), sibling PRs IR #945 (cc5034a) and RT #209 (d82a03d), open QSL PRs #2477 and #2489, and the driver WIP branch code/ir721-driver-closure (48ccd11). Review set: subset (base, integrity, dependency, failure-domain). No AssuranceProfile in this repository requires an enforced run. Lens: dependency.

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
| FND-001 | high | The authority-handoff row disagrees with QSL PR #2489 (open, IR-1383 plan authority). There the handoff is `PlanRoot::admit_authority(self, PlanAuthoritySelection{source, claim, observation_subject, observation_clock, observation_source}, Option<progress::View>, PlanAuthorityLimits{read, order, progress}) -> AdmittedPlanAuthority`. It consumes the PlanRoot by value, takes the observation owner's progress view, performs its own strict read and owns the admitted package. FR-019's request borrows `&PlanRoot` (FR-019:59), carries no progress view or observation selection (FR-019:55-75), and does its own strict read in P1. If both reads happen, FR-361 ForeignEmission refuses the healthy path, because the admission comes from a different package instance. QSL's order and progress limits are also outside the one budget, and FR-019 does not say so. Align the request and P1/P4 with the real handoff before coders start. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:90 | wrong-requirement |
| FND-002 | high | FR-019-AC-6 requires a reached map collection to lower with its key and value types, and IR #945's FR-357-AC-16 adds key_node for this. QSL PR #2477 (open, selected-row budget) has `SelectedSourceCollectionRow::to_ir` return `NativeRowError::unsupported_map` for any row with key_node or ValueType::Map. If #2477 merges as written, AC-6 cannot pass. #2477's adapters also charge with `ReachedRow(i)`, while IR #945's FR-358 adds `AdapterRow(i)` for exactly these adapters. Resolve this with the QSL lane, which owns the fix. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:164 | wrong-requirement |
| FND-003 | medium | The QSL capability rows do not match the real APIs. `identity_rows` returns `qsl_package::ReadIdentityRows` (declarations, fields, source_records, source_unions) with `IdentityFileRefusal`, not `SelectedExactIdentities`. On QSL main and on #2477 it clones records and unions without charging them, so 'charges every owned record, member and union allocation' has no QSL owner requirement: FR-251-AC-45..48 cover only the authority handoff. The adapters are unnamed. Their real names are `equality_witness(package, budget) -> ReadEqualityRows`, `selected_source_enum_rows(budget)`, `selected_ir_model_enum_shape_rows(budget)` and the per-row `SelectedSourceCollectionRow::to_ir(budget, locus)`, which needs a caller-supplied locus. Name the real signatures and cite the QSL requirement that owns the charging. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:86-89 | wrong-requirement |
| FND-004 | medium | P5 says 'with every row and the authority row' but gives no mapping from the QSL outputs into the 14 fields of IR SelectedMonitorClosureRows plus `authority`. The fields are declarations, fields, model_tuples, source_units, model_enum_declarations, model_enum_members, model_enum_shapes, source_records, source_tuples, source_unions, source_enums, source_collections, equality_applications and equality_facts. source_units is available both from the emission borrow and from ReadEqualityRows. The WIP branch code/ir721-driver-closure (48ccd11) already has to choose. Add the mapping table. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:120 | missing-requirement |
| FND-005 | low | FR-019 consumes `PlanRoot::emission()` and 'read-only accessors', but on QSL main PlanRoot has public fields (plan, obligations, emission). The accessors exist only in open QSL #2489. FR-019's Dependencies should name that this is consumed from FR-251's planned handoff. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:59 | missing-requirement |

## Verdict

NOT MERGEABLE. The IR seams (NativeJoinBudget resume/accounted_charge/first_unpaid on IR main e35b912; AuthorityRow and PreparationOutput loci, key_node and FR-361 in IR #945 cc5034a) and the RT seams (Layout, ProgramParts, Program::try_new on RT main; the budget boundary in RT #209 d82a03d) agree with FR-019. The consumed QSL interface does not match the QSL code that is in flight.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-006 | low | The request field `progress: Option<progress::View>` (also at FR-019:86) names no crate. Every other cross-owner type in the table is fully qualified, and the Dependencies list has no edge to the observation owner that issues the view. Qualify the path and add the dependency. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:65 | missing-requirement |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-007 | low | FR-019:89 and the P1 call at :139 give QSL's admit_authority as (self, selection, progress, &mut NativeJoinBudget, PlanAuthorityValidationCaps, NativeJoinLocus). QSL #2489 at head 01145fe declares (self, selection, progress, validation: PlanAuthorityValidationCaps, budget: &mut NativeJoinBudget, locus): the caps come before the budget. Swap the fourth and fifth arguments in both places. | spec/functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md:89 | wrong-requirement |

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
