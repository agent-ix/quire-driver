// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linux and macOS signal forwarding for explicitly routed plugin sessions.
//!
//! Ordinary CLI commands do not install handlers. During a routed plugin
//! session an inherited `SIG_IGN` disposition is overridden by `signal-hook`
//! for the process lifetime; cleanup on plugin-session signals takes precedence.
//!
//! The tests `signal_is_forwarded_after_active_cleanup_and_defaults_without_a_session`
//! and `plugin_session_signals_cancel_and_reap_each_active_lifecycle_stage`
//! require an unsandboxed runner with cross-process signal and process-group
//! support. A sandbox failure can show `signal() == None` where SIGTERM/SIGINT
//! was expected, or a bounded-wait timeout; previous sandbox runs failed in
//! about 6–7 s versus about 14 s with normal permissions. Rerun under normal,
//! unsandboxed permissions.

use std::collections::BTreeMap;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::atomic::AtomicI32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use quire_exact::Cancel;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use quire_exact::CancelCause;

/// Shared cancellation state used by the CLI operation and process adapter.
#[derive(Clone, Default)]
pub(crate) struct SignalContext {
    shared: Arc<Shared>,
}

#[derive(Default)]
struct Shared {
    next_token: AtomicU64,
    active: Mutex<BTreeMap<u64, Cancel>>,
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    received: AtomicI32,
}

fn lock_active(shared: &Shared) -> MutexGuard<'_, BTreeMap<u64, Cancel>> {
    shared
        .active
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn emulate_default_handler(signal: i32) {
    if let Err(error) = signal_hook::low_level::emulate_default_handler(signal) {
        eprintln!("failed to restore default signal action for {signal}: {error}");
        std::process::abort();
    }
}

pub(crate) struct ActiveSession {
    context: SignalContext,
    token: u64,
}

impl SignalContext {
    pub(crate) fn activate(&self, cancel: Cancel) -> ActiveSession {
        let token = self.shared.next_token.fetch_add(1, Ordering::Relaxed);
        lock_active(&self.shared).insert(token, cancel);
        ActiveSession {
            context: self.clone(),
            token,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn observe(&self, signal: i32) {
        let active = lock_active(&self.shared);
        if !active.is_empty() {
            for cancel in active.values() {
                cancel.cancel(CancelCause::Requested);
            }
            if self
                .shared
                .received
                .compare_exchange(0, signal, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                // Keep the first signal so cleanup settles against one cause.
            }
        } else {
            drop(active);
            emulate_default_handler(signal);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn reraise_received(&self) {
        let signal = self.shared.received.swap(0, Ordering::AcqRel);
        if signal != 0 {
            emulate_default_handler(signal);
        }
    }
}

impl Drop for ActiveSession {
    fn drop(&mut self) {
        let mut active = lock_active(&self.context.shared);
        active.remove(&self.token);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let became_empty = active.is_empty();
        drop(active);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if became_empty {
            self.context.reraise_received();
        }
    }
}

/// Run an operation that routes a plugin session with a Linux/macOS signal
/// listener. Signals cancel every registered QSL handle; the original signal
/// is re-raised when the final active session completes bounded cleanup. With no
/// active session, the listener emulates the default action. The listener
/// remains until process exit so a signal after session drop is not swallowed.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[allow(
    dead_code,
    reason = "called only by a command that routes a plugin session"
)]
pub(crate) fn run<T: Send + 'static>(
    operation: impl FnOnce(SignalContext) -> T + Send + 'static,
) -> Result<T, std::io::Error> {
    use signal_hook::consts::signal::{SIGINT, SIGTERM};
    use signal_hook::iterator::Signals;
    use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
    use std::sync::mpsc;
    use std::sync::OnceLock;

    static LISTENER: OnceLock<Result<SignalContext, String>> = OnceLock::new();
    let context = LISTENER
        .get_or_init(|| {
            let context = SignalContext::default();
            let listener_context = context.clone();
            let (ready_tx, ready_rx) = mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("quire-cli-signal-listener".to_owned())
                .spawn(move || {
                    let mut signals = match Signals::new([SIGINT, SIGTERM]) {
                        Ok(signals) => {
                            let _ = ready_tx.send(Ok(()));
                            signals
                        }
                        Err(error) => {
                            let _ = ready_tx.send(Err(error.to_string()));
                            return;
                        }
                    };
                    for signal in signals.forever() {
                        listener_context.observe(signal);
                    }
                })
                .map_err(|error| error.to_string())?;
            ready_rx.recv().map_err(|error| error.to_string())??;
            Ok(context)
        })
        .clone()
        .map_err(|error| std::io::Error::other(error.clone()))?;
    let worker_context = context.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("quire-cli-operation".to_owned())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let result = catch_unwind(AssertUnwindSafe(|| operation(worker_context)));
            let _ = sender.send(result);
        })?;

    let result = receiver
        .recv()
        .map_err(|_| std::io::Error::other("CLI operation thread stopped"))?;
    match result {
        Ok(value) => Ok(value),
        Err(payload) => resume_unwind(payload),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[allow(
    dead_code,
    reason = "called only by a command that routes a plugin session"
)]
pub(crate) fn run<T>(operation: impl FnOnce(SignalContext) -> T) -> Result<T, std::io::Error> {
    Ok(operation(SignalContext::default()))
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "signal harness failures are the test failure report"
)]
mod tests {
    //! `signal_is_forwarded_after_active_cleanup_and_defaults_without_a_session`
    //! and `plugin_session_signals_cancel_and_reap_each_active_lifecycle_stage`
    //! need normal, unsandboxed cross-process signal and process-group
    //! permissions. Sandbox failures can report `signal() == None` where
    //! SIGTERM/SIGINT was expected, or a bounded-wait timeout; rerun with normal
    //! permissions.

    use super::{lock_active, SignalContext};
    use quire_exact::Cancel;

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn read_child_output(child: &mut std::process::Child) -> (Vec<u8>, Vec<u8>) {
        use std::io::Read;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        child
            .stdout
            .take()
            .expect("captured child stdout")
            .read_to_end(&mut stdout)
            .expect("read child stdout");
        child
            .stderr
            .take()
            .expect("captured child stderr")
            .read_to_end(&mut stderr)
            .expect("read child stderr");
        (stdout, stderr)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn child_failure(status: std::process::ExitStatus, stdout: &[u8], stderr: &[u8]) -> String {
        use std::os::unix::process::ExitStatusExt;

        format!(
            "child status={status:?}, code={:?}, signal={:?}, stdout={}, stderr={}",
            status.code(),
            status.signal(),
            String::from_utf8_lossy(stdout),
            String::from_utf8_lossy(stderr)
        )
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    struct ClearReceivedOnDrop<'a>(&'a SignalContext);

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    impl Drop for ClearReceivedOnDrop<'_> {
        fn drop(&mut self) {
            self.0
                .shared
                .received
                .store(0, std::sync::atomic::Ordering::Release);
        }
    }

    #[test]
    fn active_session_is_cleared_when_its_guard_drops() {
        let context = SignalContext::default();
        let cancel = Cancel::new();
        let guard = context.activate(cancel);
        assert_eq!(lock_active(&context.shared).len(), 1);
        drop(guard);
        assert!(lock_active(&context.shared).is_empty());
    }

    #[test]
    fn dropping_one_session_keeps_other_sessions_active() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        use quire_exact::CancelCause;

        let context = SignalContext::default();
        let first = Cancel::new();
        let second = Cancel::new();
        let first_guard = context.activate(first.clone());
        let second_guard = context.activate(second.clone());
        drop(second_guard);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        context.observe(signal_hook::consts::signal::SIGTERM);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let _clear_received = ClearReceivedOnDrop(&context);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        assert_eq!(first.cause(), Some(CancelCause::Requested));
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        assert_eq!(second.cause(), None);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        drop(_clear_received);
        drop(first_guard);
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn operation_panic_is_resumed_instead_of_reported_as_channel_disconnect() {
        let panic = std::panic::catch_unwind(|| {
            let _ = super::run(|_| -> () { panic!("operation panic sentinel") });
        });
        assert!(panic.is_err());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn handled_signal_cancels_the_active_qsl_handle_and_retains_its_number() {
        use quire_exact::CancelCause;
        use signal_hook::consts::signal::SIGTERM;

        let context = SignalContext::default();
        let cancel = Cancel::new();
        let _guard = context.activate(cancel.clone());
        context.observe(SIGTERM);
        let _clear_received = ClearReceivedOnDrop(&context);
        assert_eq!(cancel.cause(), Some(CancelCause::Requested));
        assert_eq!(
            context
                .shared
                .received
                .load(std::sync::atomic::Ordering::Acquire),
            SIGTERM
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn signal_is_forwarded_after_active_cleanup_and_defaults_without_a_session() {
        use std::os::unix::process::ExitStatusExt;
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::{Duration, Instant};

        use quire_exact::{Cancel, CancelCause};
        use signal_hook::consts::signal::{SIGINT, SIGTERM};

        if let (Ok(mode), Ok(marker)) = (
            std::env::var("QUIRE_SIGNAL_TEST_MODE"),
            std::env::var("QUIRE_SIGNAL_TEST_MARKER"),
        ) {
            let marker = std::path::PathBuf::from(marker);
            if mode == "post-session" {
                super::run(|context| {
                    let _active = context.activate(Cancel::new());
                    17
                })
                .expect("signal wrapper starts");
                std::fs::write(&marker, b"session-ended").expect("write post-session marker");
                signal_hook::low_level::raise(SIGTERM).expect("raise post-session SIGTERM");
                thread::sleep(Duration::from_secs(10));
                return;
            }
            super::run(move |context| {
                if mode == "active" {
                    let cancel = Cancel::new();
                    let _active = context.activate(cancel.clone());
                    std::fs::write(&marker, b"active").expect("write active marker");
                    let deadline = std::time::Instant::now() + Duration::from_secs(3);
                    while !marker.with_extension("trigger").is_file()
                        && std::time::Instant::now() < deadline
                    {
                        thread::sleep(Duration::from_millis(2));
                    }
                    signal_hook::low_level::raise(SIGTERM).expect("raise test SIGTERM");
                    while cancel.cause().is_none() {
                        thread::sleep(Duration::from_millis(2));
                    }
                    assert_eq!(cancel.cause(), Some(CancelCause::Requested));
                    thread::sleep(Duration::from_millis(30));
                    std::fs::write(marker.with_extension("cleaned"), b"cleaned")
                        .expect("record completed cancellation cleanup");
                    drop(_active);
                    std::fs::write(marker.with_extension("outcome"), b"outcome")
                        .expect("signal must be re-raised before outcome");
                } else {
                    std::fs::write(&marker, b"idle").expect("write idle marker");
                    let deadline = std::time::Instant::now() + Duration::from_secs(3);
                    while !marker.with_extension("trigger").is_file()
                        && std::time::Instant::now() < deadline
                    {
                        thread::sleep(Duration::from_millis(2));
                    }
                    signal_hook::low_level::raise(SIGINT).expect("raise test SIGINT");
                    thread::sleep(Duration::from_secs(10));
                }
                17
            })
            .expect("signal wrapper starts");
            println!("QUIRE_OUTCOME_SENTINEL");
            return;
        }

        for mode in ["active", "idle", "post-session"] {
            let marker = std::env::temp_dir()
                .join(format!("quire-cli-signal-{}-{mode}", std::process::id()));
            let trigger = marker.with_extension("trigger");
            let cleaned = marker.with_extension("cleaned");
            let outcome = marker.with_extension("outcome");
            let _ = std::fs::remove_file(&marker);
            let _ = std::fs::remove_file(&trigger);
            let _ = std::fs::remove_file(&cleaned);
            let _ = std::fs::remove_file(&outcome);
            let mut child = Command::new(std::env::current_exe().expect("test executable"))
                .args([
                    "--quiet",
                    "--exact",
                    "signals::tests::signal_is_forwarded_after_active_cleanup_and_defaults_without_a_session",
                ])
                .env("QUIRE_SIGNAL_TEST_MODE", mode)
                .env("QUIRE_SIGNAL_TEST_MARKER", &marker)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn signal fixture process");
            let deadline = Instant::now() + Duration::from_secs(3);
            while !marker.is_file() && Instant::now() < deadline {
                if let Some(status) = child.try_wait().expect("poll child") {
                    let (stdout, stderr) = read_child_output(&mut child);
                    panic!(
                        "{mode} child exited before ready: {}",
                        child_failure(status, &stdout, &stderr)
                    );
                }
                thread::sleep(Duration::from_millis(5));
            }
            if !marker.is_file() {
                let _ = child.kill();
                let status = child.wait().expect("wait for child that missed readiness");
                let (stdout, stderr) = read_child_output(&mut child);
                panic!(
                    "{mode} child did not become ready within 3 s: {}",
                    child_failure(status, &stdout, &stderr)
                );
            }
            if mode != "post-session" {
                std::fs::write(&trigger, b"signal").expect("trigger child signal");
            }
            let deadline = Instant::now() + Duration::from_secs(4);
            let status = loop {
                if let Some(status) = child.try_wait().expect("poll signal child") {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let status = child.wait().expect("wait for timed-out signal child");
                    let (stdout, stderr) = read_child_output(&mut child);
                    let _ = std::fs::remove_file(&marker);
                    let _ = std::fs::remove_file(&trigger);
                    let _ = std::fs::remove_file(&cleaned);
                    panic!(
                        "{mode} signal child exceeded its bounded wait: {}",
                        child_failure(status, &stdout, &stderr)
                    );
                }
                thread::sleep(Duration::from_millis(5));
            };
            let expected = if mode != "idle" {
                SIGTERM
            } else {
                signal_hook::consts::signal::SIGINT
            };
            let (stdout, stderr) = read_child_output(&mut child);
            let failure = child_failure(status, &stdout, &stderr);
            assert_eq!(status.signal(), Some(expected), "{mode}: {failure}");
            assert!(
                !stdout
                    .windows(b"QUIRE_OUTCOME_SENTINEL".len())
                    .any(|window| window == b"QUIRE_OUTCOME_SENTINEL"),
                "signal path emitted outcome bytes: {failure}"
            );
            assert!(
                !outcome.exists(),
                "signal path wrote outcome document; {failure}"
            );
            if mode == "active" {
                assert!(
                    cleaned.is_file(),
                    "cleanup finishes before re-raise; {failure}"
                );
            }
            let _ = std::fs::remove_file(marker);
            let _ = std::fs::remove_file(trigger);
            let _ = std::fs::remove_file(cleaned);
            let _ = std::fs::remove_file(outcome);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn plugin_session_signals_cancel_and_reap_each_active_lifecycle_stage() {
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::{Duration, Instant};

        use qsl_route::{BackendDescriptor, BackendId, Candidate, ManifestDigest, ProviderOrigin};
        use quire_driver::{
            ProcessProvider, ProcessProviderLimits, ProcessProviderRequest, ProcessProviderSession,
            ProviderManifest, ProviderRequest, ProviderResults,
        };
        use quire_exact::Cancel;
        use serde_json::Value;

        struct RemoveMarkers(Vec<std::path::PathBuf>);
        impl Drop for RemoveMarkers {
            fn drop(&mut self) {
                for path in &self.0 {
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        if let (Ok(stage), Ok(ready), Ok(descendant), Ok(plugin_marker), Ok(outcome)) = (
            std::env::var("QUIRE_PLUGIN_SIGNAL_STAGE"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_READY"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_DESCENDANT"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_PLUGIN_MARKER"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_OUTCOME"),
        ) {
            super::run(move |context| {
                let python = std::env::split_paths(&std::env::var_os("PATH").expect("PATH"))
                    .map(|directory| directory.join("python3"))
                    .find(|candidate| candidate.is_file())
                    .expect("Python fixture interpreter");
                let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/signal-plugin.py");
                let arguments = [
                    fixture.into_os_string(),
                    stage.clone().into(),
                    ready.clone().into(),
                    descendant.clone().into(),
                    plugin_marker.clone().into(),
                ];
                let provider = crate::process_provider::HostProcessProvider::<
                    Value,
                    Value,
                    Value,
                    Value,
                    String,
                    String,
                    _,
                    _,
                    _,
                    _,
                >::new(
                    context,
                    |manifest: &Value| {
                        let identity = manifest["identity"].as_str().ok_or("missing identity")?;
                        let digest = ManifestDigest::from_digest([4; 32]);
                        let descriptor = BackendDescriptor::admit(
                            Candidate::new(
                                BackendId::from_wire(identity).map_err(|e| e.to_string())?,
                            ),
                            ProviderOrigin::Process,
                            digest,
                            Vec::<(Option<&str>, Option<&str>)>::new(),
                        )
                        .map_err(|e| format!("{e:?}"))?;
                        Ok((
                            descriptor,
                            ProviderManifest {
                                identity: identity.to_owned(),
                                manifest_digest: digest,
                                advertises: Vec::new(),
                                domains: None,
                                wire: None,
                            },
                        ))
                    },
                    |_: ProcessProviderRequest<'_>| serde_json::json!({"items": []}),
                    |_: &Value, _: &Value| Ok::<(), String>(()),
                    |_: Value| ProviderResults::default(),
                );
                let limits = ProcessProviderLimits {
                    max_frame_bytes: 4096,
                    max_total_bytes: 16_384,
                    max_stderr_bytes: 1024,
                    cancellation_grace: Duration::from_millis(100),
                    max_process_memory_bytes: None,
                    max_process_cpu_time: None,
                };
                let start_cancel = Cancel::new();
                let mut session = match provider.start(&python, &arguments, limits, &start_cancel) {
                    Ok(session) => session,
                    Err(_) if stage.starts_with("hello") => return,
                    Err(error) => panic!("adapter start failed unexpectedly: {error:?}"),
                };
                if stage.starts_with("request") {
                    let request_cancel = Cancel::new();
                    let package = crate::test_support::emitted_package();
                    let request = ProcessProviderRequest {
                        emitted_package: &package,
                        deadline_ms: 4096,
                        manifest: &[],
                        provider_request: ProviderRequest::new(&[]),
                        cancel: &request_cancel,
                    };
                    let _ = session.invoke(request);
                    if request_cancel.cause().is_some() {
                        std::fs::write(
                            std::path::PathBuf::from(&ready).with_extension("request-cancelled"),
                            b"yes",
                        )
                        .expect("record request handle cancellation");
                    }
                }
                let _ = session.shutdown();
                std::fs::write(outcome, b"QUIRE_OUTCOME_SENTINEL")
                    .expect("a signal must be re-raised before outcome settlement");
            })
            .expect("signal wrapper starts");
            println!("QUIRE_OUTCOME_SENTINEL");
            panic!("adapter signal was not re-raised");
        }

        use std::os::unix::process::ExitStatusExt;
        let root = std::env::temp_dir().join(format!("quire-host-signal-{}", std::process::id()));
        for stage in [
            "hello",
            "hello-int",
            "request",
            "request-int",
            "shutdown",
            "shutdown-int",
        ] {
            let ready = root.with_extension(format!("{stage}.ready"));
            let descendant = root.with_extension(format!("{stage}.descendant"));
            let plugin_marker = root.with_extension(format!("{stage}.plugin"));
            let outcome = root.with_extension(format!("{stage}.outcome"));
            let request_cancelled = ready.with_extension("request-cancelled");
            let _cleanup = RemoveMarkers(vec![
                ready.clone(),
                descendant.clone(),
                plugin_marker.clone(),
                outcome.clone(),
                request_cancelled.clone(),
                ready.with_extension("error"),
            ]);
            let _ = std::fs::remove_file(&ready);
            let _ = std::fs::remove_file(&descendant);
            let _ = std::fs::remove_file(&plugin_marker);
            let _ = std::fs::remove_file(&outcome);
            let _ = std::fs::remove_file(&request_cancelled);
            let mut child = Command::new(std::env::current_exe().expect("test executable"))
                .args([
                    "--quiet",
                    "--exact",
                    "signals::tests::plugin_session_signals_cancel_and_reap_each_active_lifecycle_stage",
                ])
                    .env("QUIRE_PLUGIN_SIGNAL_STAGE", stage)
                    .env("QUIRE_PLUGIN_SIGNAL_READY", &ready)
                    .env("QUIRE_PLUGIN_SIGNAL_DESCENDANT", &descendant)
                .env("QUIRE_PLUGIN_SIGNAL_PLUGIN_MARKER", &plugin_marker)
                .env("QUIRE_PLUGIN_SIGNAL_OUTCOME", &outcome)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn signal lifecycle test child");
            let started = Instant::now();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !ready.is_file() && Instant::now() < deadline {
                if let Some(status) = child.try_wait().expect("poll test child") {
                    let error =
                        std::fs::read_to_string(ready.with_extension("error")).unwrap_or_default();
                    let (stdout, stderr) = read_child_output(&mut child);
                    panic!(
                        "{stage} child exited before ready: {error}; {}",
                        child_failure(status, &stdout, &stderr)
                    );
                }
                thread::sleep(Duration::from_millis(5));
            }
            if !ready.is_file() {
                let _ = child.kill();
                let status = child.wait().expect("wait for child that missed readiness");
                let (stdout, stderr) = read_child_output(&mut child);
                panic!(
                    "{stage} fixture did not become ready within 5 s; {}",
                    child_failure(status, &stdout, &stderr)
                );
            }
            let status = loop {
                if let Some(status) = child.try_wait().expect("poll lifecycle child") {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let status = child.wait().expect("wait for timed-out lifecycle child");
                    let (stdout, stderr) = read_child_output(&mut child);
                    panic!(
                        "{stage} lifecycle child did not finish within bounded signal wait: {}",
                        child_failure(status, &stdout, &stderr)
                    );
                }
                thread::sleep(Duration::from_millis(5));
            };
            let elapsed = started.elapsed();
            let expected = if stage.ends_with("-int") {
                signal_hook::consts::signal::SIGINT
            } else {
                signal_hook::consts::signal::SIGTERM
            };
            let (stdout, stderr) = read_child_output(&mut child);
            let failure = child_failure(status, &stdout, &stderr);
            assert_eq!(status.signal(), Some(expected), "{stage}: {failure}");
            assert!(
                elapsed < Duration::from_millis(1500),
                "{stage} signal must cancel and clean the plugin promptly; {failure}"
            );
            assert!(
                !stdout
                    .windows(b"QUIRE_OUTCOME_SENTINEL".len())
                    .any(|window| window == b"QUIRE_OUTCOME_SENTINEL"),
                "{stage}: signal wrote outcome bytes: {failure}"
            );
            assert!(
                !outcome.exists(),
                "{stage}: signal wrote outcome document; {failure}"
            );
            if stage.starts_with("request") {
                assert!(
                    request_cancelled.exists(),
                    "{stage}: invoke request handle was registered; {failure}"
                );
            }
            thread::sleep(Duration::from_millis(2200));
            assert!(
                !descendant.exists(),
                "{stage}: descendant was reaped; {failure}"
            );
            assert!(
                !plugin_marker.exists(),
                "{stage}: plugin was reaped; {failure}"
            );
            let _ = std::fs::remove_file(ready);
            let _ = std::fs::remove_file(descendant);
        }
    }
}
