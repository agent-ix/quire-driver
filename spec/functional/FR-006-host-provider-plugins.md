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
  The CG routed request SHALL carry the caller's `deadline_ms` and every item
  field into the serializer. The CG routed output/routing seam supplies each
  request index, claim locus and requested kind, extent, selected backend and
  candidate set; bounds are carried at `items[].extent.bounds`, and temporal bounds at
  `items[].temporal.subject.proof_bounds`. No request-level domains or limits
  are added. The current CG routed output does not retain all these request
  fields; its projection is a named integration gap for IR-629 and QSL-654.
- The driver SHALL pass a validated plugin result to the shared settlement
  path only when QSL supplies a terminal mapping for it. A result without a
  currently supported terminal mapping SHALL remain typed, unmapped provider
  data; it SHALL NOT be assigned a fabricated terminal.
- The driver SHALL decode typed FR-331 result fields and typed FR-272
  diagnostic fields. For a `declined` result it SHALL use the typed code in
  QSL `TerminalValue::Declined { code }`; for `unsupported` with
  `tool-unavailable` and absent detail, it SHALL use QSL FR-069's typed
  `UnavailabilityCause` mapping. It SHALL preserve the FR-272 diagnostic as an
  `OutcomeDiagnostic` and SHALL NOT parse free-form diagnostic message or
  detail text to choose a terminal.
- When a plugin cannot accept a request package because the `package_id` does
  not match its decoded package bytes, it SHALL return one FR-331 result
  marking every routed item failed with the FR-462
  `stale_dependency/content-mismatch` diagnostic. When the plugin rejects an
  item value, it SHALL return one FR-331 result marking every routed item
  failed with the FR-462 `invalid_runtime_input/invalid-value` diagnostic.
  The driver SHALL read these plugin results, preserve each request index and
  backend identity, and map
  each failed result to QSL `TerminalValue::Failed`; it SHALL carry the typed
  FR-272 diagnostic as an `OutcomeDiagnostic`, separate from the terminal.
  Results for unrelated providers SHALL remain unchanged.
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
| FR-006-AC-1 | When CG negotiation selects a process provider, the driver invokes its process-provider seam with the caller-named executable and only that provider's negotiated items. | Planned (TC-013; IR-629) |
| FR-006-AC-2 | The `quire-cli` adapter starts one host session and sends its hello manifest through QSL descriptor admission exactly once with caller-supplied `Process` origin, then registers that descriptor in the existing registry without re-admission. | Test (TC-013, TC-028) |
| FR-006-AC-3 | The request serializer takes package bytes and `package_id` from the same `EmittedPackage` and emits the package as unpadded base64url. | Test (TC-013) |
| FR-006-AC-4 | The request serializer emits one retained FR-331 manifest snapshot and the `deadline_ms` supplied in CG's routed request; it adds no request-level `domains` or `limits`. | Test (TC-013) |
| FR-006-AC-5 | Each serialized item preserves the request index, claim locus, kind, extent/bounds, selected backend and candidate set supplied by CG's routed output/routing seam; temporal proof bounds are serialized under `temporal.subject.proof_bounds`. | Partial (TC-013; full source projection awaits IR-629 and QSL-654) |
| FR-006-AC-6 | A plugin response that rejects the package or an item marks every affected routed item failed, preserves item identity, and carries `content-mismatch` or `invalid-value` as a typed FR-272 diagnostic separate from QSL `TerminalValue::Failed`. | Planned (TC-013) |
| FR-006-AC-7 | A validated plugin result enters the driver's shared settlement path only when QSL supplies its terminal mapping; otherwise it remains typed, unmapped provider data. Results for unrelated providers remain unchanged. | Planned (TC-013; IR-629, IR-677, QSL-490 and QSL-655) |
| FR-006-AC-8 | Before normal CLI command termination, the adapter requests graceful shutdown of every active plugin session; on Linux and macOS, if `RunLimits.cancellation_grace` expires first, the process tree terminates and cleanup completes within one second. The merged Windows host force-terminates the Job Object through `terminate_tree`/`terminate_job`; native Windows evidence remains planned. Kill-on-close is separately planned under IR-674. | Test (TC-028) |
| FR-006-AC-9 | On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal. No QSL cancellation cause is claimed; with no active session, the platform's default signal disposition is retained. | Test (TC-028) |

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
- CG IR-629 supplies the process-origin projection in its routed output and
  constructs `BackendKind::Process` from the descriptor origin. The driver
  passes QSL's origin through and does not map the backend identity to a kind.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
