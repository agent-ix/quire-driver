// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bind an original QSL subject through its owner's final declaration lookup.

use qsl_package::Emission;
use qsl_replay::MonitorSubject;
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedOccurrenceRole, CheckedPackageDispatchResult,
    CheckedPackageEvidence, CheckedPackageReadLimits,
};
use quire_exact::Cancel;

use super::{
    budget::Budget, selection, AdmittedMonitorClockSelection, AdmittedMonitorSelection,
    MonitorPreparationError as Error, MonitorPreparationRequest, MonitorSelectionField as Field,
};

/// An original native subject joined to one final immutable admitted selection.
///
/// This establishes final occurrence selection, not complete plan identity,
/// executable closure or a clock configuration. Native acquisition, equality
/// and the owner's linear lookup are outside the driver work/storage ledger.
pub struct AdmittedMonitorFinalSubject<'a> {
    selection: AdmittedMonitorSelection<'a>,
    subject: &'a MonitorSubject,
    emission: &'a Emission,
    declaration: &'a CheckedNodeId,
}

impl<'a> AdmittedMonitorFinalSubject<'a> {
    /// Verified partial selection and its actual driver work prefix.
    pub fn selection(&self) -> &AdmittedMonitorSelection<'a> {
        &self.selection
    }
    /// Original native subject supplied by the producer's plan.
    pub fn subject(&self) -> &'a MonitorSubject {
        self.subject
    }
    /// Exact native emission whose package was strictly authenticated.
    pub fn emission(&self) -> &'a Emission {
        self.emission
    }
    /// Final declaration borrowed directly from the owner's lookup result.
    pub fn declaration(&self) -> &'a CheckedNodeId {
        self.declaration
    }
    /// Continue native clock admission without resetting the driver work prefix.
    ///
    /// Current Timed emissions read their native model-time source.
    /// Mapping success never supplies or derives a clock configuration.
    pub fn admit_clock(&self, cancel: &Cancel) -> Result<AdmittedMonitorClockSelection<'a>, Error> {
        selection::resume_clock(&self.selection, cancel)
    }
}

fn mismatch(field: Field, node: &CheckedNodeId) -> Error {
    Error::Selection {
        field,
        node: Box::new(node.clone()),
    }
}

/// Authenticate and bind an original subject to its exact final clause Claim.
///
/// Native read limits govern synchronous acquisition; native full-package
/// equality and linear declaration lookup have no driver work receipt. The
/// driver polls cancellation around that acquisition and charges its own
/// identity/name inspections and partial selection to the original allowance.
/// This entry accepts no source text, replacement plan or inferred ordinal.
pub fn admit_monitor_final_subject<'a>(
    request: &'a MonitorPreparationRequest<'a>,
    subject: &'a MonitorSubject,
    emission: &'a Emission,
    evidence: &CheckedPackageEvidence,
    read_limits: CheckedPackageReadLimits,
    cancel: &Cancel,
) -> Result<AdmittedMonitorFinalSubject<'a>, Error> {
    let mut budget = Budget::new(request.limits, cancel);
    budget.charge(0)?;
    let admitted = match read_checked_package(emission.package().bytes(), read_limits, evidence) {
        CheckedPackageDispatchResult::AdmittedV2(package) => package,
        CheckedPackageDispatchResult::Refused(native) => {
            return Err(Error::EmissionRefused(Box::new(native)))
        }
        CheckedPackageDispatchResult::Incomplete(native) => {
            return Err(Error::EmissionIncomplete(Box::new(native)))
        }
    };
    budget.charge(0)?;
    let same_package = admitted.as_ref() == request.package;
    drop(admitted);
    budget.charge(0)?;
    if !same_package {
        return Err(mismatch(Field::EmissionPackage, request.clause));
    }
    budget.inspect(subject.package().as_str())?;
    budget.inspect(emission.package().name())?;
    if subject.package().as_str() != emission.package().name() {
        return Err(mismatch(Field::ProducerSubject, request.clause));
    }
    budget.inspect(subject.occurrence().node())?;
    budget.inspect(subject.clause_node())?;
    if subject.occurrence().node() != subject.clause_node() {
        return Err(mismatch(Field::ProducerSubject, request.clause));
    }
    budget.charge(1)?; // native origin record
    budget.inspect(subject.occurrence().origin().role().as_str())?;
    budget.charge(8)?; // native origin ordinal, before conversion
    let Some(occurrence) = qsl_package::checked_occurrence(subject.occurrence().origin()) else {
        return Err(mismatch(Field::ProducerSubject, request.clause));
    };
    if occurrence.role != CheckedOccurrenceRole::Claim {
        return Err(mismatch(Field::ProducerSubject, request.clause));
    }
    // This owner lookup is linear and exposes no work receipt. In particular,
    // its len() counts clock rows and cannot meter declaration selection.
    let declaration = emission
        .temporal_binding_rows()
        .declaration(subject.clause_node(), occurrence.ordinal)
        .ok_or_else(|| Error::MissingFinalSubject(Box::new(subject.clone())))?;
    budget.charge(0)?;
    budget.inspect(declaration)?;
    budget.inspect(request.clause)?;
    if declaration != request.clause {
        return Err(mismatch(Field::FinalDeclaration, declaration));
    }
    budget.inspect(request.occurrence)?;
    if request.occurrence.node_id != *declaration
        || request.occurrence.role != occurrence.role
        || request.occurrence.ordinal != occurrence.ordinal
    {
        return Err(mismatch(Field::FinalOccurrence, declaration));
    }
    budget.inspect(subject.clause_name())?;
    let mut matched_name = false;
    for name in request.package.temporal_clause_names().unwrap_or_default() {
        budget.inspect(&name.declaration)?;
        budget.charge(8)?;
        budget.inspect(name.name.as_ref())?;
        if name.declaration == *declaration && name.occurrence == occurrence.ordinal {
            if name.name.as_ref() != subject.clause_name() {
                return Err(mismatch(Field::ClaimName, declaration));
            }
            matched_name = true;
            break;
        }
    }
    if !matched_name {
        return Err(mismatch(Field::ClaimName, declaration));
    }
    let selection = selection::select_with_budget(request, &mut budget)?;
    Ok(AdmittedMonitorFinalSubject {
        selection,
        subject,
        emission,
        declaration,
    })
}
