// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's selected QSL execution engine (FR-004).

use std::collections::BTreeMap;

use qsl_foundation::diagnostic::Category;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{Call, CallOutcome, DependencyInput, RunRefusal, SpineLimits};
use qsl_semantics::library::PackageId;

/// The engine selected for one execution request.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ExecutionEngine {
    /// QSL's linked interpreter.
    #[default]
    Interpreter,
    /// AOT execution, when linked into the binary.
    Aot,
    /// JIT execution, when linked into the binary.
    Jit,
}

impl ExecutionEngine {
    /// Parse FR-004's closed engine selector.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "interpreter" => Ok(Self::Interpreter),
            "aot" => Ok(Self::Aot),
            "jit" => Ok(Self::Jit),
            _ => Err(format!("unknown engine `{value}`")),
        }
    }

    /// The CLI spelling for this engine.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Interpreter => "interpreter",
            Self::Aot => "aot",
            Self::Jit => "jit",
        }
    }
}

impl std::fmt::Display for ExecutionEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One invocation of QSL's typed run operation.
pub struct ExecutionRequest<'a> {
    /// Source authority and revision labels.
    pub source: SourceIdentity,
    /// Display path passed to QSL; the driver does not derive identity from it.
    pub path: &'a str,
    /// Complete source bytes.
    pub bytes: &'a [u8],
    /// Digest-addressed domain package documents.
    pub packages: &'a BTreeMap<[u8; 32], Vec<u8>>,
    /// Caller-supplied library dependencies.
    pub dependencies: &'a DependencyInput,
    /// Effective lifecycle limits.
    pub limits: SpineLimits,
    /// The named function and its admitted values.
    pub call: Call,
    /// Engine selection; omitted CLI selection defaults to interpreter.
    pub engine: ExecutionEngine,
}

/// The selected interpreter's typed result and compiled package identity.
#[derive(Debug)]
pub struct ExecutionResult {
    /// The compiled package identity.
    pub package_id: PackageId,
    /// QSL's unchanged typed execution outcome.
    pub outcome: CallOutcome,
}

/// Why a selected execution did not produce a typed result.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionFailure {
    /// The requested engine is not linked into this binary.
    #[error("execution engine {0} is not linked into this binary")]
    UnsupportedEngine(ExecutionEngine),
    /// QSL refused compilation or execution.
    #[error(transparent)]
    Run(#[from] Box<RunRefusal>),
}

impl ExecutionFailure {
    /// The QSL category for this failure.
    pub fn category(&self) -> Category {
        match self {
            Self::UnsupportedEngine(_) => Category::Unsupported,
            Self::Run(refusal) => refusal.category(),
        }
    }
}

/// Run only the selected engine. The current binary links QSL's interpreter;
/// AOT and JIT selections return Unsupported before compiling the source.
pub fn execute(request: ExecutionRequest<'_>) -> Result<ExecutionResult, ExecutionFailure> {
    if request.engine != ExecutionEngine::Interpreter {
        return Err(ExecutionFailure::UnsupportedEngine(request.engine));
    }
    let (package_id, outcome) = qsl_replay::spine::run(
        request.source,
        request.path,
        request.bytes,
        request.packages,
        request.dependencies,
        request.limits,
        &request.call,
    )?;
    Ok(ExecutionResult {
        package_id,
        outcome,
    })
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a failed assertion is the test failure report"
)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::LazyLock;

    use qsl_foundation::diagnostic::Category;
    use qsl_foundation::SourceIdentity;
    use qsl_replay::spine::{
        default_accounting, Call, CallOutcome, CallValue, DependencyInput, SpineLimits,
    };
    use qsl_replay::spine::{CompileRefusal, RunRefusal};

    use super::{execute, ExecutionEngine, ExecutionFailure, ExecutionRequest};

    const UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"quire.value.complete/v1\";\n\
        function yes using v(): Boolean pure { true }\n";

    fn request(engine: ExecutionEngine, limits: SpineLimits) -> ExecutionRequest<'static> {
        ExecutionRequest {
            source: SourceIdentity::new("quire-driver-test", "unit", "test", "current"),
            path: "unit.qsl",
            bytes: UNIT.as_bytes(),
            packages: &EMPTY_PACKAGES,
            dependencies: &NO_DEPENDENCIES,
            limits,
            call: Call {
                function: "yes".to_owned(),
                arguments: Vec::new(),
                accounting: default_accounting(10),
            },
            engine,
        }
    }

    static EMPTY_PACKAGES: LazyLock<BTreeMap<[u8; 32], Vec<u8>>> = LazyLock::new(BTreeMap::new);
    static NO_DEPENDENCIES: LazyLock<DependencyInput> = LazyLock::new(DependencyInput::default);

    /// Trace: FR-004-AC-1, default interpreter result is returned unchanged.
    #[test]
    fn interpreter_runs_the_selected_function() {
        let result = execute(request(ExecutionEngine::default(), SpineLimits::default()))
            .expect("interpreter runs");
        assert!(matches!(
            result.outcome,
            CallOutcome::Completed(CallValue::Boolean(true))
        ));
    }

    /// Trace: FR-004-AC-3, unavailable engines are unsupported before work.
    #[test]
    fn unavailable_engines_refuse_without_substitution() {
        for engine in [ExecutionEngine::Aot, ExecutionEngine::Jit] {
            let failure =
                execute(request(engine, SpineLimits::default())).expect_err("engine is not linked");
            assert_eq!(failure.category(), Category::Unsupported);
            assert!(
                matches!(failure, ExecutionFailure::UnsupportedEngine(found) if found == engine)
            );
        }
    }

    /// Trace: FR-003-AC-3, the caller's typed limit reaches QSL unchanged.
    #[test]
    fn caller_source_limit_reaches_qsl_as_a_typed_limit_refusal() {
        let mut limits = SpineLimits::default();
        let mut source = limits.source;
        source.source_bytes = 1;
        limits.source = source;
        let failure = execute(request(ExecutionEngine::Interpreter, limits))
            .expect_err("one byte source limit refuses");
        assert!(matches!(failure, ExecutionFailure::Run(ref refusal)
            if matches!(&**refusal, RunRefusal::Compile(compile)
                if matches!(**compile, CompileRefusal::Limit(_)))));
    }

    /// Trace: FR-003-AC-3, pending QSL-489 correction of StageLimitExceeded category.
    #[test]
    #[ignore = "Pending QSL-489: QSL must classify StageLimitExceeded as incomplete (exit 22); driver must not remap it: https://linear.app/agent-ix/issue/QSL-489/lc1-cancel-polled-at-every-charge-split-spinecompile-into-typed-check"]
    fn qsl_stage_limit_refusal_uses_incomplete_category() {
        let mut limits = SpineLimits::default();
        let mut source = limits.source;
        source.source_bytes = 1;
        limits.source = source;
        let failure = execute(request(ExecutionEngine::Interpreter, limits))
            .expect_err("one byte source limit refuses");
        assert_eq!(failure.category(), Category::Incomplete);
    }
}
