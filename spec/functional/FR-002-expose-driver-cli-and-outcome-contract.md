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
  - target: ix://agent-ix/quire-spec-language/FR-298
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-301
    type: depends_on
  - target: ix://agent-ix/ix-cli-kit/FR-008
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
without changing its schema or values, except when the CLI reports a reserved
exit-code mapping invariant failure or SIGINT/SIGTERM terminates an active
plugin session as specified below. The driver SHALL use QSL's single total
`Category::exit_code` function as the category-to-exit mapping; AC-6 specifies
the fallback when that function returns reserved value `5`.

## Inputs

- A lifecycle command and its options as defined by QSL ADR-029.
- Input documents and source files named by the options, or stdin where the
  verb accepts stdin.
- The typed outcome returned by the operation.

## Outputs

- One QSL-defined `quire-outcome/1` JSON value in machine mode, or the
  selected text rendering, except that a reserved-code mapping invariant
  failure or signal-terminated active plugin invocation writes no outcome
  document or text result. Until QSL supplies typed outcome text rendering,
  parse, check, compile and run text requests return the typed unsupported
  refusal specified in AC-13 and write no stdout.
- The exit status returned by QSL's category exit function, except that a
  reserved-code mapping invariant failure returns code `30`, and a
  signal-terminated active plugin invocation re-raises the signal after
  cleanup without an invented numeric exit result.

## Behavior

- The `quire` binary SHALL be provided by the `quire-cli` workspace package;
  the QSL workspace supplies no competing user binary. The `quire-driver`
  library SHALL NOT compose the downstream `quire-cache` package. FR-014
  specifies the separate CLI package boundary.
- The CLI SHALL preserve the library's QSL-defined `quire-outcome/1` value
  without adding, removing or changing outcome data, except in the two
  invocation cases specified by AC-6 and AC-7.
- When an invocation on Linux or macOS is terminated by SIGINT or SIGTERM
  during an active plugin session, after the plugin cleanup specified by
  FR-006 the CLI SHALL
  restore the default signal disposition and re-raise the received signal.
  That terminated invocation SHALL write no `quire-outcome/1` document, and
  the CLI SHALL make no QSL cancellation-cause claim.
- The CLI SHALL pass QSL's `Category::exit_code` result to
  `ix_cli_kit::exit::caller_exit_code`; it SHALL NOT remap accepted values
  through the kit's typed `Outcome`.
- If `caller_exit_code` returns `CallerExitCodeError` because QSL returned
  reserved result `5`, then the CLI SHALL write a plain-text diagnostic to
  stderr that identifies reserved code `5` and fallback code `30`. In this
  path, the CLI SHALL write no result or outcome document to stdout.
- If `caller_exit_code` returns `CallerExitCodeError` because QSL returned
  reserved result `5`, then the CLI SHALL return
  `Category::InternalFailure.exit_code()` (`30`).
- The diagnostic for the reserved result `5` SHALL not attribute a QSL cause
  to the mapping failure.
- When the CLI writes results and diagnostics, it SHALL use ix-cli-kit's
  `streams` module for stream discipline.
- When the CLI writes machine-mode outcomes, it SHALL use ix-cli-kit's `json`
  module for canonical JSON encoding.
- When writing primary command output to stdout fails with `BrokenPipe`, the
  CLI SHALL retain the category of the completed operation and use that
  category's normal process exit conversion. It SHALL NOT reclassify the
  operation as an output failure or emit an `OutputFailure` diagnostic to
  stderr.
- When writing or flushing primary command output to stdout fails with an I/O
  error other than `BrokenPipe`, the CLI SHALL select
  `Category::InternalFailure` for process exit conversion and SHALL make a
  best-effort attempt to write an `OutputFailure` diagnostic to stderr that
  identifies the failed output. The CLI SHALL NOT promise that stdout is empty;
  a failed write can occur after some output bytes were written.
- When the user requests text output for parse, check, compile or run while
  QSL's typed outcome renderer is unavailable, the CLI SHALL refuse with QSL
  `Code::UnimplementedCapability` and SHALL NOT run the operation. Format
  continues to use QSL's formatter text.
- When the CLI resolves configuration values, it SHALL use ix-cli-kit's
  `config` module.
- When the user invokes `quire --version`, the CLI SHALL report the semantic
  version declared by the `quire-cli` Cargo package.
- When `quire-cli` runs its built-binary version-agreement check, it SHALL
  compare `CARGO_PKG_VERSION` with the version reported by the built
  `quire --version` command through
  `ix_cli_kit::version::Agreement::command`.
- The CLI SHALL keep its argument parser in the `quire-cli` package.
- The CLI SHALL read only input paths named by its arguments and read no
  unnamed configuration file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | The driver binary exposes the command and output surface defined by QSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation. | Test (TC-009) |
| FR-002-AC-2 | For each QSL category whose exit result is not reserved code `5`, the process status equals the value returned by QSL's `Category::exit_code`; no second category-to-exit mapping is applied. Reserved result `5` is governed by AC-6. | Test (TC-009, TC-018) |
| FR-002-AC-3 | In machine mode, each command writes one parseable `quire-outcome/1` value equal to the library outcome under QSL's schema; the CLI adds, removes or changes no outcome data. The mapping-invariant and signal exceptions are specified in AC-6 and AC-7. | Test (TC-009, TC-018, TC-028) |
| FR-002-AC-4 | The CLI passes QSL's `Category::exit_code` result through `ix_cli_kit::exit::caller_exit_code` and preserves each accepted numeric value without mapping it through the kit's typed `Outcome`. | Test (TC-009, TC-018) |
| FR-002-AC-5 | The CLI reads only input paths named by its arguments and reads no unnamed configuration file. | Test (TC-009) |
| FR-002-AC-6 | If QSL's category exit function returns reserved value `5`, the CLI writes a plain-text stderr diagnostic naming reserved code `5` and fallback code `30`, writes no text result or `quire-outcome/1` document to stdout, and returns `Category::InternalFailure.exit_code()` (`30`) without attributing a QSL cause. | Test (TC-009, TC-018) |
| FR-002-AC-7 | When a Linux or macOS invocation receives SIGINT or SIGTERM during an active plugin session, the CLI cleans up, restores the default disposition, and re-raises that signal without writing a result or outcome document or inventing a numeric exit value or QSL cause. | Test (TC-028) |
| FR-002-AC-8 | When the CLI writes results and diagnostics, it uses ix-cli-kit's `streams` module for stream discipline. | Test (TC-009, TC-018) |
| FR-002-AC-9 | When the CLI writes machine-mode outcomes, it uses ix-cli-kit's `json` module for canonical JSON encoding. | Test (TC-009, TC-018) |
| FR-002-AC-10 | When the CLI resolves configuration values, it uses ix-cli-kit's `config` module. | Test (TC-009) |
| FR-002-AC-11 | When invoked with `--version`, the CLI reports the semantic version declared by the `quire-cli` Cargo package. | Test (TC-009) |
| FR-002-AC-12 | The `quire-cli` built-binary version-agreement check uses `ix_cli_kit::version::Agreement::command` to compare `CARGO_PKG_VERSION` with the output of the built `quire --version` command and requires agreement. | Test (TC-009) |
| FR-002-AC-13 | When text rendering is unavailable for parse, check, compile or run, the CLI returns the operation-specific QSL `UnimplementedCapability` result, writes no stdout, and does not execute the requested lifecycle operation. | Test (TC-009) |
| FR-002-AC-14 | For a `BrokenPipe` while writing or flushing primary stdout, the process status equals the completed operation's QSL category exit result; no output-failure category is selected and no `OutputFailure` diagnostic appears on stderr. | Test (TC-018) |
| FR-002-AC-15 | If writing or flushing primary stdout returns an I/O error other than `BrokenPipe`, the process exits with QSL's `Category::InternalFailure` exit result. The criterion makes no assertion that stdout is empty or complete. | Test (TC-018) |
| FR-002-AC-16 | If a non-`BrokenPipe` primary stdout write or flush fails and stderr is writable, stderr contains an `OutputFailure` diagnostic that identifies the failed output. | Test (TC-018) |

## Dependencies

- QSL ADR-029 CB-1, CB-4 and CB-5; QSL FR-285, FR-286 and FR-287.
- QSL FR-298 defines typed rendering; QSL-600/QSL-601 own its pending
  `qsl-inspect` implementation. AC-13 specifies the driver's behavior until
  that renderer is available.
- QSpec FR-300 and FR-301 for the CLI output and process contract.
- [ix-cli-kit FR-017](ix://agent-ix/ix-cli-kit/FR-017) defines the caller-owned
  exit-code pass-through; QSL ADR-029 remains authoritative for category
  outcomes and codes. IR-607 adopts the exit module first and retains the
  remaining required kit mechanics as open work.
- [ix-cli-kit FR-008](ix://agent-ix/ix-cli-kit/FR-008) defines the built-binary
  version agreement used by AC-12.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
- [TC-018](../test-cases/TC-018-binary-process-boundary.md), adding verification for AC-2 through AC-4 and AC-6.
