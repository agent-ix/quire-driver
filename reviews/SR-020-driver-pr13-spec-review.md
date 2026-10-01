---
id: SR-020
title: "Spec review of quire-driver PR #13 (AD-001: QSL-354 merged)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver@d99e47b402b892d9284f9f2e1a75de988e72c9fb; spec/assurance/AD-001-driver-architecture.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/AD-001, type: reviews }
---
# Spec review of quire-driver PR #13 (AD-001: QSL-354 merged)

## Summary

Ticket: IR-324. PR: quire-driver#13, reviewed at d99e47b on base main 8e4932f. The PR is
spec-only. It rewrites the chain-ownership text in System Boundary and in Current state, and
deletes the QSL-354 Open-questions row. I checked it against QSL origin/main b5ef6475, read-only.

Matches QSL main:
- ADR-011 T-13 (:1402): the driver owns the S6b Kani run, the E9 replay through
  `qsl_replay::replay`, and writing the FR-331 terminal record.
- ADR-013 C-09 (:629, :967) and the owner table (:821):
  - CG owns the C-09 map and settles `dispositions`.
  - The map takes the Kani outcome and, for a counterexample, the replay result.
  - The driver is the one writer of `results`.
  - `replay_refused` carries the refusal's catalog code, and a replay fault settles `Failed`.
- QSL PR #550 (QSL-354) is MERGED, and CG PR #214 is still OPEN.
- CG main FR-029 still takes `KaniRunOutcome` alone.
- The only "pending" wording left is CG AD-003.
- The chain is still described as not implemented today, which is true: no code runs a harness
  or calls `replay`.
- Only the QSL-354 Open-questions row was removed.

Also confirmed: no pins, SHAs or version records, no requirement ids minted, and nothing
confidential. `make spec` passes.

## Verdict

**APPROVE (mergeable), three low findings.** Each is a one-line edit; I recommend fixing them
in this PR.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | The chain text says only "Parity is settled inside `replay`" and then names the refusal and fault outcomes. It omits the parity outcomes that T-13 and the C-09 row state: only a reproduced counterexample settles `refuted`, and a replay that disagrees or completes no value settles `inconclusive` with cause `replay_parity`. | spec/assurance/AD-001-driver-architecture.md:172-175 | wrong-requirement |
| FND-002 | low | R-Q1 and R-Q4 still say "matters to the driver once it owns" the execute or terminal-record step, but QSL-354 has merged that ownership. Also, ADR-013 (:629) now specifies `TerminalValue::Inconclusive` with `ReplayParity` and `ReplayRefused` (R-Q1's need, QSL-351 implements it), and ADR-013 (:821) makes the driver the one writer of `results` (R-Q4's need is now the driver's own). Neither row reflects that. | spec/assurance/AD-001-driver-architecture.md:199-200 | wrong-requirement |
| FND-003 | low | Pre-existing on main 8e4932f, not introduced by this PR: the bullet beginning "The driver compiles with an empty `DependencyInput` and refuses any `import`" has lost its first line. Its remainder, "(`missing_import/missing-selection`, FR-001 step 1). Known and stated, not a gap to fix here.", now trails the "Still present" bullet. This PR edits the same section. | spec/assurance/AD-001-driver-architecture.md:164-165 | wrong-requirement |
