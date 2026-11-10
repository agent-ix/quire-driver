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

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | fixed | Round 1, head 6bccc06eef8be1879fc079b57766bb45577764a2: canonical two-label-plus-file parse/format and one-request-file run/compile are implemented and used by every valid recovery row. The former implementation grammar and test oracle no longer define validity. |

## Corrected full review (disposition pass 1)

PASS for the complete current PR delta at `6bccc06eef8be1879fc079b57766bb45577764a2` against driver `55f0a001a8ca6c8050e534c2677fe1a6b6f6977d`. FND-002 is fixed; no new finding at any severity. The original PASS and later superseding FAIL above remain historical records, not the current verdict. The lead must retain this disposition in the same PR and run its final aggregate gate before merge.

All 18 changed non-review files were examined, including the manifest and lock graph, README examples, complete CLI handler diff, process-provider test migration, real bound application, fixtures and all changed test oracles. Rust API, implementation, safety/concurrency, wire/persistence and testing references were applied. No owned spec or plan changed, so no additional spec-review method applies. No applicable AssuranceProfile was found. No source or frozen branch was changed by this reviewer.

Canonical SourceOptions retains OsString until label validation and PathBuf construction. Blank-label validation uses SourceIdentity's own order and typed source refusal before file I/O. The prepared request retains owner-authored identity, package name, dependencies and Call/backend. The command dependency is CLI-only git branch main, with normal lock resolution; no root dependency, pin, copied schema or copied implementation was introduced. Clause limits mutate copies of actual Spine/Observation/Scalar bounds and commit only on success. Universally unavailable settings refuse before request I/O; selected-family bounds are validated before execution. Nonempty clause provider configuration is explicitly unsupported before any provider startup; function provider negotiation, signal cleanup and authored package identity remain in their existing path. Clause reports use existing core monitor producers and typed fault/compile/argument projections, without synthesizing an execute Call or inventing item schemas.

Current upstream origin/main FR-287 AC-7/8/9 and TC-1262, FR-286's explicit replacement rule, ADR-029 CB-4 and driver FR-002 were checked. The stale worktree FR-287 raw-package/refusal-envelope wording is not the current authority: the updated owner requires producer-owned quire-outcome/1 documents for success and refusal. Output tests compare actual owner producers byte-for-byte across relative/absolute paths and working directories. Primary stdout is attempted before pending diagnostics; BrokenPipe returns the completed category quietly, other errors choose InternalFailure and identify output failure. Outputless unsupported text keeps its existing contract.

Independent direct execution of the corrected frontend binary passed 8/8 under flock /tmp/titan-cargo.lock, nice/ionice and RUST_TEST_THREADS=2: `/tmp/qsl1813-cli-canonical-review/frontend-tests.log`. Inspected coder evidence `/tmp/qsl1813-cli-canonical-focused.log`: 10+15+23+19+8+2 = 77 passed, zero failed; fmt and test-inclusive Clippy passed in that log and `/tmp/qsl1813-cli-test-inclusive-clippy.log`. No Cargo, metadata subprocess, aggregate, make ci, Kani or proof was run by the reviewer. The final aggregate gate is pending with the lead. Windows unpaired-surrogate test is authored but not executed on Linux; no Windows platform qualification is claimed.

Manual scoped semantic binding review now supports AC-3/5/6/7/8/9 under the current owner contract (AC-6 execution evidence is Linux only). AC-4's fixed compile-ceiling cases remain outside this completion claim; its partial request-byte test intentionally has TC-only tagging. No full QSL-1813 or whole-repository matrix completion is claimed. The earlier computed matrix and coverage remain baseline history, not a recomputed result for this head. Current whole computed audit: NOT COMPLETED in this round by scoped review instruction. Plan completion: not assessed.

## Disposition Data (round 1)

```yaml
{
  "head": "6bccc06eef8be1879fc079b57766bb45577764a2",
  "method": "gap-analysis",
  "verdict": "PASS",
  "clean": true,
  "scope": [
    {
      "id": "FR-287-AC-3",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "`quire run` and `quire compile` with a missing request-file operand, and with an extra operand, exit 20 and write the selected verb's usage to stderr before any file is read: a request path that does not exist is never opened, and no stdout byte is written. `quire parse` and `quire format` with the two labels and no file, and with one extra operand, exit 20 the same way. A `parse` or `format` call with a blank label and a missing file exits 20 with `invalid_source_identity`, cause `blank-label`, the blank label's name in `label`, not a file error."
    },
    {
      "id": "FR-287-AC-4",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "context_only",
      "excerpt": "`--request-bytes` and `--dependent-bytes` set `command`'s `request_bytes` and `dependent_bytes` settings for `quire run` (defaults 1 MiB and 8 MiB per FR-026), have no ceiling, and a run limit refusal names the limit, its configured value and the option that raises it. A run request one byte over each setting refuses naming it, and the same request with the option set to its size runs. `quire compile` has fixed 1 MiB request-file and 8 MiB dependent-file ceilings, accepts no limit options, and refuses an over-limit input naming its setting and maximum without an option hint."
    },
    {
      "id": "FR-287-AC-5",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "An invalid invocation of `parse`, `format`, `run` or `compile` (an unknown option, a missing or extra operand, a malformed option value) exits 20 with no panic, and the same verb run over a valid invocation after it runs normally."
    },
    {
      "id": "FR-287-AC-6",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "A command word or `parse`/`format` label that is not valid UTF-8 (Unix bytes, Windows unpaired surrogates) refuses before the file operand is opened, exit 20, with no panic; a valid source at a file path that is not valid UTF-8 parses."
    },
    {
      "id": "FR-287-AC-7",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "When writing the machine outcome document or formatted text source to stdout fails (stdout bound to `/dev/full`), `quire parse`, `format`, `run` and `compile` exit 30, FR-301's code for tool failure; a partial stdout prefix may remain and is not a complete artifact."
    },
    {
      "id": "FR-287-AC-8",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "When the reader of stdout closes the pipe before an attempted machine-outcome or formatted-source write, `quire parse`, `format`, `run` and `compile` end quietly, write nothing to stderr and exit with the outcome's FR-285 code, not 30. A text rendering that is unsupported and makes no primary stdout write is outside this pipe-write criterion; it retains the driver's declared unsupported-rendering behavior."
    },
    {
      "id": "FR-287-AC-9",
      "path": "quire-spec-language/spec/functional/FR-287-reach-qsl-through-the-driver-cli.md",
      "role": "examined",
      "excerpt": "In machine mode, `quire compile` and `quire run` write exactly their producer-owned FR-286 `quire-outcome/1` document to stdout, for success and refusal alike, with no CLI-added or removed member; no native package bytes, native-run-result document or refusal envelope replace it on a process stream. A compile's document is built only after package construction finishes or refuses. Resolving the same valid request as a relative and an absolute operand from two working directories gives equal outcome bytes and exit code."
    },
    {
      "id": "Cargo.lock",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "# This file is automatically @generated by Cargo."
    },
    {
      "id": "README.md",
      "path": "README.md",
      "role": "examined",
      "excerpt": "# quire-driver"
    },
    {
      "id": "quire-cli/Cargo.toml",
      "path": "quire-cli/Cargo.toml",
      "role": "examined",
      "excerpt": "[package]"
    },
    {
      "id": "quire-cli/src/cli.rs",
      "path": "quire-cli/src/cli.rs",
      "role": "examined",
      "excerpt": "use std::collections::{BTreeMap, BTreeSet};"
    },
    {
      "id": "quire-cli/src/process_provider.rs",
      "path": "quire-cli/src/process_provider.rs",
      "role": "examined",
      "excerpt": "use std::convert::Infallible;"
    },
    {
      "id": "quire-cli/tests/binary_process.rs",
      "path": "quire-cli/tests/binary_process.rs",
      "role": "examined",
      "excerpt": "#[cfg(all(unix, feature = \"tc018-test-adapter\"))]"
    },
    {
      "id": "quire-cli/tests/cli_contract.rs",
      "path": "quire-cli/tests/cli_contract.rs",
      "role": "examined",
      "excerpt": "#![allow("
    },
    {
      "id": "quire-cli/tests/common/requests.rs",
      "path": "quire-cli/tests/common/requests.rs",
      "role": "examined",
      "excerpt": "use std::path::PathBuf;"
    },
    {
      "id": "quire-cli/tests/fixtures/frontend-compile-refused.json",
      "path": "quire-cli/tests/fixtures/frontend-compile-refused.json",
      "role": "examined",
      "excerpt": "{"
    },
    {
      "id": "quire-cli/tests/fixtures/frontend-compile.json",
      "path": "quire-cli/tests/fixtures/frontend-compile.json",
      "role": "examined",
      "excerpt": "{"
    },
    {
      "id": "quire-cli/tests/fixtures/frontend-run-refused.json",
      "path": "quire-cli/tests/fixtures/frontend-run-refused.json",
      "role": "examined",
      "excerpt": "{"
    },
    {
      "id": "quire-cli/tests/fixtures/frontend-run.json",
      "path": "quire-cli/tests/fixtures/frontend-run.json",
      "role": "examined",
      "excerpt": "{"
    },
    {
      "id": "quire-cli/tests/frontend_cli.rs",
      "path": "quire-cli/tests/frontend_cli.rs",
      "role": "examined",
      "excerpt": "use std::process::{Command, Output, Stdio};"
    },
    {
      "id": "quire-cli/tests/library_parity.rs",
      "path": "quire-cli/tests/library_parity.rs",
      "role": "examined",
      "excerpt": "use std::collections::BTreeMap;"
    },
    {
      "id": "src/drive.rs",
      "path": "src/drive.rs",
      "role": "examined",
      "excerpt": "use std::collections::BTreeMap;"
    },
    {
      "id": "src/execute.rs",
      "path": "src/execute.rs",
      "role": "examined",
      "excerpt": "use qsl_foundation::Code;"
    },
    {
      "id": "src/limits.rs",
      "path": "src/limits.rs",
      "role": "examined",
      "excerpt": "use qsl_foundation::{Setting, SettingLimits, SettingName, UsageRefusal};"
    },
    {
      "id": "tests/drive.rs",
      "path": "tests/drive.rs",
      "role": "examined",
      "excerpt": "use std::cell::RefCell;"
    }
  ],
  "findings": [],
  "bindings": [
    {
      "test_id": "missing_and_extra_operands_are_refused_before_any_file_is_read",
      "ac_id": "FR-287-AC-3",
      "trace": "correct"
    },
    {
      "test_id": "invalid_invocations_exit_20_and_the_next_valid_call_runs",
      "ac_id": "FR-287-AC-5",
      "trace": "correct"
    },
    {
      "test_id": "non_utf8_commands_and_labels_refuse_but_raw_file_paths_parse",
      "ac_id": "FR-287-AC-6",
      "trace": "correct"
    },
    {
      "test_id": "stdout_write_failure_exits_30",
      "ac_id": "FR-287-AC-7",
      "trace": "correct"
    },
    {
      "test_id": "closed_stdout_pipe_keeps_the_outcome_exit_code_and_stays_quiet",
      "ac_id": "FR-287-AC-8",
      "trace": "correct"
    },
    {
      "test_id": "request_paths_are_independent_and_documents_equal_the_actual_producers",
      "ac_id": "FR-287-AC-9",
      "trace": "correct"
    },
    {
      "test_id": "clause_requests_use_the_core_monitor_document_and_never_ignore_configured_providers",
      "ac_id": "FR-287-AC-9",
      "trace": "correct"
    }
  ],
  "dispositions": [
    {
      "fnd": "FND-002",
      "outcome": "fixed",
      "after_excerpt": "&[\"parse\", \"issuer\", \"document\", SOURCE, \"--format\", \"json\"],\n&[\"format\", \"issuer\", \"document\", SOURCE],\n&[\"compile\", COMPILE_REQUEST, \"--format\", \"json\"],\n&[\"run\", RUN_REQUEST, \"--format\", \"json\"],"
    }
  ]
}
```
