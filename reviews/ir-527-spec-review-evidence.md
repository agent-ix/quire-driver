---
id: SR-043
title: spec-review/evidence review of IR-527 PR 20 FR-010 dependencies
type: SpecReview
analysis: evidence
scope: agent-ix/quire-driver@9d80e93ca60fe788f15f9ffcf3f65c93d9147473; origin/main...HEAD;
  spec/functional/FR-010-send-only-bound-items-to-codegen.md:40-56; ticket IR-527
review_set: subset
---

## Summary

Ticket: IR-527. quoin advise reports Test matches with no mismatch or inconclusive result for FR-010-AC-1..3. quire matrix reports all three locally untagged; unchanged planned TC-017 is not completed evidence. The changed wording correctly treats QSL-514 as planned end-to-end quire-integration evidence for FR-307 caller-position prose, not an extra AC. QSL-514 ticket prose still says AC-6; the user-supplied planner clarification and current normative spec take precedence over stale ticket data.

## Verdict

**PASS** — no defects in the changed dependency prose. Unchanged requirement and TC text and upstream references are context only; implementation readiness is not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

```yaml
clean: true
scope:
- id: FR-010#dependencies-1
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: "- [QSL FR-307](ix://agent-ix/quire-spec-language/FR-307) defines the\n  bound-item export\
    \ and per-item results.\n  [QSpec FR-353-AC-3](ix://agent-ix/quire-specification/FR-353#FR-353-AC-3)\n\
    \  owns the per-item unbound refusal, naming each unbound element with its\n  owning `DomainPackageRef`;\n\
    \  [QSpec FR-331](ix://agent-ix/quire-specification/FR-331) owns the CG\n  request built from bound\
    \ items."
- id: FR-010#dependencies-2
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: "- [QSL ADR-017](ix://agent-ix/quire-spec-language/ADR-017) AR-4/AR-6\n  define the export,\
    \ referenced elements and what CG receives;\n  [QSL ADR-011](ix://agent-ix/quire-spec-language/ADR-011)\
    \ T-13 names\n  the orchestrating driver."
- id: FR-010#dependencies-3
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: "- QSL-514 is planned end-to-end `quire-integration` evidence for FR-307's\n  end-to-end clause,\
    \ not a requirement acceptance criterion. The\n  cross-repository acceptance test is owned by QSL's\
    \ `quire-integration` suite."
- id: FR-010#description
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: 'When an abstraction relation binds a requested item, the driver SHALL send

    only the bound items to CG''s negotiation and generation operations. The

    driver SHALL not send the package''s unbound item set to CG.'
- id: FR-010-AC-1
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: Given a package whose abstraction relation binds a subset of requested items, CG negotiation
    receives exactly that subset and receives no unbound item.
- id: FR-010-AC-2
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: CG generation receives only items that the same admitted relation bound and that negotiation
    settled for generation.
- id: FR-010-AC-3
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: When a binding required for emission is absent, the driver returns the QSL-defined typed unbound-element
    refusal and sends no unbound item to CG.
- id: TC-017#procedure
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: context_only
  excerpt: 'In the quire-integration suite, compose the QSL abstraction-relation intake

    and the driver with a package whose relation binds a strict subset of its

    requested items. Record the items supplied to CG negotiation and generation.

    Repeat with an item that QSL requires to be bound but whose binding is absent.


    Test location: planned QSL-owned cross-repository test, QSL-514.'
- id: QSL:FR-307#description
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: 'When the orchestrating driver requests the abstraction bindings for a set of

    items of a checked package, the layer-4 `package` export SHALL return, per

    item, either the item with every binding it references or a refusal naming

    every model element it references that has no binding. A refused item SHALL

    NOT affect any other item.'
- id: QSL:FR-307#caller
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: 'The export SHALL be callable after S4 and before `route` and E7, and SHALL

    name no `BackendId` (ADR-012 §11). The orchestrating driver (ADR-011 T-13)

    builds the QSpec FR-331 request from the bound items; the end-to-end run

    lives in quire-integration.'
- id: QSL:FR-307-AC-1
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: With only the ConfigVersion object type bound, a request for a clause item that reads only
    ConfigVersion fields and an item whose extent domain ranges over the unbound ConfigVersion population
    returns the first item with its one `ObjectBinding` and refuses the second `missing_declaration`/`missing-name`,
    naming the population's key and its `DomainPackageRef` identity, with no bindings.
- id: QSL:FR-307-AC-2
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: An item that references two unbound elements refuses naming both, in ascending key order, each
    with its owning `DomainPackageRef` identity.
- id: QSL:FR-307-AC-3
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: A requested occurrence key that names no requirement record refuses `missing_declaration`/`missing-name`
    naming the key, and the other requested items' results are unchanged.
- id: QSL:FR-307-AC-4
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: 'An item reading a member that `Base` declares through a receiver of static type `Sub` references
    `Sub`: with only `Base` bound it refuses naming `Sub`, and with `Sub` bound it returns `Sub`''s binding.'
- id: QSL:FR-307-AC-5
  path: ix://agent-ix/quire-spec-language/FR-307
  role: context_only
  excerpt: 'A frame item and a postcondition item of `attemptUpdate` each reference `OperationKey { ConfigVersion,
    attemptUpdate }`: with the frame bound each returns that `FrameBinding`, and without it each refuses
    naming that key.'
- id: QSpec:FR-353-AC-3
  path: ix://agent-ix/quire-specification/FR-353#FR-353-AC-3
  role: context_only
  excerpt: Emission onto implementation code referencing a model element with no bound abstraction relation
    refuses before generation with `missing_declaration`/`missing-name`, naming the unbound element and
    its owning model identity; the refusal is scoped to that item alone, independent sibling items with
    resolved bindings are unaffected, and no partial or approximated relation is emitted in its place.
- id: QSpec:FR-331#description
  path: ix://agent-ix/quire-specification/FR-331
  role: context_only
  excerpt: When a contract package is submitted to a backend provider, the producer SHALL use the immutable
    BackendProviderEnvelope contract below rather than an untyped map, display string or implementation-local
    structure.
- id: QSL:ADR-017-AR-4-Implementation export-1
  path: ix://agent-ix/quire-spec-language/ADR-017
  role: context_only
  excerpt: "A layer-4 `package` function takes the checked\n  package and the requested items, each a\
    \ requirement record's occurrence key\n  (ADR-012 §13.5). It returns, per item, either the item with\
    \ the bindings it\n  references or a refusal. Its caller is the orchestrating driver (ADR-011\n  T-13),\
    \ which calls it after S4 and before `route` and E7, as it calls\n  `route`. It names no `BackendId`\
    \ (ADR-012 §11). For an emission onto\n  implementation code, the driver puts only the bound items\
    \ in the QSpec\n  FR-331 request it sends CG, so CG receives no new inpu"
- id: QSL:ADR-017-AR-4-Implementation export-2
  path: ix://agent-ix/quire-spec-language/ADR-017
  role: context_only
  excerpt: "t; CG reads the\n  bindings from the v2 node."
- id: QSL:ADR-017-AR-4-Referenced elements-1
  path: ix://agent-ix/quire-spec-language/ADR-017
  role: context_only
  excerpt: "An item references: the receiver's static object\n  type of every model member in its checked\
    \ claim (the O-06 declaring node,\n  which is the static type even when a supertype declares the member,\n\
    \  resolved to its `DeclarationKey`); the population of every extent domain\n  (ADR-012 §15.7); and,\
    \ when the record's node is a frame, precondition or\n  postcondition, the `OperationKey` of its operation."
- id: QSL:ADR-017-AR-4-Refusals-1
  path: ix://agent-ix/quire-spec-language/ADR-017
  role: context_only
  excerpt: ", per item:\n  - A requested occurrence key that names no requirement record refuses\n   \
    \ `missing_declaration`/`missing-name`, naming the key.\n  - An item that references an element with\
    \ no binding refuses\n    `missing_declaration`/`missing-name`. The refusal names every unbound\n\
    \    element of the item, in ascending key order, each with its owning\n    `DomainPackageRef` identity\
    \ (QSpec FR-353-AC-3).\n  - A refused item returns no bindings; a bound item returns every binding\
    \ it\n    references. Other items continue.\n- CG's own unbound check (CG#84:"
- id: QSL:ADR-017-AR-4-Refusals-2
  path: ix://agent-ix/quire-spec-language/ADR-017
  role: context_only
  excerpt: " \"A claim whose model elements have no bound\n  relation refuses\") guards requests formed\
    \ outside the driver, as QSpec\n  FR-290's per-item `invalid-request` does (ADR-012 §7.2)."
- id: QSL:ADR-017-AR-6
  path: ix://agent-ix/quire-spec-language/ADR-017
  role: context_only
  excerpt: '| CG#84 Verus (IR-32) | `quire.checked-package/v2` with the Q-2 relation node; `package_id`;
    the QSpec FR-331 request, holding only bound items; the Verus QSpec FR-331 manifest | the bound claim''s
    QSpec FR-290 kind; Verus''s mode per its manifest | AR-2 keys and values; unbound items never reach
    CG |'
- id: QSL:ADR-011-T-13
  path: ix://agent-ix/quire-spec-language/ADR-011
  role: context_only
  excerpt: '| T-13 | The orchestrating driver crate (§6.1 driver row; ADR-012 §7): S1 to S4 compile, E4
    emit, the `route` candidate step, the pre-negotiation `BackendId` conversion (ADR-012 §7.2), E7 CG
    `negotiate_*`, the `route` routing step after E7, CG generation with the returned `BackendId`s, then
    for each generated harness: the S6b Kani run of its obligation, which CG''s backend adapter parses
    into IR''s `KaniOutcome` (ADR-013 O-24); for a `Counterexample`, the E8 packet, which CG builds as
    the O-25 `WitnessEnvelope` with a `ReplaySource::Witness` source (ADR-013 C-10), a'
- id: QSL-514#planner-clarification
  path: user review brief
  role: context_only
  excerpt: QSL-514 is planned end-to-end quire-integration evidence, not a requirement AC.
findings: []
bindings: []
```

## Method Selection

Ran code-review, base, scope-boundary and evidence. Skipped Rust/React/Python lanes and gap-analysis: no executable changes. Skipped EARS and criterion-strength/correctness: no requirement or AC edit. Skipped integrity: no structure change. Skipped dependency: no relationships edge edit. Skipped object: no domain-object edit. Skipped failure-domain, risk-complexity, security and architecture-evaluation: no behavioral, threat, risk or architecture change. No applicable AssuranceProfile found.
