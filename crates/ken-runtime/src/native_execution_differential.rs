//! Native execution differential reports for packaged starter executables.
//!
//! NC24 consumes compiler-produced NC23 object/linker packages and compares
//! exact native execution observations against runtime-IR evaluator reports and
//! interpreter observations when that lane is available. The report is tested
//! evidence only: it does not claim translation validation, proof, library ABI,
//! C/Rust interop, or foreign execution support.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    fnv1a_64, object_linker_executable_package_hash, object_linker_runtime_ir_run_report_hash,
    ObjectLinkerArtifactKind, ObjectLinkerExecutablePackage, RuntimeArtifactIdentity,
    RuntimeGroundValue, RuntimeInterpreterObservation, RuntimeIrRunReport, RuntimeIrTargetIdentity,
    RuntimeObservation, RuntimeProgram, RuntimeSymbol, OBJECT_LINKER_PACKAGE_KIND,
    OBJECT_LINKER_PACKAGE_VERSION,
};

pub const NATIVE_EXECUTION_DIFFERENTIAL_REPORT_KIND: &str = "KenNativeExecutionDifferentialReport";
pub const NATIVE_EXECUTION_DIFFERENTIAL_REPORT_VERSION: u32 = 0;
pub const NATIVE_EXECUTION_DIFFERENTIAL_SPEC_REF: &str =
    "docs/program/wp/NC24-native-execution-differential-suite.md";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExecutionDifferentialReport {
    pub header: NativeExecutionDifferentialHeader,
    pub target: NativeExecutionTargetIdentity,
    pub native: NativeExecutionLaneReport,
    pub runtime_ir: NativeComparisonLaneReport,
    pub interpreter: NativeComparisonLaneReport,
    pub verdict: NativeExecutionDifferentialVerdict,
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
    NativeExecution,
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
    let executable_path = validate_executable_artifact(package, artifact_root.as_ref())?;
    let native = run_packaged_executable(&executable_path, &run_report.observation.observation)?;
    let target = NativeExecutionTargetIdentity {
        package_identity: program.package_identity.clone(),
        target_symbol: package.header.target_symbol.clone(),
        runtime_artifact: RuntimeArtifactIdentity::from_program(program),
        runtime_report_hash: package.runtime_report_hash,
        object_linker_package_hash: package.header.package_hash,
        executable_artifact_hash: package.executable_artifact.artifact_hash,
        executable_relative_path: package.executable_artifact.relative_path.clone(),
    };

    let runtime_ir = compare_runtime_ir_lane(
        &target,
        &run_report.observation.observation,
        &native.observation,
    );
    let interpreter = compare_interpreter_lane(
        &target,
        &RuntimeIrTargetIdentity {
            example: target_example.name.clone(),
            checked_core_shape: target_example.checked_core_shape.clone(),
        },
        interpreter,
        &native.observation,
    );
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
) -> NativeComparisonLaneReport {
    match interpreter {
        NativeInterpreterLaneInput::Unavailable {
            reason,
            evidence_source,
        } => NativeComparisonLaneReport::Unavailable {
            lane: NativeDifferentialLane::Interpreter,
            reason,
            evidence_source,
        },
        NativeInterpreterLaneInput::Available(interpreter) => {
            if interpreter.artifact != target.runtime_artifact {
                return NativeComparisonLaneReport::Unavailable {
                    lane: NativeDifferentialLane::Interpreter,
                    reason:
                        "interpreter observation artifact identity does not match RuntimeProgram"
                            .to_string(),
                    evidence_source: interpreter.evidence_source,
                };
            }
            if &interpreter.target != expected_target {
                return NativeComparisonLaneReport::Unavailable {
                    lane: NativeDifferentialLane::Interpreter,
                    reason:
                        "interpreter observation target identity does not match RuntimeIrRunReport"
                            .to_string(),
                    evidence_source: interpreter.evidence_source,
                };
            }
            if interpreter.observation == *native {
                NativeComparisonLaneReport::TestedAgreement {
                    lane: NativeDifferentialLane::Interpreter,
                    expected: interpreter.observation.clone(),
                    observed: native.clone(),
                    evidence_source: interpreter.evidence_source,
                }
            } else {
                NativeComparisonLaneReport::Mismatch {
                    lane: NativeDifferentialLane::Interpreter,
                    expected: interpreter.observation.clone(),
                    observed: native.clone(),
                    diagnostic: mismatch_diagnostic(
                        target,
                        NativeDifferentialLane::Interpreter,
                        interpreter.observation,
                        native.clone(),
                    ),
                }
            }
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
        RuntimeSymbolMetadata, RuntimeValue,
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
