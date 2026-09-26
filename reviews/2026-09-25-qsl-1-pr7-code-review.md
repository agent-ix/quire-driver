---
id: SR-011
title: "Code and Rust review of QSL-1 PR #7: bump CG to edade59 (IR-302), un-ignore the intent tests"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@9c1a9bdee9aaef95c6e95fdd829cd55f94dd31c1; Cargo.toml; Cargo.lock; README.md; spec/functional/FR-001-orchestrate-the-driver-steps.md; spec/test-cases/TC-006-generation-through-cg.md; tests/drive.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-driver/TC-006, type: references }
---
# Code and Rust review of QSL-1 PR #7: bump CG to edade59 (IR-302), un-ignore the intent tests

## Summary

Ticket: QSL-1. PR: quire-driver#7, reviewed at 9c1a9bd.

The PR moves CG from a59eee0 to edade59 (IR-302). It un-ignores the `x + 1`
and `x + y` intent tests, which now assert `Supported` with argument ranges
`[(0,9),(1,1)]` and `[(0,9),(10,20)]`. It adds
`tc_006_x_plus_100_result_bound_is_unreachable_before_cg_ever_sees_it`, and it
rewrites FR-001's Behavior prose, TC-006 and the README to match.

The dependency bump and the un-ignored tests are correct. The main problem is
the new claim that CG's `ResultBoundUnreachable` cannot be reached from any
QSL-compiled unit. That claim is false. QSL's range check depends on the path:
it never discharges a coercion inside an `if` branch it can prove is dead. CG
ignores path conditions. A measured QSL unit compiles and reaches
`ResultBoundUnreachable` (FND-001).

Method: I read the diff and CG edade59's `exact_scalar.rs` and
`kani_obligations.rs`, plus QSL a553b7c's `qsl-semantics/src/check/facts.rs`.
I then ran a probe crate outside the repo (scratchpad, path dependency on this
worktree, same pins). It drove ten QSL units through `quire_driver::drive`
with each integer-operation node routed to Kani.

## Verdict

**CHANGES REQUESTED (medium).** The code is fine and the gates are green. The
spec and test prose state a false invariant (FND-001) and leave out two real
limits (FND-002).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The claim that `ResultBoundUnreachable` is "unreached from any unit this driver's own tests compile" and that "QSL's proof and CG's reachability computation ask the same question over the same integers" is false. QSL's `Facts::walk` checks only the branches it can prove are reachable: `outcomes` returns `None` for a branch it proves dead, and that branch's `Coerce` is never discharged. CG's `reachable_results` ranges each operand over its declared domain and ignores path conditions. Measured failure: `function f using v(x: Int[0, 9]): Int[0, 10] pure { if x > 20 then x + 100 else 0 }` compiles, and so does the same unit with `if false`. Routing the `+` to Kani settles `Unsupported { ResultBoundUnreachable { operation_identity: "quire.op.integer.add", lower: 0, upper: 10, reachable_lower: "100", reachable_upper: "109" } }`. A downstream consumer, such as QSL-22, that trusts FR-001 and treats this refusal as dead code would mishandle a real outcome. The test name `..._is_unreachable_before_cg_ever_sees_it` overclaims in the same way: it shows only that one shape is refused. | spec/functional/FR-001-orchestrate-the-driver-steps.md:176-184; spec/test-cases/TC-006-generation-through-cg.md:89-101; tests/drive.rs:781-814 |
| FND-002 | medium | FR-001's "Two limits remain" and the README undercount the remaining limits. (a) Path conditions: CG ignores the guards around an `if`. A dead branch settles `ResultBoundUnreachable` (FND-001). A live guarded branch still ranges the operand over its full declared domain: `if x > 5 then x + 100 else 0` into `Int[0, 200]` settles `Supported` with args `[(0,9),(100,100)]`, not `x` restricted to `[6,9]`. That gap is IR-304's scope, but neither doc says so. (b) The `RequiresBound` limit is not confined to "a plain-`Integer` parameter". Any operand that is neither bounded nor a literal also hits it, including a nested subexpression and a negative literal (QSL emits `-1` as a negate node). Measured: the outer `+` of `(x + 1) + 1` over `Int[0, 9]` into `Int[0, 20]` settles `RequiresBound`. So do both the `*` and the negate of `x * -1` over `Int[1, 9]` into `Int[-9, -1]`, and the negate of `-(x + 1)`. Failure scenario: a reader of FR-001 or the README expects `x * -1` or any nested arithmetic over bounded parameters to settle `Supported`, and it does not. | spec/functional/FR-001-orchestrate-the-driver-steps.md:186-193; README.md:23-29 |
| FND-003 | low | The README says "a node with more than one narrowing caller still settles `Unsupported`/`AmbiguousBound`". The code in CG edade59 (`exact_scalar::narrowing`) deduplicates the narrowing conversions by bound node. `AmbiguousBound` needs two or more distinct bounds, or a narrowing consumer alongside a non-narrowing one. FR-001:191-193 states this correctly. Failure scenario: two narrowing conversions of one node to the same bounded type settle `Supported`, contrary to the README. This was read from the code, not measured. | README.md:28-29 |
| FND-004 | low | The intent tests pin `harness.identity.node_id` and each argument's `(minimum, maximum)`, but not `harness.identity.operation_identity` or the result bound. The result bound is not in `ScalarObligationIdentity` at all; it appears only in the rendered `harness.rust` as the `lower`/`upper` literals. The `x + y` doc comment claims CG reads the conversion's `[10, 29]` bound, and nothing asserts it. Failure scenario: CG picks a different result bound, or renders the node under another operation identity with the same argument ranges, and both intent tests stay green. The pass-through tests only compare against a direct CG call, so they cannot catch it. | tests/drive.rs:702-737, 746-779 |

## Rust review

Lane: rust-review over `tests/drive.rs`, the only Rust change. The repo idioms
are in `AGENTS.md`/`CLAUDE.md`, `clippy.toml` and the `[lints]` table.

- The new test and `OVERFLOW_UNIT`/`overflow_input` follow the file's existing
  pattern. The test carries `/// FR-001-AC-6 (TC-006)` and asserts typed
  accessors (`stage()`, `code()`, `kind()`), not message text.
- `#[ignore]` was removed from both intent tests. No new `#[allow]`, no
  `unsafe`, and no panics in library code, since `src/` is unchanged.
- The `Cargo.toml` comment marks the revisions as informational, and the
  `Cargo.lock` change is the single CG source line. CG edade59 still resolves
  `quire-contract-ir` at 48ab5dc and `qsl-replay` at daa0178, and
  `tests/dependency_edges.rs` passes.
- No Rust-idiom findings. The oracle-strength gap is FND-004.

## Gate results

- Coder log
  (`/tmp/claude-1000/-home-peter-dev/09ab4eba-c376-42ef-aaec-be2e14ce86e4/scratchpad/driver-ir302-ci.log`):
  first line `head=9c1a9bdee9aaef95c6e95fdd829cd55f94dd31c1`, last line
  `exit=0`. `tests/drive.rs` shows 15 passed, 0 ignored.
- Reviewer log
  (`/tmp/claude-1000/-home-peter-dev/09ab4eba-c376-42ef-aaec-be2e14ce86e4/scratchpad/rv-driver-7-ci.log`):
  `make ci` at 9c1a9bd, first line `head=9c1a9bd…`, last line `exit=0`.
  `tests/drive.rs` shows 15 passed, 0 ignored. The fmt-check, clippy, deny,
  audit-unsafe, docs and spec steps were all green. The spec step also saw
  these two untracked SR files, which validate.
- Probe (`scratchpad/rv7/probe/tests/probe.rs`, outside the repo): 1 passed.
  It printed the dispositions cited in FND-001 and FND-002.

## Dispositions

Re-checked at 670c56e91c5d3cac64a07315f3a3ee2d8d876ae7 (fix round, one commit on top of 9c1a9bd).

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 670c56e. FR-001:176-189, TC-006 step 9 and the `OVERFLOW_UNIT` doc now say the return-context refusal at check is "one shape, not a general guarantee", and they name the dead-branch unit that reaches `ResultBoundUnreachable`. The test is renamed `tc_006_x_plus_100_in_the_return_context_is_refused_at_check`, so it no longer overclaims. |
| FND-002 | fixed | 670c56e. FR-001's "Real limits remain" and README.md:27-38 now list three things. First, path conditions are ignored until IR-304, with the dead-branch and live-branch `[(0, 9), (100, 100)]` measurements. Second, `RequiresBound` applies to any operand that is neither bounded nor a literal, with the nested `(x + 1) + 1`, `x * -1` and `-(x + 1)` cases. Third, the distinct-bound `AmbiguousBound` rule. |
| FND-003 | fixed | 670c56e. The README now says "narrowed to more than one distinct bound, or consumed by both a narrowing conversion and any other node", and says conversions to the same bound are deduplicated. FR-001 adds the same sentence. That sentence is still from reading CG's `narrowing` code, not measured, and no test backs it. It is accurate against CG edade59. |
| FND-004 | fixed | 670c56e. Both intent tests now assert `harness.identity.operation_identity == "quire.op.integer.add"`. They also assert that the rendered `harness.rust.contents` contains the result-bound pair: `rt::Integer::from(0_i64), rt::Integer::from(10_i64)` for `x + 1` and `rt::Integer::from(10_i64), rt::Integer::from(29_i64)` for `x + y`. That pair does not collide with the argument `assume` lines. A substring over rendered source is weaker than a typed field, but the identity has no result-bound field. |

Disposition-pass gates at 670c56e: the coder's `driver-ir302-ci.log` starts with `head=670c56e…` and ends `exit=0`. The reviewer's `rv-driver-7b-ci.log` (`make ci`) starts with `head=670c56e…` and ends `exit=0`, and `tests/drive.rs` shows 16 passed, 0 ignored. `quire coverage` shows 0 unbacked rows, 0 status lies and 0 untracked symbols. No new findings.
