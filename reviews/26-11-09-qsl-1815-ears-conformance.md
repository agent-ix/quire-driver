---
id: SR-6015
title: "ears-conformance review of informative root help"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-driver; QSL-1815; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md; spec/test-cases/TC-009-cli-outcome-and-exit-contract.md"
review_set: subset
---

## Summary

Reviewed driver PR #138 at 6fb00108006b3878a1bfbab0715851068d0e6d7b, scoped to the new root-help obligations and TC-009 procedure. No defects found.

## Verdict

**PASS**. The specification defines observable help content and exit/stream behavior, preserves lifecycle boundaries, and matches parser/dispatch availability. This is a specification slice; implementation and executable tests are intentionally subsequent work.

## Review context

No AssuranceProfile exists under this repository’s spec tree; no required enforced review operation applies. Direct review uses the delegated applicable subset: base and EARS, plus the reviewer role’s mandatory code-review of the Markdown diff. No code changed, so Rust and implementation gap-analysis do not apply. The four added SHALL statements have concrete responses and appropriate triggers or ubiquitous form. Existing full-repository grammar warnings are confined to unchanged FR-019. Changed artifacts and full scoped specification validation passed. No new hash, pin, duplicate semantic authority, source scan or bookkeeping machinery is required. The TC procedure is a future test design, not a claim of passing implementation evidence.

## Examined scope

- `FR-002-help-statement-1` (examined), `spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md`: When invoked with no arguments or with `--help`, `-h` or `help` as its sole   argument, the CLI SHALL write root help to stdout without starting a   lifecycle operation.
- `FR-002-help-statement-2` (examined), `spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md`: The root help SHALL include the `quire` invocation syntax, the recognized   lifecycle verb names, the `--format text|json|dot` option with the `inspect`   restriction on `dot`, and the `--version`/`-V` version options.
- `FR-002-help-statement-3` (examined), `spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md`: The root help SHALL distinguish lifecycle verbs that execute operations   from verbs that currently return an unsupported-capability result, and   identify any default text-output restriction for executable verbs.
- `FR-002-help-statement-4` (examined), `spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md`: When root help is written successfully, the CLI SHALL return QSL's success   exit result and write no stderr output. 
- `FR-002-AC-17` (examined), `spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md`: With no arguments or with each of `--help`, `-h` and `help` as the sole argument, the built `quire` binary writes root help to stdout containing invocation syntax, recognized lifecycle verb names, `--format text|json|dot` with `dot` limited to `inspect`, and `--version`/`-V`. The help distinguishes executable verbs from verbs that currently return unsupported-capability results and identifies executable verbs whose default text output is unavailable. Successful help exits with QSL's success result, writes no stderr, and starts no lifecycle operation.
- `TC-009` (examined), `spec/test-cases/TC-009-cli-outcome-and-exit-contract.md`: From an empty working directory, invoke the built `quire` binary with no     arguments and separately with each of `--help`, `-h` and `help` as its     sole argument. Capture stdout, stderr and process status. Verify the help     content specified by FR-002-AC-17, including the distinction between     executable verbs and recognized verbs that return unsupported-capability     results. Compare the advertised commands and format restrictions with the     parser and dispatch surface; verify that these invocations do not read     source input or dispatch a lifecycle operation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
