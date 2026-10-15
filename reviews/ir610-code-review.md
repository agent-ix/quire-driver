---
id: SR-6001
title: "Code review — quire-driver PR #53 full diff"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; complete PR #53 diff 49886e1..e8941e9"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-driver/FR-001"
    type: reviews
  - target: "ix://agent-ix/quire-driver/FR-004"
    type: reviews
---

## Summary

Reviewed the full Rust/Cargo change, with the Rust lane focused on dependency boundaries and test-to-production seams. Two medium findings remain: the runtime serde_json edge contradicts FR-001-AC-1, and the new FR-004-AC-5 test replaces the production call closure it claims to verify.

## Verdict

**CONDITIONAL** — the adapter changes are broadly coherent, but the dependency contract and one claimed production behavior test need correction.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The PR promotes serde_json to a normal driver dependency although FR-001-AC-1 explicitly requires it to remain dev-only; update the architecture contract or avoid the production edge. | Cargo.toml:43; spec/functional/FR-001-orchestrate-the-driver-steps.md:164 | implementation-bug-despite-evidence |
| FND-002 | medium | The FR-004-AC-5 test calls execute_using with a test-authored closure, so it passes even if execute's production closure stops forwarding the caller environment to QSL. Exercise execute's real boundary or narrow the criterion/test claim. | src/execute.rs:244-264 | correct-requirement-no-evidence |

## Scope

Examined the complete PR diff and the changed provider, registry, execution, limits, cache, handoff, CLI, upstream and test paths. Relevant criteria reviewed: FR-001-AC-1/12, FR-003-AC-1..10, FR-004-AC-1..9, FR-005 provider-origin and evidence carriers, FR-006 process-provider request/registration, FR-007 cache identity, and FR-009 execute serialization. No applicable AssuranceProfile was present under spec/assurance.

## Examined scope

| Unit | Path | Role | Excerpt |
| --- | --- | --- | --- |
| `FR-001-AC-1` | `spec/functional/FR-001-orchestrate-the-driver-steps.md` | examined | The driver's dependency edges match the QSL ADR-011 §6.1 allow-list and §7.1 crate graph: its normal and build dependencies are QSL layer crates, CG, `quire-contract-model` and `thiserror` (`serde_json` is a dev-dependency, not a normal one), the driver's `quire-contract-model` is the one CG's IR crate depends on, CG's one QSL edge is `qsl-replay`, and the lock holds exactly one `quire-contract-model` package, so the driver's direct edge, CG's IR crate and QSL's `qsl-package` all resolve the same package id. |
| `FR-004-AC-5` | `spec/functional/FR-004-select-the-execution-engine.md` | examined | The public execution request carries the caller-owned typed QSL `ObjectEnvironment`, and the selected interpreter passes that same value unchanged to QSL `spine::run`. |
| `FR-003-AC-9` | `spec/functional/FR-003-apply-caller-selected-limits.md` | examined | The driver accepts the bare accounting counter names defined by QSL FR-255 through QSL's operand APIs, without adding a namespace or aliases. |
| `FR-003-AC-10` | `spec/functional/FR-003-apply-caller-selected-limits.md` | examined | When QSL resolves a bare accounting counter name, the driver applies its value through QSL `AccountingLimits::set_bound` to `Call.accounting`; the stage `SpineLimits` remain unchanged and the accounting value reaches `spine::run`. |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The driver FR/TC now documents the exception, but QSL ADR-011 §7.1 still gives the driver crate's dependency cell as “the QSL layer crates and CG.” The change does not amend or establish an exception to that architecture contract; the dependency boundary remains unresolved. |
| FND-002 | fixed | The integration test invokes public `execute()` and observes different outcomes for the caller's unresolved-reference environment and an empty environment. |

Artifact custody note: at PR #53 head a0a2ad65c7a799664407dcb4b32da13c4020a9f7, both committed review files were verified byte-identical to these original SR artifacts; no further artifact-content change was needed for custody.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | rejected | The QSL owner clarified that ADR-011 §6.1's driver dependency cell is exhaustive for first-party layer edges, while external general-purpose crates such as serde_json are allowed. The production serde_json use reads only the optional bounds member from retained FR-331 wire; the bytes remain the source of authority. The owner is authoring an ADR clarification. |
