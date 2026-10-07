// SPDX-License-Identifier: AGPL-3.0-or-later
//! Driver-owned seam for caller-selected process providers.
//!
//! The wire authority is QSpec FR-331
//! (`agent-ix/quire-specification/spec/objects/interfaces/FR-331-backend-provider-envelope.md`)
//! and FR-462 (`quire-specification/spec/functional/tooling/FR-462-speak-the-plugin-wire.md`).
//! The seam uses the driver's [`ProviderRequest`] and [`ProviderResults`]; it
//! does not define a local FR-331 wire schema or depend on the host crate.

use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use qsl_route::BackendDescriptor;
use quire_exact::Cancel;

use crate::{ProviderRequest, ProviderResults};

/// Caller-owned process and framing budgets, independent of the host crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessProviderLimits {
    /// Maximum encoded frame size.
    pub max_frame_bytes: u64,
    /// Maximum encoded bytes across the full plugin session; maps to the
    /// host's shared aggregate byte budget.
    pub max_total_bytes: u64,
    /// Maximum stderr bytes retained for diagnostics.
    pub max_stderr_bytes: u64,
    /// Grace period for cancellation and shutdown.
    pub cancellation_grace: Duration,
    /// Maximum process-tree resident memory, when configured.
    pub max_process_memory_bytes: Option<u64>,
    /// Maximum process-tree CPU time, when configured.
    pub max_process_cpu_time: Option<Duration>,
}

/// One routed request sent over an already-started process session.
pub struct ProcessProviderRequest<'a> {
    /// Exact emitted checked-package bytes, intentionally forwarded under
    /// FR-006 without decoding or rewriting them in the driver seam.
    pub emitted_package: &'a [u8],
    /// Driver-owned typed request for this provider's negotiated items.
    pub provider_request: ProviderRequest<'a>,
    /// Caller-owned cancellation handle for this request.
    pub cancel: &'a Cancel,
}

/// A process provider whose hello has already been admitted by QSL.
///
/// `invoke` sends each request over this same session; implementations must
/// not restart the process, send another hello, or admit its descriptor again.
pub trait ProcessProviderSession {
    /// Typed adapter failure.
    type Error;

    /// Host diagnostics retained when graceful shutdown completes, including
    /// captured stderr and the process exit status when available.
    type Diagnostics;

    /// The descriptor returned by the one QSL admission of the hello manifest.
    fn descriptor(&self) -> &BackendDescriptor;

    /// Send one routed request and return driver-owned typed provider results.
    /// Current results carry Kani-specific evidence and failures; generalizing
    /// them for other provider kinds is tracked by IR-677.
    fn invoke(
        &mut self,
        request: ProcessProviderRequest<'_>,
    ) -> Result<ProviderResults, Self::Error>;

    /// Request graceful shutdown and return the host's captured diagnostics.
    /// Callers should preserve or report these diagnostics. Dropping a session
    /// without calling `shutdown` relies on the host's `Drop` cleanup path.
    fn shutdown(self) -> Result<Self::Diagnostics, Self::Error>;
}

/// Start a named process provider and admit its hello before backend routing.
pub trait ProcessProvider {
    /// Started, hello-admitted session type.
    type Session: ProcessProviderSession<Error = Self::Error>;
    /// Typed adapter failure.
    type Error;

    /// Start only the caller-named executable, read one hello, and admit its
    /// manifest exactly once with caller-supplied `Process` origin.
    fn start(
        &self,
        executable: &Path,
        arguments: &[OsString],
        limits: ProcessProviderLimits,
        cancel: &Cancel,
    ) -> Result<Self::Session, Self::Error>;
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;
    use std::ffi::OsString;
    use std::path::Path;
    use std::time::Duration;

    use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin};
    use quire_exact::Cancel;

    use crate::{ProviderRequest, ProviderResults};

    use super::{
        ProcessProvider, ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession,
    };

    struct FakeProvider;

    struct FakeSession {
        descriptor: BackendDescriptor,
    }

    impl ProcessProvider for FakeProvider {
        type Session = FakeSession;
        type Error = Infallible;

        fn start(
            &self,
            executable: &Path,
            arguments: &[OsString],
            limits: ProcessProviderLimits,
            cancel: &Cancel,
        ) -> Result<Self::Session, Self::Error> {
            assert_eq!(executable, Path::new("./provider"));
            assert_eq!(arguments, [OsString::from("--fixture")]);
            assert_eq!(limits.max_total_bytes, 1024);
            assert_eq!(cancel.cause(), None);
            Ok(FakeSession {
                descriptor: BackendDescriptor::admit(
                    Candidate::new(
                        BackendId::from_wire("agent-ix/provider/process").expect("nonempty id"),
                    ),
                    ProviderOrigin::Process,
                    ManifestDigest::from_digest([9; 32]),
                    [(Some("operation-contract"), Some("bounded"))],
                )
                .expect("the hello admitter accepts the fixture manifest"),
            })
        }
    }

    impl ProcessProviderSession for FakeSession {
        type Error = Infallible;
        type Diagnostics = ();

        fn descriptor(&self) -> &BackendDescriptor {
            &self.descriptor
        }

        fn invoke(
            &mut self,
            request: ProcessProviderRequest<'_>,
        ) -> Result<ProviderResults, Self::Error> {
            assert_eq!(request.emitted_package, b"package");
            assert!(request.provider_request.items().is_empty());
            assert_eq!(request.cancel.cause(), None);
            Ok(ProviderResults::default())
        }

        fn shutdown(self) -> Result<Self::Diagnostics, Self::Error> {
            Ok(())
        }
    }

    /// The seam starts once, exposes its admitted Process descriptor for the
    /// shared registry, and sends typed driver requests over that session.
    /// PR2's routing test must send real routed items through this seam; this
    /// test uses an empty request to isolate the session contract.
    #[test]
    fn starts_once_and_invokes_with_driver_owned_provider_types() {
        let cancel = Cancel::new();
        let limits = ProcessProviderLimits {
            max_frame_bytes: 64,
            max_total_bytes: 1024,
            max_stderr_bytes: 128,
            cancellation_grace: Duration::from_millis(50),
            max_process_memory_bytes: None,
            max_process_cpu_time: None,
        };
        let mut session = FakeProvider
            .start(
                Path::new("./provider"),
                &[OsString::from("--fixture")],
                limits,
                &cancel,
            )
            .expect("provider starts");
        assert_eq!(session.descriptor().origin(), ProviderOrigin::Process);
        assert_eq!(
            session.descriptor().id().as_str(),
            "agent-ix/provider/process"
        );

        let package = b"package";
        let provider_request = ProviderRequest::new(&[]);
        let request = ProcessProviderRequest {
            emitted_package: package,
            provider_request,
            cancel: &cancel,
        };
        let _ = session.invoke(request).expect("provider request succeeds");
        session.shutdown().expect("provider shuts down");
    }
}
