---
id: TM-001
title: "Driver command and execution test matrix"
type: TestMatrix
---
# TM-001: Driver binary, provider and execution test matrix

## Overview

This authored verification inventory integrates FR-001 and SP-D1 through
SP-D9 (FR-002 through FR-010) from IR-527, plus FR-011 from IR-616. It covers
TC-001 through TC-019. Discrete FR and TC artifacts remain authoritative;
this matrix is their index, not a computed implementation-coverage report.

`✅` means the existing source contains the test implementations named by
TC-001 through TC-008; it does not claim a fresh test run or compatibility
with current upstream APIs. `🚧` means verification is planned. This spec-only
change runs artifact validation, not source tests or the G5b gate.

## Verification Ownership

| Scope | Verification owner and location |
|---|---|
| FR-001 / TC-001..TC-008 | Driver orchestration tests in `tests/drive.rs`, `tests/dependency_edges.rs` and `src/refusal.rs`, as named by the TC artifacts; upstream API adaptation is IR-606. |
| FR-001-AC-9..10, FR-011 / TC-019 | Planned driver library cancellation and source-provision lifetime test plus API/lifetime review. |
| FR-002..FR-009 / TC-009..TC-016, TC-018 | Planned driver CLI and composition tests at the locations named by each TC; they check dispatch, input fidelity, typed result preservation and single-owner mappings. |
| Plugin protocol, cache policy, execution semantics and AOT/interpreter parity | QSL ADR-029 owner suites; driver TC-013..TC-015 check composition boundaries. They do not re-specify upstream policy. |
| FR-010 / TC-017 | Planned QSL-owned cross-repository test in `quire-integration`, QSL-514; QSL AR2 is the prerequisite. |
| G5b acceptance evidence / IR-528 | `quire-integration` only: CLI/library parity across the eight outcome kinds with identical outcome bytes and exit codes; provider/plugin installation; cache hit/miss/never-store; AOT/interpreter parity; absence of a QSL user binary; and prove execution, replay and FR-331 records. |

The driver supplies the one user-facing CLI. CG FR-005's CLI-facing
conformance is exercised through that binary; CG keeps its local generator
and atomic publication tests. CG's cross-backend parity and IR-337's
outcome-per-backend and bound-suite coverage feed the G5b evidence rather
than a second driver or CG gate. The driver and QSL suites supply local
verification; they do not substitute for IR-528 acceptance evidence.

## Requirements Traceability

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5, FR-001-AC-6, FR-001-AC-7, FR-001-AC-8, FR-001-AC-9, FR-001-AC-10 | TC-001, TC-002, TC-003, TC-004, TC-005, TC-006, TC-007, TC-008, TC-019 | ✅ TC-001..TC-008 tests present; TC-019 planned |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-009, TC-018 | 🚧 Planned |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3 | TC-010 | 🚧 Planned |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-011 | 🚧 Planned |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3 | TC-012 | 🚧 Planned |
| FR-006 | FR-006-AC-1, FR-006-AC-2 | TC-013 | 🚧 Planned |
| FR-007 | FR-007-AC-1, FR-007-AC-2 | TC-014 | 🚧 Planned |
| FR-008 | FR-008-AC-1, FR-008-AC-2 | TC-015 | 🚧 Planned |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | TC-016 | 🚧 Planned |
| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 | TC-017 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
| FR-011 | FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-011-AC-4, FR-011-AC-5 | TC-019 | 🚧 Planned |

## Test Case Summary

| Test ID | Title | Type | Traces To | Status |
|---|---|---|---|---|
| TC-001 | The steps run in ADR-011 T-13 order | Integration | FR-001-AC-4 | ✅ Tests present |
| TC-002 | An unknown BackendId refuses the run before negotiation | Integration | FR-001-AC-2 | ✅ Tests present |
| TC-003 | route runs after negotiation, for supported items only | Integration | FR-001-AC-3 | ✅ Tests present |
| TC-004 | Command-level refusals are typed | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-005 | The dependency edges match ADR-011 §6.1 and §7.1 | Integration | FR-001-AC-1 | ✅ Tests present |
| TC-006 | E5 admits the emitted package and CG generates for the routed items | Integration | FR-001-AC-6, FR-001-AC-7, FR-001-AC-8 | ✅ Tests present |
| TC-007 | Generation refusals are typed | Integration | FR-001-AC-5, FR-001-AC-6 | ✅ Tests present |
| TC-008 | A missing import or an unknown header profile refuses at compile | Integration | FR-001-AC-5 | ✅ Tests present |
| TC-009 | Exercise CLI verbs, outcome documents and exit codes | Integration | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | 🚧 Planned |
| TC-010 | Apply explicit lifecycle limits and preserve limit outcomes | Integration | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3 | 🚧 Planned |
| TC-011 | Select only the requested execution engine | Integration | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | 🚧 Planned |
| TC-012 | Register and invoke a first-party provider through Provider | Integration | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3 | 🚧 Planned |
| TC-013 | Dispatch process-provider results through the driver | Integration | FR-006-AC-1, FR-006-AC-2 | 🚧 Planned |
| TC-014 | Pass manifest content to the QSL cache | Integration | FR-007-AC-1, FR-007-AC-2 | 🚧 Planned |
| TC-015 | Invoke AOT with current checked request inputs | Integration | FR-008-AC-1, FR-008-AC-2 | 🚧 Planned |
| TC-016 | Settle one terminal record per item after replay | Integration | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | 🚧 Planned |
| TC-017 | Send only abstraction-relation-bound items to code generation | Integration | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 | 🚧 Planned in quire-integration; QSL AR2 prerequisite |
| TC-018 | Preserve the QSL outcome at the binary process boundary | Integration | FR-002-AC-2, FR-002-AC-3, FR-002-AC-4 | 🚧 Planned |
| TC-019 | Propagate library cancellation and release the emitted source provision | Integration | FR-001-AC-9, FR-001-AC-10, FR-011-AC-1..FR-011-AC-5 | 🚧 Planned |
