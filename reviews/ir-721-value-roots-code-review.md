---
id: SR-119001
title: "code-review of ordinary scalar Value roots"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver; Cargo.lock; spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md; spec/test-cases/TC-031-monitor-native-lowering.md; tests/monitor_lowering.rs; src/monitor/lowering.rs (context)"
review_set: subset
---

## Summary

Ticket: IR-721. Reviewed frozen PR #119, independently matched local and published head before review. Review is limited to this PR diff and necessary existing lowering context. No build, test, Clippy, cargo-deny, Kani or global trace/matrix/coverage/property/advisor rerun was performed, per dispatch. Author receipts and raw test-final.log show 8 passed, 0 failed in 0.43s; clippy-final.log and fmt/docs logs report success. These are author evidence, not independent execution. No applicable AssuranceProfile exists under spec/.

## Verdict

**PASS** — no defects found in the examined change.

## Review Evidence

Real public QSL parse/select/check/emit and strict IR admission supply both packages and original selectors. The existing helper authenticates same-package function identity, uses ordered callable membership for Argument ownership, and allocates a fresh per-function memo. The literal test first requires a native Literal body, then requires Unsupported with the original node, structural key, kind, empty path, absent occurrence and absent operation. The foreign selector is positively read from its own package before native UnknownNode is compared in full with the forwarded driver payload. Parameter-only output is inspected for its original key, Integer type and Argument owner/position after package, identities, roots and native views are dropped. Only the two normal branch-main IR lock entries change; no manifest pins, reader copies or production changes occur.

## Examined Scope

```yaml
scope:
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
- id: FR-017-value-roots
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: 'Ordinary source callable reads SHALL retain authenticated

    Value(Parameter) and Value(Literal) body roots as supplied by the native reader.'
- id: FR-017-parameter
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: 'A direct Value(Parameter) body SHALL materialize its declared argument
    position,

    original body key and native scalar type.'
- id: FR-017-literal
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: 'A Value(Literal) body is readable

    input but remains outside this helper''s closed capability: it SHALL reach the

    driver''s located Unsupported result with no operation record.'
- id: FR-017-boundary
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: examined
  excerpt: 'This ordinary

    source capability SHALL NOT widen the separate monitor-entry Expression-only

    read or qualify a complete monitor entry.'
- id: TC-031-value-roots
  path: spec/test-cases/TC-031-monitor-native-lowering.md
  role: examined
  excerpt: "   Read direct ordinary Value(Parameter) and Value(Literal) bodies through\
    \ the\n   actual native callable API."
- id: FR-017
  path: spec/functional/FR-017-lower-and-retain-admitted-monitor-programs.md
  role: context_only
  excerpt: 'The monitor lowerer SHALL materialize the selected admitted executable
    closure

    into the existing runtime representation without replacing source identities,

    introducing another executable vocabulary or inferring missing authority.'
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
