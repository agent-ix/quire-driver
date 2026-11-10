---
id: SR-14855
title: "IR-1494 requirement-record projection spec-review/integrity review"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-driver; IR-1494; PR #146; spec/functional/FR-020, spec/test-cases/TC-034, FR-001, FR-010, spec/spec.md, spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-driver/FR-020, type: references }
  - { target: ix://agent-ix/quire-driver/TC-034, type: references }
---
## Summary

Ticket: IR-1494. Reviewer spec-review/integrity pass over PR #146 (head 5b0e7a2), spec-only: new FR-020 and TC-034 plus FR-001/FR-010 cross-references and index updates. Verified against CG FR-022 on quire-contract-codegen origin/main, CG PR #772 (code/ir-304-slice-a-record-input), QSL origin/main (qsl-route request.rs, qsl-semantics claims.rs and claims/map.rs) and the driver code (src/drive.rs, src/refusal.rs, src/upstream.rs, Cargo.toml, quire-cli/Cargo.toml).

## Verdict

Integrity is otherwise sound: ids are unique, FR-020/TC-034 are added to spec.md relationships and index and to tests.md (both tables); FR-001 and FR-010 cross-references resolve; quire validate passes for every touched file and quire matrix lists FR-020-AC-1..11 (untagged as expected for a spec-only change; AC-11 reports method-without-symbol because it is Inspection). One stale index line remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | spec.md section 5 count left at FR-014/TC-028 while tests.md was updated to FR-020/TC-034 | spec/spec.md |

## Reviewed Units

```json
{
  "clean": false,
  "scope": [
    {
      "id": "FR-020",
      "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
      "role": "examined",
      "excerpt": "One application node can carry several scalar occurrences, each with its own result bound and path condition (QSL-266, QSL-700, QSL-904). The node id alone cannot say which claim an item makes. For each routed Kani item the driver SHALL hand CG the QSL record of that exact occurrence: the item's `RoutedRequirement`, and the package's `RequirementRecords` on the Kani generation context. A record that cannot be projected exactly is a typed driver fault, never a guess. CG owns the types and every comparison ([CG FR-022](ix://agent-ix/quire-contract-codegen/FR-022), \"Per-occurrence requirement ..."
    },
    {
      "id": "TC-034",
      "path": "spec/test-cases/TC-034-project-requirement-records-into-codegen.md",
      "role": "examined",
      "excerpt": "Each mutant must make the named assertion fail; none is equivalent to the correct driver. | Mutant | Caught by | |---|---| | Build the occurrence from the node id with role and ordinal dropped | AC-1 occurrence equality; AC-3 (two items collapse) | | Reverse the path condition, or flip a `holds` value | AC-1 ordered-guard equality | | Drop the conversion context or fill it from the node | AC-1 `None` and `Some` cases | | Build records only for routed items | AC-2 refused item's record; AC-6 B's record | | Fill a missing record from the item's own fields | AC-4 fault and no-generation-call a..."
    },
    {
      "id": "spec.md#5",
      "path": "spec/spec.md",
      "role": "examined",
      "excerpt": "[TM-001](tests.md) integrates FR-001 through FR-014, and TC-001 through TC-028."
    },
    {
      "id": "TM-001",
      "path": "spec/tests.md",
      "role": "examined",
      "excerpt": "It covers FR-001 through FR-020 and TC-001 through TC-034."
    }
  ],
  "findings": [
    {
      "fnd": "FND-001",
      "severity": "low",
      "confidence": "high",
      "check": "other",
      "artifact_id": "spec.md#5",
      "related_ids": [
        "TM-001"
      ],
      "path": "spec/spec.md",
      "lines": "258-259",
      "excerpt": "[TM-001](tests.md) integrates FR-001 through FR-014, and TC-001 through TC-028.",
      "finding": "spec.md section 5 still says TM-001 integrates 'FR-001 through FR-014, and TC-001 through TC-028' while this PR updates the same count in tests.md to FR-020/TC-034. The PR edits spec.md's index for FR-020/TC-034, so the count belongs in this PR: change it to FR-001 through FR-020 and TC-001 through TC-034 (matching tests.md).",
      "method": "spec-review/integrity"
    }
  ]
}
```

## New findings (disposition pass 1)

Re-review of head ab47adb (commits 5e98dc5, ab47adb) against the review-pass findings.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | Linear comment payloads (*.comment.md) committed under reviews/; committed SR copies now predate these dispositions | reviews/ |

```json
[
  {
    "fnd": "FND-002",
    "severity": "low",
    "confidence": "high",
    "check": "other",
    "artifact_id": "reviews/ir-1494-spec-review-base.comment.md",
    "related_ids": [],
    "path": "reviews/",
    "lines": "six *.comment.md files",
    "excerpt": "reviews/ir-1494-spec-review-{base,integrity,ears-conformance,dependency,failure-domain,criterion-strength}.comment.md",
    "finding": "The fix round committed the six Linear comment payloads (*.comment.md) under reviews/ beside the SR files; no other review in reviews/ or spec/reviews/ commits comment payloads, and the tracker already holds them. Drop them. The six committed SR files are byte-identical to the review-pass versions, so they lack this round's ## Dispositions and new findings: copy the updated SR files over them in the next fix commit."
  }
]
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head ab47adb: [TM-001](tests.md) integrates FR-001 through FR-020, and TC-001 through TC-034. |
