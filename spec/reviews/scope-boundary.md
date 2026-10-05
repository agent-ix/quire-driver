---
id: SR-036
title: "scope-boundary review of IR-616 cancellation specification"
type: SpecReview
analysis: scope-boundary
scope: "FR-011, FR-001-AC-9..10, TC-019 and AD-001 boundary/risks sections"
review_set: all
---

## Summary

Cancellation ownership is mostly well allocated: callers initiate it, QSL polls and owns categories, and the driver forwards the shared handle; CLI signal/deadline adaptation remains outside scope. The source-lifetime responsibility is not resolved because the retained `CheckedUnit` still exposes source and library documents.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Explicitly decide whether `Driven` retains the checked unit's source/libraries after E5; dropping `EmittedUnit` alone does not remove the source reachable through `Driven.package`. | FR-001, FR-011-AC-4, TC-019, AD-001 |

## System Boundary

- **In scope:** driver library accepts and forwards the caller's QSL `Cancel`; it propagates typed cancellation and releases the separate E4 `EmittedUnit` after E5; no driver timer or polling is added.
- **External dependency:** QSL's `Cancel`, cancellation causes, polling, typed `StageFailure`, category, and exit function. QSL owns these semantics; TC-019 proposes direct lifecycle integration evidence.
- **Out of scope:** CLI signal handling and deadline/timer triggers; these may later adapt a frontend signal/deadline to the same caller-owned token.

## Responsibility Allocation

| Requirement | Owning component | Class |
| --- | --- | --- |
| FR-001-AC-9..10 | Driver library | Core orchestration and returned-value lifetime |
| FR-011-AC-1..5 | Driver library with QSL lifecycle contract | Core boundary; QSL owns cancel cause/category/polling |
| Future CLI signal/deadline adapter | Driver CLI frontend (planned separately) | Cross-cutting frontend; outside FR-011 |

The retained `CheckedUnit` exposes `source()` and `libraries()`. The current wording's “caller retains its own input” rationale should state whether this existing returned-value access is intentionally preserved.
