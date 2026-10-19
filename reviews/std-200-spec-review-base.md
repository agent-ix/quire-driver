---
id: SR-6005
title: "STD-200 spec-review/base of the bound-items CG request extension"
type: SpecReview
analysis: base
scope: "agent-ix/quire-driver PR #84 @488bd1d; spec/functional/FR-010-send-only-bound-items-to-codegen.md, spec/test-cases/TC-017-only-bound-items-reach-codegen.md, spec/tests.md; ticket STD-200"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-010
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-017
    type: reviews
---
# STD-200: spec-review/base

## Summary

Ticket: STD-200. The review covered `git diff refs/remotes/origin/main...HEAD` (one commit, 488bd1d). It checked that diff against QSL FR-307, TC-1082 and ADR-017 AR-4/AR-6 (quire-spec-language `refs/remotes/origin/main` @d7add3a), QSpec FR-331, and this repository's FR-001, FR-009 and TC-003.

The partition vector matches QSL. A is bound per FR-307-AC-1. B is refused per FR-307-AC-1. C is bound with an empty binding set per FR-307-AC-6. The all-refused repeat matches FR-307-AC-6 and TC-1082 step 1. Extending FR-010 rather than adding a requirement is correct. FR-010 already owns "only bound items reach CG", and QSL FR-307's Downstream section assigns request building to the driver. A new FR would duplicate FR-010-AC-1 and AC-3. The cross-repository references follow this repository's convention: an `ix://` link with a `#AC` anchor, as FR-010 already uses for QSpec FR-353-AC-3, and "QSL FR-NNN" in prose. The wording describes behaviour, not a changelog.

The deliverable is incomplete, though. The PR and the TC cite FR-010-AC-4, but the commit adds no such criterion. The all-refused rule and the refusal-return surface are also left open.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-010-AC-4 is never defined. The AC table still ends at AC-3, so the rule exists only as a Behavior bullet, while TC-017, spec/tests.md and the PR body all cite FR-010-AC-4. | spec/functional/FR-010-send-only-bound-items-to-codegen.md:36-42 |
| FND-002 | medium | The all-refused case is ambiguous. "No item is sent to CG" follows from ADR-017 AR-6. The PR's reading, "send nothing to CG" (no CG call at all), is a new rule that no QSL/QSpec text states, and it diverges from FR-001/TC-003, where an empty request still negotiates zero items. This needs an owner decision. | spec/test-cases/TC-017-only-bound-items-reach-codegen.md:43-44 |
| FND-003 | medium | "The driver SHALL return its refusal to the caller unchanged" names no output surface. It also does not say how a per-item refusal coexists with FR-009's "exactly one FR-331 terminal record for each requested item" when A and C reach settlement. | spec/functional/FR-010-send-only-bound-items-to-codegen.md:27-31 |
| FND-004 | low | The new Behavior bullet repeats the existing absent-binding bullet (lines 32-34, AC-3). The new Dependencies bullet repeats the first one ("QSL FR-307 defines the bound-item export and per-item results"). | spec/functional/FR-010-send-only-bound-items-to-codegen.md:32-34 |
| FND-005 | low | The second vector's wording "only B requested alongside a second refused clause item" contradicts itself. The expected result says B's refusal is "reported" rather than returned unchanged with its population key and `DomainPackageRef`. | spec/test-cases/TC-017-only-bound-items-reach-codegen.md:29-30 |

## Examined scope

```yaml
scope:
  - {id: FR-010, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: examined, excerpt: "The driver SHALL build the CG request from the bound results of the QSL export's per-item partition (QSL FR-307: each requested item is either a bound result, with an empty binding set when the item references no model element, or a refusal). A refused item SHALL contribute nothing to the request, and the driver SHALL return its refusal to the caller unchanged."}
  - {id: FR-010-AC-1, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: examined, excerpt: "Given a package whose abstraction relation binds a subset of requested items, CG negotiation receives exactly that subset and receives no unbound item."}
  - {id: FR-010-AC-2, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: examined, excerpt: "CG generation receives only items that the same admitted relation bound and that negotiation settled for generation."}
  - {id: FR-010-AC-3, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: examined, excerpt: "When a binding required for emission is absent, the driver returns the QSL-defined typed unbound-element refusal and sends no unbound item to CG."}
  - {id: FR-010-dependencies, path: spec/functional/FR-010-send-only-bound-items-to-codegen.md, role: examined, excerpt: "QSL FR-307-AC-6 owns the bound-or-refused result kinds the driver reads; this requirement owns the request the driver builds from them."}
  - {id: TC-017, path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md, role: examined, excerpt: "Partition vector (FR-010-AC-4): bind nothing but the ConfigVersion object type, then request three items: A, a clause item reading only ConfigVersion fields; B, an item whose extent domain ranges over the unbound ConfigVersion population; C, an item referencing no model element. Repeat with nothing bound and only B requested alongside a second refused clause item."}
  - {id: tests.md-FR-010-row, path: spec/tests.md, role: examined, excerpt: "| FR-010 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4 | TC-017 | Planned in quire-integration; QSL AR2 prerequisite |"}
  - {id: QSL-FR-307-AC-1, path: "quire-spec-language:spec/functional/FR-307-export-the-bindings-each-item-references.md", role: context_only, excerpt: "With only the ConfigVersion object type bound, a request for a clause item that reads only ConfigVersion fields and an item whose extent domain ranges over the unbound ConfigVersion population returns the first item with its one ObjectBinding and refuses the second missing_declaration/missing-name, naming the population's key and its DomainPackageRef identity, with no bindings."}
  - {id: QSL-FR-307-AC-6, path: "quire-spec-language:spec/functional/FR-307-export-the-bindings-each-item-references.md", role: context_only, excerpt: "With no element bound, a request for a clause item and an extent-domain item returns two missing_declaration/missing-name refusals, one per item, each with no bindings, and the export itself succeeds. A requested item that references no model element returns a bound result with an empty binding set, not a refusal."}
  - {id: QSL-TC-1082, path: "quire-spec-language:spec/test-cases/TC-1082-export-returns-bound-or-refused-per-item.md", role: context_only, excerpt: "With no element bound, request a clause item reading only ConfigVersion fields and an item whose extent domain ranges over the ConfigVersion population. Request an item that references no model element."}
  - {id: QSL-ADR-017-AR-6, path: "quire-spec-language:spec/decisions/ADR-017-protocol-refinement-abstraction-boundaries.md", role: context_only, excerpt: "the QSpec FR-331 request, holding only bound items ... unbound items never reach CG"}
  - {id: QSL-ADR-017-AR-4, path: "quire-spec-language:spec/decisions/ADR-017-protocol-refinement-abstraction-boundaries.md", role: context_only, excerpt: "For an emission onto implementation code, the driver puts only the bound items in the QSpec FR-331 request it sends CG, so CG receives no new input"}
  - {id: FR-001, path: spec/functional/FR-001-orchestrate-the-driver-steps.md, role: context_only, excerpt: "The driver SHALL call route only after negotiation, and only with the items settled supported. When no item settles supported, it SHALL not call route."}
  - {id: TC-003, path: spec/test-cases/TC-003-route-after-negotiation.md, role: context_only, excerpt: "Step 5: route is never called, generation is never called, and the run still completes (an empty request negotiates zero items)."}
  - {id: FR-009, path: spec/functional/FR-009-settle-execute-and-replay-results.md, role: context_only, excerpt: "For each proof request that reaches per-item settlement, the driver SHALL produce exactly one FR-331 terminal record for each requested item."}
findings:
  - fnd: FND-001
    severity: high
    confidence: high
    method: spec-review/base
    check: untestable-ac
    artifact_id: FR-010
    related_ids: [TC-017, FR-010-AC-3]
    path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
    lines: "36-42"
    excerpt: |-
      | FR-010-AC-1 | Given a package whose abstraction relation binds a subset of requested items, CG negotiation receives exactly that subset and receives no unbound item. | Test |
      | FR-010-AC-2 | CG generation receives only items that the same admitted relation bound and that negotiation settled for generation. | Test |
      | FR-010-AC-3 | When a binding required for emission is absent, the driver returns the QSL-defined typed unbound-element refusal and sends no unbound item to CG. | Test |
    finding: "FR-010-AC-4 is never defined. The AC table still ends at AC-3, so the rule exists only as a Behavior bullet, while TC-017, spec/tests.md and the PR body all cite FR-010-AC-4. STD-200 asks for an acceptance criterion, and quoin matrix lists only FR-010-AC-1..3."
  - fnd: FND-002
    severity: medium
    confidence: medium
    method: spec-review/base
    check: ambiguous
    artifact_id: TC-017
    related_ids: [FR-010, FR-001, TC-003]
    path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
    lines: "43-44"
    excerpt: |-
      When every item is refused, no item is sent to CG and each
        refusal is reported.
    finding: "The all-refused case is ambiguous. 'No item is sent to CG' follows from ADR-017 AR-6. The PR's reading, 'send nothing to CG' (no CG call at all), is a new rule that no QSL/QSpec text states, and it diverges from FR-001/TC-003, where an empty request still negotiates zero items. This needs an owner decision."
  - fnd: FND-003
    severity: medium
    confidence: medium
    method: spec-review/base
    check: ambiguous
    artifact_id: FR-010
    related_ids: [FR-009]
    path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
    lines: "27-31"
    excerpt: |-
      The driver SHALL build the CG request from the bound results of the QSL
        export's per-item partition (QSL FR-307: each requested item is either a
        bound result, with an empty binding set when the item references no model
        element, or a refusal). A refused item SHALL contribute nothing to the
        request, and the driver SHALL return its refusal to the caller unchanged.
    finding: "'The driver SHALL return its refusal to the caller unchanged' names no output surface. It also does not say how a per-item refusal coexists with FR-009's 'exactly one FR-331 terminal record for each requested item' when A and C reach settlement."
  - fnd: FND-004
    severity: low
    confidence: medium
    method: spec-review/base
    check: other
    artifact_id: FR-010
    related_ids: [FR-010-AC-3]
    path: spec/functional/FR-010-send-only-bound-items-to-codegen.md
    lines: "32-34"
    excerpt: |-
      Where a required binding is absent, the driver SHALL preserve the typed
        unbound-element refusal defined by the owning QSL contract; it SHALL not
        invent a CG item or silently treat the item as bound.
    finding: "The new Behavior bullet repeats the existing absent-binding bullet (lines 32-34, AC-3). The new Dependencies bullet repeats the first one ('QSL FR-307 defines the bound-item export and per-item results')."
  - fnd: FND-005
    severity: low
    confidence: high
    method: spec-review/base
    check: ambiguous
    artifact_id: TC-017
    related_ids: [FR-010]
    path: spec/test-cases/TC-017-only-bound-items-reach-codegen.md
    lines: "29-30"
    excerpt: |-
      Repeat with nothing
      bound and only B requested alongside a second refused clause item.
    finding: "The second vector's wording 'only B requested alongside a second refused clause item' contradicts itself. The expected result says B's refusal is 'reported' rather than returned unchanged with its population key and DomainPackageRef."
bindings: []
```

## Recommendations

- FND-001: add FR-010-AC-4 to the AC table, with verification Test (TC-017), stating the partition vector's oracle: the CG request carries A and C, not B, and B's QSL refusal is returned unchanged.
- FND-002: the conservative choice adds no new rule. State only that no refused item reaches CG and that an all-refused request carries zero items, then defer to FR-001's existing empty-request path, which TC-003 step 5 shows still negotiates zero items and skips `route` and generation. Skipping the CG call entirely is a new driver rule. It changes FR-001's observable call sequence, so the owner would need to decide it and FR-001 would need amending.
- FND-003: name the output that carries per-item refusals, and say whether a refused item is a "requested item" under FR-009's one-terminal-record rule.
- FND-004/FND-005: merge the duplicate bullets. Reword the second vector to mirror TC-1082 step 1, and assert that the refusal is returned unchanged.

## Verdict

Not mergeable. FND-001 (high) means the requirement the ticket asks for does not exist. FND-002 needs an owner decision or a reword to the derived reading. Everything else checked is consistent: the vector against QSL FR-307-AC-1/AC-6 and TC-1082, extending FR-010 instead of adding an FR, the cross-repository reference style, and the behaviour-not-changelog wording.
