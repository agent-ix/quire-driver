---
id: FR-002
title: "Expose the lifecycle through one driver CLI and outcome contract"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-285
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-286
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-287
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-301
    type: depends_on
---
# FR-002: Expose the lifecycle through one driver CLI and outcome contract

## Description

The driver repository SHALL provide the one user-facing `quire` binary. Its
lifecycle commands, output options, multi-item severity and process behavior SHALL
conform to QSL ADR-029 LC-1, CB-1, CB-4 and CB-5.

Each verb SHALL call its corresponding typed QSL or driver operation.
The CLI SHALL not make semantic decisions based on diagnostic text.

In machine mode, each verb SHALL write exactly one QSL-defined
`quire-outcome/1` JSON value to stdout and preserve the typed library outcome
without changing its schema or values. The driver SHALL use QSL's single
total `Category::exit_code` function as the sole category-to-exit mapping.

## Inputs

- A lifecycle command and its options as defined by QSL ADR-029.
- Input documents and source files named by the options, or stdin where the
  verb accepts stdin.
- The typed outcome returned by the operation.

## Outputs

- One QSL-defined `quire-outcome/1` JSON value in machine mode, or the
  selected text rendering.
- The exit status returned by QSL's category exit function.

## Behavior

- The `quire` binary SHALL be provided by the `quire-cli` workspace package;
  the QSL workspace supplies no competing user binary. The `quire-driver`
  library SHALL NOT compose the downstream `quire-cache` package. FR-014
  specifies the separate CLI package boundary.
- The CLI SHALL preserve the library's QSL-defined `quire-outcome/1` value
  without adding, removing or changing outcome data.
- The CLI SHALL use ix-cli-kit for shared command/output, configuration and
  version mechanics. Its typed `Outcome` mapping SHALL NOT replace or remap
  QSL's category exit function. Until ix-cli-kit exposes a raw exit-code
  pass-through, the CLI SHALL construct `std::process::ExitCode` directly
  from QSL's returned code at the process boundary.
- The CLI SHALL read only input paths named by its arguments and read no
  unnamed configuration file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | The driver binary exposes the command and output surface defined by QSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation. | Test (TC-009) |
| FR-002-AC-2 | For each O-16 category, the process status equals the value returned by QSL's `Category::exit_code`; the CLI has no second category-to-exit mapping. | Test (TC-009, TC-018) |
| FR-002-AC-3 | In machine mode, each command writes one parseable `quire-outcome/1` value equal to the library outcome under QSL's schema; the CLI adds, removes or changes no outcome data. | Test (TC-009, TC-018) |
| FR-002-AC-4 | The CLI uses ix-cli-kit for shared command/output, configuration and version mechanics, while preserving QSL's numeric exit result without the kit's typed `Outcome` remapping. | Test (TC-009, TC-018) |
| FR-002-AC-5 | The CLI reads only input paths named by its arguments and reads no unnamed configuration file. | Test (TC-009) |

## Dependencies

- QSL ADR-029 CB-1, CB-4 and CB-5; QSL FR-285, FR-286 and FR-287.
- QSpec FR-300 and FR-301 for the CLI output and process contract.
- ix-cli-kit provides shared mechanics only; its current typed `Outcome`
  values cannot represent QSL's category exit codes. The API pass-through
  question is tracked on PLAT-1140; QSL ADR-029 remains authoritative.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
- [TC-018](../test-cases/TC-018-binary-process-boundary.md), adding verification for AC-2 through AC-4.
