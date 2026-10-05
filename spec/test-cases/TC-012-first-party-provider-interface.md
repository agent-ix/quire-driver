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

Scope: FR-005-AC-1 through FR-005-AC-5.

## Test Procedure

1. Implement a test provider whose manifest has a known backend identity and
   whose `run` method records its inputs and returns a typed result.
2. Register it with a second provider using a different identity.
3. Drive a request with items routed to each provider and inspect registered
   descriptors, recorded arguments, results and invocation counts.
4. After QSL-637 lands, verify the QSL registry builder assigns `Linked` to
   compile-time provider descriptors and `Process` to plugin hello descriptors.
   Register the same descriptor twice and verify idempotence. Then register
   `Linked` and `Process` descriptors with the same `BackendId` but different
   descriptor content; inspect registry refusals, withdrawn identities and
   provider call counts.
5. Once CG IR-629 lands, pass the registry-built descriptors unchanged to CG
   negotiation and verify CG classifies the backend from the origin field.

Test location: planned driver provider tests.

## Expected Results

- Each manifest's identity becomes the corresponding registered backend
  identity.
- Each provider receives the request package, routed items, limits and
  cancellation handle as supplied; typed results pass through unchanged.
- Each provider runs only for its selected items, and distinct manifest
  identities remain distinct backends.
- `qsl_route::Registry` treats an equal descriptor repeat as idempotent.
  Registering an unequal descriptor, including a different origin, for one `BackendId` returns the typed
  duplicate-backend refusal, withdraws the identity, declines its items and
  invokes neither provider.
- The QSL registry builder assigns `Process` to plugin descriptors; the driver
  passes them to CG unchanged and does not construct CG process kinds. This
  step is gated on QSL-637 and CG IR-629.
