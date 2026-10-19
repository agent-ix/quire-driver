---
id: SR-6006
title: "STD-200 spec-review/ears of the FR-010 bound-items statement"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-driver PR #84 @488bd1d; spec/functional/FR-010-send-only-bound-items-to-codegen.md Behavior; ticket STD-200"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-010
    type: reviews
---
# STD-200: spec-review/ears

## Summary

Ticket: STD-200. This review checks the EARS grammar of the one requirement statement the diff adds: the third FR-010 Behavior bullet. The FR Description and the other Behavior bullets are unchanged and were read only for context.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new bullet is a compound, ubiquitous statement. It packs three obligations (build from bound results, refused items contribute nothing, return the refusal unchanged) into one bullet, with no trigger and with a parenthetical that restates QSL FR-307's contract. | spec/functional/FR-010-send-only-bound-items-to-codegen.md:27-31 |

## Examined scope

```yaml
scope:
  - {id: FR-010, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: examined, excerpt: "The driver SHALL build the CG request from the bound results of the QSL export's per-item partition (QSL FR-307: each requested item is either a bound result, with an empty binding set when the item references no model element, or a refusal). A refused item SHALL contribute nothing to the request, and the driver SHALL return its refusal to the caller unchanged."}
findings:
  - fnd: FND-001
    severity: low
    confidence: medium
    method: spec-review/ears
    check: compound
    artifact_id: FR-010
    related_ids: []
    path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
    lines: "27-31"
    excerpt: |-
      The driver SHALL build the CG request from the bound results of the QSL
        export's per-item partition (QSL FR-307: each requested item is either a
        bound result, with an empty binding set when the item references no model
        element, or a refusal). A refused item SHALL contribute nothing to the
        request, and the driver SHALL return its refusal to the caller unchanged.
    finding: "The new bullet is a compound, ubiquitous statement. It packs three obligations (build from bound results, refused items contribute nothing, return the refusal unchanged) into one bullet, with no trigger and with a parenthetical that restates QSL FR-307's contract."
bindings: []
```

## Verdict

The statement is EARS-conformant apart from FND-001. Suggested split: "The driver SHALL build the CG request only from the QSL FR-307 export's bound results, including bound results with an empty binding set." and "When the export refuses a requested item, the driver SHALL omit that item from the CG request and SHALL return its refusal unchanged in <named output>." The named output depends on base FND-003.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The all-refused bullet is worded as a statement of fact ("the CG request carries zero items") with no SHALL, so EARS does not read it as a requirement. Use "When every requested item is refused, the driver SHALL send CG a request with zero items." | spec/functional/FR-010-send-only-bound-items-to-codegen.md:35-36 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Split into a ubiquitous bullet ("SHALL build the CG request only from the export's bound results ...") and an event-driven one ("When the export refuses a requested item, the driver SHALL omit ... and SHALL return ..."). The parenthetical restating FR-307 is gone. |
