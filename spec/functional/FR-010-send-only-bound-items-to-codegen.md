---
id: FR-010
title: "Send only abstraction-relation-bound items to code generation"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-307
    type: depends_on
---
# FR-010: Send only abstraction-relation-bound items to code generation

## Description

When an abstraction relation binds a requested item, the driver SHALL send
only the bound items to CG's negotiation and generation operations. The
driver SHALL not send the package's unbound item set to CG.

## Behavior

- The driver SHALL read the admitted abstraction relation from the checked
  package and determine which requested items it binds.
- The driver SHALL submit only those bound items to CG's negotiation and
  generation operations.
- Where a required binding is absent, the driver SHALL preserve the typed
  unbound-element refusal defined by the owning QSL contract; it SHALL not
  invent a CG item or silently treat the item as bound.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-010-AC-1 | Given a package whose abstraction relation binds a subset of requested items, CG negotiation receives exactly that subset and receives no unbound item. | Test |
| FR-010-AC-2 | CG generation receives only items that the same admitted relation bound and that negotiation settled for generation. | Test |
| FR-010-AC-3 | When a binding required for emission is absent, the driver returns the QSL-defined typed unbound-element refusal and sends no unbound item to CG. | Test |

## Dependencies

- QSL AR2 / QSL-514 and FR-307-AC-6 define the bound-item selection and
  typed unbound-element refusal. The implementation verification is planned
  until QSL AR2 lands.
- The cross-repository acceptance test is owned by QSL's
  `quire-integration` suite.
- [TC-017](../test-cases/TC-017-only-bound-items-reach-codegen.md).
