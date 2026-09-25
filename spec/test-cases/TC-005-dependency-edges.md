---
id: TC-005
title: "The dependency edges match ADR-011 §6.1 and §7.1"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-005: The dependency edges match ADR-011 §6.1 and §7.1

## Description

Verify the driver's place in the crate graph from `cargo metadata`.

Scope: FR-001-AC-1.

## Test Procedure

1. Run `cargo metadata --locked --offline` and read the resolved graph.

Test: `tests/dependency_edges.rs`, `tc_005_driver_edges_match_the_adr_011_allow_list`.

## Expected Results

- The driver's normal dependencies are exactly `qsl-foundation`,
  `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,
  `quire-contract-codegen` and `thiserror`. The QSL root crate is not one.
- No package depends on `quire-driver`.
- CG's one normal edge into the QSL layer crates is `qsl-replay`.
- Each QSL layer crate resolves to one revision, and the driver's
  `qsl-replay` is CG's.
