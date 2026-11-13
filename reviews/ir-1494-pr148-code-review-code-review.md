---
id: SR-17940486709564847047748
title: "IR-1494 PR #148 code review (rust lane)"
type: SpecReview
analysis: code-review
date: 2026-11-12
scope: "agent-ix/quire-driver PR #148 at 5438a5ac (base origin/main); IR-1494; Cargo.lock, src/refusal.rs, tests/drive.rs, tests/frame_replay.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: reviews
  - target: ix://agent-ix/quire-driver/TC-034
    type: reviews
---

## Summary

Ticket: IR-1494. Code review with the rust-review lane over `git diff origin/main...HEAD` at 5438a5ac.

Cargo.lock: manifests unchanged (`branch = "main"`), no pins, hashes or `rev`/`tag` added. Reproduced from main's lock with `cargo update -p quire-contract-codegen --precise 7fb4f53c`: the same package set moves (CG, quire-contract-ir/model, quire-exact, the QSL crates and filament-core-data, which cargo refreshes together with the CG git source); the only byte difference is that QSL main has since advanced from 4b7b6003 to af59dca3. So the lock movement is exactly what the stated command produces. The PR body says CG only; QSL and filament-core-data move too, as a side effect of the same command.

`tests/frame_replay.rs`: adapted faithfully. The frame role takes QSL's own `located.site.obligation_identity()` (the identity the replay later matches), and the contract role is a distinct identity built the same way CG's own tests build one (`role_parity_tests.rs`, `tests/common/state_frame_identities.rs`); only `.frame.identity` is consumed. `supplied_limits: None` keeps the prior behaviour.

`Recorder.tampered_requirements` lives only in `tests/drive.rs` (an integration-test file); nothing in `src/` references it, so the seam is test-only and adds no public API.

Mutants (each applied, run, reverted; `git status` clean after each): M1 guard-root loop `.iter().take(0)` in `src/drive.rs` fails 5 drive tests incl. AC-11 and AC-13; M2 `RequirementRecord => Code::InvalidRequest` in `src/refusal.rs` fails the AC-8 and AC-13 drive tests and the AC-9 unit test; M3 (reviewer) guard polarity flipped in both item and record projection fails AC-1/AC-7/AC-12 tests and the AC-11 test at its `!(` polarity assertion (tests/drive.rs:4158); M4 (reviewer) guards stripped from item and record projection fails the AC-11 test at `assumes.len() == 1` (tests/drive.rs:4157) plus AC-1/AC-7/AC-12. Baseline: drive 47 passed, refusal lib 41 passed.

## Verdict

Code is sound and the new tests can fail; two low findings only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The AC-11 test checks one guard `kani::assume` per guarded harness and its polarity, but not that the assume follows the symbolic inputs, which FR-020-AC-11 states; an assume emitted before the `kani::any()` bindings would pass. Assert the assume's line index is after the last symbolic-input binding. | tests/drive.rs:4146-4158 |
| FND-002 | low | The AC-9 unit test hand-lists the eight `RequirementRecordMismatch` variants and asserts `messages.len() == 8`; the enum is not `#[non_exhaustive]` but nothing ties the list to it, so a ninth CG variant would leave the test green and AC-9's "each variant" silently uncovered. Add a wildcard-free `match` over the variants (compile-time exhaustiveness) in the test. | src/refusal.rs:2250-2298 |

## Dispositions

Round 1, head 32113a0e. Full `cargo test --locked --workspace` and `cargo fmt --check` pass; Cargo.lock is unchanged from 5438a5ac.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | tests/drive.rs:4227-4231 asserts the last `kani::any` < the guard assume < `let outcome = ` (oracle call); reviewer mutants M3/M4 still fail this test. |
| FND-002 | fixed | src/refusal.rs:2287-2303 adds a wildcard-free `match` over all eight variants and asserts the covered set is 0..8. |
