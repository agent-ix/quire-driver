// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linux and macOS signal forwarding for active plugin sessions.

use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use quire_exact::{Cancel, CancelCause};

/// Shared cancellation state used by the CLI operation and process adapter.
#[derive(Clone, Default)]
pub(crate) struct SignalContext {
    shared: Arc<Shared>,
}

#[derive(Default)]
struct Shared {
    next_token: AtomicU64,
    active: Mutex<Option<(u64, Cancel)>>,
    received: AtomicI32,
}

fn lock_active(shared: &Shared) -> MutexGuard<'_, Option<(u64, Cancel)>> {
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
        *lock_active(&self.shared) = Some((token, cancel));
        ActiveSession {
            context: self.clone(),
            token,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn observe(&self, signal: i32) {
        let active = lock_active(&self.shared);
        if let Some((_, cancel)) = active.as_ref() {
            cancel.cancel(CancelCause::Requested);
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
        if active
            .as_ref()
            .is_some_and(|(token, _)| *token == self.token)
        {
            *active = None;
        }
    }
}

/// Run the CLI operation while polling handled signals at short intervals.
/// With no active plugin session, the installed signal handler is immediately
/// replaced by the default disposition and re-raised. With an active session,
/// the QSL handle is cancelled and the original signal is re-raised after the
/// operation has completed its bounded plugin cleanup.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) fn run<T: Send + 'static>(
    operation: impl FnOnce(SignalContext) -> T + Send + 'static,
) -> Result<T, std::io::Error> {
    use signal_hook::consts::signal::{SIGINT, SIGTERM};
    use signal_hook::iterator::Signals;
    use std::sync::mpsc;

    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    let context = SignalContext::default();
    let worker_context = context.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("quire-cli-operation".to_owned())
        .spawn(move || {
            let result = operation(worker_context);
            let _ = sender.send(result);
        })?;

    let result = loop {
        for signal in signals.pending() {
            context.observe(signal);
        }
        match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(result) => {
                for signal in signals.pending() {
                    context.observe(signal);
                }
                break result;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(std::io::Error::other("CLI operation thread stopped"));
            }
        }
    };
    context.reraise_received();
    Ok(result)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
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
        assert!(lock_active(&context.shared).is_some());
        drop(guard);
        assert!(lock_active(&context.shared).is_none());
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
        use std::os::unix::process::ExitStatusExt;
        use std::process::{Command, Stdio};
        use std::thread;
        use std::time::{Duration, Instant};

        use quire_exact::{Cancel, CancelCause};
        use signal_hook::consts::signal::SIGTERM;

        if let (Ok(mode), Ok(marker)) = (
            std::env::var("QUIRE_SIGNAL_TEST_MODE"),
            std::env::var("QUIRE_SIGNAL_TEST_MARKER"),
        ) {
            let marker = std::path::PathBuf::from(marker);
            let result = super::run(move |context| {
                if mode == "active" {
                    let cancel = Cancel::new();
                    let _active = context.activate(cancel.clone());
                    std::fs::write(&marker, b"active").expect("write active marker");
                    while cancel.cause().is_none() {
                        thread::sleep(Duration::from_millis(2));
                    }
                    assert_eq!(cancel.cause(), Some(CancelCause::Requested));
                    thread::sleep(Duration::from_millis(30));
                } else {
                    std::fs::write(&marker, b"idle").expect("write idle marker");
                    thread::sleep(Duration::from_secs(30));
                }
                17
            })
            .expect("signal wrapper starts");
            assert_eq!(result, 17, "a received signal must be re-raised");
            return;
        }

        for mode in ["active", "idle"] {
            let marker = std::env::temp_dir()
                .join(format!("quire-cli-signal-{}-{mode}", std::process::id()));
            let _ = std::fs::remove_file(&marker);
            let mut child = Command::new(std::env::current_exe().expect("test executable"))
                .args([
                    "--exact",
                    "signals::tests::signal_is_forwarded_after_active_cleanup_and_defaults_without_a_session",
                ])
                .env("QUIRE_SIGNAL_TEST_MODE", mode)
                .env("QUIRE_SIGNAL_TEST_MARKER", &marker)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn signal fixture process");
            let deadline = Instant::now() + Duration::from_secs(3);
            while !marker.is_file() && Instant::now() < deadline {
                assert!(child.try_wait().expect("poll child").is_none());
                thread::sleep(Duration::from_millis(5));
            }
            assert!(marker.is_file(), "{mode} child became ready");
            let signal = Command::new("/bin/kill")
                .args(["-TERM", &child.id().to_string()])
                .status()
                .expect("send SIGTERM");
            assert!(signal.success());
            let status = child.wait().expect("wait for signal child");
            assert_eq!(status.signal(), Some(SIGTERM), "{mode} child status");
            let _ = std::fs::remove_file(marker);
        }
    }
}
