// SPDX-License-Identifier: AGPL-3.0-or-later
//! Driver-owned seam for caller-selected process providers.
//!
//! The wire authority is QSpec FR-331
//! (`agent-ix/quire-specification/spec/objects/interfaces/FR-331-backend-provider-envelope.md`)
//! and FR-462 (`spec/functional/tooling/FR-462-speak-the-plugin-wire.md`).
//! This module owns no wire schema; its caller supplies the typed item and
//! result conversion at that authority boundary.

use std::path::Path;
use std::time::Duration;

use qsl_route::{BackendDescriptor, BackendId};
use quire_exact::Cancel;

/// Caller-owned process and framing budgets, independent of the host crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessProviderLimits {
    /// Maximum encoded frame size.
    pub max_frame_bytes: u64,
    /// Maximum encoded bytes across the full session.
    pub max_session_bytes: u64,
    /// Maximum stderr bytes retained for diagnostics.
    pub max_stderr_bytes: u64,
    /// Grace period for cancellation and shutdown.
    pub cancellation_grace: Duration,
    /// Maximum process-tree resident memory, when configured.
    pub max_process_memory_bytes: Option<u64>,
    /// Maximum process-tree CPU time, when configured.
    pub max_process_cpu_time: Option<Duration>,
}

/// One request passed from the driver to its process-provider implementation.
pub struct ProcessProviderRequest<'a, Item> {
    /// Explicit executable path selected by the caller.
    pub executable: &'a Path,
    /// Admitted provider identity selected by negotiation.
    pub backend: &'a BackendId,
    /// The descriptor admitted once from the provider hello.
    pub descriptor: &'a BackendDescriptor,
    /// Exact emitted checked-package bytes.
    pub emitted_package: &'a [u8],
    /// Exactly the request items routed to this provider.
    pub items: &'a [Item],
    /// Caller-supplied process limits.
    pub limits: ProcessProviderLimits,
    /// Caller-owned cancellation handle.
    pub cancel: &'a Cancel,
}

/// Implementation of the driver's process-provider boundary.
///
/// The associated item and result types are intentionally supplied by the
/// caller. The driver does not invent an FR-331 wire representation or depend
/// on `quire-plugin-host`.
pub trait ProcessProvider {
    /// Request-item type selected by the caller's FR-331 contract.
    type Item;
    /// Typed provider result returned to the caller's settlement path.
    type Results;
    /// Typed adapter failure.
    type Error;

    /// Dispatch one provider's routed items through the caller's adapter.
    fn invoke(
        &self,
        request: ProcessProviderRequest<'_, Self::Item>,
    ) -> Result<Self::Results, Self::Error>;
}

/// Invoke the process-provider seam with an already-admitted descriptor.
pub fn invoke_process_provider<P: ProcessProvider>(
    provider: &P,
    request: ProcessProviderRequest<'_, P::Item>,
) -> Result<P::Results, P::Error> {
    provider.invoke(request)
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "failed seam assertions are the test report"
)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin};
    use quire_exact::Cancel;

    use super::{
        invoke_process_provider, ProcessProvider, ProcessProviderLimits, ProcessProviderRequest,
    };

    struct EchoProvider;

    impl ProcessProvider for EchoProvider {
        type Item = u8;
        type Results = Vec<u8>;
        type Error = std::convert::Infallible;

        fn invoke(
            &self,
            request: ProcessProviderRequest<'_, Self::Item>,
        ) -> Result<Self::Results, Self::Error> {
            assert_eq!(request.executable, Path::new("./provider"));
            assert_eq!(request.backend.as_str(), "agent-ix/provider/process");
            assert_eq!(request.descriptor.origin(), ProviderOrigin::Process);
            assert_eq!(request.emitted_package, b"package");
            assert_eq!(request.items, [3, 5]);
            assert_eq!(request.limits.max_frame_bytes, 64);
            assert_eq!(request.cancel.cause(), None);
            Ok(request.items.to_vec())
        }
    }

    /// Trace: FR-006-AC-1, the seam forwards the named executable, admitted
    /// process descriptor, emitted bytes, exactly routed items, limits and cancellation.
    #[test]
    fn invokes_process_provider_with_the_complete_typed_request() {
        let backend = BackendId::from_wire("agent-ix/provider/process").expect("nonempty id");
        let descriptor = BackendDescriptor::new(
            Candidate::new(backend.clone()),
            ProviderOrigin::Process,
            ManifestDigest::from_digest([9; 32]),
            [],
        );
        let items = [3, 5];
        let cancel = Cancel::new();
        let results = invoke_process_provider(
            &EchoProvider,
            ProcessProviderRequest {
                executable: Path::new("./provider"),
                backend: &backend,
                descriptor: &descriptor,
                emitted_package: b"package",
                items: &items,
                limits: ProcessProviderLimits {
                    max_frame_bytes: 64,
                    max_session_bytes: 1024,
                    max_stderr_bytes: 128,
                    cancellation_grace: Duration::from_millis(50),
                    max_process_memory_bytes: None,
                    max_process_cpu_time: None,
                },
                cancel: &cancel,
            },
        )
        .expect("provider succeeds");
        assert_eq!(results, items);
    }
}
