// SPDX-License-Identifier: AGPL-3.0-or-later
//! The registry value (ADR-012 §7.1): built by the driver from provider
//! manifests and passed to every consumer as an argument.

use qsl_route::{BackendDescriptor, Candidate, ManifestDigest, Registry, ToolIdentity};

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
    /// The pinned tool identity.
    pub tool: String,
    /// Each advertised `(kind, mode)` label pair; `None` is an absent label.
    pub advertises: Vec<(Option<String>, Option<String>)>,
}

/// Build the registry from `manifests`.
///
/// Every manifest is admitted and registered. When any registration is
/// refused (an absent or unknown kind or mode, or a duplicate backend
/// identity), the run is refused with every refusal, ordered by backend
/// identity then manifest digest.
pub fn build_registry(manifests: &[ProviderManifest]) -> Result<Registry, DriverRefusal> {
    let mut registry = Registry::new();
    let mut refusals = Vec::new();
    for manifest in manifests {
        let backend = Candidate::new(
            qsl_route::BackendId::new(manifest.identity.clone()),
            manifest.manifest_digest,
        );
        let admitted = BackendDescriptor::admit(
            backend,
            ToolIdentity::new(manifest.tool.clone()),
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
        );
        match admitted {
            Ok(descriptor) => {
                if let Err(refusal) = registry.register(descriptor) {
                    refusals.push(refusal);
                }
            }
            Err(refusal) => refusals.push(refusal),
        }
    }
    if refusals.is_empty() {
        Ok(registry)
    } else {
        refusals.sort();
        Err(DriverRefusal::Registration(refusals))
    }
}
