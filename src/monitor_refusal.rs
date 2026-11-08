// SPDX-License-Identifier: AGPL-3.0-or-later
//! Diagnostic retention for monitor preparation; selection and support policy stay with the caller.

mod copy;

pub use copy::{
    copy_monitor_locus, CopiedMonitorLocus, MonitorLocusCopyError, MonitorLocusCopyErrorCode,
};

use quire_contract_model::{
    CheckedApplicationOperation, CheckedNodeId, CheckedNodeKind, CheckedOccurrence,
    CheckedPackageV2, CheckedTerm,
};

/// The original target premise whose independent comparison failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetPremise {
    /// Hardware counter name.
    Counter,
    /// Hardware counter width.
    CounterWidth,
    /// Original tick unit.
    TickUnit,
    /// Largest permitted counter reading gap.
    MaxReadingGap,
    /// Exact event rate.
    MaxEventRate,
    /// Exact clock uncertainty.
    Uncertainty,
    /// Capacity of one original buffer row.
    BufferCapacity,
    /// Capacity of live activations.
    ActivationCapacity,
}

impl TargetPremise {
    fn binding(self) -> &'static str {
        match self {
            Self::Counter => "counter",
            Self::CounterWidth => "counter_width_bits",
            Self::TickUnit => "tick_unit",
            Self::MaxReadingGap => "max_reading_gap",
            Self::MaxEventRate => "max_event_rate",
            Self::Uncertainty => "uncertainty",
            Self::BufferCapacity => "buffers",
            Self::ActivationCapacity => "activation_capacity",
        }
    }
}

/// Owned diagnostic location, not a plan identity or admission capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorRefusalLocus {
    /// Original graph-local selector supplied by the caller.
    pub node: CheckedNodeId,
    /// Original content key resolved by the selected package.
    pub structural_key: CheckedNodeId,
    /// Native child path, ending at the selected term.
    pub body_path: Vec<usize>,
    /// Original occurrence, absent when the caller selected none.
    pub occurrence: Option<CheckedOccurrence>,
}

/// Diagnostic for a target inequality already decided by its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetMismatch {
    /// The independently compared target field.
    pub premise: TargetPremise,
    /// Location of that field's actual admitted value.
    pub locus: MonitorRefusalLocus,
}

/// Diagnostic for a support-policy decision already made by its caller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedOperation {
    /// Original node and inline term location.
    pub locus: MonitorRefusalLocus,
    /// Admitted containing-node kind already resolved by the caller.
    pub kind: CheckedNodeKind,
    /// Entire original application record, when the selected term is an application.
    pub operation: Option<CheckedApplicationOperation>,
}

/// Stable reasons a diagnostic location could not be constructed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MonitorLocusErrorCode {
    /// The node is not resolved by the selected package.
    UnknownNode,
    /// A resolved node is not an admitted monitor root.
    NotMonitorRoot,
    /// A requested child or named binding does not exist.
    InvalidBodyPath,
    /// The supplied occurrence does not belong to the original node.
    OccurrenceMismatch,
    /// A buffer index is absent, excessive or supplied for another premise.
    InvalidPremiseIndex,
}

/// Typed diagnostic construction failure, separate from preparation policy.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("monitor diagnostic location {code:?} at {node:?}, path {body_path:?}")]
pub struct MonitorLocusError {
    /// Stable construction failure.
    pub code: MonitorLocusErrorCode,
    /// Original supplied node selector.
    pub node: CheckedNodeId,
    /// Original path at which construction failed.
    pub body_path: Vec<usize>,
}

fn failure(code: MonitorLocusErrorCode, node: &CheckedNodeId, path: &[usize]) -> MonitorLocusError {
    MonitorLocusError {
        code,
        node: node.clone(),
        body_path: path.to_vec(),
    }
}

fn term_at<'a>(
    package: &'a CheckedPackageV2,
    node: &CheckedNodeId,
    path: &[usize],
) -> Result<&'a CheckedTerm, MonitorLocusError> {
    let mut term = package
        .term(node)
        .ok_or_else(|| failure(MonitorLocusErrorCode::UnknownNode, node, path))?;
    for index in path {
        term = match term {
            CheckedTerm::Aggregate { members } => members.get(*index),
            CheckedTerm::Application { arguments, .. } => arguments.get(*index),
            CheckedTerm::Binding { value, .. } if *index == 0 => Some(value.as_ref()),
            _ => None,
        }
        .ok_or_else(|| failure(MonitorLocusErrorCode::InvalidBodyPath, node, path))?;
    }
    Ok(term)
}

fn locus(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
    path: &[usize],
    occurrence: Option<&CheckedOccurrence>,
) -> Result<MonitorRefusalLocus, MonitorLocusError> {
    let key = package
        .structural_key(node)
        .ok_or_else(|| failure(MonitorLocusErrorCode::UnknownNode, node, path))?;
    if let Some(occurrence) = occurrence {
        // Only occurrence membership needs this original node record. No
        // graph index, support classifier or replacement source location is built.
        let belongs =
            package.resolved_graph().nodes.iter().any(|original| {
                &original.node_id == node && original.occurrences.contains(occurrence)
            });
        if !belongs {
            return Err(failure(
                MonitorLocusErrorCode::OccurrenceMismatch,
                node,
                path,
            ));
        }
    }
    Ok(MonitorRefusalLocus {
        node: node.clone(),
        structural_key: key.clone(),
        body_path: path.to_vec(),
        occurrence: occurrence.cloned(),
    })
}

fn named_value<'a>(term: &'a CheckedTerm, name: &str) -> Option<(usize, &'a CheckedTerm)> {
    let CheckedTerm::Aggregate { members } = term else {
        return None;
    };
    members
        .iter()
        .enumerate()
        .find_map(|(index, member)| match member {
            CheckedTerm::Binding {
                name: actual,
                value,
            } if actual.as_ref() == name => Some((index, value.as_ref())),
            _ => None,
        })
}

/// Locate the actual admitted target value after the caller decides inequality.
///
/// Every path includes the final binding child zero. Only `BufferCapacity`
/// accepts a buffer index. This function neither compares targets nor initializes a program.
pub fn target_mismatch_at(
    package: &CheckedPackageV2,
    root: &CheckedNodeId,
    premise: TargetPremise,
    buffer_index: Option<usize>,
    occurrence: Option<&CheckedOccurrence>,
) -> Result<TargetMismatch, MonitorLocusError> {
    let body = term_at(package, root, &[])?;
    if !package.monitor_plans().any(|plan| plan.node_id() == root) {
        return Err(failure(MonitorLocusErrorCode::NotMonitorRoot, root, &[]));
    }
    if (premise == TargetPremise::BufferCapacity) != buffer_index.is_some() {
        return Err(failure(
            MonitorLocusErrorCode::InvalidPremiseIndex,
            root,
            &[],
        ));
    }
    let (slot, value) = named_value(body, premise.binding())
        .ok_or_else(|| failure(MonitorLocusErrorCode::InvalidBodyPath, root, &[]))?;
    let mut path = vec![slot, 0];
    if let Some(index) = buffer_index {
        let CheckedTerm::Aggregate { members } = value else {
            return Err(failure(MonitorLocusErrorCode::InvalidBodyPath, root, &path));
        };
        let row = members
            .get(index)
            .ok_or_else(|| failure(MonitorLocusErrorCode::InvalidPremiseIndex, root, &path))?;
        let (capacity, _) = named_value(row, "capacity")
            .ok_or_else(|| failure(MonitorLocusErrorCode::InvalidBodyPath, root, &path))?;
        path.extend([index, capacity, 0]);
    }
    term_at(package, root, &path)?;
    Ok(TargetMismatch {
        premise,
        locus: locus(package, root, &path, occurrence)?,
    })
}

/// Retain the actual typed subterm after the caller decides it is unsupported.
///
/// `kind` is the containing node's already resolved admitted kind; it is
/// retained as diagnostic context and is not an admission capability.
/// Reference terms terminate a path; this function never follows their targets.
pub fn unsupported_at(
    package: &CheckedPackageV2,
    node: &CheckedNodeId,
    body_path: &[usize],
    kind: CheckedNodeKind,
    occurrence: Option<&CheckedOccurrence>,
) -> Result<UnsupportedOperation, MonitorLocusError> {
    let term = term_at(package, node, body_path)?;
    let operation = match term {
        CheckedTerm::Application { operation, .. } => Some(operation.clone()),
        _ => None,
    };
    Ok(UnsupportedOperation {
        locus: locus(package, node, body_path, occurrence)?,
        kind,
        operation,
    })
}
