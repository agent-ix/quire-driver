---
id: SR-022
title: "IR-607 spec-review/base review of TC-018"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver@63aa9bcd8493d10666ece38bee1a65bfe6dac84d; spec/test-cases/TC-018-binary-process-boundary.md; FR-002 context; ticket IR-607"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/TC-018
    type: reviews
---
# IR-607: spec-review/base

## Summary

Ticket: IR-607. PR: quire-driver#18. Scope: git diff origin/main...HEAD, only TC-018. FR-002 was read from PR #17’s merged tree and matches the current file. TC-009 is context only. Single independent reviewer; no delegated review.

Ran the Quoin spec-review base checklist through its direct skill path. No AssuranceProfile or required review_selection exists in spec/. IDs and the verifies relationship are structurally valid. The additional boundary test complements TC-009; it does not replace TC-009 or claim the entire lifecycle/category matrix. The four nonzero cases are useful adverse examples, but their real-process arrangement is unspecified.

## Examined scope

The following block records every examined or contextual unit, including clean units. Repeated TC/FR identifiers refer to sections of the same artifact, not newly authored artifact IDs.

```yaml
{
  "scope": [
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "relationship",
      "excerpt": "  - target: ix://agent-ix/quire-driver/FR-002\n    type: verifies"
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-1.1",
      "excerpt": "Verify FR-002-AC-2 through FR-002-AC-4 at the driver's root argv and process\nboundary. QSL ADR-029 supplies the outcome value and exit semantics; this case\nchecks that the binary preserves them while using ix-cli-kit shared mechanics."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-2.1",
      "excerpt": "1. Inspect the binary entry point and root argv dispatch. Confirm that it invokes\n   the selected typed operation, uses the applicable ix-cli-kit output, config\n   and version helpers, and has one conversion from QSL's\n   `Category::exit_code` result to `std::process::ExitCode`. Check that no path\n   converts the category through ix-cli-kit's `Outcome` or a second numeric\n   category mapping."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-2.2",
      "excerpt": "2. Run the built `quire` process with controlled typed outcomes whose QSL\n   categories return 20, 21, 22 and 30. For each run, derive the expected status\n   by calling QSL's `Category::exit_code` on that outcome's category, then\n   compare it to the child process status."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-2.3",
      "excerpt": "3. Repeat those runs in machine mode. Parse stdout as exactly one JSON value and\n   compare it to the QSL serialization of the same typed `quire-outcome/1`\n   value; check that the binary added no wrapper or fields and kept diagnostics\n   on stderr."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-3.1",
      "excerpt": "Test location: planned driver binary integration tests and entry-point review."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-4.1",
      "excerpt": "- Root argv selects the typed operation. The binary uses ix-cli-kit helpers for\n  their applicable shared mechanics while retaining its own root argv boundary."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-4.2",
      "excerpt": "- Each child status is exactly the code returned by QSL's exit function,\n  including 20, 21, 22 and 30. There is one pass-through to\n  `std::process::ExitCode` and no ix-cli-kit `Outcome` conversion."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "unit": "paragraph-4.3",
      "excerpt": "- Machine stdout contains exactly the unchanged QSL `quire-outcome/1` value\n  for each run; diagnostics remain on stderr."
    },
    {
      "id": "FR-002-AC-1",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "acceptance-criterion",
      "excerpt": "The driver binary exposes the command and output surface defined by QSL ADR-029 LC-1, CB-1 and CB-5, and dispatches each command to its typed operation."
    },
    {
      "id": "FR-002-AC-2",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "unit": "acceptance-criterion",
      "excerpt": "For each O-16 category, the process status equals the value returned by QSL's `Category::exit_code`; the CLI has no second category-to-exit mapping."
    },
    {
      "id": "FR-002-AC-3",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "unit": "acceptance-criterion",
      "excerpt": "In machine mode, each command writes one parseable `quire-outcome/1` value equal to the library outcome under QSL's schema; the CLI adds, removes or changes no outcome data."
    },
    {
      "id": "FR-002-AC-4",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "unit": "acceptance-criterion",
      "excerpt": "The CLI uses ix-cli-kit for shared command/output, configuration and version mechanics, while preserving QSL's numeric exit result without the kit's typed `Outcome` remapping."
    },
    {
      "id": "FR-002-AC-5",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "acceptance-criterion",
      "excerpt": "The CLI reads only input paths named by its arguments and reads no unnamed configuration file."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-1.1",
      "excerpt": "The driver SHALL build the one user-facing `quire` binary. Its lifecycle\ncommands, output options, multi-item severity and process behavior SHALL\nconform to QSL ADR-029 LC-1, CB-1, CB-4 and CB-5."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-2.1",
      "excerpt": "Each verb SHALL call its corresponding typed QSL or driver operation.\nThe CLI SHALL not make semantic decisions based on diagnostic text."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.1",
      "excerpt": "In machine mode, each verb SHALL write exactly one QSL-defined\n`quire-outcome/1` JSON value to stdout and preserve the typed library outcome\nwithout changing its schema or values. The driver SHALL use QSL's single\ntotal `Category::exit_code` function as the sole category-to-exit mapping."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-4.1",
      "excerpt": "- A lifecycle command and its options as defined by QSL ADR-029."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-4.2",
      "excerpt": "- Input documents and source files named by the options, or stdin where the\n  verb accepts stdin."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-4.3",
      "excerpt": "- The typed outcome returned by the operation."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.1",
      "excerpt": "- One QSL-defined `quire-outcome/1` JSON value in machine mode, or the\n  selected text rendering."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.2",
      "excerpt": "- The exit status returned by QSL's category exit function."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-6.1",
      "excerpt": "- The `quire` binary SHALL be built from the driver repository; the QSL\n  workspace supplies no competing user binary."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-6.2",
      "excerpt": "- The CLI SHALL preserve the library's QSL-defined `quire-outcome/1` value\n  without adding, removing or changing outcome data."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-6.3",
      "excerpt": "- The CLI SHALL use ix-cli-kit for shared command/output, configuration and\n  version mechanics. Its typed `Outcome` mapping SHALL NOT replace or remap\n  QSL's category exit function. Until ix-cli-kit exposes a raw exit-code\n  pass-through, the CLI SHALL construct `std::process::ExitCode` directly\n  from QSL's returned code at the process boundary."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-6.4",
      "excerpt": "- The CLI SHALL read only input paths named by its arguments and read no\n  unnamed configuration file."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-8.1",
      "excerpt": "- QSL ADR-029 CB-1, CB-4 and CB-5; QSL FR-285, FR-286 and FR-287."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-8.2",
      "excerpt": "- QSpec FR-300 and FR-301 for the CLI output and process contract."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-8.3",
      "excerpt": "- ix-cli-kit provides shared mechanics only; its current typed `Outcome`\n  values cannot represent QSL's category exit codes. The API pass-through\n  question is tracked on PLAT-1140; QSL ADR-029 remains authoritative."
    },
    {
      "id": "FR-002",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "context_only",
      "unit": "paragraph-8.4",
      "excerpt": "- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md)."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-1.1",
      "excerpt": "Verify that the driver CLI exposes the specified verbs, preserves typed\noutcomes in `quire-outcome/1` documents and uses QSL's single exit function."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-2.1",
      "excerpt": "Scope: FR-002-AC-1 through FR-002-AC-5."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.1",
      "excerpt": "1. Invoke every listed verb with its valid format option, including `inspect`\n   with `dot`."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.2",
      "excerpt": "2. Invoke an unknown verb, option, profile and provider; capture stdout,\n   stderr, exit status and whether a lifecycle stage ran."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.3",
      "excerpt": "3. Exercise representative outcomes for every O-16 category, including\n   `undefined`, a pending monitor clause, an inconclusive proof and a\n   multi-item outcome containing multiple exit codes. Compare each process\n   status to QSL's category exit function."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.4",
      "excerpt": "4. Verify shared command/output, configuration and version mechanics use\n   ix-cli-kit. Inspect the exit boundary with an outcome category that the\n   kit's typed `Outcome` cannot represent and confirm the QSL-returned code is\n   passed through unchanged."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.5",
      "excerpt": "5. Run one operation in JSON mode and compare the serialized value with the\n   typed library outcome. Repeat with stdout/stderr write failure and a broken\n   pipe."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-3.6",
      "excerpt": "6. Run from a directory containing an unnamed configuration file and verify\n   that the CLI does not read it."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-4.1",
      "excerpt": "Test location: planned driver CLI integration tests."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.1",
      "excerpt": "- Valid verbs and format options reach their corresponding operations."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.2",
      "excerpt": "- Invalid arguments are refused before any stage with QSL's refusal category\n  and name the refused argument."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.3",
      "excerpt": "- Each category and multi-item result exits with the exact code selected by\n  QSL's single exit function; no second mapping changes the result."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.4",
      "excerpt": "- Shared CLI mechanics use ix-cli-kit without mapping QSL categories through\n  the kit's typed `Outcome` values."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.5",
      "excerpt": "- JSON mode emits one QSL-defined `quire-outcome/1` value equal to the typed\n  library outcome. A stream write failure returns the internal-failure\n  category through QSL's exit function; a broken pipe ends quietly with the\n  outcome's code."
    },
    {
      "id": "TC-009",
      "path": "spec/test-cases/TC-009-cli-outcome-and-exit-contract.md",
      "role": "context_only",
      "unit": "paragraph-5.6",
      "excerpt": "- An unnamed configuration file is not read."
    }
  ]
}
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Step 2 supplies no argv/input arrangement or test seam that makes the real built quire process return the same controlled typed outcome used by the oracle. A reviewer cannot reproduce the four requested runs or establish that expected and observed values concern the same operation. Specify concrete input/argv fixtures or the real-process outcome-control mechanism and how the independently obtained QSL value corresponds to each invocation. | spec/test-cases/TC-018-binary-process-boundary.md:25-28 |

## Verdict

**CONDITIONAL**. Resolve the medium finding before accepting the stated test procedure.

## Validation and limitations

TC-018 and the repository spec glob passed Quire validation. Catalog validation reports catalog ok (10). Ambient Quire discovery emits inline-data-schema, DuplicateArchetype and DuplicateInverseEdge warnings; the same warnings occur on unchanged TC-009. All review artifacts are validated separately at their explicit scratch paths. This is a spec review, with no implementation or executable tests claimed. The CLI workflow wrapper was not started: the spec-review skill permits this direct path when no required profile applies.

Custody: the lead or fix-round coder must copy this file to reviews/ir607-spec-review-base.md in a later authorized commit; the reviewer does not commit it.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | TC-018 copies the upstream QSL inv example into a second repository-owned spec fixture: lines 37-44 explicitly source its exact bytes from QSL FR-100-AC-5/TC-451 and repeat the inv division body. This violates the no-copying-code-or-specs rule and gives the example two maintenance homes. Remove the copied upstream example; consume producer-owned published test data or define independently authored driver-owned test input rather than reworking the copy. | spec/test-cases/TC-018-binary-process-boundary.md:37-44 |

Source check: read QSL main FR-100 and TC-451 via GitHub API, read-only. FR-100-AC-5 defines `inv(x: Int[0, 9]): Boolean { 1 / x > 0 }`; TC-018 reproduces that example as `inv using v(x: Digit): Boolean pure { 1 / x > 0 }` and explicitly names its upstream origin. This is a copied example, even though the local declaration adds profile/type syntax. No other reviewer artifacts were read or used.

## Dispositions

Round 1. Candidate and fix commit: 72ae9d01d62f089c657d675747bfc43be6c245f1. Original finding comment: f5eccfe9-0d8c-4577-98f7-8b8339e31d60.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 72ae9d01d62f089c657d675747bfc43be6c245f1; Committed step 2 now defines a test-only typed-check adapter, four case selectors, real root argv/input handling, an isolated request observation, and independent parent construction of the QSL expected outcome/status. The adapter cannot replace serialization, stream writes or exit conversion and supplies no numeric exit code. Lines 25-67 resolve the missing arrangement and request/oracle correspondence. |
| FND-002 | still-open | Newly introduced by the fix commit: the copied QSL inv example remains in TC-018 lines 37-44. No removal or producer-owned dependency has landed. Remove the copy before accepting this candidate. |

### Round 1 evidence and current verdict

Committed step 2 now defines a test-only typed-check adapter, four case selectors, real root argv/input handling, an isolated request observation, and independent parent construction of the QSL expected outcome/status. The adapter cannot replace serialization, stream writes or exit conversion and supplies no numeric exit code. Lines 25-67 resolve the missing arrangement and request/oracle correspondence.

**FAIL**. Both original review findings are closed, but the newly introduced FND-002 remains open; the candidate is not spec-review clean. No executable test result is claimed in this spec-only pass. The original Summary, Findings and Verdict remain as the historical review record.

```yaml
round: 1
reviewed: 72ae9d01d62f089c657d675747bfc43be6c245f1
original_finding_comment_id: f5eccfe9-0d8c-4577-98f7-8b8339e31d60
method: spec-review/base
dispositions:
- fnd: FND-001
  outcome: fixed
  fix_sha: 72ae9d01d62f089c657d675747bfc43be6c245f1
  after_excerpt: "   For each case, spawn the test-built `quire` with argv\n   `[\"\
    check\", \"<absolute path to inv.native>\", \"--format\", \"text\"]`\n   and that\
    \ case's `TC018_CASE`, with a fresh `TC018_REQUEST_PATH`. Capture\n   stdout,\
    \ stderr and the child status. Require the observation file to exist\n   and contain\
    \ the selected case and exact fixture bytes and path; a missing\n   observation\
    \ or mismatched request fails the test. Independently obtain the\n   typed result\
    \ in the parent using the table's operation or constructor over\n   that recorded\
    \ request, including its source identity, profile and limits,"
  reason: Committed step 2 now defines a test-only typed-check adapter, four case
    selectors, real root argv/input handling, an isolated request observation, and
    independent parent construction of the QSL expected outcome/status. The adapter
    cannot replace serialization, stream writes or exit conversion and supplies no
    numeric exit code. Lines 25-67 resolve the missing arrangement and request/oracle
    correspondence.
- fnd: FND-002
  outcome: still-open
  reason: 'Newly introduced by the fix commit: the copied QSL inv example remains
    in TC-018 lines 37-44. No removal or producer-owned dependency has landed. Remove
    the copy before accepting this candidate.'
new_findings:
- fnd: FND-002
  severity: high
  confidence: high
  method: spec-review/base
  check: other
  artifact_id: TC-018
  related_ids:
  - FR-002-AC-2
  - FR-002-AC-3
  path: spec/test-cases/TC-018-binary-process-boundary.md
  lines: 37-44
  excerpt: "   Write `inv.native` in a temporary directory with these exact UTF-8\
    \ bytes\n   (including a final newline), from QSL FR-100-AC-5 and its TC-451 example:\n\
    \n   ```text\n   language \"ix:native\" edition \"1-draft\";\n   profile v = \"\
    quire.value.complete/v1\";\n   type Digit = Int[0, 9];\n   function inv using\
    \ v(x: Digit): Boolean pure { 1 / x > 0 }"
  finding: 'TC-018 copies the upstream QSL inv example into a second repository-owned
    spec fixture: lines 37-44 explicitly source its exact bytes from QSL FR-100-AC-5/TC-451
    and repeat the inv division body. This violates the no-copying-code-or-specs rule
    and gives the example two maintenance homes. Remove the copied upstream example;
    consume producer-owned published test data or define independently authored driver-owned
    test input rather than reworking the copy.'
bindings: []
all_original_findings_closed: true
spec_review_clean: false
verdict: FAIL
scope:
- id: TC-018
  path: spec/test-cases/TC-018-binary-process-boundary.md
  role: examined
  excerpt: "1. Inspect the binary entry point and root argv dispatch. Confirm that\
    \ it invokes\n   the selected typed operation, uses the applicable ix-cli-kit\
    \ output, config\n   and version helpers, and has one conversion from QSL's\n\
    \   `Category::exit_code` result to `std::process::ExitCode`. Check that no path\n\
    \   converts the category through ix-cli-kit's `Outcome` or a second numeric\n\
    \   category mapping."
- id: TC-018
  path: spec/test-cases/TC-018-binary-process-boundary.md
  role: examined
  excerpt: "   For each case, spawn the test-built `quire` with argv\n   `[\"check\"\
    , \"<absolute path to inv.native>\", \"--format\", \"text\"]`\n   and that case's\
    \ `TC018_CASE`, with a fresh `TC018_REQUEST_PATH`. Capture\n   stdout, stderr\
    \ and the child status. Require the observation file to exist\n   and contain\
    \ the selected case and exact fixture bytes and path; a missing\n   observation\
    \ or mismatched request fails the test. Independently obtain the\n   typed result\
    \ in the parent using the table's operation or constructor over\n   that recorded\
    \ request, including its source identity, profile and limits,"
- id: TC-018
  path: spec/test-cases/TC-018-binary-process-boundary.md
  role: examined
  excerpt: "   Write `inv.native` in a temporary directory with these exact UTF-8\
    \ bytes\n   (including a final newline), from QSL FR-100-AC-5 and its TC-451 example:\n\
    \n   ```text\n   language \"ix:native\" edition \"1-draft\";\n   profile v = \"\
    quire.value.complete/v1\";\n   type Digit = Int[0, 9];\n   function inv using\
    \ v(x: Digit): Boolean pure { 1 / x > 0 }"
- id: TC-018
  path: spec/test-cases/TC-018-binary-process-boundary.md
  role: examined
  excerpt: "- Each child status is exactly the code returned by QSL's exit function,\n\
    \  including 20, 21, 22 and 30. There is one pass-through to\n  `std::process::ExitCode`\
    \ and no ix-cli-kit `Outcome` conversion.\n- Machine stdout contains exactly the\
    \ unchanged QSL `quire-outcome/1` value\n  for each run. The known `ill_typed`\
    \ diagnostic retains its structured fields\n  in that value, and its complete\
    \ nonempty rendered line appears on stderr\n  and is absent from stdout."
```
