// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed driver inputs for QSpec's backend-provider/v1 envelope.
//!
//! This module projects the path-addressed FR-331 contract into driver types;
//! it does not carry a copy of the schema or its vectors. The decoded manifest
//! JSON is retained so optional manifest members survive admission and can be
//! included in a later request's registry snapshot.

use std::collections::BTreeMap;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use qsl_package::EmittedPackage;
use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const CONTRACT_VERSION: &str = "quire.backend-provider/v1";
const CAPABILITY_VOCABULARY: &str = "quire.capability-kind/v1";
const MANIFEST_DIGEST_LIMIT: u64 = u64::MAX;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderDescriptorShape {
    backend: String,
    advertises: Vec<AdvertisedPair>,
    #[serde(default)]
    domains: Option<Vec<String>>,
    #[serde(default)]
    options: Option<BTreeMap<String, String>>,
    #[serde(default)]
    bounds: Option<BTreeMap<String, u64>>,
    #[serde(default)]
    obligation_forms: Option<Vec<String>>,
    #[serde(default)]
    reductions: Option<Vec<String>>,
    #[serde(default)]
    evidence_kinds: Option<Vec<String>>,
    #[serde(default)]
    fairness_kinds: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvertisedPair {
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    mode: Option<String>,
}

/// The FR-331 manifest-stage body received in the plugin's hello.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestStage {
    /// FR-331 contract identity.
    pub contract_version: String,
    /// FR-290 capability-vocabulary identity.
    pub capability_vocabulary: String,
    /// The plugin's losslessly retained provider-manifest descriptor.
    pub manifest: Vec<Value>,
}

/// One successfully admitted process manifest and the exact wire snapshot
/// retained for later FR-331 requests.
#[derive(Clone, Debug)]
pub struct AdmittedManifest {
    descriptor: BackendDescriptor,
    wire: Value,
}

impl AdmittedManifest {
    /// The descriptor admitted once by QSL for this process hello.
    #[must_use]
    pub fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    /// The original FR-331 descriptor, including optional manifest members.
    #[must_use]
    pub fn wire(&self) -> &Value {
        &self.wire
    }
}

/// An invalid FR-331 hello body or a typed QSL registration refusal.
#[derive(Debug, thiserror::Error)]
pub enum ManifestAdmissionError {
    /// The manifest-stage envelope identity did not match FR-331.
    #[error("plugin hello has an unsupported FR-331 envelope identity")]
    EnvelopeIdentity,
    /// A provider hello did not carry exactly one descriptor.
    #[error("plugin hello must carry exactly one provider descriptor")]
    DescriptorCount,
    /// A descriptor member had the wrong JSON shape or type.
    #[error("plugin hello manifest has an invalid {field} member")]
    InvalidMember {
        /// The member whose type or shape did not satisfy FR-331.
        field: &'static str,
    },
    /// QSL refused the descriptor's backend identity.
    #[error(transparent)]
    EmptyBackendIdentity(#[from] qsl_route::EmptyBackendIdentity),
    /// QSL refused an advertised kind or mode with its native cause.
    #[error(transparent)]
    Registration(#[from] qsl_route::RegistrationRefusal),
    /// Canonical manifest hashing refused the input or exceeded its byte bound.
    #[error("plugin hello manifest could not be canonically digested: {0}")]
    Canonical(#[from] quire_canonical::Error),
}

impl ManifestStage {
    /// Admit the hello's one manifest using the caller-supplied Process origin.
    ///
    /// The digest is computed from the exact decoded FR-331 descriptor using
    /// JCS. QSL owns identity, capability, mode and duplicate admission; this
    /// method neither normalizes labels nor assigns an origin from wire data.
    pub fn admit_process(self) -> Result<AdmittedManifest, ManifestAdmissionError> {
        if self.contract_version != CONTRACT_VERSION
            || self.capability_vocabulary != CAPABILITY_VOCABULARY
        {
            return Err(ManifestAdmissionError::EnvelopeIdentity);
        }
        let [wire] = self.manifest.as_slice() else {
            return Err(ManifestAdmissionError::DescriptorCount);
        };
        let shape: ProviderDescriptorShape =
            serde_json::from_value(wire.clone()).map_err(|_| {
                ManifestAdmissionError::InvalidMember {
                    field: "manifest descriptor",
                }
            })?;
        if shape.advertises.is_empty() {
            return Err(ManifestAdmissionError::InvalidMember {
                field: "advertises",
            });
        }
        let _preserved_optional_fields = (
            &shape.domains,
            &shape.options,
            &shape.bounds,
            &shape.obligation_forms,
            &shape.reductions,
            &shape.evidence_kinds,
            &shape.fairness_kinds,
        );
        let digest =
            quire_canonical::sha256(wire, quire_canonical::Limits::new(MANIFEST_DIGEST_LIMIT))?;
        let descriptor = BackendDescriptor::admit(
            Candidate::new(BackendId::from_wire(&shape.backend)?),
            ProviderOrigin::Process,
            ManifestDigest::from_digest(*digest.as_bytes()),
            shape
                .advertises
                .iter()
                .map(|pair| (pair.kind.as_deref(), pair.mode.as_deref())),
        )?;
        Ok(AdmittedManifest {
            descriptor,
            wire: wire.clone(),
        })
    }
}

/// One FR-331 request-stage envelope to serialize for a process provider.
#[derive(Clone, Debug, Serialize)]
pub struct RequestStage {
    /// FR-331 contract identity.
    pub contract_version: &'static str,
    /// FR-290 capability-vocabulary identity.
    pub capability_vocabulary: &'static str,
    /// Snapshot of retained FR-331 manifests for the current QSL registry.
    pub manifest: Vec<Value>,
    /// The provider request body.
    pub request: RequestBody,
}

impl RequestStage {
    /// Build the wire request from an emitted package and typed routed inputs.
    ///
    /// `package_id` is copied from the emitter's `EmittedPackage`; the driver
    /// never recomputes package identity from the bytes. `manifest` comes from
    /// retained decoded provider-manifest DTOs, not from QSL registry
    /// descriptors, which intentionally omit optional FR-331 members.
    #[must_use]
    pub fn from_emitted(
        package: &EmittedPackage,
        manifest: Vec<Value>,
        deadline_ms: u64,
        items: Vec<RequestItem>,
    ) -> Self {
        let package_id = package.package_id().record();
        Self {
            contract_version: CONTRACT_VERSION,
            capability_vocabulary: CAPABILITY_VOCABULARY,
            manifest,
            request: RequestBody {
                package: URL_SAFE_NO_PAD.encode(package.bytes()),
                package_id: PackageIdWire {
                    domain: package_id.domain().as_str(),
                    algorithm: "sha256",
                    digest: package_id.hex(),
                },
                deadline_ms,
                items,
                dependencies: None,
                provision: None,
                memory: None,
            },
        }
    }
}

/// FR-331 request members supplied for one provider call.
#[derive(Clone, Debug, Serialize)]
pub struct RequestBody {
    /// RFC 8785 emitted package bytes encoded as unpadded base64url.
    pub package: String,
    /// The emitter-owned package identity beside its bytes.
    pub package_id: PackageIdWire,
    /// Caller-supplied provider deadline in milliseconds.
    pub deadline_ms: u64,
    /// Routed items; their indices and candidates come from CG's routing seam.
    pub items: Vec<RequestItem>,
    /// Dependency package bytes supplied by the package-selection owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<Value>>,
    /// Byte provisions supplied by the owning request/selection layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provision: Option<Vec<Value>>,
    /// Memory-model selections supplied by the owning request layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<Vec<Value>>,
}

/// FR-331 checked-package identity wire record.
#[derive(Clone, Debug, Serialize)]
pub struct PackageIdWire {
    /// QSpec package identity domain.
    pub domain: &'static str,
    /// The fixed digest algorithm.
    pub algorithm: &'static str,
    /// Lowercase hexadecimal digest from the emitted package identity.
    pub digest: String,
}

/// One FR-331 request item, with its QSpec-owned claim and candidate values.
#[derive(Clone, Debug, Serialize)]
pub struct RequestItem {
    /// Zero-based request position supplied by the driver/CG routing seam.
    pub request_index: u64,
    /// Requested FR-290 capability label, kept as received from the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// QSpec locus, supplied by the driver's original request item.
    pub claim: Value,
    /// Classification and proof bounds supplied by CG's routed item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extent: Option<Extent>,
    /// The selected backend identity from CG routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    /// Registry candidate set supplied by CG routing.
    pub candidates: Value,
    /// Backend options supplied by the caller's request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
    /// Reductions supplied by the caller's request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reductions: Option<Value>,
    /// Temporal request; proof bounds are carried below its subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal: Option<TemporalRequest>,
    /// Probabilistic selections supplied by the caller's request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probabilistic: Option<Value>,
}

/// FR-331 finite or unbounded extent. Proof bounds stay under `extent.bounds`.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "classification", rename_all = "kebab-case")]
pub enum Extent {
    /// Finite extent with the supplied substitutions.
    Bounded {
        /// Per-domain proof bounds supplied by CG's routed request.
        bounds: Vec<ProofBound>,
    },
    /// Unbounded extent and its request-owned domain declarations.
    Unbounded {
        /// Domain descriptions supplied by CG's routed request.
        domains: Vec<Value>,
        /// Whether a finite bound is available for these domains.
        finite_bound_available: bool,
    },
}

/// One FR-331 ADR-014 domain substitution.
#[derive(Clone, Debug, Serialize)]
pub struct ProofBound {
    /// The bounded QSpec domain key.
    pub domain: Value,
    /// The finite bound supplied by the request.
    pub bound: Value,
    /// Future QSpec additions such as QSL-654's `kind`, retained as data.
    #[serde(flatten)]
    pub additional: BTreeMap<String, Value>,
}

/// Temporal FR-331 request members.
#[derive(Clone, Debug, Serialize)]
pub struct TemporalRequest {
    /// Temporal subject and its proof bounds.
    pub subject: TemporalSubject,
    /// Other temporal members supplied by the owning request layer.
    #[serde(flatten)]
    pub additional: BTreeMap<String, Value>,
}

/// Temporal request subject with proof bounds at the FR-331 path.
#[derive(Clone, Debug, Serialize)]
pub struct TemporalSubject {
    /// Temporal subject's QSpec-defined identity and data.
    #[serde(flatten)]
    pub fields: BTreeMap<String, Value>,
    /// Proof bounds at `temporal.subject.proof_bounds`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_bounds: Option<Vec<ProofBound>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(kind: Value, mode: Value) -> ManifestStage {
        ManifestStage {
            contract_version: CONTRACT_VERSION.to_owned(),
            capability_vocabulary: CAPABILITY_VOCABULARY.to_owned(),
            manifest: vec![serde_json::json!({
                "backend": "agent-ix/provider/test",
                "advertises": [{ "kind": kind, "mode": mode }],
                "domains": ["integer"],
                "bounds": { "max_states": 8 }
            })],
        }
    }

    #[test]
    fn hello_admits_process_descriptor_and_retains_full_manifest() {
        let admitted = hello(
            Value::String("operation-contract".to_owned()),
            Value::String("bounded".to_owned()),
        )
        .admit_process()
        .expect("known labels are admitted");

        assert_eq!(admitted.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(
            admitted.descriptor().id().as_str(),
            "agent-ix/provider/test"
        );
        assert_eq!(admitted.wire()["domains"], serde_json::json!(["integer"]));
        assert_eq!(admitted.wire()["bounds"]["max_states"], Value::from(8));
    }

    #[test]
    fn absent_and_unknown_hello_labels_reach_qsl_admission() {
        for invalid in [
            (
                Value::Null,
                Value::String("bounded".to_owned()),
                "absent-kind",
            ),
            (
                Value::String("made-up-kind".to_owned()),
                Value::String("bounded".to_owned()),
                "unknown-kind",
            ),
            (
                Value::String("operation-contract".to_owned()),
                Value::String("made-up-mode".to_owned()),
                "unknown-mode",
            ),
            (
                Value::String("operation-contract".to_owned()),
                Value::Null,
                "unknown-mode",
            ),
        ] {
            let error = hello(invalid.0, invalid.1)
                .admit_process()
                .expect_err("QSL refuses malformed capability labels");
            let ManifestAdmissionError::Registration(refusal) = error else {
                panic!("malformed labels must be classified by QSL: {error:?}");
            };
            assert_eq!(refusal.cause().as_str(), invalid.2);
        }
    }

    #[test]
    fn request_wire_keeps_package_bounds_and_temporal_proof_bounds_at_contract_paths() {
        let item = RequestItem {
            request_index: 3,
            kind: Some("operation-contract".to_owned()),
            claim: serde_json::json!({"node": {}, "occurrence": {}}),
            extent: Some(Extent::Bounded {
                bounds: vec![ProofBound {
                    domain: serde_json::json!({"type": "population"}),
                    bound: serde_json::json!({"type": "cardinality", "maximum": 4}),
                    additional: BTreeMap::new(),
                }],
            }),
            backend: Some("agent-ix/provider/test".to_owned()),
            candidates: serde_json::json!({"type": "set", "backends": ["agent-ix/provider/test"]}),
            options: None,
            reductions: None,
            temporal: Some(TemporalRequest {
                subject: TemporalSubject {
                    fields: BTreeMap::new(),
                    proof_bounds: Some(vec![ProofBound {
                        domain: serde_json::json!({"type": "population"}),
                        bound: serde_json::json!({"type": "cardinality", "maximum": 4}),
                        additional: BTreeMap::new(),
                    }]),
                },
                additional: BTreeMap::new(),
            }),
            probabilistic: None,
        };
        let request = serde_json::to_value(RequestBody {
            package: "YWJj".to_owned(),
            package_id: PackageIdWire {
                domain: "quire.package.semantic/v2",
                algorithm: "sha256",
                digest: "0".repeat(64),
            },
            deadline_ms: 100,
            items: vec![item],
            dependencies: None,
            provision: None,
            memory: None,
        })
        .expect("typed request serializes");

        assert_eq!(
            request["items"][0]["extent"]["bounds"][0]["bound"]["maximum"],
            4
        );
        assert_eq!(
            request["items"][0]["temporal"]["subject"]["proof_bounds"][0]["bound"]["maximum"],
            4
        );
        assert!(request.get("domains").is_none());
        assert!(request.get("limits").is_none());
    }
}
