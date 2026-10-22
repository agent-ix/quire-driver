---
id: SR-14834
title: IR-963 scope-boundary review
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-driver; spec/functional/FR-004-select-the-execution-engine.md;
  spec/test-cases/TC-011-execution-engine-selection.md; spec/tests.md; supporting
  src/execute.rs and CLI sources
review_set: subset
---

## Summary

Ticket: IR-963. PR: quire-driver#99. Frozen head: da14fb1faf121b23b5cafb689daadfeee838d843. Base: d07a573e8833e00b9add842c396d13149af68842.

Independent scope-boundary review of the bounded engine invocation seam.

## Verdict

**PASS** — No defect found within this method’s scope.

## Analysis

Driver src/execute.rs owns engine selection, the private invocation boundary and refusal forwarding (core). CLI src/cli.rs owns argument parsing and its explicit empty ObjectEnvironment caller policy (core). QSL owns execution and RunRefusal/OutcomeDocument semantics, consumed unchanged and compared by the existing direct-QSL parity tests (contract verification present in source, not executed in this review). The private test callback is a test producer, not a public engine or upstream API. src/lib.rs:70 exports execute and request/result types without execute_using; ExecutionRequest has no callback field. CLI run at cli.rs:760 calls execute. The scope excludes a second production engine and any public selector. The source confirms this seam is available, so the conditional requirement to wait for a second engine/upstream seam does not apply to the present design.

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
- id: FR-004-AC-10
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: The no-substitution test seam injects only the engine invocation callback
    into the private selection and dispatch function shared with public `execute`;
    the test verifies the exact typed failure and one invocation, while the public
    API, CLI, configuration, environment, and production default expose no callback
    selector.
- id: FR-004
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: "- When a test verifies no substitution after a selected engine returns\
    \ a typed\n  failure, it SHALL inject only the engine-invocation callback into\
    \ the same\n  private selection and dispatch function called by the public `execute`\n\
    \  function. The test SHALL observe the callback invocation count and the exact\n\
    \  typed failure returned by that function; a separate selector, a constructed\n\
    \  `ExecutionFailure`, or a helper that does not traverse this shared dispatch\n\
    \  function SHALL NOT count as evidence for AC-4."
- id: FR-004
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: "- When the public `execute` function runs in production, the driver SHALL\n\
    \  supply QSL `spine::run` as the engine-invocation callback."
- id: FR-004
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: "- When tests inject a callback to verify AC-4, the driver SHALL keep that\n\
    \  injection private and SHALL NOT expose it through the public API, CLI,\n  configuration,\
    \ environment, or a production default."
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
| FND-001 | low | No findings (placeholder) | - |
