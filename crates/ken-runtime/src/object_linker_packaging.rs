//! Object and linker packaging for starter Ken-only executables.
//!
//! NC23 sits above the executable contract, entrypoint package, platform
//! runtime support report, and Cranelift runtime-IR comparison path. It records
//! object/linker/build facts and smoke-run evidence for one narrow starter host
//! target. Native bytes and linker success remain evidence artifacts, not Ken
//! semantic authority or proof evidence.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::platform_runtime_support::validate_entrypoint_metadata_payload;
use crate::{
    emit_runtime_ir_object_with_cranelift, fnv1a_64, platform_runtime_support_report_hash,
    run_runtime_ir_report_with_cranelift, runtime_executable_entrypoint_package_hash,
    CraneliftObjectArtifact, NativeDifferentialStage, NativeRuntimeIrComparisonVerdict,
    NativeSeedEnvironment, PlatformRuntimeEvidenceFact, PlatformRuntimeEvidenceLane,
    PlatformRuntimeSupportReport, RuntimeArtifactIdentity, RuntimeExecutableEntrypointPackage,
    RuntimeExpr, RuntimeGroundValue, RuntimeIrRunReport, RuntimeObservation, RuntimeProgram,
    RuntimeSymbol, EXECUTABLE_ENTRYPOINT_PACKAGE_KIND, EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION,
    PLATFORM_RUNTIME_SUPPORT_KIND, PLATFORM_RUNTIME_SUPPORT_VERSION,
};

pub const OBJECT_LINKER_PACKAGE_KIND: &str = "KenObjectLinkerExecutablePackage";
pub const OBJECT_LINKER_PACKAGE_VERSION: u32 = 0;
pub const OBJECT_LINKER_PACKAGE_SPEC_REF: &str = "docs/program/wp/NC23-object-linker-packaging.md";
pub const STARTER_ENTRY_SYMBOL: &str = "ken_nc23_entrypoint";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerExecutablePackage {
    pub header: ObjectLinkerPackageHeader,
    pub runtime_artifact: RuntimeArtifactIdentity,
    pub runtime_report_hash: u64,
    pub entrypoint_package_hash: u64,
    pub platform_runtime_support_hash: u64,
    pub object_artifact: ObjectLinkerArtifactFile,
    pub executable_artifact: ObjectLinkerArtifactFile,
    pub toolchain: ObjectLinkerToolchainFacts,
    pub smoke: ObjectLinkerSmokeReport,
    pub unavailable_lanes: BTreeSet<ObjectLinkerUnavailableLane>,
    pub unsupported_lanes: BTreeSet<ObjectLinkerUnsupportedLane>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerPackageHeader {
    pub package_kind: String,
    pub version: u32,
    pub producer: String,
    pub spec_ref: String,
    pub starter_platform_target: String,
    pub target_symbol: RuntimeSymbol,
    pub package_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerArtifactFile {
    pub kind: ObjectLinkerArtifactKind,
    pub relative_path: String,
    pub artifact_hash: u64,
    pub byte_len: u64,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectLinkerArtifactKind {
    CraneliftObject,
    StarterExecutable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerToolchainFacts {
    pub ken_runtime: ObjectLinkerEvidenceFact,
    pub native_backend: ObjectLinkerEvidenceFact,
    pub backend_verifier: ObjectLinkerEvidenceFact,
    pub object_emission: ObjectLinkerEvidenceFact,
    pub linker_or_finalizer: ObjectLinkerEvidenceFact,
    pub host_platform: ObjectLinkerEvidenceFact,
    pub library_abi: ObjectLinkerEvidenceFact,
    pub c_abi_interop: ObjectLinkerEvidenceFact,
    pub rust_interop: ObjectLinkerEvidenceFact,
    pub cross_package_native_linking: ObjectLinkerEvidenceFact,
    pub whole_compiler_proof: ObjectLinkerEvidenceFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectLinkerEvidenceFact {
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
pub enum ObjectLinkerEvidenceLane {
    SemanticAuthority,
    Tested,
    BuildArtifact,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectLinkerUnavailableLane {
    LibraryAbi,
    CAbiInterop,
    RustInterop,
    CrossPackageNativeLinking,
    DynamicLinkDependencySemantics,
    ForeignAbi,
    HostEffectOrFfiExecution,
    TranslationValidation,
    WholeCompilerProof,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectLinkerUnsupportedLane {
    NonStarterPlatform,
    NonScalarSmokeObservation,
    StaleArtifactIdentity,
    MissingToolchain,
    LinkerFailure,
    SmokeExecutionFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerSmokeReport {
    pub executable_relative_path: String,
    pub expected_stdout: String,
    pub stdout: String,
    pub exit_status: i32,
    pub passed: bool,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerPackagingOptions {
    pub linker_command: String,
    pub object_relative_path: String,
    pub stub_relative_path: String,
    pub executable_relative_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLinkerPackagingError {
    pub stage: ObjectLinkerPackagingStage,
    pub field: &'static str,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjectLinkerPackagingStage {
    PlatformTarget,
    EntrypointPackage,
    PlatformRuntimeSupport,
    RuntimeIrRunReport,
    NativeComparison,
    ObjectEmission,
    Toolchain,
    LinkerOrFinalizer,
    SmokeExecution,
    Hash,
}

impl ObjectLinkerPackagingOptions {
    pub fn starter_host() -> Self {
        Self {
            linker_command: "cc".to_string(),
            object_relative_path: "ken-entrypoint.o".to_string(),
            stub_relative_path: "ken-entrypoint-main.c".to_string(),
            executable_relative_path: executable_name("ken-starter"),
        }
    }
}

impl fmt::Display for ObjectLinkerPackagingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}/{}: {}", self.stage, self.field, self.reason)
    }
}

impl std::error::Error for ObjectLinkerPackagingError {}

pub fn package_starter_executable_artifact(
    program: &RuntimeProgram,
    entrypoint_package: &RuntimeExecutableEntrypointPackage,
    platform_support: &PlatformRuntimeSupportReport,
    run_report: &RuntimeIrRunReport,
    env: &NativeSeedEnvironment,
    output_dir: impl AsRef<Path>,
    producer: impl Into<String>,
) -> Result<ObjectLinkerExecutablePackage, ObjectLinkerPackagingError> {
    package_starter_executable_artifact_with_options(
        program,
        entrypoint_package,
        platform_support,
        run_report,
        env,
        output_dir,
        producer,
        &ObjectLinkerPackagingOptions::starter_host(),
    )
}

pub fn package_starter_executable_artifact_with_options(
    program: &RuntimeProgram,
    entrypoint_package: &RuntimeExecutableEntrypointPackage,
    platform_support: &PlatformRuntimeSupportReport,
    run_report: &RuntimeIrRunReport,
    env: &NativeSeedEnvironment,
    output_dir: impl AsRef<Path>,
    producer: impl Into<String>,
    options: &ObjectLinkerPackagingOptions,
) -> Result<ObjectLinkerExecutablePackage, ObjectLinkerPackagingError> {
    validate_options(options)?;
    validate_entrypoint_package(program, entrypoint_package)?;
    validate_platform_support(program, entrypoint_package, platform_support)?;
    validate_runtime_ir_run_report(program, entrypoint_package, run_report)?;

    let native_comparison = run_runtime_ir_report_with_cranelift(program, run_report.clone(), env);
    match &native_comparison.verdict {
        NativeRuntimeIrComparisonVerdict::RuntimeIrNativeAgreement {
            stage: NativeDifferentialStage::RuntimeIrNativeCompare,
        } => {}
        verdict => {
            return Err(packaging_error(
                ObjectLinkerPackagingStage::NativeComparison,
                "native_runtime_ir_comparison",
                format!("native comparison did not produce starter agreement: {verdict:?}"),
            ));
        }
    }
    let expected_stdout = scalar_smoke_stdout(&run_report.observation.observation)?;

    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::LinkerOrFinalizer,
            "output_dir",
            format!("could not create output directory: {err}"),
        )
    })?;

    let object =
        emit_runtime_ir_object_with_cranelift(program, run_report, env, STARTER_ENTRY_SYMBOL)
            .map_err(|err| {
                packaging_error(
                    ObjectLinkerPackagingStage::ObjectEmission,
                    "cranelift_object",
                    err.to_string(),
                )
            })?;
    let object_path = output_dir.join(&options.object_relative_path);
    fs::write(&object_path, &object.object_bytes).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::ObjectEmission,
            "object_path",
            format!("could not write object artifact: {err}"),
        )
    })?;

    let stub_path = output_dir.join(&options.stub_relative_path);
    fs::write(&stub_path, starter_c_stub()).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::LinkerOrFinalizer,
            "stub_path",
            format!("could not write starter finalizer source: {err}"),
        )
    })?;

    let executable_path = output_dir.join(&options.executable_relative_path);
    let linker_version = linker_version(&options.linker_command)?;
    link_starter_executable(
        &options.linker_command,
        &object_path,
        &stub_path,
        &executable_path,
    )?;

    let executable_bytes = fs::read(&executable_path).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::LinkerOrFinalizer,
            "executable_path",
            format!("could not read linked executable artifact: {err}"),
        )
    })?;
    let smoke = smoke_executable(
        &executable_path,
        &options.executable_relative_path,
        &expected_stdout,
    )?;

    let mut package = ObjectLinkerExecutablePackage {
        header: ObjectLinkerPackageHeader {
            package_kind: OBJECT_LINKER_PACKAGE_KIND.to_string(),
            version: OBJECT_LINKER_PACKAGE_VERSION,
            producer: producer.into(),
            spec_ref: OBJECT_LINKER_PACKAGE_SPEC_REF.to_string(),
            starter_platform_target: platform_support.header.platform_target.clone(),
            target_symbol: entrypoint_package.entrypoint.target_symbol.clone(),
            package_hash: 0,
        },
        runtime_artifact: RuntimeArtifactIdentity::from_program(program),
        runtime_report_hash: runtime_ir_program_report_hash_from_run(run_report),
        entrypoint_package_hash: entrypoint_package.header.package_hash,
        platform_runtime_support_hash: platform_support.header.support_hash,
        object_artifact: object_artifact_file(&object, options),
        executable_artifact: ObjectLinkerArtifactFile {
            kind: ObjectLinkerArtifactKind::StarterExecutable,
            relative_path: options.executable_relative_path.clone(),
            artifact_hash: fnv1a_64(&executable_bytes),
            byte_len: executable_bytes.len() as u64,
            evidence_source: "linked starter executable bytes read after exact linker run"
                .to_string(),
        },
        toolchain: toolchain_facts(&object, &linker_version, platform_support),
        smoke,
        unavailable_lanes: required_unavailable_lanes(),
        unsupported_lanes: BTreeSet::new(),
    };
    package.header.package_hash = object_linker_executable_package_hash(&package);
    validate_package_hash(&package)?;
    Ok(package)
}

/// Build the tested process-shaped native starter used by PX4 and later native
/// lowering stages.
///
/// The produced artifact receives fresh OS argv, environment, and cwd on every
/// invocation. It is a validated runtime artifact, never a proof surface.
pub fn build_process_starter_executable_artifact(
    entrypoint: &RuntimeExpr,
    output_dir: impl AsRef<Path>,
) -> Result<PathBuf, ObjectLinkerPackagingError> {
    let options = ObjectLinkerPackagingOptions::starter_host();
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::LinkerOrFinalizer,
            "output_dir",
            format!("could not create process starter output directory: {err}"),
        )
    })?;
    let object =
        crate::emit_process_entrypoint_object_with_cranelift(entrypoint, STARTER_ENTRY_SYMBOL)
            .map_err(|err| {
                packaging_error(
                    ObjectLinkerPackagingStage::ObjectEmission,
                    "process_cranelift_object",
                    err.to_string(),
                )
            })?;
    let object_path = output_dir.join(&options.object_relative_path);
    fs::write(&object_path, object.object_bytes).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::ObjectEmission,
            "object_path",
            format!("could not write process object artifact: {err}"),
        )
    })?;
    let stub_path = output_dir.join(&options.stub_relative_path);
    fs::write(&stub_path, process_starter_c_stub()).map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::LinkerOrFinalizer,
            "stub_path",
            format!("could not write process starter source: {err}"),
        )
    })?;
    let executable_path = output_dir.join(&options.executable_relative_path);
    link_starter_executable(
        &options.linker_command,
        &object_path,
        &stub_path,
        &executable_path,
    )?;
    Ok(executable_path)
}

pub fn object_linker_executable_package_hash(package: &ObjectLinkerExecutablePackage) -> u64 {
    fnv1a_64(&canonical_object_linker_package_bytes(package))
}

pub fn object_linker_runtime_ir_run_report_hash(run_report: &RuntimeIrRunReport) -> u64 {
    runtime_ir_program_report_hash_from_run(run_report)
}

fn validate_options(
    options: &ObjectLinkerPackagingOptions,
) -> Result<(), ObjectLinkerPackagingError> {
    for (field, value) in [
        ("linker_command", &options.linker_command),
        ("object_relative_path", &options.object_relative_path),
        ("stub_relative_path", &options.stub_relative_path),
        (
            "executable_relative_path",
            &options.executable_relative_path,
        ),
    ] {
        if value.trim().is_empty() {
            return Err(packaging_error(
                ObjectLinkerPackagingStage::Toolchain,
                field,
                "packaging option must be explicit",
            ));
        }
        if Path::new(value).is_absolute() || value.contains("..") {
            return Err(packaging_error(
                ObjectLinkerPackagingStage::Toolchain,
                field,
                "artifact layout records only relative paths, not host absolute paths",
            ));
        }
    }
    Ok(())
}

fn validate_entrypoint_package(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
) -> Result<(), ObjectLinkerPackagingError> {
    if package.header.package_kind != EXECUTABLE_ENTRYPOINT_PACKAGE_KIND {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::EntrypointPackage,
            "package_kind",
            "entrypoint package kind is not KenExecutableEntrypointPackage",
        ));
    }
    if package.header.version != EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::EntrypointPackage,
            "version",
            "entrypoint package version is unsupported by NC23",
        ));
    }
    if package.header.package_hash != runtime_executable_entrypoint_package_hash(package) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::EntrypointPackage,
            "package_hash",
            "entrypoint package hash is stale",
        ));
    }
    validate_entrypoint_metadata_payload(package).map_err(|err| {
        packaging_error(
            match err.stage {
                crate::PlatformRuntimeSupportStage::Hash => ObjectLinkerPackagingStage::Hash,
                _ => ObjectLinkerPackagingStage::EntrypointPackage,
            },
            err.field,
            err.reason,
        )
    })?;
    if package.runtime_artifact != RuntimeArtifactIdentity::from_program(program) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::EntrypointPackage,
            "runtime_artifact",
            "entrypoint package was not produced from the exact RuntimeProgram",
        ));
    }
    if package.entrypoint.target_symbol != package.header.target {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::EntrypointPackage,
            "target_symbol",
            "entrypoint target identity is internally inconsistent",
        ));
    }
    Ok(())
}

fn validate_platform_support(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
    support: &PlatformRuntimeSupportReport,
) -> Result<(), ObjectLinkerPackagingError> {
    if support.header.support_kind != PLATFORM_RUNTIME_SUPPORT_KIND {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::PlatformRuntimeSupport,
            "support_kind",
            "platform runtime support report kind is not KenPlatformRuntimeSupport",
        ));
    }
    if support.header.version != PLATFORM_RUNTIME_SUPPORT_VERSION {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::PlatformRuntimeSupport,
            "version",
            "platform runtime support report version is unsupported by NC23",
        ));
    }
    if support.header.support_hash != platform_runtime_support_report_hash(support) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::Hash,
            "platform_runtime_support_hash",
            "platform runtime support report hash is stale",
        ));
    }
    if support.runtime_artifact != RuntimeArtifactIdentity::from_program(program) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::PlatformRuntimeSupport,
            "runtime_artifact",
            "platform runtime support report does not bind the exact RuntimeProgram",
        ));
    }
    if support.entrypoint_package_hash != package.header.package_hash
        || support.entrypoint_metadata_identity != package.entrypoint.metadata_identity
        || support.target != package.entrypoint.target_symbol
        || support.header.target_symbol != package.entrypoint.target_symbol
    {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::PlatformRuntimeSupport,
            "entrypoint_binding",
            "platform runtime support report does not bind the exact entrypoint package",
        ));
    }
    if support.header.platform_target != native_platform_target_name() {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::PlatformTarget,
            "platform_target",
            "NC23 starter packaging only supports the exact host starter platform target",
        ));
    }
    if !matches!(
        support.support_facts.starter_platform_target,
        PlatformRuntimeEvidenceFact::Available {
            lane: PlatformRuntimeEvidenceLane::Tested,
            ..
        }
    ) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::PlatformTarget,
            "starter_platform_target",
            "platform support report does not mark the starter target as tested",
        ));
    }
    Ok(())
}

fn validate_runtime_ir_run_report(
    program: &RuntimeProgram,
    package: &RuntimeExecutableEntrypointPackage,
    run_report: &RuntimeIrRunReport,
) -> Result<(), ObjectLinkerPackagingError> {
    let artifact = RuntimeArtifactIdentity::from_program(program);
    if run_report.artifact != artifact || run_report.observation.artifact != artifact {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::RuntimeIrRunReport,
            "artifact",
            "RuntimeIrRunReport does not bind the exact RuntimeProgram artifact",
        ));
    }
    if run_report.evidence.package_identity != program.package_identity
        || run_report.evidence.core_semantic_hash != program.core_semantic_hash
        || run_report.evidence.runtime_artifact_hash != program.artifact_hash
    {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::RuntimeIrRunReport,
            "evidence",
            "RuntimeIrRunReport evidence identity does not match the exact RuntimeProgram",
        ));
    }
    if run_report.observation.target != run_report.target
        || run_report.evidence.target_example != run_report.target.example
        || run_report.evidence.checked_core_shape != run_report.target.checked_core_shape
    {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport target evidence is internally inconsistent",
        ));
    }
    let mut matching_examples = program.examples.iter().filter(|example| {
        example.name == run_report.target.example
            && example.checked_core_shape == run_report.target.checked_core_shape
    });
    let Some(example) = matching_examples.next() else {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport target is absent from the exact RuntimeProgram",
        ));
    };
    if matching_examples.next().is_some() {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport target is ambiguous in the exact RuntimeProgram",
        ));
    }
    if !matches!(
        &example.ir,
        crate::RuntimeExpr::DeclarationRef { symbol }
            if symbol == &package.entrypoint.target_symbol
    ) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::RuntimeIrRunReport,
            "target",
            "RuntimeIrRunReport does not evaluate the packaged entrypoint",
        ));
    }
    Ok(())
}

fn scalar_smoke_stdout(
    observation: &RuntimeObservation,
) -> Result<String, ObjectLinkerPackagingError> {
    match observation {
        RuntimeObservation::Returned(RuntimeGroundValue::Int(value)) => Ok(format!("{value}\n")),
        RuntimeObservation::Returned(RuntimeGroundValue::Bool(value)) => {
            Ok(format!("{}\n", i64::from(*value)))
        }
        RuntimeObservation::Returned(_) => Err(packaging_error(
            ObjectLinkerPackagingStage::SmokeExecution,
            "runtime_observation",
            "NC23 starter executable smoke only supports scalar Int/Bool observations",
        )),
        RuntimeObservation::Trapped(trap) => Err(packaging_error(
            ObjectLinkerPackagingStage::SmokeExecution,
            "runtime_observation",
            format!(
                "NC23 starter executable smoke does not yet package trap reports: {}",
                trap.message
            ),
        )),
    }
}

fn link_starter_executable(
    linker: &str,
    object_path: &Path,
    stub_path: &Path,
    executable_path: &Path,
) -> Result<(), ObjectLinkerPackagingError> {
    let output = Command::new(linker)
        .arg(object_path)
        .arg(stub_path)
        .arg("-o")
        .arg(executable_path)
        .output()
        .map_err(|err| {
            packaging_error(
                ObjectLinkerPackagingStage::Toolchain,
                "linker_command",
                format!("could not execute linker/finalizer command: {err}"),
            )
        })?;
    if !output.status.success() {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::LinkerOrFinalizer,
            "linker_command",
            format!(
                "linker/finalizer failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(())
}

fn smoke_executable(
    executable_path: &Path,
    executable_relative_path: &str,
    expected_stdout: &str,
) -> Result<ObjectLinkerSmokeReport, ObjectLinkerPackagingError> {
    let output = Command::new(executable_path).output().map_err(|err| {
        packaging_error(
            ObjectLinkerPackagingStage::SmokeExecution,
            "executable_artifact",
            format!("could not execute starter artifact: {err}"),
        )
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let status = output.status.code().unwrap_or(-1);
    if !output.status.success() || stdout != expected_stdout {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::SmokeExecution,
            "stdout",
            format!(
                "starter smoke mismatch: status {status}, stdout {:?}, expected {:?}",
                stdout, expected_stdout
            ),
        ));
    }
    Ok(ObjectLinkerSmokeReport {
        executable_relative_path: executable_relative_path.to_string(),
        expected_stdout: expected_stdout.to_string(),
        stdout,
        exit_status: status,
        passed: true,
        evidence_source: "exact linked executable was run once by NC23 smoke packaging".to_string(),
    })
}

fn linker_version(linker: &str) -> Result<String, ObjectLinkerPackagingError> {
    let output = Command::new(linker)
        .arg("--version")
        .output()
        .map_err(|err| {
            packaging_error(
                ObjectLinkerPackagingStage::Toolchain,
                "linker_command",
                format!("could not execute linker/finalizer version command: {err}"),
            )
        })?;
    if !output.status.success() {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::Toolchain,
            "linker_command",
            "linker/finalizer version command failed",
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("unknown linker/finalizer")
        .to_string())
}

fn object_artifact_file(
    object: &CraneliftObjectArtifact,
    options: &ObjectLinkerPackagingOptions,
) -> ObjectLinkerArtifactFile {
    ObjectLinkerArtifactFile {
        kind: ObjectLinkerArtifactKind::CraneliftObject,
        relative_path: options.object_relative_path.clone(),
        artifact_hash: object.object_hash,
        byte_len: object.object_bytes.len() as u64,
        evidence_source: "Cranelift object bytes emitted from exact RuntimeProgram target"
            .to_string(),
    }
}

fn toolchain_facts(
    object: &CraneliftObjectArtifact,
    linker_version: &str,
    support: &PlatformRuntimeSupportReport,
) -> ObjectLinkerToolchainFacts {
    ObjectLinkerToolchainFacts {
        ken_runtime: ObjectLinkerEvidenceFact::Available {
            value: format!("ken-runtime {}", env!("CARGO_PKG_VERSION")),
            evidence_source: "compiled ken-runtime crate version embedded by Cargo".to_string(),
            lane: ObjectLinkerEvidenceLane::BuildArtifact,
        },
        native_backend: ObjectLinkerEvidenceFact::Available {
            value: object.backend_name.clone(),
            evidence_source: "Cranelift object emitter used for this exact object".to_string(),
            lane: ObjectLinkerEvidenceLane::BuildArtifact,
        },
        backend_verifier: ObjectLinkerEvidenceFact::Available {
            value: format!("Cranelift verifier passed: {}", object.verifier_passed),
            evidence_source: "Cranelift verifier ran before object emission".to_string(),
            lane: ObjectLinkerEvidenceLane::Tested,
        },
        object_emission: ObjectLinkerEvidenceFact::Available {
            value: format!("object hash {:016x}", object.object_hash),
            evidence_source: "object bytes emitted and hashed by NC23 packaging".to_string(),
            lane: ObjectLinkerEvidenceLane::BuildArtifact,
        },
        linker_or_finalizer: ObjectLinkerEvidenceFact::Available {
            value: linker_version.to_string(),
            evidence_source: "linker/finalizer --version from the exact packaging run".to_string(),
            lane: ObjectLinkerEvidenceLane::BuildArtifact,
        },
        host_platform: ObjectLinkerEvidenceFact::Available {
            value: support.header.platform_target.clone(),
            evidence_source: "NC21 starter platform runtime support report".to_string(),
            lane: ObjectLinkerEvidenceLane::Tested,
        },
        library_abi: unavailable("library ABI is outside NC23 executable packaging"),
        c_abi_interop: unavailable("C ABI interop is outside NC23 executable packaging"),
        rust_interop: unavailable("Rust interop is outside NC23 executable packaging"),
        cross_package_native_linking: unavailable(
            "cross-package native linking is outside NC23 executable packaging",
        ),
        whole_compiler_proof: unavailable(
            "linker success and smoke execution are not whole-compiler proof",
        ),
    }
}

fn unavailable(reason: &str) -> ObjectLinkerEvidenceFact {
    ObjectLinkerEvidenceFact::Unavailable {
        reason: reason.to_string(),
        lane: ObjectLinkerEvidenceLane::Unavailable,
    }
}

fn required_unavailable_lanes() -> BTreeSet<ObjectLinkerUnavailableLane> {
    BTreeSet::from([
        ObjectLinkerUnavailableLane::LibraryAbi,
        ObjectLinkerUnavailableLane::CAbiInterop,
        ObjectLinkerUnavailableLane::RustInterop,
        ObjectLinkerUnavailableLane::CrossPackageNativeLinking,
        ObjectLinkerUnavailableLane::DynamicLinkDependencySemantics,
        ObjectLinkerUnavailableLane::ForeignAbi,
        ObjectLinkerUnavailableLane::HostEffectOrFfiExecution,
        ObjectLinkerUnavailableLane::TranslationValidation,
        ObjectLinkerUnavailableLane::WholeCompilerProof,
    ])
}

fn runtime_ir_program_report_hash_from_run(run_report: &RuntimeIrRunReport) -> u64 {
    let mut out = String::new();
    push_field(&mut out, "evaluator", "direct_runtime_ir_evaluator_v1");
    push_field(&mut out, "target.example", &run_report.target.example);
    push_field(
        &mut out,
        "target.checked_core_shape",
        &run_report.target.checked_core_shape,
    );
    push_runtime_artifact(&mut out, "artifact", &run_report.artifact);
    push_runtime_artifact(
        &mut out,
        "observation.artifact",
        &run_report.observation.artifact,
    );
    push_field(
        &mut out,
        "observation.target.example",
        &run_report.observation.target.example,
    );
    push_field(
        &mut out,
        "observation.target.checked_core_shape",
        &run_report.observation.target.checked_core_shape,
    );
    push_runtime_observation(&mut out, &run_report.observation.observation);
    push_field(
        &mut out,
        "observation.evidence_source",
        &run_report.observation.evidence_source,
    );
    push_field(
        &mut out,
        "evidence.package_identity",
        &run_report.evidence.package_identity,
    );
    push_field(
        &mut out,
        "evidence.core_semantic_hash",
        &format!("{:016x}", run_report.evidence.core_semantic_hash),
    );
    push_field(
        &mut out,
        "evidence.runtime_artifact_hash",
        &format!("{:016x}", run_report.evidence.runtime_artifact_hash),
    );
    push_field(
        &mut out,
        "evidence.target_example",
        &run_report.evidence.target_example,
    );
    push_field(
        &mut out,
        "evidence.checked_core_shape",
        &run_report.evidence.checked_core_shape,
    );
    for (key, value) in &run_report.evidence.evidence_sources {
        push_field(&mut out, "evidence_source.key", key);
        push_field(&mut out, "evidence_source.value", value);
    }
    for unavailable in &run_report.evidence.unavailable {
        push_field(&mut out, "evidence.unavailable", unavailable);
    }
    fnv1a_64(&out.into_bytes())
}

fn validate_package_hash(
    package: &ObjectLinkerExecutablePackage,
) -> Result<(), ObjectLinkerPackagingError> {
    if package.header.package_hash != object_linker_executable_package_hash(package) {
        return Err(packaging_error(
            ObjectLinkerPackagingStage::Hash,
            "package_hash",
            "object/linker executable package hash is stale",
        ));
    }
    Ok(())
}

fn canonical_object_linker_package_bytes(package: &ObjectLinkerExecutablePackage) -> Vec<u8> {
    let mut out = String::new();
    push_field(&mut out, "kind", &package.header.package_kind);
    push_field(&mut out, "version", &package.header.version.to_string());
    push_field(&mut out, "producer", &package.header.producer);
    push_field(&mut out, "spec_ref", &package.header.spec_ref);
    push_field(
        &mut out,
        "starter_platform_target",
        &package.header.starter_platform_target,
    );
    push_field(&mut out, "target_symbol", &package.header.target_symbol);
    push_field(
        &mut out,
        "runtime_package_identity",
        &package.runtime_artifact.package_identity,
    );
    push_field(
        &mut out,
        "runtime_core_semantic_hash",
        &format!("{:016x}", package.runtime_artifact.core_semantic_hash),
    );
    push_field(
        &mut out,
        "runtime_artifact_hash",
        &format!("{:016x}", package.runtime_artifact.artifact_hash),
    );
    push_field(
        &mut out,
        "runtime_report_hash",
        &format!("{:016x}", package.runtime_report_hash),
    );
    push_field(
        &mut out,
        "entrypoint_package_hash",
        &format!("{:016x}", package.entrypoint_package_hash),
    );
    push_field(
        &mut out,
        "platform_runtime_support_hash",
        &format!("{:016x}", package.platform_runtime_support_hash),
    );
    push_artifact(&mut out, &package.object_artifact);
    push_artifact(&mut out, &package.executable_artifact);
    push_smoke(&mut out, &package.smoke);
    push_fact(&mut out, "ken_runtime", &package.toolchain.ken_runtime);
    push_fact(
        &mut out,
        "native_backend",
        &package.toolchain.native_backend,
    );
    push_fact(
        &mut out,
        "backend_verifier",
        &package.toolchain.backend_verifier,
    );
    push_fact(
        &mut out,
        "object_emission",
        &package.toolchain.object_emission,
    );
    push_fact(
        &mut out,
        "linker_or_finalizer",
        &package.toolchain.linker_or_finalizer,
    );
    push_fact(&mut out, "host_platform", &package.toolchain.host_platform);
    push_fact(&mut out, "library_abi", &package.toolchain.library_abi);
    push_fact(&mut out, "c_abi_interop", &package.toolchain.c_abi_interop);
    push_fact(&mut out, "rust_interop", &package.toolchain.rust_interop);
    push_fact(
        &mut out,
        "cross_package_native_linking",
        &package.toolchain.cross_package_native_linking,
    );
    push_fact(
        &mut out,
        "whole_compiler_proof",
        &package.toolchain.whole_compiler_proof,
    );
    for lane in &package.unavailable_lanes {
        push_field(&mut out, "unavailable_lane", unavailable_lane_tag(lane));
    }
    for lane in &package.unsupported_lanes {
        push_field(&mut out, "unsupported_lane", unsupported_lane_tag(lane));
    }
    out.into_bytes()
}

fn push_artifact(out: &mut String, artifact: &ObjectLinkerArtifactFile) {
    push_field(out, "artifact_kind", artifact_kind_tag(&artifact.kind));
    push_field(out, "artifact_relative_path", &artifact.relative_path);
    push_field(
        out,
        "artifact_hash",
        &format!("{:016x}", artifact.artifact_hash),
    );
    push_field(out, "artifact_byte_len", &artifact.byte_len.to_string());
    push_field(out, "artifact_evidence_source", &artifact.evidence_source);
}

fn push_smoke(out: &mut String, smoke: &ObjectLinkerSmokeReport) {
    push_field(out, "smoke_executable", &smoke.executable_relative_path);
    push_field(out, "smoke_expected_stdout", &smoke.expected_stdout);
    push_field(out, "smoke_stdout", &smoke.stdout);
    push_field(out, "smoke_exit_status", &smoke.exit_status.to_string());
    push_field(out, "smoke_passed", &smoke.passed.to_string());
    push_field(out, "smoke_evidence_source", &smoke.evidence_source);
}

fn push_runtime_artifact(out: &mut String, prefix: &str, artifact: &RuntimeArtifactIdentity) {
    push_field(
        out,
        &format!("{prefix}.package_identity"),
        &artifact.package_identity,
    );
    push_field(
        out,
        &format!("{prefix}.core_semantic_hash"),
        &format!("{:016x}", artifact.core_semantic_hash),
    );
    push_field(
        out,
        &format!("{prefix}.artifact_hash"),
        &format!("{:016x}", artifact.artifact_hash),
    );
}

fn push_runtime_observation(out: &mut String, observation: &RuntimeObservation) {
    match observation {
        RuntimeObservation::Returned(value) => {
            push_field(out, "observation.kind", "returned");
            push_ground_value(out, "observation.value", value);
        }
        RuntimeObservation::Trapped(trap) => {
            push_field(out, "observation.kind", "trapped");
            push_field(
                out,
                "observation.trap.code",
                runtime_trap_code_tag(&trap.code),
            );
            push_field(out, "observation.trap.message", &trap.message);
        }
    }
}

fn push_ground_value(out: &mut String, prefix: &str, value: &RuntimeGroundValue) {
    match value {
        RuntimeGroundValue::Bool(value) => {
            push_field(out, &format!("{prefix}.kind"), "bool");
            push_field(out, &format!("{prefix}.value"), &value.to_string());
        }
        RuntimeGroundValue::Int(value) => {
            push_field(out, &format!("{prefix}.kind"), "int");
            push_field(out, &format!("{prefix}.value"), &value.to_string());
        }
        RuntimeGroundValue::Bytes(bytes) => {
            push_field(out, &format!("{prefix}.kind"), "bytes");
            for byte in bytes {
                push_field(out, &format!("{prefix}.byte"), &byte.to_string());
            }
        }
        RuntimeGroundValue::String(value) => {
            push_field(out, &format!("{prefix}.kind"), "string");
            push_field(out, &format!("{prefix}.value"), value);
        }
        RuntimeGroundValue::Constructor { constructor, args } => {
            push_field(out, &format!("{prefix}.kind"), "constructor");
            push_field(out, &format!("{prefix}.constructor"), constructor);
            for arg in args {
                push_ground_value(out, &format!("{prefix}.arg"), arg);
            }
        }
        RuntimeGroundValue::Record { fields } => {
            push_field(out, &format!("{prefix}.kind"), "record");
            for (name, value) in fields {
                push_field(out, &format!("{prefix}.field.name"), name);
                push_ground_value(out, &format!("{prefix}.field.value"), value);
            }
        }
    }
}

fn push_fact(out: &mut String, name: &str, fact: &ObjectLinkerEvidenceFact) {
    match fact {
        ObjectLinkerEvidenceFact::Available {
            value,
            evidence_source,
            lane,
        } => {
            push_field(out, name, "available");
            push_field(out, &format!("{name}.value"), value);
            push_field(out, &format!("{name}.evidence_source"), evidence_source);
            push_field(out, &format!("{name}.lane"), evidence_lane_tag(lane));
        }
        ObjectLinkerEvidenceFact::Unavailable { reason, lane } => {
            push_field(out, name, "unavailable");
            push_field(out, &format!("{name}.reason"), reason);
            push_field(out, &format!("{name}.lane"), evidence_lane_tag(lane));
        }
    }
}

fn push_field(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push('=');
    out.push_str(&value.len().to_string());
    out.push(':');
    out.push_str(value);
    out.push('\n');
}

fn artifact_kind_tag(kind: &ObjectLinkerArtifactKind) -> &'static str {
    match kind {
        ObjectLinkerArtifactKind::CraneliftObject => "cranelift_object",
        ObjectLinkerArtifactKind::StarterExecutable => "starter_executable",
    }
}

fn evidence_lane_tag(lane: &ObjectLinkerEvidenceLane) -> &'static str {
    match lane {
        ObjectLinkerEvidenceLane::SemanticAuthority => "semantic_authority",
        ObjectLinkerEvidenceLane::Tested => "tested",
        ObjectLinkerEvidenceLane::BuildArtifact => "build_artifact",
        ObjectLinkerEvidenceLane::Unavailable => "unavailable",
        ObjectLinkerEvidenceLane::Unsupported => "unsupported",
    }
}

fn unavailable_lane_tag(lane: &ObjectLinkerUnavailableLane) -> &'static str {
    match lane {
        ObjectLinkerUnavailableLane::LibraryAbi => "library_abi",
        ObjectLinkerUnavailableLane::CAbiInterop => "c_abi_interop",
        ObjectLinkerUnavailableLane::RustInterop => "rust_interop",
        ObjectLinkerUnavailableLane::CrossPackageNativeLinking => "cross_package_native_linking",
        ObjectLinkerUnavailableLane::DynamicLinkDependencySemantics => {
            "dynamic_link_dependency_semantics"
        }
        ObjectLinkerUnavailableLane::ForeignAbi => "foreign_abi",
        ObjectLinkerUnavailableLane::HostEffectOrFfiExecution => "host_effect_or_ffi_execution",
        ObjectLinkerUnavailableLane::TranslationValidation => "translation_validation",
        ObjectLinkerUnavailableLane::WholeCompilerProof => "whole_compiler_proof",
    }
}

fn unsupported_lane_tag(lane: &ObjectLinkerUnsupportedLane) -> &'static str {
    match lane {
        ObjectLinkerUnsupportedLane::NonStarterPlatform => "non_starter_platform",
        ObjectLinkerUnsupportedLane::NonScalarSmokeObservation => "non_scalar_smoke_observation",
        ObjectLinkerUnsupportedLane::StaleArtifactIdentity => "stale_artifact_identity",
        ObjectLinkerUnsupportedLane::MissingToolchain => "missing_toolchain",
        ObjectLinkerUnsupportedLane::LinkerFailure => "linker_failure",
        ObjectLinkerUnsupportedLane::SmokeExecutionFailure => "smoke_execution_failure",
    }
}

fn runtime_trap_code_tag(code: &crate::RuntimeTrapCode) -> &'static str {
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

fn starter_c_stub() -> &'static str {
    r#"#include <stdio.h>

extern long long ken_nc23_entrypoint(const void *input);

int main(void) {
    long long value = ken_nc23_entrypoint(NULL);
    printf("%lld\n", value);
    return 0;
}
"#
}

pub(crate) fn process_starter_c_stub() -> &'static str {
    r#"#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

struct KenBorrowedValue {
    uint64_t kind;
    uint64_t tag;
    const void *data;
    size_t len;
};

enum { KEN_BYTES = 1, KEN_CONSTRUCTOR = 2 };
enum { KEN_PROCESS_INPUT = 1, KEN_NIL = 2, KEN_CONS = 3, KEN_PROD = 4 };

struct KenArena {
    struct KenBorrowedValue *values;
    size_t next;
    size_t capacity;
};

extern long long ken_nc23_entrypoint(const struct KenBorrowedValue *root);

static int constructor(
    struct KenArena *arena,
    struct KenBorrowedValue *value,
    uint64_t tag,
    size_t arity
) {
    if (arity > arena->capacity - arena->next) return 0;
    value->kind = KEN_CONSTRUCTOR;
    value->tag = tag;
    value->data = &arena->values[arena->next];
    value->len = arity;
    arena->next += arity;
    return 1;
}

static void bytes(
    struct KenBorrowedValue *value,
    const unsigned char *data,
    size_t len
) {
    value->kind = KEN_BYTES;
    value->tag = 0;
    value->data = data;
    value->len = len;
}

static int arguments(
    struct KenArena *arena,
    struct KenBorrowedValue *value,
    size_t index,
    size_t count,
    char **argv
) {
    for (; index < count; ++index) {
        if (!constructor(arena, value, KEN_CONS, 2)) return 0;
        struct KenBorrowedValue *fields = (struct KenBorrowedValue *)value->data;
        bytes(&fields[0], (const unsigned char *)argv[index], strlen(argv[index]));
        value = &fields[1];
    }
    return constructor(arena, value, KEN_NIL, 0);
}

static int environment(
    struct KenArena *arena,
    struct KenBorrowedValue *value,
    size_t index,
    size_t count,
    char **envp
) {
    for (; index < count; ++index) {
        char *separator = strchr(envp[index], '=');
        if (separator == NULL || !constructor(arena, value, KEN_CONS, 2)) return 0;
        struct KenBorrowedValue *fields = (struct KenBorrowedValue *)value->data;
        if (!constructor(arena, &fields[0], KEN_PROD, 2)) return 0;
        struct KenBorrowedValue *pair = (struct KenBorrowedValue *)fields[0].data;
        bytes(&pair[0], (const unsigned char *)envp[index], (size_t)(separator - envp[index]));
        bytes(&pair[1], (const unsigned char *)(separator + 1), strlen(separator + 1));
        value = &fields[1];
    }
    return constructor(arena, value, KEN_NIL, 0);
}

int main(int argc, char **argv, char **envp) {
    size_t argument_count = argc < 0 ? 0 : (size_t)argc;
    size_t environment_count = 0;
    while (envp[environment_count] != NULL) ++environment_count;
    char *cwd = getcwd(NULL, 0);
    if (cwd == NULL) return 1;
    if (argument_count > (SIZE_MAX - 4) / 2 ||
        environment_count > (SIZE_MAX - 4 - 2 * argument_count) / 4) {
        free(cwd); return 1;
    }
    size_t capacity = 4 + 2 * argument_count + 4 * environment_count;
    struct KenBorrowedValue *pool = calloc(capacity, sizeof(*pool));
    if (pool == NULL) { free(cwd); return 1; }
    struct KenArena arena = { .values = pool, .next = 1, .capacity = capacity };
    struct KenBorrowedValue *root = &pool[0];
    if (!constructor(&arena, root, KEN_PROCESS_INPUT, 3)) return 1;
    struct KenBorrowedValue *fields = (struct KenBorrowedValue *)root->data;
    if (!arguments(&arena, &fields[0], 0, argument_count, argv) ||
        !environment(&arena, &fields[1], 0, environment_count, envp)) {
        free(pool); free(cwd); return 1;
    }
    bytes(&fields[2], (const unsigned char *)cwd, strlen(cwd));
    if (arena.next != arena.capacity) { free(pool); free(cwd); return 1; }
    long long value = ken_nc23_entrypoint(root);
    free(cwd);
    free(pool);
    if (value < 0) {
        fputs("ken native trap: malformed borrowed process input\n", stderr);
        return 1;
    }
    return (int)value;
}
"#
}

fn executable_name(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.exe")
    } else {
        stem.to_string()
    }
}

fn native_platform_target_name() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

fn packaging_error(
    stage: ObjectLinkerPackagingStage,
    field: &'static str,
    reason: impl Into<String>,
) -> ObjectLinkerPackagingError {
    ObjectLinkerPackagingError {
        stage,
        field,
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::{
        evaluate_runtime_ir_example, executable_artifact_contract_for_runtime_report,
        executable_entrypoint_metadata_hash, executable_entrypoint_package_for_runtime_contract,
        platform_runtime_support_for_entrypoint, summarize_runtime_ir_program,
        ErasedExecutableCore, ExecutableArgumentPackaging, ExecutableArgumentShape,
        ExecutableDependencyClosure, ExecutableEntrypointPackageMetadata,
        ExecutableEntrypointTargetKind, ExecutableEntrypointVerdict, ExecutableReportContract,
        ExecutableResultObservation, ExecutableResultShape, ExecutableRuntimeSupport,
        ExecutableTrapContract, ExecutableTrapShape, RuntimeDeclaration, RuntimeDeclarationKind,
        RuntimeExpr, RuntimeIrProgramReport, RuntimeIrSeedEnvironment, RuntimeLowerabilityStatus,
        RuntimeMetadata, RuntimePartiality, RuntimePrimitive, RuntimeSymbolMetadata, RuntimeTrap,
        RuntimeTrapCode, RuntimeValue,
    };

    fn starter_program(body: RuntimeExpr, observation: RuntimeObservation) -> RuntimeProgram {
        let symbol = "decl:fixture::Executable::main".to_string();
        let mut metadata = RuntimeMetadata::default();
        metadata
            .lowerability
            .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
        RuntimeProgram {
            package_identity: "module:fixture::object-linker".to_string(),
            core_semantic_hash: 0x2301,
            artifact_hash: 0x2302,
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
            examples: vec![crate::RuntimeExample {
                name: "object-linker-main".to_string(),
                checked_core_shape: "fixture main".to_string(),
                ir: RuntimeExpr::DeclarationRef { symbol },
                observation,
            }],
        }
    }

    fn int_body(value: i64) -> RuntimeExpr {
        RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "add_int".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Int(value - 1)),
                RuntimeExpr::Value(RuntimeValue::Int(1)),
            ],
        }
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
            "object linker unit test",
        )
        .expect("contract materializes");
        let mut entrypoint = ExecutableEntrypointPackageMetadata {
            package_identity: program.package_identity.clone(),
            package_core_semantic_hash: program.core_semantic_hash,
            package_artifact_hash: program.artifact_hash,
            target_symbol: target,
            target_kind: ExecutableEntrypointTargetKind::Executable,
            closure_identity: 0x2320,
            closure_semantic_hash: 0x2321,
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
                target_closure_identity: 0x2320,
                target_closure_report_hash: 0x2322,
                evidence_source: "target closure report".to_string(),
            },
            unsupported_lanes: Default::default(),
        };
        entrypoint.metadata_identity = executable_entrypoint_metadata_hash(&entrypoint);
        let package = executable_entrypoint_package_for_runtime_contract(
            program,
            &report,
            &contract,
            entrypoint,
            "object linker unit test",
        )
        .expect("entrypoint package materializes");
        (report, package)
    }

    fn runtime_ir_run_report(program: &RuntimeProgram) -> RuntimeIrRunReport {
        evaluate_runtime_ir_example(
            program,
            &program.examples[0],
            &RuntimeIrSeedEnvironment::empty(),
        )
        .expect("runtime-IR evaluator produces an observation")
    }

    fn platform_support(
        program: &RuntimeProgram,
        entrypoint: &RuntimeExecutableEntrypointPackage,
        run_report: &RuntimeIrRunReport,
    ) -> PlatformRuntimeSupportReport {
        platform_runtime_support_for_entrypoint(
            program,
            entrypoint,
            run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes")
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

    #[test]
    fn packages_and_smokes_scalar_starter_executable() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(42));
        let program = starter_program(int_body(42), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let support = platform_runtime_support_for_entrypoint(
            &program,
            &entrypoint,
            &run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes");
        let output_dir = temp_output_dir("nc23-smoke");

        let package = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            &output_dir,
            "object linker unit test",
        )
        .expect("object/linker package materializes");

        assert_eq!(
            package.runtime_artifact,
            RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(
            package.header.package_hash,
            object_linker_executable_package_hash(&package)
        );
        assert_eq!(package.smoke.stdout, "42\n");
        assert!(package.smoke.passed);
        assert!(package.object_artifact.byte_len > 0);
        assert!(package.executable_artifact.byte_len > 0);
        assert!(package
            .unavailable_lanes
            .contains(&ObjectLinkerUnavailableLane::WholeCompilerProof));
        assert!(matches!(
            package.toolchain.whole_compiler_proof,
            ObjectLinkerEvidenceFact::Unavailable { .. }
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn same_process_artifact_observes_fresh_byte_exact_os_input() {
        use std::ffi::OsString;
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        use std::process::Command;

        let output_dir = temp_output_dir("px4-process-input");
        let cwd_one = output_dir.join(OsString::from_vec(vec![b'c', b'w', b'd', 0xfe]));
        let cwd_two = output_dir.join(OsString::from_vec(vec![b'c', b'w', b'd', 0xfd]));
        fs::create_dir_all(&cwd_one).expect("first cwd exists");
        fs::create_dir_all(&cwd_two).expect("second cwd exists");
        let option_none = "ctor:fixture::Option::None";
        let option_some = "ctor:fixture::Option::Some";
        let byte_at = |bytes: RuntimeExpr, index: i64| RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_at".to_string(),
                    partiality: RuntimePartiality::SafeOption {
                        none: option_none.to_string(),
                        some: option_some.to_string(),
                        obligation: Some("obl:px4.bytes_at.bounds".to_string()),
                    },
                },
                args: vec![bytes, RuntimeExpr::Value(RuntimeValue::Int(index))],
            }),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: option_none.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Value(RuntimeValue::Int(1)),
                },
                crate::RuntimeMatchCase {
                    constructor: option_some.to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "invalid borrowed bytes_at result".to_string(),
            },
        };
        let argv_byte = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: crate::LIST_CONS_CONSTRUCTOR.to_string(),
                binders: 2,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(1)),
                    cases: vec![crate::RuntimeMatchCase {
                        constructor: crate::LIST_CONS_CONSTRUCTOR.to_string(),
                        binders: 2,
                        body: byte_at(RuntimeExpr::Var(0), 0),
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "missing process argument".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "missing argv[0]".to_string(),
            },
        };
        let environment_byte = |field: u32| RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(1)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: crate::LIST_CONS_CONSTRUCTOR.to_string(),
                binders: 2,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![crate::RuntimeMatchCase {
                        constructor: crate::PROD_CONSTRUCTOR.to_string(),
                        binders: 2,
                        body: byte_at(RuntimeExpr::Var(field), 0),
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "environment head is not Prod".to_string(),
                    },
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "environment is empty".to_string(),
            },
        };
        let equals = |value: RuntimeExpr, expected: i64| RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "eq_int".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![value, RuntimeExpr::Value(RuntimeValue::Int(expected))],
        };
        let cwd_length = RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "bytes_length".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![RuntimeExpr::Var(2)],
        };
        let guarded = RuntimeExpr::If {
            scrutinee: Box::new(equals(argv_byte, 0xff)),
            then_expr: Box::new(RuntimeExpr::If {
                scrutinee: Box::new(equals(environment_byte(0), i64::from(b'K'))),
                then_expr: Box::new(RuntimeExpr::If {
                    scrutinee: Box::new(equals(
                        cwd_length,
                        cwd_one.as_os_str().as_bytes().len() as i64,
                    )),
                    then_expr: Box::new(RuntimeExpr::If {
                        scrutinee: Box::new(equals(
                            byte_at(
                                RuntimeExpr::Var(2),
                                cwd_one.as_os_str().as_bytes().len() as i64 - 1,
                            ),
                            i64::from(0xfe_u8),
                        )),
                        then_expr: Box::new(environment_byte(1)),
                        else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
                    }),
                    else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
                }),
                else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
            }),
            else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int(1))),
        };
        let entry = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![crate::RuntimeMatchCase {
                constructor: crate::PROCESS_INPUT_CONSTRUCTOR.to_string(),
                binders: 3,
                body: guarded,
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "entry argument is not ProcessInput".to_string(),
            },
        };
        let executable = build_process_starter_executable_artifact(&entry, &output_dir)
            .expect("process starter links");
        assert!(!process_starter_c_stub().contains("fnv"));
        assert!(!process_starter_c_stub().contains("discriminator"));

        let argument_one = OsString::from_vec(vec![0xff, b'a', b'1']);
        let key_one = OsString::from_vec(vec![b'K', 0xfd]);
        let retired = |value: u8| {
            let input = crate::NativeProcessInput {
                arguments: vec![
                    executable.as_os_str().as_bytes().to_vec(),
                    argument_one.as_os_str().as_bytes().to_vec(),
                ],
                environment: vec![(key_one.as_os_str().as_bytes().to_vec(), vec![value])],
                working_directory: cwd_one.as_os_str().as_bytes().to_vec(),
            };
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&(input.arguments.len() as u64).to_le_bytes());
            for argument in &input.arguments {
                bytes.extend_from_slice(&(argument.len() as u64).to_le_bytes());
                bytes.extend_from_slice(argument);
            }
            bytes.extend_from_slice(&(input.environment.len() as u64).to_le_bytes());
            for (key, value) in &input.environment {
                bytes.extend_from_slice(&(key.len() as u64).to_le_bytes());
                bytes.extend_from_slice(key);
                bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
                bytes.extend_from_slice(value);
            }
            bytes.extend_from_slice(&(input.working_directory.len() as u64).to_le_bytes());
            bytes.extend_from_slice(&input.working_directory);
            crate::fnv1a_64(&bytes) % 125 + 1
        };
        let (first_byte, second_byte) = (128u8..=255)
            .flat_map(|first| ((first + 1)..=255).map(move |second| (first, second)))
            .find(|(first, second)| retired(*first) == retired(*second))
            .expect("retired 125-value discriminator has a non-UTF-8 collision");
        let value_one = OsString::from_vec(vec![first_byte]);
        let output_one = Command::new(&executable)
            .arg(&argument_one)
            .env_clear()
            .env(&key_one, &value_one)
            .current_dir(&cwd_one)
            .output()
            .expect("first process invocation runs");
        let argument_two = argument_one.clone();
        let value_two = OsString::from_vec(vec![second_byte]);
        let output_two = Command::new(&executable)
            .arg(&argument_two)
            .env_clear()
            .env(&key_one, &value_two)
            .current_dir(&cwd_one)
            .output()
            .expect("second process invocation runs");
        let wrong_cwd = Command::new(&executable)
            .arg(&argument_one)
            .env_clear()
            .env(&key_one, &value_one)
            .current_dir(&cwd_two)
            .output()
            .expect("cwd discriminator invocation runs");
        let wrong_argument = Command::new(&executable)
            .arg("utf8")
            .env_clear()
            .env(&key_one, &value_one)
            .current_dir(&cwd_one)
            .output()
            .expect("argv discriminator invocation runs");
        let wrong_key = Command::new(&executable)
            .arg(&argument_one)
            .env_clear()
            .env("X", &value_one)
            .current_dir(&cwd_one)
            .output()
            .expect("environment-key discriminator invocation runs");
        assert_eq!(retired(first_byte), retired(second_byte));
        assert_eq!(output_one.status.code(), Some(i32::from(first_byte)));
        assert_eq!(output_two.status.code(), Some(i32::from(second_byte)));
        assert_eq!(wrong_cwd.status.code(), Some(1));
        assert_eq!(wrong_argument.status.code(), Some(1));
        assert_eq!(wrong_key.status.code(), Some(1));

        fs::remove_dir_all(output_dir).expect("process fixture is removed");
    }

    #[test]
    fn stale_platform_support_hash_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(7));
        let program = starter_program(int_body(7), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_runtime_support_for_entrypoint(
            &program,
            &entrypoint,
            &run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes");
        support.header.support_hash ^= 1;

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-stale-support"),
            "object linker unit test",
        )
        .expect_err("stale support report rejects");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::Hash);
        assert_eq!(err.field, "platform_runtime_support_hash");
    }

    #[test]
    fn stale_mutated_entrypoint_payload_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(11));
        let program = starter_program(int_body(11), observation);
        let (_report, mut entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let support = platform_support(&program, &entrypoint, &run_report);
        entrypoint.entrypoint.target_kind = ExecutableEntrypointTargetKind::Library;

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-stale-payload"),
            "object linker unit test",
        )
        .expect_err("stale mutated entrypoint payload rejects");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::Hash);
        assert_eq!(err.field, "entrypoint.metadata_identity");
    }

    #[test]
    fn forged_support_for_non_executable_payload_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(13));
        let program = starter_program(int_body(13), observation);
        let (_report, mut entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_support(&program, &entrypoint, &run_report);

        entrypoint.entrypoint.target_kind = ExecutableEntrypointTargetKind::Library;
        entrypoint.entrypoint.metadata_identity =
            executable_entrypoint_metadata_hash(&entrypoint.entrypoint);
        entrypoint.header.package_hash = runtime_executable_entrypoint_package_hash(&entrypoint);
        support.entrypoint_package_hash = entrypoint.header.package_hash;
        support.entrypoint_metadata_identity = entrypoint.entrypoint.metadata_identity;
        support.header.support_hash = platform_runtime_support_report_hash(&support);

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-forged-support"),
            "object linker unit test",
        )
        .expect_err("forged support around non-executable payload rejects");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::EntrypointPackage);
        assert_eq!(err.field, "entrypoint.target_kind");
    }

    #[test]
    fn forged_entrypoint_package_kind_version_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(17));
        let program = starter_program(int_body(17), observation);
        let (_report, mut entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_support(&program, &entrypoint, &run_report);

        entrypoint.header.package_kind = "ForgedEntrypointPackage".to_string();
        entrypoint.header.version = EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION + 1;
        entrypoint.header.package_hash = runtime_executable_entrypoint_package_hash(&entrypoint);
        support.entrypoint_package_hash = entrypoint.header.package_hash;
        support.header.support_hash = platform_runtime_support_report_hash(&support);

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-forged-entrypoint-header"),
            "object linker unit test",
        )
        .expect_err("forged NC20 package header rejects");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::EntrypointPackage);
        assert_eq!(err.field, "package_kind");
    }

    #[test]
    fn forged_entrypoint_package_version_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(18));
        let program = starter_program(int_body(18), observation);
        let (_report, mut entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_support(&program, &entrypoint, &run_report);

        entrypoint.header.version = EXECUTABLE_ENTRYPOINT_PACKAGE_VERSION + 1;
        entrypoint.header.package_hash = runtime_executable_entrypoint_package_hash(&entrypoint);
        support.entrypoint_package_hash = entrypoint.header.package_hash;
        support.header.support_hash = platform_runtime_support_report_hash(&support);

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-forged-entrypoint-version"),
            "object linker unit test",
        )
        .expect_err("forged NC20 package version rejects");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::EntrypointPackage);
        assert_eq!(err.field, "version");
    }

    #[test]
    fn forged_platform_support_kind_version_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(19));
        let program = starter_program(int_body(19), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_support(&program, &entrypoint, &run_report);

        support.header.support_kind = "ForgedPlatformRuntimeSupport".to_string();
        support.header.version = PLATFORM_RUNTIME_SUPPORT_VERSION + 1;
        support.header.support_hash = platform_runtime_support_report_hash(&support);

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-forged-support-header"),
            "object linker unit test",
        )
        .expect_err("forged NC21 support header rejects");

        assert_eq!(
            err.stage,
            ObjectLinkerPackagingStage::PlatformRuntimeSupport
        );
        assert_eq!(err.field, "support_kind");
    }

    #[test]
    fn forged_platform_support_version_rejects_before_linking() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(20));
        let program = starter_program(int_body(20), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_support(&program, &entrypoint, &run_report);

        support.header.version = PLATFORM_RUNTIME_SUPPORT_VERSION + 1;
        support.header.support_hash = platform_runtime_support_report_hash(&support);

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-forged-support-version"),
            "object linker unit test",
        )
        .expect_err("forged NC21 support version rejects");

        assert_eq!(
            err.stage,
            ObjectLinkerPackagingStage::PlatformRuntimeSupport
        );
        assert_eq!(err.field, "version");
    }

    #[test]
    fn unsupported_platform_target_rejects_before_object_emission() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Int(3));
        let program = starter_program(int_body(3), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let mut support = platform_runtime_support_for_entrypoint(
            &program,
            &entrypoint,
            &run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes");
        support.header.platform_target = "other-host".to_string();
        support.support_facts.starter_platform_target = PlatformRuntimeEvidenceFact::Available {
            value: "other-host".to_string(),
            evidence_source: "test mutation".to_string(),
            lane: PlatformRuntimeEvidenceLane::Tested,
        };
        support.header.support_hash = platform_runtime_support_report_hash(&support);

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-platform"),
            "object linker unit test",
        )
        .expect_err("non-host starter platform rejects");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::PlatformTarget);
        assert_eq!(err.field, "platform_target");
    }

    #[test]
    fn missing_linker_is_explicit_toolchain_failure() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Bool(true));
        let program = starter_program(RuntimeExpr::Value(RuntimeValue::Bool(true)), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let support = platform_runtime_support_for_entrypoint(
            &program,
            &entrypoint,
            &run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes");
        let mut options = ObjectLinkerPackagingOptions::starter_host();
        options.linker_command = "definitely-missing-ken-linker".to_string();

        let err = package_starter_executable_artifact_with_options(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-missing-linker"),
            "object linker unit test",
            &options,
        )
        .expect_err("missing linker fails in the toolchain lane");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::Toolchain);
        assert_eq!(err.field, "linker_command");
    }

    #[test]
    fn aggregate_observation_rejects_as_non_scalar_smoke_lane() {
        let observation = RuntimeObservation::Returned(RuntimeGroundValue::Record {
            fields: vec![("value".to_string(), RuntimeGroundValue::Int(1))],
        });
        let program = starter_program(
            RuntimeExpr::Record {
                fields: vec![(
                    "value".to_string(),
                    RuntimeExpr::Value(RuntimeValue::Int(1)),
                )],
            },
            observation,
        );
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let support = platform_runtime_support_for_entrypoint(
            &program,
            &entrypoint,
            &run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes");

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-aggregate"),
            "object linker unit test",
        )
        .expect_err("aggregate smoke is not packaged as an external ABI");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::SmokeExecution);
        assert_eq!(err.field, "runtime_observation");
    }

    #[test]
    fn trap_observation_rejects_without_promoting_runtime_error_to_build_success() {
        let trap = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "fixture trap".to_string(),
        };
        let observation = RuntimeObservation::Trapped(trap.clone());
        let program = starter_program(RuntimeExpr::Trap(trap), observation);
        let (_report, entrypoint) = packaged_entrypoint(&program);
        let run_report = runtime_ir_run_report(&program);
        let support = platform_runtime_support_for_entrypoint(
            &program,
            &entrypoint,
            &run_report,
            crate::PlatformRuntimeTarget::starter(native_platform_target_name()),
            "object linker unit test",
        )
        .expect("platform support materializes");

        let err = package_starter_executable_artifact(
            &program,
            &entrypoint,
            &support,
            &run_report,
            &NativeSeedEnvironment::empty(),
            temp_output_dir("nc23-trap"),
            "object linker unit test",
        )
        .expect_err("trap smoke is not reported as linker success");

        assert_eq!(err.stage, ObjectLinkerPackagingStage::SmokeExecution);
        assert_eq!(err.field, "runtime_observation");
    }
}
