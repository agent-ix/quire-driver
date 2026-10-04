// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's command-level refusals (ADR-011 §2.3, §5; ADR-012 §7.2).
//! AGE-394 (the lifecycle and CLI design) renders them; this module only types
//! them.

use qsl_foundation::diagnostic::InternalFault;
use qsl_foundation::digest::WireNodeId;
use qsl_foundation::Code;
use qsl_replay::spine::{CompileRefusal, SpineStage};
use qsl_route::{BackendId, RegistrationRefusal};
use quire_contract_codegen::{
    EnvelopeRefusal, KaniObligationError, OracleGenerationError, RoutedGenerationError,
};
use quire_contract_model::{CheckedPackageIncomplete, CheckedPackageRefusal};

/// Why [`crate::drive`] or [`crate::build_registry`] produced no result.
#[derive(Debug, thiserror::Error)]
pub enum DriverRefusal {
    /// A provider manifest did not register. Every refusal, ordered by
    /// backend identity then manifest digest.
    #[error("{} backend registration(s) refused", .0.len())]
    Registration(Vec<RegistrationRefusal>),
    /// S1 to S4, or the E4 emitter, refused the unit.
    #[error("{0}")]
    Compile(Box<CompileRefusal>),
    /// MD-2: a registered `BackendId` has no CG backend kind. Every
    /// unconverted id, in bytewise order. Raised before any negotiation.
    #[error("invalid_capability/unknown-backend: no CG backend kind for {}", join(.backends))]
    UnknownBackend {
        /// Each unconverted `BackendId`, in bytewise order.
        backends: Vec<BackendId>,
    },
    /// CG refused the whole FR-331 envelope before settling any item.
    #[error("CG refused the negotiation envelope: {}", envelope_message(.0))]
    Envelope(EnvelopeRefusal),
    /// E5: IR's reader stopped at one of its limits.
    #[error("IR's checked-package reader reached its {:?} limit", .0.limit_kind)]
    HandoffLimit(CheckedPackageIncomplete),
    /// CG generation refused the routed items as a whole.
    #[error("CG generation refused the routed items: {}", generation_message(.0))]
    Generation(RoutedGenerationError),
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
    HandoffRefused(CheckedPackageRefusal),
    /// IR admitted the bytes at a `package_id` other than the emitted one.
    #[error("IR admitted package {admitted}, not the emitted {emitted}")]
    PackageIdMismatch {
        /// The emitted `package_id` digest.
        emitted: String,
        /// The admitted `package_id` digest.
        admitted: String,
    },
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
}

/// The terminal outcome kind a refusal maps to (ADR-011 §2.3, §5): a
/// refusal, a limit refusal and an internal fault are distinct.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeKind {
    /// The input was refused.
    Refused,
    /// A stage reached one of its limits.
    Limit,
    /// An internal invariant was violated.
    Fault,
}

impl DriverRefusal {
    /// The catalog code, rendered `code` or `code/cause`.
    ///
    /// For [`DriverRefusal::Registration`] this is the first refusal's code
    /// in (identity, manifest digest) order; each refusal in the vector
    /// carries its own `catalog_code()`.
    pub fn code(&self) -> String {
        match self {
            Self::Registration(refusals) => refusals.first().map_or_else(
                || "invalid_capability".to_owned(),
                |refusal| refusal.catalog_code().to_string(),
            ),
            Self::Compile(refusal) => compile_code(refusal),
            Self::UnknownBackend { .. } => "invalid_capability/unknown-backend".to_owned(),
            Self::Envelope(EnvelopeRefusal::InvalidCapability { cause, .. }) => {
                format!("invalid_capability/{cause}")
            }
            Self::HandoffLimit(_) => Code::StageLimitExceeded.as_str().to_owned(),
            Self::Generation(error) => generation_code(error).as_str().to_owned(),
            Self::Fault(_) => Code::RuntimeInvariant.as_str().to_owned(),
        }
    }

    /// The step that refused.
    pub fn stage(&self) -> DriverStage {
        match self {
            Self::Registration(_) => DriverStage::Registry,
            Self::Compile(refusal) => DriverStage::Compile(refusal.stage()),
            Self::UnknownBackend { .. } | Self::Fault(DriverFault::VocabularyMismatch { .. }) => {
                DriverStage::Conversion
            }
            Self::Envelope(_) => DriverStage::Negotiation,
            Self::Fault(DriverFault::RoutingTarget) => DriverStage::Routing,
            Self::Fault(DriverFault::Spine { stage, .. }) => DriverStage::Compile(*stage),
            Self::HandoffLimit(_)
            | Self::Fault(DriverFault::HandoffRefused(_) | DriverFault::PackageIdMismatch { .. }) => {
                DriverStage::Handoff
            }
            Self::Generation(_) | Self::Fault(DriverFault::UnknownNode { .. }) => {
                DriverStage::Generation
            }
            Self::Fault(
                DriverFault::SettlementCount { .. }
                | DriverFault::SupportedOutsideCandidates { .. },
            ) => DriverStage::Negotiation,
        }
    }

    /// The outcome kind this refusal settles.
    pub fn kind(&self) -> OutcomeKind {
        match self {
            Self::Compile(refusal) if refusal.code() == Code::StageLimitExceeded => {
                OutcomeKind::Limit
            }
            Self::HandoffLimit(_) => OutcomeKind::Limit,
            Self::Generation(error) => match generation_code(error) {
                Code::RuntimeInvariant => OutcomeKind::Fault,
                Code::ResourceExhausted => OutcomeKind::Limit,
                _ => OutcomeKind::Refused,
            },
            Self::Registration(_)
            | Self::Compile(_)
            | Self::UnknownBackend { .. }
            | Self::Envelope(_) => OutcomeKind::Refused,
            Self::Fault(_) => OutcomeKind::Fault,
        }
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
        RoutedGenerationError::BackendKindDisagrees { .. }
        | RoutedGenerationError::DuplicateRequestIndex { .. }
        | RoutedGenerationError::DuplicateHarness { .. }
        | RoutedGenerationError::KaniRecordCountMismatch { .. }
        | RoutedGenerationError::KaniDuplicatePositionOutOfRange { .. }
        | RoutedGenerationError::Kani(KaniObligationError::EmptyRequest) => Code::RuntimeInvariant,
        RoutedGenerationError::Kani(KaniObligationError::TooManyItems { .. }) => {
            Code::ResourceExhausted
        }
        RoutedGenerationError::MissingKindContext { .. }
        | RoutedGenerationError::Kani(
            KaniObligationError::InvalidSubjectPath | KaniObligationError::InvalidUnwind { .. },
        ) => Code::InvalidRequest,
        RoutedGenerationError::Oracle(oracle) => oracle_code(oracle),
    }
}

/// The catalog code of a whole-generation FR-014 oracle failure.
fn oracle_code(error: &OracleGenerationError) -> Code {
    match error {
        OracleGenerationError::SourceTooLarge { .. } => Code::ResourceExhausted,
        OracleGenerationError::ClaimMapSerialization
        | OracleGenerationError::LocationMapSerialization
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
        RoutedGenerationError::BackendKindDisagrees {
            request_index,
            backend,
            routed,
            converted,
        } => format!(
            "request item {request_index} was routed to `{}` as {}, which converts to {}",
            backend.identity,
            routed.identity(),
            converted.map_or("no kind", |kind| kind.identity())
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
        RoutedGenerationError::Oracle(error) => {
            format!(
                "the derived claim map failed to generate: {}",
                oracle_message(error)
            )
        }
    }
}

fn oracle_message(error: &OracleGenerationError) -> String {
    match error {
        OracleGenerationError::SourceTooLarge { bytes } => {
            format!("the generated source is {bytes} byte(s), over the ceiling")
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
        } => format!("invalid_capability/{cause}: `{member}` is `{received}`"),
    }
}

#[cfg(test)]
mod tests {
    use qsl_foundation::digest::WireNodeId;
    use quire_contract_codegen::{
        HarnessPath, HarnessSymbol, KaniObligationError, ModuleSymbol, OracleGenerationError,
        RoutedGenerationError,
    };
    use quire_contract_model::{CheckedPackageIncomplete, CheckedPackageLimit};

    use super::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};

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
                RoutedGenerationError::KaniRecordCountMismatch { records: 1, items: 2 },
                "CG generation refused the routed items: Kani returned 1 record(s) for 2 item(s)",
            ),
            (
                RoutedGenerationError::KaniDuplicatePositionOutOfRange { first_index: 3, items: 2 },
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
        let fault = DriverRefusal::Generation(RoutedGenerationError::Oracle(
            OracleGenerationError::UnknownRuntimeVariant {
                enum_name: "IntegerDomain",
            },
        ));
        assert_eq!(fault.kind(), OutcomeKind::Fault);
        assert_eq!(fault.code(), "runtime_invariant");
        assert_eq!(fault.stage(), DriverStage::Generation);

        let fault = DriverRefusal::Generation(RoutedGenerationError::Oracle(
            OracleGenerationError::ClaimMapSerialization,
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
        let limit = DriverRefusal::Generation(RoutedGenerationError::Oracle(
            OracleGenerationError::SourceTooLarge { bytes: 1 },
        ));
        assert_eq!(limit.kind(), OutcomeKind::Limit);
        assert_eq!(limit.code(), "resource_exhausted");
        assert_eq!(limit.stage(), DriverStage::Generation);
    }
}
