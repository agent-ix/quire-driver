// SPDX-License-Identifier: AGPL-3.0-or-later
//! Borrowed independent selections and original witnesses for monitor preparation.

use quire_contract_model::checked_package::{
    CheckedNodeId, CheckedPackageLockV2, CheckedPackageV2, CheckedRational,
    CheckedSelection, CheckedSourceMapEntry, SelectedFactRows, TemporalBinding,
};

/// A package-instance-bound root selector captured before preparation.
///
/// Capturing a selector makes no admission claim and copies no premises from
/// the plan. The request must independently supply all expected values.
#[derive(Clone, Copy)]
pub struct MonitorSelection<'a> {
    pub(super) package: &'a CheckedPackageV2,
    pub(super) root: &'a CheckedNodeId,
}

impl<'a> MonitorSelection<'a> {
    /// Capture an exact package instance and its independently chosen root ID.
    pub fn new(package: &'a CheckedPackageV2, root: &'a CheckedNodeId) -> Self {
        Self { package, root }
    }
    /// The independently captured root selector, without inferring any premise.
    pub fn root(&self) -> &'a CheckedNodeId { self.root }
}

/// Caller operating allowances, separate from the plan's semantic capacities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonitorPreparationLimits {
    /// Cumulative inspected identity/value material and native projection work.
    pub work: u64,
    /// Cumulative temporary bytes reserved by preparation, excluding input storage.
    pub storage: u64,
}

/// Independent inputs for the implemented selection and projection phase.
///
/// This request does not assert complete generated-source declaration closure.
/// The supplied rows must come from one final producer emission. IR validates
/// their admitted joins; caller-created Exact values establish no authority.
/// Missing source-record/union and association handoffs cannot be replaced by
/// this request, a caller-built environment, or a callback.
pub struct MonitorPreparationRequest<'a> {
    /// Package being prepared; it must be the captured selection's exact instance.
    pub package: &'a CheckedPackageV2,
    /// Independently captured package/root selection.
    pub selection: MonitorSelection<'a>,
    /// Independently selected original immutable package lock.
    pub package_lock: &'a CheckedPackageLockV2,
    /// Expected complete root content key.
    pub root_key: &'a CheckedNodeId,
    /// Expected init, event, watermark and gap content keys, in that order.
    pub entry_keys: [&'a CheckedNodeId; 4],
    /// Independently selected clause ID.
    pub clause: &'a CheckedNodeId,
    /// Independently selected original `over` parameter of that clause.
    pub subject: &'a CheckedNodeId,
    /// Full authentic source-map occurrence of that clause.
    pub occurrence: &'a CheckedSourceMapEntry,
    /// Independently selected profile; the selected package must retain it.
    pub profile: &'a CheckedSelection,
    /// Independently selected canonical clock row of this exact clause occurrence.
    pub clock_binding: &'a TemporalBinding,
    /// Exact independently selected hardware counter name.
    pub counter: &'a str,
    /// Exact independently selected tick-unit ID.
    pub tick_unit: &'a CheckedNodeId,
    /// Independently selected hardware counter width.
    pub counter_width_bits: u8,
    /// Independently selected maximum modular reading gap.
    pub max_reading_gap: u64,
    /// Independently selected exact event rate.
    pub max_event_rate: &'a CheckedRational,
    /// Independently selected exact uncertainty.
    pub uncertainty: &'a CheckedRational,
    /// Expected subformula IDs and exact capacities, in admitted order.
    pub buffers: &'a [(CheckedNodeId, Box<str>)],
    /// Independently selected exact activation capacity, including zero.
    pub activation_capacity: &'a str,
    /// Existing original final-emission model and equality witness channels.
    pub witnesses: SelectedFactRows<'a>,
    /// Explicit operating work/storage allowances.
    pub limits: MonitorPreparationLimits,
}

impl<'a> MonitorPreparationRequest<'a> {
    /// The original independently selected root, before package resolution.
    pub fn selection_root(&self) -> &'a CheckedNodeId { self.selection.root }
}
