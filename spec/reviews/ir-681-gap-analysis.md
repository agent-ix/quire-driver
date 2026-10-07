---
id: SR-2761
title: "gap analysis of quire-driver PR 43 (IR-681) against its ticket criteria"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver PR 43 (IR-681), reviewed revision named in the tracker marker; CLAUDE.md, quire-cli/src/signals.rs; computed Test Matrix row FR-002-AC-7 (quoin matrix); spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md (FR-002-AC-7, context)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: reviews
---
# SR-2761: gap analysis of IR-681

## Summary

Ticket: IR-681. This is a planless gap analysis (plan completion: not
assessed). The PR changes no production code and no spec. The analysis maps
each IR-681 acceptance criterion to the diff, and checks the computed Test
Matrix for the criterion that these signal tests exercise.

- **AC 1** (report raw code/status, signal, stdout and stderr before
  assertions). Met. `child_failure` (`signals.rs:220-230`) formats the
  `ExitStatus` Debug, `code()`, `signal()` and lossy stdout/stderr. That string
  is built before every post-exit assertion and on every panic path in both
  lifecycle tests.
- **AC 2** (module docs and repository test docs state the runner
  requirement). Met. See `signals.rs:7-9` and `:192-193`, and `CLAUDE.md:31-32`
  next to the `make` gate commands. The wording "unsandboxed runner" covers
  "outside the Codex sandbox". Whether the docs explain how to recognise a
  sandbox failure is recorded in SR-2760 FND-004.
- **AC 3** (assertions and timing unchanged). Met for assertions and
  constants. The moved timing check is recorded in SR-2760 FND-003.
- **AC 4** (macOS and Linux `make ci`). Not assessed here. Cargo was not run
  by this reviewer. A macOS pass is reported, and the Linux normal-permission
  gate is queued.

The computed matrix (`quoin matrix`) reports FR-002-AC-7 as `untagged` with
"no evidence is bound". This gap predates the PR: neither signal test carries a
trace tag. `spec/tests.md` already tracks it as partial (🚧), and IR-681 does
not cover it, so it is not a finding against this PR. No new code without an
owning requirement, no stubs and no coverage inflation were found.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

There are no gaps between IR-681's criteria and the diff beyond the low items
already recorded in SR-2760. AC 4 depends on the queued Linux gate. FR-002-AC-7
remains untagged in the computed matrix, which predates this PR and is outside
IR-681's scope.
