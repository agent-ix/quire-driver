// SPDX-License-Identifier: AGPL-3.0-or-later
//! E5 (ADR-011 §2.1): the E4 v2 bytes handed to IR's checked-package reader,
//! admitted only at the `package_id` E4 emitted them under.

use quire_contract_model::{
    CheckedArtifactLocator, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2,
};

use qsl_foundation::digest::DigestDomain;
use qsl_package::EmittedPackage;
use qsl_semantics::model::key::hex;
use qsl_semantics::value::{CatalogRole, DefinitionLock};

use crate::{CompileInput, DriverFault, DriverRefusal, Upstream};

/// Read `emitted` through IR's reader and check the admitted `package_id`
/// is the one E4 minted.
pub(crate) fn handoff(
    upstream: &impl Upstream,
    input: &CompileInput<'_>,
    emitted: &EmittedPackage,
) -> Result<CheckedPackageV2, DriverRefusal> {
    let evidence = evidence(input)?;
    let package = match upstream.read_package(
        emitted.bytes(),
        CheckedPackageReadLimits::bounded(),
        &evidence,
    ) {
        CheckedPackageDispatchResult::AdmittedV2(package) => *package,
        CheckedPackageDispatchResult::Refused(refusal) => {
            return Err(DriverRefusal::Fault(DriverFault::HandoffRefused(refusal)))
        }
        CheckedPackageDispatchResult::Incomplete(incomplete) => {
            return Err(DriverRefusal::HandoffLimit(incomplete))
        }
    };
    let expected = emitted.package_id().hex();
    if package.package_id().digest.as_ref() != expected {
        return Err(DriverRefusal::Fault(DriverFault::PackageIdMismatch {
            emitted: expected,
            admitted: package.package_id().digest.to_string(),
        }));
    }
    Ok(package)
}

/// The evidence the lock is checked against:
///
/// - the unit's own bytes, under its source identity (IR digests them);
/// - each FR-056 domain package document the caller supplied;
/// - the definition digests of QSL's pinned `DefinitionLock` catalog, and
///   its root feature. Those digests are QSL's own informational constants,
///   never checked against the definition bytes, so they attest nothing
///   beyond QSL's catalog;
/// - the diagnostics catalog reference, `qsl_package::diagnostics_catalog()`
///   (QSL-260, #453): the same reference QSL's own emitter pins at
///   `/diagnostics/catalog`, so IR's `/diagnostics/catalog` staleness check
///   is vacuous (the entry cannot be stale against itself), but it is read
///   from QSL's own API rather than parsed back out of the emitted bytes.
fn evidence(input: &CompileInput<'_>) -> Result<CheckedPackageEvidence, DriverRefusal> {
    let mut evidence = CheckedPackageEvidence::new();
    evidence.insert_artifact_bytes(
        CheckedArtifactLocator {
            authority: input.source.authority.as_str().into(),
            identity: input.source.identity.as_str().into(),
            revision_namespace: input.source.revision_namespace.as_str().into(),
            revision_value: input.source.revision.as_str().into(),
            domain: DigestDomain::SourceBytesV1.as_str().into(),
        },
        input.bytes,
    );
    for (digest, document) in input.packages {
        evidence.insert_domain_package_document(hex(digest), document.as_slice());
    }
    let lock = DefinitionLock::pinned();
    for entry in lock.catalog() {
        evidence.insert_artifact_digest(
            CheckedArtifactLocator {
                authority: entry.authority.into(),
                identity: entry.identity.into(),
                revision_namespace: entry.revision_namespace.into(),
                revision_value: entry.revision_value.into(),
                domain: DigestDomain::DefinitionBytesV1.as_str().into(),
            },
            entry.digest,
        );
    }
    if let Some(root) = lock.entry(CatalogRole::Root) {
        evidence.support_feature(root.identity);
    }
    let catalog = qsl_package::diagnostics_catalog();
    evidence.insert_artifact_digest(
        CheckedArtifactLocator {
            authority: catalog.authority,
            identity: catalog.identity,
            revision_namespace: catalog.revision.namespace,
            revision_value: catalog.revision.value,
            domain: catalog.digest_domain,
        },
        catalog.digest,
    );
    Ok(evidence)
}
