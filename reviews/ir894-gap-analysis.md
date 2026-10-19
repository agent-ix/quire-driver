---
id: SR-6004
title: "Production gap analysis of quire-driver PR #83"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@40c5b2f3a03a2431a0940e406d8ddb51fcd78c51; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/spec.md; src/drive.rs; src/refusal.rs; Cargo.lock; tests/drive.rs; tests/dependency_edges.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
---
# Production gap analysis of quire-driver PR #83

## Summary

Ticket: IR-894. This repository-driven audit checked the frozen PR candidate against FR-001, its computed criterion-to-test matrix, and the production behavior touched by the change. No production behavior introduced by this change lacks an owning requirement; plan completion was not assessed.

## Verdict

**PASS** — all eight FR-001 criteria are statically tagged, the changed behavior is owned by FR-001-AC-5, and no reverse-gap or stub finding was identified in the reviewed production paths.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found | - |

## Coverage

- Reconciliation: `quoin matrix --repo /Users/peter/dev/quire-driver --json` (quoin 0.28.3, quire 0.36.2 / engine 0.50.2); 8 criteria tagged, 0 untagged, 0 tagged-by-ignored-test, 0 method-without-symbol. All eight report no run evidence; no execution evidence was requested or run.
- `quire coverage` reports 8/8 FR-001 criteria backed and no untracked test symbols. Its diagnostics include unrelated declarations that select no documents and a non-binding `FR-075` annotation on `src/drive.rs::drive`; these do not affect the FR-001 matrix.
- Plan completion: not assessed
- Criteria: tagged 8 · untagged 0 · tagged-by-ignored-test 0 · method-without-symbol 0
- Evidence: bound 0 · stale 0 · suspect 0 · undischarged 0 | no run evidence (8 criteria)
- Untraced behaviors / stubs: 0 / 0
- Semantic review: skipped

## Scope

The following criterion texts were examined against code and tests:

- `FR-001-AC-1`: The driver's dependency edges match the QSL ADR-011 §6.1 allow-list and §7.1 crate graph: its normal and build dependencies are QSL layer crates, CG, `quire-contract-model` and `thiserror` (`serde_json` is a dev-dependency, not a normal one), the driver's `quire-contract-model` is the one CG's IR crate depends on, nothing in its lock depends on it, CG's one QSL edge is `qsl-replay`, and the lock holds exactly one `quire-contract-model` package, so the driver's direct edge, CG's IR crate and QSL's `qsl-package` all resolve the same package id.
- `FR-001-AC-2`: A run whose registry holds a `BackendId` with no CG kind refuses before any negotiation, with `invalid_capability`/`unknown-backend` and the unconverted ids in bytewise order.
- `FR-001-AC-3`: `route` is called only after negotiation, and only for `supported` items.
- `FR-001-AC-4`: The steps run in the order of the Description: compile and emit, the E5 read, the candidate step, conversion, negotiation, routing, generation.
- `FR-001-AC-5`: Each command-level refusal carries its step, catalog code and outcome kind. A `Compile` refusal's code renders `code/cause` when the refusing stage names a cause (e.g. `missing_import`/`missing-selection`), not just the bare code.
- `FR-001-AC-6`: CG generation is called once, after routing, over the routed items only, each with its request index, its admitted `CheckedNodeId`, its routed backend and its kind; its output is keyed by request index, ascending; a missing kind context refuses the run, and a routed item naming a node the checked package does not hold is an internal fault.
- `FR-001-AC-7`: The E5 read admits the emitted v2 bytes at the emitted `package_id`; an admitted package at another `package_id`, or an IR refusal, is an internal fault at the hand-off before any backend conversion.
- `FR-001-AC-8`: CG generation's output carries the FR-014 claim map CG derived for the Kani group (IR-294), `Some` whenever one ran; a node whose operation has no derivable claim settles `Unsupported` with `no_derivable_claim` and does not affect its routed siblings' records or harnesses.
