// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed driver inputs for QSpec's backend-provider/v1 envelope.
//!
//! This module projects the path-addressed FR-331 contract into driver types;
//! it does not carry a copy of the schema or its vectors. Canonical descriptor
//! bytes are retained so optional manifest members survive admission and can
//! be included unchanged in a later request's registry snapshot.

use std::collections::BTreeMap;
use std::sync::Arc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use qsl_foundation::diagnostic::{category_of, CatalogCode, Category};
use qsl_package::EmittedPackage;
use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin};
use quire_driver::ProviderManifest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

const CONTRACT_VERSION: &str = "quire.backend-provider/v1";
const CAPABILITY_VOCABULARY: &str = "quire.capability-kind/v1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProviderDescriptorShape {
    backend: String,
    advertises: Vec<AdvertisedPair>,
    #[serde(default, deserialize_with = "deserialize_present")]
    domains: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    options: Option<BTreeMap<String, String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    bounds: Option<BTreeMap<String, u64>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    obligation_forms: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    reductions: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    evidence_kinds: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    fairness_kinds: Option<Vec<String>>,
}

fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdvertisedPair {
    kind: Option<String>,
    mode: Option<String>,
}

fn invalid_member(field: &'static str, detail: impl Into<String>) -> ManifestAdmissionError {
    ManifestAdmissionError::InvalidMember {
        field: field.to_owned(),
        detail: detail.into(),
    }
}

fn validate_shape_set(
    field: &'static str,
    values: Option<&[String]>,
    allowed: &[&str],
) -> Result<(), ManifestAdmissionError> {
    let Some(values) = values else {
        return Ok(());
    };
    if values.is_empty() {
        return Err(invalid_member(field, "must contain at least one value"));
    }
    let mut seen = std::collections::BTreeSet::new();
    if values
        .iter()
        .any(|value| !allowed.contains(&value.as_str()) || !seen.insert(value))
    {
        return Err(invalid_member(
            field,
            "contains an unknown or duplicate value",
        ));
    }
    Ok(())
}

fn validate_manifest(shape: &ProviderDescriptorShape) -> Result<(), ManifestAdmissionError> {
    if shape.advertises.is_empty() {
        return Err(invalid_member("advertises", "must not be empty"));
    }
    let mut advertised = std::collections::BTreeSet::new();
    if shape
        .advertises
        .iter()
        .any(|pair| !advertised.insert((&pair.kind, &pair.mode)))
    {
        return Err(invalid_member("advertises", "contains a duplicate pair"));
    }
    // Domain vocabulary and admission refusal belong to QSL-654. Keep the
    // typed value for QSL instead of pre-refusing it in the driver.
    if shape.options.as_ref().is_some_and(BTreeMap::is_empty) {
        return Err(invalid_member("options", "must not be empty when present"));
    }
    if let Some(bounds) = &shape.bounds {
        if bounds.is_empty()
            || bounds.iter().any(|(name, value)| {
                !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                    || !name
                        .bytes()
                        .skip(1)
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                    || *value > MAX_SAFE_INTEGER
            })
        {
            return Err(invalid_member(
                "bounds",
                "contains an invalid setting name or unsafe integer",
            ));
        }
    }
    // QSL-654 owns evidence-kind admission/refusal. Preserve that field
    // without applying a driver-side vocabulary rule.
    for (field, values, allowed) in [
        (
            "obligation_forms",
            shape.obligation_forms.as_deref(),
            &[
                "unbounded-quantification",
                "induction",
                "compositional-discharge",
                "mathematical-integers",
            ][..],
        ),
        (
            "reductions",
            shape.reductions.as_deref(),
            &["symmetry", "partial-order", "constraint"][..],
        ),
        (
            "fairness_kinds",
            shape.fairness_kinds.as_deref(),
            &["weak", "strong"][..],
        ),
    ] {
        validate_shape_set(field, values, allowed)?;
    }
    Ok(())
}

/// The FR-331 manifest-stage body received in the plugin's hello.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestStage {
    /// FR-331 contract identity.
    pub contract_version: String,
    /// FR-290 capability-vocabulary identity.
    #[serde(default)]
    pub capability_vocabulary: Option<String>,
    /// The plugin's losslessly retained provider-manifest descriptor.
    pub manifest: Vec<Arc<Value>>,
}

/// One successfully admitted process manifest and the exact wire snapshot
/// retained for later FR-331 requests.
#[derive(Clone, Debug)]
pub struct AdmittedManifest {
    descriptor: BackendDescriptor,
    manifest: ProviderManifest,
}

impl AdmittedManifest {
    /// The descriptor admitted once by QSL for this process hello.
    #[must_use]
    pub fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    /// The single driver-owned manifest carrier, including all optional
    /// FR-331 members. Its canonical wire bytes are a fresh retained encoding
    /// of the admitted hello value.
    #[must_use]
    pub fn manifest(&self) -> &ProviderManifest {
        &self.manifest
    }
}

/// An invalid FR-331 hello body or a typed QSL registration refusal.
#[derive(Debug, thiserror::Error)]
pub enum ManifestAdmissionError {
    /// The manifest-stage envelope identity did not match FR-331.
    #[error("plugin hello has an unsupported FR-331 contract identity")]
    ContractIdentity,
    /// The manifest-stage capability vocabulary is not recognized by QSL.
    #[error("{0}")]
    CapabilityVocabulary(CatalogCode),
    /// A provider hello did not carry exactly one descriptor.
    #[error("plugin hello must carry exactly one provider descriptor")]
    DescriptorCount,
    /// A descriptor member violated FR-331 wire shape or type requirements.
    #[error("plugin hello manifest has an invalid {field} member: {detail}")]
    InvalidMember {
        /// The member whose type or shape did not satisfy FR-331.
        field: String,
        /// Parser or semantic-validation detail for the rejected member.
        detail: String,
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
    /// The digest and retained bytes use RFC 8785. QSL owns identity,
    /// capability, mode and duplicate admission; this method neither
    /// normalizes labels nor assigns an origin from wire data. Domain and
    /// evidence-label admission is pending QSL-654; these members are retained
    /// without driver-side vocabulary refusal.
    pub fn admit_process(
        &self,
        max_frame_bytes: u64,
    ) -> Result<AdmittedManifest, ManifestAdmissionError> {
        if self.contract_version != CONTRACT_VERSION {
            return Err(ManifestAdmissionError::ContractIdentity);
        }
        if self.capability_vocabulary.as_deref() != Some(CAPABILITY_VOCABULARY) {
            let code = CatalogCode::new("invalid_capability", "unknown-vocabulary");
            debug_assert_eq!(category_of(&code), Some(Category::Refusal));
            return Err(ManifestAdmissionError::CapabilityVocabulary(code));
        }
        let [wire] = self.manifest.as_slice() else {
            return Err(ManifestAdmissionError::DescriptorCount);
        };
        let shape: ProviderDescriptorShape = serde_path_to_error::deserialize((**wire).clone())
            .map_err(|source| ManifestAdmissionError::InvalidMember {
                field: source.path().to_string(),
                detail: source.inner().to_string(),
            })?;
        validate_manifest(&shape)?;
        let limits = quire_canonical::Limits::new(max_frame_bytes);
        let mut sink = (Vec::new(), Sha256::new());
        quire_canonical::encode(&mut sink, &**wire, limits)?;
        let (canonical_wire, hasher) = sink;
        let digest: [u8; 32] = hasher.finalize().into();
        // The checked-in QSL API does not yet accept domains/evidence kinds.
        // Preserve them in the DTO; pass them to admission when QSL-654 adds
        // its domain- and evidence-aware signature.
        let descriptor = BackendDescriptor::admit(
            Candidate::new(BackendId::from_wire(&shape.backend)?),
            ProviderOrigin::Process,
            ManifestDigest::from_digest(digest),
            shape
                .advertises
                .iter()
                .map(|pair| (pair.kind.as_deref(), pair.mode.as_deref())),
        )?;
        Ok(AdmittedManifest {
            descriptor,
            manifest: ProviderManifest {
                identity: shape.backend,
                manifest_digest: ManifestDigest::from_digest(digest),
                advertises: shape
                    .advertises
                    .iter()
                    .map(|pair| (pair.kind.clone(), pair.mode.clone()))
                    .collect(),
                domains: shape.domains,
                wire: Some(Arc::<[u8]>::from(canonical_wire)),
            },
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

/// A caller value cannot be represented exactly by the FR-331 JSON integer contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RequestStageError {
    /// FR-331 PositiveCount requires a nonzero safe integer deadline.
    #[error("deadline_ms must be in 1..=9007199254740991")]
    Deadline,
    /// FR-331 Count requires a safe non-negative request index.
    #[error("request_index must be in 0..=9007199254740991")]
    RequestIndex,
    /// A nested bound is outside FR-331's safe JSON integer range.
    #[error("request bounds must use safe FR-331 JSON integers")]
    Bound,
    /// FR-331 requires at least one retained manifest and one routed item.
    #[error("a process request requires at least one manifest and one item")]
    EmptyRequest,
    /// A retained manifest snapshot could not be reconstructed as JSON.
    #[error("a retained FR-331 manifest descriptor is invalid")]
    ManifestDescriptor,
}

impl RequestStage {
    /// Build the wire request from an emitted package and typed routed inputs.
    ///
    /// `package_id` is copied from the emitter's `EmittedPackage`; the driver
    /// never recomputes package identity from the bytes. `manifest` comes from
    /// retained decoded provider-manifest DTOs, not from QSL registry
    /// descriptors, which intentionally omit optional FR-331 members.
    pub fn from_emitted(
        package: &EmittedPackage,
        manifest: &[ProviderManifest],
        deadline_ms: u64,
        items: Vec<RequestItem>,
    ) -> Result<Self, RequestStageError> {
        if deadline_ms == 0 || deadline_ms > MAX_SAFE_INTEGER {
            return Err(RequestStageError::Deadline);
        }
        if items
            .iter()
            .any(|item| item.request_index > MAX_SAFE_INTEGER)
        {
            return Err(RequestStageError::RequestIndex);
        }
        if items.iter().any(|item| !item_has_safe_bounds(item)) {
            return Err(RequestStageError::Bound);
        }
        if manifest.is_empty() || items.is_empty() {
            return Err(RequestStageError::EmptyRequest);
        }
        let package_id = package.package_id().record();
        Ok(Self {
            contract_version: CONTRACT_VERSION,
            capability_vocabulary: CAPABILITY_VOCABULARY,
            manifest: manifest
                .iter()
                .map(manifest_descriptor)
                .collect::<Result<Vec<_>, _>>()?,
            request: RequestBody {
                package: encode_package_bytes(package.bytes()),
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
        })
    }
}

fn encode_package_bytes(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

fn manifest_descriptor(manifest: &ProviderManifest) -> Result<Value, RequestStageError> {
    if let Some(wire) = &manifest.wire {
        let value: Value =
            serde_json::from_slice(wire).map_err(|_| RequestStageError::ManifestDescriptor)?;
        if value.get("backend").and_then(Value::as_str) != Some(manifest.identity.as_str()) {
            return Err(RequestStageError::ManifestDescriptor);
        }
        return Ok(value);
    }
    Err(RequestStageError::ManifestDescriptor)
}

fn item_has_safe_bounds(item: &RequestItem) -> bool {
    let safe = |bounds: &[ProofBound]| {
        bounds.iter().all(|bound| match &bound.bound {
            FiniteBound::Cardinality { maximum } | FiniteBound::Depth { maximum } => {
                *maximum <= MAX_SAFE_INTEGER
            }
            FiniteBound::IntegerRange { .. } => true,
        })
    };
    let extent_safe = match &item.extent {
        Some(Extent::Bounded { bounds }) => safe(bounds),
        Some(Extent::Unbounded { .. }) | None => true,
    };
    extent_safe
        && item.temporal.as_ref().is_none_or(|temporal| {
            safe(&temporal.subject.proof_bounds)
                && temporal
                    .subject
                    .initial_states
                    .iter()
                    .all(|r| valid_hex_digest(&r.digest))
                && temporal
                    .subject
                    .universes
                    .iter()
                    .all(|r| valid_hex_digest(&r.digest))
        })
}

fn valid_hex_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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
        domains: Vec<ExtentDomain>,
        /// Whether a finite bound is available for these domains.
        finite_bound_available: bool,
    },
}

/// One FR-331 unbounded domain selector.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtentDomain {
    /// The checked-package domain key.
    pub domain: DomainKey,
    /// The domain's closed classification.
    pub kind: ExtentDomainKind,
}

/// FR-331 extent-domain kind vocabulary.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExtentDomainKind {
    /// Collection domain.
    Collection,
    /// Population domain.
    Population,
    /// Integer domain.
    Integer,
    /// Recursive domain.
    Recursive,
    /// Quantity domain.
    Quantity,
    /// Loop domain.
    Loop,
    /// Infinite trace domain.
    InfiniteTrace,
}

/// One FR-331 ADR-014 domain substitution.
#[derive(Clone, Debug, Serialize)]
pub struct ProofBound {
    /// The bounded QSpec domain key.
    pub domain: DomainKey,
    /// The finite bound supplied by the request.
    pub bound: FiniteBound,
}

/// QSpec FR-331 domain key.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DomainKey {
    /// A node domain with a projection path.
    Node { node: String, path: Vec<u32> },
    /// Finite population domain.
    Population { member_type: String, ordinal: u32 },
}

/// QSpec finite proof-bound shape.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FiniteBound {
    /// Cardinality bound.
    Cardinality { maximum: u64 },
    /// Integer range bound represented by canonical integer strings.
    IntegerRange { lower: String, upper: String },
    /// Structural depth bound.
    Depth { maximum: u64 },
}

/// Temporal FR-331 request members.
#[derive(Clone, Debug, Serialize)]
pub struct TemporalRequest {
    /// Temporal subject and its proof bounds.
    pub subject: TemporalSubject,
    /// Formula or deadlock-freedom request kind.
    pub item_kind: TemporalItemKind,
    /// Required fairness constraints (empty for deadlock-freedom).
    pub fairness: Vec<Fairness>,
}

/// Temporal request subject with proof bounds at the FR-331 path.
#[derive(Clone, Debug, Serialize)]
pub struct TemporalSubject {
    /// Initial state document references.
    pub initial_states: Vec<ProvisionRef>,
    /// Universe document references.
    pub universes: Vec<ProvisionRef>,
    /// Proof bounds at `temporal.subject.proof_bounds`.
    pub proof_bounds: Vec<ProofBound>,
}

/// QSpec temporal item kind.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalItemKind {
    Formula,
    DeadlockFreedom,
}

/// QSpec temporal fairness constraint.
#[derive(Clone, Debug, Serialize)]
pub struct Fairness {
    /// Weak or strong fairness.
    pub kind: FairnessKind,
    /// The operation subject to fairness.
    pub operation: String,
    /// Whole-operation or per-operation granularity.
    pub granularity: FairnessGranularity,
}

/// QSpec fairness kind.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FairnessKind {
    Weak,
    Strong,
}

/// QSpec fairness granularity.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FairnessGranularity {
    Whole,
    Each,
}

/// FR-331 byte-provision reference.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProvisionRef {
    /// FR-364's supported digest domain.
    pub digest_domain: ProvisionDigestDomain,
    /// Lowercase digest of the canonical document.
    pub digest: String,
}

/// QSpec provision digest-domain vocabulary.
#[derive(Clone, Debug, Serialize)]
pub enum ProvisionDigestDomain {
    /// RFC 8785 SHA-256 digest.
    #[serde(rename = "sha256-jcs")]
    Sha256Jcs,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hello(kind: Value, mode: Value) -> ManifestStage {
        ManifestStage {
            contract_version: CONTRACT_VERSION.to_owned(),
            capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
            manifest: vec![serde_json::json!({
                "backend": "agent-ix/provider/test",
                "advertises": [{ "kind": kind, "mode": mode }],
                "domains": ["integer"],
                "bounds": { "max_states": 8 }
            })
            .into()],
        }
    }

    /// Trace: FR-005-AC-15.
    #[test]
    fn hello_admits_process_descriptor_and_retains_full_manifest() {
        let stage = hello(
            Value::String("operation-contract".to_owned()),
            Value::String("bounded".to_owned()),
        );
        let source_wire = Arc::clone(&stage.manifest[0]);
        let admitted = stage
            .admit_process(1024 * 1024)
            .expect("known labels are admitted");

        assert_eq!(admitted.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(
            admitted.descriptor().id().as_str(),
            "agent-ix/provider/test"
        );
        let retained: Value = serde_json::from_slice(
            admitted
                .manifest()
                .wire
                .as_deref()
                .expect("canonical wire DTO retained"),
        )
        .expect("retained descriptor is JSON");
        assert_eq!(retained, *source_wire);
        assert_eq!(
            admitted.manifest().domains.as_deref(),
            Some(["integer".to_owned()].as_slice())
        );
    }

    #[test]
    fn unsupported_capability_vocabulary_uses_qsl_typed_refusal() {
        for capability_vocabulary in [None, Some("quire.capability-kind/v0".to_owned())] {
            let mut stage = hello(
                Value::String("operation-contract".to_owned()),
                Value::String("bounded".to_owned()),
            );
            stage.capability_vocabulary = capability_vocabulary;
            let error = stage
                .admit_process(1024)
                .expect_err("unknown vocabulary refuses");
            let ManifestAdmissionError::CapabilityVocabulary(code) = error else {
                panic!("vocabulary mismatch must retain QSL's typed refusal: {error:?}");
            };
            assert_eq!(code.code(), "invalid_capability");
            assert_eq!(code.cause(), "unknown-vocabulary");
            assert_eq!(category_of(&code), Some(Category::Refusal));
        }
    }

    #[test]
    fn manifest_descriptor_checks_wire_shape_constraints() {
        let base = || {
            serde_json::json!({
                "backend": "agent-ix/provider/test",
                "advertises": [{"kind":"operation-contract", "mode":"bounded"}]
            })
        };
        let mut cases = Vec::new();
        let mut invalid = base();
        invalid["options"] = serde_json::json!({});
        cases.push((invalid, "options"));
        let mut invalid = base();
        invalid["options"] = Value::Null;
        cases.push((invalid, "options"));
        let mut invalid = base();
        invalid["bounds"] = serde_json::json!({"Max_states": 2});
        cases.push((invalid, "bounds"));
        let mut invalid = base();
        invalid["bounds"] = serde_json::json!({"max_states": MAX_SAFE_INTEGER + 1});
        cases.push((invalid, "bounds"));
        let mut invalid = base();
        invalid["unknown_member"] = Value::Bool(true);
        cases.push((invalid, "unknown_member"));
        let mut invalid = base();
        invalid["advertises"] = serde_json::json!([]);
        cases.push((invalid, "advertises"));
        for (wire, field) in cases {
            let stage = ManifestStage {
                contract_version: CONTRACT_VERSION.to_owned(),
                capability_vocabulary: Some(CAPABILITY_VOCABULARY.to_owned()),
                manifest: vec![Arc::new(wire)],
            };
            let error = stage
                .admit_process(4096)
                .expect_err("schema-invalid descriptor refuses");
            assert!(
                matches!(error, ManifestAdmissionError::InvalidMember { .. }),
                "{error:?}"
            );
            assert!(error.to_string().contains(field), "{error}");
        }
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
                .admit_process(1024 * 1024)
                .expect_err("QSL refuses malformed capability labels");
            let ManifestAdmissionError::Registration(refusal) = error else {
                panic!("malformed labels must be classified by QSL: {error:?}");
            };
            assert_eq!(refusal.cause().as_str(), invalid.2);
        }
    }

    /// Trace: FR-006-AC-6.
    #[test]
    fn package_encoding_uses_unpadded_base64url_for_special_bytes() {
        let bytes = [0xfb, 0xff];
        assert_ne!(bytes.len() % 3, 0);
        assert_eq!(
            base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes),
            "+/8"
        );
        assert_eq!(encode_package_bytes(&bytes), "-_8");
    }

    /// Trace: FR-006-AC-7, FR-006-AC-8, FR-006-AC-13.
    #[test]
    fn request_wire_keeps_package_bounds_and_temporal_proof_bounds_at_contract_paths() {
        let item = RequestItem {
            request_index: 3,
            kind: Some("operation-contract".to_owned()),
            claim: serde_json::json!({"node": {}, "occurrence": {}}),
            extent: Some(Extent::Bounded {
                bounds: vec![ProofBound {
                    domain: DomainKey::Population {
                        member_type: "member-node".to_owned(),
                        ordinal: 0,
                    },
                    bound: FiniteBound::Cardinality { maximum: 4 },
                }],
            }),
            backend: Some("agent-ix/provider/test".to_owned()),
            candidates: serde_json::json!({"type": "set", "backends": ["agent-ix/provider/test"]}),
            options: None,
            reductions: None,
            temporal: Some(TemporalRequest {
                subject: TemporalSubject {
                    initial_states: Vec::new(),
                    universes: Vec::new(),
                    proof_bounds: vec![ProofBound {
                        domain: DomainKey::Population {
                            member_type: "member-node".to_owned(),
                            ordinal: 0,
                        },
                        bound: FiniteBound::Cardinality { maximum: 4 },
                    }],
                },
                item_kind: TemporalItemKind::Formula,
                fairness: Vec::new(),
            }),
            probabilistic: None,
        };
        let package = crate::test_support::emitted_package_with_urlsafe_bytes();
        let retained_value = serde_json::json!({
            "backend":"agent-ix/provider/test",
            "advertises":[{"kind":"operation-contract", "mode":"bounded"}],
            "domains":["integer"]
        });
        let retained_wire = Arc::<[u8]>::from(
            quire_canonical::to_vec(&retained_value, quire_canonical::Limits::new(4096))
                .expect("manifest canonicalizes"),
        );
        let manifests = [
            ProviderManifest {
                identity: "agent-ix/provider/test".to_owned(),
                manifest_digest: ManifestDigest::from_digest([5; 32]),
                advertises: vec![(
                    Some("operation-contract".to_owned()),
                    Some("bounded".to_owned()),
                )],
                domains: Some(vec!["integer".to_owned()]),
                wire: Some(Arc::clone(&retained_wire)),
            },
            ProviderManifest {
                identity: "agent-ix/provider/linked".to_owned(),
                manifest_digest: ManifestDigest::from_digest([6; 32]),
                advertises: vec![(
                    Some("operation-contract".to_owned()),
                    Some("bounded".to_owned()),
                )],
                domains: Some(vec!["recursive".to_owned()]),
                wire: None,
            },
        ];
        let manifests = crate::manifest_snapshot::retain_wire_bytes(manifests.to_vec())
            .expect("linked manifest canonicalizes before adapter serialization");
        let request_stage =
            RequestStage::from_emitted(&package, &manifests, 100, vec![item.clone()])
                .expect("the real emitted identity and safe request values serialize");
        let package_id = package.package_id().record();
        let request = serde_json::to_value(request_stage).expect("typed request serializes");

        assert_eq!(
            request["request"]["package"],
            URL_SAFE_NO_PAD.encode(package.bytes())
        );
        assert_eq!(
            request["request"]["package_id"]["domain"],
            package_id.domain().as_str()
        );
        assert_eq!(request["request"]["package_id"]["algorithm"], "sha256");
        assert_eq!(request["request"]["package_id"]["digest"], package_id.hex());
        assert_eq!(request["request"]["deadline_ms"], 100);
        assert_eq!(request["manifest"][0], retained_value);
        assert_eq!(
            request["manifest"][1]["backend"],
            "agent-ix/provider/linked"
        );
        assert_eq!(
            request["manifest"][1]["domains"],
            serde_json::json!(["recursive"])
        );
        assert_eq!(
            request["manifest"][1]["advertises"][0]["kind"],
            "operation-contract"
        );
        assert!(request["manifest"][1].get("options").is_none());

        assert_eq!(
            request["request"]["items"][0]["extent"]["bounds"][0]["bound"]["maximum"],
            4
        );
        assert_eq!(
            request["request"]["items"][0]["temporal"]["subject"]["proof_bounds"][0]["bound"]
                ["maximum"],
            4
        );
        assert!(matches!(
            RequestStage::from_emitted(&package, &manifests, 100, Vec::new()),
            Err(RequestStageError::EmptyRequest)
        ));
        let mut missing_wire = manifests[1].clone();
        missing_wire.wire = None;
        assert!(matches!(
            RequestStage::from_emitted(&package, &[missing_wire], 100, vec![item.clone()]),
            Err(RequestStageError::ManifestDescriptor)
        ));
        assert!(matches!(
            RequestStage::from_emitted(&package, &[], 100, vec![item.clone()]),
            Err(RequestStageError::EmptyRequest)
        ));
        assert!(RequestStage::from_emitted(&package, &[], 0, Vec::new()).is_err());
        assert!(
            RequestStage::from_emitted(&package, &[], MAX_SAFE_INTEGER + 1, Vec::new()).is_err()
        );
    }
}
