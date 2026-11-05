// SPDX-License-Identifier: AGPL-3.0-or-later
//! The actual bounded initializer type read, without an environment substitute.

use quire_contract_model::checked_package::{CheckedTerm, ExactTypeProjection, SelectedExactIdentities};
use quire_exact::Cancel;

use super::budget::Budget;
use super::{AdmittedMonitorSelection, MonitorPreparationError as Error};

/// Read the initializer expression's native type using the original witnessed joins.
///
/// This is a partial preparation read, not complete source-closure lowering.
/// In particular its success does not authorize a `TypeEnvironment` or runtime
/// program. Native unavailable-witness and unsupported-type errors remain
/// upstream projection errors, including their original node, cause and work.
pub fn project_monitor_initializer(
    selection: &AdmittedMonitorSelection<'_>, cancel: &Cancel,
) -> Result<ExactTypeProjection, Error> {
    let request = selection.request;
    let mut budget = Budget::new(request.limits, cancel);
    budget.charge(selection.work)?;
    for row in request.witnesses.declarations {
        let before = budget.work();
        budget.inspect_debug(row)?;
        budget.reserve(usize::try_from(budget.work() - before).unwrap_or(usize::MAX))?;
    }
    for row in request.witnesses.fields {
        let before = budget.work();
        budget.inspect_debug(row)?;
        budget.reserve(usize::try_from(budget.work() - before).unwrap_or(usize::MAX))?;
    }
    let identities = SelectedExactIdentities::from_stage_rows(
        request.package, request.witnesses.declarations.iter().cloned(),
        request.witnesses.fields.iter().cloned(),
    )?;
    let init = selection.plan.init();
    budget.charge(1)?;
    let invalid = || Error::InvalidAdmittedEntry { node: Box::new(init.node_id().clone()) };
    let CheckedTerm::Aggregate { members } = init.body() else { return Err(invalid()); };
    let mut body = None;
    for member in members {
        budget.charge(1)?;
        if let CheckedTerm::Binding { name, value } = member {
            if name.as_ref() == "body" { body = Some(value.as_ref()); break; }
        }
    }
    // Strict monitor admission requires an original expression-node reference
    // here. Project that expression, not the entry's function type or a type
    // node's metatype (which could silently erase a bounded-domain type).
    let Some(CheckedTerm::Reference { target }) = body else { return Err(invalid()); };
    budget.inspect(target)?;
    request.package.project_exact_type(target, &identities, budget.remaining()).map_err(Error::Projection)
}
