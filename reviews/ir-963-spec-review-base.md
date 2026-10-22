---
id: SR-14831
title: IR-963 base review
type: SpecReview
analysis: base
scope: agent-ix/quire-driver; spec/functional/FR-004-select-the-execution-engine.md;
  spec/test-cases/TC-011-execution-engine-selection.md; spec/tests.md; supporting
  src/execute.rs and CLI sources
review_set: subset
---

## Summary

Ticket: IR-963. PR: quire-driver#99. Frozen head: da14fb1faf121b23b5cafb689daadfeee838d843. Base: d07a573e8833e00b9add842c396d13149af68842.

Independent base review of the bounded engine invocation seam.

## Verdict

**CONDITIONAL** — The findings below require explicit disposition.

## Analysis

The source-backed seam decision is feasible, and all ten criterion ids are unique. TC-011 verifies FR-004; the new criterion has a corresponding procedure and expected-result inspection. The edited status rows preserve Partial and distinguish a specified test from landed evidence. No new hash, digest, version pin, copied upstream vocabulary, or file-scanning gate is introduced in the changed units. Existing unrelated tests.md rows were context only. Behavioral coverage gaps are reported by the failure-domain and evidence methods.

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
bindings: []
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new AC-10 Verification cell contains Inspection (TC-011), but the review contract requires a verification method only. Use Inspection and retain the TC relationship in the existing verifies edge. | spec/functional/FR-004-select-the-execution-engine.md:109 |

## Disposition pass 1

Reviewed exact head 7dc323493a1401cf724c2ace246fe9925a2affec against base d07a573e8833e00b9add842c396d13149af68842. **PASS**.

Exact head and clean working tree confirmed. All eight committed SR artifacts matched the original scratch bytes before this disposition pass. Full spec validation: exit 0, 71/71 grammar-clean, zero grammar findings. Matrix strict: exit 1 with the same 35 untagged and one ignored-only criterion; only the AC-10 statement differs from the previous reviewed matrix. Properties, trace and quoin advise exit 0; diff-check exits 0. No Cargo/make/Kani/tests.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: AC-10 now declares Inspection alone; TC-011 still has its verifies edge and the FR retains its TC dependency link. |

### Fixed text

```text
| FR-004-AC-10 | The engine-invocation callback is private to the driver implementation; public `execute` supplies QSL `spine::run`, and the public API, CLI, configuration, environment, and production default expose no callback selector. | Inspection |
```
