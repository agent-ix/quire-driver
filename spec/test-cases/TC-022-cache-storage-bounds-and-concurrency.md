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

Scope: FR-012-AC-9 through FR-012-AC-12.

## Test Procedure

Observe the insertion boundary immediately before rename: the unique temporary
file must be complete and the final key name absent. Also remove the temporary
file at that boundary to exercise rename `NotFound`. Leave an abandoned owned
temporary file, an unrelated `.tmp`-like file, foreign JSON data, and a symlink
with an entry-like name before insertion and lookup. Verify all regular files
count toward capacity while only validated owned entries and owned temporaries
are removable. Start same-key writers concurrently and inject removal of the
selected eviction candidate between directory scan and unlink to exercise the
cross-process `NotFound` path. Exercise exactly-at-limit storage, insertion
requiring eviction, per-entry and total-capacity refusal, and a provider result
above the JCS integer range.

## Expected Results

- The final key name is absent while the complete temporary file is ready;
  atomic rename publishes the entry. A vanished temporary returns `NotStored`.
- Temporary files count toward total capacity. Cleanup removes abandoned
  temporaries without following symlinks.
- Concurrent same-key writers leave a valid typed entry. In the cross-process
  eviction race, `NotFound` for an already-removed candidate does not fail the
  insertion.
- Foreign files count toward capacity and remain untouched; invalid entries
  cannot be evicted as owned cache content. `.tmp` substrings alone do not make
  a file an owned temporary.
- Exactly-at-limit storage is accepted; entries that cannot fit per-entry or
  total limits return `NotStored`, and eviction otherwise keeps the store within
  its total byte limit.
- A result the canonical JSON serializer refuses is returned by the provider
  while its cache-write outcome is `NotStored`.
