---
id: US-002
title: "Run a runtime monitor from QSL source"
type: US
relationships:
  - target: ix://agent-ix/quire-driver/FR-020
    type: exercises
---
# US-002: Run a runtime monitor from QSL source

## Story

**As a** QSL user monitoring a time-sensitive claim
**I want** to run the monitor defined by my QSL source against incoming events
**So that** I can observe its runtime results as the events arrive.

## Context

The driver already prepares the runtime monitor through library calls. The CLI
does not yet expose that path. QSL's generate target includes runtime monitors;
QSL and RT own their respective source and runtime semantics.

## Acceptance Examples (Illustrative)

### US-002-EX-1: Monitor incoming events

- **Given** a source request selecting one runtime monitor and an event stream
- **When** the user runs `quire generate` and supplies events
- **Then** each event is observed once, in input order, with its runtime result

### US-002-EX-2: Stop at malformed input

- **Given** a valid event followed by a malformed event and another valid event
- **When** the event stream is consumed
- **Then** the malformed event is reported and the later event is not consumed
