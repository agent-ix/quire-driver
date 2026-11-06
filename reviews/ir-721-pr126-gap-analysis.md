---
id: SR-2026110414560054
title: IR-721 PR126 gap-analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver; spec/functional/FR-018-admit-source-record-witnesses.md; src/monitor; tests/monitor*; tests/source_record_admission.rs"
review_set: subset
---

## Summary

Independent PR #126 review at 401e5107bde84b19c7cbc5f7f865a7b47673a4cd. Ticket IR-721. No applicable AssuranceProfile found.

## Verdict

CONDITIONAL: the clock evidence and FR-015 requirement disagree.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Changed clock tests remove the only MissingClock refusal assertion, while FR-015-AC-13 still requires that refusal and its qualification prose says the current fixture produces it. Update the requirement and qualification status for native model time, or add a reachable missing-clock fixture that tests the retained obligation. | tests/monitor_preparation.rs:668 |

## Coverage

Plan completion: not assessed. Changed trace bindings were compared with their assertions; optional semantic fanout was not invoked.

## Examined units

```yaml
{
  "scope": [
    {
      "id": "FR-018-AC-5",
      "path": "spec/functional/FR-018-admit-source-record-witnesses.md",
      "role": "examined",
      "excerpt": "An authentic emission joins successfully at its measured native work and storage charge. One unit below either dimension returns no owner and preserves the original native exhausted-work or exhausted-storage payload. The outer preparation classifier maps native exhausted storage to `storage` and incomplete identity to `projection` without losing the native payload. This does not claim cumulative preparation accounting."
    },
    {
      "id": "FR-015-AC-13",
      "path": "spec/functional/FR-015-prepare-an-admitted-monitor-program.md",
      "role": "examined",
      "excerpt": "The separate clock-admission entry preserves native `WrongClause`, `WrongRole`, `UnknownOccurrence`, `MissingClock` and `WorkLimit` refusal payloads, including exact locus, occurrence, consumed work and attempted work. Semantic clock refusals map to outer `selection`; native work refusal maps to `work`. A missing clock returns no clock-admitted token even when the optional clock expectation is absent. Pre-cancellation returns the original cancellation cause, no clock-admitted token and unchanged inputs. Existing partial selection remains explicitly unclocked."
    },
    {
      "id": "FR-015-AC-15",
      "path": "spec/functional/FR-015-prepare-an-admitted-monitor-program.md",
      "role": "examined",
      "excerpt": "A real PlanRoot's original native subject and final emission admit one final-subject token only after strict native emission/package equality, owner declaration lookup, exact root/clause/Claim, full source-map, native name and existing independent selection checks. Two-clause controls select their own final declarations; absent lookup, foreign subject/emission/package, role/node/ordinal and independent request discrepancies give typed refusal. Native read limits preserve exact payloads; cancellation returns the original cause and no token. Driver work is retained across the token's actual nati"
    }
  ]
}
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | FR-015-AC-13 still requires the driver entry to return the native MissingClock refusal with no clock-admitted token when clock_binding is absent. The new test directly constructs MonitorPreparationError::ClockRead and checks only its classifier/payload; it never calls admit_monitor_clock_selection or token.admit_clock on a missing-clock plan. The revised qualification text explicitly says that admitted non-Timed driver-entry case is still needed. |
