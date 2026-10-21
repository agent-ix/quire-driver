---
id: TC-018
title: "Preserve the QSL outcome at the binary process boundary"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-002
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-014
    type: verifies
---
# TC-018: Preserve the QSL outcome at the binary process boundary

## Description

Verify FR-002-AC-2 through FR-002-AC-4, FR-002-AC-6, FR-002-AC-8 through
FR-002-AC-9 and FR-002-AC-14 through FR-002-AC-16 at the driver's root argv
and process boundary. QSL ADR-029
supplies the outcome value and exit semantics; this case checks that the
binary preserves them while using ix-cli-kit's exit, streams and JSON modules.

## Test Procedure

1. Inspect the binary entry point and root argv dispatch. Confirm that it invokes
   the selected typed operation and passes QSL's `Category::exit_code` result
   through `ix_cli_kit::exit::caller_exit_code` to
   `std::process::ExitCode`. Check that no path converts the category through
   ix-cli-kit's `Outcome` or a second numeric category mapping. `config` is used
   if CLI configuration is introduced; the optional `secrets` module is outside
   the driver scope.
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

   Write `driver-check.native` in a temporary directory using the exact bytes
   produced by the driver's existing `OVERFLOW_UNIT` fixture in `tests/drive.rs`.
   Reuse that fixture's factory and input arrangement (`overflow_input`), without
   duplicating its source or header here. Its driver-owned acceptance test,
   `tc_006_x_plus_100_in_the_return_context_is_refused_at_check`, establishes
   the check-stage refusal `undefined_expression/unproved-range`.

   | `TC018_CASE` | QSL result supplied at the operation boundary |
   | --- | --- |
   | `refused-range` | Call QSL `parse`, `select` and `check` on `driver-check.native`, with default limits and an uncancelled handle; retain its `StageFailure::Refused` with code `undefined_expression` and cause `unproved-range`, as established by the driver test above. |
   | `profile-gated` | Return the typed profile-gated `StageFailure::Refused` sample specified by QSL TC-769 step 3 and FR-285-AC-3, with its locus bound to this request's source. |
   | `cancelled` | Call QSL `check` on the parsed source with a handle already cancelled with `Requested`; retain `StageFailure::Cancelled(Requested)` (QSL FR-276-AC-1). |
   | `fault` | Return `StageFailure::Fault` holding QSL's `InternalFault::new("S3", "tc018-injected-fault")`, the fault variant specified by QSL FR-285-AC-3. |

   For each case, spawn the test-built `quire` with argv
   `["check", "<absolute path to driver-check.native>", "--format", "text"]`
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
   serialization through ix-cli-kit's `json` module, with no wrapper or changed
   fields. For `refused-range`, require
   a nonempty diagnostic with code `undefined_expression` and cause
   `unproved-range` in the parent outcome; retain its cause, locus and message
   as expected content. Render that
   diagnostic with QSL's renderer and the same source provision to obtain the
   nonempty `path:line:column: code: message` diagnostic line (ADR-029 IN-3).
   Assert that stderr contains that complete rendered line and stdout does
   not. The diagnostic's structured fields remain in stdout's outcome JSON,
   equal to the parent's value; the absent stdout content is the rendered
   diagnostic line, not those required fields.

4. Build the child with a dedicated test-only Cargo feature that enables an
   exit-function seam to supply raw code `5` in place of QSL's
   `Category::exit_code` result. The seam SHALL not be enabled in the default
   or release binary and SHALL not be selectable through production argv or
   environment variables. Confirm the child exits with
   `Category::InternalFailure.exit_code()` (`30`), stdout is empty in both
   text and machine modes, and stderr contains a plain-text diagnostic naming
   reserved code `5` and fallback code `30`.
5. Inspect the binary's output paths and verify text results and diagnostics
   use ix-cli-kit's `streams` module for stdout/stderr selection and write
   behavior. Verify the two primary-stdout failure cases in separate child
   processes using the `refused-range` case and `--format json`:

   a. For a broken pipe, use a Unix named pipe as the source path and start the
      child with stdout piped. Repeatedly attempt to open the source FIFO's
      writer in nonblocking mode until it succeeds, retrying `ENXIO` and
      `EINTR` every 10 ms. Use one 10-second deadline from child spawn through
      child exit. During retries, poll the child; if it exits before the writer
      opens, fail the test. Once the writer opens, close the
      parent's stdout reader before writing the fixture bytes to the source
      FIFO. Require the child to exit within that deadline and compare its
      status with the independently obtained QSL refusal category
      exit result (`20`). Require stderr not to contain an `OutputFailure`
      diagnostic, as specified by FR-002-AC-14. Do not assert stdout contents
      after closing its reader. On any timeout, premature child exit or other
      setup/write failure, kill the child if it is still running and wait for
      it before failing the test.
   b. On Linux, redirect the child's stdout to `/dev/full`, whose writes fail
      with a non-`BrokenPipe` I/O error. Require process status `30` and stderr
      to contain an `OutputFailure` diagnostic identifying the outcome document
      write. Do not compare device contents or require stdout to be empty or
      complete.

Test location: `quire-cli/tests/binary_process.rs` with its test-only
operation adapter and fixtures under `quire-cli/tests/fixtures/`; entry-point
review covers `quire-cli/src/main.rs` and `quire-cli/src/cli.rs`.

## Expected Results

- Root argv selects the typed operation. The exit boundary uses
  `ix_cli_kit::exit::caller_exit_code` while retaining its own root argv
  boundary. Stream writes and JSON encoding use the corresponding ix-cli-kit
  modules.
- For typed-result cases, each child status is exactly the code returned by
  QSL's exit function, including 20, 21, 22 and 30. The test-only code-5 case
  instead uses status `30` and has no stdout result. There is one pass-through
  to `std::process::ExitCode` and no ix-cli-kit `Outcome` conversion.
- Machine stdout contains exactly the unchanged QSL `quire-outcome/1` value
  for each typed-result run. The code-5 fallback case writes no stdout result
  and emits its specified plain-text diagnostic only on stderr. The known
  `undefined_expression/unproved-range` diagnostic
  retains its structured fields in that value, and its complete nonempty
  rendered line appears on stderr and is absent from stdout.
- With the source FIFO handshake, a closed stdout reader produces the original
  QSL refusal status `20` and does not produce an `OutputFailure` diagnostic.
  With Linux `/dev/full`, a non-broken stdout write failure produces status
  `30` and an `OutputFailure` diagnostic on stderr. The failed stdout write may
  have emitted a prefix, so the test makes no empty-or-complete stdout claim.
