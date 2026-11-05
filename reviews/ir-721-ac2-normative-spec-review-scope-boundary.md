---
id: SR-117206
title: IR-721 AC2 normative scope-boundary
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-driver; PR117; p9 branch titan/ir721-ac2-lowering; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md;
  spec/test-cases/TC-031-monitor-native-lowering.md
review_set: subset
---

## Summary

IR-721; driver PR117. Normative-only review of p9 contribution branch titan/ir721-ac2-lowering, read through detached isolated worktree IR-721-AC2-spec-review. Only new FR-017 and TC-031 are subjects. No applicable AssuranceProfile exists in the spec tree. The current agreed AC1/AC2 contract b363b8e4, including its accepted public native-arena scalar helper, is governing input; this clarification arrived after the contribution froze. No production code, source117 fixes/dispositions, Cargo, builds, tests, global scans or advisors were changed/run. This review does not reopen or dispose SR-117101/SR-117102 findings.

## Verdict

PASS responsibility allocation with the blocking public/private conflict recorded in base FND001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Compared the spec to the active agreed split: p8 owns opaque token, request, admission/orchestration, all shared module/error/budget/manifest/lock changes and sole PR117 pushing; p9 owns only new lowering files/tests/spec artifacts, incorporated sequentially. The by-value lower_monitor signature matches the active contract and contains no callback, caller ProgramParts or replacement environment. Original native IR/type/witness/association authority stays with the owner; the lowerer emits only RT-native owners and does not run initializer/build. Public lower_scalar_functions is accepted by the current contract as native tuple-only component API, not a second executable family or full Program. The frozen documents still prohibit that public boundary: base FND001 is the single recorded corrective finding for it, while this lens records the agreed allocation and introduces no duplicate finding. Repair only to the accepted current signature and ownership; do not broaden shared file custody or resurrect superseded contracts. Full monitor/Program authority remains unavailable until its actual owner interfaces exist.

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
