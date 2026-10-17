// SPDX-License-Identifier: AGPL-3.0-or-later
//! E5 (ADR-011 §2.1): the E4 v2 bytes handed to IR's checked-package reader,
//! admitted only at the `package_id` E4 emitted them under. The complete E4
//! value remains available through admission for bounded diagnostic sources.

use quire_contract_model::{
    CheckedPackageDispatchResult, CheckedPackageEvidence, CheckedPackageReadLimits,
    CheckedPackageV2,
};

use qsl_replay::spine::EmittedUnit;
use qsl_semantics::model::key::hex;
use qsl_semantics::value::{CatalogRole, DefinitionLock};

use crate::{CompileInput, DriverFault, DriverRefusal, Upstream};

/// Read `emitted` through IR's reader and check the admitted `package_id`
/// is the one E4 minted.
pub(crate) fn handoff(
    upstream: &impl Upstream,
    input: &CompileInput<'_>,
    emitted: &EmittedUnit,
) -> Result<CheckedPackageV2, DriverRefusal> {
    let evidence = evidence(input)?;
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

/// The evidence IR's reader needs to admit the package:
///
/// - each FR-056 domain package document the caller supplied;
/// - the root feature of QSL's pinned `DefinitionLock` catalog, as a
///   supported feature.
///
/// No artifact-byte or artifact-digest evidence is supplied; IR no longer
/// checks the package for staleness against artifact digests.
fn evidence(input: &CompileInput<'_>) -> Result<CheckedPackageEvidence, DriverRefusal> {
    let mut evidence = CheckedPackageEvidence::new();
    for (digest, document) in input.packages {
        evidence
            .insert_domain_package_document(hex(digest), document.as_slice())
            .expect("the input map supplies one document per digest");
    }
    let lock = DefinitionLock::pinned();
    let root = lock.entry(CatalogRole::Root);
    evidence.support_feature(root.identity);
    Ok(evidence)
}
