---
id: TC-023
title: "Start a plugin session and admit hello before requests"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
---
# TC-023: Start a plugin session and admit hello before requests

## Description

Verify explicit executable selection and the session ordering from process
start through hello admission, zero or more requests, and shutdown.

Scope: FR-013-AC-1 through FR-013-AC-3 and FR-013-AC-13.

## Test Procedure

Use deterministic fixture executables that record received frames and expose
their hello manifest. First invoke the host with a bare executable name, then
with an explicit fixture path. For the explicit path, test a rejected protocol,
a rejected manifest, and an admitted manifest with zero, one, and two
sequential requests. Have the admitted fixture return distinct validated
results for each request and exit on shutdown. Run the test with its fixture
runtime removed from `PATH`.

## Expected Results

- The bare name is refused without spawning a PATH-resolved executable; the
  explicit path is the process that starts.
- Hello is the first frame observed. Protocol or manifest rejection sends no
  request. Admitted sessions permit zero or multiple requests on that process,
  and preserve each result with its request identity.
- If the fixture runtime is unavailable, the test fails before spawning with a
  message naming the missing executable and installation prerequisite.
- Mutation checks that remove hello admission, request identity checking, or
  result validation each cause this test to fail.
