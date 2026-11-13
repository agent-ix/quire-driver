---
id: SR-17940486709564847047959
title: "IR-1530 PR #149 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-driver PR #149 at 9aca0720, git diff origin/main...HEAD; IR-1530; spec/functional/FR-005-register-first-party-providers.md, spec/functional/FR-009-settle-execute-and-replay-results.md (context), src/provider.rs, src/drive.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-driver/FR-005
    type: reviews
---

## Summary

Ticket: IR-1530. Planless gap analysis over the diff. Plan completion: not
assessed. `quoin matrix` (quoin 0.29.1) lists FR-005-AC-22 as `Test | tagged`
(no run evidence bound, the same as AC-20 and AC-21). Both new tests carry
`#[trace("TC-012", "FR-005-AC-22")]` and assert the AC: the general-route test
checks the harness path, node, operation identity, 4-bit width, general route
and the shipped row by equality, and that no terminal is produced. The new
production code (`run_items` gate, the two failure arms, the `refusal.rs` arm)
belongs to FR-005-AC-22 or to CG's generation error surface; nothing new is
unowned.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | No requirement or test covers the refusal at the driver's second Kani launch site. FR-005-AC-22 is scoped to the linked Kani provider; the routed scalar native execution path (FR-009, `src/drive.rs:539`, `cg::execute_kani_scalar_native` on a general-route harness) launches Kani with no tractability decision, and no FR-009 AC states that a general-route scalar the shipped table refuses is not launched there. IR-1530 closes on "the general-route scalar refused before Kani launches" on the production path, which this leaves half covered. Needed: an FR-009 AC (or widen FR-005-AC-22's statement to every driver Kani launch) with a test through `drive` with `ScalarNativeInput`. | spec/functional/FR-005-register-first-party-providers.md:172, spec/functional/FR-009-settle-execute-and-replay-results.md:268-275, src/drive.rs:534-548 |

## Verdict

FR-005-AC-22 itself is implemented and tested for the provider path. The gap
is coverage of the other Kani launch site (FND-001, same root cause as
SR-17940486709564847047958 FND-001); the native-route test's weak intent is
recorded there as FND-002 and is not repeated here.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-002 | medium | The shipped row does not cover the measured general-route timeout at 3 bits. A row at `B` bits applies when the operation width is at least `B` (CG `tractability.rs:363-375`), and the shipped row has `max_operand_bits: 4`. `interval_bits(4, 4)` is 3 (CG `oracle/clause.rs:192-196`: bit length of 4), so the decider returns `Run` for the drive.rs receipt fixture `x + 1` over `Int[4, 4]`. The fix round reports that this fixture timed out at 600 s on the general route (`ProofNotVerified`), so general-route add/sub at 3 bits or fewer still launch and use the full wall clock. The boundary is a gap in the refusal's data, not in the driver wiring: CG should record a measured row at the smallest width observed to time out (3 bits from this run, or lower after a probe) under IR-1530, and the driver picks it up with a Cargo.lock bump and no code change. It does not block this PR; it does weaken IR-1530's outcome (the general integer scalar route recorded as not tractable). | CG docs/qualification/ir1530-general-route-scalar-table.json:5, CG src/kani/tractability.rs:363-375, CG src/oracle/clause.rs:192-196, src/drive.rs:1421-1424 |

## Dispositions

Disposition pass 1, checked at d083388d against 9aca0720..d083388d.

| FND | outcome | reason |
|---|---|---|
| FND-001 | fixed | FR-009-AC-26 added; TC-016 gained the step and the scope; `scalar_native_tractability_tests::general_route_scalar_is_refused_before_the_paired_native_run` and `native_route_scalar_reaches_the_paired_native_run` are tagged `#[trace("TC-016", "FR-009-AC-26")]`. |
