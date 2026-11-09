// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete monitor preparation: a genuine QSL plan root becomes a checked
//! runtime `Program` (FR-019).

use std::rc::Rc;

use qsl_analyze::monitor_root::{PlanAuthoritySelection, PlanAuthorityValidationCaps, PlanRoot};
use quire_contract_model::CheckedNodeId;
use quire_contract_runtime::monitor::{Program, ProgramParts};
use quire_exact::Cancel;
use quire_observation::authority::progress;

use super::{
    build_type_environment, collect_monitor_closure_rows, derive_layout, lowering::lower_entries,
    MonitorPreparationError as Error, MonitorPreparationLimits,
};

/// Prepare the one monitor of `plan_root` as a checked runtime program.
///
/// `expected_time_dimension` is the independently resolved base Time
/// dimension; a plan whose tick unit differs is a `target_mismatch`. A
/// refusal here is a typed preparation failure, never a temporal verdict.
pub fn prepare_monitor(
    plan_root: PlanRoot,
    selection: PlanAuthoritySelection<'_>,
    progress: Option<progress::View>,
    caps: PlanAuthorityValidationCaps,
    expected_time_dimension: &CheckedNodeId,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<Rc<Program>, Error> {
    let mut rows =
        collect_monitor_closure_rows(plan_root, selection, progress, caps, limits, cancel)?;
    rows.read_closure(cancel, |authority, closure, facts| {
        let package = authority.admitted_package();
        let types = build_type_environment(closure, facts, limits, cancel)?;
        let bodies = lower_entries(package, closure, facts, limits, cancel)?;
        let layout = derive_layout(
            package,
            authority,
            closure,
            bodies.entries,
            expected_time_dimension,
            limits,
            cancel,
        )?;
        Program::try_new(ProgramParts {
            layout,
            types,
            functions: bodies.functions,
            expressions: bodies.expressions,
        })
        .map_err(Error::from)
    })?
}
