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

const RUN_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-run.json"
);
const COMPILE_REQUEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-compile.json"
);
const RUN_REFUSED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-run-refused.json"
);
const COMPILE_REFUSED: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/frontend-compile-refused.json"
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

#[trace("TC-1262", "FR-287-AC-3")]
#[test]
fn missing_and_extra_operands_are_refused_before_any_file_is_read() {
    let cases: [(&str, &[&str]); 8] = [
        ("run", &["run"]),
        ("run", &["run", "missing.json", "extra"]),
        ("run", &["run", "missing.qsl", "yes", "extra"]),
        ("compile", &["compile"]),
        ("compile", &["compile", "missing.qsl", "extra"]),
        ("parse", &["parse", "issuer", "document"]),
        (
            "parse",
            &["parse", "issuer", "document", "missing.qsl", "extra"],
        ),
        (
            "format",
            &["format", "issuer", "document", "missing.qsl", "extra"],
        ),
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
    let output = quire(&["format", "issuer", "document"]);
    assert_refused_quietly(&output, "format without an operand");
    for verb in ["parse", "format"] {
        for (authority, identity, label) in [
            ("", "document", "authority"),
            ("issuer", "\u{3000}", "identity"),
            (" ", "", "authority"),
        ] {
            let output = quire(&[
                verb,
                authority,
                identity,
                MISSING_SOURCE,
                "--format",
                "json",
            ]);
            assert_eq!(output.status.code(), Some(refusal_code()));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("could not read"));
            let document: serde_json::Value =
                serde_json::from_slice(&output.stdout).expect("typed source identity refusal");
            let diagnostic = &document["diagnostics"][0];
            assert_eq!(diagnostic["code"], "invalid_source_identity");
            assert_eq!(diagnostic["cause"], "blank-label");
            assert_eq!(diagnostic["label"], label);
            assert_eq!(diagnostic["locus"], serde_json::Value::Null);
        }
    }
}

#[trace("TC-1262", "FR-287-AC-5")]
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
                (
                    &["parse", "issuer", "document", MISSING_SOURCE, "extra"],
                    "usage: quire parse ",
                ),
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
                (
                    &["format", "issuer", "document", MISSING_SOURCE, "extra"],
                    "usage: quire format ",
                ),
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
            &["compile", COMPILE_REQUEST, "--format", "json"],
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
            &["run", RUN_REQUEST, "--format", "json"],
            &[
                (
                    &["run", "--bogus", MISSING_SOURCE, "yes"],
                    "unknown run option `--bogus`",
                ),
                (&["run"], "usage: quire run "),
                (
                    &["run", MISSING_SOURCE, "yes", "extra"],
                    "usage: quire run ",
                ),
                (
                    &["run", MISSING_SOURCE, "--format"],
                    "--format requires text, json, or dot",
                ),
                (
                    &["run", MISSING_SOURCE, "--format", "bogus"],
                    "unknown output format `bogus`",
                ),
                (
                    &["run", MISSING_SOURCE, "--limit"],
                    "--limit requires name=value",
                ),
                (
                    &["run", MISSING_SOURCE, "--limit", "no-equals-sign"],
                    "NotAnAssignment",
                ),
                (
                    &["run", MISSING_SOURCE, "--engine", "bogus"],
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
            verb.into(),
            invalid_os_string(),
            "document".into(),
            MISSING_SOURCE.into(),
        ]);
        cases.push(vec![
            verb.into(),
            "issuer".into(),
            invalid_os_string(),
            MISSING_SOURCE.into(),
        ]);
    }
    for arguments in cases {
        let output = Command::new(QUIRE)
            .args(&arguments)
            .output()
            .expect("run quire with invalid command or label encoding");
        assert_refused_quietly(&output, &format!("{arguments:?}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("UTF-8"), "{arguments:?}: {stderr}");
        assert!(
            !stderr.contains("could not read"),
            "{arguments:?}: {stderr}"
        );
    }
    let directory = TestDirectory(
        std::env::temp_dir().join(format!("quire-frontend-raw-path-{}", std::process::id())),
    );
    std::fs::create_dir_all(&directory.0).expect("create raw-path directory");
    let mut filename = OsString::from("source-");
    filename.push(invalid_os_string());
    filename.push(".qsl");
    let path = directory.0.join(filename);
    std::fs::copy(SOURCE, &path).expect("create source at raw OS path");
    let output = Command::new(QUIRE)
        .args(["parse", "issuer", "document"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("parse raw OS path");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("typed parse outcome");
    assert_eq!(document["category"], "success");
}

#[trace("TC-1262", "FR-287-AC-7")]
#[cfg(target_os = "linux")]
#[test]
fn stdout_write_failure_exits_30() {
    let cases: [&[&str]; 6] = [
        &["parse", "issuer", "document", SOURCE, "--format", "json"],
        &["format", "issuer", "document", SOURCE],
        &["format", "issuer", "document", SOURCE, "--format", "json"],
        &["compile", COMPILE_REQUEST, "--format", "json"],
        &["run", RUN_REQUEST, "--format", "json"],
        // A refused outcome document is written to the same stream.
        &[
            "parse",
            "issuer",
            "document",
            INVALID_SOURCE,
            "--format",
            "json",
        ],
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

#[trace("TC-1262", "FR-287-AC-8")]
#[cfg(unix)]
#[test]
fn closed_stdout_pipe_keeps_the_outcome_exit_code_and_stays_quiet() {
    let success = i32::from(Category::Success.exit_code());
    let refusal = i32::from(Category::Refusal.exit_code());
    let cases: [(&[&str], i32); 9] = [
        (
            &["parse", "issuer", "document", SOURCE, "--format", "json"],
            success,
        ),
        (&["format", "issuer", "document", SOURCE], success),
        (
            &["format", "issuer", "document", SOURCE, "--format", "json"],
            success,
        ),
        (
            &[
                "format",
                "issuer",
                "document",
                INVALID_SOURCE,
                "--format",
                "json",
            ],
            refusal,
        ),
        (&["compile", COMPILE_REQUEST, "--format", "json"], success),
        (&["run", RUN_REQUEST, "--format", "json"], success),
        (&["run", RUN_REFUSED, "--format", "json"], refusal),
        (&["compile", COMPILE_REFUSED, "--format", "json"], refusal),
        // A refused outcome keeps its own non-zero code, distinguishing it from 30.
        (
            &[
                "parse",
                "issuer",
                "document",
                INVALID_SOURCE,
                "--format",
                "json",
            ],
            refusal,
        ),
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
        assert!(
            output.stderr.is_empty(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Removes its directory when dropped, including after a failed assertion.
struct TestDirectory(std::path::PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[trace("TC-1262", "FR-287-AC-9")]
#[test]
fn request_paths_are_independent_and_documents_equal_the_actual_producers() {
    use qsl_command::command::{compile_outcome, prepare_run, PreparedRun, RunLimits};
    use qsl_replay::Operation;
    use quire_driver::{execute, ExecutionEngine, ExecutionRequest, ObjectEnvironment};
    let fixtures = std::path::Path::new(RUN_REQUEST)
        .parent()
        .expect("fixture directory");
    let elsewhere = std::env::temp_dir();
    for (verb, request) in [
        ("compile", COMPILE_REQUEST),
        ("compile", COMPILE_REFUSED),
        ("run", RUN_REQUEST),
        ("run", RUN_REFUSED),
    ] {
        let document = if verb == "compile" {
            match compile_outcome(std::path::Path::new(request)) {
                Ok(outcome) => outcome.document().clone(),
                Err(error) => error.outcome_document(Operation::Package),
            }
        } else {
            let PreparedRun::Function(prepared) =
                prepare_run(std::path::Path::new(request), RunLimits::default())
                    .expect("prepare request")
            else {
                panic!("fixture selects a function");
            };
            let input = prepared.into_input();
            let result = execute(ExecutionRequest {
                source: input.source.identity().clone(),
                package_name: &input.package_name,
                path: input.source.path(),
                bytes: input.source.text().as_bytes(),
                packages: &input.packages,
                dependencies: &input.dependencies,
                limits: qsl_replay::spine::SpineLimits::default(),
                call: input.call,
                object_environment: &ObjectEnvironment::default(),
                cancel: quire_exact::Cancel::new(),
                engine: ExecutionEngine::Interpreter,
            });
            match result {
                Ok(result) => result.document,
                Err(error) => error.into_document(),
            }
        };
        let expected = format!(
            "{}\n",
            ix_cli_kit::json::encode_canonical(&document, false).expect("producer document")
        );
        let path = std::path::Path::new(request);
        let relative = Command::new(QUIRE)
            .current_dir(fixtures)
            .arg(verb)
            .arg(path.file_name().expect("request name"))
            .args(["--format", "json"])
            .output()
            .expect("relative request");
        let absolute = Command::new(QUIRE)
            .current_dir(&elsewhere)
            .args([verb, request, "--format", "json"])
            .output()
            .expect("absolute request");
        assert_eq!(
            relative.status.code(),
            Some(i32::from(document.category().exit_code())),
            "{request}"
        );
        assert_eq!(relative.stdout, expected.as_bytes(), "{request}");
        assert_eq!(relative.stdout, absolute.stdout, "{request}");
        assert_eq!(relative.status.code(), absolute.status.code(), "{request}");
        assert_eq!(relative.stderr, absolute.stderr, "{request}");
        let value: serde_json::Value =
            serde_json::from_slice(&relative.stdout).expect("outcome JSON");
        assert_eq!(value["format"], "quire-outcome/1");
    }
}

#[trace("TC-1262")]
#[test]
fn request_byte_options_reach_owner_limits_and_compile_accepts_none() {
    let source_bytes = std::fs::metadata(SOURCE).expect("source bytes").len();
    let request_bytes = std::fs::metadata(RUN_REQUEST).expect("request bytes").len();
    for (option, setting, size) in [
        ("--request-bytes", "request_bytes", request_bytes),
        ("--dependent-bytes", "dependent_bytes", source_bytes),
    ] {
        let too_small = (size - 1).to_string();
        let output = quire(&["run", RUN_REQUEST, option, &too_small, "--format", "json"]);
        assert_eq!(
            output.status.code(),
            Some(i32::from(Category::Incomplete.exit_code()))
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(setting), "{stderr}");
        assert!(stderr.contains(&too_small), "{stderr}");
        assert!(stderr.contains(option), "{stderr}");
        let exact = size.to_string();
        let output = quire(&["run", RUN_REQUEST, option, &exact, "--format", "json"]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let maximum = usize::MAX.to_string();
        let output = quire(&["run", RUN_REQUEST, option, &maximum, "--format", "json"]);
        assert_eq!(output.status.code(), Some(0));
        let output = quire(&[
            "compile",
            MISSING_SOURCE,
            option,
            &exact,
            "--format",
            "json",
        ]);
        assert_eq!(output.status.code(), Some(refusal_code()));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unknown compile option"), "{stderr}");
        assert!(!stderr.contains("cannot read"), "{stderr}");
    }
}

#[trace("TC-1262", "FR-287-AC-9")]
#[test]
fn clause_requests_use_the_core_monitor_document_and_never_ignore_configured_providers() {
    use qsl_command::command::{prepare_run, PreparedRun, RunLimits};
    use qsl_replay::spine::ClauseRunSelection;
    use qsl_replay::{OutcomeDocument, OutcomeStage};
    let directory =
        TestDirectory(std::env::temp_dir().join(format!("quire-clause-{}", std::process::id())));
    std::fs::create_dir_all(&directory.0).expect("clause directory");
    let observation = serde_json::json!({"format":"quire.state.snapshot/v1",
        "identity":{"authority":"issuer","identity":"current"},"observation":"current"});
    std::fs::write(directory.0.join("current.json"), observation.to_string()).expect("observation");
    let request = serde_json::json!({"format":"native-run/1","request":{
        "models":[],"program":{"source":{"file":SOURCE,"authority":"issuer","identity":"document",
            "document":"CliClause","formal_revision":1}},
        "clause":{"selection":{"function":{"name":"yes","arguments":[],"snapshot":"current"}},
            "observations":{"current":"current.json"}}}});
    let request_path = directory.0.join("request.json");
    std::fs::write(&request_path, request.to_string()).expect("clause request");
    let PreparedRun::Clause(request) =
        prepare_run(&request_path, RunLimits::default()).expect("prepare actual clause")
    else {
        panic!("clause request must not become a function call");
    };
    let ClauseRunSelection::Function { ref name, .. } = request.selection else {
        panic!("function claim");
    };
    let name = name.clone();
    let report = qsl_replay::spine::run_clause(*request).expect("actual clause report");
    let document =
        OutcomeDocument::from_clause_runs(Some(OutcomeStage::S4), [(name.as_str(), &report)])
            .expect("core producer");
    let output = Command::new(QUIRE)
        .arg("run")
        .arg(&request_path)
        .args(["--format", "json"])
        .output()
        .expect("run clause");
    let expected = format!(
        "{}\n",
        ix_cli_kit::json::encode_canonical(&document, false).expect("canonical monitor")
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(
        output.status.code(),
        Some(i32::from(document.category().exit_code()))
    );
    let actual: serde_json::Value = serde_json::from_slice(&output.stdout).expect("monitor output");
    assert_eq!(actual["operation"], "monitor");
    // The nonexistent executable proves this unsupported composition never starts a provider.
    let config_path = directory.0.join("providers.json");
    let config = serde_json::json!({"providers":[{"executable":directory.0.join("never-start-provider"),
        "argv":[],"deadline_ms":1024,"limits":{"max_frame_bytes":262144,"max_total_bytes":1048576,
            "max_stderr_bytes":4096,"cancellation_grace_ms":500}}]});
    std::fs::write(&config_path, config.to_string()).expect("provider config");
    let output = Command::new(QUIRE)
        .arg("run")
        .arg(&request_path)
        .arg("--provider-config")
        .arg(&config_path)
        .args(["--format", "json"])
        .output()
        .expect("unsupported clause provider composition");
    assert_eq!(
        output.status.code(),
        Some(i32::from(Category::Unsupported.exit_code()))
    );
    let document: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("typed unsupported document");
    assert_eq!(document["operation"], "monitor");
    assert_eq!(document["category"], "unsupported");
    assert_eq!(document["last_stage"], serde_json::Value::Null);
    assert_eq!(
        document["diagnostics"][0]["code"],
        "unimplemented_capability"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("clause"), "{stderr}");
    assert!(!stderr.contains("could not start"), "{stderr}");
}
