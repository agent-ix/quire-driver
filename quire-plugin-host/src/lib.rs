// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical framing and wire types for ADR-029 PL-2 through PL-7.
//!
//! The crate owns the process-facing plugin boundary outside the checked core.
//! FR-331 bodies remain caller-typed; this crate does not define a second
//! provider schema or settlement vocabulary.

#[cfg(target_os = "linux")]
use nix::unistd::{sysconf, SysconfVar};
use process_wrap::std::{ChildWrapper, CommandWrap};
#[cfg(target_os = "linux")]
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use quire_canonical::{to_vec, Limits};
use quire_exact::{Cancel, CancelCause};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// The sole protocol identity accepted by ADR-029 PL-4.
pub const PROTOCOL: &str = "quire.plugin-wire/v1";
const WORKER_CLEANUP_LIMIT: Duration = Duration::from_secs(1);

/// Caller-controlled per-frame and per-run byte ceilings from ADR-029 PL-3.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameLimits {
    /// Maximum bytes in one canonical JSON frame, excluding its length word.
    pub max_frame_bytes: u64,
    /// Maximum aggregate length-prefixed bytes in one run.
    pub max_total_bytes: u64,
}

/// A shared aggregate byte budget for both directions of one plugin run.
#[derive(Debug)]
pub struct ByteBudget {
    maximum: u64,
    used: AtomicU64,
}

impl ByteBudget {
    /// Create a run-wide budget with the caller's total-byte ceiling.
    #[must_use]
    pub const fn new(maximum: u64) -> Self {
        Self {
            maximum,
            used: AtomicU64::new(0),
        }
    }

    fn reserve(&self, bytes: u64) -> Result<(), FrameError> {
        let mut used = self.used.load(Ordering::Relaxed);
        loop {
            let next = used.checked_add(bytes).ok_or(FrameError::TotalTooLarge)?;
            if next > self.maximum {
                return Err(FrameError::RunTooLarge {
                    actual: next,
                    maximum: self.maximum,
                });
            }
            match self
                .used
                .compare_exchange_weak(used, next, Ordering::AcqRel, Ordering::Relaxed)
            {
                Ok(_) => return Ok(()),
                Err(observed) => used = observed,
            }
        }
    }

    /// Aggregate length-prefixed bytes reserved by readers and writers.
    #[must_use]
    pub fn used(&self) -> u64 {
        self.used.load(Ordering::Acquire)
    }
}

/// One of the six closed message kinds in ADR-029 PL-4.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Plugin's first message, containing its FR-331 manifest.
    Hello,
    /// Host's FR-331 routed request.
    Request,
    /// Plugin's FR-331 result.
    Result,
    /// Host's request cancellation message.
    Cancel,
    /// Host's empty orderly shutdown message.
    Shutdown,
    /// Plugin's typed failure message.
    Fault,
}

/// The closed PL-4 envelope. `body` is decoded only after protocol and kind
/// checks so callers can deserialize it into the FR-331 type for that kind.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    /// The sole accepted plugin wire identity.
    pub protocol: String,
    /// The host-assigned request or control-message identifier.
    pub id: u64,
    /// The PL-4 message kind.
    pub kind: Kind,
    /// The kind-specific body, kept opaque until validated by the caller.
    pub body: Value,
}

impl Envelope {
    /// Require the exact protocol identity before a body is interpreted.
    pub fn validate_protocol(&self) -> Result<(), FrameError> {
        if self.protocol == PROTOCOL {
            Ok(())
        } else {
            Err(FrameError::WrongProtocol(self.protocol.clone()))
        }
    }

    /// Deserialize a previously protocol-checked body to its kind's typed
    /// representation.
    pub fn decode_body<T: DeserializeOwned>(&self) -> Result<T, FrameError> {
        self.validate_protocol()?;
        Ok(serde_json::from_value(self.body.clone())?)
    }
}

/// A bounded reader for PL-3 length-prefixed JCS frames.
pub struct FrameReader<R> {
    reader: R,
    max_frame_bytes: u64,
    budget: Arc<ByteBudget>,
    consumed: u64,
}

impl<R: Read> FrameReader<R> {
    /// Read frames from `reader` under the caller's limits.
    #[must_use]
    pub fn new(reader: R, limits: FrameLimits) -> Self {
        Self {
            reader,
            max_frame_bytes: limits.max_frame_bytes,
            budget: Arc::new(ByteBudget::new(limits.max_total_bytes)),
            consumed: 0,
        }
    }

    /// Read frames with a budget shared with the run's writer.
    #[must_use]
    pub fn with_budget(reader: R, max_frame_bytes: u64, budget: Arc<ByteBudget>) -> Self {
        Self {
            reader,
            max_frame_bytes,
            budget,
            consumed: 0,
        }
    }

    /// Read and canonically decode the next frame; `None` is clean EOF before
    /// any byte of a new length word.
    pub fn read<T: DeserializeOwned>(&mut self) -> Result<Option<T>, FrameError> {
        let mut prefix = [0_u8; 8];
        let mut read = 0;
        while read < prefix.len() {
            match self.reader.read(&mut prefix[read..]) {
                Ok(0) if read == 0 => return Ok(None),
                Ok(0) => return Err(FrameError::TruncatedLength),
                Ok(count) => read += count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(FrameError::Io(error)),
            }
        }
        let length = u64::from_be_bytes(prefix);
        if length > self.max_frame_bytes {
            return Err(FrameError::FrameTooLarge {
                actual: length,
                maximum: self.max_frame_bytes,
            });
        }
        let frame_total = 8_u64.checked_add(length).ok_or(FrameError::TotalTooLarge)?;
        self.budget.reserve(frame_total)?;
        let consumed = self.consumed.saturating_add(frame_total);
        let length = usize::try_from(length).map_err(|_| FrameError::TotalTooLarge)?;
        let mut bytes = vec![0_u8; length];
        self.reader.read_exact(&mut bytes).map_err(|error| {
            if error.kind() == io::ErrorKind::UnexpectedEof {
                FrameError::TruncatedFrame
            } else {
                FrameError::Io(error)
            }
        })?;
        self.consumed = consumed;
        let value: Value = serde_json::from_slice(&bytes)?;
        let canonical = to_vec(&value, Limits::new(self.max_frame_bytes))?;
        if canonical != bytes {
            return Err(FrameError::NonCanonical);
        }
        Ok(Some(serde_json::from_value(value)?))
    }

    /// Number of length-prefixed bytes consumed so far.
    #[must_use]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }
}

/// A bounded writer for PL-3 length-prefixed JCS frames.
pub struct FrameWriter<W> {
    writer: W,
    max_frame_bytes: u64,
    budget: Arc<ByteBudget>,
    written: u64,
}

struct WriterMessage {
    frame: Envelope,
    completion: mpsc::Sender<Result<(), FrameError>>,
}

fn spawn_writer<W: Write + Send + 'static>(
    writer: W,
    max_frame_bytes: u64,
    budget: Arc<ByteBudget>,
) -> io::Result<(mpsc::Sender<WriterMessage>, thread::JoinHandle<()>)> {
    let (sender, receiver) = mpsc::channel::<WriterMessage>();
    let thread = thread::Builder::new()
        .name("quire-plugin-stdin".to_owned())
        .spawn(move || {
            let mut writer = FrameWriter::with_budget(writer, max_frame_bytes, budget);
            while let Ok(message) = receiver.recv() {
                let result = writer.write(&message.frame);
                let failed = result.is_err();
                let _ = message.completion.send(result);
                if failed {
                    break;
                }
            }
        })?;
    Ok((sender, thread))
}

impl<W: Write> FrameWriter<W> {
    /// Write frames to `writer` under the caller's limits.
    #[must_use]
    pub fn new(writer: W, limits: FrameLimits) -> Self {
        Self {
            writer,
            max_frame_bytes: limits.max_frame_bytes,
            budget: Arc::new(ByteBudget::new(limits.max_total_bytes)),
            written: 0,
        }
    }

    /// Write frames with a budget shared with the run's reader.
    #[must_use]
    pub fn with_budget(writer: W, max_frame_bytes: u64, budget: Arc<ByteBudget>) -> Self {
        Self {
            writer,
            max_frame_bytes,
            budget,
            written: 0,
        }
    }

    /// Canonically encode and write one length-prefixed frame.
    pub fn write<T: Serialize>(&mut self, frame: &T) -> Result<(), FrameError> {
        let value = serde_json::to_value(frame)?;
        let bytes = to_vec(&value, Limits::new(self.max_frame_bytes))?;
        let length = u64::try_from(bytes.len()).map_err(|_| FrameError::TotalTooLarge)?;
        if length > self.max_frame_bytes {
            return Err(FrameError::FrameTooLarge {
                actual: length,
                maximum: self.max_frame_bytes,
            });
        }
        let frame_total = 8_u64.checked_add(length).ok_or(FrameError::TotalTooLarge)?;
        self.budget.reserve(frame_total)?;
        let written = self.written.saturating_add(frame_total);
        self.writer.write_all(&length.to_be_bytes())?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;
        self.written = written;
        Ok(())
    }

    /// Number of length-prefixed bytes written so far.
    #[must_use]
    pub const fn written(&self) -> u64 {
        self.written
    }
}

/// Caller budgets for one child-process run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RunLimits {
    /// Per-frame and aggregate stdin/stdout byte limits.
    pub frames: FrameLimits,
    /// Maximum standard-error bytes retained for display.
    pub max_stderr_bytes: u64,
    /// Grace period for a plugin to settle open items after cancellation.
    pub cancellation_grace: Duration,
    /// Maximum observed resident memory in bytes, where the platform reports it.
    pub max_process_memory_bytes: Option<u64>,
    /// Maximum observed accumulated CPU time, where the platform reports it.
    pub max_process_cpu_time: Option<Duration>,
}

/// Inputs to a hello-first plugin session. No provider request is required to
/// start the process or admit its manifest.
pub struct SessionStart<'a> {
    /// The exact executable path selected by the caller.
    pub executable: &'a Path,
    /// Caller-supplied argument vector, passed without shell interpretation.
    pub arguments: &'a [std::ffi::OsString],
    /// Session, frame, cancellation and process-tree budgets.
    pub limits: RunLimits,
    /// Caller-owned cancellation handle used while awaiting the hello.
    pub cancel: &'a Cancel,
}

/// A completed plugin session, including the admitted manifest and bounded
/// process diagnostics.
#[derive(Debug)]
pub struct SessionFinished<M> {
    /// The manifest admitted before any request was sent.
    pub manifest: M,
    /// Bounded opaque stderr captured for the full session.
    pub stderr: Vec<u8>,
    /// Whether stderr exceeded the configured retention limit.
    pub stderr_truncated: bool,
}

/// A hello-admitted process session that accepts zero or more sequential
/// provider requests before shutdown (QSL ADR-029 PL-6).
pub struct PluginSession<M, F, A> {
    child: Option<Box<dyn ChildWrapper>>,
    manifest: Option<M>,
    writer: Option<mpsc::Sender<WriterMessage>>,
    writer_thread: Option<thread::JoinHandle<()>>,
    receiver: Receiver<Result<Option<Envelope>, FrameError>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<io::Result<CapturedStderr>>>,
    monitor: ProcessMonitor,
    limits: RunLimits,
    next_id: u64,
    #[cfg(test)]
    request_write_started: bool,
    _types: std::marker::PhantomData<(F, A)>,
}

impl<M, F, A> PluginSession<M, F, A>
where
    M: DeserializeOwned,
    F: DeserializeOwned,
{
    /// Start an explicit-path plugin, validate and admit its first hello, and
    /// return a session ready for zero or more requests.
    pub fn start(
        start: SessionStart<'_>,
        admit_manifest: impl FnOnce(&M) -> Result<(), A>,
    ) -> Result<Self, PluginError<A, std::convert::Infallible, F>> {
        if !has_directory_component(start.executable) || is_shell_script(start.executable) {
            if is_shell_script(start.executable) {
                return Err(PluginError::ShellScriptExecutable(
                    start.executable.to_path_buf(),
                ));
            }
            return Err(PluginError::BareExecutableName(
                start.executable.to_path_buf(),
            ));
        }
        if let Some(cause) = start.cancel.cause() {
            return Err(cancel_error(cause));
        }
        let mut command = Command::new(start.executable);
        command
            .args(start.arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(process_wrap::std::ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(process_wrap::std::JobObject);
        let mut child = command
            .spawn()
            .map_err(|error| PluginError::from_spawn(start.executable, error))?;
        let mut monitor = ProcessMonitor::new(
            child.as_ref(),
            start.limits.max_process_memory_bytes,
            start.limits.max_process_cpu_time,
        );
        let Some(stdin) = child.stdin().take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PluginError::MissingPipe("stdin"));
        };
        let Some(stdout) = child.stdout().take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PluginError::MissingPipe("stdout"));
        };
        let Some(stderr) = child.stderr().take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PluginError::MissingPipe("stderr"));
        };

        let byte_budget = Arc::new(ByteBudget::new(start.limits.frames.max_total_bytes));
        let (writer, writer_thread) = match spawn_writer(
            stdin,
            start.limits.frames.max_frame_bytes,
            Arc::clone(&byte_budget),
        ) {
            Ok(workers) => workers,
            Err(error) => {
                monitor.terminate_tree(child.as_mut());
                return Err(PluginError::WorkerSpawn {
                    worker: "stdin writer",
                    source: error,
                });
            }
        };
        let (sender, receiver) = mpsc::channel::<Result<Option<Envelope>, FrameError>>();
        let reader_thread = match thread::Builder::new()
            .name("quire-plugin-stdout".to_owned())
            .spawn(move || {
                let mut reader = FrameReader::with_budget(
                    stdout,
                    start.limits.frames.max_frame_bytes,
                    byte_budget,
                );
                loop {
                    let event = match reader.read::<Envelope>() {
                        Ok(Some(frame)) => Some(Ok(Some(frame))),
                        Ok(None) => Some(Ok(None)),
                        Err(error) => Some(Err(error)),
                    };
                    let Some(event) = event else { break };
                    let terminal = !matches!(event, Ok(Some(_)));
                    if sender.send(event).is_err() || terminal {
                        break;
                    }
                }
            }) {
            Ok(handle) => handle,
            Err(error) => {
                drop(writer);
                monitor.terminate_tree(child.as_mut());
                if join_thread_within(&writer_thread, WORKER_CLEANUP_LIMIT) {
                    let _ = writer_thread.join();
                }
                return Err(PluginError::WorkerSpawn {
                    worker: "stdout reader",
                    source: error,
                });
            }
        };
        let stderr_limit = start.limits.max_stderr_bytes;
        let stderr_thread = match thread::Builder::new()
            .name("quire-plugin-stderr".to_owned())
            .spawn(move || capture_stderr(stderr, stderr_limit))
        {
            Ok(handle) => handle,
            Err(error) => {
                drop(writer);
                monitor.terminate_tree(child.as_mut());
                if join_thread_within(&reader_thread, WORKER_CLEANUP_LIMIT) {
                    let _ = reader_thread.join();
                }
                if join_thread_within(&writer_thread, WORKER_CLEANUP_LIMIT) {
                    let _ = writer_thread.join();
                }
                return Err(PluginError::WorkerSpawn {
                    worker: "stderr capture",
                    source: error,
                });
            }
        };
        let mut session = Self {
            child: Some(child),
            manifest: None,
            writer: Some(writer),
            writer_thread: Some(writer_thread),
            receiver,
            reader_thread: Some(reader_thread),
            stderr_thread: Some(stderr_thread),
            monitor,
            limits: start.limits,
            next_id: 1,
            #[cfg(test)]
            request_write_started: false,
            _types: std::marker::PhantomData,
        };

        let hello = (|| {
            let child = session
                .child
                .as_mut()
                .ok_or(PluginError::MissingPipe("child"))?;
            let envelope = next_frame(
                &session.receiver,
                child.as_mut(),
                start.cancel,
                &mut session.monitor,
                session.limits.cancellation_grace,
            )?;
            if envelope.kind == Kind::Fault {
                return Err(PluginError::Fault(envelope.decode_body::<F>()?));
            }
            expect_kind(&envelope, Kind::Hello)?;
            let manifest = envelope.decode_body::<M>()?;
            admit_manifest(&manifest).map_err(PluginError::ManifestAdmission)?;
            Ok(manifest)
        })();
        match hello {
            Ok(manifest) => {
                session.manifest = Some(manifest);
                Ok(session)
            }
            Err(cause) => Err(session.abort(cause)),
        }
    }

    /// The manifest admitted during `start`.
    #[must_use]
    pub fn manifest(&self) -> Option<&M> {
        self.manifest.as_ref()
    }

    /// Send one request on the admitted process session and return its
    /// validated result. The same session accepts subsequent requests.
    pub fn request<Q, R, V>(
        &mut self,
        body: &Q,
        cancel: &Cancel,
        validate_result: impl FnOnce(&M, &R) -> Result<(), V>,
    ) -> Result<R, PluginError<A, V, F>>
    where
        Q: Serialize,
        R: DeserializeOwned,
    {
        if self.child.is_none() {
            return Err(PluginError::SessionClosed);
        }
        let result = (|| {
            if let Some(cause) = cancel.cause() {
                return Err(cancel_error(cause));
            }
            self.monitor
                .begin_request()
                .map_err(PluginError::ProcessBudget)?;
            let request_id = self.next_id;
            self.next_id = self
                .next_id
                .checked_add(1)
                .ok_or(PluginError::RequestIdsExhausted)?;
            let request_body = serde_json::to_value(body).map_err(PluginError::Json)?;
            let writer = self.writer.as_ref().ok_or(PluginError::WriterThread)?;
            let child = self
                .child
                .as_mut()
                .ok_or(PluginError::MissingPipe("child"))?;
            #[cfg(test)]
            {
                self.request_write_started = true;
            }
            write_frame_cancellable(
                writer,
                Envelope {
                    protocol: PROTOCOL.to_owned(),
                    id: request_id,
                    kind: Kind::Request,
                    body: request_body,
                },
                cancel,
                child.as_mut(),
                &mut self.monitor,
            )?;
            let frame = match next_frame(
                &self.receiver,
                self.child
                    .as_mut()
                    .map(Box::as_mut)
                    .ok_or(PluginError::MissingPipe("child"))?,
                cancel,
                &mut self.monitor,
                self.limits.cancellation_grace,
            ) {
                Ok(frame) => frame,
                Err(PluginError::Cancelled) => {
                    let writer = self.writer.as_ref().ok_or(PluginError::WriterThread)?;
                    write_frame_control(
                        writer,
                        Envelope {
                            protocol: PROTOCOL.to_owned(),
                            id: request_id,
                            kind: Kind::Cancel,
                            body: serde_json::json!({"id": request_id}),
                        },
                        self.limits.cancellation_grace,
                    )?;
                    let frame = match next_frame_with_grace::<A, V, F>(
                        &self.receiver,
                        self.child
                            .as_mut()
                            .map(Box::as_mut)
                            .ok_or(PluginError::MissingPipe("child"))?,
                        self.limits.cancellation_grace,
                        &mut self.monitor,
                    ) {
                        Ok(frame) => frame,
                        Err(PluginError::GraceExpired) => {
                            return Err(cancel_error(
                                cancel.cause().unwrap_or(CancelCause::Requested),
                            ));
                        }
                        Err(error) => return Err(error),
                    };
                    frame
                }
                Err(error) => return Err(error),
            };
            expect_request_id(&frame, request_id)?;
            if frame.kind == Kind::Fault {
                return Err(PluginError::Fault(frame.decode_body::<F>()?));
            }
            expect_kind(&frame, Kind::Result)?;
            let typed = frame.decode_body::<R>()?;
            let manifest = self.manifest.as_ref().ok_or(PluginError::MissingManifest)?;
            validate_result(manifest, &typed).map_err(PluginError::ResultValidation)?;
            let child = self
                .child
                .as_mut()
                .map(Box::as_mut)
                .ok_or(PluginError::MissingPipe("child"))?;
            if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                if !status.success() {
                    return Err(PluginError::Exited(status));
                }
            } else {
                self.monitor.check().map_err(PluginError::ProcessBudget)?;
            }
            Ok(typed)
        })();
        match result {
            Ok(result) => Ok(result),
            Err(cause) => Err(self.abort(cause)),
        }
    }

    /// Send shutdown, wait for the process to exit, and return session
    /// diagnostics. A session with zero requests may be shut down normally.
    pub fn shutdown(
        mut self,
    ) -> Result<SessionFinished<M>, PluginError<A, std::convert::Infallible, F>> {
        let result = (|| {
            let id = self.next_id.saturating_sub(1);
            let writer = self.writer.as_ref().ok_or(PluginError::WriterThread)?;
            send_shutdown(writer, id, self.limits.cancellation_grace)?;
            self.writer.take();
            let writer_thread = self.writer_thread.take().ok_or(PluginError::WriterThread)?;
            if !join_thread_within(&writer_thread, self.limits.cancellation_grace) {
                return Err(PluginError::WorkerJoinTimeout("writer"));
            }
            writer_thread
                .join()
                .map_err(|_| PluginError::WriterThread)?;
            let child = self
                .child
                .as_mut()
                .map(Box::as_mut)
                .ok_or(PluginError::MissingPipe("child"))?;
            wait_for_exit(
                &self.receiver,
                child,
                &mut self.monitor,
                self.limits.cancellation_grace,
            )?;
            let (stderr, stderr_truncated) = join_stderr(
                self.stderr_thread
                    .take()
                    .ok_or(PluginError::CaptureThread)?,
                self.limits.cancellation_grace,
            )?;
            join_reader(
                self.reader_thread.take().ok_or(PluginError::ReaderThread)?,
                self.limits.cancellation_grace,
            )?;
            if let Some(child) = self.child.as_mut() {
                if !self.monitor.terminate_tree(child.as_mut()) {
                    return Err(PluginError::ProcessTreeCleanupTimeout);
                }
            }
            self.child.take();
            Ok(SessionFinished {
                manifest: self.manifest.take().ok_or(PluginError::MissingManifest)?,
                stderr,
                stderr_truncated,
            })
        })();
        match result {
            Ok(finished) => Ok(finished),
            Err(cause) => Err(self.abort(cause)),
        }
    }

    fn abort<V>(&mut self, cause: PluginError<A, V, F>) -> PluginError<A, V, F> {
        let cleanup_until = Instant::now()
            .checked_add(WORKER_CLEANUP_LIMIT)
            .unwrap_or_else(Instant::now);
        let process_tree_cleanup_failed = self
            .child
            .as_mut()
            .is_some_and(|child| !self.monitor.terminate_tree(child.as_mut()));
        self.child.take();
        self.writer.take();
        let (stderr, stderr_truncated, capture_error) = match self.stderr_thread.take() {
            Some(handle) if join_thread_until(&handle, cleanup_until) => match handle.join() {
                Ok(Ok(captured)) => (captured.bytes, captured.truncated, None),
                Ok(Err(error)) => (Vec::new(), false, Some(error.to_string())),
                Err(_) => (Vec::new(), false, Some("stderr worker panicked".to_owned())),
            },
            Some(_) => (
                Vec::new(),
                false,
                Some("stderr worker did not stop within the one-second cleanup limit".to_owned()),
            ),
            None => (Vec::new(), false, None),
        };
        let reader_join_failed = self.reader_thread.take().is_some_and(|handle| {
            !join_thread_until(&handle, cleanup_until) || handle.join().is_err()
        });
        let writer_join_failed = self.writer_thread.take().is_some_and(|handle| {
            !join_thread_until(&handle, cleanup_until) || handle.join().is_err()
        });
        PluginError::WithDiagnostics {
            cause: Box::new(cause),
            stderr,
            stderr_truncated,
            capture_error,
            process_tree_cleanup_failed,
            reader_join_failed,
            writer_join_failed,
        }
    }
}

impl<M, F, A> Drop for PluginSession<M, F, A> {
    fn drop(&mut self) {
        let cleanup_until = Instant::now()
            .checked_add(WORKER_CLEANUP_LIMIT)
            .unwrap_or_else(Instant::now);
        if let Some(child) = self.child.as_mut() {
            let _ = self.monitor.terminate_tree(child.as_mut());
        }
        self.writer.take();
        if let Some(handle) = self.writer_thread.take() {
            if join_thread_until(&handle, cleanup_until) {
                let _ = handle.join();
            }
        }
        if let Some(handle) = self.reader_thread.take() {
            if join_thread_until(&handle, cleanup_until) {
                let _ = handle.join();
            }
        }
        if let Some(handle) = self.stderr_thread.take() {
            if join_thread_until(&handle, cleanup_until) {
                let _ = handle.join();
            }
        }
    }
}

fn next_frame<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut dyn ChildWrapper,
    cancel: &Cancel,
    monitor: &mut ProcessMonitor,
    eof_grace: Duration,
) -> Result<Envelope, PluginError<A, V, F>> {
    loop {
        if let Some(cause) = cancel.cause() {
            return Err(cancel_error(cause));
        }
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(Ok(Some(frame))) => {
                frame.validate_protocol()?;
                if frame.kind != Kind::Result {
                    monitor.check().map_err(PluginError::ProcessBudget)?;
                }
                return Ok(frame);
            }
            Ok(Ok(None)) => return Err(eof_error(child, eof_grace)?),
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Disconnected) => return Err(eof_error(child, eof_grace)?),
            Err(RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                    return Err(PluginError::Exited(status));
                }
                monitor.check().map_err(PluginError::ProcessBudget)?;
            }
        }
    }
}

fn next_frame_with_grace<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut dyn ChildWrapper,
    grace: Duration,
    monitor: &mut ProcessMonitor,
) -> Result<Envelope, PluginError<A, V, F>> {
    let Some(until) = Instant::now().checked_add(grace) else {
        return Err(PluginError::GraceExpired);
    };
    loop {
        let Some(remaining) = until.checked_duration_since(Instant::now()) else {
            return Err(PluginError::GraceExpired);
        };
        match receiver.recv_timeout(remaining.min(Duration::from_millis(20))) {
            Ok(Ok(Some(frame))) => {
                frame.validate_protocol()?;
                if frame.kind != Kind::Result {
                    monitor.check().map_err(PluginError::ProcessBudget)?;
                }
                return Ok(frame);
            }
            Ok(Ok(None)) | Err(RecvTimeoutError::Disconnected) => {
                return Err(PluginError::MissingFrame);
            }
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                    return Err(PluginError::Exited(status));
                }
                monitor.check().map_err(PluginError::ProcessBudget)?;
            }
        }
    }
}

fn write_frame_cancellable<A, V, F>(
    writer: &mpsc::Sender<WriterMessage>,
    frame: Envelope,
    cancel: &Cancel,
    child: &mut dyn ChildWrapper,
    monitor: &mut ProcessMonitor,
) -> Result<(), PluginError<A, V, F>> {
    let (completion, receiver) = mpsc::channel();
    writer
        .send(WriterMessage { frame, completion })
        .map_err(|_| PluginError::WriterThread)?;
    loop {
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Disconnected) => return Err(PluginError::WriterThread),
            Err(RecvTimeoutError::Timeout) => {
                if let Some(result) = completed_write_wins_cancellation(&receiver, cancel) {
                    return result;
                }
                if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                    return Err(PluginError::Exited(status));
                }
                monitor.check().map_err(PluginError::ProcessBudget)?;
            }
        }
    }
}

fn write_frame_control<A, V, F>(
    writer: &mpsc::Sender<WriterMessage>,
    frame: Envelope,
    grace: Duration,
) -> Result<(), PluginError<A, V, F>> {
    let (completion, receiver) = mpsc::channel();
    writer
        .send(WriterMessage { frame, completion })
        .map_err(|_| PluginError::WriterThread)?;
    match receiver.recv_timeout(grace) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(PluginError::Frame(error)),
        Err(RecvTimeoutError::Timeout) => Err(PluginError::WriterUnresponsive),
        Err(RecvTimeoutError::Disconnected) => Err(PluginError::WriterThread),
    }
}

fn completed_write_wins_cancellation<A, V, F>(
    receiver: &Receiver<Result<(), FrameError>>,
    cancel: &Cancel,
) -> Option<Result<(), PluginError<A, V, F>>> {
    let cause = cancel.cause()?;
    Some(match receiver.try_recv() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(PluginError::Frame(error)),
        Err(mpsc::TryRecvError::Empty) => Err(cancel_error(cause)),
        Err(mpsc::TryRecvError::Disconnected) => Err(PluginError::WriterThread),
    })
}

fn join_thread_within(handle: &thread::JoinHandle<impl Sized>, timeout: Duration) -> bool {
    let Some(until) = Instant::now().checked_add(timeout) else {
        return false;
    };
    join_thread_until(handle, until)
}

fn join_thread_until(handle: &thread::JoinHandle<impl Sized>, until: Instant) -> bool {
    while !handle.is_finished() {
        let now = Instant::now();
        if now >= until {
            return false;
        }
        thread::sleep(Duration::from_millis(5).min(until.saturating_duration_since(now)));
    }
    true
}

fn expect_kind<A, V, F>(envelope: &Envelope, expected: Kind) -> Result<(), PluginError<A, V, F>> {
    if envelope.kind == expected {
        Ok(())
    } else {
        Err(PluginError::UnexpectedKind {
            expected,
            actual: envelope.kind,
        })
    }
}

fn expect_request_id<A, V, F>(
    envelope: &Envelope,
    expected: u64,
) -> Result<(), PluginError<A, V, F>> {
    if envelope.id == expected {
        Ok(())
    } else {
        Err(PluginError::UnexpectedId {
            expected,
            actual: envelope.id,
        })
    }
}

fn send_shutdown<A, V, F>(
    writer: &mpsc::Sender<WriterMessage>,
    id: u64,
    grace: Duration,
) -> Result<(), PluginError<A, V, F>> {
    write_frame_control(
        writer,
        Envelope {
            protocol: PROTOCOL.to_owned(),
            id,
            kind: Kind::Shutdown,
            body: serde_json::json!({}),
        },
        grace,
    )?;
    Ok(())
}

fn wait_for_exit<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut dyn ChildWrapper,
    monitor: &mut ProcessMonitor,
    shutdown_grace: Duration,
) -> Result<ExitStatus, PluginError<A, V, F>> {
    let mut status = None;
    let mut stdout_closed = false;
    let Some(until) = Instant::now().checked_add(shutdown_grace) else {
        return Err(PluginError::ShutdownTimeout);
    };
    loop {
        if Instant::now() >= until {
            return Err(PluginError::ShutdownTimeout);
        }
        if status.is_none() {
            status = child.try_wait().map_err(PluginError::Wait)?;
        }
        if stdout_closed {
            if let Some(status) = status {
                return if status.success() {
                    Ok(status)
                } else {
                    Err(PluginError::Exited(status))
                };
            }
            thread::sleep(Duration::from_millis(20));
            status = child.try_wait().map_err(PluginError::Wait)?;
            if let Some(status) = status {
                return if status.success() {
                    Ok(status)
                } else {
                    Err(PluginError::Exited(status))
                };
            }
            monitor.check().map_err(PluginError::ProcessBudget)?;
            continue;
        }
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(Ok(Some(_))) => return Err(PluginError::UnexpectedFrame),
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Ok(Ok(None)) | Err(RecvTimeoutError::Disconnected) => stdout_closed = true,
            Err(RecvTimeoutError::Timeout) => {
                monitor.check().map_err(PluginError::ProcessBudget)?;
            }
        }
    }
}

fn has_directory_component(path: &Path) -> bool {
    path.components().any(|component| {
        matches!(
            component,
            Component::CurDir | Component::ParentDir | Component::RootDir
        )
    }) || path.components().count() > 1
}

fn is_shell_script(path: &Path) -> bool {
    #[cfg(windows)]
    {
        path.extension().is_some_and(|extension| {
            extension.eq_ignore_ascii_case("bat") || extension.eq_ignore_ascii_case("cmd")
        })
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}

fn eof_error<A, V, F>(
    child: &mut dyn ChildWrapper,
    grace: Duration,
) -> Result<PluginError<A, V, F>, PluginError<A, V, F>> {
    let Some(until) = Instant::now().checked_add(grace) else {
        return Ok(PluginError::MissingFrame);
    };
    loop {
        match child.try_wait().map_err(PluginError::Wait)? {
            Some(status) => return Ok(PluginError::Exited(status)),
            None if Instant::now() < until => thread::sleep(Duration::from_millis(1)),
            None => return Ok(PluginError::MissingFrame),
        }
    }
}

struct ProcessMonitor {
    #[cfg(unix)]
    group_id: Option<u32>,
    memory_limit: Option<u64>,
    cpu_time_limit: Option<Duration>,
    #[cfg(target_os = "linux")]
    last_cpu_ticks: HashMap<(u32, u64), u64>,
    #[cfg(target_os = "linux")]
    request_cpu_ticks: u64,
    request_active: bool,
    #[cfg(test)]
    force_unavailable: bool,
}

impl ProcessMonitor {
    fn new(
        _child: &dyn ChildWrapper,
        memory_limit: Option<u64>,
        cpu_time_limit: Option<Duration>,
    ) -> Self {
        Self {
            #[cfg(unix)]
            group_id: Some(_child.id()),
            memory_limit,
            cpu_time_limit,
            #[cfg(target_os = "linux")]
            last_cpu_ticks: HashMap::new(),
            #[cfg(target_os = "linux")]
            request_cpu_ticks: 0,
            request_active: false,
            #[cfg(test)]
            force_unavailable: false,
        }
    }

    fn begin_request(&mut self) -> Result<(), ProcessBudgetError> {
        if self.memory_limit.is_none() && self.cpu_time_limit.is_none() {
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        {
            self.last_cpu_ticks.clear();
            self.request_cpu_ticks = 0;
            let samples = self.refresh_tree()?;
            for sample in samples {
                self.last_cpu_ticks
                    .insert((sample.pid, sample.start_ticks), sample.cpu_ticks);
            }
            self.request_active = true;
            self.check()
        }
        #[cfg(not(target_os = "linux"))]
        Err(ProcessBudgetError::Unavailable)
    }

    fn check(&mut self) -> Result<(), ProcessBudgetError> {
        if (self.memory_limit.is_none() && self.cpu_time_limit.is_none()) || !self.request_active {
            return Ok(());
        }
        #[cfg(target_os = "linux")]
        {
            let samples = self.refresh_tree()?;
            let mut resident_bytes = 0_u64;
            for sample in samples {
                resident_bytes = resident_bytes.saturating_add(sample.memory_bytes);
                let identity = (sample.pid, sample.start_ticks);
                let previous = self
                    .last_cpu_ticks
                    .insert(identity, sample.cpu_ticks)
                    .unwrap_or(0);
                self.request_cpu_ticks = self
                    .request_cpu_ticks
                    .saturating_add(sample.cpu_ticks.saturating_sub(previous));
            }
            if self
                .memory_limit
                .is_some_and(|limit| resident_bytes > limit)
            {
                return Err(ProcessBudgetError::MemoryExceeded);
            }
            if let Some(limit) = self.cpu_time_limit {
                if cpu_ticks_exceed(self.request_cpu_ticks, limit)
                    .ok_or(ProcessBudgetError::Unavailable)?
                {
                    return Err(ProcessBudgetError::CpuTimeExceeded);
                }
            }
            Ok(())
        }
        #[cfg(not(target_os = "linux"))]
        Err(ProcessBudgetError::Unavailable)
    }

    #[cfg(target_os = "linux")]
    fn refresh_tree(&self) -> Result<Vec<LinuxProcessSample>, ProcessBudgetError> {
        if {
            #[cfg(test)]
            {
                self.force_unavailable
            }
            #[cfg(not(test))]
            {
                false
            }
        } {
            return Err(ProcessBudgetError::Unavailable);
        }
        let group_id = self.group_id.ok_or(ProcessBudgetError::Unavailable)?;
        let members =
            linux_process_group_samples(group_id).ok_or(ProcessBudgetError::Unavailable)?;
        if members.is_empty() {
            return Err(ProcessBudgetError::Unavailable);
        }
        Ok(members)
    }

    fn terminate_tree(&mut self, child: &mut dyn ChildWrapper) -> bool {
        // Unix descendants that deliberately leave this process group (for
        // example with setsid) are outside this host's cleanup boundary.
        let Some(until) = Instant::now().checked_add(WORKER_CLEANUP_LIMIT) else {
            return false;
        };
        #[cfg(unix)]
        {
            if self.group_id.is_none() {
                let _ = child.try_wait();
                return false;
            }
            loop {
                let _ = child.try_wait();
                match child.start_kill() {
                    Ok(()) => {}
                    Err(error) if error.raw_os_error() == Some(nix::errno::Errno::ESRCH as i32) => {
                        return true;
                    }
                    Err(_) => return false,
                }
                #[cfg(target_os = "linux")]
                if self
                    .group_id
                    .and_then(linux_process_group_samples)
                    .is_some_and(|members| {
                        members
                            .iter()
                            .all(|member| matches!(member.state, 'Z' | 'X'))
                    })
                {
                    return true;
                }
                if Instant::now() >= until {
                    return false;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        #[cfg(windows)]
        {
            if child.start_kill().is_err() {
                return false;
            }
            loop {
                if child.try_wait().is_ok_and(|status| status.is_some()) {
                    return true;
                }
                if Instant::now() >= until {
                    return false;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = child;
            false
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug)]
struct LinuxProcessSample {
    pid: u32,
    start_ticks: u64,
    cpu_ticks: u64,
    memory_bytes: u64,
    state: char,
}

#[cfg(target_os = "linux")]
fn linux_process_group_samples(group_id: u32) -> Option<Vec<LinuxProcessSample>> {
    let ticks_per_second = u64::try_from(sysconf(SysconfVar::CLK_TCK).ok()??).ok()?;
    let page_size = u64::try_from(sysconf(SysconfVar::PAGE_SIZE).ok()??).ok()?;
    if ticks_per_second == 0 || page_size == 0 {
        return None;
    }
    let mut members = Vec::new();
    for entry in std::fs::read_dir("/proc").ok()? {
        let Ok(entry) = entry else { continue };
        let pid = match entry.file_name().to_str()?.parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };
        let stat = match std::fs::read_to_string(entry.path().join("stat")) {
            Ok(stat) => stat,
            Err(_) => continue,
        };
        let Some(close) = stat.rfind(')') else {
            continue;
        };
        let Some(fields) = stat.get(close + 1..) else {
            continue;
        };
        let fields: Vec<&str> = fields.split_ascii_whitespace().collect();
        // After the parenthesized command name, stat fields start at state:
        // state, ppid, pgrp, ..., utime, stime, ..., starttime, ...
        let Some(state) = fields.get(0).and_then(|value| value.chars().next()) else {
            continue;
        };
        let Some(process_group) = fields.get(2).and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        if process_group == group_id {
            let Ok(statm) = std::fs::read_to_string(entry.path().join("statm")) else {
                continue;
            };
            let statm_fields: Vec<&str> = statm.split_ascii_whitespace().collect();
            let Some(resident_pages) = statm_fields
                .get(1)
                .and_then(|value| value.parse::<u64>().ok())
            else {
                continue;
            };
            let Some(user_ticks) = fields.get(11).and_then(|value| value.parse::<u64>().ok())
            else {
                continue;
            };
            let Some(system_ticks) = fields.get(12).and_then(|value| value.parse::<u64>().ok())
            else {
                continue;
            };
            let Some(start_ticks) = fields.get(19).and_then(|value| value.parse::<u64>().ok())
            else {
                continue;
            };
            members.push(LinuxProcessSample {
                pid,
                start_ticks,
                cpu_ticks: user_ticks.saturating_add(system_ticks),
                memory_bytes: resident_pages.saturating_mul(page_size),
                state,
            });
        }
    }
    Some(members)
}

#[cfg(target_os = "linux")]
fn cpu_ticks_exceed(ticks: u64, limit: Duration) -> Option<bool> {
    let clock_ticks = sysconf(SysconfVar::CLK_TCK).ok()??;
    let ticks_per_second = u64::try_from(clock_ticks).ok()?;
    (ticks_per_second > 0).then(|| {
        u128::from(ticks) * 1_000_000_000
            > limit
                .as_nanos()
                .saturating_mul(u128::from(ticks_per_second))
    })
}

/// Why the host could not continue enforcing the requested process budget.
#[derive(Debug, Error)]
pub enum ProcessBudgetError {
    /// The child exceeded its observed resident-memory limit.
    #[error("plugin process tree exceeded its memory limit")]
    MemoryExceeded,
    /// The child exceeded its observed accumulated CPU-time limit.
    #[error("plugin process tree exceeded its CPU-time limit")]
    CpuTimeExceeded,
    /// The platform could not provide process measurements.
    #[error("the complete plugin process tree cannot be measured on this platform")]
    Unavailable,
}

struct CapturedStderr {
    bytes: Vec<u8>,
    truncated: bool,
}

fn capture_stderr(mut stderr: impl Read, maximum: u64) -> io::Result<CapturedStderr> {
    let maximum = usize::try_from(maximum).unwrap_or(usize::MAX);
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 4096];
    let mut truncated = false;
    loop {
        let count = match stderr.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let retained = maximum.saturating_sub(bytes.len()).min(count);
        bytes.extend_from_slice(&buffer[..retained]);
        truncated |= retained < count;
    }
    Ok(CapturedStderr { bytes, truncated })
}

fn join_stderr<A, V, F>(
    handle: thread::JoinHandle<io::Result<CapturedStderr>>,
    timeout: Duration,
) -> Result<(Vec<u8>, bool), PluginError<A, V, F>> {
    if !join_thread_within(&handle, timeout.min(WORKER_CLEANUP_LIMIT)) {
        return Err(PluginError::WorkerJoinTimeout("stderr"));
    }
    let captured = handle
        .join()
        .map_err(|_| PluginError::CaptureThread)?
        .map_err(PluginError::Stderr)?;
    Ok((captured.bytes, captured.truncated))
}

fn join_reader<A, V, F>(
    handle: thread::JoinHandle<()>,
    timeout: Duration,
) -> Result<(), PluginError<A, V, F>> {
    if !join_thread_within(&handle, timeout.min(WORKER_CLEANUP_LIMIT)) {
        return Err(PluginError::WorkerJoinTimeout("stdout reader"));
    }
    handle.join().map_err(|_| PluginError::ReaderThread)
}

fn cancel_error<A, V, F>(cause: CancelCause) -> PluginError<A, V, F> {
    match cause {
        CancelCause::Requested => PluginError::Cancelled,
        CancelCause::Deadline => PluginError::TimedOut,
    }
}

/// Errors from the plugin process boundary. The caller keeps admission and
/// typed FR-331 result validation errors in their original error types.
#[derive(Debug, thiserror::Error)]
pub enum PluginError<A, V, F> {
    /// The original typed failure plus bounded diagnostics captured while
    /// terminating and joining the plugin process workers.
    #[error("{cause}")]
    WithDiagnostics {
        /// The original protocol, process, validation or cancellation failure.
        cause: Box<PluginError<A, V, F>>,
        /// Bounded opaque stderr captured before the error was returned.
        stderr: Vec<u8>,
        /// Whether stderr bytes beyond the configured retention limit existed.
        stderr_truncated: bool,
        /// A stderr capture failure, if one occurred while preserving the cause.
        capture_error: Option<String>,
        /// Whether a process-group or Job Object member missed the cleanup bound.
        process_tree_cleanup_failed: bool,
        /// Whether the stdout reader worker failed to join.
        reader_join_failed: bool,
        /// Whether the stdin writer worker failed to join.
        writer_join_failed: bool,
    },
    /// The requested executable had no directory component and was refused
    /// instead of being looked up through `PATH`.
    #[error("plugin executable {0:?} must be an explicit path with a directory component")]
    BareExecutableName(PathBuf),
    /// Windows batch scripts would invoke a command shell.
    #[error("plugin executable {0:?} cannot be a Windows batch script")]
    ShellScriptExecutable(PathBuf),
    /// The session did not contain a hello-admitted manifest.
    #[error("plugin session has no admitted manifest")]
    MissingManifest,
    /// Starting the caller-named executable failed.
    #[error("could not start plugin {path:?}: {source}")]
    Spawn {
        /// The executable path selected by the caller.
        path: PathBuf,
        /// The operating-system error.
        #[source]
        source: io::Error,
    },
    /// A child-process pipe was unavailable.
    #[error("plugin child did not expose {0}")]
    MissingPipe(&'static str),
    /// The session was already terminated after an earlier failure.
    #[error("the plugin session is closed")]
    SessionClosed,
    /// A pipe read or write failed.
    #[error(transparent)]
    Frame(#[from] FrameError),
    /// Child process status inspection failed.
    #[error("plugin process wait failed: {0}")]
    Wait(#[source] io::Error),
    /// The plugin closed stdout without the expected frame.
    #[error("plugin closed stdout before the expected message")]
    MissingFrame,
    /// The plugin exited unsuccessfully or before the protocol completed.
    #[error("plugin exited with status {0}")]
    Exited(ExitStatus),
    /// A message kind differed from the required lifecycle message.
    #[error("expected plugin message {expected:?}, got {actual:?}")]
    UnexpectedKind {
        /// The message kind required at this protocol step.
        expected: Kind,
        /// The kind received from the plugin.
        actual: Kind,
    },
    /// A plugin reply named a request id the host did not assign.
    #[error("expected plugin reply id {expected}, got {actual}")]
    UnexpectedId {
        /// The request id assigned by the host.
        expected: u64,
        /// The id included in the plugin reply.
        actual: u64,
    },
    /// The session exhausted its monotonically increasing request ids.
    #[error("plugin session exhausted request identifiers")]
    RequestIdsExhausted,
    /// A plugin sent a second frame after its result and before exit.
    #[error("plugin sent an unexpected frame after its terminal result")]
    UnexpectedFrame,
    /// Caller cancellation reached its grace-period end without a result.
    #[error("plugin did not settle cancellation within the grace period")]
    GraceExpired,
    /// The child did not exit after receiving the shutdown frame.
    #[error("plugin did not exit within the shutdown grace period")]
    ShutdownTimeout,
    /// The plugin process group or Job Object remained alive after termination.
    #[error("plugin process group did not stop within the one-second cleanup limit")]
    ProcessTreeCleanupTimeout,
    /// The caller cancelled the plugin run.
    #[error("plugin run was cancelled")]
    Cancelled,
    /// The caller deadline elapsed while the plugin run remained open.
    #[error("plugin run exceeded its caller-owned deadline")]
    TimedOut,
    /// The plugin sent a typed `fault` body.
    #[error("plugin sent a typed fault")]
    Fault(F),
    /// The caller rejected the FR-331 hello manifest during admission.
    #[error("plugin manifest admission failed")]
    ManifestAdmission(A),
    /// The caller's typed FR-331 reader rejected the result.
    #[error("plugin result validation failed")]
    ResultValidation(V),
    /// A typed request or fault body could not be serialized or deserialized.
    #[error("plugin typed JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    /// The stdout frame thread panicked.
    #[error("plugin frame reader thread terminated unexpectedly")]
    ReaderThread,
    /// The stdin frame writer thread stopped unexpectedly.
    #[error("plugin frame writer thread terminated unexpectedly")]
    WriterThread,
    /// A control frame could not be written within the cancellation grace.
    #[error("plugin frame writer did not complete within the grace period")]
    WriterUnresponsive,
    /// A process I/O worker remained alive after bounded cleanup.
    #[error("plugin {0} worker did not stop within the cleanup grace")]
    WorkerJoinTimeout(&'static str),
    /// A named process I/O worker could not be started.
    #[error("could not start plugin {worker} worker: {source}")]
    WorkerSpawn {
        /// The worker role that failed to start.
        worker: &'static str,
        /// The operating-system thread creation error.
        #[source]
        source: io::Error,
    },
    /// The stderr capture thread panicked.
    #[error("plugin stderr capture thread terminated unexpectedly")]
    CaptureThread,
    /// Reading opaque plugin stderr failed.
    #[error("plugin stderr capture failed: {0}")]
    Stderr(#[source] io::Error),
    /// The host observed a process resource limit breach or could not measure
    /// a requested process budget on this platform.
    #[error("plugin process resource budget failed: {0:?}")]
    ProcessBudget(ProcessBudgetError),
}

impl<A, V, F> PluginError<A, V, F> {
    fn from_spawn(path: &Path, source: io::Error) -> Self {
        Self::Spawn {
            path: path.to_path_buf(),
            source,
        }
    }

    /// Return the primary typed failure beneath any captured diagnostics.
    #[must_use]
    pub fn primary_cause(&self) -> &Self {
        match self {
            Self::WithDiagnostics { cause, .. } => cause.primary_cause(),
            cause => cause,
        }
    }
}

/// Errors while framing, canonicalizing or reading a plugin message.
#[derive(Debug, Error)]
pub enum FrameError {
    /// The input stream ended partway through an eight-byte length word.
    #[error("plugin frame ended partway through its length prefix")]
    TruncatedLength,
    /// The input stream ended before all declared frame bytes arrived.
    #[error("plugin frame ended before its declared length")]
    TruncatedFrame,
    /// A frame exceeded the caller's per-frame limit.
    #[error("plugin frame length {actual} exceeds limit {maximum}")]
    FrameTooLarge {
        /// The encoded frame's declared size.
        actual: u64,
        /// The caller's maximum frame size.
        maximum: u64,
    },
    /// A run exceeded the caller's aggregate byte limit.
    #[error("plugin run byte count {actual} exceeds limit {maximum}")]
    RunTooLarge {
        /// The aggregate encoded bytes requested or received.
        actual: u64,
        /// The caller's maximum total bytes.
        maximum: u64,
    },
    /// A length or aggregate count could not be represented safely.
    #[error("plugin frame byte count overflow")]
    TotalTooLarge,
    /// The JSON bytes were valid but not their exact RFC 8785 encoding.
    #[error("plugin frame is not RFC 8785 canonical JSON")]
    NonCanonical,
    /// The envelope used an identity other than the sole accepted protocol.
    #[error("plugin protocol identity {0:?} is not accepted")]
    WrongProtocol(String),
    /// JSON parsing or typed body conversion failed.
    #[error("plugin frame JSON failed: {0}")]
    Json(#[from] serde_json::Error),
    /// Canonical encoding failed or exceeded its configured limit.
    #[error("plugin frame canonical encoding failed: {0}")]
    Canonical(#[from] quire_canonical::Error),
    /// An input or output operation failed.
    #[error("plugin frame I/O failed: {0}")]
    Io(#[from] io::Error),
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use super::{
        spawn_writer, write_frame_control, Envelope, FrameError, FrameLimits, FrameReader,
        FrameWriter, Kind, PluginError, PluginSession, ProcessBudgetError, RunLimits, SessionStart,
        PROTOCOL,
    };
    use quire_exact::Cancel;
    use quire_exact::CancelCause;
    use serde::{Deserialize, Serialize};
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestManifest {
        identity: String,
        advertises: Vec<String>,
    }

    #[derive(Serialize)]
    struct TestRequest {
        items: Vec<String>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestResult {
        identity: String,
        items: Vec<String>,
    }

    const LIMITS: FrameLimits = FrameLimits {
        max_frame_bytes: 1024,
        max_total_bytes: 4096,
    };

    #[test]
    fn frame_round_trips_one_canonical_value() {
        let envelope = Envelope {
            protocol: PROTOCOL.to_owned(),
            id: 7,
            kind: Kind::Request,
            body: serde_json::json!({"z": 1, "a": true}),
        };
        let mut bytes = Vec::new();
        FrameWriter::new(&mut bytes, LIMITS)
            .write(&envelope)
            .expect("canonical frame writes");
        assert_eq!(
            u64::from_be_bytes(bytes[..8].try_into().expect("length")),
            (bytes.len() - 8) as u64
        );
        let mut reader = FrameReader::new(bytes.as_slice(), LIMITS);
        assert_eq!(
            reader.read::<Envelope>().expect("frame reads"),
            Some(envelope)
        );
        assert_eq!(reader.read::<Envelope>().expect("clean EOF"), None);
    }

    /// Trace: FR-013-AC-7, TC-024; refuse non-canonical and oversized input before decoding.
    #[test]
    fn reader_rejects_noncanonical_and_oversized_frames() {
        let mut noncanonical = (13_u64).to_be_bytes().to_vec();
        noncanonical.extend_from_slice(b"{\"b\":1,\"a\":2}");
        assert!(matches!(
            FrameReader::new(noncanonical.as_slice(), LIMITS).read::<serde_json::Value>(),
            Err(FrameError::NonCanonical)
        ));

        let oversized = (1025_u64).to_be_bytes();
        assert!(matches!(
            FrameReader::new(oversized.as_slice(), LIMITS).read::<serde_json::Value>(),
            Err(FrameError::FrameTooLarge { .. })
        ));
    }

    /// Trace: FR-013-AC-7, TC-024; input and output share one aggregate frame budget.
    #[test]
    fn writer_and_reader_enforce_aggregate_bytes() {
        let value = serde_json::json!("x");
        let small = FrameLimits {
            max_frame_bytes: 32,
            max_total_bytes: 10,
        };
        let mut bytes = Vec::new();
        let mut writer = FrameWriter::new(&mut bytes, small);
        assert!(matches!(
            writer.write(&value),
            Err(FrameError::RunTooLarge { .. })
        ));
        let mut framed = Vec::new();
        FrameWriter::new(&mut framed, LIMITS)
            .write(&value)
            .expect("first frame writes");
        let mut reader = FrameReader::new(framed.as_slice(), small);
        assert!(matches!(
            reader.read::<serde_json::Value>(),
            Err(FrameError::RunTooLarge { .. })
        ));
    }

    /// Trace: FR-013-AC-2, TC-023; reject protocol identity before decoding a typed body.
    #[test]
    fn body_decode_requires_protocol_identity() {
        let envelope = Envelope {
            protocol: "other/v1".to_owned(),
            id: 1,
            kind: Kind::Hello,
            body: serde_json::json!({"identity": "p"}),
        };
        assert!(matches!(
            envelope.decode_body::<serde_json::Value>(),
            Err(FrameError::WrongProtocol(_))
        ));
    }

    fn failure_fixture() -> std::path::PathBuf {
        ensure_fixture_runtime("python3").unwrap_or_else(|message| panic!("{message}"));
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/failure-plugin.py")
    }

    fn echo_fixture() -> std::path::PathBuf {
        ensure_fixture_runtime("python3").unwrap_or_else(|message| panic!("{message}"));
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/echo-plugin.py")
    }

    fn ensure_fixture_runtime(executable: &str) -> Result<(), String> {
        let available = Command::new(executable)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if available {
            Ok(())
        } else {
            Err(format!(
                "quire-plugin-host tests require fixture runtime {executable:?}; install Python 3 and rerun make test"
            ))
        }
    }

    fn ready_fixture_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "quire-plugin-host-ready-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time is after epoch")
                .as_nanos()
        ))
    }

    fn cancel_after_fixture_ready(
        path: std::path::PathBuf,
        cancel: Cancel,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            while !path.exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            assert!(
                path.exists(),
                "fixture reaches the synchronized cancellation point"
            );
            cancel.cancel(CancelCause::Requested);
        })
    }

    fn session_limits() -> RunLimits {
        RunLimits {
            frames: LIMITS,
            max_stderr_bytes: 128,
            cancellation_grace: Duration::from_millis(250),
            max_process_memory_bytes: None,
            max_process_cpu_time: None,
        }
    }

    fn test_request() -> TestRequest {
        TestRequest {
            items: vec!["item-1".to_owned()],
        }
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-1, FR-013-AC-2, FR-013-AC-3, TC-023; admit hello and serve zero or multiple requests.
    #[test]
    fn session_admits_hello_then_accepts_zero_or_multiple_requests() {
        let cancel = Cancel::new();
        let executable = echo_fixture();
        for request_count in 0..=2 {
            let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
                SessionStart {
                    executable: &executable,
                    arguments: &[],
                    limits: session_limits(),
                    cancel: &cancel,
                },
                |manifest| {
                    (manifest.identity == "agent-ix/provider/test")
                        .then_some(())
                        .ok_or_else(|| "unexpected identity".to_owned())
                },
            )
            .expect("hello is admitted before any request exists");
            assert_eq!(
                session
                    .manifest()
                    .map(|manifest| manifest.identity.as_str()),
                Some("agent-ix/provider/test")
            );
            for index in 0..request_count {
                let item = format!("item-{}", index + 1);
                let request = TestRequest {
                    items: vec![item.clone()],
                };
                let result = session
                    .request(&request, &cancel, |manifest, result: &TestResult| {
                        (manifest.identity == result.identity && result.items == request.items)
                            .then_some(())
                            .ok_or_else(|| "result did not match request".to_owned())
                    })
                    .expect("session returns each validated result");
                assert_eq!(result.items, [item]);
            }
            let finished = session
                .shutdown()
                .expect("session shuts down after zero, one, or two requests");
            assert_eq!(finished.manifest.identity, "agent-ix/provider/test");
            assert_eq!(finished.stderr, b"opaque provider diagnostic\n");
        }
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-5, TC-025; return a validated settlement received within cancellation grace.
    #[test]
    fn session_preserves_valid_result_received_during_cancel_grace() {
        let cancel = Cancel::new();
        let notifier = cancel.clone();
        let ready = ready_fixture_path();
        let cancellation = cancel_after_fixture_ready(ready.clone(), notifier);
        let executable = echo_fixture();
        let arguments = [OsString::from("wait-cancel"), ready.as_os_str().to_owned()];
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = test_request();
        let result = session
            .request(&request, &cancel, |manifest, result: &TestResult| {
                (manifest.identity == result.identity && result.items == request.items)
                    .then_some(())
                    .ok_or_else(|| "invalid settlement".to_owned())
            })
            .expect("valid cancellation-grace settlement is retained");
        assert_eq!(result.items, request.items);
        cancellation.join().expect("cancellation sender exits");
        session.shutdown().expect("settled session shuts down");
        let _ = std::fs::remove_file(ready);
    }

    /// Trace: FR-013-AC-5, TC-025; a completed writer notification wins the simultaneous cancel.
    #[test]
    fn completed_write_notification_wins_cancellation() {
        let cancel = Cancel::new();
        cancel.cancel(CancelCause::Requested);
        let (sender, receiver) = super::mpsc::channel();
        sender.send(Ok(())).expect("writer completion is queued");
        let result = super::completed_write_wins_cancellation::<String, String, serde_json::Value>(
            &receiver, &cancel,
        );
        assert!(matches!(result, Some(Ok(()))));
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-2, FR-013-AC-4, TC-023, TC-024; preserve admission, validation, plugin fault, and exit failures.
    #[test]
    fn session_reports_admission_result_fault_and_exit_failures_with_diagnostics() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("echo")];
        let rejected = match PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Err::<(), _>("rejected hello".to_owned()),
        ) {
            Ok(_) => panic!("admission rejection stops before a request"),
            Err(error) => error,
        };
        assert!(matches!(
            rejected.primary_cause(),
            PluginError::ManifestAdmission(_)
        ));

        for (mode, expected) in [
            ("invalid-result", "validation"),
            ("fault", "fault"),
            ("nonzero", "exit"),
        ] {
            let arguments = [OsString::from(mode)];
            let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
                SessionStart {
                    executable: &executable,
                    arguments: &arguments,
                    limits: session_limits(),
                    cancel: &cancel,
                },
                |_manifest| Ok::<(), String>(()),
            )
            .expect("hello is admitted");
            let request = test_request();
            let result = session.request(&request, &cancel, |_, result: &TestResult| {
                (result.identity == "agent-ix/provider/test")
                    .then_some(())
                    .ok_or_else(|| "unexpected identity".to_owned())
            });
            let error = result.expect_err("fixture failure is preserved");
            match expected {
                "validation" => assert!(matches!(
                    error.primary_cause(),
                    PluginError::ResultValidation(_)
                )),
                "fault" => assert!(matches!(error.primary_cause(), PluginError::Fault(_))),
                "exit" => assert!(
                    matches!(error.primary_cause(), PluginError::Exited(status) if status.code() == Some(17))
                ),
                _ => unreachable!(),
            }
            let PluginError::WithDiagnostics { stderr, .. } = &error else {
                panic!("failures retain process diagnostics");
            };
            if expected == "exit" {
                assert!(stderr
                    .windows(b"plugin exited with failure".len())
                    .any(|window| window == b"plugin exited with failure"));
            }
        }
    }

    /// Trace: FR-013-AC-4, FR-013-AC-5, TC-024, TC-025; a plugin fault during cancellation grace remains a fault.
    #[cfg(unix)]
    #[test]
    fn cancellation_grace_preserves_plugin_faults() {
        let cancel = Cancel::new();
        let notifier = cancel.clone();
        let ready = ready_fixture_path();
        let cancellation = cancel_after_fixture_ready(ready.clone(), notifier);
        let executable = failure_fixture();
        let arguments = [
            OsString::from("fault-after-cancel"),
            ready.as_os_str().to_owned(),
        ];
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = test_request();
        let error = session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect_err("fault during cancellation grace is not remapped");
        assert!(matches!(error.primary_cause(), PluginError::Fault(_)));
        cancellation.join().expect("cancellation sender exits");
        let _ = std::fs::remove_file(ready);
    }

    /// Trace: FR-013-AC-5, FR-013-AC-9, TC-025; grace expiry kills the process group and joins workers within one second.
    #[cfg(unix)]
    #[test]
    fn cancellation_grace_expiry_terminates_the_plugin() {
        let cancel = Cancel::new();
        let notifier = cancel.clone();
        let ready = ready_fixture_path();
        let cancellation = cancel_after_fixture_ready(ready.clone(), notifier);
        let executable = failure_fixture();
        let arguments = [
            OsString::from("ignore-cancel"),
            ready.as_os_str().to_owned(),
        ];
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = test_request();
        let started = Instant::now();
        let error = session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect_err("missing settlement expires cancellation grace");
        assert!(matches!(error.primary_cause(), PluginError::Cancelled));
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(matches!(
            &error,
            PluginError::WithDiagnostics {
                reader_join_failed: false,
                writer_join_failed: false,
                ..
            }
        ));
        cancellation.join().expect("cancellation sender exits");
        let _ = std::fs::remove_file(ready);
    }

    /// Trace: FR-013-AC-9, FR-013-AC-10, TC-026; kill descendants holding inherited pipes.
    #[cfg(unix)]
    #[test]
    fn shutdown_kills_descendant_holding_stdout_and_bounds_reader_join() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("descendant-hang")];
        let mut limits = session_limits();
        limits.cancellation_grace = Duration::from_millis(100);
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = test_request();
        session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect("plugin settles the request");
        let started = Instant::now();
        let error = session
            .shutdown()
            .expect_err("inherited stdout remains open");
        assert!(matches!(
            error.primary_cause(),
            PluginError::ShutdownTimeout
        ));
        assert!(started.elapsed() < Duration::from_secs(2));
        match error {
            PluginError::WithDiagnostics {
                reader_join_failed,
                writer_join_failed,
                ..
            } => {
                assert!(
                    !reader_join_failed,
                    "group cleanup releases inherited stdout"
                );
                assert!(!writer_join_failed, "shutdown releases the stdin writer");
            }
            other => panic!("expected cleanup diagnostics, got {other:?}"),
        }
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-7, FR-013-AC-9, FR-013-AC-10, TC-024, TC-026; preserve truncated frames and bound stderr.
    #[test]
    fn session_preserves_truncated_hello_status_and_bounds_stderr() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let eof_arguments = [OsString::from("eof-before-hello")];
        let eof_error = match PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &eof_arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        ) {
            Ok(_) => panic!("EOF before hello preserves the child's exit status"),
            Err(error) => error,
        };
        assert!(matches!(
            eof_error.primary_cause(),
            PluginError::Exited(status) if status.code() == Some(17)
        ));

        let arguments = [OsString::from("truncated-hello")];
        let error = match PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        ) {
            Ok(_) => panic!("truncated hello fails"),
            Err(error) => error,
        };
        assert!(matches!(
            error.primary_cause(),
            PluginError::Frame(FrameError::TruncatedLength)
        ));

        let arguments = [OsString::from("stderr")];
        let mut limits = session_limits();
        limits.max_stderr_bytes = 17;
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = test_request();
        session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect("request succeeds");
        let finished = session.shutdown().expect("shutdown succeeds");
        assert_eq!(finished.stderr, vec![b'x'; 17]);
        assert!(finished.stderr_truncated);
    }

    /// Trace: FR-013-AC-13, TC-027; report an actionable fixture runtime prerequisite.
    #[test]
    fn fixture_runtime_prerequisite_names_python3() {
        assert_eq!(ensure_fixture_runtime("python3"), Ok(()));
        assert_eq!(
            ensure_fixture_runtime("quire-plugin-host-python3-missing-test"),
            Err("quire-plugin-host tests require fixture runtime \"quire-plugin-host-python3-missing-test\"; install Python 3 and rerun make test".to_owned())
        );
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-6, TC-025; deadlines interrupt a blocked request write.
    #[test]
    fn blocked_request_write_observes_deadline_within_a_bounded_interval() {
        let cancel = Cancel::new();
        let notifier = cancel.clone();
        let ready = ready_fixture_path();
        let ready_for_thread = ready.clone();
        let cancellation = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            while !ready_for_thread.exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(5));
            }
            assert!(
                ready_for_thread.exists(),
                "blocked fixture is synchronized before timeout"
            );
            notifier.cancel(CancelCause::Deadline);
        });
        let executable = failure_fixture();
        let arguments = [
            OsString::from("blocked-write"),
            ready.as_os_str().to_owned(),
        ];
        let mut limits = session_limits();
        limits.frames.max_frame_bytes = 2 * 1024 * 1024;
        limits.frames.max_total_bytes = 4 * 1024 * 1024;
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = TestRequest {
            items: vec!["x".repeat(1024 * 1024)],
        };
        let started = Instant::now();
        let error = session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect_err("deadline interrupts a blocked request write");
        assert!(matches!(error.primary_cause(), PluginError::TimedOut));
        assert!(started.elapsed() < Duration::from_secs(2));
        cancellation.join().expect("deadline sender exits");
        let _ = std::fs::remove_file(ready);
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-1, TC-023; refuse bare executable names.
    #[test]
    fn session_enforces_cpu_budget_and_refuses_bare_executable_names() {
        let cancel = Cancel::new();
        let error = match PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: Path::new("bare-plugin-name"),
                arguments: &[],
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        ) {
            Ok(_) => panic!("bare executable names are not resolved through PATH"),
            Err(error) => error,
        };
        assert!(matches!(error, PluginError::BareExecutableName(_)));

        let executable = failure_fixture();
        let arguments = [OsString::from("cpu")];
        let mut limits = session_limits();
        limits.max_process_cpu_time = Some(Duration::ZERO);
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let request = test_request();
        let error = session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect_err("CPU budget stops a spinning plugin");
        if cfg!(target_os = "linux") {
            assert!(matches!(
                error.primary_cause(),
                PluginError::ProcessBudget(ProcessBudgetError::CpuTimeExceeded)
            ));
        } else {
            assert!(matches!(
                error.primary_cause(),
                PluginError::ProcessBudget(ProcessBudgetError::Unavailable)
            ));
        }
    }

    /// Trace: FR-013-AC-2, TC-023; reject protocol identity before manifest admission.
    #[cfg(unix)]
    #[test]
    fn session_rejects_wrong_protocol_before_admission() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("wrong-protocol")];
        let error = match PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        ) {
            Ok(_) => panic!("wrong protocol must not reach admission"),
            Err(error) => error,
        };
        assert!(matches!(
            error.primary_cause(),
            PluginError::Frame(FrameError::WrongProtocol(_))
        ));
    }

    /// Trace: FR-013-AC-4, FR-013-AC-7, TC-024; reject a response correlated to another request.
    #[cfg(unix)]
    #[test]
    fn session_rejects_wrong_request_id() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("wrong-id")];
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let error = session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect_err("wrong request id is rejected");
        assert!(matches!(
            error.primary_cause(),
            PluginError::UnexpectedId {
                expected: 1,
                actual: 2
            }
        ));
    }

    /// Trace: FR-013-AC-8, TC-026; refuse dispatch when complete process measurement is unavailable.
    #[cfg(unix)]
    #[test]
    fn begin_request_returns_unavailable_before_dispatch() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let marker = ready_fixture_path();
        let ready = PathBuf::from(format!("{}.ready", marker.display()));
        let arguments = [
            OsString::from("request-marker"),
            marker.as_os_str().to_owned(),
        ];
        let mut limits = session_limits();
        limits.max_process_memory_bytes = Some(u64::MAX);
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let deadline = Instant::now() + Duration::from_secs(1);
        while !ready.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        assert!(ready.exists(), "fixture is waiting for its first request");
        session.monitor.force_unavailable = true;
        let error = session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect_err("measurement failure prevents request dispatch");
        assert!(matches!(
            error.primary_cause(),
            PluginError::ProcessBudget(ProcessBudgetError::Unavailable)
        ));
        assert!(!session.request_write_started);
        assert!(
            !marker.exists(),
            "the request was not written before measurement"
        );
        let closed = session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect_err("a terminated session cannot be reused");
        assert!(matches!(closed, PluginError::SessionClosed));
        let _ = std::fs::remove_file(marker);
        let _ = std::fs::remove_file(ready);
    }

    /// Trace: FR-013-AC-8, TC-026; sum memory for double-forked process-group members.
    #[cfg(target_os = "linux")]
    #[test]
    fn process_memory_budget_includes_descendant_usage() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let ready = ready_fixture_path();
        let arguments = [
            OsString::from("descendant-memory"),
            ready.as_os_str().to_owned(),
        ];
        let mut limits = session_limits();
        limits.max_process_memory_bytes = Some(48 * 1024 * 1024);
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        assert!(
            (0..3).all(|index| PathBuf::from(format!("{}.{}", ready.display(), index)).exists()),
            "three double-fork descendants signaled readiness"
        );
        let members = session
            .monitor
            .refresh_tree()
            .expect("group members are measured");
        let member_memory: Vec<u64> = members.iter().map(|sample| sample.memory_bytes).collect();
        assert!(member_memory.iter().all(|bytes| *bytes < 48 * 1024 * 1024));
        assert!(
            members
                .iter()
                .filter(|sample| sample.memory_bytes > 0)
                .count()
                >= 3
        );
        assert!(member_memory.iter().sum::<u64>() > 48 * 1024 * 1024);
        let error = session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect_err("descendant pushes aggregate process memory over the limit");
        assert!(matches!(
            error.primary_cause(),
            PluginError::ProcessBudget(ProcessBudgetError::MemoryExceeded)
        ));
        let _ = std::fs::remove_file(ready);
        for index in 0..3 {
            let _ = std::fs::remove_file(format!("{}.{}", ready.display(), index));
        }
    }

    /// Trace: FR-013-AC-8, TC-026; fork and exit races remain best-effort samples.
    #[cfg(target_os = "linux")]
    #[test]
    fn forking_plugin_does_not_turn_process_sample_races_into_unavailable() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("fork-during-request")];
        let mut limits = session_limits();
        limits.max_process_memory_bytes = Some(512 * 1024 * 1024);
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let result = session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect("a process appearing during polling is not an unavailable failure");
        assert_eq!(result.items, test_request().items);
        session.shutdown().expect("fork descendants are terminated");
    }

    /// Trace: FR-013-AC-8, TC-026; accumulate CPU deltas for a short-lived descendant.
    #[cfg(target_os = "linux")]
    #[test]
    fn process_cpu_budget_includes_short_lived_descendant() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("descendant-cpu")];
        let mut limits = session_limits();
        limits.max_process_cpu_time = Some(Duration::from_millis(50));
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let error = session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect_err("descendant CPU exceeds aggregate request limit");
        assert!(matches!(
            error.primary_cause(),
            PluginError::ProcessBudget(ProcessBudgetError::CpuTimeExceeded)
        ));
    }

    struct SlowWriter;

    impl std::io::Write for SlowWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            thread::sleep(Duration::from_millis(500));
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Trace: FR-013-AC-5, FR-013-AC-9, TC-025; control writes use the configured grace.
    #[test]
    fn cancellation_control_write_wait_is_bounded_by_grace() {
        let (writer, worker) = spawn_writer(
            SlowWriter,
            LIMITS.max_frame_bytes,
            std::sync::Arc::new(super::ByteBudget::new(LIMITS.max_total_bytes)),
        )
        .expect("writer worker starts");
        let grace = Duration::from_millis(80);
        let started = Instant::now();
        let error = write_frame_control::<String, String, String>(
            &writer,
            Envelope {
                protocol: PROTOCOL.to_owned(),
                id: 1,
                kind: Kind::Cancel,
                body: serde_json::json!({"id": 1}),
            },
            grace,
        )
        .expect_err("blocked control write is bounded");
        assert!(matches!(error, PluginError::WriterUnresponsive));
        assert!(started.elapsed() >= Duration::from_millis(60));
        assert!(started.elapsed() < Duration::from_millis(250));
        drop(writer);
        assert!(super::join_thread_within(&worker, Duration::from_secs(1)));
        worker
            .join()
            .expect("worker exits after its bounded fake write");
    }

    /// Trace: FR-013-AC-8, TC-026; sysinfo thread tasks do not inflate process-tree membership.
    #[cfg(target_os = "linux")]
    #[test]
    fn multithreaded_plugin_counts_as_one_process() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("thread-tasks")];
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        let members = session.monitor.refresh_tree().expect("tree sampling works");
        assert_eq!(members.len(), 1);
        drop(session);
    }
    /// Trace: FR-013-AC-7, TC-024; charge hello, requests and results to one session budget.
    #[cfg(unix)]
    #[test]
    fn session_shares_aggregate_byte_budget_across_directions() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("echo")];
        let mut limits = session_limits();
        limits.frames.max_total_bytes = 400;
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits,
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello fits within the shared run budget");
        let request = TestRequest {
            items: vec!["x".repeat(160)],
        };
        let error = session
            .request(&request, &cancel, |_manifest, _result: &TestResult| {
                Ok::<(), String>(())
            })
            .expect_err("the response exceeds the combined hello/request/response budget");
        assert!(matches!(
            error.primary_cause(),
            PluginError::Frame(FrameError::RunTooLarge { .. })
        ));
    }

    /// Trace: FR-013-AC-7, TC-024; reject frames sent after the request result.
    #[cfg(unix)]
    #[test]
    fn session_rejects_extra_frame_after_result() {
        let cancel = Cancel::new();
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extra-frame-plugin.py");
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &[],
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect("the primary result is valid");
        let error = session
            .shutdown()
            .expect_err("post-result frames are rejected");
        assert!(matches!(
            error.primary_cause(),
            PluginError::UnexpectedFrame
        ));
    }
    /// Trace: FR-013-AC-9, FR-013-AC-10, TC-026; clean shutdown kills descendants with detached standard streams.
    #[cfg(unix)]
    #[test]
    fn clean_shutdown_terminates_detached_stdio_descendant() {
        let cancel = Cancel::new();
        let executable = failure_fixture();
        let arguments = [OsString::from("detached-stdio")];
        let mut session = PluginSession::<TestManifest, serde_json::Value, String>::start(
            SessionStart {
                executable: &executable,
                arguments: &arguments,
                limits: session_limits(),
                cancel: &cancel,
            },
            |_manifest| Ok::<(), String>(()),
        )
        .expect("hello is admitted");
        session
            .request(
                &test_request(),
                &cancel,
                |_manifest, _result: &TestResult| Ok::<(), String>(()),
            )
            .expect("plugin returns a valid result");
        let started = Instant::now();
        session
            .shutdown()
            .expect("shutdown reaps detached descendants");
        assert!(started.elapsed() < Duration::from_secs(1));
    }
}
