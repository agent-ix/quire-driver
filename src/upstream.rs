// SPDX-License-Identifier: AGPL-3.0-or-later
//! The upstream calls the driver makes, as one trait.
//!
//! [`QslCg`] is the production implementation. Its compile operation composes
//! QSL's typed front-end lifecycle; the other methods call one QSL or CG
//! operation each. The trait lets tests wrap `QslCg`, record driver calls and
//! still run the real functions.

use qsl_foundation::diagnostic::StageFailure;
use qsl_foundation::source::provenance::OccurrenceKey;
use qsl_foundation::SourceIdentity;
use qsl_package::{BoundItem, CheckedPackage, ExportRefusal};
use qsl_replay::spine::{
    check, package, parse, refusal_or_fault, select, AdmittedModels, CheckedUnit, DependencyInput,
    EmittedUnit, FrontEndFailure, PackageLimits, ParseRequest, SpineLimits, SpineStage,
};
use qsl_route::routing::{route, Disposition};
use qsl_route::Candidate;
use qsl_semantics::model::intake::PackageInput;
use quire_contract_codegen::{
    generate_routed, negotiate_contract_package, BackendDescriptor, BackendKind,
    BackendProviderEnvelope, ContractItemSettlement, ContractNegotiationError,
    ContractRequestPosition, GenerationContexts, RoutedGeneration, RoutedGenerationError,
    RoutedGenerationItem,
};
use quire_contract_model::{
    read_checked_package, CheckedPackageDispatchResult, CheckedPackageEvidence,
    CheckedPackageReadLimits, ContractPackageV1,
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
    /// Whether the checked package declares an abstraction relation.
    fn has_abstraction_relation(&self, package: &CheckedPackage) -> bool;
    /// QSL FR-307's per-item binding export.
    fn export_bindings(
        &self,
        package: &CheckedPackage,
        items: &[OccurrenceKey],
    ) -> Result<Vec<Result<BoundItem, ExportRefusal>>, ExportRefusal>;
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
        package_name: &str,
        packages: &PackageInput,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, AdmittedModels, EmittedUnit), DriverRefusal>;

    /// E5: IR's checked-package reader,
    /// `quire_contract_model::read_checked_package`. The full E4 value remains
    /// available here so E5 adapters can consult its bounded source provisions.
    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult;

    /// CG's origin-aware conversion of the exact projected descriptor to its
    /// closed backend kind: `BackendKind::from_descriptor`.
    fn backend_kind(&self, descriptor: &BackendDescriptor) -> Option<BackendKind>;

    /// E7: `quire_contract_codegen::negotiate_contract_package`.
    fn negotiate(
        &self,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ContractItemSettlement>, Box<ContractNegotiationError>>;

    /// The routing step: `qsl_route::routing::route`.
    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>>;

    /// CG generation over the routed items:
    /// `quire_contract_codegen::generate_routed`.
    fn generate(
        &self,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError>;
}

/// The production [`Upstream`]: the real QSL and CG functions.
#[derive(Clone, Copy, Debug, Default)]
pub struct QslCg;

impl Upstream for QslCg {
    fn has_abstraction_relation(&self, package: &CheckedPackage) -> bool {
        package.abstraction().is_some()
    }
    fn export_bindings(
        &self,
        package: &CheckedPackage,
        items: &[OccurrenceKey],
    ) -> Result<Vec<Result<BoundItem, ExportRefusal>>, ExportRefusal> {
        Ok(qsl_package::export_bindings(package, items))
    }
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        package_name: &str,
        packages: &PackageInput,
        limits: SpineLimits,
        cancel: &Cancel,
    ) -> Result<(CheckedUnit, AdmittedModels, EmittedUnit), DriverRefusal> {
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
            package_name.to_owned(),
            &models,
            &DependencyInput::default(),
            limits,
            cancel,
        )
        .map_err(|failure| driver_refusal(failure, LifecycleOperation::Check))?
        .into_value();
        let emitted = package(&checked, PackageLimits::default(), cancel)
            .map_err(|failure| driver_refusal(failure, LifecycleOperation::Package))?
            .into_value();
        Ok((checked, models, emitted))
    }

    fn read_package(
        &self,
        emitted: &EmittedUnit,
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        read_checked_package(emitted.package().bytes(), limits, evidence)
    }

    fn backend_kind(&self, descriptor: &BackendDescriptor) -> Option<BackendKind> {
        BackendKind::from_descriptor(descriptor)
    }

    fn negotiate(
        &self,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        envelope: &BackendProviderEnvelope,
    ) -> Result<Vec<ContractItemSettlement>, Box<ContractNegotiationError>> {
        negotiate_contract_package(package, positions, envelope).map_err(Box::new)
    }

    fn route<'d>(&self, dispositions: &'d [Disposition]) -> Vec<Option<&'d Candidate>> {
        route(dispositions)
    }

    fn generate(
        &self,
        package: &ContractPackageV1,
        positions: &[ContractRequestPosition],
        routed: &[RoutedGenerationItem],
        contexts: &GenerationContexts<'_>,
    ) -> Result<RoutedGeneration, RoutedGenerationError> {
        generate_routed(package, positions, routed, contexts)
    }
}

#[cfg(test)]
mod tests {
    use qsl_foundation::diagnostic::{LimitExceeded, StageFailure};
    use qsl_foundation::Setting;
    use qsl_replay::spine::CompileRefusal;
    use quire_exact::CancelCause;
    use quire_semantic_value::catalog::InternalFault;

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
            let refusal = driver_refusal(StageFailure::Fault(expected.clone()), operation);
            assert_eq!(refusal.code(), "runtime_invariant");
            assert_eq!(refusal.kind(), OutcomeKind::Fault);
            assert_eq!(refusal.stage(), DriverStage::Compile(expected_stage));
            let DriverRefusal::Fault(DriverFault::Spine { fault, .. }) = refusal else {
                panic!("a lifecycle fault must stay an internal fault");
            };
            assert_eq!(fault, expected);
        }
    }

    /// Trace: FR-011-AC-2, FR-011-AC-3, FR-011-AC-4.
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
                    Some(quire_semantic_value::catalog::Category::Incomplete)
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
        for (setting, expected_stage, code) in [
            (
                Setting::S1InputBytes,
                SpineStage::Source,
                "stage_limit_exceeded/input-bytes-exceeded",
            ),
            (
                Setting::S3WorkUnits,
                SpineStage::Check,
                "stage_limit_exceeded/work-budget-exceeded",
            ),
        ] {
            let expected = LimitExceeded::new(setting, 8, 9);
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
