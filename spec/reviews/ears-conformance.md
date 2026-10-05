---
id: SR-037
title: "EARS conformance review of IR-616 requirement statements"
type: SpecReview
analysis: ears-conformance
scope: "Changed requirement statements in FR-011 and FR-001; Quire validation over spec/**/*.md"
review_set: all
---

## Summary

Quire's deterministic EARS check reported zero grammar findings across 34 specification documents. Manual review found one noncanonical temporal trigger in a normative FR-011 statement: an `After` clause introduces a SHALL response where the canonical event trigger is `When`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Replace the normative `After E5 has consumed ...` trigger with a canonical `When` event trigger so the release obligation has explicit EARS form. | FR-011 Description |

## Manual Semantic Review

- **Keyword vs intent:** E5 consumption is a discrete event, so `When E5 consumes the E4 source provision, the driver SHALL release ...` expresses the intended transition more canonically.
- **Concrete response:** The response (release the `EmittedUnit` and source provision) is observable by object lifetime, though the retained `CheckedUnit` makes the current claimed memory effect unclear (see SR-034 FND-001).
- **Pattern selection:** The statement is event-triggered; `When` is appropriate. The source-retention decision remains a separate contract issue.
