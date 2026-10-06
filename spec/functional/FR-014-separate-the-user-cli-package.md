---
id: FR-014
title: "Place the user CLI in a separate workspace package"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
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
package. The `quire-cli` package SHALL own the binary argument parser, CLI
tests and their fixtures. It SHALL compose the driver library and downstream
cache.

## Inputs

- The driver repository Cargo workspace and its package manifests.
- The `quire-driver` and `quire-cache` packages.

## Outputs

- A workspace package named `quire-cli` that provides the `quire` executable.
- The binary argument parser and CLI tests/fixtures owned by `quire-cli`.
- A Cargo dependency graph in which the CLI composes the driver and cache.

## Behavior

- The driver repository's Cargo workspace SHALL include a separate
  `quire-cli` package.
- Exactly one workspace package, `quire-cli`, SHALL provide the `quire` binary
  target.
- `quire-cli` SHALL have direct normal dependencies on `quire-driver` and
  `quire-cache`.
- The workspace root SHALL list `quire-cli` in `default-members`.
- The `quire-cli` package SHALL own the argument parser in
  `quire-cli/src/cli.rs`, with the binary entry point in
  `quire-cli/src/main.rs`.
- CLI contract and binary process-boundary tests and their fixtures SHALL
  reside in the `quire-cli` package.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-014-AC-1 | Exactly one workspace package, `quire-cli`, provides the `quire` binary target. | Test (TC-028) |
| FR-014-AC-2 | `quire-cli` has direct normal dependency edges to `quire-driver` and `quire-cache`. | Test (TC-028) |
| FR-014-AC-3 | The workspace root lists `quire-cli` in `default-members`. | Test (TC-005) |
| FR-014-AC-4 | CLI contract and binary process-boundary tests and their fixtures reside in the `quire-cli` package. | Test (TC-009, TC-018, TC-028) |
| FR-014-AC-5 | The CLI parser definition resides in `quire-cli/src/cli.rs`, and the binary entry point is in `quire-cli/src/main.rs`; `quire-driver` does not export a `cli` module. | Test (TC-028) |

## Dependencies

- QSL ADR-029 CB-3 package-boundary guidance.
- FR-001-AC-11 and FR-012-AC-13 require the driver closure to exclude
  `quire-cache`; TC-005 verifies those criteria.
- [FR-001](./FR-001-orchestrate-the-driver-steps.md) keeps the driver core
  independent of downstream cache implementation.
- [FR-002](./FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  binary's command and outcome contract.
- [FR-012](./FR-012-implement-the-downstream-content-addressed-cache.md)
  defines the separate cache package.
- [TC-005](../test-cases/TC-005-dependency-edges.md) and
  [TC-028](../test-cases/TC-028-cli-package-boundary.md) verify the graph.
