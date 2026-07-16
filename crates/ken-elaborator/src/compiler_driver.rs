//! NC10 compiler driver boundary.
//!
//! This module is deliberately still on the Language side of the compiler
//! boundary: Ken source is elaborated and admitted into the kernel, then emitted
//! as `CheckedCorePackage v0`. Target selection is package metadata and report
//! shaping only; it does not claim runtime lowering, native artifacts, or
//! validation facts.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

use ken_kernel::{Decl, GlobalId, Term};

use crate::checked_core::{
    canonical_decl_bytes, canonical_symbol_bytes, checked_core_declaration_body_view,
    emit_checked_core_package, semantic_fingerprint, validate_checked_core_package,
    AssumptionTrustKind, AssumptionTrustMetadata, CheckedCoreArtifactInputs, CheckedCoreBodyTerm,
    CheckedCoreBodyViewError, CheckedCoreBodyViewSelection, CheckedCorePackage,
    CheckedCorePackageError, CheckedCorePackageHeader, CheckedCoreSemanticInputs,
    ConstructorMetadata, DataMetadata, LowerabilityStatus, PartialityMetadata, PrimitiveMetadata,
    PrimitiveReductionMetadata, StableSymbol, StableSymbolTable, SymbolNamespace,
};
use crate::program_admission::{admit_checked_main, CheckedMainDescriptor, ProgramAdmissionError};
use crate::{ElabEnv, ElabError};

const PRODUCER: &str = "ken-elaborator:compiler-driver:nc10";
const KERNEL_REF: &str = "ken-kernel:current";
const SPEC_REF: &str = "docs/program/wp/NC10-compiler-driver-target-selection.md";
const PRIMITIVE_REGISTRY_REF: &str = "spec/10-kernel/18a-primitive-registry.md:current";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerSource {
    pub name: String,
    pub text: String,
}

impl CompilerSource {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerManifest {
    pub package_name: String,
    pub targets: Vec<ManifestTarget>,
}

impl CompilerManifest {
    pub fn new(package_name: impl Into<String>, targets: Vec<ManifestTarget>) -> Self {
        Self {
            package_name: package_name.into(),
            targets,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestTarget {
    pub symbol: StableSymbol,
    pub kind: CompilerTargetKind,
    pub package_identity: Option<StableSymbol>,
    pub lowerability: Option<LowerabilityStatus>,
}

impl ManifestTarget {
    pub fn executable(symbol: StableSymbol) -> Self {
        Self {
            symbol,
            kind: CompilerTargetKind::Executable,
            package_identity: None,
            lowerability: None,
        }
    }

    pub fn library(symbol: StableSymbol) -> Self {
        Self {
            symbol,
            kind: CompilerTargetKind::Library,
            package_identity: None,
            lowerability: None,
        }
    }

    pub fn non_runtime(symbol: StableSymbol) -> Self {
        Self {
            symbol,
            kind: CompilerTargetKind::NonRuntime,
            package_identity: None,
            lowerability: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerTargetKind {
    Executable,
    Library,
    NonRuntime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetSelector {
    StableSymbol {
        package_identity: StableSymbol,
        symbol: StableSymbol,
        kind: CompilerTargetKind,
    },
    Manifest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerDriverOutput {
    pub package: CheckedCorePackage,
    pub report: TargetSelectionReport,
    pub closures: Vec<TargetClosure>,
    pub executable_entrypoints: Vec<ExecutableEntrypointPackage>,
}

/// Hash-free semantic entrypoint plan. Construction is confined to the
/// checked native-production transaction below.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeEntrypointPlanV1 {
    pub(crate) main: StableSymbol,
    pub(crate) process_input: StableSymbol,
    pub(crate) process_input_constructor: StableSymbol,
    pub(crate) program_caps: StableSymbol,
    pub(crate) program_caps_constructor: StableSymbol,
    pub(crate) cap: StableSymbol,
    pub(crate) authority_constructor: StableSymbol,
    pub(crate) host_io: StableSymbol,
    pub(crate) host_exit: StableSymbol,
    pub(crate) exit_code: StableSymbol,
    pub(crate) success_constructor: StableSymbol,
    pub(crate) failure_constructor: StableSymbol,
    pub(crate) ret_constructor: StableSymbol,
    pub(crate) list_nil_constructor: StableSymbol,
    pub(crate) list_cons_constructor: StableSymbol,
    pub(crate) prod_constructor: StableSymbol,
    pub(crate) authority_name: String,
}

impl NativeEntrypointPlanV1 {
    pub fn main(&self) -> &StableSymbol {
        &self.main
    }

    pub fn authority_name(&self) -> &str {
        &self.authority_name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeProgramBuildOutput {
    pub package: CheckedCorePackage,
    pub plan: NativeEntrypointPlanV1,
    pub plan_transport_hash: u64,
    pub closure: TargetClosure,
    pub executable_closure: BTreeSet<StableSymbol>,
    pub runtime_program: ken_runtime::RuntimeProgram,
    pub executable_entrypoint: ExecutableEntrypointPackage,
    pub artifact: ken_runtime::BoundProcessExecutableArtifact,
    pub report: TargetSelectionReport,
}

#[derive(Debug)]
pub enum NativeProgramBuildError {
    Driver(CompilerDriverError),
    Admission(ProgramAdmissionError),
    Erasure(crate::erasure::ErasureError),
    Packaging(ken_runtime::ObjectLinkerPackagingError),
    Unavailable(UnavailableLane),
}

impl fmt::Display for NativeProgramBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Driver(error) => error.fmt(f),
            Self::Admission(error) => write!(f, "program admission failed: {error:?}"),
            Self::Erasure(error) => error.fmt(f),
            Self::Packaging(error) => error.fmt(f),
            Self::Unavailable(lane) => write!(f, "{}: {}", lane.lane, lane.reason),
        }
    }
}

impl std::error::Error for NativeProgramBuildError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetSelectionReport {
    pub package_identity: StableSymbol,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub report_identity: u64,
    pub checked_core_emission: ReportFact,
    pub selected_targets: Vec<SelectedTargetReport>,
    pub runtime_lowering: ReportFact,
    pub native_artifact: ReportFact,
    pub validation_facts: ReportFact,
    pub dependency_semantic_hashes: BTreeMap<StableSymbol, String>,
    pub obligations: BTreeSet<StableSymbol>,
    pub assumptions: BTreeSet<StableSymbol>,
    pub unsupported_lanes: BTreeMap<StableSymbol, Vec<UnavailableLane>>,
    pub trusted_base_delta: BTreeSet<StableSymbol>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedTargetReport {
    pub package_identity: StableSymbol,
    pub symbol: StableSymbol,
    pub kind: CompilerTargetKind,
    pub lowerability: Option<LowerabilityStatus>,
    pub lanes: Vec<UnavailableLane>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetClosure {
    pub package_identity: StableSymbol,
    pub target: SelectedTargetReport,
    pub closure_identity: u64,
    pub reachable_declarations: BTreeSet<StableSymbol>,
    pub external_symbols: BTreeSet<StableSymbol>,
    pub semantic: CheckedCoreSemanticInputs,
    pub report: TargetClosureReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetClosureReport {
    pub package_identity: StableSymbol,
    pub target_symbol: StableSymbol,
    pub target_kind: CompilerTargetKind,
    pub package_core_semantic_hash: u64,
    pub package_artifact_hash: u64,
    pub closure_semantic_hash: u64,
    pub closure_identity: u64,
    pub reachable_declarations: BTreeSet<StableSymbol>,
    pub external_symbols: BTreeSet<StableSymbol>,
    pub dependency_semantic_hashes: BTreeMap<StableSymbol, String>,
    pub imported_declaration_refs: BTreeMap<StableSymbol, StableSymbol>,
    pub dictionary_runtime_fields: BTreeMap<StableSymbol, BTreeSet<String>>,
    pub dictionary_erased_fields: BTreeMap<StableSymbol, BTreeSet<String>>,
    pub obligations: BTreeSet<StableSymbol>,
    pub assumptions: BTreeSet<StableSymbol>,
    pub lowerability: BTreeMap<StableSymbol, LowerabilityStatus>,
    pub unsupported_lanes: BTreeMap<StableSymbol, Vec<UnavailableLane>>,
    pub trusted_base_delta: BTreeSet<StableSymbol>,
    pub runtime_lowering: ReportFact,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableEntrypointPackage {
    pub package_identity: StableSymbol,
    pub package_core_semantic_hash: u64,
    pub package_artifact_hash: u64,
    pub target_symbol: StableSymbol,
    pub target_kind: CompilerTargetKind,
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
    pub unsupported_lanes: BTreeMap<StableSymbol, Vec<UnavailableLane>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableEntrypointVerdict {
    ClosedKenOnly,
    Unavailable { lanes: Vec<UnavailableLane> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableDependencyClosure {
    ClosedKenOnly,
    ImportsUnavailable {
        external_symbols: BTreeSet<StableSymbol>,
        imported_declaration_refs: BTreeMap<StableSymbol, StableSymbol>,
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
    CheckedProgramAbi,
    UnsupportedRuntimeArguments { parameter_count: usize },
    Unavailable { lane: UnavailableLane },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableResultObservation {
    pub shape: ExecutableResultShape,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableResultShape {
    RuntimeValue,
    Unavailable { lane: UnavailableLane },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableTrapContract {
    pub shape: ExecutableTrapShape,
    pub blocking_lanes: BTreeMap<StableSymbol, Vec<UnavailableLane>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutableTrapShape {
    RuntimeTrapReport,
    Unavailable { lane: UnavailableLane },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutableReportContract {
    pub target_closure_identity: u64,
    pub target_closure_report_hash: u64,
    pub evidence_source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportFact {
    Emitted,
    Unavailable(UnavailableLane),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnavailableLane {
    pub lane: String,
    pub reason: String,
}

impl UnavailableLane {
    fn new(lane: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            lane: lane.into(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug)]
pub enum CompilerDriverError {
    EmptyPackageName,
    EmptySourceSet,
    Io(String),
    Elaboration(ElabError),
    Package(CheckedCorePackageError),
    MissingTarget {
        symbol: StableSymbol,
    },
    AmbiguousManifestTarget {
        package_identity: StableSymbol,
        count: usize,
    },
    MismatchedPackageIdentity {
        expected: StableSymbol,
        found: StableSymbol,
    },
    TargetFromDifferentPackage {
        expected_package: String,
        symbol: StableSymbol,
    },
    MissingStableSymbol {
        id: GlobalId,
    },
    MissingClosureMetadata {
        section: &'static str,
        symbol: StableSymbol,
    },
    EntrypointClosurePackageMismatch {
        field: &'static str,
        expected: String,
        found: String,
    },
}

impl fmt::Display for CompilerDriverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerDriverError::EmptyPackageName => write!(f, "compiler package name is empty"),
            CompilerDriverError::EmptySourceSet => write!(f, "compiler source set is empty"),
            CompilerDriverError::Io(err) => write!(f, "compiler input I/O failed: {err}"),
            CompilerDriverError::Elaboration(err) => write!(f, "elaboration failed: {err:?}"),
            CompilerDriverError::Package(err) => err.fmt(f),
            CompilerDriverError::MissingTarget { symbol } => {
                write!(
                    f,
                    "target {symbol} is not present in the checked-core package"
                )
            }
            CompilerDriverError::AmbiguousManifestTarget {
                package_identity,
                count,
            } => write!(
                f,
                "manifest for {package_identity} declares {count} targets; selector is ambiguous"
            ),
            CompilerDriverError::MismatchedPackageIdentity { expected, found } => write!(
                f,
                "target selector expected package identity {expected}, found {found}"
            ),
            CompilerDriverError::TargetFromDifferentPackage {
                expected_package,
                symbol,
            } => write!(
                f,
                "target {symbol} does not belong to package {expected_package}"
            ),
            CompilerDriverError::MissingStableSymbol { id } => {
                write!(f, "missing stable symbol for admitted global {id}")
            }
            CompilerDriverError::MissingClosureMetadata { section, symbol } => write!(
                f,
                "target closure is missing required {section} metadata for {symbol}"
            ),
            CompilerDriverError::EntrypointClosurePackageMismatch {
                field,
                expected,
                found,
            } => write!(
                f,
                "entrypoint closure {field} mismatch: expected {expected}, found {found}"
            ),
        }
    }
}

impl std::error::Error for CompilerDriverError {}

impl From<ElabError> for CompilerDriverError {
    fn from(value: ElabError) -> Self {
        CompilerDriverError::Elaboration(value)
    }
}

impl From<CheckedCorePackageError> for CompilerDriverError {
    fn from(value: CheckedCorePackageError) -> Self {
        CompilerDriverError::Package(value)
    }
}

pub fn compile_ken_source(
    package_name: &str,
    source: CompilerSource,
    selector: TargetSelector,
) -> Result<CompilerDriverOutput, CompilerDriverError> {
    let manifest = CompilerManifest::new(package_name, Vec::new());
    compile_ken_package_sources(&manifest, vec![source], selector)
}

pub fn compile_ken_file(
    package_name: &str,
    path: impl AsRef<Path>,
    selector: TargetSelector,
) -> Result<CompilerDriverOutput, CompilerDriverError> {
    let path = path.as_ref();
    let text =
        std::fs::read_to_string(path).map_err(|err| CompilerDriverError::Io(err.to_string()))?;
    let source = CompilerSource::new(path.display().to_string(), text);
    compile_ken_source(package_name, source, selector)
}

pub fn compile_ken_package_sources(
    manifest: &CompilerManifest,
    sources: Vec<CompilerSource>,
    selector: TargetSelector,
) -> Result<CompilerDriverOutput, CompilerDriverError> {
    if manifest.package_name.is_empty() {
        return Err(CompilerDriverError::EmptyPackageName);
    }
    if sources.is_empty() {
        return Err(CompilerDriverError::EmptySourceSet);
    }

    let mut env = ElabEnv::new()?;
    let mut admitted = Vec::new();
    for source in &sources {
        let ids = if source.name.ends_with(".ken.md") {
            env.elaborate_ken_md_file(&source.text)?
        } else {
            env.elaborate_file(&source.text)?
        };
        admitted.extend(ids);
    }

    let package = emit_package_from_env(manifest, &sources, &env, &admitted, None)?;
    let selected = select_targets(manifest, &package, selector)?;
    let closures = build_target_closures(&package, &selected)?;
    let executable_entrypoints = package_executable_entrypoints(&package, &closures)?;
    let report = build_target_selection_report(&package, selected);
    Ok(CompilerDriverOutput {
        package,
        report,
        closures,
        executable_entrypoints,
    })
}

/// Compile one exact checked Program I `main` through lowering and linked
/// process-artifact production as a single identity-consistent transaction.
pub fn compile_native_program_sources(
    package_name: &str,
    sources: Vec<CompilerSource>,
    output_dir: impl AsRef<Path>,
) -> Result<NativeProgramBuildOutput, NativeProgramBuildError> {
    let manifest = CompilerManifest::new(package_name, Vec::new());
    if package_name.is_empty() {
        return Err(NativeProgramBuildError::Driver(
            CompilerDriverError::EmptyPackageName,
        ));
    }
    if sources.is_empty() {
        return Err(NativeProgramBuildError::Driver(
            CompilerDriverError::EmptySourceSet,
        ));
    }
    let mut env = ElabEnv::new()
        .map_err(CompilerDriverError::Elaboration)
        .map_err(NativeProgramBuildError::Driver)?;
    let mut admitted_ids = Vec::new();
    for source in &sources {
        let ids = if source.name.ends_with(".ken.md") {
            env.elaborate_ken_md_file(&source.text)
        } else {
            env.elaborate_file(&source.text)
        }
        .map_err(CompilerDriverError::Elaboration)
        .map_err(NativeProgramBuildError::Driver)?;
        admitted_ids.extend(ids);
    }
    let checked = admit_checked_main(&env).map_err(NativeProgramBuildError::Admission)?;
    let main_has_host_effect = env
        .effect_rows
        .get("main")
        .is_some_and(|row| !row.concrete_effects().is_empty());
    let (symbols, symbol_table) = stable_symbols_for_env(package_name, &env, true);
    let plan =
        native_entrypoint_plan(&checked, &symbols).map_err(NativeProgramBuildError::Driver)?;
    let plan_bytes = canonical_native_entrypoint_plan_bytes(&plan);
    let plan_transport_hash = fingerprint(&plan_bytes);
    let host_spine =
        checked_host_spine_v1(&env, &symbols).map_err(NativeProgramBuildError::Driver)?;
    let host_spine_bytes = canonical_checked_host_spine_v1_bytes(&host_spine);
    // The production package owns the exact live-environment closure, including
    // prelude definitions referenced by `main`; source-only generic packages
    // deliberately retain their narrower historical emission surface.
    admitted_ids.extend(env.env.decls().map(Decl::id));
    admitted_ids.sort();
    admitted_ids.dedup();
    let mut package =
        emit_package_from_env(&manifest, &sources, &env, &admitted_ids, Some(plan_bytes))
            .map_err(NativeProgramBuildError::Driver)?;
    if main_has_host_effect {
        let symbol = StableSymbol::new(
            SymbolNamespace::Metadata,
            vec![package_name.to_string(), "HostEffectSpineV1".to_string()],
        );
        package.artifact.semantic.symbols.insert(symbol.clone());
        package
            .artifact
            .semantic
            .metadata
            .insert(symbol, host_spine_bytes);
        package = emit_checked_core_package(package.header.clone(), package.artifact.clone())
            .map_err(CompilerDriverError::from)
            .map_err(NativeProgramBuildError::Driver)?;
    }
    let selector = TargetSelector::StableSymbol {
        package_identity: package.header.package_identity.clone(),
        symbol: plan.main.clone(),
        kind: CompilerTargetKind::Executable,
    };
    let mut selected =
        select_targets(&manifest, &package, selector).map_err(NativeProgramBuildError::Driver)?;
    for target in &mut selected {
        target
            .lanes
            .retain(|lane| lane.lane != "runtime_lowering_unavailable");
    }
    let closures =
        build_target_closures(&package, &selected).map_err(NativeProgramBuildError::Driver)?;
    let mut closure = closures
        .into_iter()
        .next()
        .expect("one exact checked-main selector produces one closure");
    if main_has_host_effect {
        // ITree/Coproduct/operation declarations are checked semantic
        // dependencies of the recognizer, not runtime declarations.  The
        // normalized root is self-contained after deforestation.
        closure.reachable_declarations = BTreeSet::from([plan.main.clone()]);
        closure.report.reachable_declarations = closure.reachable_declarations.clone();
    }
    let normalized_host_package = if main_has_host_effect {
        let mut normalized = package.clone();
        let declaration = env.env.lookup(checked.main).ok_or_else(|| {
            NativeProgramBuildError::Driver(CompilerDriverError::MissingStableSymbol {
                id: checked.main,
            })
        })?;
        let Decl::Transparent {
            id,
            level_params,
            ty,
            body,
        } = declaration
        else {
            unreachable!("checked main admission requires a transparent declaration")
        };
        let normalized_main = Decl::Transparent {
            id: *id,
            level_params: level_params.clone(),
            ty: ty.clone(),
            body: ken_kernel::normalize(&env.env, &ken_kernel::Context::new(), body),
        };
        let bytes = canonical_decl_bytes(&normalized_main, &symbol_table).map_err(|_| {
            NativeProgramBuildError::Driver(CompilerDriverError::MissingStableSymbol {
                id: checked.main,
            })
        })?;
        normalized
            .artifact
            .semantic
            .declarations
            .insert(plan.main.clone(), bytes);
        Some(
            emit_checked_core_package(normalized.header.clone(), normalized.artifact.clone())
                .map_err(CompilerDriverError::from)
                .map_err(NativeProgramBuildError::Driver)?,
        )
    } else {
        None
    };
    let executable_view = normalized_host_package.as_ref().unwrap_or(&package);
    let mut executable_closure_view = closure.clone();
    executable_closure_view.report.package_core_semantic_hash = executable_view.core_semantic_hash;
    executable_closure_view.report.package_artifact_hash = executable_view.artifact_hash;
    let mut executable_entrypoint =
        package_executable_entrypoint_mode(executable_view, &executable_closure_view, true)
            .map_err(NativeProgramBuildError::Driver)?;
    if main_has_host_effect {
        for lanes in executable_entrypoint.unsupported_lanes.values_mut() {
            lanes.retain(|lane| lane.lane != "host_effect_lowering_unavailable");
        }
        executable_entrypoint
            .unsupported_lanes
            .retain(|_, lanes| !lanes.is_empty());
    }
    if let Some(lane) = executable_entrypoint
        .unsupported_lanes
        .values()
        .flatten()
        .find(|lane| {
            lane.lane == "host_effect_lowering_unavailable"
                || lane.lane == "foreign_entrypoint_unavailable"
        })
        .cloned()
    {
        return Err(NativeProgramBuildError::Unavailable(lane));
    }
    // One closure owns the entire transaction: the declarations reported as
    // reachable are exactly the declarations erased into the runtime program
    // and linked artifact. Source admission is deliberately not a reachability
    // signal; an unused sibling must not enter native production.
    let executable_closure = closure.reachable_declarations.clone();
    let mut runtime_program = if main_has_host_effect {
        let mut program = crate::erasure::erase_checked_host_package_for_target(
            normalized_host_package
                .as_ref()
                .expect("host-effect main owns a normalized private view"),
            executable_closure.iter(),
            &plan.main,
            &host_spine,
        )
        .map_err(NativeProgramBuildError::Erasure)?;
        // Normalization changes only the private producer view, never the
        // checked package identity bound into the artifact.
        program.core_semantic_hash = package.core_semantic_hash;
        program.artifact_hash = package.artifact_hash;
        program
    } else {
        crate::erasure::erase_checked_core_package_for_target(&package, executable_closure.iter())
            .map_err(NativeProgramBuildError::Erasure)?
    };
    let runtime_targets = runtime_program
        .declarations
        .iter()
        .map(|declaration| declaration.symbol.clone())
        .collect::<BTreeSet<_>>();
    runtime_program
        .erased_core
        .metadata
        .assumptions
        .retain(|symbol, _| runtime_targets.contains(symbol));
    runtime_program
        .erased_core
        .metadata
        .assumption_trust_metadata
        .retain(|_, metadata| runtime_targets.contains(&metadata.target));
    runtime_program
        .erased_core
        .metadata
        .trusted_base_delta
        .retain(|symbol, _| runtime_targets.contains(symbol));
    let artifact = ken_runtime::build_bound_process_starter_executable_artifact(
        &runtime_program,
        &ken_runtime::BoundProcessEntrypoint {
            target_symbol: plan.main.to_string(),
            program_caps_constructor: plan.program_caps_constructor.to_string(),
            authority: match plan.authority_name.as_str() {
                "ANone" => 0,
                "APartial" => 1,
                "AFull" => 2,
                _ => unreachable!("checked main authority is a known Auth constructor"),
            },
            plan_hash: plan_transport_hash,
            ret_constructor: plan.ret_constructor.to_string(),
            process_symbols: ken_runtime::NativeProcessSymbols {
                process_input: plan.process_input_constructor.to_string(),
                list_nil: plan.list_nil_constructor.to_string(),
                list_cons: plan.list_cons_constructor.to_string(),
                prod: plan.prod_constructor.to_string(),
                exit_success: plan.success_constructor.to_string(),
                exit_failure: plan.failure_constructor.to_string(),
                result_err: host_spine.result_err.to_string(),
                result_ok: host_spine.result_ok.to_string(),
                option_some: host_spine.option_some.to_string(),
                file_error: host_spine.file_error.to_string(),
                file_operation_read: host_spine.file_operation_read.to_string(),
                file_operation_write: host_spine.file_operation_write.to_string(),
                io_errors: host_spine
                    .io_errors
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                unit: host_spine.unit.to_string(),
                bool_false: host_spine.bool_false.to_string(),
                bool_true: host_spine.bool_true.to_string(),
            },
        },
        output_dir,
    )
    .map_err(NativeProgramBuildError::Packaging)?;
    let mut report = build_target_selection_report(&package, selected);
    report.runtime_lowering = ReportFact::Emitted;
    report.native_artifact = ReportFact::Emitted;
    report.report_identity = target_report_fingerprint(&report);
    Ok(NativeProgramBuildOutput {
        package,
        plan,
        plan_transport_hash,
        closure,
        executable_closure,
        runtime_program,
        executable_entrypoint,
        artifact,
        report,
    })
}

pub fn compute_target_closures(
    manifest: &CompilerManifest,
    package: &CheckedCorePackage,
    selector: TargetSelector,
) -> Result<Vec<TargetClosure>, CompilerDriverError> {
    let selected = select_targets(manifest, package, selector)?;
    build_target_closures(package, &selected)
}

pub fn package_executable_entrypoints(
    package: &CheckedCorePackage,
    closures: &[TargetClosure],
) -> Result<Vec<ExecutableEntrypointPackage>, CompilerDriverError> {
    closures
        .iter()
        .map(|closure| package_executable_entrypoint(package, closure))
        .collect()
}

pub fn package_executable_entrypoint(
    package: &CheckedCorePackage,
    closure: &TargetClosure,
) -> Result<ExecutableEntrypointPackage, CompilerDriverError> {
    package_executable_entrypoint_mode(package, closure, false)
}

fn package_executable_entrypoint_mode(
    package: &CheckedCorePackage,
    closure: &TargetClosure,
    checked_program_abi: bool,
) -> Result<ExecutableEntrypointPackage, CompilerDriverError> {
    validate_entrypoint_closure_identity(package, closure)?;

    let mut unsupported_lanes = closure.report.unsupported_lanes.clone();
    let mut body_view_error = None;
    let selection = body_view_selection_for_closure(package, closure);
    let body_view =
        match checked_core_declaration_body_view(package, &selection, &closure.target.symbol) {
            Ok(view) => Some(view),
            Err(err) => {
                let lane = lane_from_body_view_error(&err);
                unsupported_lanes
                    .entry(closure.target.symbol.clone())
                    .or_insert_with(Vec::new)
                    .push(lane.clone());
                body_view_error = Some(lane);
                None
            }
        };

    if closure.target.kind != CompilerTargetKind::Executable {
        unsupported_lanes
            .entry(closure.target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "non_executable_entrypoint",
                "selected target is not declared as an executable target",
            ));
    }

    if let Some(status) = &closure.target.lowerability {
        if status.blocks_lowering() {
            unsupported_lanes
                .entry(closure.target.symbol.clone())
                .or_insert_with(Vec::new)
                .push(UnavailableLane::new(
                    "unsupported_target_lowerability",
                    format!("entrypoint target lowerability blocks execution: {status:?}"),
                ));
        }
    }

    if !closure.external_symbols.is_empty() {
        unsupported_lanes
            .entry(closure.target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "non_closed_entrypoint",
                "entrypoint closure contains external checked-core symbols",
            ));
    }

    if !closure.report.imported_declaration_refs.is_empty() {
        unsupported_lanes
            .entry(closure.target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "imported_dependency_entrypoint",
                "NC20 executable entrypoints must be Ken-only and cannot depend on imported declarations",
            ));
    }

    let has_foreign = closure
        .semantic
        .effects_foreign_metadata
        .values()
        .any(|metadata| metadata.boundary == crate::checked_core::EffectBoundary::Foreign);
    let has_effect = closure
        .semantic
        .effects_foreign_metadata
        .values()
        .any(|metadata| metadata.boundary == crate::checked_core::EffectBoundary::Effectful);
    if has_foreign {
        unsupported_lanes
            .entry(closure.target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "foreign_entrypoint_unavailable",
                "foreign calls are outside the native executable entrypoint boundary",
            ));
    }
    if has_effect {
        unsupported_lanes
            .entry(closure.target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "host_effect_lowering_unavailable",
                "host-effect lowering is supplied by PX5, not the base producer",
            ));
    }

    let argument_count = body_view
        .as_ref()
        .map(|view| top_level_lambda_count(&view.body))
        .unwrap_or(0);
    if argument_count > 0 && !(checked_program_abi && argument_count == 2) {
        unsupported_lanes
            .entry(closure.target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "entrypoint_runtime_arguments_unavailable",
                "NC20 v0 packages argument metadata but does not claim runtime CLI argument support",
            ));
    }

    let dependency_closure = if closure.external_symbols.is_empty()
        && closure.report.imported_declaration_refs.is_empty()
    {
        ExecutableDependencyClosure::ClosedKenOnly
    } else {
        ExecutableDependencyClosure::ImportsUnavailable {
            external_symbols: closure.external_symbols.clone(),
            imported_declaration_refs: closure.report.imported_declaration_refs.clone(),
        }
    };

    let mut required_runtime_support = BTreeSet::from([
        ExecutableRuntimeSupport::RuntimeValues,
        ExecutableRuntimeSupport::TrapReporting,
    ]);
    collect_runtime_support_from_semantic(&closure.semantic, &mut required_runtime_support);
    if let Some(view) = &body_view {
        collect_runtime_support_from_term(&view.body, &mut required_runtime_support);
    }

    let argument_packaging = ExecutableArgumentPackaging {
        shape: if let Some(lane) = body_view_error.clone() {
            ExecutableArgumentShape::Unavailable { lane }
        } else if argument_count == 0 {
            ExecutableArgumentShape::ClosedNullary
        } else if checked_program_abi && argument_count == 2 {
            ExecutableArgumentShape::CheckedProgramAbi
        } else {
            ExecutableArgumentShape::UnsupportedRuntimeArguments {
                parameter_count: argument_count,
            }
        },
        evidence_source: "CheckedCoreDeclarationBodyView target body".to_string(),
    };

    let result_observation = ExecutableResultObservation {
        shape: if let Some(lane) = body_view_error.clone() {
            ExecutableResultShape::Unavailable { lane }
        } else {
            ExecutableResultShape::RuntimeValue
        },
        evidence_source: "checked-core target body result observed as a RuntimeValue".to_string(),
    };

    let trap_contract = ExecutableTrapContract {
        shape: if let Some(lane) = body_view_error {
            ExecutableTrapShape::Unavailable { lane }
        } else {
            ExecutableTrapShape::RuntimeTrapReport
        },
        blocking_lanes: unsupported_lanes.clone(),
    };

    let closed_entry = if unsupported_lanes.is_empty() {
        ExecutableEntrypointVerdict::ClosedKenOnly
    } else {
        ExecutableEntrypointVerdict::Unavailable {
            lanes: flatten_lanes(&unsupported_lanes),
        }
    };

    let report_contract = ExecutableReportContract {
        target_closure_identity: closure.closure_identity,
        target_closure_report_hash: target_closure_report_fingerprint(&closure.report),
        evidence_source: "TargetClosureReport from exact checked-core package".to_string(),
    };

    let mut entrypoint = ExecutableEntrypointPackage {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: closure.target.symbol.clone(),
        target_kind: closure.target.kind.clone(),
        closure_identity: closure.closure_identity,
        closure_semantic_hash: closure.report.closure_semantic_hash,
        metadata_identity: 0,
        closed_entry,
        dependency_closure,
        required_runtime_support,
        argument_packaging,
        result_observation,
        trap_contract,
        report_contract,
        unsupported_lanes,
    };
    entrypoint.metadata_identity = executable_entrypoint_fingerprint(&entrypoint);
    Ok(entrypoint)
}

fn validate_entrypoint_closure_identity(
    package: &CheckedCorePackage,
    closure: &TargetClosure,
) -> Result<(), CompilerDriverError> {
    require_entrypoint_symbol_match(
        "package_identity",
        &package.header.package_identity,
        &closure.package_identity,
    )?;
    require_entrypoint_symbol_match(
        "target.package_identity",
        &package.header.package_identity,
        &closure.target.package_identity,
    )?;
    require_entrypoint_symbol_match(
        "report.package_identity",
        &package.header.package_identity,
        &closure.report.package_identity,
    )?;
    require_entrypoint_u64_match(
        "package_core_semantic_hash",
        package.core_semantic_hash,
        closure.report.package_core_semantic_hash,
    )?;
    require_entrypoint_u64_match(
        "package_artifact_hash",
        package.artifact_hash,
        closure.report.package_artifact_hash,
    )?;
    require_entrypoint_symbol_match(
        "target_symbol",
        &closure.target.symbol,
        &closure.report.target_symbol,
    )?;
    require_entrypoint_u64_match(
        "closure_identity",
        closure.closure_identity,
        closure.report.closure_identity,
    )?;
    Ok(())
}

fn require_entrypoint_symbol_match(
    field: &'static str,
    expected: &StableSymbol,
    found: &StableSymbol,
) -> Result<(), CompilerDriverError> {
    if expected == found {
        Ok(())
    } else {
        Err(CompilerDriverError::EntrypointClosurePackageMismatch {
            field,
            expected: expected.to_string(),
            found: found.to_string(),
        })
    }
}

fn require_entrypoint_u64_match(
    field: &'static str,
    expected: u64,
    found: u64,
) -> Result<(), CompilerDriverError> {
    if expected == found {
        Ok(())
    } else {
        Err(CompilerDriverError::EntrypointClosurePackageMismatch {
            field,
            expected: format!("{expected:016x}"),
            found: format!("{found:016x}"),
        })
    }
}

fn body_view_selection_for_closure(
    package: &CheckedCorePackage,
    closure: &TargetClosure,
) -> CheckedCoreBodyViewSelection {
    CheckedCoreBodyViewSelection {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: closure.target.symbol.clone(),
        reachable_declarations: closure.reachable_declarations.clone(),
        external_symbols: closure.external_symbols.clone(),
        dependency_semantic_hashes: closure.report.dependency_semantic_hashes.clone(),
    }
}

fn lane_from_body_view_error(err: &CheckedCoreBodyViewError) -> UnavailableLane {
    UnavailableLane::new(
        err.lane(),
        format!("checked-core body view is unavailable for entrypoint packaging: {err}"),
    )
}

fn top_level_lambda_count(term: &CheckedCoreBodyTerm) -> usize {
    match term {
        CheckedCoreBodyTerm::Lambda { body, .. } => 1 + top_level_lambda_count(body),
        _ => 0,
    }
}

fn collect_runtime_support_from_semantic(
    semantic: &CheckedCoreSemanticInputs,
    support: &mut BTreeSet<ExecutableRuntimeSupport>,
) {
    if !semantic.primitive_metadata.is_empty() || !semantic.primitive_refs.is_empty() {
        support.insert(ExecutableRuntimeSupport::PrimitiveValues);
    }
    if !semantic.data_metadata.is_empty() {
        support.insert(ExecutableRuntimeSupport::AlgebraicData);
    }
    if !semantic.record_sigma_metadata.is_empty() {
        support.insert(ExecutableRuntimeSupport::RecordsSigma);
    }
    if !semantic.class_instance_metadata.is_empty() {
        support.insert(ExecutableRuntimeSupport::Dictionaries);
    }
    if !semantic.recursion_metadata.is_empty() {
        support.insert(ExecutableRuntimeSupport::Recursion);
    }
}

fn collect_runtime_support_from_term(
    term: &CheckedCoreBodyTerm,
    support: &mut BTreeSet<ExecutableRuntimeSupport>,
) {
    match term {
        CheckedCoreBodyTerm::Variable { .. }
        | CheckedCoreBodyTerm::IntegerLiteral { .. }
        | CheckedCoreBodyTerm::DirectDeclarationCall { .. }
        | CheckedCoreBodyTerm::ImportedDeclarationCall(_)
        | CheckedCoreBodyTerm::PrimitiveLiteral(_)
        | CheckedCoreBodyTerm::ConstructorReference(_)
        | CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => {}
        CheckedCoreBodyTerm::RecursiveDeclarationCall(_) => {
            support.insert(ExecutableRuntimeSupport::Recursion);
        }
        CheckedCoreBodyTerm::PrimitiveApplication(view) => {
            support.insert(ExecutableRuntimeSupport::PrimitiveOperations);
            for argument in &view.arguments {
                collect_runtime_support_from_term(argument, support);
            }
        }
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            support.insert(ExecutableRuntimeSupport::FunctionCalls);
            collect_runtime_support_from_term(body, support);
        }
        CheckedCoreBodyTerm::Application { function, argument } => {
            support.insert(ExecutableRuntimeSupport::FunctionCalls);
            collect_runtime_support_from_term(function, support);
            collect_runtime_support_from_term(argument, support);
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            collect_runtime_support_from_term(value, support);
            collect_runtime_support_from_term(body, support);
        }
        CheckedCoreBodyTerm::Match(view) => {
            support.insert(ExecutableRuntimeSupport::PatternMatching);
            collect_runtime_support_from_term(&view.scrutinee, support);
            for branch in &view.branches {
                collect_runtime_support_from_term(&branch.method, support);
            }
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
            support.insert(ExecutableRuntimeSupport::RecordsSigma);
            for field in &view.fields {
                if let crate::checked_core::CheckedCoreRecordSigmaFieldValue::Runtime {
                    value,
                    ..
                } = field
                {
                    collect_runtime_support_from_term(value, support);
                }
            }
        }
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
            support.insert(ExecutableRuntimeSupport::RecordsSigma);
            collect_runtime_support_from_term(&view.base, support);
        }
        CheckedCoreBodyTerm::DictionaryConstruction(view) => {
            support.insert(ExecutableRuntimeSupport::Dictionaries);
            for field in &view.fields {
                if let crate::checked_core::CheckedCoreDictionaryFieldValue::Runtime {
                    value, ..
                } = field
                {
                    collect_runtime_support_from_term(value, support);
                }
            }
        }
    }
}

fn flatten_lanes(lanes: &BTreeMap<StableSymbol, Vec<UnavailableLane>>) -> Vec<UnavailableLane> {
    lanes
        .values()
        .flat_map(|symbol_lanes| symbol_lanes.iter().cloned())
        .collect()
}

fn emit_package_from_env(
    manifest: &CompilerManifest,
    sources: &[CompilerSource],
    env: &ElabEnv,
    admitted: &[GlobalId],
    native_entrypoint_plan: Option<Vec<u8>>,
) -> Result<CheckedCorePackage, CompilerDriverError> {
    let package_identity = package_identity(&manifest.package_name);
    let mut semantic = CheckedCoreSemanticInputs::default();
    let native_primitives = native_entrypoint_plan.is_some();
    let (symbols, table) = stable_symbols_for_env(&manifest.package_name, env, native_primitives);

    for symbol in symbols.values() {
        semantic.symbols.insert(symbol.clone());
    }

    for id in admitted {
        let symbol = symbols
            .get(id)
            .cloned()
            .ok_or(CompilerDriverError::MissingStableSymbol { id: *id })?;
        let decl = env
            .env
            .lookup(*id)
            .ok_or(CompilerDriverError::MissingStableSymbol { id: *id })?;
        let bytes = canonical_decl_bytes(decl, &table)
            .map_err(|_| CompilerDriverError::MissingStableSymbol { id: *id })?;
        semantic.declarations.insert(symbol.clone(), bytes);
        semantic
            .lowerability
            .insert(symbol, LowerabilityStatus::Supported);
    }

    add_data_metadata(env, &symbols, &mut semantic);
    if native_primitives {
        add_native_primitive_metadata(env, &symbols, &mut semantic);
    }
    apply_manifest_target_metadata(manifest, &mut semantic);
    add_trusted_base_metadata(env, &symbols, &mut semantic);
    if let Some(plan) = native_entrypoint_plan {
        let symbol = StableSymbol::new(
            SymbolNamespace::Metadata,
            vec![
                manifest.package_name.clone(),
                "NativeEntrypointPlanV1".to_string(),
            ],
        );
        semantic.symbols.insert(symbol.clone());
        semantic.metadata.insert(symbol, plan);
    }

    let mut source_identity = BTreeMap::new();
    for source in sources {
        source_identity.insert(
            source.name.clone(),
            format!("fnv64:{:016x}", fingerprint(source.text.as_bytes())),
        );
    }

    let header = CheckedCorePackageHeader::v0(
        PRODUCER,
        KERNEL_REF,
        SPEC_REF,
        PRIMITIVE_REGISTRY_REF,
        package_identity,
    );
    let package = emit_checked_core_package(
        header,
        CheckedCoreArtifactInputs {
            semantic,
            source_identity,
            annotations: BTreeMap::new(),
        },
    )?;
    Ok(package)
}

fn native_entrypoint_plan(
    checked: &CheckedMainDescriptor,
    symbols: &BTreeMap<GlobalId, StableSymbol>,
) -> Result<NativeEntrypointPlanV1, CompilerDriverError> {
    let resolve = |id: GlobalId| {
        symbols
            .get(&id)
            .cloned()
            .ok_or(CompilerDriverError::MissingStableSymbol { id })
    };
    Ok(NativeEntrypointPlanV1 {
        main: resolve(checked.main)?,
        process_input: resolve(checked.process_input)?,
        process_input_constructor: resolve(checked.process_input_constructor)?,
        program_caps: resolve(checked.program_caps)?,
        program_caps_constructor: resolve(checked.program_caps_constructor)?,
        cap: resolve(checked.cap)?,
        authority_constructor: resolve(checked.authority_constructor)?,
        host_io: resolve(checked.host_io)?,
        host_exit: resolve(checked.host_exit)?,
        exit_code: resolve(checked.exit_code)?,
        success_constructor: resolve(checked.success_constructor)?,
        failure_constructor: resolve(checked.failure_constructor)?,
        ret_constructor: resolve(checked.ret_constructor)?,
        list_nil_constructor: resolve(checked.list_nil_constructor)?,
        list_cons_constructor: resolve(checked.list_cons_constructor)?,
        prod_constructor: resolve(checked.prod_constructor)?,
        authority_name: checked.authority_name.clone(),
    })
}

fn checked_host_spine_v1(
    env: &ElabEnv,
    symbols: &BTreeMap<GlobalId, StableSymbol>,
) -> Result<crate::erasure::CheckedHostSpineV1, CompilerDriverError> {
    let resolve = |name: &'static str| {
        let id =
            env.globals
                .get(name)
                .copied()
                .ok_or(CompilerDriverError::MissingStableSymbol {
                    id: GlobalId(u32::MAX),
                })?;
        symbols
            .get(&id)
            .cloned()
            .ok_or(CompilerDriverError::MissingStableSymbol { id })
    };
    let mut operations = BTreeMap::new();
    for (name, operation) in [
        ("Read", ken_host::HostOpV1::ConsoleRead),
        ("Write", ken_host::HostOpV1::ConsoleWrite),
        ("Flush", ken_host::HostOpV1::ConsoleFlush),
        ("IsTerminal", ken_host::HostOpV1::ConsoleIsTerminal),
        ("WallNow", ken_host::HostOpV1::ClockWallNow),
        ("ReadFile", ken_host::HostOpV1::FsReadFile),
        ("WriteFile", ken_host::HostOpV1::FsWriteFile),
        ("AppendFile", ken_host::HostOpV1::FsAppendFile),
        ("Metadata", ken_host::HostOpV1::FsMetadata),
        ("ReadDirectory", ken_host::HostOpV1::FsReadDirectory),
        ("CreateDirectory", ken_host::HostOpV1::FsCreateDirectory),
        ("RemoveFile", ken_host::HostOpV1::FsRemoveFile),
        ("RemoveDirectory", ken_host::HostOpV1::FsRemoveDirectory),
        ("Rename", ken_host::HostOpV1::FsRename),
    ] {
        operations.insert(resolve(name)?, operation);
    }
    Ok(crate::erasure::CheckedHostSpineV1 {
        ret: resolve("Ret")?,
        vis: resolve("Vis")?,
        in_l: resolve("InL")?,
        in_r: resolve("InR")?,
        fs_family: resolve("FSOp")?,
        console_family: resolve("ConsoleOp")?,
        clock_family: resolve("ClockOp")?,
        capability: resolve("Cap")?,
        result_err: resolve("Err")?,
        result_ok: resolve("Ok")?,
        option_some: resolve("Some")?,
        file_error: resolve("MkFileError")?,
        file_operation_read: resolve("OpReadFile")?,
        file_operation_write: resolve("OpWriteFile")?,
        io_errors: [
            "NotFound",
            "PermissionDenied",
            "CapabilityDenied",
            "BrokenPipe",
            "Interrupted",
            "AlreadyExists",
            "InvalidInput",
            "IsDirectory",
            "NotDirectory",
            "NotEmpty",
            "Unsupported",
            "Other",
        ]
        .into_iter()
        .map(resolve)
        .collect::<Result<Vec<_>, _>>()?,
        unit: resolve("MkUnit")?,
        bool_false: resolve("False")?,
        bool_true: resolve("True")?,
        operations,
    })
}

fn canonical_checked_host_spine_v1_bytes(spine: &crate::erasure::CheckedHostSpineV1) -> Vec<u8> {
    let mut out = b"HostEffectSpineV1\0".to_vec();
    for symbol in [
        &spine.ret,
        &spine.vis,
        &spine.in_l,
        &spine.in_r,
        &spine.fs_family,
        &spine.console_family,
        &spine.clock_family,
        &spine.capability,
        &spine.result_err,
        &spine.result_ok,
        &spine.option_some,
        &spine.file_error,
        &spine.file_operation_read,
        &spine.file_operation_write,
        &spine.unit,
        &spine.bool_false,
        &spine.bool_true,
    ] {
        let field = symbol.to_string();
        out.extend_from_slice(&(field.len() as u64).to_le_bytes());
        out.extend_from_slice(field.as_bytes());
    }
    out.extend_from_slice(&(spine.io_errors.len() as u64).to_le_bytes());
    for symbol in &spine.io_errors {
        let field = symbol.to_string();
        out.extend_from_slice(&(field.len() as u64).to_le_bytes());
        out.extend_from_slice(field.as_bytes());
    }
    out.extend_from_slice(&(spine.operations.len() as u64).to_le_bytes());
    for (symbol, operation) in &spine.operations {
        let field = symbol.to_string();
        out.extend_from_slice(&(field.len() as u64).to_le_bytes());
        out.extend_from_slice(field.as_bytes());
        out.extend_from_slice(&(*operation as u16).to_le_bytes());
    }
    out
}

fn canonical_native_entrypoint_plan_bytes(plan: &NativeEntrypointPlanV1) -> Vec<u8> {
    let fields = [
        plan.main.to_string(),
        plan.process_input.to_string(),
        plan.process_input_constructor.to_string(),
        plan.program_caps.to_string(),
        plan.program_caps_constructor.to_string(),
        plan.cap.to_string(),
        plan.authority_constructor.to_string(),
        plan.host_io.to_string(),
        plan.host_exit.to_string(),
        plan.exit_code.to_string(),
        plan.success_constructor.to_string(),
        plan.failure_constructor.to_string(),
        plan.ret_constructor.to_string(),
        plan.list_nil_constructor.to_string(),
        plan.list_cons_constructor.to_string(),
        plan.prod_constructor.to_string(),
        plan.authority_name.clone(),
    ];
    let mut out = b"NativeEntrypointPlanV1\0".to_vec();
    for field in fields {
        out.extend_from_slice(&(field.len() as u64).to_le_bytes());
        out.extend_from_slice(field.as_bytes());
    }
    out
}

fn stable_symbols_for_env(
    package_name: &str,
    env: &ElabEnv,
    native_primitives: bool,
) -> (BTreeMap<GlobalId, StableSymbol>, StableSymbolTable) {
    let mut names_by_id = BTreeMap::<GlobalId, String>::new();
    let mut global_names = env.globals.iter().collect::<Vec<_>>();
    global_names.sort_by(|(left, _), (right, _)| left.cmp(right));
    for (name, id) in global_names {
        names_by_id.entry(*id).or_insert_with(|| name.clone());
    }

    for decl in env.env.decls() {
        names_by_id
            .entry(decl.id())
            .or_insert_with(|| format!("global_{}", decl.id().0));
        if let Decl::Inductive(ind) = decl {
            for ctor in &ind.constructors {
                names_by_id
                    .entry(ctor.id)
                    .or_insert_with(|| format!("ctor_{}", ctor.id.0));
            }
        }
    }

    let mut symbols = BTreeMap::new();
    for decl in env.env.decls() {
        if native_primitives {
            if let Decl::Primitive { id, reduction, .. } = decl {
                let registry_symbol = match reduction {
                    ken_kernel::PrimReduction::Op { symbol } => Some((*symbol).to_string()),
                    ken_kernel::PrimReduction::Literal => match env.num_values.get(id) {
                        Some(crate::NumericLitVal::Int(value)) => Some(format!("lit_int_{value}")),
                        Some(crate::NumericLitVal::Str(value)) => {
                            Some(format!("lit_string_{value}"))
                        }
                        _ => None,
                    },
                    ken_kernel::PrimReduction::OpaqueType => None,
                };
                if let Some(registry_symbol) = registry_symbol {
                    symbols.insert(*id, StableSymbol::primitive(registry_symbol));
                    continue;
                }
            }
        }
        let name = names_by_id
            .get(&decl.id())
            .cloned()
            .unwrap_or_else(|| format!("global_{}", decl.id().0));
        symbols.insert(decl.id(), declaration_symbol(package_name, &name));
    }

    for decl in env.env.decls() {
        if let Decl::Inductive(ind) = decl {
            let Some(parent) = symbols.get(&ind.id).cloned() else {
                continue;
            };
            for ctor in &ind.constructors {
                let name = names_by_id
                    .get(&ctor.id)
                    .cloned()
                    .unwrap_or_else(|| format!("ctor_{}", ctor.id.0));
                symbols.insert(ctor.id, StableSymbol::constructor(&parent, name));
            }
        }
    }

    let mut table = StableSymbolTable::new();
    for (id, symbol) in &symbols {
        table.insert_global(*id, symbol.clone());
    }
    (symbols, table)
}

fn add_native_primitive_metadata(
    env: &ElabEnv,
    symbols: &BTreeMap<GlobalId, StableSymbol>,
    semantic: &mut CheckedCoreSemanticInputs,
) {
    for decl in env.env.decls() {
        let Decl::Primitive { id, reduction, .. } = decl else {
            continue;
        };
        let (registry_symbol, reduction) = match reduction {
            ken_kernel::PrimReduction::Op { symbol } => {
                ((*symbol).to_string(), PrimitiveReductionMetadata::Op)
            }
            ken_kernel::PrimReduction::Literal => match env.num_values.get(id) {
                Some(crate::NumericLitVal::Int(value)) => (
                    format!("lit_int_{value}"),
                    PrimitiveReductionMetadata::Literal,
                ),
                Some(crate::NumericLitVal::Str(value)) => (
                    format!("lit_string_{value}"),
                    PrimitiveReductionMetadata::Literal,
                ),
                _ => continue,
            },
            ken_kernel::PrimReduction::OpaqueType => continue,
        };
        let Some(stable) = symbols.get(id).cloned() else {
            continue;
        };
        semantic.primitive_refs.insert(
            stable.clone(),
            format!("primitive-registry:{registry_symbol}"),
        );
        semantic.primitive_metadata.insert(
            stable.clone(),
            PrimitiveMetadata {
                registry_symbol,
                reduction,
                partiality: PartialityMetadata::Total,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        semantic
            .lowerability
            .insert(stable, LowerabilityStatus::Supported);
    }
}

fn declaration_symbol(package_name: &str, name: &str) -> StableSymbol {
    let mut parts = name.split('.').filter(|part| !part.is_empty()).peekable();
    let mut components = vec![package_name.to_string()];
    while let Some(part) = parts.next() {
        components.push(part.to_string());
    }
    StableSymbol::new(SymbolNamespace::Declaration, components)
}

fn package_identity(package_name: &str) -> StableSymbol {
    StableSymbol::new(SymbolNamespace::Module, vec![package_name.to_string()])
}

fn apply_manifest_target_metadata(
    manifest: &CompilerManifest,
    semantic: &mut CheckedCoreSemanticInputs,
) {
    for target in &manifest.targets {
        semantic.symbols.insert(target.symbol.clone());
        if let Some(status) = &target.lowerability {
            semantic
                .lowerability
                .insert(target.symbol.clone(), status.clone());
            if status.blocks_lowering() {
                semantic.unsupported.insert(
                    target.symbol.clone(),
                    format!("manifest target metadata blocks lowering: {status:?}").into_bytes(),
                );
            }
        }
    }
}

fn add_data_metadata(
    env: &ElabEnv,
    symbols: &BTreeMap<GlobalId, StableSymbol>,
    semantic: &mut CheckedCoreSemanticInputs,
) {
    for decl in env.env.decls() {
        let Decl::Inductive(ind) = decl else {
            continue;
        };
        let Some(family) = symbols.get(&ind.id).cloned() else {
            continue;
        };
        semantic
            .lowerability
            .entry(family.clone())
            .or_insert(LowerabilityStatus::Supported);

        let constructors = ind
            .constructors
            .iter()
            .filter_map(|constructor| {
                let symbol = symbols.get(&constructor.id).cloned()?;
                semantic
                    .lowerability
                    .entry(symbol.clone())
                    .or_insert(LowerabilityStatus::Supported);
                Some(ConstructorMetadata {
                    symbol,
                    argument_count: constructor.args.len(),
                    target_index_count: constructor.target_indices.len(),
                    recursive_positions: constructor
                        .args
                        .iter()
                        .enumerate()
                        .filter_map(|(position, arg)| {
                            is_recursive_constructor_arg(arg, ind.id).then_some(position)
                        })
                        .collect(),
                    lowerability: LowerabilityStatus::Supported,
                })
            })
            .collect();

        semantic.data_metadata.insert(
            family,
            DataMetadata {
                parameter_count: ind.params.len(),
                index_count: ind.indices.len(),
                constructors,
                eliminator: LowerabilityStatus::Supported,
                lowerability: LowerabilityStatus::Supported,
            },
        );
    }
}

fn is_recursive_constructor_arg(arg: &Term, family: GlobalId) -> bool {
    match arg {
        Term::IndFormer { id, .. } => *id == family,
        Term::App(function, _) => is_recursive_constructor_arg(function, family),
        Term::Pi(_, codomain) => is_recursive_constructor_arg(codomain, family),
        _ => false,
    }
}

fn add_trusted_base_metadata(
    env: &ElabEnv,
    symbols: &BTreeMap<GlobalId, StableSymbol>,
    semantic: &mut CheckedCoreSemanticInputs,
) {
    for id in env.env.trusted_base() {
        let Some(target) = symbols.get(&id).cloned() else {
            continue;
        };
        let assumption = StableSymbol::assumption(&target, "trusted_base");
        semantic.symbols.insert(target.clone());
        semantic.symbols.insert(assumption.clone());
        semantic.assumptions.insert(
            assumption.clone(),
            format!("trusted-base assumption for {target}").into_bytes(),
        );
        semantic.assumption_trust_metadata.insert(
            assumption.clone(),
            AssumptionTrustMetadata {
                kind: AssumptionTrustKind::Postulate,
                target: target.clone(),
                affects_runtime_meaning: true,
            },
        );
        semantic
            .trusted_base_delta
            .insert(target, format!("trusted-base global {id}").into_bytes());
    }
}

fn select_targets(
    manifest: &CompilerManifest,
    package: &CheckedCorePackage,
    selector: TargetSelector,
) -> Result<Vec<SelectedTargetReport>, CompilerDriverError> {
    validate_checked_core_package(package)?;
    match selector {
        TargetSelector::StableSymbol {
            package_identity,
            symbol,
            kind,
        } => {
            require_package_identity(&package.header.package_identity, &package_identity)?;
            require_symbol_package(&manifest.package_name, &symbol)?;
            require_package_symbol(package, &symbol)?;
            Ok(vec![selected_target_report(package, symbol, kind)])
        }
        TargetSelector::Manifest => match manifest.targets.as_slice() {
            [target] => {
                if let Some(identity) = &target.package_identity {
                    require_package_identity(&package.header.package_identity, identity)?;
                }
                require_symbol_package(&manifest.package_name, &target.symbol)?;
                require_package_symbol(package, &target.symbol)?;
                Ok(vec![selected_target_report(
                    package,
                    target.symbol.clone(),
                    target.kind.clone(),
                )])
            }
            [] => Err(CompilerDriverError::MissingTarget {
                symbol: package.header.package_identity.clone(),
            }),
            many => Err(CompilerDriverError::AmbiguousManifestTarget {
                package_identity: package.header.package_identity.clone(),
                count: many.len(),
            }),
        },
    }
}

fn require_package_identity(
    expected: &StableSymbol,
    found: &StableSymbol,
) -> Result<(), CompilerDriverError> {
    if expected == found {
        Ok(())
    } else {
        Err(CompilerDriverError::MismatchedPackageIdentity {
            expected: expected.clone(),
            found: found.clone(),
        })
    }
}

fn require_symbol_package(
    expected_package: &str,
    symbol: &StableSymbol,
) -> Result<(), CompilerDriverError> {
    if symbol.components.first().map(String::as_str) == Some(expected_package) {
        Ok(())
    } else {
        Err(CompilerDriverError::TargetFromDifferentPackage {
            expected_package: expected_package.to_string(),
            symbol: symbol.clone(),
        })
    }
}

fn require_package_symbol(
    package: &CheckedCorePackage,
    symbol: &StableSymbol,
) -> Result<(), CompilerDriverError> {
    if package.artifact.semantic.declarations.contains_key(symbol) {
        Ok(())
    } else {
        Err(CompilerDriverError::MissingTarget {
            symbol: symbol.clone(),
        })
    }
}

fn selected_target_report(
    package: &CheckedCorePackage,
    symbol: StableSymbol,
    kind: CompilerTargetKind,
) -> SelectedTargetReport {
    let lowerability = package.artifact.semantic.lowerability.get(&symbol).cloned();
    let mut lanes = Vec::new();
    if kind == CompilerTargetKind::NonRuntime {
        lanes.push(UnavailableLane::new(
            "non_runtime_target",
            "selected target is package metadata/proof-only and is not a runtime target",
        ));
    }
    match &lowerability {
        Some(status) if status.blocks_lowering() => lanes.push(UnavailableLane::new(
            "unsupported_target_metadata",
            format!("checked-core lowerability blocks runtime lowering: {status:?}"),
        )),
        None => lanes.push(UnavailableLane::new(
            "missing_target_lowerability",
            "checked-core package has no lowerability metadata for the selected target",
        )),
        Some(LowerabilityStatus::Supported) => {}
        Some(_) => {}
    }
    if lanes.is_empty() {
        lanes.push(UnavailableLane::new(
            "runtime_lowering_unavailable",
            "NC10 selects targets only; runtime lowering starts in later NC work",
        ));
    }

    SelectedTargetReport {
        package_identity: package.header.package_identity.clone(),
        symbol,
        kind,
        lowerability,
        lanes,
    }
}

fn build_target_selection_report(
    package: &CheckedCorePackage,
    selected_targets: Vec<SelectedTargetReport>,
) -> TargetSelectionReport {
    let mut unsupported_lanes = BTreeMap::new();
    for target in &selected_targets {
        let lanes = target
            .lanes
            .iter()
            .filter(|lane| lane.lane != "runtime_lowering_unavailable")
            .cloned()
            .collect::<Vec<_>>();
        if !lanes.is_empty() {
            unsupported_lanes.insert(target.symbol.clone(), lanes);
        }
    }
    for symbol in package.artifact.semantic.unsupported.keys() {
        unsupported_lanes.entry(symbol.clone()).or_insert_with(|| {
            vec![UnavailableLane::new(
                "checked_core_unsupported",
                "checked-core package carries an explicit unsupported lane",
            )]
        });
    }

    let runtime_lane = selected_targets
        .iter()
        .flat_map(|target| target.lanes.iter())
        .find(|lane| lane.lane != "runtime_lowering_unavailable")
        .cloned()
        .unwrap_or_else(|| {
            UnavailableLane::new(
                "runtime_lowering_unavailable",
                "NC10 does not emit RuntimeProgram artifacts",
            )
        });

    let mut report = TargetSelectionReport {
        package_identity: package.header.package_identity.clone(),
        core_semantic_hash: package.core_semantic_hash,
        artifact_hash: package.artifact_hash,
        report_identity: 0,
        checked_core_emission: ReportFact::Emitted,
        selected_targets,
        runtime_lowering: ReportFact::Unavailable(runtime_lane),
        native_artifact: ReportFact::Unavailable(UnavailableLane::new(
            "native_artifact_unavailable",
            "NC10 stops before native artifact emission",
        )),
        validation_facts: ReportFact::Unavailable(UnavailableLane::new(
            "validation_facts_unavailable",
            "NC10 makes no NC8/NC9 validation or proof claim",
        )),
        dependency_semantic_hashes: package.artifact.semantic.dependency_semantic_hashes.clone(),
        obligations: package
            .artifact
            .semantic
            .obligations
            .keys()
            .cloned()
            .collect(),
        assumptions: package
            .artifact
            .semantic
            .assumptions
            .keys()
            .cloned()
            .collect(),
        unsupported_lanes,
        trusted_base_delta: package
            .artifact
            .semantic
            .trusted_base_delta
            .keys()
            .cloned()
            .collect(),
    };
    report.report_identity = target_report_fingerprint(&report);
    report
}

fn build_target_closures(
    package: &CheckedCorePackage,
    selected_targets: &[SelectedTargetReport],
) -> Result<Vec<TargetClosure>, CompilerDriverError> {
    validate_checked_core_package(package)?;
    selected_targets
        .iter()
        .map(|target| build_target_closure(package, target))
        .collect()
}

fn build_target_closure(
    package: &CheckedCorePackage,
    target: &SelectedTargetReport,
) -> Result<TargetClosure, CompilerDriverError> {
    let semantic = &package.artifact.semantic;
    validate_closure_metadata(semantic)?;
    let dependency_index = declaration_dependency_index(package);
    let semantic_dependency_index = declaration_semantic_dependency_index(package);
    let mut reachable_declarations = BTreeSet::from([target.symbol.clone()]);
    let mut closure_symbols = BTreeSet::from([target.symbol.clone()]);
    let mut external_symbols = BTreeSet::new();
    let mut queue = vec![target.symbol.clone()];

    while let Some(symbol) = queue.pop() {
        if let Some(dependencies) = dependency_index.get(&symbol) {
            for dependency in dependencies {
                include_closure_symbol(
                    semantic,
                    dependency.clone(),
                    &mut closure_symbols,
                    &mut reachable_declarations,
                    &mut external_symbols,
                    &mut queue,
                );
            }
        }

        if let Some(dependencies) = semantic_dependency_index.get(&symbol) {
            for dependency in dependencies {
                if dependency_index
                    .get(&symbol)
                    .is_some_and(|runtime| runtime.contains(dependency))
                {
                    continue;
                }
                include_non_runtime_closure_symbol(
                    semantic,
                    dependency.clone(),
                    &mut closure_symbols,
                    &mut external_symbols,
                );
            }
        }

        for dependency in metadata_dependencies_for_symbol(semantic, &symbol) {
            include_closure_symbol(
                semantic,
                dependency,
                &mut closure_symbols,
                &mut reachable_declarations,
                &mut external_symbols,
                &mut queue,
            );
        }
    }

    let closure_semantic =
        slice_closure_semantic(semantic, &closure_symbols, &reachable_declarations);
    validate_closure_metadata(&closure_semantic)?;
    let report = build_target_closure_report(
        package,
        target,
        &closure_semantic,
        &reachable_declarations,
        &external_symbols,
    );

    Ok(TargetClosure {
        package_identity: package.header.package_identity.clone(),
        target: target.clone(),
        closure_identity: report.closure_identity,
        reachable_declarations,
        external_symbols,
        semantic: closure_semantic,
        report,
    })
}

fn declaration_dependency_index(
    package: &CheckedCorePackage,
) -> BTreeMap<StableSymbol, BTreeSet<StableSymbol>> {
    let mut encoded_symbols = package
        .artifact
        .semantic
        .symbols
        .iter()
        .map(|symbol| (symbol.clone(), canonical_symbol_bytes(symbol)))
        .collect::<Vec<_>>();
    encoded_symbols.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut index = BTreeMap::new();
    for (owner, bytes) in &package.artifact.semantic.declarations {
        let selection = CheckedCoreBodyViewSelection {
            package_identity: package.header.package_identity.clone(),
            package_core_semantic_hash: package.core_semantic_hash,
            package_artifact_hash: package.artifact_hash,
            target_symbol: owner.clone(),
            reachable_declarations: package
                .artifact
                .semantic
                .declarations
                .keys()
                .cloned()
                .collect(),
            external_symbols: BTreeSet::new(),
            dependency_semantic_hashes: package
                .artifact
                .semantic
                .dependency_semantic_hashes
                .clone(),
        };
        let dependencies = match checked_core_declaration_body_view(package, &selection, owner) {
            Ok(view) => {
                let mut dependencies = BTreeSet::new();
                collect_runtime_declaration_dependencies(&view.body, &mut dependencies);
                dependencies.remove(owner);
                dependencies
            }
            Err(_) => {
                // Unsupported bodies remain reportable by the established
                // entrypoint lane. Preserve the conservative dependency scan
                // for those declarations; supported bodies use the executable
                // term view so type-only names never enter runtime reachability.
                encoded_symbols
                    .iter()
                    .filter_map(|(candidate, needle)| {
                        (candidate != owner && contains_subslice(bytes, needle))
                            .then_some(candidate.clone())
                    })
                    .collect()
            }
        };
        index.insert(owner.clone(), dependencies);
    }
    index
}

fn declaration_semantic_dependency_index(
    package: &CheckedCorePackage,
) -> BTreeMap<StableSymbol, BTreeSet<StableSymbol>> {
    let encoded_symbols = package
        .artifact
        .semantic
        .symbols
        .iter()
        .map(|symbol| (symbol.clone(), canonical_symbol_bytes(symbol)))
        .collect::<Vec<_>>();

    package
        .artifact
        .semantic
        .declarations
        .iter()
        .map(|(owner, bytes)| {
            let dependencies = encoded_symbols
                .iter()
                .filter_map(|(candidate, needle)| {
                    (candidate != owner && contains_subslice(bytes, needle))
                        .then_some(candidate.clone())
                })
                .collect();
            (owner.clone(), dependencies)
        })
        .collect()
}

fn collect_runtime_declaration_dependencies(
    term: &CheckedCoreBodyTerm,
    dependencies: &mut BTreeSet<StableSymbol>,
) {
    match term {
        CheckedCoreBodyTerm::DirectDeclarationCall { symbol, .. } => {
            dependencies.insert(symbol.clone());
        }
        CheckedCoreBodyTerm::RecursiveDeclarationCall(view) => {
            dependencies.insert(view.symbol.clone());
        }
        CheckedCoreBodyTerm::PrimitiveApplication(view) => {
            for argument in &view.arguments {
                collect_runtime_declaration_dependencies(argument, dependencies);
            }
        }
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            collect_runtime_declaration_dependencies(body, dependencies);
        }
        CheckedCoreBodyTerm::Application { function, argument } => {
            collect_runtime_declaration_dependencies(function, dependencies);
            collect_runtime_declaration_dependencies(argument, dependencies);
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            collect_runtime_declaration_dependencies(value, dependencies);
            collect_runtime_declaration_dependencies(body, dependencies);
        }
        CheckedCoreBodyTerm::Match(view) => {
            collect_runtime_declaration_dependencies(&view.scrutinee, dependencies);
            for branch in &view.branches {
                collect_runtime_declaration_dependencies(&branch.method, dependencies);
            }
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
            for field in &view.fields {
                if let crate::checked_core::CheckedCoreRecordSigmaFieldValue::Runtime {
                    value,
                    ..
                } = field
                {
                    collect_runtime_declaration_dependencies(value, dependencies);
                }
            }
        }
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
            collect_runtime_declaration_dependencies(&view.base, dependencies);
        }
        CheckedCoreBodyTerm::DictionaryConstruction(view) => {
            for field in &view.fields {
                if let crate::checked_core::CheckedCoreDictionaryFieldValue::Runtime {
                    value, ..
                } = field
                {
                    collect_runtime_declaration_dependencies(value, dependencies);
                }
            }
        }
        CheckedCoreBodyTerm::Variable { .. }
        | CheckedCoreBodyTerm::IntegerLiteral { .. }
        | CheckedCoreBodyTerm::ImportedDeclarationCall(_)
        | CheckedCoreBodyTerm::PrimitiveLiteral(_)
        | CheckedCoreBodyTerm::ConstructorReference(_)
        | CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => {}
    }
}

fn include_closure_symbol(
    semantic: &CheckedCoreSemanticInputs,
    symbol: StableSymbol,
    closure_symbols: &mut BTreeSet<StableSymbol>,
    reachable_declarations: &mut BTreeSet<StableSymbol>,
    external_symbols: &mut BTreeSet<StableSymbol>,
    queue: &mut Vec<StableSymbol>,
) {
    if semantic.declarations.contains_key(&symbol) {
        if reachable_declarations.insert(symbol.clone()) {
            queue.push(symbol.clone());
        }
        closure_symbols.insert(symbol);
    } else {
        if !has_semantic_entry(semantic, &symbol)
            || semantic.dependency_declaration_refs.contains_key(&symbol)
        {
            external_symbols.insert(symbol.clone());
        }
        closure_symbols.insert(symbol);
    }
}

fn include_non_runtime_closure_symbol(
    semantic: &CheckedCoreSemanticInputs,
    symbol: StableSymbol,
    closure_symbols: &mut BTreeSet<StableSymbol>,
    external_symbols: &mut BTreeSet<StableSymbol>,
) {
    if (!has_semantic_entry(semantic, &symbol) && !semantic.declarations.contains_key(&symbol))
        || semantic.dependency_declaration_refs.contains_key(&symbol)
    {
        external_symbols.insert(symbol.clone());
    }
    closure_symbols.insert(symbol);
}

fn has_semantic_entry(semantic: &CheckedCoreSemanticInputs, symbol: &StableSymbol) -> bool {
    semantic.primitive_refs.contains_key(symbol)
        || semantic.primitive_metadata.contains_key(symbol)
        || semantic.data_metadata.contains_key(symbol)
        || semantic.record_sigma_metadata.contains_key(symbol)
        || semantic.class_instance_metadata.contains_key(symbol)
        || semantic.recursion_metadata.contains_key(symbol)
        || semantic.effects_foreign_metadata.contains_key(symbol)
        || semantic.metadata.contains_key(symbol)
        || semantic.lowerability.contains_key(symbol)
        || semantic.obligation_metadata.contains_key(symbol)
        || semantic.assumption_trust_metadata.contains_key(symbol)
        || semantic.obligations.contains_key(symbol)
        || semantic.assumptions.contains_key(symbol)
        || semantic.trusted_base_delta.contains_key(symbol)
        || semantic.dependency_semantic_hashes.contains_key(symbol)
        || semantic.dependency_declaration_refs.contains_key(symbol)
        || semantic.unsupported.contains_key(symbol)
}

fn metadata_dependencies_for_symbol(
    semantic: &CheckedCoreSemanticInputs,
    symbol: &StableSymbol,
) -> Vec<StableSymbol> {
    let mut dependencies = Vec::new();

    if let Some(meta) = semantic.primitive_metadata.get(symbol) {
        match &meta.partiality {
            crate::checked_core::PartialityMetadata::Total => {}
            crate::checked_core::PartialityMetadata::CheckedPartial { obligation } => {
                dependencies.push(obligation.clone());
            }
            crate::checked_core::PartialityMetadata::TrustedPartial { assumption } => {
                dependencies.push(assumption.clone());
            }
        }
    }
    if let Some(meta) = semantic.data_metadata.get(symbol) {
        dependencies.extend(meta.constructors.iter().map(|ctor| ctor.symbol.clone()));
    }
    if let Some(meta) = semantic.record_sigma_metadata.get(symbol) {
        dependencies.extend(meta.fields.iter().map(|field| field.ty.clone()));
    }
    if let Some(meta) = semantic.class_instance_metadata.get(symbol) {
        dependencies.extend(meta.class_symbol.iter().cloned());
        dependencies.extend(meta.dictionary_symbol.iter().cloned());
        dependencies.extend(meta.head_symbol.iter().cloned());
    }
    if let Some(meta) = semantic.recursion_metadata.get(symbol) {
        dependencies.extend(meta.group_members.iter().cloned());
    }
    if let Some(meta) = semantic.effects_foreign_metadata.get(symbol) {
        dependencies.extend(meta.capabilities.iter().cloned());
        dependencies.extend(meta.runtime_checks.iter().cloned());
    }

    for (obligation, meta) in &semantic.obligation_metadata {
        if &meta.origin == symbol {
            dependencies.push(obligation.clone());
        }
    }
    for (assumption, meta) in &semantic.assumption_trust_metadata {
        if &meta.target == symbol {
            dependencies.push(assumption.clone());
        }
    }

    dependencies
}

fn slice_closure_semantic(
    semantic: &CheckedCoreSemanticInputs,
    closure_symbols: &BTreeSet<StableSymbol>,
    reachable_declarations: &BTreeSet<StableSymbol>,
) -> CheckedCoreSemanticInputs {
    let mut sliced = CheckedCoreSemanticInputs::default();
    sliced.symbols.extend(closure_symbols.iter().cloned());
    sliced.declarations = filter_map_by_keys(&semantic.declarations, reachable_declarations);
    sliced.primitive_refs = filter_map_by_keys(&semantic.primitive_refs, closure_symbols);
    sliced.primitive_metadata = filter_map_by_keys(&semantic.primitive_metadata, closure_symbols);
    sliced.data_metadata = filter_map_by_keys(&semantic.data_metadata, closure_symbols);
    sliced.record_sigma_metadata =
        filter_map_by_keys(&semantic.record_sigma_metadata, closure_symbols);
    sliced.class_instance_metadata =
        filter_map_by_keys(&semantic.class_instance_metadata, closure_symbols);
    sliced.recursion_metadata = filter_map_by_keys(&semantic.recursion_metadata, closure_symbols);
    sliced.effects_foreign_metadata =
        filter_map_by_keys(&semantic.effects_foreign_metadata, closure_symbols);
    sliced.metadata = filter_map_by_keys(&semantic.metadata, closure_symbols);
    sliced.lowerability = filter_map_by_keys(&semantic.lowerability, closure_symbols);
    sliced.obligation_metadata = filter_map_by_keys(&semantic.obligation_metadata, closure_symbols);
    sliced.assumption_trust_metadata =
        filter_map_by_keys(&semantic.assumption_trust_metadata, closure_symbols);
    sliced.obligations = filter_map_by_keys(&semantic.obligations, closure_symbols);
    sliced.assumptions = filter_map_by_keys(&semantic.assumptions, closure_symbols);
    sliced.trusted_base_delta = filter_map_by_keys(&semantic.trusted_base_delta, closure_symbols);
    sliced.dependency_semantic_hashes = semantic.dependency_semantic_hashes.clone();
    sliced.dependency_declaration_refs =
        filter_map_by_keys(&semantic.dependency_declaration_refs, closure_symbols);
    sliced.unsupported = filter_map_by_keys(&semantic.unsupported, closure_symbols);
    sliced
        .symbols
        .extend(sliced.dependency_semantic_hashes.keys().cloned());
    sliced
        .symbols
        .extend(sliced.dependency_declaration_refs.values().cloned());
    sliced
}

fn filter_map_by_keys<T: Clone>(
    map: &BTreeMap<StableSymbol, T>,
    keys: &BTreeSet<StableSymbol>,
) -> BTreeMap<StableSymbol, T> {
    map.iter()
        .filter(|(symbol, _)| keys.contains(*symbol))
        .map(|(symbol, value)| (symbol.clone(), value.clone()))
        .collect()
}

fn validate_closure_metadata(
    semantic: &CheckedCoreSemanticInputs,
) -> Result<(), CompilerDriverError> {
    for symbol in semantic.obligations.keys() {
        if !semantic.obligation_metadata.contains_key(symbol) {
            return Err(CompilerDriverError::MissingClosureMetadata {
                section: "obligation",
                symbol: symbol.clone(),
            });
        }
    }
    for symbol in semantic.assumptions.keys() {
        if !semantic.assumption_trust_metadata.contains_key(symbol) {
            return Err(CompilerDriverError::MissingClosureMetadata {
                section: "assumption",
                symbol: symbol.clone(),
            });
        }
    }
    Ok(())
}

fn build_target_closure_report(
    package: &CheckedCorePackage,
    target: &SelectedTargetReport,
    semantic: &CheckedCoreSemanticInputs,
    reachable_declarations: &BTreeSet<StableSymbol>,
    external_symbols: &BTreeSet<StableSymbol>,
) -> TargetClosureReport {
    let mut unsupported_lanes = BTreeMap::new();

    for lane in target
        .lanes
        .iter()
        .filter(|lane| lane.lane != "runtime_lowering_unavailable")
    {
        unsupported_lanes
            .entry(target.symbol.clone())
            .or_insert_with(Vec::new)
            .push(lane.clone());
    }

    for (symbol, status) in &semantic.lowerability {
        if status.blocks_lowering() {
            unsupported_lanes
                .entry(symbol.clone())
                .or_insert_with(Vec::new)
                .push(UnavailableLane::new(
                    "non_lowerable_closure_member",
                    format!("closure member blocks runtime lowering: {status:?}"),
                ));
        }
    }

    for symbol in semantic.unsupported.keys() {
        unsupported_lanes
            .entry(symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "checked_core_unsupported",
                "reachable checked-core unsupported lane blocks runtime lowering",
            ));
    }

    for symbol in external_symbols {
        if let Some(dependency) = semantic.dependency_declaration_refs.get(symbol) {
            if !semantic.dependency_semantic_hashes.contains_key(dependency) {
                unsupported_lanes
                    .entry(symbol.clone())
                    .or_insert_with(Vec::new)
                    .push(UnavailableLane::new(
                        "missing_dependency_identity",
                        "imported declaration ref names a dependency without semantic hash",
                    ));
            }
        } else {
            unsupported_lanes
                .entry(symbol.clone())
                .or_insert_with(Vec::new)
                .push(UnavailableLane::new(
                    "unresolved_checked_core_symbol",
                    "closure references a stable symbol without an in-package declaration or dependency body",
                ));
        }
    }

    let mut dictionary_runtime_fields = BTreeMap::new();
    let mut dictionary_erased_fields = BTreeMap::new();
    for (symbol, meta) in &semantic.class_instance_metadata {
        if meta.kind == crate::checked_core::ClassInstanceKind::Dictionary {
            dictionary_runtime_fields.insert(symbol.clone(), meta.runtime_fields.clone());
            let erased = meta
                .field_order
                .iter()
                .filter(|field| !meta.runtime_fields.contains(*field))
                .cloned()
                .collect::<BTreeSet<_>>();
            dictionary_erased_fields.insert(symbol.clone(), erased);
        }
    }

    let runtime_lane = unsupported_lanes
        .values()
        .flat_map(|lanes| lanes.iter())
        .next()
        .cloned()
        .unwrap_or_else(|| {
            UnavailableLane::new(
                "runtime_lowering_unavailable",
                "NC11 computes checked-core target closure only; runtime lowering starts in later NC work",
            )
        });

    let mut report = TargetClosureReport {
        package_identity: package.header.package_identity.clone(),
        target_symbol: target.symbol.clone(),
        target_kind: target.kind.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        closure_semantic_hash: semantic_fingerprint(semantic),
        closure_identity: 0,
        reachable_declarations: reachable_declarations.clone(),
        external_symbols: external_symbols.clone(),
        dependency_semantic_hashes: semantic.dependency_semantic_hashes.clone(),
        imported_declaration_refs: semantic.dependency_declaration_refs.clone(),
        dictionary_runtime_fields,
        dictionary_erased_fields,
        obligations: semantic.obligations.keys().cloned().collect(),
        assumptions: semantic.assumptions.keys().cloned().collect(),
        lowerability: semantic.lowerability.clone(),
        unsupported_lanes,
        trusted_base_delta: semantic.trusted_base_delta.keys().cloned().collect(),
        runtime_lowering: ReportFact::Unavailable(runtime_lane),
    };
    report.closure_identity = target_closure_report_fingerprint(&report);
    report
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack.len() >= needle.len()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn executable_entrypoint_fingerprint(entrypoint: &ExecutableEntrypointPackage) -> u64 {
    let mut bytes = Vec::new();
    push_str(&mut bytes, &entrypoint.package_identity.to_string());
    push_str(
        &mut bytes,
        &format!("{:016x}", entrypoint.package_core_semantic_hash),
    );
    push_str(
        &mut bytes,
        &format!("{:016x}", entrypoint.package_artifact_hash),
    );
    push_str(&mut bytes, &entrypoint.target_symbol.to_string());
    push_compiler_target_kind(&mut bytes, &entrypoint.target_kind);
    push_str(&mut bytes, &format!("{:016x}", entrypoint.closure_identity));
    push_str(
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
    push_str(
        &mut bytes,
        &format!(
            "{:016x}",
            entrypoint.report_contract.target_closure_identity
        ),
    );
    push_str(
        &mut bytes,
        &format!(
            "{:016x}",
            entrypoint.report_contract.target_closure_report_hash
        ),
    );
    push_str(&mut bytes, &entrypoint.report_contract.evidence_source);
    for (symbol, lanes) in &entrypoint.unsupported_lanes {
        push_str(&mut bytes, &symbol.to_string());
        for lane in lanes {
            push_lane(&mut bytes, lane);
        }
    }
    fingerprint(&bytes)
}

fn push_entrypoint_verdict(bytes: &mut Vec<u8>, verdict: &ExecutableEntrypointVerdict) {
    match verdict {
        ExecutableEntrypointVerdict::ClosedKenOnly => push_str(bytes, "closed_ken_only"),
        ExecutableEntrypointVerdict::Unavailable { lanes } => {
            push_str(bytes, "unavailable");
            for lane in lanes {
                push_lane(bytes, lane);
            }
        }
    }
}

fn push_dependency_closure(bytes: &mut Vec<u8>, closure: &ExecutableDependencyClosure) {
    match closure {
        ExecutableDependencyClosure::ClosedKenOnly => push_str(bytes, "closed_ken_only"),
        ExecutableDependencyClosure::ImportsUnavailable {
            external_symbols,
            imported_declaration_refs,
        } => {
            push_str(bytes, "imports_unavailable");
            for symbol in external_symbols {
                push_str(bytes, &symbol.to_string());
            }
            for (declaration, dependency) in imported_declaration_refs {
                push_str(bytes, &declaration.to_string());
                push_str(bytes, &dependency.to_string());
            }
        }
    }
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
    push_str(bytes, tag);
}

fn push_argument_packaging(bytes: &mut Vec<u8>, packaging: &ExecutableArgumentPackaging) {
    match &packaging.shape {
        ExecutableArgumentShape::ClosedNullary => push_str(bytes, "closed_nullary"),
        ExecutableArgumentShape::CheckedProgramAbi => push_str(bytes, "checked_program_abi"),
        ExecutableArgumentShape::UnsupportedRuntimeArguments { parameter_count } => {
            push_str(bytes, "unsupported_runtime_arguments");
            push_str(bytes, &parameter_count.to_string());
        }
        ExecutableArgumentShape::Unavailable { lane } => {
            push_str(bytes, "unavailable");
            push_lane(bytes, lane);
        }
    }
    push_str(bytes, &packaging.evidence_source);
}

fn push_result_observation(bytes: &mut Vec<u8>, observation: &ExecutableResultObservation) {
    match &observation.shape {
        ExecutableResultShape::RuntimeValue => push_str(bytes, "runtime_value"),
        ExecutableResultShape::Unavailable { lane } => {
            push_str(bytes, "unavailable");
            push_lane(bytes, lane);
        }
    }
    push_str(bytes, &observation.evidence_source);
}

fn push_trap_contract(bytes: &mut Vec<u8>, contract: &ExecutableTrapContract) {
    match &contract.shape {
        ExecutableTrapShape::RuntimeTrapReport => push_str(bytes, "runtime_trap_report"),
        ExecutableTrapShape::Unavailable { lane } => {
            push_str(bytes, "unavailable");
            push_lane(bytes, lane);
        }
    }
    for (symbol, lanes) in &contract.blocking_lanes {
        push_str(bytes, &symbol.to_string());
        for lane in lanes {
            push_lane(bytes, lane);
        }
    }
}

fn push_compiler_target_kind(bytes: &mut Vec<u8>, kind: &CompilerTargetKind) {
    let tag = match kind {
        CompilerTargetKind::Executable => "executable",
        CompilerTargetKind::Library => "library",
        CompilerTargetKind::NonRuntime => "non_runtime",
    };
    push_str(bytes, tag);
}

fn push_lane(bytes: &mut Vec<u8>, lane: &UnavailableLane) {
    push_str(bytes, &lane.lane);
    push_str(bytes, &lane.reason);
}

fn target_report_fingerprint(report: &TargetSelectionReport) -> u64 {
    let mut bytes = Vec::new();
    push_str(&mut bytes, &report.package_identity.to_string());
    push_str(&mut bytes, &format!("{:016x}", report.core_semantic_hash));
    push_str(&mut bytes, &format!("{:016x}", report.artifact_hash));
    push_report_fact(&mut bytes, &report.checked_core_emission);
    push_report_fact(&mut bytes, &report.runtime_lowering);
    push_report_fact(&mut bytes, &report.native_artifact);
    push_report_fact(&mut bytes, &report.validation_facts);
    for target in &report.selected_targets {
        push_str(&mut bytes, &target.package_identity.to_string());
        push_str(&mut bytes, &target.symbol.to_string());
        push_str(&mut bytes, &format!("{:?}", target.kind));
        for lane in &target.lanes {
            push_str(&mut bytes, &lane.lane);
            push_str(&mut bytes, &lane.reason);
        }
    }
    for (symbol, lanes) in &report.unsupported_lanes {
        push_str(&mut bytes, &symbol.to_string());
        for lane in lanes {
            push_str(&mut bytes, &lane.lane);
            push_str(&mut bytes, &lane.reason);
        }
    }
    for (dependency, hash) in &report.dependency_semantic_hashes {
        push_str(&mut bytes, &dependency.to_string());
        push_str(&mut bytes, hash);
    }
    for obligation in &report.obligations {
        push_str(&mut bytes, &obligation.to_string());
    }
    for assumption in &report.assumptions {
        push_str(&mut bytes, &assumption.to_string());
    }
    for trusted in &report.trusted_base_delta {
        push_str(&mut bytes, &trusted.to_string());
    }
    fingerprint(&bytes)
}

fn target_closure_report_fingerprint(report: &TargetClosureReport) -> u64 {
    let mut bytes = Vec::new();
    push_str(&mut bytes, &report.package_identity.to_string());
    push_str(&mut bytes, &report.target_symbol.to_string());
    push_str(&mut bytes, &format!("{:?}", report.target_kind));
    push_str(
        &mut bytes,
        &format!("{:016x}", report.package_core_semantic_hash),
    );
    push_str(
        &mut bytes,
        &format!("{:016x}", report.package_artifact_hash),
    );
    push_str(
        &mut bytes,
        &format!("{:016x}", report.closure_semantic_hash),
    );
    push_report_fact(&mut bytes, &report.runtime_lowering);
    for symbol in &report.reachable_declarations {
        push_str(&mut bytes, &symbol.to_string());
    }
    for symbol in &report.external_symbols {
        push_str(&mut bytes, &symbol.to_string());
    }
    for (dependency, hash) in &report.dependency_semantic_hashes {
        push_str(&mut bytes, &dependency.to_string());
        push_str(&mut bytes, hash);
    }
    for (declaration, dependency) in &report.imported_declaration_refs {
        push_str(&mut bytes, &declaration.to_string());
        push_str(&mut bytes, &dependency.to_string());
    }
    for (dictionary, fields) in &report.dictionary_runtime_fields {
        push_str(&mut bytes, &dictionary.to_string());
        for field in fields {
            push_str(&mut bytes, field);
        }
    }
    for (dictionary, fields) in &report.dictionary_erased_fields {
        push_str(&mut bytes, &dictionary.to_string());
        for field in fields {
            push_str(&mut bytes, field);
        }
    }
    for obligation in &report.obligations {
        push_str(&mut bytes, &obligation.to_string());
    }
    for assumption in &report.assumptions {
        push_str(&mut bytes, &assumption.to_string());
    }
    for (symbol, status) in &report.lowerability {
        push_str(&mut bytes, &symbol.to_string());
        push_str(&mut bytes, &format!("{status:?}"));
    }
    for (symbol, lanes) in &report.unsupported_lanes {
        push_str(&mut bytes, &symbol.to_string());
        for lane in lanes {
            push_str(&mut bytes, &lane.lane);
            push_str(&mut bytes, &lane.reason);
        }
    }
    for trusted in &report.trusted_base_delta {
        push_str(&mut bytes, &trusted.to_string());
    }
    fingerprint(&bytes)
}

fn push_report_fact(bytes: &mut Vec<u8>, fact: &ReportFact) {
    match fact {
        ReportFact::Emitted => push_str(bytes, "emitted"),
        ReportFact::Unavailable(lane) => {
            push_str(bytes, "unavailable");
            push_str(bytes, &lane.lane);
            push_str(bytes, &lane.reason);
        }
    }
}

fn push_str(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn fingerprint(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checked_core::{
        emit_checked_core_package, CheckedCoreArtifactInputs, ClassInstanceKind,
        ClassInstanceMetadata, ObligationMetadata, ObligationStatus,
    };
    use crate::erasure::erase_checked_core_package_for_target;

    fn main_symbol(package: &str) -> StableSymbol {
        StableSymbol::declaration(package, &[], "main")
    }

    fn package_id(package: &str) -> StableSymbol {
        StableSymbol::new(SymbolNamespace::Module, vec![package.to_string()])
    }

    fn real_source() -> CompilerSource {
        CompilerSource::new("src/main.ken", "const main : Bool = True")
    }

    fn dependent_source(helper_body: &str) -> CompilerSource {
        CompilerSource::new(
            "src/main.ken",
            format!("const helper : Bool = {helper_body}\nconst main : Bool = helper"),
        )
    }

    fn selector(package: &str, symbol: StableSymbol) -> TargetSelector {
        TargetSelector::StableSymbol {
            package_identity: package_id(package),
            symbol,
            kind: CompilerTargetKind::Executable,
        }
    }

    fn manifest(package: &str) -> CompilerManifest {
        CompilerManifest::new(package, Vec::new())
    }

    fn reemit_with_semantic(
        package: &CheckedCorePackage,
        semantic: CheckedCoreSemanticInputs,
    ) -> CheckedCorePackage {
        let mut header = package.header.clone();
        header.dependency_semantic_hashes = semantic.dependency_semantic_hashes.clone();
        emit_checked_core_package(
            header,
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: package.artifact.source_identity.clone(),
                annotations: package.artifact.annotations.clone(),
            },
        )
        .unwrap()
    }

    fn has_lane(
        lanes: &BTreeMap<StableSymbol, Vec<UnavailableLane>>,
        symbol: &StableSymbol,
        expected: &str,
    ) -> bool {
        lanes
            .get(symbol)
            .into_iter()
            .flatten()
            .any(|lane| lane.lane == expected)
    }

    fn runtime_entrypoint(
        entrypoint: &ExecutableEntrypointPackage,
    ) -> ken_runtime::ExecutableEntrypointPackageMetadata {
        let mut converted = ken_runtime::ExecutableEntrypointPackageMetadata {
            package_identity: entrypoint.package_identity.to_string(),
            package_core_semantic_hash: entrypoint.package_core_semantic_hash,
            package_artifact_hash: entrypoint.package_artifact_hash,
            target_symbol: entrypoint.target_symbol.to_string(),
            target_kind: match entrypoint.target_kind {
                CompilerTargetKind::Executable => {
                    ken_runtime::ExecutableEntrypointTargetKind::Executable
                }
                CompilerTargetKind::Library => ken_runtime::ExecutableEntrypointTargetKind::Library,
                CompilerTargetKind::NonRuntime => {
                    ken_runtime::ExecutableEntrypointTargetKind::NonRuntime
                }
            },
            closure_identity: entrypoint.closure_identity,
            closure_semantic_hash: entrypoint.closure_semantic_hash,
            metadata_identity: entrypoint.metadata_identity,
            closed_entry: runtime_entrypoint_verdict(&entrypoint.closed_entry),
            dependency_closure: runtime_dependency_closure(&entrypoint.dependency_closure),
            required_runtime_support: entrypoint
                .required_runtime_support
                .iter()
                .map(runtime_support)
                .collect(),
            argument_packaging: ken_runtime::ExecutableArgumentPackaging {
                shape: runtime_argument_shape(&entrypoint.argument_packaging.shape),
                evidence_source: entrypoint.argument_packaging.evidence_source.clone(),
            },
            result_observation: ken_runtime::ExecutableResultObservation {
                shape: runtime_result_shape(&entrypoint.result_observation.shape),
                evidence_source: entrypoint.result_observation.evidence_source.clone(),
            },
            trap_contract: ken_runtime::ExecutableTrapContract {
                shape: runtime_trap_shape(&entrypoint.trap_contract.shape),
                blocking_lanes: runtime_lane_map(&entrypoint.trap_contract.blocking_lanes),
            },
            report_contract: ken_runtime::ExecutableReportContract {
                target_closure_identity: entrypoint.report_contract.target_closure_identity,
                target_closure_report_hash: entrypoint.report_contract.target_closure_report_hash,
                evidence_source: entrypoint.report_contract.evidence_source.clone(),
            },
            unsupported_lanes: runtime_lane_map(&entrypoint.unsupported_lanes),
        };
        converted.metadata_identity = ken_runtime::executable_entrypoint_metadata_hash(&converted);
        converted
    }

    fn runtime_entrypoint_verdict(
        verdict: &ExecutableEntrypointVerdict,
    ) -> ken_runtime::ExecutableEntrypointVerdict {
        match verdict {
            ExecutableEntrypointVerdict::ClosedKenOnly => {
                ken_runtime::ExecutableEntrypointVerdict::ClosedKenOnly
            }
            ExecutableEntrypointVerdict::Unavailable { lanes } => {
                ken_runtime::ExecutableEntrypointVerdict::Unavailable {
                    lanes: lanes.iter().map(runtime_lane).collect(),
                }
            }
        }
    }

    fn runtime_dependency_closure(
        closure: &ExecutableDependencyClosure,
    ) -> ken_runtime::ExecutableDependencyClosure {
        match closure {
            ExecutableDependencyClosure::ClosedKenOnly => {
                ken_runtime::ExecutableDependencyClosure::ClosedKenOnly
            }
            ExecutableDependencyClosure::ImportsUnavailable {
                external_symbols,
                imported_declaration_refs,
            } => ken_runtime::ExecutableDependencyClosure::ImportsUnavailable {
                external_symbols: external_symbols.iter().map(ToString::to_string).collect(),
                imported_declaration_refs: imported_declaration_refs
                    .iter()
                    .map(|(declaration, dependency)| {
                        (declaration.to_string(), dependency.to_string())
                    })
                    .collect(),
            },
        }
    }

    fn runtime_support(
        support: &ExecutableRuntimeSupport,
    ) -> ken_runtime::ExecutableRuntimeSupport {
        match support {
            ExecutableRuntimeSupport::RuntimeValues => {
                ken_runtime::ExecutableRuntimeSupport::RuntimeValues
            }
            ExecutableRuntimeSupport::FunctionCalls => {
                ken_runtime::ExecutableRuntimeSupport::FunctionCalls
            }
            ExecutableRuntimeSupport::PrimitiveValues => {
                ken_runtime::ExecutableRuntimeSupport::PrimitiveValues
            }
            ExecutableRuntimeSupport::PrimitiveOperations => {
                ken_runtime::ExecutableRuntimeSupport::PrimitiveOperations
            }
            ExecutableRuntimeSupport::AlgebraicData => {
                ken_runtime::ExecutableRuntimeSupport::AlgebraicData
            }
            ExecutableRuntimeSupport::PatternMatching => {
                ken_runtime::ExecutableRuntimeSupport::PatternMatching
            }
            ExecutableRuntimeSupport::RecordsSigma => {
                ken_runtime::ExecutableRuntimeSupport::RecordsSigma
            }
            ExecutableRuntimeSupport::Dictionaries => {
                ken_runtime::ExecutableRuntimeSupport::Dictionaries
            }
            ExecutableRuntimeSupport::Recursion => ken_runtime::ExecutableRuntimeSupport::Recursion,
            ExecutableRuntimeSupport::TrapReporting => {
                ken_runtime::ExecutableRuntimeSupport::TrapReporting
            }
        }
    }

    fn runtime_argument_shape(
        shape: &ExecutableArgumentShape,
    ) -> ken_runtime::ExecutableArgumentShape {
        match shape {
            ExecutableArgumentShape::ClosedNullary => {
                ken_runtime::ExecutableArgumentShape::ClosedNullary
            }
            ExecutableArgumentShape::CheckedProgramAbi => {
                ken_runtime::ExecutableArgumentShape::ProcessInput {
                    arguments: Vec::new(),
                    environment: Vec::new(),
                    working_directory: Vec::new(),
                }
            }
            ExecutableArgumentShape::UnsupportedRuntimeArguments { parameter_count } => {
                ken_runtime::ExecutableArgumentShape::UnsupportedRuntimeArguments {
                    parameter_count: *parameter_count,
                }
            }
            ExecutableArgumentShape::Unavailable { lane } => {
                ken_runtime::ExecutableArgumentShape::Unavailable {
                    lane: runtime_lane(lane),
                }
            }
        }
    }

    fn runtime_result_shape(shape: &ExecutableResultShape) -> ken_runtime::ExecutableResultShape {
        match shape {
            ExecutableResultShape::RuntimeValue => ken_runtime::ExecutableResultShape::RuntimeValue,
            ExecutableResultShape::Unavailable { lane } => {
                ken_runtime::ExecutableResultShape::Unavailable {
                    lane: runtime_lane(lane),
                }
            }
        }
    }

    fn runtime_trap_shape(shape: &ExecutableTrapShape) -> ken_runtime::ExecutableTrapShape {
        match shape {
            ExecutableTrapShape::RuntimeTrapReport => {
                ken_runtime::ExecutableTrapShape::RuntimeTrapReport
            }
            ExecutableTrapShape::Unavailable { lane } => {
                ken_runtime::ExecutableTrapShape::Unavailable {
                    lane: runtime_lane(lane),
                }
            }
        }
    }

    fn runtime_lane_map(
        lanes: &BTreeMap<StableSymbol, Vec<UnavailableLane>>,
    ) -> BTreeMap<String, Vec<ken_runtime::ExecutableEntrypointUnavailableLane>> {
        lanes
            .iter()
            .map(|(symbol, lanes)| {
                (
                    symbol.to_string(),
                    lanes.iter().map(runtime_lane).collect::<Vec<_>>(),
                )
            })
            .collect()
    }

    fn runtime_lane(lane: &UnavailableLane) -> ken_runtime::ExecutableEntrypointUnavailableLane {
        ken_runtime::ExecutableEntrypointUnavailableLane {
            lane: lane.lane.clone(),
            reason: lane.reason.clone(),
        }
    }

    #[test]
    fn real_source_reaches_checked_core_and_selects_stable_target() {
        let target = main_symbol("nc10_demo");
        let out = compile_ken_source(
            "nc10_demo",
            real_source(),
            TargetSelector::StableSymbol {
                package_identity: package_id("nc10_demo"),
                symbol: target.clone(),
                kind: CompilerTargetKind::Executable,
            },
        )
        .expect("real source compiles through checked-core");

        validate_checked_core_package(&out.package).unwrap();
        assert_eq!(out.report.package_identity, package_id("nc10_demo"));
        assert_eq!(out.report.selected_targets.len(), 1);
        assert_eq!(out.report.selected_targets[0].symbol, target);
        assert_eq!(out.closures.len(), 1);
        assert_eq!(out.closures[0].target.symbol, target);
        assert_eq!(out.executable_entrypoints.len(), 1);
        let entrypoint = &out.executable_entrypoints[0];
        assert_eq!(entrypoint.package_identity, package_id("nc10_demo"));
        assert_eq!(entrypoint.target_symbol, target);
        assert_eq!(
            entrypoint.closure_identity,
            out.closures[0].closure_identity
        );
        assert!(matches!(
            entrypoint.closed_entry,
            ExecutableEntrypointVerdict::ClosedKenOnly
        ));
        assert_eq!(
            entrypoint.dependency_closure,
            ExecutableDependencyClosure::ClosedKenOnly
        );
        assert!(matches!(
            entrypoint.argument_packaging.shape,
            ExecutableArgumentShape::ClosedNullary
        ));
        assert!(entrypoint
            .required_runtime_support
            .contains(&ExecutableRuntimeSupport::RuntimeValues));
        assert!(matches!(
            entrypoint.result_observation.shape,
            ExecutableResultShape::RuntimeValue
        ));
        assert!(matches!(
            entrypoint.trap_contract.shape,
            ExecutableTrapShape::RuntimeTrapReport
        ));
        assert_eq!(
            entrypoint.report_contract.target_closure_identity,
            out.closures[0].closure_identity
        );
        assert_eq!(out.report.checked_core_emission, ReportFact::Emitted);
        assert!(matches!(
            out.report.runtime_lowering,
            ReportFact::Unavailable(UnavailableLane { ref lane, .. })
                if lane == "runtime_lowering_unavailable"
        ));
        assert!(matches!(
            out.report.native_artifact,
            ReportFact::Unavailable(UnavailableLane { ref lane, .. })
                if lane == "native_artifact_unavailable"
        ));
        assert!(matches!(
            out.report.validation_facts,
            ReportFact::Unavailable(UnavailableLane { ref lane, .. })
                if lane == "validation_facts_unavailable"
        ));
    }

    #[test]
    fn compiler_produced_entrypoint_materializes_runtime_packaging() {
        let target = main_symbol("nc20_demo");
        let out = compile_ken_source("nc20_demo", real_source(), selector("nc20_demo", target))
            .expect("real source compiles through checked-core");
        let closure = out.closures.first().expect("selected target closure");
        let program = erase_checked_core_package_for_target(
            &out.package,
            closure.reachable_declarations.iter(),
        )
        .expect("compiler-produced closure lowers to runtime IR");
        let report = ken_runtime::summarize_runtime_ir_program(&program);
        let contract = ken_runtime::executable_artifact_contract_for_runtime_report(
            &program,
            &report,
            out.executable_entrypoints[0].target_symbol.to_string(),
            "ken-elaborator compiler-driver test",
        )
        .expect("runtime contract materializes");
        let entrypoint = runtime_entrypoint(&out.executable_entrypoints[0]);

        let package = ken_runtime::executable_entrypoint_package_for_runtime_contract(
            &program,
            &report,
            &contract,
            entrypoint,
            "ken-elaborator compiler-driver test",
        )
        .expect("runtime entrypoint package materializes");

        assert_eq!(
            package.header.target,
            out.executable_entrypoints[0].target_symbol.to_string()
        );
        assert_eq!(
            package.runtime_artifact,
            ken_runtime::RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(
            package.runtime_report_hash,
            ken_runtime::runtime_ir_program_report_hash(&report)
        );
        assert_eq!(
            package.executable_contract_hash,
            ken_runtime::executable_artifact_contract_hash(&contract)
        );
        assert!(package.header.package_hash != 0);
    }

    #[test]
    fn executable_entrypoint_metadata_identity_is_deterministic_and_content_addressed() {
        let target_selector = selector("entry_stable", main_symbol("entry_stable"));
        let a = compile_ken_source("entry_stable", real_source(), target_selector.clone()).unwrap();
        let b = compile_ken_source("entry_stable", real_source(), target_selector).unwrap();
        assert_eq!(
            a.executable_entrypoints[0].metadata_identity,
            b.executable_entrypoints[0].metadata_identity
        );

        let changed = compile_ken_package_sources(
            &manifest("entry_stable"),
            vec![dependent_source("False")],
            selector("entry_stable", main_symbol("entry_stable")),
        )
        .unwrap();
        assert_ne!(
            a.executable_entrypoints[0].metadata_identity,
            changed.executable_entrypoints[0].metadata_identity,
            "entrypoint identity must move when checked-core closure content moves"
        );
    }

    #[test]
    fn executable_entrypoint_rejects_stale_package_identity() {
        let target = main_symbol("entry_stale");
        let out = compile_ken_source(
            "entry_stale",
            real_source(),
            selector("entry_stale", target),
        )
        .unwrap();
        let mut stale_closure = out.closures[0].clone();
        stale_closure.package_identity = package_id("different_package");

        let err = package_executable_entrypoint(&out.package, &stale_closure).unwrap_err();
        assert!(matches!(
            err,
            CompilerDriverError::EntrypointClosurePackageMismatch {
                field: "package_identity",
                ..
            }
        ));
    }

    #[test]
    fn executable_entrypoint_reports_imported_dependency_as_non_closed() {
        let package_name = "entry_import";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let imported = StableSymbol::declaration("dep_pkg", &["Dep"], "value");
        let dependency = StableSymbol::new(
            SymbolNamespace::Dependency,
            vec!["dep_pkg".to_string(), "checked-core".to_string()],
        );
        let mut table = StableSymbolTable::new();
        table.insert_global(GlobalId(1), target.clone());
        table.insert_global(GlobalId(90), imported.clone());
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::Const {
                id: GlobalId(90),
                level_args: Vec::new(),
            },
            body: Term::Const {
                id: GlobalId(90),
                level_args: Vec::new(),
            },
        };
        let mut semantic = out.package.artifact.semantic.clone();
        semantic.symbols.insert(imported.clone());
        semantic.symbols.insert(dependency.clone());
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic
            .lowerability
            .insert(imported.clone(), LowerabilityStatus::Supported);
        semantic
            .dependency_semantic_hashes
            .insert(dependency.clone(), "sha256:dep".to_string());
        semantic
            .dependency_declaration_refs
            .insert(imported.clone(), dependency);
        let package = reemit_with_semantic(&out.package, semantic);
        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target.clone()),
        )
        .unwrap();
        let entrypoint = package_executable_entrypoint(&package, &closures[0]).unwrap();

        assert!(has_lane(
            &entrypoint.unsupported_lanes,
            &target,
            "non_closed_entrypoint"
        ));
        assert!(has_lane(
            &entrypoint.unsupported_lanes,
            &target,
            "imported_dependency_entrypoint"
        ));
        assert!(matches!(
            entrypoint.dependency_closure,
            ExecutableDependencyClosure::ImportsUnavailable { .. }
        ));
        assert!(matches!(
            entrypoint.closed_entry,
            ExecutableEntrypointVerdict::Unavailable { .. }
        ));
    }

    #[test]
    fn executable_entrypoint_reports_unresolved_dependency_lane() {
        let package_name = "entry_unresolved";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let bool_symbol = StableSymbol::declaration(package_name, &[], "Bool");
        let true_symbol = StableSymbol::constructor(&bool_symbol, "True");
        let mut semantic = out.package.artifact.semantic.clone();
        semantic.data_metadata.remove(&bool_symbol);
        semantic.lowerability.remove(&bool_symbol);
        semantic.lowerability.remove(&true_symbol);
        let package = reemit_with_semantic(&out.package, semantic);
        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target),
        )
        .unwrap();
        let entrypoint = package_executable_entrypoint(&package, &closures[0]).unwrap();

        assert!(entrypoint
            .unsupported_lanes
            .values()
            .flatten()
            .any(|lane| lane.lane == "unresolved_checked_core_symbol"));
        assert!(matches!(
            entrypoint.closed_entry,
            ExecutableEntrypointVerdict::Unavailable { .. }
        ));
    }

    #[test]
    fn executable_entrypoint_reports_unsupported_lowerability_before_native_work() {
        let target = main_symbol("entry_unsupported");
        let mut manifest_target = ManifestTarget::executable(target.clone());
        manifest_target.lowerability = Some(LowerabilityStatus::RequiresFeature {
            feature: "fixture-runtime-feature".to_string(),
            reason: "fixture blocks executable entrypoint packaging".to_string(),
        });
        let manifest = CompilerManifest::new("entry_unsupported", vec![manifest_target]);
        let out =
            compile_ken_package_sources(&manifest, vec![real_source()], TargetSelector::Manifest)
                .unwrap();
        let entrypoint = &out.executable_entrypoints[0];

        assert!(has_lane(
            &entrypoint.unsupported_lanes,
            &target,
            "unsupported_target_lowerability"
        ));
        assert!(matches!(
            entrypoint.closed_entry,
            ExecutableEntrypointVerdict::Unavailable { .. }
        ));
    }

    #[test]
    fn executable_entrypoint_packages_but_rejects_runtime_arguments() {
        let package_name = "entry_args";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let bool_symbol = StableSymbol::declaration(package_name, &[], "Bool");
        let mut table = StableSymbolTable::new();
        table.insert_global(GlobalId(1), target.clone());
        table.insert_global(GlobalId(2), bool_symbol);
        let bool_ty = Term::Const {
            id: GlobalId(2),
            level_args: Vec::new(),
        };
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::pi(bool_ty.clone(), bool_ty.clone()),
            body: Term::lam(bool_ty, Term::var(0)),
        };
        let mut semantic = out.package.artifact.semantic.clone();
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        let package = reemit_with_semantic(&out.package, semantic);
        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target.clone()),
        )
        .unwrap();
        let entrypoint = package_executable_entrypoint(&package, &closures[0]).unwrap();

        assert!(has_lane(
            &entrypoint.unsupported_lanes,
            &target,
            "entrypoint_runtime_arguments_unavailable"
        ));
        assert!(matches!(
            entrypoint.argument_packaging.shape,
            ExecutableArgumentShape::UnsupportedRuntimeArguments { parameter_count: 1 }
        ));
        assert!(entrypoint
            .required_runtime_support
            .contains(&ExecutableRuntimeSupport::FunctionCalls));
    }

    #[test]
    fn target_closure_reaches_declaration_dependencies_and_changes_with_content() {
        let package = "closure_pkg";
        let target = main_symbol(package);
        let helper = StableSymbol::declaration(package, &[], "helper");
        let selector = selector(package, target.clone());
        let true_out = compile_ken_package_sources(
            &manifest(package),
            vec![dependent_source("True")],
            selector.clone(),
        )
        .unwrap();
        let false_out = compile_ken_package_sources(
            &manifest(package),
            vec![dependent_source("False")],
            selector,
        )
        .unwrap();

        let closure = &true_out.closures[0];
        assert!(closure.reachable_declarations.contains(&target));
        assert!(
            closure.reachable_declarations.contains(&helper),
            "helper must be discovered from checked-core declaration bytes, not a hand table"
        );
        assert_ne!(
            true_out.closures[0].closure_identity, false_out.closures[0].closure_identity,
            "reachable checked-core body changes must change target closure identity"
        );
    }

    #[test]
    fn target_closure_preserves_obligations_trust_and_dependencies() {
        let package_name = "closure_meta";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let mut semantic = out.package.artifact.semantic.clone();
        let obligation = StableSymbol::obligation("main.ensures.0");
        let assumption = StableSymbol::assumption(&target, "trusted-fixture");
        let dependency = StableSymbol::new(
            SymbolNamespace::Dependency,
            vec!["dep-pkg".to_string(), "checked-core".to_string()],
        );
        semantic.symbols.insert(obligation.clone());
        semantic.symbols.insert(assumption.clone());
        semantic.symbols.insert(dependency.clone());
        semantic
            .obligations
            .insert(obligation.clone(), b"goal-core".to_vec());
        semantic
            .assumptions
            .insert(assumption.clone(), b"trusted fixture assumption".to_vec());
        semantic.obligation_metadata.insert(
            obligation.clone(),
            ObligationMetadata {
                status: ObligationStatus::Unknown,
                origin: target.clone(),
                affects_runtime_meaning: true,
            },
        );
        semantic.assumption_trust_metadata.insert(
            assumption.clone(),
            AssumptionTrustMetadata {
                kind: AssumptionTrustKind::Hole,
                target: target.clone(),
                affects_runtime_meaning: true,
            },
        );
        semantic
            .trusted_base_delta
            .insert(target.clone(), b"trusted fixture target".to_vec());
        semantic
            .dependency_semantic_hashes
            .insert(dependency.clone(), "sha256:dependency".to_string());
        let package = reemit_with_semantic(&out.package, semantic);

        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target.clone()),
        )
        .unwrap();
        let closure = &closures[0];
        assert!(closure.semantic.obligations.contains_key(&obligation));
        assert!(closure
            .semantic
            .obligation_metadata
            .contains_key(&obligation));
        assert!(closure.semantic.assumptions.contains_key(&assumption));
        assert!(closure
            .semantic
            .assumption_trust_metadata
            .contains_key(&assumption));
        assert!(closure.semantic.trusted_base_delta.contains_key(&target));
        assert!(closure.report.assumptions.contains(&assumption));
        assert!(closure.report.trusted_base_delta.contains(&target));
        assert_eq!(
            closure.report.dependency_semantic_hashes.get(&dependency),
            Some(&"sha256:dependency".to_string())
        );
    }

    #[test]
    fn target_closure_reports_imported_refs_and_dictionary_runtime_fields() {
        let package_name = "closure_nc17";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let imported = StableSymbol::declaration("dep_pkg", &["Dep"], "value");
        let dependency = StableSymbol::new(
            SymbolNamespace::Dependency,
            vec!["dep_pkg".to_string(), "checked-core".to_string()],
        );
        let dictionary = StableSymbol::declaration(package_name, &[], "EqBoolDict");
        let class = StableSymbol::declaration(package_name, &[], "Eq");
        let head = StableSymbol::declaration(package_name, &[], "Bool");
        let table = {
            let mut table = StableSymbolTable::new();
            table.insert_global(GlobalId(1), target.clone());
            table.insert_global(GlobalId(10), dictionary.clone());
            table.insert_global(GlobalId(90), imported.clone());
            table
        };
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::Const {
                id: GlobalId(10),
                level_args: Vec::new(),
            },
            body: Term::Const {
                id: GlobalId(90),
                level_args: Vec::new(),
            },
        };
        let mut semantic = out.package.artifact.semantic.clone();
        for symbol in [&imported, &dependency, &dictionary, &class, &head] {
            semantic.symbols.insert(symbol.clone());
        }
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic
            .lowerability
            .insert(imported.clone(), LowerabilityStatus::Supported);
        semantic
            .lowerability
            .insert(dictionary.clone(), LowerabilityStatus::Supported);
        semantic
            .dependency_semantic_hashes
            .insert(dependency.clone(), "sha256:dep".to_string());
        semantic
            .dependency_declaration_refs
            .insert(imported.clone(), dependency.clone());
        semantic.class_instance_metadata.insert(
            dictionary.clone(),
            ClassInstanceMetadata {
                kind: ClassInstanceKind::Dictionary,
                class_symbol: Some(class),
                dictionary_symbol: Some(dictionary.clone()),
                head_symbol: Some(head),
                field_order: vec!["eq".to_string(), "law".to_string()],
                runtime_fields: BTreeSet::from(["eq".to_string()]),
                law_fields: BTreeSet::from(["law".to_string()]),
                lowerability: LowerabilityStatus::Supported,
            },
        );
        let package = reemit_with_semantic(&out.package, semantic);

        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target),
        )
        .unwrap();
        let report = &closures[0].report;

        assert_eq!(
            report.imported_declaration_refs.get(&imported),
            Some(&dependency)
        );
        assert!(report.external_symbols.contains(&imported));
        assert!(!report
            .unsupported_lanes
            .values()
            .flatten()
            .any(|lane| lane.lane == "unresolved_checked_core_symbol"));
        assert_eq!(
            report.dictionary_runtime_fields.get(&dictionary),
            Some(&BTreeSet::from(["eq".to_string()]))
        );
        assert_eq!(
            report.dictionary_erased_fields.get(&dictionary),
            Some(&BTreeSet::from(["law".to_string()]))
        );
    }

    #[test]
    fn target_closure_reports_unresolved_references_without_runtime_success() {
        let package_name = "closure_unresolved";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let closure = &out.closures[0];
        let bool_symbol = StableSymbol::declaration(package_name, &[], "Bool");
        let true_symbol = StableSymbol::constructor(&bool_symbol, "True");

        assert!(
            !closure.external_symbols.contains(&bool_symbol),
            "package data metadata resolves constructor families without external fallback"
        );
        assert!(closure.semantic.data_metadata.contains_key(&bool_symbol));

        let mut semantic = out.package.artifact.semantic.clone();
        semantic.data_metadata.remove(&bool_symbol);
        semantic.lowerability.remove(&bool_symbol);
        semantic.lowerability.remove(&true_symbol);
        let package = reemit_with_semantic(&out.package, semantic);
        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target),
        )
        .unwrap();
        let closure = &closures[0];

        assert!(
            closure.external_symbols.contains(&bool_symbol)
                || closure.external_symbols.contains(&true_symbol),
            "declaration references without package metadata must remain explicit externals"
        );
        assert!(closure
            .report
            .unsupported_lanes
            .values()
            .flatten()
            .any(|lane| lane.lane == "unresolved_checked_core_symbol"));
        assert!(matches!(
            closure.report.runtime_lowering,
            ReportFact::Unavailable(UnavailableLane { ref lane, .. })
                if lane == "unresolved_checked_core_symbol"
        ));
    }

    #[test]
    fn target_closure_rejects_dropped_reachable_metadata() {
        let package_name = "closure_gap";
        let target = main_symbol(package_name);
        let out = compile_ken_source(
            package_name,
            real_source(),
            selector(package_name, target.clone()),
        )
        .unwrap();
        let mut semantic = out.package.artifact.semantic.clone();
        let obligation = StableSymbol::obligation("main.ensures.0");
        semantic.symbols.insert(obligation.clone());
        semantic
            .obligations
            .insert(obligation.clone(), b"goal-core".to_vec());
        let package = reemit_with_semantic(&out.package, semantic);

        let err = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CompilerDriverError::MissingClosureMetadata { section: "obligation", symbol }
                if symbol == obligation
        ));
    }

    #[test]
    fn target_closure_reports_non_lowerable_reachable_members() {
        let package_name = "closure_unsupported";
        let target = main_symbol(package_name);
        let helper = StableSymbol::declaration(package_name, &[], "helper");
        let out = compile_ken_package_sources(
            &manifest(package_name),
            vec![dependent_source("True")],
            selector(package_name, target.clone()),
        )
        .unwrap();
        let mut semantic = out.package.artifact.semantic.clone();
        let status = LowerabilityStatus::Unsupported {
            reason: "fixture helper blocks lowering".to_string(),
        };
        semantic.lowerability.insert(helper.clone(), status);
        semantic
            .unsupported
            .insert(helper.clone(), b"helper blocked".to_vec());
        let package = reemit_with_semantic(&out.package, semantic);

        let closures = compute_target_closures(
            &manifest(package_name),
            &package,
            selector(package_name, target),
        )
        .unwrap();
        assert!(closures[0]
            .report
            .unsupported_lanes
            .get(&helper)
            .unwrap()
            .iter()
            .any(|lane| lane.lane == "non_lowerable_closure_member"));
        assert!(matches!(
            closures[0].report.runtime_lowering,
            ReportFact::Unavailable(_)
        ));
    }

    #[test]
    fn target_selection_report_identity_is_deterministic() {
        let selector = TargetSelector::StableSymbol {
            package_identity: package_id("stable_pkg"),
            symbol: main_symbol("stable_pkg"),
            kind: CompilerTargetKind::Executable,
        };
        let a = compile_ken_source("stable_pkg", real_source(), selector.clone()).unwrap();
        let b = compile_ken_source("stable_pkg", real_source(), selector).unwrap();

        assert_eq!(a.package.core_semantic_hash, b.package.core_semantic_hash);
        assert_eq!(a.package.artifact_hash, b.package.artifact_hash);
        assert_eq!(a.report.report_identity, b.report.report_identity);
    }

    #[test]
    fn manifest_declared_target_selects_library_target() {
        let target = main_symbol("manifest_pkg");
        let manifest = CompilerManifest::new(
            "manifest_pkg",
            vec![ManifestTarget::library(target.clone())],
        );

        let out =
            compile_ken_package_sources(&manifest, vec![real_source()], TargetSelector::Manifest)
                .expect("manifest target selected");

        assert_eq!(out.report.selected_targets[0].symbol, target);
        assert_eq!(
            out.report.selected_targets[0].kind,
            CompilerTargetKind::Library
        );
        assert!(has_lane(
            &out.executable_entrypoints[0].unsupported_lanes,
            &target,
            "non_executable_entrypoint"
        ));
    }

    #[test]
    fn missing_and_ambiguous_targets_reject() {
        let missing = StableSymbol::declaration("missing_pkg", &[], "absent");
        let err = compile_ken_source(
            "missing_pkg",
            real_source(),
            TargetSelector::StableSymbol {
                package_identity: package_id("missing_pkg"),
                symbol: missing.clone(),
                kind: CompilerTargetKind::Executable,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CompilerDriverError::MissingTarget { symbol } if symbol == missing
        ));

        let manifest = CompilerManifest::new(
            "ambiguous_pkg",
            vec![
                ManifestTarget::executable(main_symbol("ambiguous_pkg")),
                ManifestTarget::library(StableSymbol::declaration("ambiguous_pkg", &[], "other")),
            ],
        );
        let err = compile_ken_package_sources(
            &manifest,
            vec![CompilerSource::new(
                "src/lib.ken",
                "const main : Bool = True\nconst other : Bool = False",
            )],
            TargetSelector::Manifest,
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CompilerDriverError::AmbiguousManifestTarget { count: 2, .. }
        ));
    }

    #[test]
    fn manifest_metadata_cannot_select_absent_target() {
        let absent = StableSymbol::declaration("manifest_gap", &[], "absent");
        let manifest = CompilerManifest::new(
            "manifest_gap",
            vec![ManifestTarget::executable(absent.clone())],
        );

        let err =
            compile_ken_package_sources(&manifest, vec![real_source()], TargetSelector::Manifest)
                .unwrap_err();

        assert!(matches!(
            err,
            CompilerDriverError::MissingTarget { symbol } if symbol == absent
        ));
    }

    #[test]
    fn non_runtime_target_is_reported_as_named_unavailable_lane() {
        let target = main_symbol("proof_pkg");
        let manifest = CompilerManifest::new(
            "proof_pkg",
            vec![ManifestTarget::non_runtime(target.clone())],
        );
        let out =
            compile_ken_package_sources(&manifest, vec![real_source()], TargetSelector::Manifest)
                .expect("non-runtime target is selected but marked unavailable");

        let lanes = &out.report.selected_targets[0].lanes;
        assert!(lanes.iter().any(|lane| lane.lane == "non_runtime_target"));
        assert!(out
            .report
            .unsupported_lanes
            .get(&target)
            .unwrap()
            .iter()
            .any(|lane| lane.lane == "non_runtime_target"));
        assert!(has_lane(
            &out.executable_entrypoints[0].unsupported_lanes,
            &target,
            "non_executable_entrypoint"
        ));
    }

    #[test]
    fn unsupported_target_metadata_is_reported_as_named_lane() {
        let target = main_symbol("unsupported_pkg");
        let mut manifest_target = ManifestTarget::executable(target.clone());
        manifest_target.lowerability = Some(LowerabilityStatus::Unsupported {
            reason: "fixture says no runtime lowering".to_string(),
        });
        let manifest = CompilerManifest::new("unsupported_pkg", vec![manifest_target]);

        let out =
            compile_ken_package_sources(&manifest, vec![real_source()], TargetSelector::Manifest)
                .expect("unsupported target metadata remains reportable");

        assert!(out.report.selected_targets[0]
            .lanes
            .iter()
            .any(|lane| lane.lane == "unsupported_target_metadata"));
        assert!(out
            .report
            .unsupported_lanes
            .get(&target)
            .unwrap()
            .iter()
            .any(|lane| lane.lane == "unsupported_target_metadata"));
    }

    #[test]
    fn stale_or_foreign_package_identity_rejects_target_selection() {
        let stale = compile_ken_source(
            "fresh_pkg",
            real_source(),
            TargetSelector::StableSymbol {
                package_identity: package_id("stale_pkg"),
                symbol: main_symbol("fresh_pkg"),
                kind: CompilerTargetKind::Executable,
            },
        )
        .unwrap_err();
        assert!(matches!(
            stale,
            CompilerDriverError::MismatchedPackageIdentity { .. }
        ));

        let foreign = compile_ken_source(
            "fresh_pkg",
            real_source(),
            TargetSelector::StableSymbol {
                package_identity: package_id("fresh_pkg"),
                symbol: main_symbol("other_pkg"),
                kind: CompilerTargetKind::Executable,
            },
        )
        .unwrap_err();
        assert!(matches!(
            foreign,
            CompilerDriverError::TargetFromDifferentPackage { .. }
        ));
    }
}
