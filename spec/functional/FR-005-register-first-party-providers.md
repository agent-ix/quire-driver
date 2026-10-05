---
id: FR-005
title: "Register first-party providers through one typed interface"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-288
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-075
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-002
    type: depends_on
---
# FR-005: Register first-party providers through one typed interface

## Description

The driver SHALL expose one `Provider` trait in its library for each first-party
provider linked into the binary. The trait SHALL expose the provider's FR-331
manifest and run the routed request with the emitted package, provider limits
and cancellation handle, returning typed provider results. The driver SHALL
register each provider through its manifest and SHALL pass the resulting
registry to the lifecycle operation that uses it.

## Inputs

- A first-party provider implementation.
- Its FR-331 `ProviderManifest`.
- The emitted package, routed `ProviderRequest`, `ProviderLimits` and
  `Cancel` supplied for one run.

## Outputs

- The provider manifest for registration.
- Typed `ProviderResults` for the routed request.

## Behavior

- The trait SHALL provide `manifest(&self) -> ProviderManifest` and
  `run(&self, package: &EmittedPackage, request: &ProviderRequest, limits:
  &ProviderLimits, cancel: &Cancel) -> ProviderResults`.
- The driver SHALL register the manifest using the same manifest conversion
  used to build the backend registry.
- The driver SHALL derive backend identity from the manifest identity.
- The driver SHALL pass the request's emitted package, routed items, limits
  and cancellation handle to the selected provider without altering them.
- The driver SHALL preserve provider results as typed values without
  inferring them from provider diagnostic text.
- The driver SHALL register linked providers and process-provider manifests
  through `qsl_route::Registry`. The QSL registry builder assigns origin
  `Linked` to compile-time providers and `Process` to plugin hello descriptors;
  the driver passes resulting descriptors to CG without constructing CG
  backend kinds. It SHALL preserve the registry's typed duplicate-backend
  refusal and withdrawn-identity outcome; it SHALL NOT implement duplicate
  policy itself. Equal descriptors are idempotent and an unequal descriptor
  for an existing `BackendId` withdraws that identity (QSL FR-075-AC-4/AC-7,
  FR-290-AC-9/10 and QSL-637).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | A compile-time provider implementing `Provider` returns its FR-331 manifest; the QSL registry builder assigns origin `Linked` to the registered descriptor and retains the manifest identity. | Test (TC-012; `src/registry.rs` asserts the Linked origin; Process origin remains planned under FR-005-AC-5) |
| FR-005-AC-2 | The provider receives the same emitted package, routed request, limits and cancellation handle supplied by the driver, and its typed results are returned without semantic changes. | Test (TC-012) |
| FR-005-AC-3 | Two providers with distinct manifest identities are registered as distinct backends; a run invokes only the provider selected for its routed items. | Test (TC-012) |
| FR-005-AC-4 | The driver preserves QSL registry results: an equal descriptor repeat is idempotent; an unequal descriptor for an existing `BackendId`, including one with a different origin, returns the typed duplicate-backend refusal, withdraws that identity, and invokes neither provider. | Partial (TC-012 checks equal-manifest idempotence; cross-origin provider integration follows CG IR-633) |
| FR-005-AC-5 | The driver supplies plugin hello descriptors to the QSL registry builder, which assigns origin `Process`; the driver passes resulting descriptors unchanged to CG negotiation, and CG maps that origin to its process backend kind. | Planned (plugin host and CG IR-633) |

The current linked Kani adapter is the only Provider execution adapter. Its
`ProviderLimits` and result carriers are Kani-specific; they do not yet
generalize process or other QSL engines. Cancellation is checked between
routed items. CG's current `execute_kani_obligation` API accepts no
cancellation handle, so an already-running harness continues until CG's
configured timeout. Process-origin adoption and the single origin-aware
registry integration are follow-up work; this slice remains Linked-only.

## Dependencies

- QSL ADR-029 PV-1 and PV-3 (the current driver carriers are Kani-only);
  QSL FR-075 and FR-288 for registry identity and provider-manifest
  registration.
- FR-006 defines the separate out-of-process plugin host boundary.
- [TC-012](../test-cases/TC-012-first-party-provider-interface.md).
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point that registers and invokes providers.
