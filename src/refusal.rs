// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's command-level refusals (ADR-011 §2.3, §5; ADR-012 §7.2).
//! AGE-394 (the lifecycle and CLI design) renders them; this module only types
//! them.

use qsl_foundation::diagnostic::StageFailure;
use qsl_foundation::source::provenance::OccurrenceKey;
use qsl_foundation::{BackendId, Code, EmptyBackendIdentity};
use qsl_package::ExportRefusal;
use qsl_replay::spine::{BindRefusal, CompileRefusal, RunRefusal, SpineStage};
use qsl_route::RegistrationRefusal;
use quire_contract_codegen::{
    ContractNegotiationError, ContractScalarError, EnvelopeRefusal, KaniObligationError,
    KaniScalarNativeError, KaniScalarNativeErrorClass, OracleGenerationError, PackageRequestError,
    RoutedGenerationError,
};
use quire_contract_model::{
    CheckedNodeId, CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageRefusal,
    CheckedPackageRefusalCause, CheckedPackageRefusalCode,
};
use quire_exact::CancelCause;
use quire_exact::NodeKey;
use quire_semantic_value::catalog::Category;
use quire_semantic_value::catalog::InternalFault;

/// Why [`crate::drive`] or [`crate::build_registry`] produced no result.
#[derive(Debug, thiserror::Error)]
pub enum DriverRefusal {
    /// The QSL binding export refused the run as a whole.
    #[error("QSL binding export refused the run: {0:?}")]
    BindingsExport(ExportRefusal),
    /// A provider manifest supplied QSL's typed empty-identity refusal.
    #[error(transparent)]
    EmptyBackendIdentity(EmptyBackendIdentity),
    /// A linked provider manifest cannot produce an FR-331 descriptor because
    /// it advertises no capabilities. No QSL cause exists for this schema
    /// shape, so the refusal carries only the base catalog code.
    #[error("linked FR-331 manifest must advertise at least one capability")]
    EmptyLinkedManifestAdvertisements,
    /// A provider manifest did not register. Every refusal, ordered by
    /// backend identity.
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
    Envelope(Box<EnvelopeRefusal>),
    /// CG's ContractPackage reader stopped; its source projection and the
    /// constructed FR-272 diagnostic are retained together.
    #[error("{}", envelope_message(.0.source()))]
    PackageRead(Box<PackageReadFailure>),
    /// CG refused the admitted package and complete positions built by the
    /// driver from that same package. This is an internal join fault at
    /// negotiation, not a caller request refusal.
    #[error("CG package negotiation refused: {0:?}")]
    PackageNegotiation(PackageRequestError),
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
    /// QSL's export did not return one result for each driver-built key.
    #[error("binding export returned {returned} results for {requested} items")]
    ExportCount {
        /// Number of driver-built keys submitted.
        requested: usize,
        /// Number of per-item export results returned.
        returned: usize,
    },
    /// QSL's export said a driver-built key names no requirement record.
    #[error("binding export did not recognize driver item {item:?}")]
    ExportUnknownItem {
        /// The exact occurrence the export could not match.
        item: OccurrenceKey,
    },
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
    /// FR-330's ContractPackage reader cannot stop at Diagnostics.
    #[error("CG ContractPackage reader reported the unreachable Diagnostics limit kind")]
    PackageReadDiagnostics(quire_contract_codegen::ContractPackageIncompleteProjection),
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
    /// QSL admitted a linked manifest whose advertised pair was incomplete.
    #[error("QSL admitted a linked manifest with an incomplete advertised pair")]
    AdmittedManifestMissingLabel,
    /// A registered descriptor has no retained FR-331 manifest source.
    #[error("registered descriptor has no retained manifest source")]
    AdmittedManifestMissing,
    /// A routed item names a node the checked package does not hold.
    /// Every item's node comes from `CheckedPackage::graph().requirements()`,
    /// and `checked` is IR's own admission of the bytes that same package
    /// emitted -- so this can only happen if E4
    /// emitted, or E5 admitted, a graph missing a node QSL's own requirement
    /// record names. Not a caller input: no request the driver builds from
    /// `items_from_requirements` can name a node outside its own package.
    #[error(
        "request item {request_index} names node {node:?}, which the checked package does not hold"
    )]
    UnknownNode {
        /// The item's request index.
        request_index: usize,
        /// The node the item names.
        node: NodeKey,
    },
    /// A node key maps to multiple typed identities in the admitted
    /// package, so the driver cannot choose one without guessing.
    #[error(
        "request item {request_index} names node {node:?}, which maps to multiple checked node identities"
    )]
    AmbiguousNode {
        /// The item's request index.
        request_index: usize,
        /// The node whose identity is ambiguous.
        node: NodeKey,
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
        Self::from(Box::new(refusal))
    }
}

impl From<Box<EnvelopeRefusal>> for DriverRefusal {
    fn from(refusal: Box<EnvelopeRefusal>) -> Self {
        match refusal.as_ref() {
            EnvelopeRefusal::ContractPackageReadIncomplete { incomplete }
                if incomplete.limit_kind == CheckedPackageLimit::Diagnostics =>
            {
                Self::Fault(DriverFault::PackageReadDiagnostics(incomplete.clone()))
            }
            EnvelopeRefusal::ManifestRepeatsBackendIdentity { identity } => {
                Self::Fault(DriverFault::ManifestRepeatsBackendIdentity {
                    identity: identity.clone(),
                })
            }
            EnvelopeRefusal::ContractPackageReadRefused { refusal: source } => {
                let diagnostic = PackageReadDiagnostic {
                    code: package_read_refusal_code(source.code),
                    cause: source.cause.map(package_read_refusal_cause),
                    location: source
                        .loci
                        .first()
                        .and_then(|locus| locus.path.as_ref())
                        .map(|path| PackageReadLocation::JsonPath(path.as_str().to_owned())),
                    locus: source.locus.clone(),
                    limit: None,
                };
                Self::PackageRead(Box::new(PackageReadFailure {
                    source: *refusal,
                    diagnostic,
                }))
            }
            EnvelopeRefusal::ContractPackageReadIncomplete { incomplete } => {
                let Some((cause, setting)) = package_read_limit_mapping(incomplete.limit_kind)
                else {
                    return Self::Fault(DriverFault::PackageReadDiagnostics(incomplete.clone()));
                };
                let path = if incomplete.limit_kind == CheckedPackageLimit::Bytes {
                    None
                } else {
                    incomplete.path.clone()
                };
                let diagnostic = PackageReadDiagnostic {
                    code: Code::StageLimitExceeded.as_str(),
                    cause: Some(cause),
                    location: path.clone().map(PackageReadLocation::JsonPath),
                    locus: None,
                    limit: Some(PackageReadLimit {
                        cause,
                        setting,
                        bound: incomplete.limit,
                        actual: incomplete.consumed,
                        path,
                    }),
                };
                Self::PackageRead(Box::new(PackageReadFailure {
                    source: *refusal,
                    diagnostic,
                }))
            }
            _ => Self::Envelope(refusal),
        }
    }
}

impl DriverRefusal {
    fn envelope_refusal(&self) -> Option<&EnvelopeRefusal> {
        match self {
            Self::Envelope(refusal) => Some(refusal.as_ref()),
            Self::PackageRead(failure) => Some(failure.source()),
            _ => None,
        }
    }
}

impl From<ContractNegotiationError> for DriverRefusal {
    fn from(error: ContractNegotiationError) -> Self {
        match error {
            ContractNegotiationError::Envelope(refusal) => Self::from(refusal),
            ContractNegotiationError::Package(refusal) => Self::PackageNegotiation(refusal),
            ContractNegotiationError::EnvelopeItemCount {
                requested,
                supplied,
            } => Self::Fault(DriverFault::SettlementCount {
                requested,
                settled: supplied,
            }),
        }
    }
}

/// The step a [`DriverRefusal`] comes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriverStage {
    /// QSL's binding export, after the checked-package handoff.
    BindingsExport,
    /// Building the registry from provider manifests.
    Registry,
    /// A spine stage, S1 to the E4 emitter.
    Compile(SpineStage),
    /// The pre-negotiation `BackendId` conversion.
    Conversion,
    /// E7 negotiation.
    Negotiation,
    /// CG's FR-330 ContractPackage reader.
    PackageRead,
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

/// The FR-272 projection for a CG ContractPackage read limit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageReadLimit {
    /// The FR-272 cause tag.
    pub cause: &'static str,
    /// The `CheckedPackageReadLimits` member that supplied the bound.
    pub setting: &'static str,
    /// The caller-selected limit.
    pub bound: u64,
    /// The first denied count or encoded byte count.
    pub actual: u64,
    /// The CG reader's JSON path, absent for byte limits.
    pub path: Option<String>,
}

/// The FR-274 location of a ContractPackage reader diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PackageReadLocation {
    /// A JSON field or index path, unchanged from CG's reader projection.
    JsonPath(String),
}

/// The FR-272 diagnostic constructed when CG's ContractPackage reader stops.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageReadDiagnostic {
    code: &'static str,
    cause: Option<&'static str>,
    location: Option<PackageReadLocation>,
    locus: Option<CheckedNodeId>,
    limit: Option<PackageReadLimit>,
}

impl PackageReadDiagnostic {
    /// The FR-271 catalog code.
    pub fn code(&self) -> &'static str {
        self.code
    }
    /// The FR-272 cause tag, when the reader supplied one.
    pub fn cause(&self) -> Option<&'static str> {
        self.cause
    }
    /// The FR-272 category.
    pub fn category(&self) -> Category {
        if self.limit.is_some() {
            Category::Incomplete
        } else {
            Category::Refusal
        }
    }
    /// The driver's terminal outcome kind.
    pub fn kind(&self) -> OutcomeKind {
        if self.limit.is_some() {
            OutcomeKind::Limit
        } else {
            OutcomeKind::Refused
        }
    }
    /// The fixed driver stage of this diagnostic.
    pub fn stage(&self) -> DriverStage {
        DriverStage::PackageRead
    }
    /// The FR-274 location, absent when CG supplied no path.
    pub fn location(&self) -> Option<&PackageReadLocation> {
        self.location.as_ref()
    }
    /// The typed CG node locus, when the reader supplied one.
    pub fn locus(&self) -> Option<&CheckedNodeId> {
        self.locus.as_ref()
    }
    /// The mapped FR-272 limit fields, when this diagnostic is incomplete.
    pub fn limit(&self) -> Option<&PackageReadLimit> {
        self.limit.as_ref()
    }
}

/// A reader stop retains CG's original typed source and its emitted diagnostic.
#[derive(Debug)]
pub struct PackageReadFailure {
    source: EnvelopeRefusal,
    diagnostic: PackageReadDiagnostic,
}

impl PackageReadFailure {
    /// The unchanged CG reader refusal or incomplete projection.
    pub fn source(&self) -> &EnvelopeRefusal {
        &self.source
    }
    /// The FR-272 diagnostic made at the driver boundary.
    pub fn diagnostic(&self) -> &PackageReadDiagnostic {
        &self.diagnostic
    }
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
            Self::PackageRead(failure) => Some(failure.diagnostic().category()),
            Self::Envelope(refusal) => match refusal.as_ref() {
                EnvelopeRefusal::ContractPackageReadRefused { .. } => Some(Category::Refusal),
                EnvelopeRefusal::ContractPackageReadIncomplete { incomplete } => Some(
                    if incomplete.limit_kind == CheckedPackageLimit::Diagnostics {
                        Category::InternalFailure
                    } else {
                        Category::Incomplete
                    },
                ),
                _ => None,
            },
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
    /// in identity order; each refusal in the vector
    /// carries its own `catalog_code()`.
    pub fn code(&self) -> String {
        match self {
            Self::BindingsExport(refusal) => refusal.code().as_str().to_owned(),
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
            Self::PackageRead(failure) => failure.diagnostic().code().to_owned(),
            Self::Envelope(refusal) => match refusal.as_ref() {
                EnvelopeRefusal::InvalidCapability { cause, .. } => {
                    format!("invalid_capability/{cause}")
                }
                EnvelopeRefusal::ManifestRepeatsBackendIdentity { .. } => {
                    Code::RuntimeInvariant.as_str().to_owned()
                }
                EnvelopeRefusal::ContractPackageReadRefused { refusal } => {
                    package_read_refusal_code(refusal.code).to_owned()
                }
                EnvelopeRefusal::ContractPackageReadIncomplete { incomplete }
                    if incomplete.limit_kind == CheckedPackageLimit::Diagnostics =>
                {
                    Code::RuntimeInvariant.as_str().to_owned()
                }
                EnvelopeRefusal::ContractPackageReadIncomplete { .. } => {
                    Code::StageLimitExceeded.as_str().to_owned()
                }
            },
            Self::Fault(DriverFault::PackageReadDiagnostics(_)) => {
                Code::RuntimeInvariant.as_str().to_owned()
            }
            Self::PackageNegotiation(_) => Code::RuntimeInvariant.as_str().to_owned(),
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
            Self::BindingsExport(_) | Self::Fault(DriverFault::ExportCount { .. }) => {
                DriverStage::BindingsExport
            }
            Self::Fault(DriverFault::ExportUnknownItem { .. }) => DriverStage::Handoff,
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
            Self::PackageRead(_) => DriverStage::PackageRead,
            Self::Envelope(refusal) => match refusal.as_ref() {
                EnvelopeRefusal::InvalidCapability { .. }
                | EnvelopeRefusal::ManifestRepeatsBackendIdentity { .. } => {
                    DriverStage::Negotiation
                }
                EnvelopeRefusal::ContractPackageReadRefused { .. }
                | EnvelopeRefusal::ContractPackageReadIncomplete { .. } => DriverStage::PackageRead,
            },
            Self::Fault(DriverFault::PackageReadDiagnostics(_)) => DriverStage::PackageRead,
            Self::PackageNegotiation(_) => DriverStage::Negotiation,
            Self::Fault(DriverFault::RoutingTarget) => DriverStage::Routing,
            Self::Fault(
                DriverFault::NativeExecutionLimitsMismatch | DriverFault::NativeReceiptMismatch,
            ) => DriverStage::Replay,
            Self::Fault(DriverFault::Spine { stage, .. }) => DriverStage::Compile(*stage),
            Self::HandoffLimit(_) | Self::Fault(DriverFault::HandoffRefused(_)) => {
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
            Self::BindingsExport(ExportRefusal::Fault(_)) => OutcomeKind::Fault,
            Self::BindingsExport(_) => OutcomeKind::Refused,
            Self::EmptyBackendIdentity(_) => OutcomeKind::Refused,
            Self::EmptyLinkedManifestAdvertisements => OutcomeKind::Refused,
            Self::Cancelled { .. } => OutcomeKind::Cancelled,
            Self::ScalarCall(refusal) => outcome_kind_for_qsl(refusal.category()),
            Self::QslAdmissionEvidence(refusal) => outcome_kind_for_qsl(refusal.category()),
            Self::ScalarBinding(refusal) => outcome_kind_for_qsl(refusal.code().category()),
            Self::PackageRead(failure) => failure.diagnostic().kind(),
            Self::Envelope(refusal) => match refusal.as_ref() {
                EnvelopeRefusal::ContractPackageReadRefused { .. } => OutcomeKind::Refused,
                EnvelopeRefusal::ContractPackageReadIncomplete { incomplete }
                    if incomplete.limit_kind == CheckedPackageLimit::Diagnostics =>
                {
                    OutcomeKind::Fault
                }
                EnvelopeRefusal::ContractPackageReadIncomplete { .. } => OutcomeKind::Limit,
                EnvelopeRefusal::ManifestRepeatsBackendIdentity { .. } => OutcomeKind::Fault,
                EnvelopeRefusal::InvalidCapability { .. } => OutcomeKind::Refused,
            },
            Self::Fault(DriverFault::PackageReadDiagnostics(_)) | Self::PackageNegotiation(_) => {
                OutcomeKind::Fault
            }
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
            Self::Registration(_) | Self::Compile(_) | Self::UnknownBackend { .. } => {
                OutcomeKind::Refused
            }
            Self::Fault(_) => OutcomeKind::Fault,
        }
    }

    /// The exact FR-272 cause of a retained ContractPackage read refusal.
    pub fn package_read_cause(&self) -> Option<&'static str> {
        if let Self::PackageRead(failure) = self {
            return failure.diagnostic().cause();
        }
        match self.envelope_refusal()? {
            EnvelopeRefusal::ContractPackageReadRefused { refusal } => {
                refusal.cause.map(package_read_refusal_cause)
            }
            _ => None,
        }
    }

    /// The complete CG refusal projection retained by the driver.
    pub fn package_read_refusal_projection(
        &self,
    ) -> Option<&quire_contract_codegen::ContractPackageRefusalProjection> {
        match self.envelope_refusal()? {
            EnvelopeRefusal::ContractPackageReadRefused { refusal } => Some(refusal),
            _ => None,
        }
    }

    /// The FR-272 diagnostic constructed at the CG reader boundary.
    pub fn package_read_diagnostic(&self) -> Option<&PackageReadDiagnostic> {
        match self {
            Self::PackageRead(failure) => Some(failure.diagnostic()),
            _ => None,
        }
    }

    /// The FR-272 fields derived from a retained, permitted reader limit.
    /// The original CG projection remains available unchanged in `PackageRead`.
    pub fn package_read_limit(&self) -> Option<PackageReadLimit> {
        if let Self::PackageRead(failure) = self {
            return failure.diagnostic().limit().cloned();
        }
        let EnvelopeRefusal::ContractPackageReadIncomplete { incomplete } =
            self.envelope_refusal()?
        else {
            return None;
        };
        let (cause, setting) = package_read_limit_mapping(incomplete.limit_kind)?;
        Some(PackageReadLimit {
            cause,
            setting,
            bound: incomplete.limit,
            actual: incomplete.consumed,
            path: if incomplete.limit_kind == CheckedPackageLimit::Bytes {
                None
            } else {
                incomplete.path.clone()
            },
        })
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

fn package_read_refusal_code(code: CheckedPackageRefusalCode) -> &'static str {
    match code {
        CheckedPackageRefusalCode::UnknownContractVersion => "unknown_contract_version",
        CheckedPackageRefusalCode::MalformedWire => "malformed_wire",
        CheckedPackageRefusalCode::DuplicateMember => "duplicate_member",
        CheckedPackageRefusalCode::UnknownMember => "unknown_member",
        CheckedPackageRefusalCode::NoncanonicalWire => "noncanonical_wire",
        CheckedPackageRefusalCode::UnknownRequiredCapability => "unknown_required_capability",
        CheckedPackageRefusalCode::InvalidSemanticGraph => "invalid_semantic_graph",
        CheckedPackageRefusalCode::InvalidSourceMap => "invalid_source_map",
        CheckedPackageRefusalCode::UnsupportedNodeTag => "unsupported_node_tag",
        CheckedPackageRefusalCode::UnsupportedConstruct => "unsupported_construct",
        CheckedPackageRefusalCode::MissingDeclaration => "missing_declaration",
        CheckedPackageRefusalCode::InvalidModelBinding => "invalid_model_binding",
        CheckedPackageRefusalCode::InvalidTimedModel => "invalid_timed_model",
        CheckedPackageRefusalCode::InvalidPackage => "invalid_package",
        CheckedPackageRefusalCode::IllTyped => "ill_typed",
        CheckedPackageRefusalCode::AmbiguousDeclaration => "ambiguous_declaration",
        CheckedPackageRefusalCode::MissingImport => "missing_import",
        CheckedPackageRefusalCode::UnknownProfile => "unknown_profile",
        CheckedPackageRefusalCode::WrongSnapshot => "wrong_snapshot",
    }
}

fn package_read_refusal_cause(cause: CheckedPackageRefusalCause) -> &'static str {
    match cause {
        CheckedPackageRefusalCause::NonBooleanRoot => "non-boolean-root",
        CheckedPackageRefusalCause::MissingTimedModel => "missing-timed-model",
        CheckedPackageRefusalCause::TimedNodeReference => "timed-node-reference",
        CheckedPackageRefusalCause::DuplicateClockPlace => "duplicate-clock-place",
        CheckedPackageRefusalCause::MalformedClockPlace => "malformed-clock-place",
        CheckedPackageRefusalCause::InvalidTimeQuantity => "invalid-time-quantity",
        CheckedPackageRefusalCause::ClockWithoutDenseTime => "clock-without-dense-time",
        CheckedPackageRefusalCause::MalformedJson => "malformed-json",
        CheckedPackageRefusalCause::MissingMember => "missing-member",
        CheckedPackageRefusalCause::UnknownMember => "unknown-member",
        CheckedPackageRefusalCause::WrongValueKind => "wrong-value-kind",
        CheckedPackageRefusalCause::DefinitionCycle => "definition-cycle",
        CheckedPackageRefusalCause::IncompatibleDefinition => "incompatible-definition",
        CheckedPackageRefusalCause::FeatureSetMismatch => "feature-set-mismatch",
        CheckedPackageRefusalCause::InsufficientNextCharge => "insufficient-next-charge",
        CheckedPackageRefusalCause::MissingName => "missing-name",
        CheckedPackageRefusalCause::WrongObservation => "wrong-observation",
        CheckedPackageRefusalCause::DeclarationForm => "declaration-form",
        CheckedPackageRefusalCause::WrongExport => "wrong-export",
        CheckedPackageRefusalCause::InvalidDerivedShape => "invalid-derived-shape",
        CheckedPackageRefusalCause::MalformedDeclaration => "malformed-declaration",
        CheckedPackageRefusalCause::UnknownOperation => "unknown-operation",
        CheckedPackageRefusalCause::OperationClassMismatch => "operation-class-mismatch",
        CheckedPackageRefusalCause::OperationLawMissing => "operation-law-missing",
        CheckedPackageRefusalCause::OperationLawMismatch => "operation-law-mismatch",
        CheckedPackageRefusalCause::OperationLawUnselected => "operation-law-unselected",
        CheckedPackageRefusalCause::OperationModeMismatch => "operation-mode-mismatch",
        CheckedPackageRefusalCause::OperationModeTypeMismatch => "operation-mode-type-mismatch",
        CheckedPackageRefusalCause::OperationMemberMismatch => "operation-member-mismatch",
        CheckedPackageRefusalCause::OperatorIneligible => "operator-ineligible",
        CheckedPackageRefusalCause::ExpressionForm => "expression-form",
        CheckedPackageRefusalCause::DeclarationNominalMismatch => "declaration-nominal-mismatch",
        CheckedPackageRefusalCause::AmbiguousName => "ambiguous-name",
        CheckedPackageRefusalCause::MissingSelection => "missing-selection",
        CheckedPackageRefusalCause::WrongModelSelection => "wrong-model-selection",
        CheckedPackageRefusalCause::ConflictingBinding => "conflicting-binding",
        CheckedPackageRefusalCause::UnpreservedModelMeaning => "unpreserved-model-meaning",
        CheckedPackageRefusalCause::ConflictingDefinition => "conflicting-definition",
        CheckedPackageRefusalCause::InvalidValue => "invalid-value",
        CheckedPackageRefusalCause::DuplicateMember => "duplicate-member",
        CheckedPackageRefusalCause::TypeMismatch => "type-mismatch",
        CheckedPackageRefusalCause::UnsupportedSelection => "unsupported-selection",
        CheckedPackageRefusalCause::WrongSelectionRole => "wrong-selection-role",
        CheckedPackageRefusalCause::InexactInteger => "inexact-integer",
        CheckedPackageRefusalCause::InexactNumber => "inexact-number",
        CheckedPackageRefusalCause::InvalidSemanticGraph => "invalid-semantic-graph",
        CheckedPackageRefusalCause::InvalidTaskSet => "invalid-task-set",
    }
}

fn package_read_limit_mapping(limit: CheckedPackageLimit) -> Option<(&'static str, &'static str)> {
    match limit {
        CheckedPackageLimit::Bytes => {
            Some(("input-bytes-exceeded", "CheckedPackageReadLimits.bytes"))
        }
        CheckedPackageLimit::Nodes => {
            Some(("node-count-exceeded", "CheckedPackageReadLimits.nodes"))
        }
        CheckedPackageLimit::Edges => {
            Some(("edge-count-exceeded", "CheckedPackageReadLimits.edges"))
        }
        CheckedPackageLimit::Occurrences => Some((
            "occurrence-count-exceeded",
            "CheckedPackageReadLimits.occurrences",
        )),
        CheckedPackageLimit::Work => {
            Some(("work-budget-exceeded", "CheckedPackageReadLimits.work"))
        }
        CheckedPackageLimit::Diagnostics => None,
    }
}

fn manifest_encoding_exhausted(error: &quire_canonical::Error) -> bool {
    matches!(
        error,
        quire_canonical::Error::Limit(_) | quire_canonical::Error::Allocation { .. }
    )
}

/// A generator selecting an excluded driver position breaks the driver's
/// join; the other package-request errors retain CG's request classification.
fn package_request_code(error: &PackageRequestError) -> Code {
    match error {
        PackageRequestError::ExcludedDriverIndex { .. } => Code::RuntimeInvariant,
        PackageRequestError::SourceMismatch { .. }
        | PackageRequestError::ItemPositions { .. }
        | PackageRequestError::Count { .. }
        | PackageRequestError::Identity { .. }
        | PackageRequestError::AccountingIdentity { .. }
        | PackageRequestError::Position { .. }
        | PackageRequestError::OrphanContextRoot { .. }
        | PackageRequestError::ReachedClosure { .. } => Code::InvalidRequest,
    }
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
        RoutedGenerationError::ScalarSelectionMismatch { .. } => Code::InvalidRequest,
        RoutedGenerationError::Package(error) => package_request_code(error),
        RoutedGenerationError::BackendKindDisagrees { .. }
        | RoutedGenerationError::ExcludedDriverIndex { .. }
        | RoutedGenerationError::DuplicateRequestIndex { .. }
        | RoutedGenerationError::DuplicateHarness { .. }
        | RoutedGenerationError::KaniRecordCountMismatch { .. }
        | RoutedGenerationError::KaniDuplicatePositionOutOfRange { .. }
        | RoutedGenerationError::Kani(KaniObligationError::EmptyRequest) => Code::RuntimeInvariant,
        RoutedGenerationError::Kani(KaniObligationError::TooManyItems { .. }) => {
            Code::ResourceExhausted
        }
        RoutedGenerationError::ContractScalar(inner) => match inner.as_ref() {
            ContractScalarError::Request(error) => package_request_code(error),
            ContractScalarError::Oracle(oracle) => oracle_code(oracle),
        },
        RoutedGenerationError::UnauthenticatedOperationView { .. } => Code::RuntimeInvariant,
        RoutedGenerationError::MissingKindContext { .. }
        | RoutedGenerationError::Kani(
            KaniObligationError::InvalidSubjectPath | KaniObligationError::InvalidUnwind { .. },
        ) => Code::InvalidRequest,
    }
}

/// The catalog code of a whole-generation FR-014 oracle failure.
fn oracle_code(error: &OracleGenerationError) -> Code {
    match error {
        OracleGenerationError::ContextPackageMismatch { .. }
        | OracleGenerationError::ContextPackageContentMismatch
        | OracleGenerationError::MissingOriginalCheckedPackage
        | OracleGenerationError::ContractSourceMismatch => Code::InvalidRequest,
        OracleGenerationError::CheckedPackageEncoding
        | OracleGenerationError::UnauthenticatedRequestedRootKind { .. } => Code::RuntimeInvariant,
        OracleGenerationError::EnumReconstruction(_) => Code::InvalidPackage,
        OracleGenerationError::EnvironmentAdmission(failure) => match failure {
            quire_semantic_value::declaration::EnvironmentFailure::Refused(_) => {
                Code::InvalidPackage
            }
            quire_semantic_value::declaration::EnvironmentFailure::Limit(_)
            | quire_semantic_value::declaration::EnvironmentFailure::Allocation(_)
            | quire_semantic_value::declaration::EnvironmentFailure::Capacity(_) => {
                Code::ResourceExhausted
            }
        },
        OracleGenerationError::SourceTooLarge { .. }
        | OracleGenerationError::ArtifactTooLarge { .. }
        | OracleGenerationError::ArtifactCountExceeded { .. }
        | OracleGenerationError::BundleTooLarge { .. } => Code::ResourceExhausted,
        OracleGenerationError::PackageRequest(error) => package_request_code(error),
        OracleGenerationError::ContractPackageUnavailable
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
        CompileRefusal::UnitList(refusal) => refusal.cause(),
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
        RoutedGenerationError::ExcludedDriverIndex {
            request_index,
            node_id,
        } => format!("request item {request_index} targets excluded driver position {node_id:?}"),
        RoutedGenerationError::ScalarSelectionMismatch { request_index } => {
            format!("request item {request_index} did not match its admitted scalar selection")
        }
        RoutedGenerationError::Package(error) => {
            format!("the admitted package request was inconsistent: {error:?}")
        }
        RoutedGenerationError::ContractScalar(inner) => match inner.as_ref() {
            ContractScalarError::Request(error) => {
                format!("the scalar package request was inconsistent: {error:?}")
            }
            ContractScalarError::Oracle(error) => {
                format!(
                    "the derived claim map failed to generate: {}",
                    oracle_message(error)
                )
            }
        },
        RoutedGenerationError::UnauthenticatedOperationView {
            request_index,
            node_id,
        } => format!(
            "request item {request_index} targets operation {node_id:?} without an authenticated operation view"
        ),
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
        OracleGenerationError::ContextPackageMismatch { .. } => {
            format!("the selected command context does not belong to its package: {error:?}")
        }
        OracleGenerationError::ContextPackageContentMismatch => {
            "the selected command context differs from the original checked package".to_owned()
        }
        OracleGenerationError::MissingOriginalCheckedPackage => {
            "a command context was supplied without the original checked package".to_owned()
        }
        OracleGenerationError::ContractSourceMismatch => {
            "the lowered contract package names a different original checked package".to_owned()
        }
        OracleGenerationError::CheckedPackageEncoding => {
            "the original checked package could not be re-encoded for comparison".to_owned()
        }
        OracleGenerationError::UnauthenticatedRequestedRootKind { request_index, .. } => {
            format!("request item {request_index} has a root kind a plain read cannot authenticate")
        }
        OracleGenerationError::EnumReconstruction(_) => {
            format!("the admitted enum graph could not be reconstructed: {error:?}")
        }
        OracleGenerationError::EnvironmentAdmission(_) => {
            format!("semantic environment admission stopped: {error:?}")
        }
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
        EnvelopeRefusal::ContractPackageReadRefused { refusal } => format!(
            "CG ContractPackage reader refused with {}: {refusal:?}",
            package_read_refusal_code(refusal.code)
        ),
        EnvelopeRefusal::ContractPackageReadIncomplete { incomplete } => format!(
            "CG ContractPackage reader reached its {:?} limit: {incomplete:?}",
            incomplete.limit_kind
        ),
    }
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;
    use qsl_foundation::EmptyBackendIdentity;
    use quire_contract_codegen::{
        ContractNegotiationError, ContractPackageIncompleteProjection,
        ContractPackageRefusalProjection, ContractScalarError, EnvelopeRefusal, HarnessPath,
        HarnessSymbol, KaniObligationError, KaniScalarNativeError, KaniScalarNativeErrorClass,
        ModuleSymbol, OracleGenerationError, PackageRequestError, RoutedGenerationError,
        ScalarOracleStopKind,
    };
    use quire_contract_model::{
        CheckedNodeId, CheckedPackageIncomplete, CheckedPackageLimit, CheckedPackageRefusalCause,
        CheckedPackageRefusalCode, CheckedPackageRefusalLocation, JsonPointer,
    };
    use quire_exact::NodeKey;
    use quire_semantic_value::catalog::Category;

    use super::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};

    fn node_id(name: &str) -> CheckedNodeId {
        serde_json::from_value(serde_json::json!({
            "package": "pkg",
            "path": [name],
            "ordinal": 0,
        }))
        .expect("a structural node identity")
    }

    fn loci(path: Option<&str>) -> Vec<CheckedPackageRefusalLocation> {
        vec![CheckedPackageRefusalLocation {
            path: path.map(|path| JsonPointer::parse(path).expect("a valid pointer")),
            source_region: None,
        }]
    }

    // Typed upstream-error projections, not admission or solver-origin fixtures.
    #[trace("TC-004", "FR-001-AC-5")]
    #[test]
    fn new_oracle_failures_preserve_code_kind_stage_and_exact_payload() {
        use quire_semantic_value::declaration::{
            DeclarationCause, EnvironmentFailure, EnvironmentLimit, EnvironmentLimitKind,
            InvalidDeclaration, StorageRequest, StorageUnit,
        };
        use quire_semantic_value::semantic_node::{InvalidSemanticGraph, SemanticGraphCause};

        let cases = [
            (
                OracleGenerationError::ContextPackageMismatch {
                    expected: "expected-package".into(),
                    actual: "other-package".to_owned(),
                },
                "invalid-request",
                OutcomeKind::Refused,
            ),
            (
                OracleGenerationError::EnumReconstruction(InvalidSemanticGraph {
                    cause: SemanticGraphCause::NonCanonicalPreimage,
                }),
                "invalid_package",
                OutcomeKind::Refused,
            ),
            (
                OracleGenerationError::EnvironmentAdmission(EnvironmentFailure::Refused(
                    InvalidDeclaration {
                        declaration: "Repeated".to_owned(),
                        cause: DeclarationCause::DuplicateKey,
                    },
                )),
                "invalid_package",
                OutcomeKind::Refused,
            ),
            (
                OracleGenerationError::EnvironmentAdmission(EnvironmentFailure::Limit(
                    EnvironmentLimit::new(EnvironmentLimitKind::WorkUnits, 17, 18),
                )),
                "resource_exhausted",
                OutcomeKind::Limit,
            ),
            (
                OracleGenerationError::EnvironmentAdmission(EnvironmentFailure::Allocation(
                    StorageRequest::new(23, StorageUnit::Bytes),
                )),
                "resource_exhausted",
                OutcomeKind::Limit,
            ),
            (
                OracleGenerationError::EnvironmentAdmission(EnvironmentFailure::Capacity(
                    StorageRequest::new(31, StorageUnit::AdditionalElements),
                )),
                "resource_exhausted",
                OutcomeKind::Limit,
            ),
        ];
        for (error, code, kind) in cases {
            let payload = format!("{error:?}");
            let refusal = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(
                Box::new(ContractScalarError::Oracle(error.clone())),
            ));
            assert_eq!(refusal.code(), code, "{error:?}");
            assert_eq!(refusal.kind(), kind, "{error:?}");
            assert_eq!(refusal.stage(), DriverStage::Generation, "{error:?}");
            assert!(
                refusal.to_string().contains(&payload),
                "{refusal}: missing {payload}"
            );
            if matches!(
                &error,
                OracleGenerationError::EnvironmentAdmission(
                    EnvironmentFailure::Allocation(_) | EnvironmentFailure::Capacity(_)
                )
            ) {
                assert!(!refusal.to_string().contains("configured_bound"));
            }
            assert!(matches!(&refusal, DriverRefusal::Generation(
                RoutedGenerationError::ContractScalar(inner)
            ) if matches!(inner.as_ref(), ContractScalarError::Oracle(actual) if *actual == error)));
        }
    }

    #[trace("TC-004", "FR-001-AC-5", "FR-001-AC-22")]
    #[test]
    fn new_reader_causes_preserve_the_entire_typed_projection() {
        // The driver projects the supplied code; it does not reclassify an
        // owner's cause/code pairing or construct an admitted package here.
        let cases = [
            (
                CheckedPackageRefusalCause::NonBooleanRoot,
                "non-boolean-root",
            ),
            (
                CheckedPackageRefusalCause::MissingTimedModel,
                "missing-timed-model",
            ),
            (
                CheckedPackageRefusalCause::TimedNodeReference,
                "timed-node-reference",
            ),
            (
                CheckedPackageRefusalCause::DuplicateClockPlace,
                "duplicate-clock-place",
            ),
            (
                CheckedPackageRefusalCause::MalformedClockPlace,
                "malformed-clock-place",
            ),
            (
                CheckedPackageRefusalCause::InvalidTimeQuantity,
                "invalid-time-quantity",
            ),
            (
                CheckedPackageRefusalCause::ClockWithoutDenseTime,
                "clock-without-dense-time",
            ),
        ];
        for (cause, spelling) in cases {
            let projection = ContractPackageRefusalProjection {
                code: CheckedPackageRefusalCode::InvalidPackage,
                loci: loci(Some("/package/nodes/0")),
                cause: Some(cause),
                locus: Some(node_id("node")),
                contract_version: Some("2.0.0".to_owned()),
                document_pointer: Some("/package/nodes/0".to_owned()),
                extension_detail: None,
                protocol_detail: None,
                terminal_duplicate: None,
            };
            let refusal = DriverRefusal::from(EnvelopeRefusal::ContractPackageReadRefused {
                refusal: projection.clone(),
            });
            assert_eq!(refusal.code(), "invalid_package");
            assert_eq!(refusal.kind(), OutcomeKind::Refused);
            assert_eq!(refusal.stage(), DriverStage::PackageRead);
            assert_eq!(refusal.package_read_cause(), Some(spelling));
            assert_eq!(refusal.package_read_refusal_projection(), Some(&projection));
            assert!(refusal.to_string().contains(&format!("{cause:?}")));
        }
    }

    #[trace("FR-001-AC-16")]
    #[test]
    fn excluded_package_position_is_an_internal_join_fault_at_every_cg_boundary() {
        let node_id = node_id("node");
        let excluded = PackageRequestError::ExcludedDriverIndex {
            request_index: 1,
            node_id,
        };
        let direct = DriverRefusal::from(ContractNegotiationError::Package(excluded.clone()));
        assert_eq!(direct.code(), "runtime_invariant");
        assert_eq!(direct.stage(), DriverStage::Negotiation);
        assert_eq!(direct.kind(), OutcomeKind::Fault);
        for error in [
            RoutedGenerationError::Package(excluded.clone()),
            RoutedGenerationError::ContractScalar(Box::new(ContractScalarError::Request(
                excluded.clone(),
            ))),
            RoutedGenerationError::ContractScalar(Box::new(ContractScalarError::Oracle(
                OracleGenerationError::PackageRequest(excluded.clone()),
            ))),
        ] {
            let refusal = DriverRefusal::Generation(error);
            assert_eq!(refusal.code(), "runtime_invariant");
            assert_eq!(refusal.stage(), DriverStage::Generation);
            assert_eq!(refusal.kind(), OutcomeKind::Fault);
        }
    }

    #[trace("FR-001-AC-16")]
    #[test]
    fn envelope_item_count_maps_to_typed_settlement_fault() {
        let refusal = DriverRefusal::from(ContractNegotiationError::EnvelopeItemCount {
            requested: 2,
            supplied: 1,
        });
        assert!(matches!(
            refusal,
            DriverRefusal::Fault(DriverFault::SettlementCount {
                requested: 2,
                settled: 1
            })
        ));
    }

    #[trace("FR-009-AC-6", "FR-009-AC-7")]
    #[test]
    fn native_runner_typed_refusal_keeps_cg_discriminant_and_class() {
        let refusal = DriverRefusal::ScalarNative(KaniScalarNativeError::OracleStop(
            ScalarOracleStopKind::IllTyped,
        ));
        assert_eq!(refusal.code(), "oracle_stop");
        assert_eq!(refusal.qsl_category(), None);
        assert_eq!(refusal.stage(), DriverStage::Replay);
        assert_eq!(refusal.kind(), OutcomeKind::Refused);
        assert_eq!(
            match &refusal {
                DriverRefusal::ScalarNative(error) => error.class(),
                _ => unreachable!("test constructed the scalar-native refusal"),
            },
            KaniScalarNativeErrorClass::CompletionRefusal
        );

        let fault = DriverRefusal::ScalarNative(KaniScalarNativeError::DependencyLock {
            expected: "owner".to_owned(),
            actual: None,
        });
        assert_eq!(fault.code(), "dependency_lock");
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.stage(), DriverStage::Replay);
    }

    #[test]
    #[trace("FR-001-AC-16", "TC-006")]
    fn excluded_generation_index_is_an_internal_join_fault() {
        let node_id = node_id("node");
        let refusal = DriverRefusal::Generation(RoutedGenerationError::ExcludedDriverIndex {
            request_index: 1,
            node_id: node_id.clone(),
        });
        assert_eq!(refusal.code(), "runtime_invariant");
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(refusal.stage(), DriverStage::Generation);
        assert!(refusal.to_string().contains("request item 1"));
        assert!(refusal.to_string().contains("excluded"));
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

    #[trace("TC-004", "FR-001-AC-22")]
    #[test]
    fn cg_package_read_refusal_preserves_projection_and_maps_its_typed_code_and_cause() {
        let projection = ContractPackageRefusalProjection {
            code: CheckedPackageRefusalCode::WrongSnapshot,
            loci: loci(Some("/package/dependencies/0")),
            cause: Some(CheckedPackageRefusalCause::WrongObservation),
            locus: Some(node_id("node")),
            contract_version: Some("2.0.0".to_owned()),
            document_pointer: Some("/package/dependencies/0".to_owned()),
            extension_detail: None,
            protocol_detail: None,
            terminal_duplicate: None,
        };
        let refusal = DriverRefusal::from(EnvelopeRefusal::ContractPackageReadRefused {
            refusal: projection.clone(),
        });

        assert_eq!(refusal.code(), "wrong_snapshot");
        assert_eq!(refusal.package_read_cause(), Some("wrong-observation"));
        assert_eq!(refusal.qsl_category(), Some(Category::Refusal));
        assert_eq!(refusal.kind(), OutcomeKind::Refused);
        assert_eq!(refusal.stage(), DriverStage::PackageRead);
        assert_eq!(refusal.package_read_refusal_projection(), Some(&projection));
        let diagnostic = refusal.package_read_diagnostic().unwrap();
        assert_eq!(diagnostic.code(), "wrong_snapshot");
        assert_eq!(diagnostic.cause(), Some("wrong-observation"));
        assert_eq!(diagnostic.category(), Category::Refusal);
        assert_eq!(diagnostic.stage(), DriverStage::PackageRead);
        assert_eq!(
            diagnostic.location(),
            Some(&super::PackageReadLocation::JsonPath(
                "/package/dependencies/0".to_owned()
            ))
        );
        assert_eq!(diagnostic.locus(), projection.locus.as_ref());
        assert!(matches!(
            refusal,
            DriverRefusal::PackageRead(failure)
                if matches!(failure.source(), EnvelopeRefusal::ContractPackageReadRefused { .. })
        ));

        let projection_without_optional_payloads = ContractPackageRefusalProjection {
            code: CheckedPackageRefusalCode::MalformedWire,
            loci: loci(None),
            cause: None,
            locus: None,
            contract_version: None,
            document_pointer: None,
            extension_detail: None,
            protocol_detail: None,
            terminal_duplicate: None,
        };
        let refusal = DriverRefusal::from(EnvelopeRefusal::ContractPackageReadRefused {
            refusal: projection_without_optional_payloads.clone(),
        });
        assert_eq!(refusal.code(), "malformed_wire");
        assert_eq!(refusal.package_read_cause(), None);
        let diagnostic = refusal.package_read_diagnostic().unwrap();
        assert_eq!(diagnostic.code(), "malformed_wire");
        assert_eq!(diagnostic.cause(), None);
        assert_eq!(diagnostic.location(), None);
        assert_eq!(diagnostic.locus(), None);
        assert_eq!(
            refusal.package_read_refusal_projection(),
            Some(&projection_without_optional_payloads)
        );

        let invalid_value_projection = ContractPackageRefusalProjection {
            code: CheckedPackageRefusalCode::InvalidPackage,
            loci: loci(Some("/package/cause")),
            cause: Some(CheckedPackageRefusalCause::InvalidValue),
            locus: None,
            contract_version: None,
            document_pointer: Some("/package/cause".to_owned()),
            extension_detail: None,
            protocol_detail: None,
            terminal_duplicate: None,
        };
        let refusal = DriverRefusal::from(EnvelopeRefusal::ContractPackageReadRefused {
            refusal: invalid_value_projection,
        });
        assert_eq!(refusal.code(), "invalid_package");
        assert_eq!(refusal.package_read_cause(), Some("invalid-value"));
    }

    #[trace("TC-004", "FR-001-AC-22")]
    #[test]
    fn cg_package_read_incomplete_maps_each_permitted_limit_and_faults_diagnostics() {
        let cases = [
            (
                CheckedPackageLimit::Bytes,
                "input-bytes-exceeded",
                "CheckedPackageReadLimits.bytes",
            ),
            (
                CheckedPackageLimit::Nodes,
                "node-count-exceeded",
                "CheckedPackageReadLimits.nodes",
            ),
            (
                CheckedPackageLimit::Edges,
                "edge-count-exceeded",
                "CheckedPackageReadLimits.edges",
            ),
            (
                CheckedPackageLimit::Occurrences,
                "occurrence-count-exceeded",
                "CheckedPackageReadLimits.occurrences",
            ),
            (
                CheckedPackageLimit::Work,
                "work-budget-exceeded",
                "CheckedPackageReadLimits.work",
            ),
        ];
        for (limit_kind, cause, setting) in cases {
            let source_path =
                (limit_kind != CheckedPackageLimit::Bytes).then(|| "/reader/path".to_owned());
            let projection = ContractPackageIncompleteProjection {
                limit_kind,
                limit: 10,
                consumed: 11,
                path: source_path.clone(),
            };
            let refusal = DriverRefusal::from(EnvelopeRefusal::ContractPackageReadIncomplete {
                incomplete: projection.clone(),
            });
            assert_eq!(refusal.code(), "stage_limit_exceeded");
            assert_eq!(refusal.qsl_category(), Some(Category::Incomplete));
            assert_eq!(refusal.kind(), OutcomeKind::Limit);
            assert_eq!(refusal.stage(), DriverStage::PackageRead);
            let diagnostic = refusal.package_read_diagnostic().unwrap();
            assert_eq!(diagnostic.code(), "stage_limit_exceeded");
            assert_eq!(diagnostic.cause(), Some(cause));
            assert_eq!(diagnostic.category(), Category::Incomplete);
            assert_eq!(diagnostic.stage(), DriverStage::PackageRead);
            assert_eq!(
                diagnostic.location(),
                source_path
                    .as_ref()
                    .map(|path| super::PackageReadLocation::JsonPath(path.clone()))
                    .as_ref()
            );
            assert_eq!(diagnostic.locus(), None);
            assert_eq!(
                refusal.package_read_limit(),
                Some(super::PackageReadLimit {
                    cause,
                    setting,
                    bound: 10,
                    actual: 11,
                    path: source_path,
                })
            );
            assert!(matches!(
                &refusal,
                DriverRefusal::PackageRead(failure)
                    if matches!(failure.source(), EnvelopeRefusal::ContractPackageReadIncomplete { incomplete } if incomplete == &projection)
            ));
        }

        let projection = ContractPackageIncompleteProjection {
            limit_kind: CheckedPackageLimit::Diagnostics,
            limit: 10,
            consumed: 11,
            path: Some("/diagnostics".to_owned()),
        };
        let refusal = DriverRefusal::from(EnvelopeRefusal::ContractPackageReadIncomplete {
            incomplete: projection.clone(),
        });
        assert_eq!(refusal.code(), "runtime_invariant");
        assert_eq!(refusal.kind(), OutcomeKind::Fault);
        assert_eq!(refusal.stage(), DriverStage::PackageRead);
        assert!(refusal.package_read_diagnostic().is_none());
        assert!(matches!(
            refusal,
            DriverRefusal::Fault(DriverFault::PackageReadDiagnostics(incomplete))
                if incomplete == projection
        ));
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
                RoutedGenerationError::ExcludedDriverIndex {
                    request_index: 1,
                    node_id: node_id("node"),
                },
                format!(
                    "CG generation refused the routed items: request item 1 targets excluded driver position {:?}",
                    node_id("node")
                ),
            ),
            (
                RoutedGenerationError::DuplicateHarness {
                    harness: HarnessPath {
                        module: ModuleSymbol::try_from("subject").expect("valid module"),
                        harness: HarnessSymbol::try_from("proof").expect("valid harness"),
                    },
                },
                "CG generation refused the routed items: Kani harness `subject::proof` is emitted more than once".to_owned(),
            ),
            (
                RoutedGenerationError::KaniRecordCountMismatch {
                    records: 1,
                    items: 2,
                },
                "CG generation refused the routed items: Kani returned 1 record(s) for 2 item(s)".to_owned(),
            ),
            (
                RoutedGenerationError::KaniDuplicatePositionOutOfRange {
                    first_index: 3,
                    items: 2,
                },
                "CG generation refused the routed items: Kani duplicate position 3 is outside 2 item(s)".to_owned(),
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

        let fault = DriverRefusal::Generation(RoutedGenerationError::DuplicateRequestIndex {
            request_index: 0,
        });
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        // IR-294: a whole-generation FR-014 oracle failure the driver never
        // produces is still an internal fault, not a refusal: an unknown
        // Contract Runtime variant and a claim-map serialization failure.
        let fault = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(Box::new(
            ContractScalarError::Oracle(OracleGenerationError::UnknownRuntimeVariant {
                enum_name: "IntegerDomain",
            }),
        )));
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        let fault = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(Box::new(
            ContractScalarError::Oracle(OracleGenerationError::ClaimMapSerialization),
        )));
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        // A routed item naming a node the checked package does not hold is
        // an E4/E5 inconsistency, not a caller input: every item's node
        // comes from the same package's own `requirements()`, so only a
        // broken invariant between emission and admission reaches this.
        let fault = DriverRefusal::Fault(DriverFault::UnknownNode {
            request_index: 1,
            node: NodeKey::new(
                "pkg".to_owned(),
                vec![quire_exact::PathSegment::Text("n".to_owned())],
                0,
            )
            .expect("a fixture key is admissible"),
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
        let limit = DriverRefusal::Generation(RoutedGenerationError::ContractScalar(Box::new(
            ContractScalarError::Oracle(OracleGenerationError::SourceTooLarge {
                bytes: 2,
                limit: 1,
                setting: "cg.scalar.source_bytes",
            }),
        )));
        assert_eq!(limit.kind(), OutcomeKind::Limit);
        assert_eq!(limit.code(), "resource_exhausted");
        assert_eq!(limit.stage(), DriverStage::Generation);
    }
}
