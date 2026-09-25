---
id: TC-002
title: "An unknown BackendId refuses the run before negotiation"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-002: An unknown BackendId refuses the run before negotiation

## Description

Verify the MD-2 refusal (QSL ADR-012 §7.2).

Scope: FR-001-AC-2.

## Test Procedure

1. Build a registry holding `alpha`, `kani` and `Zeta`.
2. Drive a complete-V1 unit with one item.

Test: `tests/drive.rs`, `tc_002_unknown_backend_refuses_before_negotiation_in_bytewise_order`.

## Expected Results

- The run refuses with `invalid_capability`/`unknown-backend` at the
  conversion step, outcome kind refusal.
- It names `Zeta` then `alpha`, bytewise order, which a case-insensitive
  order would reverse.
- All three ids went through the conversion, and negotiation and routing
  were never called.
