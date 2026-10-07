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
without changing its schema or values, except when SIGINT or SIGTERM terminates
an active plugin session as specified below. The driver SHALL use QSL's single
total `Category::exit_code` function as the sole category-to-exit mapping,
except for that signal-terminated invocation.

## Inputs

- A lifecycle command and its options as defined by QSL ADR-029.
- Input documents and source files named by the options, or stdin where the
  verb accepts stdin.
- The typed outcome returned by the operation.

## Outputs

- One QSL-defined `quire-outcome/1` JSON value in machine mode, or the
  selected text rendering, except that a signal-terminated active plugin
  invocation writes no outcome document.
- The exit status returned by QSL's category exit function, except that a
  signal-terminated active plugin invocation re-raises the signal after
  cleanup and has no invented numeric exit result.

## Behavior

- The `quire` binary SHALL be provided by the `quire-cli` workspace package;
  the QSL workspace supplies no competing user binary. The `quire-driver`
  library SHALL NOT compose the downstream `quire-cache` package. FR-014
  specifies the separate CLI package boundary.
- The CLI SHALL preserve the library's QSL-defined `quire-outcome/1` value
  without adding, removing or changing outcome data.
- When an invocation on Linux or macOS is terminated by SIGINT or SIGTERM
  during an active plugin session, after the plugin cleanup specified by
  FR-006 the CLI SHALL
  restore the default signal disposition and re-raise the received signal.
  That terminated invocation SHALL write no `quire-outcome/1` document, and
  the CLI SHALL make no QSL cancellation-cause claim.
- The CLI SHALL use ix-cli-kit for shared command/output, configuration and
  version mechanics. Its typed `Outcome` mapping SHALL NOT replace or remap
  QSL's category exit function.
- The CLI SHALL pass QSL's `Category::exit_code` result to
  `ix_cli_kit::exit::caller_exit_code` and return the resulting exit value
  unchanged.
- If `caller_exit_code` returns `CallerExitCodeError` for a QSL category
  result of `5`, then the CLI SHALL emit an error diagnostic and return
  `Category::InternalFailure.exit_code()` (`30`); it SHALL NOT return `5`.
  This path records a QSL category-mapping invariant failure.
- The `quire-cli` argument parser SHALL remain in `quire-cli`; ix-cli-kit
  defines no argument parser. Adoption of its `streams`, `json`, `config` and
  `version` modules is also required by this requirement and remains planned
  under IR-607 after the exit pass-through slice.
- The CLI SHALL read only input paths named by its arguments and read no
  unnamed configuration file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | The driver binary exposes the command and output surface defined by QSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation. | Test (TC-009) |
| FR-002-AC-2 | For each O-16 category, the process status equals the value returned by QSL's `Category::exit_code`; the CLI has no second category-to-exit mapping. | Test (TC-009, TC-018) |
| FR-002-AC-3 | In machine mode, each command writes one parseable `quire-outcome/1` value equal to the library outcome under QSL's schema; the CLI adds, removes or changes no outcome data. An invocation terminated by SIGINT or SIGTERM during an active plugin session is exempt and writes no outcome document. | Test (TC-009, TC-018, TC-028) |
| FR-002-AC-4 | The CLI uses ix-cli-kit for shared command/output, configuration and version mechanics; it passes QSL's category exit result through `ix_cli_kit::exit::caller_exit_code` without typed `Outcome` remapping. If QSL returns reserved code `5`, the CLI reports the invariant failure and exits with QSL's `Category::InternalFailure.exit_code()` (`30`), never `5`. An invocation on Linux or macOS terminated by SIGINT or SIGTERM during an active plugin session is exempt; after cleanup the CLI restores the default signal disposition and re-raises the signal, without inventing an exit number or QSL cause. | Test (TC-009, TC-018, TC-028) |
| FR-002-AC-5 | The CLI reads only input paths named by its arguments and reads no unnamed configuration file. | Test (TC-009) |

## Dependencies

- QSL ADR-029 CB-1, CB-4 and CB-5; QSL FR-285, FR-286 and FR-287.
- QSpec FR-300 and FR-301 for the CLI output and process contract.
- [ix-cli-kit FR-017](ix://agent-ix/ix-cli-kit/FR-017) defines the caller-owned
  exit-code pass-through; QSL ADR-029 remains authoritative for category
  outcomes and codes. IR-607 adopts the exit module first and retains the
  remaining required kit mechanics as open work.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
- [TC-018](../test-cases/TC-018-binary-process-boundary.md), adding verification for AC-2 through AC-4.
