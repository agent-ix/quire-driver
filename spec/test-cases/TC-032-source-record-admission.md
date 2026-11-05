---
id: TC-032
title: "Admit immutable source-record producer witnesses"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-018
    type: verifies
---
# TC-032: Admit immutable source-record producer witnesses

## Description

Verify the independent [FR-018](../functional/FR-018-admit-source-record-witnesses.md)
source-record component using authentic ordinary QSL source and native IR joins.
No monitor Program, clock, complete closure or cumulative preparation allowance
is inferred from this component.

## Test Procedure

1. Parse, check and emit ordinary Point, Tagged and recursive Tree declarations
   through public QSL lifecycle operations. Tagged includes a nested shared
   Point type, an optional Boolean field and a required Option<Integer> field.
   Tree includes bounded recursive Sequence<Tree> and Option<Tree> fields.
2. Call the public driver component with the unmodified immutable Emission.
   Compare every complete native joined row with the original producer row:
   source identity/authority, qualified declaration, structural node, opaque
   record key and every ordered member identity/name/presence/type node/value.
3. Also emit a genuine final PlanRoot containing ordinary source records via
   public QSL emit_plan_root. Add its original model document through the native
   IR evidence insertion API. Admit its untouched final emission and verify the
   final structural/member type IDs resolve in that exact admitted package.
   Require nonempty producer rows, exact joined/producer count equality and
   complete native-field equality for every row, including generated rows.
   Assert genuine `origin_tick` to `origin` and `amount` to `value` admitted
   binding renames while retaining the original declared member identities.
   Without the required native document evidence, compare the driver refusal
   against the original native reader refusal.
4. Read actual ordinary callable parameter IDs from the admitted package;
   project those parameter types through the driver owner. Compare native
   Composite keys and complete source_record_view rows. Require native failure
   without witnesses and with zero projection allowance. Drop the emission and
   retain the owned native rows.
5. Compare zero join allowance and zero reader byte allowance errors to the
   complete original native owner payloads. Return no partial admitted owner.
6. Separately mutate cloned native rows as counterfactual structural controls:
   foreign source/declaration, missing/reordered members, wrong presence/type,
   empty original field name or wrong admitted name, duplicate rows, and rows from a genuine
   independently emitted foreign package. Exercise IR's native join directly.
   These controls cannot enter the public producer-authority API and confer no
   authentication on caller-created opaque keys or value types.

## Expected Results

The immutable producer rows join to their exact strict-reader package and
project with their original opaque Exact keys and complete native record views.
Required/optional presence stays independent of Option value type. Structural
counterfactuals return native mismatch/conflict/unknown dispositions with work;
reader/join operating stops retain their original native payloads. Complete
monitor qualification remains Planned; no MemberId is invented for source
records and no native success-work receipt is inferred for the join.


## Qualification Status

All seven production source-record admission tests execute successfully after
adopting merged QSL-1741. The genuine final PlanRoot joins its complete final
producer row set, currently 30 rows for this fixture, and compares every
native row/member field without filtering generated records. Both generated
binding renames (`origin_tick` to `origin`, `amount` to `value`) retain their
original member identities and authentic producer type/value fields.

The ordinary-source projections, original reader/join error payloads,
structural counterfactuals, opaque-authority distinction and error catalog
remain covered by the same target. This is source-record admission evidence,
not complete IR-721 AC1 or generated executable-closure qualification.
Whole-preparation work/storage limits, healthy Timed clock receipt and retained
Program construction remain Planned; independent native limits are not combined into
a fabricated success-work receipt.
