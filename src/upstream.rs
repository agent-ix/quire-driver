// SPDX-License-Identifier: AGPL-3.0-or-later
//! The upstream calls the driver makes, as one trait.
//!
//! [`QslCg`] is the production implementation: each method calls the QSL or
//! CG function named in its doc and nothing else. The trait exists so a test
//! can wrap `QslCg`, record the order of calls and still run the real
//! functions.

use std::collections::BTreeMap;

use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{compile, CompileRefusal, Compiled, SpineLimits};
use qsl_route::routing::{route, Disposition};
use qsl_route::{BackendId, Candidate};
use quire_contract_codegen::{
    negotiate_backend_provider, BackendKind, BackendProviderEnvelope, EnvelopeRefusal,
    ItemSettlement,
};

/// The QSL and CG calls [`crate::drive`] makes.
pub trait Upstream {
    /// S1 to S4 and the E4 emitter: `qsl_replay::spine::compile`.
    fn compile(
        &self,
        source: SourceIdentity,
        path: &str,
        bytes: &[u8],
        packages: &BTreeMap<[u8; 32], Vec<u8>>,
        limits: SpineLimits,
    ) -> Result<Compiled, Box<CompileRefusal>>;

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
    ) -> Result<Compiled, Box<CompileRefusal>> {
        compile(source, path, bytes, packages, limits)
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
}
