---
id: SR-016
title: "Gap analysis of quire-driver PR #12 against FR-001 and its test cases"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver@3d72fc6b148db691bc4d45b2e9f39ddffad90abc; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-001..TC-008; spec/spec.md; src/handoff.rs; src/refusal.rs; src/drive.rs; tests/drive.rs; tests/dependency_edges.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-008, type: references }
  - { target: ix://agent-ix/quire-driver/TC-005, type: references }
---
# Gap analysis of quire-driver PR #12 against FR-001 and its test cases

## Summary

Ticket: IR-479. PR: quire-driver#12, reviewed at 3d72fc6. Plan completion:
not assessed.

`quire coverage --scope .` at 3d72fc6: 8/8 rows backed, 0 status lies,
0 untracked symbols, 23/23 tests tagged. No requirement or AC id was removed
(the FR/AC/TC/US id set is identical at base and head). E5 evidence is now
the caller's domain package documents plus QSL's `DefinitionLock` root
feature, which matches IR 0a889f9 `CheckedPackageEvidence` (domain packages,
dependency packages, supported features; the lock's raw-artifact digests are
"not checked against any evidence"). The driver supplies no dependency
packages, consistent with its empty `DependencyInput` (TC-008). The root
feature is load-bearing: removing it fails 5+ drive tests.

## Verdict

**CHANGES REQUESTED (medium).** One new test is not described by the TC it
cites; two low pre-existing gaps noted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new test `tc_008_an_unknown_header_profile_refuses_with_its_cause` cites TC-008, but TC-008 ("A missing import refuses at compile intake") lists only the import test under Tests:, and its procedure and expected results do not mention the Profile refusal (`DriverStage::Compile(SpineStage::Assembly)`, `unknown_profile/unsupported-selection`). Failure scenario: a reader of TC-008 cannot learn that the Profile cause rendering is covered, and a TC rewrite could drop it unnoticed. | spec/test-cases/TC-008-missing-import-refusal.md:9-33; tests/drive.rs:1465-1495 |
| FND-002 | low | Pre-existing, not introduced: the domain-package-document evidence path (now one of only two evidence inputs) has no test; every test passes an empty `packages` map, so dropping the loop stays green. | src/handoff.rs:57-59 |
| FND-003 | low | Pre-existing, not introduced: spec/spec.md relationships and its TC table stop at TC-007; TC-008 is absent. | spec/spec.md:14-27,63-69 |

## Coverage

- AC-1: TC-005, `tc_005_driver_edges_match_the_adr_011_allow_list` (now also asserts one `quire-contract-model` copy). The AC text itself is stale (SR-017 FND-001).
- AC-2: TC-002. AC-3: TC-003. AC-4: TC-001. Unchanged.
- AC-5: TC-004, TC-007, TC-008; new Profile test killed the `compile_cause` Profile-arm mutant.
- AC-6, AC-7, AC-8: TC-006, TC-007; harness assertions changed spelling only.
