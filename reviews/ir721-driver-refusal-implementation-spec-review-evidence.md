---
id: SR-2026110414560035
title: IR-721 PR118 implementation spec-review/evidence
type: SpecReview
analysis: evidence
scope: agent-ix/quire-driver; PR118; src/monitor_refusal.rs; tests/monitor_refusal.rs;
  src/lib.rs; spec/functional/FR-016-retain-monitor-refusal-loci.md; spec/test-cases/TC-030-monitor-refusal-loci.md
review_set: subset
---

## Summary

IR-721 driver PR118 full implementation review at frozen d8a3f1225f277e7d9ddcf36eda332d5b7722a393. Reviewed new module, full test fixture, single lib export and qualification edits. Normative endpoint finding is fixed; normative EARS/scope-boundary reasoning remains applicable. No applicable AssuranceProfile found. No manifest, lock, p8 FR015/TC029 or preparer changes occur.

## Verdict

PASS within changed implementation scope; overall PR awaits the base-review correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Manual evidence assessment without the global deterministic advisor. AC1..7 use actual functional constructor tests and typed comparisons; AC8 source inspection matches actual dependency/call paths and is not relabeled Test. Evidence is narrow diagnostic derivation and retention; the source fixture ordinary functions do not supply temporal algorithm evidence. Complete original operation comparisons at root paths are strong, while impossible nested-Application credit must be corrected per base finding. Unchanged normative EARS and scope-boundary artifacts are reused as history, not overwritten. No blanket healthy preparation or FR276 credit is introduced.

## Verification limits

No Cargo/test/gate/global advisor/census/metrics/Kani invocation by reviewer. Existing focused-receipts.json records four tests passed, no failed/ignored/filtered tests, scoped Clippy exit 0, fmt exit 0, changed-doc validation exit 0 and diff check exit 0. Earlier failed fixture/debug receipts were not credited as passing. Constructor evidence does not establish an actual temporal algorithm, healthy preparation, target comparison, support policy, Program/build or FR276 acceptance. Final scoped gate remains with lead.

## Examined units

```yaml
scope:
- id: FR-016-AC-1
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: For an actual strictly admitted node, the locus retains the supplied selector
    and exactly its package-resolved structural key as distinct values, plus the original
    path and optional occurrence; its owned fields remain readable after package/input
    destruction.
- id: FR-016-AC-2
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Aggregate/application source-order children and binding child zero resolve
    their actual typed subterms. An unavailable child, nonzero binding child or continuation
    past a reference returns InvalidBodyPath; selecting the referenced original node
    starts a separate path.
- id: FR-016-AC-3
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: On a genuinely admitted monitor root, Every TargetPremise resolves the
    value of its own actual named binding, including the final Binding.value child
    zero, without comparing or changing that value. Uncertainty resolves its actual
    uncertainty value path; BufferCapacity resolves the chosen capacity value path,
    including its final child zero and original row owner.
- id: FR-016-AC-4
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: BufferCapacity without an index, an out-of-range buffer index, and an index
    supplied for another premise each return InvalidPremiseIndex, with no target payload.
    A known ordinary node used as a root returns NotMonitorRoot; an unresolved node
    returns UnknownNode.
- id: FR-016-AC-5
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: A selected actual application yields its entire original CheckedApplicationOperation
    and the caller-resolved containing-node kind. A nonapplication terminal yields
    None. Changing the selected inline operation or path changes the observation;
    no support-policy verdict is inferred.
- id: FR-016-AC-6
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: An actual admitted role/ordinal is retained exactly; a nonmember occurrence
    returns OccurrenceMismatch. A genuine generated root succeeds with caller-supplied
    None even when it retains a Generated occurrence, without selecting a source-map
    row or inventing a selected occurrence.
- id: FR-016-AC-7
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: Construction retains typed error distinctions and returns no partial diagnostic
    on failure. Neither constructor changes the admitted package, target values, operation
    records or source correspondence.
- id: FR-016-AC-8
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: The diagnostic module uses native IR identities/terms and native Vec child
    paths, introduces no plan/target/expression DTO or evaluator, and performs no
    parse, initialization, runtime build, support-policy decision, whole-graph index
    collection or identity recomputation.
- id: src/monitor_refusal.rs
  path: src/monitor_refusal.rs
  role: examined
  excerpt: 'All 246 lines: typed public payloads, errors, term_at, locus, named_value,
    target_mismatch_at, unsupported_at.'
- id: tests/monitor_refusal.rs
  path: tests/monitor_refusal.rs
  role: examined
  excerpt: 'All 525 lines: real checked QSL/public monitor emission/strict IR fixture
    and four functional tests.'
- id: src/lib.rs
  path: src/lib.rs
  role: examined
  excerpt: pub mod monitor_refusal;
- id: spec/test-cases/TC-030-monitor-refusal-loci.md
  path: spec/test-cases/TC-030-monitor-refusal-loci.md
  role: examined
  excerpt: "Select an actual application, including an inline application under a\n\
    \   binding or aggregate when present."
- id: spec/functional/FR-016-retain-monitor-refusal-loci.md
  path: spec/functional/FR-016-retain-monitor-refusal-loci.md
  role: examined
  excerpt: 'The four focused `monitor_refusal` tests execute the diagnostic-constructor

    controls of AC-1 through AC-7 through public QSL checking/emission and strict

    IR admission.'
```
