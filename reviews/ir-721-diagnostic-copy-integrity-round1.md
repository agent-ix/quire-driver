---
id: SR-120004
title: IR-721 diagnostic copy integrity review
type: SpecReview
analysis: integrity
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

The copy-only success is expressly distinct from resolution, membership, target-path derivation and native opaque operation/error retention. Canonical fields are not a replacement semantic DTO. Caller supplies independent axes; no equality or name inference establishes authority. Shared storage is deliberately conservative per edge, with independent root accounting; exact allocator overhead is excluded explicitly. Original native cancellation/allocation causes remain failures, and Capacity is a portability operating refusal rather than semantic input rejection. Failure produces no partial locus. No fallback grants full Program success. The implementation work-bound issue is recorded once in SR120001 FND002, requiring accounting prose and tests to change coherently.

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

## Round 1 disposition checkpoint

Round 1 rechecks frozen PR120 diagnostic-copy fixes only. Both SR120001 findings are fixed; no new findings across affected source/tests/spec/status. This current scoped PASS supersedes the original blocking verdict without removing its findings/history. MonitorLocusCopyErrorCode has exact stable lowercase as_str/all/from_code and error.code; the tests exercise actual Work/Storage/Allocation/Capacity/Cancelled failures, literal complete spellings, round trips and unknown spellings. Scratch frontier doubles capacity with fallible exact reservation and admits relocation work before any allocation or storage mutation. A private denied-growth regression verifies pointer/capacity/contents/storage/work unchanged, and a genuine wide shared frontier has an independent represented-material plus geometric-relocation work and peak-storage oracle, including exact/one-under controls. FR016/TC030 document the updated schedule and codes. Native ownership, conservative per-edge Arc accounting, exact text capacity guard, native causes and final Binding child zero are retained. Supplied logs show four private, five public copy and five diagnostic tests passing, plus scoped Clippy/fmt/docs. No reviewer Cargo/build/test/gate reruns. Complete diagnostic integration, opaque native operation/error allocation, caller input acquisition and full Program preparation remain Planned; this is component readiness only. Final scoped gate remains the lead next step.
