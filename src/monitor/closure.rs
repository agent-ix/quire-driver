// SPDX-License-Identifier: AGPL-3.0-or-later
//! Phases P1 and P3 to P6 of monitor preparation: admit the plan authority and
//! read the selected type closure from one final emission.
//!
//! One fresh native join budget, sized from the caller's existing preparation
//! limits, pays QSL authority admission, the QSL row adapters and IR's closure
//! read. Nothing here is an executable program.

use qsl_analyze::monitor_root::{
    AdmittedPlanAuthority, PlanAuthoritySelection, PlanAuthorityValidationCaps, PlanRoot,
};
use qsl_package::{ReadEqualityRows, ReadIdentityRows};
use quire_contract_model::{
    CheckedOccurrence, CheckedOccurrenceRole, NativeJoinBudget, NativeJoinLocus,
    NativeJoinLocusSource, SelectedModelEnumShapeRow, SelectedMonitorAuthorityRow,
    SelectedMonitorClosureRows, SelectedMonitorTimeSource, SelectedMonitorTypeClosure,
    SelectedSourceCollectionRow, SelectedSourceEnumRow,
};
use quire_exact::Cancel;
use quire_observation::authority::progress;

use super::{MonitorPreparationError as Error, MonitorPreparationLimits};

/// Owned results of the QSL phases (authority, identity rows, row adapters).
///
/// The IR closure borrows from these, so it is produced by [`Self::read_closure`]
/// inside a scope rather than stored next to them.
pub struct MonitorClosureRows {
    authority: AdmittedPlanAuthority,
    budget: NativeJoinBudget,
    identities: ReadIdentityRows,
    equality: ReadEqualityRows,
    source_enums: Vec<SelectedSourceEnumRow>,
    model_enum_shapes: Vec<SelectedModelEnumShapeRow>,
    source_collections: Vec<SelectedSourceCollectionRow>,
}

fn poll(cancel: &Cancel) -> Result<(), Error> {
    cancel.poll();
    match cancel.cause() {
        Some(cause) => Err(Error::Cancelled(cause)),
        None => Ok(()),
    }
}

/// Admit the plan authority and collect the closure rows (P1, P3, P4).
///
/// The strict read of the root's own final emission happens once, inside QSL's
/// `admit_authority`; the driver makes none of its own.
pub fn collect_monitor_closure_rows(
    plan_root: PlanRoot,
    selection: PlanAuthoritySelection<'_>,
    progress: Option<progress::View>,
    caps: PlanAuthorityValidationCaps,
    limits: MonitorPreparationLimits,
    cancel: &Cancel,
) -> Result<MonitorClosureRows, Error> {
    let mut budget =
        NativeJoinBudget::new(limits.work, limits.storage).with_read_storage_limit(limits.storage);
    poll(cancel)?;
    let authority = plan_root
        .admit_authority(
            selection,
            progress,
            caps,
            &mut budget,
            NativeJoinLocus {
                source: NativeJoinLocusSource::PreparationSelection(0),
                form: "plan_authority",
            },
        )
        .map_err(|error| Error::Authority(Box::new(error)))?;
    poll(cancel)?;
    let package = authority.admitted_package();
    let emission = authority.root().emission();
    let identities = emission
        .identity_rows(package, &mut budget)
        .map_err(|error| Error::IdentityRows(Box::new(error)))?;
    poll(cancel)?;
    let equality = emission.equality_witness(package, &mut budget)?;
    poll(cancel)?;
    let source_enums = emission.selected_source_enum_rows(&mut budget)?;
    poll(cancel)?;
    let model_enum_shapes = emission.selected_ir_model_enum_shape_rows(&mut budget)?;
    let mut source_collections = Vec::new();
    for (index, row) in emission
        .selected_source_collection_rows()
        .iter()
        .enumerate()
    {
        poll(cancel)?;
        source_collections.push(row.to_ir(
            &mut budget,
            NativeJoinLocus {
                source: NativeJoinLocusSource::AdapterRow(index),
                form: "monitor_source_collection_row",
            },
        )?);
    }
    Ok(MonitorClosureRows {
        authority,
        budget,
        identities,
        equality,
        source_enums,
        model_enum_shapes,
        source_collections,
    })
}

impl MonitorClosureRows {
    /// The admitted authority these rows were read from.
    pub fn authority(&self) -> &AdmittedPlanAuthority {
        &self.authority
    }

    /// The preparation budget after the phases run so far.
    pub fn budget(&self) -> &NativeJoinBudget {
        &self.budget
    }

    /// Build the authority row (P5) and read the closure (P6), then hand the
    /// closure to `consume` while the borrowed rows are alive.
    pub fn read_closure<R>(
        &mut self,
        cancel: &Cancel,
        consume: impl FnOnce(&AdmittedPlanAuthority, &SelectedMonitorTypeClosure<'_>) -> R,
    ) -> Result<R, Error> {
        poll(cancel)?;
        let authority = &self.authority;
        let package = authority.admitted_package();
        let mut plans = package.monitor_plans();
        let (Some(plan), 1) = (plans.next(), 1 + plans.len()) else {
            return Err(Error::MonitorRootCount(package.monitor_plans().len()));
        };
        let emission = authority.root().emission();
        let time_source = match authority.checked_tick_period() {
            Some(period) => SelectedMonitorTimeSource::Tick {
                period: period.clone(),
            },
            None => SelectedMonitorTimeSource::Dense,
        };
        let row = SelectedMonitorAuthorityRow::try_new(
            authority.admission_binding(),
            plan.clause().clone(),
            CheckedOccurrence {
                role: CheckedOccurrenceRole::Claim,
                ordinal: authority
                    .root()
                    .plan()
                    .subject()
                    .occurrence()
                    .origin()
                    .ordinal(),
            },
            time_source,
            authority.admitted_order(),
            authority.progress_context(),
        )
        .map_err(Error::AuthorityRow)?;
        poll(cancel)?;
        let closure = plan
            .project_selected_type_closure(
                SelectedMonitorClosureRows {
                    declarations: self.identities.declarations(),
                    fields: self.identities.fields(),
                    model_tuples: emission.selected_tuple_declaration_rows(),
                    source_units: emission.selected_source_unit_rows(),
                    model_enum_declarations: emission.selected_model_enum_declaration_rows(),
                    model_enum_members: emission.selected_model_enum_member_rows(),
                    model_enum_shapes: &self.model_enum_shapes,
                    source_records: self.identities.source_records(),
                    source_tuples: emission.selected_source_tuple_rows(),
                    source_unions: self.identities.source_unions(),
                    source_enums: &self.source_enums,
                    source_collections: &self.source_collections,
                    equality_applications: self.equality.applications(),
                    equality_facts: self.equality.facts(),
                    authority: &row,
                },
                &mut self.budget,
            )
            .map_err(|error| Error::Closure(Box::new(error)))?;
        Ok(consume(authority, &closure))
    }
}
