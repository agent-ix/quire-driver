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

type ProviderWireTypes<M, Q, R, F, A, V> = fn() -> (M, Q, R, F, A, V);
type SessionWireTypes<Q, R, V> = fn() -> (Q, R, V);

fn require_admitted_descriptor<A, V, F>(
    descriptor: Option<BackendDescriptor>,
) -> Result<BackendDescriptor, AdapterError<A, V, F>> {
    descriptor.ok_or(AdapterError::MissingAdmittedDescriptor)
}

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
pub struct HostProcessProvider<M, Q, R, F, A, V, Admit, Encode, Validate, Convert> {
    admit: Arc<Admit>,
    encode: Arc<Encode>,
    validate: Arc<Validate>,
    convert: Arc<Convert>,
    signals: crate::signals::SignalContext,
    _wire_types: PhantomData<ProviderWireTypes<M, Q, R, F, A, V>>,
}

impl<M, Q, R, F, A, V, Admit, Encode, Validate, Convert>
    HostProcessProvider<M, Q, R, F, A, V, Admit, Encode, Validate, Convert>
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

impl<M, Q, R, F, A, V, Admit, Encode, Validate, Convert> ProcessProvider
    for HostProcessProvider<M, Q, R, F, A, V, Admit, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
    Admit: Fn(&M) -> Result<BackendDescriptor, A> + 'static,
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Q + 'static,
    Validate: Fn(&M, &R) -> Result<(), V> + 'static,
    Convert: Fn(R) -> ProviderResults + 'static,
{
    type Session = HostProcessSession<M, Q, R, F, A, V, Encode, Validate, Convert>;
    type Error = AdapterError<A, V, F>;

    fn start(
        &self,
        executable: &Path,
        arguments: &[OsString],
        limits: ProcessProviderLimits,
        cancel: &Cancel,
    ) -> Result<Self::Session, Self::Error> {
        let active = self.signals.activate(cancel.clone());
        let mut admitted_descriptor = None;
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
                let descriptor = (self.admit)(manifest).map_err(AdmissionFailure::Caller)?;
                if descriptor.origin() != ProviderOrigin::Process {
                    return Err(AdmissionFailure::WrongOrigin);
                }
                admitted_descriptor = Some(descriptor);
                Ok(())
            },
        )
        .map_err(AdapterError::Start)?;
        let descriptor = require_admitted_descriptor(admitted_descriptor)?;
        Ok(HostProcessSession {
            host,
            descriptor,
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
pub struct HostProcessSession<M, Q, R, F, A, V, Encode, Validate, Convert> {
    host: PluginSession<M, F, AdmissionFailure<A>>,
    descriptor: BackendDescriptor,
    signals: crate::signals::SignalContext,
    encode: Arc<Encode>,
    validate: Arc<Validate>,
    convert: Arc<Convert>,
    active: crate::signals::ActiveSession,
    _wire_types: PhantomData<SessionWireTypes<Q, R, V>>,
}

impl<M, Q, R, F, A, V, Encode, Validate, Convert> ProcessProviderSession
    for HostProcessSession<M, Q, R, F, A, V, Encode, Validate, Convert>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
    Encode: for<'a> Fn(ProcessProviderRequest<'a>) -> Q,
    Validate: Fn(&M, &R) -> Result<(), V>,
    Convert: Fn(R) -> ProviderResults,
{
    type Error = AdapterError<A, V, F>;
    type Diagnostics = SessionFinished<M>;

    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    fn invoke(
        &mut self,
        request: ProcessProviderRequest<'_>,
    ) -> Result<ProviderResults, Self::Error> {
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

    use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin};
    use quire_driver::{
        build_linked_providers, ProcessProvider, ProcessProviderLimits, ProcessProviderRequest,
        ProcessProviderSession, ProviderRequest, ProviderResults,
    };
    use quire_exact::Cancel;
    use serde::Deserialize;

    use super::HostProcessProvider;

    #[derive(Debug, Deserialize)]
    struct Hello {
        identity: String,
        advertises: Vec<(Option<String>, Option<String>)>,
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
        >::new(
            crate::signals::SignalContext::default(),
            move |manifest: &Hello| {
                admission_count.fetch_add(1, Ordering::Relaxed);
                let identity =
                    BackendId::from_wire(&manifest.identity).map_err(|error| error.to_string())?;
                BackendDescriptor::admit(
                    Candidate::new(identity),
                    ProviderOrigin::Process,
                    ManifestDigest::from_digest([9; 32]),
                    manifest
                        .advertises
                        .iter()
                        .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
                )
                .map_err(|error| format!("{error:?}"))
            },
            |request: ProcessProviderRequest<'_>| {
                assert_eq!(request.emitted_package, b"checked package bytes");
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
                ProviderResults::default()
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
            .register_admitted([session.descriptor().clone()])
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
        let request = ProcessProviderRequest {
            emitted_package: b"checked package bytes",
            provider_request: ProviderRequest::new(&[]),
            cancel: &cancel,
        };
        session.invoke(request).expect("host returns typed result");
        assert!(conversion_called.load(Ordering::Acquire));
        let finished = session.shutdown().expect("host shuts down cleanly");
        assert_eq!(finished.manifest.identity, "agent-ix/provider/test");
        assert_eq!(finished.stderr, b"opaque provider diagnostic\n");
        assert!(!finished.stderr_truncated);
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
        >::new(
            crate::signals::SignalContext::default(),
            |manifest: &Hello| {
                let identity =
                    BackendId::from_wire(&manifest.identity).map_err(|error| error.to_string())?;
                BackendDescriptor::admit(
                    Candidate::new(identity),
                    ProviderOrigin::Process,
                    ManifestDigest::from_digest([9; 32]),
                    manifest
                        .advertises
                        .iter()
                        .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
                )
                .map_err(|error| format!("{error:?}"))
            },
            |_: ProcessProviderRequest<'_>| serde_json::json!({"items": []}),
            |manifest: &Hello, result: &serde_json::Value| {
                if result["identity"] == manifest.identity {
                    Ok(())
                } else {
                    Err("result identity differs from admitted hello".to_owned())
                }
            },
            |_: serde_json::Value| ProviderResults::default(),
        );
        let cancel = Cancel::new();
        let fixture = wrong_result_fixture_path();
        let arguments = [fixture.into_os_string()];
        let mut session = provider
            .start(&python_path(), &arguments, limits(), &cancel)
            .expect("fixture hello is admitted");
        let request = ProcessProviderRequest {
            emitted_package: b"checked package bytes",
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
        >::new(
            crate::signals::SignalContext::default(),
            |manifest: &Hello| {
                BackendDescriptor::admit(
                    Candidate::new(
                        BackendId::from_wire(&manifest.identity).map_err(|e| e.to_string())?,
                    ),
                    ProviderOrigin::Linked,
                    ManifestDigest::from_digest([7; 32]),
                    manifest
                        .advertises
                        .iter()
                        .map(|(kind, mode)| (kind.as_deref(), mode.as_deref())),
                )
                .map_err(|error| format!("{error:?}"))
            },
            |_: ProcessProviderRequest<'_>| serde_json::json!({}),
            |_: &Hello, _: &serde_json::Value| Ok::<(), String>(()),
            |_: serde_json::Value| ProviderResults::default(),
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

    #[test]
    fn missing_caller_admitted_descriptor_is_a_typed_adapter_error() {
        let result: Result<BackendDescriptor, super::AdapterError<String, String, String>> =
            super::require_admitted_descriptor(None);
        assert!(matches!(
            result,
            Err(super::AdapterError::MissingAdmittedDescriptor)
        ));
    }
}
