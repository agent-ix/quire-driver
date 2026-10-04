---
id: SR-044
title: spec-review/delta review of IR-527 PR 19 final delta
type: SpecReview
analysis: base
scope: agent-ix/quire-driver@4623a6dd8ee579c925572c66f5a74881a0077bf5; range 67d8a52da3987a2ae51d9981262aa7bfd688f712..4623a6dd8ee579c925572c66f5a74881a0077bf5;
  spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/assurance/AD-001-driver-architecture.md,
  spec/spec.md, spec/tests.md
review_set: subset
---

## Summary

Ticket: IR-527, PR quire-driver#19. One independent Opus review of the final delta only: `git diff 67d8a52da3987a2ae51d9981262aa7bfd688f712..4623a6dd8ee579c925572c66f5a74881a0077bf5` (commits bcc147e and 4623a6d). The diff touches exactly the four expected files (25 insertions, 20 deletions). TC-018 is correctly indexed in spec.md and TM-001 and is mapped to FR-002-AC-2..AC-4 as additional verification. TC-009 still traces to FR-002-AC-1..AC-5 as the full command-contract case. The IR-606 status and the PR history are accurate. Two completeness defects remain: the AD-001 introduction still states the TC-001 through TC-017 inventory, and the FR-002 Dependencies list does not link TC-018.

## Verdict

CONDITIONAL. Correct FND-001 and FND-002 before accepting the delta. SR-032 and SR-036 FND-001 (fixed, disposition PASS) are not reopened. AD-001:97 and AD-001:197 are unchanged by this delta and still use whole-artifact FR-307 links with no FR-307 AC citation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AD-001's introduction still says the master specification and TM-001 integrate "TC-001 through TC-017". The same delta changes spec.md:169 and tests.md:11-12 to TC-001 through TC-018, and changes AD-001:190 to "TC-009..TC-018". The AD-001 statement about what those two artifacts integrate is now false and contradicts them. Change it to TC-001 through TC-018. | spec/assurance/AD-001-driver-architecture.md:55-56 |
| FND-002 | low | The delta adds TC-018 to the Verification cells of FR-002-AC-2, AC-3 and AC-4 (FR-002:67-69), but FR-002's Dependencies list still links only TC-009. FR-002 through FR-010 each link their verifying TC there. Add a TC-018 link so the FR's verification references match its AC table. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:79 |

## Examined Scope

```yaml
scope:
- id: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md#ac-table
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '| FR-002-AC-1 | ... | Test (TC-009) |

    | FR-002-AC-2 | ... | Test (TC-009, TC-018) |

    | FR-002-AC-3 | ... | Test (TC-009, TC-018) |

    | FR-002-AC-4 | ... | Test (TC-009, TC-018) |

    | FR-002-AC-5 | ... | Test (TC-009) |'
- id: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md#dependencies
  path: spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md
  role: examined
  excerpt: '- [TC-009](../test-cases/TC-009-cli-outcome-and-exit-contract.md).'
- id: spec/assurance/AD-001-driver-architecture.md#intro
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: '[The master specification](../spec.md)

    and [TM-001](../tests.md) integrate FR-001 through FR-010 and TC-001 through

    TC-017.'
- id: spec/assurance/AD-001-driver-architecture.md#lifecycle-composition
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: '| User command to typed operation | ... | FR-002 / TC-009, TC-018 |

    | Compile, check and E4 package emission | ... | FR-001 / TC-001, TC-008; API adaptation delivered by IR-606 |'
- id: spec/assurance/AD-001-driver-architecture.md#risks
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'The merged FR-002..FR-010 and TC-009..TC-018 specify the planned extension.

    | The upstream lifecycle APIs can change across repositories | Driver maintenance | Keep the adapter aligned with the typed QSL/IR/CG APIs; the IR-606 adaptation is merged in PR #14. |

    | CLI, caller limits/engine selection and provider/plugin/cache composition remain planned | IR-607, IR-608, IR-609 | Implement and run TC-009..TC-014 and TC-018 against their owning contracts. |'
- id: spec/spec.md#frontmatter-and-index
  path: spec/spec.md
  role: examined
  excerpt: '- target: ix://agent-ix/quire-driver/TC-018
      type: contains

    | [TC-018](test-cases/TC-018-binary-process-boundary.md) | Preserve the QSL outcome at the binary process boundary |'
- id: spec/spec.md#scope-and-history
  path: spec/spec.md
  role: examined
  excerpt: '- Implementation changes and dependency updates. TC-018 adds the binary process

    boundary case for the merged FR-002 requirement; it does not expand that

    requirement''s scope.

    [TM-001](tests.md) integrates FR-001 through FR-010 and TC-001 through TC-018.

    This reconciliation integrates the merged SP-D1–D9 specifications from driver

    PRs #15 and #17. TC-018 was added in PR #18 to verify the binary boundary for

    FR-002. IR-606 adapted the orchestrator to current upstream lifecycle APIs.'
- id: spec/tests.md#overview-ownership-matrix
  path: spec/tests.md
  role: examined
  excerpt: 'It covers the merged

    TC-001 through TC-018 set.

    | FR-002..FR-009 / TC-009..TC-016, TC-018 | Planned driver CLI and composition tests ... |

    | FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-009, TC-018 | 🚧 Planned |

    | TC-018 | Preserve the QSL outcome at the binary process boundary | Integration | FR-002-AC-2, FR-002-AC-3, FR-002-AC-4 | 🚧 Planned |'
- id: TC-018
  path: spec/test-cases/TC-018-binary-process-boundary.md
  role: context_only
  excerpt: 'Verify FR-002-AC-2 through FR-002-AC-4 at the driver''s root argv and process

    boundary.'
- id: TC-009
  path: spec/test-cases/TC-009-cli-outcome-and-exit-contract.md
  role: context_only
  excerpt: 'Scope: FR-002-AC-1 through FR-002-AC-5.'
- id: prior-reviews
  path: reviews/ir527-reconcile-spec-review-base.md
  role: context_only
  excerpt: 'SR-031..SR-039 (code-review, base, integrity, dependency, evidence, object,
    scope-boundary, matrix-completeness, test-oracle-strength-manual) at a3789aa;
    SR-032 and SR-036 FND-001 fixed in c90131a with disposition PASS.'
```

## Analysis

Method: a manual Opus spec-review of the stated range only. Steps: (1) Confirmed HEAD = 4623a6d, a clean worktree, and that `git diff --stat` for the range touches only the four expected files. (2) Read all nine canonical prior artifacts (SR-031..SR-039), including the SR-032 and SR-036 dispositions. (3) Read the merged TC-018 and TC-009, and compared their declared scope with the FR-002 AC table, the TM-001 FR and TC rows, the spec.md index and frontmatter, and AD-001. (4) Checked the PR and ticket history against `origin/main`: #15 is SP-D5..D9, #17 is SP-D1..D4, #18 (IR-607) adds only TC-018, and #14 (IR-606, "Port driver to current QSL lifecycle") updated FR-001, AD-001 and the src call sites. (5) Ran `quire validate` (quire 0.36.1, engine 0.50.1) on the four files plus TC-018. It exited 0, with only module-catalog DuplicateArchetype warnings. No source tests were run, as instructed.

Confirmed correct: TC-018 is mapped to FR-002-AC-2..AC-4 in FR-002, TM-001 and AD-001. Its own Description says the same, and TM-001's TC-009 row still traces AC-1..AC-5. spec.md says TC-018 does not expand FR-002's scope. TC-018 has a spec.md `contains` edge and an index row. Its Planned status is correct, because no TC-018 or CLI binary implementation exists in src/ or tests/. The AD-001 IR-606 status is correct: "delivered by IR-606" and "merged in PR #14" match main. Dropping the `spine::compile` drift risk row is justified, because PR #14 updated FR-001. The spec.md history line correctly names PRs #15, #17 and #18. TC-018 belongs under the IR-607 CLI owner row. Out of scope: other FRs, implementation, code, tests and G5b.

## Disposition round 1 (SR-045)

Reviewed candidate: `56c1b1d0140d48ae053758288424353f56079074`. Current verdict:
**PASS**. Both findings are fixed and no new findings were introduced.

| FND | Outcome | Fix |
| --- | --- | --- |
| FND-001 | fixed | AD-001 now states that the master specification and TM-001 integrate TC-001 through TC-018. The remaining TC-017 references correctly trace FR-010. |
| FND-002 | fixed | FR-002 Dependencies now links TC-018 as additional verification for AC-2 through AC-4. |

The delta `4623a6d..56c1b1d` is one commit and changes only the two fix sites
and this SR-044 custody artifact. The reviewer posted disposition marker
`SR-044 round=1 disposition=SR-045` on IR-527; readback confirmed it appears
once. Comment: `a5102bf1-c3bf-4343-b784-4a6e4194f982`.
