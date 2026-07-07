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

use cranelift_codegen::ir::{types, AbiParam, Function, InstBuilder, UserFuncName};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_codegen::{verify_function, Context};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{default_libcall_names, Linkage, Module};

use crate::{
    proof_erasure_boundary_facts_from_program, proof_erasure_witness_error,
    validate_supported_runtime_artifact_certificate, KenCheckedProofErasureBoundaryReport,
    ProofErasureBoundaryWitnessError, ProofErasureBoundaryWitnessStage, RuntimeArtifactCertificate,
    RuntimeArtifactIdentity, RuntimeArtifactValidationError, RuntimeArtifactValidationReport,
    RuntimeDeclarationKind, RuntimeEffectBoundary, RuntimeExample, RuntimeExpr, RuntimeGroundValue,
    RuntimeLowerabilityStatus, RuntimeObservation, RuntimePartiality, RuntimePrimitive,
    RuntimeProgram, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CraneliftRunReport {
    pub example: String,
    pub observation: RuntimeObservation,
    pub verifier_passed: bool,
    pub native_returned: Option<i64>,
    pub trust: NativeTrustReport,
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
pub enum NativeDifferentialStage {
    BoundaryPreflight,
    NativeLoweringOrExecution,
    InterpreterNativeCompare,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeFidelity {
    F0NativeExample,
    F1SeedObservationAgreement,
    F1InterpreterDifferentialAgreement,
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
            "ken_checked_proof_erasure_boundary",
            format!(
                "Ken-checked proof-erasure report identity {:?} does not match RuntimeProgram identity {:?}",
                proof_erasure_boundary.artifact, artifact
            ),
        ));
    }
    let recomputed_facts = proof_erasure_boundary_facts_from_program(program);
    if proof_erasure_boundary.facts != recomputed_facts {
        return Err(proof_erasure_witness_error(
            ProofErasureBoundaryWitnessStage::WitnessMismatch,
            "ken_checked_proof_erasure_boundary",
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
            "package carries runtime-check metadata outside the NC6 D1 supported subset",
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

fn run_example_native(
    example: &RuntimeExample,
    env: &NativeSeedEnvironment,
    fidelity: NativeFidelity,
    evidence: NativeRunEvidence,
    artifact_validation: Option<RuntimeArtifactValidationReport>,
    ken_checked_proof_erasure_boundary: Option<KenCheckedProofErasureBoundaryReport>,
) -> Result<CraneliftRunReport, CraneliftBackendError> {
    let compiled = compile_expr(&example.ir, env)?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let (observation, native_returned) = compiled.run()?;
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

struct CompiledExpr {
    module: JITModule,
    func_id: cranelift_module::FuncId,
    decoder: Option<ResultDecoder>,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    trap: Option<RuntimeTrap>,
    verifier_passed: bool,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
}

#[derive(Clone, Copy)]
enum ResultDecoder {
    Int,
    Bool,
    Table,
}

impl CompiledExpr {
    fn run(mut self) -> Result<(RuntimeObservation, Option<i64>), CraneliftBackendError> {
        if let Some(trap) = self.trap {
            return Ok((RuntimeObservation::Trapped(trap), None));
        }

        self.module
            .finalize_definitions()
            .map_err(|err| backend_module(err.to_string()))?;
        let code = self.module.get_finalized_function(self.func_id);
        let native = unsafe { mem::transmute::<_, extern "C" fn() -> i64>(code) };
        let token = native();
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
    let mut module = new_jit_module()?;
    let mut sig = module.make_signature();
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function("ken_nc6_seed", Linkage::Local, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
    let mut ctx = Context::new();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);

    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        result_table: BTreeMap::new(),
        next_token: 0,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
    };
    let (maybe_trap, decoder) = {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.switch_to_block(block);
        let lowered = compiler.lower_expr(&mut builder, expr, &[])?;
        let result = match lowered {
            Lowered::Trap(trap) => {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
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

    Ok(CompiledExpr {
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

fn new_jit_module() -> Result<JITModule, CraneliftBackendError> {
    let mut flag_builder = settings::builder();
    flag_builder
        .set("use_colocated_libcalls", "false")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    flag_builder
        .set("is_pic", "true")
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa_builder = cranelift_native::builder()
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|err| backend(BackendFailure::Target(err.to_string())))?;
    let builder = JITBuilder::with_isa(isa, default_libcall_names());
    Ok(JITModule::new(builder))
}

fn verify_cranelift_function(
    func: &Function,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<(), CraneliftBackendError> {
    verify_function(func, isa).map_err(|err| backend(BackendFailure::Verifier(err.to_string())))
}

struct Lowering<'a> {
    seed_env: &'a NativeSeedEnvironment,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    next_token: i64,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
}

#[derive(Clone)]
enum Lowered {
    Int(cranelift_codegen::ir::Value),
    Bool(cranelift_codegen::ir::Value),
    Bytes(Vec<u8>),
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
    Trap(RuntimeTrap),
}

impl<'a> Lowering<'a> {
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
            RuntimeExpr::Construct { constructor, args } => {
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
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
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                let Lowered::Constructor { constructor, args } = lowered_scrutinee else {
                    return Err(unsupported(
                        "Match",
                        "scrutinee is not a constructor value in the NC6 subset",
                    ));
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
            RuntimeExpr::Call { callee, args } => {
                let lowered_callee = self.lower_expr(builder, callee, env)?;
                let Lowered::Closure {
                    captures,
                    params,
                    body,
                } = lowered_callee
                else {
                    return Err(unsupported("Call", "callee is not a closure"));
                };
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
                let mut call_env = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                call_env.extend(captures);
                call_env.extend_from_slice(env);
                self.lower_expr(builder, &body, &call_env)
            }
            RuntimeExpr::Trap(trap) => Ok(Lowered::Trap(trap.clone())),
            RuntimeExpr::Let { .. } => Err(unsupported(
                "Let",
                "let lowering is outside the NC6 seed-example subset",
            )),
            RuntimeExpr::If { .. } => Err(unsupported(
                "If",
                "branch lowering is outside the NC6 seed-example subset",
            )),
            RuntimeExpr::Effect { effect, .. } => Err(unsupported(
                "Effect",
                format!("effect {effect} is not modeled in the NC6 D1 subset"),
            )),
        }
    }

    fn lower_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeValue::Bool(value) => Ok(Lowered::Bool(
                builder.ins().iconst(types::I64, i64::from(*value)),
            )),
            RuntimeValue::Int(value) => Ok(Lowered::Int(builder.ins().iconst(types::I64, *value))),
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
                "pre-existing closure references are not lowered by the NC6 seed backend",
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
            RuntimeGroundValue::Bool(value) => Ok(Lowered::Bool(
                builder.ins().iconst(types::I64, i64::from(*value)),
            )),
            RuntimeGroundValue::Int(value) => {
                Ok(Lowered::Int(builder.ins().iconst(types::I64, *value)))
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

        match &primitive.partiality {
            RuntimePartiality::Total => {}
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
            "add_int" => {
                if args.len() != 2 {
                    return Err(unsupported(
                        "PrimitiveCall",
                        format!("add_int expects 2 args, got {}", args.len()),
                    ));
                }
                let mut lowered_args = lowered_args.into_iter();
                let lhs = lowered_args.next().expect("arg count checked");
                let rhs = lowered_args.next().expect("arg count checked");
                let (Lowered::Int(lhs), Lowered::Int(rhs)) = (lhs, rhs) else {
                    return Err(unsupported(
                        "PrimitiveCall",
                        "add_int only supports Int arguments in NC6 D1",
                    ));
                };
                Ok(Lowered::Int(builder.ins().iadd(lhs, rhs)))
            }
            other => Err(unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the NC6 D1 supported set"),
            )),
        }
    }

    fn emit_result(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, ResultDecoder), CraneliftBackendError> {
        match value {
            Lowered::Int(value) => Ok((value, ResultDecoder::Int)),
            Lowered::Bool(value) => Ok((value, ResultDecoder::Bool)),
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

    fn ground_value(
        &mut self,
        value: Lowered,
    ) -> Result<RuntimeGroundValue, CraneliftBackendError> {
        match value {
            Lowered::Int(_) => Err(unsupported(
                "Result",
                "internal error: Int scalar should decode directly from native return",
            )),
            Lowered::Bool(_) => Err(unsupported(
                "Result",
                "internal error: Bool scalar should decode directly from native return",
            )),
            Lowered::Bytes(value) => Ok(RuntimeGroundValue::Bytes(value)),
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
            Lowered::Closure { .. } => Err(unsupported(
                "Closure",
                "closures are callable but not observable ground values in NC6 D1",
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
        nc5_seed_examples, ErasedExecutableCore, RuntimeArtifactValidationStage,
        RuntimeArtifactValidationTier, RuntimeAssumptionTrustKind, RuntimeAssumptionTrustMetadata,
        RuntimeDeclaration, RuntimeEffectsForeignAuditMetadata, RuntimeFieldStatus,
        RuntimeMatchCase, RuntimeMetadata, RuntimeSymbolMetadata,
    };

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
    fn checked_partial_primitive_still_rejects_unknown_arguments() {
        let example = RuntimeExample {
            name: "unknown-partial-arg".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_at".to_string(),
                    partiality: RuntimePartiality::CheckedTrap {
                        obligation: "obl:bytes_at.bounds".to_string(),
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
    fn unsupported_effect_is_distinct_from_backend_failure() {
        let example = RuntimeExample {
            name: "unsupported-effect".to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir: RuntimeExpr::Effect {
                effect: "Console".to_string(),
                capability: Some("cap:Console".to_string()),
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
