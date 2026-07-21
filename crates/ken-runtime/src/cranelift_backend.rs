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
            RuntimeGroundValue::Int((2).into()),
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
fn test_only_distinguished_root_join_plan() -> crate::NativeJoinPlanV1 {
    let site_id = 0;
    let declaration = "decl:fixture::CheckedRoot::main".to_string();
    let checked_occurrence_path = vec![0];
    let checked_result_type_fingerprint = 0x5058_3854_4152_4f4f;
    crate::NativeJoinPlanV1 {
        representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
        sites: vec![crate::NativeJoinPlanSiteV1 {
            site_id,
            occurrence_binding_fingerprint:
                crate::compiler_private_join_occurrence_binding_fingerprint(
                    site_id,
                    &declaration,
                    &checked_occurrence_path,
                    checked_result_type_fingerprint,
                ),
            declaration,
            checked_occurrence_path,
            checked_result_type_fingerprint,
            runtime_frame_fingerprint: crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
            answer_kind: crate::NativeJoinAnswerKindV1::ExitCode,
        }],
    }
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
        Some(test_only_distinguished_root_join_plan()),
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
pub(crate) struct Px8trNestedRouteObject {
    pub artifact: CraneliftObjectArtifact,
    pub provenance: Vec<Px8trTrapProvenanceEvent>,
}

#[cfg(test)]
fn px8tr_test_interface(name: u8) -> crate::CheckedAnswerInterfaceV1 {
    let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    bytes.push(name);
    crate::CheckedAnswerInterfaceV1::new(bytes).expect("PX8-TR test interface is canonical")
}

#[cfg(test)]
fn px8tr_nested_post_effect_fixture() -> (
    RuntimeExpr,
    RuntimeDeclaration,
    crate::OrientedSubcontinuationPlanV1,
) {
    let declaration = "decl:fixture::PX8TR::main".to_string();
    let ret_constructor = "ctor:fixture::PX8TR::ITree::Ret".to_string();
    let vis_constructor = "ctor:fixture::PX8TR::ITree::Vis".to_string();
    let result_err = "ctor:prelude::Result::Err".to_string();
    let result_ok = "ctor:prelude::Result::Ok".to_string();
    let unit = || RuntimeExpr::Construct {
        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
        args: Vec::new(),
    };
    let default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-TR checked ITree recursor default".to_string(),
    };
    let ret_body = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: result_err.clone(),
                binders: 1,
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(1.into()))],
                },
            },
            crate::RuntimeMatchCase {
                constructor: result_ok.clone(),
                binders: 1,
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                    args: Vec::new(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-TR Result default".to_string(),
        },
    };
    let terminal_body = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleFlush,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        body: Box::new(RuntimeExpr::Construct {
            constructor: result_ok,
            args: vec![unit()],
        }),
    };
    let recursive_body = RuntimeExpr::Construct {
        constructor: vis_constructor.clone(),
        args: vec![
            unit(),
            RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["response".to_string()],
                body: Box::new(terminal_body),
            },
        ],
    };
    let cases = vec![
        crate::RuntimeComputationalMatchCase {
            constructor: ret_constructor,
            argument_binders: 1,
            recursive_positions: Vec::new(),
            body: ret_body,
        },
        crate::RuntimeComputationalMatchCase {
            constructor: vis_constructor.clone(),
            argument_binders: 2,
            recursive_positions: vec![1],
            body: RuntimeExpr::CheckedComputationalIHSlots {
                slot_template_ids: vec![200],
                checked_occurrence_paths: vec![vec![20]],
                body: Box::new(RuntimeExpr::CheckedComputationalIHInvocation {
                    call_template_id: 100,
                    checked_occurrence_path: vec![30],
                    body: Box::new(RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![unit()],
                    }),
                }),
            },
        },
    ];
    let frame_fingerprint =
        crate::compiler_private_computational_match_frame_fingerprint(&cases, &default);
    let body = RuntimeExpr::Closure {
        captures: Vec::new(),
        params: vec!["process_input".to_string(), "program_caps".to_string()],
        body: Box::new(RuntimeExpr::CheckedSubcontinuationFrame {
            frame_id: 7,
            body: Box::new(RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: vis_constructor.clone(),
                    args: vec![
                        unit(),
                        RuntimeExpr::LexicalClosure {
                            captures: Vec::new(),
                            params: vec!["response".to_string()],
                            body: Box::new(recursive_body),
                        },
                    ],
                }),
                cases,
                default,
            }),
        }),
    };
    let runtime_declaration = RuntimeDeclaration {
        symbol: declaration.clone(),
        kind: RuntimeDeclarationKind::Transparent { body },
        metadata: crate::RuntimeSymbolMetadata::empty(),
    };
    let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
        frame_id: 7,
        segment_site_id: 9,
        declaration: declaration.clone(),
        checked_occurrence_path: vec![10],
        semantic_position: 0,
        input_interface: px8tr_test_interface(0),
        output_interface: px8tr_test_interface(1),
        runtime_frame_fingerprint: frame_fingerprint,
        occurrence_binding_fingerprint: 0,
        control_witness: crate::OrientedControlWitnessV1::DistinguishedRoot,
    };
    frame.occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
    let mut slot = crate::CheckedComputationalIHSlotTemplateV1 {
        slot_template_id: 200,
        declaration: declaration.clone(),
        checked_match_ordinal: 0,
        checked_occurrence_path: vec![20],
        frame_template_id: 7,
        constructor: vis_constructor,
        recursive_position: 1,
        method_binder_ordinal: 0,
        local_telescope: Vec::new(),
        ih_interface: px8tr_test_interface(0),
        segment_site_id: 9,
        frame_templates: vec![7],
        input_interface: px8tr_test_interface(0),
        output_interface: px8tr_test_interface(1),
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration: declaration.clone(),
            runtime_path: vec![2, 0, 2],
        }],
        occurrence_binding_fingerprint: 0,
    };
    slot.occurrence_binding_fingerprint =
        crate::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
    let mut call = crate::CheckedComputationalIHCallTemplateV1 {
        call_template_id: 100,
        declaration: declaration.clone(),
        checked_occurrence_path: vec![30],
        slot_template_id: 200,
        arity: 1,
        local_telescope: Vec::new(),
        result_interface: px8tr_test_interface(1),
        callee_segment_site_id: 9,
        callee_frame_templates: vec![7],
        parent_frame_template_id: Some(7),
        parent_segment_site_id: Some(9),
        caller_interface: px8tr_test_interface(1),
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration,
            runtime_path: vec![2, 0, 2, 0],
        }],
        occurrence_binding_fingerprint: 0,
    };
    call.occurrence_binding_fingerprint =
        crate::compiler_private_computational_ih_call_binding_fingerprint(&call);
    let entrypoint = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::DeclarationRef {
            symbol: runtime_declaration.symbol.clone(),
        }),
        args: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
    };
    (
        entrypoint,
        runtime_declaration,
        crate::OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame],
            recursive_calls: Vec::new(),
            computational_ih_slots: vec![slot],
            computational_ih_calls: vec![call],
        },
    )
}

#[cfg(test)]
pub(crate) fn emit_px8tr_nested_post_effect_object(
    entry_symbol: impl Into<String>,
    disable_repair: bool,
) -> Result<Px8trNestedRouteObject, CraneliftBackendError> {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.set(false);
            PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().clear());
        }
    }

    let entry_symbol = entry_symbol.into();
    let (entrypoint, declaration, plan) = px8tr_nested_post_effect_fixture();
    let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
    PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().clear());
    PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.set(disable_repair);
    let _reset = Reset;
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-px8tr-post-effect")?,
        &entry_symbol,
        Linkage::Export,
        &entrypoint,
        &NativeSeedEnvironment::empty(),
        declarations,
        None,
        true,
        Some(&crate::NativeProcessSymbols::legacy_prelude()),
        Some(test_only_distinguished_root_join_plan()),
        Some(plan),
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
    let provenance = PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow().clone());
    Ok(Px8trNestedRouteObject {
        artifact: CraneliftObjectArtifact {
            example: "px8tr-nested-post-effect".to_string(),
            entry_symbol,
            object_bytes,
            object_hash,
            platform_target: native_platform_target_name(),
            backend_name: "Cranelift PX8-TR process object".to_string(),
            verifier_passed,
            assumptions,
            unsupported,
        },
        provenance,
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
        Some(test_only_distinguished_root_join_plan()),
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
        oriented_subcontinuation_plan_for_program(program)?,
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
    ProcessStatus,
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
        let mut native_int_arena = crate::NativeIntArenaV1::default();
        let process_root = process_root
            .unwrap_or_else(|| (&mut native_int_arena as *mut crate::NativeIntArenaV1).cast());
        let native =
            unsafe { mem::transmute::<_, extern "C" fn(*const std::ffi::c_void) -> i64>(code) };
        let token = native(process_root);
        let decoder = self
            .decoder
            .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?;
        let ground = match decoder {
            ResultDecoder::Int => RuntimeGroundValue::Int(
                native_int_arena
                    .decode_final_export()
                    .ok_or_else(|| backend(BackendFailure::NativeResultDecode { token }))?,
            ),
            ResultDecoder::ProcessStatus => RuntimeGroundValue::Int(token.into()),
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

fn oriented_subcontinuation_plan_for_program(
    program: &RuntimeProgram,
) -> Result<Option<crate::OrientedSubcontinuationPlanV1>, CraneliftBackendError> {
    let candidates = program
        .erased_core
        .metadata
        .checked_core
        .metadata
        .values()
        .filter(|bytes| bytes.starts_with(crate::ORIENTED_SUBCONTINUATION_PLAN_V1_HEADER))
        .collect::<Vec<_>>();
    match candidates.as_slice() {
        [] => Ok(None),
        [bytes] => crate::OrientedSubcontinuationPlanV1::decode(bytes)
            .map(Some)
            .map_err(|reason| unsupported("OrientedSubcontinuationPlanV1", reason)),
        _ => Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked package contains multiple oriented subcontinuation plans",
        )),
    }
}

fn collect_checked_subcontinuation_frames(
    expr: &RuntimeExpr,
    frames: &mut BTreeMap<u64, u64>,
) -> Result<(), CraneliftBackendError> {
    match expr {
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
            let RuntimeExpr::ComputationalMatch { cases, default, .. } = body.as_ref() else {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked subcontinuation marker does not wrap a computational frame",
                ));
            };
            let fingerprint =
                crate::compiler_private_computational_match_frame_fingerprint(cases, default);
            if frames.insert(*frame_id, fingerprint).is_some() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "Runtime IR repeats a checked subcontinuation frame marker",
                ));
            }
            collect_checked_subcontinuation_frames(body, frames)
        }
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Project { record: body, .. }
        | RuntimeExpr::Closure { body, .. } => collect_checked_subcontinuation_frames(body, frames),
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for capture in captures {
                collect_checked_subcontinuation_frames(capture, frames)?;
            }
            collect_checked_subcontinuation_frames(body, frames)
        }
        RuntimeExpr::Let { value, body } => {
            collect_checked_subcontinuation_frames(value, frames)?;
            collect_checked_subcontinuation_frames(body, frames)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_checked_subcontinuation_frames(scrutinee, frames)?;
            collect_checked_subcontinuation_frames(then_expr, frames)?;
            collect_checked_subcontinuation_frames(else_expr, frames)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                collect_checked_subcontinuation_frames(arg, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_checked_subcontinuation_frames(scrutinee, frames)?;
            for case in cases {
                collect_checked_subcontinuation_frames(&case.body, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_checked_subcontinuation_frames(scrutinee, frames)?;
            for case in cases {
                collect_checked_subcontinuation_frames(&case.body, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (_, value) in fields {
                collect_checked_subcontinuation_frames(value, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Call { callee, args } => {
            collect_checked_subcontinuation_frames(callee, frames)?;
            for arg in args {
                collect_checked_subcontinuation_frames(arg, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_checked_subcontinuation_frames(&capability.value, frames)?;
            }
            for arg in args {
                collect_checked_subcontinuation_frames(arg, frames)?;
            }
            Ok(())
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => Ok(()),
    }
}

#[derive(Default)]
struct CheckedOrientedMarkerSets {
    recursive_calls: BTreeMap<(u64, Vec<u64>), BTreeSet<Vec<u64>>>,
    computational_ih_slots: BTreeMap<(u64, Vec<u64>), BTreeSet<Vec<u64>>>,
    computational_ih_calls: BTreeMap<(u64, Vec<u64>), BTreeSet<Vec<u64>>>,
}

impl CheckedOrientedMarkerSets {
    fn extend_from(&mut self, other: &Self) {
        for (key, paths) in &other.recursive_calls {
            self.recursive_calls
                .entry(key.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
        for (key, paths) in &other.computational_ih_slots {
            self.computational_ih_slots
                .entry(key.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
        for (key, paths) in &other.computational_ih_calls {
            self.computational_ih_calls
                .entry(key.clone())
                .or_default()
                .extend(paths.iter().cloned());
        }
    }
}

fn collect_checked_oriented_markers(
    expr: &RuntimeExpr,
    markers: &mut CheckedOrientedMarkerSets,
    root: &str,
    runtime_path: &mut Vec<u64>,
) -> Result<(), CraneliftBackendError> {
    match expr {
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => {
            if !markers
                .recursive_calls
                .entry((*call_template_id, checked_occurrence_path.clone()))
                .or_default()
                .insert(runtime_path.clone())
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    format!(
                        "Runtime IR root {root} repeats checked recursive-call marker {call_template_id} at the same structural path {runtime_path:?}"
                    ),
                ));
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            body,
        } => {
            if slot_template_ids.len() != checked_occurrence_paths.len() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH slot marker identity/location arity differs",
                ));
            }
            for (slot_template_id, checked_occurrence_path) in
                slot_template_ids.iter().zip(checked_occurrence_paths)
            {
                if !markers
                    .computational_ih_slots
                    .entry((*slot_template_id, checked_occurrence_path.clone()))
                    .or_default()
                    .insert(runtime_path.clone())
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        format!(
                            "Runtime IR root {root} repeats checked computational-IH slot marker {slot_template_id} at the same structural path {runtime_path:?}"
                        ),
                    ));
                }
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            body,
        } => {
            if !markers
                .computational_ih_calls
                .entry((*call_template_id, checked_occurrence_path.clone()))
                .or_default()
                .insert(runtime_path.clone())
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    format!(
                        "Runtime IR root {root} repeats checked computational-IH call marker {call_template_id} at the same structural path {runtime_path:?}"
                    ),
                ));
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedJoinSite { body, .. } => {
            collect_checked_oriented_child(body, markers, root, runtime_path, 0)
        }
        RuntimeExpr::Project { record, .. } => {
            collect_checked_oriented_child(record, markers, root, runtime_path, 1)
        }
        RuntimeExpr::Closure { body, .. } => {
            collect_checked_oriented_child(body, markers, root, runtime_path, 2)
        }
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            for (index, capture) in captures.iter().enumerate() {
                collect_checked_oriented_child(
                    capture,
                    markers,
                    root,
                    runtime_path,
                    10 + index as u64,
                )?;
            }
            collect_checked_oriented_child(body, markers, root, runtime_path, 3)
        }
        RuntimeExpr::Let { value, body } => {
            collect_checked_oriented_child(value, markers, root, runtime_path, 0)?;
            collect_checked_oriented_child(body, markers, root, runtime_path, 1)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_checked_oriented_child(scrutinee, markers, root, runtime_path, 0)?;
            collect_checked_oriented_child(then_expr, markers, root, runtime_path, 1)?;
            collect_checked_oriented_child(else_expr, markers, root, runtime_path, 2)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for (index, arg) in args.iter().enumerate() {
                collect_checked_oriented_child(arg, markers, root, runtime_path, index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_checked_oriented_child(scrutinee, markers, root, runtime_path, 0)?;
            for (index, case) in cases.iter().enumerate() {
                collect_checked_oriented_child(
                    &case.body,
                    markers,
                    root,
                    runtime_path,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_checked_oriented_child(scrutinee, markers, root, runtime_path, 0)?;
            for (index, case) in cases.iter().enumerate() {
                collect_checked_oriented_child(
                    &case.body,
                    markers,
                    root,
                    runtime_path,
                    1 + index as u64,
                )?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            for (index, (_, value)) in fields.iter().enumerate() {
                collect_checked_oriented_child(value, markers, root, runtime_path, index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Call { callee, args } => {
            collect_checked_oriented_child(callee, markers, root, runtime_path, 0)?;
            for (index, arg) in args.iter().enumerate() {
                collect_checked_oriented_child(arg, markers, root, runtime_path, 1 + index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_checked_oriented_child(&capability.value, markers, root, runtime_path, 0)?;
            }
            for (index, arg) in args.iter().enumerate() {
                collect_checked_oriented_child(arg, markers, root, runtime_path, 1 + index as u64)?;
            }
            Ok(())
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => Ok(()),
    }
}

fn collect_checked_oriented_child(
    expr: &RuntimeExpr,
    markers: &mut CheckedOrientedMarkerSets,
    root: &str,
    runtime_path: &mut Vec<u64>,
    edge: u64,
) -> Result<(), CraneliftBackendError> {
    runtime_path.push(edge);
    let result = collect_checked_oriented_markers(expr, markers, root, runtime_path);
    runtime_path.pop();
    result
}

fn planned_marker_locations_for_declaration(
    plan: &crate::OrientedSubcontinuationPlanV1,
    declaration: &str,
) -> CheckedOrientedMarkerSets {
    let mut expected = CheckedOrientedMarkerSets::default();
    for call in &plan.recursive_calls {
        if call.declaration == declaration {
            expected.recursive_calls.insert(
                (call.call_template_id, call.checked_occurrence_path.clone()),
                call.runtime_marker_locations
                    .iter()
                    .map(|location| location.runtime_path.clone())
                    .collect(),
            );
        }
    }
    for slot in &plan.computational_ih_slots {
        if slot.declaration == declaration {
            expected.computational_ih_slots.insert(
                (slot.slot_template_id, slot.checked_occurrence_path.clone()),
                slot.runtime_marker_locations
                    .iter()
                    .map(|location| location.runtime_path.clone())
                    .collect(),
            );
        }
    }
    for call in &plan.computational_ih_calls {
        if call.declaration == declaration {
            expected.computational_ih_calls.insert(
                (call.call_template_id, call.checked_occurrence_path.clone()),
                call.runtime_marker_locations
                    .iter()
                    .map(|location| location.runtime_path.clone())
                    .collect(),
            );
        }
    }
    expected
}

fn require_exact_marker_locations(
    declaration: &str,
    actual: &CheckedOrientedMarkerSets,
    expected: &CheckedOrientedMarkerSets,
) -> Result<(), CraneliftBackendError> {
    if actual.recursive_calls != expected.recursive_calls {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked recursive-call Runtime occurrences differ in declaration {declaration}"
            ),
        ));
    }
    if actual.computational_ih_slots != expected.computational_ih_slots {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked computational-IH slot Runtime occurrences differ in declaration {declaration}"
            ),
        ));
    }
    if actual.computational_ih_calls != expected.computational_ih_calls {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked computational-IH call Runtime occurrences differ in declaration {declaration}"
            ),
        ));
    }
    Ok(())
}

fn validate_oriented_subcontinuation_transport(
    expr: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
    plan: Option<&crate::OrientedSubcontinuationPlanV1>,
) -> Result<(), CraneliftBackendError> {
    let mut markers = BTreeMap::new();
    let mut entry_nonframe_markers = CheckedOrientedMarkerSets::default();
    let mut nonframe_markers = CheckedOrientedMarkerSets::default();
    let mut declaration_nonframe_markers = Vec::new();
    collect_checked_subcontinuation_frames(expr, &mut markers)?;
    collect_checked_oriented_markers(
        expr,
        &mut entry_nonframe_markers,
        "<entry>",
        &mut Vec::new(),
    )?;
    nonframe_markers.extend_from(&entry_nonframe_markers);
    for (symbol, declaration) in declarations.iter() {
        if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
            collect_checked_subcontinuation_frames(body, &mut markers)?;
            let mut declaration_markers = CheckedOrientedMarkerSets::default();
            collect_checked_oriented_markers(
                body,
                &mut declaration_markers,
                symbol,
                &mut Vec::new(),
            )?;
            nonframe_markers.extend_from(&declaration_markers);
            declaration_nonframe_markers.push((*symbol, declaration_markers));
        }
    }
    let markers_are_empty = markers.is_empty()
        && nonframe_markers.recursive_calls.is_empty()
        && nonframe_markers.computational_ih_slots.is_empty()
        && nonframe_markers.computational_ih_calls.is_empty();
    match (markers_are_empty, plan) {
        (true, None) => return Ok(()),
        (false, None) => {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked subcontinuation markers have no checked plan metadata",
            ));
        }
        (true, Some(plan))
            if plan.frames.is_empty()
                && plan.recursive_calls.is_empty()
                && plan.computational_ih_slots.is_empty()
                && plan.computational_ih_calls.is_empty() =>
        {
            return Ok(())
        }
        (true, Some(_)) => {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked plan has no Runtime frame markers",
            ));
        }
        (false, Some(_)) => {}
    }
    let plan = plan.expect("nonempty marker set has a plan");
    plan.validate()
        .map_err(|reason| unsupported("OrientedSubcontinuationPlanV1", reason))?;
    if !entry_nonframe_markers.recursive_calls.is_empty()
        || !entry_nonframe_markers.computational_ih_slots.is_empty()
        || !entry_nonframe_markers.computational_ih_calls.is_empty()
    {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked recursive/IH marker escaped its declaration into the entry expression",
        ));
    }
    for (declaration, markers) in &declaration_nonframe_markers {
        let expected = planned_marker_locations_for_declaration(plan, declaration);
        require_exact_marker_locations(declaration, markers, &expected)?;
    }
    if markers.len() != plan.frames.len() {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked plan and Runtime marker sets differ",
        ));
    }
    for frame in &plan.frames {
        let Some(fingerprint) = markers.remove(&frame.frame_id) else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked plan frame marker is missing or transplanted",
            ));
        };
        if fingerprint != frame.runtime_frame_fingerprint {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked plan frame fingerprint is stale",
            ));
        }
    }
    if !markers.is_empty() {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "Runtime frame marker has no checked plan entry",
        ));
    }
    let plan_recursive_calls = plan
        .recursive_calls
        .iter()
        .map(|call| (call.call_template_id, call.checked_occurrence_path.clone()))
        .collect::<BTreeSet<_>>();
    let plan_ih_slots = plan
        .computational_ih_slots
        .iter()
        .map(|slot| (slot.slot_template_id, slot.checked_occurrence_path.clone()))
        .collect::<BTreeSet<_>>();
    let plan_ih_calls = plan
        .computational_ih_calls
        .iter()
        .map(|call| (call.call_template_id, call.checked_occurrence_path.clone()))
        .collect::<BTreeSet<_>>();
    let runtime_recursive_calls = nonframe_markers
        .recursive_calls
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let runtime_ih_slots = nonframe_markers
        .computational_ih_slots
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let runtime_ih_calls = nonframe_markers
        .computational_ih_calls
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    if runtime_recursive_calls != plan_recursive_calls
        || runtime_ih_slots != plan_ih_slots
        || runtime_ih_calls != plan_ih_calls
    {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            "checked plan and Runtime recursive/IH marker sets differ",
        ));
    }
    Ok(())
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

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum Px8jDirectRecursorConsumer {
    PendingLetProducer,
    ProducerCall,
    OrdinaryCall,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum Px8jRecursorMalformation {
    SelectionRole,
    RepeatedScopeIdentity,
    BrokenScopeParent,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Px8jProducerPath {
    Composed,
    DeferredConstructor,
    SourceMachine,
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Px8jSourceTraceEvent {
    Mint {
        path: Px8jProducerPath,
        origin: RecursorProducerOriginId,
        cursor: ContinuationCursorId,
        siblings: usize,
        parent_scope: Option<RecursorProducerOriginId>,
    },
    Carrier {
        path: Px8jProducerPath,
        origin: RecursorProducerOriginId,
        cursor: ContinuationCursorId,
        sibling_position: usize,
    },
    Install {
        origin: RecursorProducerOriginId,
        selection_cursor: ContinuationCursorId,
        sibling_position: usize,
        exits: Vec<(RecursorProducerOriginId, Option<RecursorProducerOriginId>)>,
    },
    DirectConsume {
        origin: RecursorProducerOriginId,
        selection_cursor: ContinuationCursorId,
        sibling_position: usize,
        exits: Vec<(RecursorProducerOriginId, Option<RecursorProducerOriginId>)>,
    },
    Selection {
        origin: RecursorProducerOriginId,
    },
    Exit {
        origin: RecursorProducerOriginId,
        scope_origin: RecursorProducerOriginId,
        parent_scope: Option<RecursorProducerOriginId>,
    },
    ReturnHole {
        cursor: ContinuationCursorId,
    },
    ResumeOuter {
        cursor: ContinuationCursorId,
    },
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Px8trTrapProvenanceEvent {
    CheckedRecursorDefault {
        checked_frame_id: u64,
        actual_constructor: Option<RuntimeSymbol>,
        trap: RuntimeTrap,
    },
    DeforestedAnswerResumed {
        checked_frame_id: u64,
        actual_constructor: Option<RuntimeSymbol>,
        return_constructor: RuntimeSymbol,
    },
    FinalProcessObjectTrap {
        trap: RuntimeTrap,
    },
}

#[cfg(test)]
thread_local! {
    static PX8J_SOURCE_TRACE: std::cell::RefCell<Vec<Px8jSourceTraceEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static PX8J_DELETE_OWNED_SELECTED_SCOPE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
    static PX8TR_TRAP_PROVENANCE: std::cell::RefCell<Vec<Px8trTrapProvenanceEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn px8j_record_source_event(event: Px8jSourceTraceEvent) {
    PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().push(event));
}

#[cfg(test)]
fn px8tr_record_trap_provenance(event: Px8trTrapProvenanceEvent) {
    PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().push(event));
}

#[cfg(test)]
fn px8tr_deforested_answer_route_enabled() -> bool {
    !PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.get()
}

#[cfg(not(test))]
fn px8tr_deforested_answer_route_enabled() -> bool {
    true
}

#[cfg(test)]
fn px8j_record_recursor_carrier(path: Px8jProducerPath, value: &Lowered) {
    let Lowered::ComputationalRecursorClosure { invocation, .. } = value else {
        return;
    };
    px8j_record_source_event(Px8jSourceTraceEvent::Carrier {
        path,
        origin: invocation.origin,
        cursor: invocation.resume_cursor,
        sibling_position: invocation.sibling_position,
    });
}

#[cfg(test)]
fn run_px8j_malformed_recursor_consumer(
    consumer: Px8jDirectRecursorConsumer,
    malformation: Px8jRecursorMalformation,
) -> Result<Lowered, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px8j_malformed_recursor", Linkage::Local, &signature)
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
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan: None,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: None,
        native_int_compare: None,
        native_int_intern: None,
        native_int_narrow: None,
        native_int_export: None,
        native_int_tags: BTreeMap::new(),
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let origin = RecursorProducerOriginId(7);
    let cursor = ContinuationCursorId(9);
    let layer = |role| ComputationalRecursorLayer {
        cases: Vec::new(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px8j malformed recursor role".to_string(),
        },
        outer_env: Vec::new(),
        provenance: RecursorFrameProvenance(6),
        role,
        checked_frame_id: None,
        checked_invocation_id: None,
        checked_invocation_source: None,
        checked_invocation_depth: 0,
        semantic_pending: matches!(role, RecursorLayerRole::SelectsOccurrence { .. }),
    };
    let selection = layer(match malformation {
        Px8jRecursorMalformation::SelectionRole => RecursorLayerRole::ExitsScope {
            origin,
            scope_origin: origin,
            parent_scope: None,
        },
        Px8jRecursorMalformation::RepeatedScopeIdentity
        | Px8jRecursorMalformation::BrokenScopeParent => {
            RecursorLayerRole::SelectsOccurrence { origin }
        }
    });
    let unwind = match malformation {
        Px8jRecursorMalformation::SelectionRole => Vec::new(),
        Px8jRecursorMalformation::RepeatedScopeIdentity => vec![
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(11),
                parent_scope: None,
            }),
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(11),
                parent_scope: Some(RecursorProducerOriginId(11)),
            }),
        ],
        Px8jRecursorMalformation::BrokenScopeParent => vec![
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(11),
                parent_scope: None,
            }),
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(12),
                parent_scope: Some(RecursorProducerOriginId(99)),
            }),
        ],
    };
    let recursor = Lowered::ComputationalRecursorClosure {
        residual: Box::new(Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            body: RuntimeExpr::Construct {
                constructor: "ctor:fixture::PX8J::Done".to_string(),
                args: Vec::new(),
            },
        }),
        activation: ContinuationActivationId(8),
        invocation: RecursorInvocationSegment::new(
            origin,
            0,
            selection,
            RecursorUnwindStack {
                later_wrappers_in_construction_order: unwind,
            },
            cursor,
            None,
            None,
        ),
    };
    let active = ActiveContinuationFrame {
        activation: ContinuationActivationId(8),
        cursor,
        parent: None,
        pending: &[],
        selected_ancestry: &[],
        source_lineage: &[],
        source_selected_cursor: None,
        selected_scope: None,
    };
    let active_frames = [EliminatorFrame::Active(active)];
    let env = [recursor];
    let call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: Vec::new(),
    };
    let pending_let = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Var(1)),
            args: Vec::new(),
        }),
    };
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer => compiler
            .lower_computational_producer_expr(&mut builder, &pending_let, &env, &active_frames),
        Px8jDirectRecursorConsumer::ProducerCall => {
            compiler.lower_computational_producer_expr(&mut builder, &call, &env, &active_frames)
        }
        Px8jDirectRecursorConsumer::OrdinaryCall => compiler.lower_expr(&mut builder, &call, &env),
    }
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
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan: None,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: None,
        native_int_compare: None,
        native_int_intern: None,
        native_int_narrow: None,
        native_int_export: None,
        native_int_tags: BTreeMap::new(),
        native_int_mutation: NativeIntLoweringMutation::Exact,
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
                        body: RuntimeExpr::Value(RuntimeValue::Int((10).into())),
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
                                    body: RuntimeExpr::Value(RuntimeValue::Int((21).into())),
                                },
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_suc.clone(),
                                    binders: 1,
                                    body: RuntimeExpr::Value(RuntimeValue::Int((22).into())),
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
                        checked_frame_id: None,
                        checked_invocation_id: None,
                        checked_invocation_source: None,
                        checked_invocation_depth: 0,
                    },
                )];
                compiler.lower_bounded_nat_computational(&mut builder, nat, false, &frames)?
            }
        };
        let value = match lowered {
            Lowered::Int { value, .. } => value,
            other => compiler.emit_result(&mut builder, other)?.0,
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
        decoder: Some(ResultDecoder::ProcessStatus),
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
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan: None,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: None,
        native_int_compare: None,
        native_int_intern: None,
        native_int_narrow: None,
        native_int_export: None,
        native_int_tags: BTreeMap::new(),
        native_int_mutation: NativeIntLoweringMutation::Exact,
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
                body: RuntimeExpr::Value(RuntimeValue::Int((result).into())),
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
            Lowered::Int { value, .. } => value,
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
        decoder: Some(ResultDecoder::ProcessStatus),
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
        oriented_subcontinuation_plan_for_program(program)?,
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
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    validate_oriented_subcontinuation_transport(
        expr,
        &declarations,
        oriented_subcontinuation_plan.as_ref(),
    )?;
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function(function_name, linkage, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
    let native_int_wrapping_mutation = {
        #[cfg(test)]
        {
            NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get)
                == NativeIntLoweringMutation::Wrapping
        }
        #[cfg(not(test))]
        {
            false
        }
    };
    let native_int = crate::native_int_clif::emit_native_int_local_graph(
        &mut module,
        native_int_wrapping_mutation,
    )?;
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
    let int_binop = module.declare_func_in_func(native_int.binop, &mut ctx.func);
    let int_compare = module.declare_func_in_func(native_int.compare, &mut ctx.func);
    let int_intern = module.declare_func_in_func(native_int.intern, &mut ctx.func);
    let int_narrow = module.declare_func_in_func(native_int.narrow, &mut ctx.func);
    let int_export = module.declare_func_in_func(native_int.export, &mut ctx.func);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        declarations,
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        native_join_plan,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: process_mode,
        process_symbols: process_symbols
            .cloned()
            .unwrap_or_else(crate::NativeProcessSymbols::legacy_prelude),
        host_dispatch,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: Some(int_binop),
        native_int_compare: Some(int_compare),
        native_int_intern: Some(int_intern),
        native_int_narrow: Some(int_narrow),
        native_int_export: Some(int_export),
        native_int_tags: BTreeMap::new(),
        #[cfg(test)]
        native_int_mutation: NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get),
        #[cfg(test)]
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let (maybe_trap, decoder) = {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        let invocation = builder.block_params(block)[0];
        compiler.native_int_arena = Some(invocation);
        let mut initial_env = Vec::new();
        if process_mode {
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
            let int_arena = builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), invocation, 24);
            Lowering::require_nonzero(&mut builder, int_arena);
            compiler.native_int_arena = Some(int_arena);
            initial_env.push(Lowered::BorrowedNativeValue {
                pointer: process_input,
            });
            initial_env.push(Lowered::CapabilityToken { value: capability });
        }
        if let Some(value) = staged_process_input {
            initial_env.push(compiler.lower_value(&mut builder, value)?);
        }
        compiler.root_terminal_authority = compiler.take_distinguished_root_answer_authority()?;
        let lowered = compiler.lower_expr(&mut builder, expr, &initial_env)?;
        compiler.require_complete_join_plan_consumption()?;
        compiler.require_complete_dynamic_splice_edge_consumption()?;
        let result = match lowered {
            Lowered::Trap(trap) => {
                #[cfg(test)]
                if process_mode {
                    px8tr_record_trap_provenance(
                        Px8trTrapProvenanceEvent::FinalProcessObjectTrap { trap: trap.clone() },
                    );
                }
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
    next_recursor_producer_origin: u64,
    next_continuation_activation: u64,
    next_continuation_cursor: u64,
    next_source_join: u64,
    next_source_predecessor: u64,
    live_source_continuations: usize,
    source_control_root: Option<ContinuationCursorId>,
    active_oriented_semantic_regions: usize,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    consumed_join_sites: BTreeSet<u64>,
    root_terminal_authority: Option<RootTerminalAnswerAuthority>,
    active_join_site: Option<u64>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
    consumed_subcontinuation_frames: BTreeSet<(u64, u64)>,
    active_subcontinuation_frame: Option<u64>,
    consumed_recursive_call_templates: BTreeSet<u64>,
    pending_recursive_call: Option<CheckedRecursiveInvocationInstance>,
    pending_computational_ih_call: Option<u64>,
    active_recursive_invocations: Vec<CheckedRecursiveInvocationInstance>,
    next_recursive_invocation_instance: u64,
    dynamic_splice_edges: BTreeMap<DynamicSpliceEdgeId, DynamicSpliceEdge>,
    next_dynamic_splice_edge: u64,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
    process_object: bool,
    process_symbols: crate::NativeProcessSymbols,
    host_dispatch: Option<FuncRef>,
    invocation_pointer: Option<cranelift_codegen::ir::Value>,
    native_int_arena: Option<cranelift_codegen::ir::Value>,
    native_int_binop: Option<FuncRef>,
    native_int_compare: Option<FuncRef>,
    native_int_intern: Option<FuncRef>,
    native_int_narrow: Option<FuncRef>,
    native_int_export: Option<FuncRef>,
    native_int_tags: BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    #[cfg(test)]
    native_int_mutation: NativeIntLoweringMutation,
    #[cfg(test)]
    bounded_nat_mutation: BoundedNatLoweringMutation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecursorFrameProvenance(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InvocationTemplateRef {
    SameSccCall(u64),
    ComputationalIHCall(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckedRecursiveInvocationInstance {
    source: InvocationTemplateRef,
    invocation_instance_id: u64,
    semantic_depth: usize,
    dynamic_splice_edge: Option<DynamicSpliceEdgeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DynamicSpliceEdgeId(u64);

/// The unique compiler-owned authority to splice one completed dynamic child
/// invocation into one exact open parent occurrence. Lowered values retain
/// only the inert `DynamicSpliceEdgeId`; this non-`Clone` ledger entry is
/// removed and consumed before any CFG is emitted.
struct DynamicSpliceEdge {
    edge_id: DynamicSpliceEdgeId,
    child_invocation_instance_id: u64,
    parent_invocation_instance_id: u64,
    checked_call_template_id: u64,
    parent_frame_template_id: u64,
    segment_site_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContinuationActivationId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContinuationCursorId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecursorProducerOriginId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecursorLayerRole {
    SelectsOccurrence {
        origin: RecursorProducerOriginId,
    },
    ExitsScope {
        origin: RecursorProducerOriginId,
        scope_origin: RecursorProducerOriginId,
        parent_scope: Option<RecursorProducerOriginId>,
    },
}

#[derive(Clone)]
struct ComputationalRecursorFramePayload {
    cases: Vec<crate::RuntimeComputationalMatchCase>,
    default: RuntimeTrap,
    outer_env: Vec<Lowered>,
    provenance: RecursorFrameProvenance,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
}

#[derive(Clone)]
struct OwnedSelectedScope {
    scope_origin: RecursorProducerOriginId,
    parent_scope: Option<RecursorProducerOriginId>,
    frame: ComputationalRecursorFramePayload,
}

#[derive(Clone, Copy)]
struct NativeScalarPairV1 {
    tag: cranelift_codegen::ir::Value,
    payload: cranelift_codegen::ir::Value,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeIntLoweringMutation {
    Exact,
    Wrapping,
    Trap,
    SuppressTerminalExport,
    CorruptTerminalExport,
}

#[cfg(test)]
thread_local! {
    pub(crate) static NATIVE_INT_LOWERING_MUTATION: std::cell::Cell<NativeIntLoweringMutation> =
        const { std::cell::Cell::new(NativeIntLoweringMutation::Exact) };
}

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
    if constructor.ends_with("::ResourceRead") && args.is_empty() {
        Some(0)
    } else if constructor.ends_with("::ResourceMetadata") && args.is_empty() {
        Some(1)
    } else if constructor.ends_with("::ResourceWriteCreate") && args.len() == 1 {
        create_policy_tag(&args[0]).map(|tag| tag + 2)
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
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
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
    source_lineage: &'a [SourceSelectedContinuation<'a>],
    source_selected_cursor: Option<ContinuationCursorId>,
    selected_scope: Option<&'a OwnedSelectedScope>,
}

#[derive(Clone)]
struct ComputationalRecursorLayer {
    cases: Vec<crate::RuntimeComputationalMatchCase>,
    default: RuntimeTrap,
    outer_env: Vec<Lowered>,
    provenance: RecursorFrameProvenance,
    role: RecursorLayerRole,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
    semantic_pending: bool,
}

#[derive(Clone)]
struct RecursorInvocationSegment {
    origin: RecursorProducerOriginId,
    /// Declaration-order field position inside the one selected constructor
    /// case. Siblings share `origin`; this position distinguishes their
    /// immutable carriers through the consumer boundary.
    sibling_position: usize,
    selection: ComputationalRecursorLayer,
    unwind: RecursorUnwindStack,
    resume_cursor: ContinuationCursorId,
    checked_invocation: Option<CheckedRecursiveInvocationInstance>,
    computational_ih_slot_template_id: Option<u64>,
    /// Inert handles into `Lowering::dynamic_splice_edges`. Cloning a lowered
    /// recursor can copy a handle, but only one clone can consume the unique
    /// compiler-owned edge; every replay rejects before CFG.
    dynamic_splice_edges: Vec<DynamicSpliceEdgeId>,
    /// Immutable mint-time witness for every already-open control extent.
    /// Qualification may attach a fresh invocation identity later, but it may
    /// not delete, duplicate, reorder, or transplant an exit obligation.
    open_control_obligations: Vec<OpenControlObligation>,
}

#[derive(Clone)]
struct RecursorUnwindStack {
    later_wrappers_in_construction_order: Vec<ComputationalRecursorLayer>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenControlObligation {
    scope_origin: RecursorProducerOriginId,
    parent_scope: Option<RecursorProducerOriginId>,
    checked_frame_id: Option<u64>,
    semantic_pending: bool,
}

fn open_control_obligations(unwind: &RecursorUnwindStack) -> Vec<OpenControlObligation> {
    unwind
        .later_wrappers_in_construction_order
        .iter()
        .filter_map(|layer| match layer.role {
            RecursorLayerRole::ExitsScope {
                scope_origin,
                parent_scope,
                ..
            } => Some(OpenControlObligation {
                scope_origin,
                parent_scope,
                checked_frame_id: layer.checked_frame_id,
                semantic_pending: layer.semantic_pending,
            }),
            RecursorLayerRole::SelectsOccurrence { .. } => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AffineSpliceState {
    Open,
    Consumed,
}

/// Move-only compiler capability for one oriented segment splice.  It is
/// deliberately not `Clone`; validation consumes the sole open state before
/// any CFG or consumer lowering begins.
struct AffineSpliceCapability {
    state: AffineSpliceState,
}

impl AffineSpliceCapability {
    fn consume(&mut self) -> Result<(), CraneliftBackendError> {
        if std::mem::replace(&mut self.state, AffineSpliceState::Consumed)
            == AffineSpliceState::Consumed
        {
            return Err(unsupported(
                "OrientedSubcontinuation",
                "affine splice capability was consumed more than once",
            ));
        }
        Ok(())
    }
}

#[derive(Clone)]
struct OrientedControlLedgerEntry {
    frame_id: Option<u64>,
    invocation_id: Option<u64>,
    role: RecursorLayerRole,
    checked_witness: Option<crate::OrientedControlWitnessV1>,
}

fn oriented_layer_is_pending_semantic(layer: &ComputationalRecursorLayer) -> bool {
    layer.semantic_pending
}

fn validate_oriented_control_projection(
    producer_origin: RecursorProducerOriginId,
    layers: &[ComputationalRecursorLayer],
) -> Result<(), CraneliftBackendError> {
    let mut invocation_sources = BTreeMap::new();
    let mut open_scopes = BTreeMap::new();
    for layer in layers {
        let role_origin = match layer.role {
            RecursorLayerRole::SelectsOccurrence { origin }
            | RecursorLayerRole::ExitsScope { origin, .. } => origin,
        };
        if role_origin != producer_origin {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "control occurrence was transplanted across producer regions",
            ));
        }
        match (layer.checked_invocation_id, layer.checked_invocation_source) {
            (Some(instance), Some(source)) => {
                if invocation_sources
                    .insert(instance, source)
                    .is_some_and(|old| old != source)
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "one invocation instance is shared by distinct checked templates",
                    ));
                }
            }
            (None, Some(_)) => {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked invocation source has no affine instance identity",
                ));
            }
            _ => {}
        }
        match (layer.role, layer.semantic_pending) {
            (RecursorLayerRole::SelectsOccurrence { .. }, false) => {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "pending selection was misclassified as control-only",
                ));
            }
            _ => {}
        }
        if let RecursorLayerRole::ExitsScope {
            scope_origin,
            parent_scope,
            ..
        } = layer.role
        {
            if layer.checked_invocation_id.is_some() {
                if open_scopes.insert(scope_origin, parent_scope).is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "open control obligation is duplicated",
                    ));
                }
            }
        }
    }
    for parent in open_scopes.values().flatten() {
        if !open_scopes.contains_key(parent) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "open control obligation has a stale or cross-region parent",
            ));
        }
    }
    Ok(())
}

struct OwnedOrientedSubcontinuationSegment {
    producer_origin: RecursorProducerOriginId,
    sibling_position: usize,
    activation: ContinuationActivationId,
    segment_site_id: Option<u64>,
    input_interface: Option<crate::CheckedAnswerInterfaceV1>,
    output_interface: Option<crate::CheckedAnswerInterfaceV1>,
    semantic_frames: Vec<ComputationalRecursorLayer>,
    control_ledger: Vec<OrientedControlLedgerEntry>,
    resume_cursor: ContinuationCursorId,
    capability: AffineSpliceCapability,
}

struct InstalledOrientedSubcontinuationSegment {
    checked: bool,
    producer_origin: RecursorProducerOriginId,
    sibling_position: usize,
    activation: ContinuationActivationId,
    semantic_frames: Vec<ComputationalRecursorLayer>,
    control_ledger: Vec<OrientedControlLedgerEntry>,
    resume_cursor: ContinuationCursorId,
}

impl RecursorInvocationSegment {
    fn new(
        origin: RecursorProducerOriginId,
        sibling_position: usize,
        selection: ComputationalRecursorLayer,
        unwind: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        checked_invocation: Option<CheckedRecursiveInvocationInstance>,
        computational_ih_slot_template_id: Option<u64>,
    ) -> Self {
        let open_control_obligations = open_control_obligations(&unwind);
        Self {
            origin,
            sibling_position,
            selection,
            unwind,
            resume_cursor,
            checked_invocation,
            computational_ih_slot_template_id,
            dynamic_splice_edges: Vec::new(),
            open_control_obligations,
        }
    }

    fn validate_open_control_obligations(&self) -> Result<(), CraneliftBackendError> {
        if open_control_obligations(&self.unwind) != self.open_control_obligations {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "open control obligation set changed after affine mint",
            ));
        }
        Ok(())
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

fn checked_invocation_frame_templates(
    plan: &crate::OrientedSubcontinuationPlanV1,
    source: InvocationTemplateRef,
) -> Result<&[u64], CraneliftBackendError> {
    match source {
        InvocationTemplateRef::SameSccCall(call_template_id) => plan
            .recursive_call(call_template_id)
            .map(|call| call.callee_frame_templates.as_slice())
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic invocation has no checked same-SCC call template",
                )
            }),
        InvocationTemplateRef::ComputationalIHCall(call_template_id) => plan
            .computational_ih_call(call_template_id)
            .map(|call| call.callee_frame_templates.as_slice())
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic invocation has no checked computational IH call template",
                )
            }),
    }
}

fn instantiate_checked_invocation_segment(
    plan: &crate::OrientedSubcontinuationPlanV1,
    invocation: CheckedRecursiveInvocationInstance,
    segment: &mut RecursorInvocationSegment,
) -> Result<(), CraneliftBackendError> {
    let frame_templates = checked_invocation_frame_templates(plan, invocation.source)?;
    let expected = frame_templates.iter().copied().collect::<BTreeSet<_>>();
    let mut instantiated = BTreeSet::new();
    let mut visit = |layer: &mut ComputationalRecursorLayer| {
        let Some(frame_id) = layer.checked_frame_id else {
            return Ok(());
        };
        let frame = plan.frame(frame_id).ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic recursive layer has no checked frame entry",
            )
        })?;
        if frame.runtime_frame_fingerprint
            != crate::compiler_private_computational_match_frame_fingerprint(
                &layer.cases,
                &layer.default,
            )
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic recursive layer does not match its checked frame template",
            ));
        }
        if !expected.contains(&frame_id) {
            return Ok(());
        }
        match layer.checked_invocation_id {
            None => {
                layer.checked_invocation_id = Some(invocation.invocation_instance_id);
                layer.checked_invocation_source = Some(invocation.source);
                layer.checked_invocation_depth = invocation.semantic_depth;
            }
            Some(existing) if existing == invocation.invocation_instance_id => {
                if layer.checked_invocation_source != Some(invocation.source) {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic invocation source changed after qualification",
                    ));
                }
            }
            Some(_) => return Ok(()),
        }
        if !instantiated.insert(frame_id) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "one invocation instantiates a checked frame template more than once",
            ));
        }
        Ok(())
    };
    visit(&mut segment.selection)?;
    for layer in &mut segment.unwind.later_wrappers_in_construction_order {
        visit(layer)?;
    }
    if instantiated != expected {
        let actual = std::iter::once(&segment.selection)
            .chain(segment.unwind.later_wrappers_in_construction_order.iter())
            .map(|layer| (layer.checked_frame_id, layer.checked_invocation_id))
            .collect::<Vec<_>>();
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "computational invocation {:?} does not carry its exact checked frame sequence: expected={expected:?} instantiated={instantiated:?} actual={actual:?}",
                invocation.source,
            ),
        ));
    }
    segment.checked_invocation = None;
    Ok(())
}

#[cfg(any(test, feature = "px8-ds-test-support"))]
thread_local! {
    static PX8DS_RETIRED_FLAT_ORDER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Test-only causal switch for the retired cross-instance flat ordering.
///
/// This is feature-gated so ordinary Runtime and CLI artifacts cannot select
/// the invalid ordering. PX8-DS integration tests use it to drive the exact
/// checked source through the former production consumer.
#[cfg(feature = "px8-ds-test-support")]
#[doc(hidden)]
pub fn with_px8ds_retired_flat_order<R>(run: impl FnOnce() -> R) -> R {
    struct Restore(bool);

    impl Drop for Restore {
        fn drop(&mut self) {
            PX8DS_RETIRED_FLAT_ORDER.with(|enabled| enabled.set(self.0));
        }
    }

    let previous = PX8DS_RETIRED_FLAT_ORDER.with(|enabled| enabled.replace(true));
    let _restore = Restore(previous);
    run()
}

fn px8ds_retired_flat_order_enabled() -> bool {
    #[cfg(any(test, feature = "px8-ds-test-support"))]
    {
        return PX8DS_RETIRED_FLAT_ORDER.with(std::cell::Cell::get);
    }
    #[cfg(not(any(test, feature = "px8-ds-test-support")))]
    {
        false
    }
}

fn compose_oriented_subcontinuation(
    plan: Option<&crate::OrientedSubcontinuationPlanV1>,
    invocation: Option<CheckedRecursiveInvocationInstance>,
    activation: ContinuationActivationId,
    mut segment: RecursorInvocationSegment,
    dynamic_splice_edges: Vec<DynamicSpliceEdge>,
) -> Result<InstalledOrientedSubcontinuationSegment, CraneliftBackendError> {
    segment.validate_open_control_obligations()?;
    let invocation = invocation.or(segment.checked_invocation);
    if let Some(invocation) = invocation {
        let plan = plan.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic invocation has no checked oriented plan",
            )
        })?;
        instantiate_checked_invocation_segment(plan, invocation, &mut segment)?;
    }
    let producer_origin = segment.origin;
    let sibling_position = segment.sibling_position;
    let resume_cursor = segment.resume_cursor;
    let mut control_layers =
        Vec::with_capacity(1 + segment.unwind.later_wrappers_in_construction_order.len());
    control_layers.push(segment.selection);
    control_layers.extend(
        segment
            .unwind
            .later_wrappers_in_construction_order
            .into_iter()
            .rev(),
    );
    let mut control_ledger = control_layers
        .iter()
        .map(|layer| OrientedControlLedgerEntry {
            frame_id: layer.checked_frame_id,
            invocation_id: layer.checked_invocation_id,
            role: layer.role,
            checked_witness: None,
        })
        .collect::<Vec<_>>();
    validate_oriented_control_projection(producer_origin, &control_layers)?;
    #[cfg(test)]
    px8j_record_source_event(Px8jSourceTraceEvent::DirectConsume {
        origin: segment.origin,
        selection_cursor: segment.resume_cursor,
        sibling_position: segment.sibling_position,
        exits: control_layers
            .iter()
            .rev()
            .filter_map(|layer| match layer.role {
                RecursorLayerRole::ExitsScope {
                    scope_origin,
                    parent_scope,
                    ..
                } => Some((scope_origin, parent_scope)),
                RecursorLayerRole::SelectsOccurrence { .. } => None,
            })
            .collect(),
    });

    // A selected frame is pending semantic work. A freshly instantiated IH
    // layer remains semantic even when its control projection is already in
    // ExitsScope phase. Inherited exit rows carry no fresh invocation source:
    // their transformer was consumed at selection and they remain only as
    // affine open-extent obligations in the control ledger.
    let semantic_layers = control_layers
        .iter()
        .filter(|layer| oriented_layer_is_pending_semantic(layer))
        .cloned()
        .collect::<Vec<_>>();

    let planned = semantic_layers
        .iter()
        .map(|layer| (layer.checked_invocation_id, layer.checked_frame_id))
        .collect::<Vec<_>>();
    let has_planned = planned.iter().any(|(_, frame)| frame.is_some());
    if has_planned
        && planned
            .iter()
            .any(|(invocation, frame)| invocation.is_none() || frame.is_none())
    {
        let detail = semantic_layers
            .iter()
            .map(|layer| {
                (
                    layer.checked_frame_id,
                    layer.checked_invocation_id,
                    layer.checked_invocation_depth,
                    layer.provenance.0,
                    layer
                        .cases
                        .iter()
                        .map(|case| case.constructor.as_str())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "oriented segment mixes checked and inferred computational frames: {detail:?}; recursive templates: {:?}",
                plan.map(|plan| (
                    plan.recursive_calls
                        .iter()
                        .map(|call| (call.call_template_id, call.declaration.as_str(), call.callee.as_str()))
                        .collect::<Vec<_>>(),
                    plan.computational_ih_calls
                        .iter()
                        .map(|call| (call.call_template_id, call.declaration.as_str(), call.slot_template_id))
                        .collect::<Vec<_>>()
                ))
            ),
        ));
    }

    let (segment_site_id, input_interface, output_interface, semantic_frames) = if has_planned {
        let plan = plan.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "oriented segment has no checked plan metadata",
            )
        })?;
        plan.validate()
            .map_err(|reason| unsupported("OrientedSubcontinuationPlanV1", reason))?;
        for entry in &mut control_ledger {
            if entry.invocation_id.is_none() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked control occurrence has no affine invocation identity",
                ));
            }
            let frame_id = entry.frame_id.expect("all control entries are checked");
            entry.checked_witness = Some(
                plan.frame(frame_id)
                    .expect("checked control entry has a validated plan row")
                    .control_witness
                    .clone(),
            );
        }
        let mut by_id = BTreeMap::<u64, Vec<u64>>::new();
        let mut layers_by_key = BTreeMap::new();
        for layer in semantic_layers {
            let frame_id = layer.checked_frame_id.expect("all frames are checked");
            let invocation_id = layer
                .checked_invocation_id
                .expect("all checked frames have an invocation instance");
            if layers_by_key
                .insert((invocation_id, frame_id), layer)
                .is_some()
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "oriented segment repeats a checked dynamic frame key",
                ));
            }
            by_id.entry(invocation_id).or_default().push(frame_id);
        }
        for frame_ids in by_id.values_mut() {
            frame_ids.sort_by_key(|frame_id| {
                plan.frame(*frame_id)
                    .expect("checked frame exists after plan validation")
                    .semantic_position
            });
            for pair in frame_ids.windows(2) {
                let left = plan.frame(pair[0]).expect("validated frame");
                let right = plan.frame(pair[1]).expect("validated frame");
                if left.segment_site_id != right.segment_site_id {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "invocation-local oriented segment crosses checked prompt regions",
                    ));
                }
                if left.output_interface != right.input_interface {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "invocation-local oriented segment endpoints do not compose",
                    ));
                }
            }
        }

        if px8ds_retired_flat_order_enabled() {
            let mut retired = layers_by_key
                .iter()
                .map(|((invocation_id, frame_id), layer)| {
                    (
                        *invocation_id,
                        plan.frame(*frame_id).expect("validated checked frame"),
                        layer,
                    )
                })
                .collect::<Vec<_>>();
            retired.sort_by_key(|(_, frame, layer)| {
                (
                    std::cmp::Reverse(layer.checked_invocation_depth),
                    frame.semantic_position,
                )
            });
            for pair in retired.windows(2) {
                if pair[0].1.output_interface != pair[1].1.input_interface {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        format!(
                            "retired flat oriented splice answer endpoints do not compose: left=(instance={}, frame={}, depth={}) right=(instance={}, frame={}, depth={})",
                            pair[0].0,
                            pair[0].1.frame_id,
                            pair[0].2.checked_invocation_depth,
                            pair[1].0,
                            pair[1].1.frame_id,
                            pair[1].2.checked_invocation_depth,
                        ),
                    ));
                }
            }
        }

        let mut edges_by_child = BTreeMap::new();
        let mut child_by_parent_frame = BTreeMap::new();
        for edge in dynamic_splice_edges {
            if edge.child_invocation_instance_id == edge.parent_invocation_instance_id {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge forms a self-parent cycle",
                ));
            }
            let child_frames = by_id
                .get(&edge.child_invocation_instance_id)
                .ok_or_else(|| {
                    unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic splice edge names a stale child invocation",
                    )
                })?;
            let parent_frames = by_id.get(&edge.parent_invocation_instance_id);
            if parent_frames.is_some_and(|frames| !frames.contains(&edge.parent_frame_template_id))
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge names the wrong static parent frame",
                ));
            }
            let call = plan
                .computational_ih_call(edge.checked_call_template_id)
                .ok_or_else(|| {
                    unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic splice edge names a stale checked call template",
                    )
                })?;
            if call.parent_frame_template_id != Some(edge.parent_frame_template_id)
                || call.parent_segment_site_id != Some(edge.segment_site_id)
                || call.callee_segment_site_id != edge.segment_site_id
                || call.callee_frame_templates != *child_frames
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge disagrees with its checked static parent",
                ));
            }
            if call.result_interface != call.caller_interface {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice call result does not match its checked caller interface",
                ));
            }
            if edges_by_child
                .insert(edge.child_invocation_instance_id, edge)
                .is_some()
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic child invocation carries duplicate affine splice edges",
                ));
            }
        }
        let mut external_children = BTreeMap::new();
        for edge in edges_by_child.values() {
            if by_id.contains_key(&edge.parent_invocation_instance_id) {
                let key = (
                    edge.parent_invocation_instance_id,
                    edge.parent_frame_template_id,
                );
                if child_by_parent_frame
                    .insert(key, edge.child_invocation_instance_id)
                    .is_some()
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "sibling dynamic invocations contend for one affine parent edge",
                    ));
                }
            } else {
                if edge.parent_invocation_instance_id != 0 {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic splice edge names a stale parent invocation",
                    ));
                }
                if external_children
                    .insert(
                        edge.parent_frame_template_id,
                        edge.child_invocation_instance_id,
                    )
                    .is_some()
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "sibling dynamic invocations compete for one external parent edge",
                    ));
                }
            }
        }
        let roots = if !external_children.is_empty() {
            if edges_by_child.len() != by_id.len() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge deletion leaves an unparented invocation-local segment",
                ));
            }
            let mut roots = external_children.into_iter().collect::<Vec<_>>();
            roots.sort_by_key(|(parent_frame, _)| {
                plan.frame(*parent_frame)
                    .expect("validated external parent frame")
                    .semantic_position
            });
            roots.into_iter().map(|(_, child)| child).collect()
        } else {
            by_id
                .keys()
                .filter(|instance| !edges_by_child.contains_key(instance))
                .copied()
                .collect::<Vec<_>>()
        };
        if roots.is_empty() || (edges_by_child.len() < by_id.len() && roots.len() != 1) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic splice edges do not form one exact invocation-local tree",
            ));
        }
        fn append_invocation_local_segment(
            invocation_id: u64,
            by_id: &BTreeMap<u64, Vec<u64>>,
            child_by_parent_frame: &BTreeMap<(u64, u64), u64>,
            visiting: &mut BTreeSet<u64>,
            completed: &mut BTreeSet<u64>,
            order: &mut Vec<(u64, u64)>,
        ) -> Result<(), CraneliftBackendError> {
            if completed.contains(&invocation_id) {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge is shared across sibling invocation paths",
                ));
            }
            if !visiting.insert(invocation_id) {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edges form a parent cycle",
                ));
            }
            for frame_id in by_id
                .get(&invocation_id)
                .expect("validated invocation-local segment exists")
            {
                if let Some(child) = child_by_parent_frame.get(&(invocation_id, *frame_id)) {
                    append_invocation_local_segment(
                        *child,
                        by_id,
                        child_by_parent_frame,
                        visiting,
                        completed,
                        order,
                    )?;
                }
                order.push((invocation_id, *frame_id));
            }
            visiting.remove(&invocation_id);
            completed.insert(invocation_id);
            Ok(())
        }
        let mut order = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut completed = BTreeSet::new();
        for root in roots {
            append_invocation_local_segment(
                root,
                &by_id,
                &child_by_parent_frame,
                &mut visiting,
                &mut completed,
                &mut order,
            )?;
        }
        if completed.len() != by_id.len() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic splice tree leaves an invocation-local segment unreachable",
            ));
        }
        let mut ordered = order
            .into_iter()
            .map(|key| {
                let layer = layers_by_key
                    .remove(&key)
                    .expect("validated dynamic frame key exists");
                let frame = plan.frame(key.1).expect("validated checked frame exists");
                (key.0, frame, layer)
            })
            .collect::<Vec<_>>();
        let site = ordered
            .first()
            .expect("checked oriented segment is nonempty")
            .1
            .segment_site_id;
        if ordered
            .iter()
            .any(|(_, frame, _)| frame.segment_site_id != site)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "oriented dynamic splice crosses checked prompt regions",
            ));
        }
        let input = ordered.first().unwrap().1.input_interface.clone();
        let output = ordered.last().unwrap().1.output_interface.clone();
        (
            Some(site),
            Some(input),
            Some(output),
            ordered.drain(..).map(|(_, _, layer)| layer).collect(),
        )
    } else {
        (None, None, None, semantic_layers)
    };

    let mut owned = OwnedOrientedSubcontinuationSegment {
        producer_origin,
        sibling_position,
        activation,
        segment_site_id,
        input_interface,
        output_interface,
        semantic_frames,
        control_ledger,
        resume_cursor,
        capability: AffineSpliceCapability {
            state: AffineSpliceState::Open,
        },
    };
    owned.capability.consume()?;
    debug_assert_eq!(owned.capability.state, AffineSpliceState::Consumed);
    debug_assert!(owned.control_ledger.len() >= owned.semantic_frames.len());
    debug_assert_eq!(
        owned.segment_site_id.is_some(),
        owned.input_interface.is_some()
    );
    debug_assert_eq!(
        owned.segment_site_id.is_some(),
        owned.output_interface.is_some()
    );
    Ok(InstalledOrientedSubcontinuationSegment {
        checked: owned.segment_site_id.is_some(),
        producer_origin: owned.producer_origin,
        sibling_position: owned.sibling_position,
        activation: owned.activation,
        semantic_frames: owned.semantic_frames,
        control_ledger: owned.control_ledger,
        resume_cursor: owned.resume_cursor,
    })
}

fn recursor_invocation_is_checked(segment: &RecursorInvocationSegment) -> bool {
    segment.selection.checked_frame_id.is_some()
        || segment
            .unwind
            .later_wrappers_in_construction_order
            .iter()
            .any(|layer| layer.checked_frame_id.is_some())
}

fn installed_oriented_eliminator_frames(
    segment: &InstalledOrientedSubcontinuationSegment,
) -> Vec<EliminatorFrame<'_>> {
    segment
        .semantic_frames
        .iter()
        .map(|layer| {
            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                cases: &layer.cases,
                default: &layer.default,
                env: &layer.outer_env,
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: layer.provenance,
                checked_frame_id: layer.checked_frame_id,
                checked_invocation_id: layer.checked_invocation_id,
                checked_invocation_source: layer.checked_invocation_source,
                checked_invocation_depth: layer.checked_invocation_depth,
            })
        })
        .collect()
}

/// Validate the control shape available at source-machine installation.
/// Parent adjacency is established by the return-hole continuation and belongs
/// only to the flattened-consumer validator below.
fn validate_recursor_invocation_install_shape(
    segment: &RecursorInvocationSegment,
) -> Result<(), CraneliftBackendError> {
    if !matches!(
        segment.selection.role,
        RecursorLayerRole::SelectsOccurrence { origin } if origin == segment.origin
    ) {
        return Err(unsupported(
            "ComputationalRecursor",
            "recursor selection role does not select the invocation origin",
        ));
    }
    let mut scope_origins = BTreeSet::new();
    for layer in &segment.unwind.later_wrappers_in_construction_order {
        let RecursorLayerRole::ExitsScope {
            origin,
            scope_origin,
            ..
        } = layer.role
        else {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        };
        if origin != segment.origin {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        }
        if !scope_origins.insert(scope_origin) {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind repeats a selected scope identity",
            ));
        }
    }
    Ok(())
}

fn validate_recursor_invocation_segment(
    segment: &RecursorInvocationSegment,
) -> Result<(), CraneliftBackendError> {
    if !matches!(
        segment.selection.role,
        RecursorLayerRole::SelectsOccurrence { origin } if origin == segment.origin
    ) {
        return Err(unsupported(
            "ComputationalRecursor",
            "recursor selection role does not select the invocation origin",
        ));
    }
    // Construction order is outer-to-inner, while execution pops the vector
    // inner-to-outer. An outermost scope may name a parent owned by the caller;
    // every carried successor must link to the immediately preceding scope.
    let mut scope_origins = BTreeSet::new();
    let mut previous_scope = None;
    for layer in &segment.unwind.later_wrappers_in_construction_order {
        let RecursorLayerRole::ExitsScope {
            origin,
            scope_origin,
            parent_scope,
        } = layer.role
        else {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        };
        if origin != segment.origin {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        }
        if !scope_origins.insert(scope_origin) {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind repeats a selected scope identity",
            ));
        }
        if let Some(previous_scope) = previous_scope {
            if parent_scope != Some(previous_scope) {
                return Err(unsupported(
                    "ComputationalRecursor",
                    "recursor unwind has a broken selected-scope parent link",
                ));
            }
        }
        previous_scope = Some(scope_origin);
    }
    Ok(())
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

fn active_context_contains_cursor(
    active: &ActiveContinuationFrame<'_>,
    cursor: ContinuationCursorId,
) -> bool {
    find_continuation_cursor(active, cursor).is_some()
        || active.source_selected_cursor == Some(cursor)
        || active.source_lineage.iter().rev().any(|candidate| {
            let candidate = candidate.as_active(active.source_lineage);
            find_continuation_cursor(&candidate, cursor).is_some()
        })
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceComputationalAnswerRoute {
    DirectScrutinee,
    CheckedSelectedRecursor,
}

impl SourceComputationalAnswerRoute {
    fn for_recursor_layer(layer: &ComputationalRecursorLayer) -> Self {
        if layer.checked_frame_id.is_some()
            && matches!(layer.role, RecursorLayerRole::SelectsOccurrence { .. })
        {
            Self::CheckedSelectedRecursor
        } else {
            Self::DirectScrutinee
        }
    }
}

fn source_case_has_no_checked_control_markers(expr: &RuntimeExpr) -> bool {
    let mut frames = BTreeMap::new();
    if collect_checked_subcontinuation_frames(expr, &mut frames).is_err() || !frames.is_empty() {
        return false;
    }
    let mut markers = CheckedOrientedMarkerSets::default();
    collect_checked_oriented_markers(expr, &mut markers, "<source-case>", &mut Vec::new()).is_ok()
        && markers.recursive_calls.is_empty()
        && markers.computational_ih_slots.is_empty()
        && markers.computational_ih_calls.is_empty()
}

enum SourceContinuation<'a> {
    Terminal(SourceContinuationTerminal<'a>),
    CheckedRecursiveInvocationReturn {
        instance: CheckedRecursiveInvocationInstance,
        next: Box<SourceContinuation<'a>>,
    },
    CheckedComputationalIHInvocationReturn {
        call_template_id: u64,
        next: Box<SourceContinuation<'a>>,
    },
    ReturnFromSelectedCase {
        delimiter: SelectedCaseReturnDelimiter,
        next: Box<SourceContinuation<'a>>,
    },
    LetBody {
        body: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ApplyRecursorSelection {
        layer: ComputationalRecursorLayer,
        next: Box<SourceContinuation<'a>>,
    },
    UnwindRecursorSegment {
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
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
        checked_frame_id: Option<u64>,
        answer_route: SourceComputationalAnswerRoute,
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
    /// The unique affine handoff from source evaluation back to the producer.
    /// The stored unwind segment is consumed here; it is not inferred from
    /// provenance or reconstructed from the cursor.
    ReturnToProducerHole {
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        expected: ContinuationCursorId,
        active: &'a ActiveContinuationFrame<'a>,
        root_authority: Option<RootTerminalAnswerAuthority>,
    },
    ResumeOuter {
        expected: ContinuationCursorId,
        active: &'a ActiveContinuationFrame<'a>,
        root_authority: Option<RootTerminalAnswerAuthority>,
    },
    JumpToJoin(SourcePredecessorEdge<'a>),
}

#[derive(Clone)]
struct SourceJoinTarget<'a> {
    join_id: u64,
    block: cranelift_codegen::ir::Block,
    expected_outer: ContinuationCursorId,
    required_kind: ScalarMergeKind,
    terminal_active_prefix: Vec<EliminatorFrame<'a>>,
}

/// An affine capability for one mutually exclusive predecessor of a checked
/// source join. The target description is shareable; this edge deliberately is
/// not `Clone`, so a predecessor can either seal its edge or consume it into a
/// branch fan-out, never replay it.
struct SourcePredecessorEdge<'a> {
    target: SourceJoinTarget<'a>,
    predecessor_identity: u64,
}

/// A cloneable source-evaluation prefix with its terminal edge removed. A
/// branch fan-out may materialize this prefix once per mutually exclusive CFG
/// arm, but the post-cut suffix and executable predecessor edge never live in
/// the template.
#[derive(Clone)]
enum SourcePrefixTemplate {
    Terminal {
        expected_outer: ContinuationCursorId,
    },
    CheckedRecursiveInvocationReturn {
        instance: CheckedRecursiveInvocationInstance,
        next: Box<SourcePrefixTemplate>,
    },
    CheckedComputationalIHInvocationReturn {
        call_template_id: u64,
        next: Box<SourcePrefixTemplate>,
    },
    ReturnFromSelectedCase {
        delimiter: SelectedCaseReturnDelimiter,
        next: Box<SourcePrefixTemplate>,
    },
    LetBody {
        body: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    ApplyRecursorSelection {
        layer: ComputationalRecursorLayer,
        next: Box<SourcePrefixTemplate>,
    },
    UnwindRecursorSegment {
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        next: Box<SourcePrefixTemplate>,
    },
    IfScrutinee {
        then_expr: RuntimeExpr,
        else_expr: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    MatchScrutinee {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    ComputationalMatchScrutinee {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        checked_frame_id: Option<u64>,
        answer_route: SourceComputationalAnswerRoute,
        next: Box<SourcePrefixTemplate>,
    },
    ProjectRecord {
        field: String,
        next: Box<SourcePrefixTemplate>,
    },
    CallCallee {
        args: Vec<RuntimeExpr>,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    CallArgument {
        callee: Lowered,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
}

enum SourcePrefixTerminal<'a> {
    ResumeOuter {
        root_authority: Option<RootTerminalAnswerAuthority>,
    },
    Join(SourcePredecessorEdge<'a>),
}

struct SourceBranchFanout<'a> {
    source_prefix_template: SourcePrefixTemplate,
    inherited_edge: SourcePredecessorEdge<'a>,
}

struct ArmedInvocation<'a> {
    suspended: SourceControl<'a>,
    expected_selected: ContinuationCursorId,
}

#[derive(Clone)]
struct SourceSelectedContinuation<'a> {
    activation: ContinuationActivationId,
    cursor: ContinuationCursorId,
    parent: Option<&'a ActiveContinuationFrame<'a>>,
    pending: Vec<EliminatorFrame<'a>>,
    selected_ancestry: Vec<RecursorFrameProvenance>,
    selected_scope: Option<OwnedSelectedScope>,
}

impl<'a> SourceSelectedContinuation<'a> {
    fn as_active<'b>(
        &'b self,
        source_lineage: &'b [SourceSelectedContinuation<'a>],
    ) -> ActiveContinuationFrame<'b>
    where
        'a: 'b,
    {
        ActiveContinuationFrame {
            activation: self.activation,
            cursor: self.cursor,
            parent: self.parent,
            pending: &self.pending,
            selected_ancestry: &self.selected_ancestry,
            source_lineage,
            source_selected_cursor: Some(self.cursor),
            selected_scope: self.selected_scope.as_ref(),
        }
    }
}

fn source_active_cursor<'a: 'b, 'b>(
    selected: &'b SourceSelectedContinuation<'a>,
    lineage: &'b [SourceSelectedContinuation<'a>],
    cursor: ContinuationCursorId,
) -> Option<ActiveContinuationFrame<'b>> {
    std::iter::once(selected)
        .chain(lineage.iter().rev())
        .find_map(|candidate| {
            let mut active = candidate.as_active(lineage);
            active.source_selected_cursor = Some(selected.cursor);
            if active.cursor == cursor {
                Some(active)
            } else {
                let mut parent = active.parent;
                while let Some(frame) = parent {
                    if frame.cursor == cursor {
                        let mut frame = *frame;
                        frame.source_lineage = lineage;
                        frame.source_selected_cursor = Some(selected.cursor);
                        return Some(frame);
                    }
                    parent = frame.parent;
                }
                None
            }
        })
}

struct SourceControl<'a> {
    continuation: SourceContinuation<'a>,
    selected: SourceSelectedContinuation<'a>,
    selected_lineage: Vec<SourceSelectedContinuation<'a>>,
    terminal_outer: ContinuationCursorId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SelectedCaseReturnDelimiter {
    activation: ContinuationActivationId,
    cursor: ContinuationCursorId,
    scope_origin: RecursorProducerOriginId,
    frame_id: Option<u64>,
    invocation_id: Option<u64>,
}

enum SourceMachineState<'a> {
    Eval {
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'a>,
    },
    Value {
        value: Lowered,
        control: SourceControl<'a>,
    },
}

enum SourceCallOutcome<'a> {
    Continue(SourceMachineState<'a>),
    Complete(Lowered),
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

/// Proof token for the legacy closed-expression merge sites. It can only be
/// minted when source evaluation has no live continuation. Checked source joins
/// use their explicit `SourceJoinTarget.required_kind` instead.
/// Move-only proof that the native lowering machine has reached the checked
/// invocation root with no semantic or control continuation left to consume
/// the value.
struct RootTerminalAnswerAuthority {
    site_id: u64,
    checked_result_type_fingerprint: u64,
    occurrence_binding_fingerprint: u64,
    outer_cursor: Option<ContinuationCursorId>,
}

struct TerminalAnswerAuthority;

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
        RuntimeExpr::CheckedJoinSite { body, .. } => {
            produces_deforestable_aggregate_with_ih(body, aggregate_ihs)
        }
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
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
            collect_runtime_declaration_refs(body, output)
        }
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
    fn mint_recursor_producer_origin(&mut self) -> RecursorProducerOriginId {
        let origin = RecursorProducerOriginId(self.next_recursor_producer_origin);
        self.next_recursor_producer_origin = self
            .next_recursor_producer_origin
            .checked_add(1)
            .expect("compiler-private recursor producer origin exhausted");
        origin
    }

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

    fn enter_checked_subcontinuation_frame(
        &mut self,
        frame_id: u64,
    ) -> Result<(), CraneliftBackendError> {
        if self
            .active_subcontinuation_frame
            .replace(frame_id)
            .is_some()
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "nested checked subcontinuation occurrence marker",
            ));
        }
        Ok(())
    }

    fn enter_checked_recursive_invocation(
        &mut self,
        call_template_id: u64,
        body: &RuntimeExpr,
    ) -> Result<CheckedRecursiveInvocationInstance, CraneliftBackendError> {
        if self.pending_recursive_call.is_some() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "nested unchecked recursive invocation marker",
            ));
        }
        let call = self
            .oriented_subcontinuation_plan
            .as_ref()
            .and_then(|plan| plan.recursive_call(call_template_id))
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "recursive invocation marker has no checked call template",
                )
            })?;
        let RuntimeExpr::Call { callee, args } = body else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker does not wrap one complete call",
            ));
        };
        if !matches!(callee.as_ref(), RuntimeExpr::DeclarationRef { symbol } if symbol == &call.callee)
            || args.len() as u64 != call.arity
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker callee or arity is stale",
            ));
        }
        if !self
            .consumed_recursive_call_templates
            .insert(call_template_id)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation occurrence marker was consumed twice",
            ));
        }
        let instance = CheckedRecursiveInvocationInstance {
            source: InvocationTemplateRef::SameSccCall(call_template_id),
            invocation_instance_id: self.next_recursive_invocation_instance,
            semantic_depth: self.active_recursive_invocations.len() + 1,
            dynamic_splice_edge: None,
        };
        self.next_recursive_invocation_instance = self
            .next_recursive_invocation_instance
            .checked_add(1)
            .expect("compiler-private recursive invocation identity exhausted");
        self.pending_recursive_call = Some(instance);
        self.active_recursive_invocations.push(instance);
        Ok(instance)
    }

    fn leave_checked_recursive_invocation(
        &mut self,
        instance: CheckedRecursiveInvocationInstance,
    ) -> Result<(), CraneliftBackendError> {
        if self.pending_recursive_call == Some(instance) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker was not consumed by its call",
            ));
        }
        if self.active_recursive_invocations.pop() != Some(instance) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation instance stack is not affine",
            ));
        }
        Ok(())
    }

    fn enter_checked_computational_ih_invocation(
        &mut self,
        call_template_id: u64,
    ) -> Result<(), CraneliftBackendError> {
        if self
            .pending_computational_ih_call
            .replace(call_template_id)
            .is_some()
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "nested computational IH invocation marker",
            ));
        }
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation marker has no checked plan",
            )
        })?;
        plan.computational_ih_call(call_template_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH invocation marker has no checked call template",
                )
            })?;
        Ok(())
    }

    fn mint_checked_computational_ih_instance(
        &mut self,
        value: &mut Lowered,
    ) -> Result<Option<CheckedRecursiveInvocationInstance>, CraneliftBackendError> {
        let Some(call_template_id) = self.pending_computational_ih_call.take() else {
            return Ok(None);
        };
        let Lowered::ComputationalRecursorClosure { invocation, .. } = value else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH marker was applied to an ordinary value",
            ));
        };
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked plan",
            )
        })?;
        let call = plan
            .computational_ih_call(call_template_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH invocation has no checked call template",
                )
            })?;
        if invocation.computational_ih_slot_template_id != Some(call.slot_template_id) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation marker names a different slot",
            ));
        }
        let parent_frame_template_id = call.parent_frame_template_id.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked static parent",
            )
        })?;
        let segment_site_id = call.parent_segment_site_id.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked parent segment",
            )
        })?;
        let mut parents = std::iter::once(&invocation.selection)
            .chain(
                invocation
                    .unwind
                    .later_wrappers_in_construction_order
                    .iter(),
            )
            .filter(|layer| {
                layer.semantic_pending && layer.checked_frame_id == Some(parent_frame_template_id)
            });
        let selected = parents.next().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH closure has no exact checked open parent occurrence",
            )
        })?;
        if parents.next().is_some() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH closure has multiple candidate dynamic parent occurrences",
            ));
        }
        let parent_invocation_instance_id = match selected.checked_invocation_id {
            Some(instance_id) => instance_id,
            None if selected.checked_invocation_source.is_none() => 0,
            None => {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    format!(
                        "computational IH closure-selected occurrence has no dynamic parent identity: frame={:?} source={:?} depth={} handles={:?}",
                        selected.checked_frame_id,
                        selected.checked_invocation_source,
                        selected.checked_invocation_depth,
                        invocation.dynamic_splice_edges,
                    ),
                ))
            }
        };
        let selected_site = plan
            .frame(parent_frame_template_id)
            .map(|frame| frame.segment_site_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH closure-selected occurrence names a stale parent frame",
                )
            })?;
        if selected_site != segment_site_id {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH closure-selected occurrence crosses its checked segment",
            ));
        }
        let edge_id = DynamicSpliceEdgeId(self.next_dynamic_splice_edge);
        self.next_dynamic_splice_edge = self
            .next_dynamic_splice_edge
            .checked_add(1)
            .expect("compiler-private dynamic splice edge identity exhausted");
        let instance = CheckedRecursiveInvocationInstance {
            source: InvocationTemplateRef::ComputationalIHCall(call_template_id),
            invocation_instance_id: self.next_recursive_invocation_instance,
            semantic_depth: self.active_recursive_invocations.len() + 1,
            dynamic_splice_edge: Some(edge_id),
        };
        self.next_recursive_invocation_instance = self
            .next_recursive_invocation_instance
            .checked_add(1)
            .expect("compiler-private invocation identity exhausted");
        if self
            .dynamic_splice_edges
            .insert(
                edge_id,
                DynamicSpliceEdge {
                    edge_id,
                    child_invocation_instance_id: instance.invocation_instance_id,
                    parent_invocation_instance_id,
                    checked_call_template_id: call_template_id,
                    parent_frame_template_id,
                    segment_site_id,
                },
            )
            .is_some()
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic splice edge identity was minted twice",
            ));
        }
        invocation.dynamic_splice_edges.push(edge_id);
        Ok(Some(instance))
    }

    fn validate_source_dynamic_splice_parent(
        &self,
        instance: CheckedRecursiveInvocationInstance,
        open: &OwnedSelectedScope,
    ) -> Result<(), CraneliftBackendError> {
        let edge_id = instance.dynamic_splice_edge.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "source IH invocation has no affine dynamic splice edge",
            )
        })?;
        let edge = self.dynamic_splice_edges.get(&edge_id).ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "source IH invocation names a deleted or already-consumed dynamic splice edge",
            )
        })?;
        if edge.child_invocation_instance_id != instance.invocation_instance_id
            || edge.parent_invocation_instance_id != open.frame.checked_invocation_id.unwrap_or(0)
            || Some(edge.parent_frame_template_id) != open.frame.checked_frame_id
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "source open occurrence disagrees with the closure-selected dynamic parent",
            ));
        }
        Ok(())
    }

    fn take_dynamic_splice_edges(
        &mut self,
        segment: &RecursorInvocationSegment,
    ) -> Result<Vec<DynamicSpliceEdge>, CraneliftBackendError> {
        let mut seen = BTreeSet::new();
        let mut edges = Vec::with_capacity(segment.dynamic_splice_edges.len());
        for edge_id in &segment.dynamic_splice_edges {
            if !seen.insert(*edge_id) {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge handle is duplicated in one invocation carrier",
                ));
            }
            let edge = self.dynamic_splice_edges.remove(edge_id).ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge was deleted, replayed, or consumed by a sibling",
                )
            })?;
            if edge.edge_id != *edge_id {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge ledger identity is stale",
                ));
            }
            edges.push(edge);
        }
        Ok(edges)
    }

    fn finish_checked_computational_ih_marker(
        &mut self,
        mut value: Lowered,
    ) -> Result<Lowered, CraneliftBackendError> {
        let Some(instance) = self.mint_checked_computational_ih_instance(&mut value)? else {
            return Ok(value);
        };
        let Lowered::ComputationalRecursorClosure { invocation, .. } = &mut value else {
            unreachable!("IH instance mint validates one recursor closure")
        };
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked plan",
            )
        })?;
        // Qualify the exact reusable template sequence at marker consumption.
        // Existing child-qualified layers remain untouched when later parent
        // wrappers are added to the same flattened carrier.
        instantiate_checked_invocation_segment(plan, instance, invocation)?;
        Ok(value)
    }

    fn consume_checked_recursive_invocation_call(
        &mut self,
        symbol: &RuntimeSymbol,
    ) -> Result<Option<CheckedRecursiveInvocationInstance>, CraneliftBackendError> {
        let Some(instance) = self.pending_recursive_call.take() else {
            return Ok(None);
        };
        let InvocationTemplateRef::SameSccCall(call_template_id) = instance.source else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "same-SCC call consumer received a computational IH invocation",
            ));
        };
        let call = self
            .oriented_subcontinuation_plan
            .as_ref()
            .and_then(|plan| plan.recursive_call(call_template_id))
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "active recursive invocation has no checked template",
                )
            })?;
        if &call.callee != symbol {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker was transplanted to another callee",
            ));
        }
        Ok(Some(instance))
    }

    fn consume_checked_subcontinuation_frame(
        &mut self,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
    ) -> Result<Option<u64>, CraneliftBackendError> {
        let Some(frame_id) = self.active_subcontinuation_frame.take() else {
            return Ok(None);
        };
        let frame = self
            .oriented_subcontinuation_plan
            .as_ref()
            .and_then(|plan| plan.frame(frame_id))
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked Runtime marker has no transported frame entry",
                )
            })?;
        if frame.runtime_frame_fingerprint
            != crate::compiler_private_computational_match_frame_fingerprint(cases, default)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked Runtime marker no longer denotes its planned frame",
            ));
        }
        let invocation_id = self
            .active_recursive_invocations
            .last()
            .map_or(0, |instance| instance.invocation_instance_id);
        if !self
            .consumed_subcontinuation_frames
            .insert((invocation_id, frame_id))
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked Runtime frame marker was consumed more than once",
            ));
        }
        Ok(Some(frame_id))
    }

    fn computational_ih_slots_for_case(
        &self,
        case: &crate::RuntimeComputationalMatchCase,
        checked_frame_id: Option<u64>,
    ) -> Result<Vec<Option<u64>>, CraneliftBackendError> {
        let RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids, ..
        } = &case.body
        else {
            if checked_frame_id.is_some() && !case.recursive_positions.is_empty() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked computational case is missing its IH slot marker",
                ));
            }
            return Ok(vec![None; case.recursive_positions.len()]);
        };
        let frame_id = checked_frame_id.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH slot marker is detached from its checked frame",
            )
        })?;
        if slot_template_ids.len() != case.recursive_positions.len() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH slot marker is not bijective with recursive positions",
            ));
        }
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH slot marker has no checked plan",
            )
        })?;
        let mut seen = BTreeSet::new();
        slot_template_ids
            .iter()
            .copied()
            .zip(case.recursive_positions.iter().copied())
            .map(|(slot_template_id, recursive_position)| {
                if !seen.insert(slot_template_id) {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "computational IH case repeats a checked slot template",
                    ));
                }
                let slot = plan
                    .computational_ih_slot(slot_template_id)
                    .ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "computational IH case names a stale slot template",
                        )
                    })?;
                if slot.frame_template_id != frame_id
                    || slot.constructor != case.constructor
                    || slot.recursive_position != recursive_position as u64
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "computational IH slot constructor/position/frame binding is stale",
                    ));
                }
                Ok(Some(slot_template_id))
            })
            .collect()
    }

    fn enter_oriented_semantic_region(&mut self, checked: bool) {
        if checked {
            self.active_oriented_semantic_regions = self
                .active_oriented_semantic_regions
                .checked_add(1)
                .expect("compiler-private oriented segment depth exhausted");
        }
    }

    fn leave_oriented_semantic_region(&mut self, checked: bool) {
        if checked {
            self.active_oriented_semantic_regions = self
                .active_oriented_semantic_regions
                .checked_sub(1)
                .expect("oriented semantic region must be entered exactly once");
        }
    }

    fn make_computational_recursor(
        &mut self,
        recursive: Lowered,
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        outer_env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        checked_frame_id: Option<u64>,
        computational_ih_slot_template_id: Option<u64>,
        origin: RecursorProducerOriginId,
        sibling_position: usize,
        role: RecursorLayerRole,
        activation: ContinuationActivationId,
        resume_cursor: ContinuationCursorId,
        splice_caller: Option<&ActiveContinuationFrame<'_>>,
        source_control: Option<(
            &SourceSelectedContinuation<'_>,
            &[SourceSelectedContinuation<'_>],
        )>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (residual, payload) = decompose_computational_recursor(recursive);
        let active_instance = self.active_recursive_invocations.last().copied();
        let inferred_frame_id = if checked_frame_id.is_none() {
            active_instance.and_then(|instance| {
                let fingerprint =
                    crate::compiler_private_computational_match_frame_fingerprint(&cases, &default);
                self.oriented_subcontinuation_plan
                    .as_ref()
                    .and_then(|plan| checked_invocation_frame_templates(plan, instance.source).ok())
                    .and_then(|frame_templates| {
                        frame_templates.iter().copied().find(|frame_id| {
                            self.oriented_subcontinuation_plan
                                .as_ref()
                                .and_then(|plan| plan.frame(*frame_id))
                                .is_some_and(|frame| frame.runtime_frame_fingerprint == fingerprint)
                        })
                    })
            })
        } else {
            checked_frame_id
        };
        let invocation_id = inferred_frame_id
            .and_then(|_| active_instance.map(|instance| instance.invocation_instance_id));
        let invocation_source = active_instance.map(|instance| instance.source);
        let invocation_depth = active_instance.map_or(0, |instance| instance.semantic_depth);
        let mut current_layer = ComputationalRecursorLayer {
            cases,
            default,
            outer_env,
            provenance,
            role,
            checked_frame_id: inferred_frame_id,
            checked_invocation_id: invocation_id,
            checked_invocation_source: invocation_source,
            checked_invocation_depth: invocation_depth,
            semantic_pending: true,
        };
        let segment_origin = payload
            .as_ref()
            .map(|(_, invocation)| invocation.origin)
            .unwrap_or(origin);
        let segment_sibling_position = payload
            .as_ref()
            .map(|(_, invocation)| invocation.sibling_position)
            .unwrap_or(sibling_position);
        let segment_checked_invocation = payload
            .as_ref()
            .and_then(|(_, invocation)| invocation.checked_invocation)
            .or(active_instance);
        let segment_dynamic_splice_edges = payload
            .as_ref()
            .map(|(_, invocation)| invocation.dynamic_splice_edges.clone())
            .unwrap_or_default();
        let (selection, unwind) =
            if let Some((_, invocation)) = payload {
                let splice_caller = splice_caller.ok_or_else(|| {
                    unsupported(
                        "ComputationalRecursor",
                        "recursive payload splice has no active continuation",
                    )
                })?;
                let source_cursor_is_live = source_control.is_some_and(|(selected, lineage)| {
                    source_active_cursor(selected, lineage, invocation.resume_cursor).is_some()
                });
                if !active_context_contains_cursor(splice_caller, invocation.resume_cursor)
                    && !source_cursor_is_live
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "recursive payload resume cursor is not active",
                    ));
                }
                let mut unwind = invocation.unwind;
                let parent_scope = unwind.later_wrappers_in_construction_order.last().and_then(
                    |layer| match layer.role {
                        RecursorLayerRole::ExitsScope { scope_origin, .. } => Some(scope_origin),
                        RecursorLayerRole::SelectsOccurrence { .. } => None,
                    },
                );
                let unwind_role = match role {
                    RecursorLayerRole::SelectsOccurrence { origin: _ } => {
                        RecursorLayerRole::ExitsScope {
                            origin: segment_origin,
                            scope_origin: origin,
                            parent_scope,
                        }
                    }
                    RecursorLayerRole::ExitsScope {
                        origin: _,
                        scope_origin,
                        parent_scope,
                    } => RecursorLayerRole::ExitsScope {
                        origin: segment_origin,
                        scope_origin,
                        parent_scope,
                    },
                };
                current_layer.role = unwind_role;
                unwind
                    .later_wrappers_in_construction_order
                    .push(current_layer);
                if let Some((selected, lineage)) = source_control {
                    if selected.selected_scope.is_none() {
                        return Err(unsupported(
                            "ComputationalRecursor",
                            "source recursor invocation is missing its owned selected scope",
                        ));
                    }
                    for scope in lineage
                        .iter()
                        .filter_map(|selected| selected.selected_scope.as_ref())
                        .chain(selected.selected_scope.iter())
                    {
                        if unwind
                            .later_wrappers_in_construction_order
                            .iter()
                            .any(|layer| {
                                matches!(
                                    layer.role,
                                    RecursorLayerRole::ExitsScope { scope_origin, .. }
                                        if scope_origin == scope.scope_origin
                                )
                            })
                        {
                            continue;
                        }
                        unwind.later_wrappers_in_construction_order.push(
                            ComputationalRecursorLayer {
                                cases: scope.frame.cases.clone(),
                                default: scope.frame.default.clone(),
                                outer_env: scope.frame.outer_env.clone(),
                                provenance: scope.frame.provenance,
                                checked_frame_id: scope.frame.checked_frame_id,
                                checked_invocation_id: scope.frame.checked_invocation_id,
                                checked_invocation_source: scope.frame.checked_invocation_source,
                                checked_invocation_depth: scope.frame.checked_invocation_depth,
                                semantic_pending: false,
                                role: RecursorLayerRole::ExitsScope {
                                    origin: segment_origin,
                                    scope_origin: scope.scope_origin,
                                    parent_scope: scope.parent_scope,
                                },
                            },
                        );
                    }
                }
                (invocation.selection, unwind)
            } else {
                (
                    current_layer,
                    RecursorUnwindStack {
                        later_wrappers_in_construction_order: Vec::new(),
                    },
                )
            };
        let mut invocation = RecursorInvocationSegment::new(
            segment_origin,
            segment_sibling_position,
            selection,
            unwind,
            resume_cursor,
            segment_checked_invocation,
            computational_ih_slot_template_id,
        );
        invocation.dynamic_splice_edges = segment_dynamic_splice_edges;
        Ok(Lowered::ComputationalRecursorClosure {
            residual: Box::new(residual),
            activation,
            invocation,
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
            source_lineage: active.source_lineage,
            source_selected_cursor: active.source_selected_cursor,
            selected_scope: active.selected_scope,
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
        let checked_frame_id = self.consume_checked_subcontinuation_frame(cases, default)?;
        let checked_invocation_id = checked_frame_id.map(|_| {
            self.active_recursive_invocations
                .last()
                .map_or(0, |instance| instance.invocation_instance_id)
        });
        let checked_invocation_depth = self
            .active_recursive_invocations
            .last()
            .map_or(0, |instance| instance.semantic_depth);
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
                    checked_frame_id,
                    checked_invocation_id,
                    checked_invocation_source: self
                        .active_recursive_invocations
                        .last()
                        .map(|instance| instance.source),
                    checked_invocation_depth,
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
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let result = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                );
                if self.active_subcontinuation_frame.take().is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "checked subcontinuation marker was not consumed by its frame",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                body,
                ..
            } => {
                let instance = self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let result = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                );
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                self.lower_computational_producer_expr(builder, body, producer_env, eliminators)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let value = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                )?;
                self.finish_checked_computational_ih_marker(value)
            }
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
                                    let (activation, invocation) = boundary.expect(
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
                                    if !recursor_invocation_is_checked(&invocation) {
                                        validate_recursor_invocation_segment(&invocation)?;
                                    }
                                    let dynamic_splice_edges =
                                        self.take_dynamic_splice_edges(&invocation)?;
                                    let installed = compose_oriented_subcontinuation(
                                        self.oriented_subcontinuation_plan.as_ref(),
                                        self.active_recursive_invocations.last().copied(),
                                        activation,
                                        invocation,
                                        dynamic_splice_edges,
                                    )?;
                                    let frames = installed_oriented_eliminator_frames(&installed);
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
                                    self.enter_oriented_semantic_region(installed.checked);
                                    let returned = self.lower_computational_producer_expr(
                                        builder,
                                        value,
                                        producer_env,
                                        &composed,
                                    );
                                    self.leave_oriented_semantic_region(installed.checked);
                                    let returned = returned?;
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
                    mut callee @ Lowered::ComputationalRecursorClosure { .. } => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(callee);
                        let (activation, invocation) =
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
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let mut composed = installed_oriented_eliminator_frames(&installed);
                        composed.push(EliminatorFrame::InvocationReturn);
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            self.enter_oriented_semantic_region(installed.checked);
                            let returned = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &composed,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            let returned = returned?;
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
                        self.enter_oriented_semantic_region(installed.checked);
                        let returned = self.lower_computational_producer_expr(
                            builder, &body, &call_env, &composed,
                        );
                        self.leave_oriented_semantic_region(installed.checked);
                        let returned = returned?;
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
                        source_lineage: splice_caller
                            .map(|active| active.source_lineage)
                            .unwrap_or(&[]),
                        source_selected_cursor: splice_caller
                            .and_then(|active| active.source_selected_cursor),
                        selected_scope: splice_caller.and_then(|active| active.selected_scope),
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
                                checked_frame_id: None,
                                checked_invocation_id: None,
                                checked_invocation_source: None,
                                checked_invocation_depth: 0,
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
                        builder
                            .ins()
                            .jump(merge, &[value.tag.into(), value.payload.into()]);
                    }
                    builder.switch_to_block(merge);
                    let pair = NativeScalarPairV1 {
                        tag: builder.block_params(merge)[0],
                        payload: builder.block_params(merge)[1],
                    };
                    return Ok(if exit_merge == Some(true) {
                        Lowered::ProcessExitStatus {
                            value: pair.payload,
                        }
                    } else {
                        self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
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
                        builder
                            .ins()
                            .jump(merge, &[value.tag.into(), value.payload.into()]);
                    }
                    builder.switch_to_block(merge);
                    let pair = NativeScalarPairV1 {
                        tag: builder.block_params(merge)[0],
                        payload: builder.block_params(merge)[1],
                    };
                    return Ok(if exit_merge == Some(true) {
                        Lowered::ProcessExitStatus {
                            value: pair.payload,
                        }
                    } else {
                        self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
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
                let checked_frame_id =
                    self.consume_checked_subcontinuation_frame(inner_cases, inner_default)?;
                let checked_invocation_id = checked_frame_id.map(|_| {
                    self.active_recursive_invocations
                        .last()
                        .map_or(0, |instance| instance.invocation_instance_id)
                });
                let checked_invocation_depth = self
                    .active_recursive_invocations
                    .last()
                    .map_or(0, |instance| instance.semantic_depth);
                composed.push(EliminatorFrame::Computational(
                    ComputationalEliminatorFrame {
                        cases: inner_cases,
                        default: inner_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                        provenance,
                        checked_frame_id,
                        checked_invocation_id,
                        checked_invocation_source: self
                            .active_recursive_invocations
                            .last()
                            .map(|instance| instance.source),
                        checked_invocation_depth,
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
                    builder
                        .ins()
                        .jump(merge, &[value.tag.into(), value.payload.into()]);
                }
                builder.switch_to_block(merge);
                let pair = NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                };
                Ok(if exit_merge == Some(true) {
                    Lowered::ProcessExitStatus {
                        value: pair.payload,
                    }
                } else {
                    self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
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
                let producer_origin = self.mint_recursor_producer_origin();
                let selected_scope = OwnedSelectedScope {
                    scope_origin: producer_origin,
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                    frame: ComputationalRecursorFramePayload {
                        cases: eliminator.cases.to_vec(),
                        default: eliminator.default.clone(),
                        outer_env: eliminator.env.to_vec(),
                        provenance: eliminator.provenance,
                        checked_frame_id: eliminator.checked_frame_id,
                        checked_invocation_id: eliminator.checked_invocation_id,
                        checked_invocation_source: eliminator.checked_invocation_source,
                        checked_invocation_depth: eliminator.checked_invocation_depth,
                    },
                };
                let selected_scope = Some(selected_scope);
                let active_state = ActiveContinuationFrame {
                    activation,
                    cursor,
                    parent: splice_caller.and_then(|active| active.parent),
                    pending: &pending,
                    selected_ancestry: &selected_ancestry,
                    source_lineage: splice_caller
                        .map(|active| active.source_lineage)
                        .unwrap_or(&[]),
                    source_selected_cursor: splice_caller
                        .and_then(|active| active.source_selected_cursor),
                    selected_scope: selected_scope.as_ref(),
                };

                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::Composed,
                    origin: producer_origin,
                    cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                });
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                let ih_slots =
                    self.computational_ih_slots_for_case(case, eliminator.checked_frame_id)?;
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let induction_hypothesis = self.make_computational_recursor(
                        args[position].clone(),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.provenance,
                        eliminator.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        activation,
                        cursor,
                        splice_caller,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
                    induction_hypotheses.push(induction_hypothesis);
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
                if !case.recursive_positions.is_empty() {
                    return self.lower_source_machine(
                        builder,
                        &case.body,
                        &case_env,
                        &active_state,
                    );
                }
                if remaining_eliminators.is_empty() {
                    return self.lower_expr(builder, &case.body, &case_env);
                }
                return self.lower_computational_producer_expr(
                    builder,
                    &case.body,
                    &case_env,
                    remaining_eliminators,
                );
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
        builder.append_block_param(loop_block, types::I64);
        if break_decrement {
            builder.append_block_param(loop_block, types::I64);
        }
        builder.append_block_param(done_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        if break_decrement {
            builder.ins().jump(
                loop_block,
                &[
                    zero_value.into(),
                    initial.tag.into(),
                    initial.payload.into(),
                    zero_value.into(),
                ],
            );
        } else {
            builder.ins().jump(
                loop_block,
                &[
                    zero_value.into(),
                    initial.tag.into(),
                    initial.payload.into(),
                ],
            );
        }

        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction = NativeScalarPairV1 {
            tag: builder.block_params(loop_block)[1],
            payload: builder.block_params(loop_block)[2],
        };
        if break_decrement {
            let fuel = builder.block_params(loop_block)[3];
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
                &[zero_value.into(), nontermination.into()],
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
            &[induction.tag.into(), induction.payload.into()],
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
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
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
            let fuel = builder.block_params(loop_block)[3];
            let next_fuel = builder.ins().iadd_imm(fuel, 1);
            builder.ins().jump(
                loop_block,
                &[
                    successor_value.into(),
                    next.tag.into(),
                    next.payload.into(),
                    next_fuel.into(),
                ],
            );
        } else {
            builder.ins().jump(
                loop_block,
                &[successor_value.into(), next.tag.into(), next.payload.into()],
            );
        }

        builder.switch_to_block(done_block);
        Ok(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        ))
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
                let ih_slots =
                    self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                let producer_origin = self.mint_recursor_producer_origin();
                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::DeferredConstructor,
                    origin: producer_origin,
                    cursor: deferred.selected_active.cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: deferred
                        .selected_active
                        .selected_scope
                        .map(|scope| scope.scope_origin),
                });
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let induction_hypothesis = self.make_computational_recursor(
                        constructor_args[position].clone(),
                        frame.cases.to_vec(),
                        frame.default.clone(),
                        outer_tail.clone(),
                        frame.provenance,
                        frame.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        deferred.selected_active.activation,
                        deferred.selected_active.cursor,
                        deferred.splice_caller,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(
                        Px8jProducerPath::DeferredConstructor,
                        &induction_hypothesis,
                    );
                    induction_hypotheses.push(induction_hypothesis);
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
    ) -> Result<(NativeScalarPairV1, bool), CraneliftBackendError> {
        let checked_root_exit_representation = self.has_checked_root_exit_representation();
        let lowered = if checked_root_exit_representation {
            Self::unwrap_terminal_ret(lowered)
        } else {
            lowered
        };
        let zero_tag = builder.ins().iconst(types::I64, 0);
        match lowered {
            Lowered::Int { value, known } => Ok((
                NativeScalarPairV1 {
                    tag: self.native_int_tag(builder, value, known)?,
                    payload: value,
                },
                false,
            )),
            Lowered::ProcessExitStatus { value } => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: value,
                },
                true,
            )),
            lowered if checked_root_exit_representation => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: self.emit_process_exit_status(builder, lowered),
                },
                true,
            )),
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
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        let checked_root_exit_representation = self.has_checked_root_exit_representation();
        let lowered = if checked_root_exit_representation {
            Self::unwrap_terminal_ret(lowered)
        } else {
            lowered
        };
        let zero_tag = builder.ins().iconst(types::I64, 0);
        match lowered {
            Lowered::RecursiveBackedge => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: builder.ins().iconst(types::I64, 0),
                },
                ScalarMergeKind::RecursiveBackedge,
            )),
            Lowered::Int { value, known } => Ok((
                NativeScalarPairV1 {
                    tag: self.native_int_tag(builder, value, known)?,
                    payload: value,
                },
                ScalarMergeKind::Int,
            )),
            Lowered::Bool { value, .. } => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: value,
                },
                ScalarMergeKind::Bool,
            )),
            Lowered::StructuralNat(nat) => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: nat.value,
                },
                ScalarMergeKind::StructuralNat,
            )),
            Lowered::Constructor { constructor, args }
                if args.is_empty()
                    && (constructor == self.process_symbols.bool_true
                        || constructor == self.process_symbols.bool_false) =>
            {
                Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: builder.ins().iconst(
                            types::I64,
                            i64::from(constructor == self.process_symbols.bool_true),
                        ),
                    },
                    ScalarMergeKind::Bool,
                ))
            }
            Lowered::ProcessExitStatus { value } => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: value,
                },
                ScalarMergeKind::ExitCode,
            )),
            lowered if checked_root_exit_representation => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: self.emit_process_exit_status(builder, lowered),
                },
                ScalarMergeKind::ExitCode,
            )),
            _ => Err(unsupported(
                construct,
                "dynamic arms must produce scalar Int or Bool values",
            )),
        }
    }

    fn restore_root_terminal_authority(
        &mut self,
        authority: Option<RootTerminalAnswerAuthority>,
        expected_outer: ContinuationCursorId,
    ) -> Result<(), CraneliftBackendError> {
        let Some(mut authority) = authority else {
            return Ok(());
        };
        if authority.outer_cursor != Some(expected_outer) {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "checked root answer authority returned through the wrong outer cursor",
            ));
        }
        // The exact source-machine delimiter consumes this cursor binding.
        // A later source-machine episode may bind the same affine root token
        // to its own exact outer cursor; retaining the old cursor would turn a
        // lawful sequential episode into an apparent transplant.
        authority.outer_cursor = None;
        if self.root_terminal_authority.replace(authority).is_some() {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "checked root answer authority was duplicated across source control",
            ));
        }
        Ok(())
    }

    /// The checked root cut determines the temporary scalar representation
    /// used at internal CFG joins. This is validation metadata only: it cannot
    /// mint or consume terminal authority, which remains affine in
    /// `RootTerminalAnswerAuthority` until `emit_result`.
    fn has_checked_root_exit_representation(&self) -> bool {
        self.process_object
            && self.native_join_plan.as_ref().is_some_and(|plan| {
                plan.sites.iter().any(|site| {
                    site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                        && site.checked_occurrence_path == [0]
                        && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
                        && self.consumed_join_sites.contains(&site.site_id)
                })
            })
    }

    fn mint_terminal_answer_authority(
        &mut self,
    ) -> Result<TerminalAnswerAuthority, CraneliftBackendError> {
        debug_assert_eq!(
            self.live_source_continuations == 0,
            self.source_control_root.is_none(),
            "source-control ownership and diagnostic depth must agree"
        );
        let authority = self.root_terminal_authority.take().ok_or_else(|| {
            unsupported(
                "NativeJoinPlanV1",
                "terminal answer has no affine checked-root authority",
            )
        })?;
        let site = self
            .native_join_plan
            .as_ref()
            .and_then(|plan| {
                plan.sites
                    .iter()
                    .find(|site| site.site_id == authority.site_id)
            })
            .ok_or_else(|| {
                unsupported(
                    "NativeJoinPlanV1",
                    "terminal answer authority names a missing checked-root site",
                )
            })?;
        if !self.process_object
            || site.runtime_frame_fingerprint != crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
            || site.checked_occurrence_path != [0]
            || site.answer_kind != crate::NativeJoinAnswerKindV1::ExitCode
            || site.checked_result_type_fingerprint != authority.checked_result_type_fingerprint
            || site.occurrence_binding_fingerprint != authority.occurrence_binding_fingerprint
            || !self.consumed_join_sites.contains(&authority.site_id)
            || authority.outer_cursor.is_some()
            || self.source_control_root.is_some()
            || self.active_oriented_semantic_regions != 0
            || self.active_subcontinuation_frame.is_some()
            || self.active_join_site.is_some()
        {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "terminal answer authority does not match the exhausted checked root",
            ));
        }
        Ok(TerminalAnswerAuthority)
    }

    fn unwrap_terminal_ret(mut lowered: Lowered) -> Lowered {
        loop {
            match lowered {
                Lowered::Constructor {
                    constructor,
                    mut args,
                } if constructor.ends_with("::ITree::Ret") && args.len() == 1 => {
                    lowered = args.remove(0);
                }
                lowered => return lowered,
            }
        }
    }

    /// Scalarize only under the answer kind carried by an already-consumed
    /// checked join site. In particular, process-object mode is not evidence
    /// that an arbitrary constructor is terminal: only an `ExitCode` plan may
    /// invoke the terminal process decoder.
    fn merge_planned_scalar_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: Lowered,
        required_kind: ScalarMergeKind,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        if required_kind == ScalarMergeKind::ExitCode {
            let lowered = Self::unwrap_terminal_ret(lowered);
            let zero_tag = builder.ins().iconst(types::I64, 0);
            return match lowered {
                Lowered::RecursiveBackedge => Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: builder.ins().iconst(types::I64, 0),
                    },
                    ScalarMergeKind::RecursiveBackedge,
                )),
                Lowered::ProcessExitStatus { value } => Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: value,
                    },
                    ScalarMergeKind::ExitCode,
                )),
                lowered if self.process_object => Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: self.emit_process_exit_status(builder, lowered),
                    },
                    ScalarMergeKind::ExitCode,
                )),
                _ => Err(unsupported(
                    construct,
                    "checked ExitCode join is unavailable outside process-object lowering",
                )),
            };
        }
        self.merge_scalar_branch(builder, lowered, construct)
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

    fn lowered_from_scalar_pair(
        &mut self,
        kind: ScalarMergeKind,
        pair: NativeScalarPairV1,
    ) -> Lowered {
        match kind {
            ScalarMergeKind::Int => {
                self.native_int_tags.insert(pair.payload, pair.tag);
                Lowered::Int {
                    value: pair.payload,
                    known: None,
                }
            }
            ScalarMergeKind::Bool => Lowered::Bool {
                value: pair.payload,
                known: None,
            },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 {
                value: pair.payload,
            }),
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus {
                value: pair.payload,
            },
            ScalarMergeKind::RecursiveBackedge => {
                unreachable!("backedges do not establish a merge result kind")
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
                crate::compiler_private_ordinary_match_frame_fingerprint(frame.cases, frame.default)
            }
            EliminatorFrame::InvocationReturn => crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
            EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => return Ok(None),
        };
        let Some(plan) = &self.native_join_plan else {
            return Ok(None);
        };
        if matches!(frame, EliminatorFrame::InvocationReturn) && self.active_join_site.is_some() {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "distinguished root cannot consume an active match occurrence marker",
            ));
        }
        let matches = match frame {
            EliminatorFrame::InvocationReturn => plan
                .sites
                .iter()
                .filter(|site| {
                    site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                        && site.checked_occurrence_path == [0]
                        && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
                })
                .cloned()
                .collect::<Vec<_>>(),
            EliminatorFrame::Computational(_) | EliminatorFrame::Ordinary(_) => {
                let Some(site_id) = self.active_join_site else {
                    return Ok(None);
                };
                plan.sites
                    .iter()
                    .filter(|site| site.site_id == site_id)
                    .cloned()
                    .collect::<Vec<_>>()
            }
            EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => unreachable!(),
        };
        match matches.as_slice() {
            [] if self.active_join_site.is_some() => Err(unsupported(
                "NativeJoinPlanV1",
                "runtime occurrence has no exact checked join site",
            )),
            [] => Ok(None),
            [site] => {
                if site.runtime_frame_fingerprint != fingerprint
                    || site.occurrence_binding_fingerprint
                        != crate::compiler_private_join_occurrence_binding_fingerprint(
                            site.site_id,
                            &site.declaration,
                            &site.checked_occurrence_path,
                            site.checked_result_type_fingerprint,
                        )
                {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence binding is stale or inconsistent",
                    ));
                }
                if !self.consumed_join_sites.insert(site.site_id)
                    && !matches!(frame, EliminatorFrame::InvocationReturn)
                {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence was consumed twice",
                    ));
                }
                if !matches!(frame, EliminatorFrame::InvocationReturn) {
                    self.active_join_site = None;
                }
                Ok(Some(site.clone()))
            }
            _ => Err(unsupported(
                "NativeJoinPlanV1",
                "checked cut identity resolves to multiple plan sites",
            )),
        }
    }

    fn require_complete_join_plan_consumption(&self) -> Result<(), CraneliftBackendError> {
        let Some(plan) = &self.native_join_plan else {
            return Ok(());
        };
        let planned = plan
            .sites
            .iter()
            .map(|site| site.site_id)
            .collect::<BTreeSet<_>>();
        if planned != self.consumed_join_sites {
            return Err(unsupported(
                "NativeJoinPlanV1",
                format!(
                    "checked join plan contains an unconsumed or orphan site: planned {planned:?}, consumed {:?}",
                    self.consumed_join_sites
                ),
            ));
        }
        Ok(())
    }

    fn require_complete_dynamic_splice_edge_consumption(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        if self.dynamic_splice_edges.is_empty() {
            return Ok(());
        }
        Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked lowering left affine dynamic splice edges unconsumed: {:?}",
                self.dynamic_splice_edges.keys().collect::<Vec<_>>(),
            ),
        ))
    }

    fn take_distinguished_root_answer_authority(
        &mut self,
    ) -> Result<Option<RootTerminalAnswerAuthority>, CraneliftBackendError> {
        let Some(plan) = &self.native_join_plan else {
            return if self.process_object {
                Err(unsupported(
                    "NativeJoinPlanV1",
                    "process-object lowering has no checked distinguished-root answer authority",
                ))
            } else {
                Ok(None)
            };
        };
        let roots = plan
            .sites
            .iter()
            .filter(|site| {
                site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                    && site.checked_occurrence_path == [0]
                    && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
            })
            .cloned()
            .collect::<Vec<_>>();
        let site = match roots.as_slice() {
            [] if !self.process_object => return Ok(None),
            [] => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "process-object lowering has no checked distinguished-root answer authority",
                ));
            }
            [site] => site,
            _ => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "checked package contains multiple distinguished root join sites",
                ));
            }
        };
        if site.occurrence_binding_fingerprint
            != crate::compiler_private_join_occurrence_binding_fingerprint(
                site.site_id,
                &site.declaration,
                &site.checked_occurrence_path,
                site.checked_result_type_fingerprint,
            )
        {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "distinguished root join occurrence binding is stale or inconsistent",
            ));
        }
        if !self.consumed_join_sites.insert(site.site_id) {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "checked distinguished-root answer authority was consumed more than once",
            ));
        }
        Ok(Some(RootTerminalAnswerAuthority {
            site_id: site.site_id,
            checked_result_type_fingerprint: site.checked_result_type_fingerprint,
            occurrence_binding_fingerprint: site.occurrence_binding_fingerprint,
            outer_cursor: None,
        }))
    }

    fn scalar_kind_from_plan(kind: crate::NativeJoinAnswerKindV1) -> ScalarMergeKind {
        match kind {
            crate::NativeJoinAnswerKindV1::Int => ScalarMergeKind::Int,
            crate::NativeJoinAnswerKindV1::Bool => ScalarMergeKind::Bool,
            crate::NativeJoinAnswerKindV1::StructuralNat => ScalarMergeKind::StructuralNat,
            crate::NativeJoinAnswerKindV1::ExitCode => ScalarMergeKind::ExitCode,
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

    fn source_terminal_join<'b, 'c>(
        continuation: &'b SourceContinuation<'c>,
    ) -> Option<&'b SourceJoinTarget<'c>> {
        match continuation {
            SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)) => {
                Some(&edge.target)
            }
            SourceContinuation::Terminal(
                SourceContinuationTerminal::ReturnValue
                | SourceContinuationTerminal::ReturnToProducerHole { .. }
                | SourceContinuationTerminal::ResumeOuter { .. },
            ) => None,
            SourceContinuation::LetBody { next, .. }
            | SourceContinuation::CheckedRecursiveInvocationReturn { next, .. }
            | SourceContinuation::CheckedComputationalIHInvocationReturn { next, .. }
            | SourceContinuation::ReturnFromSelectedCase { next, .. }
            | SourceContinuation::ApplyRecursorSelection { next, .. }
            | SourceContinuation::UnwindRecursorSegment { next, .. }
            | SourceContinuation::IfScrutinee { next, .. }
            | SourceContinuation::ConstructArgument { next, .. }
            | SourceContinuation::MatchScrutinee { next, .. }
            | SourceContinuation::ComputationalMatchScrutinee { next, .. }
            | SourceContinuation::ProjectRecord { next, .. }
            | SourceContinuation::CallCallee { next, .. }
            | SourceContinuation::CallArgument { next, .. } => Self::source_terminal_join(next),
        }
    }

    fn discard_source_prefix<'b>(continuation: SourceContinuation<'b>) -> SourceContinuation<'b> {
        match continuation {
            terminal @ SourceContinuation::Terminal(_) => terminal,
            SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                SourceContinuation::CheckedRecursiveInvocationReturn {
                    instance,
                    next: Box::new(Self::discard_source_prefix(*next)),
                }
            }
            SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next: Box::new(Self::discard_source_prefix(*next)),
            },
            SourceContinuation::ReturnFromSelectedCase { next, .. } => {
                Self::discard_source_prefix(*next)
            }
            SourceContinuation::LetBody { next, .. }
            | SourceContinuation::ApplyRecursorSelection { next, .. }
            | SourceContinuation::UnwindRecursorSegment { next, .. }
            | SourceContinuation::IfScrutinee { next, .. }
            | SourceContinuation::ConstructArgument { next, .. }
            | SourceContinuation::MatchScrutinee { next, .. }
            | SourceContinuation::ComputationalMatchScrutinee { next, .. }
            | SourceContinuation::ProjectRecord { next, .. }
            | SourceContinuation::CallCallee { next, .. }
            | SourceContinuation::CallArgument { next, .. } => Self::discard_source_prefix(*next),
        }
    }

    fn replace_source_terminal_with_unwind<'b>(
        continuation: SourceContinuation<'b>,
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        Ok(match continuation {
            SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                SourceContinuation::CheckedRecursiveInvocationReturn {
                    instance,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                SourceContinuation::ReturnFromSelectedCase {
                    delimiter,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::LetBody { body, env, next } => SourceContinuation::LetBody {
                body,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ApplyRecursorSelection { layer, next } => {
                SourceContinuation::ApplyRecursorSelection {
                    layer,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::UnwindRecursorSegment {
                stack: outer_stack,
                resume_cursor: outer_cursor,
                next,
            } => SourceContinuation::UnwindRecursorSegment {
                stack: outer_stack,
                resume_cursor: outer_cursor,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ConstructArgument {
                constructor,
                remaining: arguments,
                lowered,
                env,
                next,
            } => SourceContinuation::ConstructArgument {
                constructor,
                remaining: arguments,
                lowered,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ProjectRecord { field, next } => {
                SourceContinuation::ProjectRecord {
                    field,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::CallCallee { args, env, next } => SourceContinuation::CallCallee {
                args,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::CallArgument {
                callee,
                remaining: arguments,
                lowered,
                env,
                next,
            } => SourceContinuation::CallArgument {
                callee,
                remaining: arguments,
                lowered,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected,
                active,
                root_authority,
            }) => SourceContinuation::Terminal(SourceContinuationTerminal::ReturnToProducerHole {
                stack,
                resume_cursor,
                expected,
                active,
                root_authority,
            }),
            terminal @ SourceContinuation::Terminal(_) => terminal,
        })
    }

    fn install_recursor_invocation<'b>(
        &mut self,
        continuation: SourceContinuation<'b>,
        activation: ContinuationActivationId,
        invocation: RecursorInvocationSegment,
        checked_ih_invocation: Option<CheckedRecursiveInvocationInstance>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        if !recursor_invocation_is_checked(&invocation) {
            validate_recursor_invocation_install_shape(&invocation)?;
        }
        #[cfg(test)]
        px8j_record_source_event(Px8jSourceTraceEvent::Install {
            origin: invocation.origin,
            selection_cursor: invocation.resume_cursor,
            sibling_position: invocation.sibling_position,
            exits: invocation
                .unwind
                .later_wrappers_in_construction_order
                .iter()
                .filter_map(|layer| match layer.role {
                    RecursorLayerRole::ExitsScope {
                        scope_origin,
                        parent_scope,
                        ..
                    } => Some((scope_origin, parent_scope)),
                    RecursorLayerRole::SelectsOccurrence { .. } => None,
                })
                .collect(),
        });
        let sibling_position = invocation.sibling_position;
        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
        let installed = compose_oriented_subcontinuation(
            self.oriented_subcontinuation_plan.as_ref(),
            checked_ih_invocation.or_else(|| self.active_recursive_invocations.last().copied()),
            activation,
            invocation,
            dynamic_splice_edges,
        )?;
        debug_assert_eq!(installed.activation, activation);
        debug_assert!(installed
            .control_ledger
            .iter()
            .all(|entry| match entry.role {
                RecursorLayerRole::SelectsOccurrence { origin }
                | RecursorLayerRole::ExitsScope { origin, .. } => {
                    origin == installed.producer_origin
                }
            }));
        debug_assert_eq!(installed.sibling_position, sibling_position);
        debug_assert!(installed.control_ledger.len() >= installed.semantic_frames.len());
        debug_assert!(installed.control_ledger.iter().all(|entry| {
            entry.frame_id.is_some() == entry.checked_witness.is_some()
                && (entry.frame_id.is_none()
                    || matches!(
                        entry.role,
                        RecursorLayerRole::SelectsOccurrence { .. }
                            | RecursorLayerRole::ExitsScope { .. }
                    ))
        }));
        if !installed.checked {
            let mut frames = installed.semantic_frames.into_iter();
            let selection = frames
                .next()
                .expect("validated recursor invocation has a selection frame");
            let stack = RecursorUnwindStack {
                later_wrappers_in_construction_order: frames.rev().collect(),
            };
            let continuation = Self::replace_source_terminal_with_unwind(
                continuation,
                stack,
                installed.resume_cursor,
            )?;
            return Ok(SourceContinuation::ApplyRecursorSelection {
                layer: selection,
                next: Box::new(continuation),
            });
        }
        let mut continuation = continuation;
        for layer in installed.semantic_frames.into_iter().rev() {
            continuation = SourceContinuation::ApplyRecursorSelection {
                layer,
                next: Box::new(continuation),
            };
        }
        Ok(continuation)
    }

    fn split_source_prefix<'b>(
        source: SourceContinuation<'b>,
    ) -> Result<(SourcePrefixTemplate, SourcePrefixTerminal<'b>), CraneliftBackendError> {
        Ok(match source {
            SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CheckedRecursiveInvocationReturn {
                        instance,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
                        call_template_id,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ReturnFromSelectedCase {
                        delimiter,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "source prefix has no exact outer terminal to split",
                ));
            }
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected,
                root_authority,
                ..
            }) => (
                SourcePrefixTemplate::Terminal {
                    expected_outer: expected,
                },
                SourcePrefixTerminal::ResumeOuter { root_authority },
            ),
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnToProducerHole {
                expected,
                root_authority,
                ..
            }) => (
                SourcePrefixTemplate::Terminal {
                    expected_outer: expected,
                },
                SourcePrefixTerminal::ResumeOuter { root_authority },
            ),
            SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)) => (
                SourcePrefixTemplate::Terminal {
                    expected_outer: edge.target.expected_outer,
                },
                SourcePrefixTerminal::Join(edge),
            ),
            SourceContinuation::LetBody { body, env, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::LetBody {
                        body,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ApplyRecursorSelection { layer, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ApplyRecursorSelection {
                        layer,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::UnwindRecursorSegment {
                stack,
                resume_cursor,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::UnwindRecursorSegment {
                        stack,
                        resume_cursor,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::IfScrutinee {
                        then_expr,
                        else_expr,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ConstructArgument {
                constructor,
                remaining,
                lowered,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ConstructArgument {
                        constructor,
                        remaining,
                        lowered,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::MatchScrutinee {
                        cases,
                        default,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ComputationalMatchScrutinee {
                        cases,
                        default,
                        env,
                        provenance,
                        checked_frame_id,
                        answer_route,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ProjectRecord { field, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ProjectRecord {
                        field,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::CallCallee { args, env, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CallCallee {
                        args,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::CallArgument {
                callee,
                remaining,
                lowered,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CallArgument {
                        callee,
                        remaining,
                        lowered,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
        })
    }

    fn instantiate_source_prefix_template<'b>(
        template: &SourcePrefixTemplate,
        edge: SourcePredecessorEdge<'b>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        Ok(match template {
            SourcePrefixTemplate::Terminal { expected_outer } => {
                if *expected_outer != edge.target.expected_outer {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "source prefix terminal does not match the planned outer cursor",
                    ));
                }
                SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge))
            }
            SourcePrefixTemplate::CheckedRecursiveInvocationReturn { instance, next } => {
                SourceContinuation::CheckedRecursiveInvocationReturn {
                    instance: *instance,
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id: *call_template_id,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ReturnFromSelectedCase { delimiter, next } => {
                SourceContinuation::ReturnFromSelectedCase {
                    delimiter: *delimiter,
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::LetBody { body, env, next } => SourceContinuation::LetBody {
                body: body.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ApplyRecursorSelection { layer, next } => {
                SourceContinuation::ApplyRecursorSelection {
                    layer: layer.clone(),
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::UnwindRecursorSegment {
                stack,
                resume_cursor,
                next,
            } => SourceContinuation::UnwindRecursorSegment {
                stack: stack.clone(),
                resume_cursor: *resume_cursor,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => SourceContinuation::IfScrutinee {
                then_expr: then_expr.clone(),
                else_expr: else_expr.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ConstructArgument {
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
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                provenance: *provenance,
                checked_frame_id: *checked_frame_id,
                answer_route: *answer_route,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ProjectRecord { field, next } => {
                SourceContinuation::ProjectRecord {
                    field: field.clone(),
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::CallCallee { args, env, next } => {
                SourceContinuation::CallCallee {
                    args: args.clone(),
                    env: env.clone(),
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::CallArgument {
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
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
        })
    }

    fn mint_source_predecessor<'b>(
        &mut self,
        target: SourceJoinTarget<'b>,
    ) -> SourcePredecessorEdge<'b> {
        let predecessor_identity = self.next_source_predecessor;
        self.next_source_predecessor = self
            .next_source_predecessor
            .checked_add(1)
            .expect("compiler-private source predecessor identity exhausted");
        SourcePredecessorEdge {
            target,
            predecessor_identity,
        }
    }

    fn seal_source_trap_branch(builder: &mut FunctionBuilder<'_>, lowered: &Lowered) -> bool {
        if matches!(lowered, Lowered::Trap(_)) {
            let failure = builder.ins().iconst(types::I64, -4);
            builder.ins().return_(&[failure]);
            true
        } else {
            false
        }
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
        let mut root_authority = self.root_terminal_authority.take();
        if let Some(authority) = &mut root_authority {
            match authority.outer_cursor {
                None => authority.outer_cursor = Some(active.cursor),
                Some(cursor) if cursor == active.cursor => {}
                Some(_) => {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked root answer authority was transplanted to another outer cursor",
                    ));
                }
            }
        }
        let control = SourceControl {
            continuation: SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected: active.cursor,
                active,
                root_authority,
            }),
            selected: SourceSelectedContinuation {
                activation: active.activation,
                cursor: active.cursor,
                parent: active.parent,
                pending: active.pending.to_vec(),
                selected_ancestry: active.selected_ancestry.to_vec(),
                selected_scope: active.selected_scope.cloned(),
            },
            selected_lineage: Vec::new(),
            terminal_outer: active.cursor,
        };
        self.lower_source_machine_with_continuation(builder, expr.clone(), env.to_vec(), control)
    }

    fn lower_source_machine_with_continuation<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let previous_source_root = self.source_control_root.replace(control.terminal_outer);
        self.live_source_continuations = self
            .live_source_continuations
            .checked_add(1)
            .expect("compiler-private live source-continuation depth exhausted");
        let result = self.lower_source_machine_with_continuation_inner(builder, expr, env, control);
        self.live_source_continuations = self
            .live_source_continuations
            .checked_sub(1)
            .expect("source-continuation depth must balance");
        self.source_control_root = previous_source_root;
        result
    }

    fn lower_source_machine_with_continuation_inner<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let mut state = SourceMachineState::Eval { expr, env, control };
        loop {
            state = match state {
                SourceMachineState::Eval {
                    expr,
                    env,
                    mut control,
                } => match expr {
                    RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                        self.enter_checked_subcontinuation_frame(frame_id)?;
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedRecursiveInvocation {
                        call_template_id,
                        body,
                        ..
                    } => {
                        let instance =
                            self.enter_checked_recursive_invocation(call_template_id, &body)?;
                        control.continuation =
                            SourceContinuation::CheckedRecursiveInvocationReturn {
                                instance,
                                next: Box::new(control.continuation),
                            };
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHInvocation {
                        call_template_id,
                        body,
                        ..
                    } => {
                        self.enter_checked_computational_ih_invocation(call_template_id)?;
                        control.continuation =
                            SourceContinuation::CheckedComputationalIHInvocationReturn {
                                call_template_id,
                                next: Box::new(control.continuation),
                            };
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Value(value) => SourceMachineState::Value {
                        value: self.lower_value(builder, &value)?,
                        control,
                    },
                    RuntimeExpr::Var(index) => SourceMachineState::Value {
                        value: env.get(index as usize).cloned().ok_or_else(|| {
                            unsupported("Var", format!("no runtime binding for index {index}"))
                        })?,
                        control,
                    },
                    RuntimeExpr::Let { value, body } => {
                        control.continuation = SourceContinuation::LetBody {
                            body: *body,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: *value,
                            env: env.clone(),
                            control,
                        }
                    }
                    RuntimeExpr::Construct {
                        constructor,
                        mut args,
                    } => {
                        if args.is_empty() {
                            SourceMachineState::Value {
                                value: self.finish_source_constructor(
                                    builder,
                                    constructor,
                                    vec![],
                                )?,
                                control,
                            }
                        } else {
                            let first = args.remove(0);
                            control.continuation = SourceContinuation::ConstructArgument {
                                constructor,
                                remaining: args,
                                lowered: Vec::new(),
                                env: env.clone(),
                                next: Box::new(control.continuation),
                            };
                            SourceMachineState::Eval {
                                expr: first,
                                env,
                                control,
                            }
                        }
                    }
                    RuntimeExpr::Match {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        control.continuation = SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: *scrutinee,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Call { callee, args } => {
                        control.continuation = SourceContinuation::CallCallee {
                            args,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: *callee,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::ComputationalMatch {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        let checked_frame_id =
                            self.consume_checked_subcontinuation_frame(&cases, &default)?;
                        control.continuation = SourceContinuation::ComputationalMatchScrutinee {
                            cases,
                            default,
                            env: env.clone(),
                            provenance: self.mint_recursor_frame_provenance(),
                            checked_frame_id,
                            answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: *scrutinee,
                            env,
                            control,
                        }
                    }
                    other => SourceMachineState::Value {
                        value: self.lower_expr(builder, &other, &env)?,
                        control,
                    },
                },
                SourceMachineState::Value { value, mut control } => {
                    if matches!(value, Lowered::Trap(_)) {
                        control.continuation = Self::discard_source_prefix(control.continuation);
                    }
                    match control.continuation {
                        SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                            return Ok(value);
                        }
                        SourceContinuation::Terminal(
                            SourceContinuationTerminal::ReturnToProducerHole {
                                stack,
                                resume_cursor,
                                expected,
                                active,
                                root_authority,
                            },
                        ) => {
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::ReturnHole {
                                cursor: resume_cursor,
                            });
                            if active.cursor != expected {
                                return Err(unsupported(
                                    "ComputationalRecursor",
                                    "producer-hole terminal cursor mismatch",
                                ));
                            }
                            if matches!(value, Lowered::Trap(_)) {
                                return Ok(value);
                            }
                            source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "producer-hole resume cursor is no longer active",
                                )
                            })?;
                            control.continuation = SourceContinuation::UnwindRecursorSegment {
                                stack,
                                resume_cursor,
                                next: Box::new(SourceContinuation::Terminal(
                                    SourceContinuationTerminal::ResumeOuter {
                                        expected,
                                        active,
                                        root_authority,
                                    },
                                )),
                            };
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                            expected,
                            active,
                            root_authority,
                        }) => {
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::ResumeOuter {
                                cursor: expected,
                            });
                            if active.cursor != expected {
                                return Err(unsupported(
                                    "ComputationalRecursor",
                                    "source continuation terminal cursor mismatch",
                                ));
                            }
                            self.restore_root_terminal_authority(root_authority, expected)?;
                            if matches!(value, Lowered::Trap(_)) {
                                return Ok(value);
                            }
                            return self.resume_active_continuation(builder, value, *active);
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(
                            edge,
                        )) => {
                            if matches!(value, Lowered::Trap(_)) {
                                let failure = builder.ins().iconst(types::I64, -4);
                                builder.ins().return_(&[failure]);
                                return Ok(Lowered::RecursiveBackedge);
                            }
                            let value = if edge.target.terminal_active_prefix.is_empty() {
                                value
                            } else {
                                let mut prefix = edge.target.terminal_active_prefix;
                                prefix.push(EliminatorFrame::InvocationReturn);
                                self.lower_computational_match_value_composed(
                                    builder, value, &prefix,
                                )?
                            };
                            let (value, actual_kind) = self.merge_planned_scalar_branch(
                                builder,
                                value,
                                edge.target.required_kind,
                                "NativeJoinPlanV1",
                            )?;
                            if actual_kind != ScalarMergeKind::RecursiveBackedge
                                && actual_kind != edge.target.required_kind
                            {
                                return Err(unsupported(
                                "NativeJoinPlanV1",
                                format!(
                                    "predecessor {} for join {} produced {actual_kind:?}, planned {:?}",
                                    edge.predecessor_identity,
                                    edge.target.join_id,
                                    edge.target.required_kind
                                ),
                            ));
                            }
                            builder
                                .ins()
                                .jump(edge.target.block, &[value.tag.into(), value.payload.into()]);
                            return Ok(Lowered::RecursiveBackedge);
                        }
                        SourceContinuation::LetBody { body, env, next } => {
                            control.continuation = *next;
                            if matches!(value, Lowered::RecursiveBackedge) {
                                SourceMachineState::Value { value, control }
                            } else if matches!(value, Lowered::Trap(_)) {
                                SourceMachineState::Value { value, control }
                            } else {
                                let mut body_env = vec![value];
                                body_env.extend(env);
                                SourceMachineState::Eval {
                                    expr: body,
                                    env: body_env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                            self.leave_checked_recursive_invocation(instance)?;
                            control.continuation = *next;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::CheckedComputationalIHInvocationReturn {
                            call_template_id,
                            next,
                        } => {
                            if self
                                .pending_computational_ih_call
                                .is_some_and(|pending| pending != call_template_id)
                            {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "computational IH invocation return crossed another marker",
                                ));
                            }
                            let value = self.finish_checked_computational_ih_marker(value)?;
                            control.continuation = *next;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                            let scope =
                                control.selected.selected_scope.as_ref().ok_or_else(|| {
                                    unsupported(
                                        "OrientedSubcontinuationPlanV1",
                                        "selected-case return has no open control obligation",
                                    )
                                })?;
                            if control.selected.activation != delimiter.activation
                                || control.selected.cursor != delimiter.cursor
                                || scope.scope_origin != delimiter.scope_origin
                                || scope.frame.checked_frame_id != delimiter.frame_id
                                || scope.frame.checked_invocation_id != delimiter.invocation_id
                            {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return delimiter does not match its open occurrence",
                                ));
                            }
                            let previous = control.selected_lineage.pop().ok_or_else(|| {
                                unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return has no exact parent control state",
                                )
                            })?;
                            control.selected = previous;
                            control.continuation = *next;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::ApplyRecursorSelection { layer, next } => {
                            #[cfg(test)]
                            match layer.role {
                                RecursorLayerRole::SelectsOccurrence { origin } => {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Selection {
                                        origin,
                                    });
                                }
                                RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                } => px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                }),
                            }
                            let answer_route =
                                SourceComputationalAnswerRoute::for_recursor_layer(&layer);
                            control.continuation =
                                SourceContinuation::ComputationalMatchScrutinee {
                                    cases: layer.cases,
                                    default: layer.default,
                                    env: layer.outer_env,
                                    provenance: layer.provenance,
                                    checked_frame_id: layer.checked_frame_id,
                                    answer_route,
                                    next,
                                };
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::UnwindRecursorSegment {
                            mut stack,
                            resume_cursor,
                            next,
                        } => {
                            source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "source recursor resume cursor is no longer active",
                                )
                            })?;
                            if let Some(layer) = stack.later_wrappers_in_construction_order.pop() {
                                #[cfg(test)]
                                if let RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                } = layer.role
                                {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                                        origin,
                                        scope_origin,
                                        parent_scope,
                                    });
                                }
                                let answer_route =
                                    SourceComputationalAnswerRoute::for_recursor_layer(&layer);
                                control.continuation =
                                    SourceContinuation::ComputationalMatchScrutinee {
                                        cases: layer.cases,
                                        default: layer.default,
                                        env: layer.outer_env,
                                        provenance: layer.provenance,
                                        checked_frame_id: layer.checked_frame_id,
                                        answer_route,
                                        next: Box::new(SourceContinuation::UnwindRecursorSegment {
                                            stack,
                                            resume_cursor,
                                            next,
                                        }),
                                    };
                                SourceMachineState::Value { value, control }
                            } else {
                                control.continuation = *next;
                                SourceMachineState::Value { value, control }
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
                            control.continuation = *next;
                            if remaining.is_empty() {
                                SourceMachineState::Value {
                                    value: self.finish_source_constructor(
                                        builder,
                                        constructor,
                                        lowered,
                                    )?,
                                    control,
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::ConstructArgument {
                                    constructor,
                                    remaining,
                                    lowered,
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            match value {
                                Lowered::BoundedNat(nat) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder, nat, false, &cases, &default, &env, control,
                                    );
                                }
                                Lowered::StructuralNat(nat) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder,
                                        BoundedNatV1::derived_from_validated(nat.value),
                                        true,
                                        &cases,
                                        &default,
                                        &env,
                                        control,
                                    );
                                }
                                Lowered::Bool { value, known } => {
                                    let true_case = cases.iter().find(|case| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::True")
                                    });
                                    let false_case = cases.iter().find(|case| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::False")
                                    });
                                    let (Some(true_case), Some(false_case)) =
                                        (true_case, false_case)
                                    else {
                                        return Err(unsupported(
                                            "Match",
                                            "Bool match requires zero-binder True and False cases",
                                        ));
                                    };
                                    if let Some(selected) = known {
                                        SourceMachineState::Eval {
                                            expr: if selected {
                                                true_case.body.clone()
                                            } else {
                                                false_case.body.clone()
                                            },
                                            env,
                                            control,
                                        }
                                    } else {
                                        return self.lower_source_dynamic_bool_match(
                                            builder,
                                            value,
                                            &true_case.body,
                                            &false_case.body,
                                            &env,
                                            control,
                                        );
                                    }
                                }
                                Lowered::HostResult {
                                    success,
                                    error,
                                    ok,
                                    err_constructor,
                                    ok_constructor,
                                } => {
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
                                        control,
                                    );
                                }
                                Lowered::DynamicConstructor(dynamic) => {
                                    return self.lower_source_dynamic_constructor_match(
                                        builder, dynamic, &cases, &default, &env, control,
                                    );
                                }
                                Lowered::Constructor { constructor, args } => {
                                    let Some(case) =
                                        cases.iter().find(|case| case.constructor == constructor)
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
                                        control,
                                    }
                                }
                                _ => {
                                    return Err(unsupported(
                                        "Match",
                                        "scrutinee is not a constructor value",
                                    ));
                                }
                            }
                        }
                        SourceContinuation::ComputationalMatchScrutinee {
                            cases,
                            default,
                            env,
                            provenance,
                            checked_frame_id,
                            answer_route,
                            next,
                        } => {
                            let retained = value.clone();
                            #[cfg(test)]
                            let actual_constructor = match &value {
                                Lowered::Constructor { constructor, .. } => {
                                    Some(constructor.clone())
                                }
                                _ => None,
                            };
                            let selected = match &value {
                                Lowered::Constructor { constructor, .. } => {
                                    cases.iter().find(|case| case.constructor == *constructor)
                                }
                                _ => None,
                            };
                            let case = if let Some(case) = selected {
                                case
                            } else if answer_route
                                == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                && matches!(&value, Lowered::Constructor { .. })
                                && px8tr_deforested_answer_route_enabled()
                            {
                                let mut returns = cases.iter().filter(|case| {
                                    case.argument_binders == 1
                                        && case.constructor.ends_with("::ITree::Ret")
                                });
                                let return_case = returns.next();
                                let exact_return = returns.next().is_none();
                                let mut visible = cases
                                    .iter()
                                    .filter(|case| case.constructor.ends_with("::ITree::Vis"));
                                let exact_visible = visible.next().is_some()
                                    && visible.next().is_none()
                                    && cases.len() == 2;
                                let Some(return_case) = return_case.filter(|return_case| {
                                    exact_return
                                        && exact_visible
                                        && source_case_has_no_checked_control_markers(
                                            &return_case.body,
                                        )
                                }) else {
                                    #[cfg(test)]
                                    px8tr_record_trap_provenance(
                                        Px8trTrapProvenanceEvent::CheckedRecursorDefault {
                                            checked_frame_id: checked_frame_id.expect(
                                                "checked answer routes carry exact frame ids",
                                            ),
                                            actual_constructor,
                                            trap: default.clone(),
                                        },
                                    );
                                    return Ok(Lowered::Trap(default));
                                };
                                #[cfg(test)]
                                px8tr_record_trap_provenance(
                                    Px8trTrapProvenanceEvent::DeforestedAnswerResumed {
                                        checked_frame_id: checked_frame_id
                                            .expect("checked answer routes carry exact frame ids"),
                                        actual_constructor,
                                        return_constructor: return_case.constructor.clone(),
                                    },
                                );
                                let mut case_env = vec![retained];
                                case_env.extend(env);
                                control.continuation = *next;
                                return self.lower_source_machine_with_continuation(
                                    builder,
                                    return_case.body.clone(),
                                    case_env,
                                    control,
                                );
                            } else {
                                if !matches!(&value, Lowered::Constructor { .. }) {
                                    return Err(unsupported(
                                        "ComputationalMatch",
                                        "source scrutinee is not a constructor value",
                                    ));
                                }
                                #[cfg(test)]
                                if answer_route
                                    == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                {
                                    px8tr_record_trap_provenance(
                                        Px8trTrapProvenanceEvent::CheckedRecursorDefault {
                                            checked_frame_id: checked_frame_id.expect(
                                                "checked answer routes carry exact frame ids",
                                            ),
                                            actual_constructor,
                                            trap: default.clone(),
                                        },
                                    );
                                }
                                return Ok(Lowered::Trap(default));
                            };
                            let Lowered::Constructor { args, .. } = value else {
                                unreachable!("a selected source case has a constructor value")
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
                            let frame = ComputationalEliminatorFrame {
                                cases: &cases,
                                default: &default,
                                env: &env,
                                retained_scrutinee_index: None,
                                deferred_constructor_case: None,
                                provenance,
                                checked_frame_id,
                                checked_invocation_id: checked_frame_id.map(|_| {
                                    self.active_recursive_invocations
                                        .last()
                                        .map_or(0, |instance| instance.invocation_instance_id)
                                }),
                                checked_invocation_source: self
                                    .active_recursive_invocations
                                    .last()
                                    .map(|instance| instance.source),
                                checked_invocation_depth: self
                                    .active_recursive_invocations
                                    .last()
                                    .map_or(0, |instance| instance.semantic_depth),
                            };
                            let activation = self.mint_continuation_activation();
                            let cursor = self.mint_continuation_cursor();
                            let mut ancestry = control.selected.selected_ancestry.clone();
                            ancestry.push(provenance);
                            let mut induction_hypotheses =
                                Vec::with_capacity(case.recursive_positions.len());
                            let ih_slots =
                                self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                            let producer_origin = self.mint_recursor_producer_origin();
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                                path: Px8jProducerPath::SourceMachine,
                                origin: producer_origin,
                                cursor,
                                siblings: case.recursive_positions.len(),
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                            });
                            let parent = control.selected.parent;
                            {
                                let qold = control.selected.as_active(&control.selected_lineage);
                                for position in case.recursive_positions.iter().rev().copied() {
                                    let slot_template_id = case
                                        .recursive_positions
                                        .iter()
                                        .position(|candidate| *candidate == position)
                                        .and_then(|index| ih_slots[index]);
                                    let induction_hypothesis = self.make_computational_recursor(
                                        args[position].clone(),
                                        cases.clone(),
                                        default.clone(),
                                        env.clone(),
                                        provenance,
                                        frame.checked_frame_id,
                                        slot_template_id,
                                        producer_origin,
                                        position,
                                        RecursorLayerRole::SelectsOccurrence {
                                            origin: producer_origin,
                                        },
                                        activation,
                                        cursor,
                                        Some(&qold),
                                        Some((
                                            &control.selected,
                                            control.selected_lineage.as_slice(),
                                        )),
                                    )?;
                                    #[cfg(test)]
                                    px8j_record_recursor_carrier(
                                        Px8jProducerPath::SourceMachine,
                                        &induction_hypothesis,
                                    );
                                    induction_hypotheses.push(induction_hypothesis);
                                }
                            }
                            let frame_env = match self.materialize_eliminator_frame_env(
                                builder,
                                EliminatorFrame::Computational(frame),
                                &retained,
                            )? {
                                Ok(frame_env) => frame_env,
                                Err(trap) => return Ok(Lowered::Trap(trap)),
                            };
                            let mut case_env = induction_hypotheses;
                            case_env.extend(args);
                            case_env.extend(frame_env);
                            let previous_selected = control.selected.clone();
                            let pending = std::mem::take(&mut control.selected.pending);
                            let selected_scope = OwnedSelectedScope {
                                scope_origin: producer_origin,
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                                frame: ComputationalRecursorFramePayload {
                                    cases: cases.clone(),
                                    default: default.clone(),
                                    outer_env: env.clone(),
                                    provenance,
                                    checked_frame_id: frame.checked_frame_id,
                                    checked_invocation_id: frame.checked_invocation_id,
                                    checked_invocation_source: frame.checked_invocation_source,
                                    checked_invocation_depth: frame.checked_invocation_depth,
                                },
                            };
                            #[cfg(test)]
                            let selected_scope =
                                (!PX8J_DELETE_OWNED_SELECTED_SCOPE.get()).then_some(selected_scope);
                            #[cfg(not(test))]
                            let selected_scope = Some(selected_scope);
                            control.continuation = if frame.checked_frame_id.is_some() {
                                let selected_scope_ref =
                                    selected_scope.as_ref().ok_or_else(|| {
                                        unsupported(
                                            "OrientedSubcontinuationPlanV1",
                                            "checked selection has no owned open-control obligation",
                                        )
                                    })?;
                                SourceContinuation::ReturnFromSelectedCase {
                                    delimiter: SelectedCaseReturnDelimiter {
                                        activation,
                                        cursor,
                                        scope_origin: selected_scope_ref.scope_origin,
                                        frame_id: selected_scope_ref.frame.checked_frame_id,
                                        invocation_id: selected_scope_ref
                                            .frame
                                            .checked_invocation_id,
                                    },
                                    next,
                                }
                            } else {
                                *next
                            };
                            control.selected = SourceSelectedContinuation {
                                activation,
                                cursor,
                                parent,
                                pending,
                                selected_ancestry: ancestry,
                                selected_scope,
                            };
                            control.selected_lineage.push(previous_selected);
                            SourceMachineState::Eval {
                                expr: case.body.clone(),
                                env: case_env,
                                control,
                            }
                        }
                        SourceContinuation::CallCallee {
                            mut args,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            if args.is_empty() {
                                match self.source_call_state(
                                    builder,
                                    value,
                                    Vec::new(),
                                    env,
                                    control,
                                )? {
                                    SourceCallOutcome::Continue(state) => state,
                                    SourceCallOutcome::Complete(value) => return Ok(value),
                                }
                            } else {
                                let first = args.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee: value,
                                    remaining: args,
                                    lowered: Vec::new(),
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
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
                            control.continuation = *next;
                            if remaining.is_empty() {
                                match self
                                    .source_call_state(builder, callee, lowered, env, control)?
                                {
                                    SourceCallOutcome::Continue(state) => state,
                                    SourceCallOutcome::Complete(value) => return Ok(value),
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee,
                                    remaining,
                                    lowered,
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
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
                    }
                }
            };
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_bounded_nat_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
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
                "structural Nat source match requires exact Zero and Suc predecessor arms",
            ));
        };

        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let (source_prefix_template, target) = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => {
                let fanout = SourceBranchFanout {
                    source_prefix_template,
                    inherited_edge,
                };
                (fanout.source_prefix_template, fanout.inherited_edge.target)
            }
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                (
                    source_prefix_template,
                    SourceJoinTarget {
                        join_id,
                        block: merge,
                        expected_outer: suffix_control.terminal_outer,
                        required_kind,
                        terminal_active_prefix: prefix,
                    },
                )
            }
        };

        let zero_block = builder.create_block();
        let suc_block = builder.create_block();
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);

        for (arm_name, block, case, predecessor) in [
            ("Zero", zero_block, zero, None),
            ("Suc", suc_block, suc, Some(predecessor)),
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
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_source_machine_with_continuation(
                builder,
                case.body.clone(),
                arm_env,
                branch_control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                let detail = match &lowered {
                    Lowered::Trap(trap) => format!("Trap({}: {:?})", trap.message, trap.code),
                    other => lowered_value_kind(other).to_string(),
                };
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "bounded-Nat {arm_name} arm produced {detail} instead of sealing its distinct affine predecessor edge"
                    ),
                ));
            }
        }

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    fn lower_source_dynamic_bool_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        condition: cranelift_codegen::ir::Value,
        true_body: &RuntimeExpr,
        false_body: &RuntimeExpr,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let true_block = builder.create_block();
        let false_block = builder.create_block();
        builder
            .ins()
            .brif(condition, true_block, &[], false_block, &[]);
        for (predecessor_id, block, body) in
            [(0, true_block, true_body), (1, false_block, false_body)]
        {
            builder.switch_to_block(block);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_source_machine_with_continuation(
                builder,
                body.clone(),
                env.to_vec(),
                branch_control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "Bool predecessor {predecessor_id} did not seal its distinct affine join edge"
                    ),
                ));
            }
        }
        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
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
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        builder.ins().brif(success, ok_block, &[], err_block, &[]);

        for (predecessor_id, block, constructor, payload) in [
            (0, ok_block, ok_constructor, ok),
            (1, err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
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
                    branch_control,
                )?
            } else {
                self.lower_source_machine_with_continuation(
                    builder,
                    RuntimeExpr::Trap(default.clone()),
                    env.to_vec(),
                    branch_control,
                )?
            };
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "HostResult predecessor {predecessor_id} did not seal its distinct affine join edge"
                    ),
                ));
            }
        }

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    fn lower_source_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;
        if Self::source_terminal_join(&suffix_control.continuation).is_some() {
            return self.lower_source_nested_dynamic_constructor_match(
                builder,
                dynamic,
                cases,
                default,
                env,
                suffix_control,
            );
        }
        self.lower_source_planned_dynamic_constructor_match(
            builder,
            dynamic,
            cases,
            default,
            env,
            suffix_control,
        )
    }

    fn lower_source_nested_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let SourcePrefixTerminal::Join(inherited_edge) = terminal else {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "nested dynamic constructor has no affine terminal edge",
            ));
        };
        let fanout = SourceBranchFanout {
            source_prefix_template,
            inherited_edge,
        };
        let target = fanout.inherited_edge.target;
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor source match block");
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
            let case = match select_dynamic_constructor_case(cases, &alternative, default)? {
                Ok(case) => case,
                Err(_) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&fanout.source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_source_machine_with_continuation(
                builder,
                case.body.clone(),
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
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
        Ok(Lowered::RecursiveBackedge)
    }

    fn lower_source_planned_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let active = suffix_control
            .selected
            .as_active(&suffix_control.selected_lineage);
        let (prefix, suffix_pending, required_kind, site_id) =
            self.planned_active_scalar_cut(active)?;
        let suffix_pending = suffix_pending.to_vec();
        let join_id = self.next_source_join;
        self.next_source_join = self
            .next_source_join
            .checked_add(1)
            .expect("compiler-private source join identity exhausted");
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let target = SourceJoinTarget {
            join_id,
            block: merge,
            expected_outer: suffix_control.terminal_outer,
            required_kind,
            terminal_active_prefix: prefix,
        };
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let root_authority = match terminal {
            SourcePrefixTerminal::ResumeOuter { root_authority } => root_authority,
            SourcePrefixTerminal::Join(_) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "planned dynamic-constructor cut unexpectedly inherited an executable edge",
                ));
            }
        };
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
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_source_machine_with_continuation(
                builder,
                case.body.clone(),
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "dynamic-constructor predecessor {predecessor_id} for checked site {site_id} did not seal its affine join edge"
                    ),
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
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
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
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
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
                Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                    expr: body,
                    env: call_env,
                    control,
                }))
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
                self.lower_source_declaration_call(
                    builder, symbol, captures, body, args, env, control,
                )
            }
            mut recursor @ Lowered::ComputationalRecursorClosure { .. } => {
                let checked_ih_invocation =
                    self.mint_checked_computational_ih_instance(&mut recursor)?;
                if let Some(CheckedRecursiveInvocationInstance {
                    source: InvocationTemplateRef::ComputationalIHCall(call_template_id),
                    ..
                }) = checked_ih_invocation
                {
                    let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation has no oriented plan",
                        )
                    })?;
                    let call = plan
                        .computational_ih_call(call_template_id)
                        .ok_or_else(|| {
                            unsupported(
                                "OrientedSubcontinuationPlanV1",
                                "checked IH invocation has no exact call template",
                            )
                        })?;
                    let open = control.selected.selected_scope.as_ref().ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation has no selected/open parent occurrence",
                        )
                    })?;
                    self.validate_source_dynamic_splice_parent(
                        checked_ih_invocation.expect("matched checked IH invocation"),
                        open,
                    )?;
                    if call.parent_frame_template_id != open.frame.checked_frame_id
                        || call.parent_segment_site_id
                            != open.frame.checked_frame_id.and_then(|frame_id| {
                                plan.frame(frame_id).map(|frame| frame.segment_site_id)
                            })
                    {
                        return Err(unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation parent edge does not match the active open occurrence",
                        ));
                    }
                }
                let (base, boundary) = decompose_computational_recursor(recursor);
                let (activation, invocation) =
                    boundary.expect("recursor closure carries an invocation segment");
                if source_active_cursor(
                    &control.selected,
                    &control.selected_lineage,
                    invocation.resume_cursor,
                )
                .is_none()
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "recursive invocation cursor is not live in source control",
                    ));
                }
                let armed = ArmedInvocation {
                    suspended: control,
                    expected_selected: invocation.resume_cursor,
                };
                if source_active_cursor(
                    &armed.suspended.selected,
                    &armed.suspended.selected_lineage,
                    armed.expected_selected,
                )
                .is_none()
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "armed invocation endpoint changed selected cursor",
                    ));
                }
                if let Lowered::BoundedNat(predecessor) = base {
                    if !args.is_empty() {
                        return Err(unsupported(
                            "BoundedNat",
                            "structural Nat recursive hypothesis takes no arguments",
                        ));
                    }
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: Lowered::BoundedNat(predecessor),
                        control: suspended,
                    }));
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
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                        expr: body,
                        env: call_env,
                        control: suspended,
                    }));
                }
            }
            _ => Err(unsupported("Call", "callee is not a closure")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_declaration_call<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: RuntimeSymbol,
        captures: Vec<Lowered>,
        body: RuntimeExpr,
        args: Vec<Lowered>,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(&symbol)?;
        if !self.declaration_is_recursive(&symbol) {
            let mut call_env = args;
            call_env.extend(captures);
            call_env.extend(env);
            return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                expr: body,
                env: call_env,
                control,
            }));
        }

        if let Some(active) = self
            .active_recursive_declarations
            .iter()
            .rev()
            .find(|active| active.symbol == symbol)
            .cloned()
        {
            if !same_recursive_argument_shapes(&active.argument_templates, &args) {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "recursive declaration {symbol} changes its native argument representation: {:?} -> {:?}",
                        active
                            .argument_templates
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>(),
                        args.iter().map(lowered_value_kind).collect::<Vec<_>>()
                    ),
                ));
            }
            if let Some(induction) = active.induction {
                return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                    value: induction,
                    control,
                }));
            }
            let mut values = Vec::new();
            append_recursive_argument_values(builder, &args, &mut values, &self.native_int_tags)?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive source declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(SourceCallOutcome::Complete(Lowered::RecursiveBackedge));
        }

        let header = builder.create_block();
        let mut initial_values = Vec::new();
        append_recursive_argument_values(
            builder,
            &args,
            &mut initial_values,
            &self.native_int_tags,
        )?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
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
        let mut loop_args = Vec::with_capacity(args.len());
        for template in &args {
            loop_args.push(rebuild_recursive_argument(
                template,
                &mut parameters,
                &mut self.native_int_tags,
            )?);
        }
        if parameters.next().is_some() {
            return Err(unsupported(
                "DeclarationRef",
                "recursive source declaration loop parameter shape is not closed",
            ));
        }
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: Some(header),
                argument_templates: args,
                induction: None,
            });
        let mut call_env = loop_args.into_iter().rev().collect::<Vec<_>>();
        call_env.extend(captures);
        call_env.extend(env);
        let lowered = self.lower_source_machine_with_continuation(builder, body, call_env, control);
        self.active_recursive_declarations.pop();
        Ok(SourceCallOutcome::Complete(lowered?))
    }

    fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: &RuntimeExpr,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        match expr {
            RuntimeExpr::Value(value) => self.lower_value(builder, value),
            RuntimeExpr::CheckedJoinSite { site_id, body } => {
                if self.active_join_site.replace(*site_id).is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "nested checked join occurrence marker",
                    ));
                }
                let result = self.lower_expr(builder, body, env);
                if self.active_join_site.take().is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence marker was not consumed",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let result = self.lower_expr(builder, body, env);
                if self.active_subcontinuation_frame.take().is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "checked subcontinuation marker was not consumed by its frame",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                body,
                ..
            } => {
                let instance =
                    self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let result = self.lower_expr(builder, body, env);
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                self.lower_expr(builder, body, env)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let value = self.lower_expr(builder, body, env)?;
                self.finish_checked_computational_ih_marker(value)
            }
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
                builder.append_block_param(merge, types::I64);
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                for (block, arm) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_expr(builder, arm, env)?;
                    let Lowered::Int { value, known } = lowered else {
                        return Err(unsupported(
                            "If",
                            "dynamic native If arms must produce scalar Int values",
                        ));
                    };
                    let tag = self.native_int_tag(builder, value, known)?;
                    builder.ins().jump(merge, &[tag.into(), value.into()]);
                }
                builder.switch_to_block(merge);
                let tag = builder.block_params(merge)[0];
                let value = builder.block_params(merge)[1];
                self.native_int_tags.insert(value, tag);
                Ok(Lowered::Int {
                    value,
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
                        builder
                            .ins()
                            .jump(merge, &[value.tag.into(), value.payload.into()]);
                    }
                    builder.switch_to_block(merge);
                    let pair = NativeScalarPairV1 {
                        tag: builder.block_params(merge)[0],
                        payload: builder.block_params(merge)[1],
                    };
                    return Ok(self.lowered_from_scalar_pair(
                        merge_kind.expect("Bool match emits both closed alternatives"),
                        pair,
                    ));
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
                    mut callee @ Lowered::ComputationalRecursorClosure { .. } => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(callee);
                        let (activation, invocation) = boundary.expect(
                            "recursor closure carries an invocation segment",
                        );
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges =
                            self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let mut frames = installed_oriented_eliminator_frames(&installed);
                        frames.push(EliminatorFrame::InvocationReturn);
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &frames,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
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
                        self.enter_oriented_semantic_region(installed.checked);
                        let result = self.lower_computational_producer_expr(
                            builder,
                            &body,
                            &call_env,
                            &frames,
                        );
                        self.leave_oriented_semantic_region(installed.checked);
                        result
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
        let mut narrow_failure: Option<(
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        )> = None;
        let mut positioned_bounds: Option<(
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        )> = None;
        let mut record_narrow_failure =
            |builder: &mut FunctionBuilder<'_>, invalid, detail: i64| {
                let detail = builder.ins().iconst(types::I64, detail);
                narrow_failure = Some(match narrow_failure.take() {
                    Some((prior_invalid, prior_detail)) => (
                        builder.ins().bor(prior_invalid, invalid),
                        builder.ins().select(prior_invalid, prior_detail, detail),
                    ),
                    None => (invalid, detail),
                });
            };
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
                    let mode = lowered.get(1).ok_or_else(|| {
                        unsupported("Effect", "FS.ChangeMode is missing its mode")
                    })?;
                    let (mode, valid_int) = self.narrow_native_int_u64(builder, mode)?;
                    let in_range = builder.ins().icmp_imm(
                        cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
                        mode,
                        0o7777,
                    );
                    let in_range = builder.ins().band(valid_int, in_range);
                    let narrowed = builder.ins().ireduce(types::I16, mode);
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
                let capacity = lowered.first().ok_or_else(|| {
                    unsupported("Effect", "BufferAllocate is missing its capacity")
                })?;
                let (capacity, valid) = self.narrow_native_int_u64(builder, capacity)?;
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                record_narrow_failure(builder, invalid, 7);
                builder
                    .ins()
                    .stack_store(capacity, request, request_offset(0));
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
                let start = lowered
                    .get(1)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its start"))?;
                let length = lowered
                    .get(2)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its length"))?;
                let (start, start_valid) = self.narrow_native_int_u64(builder, start)?;
                let (length, length_valid) = self.narrow_native_int_u64(builder, length)?;
                let valid = builder.ins().band(start_valid, length_valid);
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                record_narrow_failure(builder, invalid, 7);
                for (index, value) in [*token, start, length].into_iter().enumerate() {
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
                    let Some(value @ Lowered::Int { .. }) = lowered.get(index) else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not Int"),
                        ));
                    };
                    Ok(value)
                };
                let file = resource(0, "file")?;
                let (file_offset, file_offset_valid) =
                    self.narrow_native_int_u64(builder, integer(1, "file offset")?)?;
                let buffer = resource(2, "buffer")?;
                let (buffer_start, buffer_start_valid) =
                    self.narrow_native_int_u64(builder, integer(3, "buffer start")?)?;
                let (length, length_valid) =
                    self.narrow_native_int_u64(builder, integer(4, "length")?)?;
                positioned_bounds = Some((buffer_start, length));
                let file_offset_invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    file_offset_valid,
                    0,
                );
                record_narrow_failure(builder, file_offset_invalid, 6);
                let bounds_valid = builder.ins().band(buffer_start_valid, length_valid);
                let bounds_invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    bounds_valid,
                    0,
                );
                record_narrow_failure(builder, bounds_invalid, 7);
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
        if let Some((invalid, detail)) = narrow_failure {
            let dispatch = builder.create_block();
            let synthesize = builder.create_block();
            let decoded = builder.create_block();
            builder.ins().brif(invalid, synthesize, &[], dispatch, &[]);

            builder.switch_to_block(dispatch);
            let call = builder.ins().call(
                self.host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[invocation, op, request_pointer, request_size, reply_pointer],
            );
            let status = builder.inst_results(call)[0];
            Self::require_i64(builder, status, 0);
            builder.ins().jump(decoded, &[]);

            builder.switch_to_block(synthesize);
            let zero = builder.ins().iconst(types::I64, 0);
            for offset in [
                wire.reply_resource_error_schema_offset,
                wire.reply_resource_error_kind_offset,
                wire.reply_resource_error_identity_offset,
                wire.reply_resource_error_io_offset,
                wire.reply_resource_error_required_offset,
                wire.reply_resource_error_held_offset,
                wire.reply_resource_error_expected_kind_offset,
                wire.reply_resource_error_actual_kind_offset,
                wire.reply_bytes_data_offset,
                wire.reply_bytes_len_offset,
            ] {
                builder.ins().stack_store(
                    zero,
                    reply,
                    i32::try_from(offset).expect("reply field offset is u32"),
                );
            }
            let resource_error_tag = builder
                .ins()
                .iconst(types::I64, wire.reply_resource_error_tag as i64);
            builder.ins().stack_store(
                resource_error_tag,
                reply,
                i32::try_from(wire.reply_tag_offset).expect("reply tag offset is u32"),
            );
            builder.ins().stack_store(
                detail,
                reply,
                i32::try_from(wire.reply_detail_offset).expect("reply detail offset is u32"),
            );
            builder.ins().jump(decoded, &[]);
            builder.switch_to_block(decoded);
        } else {
            let call = builder.ins().call(
                self.host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[invocation, op, request_pointer, request_size, reply_pointer],
            );
            let status = builder.inst_results(call)[0];
            Self::require_i64(builder, status, 0);
        }
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
            let payload_int = self.lower_dynamic_small_int(builder, payload);
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
                            .then(|| vec![payload_int.clone()])
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
                let surface_io_payload_int =
                    self.lower_dynamic_small_int(builder, surface_io_payload);
                let resource_required_int =
                    self.lower_unsigned_u64_int(builder, resource_required)?;
                let resource_held_int = self.lower_unsigned_u64_int(builder, resource_held)?;
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
                                .then(|| vec![surface_io_payload_int.clone()])
                                .unwrap_or_default(),
                        })
                        .collect(),
                });
                let identity_low = builder.ins().band_imm(resource_identity, 0xffff_ffff);
                let identity_high = builder.ins().ushr_imm(resource_identity, 32);
                let identity_low_int = self.lower_dynamic_small_int(builder, identity_low);
                let identity_high_int = self.lower_dynamic_small_int(builder, identity_high);
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
                            fields: vec![resource_required_int, resource_held_int],
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
                                    args: vec![identity_low_int, identity_high_int],
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
                let (request_start, request_length) = positioned_bounds
                    .expect("positioned request bounds were narrowed before dispatch");
                let (count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    read_some,
                    detail,
                    request_start,
                    request_length,
                    Some(reply_start),
                );
                let reply_start_int = self.lower_unsigned_u64_int(builder, reply_start)?;
                let span = Lowered::Constructor {
                    constructor: self.process_symbols.private_buffer_span.clone(),
                    args: vec![reply_start_int, Lowered::BoundedNat(count)],
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
                let (request_start, request_length) = positioned_bounds
                    .expect("positioned request bounds were narrowed before dispatch");
                let (_count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    success,
                    detail,
                    request_start,
                    request_length,
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
                self.lower_unsigned_u64_int(builder, detail)?
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

    fn narrow_native_int_u64(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value), CraneliftBackendError>
    {
        let Lowered::Int { value, known } = value else {
            return Err(unsupported("Effect", "host-width operand is not Int"));
        };
        let arena = self
            .native_int_arena
            .ok_or_else(|| unsupported("Effect", "host-width Int has no invocation arena"))?;
        let helper = self.native_int_narrow.ok_or_else(|| {
            unsupported("Effect", "host-width Int has no checked narrowing helper")
        })?;
        let tag = self.native_int_tag(builder, *value, *known)?;
        let output_slot =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        let pointer_type = builder.func.dfg.value_type(arena);
        let output = builder.ins().stack_addr(pointer_type, output_slot, 0);
        let call = builder.ins().call(helper, &[arena, tag, *value, output]);
        let status = builder.inst_results(call)[0];
        Self::require_one_of_i64(builder, status, &[0, 1]);
        let valid =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, status, 0);
        let value = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), output, 0);
        Ok((value, valid))
    }

    fn lower_dynamic_small_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: cranelift_codegen::ir::Value,
    ) -> Lowered {
        let tag = builder
            .ins()
            .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
        self.native_int_tags.insert(value, tag);
        Lowered::Int { value, known: None }
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
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder.ins().jump(
            loop_block,
            &[zero.into(), initial.tag.into(), initial.payload.into()],
        );
        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction = NativeScalarPairV1 {
            tag: builder.block_params(loop_block)[1],
            payload: builder.block_params(loop_block)[2],
        };
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            target,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction.tag.into(), induction.payload.into()],
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
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
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
        builder.ins().jump(
            loop_block,
            &[successor_value.into(), next.tag.into(), next.payload.into()],
        );
        builder.switch_to_block(done_block);
        Ok(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        ))
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
        let _checked_invocation = self.consume_checked_recursive_invocation_call(symbol)?;
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
            append_recursive_argument_values(
                builder,
                &lowered_args,
                &mut values,
                &self.native_int_tags,
            )?;
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
        append_recursive_argument_values(
            builder,
            &lowered_args,
            &mut initial_values,
            &self.native_int_tags,
        )?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
        builder.append_block_param(done, types::I64);
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
        let mut loop_args = Vec::with_capacity(lowered_args.len());
        for template in &lowered_args {
            loop_args.push(rebuild_recursive_argument(
                template,
                &mut parameters,
                &mut self.native_int_tags,
            )?);
        }
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
        let (value, result_kind) = self.merge_scalar_branch(builder, lowered, "DeclarationRef")?;
        builder
            .ins()
            .jump(done, &[value.tag.into(), value.payload.into()]);
        builder.switch_to_block(done);
        Ok(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done)[0],
                payload: builder.block_params(done)[1],
            },
        ))
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
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("borrowed match emits at least one case"),
            pair,
        ))
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
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(if exit_merge == Some(true) {
            Lowered::ProcessExitStatus {
                value: pair.payload,
            }
        } else {
            self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
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
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("HostResult emits both closed alternatives"),
            pair,
        ))
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
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("both structural Nat arms were emitted"),
            pair,
        ))
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
                &[value.tag.into(), value.payload.into()],
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
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("a selected dynamic constructor case emits one arm"),
            pair,
        ))
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
            RuntimeValue::Int(crate::RuntimeIntV1::Small(value)) => Ok(Lowered::Int {
                value: builder.ins().iconst(types::I64, *value),
                known: Some(*value),
            }),
            RuntimeValue::Int(value @ crate::RuntimeIntV1::Big { .. }) => {
                self.lower_big_int_constant(builder, value)
            }
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
            RuntimeGroundValue::Int(crate::RuntimeIntV1::Small(value)) => Ok(Lowered::Int {
                value: builder.ins().iconst(types::I64, *value),
                known: Some(*value),
            }),
            RuntimeGroundValue::Int(value @ crate::RuntimeIntV1::Big { .. }) => {
                self.lower_big_int_constant(builder, value)
            }
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

    fn lower_big_int_constant(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &crate::RuntimeIntV1,
    ) -> Result<Lowered, CraneliftBackendError> {
        let crate::RuntimeIntV1::Big { sign, limbs } = value else {
            unreachable!("Big constant lowering is called only for Big Int values")
        };
        let limb_count = limbs.len();
        let byte_len = u32::try_from(limbs.len().saturating_mul(std::mem::size_of::<u64>()))
            .map_err(|_| unsupported("RuntimeValue::Int", "Big Int literal is too large"))?;
        let limbs_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            byte_len,
            3,
        ));
        for (index, limb) in limbs.iter().enumerate() {
            let limb = builder.ins().iconst(types::I64, *limb as i64);
            builder.ins().stack_store(
                limb,
                limbs_slot,
                i32::try_from(index * std::mem::size_of::<u64>()).expect("Big limb offset is u32"),
            );
        }
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let pointer_type = builder.func.dfg.value_type(
            self.native_int_arena
                .ok_or_else(|| unsupported("RuntimeValue::Int", "Big Int has no arena"))?,
        );
        let arena = self.native_int_arena.expect("Big Int arena was checked");
        let helper = self.native_int_intern.ok_or_else(|| {
            unsupported("RuntimeValue::Int", "Big Int has no local intern helper")
        })?;
        let sign = builder
            .ins()
            .iconst(types::I64, i64::from(matches!(sign, crate::Sign::Negative)));
        let limbs = builder.ins().stack_addr(pointer_type, limbs_slot, 0);
        let len = builder.ins().iconst(
            types::I64,
            i64::try_from(limb_count).expect("Big limb count fits i64"),
        );
        let output_ptr = builder.ins().stack_addr(pointer_type, output, 0);
        let call = builder
            .ins()
            .call(helper, &[arena, sign, limbs, len, output_ptr]);
        Self::require_i64(builder, builder.inst_results(call)[0], 0);
        let pair = NativeScalarPairV1 {
            tag: builder.ins().stack_load(types::I64, output, 0),
            payload: builder.ins().stack_load(types::I64, output, 8),
        };
        Ok(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
    }

    /// Reify a host-owned unsigned word into the exact native Int carrier.
    /// The shared local interner chooses Small or canonical Big; callers never
    /// reinterpret the raw `u64` bits as a signed scalar.
    fn lower_unsigned_u64_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: cranelift_codegen::ir::Value,
    ) -> Result<Lowered, CraneliftBackendError> {
        let arena = self.native_int_arena.ok_or_else(|| {
            unsupported("NativeInt", "unsigned Int producer has no invocation arena")
        })?;
        let helper = self.native_int_intern.ok_or_else(|| {
            unsupported(
                "NativeInt",
                "unsigned Int producer has no local intern helper",
            )
        })?;
        let limb =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        builder.ins().stack_store(value, limb, 0);
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let pointer_type = builder.func.dfg.value_type(arena);
        let limb = builder.ins().stack_addr(pointer_type, limb, 0);
        let output_pointer = builder.ins().stack_addr(pointer_type, output, 0);
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let call = builder
            .ins()
            .call(helper, &[arena, zero, limb, one, output_pointer]);
        Self::require_i64(builder, builder.inst_results(call)[0], 0);
        let pair = NativeScalarPairV1 {
            tag: builder.ins().stack_load(types::I64, output, 0),
            payload: builder.ins().stack_load(types::I64, output, 8),
        };
        Self::require_one_of_i64(
            builder,
            pair.tag,
            &[
                crate::NATIVE_INT_SMALL_TAG_V1 as i64,
                crate::NATIVE_INT_BIG_TAG_V1 as i64,
            ],
        );
        Ok(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
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
            "add_int" => self.lower_int_binop(builder, "add_int", lowered_args, |lhs, rhs| {
                lhs.checked_add(rhs)
            }),
            "sub_int" => self.lower_int_binop(builder, "sub_int", lowered_args, |lhs, rhs| {
                lhs.checked_sub(rhs)
            }),
            "mul_int" => self.lower_int_binop(builder, "mul_int", lowered_args, |lhs, rhs| {
                lhs.checked_mul(rhs)
            }),
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
        #[cfg(test)]
        match self.native_int_mutation {
            NativeIntLoweringMutation::Exact => {}
            NativeIntLoweringMutation::Wrapping => {}
            NativeIntLoweringMutation::Trap => {
                return Err(unsupported(
                    "PrimitiveCall",
                    "PX8-I mutation traps before exact Int support",
                ));
            }
            NativeIntLoweringMutation::SuppressTerminalExport
            | NativeIntLoweringMutation::CorruptTerminalExport => {}
        }
        let lhs_tag = self.native_int_tag(builder, lhs, lhs_known)?;
        let rhs_tag = self.native_int_tag(builder, rhs, rhs_known)?;
        let arena = self.native_int_arena.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int operation has no invocation arena",
            )
        })?;
        let helper = self.native_int_binop.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int operation has no local support function",
            )
        })?;
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let pointer_type = builder.func.dfg.value_type(arena);
        let output_pointer = builder.ins().stack_addr(pointer_type, output, 0);
        let operation = builder.ins().iconst(
            types::I64,
            match symbol {
                "add_int" => 0,
                "sub_int" => 1,
                "mul_int" => 2,
                _ => unreachable!("caller supplies exact Int arithmetic symbol"),
            },
        );
        let call = builder.ins().call(
            helper,
            &[arena, operation, lhs_tag, lhs, rhs_tag, rhs, output_pointer],
        );
        let status = builder.inst_results(call)[0];
        Self::require_i64(builder, status, 0);
        let tag = builder.ins().stack_load(types::I64, output, 0);
        let value = builder.ins().stack_load(types::I64, output, 8);
        Self::require_one_of_i64(
            builder,
            tag,
            &[
                crate::NATIVE_INT_SMALL_TAG_V1 as i64,
                crate::NATIVE_INT_BIG_TAG_V1 as i64,
            ],
        );
        self.native_int_tags.insert(value, tag);
        let known = lhs_known.and_then(|lhs| rhs_known.and_then(|rhs| eval(lhs, rhs)));
        Ok(Lowered::Int { value, known })
    }

    fn lower_int_cmp(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        _cc: cranelift_codegen::ir::condcodes::IntCC,
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
        let lhs_tag = self.native_int_tag(builder, lhs, lhs_known)?;
        let rhs_tag = self.native_int_tag(builder, rhs, rhs_known)?;
        let arena = self.native_int_arena.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int comparison has no invocation arena",
            )
        })?;
        let helper = self.native_int_compare.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int comparison has no local support function",
            )
        })?;
        let operation = builder.ins().iconst(
            types::I64,
            match symbol {
                "eq_int" => 0,
                "leq_int" => 1,
                _ => unreachable!("caller supplies exact Int comparison symbol"),
            },
        );
        let call = builder
            .ins()
            .call(helper, &[arena, operation, lhs_tag, lhs, rhs_tag, rhs]);
        let value = builder.inst_results(call)[0];
        Self::require_one_of_i64(builder, value, &[0, 1]);
        Ok(Lowered::Bool {
            value,
            known: lhs_known.and_then(|lhs| rhs_known.map(|rhs| eval(lhs, rhs))),
        })
    }

    fn native_int_tag(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        payload: cranelift_codegen::ir::Value,
        known: Option<i64>,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        if let Some(tag) = self.native_int_tags.get(&payload).copied() {
            return Ok(tag);
        }
        if known.is_some() {
            return Ok(builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64));
        }
        Err(unsupported(
            "NativeInt",
            "dynamic Int value lost its two-word tag transport",
        ))
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
            return self.lower_unsigned_u64_int(builder, len);
        }
        if let Lowered::BorrowedNativeValue { pointer } = arg {
            let kind = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 0);
            Self::require_i64(builder, kind, 1);
            let len = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 24);
            return self.lower_unsigned_u64_int(builder, len);
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
            let value = builder.block_params(merge)[1];
            let tag = builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
            self.native_int_tags.insert(value, tag);
            return Ok(Lowered::BorrowedOption {
                present: builder.block_params(merge)[0],
                value,
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
            let value = builder.block_params(merge)[0];
            let tag = builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
            self.native_int_tags.insert(value, tag);
            return Ok(Lowered::BorrowedOption {
                present,
                value,
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
            let _authority = self.mint_terminal_answer_authority()?;
            let value = Self::unwrap_terminal_ret(value);
            let value = match value {
                Lowered::ProcessExitStatus { value } => value,
                value => self.emit_process_exit_status(builder, value),
            };
            return Ok((value, ResultDecoder::ProcessStatus));
        }
        match value {
            Lowered::Int { value, known } => {
                let tag = self.native_int_tag(builder, value, known)?;
                let arena = self.native_int_arena.ok_or_else(|| {
                    unsupported("NativeResult", "Int result has no invocation arena")
                })?;
                let export = self.native_int_export.ok_or_else(|| {
                    unsupported("NativeResult", "Int result has no export support function")
                })?;
                #[cfg(test)]
                if self.native_int_mutation == NativeIntLoweringMutation::SuppressTerminalExport {
                    return Ok((value, ResultDecoder::Int));
                }
                let call = builder.ins().call(export, &[arena, tag, value]);
                Self::require_i64(builder, builder.inst_results(call)[0], 0);
                #[cfg(test)]
                if self.native_int_mutation == NativeIntLoweringMutation::CorruptTerminalExport {
                    let invalid = builder.ins().iconst(types::I64, 7);
                    builder.ins().store(
                        MemFlags::trusted(),
                        invalid,
                        arena,
                        crate::native_int_clif::ARENA_FINAL_TAG,
                    );
                }
                Ok((value, ResultDecoder::Int))
            }
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
        let Lowered::Int { known, .. } = &payload else {
            return builder.ins().iconst(types::I64, -3);
        };
        if let Some(code) = *known {
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
        let Ok((value, valid_int)) = self.narrow_native_int_u64(builder, &payload) else {
            return builder.ins().iconst(types::I64, -3);
        };
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let max = builder.ins().iconst(types::I64, 255);
        let malformed = builder.ins().iconst(types::I64, -3);
        let is_zero =
            builder
                .ins()
                .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, value, zero);
        let positive = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            value,
            zero,
        );
        let within_max = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            value,
            max,
        );
        let valid = builder.ins().band(valid_int, positive);
        let valid = builder.ins().band(valid, within_max);
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
            } => Ok(RuntimeGroundValue::Int((value).into())),
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
    builder: &mut FunctionBuilder<'_>,
    values: &[Lowered],
    output: &mut Vec<cranelift_codegen::ir::Value>,
    native_int_tags: &BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
) -> Result<(), CraneliftBackendError> {
    for value in values {
        match value {
            Lowered::Int { value, known } => {
                let tag = match native_int_tags.get(value).copied() {
                    Some(tag) => tag,
                    None if known.is_some() => builder
                        .ins()
                        .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64),
                    None => {
                        return Err(unsupported(
                            "DeclarationRef",
                            "recursive Int argument lost its two-word tag transport",
                        ));
                    }
                };
                output.push(tag);
                output.push(*value);
            }
            Lowered::Bool { value, .. }
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
                append_recursive_argument_values(builder, args, output, native_int_tags)?;
            }
            Lowered::Record { fields } => {
                for (_, field) in fields {
                    append_recursive_argument_values(
                        builder,
                        std::slice::from_ref(field),
                        output,
                        native_int_tags,
                    )?;
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
    native_int_tags: &mut BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
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
        Lowered::Int { .. } => {
            let tag = next(values)?;
            let value = next(values)?;
            native_int_tags.insert(value, tag);
            Lowered::Int { value, known: None }
        }
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
                .map(|arg| rebuild_recursive_argument(arg, values, native_int_tags))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Lowered::Record { fields } => Lowered::Record {
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rebuild_recursive_argument(value, values, native_int_tags)?,
                    ))
                })
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

pub(crate) fn backend_module(reason: String) -> CraneliftBackendError {
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

    fn oriented_test_interface(name: u8) -> crate::CheckedAnswerInterfaceV1 {
        let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
        bytes.push(name);
        crate::CheckedAnswerInterfaceV1::new(bytes).unwrap()
    }

    fn oriented_test_frame(
        frame_id: u64,
        semantic_position: u64,
        input: u8,
        output: u8,
        parent: Option<u64>,
    ) -> crate::OrientedSubcontinuationFramePlanV1 {
        let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
            frame_id,
            segment_site_id: 9,
            declaration: "decl:fixture::oriented".to_string(),
            checked_occurrence_path: vec![frame_id],
            semantic_position,
            input_interface: oriented_test_interface(input),
            output_interface: oriented_test_interface(output),
            runtime_frame_fingerprint: frame_id + 100,
            occurrence_binding_fingerprint: 0,
            control_witness: parent.map_or(
                crate::OrientedControlWitnessV1::DistinguishedRoot,
                crate::OrientedControlWitnessV1::ParentFrame,
            ),
        };
        frame.occurrence_binding_fingerprint =
            crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
        frame
    }

    fn oriented_test_layer(frame_id: u64, role: RecursorLayerRole) -> ComputationalRecursorLayer {
        ComputationalRecursorLayer {
            cases: Vec::new(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: format!("oriented frame {frame_id}"),
            },
            outer_env: Vec::new(),
            provenance: RecursorFrameProvenance(frame_id),
            role,
            checked_frame_id: Some(frame_id),
            checked_invocation_id: Some(0),
            checked_invocation_source: None,
            checked_invocation_depth: 0,
            semantic_pending: true,
        }
    }

    fn oriented_test_plan() -> crate::OrientedSubcontinuationPlanV1 {
        crate::OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            // Checked postorder is p2, p1, p0 even though control returns
            // through o0, o4, o3 below.
            frames: vec![
                oriented_test_frame(0, 2, 2, 3, None),
                oriented_test_frame(1, 1, 1, 2, Some(0)),
                oriented_test_frame(2, 0, 0, 1, Some(1)),
            ],
            recursive_calls: Vec::new(),
            computational_ih_slots: Vec::new(),
            computational_ih_calls: Vec::new(),
        }
    }

    fn oriented_test_ih_plan() -> crate::OrientedSubcontinuationPlanV1 {
        let mut plan = oriented_test_plan();
        for frame_id in 0..=2 {
            let slot_template_id = 200 + frame_id;
            let mut slot = crate::CheckedComputationalIHSlotTemplateV1 {
                slot_template_id,
                declaration: "decl:fixture::oriented".to_string(),
                checked_match_ordinal: frame_id,
                checked_occurrence_path: vec![20, frame_id],
                frame_template_id: frame_id,
                constructor: format!("Ctor{frame_id}"),
                recursive_position: 0,
                method_binder_ordinal: 0,
                local_telescope: Vec::new(),
                ih_interface: oriented_test_interface(frame_id as u8),
                segment_site_id: 9,
                frame_templates: vec![frame_id],
                input_interface: oriented_test_interface(frame_id as u8),
                output_interface: oriented_test_interface(frame_id as u8 + 1),
                runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
                    declaration: "decl:fixture::oriented".to_string(),
                    runtime_path: vec![0, frame_id],
                }],
                occurrence_binding_fingerprint: 0,
            };
            slot.occurrence_binding_fingerprint =
                crate::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
            plan.computational_ih_slots.push(slot);

            let mut call = crate::CheckedComputationalIHCallTemplateV1 {
                call_template_id: 100 + frame_id,
                declaration: "decl:fixture::oriented".to_string(),
                checked_occurrence_path: vec![30, frame_id],
                slot_template_id,
                arity: 1,
                local_telescope: Vec::new(),
                result_interface: oriented_test_interface(frame_id as u8 + 1),
                callee_segment_site_id: 9,
                callee_frame_templates: vec![frame_id],
                parent_frame_template_id: Some(frame_id),
                parent_segment_site_id: Some(9),
                caller_interface: oriented_test_interface(frame_id as u8 + 1),
                runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
                    declaration: "decl:fixture::oriented".to_string(),
                    runtime_path: vec![1, frame_id],
                }],
                occurrence_binding_fingerprint: 0,
            };
            call.occurrence_binding_fingerprint =
                crate::compiler_private_computational_ih_call_binding_fingerprint(&call);
            plan.computational_ih_calls.push(call);
        }
        plan.validate().unwrap();
        plan
    }

    fn oriented_test_instance_layer(
        frame_id: u64,
        invocation_id: u64,
        semantic_depth: usize,
        semantic_pending: bool,
        role: RecursorLayerRole,
    ) -> ComputationalRecursorLayer {
        let mut layer = oriented_test_layer(frame_id, role);
        layer.checked_invocation_id = Some(invocation_id);
        layer.checked_invocation_source =
            Some(InvocationTemplateRef::ComputationalIHCall(100 + frame_id));
        layer.checked_invocation_depth = semantic_depth;
        layer.semantic_pending = semantic_pending;
        layer
    }

    fn oriented_five_control_invocation() -> RecursorInvocationSegment {
        let origin = RecursorProducerOriginId(40);
        let mut invocation = RecursorInvocationSegment::new(
            origin,
            0,
            oriented_test_instance_layer(
                2,
                0,
                0,
                true,
                RecursorLayerRole::SelectsOccurrence { origin },
            ),
            RecursorUnwindStack {
                later_wrappers_in_construction_order: vec![
                    oriented_test_instance_layer(
                        2,
                        1,
                        0,
                        false,
                        RecursorLayerRole::ExitsScope {
                            origin,
                            scope_origin: RecursorProducerOriginId(41),
                            parent_scope: None,
                        },
                    ),
                    oriented_test_instance_layer(
                        0,
                        1,
                        0,
                        false,
                        RecursorLayerRole::ExitsScope {
                            origin,
                            scope_origin: RecursorProducerOriginId(42),
                            parent_scope: Some(RecursorProducerOriginId(41)),
                        },
                    ),
                    oriented_test_instance_layer(
                        0,
                        0,
                        0,
                        true,
                        RecursorLayerRole::ExitsScope {
                            origin,
                            scope_origin: RecursorProducerOriginId(43),
                            parent_scope: Some(RecursorProducerOriginId(42)),
                        },
                    ),
                    oriented_test_instance_layer(
                        1,
                        0,
                        0,
                        true,
                        RecursorLayerRole::ExitsScope {
                            origin,
                            scope_origin: RecursorProducerOriginId(44),
                            parent_scope: Some(RecursorProducerOriginId(43)),
                        },
                    ),
                ],
            },
            ContinuationCursorId(7),
            None,
            None,
        );
        for layer in &mut invocation.unwind.later_wrappers_in_construction_order[..2] {
            layer.checked_invocation_source = Some(InvocationTemplateRef::SameSccCall(999));
        }
        invocation.selection.checked_invocation_source = None;
        for layer in &mut invocation.unwind.later_wrappers_in_construction_order {
            if layer.semantic_pending {
                layer.checked_invocation_source = None;
            }
        }
        invocation
    }

    fn oriented_test_invocation() -> RecursorInvocationSegment {
        let origin = RecursorProducerOriginId(40);
        RecursorInvocationSegment::new(
            origin,
            0,
            oriented_test_layer(0, RecursorLayerRole::SelectsOccurrence { origin }),
            RecursorUnwindStack {
                later_wrappers_in_construction_order: vec![
                    oriented_test_layer(
                        1,
                        RecursorLayerRole::ExitsScope {
                            origin,
                            scope_origin: RecursorProducerOriginId(41),
                            parent_scope: None,
                        },
                    ),
                    oriented_test_layer(
                        2,
                        RecursorLayerRole::ExitsScope {
                            origin,
                            scope_origin: RecursorProducerOriginId(42),
                            parent_scope: Some(RecursorProducerOriginId(41)),
                        },
                    ),
                ],
            },
            ContinuationCursorId(7),
            None,
            None,
        )
    }

    #[test]
    fn oriented_segment_keeps_semantic_and_control_axes_independent() {
        let installed = compose_oriented_subcontinuation(
            Some(&oriented_test_plan()),
            None,
            ContinuationActivationId(8),
            oriented_test_invocation(),
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            installed
                .semantic_frames
                .iter()
                .map(|frame| frame.checked_frame_id.unwrap())
                .collect::<Vec<_>>(),
            vec![2, 1, 0],
            "checked composition order is p2, p1, p0"
        );
        assert_eq!(
            installed
                .control_ledger
                .iter()
                .map(|entry| entry.frame_id.unwrap())
                .collect::<Vec<_>>(),
            vec![0, 2, 1],
            "delimiter order remains independently o0, o4, o3"
        );
    }

    #[test]
    fn oriented_endpoint_corruption_and_affine_reuse_fail_closed() {
        let mut plan = oriented_test_plan();
        plan.frames[2].output_interface = oriented_test_interface(9);
        plan.frames[2].occurrence_binding_fingerprint =
            crate::compiler_private_oriented_occurrence_binding_fingerprint(&plan.frames[2]);
        let error = match compose_oriented_subcontinuation(
            Some(&plan),
            None,
            ContinuationActivationId(8),
            oriented_test_invocation(),
            Vec::new(),
        ) {
            Ok(_) => panic!("endpoint corruption must reject before installation"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "OrientedSubcontinuationPlanV1",
                reason,
            }) if reason.contains("endpoints do not compose")
        ));

        let mut capability = AffineSpliceCapability {
            state: AffineSpliceState::Open,
        };
        capability.consume().unwrap();
        assert!(capability.consume().is_err());
    }

    #[test]
    fn oriented_fresh_ih_semantics_retain_all_inherited_control_obligations() {
        let installed = compose_oriented_subcontinuation(
            Some(&oriented_test_ih_plan()),
            None,
            ContinuationActivationId(8),
            oriented_five_control_invocation(),
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            installed
                .semantic_frames
                .iter()
                .map(|frame| {
                    (
                        frame.checked_invocation_id.unwrap(),
                        frame.checked_frame_id.unwrap(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![(0, 2), (0, 1), (0, 0)],
        );
        assert_eq!(installed.control_ledger.len(), 5);
        assert_eq!(
            installed
                .control_ledger
                .iter()
                .filter(|entry| matches!(entry.role, RecursorLayerRole::ExitsScope { .. }))
                .count(),
            4,
        );
    }

    #[test]
    fn oriented_phase_misclassification_recovers_endpoint_and_missing_semantic_rejections() {
        let plan = oriented_test_ih_plan();
        let mut replayed = oriented_five_control_invocation();
        replayed.unwind.later_wrappers_in_construction_order[0].semantic_pending = true;
        replayed.open_control_obligations = open_control_obligations(&replayed.unwind);
        let replayed = match compose_oriented_subcontinuation(
            Some(&plan),
            None,
            ContinuationActivationId(8),
            replayed,
            Vec::new(),
        ) {
            Ok(_) => panic!("an inherited open scope cannot replay its semantic transformer"),
            Err(error) => error,
        };
        assert!(matches!(
            replayed,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason.contains("exact invocation-local tree")
        ));

        let mut omitted = oriented_five_control_invocation();
        omitted.selection.semantic_pending = false;
        let omitted = match compose_oriented_subcontinuation(
            Some(&plan),
            None,
            ContinuationActivationId(8),
            omitted,
            Vec::new(),
        ) {
            Ok(_) => panic!("a pending selection cannot be omitted from semantic work"),
            Err(error) => error,
        };
        assert!(matches!(
            omitted,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason == "pending selection was misclassified as control-only"
        ));
    }

    #[test]
    fn oriented_open_control_obligations_are_affine_and_mint_exact() {
        let plan = oriented_test_ih_plan();
        let mut deleted = oriented_five_control_invocation();
        deleted
            .unwind
            .later_wrappers_in_construction_order
            .remove(0);
        let deleted = match compose_oriented_subcontinuation(
            Some(&plan),
            None,
            ContinuationActivationId(8),
            deleted,
            Vec::new(),
        ) {
            Ok(_) => panic!("deleting only an inherited exit obligation must reject"),
            Err(error) => error,
        };
        assert!(matches!(
            deleted,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason == "open control obligation set changed after affine mint"
        ));

        let mut duplicated = oriented_five_control_invocation();
        let duplicate = duplicated.unwind.later_wrappers_in_construction_order[0].clone();
        duplicated
            .unwind
            .later_wrappers_in_construction_order
            .push(duplicate);
        let duplicated = match compose_oriented_subcontinuation(
            Some(&plan),
            None,
            ContinuationActivationId(8),
            duplicated,
            Vec::new(),
        ) {
            Ok(_) => panic!("duplicating an inherited exit obligation must reject"),
            Err(error) => error,
        };
        assert!(matches!(
            duplicated,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason == "open control obligation set changed after affine mint"
        ));
    }

    fn oriented_dynamic_sibling_fixture() -> (
        crate::OrientedSubcontinuationPlanV1,
        RecursorInvocationSegment,
        Vec<DynamicSpliceEdge>,
    ) {
        let plan = oriented_test_ih_plan();
        let origin = RecursorProducerOriginId(60);
        let mut segment = RecursorInvocationSegment::new(
            origin,
            0,
            oriented_test_instance_layer(
                2,
                11,
                1,
                true,
                RecursorLayerRole::SelectsOccurrence { origin },
            ),
            RecursorUnwindStack {
                later_wrappers_in_construction_order: vec![oriented_test_instance_layer(
                    0,
                    12,
                    1,
                    true,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(61),
                        parent_scope: None,
                    },
                )],
            },
            ContinuationCursorId(13),
            None,
            None,
        );
        segment.dynamic_splice_edges = vec![DynamicSpliceEdgeId(71), DynamicSpliceEdgeId(72)];
        let edges = vec![
            DynamicSpliceEdge {
                edge_id: DynamicSpliceEdgeId(71),
                child_invocation_instance_id: 11,
                parent_invocation_instance_id: 0,
                checked_call_template_id: 102,
                parent_frame_template_id: 2,
                segment_site_id: 9,
            },
            DynamicSpliceEdge {
                edge_id: DynamicSpliceEdgeId(72),
                child_invocation_instance_id: 12,
                parent_invocation_instance_id: 0,
                checked_call_template_id: 100,
                parent_frame_template_id: 0,
                segment_site_id: 9,
            },
        ];
        (plan, segment, edges)
    }

    #[test]
    fn oriented_same_depth_siblings_require_exact_dynamic_edges() {
        let (plan, segment, edges) = oriented_dynamic_sibling_fixture();

        let mut old_flat = std::iter::once(&segment.selection)
            .chain(segment.unwind.later_wrappers_in_construction_order.iter())
            .filter(|layer| layer.semantic_pending)
            .collect::<Vec<_>>();
        old_flat.sort_by_key(|layer| {
            (
                std::cmp::Reverse(layer.checked_invocation_depth),
                plan.frame(layer.checked_frame_id.unwrap())
                    .unwrap()
                    .semantic_position,
            )
        });
        let [left, right] = old_flat.as_slice() else {
            panic!("the discriminator must carry exactly two same-depth siblings")
        };
        assert_eq!(left.checked_invocation_depth, 1);
        assert_eq!(right.checked_invocation_depth, 1);
        let left = plan.frame(left.checked_frame_id.unwrap()).unwrap();
        let right = plan.frame(right.checked_frame_id.unwrap()).unwrap();
        assert_ne!(
            left.output_interface, right.input_interface,
            "the retired flat ordering must invent the non-composable sibling adjacency"
        );

        let installed = compose_oriented_subcontinuation(
            Some(&plan),
            None,
            ContinuationActivationId(14),
            segment,
            edges,
        )
        .expect("exact child-to-parent edges keep same-depth siblings separate");
        assert_eq!(
            installed
                .semantic_frames
                .iter()
                .map(|frame| (
                    frame.checked_invocation_id.unwrap(),
                    frame.checked_frame_id.unwrap(),
                ))
                .collect::<Vec<_>>(),
            vec![(11, 2), (12, 0)],
        );
    }

    #[test]
    fn oriented_dynamic_edge_mutations_reject_through_named_lanes() {
        let reject =
            |segment: RecursorInvocationSegment, edges: Vec<DynamicSpliceEdge>, expected: &str| {
                let plan = oriented_test_ih_plan();
                let error = match compose_oriented_subcontinuation(
                    Some(&plan),
                    None,
                    ContinuationActivationId(14),
                    segment,
                    edges,
                ) {
                    Ok(_) => panic!("a malformed dynamic splice graph must reject before CFG"),
                    Err(error) => error,
                };
                assert!(
                    matches!(
                        error,
                        CraneliftBackendError::Unsupported(UnsupportedLowering {
                            construct: "OrientedSubcontinuationPlanV1",
                            ref reason,
                        }) if reason.contains(expected)
                    ),
                    "expected {expected:?}, got {error:?}"
                );
            };

        let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
        edges.pop();
        reject(segment, edges, "deletion leaves an unparented");

        let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
        edges.push(DynamicSpliceEdge {
            edge_id: DynamicSpliceEdgeId(73),
            child_invocation_instance_id: 11,
            parent_invocation_instance_id: 0,
            checked_call_template_id: 102,
            parent_frame_template_id: 2,
            segment_site_id: 9,
        });
        reject(segment, edges, "duplicate affine splice edges");

        let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
        edges[0].parent_invocation_instance_id = 99;
        reject(segment, edges, "stale parent invocation");

        let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
        edges[0].parent_frame_template_id = 1;
        reject(segment, edges, "disagrees with its checked static parent");
    }

    #[test]
    fn oriented_dynamic_edge_ledger_is_affine_and_sibling_isolated() {
        let seed_env = NativeSeedEnvironment::empty();
        let mut lowering = root_authority_test_lowering(&seed_env);
        let (_, mut segment, mut edges) = oriented_dynamic_sibling_fixture();
        let edge = edges.remove(0);
        segment.dynamic_splice_edges = vec![edge.edge_id];
        lowering.dynamic_splice_edges.insert(edge.edge_id, edge);

        let consumed = lowering
            .take_dynamic_splice_edges(&segment)
            .expect("the owning invocation consumes its edge exactly once");
        assert_eq!(consumed.len(), 1);
        let stolen = match lowering.take_dynamic_splice_edges(&segment) {
            Ok(_) => panic!("a sibling cannot steal an already-consumed edge"),
            Err(error) => error,
        };
        assert!(matches!(
            stolen,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason.contains("consumed by a sibling")
        ));

        let (_, mut duplicated, mut edges) = oriented_dynamic_sibling_fixture();
        let edge = edges.remove(0);
        duplicated.dynamic_splice_edges = vec![edge.edge_id, edge.edge_id];
        lowering.dynamic_splice_edges.insert(edge.edge_id, edge);
        let duplicate = match lowering.take_dynamic_splice_edges(&duplicated) {
            Ok(_) => panic!("one carrier cannot duplicate an affine edge handle"),
            Err(error) => error,
        };
        assert!(matches!(
            duplicate,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason.contains("handle is duplicated")
        ));
    }

    #[derive(Clone, Copy, Debug)]
    enum Px8dsEdgeMutation {
        Delete,
        Duplicate,
        StaleParent,
        CrossSibling,
        WrongStaticParent,
    }

    fn run_px8ds_edge_consumer(
        consumer: Px8jDirectRecursorConsumer,
        mutation: Px8dsEdgeMutation,
    ) -> Result<Lowered, CraneliftBackendError> {
        let seed_env = NativeSeedEnvironment::empty();
        let mut compiler = root_authority_test_lowering(&seed_env);
        compiler.native_join_plan = None;
        compiler.root_terminal_authority = None;
        compiler.process_object = false;
        let (plan, mut segment, mut edges) = oriented_dynamic_sibling_fixture();
        compiler.oriented_subcontinuation_plan = Some(plan);

        match mutation {
            Px8dsEdgeMutation::Delete => {
                edges.remove(0);
            }
            Px8dsEdgeMutation::Duplicate => {
                segment
                    .dynamic_splice_edges
                    .push(segment.dynamic_splice_edges[0]);
            }
            Px8dsEdgeMutation::StaleParent => {
                edges[0].parent_invocation_instance_id = 99;
            }
            Px8dsEdgeMutation::CrossSibling => {
                let stolen = RecursorInvocationSegment {
                    dynamic_splice_edges: vec![segment.dynamic_splice_edges[0]],
                    ..segment.clone()
                };
                for edge in edges.drain(..) {
                    compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
                }
                compiler.take_dynamic_splice_edges(&stolen)?;
            }
            Px8dsEdgeMutation::WrongStaticParent => {
                edges[0].parent_frame_template_id = 1;
            }
        }
        for edge in edges {
            compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
        }

        let cursor = segment.resume_cursor;
        let activation = ContinuationActivationId(90);
        let recursor = Lowered::ComputationalRecursorClosure {
            residual: Box::new(Lowered::Closure {
                captures: Vec::new(),
                params: Vec::new(),
                body: RuntimeExpr::Construct {
                    constructor: "ctor:fixture::PX8DS::Done".to_string(),
                    args: Vec::new(),
                },
            }),
            activation,
            invocation: segment,
        };
        let active = ActiveContinuationFrame {
            activation,
            cursor,
            parent: None,
            pending: &[],
            selected_ancestry: &[],
            source_lineage: &[],
            source_selected_cursor: None,
            selected_scope: None,
        };
        let active_frames = [EliminatorFrame::Active(active)];
        let env = [recursor];
        let call = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Var(0)),
            args: Vec::new(),
        };
        let pending_let = RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            body: Box::new(RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(1)),
                args: Vec::new(),
            }),
        };

        let mut module = new_jit_module()?;
        let mut signature = module.make_signature();
        signature.returns.push(AbiParam::new(types::I64));
        let func_id = module
            .declare_function("px8ds_edge_consumer", Linkage::Local, &signature)
            .map_err(|error| backend_module(error.to_string()))?;
        let mut context = module.make_context();
        context.func =
            Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
        let mut function_context = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.switch_to_block(entry);
        match consumer {
            Px8jDirectRecursorConsumer::PendingLetProducer => compiler
                .lower_computational_producer_expr(
                    &mut builder,
                    &pending_let,
                    &env,
                    &active_frames,
                ),
            Px8jDirectRecursorConsumer::ProducerCall => compiler.lower_computational_producer_expr(
                &mut builder,
                &call,
                &env,
                &active_frames,
            ),
            Px8jDirectRecursorConsumer::OrdinaryCall => {
                compiler.lower_expr(&mut builder, &call, &env)
            }
        }
    }

    #[test]
    fn oriented_edge_mutations_reject_in_all_three_direct_consumers() {
        for consumer in [
            Px8jDirectRecursorConsumer::PendingLetProducer,
            Px8jDirectRecursorConsumer::ProducerCall,
            Px8jDirectRecursorConsumer::OrdinaryCall,
        ] {
            for (mutation, expected) in [
                (Px8dsEdgeMutation::Delete, "deleted, replayed"),
                (Px8dsEdgeMutation::Duplicate, "handle is duplicated"),
                (Px8dsEdgeMutation::StaleParent, "stale parent invocation"),
                (Px8dsEdgeMutation::CrossSibling, "consumed by a sibling"),
                (
                    Px8dsEdgeMutation::WrongStaticParent,
                    "disagrees with its checked static parent",
                ),
            ] {
                let error = match run_px8ds_edge_consumer(consumer, mutation) {
                    Ok(_) => panic!("{consumer:?}/{mutation:?} must reject before CFG"),
                    Err(error) => error,
                };
                assert!(
                    matches!(
                        error,
                        CraneliftBackendError::Unsupported(UnsupportedLowering {
                            construct: "OrientedSubcontinuationPlanV1",
                            ref reason,
                        }) if reason.contains(expected)
                    ),
                    "{consumer:?}/{mutation:?}: expected {expected:?}, got {error:?}"
                );
            }
        }
    }

    fn run_px8ds_source_consumer(mutation: Px8dsEdgeMutation) -> Result<(), CraneliftBackendError> {
        let seed_env = NativeSeedEnvironment::empty();
        let mut compiler = root_authority_test_lowering(&seed_env);
        compiler.native_join_plan = None;
        compiler.root_terminal_authority = None;
        compiler.process_object = false;
        let (plan, mut segment, mut edges) = oriented_dynamic_sibling_fixture();
        compiler.oriented_subcontinuation_plan = Some(plan);

        match mutation {
            Px8dsEdgeMutation::Delete => {
                edges.remove(0);
            }
            Px8dsEdgeMutation::Duplicate => {
                segment
                    .dynamic_splice_edges
                    .push(segment.dynamic_splice_edges[0]);
            }
            Px8dsEdgeMutation::StaleParent => {
                edges[0].parent_invocation_instance_id = 99;
            }
            Px8dsEdgeMutation::CrossSibling => {
                let stolen = RecursorInvocationSegment {
                    dynamic_splice_edges: vec![segment.dynamic_splice_edges[0]],
                    ..segment.clone()
                };
                for edge in edges.drain(..) {
                    compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
                }
                compiler.take_dynamic_splice_edges(&stolen)?;
            }
            Px8dsEdgeMutation::WrongStaticParent => {
                edges[0].parent_frame_template_id = 1;
            }
        }
        for edge in edges {
            compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
        }
        compiler
            .install_recursor_invocation(
                SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue),
                ContinuationActivationId(90),
                segment,
                None,
            )
            .map(|_| ())
    }

    #[test]
    fn oriented_edge_mutations_reject_in_the_source_machine_consumer() {
        for (mutation, expected) in [
            (Px8dsEdgeMutation::Delete, "deleted, replayed"),
            (Px8dsEdgeMutation::Duplicate, "handle is duplicated"),
            (Px8dsEdgeMutation::StaleParent, "stale parent invocation"),
            (Px8dsEdgeMutation::CrossSibling, "consumed by a sibling"),
            (
                Px8dsEdgeMutation::WrongStaticParent,
                "disagrees with its checked static parent",
            ),
        ] {
            let error = match run_px8ds_source_consumer(mutation) {
                Ok(()) => panic!("source {mutation:?} must reject before CFG"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error,
                    CraneliftBackendError::Unsupported(UnsupportedLowering {
                        construct: "OrientedSubcontinuationPlanV1",
                        ref reason,
                    }) if reason.contains(expected)
                ),
                "source {mutation:?}: expected {expected:?}, got {error:?}"
            );
        }
    }

    #[test]
    fn oriented_source_open_occurrence_cross_checks_the_closure_selected_parent() {
        let seed_env = NativeSeedEnvironment::empty();
        let mut compiler = root_authority_test_lowering(&seed_env);
        let (_, _, mut edges) = oriented_dynamic_sibling_fixture();
        let edge = edges.remove(0);
        let edge_id = edge.edge_id;
        compiler.dynamic_splice_edges.insert(edge_id, edge);
        let instance = CheckedRecursiveInvocationInstance {
            source: InvocationTemplateRef::ComputationalIHCall(102),
            invocation_instance_id: 11,
            semantic_depth: 1,
            dynamic_splice_edge: Some(edge_id),
        };
        let mut open = OwnedSelectedScope {
            scope_origin: RecursorProducerOriginId(70),
            parent_scope: None,
            frame: ComputationalRecursorFramePayload {
                cases: Vec::new(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: "PX8-DS source parent".to_string(),
                },
                outer_env: Vec::new(),
                provenance: RecursorFrameProvenance(71),
                checked_frame_id: Some(2),
                checked_invocation_id: Some(0),
                checked_invocation_source: None,
                checked_invocation_depth: 0,
            },
        };
        compiler
            .validate_source_dynamic_splice_parent(instance, &open)
            .expect("the source open occurrence agrees with closure selection");
        open.frame.checked_frame_id = Some(0);
        let mismatch = compiler
            .validate_source_dynamic_splice_parent(instance, &open)
            .expect_err("source and closure parent identities must agree before CFG");
        assert!(matches!(
            mismatch,
            CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
                if reason.contains("source open occurrence disagrees")
        ));
    }

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

    #[test]
    fn px8i_host_narrowing_rejects_negative_and_over_u64_before_dispatch() {
        let (negative, negative_fixture) =
            run_px8n_arm_fixture(PX8N_SHORT_WROTE, px8i_negative_narrow_fixture);
        assert_eq!(negative, 71);
        assert_eq!(negative_fixture.call_index, 0);

        let (oversize, oversize_fixture) =
            run_px8n_arm_fixture(PX8N_SHORT_WROTE, px8i_oversize_narrow_fixture);
        assert_eq!(oversize, 72);
        assert_eq!(oversize_fixture.call_index, 0);
    }

    #[test]
    fn px8i_positioned_start_and_metadata_promote_u64_above_i64_max() {
        let (read, read_fixture) =
            run_px8n_arm_fixture(PX8I_BIG_READ_START, px8i_big_read_start_fixture);
        assert_eq!(read_fixture.malformed_request, 0);
        assert_eq!(read_fixture.call_index, 3);
        assert_eq!(
            read, 13,
            "ReadAt keeps the narrowed start through validation"
        );

        let (write, write_fixture) =
            run_px8n_arm_fixture(PX8I_WRAPPING_WRITE_START, px8i_wrapping_write_start_fixture);
        assert_eq!(write_fixture.malformed_request, 0);
        assert_eq!(write_fixture.call_index, 3);
        assert_eq!(
            write, -1,
            "WriteAt validates progress against the narrowed start and rejects wrap"
        );

        let (metadata, metadata_fixture) =
            run_px8n_arm_fixture(PX8I_METADATA_BIG, px8i_metadata_big_fixture);
        assert_eq!(metadata_fixture.malformed_request, 0);
        assert_eq!(metadata_fixture.call_index, 2);
        assert_eq!(
            metadata, 14,
            "metadata detail is promoted to canonical Big rather than a negative Small"
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
        native_int_arena: *mut crate::NativeIntArenaV1,
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
    const PX8I_METADATA_BIG: u64 = 6;
    const PX8I_BIG_READ_START: u64 = 7;
    const PX8I_WRAPPING_WRITE_START: u64 = 8;
    const PX8I_BIG_U64: u64 = i64::MAX as u64 + 97;

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
        let expected = if fixture.call_index == 0
            || (fixture.call_index == 1 && fixture.scenario != PX8I_METADATA_BIG)
        {
            ken_host::HostOpV1::BufferAllocate
        } else if fixture.scenario == PX8I_METADATA_BIG {
            ken_host::HostOpV1::FsHandleMetadata
        } else if fixture.scenario == PX8I_WRAPPING_WRITE_START {
            ken_host::HostOpV1::FsWriteAt
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
        } else if expected == ken_host::HostOpV1::FsHandleMetadata {
            if load(wire.request_offsets[0]) != 11 {
                fixture.malformed_request = 5;
                return -1;
            }
        } else if [
            load(wire.request_offsets[0]),
            load(wire.request_offsets[1]),
            load(wire.request_offsets[2]),
            load(wire.request_offsets[3]),
            load(wire.request_offsets[4]),
        ] != [
            11,
            22,
            0,
            match fixture.scenario {
                PX8I_BIG_READ_START => PX8I_BIG_U64,
                PX8I_WRAPPING_WRITE_START => u64::MAX - 1,
                _ => 7,
            },
            4,
        ] {
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
        } else if expected == ken_host::HostOpV1::FsHandleMetadata {
            store(wire.reply_tag_offset, wire.reply_metadata_tag);
            store(wire.reply_detail_offset, PX8I_BIG_U64);
        } else {
            match fixture.scenario {
                PX8N_SHORT_WROTE | PX8I_WRAPPING_WRITE_START => {
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
                PX8I_BIG_READ_START => {
                    store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                    store(wire.reply_detail_offset, 1);
                    store(wire.reply_bytes_len_offset, PX8I_BIG_U64);
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
        let mismatch = RuntimeExpr::Value(RuntimeValue::Int((99).into()));
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

    fn px8i_invalid_allocate(
        symbols: &crate::NativeProcessSymbols,
        capacity: RuntimeExpr,
        code: i64,
    ) -> RuntimeExpr {
        RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "FS".to_string(),
                operation: ken_host::HostOpV1::BufferAllocate,
                capability: None,
                args: vec![capacity],
            }),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::Var(0)),
                        cases: vec![crate::RuntimeMatchCase {
                            constructor: symbols.resource_invalid_bounds.clone(),
                            binders: 0,
                            body: px8n_failure(
                                symbols,
                                RuntimeExpr::Value(RuntimeValue::Int(code.into())),
                            ),
                        }],
                        default: RuntimeTrap {
                            code: RuntimeTrapCode::PatternMatchFailure,
                            message: "PX8-I expected InvalidBounds".to_string(),
                        },
                    },
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(99.into()))),
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-I expected Result".to_string(),
            },
        }
    }

    fn px8i_negative_narrow_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        px8i_invalid_allocate(
            symbols,
            RuntimeExpr::Value(RuntimeValue::Int((-1).into())),
            71,
        )
    }

    fn px8i_oversize_narrow_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        px8i_invalid_allocate(symbols, big(crate::Sign::NonNegative, &[0, 1]), 72)
    }

    fn px8n_write_arm_fixture_with_start(
        symbols: &crate::NativeProcessSymbols,
        start: RuntimeExpr,
    ) -> RuntimeExpr {
        let trap = || RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N checked result default".to_string(),
        };
        let allocate = || RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
        };
        let write = RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::FsWriteAt,
            capability: None,
            args: vec![
                RuntimeExpr::Var(1),
                RuntimeExpr::Value(RuntimeValue::Int((0).into())),
                RuntimeExpr::Var(0),
                start,
                RuntimeExpr::Value(RuntimeValue::Int((4).into())),
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
                RuntimeExpr::Value(RuntimeValue::Int((3).into())),
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
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((70).into()))),
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
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((81).into()))),
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
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((80).into()))),
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

    fn px8n_write_arm_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        px8n_write_arm_fixture_with_start(
            symbols,
            RuntimeExpr::Value(RuntimeValue::Int((7).into())),
        )
    }

    fn px8i_wrapping_write_start_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        px8n_write_arm_fixture_with_start(symbols, big(crate::Sign::NonNegative, &[u64::MAX - 1]))
    }

    fn px8n_read_arm_fixture_with_start(
        symbols: &crate::NativeProcessSymbols,
        start: RuntimeExpr,
        observe_big_start: bool,
    ) -> RuntimeExpr {
        let trap = || RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N checked read result default".to_string(),
        };
        let allocate = || RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
        };
        let read = RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::FsReadAt,
            capability: None,
            args: vec![
                RuntimeExpr::Var(1),
                RuntimeExpr::Value(RuntimeValue::Int((0).into())),
                RuntimeExpr::Var(0),
                start,
                RuntimeExpr::Value(RuntimeValue::Int((4).into())),
            ],
        };
        let exact = if observe_big_start {
            RuntimeExpr::If {
                scrutinee: Box::new(total_primitive(
                    "eq_int",
                    vec![
                        RuntimeExpr::Var(1),
                        big(crate::Sign::NonNegative, &[PX8I_BIG_U64]),
                    ],
                )),
                then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((13).into()))),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((99).into()))),
            }
        } else {
            RuntimeExpr::Value(RuntimeValue::Int((12).into()))
        };
        let read_some = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: symbols.private_buffer_span.clone(),
                binders: 2,
                body: px8n_exact_nat(symbols, RuntimeExpr::Var(1), 1, exact),
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
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((10).into()))),
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
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((82).into()))),
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
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((81).into()))),
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
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((80).into()))),
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
        px8n_read_arm_fixture_with_start(
            symbols,
            RuntimeExpr::Value(RuntimeValue::Int((7).into())),
            false,
        )
    }

    fn px8i_big_read_start_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        px8n_read_arm_fixture_with_start(
            symbols,
            big(crate::Sign::NonNegative, &[PX8I_BIG_U64]),
            true,
        )
    }

    fn px8i_metadata_big_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
        let trap = || RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-I metadata result default".to_string(),
        };
        let metadata = RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::FsHandleMetadata,
            capability: None,
            args: vec![RuntimeExpr::Var(0)],
        };
        let observe = RuntimeExpr::Match {
            scrutinee: Box::new(metadata),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((98).into()))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: px8n_failure(
                        symbols,
                        RuntimeExpr::If {
                            scrutinee: Box::new(total_primitive(
                                "eq_int",
                                vec![
                                    RuntimeExpr::Var(0),
                                    big(crate::Sign::NonNegative, &[PX8I_BIG_U64]),
                                ],
                            )),
                            then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((14).into()))),
                            else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((99).into()))),
                        },
                    ),
                },
            ],
            default: trap(),
        };
        RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "FS".to_string(),
                operation: ken_host::HostOpV1::BufferAllocate,
                capability: None,
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
            }),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: symbols.result_err.clone(),
                    binders: 1,
                    body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((97).into()))),
                },
                crate::RuntimeMatchCase {
                    constructor: symbols.result_ok.clone(),
                    binders: 1,
                    body: observe,
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
            Some(test_only_distinguished_root_join_plan()),
            None,
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
        let mut native_int_arena = crate::NativeIntArenaV1::default();
        let invocation = NativeInvocationFixture {
            process_input: &input,
            host_context: (&mut fixture as *mut Px8nHostReplyFixture).cast(),
            capability: 0,
            native_int_arena: &mut native_int_arena,
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
                    vec![RuntimeExpr::Value(RuntimeValue::Int((9).into()))]
                },
            },
        }];
        if include_ok {
            producer_cases.push(RuntimeMatchCase {
                constructor: result_ok,
                binders: ok_binders,
                body: RuntimeExpr::Construct {
                    constructor: scalar_tree.clone(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
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
                    RuntimeExpr::Value(RuntimeValue::Int((payload).into()))
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
                value: Box::new(RuntimeExpr::Value(RuntimeValue::Int((41).into()))),
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
                            RuntimeExpr::Value(RuntimeValue::Int((7).into()))
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
                                RuntimeExpr::Value(RuntimeValue::Int((41).into())),
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
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int((payload).into()))],
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
                    RuntimeExpr::Value(RuntimeValue::Int((41).into())),
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

    fn px8j_deferred_recursive_field_fixture() -> RuntimeExpr {
        let wrap = "ctor:fixture::PX8JDeferred::Wrap";
        let done = "ctor:fixture::PX8JDeferred::Done";
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: wrap.to_string(),
                args: vec![
                    RuntimeExpr::LexicalClosure {
                        captures: Vec::new(),
                        params: vec!["unit".to_string()],
                        body: Box::new(RuntimeExpr::Construct {
                            constructor: done.to_string(),
                            args: Vec::new(),
                        }),
                    },
                    constructor_field_aggregate(),
                ],
            }),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: wrap.to_string(),
                    argument_binders: 2,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::Var(2)),
                        cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                            .into_iter()
                            .map(|constructor| RuntimeMatchCase {
                                constructor: constructor.to_string(),
                                binders: 1,
                                body: RuntimeExpr::Call {
                                    callee: Box::new(RuntimeExpr::Var(1)),
                                    args: vec![RuntimeExpr::Construct {
                                        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                        args: Vec::new(),
                                    }],
                                },
                            })
                            .collect(),
                        default: RuntimeTrap {
                            code: RuntimeTrapCode::PatternMatchFailure,
                            message: "PX8-J deferred selected-field default".to_string(),
                        },
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: done.to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Result::Ok".to_string(),
                        args: vec![RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                            args: Vec::new(),
                        }],
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-J deferred outer default".to_string(),
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
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
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
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((41).into())), field],
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
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
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
                    RuntimeExpr::Value(RuntimeValue::Int((13).into())),
                    aggregate,
                    RuntimeExpr::Value(RuntimeValue::Int((41).into())),
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
            RuntimeObservation::Returned(RuntimeGroundValue::Int((34).into()))
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
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((41).into()))],
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
                    RuntimeExpr::Value(RuntimeValue::Int((tag as i64 + 1).into()))
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
                            args: vec![RuntimeExpr::Value(RuntimeValue::Int((0).into()))],
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
                            .then(|| RuntimeExpr::Value(RuntimeValue::Int((1).into())))
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
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((0).into()))],
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
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((0).into()))],
        };

        emit_process_entrypoint_object_with_cranelift(
            &host_result_closure_match(effect_call),
            "ken_px7o_match_selected_call_returned_host_result",
        )
        .expect("match-selected HostResult remains owned by ordinary dynamic matching");
    }

    fn recursive_computational_result_depth(depth: usize, leaf_body: RuntimeExpr) -> RuntimeExpr {
        let node = "ctor:fixture::RecursiveTree::Node";
        let leaf = "ctor:fixture::RecursiveTree::Leaf";
        fn child(depth: usize, node: &str, leaf: &str) -> RuntimeExpr {
            RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["unit".to_string()],
                body: Box::new(if depth == 0 {
                    RuntimeExpr::Construct {
                        constructor: leaf.to_string(),
                        args: Vec::new(),
                    }
                } else {
                    RuntimeExpr::Construct {
                        constructor: node.to_string(),
                        args: vec![child(depth - 1, node, leaf)],
                    }
                }),
            }
        }
        let recursive_child = child(depth, node, leaf);
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

    fn recursive_computational_result(leaf_body: RuntimeExpr) -> RuntimeExpr {
        recursive_computational_result_depth(0, leaf_body)
    }

    fn px8j_recursive_sibling_result(
        depth: usize,
        siblings: usize,
        leaf_body: RuntimeExpr,
    ) -> RuntimeExpr {
        assert!(siblings > 0);
        let node = "ctor:fixture::PX8JSiblingTree::Node";
        let leaf = "ctor:fixture::PX8JSiblingTree::Leaf";
        fn child(depth: usize, siblings: usize, node: &str, leaf: &str) -> RuntimeExpr {
            RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["unit".to_string()],
                body: Box::new(if depth == 0 {
                    RuntimeExpr::Construct {
                        constructor: leaf.to_string(),
                        args: Vec::new(),
                    }
                } else {
                    RuntimeExpr::Construct {
                        constructor: node.to_string(),
                        args: (0..siblings)
                            .map(|_| child(depth - 1, siblings, node, leaf))
                            .collect(),
                    }
                }),
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: node.to_string(),
                args: (0..siblings)
                    .map(|_| child(depth, siblings, node, leaf))
                    .collect(),
            }),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: node.to_string(),
                    argument_binders: siblings,
                    recursive_positions: (0..siblings).collect(),
                    body: if siblings == 1 {
                        RuntimeExpr::Call {
                            callee: Box::new(RuntimeExpr::Var(0)),
                            args: vec![RuntimeExpr::Construct {
                                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                args: Vec::new(),
                            }],
                        }
                    } else {
                        RuntimeExpr::Let {
                            value: Box::new(RuntimeExpr::Call {
                                callee: Box::new(RuntimeExpr::Var(0)),
                                args: vec![RuntimeExpr::Construct {
                                    constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                    args: Vec::new(),
                                }],
                            }),
                            body: Box::new(RuntimeExpr::Call {
                                callee: Box::new(RuntimeExpr::Var(2)),
                                args: vec![RuntimeExpr::Construct {
                                    constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                    args: Vec::new(),
                                }],
                            }),
                        }
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
                message: "PX8-J sibling tree default".to_string(),
            },
        }
    }

    fn px8j_layered_recursive_result(transform_layers: usize, input_depth: usize) -> RuntimeExpr {
        let tree_constructor = |layer: usize, constructor: &str| {
            format!("ctor:fixture::PX8JTree{layer}::{constructor}")
        };
        let unit = || RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        };
        let aggregate = || RuntimeExpr::Construct {
            constructor: "ctor:prelude::Result::Ok".to_string(),
            args: vec![unit()],
        };
        fn child(depth: usize, node: &str, leaf: &str) -> RuntimeExpr {
            RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["unit".to_string()],
                body: Box::new(if depth == 0 {
                    RuntimeExpr::Construct {
                        constructor: leaf.to_string(),
                        args: Vec::new(),
                    }
                } else {
                    RuntimeExpr::Construct {
                        constructor: node.to_string(),
                        args: vec![child(depth - 1, node, leaf)],
                    }
                }),
            }
        }
        let input_node = tree_constructor(0, "Node");
        let input_leaf = tree_constructor(0, "Leaf");
        let mut producer = RuntimeExpr::Construct {
            constructor: input_node.clone(),
            args: vec![child(input_depth, &input_node, &input_leaf)],
        };
        for layer in 0..transform_layers {
            producer = RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(producer),
                cases: vec![
                    crate::RuntimeComputationalMatchCase {
                        constructor: tree_constructor(layer, "Node"),
                        argument_binders: 1,
                        recursive_positions: vec![0],
                        body: RuntimeExpr::Construct {
                            constructor: tree_constructor(layer + 1, "Node"),
                            args: vec![RuntimeExpr::Var(0)],
                        },
                    },
                    crate::RuntimeComputationalMatchCase {
                        constructor: tree_constructor(layer, "Leaf"),
                        argument_binders: 0,
                        recursive_positions: Vec::new(),
                        body: RuntimeExpr::Construct {
                            constructor: tree_constructor(layer + 1, "Leaf"),
                            args: Vec::new(),
                        },
                    },
                ],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: format!("PX8-J transform {layer} default"),
                },
            };
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(producer),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(transform_layers, "Node"),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![unit()],
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(transform_layers, "Leaf"),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: aggregate(),
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-J terminal transform default".to_string(),
            },
        }
    }

    fn px8j_scope_chain_observation_result(
        transform_layers: usize,
        input_depth: usize,
    ) -> RuntimeExpr {
        let tree_constructor = |_layer: usize, constructor: &str| {
            format!("ctor:fixture::PX8JScopeTree::{constructor}")
        };
        fn child(depth: usize, node: &str, leaf: &str) -> RuntimeExpr {
            RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["unit".to_string()],
                body: Box::new(if depth == 0 {
                    RuntimeExpr::Construct {
                        constructor: leaf.to_string(),
                        args: Vec::new(),
                    }
                } else {
                    RuntimeExpr::Construct {
                        constructor: node.to_string(),
                        args: vec![child(depth - 1, node, leaf)],
                    }
                }),
            }
        }
        let input_node = tree_constructor(0, "Node");
        let input_leaf = tree_constructor(0, "Leaf");
        let mut producer = RuntimeExpr::Construct {
            constructor: input_node.clone(),
            args: vec![child(input_depth, &input_node, &input_leaf)],
        };
        for layer in 0..transform_layers {
            producer = RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(producer),
                cases: vec![
                    crate::RuntimeComputationalMatchCase {
                        constructor: tree_constructor(layer, "Node"),
                        argument_binders: 1,
                        recursive_positions: vec![0],
                        body: RuntimeExpr::Construct {
                            constructor: tree_constructor(layer + 1, "Node"),
                            args: vec![RuntimeExpr::Var(0)],
                        },
                    },
                    crate::RuntimeComputationalMatchCase {
                        constructor: tree_constructor(layer, "Leaf"),
                        argument_binders: 0,
                        recursive_positions: Vec::new(),
                        body: RuntimeExpr::Construct {
                            constructor: tree_constructor(layer + 1, "Leaf"),
                            args: Vec::new(),
                        },
                    },
                ],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: format!("PX8-J transform {layer} default"),
                },
            };
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(producer),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(transform_layers, "Node"),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Let {
                        value: Box::new(RuntimeExpr::Call {
                            callee: Box::new(RuntimeExpr::Var(0)),
                            args: vec![RuntimeExpr::Construct {
                                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                args: Vec::new(),
                            }],
                        }),
                        body: Box::new(RuntimeExpr::Construct {
                            constructor: tree_constructor(transform_layers, "Node"),
                            args: vec![child(
                                0,
                                &tree_constructor(transform_layers, "Node"),
                                &tree_constructor(transform_layers, "Leaf"),
                            )],
                        }),
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(transform_layers, "Leaf"),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: tree_constructor(transform_layers, "Leaf"),
                        args: Vec::new(),
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-J terminal transform default".to_string(),
            },
        }
    }

    #[derive(Clone, Copy)]
    enum Px8jSelectedScopePlacement {
        BeforeReturnHole,
        AfterReturnHole,
    }

    fn px8j_equal_payload_hole_placement(placement: Px8jSelectedScopePlacement) -> RuntimeExpr {
        let input_node = "ctor:fixture::PX8JHoleInput::Node";
        let input_leaf = "ctor:fixture::PX8JHoleInput::Leaf";
        let output_node = "ctor:fixture::PX8JHoleOutput::Node";
        let output_leaf = "ctor:fixture::PX8JHoleOutput::Leaf";
        let unit = || RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        };
        let recursive_child = || RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["unit".to_string()],
            body: Box::new(RuntimeExpr::Construct {
                constructor: input_leaf.to_string(),
                args: Vec::new(),
            }),
        };
        let scoped_payload = || RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: input_node.to_string(),
                args: vec![recursive_child()],
            }),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: input_node.to_string(),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Construct {
                        constructor: output_node.to_string(),
                        args: vec![RuntimeExpr::Var(0)],
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: input_leaf.to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: output_leaf.to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-J equal-payload inner default".to_string(),
            },
        };
        let outer_scrutinee = match placement {
            Px8jSelectedScopePlacement::BeforeReturnHole => RuntimeExpr::Construct {
                constructor: output_node.to_string(),
                args: vec![RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["unit".to_string()],
                    body: Box::new(scoped_payload()),
                }],
            },
            Px8jSelectedScopePlacement::AfterReturnHole => scoped_payload(),
        };
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(outer_scrutinee),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: output_node.to_string(),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![unit()],
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: output_leaf.to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: px8j_aggregate_result(),
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "PX8-J equal-payload outer default".to_string(),
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

    fn px8j_capture_source_trace(
        expression: &RuntimeExpr,
        delete_owned_scope: bool,
        symbol: &str,
    ) -> (
        Result<CraneliftObjectArtifact, CraneliftBackendError>,
        Vec<Px8jSourceTraceEvent>,
    ) {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                PX8J_DELETE_OWNED_SELECTED_SCOPE.set(false);
                PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().clear());
            }
        }
        PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().clear());
        PX8J_DELETE_OWNED_SELECTED_SCOPE.set(delete_owned_scope);
        let _reset = Reset;
        let result = emit_process_entrypoint_object_with_cranelift(expression, symbol);
        let trace = PX8J_SOURCE_TRACE.with(|trace| trace.borrow().clone());
        (result, trace)
    }

    fn px8j_aggregate_result() -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: "ctor:prelude::Result::Ok".to_string(),
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                args: Vec::new(),
            }],
        }
    }

    #[test]
    fn px8j_all_three_producer_paths_reach_real_consumers() {
        let aggregate = RuntimeExpr::Construct {
            constructor: "ctor:prelude::Result::Ok".to_string(),
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                args: Vec::new(),
            }],
        };
        let expression =
            host_result_closure_match(recursive_computational_result_depth(2, aggregate));
        let (result, trace) =
            px8j_capture_source_trace(&expression, false, "ken_px8j_live_source_paths");
        result.expect("the composed and source-machine producer paths lower");
        for path in [Px8jProducerPath::Composed, Px8jProducerPath::SourceMachine] {
            let (origin, cursor) = trace
                .iter()
                .find_map(|event| match event {
                    Px8jSourceTraceEvent::Mint {
                        path: actual,
                        origin,
                        cursor,
                        siblings,
                        ..
                    } if *actual == path && *siblings > 0 => Some((*origin, *cursor)),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{path:?} must mint a recursive IH"));
            assert!(trace.iter().any(|event| matches!(
                event,
                Px8jSourceTraceEvent::Install {
                    origin: actual_origin,
                    selection_cursor,
                    ..
                } if *actual_origin == origin && *selection_cursor == cursor
            )));
            assert!(trace.iter().any(|event| matches!(
                event,
                Px8jSourceTraceEvent::Selection { origin: actual } if *actual == origin
            )));
        }

        let deferred = host_result_closure_match(px8j_deferred_recursive_field_fixture());
        let (result, trace) =
            px8j_capture_source_trace(&deferred, false, "ken_px8j_live_deferred_path");
        result.expect("the deferred-constructor producer path lowers");
        let (origin, cursor) = trace
            .iter()
            .find_map(|event| match event {
                Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::DeferredConstructor,
                    origin,
                    cursor,
                    siblings: 1,
                    ..
                } => Some((*origin, *cursor)),
                _ => None,
            })
            .expect("the deferred constructor mints its recursive IH");
        assert!(trace.iter().any(|event| matches!(
            event,
            Px8jSourceTraceEvent::DirectConsume {
                origin: actual_origin,
                selection_cursor,
                ..
            } if *actual_origin == origin && *selection_cursor == cursor
        )));
    }

    #[test]
    fn px8j_selected_scope_partitions_differ_across_the_real_return_hole() {
        let before = host_result_closure_match(px8j_equal_payload_hole_placement(
            Px8jSelectedScopePlacement::BeforeReturnHole,
        ));
        let after = host_result_closure_match(px8j_equal_payload_hole_placement(
            Px8jSelectedScopePlacement::AfterReturnHole,
        ));
        let (before_result, before_trace) =
            px8j_capture_source_trace(&before, false, "ken_px8j_scope_before_hole");
        let (after_result, after_trace) =
            px8j_capture_source_trace(&after, false, "ken_px8j_scope_after_hole");
        before_result.expect("the before-hole selected scope lowers");
        after_result.expect("the after-hole selected scope lowers");

        let partition = |trace: &[Px8jSourceTraceEvent]| {
            let hole = trace
                .iter()
                .position(|event| matches!(event, Px8jSourceTraceEvent::ReturnHole { .. }))
                .expect("the real source path reaches its return hole");
            let selections_before = trace[..hole]
                .iter()
                .filter(|event| matches!(event, Px8jSourceTraceEvent::Selection { .. }))
                .count();
            let exits_after = trace[hole + 1..]
                .iter()
                .filter(|event| matches!(event, Px8jSourceTraceEvent::Exit { .. }))
                .count();
            (selections_before, exits_after)
        };
        assert_eq!(partition(&before_trace), (2, 0));
        assert_eq!(partition(&after_trace), (1, 1));
    }

    #[test]
    fn px8j_one_two_three_scope_segments_reach_selection_hole_and_unwind() {
        for depth in 1..=3 {
            let expression =
                host_result_closure_match(px8j_scope_chain_observation_result(depth, 0));
            let (result, trace) = px8j_capture_source_trace(
                &expression,
                false,
                &format!("ken_px8j_live_scope_depth_{depth}"),
            );
            result.unwrap_or_else(|error| panic!("scope depth {depth} must lower: {error:?}"));
            let (origin, cursor, exits) = trace
                .iter()
                .find_map(|event| match event {
                    Px8jSourceTraceEvent::Install {
                        origin,
                        selection_cursor,
                        exits,
                        ..
                    } if exits.len() == depth => Some((*origin, *selection_cursor, exits)),
                    _ => None,
                })
                .unwrap_or_else(|| {
                    panic!("scope depth {depth} must install one exact segment: {trace:#?}")
                });
            let unique_scope_origins: BTreeSet<_> = exits
                .iter()
                .map(|(scope_origin, _)| *scope_origin)
                .collect();
            assert_eq!(unique_scope_origins.len(), depth);
            assert_eq!(exits.first().and_then(|(_, parent)| *parent), None);
            for pair in exits.windows(2) {
                let (outer_scope, _) = pair[0];
                let (_, inner_parent) = pair[1];
                assert_eq!(inner_parent, Some(outer_scope));
            }
            let selection = trace
                .iter()
                .position(|event| {
                    matches!(
                        event,
                        Px8jSourceTraceEvent::Selection { origin: actual } if *actual == origin
                    )
                })
                .expect("selection is consumed");
            let hole = trace
                .iter()
                .position(|event| {
                    matches!(
                        event,
                        Px8jSourceTraceEvent::ReturnHole { cursor: actual } if *actual == cursor
                    )
                })
                .expect("the complete caller source K reaches its return hole");
            let first_exit = trace
                .iter()
                .position(|event| {
                    matches!(
                        event,
                        Px8jSourceTraceEvent::Exit { origin: actual, .. } if *actual == origin
                    )
                })
                .expect("the installed unwind stack begins consumption");
            assert!(selection < hole && hole < first_exit);
            let consumed_exits: Vec<_> = trace[hole + 1..]
                .iter()
                .filter_map(|event| match event {
                    Px8jSourceTraceEvent::Exit {
                        origin: actual_origin,
                        scope_origin,
                        parent_scope,
                    } if *actual_origin == origin => Some((*scope_origin, *parent_scope)),
                    _ => None,
                })
                .collect();
            assert_eq!(
                consumed_exits,
                exits.iter().rev().copied().collect::<Vec<_>>(),
                "depth {depth}: {trace:#?}"
            );
        }
    }

    #[test]
    fn px8j_siblings_share_an_origin_and_nested_ih_gets_a_child_origin() {
        let expression =
            host_result_closure_match(px8j_recursive_sibling_result(1, 2, px8j_aggregate_result()));
        let (result, trace) =
            px8j_capture_source_trace(&expression, false, "ken_px8j_live_sibling_origins");
        result.expect("the sibling and nested recursive IH path lowers");
        let (sibling_origin, sibling_cursor) = trace
            .iter()
            .find_map(|event| match event {
                Px8jSourceTraceEvent::Mint {
                    origin,
                    cursor,
                    siblings: 2,
                    ..
                } => Some((*origin, *cursor)),
                _ => None,
            })
            .expect("the selected case owns the sibling IH origin");
        let sibling_carriers: BTreeSet<_> = trace
            .iter()
            .filter_map(|event| match event {
                Px8jSourceTraceEvent::Carrier {
                    origin,
                    cursor,
                    sibling_position,
                    ..
                } if *origin == sibling_origin && *cursor == sibling_cursor => {
                    Some(*sibling_position)
                }
                _ => None,
            })
            .collect();
        assert_eq!(sibling_carriers, BTreeSet::from([0, 1]));
        let sibling_consumers: BTreeSet<_> = trace
            .iter()
            .filter_map(|event| match event {
                Px8jSourceTraceEvent::Install {
                    origin,
                    selection_cursor,
                    sibling_position,
                    ..
                } if *origin == sibling_origin && *selection_cursor == sibling_cursor => {
                    Some(*sibling_position)
                }
                _ => None,
            })
            .collect();
        assert_eq!(sibling_consumers, sibling_carriers);
        assert!(
            trace.iter().any(|event| matches!(
                event,
                Px8jSourceTraceEvent::Mint {
                    origin,
                    parent_scope: Some(parent),
                    ..
                } if *origin != sibling_origin && *parent == sibling_origin
            )),
            "{trace:#?}"
        );
    }

    #[test]
    fn px8j_owned_scope_deletion_fails_closed_before_another_frame_is_emitted() {
        let expression = host_result_closure_match(px8j_layered_recursive_result(1, 1));
        let (exact_result, exact_trace) =
            px8j_capture_source_trace(&expression, false, "ken_px8j_scope_exact");
        exact_result.expect("the exact owned-scope path lowers");
        let (deleted_result, deleted_trace) =
            px8j_capture_source_trace(&expression, true, "ken_px8j_scope_deleted");
        let error = deleted_result.expect_err("deleting the owned scope must fail closed");
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalRecursor",
                ref reason,
            }) if reason == "source recursor invocation is missing its owned selected scope"
        ));
        let deleted_terminal = deleted_trace
            .last()
            .expect("deletion must leave its terminal mint observation");
        let exact_terminal_index = exact_trace
            .iter()
            .position(|event| match (event, deleted_terminal) {
                (
                    Px8jSourceTraceEvent::Mint {
                        path: exact_path,
                        origin: exact_origin,
                        cursor: exact_cursor,
                        siblings: exact_siblings,
                        ..
                    },
                    Px8jSourceTraceEvent::Mint {
                        path: deleted_path,
                        origin: deleted_origin,
                        cursor: deleted_cursor,
                        siblings: deleted_siblings,
                        ..
                    },
                ) => {
                    exact_path == deleted_path
                        && exact_origin == deleted_origin
                        && exact_cursor == deleted_cursor
                        && exact_siblings == deleted_siblings
                }
                _ => false,
            })
            .expect("the exact run reaches the deleted run's terminal mint");
        assert_eq!(
            &deleted_trace[..deleted_trace.len() - 1],
            &exact_trace[..exact_terminal_index]
        );
        assert!(matches!(
            (exact_trace.get(exact_terminal_index), deleted_trace.last()),
            (
                Some(Px8jSourceTraceEvent::Mint {
                    path: exact_path,
                    origin: exact_origin,
                    cursor: exact_cursor,
                    siblings: exact_siblings,
                    parent_scope: Some(_),
                }),
                Some(Px8jSourceTraceEvent::Mint {
                    path: deleted_path,
                    origin: deleted_origin,
                    cursor: deleted_cursor,
                    siblings: deleted_siblings,
                    parent_scope: None,
                }),
            ) if exact_path == deleted_path
                && exact_origin == deleted_origin
                && exact_cursor == deleted_cursor
                && exact_siblings == deleted_siblings
        ));
        let deleted_origin = match deleted_trace.last() {
            Some(Px8jSourceTraceEvent::Mint { origin, .. }) => *origin,
            event => panic!("deletion must stop immediately after the nested mint: {event:?}"),
        };
        assert!(!deleted_trace.iter().any(|event| matches!(
            event,
            Px8jSourceTraceEvent::Install { origin, .. }
                if *origin == deleted_origin
        )));
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
                captures: vec![RuntimeExpr::Value(RuntimeValue::Int((41).into()))],
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
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
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
            RuntimeObservation::Returned(RuntimeGroundValue::Int((34).into()))
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
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
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
            body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
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
            body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
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
            body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
        }];
        let outer_cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Outer::Hit".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Value(RuntimeValue::Int((2).into())),
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
                checked_frame_id: None,
                checked_invocation_id: None,
                checked_invocation_source: None,
                checked_invocation_depth: 0,
            },
            ComputationalEliminatorFrame {
                cases: &outer_cases,
                default: &outer_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: RecursorFrameProvenance(0),
                checked_frame_id: None,
                checked_invocation_id: None,
                checked_invocation_source: None,
                checked_invocation_depth: 0,
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
            body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
        }];
        let outer_cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Outer::Hit".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Value(RuntimeValue::Int((2).into())),
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
                checked_frame_id: None,
                checked_invocation_id: None,
                checked_invocation_source: None,
                checked_invocation_depth: 0,
            },
            ComputationalEliminatorFrame {
                cases: &outer_cases,
                default: &outer_default,
                env: &[],
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: RecursorFrameProvenance(0),
                checked_frame_id: None,
                checked_invocation_id: None,
                checked_invocation_source: None,
                checked_invocation_depth: 0,
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
            Some(test_only_distinguished_root_join_plan()),
            None,
        )
        .expect("borrowed fixture lowers");
        let mut native_int_arena = crate::NativeIntArenaV1::default();
        let invocation = NativeInvocationFixture {
            process_input: root,
            host_context: std::ptr::null_mut(),
            capability: 1_u64 << 32,
            native_int_arena: &mut native_int_arena,
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
                body: RuntimeExpr::Value(RuntimeValue::Int((0).into())),
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
                                RuntimeExpr::Value(RuntimeValue::Int((99).into())),
                            ],
                        }),
                        cases: vec![
                            RuntimeMatchCase {
                                constructor: none.to_string(),
                                binders: 0,
                                body: RuntimeExpr::Value(RuntimeValue::Int((7).into())),
                            },
                            RuntimeMatchCase {
                                constructor: some.to_string(),
                                binders: 1,
                                body: RuntimeExpr::Value(RuntimeValue::Int((9).into())),
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
                            args: vec![RuntimeExpr::Value(RuntimeValue::Int((1).into()))],
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
            Some(test_only_distinguished_root_join_plan()),
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
    fn checked_join_marker_without_exact_plan_site_rejects_before_emission() {
        let expression = RuntimeExpr::CheckedJoinSite {
            site_id: 41,
            body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((7).into()))),
        };
        let result = compile_expr_into_module(
            new_object_module("px8h-missing-join-site").unwrap(),
            "ken_px8h_missing_join_site",
            Linkage::Export,
            &expression,
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            false,
            None,
            None,
            None,
        );
        let error = match result {
            Ok(_) => panic!("a live checked occurrence without its plan site must reject"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                reason,
            }) if reason.contains("marker was not consumed")
        ));
    }

    fn self_consistent_join_site(
        site_id: u64,
        runtime_frame_fingerprint: u64,
    ) -> crate::NativeJoinPlanSiteV1 {
        let declaration = "decl:fixture::PX8H::main".to_string();
        let checked_occurrence_path = vec![1, site_id];
        let checked_result_type_fingerprint = 17;
        crate::NativeJoinPlanSiteV1 {
            site_id,
            occurrence_binding_fingerprint:
                crate::compiler_private_join_occurrence_binding_fingerprint(
                    site_id,
                    &declaration,
                    &checked_occurrence_path,
                    checked_result_type_fingerprint,
                ),
            declaration,
            checked_occurrence_path,
            checked_result_type_fingerprint,
            runtime_frame_fingerprint,
            answer_kind: crate::NativeJoinAnswerKindV1::Int,
        }
    }

    fn self_consistent_root_join_site(site_id: u64) -> crate::NativeJoinPlanSiteV1 {
        let declaration = "decl:fixture::PX8H::main".to_string();
        let checked_occurrence_path = vec![0];
        let checked_result_type_fingerprint = 19;
        crate::NativeJoinPlanSiteV1 {
            site_id,
            occurrence_binding_fingerprint:
                crate::compiler_private_join_occurrence_binding_fingerprint(
                    site_id,
                    &declaration,
                    &checked_occurrence_path,
                    checked_result_type_fingerprint,
                ),
            declaration,
            checked_occurrence_path,
            checked_result_type_fingerprint,
            runtime_frame_fingerprint: crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
            answer_kind: crate::NativeJoinAnswerKindV1::ExitCode,
        }
    }

    fn root_authority_test_lowering<'a>(seed_env: &'a NativeSeedEnvironment) -> Lowering<'a> {
        Lowering {
            seed_env,
            declarations: BTreeMap::new(),
            declaration_stack: Vec::new(),
            active_recursive_declarations: Vec::new(),
            result_table: BTreeMap::new(),
            next_token: 0,
            next_recursor_frame_provenance: 0,
            next_recursor_producer_origin: 0,
            next_continuation_activation: 0,
            next_continuation_cursor: 0,
            next_source_join: 0,
            next_source_predecessor: 0,
            live_source_continuations: 0,
            source_control_root: None,
            active_oriented_semantic_regions: 0,
            native_join_plan: Some(crate::NativeJoinPlanV1 {
                representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
                sites: vec![self_consistent_root_join_site(0)],
            }),
            consumed_join_sites: BTreeSet::new(),
            root_terminal_authority: None,
            active_join_site: None,
            oriented_subcontinuation_plan: None,
            consumed_subcontinuation_frames: BTreeSet::new(),
            active_subcontinuation_frame: None,
            consumed_recursive_call_templates: BTreeSet::new(),
            pending_recursive_call: None,
            pending_computational_ih_call: None,
            active_recursive_invocations: Vec::new(),
            next_recursive_invocation_instance: 1,
            dynamic_splice_edges: BTreeMap::new(),
            next_dynamic_splice_edge: 1,
            assumptions: BTreeSet::new(),
            unsupported: Vec::new(),
            process_object: true,
            process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
            host_dispatch: None,
            invocation_pointer: None,
            native_int_arena: None,
            native_int_binop: None,
            native_int_compare: None,
            native_int_intern: None,
            native_int_narrow: None,
            native_int_export: None,
            native_int_tags: BTreeMap::new(),
            native_int_mutation: NativeIntLoweringMutation::Exact,
            bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
        }
    }

    #[test]
    fn distinguished_root_authority_is_checked_affine_and_cursor_bound() {
        let seed_env = NativeSeedEnvironment::empty();
        let mut lowering = root_authority_test_lowering(&seed_env);
        let authority = lowering
            .take_distinguished_root_answer_authority()
            .expect("the exact checked root site validates")
            .expect("process lowering carries root authority");
        lowering.root_terminal_authority = Some(authority);
        lowering
            .mint_terminal_answer_authority()
            .expect("the first exhausted-root mint consumes the authority");
        let repeated = match lowering.mint_terminal_answer_authority() {
            Ok(_) => panic!("the affine root authority cannot mint twice"),
            Err(error) => error,
        };
        assert!(matches!(
            repeated,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                reason,
            }) if reason == "terminal answer has no affine checked-root authority"
        ));

        let mut lowering = root_authority_test_lowering(&seed_env);
        let mut authority = lowering
            .take_distinguished_root_answer_authority()
            .unwrap()
            .unwrap();
        authority.outer_cursor = Some(ContinuationCursorId(7));
        let transplanted = lowering
            .restore_root_terminal_authority(Some(authority), ContinuationCursorId(8))
            .expect_err("a root token cannot cross the wrong source cursor");
        assert!(matches!(
            transplanted,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                reason,
            }) if reason == "checked root answer authority returned through the wrong outer cursor"
        ));
    }

    #[test]
    fn process_lowering_without_checked_root_authority_rejects_before_cfg() {
        let result = compile_expr_into_module(
            new_object_module("px8ta-missing-root-authority").unwrap(),
            "ken_px8ta_missing_root_authority",
            Linkage::Export,
            &RuntimeExpr::Construct {
                constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                args: Vec::new(),
            },
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            true,
            None,
            None,
            None,
        );
        let error = match result {
            Ok(_) => panic!("process lowering must not invent root authority from process mode"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                reason,
            }) if reason == "process-object lowering has no checked distinguished-root answer authority"
        ));
    }

    fn occurrence_exact_marker_fixture(
        duplicate_call: bool,
        duplicate_slot: bool,
    ) -> (
        RuntimeExpr,
        RuntimeDeclaration,
        crate::OrientedSubcontinuationPlanV1,
    ) {
        let declaration = "decl:fixture::PX8TA::markers".to_string();
        let slot_marker = RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids: vec![200],
            checked_occurrence_paths: vec![vec![20]],
            body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
        };
        let call_marker = RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id: 100,
            checked_occurrence_path: vec![30],
            body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((2).into()))),
        };
        let slot_value = if duplicate_slot {
            RuntimeExpr::Construct {
                constructor: "ctor:fixture::Pair".to_string(),
                args: vec![slot_marker.clone(), slot_marker],
            }
        } else {
            slot_marker
        };
        let call_body = if duplicate_call {
            RuntimeExpr::Construct {
                constructor: "ctor:fixture::Pair".to_string(),
                args: vec![call_marker.clone(), call_marker],
            }
        } else {
            call_marker
        };
        let cases = vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Only".to_string(),
            argument_binders: 0,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Let {
                value: Box::new(slot_value),
                body: Box::new(call_body),
            },
        }];
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-TA marker fixture default".to_string(),
        };
        let runtime_frame_fingerprint =
            crate::compiler_private_computational_match_frame_fingerprint(&cases, &default);
        let body = RuntimeExpr::CheckedSubcontinuationFrame {
            frame_id: 0,
            body: Box::new(RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::Only".to_string(),
                    args: Vec::new(),
                }),
                cases,
                default,
            }),
        };
        let runtime_declaration = RuntimeDeclaration {
            symbol: declaration.clone(),
            kind: RuntimeDeclarationKind::Transparent { body },
            metadata: RuntimeSymbolMetadata::empty(),
        };
        let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
            frame_id: 0,
            segment_site_id: 9,
            declaration: declaration.clone(),
            checked_occurrence_path: vec![10],
            semantic_position: 0,
            input_interface: oriented_test_interface(0),
            output_interface: oriented_test_interface(1),
            runtime_frame_fingerprint,
            occurrence_binding_fingerprint: 0,
            control_witness: crate::OrientedControlWitnessV1::DistinguishedRoot,
        };
        frame.occurrence_binding_fingerprint =
            crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
        let mut slot = crate::CheckedComputationalIHSlotTemplateV1 {
            slot_template_id: 200,
            declaration: declaration.clone(),
            checked_match_ordinal: 0,
            checked_occurrence_path: vec![20],
            frame_template_id: 0,
            constructor: "ctor:fixture::Only".to_string(),
            recursive_position: 0,
            method_binder_ordinal: 0,
            local_telescope: Vec::new(),
            ih_interface: oriented_test_interface(0),
            segment_site_id: 9,
            frame_templates: vec![0],
            input_interface: oriented_test_interface(0),
            output_interface: oriented_test_interface(1),
            runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
                declaration: declaration.clone(),
                runtime_path: vec![0, 1, 0],
            }],
            occurrence_binding_fingerprint: 0,
        };
        slot.occurrence_binding_fingerprint =
            crate::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
        let mut call = crate::CheckedComputationalIHCallTemplateV1 {
            call_template_id: 100,
            declaration: declaration.clone(),
            checked_occurrence_path: vec![30],
            slot_template_id: 200,
            arity: 1,
            local_telescope: Vec::new(),
            result_interface: oriented_test_interface(1),
            callee_segment_site_id: 9,
            callee_frame_templates: vec![0],
            parent_frame_template_id: Some(0),
            parent_segment_site_id: Some(9),
            caller_interface: oriented_test_interface(1),
            runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
                declaration,
                runtime_path: vec![0, 1, 1],
            }],
            occurrence_binding_fingerprint: 0,
        };
        call.occurrence_binding_fingerprint =
            crate::compiler_private_computational_ih_call_binding_fingerprint(&call);
        (
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            runtime_declaration,
            crate::OrientedSubcontinuationPlanV1 {
                representation_rule_version:
                    crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
                frames: vec![frame],
                recursive_calls: Vec::new(),
                computational_ih_slots: vec![slot],
                computational_ih_calls: vec![call],
            },
        )
    }

    #[test]
    fn checked_marker_census_rejects_duplicate_call_and_slot_occurrences_before_cfg() {
        let (entry, declaration, plan) = occurrence_exact_marker_fixture(false, false);
        let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
        validate_oriented_subcontinuation_transport(&entry, &declarations, Some(&plan))
            .expect("the exact checked Runtime marker occurrence ledger closes");

        for (duplicate_call, duplicate_slot, expected) in [
            (
                true,
                false,
                "computational-IH call Runtime occurrences differ",
            ),
            (
                false,
                true,
                "computational-IH slot Runtime occurrences differ",
            ),
        ] {
            let (entry, declaration, plan) =
                occurrence_exact_marker_fixture(duplicate_call, duplicate_slot);
            let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
            let error =
                validate_oriented_subcontinuation_transport(&entry, &declarations, Some(&plan))
                    .expect_err(
                        "an extra static marker occurrence must reject before CFG emission",
                    );
            assert!(
                matches!(
                    error,
                    CraneliftBackendError::Unsupported(UnsupportedLowering {
                        construct: "OrientedSubcontinuationPlanV1",
                        ref reason,
                    }) if reason.contains(expected)
                ),
                "{error:?}"
            );
        }
    }

    #[derive(Clone, Copy)]
    enum Px8jInstallMalformation {
        SelectionRole,
        UnwindRole,
        UnwindOrigin,
        RepeatedScopeIdentity,
    }

    fn run_px8j_source_machine_install(
        malformation: Option<Px8jInstallMalformation>,
    ) -> Result<SourceContinuation<'static>, CraneliftBackendError> {
        let seed_env = NativeSeedEnvironment::empty();
        let mut compiler = root_authority_test_lowering(&seed_env);
        compiler.native_join_plan = None;
        compiler.root_terminal_authority = None;
        compiler.process_object = false;

        let origin = RecursorProducerOriginId(17);
        let layer = |role| ComputationalRecursorLayer {
            cases: Vec::new(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "PX8-J-ERR source install".to_string(),
            },
            outer_env: Vec::new(),
            provenance: RecursorFrameProvenance(18),
            role,
            checked_frame_id: None,
            checked_invocation_id: None,
            checked_invocation_source: None,
            checked_invocation_depth: 0,
            semantic_pending: matches!(role, RecursorLayerRole::SelectsOccurrence { .. }),
        };
        let selection = match malformation {
            Some(Px8jInstallMalformation::SelectionRole) => layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(18),
                parent_scope: None,
            }),
            _ => layer(RecursorLayerRole::SelectsOccurrence { origin }),
        };
        let unwind = match malformation {
            None => Vec::new(),
            Some(Px8jInstallMalformation::SelectionRole) => Vec::new(),
            Some(Px8jInstallMalformation::UnwindRole) => {
                vec![layer(RecursorLayerRole::SelectsOccurrence { origin })]
            }
            Some(Px8jInstallMalformation::UnwindOrigin) => {
                vec![layer(RecursorLayerRole::ExitsScope {
                    origin: RecursorProducerOriginId(99),
                    scope_origin: RecursorProducerOriginId(19),
                    parent_scope: None,
                })]
            }
            Some(Px8jInstallMalformation::RepeatedScopeIdentity) => vec![
                layer(RecursorLayerRole::ExitsScope {
                    origin,
                    scope_origin: RecursorProducerOriginId(19),
                    parent_scope: None,
                }),
                layer(RecursorLayerRole::ExitsScope {
                    origin,
                    scope_origin: RecursorProducerOriginId(19),
                    parent_scope: Some(RecursorProducerOriginId(19)),
                }),
            ],
        };
        let invocation = RecursorInvocationSegment::new(
            origin,
            0,
            selection,
            RecursorUnwindStack {
                later_wrappers_in_construction_order: unwind,
            },
            ContinuationCursorId(20),
            None,
            None,
        );
        assert!(!recursor_invocation_is_checked(&invocation));

        compiler.install_recursor_invocation(
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue),
            ContinuationActivationId(21),
            invocation,
            None,
        )
    }

    #[test]
    fn px8j_all_three_direct_consumers_propagate_the_role_validator() {
        for consumer in [
            Px8jDirectRecursorConsumer::PendingLetProducer,
            Px8jDirectRecursorConsumer::ProducerCall,
            Px8jDirectRecursorConsumer::OrdinaryCall,
        ] {
            let error = match run_px8j_malformed_recursor_consumer(
                consumer,
                Px8jRecursorMalformation::SelectionRole,
            ) {
                Ok(_) => panic!("each live recursor consumer must reject the malformed selection"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error,
                    CraneliftBackendError::Unsupported(UnsupportedLowering {
                        construct: "ComputationalRecursor",
                        ref reason,
                    }) if reason == "recursor selection role does not select the invocation origin"
                ),
                "{consumer:?}: {error:?}"
            );
        }
    }

    #[test]
    fn px8j_release_validator_rejects_repeated_and_broken_scope_lineage() {
        for (malformation, expected_reason) in [
            (
                Px8jRecursorMalformation::RepeatedScopeIdentity,
                "recursor unwind repeats a selected scope identity",
            ),
            (
                Px8jRecursorMalformation::BrokenScopeParent,
                "recursor unwind has a broken selected-scope parent link",
            ),
        ] {
            let error = match run_px8j_malformed_recursor_consumer(
                Px8jDirectRecursorConsumer::OrdinaryCall,
                malformation,
            ) {
                Ok(_) => panic!("the real direct consumer must propagate release validation"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error,
                    CraneliftBackendError::Unsupported(UnsupportedLowering {
                        construct: "ComputationalRecursor",
                        ref reason,
                    }) if reason == expected_reason
                ),
                "{malformation:?}: {error:?}"
            );
        }
    }

    #[test]
    fn px8j_source_machine_install_rejects_repeated_scope_identity() {
        let error = match run_px8j_source_machine_install(Some(
            Px8jInstallMalformation::RepeatedScopeIdentity,
        )) {
            Ok(_) => panic!("the unchecked source-machine install must validate before CFG"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalRecursor",
                reason,
            }) if reason == "recursor unwind repeats a selected scope identity"
        ));
    }

    #[test]
    fn px8j_source_machine_install_rejects_wrong_control_roles_and_origins() {
        for (malformation, expected_reason) in [
            (
                Px8jInstallMalformation::SelectionRole,
                "recursor selection role does not select the invocation origin",
            ),
            (
                Px8jInstallMalformation::UnwindRole,
                "recursor unwind role does not exit the invocation origin",
            ),
            (
                Px8jInstallMalformation::UnwindOrigin,
                "recursor unwind role does not exit the invocation origin",
            ),
        ] {
            let error = match run_px8j_source_machine_install(Some(malformation)) {
                Ok(_) => panic!("the unchecked source-machine install must validate before CFG"),
                Err(error) => error,
            };
            assert!(matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "ComputationalRecursor",
                    ref reason,
                }) if reason == expected_reason
            ));
        }
    }

    #[test]
    fn px8j_source_machine_install_accepts_valid_unchecked_segment() {
        let installed = run_px8j_source_machine_install(None)
            .expect("a valid unchecked source-machine invocation still installs");
        assert!(matches!(
            installed,
            SourceContinuation::ApplyRecursorSelection { .. }
        ));
    }

    #[test]
    fn distinguished_root_cannot_discharge_missing_match_site_marker() {
        let seed_env = NativeSeedEnvironment::empty();
        let mut lowering = Lowering {
            seed_env: &seed_env,
            declarations: BTreeMap::new(),
            declaration_stack: Vec::new(),
            active_recursive_declarations: Vec::new(),
            result_table: BTreeMap::new(),
            next_token: 0,
            next_recursor_frame_provenance: 0,
            next_recursor_producer_origin: 0,
            next_continuation_activation: 0,
            next_continuation_cursor: 0,
            next_source_join: 0,
            next_source_predecessor: 0,
            live_source_continuations: 0,
            source_control_root: None,
            active_oriented_semantic_regions: 0,
            native_join_plan: Some(crate::NativeJoinPlanV1 {
                representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
                sites: vec![self_consistent_root_join_site(0)],
            }),
            consumed_join_sites: BTreeSet::new(),
            root_terminal_authority: None,
            active_join_site: Some(41),
            oriented_subcontinuation_plan: None,
            consumed_subcontinuation_frames: BTreeSet::new(),
            active_subcontinuation_frame: None,
            consumed_recursive_call_templates: BTreeSet::new(),
            pending_recursive_call: None,
            pending_computational_ih_call: None,
            active_recursive_invocations: Vec::new(),
            next_recursive_invocation_instance: 1,
            dynamic_splice_edges: BTreeMap::new(),
            next_dynamic_splice_edge: 1,
            assumptions: BTreeSet::new(),
            unsupported: Vec::new(),
            process_object: false,
            process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
            host_dispatch: None,
            invocation_pointer: None,
            native_int_arena: None,
            native_int_binop: None,
            native_int_compare: None,
            native_int_intern: None,
            native_int_narrow: None,
            native_int_export: None,
            native_int_tags: BTreeMap::new(),
            native_int_mutation: NativeIntLoweringMutation::Exact,
            bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
        };
        let error = lowering
            .planned_join_site_for_frame(EliminatorFrame::InvocationReturn)
            .expect_err("the distinguished root must not discharge an unrelated live marker");
        assert!(
            matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "NativeJoinPlanV1",
                    ref reason,
                }) if reason.contains("root cannot consume an active match occurrence marker")
            ),
            "{error:?}"
        );
        assert_eq!(lowering.active_join_site, Some(41));
        assert!(lowering.consumed_join_sites.is_empty());
    }

    #[test]
    fn valid_root_plus_missing_marked_scalar_cut_rejects_before_emission() {
        let expression = RuntimeExpr::CheckedJoinSite {
            site_id: 41,
            body: Box::new(host_result_computational_fixture(1, true, false)),
        };
        let symbols = crate::NativeProcessSymbols::legacy_prelude();
        let result = compile_expr_into_module(
            new_object_module("px8h-root-marker-class-separation").unwrap(),
            "ken_px8h_root_marker_class_separation",
            Linkage::Export,
            &expression,
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            true,
            Some(&symbols),
            Some(crate::NativeJoinPlanV1 {
                representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
                sites: vec![self_consistent_root_join_site(0)],
            }),
            None,
        );
        let error = match result {
            Ok(_) => panic!("the root must not discharge a missing marked scalar-cut site"),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "NativeJoinPlanV1",
                    ref reason,
                }) if reason.contains("marker was not consumed")
            ),
            "{error:?}"
        );
    }

    #[test]
    fn unmarked_equal_shape_frame_cannot_consume_retained_join_site() {
        let cases = vec![RuntimeMatchCase {
            constructor: "ctor:fixture::PX8H::Only".to_string(),
            binders: 0,
            body: RuntimeExpr::Value(RuntimeValue::Int((7).into())),
        }];
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px8h unmarked equal-shape default".to_string(),
        };
        let fingerprint =
            crate::compiler_private_ordinary_match_frame_fingerprint(&cases, &default);
        let expression = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::PX8H::Only".to_string(),
                args: Vec::new(),
            }),
            cases,
            default,
        };
        let result = compile_expr_into_module(
            new_object_module("px8h-unmarked-equal-shape").unwrap(),
            "ken_px8h_unmarked_equal_shape",
            Linkage::Export,
            &expression,
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            false,
            None,
            Some(crate::NativeJoinPlanV1 {
                representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
                sites: vec![self_consistent_join_site(51, fingerprint)],
            }),
            None,
        );
        let error = match result {
            Ok(_) => panic!("an unmarked equal-shape frame must not consume a plan row"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                reason,
            }) if reason.contains("unconsumed or orphan site")
        ));
    }

    #[test]
    fn self_consistent_appended_orphan_join_site_rejects_before_emission() {
        let result = compile_expr_into_module(
            new_object_module("px8h-orphan-join-site").unwrap(),
            "ken_px8h_orphan_join_site",
            Linkage::Export,
            &RuntimeExpr::Value(RuntimeValue::Int((7).into())),
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            false,
            None,
            Some(crate::NativeJoinPlanV1 {
                representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
                sites: vec![
                    self_consistent_root_join_site(0),
                    self_consistent_join_site(52, 23),
                ],
            }),
            None,
        );
        let error = match result {
            Ok(_) => panic!("a self-consistent orphan plan row must reject"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                reason,
            }) if reason.contains("unconsumed or orphan site")
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
                    RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((3).into())),
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
                                                    RuntimeExpr::Value(RuntimeValue::Int(
                                                        (5).into(),
                                                    )),
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
                                                        RuntimeExpr::Value(RuntimeValue::Int(
                                                            (2).into(),
                                                        )),
                                                    ],
                                                ),
                                                RuntimeExpr::Value(RuntimeValue::Int((3).into())),
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
                ("value".to_string(), RuntimeGroundValue::Int((7).into())),
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
            RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into())),
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
            RuntimeGroundValue::Int((9).into()),
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
            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
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
            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        );
        program.examples.push(program.examples[0].clone());
        let mut run_report = evaluate_runtime_ir_example(
            &nc22_program_with_body(
                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
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
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((0).into())),
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
                value: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
                body: Box::new(RuntimeExpr::Var(0)),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
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
                then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((0).into()))),
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
                    RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
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
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((1).into()))],
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
                    RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
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
                            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                        ],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "unused default".to_string(),
                },
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
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
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((0).into())),
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
                record: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
                field: "x".to_string(),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
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
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
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
                    RuntimeExpr::Value(RuntimeValue::Int((0).into())),
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
                        RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                        RuntimeExpr::Value(RuntimeValue::Int((2).into())),
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
    fn overflowing_int_primitive_promotes_before_native_wrapping_semantics() {
        let example = RuntimeExample {
            name: "overflowing-add-int".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: total_primitive(
                "add_int",
                vec![
                    RuntimeExpr::Value(RuntimeValue::Int((i64::MAX).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                ],
            ),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
                crate::RuntimeIntV1::Big {
                    sign: crate::Sign::NonNegative,
                    limbs: vec![1_u64 << 63],
                },
            )),
        };

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("native lowering promotes before overflow");
        assert_eq!(report.observation, example.observation);
    }

    fn run_exact_int(expr: RuntimeExpr, expected: crate::RuntimeIntV1) {
        let direct =
            crate::evaluate_runtime_ir_expr(&expr, &crate::RuntimeIrSeedEnvironment::empty())
                .expect("backend-neutral Runtime IR evaluates exact Int expression");
        let example = RuntimeExample {
            name: "px8i-exact-int".to_string(),
            checked_core_shape: "PX8-I exact Int discriminator".to_string(),
            ir: expr,
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(expected)),
        };
        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("exact Int expression lowers and executes");
        assert_eq!(direct, example.observation);
        assert_eq!(report.observation, example.observation);
    }

    fn big(sign: crate::Sign, limbs: &[u64]) -> RuntimeExpr {
        RuntimeExpr::Value(RuntimeValue::Int(crate::RuntimeIntV1::Big {
            sign,
            limbs: limbs.to_vec(),
        }))
    }

    #[test]
    fn px8i_big_small_big_mul_and_canonical_narrow_are_exact() {
        run_exact_int(
            total_primitive(
                "add_int",
                vec![
                    big(crate::Sign::NonNegative, &[u64::MAX, 1]),
                    RuntimeExpr::Value(RuntimeValue::Int(1.into())),
                ],
            ),
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![0, 2],
            },
        );
        run_exact_int(
            total_primitive(
                "mul_int",
                vec![
                    big(crate::Sign::NonNegative, &[0, 1]),
                    big(crate::Sign::NonNegative, &[0, 1]),
                ],
            ),
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![0, 0, 1],
            },
        );
        run_exact_int(
            total_primitive(
                "sub_int",
                vec![
                    big(crate::Sign::NonNegative, &[1_u64 << 63]),
                    RuntimeExpr::Value(RuntimeValue::Int(1.into())),
                ],
            ),
            crate::RuntimeIntV1::Small(i64::MAX),
        );
        run_exact_int(
            total_primitive(
                "add_int",
                vec![
                    big(crate::Sign::Negative, &[0, 2]),
                    RuntimeExpr::Value(RuntimeValue::Int(1.into())),
                ],
            ),
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::Negative,
                limbs: vec![u64::MAX, 1],
            },
        );
        run_exact_int(
            total_primitive(
                "sub_int",
                vec![
                    RuntimeExpr::Value(RuntimeValue::Int(1.into())),
                    big(crate::Sign::NonNegative, &[0, 2]),
                ],
            ),
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::Negative,
                limbs: vec![u64::MAX, 1],
            },
        );
    }

    #[test]
    fn px8i_comparison_observes_high_limbs_and_dynamic_join_preserves_pair() {
        let lhs = big(crate::Sign::NonNegative, &[7, 1]);
        let rhs = big(crate::Sign::NonNegative, &[7, 2]);
        let condition = total_primitive("eq_int", vec![lhs.clone(), rhs]);
        run_exact_int(
            RuntimeExpr::If {
                scrutinee: Box::new(condition),
                then_expr: Box::new(big(crate::Sign::NonNegative, &[99, 9])),
                else_expr: Box::new(lhs),
            },
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![7, 1],
            },
        );

        run_exact_int(
            RuntimeExpr::If {
                scrutinee: Box::new(total_primitive(
                    "leq_int",
                    vec![
                        RuntimeExpr::Value(RuntimeValue::Int(i64::MAX.into())),
                        big(crate::Sign::NonNegative, &[0, 1]),
                    ],
                )),
                then_expr: Box::new(big(crate::Sign::NonNegative, &[17, 3])),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((-1).into()))),
            },
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![17, 3],
            },
        );
        run_exact_int(
            RuntimeExpr::If {
                scrutinee: Box::new(total_primitive(
                    "leq_int",
                    vec![
                        big(crate::Sign::Negative, &[0, 1]),
                        RuntimeExpr::Value(RuntimeValue::Int(i64::MIN.into())),
                    ],
                )),
                then_expr: Box::new(big(crate::Sign::Negative, &[23, 4])),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((-1).into()))),
            },
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::Negative,
                limbs: vec![23, 4],
            },
        );
    }

    #[test]
    fn px8i_wrapping_and_trap_mutations_are_causal_at_live_binop_lowering() {
        let expr = total_primitive(
            "add_int",
            vec![
                RuntimeExpr::Value(RuntimeValue::Int(i64::MAX.into())),
                RuntimeExpr::Value(RuntimeValue::Int(1.into())),
            ],
        );
        let example = RuntimeExample {
            name: "px8i-live-mutation".to_string(),
            checked_core_shape: "PX8-I live exact binop mutation".to_string(),
            ir: expr,
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
                crate::RuntimeIntV1::Big {
                    sign: crate::Sign::NonNegative,
                    limbs: vec![1_u64 << 63],
                },
            )),
        };

        NATIVE_INT_LOWERING_MUTATION
            .with(|mutation| mutation.set(NativeIntLoweringMutation::Wrapping));
        let wrapping = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("wrapping mutation still emits the live native expression");
        NATIVE_INT_LOWERING_MUTATION
            .with(|mutation| mutation.set(NativeIntLoweringMutation::Exact));
        assert_ne!(wrapping.observation, example.observation);
        assert_eq!(
            wrapping.observation,
            RuntimeObservation::Returned(RuntimeGroundValue::Int(i64::MIN.into()))
        );

        NATIVE_INT_LOWERING_MUTATION.with(|mutation| mutation.set(NativeIntLoweringMutation::Trap));
        let trapped = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty());
        NATIVE_INT_LOWERING_MUTATION
            .with(|mutation| mutation.set(NativeIntLoweringMutation::Exact));
        assert!(matches!(
            trapped,
            Err(CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "PrimitiveCall",
                ..
            }))
        ));
    }

    #[test]
    fn px8i_jit_terminal_requires_uncorrupted_local_export_evidence() {
        let example = RuntimeExample {
            name: "px8i-terminal-export".to_string(),
            checked_core_shape: "PX8-I terminal Big export".to_string(),
            ir: big(crate::Sign::NonNegative, &[5, 1]),
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
                crate::RuntimeIntV1::Big {
                    sign: crate::Sign::NonNegative,
                    limbs: vec![5, 1],
                },
            )),
        };
        for mutation in [
            NativeIntLoweringMutation::SuppressTerminalExport,
            NativeIntLoweringMutation::CorruptTerminalExport,
        ] {
            NATIVE_INT_LOWERING_MUTATION.with(|cell| cell.set(mutation));
            let result =
                run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty());
            NATIVE_INT_LOWERING_MUTATION.with(|cell| cell.set(NativeIntLoweringMutation::Exact));
            assert!(matches!(
                result,
                Err(CraneliftBackendError::Backend(
                    BackendFailure::NativeResultDecode { .. }
                ))
            ));
        }
    }

    #[test]
    fn px8i_jit_and_object_construct_identical_local_helper_clif() {
        let mut jit = new_jit_module().expect("JIT module constructs");
        let jit_clif = crate::native_int_clif::capture_native_int_local_graph(&mut jit)
            .expect("JIT local helper graph emits");
        let mut object =
            new_object_module("px8i-local-helper-identity").expect("object module constructs");
        let object_clif = crate::native_int_clif::capture_native_int_local_graph(&mut object)
            .expect("object local helper graph emits");
        assert_eq!(jit_clif, object_clif);
        assert!(!jit_clif.is_empty());
        // Rework (Q-RESIDUE, 2026-07-21): the bare `5` was unverified
        // provenance. Grounded against `emit_native_int_local_graph`, which
        // calls exactly six `define_*` helpers (resolve, intern, compare,
        // narrow, export, binop); `capture_native_int_local_graph` joins
        // their captured CLIF bodies with "-- helper --", so N helpers yield
        // N-1 separators. This is a fixed property of the compiler's own
        // small, deliberately-enumerated local-helper set, not an external or
        // growable corpus -- pinning it here catches a helper silently
        // failing to emit a body.
        const LOCAL_HELPER_COUNT: usize = 6;
        assert_eq!(
            jit_clif.matches("-- helper --").count(),
            LOCAL_HELPER_COUNT - 1,
            "expected all {LOCAL_HELPER_COUNT} native-Int local helpers (resolve, intern, compare, narrow, export, binop) to emit a captured CLIF body"
        );
    }

    #[test]
    fn px8i_local_helpers_reject_invalid_zero_stale_and_wrong_arena_slots() {
        let mut module = new_jit_module().expect("JIT module constructs");
        let helpers = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
            .expect("local helper graph emits");
        let pointer = module.target_config().pointer_type();

        let mut mint_signature = module.make_signature();
        mint_signature.params.push(AbiParam::new(pointer));
        mint_signature.returns.push(AbiParam::new(types::I64));
        let mint_id = module
            .declare_function("px8i_mint_probe", Linkage::Local, &mint_signature)
            .expect("mint probe declares");
        let mut mint_context = module.make_context();
        mint_context.func =
            Function::with_name_signature(UserFuncName::user(2, mint_id.as_u32()), mint_signature);
        let intern = module.declare_func_in_func(helpers.intern, &mut mint_context.func);
        let mut frontend = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut mint_context.func, &mut frontend);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let arena = builder.block_params(entry)[0];
            let limbs = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                16,
                3,
            ));
            let zero = builder.ins().iconst(types::I64, 0);
            let one = builder.ins().iconst(types::I64, 1);
            builder.ins().stack_store(zero, limbs, 0);
            builder.ins().stack_store(one, limbs, 8);
            let output = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                16,
                3,
            ));
            let limbs = builder.ins().stack_addr(pointer, limbs, 0);
            let output_pointer = builder.ins().stack_addr(pointer, output, 0);
            let two = builder.ins().iconst(types::I64, 2);
            let call = builder
                .ins()
                .call(intern, &[arena, zero, limbs, two, output_pointer]);
            let status = builder.inst_results(call)[0];
            Lowering::require_i64(&mut builder, status, 0);
            let slot = builder.ins().stack_load(types::I64, output, 8);
            builder.ins().return_(&[slot]);
            builder.seal_all_blocks();
            builder.finalize();
        }
        verify_cranelift_function(&mint_context.func, module.isa()).expect("mint verifies");
        module
            .define_function(mint_id, &mut mint_context)
            .expect("mint defines");

        let mut check_signature = module.make_signature();
        check_signature.params.push(AbiParam::new(pointer));
        check_signature.params.push(AbiParam::new(types::I64));
        check_signature.params.push(AbiParam::new(types::I64));
        check_signature.returns.push(AbiParam::new(types::I64));
        let check_id = module
            .declare_function("px8i_slot_probe", Linkage::Local, &check_signature)
            .expect("slot probe declares");
        let mut check_context = module.make_context();
        check_context.func = Function::with_name_signature(
            UserFuncName::user(2, check_id.as_u32()),
            check_signature,
        );
        let compare = module.declare_func_in_func(helpers.compare, &mut check_context.func);
        let mut frontend = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut check_context.func, &mut frontend);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let params = builder.block_params(entry).to_vec();
            let eq = builder.ins().iconst(types::I64, 0);
            let call = builder.ins().call(
                compare,
                &[params[0], eq, params[1], params[2], params[1], params[2]],
            );
            let status = builder.inst_results(call)[0];
            builder.ins().return_(&[status]);
            builder.seal_all_blocks();
            builder.finalize();
        }
        verify_cranelift_function(&check_context.func, module.isa()).expect("check verifies");
        module
            .define_function(check_id, &mut check_context)
            .expect("check defines");
        module
            .finalize_definitions()
            .expect("probe module finalizes");

        let mint = module.get_finalized_function(mint_id);
        let check = module.get_finalized_function(check_id);
        let mint = unsafe {
            mem::transmute::<_, extern "C" fn(*mut crate::NativeIntArenaV1) -> u64>(mint)
        };
        let check = unsafe {
            mem::transmute::<_, extern "C" fn(*mut crate::NativeIntArenaV1, u64, u64) -> i64>(check)
        };
        let mut first = crate::NativeIntArenaV1::default();
        let mut second = crate::NativeIntArenaV1::default();
        let slot = mint(&mut first);
        assert_ne!(slot, 0);
        assert_eq!(check(&mut first, crate::NATIVE_INT_BIG_TAG_V1, slot), 1);
        assert_eq!(check(&mut first, crate::NATIVE_INT_BIG_TAG_V1, 0), -1);
        assert_eq!(check(&mut second, crate::NATIVE_INT_BIG_TAG_V1, slot), -1);
        assert_eq!(check(&mut first, 9, slot), -1);
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
