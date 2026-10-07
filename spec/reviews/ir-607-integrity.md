---
id: SR-2662
title: "integrity review of IR-607 ix-cli-kit exit pass-through spec slice"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-driver PR #42 head; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/functional/FR-014-separate-the-user-cli-package.md, spec/assurance/AD-001-driver-architecture.md, spec/spec.md, spec/tests.md, spec/test-cases/TC-028-cli-package-boundary.md; bindings in quire-cli/tests/package_boundary.rs and quire-cli/src/cli.rs"
review_set: subset
---

## Summary

Ticket: IR-607. Consistency of the changed criteria with the existing tagged
tests and with each other. The computed matrix (quire matrix) is identical
before and after the PR: no criterion id is added or removed. However, an
existing test bound to FR-014-AC-2 asserts the negation of the edge that the
amended AC now requires. The AC-4 binding is overbroad. FR-002 and AD-001
disagree on whether config adoption is conditional.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The amended FR-014-AC-2 requires a quire-cli to ix-cli-kit edge, but the test tagged to it, `cargo_target_and_dependencies_belong_to_quire_cli`, asserts `!dependencies.contains("ix-cli-kit")`. tests.md calls the kit edge merely "unasserted", and the computed matrix reports AC-2 as tagged by a test that checks the opposite. The IR-607 code slice will fail this test, and no spec text records that it must be inverted. | quire-cli/tests/package_boundary.rs:78, quire-cli/tests/package_boundary.rs:113, spec/tests.md:73, spec/functional/FR-014-separate-the-user-cli-package.md:63 |
| FND-002 | medium | `accepts_shared_format_without_resolving_config_or_output` is tagged FR-002-AC-4, but it only checks that `--format json` parses. It exercises no kit module, exit pass-through, 5-to-30 fallback or signal clause, yet the computed matrix reports the edited AC-4 as tagged. | quire-cli/src/cli.rs:393-395, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:91 |
| FND-003 | medium | FR-002 Behavior and AC-4 require kit configuration mechanics unconditionally, and spec.md says `config` adoption "is also required by FR-002". AD-001 makes `config` conditional ("if the CLI introduces configuration") and records that the CLI has no config reader, while FR-002-AC-5 forbids reading unnamed config. The config obligation is therefore either vacuous or impossible to close. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:67-68, spec/spec.md:129-131, spec/assurance/AD-001-driver-architecture.md:211-214 |

## Verdict

Not merge-ready: FND-001 leaves the spec and the tagged test in direct
contradiction, with tests.md misdescribing the state.
