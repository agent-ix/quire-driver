// SPDX-License-Identifier: AGPL-3.0-or-later
//! The upstream calls the driver makes, as one trait.
//!
//! [`QslCg`] is the production implementation. Its compile operation composes
//! QSL's typed front-end lifecycle; the other methods call one QSL or CG
//! operation each. The trait lets tests wrap `QslCg`, record driver calls and
//! still run the real functions.

use std::collections::BTreeMap;

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, package, parse, refusal_or_fault, select, CheckedUnit, DependencyInput, EmittedUnit,
    FrontEndFailure, LockEvidence, PackageLimits, ParseRequest, SpineLimits,
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

use crate::{DriverFault, DriverRefusal};

fn driver_refusal(failure: FrontEndFailure) -> DriverRefusal {
    match refusal_or_fault(failure) {
        Ok(refusal) => DriverRefusal::Compile(refusal),
        Err(fault) => DriverRefusal::Fault(DriverFault::Spine(fault)),
    }
}

/// The QSL and CG calls [`crate::drive`] makes.
pub trait Upstream {
    /// S1 to S4 and the E4 emitter, composed from QSL's typed lifecycle
    /// operations. The driver supplies no libraries (ADR-015 D-1): it does
    /// not yet support compiling a unit's `import`s against caller-supplied
    /// library source.
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        packages: &BTreeMap<[u8; 32], Vec<u8>>,
        limits: SpineLimits,
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal>;

    /// E5: IR's checked-package reader,
    /// `quire_contract_model::read_checked_package`.
    fn read_package(
        &self,
        bytes: &[u8],
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
    ) -> Result<(CheckedUnit, EmittedUnit), DriverRefusal> {
        // All four typed lifecycle operations share one handle. The driver
        // owns this handle until a caller-facing cancellation API is added.
        let cancel = Default::default();
        let request = ParseRequest {
            source: &source,
            path,
            bytes,
        };
        let parsed = parse(&request, limits.source, &cancel)
            .map_err(driver_refusal)?
            .into_value();
        let models = select(&parsed, packages, limits.model, &cancel)
            .map_err(driver_refusal)?
            .into_value();
        let checked = check(
            &parsed,
            &models,
            &DependencyInput::default(),
            &LockEvidence::default(),
            limits,
            &cancel,
        )
        .map_err(driver_refusal)?
        .into_value();
        let emitted = package(&checked, PackageLimits::default(), &cancel)
            .map_err(driver_refusal)?
            .into_value();
        Ok((checked, emitted))
    }

    fn read_package(
        &self,
        bytes: &[u8],
        limits: CheckedPackageReadLimits,
        evidence: &CheckedPackageEvidence,
    ) -> CheckedPackageDispatchResult {
        read_checked_package(bytes, limits, evidence)
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
