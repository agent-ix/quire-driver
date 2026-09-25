// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-001 acceptance tests: the driver's step order, the MD-2 refusal and
//! routing after negotiation. Every call goes to the real QSL and CG
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
    BackendKind, BackendProviderEnvelope, EnvelopeRefusal, ItemSettlement,
};
use quire_driver::{
    build_registry, drive, CompileInput, DriverRefusal, DriverStage, OutcomeKind, ProviderManifest,
    QslCg, RequestedItem, Upstream,
};

const UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
    profile v = \"quire.value.complete/v1\" version \"1\" digest \
    \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\";\n\
    function f using v(): Boolean pure { true }\n";

/// One upstream call, as the driver made it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Call {
    Compile,
    BackendKind(String),
    Negotiate {
        items: usize,
    },
    Route {
        supported: usize,
        dispositions: usize,
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
    RequestedItem {
        node: WireNodeId::from_digest([node; 32]),
        requirements: Requirements::new(kind, ClaimExtent::Bounded),
        backend: backend.map(BackendId::new),
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

/// FR-001-AC-4 and FR-001-AC-3 (TC-001, TC-003): the driver compiles and
/// emits, converts every backend, negotiates every item once, then routes
/// only the items CG settled `supported`, each to its `BackendId`.
#[test]
fn tc_001_tc_003_steps_run_in_order_and_route_only_supported_items() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let items = [
        // Supported: kani advertises value-validity, bounded extent.
        item(1, Capability::ValueValidity, None),
        // Unsupported: no registered backend advertises operation-contract.
        item(2, Capability::OperationContract, None),
        // Invalid request: the request names an unregistered backend.
        item(3, Capability::ValueValidity, Some("missing")),
    ];

    let driven = drive(&recorder, input(&packages), &registry, &items).expect("the run completes");

    // E4 emitted v2 bytes with the package identity.
    assert!(!driven.emitted.bytes().is_empty());

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

    assert_eq!(
        recorder.calls(),
        [
            Call::Compile,
            Call::BackendKind("kani".to_owned()),
            Call::Negotiate { items: 3 },
            Call::Route {
                supported: 1,
                dispositions: 1,
            },
        ]
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

    let refusal = drive(&recorder, broken, &registry, &[]).expect_err("the unit does not parse");
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
    let refusal = drive(&Recorder::default(), deep, &registry, &[])
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
