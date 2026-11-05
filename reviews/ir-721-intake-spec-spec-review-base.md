---
id: SR-117001
title: IR-721 driver PR117 base review
type: SpecReview
analysis: base
scope: agent-ix/quire-driver; PR117; spec/functional/FR-015-prepare-an-admitted-monitor-program.md;
  spec/test-cases/TC-029-admitted-monitor-program-preparation.md
review_set: subset
---

## Summary

Ticket IR-721; driver PR117, head branch titan/ir721-intake-spec. Independent review of the frozen FR-015/TC-029 diff only. No applicable AssuranceProfile exists under spec/. Source authority was checked against fetched producer/consumer main as documented in the final interface measurement and by targeted source reads; those reads are not executable acceptance. No repository edits, Cargo commands, global scans or advisor runs occurred. Root owns artifact custody and the final docs-only scoped gate.

## Verdict

PASS within the changed specification scope; zero defects. Full preparation remains Planned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Applied the selected base checklist to the new public entry-shape subsection, typed failure categories, corrected dependency prose and TC-029 procedure/expected-result additions. IDs and owning relationships remain unchanged; relative links resolve to the existing FR/TC files. The outer request/result signature freezes no unavailable concrete request fields or implemented Rust type. It states precisely the authentic input prerequisites rather than inventing an association wrapper. The existing no-partial-program, original identity, lifetime, no-source-reparse/recheck and no-callback contracts remain intact. Collections retain structural element/kind/cardinality without a declaration identity; the revised prose does not remove their type information. No added hash, digest, pin, tracking ledger or copied fixture was found. Existing semantic provenance and original identities are domain meaning, not file tracking. Full matrix/tag coverage was not recomputed under the dispatch restriction; all preparation ACs remain Planned.

No production/code or Rust changes exist, so language review and production gap analysis are inapplicable per the dispatch. Requirement grammar is already imperative SHALL; separate EARS/object/security lenses add no changed domain content. This is scoped spec acceptance only, not a claim that prepare_monitor exists.

## Examined units

```yaml
scope:
- id: FR-015-producer-prerequisites
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: 'The final producer has been re-measured after QSL-1630 and QSL-1631. Those

    completed changes provide body-sensitive plan identity and source enum rows;

    they do not supply the missing source record/member and union identity

    handoff needed by the generated program. The existing IR-1323 union seam and

    that authentic producer handoff remain prerequisites for healthy complete

    preparation. This specification does not report an implemented entry or

    qualify a refusal-only implementation as a prepared program.'
- id: FR-015-entry-shape
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: 'The public entry''s request/result shape SHALL be:'
- id: FR-015-request-borrows
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: '`MonitorPreparationRequest` SHALL group borrows of the selected admitted

    package/root, the independent canonical selection and original admitted

    associations, same-final-emission witness rows, and finite operating

    work/storage allowances. It SHALL NOT accept `ProgramParts`, a caller-built

    `TypeEnvironment`, replacement bodies or a callback supplying semantics.

    The package SHALL resolve its selected root and four entries itself; a foreign

    borrowed view SHALL NOT replace that resolution.'
- id: FR-015-field-freeze
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: 'This freezes the entry/result shape and failure distinctions, not a concrete

    request field list or an implemented Rust type. The request''s concrete public

    fields SHALL be frozen only after the owning interfaces supply authentic

    final source record/member and union identities and admitted clock/order/

    progress associations. `PlanRoot` currently contains no final `AdmittedContext`;

    its selected model rows cannot establish generated source record authority.

    Raw caller bytes or a new local association wrapper SHALL NOT be treated as

    proof that an order/progress association was independently admitt'
- id: FR-015-failure-categories
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: '`MonitorPreparationError` SHALL expose distinct typed categories for selection,

    unsupported operation, upstream projection, work, storage, cancellation and

    runtime program construction. A broken lowering invariant SHALL retain the

    distinct driver internal-fault category. Native node/body-path or occurrence

    loci and upstream causes SHALL remain available without parsing human-readable

    error text. These are failure categories, not temporal outcomes or a complete

    new catalog of upstream causes.'
- id: FR-015-canonical-closure
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: 'The driver SHALL materialize the needed admitted native type/declaration

    closure in canonical QSV `TypeEnvironment`, using the original Exact shared

    values/types and identities. Source-owned records, tuples and unions SHALL

    retain declaration order, closed/open shape, presence and payload types. Collections
    SHALL retain their

    structural Exact element, kind and cardinality types without inventing a

    collection declaration identity. The driver SHALL NOT treat a selected-domain

    projection as a complete projection of generated source types, or invent an identity
    when the required

    projection/witness'
- id: FR-015-subset-boundary
  path: spec/functional/FR-015-prepare-an-admitted-monitor-program.md
  role: examined
  excerpt: '[IR FR-357](ix://agent-ix/quire-contract-ir/FR-357) owns the complete
    selected

    generated-source type/declaration projection needed for this environment.

    Its first bounded witnessed subset, exercised by merged integration PR #47,

    reads explicitly selected model declaration/field and equality facts only;

    it does not traverse the monitor root or establish the complete closure. That
    subset alone SHALL NOT be used to construct a complete

    `TypeEnvironment` or RT `ProgramParts`.'
- id: TC-029-reachable-unsupported
  path: spec/test-cases/TC-029-admitted-monitor-program-preparation.md
  role: examined
  excerpt: "6. Supply a reachable admitted ordinary operation not in RT's closed\n\
    \   vocabulary and an unrelated unsupported node outside the selected closure.\n\
    \   The former gives the exact unsupported source locus before init; the latter\n\
    \   does not affect the selected plan. Place a checked unsupported ordinary\n\
    \   operation, such as integer division or modulus, in a reachable rendered\n\
    \   entry, then recompile/re-emit and strictly admit it with fresh matching\n\
    \   selection and witnesses. A changed held atom alone may only change an\n  \
    \ event capture, not the rendered transition; it is not this lowering contr"
- id: TC-029-expected-prerequisites
  path: spec/test-cases/TC-029-admitted-monitor-program-preparation.md
  role: examined
  excerpt: 'Each [FR-015](../functional/FR-015-prepare-an-admitted-monitor-program.md)

    criterion has executed evidence with exact positive and adverse outcomes.

    Partial construction or pilot evidence remains partial; it does not qualify

    all of RT FR-276 or close IR-722. QSL-1463 retains and rekeys final-emission

    equality producer/fact and selected source-unit rows. QSL-1630 supplies

    body-sensitive plan identity and QSL-1631 supplies source enum

    declaration/member rows; collections are structural and need no identity

    row. Authentic generated source record/member handoff and source union

    admission/projectio'
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
```
