// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generic `quire-plugin-host` adapter for the driver's process-provider seam.

use std::convert::Infallible;
use std::ffi::OsString;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use qsl_route::{BackendDescriptor, ProviderOrigin};
use quire_driver::{
    ProcessProvider, ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession,
    ProviderResults,
};
use quire_exact::Cancel;
use quire_plugin_host::{
    FrameLimits, PluginError, PluginSession, RunLimits, SessionFinished, SessionStart,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

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
    /// Sending a request, validating its result, or process monitoring failed.
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
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Q + 'static,
    Validate: Fn(&M, &R) -> Result<(), V> + 'static,
    Convert: Fn(R) -> ProviderResults<I, E> + 'static,
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
                    max_process_memory_bytes: limits.max_process_memory_bytes,
                    max_process_cpu_time: limits.max_process_cpu_time,
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
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Q,
    Validate: Fn(&M, &R) -> Result<(), V>,
    Convert: Fn(R) -> ProviderResults<I, E>,
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
        let body = (self.encode)(request);
        let result = self
            .host
            .request(&body, cancel, |manifest, result| {
                (self.validate)(manifest, result)
            })
            .map_err(AdapterError::Request)?;
        Ok((self.convert)(result))
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
    use qsl_route::{
        descriptor_from_manifest, BackendDescriptor, Candidate, ManifestDigest, ProviderOrigin,
    };
    use quire_driver::{
        build_linked_providers, ProcessProvider, ProcessProviderLimits, ProcessProviderRequest,
        ProcessProviderSession, ProviderItemResult, ProviderManifest, ProviderRequest,
        ProviderResults,
    };
    use quire_exact::Cancel;
    use serde::Deserialize;
    use sha2::Digest as _;

    use super::HostProcessProvider;

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
        digest: u8,
    ) -> Result<(BackendDescriptor, ProviderManifest), String> {
        let identity =
            BackendId::from_wire(&manifest.identity).map_err(|error| error.to_string())?;
        let digest = ManifestDigest::from_digest([digest; 32]);
        let descriptor = descriptor_from_manifest(
            Candidate::new(identity),
            origin,
            digest,
            manifest
                .advertises
                .iter()
                .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
            manifest
                .domains
                .as_deref()
                .map(|domains| domains.iter().map(String::as_str).collect::<Vec<_>>())
                .as_deref(),
        )
        .map_err(|error| format!("{error:?}"))?;
        let retained = ProviderManifest {
            identity: manifest.identity.clone(),
            manifest_digest: digest,
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
            max_process_memory_bytes: None,
            max_process_cpu_time: None,
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
                admit_hello(manifest, ProviderOrigin::Process, 9)
            },
            |request: ProcessProviderRequest<'_>| {
                assert!(!request.emitted_package.bytes().is_empty());
                assert_eq!(request.deadline_ms, 4096);
                assert!(!request.manifest.is_empty());
                serde_json::json!({
                    "items": request.provider_request.items().iter().map(|item| item.request_index).collect::<Vec<_>>()
                })
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
                ProviderResults::<String, String> {
                    items: vec![ProviderItemResult {
                        request_index: 23,
                        backend: quire_driver::Candidate {
                            identity: identity.clone(),
                        },
                        result: Ok(identity),
                    }],
                }
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
            |_: ProcessProviderRequest<'_>| serde_json::json!({"items": []}),
            |manifest: &ManifestStage, result: &serde_json::Value| {
                if result["identity"] == manifest.manifest[0]["backend"] {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_| ProviderResults::<serde_json::Value, String>::default(),
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
        let retained_digest: [u8; 32] = sha2::Sha256::digest(retained_wire).into();
        assert_eq!(
            retained_digest,
            *session.manifest().manifest_digest.record().as_bytes()
        );
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
            |manifest: &Hello| admit_hello(manifest, ProviderOrigin::Process, 9),
            |_: ProcessProviderRequest<'_>| serde_json::json!({"items": []}),
            |manifest: &Hello, result: &serde_json::Value| {
                if result["identity"] == manifest.identity {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_: serde_json::Value| ProviderResults::<serde_json::Value, String>::default(),
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
            |manifest: &Hello| admit_hello(manifest, ProviderOrigin::Linked, 7),
            |_: ProcessProviderRequest<'_>| serde_json::json!({}),
            |_: &Hello, _: &serde_json::Value| Ok::<(), String>(()),
            |_: serde_json::Value| ProviderResults::<serde_json::Value, String>::default(),
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
