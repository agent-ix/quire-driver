// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001 acceptance tests: the driver's step order, the MD-2 refusal,
//! routing after negotiation, the E5 hand-off and CG generation. Every call goes to the real QSL and CG
//! functions; `Recorder` only logs the order in which the driver makes them.

use std::cell::RefCell;
use std::collections::BTreeMap;

use qsl_foundation::digest::WireNodeId;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{CompileRefusal, Compiled, SpineLimits, SpineStage};
use qsl_route::routing::Disposition;
use qsl_route::{BackendId, Candidate, ManifestDigest};
use qsl_semantics::check::{Capability, CheckingLimits};
use qsl_semantics::family::{ClaimExtent, Requirements};
use quire_contract_codegen::{
    generate_exact_scalar_oracles, generate_routed, AttestationContext, BackendKind,
    BackendProviderEnvelope, ClaimMap, EnvelopeRefusal, ExactScalarClaim, ExactScalarItem,
    ExactScalarOperation, GenerationContexts, IntegerOperator, ItemSettlement,
    KaniGenerationContext, KaniToolPins, RoutedGeneration, RoutedGenerationError,
    RoutedGenerationItem, IR_CANDIDATE_REVISION,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2,
};
use quire_contract_runtime::exact::{Integer, IntegerDomain, IntegerInterval};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, DriverRefusal, DriverStage, GenerationInput,
    KaniInput, OutcomeKind, ProviderManifest, QslCg, RequestedItem, Upstream,
};

const UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function f using v(): Boolean pure { true }\n";

/// A unit with an integer `+` (a `binary` expression) under a range
/// `conversion` expression: two nodes a Kani item can name.
const INT_UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function inc using v(x: Int[0, 9]): Int[0, 10] pure { x + 1 }\n";

/// A fixed attestation binding (a test value, never a sealed record).
const RECORD_DIGEST: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// One upstream call, as the driver made it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Compile,
    ReadPackage,
    BackendKind(String),
    Negotiate {
        items: usize,
    },
    Route {
        supported: usize,
        dispositions: usize,
    },
    Generate {
        items: usize,
    },
}

/// The real upstream, with a log of the calls the driver makes.
#[derive(Default)]
struct Recorder {
    calls: RefCell<Vec<Call>>,
}

impl Recorder {
    fn calls(&self) -> Vec<Call> {
        self.calls.borrow().clone()
    }
}

impl Upstream for Recorder {
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        packages: &BTreeMap<[u8; 32], Vec<u8>>,
        limits: SpineLimits,
    ) -> Result<Compiled, Box<CompileRefusal>> {
        self.calls.borrow_mut().push(Call::Compile);
        QslCg.compile(source, path, bytes, packages, limits)
    }

    fn read_package(
        &self,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        self.calls.borrow_mut().push(Call::ReadPackage);
        QslCg.read_package(bytes, limits, evidence)
    }

    fn backend_kind(&self, id: &BackendId) -> Option<BackendKind> {
        self.calls
            .borrow_mut()
            .push(Call::BackendKind(id.as_str().to_owned()));
        QslCg.backend_kind(id)
    }

    fn negotiate(
        &self,
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ItemSettlement>, EnvelopeRefusal> {
        self.calls.borrow_mut().push(Call::Negotiate {
            items: envelope.items.len(),
        });
        QslCg.negotiate(envelope)
    }

    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>> {
        let supported = dispositions
            .iter()
            .filter(|d| matches!(d, Disposition::Supported(_)))
            .count();
        self.calls.borrow_mut().push(Call::Route {
            supported,
            dispositions: dispositions.len(),
        });
        QslCg.route(dispositions)
    }

    fn generate(
        &self,
        package: &CheckedPackageV2,
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        self.calls.borrow_mut().push(Call::Generate {
            items: routed.len(),
        });
        QslCg.generate(package, routed, contexts)
    }
}

fn manifest(identity: &str, digest: u8, kinds: &[&str]) -> ProviderManifest {
    ProviderManifest {
        identity: identity.to_owned(),
        manifest_digest: ManifestDigest::from_digest([digest; 32]),
        tool: format!("{identity}-tool 1"),
        advertises: kinds
            .iter()
            .map(|kind| (Some((*kind).to_owned()), Some("bounded".to_owned())))
            .collect(),
    }
}

fn input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        source: SourceIdentity::new("agent-ix", "driver-test", "git", "1"),
        path: "unit.native",
        bytes: UNIT.as_bytes(),
        packages,
        limits: SpineLimits::default(),
    }
}

fn item(node: u8, kind: Capability, backend: Option<&str>) -> RequestedItem {
    item_at(WireNodeId::from_digest([node; 32]), kind, backend)
}

fn item_at(node: WireNodeId, kind: Capability, backend: Option<&str>) -> RequestedItem {
    RequestedItem {
        node,
        requirements: Requirements::new(kind, ClaimExtent::Bounded),
        backend: backend.map(BackendId::new),
    }
}

fn int_input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        bytes: INT_UNIT.as_bytes(),
        ..input(packages)
    }
}

/// `INT_UNIT` driven with no items: its E5 package.
fn int_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        int_input(&packages),
        &registry,
        &[],
        GenerationInput::default(),
    )
    .expect("the integer unit drives")
}

/// The one expression node of `form` in `package`: IR's id and its wire
/// spelling.
fn expression(package: &CheckedPackageV2, form: &str) -> (CheckedNodeId, WireNodeId) {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "expression" && node.semantic_form.as_ref() == form)
        .unwrap_or_else(|| panic!("the unit has a {form} expression"));
    let wire = WireNodeId::from_hex(&node.node_id.digest).expect("a wire node id");
    (node.node_id.clone(), wire)
}

/// The FR-014 claim map of the `+` node, as the caller supplies it.
fn claim_map(package: &CheckedPackageV2, add: &CheckedNodeId) -> ClaimMap<ExactScalarClaim> {
    let domain = IntegerDomain::Bounded(
        IntegerInterval::new(Integer::from(-1000_i64), Integer::from(1000_i64))
            .expect("a valid interval"),
    );
    let items = [ExactScalarItem {
        node_id: add.clone(),
        operation: ExactScalarOperation::IntegerArithmetic {
            operator: IntegerOperator::Add,
            domain,
        },
    }];
    generate_exact_scalar_oracles(package, &items)
        .expect("CG generates the claim map")
        .claim_map
}

fn attestation() -> AttestationContext<'static> {
    AttestationContext {
        record_digest: RECORD_DIGEST,
        candidate_revision: IR_CANDIDATE_REVISION,
    }
}

fn kani_input(claim_map: &ClaimMap<ExactScalarClaim>) -> GenerationInput<'_> {
    GenerationInput {
        kani: Some(KaniInput {
            claim_map,
            subject_path: "crate::subject",
            unwind: 1,
            attestation: attestation(),
        }),
    }
}

/// The kani backend as `route` returned it, in CG's wire spelling.
fn kani_candidate(driven: &Driven) -> quire_contract_codegen::Candidate {
    let candidate = &driven.routed[0].candidate;
    let digest = candidate.manifest_digest();
    quire_contract_codegen::Candidate {
        identity: candidate.id().as_str().to_owned(),
        manifest_digest: format!("{}:{}", digest.domain().as_str(), digest.hex()),
    }
}

/// FR-001-AC-2 (TC-002): a registered `BackendId` with no CG kind refuses
/// the run with `invalid_capability`/`unknown-backend`, naming every
/// unconverted id in bytewise order, and nothing is negotiated or routed.
#[test]
fn tc_002_unknown_backend_refuses_before_negotiation_in_bytewise_order() {
    // Bytewise order puts "Zeta" (0x5A) before "alpha" (0x61); a
    // case-insensitive order would not.
    let registry = build_registry(&[
        manifest("alpha", 1, &["value-validity"]),
        manifest("kani", 2, &["value-validity"]),
        manifest("Zeta", 3, &["operation-contract"]),
    ])
    .expect("the three manifests register");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();

    let refusal = drive(
        &recorder,
        input(&packages),
        &registry,
        &[item(1, Capability::ValueValidity, None)],
        GenerationInput::default(),
    )
    .expect_err("two registered backends have no CG kind");

    let DriverRefusal::UnknownBackend { backends } = &refusal else {
        panic!("expected the MD-2 refusal, got {refusal:?}");
    };
    let names: Vec<&str> = backends.iter().map(BackendId::as_str).collect();
    assert_eq!(names, ["Zeta", "alpha"]);
    assert_eq!(refusal.code(), "invalid_capability/unknown-backend");
    assert_eq!(refusal.stage(), DriverStage::Conversion);
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
    assert_eq!(
        refusal.to_string(),
        "invalid_capability/unknown-backend: no CG backend kind for Zeta, alpha"
    );

    let calls = recorder.calls();
    assert!(
        !calls
            .iter()
            .any(|c| matches!(c, Call::Negotiate { .. } | Call::Route { .. })),
        "nothing is negotiated or routed after the MD-2 refusal: {calls:?}"
    );
    // Every registered id went through the conversion, once.
    let converted: Vec<&Call> = calls
        .iter()
        .filter(|c| matches!(c, Call::BackendKind(_)))
        .collect();
    assert_eq!(converted.len(), 3, "{calls:?}");
}

/// FR-001-AC-4, FR-001-AC-3, FR-001-AC-6 and FR-001-AC-7 (TC-001, TC-003,
/// TC-006): the driver compiles and emits, hands the bytes to IR, converts
/// every backend, negotiates every item once, routes only the items CG
/// settled `supported`, then generates for the routed item through CG.
#[test]
fn tc_001_tc_003_tc_006_steps_run_in_order_through_generation() {
    let checked = int_package().checked;
    let (add, add_wire) = expression(&checked, "binary");
    let claims = claim_map(&checked, &add);

    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let items = [
        // Supported: kani advertises value-validity, bounded extent.
        item_at(add_wire, Capability::ValueValidity, None),
        // Unsupported: no registered backend advertises operation-contract.
        item(2, Capability::OperationContract, None),
        // Invalid request: the request names an unregistered backend.
        item(3, Capability::ValueValidity, Some("missing")),
    ];

    let driven = drive(
        &recorder,
        int_input(&packages),
        &registry,
        &items,
        kani_input(&claims),
    )
    .expect("the run completes");

    // E4 emitted v2 bytes; E5 admitted them at the emitted `package_id`.
    assert!(!driven.emitted.bytes().is_empty());
    assert_eq!(
        driven.checked.package_id().digest.as_ref(),
        driven.emitted.package_id().hex()
    );
    assert_eq!(driven.checked, checked, "the same unit, the same package");

    let dispositions: Vec<serde_json::Value> = driven
        .settlements
        .iter()
        .map(|s| serde_json::to_value(&s.disposition).expect("dispositions serialize"))
        .collect();
    assert_eq!(dispositions[0]["disposition"], "supported");
    assert_eq!(dispositions[1]["disposition"], "unsupported");
    assert_eq!(dispositions[2]["disposition"], "invalid-request");
    assert_eq!(dispositions[2]["cause"]["cause"], "unknown-backend");
    assert_eq!(dispositions[2]["cause"]["backend"], "missing");

    assert_eq!(driven.routed.len(), 1);
    assert_eq!(driven.routed[0].request_index, 0);
    assert_eq!(driven.routed[0].backend().as_str(), "kani");
    assert_eq!(driven.routed[0].kind, BackendKind::Kani);

    // Step 7 is CG's own `generate_routed` over the routed item, with its
    // request index, IR node, wire backend and kind, and the pinned tools.
    let backend = kani_candidate(&driven);
    let pins = KaniToolPins::pinned();
    let expected = generate_routed(
        &checked,
        &[RoutedGenerationItem {
            request_index: 0,
            node_id: add,
            backend: backend.clone(),
            kind: BackendKind::Kani,
        }],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                claim_map: &claims,
                subject_path: "crate::subject",
                pins: &pins,
                unwind: 1,
                attestation: attestation(),
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation, expected);
    assert_eq!(driven.generation.items.len(), 1);
    assert_eq!(driven.generation.items[0].request_index, 0);
    assert_eq!(driven.generation.items[0].backend, backend);

    assert_eq!(
        recorder.calls(),
        [
            Call::Compile,
            Call::ReadPackage,
            Call::BackendKind("kani".to_owned()),
            Call::Negotiate { items: 3 },
            Call::Route {
                supported: 1,
                dispositions: 1,
            },
            Call::Generate { items: 1 },
        ]
    );
}

/// FR-001-AC-6 (TC-006): the generation output is keyed by the driver's
/// request index, ascending, with the unrouted item absent.
#[test]
fn tc_006_generation_output_ascends_by_request_index() {
    let checked = int_package().checked;
    let (add, add_wire) = expression(&checked, "binary");
    let (_, conversion_wire) = expression(&checked, "conversion");
    let claims = claim_map(&checked, &add);
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let items = [
        item_at(conversion_wire, Capability::ValueValidity, None),
        item(2, Capability::OperationContract, None),
        item_at(add_wire, Capability::ValueValidity, None),
    ];

    let driven = drive(
        &recorder,
        int_input(&packages),
        &registry,
        &items,
        kani_input(&claims),
    )
    .expect("the run completes");

    let routed: Vec<usize> = driven.routed.iter().map(|r| r.request_index).collect();
    assert_eq!(routed, [0, 2]);
    let generated: Vec<usize> = driven
        .generation
        .items
        .iter()
        .map(|item| item.request_index)
        .collect();
    assert_eq!(generated, [0, 2]);
    assert!(recorder.calls().contains(&Call::Generate { items: 2 }));
}

/// FR-001-AC-5 and FR-001-AC-6 (TC-007): a routed Kani item with no Kani
/// context refuses the run with CG's `MissingKindContext`, and a routed
/// item naming a node the package does not hold refuses before CG is
/// called.
#[test]
fn tc_007_generation_refusals_are_typed() {
    let checked = int_package().checked;
    let (_, add_wire) = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let recorder = Recorder::default();
    let refusal = drive(
        &recorder,
        int_input(&packages),
        &registry,
        &[item_at(add_wire, Capability::ValueValidity, None)],
        GenerationInput::default(),
    )
    .expect_err("no Kani context is supplied");
    assert!(
        matches!(
            refusal,
            DriverRefusal::Generation(RoutedGenerationError::MissingKindContext {
                kind: BackendKind::Kani
            })
        ),
        "{refusal:?}"
    );
    assert_eq!(refusal.stage(), DriverStage::Generation);
    assert_eq!(refusal.code(), "invalid-request");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
    assert_eq!(recorder.calls().last(), Some(&Call::Generate { items: 1 }));

    let recorder = Recorder::default();
    let dangling = WireNodeId::from_digest([1; 32]);
    let refusal = drive(
        &recorder,
        int_input(&packages),
        &registry,
        &[
            item(2, Capability::OperationContract, None),
            item_at(dangling, Capability::ValueValidity, None),
        ],
        GenerationInput::default(),
    )
    .expect_err("the routed node is not in the package");
    let DriverRefusal::UnknownNode {
        request_index,
        node,
    } = &refusal
    else {
        panic!("expected the unknown-node refusal, got {refusal:?}");
    };
    assert_eq!((*request_index, *node), (1, dangling));
    assert_eq!(refusal.stage(), DriverStage::Generation);
    assert_eq!(refusal.code(), "dangling_reference");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
    assert!(
        !recorder
            .calls()
            .iter()
            .any(|c| matches!(c, Call::Generate { .. })),
        "{:?}",
        recorder.calls()
    );
}

/// FR-001-AC-3 (TC-003): when no item settles `supported`, `route` is not
/// called at all.
#[test]
fn tc_003_no_supported_item_means_no_route_call() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();

    let driven = drive(
        &recorder,
        input(&packages),
        &registry,
        &[item(1, Capability::OperationContract, None)],
        GenerationInput::default(),
    )
    .expect("the run completes");

    assert!(driven.routed.is_empty());
    assert!(
        !recorder
            .calls()
            .iter()
            .any(|c| matches!(c, Call::Route { .. })),
        "{:?}",
        recorder.calls()
    );
}

/// FR-001-AC-5 (TC-004): the command-level refusals are typed, with a
/// stage, a catalog code and an outcome kind: a spine refusal, and a
/// registration refusal.
#[test]
fn tc_004_command_level_refusals_are_typed() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let broken = CompileInput {
        bytes: b"language \"ix:native\" edition \"1-draft\";\nfunction {",
        ..input(&packages)
    };

    let refusal = drive(
        &recorder,
        broken,
        &registry,
        &[],
        GenerationInput::default(),
    )
    .expect_err("the unit does not parse");
    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Source));
    assert_eq!(refusal.code(), "invalid_syntax");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
    assert_eq!(recorder.calls(), [Call::Compile]);

    // A stage limit is its own outcome kind: `not not not true` is four
    // deep, over a checking depth of 3.
    let deep = CompileInput {
        bytes: b"language \"ix:native\" edition \"1-draft\";\n\
            profile v = \"quire.value.complete/v1\" version \"1\" digest \
            \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
            function f using v(): Boolean pure { not not not true }\n",
        limits: SpineLimits {
            checking: CheckingLimits::new(u64::MAX, 3).expect("valid limits"),
            ..SpineLimits::default()
        },
        ..input(&packages)
    };
    let refusal = drive(
        &Recorder::default(),
        deep,
        &registry,
        &[],
        GenerationInput::default(),
    )
    .expect_err("depth 3 stops a body four deep");
    assert_eq!(refusal.kind(), OutcomeKind::Limit);
    assert_eq!(refusal.code(), "stage_limit_exceeded");
    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Check));

    let refusal = build_registry(&[
        manifest("kani", 1, &["value-validity"]),
        manifest("kani", 2, &["value-validity"]),
    ])
    .expect_err("one identity with two manifests conflicts");
    assert_eq!(refusal.stage(), DriverStage::Registry);
    assert_eq!(refusal.code(), "invalid_capability/duplicate-backend");

    let refusal = build_registry(&[manifest("kani", 1, &["not-a-kind"])])
        .expect_err("an unknown kind refuses the registration");
    assert_eq!(refusal.code(), "invalid_capability/unknown-kind");
}
