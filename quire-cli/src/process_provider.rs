// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generic `quire-plugin-host` adapter for the driver's process-provider seam.

use std::convert::Infallible;
use std::ffi::OsString;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use qsl_replay::CallRequirements;
use qsl_route::request::RequirementItem;
use qsl_route::{BackendDescriptor, ProviderOrigin};
use quire_driver::{
    ProcessProvider, ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession,
    ProviderItemResult, ProviderResults, RoutedItemOutput,
};
use quire_exact::Cancel;
use quire_plugin_host::{
    FrameLimits, PluginError, PluginSession, RunLimits, SessionFinished, SessionStart,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

/// Closed FR-331 result values retained before a QSL mapping is available.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Fr331ResultKind {
    Proved,
    Refuted,
    Tested,
    Measured,
    Inconclusive,
    Unsupported,
    Declined,
    Incomplete,
    Failed,
}

/// Closed FR-331 negotiation dispositions retained without terminal mapping.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Fr331DispositionKind {
    Supported,
    RequiresBound,
    Unsupported,
    InvalidRequest,
}

/// One result or disposition for a routed item, plus its exact received envelope.
#[derive(Clone, Debug)]
pub(crate) struct ProcessProviderOutcome {
    pub(crate) item: ProcessProviderItemOutcome,
    pub(crate) response: Arc<serde_json::Value>,
}

/// The call-selected QSL items, verified against the exact package and driver
/// request that produced the routed generation.
pub(crate) struct CallItemProjection {
    pub(crate) package_name: String,
    pub(crate) items: Vec<RequirementItem>,
    pub(crate) routed: Vec<RoutedItemOutput>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum CallItemProjectionError {
    #[error("QSL call requirements and driver generation came from different packages")]
    PackageMismatch,
    #[error("a call requirement request index does not identify exactly one driver QSL item")]
    RequestIndex,
    #[error("a call requirement occurrence differs from the driver QSL item at its index")]
    Occurrence,
    #[error("QSL returned a duplicate call requirement request index")]
    DuplicateIndex,
}

/// Join QSL's call-aware members to the exact retained driver request items.
/// The package name is checked before using request indices, then every selected
/// occurrence must match; package-wide items outside the selected call are
/// excluded from the returned sidecar.
pub(crate) fn project_call_items(
    package_name: &str,
    requirements: &CallRequirements,
    driven: &quire_driver::Driven,
) -> Result<CallItemProjection, CallItemProjectionError> {
    if package_name != driven.emitted().name() {
        return Err(CallItemProjectionError::PackageMismatch);
    }

    let mut indices = std::collections::BTreeSet::new();
    let mut items = Vec::with_capacity(requirements.members().len());
    for member in requirements.members() {
        let index = member.request_index().get();
        if !indices.insert(index) {
            return Err(CallItemProjectionError::DuplicateIndex);
        }
        let mut matches = driven
            .request_items
            .iter()
            .filter(|item| item.index().get() == index);
        let item = matches
            .next()
            .ok_or(CallItemProjectionError::RequestIndex)?;
        if matches.next().is_some() {
            return Err(CallItemProjectionError::RequestIndex);
        }
        if item.occurrence() != member.occurrence() {
            return Err(CallItemProjectionError::Occurrence);
        }
        items.push(item.clone());
    }
    items.sort_by_key(|item| item.index().get());
    let selected_indices = indices;
    let mut routed_indices = std::collections::BTreeSet::new();
    let mut routed = Vec::new();
    for generated in &driven.generation().items {
        if selected_indices.contains(&generated.request_index) {
            if !routed_indices.insert(generated.request_index) {
                return Err(CallItemProjectionError::DuplicateIndex);
            }
            routed.push(generated.clone());
        }
    }
    Ok(CallItemProjection {
        package_name: package_name.to_owned(),
        items,
        routed,
    })
}

/// Build one request body from retained driver and QSL values.
pub(crate) fn encode_fr331_request(
    request: ProcessProviderRequest<'_>,
) -> Result<crate::backend_provider::RequestStage, String> {
    crate::backend_provider::RequestStage::from_process_request(&request)
        .map_err(|error| error.to_string())
}

/// Validate an FR-331 outcome against the one admitted hello identity.
pub(crate) fn validate_fr331_response(
    manifest: &crate::backend_provider::ManifestStage,
    response: &serde_json::Value,
) -> Result<(), String> {
    let admitted_identity = manifest
        .manifest
        .first()
        .and_then(|descriptor| descriptor.get("backend"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "admitted FR-331 hello has no backend identity".to_owned())?;
    let results = retain_fr331_response(response.clone())?;
    if results
        .items
        .iter()
        .any(|item| item.backend.identity != admitted_identity)
    {
        return Err("FR-331 outcome backend differs from the admitted provider".to_owned());
    }
    let mut seen = std::collections::BTreeSet::new();
    if results
        .items
        .iter()
        .any(|item| !seen.insert(item.request_index))
    {
        return Err("FR-331 outcome repeats a request index".to_owned());
    }
    Ok(())
}

/// Retain the full FR-331 outcome without selecting QSL terminals.
pub(crate) fn convert_fr331_response(
    response: serde_json::Value,
) -> Result<ProviderResults<ProcessProviderOutcome, String>, String> {
    retain_fr331_response(response)
}

/// An FR-331 item state kept distinct until the QSL mapping API is available.
#[derive(Clone, Debug)]
pub(crate) enum ProcessProviderItemOutcome {
    Result {
        capability_kind: String,
        kind: Fr331ResultKind,
        item: serde_json::Value,
    },
    Disposition {
        capability_kind: String,
        kind: Fr331DispositionKind,
        item: serde_json::Value,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fr331ResponseItem {
    request_index: u64,
    backend: String,
    kind: String,
    #[serde(default)]
    result: Option<Fr331ResultKind>,
    #[serde(default)]
    disposition: Option<Fr331DispositionKind>,
    #[serde(flatten)]
    payload: serde_json::Map<String, serde_json::Value>,
}

fn retain_fr331_response(
    response: serde_json::Value,
) -> Result<ProviderResults<ProcessProviderOutcome, String>, String> {
    let contract = response
        .get("contract_version")
        .and_then(serde_json::Value::as_str);
    let vocabulary = response
        .get("capability_vocabulary")
        .and_then(serde_json::Value::as_str);
    if contract != Some("quire.backend-provider/v1")
        || vocabulary != Some("quire.capability-kind/v1")
    {
        return Err("plugin result has an unsupported FR-331 identity".to_owned());
    }
    let response = Arc::new(response);
    let mut items = Vec::new();
    for (member, is_result) in [("results", true), ("dispositions", false)] {
        let entries = response
            .get(member)
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("plugin result is missing the `{member}` array"))?;
        for value in entries {
            let parsed: Fr331ResponseItem = serde_json::from_value(value.clone())
                .map_err(|error| format!("invalid FR-331 {member} item: {error}"))?;
            if parsed.result.is_some() != is_result || parsed.disposition.is_some() == is_result {
                return Err(format!("FR-331 {member} item has the wrong state member"));
            }
            let request_index = usize::try_from(parsed.request_index)
                .map_err(|_| "FR-331 request index does not fit this platform".to_owned())?;
            let wire = serde_json::Value::Object(parsed.payload);
            let outcome = if is_result {
                ProcessProviderItemOutcome::Result {
                    capability_kind: parsed.kind,
                    kind: parsed
                        .result
                        .ok_or_else(|| "missing result value".to_owned())?,
                    item: wire,
                }
            } else {
                ProcessProviderItemOutcome::Disposition {
                    capability_kind: parsed.kind,
                    kind: parsed
                        .disposition
                        .ok_or_else(|| "missing disposition value".to_owned())?,
                    item: wire,
                }
            };
            items.push(ProviderItemResult {
                request_index,
                backend: quire_driver::Candidate {
                    identity: parsed.backend,
                },
                result: Ok(ProcessProviderOutcome {
                    item: outcome,
                    response: Arc::clone(&response),
                }),
            });
        }
    }
    items.sort_by_key(|item| item.request_index);
    Ok(ProviderResults { items })
}

type ProviderWireTypes<M, Q, R, F, A, V, I, E> = fn() -> (M, Q, R, F, A, V, I, E);
type SessionWireTypes<Q, R, V, I, E> = fn() -> (Q, R, V, I, E);

/// Why hello admission did not provide a caller-admitted process descriptor.
#[derive(Debug, thiserror::Error)]
pub enum AdmissionFailure<E> {
    /// The caller's QSL admission boundary rejected the manifest.
    #[error("caller rejected plugin hello: {0}")]
    Caller(E),
    /// The returned descriptor did not carry the caller-supplied Process origin.
    #[error("caller-admitted descriptor does not have Process origin")]
    WrongOrigin,
}

/// Error returned by the generic host adapter.
#[derive(Debug, thiserror::Error)]
pub enum AdapterError<A, V, F> {
    /// Starting the explicit-path process or admitting its hello failed.
    #[error("plugin session start failed: {0}")]
    Start(PluginError<AdmissionFailure<A>, Infallible, F>),
    /// The typed FR-331 request could not be projected from retained driver inputs.
    #[error("plugin request projection failed: {0}")]
    Encode(V),
    /// The FR-331 result could not be retained in the shared provider carrier.
    #[error("plugin result conversion failed: {0}")]
    Convert(V),
    /// Sending a request, validating its result, or process cleanup failed.
    #[error("plugin request failed: {0}")]
    Request(PluginError<AdmissionFailure<A>, V, F>),
    /// Graceful shutdown or diagnostic capture failed.
    #[error("plugin shutdown failed: {0}")]
    Shutdown(PluginError<AdmissionFailure<A>, Infallible, F>),
    /// The host reported successful admission without invoking its admitter.
    #[error("plugin host completed admission without returning a descriptor")]
    MissingAdmittedDescriptor,
}

/// A process provider adapter parameterized by caller-owned FR-331 types and
/// conversion functions. This type defines no manifest, request, result or
/// fault wire schema.
pub struct HostProcessProvider<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert> {
    admit: Arc<Admit>,
    encode: Arc<Encode>,
    validate: Arc<Validate>,
    convert: Arc<Convert>,
    signals: crate::signals::SignalContext,
    // This function-pointer marker keeps the wire types covariant without
    // imposing their Send/Sync bounds on the adapter itself.
    #[allow(clippy::type_complexity)]
    _wire_types: PhantomData<ProviderWireTypes<M, Q, R, F, A, V, I, E>>,
}

impl<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert>
    HostProcessProvider<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert>
{
    /// Build the adapter from QSL admission and caller-owned typed FR-331
    /// request/result conversions.
    pub fn new(
        signals: crate::signals::SignalContext,
        admit: Admit,
        encode: Encode,
        validate: Validate,
        convert: Convert,
    ) -> Self {
        Self {
            admit: Arc::new(admit),
            encode: Arc::new(encode),
            validate: Arc::new(validate),
            convert: Arc::new(convert),
            signals,
            _wire_types: PhantomData,
        }
    }
}

impl<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert> ProcessProvider
    for HostProcessProvider<M, Q, R, F, A, V, I, E, Admit, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
    Admit: Fn(&M) -> Result<(BackendDescriptor, quire_driver::ProviderManifest), A> + 'static,
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Result<Q, V> + 'static,
    Validate: Fn(&M, &R) -> Result<(), V> + 'static,
    Convert: Fn(R) -> Result<ProviderResults<I, E>, V> + 'static,
{
    type Session = HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert>;
    type Error = AdapterError<A, V, F>;

    fn start(
        &self,
        executable: &Path,
        arguments: &[OsString],
        limits: ProcessProviderLimits,
        cancel: &Cancel,
    ) -> Result<Self::Session, Self::Error> {
        let active = self.signals.activate(cancel.clone());
        let mut admitted = None;
        let host = PluginSession::start(
            SessionStart {
                executable,
                arguments,
                limits: RunLimits {
                    frames: FrameLimits {
                        max_frame_bytes: limits.max_frame_bytes,
                        max_total_bytes: limits.max_total_bytes,
                    },
                    max_stderr_bytes: limits.max_stderr_bytes,
                    cancellation_grace: limits.cancellation_grace,
                },
                cancel,
            },
            |manifest| {
                let (descriptor, retained_manifest) =
                    (self.admit)(manifest).map_err(AdmissionFailure::Caller)?;
                if descriptor.origin() != ProviderOrigin::Process {
                    return Err(AdmissionFailure::WrongOrigin);
                }
                admitted = Some((descriptor, retained_manifest));
                Ok(())
            },
        )
        .map_err(AdapterError::Start)?;
        let (descriptor, manifest) = admitted.ok_or(AdapterError::MissingAdmittedDescriptor)?;
        Ok(HostProcessSession {
            host,
            descriptor,
            manifest,
            signals: self.signals.clone(),
            encode: Arc::clone(&self.encode),
            validate: Arc::clone(&self.validate),
            convert: Arc::clone(&self.convert),
            active,
            _wire_types: PhantomData,
        })
    }
}

/// An admitted host session implementing the driver-owned process seam.
pub struct HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert> {
    host: PluginSession<M, F, AdmissionFailure<A>>,
    descriptor: BackendDescriptor,
    manifest: quire_driver::ProviderManifest,
    signals: crate::signals::SignalContext,
    encode: Arc<Encode>,
    validate: Arc<Validate>,
    convert: Arc<Convert>,
    active: crate::signals::ActiveSession,
    _wire_types: PhantomData<SessionWireTypes<Q, R, V, I, E>>,
}

impl<M, Q, R, F, A, V, I, E, Encode, Validate, Convert> ProcessProviderSession
    for HostProcessSession<M, Q, R, F, A, V, I, E, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Result<Q, V>,
    Validate: Fn(&M, &R) -> Result<(), V>,
    Convert: Fn(R) -> Result<ProviderResults<I, E>, V>,
{
    type Error = AdapterError<A, V, F>;
    type ItemResult = I;
    type ItemFailure = E;
    type Diagnostics = SessionFinished<M>;

    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    fn manifest(&self) -> &quire_driver::ProviderManifest {
        &self.manifest
    }

    fn invoke(
        &mut self,
        request: ProcessProviderRequest<'_>,
    ) -> Result<ProviderResults<Self::ItemResult, Self::ItemFailure>, Self::Error> {
        let cancel = request.cancel;
        let _request_active = self.signals.activate(cancel.clone());
        let body = (self.encode)(request).map_err(AdapterError::Encode)?;
        let result = self
            .host
            .request(&body, cancel, |manifest, result| {
                (self.validate)(manifest, result)
            })
            .map_err(AdapterError::Request)?;
        (self.convert)(result).map_err(AdapterError::Convert)
    }

    fn shutdown(self) -> Result<Self::Diagnostics, Self::Error> {
        let Self { host, active, .. } = self;
        let diagnostics = host.shutdown().map_err(AdapterError::Shutdown)?;
        drop(active);
        Ok(diagnostics)
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "an adapter fixture mismatch is the test failure report"
)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use qsl_foundation::BackendId;
    use qsl_foundation::SourceIdentity;
    use qsl_replay::spine::{default_accounting, Call, CallArgument, DependencyInput, SpineLimits};
    use qsl_route::{descriptor_from_manifest, BackendDescriptor, Candidate, ProviderOrigin};
    use quire_driver::{
        build_linked_providers, drive, CompileInput, GenerationInput, ProcessProvider,
        ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession, ProviderItemResult,
        ProviderManifest, ProviderRequest, ProviderResults, QslCg,
    };
    use quire_exact::{Cancel, Integer, Value};
    use serde::Deserialize;

    use super::{project_call_items, HostProcessProvider};

    /// Trace: FR-006-AC-1, FR-006-AC-5..FR-006-AC-8, FR-006-AC-13; pair the selected QSL call
    /// with the driver generation by package, request index and occurrence before writing FR-331.
    #[test]
    fn configured_call_projection_sends_only_its_correlated_routed_items() {
        use crate::backend_provider::ManifestStage;
        let bytes = format!(
            "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
             function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure {{ x = y }}\n\
             function negate using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n",
            "quire.value.complete/v1"
        );
        let source = SourceIdentity::new("agent-ix", "configured-call");
        let path = "configured-call.native";
        let packages = std::collections::BTreeMap::new();
        let dependencies = DependencyInput::default();
        let limits = SpineLimits::default();
        let cancel = Cancel::new();
        let call = Call {
            function: "eq".to_owned(),
            arguments: vec![
                CallArgument {
                    parameter: "x".to_owned(),
                    value: Value::Integer(Integer::from(2_i64)),
                },
                CallArgument {
                    parameter: "y".to_owned(),
                    value: Value::Integer(Integer::from(3_i64)),
                },
            ],
            accounting: default_accounting(1_000_000),
            supplied: Default::default(),
            backend: Default::default(),
        };
        let objects = qsl_semantics::model::object_environment::ObjectEnvironment::default();
        let call_run = qsl_replay::run_with_requirements(
            source.clone(),
            "configured-call".to_owned(),
            path,
            bytes.as_bytes(),
            &packages,
            &dependencies,
            limits,
            &call,
            &objects,
            &cancel,
        )
        .expect("the selected call completes with its QSL requirement members");

        let hello = ManifestStage {
            contract_version: "quire.backend-provider/v1".to_owned(),
            capability_vocabulary: Some("quire.capability-kind/v1".to_owned()),
            manifest: vec![Arc::new(serde_json::json!({
                "backend": "agent-ix/provider/fr331-fixture",
                "advertises": [{ "kind": "value-validity", "mode": "bounded" }],
                "domains": ["integer"]
            }))],
        };
        let admitted = hello
            .admit_process(4096)
            .expect("QSL admits the configured process manifest");
        let mut linked = build_linked_providers(&[]).expect("one registry is built");
        linked
            .register_admitted([(admitted.descriptor().clone(), admitted.manifest().clone())])
            .expect("the already admitted descriptor is registered once");
        let driven = drive(
            &QslCg,
            CompileInput {
                source,
                package_name: "configured-call",
                path,
                bytes: bytes.as_bytes(),
                packages: &packages,
                limits,
                cancel: cancel.clone(),
            },
            linked.registry(),
            None,
            GenerationInput::default(),
        )
        .expect("the same source is negotiated and generated once by the driver");

        assert_eq!(call_run.package_name().as_str(), driven.emitted().name());
        assert_eq!(call_run.requirements().members().len(), 1);
        assert_eq!(
            driven.request_items.len(),
            2,
            "the package has two functions"
        );
        let projection = project_call_items(
            call_run.package_name().as_str(),
            call_run.requirements(),
            &driven,
        )
        .expect("same-package call items join the retained driver request");
        assert_eq!(projection.items.len(), 1);
        assert_eq!(projection.routed.len(), 1);
        let member = &call_run.requirements().members()[0];
        assert_eq!(projection.items[0].index(), member.request_index());
        assert_eq!(projection.items[0].occurrence(), member.occurrence());
        assert_eq!(
            projection.routed[0].request_index,
            member.request_index().get()
        );

        let manifest = linked.manifest_snapshot();
        let request = ProcessProviderRequest {
            emitted_package: driven.emitted(),
            deadline_ms: 4096,
            manifest: &manifest,
            provider_request: ProviderRequest::new(&projection.routed),
            qsl_items: &projection.items,
            cancel: &cancel,
        };
        let wire = serde_json::to_value(
            super::encode_fr331_request(request).expect("the selected call forms one request"),
        )
        .expect("FR-331 request serializes");
        let wire_items = wire["request"]["items"]
            .as_array()
            .expect("request items are an array");
        assert_eq!(wire_items.len(), 1, "unselected package items are excluded");
        assert_eq!(wire_items[0]["request_index"], member.request_index().get());
        assert_eq!(wire_items[0]["backend"], "agent-ix/provider/fr331-fixture");
    }

    #[derive(Debug, Deserialize)]
    struct Hello {
        identity: String,
        advertises: Vec<(Option<String>, Option<String>)>,
        #[serde(default)]
        domains: Option<Vec<String>>,
    }

    fn admit_hello(
        manifest: &Hello,
        origin: ProviderOrigin,
    ) -> Result<(BackendDescriptor, ProviderManifest), String> {
        let identity =
            BackendId::from_wire(&manifest.identity).map_err(|error| error.to_string())?;
        let descriptor = descriptor_from_manifest(
            Candidate::new(identity),
            origin,
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
            qsl_route::ManifestMembers {
                domains: manifest
                    .domains
                    .as_deref()
                    .map(|domains| domains.iter().map(String::as_str).collect::<Vec<_>>())
                    .as_deref(),
                ..qsl_route::ManifestMembers::default()
            },
        )
        .map_err(|error| format!("{error:?}"))?;
        let retained = ProviderManifest {
            identity: manifest.identity.clone(),
            advertises: manifest.advertises.clone(),
            domains: manifest.domains.clone(),
            bounds: None,
            wire: None,
        };
        Ok((descriptor, retained))
    }

    fn python_path() -> PathBuf {
        let python = std::env::split_paths(&std::env::var_os("PATH").expect("PATH is set"))
            .map(|directory| directory.join("python3"))
            .find(|candidate| candidate.is_file())
            .expect("the plugin fixture requires Python 3");
        assert!(python.is_absolute());
        python
    }

    fn fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../quire-plugin-host/tests/fixtures/echo-plugin.py")
    }

    fn backend_provider_fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/backend-provider-plugin.py")
    }

    fn wrong_result_fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/wrong-result-plugin.py")
    }

    fn limits() -> ProcessProviderLimits {
        ProcessProviderLimits {
            max_frame_bytes: 4096,
            max_total_bytes: 16_384,
            max_stderr_bytes: 1024,
            cancellation_grace: Duration::from_millis(500),
        }
    }

    /// Trace: FR-005-AC-6.
    #[test]
    fn host_adapter_returns_admitted_descriptor_and_shutdown_diagnostics() {
        let conversion_called = Arc::new(AtomicBool::new(false));
        let converted = Arc::clone(&conversion_called);
        let admissions = Arc::new(AtomicUsize::new(0));
        let admission_count = Arc::clone(&admissions);
        let provider = HostProcessProvider::<
            Hello,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            String,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            move |manifest: &Hello| {
                admission_count.fetch_add(1, Ordering::Relaxed);
                admit_hello(manifest, ProviderOrigin::Process)
            },
            |request: ProcessProviderRequest<'_>| {
                assert!(!request.emitted_package.bytes().is_empty());
                assert_eq!(request.deadline_ms, 4096);
                assert!(!request.manifest.is_empty());
                Ok(serde_json::json!({
                    "items": request.provider_request.items().iter().map(|item| item.request_index).collect::<Vec<_>>()
                }))
            },
            |manifest: &Hello, result: &serde_json::Value| {
                if result["identity"] == manifest.identity {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            move |result: serde_json::Value| {
                converted.store(result["items"].as_array().is_some(), Ordering::Release);
                let identity = result["identity"]
                    .as_str()
                    .expect("echo fixture returns an identity")
                    .to_owned();
                Ok(ProviderResults::<String, String> {
                    items: vec![ProviderItemResult {
                        request_index: 23,
                        backend: quire_driver::Candidate {
                            identity: identity.clone(),
                        },
                        result: Ok(identity),
                    }],
                })
            },
        );
        let cancel = Cancel::new();
        let fixture = fixture_path();
        assert!(fixture.is_file(), "host echo fixture exists");
        let mut session = provider
            .start(
                &python_path(),
                &[fixture.into_os_string()],
                limits(),
                &cancel,
            )
            .expect("host starts and admits hello");
        assert_eq!(session.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(session.descriptor().id().as_str(), "agent-ix/provider/test");
        assert_eq!(admissions.load(Ordering::Relaxed), 1);

        let mut linked = build_linked_providers(&[]).expect("empty linked registry builds");
        linked
            .register_admitted([(session.descriptor().clone(), session.manifest().clone())])
            .expect("caller-admitted Process descriptor joins same registry");
        assert_eq!(
            linked
                .registry()
                .descriptors()
                .next()
                .expect("one descriptor registered")
                .origin(),
            ProviderOrigin::Process
        );

        // An empty request is intentional while CG IR-629 has no process
        // BackendKind; this adapter test cannot claim routed Process items.
        let package = crate::test_support::emitted_package();
        let manifest = linked.manifest_snapshot();
        let request = ProcessProviderRequest {
            emitted_package: &package,
            deadline_ms: 4096,
            manifest: &manifest,
            provider_request: ProviderRequest::new(&[]),
            qsl_items: &[],
            cancel: &cancel,
        };
        let results = session.invoke(request).expect("host returns typed result");
        assert!(conversion_called.load(Ordering::Acquire));
        assert_eq!(results.items.len(), 1);
        assert_eq!(results.items[0].request_index, 23);
        assert_eq!(results.items[0].backend.identity, "agent-ix/provider/test");
        assert_eq!(
            results.items[0].result.as_deref(),
            Ok("agent-ix/provider/test")
        );
        let finished = session.shutdown().expect("host shuts down cleanly");
        assert_eq!(finished.manifest.identity, "agent-ix/provider/test");
        assert_eq!(finished.stderr, b"opaque provider diagnostic\n");
        assert!(!finished.stderr_truncated);
    }

    /// Trace: FR-005-AC-5, FR-005-AC-12, FR-005-AC-13, FR-005-AC-14, FR-005-AC-15.
    #[test]
    fn fr331_hello_is_admitted_once_and_retained_for_registry_snapshot() {
        use crate::backend_provider::{ManifestAdmissionError, ManifestStage};

        let admissions = Arc::new(AtomicUsize::new(0));
        let admission_count = Arc::clone(&admissions);
        let provider = HostProcessProvider::<
            ManifestStage,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            ManifestAdmissionError,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            move |manifest: &ManifestStage| {
                admission_count.fetch_add(1, Ordering::Relaxed);
                manifest
                    .admit_process(limits().max_frame_bytes)
                    .map(|admitted| (admitted.descriptor().clone(), admitted.manifest().clone()))
            },
            |_: ProcessProviderRequest<'_>| Ok(serde_json::json!({"items": []})),
            |manifest: &ManifestStage, result: &serde_json::Value| {
                if result["identity"] == manifest.manifest[0]["backend"] {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_| Ok(ProviderResults::<serde_json::Value, String>::default()),
        );
        let cancel = Cancel::new();
        let fixture = backend_provider_fixture_path();
        assert!(fixture.is_file(), "FR-331 fixture exists");
        let mut session = provider
            .start(
                &python_path(),
                &[fixture.into_os_string()],
                limits(),
                &cancel,
            )
            .expect("typed FR-331 hello is admitted");
        assert_eq!(admissions.load(Ordering::Relaxed), 1);
        assert_eq!(session.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(
            session.descriptor().id().as_str(),
            "agent-ix/provider/fr331-fixture"
        );

        let mut linked = build_linked_providers(&[]).expect("one registry builds");
        linked
            .register_admitted([(session.descriptor().clone(), session.manifest().clone())])
            .expect("no second admission is performed");
        assert_eq!(
            linked
                .registry()
                .descriptors()
                .next()
                .expect("one process descriptor is registered")
                .origin(),
            ProviderOrigin::Process
        );
        let manifest = linked.manifest_snapshot();
        assert_eq!(manifest.len(), 1);
        let retained_wire = manifest[0]
            .wire
            .as_deref()
            .expect("retained hello descriptor has canonical bytes");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(retained_wire)
                .expect("retained bytes are valid JSON"),
            serde_json::json!({
                "backend": "agent-ix/provider/fr331-fixture",
                "advertises": [{"kind": "operation-contract", "mode": "bounded"}],
                "domains": ["integer"],
                "bounds": {"max_states": 8}
            })
        );

        let package = crate::test_support::emitted_package();
        let request = ProcessProviderRequest {
            emitted_package: &package,
            deadline_ms: 4096,
            manifest: &manifest,
            provider_request: ProviderRequest::new(&[]),
            qsl_items: &[],
            cancel: &cancel,
        };
        session
            .invoke(request)
            .expect("fixture responds to request");
        let finished = session.shutdown().expect("session ends cleanly");
        assert_eq!(finished.manifest.manifest.len(), 1);
        assert_eq!(finished.manifest.manifest[0]["domains"][0], "integer");
        assert_eq!(finished.manifest.manifest[0]["bounds"]["max_states"], 8);
    }

    #[test]
    fn adapter_preserves_result_validation_rejection() {
        use quire_plugin_host::PluginError;

        let provider = HostProcessProvider::<
            Hello,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            String,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            |manifest: &Hello| admit_hello(manifest, ProviderOrigin::Process),
            |_: ProcessProviderRequest<'_>| Ok(serde_json::json!({"items": []})),
            |manifest: &Hello, result: &serde_json::Value| {
                if result["identity"] == manifest.identity {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_: serde_json::Value| Ok(ProviderResults::<serde_json::Value, String>::default()),
        );
        let cancel = Cancel::new();
        let fixture = wrong_result_fixture_path();
        let arguments = [fixture.into_os_string()];
        let mut session = provider
            .start(&python_path(), &arguments, limits(), &cancel)
            .expect("fixture hello is admitted");
        let package = crate::test_support::emitted_package();
        let request = ProcessProviderRequest {
            emitted_package: &package,
            deadline_ms: 4096,
            manifest: &[],
            provider_request: ProviderRequest::new(&[]),
            qsl_items: &[],
            cancel: &cancel,
        };
        let error = session.invoke(request);
        let rejected = match error {
            Err(super::AdapterError::Request(PluginError::ResultValidation(_))) => true,
            Err(super::AdapterError::Request(PluginError::WithDiagnostics { cause, .. })) => {
                matches!(*cause, PluginError::ResultValidation(_))
            }
            _ => false,
        };
        assert!(
            rejected,
            "adapter preserves typed result validation refusal"
        );
    }

    #[test]
    fn caller_admission_rejects_linked_origin_descriptor() {
        use quire_plugin_host::PluginError;

        let provider = HostProcessProvider::<
            Hello,
            serde_json::Value,
            serde_json::Value,
            serde_json::Value,
            String,
            String,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            crate::signals::SignalContext::default(),
            |manifest: &Hello| admit_hello(manifest, ProviderOrigin::Linked),
            |_: ProcessProviderRequest<'_>| Ok(serde_json::json!({})),
            |_: &Hello, _: &serde_json::Value| Ok::<(), String>(()),
            |_: serde_json::Value| Ok(ProviderResults::<serde_json::Value, String>::default()),
        );
        let fixture = fixture_path();
        let error = provider.start(
            &python_path(),
            &[fixture.into_os_string()],
            limits(),
            &Cancel::new(),
        );
        let refused_for_origin = match error {
            Err(super::AdapterError::Start(PluginError::ManifestAdmission(
                super::AdmissionFailure::WrongOrigin,
            ))) => true,
            Err(super::AdapterError::Start(PluginError::WithDiagnostics { cause, .. })) => {
                matches!(
                    *cause,
                    PluginError::ManifestAdmission(super::AdmissionFailure::WrongOrigin)
                )
            }
            _ => false,
        };
        assert!(refused_for_origin, "linked-origin hello is rejected");
    }
}
