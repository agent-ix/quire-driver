---
id: AD-001
title: "quire-driver architecture: the one crate that calls both QSL and CG"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-driver library, its use of QSL (typed spine lifecycle, route, emitted package) and CG (negotiate, generate), and IR's checked-package reader at the hand-off
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: references
  - target: ix://agent-ix/quire-driver/US-001
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-012
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-013
    type: references
  - target: ix://agent-ix/quire-contract-codegen/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
---
# quire-driver architecture

This is the architecture half of IR-351 (driver spec and architecture) and one of the seam
descriptions of IR-324. IR-351 owns the rest: the spec layout, requirements for each stage the
driver drives, the test matrix and the dependency-revision policy. This AD does not duplicate
them and does not decide the layout; the existing `spec/` tree is untouched.

## System Boundary

The driver is the one crate that depends on both QSL and CG, and nothing depends on it
(`src/lib.rs` crate doc). CG cannot describe the order of the steps because CG cannot depend on
the driver. QSL ADR-011 T-13 lists the steps but is QSL's document; a description that names
both sides, the order between them and the refusals the driver types can only live in this
repository.

The driver runs one request through the pipeline: compile, emit, hand the package to IR's
reader, write candidates, convert backends, negotiate, route, generate. This AD states what the
driver takes from each owner, what it does itself, and what it refuses. It states the driver's
own seam only. The driver also owns the chain after generation: run what was generated (the S6b
Kani run), replay a counterexample through the `qsl-replay` facade (`qsl_replay::replay`), and
write the FR-331 terminal record. QSL ADR-011 T-13 and ADR-013 C-09 record that ownership (QSL-354,
merged). The chain is not implemented yet (see Risks).

## Views

The seam is described as the steps and what crosses at each, the dependency direction, and who
reports each failure.

### What the driver does and what crosses

| Step | Owner of the call | Crosses in | Crosses out |
| --- | --- | --- | --- |
| 1 to 2. S1 to S4 and the E4 emitter | QSL (`spine::parse` → `select` → `check` → `package`, sharing one `Cancel`) | unit source, domain package documents, an empty `DependencyInput`, spine stage limits | in-process checked package, `quire.checked-package/v2` bytes, `package_id` |
| E5. Hand-off | IR (`read_checked_package`) | emitted bytes and `CheckedPackageEvidence` | admitted `CheckedPackageV2` |
| 3. Candidate step | QSL (`qsl_route`) | items written from the compiled unit's own checked requirement records | candidate set per item |
| 4. Backend conversion | CG (`BackendKind::from_identity`) | each registered `BackendId` | a CG kind, or a refusal naming every unconverted id |
| 5. Negotiation | CG (`negotiate_backend_provider`) | FR-331 envelope built from the registry and candidates | one settlement per item, in request order |
| 6. Routing | QSL (`qsl_route::routing::route`) | items settled `supported` only | a target per item |
| 7. Generation | CG (`generate_routed`) | routed items with request index, `CheckedNodeId`, backend, kind, and the caller's contexts | generation output keyed by request index, and the derived claim map |

Types cross between QSL and CG as their wire spellings (QSL ADR-013 T-7); the driver converts a
QSL capability label to a CG capability by name and treats a miss as `VocabularyMismatch`.

The driver owns: the order of the steps, the registry value (`build_registry`), the conversion
of every backend before any negotiation, the lookup of each routed item's `CheckedNodeId` in the
admitted package by `WireNodeId`, and the typed command-level refusal. It owns no semantics of
QSL, CG or IR and re-derives none.

The E5 step checks one content identity: the admitted package's `package_id` must equal the
`package_id` E4 emitted. That is the one content-identity digest the driver itself checks, and
it binds the admitted package to the bytes that were emitted. Other digests cross the seam
unchecked by the driver: the domain-package `sha256-jcs` keys it passes to IR's reader (which
recomputes them) and the manifest digests QSL's registry API requires.

### Dependency direction

- Normal and build dependencies: QSL layer crates, CG, IR's model crate `quire-contract-model`
  and `thiserror` (asserted by `tests/dependency_edges.rs`, TC-005; QSL ADR-011 section 6.1's
  driver row names only the QSL layer crates and CG, so the model crate and `thiserror` are this
  repository's additions to that row). Nothing depends on the driver.
- The driver may depend on QSL layer crates directly. The rule "CG reaches QSL only through
  `qsl-replay`" binds CG, not the driver. QSL depends on no CG crate (ADR-011 FB-11 forbids an
  edge that closes a cycle between QSL and CG); the driver sits downstream of both, so its edges
  close none.
- The driver uses IR's model crate, not IR's root crate, because it needs only the reader and
  the checked-package types. IR's root crate re-exports the model today; its removal is in
  IR-347's scope.
- A type that crosses CG's API (`CheckedNodeId`, `CheckedPackageV2`) must be the one type, so
  the driver and CG must build against one copy of `quire-contract-model`. The compiler enforces
  this the moment a value crosses, and the dependency-edge test gates it as one copy
  (IR-479, PR #12, merged; see Open questions). How the first-party dependencies are spelled is the
  revision policy of IR-351 and IR-479, not decided here.

### Failure outcomes and who reports them

| Condition | Reported by | Kind |
| --- | --- | --- |
| A provider manifest does not register; a duplicate backend | driver (from `qsl_route` refusals) | refusal |
| A lifecycle operation refuses the unit or reaches a limit | QSL (`CompileRefusal`), with its catalog cause preserved | refusal; `stage_limit_exceeded/<kind>-exceeded` is a limit refusal |
| A QSL lifecycle operation reports a fault or cancellation | driver (`DriverFault::Spine`), retaining the operation's compile stage | internal fault (`runtime_invariant`) |
| A registered backend has no CG kind | driver (`invalid_capability/unknown-backend`, ids in bytewise order, before negotiation) | refusal |
| Envelope refusal at negotiation | CG (`EnvelopeRefusal`) | refusal |
| IR refuses the bytes QSL just emitted, or admits them at another `package_id` | driver | internal fault: an upstream broke its contract |
| IR's reader stops at a limit | driver (`HandoffLimit`) | limit refusal |
| Negotiation returns a different number of settlements than requested, or out of request order | driver (`SettlementCount`) | internal fault |
| An item settles `supported` by a backend outside its candidates | driver (`SupportedOutsideCandidates`) | internal fault |
| A QSL capability label has no CG capability | driver (`VocabularyMismatch`) | internal fault |
| Routing returns no usable target for a `supported` item, or routes an item outside the request | driver (`RoutingTarget`) | internal fault |
| A routed item names a node the admitted package does not hold | driver | internal fault (`runtime_invariant`) |
| CG generation refusal | CG (`RoutedGenerationError`), classified by the driver | invalid-request, limit, or internal fault |
| A node with no derivable claim | CG | the item settles `Unsupported` with `no_derivable_claim`; siblings unaffected; the run is not refused |

The driver never converts a refusal to success and never calls `route` before negotiation or
for an item that did not settle `supported`.

## Decisions

The driver orders the steps and types the refusals; each owner keeps its own semantics. The
statements below are what a test can check.

### Invariants a test can check

Candidate statements (local labels; the repo assigns requirement ids when one is authored).

- D-1. The steps run in the order in the table, observable by a recording stand-in for the
  `Upstream` trait (the existing seam in `src/upstream.rs`; existing: TC-001).
- D-2. `route` is called only after negotiation and only with `supported` items; it is not called
  when none is (existing: TC-003).
- D-3. An unknown backend id refuses before any negotiation call (existing: TC-002).
- D-4. The E5 read admits the emitted bytes at the emitted `package_id`; any other result is an
  internal fault before conversion (existing: TC-004).
- D-5. Generation is called once, only for routed items, and not at all when nothing is routed
  (existing: TC-003, step 5).
- D-6. A routed item whose node is missing is an internal fault and CG is not called (existing:
  TC-007).
- D-7. The driver's normal dependencies are exactly the allow-list above, and the graph holds
  one copy of `quire-contract-model` (existing: TC-005; the one-copy assertion was added by
  IR-479, PR #12, merged).
- D-8. The driver reintroduces no CG pin or attestation API and supplies no artifact-digest
  evidence to IR's reader. QSL's own APIs that carry digests the driver does not check
  (`DefinitionLock::pinned()` for the root feature, and `ManifestDigest` in
  the registry API) are existing and listed under Current state.

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps (measured at the commit this AD was written against)

- The catch-up to current QSL, CG and IR landed with IR-479 (PR #12, merged). The driver is no
  longer red against its dependencies. The revision policy belongs to IR-351 and is not repeated
  here.
- E5 evidence in `src/handoff.rs` is now the unit's domain package documents plus the root
  feature of QSL's `DefinitionLock`. The driver compiles with an empty `DependencyInput`, so it
  supplies no dependency packages, and it supplies no artifact-byte or artifact-digest evidence:
  IR's reader no longer checks the lock's raw-artifact digests against evidence (`evidence.rs`
  doc in IR). Before IR-479 the driver called the removed `insert_artifact_bytes` and
  `insert_artifact_digest`, imported CG's removed `KaniToolPins` and `AttestationContext`, and
  carried `manifest_digest` and `pinned_tool` in its negotiation envelope; IR-479 deleted these.
- Still present, as D-8 states: `DefinitionLock::pinned()` (`src/handoff.rs`, read for the root
  feature) and `ManifestDigest` (`src/registry.rs`), which QSL's registry API
  requires of a provider manifest. The driver passes them through and checks none.
  The registry API no longer takes `ToolIdentity`, and the driver stores no tool identity.
- The driver compiles with an empty `DependencyInput` and refuses any `import`
  (`missing_import/missing-selection`, FR-001 step 1). Known and stated, not a gap to fix here.
- The driver owns the execute, replay and terminal-record chain, as QSL ADR-011 T-13 and ADR-013
  C-09 now state (QSL-354, merged): it runs each generated harness's S6b Kani obligation, calls
  `qsl_replay::replay` for a counterexample, and writes the FR-331 terminal record. Today it
  stops after generation: it does not run a harness, read a Kani outcome, call `replay` or write
  a terminal record, so the chain is unimplemented. CG owns the C-09 map and settles the
  dispositions; the map takes the Kani outcome and, for a counterexample, the replay result. The
  driver passes both, writes the results and adds no map or view of its own. Parity is settled
  inside `replay`: only a reproduced counterexample settles `refuted`, and a replay that
  disagrees or completes no value settles `inconclusive` with cause `replay_parity`. A replay
  refusal or fault is classified by C-09 (`replay_refused`
  carries only QSL replay-refusal codes; a replay fault settles `Failed`). Today CG's FR-029 and
  FR-030 map a Kani outcome alone to a terminal value; the two-input terminal step is defined by
  CG's evidence-chain AD (AD-003, quire-contract-codegen PR #214), which is pending and unmerged,
  and this AD follows it once it lands.
- The chain creates no cycle. The driver stays downstream of QSL and CG (ADR-011 FB-11) and
  nothing depends on it.

### Open questions

| Question | Owner | Recommendation | Cost of the alternative |
| --- | --- | --- | --- |
| The dependency-edge test's model-copy assertion | IR-479 (PR #12, merged) | the test gates exactly one copy of `quire-contract-model`; the revision policy stays IR-351's and IR-479's | not decided here |
| Layout, requirements per stage, matrix and revision policy | IR-351 | not decided here | duplicate descriptions |

No compatibility layer is proposed. The driver changes its calls when an owner changes its API.

### Routed gaps

Needs stated to owners, not decisions. Ids are the routing ids of IR-324; they are not
requirement ids. Each matters to the driver only as stated.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R-Q1 | A `TerminalValue` for a non-vacuous inconclusive run. ADR-013 C-09 now specifies `TerminalValue::Inconclusive`, which QSL-351 implements (as relayed, untrusted); the driver consumes it when it writes the terminal record. |
| R-Q4 | A writer for the FR-331 `results` from a `TerminalRecord`. ADR-013 makes the driver the one writer of `results`, so this is now the driver's own work and not a need on QSL; as relayed (untrusted), QSL rejected it as a QSL need. |
| R-Q6 | An emission-to-admission test in QSL over every node family and form QSL can emit; the driver's E5 step repeats that check at run time for one unit. As relayed (untrusted), tracked as QSL-353; the cross-repo test in quire-integration (QSL-342) that the relayed review text mentions concerns the replay seam, not this one. |
