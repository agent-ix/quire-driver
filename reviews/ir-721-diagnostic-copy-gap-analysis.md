---
id: SR-120002
title: IR-721 diagnostic copy gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-driver; PR120; eight-file frozen diagnostic-copy delta
review_set: subset
---

## Summary

Independent one-pass review of driver PR120, titan/ir721-diagnostic-budget-spec; eight changed files implement FR016 AC9-12 plus truthful FR015/17/TC030 scope. Read repository AGENTS/CLAUDE and mandatory code-review Rust lane/rust-style baseline. No applicable AssuranceProfile is declared in inspected spec assurance context. The selected methods are code/Rust, manual changed gap, normative base/integrity/dependency/evidence/scope boundary. No Cargo/build/test/Clippy reruns, global scans, source edits or final gate. Inspected supplied logs: two private tests, four public monitor_locus_copy tests and five monitor_refusal tests pass; warnings-denied scoped Clippy, fmt and explicit spec validation pass. These are focused implementation receipts, not the final head gate or whole Program acceptance.

## Verdict

PASS method-specific changed scope; overall PR blocked by SR120001 findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Manual changed-scope test assessment: actual target binding path and original axes are compared through real QSL check/emission and strict IR admission in monitor_refusal. Public canonical struct tests grant copying mechanics only, not admission evidence. UTF-8 text, repeated Arc ownership, independent work/storage one-under stops, pre-cancellation, deep keys and scratch peak are asserted with values; private native capacity/overflow failures are real. Every new test uses criterion trace attributes; no mock successful native semantic body, ignored test or source-text scan is added. Current deep chain and shallow shared-leaf fixtures do not exercise a wide pending frontier: the required growth regression belongs to SR120001 FND002. Stable category assertions belong to SR120001 FND001. No duplicate findings or whole preparation credit are assigned. Full matrix/trace computation was excluded; this is manual scoped analysis.

## Examined units

```yaml
scope:
- id: FR-016-AC-9
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: The bounded copy entry returns the existing canonical locus with both original
    node axes, native path and exact optional occurrence; after source destruction,
    shared nested Arc keys and copied text/path fields remain usable. It performs
    no resolution or validity claim.
- id: FR-016-AC-10
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Independently measured work and peak logical storage permit exact-limit
    success; one-under work/storage and pre-cancellation return their distinct original
    typed operating causes with no locus. Nested/shared keys, UTF-8 text and long
    native paths are included; arithmetic overflow refuses before allocation.
- id: FR-016-AC-11
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Every allocating copy/scratch reservation is fallible and precharged; reservation
    failure preserves TryReserveError, excess text capacity returns Capacity before
    conversion, and cleanup releases partial output without allocating. No allocating
    native clone or infallible boxed-text shrink hides behind a successful receipt.
- id: FR-016-AC-12
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Source inspection confirms opaque operation/native failure clones, existing
    diagnostic derivation and error boxing are outside bounded-copy coverage; no bounded
    whole-preparation claim or replacement native operation representation is introduced.
```
