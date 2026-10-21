// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict caller-selected process-provider configuration for FR-006.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The complete provider configuration file named by `--provider-config`.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProviderConfig {
    /// Process providers to admit before routing.
    pub providers: Vec<ProcessProviderConfig>,
}

/// One caller-selected process provider and its host resource limits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessProviderConfig {
    /// Executable path to start.
    pub executable: PathBuf,
    /// Arguments passed verbatim to the executable.
    pub argv: Vec<String>,
    /// Caller-owned FR-331 request deadline in milliseconds.
    #[serde(deserialize_with = "deserialize_deadline")]
    pub deadline_ms: u64,
    /// Required framing and optional process resource limits.
    pub limits: ProcessProviderLimitsConfig,
}

/// Strict JSON representation of `ProcessProviderLimits`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessProviderLimitsConfig {
    /// Maximum encoded frame size in bytes.
    pub max_frame_bytes: u64,
    /// Maximum encoded session size in bytes.
    pub max_total_bytes: u64,
    /// Maximum captured stderr size in bytes.
    pub max_stderr_bytes: u64,
    /// Cancellation grace period in milliseconds.
    pub cancellation_grace_ms: u64,
    /// Optional process-tree resident-memory budget in bytes.
    #[serde(default, deserialize_with = "deserialize_optional_u64")]
    pub max_process_memory_bytes: Option<u64>,
    /// Optional process-tree CPU-time budget in milliseconds.
    #[serde(default, deserialize_with = "deserialize_optional_u64")]
    pub max_process_cpu_time_ms: Option<u64>,
}

fn deserialize_optional_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    u64::deserialize(deserializer).map(Some)
}

fn deserialize_deadline<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;
    if (1..=9_007_199_254_740_991).contains(&value) {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(
            "deadline_ms must be in 1..=9007199254740991",
        ))
    }
}

/// Why an explicitly selected provider configuration could not be loaded.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ProviderConfigError {
    /// The named file could not be read as UTF-8 text.
    #[error("could not read `{path}` as UTF-8 JSON: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// The named file did not satisfy the strict provider configuration shape.
    #[error("`{path}` does not match the provider configuration schema: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Read only the caller-named strict JSON file.
pub(crate) fn load(path: &Path) -> Result<ProviderConfig, ProviderConfigError> {
    let display_path = path.display().to_string();
    let contents = fs::read_to_string(path).map_err(|source| ProviderConfigError::Read {
        path: display_path.clone(),
        source,
    })?;
    serde_json::from_str(&contents).map_err(|source| ProviderConfigError::Json {
        path: display_path,
        source,
    })
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "invalid fixture data should fail the test"
)]
mod tests {
    use super::*;

    const VALID: &str = r#"{
        "providers": [{
            "executable": "/tmp/provider",
            "argv": [],
            "deadline_ms": 250,
            "limits": {
                "max_frame_bytes": 1024,
                "max_total_bytes": 4096,
                "max_stderr_bytes": 256,
                "cancellation_grace_ms": 500,
                "max_process_memory_bytes": 1048576,
                "max_process_cpu_time_ms": 2000
            }
        }]
    }"#;

    /// Trace: FR-006-AC-24, FR-014-AC-6; parse the exact required and optional config fields.
    #[test]
    fn parses_provider_config_with_explicit_empty_arguments_and_optional_budgets() {
        let config: ProviderConfig = serde_json::from_str(VALID).expect("valid provider config");
        assert_eq!(config.providers.len(), 1);
        assert_eq!(
            config.providers[0].executable,
            PathBuf::from("/tmp/provider")
        );
        assert!(config.providers[0].argv.is_empty());
        assert_eq!(config.providers[0].deadline_ms, 250);
        assert_eq!(config.providers[0].limits.max_frame_bytes, 1024);
        assert_eq!(
            config.providers[0].limits.max_process_memory_bytes,
            Some(1_048_576)
        );
        assert_eq!(
            config.providers[0].limits.max_process_cpu_time_ms,
            Some(2_000)
        );
    }

    /// Trace: FR-006-AC-24; omitted optional budgets are absent, while null is invalid.
    #[test]
    fn omitted_optional_budgets_are_none_and_null_is_rejected() {
        let value = VALID.replace(
            "\"max_process_cpu_time_ms\": 2000",
            "\"max_process_cpu_time_ms\": null",
        );
        assert!(serde_json::from_str::<ProviderConfig>(&value).is_err());

        let omitted = VALID
            .replace(
                "                \"max_process_memory_bytes\": 1048576,\n",
                "",
            )
            .replace(",\n                \"max_process_cpu_time_ms\": 2000", "");
        let config: ProviderConfig = serde_json::from_str(&omitted).expect("omitted budgets");
        assert_eq!(config.providers[0].limits.max_process_memory_bytes, None);
        assert_eq!(config.providers[0].limits.max_process_cpu_time_ms, None);
    }

    /// Trace: FR-006-AC-24; missing or wrongly typed required fields fail closed.
    #[test]
    fn rejects_missing_or_wrongly_typed_required_fields() {
        for malformed in [
            "{}".to_owned(),
            VALID.replace("\"providers\": [", "\"providers\": null, \"unused\": ["),
            VALID.replace("\"argv\": [],", ""),
            VALID.replace("\"executable\": \"/tmp/provider\",", ""),
            VALID.replace("\"limits\": {", "\"limits\": null, \"ignored\": {"),
            VALID.replace("\"max_total_bytes\": 4096,", ""),
            VALID.replace("\"max_frame_bytes\": 1024", "\"max_frame_bytes\": \"1024\""),
            VALID.replace("\"argv\": []", "\"argv\": \"\""),
            VALID.replace(
                "\"max_process_memory_bytes\": 1048576",
                "\"max_process_memory_bytes\": null",
            ),
            VALID.replace(
                "\"max_process_cpu_time_ms\": 2000",
                "\"max_process_cpu_time_ms\": \"2000\"",
            ),
        ] {
            assert!(serde_json::from_str::<ProviderConfig>(&malformed).is_err());
        }
    }

    /// Trace: FR-006-AC-24; accept both positive safe-integer boundaries only.
    #[test]
    fn provider_deadline_accepts_safe_integer_boundaries_and_rejects_outside_values() {
        for deadline in [1, 9_007_199_254_740_991] {
            let config: ProviderConfig = serde_json::from_str(&VALID.replace(
                "\"deadline_ms\": 250",
                &format!("\"deadline_ms\": {deadline}"),
            ))
            .expect("boundary deadline is valid");
            assert_eq!(config.providers[0].deadline_ms, deadline);
        }
        for deadline in ["0", "-1", "1.5", "null", "\"1\"", "9007199254740992"] {
            let invalid = VALID.replace(
                "\"deadline_ms\": 250",
                &format!("\"deadline_ms\": {deadline}"),
            );
            assert!(
                serde_json::from_str::<ProviderConfig>(&invalid).is_err(),
                "accepted deadline {deadline}"
            );
        }
        let missing = VALID.replace("\"deadline_ms\": 250,", "");
        assert!(serde_json::from_str::<ProviderConfig>(&missing).is_err());
    }

    /// Trace: FR-006-AC-24; unknown fields and identity overrides are rejected at all levels.
    #[test]
    fn rejects_unknown_fields_at_every_object_level() {
        for malformed in [
            VALID.replace("\"providers\":", "\"identity\": \"caller\", \"providers\":"),
            VALID.replace(
                "\"executable\":",
                "\"identity\": \"caller\", \"executable\":",
            ),
            VALID.replace("\"limits\": {", "\"limits\": { \"other\": 1,"),
        ] {
            assert!(serde_json::from_str::<ProviderConfig>(&malformed).is_err());
        }
    }

    /// Trace: FR-006-AC-24; load only the explicitly named file and fail on absent or malformed data.
    #[test]
    fn named_file_loader_reports_missing_and_malformed_files() {
        let missing = load(Path::new("this-provider-config-file-does-not-exist.json"))
            .expect_err("a missing named file is refused");
        assert!(missing.to_string().contains("could not read"));

        let path = std::env::temp_dir().join(format!(
            "quire-provider-config-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock after Unix epoch")
                .as_nanos()
        ));
        fs::write(&path, "{ malformed").expect("write malformed JSON");
        let malformed = load(&path).expect_err("malformed JSON is refused");
        let _ = fs::remove_file(path);
        assert!(malformed.to_string().contains("schema"));
    }
}
