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

Scope: FR-005-AC-5 through FR-005-AC-11, FR-006-AC-1 through FR-006-AC-2,
and FR-006-AC-5 through FR-006-AC-17.

## Test Procedure

1. Build an emitted package and a registry snapshot from the original
   losslessly retained FR-331 manifest DTOs. Supply the deadline in the
   caller's process-provider request. Use CG's routed output/routing seam for each
   request index, claim locus and kind, extent/bounds, selected backend,
   candidates and any temporal subject proof bounds.
2. Serialize the request through the typed FR-331 adapter. Verify `package`
   contains the emitted package's RFC 8785 bytes as unpadded base64url,
   `package_id` is copied from that same `EmittedPackage`, the registry
   snapshot preserves optional descriptor fields, and `deadline_ms` comes
   from the caller. Verify bounds appear only at `items[].extent.bounds` and
   `items[].temporal.subject.proof_bounds`, with no request-level `domains` or
   `limits` member.
3. Start the explicit fixture executable. Decode its FR-331 hello DTO without
   discarding optional fields. Pass its identity and advertised labels through
   one QSL descriptor admission with caller-supplied `Process` origin, then
   register the returned descriptor in the existing registry without a
   second admission. An absent or unknown kind/mode reaches QSL and remains
   its typed `invalid_capability` refusal.
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
6. Send a request with a `package_id` that does not match the decoded package.
   Verify the plugin responds with one FR-331 result that marks every routed
   item failed and carries FR-272 `stale_dependency/content-mismatch`. Repeat
   with a non-canonical spelling of the package; verify the plugin responds
   with one FR-331 result marking every routed item failed with
   `invalid_runtime_input/invalid-value`.
   Verify the driver decodes the typed FR-331 result and FR-272 diagnostic,
   maps each failed result to QSL `TerminalValue::Failed`, and carries the
   diagnostic separately as an `OutcomeDiagnostic` in the outcome document;
   unrelated provider results remain unchanged. A free-form diagnostic
   message or detail never selects a QSL terminal.
7. Verify PL-7 transport failures are handled by the driver's typed transport
   failure mapping and are not misreported as plugin request-body refusals.

The separate host implementation's PL-1 through PL-7 conformance is verified
by [TC-023](./TC-023-plugin-session-and-admission.md) through
[TC-026](./TC-026-plugin-resource-bounds-and-cleanup.md). CG IR-629 is required
for end-to-end process negotiation and settlement. The current
`RoutedItemOutput` does not carry the full FR-331 request item or deadline. The
IR-629 routed-request seam must expose the original request index, claim and
occurrence, kind, extent/bounds, selected backend and candidates, plus the
request deadline. Bounds `kind` additionally waits on QSL-654. These missing
source fields are tracked integration gaps; the test must not synthesize them
from unrelated provider evidence.

Test location: planned `quire-cli` FR-331 hello/request adapter tests and
driver/provider composition tests. IR-672 PR3 covers the hello/admission and
request DTO stages. Result conversion and settlement use explicit typed input
fixtures and remain held for the IR-677 carrier and QSL result-mapping APIs;
end-to-end negotiation and settlement also require CG IR-629.

## Expected Results

- Request fields come from their named owners: `package` and `package_id` from
  the same `EmittedPackage`; `manifest` from retained FR-331 manifest DTOs;
  `deadline_ms` from the caller's process-provider request; and item
  request index, claim locus, kind, extent/bounds, selected backend and
  candidates from CG's routed output/routing seam.
- The test fixture supplies all-items-failed responses for package mismatch or
  non-canonical package spelling. The driver maps those to QSL `Failed`
  terminals and attaches typed FR-272 diagnostics separately; it does not
  claim QSL read the request body.
- The planned conversion matrix enumerates the 13 Kani rows in QSpec
  FR-331-AC-20's Kani result table, not the larger vectors file. Replay-only
  plugin claims become tool failures recorded as failed items only after the
  QSL-655 PL-7 amendment;
  counterexamples remain non-terminal candidates until replay, and plugin
  `proved` remains unmapped without fabricated Kani `success_checks` pending
  QSL-490 and QSL-655. Kani terminal mapping remains CG-owned.
