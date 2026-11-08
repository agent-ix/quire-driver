// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed monitor preparation stops, separate from runtime outcomes.

use quire_contract_model::{
    CheckedCallableReadError, CheckedCallableReadErrorCode, CheckedMonitorClockReadError,
    CheckedMonitorClockReadErrorCode, CheckedNodeId, CheckedOperationViewError,
    CheckedPackageIncomplete, CheckedPackageRefusal, ExactTypeProjectionError,
    ExactTypeProjectionErrorCode,
};
use quire_contract_runtime::monitor::ProgramRefusal;
use quire_exact::{CancelCause, InvalidNodeKey};

use crate::monitor_refusal::{MonitorLocusError, TargetMismatch, UnsupportedOperation};

/// The independently selected value that failed to match admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorSelectionField {
    /// The request names another immutable package instance.
    Package,
    /// The selected package has no such admitted monitor root.
    Root,
    /// The independently selected immutable package lock differs.
    PackageLock,
    /// The full root content key differs.
    RootKey,
    /// One of the four independently selected entry keys differs.
    EntryKey,
    /// The independently selected clause differs.
    Clause,
    /// The independent subject differs from the clause's admitted `over` parameter.
    Subject,
    /// The full selected occurrence does not belong to the selected clause.
    Occurrence,
    /// The independent profile is absent from the selected package's lock.
    Profile,
    /// The optional independently selected configured clock row differs.
    Clock,
    /// Strict admission of the supplied emission differs from the selected package.
    EmissionPackage,
    /// Native producer package, occurrence node or Claim role is inconsistent.
    ProducerSubject,
    /// The owner's final declaration does not match the independent clause.
    FinalDeclaration,
    /// The native Claim role/ordinal does not match the independent occurrence.
    FinalOccurrence,
    /// The native name at the exact final declaration/ordinal differs.
    ClaimName,
    /// Buffer owner/count differs; no corresponding admitted capacity value exists.
    BufferAssociation,
}

/// Stable categories for callers that do not need a native payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorPreparationErrorCode {
    /// Independent selection failed before lowering.
    Selection,
    /// An independently selected target premise differs.
    TargetMismatch,
    /// A reached ordinary operation has no supported runtime lowering.
    Unsupported,
    /// An upstream Exact projection refused; its native payload is retained.
    Projection,
    /// Original runtime program construction refused before any entry executed.
    Program,
    /// Operating work was exhausted before the next inspection.
    Work,
    /// Operating storage was exhausted or allocation failed.
    Storage,
    /// The caller cancelled preparation.
    Cancelled,
    /// A supposedly valid diagnostic locus could not be constructed.
    Internal,
}

impl MonitorPreparationErrorCode {
    /// Stable machine-readable spelling, independent of diagnostic rendering.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Selection => "selection",
            Self::TargetMismatch => "target_mismatch",
            Self::Unsupported => "unsupported",
            Self::Projection => "projection",
            Self::Program => "program",
            Self::Work => "work",
            Self::Storage => "storage",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }

    /// Every top-level category exposed by this preparation API.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Selection,
            Self::TargetMismatch,
            Self::Unsupported,
            Self::Projection,
            Self::Program,
            Self::Work,
            Self::Storage,
            Self::Cancelled,
            Self::Internal,
        ]
    }

    /// Decode an exact stable spelling; unknown or differently cased text is absent.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|candidate| candidate.as_str() == code)
    }
}

/// A typed stop with no partial program, environment, or temporal verdict.
#[derive(Debug, thiserror::Error)]
pub enum MonitorPreparationError {
    /// A selected identity failed to match the selected immutable package.
    #[error("monitor selection mismatch: {field:?}")]
    Selection {
        /// Which independently selected value differs.
        field: MonitorSelectionField,
        /// Selected native node at the stop.
        node: Box<CheckedNodeId>,
    },
    /// Original target-mismatch payload, including the helper's unchanged value path.
    #[error("monitor target mismatch: {0:?}")]
    TargetMismatch(Box<TargetMismatch>),
    /// Original unsupported-operation payload and exact inline source locus.
    #[error("monitor operation unsupported: {0:?}")]
    Unsupported(Box<UnsupportedOperation>),
    /// The native IR type-projection refusal without relabelling its cause.
    #[error(transparent)]
    Projection(#[from] ExactTypeProjectionError),
    /// The original bounded selective callable-read failure and its first locus.
    #[error(transparent)]
    CallableRead(Box<CheckedCallableReadError>),
    /// Original bounded IR clock-read refusal, retaining its exact cause and locus.
    #[error(transparent)]
    ClockRead(Box<CheckedMonitorClockReadError>),
    /// The native declaration lookup has no row for the original subject.
    #[error("no final declaration for monitor subject: {0:?}")]
    MissingFinalSubject(Box<qsl_replay::MonitorSubject>),
    /// Original strict native emission-admission refusal.
    #[error("monitor emission admission refused: {0:?}")]
    EmissionRefused(Box<CheckedPackageRefusal>),
    /// The native catalog acquisition failure, separate from malformed input.
    #[error(transparent)]
    CatalogUnavailable(quire_contract_model::CatalogReadError),
    /// Original strict native acquisition limit, outside the driver ledger.
    #[error("monitor emission acquisition incomplete: {0:?}")]
    EmissionIncomplete(Box<CheckedPackageIncomplete>),
    /// The original runtime construction refusal, before initialization.
    #[error(transparent)]
    Program(Box<ProgramRefusal>),
    /// An operation view was requested without its native admission proof.
    #[error(transparent)]
    OperationRead(#[from] CheckedOperationViewError),
    /// Native structural-key copy/allocation failure, without replacing its payload.
    #[error(transparent)]
    Key(#[from] InvalidNodeKey),
    /// Work stopped before inspecting the named input.
    #[error("monitor preparation work limit {limit} after {consumed}, next {next}")]
    Work {
        /// Caller-selected limit.
        limit: u64,
        /// Successfully admitted work.
        consumed: u64,
        /// Unadmitted next amount.
        next: u64,
    },
    /// Temporary storage stopped before reserving more material.
    #[error("monitor preparation storage limit {limit} after {consumed}, next {next}")]
    Storage {
        /// Caller-selected limit.
        limit: u64,
        /// Currently reserved live logical bytes.
        consumed: u64,
        /// Unreserved next byte count.
        next: u64,
    },
    /// Caller cancellation remains an operating stop.
    #[error("monitor preparation cancelled: {0:?}")]
    Cancelled(CancelCause),
    /// Diagnostic construction violated the driver's lowering invariant.
    #[error("monitor preparation internal locus fault: {0:?}")]
    Internal(MonitorLocusError),
    /// Native allocation failure while reserving the iterative identity worklist.
    #[error(transparent)]
    Allocation(#[from] std::collections::TryReserveError),
    /// Invalid internal release bookkeeping; the live reservation is unchanged.
    #[error(
        "monitor preparation internal storage release {released} exceeds reservation {reserved}"
    )]
    StorageAccounting {
        /// Live reservation before the invalid release.
        reserved: u64,
        /// Byte count the internal caller attempted to release.
        released: u64,
    },
    /// A strictly admitted clause no longer has its required retained body shape.
    #[error("monitor preparation internal admitted-clause fault")]
    InvalidAdmittedClause {
        /// The admitted clause whose retained shape is inconsistent.
        node: Box<CheckedNodeId>,
    },
    /// An admitted monitor entry lost its required expression-reference body.
    #[error("monitor preparation internal admitted-entry fault")]
    InvalidAdmittedEntry {
        /// Entry whose retained body disagrees with strict monitor admission.
        node: Box<CheckedNodeId>,
    },
    /// Missing or cyclic scalar facts violated the driver's admitted-input invariant.
    #[error("monitor preparation internal admitted-scalar fault")]
    InvalidAdmittedScalar {
        /// Original admitted scalar node at the invariant failure.
        node: Box<CheckedNodeId>,
    },
}

impl MonitorPreparationError {
    /// Stable top-level category; original payloads remain available in the enum.
    pub fn code(&self) -> MonitorPreparationErrorCode {
        match self {
            Self::Selection { .. } => MonitorPreparationErrorCode::Selection,
            Self::TargetMismatch(_) => MonitorPreparationErrorCode::TargetMismatch,
            Self::Unsupported(_) => MonitorPreparationErrorCode::Unsupported,
            Self::Projection(native) => match native.code {
                ExactTypeProjectionErrorCode::ExhaustedWork => MonitorPreparationErrorCode::Work,
                ExactTypeProjectionErrorCode::ExhaustedStorage => {
                    MonitorPreparationErrorCode::Storage
                }
                ExactTypeProjectionErrorCode::UnknownNode
                | ExactTypeProjectionErrorCode::ForeignNode
                | ExactTypeProjectionErrorCode::IncompleteIdentityWitness
                | ExactTypeProjectionErrorCode::UnavailableIdentityWitness
                | ExactTypeProjectionErrorCode::MismatchedIdentityWitness
                | ExactTypeProjectionErrorCode::ConflictingIdentityWitness
                | ExactTypeProjectionErrorCode::UnresolvedSelectedDeclaration
                | ExactTypeProjectionErrorCode::UnsupportedExactType => {
                    MonitorPreparationErrorCode::Projection
                }
            },
            Self::CallableRead(native) => match native.code {
                CheckedCallableReadErrorCode::WorkLimit => MonitorPreparationErrorCode::Work,
                CheckedCallableReadErrorCode::UnknownNode
                | CheckedCallableReadErrorCode::MissingBody
                | CheckedCallableReadErrorCode::MalformedBody
                | CheckedCallableReadErrorCode::AmbiguousBody
                | CheckedCallableReadErrorCode::UnavailableWitness
                | CheckedCallableReadErrorCode::MissingSelectedJoin
                | CheckedCallableReadErrorCode::ForeignSelectedJoin
                | CheckedCallableReadErrorCode::MissingFrame => {
                    MonitorPreparationErrorCode::Projection
                }
            },
            Self::ClockRead(native) => match native.code {
                CheckedMonitorClockReadErrorCode::WorkLimit => MonitorPreparationErrorCode::Work,
                CheckedMonitorClockReadErrorCode::StorageLimit => {
                    MonitorPreparationErrorCode::Storage
                }
                CheckedMonitorClockReadErrorCode::WrongClause
                | CheckedMonitorClockReadErrorCode::WrongRole
                | CheckedMonitorClockReadErrorCode::UnknownOccurrence
                | CheckedMonitorClockReadErrorCode::MissingClock => {
                    MonitorPreparationErrorCode::Selection
                }
            },
            Self::MissingFinalSubject(_) | Self::EmissionRefused(_) => {
                MonitorPreparationErrorCode::Selection
            }
            Self::CatalogUnavailable(_) => MonitorPreparationErrorCode::Internal,
            Self::EmissionIncomplete(_) => MonitorPreparationErrorCode::Work,
            Self::Program(_) => MonitorPreparationErrorCode::Program,
            Self::OperationRead(_) => MonitorPreparationErrorCode::Internal,
            Self::Key(native) => match native {
                InvalidNodeKey::Allocation { .. } => MonitorPreparationErrorCode::Storage,
                InvalidNodeKey::EmptyPackage
                | InvalidNodeKey::EmptyPath
                | InvalidNodeKey::OrdinalOutOfRange => MonitorPreparationErrorCode::Internal,
            },
            Self::Work { .. } => MonitorPreparationErrorCode::Work,
            Self::Storage { .. } => MonitorPreparationErrorCode::Storage,
            Self::Cancelled(_) => MonitorPreparationErrorCode::Cancelled,
            Self::Internal(_) => MonitorPreparationErrorCode::Internal,
            Self::Allocation(_) => MonitorPreparationErrorCode::Storage,
            Self::StorageAccounting { .. } => MonitorPreparationErrorCode::Internal,
            Self::InvalidAdmittedClause { .. } => MonitorPreparationErrorCode::Internal,
            Self::InvalidAdmittedEntry { .. } => MonitorPreparationErrorCode::Internal,
            Self::InvalidAdmittedScalar { .. } => MonitorPreparationErrorCode::Internal,
        }
    }
}

impl From<CheckedCallableReadError> for MonitorPreparationError {
    fn from(native: CheckedCallableReadError) -> Self {
        Self::CallableRead(Box::new(native))
    }
}

impl From<ProgramRefusal> for MonitorPreparationError {
    fn from(native: ProgramRefusal) -> Self {
        Self::Program(Box::new(native))
    }
}
