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
   Count normal and build edges. Identify QSL packages by their source,
   `git+https://github.com/agent-ix/quire-spec-language`.

Test: `tests/dependency_edges.rs`, `tc_005_driver_edges_match_the_adr_011_allow_list`.

## Expected Results

- The driver's normal and build dependencies are exactly `qsl-foundation`,
  `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,
  `quire-contract-codegen` and `thiserror`. The QSL root crate is not one.
- One package is named `quire-driver`, and no edge in the lock reaches a
  package of that name: nothing in this lock depends on the driver.
- CG's one normal or build edge into QSL is `qsl-replay`.
- The driver's five QSL crates and CG's `qsl-replay` share one source
  revision. Each QSL package other than the root crate resolves once. The
  root crate `quire-spec-language` arrives transitively through IR at
  another revision (f1700a9); this test states that and does not gate it.
