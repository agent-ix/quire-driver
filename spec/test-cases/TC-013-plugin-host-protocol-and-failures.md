---
id: TC-013
title: "Dispatch process-provider results through the driver"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-006
    type: verifies
---
# TC-013: Dispatch process-provider results through the driver

## Description

Verify that the driver passes negotiated process-provider work to QSL's
plugin host and returns the typed provider results through its proof path.

Scope: FR-006-AC-1 through FR-006-AC-2.

## Test Procedure

Use a request whose provider negotiation assigns distinct items to one
process provider and one first-party provider. Configure the process provider
with an explicit fixture executable path and an admitted manifest. Let each
provider return a distinct typed result. The plugin fixture shall implement
QSL ADR-029 PL-1 through PL-7; protocol-conformance cases are covered by
QSL's plugin-host tests.

Test location: planned driver/provider composition tests.

## Expected Results

- The driver gives the QSL plugin host the configured executable path,
  admitted manifest, package bytes from the current request and only the
  process provider's negotiated items.
- The host's typed result reaches the driver's proof settlement input, and
  the first-party provider's result remains associated with its own items.
- Plugin frame validation, lifecycle and typed failure behavior match the
  authoritative QSL ADR-029 PL-1 through PL-7 contract.
