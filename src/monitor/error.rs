// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed monitor preparation stops, separate from runtime outcomes.

use quire_contract_model::{CheckedNodeId, ExactTypeProjectionError};
use quire_exact::CancelCause;

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
    /// The original selected clause clock row is missing or differs.
    Clock,
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
    /// Operating work was exhausted before the next inspection.
    Work,
    /// Operating storage was exhausted or allocation failed.
    Storage,
    /// The caller cancelled preparation.
    Cancelled,
    /// A supposedly valid diagnostic locus could not be constructed.
    Internal,
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
    TargetMismatch(TargetMismatch),
    /// Original unsupported-operation payload and exact inline source locus.
    #[error("monitor operation unsupported: {0:?}")]
    Unsupported(UnsupportedOperation),
    /// The native IR type-projection refusal without relabelling its cause.
    #[error(transparent)]
    Projection(#[from] ExactTypeProjectionError),
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
        /// Successfully reserved cumulative bytes.
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
    /// Native serialization failed while accounting admitted value material.
    #[error("monitor preparation internal serialization fault: {0}")]
    InternalSerialization(serde_json::Error),
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
}

impl MonitorPreparationError {
    /// Stable top-level category; original payloads remain available in the enum.
    pub fn code(&self) -> MonitorPreparationErrorCode {
        match self {
            Self::Selection { .. } => MonitorPreparationErrorCode::Selection,
            Self::TargetMismatch(_) => MonitorPreparationErrorCode::TargetMismatch,
            Self::Unsupported(_) => MonitorPreparationErrorCode::Unsupported,
            Self::Projection(_) => MonitorPreparationErrorCode::Projection,
            Self::Work { .. } => MonitorPreparationErrorCode::Work,
            Self::Storage { .. } => MonitorPreparationErrorCode::Storage,
            Self::Cancelled(_) => MonitorPreparationErrorCode::Cancelled,
            Self::Internal(_) => MonitorPreparationErrorCode::Internal,
            Self::InternalSerialization(_) => MonitorPreparationErrorCode::Internal,
            Self::InvalidAdmittedClause { .. } => MonitorPreparationErrorCode::Internal,
            Self::InvalidAdmittedEntry { .. } => MonitorPreparationErrorCode::Internal,
        }
    }
}
