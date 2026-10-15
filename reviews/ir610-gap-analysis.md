---
id: SR-6002
title: "Gap analysis — quire-driver PR #53"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; complete PR #53 diff and mapped requirements"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-driver/FR-001"
    type: reviews
  - target: "ix://agent-ix/quire-driver/FR-004"
    type: reviews
---

## Summary

Mapped changed production paths and tests to the updated requirements and matrix. The RR-16 projection path is exercised for two occurrences, while the environment-forwarding row overstates the production coverage of its unit test; the dependency-edge test also encodes a dependency set that conflicts with the existing AC.

## Verdict

**CONDITIONAL** — two trace/evidence gaps remain in the changed scope.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | `request_carries_the_callers_object_environment` tags FR-004-AC-5 but only passes through an injected closure that itself forwards the environment; it does not execute `execute`'s production closure, so a production forwarding regression remains green. | src/execute.rs:244-264; spec/tests.md:62 | correct-requirement-no-evidence |
| FND-002 | medium | TC-005 now treats serde_json as an allowed normal driver edge, but FR-001-AC-1 says it is dev-only; the implementation and its dependency test therefore disagree with the normative allow-list. | tests/dependency_edges.rs:141-145; Cargo.toml:43; FR-001-AC-1 | implementation-bug-despite-evidence |

## Scope

Compared changed implementations and Trace bindings against FR-001-AC-1/12, FR-003-AC-1..10, FR-004-AC-1..9, the changed TC-006/010/011/013 descriptions, and `spec/tests.md`. Plan completion was not assessed. No deferred IR-610 scope was treated as delivered.

## Examined scope

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| `FR-001-AC-1` | `spec/functional/FR-001-orchestrate-the-driver-steps.md` | examined | The driver's dependency edges match the QSL ADR-011 §6.1 allow-list and §7.1 crate graph: its normal and build dependencies are QSL layer crates, CG, `quire-contract-model` and `thiserror` (`serde_json` is a dev-dependency, not a normal one), the driver's `quire-contract-model` is the one CG's IR crate depends on, CG's one QSL edge is `qsl-replay`, and the lock holds exactly one `quire-contract-model` package, so the driver's direct edge, CG's IR crate and QSL's `qsl-package` all resolve the same package id. |
| `FR-004-AC-5` | `spec/functional/FR-004-select-the-execution-engine.md` | examined | The public execution request carries the caller-owned typed QSL `ObjectEnvironment`, and the selected interpreter passes that same value unchanged to QSL `spine::run`. |
| `TC-011` | `spec/test-cases/TC-011-execution-engine-selection.md` | examined | Through the public library execution request, run a function whose result depends on an object supplied in a nonempty caller-owned `ObjectEnvironment`; run the same call with an empty environment and compare both driver outcomes with direct QSL runs using those exact environments. |
| `tc_005_driver_edges_match_the_adr_011_allow_list` | `tests/dependency_edges.rs` | examined | §6.1 driver row: QSL layer crates, quire-exact (QSL's caller-owned Cancel API), CG, IR's model crate (E5's reader and `CheckedNodeId`), and thiserror and serde_json (the latter reads retained manifest bounds). |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `tests/drive.rs` now invokes public `execute()` with the supplied unresolved-reference environment, observes Boolean(true), and observes an error with an empty environment, so the production forwarding path is exercised. |
| FND-002 | still-open | FR-001 and TC-005 now document `serde_json`, but QSL ADR-011 §7.1 still lists the driver crate's dependencies as “the QSL layer crates and CG.” No upstream ADR amendment or documented architecture exception resolves this remaining dependency-contract mismatch. |

Artifact custody note: at PR #53 head a0a2ad65c7a799664407dcb4b32da13c4020a9f7, both committed review files were verified byte-identical to these original SR artifacts; no further artifact-content change was needed for custody.

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | rejected | The QSL owner clarified that ADR-011 §6.1's driver dependency cell is exhaustive for first-party layer edges, while external general-purpose crates such as serde_json are allowed. The production serde_json use reads only the optional bounds member from retained FR-331 wire; the bytes remain the source of authority. The owner is authoring an ADR clarification. |
