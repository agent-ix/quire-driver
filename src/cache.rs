// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver-owned cache boundary (FR-007).

use qsl_route::{BackendId, ManifestDigest};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

/// A cache-eligible operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheOperation {
    /// A proof operation.
    Prove,
    /// An analysis operation.
    Analyze,
    /// A generation operation.
    Generate,
}

/// Meaning inputs for one cache lookup or insertion.
#[derive(Clone, Debug)]
pub struct CacheRequest<'a> {
    /// Operation whose result is cached.
    operation: CacheOperation,
    /// Checked package content identity.
    package_id: &'a str,
    /// Canonical JSON encoding of the typed operation request.
    request: &'a Value,
    /// Selected provider identity admitted by QSL.
    backend: &'a BackendId,
    /// Provider manifest digest admitted by the same registry descriptor.
    manifest: ManifestDigest,
    /// Target and output-affecting options, required only for generation.
    target: Option<&'a Value>,
}

impl<'a> CacheRequest<'a> {
    /// Build cache inputs from a single admitted registry descriptor.
    pub fn new(
        operation: CacheOperation,
        package_id: &'a str,
        request: &'a Value,
        target: Option<&'a Value>,
        registry: &'a crate::AdmittedRegistry,
        selected_backend: &BackendId,
    ) -> Result<Self, CacheRequestError> {
        if operation == CacheOperation::Generate && target.is_none() {
            return Err(CacheRequestError::MissingTarget);
        }
        if operation != CacheOperation::Generate && target.is_some() {
            return Err(CacheRequestError::UnexpectedTarget);
        }
        let descriptor = registry
            .descriptors()
            .find(|descriptor| descriptor.id() == selected_backend)
            .ok_or(CacheRequestError::BackendNotRegistered)?;
        Ok(Self {
            operation,
            package_id,
            request,
            backend: descriptor.candidate().id(),
            manifest: descriptor.manifest_digest(),
            target,
        })
    }

    /// The requested operation.
    pub const fn operation(&self) -> CacheOperation {
        self.operation
    }

    /// The checked package identity.
    pub fn package_id(&self) -> &str {
        self.package_id
    }

    /// The typed request as canonical JSON data.
    pub fn request(&self) -> &Value {
        self.request
    }

    /// The admitted provider identity.
    pub fn backend(&self) -> &BackendId {
        self.backend
    }

    /// The admitted provider manifest digest.
    pub const fn manifest(&self) -> ManifestDigest {
        self.manifest
    }

    /// Generation target and output-affecting options, if applicable.
    pub fn target(&self) -> Option<&Value> {
        self.target
    }
}

/// A cache request did not match the selected operation's shape.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum CacheRequestError {
    /// Generate requires target and output-affecting options.
    #[error("generate cache requests require target and output options")]
    MissingTarget,
    /// Prove and analyze do not include generation target inputs.
    #[error("non-generate cache requests cannot include target inputs")]
    UnexpectedTarget,
    /// The selected backend is not present in the admitted registry.
    #[error("selected backend is not present in the admitted registry")]
    BackendNotRegistered,
}

/// Whether a computed result may be stored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheEligibility {
    /// The result satisfies QSL ADR-029 CA-5.
    Eligible,
    /// The result is explicitly excluded by QSL ADR-029 CA-5.
    NeverStore(CacheExclusion),
}

/// A QSL CA-5 reason an operation value is excluded from the cache.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheExclusion {
    /// The result is an internal failure.
    InternalFailure,
    /// The caller cancelled the operation.
    Cancelled,
    /// The operation ended at a wall-clock deadline.
    Deadline,
    /// The incomplete result is time-limited.
    TimeLimitedIncomplete,
    /// The result came from a process plugin.
    PluginResult,
    /// The value contains wall-clock time.
    ContainsWallTime,
}

/// A cache lookup result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CacheLookup<T> {
    /// A valid value accepted by the caller's typed reader.
    Hit(T),
    /// No valid typed value is present.
    Miss,
}

/// Result of an attempted insertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheStore {
    /// The value was stored.
    Stored,
    /// QSL policy excluded the value.
    NeverStore,
    /// The value could not fit the configured storage limits.
    NotStored,
}

/// A typed operational failure from an injected cache implementation.
#[derive(Debug, thiserror::Error)]
#[error("cache operation failed: {message}")]
pub struct CacheFault {
    message: String,
}

/// Failure from either the cache boundary or the provider closure.
#[derive(Debug, thiserror::Error)]
pub enum CachedRunError<E> {
    /// Cache lookup or recording failed; it was not treated as a miss.
    #[error(transparent)]
    Cache(#[from] CacheFault),
    /// The selected provider failed.
    #[error("provider operation failed")]
    Provider(E),
}

impl CacheFault {
    /// Create a typed cache failure from an implementation's diagnostic.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Downstream cache implementation supplied to a driver operation.
///
/// Generic methods preserve the caller's result type and ensure that the
/// implementation applies the same deserializer for a cached value as for a
/// fresh provider result. Implementations are injected by downstream callers;
/// the driver package does not select or depend on a concrete cache.
pub trait Cache {
    /// Look up and type-check a result for `request`.
    fn lookup<T: DeserializeOwned>(
        &self,
        request: &CacheRequest<'_>,
    ) -> Result<CacheLookup<T>, CacheFault>;

    /// Record the exact result returned by the provider when eligible.
    fn record<T: Serialize>(
        &self,
        request: &CacheRequest<'_>,
        value: &T,
        eligibility: CacheEligibility,
    ) -> Result<CacheStore, CacheFault>;
}

/// Execute one cache-aware provider call, preserving typed hits, misses and
/// cache errors (FR-007).
pub fn run_cached<C, F, T, E>(
    cache: &C,
    request: &CacheRequest<'_>,
    eligibility: CacheEligibility,
    provider: F,
) -> Result<T, CachedRunError<E>>
where
    C: Cache,
    F: FnOnce() -> Result<T, E>,
    T: DeserializeOwned + Serialize,
{
    match cache.lookup(request)? {
        CacheLookup::Hit(value) => Ok(value),
        CacheLookup::Miss => {
            let value = provider().map_err(CachedRunError::Provider)?;
            let _ = cache.record(request, &value, eligibility)?;
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde_json::Value;

    use super::{
        run_cached, Cache, CacheEligibility, CacheExclusion, CacheFault, CacheLookup,
        CacheOperation, CacheRequest, CacheStore, CachedRunError,
    };

    #[derive(Default)]
    struct MemoryCache {
        value: Option<Value>,
        fail_lookup: bool,
        lookups: Cell<usize>,
        records: Cell<usize>,
        recorded: std::cell::RefCell<Option<Value>>,
        store_outcome: Option<CacheStore>,
        eligibility: Cell<Option<CacheEligibility>>,
    }

    impl Cache for MemoryCache {
        fn lookup<T: serde::de::DeserializeOwned>(
            &self,
            _request: &CacheRequest<'_>,
        ) -> Result<CacheLookup<T>, CacheFault> {
            self.lookups.set(self.lookups.get() + 1);
            if self.fail_lookup {
                return Err(CacheFault::new("injected I/O failure"));
            }
            match &self.value {
                Some(value) => serde_json::from_value(value.clone())
                    .map(CacheLookup::Hit)
                    .or(Ok(CacheLookup::Miss)),
                None => Ok(CacheLookup::Miss),
            }
        }

        fn record<T: serde::Serialize>(
            &self,
            _request: &CacheRequest<'_>,
            value: &T,
            eligibility: CacheEligibility,
        ) -> Result<CacheStore, CacheFault> {
            self.records.set(self.records.get() + 1);
            self.eligibility.set(Some(eligibility));
            let encoded =
                serde_json::to_value(value).map_err(|error| CacheFault::new(error.to_string()))?;
            *self.recorded.borrow_mut() = Some(encoded);
            Ok(self.store_outcome.unwrap_or(CacheStore::Stored))
        }
    }

    fn registry(
        identity: &str,
        digest: [u8; 32],
    ) -> (crate::AdmittedRegistry, qsl_route::BackendId) {
        let backend = qsl_route::BackendId::from_wire(identity).expect("valid id");
        let registry = crate::build_registry(&[crate::ProviderManifest {
            identity: identity.to_owned(),
            manifest_digest: qsl_route::ManifestDigest::from_digest(digest),
            advertises: Vec::new(),
        }])
        .expect("manifest is admitted");
        (registry, backend)
    }

    fn request<'a>(
        registry: &'a crate::AdmittedRegistry,
        backend: &qsl_route::BackendId,
    ) -> CacheRequest<'a> {
        CacheRequest::new(
            CacheOperation::Prove,
            "package",
            &EMPTY_REQUEST,
            None,
            registry,
            backend,
        )
        .expect("request matches operation")
    }

    static EMPTY_REQUEST: std::sync::LazyLock<Value> =
        std::sync::LazyLock::new(|| serde_json::json!({"items": []}));

    /// Trace: FR-007-AC-1 and FR-007-AC-6, the request carries one admitted identity/digest pair.
    #[test]
    fn cache_request_has_the_typed_meaning_and_admission_inputs() {
        let digest = qsl_route::ManifestDigest::from_digest([7; 32]);
        let (registry, backend) = registry("kani", [7; 32]);
        let request = CacheRequest::new(
            CacheOperation::Generate,
            "checked-package",
            &EMPTY_REQUEST,
            Some(&EMPTY_REQUEST),
            &registry,
            &backend,
        )
        .expect("generate request has target inputs");
        assert_eq!(request.operation(), CacheOperation::Generate);
        assert_eq!(request.package_id(), "checked-package");
        assert_eq!(request.backend().as_str(), "kani");
        assert_eq!(request.manifest(), digest);
        assert_eq!(request.target(), Some(&*EMPTY_REQUEST));
    }

    /// Trace: FR-007-AC-6; a registry refusal cannot yield a cache request or lookup.
    #[test]
    fn refused_registry_produces_no_cache_request_or_lookup() {
        let cache = MemoryCache::default();
        let backend = qsl_route::BackendId::from_wire("kani").expect("valid id");
        let registration = crate::build_registry(&[crate::ProviderManifest {
            identity: String::new(),
            manifest_digest: qsl_route::ManifestDigest::from_digest([3; 32]),
            advertises: Vec::new(),
        }]);
        assert!(registration.is_err());
        let empty_registry = crate::build_registry(&[]).expect("empty registry is admitted");
        assert!(matches!(
            CacheRequest::new(
                CacheOperation::Prove,
                "package",
                &EMPTY_REQUEST,
                None,
                &empty_registry,
                &backend,
            ),
            Err(super::CacheRequestError::BackendNotRegistered)
        ));
        assert_eq!(cache.lookups.get(), 0);
    }

    /// Trace: FR-007-AC-2, a valid hit is unchanged and skips provider dispatch.
    #[test]
    fn valid_hit_is_returned_without_calling_the_provider() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache {
            value: Some(Value::String("cached".to_owned())),
            ..MemoryCache::default()
        };
        let called = Cell::new(false);
        let value = run_cached(
            &cache,
            &request(&registry, &backend),
            CacheEligibility::Eligible,
            || {
                called.set(true);
                Ok::<_, ()>("fresh".to_owned())
            },
        )
        .expect("hit succeeds");
        assert_eq!(value, "cached");
        assert!(!called.get());
        assert_eq!(cache.records.get(), 0);
    }

    /// Trace: FR-007-AC-3 and FR-007-AC-5, a miss records and returns the provider value.
    #[test]
    fn miss_calls_provider_once_and_records_its_result() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache::default();
        let calls = Cell::new(0);
        let value = run_cached(
            &cache,
            &request(&registry, &backend),
            CacheEligibility::Eligible,
            || {
                calls.set(calls.get() + 1);
                Ok::<_, ()>("fresh".to_owned())
            },
        )
        .expect("miss succeeds");
        assert_eq!(value, "fresh");
        assert_eq!(calls.get(), 1);
        assert_eq!(cache.records.get(), 1);
        assert_eq!(
            cache.recorded.borrow().as_ref(),
            Some(&Value::String("fresh".to_owned()))
        );
    }

    /// Trace: FR-007-AC-5, NeverStore/NotStored do not replace or fail the provider result.
    #[test]
    fn non_store_outcomes_preserve_the_provider_result() {
        let (registry, backend) = registry("kani", [0; 32]);
        for store_outcome in [CacheStore::NeverStore, CacheStore::NotStored] {
            let cache = MemoryCache {
                store_outcome: Some(store_outcome),
                ..MemoryCache::default()
            };
            let value = run_cached(
                &cache,
                &request(&registry, &backend),
                CacheEligibility::NeverStore(CacheExclusion::PluginResult),
                || Ok::<_, ()>("provider-result".to_owned()),
            )
            .expect("non-store status preserves operation success");
            assert_eq!(value, "provider-result");
            assert_eq!(
                cache.eligibility.get(),
                Some(CacheEligibility::NeverStore(CacheExclusion::PluginResult))
            );
        }
    }

    /// Trace: FR-007-AC-4, cache I/O faults remain typed errors.
    #[test]
    fn cache_fault_is_not_converted_to_a_miss() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache {
            fail_lookup: true,
            ..MemoryCache::default()
        };
        let called = Cell::new(false);
        let result = run_cached(
            &cache,
            &request(&registry, &backend),
            CacheEligibility::Eligible,
            || {
                called.set(true);
                Ok::<_, ()>("fresh".to_owned())
            },
        );
        assert!(matches!(result, Err(CachedRunError::Cache(_))));
        assert!(!called.get());
    }
}
