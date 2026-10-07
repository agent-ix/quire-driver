// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver-owned cache boundary (FR-007).

use qsl_route::{BackendId, ManifestDigest};

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
    request: &'a [u8],
    /// Selected provider identity admitted by QSL.
    backend: &'a BackendId,
    /// Provider manifest digest admitted by the same registry descriptor.
    manifest: ManifestDigest,
    /// Target and output-affecting options, required only for generation.
    target: Option<&'a [u8]>,
}

impl<'a> CacheRequest<'a> {
    /// Build cache inputs from a single admitted registry descriptor.
    pub fn new(
        operation: CacheOperation,
        package_id: &'a str,
        request: &'a [u8],
        target: Option<&'a [u8]>,
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
    pub fn request(&self) -> &[u8] {
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
    pub fn target(&self) -> Option<&[u8]> {
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
pub enum CacheWriteOutcome {
    /// The value was stored.
    Stored,
    /// QSL policy excluded the value.
    NeverStore,
    /// The value could not fit the configured storage limits.
    NotStored,
}

/// A typed operational failure from an injected cache implementation.
#[derive(Debug, thiserror::Error)]
pub enum CacheFault {
    /// A typed operating-system I/O failure.
    #[error("cache I/O failed: {0}")]
    Io(#[from] std::io::Error),
    /// A non-I/O operational failure from the cache implementation.
    #[error("cache operation failed: {0}")]
    Other(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// An error preserved by a cache value codec or implementation.
pub type CacheCodecError = Box<dyn std::error::Error + Send + Sync>;

/// A cache value codec supplied by the downstream cache package.
pub trait CacheValueCodec<T> {
    /// Encode a provider result, bounded by the cache's per-entry limit.
    fn encode(&self, value: &T, max_bytes: u64) -> Result<Vec<u8>, CacheCodecError>;
    /// Decode and validate a stored value using the fresh-result reader.
    fn decode(&self, bytes: &[u8]) -> Result<T, CacheCodecError>;
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
    /// Create a non-I/O cache failure from an implementation's diagnostic.
    pub fn new(message: impl Into<String>) -> Self {
        Self::Other(Box::new(CacheMessage(message.into())))
    }

    /// Preserve a typed non-I/O source error from a cache implementation.
    pub fn from_error(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Other(Box::new(error))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct CacheMessage(String);

/// Downstream cache implementation supplied to a driver operation.
///
/// Generic methods preserve the caller's result type and ensure that the
/// implementation applies the same deserializer for a cached value as for a
/// fresh provider result. Implementations are injected by downstream callers;
/// the driver package does not select or depend on a concrete cache.
pub trait Cache {
    /// Look up and type-check a result for `request`.
    fn lookup<T>(
        &self,
        request: &CacheRequest<'_>,
        codec: &dyn CacheValueCodec<T>,
    ) -> Result<CacheLookup<T>, CacheFault>;

    /// Record the exact result returned by the provider when eligible.
    fn record<T>(
        &self,
        request: &CacheRequest<'_>,
        value: &T,
        codec: &dyn CacheValueCodec<T>,
        eligibility: CacheEligibility,
    ) -> Result<CacheWriteOutcome, CacheFault>;
}

/// Execute one cache-aware provider call, preserving typed hits, misses and
/// cache errors (FR-007).
pub fn run_cached<C, F, T, E>(
    cache: &C,
    request: &CacheRequest<'_>,
    codec: &dyn CacheValueCodec<T>,
    eligibility: CacheEligibility,
    provider: F,
) -> Result<T, CachedRunError<E>>
where
    C: Cache,
    F: FnOnce() -> Result<T, E>,
{
    match cache.lookup(request, codec)? {
        CacheLookup::Hit(value) => Ok(value),
        CacheLookup::Miss => {
            let value = provider().map_err(CachedRunError::Provider)?;
            let _ = cache.record(request, &value, codec, eligibility)?;
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde_json::Value;

    use super::{
        run_cached, Cache, CacheCodecError, CacheEligibility, CacheExclusion, CacheFault,
        CacheLookup, CacheOperation, CacheRequest, CacheValueCodec, CacheWriteOutcome,
        CachedRunError,
    };

    #[derive(Default)]
    struct MemoryCache {
        value: std::cell::RefCell<Option<Vec<u8>>>,
        fail_lookup: bool,
        fail_record: bool,
        lookups: Cell<usize>,
        records: Cell<usize>,
        recorded: std::cell::RefCell<Option<Value>>,
        store_outcome: Option<CacheWriteOutcome>,
        eligibility: Cell<Option<CacheEligibility>>,
    }

    impl Cache for MemoryCache {
        fn lookup<T>(
            &self,
            _request: &CacheRequest<'_>,
            codec: &dyn CacheValueCodec<T>,
        ) -> Result<CacheLookup<T>, CacheFault> {
            self.lookups.set(self.lookups.get() + 1);
            if self.fail_lookup {
                return Err(CacheFault::Io(std::io::Error::other(
                    "injected I/O failure",
                )));
            }
            match &*self.value.borrow() {
                Some(value) => Ok(codec
                    .decode(value)
                    .map(CacheLookup::Hit)
                    .unwrap_or(CacheLookup::Miss)),
                None => Ok(CacheLookup::Miss),
            }
        }

        fn record<T>(
            &self,
            _request: &CacheRequest<'_>,
            value: &T,
            codec: &dyn CacheValueCodec<T>,
            eligibility: CacheEligibility,
        ) -> Result<CacheWriteOutcome, CacheFault> {
            self.records.set(self.records.get() + 1);
            self.eligibility.set(Some(eligibility));
            if self.fail_record {
                return Err(CacheFault::Io(std::io::Error::other("record failed")));
            }
            let encoded = match codec.encode(value, u64::MAX) {
                Ok(encoded) => encoded,
                Err(_) => return Ok(CacheWriteOutcome::NotStored),
            };
            *self.recorded.borrow_mut() = serde_json::from_slice(&encoded).ok();
            self.value.borrow_mut().replace(encoded);
            Ok(self.store_outcome.unwrap_or(CacheWriteOutcome::Stored))
        }
    }

    struct JsonCodec;

    impl<T> CacheValueCodec<T> for JsonCodec
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        fn encode(&self, value: &T, max_bytes: u64) -> Result<Vec<u8>, CacheCodecError> {
            let bytes =
                serde_json::to_vec(value).map_err(|error| Box::new(error) as CacheCodecError)?;
            if bytes.len() as u64 > max_bytes {
                return Err(Box::new(std::io::Error::other(
                    "encoded value exceeds limit",
                )));
            }
            Ok(bytes)
        }

        fn decode(&self, bytes: &[u8]) -> Result<T, CacheCodecError> {
            serde_json::from_slice(bytes).map_err(|error| Box::new(error) as CacheCodecError)
        }
    }

    struct RefusingCodec;

    impl CacheValueCodec<String> for RefusingCodec {
        fn encode(&self, _value: &String, _max_bytes: u64) -> Result<Vec<u8>, CacheCodecError> {
            Err(Box::new(std::io::Error::other(
                "canonical encoder refused the result",
            )))
        }

        fn decode(&self, _bytes: &[u8]) -> Result<String, CacheCodecError> {
            Err(Box::new(std::io::Error::other("no value accepted")))
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
            wire: None,
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
            EMPTY_REQUEST,
            None,
            registry,
            backend,
        )
        .expect("request matches operation")
    }

    const EMPTY_REQUEST: &[u8] = b"{\"items\":[]}";

    /// Unit check of request shape; provider-dispatch integration remains planned (FR-007-AC-1/6).
    #[test]
    fn cache_request_has_the_typed_meaning_and_admission_inputs() {
        let digest = qsl_route::ManifestDigest::from_digest([7; 32]);
        let (registry, backend) = registry("kani", [7; 32]);
        let request = CacheRequest::new(
            CacheOperation::Generate,
            "checked-package",
            EMPTY_REQUEST,
            Some(EMPTY_REQUEST),
            &registry,
            &backend,
        )
        .expect("generate request has target inputs");
        assert_eq!(request.operation(), CacheOperation::Generate);
        assert_eq!(request.package_id(), "checked-package");
        assert_eq!(request.backend().as_str(), "kani");
        assert_eq!(request.manifest(), digest);
        assert_eq!(request.target(), Some(EMPTY_REQUEST));
    }

    /// Unit check of operation target shape; provider-dispatch integration remains planned (FR-007-AC-1).
    #[test]
    fn cache_request_rejects_missing_and_unexpected_generation_target() {
        let (registry, backend) = registry("kani", [1; 32]);
        assert_eq!(
            CacheRequest::new(
                CacheOperation::Generate,
                "package",
                EMPTY_REQUEST,
                None,
                &registry,
                &backend
            )
            .unwrap_err(),
            super::CacheRequestError::MissingTarget,
        );
        assert_eq!(
            CacheRequest::new(
                CacheOperation::Prove,
                "package",
                EMPTY_REQUEST,
                Some(EMPTY_REQUEST),
                &registry,
                &backend
            )
            .unwrap_err(),
            super::CacheRequestError::UnexpectedTarget,
        );
    }

    /// Unit check that the request builder selects the matching admitted descriptor (FR-007-AC-1).
    #[test]
    fn selecting_each_admitted_backend_uses_its_matching_descriptor() {
        let first = qsl_route::BackendId::from_wire("first").expect("valid id");
        let second = qsl_route::BackendId::from_wire("second").expect("valid id");
        let registry = crate::build_registry(&[
            crate::ProviderManifest {
                identity: first.as_str().to_owned(),
                manifest_digest: qsl_route::ManifestDigest::from_digest([1; 32]),
                advertises: Vec::new(),
                wire: None,
            },
            crate::ProviderManifest {
                identity: second.as_str().to_owned(),
                manifest_digest: qsl_route::ManifestDigest::from_digest([2; 32]),
                advertises: Vec::new(),
                wire: None,
            },
        ])
        .expect("both descriptors are admitted");
        let first_request = CacheRequest::new(
            CacheOperation::Prove,
            "package",
            EMPTY_REQUEST,
            None,
            &registry,
            &first,
        )
        .expect("first backend is registered");
        let second_request = CacheRequest::new(
            CacheOperation::Prove,
            "package",
            EMPTY_REQUEST,
            None,
            &registry,
            &second,
        )
        .expect("second backend is registered");
        assert_eq!(first_request.backend(), &first);
        assert_eq!(
            first_request.manifest(),
            qsl_route::ManifestDigest::from_digest([1; 32])
        );
        assert_eq!(second_request.backend(), &second);
        assert_eq!(
            second_request.manifest(),
            qsl_route::ManifestDigest::from_digest([2; 32])
        );
    }

    /// Builder check only; provider-dispatch refusal-to-cache integration remains planned (FR-007-AC-6).
    #[test]
    fn cache_request_rejects_backend_outside_admitted_registry() {
        let backend = qsl_route::BackendId::from_wire("kani").expect("valid id");
        let registration = crate::build_registry(&[crate::ProviderManifest {
            identity: String::new(),
            manifest_digest: qsl_route::ManifestDigest::from_digest([3; 32]),
            advertises: Vec::new(),
            wire: None,
        }]);
        assert!(registration.is_err());
        let empty_registry = crate::build_registry(&[]).expect("empty registry is admitted");
        assert!(matches!(
            CacheRequest::new(
                CacheOperation::Prove,
                "package",
                EMPTY_REQUEST,
                None,
                &empty_registry,
                &backend,
            ),
            Err(super::CacheRequestError::BackendNotRegistered)
        ));
    }

    /// Helper contract test; `drive()` cache-hit dispatch remains planned (FR-007-AC-2).
    #[test]
    fn valid_hit_is_returned_without_calling_the_provider() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache {
            value: std::cell::RefCell::new(Some(br#""cached""#.to_vec())),
            ..MemoryCache::default()
        };
        let called = Cell::new(false);
        let codec = JsonCodec;
        let value = run_cached(
            &cache,
            &request(&registry, &backend),
            &codec,
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

    /// Trace: FR-007-AC-3, FR-007-AC-5; a miss records and returns the provider value.
    #[test]
    fn miss_calls_provider_once_and_records_its_result() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache::default();
        let calls = Cell::new(0);
        let codec = JsonCodec;
        let value = run_cached(
            &cache,
            &request(&registry, &backend),
            &codec,
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
        let codec = JsonCodec;
        for store_outcome in [CacheWriteOutcome::NeverStore, CacheWriteOutcome::NotStored] {
            let cache = MemoryCache {
                store_outcome: Some(store_outcome),
                ..MemoryCache::default()
            };
            let value = run_cached(
                &cache,
                &request(&registry, &backend),
                &codec,
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

    /// Trace: FR-007-AC-5; a result encoding refusal is NotStored, not an operation failure.
    #[test]
    fn result_encoding_refusal_preserves_provider_result() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache::default();
        let value = run_cached(
            &cache,
            &request(&registry, &backend),
            &RefusingCodec,
            CacheEligibility::Eligible,
            || Ok::<_, ()>("provider-result".to_owned()),
        )
        .expect("result is returned when serialization declines storage");
        assert_eq!(value, "provider-result");
        assert_eq!(cache.records.get(), 1);
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
        let codec = JsonCodec;
        let result = run_cached(
            &cache,
            &request(&registry, &backend),
            &codec,
            CacheEligibility::Eligible,
            || {
                called.set(true);
                Ok::<_, ()>("fresh".to_owned())
            },
        );
        assert!(matches!(result, Err(CachedRunError::Cache(_))));
        assert!(!called.get());
    }

    /// Trace: FR-007-AC-4; typed I/O sources survive cache error propagation.
    #[test]
    fn recording_io_fault_remains_typed_after_provider_success() {
        let (registry, backend) = registry("kani", [0; 32]);
        let cache = MemoryCache {
            fail_record: true,
            ..MemoryCache::default()
        };
        let codec = JsonCodec;
        let result = run_cached(
            &cache,
            &request(&registry, &backend),
            &codec,
            CacheEligibility::Eligible,
            || Ok::<_, ()>("fresh".to_owned()),
        );
        let Err(CachedRunError::Cache(CacheFault::Io(error))) = result else {
            panic!("record I/O source remains a typed cache fault");
        };
        assert_eq!(error.to_string(), "record failed");
    }
}
