---
id: SR-117404
title: IR-721 AC2 original source spec-review-dependency
type: SpecReview
analysis: dependency
scope: agent-ix/quire-driver; PR117; detached IR-721-AC2-source-review; p9 lower_scalar_functions
  contribution
review_set: subset
---

## Summary

IR-721 / driver PR117, p9 contribution titan/ir721-ac2-lowering. ORIGINAL UNRUN source snapshot reviewed only through detached IR-721-AC2-source-review; the later tested snapshot is not substituted for this history. Subjects are lowering.rs, lowering/{key,operation,storage}.rs, monitor_lowering.rs and truthful FR017/TC031 scalar-only status. Root-owned shared APIs were read for compatibility, not edited or separately approved here. No applicable AssuranceProfile exists in this spec tree. No Cargo/build/test/Clippy rerun, global trace/matrix/coverage/properties/advisor command, Kani or repository edit was performed. SOURCE verdict and UNRUN executable status are separate.

## Verdict

PASS source-based owner/dependency assessment; normal composition still required.

Executable mapper acceptance: UNRUN in this original checkpoint.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Read actual owner APIs rather than infer a signature: IR PR906 now reports MERGED through gh pr view and its diff exposes CheckedCallableRead::{callable,work} plus indexed public node_kind. The mapper source uses that actual success-work result and charges returned work instead of row counts or a whole callable-table receipt. The original brief observed the earlier open state; this review records the later verified merge without claiming that this mapper snapshot has been normally rebuilt against it. Exact callable/anonymous-entry/type and full producer authority remain owner supplied.

The native RT Operation/Function/Expression trait definitions were inspected in the consumed owner source, establishing source FND001; no new RT trait API is assumed. Root p8 owns shared budget/error/request/registration/manifest/lock and sole PR117 pushing; p9's contribution is new lowering files/tests/spec artifacts only. No IR or shared source edits occur in this task. Native constructor allocation hook and normal coherent dependency resolution still condition executable/retained-storage qualification. No pins, legacy reader, missing-work receipt or callback supplies an absent API.

## Examined units

```yaml
scope:
- id: FR-017-AC-1
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: context_only
  excerpt: Full lowering consumes the actual opaque admitted selection and produces
    the existing RT Program from the complete four-entry native closure and authentic
    type/association inputs. No replacement request, callback, fake environment or
    constant-refusal preparation stub supplies the handoff.
- id: FR-017-AC-2
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: context_only
  excerpt: After complete success, dropping all source, package, witness, request
    and preparation objects leaves the RT-owned program usable. Two actual runtime
    builds retain independent initialized state without rebinding or a borrowed compiler
    owner.
- id: FR-017-AC-3
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: context_only
  excerpt: Full lowering preserves admission's already consumed work under the same
    total limit, native read success work, charged storage and original cancellation.
    An exact/one-under operating stop returns its original typed cause and no partial
    owner; no guessed reader receipt or semantic-capacity change supplies success.
- id: FR-017-AC-4
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: context_only
  excerpt: Native IR refusal, unsupported source operation, operating stop, cancellation,
    runtime ProgramRefusal and driver invariant failure remain distinct, with original
    source node/path and native payload. No failure becomes a temporal outcome or
    a partial successful program.
- id: FR-017-AC-5
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: The public lower_scalar_functions helper accepts the closed initial scalar
    set and returns owned native RT root addresses/functions/expressions under its
    independent allowance. Requested order and repeated-root FunctionId sharing, exact
    supported operation metadata/arity/types, parameter ownership and unsupported/native-reader
    dispositions match the declared contract. No source parse or evaluator supplies
    meaning; this is component evidence only.
- id: FR-017-AC-6
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: Within one genuine checked package, two functions sharing actual parameter
    handles retain distinct native Argument FunctionId owners and positions. Context-qualified
    memoization retains the identical original source keys without merging the owning
    invocations or rewriting keys. Native scalar outputs remain inspectable after
    setup destruction; separately lowered packages also retain their own qualified
    keys and scoped addresses. This supplies no temporal subformula-association credit.
- id: FR-017-AC-7
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: context_only
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
- id: TC-031-source-status
  path: spec/test-cases/TC-031-monitor-native-lowering.md
  role: examined
  excerpt: 'Status: Planned. The existing original-source preflight proves genuine
    shared

    parameter/body handles only; it does not execute the public helper. Real public

    scalar helper output can back only its executed

    FR-017-AC-5/6 controls and explicitly scoped component operating checks. It

    supplies no complete monitor Program, RT initialization, total admission-plus-

    lowering allowance or temporal subject/subformula evidence. Full AC-1/2/3/4/7

    remain Planned until the required native interfaces and exact controls execute.

    AC-8 is Inspection; no Test trace substitutes for that evidence method.


    The curr'
```
