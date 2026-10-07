---
id: SR-2661
title: "EARS conformance review of IR-607 ix-cli-kit exit pass-through spec slice"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-driver PR #42 head; spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md, spec/functional/FR-014-separate-the-user-cli-package.md"
review_set: subset
---

## Summary

Ticket: IR-607. EARS review of the requirement statements this slice adds or
edits. The FR-014 dependency SHALL and the FR-002 pass-through SHALL are
well-formed ubiquitous statements. The new reserved-code clause correctly uses
the If/then unwanted-behaviour form. Findings: FR-002's Description, Outputs
and AC-2 were not updated for the new exception, so they contradict it. The
fallback's diagnostic has no stated stream or format. AC-4 is compound. One
Behavior bullet carries status prose rather than a requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Under FR-002's Description and Outputs, the process status is the `Category::exit_code` result, with only the signal exception. AC-2 says the status "equals the value returned by QSL's `Category::exit_code`". The new reserved-code clause and AC-4 add a second exception (returned `5` exits `30`), so AC-2 contradicts AC-4 for that case, and TC-018's "each child status is exactly the code returned" contradicts its own step 4. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:33-35, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:49-51, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:89, spec/test-cases/TC-018-binary-process-boundary.md:97 |
| FND-002 | medium | The phrases "emit an error diagnostic" and "reports the invariant failure" name no stream, format or content. They also do not say what machine mode writes to stdout. AC-3 requires one outcome document equal to the library outcome, whose category would then disagree with exit 30. The diagnostic oracle in TC-009 step 4 and TC-018 step 4 cannot fail on content. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:73-76, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:91 |
| FND-003 | medium | FR-002-AC-4 bundles three obligations: the four open kit-module adoptions, the exit pass-through with its 5-to-30 fallback, and the signal exemption. The exit-only slice can therefore never close its own part, and the tests.md status cannot say which clause is verified. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:91 |
| FND-004 | low | One SHALL bullet carries non-requirement prose: "ix-cli-kit defines no argument parser" is a claim about another repo, and "remains planned under IR-607 after the exit pass-through slice" is status. That prose belongs in Dependencies and tests.md. | spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md:77-80 |

## Verdict

FR-014's changed statements conform. FR-002 needs the exception stated
consistently and its diagnostic made specific.
