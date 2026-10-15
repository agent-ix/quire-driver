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
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;

use qsl_foundation::diagnostic::Category;
use qsl_foundation::BackendId;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, package, parse, select, CheckedUnit, CompileRefusal, DependencyInput, EmittedUnit,
    LockEvidence, PackageLimits, ParseRequest, SpineLimits, SpineStage,
};
use qsl_route::request::items_from_requirements;
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{Candidate, ManifestDigest, RegistrationCause};
use qsl_semantics::value::{CatalogRole, DefinitionLock};
use quire_contract_codegen::{
    generate_routed, BackendKind, BackendProviderEnvelope, Candidates, Cause, ClaimDisposition,
    ContractRequestPosition, Disposition, EnvelopeRefusal, GenerationContexts, ItemSettlement,
    KaniGenerationContext, KindOutput, ObligationDisposition, ProofCeilings, RoutedGeneration,
    RoutedGenerationError, RoutedGenerationItem, UnsupportedObligation,
};
use quire_contract_model::{
    CheckedNodeId, CheckedOccurrenceRole, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2, ContractPackageV1,
};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, DriverFault, DriverRefusal, DriverStage,
    GenerationInput, KaniInput, KaniProvider, OutcomeKind, Provider, ProviderItemFailure,
    ProviderLimits, ProviderManifest, ProviderRequest, QslCg, Upstream,
};
use quire_exact::{Cancel, CancelCause};

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

// Minimal model document authored for TC-019. QSL's package_input helper
// computes its selection digest, so this test has no copied fixture or pin.
const CANCEL_MODEL_DOCUMENT: &str = r#"{
  "contractVersion": "2.0.0",
  "canonicalization": {
    "algorithm": "rfc8785-v1",
    "digest": "sha256-jcs"
  },
  "source": {
    "identity": "ix://driver/cancellation/model",
    "version": "1.0.0",
    "dialect": "spec-bundle",
    "digest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
  },
  "package": {
    "identity": "driver/cancellation-model",
    "version": "1.0.0",
    "manifestDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "mappingVersions": [],
    "profileVersions": [],
    "lockDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
  },
  "occurrences": [],
  "extensions": [],
  "constructs": [{
    "kind": {"module": "driver/cancellation-model", "name": "entity"},
    "moduleVersion": "1.0.0",
    "manifestDigest": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    "construct": {
      "identity": "none",
      "shape": "record",
      "members": {},
      "meaning": "quire.meaning.model.object-type/v1"
    }
  }],
  "types": [{
    "identity": "ix://driver/cancellation-model/Thing",
    "displayName": "Thing",
    "kind": {"module": "driver/cancellation-model", "name": "entity"},
    "roles": [],
    "origin": {
      "generated": {
        "generatorIdentity": "ix://driver/cancellation/model",
        "generatorVersion": "1.0.0",
        "inputIdentities": ["ix://driver/cancellation-model/Thing"]
      }
    },
    "constraints": [],
    "extensions": [],
    "unknownPolicy": "reject",
    "supertypes": [],
    "fields": [],
    "operations": []
  }]
}"#;

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

/// QSL FR-062 RR-16: the same canonical addition occurs below two distinct
/// narrows, with independently selected `[0, 10]` and `[0, 20]` result bounds.
static RR16_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit(
        "function g using v(p: Int[0, 10]): Boolean pure { true }\n\
         function h using v(q: Int[0, 20]): Boolean pure { true }\n\
         function f using v(x: Int[0, 9]): Boolean pure { g(x + 1) and h(x + 1) }\n",
    )
});

/// QSL FR-062's own-bound example: an add with no selected direct narrow must
/// not inherit a result bound from another occurrence or guessed context.
static UNNARROWED_ADD_UNIT: LazyLock<String> =
    LazyLock::new(|| unit("function own using v(x: Int[0, 9]): Integer pure { x + 1 }\n"));

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
    unit(
        "function unreachable_branch using v(x: Int[0, 9]): Int[0, 10] pure { if x > 20 then x + 100 else 0 }\n",
    )
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

#[derive(Clone)]
struct GenerationCall {
    package: ContractPackageV1,
    positions: Vec<ContractRequestPosition>,
    routed: Vec<RoutedGenerationItem>,
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
    /// E5's source provisions, captured while the E4 value is still alive.
    source_provision: RefCell<Option<Vec<(SourceIdentity, String)>>>,
    /// The exact typed E5 evidence supplied to IR's reader.
    read_evidence: RefCell<Option<CheckedPackageEvidence>>,
    /// Exact typed arguments passed to CG's generator.
    generation: RefCell<Option<GenerationCall>>,
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

    fn source_provision(&self) -> Vec<(SourceIdentity, String)> {
        self.source_provision
            .borrow()
            .clone()
            .expect("E5 received E4 source provision")
    }

    fn generation_call(&self) -> GenerationCall {
        self.generation
            .borrow()
            .clone()
            .expect("CG generation was called")
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
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal> {
        self.calls.borrow_mut().push(Call::Compile);
        QslCg.compile(source, path, bytes, packages, limits, cancel)
    }

    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        self.calls.borrow_mut().push(Call::ReadPackage);
        *self.read_evidence.borrow_mut() = Some(evidence.clone());
        *self.source_provision.borrow_mut() = Some(
            emitted
                .sources()
                .iter()
                .map(|source| (source.identity().clone(), source.text().to_owned()))
                .collect(),
        );
        match &self.read_answer {
            Some(answer) => answer.clone(),
            None => QslCg.read_package(emitted, limits, evidence),
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
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        self.calls.borrow_mut().push(Call::Generate {
            items: routed.len(),
        });
        *self.generation.borrow_mut() = Some(GenerationCall {
            package: package.clone(),
            positions: positions.to_vec(),
            routed: routed.to_vec(),
        });
        QslCg.generate(package, positions, routed, contexts)
    }
}

fn manifest(identity: &str, digest: u8, kinds: &[&str]) -> ProviderManifest {
    ProviderManifest {
        identity: identity.to_owned(),
        manifest_digest: ManifestDigest::from_digest([digest; 32]),
        advertises: kinds
            .iter()
            .map(|kind| (Some((*kind).to_owned()), Some("bounded".to_owned())))
            .collect(),
        domains: Some(vec!["integer".to_owned()]),
        wire: None,
    }
}

fn input(packages: &BTreeMap<[u8; 32], Vec<u8>>) -> CompileInput<'_> {
    CompileInput {
        source: SourceIdentity::new("agent-ix", "driver-test", "git", "1"),
        path: "unit.native",
        bytes: UNIT.as_bytes(),
        packages,
        limits: SpineLimits::default(),
        cancel: Cancel::new(),
    }
}

fn cancellation_source_and_packages() -> (String, BTreeMap<[u8; 32], Vec<u8>>) {
    let packages = qsl_semantics::model::intake::package_input([CANCEL_MODEL_DOCUMENT.as_bytes()]);
    let digest = packages.keys().next().expect("one model document");
    let digest_hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let source = unit(&format!(
        "model M = \"driver/cancellation-model\" version \"1.0.0\" digest \"sha256-jcs:{digest_hex}\";\n\
         function keep using v(x: M::Thing): Boolean pure {{ true }}\n"
    ));
    (source, packages)
}

/// Baseline QSL charge counts after parse, select, check and package. The
/// cancellation fixture includes a model so every stage has a charged path.
fn cancellation_boundaries(
    source_text: &str,
    packages: &BTreeMap<[u8; 32], Vec<u8>>,
) -> [usize; 4] {
    let observations = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&observations);
    let cancel = Cancel::observing(move || {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    let source = SourceIdentity::new("agent-ix", "driver-cancellation", "fixture", "1");
    let request = ParseRequest {
        source: &source,
        path: "cancellation-model.native",
        bytes: source_text.as_bytes(),
    };
    let limits = SpineLimits::default();
    let parsed = parse(&request, limits.source, &cancel)
        .expect("cancellation fixture parses")
        .into_value();
    let after_parse = observations.load(Ordering::SeqCst);
    let models = select(&parsed, packages, limits.model, &cancel)
        .expect("cancellation model selects")
        .into_value();
    let after_select = observations.load(Ordering::SeqCst);
    let checked = check(
        &parsed,
        &models,
        &DependencyInput::default(),
        &LockEvidence::default(),
        limits,
        &cancel,
    )
    .expect("cancellation fixture checks")
    .into_value();
    let after_check = observations.load(Ordering::SeqCst);
    package(&checked, PackageLimits::default(), &cancel).expect("cancellation fixture packages");
    let after_package = observations.load(Ordering::SeqCst);
    assert!(0 < after_parse && after_parse < after_select);
    assert!(after_select < after_check && after_check < after_package);
    [after_parse, after_select, after_check, after_package]
}

/// Cancel from a second thread on exactly the target QSL charge, waiting for
/// that thread to record the cause before the meter polls the handle.
fn cancel_on_charge(
    target: usize,
    cause: CancelCause,
) -> (Cancel, thread::JoinHandle<()>, Arc<AtomicBool>) {
    let observations = Arc::new(AtomicUsize::new(0));
    let timed_out = Arc::new(AtomicBool::new(false));
    let (request_cancel, request_cancel_rx) = std::sync::mpsc::channel();
    let (cancelled, cancelled_rx) = std::sync::mpsc::channel();
    let cancelled_rx = Arc::new(std::sync::Mutex::new(cancelled_rx));
    let seen = Arc::clone(&observations);
    let ack = Arc::clone(&cancelled_rx);
    let observer_timed_out = Arc::clone(&timed_out);
    let cancel = Cancel::observing(move || {
        let charge = seen.fetch_add(1, Ordering::SeqCst) + 1;
        if charge == target {
            if request_cancel.send(()).is_err() {
                observer_timed_out.store(true, Ordering::SeqCst);
                return;
            }
            let acknowledged = ack.lock().ok().is_some_and(|receiver| {
                receiver
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .is_ok()
            });
            if !acknowledged {
                observer_timed_out.store(true, Ordering::SeqCst);
            }
        }
    });
    let worker_cancel = cancel.clone();
    let worker_timed_out = Arc::clone(&timed_out);
    let worker = thread::spawn(move || {
        if request_cancel_rx
            .recv_timeout(std::time::Duration::from_secs(120))
            .is_err()
        {
            worker_timed_out.store(true, Ordering::SeqCst);
            return;
        }
        worker_cancel.cancel(cause);
        if cancelled.send(()).is_err() {
            worker_timed_out.store(true, Ordering::SeqCst);
        }
    });
    (cancel, worker, timed_out)
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

/// Trace: FR-011-AC-1, FR-011-AC-2, FR-011-AC-3, FR-011-AC-4 (TC-019 steps 2, 3 and 5).
#[test]
fn tc_019_cancellation_preserves_every_stage_and_cause() {
    let (source_text, packages) = cancellation_source_and_packages();
    let boundaries = cancellation_boundaries(&source_text, &packages);
    let registry = build_registry(&[]).expect("an empty registry builds");
    let stages = [
        SpineStage::Source,
        SpineStage::Intake,
        SpineStage::Check,
        SpineStage::Emit,
    ];

    for (index, stage) in stages.into_iter().enumerate() {
        let target = if index == 0 { 0 } else { boundaries[index - 1] } + 1;
        for cause in [CancelCause::Requested, CancelCause::Deadline] {
            let (cancel, worker, timed_out) = cancel_on_charge(target, cause);
            let recorder = Recorder::default();
            let outcome = drive(
                &recorder,
                CompileInput {
                    source: SourceIdentity::new("agent-ix", "driver-cancellation", "fixture", "1"),
                    path: "cancellation-model.native",
                    bytes: source_text.as_bytes(),
                    packages: &packages,
                    limits: SpineLimits::default(),
                    cancel,
                },
                &registry,
                None,
                GenerationInput::default(),
            );
            worker.join().expect("cancellation worker completes");
            assert!(
                !timed_out.load(Ordering::SeqCst),
                "second-thread cancellation worker timed out at {stage:?}/{cause:?}"
            );
            let refusal = outcome.expect_err("a cancelled lifecycle stage returns no Driven value");

            assert_eq!(refusal.stage(), DriverStage::Compile(stage));
            assert_eq!(refusal.code(), "cancelled");
            assert_eq!(refusal.kind(), OutcomeKind::Cancelled);
            let category = refusal
                .qsl_category()
                .expect("QSL owns cancellation category");
            assert_eq!(category, Category::Incomplete);
            assert_eq!(category.exit_code(), Category::Incomplete.exit_code());
            assert_eq!(category.exit_code(), 22);
            assert!(matches!(
                refusal,
                DriverRefusal::Cancelled {
                    stage: seen_stage,
                    cause: seen_cause,
                } if seen_stage == stage && seen_cause == cause
            ));
            assert_eq!(recorder.calls(), [Call::Compile]);
        }
    }

    // An already-cancelled caller handle stops at Source before any stage
    // work; it does not become the unshared-handle fault from FR-001.
    let cancel = Cancel::new();
    cancel.cancel(CancelCause::Requested);
    let recorder = Recorder::default();
    let refusal = drive(
        &recorder,
        CompileInput {
            source: SourceIdentity::new("agent-ix", "driver-cancellation", "fixture", "1"),
            path: "cancellation-model.native",
            bytes: source_text.as_bytes(),
            packages: &packages,
            limits: SpineLimits::default(),
            cancel,
        },
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect_err("a pre-cancelled caller handle stops at Source");
    assert!(matches!(
        refusal,
        DriverRefusal::Cancelled {
            stage: SpineStage::Source,
            cause: CancelCause::Requested,
        }
    ));
    assert_eq!(recorder.calls(), [Call::Compile]);
}

/// Trace: FR-001-AC-9, FR-001-AC-10, FR-011-AC-1 (TC-019 step 4).
#[test]
fn tc_019_e5_receives_source_provision_and_checked_unit_retains_bounded_root() {
    let (source_text, packages) = cancellation_source_and_packages();
    let registry = build_registry(&[]).expect("an empty registry builds");
    let recorder = Recorder::default();
    let source = SourceIdentity::new("agent-ix", "driver-cancellation", "fixture", "1");
    let limits = SpineLimits::default();
    let driven = drive(
        &recorder,
        CompileInput {
            source: source.clone(),
            path: "cancellation-model.native",
            bytes: source_text.as_bytes(),
            packages: &packages,
            limits,
            cancel: Cancel::new(),
        },
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the model fixture drives with empty dependencies");

    let provision = recorder.source_provision();
    assert_eq!(provision.len(), 1, "only the root source was emitted");
    assert_eq!(provision[0].0, source);
    assert_eq!(provision[0].1.as_bytes(), source_text.as_bytes());
    assert_eq!(driven.package.source().identity(), &provision[0].0);
    assert_eq!(driven.package.source().text(), provision[0].1);
    assert!(driven.package.source().text().len() <= limits.source.source_bytes);
    assert!(driven.package.libraries().is_empty());
    assert_eq!(recorder.calls()[..2], [Call::Compile, Call::ReadPackage]);
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

/// The Kani generation input. IR-294 has CG derive each routed item's claim
/// from the node's own operation and bound, so the caller supplies no claim
/// map any more.
fn kani_input() -> GenerationInput<'static> {
    GenerationInput {
        kani: Some(KaniInput {
            ceilings: proof_ceilings(),
            subject_path: "crate::subject",
            unwind: 1,
        }),
    }
}

fn proof_ceilings() -> ProofCeilings {
    ProofCeilings {
        memory_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("positive memory ceiling"),
        wall_clock: Duration::from_secs(30),
    }
}

/// The kani backend as `route` returned it, in CG's wire spelling.
fn kani_candidate(driven: &Driven) -> quire_contract_codegen::Candidate {
    let candidate = &driven.routed[0].candidate;
    quire_contract_codegen::Candidate {
        identity: candidate.id().as_str().to_owned(),
    }
}

fn assert_generation_matches(
    driven: &Driven,
    recorder: &Recorder,
    expected_positions: &[CheckedNodeId],
    expected_generated: &[(usize, CheckedNodeId)],
) {
    let call = recorder.generation_call();
    let mut positions = call.positions.iter();
    for expected_node in expected_positions {
        let Some(ContractRequestPosition::DriverItem {
            node_id,
            result_selection,
        }) = positions.next()
        else {
            panic!("each requirement retains its driver request position");
        };
        assert_eq!(node_id, expected_node);
        let quire_contract_codegen::ContractDriverResultSelection::OwnResult {
            application,
            conversion,
        } = result_selection
        else {
            panic!("scalar items keep their selected result occurrence");
        };
        assert_eq!(&application.node_id, expected_node);
        if let Some(conversion) = conversion {
            assert!(matches!(
                positions.next(),
                Some(ContractRequestPosition::ContextRoot { node_id })
                    if node_id == &conversion.node_id
            ));
        }
    }
    assert!(
        positions.next().is_none(),
        "all positions belong to requests"
    );
    assert_eq!(
        call.package.wire().package.source.package_id,
        *driven.checked.package_id(),
        "the generation package came from the admitted checked package"
    );
    let expected = generate_routed(
        &call.package,
        &call.positions,
        &call.routed,
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG accepts the exact package and positions passed by the driver");
    assert_eq!(
        driven.generation, expected,
        "generation passes CG's output through"
    );
    assert_eq!(
        driven.generation.items.len(),
        expected_generated.len(),
        "only expected supported requests are generated"
    );
    for (output, (request_index, node_id)) in driven.generation.items.iter().zip(expected_generated)
    {
        assert_eq!(output.request_index, *request_index);
        let routed = driven
            .routed
            .iter()
            .find(|routed| routed.request_index == *request_index)
            .expect("each generated request has its routed candidate");
        assert_eq!(
            output.backend,
            quire_contract_codegen::Candidate {
                identity: routed.candidate.id().as_str().to_owned(),
            },
            "generated backend matches the driver's routed item"
        );
        let KindOutput::Kani { record, .. } = &output.output else {
            panic!("the routed Kani item keeps its typed Kani output");
        };
        let quire_contract_codegen::ObligationSubject::CheckedNode {
            node_id: actual, ..
        } = &record.subject
        else {
            panic!("the generated record points to the admitted checked node");
        };
        assert_eq!(actual, node_id);
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

    let envelope = recorder.envelope();
    let extent = envelope.items[0]
        .extent
        .as_ref()
        .expect("the initial request carries its QSL extent");
    assert!(
        extent.bounds.is_empty(),
        "initial requirement requests have no substituted proof bounds"
    );
    assert!(
        extent.domains.is_empty(),
        "this bounded negation has no unbounded domains"
    );
    let quire_contract_codegen::KaniForm::V2ScalarClaim {
        positions,
        request_index,
        node_id,
        ..
    } = &envelope.items[0].kani_form
    else {
        panic!("value-validity carries its checked scalar form");
    };
    assert_eq!(*request_index, 0);
    assert_eq!(node_id, &neg);
    assert_eq!(
        positions.len(),
        2,
        "selected conversion adds one context root"
    );
    let quire_contract_codegen::ContractRequestPosition::DriverItem {
        result_selection:
            quire_contract_codegen::ContractDriverResultSelection::OwnResult {
                conversion: Some(conversion),
                ..
            },
        ..
    } = &positions[0]
    else {
        panic!("the negate request retains its selected conversion");
    };
    assert!(matches!(
        &positions[1],
        quire_contract_codegen::ContractRequestPosition::ContextRoot { node_id }
            if node_id == &conversion.node_id
    ));

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

    // Step 7 reports the exact admitted node and routed request index.
    assert_generation_matches(
        &driven,
        &recorder,
        std::slice::from_ref(&neg),
        &[(0, neg.clone())],
    );
    assert_eq!(driven.generation.items.len(), 1);
    assert_eq!(driven.generation.items[0].request_index, 0);
    assert_eq!(driven.generation.items[0].backend, kani_candidate(&driven));
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
    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output else {
        panic!("a routed Kani request produces Kani output");
    };
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

/// Trace: FR-001-AC-7; TC-006 step 11.
#[test]
fn tc_006_handoff_passes_domain_package_evidence_unchanged() {
    let packages = qsl_semantics::model::intake::package_input([CANCEL_MODEL_DOCUMENT.as_bytes()]);
    let recorder = Recorder::default();
    let registry = build_registry(&[]).expect("an empty registry builds");

    drive(
        &recorder,
        input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the ordinary unit completes while passing its package evidence to E5");

    let actual = recorder
        .read_evidence
        .borrow()
        .clone()
        .expect("E5 receives evidence");
    let mut expected = CheckedPackageEvidence::new();
    let (digest, document) = packages.iter().next().expect("one supplied domain package");
    expected
        .insert_domain_package_document(qsl_semantics::model::key::hex(digest), document.as_slice())
        .expect("the input map supplies one document per digest");
    let lock = DefinitionLock::pinned();
    expected.support_feature(lock.entry(CatalogRole::Root).identity);

    assert_eq!(
        actual, expected,
        "domain package key and bytes reach E5 unchanged"
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
/// `+` (`x + 1`) passes the typed package, selected request position, and
/// CG output through unchanged.
#[test]
fn tc_006_x_plus_1_generation_passes_through_unchanged() {
    let checked = int_package().checked;
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        int_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    assert_generation_matches(
        &driven,
        &recorder,
        std::slice::from_ref(&add),
        &[(0, add.clone())],
    );
}

/// FR-001-AC-6 (TC-006): the driver's generation output for
/// `TWO_PARAM_UNIT`'s `+` (`x + y`) passes the typed package, selected
/// request position, and CG output through unchanged. See
/// `tc_006_x_plus_1_generation_passes_through_unchanged`.
#[test]
fn tc_006_x_plus_y_generation_passes_through_unchanged() {
    let checked = two_param_package().checked;
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        two_param_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    assert_generation_matches(
        &driven,
        &recorder,
        std::slice::from_ref(&add),
        &[(0, add.clone())],
    );
}

/// QSL-712: one expression node can occur below different narrows. The
/// driver keeps each occurrence's selected narrow adjacent to its own
/// request position, while the canonical add remains one semantic node and
/// neither context root becomes an extra request or claim.
/// Trace: FR-001-AC-12.
#[test]
fn rr16_conversion_context_is_forwarded_per_occurrence() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        CompileInput {
            bytes: RR16_UNIT.as_bytes(),
            ..input(&packages)
        },
        &registry,
        None,
        kani_input(),
    )
    .expect("QSL compiles the RR-16 source and CG settles both requests");

    let qsl_items = items_from_requirements(
        driven.package.package().graph().requirements(),
        &registry,
        None,
    );
    assert_eq!(qsl_items.len(), 2, "one request per add occurrence");
    assert_eq!(qsl_items[0].node(), qsl_items[1].node());
    let qsl_contexts = qsl_items
        .iter()
        .map(|item| item.conversion_context().expect("RR-16 has direct narrows"))
        .collect::<Vec<_>>();
    assert_ne!(qsl_contexts[0], qsl_contexts[1]);

    let envelope = recorder.envelope();
    assert_eq!(
        envelope.items.len(),
        2,
        "context roots are not routed items"
    );
    assert_eq!(driven.routed.len(), 2, "only the two QSL items are routed");
    let forms = envelope
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let quire_contract_codegen::KaniForm::V2ScalarClaim {
                positions,
                request_index,
                node_id,
                contract,
                ..
            } = &item.kani_form
            else {
                panic!("RR-16 scalar requests carry their admitted form");
            };
            assert_eq!(*request_index, index);
            (positions, node_id, contract)
        })
        .collect::<Vec<_>>();
    assert_eq!(forms.len(), 2);
    let positions = forms[0].0;
    assert_eq!(
        forms[1].0, positions,
        "both items refer to one lowering call"
    );
    assert_eq!(positions.len(), 4, "each request has one context-only root");

    let mut selected_bounds = Vec::new();
    let mut selected_bound_ids = Vec::new();
    for index in 0..2 {
        let (positions, item_node, contract) = forms[index];
        assert_eq!(
            item_node.digest.as_ref(),
            qsl_items[index].node().to_string()
        );
        assert_eq!(
            contract.wire().package.source.package_id,
            *driven.checked.package_id()
        );
        let ContractRequestPosition::DriverItem {
            node_id,
            result_selection:
                quire_contract_codegen::ContractDriverResultSelection::OwnResult {
                    application,
                    conversion: Some(conversion),
                },
        } = &positions[index * 2]
        else {
            panic!("each requirement has its own scalar result selection");
        };
        assert_eq!(node_id, item_node);
        assert_eq!(&application.node_id, item_node);
        assert_eq!(conversion.role, CheckedOccurrenceRole::Expression);
        assert_eq!(conversion.ordinal, qsl_contexts[index].origin().ordinal());
        assert_eq!(
            conversion.node_id.digest.as_ref(),
            qsl_contexts[index].node().to_string(),
            "the selected conversion key comes from this exact QSL occurrence"
        );
        let ContractRequestPosition::ContextRoot {
            node_id: context_node,
        } = &positions[index * 2 + 1]
        else {
            panic!("the selected conversion is a context-only lowering root");
        };
        assert_eq!(context_node, &conversion.node_id);
        let narrow = contract
            .wire()
            .semantic_graph
            .nodes
            .iter()
            .find(|node| node.node_id == conversion.node_id)
            .expect("the selected narrow is present in the admitted package");
        let bound = contract
            .wire()
            .semantic_graph
            .nodes
            .iter()
            .find(|node| node.node_id == narrow.semantic_type)
            .expect("the selected narrow's result bound is present");
        let maximum = bound
            .body
            .pointer("/members/1/value/value")
            .and_then(serde_json::Value::as_str)
            .expect("the selected integer range has an upper endpoint")
            .to_owned();
        selected_bounds.push(maximum);
        selected_bound_ids.push(bound.node_id.clone());
    }
    assert_eq!(selected_bounds, ["10", "20"]);

    let claims = &driven
        .generation
        .claim_map
        .as_ref()
        .expect("the Kani generation returns its typed claim map")
        .items;
    assert_eq!(claims.len(), 2, "context roots create no extra claims");
    let mut checked_bounds = claims
        .iter()
        .map(|claim| {
            let ClaimDisposition::Generated(generated) = &claim.result else {
                panic!("both selected result contexts are supported");
            };
            generated
                .checked_bounds
                .first()
                .expect("each generated claim records its selected result bound")
                .clone()
        })
        .collect::<Vec<_>>();
    checked_bounds.sort();
    selected_bound_ids.sort();
    assert_eq!(checked_bounds, selected_bound_ids);
}

/// QSL owns the absence of a selected conversion. The driver forwards `None`
/// as a one-position request, and CG retains its typed `RequiresBound`
/// disposition rather than receiving a guessed context root.
#[test]
fn add_without_selected_conversion_keeps_requires_bound() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        CompileInput {
            bytes: UNNARROWED_ADD_UNIT.as_bytes(),
            ..input(&packages)
        },
        &registry,
        None,
        kani_input(),
    )
    .expect("an unselected scalar add retains its typed negotiation result");

    let items = items_from_requirements(
        driven.package.package().graph().requirements(),
        &registry,
        None,
    );
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].conversion_context(), None);
    let envelope = recorder.envelope();
    let quire_contract_codegen::KaniForm::V2ScalarClaim { positions, .. } =
        &envelope.items[0].kani_form
    else {
        panic!("the unselected add retains its checked scalar form");
    };
    assert_eq!(positions.len(), 1, "None adds no context-only root");
    assert!(matches!(
        &positions[0],
        quire_contract_codegen::ContractRequestPosition::DriverItem {
            result_selection: quire_contract_codegen::ContractDriverResultSelection::OwnResult {
                conversion: None,
                ..
            },
            ..
        }
    ));
    assert!(
        recorder.generation.borrow().is_none(),
        "no item is generated"
    );
    assert!(driven.routed.is_empty());
    assert!(driven.generation.items.is_empty());
    assert!(
        matches!(
            &driven.settlements[0].disposition,
            quire_contract_codegen::Disposition::RequiresBound { .. }
        ),
        "{:?}",
        driven.settlements
    );
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

    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        int_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    assert_generation_matches(
        &driven,
        &recorder,
        std::slice::from_ref(&add),
        &[(0, add.clone())],
    );

    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output else {
        panic!("a routed Kani request produces Kani output");
    };
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

    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        two_param_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    let KindOutput::Kani { record, harness } = &driven.generation.items[0].output else {
        panic!("a routed Kani request produces Kani output");
    };
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

/// Trace: FR-001-AC-6.
/// QSL emits the guard `x > 20` and dead-branch add `x + 100`, but neither
/// item has a selected conversion context. CG preserves their typed
/// unsupported results without routing either item to generation.
#[test]
fn tc_006_x_plus_100_in_a_dead_branch_retains_requires_bound() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        dead_branch_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the run completes");

    assert!(driven.routed.is_empty());
    assert!(driven.generation.items.is_empty());
    assert_eq!(driven.settlements.len(), 2);
    let Disposition::Unsupported {
        cause:
            Cause::KaniFormRefused {
                refusal:
                    UnsupportedObligation::ResultBoundUnreachable {
                        operation_identity,
                        lower,
                        upper,
                        reachable_lower,
                        reachable_upper,
                    },
                ..
            },
    } = &driven.settlements[0].disposition
    else {
        panic!("the add keeps CG's exact unreachable-result refusal");
    };
    assert_eq!(operation_identity, "quire.op.integer.add");
    assert_eq!((*lower, *upper), (0, 10));
    assert_eq!(reachable_lower, "100");
    assert_eq!(reachable_upper, "109");
    assert!(matches!(
        &driven.settlements[1].disposition,
        Disposition::Unsupported {
            cause: Cause::KaniFormRefused {
                refusal: UnsupportedObligation::OperationNotRendered { .. },
                ..
            }
        }
    ));
    assert!(recorder.generation.borrow().is_none());
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
    // its request index. The exact CG input positions keep `=` at request 0
    // and `-` at request 1, while only the supported `-` is routed.
    let neg = expression_by_identity(&driven.checked, "quire.op.integer.negate");
    assert_generation_matches(
        &driven,
        &recorder,
        &[
            expression_by_identity(&driven.checked, "quire.op.integer.eq"),
            neg.clone(),
        ],
        &[(1, neg)],
    );

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
/// registration refusal and reachable source/checking limits.
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

    for (limits, stage, code) in [
        {
            let mut limits = SpineLimits::default();
            limits.source.source_bytes = 8;
            (
                limits,
                SpineStage::Source,
                "stage_limit_exceeded/input-bytes-exceeded",
            )
        },
        {
            let mut limits = SpineLimits::default();
            limits.checking = limits.checking.with_work_budget(1);
            (
                limits,
                SpineStage::Check,
                "stage_limit_exceeded/work-budget-exceeded",
            )
        },
    ] {
        let recorder = Recorder::default();
        let refusal = drive(
            &recorder,
            CompileInput {
                limits,
                ..input(&packages)
            },
            &registry,
            None,
            GenerationInput::default(),
        )
        .expect_err("the configured limit is reachable");
        assert_eq!(refusal.stage(), DriverStage::Compile(stage));
        assert_eq!(refusal.code(), code);
        assert_eq!(refusal.kind(), OutcomeKind::Limit);
        assert_eq!(recorder.calls(), [Call::Compile]);
    }

    // Two manifests, one identity, two distinct digests: both conflict,
    // and `refusals()` reports one entry per distinct digest, ascending.
    let expected_conflicts = [
        (
            Candidate::new(BackendId::new("kani")),
            ManifestDigest::from_digest([1; 32]),
            RegistrationCause::DuplicateBackend,
        ),
        (
            Candidate::new(BackendId::new("kani")),
            ManifestDigest::from_digest([2; 32]),
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
fn registration_refusals(
    refusal: &DriverRefusal,
) -> Vec<(Candidate, ManifestDigest, RegistrationCause)> {
    let DriverRefusal::Registration(refusals) = refusal else {
        panic!("expected a registration refusal, got {refusal:?}");
    };
    refusals
        .iter()
        .map(|r| (r.backend().clone(), r.manifest_digest(), r.cause().clone()))
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
    let CheckedPackageDispatchResult::Refused(refusal) = quire_contract_model::read_checked_package(
        b"not a JSON document",
        CheckedPackageReadLimits::bounded(),
        &CheckedPackageEvidence::new(),
    ) else {
        panic!("malformed reader bytes must produce a typed refusal");
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

/// FR-001-AC-3, FR-001-AC-6 and FR-001-AC-8 (TC-006): the equality's
/// operation has no derivable claim, while the sibling negation is supported.
/// Only the negation is routed, retaining its original request index, and the
/// derived claim map records both nodes.
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

    assert_eq!(driven.routed.len(), 1);
    assert_eq!(driven.generation.items.len(), 1);
    assert!(driven.generation.rejected.is_empty());
    assert!(matches!(
        &driven.settlements[0].disposition,
        Disposition::Unsupported {
            cause: Cause::KaniFormRefused {
                refusal: UnsupportedObligation::NoDerivableClaim { .. },
                ..
            }
        }
    ));

    // The equality is not routed; the supported negation retains original
    // request index 1 rather than being renumbered to its routed-list slot.
    let neg_item = &driven.generation.items[0];
    assert_eq!(neg_item.request_index, 1);

    // The sibling settles Supported with its own harness, unaffected by the
    // eq item's refusal.
    let KindOutput::Kani { record, harness } = &neg_item.output else {
        panic!("the routed negation is a Kani item");
    };
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

    // FR-001-AC-8 (SR-013 FND-003): the `-` item's disposition and harness
    // are generated from its own checked request position despite the
    // underivable `=` sibling.
    assert_generation_matches(
        &driven,
        &recorder,
        &[eq.clone(), neg.clone()],
        &[(1, neg.clone())],
    );

    // The generated claim map contains only the routed negation; the
    // negotiation refusal for equality remains in `settlements` above.
    let claim_map = driven
        .generation
        .claim_map
        .as_ref()
        .expect("a Kani group ran, so a claim map is derived");
    assert_eq!(claim_map.items.len(), 1);
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

/// Trace: FR-005-AC-2, FR-005-AC-3.
#[test]
fn provider_trait_run_processes_a_selected_item() {
    let registry = build_registry(&[manifest("kani", 7, &["value-validity"])])
        .expect("the Kani manifest registers");
    let packages = BTreeMap::new();
    let driven = drive(&QslCg, int_input(&packages), &registry, None, kani_input())
        .expect("the Kani item is generated");
    assert_eq!(driven.generation.items.len(), 1);
    let KindOutput::Kani {
        harness: Some(harness),
        ..
    } = &driven.generation.items[0].output
    else {
        panic!("the selected Kani item has a generated harness");
    };
    assert_eq!(harness.identity.ceilings, proof_ceilings());

    let provider = KaniProvider::new(manifest("kani", 7, &["value-validity"]));
    let request = ProviderRequest::new(&driven.generation.items);
    let results = provider.run(
        &driven.emitted,
        &request,
        &ProviderLimits { kani: None },
        &Cancel::new(),
    );

    assert_eq!(results.items.len(), 1);
    assert!(matches!(
        results.items[0].result,
        Err(ProviderItemFailure::MissingExecutionLimits)
    ));
}
