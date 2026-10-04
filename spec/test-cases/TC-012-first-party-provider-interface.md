---
id: TC-012
title: "Register and invoke a first-party provider through Provider"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-005
    type: verifies
---
# TC-012: Register and invoke a first-party provider through Provider

## Description

Verify that the driver's `Provider` trait supplies a provider manifest and
receives a typed routed request with its package, limits and cancellation
handle.

Scope: FR-005-AC-1 through FR-005-AC-3.

## Test Procedure

1. Implement a test provider whose manifest has a known backend identity and
   whose `run` method records its inputs and returns a typed result.
2. Register it with a second provider using a different identity.
3. Drive a request with items routed to each provider and inspect registered
   descriptors, recorded arguments, results and invocation counts.

Test location: planned driver provider tests.

## Expected Results

- Each manifest's identity becomes the corresponding registered backend
  identity.
- Each provider receives the request package, routed items, limits and
  cancellation handle as supplied; typed results pass through unchanged.
- Each provider runs only for its selected items, and distinct manifest
  identities remain distinct backends.
