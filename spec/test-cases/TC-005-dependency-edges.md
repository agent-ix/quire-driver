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
   `git+https://github.com/agent-ix/quire-spec-language`. Compare the driver's
   complete direct normal/build inventory and its separate direct development
   inventory with exact-set equality. Retain exactly six normal QSL layer
   edges; `qsl-analyze` is a normal edge ([FR-019](../functional/FR-019-prepare-the-complete-monitor-program-under-one-budget.md) Dependencies).
2. Traverse the complete normal/build dependency closure from `quire-driver`.
   Identify `quire-cache` and inspect its direct dependency edges. Existing
   `tests/dependency_edges.rs` coverage verifies the driver/cache boundary and
   host-free driver closure.
3. Inspect the direct normal dependency edges from `quire-cli` and confirm it
   depends on `quire-plugin-host` and `ix-cli-kit`; confirm the driver closure
   remains host-free and kit-free. The CLI-to-host edge is already asserted by
   `tests/dependency_edges.rs` and `quire-cli/tests/package_boundary.rs`.
4. Confirm the authorised direct runtime edge and traverse RT's complete
   normal/build closure. It must contain no QSL-source package, IR/model,
   driver or codegen backedge. This dependency direction check alone is not
   evidence that monitor preparation or execution has succeeded.

Test: `tests/dependency_edges.rs`, `tc_005_driver_edges_match_the_adr_011_allow_list`.

## Expected Results

- The driver's normal and build dependencies are exactly `qsl-analyze`,
  `qsl-foundation`, `qsl-package`, `qsl-replay`, `qsl-route`, `qsl-semantics`,
  `quire-exact`, `quire-contract-codegen`, `quire-contract-model`, `quire-contract-runtime`,
  `quire-canonical`, `quire-observation`, `quire-semantic-value`, `serde`, `serde_json` and `thiserror`. `serde_json` is a
  normal edge for production projection of FR-331's retained optional
  `bounds` member. The direct `quire-semantic-value` normal edge supplies
  existing shared catalog owner types after their ownership moved out of
  QSL; this is not a re-export shim or an additional driver API. The
  dependency assertion remains exact-set equality. The QSL root crate and `quire-cache` are not direct edges.
  The RT edge supplies the native program/arena types authorised by FR-015
  and FR-017, with no QSL/IR/model/driver/codegen backedge in RT's complete
  normal/build closure.
  `quire-plugin-host` is not in the driver's normal or build dependency
  closure. No package in the driver's transitive normal/build dependency
  closure is `quire-cache`.
- The driver's direct development dependencies are exactly `ix-trace-rs`,
  and `quire-checked`. `qsl-analyze` is not a development edge: FR-019 takes
  it as a normal dependency for `PlanRoot` and `AdmittedPlanAuthority`.
  `quire-observation` is a normal edge because FR-019's request field
  `progress` has type `quire_observation::authority::progress::View`, issued
  under quire-observation FR-004 (AC-7). `quire-checked` is test-only and supplies the existing
  `DeclarationKey` owner type relocated from QSL; it adds no normal/build
  edge or driver API. `serde_json` remains a normal dependency used by tests.
- The driver's `quire-contract-model` is the package CG's
  `quire-contract-ir` depends on.
- If the workspace has a `quire-cache` package, it is separate from
  `quire-driver` and depends on the driver-owned interface. No transitive
  normal/build path leads from `quire-driver` to `quire-cache`.
- The `workspace_default_members` array includes `quire-cli`.
- When the workspace contains `quire-cli`, the graph includes its driver
  interface and cache edges, plus direct normal edges to `quire-plugin-host`
  and `ix-cli-kit`. The direct CLI-to-host edge is already asserted by the
  current dependency test together with the CLI-to-kit edge. `ix-cli-kit`
  remains outside the driver's closure. The driver's closure
  remains cache-free and host-free.
- CG's one normal or build edge into QSL is `qsl-replay`.
- The graph holds one copy of `quire-contract-model`: the driver's own
  direct dependency, CG's `quire-contract-ir` and QSL's `qsl-package` all
  resolve the same package id (by identity, not just by revision string).
