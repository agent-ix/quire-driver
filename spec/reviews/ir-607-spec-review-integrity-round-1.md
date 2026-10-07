---
id: SR-2665
title: "integrity review of IR-607 fix round 1 new findings"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-driver PR #42 fix round 1 head; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/test-cases/TC-009-cli-outcome-and-exit-contract.md, spec/test-cases/TC-018-binary-process-boundary.md, spec/test-cases/TC-028-cli-package-boundary.md, spec/tests.md, spec/reviews/ir-607-*.md"
review_set: subset
---

## Summary

Ticket: IR-607. These are new findings from the round-1 disposition pass over the
fix commit. The split of FR-002 into AC-1..AC-11 is consistent with the computed matrix:
92 criteria, of which 63 are tagged, 28 untagged and 1 method-without-symbol, against 86, 64, 21 and 1 before.
The new AC-6 fallback and the AC-10 condition are testable, and the TC-018 seam is gated.
The remaining defects are trace and coverage gaps in the newly split criteria, and
divergence in the committed review copies.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-002-AC-8 and AC-9 cite TC-009 and TC-018, but no procedure step in either TC verifies `streams` or `json` module use. TC-009 says those adoptions 'have separate criteria and remain open', and TC-018's Description covers only AC-2..AC-4 and AC-6. Yet tests.md's TC-018 row lists AC-9 and omits AC-8. A coder closing AC-8 or AC-9 has no oracle to satisfy. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:106, spec/test-cases/TC-018-binary-process-boundary.md:15, spec/tests.md:106 |
| FND-002 | low | FR-002-AC-3 is verified by TC-009, TC-018 and TC-028, and TC-028's scope lists AC-3. The rewritten tests.md FR-002 row for AC-1/2/3/5 cites only TC-009 and TC-018, so the requirement-coverage index drops TC-028 for AC-3. | spec/tests.md:47, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:101 |
| FND-003 | low | The committed SR-2662, SR-2663 and SR-2664 copies under spec/reviews/ are not the reviewer's text: backtick formatting was edited in Summary and Findings rows (no content or severity change, and they contain no SHA). They also predate this round's Dispositions. Replace them with the reviewer's latest files verbatim. | spec/reviews/ir-607-integrity.md:13, spec/reviews/ir-607-dependency.md:15-26, spec/reviews/ir-607-scope-boundary.md:14-16 |
