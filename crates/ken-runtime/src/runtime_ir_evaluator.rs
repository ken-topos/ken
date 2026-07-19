//! Backend-neutral Runtime IR evaluator and interpreter comparison report.
//!
//! This module executes `RuntimeExpr` directly from the runtime artifact. It is
//! deliberately below the native/backend boundary: evaluator success is only a
//! runtime-IR observation, not kernel evidence, source-semantics proof, native
//! validation, Cranelift validation, object validation, linker validation, or
//! broader artifact-validation evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    RuntimeArtifactIdentity, RuntimeDeclaration, RuntimeDeclarationKind, RuntimeEffectBoundary,
    RuntimeEffectsForeignAuditMetadata, RuntimeExample, RuntimeExpr, RuntimeGroundValue,
    RuntimeLowerabilityStatus, RuntimeObservation, RuntimePartiality, RuntimePrimitive,
    RuntimeProgram, RuntimeSymbol, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
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
    DirectRuntimeIrEvaluatorV1,
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
    InterpreterRuntimeIrAgreement {
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
pub struct RuntimeIrProgramReport {
    pub artifact: RuntimeArtifactIdentity,
    pub supported_runtime_targets: BTreeSet<RuntimeSymbol>,
    pub comparison_unavailable_targets: BTreeMap<RuntimeSymbol, String>,
    pub unsupported_targets: BTreeMap<RuntimeSymbol, String>,
    pub evidence_sources: BTreeMap<String, String>,
    pub unavailable: BTreeSet<String>,
    pub native_phase_gate: RuntimeIrNativePhaseGate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeIrNativePhaseGate {
    ReadyForStarterKenOnlyExecutableSubset,
    Blocked { blockers: BTreeSet<String> },
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
    RuntimeIrObservation,
    InterpreterRuntimeIrAgreement,
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
    imported_values: BTreeMap<RuntimeImportedDeclarationIdentity, RuntimeGroundValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuntimeImportedDeclarationIdentity {
    pub symbol: RuntimeSymbol,
    pub dependency: RuntimeSymbol,
    pub dependency_semantic_hash: String,
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
            imported_values: BTreeMap::new(),
        }
    }

    pub fn closure_capture_seed() -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            "decl:fixture::Local::y".to_string(),
            RuntimeGroundValue::Int((2).into()),
        );
        Self {
            values,
            imported_values: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, symbol: impl Into<RuntimeSymbol>, value: RuntimeGroundValue) {
        self.values.insert(symbol.into(), value);
    }

    pub fn insert_imported_declaration(
        &mut self,
        symbol: impl Into<RuntimeSymbol>,
        dependency: impl Into<RuntimeSymbol>,
        dependency_semantic_hash: impl Into<String>,
        value: RuntimeGroundValue,
    ) {
        self.imported_values.insert(
            RuntimeImportedDeclarationIdentity {
                symbol: symbol.into(),
                dependency: dependency.into(),
                dependency_semantic_hash: dependency_semantic_hash.into(),
            },
            value,
        );
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
            "ken-runtime direct RuntimeExpr evaluator".to_string(),
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
            tier: RuntimeIrTrustTier::RuntimeIrObservation,
            evaluator: RuntimeIrEvidenceFact::Available {
                value: "direct RuntimeExpr evaluator".to_string(),
                evidence_source: "ken-runtime evaluated the RuntimeExpr without Cranelift"
                    .to_string(),
            },
            interpreter_oracle: RuntimeIrEvidenceFact::Unavailable {
                reason: "no interpreter oracle was supplied to this standalone evaluator run"
                    .to_string(),
            },
            native_backend: RuntimeIrEvidenceFact::Unavailable {
                reason: "direct runtime-IR evaluation does not invoke Cranelift or native code"
                    .to_string(),
            },
            object_artifact: RuntimeIrEvidenceFact::Unavailable {
                reason: "direct runtime-IR evaluation does not emit object artifacts".to_string(),
            },
            linker: RuntimeIrEvidenceFact::Unavailable {
                reason: "direct runtime-IR evaluation does not invoke a linker".to_string(),
            },
            source_level_proof: RuntimeIrEvidenceFact::Unavailable {
                reason: "runtime-IR observation is not a source-level semantics proof".to_string(),
            },
        }
    }

    fn agreement() -> Self {
        let mut report = Self::caller_supplied_interpreter_observation();
        report.tier = RuntimeIrTrustTier::InterpreterRuntimeIrAgreement;
        report
    }

    fn caller_supplied_interpreter_observation() -> Self {
        let mut report = Self::observation();
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
    reject_unbound_runtime_example(program, example)?;
    let observation = evaluate_runtime_ir_program_expr(program, &example.ir, env)?;
    let artifact = RuntimeArtifactIdentity::from_program(program);
    let target = RuntimeIrTargetIdentity::from_example(example);
    Ok(RuntimeIrRunReport {
        evaluator: RuntimeIrEvaluator::DirectRuntimeIrEvaluatorV1,
        target: target.clone(),
        artifact: artifact.clone(),
        observation: RuntimeIrObservation {
            artifact,
            target,
            observation,
            evidence_source: "ken-runtime direct RuntimeExpr evaluator".to_string(),
        },
        evidence: RuntimeIrRunEvidence::from_program_and_example(program, example),
        trust: RuntimeIrTrustReport::observation(),
    })
}

pub fn evaluate_runtime_ir_expr(
    expr: &RuntimeExpr,
    env: &RuntimeIrSeedEnvironment,
) -> Result<RuntimeObservation, RuntimeIrEvaluationError> {
    let mut evaluator = RuntimeIrEvaluatorState::standalone(env);
    Ok(match evaluator.eval_expr(expr, &[])? {
        RuntimeIrOutcome::Value(value) => RuntimeObservation::Returned(ground_value(value)?),
        RuntimeIrOutcome::Trap(trap) => RuntimeObservation::Trapped(trap),
    })
}

pub fn evaluate_runtime_ir_program_expr(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    env: &RuntimeIrSeedEnvironment,
) -> Result<RuntimeObservation, RuntimeIrEvaluationError> {
    let mut evaluator = RuntimeIrEvaluatorState::for_program(program, env);
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
        Ok(report) => report,
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

    let runtime_ir = report.observation;
    let (verdict, trust) = if interpreter.observation == runtime_ir.observation {
        (
            RuntimeIrDifferentialVerdict::InterpreterRuntimeIrAgreement {
                stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
            },
            RuntimeIrTrustReport::agreement(),
        )
    } else {
        (
            RuntimeIrDifferentialVerdict::Mismatch {
                stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
                interpreter: interpreter.observation.clone(),
                runtime_ir: runtime_ir.observation.clone(),
            },
            RuntimeIrTrustReport::caller_supplied_interpreter_observation(),
        )
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

pub fn summarize_runtime_ir_program(program: &RuntimeProgram) -> RuntimeIrProgramReport {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    let mut supported_runtime_targets = BTreeSet::new();
    let mut comparison_unavailable_targets = BTreeMap::new();
    let mut unsupported_targets = BTreeMap::new();
    let mut unavailable = BTreeSet::from([
        "native_backend_validation".to_string(),
        "object_artifact_validation".to_string(),
        "linker_validation".to_string(),
        "source_level_proof_validation".to_string(),
    ]);
    let mut blockers = BTreeSet::new();

    for declaration in &program.declarations {
        if declaration.metadata.unsupported.is_some()
            || program
                .erased_core
                .metadata
                .unsupported
                .contains_key(&declaration.symbol)
        {
            let reason = format!("{} is a reachable unsupported entry", declaration.symbol);
            unsupported_targets.insert(declaration.symbol.clone(), reason.clone());
            blockers.insert(reason);
            continue;
        }

        match declaration.metadata.lowerability.as_ref().or_else(|| {
            program
                .erased_core
                .metadata
                .lowerability
                .get(&declaration.symbol)
        }) {
            Some(RuntimeLowerabilityStatus::Supported) => {}
            Some(other) => {
                let reason = format!(
                    "{} has blocking lowerability metadata: {:?}",
                    declaration.symbol, other
                );
                unsupported_targets.insert(declaration.symbol.clone(), reason.clone());
                blockers.insert(reason);
                continue;
            }
            None => {
                let reason = format!(
                    "{} is missing runtime lowerability metadata",
                    declaration.symbol
                );
                unsupported_targets.insert(declaration.symbol.clone(), reason.clone());
                blockers.insert(reason);
                continue;
            }
        }

        if let Some(reason) = effect_foreign_metadata_inconsistency_reason(program, declaration) {
            unsupported_targets.insert(declaration.symbol.clone(), reason.clone());
            blockers.insert(reason);
            continue;
        }

        if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
            if runtime_expr_contains_effect(body) {
                let reason = unsupported_runtime_effect_reason(&declaration.symbol);
                unsupported_targets.insert(declaration.symbol.clone(), reason.clone());
                blockers.insert(reason);
                continue;
            }
        }

        if let Some(reason) = effect_foreign_unavailable_reason(program, declaration) {
            unavailable.insert(format!(
                "{} comparison unavailable: {reason}",
                declaration.symbol
            ));
            comparison_unavailable_targets.insert(declaration.symbol.clone(), reason);
        } else {
            supported_runtime_targets.insert(declaration.symbol.clone());
        }
    }

    let native_phase_gate = if blockers.is_empty() && !supported_runtime_targets.is_empty() {
        RuntimeIrNativePhaseGate::ReadyForStarterKenOnlyExecutableSubset
    } else {
        if supported_runtime_targets.is_empty() {
            blockers.insert(
                "no pure supported RuntimeProgram declaration is available for the starter native-codegen subset"
                    .to_string(),
            );
        }
        RuntimeIrNativePhaseGate::Blocked { blockers }
    };

    RuntimeIrProgramReport {
        artifact,
        supported_runtime_targets,
        comparison_unavailable_targets,
        unsupported_targets,
        evidence_sources: BTreeMap::from([
            (
                "runtime_artifact_identity".to_string(),
                "RuntimeProgram package/core/artifact identity from the exact runtime artifact"
                    .to_string(),
            ),
            (
                "checked_core_effect_foreign_metadata".to_string(),
                "RuntimeProgram.erased_core.metadata.checked_core.effects_foreign_metadata"
                    .to_string(),
            ),
        ]),
        unavailable,
        native_phase_gate,
    }
}

pub fn reject_runtime_ir_program_blockers(
    program: &RuntimeProgram,
) -> Result<(), RuntimeIrEvaluationError> {
    reject_effect_foreign_metadata_inconsistency(program)?;
    if !program.erased_core.metadata.effects.is_empty() {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries effect metadata outside the supported runtime-IR subset",
        ));
    }
    if !program.erased_core.metadata.capabilities.is_empty() {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries capability metadata outside the supported runtime-IR subset",
        ));
    }
    if !program.erased_core.metadata.runtime_checks.is_empty() {
        return Err(preflight_unsupported(
            "RuntimeProgram",
            "package carries runtime-check metadata outside the supported runtime-IR subset",
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
            "package carries trust metadata outside the supported runtime-IR subset",
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
                    "{} carries effect metadata outside the supported runtime-IR subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.capabilities.is_empty() {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries capability metadata outside the supported runtime-IR subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.runtime_checks.is_empty() {
            return Err(preflight_unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries runtime-check metadata outside the supported runtime-IR subset",
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
                    "{} carries trust metadata outside the supported runtime-IR subset",
                    declaration.symbol
                ),
            ));
        }

        if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
            if !effects.is_empty() {
                return Err(preflight_unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} declares effects outside the supported runtime-IR subset",
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
                        "{} carries effects/foreign metadata outside the supported runtime-IR subset",
                        declaration.symbol
                    ),
                ));
            }
        }

        if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
            reject_runtime_expr_blockers(program, body)?;
        }
    }
    for example in &program.examples {
        reject_runtime_expr_blockers(program, &example.ir)?;
    }
    Ok(())
}

fn reject_effect_foreign_metadata_inconsistency(
    program: &RuntimeProgram,
) -> Result<(), RuntimeIrEvaluationError> {
    for declaration in &program.declarations {
        if let Some(reason) = effect_foreign_metadata_inconsistency_reason(program, declaration) {
            return Err(stale_effect_foreign_metadata_error(reason));
        }
    }
    Ok(())
}

fn effect_foreign_metadata_inconsistency_reason(
    program: &RuntimeProgram,
    declaration: &RuntimeDeclaration,
) -> Option<String> {
    let effect_meta = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(&declaration.symbol)?;

    if effect_meta.declared_effects != declaration.metadata.effects {
        return Some(stale_effect_foreign_metadata_reason(
            &declaration.symbol,
            "effects",
        ));
    }
    if effect_meta.capabilities != declaration.metadata.capabilities {
        return Some(stale_effect_foreign_metadata_reason(
            &declaration.symbol,
            "capabilities",
        ));
    }
    if effect_meta.runtime_checks != declaration.metadata.runtime_checks {
        return Some(stale_effect_foreign_metadata_reason(
            &declaration.symbol,
            "runtime_checks",
        ));
    }
    if !program
        .erased_core
        .metadata
        .effects
        .is_superset(&effect_meta.declared_effects)
    {
        return Some(stale_effect_foreign_metadata_reason(
            &declaration.symbol,
            "package effects",
        ));
    }
    if !program
        .erased_core
        .metadata
        .capabilities
        .is_superset(&effect_meta.capabilities)
    {
        return Some(stale_effect_foreign_metadata_reason(
            &declaration.symbol,
            "package capabilities",
        ));
    }
    if !program
        .erased_core
        .metadata
        .runtime_checks
        .is_superset(&effect_meta.runtime_checks)
    {
        return Some(stale_effect_foreign_metadata_reason(
            &declaration.symbol,
            "package runtime checks",
        ));
    }
    None
}

fn stale_effect_foreign_metadata_reason(symbol: &RuntimeSymbol, lane: &'static str) -> String {
    format!("{symbol} has stale or missing effect/foreign authority metadata in {lane}")
}

fn stale_effect_foreign_metadata_error(reason: String) -> RuntimeIrEvaluationError {
    preflight_unsupported("RuntimeProgram", reason)
}

fn effect_foreign_unavailable_reason(
    program: &RuntimeProgram,
    declaration: &RuntimeDeclaration,
) -> Option<String> {
    let checked_meta = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(&declaration.symbol);
    checked_meta
        .and_then(effect_foreign_audit_unavailable_reason)
        .or_else(|| {
            (!declaration.metadata.effects.is_empty()
                || !declaration.metadata.capabilities.is_empty()
                || !declaration.metadata.runtime_checks.is_empty())
            .then(|| {
                "target carries effect, capability, or runtime-check metadata without executable runtime-IR evidence"
                    .to_string()
            })
        })
        .or_else(|| {
            if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
                (!effects.is_empty()).then(|| {
                    "target declares effect-boundary metadata without host-effect execution"
                        .to_string()
                })
            } else {
                None
            }
        })
}

fn effect_foreign_audit_unavailable_reason(
    meta: &RuntimeEffectsForeignAuditMetadata,
) -> Option<String> {
    if meta.boundary == RuntimeEffectBoundary::Foreign || meta.foreign_symbol.is_some() {
        Some(
            "foreign-boundary facts are represented, but host FFI execution is unavailable"
                .to_string(),
        )
    } else if meta.boundary == RuntimeEffectBoundary::Effectful
        || !meta.declared_effects.is_empty()
        || !meta.capabilities.is_empty()
        || !meta.runtime_checks.is_empty()
    {
        Some(
            "effect/capability/runtime-check facts are represented, but host-effect execution is unavailable"
                .to_string(),
        )
    } else {
        None
    }
}

fn reject_runtime_expr_blockers(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
) -> Result<(), RuntimeIrEvaluationError> {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
            reject_runtime_expr_blockers(program, body)
        }
        RuntimeExpr::Value(_) | RuntimeExpr::Var(_) | RuntimeExpr::Trap(_) => Ok(()),
        RuntimeExpr::Let { value, body } => {
            reject_runtime_expr_blockers(program, value)?;
            reject_runtime_expr_blockers(program, body)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            reject_runtime_expr_blockers(program, scrutinee)?;
            reject_runtime_expr_blockers(program, then_expr)?;
            reject_runtime_expr_blockers(program, else_expr)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                reject_runtime_expr_blockers(program, arg)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            reject_runtime_expr_blockers(program, scrutinee)?;
            for case in cases {
                reject_runtime_expr_blockers(program, &case.body)?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            reject_runtime_expr_blockers(program, scrutinee)?;
            for case in cases {
                reject_runtime_expr_blockers(program, &case.body)?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (_, value) in fields {
                reject_runtime_expr_blockers(program, value)?;
            }
            Ok(())
        }
        RuntimeExpr::Project { record, .. } => reject_runtime_expr_blockers(program, record),
        RuntimeExpr::Closure { body, .. } => reject_runtime_expr_blockers(program, body),
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for capture in captures {
                reject_runtime_expr_blockers(program, capture)?;
            }
            reject_runtime_expr_blockers(program, body)
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            require_referenced_symbol_supported(program, "DeclarationRef", symbol)?;
            if !program
                .declarations
                .iter()
                .any(|declaration| declaration.symbol == *symbol)
            {
                return Err(preflight_unsupported(
                    "DeclarationRef",
                    format!("{symbol} is not present in RuntimeProgram.declarations"),
                ));
            }
            Ok(())
        }
        RuntimeExpr::ImportedDeclarationRef {
            symbol,
            dependency,
            dependency_semantic_hash,
        } => {
            require_referenced_symbol_supported(program, "ImportedDeclarationRef", symbol)?;
            let expected = program
                .erased_core
                .metadata
                .dependency_semantic_hashes
                .get(dependency)
                .ok_or_else(|| {
                    preflight_unsupported(
                        "ImportedDeclarationRef",
                        format!(
                            "dependency {dependency} for imported declaration {symbol} is missing from RuntimeProgram metadata"
                        ),
                    )
                })?;
            if expected != dependency_semantic_hash {
                return Err(preflight_unsupported(
                    "ImportedDeclarationRef",
                    format!(
                        "dependency hash for imported declaration {symbol} is {dependency_semantic_hash:?}, expected {expected:?}"
                    ),
                ));
            }
            Ok(())
        }
        RuntimeExpr::Call { callee, args } => {
            reject_runtime_expr_blockers(program, callee)?;
            for arg in args {
                reject_runtime_expr_blockers(program, arg)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect { args, .. } => {
            for arg in args {
                reject_runtime_expr_blockers(program, arg)?;
            }
            Err(preflight_unsupported(
                "Effect",
                unsupported_runtime_effect_reason("RuntimeExpr::Effect"),
            ))
        }
    }
}

fn runtime_expr_contains_effect(expr: &RuntimeExpr) -> bool {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
            runtime_expr_contains_effect(body)
        }
        RuntimeExpr::Value(_) | RuntimeExpr::Var(_) | RuntimeExpr::Trap(_) => false,
        RuntimeExpr::Let { value, body } => {
            runtime_expr_contains_effect(value) || runtime_expr_contains_effect(body)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            runtime_expr_contains_effect(scrutinee)
                || runtime_expr_contains_effect(then_expr)
                || runtime_expr_contains_effect(else_expr)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            args.iter().any(runtime_expr_contains_effect)
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            runtime_expr_contains_effect(scrutinee)
                || cases
                    .iter()
                    .any(|case| runtime_expr_contains_effect(&case.body))
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            runtime_expr_contains_effect(scrutinee)
                || cases
                    .iter()
                    .any(|case| runtime_expr_contains_effect(&case.body))
        }
        RuntimeExpr::Record { fields } => fields
            .iter()
            .any(|(_, value)| runtime_expr_contains_effect(value)),
        RuntimeExpr::Project { record, .. } => runtime_expr_contains_effect(record),
        RuntimeExpr::Closure { body, .. } => runtime_expr_contains_effect(body),
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            captures.iter().any(runtime_expr_contains_effect) || runtime_expr_contains_effect(body)
        }
        RuntimeExpr::DeclarationRef { .. } | RuntimeExpr::ImportedDeclarationRef { .. } => false,
        RuntimeExpr::Call { callee, args } => {
            runtime_expr_contains_effect(callee) || args.iter().any(runtime_expr_contains_effect)
        }
        RuntimeExpr::Effect { .. } => true,
    }
}

fn unsupported_runtime_effect_reason(subject: &str) -> String {
    format!(
        "{subject} contains RuntimeExpr::Effect, which is outside supported runtime-IR execution"
    )
}

fn require_referenced_symbol_supported(
    program: &RuntimeProgram,
    construct: &'static str,
    symbol: &RuntimeSymbol,
) -> Result<(), RuntimeIrEvaluationError> {
    let lowerability = program
        .erased_core
        .metadata
        .lowerability
        .get(symbol)
        .ok_or_else(|| {
            preflight_unsupported(
                construct,
                format!("{symbol} is missing runtime lowerability metadata"),
            )
        })?;
    if !matches!(lowerability, RuntimeLowerabilityStatus::Supported) {
        return Err(preflight_unsupported(
            construct,
            format!("{symbol} has blocking lowerability metadata: {lowerability:?}"),
        ));
    }
    Ok(())
}

fn reject_unbound_runtime_example(
    program: &RuntimeProgram,
    example: &RuntimeExample,
) -> Result<(), RuntimeIrEvaluationError> {
    if program
        .examples
        .iter()
        .any(|candidate| candidate == example)
    {
        return Ok(());
    }

    Err(preflight_unsupported(
        "RuntimeExample",
        format!(
            "example {} is not present byte-for-byte in RuntimeProgram.examples for the exact runtime artifact",
            example.name
        ),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum RuntimeIrOutcome {
    Value(EvaluatedValue),
    Trap(RuntimeTrap),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum EvaluatedValue {
    Bool(bool),
    Int(crate::RuntimeIntV1),
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
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    declaration_stack: Vec<RuntimeSymbol>,
}

impl<'a> RuntimeIrEvaluatorState<'a> {
    fn standalone(seed_env: &'a RuntimeIrSeedEnvironment) -> Self {
        Self {
            seed_env,
            declarations: BTreeMap::new(),
            declaration_stack: Vec::new(),
        }
    }

    fn for_program(program: &'a RuntimeProgram, seed_env: &'a RuntimeIrSeedEnvironment) -> Self {
        Self {
            seed_env,
            declarations: program
                .declarations
                .iter()
                .map(|declaration| (declaration.symbol.as_str(), declaration))
                .collect(),
            declaration_stack: Vec::new(),
        }
    }

    fn eval_expr(
        &mut self,
        expr: &RuntimeExpr,
        env: &[EvaluatedValue],
    ) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
        match expr {
            RuntimeExpr::CheckedJoinSite { body, .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
            | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
            | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => self.eval_expr(body, env),
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
            RuntimeExpr::ComputationalMatch { .. } => Err(eval_unsupported(
                "ComputationalMatch",
                "computational recursive hypotheses are native-process lowering only",
            )),
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
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                let captures = match self.eval_value_args(captures, env)? {
                    Ok(captures) => captures,
                    Err(trap) => return Ok(RuntimeIrOutcome::Trap(trap)),
                };
                Ok(RuntimeIrOutcome::Value(EvaluatedValue::Closure {
                    captures,
                    params: params.clone(),
                    body: (**body).clone(),
                }))
            }
            RuntimeExpr::DeclarationRef { symbol } => self.eval_declaration_ref(symbol),
            RuntimeExpr::ImportedDeclarationRef {
                symbol,
                dependency,
                dependency_semantic_hash,
            } => self.eval_imported_declaration_ref(symbol, dependency, dependency_semantic_hash),
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
            RuntimeExpr::Effect {
                family, operation, ..
            } => Err(eval_unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is not modeled in the supported runtime-IR subset",
                    *operation as u16
                ),
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
            RuntimeValue::Int(value) => Ok(EvaluatedValue::Int(value.clone())),
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
                "pre-existing closure references are not executable by the direct runtime-IR evaluator",
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

    fn eval_declaration_ref(
        &mut self,
        symbol: &RuntimeSymbol,
    ) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
        let kind = self
            .declarations
            .get(symbol.as_str())
            .map(|declaration| declaration.kind.clone())
            .ok_or_else(|| {
                eval_unsupported(
                    "DeclarationRef",
                    format!(
                        "declaration {symbol} is not present in the exact RuntimeProgram artifact"
                    ),
                )
            })?;
        if self.declaration_stack.len() >= 1024 {
            return Err(eval_unsupported(
                "DeclarationRef",
                format!("declaration reference depth exceeded while evaluating {symbol}"),
            ));
        }
        self.declaration_stack.push(symbol.clone());
        let result = match kind {
            RuntimeDeclarationKind::Transparent { body } => self.eval_expr(&body, &[]),
            RuntimeDeclarationKind::Primitive { op } => Err(eval_unsupported(
                "DeclarationRef",
                format!(
                    "primitive declaration {} is not a first-class runtime value",
                    op.symbol
                ),
            )),
            RuntimeDeclarationKind::Data { .. }
            | RuntimeDeclarationKind::Record { .. }
            | RuntimeDeclarationKind::RecursiveGroup { .. }
            | RuntimeDeclarationKind::EffectBoundary { .. }
            | RuntimeDeclarationKind::MetadataOnly => Err(eval_unsupported(
                "DeclarationRef",
                format!("{symbol} is metadata, not an executable transparent body"),
            )),
        };
        self.declaration_stack.pop();
        result
    }

    fn eval_imported_declaration_ref(
        &mut self,
        symbol: &RuntimeSymbol,
        dependency: &RuntimeSymbol,
        dependency_semantic_hash: &str,
    ) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
        let identity = RuntimeImportedDeclarationIdentity {
            symbol: symbol.clone(),
            dependency: dependency.clone(),
            dependency_semantic_hash: dependency_semantic_hash.to_string(),
        };
        let value = self.seed_env.imported_values.get(&identity).ok_or_else(|| {
            eval_unsupported(
                "ImportedDeclarationRef",
                format!(
                    "imported declaration {symbol} from {dependency} @ {dependency_semantic_hash} has no exact runtime seed binding"
                ),
            )
        })?;
        Ok(RuntimeIrOutcome::Value(self.eval_ground_value(value)?))
    }

    fn eval_ground_value(
        &mut self,
        value: &RuntimeGroundValue,
    ) -> Result<EvaluatedValue, RuntimeIrEvaluationError> {
        match value {
            RuntimeGroundValue::Bool(value) => Ok(EvaluatedValue::Bool(*value)),
            RuntimeGroundValue::Int(value) => Ok(EvaluatedValue::Int(value.clone())),
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
            RuntimePartiality::SafeOption { .. } | RuntimePartiality::SafeResult { .. } => {}
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
            "add_int" => eval_int_binop(&primitive.symbol, args, crate::RuntimeIntV1::add),
            "sub_int" => eval_int_binop(&primitive.symbol, args, crate::RuntimeIntV1::sub),
            "mul_int" => eval_int_binop(&primitive.symbol, args, crate::RuntimeIntV1::mul),
            "eq_int" => eval_int_cmp(&primitive.symbol, args, |lhs, rhs| lhs == rhs),
            "leq_int" => eval_int_cmp(&primitive.symbol, args, |lhs, rhs| {
                lhs.exact_cmp(rhs).is_le()
            }),
            "uint8_to_int" | "int_to_uint8_raw" => {
                let [value]: [EvaluatedValue; 1] = args.try_into().map_err(|args: Vec<_>| {
                    eval_unsupported(
                        "PrimitiveCall",
                        format!(
                            "{} expects one argument, got {}",
                            primitive.symbol,
                            args.len()
                        ),
                    )
                })?;
                let EvaluatedValue::Int(_) = value else {
                    return Err(eval_unsupported(
                        "PrimitiveCall",
                        format!("{} expects an Int-represented value", primitive.symbol),
                    ));
                };
                Ok(RuntimeIrOutcome::Value(value))
            }
            "not_bool" => eval_bool_unop(&primitive.symbol, args, |value| !value),
            "and_bool" => eval_bool_binop(&primitive.symbol, args, |lhs, rhs| lhs && rhs),
            "or_bool" => eval_bool_binop(&primitive.symbol, args, |lhs, rhs| lhs || rhs),
            "bytes_length" => eval_bytes_length(&primitive.symbol, args),
            "bytes_at" => eval_bytes_at(&primitive.symbol, args, &primitive.partiality),
            "bytes_slice" => eval_bytes_slice(&primitive.symbol, args, &primitive.partiality),
            "bytes_concat" => eval_bytes_concat(&primitive.symbol, args),
            "bytes_encode" => eval_bytes_encode(&primitive.symbol, args),
            "bytes_decode" => eval_bytes_decode(&primitive.symbol, args, &primitive.partiality),
            "byte_length" => eval_string_byte_length(&primitive.symbol, args),
            "char_length" => eval_string_char_length(&primitive.symbol, args),
            other => Err(eval_unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the supported runtime-IR set"),
            )),
        }
    }
}

fn eval_int_binop(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    op: impl FnOnce(&crate::RuntimeIntV1, &crate::RuntimeIntV1) -> crate::RuntimeIntV1,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let (lhs, rhs) = expect_two_ints(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Int(op(&lhs, &rhs))))
}

fn eval_int_cmp(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    op: impl FnOnce(&crate::RuntimeIntV1, &crate::RuntimeIntV1) -> bool,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let (lhs, rhs) = expect_two_ints(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Bool(op(
        &lhs, &rhs,
    ))))
}

fn eval_bool_unop(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    op: impl FnOnce(bool) -> bool,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let value = expect_one_bool(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Bool(op(value))))
}

fn eval_bool_binop(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    op: impl FnOnce(bool, bool) -> bool,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let (lhs, rhs) = expect_two_bools(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Bool(op(lhs, rhs))))
}

fn eval_bytes_length(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let bytes = expect_one_bytes(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Int(
        usize_to_i64(symbol, bytes.len())?.into(),
    )))
}

fn eval_bytes_at(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    partiality: &RuntimePartiality,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let RuntimePartiality::SafeOption { none, some, .. } = partiality else {
        return Err(eval_unsupported(
            "PrimitiveCall",
            format!("{symbol} requires safe Option result metadata"),
        ));
    };
    let (bytes, index) = expect_bytes_int(symbol, args)?;
    let value = usize::try_from(index)
        .ok()
        .and_then(|index| bytes.get(index).copied());
    Ok(RuntimeIrOutcome::Value(match value {
        Some(byte) => EvaluatedValue::Constructor {
            constructor: some.clone(),
            args: vec![EvaluatedValue::Int(i64::from(byte).into())],
        },
        None => EvaluatedValue::Constructor {
            constructor: none.clone(),
            args: Vec::new(),
        },
    }))
}

fn eval_bytes_slice(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    partiality: &RuntimePartiality,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let RuntimePartiality::SafeOption { none, some, .. } = partiality else {
        return Err(eval_unsupported(
            "PrimitiveCall",
            format!("{symbol} requires safe Option result metadata"),
        ));
    };
    let (bytes, start, len) = expect_bytes_int_int(symbol, args)?;
    let value = usize::try_from(start)
        .ok()
        .zip(usize::try_from(len).ok())
        .and_then(|(start, len)| {
            start
                .checked_add(len)
                .filter(|end| *end <= bytes.len())
                .map(|end| bytes[start..end].to_vec())
        });
    Ok(RuntimeIrOutcome::Value(match value {
        Some(bytes) => EvaluatedValue::Constructor {
            constructor: some.clone(),
            args: vec![EvaluatedValue::Bytes(bytes)],
        },
        None => EvaluatedValue::Constructor {
            constructor: none.clone(),
            args: Vec::new(),
        },
    }))
}

fn eval_bytes_concat(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let (left, right) = expect_two_bytes(symbol, args)?;
    let mut out = Vec::with_capacity(left.len().checked_add(right.len()).ok_or_else(|| {
        eval_unsupported(
            "PrimitiveCall",
            format!("{symbol} output length overflows the host addressable subset"),
        )
    })?);
    out.extend_from_slice(&left);
    out.extend_from_slice(&right);
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Bytes(out)))
}

fn eval_bytes_encode(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let value = expect_one_string(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Bytes(
        value.into_bytes(),
    )))
}

fn eval_bytes_decode(
    symbol: &str,
    args: Vec<EvaluatedValue>,
    partiality: &RuntimePartiality,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let RuntimePartiality::SafeResult { err, ok, error } = partiality else {
        return Err(eval_unsupported(
            "PrimitiveCall",
            format!("{symbol} requires safe Result metadata"),
        ));
    };
    let bytes = expect_one_bytes(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(match String::from_utf8(bytes) {
        Ok(value) => EvaluatedValue::Constructor {
            constructor: ok.clone(),
            args: vec![EvaluatedValue::String(value)],
        },
        Err(_) => EvaluatedValue::Constructor {
            constructor: err.clone(),
            args: vec![EvaluatedValue::Constructor {
                constructor: error.clone(),
                args: Vec::new(),
            }],
        },
    }))
}

fn eval_string_byte_length(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let value = expect_one_string(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Int(
        usize_to_i64(symbol, value.len())?.into(),
    )))
}

fn eval_string_char_length(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<RuntimeIrOutcome, RuntimeIrEvaluationError> {
    let value = expect_one_string(symbol, args)?;
    Ok(RuntimeIrOutcome::Value(EvaluatedValue::Int(
        usize_to_i64(symbol, value.chars().count())?.into(),
    )))
}

fn expect_two_ints(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<(crate::RuntimeIntV1, crate::RuntimeIntV1), RuntimeIrEvaluationError> {
    if args.len() != 2 {
        return Err(wrong_arity(symbol, 2, args.len()));
    }
    let mut args = args.into_iter();
    let lhs = args.next().expect("arg count checked");
    let rhs = args.next().expect("arg count checked");
    let (EvaluatedValue::Int(lhs), EvaluatedValue::Int(rhs)) = (lhs, rhs) else {
        return Err(wrong_type(symbol, "Int, Int"));
    };
    Ok((lhs, rhs))
}

fn expect_one_bool(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<bool, RuntimeIrEvaluationError> {
    if args.len() != 1 {
        return Err(wrong_arity(symbol, 1, args.len()));
    }
    let mut args = args.into_iter();
    let EvaluatedValue::Bool(value) = args.next().expect("arg count checked") else {
        return Err(wrong_type(symbol, "Bool"));
    };
    Ok(value)
}

fn expect_two_bools(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<(bool, bool), RuntimeIrEvaluationError> {
    if args.len() != 2 {
        return Err(wrong_arity(symbol, 2, args.len()));
    }
    let mut args = args.into_iter();
    let lhs = args.next().expect("arg count checked");
    let rhs = args.next().expect("arg count checked");
    let (EvaluatedValue::Bool(lhs), EvaluatedValue::Bool(rhs)) = (lhs, rhs) else {
        return Err(wrong_type(symbol, "Bool, Bool"));
    };
    Ok((lhs, rhs))
}

fn expect_one_bytes(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<Vec<u8>, RuntimeIrEvaluationError> {
    if args.len() != 1 {
        return Err(wrong_arity(symbol, 1, args.len()));
    }
    let mut args = args.into_iter();
    let EvaluatedValue::Bytes(value) = args.next().expect("arg count checked") else {
        return Err(wrong_type(symbol, "Bytes"));
    };
    Ok(value)
}

fn expect_two_bytes(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<(Vec<u8>, Vec<u8>), RuntimeIrEvaluationError> {
    if args.len() != 2 {
        return Err(wrong_arity(symbol, 2, args.len()));
    }
    let mut args = args.into_iter();
    let lhs = args.next().expect("arg count checked");
    let rhs = args.next().expect("arg count checked");
    let (EvaluatedValue::Bytes(lhs), EvaluatedValue::Bytes(rhs)) = (lhs, rhs) else {
        return Err(wrong_type(symbol, "Bytes, Bytes"));
    };
    Ok((lhs, rhs))
}

fn expect_bytes_int(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<(Vec<u8>, i64), RuntimeIrEvaluationError> {
    if args.len() != 2 {
        return Err(wrong_arity(symbol, 2, args.len()));
    }
    let mut args = args.into_iter();
    let bytes = args.next().expect("arg count checked");
    let index = args.next().expect("arg count checked");
    let (EvaluatedValue::Bytes(bytes), EvaluatedValue::Int(index)) = (bytes, index) else {
        return Err(wrong_type(symbol, "Bytes, Int"));
    };
    let Some(index) = index.as_small() else {
        return Err(wrong_type(symbol, "Bytes, host-width Int"));
    };
    Ok((bytes, index))
}

fn expect_bytes_int_int(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<(Vec<u8>, i64, i64), RuntimeIrEvaluationError> {
    if args.len() != 3 {
        return Err(wrong_arity(symbol, 3, args.len()));
    }
    let mut args = args.into_iter();
    let bytes = args.next().expect("arg count checked");
    let start = args.next().expect("arg count checked");
    let len = args.next().expect("arg count checked");
    let (EvaluatedValue::Bytes(bytes), EvaluatedValue::Int(start), EvaluatedValue::Int(len)) =
        (bytes, start, len)
    else {
        return Err(wrong_type(symbol, "Bytes, Int, Int"));
    };
    let (Some(start), Some(len)) = (start.as_small(), len.as_small()) else {
        return Err(wrong_type(symbol, "Bytes, host-width Int, host-width Int"));
    };
    Ok((bytes, start, len))
}

fn expect_one_string(
    symbol: &str,
    args: Vec<EvaluatedValue>,
) -> Result<String, RuntimeIrEvaluationError> {
    if args.len() != 1 {
        return Err(wrong_arity(symbol, 1, args.len()));
    }
    let mut args = args.into_iter();
    let EvaluatedValue::String(value) = args.next().expect("arg count checked") else {
        return Err(wrong_type(symbol, "String"));
    };
    Ok(value)
}

fn usize_to_i64(symbol: &str, value: usize) -> Result<i64, RuntimeIrEvaluationError> {
    i64::try_from(value).map_err(|_| {
        eval_unsupported(
            "PrimitiveCall",
            format!("{symbol} length exceeds the supported Int runtime subset"),
        )
    })
}

fn wrong_arity(symbol: &str, expected: usize, found: usize) -> RuntimeIrEvaluationError {
    eval_unsupported(
        "PrimitiveCall",
        format!("{symbol} expects {expected} args, got {found}"),
    )
}

fn wrong_type(symbol: &str, expected: &'static str) -> RuntimeIrEvaluationError {
    eval_unsupported(
        "PrimitiveCall",
        format!("{symbol} expects {expected} arguments in the supported runtime-IR subset"),
    )
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
            "closures are callable but not observable ground values in the supported runtime-IR subset",
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
