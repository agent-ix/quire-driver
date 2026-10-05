// SPDX-License-Identifier: AGPL-3.0-or-later
//! Downstream implementation of the driver's content-addressed cache boundary.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use qsl_route::{BackendId, ManifestDigest};
use quire_canonical::{sha256_with_domain, to_vec, Limits, Sha256Digest};
use quire_driver::{
    Cache, CacheEligibility, CacheExclusion, CacheFault, CacheLookup, CacheOperation, CacheRequest,
    CacheStore as DriverCacheStore,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// QSL ADR-029 CA-2 / QSpec FR-201 cache-key identity domain.
const CACHE_KEY_DOMAIN: &[u8] = b"quire.cache-key/v1";

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A cacheable operation named by ADR-029 CA-1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    /// A terminal record produced by proof.
    Prove,
    /// A terminal record produced by analysis.
    Analyze,
    /// A generated artifact.
    Generate,
}

impl Operation {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prove => "prove",
            Self::Analyze => "analyze",
            Self::Generate => "generate",
        }
    }
}

/// Inputs used to derive one cache key.
///
/// `request` is the typed operation request serialized as canonical JSON and
/// includes each meaning input and deterministic budget for the operation.
#[derive(Clone, Copy, Debug)]
pub struct KeyRequest<'a> {
    /// The operation whose value may be reused.
    pub operation: Operation,
    /// The checked package's content identity.
    pub package_id: &'a str,
    /// The typed operation request, serialized as JSON.
    pub request: &'a Value,
    /// The selected provider's backend identity.
    pub backend: &'a BackendId,
    /// Manifest digest admitted with this backend's registry descriptor.
    pub manifest: ManifestDigest,
    /// Generate target and output-affecting options; required only for `Generate`.
    pub target: Option<&'a Value>,
    /// Maximum canonical bytes for the key and manifest preimages.
    pub max_preimage_bytes: u64,
}

/// A content key and the canonical preimage it binds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheKey {
    digest: Sha256Digest,
    preimage: Vec<u8>,
}

impl CacheKey {
    /// Derive a key from the operation, package, request, provider identity,
    /// and the registry-admitted manifest digest.
    ///
    /// The manifest digest is supplied by the registry descriptor and is not
    /// recomputed here. No recorded version, git revision or caller-supplied
    /// spelling participates in this key.
    pub fn derive(request: KeyRequest<'_>) -> Result<Self, CacheError> {
        if request.operation == Operation::Generate && request.target.is_none() {
            return Err(CacheError::MissingGenerationTarget);
        }
        if request.operation != Operation::Generate && request.target.is_some() {
            return Err(CacheError::UnexpectedGenerationTarget);
        }
        let limits = Limits::new(request.max_preimage_bytes);
        let mut preimage_value = serde_json::json!({
            "operation": request.operation.as_str(),
            "package_id": request.package_id,
            "request": request.request,
            "backend": request.backend.as_str(),
            "manifest": {
                "domain": request.manifest.record().domain().as_str(),
                "digest": request.manifest.hex(),
            },
        });
        if let Some(target) = request.target {
            preimage_value["target"] = target.clone();
        }
        let preimage = to_vec(&preimage_value, limits)?;
        let digest = sha256_with_domain(CACHE_KEY_DOMAIN, &preimage_value, limits)?;
        Ok(Self { digest, preimage })
    }

    /// The lowercase hexadecimal cache key used as the entry name.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.digest.to_string()
    }

    /// The canonical JSON bytes whose identity this key binds.
    #[must_use]
    pub fn preimage(&self) -> &[u8] {
        &self.preimage
    }
}

/// Whether a typed result may be inserted into the cache.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorePolicy {
    /// A completed first-party result eligible under CA-1 and CA-5.
    Cacheable,
    /// A result that CA-5 forbids storing.
    NeverStore(CacheExclusion),
}

/// The outcome of a cache insertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreResult {
    /// The encoded entry is in the store.
    Stored,
    /// The result was excluded by `StorePolicy::NeverStore`.
    NeverStored,
    /// The entry exceeds the caller's total store-size limit.
    OverCapacity,
}

/// The caller-named on-disk cache directory and its maximum size.
pub struct CacheStore {
    directory: PathBuf,
    max_entry_bytes: u64,
    max_bytes: u64,
    operation: Mutex<()>,
}

impl CacheStore {
    /// Open a cache rooted at `directory`, with per-entry and total byte
    /// ceilings. The directory is created on the first successful insertion.
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>, max_entry_bytes: u64, max_bytes: u64) -> Self {
        Self {
            directory: directory.into(),
            max_entry_bytes,
            max_bytes,
            operation: Mutex::new(()),
        }
    }

    /// Read an entry through the caller's typed result reader.
    ///
    /// A missing, noncanonical, mismatched or unreadable typed value is a
    /// cache miss. Files are accepted only when their canonical bytes, stored
    /// preimage, digest and requested key agree.
    pub fn get<T: DeserializeOwned>(
        &self,
        key: &CacheKey,
        max_entry_bytes: u64,
        max_preimage_bytes: u64,
    ) -> Result<Option<T>, CacheError> {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| CacheError::LockPoisoned)?;
        let path = self.entry_path(key);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(CacheError::Io(error)),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Ok(None);
        }
        if metadata.len() > max_entry_bytes {
            return Ok(None);
        }
        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(CacheError::Io(error)),
        };
        let mut bytes = Vec::new();
        file.take(max_entry_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > max_entry_bytes {
            return Ok(None);
        }
        let entry: Value = match serde_json::from_slice(&bytes) {
            Ok(entry) => entry,
            Err(_) => return Ok(None),
        };
        let canonical = match to_vec(&entry, Limits::new(max_entry_bytes)) {
            Ok(canonical) => canonical,
            Err(_) => return Ok(None),
        };
        if canonical != bytes {
            return Ok(None);
        }
        let Some(object) = entry.as_object() else {
            return Ok(None);
        };
        if object.len() != 2 {
            return Ok(None);
        }
        let Some(stored_preimage) = object.get("preimage").and_then(Value::as_str) else {
            return Ok(None);
        };
        if stored_preimage.as_bytes() != key.preimage {
            return Ok(None);
        }
        let stored_value = match object.get("value") {
            Some(value) => value.clone(),
            None => return Ok(None),
        };
        let stored_preimage_value: Value = match serde_json::from_slice(stored_preimage.as_bytes())
        {
            Ok(value) => value,
            Err(_) => return Ok(None),
        };
        let canonical_preimage =
            match to_vec(&stored_preimage_value, Limits::new(max_preimage_bytes)) {
                Ok(canonical) => canonical,
                Err(_) => return Ok(None),
            };
        if canonical_preimage.as_slice() != stored_preimage.as_bytes() {
            return Ok(None);
        }
        let stored_digest = match sha256_with_domain(
            CACHE_KEY_DOMAIN,
            &stored_preimage_value,
            Limits::new(max_preimage_bytes),
        ) {
            Ok(digest) => digest,
            Err(_) => return Ok(None),
        };
        if stored_digest != key.digest {
            return Ok(None);
        }
        match serde_json::from_value(stored_value) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    /// Store a typed value if its policy permits caching.
    ///
    /// The entry is written to a temporary file and atomically renamed into
    /// place. Eviction removes lexicographically earliest cache keys until the
    /// new entry fits within the caller's total size limit.
    pub fn insert<T: Serialize>(
        &self,
        key: &CacheKey,
        value: &T,
        policy: StorePolicy,
        max_entry_bytes: u64,
    ) -> Result<StoreResult, CacheError> {
        if matches!(policy, StorePolicy::NeverStore(_)) {
            return Ok(StoreResult::NeverStored);
        }
        let _guard = self
            .operation
            .lock()
            .map_err(|_| CacheError::LockPoisoned)?;
        let value = serde_json::to_value(value)?;
        let preimage =
            std::str::from_utf8(&key.preimage).map_err(|_| CacheError::InvalidStoredPreimage)?;
        let entry = serde_json::json!({
            "preimage": preimage,
            "value": value,
        });
        let bytes = to_vec(&entry, Limits::new(u64::MAX))?;
        if bytes.len() as u64 > max_entry_bytes || bytes.len() as u64 > self.max_bytes {
            return Ok(StoreResult::OverCapacity);
        }
        fs::create_dir_all(&self.directory)?;
        let path = self.entry_path(key);
        let (temporary, mut file) = self.create_temporary(&path)?;
        let result = (|| -> Result<bool, CacheError> {
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            if !self.evict_for(&path, &temporary, bytes.len() as u64)? {
                return Ok(false);
            }
            fs::rename(&temporary, &path)?;
            Ok(true)
        })();
        if !matches!(&result, Ok(true)) {
            let _ = fs::remove_file(&temporary);
        }
        match result? {
            true => Ok(StoreResult::Stored),
            false => Ok(StoreResult::OverCapacity),
        }
    }

    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.directory.join(format!("{}.json", key.as_str()))
    }

    fn create_temporary(&self, entry: &Path) -> Result<(PathBuf, File), CacheError> {
        for _ in 0..32 {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let name = entry
                .file_name()
                .ok_or(CacheError::InvalidEntryPath)?
                .to_string_lossy();
            let temporary =
                entry.with_file_name(format!(".{name}.tmp-{}-{sequence}", std::process::id()));
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)
            {
                Ok(file) => return Ok((temporary, file)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(CacheError::Io(error)),
            }
        }
        Err(CacheError::TemporaryNameExhausted)
    }

    fn evict_for(
        &self,
        target: &Path,
        own_temporary: &Path,
        new_size: u64,
    ) -> Result<bool, CacheError> {
        let mut entries = Vec::<(String, PathBuf, u64, bool)>::new();
        let mut total = 0_u64;
        match fs::read_dir(&self.directory) {
            Ok(directory) => {
                for item in directory {
                    let item = item?;
                    let path = item.path();
                    if path == target || path == own_temporary {
                        continue;
                    }
                    let metadata = match fs::symlink_metadata(&path) {
                        Ok(metadata) => metadata,
                        Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                        Err(error) => return Err(CacheError::Io(error)),
                    };
                    if !metadata.is_file() || metadata.file_type().is_symlink() {
                        continue;
                    }
                    let name = item.file_name().to_string_lossy().into_owned();
                    if !name.ends_with(".json") && !is_temporary_name(&name) {
                        continue;
                    }
                    let is_temporary = is_temporary_name(&name);
                    let active_temporary = is_temporary && temporary_owner_is_alive(&name);
                    if is_temporary && !active_temporary {
                        match fs::remove_file(&path) {
                            Ok(()) => continue,
                            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                            Err(error) => return Err(CacheError::Io(error)),
                        }
                    }
                    total = total.saturating_add(metadata.len());
                    entries.push((name, path, metadata.len(), !active_temporary));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(new_size <= self.max_bytes)
            }
            Err(error) => return Err(CacheError::Io(error)),
        }
        entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        for (_, path, size, evictable) in entries {
            if total <= self.max_bytes && new_size <= self.max_bytes.saturating_sub(total) {
                break;
            }
            if !evictable {
                continue;
            }
            remove_eviction_candidate(&path)?;
            total = total.saturating_sub(size);
        }
        Ok(total <= self.max_bytes && new_size <= self.max_bytes.saturating_sub(total))
    }
}

fn remove_eviction_candidate(path: &Path) -> Result<(), CacheError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CacheError::Io(error)),
    }
}

fn is_temporary_name(name: &str) -> bool {
    name.rsplit_once(".tmp-").is_some_and(|(_, owner)| {
        owner
            .split_once('-')
            .is_some_and(|(pid, _)| !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()))
    })
}

fn temporary_owner_is_alive(name: &str) -> bool {
    let Some((_, owner)) = name.rsplit_once(".tmp-") else {
        return false;
    };
    let Some((pid, _)) = owner.split_once('-') else {
        return false;
    };
    let Ok(pid) = pid.parse::<u32>() else {
        return false;
    };
    if pid == std::process::id() {
        return true;
    }
    Command::new("/bin/kill")
        .arg("-0")
        .arg(pid.to_string())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

impl Cache for CacheStore {
    fn lookup<T: DeserializeOwned>(
        &self,
        request: &CacheRequest<'_>,
    ) -> Result<CacheLookup<T>, CacheFault> {
        let key = cache_key(request, self.max_entry_bytes)
            .map_err(|error| CacheFault::new(error.to_string()))?;
        self.get(&key, self.max_entry_bytes, self.max_entry_bytes)
            .map(|value| match value {
                Some(value) => CacheLookup::Hit(value),
                None => CacheLookup::Miss,
            })
            .map_err(|error| CacheFault::new(error.to_string()))
    }

    fn record<T: Serialize>(
        &self,
        request: &CacheRequest<'_>,
        value: &T,
        eligibility: CacheEligibility,
    ) -> Result<DriverCacheStore, CacheFault> {
        let key = cache_key(request, self.max_entry_bytes)
            .map_err(|error| CacheFault::new(error.to_string()))?;
        let policy = match eligibility {
            CacheEligibility::Eligible => StorePolicy::Cacheable,
            CacheEligibility::NeverStore(reason) => StorePolicy::NeverStore(reason),
        };
        self.insert(&key, value, policy, self.max_entry_bytes)
            .map(|result| match result {
                StoreResult::Stored => DriverCacheStore::Stored,
                StoreResult::NeverStored => DriverCacheStore::NeverStore,
                StoreResult::OverCapacity => DriverCacheStore::NotStored,
            })
            .map_err(|error| CacheFault::new(error.to_string()))
    }
}

fn cache_key(request: &CacheRequest<'_>, max_preimage_bytes: u64) -> Result<CacheKey, CacheError> {
    let operation = match request.operation() {
        CacheOperation::Prove => Operation::Prove,
        CacheOperation::Analyze => Operation::Analyze,
        CacheOperation::Generate => Operation::Generate,
    };
    CacheKey::derive(KeyRequest {
        operation,
        package_id: request.package_id(),
        request: request.request(),
        backend: request.backend(),
        manifest: request.manifest(),
        target: request.target(),
        max_preimage_bytes,
    })
}

/// Errors while deriving a key or reading/writing an entry.
#[derive(Debug, Error)]
pub enum CacheError {
    /// The canonical encoder refused a key or entry value.
    #[error("canonical cache encoding failed: {0}")]
    Canonical(#[from] quire_canonical::Error),
    /// A JSON value could not be parsed or serialized.
    #[error("cache JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    /// An operating-system file operation failed.
    #[error("cache I/O failed: {0}")]
    Io(#[from] io::Error),
    /// A generate operation omitted its target and options.
    #[error("generate cache keys require a target and options")]
    MissingGenerationTarget,
    /// A non-generate operation supplied generation-specific inputs.
    #[error("non-generate cache keys cannot include generation inputs")]
    UnexpectedGenerationTarget,
    /// A canonical stored preimage was not UTF-8.
    #[error("stored cache preimage is not UTF-8")]
    InvalidStoredPreimage,
    /// The cache entry path had no filename component.
    #[error("cache entry path has no filename")]
    InvalidEntryPath,
    /// A unique temporary entry name could not be allocated.
    #[error("could not allocate a cache temporary filename")]
    TemporaryNameExhausted,
    /// The cache operation lock was poisoned by a panicking caller.
    #[error("cache operation lock is poisoned")]
    LockPoisoned,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn key_for(
        operation: Operation,
        package_id: &str,
        request: &Value,
        backend: &BackendId,
        digest: u8,
        target: Option<&Value>,
    ) -> CacheKey {
        CacheKey::derive(KeyRequest {
            operation,
            package_id,
            request,
            backend,
            manifest: ManifestDigest::from_digest([digest; 32]),
            target,
            max_preimage_bytes: 16_384,
        })
        .expect("key inputs are valid")
    }

    fn key(digest: u8) -> CacheKey {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let request = serde_json::json!({
            "items": [{"occurrence_key": "item-1", "kind": "integer"}],
            "work_limit": 20,
        });
        key_for(
            Operation::Prove,
            "package-1",
            &request,
            &backend,
            digest,
            None,
        )
    }

    fn admitted_request<'a>(
        registry: &'a quire_driver::AdmittedRegistry,
        backend: &qsl_route::BackendId,
        request: &'a Value,
    ) -> CacheRequest<'a> {
        CacheRequest::new(
            CacheOperation::Prove,
            "package-1",
            request,
            None,
            registry,
            backend,
        )
        .expect("selected provider is in the admitted registry")
    }

    fn temporary_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("quire-cache-test-{}-{nonce}", std::process::id()))
    }

    /// Trace: FR-012-AC-1..FR-012-AC-4.
    #[test]
    fn admitted_manifest_digest_changes_key_without_a_revision_input() {
        let unchanged_revision = "release-7";
        assert_ne!(key(1), key(2));
        assert_eq!(unchanged_revision, "release-7");
    }

    /// Trace: FR-007-AC-1..FR-007-AC-5 and FR-012-AC-6..FR-012-AC-8.
    #[test]
    fn cache_trait_uses_the_admitted_registry_descriptor() {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let digest = ManifestDigest::from_digest([4; 32]);
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: digest,
            advertises: Vec::new(),
        }])
        .expect("descriptor is admitted");
        let request_value = serde_json::json!({"items": [], "budget": 20});
        let cache_request = admitted_request(&registry, &backend, &request_value);
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 4096, 8192);
        assert_eq!(
            Cache::lookup::<Value>(&store, &cache_request).expect("initial lookup succeeds"),
            CacheLookup::Miss
        );
        let value = serde_json::json!({"terminal": "incomplete", "consumed": 20});
        assert_eq!(
            Cache::record(&store, &cache_request, &value, CacheEligibility::Eligible)
                .expect("record succeeds"),
            DriverCacheStore::Stored
        );
        assert_eq!(
            Cache::lookup::<Value>(&store, &cache_request).expect("typed lookup succeeds"),
            CacheLookup::Hit(value)
        );
        assert_eq!(
            Cache::lookup::<u64>(&store, &cache_request).expect("wrong typed value is a miss"),
            CacheLookup::Miss
        );
        let _ = fs::remove_dir_all(directory);
    }

    /// Trace: FR-012-AC-1..FR-012-AC-5; target exists only for generate.
    #[test]
    fn target_and_meaning_members_match_the_operation_contract() {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let request = serde_json::json!({"items": [], "work_limit": 20});
        let target_a = serde_json::json!({"name": "wasm32-wasi", "options": {"opt": 1}});
        let target_b = serde_json::json!({"name": "wasm32-wasi", "options": {"opt": 2}});
        let prove = key_for(Operation::Prove, "package-1", &request, &backend, 1, None);
        let analyze = key_for(Operation::Analyze, "package-1", &request, &backend, 1, None);
        let generated_a = key_for(
            Operation::Generate,
            "package-1",
            &request,
            &backend,
            1,
            Some(&target_a),
        );
        let generated_b = key_for(
            Operation::Generate,
            "package-1",
            &request,
            &backend,
            1,
            Some(&target_b),
        );
        assert_ne!(prove, analyze);
        assert_ne!(generated_a, generated_b);
        let prove_preimage: Value = serde_json::from_slice(prove.preimage()).expect("JCS object");
        let generated_preimage: Value =
            serde_json::from_slice(generated_a.preimage()).expect("JCS object");
        assert!(prove_preimage.get("target").is_none());
        assert!(generated_preimage.get("target").is_some());
        assert_eq!(prove_preimage.as_object().expect("object").len(), 5);
        assert_eq!(generated_preimage.as_object().expect("object").len(), 6);
        let expected = serde_json::json!({
            "operation": "prove",
            "package_id": "package-1",
            "request": request,
            "backend": "agent-ix/provider/test",
            "manifest": {
                "domain": ManifestDigest::DOMAIN.as_str(),
                "digest": ManifestDigest::from_digest([1; 32]).hex(),
            },
        });
        assert_eq!(prove_preimage, expected);
        assert_ne!(
            prove,
            key_for(
                Operation::Prove,
                "package-2",
                &serde_json::json!({"items": [], "work_limit": 20}),
                &backend,
                1,
                None,
            )
        );
        assert_ne!(
            prove,
            key_for(
                Operation::Prove,
                "package-1",
                &serde_json::json!({"items": [], "work_limit": 21}),
                &backend,
                1,
                None,
            )
        );
        let other_backend = BackendId::from_wire("agent-ix/provider/other").expect("valid id");
        assert_ne!(
            prove,
            key_for(
                Operation::Prove,
                "package-1",
                &request,
                &other_backend,
                1,
                None,
            )
        );
    }

    /// Trace: FR-012-AC-1..FR-012-AC-4.
    #[test]
    fn key_pins_backend_identity_and_admitted_digest() {
        let first = key(1);
        let second = key(1);
        assert_eq!(first, second);
        assert_ne!(first, key(2));
    }

    /// Trace: FR-012-AC-6..FR-012-AC-8; includes a deterministic budget-incomplete value.
    #[test]
    fn miss_store_typed_hit_and_never_store() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 2048, 4096);
        let cache_key = key(1);
        assert_eq!(
            store
                .get::<serde_json::Value>(&cache_key, 2048, 16_384)
                .expect("cache read succeeds"),
            None
        );
        let result = serde_json::json!({
            "terminal": "incomplete",
            "kind": "work_limit",
            "configured": 20,
            "consumed": 20,
        });
        assert_eq!(
            store
                .insert(&cache_key, &result, StorePolicy::Cacheable, 2048)
                .expect("cache write succeeds"),
            StoreResult::Stored
        );
        assert_eq!(
            store
                .get::<serde_json::Value>(&cache_key, 2048, 16_384)
                .expect("cache read succeeds"),
            Some(result)
        );
        let exclusions = [
            CacheExclusion::InternalFailure,
            CacheExclusion::Cancelled,
            CacheExclusion::Deadline,
            CacheExclusion::TimeLimitedIncomplete,
            CacheExclusion::PluginResult,
            CacheExclusion::ContainsWallTime,
        ];
        for (index, reason) in exclusions.into_iter().enumerate() {
            let excluded = key((index + 2) as u8);
            assert_eq!(
                store
                    .insert(
                        &excluded,
                        &serde_json::json!({"terminal": "proved"}),
                        StorePolicy::NeverStore(reason),
                        2048,
                    )
                    .expect("excluded result is not a write error"),
                StoreResult::NeverStored
            );
            assert_eq!(
                store
                    .get::<serde_json::Value>(&excluded, 2048, 16_384)
                    .expect("cache read succeeds"),
                None
            );
        }
        let _ = fs::remove_dir_all(directory);
    }

    /// Trace: FR-012-AC-9 and FR-012-AC-12.
    #[test]
    fn invalid_typed_value_and_changed_preimage_are_misses() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 2048, 4096);
        let cache_key = key(1);
        store
            .insert(
                &cache_key,
                &serde_json::json!({"count": "not a number"}),
                StorePolicy::Cacheable,
                2048,
            )
            .expect("cache write succeeds");
        assert_eq!(
            store
                .get::<u64>(&cache_key, 2048, 16_384)
                .expect("cache read succeeds"),
            None
        );
        let path = directory.join(format!("{}.json", cache_key.as_str()));
        fs::write(path, b"{\"preimage\":\"{}\",\"value\":3}")
            .expect("corrupt test entry is writable");
        assert_eq!(
            store
                .get::<u64>(&cache_key, 2048, 16_384)
                .expect("corrupt entry is a miss"),
            None
        );
        let _ = fs::remove_dir_all(directory);
    }

    /// Trace: FR-012-AC-6; malformed, noncanonical, misplaced, oversized and mismatched entries are misses.
    #[test]
    fn malformed_and_mismatched_entries_are_misses() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 512, 4096);
        fs::create_dir_all(&directory).expect("test directory is created");
        let requested = key(20);
        let requested_path = store.entry_path(&requested);

        fs::write(&requested_path, b"not json").expect("malformed entry is written");
        assert_eq!(
            store
                .get::<Value>(&requested, 512, 512)
                .expect("read succeeds"),
            None
        );

        let preimage = std::str::from_utf8(requested.preimage()).expect("preimage is UTF-8");
        let noncanonical = format!("{{ \"preimage\":{preimage:?}, \"value\":1 }}");
        fs::write(&requested_path, noncanonical).expect("noncanonical entry is written");
        assert_eq!(
            store
                .get::<Value>(&requested, 512, 512)
                .expect("read succeeds"),
            None
        );

        let other = key(21);
        let other_path = store.entry_path(&other);
        let valid_for_requested = serde_json::json!({"preimage": preimage, "value": 1});
        fs::write(
            &other_path,
            to_vec(&valid_for_requested, Limits::new(512)).expect("entry encodes"),
        )
        .expect("wrongly named entry is written");
        assert_eq!(
            store.get::<Value>(&other, 512, 512).expect("read succeeds"),
            None
        );

        fs::write(&requested_path, vec![b'x'; 513]).expect("oversized entry is written");
        assert_eq!(
            store
                .get::<Value>(&requested, 512, 512)
                .expect("read succeeds"),
            None
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn entries_over_the_store_capacity_are_not_written() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 2048, 8);
        let cache_key = key(1);
        assert_eq!(
            store
                .insert(
                    &cache_key,
                    &serde_json::json!({"terminal": "proved"}),
                    StorePolicy::Cacheable,
                    2048,
                )
                .expect("oversize write is a normal miss"),
            StoreResult::OverCapacity
        );
        assert!(!directory.exists());
    }

    /// Trace: FR-012-AC-9; an entry above its separate per-entry limit is NotStored.
    #[test]
    fn entries_over_the_per_entry_limit_are_not_written() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 8, 4096);
        let result = store
            .insert(
                &key(1),
                &serde_json::json!({"value": "too large"}),
                StorePolicy::Cacheable,
                8,
            )
            .expect("oversize entry is a normal non-store result");
        assert_eq!(result, StoreResult::OverCapacity);
        assert!(!directory.exists());
    }

    /// Trace: FR-012-AC-9 and FR-012-AC-12; entry exactly at both limits is accepted.
    #[test]
    fn exactly_at_configured_limits_is_stored() {
        let directory = temporary_directory();
        let cache_key = key(12);
        let value = serde_json::json!({"value": "bounded"});
        let preimage = std::str::from_utf8(cache_key.preimage()).expect("key preimage is UTF-8");
        let entry = serde_json::json!({"preimage": preimage, "value": value.clone()});
        let encoded = to_vec(&entry, Limits::new(u64::MAX)).expect("entry encodes");
        let exact_limit = encoded.len() as u64;
        let store = CacheStore::new(&directory, exact_limit, exact_limit);
        assert_eq!(
            store
                .insert(&cache_key, &value, StorePolicy::Cacheable, exact_limit)
                .expect("exact boundary insertion succeeds"),
            StoreResult::Stored
        );
        let stored = fs::read(store.entry_path(&cache_key)).expect("published entry exists");
        assert_eq!(stored.len() as u64, exact_limit);
        let entries = fs::read_dir(&directory)
            .expect("store can be inspected")
            .map(|entry| entry.expect("directory entry is readable").file_name())
            .collect::<Vec<_>>();
        assert_eq!(
            entries.len(),
            1,
            "the store has entries only, no index or ledger"
        );
        assert!(entries[0].to_string_lossy().ends_with(".json"));
        let _ = fs::remove_dir_all(directory);
    }

    /// Trace: FR-012-AC-10 and FR-012-AC-12; temporary bytes count, abandoned files are cleaned, symlinks are not followed.
    #[test]
    fn temporary_capacity_and_abandoned_cleanup_are_enforced() {
        let directory = temporary_directory();
        fs::create_dir_all(&directory).expect("test directory is created");
        let abandoned = directory.join(".abandoned.json.tmp-999999999-1");
        fs::write(&abandoned, vec![b'x'; 150]).expect("stale temporary file is created");
        let active = directory.join(format!(".active.json.tmp-{}-1", std::process::id()));
        fs::write(&active, vec![b'y'; 650]).expect("live temporary file is created");
        let outside = temporary_directory().with_extension("outside");
        fs::write(&outside, b"outside").expect("outside target is created");
        let link = directory.join(format!("{}.json", key(9).as_str()));
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).expect("symlink is created");
        let store = CacheStore::new(&directory, 2048, 700);
        #[cfg(unix)]
        assert_eq!(
            store
                .get::<Value>(&key(9), 2048, 2048)
                .expect("symlink is a miss"),
            None
        );
        let result = store
            .insert(
                &key(1),
                &serde_json::json!({"value": "fits alone"}),
                StorePolicy::Cacheable,
                2048,
            )
            .expect("capacity refusal is not an I/O error");
        assert_eq!(result, StoreResult::OverCapacity);
        assert!(!abandoned.exists(), "dead-owner temporary file is removed");
        assert!(
            active.exists(),
            "active temporary file is counted, not removed"
        );
        assert_eq!(
            fs::read(&outside).expect("outside file remains"),
            b"outside"
        );
        let _ = fs::remove_dir_all(&directory);
        let _ = fs::remove_file(outside);
    }

    /// Trace: FR-012-AC-10; concurrent same-key writes publish complete entries.
    #[test]
    fn concurrent_same_key_writers_leave_a_valid_entry() {
        let directory = temporary_directory();
        let store = std::sync::Arc::new(CacheStore::new(&directory, 4096, 16_384));
        let cache_key = key(1);
        let mut writers = Vec::new();
        let directory_path = directory.clone();
        for value in 0..6_u64 {
            let directory = directory_path.clone();
            let cache_key = cache_key.clone();
            writers.push(std::thread::spawn(move || {
                CacheStore::new(directory, 4096, 16_384)
                    .insert(
                        &cache_key,
                        &serde_json::json!({"value": value}),
                        StorePolicy::Cacheable,
                        4096,
                    )
                    .expect("writer succeeds")
            }));
        }
        for writer in writers {
            assert_eq!(
                writer.join().expect("writer thread completes"),
                StoreResult::Stored
            );
        }
        let hit = store
            .get::<serde_json::Value>(&cache_key, 4096, 4096)
            .expect("final entry is readable")
            .expect("final entry exists");
        assert!(hit.get("value").and_then(Value::as_u64).is_some());
        let _ = fs::remove_dir_all(directory);
    }

    /// Trace: FR-012-AC-11; a candidate removed by another process is tolerated.
    #[test]
    fn missing_eviction_candidate_is_tolerated() {
        let directory = temporary_directory();
        fs::create_dir_all(&directory).expect("test directory is created");
        let candidate = directory.join("already-evicted.json");
        fs::write(&candidate, b"entry").expect("candidate is created");
        fs::remove_file(&candidate).expect("another process removed the candidate");
        assert!(remove_eviction_candidate(&candidate).is_ok());
        let _ = fs::remove_dir_all(directory);
    }
}
