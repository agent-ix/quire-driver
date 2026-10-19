// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's command-level refusals (ADR-011 §2.3, §5; ADR-012 §7.2).
//! AGE-394 (the lifecycle and CLI design) renders them; this module only types
//! them.

use qsl_foundation::diagnostic::InternalFault;
use qsl_foundation::diagnostic::StageFailure;
use qsl_foundation::digest::WireNodeId;
use qsl_foundation::{BackendId, Code, EmptyBackendIdentity};
use qsl_replay::spine::{BindRefusal, CompileRefusal, RunRefusal, SpineStage};
use qsl_replay::Category;
use qsl_route::RegistrationRefusal;
use quire_contract_codegen::{
    ContractScalarError, EnvelopeRefusal, KaniObligationError, KaniScalarNativeError,
    KaniScalarNativeErrorClass, OracleGenerationError, RoutedGenerationError,
};
use quire_contract_model::CheckedPackageLimit;
use quire_contract_model::{CheckedPackageIncomplete, CheckedPackageRefusal};
use quire_exact::CancelCause;

/// Why [`crate::drive`] or [`crate::build_registry`] produced no result.
#[derive(Debug, thiserror::Error)]
pub enum DriverRefusal {
    /// A provider manifest supplied QSL's typed empty-identity refusal.
    #[error(transparent)]
    EmptyBackendIdentity(EmptyBackendIdentity),
    /// A linked provider manifest cannot produce an FR-331 descriptor because
    /// it advertises no capabilities. No QSL cause exists for this schema
    /// shape, so the refusal carries only the base catalog code.
    #[error("linked FR-331 manifest must advertise at least one capability")]
    EmptyLinkedManifestAdvertisements,
    /// A provider manifest did not register. Every refusal, ordered by
    /// backend identity then manifest digest.
    #[error("{} backend registration(s) refused", .0.len())]
    Registration(Vec<RegistrationRefusal>),
    /// S1 to S4, or the E4 emitter, refused the unit.
    #[error("{0}")]
    Compile(Box<CompileRefusal>),
    /// A QSL scalar call used for occurrence binding refused unchanged.
    #[error(transparent)]
    ScalarCall(Box<RunRefusal>),
    /// QSL's exact evidence builder refused while preparing IR's one reader call.
    #[error(transparent)]
    QslAdmissionEvidence(Box<RunRefusal>),
    /// QSL refused to bind a call to the selected scalar occurrence.
    #[error(transparent)]
    ScalarBinding(BindRefusal),
    /// CG's paired proof/native runner refused or faulted with its stable
    /// classification, retained without conversion.
    #[error(transparent)]
    ScalarNative(KaniScalarNativeError),
    /// QSL stopped a lifecycle stage because the caller cancelled its handle.
    #[error("QSL {stage:?} lifecycle cancelled: {cause:?}")]
    Cancelled {
        /// The QSL lifecycle stage that stopped.
        stage: SpineStage,
        /// The caller's cancellation cause, unchanged.
        cause: CancelCause,
    },
    /// MD-2: a registered `BackendId` has no CG backend kind. Every
    /// unconverted id, in bytewise order. Raised before any negotiation.
    #[error("invalid_capability/unknown-backend: no CG backend kind for {}", join(.backends))]
    UnknownBackend {
        /// Each unconverted `BackendId`, in bytewise order.
        backends: Vec<BackendId>,
    },
    /// CG refused the whole FR-331 envelope before settling any item.
    #[error("{}", envelope_message(.0))]
    Envelope(EnvelopeRefusal),
    /// E5: IR's reader stopped at one of its limits.
    #[error("IR's checked-package reader reached its {:?} limit", .0.limit_kind)]
    HandoffLimit(CheckedPackageIncomplete),
    /// E7's exact IR contract-package lowering reached its retained bound.
    #[error("IR contract lowering reached its {limit_kind:?} limit ({consumed} over {limit})")]
    LoweringLimit {
        /// The limit that stopped package lowering.
        limit_kind: CheckedPackageLimit,
        /// The configured ceiling.
        limit: u64,
        /// The first denied work count or encoded byte count.
        consumed: u64,
    },
    /// CG generation refused the routed items as a whole.
    #[error("CG generation refused the routed items: {}", generation_message(.0))]
    Generation(RoutedGenerationError),
    /// A linked provider's admitted manifest could not be encoded canonically.
    #[error("linked provider manifest could not be encoded: {0}")]
    ManifestEncoding(quire_canonical::Error),
    /// An upstream answer broke an invariant the driver relies on. Not a
    /// refusal of the input.
    #[error("internal fault: {0}")]
    Fault(DriverFault),
}

/// A violated invariant between the driver and an upstream crate.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DriverFault {
    /// QSL's typed lifecycle reported a broken invariant.
    #[error("QSL {stage:?} spine fault: {fault:?}")]
    Spine {
        /// The typed lifecycle operation where the failure surfaced.
        stage: SpineStage,
        /// QSL's stable fault details.
        fault: InternalFault,
    },
    /// Negotiation returned a different number of settlements, or one out
    /// of request order.
    #[error(
        "negotiation settled {settled} item(s) out of order or count for {requested} requested"
    )]
    SettlementCount {
        /// Items requested.
        requested: usize,
        /// Settlements returned.
        settled: usize,
    },
    /// CG found a repeated manifest identity that the driver's registry
    /// construction should have prevented.
    #[error("CG negotiation manifest repeats backend identity `{identity}`")]
    ManifestRepeatsBackendIdentity {
        /// The bytewise-first repeated identity from CG's typed refusal.
        identity: String,
    },
    /// An item settled `supported` by a backend outside its candidate set.
    #[error(
        "request item {request_index} settled supported by `{backend}`, not one of its candidates"
    )]
    SupportedOutsideCandidates {
        /// The item's request index.
        request_index: usize,
        /// The backend negotiation named.
        backend: String,
    },
    /// A capability label QSL spells has no member in CG's vocabulary.
    #[error("CG's capability vocabulary has no `{label}`")]
    VocabularyMismatch {
        /// The QSL wire label.
        label: String,
    },
    /// Routing returned a different number of targets than it was given
    /// dispositions, or no target for a `supported` item.
    #[error(
        "routing returned no usable target for a supported item, or routed an item outside the request"
    )]
    RoutingTarget,
    /// IR's reader refused the bytes QSL's E4 emitter just wrote.
    #[error("IR refused the emitted checked package: {:?}", .0.code)]
    HandoffRefused(Box<CheckedPackageRefusal>),
    /// IR admitted the bytes at a `package_id` other than the emitted one.
    #[error("IR admitted package {admitted}, not the emitted {emitted}")]
    PackageIdMismatch {
        /// The emitted `package_id` digest.
        emitted: String,
        /// The admitted `package_id` digest.
        admitted: String,
    },
    /// QSL admitted a linked manifest whose advertised pair was incomplete.
    #[error("QSL admitted a linked manifest with an incomplete advertised pair")]
    AdmittedManifestMissingLabel,
    /// A registered descriptor has no retained FR-331 manifest source.
    #[error("registered descriptor has no retained manifest source")]
    AdmittedManifestMissing,
    /// A routed item names a node the checked package does not hold.
    /// Every item's node comes from `CheckedPackage::graph().requirements()`,
    /// and `checked` is IR's own admission of the bytes that same package
    /// emitted, at the same `package_id` -- so this can only happen if E4
    /// emitted, or E5 admitted, a graph missing a node QSL's own requirement
    /// record names. Not a caller input: no request the driver builds from
    /// `items_from_requirements` can name a node outside its own package.
    #[error(
        "request item {request_index} names node {node}, which the checked package does not hold"
    )]
    UnknownNode {
        /// The item's request index.
        request_index: usize,
        /// The node the item names.
        node: WireNodeId,
    },
    /// A wire node digest maps to multiple typed identities in the admitted
    /// package, so the driver cannot choose one without guessing.
    #[error(
        "request item {request_index} names node {node}, which maps to multiple checked node identities"
    )]
    AmbiguousNode {
        /// The item's request index.
        request_index: usize,
        /// The node whose digest is ambiguous.
        node: WireNodeId,
    },
    /// A QSL occurrence role is outside IR's closed occurrence vocabulary.
    #[error("QSL occurrence role `{role}` has no IR occurrence role")]
    UnknownOccurrenceRole {
        /// The exact QSL role spelling.
        role: String,
    },
    /// IR lowering did not return a package despite having no typed limit result.
    #[error("IR lowering returned no contract package and no limit record")]
    ContractLoweringIncomplete,
    /// CG's receipt does not consistently attest the request's original
    /// scalar limits.
    #[error("CG's native receipt does not attest the retained ScalarLimits")]
    NativeExecutionLimitsMismatch,
    /// A receipt failed the driver's artifact identity or retained-byte check.
    #[error("CG's native receipt failed the retained artifact identity check")]
    NativeReceiptMismatch,
}

impl From<EnvelopeRefusal> for DriverRefusal {
    fn from(refusal: EnvelopeRefusal) -> Self {
        match refusal {
            EnvelopeRefusal::InvalidCapability { .. } => Self::Envelope(refusal),
            EnvelopeRefusal::ManifestRepeatsBackendIdentity { identity } => {
                Self::Fault(DriverFault::ManifestRepeatsBackendIdentity { identity })
            }
        }
    }
}

/// The step a [`DriverRefusal`] comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverStage {
    /// Building the registry from provider manifests.
    Registry,
    /// A spine stage, S1 to the E4 emitter.
    Compile(SpineStage),
    /// The pre-negotiation `BackendId` conversion.
    Conversion,
    /// E7 negotiation.
    Negotiation,
    /// The routing step after negotiation.
    Routing,
    /// E5: IR's read of the emitted package.
    Handoff,
    /// CG generation for the routed items.
    Generation,
    /// Native execution or receipt verification for a routed scalar.
    Replay,
}

/// The terminal outcome kind a refusal maps to (ADR-011 §2.3, §5): input
/// refusal, limit, cancellation incompletion and internal fault remain
/// distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    /// The input was refused.
    Refused,
    /// A stage reached one of its limits.
    Limit,
    /// A caller cancellation stopped a lifecycle operation. Use
    /// [`DriverRefusal::qsl_category`] for QSL's broader incomplete category.
    Cancelled,
    /// An internal invariant was violated.
    Fault,
}

impl DriverRefusal {
    /// A QSL-owned category carried without translation by a typed refusal.
    pub fn qsl_category(&self) -> Option<Category> {
        match self {
            Self::EmptyBackendIdentity(refusal) => Some(refusal.category()),
            Self::EmptyLinkedManifestAdvertisements => Some(Category::Refusal),
            Self::ScalarCall(refusal) => Some(refusal.category()),
            Self::QslAdmissionEvidence(refusal) => Some(refusal.category()),
            Self::ScalarBinding(refusal) => Some(refusal.code().category()),
            Self::Cancelled { cause, .. } => {
                // `StageFailure::Cancelled` has a category independent of
                // its refusal payload. `LimitExceeded` satisfies the generic
                // catalog bound without inventing a driver cancellation map.
                let failure: StageFailure<qsl_foundation::diagnostic::LimitExceeded> =
                    StageFailure::Cancelled(*cause);
                Some(failure.category())
            }
            _ => None,
        }
    }

    /// The catalog code, rendered `code` or `code/cause`.
    ///
    /// For [`DriverRefusal::Registration`] this is the first refusal's code
    /// in (identity, manifest digest) order; each refusal in the vector
    /// carries its own `catalog_code()`.
    pub fn code(&self) -> String {
        match self {
            Self::EmptyBackendIdentity(refusal) => refusal.code().as_str().to_owned(),
            Self::EmptyLinkedManifestAdvertisements => "invalid_capability".to_owned(),
            Self::Cancelled { .. } => Code::Cancelled.as_str().to_owned(),
            Self::Registration(refusals) => refusals.first().map_or_else(
                || "invalid_capability".to_owned(),
                |refusal| refusal.catalog_code().to_string(),
            ),
            Self::Compile(refusal) => compile_code(refusal),
            Self::ScalarCall(refusal) => refusal.code().as_str().to_owned(),
            Self::QslAdmissionEvidence(refusal) => refusal.code().as_str().to_owned(),
            Self::ScalarBinding(refusal) => {
                let code = refusal.code().as_str();
                match refusal.cause() {
                    Some(cause) => format!("{code}/{cause}"),
                    None => code.to_owned(),
                }
            }
            Self::ScalarNative(error) => error.code().to_owned(),
            Self::UnknownBackend { .. } => "invalid_capability/unknown-backend".to_owned(),
            Self::Envelope(EnvelopeRefusal::InvalidCapability { cause, .. }) => {
                format!("invalid_capability/{cause}")
            }
            Self::Envelope(EnvelopeRefusal::ManifestRepeatsBackendIdentity { .. }) => {
                Code::RuntimeInvariant.as_str().to_owned()
            }
            Self::HandoffLimit(_) => Code::StageLimitExceeded.as_str().to_owned(),
            Self::LoweringLimit { .. } => Code::ResourceExhausted.as_str().to_owned(),
            Self::Generation(error) => generation_code(error).as_str().to_owned(),
            Self::ManifestEncoding(error) if manifest_encoding_exhausted(error) => {
                Code::ResourceExhausted.as_str().to_owned()
            }
            Self::ManifestEncoding(_) => Code::RuntimeInvariant.as_str().to_owned(),
            Self::Fault(_) => Code::RuntimeInvariant.as_str().to_owned(),
        }
    }

    /// The step that refused.
    pub fn stage(&self) -> DriverStage {
        match self {
            Self::EmptyBackendIdentity(_) => DriverStage::Registry,
            Self::EmptyLinkedManifestAdvertisements => DriverStage::Registry,
            Self::Registration(_) => DriverStage::Registry,
            Self::ManifestEncoding(_)
            | Self::Fault(
                DriverFault::AdmittedManifestMissingLabel | DriverFault::AdmittedManifestMissing,
            ) => DriverStage::Registry,
            Self::Compile(refusal) => DriverStage::Compile(refusal.stage()),
            Self::QslAdmissionEvidence(_) => DriverStage::Handoff,
            Self::ScalarCall(_) | Self::ScalarBinding(_) | Self::ScalarNative(_) => {
                DriverStage::Replay
            }
            Self::Cancelled { stage, .. } => DriverStage::Compile(*stage),
            Self::UnknownBackend { .. } | Self::Fault(DriverFault::VocabularyMismatch { .. }) => {
                DriverStage::Conversion
            }
            Self::Envelope(_) => DriverStage::Negotiation,
            Self::Fault(DriverFault::RoutingTarget) => DriverStage::Routing,
            Self::Fault(
                DriverFault::NativeExecutionLimitsMismatch | DriverFault::NativeReceiptMismatch,
            ) => DriverStage::Replay,
            Self::Fault(DriverFault::Spine { stage, .. }) => DriverStage::Compile(*stage),
            Self::HandoffLimit(_)
            | Self::Fault(DriverFault::HandoffRefused(_) | DriverFault::PackageIdMismatch { .. }) => {
                DriverStage::Handoff
            }
            Self::Generation(_)
            | Self::LoweringLimit { .. }
            | Self::Fault(DriverFault::UnknownNode { .. } | DriverFault::AmbiguousNode { .. }) => {
                DriverStage::Generation
            }
            Self::Fault(
                DriverFault::UnknownOccurrenceRole { .. } | DriverFault::ContractLoweringIncomplete,
            ) => DriverStage::Generation,
            Self::Fault(
                DriverFault::ManifestRepeatsBackendIdentity { .. }
                | DriverFault::SettlementCount { .. }
                | DriverFault::SupportedOutsideCandidates { .. },
            ) => DriverStage::Negotiation,
        }
    }

    /// The outcome kind this refusal settles.
    pub fn kind(&self) -> OutcomeKind {
        match self {
            Self::EmptyBackendIdentity(_) => OutcomeKind::Refused,
            Self::EmptyLinkedManifestAdvertisements => OutcomeKind::Refused,
            Self::Cancelled { .. } => OutcomeKind::Cancelled,
            Self::ScalarCall(refusal) => outcome_kind_for_qsl(refusal.category()),
            Self::QslAdmissionEvidence(refusal) => outcome_kind_for_qsl(refusal.category()),
            Self::ScalarBinding(refusal) => outcome_kind_for_qsl(refusal.code().category()),
            Self::ScalarNative(error) => match error.class() {
                KaniScalarNativeErrorClass::CompletionRefusal => OutcomeKind::Refused,
                KaniScalarNativeErrorClass::EnvironmentFault => OutcomeKind::Fault,
            },
            Self::Compile(refusal) if refusal.code() == Code::StageLimitExceeded => {
                OutcomeKind::Limit
            }
            Self::HandoffLimit(_) => OutcomeKind::Limit,
            Self::LoweringLimit { .. } => OutcomeKind::Limit,
            Self::Generation(error) => match generation_code(error) {
                Code::RuntimeInvariant => OutcomeKind::Fault,
                Code::ResourceExhausted => OutcomeKind::Limit,
                _ => OutcomeKind::Refused,
            },
            Self::ManifestEncoding(error) if manifest_encoding_exhausted(error) => {
                OutcomeKind::Limit
            }
            Self::ManifestEncoding(_) => OutcomeKind::Fault,
            Self::Envelope(EnvelopeRefusal::ManifestRepeatsBackendIdentity { .. }) => {
                OutcomeKind::Fault
            }
            Self::Registration(_)
            | Self::Compile(_)
            | Self::UnknownBackend { .. }
            | Self::Envelope(EnvelopeRefusal::InvalidCapability { .. }) => OutcomeKind::Refused,
            Self::Fault(_) => OutcomeKind::Fault,
        }
    }
}

fn outcome_kind_for_qsl(category: Category) -> OutcomeKind {
    match category {
        Category::Refusal | Category::Unsupported => OutcomeKind::Refused,
        Category::Incomplete => OutcomeKind::Cancelled,
        Category::InternalFailure => OutcomeKind::Fault,
        _ => OutcomeKind::Fault,
    }
}

fn manifest_encoding_exhausted(error: &quire_canonical::Error) -> bool {
    matches!(
        error,
        quire_canonical::Error::Limit(_) | quire_canonical::Error::Allocation { .. }
    )
}

/// The catalog code of a generation refusal. The driver routes each item
/// with the kind CG's own conversion gave it, once per request index, so a
/// kind disagreement, a duplicate index, a duplicate harness, a Kani record-count
/// mismatch, a Kani duplicate position out of range and an empty Kani request are
/// internal faults. A missing kind context or a malformed Kani context is
/// the caller's request; the Kani item ceiling and an oversized oracle source
/// are limits; a claim-map or location-map serialization failure and an
/// unknown Contract Runtime variant are internal faults (IR-294's Kani arm
/// derives the claim map itself; nothing the driver supplies can cause
/// them).
fn generation_code(error: &RoutedGenerationError) -> Code {
    match error {
        RoutedGenerationError::ScalarSelectionMismatch { .. }
        | RoutedGenerationError::Package(_) => Code::InvalidRequest,
        RoutedGenerationError::BackendKindDisagrees { .. }
        | RoutedGenerationError::DuplicateRequestIndex { .. }
        | RoutedGenerationError::DuplicateHarness { .. }
        | RoutedGenerationError::KaniRecordCountMismatch { .. }
        | RoutedGenerationError::KaniDuplicatePositionOutOfRange { .. }
        | RoutedGenerationError::Kani(KaniObligationError::EmptyRequest) => Code::RuntimeInvariant,
        RoutedGenerationError::Kani(KaniObligationError::TooManyItems { .. }) => {
            Code::ResourceExhausted
        }
        RoutedGenerationError::ContractScalar(ContractScalarError::Request(_)) => {
            Code::InvalidRequest
        }
        RoutedGenerationError::ContractScalar(ContractScalarError::Oracle(oracle)) => {
            oracle_code(oracle)
        }
        RoutedGenerationError::MissingKindContext { .. }
        | RoutedGenerationError::Kani(
            KaniObligationError::InvalidSubjectPath | KaniObligationError::InvalidUnwind { .. },
        ) => Code::InvalidRequest,
    }
}

/// The catalog code of a whole-generation FR-014 oracle failure.
fn oracle_code(error: &OracleGenerationError) -> Code {
    match error {
        OracleGenerationError::SourceTooLarge { .. }
        | OracleGenerationError::ArtifactTooLarge { .. }
        | OracleGenerationError::ArtifactCountExceeded { .. }
        | OracleGenerationError::BundleTooLarge { .. } => Code::ResourceExhausted,
        OracleGenerationError::PackageRequest(_)
        | OracleGenerationError::ContractPackageUnavailable
        | OracleGenerationError::InvalidTypeResolutionWorkLimit => Code::InvalidRequest,
        OracleGenerationError::ClaimMapSerialization
        | OracleGenerationError::LocationMapSerialization
        | OracleGenerationError::InvalidPlanState
        | OracleGenerationError::UnknownRuntimeVariant { .. } => Code::RuntimeInvariant,
    }
}

/// `CompileRefusal`'s own `code()` is the bare catalog `Code`; its doc
/// promises `code/cause` (e.g. `missing_import/missing-selection`) wherever
/// the refusing stage names one. `code()` carries no cause on its own, so
/// this renders `code/cause` when [`compile_cause`] finds one, or the bare
/// code otherwise.
fn compile_code(refusal: &CompileRefusal) -> String {
    let code = refusal.code().as_str();
    match compile_cause(refusal) {
        Some(cause) => format!("{code}/{cause}"),
        None => code.to_owned(),
    }
}

/// The catalog cause a `CompileRefusal` names, where its refusing stage
/// records one: `Check`'s first refusal, `Profile`'s first refusal (FR-110),
/// `Limit`'s limit kind, `Link`, an ADR-015 D-1 `Import` or `DependencyInput`
/// refusal, or (through a `Dependency` wrapper) the library refusal it carries.
/// Exhaustive, not a catch-all: `Source`,
/// `Forms`, `Intake` (a `model` declaration's own domain-package intake,
/// not an ADR-015 D-1 import), `Assembly`, `Emit` and `Omitted` name no
/// cause in QSL's own catalog today.
fn compile_cause(refusal: &CompileRefusal) -> Option<&'static str> {
    match refusal {
        CompileRefusal::Source(_)
        | CompileRefusal::Forms { .. }
        | CompileRefusal::Intake { .. }
        | CompileRefusal::OperationClause { .. }
        | CompileRefusal::TypeInvariant { .. }
        | CompileRefusal::Assembly { .. }
        | CompileRefusal::Emit(_)
        | CompileRefusal::Omitted(_) => None,
        CompileRefusal::Limit(limit) => Some(limit.kind().catalog_cause()),
        CompileRefusal::Profile { refusals, .. } => refusals.first().map(|refusal| refusal.cause()),
        CompileRefusal::Check { refusals, .. } => {
            refusals.first().and_then(|refusal| refusal.cause.cause())
        }
        CompileRefusal::DependencyInput(refusal) => refusal.cause(),
        CompileRefusal::Import { refusal, .. } => refusal.cause(),
        CompileRefusal::Dependency { refusal, .. } => compile_cause(refusal),
        CompileRefusal::Link(refusal) => refusal.cause(),
    }
}

fn join(backends: &[BackendId]) -> String {
    backends
        .iter()
        .map(BackendId::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

fn generation_message(error: &RoutedGenerationError) -> String {
    match error {
        RoutedGenerationError::ScalarSelectionMismatch { request_index } => {
            format!("request item {request_index} did not match its admitted scalar selection")
        }
        RoutedGenerationError::Package(error) => {
            format!("the admitted package request was inconsistent: {error:?}")
        }
        RoutedGenerationError::ContractScalar(ContractScalarError::Request(error)) => {
            format!("the scalar package request was inconsistent: {error:?}")
        }
        RoutedGenerationError::ContractScalar(ContractScalarError::Oracle(error)) => {
            format!(
                "the derived claim map failed to generate: {}",
                oracle_message(error)
            )
        }
        RoutedGenerationError::BackendKindDisagrees {
            request_index,
            backend,
            routed,
            converted,
        } => format!(
            "request item {request_index} was routed to `{}` as {}, which converts to {}",
            backend.identity,
            routed.identity(),
            converted.as_ref().map_or("no kind", |kind| kind.identity())
        ),
        RoutedGenerationError::DuplicateRequestIndex { request_index } => {
            format!("request index {request_index} is routed twice")
        }
        RoutedGenerationError::DuplicateHarness { harness } => {
            format!("Kani harness `{harness}` is emitted more than once")
        }
        RoutedGenerationError::KaniRecordCountMismatch { records, items } => {
            format!("Kani returned {records} record(s) for {items} item(s)")
        }
        RoutedGenerationError::KaniDuplicatePositionOutOfRange { first_index, items } => {
            format!("Kani duplicate position {first_index} is outside {items} item(s)")
        }
        RoutedGenerationError::MissingKindContext { kind } => {
            format!("no generation context for backend kind {}", kind.identity())
        }
        RoutedGenerationError::Kani(error) => {
            format!("the Kani arm refused: {}", kani_message(error))
        }
    }
}

fn oracle_message(error: &OracleGenerationError) -> String {
    match error {
        OracleGenerationError::PackageRequest(error) => {
            format!("the admitted package request was inconsistent: {error:?}")
        }
        OracleGenerationError::ContractPackageUnavailable => {
            "the lowered record has no admitted Contract IR package".to_owned()
        }
        OracleGenerationError::SourceTooLarge {
            bytes,
            limit,
            setting,
        } => {
            format!("the generated source is {bytes} byte(s), over {setting} limit {limit}")
        }
        OracleGenerationError::InvalidTypeResolutionWorkLimit => {
            "the configured type-resolution work limit cannot represent its first denied count"
                .to_owned()
        }
        OracleGenerationError::ArtifactTooLarge { bytes, limit } => {
            format!("generated artifact size {bytes} exceeds publication limit {limit}")
        }
        OracleGenerationError::ArtifactCountExceeded { count, limit } => {
            format!("generated artifact count {count} exceeds publication limit {limit}")
        }
        OracleGenerationError::BundleTooLarge { bytes, limit } => {
            format!("generated bundle size {bytes} exceeds publication limit {limit}")
        }
        OracleGenerationError::ClaimMapSerialization => {
            "the claim map could not be serialized".to_owned()
        }
        OracleGenerationError::LocationMapSerialization => {
            "the location map could not be serialized".to_owned()
        }
        OracleGenerationError::UnknownRuntimeVariant { enum_name } => {
            format!("Contract Runtime's `{enum_name}` yielded an unknown variant")
        }
        OracleGenerationError::InvalidPlanState => {
            "the validated oracle plan violated a rendering invariant".to_owned()
        }
    }
}

fn kani_message(error: &KaniObligationError) -> String {
    match error {
        KaniObligationError::EmptyRequest => "the request names no items".to_owned(),
        KaniObligationError::TooManyItems { count } => format!("{count} items exceed the ceiling"),
        KaniObligationError::InvalidSubjectPath => "the subject path is not a Rust path".to_owned(),
        KaniObligationError::InvalidUnwind { unwind } => {
            format!("unwind bound {unwind} is out of range")
        }
    }
}

fn envelope_message(refusal: &EnvelopeRefusal) -> String {
    match refusal {
        EnvelopeRefusal::InvalidCapability {
            cause,
            member,
            received,
        } => format!(
            "CG refused the negotiation envelope: invalid_capability/{cause}: `{member}` is `{received}`"
        ),
        EnvelopeRefusal::ManifestRepeatsBackendIdentity { identity } => format!(
            "internal fault: CG negotiation manifest repeats backend identity `{identity}`"
        ),
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use qsl_foundation::digest::WireNodeId;
    use qsl_foundation::EmptyBackendIdentity;
    use qsl_replay::Category;
    use quire_contract_codegen::{
        ContractScalarError, EnvelopeRefusal, HarnessPath, HarnessSymbol, KaniObligationError,
        KaniScalarNativeError, KaniScalarNativeErrorClass, ModuleSymbol, OracleGenerationError,
        RoutedGenerationError, ScalarOracleStopKind,
    };
    use quire_contract_model::{CheckedPackageIncomplete, CheckedPackageLimit};

    use super::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};

    #[trace("FR-009-AC-6", "FR-009-AC-7")]
    #[test]
    fn native_runner_typed_refusal_keeps_cg_discriminant_and_class() {
        let refusal = DriverRefusal::ScalarNative(KaniScalarNativeError::OracleStop(
            ScalarOracleStopKind::IllTyped,
        ));
        assert_eq!(refusal.code(), "oracle_stop");
        assert_eq!(refusal.qsl_category(), Some(Category::Refusal));
        assert_eq!(refusal.stage(), DriverStage::Replay);
        assert_eq!(refusal.kind(), OutcomeKind::Refused);
        assert_eq!(
            match &refusal {
                DriverRefusal::ScalarNative(error) => error.class(),
                _ => unreachable!("test constructed the scalar-native refusal"),
            },
            KaniScalarNativeErrorClass::CompletionRefusal
        );

        let fault = DriverRefusal::ScalarNative(KaniScalarNativeError::CodegenLock);
        assert_eq!(fault.code(), "codegen_lock");
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.stage(), DriverStage::Replay);
    }

    #[test]
    fn empty_backend_identity_preserves_qsl_code_and_category() {
        let refusal = DriverRefusal::EmptyBackendIdentity(EmptyBackendIdentity);
        assert_eq!(
            refusal.code(),
            qsl_foundation::Code::InvalidIdentifier.as_str()
        );
        assert_eq!(refusal.qsl_category(), Some(Category::Refusal));
        assert_eq!(refusal.kind(), OutcomeKind::Refused);
        assert_eq!(refusal.stage(), DriverStage::Registry);
    }

    #[trace("FR-001-AC-5")]
    #[test]
    fn repeated_manifest_backend_identity_is_a_negotiation_fault() {
        let refusal = DriverRefusal::from(EnvelopeRefusal::ManifestRepeatsBackendIdentity {
            identity: "plugin:example".to_owned(),
        });

        assert_eq!(
            refusal.code(),
            qsl_foundation::Code::RuntimeInvariant.as_str()
        );
        assert_eq!(refusal.stage(), DriverStage::Negotiation);
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(
            refusal.to_string(),
            "internal fault: CG negotiation manifest repeats backend identity `plugin:example`"
        );
        assert!(!refusal.to_string().contains("invalid_capability"));
    }

    /// Trace: FR-001-AC-5
    #[test]
    fn routed_generation_invariant_errors_are_generation_faults() {
        for (error, message) in [
            (
                RoutedGenerationError::DuplicateHarness {
                    harness: HarnessPath {
                        module: ModuleSymbol::try_from("subject").expect("valid module"),
                        harness: HarnessSymbol::try_from("proof").expect("valid harness"),
                    },
                },
                "CG generation refused the routed items: Kani harness `subject::proof` is emitted more than once",
            ),
            (
                RoutedGenerationError::KaniRecordCountMismatch {
                    records: 1,
                    items: 2,
                },
                "CG generation refused the routed items: Kani returned 1 record(s) for 2 item(s)",
            ),
            (
                RoutedGenerationError::KaniDuplicatePositionOutOfRange {
                    first_index: 3,
                    items: 2,
                },
                "CG generation refused the routed items: Kani duplicate position 3 is outside 2 item(s)",
            ),
        ] {
            let refusal = DriverRefusal::Generation(error);
            assert_eq!(refusal.code(), "runtime_invariant");
            assert_eq!(refusal.kind(), OutcomeKind::Fault);
            assert_eq!(refusal.stage(), DriverStage::Generation);
            assert_eq!(refusal.to_string(), message);
        }
    }

    /// FR-001-AC-5 (TC-004): an internal fault is its own outcome kind, with
    /// code `runtime_invariant`, never a refusal.
    #[test]
    fn tc_004_a_fault_is_an_internal_fault_with_runtime_invariant() {
        let fault = DriverRefusal::Fault(DriverFault::SupportedOutsideCandidates {
            request_index: 0,
            backend: "kani".to_owned(),
        });
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Negotiation);

        let fault = DriverRefusal::Fault(DriverFault::VocabularyMismatch {
            label: "new-kind".to_owned(),
        });
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Conversion);

        let fault = DriverRefusal::Fault(DriverFault::PackageIdMismatch {
            emitted: "a".to_owned(),
            admitted: "b".to_owned(),
        });
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Handoff);

        let fault = DriverRefusal::Generation(RoutedGenerationError::DuplicateRequestIndex {
            request_index: 0,
        });
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        // IR-294: a whole-generation FR-014 oracle failure the driver never
        // produces is still an internal fault, not a refusal: an unknown
        // Contract Runtime variant and a claim-map serialization failure.
        let fault = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(
            ContractScalarError::Oracle(OracleGenerationError::UnknownRuntimeVariant {
                enum_name: "IntegerDomain",
            }),
        ));
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        let fault = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(
            ContractScalarError::Oracle(OracleGenerationError::ClaimMapSerialization),
        ));
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        // A routed item naming a node the checked package does not hold is
        // an E4/E5 inconsistency, not a caller input: every item's node
        // comes from the same package's own `requirements()`, so only a
        // broken invariant between emission and admission reaches this.
        let fault = DriverRefusal::Fault(DriverFault::UnknownNode {
            request_index: 1,
            node: WireNodeId::from_digest([9; 32]),
        });
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);
    }

    /// FR-001-AC-5 (TC-004): E5's reader limit and CG's Kani item ceiling
    /// are limit refusals.
    #[test]
    fn tc_004_handoff_and_generation_limits_are_limits() {
        let limit = DriverRefusal::HandoffLimit(CheckedPackageIncomplete {
            limit_kind: CheckedPackageLimit::Bytes,
            limit: 1,
            consumed: 2,
            path: None,
        });
        assert_eq!(limit.kind(), OutcomeKind::Limit);
        assert_eq!(limit.code(), "stage_limit_exceeded");
        assert_eq!(limit.stage(), DriverStage::Handoff);

        let limit = DriverRefusal::Generation(RoutedGenerationError::Kani(
            KaniObligationError::TooManyItems { count: 9 },
        ));
        assert_eq!(limit.kind(), OutcomeKind::Limit);
        assert_eq!(limit.code(), "resource_exhausted");
        assert_eq!(limit.stage(), DriverStage::Generation);

        // IR-294: an oversized generated oracle source is a limit, the same
        // as the Kani item ceiling.
        let limit = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(
            ContractScalarError::Oracle(OracleGenerationError::SourceTooLarge {
                bytes: 2,
                limit: 1,
                setting: "cg.scalar.source_bytes",
            }),
        ));
        assert_eq!(limit.kind(), OutcomeKind::Limit);
        assert_eq!(limit.code(), "resource_exhausted");
        assert_eq!(limit.stage(), DriverStage::Generation);
    }
}
