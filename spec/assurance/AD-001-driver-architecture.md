---
id: AD-001
title: "quire-driver architecture: one CLI and the checked request lifecycle"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-driver library and its user-facing quire binary, downstream of QSL lifecycle operations, IR checked-package admission, CG generation and terminal mapping, and providers
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: references
  - target: ix://agent-ix/quire-driver/FR-001
    type: references
  - target: ix://agent-ix/quire-driver/FR-002
    type: references
  - target: ix://agent-ix/quire-driver/FR-003
    type: references
  - target: ix://agent-ix/quire-driver/FR-004
    type: references
  - target: ix://agent-ix/quire-driver/FR-005
    type: references
  - target: ix://agent-ix/quire-driver/FR-006
    type: references
  - target: ix://agent-ix/quire-driver/FR-007
    type: references
  - target: ix://agent-ix/quire-driver/FR-008
    type: references
  - target: ix://agent-ix/quire-driver/FR-009
    type: references
  - target: ix://agent-ix/quire-driver/FR-010
    type: references
  - target: ix://agent-ix/quire-driver/FR-011
    type: references
  - target: ix://agent-ix/quire-driver/FR-012
    type: references
  - target: ix://agent-ix/quire-driver/FR-013
    type: references
  - target: ix://agent-ix/quire-driver/FR-014
    type: references
  - target: ix://agent-ix/quire-driver/TM-001
    type: references
  - target: ix://agent-ix/quire-driver/TC-019
    type: references
  - target: ix://agent-ix/quire-driver/TC-020
    type: references
  - target: ix://agent-ix/quire-driver/TC-021
    type: references
  - target: ix://agent-ix/quire-driver/TC-022
    type: references
  - target: ix://agent-ix/quire-driver/TC-023
    type: references
  - target: ix://agent-ix/quire-driver/TC-024
    type: references
  - target: ix://agent-ix/quire-driver/TC-025
    type: references
  - target: ix://agent-ix/quire-driver/TC-026
    type: references
  - target: ix://agent-ix/quire-driver/TC-027
    type: references
  - target: ix://agent-ix/quire-driver/TC-028
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-012
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-307
    type: references
  - target: ix://agent-ix/quire-contract-codegen/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-codegen/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
---
# quire-driver architecture

IR-527 reconciles this seam description with QSL ADR-029 and the merged
SP-D1 through SP-D9 requirements. It absorbs IR-351's driver specification,
architecture and verification-inventory work. [The master specification](../spec.md)
and [TM-001](../tests.md) integrate FR-001 through FR-014, and
TC-001 through TC-028. This architecture describes their composition
and ownership; the discrete requirements supply the normative acceptance
criteria.

## System Boundary

The driver repository owns the downstream orchestration library and the one
user-facing Cargo `quire` binary. FR-014 places the binary in the separate
`quire-cli` workspace package. `quire-cli/src/cli.rs` owns argument parsing;
`quire-cli/src/main.rs` is the binary entry point. CLI tests and their fixtures
also belong to `quire-cli`. The CLI composes the driver, cache and separate
`quire-plugin-host` package. The binary reads explicitly named inputs, builds
typed requests, calls the appropriate library operation, and renders its
typed outcome. `prove` and `generate` are driver operations;
QSL owns the other lifecycle operations and the lifecycle contract. The CLI
performs no semantic inference from diagnostic text.

The driver composes QSL, IR and CG. It owns call order, registry construction,
provider dispatch, proof execution, counterexample replay orchestration and
writing one FR-331 terminal record per item in the CG request. QSL owns checking,
execution and replay semantics and the outcome types. IR owns checked-package
admission. CG owns negotiation, claim derivation, generation, atomic artifact
publication and the mapping from provider execution plus replay to QSL's
terminal value. The driver defines no alternative to these contracts.

QSL ADR-029 also defines the plugin, cache and execution-backend contracts.
Its CB-3 places the host, cache, AOT adapter and CLI outside the qualified core
in driver-repository components. The `quire-cli` package is a workspace
default member and composes the CLI with `quire-driver`, `quire-cache` and
`quire-plugin-host`.
The `quire-driver` package owns the cache interface and SHALL NOT depend on
the concrete `quire-cache` package. `quire-cache` implements that interface
as a downstream component; `quire-cli` composes it without adding it to the
`quire-driver` dependency closure.
FR-006 through FR-008 specify
dispatch and typed input/result preservation against QSL's PL, CA and EB
contracts; FR-012 specifies the concrete cache component under QSL's CA
policy.

## Views

The following views describe the request flow, dependency direction and
failure ownership across the driver and its upstream libraries.

### Lifecycle composition

| Boundary | Contract owner | Driver responsibility | Verification |
|---|---|---|---|
| User command to typed operation | QSL ADR-029 LC-1, CB-1, CB-4, CB-5; driver frontend | Expose one `quire` command surface, dispatch to QSL or driver operations, preserve the library outcome and its category exit result | FR-002 / TC-009, TC-018 |
| Caller limits and cancellation | QSL ADR-029 LC-3..LC-4 | Pass the caller-owned `Cancel` handle through every lifecycle stage; preserve typed cancellation. The library reads no clock. | FR-003, FR-011 / TC-010, TC-019 |
| Engine choice | QSL ADR-029 EB-8 | Select only the requested engine, preserving typed failures | FR-004 / TC-011 |
| Compile, check and E4 package emission | QSL typed lifecycle entries | Compose checking and packaging for the request; obtain the in-process checked package and emitted `quire.checked-package/v2` bytes | FR-001 / TC-001, TC-008; API adaptation delivered by IR-606 |
| E5 package admission | IR `read_checked_package` in `quire-contract-model` | Pass emitted bytes and checked-package evidence to IR; admit E4's emitted bytes before proceeding | FR-001 / TC-004, TC-006 |
| Registry and candidates | QSL layer R and ADR-029 PV | Build one registry from admitted manifests and pass it as a value; derive candidate sets from checked request items | FR-001, FR-005 / TC-002, TC-012 |
| Abstraction-relation selection | QSL AR2, [FR-307](https://github.com/agent-ix/quire-spec-language/blob/main/spec/functional/FR-307-export-the-bindings-each-item-references.md) (Behavior: Caller and position; Per-item results) | Pass only the admitted relation's bound item subset to CG; preserve QSL's typed required-binding refusal | FR-010 / TC-017 |
| Conversion and negotiation | CG; QSL ADR-029 PV-4 defines the process-provider extension | Project registered descriptors through CG's origin-aware conversion before negotiation. Validate settlement count, order and membership | FR-001, FR-006 / TC-002, TC-004, TC-013 |
| Routing | QSL `qsl-route` | Route after negotiation and only for supported items; retain request indices and checked-node identity | FR-001 / TC-003 |
| Generation and publication | CG `generate_routed` and CG FR-005 | Supply only routed, relation-bound items and caller contexts; carry generation output and CG-derived claim map without re-derivation | FR-001, FR-010 / TC-006, TC-007, TC-017 |
| Scalar call-to-generation composition | Driver owns the production join; QSL FR-367 binds the admitted call and exact occurrence; IR FR-049 lowers expression and optional context roots; CG FR-330 accounts for positions | Retain the call and ordered bound operands in a driver sidecar keyed by request index. Pass CG only its admitted package, complete typed positions, routed items and contexts; context roots create no routed claim. Do not pass QSL call/binder types into CG. | FR-001-AC-13..FR-001-AC-16 / TC-006; native receipt qualification remains TC-016 / IR-773 |
| First-party provider execution | Driver `Provider` trait; QSL ADR-029 PV | Invoke only the selected first-party provider with the current emitted package, its negotiated items, limits and cancellation; preserve typed results | FR-005 / TC-012 |
| Process-provider seam | Driver-owned process-provider interface; `quire-cli` composition adapter | The host hello admitter returns the once-admitted descriptor; its constructing caller supplies the QSL origin, and the driver registers it with linked manifests through its single QSL registry without re-admission; `quire-cli` builds no second registry; the driver dispatches selected process-provider work and settles validated results without a host-package dependency | FR-005, FR-006 / TC-012, TC-013, TC-028 |
| Process plugin host | `quire-plugin-host`, composed by `quire-cli`; QSL ADR-029 PL-1..PL-7 | Start hello-first sessions, enforce framing and caller budgets, preserve validated results and typed failures, terminate the process tree on cancellation or timeout; Windows kill-on-close mechanism is planned under IR-674 and not provided by the merged host | FR-013 / TC-023..TC-028 |
| CLI plugin lifecycle exit | `quire-cli` process-provider adapter | Close active sessions before normal CLI exit; on Linux/macOS handle SIGINT/SIGTERM through cancellation and cleanup, then restore the default disposition and re-raise the signal; Windows console-control cleanup is not claimed | FR-006 / TC-028 |
| Eligible cache calls | Driver cache interface and QSL ADR-029 CA-1..CA-5 | Supply operation, package and provider identity, typed request and generation options; preserve typed hit/miss results without depending on `quire-cache` | FR-007 / TC-014 |
| Cache key and storage | `quire-cache`, implementing QSL ADR-029 CA-1..CA-5 | Derive and verify content identity, enforce typed reads, storage policy and caller byte limits | FR-012 / TC-020..TC-022 |
| AOT execution | QSL ADR-029 EB | Pass the current checked package, selected entries, limits, cancellation and named toolchain to the selected backend; preserve failures without switching engines | FR-008 / TC-015 |
| Counterexample replay | QSL S6a / replay operation | Replay each provider counterexample through QSL; retain execution and replay as distinct inputs to settlement | FR-009 / TC-016 |
| Terminal settlement and outcome recording | CG FR-331 mapping, QSL terminal/outcome types | Call the authoritative mapping with provider result and replay result; write exactly one terminal record per item in the CG request, including unsupported and incomplete items | FR-009 / TC-016 |

The existing FR-001 sequence is compile/emit, E5 admission, candidate
construction, backend conversion, negotiation, routing and generation.
FR-010 qualifies the CG input: only relation-bound items enter negotiation
and generation. Selection never discards the need to account for each
requested item: a settled item in CG's settlement and generation output (and,
for a proof, in FR-009's outcome), a refused item in FR-001's per-item
refusal member. Proving extends generation with
provider execution, QSL replay and CG settlement; a `generate` request does
not imply that a proof was executed.

FR-011 places cancellation at the driver library boundary: a caller owns a
cloneable QSL handle and may cancel from another thread. QSL's typed result,
`incomplete` category and CB-4 exit mapping remain authoritative. FR-006
specifies CLI handling of SIGINT/SIGTERM for active plugin sessions; a
separate CLI deadline trigger remains planned. The E4
`EmittedUnit` source provision is used as E5 evidence and then released.
`Driven.package` retains QSL's `CheckedUnit` source so callers can resolve
diagnostic loci without reacquiring source. Under FR-001's current empty
`DependencyInput`, it retains only the root source, bounded by the caller's
per-source limit; the resolved library-source list is empty. Releasing
`EmittedUnit` drops its duplicate provision after hand-off.

Types cross the QSL/CG seam in their owned wire spellings (ADR-013 T-7).
The driver treats an unknown capability spelling as `VocabularyMismatch`
and looks up each routed `NodeKey` in IR's admitted package to obtain
its `CheckedNodeId`. It re-derives no checked semantics or claim.

FR-001's existing compile path uses an empty `DependencyInput`, so an import
without supplied selection refuses at intake. This reconciliation does not
add library-source intake or expand that baseline contract.

### Dependency direction and frontend placement

- QSL, IR and CG do not depend on the driver. The driver library depends on
  their layer/model APIs; downstream components and `quire-cache` may
  consume its cache interface. The `quire-driver` package does not depend on
  `quire-cache` or `quire-plugin-host`; `quire-cli` composes the driver, cache
  and process host. CG's QSL seam remains `qsl-replay`; the driver may call QSL
  layer crates directly (ADR-011 §6.1, FB-11).
- IR admission and checked types come from `quire-contract-model`, not a
  driver-local representation. The driver, CG and QSL use one copy of that
  model type, enforced by compilation and the existing TC-005 graph check.
- FR-011's public cancellation input is QSL's `quire_exact::Cancel` handle, so
  the driver has a direct `quire-exact` edge to expose and pass that type; QSL
  retains cancellation semantics and cause ownership.
- FR-001 and TC-005 describe the orchestrator's dependency graph. The frontend
  and non-core components added under ADR-029 CB-3 sit above the driver core.
  The `quire-cli` package owns its parser, CLI tests and fixtures. The
  `quire-driver` package's cache-free, host-free closure is defined by
  FR-001-AC-11, FR-012-AC-13 and FR-013-AC-11; `quire-cli` composes the
  separate cache and process-host implementations.
  An argument parser, terminal renderer, plugin host, cache implementation or
  alternate execution backend does not become
  a dependency of the core.
- `ix-cli-kit` adoption is staged under IR-607. The first code slice adopts
  the exit module's `caller_exit_code` to preserve QSL's numeric category
  result at the process boundary, without mapping it through the kit's
  `Outcome`; reserved code `5` is a QSL mapping invariant failure handled by
  the CLI. Three other shared CLI modules remain open: `streams` for result,
  diagnostic and colour handling; `json` for canonical machine-output
  encoding; and `config` if the CLI introduces configuration. FR-002-AC-11 and
  AC-12 specify the package semantic-version output and built-binary agreement;
  implementation and evidence remain planned. The current CLI has no config
  reader. The kit's feature-gated `secrets` module is outside the driver's
  adoption scope.
  The argument parser remains in `quire-cli`; the kit defines no argv parser.
- First-party Git dependencies are declared with `branch = "main"`; each
  consumer's `Cargo.lock` records the resolved commit. Dependency manifests
  do not carry a `rev` field or commit identifier.
- Dependency declarations and lockfiles are the package dependency source;
  this architecture creates no separate revision ledger or tool catalog.

### Failure flow

| Condition | Contract owner / driver action | Outcome |
|---|---|---|
| Invalid command, option, limit or engine name | Frontend invokes QSL's argument/outcome contract before a lifecycle stage | QSL typed refusal |
| Recognized engine is not linked | Driver preserves the engine-availability contract | Unsupported; no alternate engine runs |
| Compile, package or replay refuses, reaches a limit, cancels or faults | QSL typed operation result passes through | Owner category and cause retained |
| Manifest registration refuses or a backend has no CG conversion | Driver reports the registry/conversion refusal before negotiation | Typed refusal |
| CG envelope negotiation refuses | CG refusal passes through | Typed refusal |
| IR refuses QSL's just-emitted bytes | Driver stops at E5 before conversion | Internal fault: upstream contract inconsistency |
| IR reader reaches its limit | Driver preserves handoff limit cause | Limit outcome |
| Settlement count/order differs, a supported backend is outside candidates, or capability spelling is unknown | Driver rejects the inconsistent upstream answer | Internal fault |
| Routing has no usable target, routes outside the request, or names a node absent from IR's admitted package | Driver stops before generation | Internal fault (`runtime_invariant`) |
| Required abstraction-relation binding is absent | QSL owns the refusal; driver sends no unbound item to CG | Typed unbound-element refusal |
| CG generation refuses | Driver carries CG's classified result | Invalid request, limit or internal fault, as specified by FR-001 |
| CG cannot derive an item's claim | CG settles that item; driver preserves its siblings | Unsupported / `no_derivable_claim` |
| Plugin host or AOT preparation fails | Driver preserves the owner's typed failure | No success conversion or engine substitution |
| Replay reproduces a provider counterexample | QSL replay feeds CG's terminal mapping | Refuted |
| Replay disagrees or completes without reproducing the counterexample | QSL replay feeds CG's terminal mapping | Inconclusive with replay-parity cause |
| Replay refuses or faults | Driver retains typed replay cause and uses CG's mapping | Refusal cause retained; a replay fault is an internal failure, never refuted |

QSL's `quire-outcome/1` value is the machine outcome. QSL's category/severity
contract decides the process status, including multi-item outcomes and stream
failures; the driver and ix-cli-kit introduce no additional mapping.

## Decisions

| Decision | Reason and observable tradeoff |
|---|---|
| One user-facing CLI in the driver repository | It can see every lifecycle stage without a QSL-to-CG dependency cycle. QSL supplies library operations; a separate CG CLI would duplicate the user contract. |
| Policy remains with each domain owner | Driver tests can check sequence, inputs, dispatch and unchanged typed results. QSL owns plugin/cache/AOT contracts; the downstream `quire-cache` component implements CA-1..CA-5, admission stays with IR, and generation/settlement stays with CG. |
| Replay precedes counterexample settlement | `refuted` requires reproduction under QSL's reference semantics. Failed parity remains inconclusive; the driver calls CG's mapping rather than constructing another one. |
| Core and frontend dependency closures are separate | Shared CLI mechanics and non-core adapters remain usable without pulling them into the qualified core. No additional qualification binary is delivered by this reconciliation. |
| Library callers own cancellation | Realtime and embedded callers can stop a run without a process signal or driver-owned timer; a CLI trigger can later remain a thin adapter over the same handle. The driver adds no polling or exit mapping. |
| IR-674 Windows Job Object mechanism (proposed decision) | Recommend an upstream safe opt-in on `process-wrap`'s standard-process flavor that enables Job Object kill-on-close, matching the existing Tokio-flavor capability. The measured standard flavor creates its Job Object with kill-on-drop disabled, while only the Tokio flavor exposes the opt-in; assigning a second Job Object after spawn is too late because plugin code may already execute. A separate safe wrapper would own Windows-specific suspended-process creation, assignment, and lifecycle maintenance; migrating the host to Tokio is out of scope; an unsafe exception conflicts with this crate's `unsafe_code = forbid` policy and is not assumed. The host adopts the upstream API only after it is available. Until then, Windows budget setup that requires this boundary fails closed before request dispatch. This is an IR-674 mechanism decision, not an implementation or host-approval claim. |
| macOS process-resource boundary (proposed decision) | The reviewed public APIs do not establish one complete, bounded process-tree boundary with cumulative CPU, current resident-memory sampling, and termination. XNU identifies `libproc` as private and subject to change; per-PID enumeration and sampling cannot establish an atomic cohort when descendants reparent or exit. Apple’s descendant Endpoint Security client requires a granted entitlement, an installed system extension, and is documented as beta; it delivers process events but does not itself provide the required aggregate accounting or cleanup boundary. Do not treat it as a substitute without a prototype that proves every FR-013 property and a deployment decision for the extension. Until then, keep macOS budget requests typed `Unavailable` before dispatch. This preserves fail-closed behavior and does not claim full process-tree enforcement or close IR-787. [XNU libproc](https://github.com/apple-oss-distributions/xnu/blob/main/libsyscall/wrappers/libproc/libproc.h); [Endpoint Security](https://developer.apple.com/documentation/EndpointSecurity); [descendant client](https://developer.apple.com/documentation/endpointsecurity/es_new_descendants_client%28_%3A_%3A%29). |
| Retain checked-unit source; release E4 provision after E5 | The checked unit's source accessors let callers resolve diagnostic loci without reacquiring source. Under FR-001's empty `DependencyInput`, only the root source is retained, bounded by `SpineLimits.source.source_bytes`; releasing `EmittedUnit` drops its duplicate provision. |
| G5b is a cross-repository gate | IR-528 gathers evidence in `quire-integration` only. Local driver and owner tests verify components; they cannot claim the whole gate passed. |

The requirement/test inventory in TM-001 is the checkable expression of these
decisions. FR-002..FR-010 correspond to SP-D1..SP-D9; FR-001 retains the
existing generation seam, and FR-012 owns the downstream cache implementation.
CG FR-005's CLI-facing conformance and IR-337's parity/bound-suite obligations
feed G5b through the application binary. CG continues to own local atomic
publication and generation checks.

## Risks

This is a proposed architecture and a specification integration, not a claim
that the planned lifecycle is implemented. Existing source tests for
TC-001..TC-008 are present; no source tests are run by this reconciliation.
The merged FR-002..FR-010 and TC-009..TC-018 specify the planned extension.

| Open work or risk | Owner | Required evidence / review point |
|---|---|---|
| The upstream lifecycle APIs can change across repositories | Driver maintenance | Keep the adapter aligned with the typed QSL/IR/CG APIs; the IR-606 adaptation is merged in PR #14. |
| Every QSL-enforced accounting bound is not yet caller-settable through the CLI | QSL-656 / IR-684 | QSL must expose the ten counters through `accounting.<counter>` in its canonical operand parser and caller-limit DTO; until that API lands, the driver accepts no bare accounting-name compatibility layer and FR-003's every-bound goal remains unmet. |
| CLI, caller limits/cancellation/engine selection and provider/plugin/cache composition remain planned | IR-607, IR-608, IR-609, IR-616 | Implement and run TC-009..TC-014, TC-018..TC-022 against their owning contracts. TC-014 verifies the injected interface; TC-020..TC-022 verify the separate cache package. CLI signal/deadline integration is not claimed by FR-011. |
| AOT and execute/replay/terminal-record chain remain planned | IR-610, with CG's terminal-map owner | Run TC-015..TC-016 with paired provider/replay inputs and complete per-item accounting. CG AD-003 is merged (CG PR #214); its merge does not establish terminal-map implementation. |
| Relation-bound CG input depends on QSL AR2 | QSL-514 / [QSL FR-307](https://github.com/agent-ix/quire-spec-language/blob/main/spec/functional/FR-307-export-the-bindings-each-item-references.md) (bound-item export and per-item refusals) | Run TC-017 in the QSL-owned `quire-integration` test once AR2 is available. |
| ix-cli-kit serde minimum conflicts with quire-driver's exact serde requirement | PLAT-1154 | Lower the kit's serde minimum before adding the CLI dependency; no driver or upstream serde requirements move in this slice. The IR-607 code slice is blocked until PLAT-1154 merges. |
| ix-cli-kit adoption is staged; exit pass-through, streams, canonical JSON and package-version agreement are specified, while configuration remains conditional | IR-607 / driver FR-002 | Adopt `exit::caller_exit_code` with no second outcome mapping; use kit streams and JSON for CLI output; implement the `quire-cli` package version surface and verify the built binary with the kit Agreement API. No configuration reader exists; any future reader uses the kit config module. |
| The npm package `@agent-ix/quire-cli` also ships a `quire` executable; its relationship to the Cargo CLI is unresolved | IR-607 / QSL ADR-029 and FR-002 question | Clarify whether npm and Cargo distribution are the same supported user-facing CLI; FR-014 scopes Cargo workspace ownership only. |
| Cross-repository parity can differ despite local tests | IR-528 in `quire-integration` | Collect CLI/library outcome and exit parity, provider/plugin install, cache behavior, AOT parity, QSL binary deletion and the full proof-chain evidence. No G5b pass is claimed here. |
| Windows Job Object kill-on-close needs safe pre-execution assignment | IR-674 | Request the safe opt-in from `process-wrap`; consume it only after an upstream release exposes the standard-flavor capability. Until then, FR-013-AC-14 remains planned and no Windows kill-on-close implementation or native evidence is claimed. |
| No supported public macOS process scope has been demonstrated for FR-013 CPU, resident-memory and bounded-cleanup requirements | IR-787 | Prototype a supported mechanism against the native short-lived/reaped-descendant, session-escape, resident-memory and bounded-termination cases. Endpoint Security alone is not evidence of those guarantees. Until the mechanism is demonstrated, retain the host's typed pre-dispatch `Unavailable` behavior; keep the full macOS requirement open. |
| Local E5 checks cover sampled units, not every node family QSL can emit | QSL emission-to-admission test owner | Retain upstream coverage of emitted forms; driver TC-006 and the runtime handoff check do not establish exhaustive emission coverage. |

Review these seams when each owning implementation is proposed and again
before G5b acceptance. IR-351's shared spec work is absorbed by IR-527;
implementation delivery and gate acceptance stay with their named tickets.
