---
id: FR-008
title: "Execute checked entries through an AOT backend"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-008: Execute checked entries through an AOT backend

## Description

When the caller selects the AOT execution engine, the driver SHALL prepare
and execute only entries from the checked package produced by that request.
For the same checked input and accounting limits, the AOT backend SHALL return
the same evaluation outcome as QSL's interpreter.

## Inputs

- The checked package and selected entries from the current request.
- Preparation and accounting limits, cancellation, and the caller-named
  build toolchain.

## Outputs

- An `Evaluation<Value>` for each selected entry with the interpreter's
  outcome, typed cause, source location and loss records, or a typed prepare
  failure.

## Behavior

- AOT preparation SHALL lower the checked package, generate an AOT target and
  build it with the toolchain named by the caller.
- The generated executable SHALL run as a child process over the canonical
  framed protocol defined for the execution boundary.
- The AOT backend SHALL preserve interpreter semantics for completed,
  undefined, refused and incomplete results, including causes, locations,
  loss records and accounting-limit outcomes.
- When an AOT preparation limit is reached, the driver SHALL return a typed
  limit failure.
- When the AOT backend cannot prepare an entry, the driver SHALL mark only
  that entry unsupported.
- The driver SHALL NOT silently substitute the interpreter or another engine.
- The AOT executable SHALL perform no I/O beyond the execution protocol.
- The AOT executable SHALL NOT use clock, random source, environment or
  thread state to determine a semantic result.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-008-AC-1 | AOT accepts a checked package and selected entries, lowers and generates from that package, and builds with the toolchain named by the caller. | Test |
| FR-008-AC-2 | The generated executable exchanges canonical framed requests and results with the driver and reads no package bytes outside the current checked request. | Test |
| FR-008-AC-3 | For each conformance input and accounting budget, AOT and the interpreter return equal completed, undefined, refused or incomplete outcomes, including typed cause, location, loss records and limit result. | Test |
| FR-008-AC-4 | An unsupported entry remains unsupported for that entry, a preparation limit remains a typed limit, and neither condition silently falls back to another engine. | Test |
| FR-008-AC-5 | AOT's semantic result is independent of wall clock, random source, environment and thread state, and the generated executable performs no I/O outside its protocol. | Inspection |

## Dependencies

- QSL ADR-029 EB-1 through EB-6 and EB-8; QSpec FR-306.
- [TC-015](../test-cases/TC-015-aot-matches-interpreter.md).
