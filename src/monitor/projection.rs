// SPDX-License-Identifier: AGPL-3.0-or-later
//! The actual bounded initializer type read, without an environment substitute.

use quire_contract_model::{CheckedTerm, ExactTypeProjection, SelectedExactIdentities};
use quire_exact::Cancel;

use super::budget::Budget;
use super::{AdmittedMonitorSelection, MonitorPreparationError as Error};

/// Read the initializer expression's native type using identities admitted before this read.
///
/// The caller supplies an existing native identity set from the same acquisition.
/// Constructing/joining that set is outside this operation's work allowance; this
/// function does not call the unbounded `from_stage_rows` constructor. It is not
/// an authenticated complete-closure entry. Native projection has no cancellation
/// argument: cancellation is polled before it and before a successful result is exposed.
///
/// This is a partial preparation read, not complete source-closure lowering.
/// In particular its success does not authorize a `TypeEnvironment` or runtime
/// program. Native unavailable-witness and unsupported-type errors remain
/// upstream projection errors, including their original node, cause and work.
pub fn project_monitor_initializer(
    selection: &AdmittedMonitorSelection<'_>,
    identities: &SelectedExactIdentities,
    cancel: &Cancel,
) -> Result<ExactTypeProjection, Error> {
    let request = selection.request;
    let mut budget = Budget::new(request.limits, cancel);
    budget.charge(selection.work)?;
    let init = selection.plan.init();
    budget.charge(1)?;
    let invalid = || Error::InvalidAdmittedEntry {
        node: Box::new(init.node_id().clone()),
    };
    let CheckedTerm::Aggregate { members } = init.body() else {
        return Err(invalid());
    };
    let mut body = None;
    for member in members {
        budget.charge(1)?;
        if let CheckedTerm::Binding { name, value } = member {
            if name.as_ref() == "body" {
                body = Some(value.as_ref());
                break;
            }
        }
    }
    // Strict monitor admission requires an original expression-node reference
    // here. Project that expression, not the entry's function type or a type
    // node's metatype (which could silently erase a bounded-domain type).
    let Some(CheckedTerm::Reference { target }) = body else {
        return Err(invalid());
    };
    budget.inspect(target)?;
    let projected = request
        .package
        .project_exact_type(target, identities, budget.remaining())?;
    budget.charge(projected.work())?;
    Ok(projected)
}
