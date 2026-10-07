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
path; TC-028 verifies that `quire-cli` composes this seam with
`quire-plugin-host`.

Scope: FR-006-AC-1, FR-006-AC-2 and FR-006-AC-5.

## Test Procedure

Use a request whose provider negotiation assigns distinct items to one
process provider and one first-party provider. Configure the process provider
with an explicit fixture executable path and a hello manifest admitted by the
`quire-cli` adapter. Let each provider return a distinct typed result.
The separate host implementation's PL-1 through PL-7 conformance is verified by
[TC-023](./TC-023-plugin-session-and-admission.md) through
[TC-026](./TC-026-plugin-resource-bounds-and-cleanup.md); this case exercises
driver/provider composition only.

Test location: planned driver/provider composition tests.

Until CG IR-629 adds the process backend kind, run the adapter-level result
conversion checks with an explicit routed-item fixture. End-to-end negotiation
and settlement remain gated on that CG API.

## Expected Results

- The driver invokes its seam with the configured executable path, current
  package bytes and only the process provider's negotiated items. The CLI
  adapter starts `quire-plugin-host` and admits the hello manifest before
  sending the request.
- The returned typed result enters driver settlement unchanged, and unrelated
  first-party items retain their own results.
- A request `package_id` mismatch retains QSL's typed `content-mismatch`
  refusal. If QSL refuses the request body, every item routed to that process
  provider receives a typed failed result with its original request index and
  backend identity; unrelated provider results remain unchanged.
- The adapter's typed conversion covers the 13 Kani result rows in the
  QSpec vectors. Those terminal meanings are carried by QSL's typed terminal
  values; they are not inferred from diagnostics or Kani evidence.
