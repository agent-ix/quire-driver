---
id: FR-018
title: "Admit authentic source-record witnesses"
type: FR
relationships:
  - target: "ix://agent-ix/quire-driver/US-001"
    type: implements
---
# FR-018: Admit authentic source-record witnesses

## Description

The driver SHALL admit source-record witnesses from the immutable QSL Emission
that produced the strictly read IR package, preserving producer authority and
all original native identity and value fields.

## Inputs

The component accepts a genuine QSL Emission, native IR reader evidence and
package-reader limits, and required explicit work and storage limits for the
separate native source-record join.

## Outputs

The public SourceRecordAdmissionErrorCode SHALL expose the exact stable spellings
`read_refused`, `read_incomplete` and `join` through `as_str`, a complete `all`
catalog, and exact case-sensitive `from_code` decoding. Unknown spellings SHALL
return None; native payloads remain unchanged.

The component returns an opaque owner containing the exact reader-admitted
package and its native SelectedExactIdentities, or the original typed native
reader refusal, incompleteness, or projection failure.

## Behavior

The public admission function SHALL read the emission's own bytes using IR's
public strict reader and native evidence. Ordinary callers supply the emission's
evidence; model-dependent callers extend that original native evidence with the
original domain documents using IR's public insertion API. Native reading, not a
driver trust flag or copied schema reader, validates those documents. The function SHALL obtain source-record
rows only from that same emission's immutable getter. The function SHALL map every
source authority, source identity, qualified declaration segment, structural node,
opaque record key, ordered original declared member name, admitted binding name,
presence, type node and Exact value type directly into the native IR row.
The function SHALL invoke the native finite source-record join from empty selected
identities with a budget constructed from the caller's work and storage limits.
It SHALL NOT mint MemberId, decode opaque Exact values through a wire
schema, reconstruct a plan or runtime environment, or accept caller-created rows
as producer authority. Projection SHALL use the retained package and identities
with a separate caller projection allowance and retain native error payload/work.

This component has independent native reader, join and projection allowances.
The native join charges work and logical storage against the caller's explicit
limits and reports its own success charge or typed resource refusal. The driver
preserves either exhausted resource cause and returns no owner after refusal.
The component makes no cumulative preparation-work or complete storage
qualification: producer-row materialization is outside the native join allowance,
and the separate join charge is not composed with the remaining preparation
allowance. Model-owned identity
joining, complete generated-source closure, clock admission, four-entry reachability
and executable Program construction remain outside this component.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-018-AC-1 | Genuine ordinary source records emitted by public QSL and strictly admitted by IR retain all complete joined native rows and opaque record keys through driver admission/projection, including required, optional and Option fields and recursive/shared types where the owners admit them. | Test |
| FR-018-AC-2 | Native structural/source/owner/member-order/presence/type mismatch and duplicate controls return the original native error payload and work; independent join exhaustion returns no admitted owner. Counterfactual native-row controls establish structural checks only and never producer authority. | Test |
| FR-018-AC-3 | The public API accepts genuine immutable Emission authority and binds its rows to its own strict reader output; caller-created row structs cannot enter production admission. Native reader refusal/incompleteness remain distinct. | Test |
| FR-018-AC-4 | Projection uses the retained package and native identities with its independent limit, retaining original native type and complete source_record_view without schema reconstruction or invented member identities. | Test |
| FR-018-AC-5 | An authentic emission joins successfully at its measured native work and storage charge. One unit below either dimension returns no owner and preserves the original native exhausted-work or exhausted-storage payload. The outer preparation classifier maps native exhausted storage to `storage` and incomplete identity to `projection` without losing the native payload. This does not claim cumulative preparation accounting. | Test |

## Qualification Status

Qualification is partial. The production tests execute genuine ordinary-source
record admission and Tagged/recursive Tree parameter projections successfully,
including complete producer-field equality, native opaque keys and record views,
join exhaustion, reader incompleteness, structural counterfactuals, opaque-claim
authority separation and the stable error-code catalog. Earlier affected
monitor preparation and dependency-edge controls were also executed; this
update's focused evidence is the source-record target.

After adopting merged QSL-1741, the original seven production
`tests/source_record_admission.rs` controls executed successfully. The genuine
final PlanRoot now joins every immutable final producer row (30 rows in the
current fixture), including generated records. The test requires a nonempty
producer set and exact native row-count equality rather than a fixed count or
filtered subset. It compares every source/structural/declaration/opaque-key
field and every ordered original member identity, admitted name, presence,
type node and Exact value type with that same emission's producer rows.

The generated `origin_tick` to `origin` and `amount` to `value` binding renames
are exercised against real final producer rows, with the original member
identities retained. Every final record and member type ID resolves in the
strictly admitted package. Missing original domain evidence still preserves
the exact native reader refusal. No driver row repair, filtering, fabricated
witness or emitted-package mutation supplies this success. Historical review
artifacts retain the earlier compilation and producer-rename blockers as
historical evidence; they are not current qualification status.

This independent record component contributes to the source-record slice
of [FR-015](FR-015-prepare-an-admitted-monitor-program.md) and IR-721 AC1 only.
Complete IR-721 AC1, monitor preparation and whole-preparation limits remain Planned.

## Dependencies

- [FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns complete monitor admission.
- QSL owns immutable Emission producer rows and final emitted identity rekeying.
- IR/model owns strict reading, finite native record joining and exact projection.
