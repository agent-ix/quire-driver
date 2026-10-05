---
id: TC-022
title: "Keep cache writes atomic and within the caller's size limit"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-012
    type: verifies
---
# TC-022: Keep cache writes atomic and within the caller's size limit

## Description

Verify cache write, replacement, eviction and same-key writer behavior against
the caller's store-size limit.

Scope: FR-012-AC-5 through FR-012-AC-7.

## Test Procedure

Write an entry and replace it under the same key. Leave an abandoned temporary
filename and a symlink with an entry-like name in the directory before
repeating insertion and lookup. Start concurrent writers for the same
eligible key and read the entry after both finish. Exercise a total store size
exactly at its limit, add another entry to trigger eviction, and attempt an
entry larger than the total limit.

## Expected Results

- No lookup observes a partial temporary write; a later write removes the
  abandoned temporary file without following the symlink, and the final named
  entry decodes with the requested typed reader.
- Concurrent same-key writers leave a valid entry and do not turn a successful
  write into a failure solely because a competing writer completed first.
- A store exactly at the configured limit remains accepted; adding an entry
  evicts as needed, and an entry that cannot fit is not stored.
