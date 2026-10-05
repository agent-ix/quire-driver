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

Verify cache reads accept only entries matching the requested identity and
typed reader, and that CA-5 values are not stored.

Scope: FR-012-AC-3 through FR-012-AC-4.

## Test Procedure

Insert an eligible deterministic-budget incomplete result and confirm a typed
hit; an always-miss fake must fail this assertion. Then try a missing entry and
entries with malformed JSON, noncanonical bytes, an incorrect filename, a
changed stored preimage, a wrong preimage digest, an entry above the per-entry
limit, and a value the requested typed reader rejects. Attempt to store each
excluded result category separately: internal failure, cancellation, deadline,
time-limited incomplete result, plugin result and a value containing wall time.
Query each key after the attempted insertion.

## Expected Results

- A valid deterministic-budget incomplete entry is returned as a typed hit.
- Every absent, invalid, mismatched, per-entry-oversize or incorrectly typed
  entry is a miss; no invalid value is returned.
- Every CA-5 excluded category returns `NeverStore` and remains unreadable on a
  subsequent lookup.
- A valid typed entry returns the value produced by the same typed reader used
  for a fresh result.
