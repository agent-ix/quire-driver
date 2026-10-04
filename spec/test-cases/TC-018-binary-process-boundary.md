---
id: TC-018
title: "Preserve the QSL outcome at the binary process boundary"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: verifies
---
# TC-018: Preserve the QSL outcome at the binary process boundary

## Description

Verify FR-002-AC-2 through FR-002-AC-4 at the driver's root argv and process
boundary. QSL ADR-029 supplies the outcome value and exit semantics; this case
checks that the binary preserves them while using ix-cli-kit shared mechanics.

## Test Procedure

1. Inspect the binary entry point and root argv dispatch. Confirm that it invokes
   the selected typed operation, uses the applicable ix-cli-kit output, config
   and version helpers, and has one conversion from QSL's
   `Category::exit_code` result to `std::process::ExitCode`. Check that no path
   converts the category through ix-cli-kit's `Outcome` or a second numeric
   category mapping.
2. Build the `quire` child process with a test-only adapter at the typed `check`
   operation call. Select its case through the child environment variable
   `TC018_CASE`, with the four values in the table below. The adapter receives
   the request produced by the normal root argv parser and input reader; it
   records the received source bytes, source identity, path, profile, limits
   and selected case in a separate test observation file named by the absolute
   path in `TC018_REQUEST_PATH`, before returning the selected QSL typed result.
   This test observation is not written to stdout or stderr. The adapter does
   not replace argv parsing, outcome serialization, stream writes or the
   process exit conversion, and it supplies no numeric exit code. The selector
   exists only in the test build.

   Write `inv.native` in a temporary directory with these exact UTF-8 bytes
   (including a final newline), from QSL FR-100-AC-5 and its TC-451 example:

   ```text
   language "ix:native" edition "1-draft";
   profile v = "quire.value.complete/v1";
   type Digit = Int[0, 9];
   function inv using v(x: Digit): Boolean pure { 1 / x > 0 }
   ```

   | `TC018_CASE` | QSL result supplied at the operation boundary |
   | --- | --- |
   | `ill-typed` | Call QSL `parse`, `select` and `check` on `inv.native`, with default limits and an uncancelled handle; retain its `StageFailure::Refused` with the `ill_typed` diagnostic (QSL FR-286-AC-2). |
   | `profile-gated` | Return the typed profile-gated `StageFailure::Refused` sample specified by QSL TC-769 step 3 and FR-285-AC-3, with its locus bound to this request's source. |
   | `cancelled` | Call QSL `check` on the parsed source with a handle already cancelled with `Requested`; retain `StageFailure::Cancelled(Requested)` (QSL FR-276-AC-1). |
   | `fault` | Return `StageFailure::Fault` holding QSL's `InternalFault::new("S3", "tc018-injected-fault")`, the fault variant specified by QSL FR-285-AC-3. |

   For each case, spawn the test-built `quire` with argv
   `["check", "<absolute path to inv.native>", "--format", "text"]`
   and that case's `TC018_CASE`, with a fresh `TC018_REQUEST_PATH`. Capture
   stdout, stderr and the child status. Require the observation file to exist
   and contain the selected case and exact fixture bytes and path; a missing
   observation or mismatched request fails the test. Independently obtain the
   typed result in the parent using the table's operation or constructor over
   that recorded request, including its source identity, profile and limits,
   and the table's cancellation arrangement. Use QSL's outcome conversion and
   serialization to obtain the expected `quire-outcome/1` value, and call
   QSL's `Category::exit_code` on its category for the expected status.
   Compare the status to that case's independently obtained expectation; the
   four expectations cover 20, 21, 22 and 30 without deriving them from child
   output or assigning them in the adapter.
3. Repeat each invocation with `--format json`, keeping its case, source and
   request arrangement unchanged. Parse stdout as exactly one JSON value,
   followed only by whitespace, and compare it to the parent's expected QSL
   serialization, with no wrapper or changed fields. For `ill-typed`, require
   exactly one nonempty diagnostic with code `ill_typed` in the parent outcome;
   retain its cause, locus and message as expected content. Render that
   diagnostic with QSL's renderer and the same source provision to obtain the
   nonempty `path:line:column: code: message` diagnostic line (ADR-029 IN-3).
   Assert that stderr contains that complete rendered line and stdout does
   not. The diagnostic's structured fields remain in stdout's outcome JSON,
   equal to the parent's value; the absent stdout content is the rendered
   diagnostic line, not those required fields.

Test location: planned driver binary integration tests with the test-only
operation adapter, and entry-point review.

## Expected Results

- Root argv selects the typed operation. The binary uses ix-cli-kit helpers for
  their applicable shared mechanics while retaining its own root argv boundary.
- Each child status is exactly the code returned by QSL's exit function,
  including 20, 21, 22 and 30. There is one pass-through to
  `std::process::ExitCode` and no ix-cli-kit `Outcome` conversion.
- Machine stdout contains exactly the unchanged QSL `quire-outcome/1` value
  for each run. The known `ill_typed` diagnostic retains its structured fields
  in that value, and its complete nonempty rendered line appears on stderr
  and is absent from stdout.
