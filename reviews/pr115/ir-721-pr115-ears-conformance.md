---
id: SR-20261104014360
title: "Requirement grammar and semantic trigger clarity"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-driver; PR 115; FR-015, TC-029, master index and AD-001; IR-721"
review_set: subset
---

## Summary

Independent scoped ears-conformance review of the planned admitted-monitor preparation interface. **PASS**, no findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

- examined: FR-015 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Given an independently selected, strictly admitted
monitor plan, the driver SHALL lower its complete ordinary executable closure
once into the runtime-owned immutable program. The driver SHALL preserve the
admitted selection, native identities, types, provenance and exact premises.
Preparation SHALL execute no initializer or transition.
- examined: FR-015-AC-1 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. A fresh QSL plan-root package is strictly admitted and its independently selected root, clause, authentic occurrence, subject and target prepare one RT owner. Changing each selection independently, including another package instance with equal labels/indices, gives typed selection refusal before init and no owner.
- examined: FR-015-AC-2 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Layout retains the admitted root/four-entry content keys, lock, clock/order/progress associations, exact tick premises and capacities. Independent edits are either retained exactly after valid admission or refused at the owning selection boundary; labels, metadata PlanId, defaults and truncated keys never supply a match.
- examined: FR-015-AC-3 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. A genuine generated closure containing shared expressions, lexical bindings, lazy branches, recursive calls, records, tuples, sequences/folds, union payloads and native comparisons lowers with original identities/order and one address per expression/function. Its actual admitted bodies execute; no handwritten temporal body substitutes.
- examined: FR-015-AC-4 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Generated source records, tuples, collections and unions reach the canonical TypeEnvironment with complete shapes, presence and original member/variant identities. Missing/foreign identity or projection gives the native typed refusal and no invented type or partial owner.
- examined: FR-015-AC-5 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Reachable equalities join same-acquisition producer witnesses against final admitted identities and adopt CheckedEquality without rechecking. Missing, conflicting, foreign and stale-after-rewrite rows refuse with exact native cause/locus; no seal/mint or fallback schedule path exists in the driver.
- examined: FR-015-AC-6 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. A seventeen-binding plan retains the ordered State fields, buffers, exact event_position and admitted opaque activation-status type. Zero and above-host-width capacities retain their values without a local cap, bound interval table or status-schema restriction.
- examined: FR-015-AC-7 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. A reachable admitted operation absent from RT gives typed unsupported lowering at that operation before init; an unrelated unsupported node outside the closure does not block the plan. Neither invokes a callback, evaluator, parser, compiler or replacement temporal algorithm.
- examined: FR-015-AC-8 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Work/storage exhaustion, cancellation, upstream nominal/equality refusal and RT ProgramRefusal preserve distinct typed causes/loci, return no partial program and leave inputs unchanged. A broken lowering invariant is a driver fault rather than input refusal; expected operating failure is not a fault. No preparation failure becomes a temporal verdict, StepResourceStop or ExecutorInvariantFailure.
- examined: FR-015-AC-9 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. After producer/reader preparation is dropped, two builds of one prepared owner have independent fresh state. Actual admitted entries execute with explicit StepBudget; baseline event, watermark, poll and gap controls preserve RT's event-position/baseline rules and exact outcomes. Preparation itself invokes no entry or build.
- examined: FR-015-AC-10 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Real producer/read/lower/RT execution is compared with QSL's independent tick reference for RT FR-276's boundary, uncertainty, buffer21/22, wrap/gap, changed identity/order, activation and typed-stop controls. QSpec vectors are read by reference. Construction diagnostics and a single pilot plan receive no complete FR-276 execution credit.
- examined: FR-015-AC-11 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. A large selected closure amid an unrelated long graph visits only relevant reachable nodes/edges and identity/value material under the caller allowance, without a whole-package sort or repeated full indexing. Equal budgets give equal typed work stops without a semantic depth/size ceiling.
- examined: FR-015-AC-12 (spec/functional/FR-015-prepare-an-admitted-monitor-program.md): Planned until IR-721 lands. Public consumer/API and dependency controls establish one retained RT Program, no source input/callback, no driver seal/mint export or copied plan/expression/value family, and no IR/QSL/driver backedge from RT or RT edge from the cycle-free IR model.
- examined: TC-029 (spec/test-cases/TC-029-admitted-monitor-program-preparation.md): Planned until IR-721 lands. Verify the driver boundary from a genuinely
QSL-produced, strictly IR-admitted monitor root to the one RT-owned program.
This case does not substitute the existing hand-built construction arenas for
the emitted entries or treat the absence of a witness as a passing negative
compiler control.
- examined: AD-001 (spec/assurance/AD-001-driver-architecture.md): - Planned until IR-721 lands, FR-015 adds a driver-to-RT production edge. RT
  remains a leaf without IR, QSL or driver dependencies, and the cycle-free IR
  model remains independent of RT. The returned RT Program owns the canonical
  environment and expression/function arenas after driver preparation drops.
- examined: Master Requirements Specification (spec/spec.md): Planned until IR-721 lands: FR-015 adds the library handoff from an
independently selected admitted monitor root to RT's retained executable owner.
It does not add monitor syntax, a temporal evaluator, another CLI operation or
a second runtime representation. Producer qualification and runtime execution
remain separate obligations.

## Analysis

The deterministic scoped grammar check reports 84/84 documents grammar-clean and zero grammar findings. All twelve new criteria are concrete examples or invariants; the FR Description states the prerequisite selection and the driver's lowering response. The continuous lifetime and dependency constraints are expressed as invariant obligations. Unsupported, witness, cancellation and operating-stop conditions have explicit failure responses. No contradictory event/state trigger or vague performance adjective was found. Statements prohibit extra initialization, semantic reconstruction and partial success in measurable terms. The semantic review agrees with the engine for the changed requirement; no advisory warning is hidden.

## Verification and Limits

The four changed specification artifacts pass scoped Quire validation; the repository grammar summary reports zero findings. Evidence advice for the twelve new criteria is retained externally. Tooling: Quire CLI 0.38.1 / engine 0.51.0 and Quoin 0.29.1. No applicable AssuranceProfile is declared under this repository's spec tree; AD-001 is an architecture document, not such a profile. No enforced review workflow is required by a discovered policy. No Rust, Cargo, backend proof, runtime execution, source edit, merge or optional repository-wide criterion-strength analysis was performed. All twelve new obligations remain planned until IR-721 lands.

Plan completion: not assessed
