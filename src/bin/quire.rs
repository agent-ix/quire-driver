// SPDX-License-Identifier: AGPL-3.0-or-later
//! Initial command shell for the one driver-owned `quire` binary.

use std::ffi::OsString;
use std::process::ExitCode;

use qsl_foundation::diagnostic::Category;
use quire_driver::cli::process_exit_code;

#[cfg(test)]
const VERBS: &[&str] = &[
    "parse", "format", "check", "compile", "run", "analyze", "monitor", "prove", "replay",
    "generate", "inspect", "render",
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum OutputFormat {
    #[default]
    Text,
    Json,
    Dot,
}

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
    Operation { verb: Verb, format: OutputFormat },
}

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
    Ok(Invocation::Operation { verb, format })
}

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().skip(1).take(64).collect();
    match parse_invocation(&arguments) {
        Ok(Invocation::Help | Invocation::Version) => process_exit_code(Category::Success),
        Ok(Invocation::Operation { verb, format }) => {
            let _ = verb;
            let _ = format;
            process_exit_code(Category::Unsupported)
        }
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
    use super::{parse_invocation, Invocation, OutputFormat, Verb, VERBS};
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
                format: OutputFormat::Dot
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
}
