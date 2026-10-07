// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical framing and wire types for ADR-029 PL-2 through PL-7.
//!
//! The crate owns the process-facing plugin boundary outside the checked core.
//! FR-331 bodies remain caller-typed; this crate does not define a second
//! provider schema or settlement vocabulary.

use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, Signal, System};

use quire_canonical::{to_vec, Limits};
use quire_exact::{Cancel, CancelCause};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// The sole protocol identity accepted by ADR-029 PL-4.
pub const PROTOCOL: &str = "quire.plugin-wire/v1";

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
) -> (mpsc::Sender<WriterMessage>, thread::JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel::<WriterMessage>();
    let thread = thread::spawn(move || {
        let mut writer = FrameWriter::with_budget(writer, max_frame_bytes, budget);
        while let Ok(message) = receiver.recv() {
            let result = writer.write(&message.frame);
            let failed = result.is_err();
            let _ = message.completion.send(result);
            if failed {
                break;
            }
        }
    });
    (sender, thread)
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

/// Inputs to one host-managed process-provider request.
pub struct RunRequest<'a, Q> {
    /// The exact executable path selected by the caller; no discovery occurs.
    pub executable: &'a Path,
    /// Caller-supplied argument vector, passed without shell interpretation.
    pub arguments: &'a [std::ffi::OsString],
    /// The FR-331 request body for the items routed to this process.
    pub body: &'a Q,
    /// Caller-selected transport and cancellation budgets.
    pub limits: RunLimits,
    /// Caller-owned cancellation handle. The host reads its cause but no clock.
    pub cancel: &'a Cancel,
}

/// A successful one-request plugin run.
#[derive(Debug)]
pub struct PluginRun<M, R> {
    /// The admitted manifest returned in the plugin's initial `hello`.
    pub manifest: M,
    /// The typed FR-331 result for the routed request.
    pub result: R,
    /// Opaque standard-error bytes, truncated to the caller's limit.
    pub stderr: Vec<u8>,
    /// Whether additional standard-error bytes were discarded.
    pub stderr_truncated: bool,
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
    child: Option<Child>,
    manifest: Option<M>,
    writer: Option<mpsc::Sender<WriterMessage>>,
    writer_thread: Option<thread::JoinHandle<()>>,
    receiver: Receiver<Result<Option<Envelope>, FrameError>>,
    reader_thread: Option<thread::JoinHandle<()>>,
    stderr_thread: Option<thread::JoinHandle<io::Result<CapturedStderr>>>,
    monitor: ProcessMonitor,
    limits: RunLimits,
    next_id: u64,
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
        if !has_directory_component(start.executable) {
            return Err(PluginError::BareExecutableName(
                start.executable.to_path_buf(),
            ));
        }
        if let Some(cause) = start.cancel.cause() {
            return Err(cancel_error(cause));
        }
        let mut child = Command::new(start.executable)
            .args(start.arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| PluginError::from_spawn(start.executable, error))?;
        let monitor = ProcessMonitor::new(
            &child,
            start.limits.max_process_memory_bytes,
            start.limits.max_process_cpu_time,
        );
        let Some(stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PluginError::MissingPipe("stdin"));
        };
        let Some(stdout) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PluginError::MissingPipe("stdout"));
        };
        let Some(stderr) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PluginError::MissingPipe("stderr"));
        };

        let byte_budget = Arc::new(ByteBudget::new(start.limits.frames.max_total_bytes));
        let (writer, writer_thread) = spawn_writer(
            stdin,
            start.limits.frames.max_frame_bytes,
            Arc::clone(&byte_budget),
        );
        let (sender, receiver) = mpsc::channel::<Result<Option<Envelope>, FrameError>>();
        let reader_thread = thread::spawn(move || {
            let mut reader =
                FrameReader::with_budget(stdout, start.limits.frames.max_frame_bytes, byte_budget);
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
        });
        let stderr_limit = start.limits.max_stderr_bytes;
        let stderr_thread = thread::spawn(move || capture_stderr(stderr, stderr_limit));
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
            _types: std::marker::PhantomData,
        };

        let hello = (|| {
            let child = session
                .child
                .as_mut()
                .ok_or(PluginError::MissingPipe("child"))?;
            let envelope =
                next_frame(&session.receiver, child, start.cancel, &mut session.monitor)?;
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
            write_frame_cancellable(
                writer,
                Envelope {
                    protocol: PROTOCOL.to_owned(),
                    id: request_id,
                    kind: Kind::Request,
                    body: request_body,
                },
                cancel,
                child,
                &mut self.monitor,
            )?;
            let frame = match next_frame(
                &self.receiver,
                self.child
                    .as_mut()
                    .ok_or(PluginError::MissingPipe("child"))?,
                cancel,
                &mut self.monitor,
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
                    )?;
                    let frame = match next_frame_with_grace::<A, V, F>(
                        &self.receiver,
                        self.child
                            .as_mut()
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
            send_shutdown(writer, id)?;
            self.writer.take();
            self.writer_thread
                .take()
                .ok_or(PluginError::WriterThread)?
                .join()
                .map_err(|_| PluginError::WriterThread)?;
            let child = self
                .child
                .as_mut()
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
            )?;
            join_reader(self.reader_thread.take().ok_or(PluginError::ReaderThread)?)?;
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
        if let Some(child) = self.child.as_mut() {
            self.monitor.terminate_tree(child);
        }
        self.writer.take();
        let (stderr, stderr_truncated, capture_error) = match self.stderr_thread.take() {
            Some(handle) => match handle.join() {
                Ok(Ok(captured)) => (captured.bytes, captured.truncated, None),
                Ok(Err(error)) => (Vec::new(), false, Some(error.to_string())),
                Err(_) => (Vec::new(), false, Some("stderr worker panicked".to_owned())),
            },
            None => (Vec::new(), false, None),
        };
        let reader_join_failed = self
            .reader_thread
            .take()
            .is_some_and(|handle| handle.join().is_err());
        let writer_join_failed = self
            .writer_thread
            .take()
            .is_some_and(|handle| handle.join().is_err());
        PluginError::WithDiagnostics {
            cause: Box::new(cause),
            stderr,
            stderr_truncated,
            capture_error,
            reader_join_failed,
            writer_join_failed,
        }
    }
}

impl<M, F, A> Drop for PluginSession<M, F, A> {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            self.monitor.terminate_tree(child);
        }
        self.writer.take();
        if let Some(handle) = self.writer_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Run the caller-named plugin through the ADR-029 PL-2..PL-7 lifecycle.
///
/// `admit_manifest` performs the caller's typed registration/admission before
/// any request is sent. `validate_result` checks item membership, advertised
/// kinds, terminal completeness and the manifest identity against the caller's
/// typed FR-331 reader. No second manifest, request or result schema is defined
/// here.
pub fn run<M, Q, R, F, A, V>(
    request: RunRequest<'_, Q>,
    admit_manifest: impl FnOnce(&M) -> Result<(), A>,
    validate_result: impl FnOnce(&M, &R) -> Result<(), V>,
) -> Result<PluginRun<M, R>, PluginError<A, V, F>>
where
    M: DeserializeOwned,
    Q: Serialize,
    R: DeserializeOwned,
    F: DeserializeOwned,
{
    if !has_directory_component(request.executable) {
        return Err(PluginError::BareExecutableName(
            request.executable.to_path_buf(),
        ));
    }
    if request.cancel.cause().is_some() {
        return Err(cancel_error(
            request.cancel.cause().unwrap_or(CancelCause::Requested),
        ));
    }
    let mut child = Command::new(request.executable)
        .args(request.arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| PluginError::from_spawn(request.executable, error))?;
    let mut monitor = ProcessMonitor::new(
        &child,
        request.limits.max_process_memory_bytes,
        request.limits.max_process_cpu_time,
    );
    let stdin = child
        .stdin
        .take()
        .ok_or(PluginError::MissingPipe("stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or(PluginError::MissingPipe("stdout"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or(PluginError::MissingPipe("stderr"))?;

    let byte_budget = Arc::new(ByteBudget::new(request.limits.frames.max_total_bytes));
    let (writer, writer_thread) = spawn_writer(
        stdin,
        request.limits.frames.max_frame_bytes,
        Arc::clone(&byte_budget),
    );
    let (sender, receiver) = mpsc::channel::<Result<Option<Envelope>, FrameError>>();
    let reader_budget = Arc::clone(&byte_budget);
    let frame_max = request.limits.frames.max_frame_bytes;
    let mut reader_thread = Some(thread::spawn(move || {
        let mut reader = FrameReader::with_budget(stdout, frame_max, reader_budget);
        loop {
            match reader.read::<Envelope>() {
                Ok(Some(frame)) => {
                    if sender.send(Ok(Some(frame))).is_err() {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = sender.send(Ok(None));
                    break;
                }
                Err(error) => {
                    let _ = sender.send(Err(error));
                    break;
                }
            }
        }
    }));
    let stderr_limit = request.limits.max_stderr_bytes;
    let mut stderr_thread = Some(thread::spawn(move || capture_stderr(stderr, stderr_limit)));

    let result = (|| {
        let hello = next_frame(&receiver, &mut child, request.cancel, &mut monitor)?;
        if hello.kind == Kind::Fault {
            hello.validate_protocol()?;
            return Err(PluginError::Fault(hello.decode_body::<F>()?));
        }
        expect_kind(&hello, Kind::Hello)?;
        let manifest = hello.decode_body::<M>()?;
        admit_manifest(&manifest).map_err(PluginError::ManifestAdmission)?;

        // Refuse unobservable requested process budgets before exposing any
        // request bytes to the child (FR-013-AC-8).
        monitor
            .begin_request()
            .map_err(PluginError::ProcessBudget)?;

        let request_id = 1_u64;
        let request_body = serde_json::to_value(request.body).map_err(PluginError::Json)?;
        write_frame_cancellable(
            &writer,
            Envelope {
                protocol: PROTOCOL.to_owned(),
                id: request_id,
                kind: Kind::Request,
                body: request_body,
            },
            request.cancel,
            &mut child,
            &mut monitor,
        )?;

        let result_frame = match next_frame(&receiver, &mut child, request.cancel, &mut monitor) {
            Ok(frame) => frame,
            Err(PluginError::Cancelled) => {
                write_frame_control(
                    &writer,
                    Envelope {
                        protocol: PROTOCOL.to_owned(),
                        id: request_id,
                        kind: Kind::Cancel,
                        body: serde_json::json!({"id": request_id}),
                    },
                )?;
                let frame = next_frame_with_grace::<A, V, F>(
                    &receiver,
                    &mut child,
                    request.limits.cancellation_grace,
                    &mut monitor,
                );
                match frame {
                    Ok(frame) => {
                        expect_request_id(&frame, request_id)?;
                        if frame.kind == Kind::Fault {
                            return Err(PluginError::Fault(frame.decode_body::<F>()?));
                        }
                        expect_kind(&frame, Kind::Result)?;
                        let typed = frame.decode_body::<R>()?;
                        validate_result(&manifest, &typed)
                            .map_err(PluginError::ResultValidation)?;
                        send_shutdown(&writer, request_id)?;
                        wait_for_exit(
                            &receiver,
                            &mut child,
                            &mut monitor,
                            request.limits.cancellation_grace,
                        )?;
                        let (stderr, stderr_truncated) =
                            join_stderr(stderr_thread.take().ok_or(PluginError::CaptureThread)?)?;
                        join_reader(reader_thread.take().ok_or(PluginError::ReaderThread)?)?;
                        return Ok(PluginRun {
                            manifest,
                            result: typed,
                            stderr,
                            stderr_truncated,
                        });
                    }
                    Err(PluginError::GraceExpired) => {
                        return Err(cancel_error(
                            request.cancel.cause().unwrap_or(CancelCause::Requested),
                        ));
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(error) => return Err(error),
        };

        expect_request_id(&result_frame, request_id)?;
        if result_frame.kind == Kind::Fault {
            return Err(PluginError::Fault(result_frame.decode_body::<F>()?));
        }
        expect_kind(&result_frame, Kind::Result)?;
        let typed = result_frame.decode_body::<R>()?;
        validate_result(&manifest, &typed).map_err(PluginError::ResultValidation)?;
        send_shutdown(&writer, request_id)?;
        wait_for_exit(
            &receiver,
            &mut child,
            &mut monitor,
            request.limits.cancellation_grace,
        )?;
        let (stderr, stderr_truncated) =
            join_stderr(stderr_thread.take().ok_or(PluginError::CaptureThread)?)?;
        join_reader(reader_thread.take().ok_or(PluginError::ReaderThread)?)?;
        Ok(PluginRun {
            manifest,
            result: typed,
            stderr,
            stderr_truncated,
        })
    })();

    match result {
        Ok(run) => {
            drop(writer);
            writer_thread
                .join()
                .map_err(|_| PluginError::WriterThread)?;
            Ok(run)
        }
        Err(cause) => {
            drop(writer);
            monitor.terminate_tree(&mut child);
            let (stderr, stderr_truncated, capture_error) = match stderr_thread.take() {
                Some(handle) => match handle.join() {
                    Ok(Ok(captured)) => (captured.bytes, captured.truncated, None),
                    Ok(Err(error)) => (Vec::new(), false, Some(error.to_string())),
                    Err(_) => (Vec::new(), false, Some("stderr worker panicked".to_owned())),
                },
                None => (Vec::new(), false, None),
            };
            let reader_join_failed = reader_thread
                .take()
                .is_some_and(|handle| handle.join().is_err());
            let writer_join_failed = writer_thread.join().is_err();
            Err(PluginError::WithDiagnostics {
                cause: Box::new(cause),
                stderr,
                stderr_truncated,
                capture_error,
                reader_join_failed,
                writer_join_failed,
            })
        }
    }
}

fn next_frame<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut Child,
    cancel: &Cancel,
    monitor: &mut ProcessMonitor,
) -> Result<Envelope, PluginError<A, V, F>> {
    loop {
        if let Some(cause) = cancel.cause() {
            return Err(cancel_error(cause));
        }
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(Ok(Some(frame))) => {
                monitor.check().map_err(PluginError::ProcessBudget)?;
                frame.validate_protocol()?;
                return Ok(frame);
            }
            Ok(Ok(None)) => return Err(eof_error(child)?),
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Disconnected) => return Err(eof_error(child)?),
            Err(RecvTimeoutError::Timeout) => {
                monitor.check().map_err(PluginError::ProcessBudget)?;
                if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                    return Err(PluginError::Exited(status));
                }
            }
        }
    }
}

fn next_frame_with_grace<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut Child,
    grace: Duration,
    monitor: &mut ProcessMonitor,
) -> Result<Envelope, PluginError<A, V, F>> {
    let until = Instant::now() + grace;
    loop {
        let Some(remaining) = until.checked_duration_since(Instant::now()) else {
            return Err(PluginError::GraceExpired);
        };
        match receiver.recv_timeout(remaining.min(Duration::from_millis(20))) {
            Ok(Ok(Some(frame))) => {
                monitor.check().map_err(PluginError::ProcessBudget)?;
                frame.validate_protocol()?;
                return Ok(frame);
            }
            Ok(Ok(None)) | Err(RecvTimeoutError::Disconnected) => {
                return Err(PluginError::MissingFrame);
            }
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Timeout) => {
                monitor.check().map_err(PluginError::ProcessBudget)?;
                if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                    return Err(PluginError::Exited(status));
                }
            }
        }
    }
}

fn write_frame_cancellable<A, V, F>(
    writer: &mpsc::Sender<WriterMessage>,
    frame: Envelope,
    cancel: &Cancel,
    child: &mut Child,
    monitor: &mut ProcessMonitor,
) -> Result<(), PluginError<A, V, F>> {
    let (completion, receiver) = mpsc::channel();
    writer
        .send(WriterMessage { frame, completion })
        .map_err(|_| PluginError::WriterThread)?;
    loop {
        if let Some(cause) = cancel.cause() {
            return Err(cancel_error(cause));
        }
        match receiver.recv_timeout(Duration::from_millis(20)) {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Disconnected) => return Err(PluginError::WriterThread),
            Err(RecvTimeoutError::Timeout) => {
                monitor.check().map_err(PluginError::ProcessBudget)?;
                if let Some(status) = child.try_wait().map_err(PluginError::Wait)? {
                    return Err(PluginError::Exited(status));
                }
            }
        }
    }
}

fn write_frame_control<A, V, F>(
    writer: &mpsc::Sender<WriterMessage>,
    frame: Envelope,
) -> Result<(), PluginError<A, V, F>> {
    let (completion, receiver) = mpsc::channel();
    writer
        .send(WriterMessage { frame, completion })
        .map_err(|_| PluginError::WriterThread)?;
    match receiver.recv_timeout(Duration::from_millis(20)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(PluginError::Frame(error)),
        Err(RecvTimeoutError::Timeout) => Err(PluginError::WriterUnresponsive),
        Err(RecvTimeoutError::Disconnected) => Err(PluginError::WriterThread),
    }
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
) -> Result<(), PluginError<A, V, F>> {
    write_frame_control(
        writer,
        Envelope {
            protocol: PROTOCOL.to_owned(),
            id,
            kind: Kind::Shutdown,
            body: serde_json::json!({}),
        },
    )?;
    Ok(())
}

fn wait_for_exit<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut Child,
    monitor: &mut ProcessMonitor,
    shutdown_grace: Duration,
) -> Result<ExitStatus, PluginError<A, V, F>> {
    let mut status = None;
    let mut stdout_closed = false;
    let until = Instant::now() + shutdown_grace;
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

fn eof_error<A, V, F>(child: &mut Child) -> Result<PluginError<A, V, F>, PluginError<A, V, F>> {
    let until = Instant::now() + Duration::from_millis(20);
    loop {
        match child.try_wait().map_err(PluginError::Wait)? {
            Some(status) => return Ok(PluginError::Exited(status)),
            None if Instant::now() < until => thread::sleep(Duration::from_millis(1)),
            None => return Ok(PluginError::MissingFrame),
        }
    }
}

struct ProcessMonitor {
    system: System,
    pid: Pid,
    memory_limit: Option<u64>,
    cpu_time_limit: Option<Duration>,
    last_cpu_ms: HashMap<(Pid, u64), u64>,
    request_cpu_ms: u64,
    request_active: bool,
}

impl ProcessMonitor {
    fn new(child: &Child, memory_limit: Option<u64>, cpu_time_limit: Option<Duration>) -> Self {
        Self {
            system: System::new(),
            pid: Pid::from_u32(child.id()),
            memory_limit,
            cpu_time_limit,
            last_cpu_ms: HashMap::new(),
            request_cpu_ms: 0,
            request_active: false,
        }
    }

    fn begin_request(&mut self) -> Result<(), ProcessBudgetError> {
        if self.memory_limit.is_none() && self.cpu_time_limit.is_none() {
            return Ok(());
        }
        self.last_cpu_ms.clear();
        self.request_cpu_ms = 0;
        let pids = self.refresh_tree()?;
        for pid in pids {
            if let Some(process) = self.system.process(pid) {
                self.last_cpu_ms
                    .insert((pid, process.start_time()), process.accumulated_cpu_time());
            }
        }
        self.request_active = true;
        Ok(())
    }

    fn check(&mut self) -> Result<(), ProcessBudgetError> {
        if (self.memory_limit.is_none() && self.cpu_time_limit.is_none()) || !self.request_active {
            return Ok(());
        }
        let pids = self.refresh_tree()?;
        let mut resident_bytes = 0_u64;
        for pid in pids {
            let process = self
                .system
                .process(pid)
                .ok_or(ProcessBudgetError::Unavailable)?;
            resident_bytes = resident_bytes.saturating_add(process.memory());
            let identity = (pid, process.start_time());
            let current = process.accumulated_cpu_time();
            let previous = self.last_cpu_ms.insert(identity, current).unwrap_or(0);
            self.request_cpu_ms = self
                .request_cpu_ms
                .saturating_add(current.saturating_sub(previous));
        }
        if self
            .memory_limit
            .is_some_and(|limit| resident_bytes > limit)
        {
            return Err(ProcessBudgetError::MemoryExceeded);
        }
        if self
            .cpu_time_limit
            .is_some_and(|limit| Duration::from_millis(self.request_cpu_ms) > limit)
        {
            return Err(ProcessBudgetError::CpuTimeExceeded);
        }
        Ok(())
    }

    fn refresh_tree(&mut self) -> Result<Vec<Pid>, ProcessBudgetError> {
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            return Err(ProcessBudgetError::Unavailable);
        }
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory().with_cpu(),
        );
        if self.system.process(self.pid).is_none() {
            return Err(ProcessBudgetError::Unavailable);
        }
        let mut tree = vec![self.pid];
        let mut members = HashSet::from([self.pid]);
        loop {
            let mut changed = false;
            for (pid, process) in self.system.processes() {
                if members.contains(pid) {
                    continue;
                }
                if process
                    .parent()
                    .is_some_and(|parent| members.contains(&parent))
                {
                    members.insert(*pid);
                    tree.push(*pid);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Ok(tree)
    }

    fn terminate_tree(&mut self, child: &mut Child) {
        if let Ok(tree) = self.refresh_tree() {
            for pid in tree.into_iter().rev() {
                if let Some(process) = self.system.process(pid) {
                    let _ = process.kill_with(Signal::Kill);
                }
            }
        }
        let _ = child.kill();
        let _ = child.wait();
    }
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
) -> Result<(Vec<u8>, bool), PluginError<A, V, F>> {
    let captured = handle
        .join()
        .map_err(|_| PluginError::CaptureThread)?
        .map_err(PluginError::Stderr)?;
    Ok((captured.bytes, captured.truncated))
}

fn join_reader<A, V, F>(handle: thread::JoinHandle<()>) -> Result<(), PluginError<A, V, F>> {
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
        /// Whether the stdout reader worker failed to join.
        reader_join_failed: bool,
        /// Whether the stdin writer worker failed to join.
        writer_join_failed: bool,
    },
    /// The requested executable had no directory component and was refused
    /// instead of being looked up through `PATH`.
    #[error("plugin executable {0:?} must be an explicit path with a directory component")]
    BareExecutableName(PathBuf),
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
    /// A control frame could not be written within one polling interval.
    #[error("plugin frame writer did not complete within 20 milliseconds")]
    WriterUnresponsive,
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
        run, Envelope, FrameError, FrameLimits, FrameReader, FrameWriter, Kind, PluginError,
        PluginSession, ProcessBudgetError, RunLimits, RunRequest, SessionStart, PROTOCOL,
    };
    use quire_exact::Cancel;
    use quire_exact::CancelCause;
    use serde::{Deserialize, Serialize};
    use std::ffi::OsString;
    use std::path::Path;
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/failure-plugin.py")
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
    /// Trace: FR-013-AC-1, FR-013-AC-2, FR-013-AC-3; admit hello and serve zero or multiple requests.
    #[test]
    fn session_admits_hello_then_accepts_zero_or_multiple_requests() {
        let cancel = Cancel::new();
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/echo-plugin.py");
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
    /// Trace: FR-013-AC-5; return a validated settlement received within cancellation grace.
    #[test]
    fn session_preserves_valid_result_received_during_cancel_grace() {
        let cancel = Cancel::new();
        let notifier = cancel.clone();
        let ready = ready_fixture_path();
        let cancellation = cancel_after_fixture_ready(ready.clone(), notifier);
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/echo-plugin.py");
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

    #[cfg(unix)]
    /// Trace: FR-013-AC-2, FR-013-AC-4; preserve admission, validation, plugin fault, and exit failures.
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

    /// Trace: FR-013-AC-4, FR-013-AC-5; a plugin fault during cancellation grace remains a fault.
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

    /// Trace: FR-013-AC-5; grace expiry terminates a non-settling plugin and returns cancellation.
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
        assert!(started.elapsed() < Duration::from_secs(2));
        cancellation.join().expect("cancellation sender exits");
        let _ = std::fs::remove_file(ready);
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-7, FR-013-AC-9, FR-013-AC-10; preserve truncated frames and bound stderr.
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

    /// Trace: FR-013-AC-13; report the fixture runtime prerequisite clearly.
    #[test]
    fn fixture_runtime_prerequisite_names_python3() {
        let result = Command::new("python3").arg("--version").output();
        assert!(
            result.is_ok_and(|output| output.status.success()),
            "quire-plugin-host tests require Python 3 on PATH; install Python 3 and rerun make test"
        );
    }

    #[cfg(unix)]
    /// Trace: FR-013-AC-6; deadlines interrupt a blocked request write.
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
    /// Trace: FR-013-AC-1, FR-013-AC-8; refuse bare executable names and enforce CPU budgets.
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
        assert!(matches!(
            error.primary_cause(),
            PluginError::ProcessBudget(ProcessBudgetError::CpuTimeExceeded)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn process_run_admits_manifest_validates_result_and_captures_stderr() {
        let cancel = Cancel::new();
        let request = TestRequest {
            items: vec!["item-1".to_owned(), "item-2".to_owned()],
        };
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/echo-plugin.py");
        let run = run::<TestManifest, TestRequest, TestResult, serde_json::Value, _, _>(
            RunRequest {
                executable: &executable,
                arguments: &[],
                body: &request,
                limits: RunLimits {
                    frames: LIMITS,
                    max_stderr_bytes: 128,
                    cancellation_grace: Duration::from_millis(50),
                    max_process_memory_bytes: None,
                    max_process_cpu_time: None,
                },
                cancel: &cancel,
            },
            |manifest: &TestManifest| {
                (manifest.identity == "agent-ix/provider/test")
                    .then_some(())
                    .ok_or_else(|| "unexpected identity".to_owned())
            },
            |manifest: &TestManifest, result: &TestResult| {
                (manifest.identity == result.identity && result.items == request.items)
                    .then_some(())
                    .ok_or_else(|| "result did not match admitted request".to_owned())
            },
        )
        .expect("fixture plugin completes a valid run");

        assert_eq!(run.manifest.identity, "agent-ix/provider/test");
        assert!(run.manifest.advertises.is_empty());
        assert_eq!(run.result.items, request.items);
        assert_eq!(run.stderr, b"opaque provider diagnostic\n");
        assert!(!run.stderr_truncated);
    }

    /// Trace: FR-013-AC-7, unexpected extra frames retain their typed failure.
    #[cfg(unix)]
    #[test]
    fn process_run_rejects_a_frame_after_the_result_before_clean_exit() {
        let cancel = Cancel::new();
        let request = TestRequest {
            items: vec!["item-1".to_owned()],
        };
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extra-frame-plugin.py");
        let result = run::<TestManifest, TestRequest, TestResult, serde_json::Value, _, _>(
            RunRequest {
                executable: &executable,
                arguments: &[],
                body: &request,
                limits: RunLimits {
                    frames: LIMITS,
                    max_stderr_bytes: 16,
                    cancellation_grace: Duration::from_millis(50),
                    max_process_memory_bytes: None,
                    max_process_cpu_time: None,
                },
                cancel: &cancel,
            },
            |_manifest: &TestManifest| Ok::<(), String>(()),
            |_manifest: &TestManifest, _result: &TestResult| Ok::<(), String>(()),
        );

        assert!(matches!(
            result
                .as_ref()
                .expect_err("extra frame is rejected")
                .primary_cause(),
            PluginError::UnexpectedFrame
        ));
    }

    /// Trace: FR-013-AC-8, observed memory budget breaches are typed.
    #[cfg(unix)]
    #[test]
    fn process_run_stops_when_observed_memory_exceeds_the_caller_limit() {
        let cancel = Cancel::new();
        let request = TestRequest {
            items: vec!["item-1".to_owned()],
        };
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/echo-plugin.py");
        let result = run::<TestManifest, TestRequest, TestResult, serde_json::Value, _, _>(
            RunRequest {
                executable: &executable,
                arguments: &[],
                body: &request,
                limits: RunLimits {
                    frames: LIMITS,
                    max_stderr_bytes: 16,
                    cancellation_grace: Duration::from_millis(50),
                    max_process_memory_bytes: Some(1),
                    max_process_cpu_time: None,
                },
                cancel: &cancel,
            },
            |_manifest: &TestManifest| Ok::<(), String>(()),
            |_manifest: &TestManifest, _result: &TestResult| Ok::<(), String>(()),
        );

        assert!(matches!(
            result
                .as_ref()
                .expect_err("memory budget is enforced")
                .primary_cause(),
            PluginError::ProcessBudget(ProcessBudgetError::MemoryExceeded)
        ));
    }
}
