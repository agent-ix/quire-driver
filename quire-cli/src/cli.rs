// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parser and command handlers for the user-facing `quire` binary.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use ix_cli_kit::exit::{caller_exit_code, RESERVED_COMMAND_NOT_FOUND};
use ix_cli_kit::streams::{self, Diagnostics};
use qsl_foundation::diagnostic::{Category, Code};
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    default_accounting, Call, CallArgument, DependencyInput, SpineLimits, DEFAULT_WORK_UNITS,
};
use qsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};
use quire_driver::{execute, ExecutionEngine, ExecutionFailure, ExecutionRequest, LimitOverrides};

/// Pass QSL's category result through the kit without converting it to an `Outcome`.
#[must_use]
fn process_exit_code(category: Category) -> ExitCode {
    let result = category_exit_result(category);
    match caller_exit_code(result) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            let fallback = Category::InternalFailure.exit_code();
            emit_diagnostic(
                "ExitMappingFailure",
                &format!(
                    "driver exit mapping rejected result {} (reserved code {}); using fallback code {fallback}.",
                    error.rejected_code, error.reserved_code
                ),
            );
            ExitCode::from(fallback)
        }
    }
}

#[cfg(all(feature = "tc018-test-adapter", not(debug_assertions)))]
compile_error!("the TC-018 test adapter is unavailable in release builds");

/// Return QSL's exit result, with a test-only seam for the reserved-code case.
fn category_exit_result(category: Category) -> u8 {
    #[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
    if std::env::var_os("TC018_CASE").as_deref() == Some(std::ffi::OsStr::new("reserved-exit")) {
        return RESERVED_COMMAND_NOT_FOUND;
    }

    category.exit_code()
}

/// Write a diagnostic to a caller-owned sink without panicking on a closed
/// stderr pipe.
fn write_diagnostic<W: Write>(sink: &mut W, kind: &str, message: &str) -> io::Result<()> {
    streams::Record::error(kind, message).write(Diagnostics::default(), sink)
}

/// Best-effort stderr output; a closed diagnostic pipe must not replace the
/// command's already selected QSL exit result with a panic.
fn emit_diagnostic(kind: &str, message: &str) {
    let mut stderr = io::stderr().lock();
    let _ = write_diagnostic(&mut stderr, kind, message);
}

fn version_exit_code() -> ExitCode {
    let version = concat!(env!("CARGO_PKG_VERSION"), "\n");
    match streams::write_primary_stdout(version.as_bytes()) {
        Ok(()) => process_exit_code(Category::Success),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
            process_exit_code(Category::Success)
        }
        Err(error) => {
            emit_diagnostic(
                "VersionOutputFailure",
                &format!("could not write package version: {error}"),
            );
            process_exit_code(Category::InternalFailure)
        }
    }
}

/// Parser selectors implement the command surface owned by QSL ADR-029.
#[cfg(test)]
const VERBS: &[&str] = &[
    "parse", "format", "check", "compile", "run", "analyze", "monitor", "prove", "replay",
    "generate", "inspect", "render",
];

/// Output selector for QSL ADR-029's CLI grammar.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Dot,
}

/// Dispatch selector for verbs specified by QSL ADR-029.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Verb {
    Parse,
    Format,
    Check,
    Compile,
    Run,
    Analyze,
    Monitor,
    Prove,
    Replay,
    Generate,
    Inspect,
    Render,
}

impl Verb {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "parse" => Self::Parse,
            "format" => Self::Format,
            "check" => Self::Check,
            "compile" => Self::Compile,
            "run" => Self::Run,
            "analyze" => Self::Analyze,
            "monitor" => Self::Monitor,
            "prove" => Self::Prove,
            "replay" => Self::Replay,
            "generate" => Self::Generate,
            "inspect" => Self::Inspect,
            "render" => Self::Render,
            _ => return None,
        })
    }
}

#[derive(Debug)]
enum Invocation {
    Help,
    Version,
    Operation {
        verb: Verb,
        format: OutputFormat,
        raw_arguments: Vec<OsString>,
    },
}

/// Parse the root command surface defined by QSL ADR-029.
fn parse_invocation(arguments: &[OsString]) -> Result<Invocation, String> {
    let Some(command) = arguments.first() else {
        return Ok(Invocation::Help);
    };
    let command = command
        .to_str()
        .ok_or_else(|| "command must be valid UTF-8".to_owned())?;
    if matches!(command, "--help" | "-h" | "help") {
        return Ok(Invocation::Help);
    }
    if matches!(command, "--version" | "-V") {
        return Ok(Invocation::Version);
    }
    let verb = Verb::parse(command).ok_or_else(|| format!("unknown verb `{command}`"))?;
    let mut format = OutputFormat::Text;
    let mut index = 1;
    while index < arguments.len() {
        let arg = arguments[index]
            .to_str()
            .ok_or_else(|| "option must be valid UTF-8".to_owned())?;
        if arg == "--format" {
            index += 1;
            let value = arguments
                .get(index)
                .and_then(|value| value.to_str())
                .ok_or_else(|| "--format requires text, json, or dot".to_owned())?;
            format = match value {
                "text" => OutputFormat::Text,
                "json" => OutputFormat::Json,
                "dot" => OutputFormat::Dot,
                other => return Err(format!("unknown output format `{other}`")),
            };
        }
        // Operation-specific options pass through untouched for later handlers.
        index += 1;
    }
    if format == OutputFormat::Dot && verb != Verb::Inspect {
        return Err("the `dot` format is available only for `inspect`".to_owned());
    }
    Ok(Invocation::Operation {
        verb,
        format,
        raw_arguments: arguments[1..].to_vec(),
    })
}

struct RunOptions {
    path: String,
    function: String,
    arguments: Vec<CallArgument>,
    limits: LimitOverrides,
    engine: ExecutionEngine,
}

impl RunOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut limit_operands = Vec::new();
        let mut engine = None;
        let mut index = 0;
        while index < arguments.len() {
            let arg = arguments[index]
                .to_str()
                .ok_or_else(|| "run arguments must be valid UTF-8".to_owned())?;
            match arg {
                "--limit" => {
                    index += 1;
                    limit_operands.push(
                        arguments
                            .get(index)
                            .and_then(|value| value.to_str())
                            .ok_or_else(|| "--limit requires name=value".to_owned())?
                            .to_owned(),
                    );
                }
                "--engine" => {
                    index += 1;
                    let value = arguments
                        .get(index)
                        .and_then(|value| value.to_str())
                        .ok_or_else(|| "--engine requires interpreter, aot, or jit".to_owned())?;
                    if engine.is_some() {
                        return Err("--engine may be supplied only once".to_owned());
                    }
                    engine = Some(ExecutionEngine::parse(value)?);
                }
                "--format" | "--color" => {
                    index += 1;
                    if arguments
                        .get(index)
                        .and_then(|value| value.to_str())
                        .is_none()
                    {
                        return Err(format!("{arg} requires a value"));
                    }
                }
                option if option.starts_with('-') => {
                    return Err(format!("unknown run option `{option}`"));
                }
                value => positionals.push(value.to_owned()),
            }
            index += 1;
        }
        if positionals.len() < 2 {
            return Err("run requires a source path and function name".to_owned());
        }
        let mut call_arguments = Vec::new();
        for argument in &positionals[2..] {
            let (parameter, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("run argument `{argument}` must be parameter=value"))?;
            if parameter.is_empty() {
                return Err(format!(
                    "run argument `{argument}` has an empty parameter name"
                ));
            }
            let value = match value {
                "true" => 1,
                "false" => 0,
                _ => value.parse::<i64>().map_err(|_| {
                    format!("run argument `{argument}` is not a Boolean or integer")
                })?,
            };
            call_arguments.push(CallArgument {
                parameter: parameter.to_owned(),
                value,
            });
        }
        Ok(Self {
            path: positionals[0].clone(),
            function: positionals[1].clone(),
            arguments: call_arguments,
            limits: LimitOverrides::parse(&limit_operands).map_err(|error| error.to_string())?,
            engine: engine.unwrap_or_default(),
        })
    }
}

/// One command's QSL category, optional outcome document, and diagnostic.
struct CommandResult {
    category: Category,
    document: Option<OutcomeDocument>,
    diagnostic: Option<String>,
}

impl CommandResult {
    fn document(document: OutcomeDocument) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
        }
    }

    fn failure(category: Category, message: impl Into<String>) -> Self {
        Self {
            category,
            document: None,
            diagnostic: Some(message.into()),
        }
    }

    /// An input refusal raised before QSL runs, recorded with QSL's native code.
    fn input_refusal(message: impl Into<String>) -> Self {
        Self::catalog_failure(Code::InvalidRequest, message)
    }

    fn catalog_failure(code: Code, message: impl Into<String>) -> Self {
        let message = message.into();
        Self::document(
            OutcomeDocument::new(Operation::Execute, None, code.category()).with_diagnostics(vec![
                OutcomeDiagnostic::new(None, code.as_str(), None, message.clone()),
            ]),
        )
        .with_diagnostic(message)
    }

    fn with_diagnostic(mut self, message: impl Into<String>) -> Self {
        self.diagnostic = Some(message.into());
        self
    }
}

fn write_result_diagnostics<W: Write>(result: &CommandResult, sink: &mut W) -> io::Result<()> {
    if let Some(message) = &result.diagnostic {
        return write_diagnostic(sink, "DriverDiagnostic", message);
    }
    if let Some(document) = &result.document {
        for diagnostic in document.diagnostics() {
            write_diagnostic(sink, "QslDiagnostic", diagnostic.message())?;
        }
    }
    Ok(())
}

/// Run one complete-V1 function through QSL's interpreter.
fn run(options: RunOptions) -> CommandResult {
    if options.engine != ExecutionEngine::Interpreter {
        let failure = ExecutionFailure::UnsupportedEngine(options.engine);
        return CommandResult::failure(failure.category(), failure.to_string());
    }
    let packages = BTreeMap::new();
    let dependencies = DependencyInput::default();
    let mut limits = SpineLimits::default();
    let mut call = Call {
        function: options.function,
        arguments: options.arguments,
        accounting: default_accounting(DEFAULT_WORK_UNITS),
    };
    if let Err(error) = options.limits.apply_to(&mut limits, &mut call) {
        return CommandResult::input_refusal(error.to_string());
    }
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure(Code::IoError, message);
        }
    };
    let source = SourceIdentity::new("quire", &options.path, "filesystem", "current");
    match execute(ExecutionRequest {
        source,
        path: &options.path,
        bytes: &bytes,
        packages: &packages,
        dependencies: &dependencies,
        limits,
        call,
        engine: options.engine,
    }) {
        Ok(result) => CommandResult::document(OutcomeDocument::from_call(&result.outcome)),
        Err(failure) => CommandResult::failure(failure.category(), failure.to_string()),
    }
}

/// Read at most one byte beyond the caller's source limit. Passing that
/// sentinel byte to QSL preserves its typed source-limit refusal while
/// keeping the CLI's input buffer bounded by the configured limit.
fn read_source(path: &str, source_limit: usize) -> std::io::Result<Vec<u8>> {
    let read_limit = u64::try_from(source_limit)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    let mut bytes = Vec::new();
    File::open(path)?.take(read_limit).read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Dispatch the root verb while preserving its QSL-owned result category.
fn dispatch(
    verb: Verb,
    _format: OutputFormat,
    raw_arguments: &[OsString],
    _signals: crate::signals::SignalContext,
) -> CommandResult {
    if verb == Verb::Run {
        return match RunOptions::parse(raw_arguments) {
            Ok(options) => run(options),
            Err(error) => CommandResult::input_refusal(error),
        };
    }
    if raw_arguments
        .iter()
        .any(|argument| matches!(argument.to_str(), Some("--engine" | "--limit")))
    {
        let option = raw_arguments
            .iter()
            .find_map(|argument| match argument.to_str() {
                Some(option @ ("--engine" | "--limit")) => Some(option),
                _ => None,
            })
            .unwrap_or("--engine or --limit");
        return CommandResult::failure(
            Category::Refusal,
            format!("option `{option}` is only accepted by `run`"),
        );
    }
    CommandResult {
        category: Category::Unsupported,
        document: None,
        diagnostic: None,
    }
}

/// Emit one result document with the kit's canonical JSON and stream APIs.
fn emit_result(result: &CommandResult, format: OutputFormat) -> Category {
    let mut stderr = io::stderr().lock();
    let _ = write_result_diagnostics(result, &mut stderr);
    drop(stderr);
    let Some(document) = &result.document else {
        return result.category;
    };
    if format != OutputFormat::Json {
        return result.category;
    }
    match ix_cli_kit::json::encode_canonical(document, false) {
        Ok(mut encoded) => {
            encoded.push('\n');
            match streams::write_primary_stdout(encoded.as_bytes()) {
                Ok(()) => result.category,
                Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => result.category,
                Err(error) => {
                    emit_diagnostic(
                        "OutputFailure",
                        &format!("could not write outcome document: {error}"),
                    );
                    Category::InternalFailure
                }
            }
        }
        Err(error) => {
            emit_diagnostic(
                "OutputFailure",
                &format!("could not encode outcome document: {error}"),
            );
            Category::InternalFailure
        }
    }
}

pub(super) fn entrypoint() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    match parse_invocation(&arguments) {
        Ok(Invocation::Help) => process_exit_code(Category::Success),
        Ok(Invocation::Version) => version_exit_code(),
        Ok(Invocation::Operation {
            verb,
            format,
            raw_arguments,
        }) => {
            // Check the exit invariant before output so a rejected reserved
            // result cannot accompany an outcome document.
            let result = dispatch(
                verb,
                format,
                &raw_arguments,
                crate::signals::SignalContext::default(),
            );
            let exit_result = category_exit_result(result.category);
            if exit_result == RESERVED_COMMAND_NOT_FOUND {
                process_exit_code(result.category)
            } else {
                process_exit_code(emit_result(&result, format))
            }
        }
        Err(message) => {
            let result = CommandResult::failure(Category::Refusal, message);
            process_exit_code(emit_result(&result, OutputFormat::Text))
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod outcome_tests {
    use std::io;
    use std::process::ExitCode;

    use qsl_foundation::diagnostic::Category;
    use qsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};

    use super::{process_exit_code, write_diagnostic, write_result_diagnostics, CommandResult};

    /// Trace: FR-002-AC-2; one QSL exit function preserves its numeric results.
    #[test]
    fn process_code_is_qsl_category_exit_code_without_remapping() {
        let expected = [0, 10, 10, 20, 21, 22, 22, 30];
        assert_eq!(Category::ALL.len(), expected.len());
        for (category, expected_code) in Category::ALL.into_iter().zip(expected) {
            assert_eq!(process_exit_code(category), ExitCode::from(expected_code));
        }
    }

    struct ClosedPipe;

    impl io::Write for ClosedPipe {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }

    /// Trace: FR-002-AC-8; a closed diagnostic sink returns an error without panicking.
    #[test]
    fn diagnostic_write_reports_a_closed_stderr_pipe_without_panicking() {
        let mut sink = ClosedPipe;
        let error = write_diagnostic(&mut sink, "DriverDiagnostic", "refused request")
            .expect_err("closed pipe is reported to its caller");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    }

    /// Trace: FR-002-AC-8; QSL document diagnostics are retained on stderr.
    #[test]
    fn typed_outcome_diagnostics_are_written_to_stderr() {
        let document = OutcomeDocument::new(Operation::Execute, None, Category::Refusal)
            .with_diagnostics(vec![OutcomeDiagnostic::new(
                None,
                "invalid-request",
                None,
                "the selected function has no parameter named x",
            )]);
        let result = CommandResult::document(document);
        let mut stderr = Vec::new();
        write_result_diagnostics(&result, &mut stderr).expect("write QSL diagnostic");
        assert_eq!(stderr, b"the selected function has no parameter named x\n");
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use super::{
        parse_invocation, read_source, ExecutionEngine, Invocation, OutputFormat, RunOptions, Verb,
        VERBS,
    };
    use std::ffi::OsString;

    /// Trace: FR-002-AC-1, root command surface.
    #[test]
    fn root_parser_recognizes_every_lifecycle_verb() {
        for verb in VERBS {
            let args = vec![OsString::from(verb)];
            assert!(
                matches!(parse_invocation(&args), Ok(Invocation::Operation { .. })),
                "{verb}"
            );
        }
    }

    /// Trace: FR-002-AC-1, inspect's graph format.
    #[test]
    fn inspect_accepts_dot_and_other_verbs_reject_it() {
        let inspect = vec![
            OsString::from("inspect"),
            OsString::from("--format"),
            OsString::from("dot"),
        ];
        assert!(matches!(
            parse_invocation(&inspect),
            Ok(Invocation::Operation {
                verb: Verb::Inspect,
                format: OutputFormat::Dot,
                ..
            })
        ));
        let check = vec![
            OsString::from("check"),
            OsString::from("--format"),
            OsString::from("dot"),
        ];
        assert!(parse_invocation(&check).is_err());
    }

    /// Trace: FR-002-AC-1
    #[test]
    fn accepts_shared_format_without_resolving_config_or_output() {
        let args = vec![
            OsString::from("check"),
            OsString::from("--format"),
            OsString::from("json"),
        ];
        assert!(matches!(
            parse_invocation(&args),
            Ok(Invocation::Operation {
                format: OutputFormat::Json,
                ..
            })
        ));
    }

    /// Trace: FR-002, preserve the raw operation arguments for handlers.
    #[test]
    fn retains_operation_arguments_at_the_dispatch_boundary() {
        let args = vec![
            OsString::from("run"),
            OsString::from("--limit"),
            OsString::from("steps=12"),
            OsString::from("unit.qsl"),
        ];
        assert!(matches!(
            parse_invocation(&args),
            Ok(Invocation::Operation { raw_arguments, .. }) if raw_arguments == args[1..]
        ));
    }

    /// Trace: FR-003-AC-1 / FR-004-AC-1, parse both options into the run call.
    #[test]
    fn run_options_keep_the_requested_engine_and_limits() {
        let args = vec![
            OsString::from("unit.qsl"),
            OsString::from("yes"),
            OsString::from("--engine"),
            OsString::from("interpreter"),
            OsString::from("--limit"),
            OsString::from("s1.input_bytes=1024"),
        ];
        let options = RunOptions::parse(&args).expect("valid run options");
        assert_eq!(options.path, "unit.qsl");
        assert_eq!(options.function, "yes");
        assert_eq!(options.engine, ExecutionEngine::Interpreter);
        assert_eq!(options.limits.values().get("s1.input_bytes"), Some(&1024));
    }

    /// Trace: FR-004-AC-2, an unknown engine fails before any source stage.
    #[test]
    fn run_options_refuse_unknown_engine_values() {
        let args = vec![
            OsString::from("unit.qsl"),
            OsString::from("yes"),
            OsString::from("--engine"),
            OsString::from("gpu"),
        ];
        assert!(RunOptions::parse(&args).is_err());
    }

    /// Trace: FR-003-AC-3, the bounded read includes input exactly at its limit.
    #[test]
    fn bounded_source_read_accepts_bytes_at_the_limit() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");
        let expected = std::fs::read(path).expect("fixture source");
        assert_eq!(
            read_source(path, expected.len()).expect("source at limit"),
            expected
        );
    }

    /// Trace: FR-003-AC-3, over-limit read buffers only the one-byte sentinel.
    #[test]
    fn bounded_source_read_captures_one_sentinel_byte_over_limit() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");
        let expected = std::fs::read(path).expect("fixture source");
        let limit = 8;
        assert!(
            expected.len() > limit * 10,
            "fixture exceeds the buffer bound"
        );
        let actual = read_source(path, limit).expect("source over limit");
        assert_eq!(
            actual.len(),
            limit + 1,
            "read is bounded to the limit plus one sentinel byte"
        );
        assert_eq!(actual, expected[..=limit]);
    }
}
