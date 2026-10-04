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

The driver SHALL build the user-facing `quire` binary. The driver SHALL expose
the lifecycle verbs `parse`, `format`, `check`, `compile`, `run`, `analyze`,
`monitor`, `prove`, `replay`, `generate`, `inspect` and `version`. The CLI SHALL
accept `--format json|text` on every verb, including `--format dot` for
`inspect`.

Each verb SHALL call its corresponding typed QSL or driver operation.
The CLI SHALL not make semantic decisions based on diagnostic text.

In machine mode, each verb SHALL write exactly one QSL-defined
`quire-outcome/1` JSON value to stdout and preserve the typed library outcome
without changing its schema or values. The driver SHALL use QSL's single
total `Category::exit_code` function as the sole category-to-exit mapping.

## Inputs

- A lifecycle verb and its options.
- Input documents and source files named by the options, or stdin where the
  verb accepts stdin.
- The typed outcome returned by the operation.

## Outputs

- One QSL-defined `quire-outcome/1` JSON value in machine mode, or the
  selected text rendering.
- The exit status returned by QSL's category exit function.

## Behavior

- The `quire` binary SHALL be built from the driver repository; the QSL
  workspace supplies no competing user binary.
- The CLI SHALL preserve the library's QSL-defined `quire-outcome/1` value
  without adding, removing or changing outcome data.
- For a multi-item outcome, the CLI SHALL use the most severe item code under
  QSL's defined severity order.
- The CLI SHALL use ix-cli-kit for shared command/output, configuration and
  version mechanics. Its typed `Outcome` mapping SHALL NOT replace or remap
  QSL's category exit function. Until ix-cli-kit exposes a raw exit-code
  pass-through, the CLI SHALL construct `std::process::ExitCode` directly
  from QSL's returned code at the process boundary.
- If the CLI receives an unknown verb, option, profile or provider, then the
  CLI SHALL refuse before a lifecycle stage runs with QSL's refusal category
  and a diagnostic naming the refused argument.
- If writing to stdout or stderr fails, then the CLI SHALL return an internal
  failure category and obtain the process status from QSL's category exit
  function.
- If the CLI encounters a broken pipe, then the CLI SHALL end quietly with
  the outcome's own exit status.
- The CLI SHALL read only input paths named by its arguments and read no
  unnamed configuration file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | The `quire` binary accepts exactly the listed lifecycle verbs, each accepts `--format json|text`, and `inspect` accepts `dot`; an unknown verb, option, profile or provider is refused before any stage with QSL's refusal category and a diagnostic naming the argument. | Test (TC-009) |
| FR-002-AC-2 | For each O-16 category, the process status equals the value returned by QSL's `Category::exit_code`; multi-item outcomes use QSL's severity order, and no second category-to-exit mapping runs. | Test (TC-009) |
| FR-002-AC-3 | In JSON mode, each verb writes one parseable `quire-outcome/1` value equal to the library outcome under QSL's schema; the CLI adds, removes or changes no outcome data. | Test (TC-009) |
| FR-002-AC-4 | The CLI uses ix-cli-kit for shared command/output, configuration and version mechanics, while preserving QSL's numeric exit result without the kit's typed `Outcome` remapping; no unnamed configuration file is read. | Test (TC-009) |
| FR-002-AC-5 | A stdout or stderr write failure returns the internal-failure category and uses QSL's category exit function; a broken pipe ends quietly with the outcome's own status. | Test (TC-009) |

## Dependencies

- QSL ADR-029 CB-1, CB-4 and CB-5; QSL FR-285, FR-286 and FR-287.
- QSpec FR-300 and FR-301 for the CLI output and process contract.
- ix-cli-kit provides shared mechanics only; its current typed `Outcome`
  values cannot represent QSL's category exit codes. The API pass-through
  question is tracked on PLAT-1140; QSL ADR-029 remains authoritative.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
