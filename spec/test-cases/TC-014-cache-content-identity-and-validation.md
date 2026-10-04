---
id: TC-014
title: "Miss on changed manifest content and reject corrupt cache entries"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-007
    type: verifies
---
# TC-014: Miss on changed manifest content and reject corrupt cache entries

## Description

Verify that cache identity follows canonical meaning inputs, including provider
manifest bytes, and that reads validate the entry before reuse.

Scope: FR-007-AC-1 through FR-007-AC-6.

## Test Procedure

1. Cache one eligible result for a fixed request and provider manifest.
2. Repeat the request unchanged, then change one manifest byte while keeping
   any recorded repository pin constant.
3. Change each other meaning input independently; then change only a path,
   timestamp, application version, deadline, or selected execution backend.
4. Corrupt the stored preimage, make the stored name disagree with the key,
   and replace the value with one rejected by the fresh-result reader.
5. Store a plugin result, an internal failure, a cancelled/deadline result, a
   time-limited incomplete result and a wall-time-bearing value.
6. Race two writers for one key and fill the store past its size limit.
7. Repeat for parse, check, package, execute and monitor operations.

Test location: planned cache integration tests.

## Expected Results

- An unchanged request hits. Changing manifest bytes or any other meaning
  input misses even when a recorded pin is unchanged.
- Changing only an excluded input does not invalidate a verified entry.
- A name, preimage, requested-key or typed-reader mismatch is a miss.
- Prohibited outcomes and plugin results are never stored.
- Concurrent writers publish identical complete bytes; eviction keeps the
  store at or below its configured size limit.
- Parse, check, package, execute and monitor run afresh on each request.
