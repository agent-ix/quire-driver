---
id: SR-120001
title: IR-721 diagnostic copy code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-driver; PR120; eight-file frozen diagnostic-copy delta
review_set: subset
---

## Summary

Independent one-pass review of driver PR120, titan/ir721-diagnostic-budget-spec; eight changed files implement FR016 AC9-12 plus truthful FR015/17/TC030 scope. Read repository AGENTS/CLAUDE and mandatory code-review Rust lane/rust-style baseline. No applicable AssuranceProfile is declared in inspected spec assurance context. The selected methods are code/Rust, manual changed gap, normative base/integrity/dependency/evidence/scope boundary. No Cargo/build/test/Clippy reruns, global scans, source edits or final gate. Inspected supplied logs: two private tests, four public monitor_locus_copy tests and five monitor_refusal tests pass; warnings-denied scoped Clippy, fmt and explicit spec validation pass. These are focused implementation receipts, not the final head gate or whole Program acceptance.

## Verdict

FAIL scoped source; two findings require same-PR fixes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | MonitorLocusCopyError exposes a new public thiserror envelope but no stable Copy code enum/catalog or code accessor. The mandatory Rust baseline requires as_str/all/from_code stable categories; existing MonitorPreparationError follows that rule. Add nonallocating stable codes for Work, Storage, Allocation, Capacity and Cancelled, expose the error code without formatting, document the API and assert actual error classification plus round trips/unknown spellings. Preserve native causes and allocation-free failures. | src/monitor_refusal/copy.rs:23-60 |
| FND-002 | medium | shared() grows its pending frontier using try_reserve_exact(1) for each extra child, without charging relocation of the existing pending references before allocation. A shared child with many Node edges makes a wide frontier: on exact-capacity allocations there are linear growth calls and quadratic copied references under the linear reported work schedule. Use geometric fallible scratch growth with a pre-action relocation charge (or another bounded iterative approach), update the documented accounting schedule, and add a broad-frontier/denied-growth control proving Work is returned before allocation or storage-ledger mutation. Existing deep-chain/two-leaf controls never exercise this frontier-growth branch. | src/monitor_refusal/copy.rs:162-178 |

## Coverage

Canonical node/key axes and entire body path are retained unchanged, including final Binding child zero from the actual producer-backed target-locus control. Root text/path/Name fields are fallibly owned; Node arcs retain exact original backing, measured iteratively with conservative per-edge descendant accounting. Shared backing includes native node size, boxed text and path capacity, excluding declared allocator/control-block overhead. Both root copies and scratch share a peak ledger; success releases scratch. Cancellation precedes charges/reservations and retains CancelCause. Exact text capacity is guarded before push and allocation-free boxing. Failure partial roots hold borrowed-source Arc counterparts, so cleanup cannot uniquely unwrap their descendant trees; owned vectors/text drop without allocation and no cancellation poll. Arithmetic checked refusals and native TryReserveError remain distinct. No source parse, validity proof, operation DTO, hidden target/unsupported prepass or fabricated semantic owner appears. The two code findings below prevent scoped approval.

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
