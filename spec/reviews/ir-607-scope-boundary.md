---
id: SR-2664
title: "scope-boundary review of IR-607 ix-cli-kit exit pass-through spec slice"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-driver PR #42 head; spec/test-cases/TC-009-cli-outcome-and-exit-contract.md, spec/test-cases/TC-018-binary-process-boundary.md, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/assurance/AD-001-driver-architecture.md; ix-cli-kit main FR-013, FR-017, src/exit.rs; QSL main FR-285, ADR-029 CB-4"
review_set: subset
---

## Summary

Ticket: IR-607. The allocation is sound. The kit owns the reserved value and
the conversion. QSL owns the category table: `Category::exit_code` yields only
0, 10, 20, 21, 22 and 30, so `5` is unreachable unless QSL changes. `quire-cli`
owns argv parsing (kit FR-013 ships no parser) and the code-5 policy, which
uses `Category::InternalFailure` (`30`) rather than a second numeric mapping.
Kit FR-017 explicitly leaves process-status choice to the caller. Findings
concern where the test obligations land.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-018 step 4 drives the code-5 fallback in a child `quire` process "through a test-only exit-function seam in the binary". Integration tests run the non-`cfg(test)` binary, so the seam must ship in the release binary unless a cargo feature gates it, and the spec states neither. An env-reachable seam would let any environment force exit 30 plus an invariant diagnostic in production. | spec/test-cases/TC-018-binary-process-boundary.md:82-85 |
| FND-002 | low | TC-009 step 4 re-verifies the kit's own FR-017-AC-1/AC-2 over every `u8` inside the driver. That is the kit's obligation and test. The driver's boundary is the call site, which passes `Category::exit_code` unchanged, and the fallback. | spec/test-cases/TC-009-cli-outcome-and-exit-contract.md:30-36 |

## Verdict

Boundary allocation matches kit FR-013 and FR-017 and QSL FR-285 / ADR-029
CB-4. The binary seam needs a stated gating mechanism.
