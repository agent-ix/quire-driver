---
id: FR-008
title: "Invoke the selected AOT execution backend"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-008: Invoke the selected AOT execution backend

## Description

When an execution request selects AOT, the driver SHALL invoke QSL's AOT
backend with only the checked package and selected entries from that request.
The driver SHALL return the backend's typed result or failure without
substituting another engine.

## Inputs

- The checked package and selected entries produced by the current request.
- The selected AOT engine, preparation and accounting limits, cancellation
  input and caller-named build toolchain.

## Outputs

- The AOT backend's typed evaluation results or typed preparation failures.

## Behavior

- QSL ADR-029 EB-1 through EB-6 and EB-8 own the execution seam, checked-only
  admission, interpreter parity, AOT preparation and engine selection
  semantics.
- The driver SHALL pass the current request's checked package, selected
  entries, limits, cancellation input and named toolchain to the AOT backend.
- The driver SHALL propagate the backend's typed results and failures
  unchanged without retrying another execution engine.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-008-AC-1 | When a request selects AOT, the driver calls the backend with the checked package and selected entries from that request, its limits and cancellation input, and the caller-named toolchain. | Test |
| FR-008-AC-2 | The driver returns the backend's typed results and preparation failures unchanged and does not invoke another engine after an AOT failure. | Test |

## Dependencies

- QSL ADR-029 EB-1 through EB-6 and EB-8 define the execution and parity
  contract; QSpec FR-306 defines cross-backend conformance evidence.
- [TC-015](../test-cases/TC-015-aot-matches-interpreter.md).
