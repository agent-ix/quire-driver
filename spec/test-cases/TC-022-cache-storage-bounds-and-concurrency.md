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

Verify cache write, replacement, eviction and concurrent insertion against the
caller's per-entry and total-store limits.

Scope: FR-012-AC-5 through FR-012-AC-7.

## Test Procedure

Use a filesystem spy that fails a direct write to the final key name. Write an
entry through a unique temporary file in the same directory, flush and close
it, then atomically rename it; replace it under the same key. Leave an
abandoned temporary file and a symlink with an entry-like name before repeating
insertion and lookup. Verify temporary bytes count toward total capacity. Start
same-key writers concurrently, then run two cache processes that select the
same eviction candidate and arrange for one to remove it first. Exercise an
exactly-at-limit store, an insertion requiring eviction, an entry exceeding the
per-entry limit, and an entry unable to fit within the total limit.

## Expected Results

- Direct writes to a final entry name fail verification; only atomic rename
  publishes an entry, and no lookup observes a partial write.
- Temporary files count toward total capacity. Cleanup removes abandoned
  temporaries without following symlinks.
- Concurrent same-key writers leave a valid typed entry. In the cross-process
  eviction race, `NotFound` for an already-removed candidate does not fail the
  insertion.
- Exactly-at-limit storage is accepted; entries that cannot fit per-entry or
  total limits return `NotStored`, and eviction otherwise keeps the store within
  its total byte limit.
