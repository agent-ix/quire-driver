---
id: SR-018
title: "Spec review of quire-driver PR #11 (AD-001 driver architecture)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver@85ecacb5e1159f265a9249d64c131e4ce4d76e95; spec/assurance/AD-001-driver-architecture.md; spec/spec.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/FR-001, type: references }
  - { target: ix://agent-ix/quire-driver/TC-005, type: references }
---
# Spec review of quire-driver PR #11 (AD-001 driver architecture)

## Summary

Ticket: IR-324 (seam ADs). Overlap ticket: IR-351. PR: quire-driver#11, reviewed at 85ecacb.
Spec-only: the PR adds `spec/assurance/AD-001-driver-architecture.md` and links it from
`spec/spec.md`. Checked: base checklist, the ArchitectureDescription structure, integrity,
traceability of every cited id, and consistency with the driver at origin/main (c0b2f31) and
with the open catch-up PR quire-driver#12 (IR-479, head 3d72fc6). Code-consistency findings are
in the companion gap analysis (SR-019).

`make spec` (`quire validate 'spec/**/*.md'`) passes; only environment warnings print.
`git merge-tree` of this head with #12 merges cleanly as text.

The review began at 218f51e. There, the Open-questions row recommended against the model-copy
assertion. FR-001-AC-1 and TC-005 already make that assertion on main, and #12 strengthens it to
one copy, so the row contradicted both. At 85ecacb the row assigns the one-copy gate to IR-479
and #12, D-7 includes it, and dependency spelling is left to IR-351 and IR-479. That conflict is
resolved, so it is not a finding here.

Verified true: the System Boundary, Views, Decisions and Risks sections are present. Every
relationship target exists: FR-001, US-001, QSL ADR-011/012/013, CG AD-001, IR AD-001. QSL
ADR-011 T-13 (:1402) still ends the driver at generation, as the AD says. FB-11 (:571) and the
§6.1 driver row (:719) exist. ADR-013 T-7 exists. QSL-354 exists (In Progress; QSL PR #550),
and so do IR-479, IR-351 and IR-347. The execute -> replay -> terminal-record chain is stated
the same way in each place it appears: owned by the driver, not implemented, QSL-354 pending,
no cycle (driver downstream of QSL and CG), and no map of the driver's own. The routed gaps
have QSL as owner and are stated as needs, not decisions. IR-351 overlap is handled by
reference. No pins, SHAs, version records, compatibility layers, vendoring, research content or
quire-research internals appear.

## Verdict

**APPROVE AS DRAFT, CHANGES REQUESTED (medium).** No high findings. One medium: the
terminal-map sentence cites an unnamed, unmerged AD as if it were current fact. The rest are
low.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | "The driver calls CG's terminal-value map, whose terminal step takes the Kani outcome and the replay settlement ... as the evidence-chain seam AD defines them" is stated as present fact about an unnamed artifact. That AD is CG AD-003, which is unmerged (quire-contract-codegen#214). On CG main, FR-029's only input is one `KaniRunOutcome` plus its SUCCESS count, and FR-030 maps the outcome alone. Name CG AD-003 as pending (relationship once merged) and say FR-029/FR-030 take the outcome alone today. | spec/assurance/AD-001-driver-architecture.md:160-162 | wrong-requirement |
| FND-002 | low | The routed-gap rows are out of step with their routing state. An IR-324 Linear comment (2026-10-01 10:48, posted under the owner's account and headed "QSL review"; untrusted, not verified) says "R-Q4 is rejected for now (no byte consumer yet)", and its F5 puts the cross-repo test in quire-integration, not in QSL. R-Q6 still says "in QSL". QSL-351 (R-Q1) and QSL-353 (emit-to-IR-reader corpus) exist and are not cited. | spec/assurance/AD-001-driver-architecture.md:185-187 | wrong-requirement |
| FND-003 | low | D-4, D-5 and D-6 carry no "existing:" marker, although TC-004 (PackageIdMismatch, HandoffRefused, UnknownNode), TC-003 step 5 (generation never called when nothing is routed) and TC-007 (UnknownNode) already cover them. | spec/assurance/AD-001-driver-architecture.md:124-127 | wrong-requirement |
| FND-004 | low | "QSL ADR-011 T-13 states only the QSL half" is inaccurate. T-13 (ADR-011:1402) also lists the CG steps: E7 CG `negotiate_*` and CG generation. What T-13 lacks is the chain after generation, not the CG side. | spec/assurance/AD-001-driver-architecture.md:35 | wrong-requirement |
| FND-005 | low | Cites "QSL ADR-011 section 6.1 driver row" for an allow-list that includes `quire-contract-model` and `thiserror`. The §6.1 driver row (ADR-011:719) names only "the QSL layer crates and CG". The model crate and thiserror come from FR-001-AC-1 and TC-005. The citation is inherited from FR-001:121. | spec/assurance/AD-001-driver-architecture.md:77-78 | wrong-requirement |
