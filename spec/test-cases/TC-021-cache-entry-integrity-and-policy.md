---
id: TC-021
title: "Reject invalid cache entries and excluded values"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-012
    type: verifies
---
# TC-021: Reject invalid cache entries and excluded values

## Description

Verify that cache reads accept only entries matching the requested identity
and typed reader, and that CA-5 values are not stored.

Scope: FR-012-AC-3 through FR-012-AC-4.

## Test Procedure

For one valid key, try a missing entry and entries with malformed JSON,
noncanonical bytes, an incorrect filename, a changed stored preimage, a wrong
preimage digest and a value the requested typed reader rejects. Then attempt to
store each excluded result category separately: internal failure, cancellation,
deadline, time-limited incomplete, plugin result and a value containing wall
time. Query each corresponding key after the attempted insertion.

## Expected Results

- Every absent, invalid, mismatched, oversize or incorrectly typed entry is a
  miss; no invalid value is returned.
- Every excluded category returns never-stored and remains unreadable on a
  subsequent lookup.
- A valid typed entry still returns the value produced by the same typed
  reader used for a fresh result.
