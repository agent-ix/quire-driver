---
id: SR-14836
title: IR-963 evidence review
type: SpecReview
analysis: evidence
scope: agent-ix/quire-driver; spec/functional/FR-004-select-the-execution-engine.md;
  spec/test-cases/TC-011-execution-engine-selection.md; spec/tests.md; supporting
  src/execute.rs and CLI sources
review_set: subset
---

## Summary

Ticket: IR-963. PR: quire-driver#99. Frozen head: da14fb1faf121b23b5cafb689daadfeee838d843. Base: d07a573e8833e00b9add842c396d13149af68842.

Independent evidence review of the bounded engine invocation seam.

## Verdict

**CONDITIONAL** — The findings below require explicit disposition.

## Analysis

quoin advise --json recommends Test for AC-4 with example-based unit/E2E evidence. It flags AC-10 Inspection against architecture-conformance and test suggestions. Reviewer judgment retains Inspection for AC-10 because it audits callback isolation and the existence/content of the AC-4 test; the runtime property remains Test under AC-4. AC-9 is unchanged contextual Inspection. These advisor mismatches are advisory, not automatic defects. The existing callback is FnOnce and selected once in the current source; a count of one measures that callback, not arbitrary invocations introduced beside it. TC-011 step 6 checks shared delegation and exposure but does not require all invocation paths to traverse the counted boundary. Exact failure equality catches replacement of the result but cannot detect a discarded fallback result. No runtime or mutation gate was run; the counterexample is a source-level analysis.

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
- id: spec/tests.md:FR-004
  path: spec/tests.md
  role: examined
  excerpt: AC-4 remains partial until a test asserts exact typed-failure preservation
    and one invocation through that shared path; a separate helper or selector is
    insufficient.
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
| FND-001 | medium | Counting only the injected callback does not detect an engine invocation that bypasses that callback. A dispatcher can call it once, invoke QSL directly afterward, and still return the original refusal/document, passing the stated assertions. Require inspection that every reachable engine invocation uses the counted boundary, and an adversarial oracle that rejects any extra or alternate invocation before claiming AC-4 complete. | spec/test-cases/TC-011-execution-engine-selection.md:30-40 |
