---
id: SR-14839
title: code-review — IR-1277 canonical driver APIs
type: SpecReview
analysis: code-review
scope: agent-ix/quire-driver; IR-1277; spec/functional/FR-001-orchestrate-the-driver-steps.md,
  src/drive.rs, src/upstream.rs, src/refusal.rs, src/provider.rs, quire-cli/src/backend_provider.rs,
  tests/drive.rs
review_set: subset
---

## Summary

The original admitted value moves once into Arc, claim forms clone only that owner, and both CG boundaries borrow it. The borrowed structural comparison preserves package/path/ordinal and returns the admitted wire ID; no source package, DTO or semantic evaluator is reconstructed. New refusal arms are explicit and retain typed payloads, including negotiation invariant faults versus generation invalid requests. Trait decorators and optional statistical fields follow their owners directly. No gate is weakened, unsafe code added or file/version ledger introduced.

## Verdict

PASS.

## Examined scope

```yaml
- id: FR-001-AC-5
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: Each command-level refusal carries its step, catalog code and outcome kind.
    A `Compile` refusal's code renders `code/cause` when the refusing stage names
    a cause (e.g. `missing_import`/`missing-selection`), not just the bare code.
- id: FR-001-AC-12
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: For each request item, the driver's CG negotiation input carries the typed
    IR tag/form projected from the admitted checked requirement and node named by
    that same item's QSL requirement record; generation preserves that item's exact
    application occurrence and direct conversion-context occurrence as ordered lowering
    roots, including when several items share one application node. No caller-computed
    lowerability/refusal or display/debug-derived form substitutes for the checked
    form.
- id: FR-001-AC-15
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: CG receives the original E5-admitted `CheckedPackageV2` by reference alongside
    its existing admitted `ContractPackageV1`, complete FR-330 position vector, routed
    items, and generation contexts. The driver retains and shares the original admitted
    source owner without reconstruction or deep copying. The position vector retains
    one position per original request item, in driver-index order (a `DriverItem`
    for each bound item, an excluded position, keeping the item's `node_id` and counted
    in driver indices, for each export-refused item; excluded positions are absent
    from the FR-331 envelope), and a
- id: FR-001-AC-16
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'No selected scalar item means no scalar binding or native execution and
    no fallback selection. QSL''s typed missing/ambiguous call or occurrence refusal
    is preserved unchanged. A missing/ambiguous required conversion context preserves
    its owning QSL/IR/CG typed refusal; `None` is valid only when the exact QSL item
    has no conversion context. An admitted-package/request-position/routed-item mismatch
    preserves the owning IR/CG typed refusal, or is a driver `runtime_invariant` fault
    when the driver detects an inconsistent join. Each case stops before native execution
    or replay and invents no call, '
- id: FR-001-AC-22
  path: spec/functional/FR-001-orchestrate-the-driver-steps.md
  role: examined
  excerpt: 'When CG returns `EnvelopeRefusal::ContractPackageReadRefused`, the driver
    emits an FR-272 refusal diagnostic at `package_read`, retaining the complete CG
    projection unchanged as typed source data: `code`, `path`, `cause`, `locus`, `contract_version`,
    `document_pointer`, `expected` and `actual`. The diagnostic carries the same code
    and cause, has category `refusal`, maps `path` to the FR-274 `json_path` location,
    and carries `locus` as a `CheckedNodeId` payload when present; every returned
    code has an FR-271 row. When CG returns `ContractPackageReadIncomplete`, the driver
    emits `stage_limit_exc'
- id: src/drive.rs
  path: src/drive.rs
  role: examined
  excerpt: "fn structural_key_matches(id: &CheckedNodeId, key: &quire_exact::NodeKey)\
    \ -> bool {\n    use quire_contract_model::CheckedNodePathSegment;\n    use quire_exact::PathSegmentView;\n\
    \n    let mut pending = vec![(id, key.view())];\n    while let Some((id, key))\
    \ = pending.pop() {\n        if id.package.as_ref() != key.package() || id.ordinal\
    \ != key.ordinal() {\n            return false;\n        }\n        let mut path\
    \ = key.path();\n        for segment in &id.path {\n            match segment\
    \ {\n               "
- id: src/upstream.rs
  path: src/upstream.rs
  role: examined
  excerpt: "    fn negotiate(\n        &self,\n        source: &CheckedPackageV2,\n\
    \        package: &ContractPackageV1,\n        positions: &[ContractRequestPosition],\n\
    \        envelope: &BackendProviderEnvelope,\n    ) -> Result<Vec<ContractItemSettlement>,\
    \ Box<ContractNegotiationError>>;\n\n    /// The routing step: `qsl_route::routing::route`.\n\
    \    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>>;\n\
    \n    /// CG generation over the routed items:\n    /// `quire_contract_codegen::gene"
- id: src/refusal.rs
  path: src/refusal.rs
  role: examined
  excerpt: "fn package_request_code(error: &PackageRequestError) -> Code {\n    match\
    \ error {\n        PackageRequestError::ExcludedDriverIndex { .. } => Code::RuntimeInvariant,\n\
    \        PackageRequestError::SourceMismatch { .. }\n        | PackageRequestError::ItemPositions\
    \ { .. }\n        | PackageRequestError::Count { .. }\n        | PackageRequestError::Identity\
    \ { .. }\n        | PackageRequestError::AccountingIdentity { .. }\n        |\
    \ PackageRequestError::Position { .. }\n        | PackageRequestError::OrphanC"
- id: src/provider.rs
  path: src/provider.rs
  role: examined
  excerpt: "                delay_grid_bits: None,\n            },\n            provenance:\
    \ StatisticalProvenance {\n                seed: 0,\n                timed_horizon:\
    \ None,\n                delay_grid_bits: None,\n                max_sample_steps:\
    \ None,\n                sampler: qsl_foundation::selection::DefinitionRef::new(\n\
    \                    \"agent-ix\",\n                    \"quire.simulation.sampler/v1\"\
    ,\n                )\n                .expect(\"sampler has a valid identity\"\
    ),\n                tests: Vec:"
- id: quire-cli/src/backend_provider.rs
  path: quire-cli/src/backend_provider.rs
  role: examined
  excerpt: "        fn negotiate(\n            &self,\n            source: &quire_contract_model::CheckedPackageV2,\n\
    \            package: &quire_contract_model::ContractPackageV1,\n            positions:\
    \ &[cg::ContractRequestPosition],\n            envelope: &cg::BackendProviderEnvelope,\n\
    \        ) -> Result<Vec<cg::ContractItemSettlement>, Box<cg::ContractNegotiationError>>\
    \ {\n            let mut settlements = QslCg.negotiate(source, package, positions,\
    \ envelope)?;\n            for index in &self.unsupported_at {\n"
- id: tests/drive.rs
  path: tests/drive.rs
  role: examined
  excerpt: "fn foreign_admitted_source_is_refused_by_negotiation_and_generation()\
    \ {\n    let registry = build_registry(&[manifest(\"kani\", &[\"value-validity\"\
    ])])\n        .expect(\"the kani manifest registers\");\n    let packages = BTreeMap::new();\n\
    \    let recorder = Recorder::default();\n    let original = drive(\n        &recorder,\n\
    \        int_input(&packages),\n        &registry,\n        None,\n        kani_input(),\n\
    \    )\n    .expect(\"the original admitted source generates\");\n    let foreign\
    \ = drive(\n        &Qsl"
```

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Rust API, implementation, safety/concurrency, wire and testing rules were applied. No applicable AssuranceProfile exists in the repository. Inspected native receipts: driver IT 35 passed; library 81 passed/1 ignored; workspace all-target Clippy with CLI adapter passed. The parent owns the remaining final/downstream gate; no heavy build, backend proof or hosted workflow was replayed by this reviewer. Optional gap semantic fan-out and unrelated language/security analyses were not selected.
