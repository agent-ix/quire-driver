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
  after handled SIGINT/SIGTERM.

## Behavior

- The driver SHALL expose and invoke a process-provider seam that receives
  the named executable, admitted provider identity, emitted package bytes,
  negotiated items and caller limits. The driver package SHALL NOT depend on
  `quire-plugin-host`.
- The `quire-cli` composition SHALL implement the process-provider seam with
  `quire-plugin-host`, using QSL ADR-029 PL-1 through PL-7 as the authoritative
  contract; it SHALL add no plugin wire format, lifecycle or failure mapping.
- The `quire-cli` adapter SHALL start the host session with the caller-named
  executable path, admit its hello manifest through the shared manifest
  boundary, and send only the items negotiated to that provider.
- The driver SHALL pass the seam's validated typed results to the same
  settlement path used for first-party provider results.
- The `quire-cli` adapter SHALL close every active plugin session before
  normal command termination. If graceful session shutdown exceeds the QSL
  cancellation grace, it SHALL terminate the process tree and await cleanup
  within the one-second bound in FR-013.
- When `quire-cli` receives SIGINT or SIGTERM while a plugin session is
  starting, running or shutting down, it SHALL request cancellation through
  the caller-owned QSL handle,
  allow the QSL cancellation grace to complete, terminate any remaining
  plugin process tree, and await cleanup within the one-second bound in
  FR-013 before exiting. This guarantee covers handled signals and normal
  command termination; it does not cover an uncatchable host kill on Unix or
  a plugin descendant that deliberately leaves the host's process group.
- When `quire-cli` runs on Windows, it SHALL close each active plugin session
  before CLI termination. The host SHALL use the session's owning Job Object
  handle configured with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`; closing that
  handle SHALL terminate remaining job members.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-006-AC-1 | When negotiation selects a process provider, the driver invokes its process-provider seam with the caller-named executable path, current emitted package bytes and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and admits the hello manifest. | Test (TC-013, TC-028) |
| FR-006-AC-2 | Validated typed results returned through the process-provider seam enter the same driver proof settlement path as first-party provider results, while unrelated routed items retain their own provider results. | Test (TC-013) |
| FR-006-AC-3 | Before normal CLI command termination, every active plugin session is shut down; if graceful shutdown expires, its process tree terminates and cleanup completes within one second. | Test (TC-028) |
| FR-006-AC-4 | Handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests QSL cancellation and cleans up the plugin process tree before CLI exit; forced termination cleanup completes within one second after it begins. | Test (TC-028) |
| FR-006-AC-5 | When `quire-cli` terminates on Windows with a plugin session active, it closes the session's owning Job Object handle; the host configures `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` so remaining job members terminate. | Test (TC-028) |

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
