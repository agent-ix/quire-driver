// SPDX-License-Identifier: AGPL-3.0-or-later
//! The upstream calls the driver makes, as one trait.
//!
//! [`QslCg`] is the production implementation. Its compile operation composes
//! QSL's typed front-end lifecycle; the other methods call one QSL or CG
//! operation each. The trait lets tests wrap `QslCg`, record driver calls and
//! still run the real functions.

use std::collections::BTreeMap;

use qsl_foundation::diagnostic::StageFailure;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, package, parse, refusal_or_fault, select, CheckedUnit, DependencyInput, EmittedUnit,
    FrontEndFailure, LockEvidence, PackageLimits, ParseRequest, SpineLimits, SpineStage,
};
use qsl_route::routing::{route, Disposition};
use qsl_route::{BackendId, Candidate};
use quire_contract_codegen::{
    generate_routed, negotiate_backend_provider, BackendKind, BackendProviderEnvelope,
    EnvelopeRefusal, GenerationContexts, ItemSettlement, RoutedGeneration, RoutedGenerationError,
    RoutedGenerationItem,
};
use quire_contract_model::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, CheckedPackageV2,
};
use quire_exact::Cancel;

use crate::{DriverFault, DriverRefusal};

#[derive(Clone, Copy, Debug)]
enum LifecycleOperation {
    Parse,
    Select,
    Check,
    Package,
}

impl LifecycleOperation {
    fn stage(self) -> SpineStage {
        match self {
            Self::Parse => SpineStage::Source,
            Self::Select => SpineStage::Intake,
            Self::Check => SpineStage::Check,
            Self::Package => SpineStage::Emit,
        }
    }
}

fn driver_refusal(failure: FrontEndFailure, operation: LifecycleOperation) -> DriverRefusal {
    match failure {
        StageFailure::Cancelled(cause) => DriverRefusal::Cancelled {
            stage: operation.stage(),
            cause,
        },
        failure => match refusal_or_fault(failure) {
            Ok(refusal) => DriverRefusal::Compile(refusal),
            Err(fault) => DriverRefusal::Fault(DriverFault::Spine {
                stage: operation.stage(),
                fault,
            }),
        },
    }
}

/// The QSL and CG calls [`crate::drive`] makes.
pub trait Upstream {
    /// S1 to S4 and the E4 emitter, composed from QSL's typed lifecycle
    /// operations. The driver supplies no libraries (ADR-015 D-1): it does
    /// not yet support compiling a unit's `import`s against caller-supplied
    /// library source. `cancel` is the caller's handle and is passed unchanged
    /// to each QSL lifecycle call.
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        packages: &BTreeMap<[u8; 32], Vec<u8>>,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal>;

    /// E5: IR's checked-package reader,
    /// `quire_contract_model::read_checked_package`. The full E4 value remains
    /// available here so E5 adapters can consult its bounded source provisions.
    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult;

    /// CG's one total conversion from a backend identity to its closed
    /// backend kind: `BackendKind::from_identity`.
    fn backend_kind(&self, id: &BackendId) -> Option<BackendKind>;

    /// E7: `quire_contract_codegen::negotiate_backend_provider`.
    fn negotiate(
        &self,
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ItemSettlement>, EnvelopeRefusal>;

    /// The routing step: `qsl_route::routing::route`.
    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>>;

    /// CG generation over the routed items:
    /// `quire_contract_codegen::generate_routed`.
    fn generate(
        &self,
        package: &CheckedPackageV2,
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError>;
}

/// The production [`Upstream`]: the real QSL and CG functions.
#[derive(Clone, Copy, Debug, Default)]
pub struct QslCg;

impl Upstream for QslCg {
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        packages: &BTreeMap<[u8; 32], Vec<u8>>,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal> {
        let request = ParseRequest {
            source: &source,
            path,
            bytes,
        };
        let parsed = parse(&request, limits.source, cancel)
            .map_err(|failure| driver_refusal(failure, LifecycleOperation::Parse))?
            .into_value();
        let models = select(&parsed, packages, limits.model, cancel)
            .map_err(|failure| driver_refusal(failure, LifecycleOperation::Select))?
            .into_value();
        let checked = check(
            &parsed,
            &models,
            &DependencyInput::default(),
            &LockEvidence::default(),
            limits,
            cancel,
        )
        .map_err(|failure| driver_refusal(failure, LifecycleOperation::Check))?
        .into_value();
        let emitted = package(&checked, PackageLimits::default(), cancel)
            .map_err(|failure| driver_refusal(failure, LifecycleOperation::Package))?
            .into_value();
        Ok((checked, emitted))
    }

    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        read_checked_package(emitted.package().bytes(), limits, evidence)
    }

    fn backend_kind(&self, id: &BackendId) -> Option<BackendKind> {
        BackendKind::from_identity(id.as_str())
    }

    fn negotiate(
        &self,
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ItemSettlement>, EnvelopeRefusal> {
        negotiate_backend_provider(envelope)
    }

    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>> {
        route(dispositions)
    }

    fn generate(
        &self,
        package: &CheckedPackageV2,
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        generate_routed(package, routed, contexts)
    }
}

#[cfg(test)]
mod tests {
    use qsl_foundation::diagnostic::{
        InternalFault, LimitExceeded, LimitKind, LimitsField, StageFailure,
    };
    use qsl_replay::spine::CompileRefusal;
    use quire_exact::CancelCause;

    use super::{driver_refusal, DriverFault, DriverRefusal, LifecycleOperation, SpineStage};
    use crate::{DriverStage, OutcomeKind};

    const OPERATIONS: [(LifecycleOperation, SpineStage); 4] = [
        (LifecycleOperation::Parse, SpineStage::Source),
        (LifecycleOperation::Select, SpineStage::Intake),
        (LifecycleOperation::Check, SpineStage::Check),
        (LifecycleOperation::Package, SpineStage::Emit),
    ];

    /// Trace: FR-001-AC-5
    #[test]
    fn lifecycle_faults_retain_details_and_operation_stage() {
        for (operation, expected_stage) in OPERATIONS {
            let expected = InternalFault::new("S3", "test-invariant");
            let refusal = driver_refusal(StageFailure::Fault(expected), operation);
            assert_eq!(refusal.code(), "runtime_invariant");
            assert_eq!(refusal.kind(), OutcomeKind::Fault);
            assert_eq!(refusal.stage(), DriverStage::Compile(expected_stage));
            let DriverRefusal::Fault(DriverFault::Spine { fault, .. }) = refusal else {
                panic!("a lifecycle fault must stay an internal fault");
            };
            assert_eq!(fault, expected);
        }
    }

    /// Trace: FR-011-AC-2, FR-011-AC-3 and FR-011-AC-4.
    #[test]
    fn lifecycle_cancellation_is_incomplete_and_preserves_stage_and_cause() {
        for (operation, expected_stage) in OPERATIONS {
            for cause in [CancelCause::Requested, CancelCause::Deadline] {
                let refusal = driver_refusal(StageFailure::Cancelled(cause), operation);
                assert_eq!(refusal.code(), "cancelled");
                assert_eq!(refusal.kind(), OutcomeKind::Cancelled);
                assert_eq!(refusal.stage(), DriverStage::Compile(expected_stage));
                assert_eq!(
                    refusal.qsl_category(),
                    Some(qsl_replay::Category::Incomplete)
                );
                let DriverRefusal::Cancelled { stage, cause: seen } = refusal else {
                    panic!("QSL cancellation must remain a distinct typed result");
                };
                assert_eq!(stage, expected_stage);
                assert_eq!(seen, cause);
            }
        }
    }

    /// Trace: FR-001-AC-5
    #[test]
    fn lifecycle_limits_preserve_qsl_payload_cause_and_field_stage() {
        for (kind, field, expected_stage, code) in [
            (
                LimitKind::InputBytes,
                LimitsField::SourceBytes,
                SpineStage::Source,
                "stage_limit_exceeded/input-bytes-exceeded",
            ),
            (
                LimitKind::WorkBudget,
                LimitsField::CheckingWorkBudget,
                SpineStage::Check,
                "stage_limit_exceeded/work-budget-exceeded",
            ),
        ] {
            let expected = LimitExceeded::new(kind, 8, 9).named(field);
            for (operation, _) in OPERATIONS {
                let refusal = driver_refusal(StageFailure::Limit(expected.clone()), operation);
                assert_eq!(refusal.code(), code);
                assert_eq!(refusal.kind(), OutcomeKind::Limit);
                assert_eq!(refusal.stage(), DriverStage::Compile(expected_stage));
                let DriverRefusal::Compile(refusal) = refusal else {
                    panic!("a lifecycle limit must be a compile limit");
                };
                let CompileRefusal::Limit(limit) = *refusal else {
                    panic!("the mapped refusal must retain the limit variant");
                };
                assert_eq!(limit, expected);
            }
        }
    }
}
