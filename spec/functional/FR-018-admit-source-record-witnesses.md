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

The component accepts a genuine QSL Emission, native IR reader evidence and package-reader limits,
and a separate finite native source-record join allowance.

## Outputs

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
identities. It SHALL NOT mint MemberId, decode opaque Exact values through a wire
schema, reconstruct a plan or runtime environment, or accept caller-created rows
as producer authority. Projection SHALL use the retained package and identities
with a separate caller projection allowance and retain native error payload/work.

This component has independent native reader, join and projection allowances.
It makes no cumulative preparation-work or storage qualification: the native join
returns no success-work receipt, and producer-row materialization is outside its
native join allowance. No receipt is guessed or recomputed. Model-owned identity
joining, complete generated-source closure, clock admission, four-entry reachability
and executable Program construction remain outside this component.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-018-AC-1 | Genuine ordinary source records emitted by public QSL and strictly admitted by IR retain all complete joined native rows and opaque record keys through driver admission/projection, including required, optional and Option fields and recursive/shared types where the owners admit them. | Test |
| FR-018-AC-2 | Native structural/source/owner/member-order/presence/type mismatch and duplicate controls return the original native error payload and work; independent join exhaustion returns no admitted owner. Counterfactual native-row controls establish structural checks only and never producer authority. | Test |
| FR-018-AC-3 | The public API accepts genuine immutable Emission authority and binds its rows to its own strict reader output; caller-created row structs cannot enter production admission. Native reader refusal/incompleteness remain distinct. | Test |
| FR-018-AC-4 | Projection uses the retained package and native identities with its independent limit, retaining original native type and complete source_record_view without schema reconstruction or invented member identities. | Test |

## Qualification Status

Planned. This independent record component contributes to the source-record slice
of [FR-015](FR-015-prepare-an-admitted-monitor-program.md) and IR-721 AC1 only.
Complete IR-721 AC1, monitor preparation and whole-preparation limits remain Planned.

## Dependencies

- [FR-015](FR-015-prepare-an-admitted-monitor-program.md) owns complete monitor admission.
- QSL owns immutable Emission producer rows and final emitted identity rekeying.
- IR/model owns strict reading, finite native record joining and exact projection.
