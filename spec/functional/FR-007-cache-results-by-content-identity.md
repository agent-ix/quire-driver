---
id: FR-007
title: "Integrate eligible operations through the cache interface"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-007: Integrate eligible operations through the cache interface

## Description

When a cache-eligible `prove`, `analyze` or `generate` operation is dispatched,
the driver SHALL call its required `Cache` interface with a `CacheRequest`
containing the operation, package identity, canonical typed request, selected
`BackendId`, admitted `ManifestDigest`, and (for `generate`) target and
output-affecting options. Eligibility is exactly QSL ADR-029 CA-5: deterministic
terminal values without wall-time fields, plugin results, internal failures,
cancellation, deadlines or time-limited incomplete results.

## Inputs

- The typed eligible operation request and selected provider identity.
- The provider manifest content identity (`qsl_route::ManifestDigest`) admitted
  with the registry entry; the digest is computed by QSL from canonical FR-331
  manifest bytes.
- The caller's cache configuration and an injected cache interface.
- The selected provider for a cache miss.

## Outputs

- `CacheLookup::Hit(TypedResult)` or `CacheLookup::Miss`, or a typed cache I/O
  fault. A non-eligible result yields `CacheStore::NeverStore`; a result too
  large or unable to fit the configured store yields `CacheStore::NotStored`
  and does not fail the provider operation.

## Behavior

- When an eligible operation is dispatched, the driver calls
  `Cache::lookup(CacheRequest, TypedResultReader)` and
  `Cache::record(CacheRequest, TypedResult)`; a cache I/O fault remains a typed
  error and does not become a miss.
- When lookup returns a valid typed hit, the driver returns it unchanged and
  does not invoke the provider.
- When lookup returns a miss, the driver invokes the selected provider exactly
  once, returns its typed result unchanged, and records that same value.
- The downstream composition SHALL require a `quire-cache` implementation
  that depends on the driver interface; `quire-driver` SHALL NOT depend on the
  concrete `quire-cache` package.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | Each lookup carries operation, package identity, canonical request, selected `BackendId`, admitted `ManifestDigest`, and (for `generate`) target and output-affecting options. | Test (TC-014) |
| FR-007-AC-2 | A valid `CacheLookup::Hit` is returned unchanged and does not invoke the provider. | Test (TC-014) |
| FR-007-AC-3 | A `CacheLookup::Miss` invokes the selected provider exactly once and records the same typed result that the driver returns. | Test (TC-014) |
| FR-007-AC-4 | A cache I/O fault remains a typed error and is not treated as a miss. | Test (TC-014) |
| FR-007-AC-5 | `NeverStore` and `NotStored` preserve the provider result and do not fail the operation. | Test (TC-014) |

## Dependencies

- QSL ADR-029 CA-1 through CA-5 defines cache eligibility and cache semantics;
  QSL-636 amends QSpec FR-201/FR-306 and QSL FR-288 so `ManifestDigest` is
  explicit key material while `BackendId` remains identity-only.
- [FR-012](./FR-012-implement-the-downstream-content-addressed-cache.md)
  defines the downstream cache component that implements the interface.
- [TC-014](../test-cases/TC-014-cache-content-identity-and-validation.md)
  verifies driver-to-cache interface integration.
