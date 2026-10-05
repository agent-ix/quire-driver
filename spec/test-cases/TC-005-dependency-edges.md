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

Scope: FR-001-AC-1, FR-001-AC-11 and FR-012-AC-13.

## Test Procedure

1. Run `cargo metadata --locked --offline` and read the resolved graph.
   Count normal and build edges. Identify QSL packages by their source,
   `git+https://github.com/agent-ix/quire-spec-language`.
2. Traverse the complete normal/build dependency closure from `quire-driver`.
   Identify `quire-cache` and inspect its direct dependency edges.

Test: `tests/dependency_edges.rs`, `tc_005_driver_edges_match_the_adr_011_allow_list`.

## Expected Results

- The driver's normal and build dependencies are exactly `qsl-foundation`,
  `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,
  `quire-exact`, `quire-contract-codegen`, `quire-contract-model` and
  `thiserror`. The QSL root crate and `quire-cache` are not direct edges.
  `serde_json` is a dev-dependency (this crate's own tests use it), not a
  normal one. No package in the driver's transitive normal/build dependency
  closure is `quire-cache`.
- The driver's `quire-contract-model` is the package CG's
  `quire-contract-ir` depends on.
- If the workspace has a `quire-cache` package, it is separate from
  `quire-driver` and depends on the driver-owned interface. No transitive
  normal/build path leads from `quire-driver` to `quire-cache`.
- No QSL, IR or CG package depends on `quire-driver`. `quire-cache` depends on
  the driver-owned interface; `quire-cli` may depend on the driver interface.
- CG's one normal or build edge into QSL is `qsl-replay`.
- The graph holds one copy of `quire-contract-model`: the driver's own
  direct dependency, CG's `quire-contract-ir` and QSL's `qsl-package` all
  resolve the same package id (by identity, not just by revision string).
