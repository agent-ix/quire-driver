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
