---
id: SR-021
title: "Code review (with Rust lane) of quire-driver PR #14"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-driver@8ca0d9a66a845472a2adabd1a6d39801cad9e93c; Cargo.lock; README.md; src/drive.rs; src/handoff.rs; src/lib.rs; src/refusal.rs; src/registry.rs; src/upstream.rs; tests/drive.rs"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-driver/FR-001, type: reviews }
---
# Code review (with Rust lane) of quire-driver PR #14

## Summary

Ticket: IR-606. PR: quire-driver#14, frozen at 8ca0d9a. The diff ports
`drive` from the removed `qsl_replay::spine::compile` to QSL's typed `parse`
→ `select` → `check` → `package` lifecycle, adds `DriverFault::Spine` for
QSL lifecycle faults, classifies three new CG generation errors, and bumps
the QSL, IR, CG and canonical lock revisions. The port itself is faithful to
QSL's own composition, but compile-stage limit refusals lost their catalog
cause: they now render `stage_limit_exceeded` where FR-001-AC-5 requires
`stage_limit_exceeded/<kind>-exceeded`. A probe in a throwaway copy of the
tree reproduced it. The test that would have caught it was deleted in this
same diff.

## Verdict

**FAIL**: one `high` finding (FND-001, a spec claim the code violates).

Clean areas: no vendoring or in-repo duplication. The lifecycle composition
in `QslCg::compile` (src/upstream.rs:92-125) is written against QSL's public
operations and copies nothing; it matches the order QSL documents for the
CLI's `compile`. No new hash, digest or pin. The lock changes are package-manager
resolution only, and `unicode-ident` 1.0.26→1.0.24 is a resolver result,
not a hand pin. No new `unwrap`/`expect`/`panic!` or `as` cast in library code.
`handoff.rs` drops the `if let Some(root)` because `DefinitionLock::entry`
now returns the row directly; that is correct. `compile_cause` and
`generation_code` stay exhaustive with no catch-all arm. No `#[allow]`,
lint or gate was weakened, and no CI workflow file changed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `compile_cause` maps `CompileRefusal::Limit` to `None`, so a compile-stage limit renders bare `stage_limit_exceeded` and drops QSL's own cause (`input-bytes-exceeded`, `work-budget-exceeded`, ...) | src/refusal.rs:287 |
| FND-002 | low | `ProviderManifest::tool` is now read nowhere: `BackendDescriptor::admit` no longer takes a `ToolIdentity`, yet the public field and its "fields the driver registers a backend from" doc remain | src/registry.rs:21 |
| FND-003 | low | Crate doc step 2 still says "E4: the same call emits" the v2 bytes; E4 is now the separate `package` operation, which step 1 already names | src/lib.rs:12 |
| FND-004 | low | `generation_code`'s doc enumerates the internal-fault variants but omits the three this diff adds (`DuplicateHarness`, `KaniRecordCountMismatch`, `KaniDuplicatePositionOutOfRange`) | src/refusal.rs:218 |

### FND-001 detail

Escape cause: correct-requirement-no-evidence. FR-001-AC-5 states the rule; this diff removed the evidence for it.

`refusal_or_fault` turns every `StageFailure::Limit(limit)` into
`CompileRefusal::Limit(limit)`. `LimitExceeded::kind().catalog_cause()`
names the cause, and QSL's own `Display` renders
`stage_limit_exceeded/input-bytes-exceeded (bound 8, reached 123, raised by
source.source_bytes)`. The driver's `compile_code` appends a cause only when
`compile_cause` returns one, and the new arm returns `None`. The
`compile_cause` doc also leaves `Limit` out of its list of variants that
"name no cause". Reproduced by a probe test in a copy of the tree at this
sha:

- `limits.source.source_bytes = 8`: `kind=Limit code=stage_limit_exceeded stage=Compile(Source)`
- `limits.checking = limits.checking.with_work_budget(1)`: `kind=Limit code=stage_limit_exceeded stage=Compile(Check)`

The deleted `tc_004_command_level_refusals_are_typed` step asserted
`stage_limit_exceeded/nesting-depth-exceeded`. Fix: return
`Some(limit.kind().catalog_cause())` for `CompileRefusal::Limit`, and
restore a limit step that drives a reachable bound (gap-analysis SR-022
FND-001).

## Gates

The macOS `make ci` log for this head (`/private/tmp/IR-606-ci.log`, `head=8ca0d9a…`,
`exit=0`) shows these passing: `cargo fmt --all -- --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test` (3 + 1 + 20 + 0
tests passed), `cargo deny check licenses sources`, `check_unsafe_comments.sh`,
`cargo doc` with `-D warnings`, and `quire validate 'spec/**/*.md'`. Per the
dispatching brief, neither the gates nor Linux CI were re-run.

## Dispositions

Round 1, reviewed at `e48cca7f017f6a2642a26a593d4150b971024a0f` (fix commit `e48cca7` on top of `8ca0d9a`). Each outcome was checked against the code, spec and tests at that sha.

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | e48cca7 (src/refusal.rs:289) |
| FND-002 | fixed | e48cca7 (src/registry.rs:15-21) |
| FND-003 | fixed | e48cca7 (src/lib.rs:11-13) |
| FND-004 | fixed | e48cca7 (src/refusal.rs:221-225) |
