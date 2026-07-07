//! Checked-core stable identity and canonical semantic encoding.
//!
//! NC2 lives after elaboration and kernel admission, but before erasure/runtime
//! IR. This module deliberately stays on the elaborator/package-emitter side:
//! it maps producer-local kernel ids to stable package symbols and encodes the
//! checked-core semantic inputs without making module paths kernel features.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ken_kernel::env::{Decl, PrimReduction};
use ken_kernel::{GlobalId, Level, Term};

pub const CHECKED_CORE_PACKAGE_KIND: &str = "CheckedCorePackage";
pub const CHECKED_CORE_SCHEMA_VERSION: u32 = 0;

/// The semantic role of a stable checked-core identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolNamespace {
    Declaration,
    Constructor,
    Primitive,
    Module,
    Metadata,
    Obligation,
    Assumption,
    Dependency,
    Unsupported,
}

impl SymbolNamespace {
    fn as_str(self) -> &'static str {
        match self {
            SymbolNamespace::Declaration => "decl",
            SymbolNamespace::Constructor => "ctor",
            SymbolNamespace::Primitive => "prim",
            SymbolNamespace::Module => "module",
            SymbolNamespace::Metadata => "meta",
            SymbolNamespace::Obligation => "obl",
            SymbolNamespace::Assumption => "assume",
            SymbolNamespace::Dependency => "dep",
            SymbolNamespace::Unsupported => "unsupported",
        }
    }
}

/// Artifact-level identity for checked-core references.
///
/// A local [`GlobalId`] may point at a stable symbol while emitting a package,
/// but the stable symbol is the only identity serialized into canonical
/// checked-core references. Components are already canonical package/module
/// path atoms; the kernel never sees them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StableSymbol {
    pub namespace: SymbolNamespace,
    pub components: Vec<String>,
}

impl StableSymbol {
    pub fn new(namespace: SymbolNamespace, components: impl IntoIterator<Item = String>) -> Self {
        Self {
            namespace,
            components: components.into_iter().collect(),
        }
    }

    pub fn declaration(
        package: impl Into<String>,
        module: &[&str],
        name: impl Into<String>,
    ) -> Self {
        let mut components = vec![package.into()];
        components.extend(module.iter().map(|part| (*part).to_string()));
        components.push(name.into());
        Self::new(SymbolNamespace::Declaration, components)
    }

    pub fn constructor(parent: &StableSymbol, name: impl Into<String>) -> Self {
        let mut components = parent.components.clone();
        components.push(name.into());
        Self::new(SymbolNamespace::Constructor, components)
    }

    pub fn primitive(symbol: impl Into<String>) -> Self {
        Self::new(SymbolNamespace::Primitive, vec![symbol.into()])
    }

    pub fn obligation(id: impl Into<String>) -> Self {
        Self::new(SymbolNamespace::Obligation, vec![id.into()])
    }

    pub fn assumption(target: &StableSymbol, kind: impl Into<String>) -> Self {
        let mut components = target.components.clone();
        components.push(kind.into());
        Self::new(SymbolNamespace::Assumption, components)
    }

    fn encode(&self, out: &mut CanonicalSink) {
        out.tag("symbol");
        out.str(self.namespace.as_str());
        out.seq_len(self.components.len());
        for component in &self.components {
            out.str(component);
        }
    }
}

impl fmt::Display for StableSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.namespace.as_str())?;
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{component}")?;
        }
        Ok(())
    }
}

/// Producer-local mapping from kernel ids to stable symbols.
#[derive(Clone, Debug, Default)]
pub struct StableSymbolTable {
    globals: BTreeMap<GlobalId, StableSymbol>,
}

impl StableSymbolTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_global(&mut self, id: GlobalId, symbol: StableSymbol) -> Option<StableSymbol> {
        self.globals.insert(id, symbol)
    }

    pub fn resolve_global(&self, id: GlobalId) -> Result<&StableSymbol, CanonicalEncodingError> {
        self.globals
            .get(&id)
            .ok_or(CanonicalEncodingError::MissingStableSymbol(id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonicalEncodingError {
    MissingStableSymbol(GlobalId),
}

impl fmt::Display for CanonicalEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanonicalEncodingError::MissingStableSymbol(id) => {
                write!(f, "missing stable symbol for local {id}")
            }
        }
    }
}

impl std::error::Error for CanonicalEncodingError {}

/// Inputs that participate in `core_semantic_hash`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckedCoreSemanticInputs {
    pub symbols: BTreeSet<StableSymbol>,
    pub declarations: BTreeMap<StableSymbol, Vec<u8>>,
    pub primitive_refs: BTreeMap<StableSymbol, String>,
    pub primitive_metadata: BTreeMap<StableSymbol, PrimitiveMetadata>,
    pub data_metadata: BTreeMap<StableSymbol, DataMetadata>,
    pub record_sigma_metadata: BTreeMap<StableSymbol, RecordSigmaMetadata>,
    pub class_instance_metadata: BTreeMap<StableSymbol, ClassInstanceMetadata>,
    pub recursion_metadata: BTreeMap<StableSymbol, RecursionMetadata>,
    pub effects_foreign_metadata: BTreeMap<StableSymbol, EffectsForeignMetadata>,
    pub metadata: BTreeMap<StableSymbol, Vec<u8>>,
    pub lowerability: BTreeMap<StableSymbol, LowerabilityStatus>,
    pub obligation_metadata: BTreeMap<StableSymbol, ObligationMetadata>,
    pub assumption_trust_metadata: BTreeMap<StableSymbol, AssumptionTrustMetadata>,
    pub obligations: BTreeMap<StableSymbol, Vec<u8>>,
    pub assumptions: BTreeMap<StableSymbol, Vec<u8>>,
    pub trusted_base_delta: BTreeMap<StableSymbol, Vec<u8>>,
    pub dependency_semantic_hashes: BTreeMap<StableSymbol, String>,
    pub unsupported: BTreeMap<StableSymbol, Vec<u8>>,
}

/// Non-semantic envelope inputs for `artifact_hash`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckedCoreArtifactInputs {
    pub semantic: CheckedCoreSemanticInputs,
    pub source_identity: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCorePackageHeader {
    pub package_kind: String,
    pub version: Option<u32>,
    pub producer: String,
    pub kernel_ref: String,
    pub spec_ref: String,
    pub primitive_registry_ref: String,
    pub package_identity: StableSymbol,
    pub dependency_semantic_hashes: BTreeMap<StableSymbol, String>,
}

impl CheckedCorePackageHeader {
    pub fn v0(
        producer: impl Into<String>,
        kernel_ref: impl Into<String>,
        spec_ref: impl Into<String>,
        primitive_registry_ref: impl Into<String>,
        package_identity: StableSymbol,
    ) -> Self {
        Self {
            package_kind: CHECKED_CORE_PACKAGE_KIND.to_string(),
            version: Some(CHECKED_CORE_SCHEMA_VERSION),
            producer: producer.into(),
            kernel_ref: kernel_ref.into(),
            spec_ref: spec_ref.into(),
            primitive_registry_ref: primitive_registry_ref.into(),
            package_identity,
            dependency_semantic_hashes: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCorePackage {
    pub header: CheckedCorePackageHeader,
    pub artifact: CheckedCoreArtifactInputs,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
}

impl CheckedCorePackage {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_package_bytes(
            &self.header,
            &self.artifact,
            self.core_semantic_hash,
            self.artifact_hash,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumedCheckedCorePackage {
    pub package_identity: StableSymbol,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub symbols: BTreeSet<StableSymbol>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreFixture {
    pub name: String,
    pub package: CheckedCorePackage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCorePackageError {
    UnsupportedPackageKind {
        found: String,
    },
    MissingVersion,
    UnsupportedVersion {
        found: u32,
    },
    EmptyHeaderField {
        field: &'static str,
    },
    MissingSymbol {
        section: &'static str,
        symbol: StableSymbol,
    },
    MissingLowerability {
        symbol: StableSymbol,
    },
    UnsupportedEntryNotBlocking {
        symbol: StableSymbol,
    },
    SemanticHashMismatch {
        expected: u64,
        actual: u64,
    },
    ArtifactHashMismatch {
        expected: u64,
        actual: u64,
    },
    LoweringReadiness(LoweringReadinessError),
}

impl fmt::Display for CheckedCorePackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckedCorePackageError::UnsupportedPackageKind { found } => {
                write!(f, "unsupported checked-core package kind {found:?}")
            }
            CheckedCorePackageError::MissingVersion => {
                write!(f, "missing checked-core package version")
            }
            CheckedCorePackageError::UnsupportedVersion { found } => {
                write!(f, "unsupported checked-core package version {found}")
            }
            CheckedCorePackageError::EmptyHeaderField { field } => {
                write!(f, "empty checked-core package header field {field}")
            }
            CheckedCorePackageError::MissingSymbol { section, symbol } => {
                write!(f, "{section} references undeclared stable symbol {symbol}")
            }
            CheckedCorePackageError::MissingLowerability { symbol } => {
                write!(f, "missing lowerability metadata for {symbol}")
            }
            CheckedCorePackageError::UnsupportedEntryNotBlocking { symbol } => write!(
                f,
                "unsupported entry for {symbol} must also have blocking lowerability"
            ),
            CheckedCorePackageError::SemanticHashMismatch { expected, actual } => write!(
                f,
                "checked-core semantic hash mismatch: expected {expected:#x}, got {actual:#x}"
            ),
            CheckedCorePackageError::ArtifactHashMismatch { expected, actual } => write!(
                f,
                "checked-core artifact hash mismatch: expected {expected:#x}, got {actual:#x}"
            ),
            CheckedCorePackageError::LoweringReadiness(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for CheckedCorePackageError {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LowerabilityStatus {
    Supported,
    Unsupported { reason: String },
    Deferred { later_stage: String, reason: String },
    RequiresFeature { feature: String, reason: String },
    Explicit { state: String, reason: String },
}

impl LowerabilityStatus {
    pub fn blocks_lowering(&self) -> bool {
        matches!(
            self,
            LowerabilityStatus::Unsupported { .. }
                | LowerabilityStatus::Deferred { .. }
                | LowerabilityStatus::RequiresFeature { .. }
                | LowerabilityStatus::Explicit { .. }
        )
    }

    fn encode(&self, out: &mut CanonicalSink) {
        match self {
            LowerabilityStatus::Supported => out.tag("supported"),
            LowerabilityStatus::Unsupported { reason } => {
                out.tag("unsupported");
                out.str(reason);
            }
            LowerabilityStatus::Deferred {
                later_stage,
                reason,
            } => {
                out.tag("deferred");
                out.str(later_stage);
                out.str(reason);
            }
            LowerabilityStatus::RequiresFeature { feature, reason } => {
                out.tag("requires_feature");
                out.str(feature);
                out.str(reason);
            }
            LowerabilityStatus::Explicit { state, reason } => {
                out.tag("explicit");
                out.str(state);
                out.str(reason);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoweringReadinessError {
    MissingLowerability {
        symbol: StableSymbol,
    },
    Blocked {
        symbol: StableSymbol,
        status: LowerabilityStatus,
    },
}

impl fmt::Display for LoweringReadinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoweringReadinessError::MissingLowerability { symbol } => {
                write!(f, "missing lowerability metadata for {symbol}")
            }
            LoweringReadinessError::Blocked { symbol, status } => {
                write!(f, "lowering blocked for {symbol}: {status:?}")
            }
        }
    }
}

impl std::error::Error for LoweringReadinessError {}

pub fn ensure_lowerable_for_target<'a>(
    target_closure: impl IntoIterator<Item = &'a StableSymbol>,
    lowerability: &BTreeMap<StableSymbol, LowerabilityStatus>,
) -> Result<(), LoweringReadinessError> {
    for symbol in target_closure {
        let status = lowerability.get(symbol).ok_or_else(|| {
            LoweringReadinessError::MissingLowerability {
                symbol: symbol.clone(),
            }
        })?;
        if status.blocks_lowering() {
            return Err(LoweringReadinessError::Blocked {
                symbol: symbol.clone(),
                status: status.clone(),
            });
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimitiveMetadata {
    pub registry_symbol: String,
    pub reduction: PrimitiveReductionMetadata,
    pub partiality: PartialityMetadata,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrimitiveReductionMetadata {
    OpaqueType,
    Literal,
    Op,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PartialityMetadata {
    Total,
    CheckedPartial { obligation: StableSymbol },
    TrustedPartial { assumption: StableSymbol },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataMetadata {
    pub parameter_count: usize,
    pub index_count: usize,
    pub constructors: Vec<ConstructorMetadata>,
    pub eliminator: LowerabilityStatus,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructorMetadata {
    pub symbol: StableSymbol,
    pub argument_count: usize,
    pub target_index_count: usize,
    pub recursive_positions: Vec<usize>,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordSigmaMetadata {
    pub kind: RecordSigmaKind,
    pub fields: Vec<FieldMetadata>,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordSigmaKind {
    Record,
    Sigma,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldMetadata {
    pub name: String,
    pub ty: StableSymbol,
    pub runtime: RuntimeFieldStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeFieldStatus {
    Runtime,
    ErasedLaw,
    ErasedProof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassInstanceMetadata {
    pub kind: ClassInstanceKind,
    pub class_symbol: Option<StableSymbol>,
    pub dictionary_symbol: Option<StableSymbol>,
    pub head_symbol: Option<StableSymbol>,
    pub field_order: Vec<String>,
    pub law_fields: BTreeSet<String>,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClassInstanceKind {
    Class,
    Instance,
    Dictionary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursionMetadata {
    pub group_members: Vec<StableSymbol>,
    pub admission: RecursionAdmission,
    pub scc_index: usize,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecursionAdmission {
    NonRecursive,
    AcceptedStructural,
    AcceptedSizeChange,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectsForeignMetadata {
    pub declared_effects: BTreeSet<String>,
    pub capabilities: BTreeSet<StableSymbol>,
    pub foreign_symbol: Option<String>,
    pub boundary: EffectBoundary,
    pub runtime_checks: BTreeSet<StableSymbol>,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectBoundary {
    Pure,
    Effectful,
    Foreign,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObligationMetadata {
    pub status: ObligationStatus,
    pub origin: StableSymbol,
    pub affects_runtime_meaning: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObligationStatus {
    Proved,
    Tested,
    Delegated,
    Unknown,
    Disproved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssumptionTrustMetadata {
    pub kind: AssumptionTrustKind,
    pub target: StableSymbol,
    pub affects_runtime_meaning: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssumptionTrustKind {
    Postulate,
    Hole,
    Foreign,
    Declassify,
    PrimitiveAssumption,
}

pub fn canonical_level_bytes(level: &Level) -> Vec<u8> {
    let mut out = CanonicalSink::new();
    encode_level(&level.normalize(), &mut out);
    out.finish()
}

pub fn canonical_term_bytes(
    term: &Term,
    symbols: &StableSymbolTable,
) -> Result<Vec<u8>, CanonicalEncodingError> {
    let mut out = CanonicalSink::new();
    encode_term(term, symbols, &mut out)?;
    Ok(out.finish())
}

pub fn canonical_decl_bytes(
    decl: &Decl,
    symbols: &StableSymbolTable,
) -> Result<Vec<u8>, CanonicalEncodingError> {
    let mut out = CanonicalSink::new();
    encode_decl(decl, symbols, &mut out)?;
    Ok(out.finish())
}

pub fn canonical_semantic_bytes(inputs: &CheckedCoreSemanticInputs) -> Vec<u8> {
    let mut out = CanonicalSink::new();
    out.tag("CheckedCorePackage");
    out.u64(0);
    encode_symbol_set("symbols", &inputs.symbols, &mut out);
    encode_bytes_map("declarations", &inputs.declarations, &mut out);
    encode_string_map("primitive_refs", &inputs.primitive_refs, &mut out);
    encode_primitive_metadata_map("primitive_metadata", &inputs.primitive_metadata, &mut out);
    encode_data_metadata_map("data_metadata", &inputs.data_metadata, &mut out);
    encode_record_sigma_metadata_map(
        "record_sigma_metadata",
        &inputs.record_sigma_metadata,
        &mut out,
    );
    encode_class_instance_metadata_map(
        "class_instance_metadata",
        &inputs.class_instance_metadata,
        &mut out,
    );
    encode_recursion_metadata_map("recursion_metadata", &inputs.recursion_metadata, &mut out);
    encode_effects_foreign_metadata_map(
        "effects_foreign_metadata",
        &inputs.effects_foreign_metadata,
        &mut out,
    );
    encode_bytes_map("metadata", &inputs.metadata, &mut out);
    encode_lowerability_map("lowerability", &inputs.lowerability, &mut out);
    encode_obligation_metadata_map("obligation_metadata", &inputs.obligation_metadata, &mut out);
    encode_assumption_trust_metadata_map(
        "assumption_trust_metadata",
        &inputs.assumption_trust_metadata,
        &mut out,
    );
    encode_bytes_map("obligations", &inputs.obligations, &mut out);
    encode_bytes_map("assumptions", &inputs.assumptions, &mut out);
    encode_bytes_map("trusted_base_delta", &inputs.trusted_base_delta, &mut out);
    encode_string_map(
        "dependency_semantic_hashes",
        &inputs.dependency_semantic_hashes,
        &mut out,
    );
    encode_bytes_map("unsupported", &inputs.unsupported, &mut out);
    out.finish()
}

pub fn canonical_artifact_bytes(inputs: &CheckedCoreArtifactInputs) -> Vec<u8> {
    let mut out = CanonicalSink::new();
    out.tag("CheckedCoreArtifact");
    out.bytes(&canonical_semantic_bytes(&inputs.semantic));
    out.tag("source_identity");
    out.seq_len(inputs.source_identity.len());
    for (key, value) in &inputs.source_identity {
        out.str(key);
        out.str(value);
    }
    out.tag("annotations");
    out.seq_len(inputs.annotations.len());
    for (key, value) in &inputs.annotations {
        out.str(key);
        out.bytes(value);
    }
    out.finish()
}

/// Deterministic non-cryptographic fingerprint for tests and internal
/// comparison. The production hash algorithm is selected by the package
/// contract; the load-bearing property here is the canonical byte input.
pub fn semantic_fingerprint(inputs: &CheckedCoreSemanticInputs) -> u64 {
    fingerprint(&canonical_semantic_bytes(inputs))
}

pub fn artifact_fingerprint(inputs: &CheckedCoreArtifactInputs) -> u64 {
    fingerprint(&canonical_artifact_bytes(inputs))
}

pub fn emit_checked_core_package(
    header: CheckedCorePackageHeader,
    mut artifact: CheckedCoreArtifactInputs,
) -> Result<CheckedCorePackage, CheckedCorePackageError> {
    validate_header(&header)?;
    materialize_emitter_completeness(&mut artifact.semantic);

    let core_semantic_hash = semantic_fingerprint(&artifact.semantic);
    let artifact_hash = package_artifact_fingerprint(&header, &artifact, core_semantic_hash);
    let package = CheckedCorePackage {
        header,
        artifact,
        core_semantic_hash,
        artifact_hash,
    };
    validate_checked_core_package(&package)?;
    Ok(package)
}

pub fn validate_checked_core_package(
    package: &CheckedCorePackage,
) -> Result<(), CheckedCorePackageError> {
    validate_header(&package.header)?;
    validate_semantic_contract(&package.artifact.semantic)?;

    let expected_semantic = semantic_fingerprint(&package.artifact.semantic);
    if package.core_semantic_hash != expected_semantic {
        return Err(CheckedCorePackageError::SemanticHashMismatch {
            expected: expected_semantic,
            actual: package.core_semantic_hash,
        });
    }

    let expected_artifact = package_artifact_fingerprint(
        &package.header,
        &package.artifact,
        package.core_semantic_hash,
    );
    if package.artifact_hash != expected_artifact {
        return Err(CheckedCorePackageError::ArtifactHashMismatch {
            expected: expected_artifact,
            actual: package.artifact_hash,
        });
    }

    Ok(())
}

pub fn consume_checked_core_package_for_target<'a>(
    package: &CheckedCorePackage,
    target_closure: impl IntoIterator<Item = &'a StableSymbol>,
) -> Result<ConsumedCheckedCorePackage, CheckedCorePackageError> {
    validate_checked_core_package(package)?;
    ensure_lowerable_for_target(target_closure, &package.artifact.semantic.lowerability)
        .map_err(CheckedCorePackageError::LoweringReadiness)?;

    Ok(ConsumedCheckedCorePackage {
        package_identity: package.header.package_identity.clone(),
        core_semantic_hash: package.core_semantic_hash,
        artifact_hash: package.artifact_hash,
        symbols: package.artifact.semantic.symbols.clone(),
    })
}

pub fn representative_checked_core_fixtures(
) -> Result<Vec<CheckedCoreFixture>, CheckedCorePackageError> {
    Ok(vec![CheckedCoreFixture {
        name: "bool-nat-option-list-dictionary-effects".to_string(),
        package: emit_checked_core_package(
            fixture_header("bool_nat_option_list_dictionary_effects"),
            CheckedCoreArtifactInputs {
                semantic: representative_fixture_semantic_inputs(),
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )?,
    }])
}

fn validate_header(header: &CheckedCorePackageHeader) -> Result<(), CheckedCorePackageError> {
    if header.package_kind != CHECKED_CORE_PACKAGE_KIND {
        return Err(CheckedCorePackageError::UnsupportedPackageKind {
            found: header.package_kind.clone(),
        });
    }
    match header.version {
        Some(CHECKED_CORE_SCHEMA_VERSION) => {}
        Some(found) => return Err(CheckedCorePackageError::UnsupportedVersion { found }),
        None => return Err(CheckedCorePackageError::MissingVersion),
    }
    for (field, value) in [
        ("producer", &header.producer),
        ("kernel_ref", &header.kernel_ref),
        ("spec_ref", &header.spec_ref),
        ("primitive_registry_ref", &header.primitive_registry_ref),
    ] {
        if value.is_empty() {
            return Err(CheckedCorePackageError::EmptyHeaderField { field });
        }
    }
    Ok(())
}

fn validate_semantic_contract(
    semantic: &CheckedCoreSemanticInputs,
) -> Result<(), CheckedCorePackageError> {
    for (section, symbol) in semantic_symbol_references(semantic) {
        if !semantic.symbols.contains(&symbol) {
            return Err(CheckedCorePackageError::MissingSymbol { section, symbol });
        }
    }

    for symbol in compiler_relevant_symbols(semantic) {
        if !semantic.lowerability.contains_key(&symbol) {
            return Err(CheckedCorePackageError::MissingLowerability { symbol });
        }
    }

    for symbol in semantic.unsupported.keys() {
        match semantic.lowerability.get(symbol) {
            Some(status) if status.blocks_lowering() => {}
            _ => {
                return Err(CheckedCorePackageError::UnsupportedEntryNotBlocking {
                    symbol: symbol.clone(),
                })
            }
        }
    }

    Ok(())
}

fn materialize_emitter_completeness(semantic: &mut CheckedCoreSemanticInputs) {
    for (_, symbol) in semantic_symbol_references(semantic) {
        semantic.symbols.insert(symbol);
    }

    for symbol in compiler_relevant_symbols(semantic) {
        semantic.symbols.insert(symbol.clone());
        semantic.lowerability.entry(symbol.clone()).or_insert_with(|| {
            let reason = format!(
                "NC4 emitter has no compiler-relevant metadata for {symbol}; explicit unsupported entry materialized instead of omitting it"
            );
            semantic
                .unsupported
                .entry(symbol.clone())
                .or_insert_with(|| reason.as_bytes().to_vec());
            LowerabilityStatus::Unsupported { reason }
        });
    }
}

fn compiler_relevant_symbols(semantic: &CheckedCoreSemanticInputs) -> BTreeSet<StableSymbol> {
    let mut symbols = BTreeSet::new();
    symbols.extend(semantic.declarations.keys().cloned());
    symbols.extend(semantic.primitive_refs.keys().cloned());
    symbols.extend(semantic.primitive_metadata.keys().cloned());
    symbols.extend(semantic.data_metadata.keys().cloned());
    for meta in semantic.data_metadata.values() {
        for ctor in &meta.constructors {
            symbols.insert(ctor.symbol.clone());
        }
    }
    symbols.extend(semantic.record_sigma_metadata.keys().cloned());
    symbols.extend(semantic.class_instance_metadata.keys().cloned());
    symbols.extend(semantic.recursion_metadata.keys().cloned());
    for meta in semantic.recursion_metadata.values() {
        symbols.extend(meta.group_members.iter().cloned());
    }
    symbols.extend(semantic.effects_foreign_metadata.keys().cloned());
    symbols
}

fn semantic_symbol_references(
    semantic: &CheckedCoreSemanticInputs,
) -> Vec<(&'static str, StableSymbol)> {
    let mut refs = Vec::new();
    refs.extend(
        semantic
            .declarations
            .keys()
            .cloned()
            .map(|symbol| ("declarations", symbol)),
    );
    refs.extend(
        semantic
            .primitive_refs
            .keys()
            .cloned()
            .map(|symbol| ("primitive_refs", symbol)),
    );
    refs.extend(
        semantic
            .primitive_metadata
            .keys()
            .cloned()
            .map(|symbol| ("primitive_metadata", symbol)),
    );
    for (symbol, meta) in &semantic.primitive_metadata {
        refs.push(("primitive_metadata", symbol.clone()));
        match &meta.partiality {
            PartialityMetadata::Total => {}
            PartialityMetadata::CheckedPartial { obligation } => {
                refs.push(("primitive_metadata.partiality", obligation.clone()));
            }
            PartialityMetadata::TrustedPartial { assumption } => {
                refs.push(("primitive_metadata.partiality", assumption.clone()));
            }
        }
    }
    for (symbol, meta) in &semantic.data_metadata {
        refs.push(("data_metadata", symbol.clone()));
        for ctor in &meta.constructors {
            refs.push(("data_metadata.constructors", ctor.symbol.clone()));
        }
    }
    for (symbol, meta) in &semantic.record_sigma_metadata {
        refs.push(("record_sigma_metadata", symbol.clone()));
        for field in &meta.fields {
            refs.push(("record_sigma_metadata.fields", field.ty.clone()));
        }
    }
    for (symbol, meta) in &semantic.class_instance_metadata {
        refs.push(("class_instance_metadata", symbol.clone()));
        if let Some(class_symbol) = &meta.class_symbol {
            refs.push(("class_instance_metadata.class", class_symbol.clone()));
        }
        if let Some(dictionary_symbol) = &meta.dictionary_symbol {
            refs.push((
                "class_instance_metadata.dictionary",
                dictionary_symbol.clone(),
            ));
        }
        if let Some(head_symbol) = &meta.head_symbol {
            refs.push(("class_instance_metadata.head", head_symbol.clone()));
        }
    }
    for (symbol, meta) in &semantic.recursion_metadata {
        refs.push(("recursion_metadata", symbol.clone()));
        for member in &meta.group_members {
            refs.push(("recursion_metadata.group_members", member.clone()));
        }
    }
    for (symbol, meta) in &semantic.effects_foreign_metadata {
        refs.push(("effects_foreign_metadata", symbol.clone()));
        for capability in &meta.capabilities {
            refs.push(("effects_foreign_metadata.capabilities", capability.clone()));
        }
        for runtime_check in &meta.runtime_checks {
            refs.push((
                "effects_foreign_metadata.runtime_checks",
                runtime_check.clone(),
            ));
        }
    }
    for (symbol, meta) in &semantic.obligation_metadata {
        refs.push(("obligation_metadata", symbol.clone()));
        refs.push(("obligation_metadata.origin", meta.origin.clone()));
    }
    for (symbol, meta) in &semantic.assumption_trust_metadata {
        refs.push(("assumption_trust_metadata", symbol.clone()));
        refs.push(("assumption_trust_metadata.target", meta.target.clone()));
    }
    refs.extend(
        semantic
            .obligations
            .keys()
            .cloned()
            .map(|symbol| ("obligations", symbol)),
    );
    refs.extend(
        semantic
            .assumptions
            .keys()
            .cloned()
            .map(|symbol| ("assumptions", symbol)),
    );
    refs.extend(
        semantic
            .trusted_base_delta
            .keys()
            .cloned()
            .map(|symbol| ("trusted_base_delta", symbol)),
    );
    refs.extend(
        semantic
            .dependency_semantic_hashes
            .keys()
            .cloned()
            .map(|symbol| ("dependency_semantic_hashes", symbol)),
    );
    refs.extend(
        semantic
            .unsupported
            .keys()
            .cloned()
            .map(|symbol| ("unsupported", symbol)),
    );
    refs.extend(
        semantic
            .lowerability
            .keys()
            .cloned()
            .map(|symbol| ("lowerability", symbol)),
    );
    refs
}

fn package_artifact_fingerprint(
    header: &CheckedCorePackageHeader,
    artifact: &CheckedCoreArtifactInputs,
    core_semantic_hash: u64,
) -> u64 {
    fingerprint(&canonical_package_envelope_bytes(
        header,
        artifact,
        core_semantic_hash,
    ))
}

fn canonical_package_bytes(
    header: &CheckedCorePackageHeader,
    artifact: &CheckedCoreArtifactInputs,
    core_semantic_hash: u64,
    artifact_hash: u64,
) -> Vec<u8> {
    let mut out = CanonicalSink::new();
    out.bytes(&canonical_package_envelope_bytes(
        header,
        artifact,
        core_semantic_hash,
    ));
    out.tag("artifact_hash");
    out.u64(artifact_hash);
    out.finish()
}

fn canonical_package_envelope_bytes(
    header: &CheckedCorePackageHeader,
    artifact: &CheckedCoreArtifactInputs,
    core_semantic_hash: u64,
) -> Vec<u8> {
    let mut out = CanonicalSink::new();
    out.tag("CheckedCorePackageEnvelope");
    encode_package_header(header, &mut out);
    out.tag("core_semantic_hash");
    out.u64(core_semantic_hash);
    out.bytes(&canonical_artifact_bytes(artifact));
    out.finish()
}

fn encode_package_header(header: &CheckedCorePackageHeader, out: &mut CanonicalSink) {
    out.tag("header");
    out.str(&header.package_kind);
    match header.version {
        Some(version) => {
            out.tag("version_some");
            out.u64(u64::from(version));
        }
        None => out.tag("version_none"),
    }
    out.str(&header.producer);
    out.str(&header.kernel_ref);
    out.str(&header.spec_ref);
    out.str(&header.primitive_registry_ref);
    header.package_identity.encode(out);
    encode_string_map(
        "dependency_semantic_hashes",
        &header.dependency_semantic_hashes,
        out,
    );
}

fn fixture_header(name: &str) -> CheckedCorePackageHeader {
    CheckedCorePackageHeader::v0(
        "ken-elaborator:checked-core-emitter",
        "ken-kernel:current",
        "spec/40-runtime/46-checked-core-package.md:v0",
        "spec/10-kernel/18a-primitive-registry.md:current",
        StableSymbol::new(
            SymbolNamespace::Module,
            vec!["fixture".to_string(), name.to_string()],
        ),
    )
}

fn representative_fixture_semantic_inputs() -> CheckedCoreSemanticInputs {
    let bool_ty = StableSymbol::declaration("fixture", &["Core"], "Bool");
    let false_ctor = StableSymbol::constructor(&bool_ty, "False");
    let true_ctor = StableSymbol::constructor(&bool_ty, "True");
    let nat_ty = StableSymbol::declaration("fixture", &["Core"], "Nat");
    let zero_ctor = StableSymbol::constructor(&nat_ty, "Zero");
    let succ_ctor = StableSymbol::constructor(&nat_ty, "Succ");
    let option_ty = StableSymbol::declaration("fixture", &["Core"], "Option");
    let none_ctor = StableSymbol::constructor(&option_ty, "None");
    let some_ctor = StableSymbol::constructor(&option_ty, "Some");
    let list_ty = StableSymbol::declaration("fixture", &["Core"], "List");
    let nil_ctor = StableSymbol::constructor(&list_ty, "Nil");
    let cons_ctor = StableSymbol::constructor(&list_ty, "Cons");
    let eq_class = StableSymbol::declaration("fixture", &["Classes"], "Eq");
    let eq_bool_dict = StableSymbol::declaration("fixture", &["Classes"], "EqBoolDict");
    let add_nat = StableSymbol::primitive("nat_add");
    let append_group = StableSymbol::declaration("fixture", &["Core"], "List.append.group");
    let effectful = StableSymbol::declaration("fixture", &["Effects"], "print_line");
    let cap = StableSymbol::new(
        SymbolNamespace::Metadata,
        vec!["fixture".to_string(), "ConsoleCap".to_string()],
    );

    let mut inputs = CheckedCoreSemanticInputs::default();
    for symbol in [
        bool_ty.clone(),
        false_ctor.clone(),
        true_ctor.clone(),
        nat_ty.clone(),
        zero_ctor.clone(),
        succ_ctor.clone(),
        option_ty.clone(),
        none_ctor.clone(),
        some_ctor.clone(),
        list_ty.clone(),
        nil_ctor.clone(),
        cons_ctor.clone(),
        eq_class.clone(),
        eq_bool_dict.clone(),
        add_nat.clone(),
        append_group.clone(),
        effectful.clone(),
        cap.clone(),
    ] {
        inputs.symbols.insert(symbol);
    }

    for symbol in [
        bool_ty.clone(),
        nat_ty.clone(),
        option_ty.clone(),
        list_ty.clone(),
        eq_class.clone(),
        eq_bool_dict.clone(),
        append_group.clone(),
        effectful.clone(),
    ] {
        inputs.declarations.insert(
            symbol.clone(),
            format!("checked-decl:{symbol}").into_bytes(),
        );
    }

    inputs.data_metadata.insert(
        bool_ty.clone(),
        DataMetadata {
            parameter_count: 0,
            index_count: 0,
            constructors: vec![
                ConstructorMetadata {
                    symbol: false_ctor.clone(),
                    argument_count: 0,
                    target_index_count: 0,
                    recursive_positions: Vec::new(),
                    lowerability: LowerabilityStatus::Supported,
                },
                ConstructorMetadata {
                    symbol: true_ctor.clone(),
                    argument_count: 0,
                    target_index_count: 0,
                    recursive_positions: Vec::new(),
                    lowerability: LowerabilityStatus::Supported,
                },
            ],
            eliminator: LowerabilityStatus::Supported,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs.data_metadata.insert(
        nat_ty.clone(),
        DataMetadata {
            parameter_count: 0,
            index_count: 0,
            constructors: vec![
                ConstructorMetadata {
                    symbol: zero_ctor.clone(),
                    argument_count: 0,
                    target_index_count: 0,
                    recursive_positions: Vec::new(),
                    lowerability: LowerabilityStatus::Supported,
                },
                ConstructorMetadata {
                    symbol: succ_ctor.clone(),
                    argument_count: 1,
                    target_index_count: 0,
                    recursive_positions: vec![0],
                    lowerability: LowerabilityStatus::Supported,
                },
            ],
            eliminator: LowerabilityStatus::Supported,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs.data_metadata.insert(
        option_ty.clone(),
        DataMetadata {
            parameter_count: 1,
            index_count: 0,
            constructors: vec![
                ConstructorMetadata {
                    symbol: none_ctor.clone(),
                    argument_count: 0,
                    target_index_count: 0,
                    recursive_positions: Vec::new(),
                    lowerability: LowerabilityStatus::Supported,
                },
                ConstructorMetadata {
                    symbol: some_ctor.clone(),
                    argument_count: 1,
                    target_index_count: 0,
                    recursive_positions: Vec::new(),
                    lowerability: LowerabilityStatus::Supported,
                },
            ],
            eliminator: LowerabilityStatus::Supported,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs.data_metadata.insert(
        list_ty.clone(),
        DataMetadata {
            parameter_count: 1,
            index_count: 0,
            constructors: vec![
                ConstructorMetadata {
                    symbol: nil_ctor.clone(),
                    argument_count: 0,
                    target_index_count: 0,
                    recursive_positions: Vec::new(),
                    lowerability: LowerabilityStatus::Supported,
                },
                ConstructorMetadata {
                    symbol: cons_ctor.clone(),
                    argument_count: 2,
                    target_index_count: 0,
                    recursive_positions: vec![1],
                    lowerability: LowerabilityStatus::Supported,
                },
            ],
            eliminator: LowerabilityStatus::Supported,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs.class_instance_metadata.insert(
        eq_bool_dict.clone(),
        ClassInstanceMetadata {
            kind: ClassInstanceKind::Dictionary,
            class_symbol: Some(eq_class.clone()),
            dictionary_symbol: Some(eq_bool_dict.clone()),
            head_symbol: Some(bool_ty.clone()),
            field_order: vec!["eq".to_string(), "refl".to_string()],
            law_fields: BTreeSet::from(["refl".to_string()]),
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs
        .primitive_refs
        .insert(add_nat.clone(), "primitive-registry:nat_add".to_string());
    inputs.primitive_metadata.insert(
        add_nat.clone(),
        PrimitiveMetadata {
            registry_symbol: "nat_add".to_string(),
            reduction: PrimitiveReductionMetadata::Op,
            partiality: PartialityMetadata::Total,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs.recursion_metadata.insert(
        append_group.clone(),
        RecursionMetadata {
            group_members: vec![append_group.clone()],
            admission: RecursionAdmission::AcceptedStructural,
            scc_index: 0,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    inputs.effects_foreign_metadata.insert(
        effectful.clone(),
        EffectsForeignMetadata {
            declared_effects: BTreeSet::from(["Console".to_string()]),
            capabilities: BTreeSet::from([cap.clone()]),
            foreign_symbol: None,
            boundary: EffectBoundary::Effectful,
            runtime_checks: BTreeSet::new(),
            lowerability: LowerabilityStatus::Supported,
        },
    );

    for symbol in compiler_relevant_symbols(&inputs) {
        inputs
            .lowerability
            .insert(symbol, LowerabilityStatus::Supported);
    }

    inputs
}

fn encode_decl(
    decl: &Decl,
    symbols: &StableSymbolTable,
    out: &mut CanonicalSink,
) -> Result<(), CanonicalEncodingError> {
    match decl {
        Decl::Transparent {
            id,
            level_params,
            ty,
            body,
        } => {
            out.tag("transparent");
            encode_global(*id, symbols, out)?;
            encode_level_params(level_params, out);
            encode_term(ty, symbols, out)?;
            encode_term(body, symbols, out)?;
        }
        Decl::Opaque {
            id,
            level_params,
            ty,
        } => {
            out.tag("opaque");
            encode_global(*id, symbols, out)?;
            encode_level_params(level_params, out);
            encode_term(ty, symbols, out)?;
        }
        Decl::Primitive {
            id,
            level_params,
            ty,
            reduction,
        } => {
            out.tag("primitive");
            encode_global(*id, symbols, out)?;
            encode_level_params(level_params, out);
            encode_term(ty, symbols, out)?;
            match reduction {
                PrimReduction::OpaqueType => out.tag("opaque_type"),
                PrimReduction::Literal => out.tag("literal"),
                PrimReduction::Op { symbol } => {
                    out.tag("op");
                    out.str(symbol);
                }
            }
        }
        Decl::Inductive(ind) => {
            out.tag("inductive");
            encode_global(ind.id, symbols, out)?;
            encode_level_params(&ind.level_params, out);
            encode_terms(&ind.params, symbols, out)?;
            encode_terms(&ind.indices, symbols, out)?;
            encode_level(&ind.level, out);
            encode_term(&ind.former_type, symbols, out)?;
            out.tag("constructors");
            out.seq_len(ind.constructors.len());
            for ctor in &ind.constructors {
                out.tag("constructor");
                encode_global(ctor.id, symbols, out)?;
                encode_terms(&ctor.args, symbols, out)?;
                encode_terms(&ctor.target_indices, symbols, out)?;
                encode_term(&ctor.type_, symbols, out)?;
                out.tag("recursive_positions");
                out.seq_len(ctor.recursive_positions.len());
                for pos in &ctor.recursive_positions {
                    out.u64(*pos as u64);
                }
            }
        }
    }
    Ok(())
}

fn encode_terms(
    terms: &[Term],
    symbols: &StableSymbolTable,
    out: &mut CanonicalSink,
) -> Result<(), CanonicalEncodingError> {
    out.seq_len(terms.len());
    for term in terms {
        encode_term(term, symbols, out)?;
    }
    Ok(())
}

fn encode_term(
    term: &Term,
    symbols: &StableSymbolTable,
    out: &mut CanonicalSink,
) -> Result<(), CanonicalEncodingError> {
    match term {
        Term::Type(level) => {
            out.tag("type");
            encode_level(level, out);
        }
        Term::Omega(level) => {
            out.tag("omega");
            encode_level(level, out);
        }
        Term::Var(index) => {
            out.tag("var");
            out.u64(*index as u64);
        }
        Term::Const { id, level_args } => {
            out.tag("const");
            encode_global(*id, symbols, out)?;
            encode_levels(level_args, out);
        }
        Term::IndFormer { id, level_args } => {
            out.tag("ind_former");
            encode_global(*id, symbols, out)?;
            encode_levels(level_args, out);
        }
        Term::Constructor { id, level_args } => {
            out.tag("constructor_ref");
            encode_global(*id, symbols, out)?;
            encode_levels(level_args, out);
        }
        Term::Elim {
            fam,
            level_args,
            params,
            motive,
            methods,
            indices,
            scrut,
        } => {
            out.tag("elim");
            encode_global(*fam, symbols, out)?;
            encode_levels(level_args, out);
            encode_terms(params, symbols, out)?;
            encode_term(motive, symbols, out)?;
            encode_terms(methods, symbols, out)?;
            encode_terms(indices, symbols, out)?;
            encode_term(scrut, symbols, out)?;
        }
        Term::Pi(a, b) => encode_binary("pi", a, b, symbols, out)?,
        Term::Lam(a, b) => encode_binary("lam", a, b, symbols, out)?,
        Term::App(f, a) => encode_binary("app", f, a, symbols, out)?,
        Term::Sigma(a, b) => encode_binary("sigma", a, b, symbols, out)?,
        Term::Pair(a, b) => encode_binary("pair", a, b, symbols, out)?,
        Term::Proj1(p) => encode_unary("proj1", p, symbols, out)?,
        Term::Proj2(p) => encode_unary("proj2", p, symbols, out)?,
        Term::Let { ty, val, body } => {
            out.tag("let");
            encode_term(ty, symbols, out)?;
            encode_term(val, symbols, out)?;
            encode_term(body, symbols, out)?;
        }
        Term::Ascript(t, a) => encode_binary("ascript", t, a, symbols, out)?,
        Term::Eq(a, t, u) => {
            out.tag("eq");
            encode_term(a, symbols, out)?;
            encode_term(t, symbols, out)?;
            encode_term(u, symbols, out)?;
        }
        Term::Refl(t) => encode_unary("refl", t, symbols, out)?,
        Term::Cast(a, b, e, t) => {
            out.tag("cast");
            encode_term(a, symbols, out)?;
            encode_term(b, symbols, out)?;
            encode_term(e, symbols, out)?;
            encode_term(t, symbols, out)?;
        }
        Term::J(m, d, e) => {
            out.tag("j");
            encode_term(m, symbols, out)?;
            encode_term(d, symbols, out)?;
            encode_term(e, symbols, out)?;
        }
        Term::Quot(a, r) => encode_binary("quot", a, r, symbols, out)?,
        Term::QuotClass(t) => encode_unary("quot_class", t, symbols, out)?,
        Term::QuotElim {
            motive,
            method,
            respect,
            scrut,
        } => {
            out.tag("quot_elim");
            encode_term(motive, symbols, out)?;
            encode_term(method, symbols, out)?;
            encode_term(respect, symbols, out)?;
            encode_term(scrut, symbols, out)?;
        }
        Term::Trunc(t) => encode_unary("trunc", t, symbols, out)?,
        Term::TruncProj(t) => encode_unary("trunc_proj", t, symbols, out)?,
        Term::Absurd(motive, proof) => encode_binary("absurd", motive, proof, symbols, out)?,
    }
    Ok(())
}

fn encode_unary(
    tag: &'static str,
    term: &Term,
    symbols: &StableSymbolTable,
    out: &mut CanonicalSink,
) -> Result<(), CanonicalEncodingError> {
    out.tag(tag);
    encode_term(term, symbols, out)
}

fn encode_binary(
    tag: &'static str,
    left: &Term,
    right: &Term,
    symbols: &StableSymbolTable,
    out: &mut CanonicalSink,
) -> Result<(), CanonicalEncodingError> {
    out.tag(tag);
    encode_term(left, symbols, out)?;
    encode_term(right, symbols, out)
}

fn encode_global(
    id: GlobalId,
    symbols: &StableSymbolTable,
    out: &mut CanonicalSink,
) -> Result<(), CanonicalEncodingError> {
    symbols.resolve_global(id)?.encode(out);
    Ok(())
}

fn encode_level_params(params: &[ken_kernel::LevelVar], out: &mut CanonicalSink) {
    out.tag("level_params");
    out.seq_len(params.len());
    for param in params {
        out.u64(param.0 as u64);
    }
}

fn encode_levels(levels: &[Level], out: &mut CanonicalSink) {
    out.seq_len(levels.len());
    for level in levels {
        encode_level(level, out);
    }
}

fn encode_level(level: &Level, out: &mut CanonicalSink) {
    match level.normalize() {
        Level::Zero => out.tag("level_zero"),
        Level::Suc(inner) => {
            out.tag("level_suc");
            encode_level(&inner, out);
        }
        Level::Max(left, right) => {
            out.tag("level_max");
            encode_level(&left, out);
            encode_level(&right, out);
        }
        Level::Var(var) => {
            out.tag("level_var");
            out.u64(var.0 as u64);
        }
    }
}

fn encode_symbol_set(tag: &'static str, set: &BTreeSet<StableSymbol>, out: &mut CanonicalSink) {
    out.tag(tag);
    out.seq_len(set.len());
    for symbol in set {
        symbol.encode(out);
    }
}

fn encode_bytes_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, Vec<u8>>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, value) in map {
        symbol.encode(out);
        out.bytes(value);
    }
}

fn encode_string_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, String>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, value) in map {
        symbol.encode(out);
        out.str(value);
    }
}

fn encode_lowerability_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, LowerabilityStatus>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, status) in map {
        symbol.encode(out);
        status.encode(out);
    }
}

fn encode_primitive_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, PrimitiveMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        out.str(&meta.registry_symbol);
        encode_primitive_reduction_metadata(&meta.reduction, out);
        encode_partiality_metadata(&meta.partiality, out);
        meta.lowerability.encode(out);
    }
}

fn encode_primitive_reduction_metadata(meta: &PrimitiveReductionMetadata, out: &mut CanonicalSink) {
    match meta {
        PrimitiveReductionMetadata::OpaqueType => out.tag("opaque_type"),
        PrimitiveReductionMetadata::Literal => out.tag("literal"),
        PrimitiveReductionMetadata::Op => out.tag("op"),
    }
}

fn encode_partiality_metadata(meta: &PartialityMetadata, out: &mut CanonicalSink) {
    match meta {
        PartialityMetadata::Total => out.tag("total"),
        PartialityMetadata::CheckedPartial { obligation } => {
            out.tag("checked_partial");
            obligation.encode(out);
        }
        PartialityMetadata::TrustedPartial { assumption } => {
            out.tag("trusted_partial");
            assumption.encode(out);
        }
    }
}

fn encode_data_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, DataMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        out.u64(meta.parameter_count as u64);
        out.u64(meta.index_count as u64);
        out.tag("constructors");
        out.seq_len(meta.constructors.len());
        for ctor in &meta.constructors {
            ctor.symbol.encode(out);
            out.u64(ctor.argument_count as u64);
            out.u64(ctor.target_index_count as u64);
            out.tag("recursive_positions");
            out.seq_len(ctor.recursive_positions.len());
            for pos in &ctor.recursive_positions {
                out.u64(*pos as u64);
            }
            ctor.lowerability.encode(out);
        }
        meta.eliminator.encode(out);
        meta.lowerability.encode(out);
    }
}

fn encode_record_sigma_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, RecordSigmaMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        match meta.kind {
            RecordSigmaKind::Record => out.tag("record"),
            RecordSigmaKind::Sigma => out.tag("sigma"),
        }
        out.tag("fields");
        out.seq_len(meta.fields.len());
        for field in &meta.fields {
            out.str(&field.name);
            field.ty.encode(out);
            match field.runtime {
                RuntimeFieldStatus::Runtime => out.tag("runtime"),
                RuntimeFieldStatus::ErasedLaw => out.tag("erased_law"),
                RuntimeFieldStatus::ErasedProof => out.tag("erased_proof"),
            }
        }
        meta.lowerability.encode(out);
    }
}

fn encode_class_instance_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, ClassInstanceMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        match meta.kind {
            ClassInstanceKind::Class => out.tag("class"),
            ClassInstanceKind::Instance => out.tag("instance"),
            ClassInstanceKind::Dictionary => out.tag("dictionary"),
        }
        encode_optional_symbol(&meta.class_symbol, out);
        encode_optional_symbol(&meta.dictionary_symbol, out);
        encode_optional_symbol(&meta.head_symbol, out);
        out.tag("field_order");
        out.seq_len(meta.field_order.len());
        for field in &meta.field_order {
            out.str(field);
        }
        out.tag("law_fields");
        out.seq_len(meta.law_fields.len());
        for field in &meta.law_fields {
            out.str(field);
        }
        meta.lowerability.encode(out);
    }
}

fn encode_recursion_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, RecursionMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        out.tag("group_members");
        out.seq_len(meta.group_members.len());
        for member in &meta.group_members {
            member.encode(out);
        }
        match meta.admission {
            RecursionAdmission::NonRecursive => out.tag("non_recursive"),
            RecursionAdmission::AcceptedStructural => out.tag("accepted_structural"),
            RecursionAdmission::AcceptedSizeChange => out.tag("accepted_size_change"),
            RecursionAdmission::Rejected => out.tag("rejected"),
        }
        out.u64(meta.scc_index as u64);
        meta.lowerability.encode(out);
    }
}

fn encode_effects_foreign_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, EffectsForeignMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        out.tag("declared_effects");
        out.seq_len(meta.declared_effects.len());
        for effect in &meta.declared_effects {
            out.str(effect);
        }
        encode_symbol_set("capabilities", &meta.capabilities, out);
        match &meta.foreign_symbol {
            Some(symbol) => {
                out.tag("foreign_symbol_some");
                out.str(symbol);
            }
            None => out.tag("foreign_symbol_none"),
        }
        match meta.boundary {
            EffectBoundary::Pure => out.tag("pure"),
            EffectBoundary::Effectful => out.tag("effectful"),
            EffectBoundary::Foreign => out.tag("foreign"),
        }
        encode_symbol_set("runtime_checks", &meta.runtime_checks, out);
        meta.lowerability.encode(out);
    }
}

fn encode_obligation_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, ObligationMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        match meta.status {
            ObligationStatus::Proved => out.tag("proved"),
            ObligationStatus::Tested => out.tag("tested"),
            ObligationStatus::Delegated => out.tag("delegated"),
            ObligationStatus::Unknown => out.tag("unknown"),
            ObligationStatus::Disproved => out.tag("disproved"),
        }
        meta.origin.encode(out);
        out.u64(if meta.affects_runtime_meaning { 1 } else { 0 });
    }
}

fn encode_assumption_trust_metadata_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, AssumptionTrustMetadata>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, meta) in map {
        symbol.encode(out);
        match meta.kind {
            AssumptionTrustKind::Postulate => out.tag("postulate"),
            AssumptionTrustKind::Hole => out.tag("hole"),
            AssumptionTrustKind::Foreign => out.tag("foreign"),
            AssumptionTrustKind::Declassify => out.tag("declassify"),
            AssumptionTrustKind::PrimitiveAssumption => out.tag("primitive_assumption"),
        }
        meta.target.encode(out);
        out.u64(if meta.affects_runtime_meaning { 1 } else { 0 });
    }
}

fn encode_optional_symbol(symbol: &Option<StableSymbol>, out: &mut CanonicalSink) {
    match symbol {
        Some(symbol) => {
            out.tag("some");
            symbol.encode(out);
        }
        None => out.tag("none"),
    }
}

fn fingerprint(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

struct CanonicalSink {
    bytes: Vec<u8>,
}

impl CanonicalSink {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn tag(&mut self, tag: &'static str) {
        self.str(tag);
    }

    fn str(&mut self, value: &str) {
        self.bytes
            .extend_from_slice(&(value.len() as u64).to_be_bytes());
        self.bytes.extend_from_slice(value.as_bytes());
    }

    fn bytes(&mut self, value: &[u8]) {
        self.bytes
            .extend_from_slice(&(value.len() as u64).to_be_bytes());
        self.bytes.extend_from_slice(value);
    }

    fn seq_len(&mut self, len: usize) {
        self.u64(len as u64);
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }
}

#[cfg(test)]
mod tests {
    use ken_kernel::env::{Decl, PrimReduction};
    use ken_kernel::{GlobalId, Level, LevelVar, Term};

    use super::*;

    fn decl_symbol(name: &str) -> StableSymbol {
        StableSymbol::declaration("pkg", &["M"], name)
    }

    fn table(id: GlobalId, symbol: StableSymbol) -> StableSymbolTable {
        let mut table = StableSymbolTable::new();
        table.insert_global(id, symbol);
        table
    }

    #[test]
    fn global_id_drift_does_not_enter_canonical_term_identity() {
        let symbol = decl_symbol("f");
        let bytes_a = canonical_term_bytes(
            &Term::Const {
                id: GlobalId(7),
                level_args: Vec::new(),
            },
            &table(GlobalId(7), symbol.clone()),
        )
        .unwrap();
        let bytes_b = canonical_term_bytes(
            &Term::Const {
                id: GlobalId(99),
                level_args: Vec::new(),
            },
            &table(GlobalId(99), symbol),
        )
        .unwrap();

        assert_eq!(
            bytes_a, bytes_b,
            "canonical term references must use stable symbols, not GlobalId allocation order"
        );
    }

    #[test]
    fn missing_stable_symbol_rejects_instead_of_serializing_local_id() {
        let err = canonical_term_bytes(
            &Term::Const {
                id: GlobalId(42),
                level_args: Vec::new(),
            },
            &StableSymbolTable::new(),
        )
        .unwrap_err();

        assert_eq!(
            err,
            CanonicalEncodingError::MissingStableSymbol(GlobalId(42)),
            "a producer-local GlobalId without a stable binding is not a package identity"
        );
    }

    #[test]
    fn level_encoding_uses_semilattice_normal_form() {
        let u0 = Level::Var(LevelVar(0));
        let u1 = Level::Var(LevelVar(1));
        let left = u0.clone().max(u1.clone()).max(u0.clone());
        let right = u1.max(u0);

        assert_eq!(
            canonical_level_bytes(&left),
            canonical_level_bytes(&right),
            "level max order and duplication must not perturb semantic hashes"
        );
    }

    #[test]
    fn primitive_identity_is_registry_symbol_not_primitive_global_id() {
        let ty = Term::Type(Level::zero());
        let decl_a = Decl::Primitive {
            id: GlobalId(11),
            level_params: Vec::new(),
            ty: ty.clone(),
            reduction: PrimReduction::Op { symbol: "add_int" },
        };
        let decl_b = Decl::Primitive {
            id: GlobalId(77),
            level_params: Vec::new(),
            ty,
            reduction: PrimReduction::Op { symbol: "add_int" },
        };
        let stable = StableSymbol::primitive("add_int");

        assert_eq!(
            canonical_decl_bytes(&decl_a, &table(GlobalId(11), stable.clone())).unwrap(),
            canonical_decl_bytes(&decl_b, &table(GlobalId(77), stable)).unwrap(),
            "primitive GlobalId drift must not change canonical primitive identity"
        );
    }

    #[test]
    fn semantic_sections_are_order_independent() {
        let f = decl_symbol("f");
        let g = decl_symbol("g");
        let mut a = CheckedCoreSemanticInputs::default();
        a.symbols.insert(g.clone());
        a.symbols.insert(f.clone());
        a.declarations.insert(g.clone(), b"g-body".to_vec());
        a.declarations.insert(f.clone(), b"f-body".to_vec());
        a.metadata.insert(g.clone(), b"g-meta".to_vec());
        a.metadata.insert(f.clone(), b"f-meta".to_vec());

        let mut b = CheckedCoreSemanticInputs::default();
        b.metadata.insert(f.clone(), b"f-meta".to_vec());
        b.metadata.insert(g.clone(), b"g-meta".to_vec());
        b.declarations.insert(f.clone(), b"f-body".to_vec());
        b.declarations.insert(g.clone(), b"g-body".to_vec());
        b.symbols.insert(f);
        b.symbols.insert(g);

        assert_eq!(
            canonical_semantic_bytes(&a),
            canonical_semantic_bytes(&b),
            "set/map-like package sections must sort by stable key"
        );
    }

    #[test]
    fn semantic_and_trust_changes_flip_semantic_fingerprint() {
        let f = decl_symbol("f");
        let trust = StableSymbol::assumption(&f, "open-hole");
        let mut base = CheckedCoreSemanticInputs::default();
        base.symbols.insert(f.clone());
        base.declarations.insert(f.clone(), b"body:v1".to_vec());

        let mut body_changed = base.clone();
        body_changed
            .declarations
            .insert(f.clone(), b"body:v2".to_vec());

        let mut trust_changed = base.clone();
        trust_changed
            .trusted_base_delta
            .insert(trust, b"hole still open".to_vec());

        assert_ne!(
            semantic_fingerprint(&base),
            semantic_fingerprint(&body_changed)
        );
        assert_ne!(
            semantic_fingerprint(&base),
            semantic_fingerprint(&trust_changed)
        );
    }

    #[test]
    fn annotations_do_not_change_semantic_fingerprint_but_do_change_artifact() {
        let f = decl_symbol("f");
        let mut semantic = CheckedCoreSemanticInputs::default();
        semantic.symbols.insert(f.clone());
        semantic.declarations.insert(f, b"body:v1".to_vec());

        let mut with_a = CheckedCoreArtifactInputs {
            semantic: semantic.clone(),
            ..CheckedCoreArtifactInputs::default()
        };
        with_a.annotations.insert("display".into(), b"one".to_vec());

        let mut with_b = CheckedCoreArtifactInputs {
            semantic,
            ..CheckedCoreArtifactInputs::default()
        };
        with_b.annotations.insert("display".into(), b"two".to_vec());

        assert_eq!(
            semantic_fingerprint(&with_a.semantic),
            semantic_fingerprint(&with_b.semantic)
        );
        assert_ne!(artifact_fingerprint(&with_a), artifact_fingerprint(&with_b));
    }

    #[test]
    fn obligation_and_dependency_keys_are_stable_symbols() {
        let f = decl_symbol("f");
        let obl = StableSymbol::obligation("f.ensures.0");
        let dep = StableSymbol::new(
            SymbolNamespace::Dependency,
            vec!["dep-pkg".to_string(), "exported".to_string()],
        );
        let mut inputs = CheckedCoreSemanticInputs::default();
        inputs.symbols.insert(f.clone());
        inputs.obligations.insert(obl, b"goal-core".to_vec());
        inputs
            .dependency_semantic_hashes
            .insert(dep, "sha256:abc".to_string());

        let bytes = canonical_semantic_bytes(&inputs);
        assert!(
            !bytes.is_empty(),
            "obligation and dependency sections must have canonical key spaces"
        );
    }

    #[test]
    fn runtime_meaning_metadata_changes_semantic_fingerprint() {
        let add = StableSymbol::primitive("int_add");
        let obligation = StableSymbol::obligation("int_add.totality");
        let assumption = StableSymbol::assumption(&add, "primitive-totality");
        let mut base = CheckedCoreSemanticInputs::default();
        base.symbols.insert(add.clone());
        base.primitive_refs
            .insert(add.clone(), "primitive-registry:int_add".to_string());
        base.primitive_metadata.insert(
            add.clone(),
            PrimitiveMetadata {
                registry_symbol: "int_add".to_string(),
                reduction: PrimitiveReductionMetadata::Op,
                partiality: PartialityMetadata::CheckedPartial { obligation },
                lowerability: LowerabilityStatus::Supported,
            },
        );

        let mut changed = base.clone();
        changed.primitive_metadata.insert(
            add.clone(),
            PrimitiveMetadata {
                registry_symbol: "int_add".to_string(),
                reduction: PrimitiveReductionMetadata::Op,
                partiality: PartialityMetadata::TrustedPartial { assumption },
                lowerability: LowerabilityStatus::Supported,
            },
        );

        assert_ne!(
            semantic_fingerprint(&base),
            semantic_fingerprint(&changed),
            "partiality/trust metadata affects runtime meaning and must enter core_semantic_hash"
        );
    }

    #[test]
    fn lowering_rejects_reachable_unsupported_symbol() {
        let f = decl_symbol("uses_big_int_div");
        let g = decl_symbol("needs_closure_erasure");
        let h = decl_symbol("needs_codegen_feature");
        let i = decl_symbol("blocked_explicit_state");
        let ok = decl_symbol("plain_supported");
        let mut lowerability = BTreeMap::new();
        lowerability.insert(
            f.clone(),
            LowerabilityStatus::Unsupported {
                reason: "native lowering has no checked division trap metadata".to_string(),
            },
        );
        lowerability.insert(
            g.clone(),
            LowerabilityStatus::Deferred {
                later_stage: "NC5-erasure-runtime-ir".to_string(),
                reason: "closure representation is not selected in NC3".to_string(),
            },
        );
        lowerability.insert(
            h.clone(),
            LowerabilityStatus::RequiresFeature {
                feature: "native-ffi-v1".to_string(),
                reason: "FFI lowering must be explicitly enabled".to_string(),
            },
        );
        lowerability.insert(
            i.clone(),
            LowerabilityStatus::Explicit {
                state: "blocked-by-policy".to_string(),
                reason: "policy state has no continue semantics for this consumer".to_string(),
            },
        );
        lowerability.insert(ok.clone(), LowerabilityStatus::Supported);

        ensure_lowerable_for_target([&ok], &lowerability)
            .expect("supported entries must be accepted by the package-side gate");

        let err = ensure_lowerable_for_target([&f], &lowerability).unwrap_err();

        assert_eq!(
            err,
            LoweringReadinessError::Blocked {
                symbol: f,
                status: LowerabilityStatus::Unsupported {
                    reason: "native lowering has no checked division trap metadata".to_string(),
                },
            },
            "reachable unsupported entries must fail loudly before erasure/runtime IR"
        );

        let err = ensure_lowerable_for_target([&g], &lowerability).unwrap_err();

        assert_eq!(
            err,
            LoweringReadinessError::Blocked {
                symbol: g,
                status: LowerabilityStatus::Deferred {
                    later_stage: "NC5-erasure-runtime-ir".to_string(),
                    reason: "closure representation is not selected in NC3".to_string(),
                },
            },
            "deferred entries must fail closed for a target lowerer that is not the named stage"
        );

        let err = ensure_lowerable_for_target([&h], &lowerability).unwrap_err();

        assert_eq!(
            err,
            LoweringReadinessError::Blocked {
                symbol: h,
                status: LowerabilityStatus::RequiresFeature {
                    feature: "native-ffi-v1".to_string(),
                    reason: "FFI lowering must be explicitly enabled".to_string(),
                },
            },
            "requires-feature entries must fail closed unless a versioned feature consumer handles them"
        );

        let err = ensure_lowerable_for_target([&i], &lowerability).unwrap_err();

        assert_eq!(
            err,
            LoweringReadinessError::Blocked {
                symbol: i,
                status: LowerabilityStatus::Explicit {
                    state: "blocked-by-policy".to_string(),
                    reason: "policy state has no continue semantics for this consumer".to_string(),
                },
            },
            "unknown explicit states must not default to supported"
        );

        let missing = decl_symbol("missing_lowerability");
        let err = ensure_lowerable_for_target([&missing], &lowerability).unwrap_err();

        assert_eq!(
            err,
            LoweringReadinessError::MissingLowerability { symbol: missing },
            "missing lowerability metadata must not default to supported"
        );
    }

    #[test]
    fn lowerability_metadata_is_semantic_and_stably_ordered() {
        let f = decl_symbol("f");
        let g = decl_symbol("g");
        let mut a = CheckedCoreSemanticInputs::default();
        a.lowerability
            .insert(g.clone(), LowerabilityStatus::Supported);
        a.lowerability.insert(
            f.clone(),
            LowerabilityStatus::Deferred {
                later_stage: "NC5-erasure-runtime-ir".to_string(),
                reason: "needs erased closure representation".to_string(),
            },
        );

        let mut b = CheckedCoreSemanticInputs::default();
        b.lowerability.insert(
            f.clone(),
            LowerabilityStatus::Deferred {
                later_stage: "NC5-erasure-runtime-ir".to_string(),
                reason: "needs erased closure representation".to_string(),
            },
        );
        b.lowerability
            .insert(g.clone(), LowerabilityStatus::Supported);

        let mut changed = a.clone();
        changed.lowerability.insert(
            f,
            LowerabilityStatus::Unsupported {
                reason: "not lowerable by this compiler stage".to_string(),
            },
        );

        assert_eq!(canonical_semantic_bytes(&a), canonical_semantic_bytes(&b));
        assert_ne!(semantic_fingerprint(&a), semantic_fingerprint(&changed));
    }

    #[test]
    fn acceptance_examples_fit_metadata_without_runtime_layout() {
        let bool_ty = decl_symbol("Bool");
        let false_ctor = StableSymbol::constructor(&bool_ty, "False");
        let true_ctor = StableSymbol::constructor(&bool_ty, "True");
        let nat_ty = decl_symbol("Nat");
        let zero_ctor = StableSymbol::constructor(&nat_ty, "Zero");
        let succ_ctor = StableSymbol::constructor(&nat_ty, "Succ");
        let option_ty = decl_symbol("Option");
        let list_ty = decl_symbol("List");
        let eq_class = decl_symbol("Eq");
        let eq_bool_dict = decl_symbol("EqBoolDict");
        let add_nat = StableSymbol::primitive("nat_add");
        let append_group = decl_symbol("List.append.group");
        let effectful = decl_symbol("print_line");
        let cap = StableSymbol::new(
            SymbolNamespace::Metadata,
            vec!["pkg".to_string(), "ConsoleCap".to_string()],
        );

        let mut inputs = CheckedCoreSemanticInputs::default();
        inputs.data_metadata.insert(
            bool_ty.clone(),
            DataMetadata {
                parameter_count: 0,
                index_count: 0,
                constructors: vec![
                    ConstructorMetadata {
                        symbol: false_ctor,
                        argument_count: 0,
                        target_index_count: 0,
                        recursive_positions: Vec::new(),
                        lowerability: LowerabilityStatus::Supported,
                    },
                    ConstructorMetadata {
                        symbol: true_ctor,
                        argument_count: 0,
                        target_index_count: 0,
                        recursive_positions: Vec::new(),
                        lowerability: LowerabilityStatus::Supported,
                    },
                ],
                eliminator: LowerabilityStatus::Supported,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.data_metadata.insert(
            nat_ty.clone(),
            DataMetadata {
                parameter_count: 0,
                index_count: 0,
                constructors: vec![
                    ConstructorMetadata {
                        symbol: zero_ctor,
                        argument_count: 0,
                        target_index_count: 0,
                        recursive_positions: Vec::new(),
                        lowerability: LowerabilityStatus::Supported,
                    },
                    ConstructorMetadata {
                        symbol: succ_ctor,
                        argument_count: 1,
                        target_index_count: 0,
                        recursive_positions: vec![0],
                        lowerability: LowerabilityStatus::Supported,
                    },
                ],
                eliminator: LowerabilityStatus::Supported,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.data_metadata.insert(
            option_ty,
            DataMetadata {
                parameter_count: 1,
                index_count: 0,
                constructors: Vec::new(),
                eliminator: LowerabilityStatus::Supported,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.data_metadata.insert(
            list_ty,
            DataMetadata {
                parameter_count: 1,
                index_count: 0,
                constructors: Vec::new(),
                eliminator: LowerabilityStatus::Supported,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.class_instance_metadata.insert(
            eq_class.clone(),
            ClassInstanceMetadata {
                kind: ClassInstanceKind::Dictionary,
                class_symbol: Some(eq_class.clone()),
                dictionary_symbol: Some(eq_bool_dict),
                head_symbol: Some(bool_ty),
                field_order: vec!["eq".to_string(), "refl".to_string()],
                law_fields: BTreeSet::from(["refl".to_string()]),
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.primitive_metadata.insert(
            add_nat,
            PrimitiveMetadata {
                registry_symbol: "nat_add".to_string(),
                reduction: PrimitiveReductionMetadata::Op,
                partiality: PartialityMetadata::Total,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.recursion_metadata.insert(
            append_group.clone(),
            RecursionMetadata {
                group_members: vec![append_group],
                admission: RecursionAdmission::AcceptedStructural,
                scc_index: 0,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        inputs.effects_foreign_metadata.insert(
            effectful,
            EffectsForeignMetadata {
                declared_effects: BTreeSet::from(["Console".to_string()]),
                capabilities: BTreeSet::from([cap]),
                foreign_symbol: None,
                boundary: EffectBoundary::Effectful,
                runtime_checks: BTreeSet::new(),
                lowerability: LowerabilityStatus::Supported,
            },
        );

        assert!(!canonical_semantic_bytes(&inputs).is_empty());
    }

    #[test]
    fn emitter_adds_v0_schema_hashes_and_validates_representative_fixtures() {
        let fixtures = representative_checked_core_fixtures().unwrap();

        assert_eq!(fixtures.len(), 1);
        for fixture in fixtures {
            assert_eq!(
                fixture.package.header.version,
                Some(CHECKED_CORE_SCHEMA_VERSION)
            );
            assert_eq!(
                fixture.package.header.package_kind,
                CHECKED_CORE_PACKAGE_KIND
            );
            validate_checked_core_package(&fixture.package).unwrap();
            assert!(!fixture.package.canonical_bytes().is_empty());
        }
    }

    #[test]
    fn validator_rejects_missing_or_unsupported_artifact_versions() {
        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;

        package.header.version = Some(CHECKED_CORE_SCHEMA_VERSION + 1);
        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::UnsupportedVersion {
                found: CHECKED_CORE_SCHEMA_VERSION + 1,
            }
        );

        package.header.version = None;
        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::MissingVersion
        );
    }

    #[test]
    fn emitter_materializes_missing_compiler_metadata_as_unsupported() {
        let f = decl_symbol("stage_gap");
        let mut semantic = CheckedCoreSemanticInputs::default();
        semantic.symbols.insert(f.clone());
        semantic
            .declarations
            .insert(f.clone(), b"checked-core".to_vec());

        let package = emit_checked_core_package(
            fixture_header("stage_gap"),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();

        assert!(
            package.artifact.semantic.unsupported.contains_key(&f),
            "the emitter must materialize a loud unsupported entry, not omit compiler metadata"
        );
        assert!(matches!(
            package.artifact.semantic.lowerability.get(&f),
            Some(LowerabilityStatus::Unsupported { .. })
        ));

        let err = consume_checked_core_package_for_target(&package, [&f]).unwrap_err();
        assert!(
            matches!(err, CheckedCorePackageError::LoweringReadiness(_)),
            "target consumption must fail before erasure/runtime IR when closure reaches the unsupported entry"
        );
    }

    #[test]
    fn package_consumer_uses_artifact_without_surface_source() {
        let mut fixture = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        fixture.artifact.source_identity.clear();
        fixture.artifact.annotations.clear();
        fixture.core_semantic_hash = semantic_fingerprint(&fixture.artifact.semantic);
        fixture.artifact_hash = package_artifact_fingerprint(
            &fixture.header,
            &fixture.artifact,
            fixture.core_semantic_hash,
        );

        let target = StableSymbol::declaration("fixture", &["Core"], "Bool");
        let consumed = consume_checked_core_package_for_target(&fixture, [&target]).unwrap();

        assert_eq!(consumed.core_semantic_hash, fixture.core_semantic_hash);
        assert!(
            consumed.symbols.contains(&target),
            "consume path should read the checked-core artifact symbol table, not surface source"
        );
    }

    #[test]
    fn validator_rejects_semantic_and_artifact_hash_mismatches() {
        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        let bool_ty = StableSymbol::declaration("fixture", &["Core"], "Bool");
        package
            .artifact
            .semantic
            .declarations
            .insert(bool_ty, b"tampered".to_vec());

        assert!(matches!(
            validate_checked_core_package(&package),
            Err(CheckedCorePackageError::SemanticHashMismatch { .. })
        ));

        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        package.artifact.annotations.insert(
            "diagnostic.display".to_string(),
            b"changed envelope only".to_vec(),
        );
        assert!(matches!(
            validate_checked_core_package(&package),
            Err(CheckedCorePackageError::ArtifactHashMismatch { .. })
        ));
    }

    #[test]
    fn validator_rejects_orphan_metadata_after_emission() {
        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        let add = StableSymbol::primitive("nat_add");
        package.artifact.semantic.symbols.remove(&add);
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );

        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::MissingSymbol {
                section: "primitive_refs",
                symbol: add,
            }
        );
    }
}
