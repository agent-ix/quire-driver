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
- The driver SHALL build the CG request only from the export's bound
  results, including a bound result with an empty binding set.
- When the export refuses a requested item, the driver SHALL omit that item
  from the CG request.
- When every requested item is refused, the driver SHALL send CG a request
  with zero items; FR-001 governs an empty request unchanged.
- Where a required binding is absent, the driver SHALL preserve the typed
  unbound-element refusal defined by the owning QSL contract; it SHALL not
  invent a CG item or silently treat the item as bound.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-010-AC-1 | Given a package whose abstraction relation binds a subset of requested items, CG negotiation receives exactly that subset and receives no unbound item. | Test |
| FR-010-AC-2 | CG generation receives only items that the same admitted relation bound and that negotiation settled for generation. | Test |
| FR-010-AC-3 | When a binding required for emission is absent, the driver returns the QSL-defined typed unbound-element refusal and sends no unbound item to CG. | Test |
| FR-010-AC-4 | Given an export that returns a bound result for a clause item A, a refusal for an extent-domain item B over an unbound population, and a bound result with an empty binding set for an item C that references no model element, the CG request carries A and C and not B. | Test |
| FR-010-AC-5 | When the export refuses every requested item, the driver sends CG a request with zero items. | Test |

## Dependencies

- [QSL FR-307](ix://agent-ix/quire-spec-language/FR-307) defines the
  bound-item export and per-item results, including the bound-or-refused
  result kinds in FR-307-AC-6.
  [QSpec FR-353-AC-3](ix://agent-ix/quire-specification/FR-353#FR-353-AC-3)
  owns the per-item unbound refusal, naming each unbound element with its
  owning `DomainPackageRef`;
  [QSpec FR-331](ix://agent-ix/quire-specification/FR-331) owns the CG
  request built from bound items.
- [QSL ADR-017](ix://agent-ix/quire-spec-language/ADR-017) AR-4/AR-6
  define the export, referenced elements and what CG receives;
  [QSL ADR-011](ix://agent-ix/quire-spec-language/ADR-011) T-13 names
  the orchestrating driver.
- QSL-514 is planned end-to-end `quire-integration` evidence for FR-307's
  end-to-end clause, not a requirement acceptance criterion. The
  cross-repository acceptance test is owned by QSL's `quire-integration` suite.
- [TC-017](../test-cases/TC-017-only-bound-items-reach-codegen.md).
