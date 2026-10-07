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
  - target: ix://agent-ix/quire-specification/FR-331
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-069
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
  lifecycle and failure contract. It SHALL use FR-331's provider envelope and
  QSL-owned typed terminal values; it SHALL NOT define a second wire format or
  infer terminal categories from provider-specific evidence or diagnostics.
- The `quire-cli` adapter SHALL start the host session with the caller-named
  executable path, pass the once-admitted descriptor returned by the host hello
  admitter to the driver's existing registry without re-admitting it or
  building a second registry, and send only the items negotiated to that
  provider.
- The driver SHALL retain the losslessly decoded FR-331 provider-manifest
  descriptors when they are admitted and SHALL compose each request's
  `manifest` registry snapshot from those descriptors. It SHALL NOT reconstruct
  the snapshot from QSL registry descriptors, which do not retain optional
  FR-331 fields.
- The driver SHALL build each request's `package` from the emitted package's
  exact bytes and its `package_id` from that same `EmittedPackage` value.
- The driver SHALL copy the `deadline_ms` supplied in the caller's
  process-provider request into the FR-331 request. It SHALL NOT add
  request-level `domains` or `limits`.
- The driver SHALL source each serialized item from CG's routed output and
  routing request: request index, claim locus and kind, extent/bounds, selected
  backend and candidate set. Temporal proof bounds SHALL appear under
  `items[].temporal.subject.proof_bounds`. The current CG routed output does
  not retain all these fields; their projection is a named integration gap for
  IR-629, and bounds `kind` additionally depends on QSL-654.
- The driver SHALL pass a validated plugin result to the shared settlement
  path only when QSL supplies a terminal mapping for it. A result without a
  currently supported terminal mapping SHALL remain typed, unmapped provider
  data; it SHALL NOT be assigned a fabricated terminal.
- The driver SHALL map a FR-331 `declined` result to QSL
  `TerminalValue::Declined { code }` using its typed code, and SHALL keep any
  response without a specified QSL terminal mapping typed and unmapped.
- The driver SHALL put the FR-272 diagnostic code and available locus and
  message into a QSL `OutcomeDiagnostic` in the outcome document. It SHALL NOT
  parse free-form message or `detail` text to choose a terminal; terminal
  mappings must be defined by QSL.
- When the plugin rejects a request because `package_id` does not match the
  decoded package bytes, or because the package spelling is non-canonical, the
  driver SHALL decode the plugin's FR-331 response, preserve each routed item
  identity, and map each failed item to QSL `TerminalValue::Failed`. The
  corresponding typed FR-272 diagnostic is carried separately in the outcome
  document. Results for unrelated providers SHALL remain unchanged.
- The driver SHALL synthesize provider failures only for host/protocol failures
  defined by ADR-029 PL-7; it SHALL NOT attribute a request-body refusal to
  QSL, which has no process request-body reader.
- The `quire-cli` adapter SHALL request graceful shutdown of every active
  plugin session before normal command termination. On Linux and macOS, if
  graceful shutdown exceeds `RunLimits.cancellation_grace`, it SHALL terminate
  the process tree and await cleanup within the one-second bound in FR-013.
  On Windows, the merged host's forced cleanup path terminates the Job Object
  through `terminate_tree`/`terminate_job`; native Windows evidence remains
  planned. The separate Job Object kill-on-close mechanism is planned under
  IR-674 and specified by FR-013-AC-14.
- When `quire-cli` on Linux or macOS receives SIGINT or SIGTERM while a plugin
  session is starting, running or shutting down, it SHALL request cancellation
  through the caller-owned QSL handle, allow `RunLimits.cancellation_grace` to
  expire or the session to settle, terminate any remaining
  plugin process tree, and await cleanup within the one-second bound in
  FR-013 before exiting. The CLI SHALL restore the default signal disposition
  and re-raise the received signal; this requirement makes no claim about a
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
| FR-006-AC-1 | When negotiation selects a process provider, the driver invokes its process-provider seam with the caller-named executable path, emitted package bytes and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry. | Planned (TC-013, TC-028; process invocation awaits IR-629) |
| FR-006-AC-2 | Validated typed results returned through the process-provider seam enter the same driver proof settlement path as first-party provider results when QSL defines their terminal mapping; results without a current QSL mapping remain typed and unmapped, while unrelated routed items retain their own results. | Planned (TC-013; IR-629, IR-677, QSL-490 and QSL-655) |
| FR-006-AC-3 | Before normal CLI command termination, the adapter requests graceful shutdown of every active plugin session; on Linux and macOS, if `RunLimits.cancellation_grace` expires first, the process tree terminates and cleanup completes within one second. The merged Windows host force-terminates the Job Object through `terminate_tree`/`terminate_job`; native Windows evidence remains planned. Kill-on-close is separately planned under IR-674. | Test (TC-028) |
| FR-006-AC-4 | On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal. No QSL cancellation cause is claimed; with no active session, the platform's default signal disposition is retained. | Test (TC-028) |
| FR-006-AC-5 | The request's manifest snapshot is composed from retained, losslessly decoded FR-331 hello DTOs; the adapter does not reconstruct optional FR-331 fields from QSL registry descriptors. | Planned (TC-013) |
| FR-006-AC-6 | The serialized `package` is the unpadded base64url encoding of the emitted package's exact bytes. | Planned (TC-013) |
| FR-006-AC-7 | The serialized `package_id` comes from the same `EmittedPackage` value as the encoded package bytes. | Planned (TC-013) |
| FR-006-AC-8 | The serialized request's `deadline_ms` equals the caller-supplied process-provider deadline. | Planned (TC-013) |
| FR-006-AC-9 | The serialized request has no top-level `domains` or `limits` member. | Planned (TC-013) |
| FR-006-AC-10 | Each serialized item preserves its request index, claim locus and kind from the CG routed output. | Planned (TC-013; full source projection awaits IR-629) |
| FR-006-AC-11 | Each serialized item preserves its selected backend and candidate set from the CG routed output. | Planned (TC-013; full source projection awaits IR-629) |
| FR-006-AC-12 | Each serialized item preserves extent/bounds, and temporal proof bounds appear under `items[].temporal.subject.proof_bounds`. | Planned (TC-013; full source projection awaits IR-629 and QSL-654) |
| FR-006-AC-13 | A plugin `declined` result's typed code is preserved in QSL `TerminalValue::Declined { code }`; a result without a specified QSL terminal mapping remains typed and unmapped. | Planned (TC-013) |
| FR-006-AC-14 | The FR-272 diagnostic code and available locus and message are carried in a QSL `OutcomeDiagnostic` in the outcome document; free-form message or `detail` text does not select a terminal. | Planned (TC-013) |
| FR-006-AC-15 | Given a plugin response marking every routed item failed with FR-272 `stale_dependency/content-mismatch`, the driver maps each item to QSL `TerminalValue::Failed` and preserves its identity. | Planned (TC-013) |
| FR-006-AC-16 | Given a plugin response marking every routed item failed with FR-272 `invalid_runtime_input/invalid-value` for a non-canonical package spelling, the driver maps each item to QSL `TerminalValue::Failed` and preserves its identity. | Planned (TC-013) |
| FR-006-AC-17 | The driver synthesizes failed provider results only for host/protocol failures defined by ADR-029 PL-7; it does not attribute a process request-body refusal to QSL. | Planned (TC-013) |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 define the plugin protocol, host lifecycle,
  manifest admission and typed failure behavior; PV-1 through PV-4 define
  provider registration and negotiation. [FR-013](./FR-013-host-process-plugins.md)
  specifies the separate host implementation. FR-014 specifies its
  composition by the `quire-cli` package.
- [FR-331](ix://agent-ix/quire-specification/FR-331) owns the
  `quire.backend-provider/v1` envelope; QSL owns typed terminal values and
  `quire-outcome/1` serialization.
- QSL-654 adds the `kind` member to proof bounds; the driver does not add that
  field until QSpec/QSL publishes it. CG's separate `bounds[].kind` work is
  outside this request adapter.
- QSL-490 and QSL-655 are prerequisites for a plugin `proved` result's trusted
  `ProofBasis` and for replay settlement of plugin counterexample candidates.
  Until those APIs land, these result forms are not mapped to terminals.
- FR-005 defines the first-party `Provider` interface and shared manifest
  admission.
- CG IR-629 specifies the future process-origin projection under CG FR-019 and
  FR-022. Until its code lands, the driver does not construct or route a
  `BackendKind::Process`; the driver passes QSL's origin through unchanged.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
