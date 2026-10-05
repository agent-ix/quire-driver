---
id: SR-034
title: "evidence review of IR-616 cancellation specification"
type: SpecReview
analysis: evidence
scope: "FR-011-AC-1..5, FR-001-AC-9..10, TC-019 and TM-001"
review_set: all
---

## Summary

All new acceptance criteria name test or inspection verification, and TC-019 provides a concrete planned test procedure. Its source-lifetime oracle can pass while source bytes remain reachable through `Driven.package: CheckedUnit`, so it does not establish the claimed release.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-019 checks that `Driven` has no `EmittedUnit`/source-provision field but does not account for `CheckedUnit::source()` and `CheckedUnit::libraries()` retaining the same source objects; test the actual reachable lifetime or revise the claim. | FR-001-AC-10, FR-011-AC-4, TC-019 |

## Evidence Map

| Requirement | Method | Artifact | Assessment |
| --- | --- | --- | --- |
| FR-001-AC-9 | Test | TC-019 steps 1–3 | Explicit same-handle instrumentation and cancellation causes. |
| FR-001-AC-10 | Test | TC-019 step 4 | Current oracle checks absent wrapper fields, not source reachability through the retained `CheckedUnit`. |
| FR-011-AC-1 | Test | TC-019 step 1 | Verifies the shared handle at the four QSL entry points. |
| FR-011-AC-2 | Test | TC-019 steps 2–3 | Checks typed cause and no partial result. |
| FR-011-AC-3 | Test + inspection | TC-019 step 5 | Checks QSL category and inspects the single exit conversion; public result type should be explicit. |
| FR-011-AC-4 | Test | TC-019 step 4 | Source-lifetime oracle gap described in FND-001. |
| FR-011-AC-5 | Inspection | TC-019 step 5 | Checks signal/deadline adapter exclusion. |
