---
id: SR-14801
title: "Base checklist — IR-607 stdout-failure spec slice"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver; FR-002 AC-14..AC-16, TC-018 process-boundary procedures and expected results, FR-002/TC-018 tests.md rows"
review_set: subset
---

## Summary

Disposition pass 1 checked the fix on PR #95 exact head f842c17b347879e963cdd018e47d64920e3fcc38 against the two findings from the base review. The FIFO rendezvous now has a bounded nonblocking retry, child-exit polling, and kill/wait cleanup. The BrokenPipe test still requires absence of an `OutputFailure` diagnostic, while FR-002-AC-14 only requires preserving the operation category and not selecting an output-failure category; this leaves that oracle stronger than its acceptance criterion.

## Verdict

**CONDITIONAL** — FND-001 is fixed; FND-002 remains open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The FIFO writer rendezvous has no deadline or cleanup rule; if the child exits before opening the FIFO, the parent can block indefinitely and leave the child running. | spec/test-cases/TC-018-binary-process-boundary.md:98-101 |
| FND-002 | medium | TC-018 requires no `OutputFailure` diagnostic on BrokenPipe, but FR-002-AC-14 only forbids selecting an output-failure category; the test oracle imposes an additional observable behavior. | spec/test-cases/TC-018-binary-process-boundary.md:101-104; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:136 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | The FIFO writer uses bounded nonblocking retries and ensures child cleanup. |
| FND-002 | still-open | AC-14 does not prohibit an `OutputFailure` diagnostic; it only says the CLI does not select an output-failure category. The TC assertion remains stronger than AC-14. |

FND-001 `after_excerpt`:

> Repeatedly attempt to open the source FIFO's writer in nonblocking mode until it succeeds, retrying `ENXIO` and `EINTR` every 10 ms. Use one 10-second deadline from child spawn through child exit. During retries, poll the child; if it exits before the writer opens, fail the test. Once the writer opens, close the parent's stdout reader before writing the fixture bytes to the source FIFO. Require the child to exit within that deadline and compare its status with the independently obtained QSL refusal category exit result (`20`). Require stderr not to contain an `OutputFailure` diagnostic, as specified by FR-002-AC-14. Do not assert stdout contents after closing its reader. On any timeout, premature child exit or other setup/write failure, kill the child if it is still running and wait for it before failing the test.

## Disposition pass 2

Checked PR #95 exact head `daae6528ff280a1febda202efca1c9d55cdad7bd`. FND-002 is fixed: the Behavior statement, AC-14, TC-018 step 5a, and Expected Results now all require that BrokenPipe preserve the QSL category and emit no `OutputFailure` diagnostic. The diff is limited to that contract clarification; the FIFO procedure and matrix status remain unchanged.

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-002 | fixed | Behavior, AC-14, and TC-018 now align on the no-diagnostic requirement. |

FND-002 `after_excerpt`:

> When writing primary command output to stdout fails with `BrokenPipe`, the CLI SHALL retain the category of the completed operation and use that category's normal process exit conversion. It SHALL NOT reclassify the operation as an output failure or emit an `OutputFailure` diagnostic to stderr.
