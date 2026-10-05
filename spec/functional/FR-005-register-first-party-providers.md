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
- The driver SHALL apply QSL registry duplicate rules to linked providers and
  process-provider manifests. An identical identity/digest pair is idempotent;
  conflicting manifests for one `BackendId` withdraw that identity, leaving
  its items declined rather than invoking either provider (QSL FR-075-AC-2,
  FR-290-AC-9/10 and QSL-637).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | A first-party provider implementing `Provider` returns its FR-331 manifest, which the driver registers with the backend identity equal to the manifest identity. | Test (TC-012) |
| FR-005-AC-2 | The provider receives the same emitted package, routed request, limits and cancellation handle supplied by the driver, and its typed results are returned without semantic changes. | Test (TC-012) |
| FR-005-AC-3 | Two providers with distinct manifest identities are registered as distinct backends; a run invokes only the provider selected for its routed items. | Test (TC-012) |
| FR-005-AC-4 | Identical manifests for one `BackendId` are idempotent; conflicting manifest digests withdraw the identity and its items are declined without invoking either provider. | Test (TC-012) |

## Dependencies

- QSL ADR-029 PV-1 and PV-3; QSL FR-288 for provider-manifest registration.
- FR-006 defines the separate out-of-process plugin host boundary.
- [TC-012](../test-cases/TC-012-first-party-provider-interface.md).
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point that registers and invokes providers.
