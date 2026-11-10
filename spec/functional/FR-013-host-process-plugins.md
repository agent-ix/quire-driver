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
same process. The `quire-cli` process-provider adapter SHALL perform manifest
admission and typed result validation through callbacks supplied to the host.
This requirement specifies the host component; [FR-006](./FR-006-host-provider-plugins.md)
specifies the driver-owned process-provider seam and settlement composition.

## Inputs

- An executable path with at least one directory component, argument vector,
  caller-selected frame, session-byte and stderr-retention limits, and the
  caller's inherited environment and working directory.
- Manifest-admission and typed result-validation callbacks supplied by the
  `quire-cli` process-provider adapter.
- Zero or more provider requests, each with its request identity, package
  bytes, negotiated items, cancellation, deadline, and cancellation-grace
  limit.

## Outputs

- A session hello containing the plugin's protocol identity and manifest,
  returned before any provider request can be sent.
- For each accepted request, the validated typed result or its original typed
  protocol, plugin, process, cancellation, or timeout failure.
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
- When admission succeeds, the session SHALL accept zero or more requests and
  correlate each response to its request identity, validate each result
  through the `quire-cli` adapter's callback, and preserve the validated typed
  result unchanged.
- Each frame SHALL use the QSL PL-3 length-prefixed JCS wire format.
- The host SHALL enforce the per-frame limit before allocation and the shared
  session byte limit across both directions and all requests. Stderr retention
  is also capped for the whole session. Truncated, non-canonical, malformed,
  oversized, unexpected, or wrongly correlated frames SHALL retain their
  specific typed tool failure.
- The host SHALL make frame writes interruptible by caller cancellation and
  deadline. A deadline SHALL terminate the plugin process tree and settle as
  `TimedOut`; the host SHALL poll cancellation, deadlines and process exit at
  intervals no longer than 20 milliseconds. A blocked plugin
  pipe SHALL NOT delay that settlement beyond the polling interval plus the
  one-second cleanup bound.
- When cancellation is requested, the host SHALL send the protocol cancel
  frame and continue reading for the configured grace period. A valid,
  admitted and validated plugin settlement received within grace SHALL be
  returned unchanged. If no valid settlement arrives by grace expiry, the
  host SHALL terminate the process tree and settle as cancelled.
- The host SHALL preserve the specific typed cause of malformed frames,
  protocol or result-validation errors, plugin `fault`s, and non-zero exits
  during cancellation or timeout.
- The host SHALL terminate the process tree and drain and join its I/O workers
  within one second after termination begins for cancellation, timeout, or
  protocol failure. It SHALL preserve captured stderr in the returned
  diagnostic.
- The host SHALL enforce stderr retention limits while continuing to drain
  excess bytes.
- The host SHALL NOT parse or treat stderr as protocol data.
- The host SHALL avoid unbounded busy-wait while waiting for process exit.
- When graceful shutdown begins, the host SHALL send the shutdown frame and
  observe process exit within the configured grace.
- The merged host's Windows forced-cleanup path terminates the process tree
  through `terminate_tree`/`terminate_job`. It does not configure
  `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` through its standard-library Job Object
  path; kill-on-close remains planned under IR-674, and this specification
  makes no claim that automatic handle-close cleanup is implemented.
- When the child exits non-zero, the host SHALL return a process failure.
  EOF before the expected frame SHALL preserve an available
  process exit status in preference to a generic missing-frame error.
- When Windows process-tree setup fails before plugin execution, the host SHALL
  return a typed host-start failure, dispatch no provider request, and
  terminate any child already created.
- The process host SHALL remain a separate package.
- The `quire-driver` package SHALL NOT have a normal or build dependency on
  `quire-plugin-host`.
- The `quire-cli` package SHALL compose the host implementation with the
  driver's process-provider seam, as specified by FR-014.
- The process host SHALL NOT implement QSL manifest hashing, cache results, or
  map CG process backend variants; those responsibilities remain with their
  owning components.
- The repository test suite SHALL run `cargo fetch --locked` before its
  offline Cargo metadata dependency check.
- When the plugin fixture runtime is unavailable, the host test suite SHALL
  fail with an actionable message naming the missing runtime and how to
  install it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-013-AC-1 | Given a bare executable name, the host refuses to spawn it; given an explicit path, it launches only that executable with the supplied arguments and no shell. | Test |
| FR-013-AC-2 | Before any request, the session returns a valid canonical `hello`; a rejected protocol identity or manifest ends the session and writes no request frame. | Test |
| FR-013-AC-3 | After successful admission, a session accepts zero, one, or multiple requests over the same process, validates each response, and preserves each typed result and request identity. | Test |
| FR-013-AC-4 | An invalid typed result, wrong request identity, plugin fault, or non-zero exit is returned as its specific failure and is not converted to cancellation or timeout. | Test |
| FR-013-AC-5 | A plugin's valid typed cancellation settlement received within grace is returned unchanged; no settlement by grace expiry terminates the child and descendants and returns cancellation within the one-second cleanup bound. | Test |
| FR-013-AC-6 | A deadline expiring while a plugin does not read a request frame returns `TimedOut` within the 20-millisecond polling interval plus the one-second cleanup bound. | Test |
| FR-013-AC-7 | Truncated, malformed, non-canonical, oversized, unexpected, and wrongly correlated frames produce their specific typed failures without allocation above configured limits. | Test |
| FR-013-AC-9 | Stderr beyond the session retention limit is drained but not retained; stderr is available on failures, and reader/writer workers are joined within one second after termination begins. | Test |
| FR-013-AC-10 | After stdout closes while the child remains alive, process-exit waiting does not busy-wait; EOF before a response preserves an available child exit status. | Test |
| FR-013-AC-11 | The host uses QSL ADR-029 PL-1 through PL-7 without defining a local wire or manifest-digest format; `quire-driver` excludes the host crate, and `quire-cli` has a direct host dependency and composes it with the driver's seam. | Test (TC-005, TC-028) and analysis |
| FR-013-AC-12 | On a clean Cargo cache, the repository `make test` target fetches every locked platform dependency before running the offline TC-005 dependency metadata check, and the check passes. | Test (TC-027) |
| FR-013-AC-13 | The host test suite reports an actionable prerequisite failure naming the missing fixture runtime and its installation instructions. | Test (TC-027) |
| FR-013-AC-14 | When a supported Windows process-tree cleanup mechanism is selected under IR-674, the host assigns the plugin process tree to a kill-on-close Job Object before plugin code executes; closing the owning job handle terminates remaining job members. | Planned under IR-674; native Windows evidence required |
| FR-013-AC-15 | When Windows process-tree setup fails before plugin execution, the host returns a typed host-start failure, dispatches no provider request, and terminates any child already created. | Analysis/Inspection (review Windows host startup/error path in `quire-plugin-host`; runtime evidence requires a native Windows runner and an injectable or distinguishable setup-failure seam) |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 and PV-1 define the authoritative process
  protocol, session lifecycle, and admission boundary.
- [FR-006](./FR-006-host-provider-plugins.md) defines driver dispatch and
  settlement composition; [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md)
  verifies that composition.
- [TC-023](../test-cases/TC-023-plugin-session-and-admission.md) through
  [TC-027](../test-cases/TC-027-clean-cache-plugin-host-gate.md) verify the
  host-owned process behavior, resource limits, and clean-cache test gate.
