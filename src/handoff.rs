// SPDX-License-Identifier: AGPL-3.0-or-later
//! E5 (ADR-011 §2.1): the E4 v2 bytes handed to IR's checked-package reader,
//! admitted only at the `package_id` E4 emitted them under. The complete E4
//! value remains available through admission for bounded diagnostic sources.

use quire_contract_model::{
    CheckedPackageDispatchResult, CheckedPackageReadLimits, CheckedPackageV2,
};

use qsl_replay::spine::{admission_evidence, AdmittedModels, CheckedUnit, EmittedUnit};

use crate::{DriverFault, DriverRefusal, Upstream};

/// Read `emitted` through IR's reader and check the admitted `package_id`
/// is the one E4 minted.
pub(crate) fn handoff(
    upstream: &impl Upstream,
    models: &AdmittedModels,
    checked: &CheckedUnit,
    emitted: &EmittedUnit,
    imports: qsl_package::V2ReadLimits,
) -> Result<CheckedPackageV2, DriverRefusal> {
    let evidence = admission_evidence(models, checked, emitted, imports)
        .map_err(DriverRefusal::QslAdmissionEvidence)?;
    let emitted_package = emitted.package();
    let package =
        match upstream.read_package(emitted, CheckedPackageReadLimits::bounded(), &evidence) {
            CheckedPackageDispatchResult::AdmittedV2(package) => *package,
            CheckedPackageDispatchResult::Refused(refusal) => {
                return Err(DriverRefusal::Fault(DriverFault::HandoffRefused(Box::new(
                    refusal,
                ))))
            }
            CheckedPackageDispatchResult::Incomplete(incomplete) => {
                return Err(DriverRefusal::HandoffLimit(incomplete))
            }
        };
    let expected = emitted_package.package_id().hex();
    if package.package_id().digest.as_ref() != expected {
        return Err(DriverRefusal::Fault(DriverFault::PackageIdMismatch {
            emitted: expected,
            admitted: package.package_id().digest.to_string(),
        }));
    }
    Ok(package)
}
