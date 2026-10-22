---
id: SR-14835
title: IR-963 failure-domain review
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-driver; spec/functional/FR-004-select-the-execution-engine.md;
  spec/test-cases/TC-011-execution-engine-selection.md; spec/tests.md; supporting
  src/execute.rs and CLI sources
review_set: subset
---

## Summary

Ticket: IR-963. PR: quire-driver#99. Frozen head: da14fb1faf121b23b5cafb689daadfeee838d843. Base: d07a573e8833e00b9add842c396d13149af68842.

Independent failure-domain review of the bounded engine invocation seam.

## Verdict

**CONDITIONAL** — The findings below require explicit disposition.

## Analysis

The failure policy is strict passthrough: the selected callback failure and its QSL document survive the production dispatcher, and public injection is prohibited. Engine identity comes from ExecutionEngine; no new registry keys or graph traversal are introduced. The callback may count calls as a test observation. Existing unavailable_engines_refuse_without_substitution takes the guard before run at execute.rs:164; selected_interpreter_preparation_refusal_matches_direct_qsl_run and pre_cancelled_caller_handle_preserves_qsl_refusal_and_document do not exercise a selected callback returning Unsupported. The proposed source-byte preparation refusal is Category::Incomplete, not selected-engine Unsupported. This is a missing planned case, not a demand to implement it in this Markdown-only PR.

## Validation

Validation: installed quoin 0.28.3 and quire 0.36.2 (engine 0.50.2). Changed-document validation exits 0, 3/3 grammar-clean; full spec validation exits 0, 71/71 grammar-clean, zero grammar findings. quire matrix --strict exits 1 on both exact base and head for the same 35 untagged criteria and one ignored-only criterion outside this change. The sole matrix delta is FR-004-AC-10, method-without-symbol. FR-004 AC-1..AC-8 are tagged; AC-9/10 are method-without-symbol. Trace, property analysis and quoin advise completed. git diff --check passed. Ambient module diagnostics (duplicate archetypes/inverse label and inline data schema) are also present at base. No Cargo, make, Kani, replay or mutation gates were run.

## Examined Scope

```yaml
scope:
- id: FR-004-AC-4
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: When the selected engine returns unsupported or a typed preparation failure,
    that result is returned and no other engine is invoked.
- id: TC-011
  path: spec/test-cases/TC-011-execution-engine-selection.md
  role: examined
  excerpt: "5. Invoke the private selection and dispatch function that public `execute`\n\
    \   calls, injecting only its engine-invocation callback. Have that callback\n\
    \   return a designated typed preparation refusal and record its invocation\n\
    \   count. Assert that the exact refusal and its QSL document are returned and\n\
    \   the callback ran exactly once. The test SHALL fail if it constructs the\n\
    \   expected `ExecutionFailure` itself, uses a separate selector, or does not\n\
    \   traverse this shared production dispatch function."
- id: TC-011
  path: spec/test-cases/TC-011-execution-engine-selection.md
  role: examined
  excerpt: "6. Inspect `src/execute.rs` and `quire-cli/src/cli.rs`: confirm public\n\
    \   `execute` calls the same private dispatch function with QSL\n   `spine::run`,\
    \ and neither the public API nor CLI, configuration, or\n   environment can select\
    \ the test callback."
- id: src/execute.rs::execute_using
  path: src/execute.rs
  role: context_only
  excerpt: "fn execute_using(\n    request: ExecutionRequest<'_>,\n    run: impl FnOnce(\n\
    \        &ExecutionRequest<'_>,\n        &ObjectEnvironment,\n    ) -> Result<(PackageId,\
    \ CallOutcome), Box<RunRefusal>>,\n) -> Result<ExecutionResult, ExecutionFailure>\
    \ {\n    if request.engine != ExecutionEngine::Interpreter {\n        return Err(ExecutionFailure::UnsupportedEngine(request.engine));\n\
    \    }\n    let outcome = run(&request, request.object_environment);\n    let\
    \ document = OutcomeDocument::from_run(&outcome);\n    match outcome {\n     \
    \   Ok((package_id, outcome)) => Ok(ExecutionResult {\n            package_id,\n\
    \   "
bindings: []
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-011 step 5 injects only a typed preparation refusal, leaving the selected-engine Unsupported branch of FR-004-AC-4 without a planned no-fallback exercise. Step 3 covers an engine absent before invocation, a different path. Specify a selected-callback Unsupported result as well, or explicitly retain that portion as Partial pending a supported carrier. | spec/test-cases/TC-011-execution-engine-selection.md:30-36 |

## Disposition pass 1

Reviewed exact head 7dc323493a1401cf724c2ace246fe9925a2affec against base d07a573e8833e00b9add842c396d13149af68842. **CONDITIONAL**.

Exact head and clean working tree confirmed. All eight committed SR artifacts matched the original scratch bytes before this disposition pass. Full spec validation: exit 0, 71/71 grammar-clean, zero grammar findings. Matrix strict: exit 1 with the same 35 untagged and one ignored-only criterion; only the AC-10 statement differs from the previous reviewed matrix. Properties, trace and quoin advise exit 0; diff-check exits 0. No Cargo/make/Kani/tests.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: TC-011 step 7 now explicitly plans selected-engine Unsupported and preparation failure, while full AC-4 remains Partial. The new incorrect carrier-availability explanation is a separate FND-002 below. |
| FND-002 | fixed | Round 2: TC-011 now specifies both RunRefusal::UnsupportedResult and typed preparation refusal through the existing shared dispatcher, preserves exact result/document assertions, and distinguishes selected-engine Unsupported from pre-invocation UnsupportedEngine. FR-004 and both inventory rows agree; Partial remains for direct/alternate invocation observability. |

### Fixed text

```text
7. Full AC-4 verification remains blocked until a second production engine
   or upstream-supported shared dispatch contract makes every reachable
   engine invocation observable through the same test boundary. At that
   boundary, inject a typed `Unsupported` result and a typed preparation
   failure from the selected engine; assert exact result preservation, one
   selected-engine invocation, and zero invocations of every alternative
```


## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | TC-011 step 6 incorrectly says the existing callback cannot produce selected-engine Unsupported. The locked QSL RunRefusal::UnsupportedResult maps through Code::UnsupportedConstruct to Category::Unsupported and fits the callback's Box<RunRefusal> error type. Distinguish this supported selected-engine result from pre-invocation UnsupportedEngine; correct the repeated carrier-unavailability claims while retaining Partial for the all-invocation observation gap. | spec/test-cases/TC-011-execution-engine-selection.md:36 |

Source inspection used the QSL revision resolved by Cargo.lock (485ccaa3bd289ae94baa74e96a976adce99fd0d8): qsl-replay/src/spine/call.rs:252 declares public RunRefusal::UnsupportedResult { function: String }; line 390 maps it to Code::UnsupportedConstruct; lines 407-408 read that code's category. qsl-foundation/src/diagnostic.rs:331 lists UnsupportedConstruct as unsupported and lines 345-349 return Category::Unsupported. Driver src/execute.rs:162 accepts Box<RunRefusal>; lines 175-177 preserve it in ExecutionFailure::Run; line 118 delegates its category. Thus Err(Box::new(RunRefusal::UnsupportedResult { function: "designated".to_owned() })) is a supported callback return carrier. The unavailable-engine guard is a different variant and does not imply absence of this carrier. No Rust execution or compilation was used for this evidence.

FND-002 has no disposition yet and requires the next fix round.

## Disposition pass 2

Reviewed exact head 1ffbbddd363147086faa70858047a00b88b8498b against base d07a573e8833e00b9add842c396d13149af68842. **PASS**: FND-002 is fixed, all earlier findings remain fixed, and no new findings were found.

TC-011 step 5 specifies the two callback-injected failure cases, exact refusal/document equality and one callback call. Step 6 correctly identifies the supported QSL carrier and distinguishes the unavailable-engine guard. Step 7 and Expected Results preserve the all-invocation oracle prerequisite and Partial status. FR-004 and tests.md contain no remaining carrier-unavailability claim. AC-10 still uses Inspection alone, callback exposure remains forbidden and helper-only bypasses remain invalid evidence. The three earlier FND-001 outcomes remain fixed; no regressions or new findings were found.

Full spec validation passed, 71/71 grammar-clean, zero grammar findings. Matrix strict exits 1 for the same 35 untagged and one ignored-only criterion; the matrix is byte-for-byte data-equivalent to disposition round 1. Trace, properties and quoin advise passed; git diff --check passed. No Cargo/make/Kani/tests were run.

### Fixed text (round 2)

```text
6. Inspect `src/execute.rs`: confirm the callback type returns only QSL run
   success or `RunRefusal`, including `RunRefusal::UnsupportedResult` whose
   category is `Unsupported`. Distinguish this selected-engine result from
   `ExecutionFailure::UnsupportedEngine`, which is produced before callback
   invocation when an engine is not linked into the binary.
```
