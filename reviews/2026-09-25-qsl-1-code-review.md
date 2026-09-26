---
id: SR-009
title: "Code and Rust review of QSL-1: consume QSL's diagnostics-catalog and DependencyInput APIs"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@46ea01228ce32ac661ba4be4097992be8ed0c6ef; Cargo.toml; Cargo.lock; README.md; src/handoff.rs; src/refusal.rs; src/registry.rs; src/upstream.rs; tests/drive.rs; tests/dependency_edges.rs; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-005-dependency-edges.md; spec/test-cases/TC-006-generation-through-cg.md; spec/test-cases/TC-008-missing-import-refusal.md"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-004, type: references }
  - { target: ix://agent-ix/quire-driver/TC-006, type: references }
  - { target: ix://agent-ix/quire-driver/TC-008, type: references }
---
# Code and Rust review of QSL-1: consume QSL's diagnostics-catalog and DependencyInput APIs

## Summary

Ticket: QSL-1. PR: quire-driver#6, reviewed at 46ea012.

This PR moves the driver to QSL a553b7c1 and CG a59eee0c. It reads the
diagnostics catalog from `qsl_package::diagnostics_catalog()`. It adapts to
`spine::compile`'s `DependencyInput` and to `Registry::register` returning a
`Vec`. It reads deduped registry conflicts from `Registry::refusals()`. It
renders `code/cause` for import refusals. It replaces the pinned
`RequiresBound`/`AmbiguousBound` expectations with pass-through tests plus
`#[ignore]`d IR-302 intent tests.

The earlier review's nine items are fixed. Two partial residues remain:
stale test comments (FND-002), and an AC-5 wording that the code does not
meet for Check-stage causes (FND-001).

## Verdict

**PASS with findings.** One medium: an AC and code disagreement on which
Compile refusals render `code/cause`. One low: stale test comments. Neither
changes the driver's run output for any unit the tests compile. `make ci`
exits 0.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `compile_cause` ends in a catch-all `_ => None`, so it renders a cause only for `Import`, `DependencyInput` and `Dependency` refusals. FR-001-AC-5 now says a Compile refusal renders `code/cause` whenever the refusing stage names a cause. QSL's `CheckCause::cause()` and `LinkRefusal::cause()` do name causes. Failure scenario: TC-004 step 2's four-deep body under checking depth 3 refuses at Check with `ResourceExhausted(NestingDepth)`, whose cause is `nesting-depth-exceeded`. `DriverRefusal::code()` returns bare `stage_limit_exceeded`, and TC-004 step 2 plus `tc_004_command_level_refusals_are_typed` pin that bare form, contradicting AC-5. Any new cause-bearing `CompileRefusal` variant is also absorbed silently. The `DependencyInput` and `Dependency` arms cannot be reached under the driver's own empty `DependencyInput`, and no test covers them. Fix it one of two ways. Match exhaustively and render the Check and Link causes, then update TC-004 step 2 and its test. Or narrow AC-5 to ADR-015 D-1 import and dependency refusals. | src/refusal.rs:261-266; spec/functional/FR-001-orchestrate-the-driver-steps.md:113; spec/test-cases/TC-004-command-level-refusals.md:41; tests/drive.rs:956 |
| FND-002 | low | Stale test comments, left over from prior item 1. The `tc_001_tc_003_tc_006_steps_run_in_order_through_generation` comment cites two tests that no longer exist: `tc_006_a_literal_beside_an_owned_bound_requires_bound` and `tc_006_two_independently_bounded_operands_is_ambiguous_not_supported`. It also calls `RequiresBound`/`AmbiguousBound` the rules "this bump introduces", with no IR-302 framing. `INT_UNIT`'s doc says CG "settles this `+` `RequiresBound`, not `Supported`" as a plain fact. Failure scenario: a reader looks for the cited tests, finds none, and takes CG's regression as the driver's intended behavior. That is the misreading prior item 1 removed from FR-001 and TC-006. | tests/drive.rs:523-537; tests/drive.rs:37-43 |

## Rust review

- **Idioms.** `serde_json` has moved to `[dev-dependencies]` and `src/` no
  longer uses it. `DriverFault::DiagnosticsCatalog` and its `stage()` arm
  are gone. `evidence()` lost its `emitted` parameter. No `unwrap`/`expect`
  was added on a library path.
- **`let _ = registry.register(descriptor)`** (src/registry.rs:61) discards
  a `Result<(), Vec<RegistrationRefusal>>` on purpose, and a comment says
  why. `Registry::refusals()` at a553b7c is documented as stable and
  deduped by (identity, digest), so reading it once after the loop is
  correct. Vocabulary refusals and duplicate-backend refusals cannot
  collide, and the final `sort()` keeps the documented order.
- **Matching.** The catch-all in `compile_cause` is FND-001. The
  `DriverRefusal::stage()` change stays exhaustive.
- **Tests.** Every new test carries an `FR-001-AC-n (TC-00n)` tag, and
  every cited id resolves in this repo. Both `#[ignore]`s give a tracked
  reason: `IR-302: …`.
- **Oracle strength, measured on a scratch mutation that was reverted:**
  - I made `QslCg::generate` clear `claim_map`. Both
    `tc_006_x_plus_{1,y}_generation_passes_through_unchanged` failed at
    their full `RoutedGeneration` equality, tests/drive.rs:636 and :680.
  - I restored the old per-call accumulation in `build_registry`.
    `tc_004_command_level_refusals_are_typed` failed at tests/drive.rs:990
    on the A/B/A case, with left = [d1, d1, d2] and right = [d1, d2].
- **Ignored intent tests.** I ran them with
  `cargo test --test drive -- --include-ignored`. Both fail at the
  disposition assertion for the reason they document:
  - `tc_006_x_plus_1_settles_supported` panics at tests/drive.rs:709 with
    `RequiresBound { unbounded_type: … }`.
  - `tc_006_x_plus_y_settles_supported_with_independent_argument_bounds`
    panics at :754 with
    `Unsupported { OracleRefused { AmbiguousBound { expected_form: IntegerRange } } }`.
- **Checks that found nothing:** no unsafe, no new `#[allow]`, no workflow
  change, no lint or deny relaxation, and no async surface.

## Gate results

| Gate | Result |
| --- | --- |
| `make ci` (fmt-check, clippy -D warnings, test, deny, audit-unsafe, docs, quire validate spec) at 46ea012 | pass, exit=0: unit 2 passed; dependency_edges 1 passed; drive 12 passed, 2 ignored |
| `cargo test --test drive -- --include-ignored` | 12 passed, 2 failed; both failures are the IR-302 intent tests, failing for the documented reason |
| Coder's log `driver-qsl1-final-ci.log` | first line `head=46ea01228ce32ac661ba4be4097992be8ed0c6ef`, last line `exit=0` |
