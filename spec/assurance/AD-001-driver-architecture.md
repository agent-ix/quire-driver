---
id: AD-001
title: "quire-driver architecture: the one crate that calls both QSL and CG"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-driver library, its use of QSL (spine compile, route, emitted package) and CG (negotiate, generate), and IR's checked-package reader at the hand-off
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
the driver. QSL ADR-011 T-13 states only the QSL half. A description that names both sides and
the order between them can only live here.

The driver runs one request through the pipeline: compile, emit, hand the package to IR's
reader, write candidates, convert backends, negotiate, route, generate. This AD states what the
driver takes from each owner, what it does itself, and what it refuses. It states the driver's
own seam only. The driver also owns the chain after generation: run what was generated,
replay a counterexample through the `qsl-replay` facade, and settle the terminal record. QSL's
ADR-011 T-13 amendment (QSL-354, pending) records that ownership on QSL's side. The chain is not
implemented yet (see Risks).

## Views

The seam is described as the steps and what crosses at each, the dependency direction, and who
reports each failure.

### What the driver does and what crosses

| Step | Owner of the call | Crosses in | Crosses out |
| --- | --- | --- | --- |
| 1 to 2. S1 to S4 and the E4 emitter | QSL (`qsl_replay::spine::compile`) | unit source, domain package documents, an empty `DependencyInput`, spine stage limits | in-process checked package, `quire.checked-package/v2` bytes, `package_id` |
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
`package_id` E4 emitted. That is the single canonical content-identity digest on this seam, and
it binds the admitted package to the bytes that were emitted.

### Dependency direction

- Normal and build dependencies: QSL layer crates, CG, IR's model crate `quire-contract-model`
  and `thiserror` (QSL ADR-011 section 6.1 driver row; asserted by `tests/dependency_edges.rs`,
  TC-005). Nothing depends on the driver.
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
  (IR-479, PR #12; see Open questions). How the first-party dependencies are spelled is the
  revision policy of IR-351 and IR-479, not decided here.

### Failure outcomes and who reports them

| Condition | Reported by | Kind |
| --- | --- | --- |
| A provider manifest does not register; a duplicate backend | driver (from `qsl_route` refusals) | refusal |
| A spine stage refuses the unit or the emitter fails | QSL, carried unchanged (`CompileRefusal`) | refusal; `stage_limit_exceeded` is a limit refusal |
| A registered backend has no CG kind | driver (`invalid_capability/unknown-backend`, ids in bytewise order, before negotiation) | refusal |
| Envelope refusal at negotiation | CG (`EnvelopeRefusal`) | refusal |
| IR refuses the bytes QSL just emitted, or admits them at another `package_id` | driver | internal fault: an upstream broke its contract |
| IR's reader stops at a limit | driver (`HandoffLimit`) | limit refusal |
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
  internal fault before conversion.
- D-5. Generation is called once, only for routed items, and not at all when nothing is routed.
- D-6. A routed item whose node is missing is an internal fault and CG is not called.
- D-7. The driver's normal dependencies are exactly the allow-list above, and the graph holds
  one copy of `quire-contract-model` (existing: TC-005; the one-copy assertion is added by
  IR-479, PR #12).
- D-8. The driver uses no pin, attestation or digest-evidence API (blocked on IR-479).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps (measured at the commit this AD was written against)

- The next two bullets describe the driver before IR-479 (PR #12, the catch-up to current QSL,
  CG and IR). Once it merges they no longer hold. The catch-up and the revision policy belong
  to IR-479 and IR-351 and are not repeated here.
- Before IR-479, E5 evidence in `src/handoff.rs` calls `insert_artifact_bytes` and `insert_artifact_digest`.
  At current IR `main`, `CheckedPackageEvidence` holds only domain package documents,
  dependency packages and supported features and has no such methods; the lock's raw-artifact
  digests are not checked against any evidence (`evidence.rs` doc in IR). The driver's evidence
  code uses the removed API. Its own doc says the digests it supplies are "QSL's own
  informational constants, never checked", and that `/diagnostics/catalog` staleness is vacuous:
  a check that cannot fail. After the catch-up the evidence is the unit's domain packages,
  supported features and any dependency packages.
- Before IR-479, the driver imports `KaniToolPins` and `AttestationContext` (`src/drive.rs`; `pinned()` is
  called at line 205), which CG no longer has on `main`. This AD does not describe them as
  design.
- The driver compiles with an empty `DependencyInput` and refuses any `import`
  (`missing_import/missing-selection`, FR-001 step 1). Known and stated, not a gap to fix here.
- The driver owns the execute, replay and terminal-record chain: run what CG generated, replay a
  counterexample through the `qsl-replay` facade, and settle the terminal record. Today it stops
  after generation: it does not run a harness, read a Kani outcome, call `replay`, map to a
  terminal value or write a terminal record, so the chain is unimplemented. QSL's ADR-011 T-13
  amendment (QSL-354, pending) records the ownership on QSL's side; ADR-011 as it stands still
  ends the driver at generation. The driver calls CG's terminal-value map, whose terminal step
  takes the Kani outcome and the replay settlement, and the `qsl-replay` facade, as the
  evidence-chain seam AD defines them. It adds no map or view of its own.
- The chain creates no cycle. The driver stays downstream of QSL and CG (ADR-011 FB-11) and
  nothing depends on it.

### Open questions

| Question | Owner | Recommendation | Cost of the alternative |
| --- | --- | --- | --- |
| Record the driver's ownership of execute, replay and the terminal record in QSL's ADR-011 T-13 | QSL (QSL-354) | amend T-13 | QSL and this AD disagree on the driver's role |
| The dependency-edge test's model-copy assertion | IR-479 (PR #12) | the test gates exactly one copy of `quire-contract-model`, as the catch-up lands it; the revision policy stays IR-351's and IR-479's | not decided here |
| Layout, requirements per stage, matrix and revision policy | IR-351 | not decided here | duplicate descriptions |

No compatibility layer is proposed. The driver changes its calls when an owner changes its API.

### Routed gaps

Needs stated to owners, not decisions. Ids are the routing ids of IR-324; they are not
requirement ids. Each matters to the driver only as stated.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R-Q1 | A `TerminalValue` for a non-vacuous inconclusive run; matters to the driver once it owns the execute step. |
| R-Q4 | A writer for the FR-331 `results` from a `TerminalRecord`; matters to the driver once it owns the terminal-record step. |
| R-Q6 | An emission-to-admission test in QSL over every node family and form QSL can emit; the driver's E5 step repeats that check at run time for one unit. |
