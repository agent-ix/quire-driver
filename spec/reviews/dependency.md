---
id: SR-004
title: "Dependency review of IR-616 cancellation and source lifetime"
type: SpecReview
analysis: dependency
scope: "FR-001, FR-011, QSL ADR-029 LC-3, QSL FR-276, TC-019"
review_set: all
---
# Dependency review of IR-616 cancellation and source lifetime

## Summary

FR-011 depends on QSL's existing cancellation and lifecycle contracts and
FR-001's compile and hand-off sequence. The later CLI signal/deadline adapter
depends on the library token contract and remains a separate planned layer;
no cycle or ordering conflict was found.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | No issues found | - |
