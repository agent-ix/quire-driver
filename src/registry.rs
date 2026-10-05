// SPDX-License-Identifier: AGPL-3.0-or-later
//! The registry value (ADR-012 §7.1): built by the driver from provider
//! manifests and passed to every consumer as an argument.

use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, Registry};
use std::ops::Deref;

use crate::DriverRefusal;

/// The FR-331 provider-manifest fields the driver registers a backend from.
///
/// The `advertises` pairs are the manifest's `(kind, mode)` labels exactly
/// as it states them; `route` admits them (`BackendDescriptor::admit`), so an
/// absent or unknown label refuses the registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderManifest {
    /// The backend identity (`backend.identity`).
    pub identity: String,
    /// The digest of this manifest (`backend.manifest_digest`).
    pub manifest_digest: ManifestDigest,
    /// Each advertised `(kind, mode)` label pair; `None` is an absent label.
    pub advertises: Vec<(Option<String>, Option<String>)>,
}

/// A registry accepted by the driver's complete manifest-admission pass.
///
/// The wrapper exposes only shared access, so callers cannot add a conflicting
/// descriptor after admission and then create cache requests from stale state.
#[derive(Debug)]
pub struct AdmittedRegistry {
    registry: Registry,
}

impl Deref for AdmittedRegistry {
    type Target = Registry;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

/// Build the registry from `manifests`.
///
/// Every manifest is admitted and registered. When any registration is
/// refused (an absent or unknown kind or mode, or a duplicate backend
/// identity), the run is refused with every refusal, ordered by backend
/// identity then manifest digest.
///
/// `Registry::register`'s own `Err` is what that one call caused or
/// observed, not a diff against the registry's prior state, so it can
/// repeat a conflict an earlier call in this same loop already reported.
/// Rather than accumulate those (and dedup them by hand), this collects
/// only the vocabulary refusals `BackendDescriptor::admit` produces during
/// the loop, then reads every duplicate-backend conflict once, after the
/// loop, from `Registry::refusals` -- the registry's own stable, deduped
/// account of every identity conflict it currently holds.
pub fn build_registry(manifests: &[ProviderManifest]) -> Result<AdmittedRegistry, DriverRefusal> {
    let mut registry = Registry::new();
    let mut refusals = Vec::new();
    for manifest in manifests {
        let backend_id = BackendId::from_wire(&manifest.identity)
            .map_err(DriverRefusal::EmptyBackendIdentity)?;
        let backend = Candidate::new(backend_id);
        let admitted = BackendDescriptor::admit(
            backend,
            manifest.manifest_digest,
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
        );
        match admitted {
            Ok(descriptor) => {
                // The conflict is read back, deduped, from `refusals()`
                // below; a successful registration needs nothing here.
                let _ = registry.register(descriptor);
            }
            Err(refusal) => refusals.push(refusal),
        }
    }
    refusals.extend(registry.refusals());
    if refusals.is_empty() {
        Ok(AdmittedRegistry { registry })
    } else {
        refusals.sort();
        Err(DriverRefusal::Registration(refusals))
    }
}
