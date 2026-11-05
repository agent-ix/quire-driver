---
id: TC-031
title: "Retain native monitor lowering ownership and operating boundaries"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-017
    type: verifies
---
# TC-031: Retain native monitor lowering ownership and operating boundaries

## Description

Verify [FR-017](../functional/FR-017-lower-and-retain-admitted-monitor-programs.md)
through native owner facts and RT-native output. Exercise the agreed public
`quire_driver::monitor::lowering::lower_scalar_functions` helper separately
from complete monitor selection, construction and accounting. Its implementation
shares the private worklist with the full by-value lowerer; no fake full lowerer
or second plan DTO is introduced by the test harness.

## Test Procedure

1. Author ordinary pure Integer/Boolean functions covering parameter-only
   bodies and every identity in FR-017's closed initial scalar table: add, sub,
   mul, negate, lt/le/gt/ge, not, and, or. Use parameter references and genuine
   nested supported expression references, with exact plain metadata and native
   operator/arity/types. Compile/check/emit through public QSL
   and strictly admit the exact bytes through IR. This component setup invokes
   no monitor-plan producer and does not depend on its separate qualification.
   Obtain actual native callable headers, primitive type projections, node
   metadata and structural keys from that same admission. Do not hand-build
   a callable, nominal environment, semantic identity or checked package.
2. Invoke the public helper with that package, requested root selectors,
   original SelectedExactIdentities, independent limits and Cancel. Inspect its
   existing RT root FunctionId addresses, Function and Expression arenas.
   Request roots in a nondeclaration order and repeat one root; require output
   address order to match requests and identical FunctionId for the repetition,
   without duplicate function declarations. Check empty-root output too. Compare exact source keys, native projected parameter/result types,
   actual operation alternatives, ordered operands and
   owning-function argument positions. An arithmetic source change must change
   the native operation; repeated references retain correct sharing/context.
   Include two actual functions with identical formal names/types/levels and
   actual shared parameter handles in the same checked package. Assert
   that real sharing before materialization. Require native Argument values
   with distinct actual owning FunctionIds and unchanged parameter positions;
   owner-dependent expressions remain distinct in their function contexts
   while their original source keys are retained exactly, including equal
   keys where the source really shares them. No identity prefix rewrite or
   forged shared node is used to make the test pass.
   No local evaluator computes the expected behavior.
3. Drop all source buffers, checked objects, packages, native setup views and
   temporary assembly state. Inspect only the owned native arenas afterward.
   Lower another genuine package with the same source spelling/numeric arena
   addresses and require its different package-qualified keys and independent
   argument ownership. This proves function-arena scope, not temporal
   subject/subformula association.
4. Bound the public helper's selective native reads and assembly with its
   independent actual work/storage/cancellation allowance. Preserve typed stops
   and publish no partial tuple. Observe native successful-read receipts; do not
   guess them or use a whole callable table for an unrelated global receipt.
   Setup-only source preflight is not public-helper or full preparation work.
   Exercise genuinely admitted excluded forms: literal, If, Let, ordinary
   source Call (including a recursive call if the producer admits one), integer
   modulo and Boolean implies. Require located Unsupported rather than native
   executable substitutes or recursive expansion. Include available native
   read failures and preserve their exact payloads. For supported operations,
   compare every metadata field, arity and projected type against FR-017's
   closed table; an admitted variant outside it must be Unsupported. Retain
   shared acyclic expressions and owner-independent deduplication without
   inventing a cyclic healthy fixture or relaxing strict IR admission.
5. Once the actual complete authority-bearing native interfaces exist, obtain
   a genuine final monitor emission, admit it, and consume p8's original opaque
   AdmittedMonitorSelection by value through the concrete full lowerer. Use
   authentic anonymous entry, record/member, union, type and clock/order/
   progress channels from the owner. Require RT Program::try_new success with
   the unchanged original Layout and complete retained arenas. A missing
   native fact is a dependency/refusal, never an empty environment or a skip.
6. Drop preparation inputs and build the actual retained program twice with
   explicit runtime budgets. Assert independent prescribed initial state and
   unchanged source/association ownership. Exercise genuine different subjects
   with equal numeric subformula indices and require no aliasing. These full
   controls cannot be satisfied by scalar arena inspection.
7. Exercise full lowering after nonzero admission work under the same total
   allowance. Check exact and one-under work/storage boundaries, original
   cancellation, native IR projection refusal, a real unsupported operation
   and native RT ProgramRefusal. Preserve each original typed cause/locus and
   no partial output. Observe actual owner-reported successful read work;
   setup-only component costs or a guessed count supply no total-budget proof.
8. Inspect the production paths for another parser/evaluator, copied transition,
   duplicate reader/vocabulary, hidden source-identity mint, whole-package
   graph/index scan or semantic depth cap. Keep this inspection separate from
   executable test trace credit.

## Expected Results

Status: Planned. The existing original-source preflight proves genuine shared
parameter/body handles only; it does not execute the public helper. Real public
scalar helper output can back only its executed
FR-017-AC-5/6 controls and explicitly scoped component operating checks. It
supplies no complete monitor Program, RT initialization, total admission-plus-
lowering allowance or temporal subject/subformula evidence. Full AC-1/2/3/4/7
remain Planned until the required native interfaces and exact controls execute.
AC-8 is Inspection; no Test trace substitutes for that evidence method.

Native key-constructor internal reservations are now observed through the
owner's callback, alongside measurable driver vector/text/worklist storage.
The same genuine public-helper fixture exercises measured storage success and
one-under typed denial. This qualifies those component reservation cells only;
caller-owned input acquisition and native diagnostic clone allocation remain
outside complete retained-storage qualification. Exact whole-preparation
storage remains Planned.

Observed partial execution covers the eleven plain operation mappings, exact
native parameter/result types, requested order and repeated roots, shared
parameter/body handles with distinct FunctionId owners, nested expression
sharing, retained keys after setup drop, package scoping, bounded-type refusal,
and original unsupported modulo/call/If/Let/implies loci. Native parameter-only
and literal-only callable reads currently refuse with MalformedBody; tests
preserve the exact original native payload rather than substituting a driver
Unsupported or claiming parameter-only success. That positive remains Planned.
