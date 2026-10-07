---
id: FR-006
title: "Route negotiated items to process providers"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-014
    type: depends_on
---
# FR-006: Route negotiated items to process providers

## Description

When backend negotiation selects an out-of-process provider, the driver SHALL
dispatch only that provider's negotiated items through its driver-owned
process-provider seam, using the executable path named by the caller. The
`quire-cli` composition SHALL implement that seam with `quire-plugin-host`.
The driver SHALL feed returned typed provider results into the same proof
settlement path used for first-party providers.

## Inputs

- The caller-named plugin executable path.
- The admitted provider manifest, negotiated provider identity and routed
  request items.
- The emitted checked-package bytes and provider limits.
- The caller-owned QSL cancellation handle for the active run.
- The process-provider seam supplied by the `quire-cli` composition.

## Outputs

- The typed provider results returned for the routed items, consumed by the
  driver's proof settlement path.
- Bounded plugin process-tree cleanup before normal CLI termination or exit
  after handled SIGINT/SIGTERM on platforms covered by this requirement.

## Behavior

- The driver SHALL expose and invoke a process-provider seam that receives
  the named executable, admitted provider identity, emitted package bytes,
  negotiated items and caller limits. The driver package SHALL NOT depend on
  `quire-plugin-host`.
- The `quire-cli` composition SHALL implement the process-provider seam with
  `quire-plugin-host`, using QSL ADR-029 PL-1 through PL-7 as the authoritative
  contract; it SHALL add no plugin wire format, lifecycle or failure mapping.
- The `quire-cli` adapter SHALL start the host session with the caller-named
  executable path, pass the once-admitted descriptor returned by the host hello
  admitter to the driver's existing registry without re-admitting it or
  building a second registry, and send only the items negotiated to that
  provider.
- The driver SHALL pass the seam's validated typed results to the same
  settlement path used for first-party provider results.
- The `quire-cli` adapter SHALL request graceful shutdown of every active
  plugin session before normal command termination. On Linux and macOS, if
  graceful shutdown exceeds `RunLimits.cancellation_grace`, it SHALL terminate
  the process tree and await cleanup within the one-second bound in FR-013.
  Windows forced process-tree cleanup is planned under IR-674 and specified by
  FR-013-AC-14; this requirement does not claim the merged host provides it.
- When `quire-cli` on Linux or macOS receives SIGINT or SIGTERM while a plugin
  session is starting, running or shutting down, it SHALL request cancellation
  through the caller-owned QSL handle, allow `RunLimits.cancellation_grace` to
  expire or the session to settle, terminate any remaining
  plugin process tree, and await cleanup within the one-second bound in
  FR-013 before exiting. The CLI SHALL then exit with the conventional
  signal-terminated process status; this requirement makes no claim about a
  QSL cancellation cause for the signal. If no plugin session is active, the
  CLI SHALL retain the platform's default signal disposition. This guarantee
  does not cover an uncatchable host kill on Unix or a plugin descendant that
  deliberately leaves the host's process group.
- Windows console control events, including Ctrl+C and console close, retain
  the platform's default disposition; this requirement makes no plugin cleanup
  guarantee for those events. Windows process-tree cleanup, if supported by
  the host, is specified only by [FR-013-AC-14](./FR-013-host-process-plugins.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | When negotiation selects a process provider, the driver invokes its process-provider seam with the caller-named executable path, current emitted package bytes and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry. | Test (TC-013, TC-028) |
| FR-006-AC-2 | Validated typed results returned through the process-provider seam enter the same driver proof settlement path as first-party provider results, while unrelated routed items retain their own provider results. | Test (TC-013) |
| FR-006-AC-3 | Before normal CLI command termination, the adapter requests graceful shutdown of every active plugin session; on Linux and macOS, if `RunLimits.cancellation_grace` expires first, the process tree terminates and cleanup completes within one second. Windows forced process-tree cleanup is planned under IR-674 and is not claimed here. | Test (TC-028) |
| FR-006-AC-4 | On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, cleans up the plugin process tree before CLI exit, and returns the conventional signal-terminated process status without claiming a QSL cause; with no active session, the platform's default signal disposition is retained. | Test (TC-028) |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 define the plugin protocol, host lifecycle,
  manifest admission and typed failure behavior; PV-1 through PV-4 define
  provider registration and negotiation. [FR-013](./FR-013-host-process-plugins.md)
  specifies the separate host implementation. FR-014 specifies its
  composition by the `quire-cli` package.
- FR-005 defines the first-party `Provider` interface and shared manifest
  admission.
- CG IR-629 supplies the process `BackendKind` mapping required before the
  process-provider invocation in TC-013/TC-028 can execute. This FR does not
  define or infer that mapping.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
