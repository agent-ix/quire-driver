---
id: SR-117406
title: IR-721 root dependency delta review
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-driver; PR117; frozen dependency_edges FR001 TC005 delta"
review_set: subset
---

## Summary

Round 2 inspected detached IR-721-AC2-growth-fix-review plus the separately frozen root dependency-edge delta. Scoped scalar foundation is review-clean: all original findings now fixed. Native scalar output remains a component, not complete prepare_monitor/lower_monitor, RT Program/Layout or temporal ownership. Caller input acquisition and diagnostic storage remain unqualified/Planned. Inspected existing receipts: eight public mapper tests and one private growth regression pass; lib/public Clippy, owned formatting and explicit docs pass. The root dependency target passes once, with formatting/docs/diff checks passing. Reviewer ran no Cargo/build/test/Clippy or global scans. Final composed-head scoped gate remains pending.

## Verdict

PASS scoped dependency/test/normative delta. Code/Rust, manual changed-gap, base integrity, dependency, evidence and scope lenses examined together; no fresh global methods or computations.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

FR001 and TC005 explicitly authorize the native RT exact edge and genuine producer dev-only qsl-analyze. Exact normal/build and dev set equality remains; five normal QSL layer edges, no root QSL/cache/host/kit closure, CLI host/kit/default membership, one shared IR model, CG source identity and QSL emitter model identity remain checked. RT normal/build closure adds explicit no QSL/IR/model/driver/CG backedge. Graph reachability filters both normal and build kinds. Existing manifest has branch-main RT default-features=false exact and dev-only qsl-analyze; no pin/shim is introduced. Trace attributes cite only actually asserted FR001 AC1/11, FR012 AC13, FR014 AC2/3, with no full FR015 credit. Dependency direction provides no monitor execution acceptance. The changed dependency target log records one passing test; no gate failure is invented.

## Examined units

```yaml
scope:
- id: FR-001-AC-1
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: RT supplies FR-015/FR-017's native retained program/arena types.
- id: TC-005
  path: spec/test-cases/TC-005-dependency-edges.md
  role: examined
  excerpt: This dependency direction check alone is not evidence that monitor preparation or execution has succeeded.
```
