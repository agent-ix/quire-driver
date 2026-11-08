---
id: SR-6012
title: "gap analysis of quire-driver PR 131 (QSL-1813) against FR-287-AC-3 and AC-5..AC-9 (TC-1262)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver PR 131 (QSL-1813), head 42371f61a65189590869a1788bd38d084e38ac3b; quire-cli/tests/frontend_cli.rs; QSL spec FR-287-AC-3, AC-5..AC-9 and TC-1262 as on quire-spec-language origin/main; PR body"
review_set: subset
---
# SR-6012: gap analysis of QSL-1813

## Summary

Ticket: QSL-1813. Planless; plan completion was not assessed. Criteria were
read on quire-spec-language `origin/main`. The dispatch brief said commit
2d6ccbcac removed FR-287-AC-3..9 from the live spec. Measured, that is wrong:
2d6ccbcac added them, and AC-3 and AC-5..AC-9 are on QSL `origin/main` today
with the same wording as commit 98a9115bf. Only AC-4, which is out of scope
here, was reworded later. So the trace tags resolve to live criteria.

Coverage per criterion:

- **AC-3:** tests arity refusal before I/O and empty stdout. It does not test
  usage text or labels.
- **AC-5:** fully covered.
- **AC-6:** covers the command word and a non-UTF-8 operand on Unix only. The
  Windows half is not covered, and the path clause is unmet.
- **AC-7:** covers the success outputs only.
- **AC-8:** covers it, with the ordering caveat in SR-6011 FND-001.
- **AC-9:** covers the relative and absolute comparison for successful runs
  only. Raw package bytes and the refusal envelope are not covered.

The PR body states the label, non-UTF-8 path and raw-bytes gaps honestly.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | PR body omits the AC-3 usage-text gap: the driver writes a one-line refusal message, not the verb's usage, and the test asserts only that the verb name appears | quire-cli/tests/frontend_cli.rs:36-62 |
| FND-002 | low | PR body says "See ticket report" but QSL-1813 had no comments at review time, so the referenced report does not exist | PR body |
| FND-003 | low | AC-7 names the refusal envelope among the outputs whose write failure exits 30; the test covers only success outputs, and the gap is unstated | quire-cli/tests/frontend_cli.rs:154-180 |
| FND-004 | low | AC-8 says nothing is written to stderr; the refused-outcome case asserts only no OutputFailure because the driver writes a QslDiagnostic for refusals, and this divergence is unstated | quire-cli/tests/frontend_cli.rs:213-218 |
| FND-005 | low | AC-6 Windows unpaired-surrogate half is untested (cfg(unix) only) and unstated | quire-cli/tests/frontend_cli.rs:124-152 |
| FND-006 | low | TC-1262 step 6 compares a refused request across the two working directories; only successful compile/run are compared, unstated | quire-cli/tests/frontend_cli.rs:222-259 |

## Verdict

Changes requested on the PR body and test scope. State each unmet clause
(FND-001, FND-003..FND-006) or cover it. No vendored fixtures or spec text.
Every unmet clause follows from the driver's current surface. Its `run` and
`compile` take a source and a function rather than a FR-026/027 request file,
and it writes no usage text and has no labels. Those are driver behaviour
gaps against FR-287 and belong on a follow-up ticket, not in this test PR.
