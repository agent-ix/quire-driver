---
id: SR-6020
title: "QSL-1813 CLI invocation gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; QSL-1813; PR #141; full local matrix and changed CLI invocation slice"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-spec-language/FR-287, type: references }
  - { target: ix://agent-ix/quire-driver/FR-002, type: references }
---
## Summary

The repository-wide computed traceability audit retains existing gaps. The PR delta implements and tests the selected invocation behavior without introducing a new in-scope gap or claiming the remaining QSL-1813 clauses complete.

## Verdict

FAIL for repository-wide baseline assurance: 71 criteria remain untagged and one criterion is tagged only by an ignored test. PASS for this PR delta: the local criterion/status projection is byte-identical to the merged baseline, and the selected upstream AC-5/tag/implementation chain is manually verified. These existing gaps are not a new merge blocker for this scoped slice.

## Assurance Context

No AssuranceProfile applies to this diff. AD-001 is proposed architecture context; no enforced-profile acceptance is claimed. The owning upstream FR-287/TC-1262 and driver FR-002 were read. The repository's direct CLI/driver boundary and existing category ownership remain unchanged.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Plan completion: not assessed

Computed quire matrix completed successfully over the entire driver repository: 241 criteria, 160 tagged, 71 untagged, nine method-without-symbol, and one tagged-by-ignored-test. A separate read-only origin/main worktree produced the same local id/status projection, compared byte-for-byte. Matrix computation exits zero in its non-strict report mode; that is not a claim that all criteria are covered.

Quire coverage also completed successfully in non-strict mode: 184 backed of 274 reported obligations/rows, including 241 criteria. It reports 12 untracked symbols. The counts describe the repository, not completion of QSL-1813. Existing planned driver/provider/platform and lifecycle criteria are retained; this review adds no artificial tags or copied requirements to make them green.

FR-287 and TC-1262 are owned by QSL, not minted as local driver requirements. The local matrix therefore does not independently establish the upstream criterion's coverage; the source-level attribute and actual owning text were inspected manually. TC-1262 explicitly requires the exact TC/AC attribute used here. AC-5's invalid invocation, category, diagnostic specificity and per-case valid recovery are exercised by real binary process tests. The arity change covers only the run bare-extra-operand subcase of AC-3.

The source changes remain inside the existing typed frontend boundary and are specified by the inherited driver dependency on FR-287. No new unowned API, semantics, stub, compatibility route or duplicated owner vocabulary was found. No coverage credit is added for the unimplemented AC-3 label/request clauses, AC-6 filesystem/Windows clauses, AC-7 refusal envelope, AC-8 full-mode quiet refusal or AC-9 request/raw-output semantics. Those remain explicit open work, not findings newly introduced by this PR.

## Evidence

Application: quire-cli 0.1.0. The frozen candidate and both changed files were inspected. Coder logs record fmt/test-inclusive Clippy passing and six frontend tests passing. The reviewer independently ran the existing frontend test binary under the shared thermal lock, nice/ionice, two test threads and the worktree target: six passed, zero failed, exit 0; /tmp/qsl1813-cli-review/focused-tests.log. No Cargo command, aggregate gate, Kani, make ci or source mutation was run by the reviewer. The code lead owns the final aggregate gate.

The coder's red control restores the old run error and fails specifically on the missing usage prefix for run missing.qsl yes extra; restored code passes. Logs /tmp/qsl1813-cli-red-control.log and /tmp/qsl1813-cli-green-control.log were inspected.

## Review Limitations

This is a scoped manual semantic check of the upstream AC-5 and changed arity behavior, authorized in the brief. It is not a semantic qualification of every local criterion, nor a platform portability claim. No aggregate gate or final-head merge authorization is claimed.

## Reviewer Data

```json
{
  "clean": true,
  "scope": [
    {
      "id": "FR-287-AC-5",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "An invalid invocation of `parse`, `format`, `run` or `compile` (an unknown option, a missing or extra operand, a malformed option value) exits 20 with no panic, and the same verb run over a valid invocation after it runs normally."
    },
    {
      "id": "FR-287-AC-3",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "`quire run` and `quire compile` with a missing request-file operand, and with an extra operand, exit 20 and write the selected verb's usage to stderr before any file is read: a request path that does not exist is never opened, and no stdout byte is written."
    },
    {
      "id": "TC-1262",
      "path": "quire-spec-language/spec/test-cases/TC-1262-the-quire-frontend-parses-encodes-writes-and-exits.md",
      "role": "examined",
      "excerpt": "Tag the tests `#[trace(\"TC-1262\", \"<AC id>\")]`."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "excerpt": "Each verb SHALL call its corresponding typed QSL or driver operation.\nThe CLI SHALL not make semantic decisions based on diagnostic text."
    },
    {
      "id": "quire-cli/src/cli.rs",
      "path": "quire-cli/src/cli.rs",
      "role": "examined",
      "excerpt": "const USAGE: &str = \"usage: quire run <source-path> <function> [parameter=value ...] [--limit name=value] [--engine interpreter|aot|jit] [--provider-config <path>] [--format json|text]\";"
    },
    {
      "id": "quire-cli/tests/frontend_cli.rs",
      "path": "quire-cli/tests/frontend_cli.rs",
      "role": "examined",
      "excerpt": "#[trace(\"TC-1262\", \"FR-287-AC-5\")]\n#[test]\nfn invalid_invocations_exit_20_and_the_next_valid_call_runs() {"
    }
  ],
  "findings": [],
  "bindings": [
    {
      "test_id": "invalid_invocations_exit_20_and_the_next_valid_call_runs",
      "ac_id": "FR-287-AC-5",
      "trace": "correct"
    }
  ]
}
```

## Post-review contract correction

The initial scoped PASS and the initial correct-binding assessment above are preserved as review history, but are superseded by this correction. I incorrectly treated the implemented path-only frontend as the validity baseline. The declared contract, not existing behavior, defines a valid recovery invocation.

Driver FR-002 explicitly depends on QSL FR-287 and requires the ADR-029 surface. FR-287-AC-3 names the two parse/format labels and the file separately; AC-6 distinguishes label encoding from filesystem-path encoding. TC-1262 requires blank-label/missing-file and canonical arity cases. ADR-029 CB-1 delegates the thin frontend to the driver; it does not grant an alternate path-only grammar. Driver PR #17 already deferred to the QSL surface. PR #54 introduced path-only handlers; its FR-002 edits add text-rendering behavior, not a reviewed replacement for the label grammar.

The newly tagged test uses parse SOURCE --format json and format SOURCE as valid recovery calls. FormatOptions::parse and ParseOptions::parse require exactly one positional, so a canonical invocation with both labels and the file is rejected at the arity branch. Six passing tests and the run-usage red control establish behavior for that implementation surface, not the missing canonical recovery path.

## New findings (post-review contract check)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The new AC-5 binding and completion claim use path-only parse/format recovery calls instead of the declared two-label-plus-file invocations. The parsers require exactly one positional, so the test can pass while canonical valid calls are rejected. Existing implementation grammar is not an alternate contract. Port canonical handling and exercise canonical recovery before claiming AC-5 complete. | quire-cli/tests/frontend_cli.rs:77-121; quire-cli/src/cli.rs:462-490 |

FND-001 remains the original no-findings placeholder, not a real defect. FND-002 is the first real finding; its identifier continues the file's existing sequence without rewriting that placeholder.

## Current Verdict

FAIL for the PR delta: FND-002 is open. PR #141 is draft and is not mergeable under this review. Removing or narrowing a coverage claim alone would avoid false credit but would not complete the requested canonical CLI port. Canonical tests and handling are required; no user grammar choice is needed because the target is already declared.

The full AC-3/6/7/8/9 gaps stay open, including labels and source identity, non-UTF-8 paths, request/output envelopes and stream behavior. A lack of current owner payload projection is an implementation prerequisite to address at its owner, not permission to use the old grammar. The earlier local matrix counts and baseline status comparison remain historical local evidence; they do not establish upstream AC-5 completion.

This correction used only source, specification and Git-history inspection. No Cargo, matrix/coverage probe, test execution, source edit or push was performed in this pass.

## Current Reviewer Data

```json
{
  "clean": false,
  "scope": [
    {
      "id": "FR-287-AC-5",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "An invalid invocation of `parse`, `format`, `run` or `compile` (an unknown option, a missing or extra operand, a malformed option value) exits 20 with no panic, and the same verb run over a valid invocation after it runs normally."
    },
    {
      "id": "FR-287-AC-3",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "`quire parse` and `quire format` with the two labels and no file, and with one extra operand, exit 20 the same way. A `parse` or `format` call with a blank label and a missing file exits 20 with `invalid_source_identity`, cause `blank-label`, the blank label's name in `label`, not a file error."
    },
    {
      "id": "FR-287-AC-6",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "context_only",
      "excerpt": "A command word or `parse`/`format` label that is not valid UTF-8 (Unix bytes, Windows unpaired surrogates) refuses before the file operand is opened, exit 20, with no panic; a valid source at a file path that is not valid UTF-8 parses."
    },
    {
      "id": "TC-1262",
      "path": "quire-spec-language/spec/test-cases/TC-1262-the-quire-frontend-parses-encodes-writes-and-exits.md",
      "role": "examined",
      "excerpt": "Run each verb with a missing and with an extra operand, a nonexistent request or source path, a blank label with a missing file, an unknown option and a malformed option value; record exit code, stdout and stderr, and whether the path was opened."
    },
    {
      "id": "FR-002-AC-1",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "The driver binary exposes the command and output surface defined by QSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation."
    }
  ],
  "findings": [
    {
      "fnd": "FND-002",
      "severity": "high",
      "confidence": "high",
      "method": "gap-analysis",
      "check": "trace",
      "artifact_id": "FR-287-AC-5",
      "related_ids": [
        "FR-287-AC-3",
        "FR-287-AC-6",
        "TC-1262",
        "FR-002-AC-1"
      ],
      "path": "quire-cli/tests/frontend_cli.rs",
      "lines": "77-121",
      "excerpt": "An invalid invocation of `parse`, `format`, `run` or `compile` (an unknown option, a missing or extra operand, a malformed option value) exits 20 with no panic, and the same verb run over a valid invocation after it runs normally.",
      "finding": "The new AC-5 binding and completion claim use path-only parse/format recovery calls instead of the declared two-label-plus-file invocations. The parsers require exactly one positional, so the test can pass while canonical valid calls are rejected. Existing implementation grammar is not an alternate contract. Port canonical handling and exercise canonical recovery before claiming AC-5 complete.",
      "test_id": "invalid_invocations_exit_20_and_the_next_valid_call_runs",
      "claimed_ac": "FR-287-AC-5",
      "correct_ac": null
    }
  ],
  "bindings": [
    {
      "test_id": "invalid_invocations_exit_20_and_the_next_valid_call_runs",
      "ac_id": "FR-287-AC-5",
      "trace": "wrong"
    }
  ]
}
```


