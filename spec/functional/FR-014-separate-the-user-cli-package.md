---
id: FR-014
title: "Place the user CLI in a separate workspace package"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: references
  - target: ix://agent-ix/quire-driver/FR-002
    type: references
  - target: ix://agent-ix/quire-driver/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-014: Place the user CLI in a separate workspace package

## Description

The driver repository SHALL provide its user-facing `quire` executable from a
workspace package named `quire-cli`, separate from the `quire-driver` library
package. `quire-cli` SHALL compose the CLI kit, driver library and downstream
cache without adding the cache to the driver library's dependency closure.

## Inputs

- The driver repository Cargo workspace and its package manifests.
- The `ix-cli-kit`, `quire-driver` and `quire-cache` packages.

## Outputs

- A workspace package named `quire-cli` that provides the `quire` executable.
- A Cargo dependency graph in which the CLI composes the driver and cache while
  the driver remains independent of the cache.

## Behavior

- The driver repository's Cargo workspace SHALL include a separate
  `quire-cli` package.
- The `quire-cli` package SHALL provide the `quire` binary target.
- `quire-cli` SHALL have direct normal dependencies on `ix-cli-kit`,
  `quire-driver` and `quire-cache`.
- `quire-driver` SHALL NOT have a direct or transitive normal/build dependency
  path to `quire-cache`.
- The `quire-driver` package SHALL NOT provide a `quire` binary target.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-014-AC-1 | The workspace contains a distinct `quire-cli` package with the `quire` binary target, and `quire-driver` does not provide that binary target. | Test (TC-028) |
| FR-014-AC-2 | `quire-cli` has direct normal dependency edges to `ix-cli-kit`, `quire-driver` and `quire-cache`. | Test (TC-028) |
| FR-014-AC-3 | No normal/build dependency path from `quire-driver` reaches `quire-cache`, while the `quire-cli` graph composes both packages. | Test (TC-005, TC-028) |

## Dependencies

- QSL ADR-029 CB-3 package-boundary guidance.
- [FR-001](./FR-001-orchestrate-the-driver-steps.md) keeps the driver core
  independent of downstream cache implementation.
- [FR-002](./FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  binary's command and outcome contract.
- [FR-012](./FR-012-implement-the-downstream-content-addressed-cache.md)
  defines the separate cache package.
- [TC-005](../test-cases/TC-005-dependency-edges.md) and
  [TC-028](../test-cases/TC-028-cli-package-boundary.md) verify the graph.
