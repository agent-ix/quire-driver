// SPDX-License-Identifier: AGPL-3.0-or-later
//! Parser and command handlers for the user-facing `quire` binary.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::ExitCode;

use ix_cli_kit::exit::{caller_exit_code, RESERVED_COMMAND_NOT_FOUND};
use ix_cli_kit::streams::{self, Diagnostics};
use qsl_cst::format::{format_with_limits, FormatLimits, FormatRefusal};
use qsl_foundation::diagnostic::Staged;
use qsl_foundation::diagnostic::{Category, Code};
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    check, default_accounting, package, parse, select, Call, CallArgument, DependencyInput,
    LockEvidence, PackageLimits, ParseRequest, SpineLimits, DEFAULT_WORK_UNITS,
};
use qsl_replay::{Operation, OutcomeDiagnostic, OutcomeDocument};
use qsl_semantics::value::supplied::SuppliedLimits;
use quire_driver::{
    execute, ExecutionEngine, ExecutionFailure, ExecutionRequest, LimitOverrides, ObjectEnvironment,
};
use quire_exact::Cancel;
use quire_exact::{Integer, Value};

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

/// Write command output with one policy for closed pipes and output failures.
fn write_primary_output(bytes: &[u8], description: &str, success: Category) -> Category {
    match streams::write_primary_stdout(bytes) {
        Ok(()) => success,
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => success,
        Err(error) => {
            emit_diagnostic(
                "OutputFailure",
                &format!("OutputFailure: could not write {description}: {error}"),
            );
            Category::InternalFailure
        }
    }
}

fn version_exit_code() -> ExitCode {
    if category_exit_result(Category::Success) == RESERVED_COMMAND_NOT_FOUND {
        return process_exit_code(Category::Success);
    }

    let version = concat!(env!("CARGO_PKG_VERSION"), "\n");
    process_exit_code(write_primary_output(
        version.as_bytes(),
        "package version",
        Category::Success,
    ))
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
    provider_config: Option<std::path::PathBuf>,
}

struct CompileOptions {
    path: String,
}

struct CheckOptions {
    path: String,
}

struct ParseOptions {
    path: String,
}

struct FormatOptions {
    path: String,
}

impl FormatOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index]
                .to_str()
                .ok_or_else(|| "format arguments must be valid UTF-8".to_owned())?;
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument.starts_with('-') {
                return Err(format!("unknown format option `{argument}`"));
            }
            positionals.push(argument.to_owned());
            index += 1;
        }
        if positionals.len() != 1 {
            return Err("format requires exactly one source path".to_owned());
        }
        Ok(Self {
            path: positionals.remove(0),
        })
    }
}

impl ParseOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index]
                .to_str()
                .ok_or_else(|| "parse arguments must be valid UTF-8".to_owned())?;
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument.starts_with('-') {
                return Err(format!("unknown parse option `{argument}`"));
            }
            positionals.push(argument.to_owned());
            index += 1;
        }
        if positionals.len() != 1 {
            return Err("parse requires exactly one source path".to_owned());
        }
        Ok(Self {
            path: positionals.remove(0),
        })
    }
}

impl CompileOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index]
                .to_str()
                .ok_or_else(|| "compile arguments must be valid UTF-8".to_owned())?;
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument.starts_with('-') {
                return Err(format!("unknown compile option `{argument}`"));
            }
            positionals.push(argument.to_owned());
            index += 1;
        }
        if positionals.len() != 1 {
            return Err("compile requires exactly one source path".to_owned());
        }
        Ok(Self {
            path: positionals.remove(0),
        })
    }
}

impl CheckOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index]
                .to_str()
                .ok_or_else(|| "check arguments must be valid UTF-8".to_owned())?;
            if argument == "--format" {
                index += 2;
                continue;
            }
            if argument.starts_with('-') {
                return Err(format!("unknown check option `{argument}`"));
            }
            positionals.push(argument.to_owned());
            index += 1;
        }
        if positionals.len() != 1 {
            return Err("check requires exactly one source path".to_owned());
        }
        Ok(Self {
            path: positionals.remove(0),
        })
    }
}

impl RunOptions {
    fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut positionals = Vec::new();
        let mut limit_operands = Vec::new();
        let mut engine = None;
        let mut provider_config = None;
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
                "--provider-config" => {
                    index += 1;
                    let path = arguments
                        .get(index)
                        .filter(|value| !value.is_empty())
                        .ok_or_else(|| "--provider-config requires a path".to_owned())?;
                    if provider_config.replace(path.to_owned()).is_some() {
                        return Err("--provider-config may be supplied only once".to_owned());
                    }
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
                "true" => Value::Boolean(true),
                "false" => Value::Boolean(false),
                _ => Value::Integer(Integer::from(value.parse::<i64>().map_err(|_| {
                    format!("run argument `{argument}` is not a Boolean or integer")
                })?)),
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
            provider_config,
        })
    }
}

/// One command's QSL category, optional outcome document, and diagnostic.
struct CommandResult {
    category: Category,
    document: Option<OutcomeDocument>,
    diagnostic: Option<String>,
    text_output: Option<String>,
}

impl CommandResult {
    fn document(document: OutcomeDocument) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
            text_output: None,
        }
    }

    fn failure(category: Category, message: impl Into<String>) -> Self {
        Self {
            category,
            document: None,
            diagnostic: Some(message.into()),
            text_output: None,
        }
    }

    fn formatted(document: OutcomeDocument, text_output: Option<String>) -> Self {
        Self {
            category: document.category(),
            document: Some(document),
            diagnostic: None,
            text_output,
        }
    }

    /// An input refusal raised before QSL runs, recorded with QSL's native code.
    fn input_refusal(operation: Operation, message: impl Into<String>) -> Self {
        Self::catalog_failure_for(operation, Code::InvalidRequest, message)
    }

    fn catalog_failure(code: Code, message: impl Into<String>) -> Self {
        Self::catalog_failure_for(Operation::Execute, code, message)
    }

    fn catalog_failure_for(operation: Operation, code: Code, message: impl Into<String>) -> Self {
        let message = message.into();
        Self::document(
            OutcomeDocument::new(operation, None, code.category()).with_diagnostics(vec![
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
    let provider_config = match options.provider_config.as_deref() {
        Some(path) => match crate::provider_config::load(path) {
            Ok(config) => config,
            Err(error) => {
                return CommandResult::input_refusal(
                    Operation::Execute,
                    format!("invalid process-provider configuration: {error}"),
                );
            }
        },
        None => crate::provider_config::ProviderConfig::default(),
    };
    if !provider_config.providers.is_empty() {
        return CommandResult::catalog_failure(
            Code::UnimplementedCapability,
            "process-provider dispatch is not available in this command path",
        );
    }
    if options.engine != ExecutionEngine::Interpreter {
        let failure = ExecutionFailure::UnsupportedEngine(options.engine);
        return CommandResult::document(failure.into_document());
    }
    let packages = BTreeMap::new();
    let dependencies = DependencyInput::default();
    let mut limits = SpineLimits::default();
    let mut call = Call {
        function: options.function,
        arguments: options.arguments,
        accounting: default_accounting(DEFAULT_WORK_UNITS),
        supplied: SuppliedLimits::default(),
    };
    if let Err(error) = options.limits.apply_to(&mut limits, &mut call) {
        return CommandResult::input_refusal(Operation::Execute, error.to_string());
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
        object_environment: &ObjectEnvironment::default(),
        cancel: Cancel::new(),
        engine: options.engine,
    }) {
        Ok(result) => CommandResult::document(result.document),
        Err(failure) => CommandResult::document(failure.into_document()),
    }
}

/// Compile one source through QSL's typed spine and serialize its package outcome.
fn compile(options: CompileOptions) -> CommandResult {
    let limits = SpineLimits::default();
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Package, Code::IoError, message);
        }
    };
    let source = SourceIdentity::new("quire", &options.path, "filesystem", "current");
    let dependencies = DependencyInput::default();
    let packages = BTreeMap::new();
    let cancel = Cancel::new();
    let result = parse(
        &ParseRequest {
            source: &source,
            path: &options.path,
            bytes: &bytes,
        },
        limits.source,
        &cancel,
    )
    .and_then(|parsed| {
        select(parsed.value(), &packages, limits.model, &cancel).and_then(|models| {
            check(
                parsed.value(),
                models.value(),
                &dependencies,
                &LockEvidence::default(),
                limits,
                &cancel,
            )
            .and_then(|checked| package(checked.value(), PackageLimits::default(), &cancel))
        })
    });
    CommandResult::document(OutcomeDocument::from_package(&result))
}

/// Check one source through QSL's typed front-end and serialize its S4 outcome.
fn check_source(options: CheckOptions) -> CommandResult {
    let limits = SpineLimits::default();
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Check, Code::IoError, message);
        }
    };
    let source = SourceIdentity::new("quire", &options.path, "filesystem", "current");
    let packages = BTreeMap::new();
    let cancel = Cancel::new();
    #[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
    tc018_record_check_request(&options, &source, &bytes, limits);
    let result = parse(
        &ParseRequest {
            source: &source,
            path: &options.path,
            bytes: &bytes,
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
    CommandResult::document(OutcomeDocument::from_check(&result))
}

/// Observe the TC-018 refused-range selection at the typed check boundary.
/// The normal QSL parse/select/check below supplies the result; argv parsing,
/// source reading, serialization and exit mapping remain the production paths.
#[cfg(all(feature = "tc018-test-adapter", debug_assertions))]
fn tc018_record_check_request(
    options: &CheckOptions,
    source: &SourceIdentity,
    bytes: &[u8],
    limits: SpineLimits,
) {
    if std::env::var_os("TC018_CASE").as_deref() != Some(std::ffi::OsStr::new("refused-range")) {
        return;
    }

    if let Some(observation_path) = std::env::var_os("TC018_REQUEST_PATH") {
        let profile = bytes
            .split(|byte| *byte == b'\n')
            .find(|line| line.starts_with(b"profile "))
            .map(String::from_utf8_lossy);
        let observation = serde_json::json!({
            "case": "refused-range",
            "source_identity": source,
            "path": options.path,
            "profile": profile.map(|line| line.into_owned()),
            "limits": format!("{limits:?}"),
            "source_bytes": bytes,
        });
        if let Ok(encoded) = serde_json::to_vec(&observation) {
            let _ = std::fs::write(observation_path, encoded);
        }
    }
}

/// Parse one source through QSL and serialize its typed parse outcome.
fn parse_source(options: ParseOptions) -> CommandResult {
    let limits = SpineLimits::default();
    let bytes = match read_source(&options.path, limits.source.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Parse, Code::IoError, message);
        }
    };
    let source = SourceIdentity::new("quire", &options.path, "filesystem", "current");
    let result = parse(
        &ParseRequest {
            source: &source,
            path: &options.path,
            bytes: &bytes,
        },
        limits.source,
        &Cancel::new(),
    );
    CommandResult::document(OutcomeDocument::from_parse(&result))
}

/// Format one parsed source through QSL and retain its typed outcome.
fn format_source(options: FormatOptions) -> CommandResult {
    let limits = qsl_cst::Limits::default();
    let bytes = match read_source(&options.path, limits.source_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("could not read source {}: {error}", options.path);
            return CommandResult::catalog_failure_for(Operation::Format, Code::IoError, message);
        }
    };
    let source = SourceIdentity::new("quire", &options.path, "filesystem", "current");
    let cancel = Cancel::new();
    let result = qsl_cst::parse_with_cancel(source, &options.path, &bytes, limits, &cancel)
        .map_err(FormatRefusal::DiagnosedSource)
        .and_then(|parsed| format_with_limits(&parsed, FormatLimits::default()).map(Staged::new));
    let document = OutcomeDocument::from_format(&result);
    let text_output = result.ok().map(Staged::into_value);
    CommandResult::formatted(document, text_output)
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
    format: OutputFormat,
    raw_arguments: &[OsString],
    _signals: crate::signals::SignalContext,
) -> CommandResult {
    if verb == Verb::Run {
        return match RunOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Execute, "run")
            }
            Ok(options) => run(options),
            Err(error) => CommandResult::input_refusal(Operation::Execute, error),
        };
    }
    if verb == Verb::Compile {
        return match CompileOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Package, "compile")
            }
            Ok(options) => compile(options),
            Err(error) => CommandResult::input_refusal(Operation::Package, error),
        };
    }
    if verb == Verb::Check {
        return match CheckOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Check, "check")
            }
            Ok(options) => check_source(options),
            Err(error) => CommandResult::input_refusal(Operation::Check, error),
        };
    }
    if verb == Verb::Parse {
        return match ParseOptions::parse(raw_arguments) {
            Ok(_options) if format == OutputFormat::Text => {
                text_rendering_unavailable(Operation::Parse, "parse")
            }
            Ok(options) => parse_source(options),
            Err(error) => CommandResult::input_refusal(Operation::Parse, error),
        };
    }
    if verb == Verb::Format {
        return match FormatOptions::parse(raw_arguments) {
            Ok(options) => format_source(options),
            Err(error) => CommandResult::input_refusal(Operation::Format, error),
        };
    }
    if raw_arguments.iter().any(|argument| {
        matches!(
            argument.to_str(),
            Some("--engine" | "--limit" | "--provider-config")
        )
    }) {
        let option = raw_arguments
            .iter()
            .find_map(|argument| match argument.to_str() {
                Some(option @ ("--engine" | "--limit" | "--provider-config")) => Some(option),
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
        text_output: None,
    }
}

/// Refuse a text selection whose QSL outcome renderer is not available.
/// Machine mode remains fully QSL-serialized; the diagnostic includes the
/// native QSL code because text mode has no outcome-document renderer.
fn text_rendering_unavailable(operation: Operation, verb: &str) -> CommandResult {
    let code = Code::UnimplementedCapability;
    CommandResult::catalog_failure_for(
        operation,
        code,
        format!(
            "text rendering for `{verb}` is unavailable ({})",
            code.as_str()
        ),
    )
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
        if let Some(text) = &result.text_output {
            return write_primary_output(text.as_bytes(), "formatted source", result.category);
        }
        return result.category;
    }
    match ix_cli_kit::json::encode_canonical(document, false) {
        Ok(mut encoded) => {
            encoded.push('\n');
            write_primary_output(encoded.as_bytes(), "outcome document", result.category)
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
                if format == OutputFormat::Text {
                    let mut stderr = io::stderr().lock();
                    let _ = write_result_diagnostics(&result, &mut stderr);
                }
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

    /// Trace: FR-002-AC-2, FR-002-AC-4; one QSL exit result reaches process exit unchanged through the kit.
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
        assert_eq!(
            options.limits.values(),
            &[(
                qsl_foundation::SettingName::Stage(qsl_foundation::Setting::S1InputBytes),
                1024,
            )]
        );
        assert_eq!(options.provider_config, None);
    }

    /// Trace: FR-006-AC-24, FR-014-AC-6; accept only one explicitly named config path.
    #[test]
    fn run_options_accept_one_provider_config_path() {
        let args = vec![
            OsString::from("unit.qsl"),
            OsString::from("yes"),
            OsString::from("--provider-config"),
            OsString::from("providers.json"),
        ];
        let options = RunOptions::parse(&args).expect("valid provider config path");
        assert_eq!(
            options.provider_config.as_deref(),
            Some(std::path::Path::new("providers.json"))
        );
    }

    /// Trace: FR-006-AC-24, FR-014-AC-6; malformed caller argv is rejected before loading.
    #[test]
    fn run_options_reject_missing_or_duplicate_provider_config_path() {
        let missing = vec![
            OsString::from("unit.qsl"),
            OsString::from("yes"),
            OsString::from("--provider-config"),
        ];
        assert!(RunOptions::parse(&missing)
            .err()
            .expect("provider config path is required")
            .contains("--provider-config requires a path"));

        let duplicate = vec![
            OsString::from("unit.qsl"),
            OsString::from("yes"),
            OsString::from("--provider-config"),
            OsString::from("one.json"),
            OsString::from("--provider-config"),
            OsString::from("two.json"),
        ];
        assert!(RunOptions::parse(&duplicate)
            .err()
            .expect("provider config may be selected once")
            .contains("only once"));
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
