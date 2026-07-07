//! NC12 backend-neutral Runtime IR evaluator and interpreter comparison report.
//!
//! This module executes `RuntimeExpr` directly from the runtime artifact. It is
//! deliberately below the native/backend boundary: evaluator success is only a
//! runtime-IR observation, not kernel evidence, source-semantics proof, native
//! validation, Cranelift validation, object validation, linker validation, or
//! broader NC8/NC9 evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    RuntimeArtifactIdentity, RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExample,
    RuntimeExpr, RuntimeGroundValue, RuntimeLowerabilityStatus, RuntimeObservation,
    RuntimePartiality, RuntimePrimitive, RuntimeProgram, RuntimeSymbol, RuntimeTrap,
    RuntimeTrapCode, RuntimeValue,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrRunReport {
    pub evaluator: RuntimeIrEvaluator,
    pub target: RuntimeIrTargetIdentity,
    pub artifact: RuntimeArtifactIdentity,
    pub observation: RuntimeIrObservation,
    pub evidence: RuntimeIrRunEvidence,
    pub trust: RuntimeIrTrustReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrEvaluator {
    Nc12RuntimeIrEvaluatorV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrTargetIdentity {
    pub example: String,
    pub checked_core_shape: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrObservation {
    pub artifact: RuntimeArtifactIdentity,
    pub target: RuntimeIrTargetIdentity,
    pub observation: RuntimeObservation,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInterpreterObservation {
    pub artifact: RuntimeArtifactIdentity,
    pub target: RuntimeIrTargetIdentity,
    pub observation: RuntimeObservation,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrDifferentialReport {
    pub artifact: RuntimeArtifactIdentity,
    pub target: RuntimeIrTargetIdentity,
    pub interpreter: RuntimeInterpreterObservation,
    pub runtime_ir: Option<RuntimeIrObservation>,
    pub trust: RuntimeIrTrustReport,
    pub verdict: RuntimeIrDifferentialVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrDifferentialVerdict {
    Nc12InterpreterRuntimeIrAgreement {
        stage: RuntimeIrDifferentialStage,
    },
    Unsupported {
        stage: RuntimeIrDifferentialStage,
        construct: &'static str,
        reason: String,
    },
    Mismatch {
        stage: RuntimeIrDifferentialStage,
        interpreter: RuntimeObservation,
        runtime_ir: RuntimeObservation,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrDifferentialStage {
    BoundaryPreflight,
    RuntimeIrEvaluation,
    InterpreterRuntimeIrCompare,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrRunEvidence {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub runtime_artifact_hash: u64,
    pub target_example: String,
    pub checked_core_shape: String,
    pub evidence_sources: BTreeMap<String, String>,
    pub unavailable: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrTrustReport {
    pub tier: RuntimeIrTrustTier,
    pub evaluator: RuntimeIrEvidenceFact,
    pub interpreter_oracle: RuntimeIrEvidenceFact,
    pub native_backend: RuntimeIrEvidenceFact,
    pub object_artifact: RuntimeIrEvidenceFact,
    pub linker: RuntimeIrEvidenceFact,
    pub source_level_proof: RuntimeIrEvidenceFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrTrustTier {
    Nc12RuntimeIrObservation,
    Nc12InterpreterRuntimeIrAgreement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrEvidenceFact {
    Available {
        value: String,
        evidence_source: String,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrSeedEnvironment {
    values: BTreeMap<RuntimeSymbol, RuntimeGroundValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeIrEvaluationError {
    pub stage: RuntimeIrEvaluationStage,
    pub construct: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrEvaluationStage {
    BoundaryPreflight,
    RuntimeIrEvaluation,
}

impl RuntimeIrSeedEnvironment {
    pub fn empty() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn nc5_seed() -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            "decl:fixture::Local::y".to_string(),
            RuntimeGroundValue::Int(2),
        );
        Self { values }
    }

    pub fn insert(&mut self, symbol: impl Into<RuntimeSymbol>, value: RuntimeGroundValue) {
        self.values.insert(symbol.into(), value);
    }
}

impl RuntimeIrTargetIdentity {
    pub fn from_example(example: &RuntimeExample) -> Self {
        Self {
            example: example.name.clone(),
            checked_core_shape: example.checked_core_shape.clone(),
        }
    }
}

impl RuntimeIrRunEvidence {
    fn from_program_and_example(program: &RuntimeProgram, example: &RuntimeExample) -> Self {
        let mut evidence_sources = BTreeMap::new();
        evidence_sources.insert(
            "package_identity".to_string(),
            "RuntimeProgram.package_identity from the exact runtime artifact".to_string(),
        );
        evidence_sources.insert(
            "core_semantic_hash".to_string(),
            "RuntimeProgram.core_semantic_hash from the exact runtime artifact".to_string(),
        );
        evidence_sources.insert(
            "runtime_artifact_hash".to_string(),
            "RuntimeProgram.artifact_hash from the exact runtime artifact".to_string(),
        );
        evidence_sources.insert(
            "target_example".to_string(),
            "RuntimeExample.name from the exact runtime artifact".to_string(),
        );
        evidence_sources.insert(
            "runtime_ir_evaluator".to_string(),
            "ken-runtime NC12 direct RuntimeExpr evaluator".to_string(),
        );
        Self {
            package_identity: program.package_identity.clone(),
            core_semantic_hash: program.core_semantic_hash,
            runtime_artifact_hash: program.artifact_hash,
            target_example: example.name.clone(),
            checked_core_shape: example.checked_core_shape.clone(),
            evidence_sources,
            unavailable: BTreeSet::from([
                "native_backend_validation".to_string(),
                "object_artifact_validation".to_string(),
                "linker_validation".to_string(),
                "source_level_proof_validation".to_string(),
            ]),
        }
    }
}

impl RuntimeIrTrustReport {
    fn observation() -> Self {
        Self {
            tier: RuntimeIrTrustTier::Nc12RuntimeIrObservation,
            evaluator: RuntimeIrEvidenceFact::Available {
                value: "NC12 direct RuntimeExpr evaluator".to_string(),
                evidence_source: "ken-runtime evaluated the RuntimeExpr without Cranelift"
                    .to_string(),
            },
            interpreter_oracle: RuntimeIrEvidenceFact::Unavailable {
                reason: "no interpreter oracle was supplied to this standalone evaluator run"
                    .to_string(),
            },
            native_backend: RuntimeIrEvidenceFact::Unavailable {
                reason: "NC12 runtime-IR evaluation does not invoke Cranelift or native code"
                    .to_string(),
            },
            object_artifact: RuntimeIrEvidenceFact::Unavailable {
                reason: "NC12 runtime-IR evaluation does not emit object artifacts".to_string(),
            },
            linker: RuntimeIrEvidenceFact::Unavailable {
                reason: "NC12 runtime-IR evaluation does not invoke a linker".to_string(),
            },
            source_level_proof: RuntimeIrEvidenceFact::Unavailable {
                reason: "runtime-IR observation is not a source-level semantics proof".to_string(),
            },
        }
    }

    fn agreement() -> Self {
        let mut report = Self::observation();
        report.tier = RuntimeIrTrustTier::Nc12InterpreterRuntimeIrAgreement;
        report.interpreter_oracle = RuntimeIrEvidenceFact::Available {
            value: "caller-supplied interpreter observation".to_string(),
            evidence_source: "RuntimeInterpreterObservation supplied by the caller; this API verifies only artifact and target identity, not interpreter provenance".to_string(),
        };
        report
    }
}

impl fmt::Display for RuntimeIrEvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.construct, self.reason)
    }
}

impl std::error::Error for RuntimeIrEvaluationError {}

pub fn evaluate_runtime_ir_example(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &RuntimeIrSeedEnvironment,
) -> Result<RuntimeIrRunReport, RuntimeIrEvaluationError> {
    reject_runtime_ir_program_blockers(program)?;
    let observation = evaluate_runtime_ir_expr(&example.ir, env)?;
    let artifact = RuntimeArtifactIdentity::from_program(program);
    let target = RuntimeIrTargetIdentity::from_example(example);
    Ok(RuntimeIrRunReport {
        evaluator: RuntimeIrEvaluator::Nc12RuntimeIrEvaluatorV1,
        target: target.clone(),
        artifact: artifact.clone(),
        observation: RuntimeIrObservation {
            artifact,
            target,
            observation,
            evidence_source: "ken-runtime NC12 direct RuntimeExpr evaluator".to_string(),
        },
        evidence: RuntimeIrRunEvidence::from_program_and_example(program, example),
        trust: RuntimeIrTrustReport::observation(),
    })
}

pub fn evaluate_runtime_ir_expr(
    expr: &RuntimeExpr,
    env: &RuntimeIrSeedEnvironment,
) -> Result<RuntimeObservation, RuntimeIrEvaluationError> {
    let mut evaluator = RuntimeIrEvaluatorState { seed_env: env };
    Ok(match evaluator.eval_expr(expr, &[])? {
        RuntimeIrOutcome::Value(value) => RuntimeObservation::Returned(ground_value(value)?),
        RuntimeIrOutcome::Trap(trap) => RuntimeObservation::Trapped(trap),
    })
}

pub fn compare_runtime_ir_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &RuntimeIrSeedEnvironment,
    interpreter: RuntimeInterpreterObservation,
) -> RuntimeIrDifferentialReport {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    let target = RuntimeIrTargetIdentity::from_example(example);

    if interpreter.artifact != artifact {
        return RuntimeIrDifferentialReport {
            artifact,
            target,
            interpreter,
            runtime_ir: None,
            trust: RuntimeIrTrustReport::observation(),
            verdict: RuntimeIrDifferentialVerdict::Unsupported {
                stage: RuntimeIrDifferentialStage::BoundaryPreflight,
                construct: "RuntimeInterpreterObservation",
                reason: "interpreter observation artifact identity does not match RuntimeProgram"
                    .to_string(),
            },
        };
    }
    if interpreter.target != target {
        return RuntimeIrDifferentialReport {
            artifact,
            target,
            interpreter,
            runtime_ir: None,
            trust: RuntimeIrTrustReport::observation(),
            verdict: RuntimeIrDifferentialVerdict::Unsupported {
                stage: RuntimeIrDifferentialStage::BoundaryPreflight,
                construct: "RuntimeInterpreterObservation",
                reason: "interpreter observation target identity does not match RuntimeExample"
                    .to_string(),
            },
        };
    }

    let report = match evaluate_runtime_ir_example(program, example, env) {
        Ok(mut report) => {
            report.trust = RuntimeIrTrustReport::agreement();
            report
        }
        Err(err) => {
            return RuntimeIrDifferentialReport {
                artifact,
                target,
                interpreter,
                runtime_ir: None,
                trust: RuntimeIrTrustReport::observation(),
                verdict: RuntimeIrDifferentialVerdict::Unsupported {
                    stage: match err.stage {
                        RuntimeIrEvaluationStage::BoundaryPreflight => {
                            RuntimeIrDifferentialStage::BoundaryPreflight
                        }
                        RuntimeIrEvaluationStage::RuntimeIrEvaluation => {
                            RuntimeIrDifferentialStage::RuntimeIrEvaluation
                        }
                    },
                    construct: err.construct,
                    reason: err.reason,
                },
            };
        }
    };

    let trust = report.trust;
    let runtime_ir = report.observation;
    let verdict = if interpreter.observation == runtime_ir.observation {
        RuntimeIrDifferentialVerdict::Nc12InterpreterRuntimeIrAgreement {
            stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
        }
    } else {
        RuntimeIrDifferentialVerdict::Mismatch {
            stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
            interpreter: interpreter.observation.clone(),
            runtime_ir: runtime_ir.observation.clone(),
        }
    };

    RuntimeIrDifferentialReport {
        artifact,
        target,
        interpreter,
        runtime_ir: Some(runtime_ir),
        trust,
        verdict,
    }
}

pub fn reject_runtime_ir_program_blockers(
    program: &RuntimeProgram,
) -> Result<(), RuntimeIrEvaluationError> {
    if !program.erased_core.metadata.effects.is_empty() {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries effect metadata outside the NC12 supported subset",
        ));
    }
    if !program.erased_core.metadata.capabilities.is_empty() {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries capability metadata outside the NC12 supported subset",
        ));
    }
    if !program.erased_core.metadata.runtime_checks.is_empty() {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries runtime-check metadata outside the NC12 supported subset",
        ));
    }
    if !program.erased_core.metadata.assumptions.is_empty()
        || !program
            .erased_core
            .metadata
            .assumption_trust_metadata
            .is_empty()
        || !program.erased_core.metadata.trusted_base_delta.is_empty()
    {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries trust metadata outside the NC12 supported subset",
        ));
    }

    for declaration in &program.declarations {
        if declaration.metadata.unsupported.is_some()
            || program
                .erased_core
                .metadata
                .unsupported
                .contains_key(&declaration.symbol)
        {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!("reachable unsupported entry {}", declaration.symbol),
            ));
        }

        let lowerability = declaration
            .metadata
            .lowerability
            .as_ref()
            .or_else(|| {
                program
                    .erased_core
                    .metadata
                    .lowerability
                    .get(&declaration.symbol)
            })
            .ok_or_else(|| {
                preflight_unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} is missing runtime lowerability metadata",
                        declaration.symbol
                    ),
                )
            })?;
        if !matches!(lowerability, RuntimeLowerabilityStatus::Supported) {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} has blocking lowerability metadata: {:?}",
                    declaration.symbol, lowerability
                ),
            ));
        }

        if !declaration.metadata.effects.is_empty() {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries effect metadata outside the NC12 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.capabilities.is_empty() {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries capability metadata outside the NC12 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.runtime_checks.is_empty() {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries runtime-check metadata outside the NC12 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.assumptions.is_empty()
            || !declaration.metadata.assumption_trust_metadata.is_empty()
            || !declaration.metadata.trusted_base_delta.is_empty()
        {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries trust metadata outside the NC12 supported subset",
                    declaration.symbol
                ),
            ));
        }

        if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
            if !effects.is_empty() {
                return Err(preflight_unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} declares effects outside the NC12 supported subset",
                        declaration.symbol
                    ),
                ));
            }
        }

        if let Some(effect_meta) = program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .get(&declaration.symbol)
        {
            if effect_meta.boundary == RuntimeEffectBoundary::Foreign
                || effect_meta.boundary == RuntimeEffectBoundary::Effectful
                || effect_meta.foreign_symbol.is_some()
                || !effect_meta.declared_effects.is_empty()
                || !effect_meta.capabilities.is_empty()
                || !effect_meta.runtime_checks.is_empty()
            {
                return Err(preflight_unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} carries effects/foreign metadata outside the NC12 subset",
                        declaration.symbol
                    ),
                ));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RuntimeIrOutcome {
    Value(EvaluatedValue),
    Trap(RuntimeTrap),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EvaluatedValue {
    Bool(bool),
    Int(i64),
    Bytes(Vec<u8>),
    String(String),
    Constructor {
        constructor: RuntimeSymbol,
        args: Vec<EvaluatedValue>,
    },
    Record {
        fields: Vec<(String, EvaluatedValue)>,
    },
    Closure {
        captures: Vec<EvaluatedValue>,
        params: Vec<String>,
        body: RuntimeExpr,
    },
}

struct RuntimeIrEvaluatorState<'a> {
    seed_env: &'a RuntimeIrSeedEnvironment,
}

impl<'a> RuntimeIrEvaluatorState<'a> {
    fn eval_expr(
        &mut self,
        expr: &RuntimeExpr,
        env: &[EvaluatedValue],
    ) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
        match expr {
            RuntimeExpr::Value(value) => Ok(RuntimeIrOutcome::Value(self.eval_value(value)?)),
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .cloned()
                .map(RuntimeIrOutcome::Value)
                .ok_or_else(|| {
                    eval_unsupported("Var", format!("no runtime binding for index {index}"))
                }),
            RuntimeExpr::Let { value, body } => {
                let value = match value_or_trap(self.eval_expr(value, env)?)? {
                    Ok(value) => value,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                let mut body_env = vec![value];
                body_env.extend_from_slice(env);
                self.eval_expr(body, &body_env)
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let scrutinee = match value_or_trap(self.eval_expr(scrutinee, env)?)? {
                    Ok(value) => value,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                match scrutinee {
                    EvaluatedValue::Bool(true) => self.eval_expr(then_expr, env),
                    EvaluatedValue::Bool(false) => self.eval_expr(else_expr, env),
                    _ => Err(eval_unsupported("If", "scrutinee is not Bool")),
                }
            }
            RuntimeExpr::PrimitiveCall { primitive, args } => {
                self.eval_primitive_call(primitive, args, env)
            }
            RuntimeExpr::Construct { constructor, args } => {
                let args = match self.eval_value_args(args, env)? {
                    Ok(args) => args,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                Ok(RuntimeIrOutcome::Value(EvaluatedValue::Constructor {
                    constructor: constructor.clone(),
                    args,
                }))
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                let scrutinee = match value_or_trap(self.eval_expr(scrutinee, env)?)? {
                    Ok(value) => value,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                let EvaluatedValue::Constructor { constructor, args } = scrutinee else {
                    return Err(eval_unsupported(
                        "Match",
                        "scrutinee is not a constructor value",
                    ));
                };
                let Some(case) = cases.iter().find(|case| case.constructor == constructor) else {
                    return Ok(RuntimeIrOutcome::Trap(default.clone()));
                };
                if case.binders != args.len() {
                    return Err(eval_unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            args.len()
                        ),
                    ));
                }
                let mut case_env = args;
                case_env.extend_from_slice(env);
                self.eval_expr(&case.body, &case_env)
            }
            RuntimeExpr::Record { fields } => {
                let fields = match self.eval_record_fields(fields, env)? {
                    Ok(fields) => fields,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                Ok(RuntimeIrOutcome::Value(EvaluatedValue::Record { fields }))
            }
            RuntimeExpr::Project { record, field } => {
                let record = match value_or_trap(self.eval_expr(record, env)?)? {
                    Ok(value) => value,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                let EvaluatedValue::Record { fields } = record else {
                    return Err(eval_unsupported(
                        "Project",
                        "projection needs a record value",
                    ));
                };
                fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *field).then_some(value))
                    .map(RuntimeIrOutcome::Value)
                    .ok_or_else(|| eval_unsupported("Project", format!("missing field {field}")))
            }
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let captures = captures
                    .iter()
                    .map(|symbol| self.eval_seed_capture(symbol))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(RuntimeIrOutcome::Value(EvaluatedValue::Closure {
                    captures,
                    params: params.clone(),
                    body: (**body).clone(),
                }))
            }
            RuntimeExpr::Call { callee, args } => {
                let callee = match value_or_trap(self.eval_expr(callee, env)?)? {
                    Ok(value) => value,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                let EvaluatedValue::Closure {
                    captures,
                    params,
                    body,
                } = callee
                else {
                    return Err(eval_unsupported("Call", "callee is not a closure"));
                };
                if params.len() != args.len() {
                    return Err(eval_unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut call_env = match self.eval_value_args(args, env)? {
                    Ok(args) => args,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                call_env.extend(captures);
                call_env.extend_from_slice(env);
                self.eval_expr(&body, &call_env)
            }
            RuntimeExpr::Effect { effect, .. } => Err(eval_unsupported(
                "Effect",
                format!("effect {effect} is not modeled in the NC12 supported subset"),
            )),
            RuntimeExpr::Trap(trap) => Ok(RuntimeIrOutcome::Trap(trap.clone())),
        }
    }

    fn eval_value_args(
        &mut self,
        args: &[RuntimeExpr],
        env: &[EvaluatedValue],
    ) -> Result<Result<Vec<EvaluatedValue>, RuntimeTrap>, RuntimeIrEvaluationError> {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            match value_or_trap(self.eval_expr(arg, env)?)? {
                Ok(value) => values.push(value),
                Err(trap) => return Ok(Err(trap)),
            }
        }
        Ok(Ok(values))
    }

    fn eval_record_fields(
        &mut self,
        fields: &[(String, RuntimeExpr)],
        env: &[EvaluatedValue],
    ) -> Result<Result<Vec<(String, EvaluatedValue)>, RuntimeTrap>, RuntimeIrEvaluationError> {
        let mut values = Vec::with_capacity(fields.len());
        for (name, expr) in fields {
            match value_or_trap(self.eval_expr(expr, env)?)? {
                Ok(value) => values.push((name.clone(), value)),
                Err(trap) => return Ok(Err(trap)),
            }
        }
        Ok(Ok(values))
    }

    fn eval_value(
        &mut self,
        value: &RuntimeValue,
    ) -> Result<EvaluatedValue, RuntimeIrEvaluationError> {
        match value {
            RuntimeValue::Bool(value) => Ok(EvaluatedValue::Bool(*value)),
            RuntimeValue::Int(value) => Ok(EvaluatedValue::Int(*value)),
            RuntimeValue::Bytes(value) => Ok(EvaluatedValue::Bytes(value.clone())),
            RuntimeValue::String(value) => Ok(EvaluatedValue::String(value.clone())),
            RuntimeValue::Constructor { constructor, args } => Ok(EvaluatedValue::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.eval_value(arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeValue::Record { fields } => Ok(EvaluatedValue::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), self.eval_value(value)?)))
                    .collect::<Result<Vec<_>, RuntimeIrEvaluationError>>()?,
            }),
            RuntimeValue::ClosureRef { .. } => Err(eval_unsupported(
                "ClosureRef",
                "pre-existing closure references are not executable by the NC12 evaluator",
            )),
            RuntimeValue::Unknown => Err(eval_unsupported(
                "Unknown",
                "unknown runtime values must not evaluate successfully",
            )),
        }
    }

    fn eval_seed_capture(
        &mut self,
        symbol: &RuntimeSymbol,
    ) -> Result<EvaluatedValue, RuntimeIrEvaluationError> {
        let value = self.seed_env.values.get(symbol).ok_or_else(|| {
            eval_unsupported(
                "Closure",
                format!("capture {symbol} has no runtime value in the seed environment"),
            )
        })?;
        self.eval_ground_value(value)
    }

    fn eval_ground_value(
        &mut self,
        value: &RuntimeGroundValue,
    ) -> Result<EvaluatedValue, RuntimeIrEvaluationError> {
        match value {
            RuntimeGroundValue::Bool(value) => Ok(EvaluatedValue::Bool(*value)),
            RuntimeGroundValue::Int(value) => Ok(EvaluatedValue::Int(*value)),
            RuntimeGroundValue::Bytes(value) => Ok(EvaluatedValue::Bytes(value.clone())),
            RuntimeGroundValue::String(value) => Ok(EvaluatedValue::String(value.clone())),
            RuntimeGroundValue::Constructor { constructor, args } => {
                Ok(EvaluatedValue::Constructor {
                    constructor: constructor.clone(),
                    args: args
                        .iter()
                        .map(|arg| self.eval_ground_value(arg))
                        .collect::<Result<Vec<_>, _>>()?,
                })
            }
            RuntimeGroundValue::Record { fields } => Ok(EvaluatedValue::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), self.eval_ground_value(value)?)))
                    .collect::<Result<Vec<_>, RuntimeIrEvaluationError>>()?,
            }),
        }
    }

    fn eval_primitive_call(
        &mut self,
        primitive: &RuntimePrimitive,
        args: &[RuntimeExpr],
        env: &[EvaluatedValue],
    ) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
        let args = match self.eval_value_args(args, env)? {
            Ok(args) => args,
            Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
        };

        match &primitive.partiality {
            RuntimePartiality::Total => {}
            RuntimePartiality::CheckedTrap { obligation } => {
                let message = if obligation.ends_with(".bounds") {
                    format!("{} bounds obligation failed", primitive.symbol)
                } else {
                    format!("{} checked partiality trapped", primitive.symbol)
                };
                return Ok(RuntimeIrOutcome::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message,
                }));
            }
            RuntimePartiality::TrustedTrap { .. } => {
                return Ok(RuntimeIrOutcome::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: format!("{} trusted partiality trapped", primitive.symbol),
                }));
            }
        }

        match primitive.symbol.as_str() {
            "add_int" => {
                if args.len() != 2 {
                    return Err(eval_unsupported(
                        "PrimitiveCall",
                        format!("add_int expects 2 args, got {}", args.len()),
                    ));
                }
                let mut args = args.into_iter();
                let lhs = args.next().expect("arg count checked");
                let rhs = args.next().expect("arg count checked");
                let (EvaluatedValue::Int(lhs), EvaluatedValue::Int(rhs)) = (lhs, rhs) else {
                    return Err(eval_unsupported(
                        "PrimitiveCall",
                        "add_int only supports Int arguments in NC12",
                    ));
                };
                Ok(RuntimeIrOutcome::Value(EvaluatedValue::Int(lhs + rhs)))
            }
            other => Err(eval_unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the NC12 supported set"),
            )),
        }
    }
}

fn ground_value(value: EvaluatedValue) -> Result<RuntimeGroundValue, RuntimeIrEvaluationError> {
    match value {
        EvaluatedValue::Bool(value) => Ok(RuntimeGroundValue::Bool(value)),
        EvaluatedValue::Int(value) => Ok(RuntimeGroundValue::Int(value)),
        EvaluatedValue::Bytes(value) => Ok(RuntimeGroundValue::Bytes(value)),
        EvaluatedValue::String(value) => Ok(RuntimeGroundValue::String(value)),
        EvaluatedValue::Constructor { constructor, args } => Ok(RuntimeGroundValue::Constructor {
            constructor,
            args: args
                .into_iter()
                .map(ground_value)
                .collect::<Result<Vec<_>, _>>()?,
        }),
        EvaluatedValue::Record { fields } => Ok(RuntimeGroundValue::Record {
            fields: fields
                .into_iter()
                .map(|(name, value)| Ok((name, ground_value(value)?)))
                .collect::<Result<Vec<_>, RuntimeIrEvaluationError>>()?,
        }),
        EvaluatedValue::Closure { .. } => Err(eval_unsupported(
            "Closure",
            "closures are callable but not observable ground values in NC12",
        )),
    }
}

fn value_or_trap(
    outcome: RuntimeIrOutcome,
) -> Result<Result<EvaluatedValue, RuntimeTrap>, RuntimeIrEvaluationError> {
    match outcome {
        RuntimeIrOutcome::Value(value) => Ok(Ok(value)),
        RuntimeIrOutcome::Trap(trap) => Ok(Err(trap)),
    }
}

fn preflight_unsupported(
    construct: &'static str,
    reason: impl Into<String>,
) -> RuntimeIrEvaluationError {
    RuntimeIrEvaluationError {
        stage: RuntimeIrEvaluationStage::BoundaryPreflight,
        construct,
        reason: reason.into(),
    }
}

fn eval_unsupported(
    construct: &'static str,
    reason: impl Into<String>,
) -> RuntimeIrEvaluationError {
    RuntimeIrEvaluationError {
        stage: RuntimeIrEvaluationStage::RuntimeIrEvaluation,
        construct,
        reason: reason.into(),
    }
}
