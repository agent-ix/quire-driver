---
id: SR-2660
title: "base checklist review of IR-607 ix-cli-kit exit pass-through spec slice"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #42 head; spec/assurance/AD-001-driver-architecture.md, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/functional/FR-014-separate-the-user-cli-package.md, spec/spec.md, spec/test-cases/TC-005-dependency-edges.md, spec/test-cases/TC-009-cli-outcome-and-exit-contract.md, spec/test-cases/TC-018-binary-process-boundary.md, spec/test-cases/TC-028-cli-package-boundary.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: IR-607. Base checklist over the nine changed documents of the
spec-only slice that stages ix-cli-kit adoption by its
`exit::caller_exit_code` pass-through. IDs are unique, no AC or TC id is added
or removed, every changed AC keeps its TC trace, and the computed matrix is
unchanged before and after (86 criteria: 64 tagged, 21 untagged, 1
method-without-symbol). No new hash, digest, SHA, pin or local path is
introduced. The stale "construct `ExitCode` directly until the kit exposes
pass-through" claim and the PLAT-1140 risk row are gone from FR-002, spec.md
and AD-001. The kit signature, reserved value `5`, error fields and QSL codes
(0, 10, 20, 21, 22, 30) match kit main and QSL FR-285 / ADR-029 CB-4.
Findings are status-honesty defects in the rows and TC prose the slice
rewrote. Each changed document validates with exit 0 under quire 0.36.1
(engine 0.50.1), with seven ambient catalog notices each
(semantic.inline-data-schema, DuplicateArchetype x5, DuplicateInverseEdge).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The rewritten TC-005 step 3 and expected results, and tests.md rows FR-014-AC-2, TC-005 and TC-028, still say the direct quire-cli to quire-plugin-host edge assertion is missing and planned for IR-672. The PR base (IR-672 PR2, #41) already asserts it at tests/dependency_edges.rs:178 and quire-cli/tests/package_boundary.rs:112. | spec/test-cases/TC-005-dependency-edges.md:35-37, spec/test-cases/TC-005-dependency-edges.md:59-62, spec/tests.md:73, spec/tests.md:86, spec/tests.md:109 |
| FND-002 | medium | The tests.md TC-018 row says `quire-cli/tests/binary_process.rs` "checks ... the code-5 invariant fallback". No such test exists: binary_process.rs holds six FR-003/FR-004 tests, and the repo has no ix-cli-kit dependency yet. | spec/tests.md:99 |
| FND-003 | low | AD-001 says "the four other shared modules", but kit main's lib.rs lists six modules: exit, streams, json, version, config and the feature-gated secrets module. AD-001 does not say that `secrets` is out of scope. | spec/assurance/AD-001-driver-architecture.md:208-214 |
| FND-004 | low | TC-009 step 4 asserts the error's `rejected_code` and `reserved_code` "both equal the specified values" without naming them (`5` and `RESERVED_COMMAND_NOT_FOUND`), so the oracle depends on the reader. | spec/test-cases/TC-009-cli-outcome-and-exit-contract.md:35-36 |

## Verdict

Not merge-ready on this lens alone: the medium rows misreport test status in
the rows this slice rewrote.
