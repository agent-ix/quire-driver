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
  - target: ix://agent-ix/quire-driver/FR-012
    type: contains
  - target: ix://agent-ix/quire-driver/FR-013
    type: contains
  - target: ix://agent-ix/quire-driver/FR-014
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
  - target: ix://agent-ix/quire-driver/TC-020
    type: contains
  - target: ix://agent-ix/quire-driver/TC-021
    type: contains
  - target: ix://agent-ix/quire-driver/TC-022
    type: contains
  - target: ix://agent-ix/quire-driver/TC-023
    type: contains
  - target: ix://agent-ix/quire-driver/TC-024
    type: contains
  - target: ix://agent-ix/quire-driver/TC-025
    type: contains
  - target: ix://agent-ix/quire-driver/TC-026
    type: contains
  - target: ix://agent-ix/quire-driver/TC-027
    type: contains
  - target: ix://agent-ix/quire-driver/TC-028
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
FR-002 routes that numeric result through `ix_cli_kit::exit::caller_exit_code`,
which preserves every value except reserved code `5`; the CLI handles that
error as a QSL category-mapping invariant failure. The CLI continues to own
argv parsing. Adoption of ix-cli-kit's `streams`, `json` and `version` modules
is required by FR-002 and remains open under IR-607; if the CLI adds
configuration resolution, it will use the kit's `config` module. The kit's
feature-gated `secrets` module is outside the driver's adoption scope.

### 2.2 Out of Scope

- QSL syntax, checking, replay semantics, plugin protocol, cache policy and
  execution-backend semantics; this repository specifies their composition
  according to QSL ADR-029, not replacement contracts. FR-012 specifies the
  driver repository's cache implementation under that upstream cache policy;
  FR-013 specifies the separate driver-repository plugin host under that
  upstream plugin contract; FR-014 specifies `quire-cli` composition of the
  host and driver-owned process-provider seam.
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

The `quire-driver` library sequences QSL, IR and CG calls and builds the
registry value passed to consumers. FR-014 places the one user-facing Cargo
`quire` binary in the separate `quire-cli` workspace package. That package
owns the parser, CLI tests and their fixtures, and is a workspace default
member. It composes `quire-driver`, `quire-cache` and `quire-plugin-host`; the
`quire-driver` package remains independent of the cache and host under
FR-001-AC-11, FR-012-AC-13 and FR-013-AC-11. QSL, IR and CG do not depend on
the driver. CLI mechanics and
the cache implementation remain outside the qualified core (QSL ADR-029
CB-3).

## 4. Requirements Architecture

`quire-driver` owns the sequencing across QSL, IR and CG for the checked
request lifecycle. The `quire-cli` workspace package composes that library
with `quire-cache` and `quire-plugin-host`; the cache and host remain outside
the driver core's dependency closure under FR-001-AC-11, FR-012-AC-13 and
FR-013-AC-11.
IR-527 absorbs the driver spec, architecture, requirements and test-matrix
scope previously tracked by IR-351. The tables index the discrete normative
artifacts; they do not replace their criteria. FR-002 through FR-010 specify
SP-D1 through SP-D9 respectively; FR-012 specifies the downstream cache
implementation; FR-013 specifies the in-repository plugin host; FR-014 specifies the
`quire-cli` package boundary and its parser and test ownership.

| ID | Title |
|---|---|
| [US-001](usecase/US-001-run-a-checked-request.md) | Run one checked request from source to routed backends |
| [FR-001](functional/FR-001-orchestrate-the-driver-steps.md) | Orchestrate compile, backend conversion, negotiation, routing and generation |
| [FR-002](functional/FR-002-expose-driver-cli-and-outcome-contract.md) | Expose the lifecycle through one driver CLI and outcome contract |
| [FR-003](functional/FR-003-apply-caller-selected-limits.md) | Apply caller-selected lifecycle limits |
| [FR-004](functional/FR-004-select-the-execution-engine.md) | Select the execution engine without substitution |
| [FR-005](functional/FR-005-register-first-party-providers.md) | Register first-party providers through one typed interface |
| [FR-006](functional/FR-006-host-provider-plugins.md) | Route negotiated items to process providers |
| [FR-007](functional/FR-007-cache-results-by-content-identity.md) | Integrate eligible operations through the cache interface |
| [FR-008](functional/FR-008-aot-execution-backend.md) | Invoke the selected AOT execution backend |
| [FR-009](functional/FR-009-settle-execute-and-replay-results.md) | Settle executions after replaying counterexamples |
| [FR-010](functional/FR-010-send-only-bound-items-to-codegen.md) | Send only abstraction-relation-bound items to code generation |
| [FR-011](functional/FR-011-cancel-a-driver-library-operation.md) | Cancel driver lifecycle stages through a caller-owned handle |
| [FR-012](functional/FR-012-implement-the-downstream-content-addressed-cache.md) | Implement the downstream structural-key cache |
| [FR-013](functional/FR-013-host-process-plugins.md) | Host process providers through a bounded session |
| [FR-014](functional/FR-014-separate-the-user-cli-package.md) | Place the user CLI in a separate workspace package |
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
| [TC-014](test-cases/TC-014-cache-content-identity-and-validation.md) | Exercise driver calls through a recording cache interface |
| [TC-015](test-cases/TC-015-aot-matches-interpreter.md) | Invoke AOT with current checked request inputs |
| [TC-016](test-cases/TC-016-execute-replay-and-terminal-records.md) | Settle one terminal record per item after replay |
| [TC-017](test-cases/TC-017-only-bound-items-reach-codegen.md) | Send only abstraction-relation-bound items to code generation |
| [TC-018](test-cases/TC-018-binary-process-boundary.md) | Preserve the QSL outcome at the binary process boundary |
| [TC-019](test-cases/TC-019-library-cancellation-and-source-release.md) | Propagate library cancellation and retain bounded checked-unit source |
| [TC-020](test-cases/TC-020-cache-content-identity.md) | Derive cache identity from meaning inputs and manifest content |
| [TC-021](test-cases/TC-021-cache-entry-integrity-and-policy.md) | Reject invalid cache entries and excluded values |
| [TC-022](test-cases/TC-022-cache-storage-bounds-and-concurrency.md) | Keep cache writes atomic and within the caller's size limit |
| [TC-023](test-cases/TC-023-plugin-session-and-admission.md) | Start a plugin session and admit hello before requests |
| [TC-024](test-cases/TC-024-plugin-frames-and-failure-identity.md) | Enforce plugin framing and preserve typed failures |
| [TC-025](test-cases/TC-025-plugin-cancellation-and-deadline.md) | Preserve plugin settlement during cancellation and bound deadlines |
| [TC-026](test-cases/TC-026-plugin-resource-bounds-and-cleanup.md) | Enforce plugin process-tree budgets and bounded cleanup |
| [TC-027](test-cases/TC-027-clean-cache-plugin-host-gate.md) | Run plugin-host tests and dependency checks from a clean cache |
| [TC-028](test-cases/TC-028-cli-package-boundary.md) | Keep the CLI package separate from the driver and compose cache and host |

## 5. Verification and Delivery Boundary

[TM-001](tests.md) integrates FR-001 through FR-014, and TC-001 through
TC-028.
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
AOT and proof execution chain. IR-609 separates driver/cache integration
(FR-007, TC-014) from the downstream cache implementation (FR-012,
TC-020..TC-022). Process-origin backend-kind projection now uses CG's
origin-aware descriptor conversion and the QSL registry origin; process-provider
dispatch and settlement remain gated on their routed-request and result APIs.
The `quire-cli` adapter supplies plugin hello manifests to the QSL registry
builder, which admits them and assigns `Process` origin; the driver passes its
registry descriptors unchanged to CG negotiation. The driver does not
construct `BackendKind::Process`. The mapping checklist is: preserve manifest
domains and bounds in CG's process descriptor; handle `KindOutput::Process`
without an artifact; and use the plugin host plus typed FR-331 reader for
execution and terminal results. Same-identity descriptors with different
origins conflict and withdraw that identity. IR-351's shared specification work is absorbed
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
