// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete linked-provider manifest carriers before the FR-331 adapter writes
//! a process request.

use std::sync::Arc;

use quire_driver::ProviderManifest;
use serde_json::{Map, Value};

/// Retain canonical FR-331 bytes in each linked manifest carrier. Process
/// hello carriers already contain the exact bytes admitted by QSL and pass
/// through unchanged.
pub(crate) fn retain_wire_bytes(
    mut manifests: Vec<ProviderManifest>,
) -> Result<Vec<ProviderManifest>, quire_canonical::Error> {
    for manifest in &mut manifests {
        if manifest.wire.is_some() {
            continue;
        }

        let advertises = manifest
            .advertises
            .iter()
            .filter_map(|(kind, mode)| {
                // An incomplete pair cannot pass QSL descriptor admission;
                // registered linked manifests therefore retain complete pairs.
                Some(serde_json::json!({
                    "kind": kind.as_deref()?,
                    "mode": mode.as_deref()?,
                }))
            })
            .collect::<Vec<_>>();
        let mut descriptor = Map::new();
        descriptor.insert("advertises".to_owned(), Value::Array(advertises));
        descriptor.insert(
            "backend".to_owned(),
            Value::String(manifest.identity.clone()),
        );
        if let Some(domains) = &manifest.domains {
            descriptor.insert("domains".to_owned(), serde_json::json!(domains));
        }
        let bytes = quire_canonical::to_vec(
            &Value::Object(descriptor),
            quire_canonical::Limits::new(1_048_576),
        )?;
        manifest.wire = Some(Arc::from(bytes));
    }
    Ok(manifests)
}
