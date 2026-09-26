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
  uses), `serde_json` and `thiserror` (QSL ADR-011 §6.1 driver row), and
  nothing in its lock SHALL depend on it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | The driver's dependency edges match the QSL ADR-011 §6.1 allow-list and §7.1 crate graph: its normal and build dependencies are QSL layer crates, CG, `quire-contract-model`, `serde_json` and `thiserror`, the driver's `quire-contract-model` is the one CG's IR crate depends on, nothing in its lock depends on it, CG's one QSL edge is `qsl-replay`, and the driver's QSL crates and CG's `qsl-replay` share one QSL revision. Other copies of `quire-contract-model` and of QSL layer crates, reached only through CG's IR edge or QSL's own edges, are informational and not gated. | Test (TC-005) |
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
  step, routing step).
- CG `negotiate_backend_provider` and `BackendKind::from_identity`
  (agent-ix/quire-contract-codegen#86); CG `generate_routed` (CG FR-022,
  IR-293, IR-294).
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
informational constants, never checked against the definition bytes. QSL
pins the diagnostics catalog reference inside its emitter and exports no
accessor (QSL-260), so the driver reads it back from the emitted wire, and
IR's `/diagnostics/catalog` staleness check is vacuous until QSL exports
it. The read-back and the normal `serde_json` dependency go then.

On a QSL-emitted package, CG's Kani arm settles the `+` of
`x + 1` (`x: Int[0, 9]`) `Supported` with a scalar harness: QSL-261 is
resolved by IR-296 and IR-297. IR-296 makes `require_bounds` stop treating a
type reached only as a literal's `type` annotation as the closure's bound --
here, the unbounded type of the literal `1` -- so it no longer masks `x`'s
own `[0, 9]` bound, which was always reachable. IR-297 makes CG's
`exact_scalar` follow `bounded_domain` to its base and read bounds written
as bindings. So the tests' generation output is a record with a harness,
and TC-006 asserts exactly that. The driver passes CG's output through
unchanged.

A unit with two bounded parameters still settles `Unsupported` (oracle
refusal `AmbiguousBound`; IR-298, IR-299), a known limit this crate's tests
do not exercise.

`qsl-package` does not yet expose the per-item requirement records of a
`CheckedPackage`, so the caller supplies the requested items (QSL-258).
