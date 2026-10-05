// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canonical framing and wire types for ADR-029 PL-2 through PL-7.
//!
//! The crate owns the process-facing plugin boundary outside the checked core.
//! FR-331 bodies remain caller-typed; this crate does not define a second
//! provider schema or settlement vocabulary.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

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
    if request.cancel.cause().is_some() {
        return Err(cancel_error(
            request.cancel.cause().unwrap_or(CancelCause::Requested),
        ));
    }
    let mut child = Command::new(request.executable)
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
    let mut writer = FrameWriter::with_budget(
        stdin,
        request.limits.frames.max_frame_bytes,
        Arc::clone(&byte_budget),
    );
    let (sender, receiver) = mpsc::channel::<Result<Option<Envelope>, FrameError>>();
    let reader_budget = Arc::clone(&byte_budget);
    let frame_max = request.limits.frames.max_frame_bytes;
    let reader_thread = thread::spawn(move || {
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
    });
    let stderr_limit = request.limits.max_stderr_bytes;
    let stderr_thread = thread::spawn(move || capture_stderr(stderr, stderr_limit));

    let result = (|| {
        let hello = next_frame(&receiver, &mut child, request.cancel, &mut monitor)?;
        if hello.kind == Kind::Fault {
            hello.validate_protocol()?;
            return Err(PluginError::Fault(hello.decode_body::<F>()?));
        }
        expect_kind(&hello, Kind::Hello)?;
        let manifest = hello.decode_body::<M>()?;
        admit_manifest(&manifest).map_err(PluginError::ManifestAdmission)?;

        let request_id = 1_u64;
        let request_body = serde_json::to_value(request.body).map_err(PluginError::Json)?;
        writer.write(&Envelope {
            protocol: PROTOCOL.to_owned(),
            id: request_id,
            kind: Kind::Request,
            body: request_body,
        })?;

        let result_frame = match next_frame(&receiver, &mut child, request.cancel, &mut monitor) {
            Ok(frame) => frame,
            Err(PluginError::Cancelled) => {
                writer.write(&Envelope {
                    protocol: PROTOCOL.to_owned(),
                    id: request_id,
                    kind: Kind::Cancel,
                    body: serde_json::json!({"id": request_id}),
                })?;
                let frame = next_frame_with_grace::<A, V, F>(
                    &receiver,
                    &mut child,
                    request.limits.cancellation_grace,
                    &mut monitor,
                );
                match frame {
                    Ok(frame) => {
                        expect_request_id(&frame, request_id)?;
                        expect_kind(&frame, Kind::Result)?;
                        let typed = frame.decode_body::<R>()?;
                        validate_result(&manifest, &typed)
                            .map_err(PluginError::ResultValidation)?;
                        send_shutdown(&mut writer, request_id)?;
                        drop(writer);
                        wait_for_exit(&receiver, &mut child, request.cancel, &mut monitor)?;
                        let (stderr, stderr_truncated) = join_stderr(stderr_thread)?;
                        join_reader(reader_thread)?;
                        return Ok(PluginRun {
                            manifest,
                            result: typed,
                            stderr,
                            stderr_truncated,
                        });
                    }
                    Err(_) => {
                        return Err(cancel_error(
                            request.cancel.cause().unwrap_or(CancelCause::Requested),
                        ));
                    }
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
        send_shutdown(&mut writer, request_id)?;
        drop(writer);
        wait_for_exit(&receiver, &mut child, request.cancel, &mut monitor)?;
        let (stderr, stderr_truncated) = join_stderr(stderr_thread)?;
        join_reader(reader_thread)?;
        Ok(PluginRun {
            manifest,
            result: typed,
            stderr,
            stderr_truncated,
        })
    })();

    if result.is_err() {
        terminate(&mut child);
    }
    result
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
            Ok(Ok(None)) => return Err(PluginError::MissingFrame),
            Ok(Err(error)) => return Err(PluginError::Frame(error)),
            Err(RecvTimeoutError::Disconnected) => return Err(PluginError::MissingFrame),
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

fn send_shutdown<W: Write, A, V, F>(
    writer: &mut FrameWriter<W>,
    id: u64,
) -> Result<(), PluginError<A, V, F>> {
    writer.write(&Envelope {
        protocol: PROTOCOL.to_owned(),
        id,
        kind: Kind::Shutdown,
        body: serde_json::json!({}),
    })?;
    Ok(())
}

fn wait_for_exit<A, V, F>(
    receiver: &Receiver<Result<Option<Envelope>, FrameError>>,
    child: &mut Child,
    cancel: &Cancel,
    monitor: &mut ProcessMonitor,
) -> Result<ExitStatus, PluginError<A, V, F>> {
    let mut status = None;
    let mut stdout_closed = false;
    loop {
        if let Some(cause) = cancel.cause() {
            return Err(cancel_error(cause));
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

struct ProcessMonitor {
    system: System,
    pid: Pid,
    memory_limit: Option<u64>,
    cpu_time_limit: Option<Duration>,
}

impl ProcessMonitor {
    fn new(child: &Child, memory_limit: Option<u64>, cpu_time_limit: Option<Duration>) -> Self {
        Self {
            system: System::new(),
            pid: Pid::from(child.id() as usize),
            memory_limit,
            cpu_time_limit,
        }
    }

    fn check(&mut self) -> Result<(), ProcessBudgetError> {
        if self.memory_limit.is_none() && self.cpu_time_limit.is_none() {
            return Ok(());
        }
        if !sysinfo::IS_SUPPORTED_SYSTEM {
            return Err(ProcessBudgetError::Unavailable);
        }
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.pid]),
            false,
            ProcessRefreshKind::nothing().with_memory().with_cpu(),
        );
        let process = self
            .system
            .process(self.pid)
            .ok_or(ProcessBudgetError::Unavailable)?;
        if self
            .memory_limit
            .is_some_and(|limit| process.memory() > limit)
        {
            return Err(ProcessBudgetError::MemoryExceeded);
        }
        if self
            .cpu_time_limit
            .is_some_and(|limit| Duration::from_millis(process.accumulated_cpu_time()) > limit)
        {
            return Err(ProcessBudgetError::CpuTimeExceeded);
        }
        Ok(())
    }
}

/// Why the host could not continue enforcing the requested process budget.
#[derive(Debug)]
pub enum ProcessBudgetError {
    /// The child exceeded its observed resident-memory limit.
    MemoryExceeded,
    /// The child exceeded its observed accumulated CPU-time limit.
    CpuTimeExceeded,
    /// The platform could not provide process measurements.
    Unavailable,
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
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
    /// A plugin sent a second frame after its result and before exit.
    #[error("plugin sent an unexpected frame after its terminal result")]
    UnexpectedFrame,
    /// Caller cancellation reached its grace-period end without a result.
    #[error("plugin did not settle cancellation within the grace period")]
    GraceExpired,
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
        ProcessBudgetError, RunLimits, RunRequest, PROTOCOL,
    };
    use quire_exact::Cancel;
    use serde::{Deserialize, Serialize};
    use std::path::Path;
    use std::time::Duration;

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

        assert!(matches!(result, Err(PluginError::UnexpectedFrame)));
    }

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
            result,
            Err(PluginError::ProcessBudget(
                ProcessBudgetError::MemoryExceeded
            ))
        ));
    }
}
