---
id: US-001
title: "Run one checked request from source to routed backends"
type: US
relationships:
  - target: ix://agent-ix/quire-driver/FR-001
    type: exercises
---
# US-001: Run one checked request from source to routed backends

## Story

**As a** QSL user proving a property of a checked package
**I want** one call that compiles my source, negotiates each requested item
with the registered backends and routes the supported ones
**So that** each item reaches the backend that can discharge it, or gets a
typed reason why none can.

## Context

QSL compiles and routes; CG negotiates (and will generate). Neither may call the
other in that direction, so the driver calls both (QSL ADR-011 §6.1 driver
row, T-13).

## Acceptance Examples (Illustrative)

### US-001-EX-1

A unit compiles, Kani is registered for `value-validity`, and three items
are requested: one `value-validity` item, one `operation-contract` item
and one naming an unregistered backend. The first is routed to `kani`, the
second settles `unsupported` and the third `invalid-request`.

### US-001-EX-2

The registry holds a backend CG has no kind for. The run is refused with
`invalid_capability`/`unknown-backend` before any item is negotiated.
