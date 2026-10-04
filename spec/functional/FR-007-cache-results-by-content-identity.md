---
id: FR-007
title: "Supply complete provider identity to the QSL cache"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-007: Supply complete provider identity to the QSL cache

## Description

When the driver enables caching for an eligible operation, it SHALL pass the
selected provider's manifest bytes as content input to QSL's cache contract.
The driver SHALL NOT substitute a recorded repository revision or version
pin for those bytes.

## Inputs

- The typed eligible operation request and selected provider.
- The selected provider's manifest bytes.
- The caller's cache configuration.

## Outputs

- The typed result returned by QSL's cache contract on a hit or miss.

## Behavior

- QSL ADR-029 CA-1 through CA-5 own cache eligibility, key construction,
  invalidation, entry validation, storage and result handling.
- The driver SHALL pass the exact selected provider manifest bytes and typed
  operation request to the QSL cache interface; it SHALL not implement a
  second cache key or cache-result policy.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-007-AC-1 | For an eligible cached request, the driver supplies the typed operation request and exact selected provider manifest bytes to the QSL cache interface. | Test |
| FR-007-AC-2 | Changing manifest bytes while keeping any recorded revision or version pin fixed changes the content input supplied by the driver; the driver does not replace those bytes with the pin. | Test |

## Dependencies

- QSL ADR-029 CA-1 through CA-5 define eligibility, canonical content key,
  invalidation, validation and storage behavior; QSpec FR-201 and FR-306
  define the related operation contracts.
- [TC-014](../test-cases/TC-014-cache-content-identity-and-validation.md).
