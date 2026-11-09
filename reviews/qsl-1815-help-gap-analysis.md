---
id: SR-6018
title: "Scoped gap review of informative root help"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; QSL-1815; PR 140; FR-002-AC-17, TC-009 and informative-root-help implementation/test bindings"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-driver/FR-002", type: references}
  - {target: "ix://agent-ix/quire-driver/TC-009", type: references}
---

## Summary

Scoped traceability, reverse-gap and oracle review of informative root help at frozen head 90f3041717ca507f3b608f895beed009ae7d57f6. The actual computed matrix was generated and the relevant row compared with the source criterion, trace attributes, tests and production path. One inherited, directly owning criterion-table defect was confirmed; no production stub or unowned behavior was found.

## Verdict

CONDITIONAL until FND-001 is corrected and the computed row is read back. Runtime code and meaningful tagged tests implement the intended complete help obligation, but the current computed criterion is truncated by the requirement's unescaped table delimiters. This is a scoped review, not a repository-wide assurance or QSL-1815 completion claim.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Unescaped pipes in the owning AC-17 table cell truncate the computed statement after `--format text` and parse `json` as its verification method instead of `Test`. Escape the two pipes in that table cell without changing the requirement, then verify the computed full statement and Test method. The malformed row predates this implementation PR but directly owns its new trace bindings. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:153 |

## Coverage

Plan completion: not assessed

No plan was supplied and no applicable AssuranceProfile was found. This scoped analysis examines the changed behavior and its direct owning units; it does not declare the rest of the repository fully tested. No Test Matrix was authored or edited.

- FR-002-AC-17, examined: "With no arguments or with each of `--help`, `-h` and `help` as the sole argument, the built `quire` binary writes root help to stdout containing invocation syntax, recognized lifecycle verb names, `--format text|json|dot` with `dot` limited to `inspect`, and `--version`/`-V`. The help distinguishes executable verbs from verbs that currently return unsupported-capability results and identifies executable verbs whose default text output is unavailable. Successful help exits with QSL's success result, writes no stderr, and starts no lifecycle operation."
- TC-009 step 11, examined: selectors and invocation from an empty working directory, actual command/format availability, no source input or lifecycle dispatch.
- FR-002-AC-11/12, context_only: retained version controls, not new help behavior.

Computed evidence: quire matrix --scope /home/peter/dev/worktrees/QSL1815-help-code-codex1 --format json completed exit 0. matrix.json records FR-002-AC-17 as tagged by all three new process tests, but its statement ends with `--format text` and its method is `json`; the raw criterion's verification cell is Test. The source row contains literal unescaped pipes inside the inline-code output selector. matrix.stderr contains installed catalog warnings; it does not explain or repair that malformed row.

The three FR-002-AC-17 bindings are semantically correct against the complete intended criterion: root_help_selectors_print_the_same_informative_help_without_an_operation; root_help_identifies_current_command_availability_and_text_restrictions; informative_help_does_not_replace_invalid_command_or_format_refusals. No tag is removed and no ignored or fabricated test supplies this evidence. Their code uses real child processes, full stdout comparisons, content checks and real unsupported/refusal controls. Direct source inspection confirms the help branch cannot reach lifecycle dispatch. Existing version and parser checks remain successful.

No copied upstream semantics, compatibility layer, stub, gate weakening or coverage suppression was introduced. The lead's full package gate remains pending; package_boundary must use the documented quiet-window exception because it shells Cargo metadata. No full repository semantic audit or plan-completeness assessment was performed.
