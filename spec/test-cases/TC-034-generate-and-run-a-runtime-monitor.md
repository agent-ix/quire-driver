---
id: TC-034
title: "Generate and run a runtime monitor from QSL source"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: verifies
---
# TC-034: Generate and run a runtime monitor from QSL source

## Description

Verify that `quire generate` constructs an RT runtime monitor from a selected
QSL plan, consumes live RT event records in order and returns the typed step
results in one final outcome.

Scope: FR-020-AC-1 through FR-020-AC-6.

## Test Procedure

1. Prepare a QSL source request with one runtime-monitor plan, its required
   authority and limits, and a runtime source fixture that returns distinguishable
   results for its first and second events and watermark.
2. Encode two RT `Event` records and one RT `Watermark` record using their
   declared fields and value encodings; write them as newline-delimited JSON
   to a named file and invoke `quire generate <request-file> --events <path>`.
3. Compare each event and watermark result with the corresponding direct RT
   operation on the same prepared program. Repeat with `--events -` and the
   same bytes on standard input.
4. Repeat with an empty stream; with no selected plan; with two selected plans;
   with program construction or initialization refused; and with a valid record,
   a malformed or RT-rejected record, then a valid record.
5. End a successful stream at EOF and compare its results with the last
   completed RT step; do not send a synthetic watermark or event.
6. Capture stdout, stderr and process status in JSON mode. Verify the CLI emits
   one final outcome document and that event records are consumed only as
   specified by FR-020.

## Expected Results

- The valid request builds the selected QSL monitor through the driver and RT
  APIs and emits one step result per consumed record in input order.
- The named event file and standard-input stream produce equal results.
- An empty stream runs initialization once and has no event-derived step or
  temporal verdict.
- A request or RT startup refusal is returned before the first event record is
  consumed.
- A malformed or refused record stops consumption at that record, preserves
  prior committed results and does not consume the next record.
- EOF does not synthesize an event, watermark or temporal verdict.
- JSON mode emits exactly one QSL `quire-outcome/1` document for the invocation.
