---
type: master-requirements
name: quire-driver
org: agent-ix
component_type: rust-lib
implementation_language: rust
standards_alignment:
  - iso-iec-ieee-29148
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: contains
  - target: ix://agent-ix/quire-driver/AD-001
    type: contains
  - target: ix://agent-ix/quire-driver/TM-001
    type: contains
  - target: ix://agent-ix/quire-driver/FR-001
    type: contains
  - target: ix://agent-ix/quire-driver/FR-002
    type: contains
  - target: ix://agent-ix/quire-driver/FR-003
    type: contains
  - target: ix://agent-ix/quire-driver/FR-004
    type: contains
  - target: ix://agent-ix/quire-driver/FR-005
    type: contains
  - target: ix://agent-ix/quire-driver/FR-006
    type: contains
  - target: ix://agent-ix/quire-driver/FR-007
    type: contains
  - target: ix://agent-ix/quire-driver/FR-008
    type: contains
  - target: ix://agent-ix/quire-driver/FR-009
    type: contains
  - target: ix://agent-ix/quire-driver/FR-010
    type: contains
  - target: ix://agent-ix/quire-driver/FR-011
    type: contains
  - target: ix://agent-ix/quire-driver/TC-001
    type: contains
  - target: ix://agent-ix/quire-driver/TC-002
    type: contains
  - target: ix://agent-ix/quire-driver/TC-003
    type: contains
  - target: ix://agent-ix/quire-driver/TC-004
    type: contains
  - target: ix://agent-ix/quire-driver/TC-005
    type: contains
  - target: ix://agent-ix/quire-driver/TC-006
    type: contains
  - target: ix://agent-ix/quire-driver/TC-007
    type: contains
  - target: ix://agent-ix/quire-driver/TC-008
    type: contains
  - target: ix://agent-ix/quire-driver/TC-009
    type: contains
  - target: ix://agent-ix/quire-driver/TC-010
    type: contains
  - target: ix://agent-ix/quire-driver/TC-011
    type: contains
  - target: ix://agent-ix/quire-driver/TC-012
    type: contains
  - target: ix://agent-ix/quire-driver/TC-013
    type: contains
  - target: ix://agent-ix/quire-driver/TC-014
    type: contains
  - target: ix://agent-ix/quire-driver/TC-015
    type: contains
  - target: ix://agent-ix/quire-driver/TC-016
    type: contains
  - target: ix://agent-ix/quire-driver/TC-017
    type: contains
  - target: ix://agent-ix/quire-driver/TC-018
    type: contains
  - target: ix://agent-ix/quire-driver/TC-019
    type: contains
---
# Master Requirements Specification

## quire-driver

## 1. Purpose

`quire-driver` is the downstream driver library and the owner of the one
user-facing `quire` binary for the checked request lifecycle. It coordinates
QSL, IR and CG operations while leaving each upstream's domain policy with
that owner. This specification reconciles the driver scope formerly tracked
under IR-351 as IR-527.

## 2. Scope

### 2.1 In Scope

The driver orchestrates checked compile and package handoff, backend
registration and negotiation, routing, generation, execution, replay, and
terminal outcome recording. It provides one command line across the lifecycle
operations defined by QSL ADR-029. Driver operations own `prove` and
`generate`; the remaining lifecycle operations delegate to their typed QSL
library entries.

QSL owns lifecycle policy and the `quire-outcome/1`, diagnostic, exit-category
and operation contracts. The driver preserves those typed outcomes and uses
QSL's `Category::exit_code` as the sole category-to-process exit mapping.
`ix-cli-kit` supplies shared CLI mechanics. FR-002 defines the direct numeric
pass-through at the process boundary until the kit can carry QSL's codes.

### 2.2 Out of Scope

- QSL syntax, checking, replay semantics, plugin protocol, cache policy and
  execution-backend semantics; this repository specifies their composition
  according to QSL ADR-029, not replacement contracts.
- IR checked-package admission semantics, and CG negotiation, claim derivation,
  generation, atomic publication and terminal mapping semantics.
- A competing QSL or CG user CLI, a second outcome schema or exit mapping,
  and automatic engine substitution.
- G5b test implementation or gate evidence in this repository. IR-528 owns
  that evidence in `quire-integration`.
- Implementation changes and dependency updates. TC-018 adds the binary process
  boundary case for the merged FR-002 requirement; it does not expand that
  requirement's scope.

## 3. System Overview

A downstream Rust driver library and one user-facing `quire` binary. The
binary builds typed requests, calls QSL or driver operations, and renders the
returned outcome. The library sequences QSL, IR and CG calls and builds the
registry value passed to consumers. QSL, IR and CG do not depend on the driver;
the driver frontend and cross-repository integration suite are downstream
consumers. CLI mechanics remain outside the qualified core (QSL ADR-029 CB-3).

## 4. Requirements Architecture

`quire-driver` owns the sequencing across QSL, IR and CG for the checked
request lifecycle and supplies the sole user-facing `quire` binary. It sits
downstream of those libraries. IR-527 absorbs the driver spec, architecture,
requirements and test-matrix scope previously tracked by IR-351. The tables
index the discrete normative artifacts; they do not replace their criteria.
FR-002 through FR-010 specify SP-D1 through SP-D9 respectively.

| ID | Title |
|---|---|
| [US-001](usecase/US-001-run-a-checked-request.md) | Run one checked request from source to routed backends |
| [FR-001](functional/FR-001-orchestrate-the-driver-steps.md) | Orchestrate compile, backend conversion, negotiation, routing and generation |
| [FR-002](functional/FR-002-expose-driver-cli-and-outcome-contract.md) | Expose the lifecycle through one driver CLI and outcome contract |
| [FR-003](functional/FR-003-apply-caller-selected-limits.md) | Apply caller-selected lifecycle limits |
| [FR-004](functional/FR-004-select-the-execution-engine.md) | Select the execution engine without substitution |
| [FR-005](functional/FR-005-register-first-party-providers.md) | Register first-party providers through one typed interface |
| [FR-006](functional/FR-006-host-provider-plugins.md) | Route negotiated items to process providers |
| [FR-007](functional/FR-007-cache-results-by-content-identity.md) | Supply complete provider identity to the QSL cache |
| [FR-008](functional/FR-008-aot-execution-backend.md) | Invoke the selected AOT execution backend |
| [FR-009](functional/FR-009-settle-execute-and-replay-results.md) | Settle executions after replaying counterexamples |
| [FR-010](functional/FR-010-send-only-bound-items-to-codegen.md) | Send only abstraction-relation-bound items to code generation |
| [FR-011](functional/FR-011-cancel-a-driver-library-operation.md) | Cancel a driver library operation through a caller-owned handle |
| [TC-001](test-cases/TC-001-steps-run-in-order.md) | The steps run in ADR-011 T-13 order |
| [TC-002](test-cases/TC-002-unknown-backend-refusal.md) | An unknown `BackendId` refuses the run before negotiation |
| [TC-003](test-cases/TC-003-route-after-negotiation.md) | `route` runs after negotiation, for `supported` items only |
| [TC-004](test-cases/TC-004-command-level-refusals.md) | Command-level refusals are typed |
| [TC-005](test-cases/TC-005-dependency-edges.md) | The dependency edges match ADR-011 §6.1 and §7.1 |
| [TC-006](test-cases/TC-006-generation-through-cg.md) | E5 admits the emitted package and CG generates for the routed items |
| [TC-007](test-cases/TC-007-generation-refusals.md) | Generation refusals are typed |
| [TC-008](test-cases/TC-008-missing-import-refusal.md) | A missing import or an unknown header profile refuses at compile |
| [TC-009](test-cases/TC-009-cli-outcome-and-exit-contract.md) | Exercise CLI verbs, outcome documents and exit codes |
| [TC-010](test-cases/TC-010-caller-selected-limits.md) | Apply explicit lifecycle limits and preserve limit outcomes |
| [TC-011](test-cases/TC-011-execution-engine-selection.md) | Select only the requested execution engine |
| [TC-012](test-cases/TC-012-first-party-provider-interface.md) | Register and invoke a first-party provider through Provider |
| [TC-013](test-cases/TC-013-plugin-host-protocol-and-failures.md) | Dispatch process-provider results through the driver |
| [TC-014](test-cases/TC-014-cache-content-identity-and-validation.md) | Pass manifest content to the QSL cache |
| [TC-015](test-cases/TC-015-aot-matches-interpreter.md) | Invoke AOT with current checked request inputs |
| [TC-016](test-cases/TC-016-execute-replay-and-terminal-records.md) | Settle one terminal record per item after replay |
| [TC-017](test-cases/TC-017-only-bound-items-reach-codegen.md) | Send only abstraction-relation-bound items to code generation |
| [TC-018](test-cases/TC-018-binary-process-boundary.md) | Preserve the QSL outcome at the binary process boundary |
| [TC-019](test-cases/TC-019-library-cancellation-and-source-release.md) | Propagate library cancellation and release the emitted source provision |

## 5. Verification and Delivery Boundary

[TM-001](tests.md) integrates FR-001 through FR-011 and TC-001 through TC-019.
It distinguishes existing orchestration tests from planned driver tests and
records their verification owners. Cross-backend and CLI/library conformance
formerly described by CG FR-005 and IR-337 feed IR-528's G5b gate exclusively
in `quire-integration`; CG retains its local generation and atomic publication
obligations. TC-017 is a QSL-owned cross-repository binding test in that suite
and awaits QSL AR2 / QSL-514.

This reconciliation integrates the merged SP-D1–D9 specifications from driver
PRs #15 and #17. TC-018 was added in PR #18 to verify the binary boundary for
FR-002. IR-616 specifies caller-owned library cancellation and the
post-handoff source-provision lifetime; CLI signal and deadline triggers
remain planned. IR-606 adapted the orchestrator to current upstream lifecycle APIs.
IR-607 through IR-610 deliver the remaining CLI, options, providers, cache,
AOT and proof execution chain. IR-351's shared specification work is absorbed
here, with no separate dependency revision ledger or policy artifact.

## References

- [QSL ADR-029](https://github.com/agent-ix/quire-spec-language/blob/main/spec/decisions/ADR-029-lifecycle-cli-provider-plugin-cache-codegen-boundaries.md), with ADR-011, ADR-012 and ADR-013.
- CG FR-005 CLI conformance and IR-337 parity scope; the one CLI is specified
  here, while CG retains codegen-specific atomic publication requirements.
- [IR-527](https://linear.app/agent-ix/issue/IR-527) owns this reconciliation;
  [IR-528](https://linear.app/agent-ix/issue/IR-528) owns G5b gate evidence in
  `quire-integration`.
- [Driver architecture](assurance/AD-001-driver-architecture.md).
- [Driver test matrix](tests.md).
