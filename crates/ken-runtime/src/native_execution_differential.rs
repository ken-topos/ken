//! Native execution differential reports for packaged starter executables.
//!
//! NC24 consumes compiler-produced NC23 object/linker packages and compares
//! exact native execution observations against runtime-IR evaluator reports and
//! interpreter observations when that lane is available. NC25 carries NC18
//! effect/foreign facts through that report surface so host-effect and FFI
//! execution stay explicitly unavailable unless a later policy makes them
//! executable. The report is tested evidence only: it does not claim translation
//! validation, proof, library ABI, C/Rust interop, or foreign execution support.
//! NC27 consumes those exact reports as a phase closeout manifest, keeping the
//! starter corpus and excluded claims explicit before NC28 broad validation.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    fnv1a_64, object_linker_executable_package_hash, object_linker_runtime_ir_run_report_hash,
    ObjectLinkerArtifactKind, ObjectLinkerEvidenceFact, ObjectLinkerEvidenceLane,
    ObjectLinkerExecutablePackage, RuntimeArtifactIdentity, RuntimeDeclaration,
    RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExpr, RuntimeGroundValue,
    RuntimeInterpreterObservation, RuntimeIrRunReport, RuntimeIrTargetIdentity,
    RuntimeLowerabilityStatus, RuntimeObservation, RuntimeProgram, RuntimeSymbol,
    OBJECT_LINKER_PACKAGE_KIND, OBJECT_LINKER_PACKAGE_VERSION,
};

pub const NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND: &str = "KenNativeExecutionDifferentialReport";
pub const NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION: u32 = 2;
pub const NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF: &str =
    "docs/program/wp/NC26-native-trust-report-provenance.md";
pub const NATIVE_EXECUTABLE_PHASE_CLOSEOUT_REPORT_KIND: &str =
    "KenNativeExecutablePhaseCloseoutReport";
pub const NATIVE_EXECUTABLE_PHASE_CLOSEOUT_REPORT_VERSION: u32 = 1;
pub const NATIVE_EXECUTABLE_PHASE_CLOSEOUT_SPEC_REF: &str =
    "docs/program/wp/NC27-executable-phase-closeout.md";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionDifferentialReport {
    pub header: NativeExecutionDifferentialHeader,
    pub target: NativeExecutionTargetIdentity,
    pub native: NativeExecutionLaneReport,
    pub runtime_ir: NativeComparisonLaneReport,
    pub interpreter: NativeComparisonLaneReport,
    pub verdict: NativeExecutionDifferentialVerdict,
    pub effect_foreign_policy: NativeEffectForeignExecutablePolicyReport,
    pub trust: NativeExecutableTrustReport,
    pub unavailable_claims: BTreeSet<NativeExecutionUnavailableClaim>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutablePhaseCloseoutReport {
    pub header: NativeExecutablePhaseCloseoutHeader,
    pub corpus: NativeExecutableCorpusManifest,
    pub claim_inventory: Vec<NativeExecutablePhaseClaim>,
    pub exclusions: Vec<NativeExecutableCorpusExclusion>,
    pub recommendation: NativeExecutablePhaseRecommendation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutablePhaseCloseoutHeader {
    pub report_kind: String,
    pub version: u32,
    pub spec_ref: String,
    pub producer: String,
    pub source_report_kind: String,
    pub source_report_version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutableCorpusManifest {
    pub positive_cases: Vec<NativeExecutableCorpusCase>,
    pub blockers: Vec<NativeExecutableCorpusBlocker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutableCorpusCase {
    pub target: NativeExecutionTargetIdentity,
    pub checked_core: NativeCheckedCoreProvenance,
    pub runtime_ir: NativeRuntimeIrProvenance,
    pub object_linker: NativeObjectLinkerProvenance,
    pub native_differential_report_hash: u64,
    pub status: NativeExecutablePhaseStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutableCorpusBlocker {
    pub target: NativeExecutionTargetIdentity,
    pub native_differential_report_hash: u64,
    pub status: NativeExecutablePhaseStatus,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutablePhaseClaim {
    pub target_symbol: RuntimeSymbol,
    pub claim: NativeExecutableEvidenceClaim,
    pub status: NativeExecutablePhaseStatus,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutableCorpusExclusion {
    pub exclusion: NativeExecutableCorpusExclusionKind,
    pub status: NativeExecutablePhaseStatus,
    pub reason: String,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutableCorpusExclusionKind {
    NonStarterExecutableArtifact,
    NonScalarResultDecoding,
    TrapDecoding,
    HostEffectExecution,
    ForeignExecution,
    LibraryAbi,
    CAbiInterop,
    RustInterop,
    CrossPackageNativeLinking,
    TranslationValidation,
    WholeCompilerProof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutablePhaseStatus {
    Supported,
    Tested,
    Validated,
    Proved,
    Unavailable { reason: String },
    Unsupported { reason: String },
    Failed { reason: String },
    Deferred { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutablePhaseRecommendation {
    ProceedToNc28 {
        reason: String,
    },
    FramePrerequisiteWp {
        reason: String,
        blockers: Vec<NativeExecutableCorpusBlocker>,
    },
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
pub struct NativeExecutableTrustReport {
    pub provenance: NativeExecutableProvenance,
    pub evidence_lanes: Vec<NativeExecutableEvidenceLane>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutableProvenance {
    pub checked_core: NativeCheckedCoreProvenance,
    pub runtime_ir: NativeRuntimeIrProvenance,
    pub object_linker: NativeObjectLinkerProvenance,
    pub toolchain: NativeToolchainProvenance,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeCheckedCoreProvenance {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRuntimeIrProvenance {
    pub runtime_artifact: RuntimeArtifactIdentity,
    pub target: RuntimeIrTargetIdentity,
    pub runtime_report_hash: u64,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeObjectLinkerProvenance {
    pub object_linker_package_hash: u64,
    pub entrypoint_package_hash: u64,
    pub platform_runtime_support_hash: u64,
    pub object_artifact_kind: ObjectLinkerArtifactKind,
    pub object_artifact_hash: u64,
    pub object_byte_len: u64,
    pub executable_artifact_hash: u64,
    pub executable_byte_len: u64,
    pub executable_relative_path: String,
    pub smoke_evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeToolchainProvenance {
    pub ken_runtime: NativeRecordedFact,
    pub native_backend: NativeRecordedFact,
    pub backend_verifier: NativeRecordedFact,
    pub object_emission: NativeRecordedFact,
    pub linker_or_finalizer: NativeRecordedFact,
    pub host_platform: NativeRecordedFact,
    pub library_abi: NativeRecordedFact,
    pub c_abi_interop: NativeRecordedFact,
    pub rust_interop: NativeRecordedFact,
    pub cross_package_native_linking: NativeRecordedFact,
    pub whole_compiler_proof: NativeRecordedFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeRecordedFact {
    Available {
        value: String,
        evidence_source: String,
        lane: ObjectLinkerEvidenceLane,
    },
    Unavailable {
        reason: String,
        lane: ObjectLinkerEvidenceLane,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutableEvidenceLane {
    pub claim: NativeExecutableEvidenceClaim,
    pub status: NativeExecutableEvidenceStatus,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutableEvidenceClaim {
    NativeExecution,
    RuntimeIrDifferential,
    InterpreterDifferential,
    EffectForeignExecutablePolicy,
    TranslationValidation,
    WholeCompilerProof,
    LibraryAbi,
    CAbiInterop,
    RustInterop,
    CrossPackageNativeLinking,
    ForeignExecution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeExecutableEvidenceStatus {
    Tested,
    Validated,
    Proved,
    Unavailable { reason: String },
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
    Provenance,
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
    crate::assert_native_target_abi().map_err(|error| NativeExecutionDifferentialError {
        stage: NativeExecutionDifferentialStage::NativeExecution,
        field: "target_abi_manifest_hash",
        reason: error.to_string(),
    })?;
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
        let native = NativeExecutionLaneReport::Unavailable {
            reason: reason.clone(),
            evidence_source:
                "NC25 effect/foreign executable policy rejected native execution before launch"
                    .to_string(),
        };
        let runtime_ir = NativeComparisonLaneReport::Unavailable {
            lane: NativeDifferentialLane::NativeExecution,
            reason: reason.clone(),
            evidence_source:
                "NC18 effect/foreign facts are represented; NC25 keeps native execution unavailable"
                    .to_string(),
        };
        let verdict = NativeExecutionDifferentialVerdict::Unavailable {
            lane: NativeDifferentialLane::NativeExecution,
            reason,
        };
        let trust = native_executable_trust_report(
            program,
            package,
            run_report,
            &target,
            &native,
            &runtime_ir,
            &interpreter,
            &verdict,
            &effect_foreign_policy,
        );
        return Ok(NativeExecutionDifferentialReport {
            header: NativeExecutionDifferentialHeader {
                report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
                version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
                spec_ref: NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF.to_string(),
                producer: producer.into(),
            },
            target,
            native,
            runtime_ir,
            interpreter,
            verdict,
            effect_foreign_policy,
            trust,
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
        let native = NativeExecutionLaneReport::Unavailable {
            reason: reason.clone(),
            evidence_source:
                "RuntimeIrRunReport observed a trap; NC23 executable ABI carries scalar stdout only"
                    .to_string(),
        };
        let runtime_ir = NativeComparisonLaneReport::Unavailable {
            lane: NativeDifferentialLane::NativeExecution,
            reason: reason.clone(),
            evidence_source:
                "runtime-IR trap observation is preserved but native trap decoding is unavailable"
                    .to_string(),
        };
        let verdict = NativeExecutionDifferentialVerdict::Unavailable {
            lane: NativeDifferentialLane::NativeExecution,
            reason,
        };
        let trust = native_executable_trust_report(
            program,
            package,
            run_report,
            &target,
            &native,
            &runtime_ir,
            &interpreter,
            &verdict,
            &effect_foreign_policy,
        );
        return Ok(NativeExecutionDifferentialReport {
            header: NativeExecutionDifferentialHeader {
                report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
                version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
                spec_ref: NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF.to_string(),
                producer: producer.into(),
            },
            target,
            native,
            runtime_ir,
            interpreter,
            verdict,
            effect_foreign_policy,
            trust,
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
    let native = NativeExecutionLaneReport::Available(native);
    let trust = native_executable_trust_report(
        program,
        package,
        run_report,
        &target,
        &native,
        &runtime_ir,
        &interpreter,
        &verdict,
        &effect_foreign_policy,
    );

    Ok(NativeExecutionDifferentialReport {
        header: NativeExecutionDifferentialHeader {
            report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
            version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
            spec_ref: NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF.to_string(),
            producer: producer.into(),
        },
        target,
        native,
        runtime_ir,
        interpreter,
        verdict,
        effect_foreign_policy,
        trust,
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

pub fn close_native_executable_phase<'a, I, J>(
    reports: I,
    additional_exclusions: J,
    producer: impl Into<String>,
) -> NativeExecutablePhaseCloseoutReport
where
    I: IntoIterator<Item = &'a NativeExecutionDifferentialReport>,
    J: IntoIterator<Item = NativeExecutableCorpusExclusion>,
{
    let mut positive_cases = Vec::new();
    let mut blockers = Vec::new();
    let mut claim_inventory = Vec::new();

    for report in reports {
        let report_hash = native_execution_differential_report_hash(report);
        claim_inventory.extend(phase_claim_inventory(report));
        if let Some(status) = positive_corpus_blocker(report) {
            blockers.push(NativeExecutableCorpusBlocker {
                target: report.target.clone(),
                native_differential_report_hash: report_hash,
                status,
                evidence_source: "NC27 closeout checked the exact NC26 report lanes".to_string(),
            });
        } else {
            positive_cases.push(NativeExecutableCorpusCase {
                target: report.target.clone(),
                checked_core: report.trust.provenance.checked_core.clone(),
                runtime_ir: report.trust.provenance.runtime_ir.clone(),
                object_linker: report.trust.provenance.object_linker.clone(),
                native_differential_report_hash: report_hash,
                status: NativeExecutablePhaseStatus::Supported,
            });
        }
    }

    let mut exclusions = required_nc27_exclusions();
    exclusions.extend(additional_exclusions);
    let recommendation = if !positive_cases.is_empty() && blockers.is_empty() {
        NativeExecutablePhaseRecommendation::ProceedToNc28 {
            reason: "starter executable corpus traverses the NC19-NC26 chain with native, runtime-IR, interpreter, effect-policy, and trust/provenance lanes tested; remaining executable claims are explicitly unavailable or unsupported"
                .to_string(),
        }
    } else {
        let reason = if positive_cases.is_empty() {
            "no exact starter executable corpus item completed the full NC19-NC26 chain".to_string()
        } else {
            "at least one intended starter executable item failed the NC27 closeout gate"
                .to_string()
        };
        NativeExecutablePhaseRecommendation::FramePrerequisiteWp {
            reason,
            blockers: blockers.clone(),
        }
    };

    NativeExecutablePhaseCloseoutReport {
        header: NativeExecutablePhaseCloseoutHeader {
            report_kind: NATIVE_EXECUTABLE_PHASE_CLOSEOUT_REPORT_KIND.to_string(),
            version: NATIVE_EXECUTABLE_PHASE_CLOSEOUT_REPORT_VERSION,
            spec_ref: NATIVE_EXECUTABLE_PHASE_CLOSEOUT_SPEC_REF.to_string(),
            producer: producer.into(),
            source_report_kind: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND.to_string(),
            source_report_version: NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION,
        },
        corpus: NativeExecutableCorpusManifest {
            positive_cases,
            blockers,
        },
        claim_inventory,
        exclusions,
        recommendation,
    }
}

pub fn native_execution_differential_report_hash(
    report: &NativeExecutionDifferentialReport,
) -> u64 {
    fnv1a_64(&canonical_native_execution_differential_report_bytes(
        report,
    ))
}

pub fn required_nc27_exclusions() -> Vec<NativeExecutableCorpusExclusion> {
    vec![
        exclusion(
            NativeExecutableCorpusExclusionKind::NonStarterExecutableArtifact,
            NativeExecutablePhaseStatus::Unsupported {
                reason: "NC24 only runs NC23 StarterExecutable artifacts".to_string(),
            },
            "non-starter executable artifacts are outside the NC19-NC27 starter corpus",
            "NC24 validate_executable_artifact executable_artifact.kind preflight",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::NonScalarResultDecoding,
            NativeExecutablePhaseStatus::Unsupported {
                reason: "NC24 starter native execution only decodes scalar Int/Bool observations"
                    .to_string(),
            },
            "non-scalar native result decoding is outside the starter executable ABI",
            "NC24 decode_native_stdout runtime_observation preflight",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::TrapDecoding,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "NC23 executable ABI carries scalar stdout only; trap decoding remains unavailable"
                    .to_string(),
            },
            "trap observations are preserved as unavailable instead of hidden from the corpus",
            "NC24 trap path returns first-class unavailable native execution lanes",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::HostEffectExecution,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "host-effect execution is represented by NC18/NC25 facts but unavailable for native execution"
                    .to_string(),
            },
            "host-effect targets are not part of the positive starter executable corpus",
            "NC25 effect/foreign executable policy report",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::ForeignExecution,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "foreign execution remains unavailable unless a later host policy produces authority"
                    .to_string(),
            },
            "foreign-boundary targets are explicitly unavailable for NC19-NC27",
            "NC25 effect/foreign executable policy and NC26 trust lanes",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::LibraryAbi,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "native library ABI is outside the NC19-NC27 executable phase".to_string(),
            },
            "library artifacts are deferred to NC37-NC45",
            "NC26 trust report unavailable lane",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::CAbiInterop,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "C ABI interop is outside the NC19-NC27 executable phase".to_string(),
            },
            "C ABI interop is deferred until library/FFI phases",
            "NC26 trust report unavailable lane",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::RustInterop,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "Rust interop is outside the NC19-NC27 executable phase".to_string(),
            },
            "Rust interop is deferred until library/FFI phases",
            "NC26 trust report unavailable lane",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::CrossPackageNativeLinking,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "cross-package native linking is outside the NC19-NC27 executable phase"
                    .to_string(),
            },
            "cross-package native linking is deferred until library artifacts exist",
            "NC26 trust report unavailable lane",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::TranslationValidation,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "NC26 records no translation-validation producing check for native executables"
                    .to_string(),
            },
            "translation validation belongs to NC28-NC36, not the executable emitter closeout",
            "NC26 trust report honesty guard",
        ),
        exclusion(
            NativeExecutableCorpusExclusionKind::WholeCompilerProof,
            NativeExecutablePhaseStatus::Unavailable {
                reason: "native execution, Cranelift emission, and linker smoke tests are not Ken proof"
                    .to_string(),
            },
            "whole-compiler proof remains unavailable after the NC19-NC27 executable phase",
            "NC26 trust report honesty guard",
        ),
    ]
}

fn exclusion(
    exclusion: NativeExecutableCorpusExclusionKind,
    status: NativeExecutablePhaseStatus,
    reason: &str,
    evidence_source: &str,
) -> NativeExecutableCorpusExclusion {
    NativeExecutableCorpusExclusion {
        exclusion,
        status,
        reason: reason.to_string(),
        evidence_source: evidence_source.to_string(),
    }
}

fn positive_corpus_blocker(
    report: &NativeExecutionDifferentialReport,
) -> Option<NativeExecutablePhaseStatus> {
    if report.header.report_kind != NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND
        || report.header.version != NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION
    {
        return Some(NativeExecutablePhaseStatus::Failed {
            reason: "source report is not the landed NC26 native differential report kind/version"
                .to_string(),
        });
    }
    match &report.effect_foreign_policy.status {
        NativeEffectForeignExecutableStatus::NativeTested => {}
        NativeEffectForeignExecutableStatus::RepresentedUnavailable { reason } => {
            return Some(NativeExecutablePhaseStatus::Unavailable {
                reason: reason.clone(),
            });
        }
        NativeEffectForeignExecutableStatus::Unsupported { reason } => {
            return Some(NativeExecutablePhaseStatus::Unsupported {
                reason: reason.clone(),
            });
        }
    }
    if !matches!(report.native, NativeExecutionLaneReport::Available(_)) {
        return Some(NativeExecutablePhaseStatus::Unavailable {
            reason: "native execution lane is unavailable for this target".to_string(),
        });
    }
    match &report.runtime_ir {
        NativeComparisonLaneReport::TestedAgreement {
            lane: NativeDifferentialLane::RuntimeIrEvaluator,
            ..
        } => {}
        NativeComparisonLaneReport::Mismatch { diagnostic, .. } => {
            return Some(NativeExecutablePhaseStatus::Failed {
                reason: diagnostic.message.clone(),
            });
        }
        NativeComparisonLaneReport::TestedAgreement { .. } => {
            return Some(NativeExecutablePhaseStatus::Failed {
                reason: "runtime-IR differential lane has the wrong comparison lane".to_string(),
            });
        }
        NativeComparisonLaneReport::Unavailable { reason, .. } => {
            return Some(NativeExecutablePhaseStatus::Unavailable {
                reason: format!("runtime-IR differential unavailable: {reason}"),
            });
        }
    }
    match &report.interpreter {
        NativeComparisonLaneReport::TestedAgreement {
            lane: NativeDifferentialLane::Interpreter,
            ..
        } => {}
        NativeComparisonLaneReport::Mismatch { diagnostic, .. } => {
            return Some(NativeExecutablePhaseStatus::Failed {
                reason: diagnostic.message.clone(),
            });
        }
        NativeComparisonLaneReport::TestedAgreement { .. } => {
            return Some(NativeExecutablePhaseStatus::Failed {
                reason: "interpreter differential lane has the wrong comparison lane".to_string(),
            });
        }
        NativeComparisonLaneReport::Unavailable { reason, .. } => {
            return Some(NativeExecutablePhaseStatus::Unavailable {
                reason: format!("interpreter differential unavailable: {reason}"),
            });
        }
    }
    if !matches!(
        report.verdict,
        NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement {
            runtime_ir: NativeLaneVerdict::TestedAgreement,
            interpreter: NativeLaneVerdict::TestedAgreement,
        }
    ) {
        return Some(NativeExecutablePhaseStatus::Failed {
            reason: "native differential verdict does not close both comparison lanes".to_string(),
        });
    }
    if !required_unavailable_claims().is_subset(&report.unavailable_claims) {
        return Some(NativeExecutablePhaseStatus::Failed {
            reason: "NC26 unavailable claim set is incomplete".to_string(),
        });
    }
    for claim in [
        NativeExecutableEvidenceClaim::NativeExecution,
        NativeExecutableEvidenceClaim::RuntimeIrDifferential,
        NativeExecutableEvidenceClaim::InterpreterDifferential,
        NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy,
    ] {
        match report
            .trust
            .evidence_lanes
            .iter()
            .find(|lane| lane.claim == claim)
        {
            Some(NativeExecutableEvidenceLane {
                status: NativeExecutableEvidenceStatus::Tested,
                ..
            }) => {}
            _ => {
                return Some(NativeExecutablePhaseStatus::Failed {
                    reason: format!(
                        "required tested trust lane is absent or not tested: {claim:?}"
                    ),
                });
            }
        }
    }
    for lane in &report.trust.evidence_lanes {
        if out_of_phase_claim(&lane.claim)
            && matches!(
                lane.status,
                NativeExecutableEvidenceStatus::Tested
                    | NativeExecutableEvidenceStatus::Validated
                    | NativeExecutableEvidenceStatus::Proved
            )
        {
            return Some(NativeExecutablePhaseStatus::Failed {
                reason: format!(
                    "out-of-phase executable claim was over-promoted in NC26 trust report: {:?}",
                    lane.claim
                ),
            });
        }
    }
    None
}

fn out_of_phase_claim(claim: &NativeExecutableEvidenceClaim) -> bool {
    matches!(
        claim,
        NativeExecutableEvidenceClaim::TranslationValidation
            | NativeExecutableEvidenceClaim::WholeCompilerProof
            | NativeExecutableEvidenceClaim::LibraryAbi
            | NativeExecutableEvidenceClaim::CAbiInterop
            | NativeExecutableEvidenceClaim::RustInterop
            | NativeExecutableEvidenceClaim::CrossPackageNativeLinking
            | NativeExecutableEvidenceClaim::ForeignExecution
    )
}

fn phase_claim_inventory(
    report: &NativeExecutionDifferentialReport,
) -> Vec<NativeExecutablePhaseClaim> {
    let mut inventory = vec![
        phase_claim(
            report,
            NativeExecutableEvidenceClaim::NativeExecution,
            phase_status_from_native_execution(&report.native, &report.effect_foreign_policy),
            native_execution_evidence_source(&report.native),
        ),
        phase_claim(
            report,
            NativeExecutableEvidenceClaim::RuntimeIrDifferential,
            phase_status_from_comparison(&report.runtime_ir),
            comparison_evidence_source(&report.runtime_ir),
        ),
        phase_claim(
            report,
            NativeExecutableEvidenceClaim::InterpreterDifferential,
            phase_status_from_comparison(&report.interpreter),
            comparison_evidence_source(&report.interpreter),
        ),
        phase_claim(
            report,
            NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy,
            phase_status_from_evidence(&effect_policy_evidence_status(
                &report.effect_foreign_policy,
            )),
            report.effect_foreign_policy.evidence_source.clone(),
        ),
    ];
    inventory.extend(
        report
            .trust
            .evidence_lanes
            .iter()
            .filter(|lane| out_of_phase_claim(&lane.claim))
            .map(|lane| {
                phase_claim(
                    report,
                    lane.claim.clone(),
                    phase_status_from_out_of_phase_claim(&lane.claim, &lane.status),
                    lane.evidence_source.clone(),
                )
            }),
    );
    inventory
}

fn phase_claim(
    report: &NativeExecutionDifferentialReport,
    claim: NativeExecutableEvidenceClaim,
    status: NativeExecutablePhaseStatus,
    evidence_source: String,
) -> NativeExecutablePhaseClaim {
    NativeExecutablePhaseClaim {
        target_symbol: report.target.target_symbol.clone(),
        claim,
        status,
        evidence_source,
    }
}

fn phase_status_from_comparison(
    report: &NativeComparisonLaneReport,
) -> NativeExecutablePhaseStatus {
    match report {
        NativeComparisonLaneReport::TestedAgreement { .. } => NativeExecutablePhaseStatus::Tested,
        NativeComparisonLaneReport::Mismatch { diagnostic, .. } => {
            NativeExecutablePhaseStatus::Failed {
                reason: diagnostic.message.clone(),
            }
        }
        NativeComparisonLaneReport::Unavailable { reason, .. } => {
            NativeExecutablePhaseStatus::Unavailable {
                reason: reason.clone(),
            }
        }
    }
}

fn phase_status_from_native_execution(
    native: &NativeExecutionLaneReport,
    effect_policy: &NativeEffectForeignExecutablePolicyReport,
) -> NativeExecutablePhaseStatus {
    match (native, &effect_policy.status) {
        (
            NativeExecutionLaneReport::Available(_),
            NativeEffectForeignExecutableStatus::NativeTested,
        ) => NativeExecutablePhaseStatus::Tested,
        (
            NativeExecutionLaneReport::Available(_),
            NativeEffectForeignExecutableStatus::RepresentedUnavailable { reason },
        ) => NativeExecutablePhaseStatus::Failed {
            reason: format!(
                "native execution was observed despite unavailable effect/foreign policy: {reason}"
            ),
        },
        (
            NativeExecutionLaneReport::Available(_),
            NativeEffectForeignExecutableStatus::Unsupported { reason },
        ) => NativeExecutablePhaseStatus::Failed {
            reason: format!(
                "native execution was observed despite unsupported effect/foreign policy: {reason}"
            ),
        },
        (
            NativeExecutionLaneReport::Unavailable { .. },
            NativeEffectForeignExecutableStatus::Unsupported { reason },
        ) => NativeExecutablePhaseStatus::Unsupported {
            reason: reason.clone(),
        },
        (NativeExecutionLaneReport::Unavailable { reason, .. }, _) => {
            NativeExecutablePhaseStatus::Unavailable {
                reason: reason.clone(),
            }
        }
    }
}

fn phase_status_from_out_of_phase_claim(
    claim: &NativeExecutableEvidenceClaim,
    status: &NativeExecutableEvidenceStatus,
) -> NativeExecutablePhaseStatus {
    match status {
        NativeExecutableEvidenceStatus::Tested
        | NativeExecutableEvidenceStatus::Validated
        | NativeExecutableEvidenceStatus::Proved => NativeExecutablePhaseStatus::Failed {
            reason: format!(
                "out-of-phase executable claim was over-promoted in NC26 trust report: {claim:?}"
            ),
        },
        NativeExecutableEvidenceStatus::Unavailable { reason } => {
            NativeExecutablePhaseStatus::Unavailable {
                reason: reason.clone(),
            }
        }
        NativeExecutableEvidenceStatus::Unsupported { reason } => {
            NativeExecutablePhaseStatus::Unsupported {
                reason: reason.clone(),
            }
        }
    }
}

fn phase_status_from_evidence(
    status: &NativeExecutableEvidenceStatus,
) -> NativeExecutablePhaseStatus {
    match status {
        NativeExecutableEvidenceStatus::Tested => NativeExecutablePhaseStatus::Tested,
        NativeExecutableEvidenceStatus::Validated => NativeExecutablePhaseStatus::Validated,
        NativeExecutableEvidenceStatus::Proved => NativeExecutablePhaseStatus::Proved,
        NativeExecutableEvidenceStatus::Unavailable { reason } => {
            NativeExecutablePhaseStatus::Unavailable {
                reason: reason.clone(),
            }
        }
        NativeExecutableEvidenceStatus::Unsupported { reason } => {
            NativeExecutablePhaseStatus::Unsupported {
                reason: reason.clone(),
            }
        }
    }
}

fn canonical_native_execution_differential_report_bytes(
    report: &NativeExecutionDifferentialReport,
) -> Vec<u8> {
    let mut out = String::new();
    push_native_field(&mut out, "header.report_kind", &report.header.report_kind);
    push_native_field(
        &mut out,
        "header.version",
        &report.header.version.to_string(),
    );
    push_native_field(&mut out, "header.spec_ref", &report.header.spec_ref);
    push_native_field(&mut out, "header.producer", &report.header.producer);
    push_target_identity(&mut out, "target", &report.target);
    push_native_lane_report(&mut out, "native", &report.native);
    push_comparison_lane_report(&mut out, "runtime_ir", &report.runtime_ir);
    push_comparison_lane_report(&mut out, "interpreter", &report.interpreter);
    push_differential_verdict(&mut out, "verdict", &report.verdict);
    push_policy_report(&mut out, &report.effect_foreign_policy);
    push_trust_report(&mut out, &report.trust);
    for claim in &report.unavailable_claims {
        push_native_field(&mut out, "unavailable_claim", unavailable_claim_tag(claim));
    }
    out.into_bytes()
}

fn push_target_identity(out: &mut String, prefix: &str, target: &NativeExecutionTargetIdentity) {
    push_native_field(
        out,
        &format!("{prefix}.package_identity"),
        &target.package_identity,
    );
    push_native_field(
        out,
        &format!("{prefix}.target_symbol"),
        &target.target_symbol,
    );
    push_runtime_artifact_identity(
        out,
        &format!("{prefix}.runtime_artifact"),
        &target.runtime_artifact,
    );
    push_native_field(
        out,
        &format!("{prefix}.runtime_report_hash"),
        &format!("{:016x}", target.runtime_report_hash),
    );
    push_native_field(
        out,
        &format!("{prefix}.object_linker_package_hash"),
        &format!("{:016x}", target.object_linker_package_hash),
    );
    push_native_field(
        out,
        &format!("{prefix}.executable_artifact_hash"),
        &format!("{:016x}", target.executable_artifact_hash),
    );
    push_native_field(
        out,
        &format!("{prefix}.executable_relative_path"),
        &target.executable_relative_path,
    );
}

fn push_runtime_artifact_identity(
    out: &mut String,
    prefix: &str,
    artifact: &RuntimeArtifactIdentity,
) {
    push_native_field(
        out,
        &format!("{prefix}.package_identity"),
        &artifact.package_identity,
    );
    push_native_field(
        out,
        &format!("{prefix}.core_semantic_hash"),
        &format!("{:016x}", artifact.core_semantic_hash),
    );
    push_native_field(
        out,
        &format!("{prefix}.artifact_hash"),
        &format!("{:016x}", artifact.artifact_hash),
    );
}

fn push_runtime_ir_target_identity(
    out: &mut String,
    prefix: &str,
    target: &RuntimeIrTargetIdentity,
) {
    push_native_field(out, &format!("{prefix}.example"), &target.example);
    push_native_field(
        out,
        &format!("{prefix}.checked_core_shape"),
        &target.checked_core_shape,
    );
}

fn push_native_lane_report(out: &mut String, prefix: &str, lane: &NativeExecutionLaneReport) {
    match lane {
        NativeExecutionLaneReport::Available(observation) => {
            push_native_field(out, &format!("{prefix}.status"), "available");
            push_observation(
                out,
                &format!("{prefix}.observation"),
                &observation.observation,
            );
            push_native_field(out, &format!("{prefix}.stdout"), &observation.stdout);
            push_native_field(
                out,
                &format!("{prefix}.exit_status"),
                &observation.exit_status.to_string(),
            );
            push_native_field(
                out,
                &format!("{prefix}.evidence_source"),
                &observation.evidence_source,
            );
        }
        NativeExecutionLaneReport::Unavailable {
            reason,
            evidence_source,
        } => {
            push_native_field(out, &format!("{prefix}.status"), "unavailable");
            push_native_field(out, &format!("{prefix}.reason"), reason);
            push_native_field(out, &format!("{prefix}.evidence_source"), evidence_source);
        }
    }
}

fn push_comparison_lane_report(out: &mut String, prefix: &str, lane: &NativeComparisonLaneReport) {
    match lane {
        NativeComparisonLaneReport::TestedAgreement {
            lane,
            expected,
            observed,
            evidence_source,
        } => {
            push_native_field(out, &format!("{prefix}.status"), "tested_agreement");
            push_native_field(out, &format!("{prefix}.lane"), differential_lane_tag(lane));
            push_observation(out, &format!("{prefix}.expected"), expected);
            push_observation(out, &format!("{prefix}.observed"), observed);
            push_native_field(out, &format!("{prefix}.evidence_source"), evidence_source);
        }
        NativeComparisonLaneReport::Mismatch {
            lane,
            expected,
            observed,
            diagnostic,
        } => {
            push_native_field(out, &format!("{prefix}.status"), "mismatch");
            push_native_field(out, &format!("{prefix}.lane"), differential_lane_tag(lane));
            push_observation(out, &format!("{prefix}.expected"), expected);
            push_observation(out, &format!("{prefix}.observed"), observed);
            push_mismatch_diagnostic(out, &format!("{prefix}.diagnostic"), diagnostic);
        }
        NativeComparisonLaneReport::Unavailable {
            lane,
            reason,
            evidence_source,
        } => {
            push_native_field(out, &format!("{prefix}.status"), "unavailable");
            push_native_field(out, &format!("{prefix}.lane"), differential_lane_tag(lane));
            push_native_field(out, &format!("{prefix}.reason"), reason);
            push_native_field(out, &format!("{prefix}.evidence_source"), evidence_source);
        }
    }
}

fn push_differential_verdict(
    out: &mut String,
    prefix: &str,
    verdict: &NativeExecutionDifferentialVerdict,
) {
    match verdict {
        NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement {
            runtime_ir,
            interpreter,
        } => {
            push_native_field(
                out,
                &format!("{prefix}.kind"),
                "runtime_ir_tested_agreement",
            );
            push_lane_verdict(out, &format!("{prefix}.runtime_ir"), runtime_ir);
            push_lane_verdict(out, &format!("{prefix}.interpreter"), interpreter);
        }
        NativeExecutionDifferentialVerdict::Mismatch { lane, diagnostic } => {
            push_native_field(out, &format!("{prefix}.kind"), "mismatch");
            push_native_field(out, &format!("{prefix}.lane"), differential_lane_tag(lane));
            push_mismatch_diagnostic(out, &format!("{prefix}.diagnostic"), diagnostic);
        }
        NativeExecutionDifferentialVerdict::Unavailable { lane, reason } => {
            push_native_field(out, &format!("{prefix}.kind"), "unavailable");
            push_native_field(out, &format!("{prefix}.lane"), differential_lane_tag(lane));
            push_native_field(out, &format!("{prefix}.reason"), reason);
        }
    }
}

fn push_lane_verdict(out: &mut String, prefix: &str, verdict: &NativeLaneVerdict) {
    match verdict {
        NativeLaneVerdict::TestedAgreement => {
            push_native_field(out, &format!("{prefix}.kind"), "tested_agreement");
        }
        NativeLaneVerdict::Unavailable { reason } => {
            push_native_field(out, &format!("{prefix}.kind"), "unavailable");
            push_native_field(out, &format!("{prefix}.reason"), reason);
        }
    }
}

fn push_policy_report(out: &mut String, report: &NativeEffectForeignExecutablePolicyReport) {
    push_native_field(out, "effect_policy.target_symbol", &report.target_symbol);
    match &report.status {
        NativeEffectForeignExecutableStatus::NativeTested => {
            push_native_field(out, "effect_policy.status", "native_tested");
        }
        NativeEffectForeignExecutableStatus::RepresentedUnavailable { reason } => {
            push_native_field(out, "effect_policy.status", "represented_unavailable");
            push_native_field(out, "effect_policy.reason", reason);
        }
        NativeEffectForeignExecutableStatus::Unsupported { reason } => {
            push_native_field(out, "effect_policy.status", "unsupported");
            push_native_field(out, "effect_policy.reason", reason);
        }
    }
    for fact in &report.facts {
        push_native_field(out, "effect_policy.fact", fact);
    }
    push_native_field(
        out,
        "effect_policy.evidence_source",
        &report.evidence_source,
    );
}

fn push_trust_report(out: &mut String, report: &NativeExecutableTrustReport) {
    push_checked_core_provenance(out, &report.provenance.checked_core);
    push_runtime_ir_provenance(out, &report.provenance.runtime_ir);
    push_object_linker_provenance(out, &report.provenance.object_linker);
    push_toolchain_provenance(out, &report.provenance.toolchain);
    for lane in &report.evidence_lanes {
        push_native_field(out, "trust.evidence.claim", evidence_claim_tag(&lane.claim));
        push_phase_evidence_status(out, "trust.evidence.status", &lane.status);
        push_native_field(out, "trust.evidence.evidence_source", &lane.evidence_source);
    }
}

fn push_checked_core_provenance(out: &mut String, provenance: &NativeCheckedCoreProvenance) {
    push_native_field(
        out,
        "trust.checked_core.package_identity",
        &provenance.package_identity,
    );
    push_native_field(
        out,
        "trust.checked_core.core_semantic_hash",
        &format!("{:016x}", provenance.core_semantic_hash),
    );
    push_native_field(
        out,
        "trust.checked_core.artifact_hash",
        &format!("{:016x}", provenance.artifact_hash),
    );
    push_native_field(
        out,
        "trust.checked_core.evidence_source",
        &provenance.evidence_source,
    );
}

fn push_runtime_ir_provenance(out: &mut String, provenance: &NativeRuntimeIrProvenance) {
    push_runtime_artifact_identity(
        out,
        "trust.runtime_ir.runtime_artifact",
        &provenance.runtime_artifact,
    );
    push_runtime_ir_target_identity(out, "trust.runtime_ir.target", &provenance.target);
    push_native_field(
        out,
        "trust.runtime_ir.runtime_report_hash",
        &format!("{:016x}", provenance.runtime_report_hash),
    );
    push_native_field(
        out,
        "trust.runtime_ir.evidence_source",
        &provenance.evidence_source,
    );
}

fn push_object_linker_provenance(out: &mut String, provenance: &NativeObjectLinkerProvenance) {
    push_native_field(
        out,
        "trust.object_linker.package_hash",
        &format!("{:016x}", provenance.object_linker_package_hash),
    );
    push_native_field(
        out,
        "trust.object_linker.entrypoint_package_hash",
        &format!("{:016x}", provenance.entrypoint_package_hash),
    );
    push_native_field(
        out,
        "trust.object_linker.platform_runtime_support_hash",
        &format!("{:016x}", provenance.platform_runtime_support_hash),
    );
    push_native_field(
        out,
        "trust.object_linker.object_kind",
        object_artifact_kind_tag(&provenance.object_artifact_kind),
    );
    push_native_field(
        out,
        "trust.object_linker.object_hash",
        &format!("{:016x}", provenance.object_artifact_hash),
    );
    push_native_field(
        out,
        "trust.object_linker.object_byte_len",
        &provenance.object_byte_len.to_string(),
    );
    push_native_field(
        out,
        "trust.object_linker.executable_hash",
        &format!("{:016x}", provenance.executable_artifact_hash),
    );
    push_native_field(
        out,
        "trust.object_linker.executable_byte_len",
        &provenance.executable_byte_len.to_string(),
    );
    push_native_field(
        out,
        "trust.object_linker.executable_relative_path",
        &provenance.executable_relative_path,
    );
    push_native_field(
        out,
        "trust.object_linker.smoke_evidence_source",
        &provenance.smoke_evidence_source,
    );
}

fn push_toolchain_provenance(out: &mut String, provenance: &NativeToolchainProvenance) {
    push_recorded_fact(out, "trust.toolchain.ken_runtime", &provenance.ken_runtime);
    push_recorded_fact(
        out,
        "trust.toolchain.native_backend",
        &provenance.native_backend,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.backend_verifier",
        &provenance.backend_verifier,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.object_emission",
        &provenance.object_emission,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.linker_or_finalizer",
        &provenance.linker_or_finalizer,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.host_platform",
        &provenance.host_platform,
    );
    push_recorded_fact(out, "trust.toolchain.library_abi", &provenance.library_abi);
    push_recorded_fact(
        out,
        "trust.toolchain.c_abi_interop",
        &provenance.c_abi_interop,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.rust_interop",
        &provenance.rust_interop,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.cross_package_native_linking",
        &provenance.cross_package_native_linking,
    );
    push_recorded_fact(
        out,
        "trust.toolchain.whole_compiler_proof",
        &provenance.whole_compiler_proof,
    );
}

fn push_recorded_fact(out: &mut String, prefix: &str, fact: &NativeRecordedFact) {
    match fact {
        NativeRecordedFact::Available {
            value,
            evidence_source,
            lane,
        } => {
            push_native_field(out, &format!("{prefix}.kind"), "available");
            push_native_field(out, &format!("{prefix}.value"), value);
            push_native_field(out, &format!("{prefix}.evidence_source"), evidence_source);
            push_native_field(
                out,
                &format!("{prefix}.lane"),
                object_linker_evidence_lane_tag(lane),
            );
        }
        NativeRecordedFact::Unavailable { reason, lane } => {
            push_native_field(out, &format!("{prefix}.kind"), "unavailable");
            push_native_field(out, &format!("{prefix}.reason"), reason);
            push_native_field(
                out,
                &format!("{prefix}.lane"),
                object_linker_evidence_lane_tag(lane),
            );
        }
    }
}

fn push_phase_evidence_status(
    out: &mut String,
    prefix: &str,
    status: &NativeExecutableEvidenceStatus,
) {
    match status {
        NativeExecutableEvidenceStatus::Tested => {
            push_native_field(out, prefix, "tested");
        }
        NativeExecutableEvidenceStatus::Validated => {
            push_native_field(out, prefix, "validated");
        }
        NativeExecutableEvidenceStatus::Proved => {
            push_native_field(out, prefix, "proved");
        }
        NativeExecutableEvidenceStatus::Unavailable { reason } => {
            push_native_field(out, prefix, "unavailable");
            push_native_field(out, &format!("{prefix}.reason"), reason);
        }
        NativeExecutableEvidenceStatus::Unsupported { reason } => {
            push_native_field(out, prefix, "unsupported");
            push_native_field(out, &format!("{prefix}.reason"), reason);
        }
    }
}

fn push_mismatch_diagnostic(out: &mut String, prefix: &str, diagnostic: &NativeMismatchDiagnostic) {
    push_native_field(
        out,
        &format!("{prefix}.package_identity"),
        &diagnostic.package_identity,
    );
    push_native_field(
        out,
        &format!("{prefix}.target_symbol"),
        &diagnostic.target_symbol,
    );
    push_native_field(
        out,
        &format!("{prefix}.executable_artifact_hash"),
        &format!("{:016x}", diagnostic.executable_artifact_hash),
    );
    push_native_field(
        out,
        &format!("{prefix}.lane"),
        differential_lane_tag(&diagnostic.lane),
    );
    push_observation(out, &format!("{prefix}.expected"), &diagnostic.expected);
    push_observation(out, &format!("{prefix}.observed"), &diagnostic.observed);
    push_native_field(out, &format!("{prefix}.message"), &diagnostic.message);
}

fn push_observation(out: &mut String, prefix: &str, observation: &RuntimeObservation) {
    match observation {
        RuntimeObservation::Returned(value) => {
            push_native_field(out, &format!("{prefix}.kind"), "returned");
            push_ground_value(out, &format!("{prefix}.value"), value);
        }
        RuntimeObservation::Trapped(trap) => {
            push_native_field(out, &format!("{prefix}.kind"), "trapped");
            push_native_field(
                out,
                &format!("{prefix}.trap.code"),
                trap_code_tag(&trap.code),
            );
            push_native_field(out, &format!("{prefix}.trap.message"), &trap.message);
        }
    }
}

fn push_ground_value(out: &mut String, prefix: &str, value: &RuntimeGroundValue) {
    match value {
        RuntimeGroundValue::Bool(value) => {
            push_native_field(out, &format!("{prefix}.kind"), "bool");
            push_native_field(out, &format!("{prefix}.value"), &value.to_string());
        }
        RuntimeGroundValue::Int(value) => {
            push_native_field(out, &format!("{prefix}.kind"), "int");
            push_native_field(out, &format!("{prefix}.value"), &value.to_string());
        }
        RuntimeGroundValue::Bytes(bytes) => {
            push_native_field(out, &format!("{prefix}.kind"), "bytes");
            for byte in bytes {
                push_native_field(out, &format!("{prefix}.byte"), &byte.to_string());
            }
        }
        RuntimeGroundValue::String(value) => {
            push_native_field(out, &format!("{prefix}.kind"), "string");
            push_native_field(out, &format!("{prefix}.value"), value);
        }
        RuntimeGroundValue::Constructor { constructor, args } => {
            push_native_field(out, &format!("{prefix}.kind"), "constructor");
            push_native_field(out, &format!("{prefix}.constructor"), constructor);
            for arg in args {
                push_ground_value(out, &format!("{prefix}.arg"), arg);
            }
        }
        RuntimeGroundValue::Record { fields } => {
            push_native_field(out, &format!("{prefix}.kind"), "record");
            for (name, value) in fields {
                push_native_field(out, &format!("{prefix}.field.name"), name);
                push_ground_value(out, &format!("{prefix}.field.value"), value);
            }
        }
    }
}

fn push_native_field(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push('=');
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push('\n');
}

fn evidence_claim_tag(claim: &NativeExecutableEvidenceClaim) -> &'static str {
    match claim {
        NativeExecutableEvidenceClaim::NativeExecution => "native_execution",
        NativeExecutableEvidenceClaim::RuntimeIrDifferential => "runtime_ir_differential",
        NativeExecutableEvidenceClaim::InterpreterDifferential => "interpreter_differential",
        NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy => {
            "effect_foreign_executable_policy"
        }
        NativeExecutableEvidenceClaim::TranslationValidation => "translation_validation",
        NativeExecutableEvidenceClaim::WholeCompilerProof => "whole_compiler_proof",
        NativeExecutableEvidenceClaim::LibraryAbi => "library_abi",
        NativeExecutableEvidenceClaim::CAbiInterop => "c_abi_interop",
        NativeExecutableEvidenceClaim::RustInterop => "rust_interop",
        NativeExecutableEvidenceClaim::CrossPackageNativeLinking => "cross_package_native_linking",
        NativeExecutableEvidenceClaim::ForeignExecution => "foreign_execution",
    }
}

fn unavailable_claim_tag(claim: &NativeExecutionUnavailableClaim) -> &'static str {
    match claim {
        NativeExecutionUnavailableClaim::LibraryAbi => "library_abi",
        NativeExecutionUnavailableClaim::CAbiInterop => "c_abi_interop",
        NativeExecutionUnavailableClaim::RustInterop => "rust_interop",
        NativeExecutionUnavailableClaim::ForeignExecution => "foreign_execution",
        NativeExecutionUnavailableClaim::EffectPolicyBroadening => "effect_policy_broadening",
        NativeExecutionUnavailableClaim::TranslationValidation => "translation_validation",
        NativeExecutionUnavailableClaim::WholeCompilerProof => "whole_compiler_proof",
    }
}

fn differential_lane_tag(lane: &NativeDifferentialLane) -> &'static str {
    match lane {
        NativeDifferentialLane::NativeExecution => "native_execution",
        NativeDifferentialLane::RuntimeIrEvaluator => "runtime_ir_evaluator",
        NativeDifferentialLane::Interpreter => "interpreter",
    }
}

fn object_artifact_kind_tag(kind: &ObjectLinkerArtifactKind) -> &'static str {
    match kind {
        ObjectLinkerArtifactKind::CraneliftObject => "cranelift_object",
        ObjectLinkerArtifactKind::StarterExecutable => "starter_executable",
    }
}

fn object_linker_evidence_lane_tag(lane: &ObjectLinkerEvidenceLane) -> &'static str {
    match lane {
        ObjectLinkerEvidenceLane::SemanticAuthority => "semantic_authority",
        ObjectLinkerEvidenceLane::Tested => "tested",
        ObjectLinkerEvidenceLane::BuildArtifact => "build_artifact",
        ObjectLinkerEvidenceLane::Unavailable => "unavailable",
        ObjectLinkerEvidenceLane::Unsupported => "unsupported",
    }
}

fn trap_code_tag(code: &crate::RuntimeTrapCode) -> &'static str {
    match code {
        crate::RuntimeTrapCode::UnsupportedErasure => "unsupported_erasure",
        crate::RuntimeTrapCode::UnsupportedPrimitivePartiality => {
            "unsupported_primitive_partiality"
        }
        crate::RuntimeTrapCode::MissingRuntimeMetadata => "missing_runtime_metadata",
        crate::RuntimeTrapCode::PatternMatchFailure => "pattern_match_failure",
        crate::RuntimeTrapCode::ExplicitTrap => "explicit_trap",
    }
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
    validate_object_linker_provenance(package)?;
    Ok(())
}

fn validate_object_linker_provenance(
    package: &ObjectLinkerExecutablePackage,
) -> Result<(), NativeExecutionDifferentialError> {
    if package.object_artifact.kind != ObjectLinkerArtifactKind::CraneliftObject {
        return Err(provenance_error(
            "object_artifact.kind",
            "object/linker package object artifact must be the Cranelift object named by NC23",
        ));
    }
    require_available_fact(
        "toolchain.native_backend",
        &package.toolchain.native_backend,
        ObjectLinkerEvidenceLane::BuildArtifact,
    )?;
    require_available_fact(
        "toolchain.linker_or_finalizer",
        &package.toolchain.linker_or_finalizer,
        ObjectLinkerEvidenceLane::BuildArtifact,
    )?;
    require_available_fact(
        "toolchain.ken_runtime",
        &package.toolchain.ken_runtime,
        ObjectLinkerEvidenceLane::BuildArtifact,
    )?;
    require_available_fact(
        "toolchain.host_platform",
        &package.toolchain.host_platform,
        ObjectLinkerEvidenceLane::Tested,
    )?;
    require_available_fact(
        "toolchain.backend_verifier",
        &package.toolchain.backend_verifier,
        ObjectLinkerEvidenceLane::Tested,
    )?;
    require_available_fact(
        "toolchain.object_emission",
        &package.toolchain.object_emission,
        ObjectLinkerEvidenceLane::BuildArtifact,
    )?;
    require_unavailable_fact("toolchain.library_abi", &package.toolchain.library_abi)?;
    require_unavailable_fact("toolchain.c_abi_interop", &package.toolchain.c_abi_interop)?;
    require_unavailable_fact("toolchain.rust_interop", &package.toolchain.rust_interop)?;
    require_unavailable_fact(
        "toolchain.cross_package_native_linking",
        &package.toolchain.cross_package_native_linking,
    )?;
    require_unavailable_fact(
        "toolchain.whole_compiler_proof",
        &package.toolchain.whole_compiler_proof,
    )?;

    let expected_object_hash =
        format!("object hash {:016x}", package.object_artifact.artifact_hash);
    match &package.toolchain.object_emission {
        ObjectLinkerEvidenceFact::Available { value, .. } if value == &expected_object_hash => {}
        ObjectLinkerEvidenceFact::Available { .. } => {
            return Err(provenance_error(
                "toolchain.object_emission",
                "object emission fact does not name the exact object artifact hash",
            ));
        }
        ObjectLinkerEvidenceFact::Unavailable { .. } => {
            unreachable!("require_available_fact accepted only available object emission facts")
        }
    }
    match &package.toolchain.host_platform {
        ObjectLinkerEvidenceFact::Available { value, .. }
            if value == &package.header.starter_platform_target => {}
        ObjectLinkerEvidenceFact::Available { .. } => {
            return Err(provenance_error(
                "toolchain.host_platform",
                "host platform fact does not match the object/linker package target",
            ));
        }
        ObjectLinkerEvidenceFact::Unavailable { .. } => {
            unreachable!("require_available_fact accepted only available host facts")
        }
    }
    match &package.toolchain.backend_verifier {
        ObjectLinkerEvidenceFact::Available { value, .. }
            if value == "Cranelift verifier passed: true" => {}
        ObjectLinkerEvidenceFact::Available { .. } => {
            return Err(provenance_error(
                "toolchain.backend_verifier",
                "backend verifier fact contradicts the exact successful Cranelift verifier producer invariant",
            ));
        }
        ObjectLinkerEvidenceFact::Unavailable { .. } => {
            unreachable!("require_available_fact accepted only available backend verifier facts")
        }
    }
    Ok(())
}

fn require_available_fact(
    field: &'static str,
    fact: &ObjectLinkerEvidenceFact,
    expected_lane: ObjectLinkerEvidenceLane,
) -> Result<(), NativeExecutionDifferentialError> {
    match fact {
        ObjectLinkerEvidenceFact::Available {
            value,
            evidence_source,
            lane,
        } if !value.trim().is_empty()
            && !evidence_source.trim().is_empty()
            && *lane == expected_lane =>
        {
            Ok(())
        }
        ObjectLinkerEvidenceFact::Available { .. } => Err(provenance_error(
            field,
            "available toolchain fact has the wrong lane or missing evidence text",
        )),
        ObjectLinkerEvidenceFact::Unavailable { .. } => Err(provenance_error(
            field,
            "required object/linker toolchain fact is unavailable",
        )),
    }
}

fn require_unavailable_fact(
    field: &'static str,
    fact: &ObjectLinkerEvidenceFact,
) -> Result<(), NativeExecutionDifferentialError> {
    match fact {
        ObjectLinkerEvidenceFact::Unavailable { reason, lane }
            if !reason.trim().is_empty() && *lane == ObjectLinkerEvidenceLane::Unavailable =>
        {
            Ok(())
        }
        ObjectLinkerEvidenceFact::Unavailable { .. } => Err(provenance_error(
            field,
            "unavailable toolchain fact has the wrong lane or missing reason",
        )),
        ObjectLinkerEvidenceFact::Available { .. } => Err(provenance_error(
            field,
            "unsupported library/interop/proof lane was reported available",
        )),
    }
}

fn provenance_error(
    field: &'static str,
    reason: impl Into<String>,
) -> NativeExecutionDifferentialError {
    differential_error(NativeExecutionDifferentialStage::Provenance, field, reason)
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
        let mut operations = BTreeSet::new();
        runtime_expr_host_ops(body, &mut operations);
        if !operations.is_empty() {
            for operation in &operations {
                facts.insert(format!(
                    "runtime_expr.host_op={:04x}:{:?}",
                    *operation as u16, operation
                ));
            }
            if let Some(operation) = operations
                .iter()
                .find(|operation| {
                    operation.availability() != ken_host::HostOpAvailabilityV1::NativeTested
                })
                .copied()
            {
                return Ok(policy_report(
                    target_symbol,
                    NativeEffectForeignExecutableStatus::RepresentedUnavailable {
                        reason: format!(
                            "host operation {operation:?} ({:04x}) remains individually unavailable",
                            operation as u16
                        ),
                    },
                    facts,
                    "sealed HostOpV1 availability policy",
                ));
            }
            return Ok(policy_report(
                target_symbol,
                NativeEffectForeignExecutableStatus::NativeTested,
                facts,
                "all transparent RuntimeExpr::Effect operations are in the pinned native set",
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

fn runtime_expr_host_ops(expr: &RuntimeExpr, operations: &mut BTreeSet<ken_host::HostOpV1>) {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. } => runtime_expr_host_ops(body, operations),
        RuntimeExpr::Effect {
            operation, args, ..
        } => {
            operations.insert(*operation);
            for arg in args {
                runtime_expr_host_ops(arg, operations);
            }
        }
        RuntimeExpr::Let { value, body } => {
            runtime_expr_host_ops(value, operations);
            runtime_expr_host_ops(body, operations);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            runtime_expr_host_ops(scrutinee, operations);
            runtime_expr_host_ops(then_expr, operations);
            runtime_expr_host_ops(else_expr, operations);
        }
        RuntimeExpr::PrimitiveCall { args, .. }
        | RuntimeExpr::Construct { args, .. }
        | RuntimeExpr::Call { args, .. } => {
            for arg in args {
                runtime_expr_host_ops(arg, operations);
            }
            if let RuntimeExpr::Call { callee, .. } = expr {
                runtime_expr_host_ops(callee, operations);
            }
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            runtime_expr_host_ops(scrutinee, operations);
            for case in cases {
                runtime_expr_host_ops(&case.body, operations);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            runtime_expr_host_ops(scrutinee, operations);
            for case in cases {
                runtime_expr_host_ops(&case.body, operations);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, value) in fields {
                runtime_expr_host_ops(value, operations);
            }
        }
        RuntimeExpr::Project { record, .. } => runtime_expr_host_ops(record, operations),
        RuntimeExpr::Closure { body, .. } => runtime_expr_host_ops(body, operations),
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for capture in captures {
                runtime_expr_host_ops(capture, operations);
            }
            runtime_expr_host_ops(body, operations);
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
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
            Ok(RuntimeObservation::Returned(RuntimeGroundValue::Int(
                (value).into(),
            )))
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

fn native_executable_trust_report(
    program: &RuntimeProgram,
    package: &ObjectLinkerExecutablePackage,
    run_report: &RuntimeIrRunReport,
    target: &NativeExecutionTargetIdentity,
    native: &NativeExecutionLaneReport,
    runtime_ir: &NativeComparisonLaneReport,
    interpreter: &NativeComparisonLaneReport,
    verdict: &NativeExecutionDifferentialVerdict,
    effect_policy: &NativeEffectForeignExecutablePolicyReport,
) -> NativeExecutableTrustReport {
    NativeExecutableTrustReport {
        provenance: NativeExecutableProvenance {
            checked_core: NativeCheckedCoreProvenance {
                package_identity: program.package_identity.clone(),
                core_semantic_hash: program.core_semantic_hash,
                artifact_hash: program.artifact_hash,
                evidence_source:
                    "RuntimeProgram package/core/artifact identity from exact checked-core erasure"
                        .to_string(),
            },
            runtime_ir: NativeRuntimeIrProvenance {
                runtime_artifact: target.runtime_artifact.clone(),
                target: run_report.target.clone(),
                runtime_report_hash: target.runtime_report_hash,
                evidence_source:
                    "RuntimeIrRunReport hash and target were preflight-checked against object/linker package"
                        .to_string(),
            },
            object_linker: NativeObjectLinkerProvenance {
                object_linker_package_hash: target.object_linker_package_hash,
                entrypoint_package_hash: package.entrypoint_package_hash,
                platform_runtime_support_hash: package.platform_runtime_support_hash,
                object_artifact_kind: package.object_artifact.kind.clone(),
                object_artifact_hash: package.object_artifact.artifact_hash,
                object_byte_len: package.object_artifact.byte_len,
                executable_artifact_hash: target.executable_artifact_hash,
                executable_byte_len: package.executable_artifact.byte_len,
                executable_relative_path: target.executable_relative_path.clone(),
                smoke_evidence_source: package.smoke.evidence_source.clone(),
            },
            toolchain: NativeToolchainProvenance {
                ken_runtime: recorded_fact(&package.toolchain.ken_runtime),
                native_backend: recorded_fact(&package.toolchain.native_backend),
                backend_verifier: recorded_fact(&package.toolchain.backend_verifier),
                object_emission: recorded_fact(&package.toolchain.object_emission),
                linker_or_finalizer: recorded_fact(&package.toolchain.linker_or_finalizer),
                host_platform: recorded_fact(&package.toolchain.host_platform),
                library_abi: recorded_fact(&package.toolchain.library_abi),
                c_abi_interop: recorded_fact(&package.toolchain.c_abi_interop),
                rust_interop: recorded_fact(&package.toolchain.rust_interop),
                cross_package_native_linking: recorded_fact(
                    &package.toolchain.cross_package_native_linking,
                ),
                whole_compiler_proof: recorded_fact(&package.toolchain.whole_compiler_proof),
            },
        },
        evidence_lanes: native_executable_evidence_lanes(
            native,
            runtime_ir,
            interpreter,
            verdict,
            effect_policy,
        ),
    }
}

fn recorded_fact(fact: &ObjectLinkerEvidenceFact) -> NativeRecordedFact {
    match fact {
        ObjectLinkerEvidenceFact::Available {
            value,
            evidence_source,
            lane,
        } => NativeRecordedFact::Available {
            value: value.clone(),
            evidence_source: evidence_source.clone(),
            lane: lane.clone(),
        },
        ObjectLinkerEvidenceFact::Unavailable { reason, lane } => NativeRecordedFact::Unavailable {
            reason: reason.clone(),
            lane: lane.clone(),
        },
    }
}

fn native_executable_evidence_lanes(
    native: &NativeExecutionLaneReport,
    runtime_ir: &NativeComparisonLaneReport,
    interpreter: &NativeComparisonLaneReport,
    verdict: &NativeExecutionDifferentialVerdict,
    effect_policy: &NativeEffectForeignExecutablePolicyReport,
) -> Vec<NativeExecutableEvidenceLane> {
    vec![
        evidence_lane(
            NativeExecutableEvidenceClaim::NativeExecution,
            native_execution_evidence_status(native, verdict, effect_policy),
            native_execution_evidence_source(native),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::RuntimeIrDifferential,
            comparison_evidence_status(runtime_ir),
            comparison_evidence_source(runtime_ir),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::InterpreterDifferential,
            comparison_evidence_status(interpreter),
            comparison_evidence_source(interpreter),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy,
            effect_policy_evidence_status(effect_policy),
            effect_policy.evidence_source.clone(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::TranslationValidation,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "NC26 records no translation-validation producing check for native executables"
                    .to_string(),
            },
            "NC26 trust report honesty guard".to_string(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::WholeCompilerProof,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "native execution, Cranelift emission, and linker smoke tests are not Ken proof"
                    .to_string(),
            },
            "NC26 trust report honesty guard".to_string(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::LibraryAbi,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "native library ABI is outside the NC19-NC27 executable phase".to_string(),
            },
            "NC26 trust report unavailable lane".to_string(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::CAbiInterop,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "C ABI interop is outside the NC19-NC27 executable phase".to_string(),
            },
            "NC26 trust report unavailable lane".to_string(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::RustInterop,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "Rust interop is outside the NC19-NC27 executable phase".to_string(),
            },
            "NC26 trust report unavailable lane".to_string(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::CrossPackageNativeLinking,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "cross-package native linking is outside the NC19-NC27 executable phase"
                    .to_string(),
            },
            "NC26 trust report unavailable lane".to_string(),
        ),
        evidence_lane(
            NativeExecutableEvidenceClaim::ForeignExecution,
            NativeExecutableEvidenceStatus::Unavailable {
                reason: "foreign execution remains unavailable unless a later host policy produces authority"
                    .to_string(),
            },
            "NC26 trust report unavailable lane".to_string(),
        ),
    ]
}

fn evidence_lane(
    claim: NativeExecutableEvidenceClaim,
    status: NativeExecutableEvidenceStatus,
    evidence_source: String,
) -> NativeExecutableEvidenceLane {
    NativeExecutableEvidenceLane {
        claim,
        status,
        evidence_source,
    }
}

fn native_execution_evidence_status(
    native: &NativeExecutionLaneReport,
    verdict: &NativeExecutionDifferentialVerdict,
    effect_policy: &NativeEffectForeignExecutablePolicyReport,
) -> NativeExecutableEvidenceStatus {
    match (native, verdict, &effect_policy.status) {
        (
            NativeExecutionLaneReport::Available(_),
            NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement { .. },
            NativeEffectForeignExecutableStatus::NativeTested,
        ) => NativeExecutableEvidenceStatus::Tested,
        (
            NativeExecutionLaneReport::Available(_),
            NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement { .. },
            NativeEffectForeignExecutableStatus::RepresentedUnavailable { reason },
        ) => NativeExecutableEvidenceStatus::Unavailable {
            reason: format!(
                "inconsistent NC26 report: native execution was tested despite unavailable effect/foreign policy: {reason}"
            ),
        },
        (
            NativeExecutionLaneReport::Available(_),
            NativeExecutionDifferentialVerdict::RuntimeIrTestedAgreement { .. },
            NativeEffectForeignExecutableStatus::Unsupported { reason },
        ) => NativeExecutableEvidenceStatus::Unsupported {
            reason: format!(
                "inconsistent NC26 report: native execution was tested despite unsupported effect/foreign policy: {reason}"
            ),
        },
        (NativeExecutionLaneReport::Available(_), NativeExecutionDifferentialVerdict::Mismatch { diagnostic, .. }, _) => {
            NativeExecutableEvidenceStatus::Unavailable {
                reason: format!("native execution produced a mismatch: {}", diagnostic.message),
            }
        }
        (NativeExecutionLaneReport::Available(_), NativeExecutionDifferentialVerdict::Unavailable { reason, .. }, _) => {
            NativeExecutableEvidenceStatus::Unavailable {
                reason: reason.clone(),
            }
        }
        (
            NativeExecutionLaneReport::Unavailable { .. },
            _,
            NativeEffectForeignExecutableStatus::Unsupported { reason },
        ) => NativeExecutableEvidenceStatus::Unsupported {
            reason: reason.clone(),
        },
        (
            NativeExecutionLaneReport::Unavailable { reason, .. },
            _,
            NativeEffectForeignExecutableStatus::NativeTested
            | NativeEffectForeignExecutableStatus::RepresentedUnavailable { .. },
        ) => NativeExecutableEvidenceStatus::Unavailable {
            reason: reason.clone(),
        },
    }
}

fn native_execution_evidence_source(native: &NativeExecutionLaneReport) -> String {
    match native {
        NativeExecutionLaneReport::Available(observation) => observation.evidence_source.clone(),
        NativeExecutionLaneReport::Unavailable {
            evidence_source, ..
        } => evidence_source.clone(),
    }
}

fn comparison_evidence_status(
    report: &NativeComparisonLaneReport,
) -> NativeExecutableEvidenceStatus {
    match report {
        NativeComparisonLaneReport::TestedAgreement { .. } => {
            NativeExecutableEvidenceStatus::Tested
        }
        NativeComparisonLaneReport::Mismatch { diagnostic, .. } => {
            NativeExecutableEvidenceStatus::Unavailable {
                reason: format!(
                    "comparison lane produced a mismatch: {}",
                    diagnostic.message
                ),
            }
        }
        NativeComparisonLaneReport::Unavailable { reason, .. } => {
            NativeExecutableEvidenceStatus::Unavailable {
                reason: reason.clone(),
            }
        }
    }
}

fn comparison_evidence_source(report: &NativeComparisonLaneReport) -> String {
    match report {
        NativeComparisonLaneReport::TestedAgreement {
            evidence_source, ..
        }
        | NativeComparisonLaneReport::Unavailable {
            evidence_source, ..
        } => evidence_source.clone(),
        NativeComparisonLaneReport::Mismatch { diagnostic, .. } => diagnostic.message.clone(),
    }
}

fn effect_policy_evidence_status(
    report: &NativeEffectForeignExecutablePolicyReport,
) -> NativeExecutableEvidenceStatus {
    match &report.status {
        NativeEffectForeignExecutableStatus::NativeTested => NativeExecutableEvidenceStatus::Tested,
        NativeEffectForeignExecutableStatus::RepresentedUnavailable { reason } => {
            NativeExecutableEvidenceStatus::Unavailable {
                reason: reason.clone(),
            }
        }
        NativeEffectForeignExecutableStatus::Unsupported { reason } => {
            NativeExecutableEvidenceStatus::Unsupported {
                reason: reason.clone(),
            }
        }
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
                            RuntimeExpr::Value(RuntimeValue::Int((value - 1).into())),
                            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
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
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((value).into())),
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
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleRead,
                capability: None,
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((1).into()))],
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

    fn trust_lane_status<'a>(
        report: &'a NativeExecutionDifferentialReport,
        claim: &NativeExecutableEvidenceClaim,
    ) -> &'a NativeExecutableEvidenceStatus {
        report
            .trust
            .evidence_lanes
            .iter()
            .find(|lane| &lane.claim == claim)
            .map(|lane| &lane.status)
            .expect("trust lane is present")
    }

    fn closeout_claim_status<'a>(
        closeout: &'a NativeExecutablePhaseCloseoutReport,
        target_symbol: &RuntimeSymbol,
        claim: &NativeExecutableEvidenceClaim,
    ) -> &'a NativeExecutablePhaseStatus {
        closeout
            .claim_inventory
            .iter()
            .find(|entry| entry.target_symbol == *target_symbol && &entry.claim == claim)
            .map(|entry| &entry.status)
            .expect("closeout claim inventory entry is present")
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
        assert_eq!(
            report.header.version,
            NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION
        );
        assert_eq!(
            report.header.spec_ref,
            NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF
        );
        assert_eq!(report.target.package_identity, program.package_identity);
        assert_eq!(
            report.target.executable_artifact_hash,
            package.executable_artifact.artifact_hash
        );
        assert_eq!(
            report.trust.provenance.checked_core.package_identity,
            program.package_identity
        );
        assert_eq!(
            report.trust.provenance.checked_core.core_semantic_hash,
            program.core_semantic_hash
        );
        assert_eq!(
            report.trust.provenance.checked_core.artifact_hash,
            program.artifact_hash
        );
        assert_eq!(
            report.trust.provenance.runtime_ir.runtime_artifact,
            RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(
            report.trust.provenance.runtime_ir.runtime_report_hash,
            package.runtime_report_hash
        );
        assert_eq!(
            report
                .trust
                .provenance
                .object_linker
                .object_linker_package_hash,
            package.header.package_hash
        );
        assert_eq!(
            report.trust.provenance.object_linker.object_artifact_hash,
            package.object_artifact.artifact_hash
        );
        assert_eq!(
            report
                .trust
                .provenance
                .object_linker
                .executable_artifact_hash,
            package.executable_artifact.artifact_hash
        );
        assert!(matches!(
            report.trust.provenance.toolchain.host_platform,
            NativeRecordedFact::Available {
                lane: ObjectLinkerEvidenceLane::Tested,
                ..
            }
        ));
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Available(NativeExecutionObservation {
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
                    crate::RuntimeIntV1::Small(24),
                )),
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
        assert_eq!(
            trust_lane_status(&report, &NativeExecutableEvidenceClaim::NativeExecution),
            &NativeExecutableEvidenceStatus::Tested
        );
        assert_eq!(
            trust_lane_status(
                &report,
                &NativeExecutableEvidenceClaim::RuntimeIrDifferential
            ),
            &NativeExecutableEvidenceStatus::Tested
        );
        assert_eq!(
            trust_lane_status(
                &report,
                &NativeExecutableEvidenceClaim::InterpreterDifferential
            ),
            &NativeExecutableEvidenceStatus::Tested
        );
        assert_eq!(
            trust_lane_status(
                &report,
                &NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy
            ),
            &NativeExecutableEvidenceStatus::Tested
        );
        assert!(matches!(
            trust_lane_status(
                &report,
                &NativeExecutableEvidenceClaim::TranslationValidation
            ),
            NativeExecutableEvidenceStatus::Unavailable { reason }
                if reason.contains("no translation-validation producing check")
        ));
        assert!(matches!(
            trust_lane_status(&report, &NativeExecutableEvidenceClaim::WholeCompilerProof),
            NativeExecutableEvidenceStatus::Unavailable { reason }
                if reason.contains("not Ken proof")
        ));
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
    fn recomputed_hash_with_mismatched_host_toolchain_rejects_before_report() {
        let program = starter_program(46);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc26-mismatched-host-toolchain");
        let mut package = package_for(&program, &run_report, &output_dir);
        match &mut package.toolchain.host_platform {
            ObjectLinkerEvidenceFact::Available { value, .. } => {
                *value = "forged-host-platform".to_string();
            }
            ObjectLinkerEvidenceFact::Unavailable { .. } => unreachable!(),
        }
        package.header.package_hash = object_linker_executable_package_hash(&package);

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("mismatched host toolchain fact rejects");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::Provenance);
        assert_eq!(err.field, "toolchain.host_platform");
        assert!(err.reason.contains("does not match"));
    }

    #[test]
    fn recomputed_hash_with_overclaimed_proof_lane_rejects_before_report() {
        let program = starter_program(47);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc26-overclaimed-proof");
        let mut package = package_for(&program, &run_report, &output_dir);
        package.toolchain.whole_compiler_proof = ObjectLinkerEvidenceFact::Available {
            value: "proved native executable".to_string(),
            evidence_source: "forged unit test proof claim".to_string(),
            lane: ObjectLinkerEvidenceLane::Tested,
        };
        package.header.package_hash = object_linker_executable_package_hash(&package);

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("overclaimed proof lane rejects");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::Provenance);
        assert_eq!(err.field, "toolchain.whole_compiler_proof");
        assert!(err.reason.contains("reported available"));
    }

    #[test]
    fn recomputed_hash_with_contradictory_backend_verifier_rejects_before_report() {
        let program = starter_program(48);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc26-contradictory-backend-verifier");
        let mut package = package_for(&program, &run_report, &output_dir);
        match &mut package.toolchain.backend_verifier {
            ObjectLinkerEvidenceFact::Available { value, .. } => {
                *value = "Cranelift verifier passed: false".to_string();
            }
            ObjectLinkerEvidenceFact::Unavailable { .. } => unreachable!(),
        }
        package.header.package_hash = object_linker_executable_package_hash(&package);

        let err = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect_err("contradictory backend verifier fact rejects");

        assert_eq!(err.stage, NativeExecutionDifferentialStage::Provenance);
        assert_eq!(err.field, "toolchain.backend_verifier");
        assert!(err.reason.contains("contradicts"));
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
        assert!(matches!(
            trust_lane_status(&report, &NativeExecutableEvidenceClaim::NativeExecution),
            NativeExecutableEvidenceStatus::Unavailable { reason }
                if reason.contains("foreign")
        ));
        assert!(matches!(
            trust_lane_status(
                &report,
                &NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy
            ),
            NativeExecutableEvidenceStatus::Unavailable { reason }
                if reason.contains("foreign-boundary")
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
        assert!(matches!(
            trust_lane_status(&report, &NativeExecutableEvidenceClaim::NativeExecution),
            NativeExecutableEvidenceStatus::Unsupported { reason }
                if reason.contains("not native-lowerable")
        ));
        assert!(matches!(
            trust_lane_status(
                &report,
                &NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy
            ),
            NativeExecutableEvidenceStatus::Unsupported { reason }
                if reason.contains("not native-lowerable")
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
    fn hidden_deferred_effect_body_reports_named_unavailable_without_native_execution() {
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
        .expect("hidden deferred effect is a represented-unavailable policy report");

        assert!(matches!(
            report.effect_foreign_policy.status,
            NativeEffectForeignExecutableStatus::RepresentedUnavailable { ref reason }
                if reason.contains("ConsoleRead")
        ));
        assert!(report
            .effect_foreign_policy
            .facts
            .contains("runtime_expr.host_op=0101:ConsoleRead"));
        assert!(matches!(
            report.native,
            NativeExecutionLaneReport::Unavailable { .. }
        ));
    }

    #[test]
    fn pinned_effect_body_is_native_tested_by_exact_operation_identity() {
        let mut program = starter_program(43);
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleWrite,
                capability: None,
                args: Vec::new(),
            },
        };
        let report =
            effect_foreign_executable_policy_report(&program, &program.declarations[0].symbol)
                .unwrap();
        assert_eq!(
            report.status,
            NativeEffectForeignExecutableStatus::NativeTested
        );
        assert!(report
            .facts
            .contains("runtime_expr.host_op=0102:ConsoleWrite"));
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
            RuntimeObservation::Returned(RuntimeGroundValue::Int((30).into()));
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
    fn closeout_report_recommends_nc28_for_full_chain_starter_corpus() {
        let program = starter_program(49);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc27-closeout-positive");
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

        let closeout =
            close_native_executable_phase([&report], std::iter::empty(), "nc27 unit test");

        assert_eq!(
            closeout.header.report_kind,
            NATIVE_EXECUTABLE_PHASE_CLOSEOUT_REPORT_KIND
        );
        assert_eq!(closeout.corpus.positive_cases.len(), 1);
        assert!(closeout.corpus.blockers.is_empty());
        let case = &closeout.corpus.positive_cases[0];
        assert_eq!(case.target, report.target);
        assert_eq!(
            case.checked_core.package_identity,
            report.trust.provenance.checked_core.package_identity
        );
        assert_eq!(
            case.runtime_ir.runtime_report_hash,
            report.trust.provenance.runtime_ir.runtime_report_hash
        );
        assert_eq!(
            case.object_linker.executable_artifact_hash,
            report
                .trust
                .provenance
                .object_linker
                .executable_artifact_hash
        );
        assert_eq!(
            case.native_differential_report_hash,
            native_execution_differential_report_hash(&report)
        );
        assert_eq!(case.status, NativeExecutablePhaseStatus::Supported);
        assert!(matches!(
            closeout.recommendation,
            NativeExecutablePhaseRecommendation::ProceedToNc28 { .. }
        ));
        assert!(closeout.claim_inventory.iter().any(|claim| {
            claim.claim == NativeExecutableEvidenceClaim::NativeExecution
                && claim.status == NativeExecutablePhaseStatus::Tested
        }));
        assert!(closeout.exclusions.iter().any(|exclusion| {
            exclusion.exclusion == NativeExecutableCorpusExclusionKind::TrapDecoding
                && matches!(
                    exclusion.status,
                    NativeExecutablePhaseStatus::Unavailable { .. }
                )
        }));
        assert!(closeout.exclusions.iter().any(|exclusion| {
            exclusion.exclusion == NativeExecutableCorpusExclusionKind::TranslationValidation
                && matches!(
                    exclusion.status,
                    NativeExecutablePhaseStatus::Unavailable { .. }
                )
        }));

        let mut changed = report.clone();
        changed.header.producer.push_str(" changed");
        assert_ne!(
            native_execution_differential_report_hash(&report),
            native_execution_differential_report_hash(&changed)
        );
    }

    #[test]
    fn closeout_frames_prerequisite_when_interpreter_lane_is_unavailable() {
        let program = starter_program(50);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc27-closeout-interpreter-unavailable");
        let package = package_for(&program, &run_report, &output_dir);
        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            NativeInterpreterLaneInput::Unavailable {
                reason: "interpreter observation is not available for this fixture".to_string(),
                evidence_source: "unit test unavailable lane".to_string(),
            },
            "native differential unit test",
        )
        .expect("native differential report materializes");

        let closeout =
            close_native_executable_phase([&report], std::iter::empty(), "nc27 unit test");

        assert!(closeout.corpus.positive_cases.is_empty());
        assert_eq!(closeout.corpus.blockers.len(), 1);
        assert!(matches!(
            closeout.corpus.blockers[0].status,
            NativeExecutablePhaseStatus::Unavailable { ref reason }
                if reason.contains("interpreter differential")
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::InterpreterDifferential
            ),
            NativeExecutablePhaseStatus::Unavailable { reason }
                if reason.contains("interpreter observation is not available")
        ));
        assert!(matches!(
            closeout.recommendation,
            NativeExecutablePhaseRecommendation::FramePrerequisiteWp { .. }
        ));
    }

    #[test]
    fn closeout_classifies_interpreter_mismatch_as_failed() {
        let program = starter_program(52);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc27-closeout-interpreter-mismatch");
        let package = package_for(&program, &run_report, &output_dir);
        let mut interpreter = match interpreter_available(&program, &run_report) {
            NativeInterpreterLaneInput::Available(interpreter) => interpreter,
            NativeInterpreterLaneInput::Unavailable { .. } => unreachable!(),
        };
        interpreter.observation =
            RuntimeObservation::Returned(RuntimeGroundValue::Int((53).into()));
        let report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            NativeInterpreterLaneInput::Available(interpreter),
            "native differential unit test",
        )
        .expect("interpreter mismatch report materializes");

        let closeout =
            close_native_executable_phase([&report], std::iter::empty(), "nc27 unit test");

        assert!(closeout.corpus.positive_cases.is_empty());
        assert_eq!(closeout.corpus.blockers.len(), 1);
        assert!(matches!(
            closeout.corpus.blockers[0].status,
            NativeExecutablePhaseStatus::Failed { ref reason }
                if reason.contains("native differential mismatch")
                    && reason.contains(&program.package_identity)
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::NativeExecution
            ),
            NativeExecutablePhaseStatus::Tested
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::InterpreterDifferential
            ),
            NativeExecutablePhaseStatus::Failed { reason }
                if reason.contains("native differential mismatch")
                    && reason.contains(&program.package_identity)
        ));
    }

    #[test]
    fn closeout_classifies_runtime_ir_mismatch_inventory_as_failed() {
        let program = starter_program(54);
        let mut run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc27-closeout-runtime-ir-mismatch");
        let mut package = package_for(&program, &run_report, &output_dir);
        run_report.observation.observation =
            RuntimeObservation::Returned(RuntimeGroundValue::Int((55).into()));
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
        .expect("runtime-IR mismatch report materializes");

        let closeout =
            close_native_executable_phase([&report], std::iter::empty(), "nc27 unit test");

        assert!(closeout.corpus.positive_cases.is_empty());
        assert_eq!(closeout.corpus.blockers.len(), 1);
        assert!(matches!(
            closeout.corpus.blockers[0].status,
            NativeExecutablePhaseStatus::Failed { ref reason }
                if reason.contains("native differential mismatch")
                    && reason.contains(&program.package_identity)
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::NativeExecution
            ),
            NativeExecutablePhaseStatus::Tested
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::RuntimeIrDifferential
            ),
            NativeExecutablePhaseStatus::Failed { reason }
                if reason.contains("native differential mismatch")
                    && reason.contains(&program.package_identity)
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::InterpreterDifferential
            ),
            NativeExecutablePhaseStatus::Unavailable { reason }
                if reason.contains("interpreter lane intentionally unavailable")
        ));
    }

    #[test]
    fn closeout_classifies_deferred_effect_policy_as_unavailable() {
        let base_program = starter_program(53);
        let run_report = runtime_ir_run_report(&base_program);
        let output_dir = temp_output_dir("nc27-closeout-effect-unsupported");
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
        .expect("effect policy unsupported report materializes");

        let closeout =
            close_native_executable_phase([&report], std::iter::empty(), "nc27 unit test");

        assert!(closeout.corpus.positive_cases.is_empty());
        assert_eq!(closeout.corpus.blockers.len(), 1);
        assert!(matches!(
            closeout.corpus.blockers[0].status,
            NativeExecutablePhaseStatus::Unavailable { ref reason }
                if reason.contains("ConsoleRead")
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::EffectForeignExecutablePolicy
            ),
            NativeExecutablePhaseStatus::Unavailable { reason }
                if reason.contains("ConsoleRead")
        ));
    }

    #[test]
    fn closeout_rejects_overclaimed_out_of_phase_proof_lane() {
        let program = starter_program(51);
        let run_report = runtime_ir_run_report(&program);
        let output_dir = temp_output_dir("nc27-closeout-overclaimed-proof");
        let package = package_for(&program, &run_report, &output_dir);
        let mut report = run_native_execution_differential(
            &program,
            &package,
            &run_report,
            &output_dir,
            interpreter_available(&program, &run_report),
            "native differential unit test",
        )
        .expect("native differential report materializes");
        let proof_lane = report
            .trust
            .evidence_lanes
            .iter_mut()
            .find(|lane| lane.claim == NativeExecutableEvidenceClaim::WholeCompilerProof)
            .expect("whole-compiler proof lane is present");
        proof_lane.status = NativeExecutableEvidenceStatus::Proved;

        let closeout =
            close_native_executable_phase([&report], std::iter::empty(), "nc27 unit test");

        assert!(closeout.corpus.positive_cases.is_empty());
        assert_eq!(closeout.corpus.blockers.len(), 1);
        assert!(matches!(
            closeout.corpus.blockers[0].status,
            NativeExecutablePhaseStatus::Failed { ref reason }
                if reason.contains("WholeCompilerProof")
        ));
        assert!(matches!(
            closeout_claim_status(
                &closeout,
                &report.target.target_symbol,
                &NativeExecutableEvidenceClaim::WholeCompilerProof
            ),
            NativeExecutablePhaseStatus::Failed { reason }
                if reason.contains("WholeCompilerProof")
        ));
        assert!(matches!(
            closeout.recommendation,
            NativeExecutablePhaseRecommendation::FramePrerequisiteWp { .. }
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
