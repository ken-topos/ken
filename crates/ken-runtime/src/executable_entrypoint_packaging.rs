//! Executable entrypoint packaging above `RuntimeProgram`.
//!
//! This layer consumes package-authoritative entrypoint metadata and binds it
//! to the exact runtime artifact plus the executable-artifact contract. It does
//! not select targets, inspect raw source/module paths, emit native objects, or
//! treat native/toolchain facts as semantic authority.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    executable_artifact_contract_hash, runtime_ir_program_report_hash,
    validate_executable_artifact_contract, ExecutableArtifactContract,
    ExecutableArtifactContractError, ExecutableArtifactContractReport,
    ExecutableNativeArtifactStatus, RuntimeArtifactIdentity, RuntimeIrProgramReport,
    RuntimeProgram, RuntimeSymbol,
};

pub const EXECUTABLE_ENTRYPOINT_PACKAGE_KIND: &str = "KenExecutableEntrypointPackage";
pub const EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION: u32 = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableEntrypointPackageMetadata {
    pub package_identity: RuntimeSymbol,
    pub package_core_semantic_hash: u64,
    pub package_artifact_hash: u64,
    pub target_symbol: RuntimeSymbol,
    pub target_kind: ExecutableEntrypointTargetKind,
    pub closure_identity: u64,
    pub closure_semantic_hash: u64,
    pub metadata_identity: u64,
    pub closed_entry: ExecutableEntrypointVerdict,
    pub dependency_closure: ExecutableDependencyClosure,
    pub required_runtime_support: BTreeSet<ExecutableRuntimeSupport>,
    pub argument_packaging: ExecutableArgumentPackaging,
    pub result_observation: ExecutableResultObservation,
    pub trap_contract: ExecutableTrapContract,
    pub report_contract: ExecutableReportContract,
    pub unsupported_lanes: BTreeMap<RuntimeSymbol, Vec<ExecutableEntrypointUnavailableLane>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableEntrypointTargetKind {
    Executable,
    Library,
    NonRuntime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableEntrypointVerdict {
    ClosedKenOnly,
    Unavailable {
        lanes: Vec<ExecutableEntrypointUnavailableLane>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableDependencyClosure {
    ClosedKenOnly,
    ImportsUnavailable {
        external_symbols: BTreeSet<RuntimeSymbol>,
        imported_declaration_refs: BTreeMap<RuntimeSymbol, RuntimeSymbol>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutableRuntimeSupport {
    RuntimeValues,
    FunctionCalls,
    PrimitiveValues,
    PrimitiveOperations,
    AlgebraicData,
    PatternMatching,
    RecordsSigma,
    Dictionaries,
    Recursion,
    TrapReporting,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableArgumentPackaging {
    pub shape: ExecutableArgumentShape,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableArgumentShape {
    ClosedNullary,
    ProcessInput {
        arguments: Vec<Vec<u8>>,
        environment: Vec<(Vec<u8>, Vec<u8>)>,
        working_directory: Vec<u8>,
    },
    UnsupportedRuntimeArguments {
        parameter_count: usize,
    },
    Unavailable {
        lane: ExecutableEntrypointUnavailableLane,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableResultObservation {
    pub shape: ExecutableResultShape,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableResultShape {
    RuntimeValue,
    Unavailable {
        lane: ExecutableEntrypointUnavailableLane,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableTrapContract {
    pub shape: ExecutableTrapShape,
    pub blocking_lanes: BTreeMap<RuntimeSymbol, Vec<ExecutableEntrypointUnavailableLane>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableTrapShape {
    RuntimeTrapReport,
    Unavailable {
        lane: ExecutableEntrypointUnavailableLane,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableReportContract {
    pub target_closure_identity: u64,
    pub target_closure_report_hash: u64,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableEntrypointUnavailableLane {
    pub lane: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutableEntrypointPackage {
    pub header: RuntimeExecutableEntrypointPackageHeader,
    pub runtime_artifact: RuntimeArtifactIdentity,
    pub runtime_report_hash: u64,
    pub executable_contract_hash: u64,
    pub executable_contract_report: ExecutableArtifactContractReport,
    pub entrypoint: ExecutableEntrypointPackageMetadata,
    pub native_artifact_status: ExecutableNativeArtifactStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutableEntrypointPackageHeader {
    pub package_kind: String,
    pub version: u32,
    pub producer: String,
    pub target: RuntimeSymbol,
    pub package_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableEntrypointPackagingError {
    pub stage: ExecutableEntrypointPackagingStage,
    pub field: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableEntrypointPackagingStage {
    EntrypointIdentity,
    EntrypointClosed,
    RuntimeBinding,
    ReportBinding,
    ContractBinding,
    Hash,
}

impl fmt::Display for ExecutableEntrypointPackagingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.field, self.reason)
    }
}

impl std::error::Error for ExecutableEntrypointPackagingError {}

impl From<ExecutableArtifactContractError> for ExecutableEntrypointPackagingError {
    fn from(err: ExecutableArtifactContractError) -> Self {
        packaging_error(
            ExecutableEntrypointPackagingStage::ContractBinding,
            err.field,
            err.reason,
        )
    }
}

pub fn executable_entrypoint_package_for_runtime_contract(
    program: &RuntimeProgram,
    report: &RuntimeIrProgramReport,
    contract: &ExecutableArtifactContract,
    entrypoint: ExecutableEntrypointPackageMetadata,
    producer: impl Into<String>,
) -> Result<RuntimeExecutableEntrypointPackage, ExecutableEntrypointPackagingError> {
    let contract_report = validate_executable_artifact_contract(program, report, contract)?;
    validate_entrypoint_package(program, report, contract, &entrypoint)?;

    let mut package = RuntimeExecutableEntrypointPackage {
        header: RuntimeExecutableEntrypointPackageHeader {
            package_kind: EXECUTABLE_ENTRYPOINT_PACKAGE_KIND.to_string(),
            version: EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION,
            producer: producer.into(),
            target: entrypoint.target_symbol.clone(),
            package_hash: 0,
        },
        runtime_artifact: RuntimeArtifactIdentity::from_program(program),
        runtime_report_hash: runtime_ir_program_report_hash(report),
        executable_contract_hash: executable_artifact_contract_hash(contract),
        executable_contract_report: contract_report,
        native_artifact_status: contract.native_artifact.status.clone(),
        entrypoint,
    };
    package.header.package_hash = runtime_executable_entrypoint_package_hash(&package);
    Ok(package)
}

pub fn validate_entrypoint_package(
    program: &RuntimeProgram,
    report: &RuntimeIrProgramReport,
    contract: &ExecutableArtifactContract,
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Result<(), ExecutableEntrypointPackagingError> {
    validate_entrypoint_identity(program, entrypoint)?;
    validate_entrypoint_closed(entrypoint)?;
    validate_runtime_binding(program, entrypoint)?;
    validate_report_binding(report, entrypoint)?;
    validate_contract_binding(contract, entrypoint)?;
    Ok(())
}

pub fn executable_entrypoint_metadata_hash(
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> u64 {
    crate::fnv1a_64(&canonical_entrypoint_metadata_bytes(entrypoint))
}

pub fn runtime_executable_entrypoint_package_hash(
    package: &RuntimeExecutableEntrypointPackage,
) -> u64 {
    crate::fnv1a_64(&canonical_runtime_entrypoint_package_bytes(package))
}

fn validate_entrypoint_identity(
    program: &RuntimeProgram,
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Result<(), ExecutableEntrypointPackagingError> {
    if entrypoint.package_identity != program.package_identity {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointIdentity,
            "package_identity",
            "entrypoint package identity does not match the RuntimeProgram",
        ));
    }
    if entrypoint.package_core_semantic_hash != program.core_semantic_hash {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointIdentity,
            "package_core_semantic_hash",
            "entrypoint semantic hash does not match the RuntimeProgram",
        ));
    }
    if entrypoint.package_artifact_hash != program.artifact_hash {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointIdentity,
            "package_artifact_hash",
            "entrypoint artifact hash does not match the RuntimeProgram",
        ));
    }
    if !program
        .declarations
        .iter()
        .any(|declaration| declaration.symbol == entrypoint.target_symbol)
    {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointIdentity,
            "target_symbol",
            "entrypoint target is not present in the exact RuntimeProgram",
        ));
    }
    validate_nonempty_text(
        ExecutableEntrypointPackagingStage::EntrypointIdentity,
        "report_contract.evidence_source",
        &entrypoint.report_contract.evidence_source,
    )?;
    if entrypoint.report_contract.target_closure_identity != entrypoint.closure_identity {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointIdentity,
            "report_contract.target_closure_identity",
            "entrypoint report contract closure identity does not match the entrypoint closure identity",
        ));
    }
    validate_lane_map(&entrypoint.unsupported_lanes)?;
    if entrypoint.metadata_identity != executable_entrypoint_metadata_hash(entrypoint) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::Hash,
            "metadata_identity",
            "entrypoint metadata identity is stale",
        ));
    }
    Ok(())
}

fn validate_entrypoint_closed(
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Result<(), ExecutableEntrypointPackagingError> {
    if entrypoint.target_kind != ExecutableEntrypointTargetKind::Executable {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "target_kind",
            "entrypoint metadata does not name an executable target",
        ));
    }
    if !matches!(
        entrypoint.closed_entry,
        ExecutableEntrypointVerdict::ClosedKenOnly
    ) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "closed_entry",
            "entrypoint is not a closed Ken-only executable target",
        ));
    }
    if !matches!(
        entrypoint.dependency_closure,
        ExecutableDependencyClosure::ClosedKenOnly
    ) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "dependency_closure",
            "entrypoint has unresolved imported or external dependencies",
        ));
    }
    if !entrypoint.unsupported_lanes.is_empty() {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "unsupported_lanes",
            "entrypoint carries explicit unsupported or unavailable lanes",
        ));
    }
    if !entrypoint
        .required_runtime_support
        .contains(&ExecutableRuntimeSupport::RuntimeValues)
        || !entrypoint
            .required_runtime_support
            .contains(&ExecutableRuntimeSupport::TrapReporting)
    {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "required_runtime_support",
            "entrypoint package must explicitly require runtime values and trap reporting",
        ));
    }
    if !matches!(
        entrypoint.argument_packaging.shape,
        ExecutableArgumentShape::ClosedNullary | ExecutableArgumentShape::ProcessInput { .. }
    ) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "argument_packaging",
            "runtime executable packaging only accepts closed nullary or process-shaped entrypoints",
        ));
    }
    if !matches!(
        entrypoint.result_observation.shape,
        ExecutableResultShape::RuntimeValue
    ) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "result_observation",
            "entrypoint result observation is unavailable",
        ));
    }
    if !matches!(
        entrypoint.trap_contract.shape,
        ExecutableTrapShape::RuntimeTrapReport
    ) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::EntrypointClosed,
            "trap_contract",
            "entrypoint trap contract is unavailable",
        ));
    }
    validate_nonempty_text(
        ExecutableEntrypointPackagingStage::EntrypointClosed,
        "argument_packaging.evidence_source",
        &entrypoint.argument_packaging.evidence_source,
    )?;
    validate_nonempty_text(
        ExecutableEntrypointPackagingStage::EntrypointClosed,
        "result_observation.evidence_source",
        &entrypoint.result_observation.evidence_source,
    )?;
    Ok(())
}

fn validate_runtime_binding(
    program: &RuntimeProgram,
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Result<(), ExecutableEntrypointPackagingError> {
    let target_lowerability = program
        .erased_core
        .metadata
        .lowerability
        .get(&entrypoint.target_symbol)
        .or_else(|| {
            program
                .declarations
                .iter()
                .find(|declaration| declaration.symbol == entrypoint.target_symbol)
                .and_then(|declaration| declaration.metadata.lowerability.as_ref())
        });
    if !matches!(
        target_lowerability,
        Some(crate::RuntimeLowerabilityStatus::Supported)
    ) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::RuntimeBinding,
            "lowerability",
            "entrypoint target is not supported by the exact RuntimeProgram",
        ));
    }
    Ok(())
}

fn validate_report_binding(
    report: &RuntimeIrProgramReport,
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Result<(), ExecutableEntrypointPackagingError> {
    let target = &entrypoint.target_symbol;
    if !report.supported_runtime_targets.contains(target) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::ReportBinding,
            "supported_runtime_targets",
            "entrypoint target is absent from RuntimeIrProgramReport.supported_runtime_targets",
        ));
    }
    if report.unsupported_targets.contains_key(target) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::ReportBinding,
            "unsupported_targets",
            "entrypoint target is unsupported by the RuntimeIrProgramReport",
        ));
    }
    if report.comparison_unavailable_targets.contains_key(target) {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::ReportBinding,
            "comparison_unavailable_targets",
            "entrypoint target comparison is unavailable in the RuntimeIrProgramReport",
        ));
    }
    Ok(())
}

fn validate_contract_binding(
    contract: &ExecutableArtifactContract,
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Result<(), ExecutableEntrypointPackagingError> {
    if contract.header.target != entrypoint.target_symbol
        || contract.runtime.selected_target != entrypoint.target_symbol
        || contract.report.selected_target != entrypoint.target_symbol
    {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::ContractBinding,
            "target_symbol",
            "entrypoint target does not match the executable-artifact contract target",
        ));
    }
    if contract.checked_core.package_identity != entrypoint.package_identity
        || contract.checked_core.core_semantic_hash != entrypoint.package_core_semantic_hash
        || contract.checked_core.artifact_hash != entrypoint.package_artifact_hash
    {
        return Err(packaging_error(
            ExecutableEntrypointPackagingStage::ContractBinding,
            "checked_core",
            "entrypoint package identity does not match the executable-artifact contract",
        ));
    }
    Ok(())
}

fn validate_lane_map(
    lanes: &BTreeMap<RuntimeSymbol, Vec<ExecutableEntrypointUnavailableLane>>,
) -> Result<(), ExecutableEntrypointPackagingError> {
    for (symbol, lanes) in lanes {
        validate_nonempty_text(
            ExecutableEntrypointPackagingStage::EntrypointIdentity,
            "unsupported_lanes.symbol",
            symbol,
        )?;
        for lane in lanes {
            validate_nonempty_text(
                ExecutableEntrypointPackagingStage::EntrypointIdentity,
                "unsupported_lanes.lane",
                &lane.lane,
            )?;
            validate_nonempty_text(
                ExecutableEntrypointPackagingStage::EntrypointIdentity,
                "unsupported_lanes.reason",
                &lane.reason,
            )?;
        }
    }
    Ok(())
}

fn validate_nonempty_text(
    stage: ExecutableEntrypointPackagingStage,
    field: &'static str,
    value: &str,
) -> Result<(), ExecutableEntrypointPackagingError> {
    if value.trim().is_empty() {
        return Err(packaging_error(
            stage,
            field,
            "required entrypoint packaging field must be nonempty",
        ));
    }
    Ok(())
}

fn canonical_entrypoint_metadata_bytes(
    entrypoint: &ExecutableEntrypointPackageMetadata,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_len_str(&mut bytes, &entrypoint.package_identity);
    push_len_str(
        &mut bytes,
        &format!("{:016x}", entrypoint.package_core_semantic_hash),
    );
    push_len_str(
        &mut bytes,
        &format!("{:016x}", entrypoint.package_artifact_hash),
    );
    push_len_str(&mut bytes, &entrypoint.target_symbol);
    push_target_kind(&mut bytes, &entrypoint.target_kind);
    push_len_str(&mut bytes, &format!("{:016x}", entrypoint.closure_identity));
    push_len_str(
        &mut bytes,
        &format!("{:016x}", entrypoint.closure_semantic_hash),
    );
    push_entrypoint_verdict(&mut bytes, &entrypoint.closed_entry);
    push_dependency_closure(&mut bytes, &entrypoint.dependency_closure);
    for support in &entrypoint.required_runtime_support {
        push_runtime_support(&mut bytes, support);
    }
    push_argument_packaging(&mut bytes, &entrypoint.argument_packaging);
    push_result_observation(&mut bytes, &entrypoint.result_observation);
    push_trap_contract(&mut bytes, &entrypoint.trap_contract);
    push_len_str(
        &mut bytes,
        &format!(
            "{:016x}",
            entrypoint.report_contract.target_closure_identity
        ),
    );
    push_len_str(
        &mut bytes,
        &format!(
            "{:016x}",
            entrypoint.report_contract.target_closure_report_hash
        ),
    );
    push_len_str(&mut bytes, &entrypoint.report_contract.evidence_source);
    for (symbol, lanes) in &entrypoint.unsupported_lanes {
        push_len_str(&mut bytes, symbol);
        for lane in lanes {
            push_lane(&mut bytes, lane);
        }
    }
    bytes
}

fn canonical_runtime_entrypoint_package_bytes(
    package: &RuntimeExecutableEntrypointPackage,
) -> Vec<u8> {
    let mut out = String::new();
    push_field(&mut out, "kind", &package.header.package_kind);
    push_field(&mut out, "version", &package.header.version.to_string());
    push_field(&mut out, "producer", &package.header.producer);
    push_field(&mut out, "target", &package.header.target);
    push_field(
        &mut out,
        "runtime_package_identity",
        &package.runtime_artifact.package_identity,
    );
    push_field(
        &mut out,
        "runtime_core_semantic_hash",
        &package.runtime_artifact.core_semantic_hash.to_string(),
    );
    push_field(
        &mut out,
        "runtime_artifact_hash",
        &package.runtime_artifact.artifact_hash.to_string(),
    );
    push_field(
        &mut out,
        "runtime_report_hash",
        &package.runtime_report_hash.to_string(),
    );
    push_field(
        &mut out,
        "executable_contract_hash",
        &package.executable_contract_hash.to_string(),
    );
    push_field(
        &mut out,
        "entrypoint_metadata_identity",
        &package.entrypoint.metadata_identity.to_string(),
    );
    out.into_bytes()
}

fn push_entrypoint_verdict(bytes: &mut Vec<u8>, verdict: &ExecutableEntrypointVerdict) {
    match verdict {
        ExecutableEntrypointVerdict::ClosedKenOnly => push_len_str(bytes, "closed_ken_only"),
        ExecutableEntrypointVerdict::Unavailable { lanes } => {
            push_len_str(bytes, "unavailable");
            for lane in lanes {
                push_lane(bytes, lane);
            }
        }
    }
}

fn push_dependency_closure(bytes: &mut Vec<u8>, closure: &ExecutableDependencyClosure) {
    match closure {
        ExecutableDependencyClosure::ClosedKenOnly => push_len_str(bytes, "closed_ken_only"),
        ExecutableDependencyClosure::ImportsUnavailable {
            external_symbols,
            imported_declaration_refs,
        } => {
            push_len_str(bytes, "imports_unavailable");
            for symbol in external_symbols {
                push_len_str(bytes, symbol);
            }
            for (declaration, dependency) in imported_declaration_refs {
                push_len_str(bytes, declaration);
                push_len_str(bytes, dependency);
            }
        }
    }
}

fn push_argument_packaging(bytes: &mut Vec<u8>, packaging: &ExecutableArgumentPackaging) {
    match &packaging.shape {
        ExecutableArgumentShape::ClosedNullary => push_len_str(bytes, "closed_nullary"),
        ExecutableArgumentShape::ProcessInput {
            arguments,
            environment,
            working_directory,
        } => {
            push_len_str(bytes, "process_input");
            push_len_str(bytes, &arguments.len().to_string());
            for argument in arguments {
                push_len_bytes(bytes, argument);
            }
            push_len_str(bytes, &environment.len().to_string());
            for (key, value) in environment {
                push_len_bytes(bytes, key);
                push_len_bytes(bytes, value);
            }
            push_len_bytes(bytes, working_directory);
        }
        ExecutableArgumentShape::UnsupportedRuntimeArguments { parameter_count } => {
            push_len_str(bytes, "unsupported_runtime_arguments");
            push_len_str(bytes, &parameter_count.to_string());
        }
        ExecutableArgumentShape::Unavailable { lane } => {
            push_len_str(bytes, "unavailable");
            push_lane(bytes, lane);
        }
    }
    push_len_str(bytes, &packaging.evidence_source);
}

fn push_result_observation(bytes: &mut Vec<u8>, observation: &ExecutableResultObservation) {
    match &observation.shape {
        ExecutableResultShape::RuntimeValue => push_len_str(bytes, "runtime_value"),
        ExecutableResultShape::Unavailable { lane } => {
            push_len_str(bytes, "unavailable");
            push_lane(bytes, lane);
        }
    }
    push_len_str(bytes, &observation.evidence_source);
}

fn push_trap_contract(bytes: &mut Vec<u8>, contract: &ExecutableTrapContract) {
    match &contract.shape {
        ExecutableTrapShape::RuntimeTrapReport => push_len_str(bytes, "runtime_trap_report"),
        ExecutableTrapShape::Unavailable { lane } => {
            push_len_str(bytes, "unavailable");
            push_lane(bytes, lane);
        }
    }
    for (symbol, lanes) in &contract.blocking_lanes {
        push_len_str(bytes, symbol);
        for lane in lanes {
            push_lane(bytes, lane);
        }
    }
}

fn push_target_kind(bytes: &mut Vec<u8>, kind: &ExecutableEntrypointTargetKind) {
    let tag = match kind {
        ExecutableEntrypointTargetKind::Executable => "executable",
        ExecutableEntrypointTargetKind::Library => "library",
        ExecutableEntrypointTargetKind::NonRuntime => "non_runtime",
    };
    push_len_str(bytes, tag);
}

fn push_runtime_support(bytes: &mut Vec<u8>, support: &ExecutableRuntimeSupport) {
    let tag = match support {
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
    };
    push_len_str(bytes, tag);
}

fn push_lane(bytes: &mut Vec<u8>, lane: &ExecutableEntrypointUnavailableLane) {
    push_len_str(bytes, &lane.lane);
    push_len_str(bytes, &lane.reason);
}

fn push_len_str(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn push_len_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value);
}

fn push_field(out: &mut String, name: &str, value: &str) {
    out.push_str(name);
    out.push('=');
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push(';');
}

fn packaging_error(
    stage: ExecutableEntrypointPackagingStage,
    field: &'static str,
    reason: impl Into<String>,
) -> ExecutableEntrypointPackagingError {
    ExecutableEntrypointPackagingError {
        stage,
        field,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        executable_artifact_contract_for_runtime_report, summarize_runtime_ir_program,
        ErasedExecutableCore, RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr,
        RuntimeGroundValue, RuntimeLowerabilityStatus, RuntimeMetadata, RuntimeObservation,
        RuntimeSymbolMetadata, RuntimeValue,
    };

    fn pure_program() -> RuntimeProgram {
        let symbol = "decl:fixture::Entrypoint::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::entrypoint".to_string(),
            core_semantic_hash: 0x2001,
            artifact_hash: 0x2002,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol: symbol.clone(),
                kind: RuntimeDeclarationKind::Transparent {
                    body: RuntimeExpr::Value(RuntimeValue::Int((42).into())),
                },
                metadata: RuntimeSymbolMetadata {
                    lowerability: Some(RuntimeLowerabilityStatus::Supported),
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: vec![crate::RuntimeExample {
                name: "entrypoint-main".to_string(),
                checked_core_shape: "42".to_string(),
                ir: RuntimeExpr::DeclarationRef {
                    symbol: symbol.clone(),
                },
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((42).into())),
            }],
        }
    }

    fn blocked_program() -> RuntimeProgram {
        let symbol = "decl:fixture::Entrypoint::blocked".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata.lowerability.insert(
            symbol.clone(),
            RuntimeLowerabilityStatus::RequiresFeature {
                feature: "later-native-entrypoint".to_string(),
                reason: "outside executable packaging subset".to_string(),
            },
        );
        RuntimeProgram {
            package_identity: "module:fixture::entrypoint".to_string(),
            core_semantic_hash: 0x2001,
            artifact_hash: 0x2003,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol,
                kind: RuntimeDeclarationKind::MetadataOnly,
                metadata: RuntimeSymbolMetadata::empty(),
            }],
            examples: vec![],
        }
    }

    fn entrypoint_for(program: &RuntimeProgram) -> ExecutableEntrypointPackageMetadata {
        let target = program.declarations[0].symbol.clone();
        let mut entrypoint = ExecutableEntrypointPackageMetadata {
            package_identity: program.package_identity.clone(),
            package_core_semantic_hash: program.core_semantic_hash,
            package_artifact_hash: program.artifact_hash,
            target_symbol: target,
            target_kind: ExecutableEntrypointTargetKind::Executable,
            closure_identity: 0x2101,
            closure_semantic_hash: 0x2102,
            metadata_identity: 0,
            closed_entry: ExecutableEntrypointVerdict::ClosedKenOnly,
            dependency_closure: ExecutableDependencyClosure::ClosedKenOnly,
            required_runtime_support: BTreeSet::from([
                ExecutableRuntimeSupport::RuntimeValues,
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
                target_closure_identity: 0x2101,
                target_closure_report_hash: 0x2103,
                evidence_source: "target closure report".to_string(),
            },
            unsupported_lanes: BTreeMap::new(),
        };
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);
        entrypoint
    }

    fn runtime_contract(
        program: &RuntimeProgram,
    ) -> (RuntimeIrProgramReport, ExecutableArtifactContract) {
        let report = summarize_runtime_ir_program(program);
        let target = program.declarations[0].symbol.clone();
        let contract = executable_artifact_contract_for_runtime_report(
            program,
            &report,
            target,
            "entrypoint packaging unit test",
        )
        .expect("contract materializes");
        (report, contract)
    }

    #[test]
    fn runtime_entrypoint_package_binds_entrypoint_runtime_report_and_contract() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let entrypoint = entrypoint_for(&program);

        let package = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint.clone(),
            "ken-runtime unit test",
        )
        .expect("entrypoint package materializes");

        assert_eq!(package.header.target, entrypoint.target_symbol);
        assert_eq!(
            package.runtime_artifact,
            RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(
            package.runtime_report_hash,
            runtime_ir_program_report_hash(&report)
        );
        assert_eq!(
            package.executable_contract_hash,
            executable_artifact_contract_hash(&contract)
        );
        assert!(package.header.package_hash != 0);
        assert!(matches!(
            package.native_artifact_status,
            ExecutableNativeArtifactStatus::Unavailable { .. }
        ));
    }

    #[test]
    fn process_entrypoint_packaging_binds_raw_argv_environment_and_cwd() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let mut entrypoint = entrypoint_for(&program);
        entrypoint.argument_packaging = ExecutableArgumentPackaging {
            shape: ExecutableArgumentShape::ProcessInput {
                arguments: vec![b"ken".to_vec(), vec![0xff, 0x00]],
                environment: vec![(vec![0xfe], vec![0xfd])],
                working_directory: vec![b'/', 0xfc],
            },
            evidence_source: "raw process bytes staged by native runtime init".to_string(),
        };
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);

        let package = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime process packaging test",
        )
        .expect("process-shaped entrypoint package materializes");
        assert!(matches!(
            package.entrypoint.argument_packaging.shape,
            ExecutableArgumentShape::ProcessInput { .. }
        ));
        assert_ne!(package.header.package_hash, 0);
    }

    #[test]
    fn stale_entrypoint_package_identity_rejects() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let mut entrypoint = entrypoint_for(&program);
        entrypoint.package_artifact_hash ^= 1;
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("stale package artifact hash rejects");
        assert_eq!(
            err.stage,
            ExecutableEntrypointPackagingStage::EntrypointIdentity
        );
        assert_eq!(err.field, "package_artifact_hash");
    }

    #[test]
    fn stale_entrypoint_metadata_identity_rejects_after_hash_refresh_attack() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let mut entrypoint = entrypoint_for(&program);
        entrypoint.argument_packaging.evidence_source = "tampered".to_string();

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("stale metadata identity rejects");
        assert_eq!(err.stage, ExecutableEntrypointPackagingStage::Hash);
        assert_eq!(err.field, "metadata_identity");
    }

    #[test]
    fn mismatched_report_contract_closure_identity_rejects_after_hash_refresh() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let mut entrypoint = entrypoint_for(&program);
        entrypoint.report_contract.target_closure_identity ^= 1;
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("mismatched report-contract closure identity rejects");
        assert_eq!(
            err.stage,
            ExecutableEntrypointPackagingStage::EntrypointIdentity
        );
        assert_eq!(err.field, "report_contract.target_closure_identity");
    }

    #[test]
    fn non_closed_or_imported_entrypoint_rejects_before_native_work() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let mut entrypoint = entrypoint_for(&program);
        entrypoint.dependency_closure = ExecutableDependencyClosure::ImportsUnavailable {
            external_symbols: BTreeSet::from(["decl:dep::external".to_string()]),
            imported_declaration_refs: BTreeMap::new(),
        };
        entrypoint.closed_entry = ExecutableEntrypointVerdict::Unavailable {
            lanes: vec![ExecutableEntrypointUnavailableLane {
                lane: "non_closed_entrypoint".to_string(),
                reason: "entrypoint closure contains external symbols".to_string(),
            }],
        };
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("non-closed entrypoint rejects");
        assert_eq!(
            err.stage,
            ExecutableEntrypointPackagingStage::EntrypointClosed
        );
        assert_eq!(err.field, "closed_entry");
    }

    #[test]
    fn blocked_runtime_lowerability_rejects_even_with_closed_entrypoint_claim() {
        let program = blocked_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "entrypoint packaging unit test",
        )
        .expect("unsupported contract materializes");
        let entrypoint = entrypoint_for(&program);

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("runtime lowerability blocks packaging");
        assert_eq!(
            err.stage,
            ExecutableEntrypointPackagingStage::RuntimeBinding
        );
        assert_eq!(err.field, "lowerability");
    }

    #[test]
    fn target_absent_from_supported_report_rejects() {
        let program = pure_program();
        let (mut report, _) = runtime_contract(&program);
        let entrypoint = entrypoint_for(&program);
        report
            .supported_runtime_targets
            .remove(&entrypoint.target_symbol);
        let contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            entrypoint.target_symbol.clone(),
            "entrypoint packaging unit test",
        )
        .expect("contract materializes against mutated report");

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("report support mismatch rejects");
        assert_eq!(err.stage, ExecutableEntrypointPackagingStage::ReportBinding);
        assert_eq!(err.field, "supported_runtime_targets");
    }

    #[test]
    fn contract_target_mismatch_rejects_entrypoint_binding() {
        let program = pure_program();
        let (report, contract) = runtime_contract(&program);
        let mut entrypoint = entrypoint_for(&program);
        entrypoint.target_symbol = "decl:fixture::Entrypoint::other".to_string();
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);

        let err = executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-runtime unit test",
        )
        .expect_err("contract target mismatch rejects");
        assert_eq!(
            err.stage,
            ExecutableEntrypointPackagingStage::EntrypointIdentity
        );
        assert_eq!(err.field, "target_symbol");
    }
}
