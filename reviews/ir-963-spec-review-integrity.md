---
id: SR-14833
title: IR-963 integrity review
type: SpecReview
analysis: integrity
scope: agent-ix/quire-driver; spec/functional/FR-004-select-the-execution-engine.md;
  spec/test-cases/TC-011-execution-engine-selection.md; spec/tests.md; supporting
  src/execute.rs and CLI sources
review_set: subset
---

## Summary

Ticket: IR-963. PR: quire-driver#99. Frozen head: da14fb1faf121b23b5cafb689daadfeee838d843. Base: d07a573e8833e00b9add842c396d13149af68842.

Independent integrity review of the bounded engine invocation seam.

## Verdict

**PASS** — No defect found within this method’s scope.

## Analysis

FR-004 still implements US-001, whose checked-request story is contextual to engine routing. The changed criteria retain the existing FR-002 command-boundary dependency and QSL ownership. Traceability is US-001 -> FR-004 -> AC-1..AC-10 -> TC-011 -> source trace binders for AC-1..AC-8 / Inspection for AC-9..AC-10. No StR artifact applies in this repository scope. Adding AC-10 and expanding the TC and inventory ranges is internally consistent. AC-4 remains explicitly Partial; neither a matrix tag nor method-without-symbol is presented as new runtime evidence. No graph algorithm, paginated API, external-tool lookup, or competing registry source is added. The missing failure-class case and counter scope are reported in their dedicated methods.

## Validation

Validation: installed quoin 0.28.3 and quire 0.36.2 (engine 0.50.2). Changed-document validation exits 0, 3/3 grammar-clean; full spec validation exits 0, 71/71 grammar-clean, zero grammar findings. quire matrix --strict exits 1 on both exact base and head for the same 35 untagged criteria and one ignored-only criterion outside this change. The sole matrix delta is FR-004-AC-10, method-without-symbol. FR-004 AC-1..AC-8 are tagged; AC-9/10 are method-without-symbol. Trace, property analysis and quoin advise completed. git diff --check passed. Ambient module diagnostics (duplicate archetypes/inverse label and inline data schema) are also present at base. No Cargo, make, Kani, replay or mutation gates were run.

## Examined Scope

```yaml
scope:
- id: FR-004-AC-1
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: An execution request without `--engine` runs the interpreter and returns
    its result.
- id: FR-004-AC-2
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: A value outside the closed engine set is refused before any stage with
    QSL's refusal category and names the supplied value.
- id: FR-004-AC-3
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: A recognized engine absent from the binary is refused before any stage
    with category unsupported; no other engine runs, and process status equals QSL's
    category exit function.
- id: FR-004-AC-4
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: When the selected engine returns unsupported or a typed preparation failure,
    that result is returned and no other engine is invoked.
- id: FR-004-AC-5
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: The public execution request carries the caller-owned typed QSL `ObjectEnvironment`,
    and the selected interpreter passes that same value unchanged to QSL `spine::run`.
- id: FR-004-AC-6
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: Given a function whose result depends on object context, a nonempty caller
    environment produces the QSL result for that context and differs from the result
    with an empty environment; the public library request derives no context or default.
- id: FR-004-AC-7
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: Given a QSL-defined absent object lookup whose QSL result is `Undefined`,
    the driver returns that `Undefined` outcome unchanged and does not replace it
    with a refusal or another outcome.
- id: FR-004-AC-8
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: Given a population argument whose identity has no binding in the caller-supplied
    environment, the driver returns QSL's existing typed refusal unchanged, as required
    by QSL FR-089-AC-4.
- id: FR-004-AC-9
  path: spec/functional/FR-004-select-the-execution-engine.md
  role: examined
  excerpt: The current CLI handler explicitly supplies an empty `ObjectEnvironment`
    as its caller policy because its invocation surface has no object-context input;
    public library callers still must supply the required environment themselves.
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
bindings: []
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
