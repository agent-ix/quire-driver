---
id: SR-6009
title: "STD-224 spec-review/ears of the per-item refusal statements"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-driver PR #85 @970b890; spec/functional/FR-001-orchestrate-the-driver-steps.md Outputs/Behavior, spec/functional/FR-009-settle-execute-and-replay-results.md Description, spec/functional/FR-010-send-only-bound-items-to-codegen.md Behavior; ticket STD-224"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-009
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-010
    type: reviews
---
# STD-224: spec-review/spec-ears-analysis

## Summary

Ticket: STD-224. This review checks the EARS grammar of the requirement statements the diff adds or edits: FR-001's new Outputs and Behavior bullets, FR-009's new Description sentence, and FR-010's edited absent-binding bullet. The ACs were read for context only. The Outputs bullet is an output description, not a requirement statement, and conforms.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The FR-001 Behavior bullet is compound and half non-normative. It packs SHALL record and SHALL NOT return into one ubiquitous statement with no trigger. Its second sentence ('A run in which every item is refused completes with no settled item ...') states the all-refused obligation with no SHALL. Split it into an event-driven 'When the export refuses an item, the driver SHALL ...' and a separate 'When the export refuses every requested item, the driver SHALL complete the run with ...'. | spec/functional/FR-001-orchestrate-the-driver-steps.md:192-196 |
| FND-002 | low | The new FR-009 Description sentence ('An item the export refused is not in the CG request, receives no terminal record, and keeps its refusal ...') is a normative rule with no SHALL. Write 'The driver SHALL NOT produce a terminal record for an item the export refused'. | spec/functional/FR-009-settle-execute-and-replay-results.md:24-26 |
| FND-003 | low | The FR-010 absent-binding bullet now carries three obligations in one sentence: preserve the refusal, return it in FR-001's member, and not invent a CG item. The member placement duplicates FR-001's own Behavior rule. Keep one owner (FR-001) and reference it. | spec/functional/FR-010-send-only-bound-items-to-codegen.md:37-40 |

## Examined scope

```yaml
clean: false
scope:
- id: FR-001-behavior
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'A refused item''s typed refusal is an item-level result, not a command-level refusal: the driver SHALL record it in `Driven`''s per-item refusal member and SHALL NOT return a `DriverRefusal` for it. A run in which every item is refused completes with no settled item and one per-item refusal for each requested item.'
- id: FR-001-outputs
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: '`Driven` also carries a per-item refusal member: for each requested item the export refused, in request order, the item''s occurrence key and the export''s typed refusal unchanged. The member is empty when every item is bound. A partly bound package therefore returns its settled items and each refused item''s refusal in one completed result.'
- id: FR-009-desc
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: An item the export refused is not in the CG request, receives no terminal record, and keeps its refusal in FR-001's per-item refusal member.
- id: FR-010-absent
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: Where a required binding is absent, the driver SHALL preserve the typed unbound-element refusal defined by the owning QSL contract and SHALL return it in FR-001's per-item refusal member; it SHALL not invent a CG item or silently treat the item as bound.
- id: FR-001-AC-17
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: Given a package whose export returns a bound result for A and C and a refusal for B, the run completes, `Driven` holds the settlement of A and C and B's refusal in the per-item refusal member, and no `DriverRefusal` is returned; with every item refused, `Driven` holds no settled item and one refusal per requested item.
- id: FR-009-AC-16
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: Given a partly bound package, the outcome holds one terminal record for each item in the CG request and none for an export-refused item, whose refusal remains in `Driven`'s per-item refusal member.
findings:
- fnd: FND-001
  severity: low
  confidence: high
  method: spec-review/spec-ears-analysis
  check: compound
  artifact_id: FR-001-behavior
  related_ids:
  - FR-001-AC-17
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  lines: 192-196
  excerpt: 'A refused item''s typed refusal is an item-level result, not a command-level refusal: the driver SHALL record it in `Driven`''s per-item refusal member and SHALL NOT return a `DriverRefusal` for it. A run in which every item is refused completes with no settled item and one per-item refusal for each requested item.'
  finding: The FR-001 Behavior bullet is compound and half non-normative. It packs SHALL record and SHALL NOT return into one ubiquitous statement with no trigger. Its second sentence ('A run in which every item is refused completes with no settled item ...') states the all-refused obligation with no SHALL. Split it into an event-driven 'When the export refuses an item, the driver SHALL ...' and a separate 'When the export refuses every requested item, the driver SHALL complete the run with ...'.
- fnd: FND-002
  severity: low
  confidence: high
  method: spec-review/spec-ears-analysis
  check: other
  artifact_id: FR-009-desc
  related_ids:
  - FR-009-outputs
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  lines: 24-26
  excerpt: An item the export refused is not in the CG request, receives no terminal record, and keeps its refusal in FR-001's per-item refusal member.
  finding: The new FR-009 Description sentence ('An item the export refused is not in the CG request, receives no terminal record, and keeps its refusal ...') is a normative rule with no SHALL. Write 'The driver SHALL NOT produce a terminal record for an item the export refused'.
- fnd: FND-003
  severity: low
  confidence: medium
  method: spec-review/spec-ears-analysis
  check: compound
  artifact_id: FR-010-absent
  related_ids:
  - FR-010-AC-3
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  lines: 37-40
  excerpt: Where a required binding is absent, the driver SHALL preserve the typed unbound-element refusal defined by the owning QSL contract and SHALL return it in FR-001's per-item refusal member; it SHALL not invent a CG item or silently treat the item as bound.
  finding: 'The FR-010 absent-binding bullet now carries three obligations in one sentence: preserve the refusal, return it in FR-001''s member, and not invent a CG item. The member placement duplicates FR-001''s own Behavior rule. Keep one owner (FR-001) and reference it.'
bindings: []
```

## Recommendations

- Split FR-001's bullet into event-driven SHALL statements; add SHALL to FR-009's sentence; leave FR-010 to reference FR-001.

## Verdict

Mergeable on EARS grounds: three low findings, all wording. FR-001's Outputs bullet conforms.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix 86fe419: When the export refuses a requested item, the driver SHALL record the refusal in the member. / When the export refuses a requested item, the driver SHALL NOT return a `DriverRefusal` for it. / When the export refuses every requested item, the driver SHALL complete the run with no settled item and one per-item refusal for each requested item. |
| FND-002 | fixed | Fix 86fe419: For an item the export refused, the driver SHALL write no terminal record and SHALL keep its refusal in FR-001's per-item refusal member. |
| FND-003 | fixed | Fix 86fe419: - Where a required binding is absent, the driver SHALL preserve the typed unbound-element refusal defined by the owning QSL contract. - Where a required binding is absent, the driver SHALL return that refusal in FR-001's per-item refusal member. - The driver SHALL NOT invent a CG item or treat an unbound item as bound. |
