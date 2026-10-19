---
id: TC-017
title: "Send only abstraction-relation-bound items to code generation"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-010
    type: verifies
---
# TC-017: Send only abstraction-relation-bound items to code generation

## Description

Verify that the driver passes only the item set bound by the admitted
abstraction relation to CG and preserves QSL's typed refusal for a required
unbound item.

Scope: FR-010-AC-1 through FR-010-AC-5.

## Test Procedure

In the quire-integration suite, compose the QSL abstraction-relation intake
and the driver with a package whose relation binds a strict subset of its
requested items. Record the items supplied to CG negotiation and generation.
Repeat with an item that QSL requires to be bound but whose binding is absent.

Partition vector (FR-010-AC-4, FR-010-AC-5): bind nothing but the ConfigVersion object
type, then request three items: A, a clause item reading only ConfigVersion
fields; B, an item whose extent domain ranges over the unbound ConfigVersion
population; C, an item referencing no model element. Repeat with no element
bound and a clause item and B requested.

Test location: planned QSL-owned cross-repository test, QSL-514.

## Expected Results

- CG negotiation receives exactly the bound subset; generation receives
  only bound items subsequently routed for generation.
- No unbound item reaches CG. Where a required binding is absent, the
  driver returns QSL's typed unbound-element refusal.
- For the partition vector, the export returns A bound, B refused
  `missing_declaration`/`missing-name`, and C bound with an empty binding
  set; the CG request carries A and C and not B (FR-010-AC-4).
- For the all-refused repeat, the CG request carries zero items
  (FR-010-AC-5).
