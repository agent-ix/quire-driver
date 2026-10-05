---
id: FR-013
title: "Host process providers through a bounded session"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-driver/FR-006
    type: references
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
---
# FR-013: Host process providers through a bounded session

## Description

The `quire-plugin-host` package SHALL implement the process boundary required
by QSL ADR-029 PL-1 through PL-7. It SHALL expose a session that starts a
named executable, reads and returns its `hello` manifest before accepting any
provider request, permits zero or more validated requests, and shuts down the
same process. The caller SHALL perform manifest admission and typed result
validation through the host's supplied validation callbacks. This requirement
specifies the host component; [FR-006](./FR-006-host-provider-plugins.md)
specifies driver dispatch and settlement composition.

## Inputs

- An executable path with at least one directory component, argument vector,
  caller-selected frame and total-byte limits, and the caller's inherited
  environment and working directory.
- A caller-provided manifest-admission callback and typed result-validation
  callback.
- Zero or more provider requests, each with its request identity, package
  bytes, negotiated items, and caller-selected cancellation, deadline, memory,
  CPU-time, and cancellation-grace limits.

## Outputs

- A session hello containing the plugin's protocol identity and manifest,
  returned before any provider request can be sent.
- For each accepted request, the validated typed result or its original typed
  protocol, plugin, process, budget, cancellation, or timeout failure.
- Bounded opaque stderr bytes alongside success and failure diagnostics.

## Behavior

- The host SHALL spawn only the executable named by the caller, with the
  supplied arguments and no shell.
- When the executable value is a bare name, the host SHALL refuse to spawn it
  instead of searching `PATH`.
- The host SHALL read and validate one canonical JCS `hello` frame before
  exposing the session. The caller's manifest-admission callback SHALL accept
  the manifest before the session can send a request.
- When manifest admission rejects the hello manifest, the host SHALL write no
  request frame.
- When admission succeeds, the session SHALL accept zero or more requests and correlate
  each response to its request identity, validate each result through the
  caller's callback, and preserve the validated typed result unchanged.
- Each frame SHALL use the QSL PL-3 length-prefixed JCS wire format.
- The host SHALL enforce the per-frame limit before allocation and the shared
  per-run byte limit across both directions. Truncated, non-canonical, malformed,
  oversized, unexpected, or wrongly correlated frames SHALL retain their
  specific typed tool failure.
- The host SHALL make frame writes interruptible by caller cancellation and
  deadline. A deadline SHALL terminate the plugin process tree and settle as
  `TimedOut`; a blocked plugin pipe SHALL NOT delay that settlement beyond the
  configured termination grace plus the documented polling interval.
- When cancellation is requested, the host SHALL send the protocol cancel
  frame and continue reading for the configured grace period. A valid,
  admitted and validated plugin settlement received within grace SHALL be
  returned unchanged. If no valid settlement arrives by grace expiry, the
  host SHALL terminate the process tree and settle as cancelled.
- Cancellation or timeout SHALL NOT remap a malformed frame, a protocol or
  result-validation error, a plugin `fault`, a process-budget breach, or a
  non-zero exit. Those failures SHALL retain their specific typed cause.
- Memory and CPU-time budgets SHALL account for the plugin and its descendant
  processes. On cancellation, timeout, protocol failure, or budget breach,
  the host SHALL terminate the process tree, drain and join its I/O workers
  within a bounded cleanup interval, and preserve captured stderr in the
  returned diagnostic.
- The host SHALL enforce stderr retention limits while continuing to drain
  excess bytes.
- The host SHALL NOT parse or treat stderr as protocol data.
- The host SHALL avoid unbounded busy-wait while waiting for process exit.
- When graceful shutdown begins, the host SHALL send the shutdown frame and
  observe process exit within the configured grace.
- When the child exits non-zero, the host SHALL return a process failure.
  EOF before the expected frame SHALL preserve an available
  process exit status in preference to a generic missing-frame error.
- The process host SHALL remain a separate package.
- The `quire-driver` package SHALL NOT have a normal or build dependency on
  `quire-plugin-host`.
- The process host SHALL NOT implement QSL manifest hashing, cache results, or
  map CG process backend variants; those responsibilities remain with their
  owning components.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-013-AC-1 | Given a bare executable name, the host refuses to spawn it; given an explicit path, it launches only that executable with the supplied arguments and no shell. | Test |
| FR-013-AC-2 | Before any request, the session returns a valid canonical `hello`; a rejected protocol identity or manifest ends the session and writes no request frame. | Test |
| FR-013-AC-3 | After successful admission, a session accepts zero, one, or multiple requests over the same process, validates each response, and preserves each typed result and request identity. | Test |
| FR-013-AC-4 | An invalid typed result, wrong request identity, plugin fault, or non-zero exit is returned as its specific failure and is not converted to cancellation or timeout. | Test |
| FR-013-AC-5 | A plugin's valid typed cancellation settlement received within grace is returned unchanged; no settlement by grace expiry terminates the child and descendants and returns cancellation. | Test |
| FR-013-AC-6 | A deadline expiring while a plugin does not read a request frame returns `TimedOut` within the configured termination grace plus the documented polling interval. | Test |
| FR-013-AC-7 | Truncated, malformed, non-canonical, oversized, unexpected, and wrongly correlated frames produce their specific typed failures without allocation above configured limits. | Test |
| FR-013-AC-8 | Memory and CPU-time budget breaches account for the plugin process tree, terminate its descendants, and report the corresponding budget failure. | Test |
| FR-013-AC-9 | Stderr beyond the configured retention limit is drained but not retained; stderr is available on failures, and reader/writer workers are joined within the cleanup bound. | Test |
| FR-013-AC-10 | After stdout closes while the child remains alive, process-exit waiting does not busy-wait; EOF before a response preserves an available child exit status. | Test |
| FR-013-AC-11 | The host uses QSL ADR-029 PL-1 through PL-7 without defining a local wire or manifest-digest format, and the driver dependency closure excludes the host crate. | Analysis |
| FR-013-AC-12 | On a clean Cargo cache, `make test` fetches every locked platform dependency before running the offline TC-005 dependency metadata check, and the check passes. | Test |
| FR-013-AC-13 | The host test suite reports an actionable prerequisite failure when its required fixture runtime is absent. | Test |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 and PV-1 define the authoritative process
  protocol, session lifecycle, and admission boundary.
- [FR-006](./FR-006-host-provider-plugins.md) defines driver dispatch and
  settlement composition; [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md)
  verifies that composition.
- [TC-023](../test-cases/TC-023-plugin-session-and-admission.md) through
  [TC-027](../test-cases/TC-027-clean-cache-plugin-host-gate.md) verify the
  host-owned process behavior, resource limits, and clean-cache test gate.
