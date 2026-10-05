// SPDX-License-Identifier: AGPL-3.0-or-later
//! Initial command shell for the one driver-owned `quire` binary.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::process::ExitCode;

use qsl_foundation::diagnostic::Category;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{
    default_accounting, Call, CallArgument, DependencyInput, SpineLimits, DEFAULT_WORK_UNITS,
};
use quire_driver::cli::process_exit_code;
use quire_driver::{execute, ExecutionEngine, ExecutionFailure, ExecutionRequest, LimitOverrides};

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

/// Run one complete-V1 function through QSL's interpreter.
fn run(options: RunOptions) -> Category {
    if options.engine != ExecutionEngine::Interpreter {
        let failure = ExecutionFailure::UnsupportedEngine(options.engine);
        eprintln!("{failure}");
        return failure.category();
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
        eprintln!("{error}");
        return Category::Refusal;
    }
    let Ok(bytes) = read_source(&options.path, limits.source.source_bytes) else {
        return Category::Refusal;
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
        Ok(result) => result.outcome.category(),
        Err(failure) => failure.category(),
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
fn dispatch(verb: Verb, _format: OutputFormat, raw_arguments: &[OsString]) -> Category {
    if verb == Verb::Run {
        return match RunOptions::parse(raw_arguments) {
            Ok(options) => run(options),
            Err(error) => {
                eprintln!("{error}");
                Category::Refusal
            }
        };
    }
    if raw_arguments
        .iter()
        .any(|argument| matches!(argument.to_str(), Some("--engine" | "--limit")))
    {
        return Category::Refusal;
    }
    Category::Unsupported
}

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    match parse_invocation(&arguments) {
        Ok(Invocation::Help | Invocation::Version) => process_exit_code(Category::Success),
        Ok(Invocation::Operation {
            verb,
            format,
            raw_arguments,
        }) => process_exit_code(dispatch(verb, format, &raw_arguments)),
        Err(_) => process_exit_code(Category::Refusal),
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

    /// Trace: FR-002-AC-4, shared options remain a kit adoption seam.
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
        let limit = expected.len() - 1;
        let actual = read_source(path, limit).expect("source over limit");
        assert_eq!(
            actual.len(),
            limit + 1,
            "read is bounded to the limit plus one sentinel byte"
        );
        assert_eq!(actual, expected);
    }
}
