---
id: SR-6008
title: "STD-224 spec-review/base of the per-item refusal member"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #85 @970b890; spec/functional/FR-001-orchestrate-the-driver-steps.md, spec/functional/FR-009-settle-execute-and-replay-results.md, spec/functional/FR-010-send-only-bound-items-to-codegen.md, spec/test-cases/TC-017-only-bound-items-reach-codegen.md, spec/tests.md, spec/assurance/AD-001-driver-architecture.md; ticket STD-224"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-009
    type: reviews
  - target: ix://agent-ix/quire-driver/FR-010
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-017
    type: reviews
  - target: ix://agent-ix/quire-driver/AD-001
    type: reviews
---
# STD-224: spec-review/base

## Summary

Ticket: STD-224. The review covered `git diff refs/remotes/origin/main...970b890` (one commit). It checked that diff against this repository's merged FR-001, FR-009, FR-010, TC-003, TC-016 and AD-001 (main @7865f37, including #84's zero-items rule and FR-009's 'each item in the CG request'), and against QSL FR-307 (quire-spec-language main @c3ed9a2).

The design is sound and closes STD-200's deferred FND-003. A partly bound run completes with A and C settled and B's refusal in a named per-item member; FR-009 and AD-001 now consistently say 'each item in the CG request'. The occurrence key plus the export's typed refusal matches QSL FR-307, whose inputs are occurrence keys and whose refusal is `missing_declaration`/`missing-name` naming each unbound element with its `DomainPackageRef`. The all-refused case does not conflict with FR-001's empty-request path: zero CG items negotiate zero settlements and skip `route` and generation, as TC-003 step 5 already shows, and 'one refusal per requested item' agrees with FR-307's 'one refusal per item'. A unit with no requirement records gives an empty member. The text describes behaviour, not a changelog.

Four medium gaps remain: the unqualified 'refused item' rule, the index/key join, the whole-run boundary around the export, and FR-009-AC-16's missing proof precondition.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The new FR-001 Behavior rule says 'A refused item's typed refusal' goes to the per-item member and is never a `DriverRefusal`, without saying 'export-refused'. FR-001 has other per-item refusals: a negotiation settlement such as `invalid-request`/`unknown-backend` (TC-003 step 2), CG's `NoDerivableClaim` settlement right above it, and QSL's typed `RunRefusal`/`BindRefusal` for a scalar item, which FR-001-AC-16 says stops the composition. Read literally, the rule turns each of these into a member entry and forbids a `DriverRefusal` for them. Qualify it as 'an item the export (QSL FR-307) refused' to match the Outputs bullet. | spec/functional/FR-001-orchestrate-the-driver-steps.md:192-196 |
| FND-002 | medium | Ordering and keys are not stated well enough to join the two halves of `Driven`. Settlements and generation output are keyed by request index (TC-003 step 4), but the refusal member is keyed by occurrence key. 'Request order' now means the driver's request in one bullet and 'the CG request' in the other. Nothing says whether C keeps driver index 2 or becomes CG-request position 1 once B is omitted. FR-001-AC-15 also still says the FR-330 position vector holds one `DriverItem` per original request item, 'whether or not routing later selects it', so it is open whether B keeps a position. State which index the settlements carry after an omission, whether B occupies an FR-330 position, and give the member a request index (or say occurrence key is the only join key). | spec/functional/FR-001-orchestrate-the-driver-steps.md:78-89 |
| FND-003 | medium | The boundary is stated only from the per-item side. Nothing says what stays a whole-run `DriverRefusal` around the export: the export call has no step in FR-001's sequence or in the command-level refusal list (no step, code or outcome kind for an export that fails as a whole). A QSL FR-307 'occurrence key names no requirement record' refusal for a key the driver itself built from `requirements()` is the same E4/E5 inconsistency that FR-001 treats as a `runtime_invariant` fault for a missing node, yet the new rule files it as an ordinary per-item refusal. A package with no abstraction relation (FR-010 applies only 'when an abstraction relation binds') is not covered by 'the member is empty when every item is bound'. Add the export step, its whole-run failure classification, the unknown-key case, and the no-relation case. | spec/functional/FR-001-orchestrate-the-driver-steps.md:192-196 |
| FND-004 | medium | FR-009-AC-16 omits FR-009's precondition. Terminal records exist only 'when the caller requests a proof' and the request 'reaches per-item settlement', but the AC says only 'Given a partly bound package'. TC-017's procedure requests no proof: it records CG negotiation and generation inputs. Kani execution is also blocked (IR-301, FR-001 Status). So TC-017 cannot observe 'the outcome holds terminal records for A and C' as written. Add 'for a proof request that reaches per-item settlement' to the AC and a proof run to TC-017's procedure, or move the terminal-record half to TC-016. | spec/functional/FR-009-settle-execute-and-replay-results.md:188 |
| FND-005 | low | TC-017's second Expected Results bullet still says 'the driver returns QSL's typed unbound-element refusal'. FR-010-AC-3 now places that refusal in the per-item member of a completed result, and the old wording reads like a `DriverRefusal` return. Reword it to 'the completed result's per-item refusal member holds QSL's typed unbound-element refusal unchanged'. | spec/test-cases/TC-017-only-bound-items-reach-codegen.md:42-43 |
| FND-006 | low | The AD-001 accounting sentence places a settled item 'in FR-009's final outcome'. FR-009's outcome exists only for a proof request, and the same paragraph says a `generate` request implies no proof. For a generate run a settled item is accounted for by CG's settlement in `Driven` (FR-001 Outputs). Say 'a settled item in `Driven`'s CG settlement (and, for a proof, its FR-009 terminal record)'. | spec/assurance/AD-001-driver-architecture.md:155-160 |
| FND-007 | low | FR-009 Outputs says B's refusal stays in `Driven` 'alongside the records of the items that settled'. The terminal records are in the `quire-outcome/1` outcome document, not in `Driven`, so the two are not side by side in one output. No requirement says whether the CLI's outcome document carries the refused item at all, so a CLI `prove` of a partly bound package would currently report A and C and silently drop B. Either name the outcome document's handling of refused items or drop 'alongside'. | spec/functional/FR-009-settle-execute-and-replay-results.md:55-57 |

## Examined scope

```yaml
clean: false
scope:
- id: FR-001-outputs
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: '`Driven` also carries a per-item refusal member: for each requested item the export refused, in request order, the item''s occurrence key and the export''s typed refusal unchanged. The member is empty when every item is bound. A partly bound package therefore returns its settled items and each refused item''s refusal in one completed result.'
- id: FR-001-settle
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: The in-process `CheckedUnit`, the emitted v2 package, IR's admitted `CheckedPackageV2`, CG's settlement of every item in the CG request, in request order, each routed item with its `BackendId` and CG backend kind, and CG's generation output in ascending request index.
- id: FR-001-behavior
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'A refused item''s typed refusal is an item-level result, not a command-level refusal: the driver SHALL record it in `Driven`''s per-item refusal member and SHALL NOT return a `DriverRefusal` for it. A run in which every item is refused completes with no settled item and one per-item refusal for each requested item.'
- id: FR-001-AC-17
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Given a package whose export returns a bound result for A and C and a refusal for B, the run completes, `Driven` holds the settlement of A and C and B's refusal in the per-item refusal member, and no `DriverRefusal` is returned; with every item refused, `Driven` holds no settled item and one refusal per requested item.
- id: FR-009-desc
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: An item the export refused is not in the CG request, receives no terminal record, and keeps its refusal in FR-001's per-item refusal member.
- id: FR-009-outputs
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: For each export-refused item, no terminal record; its refusal stays in `Driven`'s per-item refusal member (FR-001), alongside the records of the items that settled.
- id: FR-009-AC-16
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: examined
  excerpt: Given a partly bound package, the outcome holds one terminal record for each item in the CG request and none for an export-refused item, whose refusal remains in `Driven`'s per-item refusal member.
- id: FR-010-absent
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: Where a required binding is absent, the driver SHALL preserve the typed unbound-element refusal defined by the owning QSL contract and SHALL return it in FR-001's per-item refusal member; it SHALL not invent a CG item or silently treat the item as bound.
- id: FR-010-AC-3
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: When a binding required for emission is absent, the item's QSL-defined typed unbound-element refusal appears unchanged in the per-item refusal member of the completed result, and no unbound item is sent to CG.
- id: FR-010-AC-4
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: examined
  excerpt: Given an export that returns a bound result for a clause item A, a refusal for an extent-domain item B over an unbound population, and a bound result with an empty binding set for an item C that references no model element, the CG request carries A and C and not B, and B's refusal is in the per-item refusal member beside the settlement of A and C.
- id: TC-017-vector
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: examined
  excerpt: The completed result holds the settlement of A and C and B's refusal, unchanged, in the per-item refusal member; no `DriverRefusal` is returned (FR-001-AC-17). The outcome holds terminal records for A and C and none for B (FR-009-AC-16).
- id: TC-017-allrefused
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: examined
  excerpt: For the all-refused repeat, the CG request carries zero items (FR-010-AC-5), and the completed result holds no settled item and one refusal per requested item (FR-001-AC-17).
- id: TC-017-expected
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: examined
  excerpt: No unbound item reaches CG. Where a required binding is absent, the driver returns QSL's typed unbound-element refusal.
- id: AD-001-accounting
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: 'FR-010 qualifies the CG input: only relation-bound items enter negotiation and generation. Selection never discards the need to account for each requested item: a settled item in FR-009''s final outcome, a refused item in FR-001''s per-item refusal member. Proving extends generation with provider execution, QSL replay and CG settlement; a `generate` request does not imply that a proof was executed.'
- id: AD-001-row
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: Call the authoritative mapping with provider result and replay result; write exactly one terminal record per item in the CG request, including unsupported and incomplete items
- id: AD-001-summary
  path: spec/assurance/AD-001-driver-architecture.md
  role: examined
  excerpt: writing one FR-331 terminal record per item in the CG request.
- id: FR-001-AC-15
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: CG receives only its existing admitted `ContractPackageV1`, complete FR-330 position vector, routed items, and generation contexts. The position vector retains one `DriverItem` per original request item, in driver-index order, and a separate `ContextRoot` for each selected direct conversion occurrence; context roots create no route, claim, or generated item and do not advance driver indices.
- id: FR-001-AC-16
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: No selected scalar item means no scalar binding or native execution and no fallback selection. QSL's typed missing/ambiguous call or occurrence refusal is preserved unchanged. ... Each case stops before native execution or replay and invents no call, occurrence, context root, or operand order.
- id: FR-001-cmdrefusal
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: 'Each command-level refusal SHALL carry its step, its catalog code and an outcome kind: refusal, limit refusal or internal fault. A provider manifest that does not register, a spine refusal, the unknown-backend refusal and an envelope refusal are refusals. ... An upstream answer that breaks the driver''s invariants is an internal fault'
- id: FR-001-inputs
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: context_only
  excerpt: one item per record in occurrence-key order (`qsl_route::request::items_from_requirements`, FR-075).
- id: FR-009-scope
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  role: context_only
  excerpt: When the caller requests a proof, the driver SHALL run the generated provider request and replay each counterexample through QSL's replay operation. For each proof request that reaches per-item settlement, the driver SHALL produce exactly one FR-331 terminal record for each item in the CG request.
- id: FR-010-AC-5
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: When the export refuses every requested item, the driver sends CG a request with zero items.
- id: FR-010-desc
  path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
  role: context_only
  excerpt: When an abstraction relation binds a requested item, the driver SHALL send only the bound items to CG's negotiation and generation operations.
- id: TC-003-step5
  path: spec/test-cases/TC-003-route-after-negotiation.md
  role: context_only
  excerpt: 'Step 5: `route` is never called, generation is never called, and the run still completes (an empty request negotiates zero items).'
- id: TC-003-step4
  path: spec/test-cases/TC-003-route-after-negotiation.md
  role: context_only
  excerpt: 'Both the routed and the generated item carry request index 1, not 0: keying by list position instead of `settlement.request_index` would put 0 there instead, and fail this test.'
- id: TC-017-procedure
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  role: context_only
  excerpt: In the quire-integration suite, compose the QSL abstraction-relation intake and the driver with a package whose relation binds a strict subset of its requested items. Record the items supplied to CG negotiation and generation.
- id: QSL-FR-307-items
  path: quire-spec-language:spec/functional/FR-307-export-the-bindings-each-item-references.md
  role: context_only
  excerpt: A requested occurrence key that names no requirement record SHALL refuse `missing_declaration`/`missing-name`, naming the key. An item that references an element with no binding SHALL refuse `missing_declaration`/`missing-name`. The refusal SHALL name every unbound element of the item, in ascending key order ..., each with its owning `DomainPackageRef` identity (QSpec FR-353-AC-3). ... When every requested item is refused, the export SHALL still return one refusal per item.
- id: QSL-FR-307-inputs
  path: quire-spec-language:spec/functional/FR-307-export-the-bindings-each-item-references.md
  role: context_only
  excerpt: 'The requested items, each a requirement record''s occurrence key (ADR-012 §13.5). Outputs: One result per requested item: the item with the bindings it references, or a `missing_declaration`/`missing-name` refusal.'
findings:
- fnd: FND-001
  severity: medium
  confidence: high
  method: spec-review/base
  check: ambiguous
  artifact_id: FR-001-behavior
  related_ids:
  - FR-001-AC-16
  - FR-001-outputs
  - TC-003-step4
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  lines: 192-196
  excerpt: 'A refused item''s typed refusal is an item-level result, not a command-level refusal: the driver SHALL record it in `Driven`''s per-item refusal member and SHALL NOT return a `DriverRefusal` for it. A run in which every item is refused completes with no settled item and one per-item refusal for each requested item.'
  finding: 'The new FR-001 Behavior rule says ''A refused item''s typed refusal'' goes to the per-item member and is never a `DriverRefusal`, without saying ''export-refused''. FR-001 has other per-item refusals: a negotiation settlement such as `invalid-request`/`unknown-backend` (TC-003 step 2), CG''s `NoDerivableClaim` settlement right above it, and QSL''s typed `RunRefusal`/`BindRefusal` for a scalar item, which FR-001-AC-16 says stops the composition. Read literally, the rule turns each of these into a member entry and forbids a `DriverRefusal` for them. Qualify it as ''an item the export (QSL FR-307) refused'' to match the Outputs bullet.'
- fnd: FND-002
  severity: medium
  confidence: medium
  method: spec-review/base
  check: ambiguous
  artifact_id: FR-001-outputs
  related_ids:
  - FR-001-settle
  - FR-001-AC-15
  - FR-010-AC-4
  - TC-003-step4
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  lines: 78-89
  excerpt: '`Driven` also carries a per-item refusal member: for each requested item the export refused, in request order, the item''s occurrence key and the export''s typed refusal unchanged. The member is empty when every item is bound. A partly bound package therefore returns its settled items and each refused item''s refusal in one completed result.'
  finding: Ordering and keys are not stated well enough to join the two halves of `Driven`. Settlements and generation output are keyed by request index (TC-003 step 4), but the refusal member is keyed by occurrence key. 'Request order' now means the driver's request in one bullet and 'the CG request' in the other. Nothing says whether C keeps driver index 2 or becomes CG-request position 1 once B is omitted. FR-001-AC-15 also still says the FR-330 position vector holds one `DriverItem` per original request item, 'whether or not routing later selects it', so it is open whether B keeps a position. State which index the settlements carry after an omission, whether B occupies an FR-330 position, and give the member a request index (or say occurrence key is the only join key).
- fnd: FND-003
  severity: medium
  confidence: medium
  method: spec-review/base
  check: soundness
  artifact_id: FR-001-behavior
  related_ids:
  - FR-001-cmdrefusal
  - FR-001-AC-5
  - FR-010-desc
  - QSL-FR-307-items
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  lines: 192-196
  excerpt: 'A refused item''s typed refusal is an item-level result, not a command-level refusal: the driver SHALL record it in `Driven`''s per-item refusal member and SHALL NOT return a `DriverRefusal` for it. A run in which every item is refused completes with no settled item and one per-item refusal for each requested item.'
  finding: 'The boundary is stated only from the per-item side. Nothing says what stays a whole-run `DriverRefusal` around the export: the export call has no step in FR-001''s sequence or in the command-level refusal list (no step, code or outcome kind for an export that fails as a whole). A QSL FR-307 ''occurrence key names no requirement record'' refusal for a key the driver itself built from `requirements()` is the same E4/E5 inconsistency that FR-001 treats as a `runtime_invariant` fault for a missing node, yet the new rule files it as an ordinary per-item refusal. A package with no abstraction relation (FR-010 applies only ''when an abstraction relation binds'') is not covered by ''the member is empty when every item is bound''. Add the export step, its whole-run failure classification, the unknown-key case, and the no-relation case.'
- fnd: FND-004
  severity: medium
  confidence: high
  method: spec-review/base
  check: untestable-ac
  artifact_id: FR-009-AC-16
  related_ids:
  - FR-009-scope
  - TC-017-vector
  - TC-017-procedure
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  lines: '188'
  excerpt: Given a partly bound package, the outcome holds one terminal record for each item in the CG request and none for an export-refused item, whose refusal remains in `Driven`'s per-item refusal member.
  finding: 'FR-009-AC-16 omits FR-009''s precondition. Terminal records exist only ''when the caller requests a proof'' and the request ''reaches per-item settlement'', but the AC says only ''Given a partly bound package''. TC-017''s procedure requests no proof: it records CG negotiation and generation inputs. Kani execution is also blocked (IR-301, FR-001 Status). So TC-017 cannot observe ''the outcome holds terminal records for A and C'' as written. Add ''for a proof request that reaches per-item settlement'' to the AC and a proof run to TC-017''s procedure, or move the terminal-record half to TC-016.'
- fnd: FND-005
  severity: low
  confidence: high
  method: spec-review/base
  check: other
  artifact_id: TC-017-expected
  related_ids:
  - FR-010-AC-3
  path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
  lines: 42-43
  excerpt: No unbound item reaches CG. Where a required binding is absent, the driver returns QSL's typed unbound-element refusal.
  finding: TC-017's second Expected Results bullet still says 'the driver returns QSL's typed unbound-element refusal'. FR-010-AC-3 now places that refusal in the per-item member of a completed result, and the old wording reads like a `DriverRefusal` return. Reword it to 'the completed result's per-item refusal member holds QSL's typed unbound-element refusal unchanged'.
- fnd: FND-006
  severity: low
  confidence: medium
  method: spec-review/base
  check: other
  artifact_id: AD-001-accounting
  related_ids:
  - FR-009-scope
  path: spec/assurance/AD-001-driver-architecture.md
  lines: 155-160
  excerpt: 'FR-010 qualifies the CG input: only relation-bound items enter negotiation and generation. Selection never discards the need to account for each requested item: a settled item in FR-009''s final outcome, a refused item in FR-001''s per-item refusal member. Proving extends generation with provider execution, QSL replay and CG settlement; a `generate` request does not imply that a proof was executed.'
  finding: The AD-001 accounting sentence places a settled item 'in FR-009's final outcome'. FR-009's outcome exists only for a proof request, and the same paragraph says a `generate` request implies no proof. For a generate run a settled item is accounted for by CG's settlement in `Driven` (FR-001 Outputs). Say 'a settled item in `Driven`'s CG settlement (and, for a proof, its FR-009 terminal record)'.
- fnd: FND-007
  severity: low
  confidence: medium
  method: spec-review/base
  check: ambiguous
  artifact_id: FR-009-outputs
  related_ids:
  - FR-009-desc
  path: spec/functional/FR-009-settle-execute-and-replay-results.md
  lines: 55-57
  excerpt: For each export-refused item, no terminal record; its refusal stays in `Driven`'s per-item refusal member (FR-001), alongside the records of the items that settled.
  finding: FR-009 Outputs says B's refusal stays in `Driven` 'alongside the records of the items that settled'. The terminal records are in the `quire-outcome/1` outcome document, not in `Driven`, so the two are not side by side in one output. No requirement says whether the CLI's outcome document carries the refused item at all, so a CLI `prove` of a partly bound package would currently report A and C and silently drop B. Either name the outcome document's handling of refused items or drop 'alongside'.
bindings: []
```

## Recommendations

- FND-001: write 'an item the export (QSL FR-307) refused' in the FR-001 Behavior bullet and AC-17.
- FND-002: state the request index a settlement carries after an omission, whether an export-refused item keeps an FR-330 `DriverItem` position, and give each member entry its request index.
- FND-003: add the export call to FR-001's step list with its whole-run failure classification. Classify an unknown-key refusal for a driver-built key as `runtime_invariant`, and say what a package with no abstraction relation does.
- FND-004: add the proof-request precondition to FR-009-AC-16 and a proof run to TC-017, or verify the terminal half in TC-016.
- FND-005..FND-007: reword the TC-017 bullet, the AD-001 accounting sentence and FR-009's 'alongside'.

## Verdict

Not mergeable as is: four medium findings (FND-001..FND-004) leave the per-item/whole-run boundary and FR-009-AC-16's oracle open. Clean: the FR-307 refusal shape, the all-refused case against TC-003 step 5 and FR-010-AC-5, the FR-009 narrowing, AD-001's per-CG-request wording, and the behaviour-not-changelog check.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | Keeping B's `DriverItem` in the FR-330 position vector contradicts FR-010 unless something says otherwise. FR-001 Behavior says each `DriverItem` position carries 'its exact admitted node and scalar result selection', and CG generation receives the complete vector (FR-001-AC-15). So B's node reaches CG, against FR-010's 'SHALL not send the package's unbound item set to CG', TC-017's 'No unbound item reaches CG', and QSL ADR-017 AR-6. State that an export-refused item's position is accounting-only (no node or selection CG can lower), or drop it from the vector and say how driver indices are kept. | spec/functional/FR-001-orchestrate-the-driver-steps.md:93-98 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | The CG dependency is understated, and 'no node' asks for the wrong CG change. Read in quire-contract-codegen main @7373156 (src/core/contract_package.rs `PackageItems::new`, src/routed/capability.rs `negotiate_contract_package`). (1) CG requires positions.len() == the admitted package's feature_accounting.len(), and each position's node_id must equal its accounting record's node_id (`Count`, `AccountingIdentity`). B's record is in the admitted package, so dropping B from the vector fails `Count`; dropping is not the CG-free alternative the PR body offers. A node-less position fails `AccountingIdentity` unless CG skips the check. (2) CG requires the FR-331 envelope's item count to equal the number of `DriverItem` positions (`EnvelopeItemCount`). Sending an A,C envelope (FR-010-AC-4) while B keeps a counted position fails, so FR-010-AC-4 itself depends on the CG change. (3) CG numbers request_index only over `DriverItem` entries (`driver_to_accounting`). A new variant that is not counted gives C CG index 1, not 2, which breaks FR-001-AC-17 unless CG counts it. So both options need CG work. The conservative default is still to keep the position, because it preserves CG's package accounting and the driver-index join. But specify it as an excluded position that keeps its node_id (needed for CG's identity check; the node is already in the `ContractPackageV1` CG receives), counts toward driver indices, and is excluded from the FR-331 envelope count, negotiation, routing and generation. Name all three CG checks in the Status dependency, with a CG ticket. | spec/functional/FR-001-orchestrate-the-driver-steps.md:164-167, 311-315 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Fix 86fe419: The per-item refusal member SHALL hold only an item the export refused. It holds no negotiation settlement (`invalid-request`, `unknown-backend`, `NoDerivableClaim`) and no scalar `RunRefusal` or `BindRefusal`; FR-001-AC-16 still stops the composition for the latter. |
| FND-002 | fixed | Fix 86fe419: The two halves of `Driven` join on the item's original driver request index. Settlements and generation output stay keyed by that index, so an omitted refused item leaves a gap and C keeps driver index 2 when B (index 1) is omitted; the refusal member carries the occurrence key and that same driver index. The FR-330 position vector keeps one `DriverItem` per original request item (FR-001-AC-15). |
| FND-003 | still-open | Fix 86fe419 resolves most of this finding. The export call is now placed after E5 and before step 3. An export that fails as a whole is a command-level refusal, and a driver-built key reported as naming no requirement record is a `runtime_invariant` fault (AC-19). Two things remain. (a) The no-relation bullet ('A package that carries no abstraction relation binds no element; the export then returns a bound result for an item that references no element and a refusal for every other item') has no QSL source. QSL FR-307's Inputs require a `CheckedPackage` carrying a `CheckedAbstractionRelation`, and FR-306 writes that node only 'when ... [the] unit declares an abstraction relation'. So FR-307 does not define the export on a package without a relation. The derivation also has a large consequence: every unit with no relation declaration that reads a model member or ranges over a population would refuse those items, so they would never reach CG. That changes today's path and conflicts with FR-010's conditional Description ('When an abstraction relation binds a requested item'). The bullet also states QSL's behaviour as fact, with no SHALL. Either skip the export when the package carries no relation (all items go to CG and the member is empty), or have QSL define the relation-less export first, and record that as an owner decision. (b) A whole-export refusal names no `DriverStage`, although FR-001-AC-5 requires each command-level refusal to carry its step. |
| FND-004 | fixed | Fix 86fe419: / FR-009-AC-16 / For the FR-010-AC-4 vector and a proof request that reaches per-item settlement, the outcome holds one terminal record for each of A and C and none for B, whose refusal appears only in `Driven`'s per-item refusal member. / Test (TC-017) /  (TC-017: 'With a fixture provider that returns a terminal result for each item in the CG request, a proof request holds terminal records for A and C and none for B (FR-009-AC-16); no Kani execution is needed.') |
| FND-005 | fixed | Fix 86fe419: No unbound item reaches CG. Where a required binding is absent, the item's QSL typed unbound-element refusal appears unchanged in the per-item refusal member of the completed result. |
| FND-006 | fixed | Fix 86fe419: Selection never discards the need to account for each requested item: a settled item in CG's settlement and generation output (and, for a proof, in FR-009's outcome), a refused item in FR-001's per-item refusal member. |
| FND-007 | fixed | Fix 86fe419: For each export-refused item, no terminal record; its refusal stays in `Driven`'s per-item refusal member (FR-001), and it does not appear in the `quire-outcome/1` document that holds the records of the items that settled. |
| FND-003 | fixed | Fix 01ca369: When the export fails as a whole, the driver SHALL refuse the run with the export's typed refusal, carrying the bindings-export step. ... When the package declares no abstraction relation, the driver SHALL skip the export, send every item to CG as FR-001 otherwise states, and leave the per-item refusal member empty.  (FR-001-AC-19 and FR-001-AC-21; the PR body records the skip as an owner decision) |
| FND-008 | still-open | Fix 01ca369 adds to FR-001 Outputs: 'A refused item's position is accounting-only: it carries no admitted node and no claim, and CG receives nothing from it that it can lower, route or generate.' The normative Behavior bullet (FR-001 lines 162-165) is unchanged. It still says 'Each original driver request item contributes one CG `DriverItem` position with its exact admitted node and scalar result selection, whether or not routing later selects it', so FR-001 now contradicts itself. Qualify that bullet (and FR-001-AC-15) for export-refused items. Also, nothing shows that CG's FR-330 `DriverItem` admits a position with no node, and FR-001 Status says this composition is 'not a QSL/CG API change'. Confirm that CG's type supports an accounting-only position, or name the CG dependency. |
| FND-008 | fixed | Fix fb87cf2: Each original driver request item the export bound contributes one CG `DriverItem` position with its exact admitted node and scalar result selection, whether or not routing later selects it; an export-refused item contributes an accounting-only position at its driver index with no node, claim or result selection; ...  (FR-001-AC-15 qualified to match; FR-001 Status names the CG dependency) |
