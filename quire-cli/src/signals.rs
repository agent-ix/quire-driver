// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linux and macOS signal forwarding for explicitly routed plugin sessions.
//!
//! Ordinary CLI commands do not install handlers. During an active plugin
//! session an inherited `SIG_IGN` disposition is necessarily overridden by
//! `signal-hook`; cleanup on plugin-session signals takes precedence there.

use std::collections::BTreeMap;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::atomic::AtomicI32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use quire_exact::{Cancel, CancelCause};

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
            let _ = self.shared.received.compare_exchange(
                0,
                signal,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
        } else {
            drop(active);
            let _ = signal_hook::low_level::emulate_default_handler(signal);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn reraise_received(&self) {
        let signal = self.shared.received.swap(0, Ordering::AcqRel);
        if signal != 0 {
            let _ = signal_hook::low_level::emulate_default_handler(signal);
        }
    }
}

impl Drop for ActiveSession {
    fn drop(&mut self) {
        let mut active = lock_active(&self.context.shared);
        active.remove(&self.token);
    }
}

/// Run an operation that routes a plugin session with a Linux/macOS signal
/// listener. Signals cancel every registered QSL handle; the original signal
/// is re-raised after the operation completes bounded plugin cleanup. With no
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
            let mut signals = Signals::new([SIGINT, SIGTERM]).map_err(|error| error.to_string())?;
            let context = SignalContext::default();
            let listener_context = context.clone();
            std::thread::Builder::new()
                .name("quire-cli-signal-listener".to_owned())
                .spawn(move || {
                    for signal in signals.forever() {
                        listener_context.observe(signal);
                    }
                })
                .map_err(|error| error.to_string())?;
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
    context.reraise_received();
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
    use super::{lock_active, SignalContext};
    use quire_exact::Cancel;

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
        use quire_exact::CancelCause;

        let context = SignalContext::default();
        let first = Cancel::new();
        let second = Cancel::new();
        let first_guard = context.activate(first.clone());
        let second_guard = context.activate(second.clone());
        drop(second_guard);
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        context.observe(signal_hook::consts::signal::SIGTERM);
        assert_eq!(first.cause(), Some(CancelCause::Requested));
        assert_eq!(second.cause(), None);
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
        use std::io::Read;
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
                } else {
                    std::fs::write(&marker, b"idle").expect("write idle marker");
                    let deadline = std::time::Instant::now() + Duration::from_secs(3);
                    while !marker.with_extension("trigger").is_file()
                        && std::time::Instant::now() < deadline
                    {
                        thread::sleep(Duration::from_millis(2));
                    }
                    signal_hook::low_level::raise(SIGINT).expect("raise test SIGINT");
                }
                17
            })
            .expect("signal wrapper starts");
            return;
        }

        for mode in ["active", "idle", "post-session"] {
            let marker = std::env::temp_dir()
                .join(format!("quire-cli-signal-{}-{mode}", std::process::id()));
            let trigger = marker.with_extension("trigger");
            let cleaned = marker.with_extension("cleaned");
            let _ = std::fs::remove_file(&marker);
            let _ = std::fs::remove_file(&trigger);
            let _ = std::fs::remove_file(&cleaned);
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
                assert!(child.try_wait().expect("poll child").is_none());
                thread::sleep(Duration::from_millis(5));
            }
            assert!(marker.is_file(), "{mode} child became ready");
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
                    let _ = child.wait();
                    let _ = std::fs::remove_file(&marker);
                    let _ = std::fs::remove_file(&trigger);
                    let _ = std::fs::remove_file(&cleaned);
                    panic!("{mode} signal child exceeded its bounded wait");
                }
                thread::sleep(Duration::from_millis(5));
            };
            let expected = if mode != "idle" {
                SIGTERM
            } else {
                signal_hook::consts::signal::SIGINT
            };
            assert_eq!(status.signal(), Some(expected), "{mode} child status");
            let mut stdout = Vec::new();
            child
                .stdout
                .take()
                .expect("captured signal child stdout")
                .read_to_end(&mut stdout)
                .expect("read signal child stdout");
            let non_harness_output = stdout
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty() && *line != b"running 1 test");
            assert_eq!(
                non_harness_output.count(),
                0,
                "signal path emitted outcome bytes"
            );
            if mode == "active" {
                assert!(cleaned.is_file(), "cleanup finishes before re-raise");
            }
            let _ = std::fs::remove_file(marker);
            let _ = std::fs::remove_file(trigger);
            let _ = std::fs::remove_file(cleaned);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn plugin_session_signals_cancel_and_reap_each_active_lifecycle_stage() {
        use std::io::Read;
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::{Duration, Instant};

        use quire_exact::Cancel;
        use quire_plugin_host::{FrameLimits, PluginSession, RunLimits, SessionStart};
        use serde_json::Value;

        struct RemoveMarkers(Vec<std::path::PathBuf>);
        impl Drop for RemoveMarkers {
            fn drop(&mut self) {
                for path in &self.0 {
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        if let (Ok(stage), Ok(ready), Ok(descendant), Ok(plugin_marker)) = (
            std::env::var("QUIRE_PLUGIN_SIGNAL_STAGE"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_READY"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_DESCENDANT"),
            std::env::var("QUIRE_PLUGIN_SIGNAL_PLUGIN_MARKER"),
        ) {
            super::run(move |context| {
                let python = std::env::split_paths(&std::env::var_os("PATH").expect("PATH"))
                    .map(|directory| directory.join("python3"))
                    .find(|candidate| candidate.is_file())
                    .expect("Python fixture interpreter");
                let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/signal-plugin.py");
                let arguments = [
                    std::ffi::OsString::from(&stage),
                    std::ffi::OsString::from(&ready),
                    std::ffi::OsString::from(&descendant),
                    std::ffi::OsString::from(&plugin_marker),
                ];
                let cancel = Cancel::new();
                let _active = context.activate(cancel.clone());
                let session = PluginSession::<Value, Value, std::convert::Infallible>::start(
                    SessionStart {
                        executable: &python,
                        arguments: &[
                            fixture.into_os_string(),
                            arguments[0].clone(),
                            arguments[1].clone(),
                            arguments[2].clone(),
                            arguments[3].clone(),
                        ],
                        limits: RunLimits {
                            frames: FrameLimits {
                                max_frame_bytes: 4096,
                                max_total_bytes: 16_384,
                            },
                            max_stderr_bytes: 1024,
                            cancellation_grace: Duration::from_millis(100),
                            max_process_memory_bytes: None,
                            max_process_cpu_time: None,
                        },
                        cancel: &cancel,
                    },
                    |_| Ok::<(), std::convert::Infallible>(()),
                );
                if stage == "hello" {
                    if let Err(error) = session {
                        let _ = std::fs::write(
                            std::path::PathBuf::from(&ready).with_extension("error"),
                            format!("{error:?}"),
                        );
                    } else {
                        panic!("hello-stage signal did not cancel the pending hello");
                    }
                    return;
                }
                let mut session = session.expect("plugin hello is admitted");
                if stage.starts_with("request") {
                    let request_cancel = Cancel::new();
                    let _request_active = context.activate(request_cancel.clone());
                    let _ = session.request::<Value, Value, _>(
                        &serde_json::json!({"items": []}),
                        &request_cancel,
                        |_, _| Ok::<(), std::convert::Infallible>(()),
                    );
                } else {
                    let _ = session.shutdown();
                }
            })
            .expect("signal wrapper starts");
            panic!("plugin-session signal was not re-raised");
        }

        use std::os::unix::process::ExitStatusExt;
        let root = std::env::temp_dir().join(format!("quire-host-signal-{}", std::process::id()));
        for stage in ["hello", "request", "request-int", "shutdown"] {
            let ready = root.with_extension(format!("{stage}.ready"));
            let descendant = root.with_extension(format!("{stage}.descendant"));
            let plugin_marker = root.with_extension(format!("{stage}.plugin"));
            let _cleanup = RemoveMarkers(vec![
                ready.clone(),
                descendant.clone(),
                plugin_marker.clone(),
                ready.with_extension("error"),
            ]);
            let _ = std::fs::remove_file(&ready);
            let _ = std::fs::remove_file(&descendant);
            let _ = std::fs::remove_file(&plugin_marker);
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
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn signal lifecycle test child");
            let started = Instant::now();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !ready.is_file() && Instant::now() < deadline {
                if let Some(status) = child.try_wait().expect("poll test child") {
                    let mut stderr = String::new();
                    let mut stdout = String::new();
                    let error =
                        std::fs::read_to_string(ready.with_extension("error")).unwrap_or_default();
                    child
                        .stderr
                        .take()
                        .expect("captured child stderr")
                        .read_to_string(&mut stderr)
                        .expect("read child stderr");
                    child
                        .stdout
                        .take()
                        .expect("captured child stdout")
                        .read_to_string(&mut stdout)
                        .expect("read child stdout");
                    panic!(
                        "{stage} child exited before ready ({status}): {error}\n{stderr}\n{stdout}"
                    );
                }
                thread::sleep(Duration::from_millis(5));
            }
            assert!(
                ready.is_file(),
                "{stage} fixture reached its blocking point"
            );
            let status = loop {
                if let Some(status) = child.try_wait().expect("poll lifecycle child") {
                    break status;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("{stage} lifecycle child did not finish after SIGTERM");
                }
                thread::sleep(Duration::from_millis(5));
            };
            let expected = if stage == "request-int" {
                signal_hook::consts::signal::SIGINT
            } else {
                signal_hook::consts::signal::SIGTERM
            };
            assert_eq!(status.signal(), Some(expected));
            assert!(
                started.elapsed() < Duration::from_millis(1500),
                "{stage} signal must cancel and clean the plugin promptly"
            );
            let mut stdout = Vec::new();
            child
                .stdout
                .take()
                .expect("captured child stdout")
                .read_to_end(&mut stdout)
                .expect("read child stdout");
            let unexpected: Vec<_> = stdout
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty() && *line != b"running 1 test")
                .collect();
            assert!(unexpected.is_empty(), "signal wrote outcome bytes");
            thread::sleep(Duration::from_millis(2200));
            assert!(!descendant.exists(), "{stage} descendant was reaped");
            assert!(!plugin_marker.exists(), "{stage} plugin was reaped");
            let _ = std::fs::remove_file(ready);
            let _ = std::fs::remove_file(descendant);
        }
    }
}
