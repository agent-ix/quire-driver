---
id: FR-006
title: "Host out-of-process backend providers"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-006: Host out-of-process backend providers

## Description

When a request names an out-of-process provider, the driver SHALL run that
provider through the plugin host boundary in QSL ADR-029 PL-1 to PL-7. The
driver SHALL treat every plugin frame as untrusted input. The driver SHALL
settle only the items routed to that plugin from its typed result.

## Inputs

- The plugin executable path explicitly named by the caller.
- Its registered `BackendId`, admitted provider manifest, routed request,
  emitted checked-package bytes, limits and cancellation handle.

## Outputs

- Typed provider results for the routed items, or typed terminal failures for
  the affected items.

## Behavior

- A plugin SHALL be a separately launched executable speaking
  `quire.plugin-wire/v1`.
- The host SHALL NOT load a plugin as a dynamic library or search for it on
  the caller's behalf.
- The first plugin message SHALL be a `hello` envelope carrying its FR-331
  provider manifest.
- The host SHALL derive the plugin backend identity from the manifest's
  identity string and admit the manifest through the same registry conversion
  used for first-party providers.
- Each frame SHALL contain an unsigned 64-bit big-endian length and one JCS
  JSON envelope.
- The host SHALL reject non-canonical JSON, an unknown protocol identity, a
  mismatched request id, an oversized frame or a run exceeding its total byte
  budget.
- The host SHALL send routed FR-331 requests with the emitted v2 package
  bytes and provider limits.
- When the caller cancels, the host SHALL send `cancel`.
- When the caller's grace period expires, the host SHALL stop the plugin
  process.
- If a malformed reply, crash, non-zero process exit, missing terminal
  record, unrouted-item result or unadvertised-kind result occurs, then the
  host SHALL settle each affected item as `failed` with a typed tool-failure
  cause.
- When a timeout or cancellation leaves an item open, the host SHALL settle
  that item as `incomplete` with its cause.
- The host SHALL leave items routed to another provider unaffected by one
  plugin's result or failure.
- When a plugin returns a counterexample, the driver SHALL settle it
  `refuted` only after QSL replay reproduces it.
- When a plugin returns `proved`, the driver SHALL retain its plugin identity
  and the `trusted` basis label.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | A named plugin starts only at the caller-supplied executable path; the host performs no discovery, search-path lookup or dynamic-library loading. | Test |
| FR-006-AC-2 | The host accepts a canonical `hello` frame for `quire.plugin-wire/v1`, derives the backend identity from the manifest, and submits it through the same registry conversion as a first-party provider. | Test |
| FR-006-AC-3 | The host rejects a non-canonical frame, wrong protocol identity, wrong request id, frame over the configured frame limit or run over its total byte limit before accepting a result. | Test |
| FR-006-AC-4 | On cancellation the host sends `cancel`; when the grace period expires it terminates the process and settles each still-open routed item as `incomplete` with the cancellation or timeout cause. | Test |
| FR-006-AC-5 | A crash, non-zero exit, malformed result, missing terminal record, unrouted-item result or unadvertised-kind result settles only the items routed to that plugin as `failed`; other providers' items are unchanged. | Test |
| FR-006-AC-6 | A plugin counterexample is `refuted` only when QSL replay reproduces it, and a plugin `proved` record identifies the plugin and carries basis `trusted`. | Test |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 and PV-1 through PV-4.
- FR-005 defines the first-party `Provider` interface and shared manifest
  admission.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
