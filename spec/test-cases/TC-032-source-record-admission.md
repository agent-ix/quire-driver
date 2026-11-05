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

The production ordinary-source, native error/control and error-catalog checks
execute successfully. The genuine final PlanRoot positive is unqualified: QSL's
Activation row retains admitted_name `origin_tick` after final emission renames
the actual binding to `origin`. Native IR correctly returns its original
MismatchedIdentityWitness at Activation with work 197. The test preserves this
failing positive until the producer publishes the correct final binding name;
it does not filter generated records or mutate/reseal the emitted package.
