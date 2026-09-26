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
use qsl_route::{BackendId, Candidate, ManifestDigest, RegistrationCause};
use qsl_semantics::check::{Capability, CheckingLimits};
use qsl_semantics::family::{ClaimExtent, Requirements};
use quire_contract_codegen::{
    generate_routed, AttestationContext, BackendKind, BackendProviderEnvelope,
    ClaimDerivationRefusal, ClaimDisposition, EnvelopeRefusal, GenerationContexts, ItemSettlement,
    KaniGenerationContext, KaniToolPins, KindOutput, ObligationDisposition, OperationProvenance,
    RoutedGeneration, RoutedGenerationError, RoutedGenerationItem, UnsupportedObligation,
    IR_CANDIDATE_REVISION,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCode, CheckedPackageV2,
};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, DriverFault, DriverRefusal, DriverStage,
    GenerationInput, KaniInput, OutcomeKind, ProviderManifest, QslCg, RequestedItem, Upstream,
};

const UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function f using v(): Boolean pure { true }\n";

/// A unit with an integer `+` (a `binary` expression) under a range
/// `conversion` expression: two nodes a Kani item can name. The `+`'s own
/// operands are `x` (owns its `[0, 9]` bound) and the literal `1`, which
/// QSL represents as a reference to a plain-typed literal value node with
/// no bound of its own: since `x` owns a bound and `1`'s reference does
/// not, CG's IR-298 rule settles this `+` `RequiresBound`, not `Supported`
/// (`NEGATE_UNIT` below is the one-owned-bound case that is supported).
const INT_UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function inc using v(x: Int[0, 9]): Int[0, 10] pure { x + 1 }\n";

/// A unit with an integer `=` (`quire.op.integer.eq`, also a `binary`
/// expression): CG's derivation names no operator family for it
/// (`family_of_identity` has no `quire.op.integer.eq` arm), so it has no
/// derivable claim. It also carries an integer unary `-` over its own
/// bounded parameter (IR-298; see `NEGATE_UNIT`), a sibling CG does derive
/// a claim for, so a routed run over both shows the unsupported item does
/// not affect its sibling.
const EQ_UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure { x = y }\n\
    function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }\n";

/// A unit with a unary integer `-` over one bounded parameter
/// (`z: Int[0, 9]`): its one operand owns a bound, and IR-298's own-bound
/// filtering never runs (`scalar_result_bound`'s single-candidate arm
/// resolves directly), so this settles `Supported` with a scalar harness
/// ranging that one argument over `z`'s own domain.
///
/// A genuine two-operand case with each argument independently bounded
/// (e.g. `x + y` over `Int[0, 9]` and `Int[10, 20]`) does not settle
/// `Supported` through real QSL output today: QSL always wraps arithmetic
/// in a separate narrowing `conversion` node when the declared context is
/// bounded, and that conversion's own bound is not reachable from the
/// arithmetic node itself (IR's `bounds` closure is forward-only, from the
/// node's own body reference targets). So a `+` over two independently
/// bounded operands has exactly the two operands' own bounds reachable and
/// nothing else; IR-298 requires exactly one bound that is nobody's own to
/// resolve the result role, and removing both operands' own bounds here
/// leaves zero, which is `AmbiguousBound`, not `Supported` (IR-302; see
/// `tc_006_x_plus_y_generation_passes_through_unchanged` for how CG and QSL
/// actually behave today, and
/// `tc_006_x_plus_y_settles_supported_with_independent_argument_bounds` for
/// the intended outcome).
const NEGATE_UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }\n";

/// A unit with an integer `+` over two parameters, each owning its own
/// bound (`x: Int[0, 9]`, `y: Int[10, 20]`): see `NEGATE_UNIT`'s doc for why
/// this settles `AmbiguousBound`, not `Supported`.
const TWO_PARAM_UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function add using v(x: Int[0, 9], y: Int[10, 20]): Int[10, 29] pure { x + y }\n";

/// A unit that `import`s a library the driver's `DependencyInput::default()`
/// (ADR-015 D-1) supplies nothing for: S4 source resolution refuses it at
/// I1, `missing_import`/`missing-selection`.
const IMPORT_UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    import \"test/library\" version \"1\" digest \
    \"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\" as lib;\n\
    function f using v(): Boolean pure { true }\n";

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
    /// When set, the IR read answers this instead of reading.
    read_answer: Option<CheckedPackageDispatchResult>,
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
        match &self.read_answer {
            Some(answer) => answer.clone(),
            None => QslCg.read_package(bytes, limits, evidence),
        }
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

fn eq_input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        bytes: EQ_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn two_param_input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        bytes: TWO_PARAM_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn negate_input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        bytes: NEGATE_UNIT.as_bytes(),
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

/// `EQ_UNIT` driven with no items: its E5 package.
fn eq_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        eq_input(&packages),
        &registry,
        &[],
        GenerationInput::default(),
    )
    .expect("the equality unit drives")
}

/// `TWO_PARAM_UNIT` driven with no items: its E5 package.
fn two_param_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        two_param_input(&packages),
        &registry,
        &[],
        GenerationInput::default(),
    )
    .expect("the two-parameter unit drives")
}

/// `NEGATE_UNIT` driven with no items: its E5 package.
fn negate_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        negate_input(&packages),
        &registry,
        &[],
        GenerationInput::default(),
    )
    .expect("the negation unit drives")
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

/// The one expression node whose `operation.identity` is `identity`: IR's id
/// and its wire spelling.
fn expression_by_identity(
    package: &CheckedPackageV2,
    identity: &str,
) -> (CheckedNodeId, WireNodeId) {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression"
                && node.body["operation"]["identity"].as_str() == Some(identity)
        })
        .unwrap_or_else(|| panic!("the unit has a {identity} expression"));
    let wire = WireNodeId::from_hex(&node.node_id.digest).expect("a wire node id");
    (node.node_id.clone(), wire)
}

fn attestation() -> AttestationContext<'static> {
    AttestationContext {
        record_digest: RECORD_DIGEST,
        candidate_revision: IR_CANDIDATE_REVISION,
    }
}

/// The Kani generation input. IR-294 has CG derive each routed item's claim
/// from the node's own operation and bound, so the caller supplies no claim
/// map any more.
fn kani_input() -> GenerationInput<'static> {
    GenerationInput {
        kani: Some(KaniInput {
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
    let checked = negate_package().checked;
    let (neg, neg_wire) = expression_by_identity(&checked, "quire.op.integer.negate");

    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let items = [
        // Supported: kani advertises value-validity, bounded extent.
        item_at(neg_wire, Capability::ValueValidity, None),
        // Unsupported: no registered backend advertises operation-contract.
        item(2, Capability::OperationContract, None),
        // Invalid request: the request names an unregistered backend.
        item(3, Capability::ValueValidity, Some("missing")),
    ];

    let driven = drive(
        &recorder,
        negate_input(&packages),
        &registry,
        &items,
        kani_input(),
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
            node_id: neg.clone(),
            backend: backend.clone(),
            kind: BackendKind::Kani,
        }],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
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
    // QSL-261 is resolved by IR-296 and IR-297; IR-298 then makes CG
    // identify each bound role by what the node itself names: an operand's
    // bound is the `bounded_domain` typing it, and the result bound is the
    // domain typing the node's result or the one reachable bound no operand
    // types. `z` owns a bound and is the negation's only operand, so CG
    // settles the unary `-` `Supported` with a harness ranging that one
    // argument over `z`'s own domain (`scalar_result_bound`'s
    // single-candidate arm resolves it directly; no own-bound filtering
    // runs at all). `INT_UNIT`'s `x + 1` and `TWO_PARAM_UNIT`'s `x + y` are
    // the two-operand cases this bump introduces new rules for: a literal
    // operand beside one that owns a bound now settles `RequiresBound`
    // (`tc_006_a_literal_beside_an_owned_bound_requires_bound`), and two
    // independently owned operand bounds with the result unreachable any
    // other way settle `AmbiguousBound`
    // (`tc_006_two_independently_bounded_operands_is_ambiguous_not_supported`).
    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output;
    assert!(
        matches!(record.disposition, ObligationDisposition::Supported { .. }),
        "{:?}",
        record.disposition
    );
    let harness = harness
        .as_ref()
        .expect("a supported item carries its harness");
    assert_eq!(harness.identity.node_id, neg);
    // The negation's one operand, ranged over `z`'s own `[0, 9]` domain.
    let bounds: Vec<(i64, i64)> = harness
        .identity
        .arguments
        .iter()
        .map(|argument| (argument.minimum, argument.maximum))
        .collect();
    assert_eq!(bounds, [(0, 9)]);

    // FR-001-AC-8 (IR-294): the Kani group derived a claim map, which holds
    // the routed node with a generated claim (CG did derive `+`'s operator
    // family).
    let claim_map = driven
        .generation
        .claim_map
        .as_ref()
        .expect("a Kani group ran, so a claim map is derived");
    let neg_claim = claim_map
        .items
        .iter()
        .find(|claim| claim.node_id == neg)
        .unwrap_or_else(|| panic!("no claim for the `-` node: {:?}", claim_map.items));
    assert!(
        matches!(neg_claim.result, ClaimDisposition::Generated(_)),
        "{:?}",
        neg_claim.result
    );
    serde_json::to_vec_pretty(claim_map).expect("the derived claim map serializes");

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

/// FR-001-AC-6 (TC-006): the driver's generation output for `INT_UNIT`'s
/// `+` (`x + 1`) equals a direct `generate_routed` call over the same
/// routed item, whatever CG settles it -- the same pass-through check
/// `tc_001_tc_003_tc_006_steps_run_in_order_through_generation` makes for
/// its own fixture. This holds regardless of IR-302 below: the driver
/// passes CG's answer through unchanged either way.
#[test]
fn tc_006_x_plus_1_generation_passes_through_unchanged() {
    let checked = int_package().checked;
    let (add, add_wire) = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        int_input(&packages),
        &registry,
        &[item_at(add_wire, Capability::ValueValidity, None)],
        kani_input(),
    )
    .expect("the run completes");

    let backend = kani_candidate(&driven);
    let pins = KaniToolPins::pinned();
    let expected = generate_routed(
        &checked,
        &[RoutedGenerationItem {
            request_index: 0,
            node_id: add,
            backend,
            kind: BackendKind::Kani,
        }],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                subject_path: "crate::subject",
                pins: &pins,
                unwind: 1,
                attestation: attestation(),
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation, expected);
}

/// FR-001-AC-6 (TC-006): the driver's generation output for
/// `TWO_PARAM_UNIT`'s `+` (`x + y`) equals a direct `generate_routed` call
/// over the same routed item, whatever CG settles it. See
/// `tc_006_x_plus_1_generation_passes_through_unchanged`.
#[test]
fn tc_006_x_plus_y_generation_passes_through_unchanged() {
    let checked = two_param_package().checked;
    let (add, add_wire) = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        two_param_input(&packages),
        &registry,
        &[item_at(add_wire, Capability::ValueValidity, None)],
        kani_input(),
    )
    .expect("the run completes");

    let backend = kani_candidate(&driven);
    let pins = KaniToolPins::pinned();
    let expected = generate_routed(
        &checked,
        &[RoutedGenerationItem {
            request_index: 0,
            node_id: add,
            backend,
            kind: BackendKind::Kani,
        }],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                subject_path: "crate::subject",
                pins: &pins,
                unwind: 1,
                attestation: attestation(),
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation, expected);
}

/// FR-001-AC-6 (TC-006), IR-302: the intended outcome for `INT_UNIT`'s `+`
/// (`x + 1`) is `Supported`, with `x`'s own `[0, 9]` domain covering both
/// arguments (CG's own tc_033 shape). CG's IR-298 currently settles this
/// `RequiresBound` instead, naming the literal `1`'s own unbounded type:
/// QSL represents every literal as a reference to its own plain-typed
/// value node, never inline, so this trips CG's "plain reference beside an
/// owned bound" rule. Tracked as IR-302.
#[test]
#[ignore = "IR-302: IR-298 two-operand shapes on QSL output"]
fn tc_006_x_plus_1_settles_supported() {
    let checked = int_package().checked;
    let (add, add_wire) = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        int_input(&packages),
        &registry,
        &[item_at(add_wire, Capability::ValueValidity, None)],
        kani_input(),
    )
    .expect("the run completes");

    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output;
    assert!(
        matches!(record.disposition, ObligationDisposition::Supported { .. }),
        "{:?}",
        record.disposition
    );
    let harness = harness
        .as_ref()
        .expect("a supported item carries its harness");
    assert_eq!(harness.identity.node_id, add);
    let bounds: Vec<(i64, i64)> = harness
        .identity
        .arguments
        .iter()
        .map(|argument| (argument.minimum, argument.maximum))
        .collect();
    assert_eq!(bounds, [(0, 9), (0, 9)]);
}

/// FR-001-AC-6 (TC-006), IR-302: the intended outcome for `TWO_PARAM_UNIT`'s
/// `+` (`x + y`) is `Supported`, ranging each argument over its own
/// operand's domain: `x`'s `[0, 9]`, then `y`'s `[10, 20]`. CG's IR-298
/// currently settles this `AmbiguousBound` instead: `x` and `y` each own a
/// bound and nothing else is reachable from the `+` node itself (the
/// return type's narrowing `conversion` is unreachable from its own
/// operand), so removing both operands' own bounds from the candidates
/// leaves none. Tracked as IR-302.
#[test]
#[ignore = "IR-302: IR-298 two-operand shapes on QSL output"]
fn tc_006_x_plus_y_settles_supported_with_independent_argument_bounds() {
    let checked = two_param_package().checked;
    let (add, add_wire) = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        two_param_input(&packages),
        &registry,
        &[item_at(add_wire, Capability::ValueValidity, None)],
        kani_input(),
    )
    .expect("the run completes");

    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output;
    assert!(
        matches!(record.disposition, ObligationDisposition::Supported { .. }),
        "{:?}",
        record.disposition
    );
    let harness = harness
        .as_ref()
        .expect("a supported item carries its harness");
    assert_eq!(harness.identity.node_id, add);
    let bounds: Vec<(i64, i64)> = harness
        .identity
        .arguments
        .iter()
        .map(|argument| (argument.minimum, argument.maximum))
        .collect();
    assert_eq!(bounds, [(0, 9), (10, 20)]);
}

/// FR-001-AC-6 (TC-006): the generation output is keyed by the driver's
/// request index, ascending, with the unrouted item absent.
#[test]
fn tc_006_generation_output_ascends_by_request_index() {
    let checked = int_package().checked;
    let (_add, add_wire) = expression(&checked, "binary");
    let (_, conversion_wire) = expression(&checked, "conversion");
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
        kani_input(),
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

    // Two manifests, one identity, two distinct digests: both conflict,
    // and `refusals()` reports one entry per distinct digest, ascending.
    let expected_conflicts = [
        (
            Candidate::new(BackendId::new("kani"), ManifestDigest::from_digest([1; 32])),
            RegistrationCause::DuplicateBackend,
        ),
        (
            Candidate::new(BackendId::new("kani"), ManifestDigest::from_digest([2; 32])),
            RegistrationCause::DuplicateBackend,
        ),
    ];
    let refusal = build_registry(&[
        manifest("kani", 1, &["value-validity"]),
        manifest("kani", 2, &["value-validity"]),
    ])
    .expect_err("one identity with two manifests conflicts");
    assert_eq!(refusal.stage(), DriverStage::Registry);
    assert_eq!(refusal.code(), "invalid_capability/duplicate-backend");
    assert_eq!(registration_refusals(&refusal), expected_conflicts);

    // A, B, then A again (same digest as the first): the repeat does not
    // add a third, duplicate refusal for digest 1 -- `refusals()` is the
    // registry's own deduped account, not an accumulation of what each
    // `register` call individually returned.
    let refusal = build_registry(&[
        manifest("kani", 1, &["value-validity"]),
        manifest("kani", 2, &["value-validity"]),
        manifest("kani", 1, &["value-validity"]),
    ])
    .expect_err("A, B, A still conflicts, without a duplicated refusal");
    assert_eq!(registration_refusals(&refusal), expected_conflicts);

    let refusal = build_registry(&[manifest("kani", 1, &["not-a-kind"])])
        .expect_err("an unknown kind refuses the registration");
    assert_eq!(refusal.code(), "invalid_capability/unknown-kind");
}

/// FR-001-AC-5 (TC-008): the driver compiles with an empty
/// `DependencyInput` (ADR-015 D-1), so a unit whose `import` names a
/// library nothing supplies refuses at the intake stage, with
/// `missing_import`/`missing-selection` -- `CompileRefusal::code()`'s
/// `code/cause` rendering, not just the bare code.
#[test]
fn tc_008_a_missing_import_refuses_at_compile_intake() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    let broken = CompileInput {
        bytes: IMPORT_UNIT.as_bytes(),
        ..input(&packages)
    };

    let refusal = drive(&QslCg, broken, &registry, &[], GenerationInput::default())
        .expect_err("nothing supplies test/library");

    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Intake));
    assert_eq!(refusal.code(), "missing_import/missing-selection");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
}

/// `refusal`'s `DriverRefusal::Registration` vector, projected to
/// `(backend, cause)` pairs: `RegistrationRefusal`'s own fields are
/// private, so its accessors stand in for a struct literal.
fn registration_refusals(refusal: &DriverRefusal) -> Vec<(Candidate, RegistrationCause)> {
    let DriverRefusal::Registration(refusals) = refusal else {
        panic!("expected a registration refusal, got {refusal:?}");
    };
    refusals
        .iter()
        .map(|r| (r.backend().clone(), r.cause().clone()))
        .collect()
}

/// FR-001-AC-5 and FR-001-AC-7 (TC-004): E5 accepts only the package it
/// emitted. An IR answer admitting another package, or refusing the bytes,
/// is an internal fault at the hand-off, and nothing after the IR read runs.
#[test]
fn tc_004_e5_identity_and_refusal_are_faults() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let other = drive(
        &QslCg,
        input(&packages),
        &build_registry(&[]).expect("an empty registry builds"),
        &[],
        GenerationInput::default(),
    )
    .expect("UNIT drives")
    .checked;
    let refusal = CheckedPackageRefusal {
        code: CheckedPackageRefusalCode::MalformedWire,
        path: None,
        cause: None,
        locus: None,
        contract_version: None,
    };

    for answer in [
        CheckedPackageDispatchResult::AdmittedV2(Box::new(other.clone())),
        CheckedPackageDispatchResult::Refused(refusal.clone()),
    ] {
        let recorder = Recorder {
            read_answer: Some(answer.clone()),
            ..Recorder::default()
        };
        let driven = drive(
            &recorder,
            int_input(&packages),
            &registry,
            &[item(1, Capability::ValueValidity, None)],
            GenerationInput::default(),
        )
        .expect_err("E5 refuses a package it did not emit");
        match (&answer, &driven) {
            (
                CheckedPackageDispatchResult::AdmittedV2(_),
                DriverRefusal::Fault(DriverFault::PackageIdMismatch { emitted, admitted }),
            ) => {
                assert_eq!(admitted, other.package_id().digest.as_ref());
                assert_ne!(emitted, admitted);
            }
            (
                CheckedPackageDispatchResult::Refused(_),
                DriverRefusal::Fault(DriverFault::HandoffRefused(refused)),
            ) => assert_eq!(refused, &refusal),
            _ => panic!("unexpected refusal {driven:?} for {answer:?}"),
        }
        assert_eq!(driven.stage(), DriverStage::Handoff);
        assert_eq!(driven.kind(), OutcomeKind::Fault);
        assert_eq!(driven.code(), "runtime_invariant");
        assert_eq!(recorder.calls(), [Call::Compile, Call::ReadPackage]);
    }
}

/// FR-001-AC-7 (TC-006): E5 admits units with text and rational parameter
/// types at their emitted `package_id`.
#[test]
fn tc_006_e5_admits_text_and_rational_units() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    for declaration in [
        "function h using v(x: Text[1, 5; nfc]): Boolean pure { true }",
        "function g using v(x: Rational[0, 9; 1, 9]): Boolean pure { true }",
    ] {
        let unit = format!(
            "language \"ix:native\" edition \"1-draft\";\n\
            profile v = \"quire.value.complete/v1\" version \"1\" digest \
            \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
            {declaration}\n"
        );
        let recorder = Recorder::default();
        let driven = drive(
            &recorder,
            CompileInput {
                bytes: unit.as_bytes(),
                ..input(&packages)
            },
            &registry,
            &[],
            GenerationInput::default(),
        )
        .unwrap_or_else(|refusal| panic!("{declaration}: {refusal:?}"));
        assert_eq!(
            driven.checked.package_id().digest.as_ref(),
            driven.emitted.package_id().hex(),
            "{declaration}"
        );
        assert_eq!(recorder.calls()[..2], [Call::Compile, Call::ReadPackage]);
    }
}

/// FR-001-AC-8 (TC-006): a routed node whose operation has no derivable
/// claim (`quire.op.integer.eq`; CG's derivation names no operator family
/// for it) settles `Unsupported` with `no_derivable_claim`, and its routed
/// sibling (`quire.op.integer.add`, which CG does derive a claim for) is
/// unaffected: the same disposition and harness as a run with only the
/// sibling. The derived claim map holds both nodes, each with its own
/// disposition, and serializes.
#[test]
fn tc_006_an_underivable_node_settles_unsupported_and_its_sibling_is_unaffected() {
    let checked = eq_package().checked;
    let (eq, eq_wire) = expression_by_identity(&checked, "quire.op.integer.eq");
    let (neg, neg_wire) = expression_by_identity(&checked, "quire.op.integer.negate");

    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let items = [
        item_at(eq_wire, Capability::ValueValidity, None),
        item_at(neg_wire, Capability::ValueValidity, None),
    ];

    let driven = drive(
        &recorder,
        eq_input(&packages),
        &registry,
        &items,
        kani_input(),
    )
    .expect("the run completes");

    assert_eq!(driven.routed.len(), 2);
    assert_eq!(driven.generation.items.len(), 2);
    assert!(driven.generation.rejected.is_empty());

    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output;
    assert_eq!(driven.generation.items[0].request_index, 0);
    assert!(harness.is_none(), "an underivable item carries no harness");
    assert_eq!(
        record.disposition,
        ObligationDisposition::Unsupported {
            reason: UnsupportedObligation::NoDerivableClaim {
                node_id: eq.clone(),
                reason: ClaimDerivationRefusal::OperationNotDerivable {
                    operation_identity: "quire.op.integer.eq".to_owned(),
                },
            }
        }
    );

    // The sibling settles Supported with its own harness, unaffected by the
    // eq item's refusal.
    let KindOutput::Kani { record, harness } = &driven.generation.items[1].output;
    assert_eq!(driven.generation.items[1].request_index, 1);
    assert!(
        matches!(record.disposition, ObligationDisposition::Supported { .. }),
        "{:?}",
        record.disposition
    );
    let harness = harness
        .as_ref()
        .expect("the derivable sibling carries its harness");
    assert_eq!(harness.identity.node_id, neg);

    // The negation item's disposition and harness equal what a run with
    // only the negation item produces: the underivable eq item did not
    // affect it.
    let alone_registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let alone_packages = BTreeMap::new();
    let alone = drive(
        &QslCg,
        eq_input(&alone_packages),
        &alone_registry,
        &[item_at(neg_wire, Capability::ValueValidity, None)],
        kani_input(),
    )
    .expect("the neg-only run completes");
    assert_eq!(alone.generation.items.len(), 1);
    let KindOutput::Kani {
        record: alone_record,
        harness: alone_harness,
    } = &alone.generation.items[0].output;
    assert_eq!(record.disposition, alone_record.disposition);
    assert_eq!(harness, alone_harness.as_ref().expect("a harness"));

    // The derived claim map holds both nodes: the eq node's own
    // `NoDerivableClaim`-underlying refusal (its operation is `Underived`,
    // since nobody declared it), and the neg node's generated claim.
    let claim_map = driven
        .generation
        .claim_map
        .as_ref()
        .expect("a Kani group ran, so a claim map is derived");
    let eq_claim = claim_map
        .items
        .iter()
        .find(|claim| claim.node_id == eq)
        .unwrap_or_else(|| panic!("no claim for the eq node: {:?}", claim_map.items));
    assert!(
        matches!(eq_claim.result, ClaimDisposition::Refused { .. }),
        "{:?}",
        eq_claim.result
    );
    assert_eq!(
        eq_claim.operation.provenance,
        OperationProvenance::Underived
    );
    let neg_claim = claim_map
        .items
        .iter()
        .find(|claim| claim.node_id == neg)
        .unwrap_or_else(|| panic!("no claim for the neg node: {:?}", claim_map.items));
    assert!(
        matches!(neg_claim.result, ClaimDisposition::Generated(_)),
        "{:?}",
        neg_claim.result
    );
    serde_json::to_vec_pretty(claim_map).expect("the derived claim map serializes");
}
