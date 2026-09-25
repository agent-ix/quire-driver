// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's command-level refusals (ADR-011 §2.3, §5; ADR-012 §7.2).
//! AGE-394 (the lifecycle and CLI design) renders them; this module only types
//! them.

use qsl_foundation::Code;
use qsl_replay::spine::{CompileRefusal, SpineStage};
use qsl_route::{BackendId, RegistrationRefusal};
use quire_contract_codegen::EnvelopeRefusal;

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
    /// An upstream answer broke an invariant the driver relies on. Not a
    /// refusal of the input.
    #[error("internal fault: {0}")]
    Fault(DriverFault),
}

/// A violated invariant between the driver and an upstream crate.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DriverFault {
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
    #[error("routing returned no target for a supported item")]
    RoutingTarget,
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
            Self::Compile(refusal) => refusal.code().as_str().to_owned(),
            Self::UnknownBackend { .. } => "invalid_capability/unknown-backend".to_owned(),
            Self::Envelope(EnvelopeRefusal::InvalidCapability { cause, .. }) => {
                format!("invalid_capability/{cause}")
            }
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
            Self::Registration(_)
            | Self::Compile(_)
            | Self::UnknownBackend { .. }
            | Self::Envelope(_) => OutcomeKind::Refused,
            Self::Fault(_) => OutcomeKind::Fault,
        }
    }
}

fn join(backends: &[BackendId]) -> String {
    backends
        .iter()
        .map(BackendId::as_str)
        .collect::<Vec<_>>()
        .join(", ")
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
    use super::{DriverFault, DriverRefusal, DriverStage, OutcomeKind};

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
    }
}
