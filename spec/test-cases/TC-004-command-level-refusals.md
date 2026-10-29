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
refusals, including lossless FR-272 mapping of CG contract-package reader
results.

Scope: FR-001-AC-5 and FR-001-AC-22.

## Test Procedure

1. Drive a unit that does not parse.
2. Drive a valid unit with `limits.source.source_bytes = 8`, then with
   `limits.checking.with_work_budget(1)`.
3. Build a registry from two manifests of `kani` advertising different kinds,
   then from three: the same two, plus a third repeating the first (A, B, A).
4. Build a registry from a manifest advertising `not-a-kind`.
5. Take a `DriverRefusal::Fault` for a `supported` settlement outside its
   candidates, one for a vocabulary mismatch, a generation refusal for a duplicate request index, and one
   for a duplicate harness, a Kani record-count mismatch, a Kani duplicate
   position out of range, and a routed item naming a node the checked package
   does not hold
   (`DriverFault::UnknownNode`).
6. Take an E5 reader limit and CG's Kani item ceiling.
7. Drive the TC-006 integer unit through an upstream whose IR read answers
   with another unit's admitted package, then with a refusal.
8. With `kani` registered for `operation-contract` only, drive the TC-003
   negation unit naming `kani` as the request's backend.
9. Feed QSL lifecycle `Fault`, `Cancelled` and `Limit` failures to the
   driver mapping for `parse`, `select`, `check` and `package`.
10. At the CG package-reader boundary, return
    `ContractPackageReadRefused` results covering a returned code with a
    cause and one without a cause, plus present and absent optional payload
    members. Return `ContractPackageReadIncomplete` once for each permitted
    `limit_kind`: `Bytes`, `Nodes`, `Edges`, `Occurrences` and `Work`; also
    inject `Diagnostics` as a contract-violating CG result.

Tests: `tests/drive.rs`, `tc_004_command_level_refusals_are_typed`,
`tc_004_e5_identity_and_refusal_are_faults` and
`tc_004_a_named_registered_backend_that_does_not_advertise_the_kind_settles_unsupported`;
`src/refusal.rs`, `tc_004_a_fault_is_an_internal_fault_with_runtime_invariant`
and `tc_004_handoff_and_generation_limits_are_limits`, plus
`routed_generation_invariant_errors_are_generation_faults`;
`src/upstream.rs`, `lifecycle_faults_retain_details_and_operation_stage`,
`lifecycle_cancellation_is_an_internal_fault_at_the_operation_stage` and
`lifecycle_limits_preserve_qsl_payload_cause_and_field_stage`.

## Expected Results

- Step 1 refuses at the source stage with `invalid_syntax`, outcome kind
  refusal, and only the compile call was made.
- Step 2 refuses at the check stage with
  `stage_limit_exceeded/work-budget-exceeded`, outcome kind limit. The
  source-byte case refuses at the source stage with
  `stage_limit_exceeded/input-bytes-exceeded`. Only the compile call is made
  in either case.
- Step 3 refuses at the registry with `invalid_capability`/`duplicate-backend`,
  one refusal for the conflicted identity: the A, B, A case still yields
  exactly that one, not a second for the repeated manifest -- `build_registry` reads `Registry::refusals()` once, after every
  manifest is registered, rather than accumulating what each `register` call
  individually returned.
- Step 4 refuses with `invalid_capability`/`unknown-kind`.
- Step 5's faults have outcome kind fault and code `runtime_invariant`, at
  the negotiation, conversion, hand-off and generation steps; the
  `UnknownNode` fault is at the generation step.
- Step 7 faults with `HandoffRefused` at the hand-off step with `runtime_invariant`; the call log ends at the IR
  read, with no backend conversion and no negotiation.
- Step 6's are limit refusals: `stage_limit_exceeded` at the hand-off and
  `resource_exhausted` at generation.
- Step 9 maps faults and cancellation to `runtime_invariant`, outcome kind
  fault, at the operation's compile stage: source, intake, check or emit.
  Limits preserve their QSL payload and catalog cause with outcome kind limit.
- Step 8 settles `unsupported`, naming `kani` in the cause
  (`unsupported-requested-capability`): `kani` is registered but does not
  advertise `value-validity`. Nothing routes.
- Step 10's refused result becomes an FR-272 refusal diagnostic at
  `package_read`, with CG code and cause unchanged, category `refusal`, the
  `path` as the FR-274 `json_path` location, and a present `locus` retained
  as typed `CheckedNodeId`. The complete typed CG projection remains
  available unchanged, including `code`, `path`, `cause`, `locus`,
  `contract_version`, `document_pointer`, `expected` and `actual`, including
  all optional members and their absence. Each returned code has an FR-271
  row; the diagnostic classification is not derived from text.
  The incomplete results map exactly as follows: `Bytes` to
  `input-bytes-exceeded` / `CheckedPackageReadLimits.bytes`; `Nodes` to
  `node-count-exceeded` / `CheckedPackageReadLimits.nodes`; `Edges` to
  `edge-count-exceeded` / `CheckedPackageReadLimits.edges`; `Occurrences` to
  `occurrence-count-exceeded` / `CheckedPackageReadLimits.occurrences`;
  `Work` to `work-budget-exceeded` / `CheckedPackageReadLimits.work`. Each has
  `stage_limit_exceeded` at `package_read`, category `incomplete` (numeric
  category 22), `bound = limit`, `actual = consumed`, `kind` equal to the
  cause tag, and location from `path`, absent for `Bytes`. The CG typed
  incomplete projection (`limit_kind`, `limit`, `consumed`, `path`) is
  retained unchanged. `Diagnostics` returns a `runtime_invariant` fault at
  `package_read`, preserves its typed CG projection and emits no FR-272 stop
  diagnostic. No mapping is inferred from enum formatting or payload text,
  and no reader result is converted into the E5 result.
