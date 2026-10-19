---
id: FR-001
title: "Orchestrate compile, backend conversion, negotiation, routing and generation"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-codegen/FR-019
    type: depends_on
---
# FR-001: Orchestrate compile, backend conversion, negotiation, routing and generation

## Description

The driver SHALL run one request through these steps, in this order (QSL
ADR-011 T-13, ADR-012 §7.2):

1. S1 to S4 through QSL's typed `parse`, `select` and `check` operations.
   These operations and step 2 share one `Cancel` handle. `check` receives an
   empty `DependencyInput` (ADR-015 D-1): the driver does not yet support
   compiling a unit's `import`s against caller-supplied library source, so
   any `import` refuses at the intake stage,
   `missing_import`/`missing-selection`.
2. E4: QSL's `package` operation emits the `quire.checked-package/v2`
   bytes and `package_id` from the checked unit.
3. The `route` candidate step for each requested item.
4. The driver projects each registry descriptor with CG's
   `BackendKind::from_descriptor`. CG classifies Process-origin descriptors
   from their admitted origin; Linked descriptors still require a recognized
   Linked identity. The driver does not construct a backend kind or infer
   origin from provider bytes.
5. E7: CG `negotiate_backend_provider` over the FR-331 envelope built from
   the registry and the candidate sets.
6. The `route` routing step, after negotiation, over the items CG settled
   `supported`.
7. CG generation: one `generate_routed` call over the routed items, each
   with its request index, its checked node, its routed backend and its CG
   kind.

After the E5 hand-off and before step 3, the driver calls QSL's layer-4
bindings export (QSL FR-307) for the requested items and partitions them
into bound and refused (FR-010).

Between steps 2 and 3, the E5 hand-off: IR's checked-package reader
(`read_checked_package`) reads the emitted v2 bytes, and the driver accepts
the admitted package only at the `package_id` E4 emitted.

## Inputs

- The unit: source identity, display path, complete-V1 bytes, the FR-056
  domain package input and the spine stage limits.
- The registry value, built by the driver from provider manifests
  (`build_registry`) and passed as an argument.
- The backend the request names, if any (`named`). The requested items
  themselves are not a caller input: they are written from the compiled
  unit's own checked requirement records
  (`qsl_package::CheckedPackage::graph().requirements()`, keyed one per
  scalar operation application -- QSL-266), one item per record in
  occurrence-key order (`qsl_route::request::items_from_requirements`,
  FR-075). A unit with no scalar operation application writes no requirement
  record, so the driver routes nothing for it.
- The generation input: one optional context per CG backend kind. The Kani
  context is the subject path and the unwind bound. The caller
  supplies no claim map: CG derives each routed item's claim from the
  node's own operation and bound (IR-294).
- A caller-owned, cloneable QSL `Cancel` handle. The driver passes the same
  handle through `parse`, `select`, `check` and `package`; the caller may
  cancel it from another thread while a stage runs. QSL owns polling and
  returns the typed cancellation failure.
- For scalar proof generation, the caller's existing typed QSL `Call`, its
  `ObjectEnvironment`, and the same caller-owned `Cancel` handle, supplied
  with the compile input as one driver-library operation. This is an internal
  library composition input; it adds no CLI syntax. The driver does not accept
  a second caller-built package for this call.

## Outputs

- The in-process `CheckedUnit`, the emitted v2 package, IR's admitted
  `CheckedPackageV2`, CG's settlement of every item in the CG request, in request order, each
  routed item with its `BackendId` and CG backend kind, and CG's generation
  output in ascending request index. That output's `claim_map` is the
  FR-014 claim map CG derived for the Kani group, `Some` whenever one ran;
  a caller writes it as `claim-map.json`.
- `Driven` also carries a per-item refusal member: for each requested item
  the export refused, the item's occurrence key and the export's typed
  refusal unchanged, in driver request order. The member is empty when every
  item is bound. A partly bound package therefore returns its settled items
  and each refused item's refusal in one completed result.
- The two halves of `Driven` join on the item's original driver request
  index. Settlements and generation output stay keyed by that index, so an
  omitted refused item leaves a gap and C keeps driver index 2 when B (index
  1) is omitted; the refusal member carries the occurrence key and that same
  driver index. The FR-330 position vector keeps one `DriverItem` per
  original request item (FR-001-AC-15). A refused item's position is
  accounting-only: it carries no admitted node and no claim, and CG receives
  nothing from it that it can lower, route or generate.
- Or one command-level refusal.
- Or a `DriverRefusal::Cancelled` result carrying the cancelled QSL stage and
  cause.
- `Driven` retains the checked unit, emitted v2 package bytes, admitted
  package and downstream results. The checked unit retains its root source
  and resolved library sources so callers can resolve diagnostic loci without
  reacquiring source data. E5 consumes the separate E4 `EmittedUnit` source
  provision; the driver releases that provision after hand-off. Under this
  requirement's empty `DependencyInput`, no library source is admitted, and
  the retained root source is bounded by the caller's
  `SpineLimits.source.source_bytes` setting.

## Behavior

- When any registered `BackendId` has no CG kind, the driver SHALL refuse
  the run with `invalid_capability`/`unknown-backend`, naming every
  unconverted `BackendId` in bytewise order, before any negotiation.
- The driver SHALL call `route` only after negotiation, and only with the
  items settled `supported`. When no item settles `supported`, it SHALL
  not call `route`.
- Types cross between QSL and CG as their wire spellings (QSL ADR-013 T-7).
- For every request item, the driver SHALL project the typed IR tag/form
  from the same admitted checked package at the E5 `package_id`, requirement
  record and node named by that item's QSL record, matched by occurrence key.
  It SHALL pass the form as CG's typed in-process negotiation input and
  preserve the checked form's typed identity and structure. The driver SHALL
  NOT accept a caller-computed lowerability result or refusal, classify the
  form, or infer it from display or debug text.
- Each routed item's node is the admitted package's `CheckedNodeId` whose
  digest spells the item's `WireNodeId`, found by lookup. Every item's node
  comes from the same compiled package's own
  `CheckedGraph::requirements()`, and `checked` is IR's own admission of
  the bytes that same package emitted, at the same `package_id`, so this
  lookup cannot miss on any input the driver itself builds a request from.
  A routed item whose node the package does not hold is not a caller
  refusal: it is an internal fault (`runtime_invariant`), reachable only
  through an E4/E5 inconsistency (E4 emitted, or E5 admitted, a graph
  missing a node QSL's own requirement record names), and the run stops
  before CG is called.
- The driver SHALL call CG generation only after routing, only for routed
  items, and not at all when nothing is routed.
- For scalar proof generation, the driver library SHALL accept the caller's
  exact QSL `Call`, `ObjectEnvironment`, and `Cancel` with the compile input
  as one production composition operation. This operation SHALL use the QSL
  compiled/emitted package and the IR-admitted view of those same emitted
  bytes. It SHALL NOT rebuild a call or package, re-emit a package for
  binding, or admit a second package.
- For each routed scalar item selected for native proof, the driver SHALL join
  the QSL `RequirementItem` to the corresponding CG driver item by its
  `request_index` and exact QSL `OccurrenceKey`. It SHALL call QSL's
  `bind_scalar_operands` with that same request's compiled/emitted package,
  unchanged `Call`, the selected item's exact function/application/occurrence,
  `ObjectEnvironment`, and `Cancel`. It SHALL retain QSL's typed binding
  result and ordered bound operands in a driver-owned per-item context keyed
  by `request_index`. Operand values SHALL follow QSL's bound operand order,
  which is the corresponding CG `ScalarObligationIdentity.arguments` order;
  operand source and ordinal data remain attached to those values. The
  `ScalarLimits` retained for later native execution SHALL come from that
  same `Call`'s accounting. The driver SHALL NOT zip argument and operand
  vectors by position, select an occurrence by node ID alone, or take limits
  from another call.
- The driver SHALL construct CG's complete request-position vector in the
  admitted FR-330 accounting order. Each original driver request item
  the export bound contributes one CG `DriverItem` position with its exact
  admitted node and scalar result selection, whether or not routing later
  selects it; an export-refused item contributes an accounting-only position
  at its driver index with no node, claim or result selection; a direct
  conversion occurrence selected by that QSL item
  contributes its own `ContextRoot` position. A `ContextRoot` is not a routed
  item and SHALL create no backend settlement, claim, or output item. A QSL
  item with no conversion context uses no conversion root; the driver SHALL
  not choose a different context. The position's driver index counts only
  `DriverItem` entries; context roots do not shift request indices. CG
  receives only its admitted `ContractPackageV1`, complete position vector,
  routed items, and generation contexts. The QSL `Call` and bound operands
  remain in the driver's per-item context and are not added to CG's API.
- When no scalar item is selected for routed native proof, the driver SHALL
  skip scalar binding and native execution. It SHALL select no fallback item
  or synthesize a scalar request.
- When QSL binding reports a missing or ambiguous call/occurrence as a typed
  `RunRefusal` or `BindRefusal`, the driver SHALL preserve that refusal
  unchanged. It SHALL stop the scalar generation composition before native
  execution.
- The driver SHALL preserve the owning QSL/IR/CG typed refusal when a required
  conversion context is missing or ambiguous.
- The driver SHALL treat `None` as a valid conversion context only when that
  exact QSL item has no conversion context; the driver SHALL add no conversion
  root in that case.
- When the admitted package, request position, or routed item disagree, the
  driver SHALL preserve the owning IR/CG typed refusal. A driver-detected join
  invariant failure SHALL be a typed `runtime_invariant` fault before native
  execution or replay. No path may guess a call, occurrence, context root, or
  operand order.
- A routed item whose kind has no context refuses the run with
  `invalid-request`. CG's other generation refusals map to `invalid-request`
  (a malformed Kani context), a limit refusal (`resource_exhausted`, the
  Kani item ceiling or an oversized generated source) or an internal fault
  (a kind disagreement, a duplicate request index, a duplicate harness,
  a Kani record-count mismatch, a Kani duplicate position out of range,
  an empty Kani request, a claim-map or location-map serialization failure, or an
  unknown Contract Runtime variant -- none of which the driver produces).
- A node whose operation has no derivable claim (CG's `NoDerivableClaim`)
  settles `Unsupported` with `no_derivable_claim`, naming the node and CG's
  derivation refusal, and carries no harness. It does not reject its
  siblings, and does not refuse the run.
- The per-item refusal member SHALL hold only an item the export refused. It
  holds no negotiation settlement (`invalid-request`, `unknown-backend`,
  `NoDerivableClaim`) and no scalar `RunRefusal` or `BindRefusal`;
  FR-001-AC-16 still stops the composition for the latter.
- When the export refuses a requested item, the driver SHALL record the
  refusal in the member.
- When the export refuses a requested item, the driver SHALL NOT return a
  `DriverRefusal` for it.
- When the export refuses every requested item, the driver SHALL complete the
  run with no settled item and one per-item refusal for each requested item.
- When the export fails as a whole, the driver SHALL refuse the run with the
  export's typed refusal, carrying the bindings-export step.
- When the export reports that an occurrence key the driver built names no
  requirement record, the driver SHALL treat it as a `runtime_invariant`
  internal fault at the hand-off.
- When the package declares no abstraction relation, the driver SHALL skip
  the export, send every item to CG as FR-001 otherwise states, and leave the
  per-item refusal member empty.
- Each command-level refusal SHALL carry its step, its catalog code and an
  outcome kind: refusal, limit refusal or internal fault. A provider
  manifest that does not register, a spine refusal, the unknown-backend
  refusal, an envelope refusal and an export that fails as a whole are
  refusals. A spine
  `stage_limit_exceeded` and an IR reader limit are limit refusals. An
  upstream answer that breaks the driver's invariants is an internal fault;
  that includes IR refusing, or admitting at another `package_id`, the bytes
  QSL just emitted, and a routed item naming a node the checked package does
  not hold.
- `build_registry` registers every manifest first, then reads
  `Registry::refusals()` once: one duplicate-backend refusal per distinct
  conflicting digest, not one per conflicting `register` call. Repeating an
  already-conflicting digest (manifests A, B, then A again) does not add a
  third refusal for it.
- A cancellation failure from any QSL stage propagates as a typed result.
  QSL owns its category and exit function (ADR-029 CB-4); the driver defines
  no cancellation-specific category or exit mapping.
- When E5 consumes the E4 `EmittedUnit` source provision, the driver SHALL
  release that E4 provision after hand-off.
- The driver SHALL retain the `CheckedUnit` root source for diagnostic loci.
  Under the current empty `DependencyInput`, it retains no library source,
  and its retained root-source bytes do not exceed
  `SpineLimits.source.source_bytes`.
- The driver's normal and build dependencies SHALL be exactly
  `qsl-foundation`, `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,
  `quire-exact`, `quire-contract-codegen`, `quire-contract-model`,
  `quire-canonical`, `serde`, `serde_json` and `thiserror`. The direct
  `serde_json` edge is required to project FR-331's optional `bounds` member
  from the retained caller wire into CG's typed descriptor; the driver
  preserves the wire snapshot and does not define a parallel FR-331 schema.
  The cache boundary is specified separately in FR-001-AC-11 and FR-012.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | The driver's normal and build dependency set matches QSL ADR-011 §6.1 and the specified driver-layer exceptions: it contains exactly `qsl-foundation`, `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`, `quire-exact`, `quire-contract-codegen`, `quire-contract-model`, `quire-canonical`, `serde`, `serde_json` and `thiserror`; `serde_json` is required for production projection of FR-331's retained optional `bounds` member. The driver's `quire-contract-model` is the one CG's IR crate depends on, CG's one QSL edge is `qsl-replay`, and the lock holds exactly one `quire-contract-model` package, so the driver's direct edge, CG's IR crate and QSL's `qsl-package` all resolve the same package id. | Test (TC-005) |
| FR-001-AC-2 | A run whose registry holds a `BackendId` with no CG kind refuses before any negotiation, with `invalid_capability`/`unknown-backend` and the unconverted ids in bytewise order. | Test (TC-002) |
| FR-001-AC-3 | `route` is called only after negotiation, and only for `supported` items. | Test (TC-003) |
| FR-001-AC-4 | The steps run in the order of the Description: compile and emit, the E5 read, the candidate step, conversion, negotiation, routing, generation. | Test (TC-001) |
| FR-001-AC-5 | Each command-level refusal carries its step, catalog code and outcome kind. A `Compile` refusal's code renders `code/cause` when the refusing stage names a cause (e.g. `missing_import`/`missing-selection`), not just the bare code. | Test (TC-004, TC-007, TC-008) |
| FR-001-AC-6 | CG generation is called once, after routing, over the routed items only, each with its request index, its admitted `CheckedNodeId`, its routed backend and its kind; its output is keyed by request index, ascending; a missing kind context refuses the run, and a routed item naming a node the checked package does not hold is an internal fault. | Test (TC-006, TC-007) |
| FR-001-AC-7 | The E5 read receives every caller-supplied FR-056 domain package document under its supplied digest and bytes unchanged, then admits the emitted v2 bytes at the emitted `package_id`; an admitted package at another `package_id`, or an IR refusal, is an internal fault at the hand-off before any backend conversion. | Test (TC-004, TC-006) |
| FR-001-AC-8 | CG generation's output carries the FR-014 claim map CG derived for the Kani group (IR-294), `Some` whenever one ran; a node whose operation has no derivable claim settles `Unsupported` with `no_derivable_claim` and does not affect its routed siblings' records or harnesses. | Test (TC-006) |
| FR-001-AC-9 | The same caller-owned `Cancel` handle is passed to QSL `parse`, `select`, `check` and `package`; cancellation returns `DriverRefusal::Cancelled` with QSL's stage and cause, and the driver adds no category or exit mapping. | Test (TC-019) |
| FR-001-AC-10 | E5 receives the E4 `EmittedUnit` source provision; the returned `Driven` contains no `EmittedUnit`, retains the `CheckedUnit` root source for diagnostic loci, retains no library source under the current empty `DependencyInput`, and retains no more root-source bytes than `SpineLimits.source.source_bytes`. | Test (TC-019) |
| FR-001-AC-11 | The normal/build dependency closure rooted at `quire-driver` contains no `quire-cache` package. | Test (TC-005) |
| FR-001-AC-12 | For each request item, the driver's CG negotiation input carries the typed IR tag/form projected from the admitted checked requirement and node named by that same item's QSL requirement record; generation preserves that item's exact application occurrence and direct conversion-context occurrence as ordered lowering roots, including when several items share one application node. No caller-computed lowerability/refusal or display/debug-derived form substitutes for the checked form. | Test (TC-006) |
| FR-001-AC-13 | The driver's internal scalar-generation composition receives the caller's unchanged QSL `Call`, `ObjectEnvironment`, and `Cancel` with the request's compile input. QSL binding and IR admission use the package produced by that same request: the exact emitted bytes are admitted once and are not reconstructed or re-admitted. The generated result remains associated with that call and admitted package in the driver library. | Test (TC-006) |
| FR-001-AC-14 | For each routed scalar item selected for native proof, the driver joins the QSL item and CG driver item by request index and exact `OccurrenceKey`, then uses QSL `bind_scalar_operands` on that call's package and exact function/application/occurrence. Bound operands remain in QSL order, matching the CG scalar identity's argument order, with their argument source and ordinal; `ScalarLimits` come from the same call's accounting. Two occurrences on one node remain distinct. No positional zip or node-only occurrence selection is permitted. | Test (TC-006) |
| FR-001-AC-15 | CG receives only its existing admitted `ContractPackageV1`, complete FR-330 position vector, routed items, and generation contexts. The position vector retains one position per original request item, in driver-index order (a `DriverItem` for each bound item, an accounting-only position for each export-refused item), and a separate `ContextRoot` for each selected direct conversion occurrence; context roots create no route, claim, or generated item and do not advance driver indices. The exact QSL call and bound operands remain in a driver-owned sidecar keyed by request index. | Test (TC-006) |
| FR-001-AC-16 | No selected scalar item means no scalar binding or native execution and no fallback selection. QSL's typed missing/ambiguous call or occurrence refusal is preserved unchanged. A missing/ambiguous required conversion context preserves its owning QSL/IR/CG typed refusal; `None` is valid only when the exact QSL item has no conversion context. An admitted-package/request-position/routed-item mismatch preserves the owning IR/CG typed refusal, or is a driver `runtime_invariant` fault when the driver detects an inconsistent join. Each case stops before native execution or replay and invents no call, occurrence, context root, or operand order. | Test (TC-006) |
| FR-001-AC-17 | For the FR-010-AC-4 vector, the run completes, `Driven` holds the settlements of A (driver index 0) and C (driver index 2) and B's refusal, unchanged, with its occurrence key and driver index 1 in the per-item refusal member, and no `DriverRefusal` is returned. | Test (TC-017) |
| FR-001-AC-18 | When the export refuses every requested item, `Driven` holds no settled item and one refusal in the per-item refusal member for each requested item. | Test (TC-017) |
| FR-001-AC-19 | An export that fails as a whole refuses the run with the export's typed refusal, and the refusal carries the bindings-export step. | Test (TC-017) |
| FR-001-AC-20 | An occurrence key the driver built that the export reports as naming no requirement record is a `runtime_invariant` internal fault. | Test (TC-017) |
| FR-001-AC-21 | When the package declares no abstraction relation, the driver does not call the export, every item goes to CG, and the per-item refusal member is empty. | Test (TC-017) |

## Dependencies

- QSL ADR-011 §2.1, §6.1 driver row, §7.1 and T-13; ADR-012 §7.1 and §7.2.
- QSL `qsl-replay` (typed spine lifecycle) and `qsl-route` (registry, candidate
  step, routing step).
- CG `BackendKind::from_descriptor`,
  `negotiate_backend_provider` and `generate_routed` (CG FR-022,
  FR-330, IR-293, IR-294, IR-298, IR-300, IR-302).
- QSL FR-367's `bind_scalar_operands` contract for binding the admitted call's
  named values to one exact scalar application occurrence; IR FR-049's
  context-only lowering roots; and CG's typed `ContractRequestPosition`
  accounting. These APIs retain their owning QSL, IR and CG semantics. The
  driver owns only the request-index/occurrence join and the per-item context.
- IR `quire-contract-model`: `read_checked_package`, `CheckedPackageV2`,
  `CheckedNodeId`.
- [CG FR-019](ix://agent-ix/quire-contract-codegen/FR-019) defines the typed
  IR tag/form negotiation input, refusal ownership and classification order;
  [CG TC-030](ix://agent-ix/quire-contract-codegen/TC-030) names the implemented
  and planned form selectors. The driver projects the checked IR value and
  does not implement CG's classifier.
- QSL ADR-029 LC-3 and CB-4, and QSL FR-276 cancellation semantics.

## Status

Steps 1 to 7 and the E5 hand-off are implemented. The scalar call-to-routed-
generation composition in FR-001-AC-13 through FR-001-AC-16 remains planned;
it is a driver-library boundary, not a CLI extension or a QSL/CG API change.

The accounting-only position for an export-refused item depends on CG: CG's
`ContractRequestPosition` holds only `DriverItem`, which requires an exact
root `node_id` that CG checks against a feature-accounting record, and
`ContextRoot`. CG must define a position that holds no node before the driver
can build it.

The Kani claim map is derived by CG from the package and the routed node
alone (IR-294); the caller supplies no claim map, and the driver surfaces
CG's derived claim map in its output.

E5's evidence is the caller's domain package documents and the root feature
of QSL's pinned `DefinitionLock`. The driver supplies no artifact-byte or
artifact-digest evidence: IR no longer checks staleness against artifact
digests. Production `serde_json` use reads the optional FR-331 `bounds`
member from the retained manifest wire for CG projection; tests also use it
to assert wire shapes and parse `cargo metadata`'s JSON.

CG's Kani arm derives each routed node's operand and result bound roles
from what the node itself names (IR-298, IR-302): an operand's bound is
the `bounded_domain` typing it; a `reference` operand whose target is a
`value` node with a literal body is a literal operand instead, owning no
bound of its own (constrained to exactly its own value); and the result
bound is the domain typing the node's result, the one `bounded_domain` a
narrowing `conversion` alone narrows the node to (QSL's own shape for an
expression in a bounded context: the node's own type is the plain scalar
type, and the declared bound types a separate `conversion` node that
consumes it), or the one reachable bound that is nobody's operand. On a
QSL-emitted package:

- A unary operation over one bounded parameter (e.g. `-z` over `z: Int[0,
  9]`) settles `Supported`, ranging its one argument over `z`'s own domain.
- A binary operation with one bounded operand and one literal (e.g.
  `x + 1` over `x: Int[0, 9]`) settles `Supported`: the literal is
  constrained to exactly its own value, and `x`'s own domain ranges the
  other argument.
- A binary operation with two independently bounded operands, narrowed by
  a `conversion` to the declared result bound (e.g. `x + y` over
  `x: Int[0, 9]`, `y: Int[10, 20]`, narrowed to `Int[10, 29]`) settles
  `Supported`, ranging each argument over its own operand's domain.
- A literal that pushes every exact result outside the declared bound
  settles `Unsupported`/`ResultBoundUnreachable`, naming the reachable and
  declared ranges: no harness is rendered, since no argument assignment
  gives a result a harness could assume. Directly in the return context
  (e.g. `x + 100` into `Int[0, 10]` over `x: Int[0, 9]`), QSL's own checker
  proves the identical disjoint-interval fact first and refuses to compile
  the unit at all (`Unproved`/`unproved-range`, at the check stage), so CG
  is never called for that shape. That is one shape, not a general
  guarantee: CG's own reachability computation ignores path conditions
  (below), so the identical refusal is reached whenever the same
  arithmetic sits behind a branch QSL's checker proves dead -- `if x > 20
  then x + 100 else 0` compiles, because QSL never discharges a coercion
  inside a branch it proves unreachable, and the `+` still settles
  `Unsupported`/`ResultBoundUnreachable` when routed.

Real limits remain, all refusals rather than internal faults:

- **Path conditions are ignored, until IR-304.** CG's reachability
  computation ranges every operand over its full declared domain,
  regardless of the `if` guard that reaches it: a dead branch's arithmetic
  is checked as if it always ran (above), and a live branch fares no
  better -- `if x > 5 then x + 100 else 0` into `Int[0, 200]` settles
  `Supported`, but ranges `x` over its full `[0, 9]` domain rather than the
  guard's own `[6, 9]` (harness args `[(0, 9), (100, 100)]`, not `x`
  narrowed by the guard).
- **`RequiresBound` is not confined to a plain-`Integer` parameter.** Any
  operand that is neither bounded nor a literal reaches it, including a
  nested subexpression and a negative literal (QSL emits `-1` as a negate
  node, not an inline literal): the outer `+` of `(x + 1) + 1` over
  `Int[0, 9]` into `Int[0, 20]`; both the `*` and the negate of `x * -1`
  over `Int[1, 9]` into `Int[-9, -1]`; and the negate of `-(x + 1)`.
- A node narrowed by more than one distinct bound, or consumed by both a
  narrowing conversion and any other node, settles `Unsupported`
  (`AmbiguousBound`), until IR-304. Several narrowing conversions to the
  *same* bound dedupe and settle `Supported`; only distinct bounds are
  ambiguous.

`tests/drive.rs` covers these shapes:
`tc_001_tc_006_steps_run_in_order_through_generation` (the unary case),
`tc_006_x_plus_1_settles_supported`,
`tc_006_x_plus_y_settles_supported_with_independent_argument_bounds`,
`tc_006_x_plus_100_in_the_return_context_is_refused_at_check` and
`tc_006_x_plus_100_in_a_dead_branch_settles_result_bound_unreachable` (which
also asserts the dead branch's own `if` guard, a second routed item),
alongside the unconditional pass-through checks
(`tc_006_x_plus_1_generation_passes_through_unchanged`,
`tc_006_x_plus_y_generation_passes_through_unchanged`). The driver passes
CG's output through unchanged in every case.

CG at IR-300 (#180) also returns `RoutedGeneration::oracle_artifacts`, the
FR-014 crate the Kani arm generated (`Cargo.toml`, `src/lib.rs` and
`claim-map.json`). The driver does not yet write these out: Kani execution
is blocked on IR-301. `Driven::generation` still carries them unchanged for
a future caller.

`qsl_package::CheckedPackage::graph().requirements()` (#454, QSL-258; keyed
per scalar operation application by QSL-266, #459) is now the sole source
of the driver's requested items: `qsl_route::request::items_from_requirements`
writes one item per record, in occurrence-key order, each already carrying
the node it names, its capability kind, its extent and its candidate outcome
against the registry for the backend the request names (if any). The driver
no longer accepts a caller-supplied item list (`RequestedItem` is removed);
the only caller inputs left to name a request are the registry and the
named backend. A unit with no scalar operation application (FR-062, ADR-012
§2 "Operation application claims") writes no requirement record, so the
driver calls neither `route` nor CG generation for it.
