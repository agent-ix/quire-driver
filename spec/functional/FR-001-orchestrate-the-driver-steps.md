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
---
# FR-001: Orchestrate compile, backend conversion, negotiation, routing and generation

## Description

The driver SHALL run one request through these steps, in this order (QSL
ADR-011 T-13, ADR-012 §7.2):

1. S1 to S4 through `qsl_replay::spine::compile`.
2. E4: the `quire.checked-package/v2` bytes and `package_id` that the same
   call emits.
3. The `route` candidate step for each requested item.
4. The pre-negotiation conversion. Every registered `BackendId` passes once
   through CG's one total conversion, `BackendKind::from_identity`.
5. E7: CG `negotiate_backend_provider` over the FR-331 envelope built from
   the registry and the candidate sets.
6. The `route` routing step, after negotiation, over the items CG settled
   `supported`.
7. CG generation: one `generate_routed` call over the routed items, each
   with its request index, its checked node, its routed backend and its CG
   kind.

Between steps 2 and 3, the E5 hand-off: IR's checked-package reader
(`read_checked_package`) reads the emitted v2 bytes, and the driver accepts
the admitted package only at the `package_id` E4 emitted.

## Inputs

- The unit: source identity, display path, complete-V1 bytes, the FR-056
  domain package input and the spine stage limits.
- The registry value, built by the driver from provider manifests
  (`build_registry`) and passed as an argument.
- The requested items: each checked item's requirement record (node,
  capability kind, extent) and the backend the request names, if any.
- The generation input: one optional context per CG backend kind. The Kani
  context is the subject path, the unwind bound and the attestation
  binding. The tool pins are always `KaniToolPins::pinned()`. The caller
  supplies no claim map: CG derives each routed item's claim from the
  node's own operation and bound (IR-294).

## Outputs

- The in-process checked package, the emitted v2 package, IR's admitted
  `CheckedPackageV2`, CG's settlement of every item in request order, each
  routed item with its `BackendId` and CG backend kind, and CG's generation
  output in ascending request index. That output's `claim_map` is the
  FR-014 claim map CG derived for the Kani group, `Some` whenever one ran;
  a caller writes it as `claim-map.json`.
- Or one command-level refusal.

## Behavior

- When any registered `BackendId` has no CG kind, the driver SHALL refuse
  the run with `invalid_capability`/`unknown-backend`, naming every
  unconverted `BackendId` in bytewise order, before any negotiation.
- The driver SHALL call `route` only after negotiation, and only with the
  items settled `supported`. When no item settles `supported`, it SHALL
  not call `route`.
- Types cross between QSL and CG as their wire spellings (QSL ADR-013 T-7).
- Each routed item's node is the admitted package's `CheckedNodeId` whose
  digest spells the item's `WireNodeId`, found by lookup. A routed item
  whose node the package does not hold refuses the run with
  `dangling_reference` before CG is called.
- The driver SHALL call CG generation only after routing, only for routed
  items, and not at all when nothing is routed.
- A routed item whose kind has no context refuses the run with
  `invalid-request`. CG's other generation refusals map to `invalid-request`
  (a malformed Kani context), a limit refusal (`resource_exhausted`, the
  Kani item ceiling or an oversized generated source) or an internal fault
  (a kind disagreement, a duplicate request index, an empty or unpinned
  Kani request, a claim-map or location-map serialization failure, or an
  unknown Contract Runtime variant -- none of which the driver produces).
- A node whose operation has no derivable claim (CG's `NoDerivableClaim`)
  settles `Unsupported` with `no_derivable_claim`, naming the node and CG's
  derivation refusal, and carries no harness. It does not reject its
  siblings, and does not refuse the run.
- Each command-level refusal SHALL carry its step, its catalog code and an
  outcome kind: refusal, limit refusal or internal fault. A provider
  manifest that does not register, a spine refusal, the unknown-backend
  refusal and an envelope refusal are refusals. A spine
  `stage_limit_exceeded` and an IR reader limit are limit refusals. An
  upstream answer that breaks the driver's invariants is an internal fault;
  that includes IR refusing, or admitting at another `package_id`, the bytes
  QSL just emitted.
- The driver's normal and build dependencies SHALL be the QSL layer crates,
  CG, IR's model crate `quire-contract-model` (at the revision CG's IR crate
  uses) and `thiserror` (QSL ADR-011 §6.1 driver row), and nothing in its
  lock SHALL depend on it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | The driver's dependency edges match the QSL ADR-011 §6.1 allow-list and §7.1 crate graph: its normal and build dependencies are QSL layer crates, CG, `quire-contract-model` and `thiserror` (`serde_json` is a dev-dependency, not a normal one), the driver's `quire-contract-model` is the one CG's IR crate depends on, nothing in its lock depends on it, CG's one QSL edge is `qsl-replay`, and the driver's own QSL pin resolves the same `quire-contract-model` package id as the driver's own direct pin. Other copies of `quire-contract-model` and of QSL layer crates, reached only through CG's own (not yet caught up) `qsl-replay` edge or through IR's own root-crate edge, are informational and not gated. | Test (TC-005) |
| FR-001-AC-2 | A run whose registry holds a `BackendId` with no CG kind refuses before any negotiation, with `invalid_capability`/`unknown-backend` and the unconverted ids in bytewise order. | Test (TC-002) |
| FR-001-AC-3 | `route` is called only after negotiation, and only for `supported` items. | Test (TC-003) |
| FR-001-AC-4 | The steps run in the order of the Description: compile and emit, the E5 read, the candidate step, conversion, negotiation, routing, generation. | Test (TC-001) |
| FR-001-AC-5 | Each command-level refusal carries its step, catalog code and outcome kind. | Test (TC-004, TC-007) |
| FR-001-AC-6 | CG generation is called once, after routing, over the routed items only, each with its request index, its admitted `CheckedNodeId`, its routed backend and its kind; its output is keyed by request index, ascending; a missing kind context or an unknown node refuses the run. | Test (TC-006, TC-007) |
| FR-001-AC-7 | The E5 read admits the emitted v2 bytes at the emitted `package_id`; an admitted package at another `package_id`, or an IR refusal, is an internal fault at the hand-off before any backend conversion. | Test (TC-004, TC-006) |
| FR-001-AC-8 | CG generation's output carries the FR-014 claim map CG derived for the Kani group (IR-294), `Some` whenever one ran; a node whose operation has no derivable claim settles `Unsupported` with `no_derivable_claim` and does not affect its routed siblings' records or harnesses. | Test (TC-006) |

## Dependencies

- QSL ADR-011 §2.1, §6.1 driver row, §7.1 and T-13; ADR-012 §7.1 and §7.2.
- QSL `qsl-replay` (`spine::compile`) and `qsl-route` (registry, candidate
  step, routing step); `qsl_package::diagnostics_catalog()` (QSL-260, #453).
- CG `negotiate_backend_provider` and `BackendKind::from_identity`
  (agent-ix/quire-contract-codegen#86); CG `generate_routed` (CG FR-022,
  IR-293, IR-294, IR-298, IR-300).
- IR `quire-contract-model`: `read_checked_package`, `CheckedPackageV2`,
  `CheckedNodeId`.

## Status

Steps 1 to 7 and the E5 hand-off are implemented.

The Kani claim map is derived by CG from the package and the routed node
alone (IR-294); the caller supplies no claim map, and the driver surfaces
CG's derived claim map in its output.

E5's evidence is the unit's bytes, the caller's domain package documents,
QSL's pinned `DefinitionLock` digests and its root feature, and the
diagnostics catalog reference. The `DefinitionLock` digests are QSL's own
informational constants, never checked against the definition bytes. The
diagnostics catalog reference is `qsl_package::diagnostics_catalog()`
(QSL-260 closed by #453): the same reference QSL's own emitter pins at
`/diagnostics/catalog`, read from QSL's own API rather than parsed back out
of the emitted bytes. `serde_json` is now only a dev-dependency: this
crate's own tests use it to assert wire shapes (settlement dispositions,
the derived claim map) and to parse `cargo metadata`'s JSON
(`tests/dependency_edges.rs`).

CG's Kani arm derives each routed node's operand and result bound roles
from what the node itself names (IR-298): an operand's bound is the
`bounded_domain` typing it, and the result bound is the domain typing the
node's result, or the one reachable bound that is nobody's operand. On a
QSL-emitted package:

- A unary operation over one bounded parameter (e.g. `-z` over `z: Int[0,
  9]`) settles `Supported`, ranging its one argument over `z`'s own domain:
  `scalar_result_bound`'s single-candidate arm resolves the result role
  directly, so no own-bound filtering runs at all.
- A binary operation with one bounded operand and one that is not (e.g.
  `x + 1` over `x: Int[0, 9]`) settles `RequiresBound`, naming the
  unbounded operand's own type: QSL represents every literal as a
  reference to its own plain-typed value node, never as an inline literal
  term, so this is CG's "a reference operand typed by a plain scalar type
  beside one that owns a bound" rule, not the exempted inline-literal case.
- A binary operation with two independently bounded operands (e.g. `x + y`
  over `x: Int[0, 9]`, `y: Int[10, 20]`) settles `Unsupported`
  (`AmbiguousBound`), not `Supported`, through real QSL output today: QSL
  always wraps arithmetic in a separate narrowing `conversion` node when
  the declared context is bounded, and that conversion's own bound is not
  reachable from the arithmetic node itself (IR's `bounds` closure is
  forward-only, from the node's own body reference targets). So the node's
  only reachable candidates are its two operands' own bounds; removing
  both as "someone's own" leaves none, which CG reports as ambiguous
  rather than resolved. IR-298's own two-bounded-parameter case (its title)
  needs the arithmetic node's own result type to be directly the target
  bounded domain, unwrapped -- a shape CG's own synthetic test corpus
  builds directly but real QSL compilation does not appear to produce for
  a bounded-context arithmetic expression.

`tests/drive.rs` covers all three shapes
(`tc_001_tc_003_tc_006_steps_run_in_order_through_generation`,
`tc_006_a_literal_beside_an_owned_bound_requires_bound`,
`tc_006_two_independently_bounded_operands_is_ambiguous_not_supported`).
The driver passes CG's output through unchanged in every case.

CG at IR-300 (#180) also returns `RoutedGeneration::oracle_artifacts`, the
FR-014 crate the Kani arm generated (`Cargo.toml`, `src/lib.rs` and
`claim-map.json`). The driver does not yet write these out: Kani execution
is blocked on IR-301. `Driven::generation` still carries them unchanged for
a future caller.

`qsl_package::CheckedPackage::graph().requirements()` (#454, QSL-258) is
not yet consumed here. Two things stand in the way of routing an item from
it alone: `ValueFunctionFamily`, the only family QSL ships today, requests
no FR-057 capability kind for a function declaration (FR-062-AC-4), so the
map is empty for every unit this crate's tests compile; and even populated,
it is keyed by checked *declaration* occurrence, not by the arbitrary body
node (an expression like `+` or `=`) a Kani item names here. So the caller
still supplies the requested items (`RequestedItem`), each with the node it
names directly; this goes when a family requests capability kinds over the
nodes a Kani item can actually name (QSL-42, QSL-43 and beyond).
