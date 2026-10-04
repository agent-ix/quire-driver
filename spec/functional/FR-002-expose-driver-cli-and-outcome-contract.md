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

In machine mode, each verb SHALL write exactly one `quire-outcome/1` JSON
document to stdout. The driver SHALL obtain the process exit status through
QSL's single total exit function for the outcome's `Category`. The driver SHALL
use that function as its only category-to-exit mapping.

## Inputs

- A lifecycle verb and its options.
- Input documents and source files named by the options, or stdin where the
  verb accepts stdin.
- The typed outcome returned by the operation.

## Outputs

- One `quire-outcome/1` JSON document in machine mode, or the selected text
  rendering.
- The exit status returned by QSL's category exit function.

## Behavior

- The `quire` binary SHALL be built from the driver repository; the QSL
  workspace supplies no competing user binary.
- The outcome document SHALL carry `format`, `operation`, `last_stage`,
  `category`, `items`, `diagnostics` and `artifacts` as defined by QSL's
  outcome contract.
- When rendering a library outcome, the CLI SHALL preserve typed category and
  cause data and SHALL add or remove no members.
- For a multi-item outcome, the CLI SHALL use the most severe item code under
  QSL's defined severity order.
- If the CLI receives an unknown verb, option, profile or provider, then the
  CLI SHALL refuse before a lifecycle stage runs with exit status 20 and a
  diagnostic naming the refused argument.
- If writing to stdout or stderr fails, then the CLI SHALL use exit status 30.
- If the CLI encounters a broken pipe, then the CLI SHALL end quietly with
  the outcome's own exit status.
- The CLI SHALL read only input paths named by its arguments and read no
  unnamed configuration file.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-002-AC-1 | The `quire` binary accepts exactly the listed lifecycle verbs, each accepts `--format json|text`, and `inspect` accepts `dot`; an unknown verb, option, profile or provider is refused before any stage with exit 20 and a diagnostic naming the argument. | Test (TC-009) |
| FR-002-AC-2 | For each O-16 category, the CLI exit status equals QSL's single category exit function: success 0; violation and undefined 10; refusal 20; unsupported 21; incomplete 22; inconclusive proof/analyze 22 and pending monitor clause 0; internal failure 30. Multi-item outcomes use QSL's severity order. | Test (TC-009) |
| FR-002-AC-3 | In JSON mode, each verb writes exactly one parseable `quire-outcome/1` document with the operation's typed category, causes, items, diagnostics and artifacts unchanged from the library outcome. | Test (TC-009) |
| FR-002-AC-4 | A stdout or stderr write failure exits 30, except that a broken pipe ends quietly with the outcome's exit status; no unnamed configuration file is read. | Test (TC-009) |

## Dependencies

- QSL ADR-029 CB-1, CB-4 and CB-5; QSL FR-285, FR-286 and FR-287.
- QSpec FR-300 and FR-301 for the CLI output and process contract.
- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).
