---
id: SR-117201
title: IR-721 AC2 normative base
type: SpecReview
analysis: base
scope: agent-ix/quire-driver; PR117; p9 branch titan/ir721-ac2-lowering; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md;
  spec/test-cases/TC-031-monitor-native-lowering.md
review_set: subset
---

## Summary

IR-721; driver PR117. Normative-only review of p9 contribution branch titan/ir721-ac2-lowering, read through detached isolated worktree IR-721-AC2-spec-review. Only new FR-017 and TC-031 are subjects. No applicable AssuranceProfile exists in the spec tree. The current agreed AC1/AC2 contract b363b8e4, including its accepted public native-arena scalar helper, is governing input; this clarification arrived after the contribution froze. No production code, source117 fixes/dispositions, Cargo, builds, tests, global scans or advisors were changed/run. This review does not reopen or dispose SR-117101/SR-117102 findings.

## Verdict

FAIL: public helper boundary contradicts the active contract; closed scalar capability is missing.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-017 and TC-031 prohibit a public scalar component API, contradicting the active agreed b363b8e4 contract that explicitly accepts public lower_scalar_functions with RT-native tuple output. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:60 |
| FND-002 | medium | The scalar component requires only selected arithmetic/Boolean/call operations without a closed supported form/operation/type set or explicit unsupported remainder, leaving its public capability contract undefined before implementation. | spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md:86 |

## Coverage

Applied the base checklist to the new lowering requirement and companion procedure. FR/AC/TC IDs and local FR015/FR016/TC relationship paths resolve, and native RT FunctionId/Function/Expression tuple outputs match the existing runtime owner types. The full lower_monitor entry consumes the existing opaque selection by value and returns the original owned Rc<Program>, not a second DTO. All criteria remain Planned; component output is not full owner evidence. No new pins, digests, source ledger, semantic identity mint or copied fixture instruction was found.

FND-001: the active agreed contract b363b8e4 now explicitly permits public quire_driver::monitor::lowering::lower_scalar_functions(package, roots, identities, limits, cancel) -> Result<(Vec<FunctionId>, Vec<Function>, Vec<Expression>), MonitorPreparationError>. FR017 Outputs instead says staging SHALL NOT be exposed as a public scalar API, AC5 calls the component private, and TC031 Description forbids an extra public scalar API and directs tests at a private phase. Reconcile those statements and qualification procedure with the exact accepted helper signature/path, requested-root order and repeated-root sharing semantics. Keep the private shared worklist/private assembly distinction: the public independent-allowance helper is allowed, while full lower_monitor must use admission-resumed budget and returns a complete Program only with real authority. This is contract drift after the spec freeze, not permission to revive any superseded API or custody agreement.

FND-002: no closed list says which native Integer/Boolean forms this helper supports. The phrases selected arithmetic/Boolean/call operations permit incompatible implementations (for example Add-only versus Add/Subtract/Multiply), while the active contract explicitly requires supported scalar forms specified before their implementation. Add a closed capability list over actual admitted typed operation identity/mode/arity and type restrictions, and name typed UnsupportedOperation/native-reader refusal behavior outside it. Integer divide/modulo and Boolean implies have no corresponding current RT Operation; do not synthesize an evaluator or infer support from a broad ApplicationOperator family. TC031 should then choose genuine examples from that declared list and its excluded remainder.

No additional public API, speculative owner getter or producer witness schema should be invented to repair these two issues.

## Examined units

```yaml
scope:
- id: FR-017-AC-1
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Full lowering consumes the actual opaque admitted selection and produces
    the existing RT Program from the complete four-entry native closure and authentic
    type/association inputs. No replacement request, callback, fake environment or
    constant-refusal preparation stub supplies the handoff.
- id: FR-017-AC-2
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: After complete success, dropping all source, package, witness, request
    and preparation objects leaves the RT-owned program usable. Two actual runtime
    builds retain independent initialized state without rebinding or a borrowed compiler
    owner.
- id: FR-017-AC-3
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Full lowering preserves admission's already consumed work under the same
    total limit, native read success work, charged storage and original cancellation.
    An exact/one-under operating stop returns its original typed cause and no partial
    owner; no guessed reader receipt or semantic-capacity change supplies success.
- id: FR-017-AC-4
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Native IR refusal, unsupported source operation, operating stop, cancellation,
    runtime ProgramRefusal and driver invariant failure remain distinct, with original
    source node/path and native payload. No failure becomes a temporal outcome or
    a partial successful program.
- id: FR-017-AC-5
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: A private component consumes genuine QSL-produced/strictly admitted ordinary
    Integer/Boolean callable facts and materializes native RT root addresses/functions/expressions.
    Its selected arithmetic/Boolean/call operations, argument positions, native projected
    types and complete source keys match the actual input; no source parse or evaluator
    supplies lowering meaning. This is component evidence only.
- id: FR-017-AC-6
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Within one genuine checked package, two functions sharing actual parameter/query
    handles retain distinct native Argument FunctionId owners and positions. Context-qualified
    memoization retains the identical original source keys without merging the owning
    invocations or rewriting keys. Native scalar outputs remain inspectable after
    setup destruction; separately lowered packages also retain their own qualified
    keys and scoped addresses. This supplies no temporal subformula-association credit.
- id: FR-017-AC-7
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: The complete monitor program retains original clause/subject/occurrence/root/buffer
    ownership. Equal numeric subformula indices under different genuine subjects do
    not alias in selection or retained Layout; no scalar-only sample substitutes for
    this control.
- id: FR-017-AC-8
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Lowering consumes native typed owner facts and emits only RT-native executable
    types, with no copied evaluator, transition implementation, second IR reader/plan
    vocabulary, hidden identity mint, whole-package index, repeated unrelated callable
    scan or semantic depth cap.
- id: FR-017-input-handoff
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: 'The lowerer SHALL consume the existing opaque selection by value. It SHALL

    obtain the original package, four-entry plan view, independent premises,

    canonical witness borrows and operating limits through the selection''s

    read-only interface.'
- id: FR-017-private-output
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: 'Private component work may materialize existing RT `FunctionId` root addresses,

    `Vec<Function>` and `Vec<Expression>` directly. Such staging consists only of

    native RT owners, not another plan/type/expression DTO. It SHALL NOT be exposed

    as a separate public scalar preparation API or counted as a complete Program.'
- id: TC-031-component-procedure
  path: spec/test-cases/TC-031-monitor-native-lowering.md
  role: examined
  excerpt: 'Verify [FR-017](../functional/FR-017-lower-and-retain-admitted-monitor-programs.md)

    through native owner facts and RT-native output. Keep private scalar worklist

    checks separate from complete monitor selection, construction and accounting.

    No extra public scalar-preparation API or fake full lowerer is introduced by

    the test harness.'
- id: TC-031-status
  path: spec/test-cases/TC-031-monitor-native-lowering.md
  role: examined
  excerpt: 'Status: Planned. Real scalar component output can back only its executed

    FR-017-AC-5/6 controls and explicitly scoped component operating checks. It

    supplies no complete monitor Program, RT initialization, total admission-plus-

    lowering allowance or temporal subject/subformula evidence. Full AC-1/2/3/4/7

    remain Planned until the required native interfaces and exact controls execute.

    AC-8 is Inspection; no Test trace substitutes for that evidence method.

    '
```
