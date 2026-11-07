// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read the atomic selected closure from one authenticated final emission.

use quire_contract_model::{
    NativeJoinBudget, SelectedMonitorClosureRows, SelectedMonitorTypeClosure,
};
use quire_exact::Cancel;

use super::{AdmittedMonitorFinalSubject, MonitorPreparationError as Error};

/// Read all reached source, model, clock, Exact, and equality facts for one
/// final subject through IR's native atomic reader.
///
/// The caller retains one finite native meter across the QSL identity join,
/// this closure, and any following native projection. The returned closure
/// borrows only the authenticated admitted package. It is not an RT Program.
pub fn read_monitor_type_closure<'a>(
    admitted: &'a AdmittedMonitorFinalSubject<'a>,
    budget: &mut NativeJoinBudget,
    cancel: &Cancel,
) -> Result<SelectedMonitorTypeClosure<'a>, Error> {
    cancel.poll();
    if let Some(cause) = cancel.cause() {
        return Err(Error::Cancelled(cause));
    }
    let emission = admitted.emission();
    let package = admitted.selection().package();
    let identities = emission
        .identity_rows(package, budget)
        .map_err(|error| Error::ClosureRows(Box::new(error)))?;
    let equality = emission.equality_witness(package);
    let source_enums = emission.selected_source_enum_rows();
    cancel.poll();
    if let Some(cause) = cancel.cause() {
        return Err(Error::Cancelled(cause));
    }
    admitted
        .selection()
        .plan()
        .project_selected_type_closure(
            SelectedMonitorClosureRows {
                declarations: identities.declarations(),
                fields: identities.fields(),
                model_tuples: emission.selected_tuple_declaration_rows(),
                source_units: emission.selected_source_unit_rows(),
                model_enum_declarations: emission.selected_model_enum_declaration_rows(),
                model_enum_members: emission.selected_model_enum_member_rows(),
                model_enum_shapes: emission.selected_model_enum_shape_rows(),
                source_records: identities.source_records(),
                source_tuples: emission.selected_source_tuple_rows(),
                source_unions: identities.source_unions(),
                source_enums: &source_enums,
                source_collections: emission.selected_source_collection_rows(),
                equality_applications: equality.applications(),
                equality_facts: equality.facts(),
            },
            budget,
        )
        .map_err(|error| Error::ClosureRead(Box::new(error)))
}
