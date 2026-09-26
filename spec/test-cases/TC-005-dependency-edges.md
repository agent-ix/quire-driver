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
  `quire-contract-codegen`, `quire-contract-model` and `thiserror`. The QSL
  root crate is not one. `serde_json` is a dev-dependency (this crate's own
  tests use it), not a normal one.
- The driver's `quire-contract-model` is the package CG's
  `quire-contract-ir` depends on.
- One package is named `quire-driver`, and no edge in the lock reaches a
  package of that name: nothing in this lock depends on the driver.
- CG's one normal or build edge into QSL is `qsl-replay`.
- The driver's own QSL pin's `qsl-package` resolves the same
  `quire-contract-model` package id as the driver's own direct pin (by
  identity, not just by revision string): QSL-255b PR 2 (this crate's own
  QSL bump) made the two share one copy. Two more copies remain, both
  outside this crate's own pins and not removed by this bump: CG's own
  `qsl-replay` pin is still `daa0178` (CG has not moved it to match this
  crate's QSL revision, and no code in CG's own `src/` calls it), so it
  still resolves an older `qsl-package` and, through it, an older
  `quire-contract-model` (`b52c263`); and `quire-contract-ir` (CG's IR
  edge) depends on the QSL root crate at a revision of its own, which pins
  a third copy (`7b1ec7a`). Both are informational (Cargo.toml's revision
  note) and not gated here.
