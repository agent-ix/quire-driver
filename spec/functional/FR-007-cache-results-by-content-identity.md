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

When an eligible operation is executed, the driver SHALL consult its injected
cache interface with the typed request and the selected provider's exact
manifest bytes.

## Inputs

- The typed eligible operation request and selected provider identity.
- The exact canonical manifest bytes admitted for that provider.
- The caller's cache configuration and an injected cache interface.
- The selected provider for a cache miss.

## Outputs

- The same typed result that a fresh provider call or a verified cache hit
  returns.

## Behavior

- The driver SHALL pass the operation, package identity, typed request,
  selected provider identity, exact manifest bytes and generation target and
  options when applicable to the cache interface.
- When the cache interface returns a hit, the driver SHALL return that typed
  result without invoking the provider.
- When the cache interface returns a miss, the driver SHALL invoke the selected
  provider once and pass its typed result to the cache interface for recording.
- The `quire-driver` package SHALL NOT depend on the concrete `quire-cache`
  package; a downstream composition SHALL inject an implementation of the
  cache interface.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | Given an eligible cache miss, the recording cache interface receives the typed operation request, package identity, selected provider identity and byte-for-byte exact admitted manifest content, and receives the provider's typed result for recording. | Test |
| FR-007-AC-2 | Given a cache hit, the driver returns the cached typed result unchanged and does not invoke the provider; changing manifest content while the provider identity and any recorded revision or version pin remain fixed produces a distinct interface input. | Test |

## Dependencies

- QSL ADR-029 CA-1 through CA-5 defines cache eligibility and cache semantics;
  QSpec FR-201 and FR-306 define related content identity.
- [FR-012](./FR-012-implement-the-downstream-content-addressed-cache.md)
  defines the downstream cache component that implements the interface.
- [TC-014](../test-cases/TC-014-cache-content-identity-and-validation.md)
  verifies driver-to-cache interface integration.
