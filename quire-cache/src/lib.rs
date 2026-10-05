// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's content-addressed provider-result cache.
//!
//! The cache key includes the selected provider's manifest content identity,
//! computed from its canonical FR-331 bytes. A caller-supplied revision or
//! version string is not a cache input. Values are stored only when the caller
//! classifies a completed first-party result as cacheable; plugin results,
//! failures, cancellation, deadlines and time-limited results are never stored.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use quire_canonical::{sha256_with_domain, to_vec, Limits, Sha256Digest};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

/// FR-201 content-identity domain for cache keys.
pub const CACHE_KEY_DOMAIN: &[u8] = b"quire.cache-key/v1";

/// FR-201 content-identity domain for a canonical FR-331 provider manifest.
pub const PROVIDER_MANIFEST_DOMAIN: &[u8] = b"quire.tool-manifest.jcs/v1";

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

/// The target-specific key inputs required for a generate operation.
#[derive(Clone, Copy, Debug)]
pub struct GenerationTarget<'a> {
    /// The selected generation target.
    pub target: &'a str,
    /// Every target option that can change the generated artifact.
    pub options: &'a Value,
}

/// Inputs used to derive one cache key.
///
/// `request` is the typed operation request serialized to a JSON value. It
/// must include each meaning input and deterministic budget for the operation.
/// `provider_manifest_bytes` is the exact canonical FR-331 manifest supplied
/// for the selected provider.
#[derive(Clone, Copy, Debug)]
pub struct KeyRequest<'a> {
    /// The operation whose value may be reused.
    pub operation: Operation,
    /// The checked package's content identity.
    pub package_id: &'a str,
    /// The typed operation request, serialized as JSON.
    pub request: &'a Value,
    /// The selected provider's backend identity.
    pub provider_identity: &'a str,
    /// The exact canonical FR-331 manifest bytes for that provider.
    pub provider_manifest_bytes: &'a [u8],
    /// Generate target and options; required only for `Generate`.
    pub generation: Option<GenerationTarget<'a>>,
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
    /// and exact canonical manifest content.
    ///
    /// The manifest must already be canonical FR-331 JSON. The manifest's
    /// content digest is computed from those bytes; no recorded version, git
    /// revision or caller-supplied digest participates in this key.
    pub fn derive(request: KeyRequest<'_>) -> Result<Self, CacheError> {
        if request.operation == Operation::Generate && request.generation.is_none() {
            return Err(CacheError::MissingGenerationTarget);
        }
        if request.operation != Operation::Generate && request.generation.is_some() {
            return Err(CacheError::UnexpectedGenerationTarget);
        }
        let limits = Limits::new(request.max_preimage_bytes);
        let manifest: Value = serde_json::from_slice(request.provider_manifest_bytes)?;
        let canonical_manifest = to_vec(&manifest, limits)?;
        if canonical_manifest != request.provider_manifest_bytes {
            return Err(CacheError::NonCanonicalManifest);
        }
        let manifest_identity = manifest
            .get("identity")
            .and_then(Value::as_str)
            .ok_or(CacheError::MissingManifestIdentity)?;
        if manifest_identity != request.provider_identity {
            return Err(CacheError::ManifestIdentityMismatch);
        }
        let manifest_digest = sha256_with_domain(PROVIDER_MANIFEST_DOMAIN, &manifest, limits)?;
        let generation = request.generation.map(|target| {
            serde_json::json!({
                "target": target.target,
                "options": target.options,
            })
        });
        let preimage_value = serde_json::json!({
            "operation": request.operation.as_str(),
            "package_id": request.package_id,
            "request": request.request,
            "provider": {
                "identity": request.provider_identity,
                "manifest_digest": {
                    "domain": "quire.tool-manifest.jcs/v1",
                    "digest": manifest_digest.to_string(),
                },
            },
            "generation": generation,
        });
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

/// A reason an operation result is excluded from storage by ADR-029 CA-5.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NeverStoreReason {
    /// The result category is internal failure.
    InternalFailure,
    /// The caller cancelled the operation.
    Cancelled,
    /// The operation ended at a deadline.
    Deadline,
    /// A time-limited result is incomplete.
    TimedOut,
    /// The result came from a process plugin.
    PluginResult,
    /// The value records wall-clock time.
    ContainsWallTime,
}

/// Whether a typed result may be inserted into the cache.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorePolicy {
    /// A completed first-party result eligible under CA-1 and CA-5.
    Cacheable,
    /// A result that CA-5 forbids storing.
    NeverStore(NeverStoreReason),
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
    max_bytes: u64,
    operation: Mutex<()>,
}

impl CacheStore {
    /// Open a cache rooted at the caller-named `directory` with a total byte
    /// ceiling. The directory is created on the first successful insertion.
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>, max_bytes: u64) -> Self {
        Self {
            directory: directory.into(),
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
        let file = File::open(path)?;
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
        let bytes = to_vec(&entry, Limits::new(max_entry_bytes))?;
        if bytes.len() as u64 > self.max_bytes {
            return Ok(StoreResult::OverCapacity);
        }
        fs::create_dir_all(&self.directory)?;
        let path = self.entry_path(key);
        self.evict_for(&path, bytes.len() as u64)?;
        let (temporary, mut file) = self.create_temporary(&path)?;
        let result = (|| -> Result<(), CacheError> {
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result?;
        Ok(StoreResult::Stored)
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

    fn evict_for(&self, target: &Path, new_size: u64) -> Result<(), CacheError> {
        let mut entries = Vec::<(String, PathBuf, u64)>::new();
        let mut total = 0_u64;
        match fs::read_dir(&self.directory) {
            Ok(directory) => {
                for item in directory {
                    let item = item?;
                    let path = item.path();
                    if path == target || path.extension().is_none_or(|ext| ext != "json") {
                        continue;
                    }
                    let metadata = fs::symlink_metadata(&path)?;
                    if !metadata.is_file() || metadata.file_type().is_symlink() {
                        continue;
                    }
                    let name = item.file_name().to_string_lossy().into_owned();
                    total = total.saturating_add(metadata.len());
                    entries.push((name, path, metadata.len()));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(CacheError::Io(error)),
        }
        entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
        for (_, path, size) in entries {
            if total <= self.max_bytes && new_size <= self.max_bytes.saturating_sub(total) {
                break;
            }
            fs::remove_file(path)?;
            total = total.saturating_sub(size);
        }
        if total > self.max_bytes || new_size > self.max_bytes.saturating_sub(total) {
            return Err(CacheError::EvictionFailed);
        }
        Ok(())
    }
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
    /// The supplied manifest bytes are valid JSON but not canonical JSON.
    #[error("provider manifest bytes are not canonical FR-331 JSON")]
    NonCanonicalManifest,
    /// The provider manifest did not contain a string identity.
    #[error("provider manifest has no string identity")]
    MissingManifestIdentity,
    /// The selected backend identity did not match the manifest identity.
    #[error("selected provider identity does not match the FR-331 manifest")]
    ManifestIdentityMismatch,
    /// A canonical stored preimage was not UTF-8.
    #[error("stored cache preimage is not UTF-8")]
    InvalidStoredPreimage,
    /// The cache entry path had no filename component.
    #[error("cache entry path has no filename")]
    InvalidEntryPath,
    /// A unique temporary entry name could not be allocated.
    #[error("could not allocate a cache temporary filename")]
    TemporaryNameExhausted,
    /// Eviction could not make enough room under the configured limit.
    #[error("cache eviction could not satisfy the configured size limit")]
    EvictionFailed,
    /// The cache operation lock was poisoned by a panicking caller.
    #[error("cache operation lock is poisoned")]
    LockPoisoned,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn manifest(mode: &str) -> Vec<u8> {
        let value = serde_json::json!({
            "contract_version": "quire.backend-provider/v1",
            "identity": "agent-ix/provider/test",
            "advertises": [{"kind": "integer", "mode": mode}],
        });
        to_vec(&value, Limits::new(4096)).expect("manifest is canonicalizable")
    }

    fn key(manifest_bytes: &[u8]) -> CacheKey {
        let request = serde_json::json!({
            "items": [{"occurrence_key": "item-1", "kind": "integer"}],
            "work_limit": 20,
        });
        CacheKey::derive(KeyRequest {
            operation: Operation::Prove,
            package_id: "package-1",
            request: &request,
            provider_identity: "agent-ix/provider/test",
            provider_manifest_bytes: manifest_bytes,
            generation: None,
            max_preimage_bytes: 16_384,
        })
        .expect("key inputs are valid")
    }

    fn temporary_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("quire-cache-test-{}-{nonce}", std::process::id()))
    }

    #[test]
    fn manifest_content_changes_the_key_without_a_revision_input() {
        let unchanged_revision = "release-7";
        let first = manifest("bounded");
        let second = manifest("unbounded");
        assert_ne!(first, second);
        assert_ne!(key(&first), key(&second));
        assert_eq!(unchanged_revision, "release-7");
    }

    #[test]
    fn key_rejects_a_manifest_for_a_different_provider_identity() {
        let manifest_bytes = manifest("bounded");
        let request = serde_json::json!({"items": [], "work_limit": 20});
        let result = CacheKey::derive(KeyRequest {
            operation: Operation::Prove,
            package_id: "package-1",
            request: &request,
            provider_identity: "agent-ix/provider/other",
            provider_manifest_bytes: &manifest_bytes,
            generation: None,
            max_preimage_bytes: 16_384,
        });
        assert!(matches!(result, Err(CacheError::ManifestIdentityMismatch)));
    }

    #[test]
    fn miss_store_typed_hit_and_never_store() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 4096);
        let cache_key = key(&manifest("bounded"));
        assert_eq!(
            store
                .get::<serde_json::Value>(&cache_key, 2048, 16_384)
                .expect("cache read succeeds"),
            None
        );
        let result = serde_json::json!({"terminal": "proved", "backend": "kani"});
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
        let other = key(&manifest("unbounded"));
        assert_eq!(
            store
                .insert(
                    &other,
                    &serde_json::json!({"terminal": "proved"}),
                    StorePolicy::NeverStore(NeverStoreReason::PluginResult),
                    2048,
                )
                .expect("excluded result is not a write error"),
            StoreResult::NeverStored
        );
        assert_eq!(
            store
                .get::<serde_json::Value>(&other, 2048, 16_384)
                .expect("cache read succeeds"),
            None
        );
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn invalid_typed_value_and_changed_preimage_are_misses() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 4096);
        let cache_key = key(&manifest("bounded"));
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

    #[test]
    fn entries_over_the_store_capacity_are_not_written() {
        let directory = temporary_directory();
        let store = CacheStore::new(&directory, 8);
        let cache_key = key(&manifest("bounded"));
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
}
