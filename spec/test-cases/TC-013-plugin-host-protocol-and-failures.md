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

Verify that the driver passes negotiated process-provider work through its
process-provider seam and returns typed provider results through its proof
path; TC-028 verifies that `quire-cli` composes this seam with the QSL host.

Scope: FR-006-AC-1 through FR-006-AC-2.

## Test Procedure

Use a request whose provider negotiation assigns distinct items to one
process provider and one first-party provider. Configure the process provider
with an explicit fixture executable path and a hello manifest accepted by the
driver's admission callback. Let each provider return a distinct typed result.
The separate host implementation's PL-1 through PL-7 conformance is verified by
[TC-023](./TC-023-plugin-session-and-admission.md) through
[TC-026](./TC-026-plugin-resource-bounds-and-cleanup.md); this case exercises
driver/provider composition only.

Test location: planned driver/provider composition tests.

## Expected Results

- The driver invokes its seam with the configured executable path, current
  package bytes and only the process provider's negotiated items. The CLI
  adapter starts `quire-plugin-host` and admits the hello manifest before
  sending the request.
- The host's typed result reaches the driver's proof settlement input, and
  the first-party provider's result remains associated with its own items.
- The returned typed result enters driver settlement unchanged, and unrelated
  first-party items retain their own results.
