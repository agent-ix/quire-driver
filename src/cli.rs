// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared seams for the `quire` binary.
//!
//! This module deliberately carries QSL's serialized outcome as opaque text.
//! It does not define an outcome shape or translate categories into a second
//! exit taxonomy. Command operations will hand their QSL outcome to
//! [`finish_outcome`].

use std::io;
use std::process::ExitCode;

use qsl_foundation::diagnostic::Category;

/// Convert QSL's category exit result to the process exit type.
///
/// This is the only category-to-process conversion in the driver. It calls
/// QSL's total [`Category::exit_code`] function and does not use
/// `ix-cli-kit::Outcome`.
#[must_use]
pub fn process_exit_code(category: Category) -> ExitCode {
    ExitCode::from(category.exit_code())
}

/// Boundary for future shared CLI-kit outcome writing.
///
/// IR-618 and the PLAT-1140 exit-model answer must land before ix-cli-kit can
/// be adopted. The driver keeps QSL's outcome and category authoritative.
pub trait OutcomeSink {
    /// Write the already serialized QSL outcome unchanged.
    fn write_outcome(&mut self, outcome_json: &str) -> io::Result<()>;
}

/// Preserve QSL's operation status while leaving output mechanics at a seam.
#[must_use]
pub fn finish_outcome<S: OutcomeSink>(
    sink: &mut S,
    category: Category,
    outcome_json: &str,
) -> ExitCode {
    match sink.write_outcome(outcome_json) {
        Ok(()) => process_exit_code(category),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => process_exit_code(category),
        Err(_) => process_exit_code(Category::InternalFailure),
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use std::io;
    use std::process::ExitCode;

    use qsl_foundation::diagnostic::Category;

    use super::{finish_outcome, process_exit_code, OutcomeSink};

    /// Trace: IR-607, one QSL category exit function.
    #[test]
    fn process_code_is_qsl_category_exit_code_without_remapping() {
        for category in Category::ALL {
            assert_eq!(
                process_exit_code(category),
                ExitCode::from(category.exit_code())
            );
        }
    }

    /// Trace: IR-607, preserve the serialized QSL outcome.
    #[test]
    fn outcome_sink_receives_opaque_json_unchanged() {
        let outcome = r#"{"schema":"quire-outcome/1","items":[]}"#;
        let mut output = CapturingSink(None);
        let status = finish_outcome(&mut output, Category::Success, outcome);
        assert_eq!(status, ExitCode::from(Category::Success.exit_code()));
        assert_eq!(output.0.as_deref(), Some(outcome));
    }

    struct CapturingSink(Option<String>);

    impl OutcomeSink for CapturingSink {
        fn write_outcome(&mut self, outcome_json: &str) -> io::Result<()> {
            self.0 = Some(outcome_json.to_owned());
            Ok(())
        }
    }

    struct FailingSink(io::ErrorKind);

    impl OutcomeSink for FailingSink {
        fn write_outcome(&mut self, _outcome_json: &str) -> io::Result<()> {
            Err(io::Error::from(self.0))
        }
    }

    /// Trace: IR-607, preserve the outcome code on broken pipes.
    #[test]
    fn broken_pipe_keeps_the_operation_category() {
        let mut output = FailingSink(io::ErrorKind::BrokenPipe);
        let status = finish_outcome(&mut output, Category::Incomplete, "{}");
        assert_eq!(status, ExitCode::from(Category::Incomplete.exit_code()));
    }

    /// Trace: IR-607, classify a failed output stream through QSL.
    #[test]
    fn other_write_failure_uses_qsl_internal_failure_category() {
        let mut output = FailingSink(io::ErrorKind::Other);
        let status = finish_outcome(&mut output, Category::Success, "{}");
        assert_eq!(
            status,
            ExitCode::from(Category::InternalFailure.exit_code())
        );
    }
}
