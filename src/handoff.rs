// SPDX-License-Identifier: AGPL-3.0-or-later
//! E5 (ADR-011 §2.1): the E4 v2 bytes handed to IR's checked-package reader,
//! The complete E4
//! value remains available through admission for bounded diagnostic sources.

use quire_contract_model::{
    CheckedPackageDispatchResult, CheckedPackageReadLimits, CheckedPackageV2,
};

use qsl_replay::spine::{admission_evidence, AdmittedModels, CheckedUnit, EmittedUnit};

use crate::{DriverFault, DriverRefusal, Upstream};

/// Read `emitted` through IR's reader.
pub(crate) fn handoff(
    upstream: &impl Upstream,
    models: &AdmittedModels,
    checked: &CheckedUnit,
    emitted: &EmittedUnit,
    imports: qsl_package::V2ReadLimits,
    emit: qsl_package::EmitLimits,
) -> Result<CheckedPackageV2, DriverRefusal> {
    let evidence = admission_evidence(models, checked, emitted, imports, emit)
        .map_err(DriverRefusal::QslAdmissionEvidence)?;
    let package =
        match upstream.read_package(emitted, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageDispatchResult::AdmittedV2(package) => *package,
            CheckedPackageDispatchResult::Refused(refusal) => {
                return Err(DriverRefusal::Fault(DriverFault::HandoffRefused(Box::new(
                    refusal,
                ))))
            }
            CheckedPackageDispatchResult::CatalogUnavailable(native) => {
                return Err(DriverRefusal::HandoffCatalogUnavailable(native));
            }
            CheckedPackageDispatchResult::Incomplete(incomplete) => {
                return Err(DriverRefusal::HandoffLimit(incomplete))
            }
        };
    Ok(package)
}
