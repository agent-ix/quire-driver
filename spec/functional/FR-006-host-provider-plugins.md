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
  - target: ix://agent-ix/quire-specification/FR-290
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
- Explicit process-provider configuration supplied to the `quire-cli`
  composition by `--provider-config <path>`: an executable path, argument
  vector, a per-provider `deadline_ms`, and `ProcessProviderLimits` per entry.
- The admitted provider manifest, negotiated provider identity and routed
  request items.
- The emitted checked-package bytes, frame and session-byte limits, stderr
  retention limit, and cancellation-grace interval.
- The caller-owned QSL cancellation handle for the active run.
- The process-provider seam supplied by the `quire-cli` composition.

## Outputs

- The typed provider results returned for the routed items, consumed by the
  driver's proof settlement path.
- Bounded plugin process-tree cleanup before normal CLI termination or exit
  after handled SIGINT/SIGTERM on platforms covered by this requirement.

## Behavior

- The driver SHALL expose and invoke a process-provider seam that receives
  the named executable, admitted provider identity, one `EmittedPackage` value
  (including its bytes), caller-supplied deadline,
  the driver's registered-candidate manifest snapshot, negotiated items and
  caller limits. The driver package SHALL NOT depend on `quire-plugin-host`.
- The `quire-cli` composition SHALL implement the process-provider seam with
  `quire-plugin-host`, using QSL ADR-029 PL-1 through PL-7 as the authoritative
  lifecycle and failure contract. It SHALL use FR-331's provider envelope and
  QSL-owned typed terminal values; it SHALL NOT define a second wire format or
  infer terminal categories from provider-specific evidence or diagnostics.
- The `quire-cli` `run` command SHALL accept `--provider-config <path>` to name
  the process-provider configuration file. The file SHALL be UTF-8 JSON with
  one top-level field, `providers`, whose value is an array. Each entry SHALL
  have exactly `executable` (the executable path), `argv` (an array of argument
  strings, including an explicit empty array when no arguments are needed),
  `deadline_ms`, and `limits` fields. `deadline_ms` SHALL be a positive safe
  JSON integer from 1 through 9,007,199,254,740,991 inclusive, the FR-331
  `deadline_ms` range. It is the caller-supplied deadline for that provider's
  FR-331 request, not a host resource limit. The `limits` object SHALL contain
  exactly the required unsigned integer fields in the `u64` range:
  `max_frame_bytes`,
  `max_total_bytes`, `max_stderr_bytes`, and `cancellation_grace_ms`. Byte
  fields are byte counts; duration fields are milliseconds.
- The `quire-cli` composition SHALL reject an unreadable or missing file,
  malformed JSON, missing or wrongly typed required fields, and unknown fields
  at every object level before starting any provider. It SHALL also reject a
  missing, non-integer, zero, negative, or out-of-range `deadline_ms` before
  starting any provider.
- For each configured entry, the `quire-cli` composition SHALL start the named
  executable and admit its hello before negotiation. The identity in that
  once-admitted hello descriptor SHALL be the sole identity used to register
  and route that process provider. The composition SHALL pass the entry's
  executable, `argv`, `deadline_ms`, and limits unchanged to the
  process-provider seam.
- The `quire-cli` composition SHALL start and admit every configured entry
  before beginning negotiation.
- If distinct configured process sessions return the same identity from their
  admitted hellos, the `quire-cli` composition SHALL stop before negotiation and
  close all those sessions. It SHALL preserve any QSL registration refusal
  unchanged; if QSL reports identical repeated admission as idempotent, the
  composition SHALL return the existing CLI `InvalidRequest` input refusal
  rather than route an ambiguous identity.
- If a configured provider fails to start or its hello fails before all
  providers are prepared, the `quire-cli` composition SHALL begin no
  negotiation, close every earlier active session, and preserve the original
  typed host failure.
- The `quire-cli` composition SHALL NOT read process-provider configuration
  from ambient environment state, a default file, or a search path. When
  `--provider-config` is omitted, the run SHALL register no process providers;
  when the file contains an empty `providers` array, it SHALL also register no
  process providers.
- The `quire-cli` adapter SHALL start the host session with the caller-named
  executable path, pass the once-admitted descriptor returned by the host hello
  admitter to the driver's existing registry without re-admitting it or
  building a second registry, and send only the items negotiated to that
  provider.
- The driver registry record SHALL retain one lossless FR-331 manifest DTO for
  each registered candidate: process DTOs are retained at hello admission and
  linked-provider DTOs are retained when their descriptors are built. The
  driver SHALL compose the request's manifest snapshot from those records for
  the complete registered candidate set. It SHALL NOT reconstruct the
  snapshot from QSL registry descriptors, which do not retain optional FR-331
  fields.
- The `quire-cli` adapter SHALL serialize the supplied manifest snapshot
  without reconstructing it or adding members.
- The `quire-cli` adapter SHALL build each request's `package` from the emitted
  package's exact bytes. The request carries no `package_id`.
- The `quire-cli` adapter SHALL copy the `deadline_ms` supplied in the caller's
  selected provider configuration entry into the process-provider request and
  copy that same value unchanged into the FR-331 request. It SHALL NOT derive a
  deadline from a default, ambient state, or another provider entry. It SHALL
  NOT add request-level `domains` or `limits`.
- The `quire-cli` adapter SHALL serialize each item's request index, claim
  locus, kind, extent/bounds and candidate set from the corresponding retained
  QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to
  build typed CG negotiation. CG's routed output SHALL supply the selected
  backend. The adapter SHALL NOT treat the routed output as the source of the
  QSL-owned item fields.
- The `quire-cli` adapter SHALL place temporal proof bounds under
  `items[].temporal.subject.proof_bounds`. The current CG routed output does
  not retain all these fields; their projection through production composition
  remains incomplete. CG negotiation owns the PV-4 domain-kind
  comparison defined by QSpec FR-290-AC-13. The adapter SHALL NOT implement or
  restate that comparison.
  FR-331 specifies the proof-bound `kind`; QSL-654 owns QSL emitter support for
  writing that member, which remains pending. Other request projection fields
  are exercised by the selected-provider composition test.
- The driver SHALL pass a validated plugin result to the shared settlement
  path only when QSL supplies a terminal mapping for it. A result without a
  currently supported terminal mapping SHALL remain typed, unmapped provider
  data; it SHALL NOT be assigned a fabricated terminal.
- The driver's shared settlement path SHALL map a FR-331 `declined` result to
  QSL `TerminalValue::Declined { code }` using its typed code, and SHALL keep
  any response without a specified QSL terminal mapping typed and unmapped.
- The driver's shared settlement path SHALL put the FR-272 cause, diagnostic
  code and available locus and message into a QSL `OutcomeDiagnostic` in the
  outcome document. It SHALL NOT parse free-form message or `detail` text to
  choose a terminal; terminal mappings must be defined by QSL.
- Given an FR-331 response for non-canonical package spelling, the `quire-cli`
  adapter SHALL preserve each routed item identity and the typed FR-272
  diagnostic (`invalid_runtime_input` code with `invalid-value` cause).
- For either preserved response, the driver's shared settlement path SHALL map
  each failed item to QSL `TerminalValue::Failed` and carry the diagnostic
  separately in the outcome document. Results for unrelated providers SHALL
  remain unchanged.
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
| FR-006-AC-1 | When negotiation selects a process provider, the driver invokes its process-provider seam with the configuration entry associated with that provider's once-admitted hello identity, one `EmittedPackage` value, that entry's caller-supplied deadline, the registered-candidate manifest snapshot and exactly that provider's negotiated items; `quire-cli` implements the seam with `quire-plugin-host` and passes the once-admitted hello descriptor to the driver's existing registry. | Partial (TC-013, TC-028; production dispatch and typed-result retention are tested; complete field projection and later-start failure cleanup remain partial) |
| FR-006-AC-2 | Validated typed results returned through the process-provider seam enter the same driver proof settlement path as first-party provider results when QSL defines their terminal mapping; results without a current QSL mapping remain typed and unmapped, while unrelated routed items retain their own results. | Planned (TC-013; no end-to-end claim from an injected boundary; IR-672, IR-677, QSL-490 and QSL-655) |
| FR-006-AC-3 | Before normal CLI command termination, the adapter requests graceful shutdown of every active plugin session; on Linux and macOS, if `RunLimits.cancellation_grace` expires first, the process tree terminates and cleanup completes within one second. The merged Windows host force-terminates the Job Object through `terminate_tree`/`terminate_job`; native Windows evidence remains planned. Kill-on-close is separately planned under IR-674. | Partial (TC-013, TC-028; normal host shutdown and production signal cleanup are covered, but the one-second forced-cleanup guarantee lacks a production binary assertion) |
| FR-006-AC-4 | On Linux or macOS, handled SIGINT or SIGTERM while a plugin session is starting, running or shutting down requests cancellation through the caller-owned QSL handle, waits for settlement or `RunLimits.cancellation_grace` expiry, and cleans up the plugin process tree before the CLI restores the default disposition and re-raises the signal. No QSL cancellation cause is claimed; with no active session, the platform's default signal disposition is retained. | Partial (TC-028; production binary tests cover SIGINT and SIGTERM during a request; startup and shutdown-wait lifecycle states remain untested) |
| FR-006-AC-5 | The driver composes the request manifest snapshot from the lossless FR-331 DTO retained in each registered-candidate registry record, covering linked and process providers; the `quire-cli` adapter serializes that snapshot without reconstructing or adding members. | Planned (TC-013) |
| FR-006-AC-6 | The `quire-cli` adapter serializes `package` as the unpadded base64url encoding of the emitted package's exact bytes. | Planned (TC-013) |
| FR-006-AC-7 | Retired: the request carries no `package_id`; the `quire-cli` adapter serializes the package bytes from the `EmittedPackage` value. | Not applicable |
| FR-006-AC-8 | The `quire-cli` adapter serializes `deadline_ms` equal to the `deadline_ms` in the selected provider's validated `--provider-config` entry, unchanged. | Test (TC-013, TC-028) |
| FR-006-AC-9 | The `quire-cli` adapter serializes a request without top-level `domains` or `limits` members. | Planned (TC-013) |
| FR-006-AC-10 | The `quire-cli` adapter serializes each item's request index, claim locus and kind from the corresponding retained QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation. | Partial (TC-013; production composition checks index, claim and kind; full field coverage remains incomplete) |
| FR-006-AC-11 | The `quire-cli` adapter serializes each item's selected backend from CG's routed output and its candidate set from the corresponding retained QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation. | Test (TC-013) |
| FR-006-AC-12 | The `quire-cli` adapter serializes each item's extent and bounds from the corresponding retained QSL `RequirementItem`/`CandidateOutcome`, using the same request data used to build typed CG negotiation. | Partial (TC-013; extent is asserted; proof-bound kind emission remains pending QSL support) |
| FR-006-AC-13 | The `quire-cli` adapter serializes each item's temporal proof bounds under `items[].temporal.subject.proof_bounds`. | Partial (TC-013; production projection exists, but temporal-bound output is not yet asserted) |
| FR-006-AC-14 | The driver's shared settlement path preserves a plugin `declined` result's typed code in QSL `TerminalValue::Declined { code }`. | Planned (TC-013) |
| FR-006-AC-15 | A validated result without a specified QSL terminal mapping remains typed and unmapped in the driver's shared settlement path. | Planned (TC-013; IR-677 and QSL result-mapping APIs) |
| FR-006-AC-16 | The driver's shared settlement path carries the FR-272 cause, diagnostic code and available locus and message in a QSL `OutcomeDiagnostic`; free-form message or `detail` text does not select a terminal. | Planned (TC-013) |
| FR-006-AC-17 | Retired: the package-id mismatch case no longer exists (the request carries no `package_id`); the id is not reused. | Not applicable |
| FR-006-AC-18 | Given an FR-331 response for non-canonical package spelling, the `quire-cli` adapter preserves each routed item identity and the FR-272 diagnostic with code `invalid_runtime_input` and cause `invalid-value`. | Planned (TC-013) |
| FR-006-AC-19 | The driver synthesizes failed provider results only for host/protocol failures defined by ADR-029 PL-7; it does not attribute a process request-body refusal to QSL. | Planned (TC-013) |
| FR-006-AC-20 | For a preserved package-mismatch response, the driver's shared settlement path maps each failed routed item to QSL `TerminalValue::Failed`. | Planned (TC-013; IR-677 and QSL settlement API) |
| FR-006-AC-21 | For a preserved non-canonical-package response, the driver's shared settlement path maps each failed routed item to QSL `TerminalValue::Failed`. | Planned (TC-013; IR-677 and QSL settlement API) |
| FR-006-AC-22 | For package-mismatch and non-canonical-package responses, the driver's shared settlement path carries each typed FR-272 diagnostic separately in QSL `OutcomeDiagnostic`. | Planned (TC-013; IR-677 and QSL settlement API) |
| FR-006-AC-23 | Settling a failed process-provider response leaves unrelated provider results unchanged. | Planned (TC-013; IR-677 and QSL settlement API) |
| FR-006-AC-24 | The `--provider-config` file accepts only the documented strict JSON shape; missing/unreadable files, malformed JSON, missing or wrongly typed fields, a `deadline_ms` that is missing, non-integer, outside 1..=9,007,199,254,740,991, and unknown fields at any object level are rejected before any provider starts. In machine mode, these failures use the existing `Operation::Execute` `Code::InvalidRequest` input-refusal result and serialize that typed outcome once. | Test (TC-028) |
| FR-006-AC-25 | The `quire-cli` composition starts and admits every configured provider before negotiation, and uses each admitted hello descriptor as the sole provider identity for subsequent selection and dispatch. | Partial (TC-028; selected-provider production dispatch is tested; later-start/hello failure cleanup remains untested) |
| FR-006-AC-26 | If distinct configured sessions return the same admitted identity, the composition begins no negotiation and closes all sessions; it preserves a QSL registration refusal, or returns the existing CLI `InvalidRequest` input refusal when QSL treats repeated identical admission as idempotent. | Test (TC-028; idempotent duplicate admission path) |
| FR-006-AC-27 | If a configured provider fails to start or admit its hello, the composition begins no negotiation, closes all earlier active sessions, and preserves the original typed host failure. | Planned (TC-028) |

## Dependencies

- QSL ADR-029 PL-1 through PL-7 define the plugin protocol, host lifecycle,
  manifest admission and typed failure behavior; PV-1 through PV-4 define
  provider registration and negotiation. [FR-013](./FR-013-host-process-plugins.md)
  specifies the separate host implementation. FR-014 specifies its
  composition by the `quire-cli` package.
- [FR-331](ix://agent-ix/quire-specification/FR-331) owns the
  `quire.backend-provider/v1` envelope; QSpec FR-290-AC-13 owns the PV-4
  domain-kind rule; QSL owns typed terminal values and
  `quire-outcome/1` serialization.
- QSL-654 specifies the proof-bound `kind` member; QSL emitter support for
  writing it remains pending. QSpec FR-290-AC-13 defines the PV-4 domain-kind
  comparison rule. CG's separate `bounds[].kind` work is
  outside this request adapter.
- QSL-490 and QSL-655 are prerequisites for a plugin `proved` result's trusted
  `ProofBasis` and for replay settlement of plugin counterexample candidates.
  Until those APIs land, these result forms are not mapped to terminals.
- FR-005 defines the first-party `Provider` interface and shared manifest
  admission.
- CG IR-629 specifies the process-origin projection under CG FR-019 and
  FR-022. Its typed mapping is available; selected-provider production
  composition is implemented, with remaining request-field and failure-path
  coverage identified in the acceptance rows.
- [TC-013](../test-cases/TC-013-plugin-host-protocol-and-failures.md).
