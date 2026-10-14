---
id: SR-6000
title: "IR-788 spec-review/base of caller-owned ObjectEnvironment contract"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #70; FR-004, TC-011, spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-004
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-011
    type: reviews
---
# IR-788: spec-review/base

## Summary

Ticket: IR-788. This review examined FR-004, TC-011, and their tests.md mapping for caller-owned `ObjectEnvironment` behavior. The initial review found one medium ambiguity: AC-7 generalized a typed refusal to absent context, although QSL FR-090 specifies `Undefined` for an absent lookup. The subsequent correction separates that outcome from FR-089's unresolved-population refusal; the current PR text passes this review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-004-AC-7 and TC-011 step 6 treated absent object context as always producing a typed refusal, but QSL FR-090 specifies an `Undefined` result for an absent lookup. | spec/functional/FR-004-select-the-execution-engine.md:78; spec/test-cases/TC-011-execution-engine-selection.md:34-36 |

## Examined scope

```yaml
scope:
  - {id: FR-004-AC-1, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "An execution request without `--engine` runs the interpreter and returns its result."}
  - {id: FR-004-AC-2, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "A value outside the closed engine set is refused before any stage with QSL's refusal category and names the supplied value."}
  - {id: FR-004-AC-3, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "A recognized engine absent from the binary is refused before any stage with category unsupported; no other engine runs, and process status equals QSL's category exit function."}
  - {id: FR-004-AC-4, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "When the selected engine returns unsupported or a typed preparation failure, that result is returned and no other engine is invoked."}
  - {id: FR-004-AC-5, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "The public execution request carries the caller-owned typed QSL `ObjectEnvironment`, and the selected interpreter passes that same value unchanged to QSL `spine::run`."}
  - {id: FR-004-AC-6, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "Given a function whose result depends on object context, a nonempty caller environment produces the QSL result for that context and differs from the result with an empty environment; the public library request derives no context or default."}
  - {id: FR-004-AC-7, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "Given a QSL-defined absent object lookup whose QSL result is `Undefined`, the driver returns that `Undefined` outcome unchanged and does not replace it with a refusal or another outcome."}
  - {id: FR-004-AC-8, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "Given a population argument whose identity has no binding in the caller-supplied environment, the driver returns QSL's existing typed refusal unchanged, as required by QSL FR-089-AC-4."}
  - {id: FR-004-AC-9, path: spec/functional/FR-004-select-the-execution-engine.md, role: examined, excerpt: "The current CLI handler explicitly supplies an empty `ObjectEnvironment` as its caller policy because its invocation surface has no object-context input; public library callers still must supply the required environment themselves."}
  - {id: TC-011, path: spec/test-cases/TC-011-execution-engine-selection.md, role: examined, excerpt: "Inspect the CLI's explicit empty-environment policy; separately exercise the public library request with nonempty context, an unresolved population identity, and an absent undefined lookup."}
  - {id: tests.md-FR-004-row, path: spec/tests.md, role: examined, excerpt: "FR-004 AC-1 through AC-9 map to TC-011 and remain planned pending the implementation of caller-owned context pass-through and its operation-specific outcomes."}
  - {id: QSL-FR-090, path: context/QSL-FR-090, role: context_only, excerpt: "PreconditionFalse and the absent lookup key SHALL reach an S6a caller in FamilyOutcome::FamilyEvaluated(FamilyResult::Undefined(_)), carrying the undefined reasons precondition-false and absent-key."}
  - {id: QSL-FR-089-AC-4, path: context/QSL-FR-089, role: context_only, excerpt: "An unresolved population identity produces a typed evaluator refusal, not a panic, an Undefined outcome, or a silently substituted default binding."}
findings:
  - fnd: FND-001
    severity: medium
    confidence: high
    method: spec-review/base
    check: soundness
    artifact_id: FR-004-AC-7
    related_ids: [TC-011]
    path: spec/functional/FR-004-select-the-execution-engine.md
    lines: "78"
    excerpt: |-
      Given a function that requires object context absent from the caller-supplied environment, the driver returns QSL's existing typed refusal unchanged and does not substitute another outcome.
    finding: "This made absence imply refusal generally, while QSL FR-090 specifies an Undefined outcome for an absent lookup."
bindings: []
```

## Verdict

The initial finding is resolved in current PR text: absent lookup and unresolved population identity are distinct operation-specific outcomes. The current CLI's explicit empty environment is clearly separated from the public library request's required caller-owned environment.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-004-AC-7 now covers the QSL FR-090 Undefined case; FR-004-AC-8 separately covers the FR-089 typed-refusal case. TC-011 and tests.md match. |
