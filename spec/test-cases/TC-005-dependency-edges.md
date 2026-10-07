---
id: TC-005
title: "The dependency edges match ADR-011 §6.1 and §7.1"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-012
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-014
    type: verifies
---
# TC-005: The dependency edges match ADR-011 §6.1 and §7.1

## Description

Verify the driver's place in the crate graph from `cargo metadata`.

Scope: FR-001-AC-1, FR-001-AC-11, FR-012-AC-13, FR-013-AC-11,
FR-014-AC-2 and FR-014-AC-3.

## Test Procedure

1. Run `cargo metadata --locked --offline` and read the resolved graph and
   `workspace_default_members`. Count normal and build edges. Identify QSL
   packages by their source,
   `git+https://github.com/agent-ix/quire-spec-language`.
2. Traverse the complete normal/build dependency closure from `quire-driver`.
   Identify `quire-cache` and inspect its direct dependency edges. Existing
   `tests/dependency_edges.rs` coverage verifies the driver/cache boundary and
   host-free driver closure.
3. Inspect the direct normal dependency edges from `quire-cli` and confirm it
   depends on `quire-plugin-host` and `ix-cli-kit`; confirm the driver closure
   remains host-free. The direct CLI-to-host assertion is missing from the
   current test and is planned for the IR-672 code slice. The kit edge
   assertion is planned for the IR-607 kit-adoption slice.

Test: `tests/dependency_edges.rs`, `tc_005_driver_edges_match_the_adr_011_allow_list`.

## Expected Results

- The driver's normal and build dependencies are exactly `qsl-foundation`,
  `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,
  `quire-exact`, `quire-contract-codegen`, `quire-contract-model` and
  `thiserror`. The QSL root crate and `quire-cache` are not direct edges.
  `quire-plugin-host` is not in the driver's normal or build dependency
  closure.
  `serde_json` is a dev-dependency (this crate's own tests use it), not a
  normal one. No package in the driver's transitive normal/build dependency
  closure is `quire-cache`.
- The driver's `quire-contract-model` is the package CG's
  `quire-contract-ir` depends on.
- If the workspace has a `quire-cache` package, it is separate from
  `quire-driver` and depends on the driver-owned interface. No transitive
  normal/build path leads from `quire-driver` to `quire-cache`.
- The `workspace_default_members` array includes `quire-cli`.
- When the workspace contains `quire-cli`, the graph includes its driver
  interface and cache edges, plus direct normal edges to `quire-plugin-host`
  and `ix-cli-kit`. The direct CLI-to-host edge assertion remains planned for
  the IR-672 code slice; the CLI-to-kit assertion remains planned for the
  IR-607 kit-adoption slice. The driver's closure remains cache-free and
  host-free and does not include `ix-cli-kit`.
- CG's one normal or build edge into QSL is `qsl-replay`.
- The graph holds one copy of `quire-contract-model`: the driver's own
  direct dependency, CG's `quire-contract-ir` and QSL's `qsl-package` all
  resolve the same package id (by identity, not just by revision string).
