---
type: master-requirements
name: quire-driver
org: agent-ix
component_type: rust-lib
implementation_language: rust
standards_alignment:
  - iso-iec-ieee-29148
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: contains
  - target: ix://agent-ix/quire-driver/FR-001
    type: contains
  - target: ix://agent-ix/quire-driver/TC-001
    type: contains
  - target: ix://agent-ix/quire-driver/TC-002
    type: contains
  - target: ix://agent-ix/quire-driver/TC-003
    type: contains
  - target: ix://agent-ix/quire-driver/TC-004
    type: contains
  - target: ix://agent-ix/quire-driver/TC-005
    type: contains
  - target: ix://agent-ix/quire-driver/TC-006
    type: contains
  - target: ix://agent-ix/quire-driver/TC-007
    type: contains
---
# Master Requirements Specification

## quire-driver

## 1. Purpose

`quire-driver` is the orchestrating driver crate of QSL ADR-011 T-13 (§6.1
driver row) and ADR-012 §7. It is the one crate that calls both QSL and CG
(`quire-contract-codegen`).

## 2. Scope

It compiles a unit through QSL, converts every registered backend to its CG
kind, has CG negotiate every requested item, routes the supported items
through QSL `route`, hands the emitted package to IR's reader (E5) and has CG
generate for each routed item. It sits downstream of QSL and CG, and nothing in its
lock depends on it.

## 3. System Overview

A Rust library. The registry is a value the caller builds with
`build_registry` and passes to `drive`.

## 4. Requirements Architecture

`quire-driver` is the orchestrating driver crate of QSL ADR-011 T-13 (§6.1
driver row) and ADR-012 §7. It is the one crate that calls both QSL and CG
(`quire-contract-codegen`). It sits downstream of both, and nothing depends
on it.

| ID | Title |
|---|---|
| US-001 | Run one checked request from source to routed backends |
| FR-001 | Orchestrate compile, backend conversion, negotiation, routing and generation |
| TC-001 | The steps run in ADR-011 T-13 order |
| TC-002 | An unknown `BackendId` refuses the run before negotiation |
| TC-003 | `route` runs after negotiation, for `supported` items only |
| TC-004 | Command-level refusals are typed |
| TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 |
| TC-006 | E5 admits the emitted package and CG generates for the routed items |
| TC-007 | Generation refusals are typed |

## References

- QSL ADR-011, ADR-012 and ADR-013 (agent-ix/quire-spec-language).
- QSL-1 (Linear), the ticket that owns this crate.
