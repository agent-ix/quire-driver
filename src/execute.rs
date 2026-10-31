// SPDX-License-Identifier: AGPL-3.0-or-later
//! The driver's selected QSL execution engine (FR-004).

use qsl_foundation::Code;
use qsl_foundation::SourceIdentity;
use qsl_replay::spine::{Call, CallOutcome, DependencyInput, RunRefusal, SpineLimits};
use qsl_replay::{OutcomeDocument, OutcomeWriteError};
use qsl_semantics::model::object_environment::ObjectEnvironment;
use quire_exact::Cancel;
use quire_semantic_value::catalog::Category;

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
    /// The checked package's name.
    pub package_name: &'a str,
    /// Domain package documents by model identity.
    pub packages: &'a qsl_semantics::model::intake::PackageInput,
    /// Caller-supplied library dependencies.
    pub dependencies: &'a DependencyInput,
    /// Effective lifecycle limits.
    pub limits: SpineLimits,
    /// The named function and its admitted values.
    pub call: Call,
    /// The caller-owned object context used unchanged by QSL evaluation.
    pub object_environment: &'a ObjectEnvironment,
    /// The caller-owned cancellation handle for this run.
    pub cancel: Cancel,
    /// Engine selection; omitted CLI selection defaults to interpreter.
    pub engine: ExecutionEngine,
}

/// The selected interpreter's typed result.
#[derive(Debug)]
pub struct ExecutionResult {
    /// QSL's unchanged typed execution outcome.
    pub outcome: CallOutcome,
    /// QSL's canonical document for the complete run.
    pub document: OutcomeDocument,
}

impl ExecutionResult {
    /// Serialize QSL's canonical `quire-outcome/1` document for this call.
    /// The driver does not define or translate the document shape.
    pub fn outcome_bytes(&self) -> Result<Vec<u8>, OutcomeWriteError> {
        self.document.to_bytes()
    }
}

/// Why a selected execution did not produce a typed result.
#[derive(Debug, thiserror::Error)]
pub enum ExecutionFailure {
    /// The requested engine is not linked into this binary.
    #[error("execution engine {0} is not linked into this binary")]
    UnsupportedEngine(ExecutionEngine),
    /// QSL refused compilation or execution.
    #[error("{refusal}")]
    Run {
        /// QSL's unchanged typed run refusal.
        refusal: Box<RunRefusal>,
        /// QSL's canonical document for that run refusal.
        document: Box<OutcomeDocument>,
    },
}

impl ExecutionFailure {
    /// The QSL category for this failure.
    pub fn category(&self) -> Category {
        match self {
            Self::UnsupportedEngine(_) => Category::Unsupported,
            Self::Run { refusal, .. } => refusal.category(),
        }
    }

    /// Consume the failure into QSL's native execute document.
    pub fn into_document(self) -> OutcomeDocument {
        match self {
            Self::UnsupportedEngine(engine) => {
                OutcomeDocument::new(qsl_replay::Operation::Execute, None, Category::Unsupported)
                    .with_diagnostics(vec![qsl_replay::OutcomeDiagnostic::new(
                        None,
                        Code::UnimplementedCapability.as_str(),
                        None,
                        format!("execution engine {engine} is not linked into this binary"),
                    )])
            }
            Self::Run { document, .. } => *document,
        }
    }
}

/// Run only the selected engine. The current binary links QSL's interpreter;
/// AOT and JIT selections return Unsupported before compiling the source.
pub fn execute(request: ExecutionRequest<'_>) -> Result<ExecutionResult, ExecutionFailure> {
    execute_using(request, |request, object_environment| {
        qsl_replay::spine::run(
            request.source.clone(),
            request.package_name.to_owned(),
            request.path,
            request.bytes,
            request.packages,
            request.dependencies,
            request.limits,
            &request.call,
            object_environment,
            &request.cancel,
        )
    })
}

fn execute_using(
    request: ExecutionRequest<'_>,
    run: impl FnOnce(&ExecutionRequest<'_>, &ObjectEnvironment) -> Result<CallOutcome, Box<RunRefusal>>,
) -> Result<ExecutionResult, ExecutionFailure> {
    if request.engine != ExecutionEngine::Interpreter {
        return Err(ExecutionFailure::UnsupportedEngine(request.engine));
    }
    let outcome = run(&request, request.object_environment);
    let document = OutcomeDocument::from_run(&outcome);
    match outcome {
        Ok(outcome) => Ok(ExecutionResult { outcome, document }),
        Err(refusal) => Err(ExecutionFailure::Run {
            refusal,
            document: Box::new(document),
        }),
    }
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

    use qsl_foundation::Setting;
    use qsl_foundation::SourceIdentity;
    use qsl_replay::spine::{
        default_accounting, Call, CallArgument, CallOutcome, CallValue, DependencyInput,
        SpineLimits,
    };
    use qsl_replay::spine::{CompileRefusal, RunRefusal};
    use qsl_replay::OutcomeDocument;
    use qsl_semantics::model::dispatch::GeneralizationClosure;
    use qsl_semantics::model::domain_package::DomainPackageRecord;
    use qsl_semantics::model::intake::package_input;
    use qsl_semantics::model::object_environment::ObjectEnvironment;
    use qsl_semantics::model::population::{
        admit_binding, AdmissionMeter, AdmissionOutcome, PopulationAdmissionLimits,
        PopulationDocument, PopulationMember,
    };
    use quire_exact::{Cancel, CancelCause, Value};
    use quire_semantic_value::catalog::Category;
    use quire_semantic_value::object_closure::ObjectClosure;

    use super::{execute, ExecutionEngine, ExecutionFailure, ExecutionRequest};

    const UNIT: &str = "language \"ix:native\" edition \"1-draft\";\n\
        profile v = \"quire.value.complete/v1\";\n\
        function yes using v(): Boolean pure { true }\n";

    fn request(engine: ExecutionEngine, limits: SpineLimits) -> ExecutionRequest<'static> {
        ExecutionRequest {
            source: SourceIdentity::new("quire-driver-test", "unit", "test", "current"),
            package_name: "unit",
            path: "unit.qsl",
            bytes: UNIT.as_bytes(),
            packages: &EMPTY_PACKAGES,
            dependencies: &NO_DEPENDENCIES,
            limits,
            call: Call {
                function: "yes".to_owned(),
                arguments: Vec::new(),
                accounting: default_accounting(10),
                supplied: quire_semantic_value::supplied_stop::SuppliedLimits::default(),
            },
            object_environment: &EMPTY_OBJECT_ENVIRONMENT,
            cancel: Cancel::new(),
            engine,
        }
    }

    static EMPTY_PACKAGES: LazyLock<qsl_semantics::model::intake::PackageInput> =
        LazyLock::new(BTreeMap::new);
    static NO_DEPENDENCIES: LazyLock<DependencyInput> = LazyLock::new(DependencyInput::default);
    static EMPTY_OBJECT_ENVIRONMENT: LazyLock<ObjectEnvironment> =
        LazyLock::new(ObjectEnvironment::default);

    const OBJECT_MODEL_DOCUMENT: &str = r#"{
      "contractVersion": "2.0.0",
      "canonicalization": {"algorithm": "rfc8785-v1"},
      "source": {"identity": "ix://driver/ir608/model", "version": "1.0.0", "dialect": "spec-bundle"},
      "package": {"identity": "driver/ir608-model", "version": "1.0.0", "mappingVersions": [], "profileVersions": []},
      "occurrences": [],
      "extensions": [],
      "constructs": [
        {"kind": {"module": "driver/ir608-model", "name": "entity"}, "moduleVersion": "1.0.0", "construct": {"identity": "none", "shape": "record", "members": {}, "meaning": "quire.meaning.model.object-type/v1"}},
        {"kind": {"module": "driver/ir608-model", "name": "population"}, "moduleVersion": "1.0.0", "construct": {"identity": "none", "shape": "record", "members": {}, "meaning": "quire.meaning.model.population/v1"}}
      ],
      "types": [{"identity": "ix://driver/ir608-model/Thing", "displayName": "Thing", "kind": {"module": "driver/ir608-model", "name": "entity"}, "roles": [], "origin": {"generated": {"generatorIdentity": "ix://driver/ir608/model", "generatorVersion": "1.0.0", "inputIdentities": ["ix://driver/ir608-model/Thing"]}}, "constraints": [], "extensions": [], "unknownPolicy": "reject", "supertypes": [], "fields": [], "operations": []}],
      "populations": [{"identity": "ix://driver/ir608-model/Fleet", "displayName": "Fleet", "kind": {"module": "driver/ir608-model", "name": "population"}, "members": ["ix://driver/ir608-model/Thing"], "extent": "closed", "origin": {"generated": {"generatorIdentity": "ix://driver/ir608-model/Fleet", "generatorVersion": "1.0.0", "inputIdentities": ["ix://driver/ir608-model/Fleet"]}}}]
    }"#;

    struct ObjectContextFixture {
        source: String,
        packages: qsl_semantics::model::intake::PackageInput,
        population: quire_exact::PopulationId,
        empty_population_id: quire_exact::PopulationId,
        reference: quire_exact::ObjectReference,
        populated: ObjectEnvironment,
        empty_context: ObjectEnvironment,
        empty_population_env: ObjectEnvironment,
    }

    fn object_context_fixture() -> ObjectContextFixture {
        let packages =
            package_input([OBJECT_MODEL_DOCUMENT.as_bytes()]).expect("the fixture is a package");
        let source = "language \"ix:native\" edition \"1-draft\";\n\
             profile v = \"quire.model.complete/v1\";\n\
             model M = \"driver/ir608-model\";\n\
             function query_present using v(p: Population<M::Thing>[1], r: M::Thing): M::Thing pure { lookup<M::Thing>(p, r) absent refused }\n\
             function query_absent using v(p: Population<M::Thing>[1], r: M::Thing): M::Thing pure { lookup<M::Thing>(p, r) absent undefined }\n";
        let source_identity =
            SourceIdentity::new("quire-driver-test", "object-context", "test", "current");
        let cancel = Cancel::new();
        let parsed = qsl_replay::spine::parse(
            &qsl_replay::spine::ParseRequest {
                source: &source_identity,
                path: "object-context.qsl",
                bytes: source.as_bytes(),
            },
            SpineLimits::default().source,
            &cancel,
        )
        .expect("model-call source parses")
        .into_value();
        let models =
            qsl_replay::spine::select(&parsed, &packages, SpineLimits::default().model, &cancel)
                .expect("model-call package selects")
                .into_value();
        let checked = qsl_replay::spine::check(
            &parsed,
            "object-context".to_owned(),
            &models,
            &DependencyInput::default(),
            SpineLimits::default(),
            &cancel,
        )
        .expect("model-call source checks")
        .into_value();
        let view = &models.models()[0].view;
        let (type_key, type_identity) = view
            .type_identities()
            .iter()
            .find(|(key, _)| key.node.rsplit('/').next() == Some("Thing"))
            .map(|(key, identity)| (key.clone(), identity.clone()))
            .expect("Thing is declared by the model package");
        let population_key = view
            .domain_package()
            .records
            .iter()
            .find_map(|record| match record {
                DomainPackageRecord::Population(population)
                    if population.key.node.rsplit('/').next() == Some("Fleet") =>
                {
                    Some(population.key.clone())
                }
                _ => None,
            })
            .expect("Fleet is declared by the model package");
        let admit = |members| {
            let document = PopulationDocument {
                model_identity: view.model_selection().identity.clone(),
                members,
            };
            let mut meter = AdmissionMeter::new(PopulationAdmissionLimits::default());
            let AdmissionOutcome::Admitted(binding) = admit_binding(
                view,
                &document,
                &population_key,
                GeneralizationClosure::Closed,
                Some(1),
                &mut meter,
            ) else {
                panic!("the fixture population admits");
            };
            binding
        };
        let member = PopulationMember {
            object: "g1".to_owned(),
            type_identity: type_key.clone(),
            field_values: Vec::new(),
        };
        let populated_binding = admit(vec![member]);
        let population = populated_binding.population_id();
        let reference = quire_exact::ObjectReference::new(
            view.object_universe_of(&type_key)
                .expect("Thing has an object universe")
                .identity()
                .expect("the universe has an identity"),
            type_identity,
            quire_exact::ObjectId::new("g1").expect("fixture object id is valid"),
        );
        let world = ObjectClosure::new(
            checked.package().graph().scope().types(),
            [(reference.clone(), Vec::new())],
            &[],
        )
        .expect("the fixture object world closes");
        let populated = ObjectEnvironment::new(world)
            .with_population(populated_binding.clone())
            .expect("the fixture population is recorded");
        let empty_context = ObjectEnvironment::new(ObjectClosure::default())
            .with_population(populated_binding)
            .expect("the same population remains recorded without its object");
        let empty_population_binding = admit(Vec::new());
        let empty_population_id = empty_population_binding.population_id();
        let empty_population_environment = ObjectEnvironment::new(
            ObjectClosure::new(
                checked.package().graph().scope().types(),
                [(reference.clone(), Vec::new())],
                &[],
            )
            .expect("the lookup object is admitted independently of the empty population"),
        )
        .with_population(empty_population_binding)
        .expect("the empty fixture population is recorded");
        ObjectContextFixture {
            source: source.to_owned(),
            packages,
            population,
            empty_population_id,
            reference,
            populated,
            empty_context,
            empty_population_env: empty_population_environment,
        }
    }

    fn model_request<'a>(
        fixture: &'a ObjectContextFixture,
        environment: &'a ObjectEnvironment,
        function: &str,
        population: quire_exact::PopulationId,
        reference: quire_exact::ObjectReference,
    ) -> ExecutionRequest<'a> {
        ExecutionRequest {
            source: SourceIdentity::new("quire-driver-test", "object-context", "test", "current"),
            package_name: "object-context",
            path: "object-context.qsl",
            bytes: fixture.source.as_bytes(),
            packages: &fixture.packages,
            dependencies: &NO_DEPENDENCIES,
            limits: SpineLimits::default(),
            call: Call {
                function: function.to_owned(),
                arguments: vec![
                    CallArgument {
                        parameter: "p".to_owned(),
                        value: Value::Population(population),
                    },
                    CallArgument {
                        parameter: "r".to_owned(),
                        value: Value::Reference(reference),
                    },
                ],
                accounting: default_accounting(10),
                supplied: Default::default(),
            },
            object_environment: environment,
            cancel: Cancel::new(),
            engine: ExecutionEngine::Interpreter,
        }
    }

    fn direct_document(request: &ExecutionRequest<'_>) -> OutcomeDocument {
        let outcome = qsl_replay::spine::run(
            request.source.clone(),
            request.package_name.to_owned(),
            request.path,
            request.bytes,
            request.packages,
            request.dependencies,
            request.limits,
            &request.call,
            request.object_environment,
            &request.cancel,
        );
        OutcomeDocument::from_run(&outcome)
    }

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

    #[test]
    fn execute_result_uses_qsl_outcome_document_serializer() {
        let request = request(ExecutionEngine::default(), SpineLimits::default());
        let expected_run = qsl_replay::spine::run(
            request.source.clone(),
            request.package_name.to_owned(),
            request.path,
            request.bytes,
            request.packages,
            request.dependencies,
            request.limits,
            &request.call,
            request.object_environment,
            &request.cancel,
        );
        let expected_document = OutcomeDocument::from_run(&expected_run);
        let result = execute(request).expect("interpreter runs");
        assert_eq!(
            result.outcome_bytes().expect("QSL encodes outcome"),
            expected_document
                .to_bytes()
                .expect("QSL encodes direct run document")
        );
        assert_eq!(result.document.category(), Category::Success);
    }

    /// Trace: FR-004-AC-5, TC-011. The public caller's non-empty
    /// unresolved-reference policy reaches QSL unchanged.
    #[test]
    fn caller_object_environment_is_forwarded_unchanged() {
        let objects = ObjectEnvironment::default().with_unresolved_references();
        assert!(objects.references_are_unresolved());
        let mut request = request(ExecutionEngine::Interpreter, SpineLimits::default());
        request.object_environment = &objects;

        let expected = qsl_replay::spine::run(
            request.source.clone(),
            request.package_name.to_owned(),
            request.path,
            request.bytes,
            request.packages,
            request.dependencies,
            request.limits,
            &request.call,
            &objects,
            &request.cancel,
        );
        let expected_document = OutcomeDocument::from_run(&expected);
        let actual = execute(request).expect("the public driver run completes");
        let expected_bytes = expected_document
            .to_bytes()
            .expect("QSL serializes the direct run result");
        assert_eq!(
            actual
                .outcome_bytes()
                .expect("QSL serializes the driver result"),
            expected_bytes
        );
    }

    /// Trace: FR-004-AC-6, TC-011. The object member is admitted only by the
    /// caller's populated environment, and the driver agrees with direct QSL
    /// for both that environment and the otherwise identical empty context.
    #[test]
    fn caller_object_context_controls_lookup_result() {
        let fixture = object_context_fixture();
        let populated_request = model_request(
            &fixture,
            &fixture.populated,
            "query_present",
            fixture.population.clone(),
            fixture.reference.clone(),
        );
        let populated_expected = direct_document(&populated_request);
        let populated_result = execute(populated_request).expect("the populated lookup completes");
        assert!(matches!(
            &populated_result.outcome,
            CallOutcome::Completed(CallValue::Reference(reference)) if reference == &fixture.reference
        ));
        assert_eq!(
            populated_result
                .outcome_bytes()
                .expect("driver encodes the populated result"),
            populated_expected
                .to_bytes()
                .expect("direct QSL encodes the populated result")
        );

        let empty_request = model_request(
            &fixture,
            &fixture.empty_context,
            "query_present",
            fixture.population.clone(),
            fixture.reference.clone(),
        );
        let empty_expected = direct_document(&empty_request);
        let empty_failure = execute(empty_request).expect_err("the empty context lacks g1");
        let ExecutionFailure::Run {
            refusal,
            document: empty_document,
        } = empty_failure
        else {
            panic!("the empty object context must preserve QSL's typed refusal");
        };
        assert!(matches!(
            refusal.as_ref(),
            RunRefusal::DanglingReference { position: 1 }
        ));
        let empty_bytes = empty_document
            .to_bytes()
            .expect("driver encodes the empty-context refusal");
        assert_eq!(
            empty_bytes,
            empty_expected
                .to_bytes()
                .expect("direct QSL encodes the empty-context refusal")
        );
        assert_ne!(
            populated_expected
                .to_bytes()
                .expect("direct QSL encodes the populated result"),
            empty_bytes,
            "a caller-supplied object context changes the QSL result"
        );
    }

    /// Trace: FR-004-AC-7, TC-011. QSL defines this absent lookup as
    /// `Undefined`; the driver retains that outcome and its canonical document.
    #[test]
    fn absent_undefined_lookup_matches_direct_qsl() {
        let fixture = object_context_fixture();
        let request = model_request(
            &fixture,
            &fixture.empty_population_env,
            "query_absent",
            fixture.empty_population_id.clone(),
            fixture.reference.clone(),
        );
        let expected = direct_document(&request);
        let actual = execute(request).expect("the absent undefined lookup is an outcome");
        assert!(matches!(
            actual.outcome,
            CallOutcome::Undefined {
                reason: "absent-key"
            }
        ));
        assert_eq!(
            actual
                .outcome_bytes()
                .expect("driver encodes QSL's undefined outcome"),
            expected
                .to_bytes()
                .expect("direct QSL encodes its undefined outcome")
        );
    }

    /// Trace: FR-004-AC-8, TC-011. A caller population identity not present in
    /// its environment keeps the exact typed QSL refusal and document.
    #[test]
    fn unresolved_population_identity_matches_direct_qsl_refusal() {
        let fixture = object_context_fixture();
        let environment = ObjectEnvironment::default();
        let request = model_request(
            &fixture,
            &environment,
            "query_present",
            fixture.empty_population_id.clone(),
            fixture.reference.clone(),
        );
        let expected = direct_document(&request);
        let failure = execute(request).expect_err("the population identity is not admitted");
        assert_eq!(failure.category(), Category::Refusal);
        let ExecutionFailure::Run { refusal, document } = failure else {
            panic!("the driver must preserve QSL's typed population refusal");
        };
        assert!(matches!(
            refusal.as_ref(),
            RunRefusal::WrongValueKind { position: 0 }
        ));
        assert_eq!(
            document
                .to_bytes()
                .expect("driver encodes QSL's population refusal"),
            expected
                .to_bytes()
                .expect("direct QSL encodes its population refusal")
        );
    }

    /// Trace: FR-004-AC-3, unavailable engines are unsupported before work.
    #[test]
    fn unavailable_engines_refuse_without_substitution() {
        for engine in [ExecutionEngine::Aot, ExecutionEngine::Jit] {
            let failure =
                execute(request(engine, SpineLimits::default())).expect_err("engine is not linked");
            assert_eq!(failure.category(), Category::Unsupported);
            let document = failure.into_document();
            assert_eq!(document.category(), Category::Unsupported);
            assert_eq!(document.category().exit_code(), 21);
            let json: serde_json::Value = serde_json::from_slice(
                &document.to_bytes().expect("QSL encodes unsupported engine"),
            )
            .expect("valid JSON document");
            assert_eq!(json["diagnostics"][0]["code"], "unimplemented_capability");
            assert!(json["diagnostics"][0]["message"]
                .as_str()
                .expect("diagnostic message")
                .contains(engine.as_str()));
        }
    }

    /// Trace: FR-004-AC-4, the linked interpreter preserves QSL's exact preparation refusal.
    #[test]
    fn selected_interpreter_preparation_refusal_matches_direct_qsl_run() {
        let mut request = request(ExecutionEngine::Interpreter, SpineLimits::default());
        let mut source = request.limits.source;
        source.source_bytes = 1;
        request.limits.source = source;

        let expected = qsl_replay::spine::run(
            request.source.clone(),
            request.package_name.to_owned(),
            request.path,
            request.bytes,
            request.packages,
            request.dependencies,
            request.limits,
            &request.call,
            request.object_environment,
            &request.cancel,
        );
        let expected_document = OutcomeDocument::from_run(&expected);
        let failure = execute(request).expect_err("selected interpreter returns its refusal");
        let ExecutionFailure::Run {
            refusal: actual,
            document,
        } = failure
        else {
            panic!("the driver must preserve the QSL preparation refusal");
        };
        assert_eq!(
            format!("{actual:?}"),
            format!("{:?}", expected.unwrap_err())
        );
        assert_eq!(
            document.to_bytes().unwrap(),
            expected_document.to_bytes().unwrap()
        );
    }

    /// Trace: FR-004-AC-4, caller cancellation and its QSL document pass through unchanged.
    #[test]
    fn pre_cancelled_caller_handle_preserves_qsl_refusal_and_document() {
        let request = request(ExecutionEngine::Interpreter, SpineLimits::default());
        request.cancel.cancel(CancelCause::Requested);

        let expected_run = qsl_replay::spine::run(
            request.source.clone(),
            request.package_name.to_owned(),
            request.path,
            request.bytes,
            request.packages,
            request.dependencies,
            request.limits,
            &request.call,
            request.object_environment,
            &request.cancel,
        );
        let expected_document = OutcomeDocument::from_run(&expected_run);
        let expected_refusal = expected_run.expect_err("QSL observes the caller's cancellation");
        assert!(matches!(
            expected_refusal.as_ref(),
            RunRefusal::Cancelled(CancelCause::Requested)
        ));

        let failure = execute(request).expect_err("the same caller handle stops execution");
        assert_eq!(failure.category(), Category::Incomplete);
        let ExecutionFailure::Run { refusal, document } = failure else {
            panic!("execute must preserve QSL's cancellation refusal");
        };
        assert!(matches!(
            refusal.as_ref(),
            RunRefusal::Cancelled(CancelCause::Requested)
        ));
        assert_eq!(format!("{refusal:?}"), format!("{expected_refusal:?}"));
        let expected_bytes = expected_document
            .to_bytes()
            .expect("QSL encodes cancellation");
        assert_eq!(
            document.to_bytes().expect("driver returns QSL document"),
            expected_bytes
        );

        let failure = ExecutionFailure::Run { refusal, document };
        assert_eq!(
            failure
                .into_document()
                .to_bytes()
                .expect("failure returns QSL document"),
            expected_bytes
        );
    }

    /// Trace: FR-003-AC-3, the driver wrapper preserves QSL limit metadata.
    #[test]
    fn caller_source_limit_preserves_configured_bound_and_field() {
        let mut limits = SpineLimits::default();
        let mut source = limits.source;
        source.source_bytes = 1;
        limits.source = source;
        let failure = execute(request(ExecutionEngine::Interpreter, limits))
            .expect_err("one byte source limit refuses");
        let ExecutionFailure::Run { refusal, .. } = failure else {
            panic!("the selected engine must preserve QSL's typed limit error");
        };
        let RunRefusal::Compile(compile) = *refusal else {
            panic!("a source-byte ceiling is a compile-stage limit");
        };
        let CompileRefusal::Limit(limit) = *compile else {
            panic!("the driver must not remap the QSL limit variant");
        };
        assert_eq!(limit.configured_bound(), 1);
        assert_eq!(limit.setting(), Setting::S1InputBytes);
        assert!(limit.actual() > u128::from(limit.configured_bound()));
    }

    /// Trace: FR-003-AC-3, QSL's stage-limit category reaches the caller unchanged.
    #[test]
    fn qsl_stage_limit_refusal_uses_incomplete_category() {
        let mut limits = SpineLimits::default();
        let mut source = limits.source;
        source.source_bytes = 1;
        limits.source = source;
        let failure = execute(request(ExecutionEngine::Interpreter, limits))
            .expect_err("one byte source limit refuses");
        assert_eq!(failure.category(), Category::Incomplete);
        assert_eq!(failure.category().exit_code(), 22);
    }
}
