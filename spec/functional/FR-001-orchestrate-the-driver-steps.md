---
id: FR-001
title: "Orchestrate compile, backend conversion, negotiation and routing"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-011
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-012
    type: depends_on
---
# FR-001: Orchestrate compile, backend conversion, negotiation and routing

## Description

The driver SHALL run one request through these steps, in this order (QSL
ADR-011 T-13, ADR-012 §7.2):

1. S1 to S4 through `qsl_replay::spine::compile`.
2. E4: the `quire.checked-package/v2` bytes and `package_id` that the same
   call emits.
3. The pre-negotiation conversion. Every registered `BackendId` passes once
   through CG's one total conversion, `BackendKind::from_identity`.
4. The `route` candidate step for each requested item, then E7: CG
   `negotiate_backend_provider` over the FR-331 envelope built from the
   registry and the candidate sets.
5. The `route` routing step, after negotiation, over the items CG settled
   `supported`.
6. CG generation for each routed item, with its `BackendId`.

## Inputs

- The unit: source identity, display path, complete-V1 bytes, the FR-056
  domain package input and the spine stage limits.
- The registry value, built by the driver from provider manifests
  (`build_registry`) and passed as an argument.
- The requested items: each checked item's requirement record (node,
  capability kind, extent) and the backend the request names, if any.

## Outputs

- The in-process checked package, the emitted v2 package, CG's settlement
  of every item in request order, and each routed item with its
  `BackendId` and CG backend kind.
- Or one command-level refusal.

## Behavior

- When any registered `BackendId` has no CG kind, the driver SHALL refuse
  the run with `invalid_capability`/`unknown-backend`, naming every
  unconverted `BackendId` in bytewise order, before any negotiation.
- The driver SHALL call `route` only after negotiation, and only with the
  items settled `supported`. When no item settles `supported`, it SHALL
  not call `route`.
- Types cross between QSL and CG as their wire spellings (QSL ADR-013 T-7).
- Each command-level refusal SHALL carry its step, its catalog code and an
  outcome kind: refusal, limit refusal or internal fault. A provider
  manifest that does not register, a spine refusal, the unknown-backend
  refusal and an envelope refusal are refusals. A spine
  `stage_limit_exceeded` is a limit refusal. An upstream answer that breaks
  the driver's invariants is an internal fault.
- The driver's normal dependencies SHALL be the QSL layer crates and CG
  (QSL ADR-011 §6.1 driver row), and no crate SHALL depend on it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | The driver's dependency edges match the QSL ADR-011 §6.1 allow-list and §7.1 crate graph: its normal dependencies are QSL layer crates and CG, nothing depends on it, CG's one QSL edge is `qsl-replay`, and each QSL layer crate resolves to one revision. | Test (TC-005) |
| FR-001-AC-2 | A run whose registry holds a `BackendId` with no CG kind refuses before any negotiation, with `invalid_capability`/`unknown-backend` and the unconverted ids in bytewise order. | Test (TC-002) |
| FR-001-AC-3 | `route` is called only after negotiation, and only for `supported` items. | Test (TC-003) |
| FR-001-AC-4 | The steps run in the order of the Description: compile and emit, conversion, negotiation, routing. | Test (TC-001) |
| FR-001-AC-5 | Each command-level refusal carries its step, catalog code and outcome kind. | Test (TC-004) |

## Dependencies

- QSL ADR-011 §2.1, §6.1 driver row, §7.1 and T-13; ADR-012 §7.1 and §7.2.
- QSL `qsl-replay` (`spine::compile`) and `qsl-route` (registry, candidate
  step, routing step).
- CG `negotiate_backend_provider` and `BackendKind::from_identity`
  (agent-ix/quire-contract-codegen#86).

## Status

Steps 1 to 5 are implemented. Step 6 is not wired: CG has no generation
entry point that takes a routed `BackendId` or `BackendKind`. The Kani
generator, `negotiate_kani_obligations`, is Kani-only and negotiates its
own items again. `Driven::routed` is the input step 6 takes.

`qsl-package` does not yet expose the per-item requirement records of a
`CheckedPackage`, so the caller supplies the requested items.
