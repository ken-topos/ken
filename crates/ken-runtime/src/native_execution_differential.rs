//! Native execution differential reports for packaged starter executables.
//!
//! NC24 consumes compiler-produced NC23 object/linker packages and compares
//! exact native execution observations against runtime-IR evaluator reports and
//! interpreter observations when that lane is available. NC25 carries NC18
//! effect/foreign facts through that report surface so host-effect and FFI
//! execution stay explicitly unavailable unless a later policy makes them
//! executable. The report is tested evidence only: it does not claim translation
//! validation, proof, library ABI, C/Rust interop, or foreign execution support.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    fnv1a_64, object_linker_executable_package_hash, object_linker_runtime_ir_run_report_hash,
    ObjectLinkerArtifactKind, ObjectLinkerExecutablePackage, RuntimeArtifactIdentity,
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExpr,
    RuntimeGroundValue, RuntimeInterpreterObservation, RuntimeIrRunReport, RuntimeIrTargetIdentity,
    RuntimeLowerabilityStatus, RuntimeObservation, RuntimeProgram, RuntimeSymbol,
    OBJECT_LINKER_PACKAGE_KIND, OBJECT_LINKER_PACKAGE_VERSION,
};

pub const NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND: &str = "KenNativeExecutionDifferentialReport";
pub const NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION: u32 = 1;
pub const NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF: &str =
    "docs/program/wp/NC25-effects-foreign-executable-policy.md";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionDifferentialReport {
    pub header: NativeExecutionDifferentialHeader,
    pub target: NativeExecutionTargetIdentity,
    pub native: NativeExecutionLaneReport,
    pub runtime_ir: NativeComparisonLaneReport,
    pub interpreter: NativeComparisonLaneReport,
    pub verdict: NativeExecutionDifferentialVerdict,
    pub effect_foreign_policy: NativeEffectForeignExecutablePolicyReport,
    pub unavailable_claims: BTreeSet<NativeExecutionUnavailableClaim>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionDifferentialHeader {
    pub report_kind: String,
    pub version: u32,
    pub spec_ref: String,
    pub producer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionTargetIdentity {
    pub package_identity: String,
    pub target_symbol: RuntimeSymbol,
    pub runtime_artifact: RuntimeArtifactIdentity,
    pub runtime_report_hash: u64,
    pub object_linker_package_hash: u64,
    pub executable_artifact_hash: u64,
    pub executable_relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionObservation {
    pub observation: RuntimeObservation,
    pub stdout: String,
    pub exit_status: i32,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutionLaneReport {
    Available(NativeExecutionObservation),
    Unavailable {
        reason: String,
        evidence_source: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeComparisonLaneReport {
    TestedAgreement {
        lane: NativeDifferentialLane,
        expected: RuntimeObservation,
        observed: RuntimeObservation,
        evidence_source: String,
    },
    Mismatch {
        lane: NativeDifferentialLane,
        expected: RuntimeObservation,
        observed: RuntimeObservation,
        diagnostic: NativeMismatchDiagnostic,
    },
    Unavailable {
        lane: NativeDifferentialLane,
        reason: String,
        evidence_source: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutionDifferentialVerdict {
    RuntimeIrTestedAgreement {
        runtime_ir: NativeLaneVerdict,
        interpreter: NativeLaneVerdict,
    },
    Mismatch {
        lane: NativeDifferentialLane,
        diagnostic: NativeMismatchDiagnostic,
    },
    Unavailable {
        lane: NativeDifferentialLane,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeLaneVerdict {
    TestedAgreement,
    Unavailable { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeEffectForeignExecutablePolicyReport {
    pub target_symbol: RuntimeSymbol,
    pub status: NativeEffectForeignExecutableStatus,
    pub facts: BTreeSet<String>,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeEffectForeignExecutableStatus {
    NativeTested,
    RepresentedUnavailable { reason: String },
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeMismatchDiagnostic {
    pub package_identity: String,
    pub target_symbol: RuntimeSymbol,
    pub executable_artifact_hash: u64,
    pub lane: NativeDifferentialLane,
    pub expected: RuntimeObservation,
    pub observed: RuntimeObservation,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeInterpreterLaneInput {
    Available(RuntimeInterpreterObservation),
    Unavailable {
        reason: String,
        evidence_source: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeDifferentialLane {
    NativeExecution,
    RuntimeIrEvaluator,
    Interpreter,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeExecutionUnavailableClaim {
    LibraryAbi,
    CAbiInterop,
    RustInterop,
    ForeignExecution,
    EffectPolicyBroadening,
    TranslationValidation,
    WholeCompilerProof,
}

pub struct NativeExecutionDifferentialCase<'a> {
    pub program: &'a RuntimeProgram,
    pub package: &'a ObjectLinkerExecutablePackage,
    pub run_report: &'a RuntimeIrRunReport,
    pub artifact_root: &'a Path,
    pub interpreter: NativeInterpreterLaneInput,
    pub producer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionDifferentialError {
    pub stage: NativeExecutionDifferentialStage,
    pub field: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutionDifferentialStage {
    PackageIdentity,
    RuntimeIrRunReport,
    ArtifactFile,
    EffectForeignExecutablePolicy,
    NativeExecution,
    InterpreterEvidence,
}

impl fmt::Display for NativeExecutionDifferentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.field, self.reason)
    }
}

impl std::error::Error for NativeExecutionDifferentialError {}

pub fn run_native_execution_differential(
    program: &RuntimeProgram,
    package: &ObjectLinkerExecutablePackage,
    run_report: &RuntimeIrRunReport,
    artifact_root: impl AsRef<Path>,
    interpreter: NativeInterpreterLaneInput,
    producer: impl Into<String>,
) -> Result<NativeExecutionDifferentialReport, NativeExecutionDifferentialError> {
    validate_object_linker_package(program, package)?;
    let target_example = validate_runtime_ir_report(program, package, run_report)?;
    let target = NativeExecutionTargetIdentity {
        package_identity: program.package_identity.clone(),
        target_symbol: package.header.target_symbol.clone(),
        runtime_artifact: RuntimeArtifactIdentity::from_program(program),
        runtime_report_hash: package.runtime_report_hash,
        object_linker_package_hash: package.header.package_hash,
        executable_artifact_hash: package.executable_artifact.artifact_hash,
        executable_relative_path: package.executable_artifact.relative_path.clone(),
    };
    let expected_target = RuntimeIrTargetIdentity {
        example: target_example.name.clone(),
        checked_core_shape: target_example.checked_core_shape.clone(),
    };
    let executable_path = validate_executable_artifact(package, artifact_root.as_ref())?;
    let effect_foreign_policy =
        effect_foreign_executable_policy_report(program, &package.header.target_symbol)?;

    if let Some(reason) = effect_foreign_policy_unavailable_reason(&effect_foreign_policy) {
        let interpreter = unavailable_interpreter_for_policy(
            &target,
            &expected_target,
            interpreter,
            &run_report.observation.observation,
            &reason,
        )?;
        return Ok(NativeExecutionDifferentialReport {
            header: NativeExecutionDifferentialHeader {
                report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
                version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
                spec_ref: NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF.to_string(),
                producer: producer.into(),
            },
            target,
            native: NativeExecutionLaneReport::Unavailable {
                reason: reason.clone(),
                evidence_source:
                    "NC25 effect/foreign executable policy rejected native execution before launch"
                        .to_string(),
            },
            runtime_ir: NativeComparisonLaneReport::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                reason: reason.clone(),
                evidence_source:
                    "NC18 effect/foreign facts are represented; NC25 keeps native execution unavailable"
                        .to_string(),
            },
            interpreter,
            verdict: NativeExecutionDifferentialVerdict::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                reason,
            },
            effect_foreign_policy,
            unavailable_claims: required_unavailable_claims(),
        });
    }

    if matches!(
        run_report.observation.observation,
        RuntimeObservation::Trapped(_)
    ) {
        let interpreter = unavailable_interpreter_for_trap(
            &target,
            &expected_target,
            interpreter,
            &run_report.observation.observation,
        )?;
        let reason = "NC24 starter native execution cannot decode trap reports from the NC23 scalar executable ABI; trap comparison remains first-class unavailable".to_string();
        return Ok(NativeExecutionDifferentialReport {
            header: NativeExecutionDifferentialHeader {
                report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
                version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
                spec_ref: NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF.to_string(),
                producer: producer.into(),
            },
            target,
            native: NativeExecutionLaneReport::Unavailable {
                reason: reason.clone(),
                evidence_source: "RuntimeIrRunReport observed a trap; NC23 executable ABI carries scalar stdout only"
                    .to_string(),
            },
            runtime_ir: NativeComparisonLaneReport::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                reason: reason.clone(),
                evidence_source:
                    "runtime-IR trap observation is preserved but native trap decoding is unavailable"
                        .to_string(),
            },
            interpreter,
            verdict: NativeExecutionDifferentialVerdict::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                reason,
            },
            effect_foreign_policy,
            unavailable_claims: required_unavailable_claims(),
        });
    }

    let native = run_packaged_executable(&executable_path, &run_report.observation.observation)?;

    let runtime_ir = compare_runtime_ir_lane(
        &target,
        &run_report.observation.observation,
        &native.observation,
    );
    let interpreter =
        compare_interpreter_lane(&target, &expected_target, interpreter, &native.observation)?;
    let verdict = differential_verdict(&runtime_ir, &interpreter);

    Ok(NativeExecutionDifferentialReport {
        header: NativeExecutionDifferentialHeader {
            report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
            version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
            spec_ref: NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF.to_string(),
            producer: producer.into(),
        },
        target,
        native: NativeExecutionLaneReport::Available(native),
        runtime_ir,
        interpreter,
        verdict,
        effect_foreign_policy,
        unavailable_claims: required_unavailable_claims(),
    })
}

pub fn run_native_execution_differential_suite<'a, I>(
    cases: I,
) -> Result<Vec<NativeExecutionDifferentialReport>, NativeExecutionDifferentialError>
where
    I: IntoIterator<Item = NativeExecutionDifferentialCase<'a>>,
{
    cases
        .into_iter()
        .map(|case| {
            run_native_execution_differential(
                case.program,
                case.package,
                case.run_report,
                case.artifact_root,
                case.interpreter,
                case.producer,
            )
        })
        .collect()
}

fn validate_object_linker_package(
    program: &RuntimeProgram,
    package: &ObjectLinkerExecutablePackage,
) -> Result<(), NativeExecutionDifferentialError> {
    if package.header.package_kind != OBJECT_LINKER_PACKAGE_KIND {
        return Err(differential_error(
            NativeExecutionDifferentialStage::PackageIdentity,
            "package_kind",
            "object/linker package kind is not KenObjectLinkerExecutablePackage",
        ));
    }
    if package.header.version != OBJECT_LINKER_PACKAGE_VERSION {
        return Err(differential_error(
            NativeExecutionDifferentialStage::PackageIdentity,
            "version",
            "object/linker package version is unsupported by NC24",
        ));
    }
    if package.header.package_hash != object_linker_executable_package_hash(package) {
        return Err(differential_error(
            NativeExecutionDifferentialStage::PackageIdentity,
            "package_hash",
            "object/linker package hash is stale",
        ));
    }
    if package.runtime_artifact != RuntimeArtifactIdentity::from_program(program) {
        return Err(differential_error(
            NativeExecutionDifferentialStage::PackageIdentity,
            "runtime_artifact",
            "object/linker package does not bind the exact RuntimeProgram",
        ));
    }
    if package.header.target_symbol.trim().is_empty() {
        return Err(differential_error(
            NativeExecutionDifferentialStage::PackageIdentity,
            "target_symbol",
            "object/linker package target symbol must be explicit",
        ));
    }
    if !package.smoke.passed {
        return Err(differential_error(
            NativeExecutionDifferentialStage::PackageIdentity,
            "smoke",
            "NC24 only consumes NC23 packages whose exact smoke run passed",
        ));
    }
    Ok(())
}

fn validate_runtime_ir_report<'a>(
    program: &'a RuntimeProgram,
    package: &ObjectLinkerExecutablePackage,
    run_report: &RuntimeIrRunReport,
) -> Result<&'a crate::RuntimeExample, NativeExecutionDifferentialError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if run_report.artifact != artifact || run_report.observation.artifact != artifact {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "artifact",
            "RuntimeIrRunReport does not bind the exact RuntimeProgram artifact",
        ));
    }
    if package.runtime_report_hash != object_linker_runtime_ir_run_report_hash(run_report) {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "runtime_report_hash",
            "object/linker package does not bind the exact RuntimeIrRunReport",
        ));
    }
    if run_report.observation.target != run_report.target
        || run_report.evidence.target_example != run_report.target.example
        || run_report.evidence.checked_core_shape != run_report.target.checked_core_shape
    {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport target evidence is internally inconsistent",
        ));
    }
    if run_report.evidence.package_identity != program.package_identity
        || run_report.evidence.core_semantic_hash != program.core_semantic_hash
        || run_report.evidence.runtime_artifact_hash != program.artifact_hash
    {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "evidence",
            "RuntimeIrRunReport evidence identity does not match the RuntimeProgram",
        ));
    }

    let mut matching_examples = program
        .examples
        .iter()
        .filter(|example| RuntimeIrTargetIdentity::from_example(example) == run_report.target);
    let Some(example) = matching_examples.next() else {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport target is absent from the exact RuntimeProgram",
        ));
    };
    if matching_examples.next().is_some() {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport target is ambiguous in the exact RuntimeProgram",
        ));
    }
    if !matches!(
        &example.ir,
        crate::RuntimeExpr::DeclarationRef { symbol }
            if symbol == &package.header.target_symbol
    ) {
        return Err(differential_error(
            NativeExecutionDifferentialStage::RuntimeIrRunReport,
            "target_symbol",
            "RuntimeIrRunReport does not evaluate the packaged executable target",
        ));
    }
    Ok(example)
}

fn validate_executable_artifact(
    package: &ObjectLinkerExecutablePackage,
    artifact_root: &Path,
) -> Result<std::path::PathBuf, NativeExecutionDifferentialError> {
    if package.executable_artifact.kind != ObjectLinkerArtifactKind::StarterExecutable {
        return Err(differential_error(
            NativeExecutionDifferentialStage::ArtifactFile,
            "executable_artifact.kind",
            "NC24 only runs NC23 starter executable artifacts",
        ));
    }
    let relative = &package.executable_artifact.relative_path;
    if relative.trim().is_empty() || Path::new(relative).is_absolute() || relative.contains("..") {
        return Err(differential_error(
            NativeExecutionDifferentialStage::ArtifactFile,
            "executable_artifact.relative_path",
            "executable artifact path must be a relative package path",
        ));
    }
    let path = artifact_root.join(relative);
    let bytes = fs::read(&path).map_err(|err| {
        differential_error(
            NativeExecutionDifferentialStage::ArtifactFile,
            "executable_artifact",
            format!("could not read packaged executable artifact: {err}"),
        )
    })?;
    if bytes.len() as u64 != package.executable_artifact.byte_len {
        return Err(differential_error(
            NativeExecutionDifferentialStage::ArtifactFile,
            "executable_artifact.byte_len",
            "packaged executable byte length is stale",
        ));
    }
    if fnv1a_64(&bytes) != package.executable_artifact.artifact_hash {
        return Err(differential_error(
            NativeExecutionDifferentialStage::ArtifactFile,
            "executable_artifact.artifact_hash",
            "packaged executable bytes do not match the NC23 artifact hash",
        ));
    }
    Ok(path)
}

fn effect_foreign_executable_policy_report(
    program: &RuntimeProgram,
    target_symbol: &RuntimeSymbol,
) -> Result<NativeEffectForeignExecutablePolicyReport, NativeExecutionDifferentialError> {
    let declaration = program
        .declarations
        .iter()
        .find(|declaration| declaration.symbol == *target_symbol)
        .ok_or_else(|| {
            differential_error(
                NativeExecutionDifferentialStage::EffectForeignExecutablePolicy,
                "target_symbol",
                "packaged target declaration is absent from the RuntimeProgram",
            )
        })?;

    let mut facts = BTreeSet::new();
    let checked_meta = program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .get(target_symbol);

    if let Some(reason) =
        effect_foreign_metadata_inconsistency_reason(program, declaration, checked_meta)
    {
        return Err(differential_error(
            NativeExecutionDifferentialStage::EffectForeignExecutablePolicy,
            "effect_foreign_metadata",
            reason,
        ));
    }

    if let Some(meta) = checked_meta {
        facts.insert(format!(
            "checked_core.boundary={}",
            boundary_tag(&meta.boundary)
        ));
        facts.insert(format!(
            "checked_core.lowerability={}",
            lowerability_status_tag(&meta.lowerability)
        ));
        for effect in &meta.declared_effects {
            facts.insert(format!("checked_core.effect={effect}"));
        }
        for capability in &meta.capabilities {
            facts.insert(format!("checked_core.capability={capability}"));
        }
        for runtime_check in &meta.runtime_checks {
            facts.insert(format!("checked_core.runtime_check={runtime_check}"));
        }
        if let Some(foreign_symbol) = &meta.foreign_symbol {
            facts.insert(format!("checked_core.foreign_symbol={foreign_symbol}"));
        }

        if !matches!(meta.lowerability, RuntimeLowerabilityStatus::Supported) {
            return Ok(policy_report(
                target_symbol,
                NativeEffectForeignExecutableStatus::Unsupported {
                    reason: format!(
                        "checked-core effect/foreign metadata is not native-lowerable: {:?}",
                        meta.lowerability
                    ),
                },
                facts,
                "RuntimeProgram.erased_core.metadata.checked_core.effects_foreign_metadata.lowerability",
            ));
        }

        if meta.boundary == RuntimeEffectBoundary::Foreign || meta.foreign_symbol.is_some() {
            return Ok(policy_report(
                target_symbol,
                NativeEffectForeignExecutableStatus::RepresentedUnavailable {
                    reason:
                        "foreign-boundary facts are represented, but native FFI execution is unavailable"
                            .to_string(),
                },
                facts,
                "RuntimeProgram.erased_core.metadata.checked_core.effects_foreign_metadata",
            ));
        }
        if meta.boundary == RuntimeEffectBoundary::Effectful
            || !meta.declared_effects.is_empty()
            || !meta.capabilities.is_empty()
            || !meta.runtime_checks.is_empty()
        {
            return Ok(policy_report(
                target_symbol,
                NativeEffectForeignExecutableStatus::RepresentedUnavailable {
                    reason:
                        "effect/capability/runtime-check facts are represented, but host-effect execution is unavailable"
                            .to_string(),
                },
                facts,
                "RuntimeProgram.erased_core.metadata.checked_core.effects_foreign_metadata",
            ));
        }
    }

    if !declaration.metadata.effects.is_empty()
        || !declaration.metadata.capabilities.is_empty()
        || !declaration.metadata.runtime_checks.is_empty()
    {
        for effect in &declaration.metadata.effects {
            facts.insert(format!("runtime_symbol.effect={effect}"));
        }
        for capability in &declaration.metadata.capabilities {
            facts.insert(format!("runtime_symbol.capability={capability}"));
        }
        for runtime_check in &declaration.metadata.runtime_checks {
            facts.insert(format!("runtime_symbol.runtime_check={runtime_check}"));
        }
        facts.insert("checked_core.effect_foreign_authority=missing".to_string());
        return Ok(policy_report(
            target_symbol,
            NativeEffectForeignExecutableStatus::RepresentedUnavailable {
                reason:
                    "target carries effect, capability, or runtime-check metadata without checked-core executable authority"
                        .to_string(),
            },
            facts,
            "RuntimeDeclaration.metadata effect/capability/runtime-check facts",
        ));
    }

    if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
        if !effects.is_empty() {
            for effect in effects {
                facts.insert(format!("runtime_effect_boundary.effect={effect}"));
            }
            return Ok(policy_report(
                target_symbol,
                NativeEffectForeignExecutableStatus::RepresentedUnavailable {
                    reason:
                        "target declares effect-boundary metadata without host-effect execution"
                            .to_string(),
                },
                facts,
                "RuntimeDeclarationKind::EffectBoundary",
            ));
        }
    }

    if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
        if let Some(effect) = runtime_expr_effect(body) {
            facts.insert(format!("runtime_expr.effect={effect}"));
            return Ok(policy_report(
                target_symbol,
                NativeEffectForeignExecutableStatus::Unsupported {
                    reason:
                        "transparent RuntimeExpr::Effect bodies are outside native executable policy"
                            .to_string(),
                },
                facts,
                "RuntimeDeclaration transparent body",
            ));
        }
    }

    Ok(policy_report(
        target_symbol,
        NativeEffectForeignExecutableStatus::NativeTested,
        facts,
        "NC25 found no effect/foreign facts on the packaged target",
    ))
}

fn effect_foreign_metadata_inconsistency_reason(
    program: &RuntimeProgram,
    declaration: &RuntimeDeclaration,
    checked_meta: Option<&crate::RuntimeEffectsForeignAuditMetadata>,
) -> Option<String> {
    let effect_meta = checked_meta?;
    if effect_meta.declared_effects != declaration.metadata.effects {
        return Some(format!(
            "{} has stale or missing effect/foreign authority metadata in effects",
            declaration.symbol
        ));
    }
    if effect_meta.capabilities != declaration.metadata.capabilities {
        return Some(format!(
            "{} has stale or missing effect/foreign authority metadata in capabilities",
            declaration.symbol
        ));
    }
    if effect_meta.runtime_checks != declaration.metadata.runtime_checks {
        return Some(format!(
            "{} has stale or missing effect/foreign authority metadata in runtime_checks",
            declaration.symbol
        ));
    }
    if !program
        .erased_core
        .metadata
        .effects
        .is_superset(&effect_meta.declared_effects)
    {
        return Some(format!(
            "{} has stale or missing effect/foreign authority metadata in package effects",
            declaration.symbol
        ));
    }
    if !program
        .erased_core
        .metadata
        .capabilities
        .is_superset(&effect_meta.capabilities)
    {
        return Some(format!(
            "{} has stale or missing effect/foreign authority metadata in package capabilities",
            declaration.symbol
        ));
    }
    if !program
        .erased_core
        .metadata
        .runtime_checks
        .is_superset(&effect_meta.runtime_checks)
    {
        return Some(format!(
            "{} has stale or missing effect/foreign authority metadata in package runtime checks",
            declaration.symbol
        ));
    }
    None
}

fn policy_report(
    target_symbol: &RuntimeSymbol,
    status: NativeEffectForeignExecutableStatus,
    facts: BTreeSet<String>,
    evidence_source: impl Into<String>,
) -> NativeEffectForeignExecutablePolicyReport {
    NativeEffectForeignExecutablePolicyReport {
        target_symbol: target_symbol.clone(),
        status,
        facts,
        evidence_source: evidence_source.into(),
    }
}

fn effect_foreign_policy_unavailable_reason(
    report: &NativeEffectForeignExecutablePolicyReport,
) -> Option<String> {
    match &report.status {
        NativeEffectForeignExecutableStatus::NativeTested => None,
        NativeEffectForeignExecutableStatus::RepresentedUnavailable { reason }
        | NativeEffectForeignExecutableStatus::Unsupported { reason } => Some(reason.clone()),
    }
}

fn boundary_tag(boundary: &RuntimeEffectBoundary) -> &'static str {
    match boundary {
        RuntimeEffectBoundary::Pure => "pure",
        RuntimeEffectBoundary::Effectful => "effectful",
        RuntimeEffectBoundary::Foreign => "foreign",
    }
}

fn lowerability_status_tag(status: &RuntimeLowerabilityStatus) -> String {
    match status {
        RuntimeLowerabilityStatus::Supported => "supported".to_string(),
        RuntimeLowerabilityStatus::Unsupported { reason } => {
            format!("unsupported:{reason}")
        }
        RuntimeLowerabilityStatus::Deferred {
            later_stage,
            reason,
        } => format!("deferred:{later_stage}:{reason}"),
        RuntimeLowerabilityStatus::RequiresFeature { feature, reason } => {
            format!("requires_feature:{feature}:{reason}")
        }
        RuntimeLowerabilityStatus::Explicit { state, reason } => {
            format!("explicit:{state}:{reason}")
        }
    }
}

fn runtime_expr_effect(expr: &RuntimeExpr) -> Option<&str> {
    match expr {
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => None,
        RuntimeExpr::Let { value, body } => {
            runtime_expr_effect(value).or_else(|| runtime_expr_effect(body))
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => runtime_expr_effect(scrutinee)
            .or_else(|| runtime_expr_effect(then_expr))
            .or_else(|| runtime_expr_effect(else_expr)),
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            args.iter().find_map(runtime_expr_effect)
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => runtime_expr_effect(scrutinee).or_else(|| {
            cases
                .iter()
                .find_map(|case| runtime_expr_effect(&case.body))
        }),
        RuntimeExpr::Record { fields } => fields
            .iter()
            .find_map(|(_, value)| runtime_expr_effect(value)),
        RuntimeExpr::Project { record, .. } => runtime_expr_effect(record),
        RuntimeExpr::Closure { body, .. } => runtime_expr_effect(body),
        RuntimeExpr::Call { callee, args } => {
            runtime_expr_effect(callee).or_else(|| args.iter().find_map(runtime_expr_effect))
        }
        RuntimeExpr::Effect { effect, .. } => Some(effect),
    }
}

fn run_packaged_executable(
    executable_path: &Path,
    runtime_observation: &RuntimeObservation,
) -> Result<NativeExecutionObservation, NativeExecutionDifferentialError> {
    let output = Command::new(executable_path).output().map_err(|err| {
        differential_error(
            NativeExecutionDifferentialStage::NativeExecution,
            "executable_artifact",
            format!("could not run packaged executable artifact: {err}"),
        )
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let exit_status = output.status.code().unwrap_or(-1);
    if !output.status.success() {
        return Err(differential_error(
            NativeExecutionDifferentialStage::NativeExecution,
            "exit_status",
            format!("packaged executable exited with status {exit_status}"),
        ));
    }
    let observation = decode_native_stdout(&stdout, runtime_observation)?;
    Ok(NativeExecutionObservation {
        observation,
        stdout,
        exit_status,
        evidence_source: "NC24 reran the exact NC23 packaged executable artifact".to_string(),
    })
}

fn decode_native_stdout(
    stdout: &str,
    runtime_observation: &RuntimeObservation,
) -> Result<RuntimeObservation, NativeExecutionDifferentialError> {
    let trimmed = stdout.trim_end_matches('\n');
    match runtime_observation {
        RuntimeObservation::Returned(RuntimeGroundValue::Int(_)) => {
            let value = trimmed.parse::<i64>().map_err(|err| {
                differential_error(
                    NativeExecutionDifferentialStage::NativeExecution,
                    "stdout",
                    format!("native stdout is not an Int observation: {err}"),
                )
            })?;
            Ok(RuntimeObservation::Returned(RuntimeGroundValue::Int(value)))
        }
        RuntimeObservation::Returned(RuntimeGroundValue::Bool(_)) => match trimmed {
            "0" => Ok(RuntimeObservation::Returned(RuntimeGroundValue::Bool(
                false,
            ))),
            "1" => Ok(RuntimeObservation::Returned(RuntimeGroundValue::Bool(true))),
            _ => Err(differential_error(
                NativeExecutionDifferentialStage::NativeExecution,
                "stdout",
                "native stdout is not a Bool observation encoded as 0 or 1",
            )),
        },
        RuntimeObservation::Returned(_) => Err(differential_error(
            NativeExecutionDifferentialStage::NativeExecution,
            "runtime_observation",
            "NC24 starter native execution only decodes scalar Int/Bool observations",
        )),
        RuntimeObservation::Trapped(_) => Err(differential_error(
            NativeExecutionDifferentialStage::NativeExecution,
            "runtime_observation",
            "NC24 starter native execution does not yet decode trap reports",
        )),
    }
}

fn compare_runtime_ir_lane(
    target: &NativeExecutionTargetIdentity,
    runtime_ir: &RuntimeObservation,
    native: &RuntimeObservation,
) -> NativeComparisonLaneReport {
    if runtime_ir == native {
        NativeComparisonLaneReport::TestedAgreement {
            lane: NativeDifferentialLane::RuntimeIrEvaluator,
            expected: runtime_ir.clone(),
            observed: native.clone(),
            evidence_source:
                "NC24 compared native execution against RuntimeIrRunReport observation".to_string(),
        }
    } else {
        NativeComparisonLaneReport::Mismatch {
            lane: NativeDifferentialLane::RuntimeIrEvaluator,
            expected: runtime_ir.clone(),
            observed: native.clone(),
            diagnostic: mismatch_diagnostic(
                target,
                NativeDifferentialLane::RuntimeIrEvaluator,
                runtime_ir.clone(),
                native.clone(),
            ),
        }
    }
}

fn compare_interpreter_lane(
    target: &NativeExecutionTargetIdentity,
    expected_target: &RuntimeIrTargetIdentity,
    interpreter: NativeInterpreterLaneInput,
    native: &RuntimeObservation,
) -> Result<NativeComparisonLaneReport, NativeExecutionDifferentialError> {
    match interpreter {
        NativeInterpreterLaneInput::Unavailable {
            reason,
            evidence_source,
        } => Ok(NativeComparisonLaneReport::Unavailable {
            lane: NativeDifferentialLane::Interpreter,
            reason,
            evidence_source,
        }),
        NativeInterpreterLaneInput::Available(interpreter) => {
            if interpreter.artifact != target.runtime_artifact {
                return Err(detached_interpreter_evidence(
                    "artifact",
                    "asserted-available interpreter observation artifact identity does not match RuntimeProgram",
                ));
            }
            if &interpreter.target != expected_target {
                return Err(detached_interpreter_evidence(
                    "target",
                    "asserted-available interpreter observation target identity does not match RuntimeIrRunReport",
                ));
            }
            if interpreter.observation == *native {
                Ok(NativeComparisonLaneReport::TestedAgreement {
                    lane: NativeDifferentialLane::Interpreter,
                    expected: interpreter.observation.clone(),
                    observed: native.clone(),
                    evidence_source: interpreter.evidence_source,
                })
            } else {
                Ok(NativeComparisonLaneReport::Mismatch {
                    lane: NativeDifferentialLane::Interpreter,
                    expected: interpreter.observation.clone(),
                    observed: native.clone(),
                    diagnostic: mismatch_diagnostic(
                        target,
                        NativeDifferentialLane::Interpreter,
                        interpreter.observation,
                        native.clone(),
                    ),
                })
            }
        }
    }
}

fn unavailable_interpreter_for_trap(
    target: &NativeExecutionTargetIdentity,
    expected_target: &RuntimeIrTargetIdentity,
    interpreter: NativeInterpreterLaneInput,
    runtime_ir: &RuntimeObservation,
) -> Result<NativeComparisonLaneReport, NativeExecutionDifferentialError> {
    match interpreter {
        NativeInterpreterLaneInput::Unavailable {
            reason,
            evidence_source,
        } => Ok(NativeComparisonLaneReport::Unavailable {
            lane: NativeDifferentialLane::Interpreter,
            reason,
            evidence_source,
        }),
        NativeInterpreterLaneInput::Available(interpreter) => {
            if interpreter.artifact != target.runtime_artifact {
                return Err(detached_interpreter_evidence(
                    "artifact",
                    "asserted-available interpreter observation artifact identity does not match RuntimeProgram",
                ));
            }
            if &interpreter.target != expected_target {
                return Err(detached_interpreter_evidence(
                    "target",
                    "asserted-available interpreter observation target identity does not match RuntimeIrRunReport",
                ));
            }
            Ok(NativeComparisonLaneReport::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                reason: "interpreter trap observation is present, but native trap decoding is unavailable in NC24"
                    .to_string(),
                evidence_source: format!(
                    "{}; interpreter observation {:?}; runtime-IR observation {:?}",
                    interpreter.evidence_source, interpreter.observation, runtime_ir
                ),
            })
        }
    }
}

fn unavailable_interpreter_for_policy(
    target: &NativeExecutionTargetIdentity,
    expected_target: &RuntimeIrTargetIdentity,
    interpreter: NativeInterpreterLaneInput,
    runtime_ir: &RuntimeObservation,
    policy_reason: &str,
) -> Result<NativeComparisonLaneReport, NativeExecutionDifferentialError> {
    match interpreter {
        NativeInterpreterLaneInput::Unavailable {
            reason,
            evidence_source,
        } => Ok(NativeComparisonLaneReport::Unavailable {
            lane: NativeDifferentialLane::Interpreter,
            reason,
            evidence_source,
        }),
        NativeInterpreterLaneInput::Available(interpreter) => {
            if interpreter.artifact != target.runtime_artifact {
                return Err(detached_interpreter_evidence(
                    "artifact",
                    "asserted-available interpreter observation artifact identity does not match RuntimeProgram",
                ));
            }
            if &interpreter.target != expected_target {
                return Err(detached_interpreter_evidence(
                    "target",
                    "asserted-available interpreter observation target identity does not match RuntimeIrRunReport",
                ));
            }
            Ok(NativeComparisonLaneReport::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                reason: policy_reason.to_string(),
                evidence_source: format!(
                    "{}; interpreter observation {:?}; runtime-IR observation {:?}",
                    interpreter.evidence_source, interpreter.observation, runtime_ir
                ),
            })
        }
    }
}

fn differential_verdict(
    runtime_ir: &NativeComparisonLaneReport,
    interpreter: &NativeComparisonLaneReport,
) -> NativeExecutionDifferentialVerdict {
    if let NativeComparisonLaneReport::Mismatch {
        lane, diagnostic, ..
    } = runtime_ir
    {
        return NativeExecutionDifferentialVerdict::Mismatch {
            lane: lane.clone(),
            diagnostic: diagnostic.clone(),
        };
    }
    if let NativeComparisonLaneReport::Mismatch {
        lane, diagnostic, ..
    } = interpreter
    {
        return NativeExecutionDifferentialVerdict::Mismatch {
            lane: lane.clone(),
            diagnostic: diagnostic.clone(),
        };
    }

    let runtime_ir = lane_verdict(runtime_ir);
    let interpreter = lane_verdict(interpreter);
    if let NativeLaneVerdict::Unavailable { reason } = &runtime_ir {
        return NativeExecutionDifferentialVerdict::Unavailable {
            lane: NativeDifferentialLane::RuntimeIrEvaluator,
            reason: reason.clone(),
        };
    }
    NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement {
        runtime_ir,
        interpreter,
    }
}

fn lane_verdict(report: &NativeComparisonLaneReport) -> NativeLaneVerdict {
    match report {
        NativeComparisonLaneReport::TestedAgreement { .. } => NativeLaneVerdict::TestedAgreement,
        NativeComparisonLaneReport::Mismatch { .. } => {
            unreachable!("mismatch verdicts are handled before lane verdict projection")
        }
        NativeComparisonLaneReport::Unavailable { reason, .. } => NativeLaneVerdict::Unavailable {
            reason: reason.clone(),
        },
    }
}

fn mismatch_diagnostic(
    target: &NativeExecutionTargetIdentity,
    lane: NativeDifferentialLane,
    expected: RuntimeObservation,
    observed: RuntimeObservation,
) -> NativeMismatchDiagnostic {
    NativeMismatchDiagnostic {
        package_identity: target.package_identity.clone(),
        target_symbol: target.target_symbol.clone(),
        executable_artifact_hash: target.executable_artifact_hash,
        lane,
        expected: expected.clone(),
        observed: observed.clone(),
        message: format!(
            "native differential mismatch for package {}, target {}, executable {:016x}: expected {:?}, observed {:?}",
            target.package_identity, target.target_symbol, target.executable_artifact_hash, expected, observed
        ),
    }
}

fn differential_error(
    stage: NativeExecutionDifferentialStage,
    field: &'static str,
    reason: impl Into<String>,
) -> NativeExecutionDifferentialError {
    NativeExecutionDifferentialError {
        stage,
        field,
        reason: reason.into(),
    }
}

fn detached_interpreter_evidence(
    field: &'static str,
    reason: impl Into<String>,
) -> NativeExecutionDifferentialError {
    differential_error(
        NativeExecutionDifferentialStage::InterpreterEvidence,
        field,
        reason,
    )
}

fn required_unavailable_claims() -> BTreeSet<NativeExecutionUnavailableClaim> {
    BTreeSet::from([
        NativeExecutionUnavailableClaim::LibraryAbi,
        NativeExecutionUnavailableClaim::CAbiInterop,
        NativeExecutionUnavailableClaim::RustInterop,
        NativeExecutionUnavailableClaim::ForeignExecution,
        NativeExecutionUnavailableClaim::EffectPolicyBroadening,
        NativeExecutionUnavailableClaim::TranslationValidation,
        NativeExecutionUnavailableClaim::WholeCompilerProof,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use crate::{
        evaluate_runtime_ir_example, executable_artifact_contract_for_runtime_report,
        executable_entrypoint_metadata_hash, executable_entrypoint_package_for_runtime_contract,
        package_starter_executable_artifact, platform_runtime_support_for_entrypoint,
        summarize_runtime_ir_program, ErasedExecutableCore, ExecutableArgumentPackaging,
        ExecutableArgumentShape, ExecutableDependencyClosure, ExecutableEntrypointPackageMetadata,
        ExecutableEntrypointTargetKind, ExecutableEntrypointVerdict, ExecutableReportContract,
        ExecutableResultObservation, ExecutableResultShape, ExecutableRuntimeSupport,
        ExecutableTrapContract, ExecutableTrapShape, NativeSeedEnvironment, PlatformRuntimeTarget,
        RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr, RuntimeIrSeedEnvironment,
        RuntimeLowerabilityStatus, RuntimeMetadata, RuntimePartiality, RuntimePrimitive,
        RuntimeSymbolMetadata, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
    };

    fn starter_program(value: i64) -> RuntimeProgram {
        let symbol = "decl:fixture::NativeDifferential::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::native-differential".to_string(),
            core_semantic_hash: 0x2401,
            artifact_hash: 0x2402,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol: symbol.clone(),
                kind: RuntimeDeclarationKind::Transparent {
                    body: RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "add_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![
                            RuntimeExpr::Value(RuntimeValue::Int(value - 1)),
                            RuntimeExpr::Value(RuntimeValue::Int(1)),
                        ],
                    },
                },
                metadata: RuntimeSymbolMetadata {
                    lowerability: Some(RuntimeLowerabilityStatus::Supported),
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: vec![crate::RuntimeExample {
                name: "native-differential-main".to_string(),
                checked_core_shape: "fixture main".to_string(),
                ir: RuntimeExpr::DeclarationRef { symbol },
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(value)),
            }],
        }
    }

    fn add_checked_effect_foreign_metadata(
        program: &mut RuntimeProgram,
        boundary: RuntimeEffectBoundary,
        effects: BTreeSet<String>,
        capabilities: BTreeSet<String>,
        runtime_checks: BTreeSet<String>,
        foreign_symbol: Option<String>,
    ) {
        let symbol = program.declarations[0].symbol.clone();
        program.declarations[0].metadata.effects = effects.clone();
        program.declarations[0].metadata.capabilities = capabilities.clone();
        program.declarations[0].metadata.runtime_checks = runtime_checks.clone();
        program
            .erased_core
            .metadata
            .effects
            .extend(effects.iter().cloned());
        program
            .erased_core
            .metadata
            .capabilities
            .extend(capabilities.iter().cloned());
        program
            .erased_core
            .metadata
            .runtime_checks
            .extend(runtime_checks.iter().cloned());
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                crate::RuntimeEffectsForeignAuditMetadata {
                    declared_effects: effects,
                    capabilities,
                    foreign_symbol,
                    boundary,
                    runtime_checks,
                    lowerability: RuntimeLowerabilityStatus::Supported,
                },
            );
    }

    fn add_runtime_effect_metadata_without_checked_authority(program: &mut RuntimeProgram) {
        program.declarations[0]
            .metadata
            .effects
            .insert("host.io".to_string());
        program
            .erased_core
            .metadata
            .effects
            .insert("host.io".to_string());
    }

    fn replace_target_body_with_effect(program: &mut RuntimeProgram) {
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::Effect {
                effect: "host.io".to_string(),
                capability: None,
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(1))],
            },
        };
    }

    fn packaged_entrypoint(program: &RuntimeProgram) -> crate::RuntimeExecutableEntrypointPackage {
        let report = summarize_runtime_ir_program(program);
        let target = program.declarations[0].symbol.clone();
        let contract = executable_artifact_contract_for_runtime_report(
            program,
            &report,
            target.clone(),
            "native differential unit test",
        )
        .expect("contract materializes");
        let mut entrypoint = ExecutableEntrypointPackageMetadata {
            package_identity: program.package_identity.clone(),
            package_core_semantic_hash: program.core_semantic_hash,
            package_artifact_hash: program.artifact_hash,
            target_symbol: target,
            target_kind: ExecutableEntrypointTargetKind::Executable,
            closure_identity: 0x2420,
            closure_semantic_hash: 0x2421,
            metadata_identity: 0,
            closed_entry: ExecutableEntrypointVerdict::ClosedKenOnly,
            dependency_closure: ExecutableDependencyClosure::ClosedKenOnly,
            required_runtime_support: BTreeSet::from([
                ExecutableRuntimeSupport::RuntimeValues,
                ExecutableRuntimeSupport::PrimitiveValues,
                ExecutableRuntimeSupport::PrimitiveOperations,
                ExecutableRuntimeSupport::TrapReporting,
            ]),
            argument_packaging: ExecutableArgumentPackaging {
                shape: ExecutableArgumentShape::ClosedNullary,
                evidence_source: "checked-core target body".to_string(),
            },
            result_observation: ExecutableResultObservation {
                shape: ExecutableResultShape::RuntimeValue,
                evidence_source: "runtime value result".to_string(),
            },
            trap_contract: ExecutableTrapContract {
                shape: ExecutableTrapShape::RuntimeTrapReport,
                blocking_lanes: Default::default(),
            },
            report_contract: ExecutableReportContract {
                target_closure_identity: 0x2420,
                target_closure_report_hash: 0x2422,
                evidence_source: "target closure report".to_string(),
            },
            unsupported_lanes: Default::default(),
        };
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);
        executable_entrypoint_package_for_runtime_contract(
            program,
            &report,
            &contract,
            entrypoint,
            "native differential unit test",
        )
        .expect("entrypoint package materializes")
    }

    fn runtime_ir_run_report(program: &RuntimeProgram) -> RuntimeIrRunReport {
        evaluate_runtime_ir_example(
            program,
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator produces an observation")
    }

    fn package_for(
        program: &RuntimeProgram,
        run_report: &RuntimeIrRunReport,
        output_dir: &Path,
    ) -> ObjectLinkerExecutablePackage {
        let entrypoint = packaged_entrypoint(program);
        let support = platform_runtime_support_for_entrypoint(
            program,
            &entrypoint,
            run_report,
            PlatformRuntimeTarget::starter(native_platform_target_name()),
            "native differential unit test",
        )
        .expect("platform support materializes");
        package_starter_executable_artifact(
            program,
            &entrypoint,
            &support,
            run_report,
            &NativeSeedEnvironment::empty(),
            output_dir,
            "native differential unit test",
        )
        .expect("object/linker package materializes")
    }

    fn interpreter_available(
        program: &RuntimeProgram,
        run_report: &RuntimeIrRunReport,
    ) -> NativeInterpreterLaneInput {
        NativeInterpreterLaneInput::Available(RuntimeInterpreterObservation {
            artifact: RuntimeArtifactIdentity::from_program(program),
            target: run_report.target.clone(),
            observation: run_report.observation.observation.clone(),
            evidence_source: "caller-supplied NC24 interpreter observation".to_string(),
        })
    }

    fn temp_output_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "ken-runtime-{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock is after epoch")
                .as_nanos()
        ));
        dir
    }

    fn native_platform_target_name() -> String {
        format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
    }

    #[test]
    fn reports_tested_native_runtime_and_interpreter_agreement() {
        let program = starter_program(24);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-agreement");
        let package = package_for(&program, &run_report, &output_dir);

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("native differential report materializes");

        assert_eq!(
            report.header.report_kind,
            NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND
        );
        assert_eq!(report.target.package_identity, program.package_identity);
        assert_eq!(
            report.target.executable_artifact_hash,
            package.executable_artifact.artifact_hash
        );
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Available(NativeExecutionObservation {
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(24)),
                ..
            })
        ));
        assert!(matches!(
            report.runtime_ir,
            NativeComparisonLaneReport::TestedAgreement {
                lane: NativeDifferentialLane::RuntimeIrEvaluator,
                ..
            }
        ));
        assert!(matches!(
            report.interpreter,
            NativeComparisonLaneReport::TestedAgreement {
                lane: NativeDifferentialLane::Interpreter,
                ..
            }
        ));
        assert!(report
            .unavailable_claims
            .contains(&NativeExecutionUnavailableClaim::WholeCompilerProof));
        assert!(report
            .unavailable_claims
            .contains(&NativeExecutionUnavailableClaim::LibraryAbi));
        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::NativeTested
        ));
        assert_eq!(
            report.verdict,
            NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement {
                runtime_ir: NativeLaneVerdict::TestedAgreement,
                interpreter: NativeLaneVerdict::TestedAgreement,
            }
        );
    }

    #[test]
    fn unavailable_interpreter_lane_stays_first_class_not_passed() {
        let program = starter_program(25);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-interpreter-unavailable");
        let package = package_for(&program, &run_report, &output_dir);

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            NativeInterpreterLaneInput::Unavailable {
                reason: "NC18 report marks interpreter lane unavailable".to_string(),
                evidence_source: "NC18 comparison_unavailable_targets".to_string(),
            },
            "native differential unit test",
        )
        .expect("native differential report materializes");

        assert!(matches!(
            report.runtime_ir,
            NativeComparisonLaneReport::TestedAgreement { .. }
        ));
        assert!(matches!(
            report.interpreter,
            NativeComparisonLaneReport::Unavailable {
                lane: NativeDifferentialLane::Interpreter,
                ref reason,
                ..
            } if reason.contains("NC18 report")
        ));
        assert!(matches!(
            report.verdict,
            NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement {
                runtime_ir: NativeLaneVerdict::TestedAgreement,
                interpreter: NativeLaneVerdict::Unavailable { .. },
            }
        ));
    }

    #[test]
    fn stale_object_linker_package_hash_rejects_before_execution() {
        let program = starter_program(26);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-stale-package");
        let mut package = package_for(&program, &run_report, &output_dir);
        package.header.package_hash ^= 1;

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("stale package hash rejects");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::PackageIdentity);
        assert_eq!(err.field, "package_hash");
    }

    #[test]
    fn trap_observation_is_first_class_unavailable_native_lane() {
        let program = starter_program(26);
        let mut run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-trap-unavailable");
        let mut package = package_for(&program, &run_report, &output_dir);
        run_report.observation.observation = RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "fixture trap".to_string(),
        });
        package.runtime_report_hash = object_linker_runtime_ir_run_report_hash(&run_report);
        package.header.package_hash = object_linker_executable_package_hash(&package);

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("trap lane report materializes");

        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Unavailable { ref reason, .. }
                if reason.contains("trap")
        ));
        assert!(matches!(
            report.runtime_ir,
            NativeComparisonLaneReport::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                ref reason,
                ..
            } if reason.contains("trap")
        ));
        assert!(matches!(
            report.verdict,
            NativeExecutionDifferentialVerdict::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                ref reason,
            } if reason.contains("trap")
        ));
    }

    #[test]
    fn trap_observation_still_rejects_stale_executable_artifact() {
        let program = starter_program(38);
        let mut run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-trap-stale-executable");
        let mut package = package_for(&program, &run_report, &output_dir);
        run_report.observation.observation = RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "fixture trap".to_string(),
        });
        package.runtime_report_hash = object_linker_runtime_ir_run_report_hash(&run_report);
        package.header.package_hash = object_linker_executable_package_hash(&package);
        fs::write(
            output_dir.join(&package.executable_artifact.relative_path),
            b"not the packaged executable",
        )
        .expect("mutate packaged executable bytes");

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("trap path rejects stale executable bytes before report");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::ArtifactFile);
        assert_eq!(err.field, "executable_artifact.byte_len");
    }

    #[test]
    fn foreign_boundary_target_reports_policy_unavailable_before_native_execution() {
        let base_program = starter_program(39);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-foreign-unavailable");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        add_checked_effect_foreign_metadata(
            &mut program,
            RuntimeEffectBoundary::Foreign,
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            Some("host_print".to_string()),
        );

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("foreign boundary is represented as unavailable");

        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::RepresentedUnavailable { ref reason }
                if reason.contains("foreign-boundary")
        ));
        assert!(report
            .effect_foreign_policy
            .facts
            .contains("checked_core.foreign_symbol=host_print"));
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Unavailable { ref reason, .. }
                if reason.contains("foreign")
        ));
        assert!(matches!(
            report.verdict,
            NativeExecutionDifferentialVerdict::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                ..
            }
        ));
    }

    #[test]
    fn missing_effect_authority_reports_represented_unavailable() {
        let base_program = starter_program(40);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-missing-effect-authority");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        add_runtime_effect_metadata_without_checked_authority(&mut program);

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("missing authority is represented as unavailable");

        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::RepresentedUnavailable { ref reason }
                if reason.contains("without checked-core executable authority")
        ));
        assert!(report
            .effect_foreign_policy
            .facts
            .contains("checked_core.effect_foreign_authority=missing"));
    }

    #[test]
    fn unsupported_capability_facts_remain_unavailable_not_native_tested() {
        let base_program = starter_program(41);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-capability-unavailable");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        add_checked_effect_foreign_metadata(
            &mut program,
            RuntimeEffectBoundary::Effectful,
            BTreeSet::from(["host.io".to_string()]),
            BTreeSet::from(["cap:fixture::HostIo".to_string()]),
            BTreeSet::new(),
            None,
        );

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("capability facts are represented as unavailable");

        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::RepresentedUnavailable { ref reason }
                if reason.contains("effect/capability/runtime-check")
        ));
        assert!(report
            .effect_foreign_policy
            .facts
            .contains("checked_core.capability=cap:fixture::HostIo"));
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Unavailable { .. }
        ));
    }

    #[test]
    fn non_supported_effect_foreign_lowerability_is_unsupported_not_native_tested() {
        let base_program = starter_program(45);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-lowerability-unsupported");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                crate::RuntimeEffectsForeignAuditMetadata {
                    declared_effects: BTreeSet::new(),
                    capabilities: BTreeSet::new(),
                    foreign_symbol: None,
                    boundary: RuntimeEffectBoundary::Pure,
                    runtime_checks: BTreeSet::new(),
                    lowerability: RuntimeLowerabilityStatus::Unsupported {
                        reason: "requires host-effect policy".to_string(),
                    },
                },
            );

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("non-supported effect/foreign lowerability reports unsupported");

        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::Unsupported { ref reason }
                if reason.contains("not native-lowerable")
        ));
        assert!(report
            .effect_foreign_policy
            .facts
            .contains("checked_core.lowerability=unsupported:requires host-effect policy"));
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Unavailable { .. }
        ));
        assert!(matches!(
            report.verdict,
            NativeExecutionDifferentialVerdict::Unavailable {
                lane: NativeDifferentialLane::NativeExecution,
                ..
            }
        ));
    }

    #[test]
    fn stale_effect_metadata_rejects_before_native_execution() {
        let base_program = starter_program(42);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-stale-effect-metadata");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .effects
            .insert("host.io".to_string());
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                crate::RuntimeEffectsForeignAuditMetadata {
                    declared_effects: BTreeSet::from(["host.io".to_string()]),
                    capabilities: BTreeSet::new(),
                    foreign_symbol: None,
                    boundary: RuntimeEffectBoundary::Effectful,
                    runtime_checks: BTreeSet::new(),
                    lowerability: RuntimeLowerabilityStatus::Supported,
                },
            );

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("stale effect metadata rejects");

        assert_eq!(
            err.stage,
            NativeExecutionDifferentialStage::EffectForeignExecutablePolicy
        );
        assert_eq!(err.field, "effect_foreign_metadata");
        assert!(err.reason.contains("stale or missing"));
    }

    #[test]
    fn hidden_runtime_effect_body_reports_unsupported_without_native_execution() {
        let base_program = starter_program(43);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-hidden-effect-body");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        replace_target_body_with_effect(&mut program);

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("hidden RuntimeExpr::Effect is an unsupported policy report");

        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::Unsupported { ref reason }
                if reason.contains("RuntimeExpr::Effect")
        ));
        assert!(report
            .effect_foreign_policy
            .facts
            .contains("runtime_expr.effect=host.io"));
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Unavailable { .. }
        ));
    }

    #[test]
    fn effect_policy_unavailable_still_rejects_stale_executable_artifact() {
        let base_program = starter_program(44);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc25-effect-stale-executable");
        let package = package_for(&base_program, &run_report, &output_dir);
        let mut program = base_program.clone();
        add_checked_effect_foreign_metadata(
            &mut program,
            RuntimeEffectBoundary::Foreign,
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
            Some("host_print".to_string()),
        );
        fs::write(
            output_dir.join(&package.executable_artifact.relative_path),
            b"not the packaged executable",
        )
        .expect("mutate packaged executable bytes");

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("policy unavailable path validates executable identity first");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::ArtifactFile);
        assert_eq!(err.field, "executable_artifact.byte_len");
    }

    #[test]
    fn asserted_available_interpreter_artifact_mismatch_rejects() {
        let program = starter_program(36);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-interpreter-artifact-mismatch");
        let package = package_for(&program, &run_report, &output_dir);
        let mut interpreter = match interpreter_available(&program, &run_report) {
            NativeInterpreterLaneInput::Available(interpreter) => interpreter,
            NativeInterpreterLaneInput::Unavailable { .. } => unreachable!(),
        };
        interpreter.artifact.artifact_hash ^= 1;

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            NativeInterpreterLaneInput::Available(interpreter),
            "native differential unit test",
        )
        .expect_err("detached interpreter artifact rejects");

        assert_eq!(
            err.stage,
            NativeExecutionDifferentialStage::InterpreterEvidence
        );
        assert_eq!(err.field, "artifact");
    }

    #[test]
    fn asserted_available_interpreter_target_mismatch_rejects() {
        let program = starter_program(37);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-interpreter-target-mismatch");
        let package = package_for(&program, &run_report, &output_dir);
        let mut interpreter = match interpreter_available(&program, &run_report) {
            NativeInterpreterLaneInput::Available(interpreter) => interpreter,
            NativeInterpreterLaneInput::Unavailable { .. } => unreachable!(),
        };
        interpreter.target.example = "detached-example".to_string();

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            NativeInterpreterLaneInput::Available(interpreter),
            "native differential unit test",
        )
        .expect_err("detached interpreter target rejects");

        assert_eq!(
            err.stage,
            NativeExecutionDifferentialStage::InterpreterEvidence
        );
        assert_eq!(err.field, "target");
    }

    #[test]
    fn detached_runtime_ir_report_rejects_before_execution() {
        let program = starter_program(27);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-detached-report");
        let package = package_for(&program, &run_report, &output_dir);
        let mut other_program = starter_program(28);
        other_program.artifact_hash ^= 1;
        let other_report = runtime_ir_run_report(&other_program);

        let err = run_native_execution_differential(
            &program,
            &package,
            &other_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("detached run report rejects");

        assert_eq!(
            err.stage,
            NativeExecutionDifferentialStage::RuntimeIrRunReport
        );
        assert_eq!(err.field, "artifact");
    }

    #[test]
    fn runtime_ir_mismatch_names_package_target_artifact_and_lane() {
        let program = starter_program(29);
        let mut run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-runtime-mismatch");
        let mut package = package_for(&program, &run_report, &output_dir);
        run_report.observation.observation =
            RuntimeObservation::Returned(RuntimeGroundValue::Int(30));
        package.runtime_report_hash = object_linker_runtime_ir_run_report_hash(&run_report);
        package.header.package_hash = object_linker_executable_package_hash(&package);

        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            NativeInterpreterLaneInput::Unavailable {
                reason: "interpreter lane intentionally unavailable in mismatch fixture"
                    .to_string(),
                evidence_source: "unit test".to_string(),
            },
            "native differential unit test",
        )
        .expect("mismatch report materializes");

        let NativeExecutionDifferentialVerdict::Mismatch { lane, diagnostic } = report.verdict
        else {
            panic!("expected runtime-IR mismatch verdict");
        };
        assert_eq!(lane, NativeDifferentialLane::RuntimeIrEvaluator);
        assert_eq!(diagnostic.package_identity, program.package_identity);
        assert_eq!(diagnostic.target_symbol, package.header.target_symbol);
        assert_eq!(
            diagnostic.executable_artifact_hash,
            package.executable_artifact.artifact_hash
        );
        assert_eq!(diagnostic.lane, NativeDifferentialLane::RuntimeIrEvaluator);
        assert!(diagnostic.message.contains(&program.package_identity));
        assert!(diagnostic.message.contains(&package.header.target_symbol));
    }

    #[test]
    fn stale_executable_bytes_reject_before_native_execution() {
        let program = starter_program(31);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-stale-executable");
        let package = package_for(&program, &run_report, &output_dir);
        fs::write(
            output_dir.join(&package.executable_artifact.relative_path),
            b"not the packaged executable",
        )
        .expect("mutate packaged executable bytes");

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("stale executable bytes reject");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::ArtifactFile);
        assert_eq!(err.field, "executable_artifact.byte_len");
    }

    #[test]
    fn suite_runner_preserves_per_case_lane_reports() {
        let first_program = starter_program(34);
        let first_run = runtime_ir_run_report(&first_program);
        let first_dir = temp_output_dir("nc24-suite-first");
        let first_package = package_for(&first_program, &first_run, &first_dir);
        let second_program = starter_program(35);
        let second_run = runtime_ir_run_report(&second_program);
        let second_dir = temp_output_dir("nc24-suite-second");
        let second_package = package_for(&second_program, &second_run, &second_dir);

        let reports = run_native_execution_differential_suite([
            NativeExecutionDifferentialCase {
                program: &first_program,
                package: &first_package,
                run_report: &first_run,
                artifact_root: &first_dir,
                interpreter: interpreter_available(&first_program, &first_run),
                producer: "native differential unit test".to_string(),
            },
            NativeExecutionDifferentialCase {
                program: &second_program,
                package: &second_package,
                run_report: &second_run,
                artifact_root: &second_dir,
                interpreter: NativeInterpreterLaneInput::Unavailable {
                    reason: "interpreter unavailable for second fixture".to_string(),
                    evidence_source: "unit test".to_string(),
                },
                producer: "native differential unit test".to_string(),
            },
        ])
        .expect("suite reports materialize");

        assert_eq!(reports.len(), 2);
        assert!(matches!(
            reports[0].interpreter,
            NativeComparisonLaneReport::TestedAgreement { .. }
        ));
        assert!(matches!(
            reports[1].interpreter,
            NativeComparisonLaneReport::Unavailable { .. }
        ));
    }

    #[test]
    fn forged_object_linker_hash_rejects_unsupported_package_kind() {
        let program = starter_program(32);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc24-forged-kind");
        let mut package = package_for(&program, &run_report, &output_dir);
        package.header.package_kind = "ForgedObjectLinkerPackage".to_string();
        package.header.package_hash = object_linker_executable_package_hash(&package);

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("forged package kind rejects");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::PackageIdentity);
        assert_eq!(err.field, "package_kind");
    }
}
