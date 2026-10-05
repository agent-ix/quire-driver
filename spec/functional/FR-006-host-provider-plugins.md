---
id: FR-006
title: "Route negotiated items to process providers"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-006: Route negotiated items to process providers

## Description

When backend negotiation selects an out-of-process provider, the driver SHALL
dispatch only that provider's negotiated items to the `quire-plugin-host`
session, using the executable path named by the caller. The driver SHALL feed
the returned typed provider results into the same proof settlement path used
for first-party providers.

## Inputs

- The caller-named plugin executable path.
- The admitted provider manifest, negotiated provider identity and routed
  request items.
- The emitted checked-package bytes and provider limits.

## Outputs

- The typed provider results returned for the routed items, consumed by the
  driver's proof settlement path.

## Behavior

- The driver SHALL use QSL ADR-029 PL-1 through PL-7 as the authoritative
  plugin-host contract implemented by `quire-plugin-host`; it SHALL add no
  plugin wire format, lifecycle or failure mapping.
- The driver SHALL start the host session with the caller-named executable
  path, admit its hello manifest through the shared manifest boundary, and
  send the emitted package bytes and only the items negotiated to that
  provider.
- The driver SHALL pass the host's validated typed results to the same
  settlement path used for first-party provider results.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | When negotiation selects a process provider, the driver starts `quire-plugin-host` at the caller-named executable path, admits the hello manifest, and sends the current emitted package bytes and exactly that provider's negotiated items. | Test |
| FR-006-AC-2 | Validated typed results returned by `quire-plugin-host` enter the same driver proof settlement path as first-party provider results, while unrelated routed items retain their own provider results. | Test |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 define the plugin protocol, host lifecycle,
  manifest admission and typed failure behavior; PV-1 through PV-4 define
  provider registration and negotiation. [FR-013](./FR-013-host-process-plugins.md)
  specifies the in-repository host implementation.
- FR-005 defines the first-party `Provider` interface and shared manifest
  admission.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
