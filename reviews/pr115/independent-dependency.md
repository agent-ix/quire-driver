---
id: SR-20261104030002
title: "Independent PR 115 dependency review"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-driver; PR 115; FR-015, TC-029, AD-001, spec/spec.md, TM-001; IR-721"
review_set: subset
---

## Summary

Independent scoped review of PR 115 at b94faf8. FAIL: actionable findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-015 says equality/source-unit witnesses are currently discarded, but QSL 13c7edf retains those final-emission rows. Name QSL-1631’s remaining source-owned witness gap; keep complete closure planned. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:223-232; spec/test-cases/TC-029-admitted-monitor-program-preparation.md:87-91 |
| FND-002 | medium | FR-015 requires the complete generated-source type closure but omits IR FR-357 from its prerequisite edges and dependency text. Add that dependency and state its first witnessed subset cannot populate a complete TypeEnvironment or RT ProgramParts. | spec/functional/FR-015-prepare-an-admitted-monitor-program.md:12-23,116-123,229-232 |

## Examined Scope

- examined: FR-015: Complete closure and producer witness dependencies.
- examined: TC-029: Expected Results call upstream witness retention planned.
- context_only: FR-357: First witnessed subset refuses current PlanRoot at reached unsupported source-owned form; complete closure awaits QSL-1631.

## Analysis

QSL 13c7edf `PlanRoot.emission` calls `emit_monitor_plan_over`, which rekeys retained equality producer/fact and selected source-unit rows. IR PR 881 FR-357 says QSL-1463 has landed those rows while QSL-1631 remains pending for source-owned enum, collection and union witnesses; its first IR subset must refuse the reached current PlanRoot. FR-015 can specify full eventual closure, but must describe these separate prerequisites accurately.

## Verification and Limits

Quire 0.36.2 structural validation and grammar summary pass (84/84). The computed matrix marks FR-015-AC-1 through AC-12 untagged, consistent with planned work. No Cargo tests or runtime gates were run. No applicable AssuranceProfile was found in the driver spec tree.
