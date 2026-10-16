---
id: TC-013
title: "Dispatch process-provider results through the driver"
type: TC
relationships:
  - target: ix://agent-ix/quire-driver/FR-006
    type: verifies
  - target: ix://agent-ix/quire-driver/FR-005
    type: verifies
---
# TC-013: Dispatch process-provider results through the driver

## Description

Verify that the driver passes negotiated process-provider work through its
process-provider seam and returns typed provider results through its proof
path; TC-028 verifies that `quire-cli` composes this seam with
`quire-plugin-host`.

Scope: FR-005-AC-12 through FR-005-AC-19, FR-006-AC-1 through FR-006-AC-2,
and FR-006-AC-5 through FR-006-AC-23.

## Test Procedure

1. Build an emitted package and retain one lossless FR-331 manifest DTO in the
   driver's registry record for every linked and process candidate. Have the
   driver compose the full registered-candidate snapshot. For each constructed
   request item, serialize request index, claim locus, kind, extent/bounds and
   candidate set from the corresponding retained QSL
   `RequirementItem`/`CandidateOutcome`, using the same request data used to
   build typed CG negotiation; take the selected backend from CG's routed
   output. Supply the snapshot and constructed request data with the package,
   its package identity and caller's deadline to the adapter serialization
   check. This step checks field ownership and serialization only; it does not
   imply that the driver dispatches negotiated work through the Process seam,
   which remains planned under IR-629. Temporal subject proof bounds remain
   sourced as specified in FR-006.
2. Have the `quire-cli` adapter serialize the request through the typed FR-331
   adapter. Verify `package`
   contains the emitted package's RFC 8785 bytes as unpadded base64url,
   `package_id` is copied from that same `EmittedPackage`, the `quire-cli`
   adapter serializes the supplied manifest snapshot without reconstructing
   it or adding members, and `deadline_ms` comes
   from the caller. Verify bounds appear only at `items[].extent.bounds` and
   `items[].temporal.subject.proof_bounds`, with no request-level `domains` or
   `limits` member. QSpec TC-271 owns the CG negotiation checks for QSpec
   FR-290-AC-13. Do not claim a serialized proof-bound
   `kind` until QSL-654 emitter support lands.
3. Start the explicit fixture executable. Have the `quire-cli` adapter decode
   its FR-331 hello DTO without discarding optional fields. Pass its identity,
   advertised labels and domain labels through one QSL descriptor admission
   with caller-supplied `Process` origin, then return the admitted descriptor
   and lossless FR-331 DTO to the driver. Register the descriptor with the one
   QSL registry owned by the driver's `AdmittedRegistry`, without a second
   admission. Retain one lossless `ProviderManifest` carrier for each
   registered candidate. Convert the process hello DTO losslessly into that
   carrier; populate linked-provider carriers from their FR-331 manifests and
   do not synthesize absent optional members. An absent or unknown kind/mode
   reaches QSL and remains its typed `invalid_capability` refusal. Have the
   `quire-cli` host hello admitter pass domain labels to QSL and verify it returns
   `invalid_capability` with cause `invalid-domains`;
   the driver does not validate domains or construct that refusal. Verify
   descriptor identity, advertised pairs and supplied origin are passed to CG
   unchanged (FR-005-AC-5). Verify the driver passes the projected descriptor
   to CG's `BackendKind::from_descriptor`; CG constructs `BackendKind::Process`
   from the supplied origin. TC-012 covers this conversion. The driver does not
   construct that kind or infer it from backend identity.

4. Use each of the 13 Kani result rows in QSpec FR-331-AC-20's Kani results
   table as a conversion case. For each row, compare the typed result fields
   with the current QSL terminal contract and exercise a terminal only where
   that contract supplies one. Keep a plugin counterexample in the shared
   carrier as a non-terminal candidate until QSL S6a replay reproduces it.
   Do not create a terminal for that candidate before replay. Keep plugin
   `proved` results unmapped until QSL-490 and QSL-655 supply the trusted
   proof-basis terminal.
5. Once QSL-655 amends ADR-029 PL-7 to define plugin claims of
   `replay-parity`, `replay-refused` or `kani_vacuous_proof` as invalid, have
   the plugin make each claim. Verify the driver rejects each as a tool failure
   and records the affected item as failed; the plugin cannot supply QSL's
   replay disagreement payload or replay result. Until that PL-7 amendment
   lands, this case remains planned and defines no driver behavior.
6. Send a valid plugin response with a `declined` result and a typed code.
   Verify that the `quire-cli` adapter preserves the decoded result and that
   the driver's shared settlement path preserves the code in QSL
   `TerminalValue::Declined { code }`. Send a valid response whose result has
   no current QSL terminal mapping; verify it remains typed and unmapped.
   These cases remain planned until the relevant QSL mapping is available.
7. Send a request with a `package_id` that does not match the decoded package.
   Verify the FR-331 response marks every routed item failed with code
   `stale_dependency` and cause `content-mismatch`.
8. Verify the `quire-cli` adapter preserves every routed item identity and the
   typed diagnostic from the package-mismatch response.
9. Verify the driver's shared settlement path maps each failed routed item to
   QSL `TerminalValue::Failed`.
10. Verify the shared settlement path carries the typed diagnostic separately
    in QSL `OutcomeDiagnostic`.
11. Verify settlement of the failed process-provider response leaves
    unrelated provider results unchanged.
12. Repeat with a non-canonical spelling of the package. Verify the FR-331
    response marks every routed item failed with code `invalid_runtime_input`
    and cause `invalid-value`.
13. Verify the `quire-cli` adapter preserves every routed item identity and the
    typed diagnostic from the non-canonical-package response.
14. Verify the driver's shared settlement path maps each failed routed item to
    QSL `TerminalValue::Failed` and carries the diagnostic separately in QSL
    `OutcomeDiagnostic`.
15. Verify settlement leaves unrelated provider results unchanged. A free-form
    diagnostic message or detail never selects a QSL terminal.
16. Verify PL-7 transport failures are handled by the driver's typed transport
    failure mapping and are not misreported as plugin request-body refusals.

The separate host implementation's PL-1 through PL-7 conformance is verified
by [TC-023](./TC-023-plugin-session-and-admission.md) through
[TC-026](./TC-026-plugin-resource-bounds-and-cleanup.md). The driver now uses
CG IR-629's origin-aware descriptor conversion. End-to-end process-provider
dispatch and settlement remain planned. The current
`RoutedItemOutput` does not carry the full FR-331 request item or deadline. The
IR-629 routed-request seam must expose the original request index, claim and
occurrence, kind, extent/bounds, selected backend and candidates, plus the
request deadline. FR-331 specifies the proof-bound `kind`; QSL-654 owns the
pending QSL emitter support for writing it. These missing source fields are
tracked integration gaps; the test must not synthesize them from unrelated provider
evidence.

Test location: planned `quire-cli` FR-331 hello/request adapter tests and
driver/provider composition tests. IR-672 PR3 covers the hello/admission and
request DTO stages. Result conversion and settlement use explicit typed input
fixtures; the shared carrier preserves generic typed results, while settlement
remains held for QSL result-mapping APIs;
end-to-end process-provider dispatch and settlement remain planned under
IR-672 and the QSL result-mapping dependencies.

## Expected Results

- Request fields are serialized by the `quire-cli` adapter from their named
  owners: `package` and `package_id` from
  the same `EmittedPackage`; `manifest` from the driver's registered-candidate
  records; `deadline_ms` from the caller's process-provider request; and item
  request index, claim locus, kind, extent/bounds, selected backend and
  candidates from CG's routed output/routing seam.
- The driver composes the manifest snapshot from one retained DTO in each
  registered-candidate record; the adapter does not reconstruct or add
  members.
- The test fixture supplies all-items-failed responses for package mismatch or
  non-canonical package spelling. The `quire-cli` adapter decodes those
  responses; the driver's shared settlement path maps them to QSL `Failed`
  terminals and attaches typed FR-272 diagnostics separately. It does not
  claim QSL read the request body.
- The planned conversion matrix enumerates the 13 Kani rows in QSpec
  FR-331-AC-20's Kani result table, not the larger vectors file. Replay-only
  plugin claims become tool failures recorded as failed items only after the
  QSL-655 PL-7 amendment;
  counterexamples remain non-terminal candidates until replay, and plugin
  `proved` remains unmapped without fabricated Kani `success_checks` pending
  QSL-490 and QSL-655. Kani terminal mapping remains CG-owned.
