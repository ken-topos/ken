//! Internal platform runtime support for Ken-only executables.
//!
//! NC21 sits between executable entrypoint packaging and later native lowering.
//! This module records the starter platform runtime facts and validates that an
//! NC20 entrypoint package can be represented by Ken's internal executable
//! runtime support without exposing a stable ABI, object, linker, library, or
//! host-effect claim.

use std::collections::BTreeSet;
use std::fmt;

use crate::{
    executable_entrypoint_metadata_hash, runtime_executable_entrypoint_package_hash,
    ExecutableArgumentShape, ExecutableDependencyClosure, ExecutableEntrypointTargetKind,
    ExecutableEntrypointVerdict, ExecutableResultShape, ExecutableRuntimeSupport,
    ExecutableTrapShape, RuntimeArtifactIdentity, RuntimeDeclaration, RuntimeDeclarationKind,
    RuntimeExecutableEntrypointPackage, RuntimeExpr, RuntimeGroundValue, RuntimeIrRunReport,
    RuntimeLowerabilityStatus, RuntimeObservation, RuntimeProgram, RuntimeSymbol, RuntimeTrap,
    RuntimeTrapCode, RuntimeValue, EXECUTABLE_ENTRYPOINT_PACKAGE_KIND,
    EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION,
};

pub const PLATFORM_RUNTIME_SUPPORT_KIND: &str = "KenPlatformRuntimeSupport";
pub const PLATFORM_RUNTIME_SUPPORT_VERSION: u32 = 0;
pub const PLATFORM_RUNTIME_SUPPORT_SPEC_REF: &str =
    "docs/program/wp/NC21-platform-runtime-support.md";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeSupportReport {
    pub header: PlatformRuntimeSupportHeader,
    pub runtime_artifact: RuntimeArtifactIdentity,
    pub entrypoint_package_hash: u64,
    pub entrypoint_metadata_identity: u64,
    pub target: RuntimeSymbol,
    pub required_runtime_support: BTreeSet<ExecutableRuntimeSupport>,
    pub representation: PlatformRuntimeRepresentationPolicy,
    pub lifecycle: PlatformExecutableLifecycle,
    pub observation: PlatformRuntimeObservation,
    pub support_facts: PlatformRuntimeSupportFacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeSupportHeader {
    pub support_kind: String,
    pub version: u32,
    pub producer: String,
    pub spec_ref: String,
    pub platform_target: String,
    pub target_symbol: RuntimeSymbol,
    pub support_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeTarget {
    pub kind: PlatformRuntimeTargetKind,
    pub platform_triple: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeTargetKind {
    StarterKenOnlyExecutableV0,
    StableCAbi,
    StableRustAbi,
    SharedLibrary,
    StaticLibrary,
    HostEffectExecution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeRepresentationPolicy {
    pub value_model: String,
    pub closure_model: String,
    pub constructor_model: String,
    pub record_model: String,
    pub primitive_literal_model: String,
    pub trap_model: String,
    pub observation_model: String,
    pub supported_shapes: BTreeSet<PlatformRuntimeShape>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlatformRuntimeShape {
    Bool,
    Int,
    Bytes,
    String,
    Constructor,
    Record,
    Closure,
    PrimitiveLiteral,
    PrimitiveOperation,
    FunctionCall,
    PatternMatch,
    Trap,
    Observation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformExecutableLifecycle {
    pub startup: PlatformExecutableStartup,
    pub shutdown: PlatformExecutableShutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformExecutableStartup {
    pub entrypoint_symbol: RuntimeSymbol,
    pub argument_mode: PlatformExecutableArgumentMode,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformExecutableArgumentMode {
    ClosedNullaryEntrypoint,
    ProcessInputEntrypoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformExecutableShutdown {
    pub result_observation: PlatformRuntimeObservationMode,
    pub trap_observation: PlatformRuntimeObservationMode,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeObservationMode {
    DeterministicRuntimeReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeObservation {
    Returned(PlatformRuntimeValue),
    Trapped(PlatformRuntimeTrap),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeValue {
    Bool(bool),
    Int(crate::RuntimeIntV1),
    Bytes(Vec<u8>),
    String(String),
    Constructor {
        constructor: RuntimeSymbol,
        args: Vec<PlatformRuntimeValue>,
    },
    Record {
        fields: Vec<(String, PlatformRuntimeValue)>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeTrap {
    pub code: RuntimeTrapCode,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeSupportFacts {
    pub semantic_authority: PlatformRuntimeEvidenceFact,
    pub entrypoint_package: PlatformRuntimeEvidenceFact,
    pub runtime_support_component: PlatformRuntimeEvidenceFact,
    pub starter_platform_target: PlatformRuntimeEvidenceFact,
    pub internal_runtime_abi: PlatformRuntimeEvidenceFact,
    pub stable_c_abi: PlatformRuntimeEvidenceFact,
    pub stable_rust_abi: PlatformRuntimeEvidenceFact,
    pub shared_library: PlatformRuntimeEvidenceFact,
    pub static_library: PlatformRuntimeEvidenceFact,
    pub host_effect_execution: PlatformRuntimeEvidenceFact,
    pub garbage_collection_policy: PlatformRuntimeEvidenceFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeEvidenceFact {
    Available {
        value: String,
        evidence_source: String,
        lane: PlatformRuntimeEvidenceLane,
    },
    Unavailable {
        reason: String,
        lane: PlatformRuntimeEvidenceLane,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeEvidenceLane {
    SemanticAuthority,
    Tested,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformRuntimeSupportError {
    pub stage: PlatformRuntimeSupportStage,
    pub field: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformRuntimeSupportStage {
    PlatformTarget,
    EntrypointPackage,
    RuntimeBinding,
    RuntimeExpressionSupport,
    RuntimeObservationSupport,
    Hash,
}

impl PlatformRuntimeTarget {
    pub fn starter(platform_triple: impl Into<String>) -> Self {
        Self {
            kind: PlatformRuntimeTargetKind::StarterKenOnlyExecutableV0,
            platform_triple: platform_triple.into(),
        }
    }

    pub fn starter_host() -> Self {
        Self::starter(format!(
            "{}-{}",
            std::env::consts::ARCH,
            std::env::consts::OS
        ))
    }
}

impl fmt::Display for PlatformRuntimeSupportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.field, self.reason)
    }
}

impl std::error::Error for PlatformRuntimeSupportError {}

pub fn platform_runtime_support_for_entrypoint(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
    run_report: &RuntimeIrRunReport,
    platform: PlatformRuntimeTarget,
    producer: impl Into<String>,
) -> Result<PlatformRuntimeSupportReport, PlatformRuntimeSupportError> {
    validate_runtime_ir_run_report_binding(program, run_report)?;
    validate_runtime_ir_run_report_target(program, package, run_report)?;
    platform_runtime_support_for_observation(
        program,
        package,
        &run_report.observation.observation,
        platform,
        producer,
    )
}

fn platform_runtime_support_for_observation(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
    observation: &RuntimeObservation,
    platform: PlatformRuntimeTarget,
    producer: impl Into<String>,
) -> Result<PlatformRuntimeSupportReport, PlatformRuntimeSupportError> {
    validate_platform_target(&platform)?;
    validate_entrypoint_package_binding(program, package)?;
    let target_declaration = target_declaration(program, &package.entrypoint.target_symbol)?;
    let mut shapes = BTreeSet::new();
    collect_runtime_support_for_declaration(program, target_declaration, &mut shapes)?;
    let observation = platform_observation(observation, &mut shapes)?;
    shapes.insert(PlatformRuntimeShape::Observation);
    validate_required_runtime_support(package)?;

    let mut report = PlatformRuntimeSupportReport {
        header: PlatformRuntimeSupportHeader {
            support_kind: PLATFORM_RUNTIME_SUPPORT_KIND.to_string(),
            version: PLATFORM_RUNTIME_SUPPORT_VERSION,
            producer: producer.into(),
            spec_ref: PLATFORM_RUNTIME_SUPPORT_SPEC_REF.to_string(),
            platform_target: platform.platform_triple,
            target_symbol: package.entrypoint.target_symbol.clone(),
            support_hash: 0,
        },
        runtime_artifact: RuntimeArtifactIdentity::from_program(program),
        entrypoint_package_hash: package.header.package_hash,
        entrypoint_metadata_identity: package.entrypoint.metadata_identity,
        target: package.entrypoint.target_symbol.clone(),
        required_runtime_support: package.entrypoint.required_runtime_support.clone(),
        representation: representation_policy(shapes),
        lifecycle: lifecycle_for_entrypoint(package),
        observation,
        support_facts: support_facts(program, package),
    };
    report.header.support_hash = platform_runtime_support_report_hash(&report);
    Ok(report)
}

pub fn platform_runtime_support_report_hash(report: &PlatformRuntimeSupportReport) -> u64 {
    crate::fnv1a_64(&canonical_platform_runtime_support_report_bytes(report))
}

fn validate_platform_target(
    platform: &PlatformRuntimeTarget,
) -> Result<(), PlatformRuntimeSupportError> {
    if platform.platform_triple.trim().is_empty() {
        return Err(platform_error(
            PlatformRuntimeSupportStage::PlatformTarget,
            "platform_triple",
            "platform target must be recorded explicitly",
        ));
    }
    match platform.kind {
        PlatformRuntimeTargetKind::StarterKenOnlyExecutableV0 => Ok(()),
        PlatformRuntimeTargetKind::StableCAbi => Err(platform_error(
            PlatformRuntimeSupportStage::PlatformTarget,
            "target.kind",
            "NC21 does not expose a stable C ABI",
        )),
        PlatformRuntimeTargetKind::StableRustAbi => Err(platform_error(
            PlatformRuntimeSupportStage::PlatformTarget,
            "target.kind",
            "NC21 does not expose a stable Rust ABI",
        )),
        PlatformRuntimeTargetKind::SharedLibrary => Err(platform_error(
            PlatformRuntimeSupportStage::PlatformTarget,
            "target.kind",
            "NC21 does not generate shared-library artifacts",
        )),
        PlatformRuntimeTargetKind::StaticLibrary => Err(platform_error(
            PlatformRuntimeSupportStage::PlatformTarget,
            "target.kind",
            "NC21 does not generate static-library artifacts",
        )),
        PlatformRuntimeTargetKind::HostEffectExecution => Err(platform_error(
            PlatformRuntimeSupportStage::PlatformTarget,
            "target.kind",
            "NC21 does not execute host effects or FFI",
        )),
    }
}

fn validate_entrypoint_package_binding(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
) -> Result<(), PlatformRuntimeSupportError> {
    if package.header.package_kind != EXECUTABLE_ENTRYPOINT_PACKAGE_KIND {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "package_kind",
            "entrypoint package kind is not KenExecutableEntrypointPackage",
        ));
    }
    if package.header.version != EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "version",
            "entrypoint package version is unsupported by NC21",
        ));
    }
    if package.header.target != package.entrypoint.target_symbol {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "target_symbol",
            "entrypoint package header target does not match entrypoint metadata",
        ));
    }
    if package.header.package_hash != runtime_executable_entrypoint_package_hash(package) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::Hash,
            "package_hash",
            "entrypoint package hash is stale",
        ));
    }
    validate_entrypoint_metadata_payload(package)?;
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if package.runtime_artifact != artifact {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeBinding,
            "runtime_artifact",
            "entrypoint package was not produced from the exact RuntimeProgram",
        ));
    }
    if package.entrypoint.package_identity != program.package_identity
        || package.entrypoint.package_core_semantic_hash != program.core_semantic_hash
        || package.entrypoint.package_artifact_hash != program.artifact_hash
    {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeBinding,
            "package_identity",
            "entrypoint metadata does not match the exact RuntimeProgram identity",
        ));
    }
    Ok(())
}

pub(crate) fn validate_entrypoint_metadata_payload(
    package: &RuntimeExecutableEntrypointPackage,
) -> Result<(), PlatformRuntimeSupportError> {
    let entrypoint = &package.entrypoint;
    if entrypoint.metadata_identity != executable_entrypoint_metadata_hash(entrypoint) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::Hash,
            "entrypoint.metadata_identity",
            "entrypoint metadata identity is stale",
        ));
    }
    if entrypoint.target_kind != ExecutableEntrypointTargetKind::Executable {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.target_kind",
            "entrypoint metadata does not name an executable target",
        ));
    }
    if !matches!(
        entrypoint.closed_entry,
        ExecutableEntrypointVerdict::ClosedKenOnly
    ) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.closed_entry",
            "entrypoint metadata is not closed Ken-only",
        ));
    }
    if !matches!(
        entrypoint.dependency_closure,
        ExecutableDependencyClosure::ClosedKenOnly
    ) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.dependency_closure",
            "entrypoint metadata carries imported or external dependencies",
        ));
    }
    if !entrypoint.unsupported_lanes.is_empty() {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.unsupported_lanes",
            "entrypoint metadata carries unsupported or unavailable lanes",
        ));
    }
    if !matches!(
        entrypoint.argument_packaging.shape,
        ExecutableArgumentShape::ClosedNullary | ExecutableArgumentShape::ProcessInput { .. }
    ) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.argument_packaging",
            "platform runtime only supports closed nullary or process-shaped entrypoints",
        ));
    }
    if !matches!(
        entrypoint.result_observation.shape,
        ExecutableResultShape::RuntimeValue
    ) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.result_observation",
            "entrypoint result observation is not a runtime value",
        ));
    }
    if !matches!(
        entrypoint.trap_contract.shape,
        ExecutableTrapShape::RuntimeTrapReport
    ) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.trap_contract",
            "entrypoint trap contract is not a runtime trap report",
        ));
    }
    if entrypoint.report_contract.target_closure_identity != entrypoint.closure_identity {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "entrypoint.report_contract.target_closure_identity",
            "entrypoint report contract closure identity does not match the entrypoint closure identity",
        ));
    }
    Ok(())
}

fn validate_runtime_ir_run_report_binding(
    program: &RuntimeProgram,
    run_report: &RuntimeIrRunReport,
) -> Result<(), PlatformRuntimeSupportError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if run_report.artifact != artifact || run_report.observation.artifact != artifact {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.artifact",
            "RuntimeIrRunReport does not bind to the exact RuntimeProgram artifact",
        ));
    }
    if run_report.evidence.package_identity != program.package_identity
        || run_report.evidence.core_semantic_hash != program.core_semantic_hash
        || run_report.evidence.runtime_artifact_hash != program.artifact_hash
    {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.evidence",
            "RuntimeIrRunReport evidence does not match the exact RuntimeProgram identity",
        ));
    }
    if run_report.observation.target != run_report.target {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.target",
            "RuntimeIrRunReport observation target does not match the run target",
        ));
    }
    Ok(())
}

fn validate_runtime_ir_run_report_target(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
    run_report: &RuntimeIrRunReport,
) -> Result<(), PlatformRuntimeSupportError> {
    if run_report.evidence.target_example != run_report.target.example
        || run_report.evidence.checked_core_shape != run_report.target.checked_core_shape
    {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.target",
            "RuntimeIrRunReport evidence target does not match the run target",
        ));
    }
    let mut matching_examples = program.examples.iter().filter(|example| {
        example.name == run_report.target.example
            && example.checked_core_shape == run_report.target.checked_core_shape
    });
    let Some(example) = matching_examples.next() else {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.target",
            "RuntimeIrRunReport target is not present in the exact RuntimeProgram examples",
        ));
    };
    if matching_examples.next().is_some() {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.target",
            "RuntimeIrRunReport target identity is ambiguous in the exact RuntimeProgram examples",
        ));
    }
    if !matches!(
        &example.ir,
        RuntimeExpr::DeclarationRef { symbol } if symbol == &package.entrypoint.target_symbol
    ) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeObservationSupport,
            "runtime_ir_run_report.target",
            "RuntimeIrRunReport target does not evaluate the packaged executable entrypoint",
        ));
    }
    Ok(())
}

fn validate_required_runtime_support(
    package: &RuntimeExecutableEntrypointPackage,
) -> Result<(), PlatformRuntimeSupportError> {
    if !package
        .entrypoint
        .required_runtime_support
        .contains(&ExecutableRuntimeSupport::RuntimeValues)
        || !package
            .entrypoint
            .required_runtime_support
            .contains(&ExecutableRuntimeSupport::TrapReporting)
    {
        return Err(platform_error(
            PlatformRuntimeSupportStage::EntrypointPackage,
            "required_runtime_support",
            "NC21 requires explicit RuntimeValues and TrapReporting support",
        ));
    }

    for support in &package.entrypoint.required_runtime_support {
        match support {
            ExecutableRuntimeSupport::RuntimeValues
            | ExecutableRuntimeSupport::FunctionCalls
            | ExecutableRuntimeSupport::PrimitiveValues
            | ExecutableRuntimeSupport::PrimitiveOperations
            | ExecutableRuntimeSupport::AlgebraicData
            | ExecutableRuntimeSupport::PatternMatching
            | ExecutableRuntimeSupport::RecordsSigma
            | ExecutableRuntimeSupport::Dictionaries
            | ExecutableRuntimeSupport::Recursion
            | ExecutableRuntimeSupport::TrapReporting => {}
        }
    }
    Ok(())
}

fn target_declaration<'a>(
    program: &'a RuntimeProgram,
    target: &RuntimeSymbol,
) -> Result<&'a RuntimeDeclaration, PlatformRuntimeSupportError> {
    program
        .declarations
        .iter()
        .find(|declaration| declaration.symbol == *target)
        .ok_or_else(|| {
            platform_error(
                PlatformRuntimeSupportStage::RuntimeBinding,
                "target_symbol",
                "entrypoint target is not present in the exact RuntimeProgram",
            )
        })
}

fn collect_runtime_support_for_declaration(
    program: &RuntimeProgram,
    declaration: &RuntimeDeclaration,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
) -> Result<(), PlatformRuntimeSupportError> {
    let mut visiting = BTreeSet::new();
    collect_runtime_support_for_declaration_inner(program, declaration, shapes, &mut visiting)
}

fn collect_runtime_support_for_declaration_inner(
    program: &RuntimeProgram,
    declaration: &RuntimeDeclaration,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
    visiting: &mut BTreeSet<RuntimeSymbol>,
) -> Result<(), PlatformRuntimeSupportError> {
    require_supported_lowerability(program, &declaration.symbol)?;
    match &declaration.kind {
        RuntimeDeclarationKind::Transparent { body } => {
            if !visiting.insert(declaration.symbol.clone()) {
                shapes.insert(PlatformRuntimeShape::FunctionCall);
                return Ok(());
            }
            let result = collect_runtime_support_for_expr(program, body, shapes, visiting);
            visiting.remove(&declaration.symbol);
            result
        }
        RuntimeDeclarationKind::Primitive { .. }
        | RuntimeDeclarationKind::Data { .. }
        | RuntimeDeclarationKind::Record { .. }
        | RuntimeDeclarationKind::RecursiveGroup { .. }
        | RuntimeDeclarationKind::EffectBoundary { .. }
        | RuntimeDeclarationKind::MetadataOnly => Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeExpressionSupport,
            "target_declaration",
            "NC21 executable startup requires a transparent target body",
        )),
    }
}

fn collect_runtime_support_for_expr(
    program: &RuntimeProgram,
    expr: &RuntimeExpr,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
    visiting: &mut BTreeSet<RuntimeSymbol>,
) -> Result<(), PlatformRuntimeSupportError> {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. } => {
            collect_runtime_support_for_expr(program, body, shapes, visiting)
        }
        RuntimeExpr::Value(value) => collect_runtime_support_for_value(value, shapes),
        RuntimeExpr::Var(_) => Ok(()),
        RuntimeExpr::Let { value, body } => {
            collect_runtime_support_for_expr(program, value, shapes, visiting)?;
            collect_runtime_support_for_expr(program, body, shapes, visiting)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            shapes.insert(PlatformRuntimeShape::PatternMatch);
            collect_runtime_support_for_expr(program, scrutinee, shapes, visiting)?;
            collect_runtime_support_for_expr(program, then_expr, shapes, visiting)?;
            collect_runtime_support_for_expr(program, else_expr, shapes, visiting)
        }
        RuntimeExpr::PrimitiveCall { primitive, args } => {
            shapes.insert(PlatformRuntimeShape::PrimitiveOperation);
            match primitive.partiality {
                crate::RuntimePartiality::Total
                | crate::RuntimePartiality::SafeOption { .. }
                | crate::RuntimePartiality::SafeResult { .. }
                | crate::RuntimePartiality::CheckedTrap { .. }
                | crate::RuntimePartiality::TrustedTrap { .. } => {}
            }
            for arg in args {
                collect_runtime_support_for_expr(program, arg, shapes, visiting)?;
            }
            Ok(())
        }
        RuntimeExpr::Construct { args, .. } => {
            shapes.insert(PlatformRuntimeShape::Constructor);
            for arg in args {
                collect_runtime_support_for_expr(program, arg, shapes, visiting)?;
            }
            Ok(())
        }
        RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } => {
            shapes.insert(PlatformRuntimeShape::PatternMatch);
            collect_runtime_support_for_expr(program, scrutinee, shapes, visiting)?;
            collect_runtime_support_for_trap(default, shapes);
            for case in cases {
                collect_runtime_support_for_expr(program, &case.body, shapes, visiting)?;
            }
            Ok(())
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default,
        } => {
            shapes.insert(PlatformRuntimeShape::PatternMatch);
            collect_runtime_support_for_expr(program, scrutinee, shapes, visiting)?;
            collect_runtime_support_for_trap(default, shapes);
            for case in cases {
                collect_runtime_support_for_expr(program, &case.body, shapes, visiting)?;
            }
            Ok(())
        }
        RuntimeExpr::Record { fields } => {
            shapes.insert(PlatformRuntimeShape::Record);
            for (_, value) in fields {
                collect_runtime_support_for_expr(program, value, shapes, visiting)?;
            }
            Ok(())
        }
        RuntimeExpr::Project { record, .. } => {
            shapes.insert(PlatformRuntimeShape::Record);
            collect_runtime_support_for_expr(program, record, shapes, visiting)
        }
        RuntimeExpr::Closure { body, .. } => {
            shapes.insert(PlatformRuntimeShape::Closure);
            collect_runtime_support_for_expr(program, body, shapes, visiting)
        }
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            shapes.insert(PlatformRuntimeShape::Closure);
            for capture in captures {
                collect_runtime_support_for_expr(program, capture, shapes, visiting)?;
            }
            collect_runtime_support_for_expr(program, body, shapes, visiting)
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            let declaration = target_declaration(program, symbol)?;
            collect_runtime_support_for_declaration_inner(program, declaration, shapes, visiting)
        }
        RuntimeExpr::ImportedDeclarationRef { symbol, .. } => Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeExpressionSupport,
            "ImportedDeclarationRef",
            format!(
                "imported declaration {symbol} is outside the closed Ken-only executable subset"
            ),
        )),
        RuntimeExpr::Call { callee, args } => {
            shapes.insert(PlatformRuntimeShape::FunctionCall);
            collect_runtime_support_for_expr(program, callee, shapes, visiting)?;
            for arg in args {
                collect_runtime_support_for_expr(program, arg, shapes, visiting)?;
            }
            Ok(())
        }
        RuntimeExpr::Effect {
            family, operation, ..
        } => Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeExpressionSupport,
            "Effect",
            format!(
                "host effect {family}.{} is unavailable for NC21 executable runtime support",
                *operation as u16
            ),
        )),
        RuntimeExpr::Trap(trap) => {
            collect_runtime_support_for_trap(trap, shapes);
            Ok(())
        }
    }
}

fn collect_runtime_support_for_value(
    value: &RuntimeValue,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
) -> Result<(), PlatformRuntimeSupportError> {
    match value {
        RuntimeValue::Bool(_) => {
            shapes.insert(PlatformRuntimeShape::Bool);
            shapes.insert(PlatformRuntimeShape::PrimitiveLiteral);
            Ok(())
        }
        RuntimeValue::Int(_) => {
            shapes.insert(PlatformRuntimeShape::Int);
            shapes.insert(PlatformRuntimeShape::PrimitiveLiteral);
            Ok(())
        }
        RuntimeValue::Bytes(_) => {
            shapes.insert(PlatformRuntimeShape::Bytes);
            shapes.insert(PlatformRuntimeShape::PrimitiveLiteral);
            Ok(())
        }
        RuntimeValue::String(_) => {
            shapes.insert(PlatformRuntimeShape::String);
            shapes.insert(PlatformRuntimeShape::PrimitiveLiteral);
            Ok(())
        }
        RuntimeValue::Constructor { args, .. } => {
            shapes.insert(PlatformRuntimeShape::Constructor);
            for arg in args {
                collect_runtime_support_for_value(arg, shapes)?;
            }
            Ok(())
        }
        RuntimeValue::Record { fields } => {
            shapes.insert(PlatformRuntimeShape::Record);
            for (_, value) in fields {
                collect_runtime_support_for_value(value, shapes)?;
            }
            Ok(())
        }
        RuntimeValue::ClosureRef { symbol, .. } => Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeExpressionSupport,
            "ClosureRef",
            format!("pre-existing closure reference {symbol} has no NC21 runtime handle"),
        )),
        RuntimeValue::Unknown => Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeExpressionSupport,
            "Unknown",
            "unknown runtime values must fail before native execution",
        )),
    }
}

fn platform_observation(
    observation: &RuntimeObservation,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
) -> Result<PlatformRuntimeObservation, PlatformRuntimeSupportError> {
    match observation {
        RuntimeObservation::Returned(value) => Ok(PlatformRuntimeObservation::Returned(
            platform_value(value, shapes)?,
        )),
        RuntimeObservation::Trapped(trap) => {
            collect_runtime_support_for_trap(trap, shapes);
            Ok(PlatformRuntimeObservation::Trapped(PlatformRuntimeTrap {
                code: trap.code.clone(),
                message: trap.message.clone(),
            }))
        }
    }
}

fn platform_value(
    value: &RuntimeGroundValue,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
) -> Result<PlatformRuntimeValue, PlatformRuntimeSupportError> {
    match value {
        RuntimeGroundValue::Bool(value) => {
            shapes.insert(PlatformRuntimeShape::Bool);
            Ok(PlatformRuntimeValue::Bool(*value))
        }
        RuntimeGroundValue::Int(value) => {
            shapes.insert(PlatformRuntimeShape::Int);
            Ok(PlatformRuntimeValue::Int(value.clone()))
        }
        RuntimeGroundValue::Bytes(value) => {
            shapes.insert(PlatformRuntimeShape::Bytes);
            Ok(PlatformRuntimeValue::Bytes(value.clone()))
        }
        RuntimeGroundValue::String(value) => {
            shapes.insert(PlatformRuntimeShape::String);
            Ok(PlatformRuntimeValue::String(value.clone()))
        }
        RuntimeGroundValue::Constructor { constructor, args } => {
            shapes.insert(PlatformRuntimeShape::Constructor);
            Ok(PlatformRuntimeValue::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| platform_value(arg, shapes))
                    .collect::<Result<Vec<_>, _>>()?,
            })
        }
        RuntimeGroundValue::Record { fields } => {
            shapes.insert(PlatformRuntimeShape::Record);
            Ok(PlatformRuntimeValue::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), platform_value(value, shapes)?)))
                    .collect::<Result<Vec<_>, PlatformRuntimeSupportError>>()?,
            })
        }
    }
}

fn collect_runtime_support_for_trap(
    trap: &RuntimeTrap,
    shapes: &mut BTreeSet<PlatformRuntimeShape>,
) {
    match trap.code {
        RuntimeTrapCode::UnsupportedErasure
        | RuntimeTrapCode::UnsupportedPrimitivePartiality
        | RuntimeTrapCode::MissingRuntimeMetadata
        | RuntimeTrapCode::PatternMatchFailure
        | RuntimeTrapCode::ExplicitTrap => {}
    }
    shapes.insert(PlatformRuntimeShape::Trap);
}

fn require_supported_lowerability(
    program: &RuntimeProgram,
    symbol: &RuntimeSymbol,
) -> Result<(), PlatformRuntimeSupportError> {
    let lowerability = program
        .erased_core
        .metadata
        .lowerability
        .get(symbol)
        .or_else(|| {
            program
                .declarations
                .iter()
                .find(|declaration| declaration.symbol == *symbol)
                .and_then(|declaration| declaration.metadata.lowerability.as_ref())
        })
        .ok_or_else(|| {
            platform_error(
                PlatformRuntimeSupportStage::RuntimeExpressionSupport,
                "lowerability",
                format!("{symbol} is missing runtime lowerability metadata"),
            )
        })?;
    if !matches!(lowerability, RuntimeLowerabilityStatus::Supported) {
        return Err(platform_error(
            PlatformRuntimeSupportStage::RuntimeExpressionSupport,
            "lowerability",
            format!("{symbol} has blocking lowerability metadata: {lowerability:?}"),
        ));
    }
    Ok(())
}

fn representation_policy(
    supported_shapes: BTreeSet<PlatformRuntimeShape>,
) -> PlatformRuntimeRepresentationPolicy {
    PlatformRuntimeRepresentationPolicy {
        value_model: "runtime scalar immediates plus content-addressed compound value handles"
            .to_string(),
        closure_model: "closed Ken closures with captured runtime values; no stable ABI handle"
            .to_string(),
        constructor_model:
            "constructor tag plus ordered runtime-value payloads in the internal runtime"
                .to_string(),
        record_model: "field-name ordered runtime-value payloads in the internal runtime"
            .to_string(),
        primitive_literal_model:
            "Bool, Int, Bytes, and String literals represented as runtime values".to_string(),
        trap_model: "deterministic RuntimeTrap code and message report".to_string(),
        observation_model: "RuntimeObservation-compatible returned value or trapped report"
            .to_string(),
        supported_shapes,
    }
}

fn lifecycle_for_entrypoint(
    package: &RuntimeExecutableEntrypointPackage,
) -> PlatformExecutableLifecycle {
    let (argument_mode, evidence_source) = match package.entrypoint.argument_packaging.shape {
        ExecutableArgumentShape::ClosedNullary => (
            PlatformExecutableArgumentMode::ClosedNullaryEntrypoint,
            "ExecutableArgumentShape::ClosedNullary accepted before runtime startup",
        ),
        ExecutableArgumentShape::ProcessInput { .. } => (
            PlatformExecutableArgumentMode::ProcessInputEntrypoint,
            "byte-accurate ExecutableArgumentShape::ProcessInput staged before runtime startup",
        ),
        _ => unreachable!("entrypoint argument shape validated before lifecycle construction"),
    };
    PlatformExecutableLifecycle {
        startup: PlatformExecutableStartup {
            entrypoint_symbol: package.entrypoint.target_symbol.clone(),
            argument_mode,
            evidence_source: evidence_source.to_string(),
        },
        shutdown: PlatformExecutableShutdown {
            result_observation: PlatformRuntimeObservationMode::DeterministicRuntimeReport,
            trap_observation: PlatformRuntimeObservationMode::DeterministicRuntimeReport,
            evidence_source:
                "NC20 result/trap contracts require RuntimeValue and RuntimeTrapReport".to_string(),
        },
    }
}

fn support_facts(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
) -> PlatformRuntimeSupportFacts {
    PlatformRuntimeSupportFacts {
        semantic_authority: PlatformRuntimeEvidenceFact::Available {
            value: format!(
                "{} @ core {:016x}, runtime artifact {:016x}",
                program.package_identity, program.core_semantic_hash, program.artifact_hash
            ),
            evidence_source: "RuntimeProgram and NC20 entrypoint package identity".to_string(),
            lane: PlatformRuntimeEvidenceLane::SemanticAuthority,
        },
        entrypoint_package: PlatformRuntimeEvidenceFact::Available {
            value: format!("{:016x}", package.header.package_hash),
            evidence_source: "RuntimeExecutableEntrypointPackage.header.package_hash".to_string(),
            lane: PlatformRuntimeEvidenceLane::Tested,
        },
        runtime_support_component: PlatformRuntimeEvidenceFact::Available {
            value: "ken-runtime internal platform runtime support v0".to_string(),
            evidence_source: "NC21 platform_runtime_support module".to_string(),
            lane: PlatformRuntimeEvidenceLane::Tested,
        },
        starter_platform_target: PlatformRuntimeEvidenceFact::Available {
            value: "starter Ken-only executable target".to_string(),
            evidence_source: "PlatformRuntimeTargetKind::StarterKenOnlyExecutableV0".to_string(),
            lane: PlatformRuntimeEvidenceLane::Tested,
        },
        internal_runtime_abi: PlatformRuntimeEvidenceFact::Available {
            value: "internal Ken executable runtime support only".to_string(),
            evidence_source: "NC21 guardrail: not a public library ABI".to_string(),
            lane: PlatformRuntimeEvidenceLane::Tested,
        },
        stable_c_abi: unavailable_fact("stable C ABI is outside NC21"),
        stable_rust_abi: unavailable_fact("stable Rust ABI is outside NC21"),
        shared_library: unavailable_fact("shared-library generation is outside NC21"),
        static_library: unavailable_fact("static-library generation is outside NC21"),
        host_effect_execution: unavailable_fact("host-effect and FFI execution are outside NC21"),
        garbage_collection_policy: unavailable_fact(
            "NC21 records no garbage-collection policy beyond starter runtime support",
        ),
    }
}

fn unavailable_fact(reason: impl Into<String>) -> PlatformRuntimeEvidenceFact {
    PlatformRuntimeEvidenceFact::Unavailable {
        reason: reason.into(),
        lane: PlatformRuntimeEvidenceLane::Unavailable,
    }
}

fn canonical_platform_runtime_support_report_bytes(
    report: &PlatformRuntimeSupportReport,
) -> Vec<u8> {
    let mut out = String::new();
    push_field(&mut out, "kind", &report.header.support_kind);
    push_field(&mut out, "version", &report.header.version.to_string());
    push_field(&mut out, "producer", &report.header.producer);
    push_field(&mut out, "spec_ref", &report.header.spec_ref);
    push_field(&mut out, "platform_target", &report.header.platform_target);
    push_field(&mut out, "target_symbol", &report.header.target_symbol);
    push_field(
        &mut out,
        "runtime_package_identity",
        &report.runtime_artifact.package_identity,
    );
    push_field(
        &mut out,
        "runtime_core_semantic_hash",
        &format!("{:016x}", report.runtime_artifact.core_semantic_hash),
    );
    push_field(
        &mut out,
        "runtime_artifact_hash",
        &format!("{:016x}", report.runtime_artifact.artifact_hash),
    );
    push_field(
        &mut out,
        "entrypoint_package_hash",
        &format!("{:016x}", report.entrypoint_package_hash),
    );
    push_field(
        &mut out,
        "entrypoint_metadata_identity",
        &format!("{:016x}", report.entrypoint_metadata_identity),
    );
    for support in &report.required_runtime_support {
        push_field(
            &mut out,
            "required_runtime_support",
            runtime_support_tag(support),
        );
    }
    for shape in &report.representation.supported_shapes {
        push_field(&mut out, "supported_shape", platform_shape_tag(shape));
    }
    push_field(
        &mut out,
        "observation",
        &platform_observation_tag(&report.observation),
    );
    out.into_bytes()
}

fn platform_observation_tag(observation: &PlatformRuntimeObservation) -> String {
    match observation {
        PlatformRuntimeObservation::Returned(value) => {
            format!("returned:{}", platform_value_tag(value))
        }
        PlatformRuntimeObservation::Trapped(trap) => {
            format!("trapped:{:?}:{}", trap.code, trap.message)
        }
    }
}

fn platform_value_tag(value: &PlatformRuntimeValue) -> String {
    match value {
        PlatformRuntimeValue::Bool(value) => format!("bool:{value}"),
        PlatformRuntimeValue::Int(value) => format!("int:{value}"),
        PlatformRuntimeValue::Bytes(value) => format!("bytes:{value:?}"),
        PlatformRuntimeValue::String(value) => format!("string:{value}"),
        PlatformRuntimeValue::Constructor { constructor, args } => {
            let args = args
                .iter()
                .map(platform_value_tag)
                .collect::<Vec<_>>()
                .join(",");
            format!("constructor:{constructor}[{args}]")
        }
        PlatformRuntimeValue::Record { fields } => {
            let fields = fields
                .iter()
                .map(|(name, value)| format!("{name}={}", platform_value_tag(value)))
                .collect::<Vec<_>>()
                .join(",");
            format!("record:{{{fields}}}")
        }
    }
}

fn platform_shape_tag(shape: &PlatformRuntimeShape) -> &'static str {
    match shape {
        PlatformRuntimeShape::Bool => "bool",
        PlatformRuntimeShape::Int => "int",
        PlatformRuntimeShape::Bytes => "bytes",
        PlatformRuntimeShape::String => "string",
        PlatformRuntimeShape::Constructor => "constructor",
        PlatformRuntimeShape::Record => "record",
        PlatformRuntimeShape::Closure => "closure",
        PlatformRuntimeShape::PrimitiveLiteral => "primitive_literal",
        PlatformRuntimeShape::PrimitiveOperation => "primitive_operation",
        PlatformRuntimeShape::FunctionCall => "function_call",
        PlatformRuntimeShape::PatternMatch => "pattern_match",
        PlatformRuntimeShape::Trap => "trap",
        PlatformRuntimeShape::Observation => "observation",
    }
}

fn runtime_support_tag(support: &ExecutableRuntimeSupport) -> &'static str {
    match support {
        ExecutableRuntimeSupport::RuntimeValues => "runtime_values",
        ExecutableRuntimeSupport::FunctionCalls => "function_calls",
        ExecutableRuntimeSupport::PrimitiveValues => "primitive_values",
        ExecutableRuntimeSupport::PrimitiveOperations => "primitive_operations",
        ExecutableRuntimeSupport::AlgebraicData => "algebraic_data",
        ExecutableRuntimeSupport::PatternMatching => "pattern_matching",
        ExecutableRuntimeSupport::RecordsSigma => "records_sigma",
        ExecutableRuntimeSupport::Dictionaries => "dictionaries",
        ExecutableRuntimeSupport::Recursion => "recursion",
        ExecutableRuntimeSupport::TrapReporting => "trap_reporting",
    }
}

fn push_field(out: &mut String, name: &str, value: &str) {
    out.push_str(name);
    out.push('=');
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push(';');
}

fn platform_error(
    stage: PlatformRuntimeSupportStage,
    field: &'static str,
    reason: impl Into<String>,
) -> PlatformRuntimeSupportError {
    PlatformRuntimeSupportError {
        stage,
        field,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use crate::{
        evaluate_runtime_ir_example, executable_artifact_contract_for_runtime_report,
        executable_entrypoint_metadata_hash, executable_entrypoint_package_for_runtime_contract,
        summarize_runtime_ir_program, ErasedExecutableCore, ExecutableArgumentPackaging,
        ExecutableArgumentShape, ExecutableDependencyClosure, ExecutableEntrypointPackageMetadata,
        ExecutableEntrypointTargetKind, ExecutableEntrypointVerdict, ExecutableReportContract,
        ExecutableResultObservation, ExecutableResultShape, ExecutableTrapContract,
        ExecutableTrapShape, RuntimeIrProgramReport, RuntimeIrSeedEnvironment, RuntimeMatchCase,
        RuntimeMetadata, RuntimePartiality, RuntimePrimitive, RuntimeSymbolMetadata,
    };

    fn starter_program(body: RuntimeExpr, observation: RuntimeObservation) -> RuntimeProgram {
        let symbol = "decl:fixture::Executable::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::platform-runtime".to_string(),
            core_semantic_hash: 0x3101,
            artifact_hash: 0x3102,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol: symbol.clone(),
                kind: RuntimeDeclarationKind::Transparent { body: body.clone() },
                metadata: RuntimeSymbolMetadata {
                    lowerability: Some(RuntimeLowerabilityStatus::Supported),
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: vec![crate::RuntimeExample {
                name: "platform-runtime-main".to_string(),
                checked_core_shape: "fixture main".to_string(),
                ir: RuntimeExpr::DeclarationRef { symbol },
                observation,
            }],
        }
    }

    fn supported_body_and_observation() -> (RuntimeExpr, RuntimeObservation) {
        let some = "ctor:fixture::Option::Some".to_string();
        let body = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Closure {
                captures: vec![],
                params: vec!["x".to_string()],
                body: Box::new(RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Construct {
                        constructor: some.clone(),
                        args: vec![RuntimeExpr::Var(0)],
                    }),
                    cases: vec![RuntimeMatchCase {
                        constructor: some.clone(),
                        binders: 1,
                        body: RuntimeExpr::Record {
                            fields: vec![
                                (
                                    "payload".to_string(),
                                    RuntimeExpr::PrimitiveCall {
                                        primitive: RuntimePrimitive {
                                            symbol: "add_int".to_string(),
                                            partiality: RuntimePartiality::Total,
                                        },
                                        args: vec![
                                            RuntimeExpr::Var(0),
                                            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                                        ],
                                    },
                                ),
                                (
                                    "wrapped".to_string(),
                                    RuntimeExpr::Construct {
                                        constructor: some.clone(),
                                        args: vec![RuntimeExpr::Var(0)],
                                    },
                                ),
                            ],
                        },
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "no Some case selected".to_string(),
                    },
                }),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((4).into()))],
        };
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Record {
            fields: vec![
                ("payload".to_string(), RuntimeGroundValue::Int((5).into())),
                (
                    "wrapped".to_string(),
                    RuntimeGroundValue::Constructor {
                        constructor: some,
                        args: vec![RuntimeGroundValue::Int((4).into())],
                    },
                ),
            ],
        });
        (body, observation)
    }

    fn packaged_entrypoint(
        program: &RuntimeProgram,
    ) -> (RuntimeIrProgramReport, RuntimeExecutableEntrypointPackage) {
        let report = summarize_runtime_ir_program(program);
        let target = program.declarations[0].symbol.clone();
        let contract = executable_artifact_contract_for_runtime_report(
            program,
            &report,
            target.clone(),
            "platform runtime support unit test",
        )
        .expect("contract materializes");
        let mut entrypoint = ExecutableEntrypointPackageMetadata {
            package_identity: program.package_identity.clone(),
            package_core_semantic_hash: program.core_semantic_hash,
            package_artifact_hash: program.artifact_hash,
            target_symbol: target,
            target_kind: ExecutableEntrypointTargetKind::Executable,
            closure_identity: 0x3201,
            closure_semantic_hash: 0x3202,
            metadata_identity: 0,
            closed_entry: ExecutableEntrypointVerdict::ClosedKenOnly,
            dependency_closure: ExecutableDependencyClosure::ClosedKenOnly,
            required_runtime_support: BTreeSet::from([
                ExecutableRuntimeSupport::RuntimeValues,
                ExecutableRuntimeSupport::FunctionCalls,
                ExecutableRuntimeSupport::PrimitiveValues,
                ExecutableRuntimeSupport::PrimitiveOperations,
                ExecutableRuntimeSupport::AlgebraicData,
                ExecutableRuntimeSupport::PatternMatching,
                ExecutableRuntimeSupport::RecordsSigma,
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
                blocking_lanes: BTreeMap::new(),
            },
            report_contract: ExecutableReportContract {
                target_closure_identity: 0x3201,
                target_closure_report_hash: 0x3203,
                evidence_source: "target closure report".to_string(),
            },
            unsupported_lanes: BTreeMap::new(),
        };
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);
        let package = executable_entrypoint_package_for_runtime_contract(
            program,
            &report,
            &contract,
            entrypoint,
            "platform runtime support unit test",
        )
        .expect("entrypoint package materializes");
        (report, package)
    }

    fn runtime_ir_run_report(program: &RuntimeProgram) -> RuntimeIrRunReport {
        runtime_ir_run_report_for_example(program, 0)
    }

    fn runtime_ir_run_report_for_example(
        program: &RuntimeProgram,
        example_index: usize,
    ) -> RuntimeIrRunReport {
        evaluate_runtime_ir_example(
            program,
            &program.examples[example_index],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator produces an observation")
    }

    #[test]
    fn platform_support_binds_entrypoint_and_represents_starter_shapes() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation.clone());
        let (_report, package) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        assert_eq!(run_report.observation.observation, observation);

        let support = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect("platform support materializes");

        assert_eq!(
            support.runtime_artifact,
            RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(support.entrypoint_package_hash, package.header.package_hash);
        assert_eq!(
            support.header.support_hash,
            platform_runtime_support_report_hash(&support)
        );
        assert!(support
            .representation
            .supported_shapes
            .contains(&PlatformRuntimeShape::Closure));
        assert!(support
            .representation
            .supported_shapes
            .contains(&PlatformRuntimeShape::Constructor));
        assert!(support
            .representation
            .supported_shapes
            .contains(&PlatformRuntimeShape::Record));
        assert!(support
            .representation
            .supported_shapes
            .contains(&PlatformRuntimeShape::PrimitiveOperation));
        assert!(matches!(
            support.support_facts.stable_c_abi,
            PlatformRuntimeEvidenceFact::Unavailable { .. }
        ));
    }

    #[test]
    fn process_entrypoint_lifecycle_records_staged_process_input() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation);
        let (_report, mut package) = packaged_entrypoint(&program);
        package.entrypoint.argument_packaging = crate::ExecutableArgumentPackaging {
            shape: ExecutableArgumentShape::ProcessInput {
                arguments: vec![b"ken".to_vec(), vec![0xff]],
                environment: vec![(vec![0xfe], vec![0xfd])],
                working_directory: b"/tmp".to_vec(),
            },
            evidence_source: "raw process bytes staged by native runtime init".to_string(),
        };
        package.entrypoint.metadata_identity =
            executable_entrypoint_metadata_hash(&package.entrypoint);
        package.header.package_hash = runtime_executable_entrypoint_package_hash(&package);
        let run_report = runtime_ir_run_report(&program);

        let support = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime process lifecycle test",
        )
        .expect("process lifecycle materializes");
        assert_eq!(
            support.lifecycle.startup.argument_mode,
            PlatformExecutableArgumentMode::ProcessInputEntrypoint
        );
        assert!(support
            .lifecycle
            .startup
            .evidence_source
            .contains("byte-accurate"));
    }

    #[test]
    fn trap_observation_stays_runtime_report_comparable() {
        let trap = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "entrypoint trapped deterministically".to_string(),
        };
        let observation = RuntimeObservation::Trapped(trap.clone());
        let program = starter_program(RuntimeExpr::Trap(trap), observation.clone());
        let (_report, package) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        assert_eq!(run_report.observation.observation, observation);

        let support = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect("trap observation is representable");

        assert!(support
            .representation
            .supported_shapes
            .contains(&PlatformRuntimeShape::Trap));
        assert!(matches!(
            support.observation,
            PlatformRuntimeObservation::Trapped(PlatformRuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                ..
            })
        ));
    }

    #[test]
    fn stale_entrypoint_package_hash_rejects_before_runtime_support() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation.clone());
        let (_report, mut package) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        package.header.package_hash ^= 1;

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("stale package hash rejects");
        assert_eq!(err.stage, PlatformRuntimeSupportStage::Hash);
        assert_eq!(err.field, "package_hash");
    }

    #[test]
    fn stale_entrypoint_metadata_payload_rejects_before_runtime_support() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation.clone());
        let (_report, mut package) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        package.entrypoint.target_kind = ExecutableEntrypointTargetKind::Library;

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("stale embedded entrypoint metadata rejects");
        assert_eq!(err.stage, PlatformRuntimeSupportStage::Hash);
        assert_eq!(err.field, "entrypoint.metadata_identity");
    }

    #[test]
    fn refreshed_non_executable_entrypoint_payload_rejects() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation.clone());
        let (_report, mut package) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        package.entrypoint.target_kind = ExecutableEntrypointTargetKind::Library;
        package.entrypoint.metadata_identity =
            executable_entrypoint_metadata_hash(&package.entrypoint);
        package.header.package_hash = runtime_executable_entrypoint_package_hash(&package);

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("refreshed non-executable entrypoint metadata rejects");
        assert_eq!(err.stage, PlatformRuntimeSupportStage::EntrypointPackage);
        assert_eq!(err.field, "entrypoint.target_kind");
    }

    #[test]
    fn stable_abi_target_rejects_before_native_execution() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation.clone());
        let (_report, package) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget {
                kind: PlatformRuntimeTargetKind::StableCAbi,
                platform_triple: "test-starter-platform".to_string(),
            },
            "ken-runtime unit test",
        )
        .expect_err("stable ABI target rejects");
        assert_eq!(err.stage, PlatformRuntimeSupportStage::PlatformTarget);
        assert_eq!(err.field, "target.kind");
    }

    #[test]
    fn host_effect_execution_target_stays_a_named_unavailable_lane() {
        let target = PlatformRuntimeTarget {
            kind: PlatformRuntimeTargetKind::HostEffectExecution,
            platform_triple: "test-host-effect-target".to_string(),
        };
        let err = validate_platform_target(&target)
            .expect_err("HostEffectExecution remains unavailable until PX5");
        assert_eq!(err.stage, PlatformRuntimeSupportStage::PlatformTarget);
        assert_eq!(err.field, "target.kind");
        assert!(err.reason.contains("does not execute host effects"));
    }

    #[test]
    fn imported_declaration_ref_rejects_even_under_closed_entrypoint_claim() {
        let body = RuntimeExpr::ImportedDeclarationRef {
            symbol: "decl:dep::value".to_string(),
            dependency: "module:dep".to_string(),
            dependency_semantic_hash: "dep-hash".to_string(),
        };
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into()));
        let program = starter_program(body, observation.clone());
        let (_report, package) = packaged_entrypoint(&program);

        let err = platform_runtime_support_for_observation(
            &program,
            &package,
            &observation,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("imported refs fail closed at NC21");
        assert_eq!(
            err.stage,
            PlatformRuntimeSupportStage::RuntimeExpressionSupport
        );
        assert_eq!(err.field, "ImportedDeclarationRef");
    }

    #[test]
    fn declaration_ref_scans_referenced_transparent_body() {
        let helper = "decl:fixture::Executable::helper".to_string();
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into()));
        let mut program = starter_program(
            RuntimeExpr::DeclarationRef {
                symbol: helper.clone(),
            },
            observation.clone(),
        );
        program.erased_core.symbols.insert(helper.clone());
        program
            .erased_core
            .metadata
            .lowerability
            .insert(helper.clone(), RuntimeLowerabilityStatus::Supported);
        program.declarations.push(RuntimeDeclaration {
            symbol: helper,
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::ImportedDeclarationRef {
                    symbol: "decl:dep::hidden".to_string(),
                    dependency: "module:dep".to_string(),
                    dependency_semantic_hash: "dep-hash".to_string(),
                },
            },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        });
        let (_report, package) = packaged_entrypoint(&program);

        let err = platform_runtime_support_for_observation(
            &program,
            &package,
            &observation,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("referenced unsupported body fails closed");
        assert_eq!(
            err.stage,
            PlatformRuntimeSupportStage::RuntimeExpressionSupport
        );
        assert_eq!(err.field, "ImportedDeclarationRef");
    }

    #[test]
    fn runtime_ir_run_report_must_bind_exact_runtime_artifact() {
        let (body, observation) = supported_body_and_observation();
        let program = starter_program(body, observation);
        let (_report, package) = packaged_entrypoint(&program);
        let mut run_report = runtime_ir_run_report(&program);
        run_report.artifact.artifact_hash ^= 1;

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("stale runtime-IR run report rejects");
        assert_eq!(
            err.stage,
            PlatformRuntimeSupportStage::RuntimeObservationSupport
        );
        assert_eq!(err.field, "runtime_ir_run_report.artifact");
    }

    #[test]
    fn runtime_ir_run_report_must_evaluate_packaged_entrypoint() {
        let (body, observation) = supported_body_and_observation();
        let mut program = starter_program(body, observation);
        let helper = "decl:fixture::Executable::helper".to_string();
        program.erased_core.symbols.insert(helper.clone());
        program
            .erased_core
            .metadata
            .lowerability
            .insert(helper.clone(), RuntimeLowerabilityStatus::Supported);
        program.declarations.push(RuntimeDeclaration {
            symbol: helper.clone(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::Value(RuntimeValue::Bool(true)),
            },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        });
        program.examples.push(crate::RuntimeExample {
            name: "platform-runtime-helper".to_string(),
            checked_core_shape: "fixture helper".to_string(),
            ir: RuntimeExpr::DeclarationRef { symbol: helper },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Bool(true)),
        });
        let (_report, package) = packaged_entrypoint(&program);
        let helper_run_report = runtime_ir_run_report_for_example(&program, 1);

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &helper_run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("helper run report must not satisfy the main entrypoint");
        assert_eq!(
            err.stage,
            PlatformRuntimeSupportStage::RuntimeObservationSupport
        );
        assert_eq!(err.field, "runtime_ir_run_report.target");
    }

    #[test]
    fn runtime_ir_run_report_target_identity_must_be_unique() {
        let (body, observation) = supported_body_and_observation();
        let mut program = starter_program(body, observation);
        let helper = "decl:fixture::Executable::helper".to_string();
        program.erased_core.symbols.insert(helper.clone());
        program
            .erased_core
            .metadata
            .lowerability
            .insert(helper.clone(), RuntimeLowerabilityStatus::Supported);
        program.declarations.push(RuntimeDeclaration {
            symbol: helper.clone(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::Value(RuntimeValue::Bool(true)),
            },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        });
        program.examples.push(crate::RuntimeExample {
            name: program.examples[0].name.clone(),
            checked_core_shape: program.examples[0].checked_core_shape.clone(),
            ir: RuntimeExpr::DeclarationRef { symbol: helper },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Bool(true)),
        });
        let (_report, package) = packaged_entrypoint(&program);
        let helper_run_report = runtime_ir_run_report_for_example(&program, 1);

        let err = platform_runtime_support_for_entrypoint(
            &program,
            &package,
            &helper_run_report,
            PlatformRuntimeTarget::starter("test-starter-platform"),
            "ken-runtime unit test",
        )
        .expect_err("duplicate runtime example target identity rejects");
        assert_eq!(
            err.stage,
            PlatformRuntimeSupportStage::RuntimeObservationSupport
        );
        assert_eq!(err.field, "runtime_ir_run_report.target");
    }
}
