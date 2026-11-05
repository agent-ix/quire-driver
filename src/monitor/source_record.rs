// SPDX-License-Identifier: AGPL-3.0-or-later
//! Authentic QSL source-record witnesses joined to their own strict IR admission.

use qsl_package::Emission;
use quire_contract_model::{
    read_checked_package, CheckedNodeId, CheckedPackageDispatchResult, CheckedPackageIncomplete,
    CheckedPackageReadLimits, CheckedPackageRefusal, CheckedPackageV2, ExactTypeProjection,
    ExactTypeProjectionError, SelectedExactIdentities, SelectedSourceRecordMemberRow,
    SelectedSourceRecordRow,
};

/// Stable disposition of an independent source-record admission failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceRecordAdmissionErrorCode {
    /// Strict IR reading refused the emission.
    ReadRefused,
    /// Strict IR reading stopped at a reader limit.
    ReadIncomplete,
    /// Native source-record joining failed or exhausted its allowance.
    Join,
}

/// Original owner payload from strict reading or native source-record joining.
#[derive(Debug, thiserror::Error)]
pub enum SourceRecordAdmissionError {
    /// The strict reader's original refusal.
    #[error("source-record package read refused: {0:?}")]
    ReadRefused(CheckedPackageRefusal),
    /// The strict reader's original incomplete result.
    #[error("source-record package read incomplete: {0:?}")]
    ReadIncomplete(CheckedPackageIncomplete),
    /// The native join's original error, including work and source locus.
    #[error("source-record witness join failed: {0}")]
    Join(#[source] ExactTypeProjectionError),
}

impl SourceRecordAdmissionError {
    /// Stable classification without inspecting diagnostic text.
    pub fn code(&self) -> SourceRecordAdmissionErrorCode {
        match self {
            Self::ReadRefused(_) => SourceRecordAdmissionErrorCode::ReadRefused,
            Self::ReadIncomplete(_) => SourceRecordAdmissionErrorCode::ReadIncomplete,
            Self::Join(_) => SourceRecordAdmissionErrorCode::Join,
        }
    }
}

/// Source-record identities authenticated by one immutable producer emission.
///
/// This component establishes no model-owned identities, executable closure,
/// clock association or complete monitor preparation. Its owned package is the
/// exact strict-reader output against which its private identities were joined.
pub struct AdmittedSourceRecords {
    package: Box<CheckedPackageV2>,
    identities: SelectedExactIdentities,
}

impl AdmittedSourceRecords {
    /// The original strict-reader output for these producer witnesses.
    pub fn package(&self) -> &CheckedPackageV2 {
        &self.package
    }

    /// Complete native joined rows, retaining producer order within each row.
    pub fn rows(&self) -> impl Iterator<Item = &SelectedSourceRecordRow> {
        self.identities.source_record_rows()
    }

    /// Project an independently selected node using this owner's authentic rows.
    ///
    /// `work_limit` is a separate native projection allowance. Native success
    /// work and errors remain unchanged; it is not a preparation-wide receipt.
    pub fn project(
        &self,
        node: &CheckedNodeId,
        work_limit: u64,
    ) -> Result<ExactTypeProjection, ExactTypeProjectionError> {
        self.package
            .project_exact_type(node, &self.identities, work_limit)
    }
}

/// Strictly read an authentic emission and join its complete source-record rows.
///
/// IR owns reader limits and the finite `join_work_limit`. Row materialization
/// and producer acquisition are outside the native join allowance. The native
/// join returns no success-work receipt, so these independent limits do not
/// establish cumulative preparation work or complete storage qualification.
/// No caller-created row or separately admitted package supplies authority.
pub fn admit_source_records(
    emission: &Emission,
    read_limits: CheckedPackageReadLimits,
    join_work_limit: u64,
) -> Result<AdmittedSourceRecords, SourceRecordAdmissionError> {
    let package =
        match read_checked_package(emission.package().bytes(), read_limits, emission.evidence()) {
            CheckedPackageDispatchResult::AdmittedV2(package) => package,
            CheckedPackageDispatchResult::Refused(error) => {
                return Err(SourceRecordAdmissionError::ReadRefused(error));
            }
            CheckedPackageDispatchResult::Incomplete(error) => {
                return Err(SourceRecordAdmissionError::ReadIncomplete(error));
            }
        };
    let rows = emission
        .selected_source_record_rows()
        .iter()
        .map(|row| SelectedSourceRecordRow {
            structural_node: row.structural_node.clone(),
            source_authority: row.source_authority.clone(),
            source_identity: row.source_identity.clone(),
            qualified_declaration: row.qualified_declaration.clone(),
            record_key: row.record_key.clone(),
            members: row
                .members
                .iter()
                .map(|member| SelectedSourceRecordMemberRow {
                    original_member_identity: member.original_member_identity.clone(),
                    admitted_name: member.admitted_name.clone(),
                    presence: member.presence,
                    type_node: member.type_node.clone(),
                    value_type: member.value_type.clone(),
                })
                .collect(),
        });
    let identities = SelectedExactIdentities::empty()
        .with_source_record_rows(&package, rows, join_work_limit)
        .map_err(SourceRecordAdmissionError::Join)?;
    Ok(AdmittedSourceRecords {
        package,
        identities,
    })
}
