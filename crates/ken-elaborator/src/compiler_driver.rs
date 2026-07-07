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
    canonical_decl_bytes, canonical_symbol_bytes, emit_checked_core_package, semantic_fingerprint,
    validate_checked_core_package, AssumptionTrustKind, AssumptionTrustMetadata,
    CheckedCoreArtifactInputs, CheckedCorePackage, CheckedCorePackageError,
    CheckedCorePackageHeader, CheckedCoreSemanticInputs, ConstructorMetadata, DataMetadata,
    LowerabilityStatus, StableSymbol, StableSymbolTable, SymbolNamespace,
};
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
}

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
    pub obligations: BTreeSet<StableSymbol>,
    pub assumptions: BTreeSet<StableSymbol>,
    pub lowerability: BTreeMap<StableSymbol, LowerabilityStatus>,
    pub unsupported_lanes: BTreeMap<StableSymbol, Vec<UnavailableLane>>,
    pub trusted_base_delta: BTreeSet<StableSymbol>,
    pub runtime_lowering: ReportFact,
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

    let package = emit_package_from_env(manifest, &sources, &env, &admitted)?;
    let selected = select_targets(manifest, &package, selector)?;
    let closures = build_target_closures(&package, &selected)?;
    let report = build_target_selection_report(&package, selected);
    Ok(CompilerDriverOutput {
        package,
        report,
        closures,
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

fn emit_package_from_env(
    manifest: &CompilerManifest,
    sources: &[CompilerSource],
    env: &ElabEnv,
    admitted: &[GlobalId],
) -> Result<CheckedCorePackage, CompilerDriverError> {
    let package_identity = package_identity(&manifest.package_name);
    let mut semantic = CheckedCoreSemanticInputs::default();
    let (symbols, table) = stable_symbols_for_env(&manifest.package_name, env);

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
    apply_manifest_target_metadata(manifest, &mut semantic);
    add_trusted_base_metadata(env, &symbols, &mut semantic);

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

fn stable_symbols_for_env(
    package_name: &str,
    env: &ElabEnv,
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
        let mut dependencies = BTreeSet::new();
        for (candidate, needle) in &encoded_symbols {
            if candidate != owner && contains_subslice(bytes, needle) {
                dependencies.insert(candidate.clone());
            }
        }
        index.insert(owner.clone(), dependencies);
    }
    index
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
        if !has_semantic_entry(semantic, &symbol) {
            external_symbols.insert(symbol.clone());
        }
        closure_symbols.insert(symbol);
    }
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
    sliced.unsupported = filter_map_by_keys(&semantic.unsupported, closure_symbols);
    sliced
        .symbols
        .extend(sliced.dependency_semantic_hashes.keys().cloned());
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
        unsupported_lanes
            .entry(symbol.clone())
            .or_insert_with(Vec::new)
            .push(UnavailableLane::new(
                "unresolved_checked_core_symbol",
                "closure references a stable symbol without an in-package declaration or dependency body",
            ));
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
        emit_checked_core_package, CheckedCoreArtifactInputs, ObligationMetadata, ObligationStatus,
    };

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
