---
id: SR-035
title: "risk-complexity review of IR-616 cancellation specification"
type: SpecReview
analysis: risk-complexity
scope: "FR-011 and its TC-019 evidence plan"
review_set: all
---

## Summary

FR-011 introduces cross-thread cancellation through an upstream QSL handle, which is technically high risk and exposed to an external API contract. TC-019 has a deterministic work-meter observer plan, but its in-flight cancellation case does not explicitly enumerate the four lifecycle stages.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Make the in-flight cancellation matrix cover parse, select, check, and package individually; one unspecified selected stage leaves stage-specific propagation and cancellation races unproven. | FR-011-AC-1..2, TC-019 steps 1–3 |

## Risk Register

| Requirement | Technical risk | Volatility | Drivers | Mitigation |
| --- | --- | --- | --- | --- |
| FR-001 existing flow | Low for this change | Medium | Changes are limited to new handle/input and output lifetime semantics. | Retain existing TC-001..TC-008 regression gates. |
| FR-011 | High | High | Cross-thread cancellation timing; upstream QSL owns handle, polling points, and typed cancellation semantics. | Run deterministic cancellation at each of parse/select/check/package, with both causes; preserve stage/cause and no-partial-result assertions. |

## Top Hazards

1. Cancellation can be requested while a QSL stage is active; the driver must not lose the handle, stage, cause, or incomplete category.
2. The QSL external lifecycle contract can change; keep the integration evidence tied to current typed entry points.

## Failure-Domain Cross-Check

Caller ownership, cross-thread cancellation, QSL polling ownership, and the post-package stop boundary are explicit. The existing public error mapping and the `CheckedUnit` source retention require the fixes noted in the base and evidence reviews.
