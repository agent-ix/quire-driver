---
id: TC-024
title: "Enforce plugin framing and preserve typed failures"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-013
    type: verifies
---
# TC-024: Enforce plugin framing and preserve typed failures

## Description

Verify the host applies QSL PL-3 frame rules and reports the original protocol,
plugin, validation, and process failures.

Scope: FR-013-AC-4 and FR-013-AC-7.

## Test Procedure

Use deterministic fixtures that independently send a truncated length word, a
truncated body, malformed JSON, non-canonical JCS, an oversized frame, an
unexpected frame, a wrong request identity, an invalid typed result, a `fault`,
and a non-zero exit. Include one canonical valid exchange as a control. Also
send multiple individually valid frames whose combined request-and-response
bytes exceed the session limit. Record the process exit status and host error
for each case.

## Expected Results

- The canonical exchange succeeds; every invalid exchange returns its specific
  typed framing, protocol, validation, plugin, or process failure.
- No failure is converted to cancellation or timeout, and no frame allocates
  beyond its configured bound. The cumulative limit counts both directions
  across the whole session, including every request and response.
- Mutation checks that remove canonical re-encoding, frame-size enforcement,
  result validation, or process-exit validation each cause this test to fail.
