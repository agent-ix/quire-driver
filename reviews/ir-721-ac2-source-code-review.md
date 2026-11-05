---
id: SR-117401
title: IR-721 AC2 original source code-review
type: SpecReview
analysis: code-review
scope: agent-ix/quire-driver; PR117; detached IR-721-AC2-source-review; p9 lower_scalar_functions
  contribution
review_set: subset
---

## Summary

IR-721 / driver PR117, p9 contribution titan/ir721-ac2-lowering. ORIGINAL UNRUN source snapshot reviewed only through detached IR-721-AC2-source-review; the later tested snapshot is not substituted for this history. Subjects are lowering.rs, lowering/{key,operation,storage}.rs, monitor_lowering.rs and truthful FR017/TC031 scalar-only status. Root-owned shared APIs were read for compatibility, not edited or separately approved here. No applicable AssuranceProfile exists in this spec tree. No Cargo/build/test/Clippy rerun, global trace/matrix/coverage/properties/advisor command, Kani or repository edit was performed. SOURCE verdict and UNRUN executable status are separate.

## Verdict

FAIL source: native RT trait compile blockers; late growth work charge.

Executable mapper acceptance: UNRUN in this original checkpoint.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The mapper/tests assume Clone or Debug on native RT Operation/Function/Expression, which those owner types do not implement: source.clone() yields a borrowed Operation and the authored Debug/unwrap_err diagnostics cannot compile. | src/monitor/lowering/operation.rs:129 |
| FND-002 | medium | Vector-growth relocation work is charged only after try_reserve_exact has already allocated or moved the existing elements; a Work stop can therefore occur after the unadmitted vector-growth action. | src/monitor/lowering/storage.rs:27 |

## Coverage

Rust lane included. The closed eleven-operation mapping uses actual RT IntegerAdd/Subtract/Multiply/Negate/Order and BooleanNot/And/Or alternatives, native OrderingOperator values, exact type/arity and empty metadata checks. Other admitted forms become located original Unsupported; native reads preserve owner errors. Fresh memo per callable correctly scopes Argument FunctionId ownership even when the same checked package shares parameter/body handles. Requested order and repeated-root FunctionId sharing are retained, and output is only native root/function/expression arenas. No evaluator, copied plan/target DTO, identity prefix rewriting, whole callable table, semantic depth cap, fake environment or placeholder complete lower_monitor is introduced.

FND-001 is a static owner-API incompatibility, not an observed compiler run. The actual consumed native RT source defines Operation, Function and Expression without Clone/Debug implementations. At operation.rs:141, source:&Operation calls the reference Clone implementation, producing &Operation where Some expects owned Operation. The test at monitor_lowering.rs:285 formats an Operation with {:?}; the excluded-form test at line384 calls unwrap_err on Result whose successful tuple contains Vec<Function>/Vec<Expression>, requiring Debug on those absent owner traits. Explicitly copy only the closed nonallocating scalar variants/fields, and use explicit Result matching/non-Debug mismatch diagnostics in tests. Do not add foreign owner traits, a copied RT enum or a shim to hide the mismatch.

FND-002: storage::push first charges one insertion unit and reserves bytes, then invokes Vec::try_reserve_exact, and only afterward charges values.len() for relocating existing entries. With a full vector, sufficient storage and less remaining work than the relocation charge, the vector may already have grown/moved before Work is returned. Charge the declared growth/move work before the allocation/mutation (and roll back any logical reservation on a denied action). A focused growth-boundary control can assert Work leaves vector capacity/content unchanged rather than adding an allocator or large-memory lane.

The key conversion is iterative and mechanically retains complete package/path/nested node/ordinal fields through the native constructor. Native constructor internal reservations and diagnostic cloning are explicitly unqualified, so this pass makes no full retained-storage claim. Inline/reference handling resets to original target nodes; root scalar outputs carry empty native node-body paths. Storage/work/cancellation stops return no partial tuple. Future full by-value lower_monitor must use the same private assembly under admission-inherited budget; this contribution only implements the independent helper. All source is UNRUN at this original checkpoint.

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

## Round 1 checkpoint

Round 1 inspects ONLY detached IR-721-AC2-tested-review and the supplied implementation receipts/logs. Seven monitor_lowering tests passed; scoped Clippy, owned formatting and explicit docs passed. No reviewer reruns. Native RT variant copies and non-Debug diagnostics resolve compilation assumptions; exact native parameter vector/result/body type assertions resolve the original evidence gap. Native key construction now uses the owner try_new_with reservation policy, preserving Policy versus Construction errors and original full keys. The new bounded-parameter, native parameter-only refusal, package-change and measured component-storage controls retain truthful component scope. No additional delta finding was identified. Full prepare_monitor/lower_monitor Program, Layout, temporal ownership, input acquisition and diagnostic storage remain Planned, with no global acceptance. Original SR117401/FND002 remains blocking; seven green tests do not exercise denied-work vector growth.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Explicit native field copy and matching executed component assertions inspected. |
| FND-002 | still-open | storage::push still invokes try_reserve_exact before budget.charge(values.len()). A denied relocation-work charge can occur after vector allocation/movement. Charge that work before allocation and add a growth-boundary control proving denied Work leaves capacity/content unchanged. |

## Round 2 checkpoint

Round 2 inspected detached IR-721-AC2-growth-fix-review plus the separately frozen root dependency-edge delta. Scoped scalar foundation is review-clean: all original findings now fixed. Native scalar output remains a component, not complete prepare_monitor/lower_monitor, RT Program/Layout or temporal ownership. Caller input acquisition and diagnostic storage remain unqualified/Planned. Inspected existing receipts: eight public mapper tests and one private growth regression pass; lib/public Clippy, owned formatting and explicit docs pass. The root dependency target passes once, with formatting/docs/diff checks passing. Reviewer ran no Cargo/build/test/Clippy or global scans. Final composed-head scoped gate remains pending.

Growth relocation work is admitted before reserve_storage and try_reserve_exact. The private regression denies relocation work after one insertion unit and verifies the original pointer, capacity, content, reservation and live ledger are unchanged. The heterogeneous real producer signature preserves [Integer, Boolean, Integer], unused parameters, Boolean result and Argument owner/position 1. Earlier native-variant and exact-type fixes remain intact. The native key hook is unchanged from the previous inspected snapshot. No additional source/status delta finding.

## Round 2 dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Source fix and executed focused regression inspected; earlier fix retained. |
| FND-002 | fixed | Source fix and executed focused regression inspected; earlier fix retained. |
