---
id: FR-005
title: "Register first-party providers through one typed interface"
type: FR
relationships:
  - target: ix://agent-ix/quire-driver/US-001
    type: implements
  - target: ix://agent-ix/quire-spec-language/ADR-029
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-288
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-075
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-331
    type: depends_on
  - target: ix://agent-ix/quire-spec-language/FR-069
    type: depends_on
  - target: ix://agent-ix/quire-driver/FR-002
    type: depends_on
---
# FR-005: Register first-party providers through one typed interface

## Description

The driver SHALL expose one `Provider` trait in its library for each first-party
provider linked into the binary. The trait SHALL expose the provider's FR-331
manifest and run the routed request with the emitted package, provider limits
and cancellation handle, returning typed provider results. The driver SHALL
register each linked provider through its manifest and pass the resulting
registry to the lifecycle operation that uses it. Process-provider registration
is specified in Behavior.

## Inputs

- A first-party provider implementation.
- Its FR-331 `ProviderManifest`.
- The emitted package, routed `ProviderRequest`, `ProviderLimits` and
  `Cancel` supplied for one run.
- For Kani generation, an explicit CG `ProofCeilings` value carrying the
  memory-byte and wall-clock ceilings that become part of each harness
  identity.

## Outputs

- The provider manifest for registration.
- Typed `ProviderResults` for the routed request.

## Behavior

- The trait SHALL provide `manifest(&self) -> ProviderManifest` and
  `run(&self, package: &EmittedPackage, request: &ProviderRequest, limits:
  &ProviderLimits, cancel: &Cancel) -> ProviderResults`.
- The driver SHALL register the manifest using the same manifest conversion
  used to build the backend registry.
- The driver SHALL derive backend identity from the manifest identity.
- The driver SHALL pass the request's emitted package, routed items, limits
  and cancellation handle to the selected provider without altering them.
- The driver SHALL preserve each provider result's request index and backend
  identity in the shared result carrier.
- For Kani results, the driver SHALL use CG's `run_terminal_value` mapping
  (FR-009-AC-4); Kani evidence SHALL NOT be used to define a second terminal
  mapping in the driver.
- When QSL replay has not settled a plugin counterexample, the driver SHALL
  retain it as a non-terminal candidate and SHALL NOT emit `Refuted` for it.
  QSL replay must reproduce the candidate before the driver emits `Refuted`
  (ADR-029 PV-2 item 4).
- The driver's shared `ProviderResults` SHALL represent provider results and
  failures without requiring Kani-specific evidence or failure types. For a
  Kani result, the driver SHALL use CG's authoritative `run_terminal_value`
  mapping (FR-009-AC-4); for a process-provider result, QSL owns terminal
  values and their categories. The driver SHALL preserve typed FR-331 result
  fields and FR-272 diagnostic fields without parsing free-form diagnostic
  text to choose a QSL terminal. It SHALL NOT define a parallel terminal
  vocabulary or remap a QSL terminal from text.
- The driver SHALL preserve a provider failure's typed FR-272 `Diagnostic`,
  including fields represented by QSL's `OutcomeDiagnostic`, separate from
  the QSL terminal value. It SHALL NOT inspect free-form diagnostic text or
  detail to choose a terminal; a terminal mapping must be defined by QSL.
- When QSL has not provided its trusted proof-basis terminal for a plugin
  `proved` result, the driver SHALL retain the result as unmapped data and omit
  fabricated Kani `success_checks`.
- If QSL-655 amends ADR-029 PL-7 to define plugin claims of `replay-parity`,
  `replay-refused`, or `kani_vacuous_proof` as invalid, the driver SHALL reject
  those claims as tool failures. Until that amendment lands, this requirement
  does not define handling for those plugin claims.
- When a caller requests Kani generation, the caller SHALL supply CG
  `ProofCeilings` containing explicit memory-byte and wall-clock bounds.
- The driver SHALL pass the caller's `ProofCeilings` unchanged to
  `KaniGenerationContext`.
- When Kani execution begins, the provider SHALL require the caller's
  `ProofCeilings` to equal those embedded in the generated harness identity.
  A mismatch SHALL remain a typed refusal.
- CG execution SHALL use the embedded identity ceilings and retain its
  observed ceilings, memory observation and symbolic-argument bounds in the
  typed execution evidence. CG FR-028 defines the wall-clock ceiling as the
  execution timeout; the driver SHALL NOT supply a second timeout.
- For each plugin hello, the `quire-cli` host hello admitter SHALL call the QSL
  descriptor-admission boundary once with caller-supplied origin `Process` and
  return the admitted descriptor to the driver; the driver's single
  `qsl_route::Registry` SHALL register that descriptor without re-admitting
  it, and `quire-cli` SHALL NOT build another registry. The driver descriptor
  builder SHALL supply origin `Linked` for compile-time providers, and
  `Registry::register` SHALL preserve each caller-supplied origin. The driver passes
  resulting descriptors to CG without constructing CG backend kinds. It SHALL
  preserve the registry's typed duplicate-backend
  refusal and withdrawn-identity outcome; it SHALL NOT implement duplicate
  policy itself. Equal descriptors are idempotent and an unequal descriptor
  for an existing `BackendId` withdraws that identity (QSL FR-075-AC-4/AC-7,
  FR-290-AC-9/10 and QSL-637).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | A compile-time provider implementing `Provider` returns its FR-331 manifest; the descriptor-construction caller supplies origin `Linked`, and QSL registry registration preserves that origin and the manifest identity. | Test (TC-012; `src/registry.rs` asserts the Linked origin; Process origin remains planned under FR-005-AC-5) |
| FR-005-AC-2 | The provider receives the same emitted package, routed request, limits and cancellation handle supplied by the driver, and its typed results are returned without semantic changes. Kani generation uses caller-supplied `ProofCeilings`; execution requires those ceilings to match the harness identity and preserves CG's ceiling, memory and symbolic-argument evidence. | Test (TC-012) |
| FR-005-AC-3 | Two providers with distinct manifest identities are registered as distinct backends; a run invokes only the provider selected for its routed items. | Test (TC-012) |
| FR-005-AC-4 | The driver preserves QSL registry results: an equal descriptor repeat is idempotent; an unequal descriptor for an existing `BackendId`, including one with a different origin, returns the typed duplicate-backend refusal, withdraws that identity, and invokes neither provider. | Partial (TC-012 checks equal linked-manifest idempotence; process-origin registration remains with plugin-host work) |
| FR-005-AC-5 | The `quire-cli` host hello admitter calls QSL descriptor admission once with caller-supplied origin `Process` and returns the admitted descriptor; the driver registers it in its single QSL registry without re-admission. The driver passes the descriptor's identity, advertised pairs and supplied origin to CG unchanged. After CG IR-629 implements the conversion specified by FR-019/FR-022, the driver calls that public conversion and CG constructs `BackendKind::Process` from the `Process` origin. The driver does not construct the kind or map backend identity to it. | Partial (TC-012 covers descriptor projection only; CLI/host composition and CG IR-629 code remain planned.) |
| FR-005-AC-6 | The driver's shared provider-result carrier preserves each routed item's request index and backend identity. | Planned (TC-012, TC-013; IR-677) |
| FR-005-AC-7 | A plugin counterexample received before QSL replay is represented in the shared result carrier as a non-terminal candidate; it has no `Refuted` terminal until QSL S6a replay reproduces it. | Planned (TC-013; QSL-655) |
| FR-005-AC-8 | For Kani results, the driver uses CG's authoritative `run_terminal_value` mapping; driver evidence and diagnostic text do not define or alter the terminal category. | Partial (TC-012 exercises CG's mapping; the driver does not implement a second mapping) |
| FR-005-AC-9 | The driver represents non-Kani provider data without Kani-specific evidence or failure types, while QSL remains the owner of terminal values and categories. | Planned (TC-012, TC-013; IR-677, QSL-655) |
| FR-005-AC-10 | The driver does not map plugin `proved` results to a terminal until QSL supports its trusted proof basis, and it does not fabricate Kani `success_checks`. | Planned (TC-013; QSL-490 and QSL-655) |
| FR-005-AC-11 | Once QSL-655 amends ADR-029 PL-7 to define plugin claims of `replay-parity`, `replay-refused`, or `kani_vacuous_proof` as invalid, the driver rejects those claims as tool failures. | Planned (TC-013; QSL-655 / ADR-029 PL-7 amendment) |

Current implementation status: the linked Kani adapter is the only Provider
execution adapter, and its `ProviderLimits` and result carriers are
Kani-specific. IR-677 generalizes the result carrier; it does not add a second
terminal vocabulary. Callers provide `ProofCeilings`
(`memory_bytes` and `wall_clock`) during generation and execution; the driver
requires equality with the generated harness identity, and CG uses that
embedded value for bounded execution (CG FR-028). Cancellation is checked
between routed items. CG's current `execute_kani_obligation` API accepts no
cancellation handle, so an already-running harness continues until the
generated identity's `wall_clock` ceiling. Process-origin plugin admission and backend-kind mapping
remain follow-up work; the QSL-to-CG descriptor projection is covered by
TC-012 while the execution slice remains Linked-only.

## Dependencies

- CG FR-017 and FR-028 for Kani execution evidence and proof ceilings.
- QSL ADR-029 PV-1 and PV-3 (the current driver carriers are Kani-only);
  QSL FR-075 and FR-288 for registry identity and provider-manifest
  registration.
- FR-006 defines the driver-owned process-provider seam; FR-013 defines the
  separate host, and FR-014 defines its `quire-cli` composition.
- QSL FR-069 defines the typed terminal reader and `quire-outcome/1` remains
  the serialized outcome contract.
- QSL-490 and QSL-655 are prerequisites for mapping plugin `proved` results to
  a trusted `ProofBasis` terminal and for replay-settling plugin counterexample
  candidates. Until then those plugin results remain unmapped at the terminal
  layer.
- [TC-012](../test-cases/TC-012-first-party-provider-interface.md).
- [FR-002](FR-002-expose-driver-cli-and-outcome-contract.md) defines the
  command entry point that registers and invokes providers.
