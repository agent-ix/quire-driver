---
id: SR-2663
title: "dependency review of IR-607 ix-cli-kit exit pass-through spec slice"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-driver PR #42 head; spec/functional/FR-014-separate-the-user-cli-package.md, spec/assurance/AD-001-driver-architecture.md, spec/functional/FR-002-expose-driver-cli-and-outcome-contract.md; context Cargo.toml, quire-cli/Cargo.toml, deny.toml, CLAUDE.md; ix-cli-kit main Cargo.toml, README.md, spec FR-013, FR-016, FR-017"
review_set: subset
---

## Summary

Ticket: IR-607. The slice adds one edge, from quire-cli to ix-cli-kit, and
keeps it out of the driver closure. None of the driver's upstreams (QSL, CG,
IR, quire-exact) depend on the kit, so the closure exclusion is achievable.
`deny.toml` `[sources.allow-org] github = ["agent-ix"]` already admits the kit's
git source. The repo convention for first-party git dependencies is
`branch = "main"`, with exact commits only in `Cargo.lock`. The PR adds no pins,
SHAs or local paths and states no conflicting convention. Two concrete blockers
for the edge are unrecorded in the slice's AD-001 risk row.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | ix-cli-kit main declares `serde = "1.0.229"`, a caret floor, while every workspace manifest pins `serde = "=1.0.228"` and `Cargo.lock` holds 1.0.228. Adding the specified edge makes resolution fail unless the serde pins move across all four manifests. AD-001's risk row and FR-014 do not record this. | spec/assurance/AD-001-driver-architecture.md:276, spec/functional/FR-014-separate-the-user-cli-package.md:49-50, quire-cli/Cargo.toml:25 |
| FND-002 | medium | The kit README tells consumers to declare ix-cli-kit with `rev = "<40-hex sha>"`, and kit FR-016 also says pin by revision. That contradicts this repo's `CLAUDE.md` (no new SHAs or pins) and its `branch = "main"` plus Cargo.lock convention. The slice specifies the edge but not its declaration form, so a coder who follows the upstream doc will add a SHA pin. | spec/functional/FR-014-separate-the-user-cli-package.md:49-50, spec/assurance/AD-001-driver-architecture.md:216-217 |

## Verdict

The edge direction and closure exclusion are correct. Before the code slice,
the risk row should record the serde floor conflict and the branch-on-main
declaration form.
