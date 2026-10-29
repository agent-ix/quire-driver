// SPDX-License-Identifier: AGPL-3.0-or-later
//! Downstream implementation of the driver's structural-key cache boundary.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

#[cfg(test)]
use qsl_foundation::BackendId;
#[cfg(test)]
use qsl_route::ManifestDigest;
use quire_canonical::{to_vec, Limits};
use quire_driver::{
    Cache, CacheEligibility, CacheFault, CacheLookup, CacheOperation, CacheRequest,
    CacheValueCodec, CacheWriteOutcome,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const STALE_TEMPORARY_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// A structural cache key: the canonical JSON preimage itself, never hashed.
#[derive(Clone, Debug, Eq, PartialEq)]
struct CacheKey {
    preimage: Vec<u8>,
}

impl CacheKey {
    /// Derive a key from the operation, package, request and provider
    /// identity. No digest, recorded version, git revision or caller-supplied
    /// spelling participates in this key.
    fn derive(request: &CacheRequest<'_>, max_preimage_bytes: u64) -> Result<Self, CacheError> {
        if request.operation() == CacheOperation::Generate && request.target().is_none() {
            return Err(CacheError::MissingGenerationTarget);
        }
        if request.operation() != CacheOperation::Generate && request.target().is_some() {
            return Err(CacheError::UnexpectedGenerationTarget);
        }
        let limits = Limits::new(max_preimage_bytes);
        let request_value = meaning_only(serde_json::from_slice(request.request())?);
        let mut preimage_value = serde_json::json!({
            "operation": match request.operation() {
                CacheOperation::Prove => "prove",
                CacheOperation::Analyze => "analyze",
                CacheOperation::Generate => "generate",
            },
            "package_id": request.package_id(),
            "request": request_value,
            "backend": request.backend().as_str(),
        });
        if let Some(target) = request.target() {
            preimage_value["target"] = serde_json::from_slice(target)?;
        }
        let preimage = to_vec(&preimage_value, limits)?;
        Ok(Self { preimage })
    }

    #[cfg(test)]
    fn preimage(&self) -> &[u8] {
        &self.preimage
    }
}

fn meaning_only(value: Value) -> Value {
    const EXCLUDED: &[&str] = &[
        "path",
        "timestamp",
        "deadline",
        "pin",
        "revision",
        "applicationversion",
        "walltime",
        "executionbackend",
    ];
    match value {
        Value::Array(values) => Value::Array(values),
        Value::Object(object) => Value::Object(
            object
                .into_iter()
                .filter(|(key, _)| {
                    let normalized = key.to_ascii_lowercase().replace(['_', '-'], "");
                    !EXCLUDED.contains(&normalized.as_str())
                })
                .collect(),
        ),
        other => other,
    }
}

/// The caller-named on-disk cache directory and its maximum size.
pub struct DirectoryCache {
    directory: PathBuf,
    max_entry_bytes: u64,
    max_bytes: u64,
    operation: Mutex<()>,
}

/// Canonical JSON codec used by the on-disk cache for typed values.
#[derive(Clone, Copy, Debug, Default)]
pub struct CanonicalJsonCodec;

impl<T> CacheValueCodec<T> for CanonicalJsonCodec
where
    T: Serialize + DeserializeOwned,
{
    fn encode(&self, value: &T, max_bytes: u64) -> Result<Vec<u8>, quire_driver::CacheCodecError> {
        let json = serde_json::to_value(value)
            .map_err(|error| Box::new(error) as quire_driver::CacheCodecError)?;
        to_vec(&json, Limits::new(max_bytes))
            .map_err(|error| Box::new(error) as quire_driver::CacheCodecError)
    }

    fn decode(&self, bytes: &[u8]) -> Result<T, quire_driver::CacheCodecError> {
        serde_json::from_slice(bytes)
            .map_err(|error| Box::new(error) as quire_driver::CacheCodecError)
    }
}

impl DirectoryCache {
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
    /// cache miss. An owned entry is accepted only when its canonical bytes
    /// are valid and its stored preimage equals the requested preimage.
    fn read_value(&self, key: &CacheKey) -> Result<Option<Value>, CacheError> {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| CacheError::LockPoisoned)?;
        let Some(path) = self.find_entry(key)? else {
            return Ok(None);
        };
        Ok(self
            .read_entry(&path)?
            .filter(|(preimage, _)| preimage.as_bytes() == key.preimage)
            .map(|(_, value)| value))
    }

    /// The owned entry file names in the cache directory, with their sequence.
    fn owned_names(&self) -> Result<Vec<(u64, PathBuf)>, CacheError> {
        let mut names = Vec::new();
        match fs::read_dir(&self.directory) {
            Ok(directory) => {
                for item in directory {
                    let item = item?;
                    let name = item.file_name().to_string_lossy().into_owned();
                    if let Some(sequence) = entry_sequence(&name) {
                        names.push((sequence, item.path()));
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(CacheError::Io(error)),
        }
        names.sort();
        Ok(names)
    }

    /// The valid owned entry whose stored preimage equals the key's.
    fn find_entry(&self, key: &CacheKey) -> Result<Option<PathBuf>, CacheError> {
        for (_, path) in self.owned_names()? {
            if let Some((preimage, _)) = self.read_entry(&path)? {
                if preimage.as_bytes() == key.preimage {
                    return Ok(Some(path));
                }
            }
        }
        Ok(None)
    }

    /// Read one entry file: its stored preimage and typed value, or `None`
    /// when the file is not a valid canonical entry within the size limit.
    fn read_entry(&self, path: &Path) -> Result<Option<(String, Value)>, CacheError> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(CacheError::Io(error)),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Ok(None);
        }
        if metadata.len() > self.max_entry_bytes {
            return Ok(None);
        }
        let file = match open_regular_file_no_follow(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(CacheError::Io(error)),
        };
        let mut bytes = Vec::new();
        file.take(self.max_entry_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > self.max_entry_bytes {
            return Ok(None);
        }
        let Ok(entry) = serde_json::from_slice::<Value>(&bytes) else {
            return Ok(None);
        };
        match to_vec(&entry, Limits::new(self.max_entry_bytes)) {
            Ok(canonical) if canonical == bytes => {}
            _ => return Ok(None),
        }
        let Some(object) = entry.as_object() else {
            return Ok(None);
        };
        if object.len() != 2 {
            return Ok(None);
        }
        let (Some(preimage), Some(value)) = (
            object.get("preimage").and_then(Value::as_str),
            object.get("value"),
        ) else {
            return Ok(None);
        };
        let Ok(preimage_value) = serde_json::from_str::<Value>(preimage) else {
            return Ok(None);
        };
        match to_vec(&preimage_value, Limits::new(self.max_entry_bytes)) {
            Ok(canonical) if canonical.as_slice() == preimage.as_bytes() => {}
            _ => return Ok(None),
        }
        Ok(Some((preimage.to_owned(), value.clone())))
    }

    /// Store a typed value if its policy permits caching.
    ///
    /// The entry is written to a temporary file and atomically renamed into
    /// place. Eviction counts every regular file in the caller-named directory
    /// but removes only validated cache entries and owned temporary files.
    /// Entry validation currently runs under the process-local operation lock
    /// so an insertion sees a consistent eviction decision.
    fn insert_value(&self, key: &CacheKey, value: Value) -> Result<CacheWriteOutcome, CacheError> {
        self.insert_value_before_publish(key, value, |_, _| {})
    }

    fn insert_value_before_publish<F>(
        &self,
        key: &CacheKey,
        value: Value,
        before_publish: F,
    ) -> Result<CacheWriteOutcome, CacheError>
    where
        F: FnOnce(&Path, &Path),
    {
        let _guard = self
            .operation
            .lock()
            .map_err(|_| CacheError::LockPoisoned)?;
        let preimage =
            std::str::from_utf8(&key.preimage).map_err(|_| CacheError::InvalidStoredPreimage)?;
        let entry = serde_json::json!({
            "preimage": preimage,
            "value": value,
        });
        let bytes = to_vec(&entry, Limits::new(u64::MAX))?;
        if bytes.len() as u64 > self.max_entry_bytes || bytes.len() as u64 > self.max_bytes {
            return Ok(CacheWriteOutcome::NotStored);
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
            before_publish(&temporary, &path);
            match fs::rename(&temporary, &path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
                Err(error) => return Err(CacheError::Io(error)),
            }
            Ok(true)
        })();
        if !matches!(&result, Ok(true)) {
            let _ = fs::remove_file(&temporary);
        }
        match result? {
            true => Ok(CacheWriteOutcome::Stored),
            false => Ok(CacheWriteOutcome::NotStored),
        }
    }

    /// The path of the entry holding this key, or else the next unused
    /// sequence name. Names carry no key meaning.
    fn entry_path(&self, key: &CacheKey) -> PathBuf {
        if let Ok(Some(path)) = self.find_entry(key) {
            return path;
        }
        let next = self
            .owned_names()
            .ok()
            .and_then(|names| names.last().map(|(sequence, _)| sequence.saturating_add(1)))
            .unwrap_or(0);
        self.directory.join(format!("entry-{next}.json"))
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
        self.evict_for_with_hook(target, own_temporary, new_size, |_| {})
    }

    fn evict_for_with_hook<F>(
        &self,
        target: &Path,
        own_temporary: &Path,
        new_size: u64,
        before_remove: F,
    ) -> Result<bool, CacheError>
    where
        F: Fn(&Path),
    {
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
                    let is_temporary = is_temporary_name(&name);
                    let active_temporary =
                        is_temporary && !temporary_is_stale(&path, SystemTime::now());
                    if is_temporary && !active_temporary {
                        match fs::remove_file(&path) {
                            Ok(()) => continue,
                            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                            Err(error) => return Err(CacheError::Io(error)),
                        }
                    }
                    let owned_entry = !is_temporary && self.is_owned_entry(&path, &name);
                    total = total.saturating_add(metadata.len());
                    entries.push((
                        name,
                        path,
                        metadata.len(),
                        if is_temporary {
                            !active_temporary
                        } else {
                            owned_entry
                        },
                    ));
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
            before_remove(&path);
            remove_eviction_candidate(&path)?;
            total = total.saturating_sub(size);
        }
        Ok(total <= self.max_bytes && new_size <= self.max_bytes.saturating_sub(total))
    }

    fn is_owned_entry(&self, path: &Path, name: &str) -> bool {
        entry_sequence(name).is_some() && matches!(self.read_entry(path), Ok(Some(_)))
    }
}

/// The sequence of an owned entry name `entry-<decimal>.json`.
fn entry_sequence(name: &str) -> Option<u64> {
    let digits = name.strip_prefix("entry-")?.strip_suffix(".json")?;
    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| digits.parse().ok())
        .flatten()
}

fn remove_eviction_candidate(path: &Path) -> Result<(), CacheError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CacheError::Io(error)),
    }
}

fn is_temporary_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('.') else {
        return false;
    };
    let Some((key, suffix)) = rest.split_once(".json.tmp-") else {
        return false;
    };
    let key = format!("{key}.json");
    entry_sequence(&key).is_some()
        && suffix.split_once('-').is_some_and(|(pid, sequence)| {
            !pid.is_empty()
                && pid.bytes().all(|byte| byte.is_ascii_digit())
                && !sequence.is_empty()
                && sequence.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn temporary_is_stale(path: &Path, now: SystemTime) -> bool {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| now.duration_since(modified).ok())
        .is_some_and(|age| age > STALE_TEMPORARY_AGE)
}

fn open_regular_file_no_follow(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cache entry is not a regular file",
        ));
    }
    Ok(file)
}

impl Cache for DirectoryCache {
    fn lookup<T>(
        &self,
        request: &CacheRequest<'_>,
        codec: &dyn CacheValueCodec<T>,
    ) -> Result<CacheLookup<T>, CacheFault> {
        let key = match CacheKey::derive(request, self.max_entry_bytes) {
            Ok(key) => key,
            Err(_) => return Ok(CacheLookup::Miss),
        };
        let Some(value) = self.read_value(&key).map_err(cache_fault)? else {
            return Ok(CacheLookup::Miss);
        };
        let encoded = match to_vec(&value, Limits::new(self.max_entry_bytes)) {
            Ok(encoded) => encoded,
            Err(_) => return Ok(CacheLookup::Miss),
        };
        Ok(codec
            .decode(&encoded)
            .map(CacheLookup::Hit)
            .unwrap_or(CacheLookup::Miss))
    }

    fn record<T>(
        &self,
        request: &CacheRequest<'_>,
        value: &T,
        codec: &dyn CacheValueCodec<T>,
        eligibility: CacheEligibility,
    ) -> Result<CacheWriteOutcome, CacheFault> {
        if let CacheEligibility::NeverStore(_) = eligibility {
            return Ok(CacheWriteOutcome::NeverStore);
        }
        let key = match CacheKey::derive(request, self.max_entry_bytes) {
            Ok(key) => key,
            Err(_) => return Ok(CacheWriteOutcome::NotStored),
        };
        let encoded = match codec.encode(value, self.max_entry_bytes) {
            Ok(encoded) => encoded,
            Err(_) => return Ok(CacheWriteOutcome::NotStored),
        };
        if u64::try_from(encoded.len()).unwrap_or(u64::MAX) > self.max_entry_bytes {
            return Ok(CacheWriteOutcome::NotStored);
        }
        let value = match serde_json::from_slice(&encoded) {
            Ok(value) => value,
            Err(_) => return Ok(CacheWriteOutcome::NotStored),
        };
        self.insert_value(&key, value).map_err(cache_fault)
    }
}

fn cache_fault(error: CacheError) -> CacheFault {
    match error {
        CacheError::Io(error) => CacheFault::Io(error),
        other => CacheFault::from_error(other),
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
    use quire_driver::CacheExclusion;
    use serde::de::DeserializeOwned;
    use std::ops::Deref;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDir(PathBuf);

    impl Deref for TempDir {
        type Target = Path;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl AsRef<Path> for TempDir {
        fn as_ref(&self) -> &Path {
            &self.0
        }
    }

    impl From<&TempDir> for PathBuf {
        fn from(value: &TempDir) -> Self {
            value.0.clone()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn key_for(
        operation: CacheOperation,
        package_id: &str,
        request: &Value,
        backend: &BackendId,
        digest: u8,
        target: Option<&Value>,
    ) -> CacheKey {
        let request_bytes = serde_json::to_vec(request).expect("request encodes");
        let target_bytes = target.map(|value| serde_json::to_vec(value).expect("target encodes"));
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: ManifestDigest::from_digest([digest; 32]),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("test manifest is admitted");
        let request = CacheRequest::new(
            operation,
            package_id,
            &request_bytes,
            target_bytes.as_deref(),
            &registry,
            backend,
        )
        .expect("test request is valid");
        CacheKey::derive(&request, 16_384).expect("key inputs are valid")
    }

    fn key(digest: u8) -> CacheKey {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let request = serde_json::json!({
            "items": [{"occurrence_key": "item-1", "kind": "integer"}],
            "work_limit": 20,
        });
        key_for(
            CacheOperation::Prove,
            &format!("package-{digest}"),
            &request,
            &backend,
            digest,
            None,
        )
    }

    fn admitted_request<'a>(
        registry: &'a quire_driver::AdmittedRegistry,
        backend: &BackendId,
        request: &'a [u8],
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

    fn temporary_directory() -> TempDir {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "quire-cache-test-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary test directory is created");
        TempDir(path)
    }

    fn read_typed<T: DeserializeOwned>(
        store: &DirectoryCache,
        key: &CacheKey,
    ) -> Result<Option<T>, CacheError> {
        let Some(value) = store.read_value(key)? else {
            return Ok(None);
        };
        Ok(serde_json::from_value(value).ok())
    }

    fn put<T: serde::Serialize>(
        store: &DirectoryCache,
        key: &CacheKey,
        value: &T,
    ) -> Result<CacheWriteOutcome, CacheError> {
        store.insert_value(key, serde_json::to_value(value)?)
    }

    /// Trace: FR-012-AC-6; invalid typed entries are misses for the requested reader.
    #[test]
    fn directory_cache_uses_the_driver_request_and_typed_reader() {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let digest = ManifestDigest::from_digest([4; 32]);
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: digest,
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("descriptor is admitted");
        let request_value = serde_json::json!({"items": [], "budget": 20});
        let request_bytes = serde_json::to_vec(&request_value).expect("request encodes");
        let cache_request = admitted_request(&registry, &backend, &request_bytes);
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 4096, 8192);
        let codec = CanonicalJsonCodec;
        assert_eq!(
            Cache::lookup::<Value>(&store, &cache_request, &codec)
                .expect("initial lookup succeeds"),
            CacheLookup::Miss
        );
        let value = serde_json::json!({"terminal": "incomplete", "consumed": 20});
        assert_eq!(
            Cache::record(
                &store,
                &cache_request,
                &value,
                &codec,
                CacheEligibility::Eligible
            )
            .expect("record succeeds"),
            CacheWriteOutcome::Stored
        );
        assert_eq!(
            Cache::lookup::<Value>(&store, &cache_request, &codec).expect("typed lookup succeeds"),
            CacheLookup::Hit(value)
        );
        assert_eq!(
            Cache::lookup::<u64>(&store, &cache_request, &codec)
                .expect("wrong typed value is a miss"),
            CacheLookup::Miss
        );
    }

    /// Trace: FR-012-AC-1, FR-012-AC-3, FR-012-AC-5.
    #[test]
    fn target_and_meaning_members_match_the_operation_contract() {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let request = serde_json::json!({"items": [], "work_limit": 20});
        let target_a = serde_json::json!({"name": "wasm32-wasi", "options": {"opt": 1}});
        let target_b = serde_json::json!({"name": "wasm32-wasi", "options": {"opt": 2}});
        let prove = key_for(
            CacheOperation::Prove,
            "package-1",
            &request,
            &backend,
            1,
            None,
        );
        let analyze = key_for(
            CacheOperation::Analyze,
            "package-1",
            &request,
            &backend,
            1,
            None,
        );
        let generated_a = key_for(
            CacheOperation::Generate,
            "package-1",
            &request,
            &backend,
            1,
            Some(&target_a),
        );
        let generated_b = key_for(
            CacheOperation::Generate,
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
        assert_eq!(prove_preimage.as_object().expect("object").len(), 4);
        assert_eq!(generated_preimage.as_object().expect("object").len(), 5);
        let expected = serde_json::json!({
            "operation": "prove",
            "package_id": "package-1",
            "request": request,
            "backend": "agent-ix/provider/test",
        });
        assert_eq!(prove_preimage, expected);
        assert_ne!(
            prove,
            key_for(
                CacheOperation::Prove,
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
                CacheOperation::Prove,
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
                CacheOperation::Prove,
                "package-1",
                &request,
                &other_backend,
                1,
                None,
            )
        );
    }

    /// Trace: FR-012-AC-4; excluded execution metadata cannot perturb content identity.
    #[test]
    fn excluded_path_time_deadline_pin_revision_version_and_backend_labels_do_not_change_key() {
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("id is valid");
        let baseline = serde_json::json!({"items": [{"kind": "integer", "value": 7}]});
        let expected = key_for(
            CacheOperation::Prove,
            "package-1",
            &baseline,
            &backend,
            5,
            None,
        );
        let excluded = serde_json::json!({
            "items": [{"kind": "integer", "value": 7}],
            "path": "/tmp/b", "deadline": 4, "pin": "v1", "revision": "abc",
            "application_version": "1.0", "wall_time": 10, "execution_backend": "local",
        });
        assert_eq!(
            expected,
            key_for(
                CacheOperation::Prove,
                "package-1",
                &excluded,
                &backend,
                5,
                None
            )
        );
        let nested_meaning = serde_json::json!({
            "items": [{"kind": "integer", "value": 7, "path": "meaningful-symbol"}]
        });
        assert_ne!(
            expected,
            key_for(
                CacheOperation::Prove,
                "package-1",
                &nested_meaning,
                &backend,
                5,
                None
            ),
            "nested request fields remain meaningful when named path"
        );
    }

    /// Trace: FR-012-AC-6, FR-012-AC-7, FR-012-AC-8; includes a deterministic budget-incomplete value.
    #[test]
    fn miss_store_typed_hit_and_never_store() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 2048, 4096);
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("backend is valid");
        let manifest = ManifestDigest::from_digest([1; 32]);
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: manifest,
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("manifest is admitted");
        let request_bytes = br#"{"items":[],"work_limit":20}"#;
        let cache_request = admitted_request(&registry, &backend, request_bytes);
        let codec = CanonicalJsonCodec;
        assert_eq!(
            Cache::lookup::<Value>(&store, &cache_request, &codec).expect("cache read succeeds"),
            CacheLookup::Miss
        );
        let result = serde_json::json!({
            "terminal": "incomplete",
            "kind": "work_limit",
            "configured": 20,
            "consumed": 20,
        });
        assert_eq!(
            Cache::record(
                &store,
                &cache_request,
                &result,
                &codec,
                CacheEligibility::Eligible
            )
            .expect("cache write succeeds"),
            CacheWriteOutcome::Stored
        );
        assert_eq!(
            Cache::lookup::<Value>(&store, &cache_request, &codec).expect("cache read succeeds"),
            CacheLookup::Hit(result)
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
            let excluded_request_bytes = format!(r#"{{"items":[],"work_limit":{}}}"#, 30 + index);
            let excluded_registry =
                quire_driver::build_registry(&[quire_driver::ProviderManifest {
                    identity: backend.as_str().to_owned(),
                    manifest_digest: ManifestDigest::from_digest([2; 32]),
                    advertises: vec![(
                        Some("value-validity".to_owned()),
                        Some("bounded".to_owned()),
                    )],
                    domains: Some(vec!["integer".to_owned()]),
                    bounds: None,
                    wire: None,
                }])
                .expect("excluded descriptor is admitted");
            let excluded_request = admitted_request(
                &excluded_registry,
                &backend,
                excluded_request_bytes.as_bytes(),
            );
            assert_eq!(
                Cache::record(
                    &store,
                    &excluded_request,
                    &serde_json::json!({"terminal": "proved"}),
                    &codec,
                    CacheEligibility::NeverStore(reason)
                )
                .expect("excluded result is not a write error"),
                CacheWriteOutcome::NeverStore
            );
            assert_eq!(
                Cache::lookup::<Value>(&store, &excluded_request, &codec)
                    .expect("cache read succeeds"),
                CacheLookup::Miss
            );
        }
    }

    /// Trace: FR-012-AC-9; a JCS integer refusal becomes NotStored.
    #[test]
    fn result_above_jcs_integer_range_is_not_stored() {
        let directory = temporary_directory();
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("id is valid");
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: ManifestDigest::from_digest([27; 32]),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("descriptor is admitted");
        let request = admitted_request(&registry, &backend, br#"{"items":[]}"#);
        let store = DirectoryCache::new(&directory, 4096, 8192);
        let outcome = Cache::record(
            &store,
            &request,
            &9_007_199_254_740_993_u64,
            &CanonicalJsonCodec,
            CacheEligibility::Eligible,
        )
        .expect("unencodable cache result does not fail the provider call");
        assert_eq!(outcome, CacheWriteOutcome::NotStored);
        assert_eq!(
            fs::read_dir(&directory).expect("cache dir exists").count(),
            0
        );
    }

    /// Trace: FR-012-AC-6; typed filesystem errors remain I/O errors at the cache boundary.
    #[test]
    fn cache_filesystem_io_fault_retains_its_io_type() {
        let parent = temporary_directory();
        let file = parent.join("not-a-directory");
        fs::write(&file, b"file").expect("file occupies the intended cache path");
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("id is valid");
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: ManifestDigest::from_digest([28; 32]),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("descriptor is admitted");
        let request = admitted_request(&registry, &backend, br#"{"items":[]}"#);
        let store = DirectoryCache::new(&file, 4096, 8192);
        let error = Cache::lookup::<Value>(&store, &request, &CanonicalJsonCodec)
            .expect_err("non-directory path fails with I/O");
        assert!(matches!(&error, CacheFault::Io(_)));
        assert!(std::error::Error::source(&error).is_some());
    }

    /// Trace: FR-012-AC-6; typed values rejected by the requested reader are misses.
    #[test]
    fn invalid_typed_value_and_changed_preimage_are_misses() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 2048, 4096);
        let cache_key = key(1);
        put(
            &store,
            &cache_key,
            &serde_json::json!({"count": "not a number"}),
        )
        .expect("cache write succeeds");
        assert_eq!(
            read_typed::<u64>(&store, &cache_key).expect("cache read succeeds"),
            None
        );
        let path = store.entry_path(&cache_key);
        fs::write(path, b"{\"preimage\":\"{}\",\"value\":3}")
            .expect("corrupt test entry is writable");
        assert_eq!(
            read_typed::<u64>(&store, &cache_key).expect("corrupt entry is a miss"),
            None
        );
    }

    /// Trace: FR-012-AC-6; malformed, noncanonical, misplaced, oversized and mismatched entries are misses.
    #[test]
    fn malformed_and_mismatched_entries_are_misses() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 512, 4096);
        fs::create_dir_all(&directory).expect("test directory is created");
        let requested = key(20);
        let requested_path = store.entry_path(&requested);

        fs::write(&requested_path, b"not json").expect("malformed entry is written");
        assert_eq!(
            read_typed::<Value>(&store, &requested).expect("read succeeds"),
            None
        );

        let preimage = std::str::from_utf8(requested.preimage()).expect("preimage is UTF-8");
        let noncanonical = format!("{{ \"preimage\":{preimage:?}, \"value\":1 }}");
        fs::write(&requested_path, noncanonical).expect("noncanonical entry is written");
        assert_eq!(
            read_typed::<Value>(&store, &requested).expect("read succeeds"),
            None
        );

        let other = key(21);
        let other_path = store.entry_path(&other);
        let valid_for_requested = serde_json::json!({"preimage": preimage, "value": 1});
        fs::write(
            &other_path,
            to_vec(&valid_for_requested, Limits::new(512)).expect("entry encodes"),
        )
        .expect("entry for another preimage is written");
        assert_eq!(
            read_typed::<Value>(&store, &other).expect("read succeeds"),
            None,
            "an entry is matched by its stored preimage, never by its name"
        );
        fs::remove_file(&other_path).expect("foreign entry is removed");

        fs::write(&requested_path, vec![b'x'; 513]).expect("oversized entry is written");
        assert_eq!(
            read_typed::<Value>(&store, &requested).expect("read succeeds"),
            None
        );
    }

    /// Trace: FR-012-AC-6; a valid canonical entry above the configured limit is a miss.
    #[test]
    fn valid_oversized_entry_is_a_miss() {
        let directory = temporary_directory();
        let cache_key = key(22);
        let preimage = std::str::from_utf8(cache_key.preimage()).expect("preimage is UTF-8");
        let bytes = to_vec(
            &serde_json::json!({"preimage": preimage, "value": "x".repeat(1024)}),
            Limits::new(u64::MAX),
        )
        .expect("valid entry encodes");
        fs::write(directory.join("entry-0.json"), bytes).expect("oversized valid entry is written");
        let store = DirectoryCache::new(&directory, 128, 4096);
        assert_eq!(
            read_typed::<Value>(&store, &cache_key).expect("read succeeds"),
            None
        );
    }

    /// Trace: FR-012-AC-1, FR-012-AC-9; an oversized key preimage misses and cannot be stored.
    #[test]
    fn oversized_preimage_misses_and_record_is_not_stored() {
        let directory = temporary_directory();
        let backend = BackendId::from_wire("agent-ix/provider/test").expect("id is valid");
        let registry = quire_driver::build_registry(&[quire_driver::ProviderManifest {
            identity: backend.as_str().to_owned(),
            manifest_digest: ManifestDigest::from_digest([23; 32]),
            advertises: vec![(
                Some("value-validity".to_owned()),
                Some("bounded".to_owned()),
            )],
            domains: Some(vec!["integer".to_owned()]),
            bounds: None,
            wire: None,
        }])
        .expect("descriptor is admitted");
        let request_bytes = serde_json::to_vec(&serde_json::json!({"meaning": "x".repeat(1024)}))
            .expect("request encodes");
        let request = admitted_request(&registry, &backend, &request_bytes);
        let store = DirectoryCache::new(&directory, 128, 4096);
        let codec = CanonicalJsonCodec;
        assert_eq!(
            Cache::lookup::<Value>(&store, &request, &codec).expect("large key misses"),
            CacheLookup::Miss
        );
        assert_eq!(
            Cache::record(
                &store,
                &request,
                &serde_json::json!({"terminal":"proved"}),
                &codec,
                CacheEligibility::Eligible
            )
            .expect("large key does not fail operation"),
            CacheWriteOutcome::NotStored
        );
        assert_eq!(
            fs::read_dir(&directory)
                .map(|entries| entries.count())
                .unwrap_or(0),
            0
        );
    }

    /// Trace: FR-012-AC-12; entries rejected by the total byte budget are not published.
    #[test]
    fn entries_over_the_store_capacity_are_not_written() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 2048, 8);
        let cache_key = key(1);
        assert_eq!(
            put(
                &store,
                &cache_key,
                &serde_json::json!({"terminal": "proved"})
            )
            .expect("oversize write is a normal miss"),
            CacheWriteOutcome::NotStored
        );
        assert!(!store.entry_path(&cache_key).exists());
    }

    /// Trace: FR-012-AC-9, FR-012-AC-12; entry-size refusal preserves the cache bound.
    #[test]
    fn entries_over_the_per_entry_limit_are_not_written() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 8, 4096);
        let cache_key = key(1);
        let result = put(
            &store,
            &cache_key,
            &serde_json::json!({"value": "too large"}),
        )
        .expect("oversize entry is a normal non-store result");
        assert_eq!(result, CacheWriteOutcome::NotStored);
        assert!(!store.entry_path(&cache_key).exists());
    }

    /// Trace: FR-012-AC-12; insertion evicts an owned entry when that is needed for the bound.
    #[test]
    fn insertion_evicts_an_owned_entry_to_stay_within_capacity() {
        let directory = temporary_directory();
        let first = key(31);
        let second = key(32);
        let store = DirectoryCache::new(&directory, 4096, 4096);
        let value = serde_json::json!({"payload": "v".repeat(120)});
        assert_eq!(
            put(&store, &first, &value).expect("first entry stores"),
            CacheWriteOutcome::Stored
        );
        let entry_size = fs::metadata(store.entry_path(&first))
            .expect("first entry exists")
            .len();
        let bounded = DirectoryCache::new(&directory, 4096, entry_size + 32);
        assert_eq!(
            put(&bounded, &second, &value).expect("second entry stores after eviction"),
            CacheWriteOutcome::Stored
        );
        let names = fs::read_dir(&directory).expect("store lists").count();
        assert_eq!(names, 1);
        assert!(
            bounded.entry_path(&second).exists(),
            "newly inserted entry remains"
        );
        assert!(
            !bounded.entry_path(&first).exists(),
            "older owned entry was evicted"
        );
    }

    /// Trace: FR-012-AC-12; foreign files count toward capacity but are never removed.
    #[test]
    fn foreign_json_and_tmp_like_files_are_counted_but_preserved() {
        let directory = temporary_directory();
        let large = DirectoryCache::new(&directory, 4096, 16_384);
        let old_key = key(51);
        let new_key = key(52);
        let value = serde_json::json!({"payload": "v".repeat(90)});
        assert_eq!(
            put(&large, &old_key, &value).expect("owned entry stores"),
            CacheWriteOutcome::Stored
        );
        let old_entry = large.entry_path(&old_key);
        let foreign_json = directory.join("000-unrelated.json");
        fs::write(&foreign_json, b"foreign json data").expect("foreign JSON exists");
        let foreign_temp = directory.join("unrelated.tmp-backup");
        fs::write(&foreign_temp, b"foreign temp data").expect("foreign temp-like file exists");
        let old_time = fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH);
        fs::OpenOptions::new()
            .write(true)
            .open(&foreign_temp)
            .expect("temp-like file opens")
            .set_times(old_time)
            .expect("old timestamp is set");
        let new_size = fs::metadata(&old_entry).expect("old entry exists").len();
        let foreign_bytes =
            fs::metadata(&foreign_json).unwrap().len() + fs::metadata(&foreign_temp).unwrap().len();
        let bounded = DirectoryCache::new(&directory, 4096, foreign_bytes + new_size);
        assert_eq!(
            put(&bounded, &new_key, &value).expect("owned entry is evicted"),
            CacheWriteOutcome::Stored
        );
        assert!(foreign_json.exists(), "unrelated JSON was not evicted");
        assert!(
            foreign_temp.exists(),
            "unrelated temp-like file was not reaped"
        );
        assert!(bounded.entry_path(&new_key).exists());
        assert!(
            !old_entry.exists(),
            "validated owned entry was selected for eviction"
        );
    }

    /// Trace: FR-012-AC-12; invalid or oversized entry-shaped files count but are not eviction candidates.
    #[test]
    fn oversized_or_corrupt_entry_shaped_file_blocks_over_capacity_write() {
        let directory = temporary_directory();
        let malformed = directory.join("entry-53.json");
        fs::write(&malformed, vec![b'x'; 32]).expect("oversized malformed entry exists");
        let store = DirectoryCache::new(&directory, 8, 16);
        assert!(!store
            .evict_for(
                &directory.join("new-entry.json"),
                &directory.join("new-temp"),
                1
            )
            .expect("capacity scan succeeds"));
        assert!(
            malformed.exists(),
            "the cache preserves an invalid entry-shaped file"
        );
    }

    /// Trace: FR-012-AC-9, FR-012-AC-12; entry exactly at both limits is accepted.
    #[test]
    fn exactly_at_configured_limits_is_stored() {
        let directory = temporary_directory();
        let cache_key = key(12);
        let value = serde_json::json!({"value": "bounded"});
        let preimage = std::str::from_utf8(cache_key.preimage()).expect("key preimage is UTF-8");
        let entry = serde_json::json!({"preimage": preimage, "value": value.clone()});
        let encoded = to_vec(&entry, Limits::new(u64::MAX)).expect("entry encodes");
        let exact_limit = encoded.len() as u64;
        let store = DirectoryCache::new(&directory, exact_limit, exact_limit);
        assert_eq!(
            put(&store, &cache_key, &value).expect("exact boundary insertion succeeds"),
            CacheWriteOutcome::Stored
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
    }

    /// Trace: FR-012-AC-10; a vanished temporary file at the rename site is a normal NotStored.
    #[test]
    fn vanished_temporary_rename_returns_not_stored() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 4096, 8192);
        let cache_key = key(26);
        let final_path = store.entry_path(&cache_key);
        let outcome = store
            .insert_value_before_publish(
                &cache_key,
                serde_json::json!({"terminal":"proved"}),
                |temporary, _| fs::remove_file(temporary).expect("simulated concurrent removal"),
            )
            .expect("a vanished temp is not an operational fault");
        assert_eq!(outcome, CacheWriteOutcome::NotStored);
        assert!(!final_path.exists());
    }

    /// Trace: FR-012-AC-10; bytes stay under the temporary name until atomic rename.
    #[test]
    fn final_name_is_absent_until_temporary_file_is_ready_to_rename() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 4096, 8192);
        let cache_key = key(25);
        let final_path = store.entry_path(&cache_key);
        assert_eq!(
            store
                .insert_value_before_publish(
                    &cache_key,
                    serde_json::json!({"terminal":"proved"}),
                    |temporary, final_name| {
                        assert_eq!(final_name, final_path);
                        assert!(
                            temporary.exists(),
                            "complete temporary entry exists before publish"
                        );
                        assert!(!final_name.exists(), "final name is not written directly");
                    },
                )
                .expect("atomic insertion succeeds"),
            CacheWriteOutcome::Stored
        );
        assert!(final_path.exists(), "rename publishes the entry");
    }

    /// Trace: FR-012-AC-10, FR-012-AC-12; temporary bytes count, abandoned files are cleaned, symlinks are not followed.
    #[test]
    fn temporary_capacity_and_abandoned_cleanup_are_enforced() {
        let directory = temporary_directory();
        fs::create_dir_all(&directory).expect("test directory is created");
        let abandoned = directory.join(".entry-3.json.tmp-999999999-1");
        fs::write(&abandoned, vec![b'x'; 150]).expect("stale temporary file is created");
        let old = fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH);
        fs::OpenOptions::new()
            .write(true)
            .open(&abandoned)
            .expect("abandoned file opens")
            .set_times(old)
            .expect("abandoned timestamp is set");
        let active = directory.join(format!(".entry-4.json.tmp-{}-1", std::process::id()));
        fs::write(&active, vec![b'y'; 650]).expect("live temporary file is created");
        let outside_directory = temporary_directory();
        let outside = outside_directory.join("outside");
        fs::write(&outside, b"outside").expect("outside target is created");
        let link = directory.join("entry-0.json");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).expect("symlink is created");
        #[cfg(unix)]
        assert!(
            open_regular_file_no_follow(&link).is_err(),
            "open refuses the symlink itself"
        );
        let store = DirectoryCache::new(&directory, 2048, 700);
        #[cfg(unix)]
        assert_eq!(
            read_typed::<Value>(&store, &key(9)).expect("symlink is a miss"),
            None
        );
        let result = put(&store, &key(1), &serde_json::json!({"value": "fits alone"}))
            .expect("capacity refusal is not an I/O error");
        assert_eq!(result, CacheWriteOutcome::NotStored);
        assert!(!abandoned.exists(), "dead-owner temporary file is removed");
        assert!(
            active.exists(),
            "active temporary file is counted, not removed"
        );
        assert_eq!(
            fs::read(&outside).expect("outside file remains"),
            b"outside"
        );
    }

    /// Trace: FR-012-AC-11; concurrent same-key writes publish complete entries.
    #[test]
    fn concurrent_same_key_writers_leave_a_valid_entry() {
        let directory = temporary_directory();
        let store = std::sync::Arc::new(DirectoryCache::new(&directory, 4096, 65_536));
        let cache_key = key(1);
        let mut writers = Vec::new();
        let directory_path = directory.0.clone();
        for value in 0..6_u64 {
            let directory = directory_path.clone();
            let cache_key = cache_key.clone();
            writers.push(std::thread::spawn(move || {
                DirectoryCache::new(directory, 4096, 65_536)
                    .insert_value(&cache_key, serde_json::json!({"value": value}))
                    .expect("writer succeeds")
            }));
        }
        for writer in writers {
            assert_eq!(
                writer.join().expect("writer thread completes"),
                CacheWriteOutcome::Stored
            );
        }
        let hit = read_typed::<serde_json::Value>(&store, &cache_key)
            .expect("final entry is readable")
            .expect("final entry exists");
        assert!(hit.get("value").and_then(Value::as_u64).is_some());
    }

    /// Trace: FR-012-AC-11; a candidate removed by another process is tolerated.
    #[test]
    fn missing_eviction_candidate_is_tolerated() {
        let directory = temporary_directory();
        let store = DirectoryCache::new(&directory, 4096, 4096);
        let existing_key = key(40);
        assert_eq!(
            put(&store, &existing_key, &serde_json::json!({"v": "x"})).expect("owned entry stores"),
            CacheWriteOutcome::Stored
        );
        let entry = store.entry_path(&existing_key);
        let size = fs::metadata(&entry).expect("entry exists").len();
        let bounded = DirectoryCache::new(&directory, 4096, size + 8);
        let target = directory.join("new-target.json");
        assert!(bounded
            .evict_for_with_hook(&target, Path::new("own-temp"), 16, |candidate| {
                if candidate == entry {
                    fs::remove_file(candidate).expect("other process removes candidate");
                }
            })
            .expect("eviction ignores a concurrent NotFound"));
    }
}
