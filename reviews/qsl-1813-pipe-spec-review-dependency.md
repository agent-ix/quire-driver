---
id: SR-6025
title: "QSL-1813 primary-output BrokenPipe spec-review/dependency review"
type: SpecReview
analysis: dependency
review_set: subset
scope: "agent-ix/quire-driver; QSL-1813; PR #142; FR-002 and TC-018 primary-output BrokenPipe specification"
relationships:
  - { target: ix://agent-ix/quire-driver/FR-002, type: references }
  - { target: ix://agent-ix/quire-driver/TC-018, type: references }
---
## Summary

Independent spec-review/dependency review of the two-document PR #142. No introduced or in-scope defect found; this is specification readiness, not implementation qualification.

## Assurance Context

No AssuranceProfile document is present in the repository spec tree. AD-001 and the master requirements were read as architecture context; no enforced profile review or human acceptance is claimed. The code-lead selected the direct base, EARS, integrity and dependency analyses for this narrow rule change.

## Verdict

PASS — mergeable as an owning-spec prerequisite after artifact custody and Luna's merge decision. This does not qualify source PR #141, complete QSL-1813, or claim full FR-287-AC-8 coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

QSL FR-287 supplies the quiet-pipe policy and FR-285/ADR-029 supply the category conversion; driver FR-002 owns process adaptation and TC-018 owns boundary verification. Existing kit stream handling supplies write/flush disposition. Logical order is owning policy and typed operation result, then local disposition rule, then future implementation and binary qualification. Production requests must remain valid under the owning canonical input contract, not be justified by current path-only implementation. Outputless text qualification depends on future primary rendering or an owning mode clarification and is explicitly not claimed here. Source-identity refusal projection is separate, not an invented prerequisite or local workaround for this I/O specification. No new dependency edge, cycle, enablement API or alternative outcome schema is introduced.

## Verification and Limits

Both edited documents and the repository specification validate. The engine's only two grammar warnings concern unchanged FR-019; neither changed document has a grammar warning. Diff check and frozen-head/clean-worktree checks pass. No edited artifact is embedded by a Rust test. No Cargo, Kani, CBMC, runtime test, aggregate gate, source edit or push was performed. No repository-wide matrix or implementation PASS is claimed for this spec-only review.

## Reviewed Units

```json
{
  "clean": true,
  "scope": [
    {
      "id": "FR-002-AC-14",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "For a `BrokenPipe` while writing or flushing primary stdout, the process status equals the completed operation's QSL category exit result and stderr is empty, including for a refused operation with nonempty diagnostics when stdout is writable. No output-failure category is selected and no `OutputFailure` diagnostic is emitted."
    },
    {
      "id": "FR-002-AC-13",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "When text rendering is unavailable for parse, check, compile or run, the CLI returns the operation-specific QSL `UnimplementedCapability` result, writes no stdout, and does not execute the requested lifecycle operation."
    },
    {
      "id": "FR-002-AC-15",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "If writing or flushing primary stdout returns an I/O error other than `BrokenPipe`, the process exits with QSL's `Category::InternalFailure` exit result. The criterion makes no assertion that stdout is empty or complete."
    },
    {
      "id": "FR-002-AC-16",
      "path": "spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md",
      "role": "examined",
      "excerpt": "If a non-`BrokenPipe` primary stdout write or flush fails and stderr is writable, stderr contains an `OutputFailure` diagnostic that identifies the failed output."
    },
    {
      "id": "TC-018",
      "path": "spec/test-cases/TC-018-binary-process-boundary.md",
      "role": "examined",
      "excerpt": "Repeat the closed-reader control with the production binary for valid and refused JSON requests to `parse`, `format`, `compile` and `run`, and for successful formatted-text output."
    },
    {
      "id": "FR-287-AC-8",
      "path": "ix://agent-ix/quire-spec-language/FR-287",
      "role": "context_only",
      "excerpt": "When the reader of stdout closes the pipe before the output is written, `quire parse`, `format`, `run` and `compile` end quietly, write nothing to stderr and exit with the outcome's FR-285 code, not 30."
    },
    {
      "id": "ADR-029",
      "path": "ix://agent-ix/quire-spec-language/ADR-029",
      "role": "context_only",
      "excerpt": "Every verb writes one JSON outcome document to stdout in machine mode."
    }
  ],
  "findings": []
}
```

