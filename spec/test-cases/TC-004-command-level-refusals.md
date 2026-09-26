---
id: TC-004
title: "Command-level refusals are typed"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: verifies
---
# TC-004: Command-level refusals are typed

## Description

Verify the step, catalog code and outcome kind of the command-level
refusals.

Scope: FR-001-AC-5.

## Test Procedure

1. Drive a unit that does not parse.
2. Drive a unit whose body `not not not true` is four deep, with a checking
   depth limit of 3.
3. Build a registry from two manifests of `kani` with different digests, then
   from three: the same two, plus a third repeating the first digest (A, B,
   A).
4. Build a registry from a manifest advertising `not-a-kind`.
5. Take a `DriverRefusal::Fault` for a `supported` settlement outside its
   candidates, one for a vocabulary mismatch, one for an E5 `package_id`
   mismatch, a generation refusal for a duplicate request index, and one
   for a routed item naming a node the checked package does not hold
   (`DriverFault::UnknownNode`).
6. Take an E5 reader limit and CG's Kani item ceiling.
7. Drive the TC-006 integer unit through an upstream whose IR read answers
   with another unit's admitted package, then with a refusal.
8. With `kani` registered for `operation-contract` only, drive the TC-003
   negation unit naming `kani` as the request's backend.

Tests: `tests/drive.rs`, `tc_004_command_level_refusals_are_typed`,
`tc_004_e5_identity_and_refusal_are_faults` and
`tc_004_a_named_registered_backend_that_does_not_advertise_the_kind_settles_unsupported`;
`src/refusal.rs`, `tc_004_a_fault_is_an_internal_fault_with_runtime_invariant`
and `tc_004_handoff_and_generation_limits_are_limits`.

## Expected Results

- Step 1 refuses at the source stage with `invalid_syntax`, outcome kind
  refusal, and only the compile call was made.
- Step 2 refuses at the check stage with
  `stage_limit_exceeded`/`nesting-depth-exceeded`, outcome kind limit.
- Step 3 refuses at the registry with `invalid_capability`/`duplicate-backend`,
  one refusal per distinct conflicting digest (two), ascending: the A, B, A
  case still yields exactly those same two, not a third for the repeated
  digest -- `build_registry` reads `Registry::refusals()` once, after every
  manifest is registered, rather than accumulating what each `register` call
  individually returned.
- Step 4 refuses with `invalid_capability`/`unknown-kind`.
- Step 5's faults have outcome kind fault and code `runtime_invariant`, at
  the negotiation, conversion, hand-off and generation steps; the
  `UnknownNode` fault is at the generation step.
- Step 7 faults with `PackageIdMismatch`, then `HandoffRefused`, both at
  the hand-off step with `runtime_invariant`; the call log ends at the IR
  read, with no backend conversion and no negotiation.
- Step 6's are limit refusals: `stage_limit_exceeded` at the hand-off and
  `resource_exhausted` at generation.
- Step 8 settles `unsupported`, naming `kani` in the cause
  (`unsupported-requested-capability`): `kani` is registered but does not
  advertise `value-validity`. Nothing routes.
