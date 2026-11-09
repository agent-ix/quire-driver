// SPDX-License-Identifier: AGPL-3.0-or-later
//! The runtime `Layout` of a prepared program, derived only from the selected
//! closure and the admitted plan.

use qsl_analyze::monitor_root::AdmittedPlanAuthority;
use quire_contract_model::{CheckedNodeId, CheckedPackageV2, SelectedMonitorTypeClosure};
use quire_contract_runtime::monitor::{Buffer, Entries, Layout, TypeBinding};
use quire_exact::{Cancel, Integer};

use super::{
    lowering::node_key, types::rational, MonitorPreparationError as Error, MonitorPreparationLimits,
};
use crate::monitor_refusal::{target_mismatch_at, TargetPremise};

fn binding(
    package: &CheckedPackageV2,
    closure: &SelectedMonitorTypeClosure<'_>,
    node: &CheckedNodeId,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<TypeBinding, Error> {
    let projection = closure
        .exact_types()
        .iter()
        .find(|projection| projection.node() == node)
        .ok_or_else(|| Error::InvalidAdmittedEntry {
            node: Box::new(node.clone()),
        })?;
    let original = package
        .structural_key(node)
        .ok_or_else(|| Error::InvalidAdmittedEntry {
            node: Box::new(node.clone()),
        })?;
    Ok(TypeBinding {
        key: node_key(original, limits, cancel)?,
        value_type: projection.value_type().clone(),
    })
}

fn integer(text: &str) -> Result<Integer, Error> {
    text.parse::<Integer>().map_err(|_| Error::Rational)
}

/// Derive the `Layout` for the closure's plan.
///
/// `expected_time_dimension` is the independently resolved base Time
/// dimension; a tick unit of any other dimension is a target mismatch.
pub fn derive_layout(
    package: &CheckedPackageV2,
    authority: &AdmittedPlanAuthority,
    closure: &SelectedMonitorTypeClosure<'_>,
    entries: Entries,
    expected_time_dimension: &CheckedNodeId,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<Layout, Error> {
    let plan = closure.plan();
    let context = closure.layout_context();
    let dimension = closure.tick_unit_dimension();
    if dimension != expected_time_dimension {
        let mismatch =
            target_mismatch_at(package, plan.node_id(), TargetPremise::TickUnit, None, None)
                .map_err(Error::Internal)?;
        return Err(Error::TargetMismatch(Box::new(mismatch)));
    }
    let state = binding(package, closure, plan.state(), limits, cancel)?;
    let results = binding(package, closure, plan.results(), limits, cancel)?;
    let step_result = binding(
        package,
        closure,
        plan.on_event().result_type(),
        limits,
        cancel,
    )?;
    for entry in [plan.on_watermark(), plan.on_gap()] {
        if entry.result_type() != plan.on_event().result_type() {
            return Err(Error::InvalidAdmittedEntry {
                node: Box::new(entry.node_id().clone()),
            });
        }
    }
    if plan.init().result_type() != plan.state() {
        return Err(Error::InvalidAdmittedEntry {
            node: Box::new(plan.init().node_id().clone()),
        });
    }
    let mut buffers = Vec::with_capacity(plan.buffers().len());
    for buffer in plan.buffers() {
        buffers.push(Buffer {
            owner: node_key(
                package.structural_key(buffer.subformula()).ok_or_else(|| {
                    Error::InvalidAdmittedEntry {
                        node: Box::new(buffer.subformula().clone()),
                    }
                })?,
                limits,
                cancel,
            )?,
            capacity: integer(buffer.capacity())?,
        });
    }
    Ok(Layout {
        root: node_key(context.root_key(), limits, cancel)?,
        clause: node_key(context.clause_key(), limits, cancel)?,
        package_lock: context.package_lock().into(),
        subject: context.subject().to_owned(),
        occurrence: context.occurrence().into(),
        profile: context.profile().to_owned(),
        clock_binding: context.clock_binding().into(),
        admitted_order: authority.admitted_order().into(),
        progress_context: authority.progress_context().into(),
        state,
        inputs: binding(package, closure, plan.inputs(), limits, cancel)?,
        scope: binding(package, closure, plan.scope(), limits, cancel)?,
        results,
        step_result,
        entries,
        counter: plan.counter().to_owned(),
        tick_unit: node_key(plan.tick_unit(), limits, cancel)?,
        time_dimension: node_key(dimension, limits, cancel)?,
        counter_width_bits: plan.counter_width_bits(),
        max_reading_gap: plan.max_reading_gap(),
        max_event_rate: rational(plan.max_event_rate())?,
        uncertainty: rational(plan.uncertainty())?,
        buffers,
        activation_capacity: integer(plan.activation_capacity())?,
    })
}
