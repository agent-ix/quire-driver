// SPDX-License-Identifier: AGPL-3.0-or-later
//! Process-level checks of the `quire` frontend for `parse`, `format`, `run` and
//! `compile`: usage and arity refusals before I/O, invalid invocations, OS
//! encoding, stream write failures, closed pipes and path independence.

use std::process::{Command, Output, Stdio};

use quire_semantic_value::catalog::Category;

const QUIRE: &str = env!("CARGO_BIN_EXE_quire");
const SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/ir608-run.qsl");
const INVALID_SOURCE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/parse-invalid.qsl"
);

const REFUSAL: i32 = 20;
const TOOL_FAILURE: i32 = 30;

fn quire(arguments: &[&str]) -> Output {
    Command::new(QUIRE)
        .args(arguments)
        .output()
        .expect("run quire")
}

/// Assert a refusal that wrote nothing to stdout and did not panic.
fn assert_refused_quietly(output: &Output, context: &str) {
    assert_eq!(output.status.code(), Some(REFUSAL), "{context}");
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
        ("parse", &["parse", "missing.qsl", "extra"]),
        ("format", &["format", "missing.qsl", "extra"]),
    ];
    for (verb, arguments) in cases {
        let output = quire(arguments);
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(verb), "{arguments:?}: {stderr}");
        assert!(
            !stderr.contains("could not read"),
            "{arguments:?} opened its path: {stderr}"
        );
    }
    let output = quire(&["format"]);
    assert_refused_quietly(&output, "format without an operand");
}

/// Trace: FR-287-AC-5; each invalid invocation exits 20 without a panic, and the
/// same verb then runs normally on a valid invocation.
#[test]
fn invalid_invocations_exit_20_and_the_next_valid_call_runs() {
    let verbs: [(&[&str], &[&[&str]]); 4] = [
        (
            &["parse", SOURCE, "--format", "json"],
            &[
                &["parse", "--bogus", SOURCE],
                &["parse"],
                &["parse", SOURCE, "extra"],
                &["parse", SOURCE, "--format", "bogus"],
            ],
        ),
        (
            &["format", SOURCE],
            &[
                &["format", "--bogus", SOURCE],
                &["format"],
                &["format", SOURCE, "extra"],
            ],
        ),
        (
            &["compile", SOURCE, "--format", "json"],
            &[
                &["compile", "--bogus", SOURCE],
                &["compile"],
                &["compile", SOURCE, "extra"],
                &["compile", SOURCE, "--format", "bogus"],
            ],
        ),
        (
            &["run", SOURCE, "yes", "--format", "json"],
            &[
                &["run", "--bogus", SOURCE, "yes"],
                &["run", SOURCE],
                &["run", SOURCE, "yes", "extra"],
                &["run", SOURCE, "yes", "--limit"],
                &["run", SOURCE, "yes", "--limit", "no-equals-sign"],
                &["run", SOURCE, "yes", "--engine", "bogus"],
            ],
        ),
    ];
    for (valid, invalid) in verbs {
        for arguments in invalid {
            assert_refused_quietly(&quire(arguments), &format!("{arguments:?}"));
        }
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

/// Trace: FR-287-AC-6; a command word or operand that is not UTF-8 is refused
/// before the file operand is opened.
#[cfg(unix)]
#[test]
fn non_utf8_arguments_are_refused_before_the_file_is_opened() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let invalid = || OsString::from_vec(vec![0xff]);
    let cases: [Vec<OsString>; 6] = [
        vec![invalid(), "missing.qsl".into()],
        vec!["parse".into(), invalid()],
        vec!["parse".into(), "missing.qsl".into(), invalid()],
        vec!["format".into(), invalid()],
        vec!["compile".into(), invalid()],
        vec!["run".into(), "missing.qsl".into(), invalid()],
    ];
    for arguments in cases {
        let output = Command::new(QUIRE)
            .args(&arguments)
            .output()
            .expect("run quire with a non-UTF-8 argument");
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("UTF-8"), "{arguments:?}: {stderr}");
        assert!(
            !stderr.contains("could not read"),
            "{arguments:?}: {stderr}"
        );
    }
}

/// Trace: FR-287-AC-7; a failed stdout write exits 30 for every verb.
#[cfg(target_os = "linux")]
#[test]
fn stdout_write_failure_exits_30() {
    let cases: [&[&str]; 4] = [
        &["parse", SOURCE, "--format", "json"],
        &["format", SOURCE],
        &["compile", SOURCE, "--format", "json"],
        &["run", SOURCE, "yes", "--format", "json"],
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
        assert_eq!(output.status.code(), Some(TOOL_FAILURE), "{arguments:?}");
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
        (&["parse", SOURCE, "--format", "json"], success),
        (&["format", SOURCE], success),
        (&["compile", SOURCE, "--format", "json"], success),
        (&["run", SOURCE, "yes", "--format", "json"], success),
        // A refused outcome keeps its own non-zero code, distinguishing it from 30.
        (&["parse", INVALID_SOURCE, "--format", "json"], refusal),
    ];
    for (arguments, expected) in cases {
        let mut child = Command::new(QUIRE)
            .args(arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn quire");
        drop(child.stdout.take());
        let output = child.wait_with_output().expect("wait for quire");
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

/// Trace: FR-287-AC-9; the same source named by a relative and by an absolute
/// path from two working directories gives equal `run` and `compile` bytes and
/// exit codes.
#[test]
fn relative_and_absolute_source_paths_give_equal_run_and_compile_output() {
    let directory =
        std::env::temp_dir().join(format!("quire-frontend-paths-{}", std::process::id()));
    let other = directory.join("other");
    std::fs::create_dir_all(&other).expect("create working directories");
    std::fs::copy(SOURCE, directory.join("source.qsl")).expect("copy source fixture");

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
    let absolute = directory.join("source.qsl");
    let absolute = absolute.to_str().expect("UTF-8 temp path");
    for tail in [&["compile"][..], &["run", "yes"][..]] {
        let relative = run_in(&directory, "source.qsl", tail);
        let from_elsewhere = run_in(&other, absolute, tail);
        assert_eq!(relative.status.code(), Some(0), "{tail:?}");
        assert_eq!(
            relative.status.code(),
            from_elsewhere.status.code(),
            "{tail:?}"
        );
        assert!(!relative.stdout.is_empty(), "{tail:?}");
        assert_eq!(relative.stdout, from_elsewhere.stdout, "{tail:?}");
        assert_eq!(relative.stderr, from_elsewhere.stderr, "{tail:?}");
    }
    std::fs::remove_dir_all(&directory).expect("remove working directories");
}
