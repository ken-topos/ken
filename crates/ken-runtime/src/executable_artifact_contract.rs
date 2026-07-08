//! Executable-artifact contract envelope above `RuntimeProgram`.
//!
//! This is the NC19 v0 materialization of
//! `spec/40-runtime/48-executable-artifact-contract.md`: checked-core and
//! runtime IR remain the semantic authorities, while native, toolchain, object,
//! linker, ABI, interop, and proof lanes are explicit evidence or explicit
//! unavailability. The contract never executes native code and never treats
//! native facts as proof evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    fnv1a_64, RuntimeArtifactIdentity, RuntimeIrNativePhaseGate, RuntimeIrProgramReport,
    RuntimeProgram, RuntimeSymbol,
};

pub const EXECUTABLE_ARTIFACT_CONTRACT_KIND: &str = "KenExecutableArtifactContract";
pub const EXECUTABLE_ARTIFACT_CONTRACT_VERSION: u32 = 0;
pub const EXECUTABLE_ARTIFACT_CONTRACT_SPEC_REF: &str =
    "spec/40-runtime/48-executable-artifact-contract.md";
pub const RUNTIME_IR_PROGRAM_REPORT_KIND: &str = "RuntimeIrProgramReport";
pub const CHECKED_CORE_PACKAGE_KIND: &str = "CheckedCorePackage";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableArtifactContract {
    pub header: ExecutableArtifactContractHeader,
    pub checked_core: ExecutableCheckedCoreBinding,
    pub runtime: ExecutableRuntimeBinding,
    pub report: ExecutableReportBinding,
    pub native_artifact: ExecutableNativeArtifactBinding,
    pub toolchain: ExecutableToolchainBinding,
    pub required_unavailable_lanes: BTreeMap<ExecutableUnavailableLane, ExplicitUnavailableMarker>,
    pub compatibility: ExecutableContractCompatibility,
    pub unknown_semantic_fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableArtifactContractHeader {
    pub contract_kind: String,
    pub version: u32,
    pub producer: String,
    pub spec_ref: String,
    pub target: RuntimeSymbol,
    pub contract_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableCheckedCoreBinding {
    pub package_kind: String,
    pub version: u32,
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub dependency_semantic_hashes: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableRuntimeBinding {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub selected_target: RuntimeSymbol,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableReportBinding {
    pub report_kind: String,
    pub report_hash: u64,
    pub artifact: RuntimeArtifactIdentity,
    pub selected_target: RuntimeSymbol,
    pub selected_target_verdict: ExecutableReportTargetVerdict,
    pub native_phase_gate: RuntimeIrNativePhaseGate,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableReportTargetVerdict {
    SupportedRuntimeTarget,
    ComparisonUnavailable { reason: String },
    Unsupported { reason: String },
    AbsentFromReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableNativeArtifactBinding {
    pub status: ExecutableNativeArtifactStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableNativeArtifactStatus {
    Available {
        kind: String,
        artifact_hash: Option<u64>,
        backend_name: String,
        platform_target: String,
        evidence_source: String,
        produced_from: ExecutableArtifactProducedFrom,
        evidence_lane: ExecutableEvidenceLane,
    },
    Unavailable {
        marker: Option<ExplicitUnavailableMarker>,
    },
    Unsupported {
        marker: Option<ExplicitUnsupportedMarker>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableArtifactProducedFrom {
    pub checked_core: ExecutableProducedCheckedCoreIdentity,
    pub runtime: RuntimeArtifactIdentity,
    pub report_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableProducedCheckedCoreIdentity {
    pub package_identity: String,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableToolchainBinding {
    pub ken_runtime: ExecutableEvidenceFact,
    pub native_backend: ExecutableEvidenceFact,
    pub backend_verifier: ExecutableEvidenceFact,
    pub host_platform: ExecutableEvidenceFact,
    pub object_emission: ExecutableEvidenceFact,
    pub linker_or_finalizer: ExecutableEvidenceFact,
    pub provenance_or_build_attestation: ExecutableEvidenceFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableEvidenceFact {
    Available {
        value: String,
        evidence_source: String,
        evidence_lane: ExecutableEvidenceLane,
    },
    Unavailable {
        marker: ExplicitUnavailableMarker,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplicitUnavailableMarker {
    pub lane: ExecutableUnavailableLane,
    pub reason: String,
    pub evidence_lane: ExecutableEvidenceLane,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplicitUnsupportedMarker {
    pub lane: ExecutableUnsupportedLane,
    pub target: RuntimeSymbol,
    pub construct: String,
    pub reason: String,
    pub evidence_lane: ExecutableEvidenceLane,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutableUnavailableLane {
    NativeExecutableArtifact,
    ObjectEmission,
    LinkerOrFinalizer,
    LibraryAbi,
    CAbi,
    RustInterop,
    CrossPackageNativeLinking,
    StableForeignAbi,
    HostEffectOrFfiExecution,
    WholeCompilerProof,
    NativeBackendIdentity,
    BackendVerifierIdentity,
    HostPlatformTarget,
    ProvenanceOrBuildAttestation,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutableUnsupportedLane {
    RuntimeIrNativePhaseGate,
    RuntimeIrTarget,
    RuntimeIrConstruct,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableEvidenceLane {
    SemanticAuthority,
    Tested,
    Validated,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableContractCompatibility {
    pub rule: ExecutableContractCompatibilityRule,
    pub version: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableContractCompatibilityRule {
    PreserveV0,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableArtifactContractReport {
    pub contract_hash: u64,
    pub target: RuntimeSymbol,
    pub runtime_artifact: RuntimeArtifactIdentity,
    pub report_hash: u64,
    pub native_artifact_status: ExecutableNativeArtifactStatus,
    pub unavailable_lanes: BTreeSet<ExecutableUnavailableLane>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableArtifactContractError {
    pub stage: ExecutableArtifactContractStage,
    pub field: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableArtifactContractStage {
    Header,
    ClosedSchema,
    CheckedCoreBinding,
    RuntimeBinding,
    ReportBinding,
    NativeArtifactBinding,
    ToolchainBinding,
    UnavailableLane,
    Hash,
}

impl fmt::Display for ExecutableArtifactContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.field, self.reason)
    }
}

impl std::error::Error for ExecutableArtifactContractError {}

pub fn executable_artifact_contract_for_runtime_report(
    program: &RuntimeProgram,
    report: &RuntimeIrProgramReport,
    target: impl Into<RuntimeSymbol>,
    producer: impl Into<String>,
) -> Result<ExecutableArtifactContract, ExecutableArtifactContractError> {
    let target = target.into();
    let report_hash = runtime_ir_program_report_hash(report);
    let checked_core = ExecutableCheckedCoreBinding {
        package_kind: CHECKED_CORE_PACKAGE_KIND.to_string(),
        version: 0,
        package_identity: program.package_identity.clone(),
        core_semantic_hash: program.core_semantic_hash,
        artifact_hash: program.artifact_hash,
        dependency_semantic_hashes: program
            .erased_core
            .metadata
            .dependency_semantic_hashes
            .clone(),
    };
    let runtime = ExecutableRuntimeBinding {
        package_identity: program.package_identity.clone(),
        core_semantic_hash: program.core_semantic_hash,
        artifact_hash: program.artifact_hash,
        selected_target: target.clone(),
        evidence_source:
            "RuntimeProgram package/core/artifact identity from the exact runtime artifact"
                .to_string(),
    };
    let report_binding = ExecutableReportBinding {
        report_kind: RUNTIME_IR_PROGRAM_REPORT_KIND.to_string(),
        report_hash,
        artifact: report.artifact.clone(),
        selected_target: target.clone(),
        selected_target_verdict: target_verdict(report, &target),
        native_phase_gate: report.native_phase_gate.clone(),
        evidence_source: "RuntimeIrProgramReport produced for the exact runtime artifact"
            .to_string(),
    };
    let native_artifact = ExecutableNativeArtifactBinding {
        status: native_status_from_report(report, &target),
    };
    let mut contract = ExecutableArtifactContract {
        header: ExecutableArtifactContractHeader {
            contract_kind: EXECUTABLE_ARTIFACT_CONTRACT_KIND.to_string(),
            version: EXECUTABLE_ARTIFACT_CONTRACT_VERSION,
            producer: producer.into(),
            spec_ref: EXECUTABLE_ARTIFACT_CONTRACT_SPEC_REF.to_string(),
            target,
            contract_hash: 0,
        },
        checked_core,
        runtime,
        report: report_binding,
        native_artifact,
        toolchain: default_toolchain_binding(),
        required_unavailable_lanes: required_unavailable_lanes(),
        compatibility: ExecutableContractCompatibility {
            rule: ExecutableContractCompatibilityRule::PreserveV0,
            version: EXECUTABLE_ARTIFACT_CONTRACT_VERSION,
        },
        unknown_semantic_fields: BTreeMap::new(),
    };
    contract.header.contract_hash = executable_artifact_contract_hash(&contract);
    validate_executable_artifact_contract(program, report, &contract)?;
    Ok(contract)
}

pub fn validate_executable_artifact_contract(
    program: &RuntimeProgram,
    report: &RuntimeIrProgramReport,
    contract: &ExecutableArtifactContract,
) -> Result<ExecutableArtifactContractReport, ExecutableArtifactContractError> {
    validate_header(contract)?;
    validate_closed_schema(contract)?;
    validate_checked_core_binding(program, contract)?;
    validate_runtime_binding(program, contract)?;
    validate_report_binding(program, report, contract)?;
    validate_native_artifact_binding(report, contract)?;
    validate_toolchain_binding(contract)?;
    validate_required_unavailable_lanes(contract)?;
    validate_contract_hash(contract)?;

    Ok(ExecutableArtifactContractReport {
        contract_hash: contract.header.contract_hash,
        target: contract.header.target.clone(),
        runtime_artifact: RuntimeArtifactIdentity::from_program(program),
        report_hash: contract.report.report_hash,
        native_artifact_status: contract.native_artifact.status.clone(),
        unavailable_lanes: contract
            .required_unavailable_lanes
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
    })
}

pub fn executable_artifact_contract_hash(contract: &ExecutableArtifactContract) -> u64 {
    fnv1a_64(&canonical_contract_bytes(contract))
}

pub fn runtime_ir_program_report_hash(report: &RuntimeIrProgramReport) -> u64 {
    let mut out = String::new();
    push_field(&mut out, "kind", RUNTIME_IR_PROGRAM_REPORT_KIND);
    push_field(
        &mut out,
        "package_identity",
        &report.artifact.package_identity,
    );
    push_field(
        &mut out,
        "core_semantic_hash",
        &report.artifact.core_semantic_hash.to_string(),
    );
    push_field(
        &mut out,
        "runtime_artifact_hash",
        &report.artifact.artifact_hash.to_string(),
    );
    push_debug(
        &mut out,
        "supported_runtime_targets",
        &report.supported_runtime_targets,
    );
    push_debug(
        &mut out,
        "comparison_unavailable_targets",
        &report.comparison_unavailable_targets,
    );
    push_debug(&mut out, "unsupported_targets", &report.unsupported_targets);
    push_debug(&mut out, "evidence_sources", &report.evidence_sources);
    push_debug(&mut out, "unavailable", &report.unavailable);
    push_debug(&mut out, "native_phase_gate", &report.native_phase_gate);
    fnv1a_64(out.as_bytes())
}

fn validate_header(
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    if contract.header.contract_kind != EXECUTABLE_ARTIFACT_CONTRACT_KIND {
        return Err(contract_error(
            ExecutableArtifactContractStage::Header,
            "contract_kind",
            "unsupported executable-artifact contract kind",
        ));
    }
    if contract.header.version != EXECUTABLE_ARTIFACT_CONTRACT_VERSION {
        return Err(contract_error(
            ExecutableArtifactContractStage::Header,
            "version",
            "unsupported executable-artifact contract version",
        ));
    }
    if contract.header.producer.is_empty() {
        return Err(contract_error(
            ExecutableArtifactContractStage::Header,
            "producer",
            "producer must be explicit",
        ));
    }
    if contract.header.spec_ref != EXECUTABLE_ARTIFACT_CONTRACT_SPEC_REF {
        return Err(contract_error(
            ExecutableArtifactContractStage::Header,
            "spec_ref",
            "contract does not name the v0 executable-artifact specification",
        ));
    }
    if contract.header.target.is_empty() {
        return Err(contract_error(
            ExecutableArtifactContractStage::Header,
            "target",
            "selected target identity must be explicit",
        ));
    }
    Ok(())
}

fn validate_closed_schema(
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    if let Some((field, _)) = contract.unknown_semantic_fields.iter().next() {
        return Err(contract_error(
            ExecutableArtifactContractStage::ClosedSchema,
            "unknown_semantic_fields",
            format!("unknown semantic field {field:?} is not allowed in v0"),
        ));
    }
    Ok(())
}

fn validate_checked_core_binding(
    program: &RuntimeProgram,
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    if contract.checked_core.package_kind != CHECKED_CORE_PACKAGE_KIND {
        return Err(contract_error(
            ExecutableArtifactContractStage::CheckedCoreBinding,
            "package_kind",
            "checked-core binding must name CheckedCorePackage",
        ));
    }
    if contract.checked_core.version != 0 {
        return Err(contract_error(
            ExecutableArtifactContractStage::CheckedCoreBinding,
            "version",
            "unsupported checked-core package version",
        ));
    }
    if contract.checked_core.package_identity != program.package_identity {
        return Err(contract_error(
            ExecutableArtifactContractStage::CheckedCoreBinding,
            "package_identity",
            "checked-core package identity does not match the RuntimeProgram",
        ));
    }
    if contract.checked_core.core_semantic_hash != program.core_semantic_hash {
        return Err(contract_error(
            ExecutableArtifactContractStage::CheckedCoreBinding,
            "core_semantic_hash",
            "checked-core semantic hash does not match the RuntimeProgram",
        ));
    }
    if contract.checked_core.artifact_hash != program.artifact_hash {
        return Err(contract_error(
            ExecutableArtifactContractStage::CheckedCoreBinding,
            "artifact_hash",
            "checked-core artifact hash does not match the RuntimeProgram artifact hash",
        ));
    }
    if contract.checked_core.dependency_semantic_hashes
        != program.erased_core.metadata.dependency_semantic_hashes
    {
        return Err(contract_error(
            ExecutableArtifactContractStage::CheckedCoreBinding,
            "dependency_semantic_hashes",
            "checked-core dependency semantic hashes do not match the RuntimeProgram",
        ));
    }
    Ok(())
}

fn validate_runtime_binding(
    program: &RuntimeProgram,
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if contract.runtime.package_identity != artifact.package_identity {
        return Err(contract_error(
            ExecutableArtifactContractStage::RuntimeBinding,
            "package_identity",
            "runtime package identity does not match the exact RuntimeProgram",
        ));
    }
    if contract.runtime.core_semantic_hash != artifact.core_semantic_hash {
        return Err(contract_error(
            ExecutableArtifactContractStage::RuntimeBinding,
            "core_semantic_hash",
            "runtime semantic hash does not match the exact RuntimeProgram",
        ));
    }
    if contract.runtime.artifact_hash != artifact.artifact_hash {
        return Err(contract_error(
            ExecutableArtifactContractStage::RuntimeBinding,
            "artifact_hash",
            "runtime artifact hash does not match the exact RuntimeProgram",
        ));
    }
    if contract.runtime.package_identity != contract.checked_core.package_identity
        || contract.runtime.core_semantic_hash != contract.checked_core.core_semantic_hash
    {
        return Err(contract_error(
            ExecutableArtifactContractStage::RuntimeBinding,
            "checked_core_runtime_identity",
            "runtime binding must match checked-core package identity and semantic hash",
        ));
    }
    if contract.runtime.selected_target != contract.header.target {
        return Err(contract_error(
            ExecutableArtifactContractStage::RuntimeBinding,
            "selected_target",
            "runtime selected target does not match the contract header target",
        ));
    }
    Ok(())
}

fn validate_report_binding(
    program: &RuntimeProgram,
    report: &RuntimeIrProgramReport,
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    if contract.report.report_kind != RUNTIME_IR_PROGRAM_REPORT_KIND {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "report_kind",
            "unsupported runtime-IR report kind",
        ));
    }
    let report_hash = runtime_ir_program_report_hash(report);
    if contract.report.report_hash != report_hash {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "report_hash",
            "runtime-IR report hash is stale",
        ));
    }
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if report.artifact != artifact {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "report_artifact",
            "RuntimeIrProgramReport artifact identity does not match the RuntimeProgram",
        ));
    }
    if contract.report.artifact != artifact {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "artifact",
            "contract report artifact identity does not match the RuntimeProgram",
        ));
    }
    if contract.report.selected_target != contract.header.target {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "selected_target",
            "report selected target does not match the contract header target",
        ));
    }
    if contract.report.selected_target_verdict != target_verdict(report, &contract.header.target) {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "selected_target_verdict",
            "selected target verdict is stale relative to the RuntimeIrProgramReport",
        ));
    }
    if contract.report.native_phase_gate != report.native_phase_gate {
        return Err(contract_error(
            ExecutableArtifactContractStage::ReportBinding,
            "native_phase_gate",
            "native phase gate is stale relative to the RuntimeIrProgramReport",
        ));
    }
    Ok(())
}

fn validate_native_artifact_binding(
    report: &RuntimeIrProgramReport,
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    match &contract.native_artifact.status {
        ExecutableNativeArtifactStatus::Available {
            artifact_hash,
            produced_from,
            evidence_lane,
            ..
        } => {
            if *evidence_lane == ExecutableEvidenceLane::SemanticAuthority {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "evidence_lane",
                    "native artifact evidence cannot be semantic authority",
                ));
            }
            if artifact_hash.is_none() {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "artifact_hash",
                    "available native artifact is missing its native artifact hash",
                ));
            }
            if produced_from.checked_core.package_identity != contract.checked_core.package_identity
                || produced_from.checked_core.core_semantic_hash
                    != contract.checked_core.core_semantic_hash
                || produced_from.checked_core.artifact_hash != contract.checked_core.artifact_hash
            {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "produced_from.checked_core",
                    "native artifact is not bound to the checked-core identity",
                ));
            }
            if produced_from.runtime
                != (RuntimeArtifactIdentity {
                    package_identity: contract.runtime.package_identity.clone(),
                    core_semantic_hash: contract.runtime.core_semantic_hash,
                    artifact_hash: contract.runtime.artifact_hash,
                })
            {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "produced_from.runtime",
                    "native artifact is not bound to the runtime artifact identity",
                ));
            }
            if produced_from.report_hash != contract.report.report_hash {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "produced_from.report_hash",
                    "native artifact is not bound to the runtime-IR report hash",
                ));
            }
            if matches!(
                report.native_phase_gate,
                RuntimeIrNativePhaseGate::Blocked { .. }
            ) {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "native_phase_gate",
                    "blocked RuntimeIrNativePhaseGate cannot be restated as executable-ready",
                ));
            }
            if !report
                .supported_runtime_targets
                .contains(&contract.header.target)
            {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "supported_runtime_targets",
                    "selected target is absent from supported_runtime_targets",
                ));
            }
        }
        ExecutableNativeArtifactStatus::Unavailable { marker } => {
            let marker = marker.as_ref().ok_or_else(|| {
                contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "unavailable_marker",
                    "native artifact unavailability must be explicit",
                )
            })?;
            validate_unavailable_marker(marker)?;
        }
        ExecutableNativeArtifactStatus::Unsupported { marker } => {
            let marker = marker.as_ref().ok_or_else(|| {
                contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "unsupported_marker",
                    "native artifact unsupported status must be explicit",
                )
            })?;
            if marker.evidence_lane != ExecutableEvidenceLane::Unsupported {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "unsupported_marker",
                    "unsupported marker must stay in the unsupported evidence lane",
                ));
            }
            if marker.target != contract.header.target {
                return Err(contract_error(
                    ExecutableArtifactContractStage::NativeArtifactBinding,
                    "unsupported_marker.target",
                    "unsupported marker target must match the contract target",
                ));
            }
        }
    }
    Ok(())
}

fn validate_toolchain_binding(
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    for (name, fact) in [
        ("ken_runtime", &contract.toolchain.ken_runtime),
        ("native_backend", &contract.toolchain.native_backend),
        ("backend_verifier", &contract.toolchain.backend_verifier),
        ("host_platform", &contract.toolchain.host_platform),
        ("object_emission", &contract.toolchain.object_emission),
        (
            "linker_or_finalizer",
            &contract.toolchain.linker_or_finalizer,
        ),
        (
            "provenance_or_build_attestation",
            &contract.toolchain.provenance_or_build_attestation,
        ),
    ] {
        match fact {
            ExecutableEvidenceFact::Available { evidence_lane, .. } => {
                if *evidence_lane == ExecutableEvidenceLane::SemanticAuthority {
                    return Err(contract_error(
                        ExecutableArtifactContractStage::ToolchainBinding,
                        name,
                        "toolchain facts cannot be semantic authority",
                    ));
                }
            }
            ExecutableEvidenceFact::Unavailable { marker } => validate_unavailable_marker(marker)?,
        }
    }
    for (name, fact) in [
        ("object_emission", &contract.toolchain.object_emission),
        (
            "linker_or_finalizer",
            &contract.toolchain.linker_or_finalizer,
        ),
    ] {
        if matches!(fact, ExecutableEvidenceFact::Available { .. }) {
            return Err(contract_error(
                ExecutableArtifactContractStage::ToolchainBinding,
                name,
                "object/linker/finalizer lanes must remain unavailable in v0",
            ));
        }
    }
    Ok(())
}

fn validate_required_unavailable_lanes(
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    for lane in required_unavailable_lanes().keys() {
        let marker = contract
            .required_unavailable_lanes
            .get(lane)
            .ok_or_else(|| {
                contract_error(
                    ExecutableArtifactContractStage::UnavailableLane,
                    "required_unavailable_lanes",
                    format!("missing required unavailable marker for {lane:?}"),
                )
            })?;
        if marker.lane != *lane {
            return Err(contract_error(
                ExecutableArtifactContractStage::UnavailableLane,
                "required_unavailable_lanes",
                format!("unavailable marker is stored under the wrong lane: {lane:?}"),
            ));
        }
        validate_unavailable_marker(marker)?;
    }
    Ok(())
}

fn validate_contract_hash(
    contract: &ExecutableArtifactContract,
) -> Result<(), ExecutableArtifactContractError> {
    let expected = executable_artifact_contract_hash(contract);
    if contract.header.contract_hash != expected {
        return Err(contract_error(
            ExecutableArtifactContractStage::Hash,
            "contract_hash",
            format!(
                "contract hash {} does not match recomputed hash {expected}",
                contract.header.contract_hash
            ),
        ));
    }
    Ok(())
}

fn validate_unavailable_marker(
    marker: &ExplicitUnavailableMarker,
) -> Result<(), ExecutableArtifactContractError> {
    if marker.evidence_lane != ExecutableEvidenceLane::Unavailable {
        return Err(contract_error(
            ExecutableArtifactContractStage::UnavailableLane,
            "evidence_lane",
            "unavailable marker must stay in the unavailable evidence lane",
        ));
    }
    if marker.reason.is_empty() {
        return Err(contract_error(
            ExecutableArtifactContractStage::UnavailableLane,
            "reason",
            "unavailable marker requires a reason",
        ));
    }
    Ok(())
}

fn native_status_from_report(
    report: &RuntimeIrProgramReport,
    target: &RuntimeSymbol,
) -> ExecutableNativeArtifactStatus {
    if let Some(reason) = report.unsupported_targets.get(target) {
        return ExecutableNativeArtifactStatus::Unsupported {
            marker: Some(ExplicitUnsupportedMarker {
                lane: ExecutableUnsupportedLane::RuntimeIrTarget,
                target: target.clone(),
                construct: "RuntimeIrProgramReport.unsupported_targets".to_string(),
                reason: reason.clone(),
                evidence_lane: ExecutableEvidenceLane::Unsupported,
            }),
        };
    }
    if let RuntimeIrNativePhaseGate::Blocked { blockers } = &report.native_phase_gate {
        return ExecutableNativeArtifactStatus::Unsupported {
            marker: Some(ExplicitUnsupportedMarker {
                lane: ExecutableUnsupportedLane::RuntimeIrNativePhaseGate,
                target: target.clone(),
                construct: "RuntimeIrNativePhaseGate::Blocked".to_string(),
                reason: blockers
                    .iter()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "native phase gate is blocked".to_string()),
                evidence_lane: ExecutableEvidenceLane::Unsupported,
            }),
        };
    }
    if let Some(reason) = report.comparison_unavailable_targets.get(target) {
        return ExecutableNativeArtifactStatus::Unavailable {
            marker: Some(unavailable_marker(
                ExecutableUnavailableLane::HostEffectOrFfiExecution,
                reason,
            )),
        };
    }
    ExecutableNativeArtifactStatus::Unavailable {
        marker: Some(unavailable_marker(
            ExecutableUnavailableLane::NativeExecutableArtifact,
            "native executable bytes are not emitted by the v0 contract materialization",
        )),
    }
}

fn target_verdict(
    report: &RuntimeIrProgramReport,
    target: &RuntimeSymbol,
) -> ExecutableReportTargetVerdict {
    if report.supported_runtime_targets.contains(target) {
        ExecutableReportTargetVerdict::SupportedRuntimeTarget
    } else if let Some(reason) = report.comparison_unavailable_targets.get(target) {
        ExecutableReportTargetVerdict::ComparisonUnavailable {
            reason: reason.clone(),
        }
    } else if let Some(reason) = report.unsupported_targets.get(target) {
        ExecutableReportTargetVerdict::Unsupported {
            reason: reason.clone(),
        }
    } else {
        ExecutableReportTargetVerdict::AbsentFromReport
    }
}

fn default_toolchain_binding() -> ExecutableToolchainBinding {
    ExecutableToolchainBinding {
        ken_runtime: ExecutableEvidenceFact::Available {
            value: format!("ken-runtime {}", env!("CARGO_PKG_VERSION")),
            evidence_source: "compiled ken-runtime crate version embedded by Cargo".to_string(),
            evidence_lane: ExecutableEvidenceLane::Tested,
        },
        native_backend: ExecutableEvidenceFact::Unavailable {
            marker: unavailable_marker(
                ExecutableUnavailableLane::NativeBackendIdentity,
                "native backend exact-run identity is not captured by this contract materialization",
            ),
        },
        backend_verifier: ExecutableEvidenceFact::Unavailable {
            marker: unavailable_marker(
                ExecutableUnavailableLane::BackendVerifierIdentity,
                "backend verifier exact-run identity is unavailable in this phase",
            ),
        },
        host_platform: ExecutableEvidenceFact::Unavailable {
            marker: unavailable_marker(
                ExecutableUnavailableLane::HostPlatformTarget,
                "host platform target is not captured from an exact native run",
            ),
        },
        object_emission: ExecutableEvidenceFact::Unavailable {
            marker: unavailable_marker(
                ExecutableUnavailableLane::ObjectEmission,
                "object emission is outside the v0 executable-artifact contract",
            ),
        },
        linker_or_finalizer: ExecutableEvidenceFact::Unavailable {
            marker: unavailable_marker(
                ExecutableUnavailableLane::LinkerOrFinalizer,
                "linker/finalizer behavior is outside the v0 executable-artifact contract",
            ),
        },
        provenance_or_build_attestation: ExecutableEvidenceFact::Unavailable {
            marker: unavailable_marker(
                ExecutableUnavailableLane::ProvenanceOrBuildAttestation,
                "build provenance attestation is not captured by this runtime-only materialization",
            ),
        },
    }
}

fn required_unavailable_lanes() -> BTreeMap<ExecutableUnavailableLane, ExplicitUnavailableMarker> {
    BTreeMap::from([
        (
            ExecutableUnavailableLane::ObjectEmission,
            unavailable_marker(
                ExecutableUnavailableLane::ObjectEmission,
                "object emission is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::LinkerOrFinalizer,
            unavailable_marker(
                ExecutableUnavailableLane::LinkerOrFinalizer,
                "linker/finalizer behavior is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::LibraryAbi,
            unavailable_marker(
                ExecutableUnavailableLane::LibraryAbi,
                "library ABI is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::CAbi,
            unavailable_marker(
                ExecutableUnavailableLane::CAbi,
                "C ABI is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::RustInterop,
            unavailable_marker(
                ExecutableUnavailableLane::RustInterop,
                "Rust interop is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::CrossPackageNativeLinking,
            unavailable_marker(
                ExecutableUnavailableLane::CrossPackageNativeLinking,
                "cross-package native linking is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::StableForeignAbi,
            unavailable_marker(
                ExecutableUnavailableLane::StableForeignAbi,
                "stable foreign ABI is unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::HostEffectOrFfiExecution,
            unavailable_marker(
                ExecutableUnavailableLane::HostEffectOrFfiExecution,
                "host-effect and FFI execution are unavailable in v0",
            ),
        ),
        (
            ExecutableUnavailableLane::WholeCompilerProof,
            unavailable_marker(
                ExecutableUnavailableLane::WholeCompilerProof,
                "whole-compiler proof is unavailable in v0",
            ),
        ),
    ])
}

fn unavailable_marker(
    lane: ExecutableUnavailableLane,
    reason: impl Into<String>,
) -> ExplicitUnavailableMarker {
    ExplicitUnavailableMarker {
        lane,
        reason: reason.into(),
        evidence_lane: ExecutableEvidenceLane::Unavailable,
    }
}

fn canonical_contract_bytes(contract: &ExecutableArtifactContract) -> Vec<u8> {
    let mut out = String::new();
    push_field(&mut out, "contract_kind", &contract.header.contract_kind);
    push_field(&mut out, "version", &contract.header.version.to_string());
    push_field(&mut out, "producer", &contract.header.producer);
    push_field(&mut out, "spec_ref", &contract.header.spec_ref);
    push_field(&mut out, "target", &contract.header.target);
    push_debug(&mut out, "checked_core", &contract.checked_core);
    push_debug(&mut out, "runtime", &contract.runtime);
    push_debug(&mut out, "report", &contract.report);
    push_debug(&mut out, "native_artifact", &contract.native_artifact);
    push_debug(&mut out, "toolchain", &contract.toolchain);
    push_debug(
        &mut out,
        "required_unavailable_lanes",
        &contract.required_unavailable_lanes,
    );
    push_debug(&mut out, "compatibility", &contract.compatibility);
    push_debug(
        &mut out,
        "unknown_semantic_fields",
        &contract.unknown_semantic_fields,
    );
    out.into_bytes()
}

fn push_field(out: &mut String, name: &str, value: &str) {
    out.push_str(name);
    out.push('=');
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push(';');
}

fn push_debug<T: fmt::Debug>(out: &mut String, name: &str, value: &T) {
    push_field(out, name, &format!("{value:?}"));
}

fn contract_error(
    stage: ExecutableArtifactContractStage,
    field: &'static str,
    reason: impl Into<String>,
) -> ExecutableArtifactContractError {
    ExecutableArtifactContractError {
        stage,
        field,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        summarize_runtime_ir_program, ErasedExecutableCore, RuntimeDeclaration,
        RuntimeDeclarationKind, RuntimeExpr, RuntimeGroundValue, RuntimeLowerabilityStatus,
        RuntimeMetadata, RuntimeObservation, RuntimeSymbolMetadata, RuntimeValue,
    };

    fn pure_program() -> RuntimeProgram {
        let symbol = "decl:fixture::Main::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::nc19".to_string(),
            core_semantic_hash: 0x1901,
            artifact_hash: 0x1902,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from([symbol.clone()]),
                metadata,
            },
            declarations: vec![RuntimeDeclaration {
                symbol: symbol.clone(),
                kind: RuntimeDeclarationKind::Transparent {
                    body: RuntimeExpr::Value(RuntimeValue::Int(42)),
                },
                metadata: RuntimeSymbolMetadata {
                    lowerability: Some(RuntimeLowerabilityStatus::Supported),
                    ..RuntimeSymbolMetadata::empty()
                },
            }],
            examples: vec![crate::RuntimeExample {
                name: "nc19-main".to_string(),
                checked_core_shape: "42".to_string(),
                ir: RuntimeExpr::DeclarationRef { symbol },
                observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(42)),
            }],
        }
    }

    fn blocked_program() -> RuntimeProgram {
        let symbol = "decl:fixture::Main::blocked".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata.lowerability.insert(
            symbol.clone(),
            RuntimeLowerabilityStatus::RequiresFeature {
                feature: "later-native-shape".to_string(),
                reason: "outside the starter executable subset".to_string(),
            },
        );
        RuntimeProgram {
            package_identity: "module:fixture::nc19".to_string(),
            core_semantic_hash: 0x1901,
            artifact_hash: 0x1903,
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

    fn available_native_status(
        contract: &ExecutableArtifactContract,
    ) -> ExecutableNativeArtifactStatus {
        ExecutableNativeArtifactStatus::Available {
            kind: "KenNativeExecutable".to_string(),
            artifact_hash: Some(0xabc),
            backend_name: "test-backend".to_string(),
            platform_target: "test-platform".to_string(),
            evidence_source: "test exact-run native evidence".to_string(),
            produced_from: ExecutableArtifactProducedFrom {
                checked_core: ExecutableProducedCheckedCoreIdentity {
                    package_identity: contract.checked_core.package_identity.clone(),
                    core_semantic_hash: contract.checked_core.core_semantic_hash,
                    artifact_hash: contract.checked_core.artifact_hash,
                },
                runtime: RuntimeArtifactIdentity {
                    package_identity: contract.runtime.package_identity.clone(),
                    core_semantic_hash: contract.runtime.core_semantic_hash,
                    artifact_hash: contract.runtime.artifact_hash,
                },
                report_hash: contract.report.report_hash,
            },
            evidence_lane: ExecutableEvidenceLane::Tested,
        }
    }

    fn refresh_hash(contract: &mut ExecutableArtifactContract) {
        contract.header.contract_hash = executable_artifact_contract_hash(contract);
    }

    #[test]
    fn valid_contract_binds_runtime_report_and_explicit_unavailable_lanes() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target.clone(),
            "ken-runtime unit test",
        )
        .expect("valid contract");

        let validated =
            validate_executable_artifact_contract(&program, &report, &contract).unwrap();
        assert_eq!(validated.target, target);
        assert_eq!(
            validated.runtime_artifact,
            RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(
            validated.report_hash,
            runtime_ir_program_report_hash(&report)
        );
        assert!(matches!(
            validated.native_artifact_status,
            ExecutableNativeArtifactStatus::Unavailable { .. }
        ));
        for lane in required_unavailable_lanes().keys() {
            assert!(validated.unavailable_lanes.contains(lane));
        }
    }

    #[test]
    fn stale_runtime_identity_rejects_before_native_status() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        contract.runtime.artifact_hash ^= 1;
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("stale runtime hash rejects");
        assert_eq!(err.stage, ExecutableArtifactContractStage::RuntimeBinding);
        assert_eq!(err.field, "artifact_hash");
    }

    #[test]
    fn stale_report_identity_rejects() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        contract.report.report_hash ^= 1;
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("stale report hash rejects");
        assert_eq!(err.stage, ExecutableArtifactContractStage::ReportBinding);
        assert_eq!(err.field, "report_hash");
    }

    #[test]
    fn available_native_artifact_accepts_only_with_exact_ready_bindings() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        contract.native_artifact.status = available_native_status(&contract);
        refresh_hash(&mut contract);

        let validated =
            validate_executable_artifact_contract(&program, &report, &contract).unwrap();
        assert!(matches!(
            validated.native_artifact_status,
            ExecutableNativeArtifactStatus::Available { .. }
        ));
    }

    #[test]
    fn blocked_gate_rejects_available_native_artifact_claim() {
        let program = blocked_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        contract.native_artifact.status = available_native_status(&contract);
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("blocked gate cannot become available native");
        assert_eq!(
            err.stage,
            ExecutableArtifactContractStage::NativeArtifactBinding
        );
        assert_eq!(err.field, "native_phase_gate");
    }

    #[test]
    fn selected_target_absent_from_report_rejects_available_native_artifact_claim() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            "decl:fixture::Main::missing",
            "ken-runtime unit test",
        )
        .unwrap();
        contract.native_artifact.status = available_native_status(&contract);
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("target absent from supported set rejects available native");
        assert_eq!(
            err.stage,
            ExecutableArtifactContractStage::NativeArtifactBinding
        );
        assert_eq!(err.field, "supported_runtime_targets");
    }

    #[test]
    fn missing_unavailable_marker_rejects() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        contract.native_artifact.status =
            ExecutableNativeArtifactStatus::Unavailable { marker: None };
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("missing unavailable marker rejects");
        assert_eq!(
            err.stage,
            ExecutableArtifactContractStage::NativeArtifactBinding
        );
        assert_eq!(err.field, "unavailable_marker");
    }

    #[test]
    fn available_object_or_linker_lane_rejects_in_v0() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        contract.toolchain.object_emission = ExecutableEvidenceFact::Available {
            value: "object-file.o".to_string(),
            evidence_source: "test object evidence".to_string(),
            evidence_lane: ExecutableEvidenceLane::Tested,
        };
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("object lane available rejects in v0");
        assert_eq!(err.stage, ExecutableArtifactContractStage::ToolchainBinding);
        assert_eq!(err.field, "object_emission");
    }

    #[test]
    fn native_and_toolchain_facts_cannot_be_semantic_authority() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        let mut status = available_native_status(&contract);
        if let ExecutableNativeArtifactStatus::Available { evidence_lane, .. } = &mut status {
            *evidence_lane = ExecutableEvidenceLane::SemanticAuthority;
        }
        contract.native_artifact.status = status;
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("native evidence cannot be semantic authority");
        assert_eq!(
            err.stage,
            ExecutableArtifactContractStage::NativeArtifactBinding
        );
        assert_eq!(err.field, "evidence_lane");

        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            program.declarations[0].symbol.clone(),
            "ken-runtime unit test",
        )
        .unwrap();
        contract.toolchain.ken_runtime = ExecutableEvidenceFact::Available {
            value: "ken-runtime test".to_string(),
            evidence_source: "test exact-run evidence".to_string(),
            evidence_lane: ExecutableEvidenceLane::SemanticAuthority,
        };
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("toolchain evidence cannot be semantic authority");
        assert_eq!(err.stage, ExecutableArtifactContractStage::ToolchainBinding);
        assert_eq!(err.field, "ken_runtime");
    }

    #[test]
    fn native_artifact_available_without_hash_rejects() {
        let program = pure_program();
        let report = summarize_runtime_ir_program(&program);
        let target = program.declarations[0].symbol.clone();
        let mut contract = executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            target,
            "ken-runtime unit test",
        )
        .unwrap();
        let mut status = available_native_status(&contract);
        if let ExecutableNativeArtifactStatus::Available { artifact_hash, .. } = &mut status {
            *artifact_hash = None;
        }
        contract.native_artifact.status = status;
        refresh_hash(&mut contract);

        let err = validate_executable_artifact_contract(&program, &report, &contract)
            .expect_err("available native hash is required");
        assert_eq!(
            err.stage,
            ExecutableArtifactContractStage::NativeArtifactBinding
        );
        assert_eq!(err.field, "artifact_hash");
    }
}
