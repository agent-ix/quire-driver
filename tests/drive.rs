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

use ix_trace_rs::trace;
use qsl_foundation::BackendId;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, default_accounting, package, parse, select, Call as QslCall, CallArgument, CheckedUnit,
    CompileRefusal, DependencyInput, EmittedUnit, PackageLimits, ParseRequest, SpineLimits,
    SpineStage,
};
use qsl_replay::DomainKind;
use qsl_route::routing::Disposition as RouteDisposition;
use qsl_route::{Candidate, RegistrationCause};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_contract_codegen::{
    generate_routed, BackendDescriptor, BackendKind, BackendProviderEnvelope, Candidates, Cause,
    ClaimDisposition, ContractItemSettlement, ContractNegotiationError, ContractRequestPosition,
    DerivedDomain, Disposition, EnvelopeRefusal, GenerationContexts, KaniExecutionEvidence,
    KaniForm, KaniGenerationContext, KaniRunOutcome, KaniSolver, KindOutput, MemoryMechanism,
    MemoryObservation, ObligationDisposition, ProofCeilings, RoutedGeneration,
    RoutedGenerationError, RoutedGenerationItem, UnsupportedObligation,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2, CheckedTerm, ContractPackageV1,
};
use quire_driver::{
    build_registry, drive, CompileInput, Driven, DriverFault, DriverRefusal, DriverStage,
    GenerationInput, KaniInput, KaniProvider, OutcomeKind, Provider, ProviderItemFailure,
    ProviderItemResult, ProviderLimits, ProviderManifest, ProviderRequest, QslCg,
    ScalarExecutionInput, Upstream,
};
use quire_exact::{Cancel, CancelCause, Integer, Value};
use quire_semantic_value::catalog::Category;

#[path = "common/units.rs"]
mod shared_units;

/// A unit: the `ix:native` header, the `v` profile selecting QSL's own
/// root profile, then `body`.
fn unit(body: &str) -> String {
    shared_units::unit(body)
}

static UNIT: LazyLock<String> =
    LazyLock::new(|| unit("function f using v(): Boolean pure { true }\n"));

// Minimal model document authored for TC-019. QSL's package_input helper
// keys each document by its declared identity, so this test has no copied fixture.
const CANCEL_MODEL_DOCUMENT: &str = r#"{
  "contractVersion": "2.0.0",
  "canonicalization": {"algorithm": "rfc8785-v1"},
  "source": {
    "identity": "ix://driver/cancellation/model",
    "version": "1.0.0",
    "dialect": "spec-bundle"
  },
  "package": {
    "identity": "driver/cancellation-model",
    "version": "1.0.0",
    "mappingVersions": [],
    "profileVersions": []
  },
  "occurrences": [],
  "extensions": [],
  "constructs": [{
    "kind": {"module": "driver/cancellation-model", "name": "entity"},
    "moduleVersion": "1.0.0",
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

static PARTITION_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit(
        "function a using v(x: Int[0, 9]): Int[-9, 0] pure { -x }\n\
          function b using v(y: Int[0, 9]): Int[-9, 0] pure { -y }\n\
          function c using v(z: Int[0, 9]): Int[-9, 0] pure { -z }\n",
    )
});

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
static OVERFLOW_UNIT: LazyLock<String> = LazyLock::new(shared_units::overflow_unit);

/// A unit whose `+` (`x + 100`) is the same shape `OVERFLOW_UNIT` declares,
/// but reached only through an `if` branch QSL's checker proves dead
/// (`x > 20` never holds over `x: Int[0, 9]`): QSL's `Facts::walk` never
/// discharges a coercion inside a branch it proves unreachable, so the
/// package still emits. CG ignores path conditions (`reachable_results`
/// ranges every operand over its full declared domain, live branch or
/// dead), so routing this `+` to Kani settles the same
/// `Unsupported`/`ResultBoundUnreachable` `OVERFLOW_UNIT`'s direct case
/// never reaches CG to report. The `if` guard (`x > 20`) is itself a scalar
/// comparison, so this unit has two requirement records in QSL occurrence-key
/// order: the guard first and the `+` second for this checked package.
static DEAD_BRANCH_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit("function unreachable_branch using v(x: Int[0, 9]): Int[0, 10] pure { if x > 20 then x + 100 else 0 }\n")
});

/// RR-16: the same `x + 1` node is requested at two occurrences whose
/// direct result conversions have different target ranges.
static RR16_UNIT: LazyLock<String> = LazyLock::new(|| {
    unit(
        "function g using v(p: Int[0, 10]): Boolean pure { true }\n\
          function h using v(q: Int[0, 20]): Boolean pure { true }\n\
          function f using v(x: Int[0, 9]): Boolean pure { g(x + 1) and h(x + 1) }\n",
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
    ExportBindings,
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
    source: RefCell<Option<Arc<CheckedPackageV2>>>,
    source_matches: RefCell<Vec<bool>>,
    export_refused_at: Vec<usize>,
    export_whole_refusal: bool,
    export_unknown_at: Option<usize>,
    export_fault_at: Option<usize>,
    export_wrong_bound_at: Option<usize>,
    export_wrong_unbound_at: Option<usize>,
    export_short_result: bool,
    export_keys: RefCell<Vec<qsl_foundation::source::provenance::OccurrenceKey>>,
    calls: RefCell<Vec<Call>>,
    /// When set, the IR read answers this instead of reading.
    read_answer: Option<CheckedPackageDispatchResult>,
    /// When set, negotiation returns this typed refusal before consulting CG.
    negotiate_answer: Option<EnvelopeRefusal>,
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
    /// Exact E7 lowered package and request-position projection passed to CG.
    generation_input: RefCell<Option<(ContractPackageV1, Vec<ContractRequestPosition>)>>,
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

    fn generation_input(&self) -> (ContractPackageV1, Vec<ContractRequestPosition>) {
        self.generation_input
            .borrow()
            .clone()
            .expect("generation was called")
    }
}

impl Upstream for Recorder {
    fn has_abstraction_relation(&self, package: &qsl_package::CheckedPackage) -> bool {
        !self.export_refused_at.is_empty()
            || self.export_whole_refusal
            || self.export_unknown_at.is_some()
            || self.export_fault_at.is_some()
            || self.export_wrong_bound_at.is_some()
            || self.export_wrong_unbound_at.is_some()
            || self.export_short_result
            || QslCg.has_abstraction_relation(package)
    }
    fn export_bindings(
        &self,
        _package: &qsl_package::CheckedPackage,
        items: &[qsl_foundation::source::provenance::OccurrenceKey],
    ) -> Result<
        Vec<Result<qsl_package::BoundItem, qsl_package::ExportRefusal>>,
        qsl_package::ExportRefusal,
    > {
        self.calls.borrow_mut().push(Call::ExportBindings);
        *self.export_keys.borrow_mut() = items.to_vec();
        if self.export_whole_refusal {
            return Err(fixture_unbound(&items[0]));
        }
        let mut results = items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if self.export_unknown_at == Some(index) {
                    Err(qsl_package::ExportRefusal::UnknownItem { item: item.clone() })
                } else if self.export_fault_at == Some(index) {
                    Err(qsl_package::ExportRefusal::Fault(
                        quire_semantic_value::catalog::InternalFault::new("export", "fixture"),
                    ))
                } else if self.export_wrong_unbound_at == Some(index) {
                    Err(fixture_unbound(&items[1]))
                } else if self.export_refused_at.contains(&index) {
                    Err(fixture_unbound(item))
                } else {
                    Ok(qsl_package::BoundItem {
                        item: if self.export_wrong_bound_at == Some(index) {
                            items[1].clone()
                        } else {
                            item.clone()
                        },
                        bindings: Vec::new(),
                    })
                }
            })
            .collect::<Vec<_>>();
        if self.export_short_result {
            results.pop();
        }
        Ok(results)
    }
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        package_name: &str,
        packages: &qsl_semantics::model::intake::PackageInput,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, qsl_replay::spine::AdmittedModels, EmittedUnit), DriverRefusal> {
        self.calls.borrow_mut().push(Call::Compile);
        QslCg.compile(source, path, bytes, package_name, packages, limits, cancel)
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

    fn backend_kind(&self, descriptor: &BackendDescriptor) -> Option<BackendKind> {
        self.calls
            .borrow_mut()
            .push(Call::BackendKind(descriptor.identity.clone()));
        QslCg.backend_kind(descriptor)
    }

    fn negotiate(
        &self,
        source: &quire_contract_model::CheckedPackageV2,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ContractItemSettlement>, Box<ContractNegotiationError>> {
        self.calls.borrow_mut().push(Call::Negotiate {
            items: envelope.items.len(),
        });
        *self.envelope.borrow_mut() = Some(envelope.clone());
        if let Some(KaniForm::V2ScalarClaim { package, .. }) =
            envelope.items.first().map(|item| &item.kani_form)
        {
            self.source_matches
                .borrow_mut()
                .push(std::ptr::eq(source, package.as_ref()));
            *self.source.borrow_mut() = Some(Arc::clone(package));
        }
        if let Some(refusal) = &self.negotiate_answer {
            return Err(Box::new(ContractNegotiationError::Envelope(
                refusal.clone(),
            )));
        }
        let mut settlements = QslCg.negotiate(source, package, positions, envelope)?;
        if let Some(index) = self.unsupported_at {
            if let Some(settlement) = settlements.get_mut(index) {
                let backend = settlement.backend.as_mut().expect("a lowered item settles");
                let kind = backend
                    .kind
                    .expect("a value-validity item settles with a kind");
                backend.disposition = Disposition::Unsupported {
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
        source: &quire_contract_model::CheckedPackageV2,
        package: &quire_contract_model::ContractPackageV1,
        positions: &[quire_contract_codegen::ContractRequestPosition],
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        self.calls.borrow_mut().push(Call::Generate {
            items: routed.len(),
        });
        *self.generation_input.borrow_mut() = Some((package.clone(), positions.to_vec()));
        self.source_matches.borrow_mut().push(
            self.source
                .borrow()
                .as_ref()
                .is_some_and(|original| std::ptr::eq(source, original.as_ref())),
        );
        QslCg.generate(source, package, positions, routed, contexts)
    }
}

fn fixture_unbound(
    item: &qsl_foundation::source::provenance::OccurrenceKey,
) -> qsl_package::ExportRefusal {
    qsl_package::ExportRefusal::Unbound {
        item: item.clone(),
        elements: vec![qsl_package::UnboundElement {
            key: qsl_semantics::check::AbstractionKey::Population(
                quire_checked::key::DeclarationKey {
                    package: "test/ir914".to_owned(),
                    node: "ConfigVersion".to_owned(),
                },
            ),
            package: "test/ir914".to_owned(),
        }],
    }
}

fn manifest(identity: &str, kinds: &[&str]) -> ProviderManifest {
    ProviderManifest {
        identity: identity.to_owned(),
        advertises: kinds
            .iter()
            .map(|kind| (Some((*kind).to_owned()), Some("bounded".to_owned())))
            .collect(),
        domains: Some(vec!["integer".to_owned()]),
        bounds: None,
        wire: None,
    }
}

fn input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        source: SourceIdentity::new("agent-ix", "driver-test"),
        path: "unit.native",
        bytes: UNIT.as_bytes(),
        package_name: "driver-test",
        packages,
        limits: SpineLimits::default(),
        cancel: Cancel::new(),
    }
}

fn cancellation_source_and_packages() -> (String, qsl_semantics::model::intake::PackageInput) {
    let packages = qsl_semantics::model::intake::package_input([CANCEL_MODEL_DOCUMENT.as_bytes()])
        .expect("the cancellation model document is a package");
    let source = unit(
        "model M = \"driver/cancellation-model\";\n\
         function keep using v(x: M::Thing): Boolean pure { true }\n",
    );
    (source, packages)
}

/// Baseline QSL charge counts after parse, select, check and package. The
/// cancellation fixture includes a model so every stage has a charged path.
fn cancellation_boundaries(
    source_text: &str,
    packages: &qsl_semantics::model::intake::PackageInput,
) -> [usize; 4] {
    let observations = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&observations);
    let cancel = Cancel::observing(move || {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    let source = SourceIdentity::new("agent-ix", "driver-cancellation");
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
        "cancellation".to_owned(),
        &models,
        &DependencyInput::default(),
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

fn int_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: INT_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn partition_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: PARTITION_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn eq_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: EQ_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn two_param_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: TWO_PARAM_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn negate_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: NEGATE_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn overflow_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: OVERFLOW_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn dead_branch_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: DEAD_BRANCH_UNIT.as_bytes(),
        ..input(packages)
    }
}

fn rr16_input(packages: &qsl_semantics::model::intake::PackageInput) -> CompileInput<'_> {
    CompileInput {
        bytes: RR16_UNIT.as_bytes(),
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
                    source: SourceIdentity::new("agent-ix", "driver-cancellation"),
                    package_name: "driver-cancellation",
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
            source: SourceIdentity::new("agent-ix", "driver-cancellation"),
            package_name: "driver-cancellation",
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
    let source = SourceIdentity::new("agent-ix", "driver-cancellation");
    let limits = SpineLimits::default();
    let driven = drive(
        &recorder,
        CompileInput {
            source: source.clone(),
            package_name: "driver-cancellation",
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
    assert_eq!(driven.package().source().identity(), &provision[0].0);
    assert_eq!(driven.package().source().text(), provision[0].1);
    assert!(driven.package().source().text().len() <= limits.source.source_bytes);
    assert!(driven.package().libraries().is_empty());
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
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| node.node_tag.as_ref() == "expression" && node.semantic_form.as_ref() == form)
        .unwrap_or_else(|| panic!("the unit has a {form} expression"));
    node.node_id.clone()
}

/// The one expression node whose `operation.identity` is `identity`: IR's id.
fn expression_by_identity(package: &CheckedPackageV2, identity: &str) -> CheckedNodeId {
    let node = package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "expression"
                && matches!(package.term(&node.node_id), Some(CheckedTerm::Application {
                    operation, ..
                }) if operation.identity() == Some(identity))
        })
        .unwrap_or_else(|| panic!("the unit has a {identity} expression"));
    node.node_id.clone()
}

/// The one `bounded_domain`/`integer_range` node whose literal `min`/`max`
/// values are `lower`/`upper`: the node a Kani derivation names as an
/// operand's, or a comparison's, declared bound (`DerivedDomain::IntegerRange`).
fn integer_range_bound(package: &CheckedPackageV2, lower: &str, upper: &str) -> CheckedNodeId {
    let node = package
        .resolved_graph()
        .nodes
        .iter()
        .find(|node| {
            node.node_tag.as_ref() == "bounded_domain"
                && node.semantic_form.as_ref() == "integer_range"
                && package
                    .term(&node.node_id)
                    .and_then(|term| member_literal(term, "min"))
                    == Some(lower)
                && package
                    .term(&node.node_id)
                    .and_then(|term| member_literal(term, "max"))
                    == Some(upper)
        })
        .unwrap_or_else(|| panic!("the package has an integer_range[{lower}, {upper}] bound"));
    node.node_id.clone()
}

/// The literal text value of member `name` in an `aggregate` node's `body`.
fn member_literal<'a>(body: &'a CheckedTerm, name: &str) -> Option<&'a str> {
    let CheckedTerm::Aggregate { members } = body else {
        return None;
    };
    let binding = members.iter().find(|member| {
        matches!(member, CheckedTerm::Binding { name: member_name, .. } if member_name.as_ref() == name)
    })?;
    let CheckedTerm::Binding { value, .. } = binding else {
        return None;
    };
    let CheckedTerm::Literal { value, .. } = value.as_ref() else {
        return None;
    };
    value.as_str()
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
        scalar_execution: None,
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

#[test]
#[trace("FR-001-AC-21", "TC-017")]
fn relationless_package_skips_export_and_has_no_item_refusals() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
    .expect("relationless unit drives");
    assert!(driven.item_refusals.is_empty());
    assert_eq!(driven.request_items.len(), 1);
    assert_eq!(driven.settlements.len(), 1);
    assert!(!recorder.calls().contains(&Call::ExportBindings));
}

#[test]
#[trace(
    "FR-010-AC-1",
    "FR-010-AC-2",
    "FR-010-AC-3",
    "FR-010-AC-4",
    "FR-001-AC-15",
    "FR-001-AC-17",
    "FR-009-AC-16",
    "TC-017"
)]
fn export_partition_withholds_b_and_preserves_a_and_c_indices() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder {
        export_refused_at: vec![1],
        ..Recorder::default()
    };
    let driven = drive(
        &recorder,
        partition_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the mixed export completes");
    assert_eq!(driven.request_items.len(), 3);
    assert!(recorder.calls().contains(&Call::ExportBindings));
    assert_eq!(recorder.envelope().items.len(), 2);
    let (_, positions) = recorder.generation_input();
    let driver_positions = positions
        .iter()
        .filter(|position| !matches!(position, ContractRequestPosition::ContextRoot { .. }))
        .collect::<Vec<_>>();
    assert_eq!(driver_positions.len(), 3);
    assert!(matches!(
        driver_positions[0],
        ContractRequestPosition::DriverItem { .. }
    ));
    let ContractRequestPosition::Excluded { node_id } = driver_positions[1] else {
        panic!("the refused position must remain excluded");
    };
    assert_eq!(
        serde_json::to_value(
            driven
                .checked()
                .structural_key(node_id)
                .expect("the excluded ID is admitted")
        )
        .expect("the resolved key serializes"),
        qsl_foundation::node_wire::node_key_to_json(driven.request_items[1].node())
            .expect("the selected QSL key serializes"),
    );
    assert!(matches!(
        driver_positions[2],
        ContractRequestPosition::DriverItem { .. }
    ));
    assert_eq!(
        driven
            .settlements
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(
        driven
            .settlements
            .iter()
            .map(|item| item
                .backend
                .as_ref()
                .expect("bound item settles")
                .request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert_eq!(driven.item_refusals.len(), 1);
    assert_eq!(driven.item_refusals[0].request_index, 1);
    assert_eq!(
        driven.item_refusals[0].occurrence,
        *driven.request_items[1].occurrence()
    );
    assert_eq!(
        driven.item_refusals[0].refusal,
        fixture_unbound(driven.request_items[1].occurrence()),
    );
    assert_eq!(
        driven
            .generation()
            .items
            .iter()
            .map(|item| item.request_index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    let terminal_indexes = driven
        .generation()
        .items
        .iter()
        .map(|item| {
            ProviderItemResult {
                request_index: item.request_index,
                backend: item.backend.clone(),
                result: Ok(KaniExecutionEvidence {
                    ceilings: proof_ceilings(),
                    memory: MemoryObservation {
                        mechanism: MemoryMechanism::LinuxPidNamespaceProcfsTreeRss,
                        peak_resident_bytes: Some(16),
                    },
                    symbolic_arguments: Vec::new(),
                    kind: None,
                    harness_path: "fixture::claim".to_owned(),
                    launcher_path: "/tools/cargo-kani".to_owned(),
                    arguments: Vec::new(),
                    unwind: 1,
                    solver: KaniSolver::Cadical,
                    exit_code: Some(0),
                    outcome: KaniRunOutcome::Verified,
                    success_checks: 1,
                    checks: Vec::new(),
                    batch: None,
                    diagnostics: None,
                    failure_cause: None,
                }),
            }
            .terminal_record()
            .expect("fixture provider returns one terminal per generated item")
            .request_index()
            .get()
        })
        .collect::<Vec<_>>();
    assert_eq!(terminal_indexes, [0, 2]);
}

#[test]
#[trace("FR-010-AC-5", "FR-001-AC-18", "TC-017")]
fn all_export_refusals_send_zero_item_request_to_cg() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder {
        export_refused_at: vec![0, 1, 2],
        ..Recorder::default()
    };
    let driven = drive(
        &recorder,
        partition_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("all per-item refusals complete the run");
    assert_eq!(driven.item_refusals.len(), 3);
    assert!(driven.settlements.is_empty());
    assert!(driven.generation().items.is_empty());
    assert!(recorder.calls().contains(&Call::Negotiate { items: 0 }));
    assert!(!recorder
        .calls()
        .iter()
        .any(|call| matches!(call, Call::Generate { .. })));
}

#[test]
#[trace("FR-001-AC-19", "FR-001-AC-20", "TC-017")]
fn export_run_refusal_and_unknown_driver_key_keep_their_distinct_outcomes() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let whole = Recorder {
        export_whole_refusal: true,
        ..Recorder::default()
    };
    let refusal = drive(&whole, int_input(&packages), &registry, None, kani_input())
        .expect_err("whole export refusal stops the run");
    assert_eq!(refusal.stage(), DriverStage::BindingsExport);
    assert_eq!(refusal.kind(), OutcomeKind::Refused);
    assert!(matches!(refusal, DriverRefusal::BindingsExport(ref actual)
        if *actual == fixture_unbound(&whole.export_keys.borrow()[0])));
    let unknown = Recorder {
        export_unknown_at: Some(0),
        ..Recorder::default()
    };
    let fault = drive(
        &unknown,
        int_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect_err("unknown driver-built key is an invariant fault");
    assert_eq!(fault.stage(), DriverStage::Handoff);
    assert_eq!(fault.kind(), OutcomeKind::Fault);
    assert_eq!(fault.code(), "runtime_invariant");
}

#[test]
#[trace("FR-001-AC-19", "FR-001-AC-20", "TC-017")]
fn malformed_export_results_stop_at_the_typed_boundary() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let short = Recorder {
        export_short_result: true,
        ..Recorder::default()
    };
    let refusal = drive(&short, int_input(&packages), &registry, None, kani_input())
        .expect_err("export must return one result per key");
    assert!(matches!(
        refusal,
        DriverRefusal::Fault(DriverFault::ExportCount {
            requested: 1,
            returned: 0
        })
    ));

    let wrong_bound = Recorder {
        export_wrong_bound_at: Some(0),
        ..Recorder::default()
    };
    let refusal = drive(
        &wrong_bound,
        partition_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect_err("a bound item must name the original key");
    assert!(matches!(
        refusal,
        DriverRefusal::Fault(DriverFault::ExportUnknownItem { .. })
    ));

    let wrong_unbound = Recorder {
        export_wrong_unbound_at: Some(0),
        ..Recorder::default()
    };
    let refusal = drive(
        &wrong_unbound,
        partition_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect_err("an unbound item must name the original key");
    assert!(matches!(
        refusal,
        DriverRefusal::Fault(DriverFault::ExportUnknownItem { .. })
    ));

    let faulted = Recorder {
        export_fault_at: Some(0),
        ..Recorder::default()
    };
    let refusal = drive(
        &faulted,
        int_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect_err("an export fault refuses the whole run");
    assert!(matches!(
        &refusal,
        DriverRefusal::BindingsExport(qsl_package::ExportRefusal::Fault(_))
    ));
    assert_eq!(refusal.stage(), DriverStage::BindingsExport);
    assert_eq!(refusal.kind(), OutcomeKind::Fault);
}

fn routed_generation_item(
    request_index: usize,
    node_id: CheckedNodeId,
    backend: quire_contract_codegen::Candidate,
) -> RoutedGenerationItem {
    let descriptor = quire_contract_codegen::BackendDescriptor {
        identity: backend.identity.clone(),
        advertised: vec![(
            quire_contract_codegen::CapabilityKind::ValueValidity,
            quire_contract_codegen::Mode::Bounded,
        )],
        origin: quire_contract_codegen::ProviderOrigin::Linked,
        domains: Some(std::collections::BTreeSet::from([DomainKind::Integer])),
        bounds: None,
    };
    RoutedGenerationItem::from_descriptor(request_index, node_id, backend, &descriptor)
        .expect("the Kani candidate matches its linked descriptor")
}

fn direct_generation(
    recorder: &Recorder,
    routed: &[RoutedGenerationItem],
    contexts: &GenerationContexts<'_>,
) -> Result<RoutedGeneration, RoutedGenerationError> {
    let (package, positions) = recorder.generation_input();
    let routed = routed
        .iter()
        .cloned()
        .map(|item| {
            let selection = positions
                .iter()
                .filter(|position| !matches!(position, ContractRequestPosition::ContextRoot { .. }))
                .nth(item.request_index)
                .and_then(|position| match position {
                    ContractRequestPosition::DriverItem {
                        result_selection, ..
                    } => Some(result_selection),
                    ContractRequestPosition::Excluded { .. }
                    | ContractRequestPosition::ContextRoot { .. } => None,
                })
                .expect("the request index has an explicit lowering position")
                .clone();
            item.with_scalar_selection(&package, selection)
        })
        .collect::<Vec<_>>();
    let source = recorder.source.borrow();
    generate_routed(
        source.as_deref().expect("negotiation retained its source"),
        &package,
        &positions,
        &routed,
        contexts,
    )
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
        manifest("alpha", &["value-validity"]),
        manifest("kani", &["value-validity"]),
        manifest("Zeta", &["operation-contract"]),
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
    let checked = negate_package().checked().clone();
    let neg = expression_by_identity(&checked, "quire.op.integer.negate");

    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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

    // E4 emitted v2 bytes; E5 admitted them.
    assert!(!driven.emitted().bytes().is_empty());
    assert_eq!(
        driven.checked(),
        &checked,
        "the same unit, the same package"
    );

    let dispositions: Vec<serde_json::Value> = driven
        .settlements
        .iter()
        .map(|s| {
            serde_json::to_value(
                &s.backend
                    .as_ref()
                    .expect("a lowered item settles")
                    .disposition,
            )
            .expect("dispositions serialize")
        })
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
    let expected = direct_generation(
        &recorder,
        &[routed_generation_item(0, neg.clone(), backend.clone())],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation(), &expected);
    assert_eq!(driven.generation().items.len(), 1);
    assert_eq!(driven.generation().items[0].request_index, 0);
    assert_eq!(driven.generation().items[0].backend, backend);
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
    let KindOutput::Kani { record, harness } = &driven.generation().items[0].output else {
        panic!(
            "expected Kani output, got {:?}",
            driven.generation().items[0].output
        );
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
        .generation()
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

/// Trace: FR-001-AC-25 (TC-001). The caller's exact E4 ceiling reaches the
/// E5 hand-off; reducing it by one byte refuses emission before E5.
#[test]
fn tc_001_caller_emit_ceiling_bounds_e4_and_admits_at_e5() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    let ordinary = drive(
        &QslCg,
        input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("ordinary emission admits");
    let byte_len = u64::try_from(ordinary.emitted().bytes().len()).expect("byte length fits u64");
    assert!(byte_len > 0);

    let exact = Recorder::default();
    let exact_driven = drive(
        &exact,
        CompileInput {
            limits: SpineLimits {
                package: SpineLimits::default().package.with_output_bytes(byte_len),
                ..SpineLimits::default()
            },
            ..input(&packages)
        },
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect("the exact caller ceiling admits at E5");
    assert_eq!(
        u64::try_from(exact_driven.emitted().bytes().len()).expect("byte length fits u64"),
        byte_len
    );
    assert!(exact.calls().contains(&Call::ReadPackage));

    let low = Recorder::default();
    let refusal = drive(
        &low,
        CompileInput {
            limits: SpineLimits {
                package: SpineLimits::default()
                    .package
                    .with_output_bytes(byte_len - 1),
                ..SpineLimits::default()
            },
            ..input(&packages)
        },
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect_err("one byte below the emitted length refuses");
    assert_eq!(refusal.stage(), DriverStage::Compile(SpineStage::Emit));
    assert_eq!(refusal.kind(), OutcomeKind::Limit);
    assert_eq!(low.calls(), [Call::Compile]);
}

/// Trace: FR-001-AC-7; TC-006 step 11.
#[test]
fn tc_006_handoff_passes_domain_package_evidence_unchanged() {
    let packages = qsl_semantics::model::intake::package_input([CANCEL_MODEL_DOCUMENT.as_bytes()])
        .expect("the cancellation model document is a package");
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
    let (identity, document) = packages.iter().next().expect("one supplied domain package");
    expected
        .insert_domain_package_document(identity.as_str(), document.as_slice())
        .expect("the evidence map contains one document per identity");
    expected.support_feature("quire.value.complete/v1");

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
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
        .map(|s| {
            serde_json::to_value(
                &s.backend
                    .as_ref()
                    .expect("a lowered item settles")
                    .disposition,
            )
            .expect("dispositions serialize")
        })
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
    assert_eq!(driven.generation(), &RoutedGeneration::default());
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
    let registry = build_registry(&[manifest("kani", &["operation-contract"])])
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
        .map(|s| {
            serde_json::to_value(
                &s.backend
                    .as_ref()
                    .expect("a lowered item settles")
                    .disposition,
            )
            .expect("dispositions serialize")
        })
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

/// Trace: FR-001-AC-6, FR-001-AC-12.
/// The driver's generation output for `INT_UNIT`'s
/// `+` (`x + 1`) equals a direct `generate_routed` call over the same
/// routed item, whatever CG settles it -- the same pass-through check
/// `tc_001_tc_006_steps_run_in_order_through_generation` makes for its own
/// fixture. This holds regardless of IR-302 below: the driver passes CG's
/// answer through unchanged either way.
#[test]
fn tc_006_x_plus_1_generation_passes_through_unchanged() {
    let checked = int_package().checked().clone();
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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

    let envelope = recorder.envelope();
    let (contract, positions) = recorder.generation_input();
    assert_eq!(
        envelope.items[0].kani_form,
        KaniForm::V2ScalarClaim {
            package: Arc::new(driven.checked().clone()),
            contract: Box::new(contract.clone()),
            positions: positions.clone(),
            request_index: 0,
            node_id: add.clone(),
        },
        "CG receives the admitted package and the exact requirement node"
    );

    let backend = kani_candidate(&driven);
    let expected = direct_generation(
        &recorder,
        &[routed_generation_item(0, add, backend)],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation(), &expected);
}

/// The CG negotiation input carries the exact admitted checked package and
/// selected requirement node, even when later CG settlement does not route it.
#[trace("TC-006", "FR-001-AC-12", "FR-001-AC-15")]
#[test]
fn tc_006_kani_form_projects_exact_package_and_selected_node() {
    let checked = int_package().checked().clone();
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
    .expect("checked-form negotiation completes");

    let envelope = recorder.envelope();
    let (contract, positions) = recorder.generation_input();
    assert_eq!(envelope.items.len(), 1);
    assert_eq!(
        *recorder.source_matches.borrow(),
        [true, true],
        "negotiation and generation borrow the same original admitted source"
    );
    let KaniForm::V2ScalarClaim { package, .. } = &envelope.items[0].kani_form else {
        panic!("a scalar claim must retain its admitted source");
    };
    assert!(
        std::ptr::eq(package.as_ref(), driven.checked()),
        "the completed driver retains the original source shared with the claim"
    );
    assert_eq!(
        envelope.items[0].kani_form,
        KaniForm::V2ScalarClaim {
            package: Arc::new(driven.checked().clone()),
            contract: Box::new(contract),
            positions,
            request_index: 0,
            node_id: add,
        },
        "the typed form is projected from the admitted package and exact selected node"
    );
}

#[trace("TC-004", "TC-006", "FR-001-AC-5", "FR-001-AC-15")]
#[test]
fn foreign_admitted_source_is_refused_by_negotiation_and_generation() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let original = drive(
        &recorder,
        int_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the original admitted source generates");
    let foreign = drive(
        &QslCg,
        CompileInput {
            package_name: "foreign",
            ..int_input(&packages)
        },
        &registry,
        None,
        kani_input(),
    )
    .expect("a separate named source admits");
    let (contract, positions) = recorder.generation_input();
    let expected = contract.wire().package.source.clone();
    let negotiation = QslCg
        .negotiate(
            foreign.checked(),
            &contract,
            &positions,
            &recorder.envelope(),
        )
        .expect_err("a foreign source must not negotiate the original lowering");
    let generation = QslCg
        .generate(
            foreign.checked(),
            &contract,
            &positions,
            &[],
            &GenerationContexts::default(),
        )
        .expect_err("a foreign source must not generate the original lowering");
    for (refusal, code, kind, stage) in [
        (
            DriverRefusal::from(*negotiation),
            "runtime_invariant",
            OutcomeKind::Fault,
            DriverStage::Negotiation,
        ),
        (
            DriverRefusal::Generation(generation),
            "invalid-request",
            OutcomeKind::Refused,
            DriverStage::Generation,
        ),
    ] {
        assert_eq!(refusal.code(), code);
        assert_eq!(refusal.kind(), kind);
        assert_eq!(refusal.stage(), stage);
        let error = match &refusal {
            DriverRefusal::PackageNegotiation(error) => error,
            DriverRefusal::Generation(RoutedGenerationError::Package(error)) => error,
            other => panic!("expected typed source mismatch, got {other:?}"),
        };
        let quire_contract_codegen::PackageRequestError::SourceMismatch {
            expected: actual_expected,
            actual,
        } = error
        else {
            panic!("the source identity distinction must remain typed");
        };
        assert_eq!(actual_expected.as_ref(), &expected);
        assert_eq!(actual.name.as_ref(), foreign.checked().name());
        assert_eq!(&actual.lock, foreign.checked().lock());
        assert_ne!(actual.name.as_ref(), original.checked().name());
    }
}

/// Trace: FR-001-AC-12.
/// Provenance: QSL RR-16.
/// Two occurrences of the same addition keep their own selected narrow
/// conversion roots through IR lowering and CG generation.
#[test]
fn tc_006_rr16_preserves_each_occurrence_conversion_context() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        rr16_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("the RR-16 unit drives");

    assert_eq!(driven.routed.len(), 2, "only the two add occurrences route");
    assert_eq!(
        driven.generation().items.len(),
        2,
        "CG emits two request items"
    );

    let qsl_requirements = driven.package().package().graph().requirements();
    let expected_contexts = qsl_requirements
        .iter()
        .filter_map(|(application, requirement)| {
            requirement
                .conversion_context()
                .map(|conversion| (application, conversion))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        expected_contexts.len(),
        2,
        "QSL supplies two narrow contexts"
    );

    let (lowered, positions) = recorder.generation_input();
    assert!(!lowered.wire().feature_accounting.is_empty());
    assert_eq!(positions.len(), 4, "[add, narrow, add, narrow]");
    let envelope = recorder.envelope();
    assert_eq!(envelope.items.len(), 2);
    for (request_index, request) in envelope.items.iter().enumerate() {
        let KaniForm::V2ScalarClaim {
            package,
            contract,
            positions: form_positions,
            request_index: form_index,
            node_id,
        } = &request.kani_form
        else {
            panic!("expected a typed scalar claim, got {:?}", request.kani_form);
        };
        let expected_node = positions
            .iter()
            .filter(|position| !matches!(position, ContractRequestPosition::ContextRoot { .. }))
            .nth(request_index)
            .and_then(|position| match position {
                ContractRequestPosition::DriverItem { node_id, .. } => Some(node_id),
                ContractRequestPosition::Excluded { .. }
                | ContractRequestPosition::ContextRoot { .. } => None,
            })
            .expect("each request has a lowered driver position");
        assert_eq!(package.as_ref(), driven.checked());
        assert_eq!(contract.as_ref(), &lowered);
        assert_eq!(form_positions, &positions);
        assert_eq!(*form_index, request_index);
        assert_eq!(node_id, expected_node);
    }
    let mut actual_contexts = Vec::new();
    let (position_pairs, remainder) = positions.as_chunks::<2>();
    assert!(remainder.is_empty(), "positions are grouped into pairs");
    for pair in position_pairs {
        let [ContractRequestPosition::DriverItem {
            node_id,
            result_selection:
                quire_contract_codegen::ContractDriverResultSelection::OwnResult {
                    application,
                    conversion: Some(conversion),
                },
        }, ContractRequestPosition::ContextRoot {
            node_id: context_root,
        }] = pair
        else {
            panic!("each driver add is followed by its direct narrow root: {pair:?}");
        };
        assert_eq!(application.node_id, *node_id);
        assert_eq!(conversion.node_id, *context_root);
        actual_contexts.push((application, conversion));
    }

    let assert_key = |id: &CheckedNodeId, key: &quire_exact::NodeKey| {
        assert_eq!(
            serde_json::to_value(
                driven
                    .checked()
                    .structural_key(id)
                    .expect("the occurrence ID is admitted")
            )
            .expect("the resolved key serializes"),
            qsl_foundation::node_wire::node_key_to_json(key)
                .expect("the selected QSL key serializes")
        );
    };
    for ((expected_application, expected_conversion), (application, conversion)) in
        expected_contexts.iter().zip(actual_contexts.iter())
    {
        assert_key(&application.node_id, expected_application.node());
        assert_eq!(
            application.role,
            quire_contract_model::CheckedOccurrenceRole::Expression
        );
        assert_eq!(application.ordinal, expected_application.origin().ordinal());
        assert_key(&conversion.node_id, expected_conversion.node());
        assert_eq!(
            conversion.role,
            quire_contract_model::CheckedOccurrenceRole::Expression
        );
        assert_eq!(conversion.ordinal, expected_conversion.origin().ordinal());
    }
    assert_ne!(
        actual_contexts[0].0.ordinal, actual_contexts[1].0.ordinal,
        "the shared add node retains distinct request occurrences"
    );
    assert_ne!(
        actual_contexts[0].1.node_id, actual_contexts[1].1.node_id,
        "each add retains its own conversion root"
    );

    let backend = kani_candidate(&driven);
    let expected = direct_generation(
        &recorder,
        &[
            routed_generation_item(0, actual_contexts[0].0.node_id.clone(), backend.clone()),
            routed_generation_item(1, actual_contexts[1].0.node_id.clone(), backend),
        ],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates each routed occurrence with its own context");
    assert_eq!(driven.generation(), &expected);
}

/// Trace: FR-001-AC-13, FR-001-AC-14, FR-001-AC-15, FR-001-AC-16, FR-009-AC-13, TC-006.
/// Provenance: QSL RR-16.
#[test]
fn tc_006_rr16_binds_each_exact_occurrence_to_its_generated_item() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let cancel = Cancel::new();
    let mut compile = rr16_input(&packages);
    compile.cancel = cancel.clone();
    let call = QslCall {
        function: "f".to_owned(),
        arguments: vec![CallArgument {
            parameter: "x".to_owned(),
            value: Value::Integer(Integer::from(3_i64)),
        }],
        accounting: default_accounting(1_000_000),
        supplied: Default::default(),
    };
    let environment = ObjectEnvironment::default()
        .with_population_sizes(BTreeMap::from([("rr16-marker".to_owned(), 2)]));
    let mut generation = kani_input();
    generation.scalar_execution = Some(ScalarExecutionInput {
        call: &call,
        replay_limits: qsl_replay::ReplayLimits::default(),
        environment: &environment,
        native: None,
    });

    let recorder = Recorder::default();
    let driven = drive(&recorder, compile, &registry, None, generation)
        .expect("the production driver binds and generates both scalar occurrences");

    assert_eq!(driven.routed.len(), 2);
    assert_eq!(driven.scalar_executions().len(), 2);
    let first = &driven.scalar_executions()[0];
    let second = &driven.scalar_executions()[1];
    assert_eq!((first.request_index(), second.request_index()), (0, 1));
    for retained in [first.call(), second.call()] {
        assert_eq!(retained.function, call.function);
        assert_eq!(retained.arguments.len(), call.arguments.len());
        assert_eq!(retained.arguments[0].parameter, call.arguments[0].parameter);
        assert_eq!(
            format!("{:?}", retained.arguments[0].value),
            format!("{:?}", call.arguments[0].value)
        );
        assert_eq!(
            format!("{:?}", retained.accounting),
            format!("{:?}", call.accounting)
        );
        assert_eq!(retained.supplied, call.supplied);
    }
    assert_eq!(first.environment().population_size("rr16-marker"), Some(2));
    assert_eq!(second.environment().population_size("rr16-marker"), Some(2));
    assert_eq!(first.cancel(), &cancel);
    assert_eq!(second.cancel(), &cancel);
    assert_eq!(first.call().function, "f");
    assert_eq!(second.call().function, "f");
    assert_eq!(first.replay_limits(), qsl_replay::ReplayLimits::default());
    assert_eq!(second.replay_limits(), qsl_replay::ReplayLimits::default());
    assert_eq!(first.operands().values(), [3, 1]);
    assert_eq!(second.operands().values(), [3, 1]);
    for (execution, generated) in [first, second].into_iter().zip(&driven.generation().items) {
        let quire_contract_codegen::KindOutput::Kani {
            harness: Some(harness),
            ..
        } = &generated.output
        else {
            panic!("each routed RR-16 item has a scalar harness");
        };
        assert_eq!(generated.request_index, execution.request_index());
        assert_eq!(
            execution.operands().operands().len(),
            harness.identity.arguments.len()
        );
        for (ordinal, (operand, argument)) in execution
            .operands()
            .operands()
            .iter()
            .zip(&harness.identity.arguments)
            .enumerate()
        {
            assert_eq!(operand.ordinal, ordinal as u64);
            assert_eq!(operand.range.lower, i128::from(argument.minimum));
            assert_eq!(operand.range.upper, i128::from(argument.maximum));
        }
    }
    assert_ne!(
        first.occurrence().occurrence,
        second.occurrence().occurrence
    );
    assert_eq!(first.operands().occurrence(), first.occurrence());
    assert_eq!(second.operands().occurrence(), second.occurrence());
    assert!(first.receipt().is_none() && second.receipt().is_none());

    let calls = recorder.calls();
    assert_eq!(
        calls.iter().filter(|call| **call == Call::Compile).count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| **call == Call::ReadPackage)
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::Generate { .. }))
            .count(),
        1
    );

    let selections = driven
        .generation()
        .items
        .iter()
        .map(|item| item.request_index)
        .collect::<Vec<_>>();
    assert_eq!(
        selections,
        [0, 1],
        "context roots do not shift routed indices"
    );
}

/// Trace: FR-001-AC-16, FR-009-AC-13, TC-006.
#[test]
fn tc_006_no_selected_scalar_item_skips_binding_and_native_execution() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let call = QslCall {
        function: "f".to_owned(),
        arguments: Vec::new(),
        accounting: default_accounting(1_000_000),
        supplied: Default::default(),
    };
    let environment = ObjectEnvironment::default();
    let mut generation = kani_input();
    generation.scalar_execution = Some(ScalarExecutionInput {
        call: &call,
        replay_limits: qsl_replay::ReplayLimits::default(),
        environment: &environment,
        native: None,
    });

    let driven = drive(
        &QslCg,
        CompileInput {
            bytes: UNIT.as_bytes(),
            ..input(&packages)
        },
        &registry,
        None,
        generation,
    )
    .expect("a source with no scalar requirement runs without scalar fallback");

    assert!(driven.request_items.is_empty());
    assert!(driven.routed.is_empty());
    assert!(driven.generation().items.is_empty());
    assert!(driven.scalar_executions().is_empty());
}

/// FR-001-AC-6 (TC-006): the driver's generation output for
/// `TWO_PARAM_UNIT`'s `+` (`x + y`) equals a direct `generate_routed` call
/// over the same routed item, whatever CG settles it. See
/// `tc_006_x_plus_1_generation_passes_through_unchanged`.
#[test]
fn tc_006_x_plus_y_generation_passes_through_unchanged() {
    let checked = two_param_package().checked().clone();
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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

    let backend = kani_candidate(&driven);
    let expected = direct_generation(
        &recorder,
        &[routed_generation_item(0, add, backend)],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the routed item");
    assert_eq!(driven.generation(), &expected);
}

/// FR-001-AC-6 (TC-006), IR-302: `INT_UNIT`'s `+` (`x + 1`) settles
/// `Supported`. CG's IR-302 reads a `reference` to a `value` node whose
/// body is a literal as a literal operand (QSL's own shape for `1`), not a
/// plain-typed unbounded reference: the literal owns no bound and is
/// constrained to its own value, so only `x`'s own `[0, 9]` domain ranges
/// the first argument.
#[test]
fn tc_006_x_plus_1_settles_supported() {
    let checked = int_package().checked().clone();
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();

    let driven = drive(&QslCg, int_input(&packages), &registry, None, kani_input())
        .expect("the run completes");

    let KindOutput::Kani { record, harness } = &driven.generation().items[0].output else {
        panic!(
            "expected Kani output, got {:?}",
            driven.generation().items[0].output
        );
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
    let checked = two_param_package().checked().clone();
    let add = expression(&checked, "binary");
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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

    let KindOutput::Kani { record, harness } = &driven.generation().items[0].output else {
        panic!(
            "expected Kani output, got {:?}",
            driven.generation().items[0].output
        );
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

/// FR-001-AC-6 (TC-006), IR-302: `DEAD_BRANCH_UNIT` compiles -- QSL's
/// `Facts::walk` proves `x > 20` is never true over `x: Int[0, 9]` and
/// never discharges the dead branch's coercion, so the checked package
/// still emits. The unit carries two scalar operation applications --
/// the guard `x > 20` and the dead branch's `x + 100` -- so two
/// requirement records, both bounded (`x`'s own `[0, 9]` domain). CG's
/// selected-result preflight refuses both before routing. The `+` item is
/// `Unsupported`/`ResultBoundUnreachable`: CG ignores the
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
    .checked()
    .clone();
    let x_bound = integer_range_bound(&checked, "0", "9");
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
    assert!(driven.generation().items.is_empty());
    let add_settlement = driven
        .settlements
        .iter()
        .find(|settlement| {
            matches!(
                settlement
                    .backend
                    .as_ref()
                    .map(|backend| &backend.disposition),
                Some(Disposition::Unsupported {
                    cause: Cause::KaniFormRefused {
                        refusal: UnsupportedObligation::ResultBoundUnreachable { .. },
                        ..
                    }
                })
            )
        })
        .expect("the dead branch's `+` settles ResultBoundUnreachable");
    assert!(matches!(
        &add_settlement.backend.as_ref().expect("a lowered item settles").disposition,
        Disposition::Unsupported {
            cause: Cause::KaniFormRefused {
                refusal: UnsupportedObligation::ResultBoundUnreachable {
                    operation_identity,
                    lower: 0,
                    upper: 10,
                    reachable_lower,
                    reachable_upper,
                },
                ..
            }
        } if operation_identity == "quire.op.integer.add"
            && reachable_lower == "100"
            && reachable_upper == "109"
    ));
    assert!(matches!(
        &driven.settlements[1].backend.as_ref().expect("a lowered item settles").disposition,
        Disposition::Unsupported {
            cause: Cause::KaniFormRefused {
                refusal: UnsupportedObligation::OperationNotRendered {
                    operation_identity,
                    derived_domains,
                },
                ..
            }
        } if operation_identity == "quire.op.integer.gt"
            && derived_domains == &vec![DerivedDomain::IntegerRange {
                bound: x_bound,
                lower: "0".to_owned(),
                upper: "9".to_owned(),
            }]
    ));
}

/// FR-001-AC-5 and FR-001-AC-6 (TC-007): a routed Kani item with no Kani
/// context refuses the run with CG's `MissingKindContext`.
#[test]
fn tc_007_generation_refusals_are_typed() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
    let registry = build_registry(&[manifest("kani", &["operation-contract"])])
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
        .map(|s| {
            serde_json::to_value(
                &s.backend
                    .as_ref()
                    .expect("a lowered item settles")
                    .disposition,
            )
            .expect("dispositions serialize")
        })
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
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
        .map(|s| {
            serde_json::to_value(
                &s.backend
                    .as_ref()
                    .expect("a lowered item settles")
                    .disposition,
            )
            .expect("dispositions serialize")
        })
        .collect();
    assert_eq!(dispositions.len(), 2);
    assert_eq!(dispositions[0]["disposition"], "unsupported");
    assert_eq!(dispositions[1]["disposition"], "supported");

    // Positional: `routed`'s only element, at list position 0, carries
    // request index 1 -- the gap the rewritten item 0 left.
    let routed_indexes: Vec<usize> = driven.routed.iter().map(|r| r.request_index).collect();
    assert_eq!(routed_indexes, [1]);
    let generated_indexes: Vec<usize> = driven
        .generation()
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
    let neg = expression_by_identity(driven.checked(), "quire.op.integer.negate");
    let backend = kani_candidate(&driven);
    let expected = direct_generation(
        &recorder,
        &[routed_generation_item(1, neg, backend)],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the `-` item alone");
    assert_eq!(driven.generation(), &expected);

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

/// Trace: FR-001-AC-5.
/// A typed CG envelope fault remains an internal fault at negotiation and
/// stops the driver before routing or generation.
#[test]
fn tc_004_repeated_manifest_identity_from_negotiation_is_an_internal_fault() {
    let registry = build_registry(&[]).expect("an empty registry builds");
    let packages = BTreeMap::new();
    let recorder = Recorder {
        negotiate_answer: Some(EnvelopeRefusal::ManifestRepeatsBackendIdentity {
            identity: "plugin:example".to_owned(),
        }),
        ..Recorder::default()
    };

    let refusal = drive(
        &recorder,
        int_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect_err("CG's repeated manifest identity is not a caller refusal");

    assert_eq!(refusal.stage(), DriverStage::Negotiation);
    assert_eq!(refusal.code(), "runtime_invariant");
    assert_eq!(refusal.kind(), OutcomeKind::Fault);
    assert!(refusal.to_string().contains("plugin:example"));
    assert!(
        !recorder
            .calls()
            .iter()
            .any(|call| matches!(call, Call::Route { .. } | Call::Generate { .. })),
        "routing and generation stop after the negotiation fault: {:?}",
        recorder.calls()
    );
}

/// FR-001-AC-3 and FR-001-AC-6 (TC-003): a unit with no scalar operation
/// application writes no requirement record (QSL FR-075), so the driver
/// routes nothing and calls neither `route` nor `generate` -- with no
/// caller input naming an item, `UNIT` (`function f(): Boolean pure { true
/// }`) still drives to completion.
#[test]
fn tc_003_a_unit_with_no_scalar_applications_routes_nothing() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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
    assert_eq!(driven.generation(), &RoutedGeneration::default());
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

    // Two manifests, one identity, different advertisements: the identity is
    // withdrawn and `refusals()` reports it once.
    let expected_conflicts = [(
        Candidate::new(BackendId::new("kani")),
        RegistrationCause::DuplicateBackend,
    )];
    let refusal = build_registry(&[
        manifest("kani", &["value-validity"]),
        manifest("kani", &["operation-contract"]),
    ])
    .expect_err("one identity with two manifests conflicts");
    assert_eq!(refusal.stage(), DriverStage::Registry);
    assert_eq!(refusal.code(), "invalid_capability/duplicate-backend");
    assert_eq!(registration_refusals(&refusal), expected_conflicts);

    // A, B, then A again: the repeat does not add a third, duplicate
    // refusal -- `refusals()` is the registry's own deduped account, not an
    // accumulation of what each `register` call individually returned.
    let refusal = build_registry(&[
        manifest("kani", &["value-validity"]),
        manifest("kani", &["operation-contract"]),
        manifest("kani", &["value-validity"]),
    ])
    .expect_err("A, B, A still conflicts, without a duplicated refusal");
    assert_eq!(registration_refusals(&refusal), expected_conflicts);

    let refusal = build_registry(&[manifest("kani", &["not-a-kind"])])
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

/// FR-001-AC-5 and FR-001-AC-7 (TC-004): an IR answer refusing the emitted
/// bytes is an internal fault at the hand-off, and nothing after the IR read
/// runs.
#[test]
fn tc_004_e5_refusal_is_a_fault() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the kani manifest registers");
    let packages = BTreeMap::new();
    let CheckedPackageDispatchResult::Refused(refusal) = quire_contract_model::read_checked_package(
        b"not a JSON document",
        CheckedPackageReadLimits::bounded(),
        &CheckedPackageEvidence::new(),
    ) else {
        panic!("malformed reader bytes must produce a typed refusal");
    };

    let recorder = Recorder {
        read_answer: Some(CheckedPackageDispatchResult::Refused(refusal.clone())),
        ..Recorder::default()
    };
    let driven = drive(
        &recorder,
        int_input(&packages),
        &registry,
        None,
        GenerationInput::default(),
    )
    .expect_err("E5 refuses the emitted bytes");
    match &driven {
        DriverRefusal::Fault(DriverFault::HandoffRefused(refused)) => {
            assert_eq!(refused.as_ref(), &refusal);
        }
        _ => panic!("unexpected refusal {driven:?}"),
    }
    assert_eq!(driven.stage(), DriverStage::Handoff);
    assert_eq!(driven.kind(), OutcomeKind::Fault);
    assert_eq!(driven.code(), "runtime_invariant");
    assert_eq!(recorder.calls(), [Call::Compile, Call::ReadPackage]);
}

/// FR-001-AC-7 (TC-006): E5 admits units with text and rational parameter
/// types.
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
        drive(
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
        assert_eq!(recorder.calls()[..2], [Call::Compile, Call::ReadPackage]);
    }
}

/// Trace: FR-001-AC-12.
/// CG receives the typed forms from the admitted package. It refuses the
/// equality form during negotiation, before generation, while the sibling
/// negation remains routable and generates its own claim.
#[test]
fn tc_006_typed_form_refusal_preserves_supported_sibling() {
    let checked = eq_package().checked().clone();
    let eq = expression_by_identity(&checked, "quire.op.integer.eq");
    let neg = expression_by_identity(&checked, "quire.op.integer.negate");

    let registry = build_registry(&[manifest("kani", &["value-validity"])])
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

    let envelope = recorder.envelope();
    let (contract, positions) = recorder.generation_input();
    assert_eq!(envelope.items.len(), 2);
    assert_eq!(
        envelope.items[0].kani_form,
        KaniForm::V2ScalarClaim {
            package: Arc::new(driven.checked().clone()),
            contract: Box::new(contract.clone()),
            positions: positions.clone(),
            request_index: 0,
            node_id: eq.clone(),
        }
    );
    assert_eq!(
        envelope.items[1].kani_form,
        KaniForm::V2ScalarClaim {
            package: Arc::new(driven.checked().clone()),
            contract: Box::new(contract),
            positions,
            request_index: 1,
            node_id: neg.clone(),
        }
    );
    assert!(matches!(
        &driven.settlements[0].backend.as_ref().expect("a lowered item settles").disposition,
        Disposition::Unsupported {
            cause: Cause::KaniFormRefused { form, .. }
        } if form.operation_identity.as_deref() == Some("quire.op.integer.eq")
    ));
    assert!(matches!(
        driven.settlements[1]
            .backend
            .as_ref()
            .expect("a lowered item settles")
            .disposition,
        Disposition::Supported { .. }
    ));

    assert_eq!(driven.routed.len(), 1);
    assert_eq!(driven.routed[0].request_index, 1);
    assert_eq!(driven.generation().items.len(), 1);
    assert!(driven.generation().rejected.is_empty());

    // The sibling's generation output is keyed by its original request index.
    let neg_item = &driven.generation().items[0];
    assert_eq!(neg_item.request_index, 1);
    let KindOutput::Kani { record, harness } = &neg_item.output else {
        panic!("expected Kani output, got {:?}", neg_item.output);
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

    // FR-001-AC-8: the `-` item's disposition and its
    // whole harness equal a direct `generate_routed` call over the `-`
    // item alone, over the same package -- the underivable `=` sibling
    // leaks into neither.
    let backend = kani_candidate(&driven);
    let alone = direct_generation(
        &recorder,
        &[routed_generation_item(1, neg.clone(), backend)],
        &GenerationContexts {
            kani: Some(KaniGenerationContext {
                ceilings: proof_ceilings(),
                subject_path: "crate::subject",
                unwind: 1,
            }),
        },
    )
    .expect("CG generates for the `-` item alone");
    assert_eq!(*neg_item, alone.items[0]);

    // Only routed items reach claim generation; the equality item was
    // refused earlier by typed form negotiation.
    let claim_map = driven
        .generation()
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
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the Kani manifest registers");
    let packages = BTreeMap::new();
    let driven = drive(&QslCg, int_input(&packages), &registry, None, kani_input())
        .expect("the Kani item is generated");
    assert_eq!(driven.generation().items.len(), 1);
    let KindOutput::Kani {
        harness: Some(harness),
        ..
    } = &driven.generation().items[0].output
    else {
        panic!("the selected Kani item has a generated harness");
    };
    assert_eq!(harness.identity.ceilings, proof_ceilings());

    let provider = KaniProvider::new(manifest("kani", &["value-validity"]));
    let request = ProviderRequest::new(&driven.generation().items);
    let results = provider.run(
        driven.emitted(),
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

#[trace("FR-001-AC-12", "FR-001-AC-16", "TC-006")]
#[test]
fn current_owner_boundary_raw_slots_resolve_in_each_holding_row() {
    let registry = build_registry(&[manifest("kani", &["value-validity"])])
        .expect("the native backend registers");
    let packages = BTreeMap::new();
    let recorder = Recorder::default();
    let driven = drive(
        &recorder,
        partition_input(&packages),
        &registry,
        None,
        kani_input(),
    )
    .expect("each native operation resolves in its own admitted frame");
    let package = driven.checked();
    let raw = &package.graph().nodes;
    let collision = raw.iter().enumerate().find_map(|(left, first)| {
        raw.iter()
            .enumerate()
            .skip(left + 1)
            .find_map(|(right, second)| {
                (first.node_id == second.node_id
                    && first.node_id.path.iter().any(|segment| {
                        matches!(
                            segment,
                            quire_contract_model::CheckedNodePathSegment::Operand(_)
                        )
                    })
                    && package.structural_key_at(left) != package.structural_key_at(right))
                .then_some((left, right))
            })
    });
    let (left, right) =
        collision.expect("authentic repeated operations have equal raw slots in distinct rows");
    assert!(
        package.structural_key(&raw[left].node_id).is_none(),
        "raw bytes alone cannot select a holder"
    );
    assert_ne!(
        package.document_held_id(left),
        package.document_held_id(right)
    );
    assert_eq!(
        package.resolve_node_held(left, &raw[left].node_id).as_ref(),
        package.structural_key_at(left)
    );
    assert_eq!(
        package
            .resolve_node_held(right, &raw[right].node_id)
            .as_ref(),
        package.structural_key_at(right)
    );
    let (_, positions) = recorder.generation_input();
    assert_eq!(
        positions
            .iter()
            .filter(|position| matches!(position, ContractRequestPosition::DriverItem { .. }))
            .count(),
        3,
        "all three operations reach generation"
    );
}
