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
    "finding": "The fix round committed the six Linear comment payloads (*.comment.md) under reviews/ beside the SR files; no other review in reviews/ or spec/reviews/ commits comment payloads, and the tracker already holds them. Drop them. The six committed SR files are byte-identical to the review-pass versions, so they lack this round's ## New findings (disposition pass 2)

Re-review of head ddf8761 (commit ddf8761 on ab47adb).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | FR-010 and FR-001 cross-references still say every bound/routed Kani item carries a requirement, contradicting the value-validity-only rule | spec/functional/FR-010-send-only-bound-items-to-codegen.md |

```json
[
  {
    "fnd": "FND-003",
    "severity": "low",
    "confidence": "high",
    "check": "other",
    "artifact_id": "FR-010",
    "related_ids": [
      "FR-001",
      "FR-020"
    ],
    "path": "spec/functional/FR-010-send-only-bound-items-to-codegen.md",
    "lines": "Dependencies, FR-020 line; FR-001 Inputs 'Each routed Kani item also carries'",
    "excerpt": "- [FR-020](FR-020-project-requirement-records-into-codegen-input.md) attaches a requirement record to each bound item sent to generation and none to a refused item.",
    "finding": "FR-010 says FR-020 'attaches a requirement record to each bound item sent to generation' and FR-001 Inputs says 'Each routed Kani item also carries its QSL requirement record'. Under the round-2 text a bound non-value-validity item carries none. Align both with whatever rule base FND-003 settles on (under the recommended rule, every item routed to Kani is value-validity and the FR-001 wording becomes true again; FR-010's 'each bound item' still needs 'routed to Kani')."
  }
]
```

## New findings (disposition pass 4)

Re-review of head 2674ff5 (commit 2674ff5 on e5bb946).

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Outputs still says every bound value-validity item's requirement is 'attached with with_requirement'; Behavior attaches only to the Kani-routed subset | spec/functional/FR-020-project-requirement-records-into-codegen-input.md |

```json
[
  {
    "fnd": "FND-004",
    "severity": "low",
    "confidence": "high",
    "check": "ambiguous",
    "artifact_id": "FR-020",
    "related_ids": [],
    "path": "spec/functional/FR-020-project-requirement-records-into-codegen-input.md",
    "lines": "Outputs first bullet vs Behavior 'attach the projected RoutedRequirement ... for the subset negotiation routes to Kani'",
    "excerpt": "- For each export-bound `value-validity` item that has a record, selected with `OwnResult`, a CG `RoutedRequirement { occurrence, result_bound, conversion_context, path_condition }`, attached with `RoutedGenerationItem::with_requirement`.",
    "finding": "The Outputs bullet says a RoutedRequirement is produced for each export-bound value-validity item with a record and 'attached with RoutedGenerationItem::with_requirement'. Behavior now builds it for every such item at lowering but attaches it only to generation items negotiation routes to Kani (none for an item routed to a process backend or not routed). Reword Outputs to 'projected for each ... and attached to the generation item of each one negotiation routes to Kani'."
  }
]
```

## Dispositions and new findings: copy the updated SR files over them in the next fix commit."
  }
]
```

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, head ab47adb: [TM-001](tests.md) integrates FR-001 through FR-020, and TC-001 through TC-034. |
| FND-002 | fixed | Round 2, head ddf8761: reviews/ now holds only the six SR files (ir-1494-spec-review-{base,criterion-strength,dependency,ears-conformance,failure-domain,integrity}.md), byte-identical to the round-1 reviewer versions; the *.comment.md payloads are removed. |
| FND-003 | fixed | Round 3, head e5bb946: - [FR-020](FR-020-project-requirement-records-into-codegen-input.md) attaches a requirement record to each bound item routed to Kani and none to a refused item. |

