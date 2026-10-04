// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001 acceptance tests: the driver's step order, the MD-2 refusal,
//! routing after negotiation, the E5 hand-off and CG generation. Every call goes to the real QSL and CG
//! functions; `Recorder` only logs the order in which the driver makes them.
//!
//! QSL-266 (#459) keys `CheckedGraph::requirements()` by one record per
//! scalar operation application. The driver writes its request items from
//! those records (`items_from_requirements`, FR-075), not from a
//! caller-supplied list, so every test here drives a real unit and reads
//! its items back off the package the unit compiles to, rather than
//! constructing a `RequestedItem` naming an arbitrary node.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::LazyLock;

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{CheckedUnit, CompileRefusal, EmittedUnit, SpineLimits, SpineStage};
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{BackendId, Candidate, ManifestDigest, RegistrationCause};
use qsl_semantics::value::{CatalogRole, DefinitionLock};
use quire_contract_codegen::{
    generate_routed, BackendKind, BackendProviderEnvelope, Candidates, Cause,
    ClaimDerivationRefusal, ClaimDisposition, DerivedDomain, Disposition, EnvelopeRefusal,
    GenerationContexts, ItemSettlement, KaniGenerationContext, KindOutput, ObligationDisposition,
    OperationProvenance, RoutedGeneration, RoutedGenerationError, RoutedGenerationItem,
    UnsupportedObligation,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageRefusal, CheckedPackageRefusalCode, CheckedPackageV2,
};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, DriverFault, DriverRefusal, DriverStage,
    GenerationInput, KaniInput, OutcomeKind, ProviderManifest, QslCg, Upstream,
};

/// A unit: the `ix:native` header, the `v` profile selecting QSL's own
/// `DefinitionLock` root row (read from QSL, never copied here), then `body`.
fn unit(body: &str) -> String {
    let lock = DefinitionLock::pinned();
    let root = lock.entry(CatalogRole::Root);
    format!(
        "language \"ix:native\" edition \"1-draft\";\n\
         profile v = \"{}\";\n{body}",
        root.identity
    )
}

static UNIT: LazyLock<String> =
    LazyLock::new(|| unit("function f using v(): Boolean pure { true }\n"));

/// A unit with an integer `+` (a `binary` expression) under a range
/// `conversion` expression. The narrowing `conversion` is not a scalar
/// operation application (`is_scalar_application` excludes
/// `quire.op.numeric.narrow` by name) and so is not itself a requirement
/// record; only the `+` is. The `+`'s own operands are `x` (owns its
/// `[0, 9]` bound) and the literal `1`, which
/// QSL represents as a reference to its own plain-typed `value` node: CG's
/// IR-302 reads that reference as a literal operand (its body decides it,
/// not its type), owning no bound and constrained to exactly its own
/// value, so this `+` settles `Supported` (`NEGATE_UNIT` below is the
/// one-owned-bound unary case, also `Supported`). It has one requirement
/// record: the `+`.
static INT_UNIT: LazyLock<String> =
    LazyLock::new(|| unit("function inc using v(x: Int[0, 9]): Int[0, 10] pure { x + 1 }\n"));

/// A unit with an integer `=` (`quire.op.integer.eq`, also a `binary`
/// expression): CG's derivation names no operator family for it
/// (`family_of_identity` has no `quire.op.integer.eq` arm), so it has no
/// derivable claim. It also carries an integer unary `-` over its own
/// bounded parameter (IR-298; see `NEGATE_UNIT`), a sibling CG does derive
/// a claim for, so a routed run over both shows the unsupported item does
/// not affect its sibling. Both `=` and `-` are scalar operation
/// applications, so this unit has two requirement records, keyed in
/// occurrence order: the `=` node's digest sorts first.
static EQ_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit(
        "function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure { x = y }\n\
    function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }\n",
    )
});

/// A unit with a unary integer `-` over one bounded parameter
/// (`z: Int[0, 9]`): its one operand owns a bound, and IR-298's own-bound
/// filtering never runs (`scalar_result_bound`'s single-candidate arm
/// resolves directly), so this settles `Supported` with a scalar harness
/// ranging that one argument over `z`'s own domain. One requirement
/// record: the `-`.
///
/// A genuine two-operand case with each argument independently bounded
/// (e.g. `x + y` over `Int[0, 9]` and `Int[10, 20]`) also settles
/// `Supported`: QSL always wraps arithmetic in a separate narrowing
/// `conversion` node when the declared context is bounded, so the `+`
/// node's own type is the plain scalar type, and only that conversion
/// consumes it. IR-302 reads the result bound of a scalar-typed node
/// consumed only by narrowing conversions as the one bound they narrow it
/// to (see `tc_006_x_plus_y_settles_supported_with_independent_argument_bounds`).
static NEGATE_UNIT: LazyLock<String> =
    LazyLock::new(|| unit("function negate using v(z: Int[0, 9]): Int[-9, 0] pure { -z }\n"));

/// A unit with an integer `+` over two parameters, each owning its own
/// bound (`x: Int[0, 9]`, `y: Int[10, 20]`): see `NEGATE_UNIT`'s doc for why
/// this settles `Supported`, ranging each argument over its own operand's
/// domain. One requirement record: the `+`.
static TWO_PARAM_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit("function add using v(x: Int[0, 9], y: Int[10, 20]): Int[10, 29] pure { x + y }\n")
});

/// A unit with an integer `+` whose one operand's domain (`x: Int[0, 9]`)
/// puts every exact result (`[100, 109]`) outside the declared result bound
/// (`Int[0, 10]`), with no conditional to hide it behind: QSL's own checker
/// proves the identical disjoint-interval fact and refuses to emit the
/// package at all (`Unproved(Range { .. })`, at the check stage), before CG
/// is ever called. This is one shape, not every shape: `DEAD_BRANCH_UNIT`
/// below reaches CG's own `Unsupported`/`ResultBoundUnreachable` for the
/// identical reachable range, because QSL does not discharge that
/// coercion at all inside a branch it proves dead.
static OVERFLOW_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit("function unreachable_result using v(x: Int[0, 9]): Int[0, 10] pure { x + 100 }\n")
});

/// A unit whose `+` (`x + 100`) is the same shape `OVERFLOW_UNIT` declares,
/// but reached only through an `if` branch QSL's checker proves dead
/// (`x > 20` never holds over `x: Int[0, 9]`): QSL's `Facts::walk` never
/// discharges a coercion inside a branch it proves unreachable, so the
/// package still emits. CG ignores path conditions (`reachable_results`
/// ranges every operand over its full declared domain, live branch or
/// dead), so routing this `+` to Kani settles the same
/// `Unsupported`/`ResultBoundUnreachable` `OVERFLOW_UNIT`'s direct case
/// never reaches CG to report. The `if` guard (`x > 20`) is itself a scalar
/// comparison, so this unit has two requirement records: the guard and the
/// `+`, keyed in occurrence order (the `+`'s node digest sorts first).
static DEAD_BRANCH_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit("function unreachable_branch using v(x: Int[0, 9]): Int[0, 10] pure { if x > 20 then x + 100 else 0 }\n")
});

/// A unit that `import`s a library the driver's `DependencyInput::default()`
/// (ADR-015 D-1) supplies nothing for: S4 source resolution refuses it at
/// I1, `missing_import`/`missing-selection`.
static IMPORT_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit("import \"test/library\" as lib;\nfunction f using v(): Boolean pure { true }\n")
});

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
    /// When set, `negotiate` rewrites the real settlement at this request
    /// index to `unsupported`, as if no registered backend advertised its
    /// kind. Real input cannot build a mixed settlement set on its own --
    /// every requirement record settles the same kind, `value-validity`,
    /// and an `Int` parameter with no declared range does not parse -- so
    /// this is the seam a test uses to build one (SR-013 FND-001).
    unsupported_at: Option<usize>,
    /// The envelope `negotiate` was last called with, captured so a test can
    /// read `named`'s effect on each item's own `candidates` -- the
    /// `items_from_requirements` half of `named` -- independently of
    /// whatever CG's negotiation settles from `named_backend` (the
    /// `envelope` half). Every real settlement re-derives an unregistered
    /// named backend's cause from `named_backend` alone
    /// (`unroutable_named_backend`), so a settlement-only test cannot tell
    /// the two uses of `named` apart (SR-013 FND-007, SR-014 FND-006).
    envelope: RefCell<Option<BackendProviderEnvelope>>,
}

impl Recorder {
    fn calls(&self) -> Vec<Call> {
        self.calls.borrow().clone()
    }

    fn envelope(&self) -> BackendProviderEnvelope {
        self.envelope
            .borrow()
            .clone()
            .expect("negotiate was called")
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
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal> {
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
        *self.envelope.borrow_mut() = Some(envelope.clone());
        let mut settlements = QslCg.negotiate(envelope)?;
        if let Some(index) = self.unsupported_at {
            if let Some(settlement) = settlements.get_mut(index) {
                let kind = settlement
                    .kind
                    .expect("a value-validity item settles with a kind");
                settlement.disposition = Disposition::Unsupported {
                    cause: Cause::UnsupportedRequestedCapability {
                        kind,
                        backend: None,
                    },
                };
            }
        }
        Ok(settlements)
    }

    fn route<'d>(&self, dispositions: &'d [RouteDisposition]) -> Vec<Option<&'d Candidate>> {
        let supported = dispositions
            .iter()
            .filter(|d| matches!(d, RouteDisposition::Supported(_)))
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

fn overflow_input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        bytes: OVERFLOW_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn dead_branch_input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        bytes: DEAD_BRANCH_UNIT.as_bytes(),
        ..input(packages)
    }
}

/// `INT_UNIT` driven with an empty registry: its E5 package.
fn int_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        int_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the integer unit drives")
}

/// `EQ_UNIT` driven with an empty registry: its E5 package.
fn eq_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        eq_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the equality unit drives")
}

/// `TWO_PARAM_UNIT` driven with an empty registry: its E5 package.
fn two_param_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        two_param_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the two-parameter unit drives")
}

/// `NEGATE_UNIT` driven with an empty registry: its E5 package.
fn negate_package() -> Driven {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    drive(
        &QslCg,
        negate_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the negation unit drives")
}

/// The one expression node of `form` in `package`: IR's id.
fn expression(package: &CheckedPackageV2, form: &str) -> CheckedNodeId {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "expression" && node.semantic_form.as_ref() == form)
        .unwrap_or_else(|| panic!("the unit has a {form} expression"));
    node.node_id.clone()
}

/// The one expression node whose `operation.identity` is `identity`: IR's id.
fn expression_by_identity(package: &CheckedPackageV2, identity: &str) -> CheckedNodeId {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression"
                && node.body["operation"]["identity"].as_str() == Some(identity)
        })
        .unwrap_or_else(|| panic!("the unit has a {identity} expression"));
    node.node_id.clone()
}

/// The one `bounded_domain`/`integer_range` node whose literal `min`/`max`
/// values are `lower`/`upper`: the node a Kani derivation names as an
/// operand's, or a comparison's, declared bound (`DerivedDomain::IntegerRange`).
fn integer_range_bound(package: &CheckedPackageV2, lower: &str, upper: &str) -> CheckedNodeId {
    let node = package
        .graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "bounded_domain"
                && node.semantic_form.as_ref() == "integer_range"
                && member_literal(&node.body, "min") == Some(lower)
                && member_literal(&node.body, "max") == Some(upper)
        })
        .unwrap_or_else(|| panic!("the package has an integer_range[{lower}, {upper}] bound"));
    node.node_id.clone()
}

/// The literal text value of member `name` in an `aggregate` node's `body`.
fn member_literal<'a>(body: &'a serde_json::Value, name: &str) -> Option<&'a str> {
    body["members"]
        .as_array()?
        .iter()
        .find(|member| member["name"].as_str() == Some(name))?
        .get("value")?
        .get("value")?
        .as_str()
}

/// The Kani generation input. IR-294 has CG derive each routed item's claim
/// from the node's own operation and bound, so the caller supplies no claim
/// map any more.
fn kani_input() -> GenerationInput<'static> {
    GenerationInput {
        kani: Some(KaniInput {
            subject_path: "crate::subject",
            unwind: 1,
        }),
    }
}

/// The kani backend as `route` returned it, in CG's wire spelling.
fn kani_candidate(driven: &Driven) -> quire_contract_codegen::Candidate {
    let candidate = &driven.routed[0].candidate;
    quire_contract_codegen::Candidate {
        identity: candidate.id().as_str().to_owned(),
    }
}

/// FR-001-AC-2 (TC-002): a registered `BackendId` with no CG kind refuses
/// the run with `invalid_capability`/`unknown-backend`, naming every
/// unconverted id in bytewise order, and nothing is negotiated or routed.
/// This holds independently of the compiled unit's own items: `UNIT`
/// compiles with no scalar operation applications at all, so it writes no
/// requirement records, and the MD-2 conversion still runs over every
/// *registered* backend.
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
        None,
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

/// FR-001-AC-4, FR-001-AC-3 and FR-001-AC-6 (TC-001, TC-003, TC-006): the
/// driver compiles and emits, hands the bytes to IR, converts every
/// backend, negotiates the unit's one requirement record once, routes it
/// (it settles `supported`), then generates for it through CG.
/// `NEGATE_UNIT` writes exactly one requirement record (its `-`), routed
/// from `CheckedGraph::requirements()` -- no item is supplied by this test.
#[test]
fn tc_001_tc_006_steps_run_in_order_through_generation() {
    let checked = negate_package().checked;
    let neg = expression_by_identity(&checked, "quire.op.integer.negate");

    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();

    let driven = drive(
        &recorder,
        negate_input(&packages),
        &registry,
        None,
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
    assert_eq!(dispositions.len(), 1, "one requirement record: the `-`");
    assert_eq!(dispositions[0]["disposition"], "supported");

    assert_eq!(driven.routed.len(), 1);
    assert_eq!(driven.routed[0].request_index, 0);
    assert_eq!(driven.routed[0].backend().as_str(), "kani");
    assert_eq!(driven.routed[0].kind, BackendKind::Kani);

    // Step 7 is CG's own `generate_routed` over the routed item, with its
    // request index, IR node, wire backend and kind.
    let backend = kani_candidate(&driven);
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
                unwind: 1,
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
    // the two-operand cases IR-302 fixes (see `tc_006_x_plus_1_settles_supported`
    // and `tc_006_x_plus_y_settles_supported_with_independent_argument_bounds`).
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
    // the routed node with a generated claim.
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
            Call::Negotiate { items: 1 },
            Call::Route {
                supported: 1,
                dispositions: 1,
            },
            Call::Generate { items: 1 },
        ]
    );
}

/// FR-001-AC-5 (TC-004, request `named_backend`): a request that names a
/// backend the registry does not hold settles `invalid-request`, naming the
/// backend, for every item -- `qsl_route::Registry::candidates` returns
/// `CandidateOutcome::UnknownBackend` regardless of capability kind. Nothing
/// settled this way is routed or generated.
///
/// `named` reaches CG two ways from `drive` (SR-013 FND-007, SR-014
/// FND-006): threaded into `items_from_requirements`, it is what
/// `Registry::candidates` narrows this item's own candidate set against,
/// which becomes the wire item's `candidates`; threaded into `envelope`
/// separately, it is `named_backend` on that same wire item. A settlement
/// alone cannot tell the two apart here: CG's `negotiate_item` calls
/// `unroutable_named_backend(item.named_backend)` and settles
/// `invalid-request`/`unknown-backend { backend: "missing" }` straight from
/// `named_backend`, before it ever looks at `candidates`
/// (`unroutable_named_backend`, capability.rs). So this also captures the
/// envelope `negotiate` was called with, through the `Recorder` seam, and
/// asserts the item's own `candidates` directly: `Candidates::UnknownBackend
/// ("missing")`, which only `items_from_requirements` threading `named`
/// through `Registry::candidates` can produce (dropping `named` there
/// leaves `kani` as the sole advertiser of `value-validity`, giving
/// `Candidates::Set([kani])` instead, with the settlement unchanged).
#[test]
fn tc_001_a_named_unregistered_backend_settles_invalid_request() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let missing = BackendId::new("missing");

    let driven = drive(
        &recorder,
        negate_input(&packages),
        &registry,
        Some(&missing),
        GenerationInput::default(),
    )
    .expect("the run completes");

    let dispositions: Vec<serde_json::Value> = driven
        .settlements
        .iter()
        .map(|s| serde_json::to_value(&s.disposition).expect("dispositions serialize"))
        .collect();
    assert_eq!(dispositions.len(), 1);
    assert_eq!(dispositions[0]["disposition"], "invalid-request");
    assert_eq!(dispositions[0]["cause"]["cause"], "unknown-backend");
    assert_eq!(dispositions[0]["cause"]["backend"], "missing");

    let envelope = recorder.envelope();
    assert_eq!(envelope.items.len(), 1);
    assert_eq!(
        envelope.items[0].candidates,
        Candidates::UnknownBackend("missing".to_owned())
    );

    assert!(driven.routed.is_empty());
    assert_eq!(driven.generation, RoutedGeneration::default());
    let calls = recorder.calls();
    assert!(
        !calls
            .iter()
            .any(|c| matches!(c, Call::Route { .. } | Call::Generate { .. })),
        "{calls:?}"
    );
}

/// FR-001-AC-5 (TC-004, request `named_backend`): a request that names a
/// backend that *is* registered, but does not itself advertise the item's
/// kind, settles `unsupported`, naming that backend. `named` is one value
/// used twice in `drive` (SR-014 FND-004): threaded into
/// `items_from_requirements`, it is the backend `Registry::candidates`
/// narrows the item's own candidate set against; threaded into
/// `envelope`, it is `named_backend` on the wire item, which is what CG's
/// own `negotiate_item` reads back verbatim into
/// `Cause::UnsupportedRequestedCapability { backend, .. }` for an empty
/// candidate set (`[] => Unsupported { cause: UnsupportedRequestedCapability
/// { backend: item.named_backend.clone(), .. } }`, CG's own source). If
/// `envelope` dropped its own `named` (always writing `named_backend:
/// None`), `dispositions[0]["cause"]["backend"]` would be absent here
/// instead of naming `kani`, even though the candidate set is unaffected --
/// this is the half `tc_001_a_named_unregistered_backend_settles_invalid_request`
/// does not cover, since that test's `UnknownBackend` cause comes from the
/// `candidates` variant itself, not from `named_backend`.
///
/// `items_from_requirements`'s own use of `named` (the candidate-set
/// narrowing) is *not* proven by that same unregistered-backend test's
/// settlement: CG's `negotiate_item` calls `unroutable_named_backend
/// (item.named_backend)` and settles `invalid-request`/`unknown-backend`
/// straight from `named_backend`, before it consults `candidates` at all,
/// so the settlement alone cannot distinguish `candidates ==
/// UnknownBackend("missing")` from `candidates == Set([kani])` (SR-013
/// FND-007). That test instead asserts the item's own `candidates`
/// directly, captured from the envelope through the `Recorder` seam. A
/// single test still cannot exercise dropping `named` from both call sites
/// at once *through a settlement*: CG's `BackendKind::ALL` names exactly
/// one kind (`Kani`) today, so the registry can never hold a second
/// backend for `named` to narrow *away from* without MD-2 refusing the run
/// first (`tc_002_unknown_backend_refuses_before_negotiation_in_bytewise_order`).
#[test]
fn tc_004_a_named_registered_backend_that_does_not_advertise_the_kind_settles_unsupported() {
    let registry = build_registry(&[manifest("kani", 7, &["operation-contract"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let kani = BackendId::new("kani");

    let driven = drive(
        &recorder,
        negate_input(&packages),
        &registry,
        Some(&kani),
        GenerationInput::default(),
    )
    .expect("the run completes");

    let dispositions: Vec<serde_json::Value> = driven
        .settlements
        .iter()
        .map(|s| serde_json::to_value(&s.disposition).expect("dispositions serialize"))
        .collect();
    assert_eq!(dispositions.len(), 1);
    assert_eq!(dispositions[0]["disposition"], "unsupported");
    assert_eq!(
        dispositions[0]["cause"]["cause"],
        "unsupported-requested-capability"
    );
    assert_eq!(dispositions[0]["cause"]["backend"], "kani");

    assert!(driven.routed.is_empty());
    let calls = recorder.calls();
    assert!(
        !calls
            .iter()
            .any(|c| matches!(c, Call::Route { .. } | Call::Generate { .. })),
        "{calls:?}"
    );
}

/// FR-001-AC-6 (TC-006): the driver's generation output for `INT_UNIT`'s
/// `+` (`x + 1`) equals a direct `generate_routed` call over the same
/// routed item, whatever CG settles it -- the same pass-through check
/// `tc_001_tc_006_steps_run_in_order_through_generation` makes for its own
/// fixture. This holds regardless of IR-302 below: the driver passes CG's
/// answer through unchanged either way.
#[test]
fn tc_006_x_plus_1_generation_passes_through_unchanged() {
    let checked = int_package().checked;
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(&QslCg, int_input(&packages), &registry, None, kani_input())
        .expect("the run completes");

    let backend = kani_candidate(&driven);
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
                unwind: 1,
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
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        two_param_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    let backend = kani_candidate(&driven);
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
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation, expected);
}

/// FR-001-AC-6 (TC-006), IR-302: `INT_UNIT`'s `+` (`x + 1`) settles
/// `Supported`. CG's IR-302 reads a `reference` to a `value` node whose
/// body is a literal as a literal operand (QSL's own shape for `1`), not a
/// plain-typed unbounded reference: the literal owns no bound and is
/// constrained to its own value, so only `x`'s own `[0, 9]` domain ranges
/// the first argument.
#[test]
fn tc_006_x_plus_1_settles_supported() {
    let checked = int_package().checked;
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(&QslCg, int_input(&packages), &registry, None, kani_input())
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
    assert_eq!(harness.identity.operation_identity, "quire.op.integer.add");
    // `x` ranges its own `[0, 9]` domain; the literal `1` is held at exactly
    // its own value, not ranged over `1`'s plain scalar type.
    let bounds: Vec<(i64, i64)> = harness
        .identity
        .arguments
        .iter()
        .map(|argument| (argument.minimum, argument.maximum))
        .collect();
    assert_eq!(bounds, [(0, 9), (1, 1)]);
    // The declared result bound, `Int[0, 10]`: not in `ScalarObligationIdentity`
    // itself, only rendered into the harness source's non-vacuity assertion.
    assert!(
        harness
            .rust
            .contents
            .contains("integer(\"0\")?, integer(\"10\")?"),
        "{}",
        harness.rust.contents
    );
}

/// FR-001-AC-6 (TC-006), IR-302: `TWO_PARAM_UNIT`'s `+` (`x + y`) settles
/// `Supported`, ranging each argument over its own operand's domain: `x`'s
/// `[0, 9]`, then `y`'s `[10, 20]`. The `+` node's own type is the plain
/// scalar type; only a narrowing `conversion` consumes it, so CG reads the
/// conversion's own `[10, 29]` bound as the `+` node's result bound
/// (IR-302), rather than treating it as unreachable.
#[test]
fn tc_006_x_plus_y_settles_supported_with_independent_argument_bounds() {
    let checked = two_param_package().checked;
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        two_param_input(&packages),
        &registry,
        None,
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
    assert_eq!(harness.identity.operation_identity, "quire.op.integer.add");
    let bounds: Vec<(i64, i64)> = harness
        .identity
        .arguments
        .iter()
        .map(|argument| (argument.minimum, argument.maximum))
        .collect();
    assert_eq!(bounds, [(0, 9), (10, 20)]);
    // The narrowing conversion's own result bound, `Int[10, 29]`: not in
    // `ScalarObligationIdentity` itself, only rendered into the harness
    // source's non-vacuity assertion.
    assert!(
        harness
            .rust
            .contents
            .contains("integer(\"10\")?, integer(\"29\")?"),
        "{}",
        harness.rust.contents
    );
}

/// FR-001-AC-6 (TC-006): `OVERFLOW_UNIT`'s `+` (`x + 100` directly in the
/// return context of `Int[0, 10]`) never reaches CG at all: QSL's own
/// checker proves the identical disjoint-interval fact CG's
/// `ResultBoundUnreachable` names (the same exact-integer-arithmetic
/// reachability question over `[100, 109]` against `[0, 10]`) and refuses
/// to compile the unit, at the check stage. This is one shape, not a
/// general proof that CG's refusal is unreachable -- see
/// `tc_006_x_plus_100_in_a_dead_branch_settles_result_bound_unreachable`
/// for a QSL-compiled unit that does reach it.
#[test]
fn tc_006_x_plus_100_in_the_return_context_is_refused_at_check() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();

    let refusal = drive(
        &QslCg,
        overflow_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect_err("QSL's own checker refuses a result bound its own arithmetic disproves");

    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Check));
    assert_eq!(refusal.code(), "undefined_expression/unproved-range");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
}

/// FR-001-AC-6 (TC-006), IR-302: `DEAD_BRANCH_UNIT` compiles -- QSL's
/// `Facts::walk` proves `x > 20` is never true over `x: Int[0, 9]` and
/// never discharges the dead branch's coercion, so the checked package
/// still emits. The unit carries two scalar operation applications --
/// the guard `x > 20` and the dead branch's `x + 100` -- so two
/// requirement records, both bounded (`x`'s own `[0, 9]` domain), both
/// routed to kani. The driver's generation output for both equals a direct
/// `generate_routed` call over the same two routed items (the same
/// pass-through invariant every other `tc_006_x_plus_*` test asserts). The
/// `+` item is `Unsupported`/`ResultBoundUnreachable`: CG ignores the
/// `if x > 20` guard and ranges `x` over its full declared `[0, 9]` domain,
/// so every exact result still lies in `[100, 109]`, disjoint from the
/// declared `[0, 10]`. No harness is carried for it.
///
/// The guard `>` item is asserted too (SR-013 FND-004): `quire.op.integer.gt`
/// is IR-confirmed (its operands are `x`, an `Int[0, 9]` reference, and the
/// literal `20`) but this generator has no Kani harness renderer for any
/// family but `IntegerArithmetic` (CG's own `OperationNotRendered` doc), so
/// it settles `Unsupported`/`OperationNotRendered`, naming the confirmed
/// operation and the one derived domain: `x`'s own `[0, 9]` bound node, read
/// as an `IntegerRange`. No harness is carried for it either. This is
/// checked against CG's own rules, not copied from a prior run: a
/// single-declaration unit with only `x > 20` (no dead branch, no `+` at
/// all) settles the identical disposition, confirming the dead branch's
/// `+` contributes nothing to how the guard itself settles.
#[test]
fn tc_006_x_plus_100_in_a_dead_branch_settles_result_bound_unreachable() {
    let checked = drive(
        &QslCg,
        dead_branch_input(&BTreeMap::new()),
        &build_registry(&[]).expect("an empty registry builds"),
        None,
        GenerationInput::default(),
    )
    .expect("QSL does not discharge a dead branch's coercion, so this compiles")
    .checked;
    let add = expression_by_identity(&checked, "quire.op.integer.add");
    let gt = expression_by_identity(&checked, "quire.op.integer.gt");
    let x_bound = integer_range_bound(&checked, "0", "9");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(
        &QslCg,
        dead_branch_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    assert_eq!(driven.routed.len(), 2);
    let backend = kani_candidate(&driven);
    // Occurrence order (`+`'s node digest sorts before `>`'s): the `+` is
    // request index 0, the guard `>` is request index 1.
    let expected = generate_routed(
        &checked,
        &[
            RoutedGenerationItem {
                request_index: 0,
                node_id: add.clone(),
                backend: backend.clone(),
                kind: BackendKind::Kani,
            },
            RoutedGenerationItem {
                request_index: 1,
                node_id: gt.clone(),
                backend,
                kind: BackendKind::Kani,
            },
        ],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for both routed items");
    assert_eq!(driven.generation, expected);

    let add_item = driven
        .generation
        .items
        .iter()
        .find(|item| item.request_index == 0)
        .expect("the `+` is routed at index 0");
    let KindOutput::Kani { record, harness } = &add_item.output;
    assert!(harness.is_none(), "{:?}", harness);
    assert_eq!(
        record.disposition,
        ObligationDisposition::Unsupported {
            reason: UnsupportedObligation::ResultBoundUnreachable {
                operation_identity: "quire.op.integer.add".to_owned(),
                lower: 0,
                upper: 10,
                reachable_lower: "100".to_owned(),
                reachable_upper: "109".to_owned(),
            }
        }
    );

    let gt_item = driven
        .generation
        .items
        .iter()
        .find(|item| item.request_index == 1)
        .expect("the guard `>` is routed at index 1");
    let KindOutput::Kani { record, harness } = &gt_item.output;
    assert!(harness.is_none(), "{:?}", harness);
    assert_eq!(
        record.disposition,
        ObligationDisposition::Unsupported {
            reason: UnsupportedObligation::OperationNotRendered {
                operation_identity: "quire.op.integer.gt".to_owned(),
                derived_domains: vec![DerivedDomain::IntegerRange {
                    bound: x_bound,
                    lower: "0".to_owned(),
                    upper: "9".to_owned(),
                }],
            }
        }
    );
}

/// FR-001-AC-5 and FR-001-AC-6 (TC-007): a routed Kani item with no Kani
/// context refuses the run with CG's `MissingKindContext`.
#[test]
fn tc_007_generation_refusals_are_typed() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let recorder = Recorder::default();
    let refusal = drive(
        &recorder,
        int_input(&packages),
        &registry,
        None,
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
}

/// FR-001-AC-3 (TC-003): when no item settles `supported`, `route` is not
/// called at all. `NEGATE_UNIT`'s one item is `value-validity`; a registry
/// that advertises only `operation-contract` has no candidate for it, so it
/// settles `unsupported`.
#[test]
fn tc_003_no_supported_item_means_no_route_call() {
    let registry = build_registry(&[manifest("kani", 7, &["operation-contract"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();

    let driven = drive(
        &recorder,
        negate_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the run completes");

    let dispositions: Vec<serde_json::Value> = driven
        .settlements
        .iter()
        .map(|s| serde_json::to_value(&s.disposition).expect("dispositions serialize"))
        .collect();
    assert_eq!(dispositions[0]["disposition"], "unsupported");
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

/// FR-001-AC-3 and FR-001-AC-6 (TC-003, TC-006): `route` and CG generation
/// see only the items negotiation settled `supported`, keyed by each
/// item's own request index -- not its position among the routed or
/// generated items. Real input cannot build a mixed settlement set on its
/// own: every requirement record settles the same kind, `value-validity`,
/// and an `Int` parameter with no declared range does not parse, so every
/// item in a unit gets the same candidate outcome (SR-013 FND-001). This
/// rewrites CG's real negotiation answer for `EQ_UNIT` through the
/// `Recorder` seam: item 0 (the `=`) settles `unsupported`, leaving only
/// item 1 (the `-`) `supported`, so `route` and `generate` each see one
/// item, and the gap between index 0 and index 1 makes list position and
/// request index diverge for it. If `route_supported` or `generate` used
/// the routed item's position in its output list instead of
/// `settlement.request_index`, the lone routed and generated item would
/// carry request index 0 (its position), not 1 (its own settlement's
/// index), and the positional asserts below would fail.
#[test]
fn tc_003_route_and_generate_skip_a_negotiation_unsupported_item_by_request_index() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder {
        unsupported_at: Some(0),
        ..Recorder::default()
    };

    let driven = drive(
        &recorder,
        eq_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    let dispositions: Vec<serde_json::Value> = driven
        .settlements
        .iter()
        .map(|s| serde_json::to_value(&s.disposition).expect("dispositions serialize"))
        .collect();
    assert_eq!(dispositions.len(), 2);
    assert_eq!(dispositions[0]["disposition"], "unsupported");
    assert_eq!(dispositions[1]["disposition"], "supported");

    // Positional: `routed`'s only element, at list position 0, carries
    // request index 1 -- the gap the rewritten item 0 left.
    let routed_indexes: Vec<usize> = driven.routed.iter().map(|r| r.request_index).collect();
    assert_eq!(routed_indexes, [1]);
    let generated_indexes: Vec<usize> = driven
        .generation
        .items
        .iter()
        .map(|item| item.request_index)
        .collect();
    assert_eq!(generated_indexes, [1]);

    // SR-013 FND-006: the generated item's *node* is checked too, not only
    // its request index. `generate` resolves each routed item's node by
    // looking it up with `items.get(request_index)`; if it instead used the
    // routed item's position in `routed` (`items.get(position)`), position 0
    // would resolve to `items[0]`, the `=` node -- the item negotiation just
    // rewrote to `unsupported` -- while still reporting request index 1.
    // Comparing the whole generation output against a direct `generate_routed`
    // call naming the `-` node at request index 1 catches that: it would
    // disagree with whatever CG derives for the `=` node instead.
    let neg = expression_by_identity(&driven.checked, "quire.op.integer.negate");
    let backend = kani_candidate(&driven);
    let expected = generate_routed(
        &driven.checked,
        &[RoutedGenerationItem {
            request_index: 1,
            node_id: neg,
            backend,
            kind: BackendKind::Kani,
        }],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the `-` item alone");
    assert_eq!(driven.generation, expected);

    assert_eq!(
        recorder.calls(),
        [
            Call::Compile,
            Call::ReadPackage,
            Call::BackendKind("kani".to_owned()),
            Call::Negotiate { items: 2 },
            Call::Route {
                supported: 1,
                dispositions: 1,
            },
            Call::Generate { items: 1 },
        ]
    );
}

/// FR-001-AC-3 and FR-001-AC-6 (TC-003): a unit with no scalar operation
/// application writes no requirement record (QSL FR-075), so the driver
/// routes nothing and calls neither `route` nor `generate` -- with no
/// caller input naming an item, `UNIT` (`function f(): Boolean pure { true
/// }`) still drives to completion.
#[test]
fn tc_003_a_unit_with_no_scalar_applications_routes_nothing() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();

    let driven = drive(
        &recorder,
        input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the run completes");

    assert!(driven.settlements.is_empty());
    assert!(driven.routed.is_empty());
    assert_eq!(driven.generation, RoutedGeneration::default());
    let calls = recorder.calls();
    assert!(calls.contains(&Call::Negotiate { items: 0 }));
    assert!(
        !calls
            .iter()
            .any(|c| matches!(c, Call::Route { .. } | Call::Generate { .. })),
        "{calls:?}"
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
        None,
        GenerationInput::default(),
    )
    .expect_err("the unit does not parse");
    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Source));
    assert_eq!(refusal.code(), "invalid_syntax");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
    assert_eq!(recorder.calls(), [Call::Compile]);

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
/// own rendering, not just the bare code.
#[test]
fn tc_008_a_missing_import_refuses_at_compile_intake() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    let broken = CompileInput {
        bytes: IMPORT_UNIT.as_bytes(),
        ..input(&packages)
    };

    let refusal = drive(&QslCg, broken, &registry, None, GenerationInput::default())
        .expect_err("nothing supplies test/library");

    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Intake));
    assert_eq!(refusal.code(), "missing_import/missing-selection");
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
}

/// FR-001-AC-5 (TC-008): a header `profile` whose identity is in no catalog
/// refuses at the assembly stage as `CompileRefusal::Profile` (QSL FR-110),
/// rendered `unknown_profile/unsupported-selection`: the code and the first
/// refusal's cause.
#[test]
fn tc_008_an_unknown_header_profile_refuses_with_its_cause() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    let source = "language \"ix:native\" edition \"1-draft\";\nprofile v = \"test:unknown-profile\";\nfunction f using v(): Boolean pure { true }\n";
    let broken = CompileInput {
        bytes: source.as_bytes(),
        ..input(&packages)
    };

    let refusal = drive(&QslCg, broken, &registry, None, GenerationInput::default())
        .expect_err("no catalog holds test:unknown-profile");

    assert!(
        matches!(
            &refusal,
            DriverRefusal::Compile(compile) if matches!(**compile, CompileRefusal::Profile { .. })
        ),
        "{refusal:?}"
    );
    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Assembly));
    assert_eq!(refusal.code(), "unknown_profile/unsupported-selection");
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
        None,
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
        document_pointer: None,
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
            None,
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
        let unit = unit(&format!("{declaration}\n"));
        let recorder = Recorder::default();
        let driven = drive(
            &recorder,
            CompileInput {
                bytes: unit.as_bytes(),
                ..input(&packages)
            },
            &registry,
            None,
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

/// FR-001-AC-3, FR-001-AC-6 and FR-001-AC-8 (TC-006): a routed node whose
/// operation has no derivable claim (`quire.op.integer.eq`; CG's derivation
/// names no operator family for it) settles `Unsupported` with
/// `no_derivable_claim`, and its routed sibling (`quire.op.integer.negate`,
/// which CG does derive a claim for) is unaffected: the same disposition
/// and full harness as a run that routes the `-` alone. `EQ_UNIT` writes
/// two requirement records -- the `=` and the `-` -- both settle
/// `supported` and both route, so the generation output is keyed
/// (ascending) by request index: `driven.generation.items[0]` is the `=`
/// at index 0, `items[1]` is the `-` at index 1 (occurrence order: the `=`
/// node's digest sorts first). The derived claim map holds both nodes,
/// each with its own disposition, and serializes.
#[test]
fn tc_006_an_underivable_node_settles_unsupported_and_its_sibling_is_unaffected() {
    let checked = eq_package().checked;
    let eq = expression_by_identity(&checked, "quire.op.integer.eq");
    let neg = expression_by_identity(&checked, "quire.op.integer.negate");

    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();

    let driven = drive(
        &recorder,
        eq_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    assert_eq!(driven.routed.len(), 2);
    assert_eq!(driven.generation.items.len(), 2);
    assert!(driven.generation.rejected.is_empty());

    // Positional and ascending (FR-001-AC-6): both items settled
    // `supported` and both route, so `generation.items` is keyed by
    // request index in ascending order, position for position.
    let eq_item = &driven.generation.items[0];
    let neg_item = &driven.generation.items[1];
    assert_eq!(eq_item.request_index, 0);
    assert_eq!(neg_item.request_index, 1);

    let KindOutput::Kani { record, harness } = &eq_item.output;
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
    let KindOutput::Kani { record, harness } = &neg_item.output;
    assert!(
        matches!(record.disposition, ObligationDisposition::Supported { .. }),
        "{:?}",
        record.disposition
    );
    let harness = harness
        .as_ref()
        .expect("the derivable sibling carries its harness");
    assert_eq!(harness.identity.node_id, neg);
    // The negation's one operand, ranged over `z`'s own `[0, 9]` domain --
    // unaffected by its underivable `=` sibling.
    let bounds: Vec<(i64, i64)> = harness
        .identity
        .arguments
        .iter()
        .map(|argument| (argument.minimum, argument.maximum))
        .collect();
    assert_eq!(bounds, [(0, 9)]);

    // FR-001-AC-8 (SR-013 FND-003): the `-` item's disposition and its
    // whole harness equal a direct `generate_routed` call over the `-`
    // item alone, over the same package -- the underivable `=` sibling
    // leaks into neither.
    let backend = kani_candidate(&driven);
    let alone = generate_routed(
        &checked,
        &[RoutedGenerationItem {
            request_index: 1,
            node_id: neg.clone(),
            backend,
            kind: BackendKind::Kani,
        }],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the `-` item alone");
    assert_eq!(*neg_item, alone.items[0]);

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
