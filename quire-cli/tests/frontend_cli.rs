// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process-level checks of the `quire` frontend for `parse`, `format`, `run` and
//! `compile`: usage and arity refusals before I/O, invalid invocations, OS
//! encoding, stream write failures, closed pipes and path independence.

use std::process::{Command, Output, Stdio};

use ix_trace_rs::trace;
use quire_semantic_value::catalog::Category;

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");
const INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/parse-invalid.qsl"
);
const MISSING_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/missing-source.qsl"
);

fn refusal_code() -> i32 {
    i32::from(Category::Refusal.exit_code())
}

fn tool_failure_code() -> i32 {
    i32::from(Category::InternalFailure.exit_code())
}

fn quire(arguments: &[&str]) -> Output {
    Command::new(QUIRE)
        .args(arguments)
        .output()
        .expect("run quire")
}

/// Assert a refusal that wrote nothing to stdout and did not panic.
fn assert_refused_quietly(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(refusal_code()), "{context}");
    assert!(output.stdout.is_empty(), "{context}: stdout must be empty");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "{context}: refusal names the problem");
    assert!(!stderr.contains("panicked"), "{context}: {stderr}");
}

/// Trace: FR-287-AC-3; a missing or extra operand is refused naming the verb,
/// before the nonexistent path operand is opened.
#[test]
fn missing_and_extra_operands_are_refused_before_any_file_is_read() {
    let cases: [(&str, &[&str]); 8] = [
        ("run", &["run"]),
        ("run", &["run", "missing.qsl"]),
        ("run", &["run", "missing.qsl", "yes", "extra"]),
        ("compile", &["compile"]),
        ("compile", &["compile", "missing.qsl", "extra"]),
        ("parse", &["parse"]),
        ("parse", &["parse", "issuer", "document", "missing.qsl", "extra"]),
        ("format", &["format", "issuer", "document", "missing.qsl", "extra"]),
    ];
    for (verb, arguments) in cases {
        let output = quire(arguments);
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.starts_with(&format!("usage: quire {verb} ")),
            "{arguments:?}: {stderr}"
        );
        assert!(
            !stderr.contains("could not read"),
            "{arguments:?} opened its path: {stderr}"
        );
    }
    let output = quire(&["format"]);
    assert_refused_quietly(&output, "format without an operand");
    for verb in ["parse", "format"] {
        for (authority, identity, label) in [
            ("", "document", "authority"),
            ("issuer", "\u{3000}", "identity"),
            (" ", "", "authority"),
        ] {
            let output = quire(&[
                verb, authority, identity, MISSING_SOURCE, "--format", "json",
            ]);
            assert_eq!(output.status.code(), Some(refusal_code()));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("could not read"));
            let document: serde_json::Value = serde_json::from_slice(&output.stdout)
                .expect("typed source identity refusal");
            let diagnostic = &document["diagnostics"][0];
            assert_eq!(diagnostic["code"], "invalid_source_identity");
            assert_eq!(diagnostic["cause"], "blank-label");
            assert_eq!(diagnostic["label"], label);
            assert_eq!(diagnostic["locus"], serde_json::Value::Null);
        }
    }
}

#[test]
fn invalid_invocations_exit_20_and_the_next_valid_call_runs() {
    type InvalidCase<'a> = (&'a [&'a str], &'a str);
    assert!(
        !std::path::Path::new(MISSING_SOURCE).exists(),
        "the invalid invocation cases require a nonexistent source"
    );
    let verbs: [(&[&str], &[InvalidCase<'_>]); 4] = [
        (
            &["parse", "issuer", "document", SOURCE, "--format", "json"],
            &[
                (
                    &["parse", "--bogus", MISSING_SOURCE],
                    "unknown parse option `--bogus`",
                ),
                (&["parse"], "usage: quire parse "),
                (&["parse", "issuer", "document", MISSING_SOURCE, "extra"], "usage: quire parse "),
                (
                    &["parse", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["parse", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
            ],
        ),
        (
            &["format", "issuer", "document", SOURCE],
            &[
                (
                    &["format", "--bogus", MISSING_SOURCE],
                    "unknown format option `--bogus`",
                ),
                (&["format"], "usage: quire format "),
                (&["format", "issuer", "document", MISSING_SOURCE, "extra"], "usage: quire format "),
                (
                    &["format", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["format", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
            ],
        ),
        (
            &["compile", SOURCE, "--format", "json"],
            &[
                (
                    &["compile", "--bogus", MISSING_SOURCE],
                    "unknown compile option `--bogus`",
                ),
                (&["compile"], "usage: quire compile "),
                (
                    &["compile", MISSING_SOURCE, "extra"],
                    "usage: quire compile ",
                ),
                (
                    &["compile", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["compile", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
            ],
        ),
        (
            &["run", SOURCE, "yes", "--format", "json"],
            &[
                (
                    &["run", "--bogus", MISSING_SOURCE, "yes"],
                    "unknown run option `--bogus`",
                ),
                (&["run", MISSING_SOURCE], "usage: quire run "),
                (
                    &["run", MISSING_SOURCE, "yes", "extra"],
                    "run argument `extra` must be parameter=value",
                ),
                (
                    &["run", MISSING_SOURCE, "yes", "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["run", MISSING_SOURCE, "yes", "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
                (
                    &["run", MISSING_SOURCE, "yes", "--limit"],
                    "--limit requires name=value",
                ),
                (
                    &["run", MISSING_SOURCE, "yes", "--limit", "no-equals-sign"],
                    "NotAnAssignment",
                ),
                (
                    &["run", MISSING_SOURCE, "yes", "--engine", "bogus"],
                    "unknown engine `bogus`",
                ),
            ],
        ),
    ];
    for (valid, invalid) in verbs {
        for (arguments, expected_error) in invalid {
            let output = quire(arguments);
            assert_refused_quietly(&output, &format!("{arguments:?}"));
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains(expected_error), "{arguments:?}: {stderr}");
            assert!(
                !stderr.contains("could not read"),
                "{arguments:?} opened its path: {stderr}"
            );
            let output = quire(valid);
            assert_eq!(
                output.status.code(),
                Some(i32::from(Category::Success.exit_code())),
                "{valid:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!output.stdout.is_empty(), "{valid:?}");
        }
    }
}

#[cfg(unix)]
fn invalid_os_string() -> std::ffi::OsString {
    use std::os::unix::ffi::OsStringExt;
    std::ffi::OsString::from_vec(vec![0xff])
}

#[cfg(windows)]
fn invalid_os_string() -> std::ffi::OsString {
    use std::os::windows::ffi::OsStringExt;
    std::ffi::OsString::from_wide(&[0xd800])
}

#[cfg(any(unix, windows))]
#[trace("TC-1262", "FR-287-AC-6")]
#[test]
fn non_utf8_commands_and_labels_refuse_but_raw_file_paths_parse() {
    use std::ffi::OsString;

    let mut cases = vec![vec![invalid_os_string(), MISSING_SOURCE.into()]];
    for verb in ["parse", "format"] {
        cases.push(vec![
            verb.into(), invalid_os_string(), "document".into(), MISSING_SOURCE.into(),
        ]);
        cases.push(vec![
            verb.into(), "issuer".into(), invalid_os_string(), MISSING_SOURCE.into(),
        ]);
    }
    for arguments in cases {
        let output = Command::new(QUIRE).args(&arguments).output()
            .expect("run quire with invalid command or label encoding");
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("UTF-8"), "{arguments:?}: {stderr}");
        assert!(!stderr.contains("could not read"), "{arguments:?}: {stderr}");
    }
    let directory = TestDirectory(std::env::temp_dir().join(format!(
        "quire-frontend-raw-path-{}", std::process::id()
    )));
    std::fs::create_dir_all(&directory.0).expect("create raw-path directory");
    let mut filename = OsString::from("source-");
    filename.push(invalid_os_string());
    filename.push(".qsl");
    let path = directory.0.join(filename);
    std::fs::copy(SOURCE, &path).expect("create source at raw OS path");
    let output = Command::new(QUIRE)
        .args(["parse", "issuer", "document"])
        .arg(&path).args(["--format", "json"]).output()
        .expect("parse raw OS path");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stderr.is_empty());
    let document: serde_json::Value = serde_json::from_slice(&output.stdout)
        .expect("typed parse outcome");
    assert_eq!(document["category"], "success");
}

/// Trace: FR-287-AC-7; a failed stdout write exits 30 for every verb.
#[cfg(target_os = "linux")]
#[test]
fn stdout_write_failure_exits_30() {
    let cases: [&[&str]; 5] = [
        &["parse", "issuer", "document", SOURCE, "--format", "json"],
        &["format", "issuer", "document", SOURCE],
        &["compile", SOURCE, "--format", "json"],
        &["run", SOURCE, "yes", "--format", "json"],
        // A refused outcome document is written to the same stream.
        &["parse", "issuer", "document", INVALID_SOURCE, "--format", "json"],
    ];
    for arguments in cases {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("open /dev/full");
        let output = Command::new(QUIRE)
            .args(arguments)
            .stdout(Stdio::from(full))
            .output()
            .expect("run quire with a full stdout");
        assert_eq!(
            output.status.code(),
            Some(tool_failure_code()),
            "{arguments:?}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("OutputFailure"),
            "{arguments:?}"
        );
    }
}

/// Trace: FR-287-AC-8; a closed stdout reader ends the run quietly with the
/// outcome's own exit code, not 30.
#[cfg(unix)]
#[test]
fn closed_stdout_pipe_keeps_the_outcome_exit_code_and_stays_quiet() {
    let success = i32::from(Category::Success.exit_code());
    let refusal = i32::from(Category::Refusal.exit_code());
    let cases: [(&[&str], i32); 5] = [
        (&["parse", "issuer", "document", SOURCE, "--format", "json"], success),
        (&["format", "issuer", "document", SOURCE], success),
        (&["compile", SOURCE, "--format", "json"], success),
        (&["run", SOURCE, "yes", "--format", "json"], success),
        // A refused outcome keeps its own non-zero code, distinguishing it from 30.
        (&["parse", "issuer", "document", INVALID_SOURCE, "--format", "json"], refusal),
    ];
    for (arguments, expected) in cases {
        // The reader is gone before the child exists, so its first write
        // always meets a closed pipe.
        let (reader, writer) = std::io::pipe().expect("create stdout pipe");
        drop(reader);
        let output = Command::new(QUIRE)
            .args(arguments)
            .stdout(Stdio::from(writer))
            .output()
            .expect("run quire with a closed stdout reader");
        assert_eq!(output.status.code(), Some(expected), "{arguments:?}");
        if expected == success {
            assert!(
                output.stderr.is_empty(),
                "{arguments:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            assert!(
                !String::from_utf8_lossy(&output.stderr).contains("OutputFailure"),
                "{arguments:?}"
            );
        }
    }
}

/// Removes its directory when dropped, including after a failed assertion.
struct TestDirectory(std::path::PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Trace: FR-287-AC-9; the same source named by a relative and by an absolute
/// path from two working directories gives equal `run` and `compile` bytes and
/// exit codes, for successful and for refused calls.
#[test]
fn relative_and_absolute_source_paths_give_equal_run_and_compile_output() {
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("quire-frontend-paths-{}", std::process::id())),
    );
    let other = directory.0.join("other");
    std::fs::create_dir_all(&other).expect("create working directories");
    std::fs::copy(SOURCE, directory.0.join("source.qsl")).expect("copy source fixture");

    let run_in = |working_directory: &std::path::Path, path: &str, tail: &[&str]| {
        Command::new(QUIRE)
            .current_dir(working_directory)
            .arg(tail[0])
            .arg(path)
            .args(&tail[1..])
            .args(["--format", "json"])
            .output()
            .expect("run quire")
    };
    let absolute = directory.0.join("source.qsl");
    let absolute = absolute.to_str().expect("UTF-8 temp path");
    let cases: [(&[&str], i32); 3] = [
        (&["compile"], 0),
        (&["run", "yes"], 0),
        // A refused call: the function does not exist.
        (&["run", "no_such_function"], refusal_code()),
    ];
    for (tail, expected) in cases {
        let relative = run_in(&directory.0, "source.qsl", tail);
        let from_elsewhere = run_in(&other, absolute, tail);
        assert_eq!(relative.status.code(), Some(expected), "{tail:?}");
        assert_eq!(
            relative.status.code(),
            from_elsewhere.status.code(),
            "{tail:?}"
        );
        assert!(!relative.stdout.is_empty(), "{tail:?}");
        assert_eq!(relative.stdout, from_elsewhere.stdout, "{tail:?}");
        assert_eq!(relative.stderr, from_elsewhere.stderr, "{tail:?}");
    }
}
