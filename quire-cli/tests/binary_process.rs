// SPDX-License-Identifier: AGPL-3.0-or-later
//! Integration checks for CLI run options and QSL results.

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io::{Read, Write};
use std::process::Command;
#[cfg(unix)]
use std::process::{Child, ExitStatus, Stdio};
#[cfg(unix)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(unix)]
use std::time::{Duration, Instant};

use qsl_foundation::diagnostic::Code;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, default_accounting, parse, select, Call, DependencyInput, LockEvidence, ParseRequest,
    SpineLimits, DEFAULT_WORK_UNITS,
};
use qsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};
use qsl_semantics::value::{CatalogRole, DefinitionLock};
use quire_driver::{execute, ExecutionEngine, ExecutionRequest, ObjectEnvironment};
use quire_exact::Cancel;
use quire_semantic_value::catalog::Category;
use quire_semantic_value::supplied_stop::SuppliedLimits;

#[cfg(unix)]
#[path = "../../tests/common/units.rs"]
mod shared_units;

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");

#[cfg(unix)]
static TEST_DIRECTORY_ID: AtomicUsize = AtomicUsize::new(0);

#[cfg(unix)]
struct TestDirectory(std::path::PathBuf);

#[cfg(unix)]
impl TestDirectory {
    fn new(label: &str) -> Self {
        let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("quire-cli-{label}-{}-{id}", std::process::id()));
        std::fs::create_dir(&path).expect("create isolated test directory");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

#[cfg(unix)]
impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
struct ChildGuard(Option<Child>);

#[cfg(unix)]
impl ChildGuard {
    fn new(child: Child) -> Self {
        Self(Some(child))
    }

    fn child(&mut self) -> &mut Child {
        self.0.as_mut().expect("child remains owned until reaped")
    }

    fn kill_and_wait(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.0 = None;
    }
}

#[cfg(unix)]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_and_wait();
    }
}

#[cfg(unix)]
fn wait_for_child_until(child: &mut ChildGuard, deadline: Instant) -> ExitStatus {
    loop {
        if let Some(status) = child.child().try_wait().expect("poll child status") {
            return status;
        }
        if Instant::now() >= deadline {
            child.kill_and_wait();
            panic!("child did not exit before the TC-018 deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn overflow_fixture_bytes() -> Vec<u8> {
    shared_units::overflow_unit().into_bytes()
}

#[cfg(unix)]
fn assert_tc018_refused_range_observation(path: &std::path::Path, source_path: &str, bytes: &[u8]) {
    let observation: serde_json::Value = serde_json::from_slice(
        &std::fs::read(path).expect("the test-only operation adapter records its request"),
    )
    .expect("the adapter observation is JSON");
    assert_eq!(observation["case"], "refused-range");
    assert_eq!(observation["path"], source_path);
    assert_eq!(
        observation["source_identity"],
        serde_json::json!({
            "authority": "quire",
            "identity": source_path,
            "revision_namespace": "filesystem",
            "revision": "current",
        })
    );
    assert_eq!(observation["source_bytes"], serde_json::json!(bytes));
    assert!(
        observation["profile"]
            .as_str()
            .is_some_and(|profile| profile.starts_with("profile v = ")),
        "the observation retains the selected profile: {observation}"
    );
    assert!(
        observation["limits"]
            .as_str()
            .is_some_and(|limits| limits.contains("SpineLimits")),
        "the observation retains the QSL limits: {observation}"
    );
}

#[cfg(unix)]
fn expected_overflow_check_document(path: &str, bytes: &[u8]) -> Vec<u8> {
    let source = SourceIdentity::new("quire", path, "filesystem", "current");
    let limits = SpineLimits::default();
    let cancel = Cancel::new();
    let packages = std::collections::BTreeMap::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path,
            bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &packages, limits.model, &cancel).and_then(|models| {
            check(
                parsed.value(),
                models.value(),
                &DependencyInput::default(),
                &LockEvidence::default(),
                limits,
                &cancel,
            )
        })
    });
    OutcomeDocument::from_check(&result)
        .to_bytes()
        .expect("QSL serializes the expected check refusal")
}

#[cfg(unix)]
fn open_fifo_writer_until(
    path: &std::path::Path,
    child: &mut ChildGuard,
    deadline: Instant,
) -> File {
    use std::os::unix::fs::OpenOptionsExt;

    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(writer) => return writer,
            Err(error)
                if error.raw_os_error() == Some(libc::ENXIO)
                    || error.raw_os_error() == Some(libc::EINTR) =>
            {
                if let Some(status) = child.child().try_wait().expect("poll FIFO child") {
                    panic!("child exited before the source FIFO opened: {status:?}");
                }
                if Instant::now() >= deadline {
                    panic!("source FIFO writer did not open before the TC-018 deadline");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("open source FIFO writer: {error}"),
        }
    }
}

#[cfg(unix)]
fn read_child_stderr(child: &mut ChildGuard) -> Vec<u8> {
    let mut stderr = Vec::new();
    child
        .child()
        .stderr
        .take()
        .expect("child stderr remains piped")
        .read_to_end(&mut stderr)
        .expect("read child stderr after exit");
    stderr
}

#[cfg(unix)]
fn status_with_unwritable_stderr(arguments: &[&str], reserved_exit: bool) -> ExitStatus {
    let mut command = Command::new(QUIRE);
    command.args(arguments).stdout(Stdio::null());
    if reserved_exit {
        command.env("TC018_CASE", "reserved-exit");
    }
    if let Ok(full) = std::fs::OpenOptions::new().write(true).open("/dev/full") {
        return command
            .stderr(Stdio::from(full))
            .status()
            .expect("run quire with /dev/full as stderr");
    }

    let mut child = command
        .stderr(Stdio::piped())
        .spawn()
        .expect("run quire with a pipe for stderr");
    drop(child.stderr.take());
    child.wait().expect("wait for quire after closing stderr")
}

fn expected_document_bytes(code: Code, message: &str) -> Vec<u8> {
    let document =
        OutcomeDocument::new(Operation::Execute, None, code.category()).with_diagnostics(vec![
            OutcomeDiagnostic::new(None, code.as_str(), None, message),
        ]);
    let mut bytes =
        ix_cli_kit::json::encode_canonical(&document, false).expect("encode expected QSL document");
    bytes.push('\n');
    bytes.into_bytes()
}

/// Trace: FR-004-AC-1, omitted engine selects the linked interpreter.
#[test]
fn run_without_engine_executes_with_the_interpreter() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--format", "json"])
        .output()
        .expect("run quire");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-002-AC-13; unsupported QSL text renderers refuse before lifecycle execution.
#[test]
fn unavailable_text_renderers_return_typed_unsupported_refusals() {
    let cases = [
        vec!["parse", "missing-text-renderer.qsl"],
        vec!["check", "missing-text-renderer.qsl"],
        vec!["compile", "missing-text-renderer.qsl"],
        vec!["run", "missing-text-renderer.qsl", "yes"],
    ];

    for arguments in cases {
        let verb = arguments[0];
        for suffix in [vec![], vec!["--format", "text"]] {
            let output = Command::new(QUIRE)
                .args(arguments.iter().copied().chain(suffix.iter().copied()))
                .output()
                .expect("run quire with unavailable typed text rendering");
            assert_eq!(
                output.status.code(),
                Some(i32::from(Category::Unsupported.exit_code())),
                "{verb} {suffix:?}"
            );
            assert!(output.stdout.is_empty(), "{verb} {suffix:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains("unimplemented_capability"),
                "{verb} {suffix:?}: {stderr}"
            );
            assert!(
                stderr.contains(&format!("text rendering for `{verb}`")),
                "{verb} {suffix:?}: {stderr}"
            );
            assert!(
                !stderr.contains("could not read source"),
                "{verb}: {stderr}"
            );
        }
    }
}

/// Trace: FR-002-AC-3, FR-002-AC-9; JSON is the unchanged QSL outcome encoded by the kit.
#[test]
fn machine_run_writes_the_canonical_qsl_outcome() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--format", "json"])
        .output()
        .expect("run quire in machine mode");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let packages = std::collections::BTreeMap::new();
    let dependencies = DependencyInput::default();
    let source_bytes = std::fs::read(SOURCE).expect("read source fixture");
    let request = ExecutionRequest {
        source: SourceIdentity::new("quire", SOURCE, "filesystem", "current"),
        path: SOURCE,
        bytes: &source_bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits: SpineLimits::default(),
        call: Call {
            function: "yes".to_owned(),
            arguments: Vec::new(),
            accounting: default_accounting(DEFAULT_WORK_UNITS),
            supplied: SuppliedLimits::default(),
        },
        object_environment: &ObjectEnvironment::default(),
        cancel: Cancel::new(),
        engine: ExecutionEngine::Interpreter,
    };
    let result = execute(request).expect("obtain typed QSL result");
    let expected = ix_cli_kit::json::encode_canonical(&result.document, false)
        .expect("encode typed outcome through kit");
    assert_eq!(output.stdout, format!("{expected}\n").as_bytes());
    let qsl_bytes = result
        .outcome_bytes()
        .expect("encode typed outcome through QSL");
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("parse child outcome"),
        serde_json::from_slice::<serde_json::Value>(&qsl_bytes).expect("parse QSL outcome bytes")
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-8, FR-002-AC-9; pre-QSL request refusals use QSL's catalog code.
#[test]
fn malformed_run_limit_emits_one_qsl_outcome_and_streams_diagnostic() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "s1.input_bytes=invalid",
            "--format",
            "json",
        ])
        .output()
        .expect("run quire with malformed limit");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("s1.input_bytes=invalid"));
    assert!(stderr.ends_with('\n'));
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-002-AC-3, FR-002-AC-8, FR-002-AC-9; source I/O errors retain QSL's native code.
#[test]
fn missing_source_emits_qsl_io_error_document_and_diagnostic() {
    let output = Command::new(QUIRE)
        .args(["run", "missing-source.qsl", "yes", "--format", "json"])
        .output()
        .expect("run quire with a missing source");
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("could not read source missing-source.qsl:"));
    assert!(stderr.ends_with('\n'));
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::IoError, stderr.trim_end())
    );
}

fn run_with_provider_config(path: &std::path::Path) -> std::process::Output {
    Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--provider-config"])
        .arg(path)
        .args(["--format", "json"])
        .output()
        .expect("run quire with provider config")
}

fn assert_provider_config_refusal(path: &std::path::Path, expected: &str) {
    let output = run_with_provider_config(path);
    assert_eq!(output.status.code(), Some(20));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid process-provider configuration"));
    assert!(stderr.contains(expected), "{stderr}");
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-006-AC-24, FR-014-AC-6; strict file errors are refused before any provider starts.
#[test]
fn provider_config_file_and_schema_errors_emit_one_invalid_request_document() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "quire-cli-provider-config-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated provider config directory");
    let missing = directory.join("missing.json");
    assert_provider_config_refusal(&missing, "could not read");

    let cases = [
        (
            "malformed.json",
            "{ malformed",
            "provider configuration schema",
        ),
        (
            "missing-field.json",
            r#"{"providers":[{"argv":[],"deadline_ms":100,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "wrong-type.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":100,"limits":{"max_frame_bytes":"1","max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "missing-deadline.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "zero-deadline.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":0,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "negative-deadline.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":-1,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "fractional-deadline.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":1.5,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "null-deadline.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":null,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "unsafe-deadline.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":9007199254740992,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1}}]}"#,
            "provider configuration schema",
        ),
        (
            "null-budget.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":100,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1,"max_process_memory_bytes":null}}]}"#,
            "provider configuration schema",
        ),
        (
            "unknown-top-level.json",
            r#"{"providers":[],"unexpected":true}"#,
            "provider configuration schema",
        ),
        (
            "unknown-provider.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":100,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1},"unexpected":true}]}"#,
            "provider configuration schema",
        ),
        (
            "unknown-limits.json",
            r#"{"providers":[{"executable":"/must-not-start","argv":[],"deadline_ms":100,"limits":{"max_frame_bytes":1,"max_total_bytes":1,"max_stderr_bytes":1,"cancellation_grace_ms":1,"unexpected":true}}]}"#,
            "provider configuration schema",
        ),
    ];
    for (name, contents, expected) in cases {
        let path = directory.join(name);
        std::fs::write(&path, contents).expect("write provider config fixture");
        assert_provider_config_refusal(&path, expected);
    }
    std::fs::remove_dir_all(directory).expect("remove isolated provider config directory");
}

/// Trace: FR-006-AC-24, FR-014-AC-6; an explicit empty config is equivalent to no process providers.
#[test]
fn empty_provider_config_executes_without_starting_a_provider() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "quire-cli-empty-provider-config-{}-{unique}.json",
        std::process::id()
    ));
    std::fs::write(&path, r#"{"providers":[]}"#).expect("write empty provider config");
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--provider-config"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run quire with empty provider config");
    let _ = std::fs::remove_file(path);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(serde_json::from_slice::<serde_json::Value>(&output.stdout).is_ok());
}

/// Trace: FR-006-AC-26; duplicate admitted identities refuse before requests and close both sessions.
#[test]
fn duplicate_provider_identities_refuse_before_negotiation_and_close_sessions() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "quire-cli-configured-duplicate-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated duplicate test directory");
    let source_path = directory.join("duplicate.native");
    let config_path = directory.join("providers.json");
    let request_markers = [directory.join("request-one"), directory.join("request-two")];
    let shutdown_markers = [
        directory.join("shutdown-one"),
        directory.join("shutdown-two"),
    ];

    let lock = DefinitionLock::pinned();
    let root = lock.entry(CatalogRole::Root);
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
         function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure {{ x = y }}\n\
         function negate using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n",
        root.identity
    );
    std::fs::write(&source_path, source).expect("write source fixture");

    let executable = std::env::split_paths(&std::env::var_os("PATH").expect("PATH is set"))
        .map(|directory| directory.join("python3"))
        .find(|candidate| candidate.is_file())
        .expect("duplicate provider fixture requires Python 3");
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/configured-fr331-provider.py"
    );
    let providers = (0..2)
        .map(|index| {
            serde_json::json!({
                "executable": executable.display().to_string(),
                "argv": [
                    fixture,
                    "agent-ix/provider/configured-duplicate",
                    request_markers[index].display().to_string(),
                    "value-validity",
                    "normal",
                    shutdown_markers[index].display().to_string(),
                ],
                "deadline_ms": 4096,
                "limits": {
                    "max_frame_bytes": 262144,
                    "max_total_bytes": 1048576,
                    "max_stderr_bytes": 4096,
                    "cancellation_grace_ms": 500,
                }
            })
        })
        .collect::<Vec<_>>();
    std::fs::write(
        &config_path,
        serde_json::to_vec(&serde_json::json!({ "providers": providers }))
            .expect("serialize provider config"),
    )
    .expect("write provider config");

    let output = Command::new(QUIRE)
        .args([
            "run",
            source_path.to_str().expect("UTF-8 source path"),
            "eq",
            "x=2",
            "y=3",
        ])
        .arg("--provider-config")
        .arg(&config_path)
        .args(["--format", "json"])
        .output()
        .expect("run production quire binary with duplicate identities");
    let message = "configured process providers repeat admitted identity `agent-ix/provider/configured-duplicate`";
    assert_eq!(output.status.code(), Some(20));
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, message)
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), message);
    assert!(request_markers.iter().all(|marker| !marker.exists()));
    assert!(shutdown_markers.iter().all(|marker| marker.is_file()));
    std::fs::remove_dir_all(directory).expect("remove duplicate test directory");
}

/// Trace: FR-006-AC-4; the production binary cancels and reaps a selected provider before re-raising SIGTERM.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn configured_process_sigterm_cleans_active_session_before_reraise() {
    use std::os::unix::process::ExitStatusExt;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after the Unix epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "quire-cli-configured-signal-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("create isolated signal test directory");
    let request_marker = directory.join("request.json");
    let ready_marker = directory.join("descendant-ready");
    let release_marker = directory.join("release-descendant");
    let descendant_marker = directory.join("descendant-leaked");
    let config_path = directory.join("providers.json");
    let source_path = directory.join("signal.native");

    let lock = DefinitionLock::pinned();
    let root = lock.entry(CatalogRole::Root);
    let source = format!(
        "language \"ix:native\" edition \"1-draft\";\nprofile v = \"{}\";\n\
         function eq using v(x: Int[0, 9], y: Int[0, 9]): Boolean pure {{ x = y }}\n\
         function negate using v(z: Int[0, 9]): Int[-9, 0] pure {{ -z }}\n",
        root.identity
    );
    std::fs::write(&source_path, source).expect("write source fixture");

    let executable = std::env::split_paths(&std::env::var_os("PATH").expect("PATH is set"))
        .map(|directory| directory.join("python3"))
        .find(|candidate| candidate.is_file())
        .expect("the signal fixture requires Python 3");
    let fixture = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/configured-fr331-provider.py"
    );
    let config = serde_json::json!({
        "providers": [{
            "executable": executable,
            "argv": [
                fixture,
                "agent-ix/provider/configured-signal",
                request_marker,
                "value-validity",
                "signal",
                ready_marker,
                release_marker,
                descendant_marker,
            ],
            "deadline_ms": 10_000,
            "limits": {
                "max_frame_bytes": 262_144,
                "max_total_bytes": 1_048_576,
                "max_stderr_bytes": 4096,
                "cancellation_grace_ms": 250,
            }
        }]
    });
    std::fs::write(
        &config_path,
        serde_json::to_vec(&config).expect("serialize provider config"),
    )
    .expect("write provider config");

    let started = Instant::now();
    let mut child = Command::new(QUIRE)
        .args([
            "run",
            source_path.to_str().expect("UTF-8 source path"),
            "eq",
            "x=2",
            "y=3",
        ])
        .arg("--provider-config")
        .arg(&config_path)
        .args(["--format", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start production quire binary");
    let deadline = started + Duration::from_secs(5);
    loop {
        if child.try_wait().expect("poll quire child").is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("collect timed-out quire child");
            let _ = std::fs::write(&release_marker, b"release");
            panic!(
                "production signal flow did not exit within its bound: status={:?}, stdout={}, stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = child
        .wait_with_output()
        .expect("collect signalled quire child");
    let _ = std::fs::write(&release_marker, b"release");

    assert!(
        request_marker.is_file(),
        "fixture received a real FR-331 request"
    );
    assert!(
        ready_marker.is_file(),
        "descendant reached its ready handshake"
    );
    assert_eq!(
        output.status.signal(),
        Some(signal_hook::consts::signal::SIGTERM),
        "the production CLI restores the default action and re-raises SIGTERM"
    );
    assert!(
        output.stdout.is_empty(),
        "signal cleanup emits no outcome document"
    );
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "cleanup remains bounded"
    );

    let descendant_deadline = Instant::now() + Duration::from_secs(2);
    while !descendant_marker.exists() && Instant::now() < descendant_deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        !descendant_marker.exists(),
        "the process-group descendant was reaped before the CLI re-raised SIGTERM"
    );
    std::fs::remove_dir_all(directory).expect("remove signal test directory");
}

/// Trace: FR-002-AC-1, FR-002-AC-8; root argv refusals name the rejected argument.
#[test]
fn root_argv_refusal_names_invalid_format_without_result_output() {
    let output = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--format", "dot"])
        .output()
        .expect("run quire with an invalid root format");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("dot"));
}

/// Trace: FR-002-AC-1, FR-002-AC-8; run-only options are named when refused on another verb.
#[test]
fn non_run_verb_refuses_and_names_run_only_option() {
    for option in ["--engine", "--limit"] {
        let output = Command::new(QUIRE)
            .args(["check", option, "gpu"])
            .output()
            .expect("run quire with a run-only option on check");
        assert_eq!(output.status.code(), Some(20), "{option}");
        assert!(output.stdout.is_empty(), "{option}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(option),
            "{option}"
        );
    }
}

/// Trace: FR-004-AC-3, unavailable engines refuse before reading a source.
#[test]
fn unavailable_engines_are_not_substituted() {
    for engine in ["aot", "jit"] {
        let output = Command::new(QUIRE)
            .args([
                "run",
                "missing-source.qsl",
                "yes",
                "--engine",
                engine,
                "--format",
                "json",
            ])
            .output()
            .expect("run quire");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Unsupported.exit_code())),
            "{engine}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(engine),
            "typed unsupported refusal names {engine}"
        );
        let document: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("unsupported result is QSL JSON");
        assert_eq!(document["category"], "unsupported");
        assert_eq!(
            document["diagnostics"][0]["code"],
            "unimplemented_capability"
        );
    }
}

/// Trace: FR-004-AC-2, a value outside the engine set is refused.
#[test]
fn an_unknown_engine_is_a_refusal() {
    let output = Command::new(QUIRE)
        .args(["run", "missing-source.qsl", "yes", "--engine", "gpu"])
        .output()
        .expect("run quire");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("gpu"));
}

/// Trace: FR-003-AC-1, the exposed checking work bound reaches QSL unchanged.
#[test]
fn checking_work_limit_changes_the_call_outcome() {
    let exhausted = Command::new(QUIRE)
        .args([
            "run",
            SOURCE,
            "yes",
            "--limit",
            "s3.work_units=0",
            "--format",
            "json",
        ])
        .output()
        .expect("run with exhausted work limit");
    assert_eq!(
        exhausted.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let sufficient = Command::new(QUIRE)
        .args([
            "run",
            SOURCE,
            "yes",
            "--limit",
            "s3.work_units=1000",
            "--format",
            "json",
        ])
        .output()
        .expect("run with sufficient work limit");
    assert_eq!(
        sufficient.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-003-AC-9, FR-003-AC-10. QSL-named accounting reaches QSL's call.
#[test]
fn named_accounting_work_units_limit_reaches_qsl_call_accounting() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            SOURCE,
            "yes",
            "--limit",
            "accounting.work_units=0",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a zero accounting budget");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("QSL writes an incomplete document");
    assert_eq!(document["category"], "incomplete");
    assert_eq!(document["result"]["kind"], "incomplete");
    assert_eq!(
        document["result"]["limit"]["setting"],
        "accounting.work_units"
    );
}

/// Trace: FR-003-AC-3 / TC-010, a lifecycle stage limit uses QSL's exit code.
#[test]
fn lifecycle_stage_limit_uses_incomplete_exit_code() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            SOURCE,
            "yes",
            "--limit",
            "s1.input_bytes=1",
            "--format",
            "json",
        ])
        .output()
        .expect("run with an exhausted source-byte limit");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Incomplete.exit_code()))
    );

    let source_bytes = std::fs::read(SOURCE).expect("read fixture to set exact source bound");
    let bound = format!("s1.input_bytes={}", source_bytes.len());
    let raised = Command::new(QUIRE)
        .args(["run", SOURCE, "yes", "--limit", &bound, "--format", "json"])
        .output()
        .expect("run with the source bound raised to the fixture length");
    assert_eq!(
        raised.status.code(),
        Some(i32::from(Category::Success.exit_code()))
    );
}

/// Trace: FR-003-AC-2, FR-003-AC-7, FR-003-AC-8.
#[test]
fn qsl_rejects_unknown_malformed_and_duplicate_limits_before_source_read() {
    for operand in [
        "work_units=0", // A bare counter is not a setting (QSL FR-255 Behavior 8).
        "normalization.work_units=1",
        "s1.input_bytes=invalid",
        "s1.input_bytes=1", // duplicated below using the same option twice
    ] {
        let mut command = Command::new(QUIRE);
        command.args(["run", "missing-source.qsl", "yes", "--limit", operand]);
        if operand == "s1.input_bytes=1" {
            command.args(["--limit", operand]);
        }
        let output = command.output().expect("run with invalid QSL limit");
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Refusal.exit_code())),
            "{operand}"
        );
        assert!(output.stdout.is_empty(), "{operand}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(operand), "{operand}: {stderr}");
        if operand == "work_units=0" {
            assert!(stderr.contains("UnknownSetting"), "{stderr}");
        }
        assert!(
            !stderr.contains("could not read source"),
            "{operand}: {stderr}"
        );
    }
}

/// Trace: FR-003-AC-5, QSL-recognized settings absent from run bounds are distinct refusals.
#[test]
fn recognized_setting_unavailable_for_run_is_named_before_source_read() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "explore.states=2",
            "--format",
            "json",
        ])
        .output()
        .expect("run with a setting unavailable to run");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Refusal.exit_code()))
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("explore.states"), "{stderr}");
    assert!(
        stderr.contains("unavailable for the `run` operation"),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read source"), "{stderr}");
    assert_eq!(
        output.stdout,
        expected_document_bytes(Code::InvalidRequest, stderr.trim_end())
    );
}

/// Trace: FR-003-AC-2, parser refusals take precedence over operation bounds.
#[test]
fn invalid_operand_precedes_recognized_but_unavailable_setting() {
    let output = Command::new(QUIRE)
        .args([
            "run",
            "missing-source.qsl",
            "yes",
            "--limit",
            "explore.states=2",
            "--limit",
            "normalization.work_units=1",
        ])
        .output()
        .expect("run with an unknown operand and unavailable setting");
    assert_eq!(output.status.code(), Some(20));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("normalization.work_units"), "{stderr}");
    assert!(
        !stderr.contains("unavailable for the `run` operation"),
        "{stderr}"
    );
    assert!(!stderr.contains("could not read source"), "{stderr}");
}

/// Trace: FR-002-AC-8; a full stderr device does not replace a refusal exit with a panic.
#[cfg(unix)]
#[test]
fn refusal_keeps_its_exit_code_when_stderr_is_unwritable() {
    let status = status_with_unwritable_stderr(&["run", SOURCE, "yes", "--format", "dot"], false);
    assert_eq!(status.code(), Some(20));
}

/// Trace: FR-002-AC-14; a broken primary stdout pipe preserves the QSL category result.
#[cfg(all(unix, feature = "tc018-test-adapter"))]
#[test]
fn fifo_broken_pipe_keeps_refusal_exit_without_output_failure_diagnostic() {
    let directory = TestDirectory::new("tc018-broken-pipe");
    let source_path = directory.path().join("driver-check.native");
    let source_text = source_path.to_str().expect("temporary path is UTF-8");
    let fifo_status = Command::new("mkfifo")
        .arg(&source_path)
        .status()
        .expect("create named-pipe source fixture");
    assert!(fifo_status.success(), "mkfifo exited with {fifo_status}");

    let bytes = overflow_fixture_bytes();
    let expected = expected_overflow_check_document(source_text, &bytes);
    let request_path = directory.path().join("tc018-request.json");
    assert!(
        expected
            .windows(b"undefined_expression".len())
            .any(|window| window == b"undefined_expression"),
        "the shared overflow fixture must retain its QSL refusal code"
    );
    assert!(
        expected
            .windows(b"unproved-range".len())
            .any(|window| window == b"unproved-range"),
        "the shared overflow fixture must retain its QSL refusal cause"
    );

    let child = Command::new(QUIRE)
        .args(["check", source_text, "--format", "json"])
        .env("TC018_CASE", "refused-range")
        .env("TC018_REQUEST_PATH", &request_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn quire check with piped stdout");
    let mut child = ChildGuard::new(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut writer = open_fifo_writer_until(&source_path, &mut child, deadline);
    drop(child.child().stdout.take());
    writer
        .write_all(&bytes)
        .expect("write the shared overflow fixture into the source FIFO");
    drop(writer);

    let status = wait_for_child_until(&mut child, deadline);
    assert_tc018_refused_range_observation(&request_path, source_text, &bytes);
    let stderr_bytes = read_child_stderr(&mut child);
    let stderr = String::from_utf8_lossy(&stderr_bytes);
    assert_eq!(
        status.code(),
        Some(i32::from(Category::Refusal.exit_code())),
        "broken pipe preserves the independently computed QSL category exit"
    );
    assert!(
        !stderr.contains("OutputFailure"),
        "BrokenPipe must not emit OutputFailure: {stderr}"
    );
}

/// Trace: FR-002-AC-15, FR-002-AC-16; non-broken stdout I/O errors select InternalFailure and diagnose.
#[cfg(all(target_os = "linux", feature = "tc018-test-adapter"))]
#[test]
fn full_stdout_device_selects_internal_failure_and_output_failure_diagnostic() {
    let directory = TestDirectory::new("tc018-full-stdout");
    let source_path = directory.path().join("driver-check.native");
    let bytes = overflow_fixture_bytes();
    std::fs::write(&source_path, &bytes).expect("write the shared overflow source fixture");
    let source_text = source_path.to_str().expect("temporary path is UTF-8");
    let expected = expected_overflow_check_document(source_text, &bytes);
    let request_path = directory.path().join("tc018-request.json");
    assert!(
        expected
            .windows(b"undefined_expression".len())
            .any(|window| window == b"undefined_expression"),
        "the shared overflow fixture must retain its QSL refusal code"
    );

    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open Linux full device");
    let child = Command::new(QUIRE)
        .args(["check", source_text, "--format", "json"])
        .env("TC018_CASE", "refused-range")
        .env("TC018_REQUEST_PATH", &request_path)
        .stdout(Stdio::from(full))
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn quire check with /dev/full as stdout");
    let mut child = ChildGuard::new(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = wait_for_child_until(&mut child, deadline);
    assert_tc018_refused_range_observation(&request_path, source_text, &bytes);
    let stderr_bytes = read_child_stderr(&mut child);
    let stderr = String::from_utf8_lossy(&stderr_bytes);

    assert_eq!(
        status.code(),
        Some(i32::from(Category::InternalFailure.exit_code()))
    );
    assert!(
        stderr.contains("OutputFailure: could not write outcome document"),
        "non-BrokenPipe failure is visibly classified and identifies the output: {stderr}"
    );
}

/// Trace: FR-002-AC-6, the reserved QSL result falls back without stdout output.
#[cfg(feature = "tc018-test-adapter")]
#[test]
fn reserved_exit_code_uses_internal_failure_fallback() {
    let version = Command::new(QUIRE)
        .arg("--version")
        .env("TC018_CASE", "reserved-exit")
        .output()
        .expect("run the test-built quire binary with --version");
    assert_eq!(version.status.code(), Some(30));
    assert!(version.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&version.stderr),
        "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30.\n"
    );

    for format in ["text", "json"] {
        let output = Command::new(QUIRE)
            .args(["run", SOURCE, "yes", "--format", format])
            .env("TC018_CASE", "reserved-exit")
            .output()
            .expect("run the test-built quire binary with a valid verb");
        assert_eq!(output.status.code(), Some(30), "{format}");
        assert!(output.stdout.is_empty(), "{format}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("reserved code 5"), "{format}: {stderr}");
        assert!(stderr.contains("fallback code 30"), "{format}: {stderr}");
        assert!(
            stderr.contains(
                "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30."
            ),
            "{format}: {stderr}"
        );
        if format == "json" {
            assert_eq!(
                stderr.as_ref(),
                "driver exit mapping rejected result 5 (reserved code 5); using fallback code 30.\n",
                "{format}"
            );
        } else {
            assert!(
                stderr.contains("unimplemented_capability"),
                "{format}: {stderr}"
            );
        }
    }

    #[cfg(unix)]
    {
        let status =
            status_with_unwritable_stderr(&["run", SOURCE, "yes", "--format", "json"], true);
        assert_eq!(status.code(), Some(30));
    }
}

/// The normal binary ignores TC-018's selector even when the caller sets it.
#[cfg(not(feature = "tc018-test-adapter"))]
#[test]
fn default_binary_does_not_enable_the_test_exit_seam() {
    let output = Command::new(QUIRE)
        .args(["check", SOURCE, "--format", "json"])
        .env("TC018_CASE", "reserved-exit")
        .output()
        .expect("run the default-feature quire binary");
    assert_eq!(output.status.code(), Some(0));
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check writes a QSL outcome");
    assert_eq!(document["operation"], "check");
    assert!(output.stderr.is_empty());
}
