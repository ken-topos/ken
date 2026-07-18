//! NC6 Cranelift backend spike for the NC5 runtime IR seed.
//!
//! This module deliberately keeps the native boundary narrow. Cranelift code
//! returns scalar `i64` values directly and aggregate observations through an
//! opaque token table decoded by this Rust layer. Native addresses, object
//! layout, allocation order, ABI details, and Cranelift internals never become
//! Ken-observable meaning.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::mem;

use cranelift_codegen::ir::{
    types, AbiParam, FuncRef, Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind,
    UserFuncName,
};
use cranelift_codegen::isa::OwnedTargetIsa;
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::verify_function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

use crate::{
    fnv1a_64, proof_erasure_boundary_facts_from_program, proof_erasure_witness_error,
    validate_supported_runtime_artifact_certificate, KenCheckedProofErasureBoundaryReport,
    ProofErasureBoundaryWitnessError, ProofErasureBoundaryWitnessStage, RuntimeArtifactCertificate,
    RuntimeArtifactIdentity, RuntimeArtifactValidationError, RuntimeArtifactValidationReport,
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExample, RuntimeExpr,
    RuntimeGroundValue, RuntimeIrRunReport, RuntimeIrTargetIdentity, RuntimeLowerabilityStatus,
    RuntimeObservation, RuntimePartiality, RuntimePrimitive, RuntimeProgram, RuntimeSymbol,
    RuntimeTrap, RuntimeTrapCode, RuntimeValue,
};

const CRANELIFT_HOST_EFFECT_CONSUMERS_V1: [ken_host::HostOpV1; 13] = [
    ken_host::HostOpV1::ConsoleWrite,
    ken_host::HostOpV1::ConsoleFlush,
    ken_host::HostOpV1::ConsoleIsTerminal,
    ken_host::HostOpV1::FsReadFile,
    ken_host::HostOpV1::FsWriteFile,
    ken_host::HostOpV1::FsChangeMode,
    ken_host::HostOpV1::FsOpen,
    ken_host::HostOpV1::FsHandleMetadata,
    ken_host::HostOpV1::FsReadAt,
    ken_host::HostOpV1::FsWriteAt,
    ken_host::HostOpV1::ResourceRelease,
    ken_host::HostOpV1::BufferAllocate,
    ken_host::HostOpV1::BufferFreeze,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraneliftRunReport {
    pub example: String,
    pub observation: RuntimeObservation,
    pub verifier_passed: bool,
    pub native_returned: Option<i64>,
    pub trust: NativeTrustReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraneliftObjectArtifact {
    pub example: String,
    pub entry_symbol: String,
    pub object_bytes: Vec<u8>,
    pub object_hash: u64,
    pub platform_target: String,
    pub backend_name: String,
    pub verifier_passed: bool,
    pub assumptions: BTreeSet<String>,
    pub unsupported: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeTrustReport {
    pub backend: &'static str,
    pub fidelity: NativeFidelity,
    pub verifier_passed: bool,
    pub artifact_validation: Option<RuntimeArtifactValidationReport>,
    pub ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
    pub toolchain: NativeToolchainReport,
    pub evidence: NativeRunEvidence,
    pub assumptions: BTreeSet<String>,
    pub unsupported: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeToolchainReport {
    pub cranelift: NativeEvidenceFact,
    pub linker: NativeEvidenceFact,
    pub runtime: NativeEvidenceFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeEvidenceFact {
    Available {
        value: String,
        evidence_source: String,
    },
    Unavailable {
        reason: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeRunEvidence {
    pub package_identity: Option<String>,
    pub core_semantic_hash: Option<u64>,
    pub runtime_artifact_hash: Option<u64>,
    pub evidence_sources: BTreeMap<String, String>,
    pub unavailable: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterpreterOracleObservation {
    pub artifact: NativeArtifactIdentity,
    pub observation: RuntimeObservation,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDifferentialReport {
    pub example: String,
    pub artifact: NativeArtifactIdentity,
    pub oracle: InterpreterOracleObservation,
    pub native: Option<CraneliftRunReport>,
    pub verdict: NativeDifferentialVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRuntimeIrComparisonReport {
    pub example: String,
    pub artifact: NativeArtifactIdentity,
    pub runtime_ir: RuntimeIrRunReport,
    pub native: Option<CraneliftRunReport>,
    pub verdict: NativeRuntimeIrComparisonVerdict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeArtifactIdentity {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub runtime_artifact_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeDifferentialVerdict {
    F1InterpreterAgreement {
        stage: NativeDifferentialStage,
    },
    Unsupported {
        stage: NativeDifferentialStage,
        construct: &'static str,
        reason: String,
    },
    Mismatch {
        stage: NativeDifferentialStage,
        interpreter: RuntimeObservation,
        native: RuntimeObservation,
    },
    BackendFailure {
        stage: NativeDifferentialStage,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeRuntimeIrComparisonVerdict {
    RuntimeIrNativeAgreement {
        stage: NativeDifferentialStage,
    },
    Unsupported {
        stage: NativeDifferentialStage,
        construct: &'static str,
        reason: String,
    },
    Mismatch {
        stage: NativeDifferentialStage,
        runtime_ir: RuntimeObservation,
        native: RuntimeObservation,
    },
    BackendFailure {
        stage: NativeDifferentialStage,
        reason: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeDifferentialStage {
    BoundaryPreflight,
    NativeLoweringOrExecution,
    InterpreterNativeCompare,
    RuntimeIrNativeCompare,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeFidelity {
    F0NativeExample,
    F1SeedObservationAgreement,
    F1InterpreterDifferentialAgreement,
    F1RuntimeIrEvaluatorAgreement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CraneliftBackendError {
    Unsupported(UnsupportedLowering),
    Backend(BackendFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidatedNativeRunError {
    Validation(RuntimeArtifactValidationError),
    Backend(CraneliftBackendError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedLowering {
    pub construct: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackendFailure {
    Target(String),
    Verifier(String),
    Module(String),
    NativeResultDecode { token: i64 },
}

impl fmt::Display for CraneliftBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CraneliftBackendError::Unsupported(err) => {
                write!(f, "unsupported runtime-IR lowering: {err}")
            }
            CraneliftBackendError::Backend(err) => write!(f, "Cranelift backend failure: {err}"),
        }
    }
}

impl std::error::Error for CraneliftBackendError {}

impl fmt::Display for ValidatedNativeRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidatedNativeRunError::Validation(err) => {
                write!(f, "runtime artifact validation failed: {err}")
            }
            ValidatedNativeRunError::Backend(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ValidatedNativeRunError {}

impl From<RuntimeArtifactValidationError> for ValidatedNativeRunError {
    fn from(err: RuntimeArtifactValidationError) -> Self {
        ValidatedNativeRunError::Validation(err)
    }
}

impl From<CraneliftBackendError> for ValidatedNativeRunError {
    fn from(err: CraneliftBackendError) -> Self {
        ValidatedNativeRunError::Backend(err)
    }
}

impl fmt::Display for UnsupportedLowering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.construct, self.reason)
    }
}

impl fmt::Display for BackendFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendFailure::Target(msg) => write!(f, "target setup failed: {msg}"),
            BackendFailure::Verifier(msg) => write!(f, "verifier rejected function: {msg}"),
            BackendFailure::Module(msg) => write!(f, "module operation failed: {msg}"),
            BackendFailure::NativeResultDecode { token } => {
                write!(f, "native result token {token} is not in the result table")
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeSeedEnvironment {
    values: BTreeMap<String, RuntimeGroundValue>,
}

impl NativeSeedEnvironment {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn nc5_seed() -> Self {
        let mut values = BTreeMap::new();
        values.insert(
            "decl:fixture::Local::y".to_string(),
            RuntimeGroundValue::Int(2),
        );
        Self { values }
    }

    pub fn insert(&mut self, symbol: impl Into<String>, value: RuntimeGroundValue) {
        self.values.insert(symbol.into(), value);
    }
}

pub fn run_nc6_seed_examples(
    program: &RuntimeProgram,
) -> Result<Vec<CraneliftRunReport>, CraneliftBackendError> {
    reject_program_blockers(program)?;
    let env = NativeSeedEnvironment::nc5_seed();
    program
        .examples
        .iter()
        .map(|example| run_example_with_seed_observation(example, &env))
        .collect()
}

pub fn run_nc8_validated_seed_examples(
    program: &RuntimeProgram,
    certificate: &RuntimeArtifactCertificate,
) -> Result<Vec<CraneliftRunReport>, ValidatedNativeRunError> {
    let validation = validate_supported_runtime_artifact_certificate(program, certificate)?;
    reject_program_blockers(program)?;
    let env = NativeSeedEnvironment::nc5_seed();
    program
        .examples
        .iter()
        .map(|example| {
            let mut report = run_example_native(
                Some(program),
                example,
                &env,
                NativeFidelity::F0NativeExample,
                NativeRunEvidence::from_program(program),
                Some(validation.clone()),
                None,
            )?;
            if report.observation == example.observation {
                report.trust.fidelity = NativeFidelity::F1SeedObservationAgreement;
            }
            Ok(report)
        })
        .collect()
}

pub fn run_example_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
) -> NativeDifferentialReport {
    run_example_with_interpreter_observation_and_reports(program, example, env, oracle, None, None)
}

pub fn run_validated_example_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    certificate: &RuntimeArtifactCertificate,
) -> Result<NativeDifferentialReport, RuntimeArtifactValidationError> {
    let validation = validate_supported_runtime_artifact_certificate(program, certificate)?;
    Ok(run_example_with_interpreter_observation_and_validation(
        program,
        example,
        env,
        oracle,
        Some(validation),
    ))
}

pub fn run_ken_checked_proof_erasure_example_with_interpreter_observation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    proof_erasure_boundary: KenCheckedProofErasureBoundaryReport,
) -> Result<NativeDifferentialReport, ProofErasureBoundaryWitnessError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if proof_erasure_boundary.artifact != artifact {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessIdentity,
            "artifact_identity",
            format!(
                "Ken-checked proof-erasure report identity {:?} does not match RuntimeProgram identity {:?}",
                proof_erasure_boundary.artifact, artifact
            ),
        ));
    }
    let recomputed_facts = proof_erasure_boundary_facts_from_program(program);
    if let Some(lane) =
        proof_erasure_boundary_report_mismatch_lane(&proof_erasure_boundary, &recomputed_facts)
    {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessMismatch,
            lane,
            "Ken-checked proof-erasure report facts do not match the RuntimeProgram lanes",
        ));
    }

    Ok(run_example_with_interpreter_observation_and_reports(
        program,
        example,
        env,
        oracle,
        None,
        Some(proof_erasure_boundary),
    ))
}

pub fn run_runtime_ir_report_with_cranelift(
    program: &RuntimeProgram,
    run_report: RuntimeIrRunReport,
    env: &NativeSeedEnvironment,
) -> NativeRuntimeIrComparisonReport {
    let artifact = NativeArtifactIdentity::from_program(program);
    let example = match runtime_ir_report_example(program, &run_report) {
        Ok(example) => example,
        Err(err) => {
            return runtime_ir_comparison_error_report(
                artifact,
                run_report,
                err,
                NativeDifferentialStage::BoundaryPreflight,
            );
        }
    };

    if let Err(err) = reject_program_blockers(program) {
        return runtime_ir_comparison_error_report(
            artifact,
            run_report,
            err,
            NativeDifferentialStage::BoundaryPreflight,
        );
    }

    match run_example_native(
        Some(program),
        example,
        env,
        NativeFidelity::F0NativeExample,
        NativeRunEvidence::from_program(program),
        None,
        None,
    ) {
        Ok(mut native) => {
            if native.observation == run_report.observation.observation {
                native.trust.fidelity = NativeFidelity::F1RuntimeIrEvaluatorAgreement;
                NativeRuntimeIrComparisonReport {
                    example: example.name.clone(),
                    artifact,
                    runtime_ir: run_report,
                    native: Some(native),
                    verdict: NativeRuntimeIrComparisonVerdict::RuntimeIrNativeAgreement {
                        stage: NativeDifferentialStage::RuntimeIrNativeCompare,
                    },
                }
            } else {
                NativeRuntimeIrComparisonReport {
                    example: example.name.clone(),
                    artifact,
                    verdict: NativeRuntimeIrComparisonVerdict::Mismatch {
                        stage: NativeDifferentialStage::RuntimeIrNativeCompare,
                        runtime_ir: run_report.observation.observation.clone(),
                        native: native.observation.clone(),
                    },
                    runtime_ir: run_report,
                    native: Some(native),
                }
            }
        }
        Err(err) => runtime_ir_comparison_error_report(
            artifact,
            run_report,
            err,
            NativeDifferentialStage::NativeLoweringOrExecution,
        ),
    }
}

pub fn emit_runtime_ir_object_with_cranelift(
    program: &RuntimeProgram,
    run_report: &RuntimeIrRunReport,
    env: &NativeSeedEnvironment,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    let example = runtime_ir_report_example(program, run_report)?;
    reject_program_blockers(program)?;

    let compiled = compile_program_expr_object(program, &example.ir, env, &entry_symbol)?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);

    Ok(CraneliftObjectArtifact {
        example: example.name.clone(),
        entry_symbol,
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

#[cfg(test)]
pub(crate) fn emit_process_entrypoint_object_with_cranelift(
    entrypoint: &RuntimeExpr,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-process-entrypoint")?,
        &entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        None,
        None,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "native-process-entrypoint".to_string(),
        entry_symbol,
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

#[cfg(test)]
fn emit_process_entrypoint_object_with_symbols(
    entrypoint: &RuntimeExpr,
    symbols: &crate::NativeProcessSymbols,
    entry_symbol: &str,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-process-entrypoint")?,
        entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(symbols),
        None,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "native-process-entrypoint".to_string(),
        entry_symbol: entry_symbol.to_string(),
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

pub(crate) fn emit_bound_process_program_object_with_cranelift(
    program: &RuntimeProgram,
    entrypoint: &RuntimeExpr,
    symbols: &crate::NativeProcessSymbols,
    entry_symbol: impl Into<String>,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let entry_symbol = entry_symbol.into();
    reject_program_blockers(program)?;
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-bound-process-entrypoint")?,
        &entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
        None,
        true,
        Some(symbols),
        native_join_plan_for_program(program)?,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "checked-native-program".to_string(),
        entry_symbol,
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift checked process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

fn proof_erasure_boundary_report_mismatch_lane(
    report: &KenCheckedProofErasureBoundaryReport,
    recomputed: &crate::ProofErasureBoundaryFacts,
) -> Option<&'static str> {
    if report.facts.runtime_declaration_targets != recomputed.runtime_declaration_targets {
        return Some("runtime_declaration_targets");
    }
    if report.facts.record_field_statuses != recomputed.record_field_statuses {
        return Some("record_field_statuses");
    }
    if report.facts.checked_core_record_field_statuses
        != recomputed.checked_core_record_field_statuses
    {
        return Some("checked_core_record_field_statuses");
    }
    if report.facts.lowerability != recomputed.lowerability {
        return Some("lowerability");
    }
    if report.facts.unsupported != recomputed.unsupported {
        return Some("unsupported");
    }
    if report.facts.obligations != recomputed.obligations {
        return Some("obligations");
    }
    if report.facts.obligation_metadata != recomputed.obligation_metadata {
        return Some("obligation_metadata");
    }
    if report.facts.assumptions != recomputed.assumptions {
        return Some("assumptions");
    }
    if report.facts.assumption_trust_metadata != recomputed.assumption_trust_metadata {
        return Some("assumption_trust_metadata");
    }
    if report.facts.trusted_base_delta != recomputed.trusted_base_delta {
        return Some("trusted_base_delta");
    }
    None
}

fn run_example_with_interpreter_observation_and_validation(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
) -> NativeDifferentialReport {
    run_example_with_interpreter_observation_and_reports(
        program,
        example,
        env,
        oracle,
        artifact_validation,
        None,
    )
}

fn run_example_with_interpreter_observation_and_reports(
    program: &RuntimeProgram,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    oracle: InterpreterOracleObservation,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
    ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
) -> NativeDifferentialReport {
    let artifact = NativeArtifactIdentity::from_program(program);

    if oracle.artifact != artifact {
        return oracle_identity_mismatch_report(example, artifact, oracle);
    }

    if let Err(err) = reject_program_blockers(program) {
        return differential_error_report(example, artifact, oracle, err, true);
    }

    match run_example_native(
        Some(program),
        example,
        env,
        NativeFidelity::F0NativeExample,
        NativeRunEvidence::from_program(program),
        artifact_validation,
        ken_checked_proof_erasure_boundary,
    ) {
        Ok(mut native) => {
            if native.observation == oracle.observation {
                native.trust.fidelity = NativeFidelity::F1InterpreterDifferentialAgreement;
                NativeDifferentialReport {
                    example: example.name.clone(),
                    artifact,
                    oracle,
                    native: Some(native),
                    verdict: NativeDifferentialVerdict::F1InterpreterAgreement {
                        stage: NativeDifferentialStage::InterpreterNativeCompare,
                    },
                }
            } else {
                NativeDifferentialReport {
                    example: example.name.clone(),
                    artifact,
                    verdict: NativeDifferentialVerdict::Mismatch {
                        stage: NativeDifferentialStage::InterpreterNativeCompare,
                        interpreter: oracle.observation.clone(),
                        native: native.observation.clone(),
                    },
                    oracle,
                    native: Some(native),
                }
            }
        }
        Err(err) => differential_error_report(example, artifact, oracle, err, false),
    }
}

pub fn reject_program_blockers(program: &RuntimeProgram) -> Result<(), CraneliftBackendError> {
    if !program.erased_core.metadata.effects.is_empty() {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries effect metadata outside the NC6 D1 supported subset",
        ));
    }
    if !program.erased_core.metadata.capabilities.is_empty() {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries capability metadata outside the NC6 D1 supported subset",
        ));
    }
    if !program.erased_core.metadata.runtime_checks.is_empty() {
        return Err(unsupported(
            "RuntimeProgram",
            "package carries runtime-check metadata outside the supported native subset",
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
        return Err(unsupported(
            "RuntimeProgram",
            "package carries trust metadata outside the supported native subset",
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
            return Err(unsupported(
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
                unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} is missing runtime lowerability metadata",
                        declaration.symbol
                    ),
                )
            })?;
        if !matches!(lowerability, RuntimeLowerabilityStatus::Supported) {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} has blocking lowerability metadata: {:?}",
                    declaration.symbol, lowerability
                ),
            ));
        }

        if !declaration.metadata.effects.is_empty() {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries effect metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.capabilities.is_empty() {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries capability metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.runtime_checks.is_empty() {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries runtime-check metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }
        if !declaration.metadata.assumptions.is_empty()
            || !declaration.metadata.assumption_trust_metadata.is_empty()
            || !declaration.metadata.trusted_base_delta.is_empty()
        {
            return Err(unsupported(
                "RuntimeProgram",
                format!(
                    "{} carries trust metadata outside the NC6 D1 supported subset",
                    declaration.symbol
                ),
            ));
        }

        if let RuntimeDeclarationKind::EffectBoundary { effects } = &declaration.kind {
            if !effects.is_empty() {
                return Err(unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} declares effects outside the NC6 D1 supported subset",
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
                return Err(unsupported(
                    "RuntimeProgram",
                    format!(
                        "{} carries effects/foreign metadata outside the NC6 D1 subset",
                        declaration.symbol
                    ),
                ));
            }
        }
    }
    Ok(())
}

pub fn run_example_with_seed_observation(
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let mut report = run_example_native(
        None,
        example,
        env,
        NativeFidelity::F0NativeExample,
        NativeRunEvidence::seed_example(),
        None,
        None,
    )?;
    if report.observation == example.observation {
        report.trust.fidelity = NativeFidelity::F1SeedObservationAgreement;
    }
    Ok(report)
}

/// Execute one runtime expression through the tested native process boundary.
///
/// `staged_process_input` is the byte-accurate argv/environment value bound to
/// `RuntimeExpr::Var(0)` for the in-process validation path. Produced process
/// objects instead bind `Var(0)` to their call-scoped borrowed ingress root.
pub(crate) fn run_process_expr_with_cranelift(
    expr: &RuntimeExpr,
    env: &NativeSeedEnvironment,
    staged_process_input: &RuntimeValue,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let compiled = compile_expr_with_declarations_and_process_input(
        expr,
        env,
        BTreeMap::new(),
        Some(staged_process_input),
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let (observation, native_returned) = compiled.run(None)?;
    Ok(CraneliftRunReport {
        example: "native-process-entrypoint".to_string(),
        observation,
        verifier_passed,
        native_returned,
        trust: NativeTrustReport {
            backend: "Cranelift JIT",
            fidelity: NativeFidelity::F0NativeExample,
            verifier_passed,
            artifact_validation: None,
            ken_checked_proof_erasure_boundary: None,
            toolchain: native_toolchain_report(),
            evidence: NativeRunEvidence::seed_example(),
            assumptions,
            unsupported,
        },
    })
}

fn run_example_native(
    program: Option<&RuntimeProgram>,
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    fidelity: NativeFidelity,
    evidence: NativeRunEvidence,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
    ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let compiled = match program {
        Some(program) => compile_program_expr(program, &example.ir, env)?,
        None => compile_expr(&example.ir, env)?,
    };
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let (observation, native_returned) = compiled.run(None)?;
    Ok(CraneliftRunReport {
        example: example.name.clone(),
        observation,
        verifier_passed,
        native_returned,
        trust: NativeTrustReport {
            backend: "Cranelift JIT",
            fidelity,
            verifier_passed,
            artifact_validation,
            ken_checked_proof_erasure_boundary,
            toolchain: native_toolchain_report(),
            evidence,
            assumptions,
            unsupported,
        },
    })
}

fn runtime_ir_report_example<'a>(
    program: &'a RuntimeProgram,
    run_report: &RuntimeIrRunReport,
) -> Result<&'a RuntimeExample, CraneliftBackendError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if run_report.artifact != artifact || run_report.observation.artifact != artifact {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport artifact identity does not match the exact RuntimeProgram",
        ));
    }
    if run_report.observation.target != run_report.target {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport observation target does not match the run target",
        ));
    }
    if run_report.evidence.package_identity != program.package_identity
        || run_report.evidence.core_semantic_hash != program.core_semantic_hash
        || run_report.evidence.runtime_artifact_hash != program.artifact_hash
    {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport evidence identity does not match the exact RuntimeProgram",
        ));
    }
    if run_report.evidence.target_example != run_report.target.example
        || run_report.evidence.checked_core_shape != run_report.target.checked_core_shape
    {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport evidence target does not match the run target",
        ));
    }

    let mut matches = program
        .examples
        .iter()
        .filter(|example| RuntimeIrTargetIdentity::from_example(example) == run_report.target);
    let Some(example) = matches.next() else {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport target is not present in RuntimeProgram.examples",
        ));
    };
    if matches.next().is_some() {
        return Err(unsupported(
            "RuntimeIrRunReport",
            "RuntimeIrRunReport target identity is ambiguous in RuntimeProgram.examples",
        ));
    }
    Ok(example)
}

impl NativeArtifactIdentity {
    fn from_program(program: &RuntimeProgram) -> Self {
        Self {
            package_identity: program.package_identity.clone(),
            core_semantic_hash: program.core_semantic_hash,
            runtime_artifact_hash: program.artifact_hash,
        }
    }
}

impl NativeRunEvidence {
    fn seed_example() -> Self {
        let mut evidence = Self::default();
        evidence.unavailable.insert(
            "package/core/runtime artifact identity unavailable for standalone seed example"
                .to_string(),
        );
        evidence.evidence_sources.insert(
            "backend".to_string(),
            "compiled Cranelift JIT run".to_string(),
        );
        evidence
    }

    fn from_program(program: &RuntimeProgram) -> Self {
        let mut evidence = Self {
            package_identity: Some(program.package_identity.clone()),
            core_semantic_hash: Some(program.core_semantic_hash),
            runtime_artifact_hash: Some(program.artifact_hash),
            evidence_sources: BTreeMap::new(),
            unavailable: BTreeSet::new(),
        };
        evidence.evidence_sources.insert(
            "package_identity".to_string(),
            "RuntimeProgram.package_identity from the exact runtime artifact".to_string(),
        );
        evidence.evidence_sources.insert(
            "core_semantic_hash".to_string(),
            "RuntimeProgram.core_semantic_hash from the exact runtime artifact".to_string(),
        );
        evidence.evidence_sources.insert(
            "runtime_artifact_hash".to_string(),
            "RuntimeProgram.artifact_hash from the exact runtime artifact".to_string(),
        );
        evidence.evidence_sources.insert(
            "backend".to_string(),
            "compiled Cranelift JIT run".to_string(),
        );
        evidence
    }
}

fn native_toolchain_report() -> NativeToolchainReport {
    NativeToolchainReport {
        cranelift: NativeEvidenceFact::Unavailable {
            reason: "Cranelift package/version fact is not captured from the exact run yet"
                .to_string(),
        },
        linker: NativeEvidenceFact::Unavailable {
            reason: "linker/finalizer fact is not captured from the exact run yet".to_string(),
        },
        runtime: NativeEvidenceFact::Available {
            value: format!("ken-runtime {}", env!("CARGO_PKG_VERSION")),
            evidence_source: "compiled ken-runtime crate version embedded by Cargo for this binary"
                .to_string(),
        },
    }
}

fn oracle_identity_mismatch_report(
    example: &RuntimeExample,
    artifact: NativeArtifactIdentity,
    oracle: InterpreterOracleObservation,
) -> NativeDifferentialReport {
    let reason = format!(
        "oracle artifact identity {:?} does not match runtime artifact identity {:?}",
        oracle.artifact, artifact
    );
    NativeDifferentialReport {
        example: example.name.clone(),
        artifact,
        oracle,
        native: None,
        verdict: NativeDifferentialVerdict::Unsupported {
            stage: NativeDifferentialStage::BoundaryPreflight,
            construct: "InterpreterOracleObservation",
            reason,
        },
    }
}

fn differential_error_report(
    example: &RuntimeExample,
    artifact: NativeArtifactIdentity,
    oracle: InterpreterOracleObservation,
    err: CraneliftBackendError,
    preflight: bool,
) -> NativeDifferentialReport {
    let verdict = match err {
        CraneliftBackendError::Unsupported(err) => NativeDifferentialVerdict::Unsupported {
            stage: if preflight {
                NativeDifferentialStage::BoundaryPreflight
            } else {
                NativeDifferentialStage::NativeLoweringOrExecution
            },
            construct: err.construct,
            reason: err.reason,
        },
        CraneliftBackendError::Backend(err) => NativeDifferentialVerdict::BackendFailure {
            stage: NativeDifferentialStage::NativeLoweringOrExecution,
            reason: err.to_string(),
        },
    };
    NativeDifferentialReport {
        example: example.name.clone(),
        artifact,
        oracle,
        native: None,
        verdict,
    }
}

fn runtime_ir_comparison_error_report(
    artifact: NativeArtifactIdentity,
    run_report: RuntimeIrRunReport,
    err: CraneliftBackendError,
    stage: NativeDifferentialStage,
) -> NativeRuntimeIrComparisonReport {
    let example = run_report.target.example.clone();
    let verdict = match err {
        CraneliftBackendError::Unsupported(err) => NativeRuntimeIrComparisonVerdict::Unsupported {
            stage,
            construct: err.construct,
            reason: err.reason,
        },
        CraneliftBackendError::Backend(err) => NativeRuntimeIrComparisonVerdict::BackendFailure {
            stage: NativeDifferentialStage::NativeLoweringOrExecution,
            reason: err.to_string(),
        },
    };
    NativeRuntimeIrComparisonReport {
        example,
        artifact,
        runtime_ir: run_report,
        native: None,
        verdict,
    }
}

struct CompiledModule<M> {
    module: M,
    func_id: FuncId,
    decoder: Option<ResultDecoder>,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    trap: Option<RuntimeTrap>,
    verifier_passed: bool,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
}

type CompiledExpr = CompiledModule<JITModule>;

#[derive(Clone, Copy)]
enum ResultDecoder {
    Int,
    Bool,
    Table,
}

impl CompiledModule<JITModule> {
    fn run(
        mut self,
        process_root: Option<*const std::ffi::c_void>,
    ) -> Result<(RuntimeObservation, Option<i64>), CraneliftBackendError> {
        if let Some(trap) = self.trap {
            return Ok((RuntimeObservation::Trapped(trap), None));
        }

        self.module
            .finalize_definitions()
            .map_err(|err| backend_module(err.to_string()))?;
        let code = self.module.get_finalized_function(self.func_id);
        // Named native-code-execution boundary. This is tested/validated JIT
        // execution, never a proof and never a host-ABI syscall boundary.
        let process_root = process_root.unwrap_or(std::ptr::null());
        let native =
            unsafe { mem::transmute::<_, extern "C" fn(*const std::ffi::c_void) -> i64>(code) };
        let token = native(process_root);
        let decoder = self
            .decoder
            .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?;
        let ground = match decoder {
            ResultDecoder::Int => RuntimeGroundValue::Int(token),
            ResultDecoder::Bool => RuntimeGroundValue::Bool(token != 0),
            ResultDecoder::Table => self
                .result_table
                .get(&token)
                .cloned()
                .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
        };
        Ok((RuntimeObservation::Returned(ground), Some(token)))
    }
}

fn compile_expr(
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations(expr, seed_env, BTreeMap::new())
}

fn compile_program_expr(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations(
        expr,
        seed_env,
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
    )
}

fn native_join_plan_for_program(
    program: &RuntimeProgram,
) -> Result<Option<crate::NativeJoinPlanV1>, CraneliftBackendError> {
    let candidates = program
        .erased_core
        .metadata
        .checked_core
        .metadata
        .values()
        .filter(|bytes| bytes.starts_with(crate::NATIVE_JOIN_PLAN_V1_HEADER))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [bytes] => crate::NativeJoinPlanV1::decode(bytes)
            .map(Some)
            .map_err(|reason| unsupported("NativeJoinPlanV1", reason)),
        _ => Err(unsupported(
            "NativeJoinPlanV1",
            "checked package contains multiple native join plans",
        )),
    }
}

fn compile_expr_with_declarations<'a>(
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_with_declarations_and_process_input(expr, seed_env, declarations, None)
}

fn compile_expr_with_declarations_and_process_input<'a>(
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
) -> Result<CompiledExpr, CraneliftBackendError> {
    compile_expr_into_module(
        new_jit_module()?,
        "ken_nc6_seed",
        Linkage::Local,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        false,
        None,
        None,
    )
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum BoundedNatFixtureObservation {
    OrdinaryCount,
    OrdinaryRemaining,
    ComputationalCount,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundedNatLoweringMutation {
    Exact,
    BrokenDecrement,
    RawScalarPredecessor,
}

/// Exercise the checked-reply mint without involving any resource operation.
/// The fixture deliberately enters through `mint_validated_progress_nat`, so
/// tests cannot manufacture the compact carrier through a second constructor.
#[cfg(test)]
fn run_checked_bounded_nat_fixture(
    count: u64,
    request_start: u64,
    request_length: u64,
    reply_start: u64,
    observation: BoundedNatFixtureObservation,
    mutation: BoundedNatLoweringMutation,
) -> Result<i64, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px8n_checked_bounded_nat", Linkage::Local, &signature)
        .map_err(|error| backend_module(error.to_string()))?;
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = Lowering {
        seed_env: &seed_env,
        declarations: BTreeMap::new(),
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        bounded_nat_mutation: mutation,
    };
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let count = builder.ins().iconst(types::I64, count as i64);
        let request_start = builder.ins().iconst(types::I64, request_start as i64);
        let request_length = builder.ins().iconst(types::I64, request_length as i64);
        let reply_start = builder.ins().iconst(types::I64, reply_start as i64);
        let one = builder.ins().iconst(types::I64, 1);
        let success =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, one, 1);
        let (count, _predecessor, remaining) = Lowering::mint_validated_progress_nat(
            &mut builder,
            success,
            count,
            request_start,
            request_length,
            Some(reply_start),
        );
        let nat = match observation {
            BoundedNatFixtureObservation::OrdinaryCount
            | BoundedNatFixtureObservation::ComputationalCount => count,
            BoundedNatFixtureObservation::OrdinaryRemaining => remaining,
        };
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N exact structural Nat default".to_string(),
        };
        let lowered = match observation {
            BoundedNatFixtureObservation::OrdinaryCount
            | BoundedNatFixtureObservation::OrdinaryRemaining => {
                let cases = vec![
                    crate::RuntimeMatchCase {
                        constructor: compiler.process_symbols.nat_zero.clone(),
                        binders: 0,
                        body: RuntimeExpr::Value(RuntimeValue::Int(10)),
                    },
                    crate::RuntimeMatchCase {
                        constructor: compiler.process_symbols.nat_suc.clone(),
                        binders: 1,
                        body: RuntimeExpr::Match {
                            scrutinee: Box::new(RuntimeExpr::Var(0)),
                            cases: vec![
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_zero.clone(),
                                    binders: 0,
                                    body: RuntimeExpr::Value(RuntimeValue::Int(21)),
                                },
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_suc.clone(),
                                    binders: 1,
                                    body: RuntimeExpr::Value(RuntimeValue::Int(22)),
                                },
                            ],
                            default: default.clone(),
                        },
                    },
                ];
                compiler.lower_bounded_nat_match(&mut builder, nat, false, &cases, &default, &[])?
            }
            BoundedNatFixtureObservation::ComputationalCount => {
                let cases = vec![
                    crate::RuntimeComputationalMatchCase {
                        constructor: compiler.process_symbols.nat_zero.clone(),
                        argument_binders: 0,
                        recursive_positions: Vec::new(),
                        body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
                    },
                    crate::RuntimeComputationalMatchCase {
                        constructor: compiler.process_symbols.nat_suc.clone(),
                        argument_binders: 1,
                        recursive_positions: vec![0],
                        body: RuntimeExpr::Match {
                            scrutinee: Box::new(RuntimeExpr::Var(1)),
                            cases: vec![
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_zero.clone(),
                                    binders: 0,
                                    body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
                                },
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_suc.clone(),
                                    binders: 1,
                                    body: RuntimeExpr::Match {
                                        scrutinee: Box::new(RuntimeExpr::Var(1)),
                                        cases: vec![
                                            crate::RuntimeMatchCase {
                                                constructor: compiler
                                                    .process_symbols
                                                    .bool_false
                                                    .clone(),
                                                binders: 0,
                                                body: RuntimeExpr::Value(RuntimeValue::Bool(true)),
                                            },
                                            crate::RuntimeMatchCase {
                                                constructor: compiler
                                                    .process_symbols
                                                    .bool_true
                                                    .clone(),
                                                binders: 0,
                                                body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
                                            },
                                        ],
                                        default: default.clone(),
                                    },
                                },
                            ],
                            default: default.clone(),
                        },
                    },
                ];
                let frames = [EliminatorFrame::Computational(
                    ComputationalEliminatorFrame {
                        cases: &cases,
                        default: &default,
                        env: &[],
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                        provenance: compiler.mint_recursor_frame_provenance(),
                    },
                )];
                compiler.lower_bounded_nat_computational(&mut builder, nat, false, &frames)?
            }
        };
        let value = compiler.emit_result(&mut builder, lowered)?.0;
        builder.ins().return_(&[value]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&context.func, module.isa())?;
    module
        .define_function(func_id, &mut context)
        .map_err(|error| backend_module(error.to_string()))?;
    let compiled = CompiledModule {
        module,
        func_id,
        decoder: Some(ResultDecoder::Int),
        result_table: compiler.result_table,
        trap: None,
        verifier_passed: true,
        assumptions: compiler.assumptions,
        unsupported: compiler.unsupported,
    };
    compiled
        .run(None)
        .map(|(_, value)| value.expect("PX8-N fixture returns one scalar"))
}

#[cfg(test)]
fn run_dynamic_constructor_dispatch_fixture(
    discriminator: i64,
    selected_tags: &[i64],
) -> Result<i64, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px7p_dynamic_dispatch", Linkage::Local, &signature)
        .map_err(|error| backend_module(error.to_string()))?;
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = Lowering {
        seed_env: &seed_env,
        declarations: BTreeMap::new(),
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let dynamic = DynamicConstructorV1 {
            discriminator: builder.ins().iconst(types::I64, discriminator),
            alternatives: vec![
                DynamicConstructorAlternativeV1 {
                    tag: 0,
                    constructor: "ctor:fixture::Dynamic::Zero".to_string(),
                    fields: Vec::new(),
                },
                DynamicConstructorAlternativeV1 {
                    tag: 1,
                    constructor: "ctor:fixture::Dynamic::One".to_string(),
                    fields: vec![Lowered::Int {
                        value: builder.ins().iconst(types::I64, 7),
                        known: Some(7),
                    }],
                },
            ],
        };
        let cases = [
            (0, "ctor:fixture::Dynamic::Zero", 0, 40),
            (1, "ctor:fixture::Dynamic::One", 1, 41),
        ]
        .into_iter()
        .filter(|(tag, ..)| selected_tags.contains(tag))
        .map(
            |(_, constructor, binders, result)| crate::RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders,
                body: RuntimeExpr::Value(RuntimeValue::Int(result)),
            },
        )
        .collect::<Vec<_>>();
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7p exact dynamic source default".to_string(),
        };
        let lowered = compiler.lower_dynamic_constructor_match(
            &mut builder,
            dynamic,
            DynamicConstructorContinuation::Ordinary {
                cases: &cases,
                default: &default,
                env: &[],
            },
        )?;
        let value = match lowered {
            Lowered::Trap(trap) => {
                assert_eq!(trap, default);
                builder.ins().iconst(types::I64, -4)
            }
            value => compiler.emit_result(&mut builder, value)?.0,
        };
        builder.ins().return_(&[value]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&context.func, module.isa())?;
    module
        .define_function(func_id, &mut context)
        .map_err(|error| backend_module(error.to_string()))?;
    let compiled = CompiledModule {
        module,
        func_id,
        decoder: Some(ResultDecoder::Int),
        result_table: compiler.result_table,
        trap: None,
        verifier_passed: true,
        assumptions: compiler.assumptions,
        unsupported: compiler.unsupported,
    };
    compiled
        .run(None)
        .map(|(_, token)| token.expect("fixture returns one scalar"))
}

fn compile_program_expr_object(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    seed_env: &NativeSeedEnvironment,
    entry_symbol: &str,
) -> Result<CompiledModule<ObjectModule>, CraneliftBackendError> {
    compile_expr_into_module(
        new_object_module("ken-runtime-cranelift-object")?,
        entry_symbol,
        Linkage::Export,
        expr,
        seed_env,
        program
            .declarations
            .iter()
            .map(|declaration| (declaration.symbol.as_str(), declaration))
            .collect(),
        None,
        false,
        None,
        native_join_plan_for_program(program)?,
    )
}

fn compile_expr_into_module<'a, M: Module>(
    mut module: M,
    function_name: &str,
    linkage: Linkage,
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function(function_name, linkage, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
    let host_dispatch = if process_mode {
        let mut host_sig = module.make_signature();
        host_sig
            .params
            .push(AbiParam::new(module.target_config().pointer_type()));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig
            .params
            .push(AbiParam::new(module.target_config().pointer_type()));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig.returns.push(AbiParam::new(types::I64));
        Some(
            module
                .declare_function("ken_host_dispatch_v1", Linkage::Import, &host_sig)
                .map_err(|err| backend_module(err.to_string()))?,
        )
    } else {
        None
    };
    let mut ctx = module.make_context();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);
    let host_dispatch = host_dispatch.map(|id| module.declare_func_in_func(id, &mut ctx.func));

    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        declarations,
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        native_join_plan,
        consumed_join_sites: BTreeSet::new(),
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: process_mode,
        process_symbols: process_symbols
            .cloned()
            .unwrap_or_else(crate::NativeProcessSymbols::legacy_prelude),
        host_dispatch,
        invocation_pointer: None,
        #[cfg(test)]
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let (maybe_trap, decoder) = {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        let mut initial_env = Vec::new();
        if process_mode {
            let invocation = builder.block_params(block)[0];
            compiler.invocation_pointer = Some(invocation);
            let pointer_type = builder.func.dfg.value_type(invocation);
            let process_input =
                builder
                    .ins()
                    .load(pointer_type, MemFlags::trusted(), invocation, 0);
            Lowering::require_nonzero(&mut builder, process_input);
            let capability = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), invocation, 16);
            initial_env.push(Lowered::BorrowedNativeValue {
                pointer: process_input,
            });
            initial_env.push(Lowered::CapabilityToken { value: capability });
        }
        if let Some(value) = staged_process_input {
            initial_env.push(compiler.lower_value(&mut builder, value)?);
        }
        let lowered = compiler.lower_expr(&mut builder, expr, &initial_env)?;
        let result = match lowered {
            Lowered::Trap(trap) => {
                let status = builder
                    .ins()
                    .iconst(types::I64, if process_mode { -4 } else { 0 });
                builder.ins().return_(&[status]);
                (Some(trap), None)
            }
            value => {
                let (token, decoder) = compiler.emit_result(&mut builder, value)?;
                builder.ins().return_(&[token]);
                (None, Some(decoder))
            }
        };
        builder.seal_all_blocks();
        builder.finalize();
        result
    };

    verify_cranelift_function(&ctx.func, module.isa())?;
    module
        .define_function(func_id, &mut ctx)
        .map_err(|err| backend_module(err.to_string()))?;

    Ok(CompiledModule {
        module,
        func_id,
        decoder,
        result_table: compiler.result_table,
        trap: maybe_trap,
        verifier_passed: true,
        assumptions: compiler.assumptions,
        unsupported: compiler.unsupported,
    })
}

fn native_isa() -> Result<OwnedTargetIsa, CraneliftBackendError> {
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    flag_builder
        .set("is_pic", "true")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa_builder = cranelift_native::builder()
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))
}

fn new_jit_module() -> Result<JITModule, CraneliftBackendError> {
    let isa = native_isa()?;
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    Ok(JITModule::new(builder))
}

fn new_object_module(name: &str) -> Result<ObjectModule, CraneliftBackendError> {
    let isa = native_isa()?;
    let builder = ObjectBuilder::new(isa, name.as_bytes().to_vec(), default_libcall_names())
        .map_err(|err| backend_module(err.to_string()))?;
    Ok(ObjectModule::new(builder))
}

fn native_platform_target_name() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

fn verify_cranelift_function(
    func: &Function,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<(), CraneliftBackendError> {
    verify_function(func, isa).map_err(|err| backend(BackendFailure::Verifier(err.to_string())))
}

struct Lowering<'a> {
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    declaration_stack: Vec<RuntimeSymbol>,
    active_recursive_declarations: Vec<ActiveRecursiveDeclarationV1>,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    next_token: i64,
    next_recursor_frame_provenance: u64,
    next_continuation_activation: u64,
    next_continuation_cursor: u64,
    next_source_join: u64,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    consumed_join_sites: BTreeSet<u64>,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
    process_object: bool,
    process_symbols: crate::NativeProcessSymbols,
    host_dispatch: Option<FuncRef>,
    invocation_pointer: Option<cranelift_codegen::ir::Value>,
    #[cfg(test)]
    bounded_nat_mutation: BoundedNatLoweringMutation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecursorFrameProvenance(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContinuationActivationId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContinuationCursorId(u64);

#[derive(Clone)]
enum Lowered {
    Int {
        value: cranelift_codegen::ir::Value,
        known: Option<i64>,
    },
    Bool {
        value: cranelift_codegen::ir::Value,
        known: Option<bool>,
    },
    ProcessExitStatus {
        value: cranelift_codegen::ir::Value,
    },
    CapabilityToken {
        value: cranelift_codegen::ir::Value,
    },
    ResourceToken {
        value: cranelift_codegen::ir::Value,
    },
    BoundedNat(BoundedNatV1),
    /// A structural `Nat` constructed by checked Ken. Unlike `BoundedNat`,
    /// this value is not a host-reply proof carrier; it is the ordinary unary
    /// constructor representation deforested to one native scalar.
    StructuralNat(StructuralNatV1),
    ResponseBytes {
        pointer: cranelift_codegen::ir::Value,
        len: cranelift_codegen::ir::Value,
    },
    HostResult {
        success: cranelift_codegen::ir::Value,
        error: Box<Lowered>,
        ok: Box<Lowered>,
        err_constructor: String,
        ok_constructor: String,
    },
    DynamicConstructor(DynamicConstructorV1),
    Bytes(Vec<u8>),
    BorrowedNativeValue {
        pointer: cranelift_codegen::ir::Value,
    },
    BorrowedOption {
        present: cranelift_codegen::ir::Value,
        value: cranelift_codegen::ir::Value,
        none: String,
        some: String,
    },
    String(String),
    Constructor {
        constructor: String,
        args: Vec<Lowered>,
    },
    Record {
        fields: Vec<(String, Lowered)>,
    },
    Closure {
        captures: Vec<Lowered>,
        params: Vec<String>,
        body: RuntimeExpr,
    },
    DeclarationClosure {
        symbol: RuntimeSymbol,
        captures: Vec<Lowered>,
        params: Vec<String>,
        body: RuntimeExpr,
    },
    ComputationalRecursorClosure {
        residual: Box<Lowered>,
        activation: ContinuationActivationId,
        invocation: RecursorInvocationSegment,
    },
    /// A tail-recursive edge already emitted as a CFG jump. The current block
    /// is predecessor-free; enclosing scalar combinators propagate this
    /// marker so it cannot be confused with an ordinary or terminal value.
    RecursiveBackedge,
    Trap(RuntimeTrap),
}

#[derive(Clone)]
struct ActiveRecursiveDeclarationV1 {
    symbol: RuntimeSymbol,
    header: Option<cranelift_codegen::ir::Block>,
    argument_templates: Vec<Lowered>,
    induction: Option<Lowered>,
}

#[derive(Clone, Copy)]
struct StructuralNatV1 {
    value: cranelift_codegen::ir::Value,
}

/// Compact private observation of a structural Nat minted from a checked host
/// reply. The scalar never enters Runtime IR or the Ken surface: only the
/// Zero/Suc eliminators below can observe it.
#[derive(Clone, Copy)]
struct BoundedNatV1 {
    value: cranelift_codegen::ir::Value,
}

impl BoundedNatV1 {
    fn mint_after_reply_validation(value: cranelift_codegen::ir::Value) -> Self {
        Self { value }
    }

    fn predecessor(self, builder: &mut FunctionBuilder<'_>) -> Self {
        Self::derived_from_validated(builder.ins().iadd_imm(self.value, -1))
    }

    fn derived_from_validated(value: cranelift_codegen::ir::Value) -> Self {
        Self { value }
    }
}

#[derive(Clone)]
struct DynamicConstructorV1 {
    discriminator: cranelift_codegen::ir::Value,
    alternatives: Vec<DynamicConstructorAlternativeV1>,
}

#[derive(Clone)]
struct DynamicConstructorAlternativeV1 {
    tag: i64,
    constructor: RuntimeSymbol,
    fields: Vec<Lowered>,
}

const MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS: i64 = -3;

fn validate_dynamic_constructor_alternatives<'a>(
    alternatives: impl IntoIterator<Item = (i64, &'a str)>,
) -> Result<(), CraneliftBackendError> {
    let mut tags = BTreeSet::new();
    let mut constructors = BTreeSet::new();
    let mut count = 0;
    for (tag, constructor) in alternatives {
        count += 1;
        if !tags.insert(tag) {
            return Err(unsupported(
                "DynamicConstructor",
                format!("duplicate alternative tag {tag}"),
            ));
        }
        if !constructors.insert(constructor) {
            return Err(unsupported(
                "DynamicConstructor",
                format!("duplicate alternative constructor {constructor}"),
            ));
        }
    }
    if count == 0 {
        return Err(unsupported(
            "DynamicConstructor",
            "closed alternative table is empty",
        ));
    }
    Ok(())
}

fn select_dynamic_constructor_case<'a>(
    cases: &'a [crate::RuntimeMatchCase],
    alternative: &DynamicConstructorAlternativeV1,
    default: &'a RuntimeTrap,
) -> Result<Result<&'a crate::RuntimeMatchCase, &'a RuntimeTrap>, CraneliftBackendError> {
    let mut selected = cases
        .iter()
        .filter(|case| case.constructor == alternative.constructor);
    let Some(case) = selected.next() else {
        return Ok(Err(default));
    };
    if selected.next().is_some() {
        return Err(unsupported(
            "DynamicConstructor",
            format!(
                "source match duplicates constructor {}",
                alternative.constructor
            ),
        ));
    }
    if case.binders != alternative.fields.len() {
        return Err(unsupported(
            "DynamicConstructor",
            format!(
                "case {} expects {} binders but alternative has {} fields",
                case.constructor,
                case.binders,
                alternative.fields.len()
            ),
        ));
    }
    Ok(Ok(case))
}

fn materialize_dynamic_constructor_env(
    alternative: &DynamicConstructorAlternativeV1,
    env: &[Lowered],
) -> Vec<Lowered> {
    let mut arm_env = alternative.fields.clone();
    arm_env.extend_from_slice(env);
    arm_env
}

fn console_stream_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if !args.is_empty() {
        return None;
    }
    if constructor.ends_with("::Stdin") {
        Some(0)
    } else if constructor.ends_with("::Stdout") {
        Some(1)
    } else if constructor.ends_with("::Stderr") {
        Some(2)
    } else {
        None
    }
}

fn create_policy_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if !args.is_empty() {
        return None;
    }
    if constructor.ends_with("::CreateNew") {
        Some(0)
    } else if constructor.ends_with("::CreateOrTruncate") {
        Some(1)
    } else if constructor.ends_with("::CreateOrKeep") {
        Some(2)
    } else {
        None
    }
}

fn resource_open_mode_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if !args.is_empty() {
        return None;
    }
    if constructor.ends_with("::ResourceRead") {
        Some(0)
    } else if constructor.ends_with("::ResourceMetadata") {
        Some(1)
    } else {
        None
    }
}

fn lowered_char_list(value: &Lowered) -> Option<Vec<u8>> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if constructor.ends_with("::Nil") && args.is_empty() {
        return Some(Vec::new());
    }
    if !constructor.ends_with("::Cons") || args.len() != 2 {
        return None;
    }
    let Lowered::Int {
        known: Some(head), ..
    } = &args[0]
    else {
        return None;
    };
    let head = u8::try_from(*head).ok()?;
    let mut tail = lowered_char_list(&args[1])?;
    tail.insert(0, head);
    Some(tail)
}

fn dynamic_host_result_producer_case<'a>(
    cases: &'a [crate::RuntimeMatchCase],
    constructor: &str,
) -> Result<Option<&'a crate::RuntimeMatchCase>, CraneliftBackendError> {
    let Some(case) = cases.iter().find(|case| case.constructor == constructor) else {
        return Ok(None);
    };
    if case.binders != 1 {
        return Err(unsupported(
            "ComputationalMatch",
            format!(
                "dynamic HostResult tree producer case {} expects exactly one binder, got {}",
                case.constructor, case.binders
            ),
        ));
    }
    Ok(Some(case))
}

#[derive(Clone, Copy)]
struct ComputationalEliminatorFrame<'a> {
    cases: &'a [crate::RuntimeComputationalMatchCase],
    default: &'a RuntimeTrap,
    env: &'a [Lowered],
    retained_scrutinee_index: Option<usize>,
    deferred_constructor_case: Option<&'a DeferredConstructorCaseEnvironment<'a>>,
    provenance: RecursorFrameProvenance,
}

#[derive(Clone, Copy)]
struct OrdinaryEliminatorFrame<'a> {
    cases: &'a [crate::RuntimeMatchCase],
    default: &'a RuntimeTrap,
    env: &'a [Lowered],
    retained_scrutinee_index: Option<usize>,
    deferred_constructor_case: Option<&'a DeferredConstructorCaseEnvironment<'a>>,
}

#[derive(Clone, Copy)]
struct PendingLetContinuationFrame<'a> {
    residual: &'a Lowered,
    args: &'a [RuntimeExpr],
    env: &'a [Lowered],
}

#[derive(Clone, Copy)]
struct ActiveContinuationFrame<'a> {
    activation: ContinuationActivationId,
    cursor: ContinuationCursorId,
    parent: Option<&'a ActiveContinuationFrame<'a>>,
    pending: &'a [EliminatorFrame<'a>],
    selected_ancestry: &'a [RecursorFrameProvenance],
}

#[derive(Clone)]
struct ComputationalRecursorLayer {
    cases: Vec<crate::RuntimeComputationalMatchCase>,
    default: RuntimeTrap,
    outer_env: Vec<Lowered>,
    provenance: RecursorFrameProvenance,
}

#[derive(Clone)]
struct RecursorInvocationSegment {
    owned_layers: Vec<ComputationalRecursorLayer>,
    resume_cursor: ContinuationCursorId,
}

impl RecursorInvocationSegment {
    fn new(
        owned_layers: Vec<ComputationalRecursorLayer>,
        resume_cursor: ContinuationCursorId,
    ) -> Self {
        assert!(
            !owned_layers.is_empty(),
            "recursor invocation segment owns at least one layer"
        );
        Self {
            owned_layers,
            resume_cursor,
        }
    }
}

fn decompose_computational_recursor(
    value: Lowered,
) -> (
    Lowered,
    Option<(ContinuationActivationId, RecursorInvocationSegment)>,
) {
    match value {
        Lowered::ComputationalRecursorClosure {
            residual,
            activation,
            invocation,
        } => (*residual, Some((activation, invocation))),
        value => (value, None),
    }
}

fn recursor_eliminator_frames(layers: &[ComputationalRecursorLayer]) -> Vec<EliminatorFrame<'_>> {
    layers
        .iter()
        .map(|layer| {
            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                cases: &layer.cases,
                default: &layer.default,
                env: &layer.outer_env,
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: layer.provenance,
            })
        })
        .collect()
}

fn active_recursor_frame<'a>(
    eliminators: &'a [EliminatorFrame<'a>],
) -> Option<&'a ActiveContinuationFrame<'a>> {
    eliminators.iter().find_map(|eliminator| match eliminator {
        EliminatorFrame::Active(frame) => Some(frame),
        EliminatorFrame::Computational(_)
        | EliminatorFrame::Ordinary(_)
        | EliminatorFrame::PendingLet(_)
        | EliminatorFrame::InvocationReturn => None,
    })
}

fn find_continuation_cursor<'a>(
    active: &'a ActiveContinuationFrame<'a>,
    cursor: ContinuationCursorId,
) -> Option<&'a ActiveContinuationFrame<'a>> {
    if active.cursor == cursor {
        Some(active)
    } else {
        active
            .parent
            .and_then(|parent| find_continuation_cursor(parent, cursor))
    }
}

#[derive(Clone, Copy)]
enum EliminatorFrame<'a> {
    Computational(ComputationalEliminatorFrame<'a>),
    Ordinary(OrdinaryEliminatorFrame<'a>),
    PendingLet(PendingLetContinuationFrame<'a>),
    InvocationReturn,
    Active(ActiveContinuationFrame<'a>),
}

/// The source-evaluation continuation above a recursive-IH invocation.  This
/// is deliberately distinct from `EliminatorFrame`: source evaluation drains
/// this owned chain before its terminal may resume the outer eliminator cursor.
enum SourceContinuation<'a> {
    Terminal(SourceContinuationTerminal<'a>),
    LetBody {
        body: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ApplyRecursorLayers {
        remaining: Vec<ComputationalRecursorLayer>,
        outer: ActiveContinuationFrame<'a>,
        next: Box<SourceContinuation<'a>>,
    },
    IfScrutinee {
        then_expr: RuntimeExpr,
        else_expr: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    MatchScrutinee {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ComputationalMatchScrutinee {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        outer: ActiveContinuationFrame<'a>,
        next: Box<SourceContinuation<'a>>,
    },
    ProjectRecord {
        field: String,
        next: Box<SourceContinuation<'a>>,
    },
    CallCallee {
        args: Vec<RuntimeExpr>,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    CallArgument {
        callee: Lowered,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
}

enum SourceContinuationTerminal<'a> {
    ReturnValue,
    ResumeOuter {
        expected: ContinuationCursorId,
        active: &'a ActiveContinuationFrame<'a>,
    },
    JumpToJoin(SourceJoinEdge<'a>),
}

struct SourceJoinEdge<'a> {
    join_id: u64,
    predecessor_id: u32,
    target: cranelift_codegen::ir::Block,
    expected_outer: ContinuationCursorId,
    required_kind: ScalarMergeKind,
    prefix: Vec<EliminatorFrame<'a>>,
}

struct ArmedInvocation<'a> {
    suspended: SourceContinuation<'a>,
    expected_outer: ContinuationCursorId,
}

enum SourceMachineState<'a> {
    Eval {
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        continuation: SourceContinuation<'a>,
    },
    Value {
        value: Lowered,
        continuation: SourceContinuation<'a>,
    },
}

#[derive(Clone, Copy)]
enum DynamicConstructorContinuation<'a> {
    Ordinary {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
        env: &'a [Lowered],
    },
    Producer {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
        env: &'a [Lowered],
        eliminators: &'a [EliminatorFrame<'a>],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScalarMergeKind {
    Int,
    Bool,
    StructuralNat,
    ExitCode,
    RecursiveBackedge,
}

struct DeferredConstructorCaseEnvironment<'a> {
    constructor: &'a str,
    lowered_prefix: &'a [Lowered],
    selected_field: usize,
    trailing_fields: &'a [RuntimeExpr],
    producer_env: &'a [Lowered],
    outer_eliminator: EliminatorFrame<'a>,
    splice_caller: Option<&'a ActiveContinuationFrame<'a>>,
    selected_active: ActiveContinuationFrame<'a>,
}

#[derive(Clone, Copy)]
enum ImmediateBinderEliminator<'a> {
    Computational {
        cases: &'a [crate::RuntimeComputationalMatchCase],
        default: &'a RuntimeTrap,
    },
    Ordinary {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
    },
}

fn immediate_binder_eliminator(
    body: &RuntimeExpr,
    argument_binder_offset: usize,
    argument_binders: usize,
) -> Option<(usize, ImmediateBinderEliminator<'_>)> {
    let (scrutinee, eliminator) = match body {
        RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default,
        } => (
            scrutinee.as_ref(),
            ImmediateBinderEliminator::Computational { cases, default },
        ),
        RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } => (
            scrutinee.as_ref(),
            ImmediateBinderEliminator::Ordinary { cases, default },
        ),
        _ => return None,
    };
    let RuntimeExpr::Var(index) = scrutinee else {
        return None;
    };
    let index = usize::try_from(*index).ok()?;
    let field = index.checked_sub(argument_binder_offset)?;
    (field < argument_binders).then_some((field, eliminator))
}

fn ordinary_match_continuation<'a>(
    params: &[String],
    body: &'a RuntimeExpr,
) -> Option<(&'a [crate::RuntimeMatchCase], &'a RuntimeTrap)> {
    if params.len() != 1 {
        return None;
    }
    let RuntimeExpr::Match {
        scrutinee,
        cases,
        default,
    } = body
    else {
        return None;
    };
    matches!(scrutinee.as_ref(), RuntimeExpr::Var(0)).then_some((cases, default))
}

fn requires_heterogeneous_deforestation(expr: &RuntimeExpr) -> bool {
    matches!(
        expr,
        RuntimeExpr::Match { .. }
            | RuntimeExpr::ComputationalMatch { .. }
            | RuntimeExpr::If { .. }
            | RuntimeExpr::Call { .. }
    ) && produces_deforestable_aggregate_with_ih(expr, &BTreeSet::new())
}

fn reaches_environment_computational_recursor(
    expr: &RuntimeExpr,
    env: &[Lowered],
    introduced_binders: usize,
) -> bool {
    let recursive_hypotheses = env
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            matches!(value, Lowered::ComputationalRecursorClosure { .. })
                .then_some(index + introduced_binders)
        })
        .collect();
    produces_deforestable_aggregate_with_ih(expr, &recursive_hypotheses)
        && !produces_deforestable_aggregate_with_ih(expr, &BTreeSet::new())
}

fn shifted_aggregate_ihs(aggregate_ihs: &BTreeSet<usize>, by: usize) -> BTreeSet<usize> {
    aggregate_ihs.iter().map(|index| index + by).collect()
}

fn produces_deforestable_aggregate_with_ih(
    expr: &RuntimeExpr,
    aggregate_ihs: &BTreeSet<usize>,
) -> bool {
    match expr {
        RuntimeExpr::Construct { .. } => true,
        RuntimeExpr::Let { body, .. } => {
            produces_deforestable_aggregate_with_ih(body, &shifted_aggregate_ihs(aggregate_ihs, 1))
        }
        RuntimeExpr::Match { cases, .. } => {
            !cases.is_empty()
                && cases.iter().all(|case| {
                    produces_deforestable_aggregate_with_ih(
                        &case.body,
                        &shifted_aggregate_ihs(aggregate_ihs, case.binders),
                    )
                })
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            !cases.is_empty()
                && cases.iter().all(|case| {
                    let mut case_ihs = (0..case.recursive_positions.len()).collect::<BTreeSet<_>>();
                    case_ihs.extend(aggregate_ihs.iter().map(|index| {
                        index + case.recursive_positions.len() + case.argument_binders
                    }));
                    produces_deforestable_aggregate_with_ih(&case.body, &case_ihs)
                })
        }
        RuntimeExpr::If {
            then_expr,
            else_expr,
            ..
        } => {
            produces_deforestable_aggregate_with_ih(then_expr, aggregate_ihs)
                && produces_deforestable_aggregate_with_ih(else_expr, aggregate_ihs)
        }
        RuntimeExpr::Call { callee, .. } => {
            if let RuntimeExpr::Var(index) = callee.as_ref() {
                return usize::try_from(*index).is_ok_and(|index| aggregate_ihs.contains(&index));
            }
            match callee.as_ref() {
                RuntimeExpr::Closure {
                    captures,
                    params,
                    body,
                } => produces_deforestable_aggregate_with_ih(
                    body,
                    &shifted_aggregate_ihs(aggregate_ihs, params.len() + captures.len()),
                ),
                RuntimeExpr::LexicalClosure {
                    captures,
                    params,
                    body,
                } => produces_deforestable_aggregate_with_ih(
                    body,
                    &shifted_aggregate_ihs(aggregate_ihs, params.len() + captures.len()),
                ),
                _ => false,
            }
        }
        _ => false,
    }
}

fn produces_recursive_deforestable_aggregate(expr: &RuntimeExpr, symbol: &str) -> bool {
    match expr {
        RuntimeExpr::Construct { .. } => true,
        RuntimeExpr::Let { body, .. } => produces_recursive_deforestable_aggregate(body, symbol),
        RuntimeExpr::Match { cases, .. } => {
            !cases.is_empty()
                && cases
                    .iter()
                    .all(|case| produces_recursive_deforestable_aggregate(&case.body, symbol))
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            !cases.is_empty()
                && cases
                    .iter()
                    .all(|case| produces_recursive_deforestable_aggregate(&case.body, symbol))
        }
        RuntimeExpr::If {
            then_expr,
            else_expr,
            ..
        } => {
            produces_recursive_deforestable_aggregate(then_expr, symbol)
                && produces_recursive_deforestable_aggregate(else_expr, symbol)
        }
        RuntimeExpr::Call { callee, .. } => {
            matches!(callee.as_ref(), RuntimeExpr::DeclarationRef { symbol: callee } if callee == symbol)
        }
        _ => false,
    }
}

fn collect_runtime_declaration_refs(expr: &RuntimeExpr, output: &mut BTreeSet<RuntimeSymbol>) {
    match expr {
        RuntimeExpr::DeclarationRef { symbol } => {
            output.insert(symbol.clone());
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Let { value, body } => {
            collect_runtime_declaration_refs(value, output);
            collect_runtime_declaration_refs(body, output);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            collect_runtime_declaration_refs(then_expr, output);
            collect_runtime_declaration_refs(else_expr, output);
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            for case in cases {
                collect_runtime_declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            for case in cases {
                collect_runtime_declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, field) in fields {
                collect_runtime_declaration_refs(field, output);
            }
        }
        RuntimeExpr::Project { record, .. }
        | RuntimeExpr::Closure { body: record, .. }
        | RuntimeExpr::LexicalClosure { body: record, .. } => {
            collect_runtime_declaration_refs(record, output);
        }
        RuntimeExpr::Call { callee, args } => {
            collect_runtime_declaration_refs(callee, output);
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_runtime_declaration_refs(&capability.value, output);
            }
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
}

fn select_ordinary_case<'a>(
    eliminator: OrdinaryEliminatorFrame<'a>,
    constructor: &str,
) -> Result<&'a crate::RuntimeMatchCase, RuntimeTrap> {
    eliminator
        .cases
        .iter()
        .find(|case| case.constructor == constructor)
        .ok_or_else(|| eliminator.default.clone())
}

fn select_computational_case<'frames, 'data>(
    eliminators: &'frames [ComputationalEliminatorFrame<'data>],
    constructor: &str,
) -> Result<
    (
        &'data crate::RuntimeComputationalMatchCase,
        &'frames [ComputationalEliminatorFrame<'data>],
    ),
    RuntimeTrap,
> {
    let Some(eliminator) = eliminators.first() else {
        return Err(RuntimeTrap {
            code: RuntimeTrapCode::UnsupportedErasure,
            message: "nested computational producer has no eliminator".to_string(),
        });
    };
    eliminator
        .cases
        .iter()
        .find(|case| case.constructor == constructor)
        .map(|case| (case, &eliminators[1..]))
        .ok_or_else(|| eliminator.default.clone())
}

impl<'a> Lowering<'a> {
    fn mint_recursor_frame_provenance(&mut self) -> RecursorFrameProvenance {
        let provenance = RecursorFrameProvenance(self.next_recursor_frame_provenance);
        self.next_recursor_frame_provenance = self
            .next_recursor_frame_provenance
            .checked_add(1)
            .expect("compiler-private recursor provenance exhausted");
        provenance
    }

    fn mint_continuation_activation(&mut self) -> ContinuationActivationId {
        let activation = ContinuationActivationId(self.next_continuation_activation);
        self.next_continuation_activation = self
            .next_continuation_activation
            .checked_add(1)
            .expect("compiler-private continuation activation exhausted");
        activation
    }

    fn mint_continuation_cursor(&mut self) -> ContinuationCursorId {
        let cursor = ContinuationCursorId(self.next_continuation_cursor);
        self.next_continuation_cursor = self
            .next_continuation_cursor
            .checked_add(1)
            .expect("compiler-private continuation cursor exhausted");
        cursor
    }

    fn make_computational_recursor(
        &self,
        recursive: Lowered,
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        outer_env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        activation: ContinuationActivationId,
        resume_cursor: ContinuationCursorId,
        splice_caller: Option<&ActiveContinuationFrame<'_>>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (residual, payload) = decompose_computational_recursor(recursive);
        let mut owned_layers = if let Some((_, invocation)) = payload {
            let splice_caller = splice_caller.ok_or_else(|| {
                unsupported(
                    "ComputationalRecursor",
                    "recursive payload splice has no active continuation",
                )
            })?;
            if find_continuation_cursor(splice_caller, invocation.resume_cursor).is_none() {
                return Err(unsupported(
                    "ComputationalRecursor",
                    "recursive payload resume cursor is not active",
                ));
            }
            invocation.owned_layers
        } else {
            Vec::new()
        };
        owned_layers.push(ComputationalRecursorLayer {
            cases,
            default,
            outer_env,
            provenance,
        });
        Ok(Lowered::ComputationalRecursorClosure {
            residual: Box::new(residual),
            activation,
            invocation: RecursorInvocationSegment::new(owned_layers, resume_cursor),
        })
    }

    fn resume_active_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
        active: ActiveContinuationFrame<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let Some((head, tail)) = active.pending.split_first() else {
            return Ok(value);
        };
        let cursor = self.mint_continuation_cursor();
        let successor = EliminatorFrame::Active(ActiveContinuationFrame {
            activation: active.activation,
            cursor,
            parent: Some(&active),
            pending: tail,
            selected_ancestry: active.selected_ancestry,
        });
        self.lower_computational_match_value_composed(builder, value, &[*head, successor])
    }

    fn lower_recursor_residual_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        residual: &Lowered,
        args: &[RuntimeExpr],
        argument_env: &[Lowered],
        saved_producer_env: &[Lowered],
        outer_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        if let Lowered::BoundedNat(predecessor) = residual {
            if !args.is_empty() {
                return Err(unsupported(
                    "BoundedNat",
                    "structural Nat recursive hypothesis takes no arguments",
                ));
            }
            return self.lower_bounded_nat_computational(
                builder,
                *predecessor,
                false,
                outer_eliminators,
            );
        }
        let Lowered::Closure {
            captures,
            params,
            body,
        } = residual
        else {
            return Err(unsupported(
                "ComputationalMatch",
                "recursive constructor field is not a closure",
            ));
        };
        let mut call_env = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, argument_env))
            .collect::<Result<Vec<_>, _>>()?;
        if params.len() != call_env.len() {
            return Err(unsupported(
                "ComputationalMatch",
                format!(
                    "recursive field expects {} args but call provides {}",
                    params.len(),
                    call_env.len()
                ),
            ));
        }
        call_env.extend_from_slice(captures);
        call_env.extend_from_slice(saved_producer_env);
        self.lower_computational_producer_expr(builder, body, &call_env, outer_eliminators)
    }

    fn lower_computational_match_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: &RuntimeExpr,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
        producer_env: &[Lowered],
        eliminator_env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let provenance = self.mint_recursor_frame_provenance();
        self.lower_computational_producer_expr(
            builder,
            scrutinee,
            producer_env,
            &[EliminatorFrame::Computational(
                ComputationalEliminatorFrame {
                    cases,
                    default,
                    env: eliminator_env,
                    retained_scrutinee_index: None,
                    deferred_constructor_case: None,
                    provenance,
                },
            )],
        )
    }

    fn lower_computational_producer_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: &RuntimeExpr,
        producer_env: &[Lowered],
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        if eliminators.is_empty() {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        }
        if matches!(eliminators[0], EliminatorFrame::InvocationReturn) {
            return self.lower_expr(builder, scrutinee, producer_env);
        }
        if let EliminatorFrame::PendingLet(continuation) = eliminators[0] {
            let value = self.lower_expr(builder, scrutinee, producer_env)?;
            if matches!(value, Lowered::RecursiveBackedge) {
                return Ok(Lowered::RecursiveBackedge);
            }
            if let Lowered::Trap(trap) = value {
                return Ok(Lowered::Trap(trap));
            }
            let mut continuation_env = vec![value];
            continuation_env.extend_from_slice(continuation.env);
            return self.lower_recursor_residual_call(
                builder,
                continuation.residual,
                continuation.args,
                &continuation_env,
                continuation.env,
                &eliminators[1..],
            );
        }
        if let EliminatorFrame::Active(active) = eliminators[0] {
            if !matches!(
                scrutinee,
                RuntimeExpr::Let { .. }
                    | RuntimeExpr::Call { .. }
                    | RuntimeExpr::Match { .. }
                    | RuntimeExpr::ComputationalMatch { .. }
                    | RuntimeExpr::If { .. }
            ) {
                let value = self.lower_expr(builder, scrutinee, producer_env)?;
                return self.resume_active_continuation(builder, value, active);
            }
        }
        match scrutinee {
            RuntimeExpr::Let { value, body } => {
                if reaches_environment_computational_recursor(body, producer_env, 1) {
                    if let RuntimeExpr::Call { callee, args } = body.as_ref() {
                        if let RuntimeExpr::Var(index) = callee.as_ref() {
                            if let Some(index) = (*index as usize).checked_sub(1) {
                                if let Some(callee @ Lowered::ComputationalRecursorClosure { .. }) =
                                    producer_env.get(index)
                                {
                                    let (residual, boundary) =
                                        decompose_computational_recursor(callee.clone());
                                    let (_, invocation) = boundary.expect(
                                        "recursor closure carries a continuation delimiter",
                                    );
                                    let resume_cursor = invocation.resume_cursor;
                                    let current =
                                        active_recursor_frame(eliminators).ok_or_else(|| {
                                            unsupported(
                                                "ComputationalRecursor",
                                                "recursive invocation has no active continuation",
                                            )
                                        })?;
                                    let _resume = find_continuation_cursor(current, resume_cursor)
                                        .ok_or_else(|| {
                                            unsupported(
                                                "ComputationalRecursor",
                                                "recursive invocation cursor is not active",
                                            )
                                        })?;
                                    let frames =
                                        recursor_eliminator_frames(&invocation.owned_layers);
                                    let mut composed = Vec::with_capacity(frames.len() + 2);
                                    composed.push(EliminatorFrame::PendingLet(
                                        PendingLetContinuationFrame {
                                            residual: &residual,
                                            args,
                                            env: producer_env,
                                        },
                                    ));
                                    composed.extend(frames);
                                    composed.push(EliminatorFrame::InvocationReturn);
                                    let returned = self.lower_computational_producer_expr(
                                        builder,
                                        value,
                                        producer_env,
                                        &composed,
                                    )?;
                                    return self.lower_computational_match_value_composed(
                                        builder,
                                        returned,
                                        eliminators,
                                    );
                                }
                            }
                        }
                    }
                }
                let value = self.lower_expr(builder, value, producer_env)?;
                if let Lowered::Trap(trap) = value {
                    return Ok(Lowered::Trap(trap));
                }
                let mut body_env = vec![value];
                body_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(builder, body, &body_env, eliminators)
            }
            RuntimeExpr::Call { callee, args } => {
                let callee = self.lower_expr(builder, callee, producer_env)?;
                match callee {
                    Lowered::DeclarationClosure {
                        symbol,
                        captures,
                        params,
                        body,
                    } => self.lower_recursive_declaration_call(
                        builder,
                        &symbol,
                        &captures,
                        &params,
                        &body,
                        args,
                        producer_env,
                        Some(eliminators),
                    ),
                    Lowered::Closure {
                        captures,
                        params,
                        body,
                    } => {
                        if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                            if let Some((cases, default)) =
                                ordinary_match_continuation(&params, &body)
                            {
                                let mut frame_env = captures;
                                frame_env.extend_from_slice(producer_env);
                                let mut composed = Vec::with_capacity(eliminators.len() + 1);
                                composed.push(EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                    cases,
                                    default,
                                    env: &frame_env,
                                    retained_scrutinee_index: Some(0),
                                    deferred_constructor_case: None,
                                }));
                                composed.extend_from_slice(eliminators);
                                return self.lower_computational_producer_expr(
                                    builder,
                                    &args[0],
                                    producer_env,
                                    &composed,
                                );
                            }
                        }
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "tree producer expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, producer_env))
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures);
                        call_env.extend_from_slice(producer_env);
                        self.lower_computational_producer_expr(
                            builder,
                            &body,
                            &call_env,
                            eliminators,
                        )
                    }
                    callee @ Lowered::ComputationalRecursorClosure { .. } => {
                        let (base, boundary) = decompose_computational_recursor(callee);
                        let (_, invocation) =
                            boundary.expect("recursor closure carries an invocation segment");
                        let current = active_recursor_frame(eliminators).ok_or_else(|| {
                            unsupported(
                                "ComputationalRecursor",
                                "recursive producer invocation has no active continuation",
                            )
                        })?;
                        let _resume = find_continuation_cursor(current, invocation.resume_cursor)
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "recursive producer invocation cursor is not active",
                                )
                            })?;
                        let mut composed = recursor_eliminator_frames(&invocation.owned_layers);
                        composed.push(EliminatorFrame::InvocationReturn);
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            let returned = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &composed,
                            )?;
                            return self.lower_computational_match_value_composed(
                                builder,
                                returned,
                                eliminators,
                            );
                        }
                        let Lowered::Closure {
                            captures,
                            params,
                            body,
                        } = base
                        else {
                            return Err(unsupported(
                                "ComputationalMatch",
                                "recursive constructor field is not a closure",
                            ));
                        };
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "recursive field expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, producer_env))
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures);
                        call_env.extend_from_slice(producer_env);
                        let returned = self.lower_computational_producer_expr(
                            builder,
                            &body,
                            &call_env,
                            &composed,
                        )?;
                        self.lower_computational_match_value_composed(
                            builder,
                            returned,
                            eliminators,
                        )
                    }
                    _ => Err(unsupported(
                        "ComputationalMatch",
                        "tree producer callee is not a closure",
                    )),
                }
            }
            RuntimeExpr::Construct { constructor, args } => {
                let eliminator = eliminators[0];
                let terminal_exit = constructor == &self.process_symbols.exit_success
                    || constructor == &self.process_symbols.exit_failure;
                let itree_frame = match eliminator {
                    EliminatorFrame::Computational(frame) => frame
                        .cases
                        .iter()
                        .any(|case| case.constructor.contains("::ITree::")),
                    EliminatorFrame::Ordinary(frame) => frame
                        .cases
                        .iter()
                        .any(|case| case.constructor.contains("::ITree::")),
                    EliminatorFrame::PendingLet(_) => {
                        unreachable!("pending Let continuations are consumed before dispatch")
                    }
                    EliminatorFrame::InvocationReturn => {
                        unreachable!("invocation returns are consumed before dispatch")
                    }
                    EliminatorFrame::Active(_) => {
                        unreachable!("active continuation cursors do not consume constructors")
                    }
                };
                if terminal_exit && itree_frame {
                    let lowered_args = args
                        .iter()
                        .map(|arg| self.lower_expr(builder, arg, producer_env))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(Lowered::Constructor {
                        constructor: constructor.clone(),
                        args: lowered_args,
                    });
                }
                let (case_body, argument_binder_offset) = match eliminator {
                    EliminatorFrame::Computational(eliminator) => {
                        let case = match eliminator
                            .cases
                            .iter()
                            .find(|case| case.constructor == *constructor)
                        {
                            Some(case) => case,
                            None => return Ok(Lowered::Trap(eliminator.default.clone())),
                        };
                        if case.argument_binders != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "case {} expects {} constructor arguments but value has {}",
                                    case.constructor,
                                    case.argument_binders,
                                    args.len()
                                ),
                            ));
                        }
                        let mut seen = BTreeSet::new();
                        for position in case.recursive_positions.iter().copied() {
                            if !seen.insert(position) || position >= args.len() {
                                return Err(unsupported(
                                    "ComputationalMatch",
                                    format!(
                                        "case {} has malformed recursive position {position}",
                                        case.constructor
                                    ),
                                ));
                            }
                        }
                        (&case.body, case.recursive_positions.len())
                    }
                    EliminatorFrame::Ordinary(eliminator) => {
                        let case = match select_ordinary_case(eliminator, constructor) {
                            Ok(case) => case,
                            Err(trap) => return Ok(Lowered::Trap(trap)),
                        };
                        if case.binders != args.len() {
                            return Err(unsupported(
                                "Match",
                                format!(
                                    "case {} expects {} binders but constructor has {} args",
                                    case.constructor,
                                    case.binders,
                                    args.len()
                                ),
                            ));
                        }
                        (&case.body, 0)
                    }
                    EliminatorFrame::PendingLet(_) => {
                        unreachable!("pending Let continuations are consumed before dispatch")
                    }
                    EliminatorFrame::InvocationReturn => {
                        unreachable!("invocation returns are consumed before dispatch")
                    }
                    EliminatorFrame::Active(_) => {
                        unreachable!("active continuation cursors do not consume constructors")
                    }
                };

                let bridge =
                    immediate_binder_eliminator(case_body, argument_binder_offset, args.len());
                let bridge =
                    bridge.filter(|(field, _)| requires_heterogeneous_deforestation(&args[*field]));

                if let Some((field, consumer)) = bridge {
                    let lowered_prefix = args[..field]
                        .iter()
                        .map(|arg| self.lower_expr(builder, arg, producer_env))
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(Lowered::Trap(trap)) = lowered_prefix
                        .iter()
                        .find(|value| matches!(value, Lowered::Trap(_)))
                    {
                        return Ok(Lowered::Trap(trap.clone()));
                    }

                    let splice_caller = active_recursor_frame(&eliminators[1..]);
                    let mut selected_ancestry = splice_caller
                        .map(|active| active.selected_ancestry.to_vec())
                        .unwrap_or_default();
                    if let EliminatorFrame::Computational(frame) = eliminator {
                        selected_ancestry.push(frame.provenance);
                    }
                    let mut pending: Vec<_> = eliminators[1..]
                        .iter()
                        .copied()
                        .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                        .collect();
                    if let Some(active) = splice_caller {
                        pending.extend_from_slice(active.pending);
                    }
                    let selected_active = ActiveContinuationFrame {
                        activation: self.mint_continuation_activation(),
                        cursor: self.mint_continuation_cursor(),
                        parent: splice_caller.and_then(|active| active.parent),
                        pending: &pending,
                        selected_ancestry: &selected_ancestry,
                    };
                    let deferred = DeferredConstructorCaseEnvironment {
                        constructor,
                        lowered_prefix: &lowered_prefix,
                        selected_field: field,
                        trailing_fields: &args[field + 1..],
                        producer_env,
                        outer_eliminator: eliminator,
                        splice_caller,
                        selected_active,
                    };
                    let mut composed = Vec::with_capacity(2);
                    composed.push(match consumer {
                        ImmediateBinderEliminator::Computational { cases, default } => {
                            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                                provenance: self.mint_recursor_frame_provenance(),
                            })
                        }
                        ImmediateBinderEliminator::Ordinary { cases, default } => {
                            EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                            })
                        }
                    });
                    composed.push(EliminatorFrame::Active(selected_active));
                    return self.lower_computational_producer_expr(
                        builder,
                        &args[field],
                        producer_env,
                        &composed,
                    );
                }

                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, producer_env))
                    .collect::<Result<Vec<_>, _>>()?;
                self.lower_computational_match_value_composed(
                    builder,
                    Lowered::Constructor {
                        constructor: constructor.clone(),
                        args: lowered_args,
                    },
                    eliminators,
                )
            }
            RuntimeExpr::Match {
                scrutinee,
                cases: producer_cases,
                default: producer_default,
            } => {
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                if let Lowered::Bool { value, known } = selected {
                    let true_case = producer_cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = producer_cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "Bool tree producer requires True and False cases",
                        ));
                    };
                    if let Some(known) = known {
                        return self.lower_computational_producer_expr(
                            builder,
                            if known {
                                &true_case.body
                            } else {
                                &false_case.body
                            },
                            producer_env,
                            eliminators,
                        );
                    }
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder.ins().brif(value, true_block, &[], false_block, &[]);
                    let mut exit_merge = None;
                    for (block, producer_case) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let lowered = self.lower_computational_producer_expr(
                            builder,
                            &producer_case.body,
                            producer_env,
                            eliminators,
                        )?;
                        let (value, is_exit) =
                            self.merge_branch_value(builder, lowered, "ComputationalMatch")?;
                        Self::record_merge_kind("ComputationalMatch", &mut exit_merge, is_exit)?;
                        builder.ins().jump(merge, &[value.into()]);
                    }
                    builder.switch_to_block(merge);
                    let value = builder.block_params(merge)[0];
                    return Ok(if exit_merge == Some(true) {
                        Lowered::ProcessExitStatus { value }
                    } else {
                        Lowered::Int { value, known: None }
                    });
                }
                if let Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                } = selected
                {
                    let ok_block = builder.create_block();
                    let err_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder.ins().brif(success, ok_block, &[], err_block, &[]);
                    let mut exit_merge = None;
                    for (block, constructor, payload) in [
                        (ok_block, ok_constructor.as_str(), *ok),
                        (err_block, err_constructor.as_str(), *error),
                    ] {
                        builder.switch_to_block(block);
                        let lowered = if let Some(producer_case) =
                            dynamic_host_result_producer_case(producer_cases, constructor)?
                        {
                            let mut case_env = vec![payload];
                            case_env.extend_from_slice(producer_env);
                            self.lower_computational_producer_expr(
                                builder,
                                &producer_case.body,
                                &case_env,
                                eliminators,
                            )?
                        } else {
                            Lowered::Trap(producer_default.clone())
                        };
                        let (value, is_exit) =
                            self.merge_branch_value(builder, lowered, "ComputationalMatch")?;
                        Self::record_merge_kind("ComputationalMatch", &mut exit_merge, is_exit)?;
                        builder.ins().jump(merge, &[value.into()]);
                    }
                    builder.switch_to_block(merge);
                    let value = builder.block_params(merge)[0];
                    return Ok(if exit_merge == Some(true) {
                        Lowered::ProcessExitStatus { value }
                    } else {
                        Lowered::Int { value, known: None }
                    });
                }
                if let Lowered::DynamicConstructor(dynamic) = selected {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Producer {
                            cases: producer_cases,
                            default: producer_default,
                            env: producer_env,
                            eliminators,
                        },
                    );
                }
                if let Lowered::BoundedNat(nat) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(builder, nat, false, &composed);
                }
                if let Lowered::StructuralNat(nat) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        &composed,
                    );
                }
                let Lowered::Constructor { constructor, args } = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match scrutinee is not Bool or a constructor",
                    ));
                };
                let Some(producer_case) = producer_cases
                    .iter()
                    .find(|case| case.constructor == constructor)
                else {
                    return Ok(Lowered::Trap(producer_default.clone()));
                };
                if producer_case.binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match constructor arity changed",
                    ));
                }
                let mut case_env = args;
                case_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(
                    builder,
                    &producer_case.body,
                    &case_env,
                    eliminators,
                )
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee: inner_scrutinee,
                cases: inner_cases,
                default: inner_default,
            } => {
                // Fuse the inner eliminator ahead of the outer stack. Its
                // selected case body remains a producer for every outer frame;
                // no intermediate aggregate is materialized or exit-lowered.
                let mut composed = Vec::with_capacity(eliminators.len() + 1);
                let provenance = self.mint_recursor_frame_provenance();
                composed.push(EliminatorFrame::Computational(
                    ComputationalEliminatorFrame {
                        cases: inner_cases,
                        default: inner_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                        provenance,
                    },
                ));
                composed.extend_from_slice(eliminators);
                self.lower_computational_producer_expr(
                    builder,
                    inner_scrutinee,
                    producer_env,
                    &composed,
                )
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                let Lowered::Bool { value, known } = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing If scrutinee is not Bool",
                    ));
                };
                if let Some(known) = known {
                    return self.lower_computational_producer_expr(
                        builder,
                        if known { then_expr } else { else_expr },
                        producer_env,
                        eliminators,
                    );
                }
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                let mut exit_merge = None;
                for (block, branch) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_computational_producer_expr(
                        builder,
                        branch,
                        producer_env,
                        eliminators,
                    )?;
                    let (value, is_exit) =
                        self.merge_branch_value(builder, lowered, "ComputationalMatch")?;
                    Self::record_merge_kind("ComputationalMatch", &mut exit_merge, is_exit)?;
                    builder.ins().jump(merge, &[value.into()]);
                }
                builder.switch_to_block(merge);
                let value = builder.block_params(merge)[0];
                Ok(if exit_merge == Some(true) {
                    Lowered::ProcessExitStatus { value }
                } else {
                    Lowered::Int { value, known: None }
                })
            }
            _ => {
                let value = self.lower_expr(builder, scrutinee, producer_env)?;
                self.lower_computational_match_value_composed(builder, value, eliminators)
            }
        }
    }

    fn lower_computational_match_value_composed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: Lowered,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        let Some(eliminator) = eliminators.first().copied() else {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        };
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(scrutinee);
        }
        if let Lowered::BoundedNat(nat) = scrutinee {
            return self.lower_bounded_nat_computational(builder, nat, false, eliminators);
        }
        if let Lowered::StructuralNat(nat) = scrutinee {
            return self.lower_bounded_nat_computational(
                builder,
                BoundedNatV1::derived_from_validated(nat.value),
                true,
                eliminators,
            );
        }
        let Lowered::Constructor { constructor, args } = scrutinee else {
            return Err(unsupported(
                "ComputationalMatch",
                "scrutinee is not a constructor value after ordinary expression lowering",
            ));
        };
        let retained_scrutinee = Lowered::Constructor {
            constructor: constructor.clone(),
            args: args.clone(),
        };
        let remaining_eliminators = &eliminators[1..];
        let (body, case_env) = match eliminator {
            EliminatorFrame::Computational(eliminator) => {
                let (case, _) = match select_computational_case(
                    std::slice::from_ref(&eliminator),
                    &constructor,
                ) {
                    Ok(selected) => selected,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                if case.argument_binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} expects {} constructor arguments but value has {}",
                            case.constructor,
                            case.argument_binders,
                            args.len()
                        ),
                    ));
                }
                let mut seen = BTreeSet::new();
                for position in case.recursive_positions.iter().copied() {
                    if !seen.insert(position) || position >= args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "case {} has malformed recursive position {position}",
                                case.constructor
                            ),
                        ));
                    }
                }

                let splice_caller = active_recursor_frame(remaining_eliminators);
                let mut selected_ancestry = splice_caller
                    .map(|active| active.selected_ancestry.to_vec())
                    .unwrap_or_default();
                selected_ancestry.push(eliminator.provenance);
                let mut pending: Vec<_> = remaining_eliminators
                    .iter()
                    .copied()
                    .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                    .collect();
                if let Some(active) = splice_caller {
                    pending.extend_from_slice(active.pending);
                }
                let activation = self.mint_continuation_activation();
                let cursor = self.mint_continuation_cursor();
                let active_state = ActiveContinuationFrame {
                    activation,
                    cursor,
                    parent: splice_caller.and_then(|active| active.parent),
                    pending: &pending,
                    selected_ancestry: &selected_ancestry,
                };

                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                for position in case.recursive_positions.iter().rev().copied() {
                    induction_hypotheses.push(self.make_computational_recursor(
                        args[position].clone(),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.provenance,
                        activation,
                        cursor,
                        splice_caller,
                    )?);
                }
                let mut case_env = induction_hypotheses;
                case_env.extend(args);
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Computational(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                case_env.extend(frame_env);
                return self.lower_source_machine(builder, &case.body, &case_env, &active_state);
            }
            EliminatorFrame::Ordinary(eliminator) => {
                let case = match select_ordinary_case(eliminator, &constructor) {
                    Ok(case) => case,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                if case.binders != args.len() {
                    return Err(unsupported(
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
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Ordinary(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                case_env.extend(frame_env);
                (&case.body, case_env)
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before value composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before value composition")
            }
            EliminatorFrame::Active(active) => {
                return self.resume_active_continuation(builder, retained_scrutinee, active);
            }
        };
        if remaining_eliminators.is_empty() {
            self.lower_expr(builder, body, &case_env)
        } else {
            self.lower_computational_producer_expr(builder, body, &case_env, remaining_eliminators)
        }
    }

    fn lower_bounded_nat_computational(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        let eliminator = eliminators[0];
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(if structural {
                Lowered::StructuralNat(StructuralNatV1 { value: nat.value })
            } else {
                Lowered::BoundedNat(nat)
            });
        }
        if let EliminatorFrame::Active(active) = eliminator {
            let value = if structural {
                Lowered::StructuralNat(StructuralNatV1 { value: nat.value })
            } else {
                Lowered::BoundedNat(nat)
            };
            return self.resume_active_continuation(builder, value, active);
        }
        let remaining = &eliminators[1..];
        let (zero_body, suc_body, computational) = match eliminator {
            EliminatorFrame::Computational(frame) => {
                let zero = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_zero
                        && case.argument_binders == 0
                        && case.recursive_positions.is_empty()
                });
                let suc = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_suc
                        && case.argument_binders == 1
                        && case.recursive_positions.as_slice() == [0]
                });
                let (Some(zero), Some(suc)) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "computational Nat requires Zero and one recursive Suc predecessor",
                    ));
                };
                (&zero.body, &suc.body, true)
            }
            EliminatorFrame::Ordinary(frame) => {
                let zero = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_zero && case.binders == 0
                });
                let suc = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_suc && case.binders == 1
                });
                let (Some(zero), Some(suc)) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "ordinary Nat frame requires exact Zero and Suc predecessor arms",
                    ));
                };
                (&zero.body, &suc.body, false)
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before Nat composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before Nat composition")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors do not consume Nat values")
            }
        };

        let zero_value = builder.ins().iconst(types::I64, 0);
        let zero_nat = if structural {
            Lowered::StructuralNat(StructuralNatV1 { value: zero_value })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(zero_value))
        };
        let zero_frame_env =
            match self.materialize_eliminator_frame_env(builder, eliminator, &zero_nat)? {
                Ok(env) => env,
                Err(trap) => return Ok(Lowered::Trap(trap)),
            };
        let zero_lowered = if remaining.is_empty() {
            self.lower_expr(builder, zero_body, &zero_frame_env)?
        } else {
            self.lower_computational_producer_expr(builder, zero_body, &zero_frame_env, remaining)?
        };
        let (initial, result_kind) =
            self.merge_scalar_branch(builder, zero_lowered, "BoundedNat")?;

        let loop_block = builder.create_block();
        let step_block = builder.create_block();
        let done_block = builder.create_block();
        #[cfg(test)]
        let break_decrement =
            self.bounded_nat_mutation == BoundedNatLoweringMutation::BrokenDecrement;
        #[cfg(not(test))]
        let break_decrement = false;
        #[cfg(test)]
        let expose_raw_predecessor =
            self.bounded_nat_mutation == BoundedNatLoweringMutation::RawScalarPredecessor;
        #[cfg(not(test))]
        let expose_raw_predecessor = false;
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        if break_decrement {
            builder.append_block_param(loop_block, types::I64);
        }
        builder.append_block_param(done_block, types::I64);
        if break_decrement {
            builder.ins().jump(
                loop_block,
                &[zero_value.into(), initial.into(), zero_value.into()],
            );
        } else {
            builder
                .ins()
                .jump(loop_block, &[zero_value.into(), initial.into()]);
        }

        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction_value = builder.block_params(loop_block)[1];
        if break_decrement {
            let fuel = builder.block_params(loop_block)[2];
            let compare_block = builder.create_block();
            let exhausted = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
                fuel,
                nat.value,
            );
            let nontermination = builder.ins().iconst(types::I64, -2);
            builder.ins().brif(
                exhausted,
                done_block,
                &[nontermination.into()],
                compare_block,
                &[],
            );
            builder.switch_to_block(compare_block);
        }
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            nat.value,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction_value.into()],
            step_block,
            &[],
        );

        builder.switch_to_block(step_block);
        let successor_value = if break_decrement {
            predecessor_value
        } else {
            builder.ins().iadd_imm(predecessor_value, 1)
        };
        let observed_predecessor = if expose_raw_predecessor {
            nat.value
        } else {
            predecessor_value
        };
        let predecessor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: observed_predecessor,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(observed_predecessor))
        };
        let retained = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: successor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(successor_value))
        };
        let frame_env =
            match self.materialize_eliminator_frame_env(builder, eliminator, &retained)? {
                Ok(env) => env,
                Err(trap) => return Ok(Lowered::Trap(trap)),
            };
        let induction = match result_kind {
            ScalarMergeKind::Int => Lowered::Int {
                value: induction_value,
                known: None,
            },
            ScalarMergeKind::Bool => Lowered::Bool {
                value: induction_value,
                known: None,
            },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 {
                value: induction_value,
            }),
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus {
                value: induction_value,
            },
            ScalarMergeKind::RecursiveBackedge => {
                unreachable!("the computational Nat base case is not a backedge")
            }
        };
        let mut suc_env = Vec::new();
        if computational {
            suc_env.push(induction);
        }
        suc_env.push(predecessor);
        suc_env.extend(frame_env);
        let suc_lowered = if remaining.is_empty() {
            self.lower_expr(builder, suc_body, &suc_env)?
        } else {
            self.lower_computational_producer_expr(builder, suc_body, &suc_env, remaining)?
        };
        let (next, next_kind) = self.merge_scalar_branch(builder, suc_lowered, "BoundedNat")?;
        if next_kind != result_kind {
            return Err(unsupported(
                "BoundedNat",
                "recursive Suc result disagrees with the Zero result kind",
            ));
        }
        if break_decrement {
            let fuel = builder.block_params(loop_block)[2];
            let next_fuel = builder.ins().iadd_imm(fuel, 1);
            builder.ins().jump(
                loop_block,
                &[successor_value.into(), next.into(), next_fuel.into()],
            );
        } else {
            builder
                .ins()
                .jump(loop_block, &[successor_value.into(), next.into()]);
        }

        builder.switch_to_block(done_block);
        let value = builder.block_params(done_block)[0];
        Ok(match result_kind {
            ScalarMergeKind::Int => Lowered::Int { value, known: None },
            ScalarMergeKind::Bool => Lowered::Bool { value, known: None },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 { value }),
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus { value },
            ScalarMergeKind::RecursiveBackedge => {
                unreachable!("the computational Nat base case is not a backedge")
            }
        })
    }

    fn materialize_eliminator_frame_env(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        eliminator: EliminatorFrame<'_>,
        retained_scrutinee: &Lowered,
    ) -> Result<Result<Vec<Lowered>, RuntimeTrap>, CraneliftBackendError> {
        let (env, retained_index, deferred, construct) = match eliminator {
            EliminatorFrame::Computational(frame) => (
                frame.env,
                frame.retained_scrutinee_index,
                frame.deferred_constructor_case,
                "ComputationalMatch",
            ),
            EliminatorFrame::Ordinary(frame) => (
                frame.env,
                frame.retained_scrutinee_index,
                frame.deferred_constructor_case,
                "Match",
            ),
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations do not materialize environments")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns do not materialize environments")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors do not materialize environments")
            }
        };
        let Some(deferred) = deferred else {
            let mut env = env.to_vec();
            if let Some(index) = retained_index {
                if index > env.len() {
                    return Err(unsupported(
                        construct,
                        "retained scrutinee index exceeds the frame environment",
                    ));
                }
                env.insert(index, retained_scrutinee.clone());
            }
            return Ok(Ok(env));
        };
        if deferred.lowered_prefix.len() != deferred.selected_field {
            return Err(unsupported(
                "Construct",
                "selected constructor field prefix does not match its binder index",
            ));
        }

        let mut constructor_args = deferred.lowered_prefix.to_vec();
        constructor_args.push(retained_scrutinee.clone());
        for field in deferred.trailing_fields {
            let lowered = self.lower_expr(builder, field, deferred.producer_env)?;
            if let Lowered::Trap(trap) = lowered {
                return Ok(Err(trap));
            }
            constructor_args.push(lowered);
        }
        let outer_scrutinee = Lowered::Constructor {
            constructor: deferred.constructor.to_string(),
            args: constructor_args.clone(),
        };
        let outer_tail = match self.materialize_eliminator_frame_env(
            builder,
            deferred.outer_eliminator,
            &outer_scrutinee,
        )? {
            Ok(env) => env,
            Err(trap) => return Ok(Err(trap)),
        };

        match deferred.outer_eliminator {
            EliminatorFrame::Computational(frame) => {
                let case = match frame
                    .cases
                    .iter()
                    .find(|case| case.constructor == deferred.constructor)
                {
                    Some(case) => case,
                    None => return Ok(Err(frame.default.clone())),
                };
                if case.argument_binders != constructor_args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} expects {} constructor arguments but value has {}",
                            case.constructor,
                            case.argument_binders,
                            constructor_args.len()
                        ),
                    ));
                }
                let mut seen = BTreeSet::new();
                for position in case.recursive_positions.iter().copied() {
                    if !seen.insert(position) || position >= constructor_args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "case {} has malformed recursive position {position}",
                                case.constructor
                            ),
                        ));
                    }
                }
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                for position in case.recursive_positions.iter().rev().copied() {
                    induction_hypotheses.push(self.make_computational_recursor(
                        constructor_args[position].clone(),
                        frame.cases.to_vec(),
                        frame.default.clone(),
                        outer_tail.clone(),
                        frame.provenance,
                        deferred.selected_active.activation,
                        deferred.selected_active.cursor,
                        deferred.splice_caller,
                    )?);
                }
                induction_hypotheses.extend(constructor_args);
                induction_hypotheses.extend(outer_tail);
                Ok(Ok(induction_hypotheses))
            }
            EliminatorFrame::Ordinary(frame) => {
                let case = match select_ordinary_case(frame, deferred.constructor) {
                    Ok(case) => case,
                    Err(trap) => return Ok(Err(trap)),
                };
                if case.binders != constructor_args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            constructor_args.len()
                        ),
                    ));
                }
                constructor_args.extend(outer_tail);
                Ok(Ok(constructor_args))
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations cannot be deferred constructor frames")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns cannot be deferred constructor frames")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors cannot be deferred constructor frames")
            }
        }
    }

    fn merge_branch_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: Lowered,
        construct: &'static str,
    ) -> Result<(cranelift_codegen::ir::Value, bool), CraneliftBackendError> {
        match lowered {
            Lowered::Int { value, .. } => Ok((value, false)),
            Lowered::ProcessExitStatus { value } => Ok((value, true)),
            lowered if self.process_object => {
                Ok((self.emit_process_exit_status(builder, lowered), true))
            }
            _ => Err(unsupported(
                construct,
                "dynamic native arms must produce scalar Int values",
            )),
        }
    }

    fn merge_scalar_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: Lowered,
        construct: &'static str,
    ) -> Result<(cranelift_codegen::ir::Value, ScalarMergeKind), CraneliftBackendError> {
        match lowered {
            Lowered::RecursiveBackedge => Ok((
                builder.ins().iconst(types::I64, 0),
                ScalarMergeKind::RecursiveBackedge,
            )),
            Lowered::Int { value, .. } => Ok((value, ScalarMergeKind::Int)),
            Lowered::Bool { value, .. } => Ok((value, ScalarMergeKind::Bool)),
            Lowered::StructuralNat(nat) => Ok((nat.value, ScalarMergeKind::StructuralNat)),
            Lowered::Constructor { constructor, args }
                if args.is_empty()
                    && (constructor == self.process_symbols.bool_true
                        || constructor == self.process_symbols.bool_false) =>
            {
                Ok((
                    builder.ins().iconst(
                        types::I64,
                        i64::from(constructor == self.process_symbols.bool_true),
                    ),
                    ScalarMergeKind::Bool,
                ))
            }
            Lowered::ProcessExitStatus { value } => Ok((value, ScalarMergeKind::ExitCode)),
            lowered if self.process_object => Ok((
                self.emit_process_exit_status(builder, lowered),
                ScalarMergeKind::ExitCode,
            )),
            _ => Err(unsupported(
                construct,
                "dynamic arms must produce scalar Int or Bool values",
            )),
        }
    }

    fn record_merge_kind(
        construct: &'static str,
        expected: &mut Option<bool>,
        exit_status: bool,
    ) -> Result<(), CraneliftBackendError> {
        match expected {
            Some(expected) if *expected != exit_status => Err(unsupported(
                construct,
                "dynamic native arms disagree on scalar versus ExitCode result",
            )),
            Some(_) => Ok(()),
            None => {
                *expected = Some(exit_status);
                Ok(())
            }
        }
    }

    fn record_scalar_merge_kind(
        construct: &'static str,
        expected: &mut Option<ScalarMergeKind>,
        kind: ScalarMergeKind,
    ) -> Result<(), CraneliftBackendError> {
        if kind == ScalarMergeKind::RecursiveBackedge {
            return Ok(());
        }
        match expected {
            Some(expected) if *expected != kind => Err(unsupported(
                construct,
                "dynamic native arms disagree on scalar result kind",
            )),
            Some(_) => Ok(()),
            None => {
                *expected = Some(kind);
                Ok(())
            }
        }
    }

    fn planned_join_site_for_frame(
        &mut self,
        frame: EliminatorFrame<'_>,
    ) -> Result<Option<crate::NativeJoinPlanSiteV1>, CraneliftBackendError> {
        let fingerprint = match frame {
            EliminatorFrame::Computational(frame) => {
                crate::compiler_private_computational_match_frame_fingerprint(
                    frame.cases,
                    frame.default,
                )
            }
            EliminatorFrame::Ordinary(frame) => {
                crate::compiler_private_ordinary_match_frame_fingerprint(
                    frame.cases,
                    frame.default,
                )
            }
            EliminatorFrame::InvocationReturn => {
                crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
            }
            EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => return Ok(None),
        };
        let Some(plan) = &self.native_join_plan else {
            return Ok(None);
        };
        let matches = plan
            .sites
            .iter()
            .filter(|site| site.runtime_frame_fingerprint == fingerprint)
            .cloned()
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [site] => {
                self.consumed_join_sites.insert(site.site_id);
                Ok(Some(site.clone()))
            }
            _ => Err(unsupported(
                "NativeJoinPlanV1",
                "equal erased frame shapes make a checked cut occurrence ambiguous",
            )),
        }
    }

    fn scalar_kind_from_plan(kind: crate::NativeJoinAnswerKindV1) -> ScalarMergeKind {
        match kind {
            crate::NativeJoinAnswerKindV1::Int => ScalarMergeKind::Int,
            crate::NativeJoinAnswerKindV1::Bool => ScalarMergeKind::Bool,
            crate::NativeJoinAnswerKindV1::StructuralNat => ScalarMergeKind::StructuralNat,
            crate::NativeJoinAnswerKindV1::ExitCode => ScalarMergeKind::ExitCode,
        }
    }

    fn reconstruct_scalar_join_value(
        kind: ScalarMergeKind,
        value: cranelift_codegen::ir::Value,
    ) -> Lowered {
        match kind {
            ScalarMergeKind::Int => Lowered::Int { value, known: None },
            ScalarMergeKind::Bool => Lowered::Bool { value, known: None },
            ScalarMergeKind::StructuralNat => {
                Lowered::StructuralNat(StructuralNatV1 { value })
            }
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus { value },
            ScalarMergeKind::RecursiveBackedge => {
                unreachable!("a checked join plan never carries RecursiveBackedge")
            }
        }
    }

    fn declaration_call_produces_deforestable_aggregate(&self, expr: &RuntimeExpr) -> bool {
        let RuntimeExpr::Call { callee, .. } = expr else {
            return false;
        };
        let RuntimeExpr::DeclarationRef { symbol } = callee.as_ref() else {
            return false;
        };
        let Some(declaration) = self.declarations.get(symbol.as_str()).copied() else {
            return false;
        };
        let RuntimeDeclarationKind::Transparent {
            body:
                RuntimeExpr::Closure {
                    body: declaration_body,
                    ..
                },
        } = &declaration.kind
        else {
            return false;
        };
        produces_recursive_deforestable_aggregate(declaration_body, symbol)
    }

    fn source_continuation_outer(
        continuation: &SourceContinuation<'_>,
    ) -> Option<ContinuationCursorId> {
        match continuation {
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => None,
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected,
                ..
            }) => Some(*expected),
            SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)) => {
                Some(edge.expected_outer)
            }
            SourceContinuation::LetBody { next, .. }
            | SourceContinuation::ApplyRecursorLayers { next, .. }
            | SourceContinuation::IfScrutinee { next, .. }
            | SourceContinuation::ConstructArgument { next, .. }
            | SourceContinuation::MatchScrutinee { next, .. }
            | SourceContinuation::ComputationalMatchScrutinee { next, .. }
            | SourceContinuation::ProjectRecord { next, .. }
            | SourceContinuation::CallCallee { next, .. }
            | SourceContinuation::CallArgument { next, .. } => {
                Self::source_continuation_outer(next)
            }
        }
    }

    fn source_terminal_active<'b>(
        continuation: &SourceContinuation<'b>,
    ) -> Option<ActiveContinuationFrame<'b>> {
        match continuation {
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                active,
                ..
            }) => Some(**active),
            SourceContinuation::Terminal(
                SourceContinuationTerminal::ReturnValue
                | SourceContinuationTerminal::JumpToJoin(_),
            ) => None,
            SourceContinuation::LetBody { next, .. }
            | SourceContinuation::ApplyRecursorLayers { next, .. }
            | SourceContinuation::IfScrutinee { next, .. }
            | SourceContinuation::ConstructArgument { next, .. }
            | SourceContinuation::MatchScrutinee { next, .. }
            | SourceContinuation::ComputationalMatchScrutinee { next, .. }
            | SourceContinuation::ProjectRecord { next, .. }
            | SourceContinuation::CallCallee { next, .. }
            | SourceContinuation::CallArgument { next, .. } => {
                Self::source_terminal_active(next)
            }
        }
    }

    fn instantiate_source_prefix_to_join<'b>(
        source: &SourceContinuation<'b>,
        edge: SourceJoinEdge<'b>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        Ok(match source {
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected,
                ..
            }) => {
                if *expected != edge.expected_outer {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "source prefix terminal does not match the planned outer cursor",
                    ));
                }
                SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge))
            }
            SourceContinuation::Terminal(
                SourceContinuationTerminal::ReturnValue
                | SourceContinuationTerminal::JumpToJoin(_),
            ) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "source prefix has no exact outer terminal to replace",
                ));
            }
            SourceContinuation::LetBody { body, env, next } => SourceContinuation::LetBody {
                body: body.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
            },
            SourceContinuation::ApplyRecursorLayers {
                remaining,
                outer,
                next,
            } => {
                SourceContinuation::ApplyRecursorLayers {
                    remaining: remaining.clone(),
                    outer: *outer,
                    next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
                }
            }
            SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => SourceContinuation::IfScrutinee {
                then_expr: then_expr.clone(),
                else_expr: else_expr.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
            },
            SourceContinuation::ConstructArgument {
                constructor,
                remaining,
                lowered,
                env,
                next,
            } => SourceContinuation::ConstructArgument {
                constructor: constructor.clone(),
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
            },
            SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
            },
            SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                outer,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                provenance: *provenance,
                outer: *outer,
                next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
            },
            SourceContinuation::ProjectRecord { field, next } => {
                SourceContinuation::ProjectRecord {
                    field: field.clone(),
                    next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
                }
            }
            SourceContinuation::CallCallee { args, env, next } => {
                SourceContinuation::CallCallee {
                    args: args.clone(),
                    env: env.clone(),
                    next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
                }
            }
            SourceContinuation::CallArgument {
                callee,
                remaining,
                lowered,
                env,
                next,
            } => SourceContinuation::CallArgument {
                callee: callee.clone(),
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_to_join(next, edge)?),
            },
        })
    }

    fn planned_active_scalar_cut<'b>(
        &mut self,
        active: ActiveContinuationFrame<'b>,
    ) -> Result<
        (
            Vec<EliminatorFrame<'b>>,
            &'b [EliminatorFrame<'b>],
            ScalarMergeKind,
            u64,
        ),
        CraneliftBackendError,
    > {
        for (index, frame) in active.pending.iter().copied().enumerate() {
            if let Some(site) = self.planned_join_site_for_frame(frame)? {
                let prefix_end = if matches!(frame, EliminatorFrame::InvocationReturn) {
                    index
                } else {
                    index + 1
                };
                return Ok((
                    active.pending[..prefix_end].to_vec(),
                    &active.pending[prefix_end..],
                    Self::scalar_kind_from_plan(site.answer_kind),
                    site.site_id,
                ));
            }
        }
        if active.pending.is_empty() {
            if let Some(site) =
                self.planned_join_site_for_frame(EliminatorFrame::InvocationReturn)?
            {
                return Ok((
                    Vec::new(),
                    active.pending,
                    Self::scalar_kind_from_plan(site.answer_kind),
                    site.site_id,
                ));
            }
        }
        Err(unsupported(
            "NativeJoinPlanV1",
            "active checked continuation has no planned scalar cut before its outer suffix",
        ))
    }

    fn lower_source_machine(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: &RuntimeExpr,
        env: &[Lowered],
        active: &ActiveContinuationFrame<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        self.lower_source_machine_with_continuation(
            builder,
            expr.clone(),
            env.to_vec(),
            SourceContinuation::Terminal(
                SourceContinuationTerminal::ResumeOuter {
                    expected: active.cursor,
                    active,
                },
            ),
        )
    }

    fn lower_source_machine_with_continuation<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        continuation: SourceContinuation<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let mut state = SourceMachineState::Eval {
            expr,
            env,
            continuation,
        };
        loop {
            state = match state {
                SourceMachineState::Eval {
                    expr,
                    env,
                    continuation,
                } => match expr {
                    RuntimeExpr::Value(value) => SourceMachineState::Value {
                        value: self.lower_value(builder, &value)?,
                        continuation,
                    },
                    RuntimeExpr::Var(index) => SourceMachineState::Value {
                        value: env.get(index as usize).cloned().ok_or_else(|| {
                            unsupported("Var", format!("no runtime binding for index {index}"))
                        })?,
                        continuation,
                    },
                    RuntimeExpr::Let { value, body } => SourceMachineState::Eval {
                        expr: *value,
                        env: env.clone(),
                        continuation: SourceContinuation::LetBody {
                            body: *body,
                            env,
                            next: Box::new(continuation),
                        },
                    },
                    RuntimeExpr::Construct { constructor, mut args } => {
                        if args.is_empty() {
                            SourceMachineState::Value {
                                value: self.finish_source_constructor(builder, constructor, vec![])?,
                                continuation,
                            }
                        } else {
                            let first = args.remove(0);
                            SourceMachineState::Eval {
                                expr: first,
                                env: env.clone(),
                                continuation: SourceContinuation::ConstructArgument {
                                    constructor,
                                    remaining: args,
                                    lowered: Vec::new(),
                                    env,
                                    next: Box::new(continuation),
                                },
                            }
                        }
                    }
                    RuntimeExpr::Match {
                        scrutinee,
                        cases,
                        default,
                    } => SourceMachineState::Eval {
                        expr: *scrutinee,
                        env: env.clone(),
                        continuation: SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env,
                            next: Box::new(continuation),
                        },
                    },
                    RuntimeExpr::Call { callee, args } => SourceMachineState::Eval {
                        expr: *callee,
                        env: env.clone(),
                        continuation: SourceContinuation::CallCallee {
                            args,
                            env,
                            next: Box::new(continuation),
                        },
                    },
                    RuntimeExpr::ComputationalMatch {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        let outer = Self::source_terminal_active(&continuation).ok_or_else(|| {
                            unsupported(
                                "ComputationalMatch",
                                "source computational match has no live outer frame",
                            )
                        })?;
                        SourceMachineState::Eval {
                            expr: *scrutinee,
                            env: env.clone(),
                            continuation: SourceContinuation::ComputationalMatchScrutinee {
                                cases,
                                default,
                                env,
                                provenance: self.mint_recursor_frame_provenance(),
                                outer,
                                next: Box::new(continuation),
                            },
                        }
                    }
                    other => SourceMachineState::Value {
                        value: self.lower_expr(builder, &other, &env)?,
                        continuation,
                    },
                },
                SourceMachineState::Value {
                    value,
                    continuation,
                } => match continuation {
                    SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                        return Ok(value);
                    }
                    SourceContinuation::Terminal(
                        SourceContinuationTerminal::ResumeOuter { expected, active },
                    ) => {
                        if active.cursor != expected {
                            return Err(unsupported(
                                "ComputationalRecursor",
                                "source continuation terminal cursor mismatch",
                            ));
                        }
                        return self.resume_active_continuation(builder, value, *active);
                    }
                    SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)) => {
                        let value = if edge.prefix.is_empty() {
                            value
                        } else {
                            let mut prefix = edge.prefix;
                            prefix.push(EliminatorFrame::InvocationReturn);
                            self.lower_computational_match_value_composed(
                                builder,
                                value,
                                &prefix,
                            )?
                        };
                        let (value, actual_kind) =
                            self.merge_scalar_branch(builder, value, "NativeJoinPlanV1")?;
                        if actual_kind != ScalarMergeKind::RecursiveBackedge
                            && actual_kind != edge.required_kind
                        {
                            return Err(unsupported(
                                "NativeJoinPlanV1",
                                format!(
                                    "predecessor {} for join {} produced {actual_kind:?}, planned {:?}",
                                    edge.predecessor_id, edge.join_id, edge.required_kind
                                ),
                            ));
                        }
                        builder.ins().jump(edge.target, &[value.into()]);
                        return Ok(Lowered::RecursiveBackedge);
                    }
                    SourceContinuation::LetBody { body, env, next } => {
                        if matches!(value, Lowered::RecursiveBackedge) {
                            SourceMachineState::Value {
                                value,
                                continuation: *next,
                            }
                        } else if matches!(value, Lowered::Trap(_)) {
                            SourceMachineState::Value {
                                value,
                                continuation: *next,
                            }
                        } else {
                            let mut body_env = vec![value];
                            body_env.extend(env);
                            SourceMachineState::Eval {
                                expr: body,
                                env: body_env,
                                continuation: *next,
                            }
                        }
                    }
                    SourceContinuation::ApplyRecursorLayers {
                        remaining,
                        outer,
                        next,
                    } => {
                        let mut frames = recursor_eliminator_frames(&remaining);
                        frames.push(EliminatorFrame::Active(outer));
                        frames.push(EliminatorFrame::InvocationReturn);
                        SourceMachineState::Value {
                            value: self.lower_computational_match_value_composed(
                                builder,
                                value,
                                &frames,
                            )?,
                            continuation: *next,
                        }
                    }
                    SourceContinuation::ConstructArgument {
                        constructor,
                        mut remaining,
                        mut lowered,
                        env,
                        next,
                    } => {
                        lowered.push(value);
                        if remaining.is_empty() {
                            SourceMachineState::Value {
                                value: self.finish_source_constructor(
                                    builder,
                                    constructor,
                                    lowered,
                                )?,
                                continuation: *next,
                            }
                        } else {
                            let first = remaining.remove(0);
                            SourceMachineState::Eval {
                                expr: first,
                                env: env.clone(),
                                continuation: SourceContinuation::ConstructArgument {
                                    constructor,
                                    remaining,
                                    lowered,
                                    env,
                                    next,
                                },
                            }
                        }
                    }
                    SourceContinuation::MatchScrutinee {
                        cases,
                        default,
                        env,
                        next,
                    } => {
                        if let Lowered::HostResult {
                            success,
                            error,
                            ok,
                            err_constructor,
                            ok_constructor,
                        } = value
                        {
                            return self.lower_source_dynamic_host_result_match(
                                builder,
                                success,
                                *error,
                                *ok,
                                &err_constructor,
                                &ok_constructor,
                                &cases,
                                default,
                                &env,
                                *next,
                            );
                        }
                        if let Lowered::DynamicConstructor(dynamic) = value {
                            return self.lower_source_dynamic_constructor_match(
                                builder,
                                dynamic,
                                &cases,
                                &default,
                                &env,
                                *next,
                            );
                        }
                        let Lowered::Constructor { constructor, args } = value else {
                            return Err(unsupported(
                                "Match",
                                "scrutinee is not a constructor value",
                            ));
                        };
                        let Some(case) = cases.iter().find(|case| case.constructor == constructor)
                        else {
                            return Ok(Lowered::Trap(default));
                        };
                        if case.binders != args.len() {
                            return Err(unsupported(
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
                        case_env.extend(env);
                        SourceMachineState::Eval {
                            expr: case.body.clone(),
                            env: case_env,
                            continuation: *next,
                        }
                    }
                    SourceContinuation::ComputationalMatchScrutinee {
                        cases,
                        default,
                        env,
                        provenance,
                        outer,
                        next,
                    } => {
                        let frame = ComputationalEliminatorFrame {
                            cases: &cases,
                            default: &default,
                            env: &env,
                            retained_scrutinee_index: None,
                            deferred_constructor_case: None,
                            provenance,
                        };
                        SourceMachineState::Value {
                            value: self.lower_computational_match_value_composed(
                                builder,
                                value,
                                &[
                                    EliminatorFrame::Computational(frame),
                                    EliminatorFrame::Active(outer),
                                    EliminatorFrame::InvocationReturn,
                                ],
                            )?,
                            continuation: *next,
                        }
                    }
                    SourceContinuation::CallCallee {
                        mut args,
                        env,
                        next,
                    } => {
                        if args.is_empty() {
                            self.source_call_state(builder, value, Vec::new(), env, *next)?
                        } else {
                            let first = args.remove(0);
                            SourceMachineState::Eval {
                                expr: first,
                                env: env.clone(),
                                continuation: SourceContinuation::CallArgument {
                                    callee: value,
                                    remaining: args,
                                    lowered: Vec::new(),
                                    env,
                                    next,
                                },
                            }
                        }
                    }
                    SourceContinuation::CallArgument {
                        callee,
                        mut remaining,
                        mut lowered,
                        env,
                        next,
                    } => {
                        lowered.push(value);
                        if remaining.is_empty() {
                            self.source_call_state(builder, callee, lowered, env, *next)?
                        } else {
                            let first = remaining.remove(0);
                            SourceMachineState::Eval {
                                expr: first,
                                env: env.clone(),
                                continuation: SourceContinuation::CallArgument {
                                    callee,
                                    remaining,
                                    lowered,
                                    env,
                                    next,
                                },
                            }
                        }
                    }
                    SourceContinuation::IfScrutinee { .. }
                    | SourceContinuation::ProjectRecord { .. } => {
                        return Err(unsupported(
                            "ComputationalRecursor",
                            "source continuation frame is not implemented",
                        ));
                    }
                },
            };
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_dynamic_host_result_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        default: RuntimeTrap,
        env: &[Lowered],
        suffix_source: SourceContinuation<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let active = Self::source_terminal_active(&suffix_source).ok_or_else(|| {
            unsupported(
                "NativeJoinPlanV1",
                "dynamic HostResult source join has no exact outer continuation",
            )
        })?;
        let (prefix, suffix_pending, required_kind, site_id) =
            self.planned_active_scalar_cut(active)?;
        let join_id = self.next_source_join;
        self.next_source_join = self
            .next_source_join
            .checked_add(1)
            .expect("compiler-private source join identity exhausted");

        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        builder.ins().brif(success, ok_block, &[], err_block, &[]);

        for (predecessor_id, block, constructor, payload) in [
            (0, ok_block, ok_constructor, ok),
            (1, err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let edge = SourceJoinEdge {
                join_id,
                predecessor_id,
                target: merge,
                expected_outer: active.cursor,
                required_kind,
                prefix: prefix.clone(),
            };
            let continuation =
                Self::instantiate_source_prefix_to_join(&suffix_source, edge)?;
            let lowered = if let Some(case) = cases
                .iter()
                .find(|case| case.constructor == constructor && case.binders == 1)
            {
                let mut arm_env = vec![payload];
                arm_env.extend_from_slice(env);
                self.lower_source_machine_with_continuation(
                    builder,
                    case.body.clone(),
                    arm_env,
                    continuation,
                )?
            } else {
                self.lower_source_machine_with_continuation(
                    builder,
                    RuntimeExpr::Trap(default.clone()),
                    env.to_vec(),
                    continuation,
                )?
            };
            if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "predecessor {predecessor_id} for checked site {site_id} did not seal its affine join edge"
                    ),
                ));
            }
        }

        builder.switch_to_block(merge);
        let merged = Self::reconstruct_scalar_join_value(
            required_kind,
            builder.block_params(merge)[0],
        );
        let suffix_active = ActiveContinuationFrame {
            activation: active.activation,
            cursor: active.cursor,
            parent: active.parent,
            pending: suffix_pending,
            selected_ancestry: active.selected_ancestry,
        };
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    fn lower_source_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_source: SourceContinuation<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;
        let SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(outer_edge)) =
            suffix_source
        else {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "nested dynamic constructor is not inside one exact planned source cut",
            ));
        };
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor source match block");
        for (predecessor_id, alternative) in dynamic.alternatives.into_iter().enumerate() {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let case = match select_dynamic_constructor_case(cases, &alternative, default)? {
                Ok(case) => case,
                Err(_) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let edge = SourceJoinEdge {
                join_id: outer_edge.join_id,
                predecessor_id: u32::try_from(predecessor_id)
                    .map_err(|_| unsupported("NativeJoinPlanV1", "too many predecessors"))?,
                target: merge,
                expected_outer: outer_edge.expected_outer,
                required_kind: outer_edge.required_kind,
                prefix: outer_edge.prefix.clone(),
            };
            let arm_env = materialize_dynamic_constructor_env(&alternative, env);
            let lowered = self.lower_source_machine_with_continuation(
                builder,
                case.body.clone(),
                arm_env,
                SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)),
            )?;
            if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "nested dynamic constructor predecessor did not seal its edge",
                ));
            }
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        builder.switch_to_block(merge);
        let value = builder.block_params(merge)[0];
        builder.ins().jump(outer_edge.target, &[value.into()]);
        Ok(Lowered::RecursiveBackedge)
    }

    fn finish_source_constructor(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        constructor: RuntimeSymbol,
        lowered_args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        if lowered_args
            .iter()
            .any(|arg| matches!(arg, Lowered::RecursiveBackedge))
        {
            return Ok(Lowered::RecursiveBackedge);
        }
        if lowered_args.is_empty()
            && (constructor == self.process_symbols.bool_true
                || constructor == self.process_symbols.bool_false)
        {
            let known = constructor == self.process_symbols.bool_true;
            return Ok(Lowered::Bool {
                value: builder.ins().iconst(types::I64, i64::from(known)),
                known: Some(known),
            });
        }
        if constructor == self.process_symbols.nat_zero && lowered_args.is_empty() {
            return Ok(Lowered::StructuralNat(StructuralNatV1 {
                value: builder.ins().iconst(types::I64, 0),
            }));
        }
        if constructor == self.process_symbols.nat_suc {
            if let [Lowered::StructuralNat(predecessor)] = lowered_args.as_slice() {
                return Ok(Lowered::StructuralNat(StructuralNatV1 {
                    value: builder.ins().iadd_imm(predecessor.value, 1),
                }));
            }
        }
        Ok(Lowered::Constructor {
            constructor,
            args: lowered_args,
        })
    }

    fn source_call_state<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        callee: Lowered,
        args: Vec<Lowered>,
        env: Vec<Lowered>,
        continuation: SourceContinuation<'b>,
    ) -> Result<SourceMachineState<'b>, CraneliftBackendError> {
        match callee {
            Lowered::Closure {
                captures,
                params,
                body,
            } => {
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut call_env = args;
                call_env.extend(captures);
                call_env.extend(env);
                Ok(SourceMachineState::Eval {
                    expr: body,
                    env: call_env,
                    continuation,
                })
            }
            Lowered::DeclarationClosure {
                symbol,
                captures,
                params,
                body,
            } => {
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut call_env = args;
                call_env.extend(captures);
                call_env.extend(env);
                let _ = symbol;
                Ok(SourceMachineState::Eval {
                    expr: body,
                    env: call_env,
                    continuation,
                })
            }
            recursor @ Lowered::ComputationalRecursorClosure { .. } => {
                let (base, boundary) = decompose_computational_recursor(recursor);
                let (_, invocation) =
                    boundary.expect("recursor closure carries an invocation segment");
                let expected_outer = Self::source_continuation_outer(&continuation).ok_or_else(|| {
                    unsupported(
                        "ComputationalRecursor",
                        "recursive invocation source context has no outer cursor",
                    )
                })?;
                if expected_outer != invocation.resume_cursor {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "recursive invocation source context has the wrong outer cursor",
                    ));
                }
                let outer = Self::source_terminal_active(&continuation).ok_or_else(|| {
                    unsupported(
                        "ComputationalRecursor",
                        "recursive invocation source context has no live outer frame",
                    )
                })?;
                let armed = ArmedInvocation {
                    suspended: continuation,
                    expected_outer,
                };
                if armed.expected_outer
                    != Self::source_continuation_outer(&armed.suspended)
                        .expect("armed invocation has an outer cursor")
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "armed invocation endpoint changed outer cursor",
                    ));
                }
                if let Lowered::BoundedNat(predecessor) = base {
                    if !args.is_empty() {
                        return Err(unsupported(
                            "BoundedNat",
                            "structural Nat recursive hypothesis takes no arguments",
                        ));
                    }
                    let mut frames = recursor_eliminator_frames(&invocation.owned_layers);
                    frames.push(EliminatorFrame::Active(outer));
                    frames.push(EliminatorFrame::InvocationReturn);
                    let returned = self.lower_bounded_nat_computational(
                        builder,
                        predecessor,
                        false,
                        &frames,
                    )?;
                    return Ok(SourceMachineState::Value {
                        value: returned,
                        continuation: armed.suspended,
                    });
                } else {
                    let Lowered::Closure {
                        captures,
                        params,
                        body,
                    } = base
                    else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "recursive constructor field is not a closure",
                        ));
                    };
                    if params.len() != args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "recursive field expects {} args but call provides {}",
                                params.len(),
                                args.len()
                            ),
                        ));
                    }
                    let mut call_env = args;
                    call_env.extend(captures);
                    call_env.extend(env);
                    return Ok(SourceMachineState::Eval {
                        expr: body,
                        env: call_env,
                        continuation: SourceContinuation::ApplyRecursorLayers {
                            remaining: invocation.owned_layers,
                            outer,
                            next: Box::new(armed.suspended),
                        },
                    });
                }
            }
            _ => Err(unsupported("Call", "callee is not a closure")),
        }
    }

    fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: &RuntimeExpr,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        match expr {
            RuntimeExpr::Value(value) => self.lower_value(builder, value),
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| unsupported("Var", format!("no runtime binding for index {index}"))),
            RuntimeExpr::PrimitiveCall { primitive, args } => {
                self.lower_primitive_call(builder, primitive, args, env)
            }
            RuntimeExpr::Let { value, body } => {
                let lowered_value = self.lower_expr(builder, value, env)?;
                if matches!(lowered_value, Lowered::RecursiveBackedge) {
                    return Ok(Lowered::RecursiveBackedge);
                }
                if let Lowered::Trap(trap) = lowered_value {
                    return Ok(Lowered::Trap(trap));
                }
                let mut body_env = vec![lowered_value];
                body_env.extend_from_slice(env);
                self.lower_expr(builder, body, &body_env)
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                if matches!(lowered_scrutinee, Lowered::RecursiveBackedge) {
                    return Ok(Lowered::RecursiveBackedge);
                }
                let Lowered::Bool { value, known } = lowered_scrutinee else {
                    return Err(unsupported(
                        "If",
                        "branch lowering requires a Bool scrutinee",
                    ));
                };
                if let Some(scrutinee) = known {
                    return if scrutinee {
                        self.lower_expr(builder, then_expr, env)
                    } else {
                        self.lower_expr(builder, else_expr, env)
                    };
                }
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                for (block, arm) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_expr(builder, arm, env)?;
                    let Lowered::Int { value, .. } = lowered else {
                        return Err(unsupported(
                            "If",
                            "dynamic native If arms must produce scalar Int values",
                        ));
                    };
                    builder.ins().jump(merge, &[value.into()]);
                }
                builder.switch_to_block(merge);
                Ok(Lowered::Int {
                    value: builder.block_params(merge)[0],
                    known: None,
                })
            }
            RuntimeExpr::Construct { constructor, args } => {
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                if lowered_args
                    .iter()
                    .any(|arg| matches!(arg, Lowered::RecursiveBackedge))
                {
                    return Ok(Lowered::RecursiveBackedge);
                }
                if lowered_args.is_empty()
                    && (constructor == &self.process_symbols.bool_true
                        || constructor == &self.process_symbols.bool_false)
                {
                    let known = constructor == &self.process_symbols.bool_true;
                    return Ok(Lowered::Bool {
                        value: builder.ins().iconst(types::I64, i64::from(known)),
                        known: Some(known),
                    });
                }
                if constructor == &self.process_symbols.nat_zero && lowered_args.is_empty() {
                    return Ok(Lowered::StructuralNat(StructuralNatV1 {
                        value: builder.ins().iconst(types::I64, 0),
                    }));
                }
                if constructor == &self.process_symbols.nat_suc {
                    if let [Lowered::StructuralNat(predecessor)] = lowered_args.as_slice() {
                        return Ok(Lowered::StructuralNat(StructuralNatV1 {
                            value: builder.ins().iadd_imm(predecessor.value, 1),
                        }));
                    }
                }
                Ok(Lowered::Constructor {
                    constructor: constructor.clone(),
                    args: lowered_args,
                })
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                if requires_heterogeneous_deforestation(scrutinee)
                    || self.declaration_call_produces_deforestable_aggregate(scrutinee)
                {
                    return self.lower_computational_producer_expr(
                        builder,
                        scrutinee,
                        env,
                        &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                            cases,
                            default,
                            env,
                            retained_scrutinee_index: None,
                            deferred_constructor_case: None,
                        })],
                    );
                }
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                if let Lowered::BorrowedNativeValue { pointer } = lowered_scrutinee {
                    return self.lower_borrowed_match(builder, pointer, cases, default, env);
                }
                if let Lowered::BorrowedOption {
                    present,
                    value,
                    none,
                    some,
                } = lowered_scrutinee
                {
                    return self.lower_borrowed_option_match(
                        builder, present, value, &none, &some, cases, default, env,
                    );
                }
                if let Lowered::BoundedNat(nat) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(builder, nat, false, cases, default, env);
                }
                if let Lowered::StructuralNat(nat) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        cases,
                        default,
                        env,
                    );
                }
                if let Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                } = lowered_scrutinee
                {
                    return self.lower_dynamic_host_result_match(
                        builder,
                        success,
                        *error,
                        *ok,
                        &err_constructor,
                        &ok_constructor,
                        cases,
                        env,
                    );
                }
                if let Lowered::DynamicConstructor(dynamic) = lowered_scrutinee {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Ordinary {
                            cases,
                            default,
                            env,
                        },
                    );
                }
                if let Lowered::Bool { value, known } = lowered_scrutinee {
                    let true_case = cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "Match",
                            "Bool match requires zero-binder True and False cases",
                        ));
                    };
                    if let Some(selected) = known {
                        return self.lower_expr(
                            builder,
                            if selected { &true_case.body } else { &false_case.body },
                            env,
                        );
                    }
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder
                        .ins()
                        .brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    for (block, case) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let lowered = self.lower_expr(builder, &case.body, env)?;
                        let (value, branch_kind) =
                            self.merge_scalar_branch(builder, lowered, "Match")?;
                        Self::record_scalar_merge_kind(
                            "Match",
                            &mut merge_kind,
                            branch_kind,
                        )?;
                        builder.ins().jump(merge, &[value.into()]);
                    }
                    builder.switch_to_block(merge);
                    let value = builder.block_params(merge)[0];
                    return Ok(match merge_kind {
                        Some(ScalarMergeKind::ExitCode) => {
                            Lowered::ProcessExitStatus { value }
                        }
                        Some(ScalarMergeKind::Bool) => Lowered::Bool { value, known: None },
                        Some(ScalarMergeKind::Int) => Lowered::Int { value, known: None },
                        Some(ScalarMergeKind::StructuralNat) => {
                            Lowered::StructuralNat(StructuralNatV1 { value })
                        }
                        Some(ScalarMergeKind::RecursiveBackedge) => {
                            unreachable!("backedges do not establish a merge result kind")
                        }
                        None => unreachable!("Bool match emits both closed alternatives"),
                    });
                }
                let Lowered::Constructor { constructor, args } = lowered_scrutinee else {
                    return Err(unsupported("Match", "scrutinee is not a constructor value"));
                };
                let Some(case) = cases.iter().find(|case| case.constructor == constructor) else {
                    return Ok(Lowered::Trap(default.clone()));
                };
                if case.binders != args.len() {
                    return Err(unsupported(
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
                self.lower_expr(builder, &case.body, &case_env)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee,
                cases,
                default,
            } => {
                self.lower_computational_match_expr(
                    builder,
                    scrutinee,
                    cases,
                    default,
                    env,
                    env,
                )
            }
            RuntimeExpr::Record { fields } => {
                let lowered_fields = fields
                    .iter()
                    .map(|(name, expr)| Ok((name.clone(), self.lower_expr(builder, expr, env)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                Ok(Lowered::Record {
                    fields: lowered_fields,
                })
            }
            RuntimeExpr::Project { record, field } => {
                let lowered_record = self.lower_expr(builder, record, env)?;
                let Lowered::Record { fields } = lowered_record else {
                    return Err(unsupported(
                        "Project",
                        "record projection needs a record value",
                    ));
                };
                fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *field).then_some(value))
                    .ok_or_else(|| unsupported("Project", format!("missing field {field}")))
            }
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let lowered_captures = captures
                    .iter()
                    .map(|symbol| self.lower_seed_capture(builder, symbol))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Lowered::Closure {
                    captures: lowered_captures,
                    params: params.clone(),
                    body: (**body).clone(),
                })
            }
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                let captures = captures
                    .iter()
                    .map(|capture| self.lower_expr(builder, capture, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Lowered::Closure {
                    captures,
                    params: params.clone(),
                    body: (**body).clone(),
                })
            }
            RuntimeExpr::DeclarationRef { symbol } => self.lower_declaration_ref(builder, symbol),
            RuntimeExpr::ImportedDeclarationRef {
                symbol,
                dependency,
                dependency_semantic_hash,
            } => Err(unsupported(
                "ImportedDeclarationRef",
                format!(
                    "imported declaration {symbol} from {dependency} @ {dependency_semantic_hash} requires dependency linking"
                ),
            )),
            RuntimeExpr::Call { callee, args } => {
                let lowered_callee = self.lower_expr(builder, callee, env)?;
                match lowered_callee {
                    Lowered::DeclarationClosure {
                        symbol,
                        captures,
                        params,
                        body,
                    } => self.lower_recursive_declaration_call(
                        builder,
                        &symbol,
                        &captures,
                        &params,
                        &body,
                        args,
                        env,
                        None,
                    ),
                    Lowered::Closure {
                        captures,
                        params,
                        body,
                    } => {
                        if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                            if let Some((cases, default)) =
                                ordinary_match_continuation(&params, &body)
                            {
                                let mut frame_env = captures;
                                frame_env.extend_from_slice(env);
                                return self.lower_computational_producer_expr(
                                    builder,
                                    &args[0],
                                    env,
                                    &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                        cases,
                                        default,
                                        env: &frame_env,
                                        retained_scrutinee_index: Some(0),
                                        deferred_constructor_case: None,
                                    })],
                                );
                            }
                        }
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, env))
                            .collect::<Result<Vec<_>, _>>()?;
                        if params.len() != call_env.len() {
                            return Err(unsupported(
                                "Call",
                                format!(
                                    "closure expects {} args but call provides {}",
                                    params.len(),
                                    call_env.len()
                                ),
                            ));
                        }
                        call_env.extend(captures);
                        call_env.extend_from_slice(env);
                        self.lower_expr(builder, &body, &call_env)
                    }
                    callee @ Lowered::ComputationalRecursorClosure { .. } => {
                        let (base, boundary) = decompose_computational_recursor(callee);
                        let (_, invocation) = boundary.expect(
                            "recursor closure carries an invocation segment",
                        );
                        let mut frames =
                            recursor_eliminator_frames(&invocation.owned_layers);
                        frames.push(EliminatorFrame::InvocationReturn);
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            return self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &frames,
                            );
                        }
                        let Lowered::Closure {
                            captures,
                            params,
                            body,
                        } = base
                        else {
                            return Err(unsupported(
                                "ComputationalMatch",
                                "recursive constructor field is not a closure",
                            ));
                        };
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, env))
                            .collect::<Result<Vec<_>, _>>()?;
                        if params.len() != call_env.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "recursive field expects {} args but call provides {}",
                                    params.len(),
                                    call_env.len()
                                ),
                            ));
                        }
                        call_env.extend(captures);
                        call_env.extend_from_slice(env);
                        self.lower_computational_producer_expr(
                            builder,
                            &body,
                            &call_env,
                            &frames,
                        )
                    }
                    _ => Err(unsupported("Call", "callee is not a closure")),
                }
            }
            RuntimeExpr::Trap(trap) => Ok(Lowered::Trap(trap.clone())),
            RuntimeExpr::Effect {
                family,
                operation,
                capability,
                args,
            } if self.process_object => {
                self.lower_process_host_effect(builder, family, *operation, capability.as_ref(), args, env)
            }
            RuntimeExpr::Effect { family, operation, .. } => Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is not modeled in the supported native subset",
                    *operation as u16
                ),
            )),
        }
    }

    fn lower_process_host_effect(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        family: &RuntimeSymbol,
        operation: ken_host::HostOpV1,
        capability: Option<&crate::RuntimeCapabilityUse>,
        args: &[RuntimeExpr],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        if !CRANELIFT_HOST_EFFECT_CONSUMERS_V1.contains(&operation) {
            return Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is a represented unavailable lane",
                    operation as u16
                ),
            ));
        }
        let lowered = args
            .iter()
            .map(|argument| self.lower_expr(builder, argument, env))
            .collect::<Result<Vec<_>, _>>()?;
        let pointer_type = builder.func.dfg.value_type(
            self.invocation_pointer
                .expect("process effect lowering owns an invocation pointer"),
        );
        let wire = ken_host::host_effect_wire_layout_v1(operation).map_err(|error| {
            unsupported(
                "Effect",
                format!("generated HostEffectAbiV1 layout rejected: {error:?}"),
            )
        })?;
        let request_offset = |index: usize| {
            i32::try_from(wire.request_offsets[index])
                .expect("C-probed request offset was checked as u32")
        };
        let request = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            wire.request_size,
            wire.request_align_shift,
        ));
        match operation {
            ken_host::HostOpV1::ConsoleWrite
            | ken_host::HostOpV1::ConsoleFlush
            | ken_host::HostOpV1::ConsoleIsTerminal => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "ambient Console carried a capability",
                    ));
                }
                let stream = lowered
                    .first()
                    .and_then(console_stream_tag)
                    .ok_or_else(|| {
                        unsupported("Effect", "Console operation has a malformed Stream operand")
                    })?;
                let stream = builder.ins().iconst(types::I64, stream);
                builder
                    .ins()
                    .stack_store(stream, request, request_offset(0));
                if operation == ken_host::HostOpV1::ConsoleWrite {
                    let (data, len) = self.wire_bytes(
                        builder,
                        lowered.get(1).ok_or_else(|| {
                            unsupported("Effect", "Console.Write is missing Bytes")
                        })?,
                    )?;
                    builder.ins().stack_store(data, request, request_offset(1));
                    builder.ins().stack_store(len, request, request_offset(2));
                }
            }
            ken_host::HostOpV1::FsReadFile
            | ken_host::HostOpV1::FsWriteFile
            | ken_host::HostOpV1::FsChangeMode
            | ken_host::HostOpV1::FsOpen => {
                let capability = capability
                    .ok_or_else(|| unsupported("Effect", "FS operation has no live capability"))?;
                let Lowered::CapabilityToken { value: token } =
                    self.lower_expr(builder, &capability.value, env)?
                else {
                    return Err(unsupported(
                        "Effect",
                        "FS capability operand is not the opaque invocation token",
                    ));
                };
                builder.ins().stack_store(token, request, request_offset(0));
                let (path, path_len) = self.wire_bytes(
                    builder,
                    lowered
                        .first()
                        .ok_or_else(|| unsupported("Effect", "FS operation is missing its path"))?,
                )?;
                builder.ins().stack_store(path, request, request_offset(1));
                builder
                    .ins()
                    .stack_store(path_len, request, request_offset(2));
                if operation == ken_host::HostOpV1::FsWriteFile {
                    let policy = lowered.get(1).and_then(create_policy_tag).ok_or_else(|| {
                        unsupported("Effect", "FS.WriteFile has a malformed CreatePolicy")
                    })?;
                    let (bytes, bytes_len) = self.wire_bytes(
                        builder,
                        lowered.get(2).ok_or_else(|| {
                            unsupported("Effect", "FS.WriteFile is missing contents")
                        })?,
                    )?;
                    let policy = builder.ins().iconst(types::I64, policy);
                    builder
                        .ins()
                        .stack_store(policy, request, request_offset(3));
                    builder.ins().stack_store(bytes, request, request_offset(4));
                    builder
                        .ins()
                        .stack_store(bytes_len, request, request_offset(5));
                } else if operation == ken_host::HostOpV1::FsChangeMode {
                    let Lowered::Int { value: mode, .. } = lowered.get(1).ok_or_else(|| {
                        unsupported("Effect", "FS.ChangeMode is missing its mode")
                    })?
                    else {
                        return Err(unsupported("Effect", "FS.ChangeMode mode is not an Int"));
                    };
                    let in_range = builder.ins().icmp_imm(
                        cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
                        *mode,
                        0o7777,
                    );
                    let narrowed = builder.ins().ireduce(types::I16, *mode);
                    let invalid = builder.ins().iconst(types::I16, 0xffff);
                    let mode = builder.ins().select(in_range, narrowed, invalid);
                    builder.ins().stack_store(mode, request, request_offset(3));
                } else if operation == ken_host::HostOpV1::FsOpen {
                    let mode =
                        lowered
                            .get(1)
                            .and_then(resource_open_mode_tag)
                            .ok_or_else(|| {
                                unsupported("Effect", "FS.Open has a malformed ResourceOpenMode")
                            })?;
                    let mode = builder.ins().iconst(types::I64, mode);
                    builder.ins().stack_store(mode, request, request_offset(3));
                }
            }
            ken_host::HostOpV1::FsHandleMetadata | ken_host::HostOpV1::ResourceRelease => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "resource operation carried a capability",
                    ));
                }
                let Lowered::ResourceToken { value: token } = lowered.first().ok_or_else(|| {
                    unsupported("Effect", "resource operation is missing its token")
                })?
                else {
                    return Err(unsupported(
                        "Effect",
                        "resource operand is not an opaque resource token",
                    ));
                };
                builder
                    .ins()
                    .stack_store(*token, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferAllocate => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "buffer allocation carried a capability",
                    ));
                }
                let Lowered::Int {
                    value: capacity, ..
                } = lowered.first().ok_or_else(|| {
                    unsupported("Effect", "BufferAllocate is missing its capacity")
                })?
                else {
                    return Err(unsupported("Effect", "BufferAllocate capacity is not Int"));
                };
                builder
                    .ins()
                    .stack_store(*capacity, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferFreeze => {
                if capability.is_some() {
                    return Err(unsupported("Effect", "BufferFreeze carried a capability"));
                }
                let Lowered::ResourceToken { value: token } = lowered
                    .first()
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its buffer"))?
                else {
                    return Err(unsupported(
                        "Effect",
                        "BufferFreeze buffer is not a resource",
                    ));
                };
                let Lowered::Int { value: start, .. } = lowered
                    .get(1)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its start"))?
                else {
                    return Err(unsupported("Effect", "BufferFreeze start is not Int"));
                };
                let Lowered::Int { value: length, .. } = lowered
                    .get(2)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its length"))?
                else {
                    return Err(unsupported("Effect", "BufferFreeze length is not Int"));
                };
                for (index, value) in [*token, *start, *length].into_iter().enumerate() {
                    builder
                        .ins()
                        .stack_store(value, request, request_offset(index));
                }
            }
            ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "positioned resource operation carried a capability",
                    ));
                }
                let resource = |index: usize, name: &str| {
                    let Some(Lowered::ResourceToken { value }) = lowered.get(index) else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not a resource"),
                        ));
                    };
                    Ok(*value)
                };
                let integer = |index: usize, name: &str| {
                    let Some(Lowered::Int { value, .. }) = lowered.get(index) else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not Int"),
                        ));
                    };
                    Ok(*value)
                };
                let file = resource(0, "file")?;
                let file_offset = integer(1, "file offset")?;
                let buffer = resource(2, "buffer")?;
                let buffer_start = integer(3, "buffer start")?;
                let length = integer(4, "length")?;
                for (index, value) in [file, buffer, file_offset, buffer_start, length]
                    .into_iter()
                    .enumerate()
                {
                    builder
                        .ins()
                        .stack_store(value, request, request_offset(index));
                }
            }
            _ => unreachable!("availability was checked above"),
        }
        let reply = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            wire.reply_size,
            wire.reply_align_shift,
        ));
        let invocation = self
            .invocation_pointer
            .expect("process effect lowering owns an invocation pointer");
        let op = builder.ins().iconst(types::I64, operation as i64);
        let request_pointer = builder.ins().stack_addr(pointer_type, request, 0);
        let request_size = builder
            .ins()
            .iconst(types::I64, i64::from(wire.request_size));
        let reply_pointer = builder.ins().stack_addr(pointer_type, reply, 0);
        let call = builder.ins().call(
            self.host_dispatch
                .expect("process effect lowering owns one host dispatch import"),
            &[invocation, op, request_pointer, request_size, reply_pointer],
        );
        let status = builder.inst_results(call)[0];
        Self::require_i64(builder, status, 0);
        let tag = builder.ins().stack_load(
            types::I64,
            reply,
            i32::try_from(wire.reply_tag_offset).expect("reply tag offset is u32"),
        );
        let detail = builder.ins().stack_load(
            types::I64,
            reply,
            i32::try_from(wire.reply_detail_offset).expect("reply detail offset is u32"),
        );
        if operation == ken_host::HostOpV1::ConsoleIsTerminal {
            Self::require_i64(builder, tag, wire.reply_bool_tag as i64);
            Ok(Lowered::Bool {
                value: detail,
                known: None,
            })
        } else {
            let success_tag = match operation {
                ken_host::HostOpV1::FsReadFile => wire.reply_bytes_tag,
                ken_host::HostOpV1::FsOpen => wire.reply_resource_tag,
                ken_host::HostOpV1::FsHandleMetadata => wire.reply_metadata_tag,
                ken_host::HostOpV1::BufferAllocate => wire.reply_resource_tag,
                ken_host::HostOpV1::BufferFreeze => wire.reply_bytes_tag,
                ken_host::HostOpV1::FsReadAt => wire.reply_read_progress_tag,
                ken_host::HostOpV1::FsWriteAt => wire.reply_write_progress_tag,
                _ => wire.reply_unit_tag,
            } as i64;
            let accepted_tags = match operation {
                ken_host::HostOpV1::FsHandleMetadata => vec![
                    success_tag,
                    wire.reply_error_tag as i64,
                    wire.reply_resource_error_tag as i64,
                ],
                ken_host::HostOpV1::ResourceRelease => {
                    vec![success_tag, wire.reply_resource_error_tag as i64]
                }
                ken_host::HostOpV1::BufferAllocate | ken_host::HostOpV1::BufferFreeze => {
                    vec![success_tag, wire.reply_resource_error_tag as i64]
                }
                ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => vec![
                    success_tag,
                    wire.reply_error_tag as i64,
                    wire.reply_resource_error_tag as i64,
                ],
                _ => vec![success_tag, wire.reply_error_tag as i64],
            };
            Self::require_one_of_i64(builder, tag, &accepted_tags);
            let resource_schema = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_schema_offset)
                    .expect("resource error schema offset is u32"),
            );
            let resource_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_kind_offset)
                    .expect("resource error kind offset is u32"),
            );
            let resource_identity = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_identity_offset)
                    .expect("resource error identity offset is u32"),
            );
            let resource_io = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_io_offset)
                    .expect("resource error io offset is u32"),
            );
            let resource_required = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_required_offset)
                    .expect("resource error required offset is u32"),
            );
            let resource_held = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_held_offset)
                    .expect("resource error held offset is u32"),
            );
            let resource_expected_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_expected_kind_offset)
                    .expect("resource error expected-kind offset is u32"),
            );
            let resource_actual_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_actual_kind_offset)
                    .expect("resource error actual-kind offset is u32"),
            );
            Self::validate_resource_error_reply(
                builder,
                tag,
                wire.reply_resource_error_tag,
                detail,
                resource_schema,
                resource_kind,
                resource_identity,
                resource_io,
                resource_required,
                resource_held,
                resource_expected_kind,
                resource_actual_kind,
                wire.resource_error_reply_schema,
                wire.resource_kind_fs_handle,
                wire.resource_kind_buffer,
            );
            let payload = builder.ins().sshr_imm(detail, 32);
            let last = self.process_symbols.io_errors.len().saturating_sub(1);
            let io_error = Lowered::DynamicConstructor(DynamicConstructorV1 {
                discriminator: builder.ins().band_imm(detail, 0xff),
                alternatives: self
                    .process_symbols
                    .io_errors
                    .iter()
                    .enumerate()
                    .map(|(tag, constructor)| DynamicConstructorAlternativeV1 {
                        tag: tag as i64,
                        constructor: constructor.clone(),
                        fields: (tag == last)
                            .then(|| {
                                vec![Lowered::Int {
                                    value: payload,
                                    known: None,
                                }]
                            })
                            .unwrap_or_default(),
                    })
                    .collect(),
            });
            let error = if matches!(
                operation,
                ken_host::HostOpV1::FsReadFile
                    | ken_host::HostOpV1::FsWriteFile
                    | ken_host::HostOpV1::FsChangeMode
                    | ken_host::HostOpV1::FsOpen
            ) {
                let path = lowered
                    .first()
                    .cloned()
                    .expect("validated FS operation has a path");
                Lowered::Constructor {
                    constructor: self.process_symbols.file_error.clone(),
                    args: vec![
                        Lowered::Constructor {
                            constructor: match operation {
                                ken_host::HostOpV1::FsReadFile => {
                                    self.process_symbols.file_operation_read.clone()
                                }
                                ken_host::HostOpV1::FsWriteFile => {
                                    self.process_symbols.file_operation_write.clone()
                                }
                                ken_host::HostOpV1::FsChangeMode => {
                                    self.process_symbols.file_operation_change_mode.clone()
                                }
                                ken_host::HostOpV1::FsOpen => {
                                    self.process_symbols.file_operation_read.clone()
                                }
                                _ => unreachable!("validated FS result operation"),
                            },
                            args: Vec::new(),
                        },
                        Lowered::Constructor {
                            constructor: self.process_symbols.option_some.clone(),
                            args: vec![path],
                        },
                        io_error,
                    ],
                }
            } else if matches!(
                operation,
                ken_host::HostOpV1::FsHandleMetadata
                    | ken_host::HostOpV1::ResourceRelease
                    | ken_host::HostOpV1::BufferAllocate
                    | ken_host::HostOpV1::BufferFreeze
                    | ken_host::HostOpV1::FsReadAt
                    | ken_host::HostOpV1::FsWriteAt
            ) {
                let generic = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    tag,
                    wire.reply_error_tag as i64,
                );
                let zero = builder.ins().iconst(types::I64, 0);
                let resource_surface_tag = builder.ins().iadd_imm(detail, 1);
                let surface_tag = builder.ins().select(generic, zero, resource_surface_tag);
                let surface_io = builder.ins().select(generic, detail, resource_io);
                let surface_io_payload = builder.ins().sshr_imm(surface_io, 32);
                let surface_io_error = Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: builder.ins().band_imm(surface_io, 0xff),
                    alternatives: self
                        .process_symbols
                        .io_errors
                        .iter()
                        .enumerate()
                        .map(|(tag, constructor)| DynamicConstructorAlternativeV1 {
                            tag: tag as i64,
                            constructor: constructor.clone(),
                            fields: (tag == last)
                                .then(|| {
                                    vec![Lowered::Int {
                                        value: surface_io_payload,
                                        known: None,
                                    }]
                                })
                                .unwrap_or_default(),
                        })
                        .collect(),
                });
                let identity_low = builder.ins().band_imm(resource_identity, 0xffff_ffff);
                let identity_high = builder.ins().ushr_imm(resource_identity, 32);
                let resource_kind_value = |discriminator| {
                    Lowered::DynamicConstructor(DynamicConstructorV1 {
                        discriminator,
                        alternatives: vec![
                            DynamicConstructorAlternativeV1 {
                                tag: wire.resource_kind_fs_handle as i64,
                                constructor: self.process_symbols.resource_kind_fs_handle.clone(),
                                fields: Vec::new(),
                            },
                            DynamicConstructorAlternativeV1 {
                                tag: wire.resource_kind_buffer as i64,
                                constructor: self.process_symbols.resource_kind_buffer.clone(),
                                fields: Vec::new(),
                            },
                        ],
                    })
                };
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: surface_tag,
                    alternatives: vec![
                        DynamicConstructorAlternativeV1 {
                            tag: 0,
                            constructor: self.process_symbols.resource_host_io.clone(),
                            fields: vec![surface_io_error.clone()],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 1,
                            constructor: self.process_symbols.resource_closed.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 2,
                            constructor: self.process_symbols.resource_malformed.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 3,
                            constructor: self.process_symbols.resource_right_not_held.clone(),
                            fields: vec![
                                Lowered::Int {
                                    value: resource_required,
                                    known: None,
                                },
                                Lowered::Int {
                                    value: resource_held,
                                    known: None,
                                },
                            ],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 4,
                            constructor: self.process_symbols.resource_release_failed.clone(),
                            fields: vec![
                                resource_kind_value(resource_kind),
                                Lowered::Constructor {
                                    constructor: self
                                        .process_symbols
                                        .resource_trace_identity
                                        .clone(),
                                    args: vec![
                                        Lowered::Int {
                                            value: identity_low,
                                            known: None,
                                        },
                                        Lowered::Int {
                                            value: identity_high,
                                            known: None,
                                        },
                                    ],
                                },
                                surface_io_error,
                            ],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 5,
                            constructor: self.process_symbols.resource_kind_mismatch.clone(),
                            fields: vec![
                                resource_kind_value(resource_expected_kind),
                                resource_kind_value(resource_actual_kind),
                            ],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 6,
                            constructor: self.process_symbols.resource_buffer_limit.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 7,
                            constructor: self.process_symbols.resource_invalid_offset.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 8,
                            constructor: self.process_symbols.resource_invalid_bounds.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 9,
                            constructor: self.process_symbols.resource_no_progress.clone(),
                            fields: Vec::new(),
                        },
                    ],
                })
            } else {
                io_error
            };
            let success = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                success_tag,
            );
            let ok = if operation == ken_host::HostOpV1::FsReadFile {
                Lowered::ResponseBytes {
                    pointer: builder.ins().stack_load(
                        pointer_type,
                        reply,
                        i32::try_from(wire.reply_bytes_data_offset)
                            .expect("reply bytes data offset is u32"),
                    ),
                    len: builder.ins().stack_load(
                        types::I64,
                        reply,
                        i32::try_from(wire.reply_bytes_len_offset)
                            .expect("reply bytes len offset is u32"),
                    ),
                }
            } else if operation == ken_host::HostOpV1::FsOpen {
                Lowered::ResourceToken { value: detail }
            } else if operation == ken_host::HostOpV1::BufferAllocate {
                Lowered::ResourceToken { value: detail }
            } else if operation == ken_host::HostOpV1::BufferFreeze {
                Lowered::ResponseBytes {
                    pointer: builder.ins().stack_load(
                        pointer_type,
                        reply,
                        i32::try_from(wire.reply_bytes_data_offset)
                            .expect("reply bytes data offset is u32"),
                    ),
                    len: builder.ins().stack_load(
                        types::I64,
                        reply,
                        i32::try_from(wire.reply_bytes_len_offset)
                            .expect("reply bytes len offset is u32"),
                    ),
                }
            } else if operation == ken_host::HostOpV1::FsReadAt {
                let reply_data = builder.ins().stack_load(
                    pointer_type,
                    reply,
                    i32::try_from(wire.reply_bytes_data_offset)
                        .expect("reply bytes data offset is u32"),
                );
                let reply_start = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_bytes_len_offset)
                        .expect("reply bytes len offset is u32"),
                );
                let nonzero = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    detail,
                    0,
                );
                let read_some = builder.ins().band(success, nonzero);
                let zero = builder.ins().iconst(types::I64, 0);
                let eof_data = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    reply_data,
                    zero,
                );
                let eof_start = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    reply_start,
                    0,
                );
                let eof_valid = builder.ins().band(eof_data, eof_start);
                let is_zero = builder.ins().bnot(nonzero);
                let read_eof = builder.ins().band(success, is_zero);
                Self::require_when(builder, read_eof, eof_valid);
                Self::require_when(builder, read_some, eof_data);
                let Lowered::Int {
                    value: request_start,
                    ..
                } = &lowered[3]
                else {
                    unreachable!("positioned request start was validated")
                };
                let Lowered::Int {
                    value: request_length,
                    ..
                } = &lowered[4]
                else {
                    unreachable!("positioned request length was validated")
                };
                let (count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    read_some,
                    detail,
                    *request_start,
                    *request_length,
                    Some(reply_start),
                );
                let span = Lowered::Constructor {
                    constructor: self.process_symbols.private_buffer_span.clone(),
                    args: vec![
                        Lowered::Int {
                            value: reply_start,
                            known: None,
                        },
                        Lowered::BoundedNat(count),
                    ],
                };
                let transferred = Lowered::Constructor {
                    constructor: self.process_symbols.private_transfer_count.clone(),
                    args: vec![
                        Lowered::BoundedNat(predecessor),
                        Lowered::BoundedNat(remaining),
                    ],
                };
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: builder.ins().uextend(types::I64, nonzero),
                    alternatives: vec![
                        DynamicConstructorAlternativeV1 {
                            tag: 0,
                            constructor: self.process_symbols.read_eof.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 1,
                            constructor: self.process_symbols.read_some.clone(),
                            fields: vec![span, transferred],
                        },
                    ],
                })
            } else if operation == ken_host::HostOpV1::FsWriteAt {
                let Lowered::Int {
                    value: request_start,
                    ..
                } = &lowered[3]
                else {
                    unreachable!("positioned request start was validated")
                };
                let Lowered::Int {
                    value: request_length,
                    ..
                } = &lowered[4]
                else {
                    unreachable!("positioned request length was validated")
                };
                let (_count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    success,
                    detail,
                    *request_start,
                    *request_length,
                    None,
                );
                Lowered::Constructor {
                    constructor: self.process_symbols.wrote.clone(),
                    args: vec![Lowered::Constructor {
                        constructor: self.process_symbols.private_transfer_count.clone(),
                        args: vec![
                            Lowered::BoundedNat(predecessor),
                            Lowered::BoundedNat(remaining),
                        ],
                    }],
                }
            } else if operation == ken_host::HostOpV1::FsHandleMetadata {
                Lowered::Int {
                    value: detail,
                    known: None,
                }
            } else {
                Lowered::Constructor {
                    constructor: self.process_symbols.unit.clone(),
                    args: Vec::new(),
                }
            };
            Ok(Lowered::HostResult {
                success,
                error: Box::new(error),
                ok: Box::new(ok),
                err_constructor: self.process_symbols.result_err.clone(),
                ok_constructor: self.process_symbols.result_ok.clone(),
            })
        }
    }

    fn wire_bytes(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value), CraneliftBackendError>
    {
        let pointer_type = builder.func.dfg.value_type(
            self.invocation_pointer
                .expect("process byte lowering owns an invocation pointer"),
        );
        match value {
            Lowered::BorrowedNativeValue { pointer } => {
                let kind = builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), *pointer, 0);
                Self::require_i64(builder, kind, 1);
                Ok((
                    builder
                        .ins()
                        .load(pointer_type, MemFlags::trusted(), *pointer, 16),
                    builder
                        .ins()
                        .load(types::I64, MemFlags::trusted(), *pointer, 24),
                ))
            }
            Lowered::ResponseBytes { pointer, len } => Ok((*pointer, *len)),
            Lowered::Bytes(bytes) => {
                if bytes.is_empty() {
                    return Ok((
                        builder.ins().iconst(pointer_type, 0),
                        builder.ins().iconst(types::I64, 0),
                    ));
                }
                let size = u32::try_from(bytes.len())
                    .map_err(|_| unsupported("Effect", "Bytes exceed native stack slot"))?;
                let slot = builder.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    size,
                    0,
                ));
                for (offset, byte) in bytes.iter().enumerate() {
                    let byte = builder.ins().iconst(types::I8, i64::from(*byte));
                    builder.ins().stack_store(byte, slot, offset as i32);
                }
                Ok((
                    builder.ins().stack_addr(pointer_type, slot, 0),
                    builder.ins().iconst(types::I64, bytes.len() as i64),
                ))
            }
            _ => Err(unsupported("Effect", "operand is not a Bytes value")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_unary_recursive_nat_fold(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        argument: Lowered,
        zero_body: &RuntimeExpr,
        suc_body: &RuntimeExpr,
        producer_env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let (target, structural) = match argument {
            Lowered::StructuralNat(nat) => (nat.value, true),
            Lowered::BoundedNat(nat) => (nat.value, false),
            _ => {
                return Err(unsupported(
                    "DeclarationRef",
                    "unary Nat recursion received a non-Nat representation",
                ));
            }
        };
        let zero = builder.ins().iconst(types::I64, 0);
        let zero_nat = if structural {
            Lowered::StructuralNat(StructuralNatV1 { value: zero })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(zero))
        };
        let mut zero_env = vec![zero_nat];
        zero_env.extend_from_slice(captures);
        zero_env.extend_from_slice(producer_env);
        let zero_lowered = self.lower_expr(builder, zero_body, &zero_env)?;
        let (initial, result_kind) =
            self.merge_scalar_branch(builder, zero_lowered, "DeclarationRef")?;
        if result_kind == ScalarMergeKind::RecursiveBackedge {
            return Err(unsupported(
                "DeclarationRef",
                "unary Nat recursion has no finite base result",
            ));
        }

        let loop_block = builder.create_block();
        let step_block = builder.create_block();
        let done_block = builder.create_block();
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder
            .ins()
            .jump(loop_block, &[zero.into(), initial.into()]);
        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction_value = builder.block_params(loop_block)[1];
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            target,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction_value.into()],
            step_block,
            &[],
        );

        builder.switch_to_block(step_block);
        let successor_value = builder.ins().iadd_imm(predecessor_value, 1);
        let predecessor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: predecessor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(predecessor_value))
        };
        let successor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: successor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(successor_value))
        };
        let induction = match result_kind {
            ScalarMergeKind::Int => Lowered::Int {
                value: induction_value,
                known: None,
            },
            ScalarMergeKind::Bool => Lowered::Bool {
                value: induction_value,
                known: None,
            },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 {
                value: induction_value,
            }),
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus {
                value: induction_value,
            },
            ScalarMergeKind::RecursiveBackedge => unreachable!(),
        };
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: None,
                argument_templates: vec![predecessor.clone()],
                induction: Some(induction),
            });
        // A Suc case sees its predecessor first, followed by the retained
        // scrutinee and the declaration's outer environment.
        let mut suc_env = vec![predecessor, successor];
        suc_env.extend_from_slice(captures);
        suc_env.extend_from_slice(producer_env);
        let next = self.lower_expr(builder, suc_body, &suc_env);
        self.active_recursive_declarations.pop();
        let (next, next_kind) = self.merge_scalar_branch(builder, next?, "DeclarationRef")?;
        if next_kind != result_kind {
            return Err(unsupported(
                "DeclarationRef",
                "unary Nat recursion changes its native result representation",
            ));
        }
        builder
            .ins()
            .jump(loop_block, &[successor_value.into(), next.into()]);
        builder.switch_to_block(done_block);
        let value = builder.block_params(done_block)[0];
        Ok(match result_kind {
            ScalarMergeKind::Int => Lowered::Int { value, known: None },
            ScalarMergeKind::Bool => Lowered::Bool { value, known: None },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 { value }),
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus { value },
            ScalarMergeKind::RecursiveBackedge => unreachable!(),
        })
    }

    fn declaration_is_recursive(&self, symbol: &RuntimeSymbol) -> bool {
        let Some(declaration) = self.declarations.get(symbol.as_str()).copied() else {
            return false;
        };
        let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
            return false;
        };

        let mut frontier = BTreeSet::new();
        let mut visited = BTreeSet::new();
        collect_runtime_declaration_refs(body, &mut frontier);
        while let Some(candidate) = frontier.pop_first() {
            if candidate == *symbol {
                return true;
            }
            if !visited.insert(candidate.clone()) {
                continue;
            }
            let Some(declaration) = self.declarations.get(candidate.as_str()).copied() else {
                continue;
            };
            if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
                collect_runtime_declaration_refs(body, &mut frontier);
            }
        }
        false
    }

    fn lower_recursive_declaration_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        params: &[String],
        body: &RuntimeExpr,
        args: &[RuntimeExpr],
        producer_env: &[Lowered],
        eliminators: Option<&[EliminatorFrame<'_>]>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let lowered_args = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, producer_env))
            .collect::<Result<Vec<_>, _>>()?;
        if params.len() != lowered_args.len() {
            return Err(unsupported(
                "DeclarationRef",
                format!(
                    "recursive declaration {symbol} expects {} args but call provides {}",
                    params.len(),
                    lowered_args.len()
                ),
            ));
        }

        if let Some(active) = self
            .active_recursive_declarations
            .iter()
            .rev()
            .find(|active| active.symbol == *symbol)
            .cloned()
        {
            if !same_recursive_argument_shapes(&active.argument_templates, &lowered_args) {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "recursive declaration {symbol} changes its native argument representation: {:?} -> {:?}",
                        active
                            .argument_templates
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>(),
                        lowered_args
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>()
                    ),
                ));
            }
            if let Some(induction) = active.induction {
                return Ok(induction);
            }
            let mut values = Vec::new();
            append_recursive_argument_values(&lowered_args, &mut values)?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );

            // Continue lowering only in a predecessor-free block. This keeps
            // the structured builder usable while the real recursive edge
            // returns directly to the loop header.
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(Lowered::RecursiveBackedge);
        }

        // Only declarations in an actual recursive SCC need the loop/result
        // closure below. Preserve the established direct-call lowering for
        // ordinary declarations, including constructor-valued HostIO trees.
        if !self.declaration_is_recursive(symbol) {
            let mut call_env = lowered_args.into_iter().rev().collect::<Vec<_>>();
            call_env.extend_from_slice(captures);
            call_env.extend_from_slice(producer_env);
            return if let Some(eliminators) = eliminators {
                self.lower_computational_producer_expr(builder, body, &call_env, eliminators)
            } else {
                self.lower_expr(builder, body, &call_env)
            };
        }

        if eliminators.is_none() && params.len() == 1 && lowered_args.len() == 1 {
            if let RuntimeExpr::Match {
                scrutinee, cases, ..
            } = body
            {
                if matches!(scrutinee.as_ref(), RuntimeExpr::Var(0)) {
                    let zero = cases.iter().find(|case| {
                        case.constructor == self.process_symbols.nat_zero && case.binders == 0
                    });
                    let suc = cases.iter().find(|case| {
                        case.constructor == self.process_symbols.nat_suc && case.binders == 1
                    });
                    if let (Some(zero), Some(suc)) = (zero, suc) {
                        return self.lower_unary_recursive_nat_fold(
                            builder,
                            symbol,
                            captures,
                            lowered_args
                                .into_iter()
                                .next()
                                .expect("unary recursion owns one argument"),
                            &zero.body,
                            &suc.body,
                            producer_env,
                        );
                    }
                }
            }
        }

        let header = builder.create_block();
        let done = builder.create_block();
        let mut initial_values = Vec::new();
        append_recursive_argument_values(&lowered_args, &mut initial_values)?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
        builder.append_block_param(done, types::I64);
        builder.ins().jump(
            header,
            &initial_values
                .iter()
                .copied()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        builder.switch_to_block(header);

        let mut parameters = builder.block_params(header).iter().copied();
        let loop_args = lowered_args
            .iter()
            .map(|template| rebuild_recursive_argument(template, &mut parameters))
            .collect::<Result<Vec<_>, _>>()?;
        if parameters.next().is_some() {
            return Err(unsupported(
                "DeclarationRef",
                "recursive declaration loop parameter shape is not closed",
            ));
        }
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: Some(header),
                argument_templates: lowered_args,
                induction: None,
            });
        // Runtime environments are de Bruijn-nearest first: source arguments
        // are evaluated left-to-right, then installed in reverse binder order,
        // followed by captures and the producer environment.
        let mut call_env = loop_args.into_iter().rev().collect::<Vec<_>>();
        call_env.extend_from_slice(captures);
        call_env.extend_from_slice(producer_env);
        let lowered = if let Some(eliminators) = eliminators {
            self.lower_computational_producer_expr(builder, body, &call_env, eliminators)
        } else {
            self.lower_expr(builder, body, &call_env)
        };
        self.active_recursive_declarations.pop();
        let lowered = lowered?;
        let (value, result_kind) = if eliminators.is_some() {
            let (value, exit_status) =
                self.merge_branch_value(builder, lowered, "DeclarationRef")?;
            (
                value,
                if exit_status {
                    ScalarMergeKind::ExitCode
                } else {
                    ScalarMergeKind::Int
                },
            )
        } else {
            match lowered {
                Lowered::Int { value, .. } => (value, ScalarMergeKind::Int),
                Lowered::Bool { value, .. } => (value, ScalarMergeKind::Bool),
                Lowered::StructuralNat(nat) => (nat.value, ScalarMergeKind::StructuralNat),
                Lowered::ProcessExitStatus { value } => (value, ScalarMergeKind::ExitCode),
                other => {
                    return Err(unsupported(
                        "DeclarationRef",
                        format!(
                            "recursive declaration {symbol} must produce Int, Bool, structural Nat, or entrypoint ExitCode; got {}",
                            lowered_value_kind(&other)
                        ),
                    ));
                }
            }
        };
        builder.ins().jump(done, &[value.into()]);
        builder.switch_to_block(done);
        let value = builder.block_params(done)[0];
        Ok(match result_kind {
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus { value },
            ScalarMergeKind::Bool => Lowered::Bool { value, known: None },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 { value }),
            ScalarMergeKind::Int => Lowered::Int { value, known: None },
            ScalarMergeKind::RecursiveBackedge => {
                unreachable!("a recursive declaration base establishes its result kind")
            }
        })
    }

    fn lower_declaration_ref(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
    ) -> Result<Lowered, CraneliftBackendError> {
        let declaration = self
            .declarations
            .get(symbol.as_str())
            .copied()
            .ok_or_else(|| {
                unsupported(
                    "DeclarationRef",
                    format!("{symbol} is not present in the exact RuntimeProgram"),
                )
            })?;
        let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
            return Err(unsupported(
                "DeclarationRef",
                format!("{symbol} is not an executable transparent declaration"),
            ));
        };
        if let RuntimeExpr::Closure {
            captures,
            params,
            body,
        } = body
        {
            let captures = captures
                .iter()
                .map(|capture| self.lower_seed_capture(builder, capture))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Lowered::DeclarationClosure {
                symbol: symbol.clone(),
                captures,
                params: params.clone(),
                body: (**body).clone(),
            });
        }
        if self.declaration_stack.contains(symbol) {
            return Err(unsupported(
                "DeclarationRef",
                format!("recursive non-function declaration {symbol} is unsupported"),
            ));
        }
        self.declaration_stack.push(symbol.clone());
        let result = self.lower_expr(builder, body, &[]);
        self.declaration_stack.pop();
        result
    }

    fn require_i64(
        builder: &mut FunctionBuilder<'_>,
        actual: cranelift_codegen::ir::Value,
        expected: i64,
    ) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let matches = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            actual,
            expected,
        );
        builder.ins().brif(matches, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_one_of_i64(
        builder: &mut FunctionBuilder<'_>,
        actual: cranelift_codegen::ir::Value,
        expected: &[i64],
    ) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let mut matches = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            actual,
            expected[0],
        );
        for expected in &expected[1..] {
            let next = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                actual,
                *expected,
            );
            matches = builder.ins().bor(matches, next);
        }
        builder.ins().brif(matches, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_nonzero(builder: &mut FunctionBuilder<'_>, value: cranelift_codegen::ir::Value) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let present =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, value, 0);
        builder.ins().brif(present, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_u8(builder: &mut FunctionBuilder<'_>, value: cranelift_codegen::ir::Value) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let in_range = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            value,
            i64::from(u8::MAX),
        );
        builder.ins().brif(in_range, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_true(builder: &mut FunctionBuilder<'_>, condition: cranelift_codegen::ir::Value) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        builder.ins().brif(condition, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_when(
        builder: &mut FunctionBuilder<'_>,
        enabled: cranelift_codegen::ir::Value,
        condition: cranelift_codegen::ir::Value,
    ) {
        let validate = builder.create_block();
        let done = builder.create_block();
        builder.ins().brif(enabled, validate, &[], done, &[]);
        builder.switch_to_block(validate);
        Self::require_true(builder, condition);
        builder.ins().jump(done, &[]);
        builder.switch_to_block(done);
    }

    fn mint_validated_progress_nat(
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        count: cranelift_codegen::ir::Value,
        request_start: cranelift_codegen::ir::Value,
        request_length: cranelift_codegen::ir::Value,
        reply_start: Option<cranelift_codegen::ir::Value>,
    ) -> (BoundedNatV1, BoundedNatV1, BoundedNatV1) {
        let positive = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            count,
            0,
        );
        let bounded = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            count,
            request_length,
        );
        let request_end = builder.ins().iadd(request_start, request_length);
        let request_no_wrap = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThanOrEqual,
            request_end,
            request_start,
        );
        let span_start = reply_start.unwrap_or(request_start);
        let span_end = builder.ins().iadd(span_start, count);
        let span_no_wrap = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThanOrEqual,
            span_end,
            span_start,
        );
        let starts_at_request = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            span_start,
            request_start,
        );
        let inside = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            span_end,
            request_end,
        );
        let valid = [
            positive,
            bounded,
            request_no_wrap,
            span_no_wrap,
            starts_at_request,
            inside,
        ]
        .into_iter()
        .reduce(|left, right| builder.ins().band(left, right))
        .expect("progress validation has fixed clauses");
        Self::require_when(builder, success, valid);

        let minted = BoundedNatV1::mint_after_reply_validation(count);
        let predecessor = minted.predecessor(builder);
        let remaining =
            BoundedNatV1::derived_from_validated(builder.ins().isub(request_length, count));
        (minted, predecessor, remaining)
    }

    fn validate_resource_io(
        builder: &mut FunctionBuilder<'_>,
        encoded: cranelift_codegen::ir::Value,
    ) {
        let discriminator = builder.ins().band_imm(encoded, 0xff);
        let other = builder.create_block();
        let ordinary = builder.create_block();
        let valid = builder.create_block();
        let is_other = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            discriminator,
            11,
        );
        builder.ins().brif(is_other, other, &[], ordinary, &[]);
        builder.switch_to_block(other);
        let middle = builder
            .ins()
            .band_imm(encoded, 0x0000_0000_ffff_ff00u64 as i64);
        Self::require_i64(builder, middle, 0);
        builder.ins().jump(valid, &[]);
        builder.switch_to_block(ordinary);
        let upper = builder.ins().ushr_imm(encoded, 8);
        Self::require_i64(builder, upper, 0);
        Self::require_one_of_i64(builder, discriminator, &[0, 1, 3, 4, 5, 6, 7, 8, 9, 10]);
        builder.ins().jump(valid, &[]);
        builder.switch_to_block(valid);
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_resource_error_reply(
        builder: &mut FunctionBuilder<'_>,
        reply_tag: cranelift_codegen::ir::Value,
        resource_reply_tag: u64,
        discriminator: cranelift_codegen::ir::Value,
        schema: cranelift_codegen::ir::Value,
        kind: cranelift_codegen::ir::Value,
        identity: cranelift_codegen::ir::Value,
        io: cranelift_codegen::ir::Value,
        required: cranelift_codegen::ir::Value,
        held: cranelift_codegen::ir::Value,
        actual_expected_kind: cranelift_codegen::ir::Value,
        actual_actual_kind: cranelift_codegen::ir::Value,
        expected_schema: u64,
        expected_kind: u64,
        buffer_kind: u64,
    ) {
        let resource = builder.create_block();
        let done = builder.create_block();
        let is_resource = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            reply_tag,
            resource_reply_tag as i64,
        );
        builder.ins().brif(is_resource, resource, &[], done, &[]);
        builder.switch_to_block(resource);
        let arms = (0..9).map(|_| builder.create_block()).collect::<Vec<_>>();
        let mut test = builder
            .current_block()
            .expect("resource reply validation block");
        for (index, arm) in arms.into_iter().enumerate() {
            let next = builder.create_block();
            if builder.current_block() != Some(test) {
                builder.switch_to_block(test);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                discriminator,
                index as i64,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            match index {
                0 | 1 => {
                    for field in [
                        schema,
                        kind,
                        identity,
                        io,
                        required,
                        held,
                        actual_expected_kind,
                        actual_actual_kind,
                    ] {
                        Self::require_i64(builder, field, 0);
                    }
                }
                2 => {
                    Self::require_i64(builder, schema, expected_schema as i64);
                    Self::require_i64(builder, kind, 0);
                    Self::require_i64(builder, identity, 0);
                    Self::require_i64(builder, io, 0);
                    Self::require_i64(builder, actual_expected_kind, 0);
                    Self::require_i64(builder, actual_actual_kind, 0);
                    Self::require_u8(builder, required);
                    Self::require_u8(builder, held);
                }
                3 => {
                    Self::require_i64(builder, schema, expected_schema as i64);
                    Self::require_one_of_i64(
                        builder,
                        kind,
                        &[expected_kind as i64, buffer_kind as i64],
                    );
                    Self::require_i64(builder, required, 0);
                    Self::require_i64(builder, held, 0);
                    Self::require_i64(builder, actual_expected_kind, 0);
                    Self::require_i64(builder, actual_actual_kind, 0);
                    Self::validate_resource_io(builder, io);
                }
                4 => {
                    for field in [schema, kind, identity, io, required, held] {
                        Self::require_i64(builder, field, 0);
                    }
                    Self::require_one_of_i64(
                        builder,
                        actual_expected_kind,
                        &[expected_kind as i64, buffer_kind as i64],
                    );
                    Self::require_one_of_i64(
                        builder,
                        actual_actual_kind,
                        &[expected_kind as i64, buffer_kind as i64],
                    );
                    let distinct = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                        actual_expected_kind,
                        actual_actual_kind,
                    );
                    Self::require_true(builder, distinct);
                }
                5..=8 => {
                    for field in [
                        schema,
                        kind,
                        identity,
                        io,
                        required,
                        held,
                        actual_expected_kind,
                        actual_actual_kind,
                    ] {
                        Self::require_i64(builder, field, 0);
                    }
                }
                _ => unreachable!(),
            }
            builder.ins().jump(done, &[]);
            test = next;
        }
        builder.switch_to_block(test);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(done);
    }

    fn lower_borrowed_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        pointer: cranelift_codegen::ir::Value,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let kind = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 0);
        Self::require_i64(builder, kind, 2);
        let tag = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 8);
        let arity = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 24);
        let pointer_type = builder.func.dfg.value_type(pointer);
        let fields = builder
            .ins()
            .load(pointer_type, MemFlags::trusted(), pointer, 16);
        if let [case] = cases {
            let (expected_tag, expected_arity) =
                borrowed_constructor_identity(&self.process_symbols, &case.constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            format!("{} has no borrowed constructor identity", case.constructor),
                        )
                    })?;
            if case.binders != expected_arity {
                return Err(unsupported(
                    "Match",
                    format!("{} borrowed arity mismatch", case.constructor),
                ));
            }
            let arm = builder.create_block();
            let rejected = builder.create_block();
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                expected_tag,
            );
            builder.ins().brif(selected, arm, &[], rejected, &[]);
            builder.switch_to_block(rejected);
            let failure = builder.ins().iconst(types::I64, -1);
            builder.ins().return_(&[failure]);
            builder.switch_to_block(arm);
            Self::require_i64(builder, arity, expected_arity as i64);
            if expected_arity != 0 {
                Self::require_nonzero(builder, fields);
            }
            let mut arm_env = (0..expected_arity)
                .map(|index| {
                    let field = builder.ins().iadd_imm(fields, (index * 32) as i64);
                    Lowered::BorrowedNativeValue { pointer: field }
                })
                .collect::<Vec<_>>();
            arm_env.extend_from_slice(env);
            return self.lower_expr(builder, &case.body, &arm_env);
        }
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        let mut test_block = builder.current_block().expect("borrowed match block");
        let mut merge_kind = None;
        for case in cases {
            let (expected_tag, expected_arity) =
                borrowed_constructor_identity(&self.process_symbols, &case.constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            format!("{} has no borrowed constructor identity", case.constructor),
                        )
                    })?;
            if case.binders != expected_arity {
                return Err(unsupported(
                    "Match",
                    format!("{} borrowed arity mismatch", case.constructor),
                ));
            }
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                expected_tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            Self::require_i64(builder, arity, expected_arity as i64);
            if expected_arity != 0 {
                Self::require_nonzero(builder, fields);
            }
            let mut arm_env = (0..expected_arity)
                .map(|index| {
                    let field = builder.ins().iadd_imm(fields, (index * 32) as i64);
                    Lowered::BorrowedNativeValue { pointer: field }
                })
                .collect::<Vec<_>>();
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, kind) = self.merge_scalar_branch(builder, lowered, "Match")?;
            Self::record_scalar_merge_kind("Match", &mut merge_kind, kind)?;
            builder.ins().jump(merge, &[value.into()]);
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(merge);
        let value = builder.block_params(merge)[0];
        Ok(match merge_kind {
            Some(ScalarMergeKind::ExitCode) => Lowered::ProcessExitStatus { value },
            Some(ScalarMergeKind::Bool) => Lowered::Bool { value, known: None },
            Some(ScalarMergeKind::StructuralNat) => {
                Lowered::StructuralNat(StructuralNatV1 { value })
            }
            Some(ScalarMergeKind::Int) => Lowered::Int { value, known: None },
            Some(ScalarMergeKind::RecursiveBackedge) => {
                unreachable!("backedges do not establish a merge result kind")
            }
            None => unreachable!("borrowed match emits at least one case"),
        })
    }

    fn lower_borrowed_option_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        present: cranelift_codegen::ir::Value,
        value: cranelift_codegen::ir::Value,
        none: &str,
        some: &str,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        let some_block = builder.create_block();
        let none_block = builder.create_block();
        let mut exit_merge = None;
        builder
            .ins()
            .brif(present, some_block, &[], none_block, &[]);
        for (block, symbol, fields) in [
            (some_block, some, vec![Lowered::Int { value, known: None }]),
            (none_block, none, Vec::new()),
        ] {
            builder.switch_to_block(block);
            let case = cases.iter().find(|case| case.constructor == symbol);
            let Some(case) = case else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            if case.binders != fields.len() {
                return Err(unsupported("Match", "borrowed Option arity mismatch"));
            }
            let mut arm_env = fields;
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, is_exit) = self.merge_branch_value(builder, lowered, "Match")?;
            Self::record_merge_kind("Match", &mut exit_merge, is_exit)?;
            builder.ins().jump(merge, &[value.into()]);
        }
        builder.switch_to_block(merge);
        let value = builder.block_params(merge)[0];
        Ok(if exit_merge == Some(true) {
            Lowered::ProcessExitStatus { value }
        } else {
            Lowered::Int { value, known: None }
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_dynamic_host_result_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        let mut merge_kind = None;
        builder.ins().brif(success, ok_block, &[], err_block, &[]);
        for (block, constructor, payload) in [
            (ok_block, ok_constructor, ok),
            (err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let Some(case) = cases
                .iter()
                .find(|case| case.constructor == constructor && case.binders == 1)
            else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            let mut arm_env = vec![payload];
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, branch_kind) = self.merge_scalar_branch(builder, lowered, "Match")?;
            Self::record_scalar_merge_kind("Match", &mut merge_kind, branch_kind)?;
            builder.ins().jump(merge, &[value.into()]);
        }
        builder.switch_to_block(merge);
        let value = builder.block_params(merge)[0];
        Ok(match merge_kind {
            Some(ScalarMergeKind::ExitCode) => Lowered::ProcessExitStatus { value },
            Some(ScalarMergeKind::Bool) => Lowered::Bool { value, known: None },
            Some(ScalarMergeKind::Int) => Lowered::Int { value, known: None },
            Some(ScalarMergeKind::StructuralNat) => {
                Lowered::StructuralNat(StructuralNatV1 { value })
            }
            Some(ScalarMergeKind::RecursiveBackedge) => {
                unreachable!("backedges do not establish a merge result kind")
            }
            None => unreachable!("HostResult emits both closed alternatives"),
        })
    }

    fn lower_bounded_nat_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let zero = cases
            .iter()
            .find(|case| case.constructor == self.process_symbols.nat_zero && case.binders == 0);
        let suc = cases
            .iter()
            .find(|case| case.constructor == self.process_symbols.nat_suc && case.binders == 1);
        let (Some(zero), Some(suc)) = (zero, suc) else {
            return Err(unsupported(
                "BoundedNat",
                "structural Nat match requires exact Zero and Suc predecessor arms",
            ));
        };
        let zero_block = builder.create_block();
        let suc_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);
        let mut merge_kind = None;
        for (block, case, predecessor) in [
            (zero_block, zero, None),
            (suc_block, suc, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let mut arm_env = predecessor
                .map(|predecessor| {
                    vec![if structural {
                        Lowered::StructuralNat(StructuralNatV1 {
                            value: predecessor.value,
                        })
                    } else {
                        Lowered::BoundedNat(predecessor)
                    }]
                })
                .unwrap_or_default();
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, kind) = self.merge_scalar_branch(builder, lowered, "BoundedNat")?;
            Self::record_scalar_merge_kind("BoundedNat", &mut merge_kind, kind)?;
            builder.ins().jump(merge, &[value.into()]);
        }
        builder.switch_to_block(merge);
        let value = builder.block_params(merge)[0];
        Ok(
            match merge_kind.expect("both structural Nat arms were emitted") {
                ScalarMergeKind::Int => Lowered::Int { value, known: None },
                ScalarMergeKind::Bool => Lowered::Bool { value, known: None },
                ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 { value }),
                ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus { value },
                ScalarMergeKind::RecursiveBackedge => {
                    unreachable!("backedges do not establish a merge result kind")
                }
            },
        )
    }

    fn lower_dynamic_constructor_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        continuation: DynamicConstructorContinuation<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;

        let (source_cases, source_default) = match continuation {
            DynamicConstructorContinuation::Ordinary { cases, default, .. }
            | DynamicConstructorContinuation::Producer { cases, default, .. } => (cases, default),
        };
        let has_selected_case = dynamic.alternatives.iter().any(|alternative| {
            source_cases
                .iter()
                .any(|case| case.constructor == alternative.constructor)
        });
        let merge = has_selected_case.then(|| {
            let merge = builder.create_block();
            builder.append_block_param(merge, types::I64);
            merge
        });
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor match block");
        let mut merge_kind = None;
        for alternative in dynamic.alternatives {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let (cases, default, env) = match continuation {
                DynamicConstructorContinuation::Ordinary {
                    cases,
                    default,
                    env,
                }
                | DynamicConstructorContinuation::Producer {
                    cases,
                    default,
                    env,
                    ..
                } => (cases, default, env),
            };
            let case = match select_dynamic_constructor_case(cases, &alternative, default)? {
                Ok(case) => case,
                Err(_owned_default) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let arm_env = materialize_dynamic_constructor_env(&alternative, env);
            let lowered = match continuation {
                DynamicConstructorContinuation::Ordinary { .. } => {
                    self.lower_expr(builder, &case.body, &arm_env)?
                }
                DynamicConstructorContinuation::Producer { eliminators, .. } => self
                    .lower_computational_producer_expr(
                        builder,
                        &case.body,
                        &arm_env,
                        eliminators,
                    )?,
            };
            let (value, branch_kind) =
                self.merge_scalar_branch(builder, lowered, "DynamicConstructor")?;
            Self::record_scalar_merge_kind("DynamicConstructor", &mut merge_kind, branch_kind)?;
            builder.ins().jump(
                merge.expect("a selected dynamic constructor case owns the merge"),
                &[value.into()],
            );
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(Lowered::Trap(source_default.clone()));
        };
        builder.switch_to_block(merge);
        let value = builder.block_params(merge)[0];
        Ok(match merge_kind {
            Some(ScalarMergeKind::ExitCode) => Lowered::ProcessExitStatus { value },
            Some(ScalarMergeKind::Bool) => Lowered::Bool { value, known: None },
            Some(ScalarMergeKind::Int) => Lowered::Int { value, known: None },
            Some(ScalarMergeKind::StructuralNat) => {
                Lowered::StructuralNat(StructuralNatV1 { value })
            }
            Some(ScalarMergeKind::RecursiveBackedge) => {
                unreachable!("backedges do not establish a merge result kind")
            }
            None => unreachable!("a selected dynamic constructor case emits one arm"),
        })
    }

    fn lower_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeValue::Bool(value) => Ok(Lowered::Bool {
                value: builder.ins().iconst(types::I64, i64::from(*value)),
                known: Some(*value),
            }),
            RuntimeValue::Int(value) => Ok(Lowered::Int {
                value: builder.ins().iconst(types::I64, *value),
                known: Some(*value),
            }),
            RuntimeValue::Bytes(value) => Ok(Lowered::Bytes(value.clone())),
            RuntimeValue::String(value) => Ok(Lowered::String(value.clone())),
            RuntimeValue::Constructor { constructor, args } => Ok(Lowered::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeValue::Record { fields } => Ok(Lowered::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), self.lower_value(builder, value)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
            RuntimeValue::ClosureRef { .. } => Err(unsupported(
                "ClosureRef",
                "pre-existing closure references are not lowered by the native backend",
            )),
            RuntimeValue::Unknown => Err(unsupported(
                "Unknown",
                "unknown runtime values must reject before backend lowering",
            )),
        }
    }

    fn lower_seed_capture(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &str,
    ) -> Result<Lowered, CraneliftBackendError> {
        let value = self.seed_env.values.get(symbol).ok_or_else(|| {
            unsupported(
                "Closure",
                format!("capture {symbol} has no runtime value in the seed environment"),
            )
        })?;
        self.lower_ground_value(builder, value)
    }

    fn lower_ground_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeGroundValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeGroundValue::Bool(value) => Ok(Lowered::Bool {
                value: builder.ins().iconst(types::I64, i64::from(*value)),
                known: Some(*value),
            }),
            RuntimeGroundValue::Int(value) => Ok(Lowered::Int {
                value: builder.ins().iconst(types::I64, *value),
                known: Some(*value),
            }),
            RuntimeGroundValue::Bytes(value) => Ok(Lowered::Bytes(value.clone())),
            RuntimeGroundValue::String(value) => Ok(Lowered::String(value.clone())),
            RuntimeGroundValue::Constructor { constructor, args } => Ok(Lowered::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_ground_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeGroundValue::Record { fields } => Ok(Lowered::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        Ok((name.clone(), self.lower_ground_value(builder, value)?))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
        }
    }

    fn lower_primitive_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        primitive: &RuntimePrimitive,
        args: &[RuntimeExpr],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let lowered_args = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, env))
            .collect::<Result<Vec<_>, _>>()?;
        if lowered_args
            .iter()
            .any(|arg| matches!(arg, Lowered::RecursiveBackedge))
        {
            return Ok(Lowered::RecursiveBackedge);
        }

        match &primitive.partiality {
            RuntimePartiality::Total => {}
            RuntimePartiality::SafeOption { .. } | RuntimePartiality::SafeResult { .. } => {}
            RuntimePartiality::CheckedTrap { obligation } => {
                self.assumptions.insert(format!(
                    "checked partial obligation {obligation} not discharged"
                ));
                let message = if obligation.ends_with(".bounds") {
                    format!("{} bounds obligation failed", primitive.symbol)
                } else {
                    format!("{} checked partiality trapped", primitive.symbol)
                };
                return Ok(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message,
                }));
            }
            RuntimePartiality::TrustedTrap { assumption } => {
                self.assumptions.insert(format!(
                    "trusted partial assumption {assumption} remains visible"
                ));
                return Ok(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: format!("{} trusted partiality trapped", primitive.symbol),
                }));
            }
        }

        match primitive.symbol.as_str() {
            "add_int" => self.lower_int_binop(
                builder,
                "add_int",
                lowered_args,
                |builder, lhs, rhs| builder.ins().iadd(lhs, rhs),
                |lhs, rhs| lhs.checked_add(rhs),
            ),
            "sub_int" => self.lower_int_binop(
                builder,
                "sub_int",
                lowered_args,
                |builder, lhs, rhs| builder.ins().isub(lhs, rhs),
                |lhs, rhs| lhs.checked_sub(rhs),
            ),
            "mul_int" => self.lower_int_binop(
                builder,
                "mul_int",
                lowered_args,
                |builder, lhs, rhs| builder.ins().imul(lhs, rhs),
                |lhs, rhs| lhs.checked_mul(rhs),
            ),
            "eq_int" => self.lower_int_cmp(
                builder,
                "eq_int",
                lowered_args,
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                |lhs, rhs| lhs == rhs,
            ),
            "leq_int" => self.lower_int_cmp(
                builder,
                "leq_int",
                lowered_args,
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                |lhs, rhs| lhs <= rhs,
            ),
            "uint8_to_int" | "int_to_uint8_raw" => {
                let [value]: [Lowered; 1] = lowered_args.try_into().map_err(|args: Vec<_>| {
                    unsupported(
                        "PrimitiveCall",
                        format!(
                            "{} expects one argument, got {}",
                            primitive.symbol,
                            args.len()
                        ),
                    )
                })?;
                let Lowered::Int { .. } = value else {
                    return Err(unsupported(
                        "PrimitiveCall",
                        format!("{} expects an Int-represented value", primitive.symbol),
                    ));
                };
                Ok(value)
            }
            "not_bool" => self.lower_bool_not(builder, lowered_args),
            "and_bool" => self.lower_bool_binop(
                builder,
                "and_bool",
                lowered_args,
                |builder, lhs, rhs| builder.ins().band(lhs, rhs),
                |lhs, rhs| lhs && rhs,
            ),
            "or_bool" => self.lower_bool_binop(
                builder,
                "or_bool",
                lowered_args,
                |builder, lhs, rhs| builder.ins().bor(lhs, rhs),
                |lhs, rhs| lhs || rhs,
            ),
            "bytes_length" => self.lower_bytes_length(builder, lowered_args),
            "bytes_at" => self.lower_bytes_at(builder, lowered_args, &primitive.partiality),
            "bytes_slice" => self.lower_bytes_slice(lowered_args, &primitive.partiality),
            "bytes_concat" => self.lower_bytes_concat(lowered_args),
            "bytes_encode" => self.lower_bytes_encode(lowered_args),
            "bytes_decode" => self.lower_bytes_decode(lowered_args, &primitive.partiality),
            "list_char_to_string" => {
                let [value]: [Lowered; 1] = lowered_args.try_into().map_err(|args: Vec<_>| {
                    unsupported(
                        "PrimitiveCall",
                        format!(
                            "list_char_to_string expects one argument, got {}",
                            args.len()
                        ),
                    )
                })?;
                let bytes = lowered_char_list(&value).ok_or_else(|| {
                    unsupported(
                        "PrimitiveCall",
                        "list_char_to_string requires a closed List Char",
                    )
                })?;
                let value = String::from_utf8(bytes).map_err(|_| {
                    unsupported(
                        "PrimitiveCall",
                        "list_char_to_string received non-UTF-8 Char values",
                    )
                })?;
                Ok(Lowered::String(value))
            }
            "byte_length" => self.lower_string_byte_length(builder, lowered_args),
            "char_length" => self.lower_string_char_length(builder, lowered_args),
            other => Err(unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the supported native set"),
            )),
        }
    }

    fn lower_int_binop(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        emit: impl FnOnce(
            &mut FunctionBuilder<'_>,
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        ) -> cranelift_codegen::ir::Value,
        eval: impl FnOnce(i64, i64) -> Option<i64>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args(symbol, args)?;
        let (
            Lowered::Int {
                value: lhs,
                known: lhs_known,
            },
            Lowered::Int {
                value: rhs,
                known: rhs_known,
            },
        ) = (lhs, rhs)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                format!("{symbol} only supports Int arguments in native lowering"),
            ));
        };
        let known = lhs_known
            .and_then(|lhs| rhs_known.and_then(|rhs| eval(lhs, rhs)))
            .ok_or_else(|| {
                unsupported(
                    "PrimitiveCall",
                    format!(
                        "{symbol} requires statically known non-overflowing Int operands in native lowering"
                    ),
                )
            })?;
        Ok(Lowered::Int {
            value: emit(builder, lhs, rhs),
            known: Some(known),
        })
    }

    fn lower_int_cmp(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        cc: cranelift_codegen::ir::condcodes::IntCC,
        eval: impl FnOnce(i64, i64) -> bool,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args(symbol, args)?;
        let (
            Lowered::Int {
                value: lhs,
                known: lhs_known,
            },
            Lowered::Int {
                value: rhs,
                known: rhs_known,
            },
        ) = (lhs, rhs)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                format!("{symbol} only supports Int arguments in native lowering"),
            ));
        };
        let cmp = builder.ins().icmp(cc, lhs, rhs);
        let value = builder.ins().uextend(types::I64, cmp);
        Ok(Lowered::Bool {
            value,
            known: lhs_known.and_then(|lhs| rhs_known.map(|rhs| eval(lhs, rhs))),
        })
    }

    fn lower_bool_not(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("not_bool expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::Bool { value, known } = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "not_bool only supports Bool arguments in native lowering",
            ));
        };
        let one = builder.ins().iconst(types::I64, 1);
        Ok(Lowered::Bool {
            value: builder.ins().bxor(value, one),
            known: known.map(|value| !value),
        })
    }

    fn lower_bool_binop(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        emit: impl FnOnce(
            &mut FunctionBuilder<'_>,
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        ) -> cranelift_codegen::ir::Value,
        eval: impl FnOnce(bool, bool) -> bool,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args(symbol, args)?;
        let (
            Lowered::Bool {
                value: lhs,
                known: lhs_known,
            },
            Lowered::Bool {
                value: rhs,
                known: rhs_known,
            },
        ) = (lhs, rhs)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                format!("{symbol} only supports Bool arguments in native lowering"),
            ));
        };
        Ok(Lowered::Bool {
            value: emit(builder, lhs, rhs),
            known: lhs_known.and_then(|lhs| rhs_known.map(|rhs| eval(lhs, rhs))),
        })
    }

    fn lower_bytes_length(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_length expects 1 arg, got {}", args.len()),
            )
        })?;
        if let Lowered::ResponseBytes { len, .. } = arg {
            return Ok(Lowered::Int {
                value: len,
                known: None,
            });
        }
        if let Lowered::BorrowedNativeValue { pointer } = arg {
            let kind = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 0);
            Self::require_i64(builder, kind, 1);
            let len = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 24);
            return Ok(Lowered::Int {
                value: len,
                known: None,
            });
        }
        let Lowered::Bytes(bytes) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_length only supports Bytes arguments in native lowering",
            ));
        };
        let len = i64::try_from(bytes.len()).map_err(|_| {
            unsupported(
                "PrimitiveCall",
                "bytes_length result does not fit the runtime Int representation",
            )
        })?;
        Ok(Lowered::Int {
            value: builder.ins().iconst(types::I64, len),
            known: Some(len),
        })
    }

    fn lower_bytes_at(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
        partiality: &RuntimePartiality,
    ) -> Result<Lowered, CraneliftBackendError> {
        let RuntimePartiality::SafeOption { none, some, .. } = partiality else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_at requires safe Option result metadata",
            ));
        };
        let (bytes, index) = expect_two_args("bytes_at", args)?;
        let Lowered::Int {
            known: Some(index), ..
        } = index
        else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_at requires a statically known Int index",
            ));
        };
        if let Lowered::ResponseBytes { pointer: data, len } = bytes {
            let index_value = builder.ins().iconst(types::I64, index);
            let present = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                index_value,
                len,
            );
            let in_bounds = builder.create_block();
            let out_of_bounds = builder.create_block();
            let merge = builder.create_block();
            builder.append_block_param(merge, types::I64);
            builder.append_block_param(merge, types::I64);
            builder
                .ins()
                .brif(present, in_bounds, &[], out_of_bounds, &[]);
            builder.switch_to_block(in_bounds);
            let address = builder.ins().iadd_imm(data, index);
            let byte = builder
                .ins()
                .load(types::I8, MemFlags::trusted(), address, 0);
            let yes = builder.ins().iconst(types::I64, 1);
            let byte = builder.ins().uextend(types::I64, byte);
            builder.ins().jump(merge, &[yes.into(), byte.into()]);
            builder.switch_to_block(out_of_bounds);
            let no = builder.ins().iconst(types::I64, 0);
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().jump(merge, &[no.into(), zero.into()]);
            builder.switch_to_block(merge);
            return Ok(Lowered::BorrowedOption {
                present: builder.block_params(merge)[0],
                value: builder.block_params(merge)[1],
                none: none.clone(),
                some: some.clone(),
            });
        }
        if let Lowered::BorrowedNativeValue { pointer } = bytes {
            let kind = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 0);
            Self::require_i64(builder, kind, 1);
            let pointer_type = builder.func.dfg.value_type(pointer);
            let data = builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), pointer, 16);
            let len = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 24);
            let index_value = builder.ins().iconst(types::I64, index);
            let present = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                index_value,
                len,
            );
            let in_bounds = builder.create_block();
            let out_of_bounds = builder.create_block();
            let merge = builder.create_block();
            builder.append_block_param(merge, types::I64);
            builder
                .ins()
                .brif(present, in_bounds, &[], out_of_bounds, &[]);
            builder.switch_to_block(in_bounds);
            Self::require_nonzero(builder, data);
            let address = builder.ins().iadd_imm(data, index);
            let byte = builder
                .ins()
                .load(types::I8, MemFlags::trusted(), address, 0);
            let byte = builder.ins().uextend(types::I64, byte);
            builder.ins().jump(merge, &[byte.into()]);
            builder.switch_to_block(out_of_bounds);
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().jump(merge, &[zero.into()]);
            builder.switch_to_block(merge);
            return Ok(Lowered::BorrowedOption {
                present,
                value: builder.block_params(merge)[0],
                none: none.clone(),
                some: some.clone(),
            });
        }
        let Lowered::Bytes(bytes) = bytes else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_at requires Bytes in native lowering",
            ));
        };
        let byte = usize::try_from(index)
            .ok()
            .and_then(|index| bytes.get(index).copied());
        Ok(match byte {
            Some(byte) => Lowered::Constructor {
                constructor: some.clone(),
                args: vec![Lowered::Int {
                    value: builder.ins().iconst(types::I64, i64::from(byte)),
                    known: Some(i64::from(byte)),
                }],
            },
            None => Lowered::Constructor {
                constructor: none.clone(),
                args: Vec::new(),
            },
        })
    }

    fn lower_bytes_slice(
        &mut self,
        args: Vec<Lowered>,
        partiality: &RuntimePartiality,
    ) -> Result<Lowered, CraneliftBackendError> {
        let RuntimePartiality::SafeOption { none, some, .. } = partiality else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_slice requires safe Option result metadata",
            ));
        };
        let [bytes, start, len]: [Lowered; 3] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_slice expects 3 args, got {}", args.len()),
            )
        })?;
        let (
            Lowered::Bytes(bytes),
            Lowered::Int {
                known: Some(start), ..
            },
            Lowered::Int {
                known: Some(len), ..
            },
        ) = (bytes, start, len)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_slice requires Bytes and statically known Int bounds",
            ));
        };
        let value = usize::try_from(start)
            .ok()
            .zip(usize::try_from(len).ok())
            .and_then(|(start, len)| {
                start
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .map(|end| bytes[start..end].to_vec())
            });
        Ok(match value {
            Some(bytes) => Lowered::Constructor {
                constructor: some.clone(),
                args: vec![Lowered::Bytes(bytes)],
            },
            None => Lowered::Constructor {
                constructor: none.clone(),
                args: Vec::new(),
            },
        })
    }

    fn lower_bytes_concat(&mut self, args: Vec<Lowered>) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args("bytes_concat", args)?;
        let (Lowered::Bytes(mut lhs), Lowered::Bytes(rhs)) = (lhs, rhs) else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_concat only supports Bytes arguments in native lowering",
            ));
        };
        lhs.extend(rhs);
        Ok(Lowered::Bytes(lhs))
    }

    fn lower_bytes_encode(&mut self, args: Vec<Lowered>) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_encode expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::String(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_encode only supports String arguments in native lowering",
            ));
        };
        Ok(Lowered::Bytes(value.into_bytes()))
    }

    fn lower_bytes_decode(
        &mut self,
        args: Vec<Lowered>,
        partiality: &RuntimePartiality,
    ) -> Result<Lowered, CraneliftBackendError> {
        let RuntimePartiality::SafeResult { err, ok, error } = partiality else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_decode requires safe Result metadata",
            ));
        };
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_decode expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::Bytes(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_decode only supports Bytes arguments in native lowering",
            ));
        };
        Ok(match String::from_utf8(value) {
            Ok(value) => Lowered::Constructor {
                constructor: ok.clone(),
                args: vec![Lowered::String(value)],
            },
            Err(_) => Lowered::Constructor {
                constructor: err.clone(),
                args: vec![Lowered::Constructor {
                    constructor: error.clone(),
                    args: Vec::new(),
                }],
            },
        })
    }

    fn lower_string_byte_length(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("byte_length expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::String(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "byte_length only supports String arguments in native lowering",
            ));
        };
        let len = i64::try_from(value.len()).map_err(|_| {
            unsupported(
                "PrimitiveCall",
                "byte_length result does not fit the runtime Int representation",
            )
        })?;
        Ok(Lowered::Int {
            value: builder.ins().iconst(types::I64, len),
            known: Some(len),
        })
    }

    fn lower_string_char_length(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("char_length expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::String(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "char_length only supports String arguments in native lowering",
            ));
        };
        let len = i64::try_from(value.chars().count()).map_err(|_| {
            unsupported(
                "PrimitiveCall",
                "char_length result does not fit the runtime Int representation",
            )
        })?;
        Ok(Lowered::Int {
            value: builder.ins().iconst(types::I64, len),
            known: Some(len),
        })
    }

    fn emit_result(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, ResultDecoder), CraneliftBackendError> {
        if self.process_object {
            let value = match value {
                Lowered::ProcessExitStatus { value } => value,
                value => self.emit_process_exit_status(builder, value),
            };
            return Ok((value, ResultDecoder::Int));
        }
        match value {
            Lowered::Int { value, .. } => Ok((value, ResultDecoder::Int)),
            Lowered::Bool { value, .. } => Ok((value, ResultDecoder::Bool)),
            value => {
                let ground = self.ground_value(value)?;
                let token = self.intern_result(ground);
                Ok((
                    builder.ins().iconst(types::I64, token),
                    ResultDecoder::Table,
                ))
            }
        }
    }

    fn emit_process_exit_status(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
    ) -> cranelift_codegen::ir::Value {
        let Lowered::Constructor { constructor, args } = value else {
            return builder.ins().iconst(types::I64, -2);
        };
        if constructor == self.process_symbols.exit_success {
            return if args.is_empty() {
                builder.ins().iconst(types::I64, 0)
            } else {
                builder.ins().iconst(types::I64, -2)
            };
        }
        if constructor != self.process_symbols.exit_failure {
            return builder.ins().iconst(types::I64, -2);
        }
        let Ok([payload]) = <Vec<Lowered> as TryInto<[Lowered; 1]>>::try_into(args) else {
            return builder.ins().iconst(types::I64, -3);
        };
        let Lowered::Int { value, known } = payload else {
            return builder.ins().iconst(types::I64, -3);
        };
        if let Some(code) = known {
            let mapping = crate::process_exit_status(crate::ProcessExitCode::Failure(code));
            return builder.ins().iconst(
                types::I64,
                if mapping.trap_report.is_some() {
                    -3
                } else {
                    i64::from(mapping.status)
                },
            );
        }
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let max = builder.ins().iconst(types::I64, 255);
        let malformed = builder.ins().iconst(types::I64, -3);
        let is_zero =
            builder
                .ins()
                .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, value, zero);
        let positive = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::SignedGreaterThan,
            value,
            zero,
        );
        let within_max = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
            value,
            max,
        );
        let valid = builder.ins().band(positive, within_max);
        let nonzero = builder.ins().select(valid, value, malformed);
        builder.ins().select(is_zero, one, nonzero)
    }

    fn ground_value(
        &mut self,
        value: Lowered,
    ) -> Result<RuntimeGroundValue, CraneliftBackendError> {
        match value {
            Lowered::Int {
                known: Some(value), ..
            } => Ok(RuntimeGroundValue::Int(value)),
            Lowered::Int { known: None, .. } => Err(unsupported(
                "Result",
                "native aggregate result contains a non-constant Int field",
            )),
            Lowered::Bool {
                known: Some(value), ..
            } => Ok(RuntimeGroundValue::Bool(value)),
            Lowered::Bool { known: None, .. } => Err(unsupported(
                "Result",
                "native aggregate result contains a non-constant Bool field",
            )),
            Lowered::ProcessExitStatus { .. } => Err(unsupported(
                "Result",
                "process exit status cannot escape a native process call",
            )),
            Lowered::Bytes(value) => Ok(RuntimeGroundValue::Bytes(value)),
            Lowered::BorrowedNativeValue { .. }
            | Lowered::BorrowedOption { .. }
            | Lowered::ResponseBytes { .. }
            | Lowered::CapabilityToken { .. }
            | Lowered::ResourceToken { .. }
            | Lowered::BoundedNat(_)
            | Lowered::StructuralNat(_)
            | Lowered::HostResult { .. }
            | Lowered::DynamicConstructor(_) => Err(unsupported(
                "Result",
                "borrowed ingress values cannot escape the native call",
            )),
            Lowered::String(value) => Ok(RuntimeGroundValue::String(value)),
            Lowered::Constructor { constructor, args } => Ok(RuntimeGroundValue::Constructor {
                constructor,
                args: args
                    .into_iter()
                    .map(|arg| self.ground_value(arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            Lowered::Record { fields } => Ok(RuntimeGroundValue::Record {
                fields: fields
                    .into_iter()
                    .map(|(name, value)| Ok((name, self.ground_value(value)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
            Lowered::Closure { .. } | Lowered::DeclarationClosure { .. } => Err(unsupported(
                "Closure",
                "closures are callable but not observable ground values in native lowering",
            )),
            Lowered::ComputationalRecursorClosure { .. } => Err(unsupported(
                "ComputationalMatch",
                "recursive hypotheses are callable but not observable ground values",
            )),
            Lowered::RecursiveBackedge => Err(unsupported(
                "DeclarationRef",
                "a recursive CFG edge cannot escape as a ground value",
            )),
            Lowered::Trap(trap) => Err(unsupported(
                "Trap",
                format!("trap result must be reported as trapped: {}", trap.message),
            )),
        }
    }

    fn intern_result(&mut self, ground: RuntimeGroundValue) -> i64 {
        let token = self.next_token;
        self.next_token += 1;
        self.result_table.insert(token, ground);
        token
    }
}

fn same_recursive_argument_shapes(left: &[Lowered], right: &[Lowered]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (Lowered::Int { .. }, Lowered::Int { .. })
                | (Lowered::Bool { .. }, Lowered::Bool { .. })
                | (Lowered::ProcessExitStatus { .. }, Lowered::ProcessExitStatus { .. })
                | (Lowered::CapabilityToken { .. }, Lowered::CapabilityToken { .. })
                | (Lowered::ResourceToken { .. }, Lowered::ResourceToken { .. })
                | (Lowered::BoundedNat(_), Lowered::BoundedNat(_))
                | (Lowered::StructuralNat(_), Lowered::StructuralNat(_))
                | (Lowered::ResponseBytes { .. }, Lowered::ResponseBytes { .. })
                | (Lowered::BorrowedNativeValue { .. }, Lowered::BorrowedNativeValue { .. }) => {
                    true
                }
                (Lowered::Bytes(left), Lowered::Bytes(right)) => left == right,
                (Lowered::String(left), Lowered::String(right)) => left == right,
                (
                    Lowered::Constructor {
                        constructor: left_constructor,
                        args: left_args,
                    },
                    Lowered::Constructor {
                        constructor: right_constructor,
                        args: right_args,
                    },
                ) => {
                    left_constructor == right_constructor
                        && same_recursive_argument_shapes(left_args, right_args)
                }
                (Lowered::Record { fields: left }, Lowered::Record { fields: right }) => {
                    left.len() == right.len()
                        && left
                            .iter()
                            .zip(right)
                            .all(|((left_name, left), (right_name, right))| {
                                left_name == right_name
                                    && same_recursive_argument_shapes(
                                        std::slice::from_ref(left),
                                        std::slice::from_ref(right),
                                    )
                            })
                }
                _ => false,
            })
}

fn lowered_value_kind(value: &Lowered) -> &'static str {
    match value {
        Lowered::Int { .. } => "Int",
        Lowered::Bool { .. } => "Bool",
        Lowered::ProcessExitStatus { .. } => "ProcessExitStatus",
        Lowered::CapabilityToken { .. } => "CapabilityToken",
        Lowered::ResourceToken { .. } => "ResourceToken",
        Lowered::BoundedNat(_) => "BoundedNat",
        Lowered::StructuralNat(_) => "StructuralNat",
        Lowered::ResponseBytes { .. } => "ResponseBytes",
        Lowered::HostResult { .. } => "HostResult",
        Lowered::DynamicConstructor(_) => "DynamicConstructor",
        Lowered::Bytes(_) => "Bytes",
        Lowered::BorrowedNativeValue { .. } => "BorrowedNativeValue",
        Lowered::BorrowedOption { .. } => "BorrowedOption",
        Lowered::String(_) => "String",
        Lowered::Constructor { .. } => "Constructor",
        Lowered::Record { .. } => "Record",
        Lowered::Closure { .. } => "Closure",
        Lowered::DeclarationClosure { .. } => "DeclarationClosure",
        Lowered::ComputationalRecursorClosure { .. } => "ComputationalRecursorClosure",
        Lowered::RecursiveBackedge => "RecursiveBackedge",
        Lowered::Trap(_) => "Trap",
    }
}

fn append_recursive_argument_values(
    values: &[Lowered],
    output: &mut Vec<cranelift_codegen::ir::Value>,
) -> Result<(), CraneliftBackendError> {
    for value in values {
        match value {
            Lowered::Int { value, .. }
            | Lowered::Bool { value, .. }
            | Lowered::ProcessExitStatus { value }
            | Lowered::CapabilityToken { value }
            | Lowered::ResourceToken { value } => output.push(*value),
            Lowered::BoundedNat(nat) => output.push(nat.value),
            Lowered::StructuralNat(nat) => output.push(nat.value),
            Lowered::ResponseBytes { pointer, len } => {
                output.push(*pointer);
                output.push(*len);
            }
            Lowered::BorrowedNativeValue { pointer } => output.push(*pointer),
            Lowered::Bytes(_) | Lowered::String(_) => {}
            Lowered::Constructor { args, .. } => {
                append_recursive_argument_values(args, output)?;
            }
            Lowered::Record { fields } => {
                for (_, field) in fields {
                    append_recursive_argument_values(std::slice::from_ref(field), output)?;
                }
            }
            _ => {
                return Err(unsupported(
                    "DeclarationRef",
                    "recursive declaration argument has an unsupported native representation",
                ));
            }
        }
    }
    Ok(())
}

fn rebuild_recursive_argument(
    template: &Lowered,
    values: &mut impl Iterator<Item = cranelift_codegen::ir::Value>,
) -> Result<Lowered, CraneliftBackendError> {
    let next = |values: &mut dyn Iterator<Item = cranelift_codegen::ir::Value>| {
        values.next().ok_or_else(|| {
            unsupported(
                "DeclarationRef",
                "recursive declaration loop parameter shape is truncated",
            )
        })
    };
    Ok(match template {
        Lowered::Int { .. } => Lowered::Int {
            value: next(values)?,
            known: None,
        },
        Lowered::Bool { .. } => Lowered::Bool {
            value: next(values)?,
            known: None,
        },
        Lowered::ProcessExitStatus { .. } => Lowered::ProcessExitStatus {
            value: next(values)?,
        },
        Lowered::CapabilityToken { .. } => Lowered::CapabilityToken {
            value: next(values)?,
        },
        Lowered::ResourceToken { .. } => Lowered::ResourceToken {
            value: next(values)?,
        },
        Lowered::BoundedNat(_) => {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(next(values)?))
        }
        Lowered::StructuralNat(_) => Lowered::StructuralNat(StructuralNatV1 {
            value: next(values)?,
        }),
        Lowered::ResponseBytes { .. } => Lowered::ResponseBytes {
            pointer: next(values)?,
            len: next(values)?,
        },
        Lowered::BorrowedNativeValue { .. } => Lowered::BorrowedNativeValue {
            pointer: next(values)?,
        },
        Lowered::Bytes(bytes) => Lowered::Bytes(bytes.clone()),
        Lowered::String(string) => Lowered::String(string.clone()),
        Lowered::Constructor { constructor, args } => Lowered::Constructor {
            constructor: constructor.clone(),
            args: args
                .iter()
                .map(|arg| rebuild_recursive_argument(arg, values))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Lowered::Record { fields } => Lowered::Record {
            fields: fields
                .iter()
                .map(|(name, value)| Ok((name.clone(), rebuild_recursive_argument(value, values)?)))
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
        },
        _ => {
            return Err(unsupported(
                "DeclarationRef",
                "recursive declaration argument has an unsupported native representation",
            ));
        }
    })
}

fn expect_two_args(
    symbol: &'static str,
    args: Vec<Lowered>,
) -> Result<(Lowered, Lowered), CraneliftBackendError> {
    let [lhs, rhs]: [Lowered; 2] = args.try_into().map_err(|args: Vec<Lowered>| {
        unsupported(
            "PrimitiveCall",
            format!("{symbol} expects 2 args, got {}", args.len()),
        )
    })?;
    Ok((lhs, rhs))
}

fn borrowed_constructor_identity(
    symbols: &crate::NativeProcessSymbols,
    symbol: &str,
) -> Option<(i64, usize)> {
    if symbol == symbols.process_input {
        Some((1, 3))
    } else if symbol == symbols.list_nil {
        Some((2, 0))
    } else if symbol == symbols.list_cons {
        Some((3, 2))
    } else if symbol == symbols.prod {
        Some((4, 2))
    } else {
        None
    }
}

fn unsupported(construct: &'static str, reason: impl Into<String>) -> CraneliftBackendError {
    CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct,
        reason: reason.into(),
    })
}

fn backend(failure: BackendFailure) -> CraneliftBackendError {
    CraneliftBackendError::Backend(failure)
}

fn backend_module(reason: String) -> CraneliftBackendError {
    backend(BackendFailure::Module(reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evaluate_runtime_ir_example, nc5_seed_examples, ErasedExecutableCore,
        RuntimeArtifactValidationStage, RuntimeArtifactValidationTier, RuntimeAssumptionTrustKind,
        RuntimeAssumptionTrustMetadata, RuntimeDeclaration, RuntimeEffectsForeignAuditMetadata,
        RuntimeFieldStatus, RuntimeIrSeedEnvironment, RuntimeMatchCase, RuntimeMetadata,
        RuntimeSymbolMetadata,
    };

    #[test]
    fn px8n_bounded_nat_observes_exact_zero_successor_and_recursive_order() {
        assert_eq!(
            run_checked_bounded_nat_fixture(
                3,
                7,
                3,
                7,
                BoundedNatFixtureObservation::OrdinaryRemaining,
                BoundedNatLoweringMutation::Exact,
            )
            .unwrap(),
            10,
            "a zero remainder selects the structural Zero arm",
        );
        assert_eq!(
            run_checked_bounded_nat_fixture(
                3,
                7,
                5,
                7,
                BoundedNatFixtureObservation::OrdinaryCount,
                BoundedNatLoweringMutation::Exact,
            )
            .unwrap(),
            22,
            "Suc exposes predecessor 2 as a second structural successor",
        );
        assert_eq!(
            run_checked_bounded_nat_fixture(
                3,
                7,
                5,
                7,
                BoundedNatFixtureObservation::ComputationalCount,
                BoundedNatLoweringMutation::Exact,
            )
            .unwrap(),
            0,
            "the recursive Suc case consumes the ordered predecessor and retained IH",
        );
    }

    #[test]
    fn px8n_bounded_nat_rejects_zero_over_bound_misaligned_and_wrapping_progress() {
        for (count, start, length, reply_start) in [
            (0, 7, 5, 7),
            (6, 7, 5, 7),
            (3, 7, 5, 8),
            (3, u64::MAX - 1, 5, u64::MAX - 1),
        ] {
            assert_eq!(
                run_checked_bounded_nat_fixture(
                    count,
                    start,
                    length,
                    reply_start,
                    BoundedNatFixtureObservation::OrdinaryCount,
                    BoundedNatLoweringMutation::Exact,
                )
                .unwrap(),
                -1,
                "invalid checked-host progress returns before carrier mint observation",
            );
        }
    }

    #[test]
    fn px8n_decrement_and_raw_scalar_mutations_fail_the_structural_oracle() {
        let run = |mutation| {
            run_checked_bounded_nat_fixture(
                3,
                7,
                5,
                7,
                BoundedNatFixtureObservation::ComputationalCount,
                mutation,
            )
            .unwrap()
        };

        let exact = run(BoundedNatLoweringMutation::Exact);
        assert_eq!(exact, 0);
        assert_eq!(
            run(BoundedNatLoweringMutation::BrokenDecrement),
            -2,
            "the live production loop's test-only fuel guard detects nontermination",
        );
        assert_eq!(
            run(BoundedNatLoweringMutation::RawScalarPredecessor),
            1,
            "the live producer exposes the exact wrong result when its Suc binder receives the raw scalar",
        );
    }

    #[test]
    fn px8n_fs_write_at_arm_constructs_short_wrote_and_exact_no_progress() {
        let (short, fixture) = run_px8n_write_arm_fixture(PX8N_SHORT_WROTE);
        assert_eq!(fixture.malformed_request, 0);
        assert_eq!(fixture.call_index, 3);
        assert_eq!(
            short, 3,
            "Wrote 1 of 4 exposes predecessor Zero and remaining structural Nat 3",
        );

        let (zero, fixture) = run_px8n_write_arm_fixture(PX8N_ZERO_WRITE);
        assert_eq!(fixture.malformed_request, 0);
        assert_eq!(fixture.call_index, 3);
        assert_eq!(
            zero, 70,
            "zero write reaches exact ResourceError.NoProgress"
        );
    }

    #[test]
    fn px8n_fs_write_at_arm_rejects_over_bound_reply_before_observation() {
        let (result, fixture) = run_px8n_write_arm_fixture(PX8N_OVER_BOUND_WRITE);
        assert_eq!(fixture.malformed_request, 0);
        assert_eq!(fixture.call_index, 3);
        assert_eq!(
            result, -1,
            "Wrote 5 for an effective request of 4 rejects before a Nat is observable",
        );
    }

    #[test]
    fn px8n_fs_read_at_arm_distinguishes_eof_and_short_read_some() {
        let (eof, fixture) = run_px8n_read_arm_fixture(PX8N_READ_EOF);
        assert_eq!(fixture.malformed_request, 0);
        assert_eq!(fixture.call_index, 3);
        assert_eq!(eof, 10, "zero read constructs exact ReadEof");

        let (short, fixture) = run_px8n_read_arm_fixture(PX8N_SHORT_READ);
        assert_eq!(fixture.malformed_request, 0);
        assert_eq!(fixture.call_index, 3);
        assert_eq!(
            short, 12,
            "ReadSome 1 of 4 carries the same structural Nat 1 in BufferSpan",
        );
    }

    #[test]
    fn px8n_fs_read_at_arm_rejects_over_bound_span_before_observation() {
        let (result, fixture) = run_px8n_read_arm_fixture(PX8N_OVER_BOUND_READ);
        assert_eq!(fixture.malformed_request, 0);
        assert_eq!(fixture.call_index, 3);
        assert_eq!(
            result, -1,
            "ReadSome 5 for an effective request of 4 rejects before a Nat is observable",
        );
    }

    #[repr(C)]
    struct BorrowedFixtureValue {
        kind: u64,
        tag: u64,
        data: *const std::ffi::c_void,
        len: usize,
    }

    #[repr(C)]
    struct NativeInvocationFixture {
        process_input: *const BorrowedFixtureValue,
        host_context: *mut std::ffi::c_void,
        capability: u64,
    }

    #[repr(C)]
    struct Px8nHostReplyFixture {
        scenario: u64,
        call_index: u64,
        malformed_request: u64,
    }

    const PX8N_SHORT_WROTE: u64 = 0;
    const PX8N_ZERO_WRITE: u64 = 1;
    const PX8N_OVER_BOUND_WRITE: u64 = 2;
    const PX8N_SHORT_READ: u64 = 3;
    const PX8N_READ_EOF: u64 = 4;
    const PX8N_OVER_BOUND_READ: u64 = 5;

    extern "C" fn px8n_scripted_host_dispatch(
        invocation: *const std::ffi::c_void,
        operation: i64,
        request: *const std::ffi::c_void,
        request_size: i64,
        reply: *mut std::ffi::c_void,
    ) -> i64 {
        // SAFETY: this symbol is installed only into the test JIT below, which
        // supplies these exact call-scoped fixtures for one synchronous call.
        let invocation = unsafe { &*(invocation.cast::<NativeInvocationFixture>()) };
        // SAFETY: `host_context` points to the live fixture for the duration of
        // the compiled call and is never retained by the dispatcher.
        let fixture = unsafe { &mut *(invocation.host_context.cast::<Px8nHostReplyFixture>()) };
        let expected = if fixture.call_index < 2 {
            ken_host::HostOpV1::BufferAllocate
        } else if fixture.scenario >= PX8N_SHORT_READ {
            ken_host::HostOpV1::FsReadAt
        } else {
            ken_host::HostOpV1::FsWriteAt
        };
        if operation != expected as i64 {
            fixture.malformed_request = 1;
            return -1;
        }
        let wire = ken_host::host_effect_wire_layout_v1(expected)
            .expect("PX8-N scripted operation has a generated wire layout");
        if request_size != i64::from(wire.request_size) {
            fixture.malformed_request = 2;
            return -1;
        }
        let load = |offset: u32| {
            // SAFETY: each offset is generated from the target-C layout for
            // this exact request record and the lowering supplied its size.
            unsafe { *(request.cast::<u8>().add(offset as usize).cast::<u64>()) }
        };
        if expected == ken_host::HostOpV1::BufferAllocate {
            if load(wire.request_offsets[0]) != 8 {
                fixture.malformed_request = 3;
                return -1;
            }
        } else if [
            load(wire.request_offsets[0]),
            load(wire.request_offsets[1]),
            load(wire.request_offsets[2]),
            load(wire.request_offsets[3]),
            load(wire.request_offsets[4]),
        ] != [11, 22, 0, 7, 4]
        {
            fixture.malformed_request = 4;
            return -1;
        }
        // SAFETY: the reply pointer names the target-C-sized stack record
        // supplied by the compiled caller for this exact operation.
        unsafe { std::ptr::write_bytes(reply.cast::<u8>(), 0, wire.reply_size as usize) };
        let store = |offset: u32, value: u64| {
            // SAFETY: generated offsets are aligned u64 fields within the
            // zeroed reply record above.
            unsafe {
                *(reply.cast::<u8>().add(offset as usize).cast::<u64>()) = value;
            }
        };
        if expected == ken_host::HostOpV1::BufferAllocate {
            store(wire.reply_tag_offset, wire.reply_resource_tag);
            store(
                wire.reply_detail_offset,
                if fixture.call_index == 0 { 11 } else { 22 },
            );
        } else {
            match fixture.scenario {
                PX8N_SHORT_WROTE => {
                    store(wire.reply_tag_offset, wire.reply_write_progress_tag);
                    store(wire.reply_detail_offset, 1);
                }
                PX8N_ZERO_WRITE => {
                    store(wire.reply_tag_offset, wire.reply_resource_error_tag);
                    store(wire.reply_detail_offset, wire.resource_error_no_progress);
                }
                PX8N_OVER_BOUND_WRITE => {
                    store(wire.reply_tag_offset, wire.reply_write_progress_tag);
                    store(wire.reply_detail_offset, 5);
                }
                PX8N_SHORT_READ => {
                    store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                    store(wire.reply_detail_offset, 1);
                    store(wire.reply_bytes_len_offset, 7);
                }
                PX8N_READ_EOF => {
                    store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                }
                PX8N_OVER_BOUND_READ => {
                    store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                    store(wire.reply_detail_offset, 5);
                    store(wire.reply_bytes_len_offset, 7);
                }
                _ => return -1,
            }
        }
        fixture.call_index += 1;
        0
    }

    fn px8n_exact_nat(
        symbols: &crate::NativeProcessSymbols,
        nat: RuntimeExpr,
        depth: usize,
        exact: RuntimeExpr,
    ) -> RuntimeExpr {
        let mismatch = RuntimeExpr::Value(RuntimeValue::Int(99));
        let cases = if depth == 0 {
            vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.nat_zero.clone(),
                    binders: 0,
                    body: exact,
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.nat_suc.clone(),
                    binders: 1,
                    body: mismatch,
                },
            ]
        } else {
            vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.nat_zero.clone(),
                    binders: 0,
                    body: mismatch,
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.nat_suc.clone(),
                    binders: 1,
                    body: px8n_exact_nat(symbols, RuntimeExpr::Var(0), depth - 1, exact),
                },
            ]
        };
        RuntimeExpr::Match {
            scrutinee: Box::new(nat),
            cases,
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: format!("PX8-N expected exact structural Nat depth {depth}"),
            },
        }
    }

    fn px8n_failure(symbols: &crate::NativeProcessSymbols, code: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: symbols.exit_failure.clone(),
            args: vec![code],
        }
    }

    fn px8n_write_arm_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        let trap = || RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N checked result default".to_string(),
        };
        let allocate = || RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(8))],
        };
        let write = RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::FsWriteAt,
            capability: None,
            args: vec![
                RuntimeExpr::Var(1),
                RuntimeExpr::Value(RuntimeValue::Int(0)),
                RuntimeExpr::Var(0),
                RuntimeExpr::Value(RuntimeValue::Int(7)),
                RuntimeExpr::Value(RuntimeValue::Int(4)),
            ],
        };
        let transfer_observation = px8n_exact_nat(
            symbols,
            RuntimeExpr::Var(0),
            0,
            px8n_exact_nat(
                symbols,
                RuntimeExpr::Var(1),
                3,
                RuntimeExpr::Value(RuntimeValue::Int(3)),
            ),
        );
        let success = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: symbols.wrote.clone(),
                binders: 1,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![crate::RuntimeMatchCase {
                        constructor: symbols.private_transfer_count.clone(),
                        binders: 2,
                        body: px8n_failure(symbols, transfer_observation),
                    }],
                    default: trap(),
                },
            }],
            default: trap(),
        };
        let error = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: symbols.resource_no_progress.clone(),
                binders: 0,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(70))),
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-N expected exact NoProgress".to_string(),
            },
        };
        let write_result = RuntimeExpr::Match {
            scrutinee: Box::new(write),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: error,
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: success,
                },
            ],
            default: trap(),
        };
        let second = RuntimeExpr::Match {
            scrutinee: Box::new(allocate()),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(81))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: write_result,
                },
            ],
            default: trap(),
        };
        RuntimeExpr::Match {
            scrutinee: Box::new(allocate()),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(80))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: second,
                },
            ],
            default: trap(),
        }
    }

    fn px8n_read_arm_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        let trap = || RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N checked read result default".to_string(),
        };
        let allocate = || RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(8))],
        };
        let read = RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::FsReadAt,
            capability: None,
            args: vec![
                RuntimeExpr::Var(1),
                RuntimeExpr::Value(RuntimeValue::Int(0)),
                RuntimeExpr::Var(0),
                RuntimeExpr::Value(RuntimeValue::Int(7)),
                RuntimeExpr::Value(RuntimeValue::Int(4)),
            ],
        };
        let read_some = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: symbols.private_buffer_span.clone(),
                binders: 2,
                body: px8n_exact_nat(
                    symbols,
                    RuntimeExpr::Var(1),
                    1,
                    RuntimeExpr::Value(RuntimeValue::Int(12)),
                ),
            }],
            default: trap(),
        };
        let read_some = px8n_failure(symbols, read_some);
        let progress = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.read_some.clone(),
                    binders: 2,
                    body: read_some,
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.read_eof.clone(),
                    binders: 0,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(10))),
                },
            ],
            default: trap(),
        };
        let read_result = RuntimeExpr::Match {
            scrutinee: Box::new(read),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(82))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: progress,
                },
            ],
            default: trap(),
        };
        let second = RuntimeExpr::Match {
            scrutinee: Box::new(allocate()),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(81))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: read_result,
                },
            ],
            default: trap(),
        };
        RuntimeExpr::Match {
            scrutinee: Box::new(allocate()),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(80))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: second,
                },
            ],
            default: trap(),
        }
    }

    fn run_px8n_arm_fixture(
        scenario: u64,
        expression: fn(&crate::NativeProcessSymbols) -> RuntimeExpr,
    ) -> (i64, Px8nHostReplyFixture) {
        let isa = native_isa().unwrap();
        let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
        builder.symbol(
            "ken_host_dispatch_v1",
            px8n_scripted_host_dispatch as *const u8,
        );
        let symbols = crate::NativeProcessSymbols::legacy_prelude();
        let compiled = compile_expr_into_module(
            JITModule::new(builder),
            "px8n_fs_write_at",
            Linkage::Local,
            &expression(&symbols),
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            true,
            Some(&symbols),
        )
        .unwrap();
        let input = BorrowedFixtureValue {
            kind: 1,
            tag: 0,
            data: std::ptr::null(),
            len: 0,
        };
        let mut fixture = Px8nHostReplyFixture {
            scenario,
            call_index: 0,
            malformed_request: 0,
        };
        let invocation = NativeInvocationFixture {
            process_input: &input,
            host_context: (&mut fixture as *mut Px8nHostReplyFixture).cast(),
            capability: 0,
        };
        let (_, result) = compiled
            .run(Some((&invocation as *const NativeInvocationFixture).cast()))
            .unwrap();
        (result.unwrap(), fixture)
    }

    fn run_px8n_write_arm_fixture(scenario: u64) -> (i64, Px8nHostReplyFixture) {
        run_px8n_arm_fixture(scenario, px8n_write_arm_fixture)
    }

    fn run_px8n_read_arm_fixture(scenario: u64) -> (i64, Px8nHostReplyFixture) {
        run_px8n_arm_fixture(scenario, px8n_read_arm_fixture)
    }

    fn host_result_computational_fixture(
        ok_binders: usize,
        include_ok: bool,
        mismatched_result_kind: bool,
    ) -> RuntimeExpr {
        let result_ok = "ctor:prelude::Result::Ok".to_string();
        let result_err = "ctor:prelude::Result::Err".to_string();
        let scalar_tree = "ctor:fixture::Tree::Scalar".to_string();
        let exit_tree = "ctor:fixture::Tree::Exit".to_string();
        let mut producer_cases = vec![RuntimeMatchCase {
            constructor: result_err,
            binders: 1,
            body: RuntimeExpr::Construct {
                constructor: if mismatched_result_kind {
                    exit_tree.clone()
                } else {
                    scalar_tree.clone()
                },
                args: if mismatched_result_kind {
                    Vec::new()
                } else {
                    vec![RuntimeExpr::Value(RuntimeValue::Int(9))]
                },
            },
        }];
        if include_ok {
            producer_cases.push(RuntimeMatchCase {
                constructor: result_ok,
                binders: ok_binders,
                body: RuntimeExpr::Construct {
                    constructor: scalar_tree.clone(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(7))],
                },
            });
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Effect {
                    family: "Console".to_string(),
                    operation: ken_host::HostOpV1::ConsoleWrite,
                    capability: None,
                    args: vec![
                        RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Stream::Stdout".to_string(),
                            args: Vec::new(),
                        },
                        RuntimeExpr::Value(RuntimeValue::Bytes(b"probe".to_vec())),
                    ],
                }),
                cases: producer_cases,
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "dynamic Result producer default".to_string(),
                },
            }),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: scalar_tree,
                    argument_binders: 1,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Var(0),
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: exit_tree,
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "computational tree default".to_string(),
            },
        }
    }

    fn nested_computational_fixture(
        outer_binders: usize,
        inner_recursive_positions: Vec<usize>,
        mismatched_result_kind: bool,
        payload_is_int: bool,
    ) -> RuntimeExpr {
        let inner_true = "ctor:fixture::Inner::TrueLeaf".to_string();
        let inner_false = "ctor:fixture::Inner::FalseLeaf".to_string();
        let aggregate_ok = "ctor:fixture::Aggregate::Ok".to_string();
        let aggregate_err = "ctor:fixture::Aggregate::Err".to_string();
        let inner_cases = [
            (inner_true.clone(), aggregate_ok.clone()),
            (inner_false.clone(), aggregate_err.clone()),
        ]
        .into_iter()
        .map(
            |(constructor, aggregate)| crate::RuntimeComputationalMatchCase {
                constructor,
                argument_binders: 1,
                recursive_positions: inner_recursive_positions.clone(),
                body: RuntimeExpr::Construct {
                    constructor: aggregate,
                    args: vec![RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "sub_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
                    }],
                },
            },
        )
        .collect();
        let producer_cases = [
            ("ctor:prelude::Bool::True", inner_true, 7),
            ("ctor:prelude::Bool::False", inner_false, 9),
        ]
        .into_iter()
        .map(|(constructor, leaf, payload)| RuntimeMatchCase {
            constructor: constructor.to_string(),
            binders: 0,
            body: RuntimeExpr::Construct {
                constructor: leaf,
                args: vec![if payload_is_int {
                    RuntimeExpr::Value(RuntimeValue::Int(payload))
                } else {
                    RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                        args: Vec::new(),
                    }
                }],
            },
        })
        .collect();
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Value(RuntimeValue::Int(41))),
                body: Box::new(RuntimeExpr::ComputationalMatch {
                    scrutinee: Box::new(RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::Effect {
                            family: "Console".to_string(),
                            operation: ken_host::HostOpV1::ConsoleIsTerminal,
                            capability: None,
                            args: vec![RuntimeExpr::Construct {
                                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                                args: Vec::new(),
                            }],
                        }),
                        cases: producer_cases,
                        default: RuntimeTrap {
                            code: RuntimeTrapCode::PatternMatchFailure,
                            message: "inner producer default".to_string(),
                        },
                    }),
                    cases: inner_cases,
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "inner eliminator default".to_string(),
                    },
                }),
            }),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: aggregate_ok,
                    argument_binders: outer_binders,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Var(0),
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: aggregate_err,
                    argument_binders: 1,
                    recursive_positions: Vec::new(),
                    body: if mismatched_result_kind {
                        RuntimeExpr::Construct {
                            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                            args: Vec::new(),
                        }
                    } else {
                        RuntimeExpr::Var(0)
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "outer eliminator default".to_string(),
            },
        }
    }

    fn ordinary_match_closure(cases: Vec<RuntimeMatchCase>, default: RuntimeTrap) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["value".to_string()],
            body: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases,
                default,
            }),
        }
    }

    fn heterogeneous_eliminator_fixture(
        inner_constructor: &str,
        inner_case_constructor: &str,
        outer_constructor: &str,
        outer_case_constructor: &str,
        inner_binders: usize,
        outer_binders: usize,
        payload_is_int: bool,
        mismatched_result_kind: bool,
    ) -> RuntimeExpr {
        let inner_default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7o exact first ordinary default".to_string(),
        };
        let outer_default = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7o exact later ordinary default".to_string(),
        };
        let producer = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleIsTerminal,
                capability: None,
                args: vec![RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Stream::Stdout".to_string(),
                    args: Vec::new(),
                }],
            }),
            cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
                .into_iter()
                .map(|constructor| RuntimeMatchCase {
                    constructor: constructor.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: inner_constructor.to_string(),
                        args: vec![if payload_is_int {
                            RuntimeExpr::Value(RuntimeValue::Int(7))
                        } else {
                            RuntimeExpr::Construct {
                                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                args: Vec::new(),
                            }
                        }],
                    },
                })
                .collect(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "px7o producer default".to_string(),
            },
        };
        let inner_call = RuntimeExpr::Call {
            callee: Box::new(ordinary_match_closure(
                vec![RuntimeMatchCase {
                    constructor: inner_case_constructor.to_string(),
                    binders: inner_binders,
                    body: RuntimeExpr::Construct {
                        constructor: outer_constructor.to_string(),
                        args: vec![RuntimeExpr::Var(0)],
                    },
                }],
                inner_default,
            )),
            args: vec![producer],
        };
        RuntimeExpr::Call {
            callee: Box::new(ordinary_match_closure(
                vec![RuntimeMatchCase {
                    constructor: outer_case_constructor.to_string(),
                    binders: outer_binders,
                    body: if mismatched_result_kind {
                        RuntimeExpr::Construct {
                            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                            args: Vec::new(),
                        }
                    } else {
                        RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "sub_int".to_string(),
                                partiality: RuntimePartiality::Total,
                            },
                            args: vec![
                                RuntimeExpr::Value(RuntimeValue::Int(41)),
                                RuntimeExpr::Var(0),
                            ],
                        }
                    },
                }],
                outer_default,
            )),
            args: vec![inner_call],
        }
    }

    fn constructor_field_aggregate() -> RuntimeExpr {
        RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleIsTerminal,
                capability: None,
                args: vec![RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Stream::Stdout".to_string(),
                    args: Vec::new(),
                }],
            }),
            cases: [
                ("ctor:prelude::Bool::True", "ctor:prelude::Result::Ok", 7),
                ("ctor:prelude::Bool::False", "ctor:prelude::Result::Err", 9),
            ]
            .into_iter()
            .map(|(constructor, result, payload)| RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: result.to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(payload))],
                },
            })
            .collect(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "px7p aggregate producer default".to_string(),
            },
        }
    }

    fn constructor_field_selected_case_fixture(
        selected_binders: usize,
        selected_field_var: u32,
    ) -> RuntimeExpr {
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Int(41)),
                    constructor_field_aggregate(),
                ],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                argument_binders: selected_binders,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(selected_field_var)),
                    cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                        .into_iter()
                        .map(|constructor| RuntimeMatchCase {
                            constructor: constructor.to_string(),
                            binders: 1,
                            body: RuntimeExpr::PrimitiveCall {
                                primitive: RuntimePrimitive {
                                    symbol: "sub_int".to_string(),
                                    partiality: RuntimePartiality::Total,
                                },
                                args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
                            },
                        })
                        .collect(),
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7p selected field default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7p exact outer default".to_string(),
            },
        }
    }

    #[test]
    fn dynamic_host_result_producer_missing_case_routes_to_default() {
        assert!(
            dynamic_host_result_producer_case(&[], "ctor:prelude::Result::Ok")
                .expect("missing case is a fail-closed default route")
                .is_none()
        );
        emit_process_entrypoint_object_with_cranelift(
            &host_result_computational_fixture(1, false, true),
            "ken_px7m_missing_case_default",
        )
        .expect("the absent dynamic arm lowers through the producer default trap");
    }

    #[test]
    fn dynamic_host_result_producer_wrong_arity_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &host_result_computational_fixture(0, true, false),
            "ken_px7m_wrong_arity",
        )
        .expect_err("dynamic Result case must bind its one payload");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "dynamic HostResult tree producer case ctor:prelude::Result::Ok expects exactly one binder, got 0"
        ));
    }

    #[test]
    fn dynamic_host_result_producer_result_kind_mismatch_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &host_result_computational_fixture(1, true, true),
            "ken_px7m_kind_mismatch",
        )
        .expect_err("scalar and ExitCode branches must not merge");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "dynamic native arms disagree on scalar versus ExitCode result"
        ));
    }

    #[test]
    fn dynamic_host_result_producer_well_formed_control_emits() {
        emit_process_entrypoint_object_with_cranelift(
            &host_result_computational_fixture(1, true, false),
            "ken_px7m_well_formed",
        )
        .expect("both dynamic Result branches recursively lower and merge");
    }

    #[test]
    fn nested_computational_producer_well_formed_control_emits() {
        emit_process_entrypoint_object_with_cranelift(
            &nested_computational_fixture(1, Vec::new(), false, true),
            "ken_px7n_well_formed",
        )
        .expect("inner dynamic branches compose through the outer eliminator");
    }

    #[test]
    fn nested_computational_outer_arity_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &nested_computational_fixture(0, Vec::new(), false, true),
            "ken_px7n_wrong_outer_arity",
        )
        .expect_err("the outer aggregate payload must remain bound");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "case ctor:fixture::Aggregate::Ok expects 0 constructor arguments but value has 1"
        ));
    }

    #[test]
    fn nested_computational_malformed_recursive_position_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &nested_computational_fixture(1, vec![1], false, true),
            "ken_px7n_bad_recursive_position",
        )
        .expect_err("an out-of-range inner recursive position must fail closed");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "case ctor:fixture::Inner::TrueLeaf has malformed recursive position 1"
        ));
    }

    #[test]
    fn nested_computational_final_merge_kind_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &nested_computational_fixture(1, Vec::new(), true, true),
            "ken_px7n_final_kind_mismatch",
        )
        .expect_err("the final scalar and ExitCode arms must not merge");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "dynamic native arms disagree on scalar versus ExitCode result"
        ));
    }

    #[test]
    fn nested_computational_payload_kind_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &nested_computational_fixture(1, Vec::new(), false, false),
            "ken_px7n_payload_kind",
        )
        .expect_err("the inner aggregate payload must retain its scalar kind");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "PrimitiveCall",
                reason,
            }) if reason == "sub_int only supports Int arguments in native lowering"
        ));
    }

    #[test]
    fn heterogeneous_eliminator_well_formed_control_emits() {
        emit_process_entrypoint_object_with_cranelift(
            &heterogeneous_eliminator_fixture(
                "ctor:fixture::Inner::Hit",
                "ctor:fixture::Inner::Hit",
                "ctor:fixture::Outer::Hit",
                "ctor:fixture::Outer::Hit",
                1,
                1,
                true,
                false,
            ),
            "ken_px7o_well_formed",
        )
        .expect("dynamic producer composes through both ordinary frames");
    }

    #[test]
    fn constructor_field_selected_case_composes_before_field_lowering() {
        emit_process_entrypoint_object_with_cranelift(
            &constructor_field_selected_case_fixture(2, 1),
            "ken_px7p_constructor_field_selected_case",
        )
        .expect("the selected trailing field remains structural through its ordinary consumer");
    }

    #[test]
    fn constructor_field_composes_through_computational_consumer() {
        let leaf = "ctor:fixture::FieldTree::Leaf".to_string();
        let field = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleIsTerminal,
                capability: None,
                args: vec![RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Stream::Stdout".to_string(),
                    args: Vec::new(),
                }],
            }),
            cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
                .into_iter()
                .map(|constructor| RuntimeMatchCase {
                    constructor: constructor.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: leaf.clone(),
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int(7))],
                    },
                })
                .collect(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "px7p computational field default".to_string(),
            },
        };
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(41)), field],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                argument_binders: 2,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::ComputationalMatch {
                    scrutinee: Box::new(RuntimeExpr::Var(1)),
                    cases: vec![crate::RuntimeComputationalMatchCase {
                        constructor: leaf,
                        argument_binders: 1,
                        recursive_positions: Vec::new(),
                        body: RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "sub_int".to_string(),
                                partiality: RuntimePartiality::Total,
                            },
                            args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
                        },
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7p computational consumer default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7p computational outer default".to_string(),
            },
        };
        emit_process_entrypoint_object_with_cranelift(
            &expr,
            "ken_px7p_constructor_field_computational_consumer",
        )
        .expect("the selected field also composes through a computational consumer");
    }

    #[test]
    fn constructor_field_recursive_ih_offset_selects_argument_binder() {
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Recursive".to_string(),
                args: vec![constructor_field_aggregate()],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Recursive".to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(1)),
                    cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                        .into_iter()
                        .map(|constructor| RuntimeMatchCase {
                            constructor: constructor.to_string(),
                            binders: 1,
                            body: RuntimeExpr::Var(0),
                        })
                        .collect(),
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7p recursive selected-field default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7p recursive outer default".to_string(),
            },
        };
        emit_process_entrypoint_object_with_cranelift(
            &expr,
            "ken_px7p_constructor_field_recursive_offset",
        )
        .expect("the recursive IH prefix does not change the selected argument field");
    }

    #[test]
    fn constructor_field_middle_binder_preserves_trailing_environment_order() {
        let aggregate = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
                .into_iter()
                .map(|constructor| RuntimeMatchCase {
                    constructor: constructor.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Result::Ok".to_string(),
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int(7))],
                    },
                })
                .collect(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "px7p middle producer default".to_string(),
            },
        };
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Int(13)),
                    aggregate,
                    RuntimeExpr::Value(RuntimeValue::Int(41)),
                ],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                argument_binders: 3,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(1)),
                    cases: vec![RuntimeMatchCase {
                        constructor: "ctor:prelude::Result::Ok".to_string(),
                        binders: 1,
                        body: RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "sub_int".to_string(),
                                partiality: RuntimePartiality::Total,
                            },
                            args: vec![RuntimeExpr::Var(3), RuntimeExpr::Var(0)],
                        },
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7p middle consumer default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7p middle outer default".to_string(),
            },
        };
        let compiled = compile_expr(&expr, &NativeSeedEnvironment::empty())
            .expect("the selected middle field composes without moving its trailing sibling");
        assert_eq!(
            compiled.run(None).expect("middle-field fixture runs").0,
            RuntimeObservation::Returned(RuntimeGroundValue::Int(34))
        );
    }

    #[test]
    fn constructor_field_binder_shift_mutation_recovers_exact_refusal() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &constructor_field_selected_case_fixture(2, 0),
            "ken_px7p_constructor_field_wrong_binder",
        )
        .expect_err("the aggregate-looking sibling is not the selected field consumer");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Match",
                reason,
            }) if reason == "scrutinee is not a constructor value"
        ));
    }

    #[test]
    fn constructor_field_bridge_removal_recovers_exact_refusal() {
        let fixture = constructor_field_selected_case_fixture(2, 1);
        let RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default,
        } = fixture
        else {
            panic!("PX7-P fixture outer shape changed");
        };
        let eagerly_materialized = RuntimeExpr::Let {
            value: scrutinee,
            body: Box::new(RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases,
                default,
            }),
        };
        let err = emit_process_entrypoint_object_with_cranelift(
            &eagerly_materialized,
            "ken_px7p_constructor_field_bridge_removed",
        )
        .expect_err("eager field lowering must recover the pre-PX7-P boundary");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Match",
                reason,
            }) if reason == "scrutinee is not a constructor value"
        ));
    }

    #[test]
    fn constructor_field_outer_arity_rejects_before_field_lowering() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &constructor_field_selected_case_fixture(1, 1),
            "ken_px7p_constructor_field_outer_arity",
        )
        .expect_err("the selected constructor case must bind every field exactly");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "case ctor:fixture::Envelope::Wrap expects 1 constructor arguments but value has 2"
        ));
    }

    #[test]
    fn constructor_field_missing_case_owns_default_before_fields() {
        let default = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p exact missing constructor default".to_string(),
        };
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Missing".to_string(),
                args: vec![RuntimeExpr::Var(999)],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Var(0),
            }],
            default: default.clone(),
        };
        let compiled = compile_expr(&expr, &NativeSeedEnvironment::empty())
            .expect("a missing constructor selects its frame-owned default");
        assert_eq!(
            compiled.run(None).expect("default trap is observable").0,
            RuntimeObservation::Trapped(default)
        );
    }

    #[test]
    fn constructor_field_aggregate_unconsumed_sibling_stays_ordinary() {
        let prefix = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Prefix::Keep".to_string(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(41))],
        };
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                args: vec![prefix, constructor_field_aggregate()],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                argument_binders: 2,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![RuntimeMatchCase {
                        constructor: "ctor:fixture::Prefix::Keep".to_string(),
                        binders: 1,
                        body: RuntimeExpr::Var(0),
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7p prefix default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7p outer default".to_string(),
            },
        };
        emit_process_entrypoint_object_with_cranelift(
            &expr,
            "ken_px7p_aggregate_unconsumed_sibling",
        )
        .expect("an unconsumed aggregate-looking field retains ordinary lowering");
    }

    #[test]
    fn constructor_field_host_result_stays_on_ordinary_dynamic_match() {
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                args: vec![console_write_effect()],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Envelope::Wrap".to_string(),
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                        .into_iter()
                        .map(|constructor| RuntimeMatchCase {
                            constructor: constructor.to_string(),
                            binders: 1,
                            body: RuntimeExpr::Construct {
                                constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                                args: Vec::new(),
                            },
                        })
                        .collect(),
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7p HostResult default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7p outer default".to_string(),
            },
        };
        emit_process_entrypoint_object_with_cranelift(
            &expr,
            "ken_px7p_constructor_field_host_result",
        )
        .expect("HostResult fields remain owned by ordinary dynamic matching");
    }

    fn host_result_closure_match(argument: RuntimeExpr) -> RuntimeExpr {
        let exit_success = || RuntimeExpr::Construct {
            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
            args: Vec::new(),
        };
        RuntimeExpr::Call {
            callee: Box::new(ordinary_match_closure(
                vec![
                    RuntimeMatchCase {
                        constructor: "ctor:prelude::Result::Err".to_string(),
                        binders: 1,
                        body: exit_success(),
                    },
                    RuntimeMatchCase {
                        constructor: "ctor:prelude::Result::Ok".to_string(),
                        binders: 1,
                        body: exit_success(),
                    },
                ],
                RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "direct HostResult default".to_string(),
                },
            )),
            args: vec![argument],
        }
    }

    fn console_write_effect() -> RuntimeExpr {
        RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleWrite,
            capability: None,
            args: vec![
                RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Stream::Stdout".to_string(),
                    args: Vec::new(),
                },
                RuntimeExpr::Value(RuntimeValue::Bytes(b"probe".to_vec())),
            ],
        }
    }

    fn fs_read_effect() -> RuntimeExpr {
        RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::FsReadFile,
            capability: Some(crate::RuntimeCapabilityUse {
                identity: "program_caps.fs".to_string(),
                value: Box::new(RuntimeExpr::Var(1)),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Bytes(
                b"dynamic-constructor.bin".to_vec(),
            ))],
        }
    }

    fn dynamic_io_error_match(producer: bool, ordinary_bool: bool) -> RuntimeExpr {
        let symbols = crate::NativeProcessSymbols::legacy_prelude();
        let tree = "ctor:fixture::DynamicConstructorTree::Code";
        let producer_tree = |code: RuntimeExpr| RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleIsTerminal,
                capability: None,
                args: vec![RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Stream::Stdout".to_string(),
                    args: Vec::new(),
                }],
            }),
            cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
                .into_iter()
                .map(|constructor| RuntimeMatchCase {
                    constructor: constructor.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: tree.to_string(),
                        args: vec![code.clone()],
                    },
                })
                .collect(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "dynamic constructor producer default".to_string(),
            },
        };
        let io_cases = symbols
            .io_errors
            .iter()
            .enumerate()
            .map(|(tag, constructor)| {
                let binders = usize::from(tag + 1 == symbols.io_errors.len());
                let code = if binders == 1 {
                    RuntimeExpr::Var(0)
                } else {
                    RuntimeExpr::Value(RuntimeValue::Int(tag as i64 + 1))
                };
                RuntimeMatchCase {
                    constructor: constructor.clone(),
                    binders,
                    body: if producer {
                        producer_tree(code)
                    } else if ordinary_bool {
                        RuntimeExpr::Value(RuntimeValue::Bool(tag % 2 == 0))
                    } else {
                        RuntimeExpr::Construct {
                            constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                            args: vec![code],
                        }
                    },
                }
            })
            .collect();
        let error = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![RuntimeMatchCase {
                constructor: symbols.file_error.clone(),
                binders: 3,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(2)),
                    cases: io_cases,
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "dynamic IOError match default".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "dynamic FileError match default".to_string(),
            },
        };
        let result = RuntimeExpr::Match {
            scrutinee: Box::new(fs_read_effect()),
            cases: vec![
                RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: error,
                },
                RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: if producer {
                        RuntimeExpr::Construct {
                            constructor: tree.to_string(),
                            args: vec![RuntimeExpr::Value(RuntimeValue::Int(0))],
                        }
                    } else if ordinary_bool {
                        RuntimeExpr::Value(RuntimeValue::Bool(false))
                    } else {
                        RuntimeExpr::Construct {
                            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                            args: Vec::new(),
                        }
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "dynamic Result match default".to_string(),
            },
        };
        if producer {
            RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(result),
                cases: vec![crate::RuntimeComputationalMatchCase {
                    constructor: tree.to_string(),
                    argument_binders: 1,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                        args: vec![RuntimeExpr::Var(0)],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: "dynamic producer consumer default".to_string(),
                },
            }
        } else if ordinary_bool {
            RuntimeExpr::Match {
                scrutinee: Box::new(result),
                cases: [
                    ("ctor:prelude::Bool::True", crate::EXIT_SUCCESS_CONSTRUCTOR),
                    ("ctor:prelude::Bool::False", crate::EXIT_FAILURE_CONSTRUCTOR),
                ]
                .into_iter()
                .map(|(constructor, exit)| RuntimeMatchCase {
                    constructor: constructor.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: exit.to_string(),
                        args: (exit == crate::EXIT_FAILURE_CONSTRUCTOR)
                            .then(|| RuntimeExpr::Value(RuntimeValue::Int(1)))
                            .into_iter()
                            .collect(),
                    },
                })
                .collect(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "dynamic Bool consumer default".to_string(),
                },
            }
        } else {
            result
        }
    }

    #[test]
    fn dynamic_constructor_dispatches_ordinary_continuation_with_mixed_arities() {
        emit_process_entrypoint_object_with_cranelift(
            &dynamic_io_error_match(false, false),
            "ken_px7p_dynamic_constructor_ordinary",
        )
        .expect("the shared dispatcher lowers ordinary nullary and unary alternatives");
    }

    #[test]
    fn dynamic_constructor_dispatches_producer_continuation_with_all_frames() {
        emit_process_entrypoint_object_with_cranelift(
            &dynamic_io_error_match(true, false),
            "ken_px7p_dynamic_constructor_producer",
        )
        .expect("the shared dispatcher preserves the active computational frame");
    }

    #[test]
    fn dynamic_constructor_ordinary_continuation_preserves_bool_kind() {
        emit_process_entrypoint_object_with_cranelift(
            &dynamic_io_error_match(false, true),
            "ken_px7p_dynamic_constructor_bool",
        )
        .expect("a dynamic Bool remains available to its enclosing Bool consumer");
    }

    #[test]
    fn dynamic_constructor_duplicate_tag_and_identity_reject_exactly() {
        let duplicate_tag = validate_dynamic_constructor_alternatives([
            (0, "ctor:fixture::Dynamic::A"),
            (0, "ctor:fixture::Dynamic::B"),
        ])
        .expect_err("closed alternatives require unique tags");
        assert!(matches!(
            duplicate_tag,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "DynamicConstructor",
                reason,
            }) if reason == "duplicate alternative tag 0"
        ));

        let duplicate_identity = validate_dynamic_constructor_alternatives([
            (0, "ctor:fixture::Dynamic::A"),
            (1, "ctor:fixture::Dynamic::A"),
        ])
        .expect_err("closed alternatives require unique constructor identities");
        assert!(matches!(
            duplicate_identity,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "DynamicConstructor",
                reason,
            }) if reason == "duplicate alternative constructor ctor:fixture::Dynamic::A"
        ));
    }

    #[test]
    fn dynamic_constructor_fields_precede_outer_environment_in_declaration_order() {
        let alternative = DynamicConstructorAlternativeV1 {
            tag: 7,
            constructor: "ctor:fixture::Dynamic::Pair".to_string(),
            fields: vec![
                Lowered::Bytes(b"first".to_vec()),
                Lowered::String("second".to_string()),
            ],
        };
        let env =
            materialize_dynamic_constructor_env(&alternative, &[Lowered::Bytes(b"outer".to_vec())]);
        assert!(matches!(&env[0], Lowered::Bytes(value) if value == b"first"));
        assert!(matches!(&env[1], Lowered::String(value) if value == "second"));
        assert!(matches!(&env[2], Lowered::Bytes(value) if value == b"outer"));
    }

    #[test]
    fn dynamic_constructor_known_omission_owns_source_default() {
        let alternative = DynamicConstructorAlternativeV1 {
            tag: 0,
            constructor: "ctor:fixture::Dynamic::Missing".to_string(),
            fields: Vec::new(),
        };
        let owned = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "exact source match default".to_string(),
        };
        let unrelated = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "unrelated outer default".to_string(),
        };
        let selected = select_dynamic_constructor_case(&[], &alternative, &owned)
            .expect("a well-formed omission selects the source default")
            .expect_err("the constructor is intentionally omitted");
        assert_eq!(selected, &owned);
        assert_ne!(selected, &unrelated);
    }

    #[test]
    fn dynamic_constructor_all_known_omitted_runs_source_default_without_panic() {
        assert_eq!(
            run_dynamic_constructor_dispatch_fixture(0, &[])
                .expect("all-omitted dispatcher executes"),
            -4
        );
        assert_eq!(
            run_dynamic_constructor_dispatch_fixture(1, &[])
                .expect("every known alternative owns the source default"),
            -4
        );
    }

    #[test]
    fn dynamic_constructor_mixed_present_and_omitted_keeps_default_distinct() {
        assert_eq!(
            run_dynamic_constructor_dispatch_fixture(0, &[1])
                .expect("known omitted tag executes the source default"),
            -4
        );
        assert_eq!(
            run_dynamic_constructor_dispatch_fixture(1, &[1])
                .expect("present unary alternative executes its selected case"),
            41
        );
    }

    #[test]
    fn dynamic_constructor_unknown_tag_runs_malformed_not_source_default() {
        let malformed = run_dynamic_constructor_dispatch_fixture(2, &[])
            .expect("unknown-tag dispatcher executes");
        assert_eq!(malformed, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        assert_eq!(malformed, -3);
        assert_ne!(malformed, -4);
    }

    #[test]
    fn dynamic_constructor_binder_arity_rejects_exactly() {
        let mut symbols = crate::NativeProcessSymbols::legacy_prelude();
        symbols.io_errors.rotate_right(1);
        let err = emit_process_entrypoint_object_with_symbols(
            &dynamic_io_error_match(false, false),
            &symbols,
            "ken_px7p_dynamic_constructor_arity",
        )
        .expect_err("constructor identity, not table position, owns binder arity");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "DynamicConstructor",
                reason,
            }) if reason == "case ctor:prelude::IOError::Other expects 1 binders but alternative has 0 fields"
        ));
    }

    #[test]
    fn direct_host_result_closure_match_keeps_established_dynamic_lane() {
        emit_process_entrypoint_object_with_cranelift(
            &host_result_closure_match(console_write_effect()),
            "ken_px7o_direct_host_result_closure_match",
        )
        .expect("direct HostResult remains owned by ordinary dynamic matching");
    }

    #[test]
    fn call_returned_host_result_keeps_established_dynamic_lane() {
        let effect_call = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["ignored".to_string()],
                body: Box::new(console_write_effect()),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(0))],
        };

        emit_process_entrypoint_object_with_cranelift(
            &host_result_closure_match(effect_call),
            "ken_px7o_call_returned_host_result_closure_match",
        )
        .expect("call-returned HostResult remains owned by ordinary dynamic matching");
    }

    #[test]
    fn match_selected_call_returned_host_result_keeps_established_dynamic_lane() {
        let effect_call = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["ignored".to_string()],
                body: Box::new(RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Bool::True".to_string(),
                        args: Vec::new(),
                    }),
                    cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
                        .into_iter()
                        .map(|constructor| RuntimeMatchCase {
                            constructor: constructor.to_string(),
                            binders: 0,
                            body: console_write_effect(),
                        })
                        .collect(),
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "static Bool default".to_string(),
                    },
                }),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(0))],
        };

        emit_process_entrypoint_object_with_cranelift(
            &host_result_closure_match(effect_call),
            "ken_px7o_match_selected_call_returned_host_result",
        )
        .expect("match-selected HostResult remains owned by ordinary dynamic matching");
    }

    fn recursive_computational_result(leaf_body: RuntimeExpr) -> RuntimeExpr {
        let node = "ctor:fixture::RecursiveTree::Node";
        let leaf = "ctor:fixture::RecursiveTree::Leaf";
        let recursive_child = RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["unit".to_string()],
            body: Box::new(RuntimeExpr::Construct {
                constructor: leaf.to_string(),
                args: Vec::new(),
            }),
        };
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: node.to_string(),
                args: vec![recursive_child],
            }),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: node.to_string(),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                            args: Vec::new(),
                        }],
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: leaf.to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: leaf_body,
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "recursive tree default".to_string(),
            },
        }
    }

    #[test]
    fn recursive_computational_host_result_keeps_established_dynamic_lane() {
        emit_process_entrypoint_object_with_cranelift(
            &host_result_closure_match(recursive_computational_result(console_write_effect())),
            "ken_px7o_recursive_computational_host_result",
        )
        .expect("recursive computational HostResult remains on ordinary dynamic matching");
    }

    #[test]
    fn recursive_computational_aggregate_traverses_ordinary_frame() {
        let aggregate = RuntimeExpr::Construct {
            constructor: "ctor:prelude::Result::Ok".to_string(),
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                args: Vec::new(),
            }],
        };

        emit_process_entrypoint_object_with_cranelift(
            &host_result_closure_match(recursive_computational_result(aggregate)),
            "ken_px7o_recursive_computational_aggregate",
        )
        .expect("recursive aggregate traverses the active ordinary frame");
    }

    #[test]
    fn heterogeneous_bridge_removal_recovers_exact_ordinary_match_refusal() {
        let fixture = heterogeneous_eliminator_fixture(
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Outer::Hit",
            "ctor:fixture::Outer::Hit",
            1,
            1,
            true,
            false,
        );
        let RuntimeExpr::Call { callee, mut args } = fixture else {
            panic!("fixture outer shape changed");
        };
        let RuntimeExpr::LexicalClosure { body, .. } = *callee else {
            panic!("fixture continuation shape changed");
        };
        let bridge_removed = RuntimeExpr::Let {
            value: Box::new(args.remove(0)),
            body,
        };
        let err = emit_process_entrypoint_object_with_cranelift(
            &bridge_removed,
            "ken_px7o_bridge_removed",
        )
        .expect_err("eagerly materializing the intermediate must recover the original defect");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Match",
                reason,
            }) if reason == "scrutinee is not a constructor value"
        ));
    }

    #[test]
    fn heterogeneous_frame_environment_and_binder_order_are_preserved() {
        let inner_call = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: vec![RuntimeExpr::Value(RuntimeValue::Int(41))],
                params: vec!["inner".to_string()],
                body: Box::new(RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![RuntimeMatchCase {
                        constructor: "ctor:fixture::Inner::Hit".to_string(),
                        binders: 1,
                        body: RuntimeExpr::Construct {
                            constructor: "ctor:fixture::Outer::Hit".to_string(),
                            args: vec![RuntimeExpr::PrimitiveCall {
                                primitive: RuntimePrimitive {
                                    symbol: "sub_int".to_string(),
                                    partiality: RuntimePartiality::Total,
                                },
                                args: vec![RuntimeExpr::Var(2), RuntimeExpr::Var(0)],
                            }],
                        },
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "px7o binder-order inner default".to_string(),
                    },
                }),
            }),
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:fixture::Inner::Hit".to_string(),
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(7))],
            }],
        };
        let expr = RuntimeExpr::Call {
            callee: Box::new(ordinary_match_closure(
                vec![RuntimeMatchCase {
                    constructor: "ctor:fixture::Outer::Hit".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                }],
                RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: "px7o binder-order outer default".to_string(),
                },
            )),
            args: vec![inner_call],
        };
        let compiled = compile_expr(&expr, &NativeSeedEnvironment::empty())
            .expect("frame environment fixture lowers");
        assert_eq!(
            compiled
                .run(None)
                .expect("frame environment fixture runs")
                .0,
            RuntimeObservation::Returned(RuntimeGroundValue::Int(34))
        );
    }

    #[test]
    fn heterogeneous_final_merge_kind_rejects_specifically() {
        let producer = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleIsTerminal,
                capability: None,
                args: vec![RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Stream::Stdout".to_string(),
                    args: Vec::new(),
                }],
            }),
            cases: vec![
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Bool::True".to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Inner::Scalar".to_string(),
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int(7))],
                    },
                },
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Bool::False".to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Inner::Exit".to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "px7o kind producer default".to_string(),
            },
        };
        let inner_call = RuntimeExpr::Call {
            callee: Box::new(ordinary_match_closure(
                vec![
                    RuntimeMatchCase {
                        constructor: "ctor:fixture::Inner::Scalar".to_string(),
                        binders: 1,
                        body: RuntimeExpr::Construct {
                            constructor: "ctor:fixture::Outer::Scalar".to_string(),
                            args: vec![RuntimeExpr::Var(0)],
                        },
                    },
                    RuntimeMatchCase {
                        constructor: "ctor:fixture::Inner::Exit".to_string(),
                        binders: 0,
                        body: RuntimeExpr::Construct {
                            constructor: "ctor:fixture::Outer::Exit".to_string(),
                            args: Vec::new(),
                        },
                    },
                ],
                RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7o kind inner default".to_string(),
                },
            )),
            args: vec![producer],
        };
        let expr = RuntimeExpr::Call {
            callee: Box::new(ordinary_match_closure(
                vec![
                    RuntimeMatchCase {
                        constructor: "ctor:fixture::Outer::Scalar".to_string(),
                        binders: 1,
                        body: RuntimeExpr::Var(0),
                    },
                    RuntimeMatchCase {
                        constructor: "ctor:fixture::Outer::Exit".to_string(),
                        binders: 0,
                        body: RuntimeExpr::Construct {
                            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                            args: Vec::new(),
                        },
                    },
                ],
                RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: "px7o kind outer default".to_string(),
                },
            )),
            args: vec![inner_call],
        };
        let err =
            emit_process_entrypoint_object_with_cranelift(&expr, "ken_px7o_final_kind_mismatch")
                .expect_err("final scalar and ExitCode arms must not merge");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalMatch",
                reason,
            }) if reason == "dynamic native arms disagree on scalar versus ExitCode result"
        ));
    }

    #[test]
    fn heterogeneous_first_ordinary_missing_selects_exact_default() {
        let first_cases = vec![RuntimeMatchCase {
            constructor: "ctor:fixture::Inner::Hit".to_string(),
            binders: 1,
            body: RuntimeExpr::Value(RuntimeValue::Int(1)),
        }];
        let first_default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7o exact first ordinary default".to_string(),
        };
        let later_default = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7o exact later ordinary default".to_string(),
        };
        let trap = select_ordinary_case(
            OrdinaryEliminatorFrame {
                cases: &first_cases,
                default: &first_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
            },
            "ctor:fixture::Inner::Missing",
        )
        .expect_err("the first ordinary frame must select its own default");
        assert_eq!(trap, first_default);
        assert_ne!(trap, later_default);
    }

    #[test]
    fn heterogeneous_later_ordinary_missing_selects_exact_default() {
        let later_cases = vec![RuntimeMatchCase {
            constructor: "ctor:fixture::Outer::Hit".to_string(),
            binders: 1,
            body: RuntimeExpr::Value(RuntimeValue::Int(1)),
        }];
        let first_default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7o exact first ordinary default".to_string(),
        };
        let later_default = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7o exact later ordinary default".to_string(),
        };
        let trap = select_ordinary_case(
            OrdinaryEliminatorFrame {
                cases: &later_cases,
                default: &later_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
            },
            "ctor:fixture::Outer::Missing",
        )
        .expect_err("the later ordinary frame must select its own default");
        assert_eq!(trap, later_default);
        assert_ne!(trap, first_default);
    }

    #[test]
    fn heterogeneous_ordinary_arity_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &heterogeneous_eliminator_fixture(
                "ctor:fixture::Inner::Hit",
                "ctor:fixture::Inner::Hit",
                "ctor:fixture::Outer::Hit",
                "ctor:fixture::Outer::Hit",
                0,
                1,
                true,
                false,
            ),
            "ken_px7o_wrong_arity",
        )
        .expect_err("ordinary frame binder arity must match the constructor");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Match",
                reason,
            }) if reason == "case ctor:fixture::Inner::Hit expects 0 binders but constructor has 1 args"
        ));
    }

    #[test]
    fn heterogeneous_nested_payload_kind_rejects_specifically() {
        let err = emit_process_entrypoint_object_with_cranelift(
            &heterogeneous_eliminator_fixture(
                "ctor:fixture::Inner::Hit",
                "ctor:fixture::Inner::Hit",
                "ctor:fixture::Outer::Hit",
                "ctor:fixture::Outer::Hit",
                1,
                1,
                false,
                false,
            ),
            "ken_px7o_payload_kind",
        )
        .expect_err("the nested aggregate payload must retain its scalar kind");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "PrimitiveCall",
                reason,
            }) if reason == "sub_int only supports Int arguments in native lowering"
        ));
    }

    #[test]
    fn nested_computational_inner_missing_selects_exact_inner_default() {
        let inner_cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Inner::Hit".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Value(RuntimeValue::Int(1)),
        }];
        let outer_cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Outer::Hit".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Value(RuntimeValue::Int(2)),
        }];
        let inner_default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7n exact inner default".to_string(),
        };
        let outer_default = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7n exact outer default".to_string(),
        };
        let frames = [
            ComputationalEliminatorFrame {
                cases: &inner_cases,
                default: &inner_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: RecursorFrameProvenance(1),
            },
            ComputationalEliminatorFrame {
                cases: &outer_cases,
                default: &outer_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: RecursorFrameProvenance(0),
            },
        ];

        let trap = match select_computational_case(&frames, "ctor:fixture::Inner::Missing") {
            Err(trap) => trap,
            Ok(_) => panic!("a missing inner case must select the inner frame default"),
        };
        assert_eq!(trap.code, RuntimeTrapCode::PatternMatchFailure);
        assert_eq!(trap.message, "px7n exact inner default");
        assert_ne!(trap.code, outer_default.code);
        assert_ne!(trap.message, outer_default.message);
    }

    #[test]
    fn nested_computational_outer_missing_selects_exact_outer_default() {
        let inner_cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Inner::Hit".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Value(RuntimeValue::Int(1)),
        }];
        let outer_cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Outer::Hit".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Value(RuntimeValue::Int(2)),
        }];
        let inner_default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7n exact inner default".to_string(),
        };
        let outer_default = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7n exact outer default".to_string(),
        };
        let frames = [
            ComputationalEliminatorFrame {
                cases: &inner_cases,
                default: &inner_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: RecursorFrameProvenance(1),
            },
            ComputationalEliminatorFrame {
                cases: &outer_cases,
                default: &outer_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: RecursorFrameProvenance(0),
            },
        ];

        let (_, outer_frames) = select_computational_case(&frames, "ctor:fixture::Inner::Hit")
            .expect("the inner case succeeds before the outer miss");
        let trap = match select_computational_case(outer_frames, "ctor:fixture::Outer::Missing") {
            Err(trap) => trap,
            Ok(_) => panic!("a missing outer case must select the outer frame default"),
        };
        assert_eq!(trap.code, RuntimeTrapCode::ExplicitTrap);
        assert_eq!(trap.message, "px7n exact outer default");
        assert_ne!(trap.code, inner_default.code);
        assert_ne!(trap.message, inner_default.message);
    }

    #[test]
    fn live_effect_emitter_inventory_and_generated_layout_mutations_are_closed() {
        assert_eq!(
            CRANELIFT_HOST_EFFECT_CONSUMERS_V1,
            ken_host::NATIVE_TESTED_TARGETS_V1
        );
        for operation in CRANELIFT_HOST_EFFECT_CONSUMERS_V1 {
            let layout = ken_host::host_effect_wire_layout_v1(operation).unwrap();
            assert_eq!(
                ken_host::verify_host_effect_wire_layout_v1(operation, &layout),
                Ok(())
            );
            let mut mutations = Vec::new();
            let mut changed = layout.clone();
            changed.request_size ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.request_align_shift ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.request_offsets[0] ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_size ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_tag_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_error_tag ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_tag ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_schema_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_kind_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_identity_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_io_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_required_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_resource_error_held_offset ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.resource_error_closed ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.resource_error_malformed ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.resource_error_right_not_held ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.resource_error_release_failed ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.resource_kind_fs_handle ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.resource_error_reply_schema ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_unit_tag ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_bool_tag ^= 1;
            mutations.push(changed);
            let mut changed = layout.clone();
            changed.reply_bytes_tag ^= 1;
            mutations.push(changed);
            for mutation in mutations {
                assert!(ken_host::verify_host_effect_wire_layout_v1(operation, &mutation).is_err());
            }
        }
    }

    fn run_borrowed_fixture(expr: &RuntimeExpr, root: &BorrowedFixtureValue) -> i64 {
        let compiled = compile_expr_into_module(
            new_jit_module().expect("JIT module"),
            "px4_borrowed_fixture",
            Linkage::Local,
            expr,
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            true,
            None,
        )
        .expect("borrowed fixture lowers");
        let invocation = NativeInvocationFixture {
            process_input: root,
            host_context: std::ptr::null_mut(),
            capability: 1_u64 << 32,
        };
        compiled
            .run(Some((&invocation as *const NativeInvocationFixture).cast()))
            .expect("borrowed fixture runs")
            .1
            .expect("borrowed fixture returns scalar")
    }

    #[test]
    fn borrowed_ingress_malformed_metadata_fails_closed() {
        let malformed = BorrowedFixtureValue {
            kind: 99,
            tag: 1,
            data: std::ptr::null(),
            len: 3,
        };
        let expr = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![RuntimeMatchCase {
                constructor: crate::PROCESS_INPUT_CONSTRUCTOR.to_string(),
                binders: 3,
                body: RuntimeExpr::Value(RuntimeValue::Int(0)),
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "malformed process root".to_string(),
            },
        };
        assert_eq!(run_borrowed_fixture(&expr, &malformed), -1);
        let null_fields = BorrowedFixtureValue {
            kind: 2,
            tag: 1,
            data: std::ptr::null(),
            len: 3,
        };
        assert_eq!(run_borrowed_fixture(&expr, &null_fields), -1);
        let wrong_arity = BorrowedFixtureValue {
            kind: 2,
            tag: 1,
            data: (&malformed as *const BorrowedFixtureValue).cast(),
            len: 2,
        };
        assert_eq!(run_borrowed_fixture(&expr, &wrong_arity), -1);
        assert!(crate::object_linker_packaging::process_starter_c_stub()
            .contains("ken native trap: malformed borrowed process input"));
    }

    #[test]
    fn borrowed_ingress_bytes_at_preserves_safe_none_bounds() {
        let cwd = [0xff_u8];
        let fields = [
            BorrowedFixtureValue {
                kind: 2,
                tag: 2,
                data: std::ptr::null(),
                len: 0,
            },
            BorrowedFixtureValue {
                kind: 2,
                tag: 2,
                data: std::ptr::null(),
                len: 0,
            },
            BorrowedFixtureValue {
                kind: 1,
                tag: 0,
                data: cwd.as_ptr().cast(),
                len: cwd.len(),
            },
        ];
        let root = BorrowedFixtureValue {
            kind: 2,
            tag: 1,
            data: fields.as_ptr().cast(),
            len: 3,
        };
        let none = "ctor:fixture::Option::None";
        let some = "ctor:fixture::Option::Some";
        let expr = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![RuntimeMatchCase {
                constructor: crate::PROCESS_INPUT_CONSTRUCTOR.to_string(),
                binders: 3,
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                    args: vec![RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "bytes_at".to_string(),
                                partiality: RuntimePartiality::SafeOption {
                                    none: none.to_string(),
                                    some: some.to_string(),
                                    obligation: Some("obl:px4.bounds".to_string()),
                                },
                            },
                            args: vec![
                                RuntimeExpr::Var(2),
                                RuntimeExpr::Value(RuntimeValue::Int(99)),
                            ],
                        }),
                        cases: vec![
                            RuntimeMatchCase {
                                constructor: none.to_string(),
                                binders: 0,
                                body: RuntimeExpr::Value(RuntimeValue::Int(7)),
                            },
                            RuntimeMatchCase {
                                constructor: some.to_string(),
                                binders: 1,
                                body: RuntimeExpr::Value(RuntimeValue::Int(9)),
                            },
                        ],
                        default: RuntimeTrap {
                            code: RuntimeTrapCode::PatternMatchFailure,
                            message: "invalid bytes_at option".to_string(),
                        },
                    }],
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "invalid process input".to_string(),
            },
        };
        assert_eq!(run_borrowed_fixture(&expr, &root), 7);
    }

    fn seed_program_with_lowerability(status: Option<RuntimeLowerabilityStatus>) -> RuntimeProgram {
        let symbol = "decl:fixture::Main::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        if let Some(status) = status.clone() {
            metadata.lowerability.insert(symbol.clone(), status);
        }
        RuntimeProgram {
            package_identity: "module:fixture::nc6".to_string(),
            core_semantic_hash: 1,
            artifact_hash: 2,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol,
                kind: RuntimeDeclarationKind::Record {
                    fields: vec![crate::RuntimeField {
                        name: "value".to_string(),
                        status: RuntimeFieldStatus::Runtime,
                    }],
                },
                metadata: RuntimeSymbolMetadata {
                    lowerability: status,
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: nc5_seed_examples(),
        }
    }

    fn nc22_program_with_body(
        body: RuntimeExpr,
        observation: RuntimeObservation,
    ) -> RuntimeProgram {
        let symbol = "decl:fixture::Main::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::nc22".to_string(),
            core_semantic_hash: 22,
            artifact_hash: 2200,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol: symbol.clone(),
                kind: RuntimeDeclarationKind::Transparent { body },
                metadata: RuntimeSymbolMetadata {
                    lowerability: Some(RuntimeLowerabilityStatus::Supported),
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: vec![RuntimeExample {
                name: "main-entrypoint".to_string(),
                checked_core_shape: "compiler-produced declaration ref".to_string(),
                ir: RuntimeExpr::DeclarationRef { symbol },
                observation,
            }],
        }
    }

    fn total_primitive(symbol: &str, args: Vec<RuntimeExpr>) -> RuntimeExpr {
        RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: symbol.to_string(),
                partiality: RuntimePartiality::Total,
            },
            args,
        }
    }

    #[test]
    fn cranelift_runs_scalar_seed_and_verifies_function() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closed-scalar-primitive")
            .expect("seed exists");

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("native run succeeds");

        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);
        assert_eq!(
            report.trust.fidelity,
            NativeFidelity::F1SeedObservationAgreement
        );
    }

    #[test]
    fn recursive_declaration_shape_change_hits_typed_boundary() {
        let symbol = "decl:fixture::Loop::run".to_string();
        let declaration = RuntimeDeclaration {
            symbol: symbol.clone(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::Closure {
                    captures: Vec::new(),
                    params: vec!["state".to_string()],
                    body: Box::new(RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::DeclarationRef {
                            symbol: symbol.clone(),
                        }),
                        args: vec![RuntimeExpr::Construct {
                            constructor: "ctor:fixture::Option::Some".to_string(),
                            args: vec![RuntimeExpr::Value(RuntimeValue::Int(1))],
                        }],
                    }),
                },
            },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        };
        let entry = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: symbol.clone(),
            }),
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:fixture::Option::None".to_string(),
                args: Vec::new(),
            }],
        };
        let declarations = BTreeMap::from([(symbol.as_str(), &declaration)]);
        let result = compile_expr_into_module(
            new_object_module("px8l-recursive-shape").unwrap(),
            "ken_px8l_recursive_shape",
            Linkage::Export,
            &entry,
            &NativeSeedEnvironment::empty(),
            declarations,
            None,
            true,
            None,
        );
        let error = match result {
            Ok(_) => panic!("a changing recursive native representation must fail closed"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "DeclarationRef",
                reason,
            }) if reason.contains("changes its native argument representation")
        ));
    }

    #[test]
    fn cranelift_runs_constructor_match_and_record_projection_seeds() {
        let env = NativeSeedEnvironment::empty();
        for name in ["adt-constructor-match", "record-construction-projection"] {
            let example = nc5_seed_examples()
                .into_iter()
                .find(|example| example.name == name)
                .expect("seed exists");

            let report =
                run_example_with_seed_observation(&example, &env).expect("native run succeeds");

            assert!(report.verifier_passed);
            assert_eq!(report.observation, example.observation);
        }
    }

    #[test]
    fn cranelift_reports_bytes_and_string_immediates_as_ground_values() {
        for (name, ir, observation) in [
            (
                "bytes-immediate",
                RuntimeExpr::Value(RuntimeValue::Bytes(vec![1, 2, 3])),
                RuntimeObservation::Returned(RuntimeGroundValue::Bytes(vec![1, 2, 3])),
            ),
            (
                "string-immediate",
                RuntimeExpr::Value(RuntimeValue::String("ken".to_string())),
                RuntimeObservation::Returned(RuntimeGroundValue::String("ken".to_string())),
            ),
        ] {
            let example = RuntimeExample {
                name: name.to_string(),
                checked_core_shape: "diagnostic label only".to_string(),
                ir,
                observation,
            };

            let report =
                run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
                    .expect("native run succeeds");

            assert!(report.verifier_passed);
            assert_eq!(report.observation, example.observation);
            assert!(
                report.native_returned.is_some(),
                "native function returns an opaque table token"
            );
        }
    }

    #[test]
    fn cranelift_runs_closure_seed_with_explicit_runtime_capture_environment() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closure-capture-application")
            .expect("seed exists");

        let report =
            run_example_with_seed_observation(&example, &NativeSeedEnvironment::nc5_seed())
                .expect("native run succeeds");

        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("missing capture must reject loudly");
        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Closure",
                ..
            })
        ));
    }

    #[test]
    fn program_runner_preflights_metadata_before_backend_lowering() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));

        let reports = run_nc6_seed_examples(&program).expect("seed program runs");

        assert_eq!(reports.len(), 5);
        assert!(reports
            .iter()
            .all(|report| report.trust.fidelity == NativeFidelity::F1SeedObservationAgreement));
    }

    #[test]
    fn nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes() {
        let body = RuntimeExpr::Let {
            value: Box::new(total_primitive(
                "add_int",
                vec![
                    RuntimeExpr::Value(RuntimeValue::Int(2)),
                    RuntimeExpr::Value(RuntimeValue::Int(3)),
                ],
            )),
            body: Box::new(RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Closure {
                    captures: Vec::new(),
                    params: vec!["x".to_string()],
                    body: Box::new(RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::Construct {
                            constructor: "ctor:fixture::Box::Box".to_string(),
                            args: vec![RuntimeExpr::Var(0)],
                        }),
                        cases: vec![RuntimeMatchCase {
                            constructor: "ctor:fixture::Box::Box".to_string(),
                            binders: 1,
                            body: RuntimeExpr::Record {
                                fields: vec![
                                    (
                                        "ok".to_string(),
                                        RuntimeExpr::If {
                                            scrutinee: Box::new(total_primitive(
                                                "eq_int",
                                                vec![
                                                    RuntimeExpr::Var(0),
                                                    RuntimeExpr::Value(RuntimeValue::Int(5)),
                                                ],
                                            )),
                                            then_expr: Box::new(RuntimeExpr::Value(
                                                RuntimeValue::Bool(true),
                                            )),
                                            else_expr: Box::new(RuntimeExpr::Value(
                                                RuntimeValue::Bool(false),
                                            )),
                                        },
                                    ),
                                    (
                                        "value".to_string(),
                                        total_primitive(
                                            "sub_int",
                                            vec![
                                                total_primitive(
                                                    "mul_int",
                                                    vec![
                                                        RuntimeExpr::Var(0),
                                                        RuntimeExpr::Value(RuntimeValue::Int(2)),
                                                    ],
                                                ),
                                                RuntimeExpr::Value(RuntimeValue::Int(3)),
                                            ],
                                        ),
                                    ),
                                ],
                            },
                        }],
                        default: RuntimeTrap {
                            code: RuntimeTrapCode::PatternMatchFailure,
                            message: "unexpected constructor".to_string(),
                        },
                    }),
                }),
                args: vec![RuntimeExpr::Var(0)],
            }),
        };
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Record {
            fields: vec![
                ("ok".to_string(), RuntimeGroundValue::Bool(true)),
                ("value".to_string(), RuntimeGroundValue::Int(7)),
            ],
        });
        let program = nc22_program_with_body(body, observation.clone());
        let run_report = evaluate_runtime_ir_example(
            &program,
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator runs the compiler-produced artifact");

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert_eq!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::RuntimeIrNativeAgreement {
                stage: NativeDifferentialStage::RuntimeIrNativeCompare,
            }
        );
        let native = report.native.expect("native side ran");
        assert_eq!(native.observation, observation);
        assert_eq!(
            native.trust.fidelity,
            NativeFidelity::F1RuntimeIrEvaluatorAgreement
        );
        assert_eq!(
            native.trust.evidence.runtime_artifact_hash,
            Some(program.artifact_hash)
        );
    }

    #[test]
    fn nc22_imported_dependency_lowers_as_stable_unsupported_native_lane() {
        let symbol = "decl:fixture::Main::main".to_string();
        let dependency = "dep:fixture".to_string();
        let imported = "decl:dep::value".to_string();
        let dependency_hash = "hash:dep".to_string();
        let mut program = nc22_program_with_body(
            RuntimeExpr::ImportedDeclarationRef {
                symbol: imported.clone(),
                dependency: dependency.clone(),
                dependency_semantic_hash: dependency_hash.clone(),
            },
            RuntimeObservation::Returned(RuntimeGroundValue::Int(9)),
        );
        program.declarations[0].symbol = symbol.clone();
        program.erased_core.symbols.insert(imported.clone());
        program
            .erased_core
            .metadata
            .lowerability
            .insert(imported.clone(), RuntimeLowerabilityStatus::Supported);
        program
            .erased_core
            .metadata
            .dependency_semantic_hashes
            .insert(dependency.clone(), dependency_hash.clone());
        let mut runtime_env = RuntimeIrSeedEnvironment::empty();
        runtime_env.insert_imported_declaration(
            imported,
            dependency,
            dependency_hash,
            RuntimeGroundValue::Int(9),
        );
        let run_report = evaluate_runtime_ir_example(&program, &program.examples[0], &runtime_env)
            .expect("runtime-IR evaluator can use an exact imported seed binding");

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert!(matches!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::Unsupported {
                stage: NativeDifferentialStage::NativeLoweringOrExecution,
                construct: "ImportedDeclarationRef",
                ..
            }
        ));
        assert!(report.native.is_none());
    }

    #[test]
    fn nc22_runtime_ir_report_identity_mismatch_rejects_before_native_lowering() {
        let program = nc22_program_with_body(
            RuntimeExpr::Value(RuntimeValue::Int(1)),
            RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
        );
        let mut run_report = evaluate_runtime_ir_example(
            &program,
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator runs");
        run_report.evidence.runtime_artifact_hash = 0xdead_beef;

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert!(matches!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::Unsupported {
                stage: NativeDifferentialStage::BoundaryPreflight,
                construct: "RuntimeIrRunReport",
                ..
            }
        ));
        assert!(report.native.is_none());
    }

    #[test]
    fn nc22_ambiguous_runtime_ir_report_target_rejects_before_native_lowering() {
        let mut program = nc22_program_with_body(
            RuntimeExpr::Value(RuntimeValue::Int(1)),
            RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
        );
        program.examples.push(program.examples[0].clone());
        let mut run_report = evaluate_runtime_ir_example(
            &nc22_program_with_body(
                RuntimeExpr::Value(RuntimeValue::Int(1)),
                RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
            ),
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator runs");
        run_report.artifact = RuntimeArtifactIdentity::from_program(&program);
        run_report.observation.artifact = RuntimeArtifactIdentity::from_program(&program);
        run_report.evidence.package_identity = program.package_identity.clone();
        run_report.evidence.core_semantic_hash = program.core_semantic_hash;
        run_report.evidence.runtime_artifact_hash = program.artifact_hash;

        let report = run_runtime_ir_report_with_cranelift(
            &program,
            run_report,
            &NativeSeedEnvironment::empty(),
        );

        assert!(matches!(
            report.verdict,
            NativeRuntimeIrComparisonVerdict::Unsupported {
                stage: NativeDifferentialStage::BoundaryPreflight,
                construct: "RuntimeIrRunReport",
                ..
            }
        ));
        assert!(report.native.is_none());
    }

    #[test]
    fn nc8_valid_certificate_records_f2_validation_separate_from_f1() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closed-scalar-primitive")
            .expect("seed exists");
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![example.clone()];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        let oracle = InterpreterOracleObservation {
            artifact: NativeArtifactIdentity::from_program(&program),
            observation: example.observation.clone(),
            evidence_source: "test oracle over matching RuntimeProgram identity".to_string(),
        };

        let report = run_validated_example_with_interpreter_observation(
            &program,
            &example,
            &NativeSeedEnvironment::empty(),
            oracle,
            &certificate,
        )
        .expect("certificate validates");

        assert_eq!(
            report.verdict,
            NativeDifferentialVerdict::F1InterpreterAgreement {
                stage: NativeDifferentialStage::InterpreterNativeCompare,
            }
        );
        let native = report.native.expect("native side ran");
        assert_eq!(
            native.trust.fidelity,
            NativeFidelity::F1InterpreterDifferentialAgreement
        );
        let validation = native
            .trust
            .artifact_validation
            .expect("validated artifact fact is report-visible");
        assert_eq!(
            validation.tier,
            RuntimeArtifactValidationTier::F2BoundedRuntimeArtifactValidation
        );
        assert_eq!(
            validation.artifact.package_identity,
            program.package_identity
        );
        assert_eq!(
            validation.artifact.core_semantic_hash,
            program.core_semantic_hash
        );
        assert_eq!(validation.artifact.artifact_hash, program.artifact_hash);
        assert!(validation
            .evidence_source
            .contains("recomputed supported-subset facts"));
    }

    #[test]
    fn nc8_certificate_wrong_identity_rejects_before_native_run() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate.artifact_hash = Some(0xdead_beef);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("wrong artifact identity rejects");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ArtifactIdentity);
        assert_eq!(err.fact, "runtime_artifact_identity");
    }

    #[test]
    fn nc8_certificate_missing_fields_rejects_loudly() {
        let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate.core_semantic_hash = None;

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("missing identity field rejects");

        assert_eq!(
            err.stage,
            RuntimeArtifactValidationStage::MalformedCertificate
        );
        assert_eq!(err.fact, "core_semantic_hash");

        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate.claim.as_mut().expect("claim exists").facts = None;
        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("missing facts reject");

        assert_eq!(
            err.stage,
            RuntimeArtifactValidationStage::MalformedCertificate
        );
        assert_eq!(err.fact, "facts");
    }

    #[test]
    fn nc8_certificate_contradictory_claim_rejects() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "closed-scalar-primitive")
            .expect("seed exists")];
        let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
        certificate
            .claim
            .as_mut()
            .expect("claim exists")
            .facts
            .as_mut()
            .expect("facts exist")
            .declaration_count = Some(program.declarations.len() + 1);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("contradictory count rejects");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimMismatch);
        assert_eq!(err.fact, "declaration_count");
    }

    #[test]
    fn nc8_certificate_false_supported_claim_rejects_by_recomputation() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Unsupported {
                reason: "not lowerable".to_string(),
            }));
        let symbol = program.declarations[0].symbol.clone();
        program.declarations[0].metadata.lowerability =
            Some(RuntimeLowerabilityStatus::Unsupported {
                reason: "not lowerable".to_string(),
            });
        program
            .erased_core
            .metadata
            .unsupported
            .insert(symbol, b"hidden blocker".to_vec());
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("false supported-subset claim rejects");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert!(matches!(
            err.fact,
            "no_reachable_unsupported_entries" | "all_reachable_lowerability_supported"
        ));
    }

    #[test]
    fn nc8_certificate_rejects_unknown_runtime_value_by_recomputation() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "unknown-runtime-value".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Value(RuntimeValue::Unknown),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(0)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("unknown runtime values are outside the supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_values_supported");
        assert!(err.reason.contains("unknown runtime data"));
    }

    #[test]
    fn nc8_certificate_rejects_let_expression_in_validated_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "let-outside-supported-subset".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
                body: Box::new(RuntimeExpr::Var(0)),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("let expressions are outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Let"));
    }

    #[test]
    fn nc8_certificate_rejects_if_expression_in_reachable_transparent_declaration() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::If {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int(0))),
            },
        };
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("if expressions are outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("If"));
    }

    #[test]
    fn nc8_certificate_rejects_unsupported_total_primitive_in_validated_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "unsupported-total-primitive".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "sub_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Int(2)),
                    RuntimeExpr::Value(RuntimeValue::Int(1)),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("unsupported total primitives are outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_primitives_supported");
        assert!(err.reason.contains("sub_int"));
    }

    #[test]
    fn nc8_certificate_rejects_add_int_wrong_arity_in_reachable_transparent_declaration() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "add_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(1))],
            },
        };
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("add_int arity mismatch is outside the NC6 supported subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_primitives_supported");
        assert!(err.reason.contains("arity 1"));
    }

    #[test]
    fn nc8_certificate_rejects_add_int_non_literal_int_operand_shape() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "add-int-non-int-operand".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "add_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Bool(true)),
                    RuntimeExpr::Value(RuntimeValue::Int(1)),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(2)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("add_int non-literal-Int operands are outside the NC8 subset");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_primitives_supported");
        assert!(err.reason.contains("non-literal-Int operand"));
    }

    #[test]
    fn nc8_certificate_rejects_add_int_var_bound_to_bool_payload() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "add-int-var-bound-to-bool".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::BoolBox::Box".to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
                }),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:fixture::BoolBox::Box".to_string(),
                    binders: 1,
                    body: RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "add_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![
                            RuntimeExpr::Var(0),
                            RuntimeExpr::Value(RuntimeValue::Int(1)),
                        ],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "unused default".to_string(),
                },
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(2)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("add_int variable operands are outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Match"));
    }

    #[test]
    fn nc8_certificate_rejects_top_level_var_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "top-level-var".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Var(0),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(0)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("unbound var is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Var"));
    }

    #[test]
    fn nc8_certificate_rejects_project_from_non_record_example() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "project-from-int".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Project {
                record: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
                field: "x".to_string(),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("project is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Project"));
    }

    #[test]
    fn nc8_certificate_rejects_top_level_observable_closure() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.examples = vec![RuntimeExample {
            name: "top-level-closure".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Closure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(1)),
        }];
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("closure is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Closure"));
    }

    #[test]
    fn nc8_certificate_rejects_var_in_reachable_transparent_declaration() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::Var(0),
        };
        let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

        let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
            .expect_err("transparent declaration var is outside the first NC8 validator");

        assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
        assert_eq!(err.fact, "all_runtime_expressions_supported");
        assert!(err.reason.contains("Var"));
    }

    #[test]
    fn missing_lowerability_metadata_rejects_before_backend_lowering() {
        let program = seed_program_with_lowerability(None);

        let err = run_nc6_seed_examples(&program).expect_err("missing metadata rejects");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_unsupported_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .unsupported
            .insert(symbol, b"unsupported target".to_vec());

        let err = run_nc6_seed_examples(&program).expect_err("unsupported metadata rejects");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_declaration_effect_metadata_rejects_before_backend_lowering() {
        for lane in [
            "effects",
            "capabilities",
            "runtime_checks",
            "assumptions",
            "assumption_trust_metadata",
            "trusted_base_delta",
        ] {
            let mut program =
                seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
            let target = program.declarations[0].symbol.clone();
            match lane {
                "effects" => {
                    program.declarations[0]
                        .metadata
                        .effects
                        .insert("Console".to_string());
                }
                "capabilities" => {
                    program.declarations[0]
                        .metadata
                        .capabilities
                        .insert("cap:Console".to_string());
                }
                "runtime_checks" => {
                    program.declarations[0]
                        .metadata
                        .runtime_checks
                        .insert("check:Console".to_string());
                }
                "assumptions" => {
                    program.declarations[0]
                        .metadata
                        .assumptions
                        .insert("assume:Console".to_string());
                }
                "assumption_trust_metadata" => {
                    program.declarations[0]
                        .metadata
                        .assumption_trust_metadata
                        .insert(
                            "assume:Console".to_string(),
                            RuntimeAssumptionTrustMetadata {
                                kind: RuntimeAssumptionTrustKind::Declassify,
                                target,
                                affects_runtime_meaning: true,
                            },
                        );
                }
                "trusted_base_delta" => {
                    program.declarations[0]
                        .metadata
                        .trusted_base_delta
                        .insert("assume:Console".to_string());
                }
                _ => unreachable!("test lanes are exhaustive"),
            }

            let err = match run_nc6_seed_examples(&program) {
                Ok(_) => panic!("expected {lane} metadata to reject"),
                Err(err) => err,
            };

            assert!(matches!(
                err,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "RuntimeProgram",
                    ..
                })
            ));
        }
    }

    #[test]
    fn reachable_package_effect_metadata_rejects_before_backend_lowering() {
        for lane in ["effects", "capabilities", "runtime_checks"] {
            let mut program =
                seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
            match lane {
                "effects" => {
                    program
                        .erased_core
                        .metadata
                        .effects
                        .insert("Console".to_string());
                }
                "capabilities" => {
                    program
                        .erased_core
                        .metadata
                        .capabilities
                        .insert("cap:Console".to_string());
                }
                "runtime_checks" => {
                    program
                        .erased_core
                        .metadata
                        .runtime_checks
                        .insert("check:Console".to_string());
                }
                _ => unreachable!("test lanes are exhaustive"),
            }

            let err = match run_nc6_seed_examples(&program) {
                Ok(_) => panic!("expected package {lane} metadata to reject"),
                Err(err) => err,
            };

            assert!(matches!(
                err,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "RuntimeProgram",
                    ..
                })
            ));
        }
    }

    #[test]
    fn reachable_effectful_checked_core_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                RuntimeEffectsForeignAuditMetadata {
                    declared_effects: BTreeSet::from(["Console".to_string()]),
                    capabilities: BTreeSet::from(["cap:Console".to_string()]),
                    foreign_symbol: None,
                    boundary: RuntimeEffectBoundary::Effectful,
                    runtime_checks: BTreeSet::from(["check:Console".to_string()]),
                    lowerability: RuntimeLowerabilityStatus::Supported,
                },
            );

        let err = run_nc6_seed_examples(&program)
            .expect_err("effectful checked-core metadata must reject");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn reachable_foreign_checked_core_metadata_rejects_before_backend_lowering() {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let symbol = program.declarations[0].symbol.clone();
        program
            .erased_core
            .metadata
            .checked_core
            .effects_foreign_metadata
            .insert(
                symbol,
                RuntimeEffectsForeignAuditMetadata {
                    declared_effects: BTreeSet::new(),
                    capabilities: BTreeSet::new(),
                    foreign_symbol: Some("host.fixture.foreign".to_string()),
                    boundary: RuntimeEffectBoundary::Foreign,
                    runtime_checks: BTreeSet::new(),
                    lowerability: RuntimeLowerabilityStatus::Supported,
                },
            );

        let err =
            run_nc6_seed_examples(&program).expect_err("foreign checked-core metadata must reject");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }

    #[test]
    fn explicit_partial_primitive_reports_trap_not_backend_bug() {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "explicit-partial-primitive-trap")
            .expect("seed exists");

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("trap report succeeds");

        assert!(report.verifier_passed);
        assert!(matches!(
            report.observation,
            RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                ..
            })
        ));
    }

    #[test]
    fn safe_bytes_at_oob_lowers_to_none_with_bounds_obligation() {
        let none = "ctor:fixture::Option::None".to_string();
        let some = "ctor:fixture::Option::Some".to_string();
        let example = RuntimeExample {
            name: "safe-bytes-at-oob".to_string(),
            checked_core_shape: "bytes_at empty 0 : Option UInt8".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_at".to_string(),
                    partiality: RuntimePartiality::SafeOption {
                        none: none.clone(),
                        some,
                        obligation: Some("obl:bytes_at.bounds".to_string()),
                    },
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Bytes(Vec::new())),
                    RuntimeExpr::Value(RuntimeValue::Int(0)),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
                constructor: none,
                args: Vec::new(),
            }),
        };

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("safe bytes_at native lowering succeeds");
        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);
    }

    #[test]
    fn safe_bytes_slice_and_decode_native_results_are_explicit() {
        let none = "ctor:fixture::Option::None".to_string();
        let some = "ctor:fixture::Option::Some".to_string();
        let err = "ctor:fixture::Result::Err".to_string();
        let ok = "ctor:fixture::Result::Ok".to_string();
        let invalid = "ctor:fixture::Utf8Error::InvalidUtf8".to_string();
        let examples = [
            RuntimeExample {
                name: "safe-bytes-slice".to_string(),
                checked_core_shape: "bytes_slice [0,1,2] 1 2".to_string(),
                ir: RuntimeExpr::PrimitiveCall {
                    primitive: RuntimePrimitive {
                        symbol: "bytes_slice".to_string(),
                        partiality: RuntimePartiality::SafeOption {
                            none,
                            some: some.clone(),
                            obligation: None,
                        },
                    },
                    args: vec![
                        RuntimeExpr::Value(RuntimeValue::Bytes(vec![0, 1, 2])),
                        RuntimeExpr::Value(RuntimeValue::Int(1)),
                        RuntimeExpr::Value(RuntimeValue::Int(2)),
                    ],
                },
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
                    constructor: some,
                    args: vec![RuntimeGroundValue::Bytes(vec![1, 2])],
                }),
            },
            RuntimeExample {
                name: "safe-bytes-decode-invalid".to_string(),
                checked_core_shape: "bytes_decode [255]".to_string(),
                ir: RuntimeExpr::PrimitiveCall {
                    primitive: RuntimePrimitive {
                        symbol: "bytes_decode".to_string(),
                        partiality: RuntimePartiality::SafeResult {
                            err: err.clone(),
                            ok,
                            error: invalid.clone(),
                        },
                    },
                    args: vec![RuntimeExpr::Value(RuntimeValue::Bytes(vec![0xff]))],
                },
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
                    constructor: err,
                    args: vec![RuntimeGroundValue::Constructor {
                        constructor: invalid,
                        args: Vec::new(),
                    }],
                }),
            },
        ];

        for example in examples {
            let report =
                run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
                    .expect("safe Bytes native lowering succeeds");
            assert!(report.verifier_passed);
            assert_eq!(report.observation, example.observation);
        }
    }

    #[test]
    fn checked_partial_primitive_still_rejects_unknown_arguments() {
        let example = RuntimeExample {
            name: "unknown-partial-arg".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "checked_index".to_string(),
                    partiality: RuntimePartiality::CheckedTrap {
                        obligation: "obl:checked_index.bounds".to_string(),
                    },
                },
                args: vec![RuntimeExpr::Value(RuntimeValue::Unknown)],
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "unused".to_string(),
            }),
        };

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("unknown argument must reject before trap reporting");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Unknown",
                ..
            })
        ));
    }

    #[test]
    fn overflowing_int_primitive_rejects_before_native_wrapping_semantics() {
        let example = RuntimeExample {
            name: "overflowing-add-int".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: total_primitive(
                "add_int",
                vec![
                    RuntimeExpr::Value(RuntimeValue::Int(i64::MAX)),
                    RuntimeExpr::Value(RuntimeValue::Int(1)),
                ],
            ),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(i64::MIN)),
        };

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("native lowering must not use wrapping Int semantics");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "PrimitiveCall",
                ..
            })
        ));
    }

    #[test]
    fn unsupported_effect_is_distinct_from_backend_failure() {
        let example = RuntimeExample {
            name: "unsupported-effect".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleRead,
                capability: None,
                args: vec![],
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::UnsupportedErasure,
                message: "unsupported".to_string(),
            }),
        };

        let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect_err("effect must reject");

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Effect",
                ..
            })
        ));
    }

    #[test]
    fn pattern_default_trap_is_observation_not_backend_error() {
        let example = RuntimeExample {
            name: "match-default".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:None".to_string(),
                    args: vec![],
                }),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:Some".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "no case selected".to_string(),
                },
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "no case selected".to_string(),
            }),
        };

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("trap report succeeds");

        assert_eq!(report.observation, example.observation);
    }
}
