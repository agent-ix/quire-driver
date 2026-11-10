// SPDX-License-Identifier: AGPL-3.0-or-later
//! Authored request fixtures for process tests; each owns its temporary request file.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_REQUEST: AtomicUsize = AtomicUsize::new(0);

/// A temporary request kept alive for the whole child invocation.
pub struct RequestFile(PathBuf);

impl RequestFile {
    /// The UTF-8 operand passed to the child binary.
    pub fn as_str(&self) -> &str {
        self.0.to_str().expect("UTF-8 request path")
    }
}

impl Drop for RequestFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Author a closed request with explicit native and formal identities.
pub fn request(source: &str, call: Option<(&str, serde_json::Value)>) -> RequestFile {
    let source_path = std::fs::canonicalize(source).expect("existing fixture source");
    let mut request = serde_json::json!({
        "models": [],
        "program": {"source": {"authority": "quire", "identity": source, "file": source_path, "document":"CliProcess", "formal_revision":1}},
    });
    let format = if let Some((function, arguments)) = call {
        request["call"] = serde_json::json!({"function": function, "arguments": arguments});
        "native-run/1"
    } else {
        "native-compile/1"
    };
    let id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("quire-request-{}-{id}.json", std::process::id()));
    std::fs::write(
        &path,
        serde_json::to_vec(&serde_json::json!({"format": format, "request": request}))
            .expect("encode authored request"),
    )
    .expect("write authored request");
    RequestFile(path)
}
