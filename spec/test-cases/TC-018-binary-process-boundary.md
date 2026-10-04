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
2. Run the built `quire` process with controlled typed outcomes whose QSL
   categories return 20, 21, 22 and 30. For each run, derive the expected status
   by calling QSL's `Category::exit_code` on that outcome's category, then
   compare it to the child process status.
3. Repeat those runs in machine mode. Parse stdout as exactly one JSON value and
   compare it to the QSL serialization of the same typed `quire-outcome/1`
   value; check that the binary added no wrapper or fields and kept diagnostics
   on stderr.

Test location: planned driver binary integration tests and entry-point review.

## Expected Results

- Root argv selects the typed operation. The binary uses ix-cli-kit helpers for
  their applicable shared mechanics while retaining its own root argv boundary.
- Each child status is exactly the code returned by QSL's exit function,
  including 20, 21, 22 and 30. There is one pass-through to
  `std::process::ExitCode` and no ix-cli-kit `Outcome` conversion.
- Machine stdout contains exactly the unchanged QSL `quire-outcome/1` value
  for each run; diagnostics remain on stderr.
