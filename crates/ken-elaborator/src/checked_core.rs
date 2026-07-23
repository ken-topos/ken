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
use num_bigint::BigInt;

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

pub fn canonical_symbol_bytes(symbol: &StableSymbol) -> Vec<u8> {
    let mut out = CanonicalSink::new();
    symbol.encode(&mut out);
    out.finish()
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
    pub dependency_declaration_refs: BTreeMap<StableSymbol, StableSymbol>,
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

/// Package-authoritative closure facts needed before exposing body views.
///
/// The producer is expected to fill this from the selected checked-core target
/// closure/report. This type deliberately carries only package and closure
/// identity facts, not source declarations or compiler-driver side tables.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreBodyViewSelection {
    pub package_identity: StableSymbol,
    pub package_core_semantic_hash: u64,
    pub package_artifact_hash: u64,
    pub target_symbol: StableSymbol,
    pub reachable_declarations: BTreeSet<StableSymbol>,
    pub external_symbols: BTreeSet<StableSymbol>,
    pub dependency_semantic_hashes: BTreeMap<StableSymbol, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreBodyView {
    pub package_identity: StableSymbol,
    pub package_core_semantic_hash: u64,
    pub package_artifact_hash: u64,
    pub target_symbol: StableSymbol,
    pub declarations: BTreeMap<StableSymbol, CheckedCoreDeclarationBodyView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreDeclarationBodyView {
    pub symbol: StableSymbol,
    pub level_params: Vec<u64>,
    pub checked_type: Vec<u8>,
    pub body: CheckedCoreBodyTerm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreConstructorView {
    pub symbol: StableSymbol,
    pub family_symbol: StableSymbol,
    pub level_args: Vec<CheckedCoreLevelView>,
    pub family_parameter_count: usize,
    pub family_index_count: usize,
    pub argument_count: usize,
    pub target_index_count: usize,
    pub recursive_positions: Vec<usize>,
    pub constructor_lowerability: LowerabilityStatus,
    pub family_lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreMatchView {
    pub family_symbol: StableSymbol,
    pub level_args: Vec<CheckedCoreLevelView>,
    pub parameters: Vec<Vec<u8>>,
    pub motive: Vec<u8>,
    pub indices: Vec<Vec<u8>>,
    pub scrutinee: Box<CheckedCoreBodyTerm>,
    pub branches: Vec<CheckedCoreMatchBranchView>,
    /// The eliminator motive is classified in `Type`, so recursive induction
    /// hypotheses are computational runtime values rather than erased proofs.
    pub computational_recursive_hypotheses: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreMatchBranchView {
    pub constructor: CheckedCoreConstructorView,
    pub method: CheckedCoreBodyTerm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCorePrimitiveView {
    pub symbol: StableSymbol,
    pub registry_symbol: String,
    pub registry_ref: String,
    pub reduction: PrimitiveReductionMetadata,
    pub partiality: PartialityMetadata,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCorePrimitiveApplicationView {
    pub primitive: CheckedCorePrimitiveView,
    pub arguments: Vec<CheckedCoreBodyTerm>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreRecursiveCallView {
    pub symbol: StableSymbol,
    pub level_args: Vec<CheckedCoreLevelView>,
    pub group_symbol: StableSymbol,
    pub group_members: Vec<StableSymbol>,
    pub admission: RecursionAdmission,
    pub scc_index: usize,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreImportedDeclarationCallView {
    pub symbol: StableSymbol,
    pub level_args: Vec<CheckedCoreLevelView>,
    pub dependency: StableSymbol,
    pub dependency_semantic_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreRecordSigmaView {
    pub symbol: StableSymbol,
    pub kind: RecordSigmaKind,
    pub fields: Vec<CheckedCoreRecordSigmaFieldView>,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreRecordSigmaFieldView {
    pub position: usize,
    pub name: String,
    pub ty: StableSymbol,
    pub runtime: RuntimeFieldStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreRecordSigmaConstructionView {
    pub record: CheckedCoreRecordSigmaView,
    pub fields: Vec<CheckedCoreRecordSigmaFieldValue>,
    pub terminator: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCoreRecordSigmaFieldValue {
    Runtime {
        field: CheckedCoreRecordSigmaFieldView,
        value: Box<CheckedCoreBodyTerm>,
    },
    Erased {
        field: CheckedCoreRecordSigmaFieldView,
        term: Vec<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreRecordSigmaProjectionView {
    pub record: CheckedCoreRecordSigmaView,
    pub field: CheckedCoreRecordSigmaFieldView,
    pub base: Box<CheckedCoreBodyTerm>,
    pub skipped_fields: Vec<CheckedCoreRecordSigmaFieldView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreDictionaryView {
    pub symbol: StableSymbol,
    pub class_symbol: Option<StableSymbol>,
    pub dictionary_symbol: Option<StableSymbol>,
    pub head_symbol: Option<StableSymbol>,
    pub fields: Vec<CheckedCoreDictionaryFieldView>,
    pub lowerability: LowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreDictionaryFieldView {
    pub position: usize,
    pub name: String,
    pub runtime: DictionaryFieldRuntimeStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DictionaryFieldRuntimeStatus {
    Runtime,
    ErasedLawProof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCoreDictionaryConstructionView {
    pub dictionary: CheckedCoreDictionaryView,
    pub fields: Vec<CheckedCoreDictionaryFieldValue>,
    pub terminator: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCoreDictionaryFieldValue {
    Runtime {
        field: CheckedCoreDictionaryFieldView,
        value: Box<CheckedCoreBodyTerm>,
    },
    Erased {
        field: CheckedCoreDictionaryFieldView,
        term: Vec<u8>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCoreBodyTerm {
    Variable {
        de_bruijn_index: usize,
    },
    IntegerLiteral {
        value: BigInt,
    },
    DirectDeclarationCall {
        symbol: StableSymbol,
        level_args: Vec<CheckedCoreLevelView>,
    },
    RecursiveDeclarationCall(CheckedCoreRecursiveCallView),
    ImportedDeclarationCall(CheckedCoreImportedDeclarationCallView),
    PrimitiveLiteral(CheckedCorePrimitiveView),
    PrimitiveApplication(CheckedCorePrimitiveApplicationView),
    ConstructorReference(CheckedCoreConstructorView),
    ErasedConstructorArgument {
        term: Vec<u8>,
    },
    Lambda {
        parameter_type: Vec<u8>,
        body: Box<CheckedCoreBodyTerm>,
    },
    Application {
        function: Box<CheckedCoreBodyTerm>,
        argument: Box<CheckedCoreBodyTerm>,
    },
    Let {
        value_type: Vec<u8>,
        value: Box<CheckedCoreBodyTerm>,
        body: Box<CheckedCoreBodyTerm>,
    },
    Match(CheckedCoreMatchView),
    RecordSigmaConstruction(CheckedCoreRecordSigmaConstructionView),
    RecordSigmaProjection(CheckedCoreRecordSigmaProjectionView),
    DictionaryConstruction(CheckedCoreDictionaryConstructionView),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCoreLevelView {
    Zero,
    Suc(Box<CheckedCoreLevelView>),
    Max(Box<CheckedCoreLevelView>, Box<CheckedCoreLevelView>),
    Var(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckedCoreBodyViewError {
    InvalidPackage(CheckedCorePackageError),
    MismatchedPackageIdentity {
        expected: StableSymbol,
        found: StableSymbol,
    },
    MismatchedCoreSemanticHash {
        expected: u64,
        found: u64,
    },
    MismatchedArtifactHash {
        expected: u64,
        found: u64,
    },
    TargetOutsideSelectedClosure {
        target: StableSymbol,
    },
    RequestedBodyOutsideSelectedClosure {
        target: StableSymbol,
        symbol: StableSymbol,
    },
    MissingDeclarationBody {
        symbol: StableSymbol,
    },
    MismatchedDeclarationSymbol {
        expected: StableSymbol,
        found: StableSymbol,
    },
    UnsupportedDeclarationKind {
        symbol: StableSymbol,
        kind: String,
    },
    UnsupportedTermShape {
        symbol: StableSymbol,
        tag: String,
    },
    StaleConstructorIdentity {
        owner: StableSymbol,
        constructor: StableSymbol,
    },
    MissingMatchBranchData {
        symbol: StableSymbol,
        family: StableSymbol,
        expected: usize,
        found: usize,
    },
    UnsupportedDependentMotive {
        symbol: StableSymbol,
        family: StableSymbol,
    },
    UnsupportedProofOnlyMatch {
        symbol: StableSymbol,
        family: StableSymbol,
    },
    UnsupportedEliminatorShape {
        symbol: StableSymbol,
        family: StableSymbol,
        reason: String,
    },
    UnsupportedDependentFieldShape {
        symbol: StableSymbol,
        reason: String,
    },
    NonExecutableErasedFieldProjection {
        symbol: StableSymbol,
        record: StableSymbol,
        field: String,
    },
    StaleFieldIdentityOrder {
        symbol: StableSymbol,
        record: StableSymbol,
        reason: String,
    },
    UnsupportedRecordProjectionShape {
        symbol: StableSymbol,
        reason: String,
    },
    StalePrimitiveMetadata {
        symbol: StableSymbol,
        primitive: StableSymbol,
        reason: String,
    },
    UnsupportedPrimitiveName {
        symbol: StableSymbol,
        primitive: StableSymbol,
    },
    HostDependentPrimitiveAttempt {
        symbol: StableSymbol,
        primitive: StableSymbol,
        reason: String,
    },
    UnjustifiedPrimitivePartiality {
        symbol: StableSymbol,
        primitive: StableSymbol,
        reason: String,
    },
    UnsupportedRecursiveShape {
        symbol: StableSymbol,
        referenced: StableSymbol,
        reason: String,
    },
    MissingDependencyIdentity {
        owner: StableSymbol,
        referenced: StableSymbol,
    },
    StaleDependencyIdentity {
        owner: StableSymbol,
        referenced: StableSymbol,
        dependency: StableSymbol,
        reason: String,
    },
    NonExecutableDictionaryFieldUse {
        symbol: StableSymbol,
        dictionary: StableSymbol,
        field: String,
    },
    StaleDictionaryFieldSelection {
        symbol: StableSymbol,
        dictionary: StableSymbol,
        reason: String,
    },
    UnjustifiedImpossibleBranch {
        symbol: StableSymbol,
    },
    BodyReferenceOutsideSelectedClosure {
        owner: StableSymbol,
        referenced: StableSymbol,
    },
    BodyReferenceWithoutDeclaration {
        owner: StableSymbol,
        referenced: StableSymbol,
    },
    MalformedCanonicalBytes {
        symbol: StableSymbol,
        reason: String,
    },
    TrailingCanonicalBytes {
        symbol: StableSymbol,
        remaining: usize,
    },
}

impl CheckedCoreBodyViewError {
    pub fn lane(&self) -> &'static str {
        match self {
            CheckedCoreBodyViewError::InvalidPackage(_) => "invalid_checked_core_package",
            CheckedCoreBodyViewError::MismatchedPackageIdentity { .. } => {
                "body_view_package_identity_mismatch"
            }
            CheckedCoreBodyViewError::MismatchedCoreSemanticHash { .. } => {
                "body_view_semantic_hash_mismatch"
            }
            CheckedCoreBodyViewError::MismatchedArtifactHash { .. } => {
                "body_view_artifact_hash_mismatch"
            }
            CheckedCoreBodyViewError::TargetOutsideSelectedClosure { .. } => {
                "target_outside_selected_closure"
            }
            CheckedCoreBodyViewError::RequestedBodyOutsideSelectedClosure { .. } => {
                "body_outside_selected_closure"
            }
            CheckedCoreBodyViewError::MissingDeclarationBody { .. } => {
                "missing_checked_declaration_body"
            }
            CheckedCoreBodyViewError::MismatchedDeclarationSymbol { .. } => {
                "mismatched_checked_declaration_symbol"
            }
            CheckedCoreBodyViewError::UnsupportedDeclarationKind { .. } => {
                "unsupported_checked_declaration_kind"
            }
            CheckedCoreBodyViewError::UnsupportedTermShape { .. } => {
                "unsupported_checked_body_shape"
            }
            CheckedCoreBodyViewError::StaleConstructorIdentity { .. } => {
                "stale_constructor_identity"
            }
            CheckedCoreBodyViewError::MissingMatchBranchData { .. } => "missing_match_branch_data",
            CheckedCoreBodyViewError::UnsupportedDependentMotive { .. } => {
                "unsupported_dependent_motive"
            }
            CheckedCoreBodyViewError::UnsupportedProofOnlyMatch { .. } => {
                "unsupported_proof_only_match"
            }
            CheckedCoreBodyViewError::UnsupportedEliminatorShape { .. } => {
                "unsupported_eliminator_shape"
            }
            CheckedCoreBodyViewError::UnsupportedDependentFieldShape { .. } => {
                "unsupported_dependent_field_shape"
            }
            CheckedCoreBodyViewError::NonExecutableErasedFieldProjection { .. } => {
                "non_executable_erased_field_projection"
            }
            CheckedCoreBodyViewError::StaleFieldIdentityOrder { .. } => {
                "stale_field_identity_order"
            }
            CheckedCoreBodyViewError::UnsupportedRecordProjectionShape { .. } => {
                "unsupported_record_projection_shape"
            }
            CheckedCoreBodyViewError::StalePrimitiveMetadata { .. } => "stale_primitive_metadata",
            CheckedCoreBodyViewError::UnsupportedPrimitiveName { .. } => {
                "unsupported_primitive_name"
            }
            CheckedCoreBodyViewError::HostDependentPrimitiveAttempt { .. } => {
                "host_dependent_primitive_attempt"
            }
            CheckedCoreBodyViewError::UnjustifiedPrimitivePartiality { .. } => {
                "unjustified_primitive_partiality"
            }
            CheckedCoreBodyViewError::UnsupportedRecursiveShape { .. } => {
                "unsupported_recursive_shape"
            }
            CheckedCoreBodyViewError::MissingDependencyIdentity { .. } => {
                "missing_dependency_identity"
            }
            CheckedCoreBodyViewError::StaleDependencyIdentity { .. } => "stale_dependency_identity",
            CheckedCoreBodyViewError::NonExecutableDictionaryFieldUse { .. } => {
                "non_executable_dictionary_field_use"
            }
            CheckedCoreBodyViewError::StaleDictionaryFieldSelection { .. } => {
                "stale_dictionary_field_selection"
            }
            CheckedCoreBodyViewError::UnjustifiedImpossibleBranch { .. } => {
                "unjustified_impossible_branch"
            }
            CheckedCoreBodyViewError::BodyReferenceOutsideSelectedClosure { .. } => {
                "body_reference_outside_selected_closure"
            }
            CheckedCoreBodyViewError::BodyReferenceWithoutDeclaration { .. } => {
                "body_reference_without_declaration"
            }
            CheckedCoreBodyViewError::MalformedCanonicalBytes { .. } => {
                "malformed_checked_declaration_body"
            }
            CheckedCoreBodyViewError::TrailingCanonicalBytes { .. } => {
                "trailing_checked_declaration_body_bytes"
            }
        }
    }
}

impl fmt::Display for CheckedCoreBodyViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckedCoreBodyViewError::InvalidPackage(err) => err.fmt(f),
            CheckedCoreBodyViewError::MismatchedPackageIdentity { expected, found } => {
                write!(
                    f,
                    "body view package identity mismatch: expected {expected}, got {found}"
                )
            }
            CheckedCoreBodyViewError::MismatchedCoreSemanticHash { expected, found } => write!(
                f,
                "body view semantic hash mismatch: expected {expected:#x}, got {found:#x}"
            ),
            CheckedCoreBodyViewError::MismatchedArtifactHash { expected, found } => write!(
                f,
                "body view artifact hash mismatch: expected {expected:#x}, got {found:#x}"
            ),
            CheckedCoreBodyViewError::TargetOutsideSelectedClosure { target } => {
                write!(f, "target {target} is outside the selected closure")
            }
            CheckedCoreBodyViewError::RequestedBodyOutsideSelectedClosure { target, symbol } => {
                write!(
                    f,
                    "requested body {symbol} is outside selected target {target} closure"
                )
            }
            CheckedCoreBodyViewError::MissingDeclarationBody { symbol } => {
                write!(f, "selected declaration {symbol} has no checked body bytes")
            }
            CheckedCoreBodyViewError::MismatchedDeclarationSymbol { expected, found } => write!(
                f,
                "checked declaration bytes identify {found}, expected {expected}"
            ),
            CheckedCoreBodyViewError::UnsupportedDeclarationKind { symbol, kind } => {
                write!(
                    f,
                    "declaration {symbol} has unsupported body-view kind {kind}"
                )
            }
            CheckedCoreBodyViewError::UnsupportedTermShape { symbol, tag } => {
                write!(f, "declaration {symbol} uses unsupported body term {tag}")
            }
            CheckedCoreBodyViewError::StaleConstructorIdentity { owner, constructor } => {
                write!(
                    f,
                    "declaration {owner} references stale constructor identity {constructor}"
                )
            }
            CheckedCoreBodyViewError::MissingMatchBranchData {
                symbol,
                family,
                expected,
                found,
            } => write!(
                f,
                "declaration {symbol} match for {family} has {found} branches, expected {expected}"
            ),
            CheckedCoreBodyViewError::UnsupportedDependentMotive { symbol, family } => write!(
                f,
                "declaration {symbol} uses unsupported dependent motive for {family}"
            ),
            CheckedCoreBodyViewError::UnsupportedProofOnlyMatch { symbol, family } => write!(
                f,
                "declaration {symbol} uses unsupported proof-only match for {family}"
            ),
            CheckedCoreBodyViewError::UnsupportedEliminatorShape {
                symbol,
                family,
                reason,
            } => write!(
                f,
                "declaration {symbol} uses unsupported eliminator shape for {family}: {reason}"
            ),
            CheckedCoreBodyViewError::UnsupportedDependentFieldShape { symbol, reason } => write!(
                f,
                "declaration {symbol} uses unsupported dependent record/Sigma field shape: {reason}"
            ),
            CheckedCoreBodyViewError::NonExecutableErasedFieldProjection {
                symbol,
                record,
                field,
            } => write!(
                f,
                "declaration {symbol} projects erased field {record}.{field} as a runtime value"
            ),
            CheckedCoreBodyViewError::StaleFieldIdentityOrder {
                symbol,
                record,
                reason,
            } => write!(
                f,
                "declaration {symbol} has stale field identity/order for {record}: {reason}"
            ),
            CheckedCoreBodyViewError::UnsupportedRecordProjectionShape { symbol, reason } => {
                write!(
                    f,
                    "declaration {symbol} uses unsupported record/Sigma projection shape: {reason}"
                )
            }
            CheckedCoreBodyViewError::StalePrimitiveMetadata {
                symbol,
                primitive,
                reason,
            } => write!(
                f,
                "declaration {symbol} has stale primitive metadata for {primitive}: {reason}"
            ),
            CheckedCoreBodyViewError::UnsupportedPrimitiveName { symbol, primitive } => write!(
                f,
                "declaration {symbol} references unsupported primitive {primitive}"
            ),
            CheckedCoreBodyViewError::HostDependentPrimitiveAttempt {
                symbol,
                primitive,
                reason,
            } => write!(
                f,
                "declaration {symbol} references host-dependent primitive {primitive}: {reason}"
            ),
            CheckedCoreBodyViewError::UnjustifiedPrimitivePartiality {
                symbol,
                primitive,
                reason,
            } => write!(
                f,
                "declaration {symbol} references primitive {primitive} without justified partiality: {reason}"
            ),
            CheckedCoreBodyViewError::UnsupportedRecursiveShape {
                symbol,
                referenced,
                reason,
            } => write!(
                f,
                "declaration {symbol} references recursive declaration {referenced} through an unsupported shape: {reason}"
            ),
            CheckedCoreBodyViewError::MissingDependencyIdentity { owner, referenced } => write!(
                f,
                "declaration {owner} references imported declaration {referenced} without package dependency identity"
            ),
            CheckedCoreBodyViewError::StaleDependencyIdentity {
                owner,
                referenced,
                dependency,
                reason,
            } => write!(
                f,
                "declaration {owner} references imported declaration {referenced} through stale dependency {dependency}: {reason}"
            ),
            CheckedCoreBodyViewError::NonExecutableDictionaryFieldUse {
                symbol,
                dictionary,
                field,
            } => write!(
                f,
                "declaration {symbol} attempts to expose non-executable dictionary field {dictionary}.{field} as a runtime value"
            ),
            CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
                symbol,
                dictionary,
                reason,
            } => write!(
                f,
                "declaration {symbol} has stale dictionary field selection for {dictionary}: {reason}"
            ),
            CheckedCoreBodyViewError::UnjustifiedImpossibleBranch { symbol } => write!(
                f,
                "declaration {symbol} uses an impossible branch without package evidence"
            ),
            CheckedCoreBodyViewError::BodyReferenceOutsideSelectedClosure { owner, referenced } => {
                write!(
                    f,
                    "declaration {owner} references {referenced} outside selected closure"
                )
            }
            CheckedCoreBodyViewError::BodyReferenceWithoutDeclaration { owner, referenced } => {
                write!(
                    f,
                    "declaration {owner} references {referenced} without a package body"
                )
            }
            CheckedCoreBodyViewError::MalformedCanonicalBytes { symbol, reason } => {
                write!(
                    f,
                    "malformed checked declaration bytes for {symbol}: {reason}"
                )
            }
            CheckedCoreBodyViewError::TrailingCanonicalBytes { symbol, remaining } => write!(
                f,
                "checked declaration bytes for {symbol} have {remaining} trailing bytes"
            ),
        }
    }
}

impl std::error::Error for CheckedCoreBodyViewError {}

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
    DependencySemanticHashesMismatch,
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
            CheckedCorePackageError::DependencySemanticHashesMismatch => write!(
                f,
                "header dependency semantic hashes must match semantic dependency hashes"
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
    pub runtime_fields: BTreeSet<String>,
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
    encode_symbol_map(
        "dependency_declaration_refs",
        &inputs.dependency_declaration_refs,
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
    validate_dependency_hash_lane_coherence(package)?;

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

fn validate_dependency_hash_lane_coherence(
    package: &CheckedCorePackage,
) -> Result<(), CheckedCorePackageError> {
    if package.header.dependency_semantic_hashes
        != package.artifact.semantic.dependency_semantic_hashes
    {
        return Err(CheckedCorePackageError::DependencySemanticHashesMismatch);
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

pub fn checked_core_body_view_for_selection(
    package: &CheckedCorePackage,
    selection: &CheckedCoreBodyViewSelection,
) -> Result<CheckedCoreBodyView, CheckedCoreBodyViewError> {
    let semantic = validate_body_view_selection(package, selection)?;
    let mut declarations = BTreeMap::new();

    for symbol in &selection.reachable_declarations {
        declarations.insert(
            symbol.clone(),
            decode_declaration_body_view(semantic, selection, symbol)?,
        );
    }

    Ok(CheckedCoreBodyView {
        package_identity: package.header.package_identity.clone(),
        package_core_semantic_hash: package.core_semantic_hash,
        package_artifact_hash: package.artifact_hash,
        target_symbol: selection.target_symbol.clone(),
        declarations,
    })
}

pub fn checked_core_declaration_body_view(
    package: &CheckedCorePackage,
    selection: &CheckedCoreBodyViewSelection,
    symbol: &StableSymbol,
) -> Result<CheckedCoreDeclarationBodyView, CheckedCoreBodyViewError> {
    let semantic = validate_body_view_selection(package, selection)?;
    if !selection.reachable_declarations.contains(symbol) {
        return Err(
            CheckedCoreBodyViewError::RequestedBodyOutsideSelectedClosure {
                target: selection.target_symbol.clone(),
                symbol: symbol.clone(),
            },
        );
    }
    decode_declaration_body_view(semantic, selection, symbol)
}

/// Decide whether a checked runtime match actually consumes a computational
/// induction hypothesis. Both checked-plan production and erasure call this
/// exact predicate.
#[derive(Debug)]
pub(crate) struct CheckedComputationalIHClassificationError {
    pub constructor: StableSymbol,
    pub position: usize,
    pub binders: usize,
}

pub(crate) fn checked_match_uses_computational_recursive_hypothesis(
    view: &CheckedCoreMatchView,
) -> Result<bool, CheckedComputationalIHClassificationError> {
    if !view.computational_recursive_hypotheses {
        return Ok(false);
    }
    for branch in &view.branches {
        let recursive_count = branch.constructor.recursive_positions.len();
        if recursive_count == 0 {
            continue;
        }
        let binders = branch.constructor.argument_count + recursive_count;
        let mut body = &branch.method;
        for position in 0..binders {
            let CheckedCoreBodyTerm::Lambda { body: next, .. } = body else {
                return Err(CheckedComputationalIHClassificationError {
                    constructor: branch.constructor.symbol.clone(),
                    position,
                    binders,
                });
            };
            body = next.as_ref();
        }
        if runtime_body_references_outer_binder_range(body, 0, recursive_count, 0) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// One match occurrence in the exact child domain retained by checked Runtime
/// erasure. Opaque type/proof metadata is deliberately absent from this census.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CheckedRuntimeMatchOccurrence {
    pub computational_ordinal: Option<u64>,
}

pub(crate) fn checked_runtime_match_census(
    term: &CheckedCoreBodyTerm,
) -> Result<Vec<CheckedRuntimeMatchOccurrence>, CheckedComputationalIHClassificationError> {
    fn visit(
        term: &CheckedCoreBodyTerm,
        next_computational_ordinal: &mut u64,
        occurrences: &mut Vec<CheckedRuntimeMatchOccurrence>,
    ) -> Result<(), CheckedComputationalIHClassificationError> {
        match term {
            CheckedCoreBodyTerm::Variable { .. }
            | CheckedCoreBodyTerm::IntegerLiteral { .. }
            | CheckedCoreBodyTerm::DirectDeclarationCall { .. }
            | CheckedCoreBodyTerm::RecursiveDeclarationCall(_)
            | CheckedCoreBodyTerm::ImportedDeclarationCall(_)
            | CheckedCoreBodyTerm::PrimitiveLiteral(_)
            | CheckedCoreBodyTerm::ConstructorReference(_)
            | CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => {}
            CheckedCoreBodyTerm::PrimitiveApplication(view) => {
                for argument in &view.arguments {
                    visit(argument, next_computational_ordinal, occurrences)?;
                }
            }
            CheckedCoreBodyTerm::Lambda { body, .. } => {
                visit(body, next_computational_ordinal, occurrences)?;
            }
            CheckedCoreBodyTerm::Application { function, argument } => {
                visit(function, next_computational_ordinal, occurrences)?;
                visit(argument, next_computational_ordinal, occurrences)?;
            }
            CheckedCoreBodyTerm::Let { value, body, .. } => {
                visit(value, next_computational_ordinal, occurrences)?;
                visit(body, next_computational_ordinal, occurrences)?;
            }
            CheckedCoreBodyTerm::Match(view) => {
                let computational = checked_match_uses_computational_recursive_hypothesis(view)?;
                let computational_ordinal = computational.then(|| {
                    let ordinal = *next_computational_ordinal;
                    *next_computational_ordinal = next_computational_ordinal
                        .checked_add(1)
                        .expect("compiler-private computational match ordinal exhausted");
                    ordinal
                });
                occurrences.push(CheckedRuntimeMatchOccurrence {
                    computational_ordinal,
                });
                visit(&view.scrutinee, next_computational_ordinal, occurrences)?;
                for branch in &view.branches {
                    visit(&branch.method, next_computational_ordinal, occurrences)?;
                }
            }
            CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
                for field in &view.fields {
                    if let CheckedCoreRecordSigmaFieldValue::Runtime { value, .. } = field {
                        visit(value, next_computational_ordinal, occurrences)?;
                    }
                }
            }
            CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
                visit(&view.base, next_computational_ordinal, occurrences)?;
            }
            CheckedCoreBodyTerm::DictionaryConstruction(view) => {
                for field in &view.fields {
                    if let CheckedCoreDictionaryFieldValue::Runtime { value, .. } = field {
                        visit(value, next_computational_ordinal, occurrences)?;
                    }
                }
            }
        }
        Ok(())
    }

    let mut occurrences = Vec::new();
    let mut next_computational_ordinal = 0;
    visit(term, &mut next_computational_ordinal, &mut occurrences)?;
    Ok(occurrences)
}

fn runtime_body_references_outer_binder_range(
    term: &CheckedCoreBodyTerm,
    start: usize,
    end: usize,
    local_depth: usize,
) -> bool {
    match term {
        CheckedCoreBodyTerm::Variable { de_bruijn_index } => {
            (local_depth + start..local_depth + end).contains(de_bruijn_index)
        }
        CheckedCoreBodyTerm::IntegerLiteral { .. }
        | CheckedCoreBodyTerm::DirectDeclarationCall { .. }
        | CheckedCoreBodyTerm::RecursiveDeclarationCall(_)
        | CheckedCoreBodyTerm::ImportedDeclarationCall(_)
        | CheckedCoreBodyTerm::PrimitiveLiteral(_)
        | CheckedCoreBodyTerm::ConstructorReference(_)
        | CheckedCoreBodyTerm::ErasedConstructorArgument { .. } => false,
        CheckedCoreBodyTerm::PrimitiveApplication(view) => view.arguments.iter().any(|child| {
            runtime_body_references_outer_binder_range(child, start, end, local_depth)
        }),
        CheckedCoreBodyTerm::Lambda { body, .. } => {
            runtime_body_references_outer_binder_range(body, start, end, local_depth + 1)
        }
        CheckedCoreBodyTerm::Application { function, argument } => {
            runtime_body_references_outer_binder_range(function, start, end, local_depth)
                || runtime_body_references_outer_binder_range(argument, start, end, local_depth)
        }
        CheckedCoreBodyTerm::Let { value, body, .. } => {
            runtime_body_references_outer_binder_range(value, start, end, local_depth)
                || runtime_body_references_outer_binder_range(body, start, end, local_depth + 1)
        }
        CheckedCoreBodyTerm::Match(view) => {
            runtime_body_references_outer_binder_range(&view.scrutinee, start, end, local_depth)
                || view.branches.iter().any(|branch| {
                    runtime_body_references_outer_binder_range(
                        &branch.method,
                        start,
                        end,
                        local_depth,
                    )
                })
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => {
            view.fields.iter().any(|field| match field {
                CheckedCoreRecordSigmaFieldValue::Runtime { value, .. } => {
                    runtime_body_references_outer_binder_range(value, start, end, local_depth)
                }
                CheckedCoreRecordSigmaFieldValue::Erased { .. } => false,
            })
        }
        CheckedCoreBodyTerm::RecordSigmaProjection(view) => {
            runtime_body_references_outer_binder_range(&view.base, start, end, local_depth)
        }
        CheckedCoreBodyTerm::DictionaryConstruction(view) => {
            view.fields.iter().any(|field| match field {
                CheckedCoreDictionaryFieldValue::Runtime { value, .. } => {
                    runtime_body_references_outer_binder_range(value, start, end, local_depth)
                }
                CheckedCoreDictionaryFieldValue::Erased { .. } => false,
            })
        }
    }
}

fn validate_body_view_selection<'a>(
    package: &'a CheckedCorePackage,
    selection: &CheckedCoreBodyViewSelection,
) -> Result<&'a CheckedCoreSemanticInputs, CheckedCoreBodyViewError> {
    validate_checked_core_package(package).map_err(CheckedCoreBodyViewError::InvalidPackage)?;
    if selection.package_identity != package.header.package_identity {
        return Err(CheckedCoreBodyViewError::MismatchedPackageIdentity {
            expected: package.header.package_identity.clone(),
            found: selection.package_identity.clone(),
        });
    }
    if selection.package_core_semantic_hash != package.core_semantic_hash {
        return Err(CheckedCoreBodyViewError::MismatchedCoreSemanticHash {
            expected: package.core_semantic_hash,
            found: selection.package_core_semantic_hash,
        });
    }
    if selection.package_artifact_hash != package.artifact_hash {
        return Err(CheckedCoreBodyViewError::MismatchedArtifactHash {
            expected: package.artifact_hash,
            found: selection.package_artifact_hash,
        });
    }
    if !selection
        .reachable_declarations
        .contains(&selection.target_symbol)
    {
        return Err(CheckedCoreBodyViewError::TargetOutsideSelectedClosure {
            target: selection.target_symbol.clone(),
        });
    }
    Ok(&package.artifact.semantic)
}

fn decode_declaration_body_view(
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    symbol: &StableSymbol,
) -> Result<CheckedCoreDeclarationBodyView, CheckedCoreBodyViewError> {
    let bytes = semantic.declarations.get(symbol).ok_or_else(|| {
        CheckedCoreBodyViewError::MissingDeclarationBody {
            symbol: symbol.clone(),
        }
    })?;
    let mut cursor = CanonicalCursor::new(bytes);
    let kind = cursor
        .read_tag()
        .map_err(|reason| malformed_body(symbol, reason))?;
    if kind != "transparent" {
        return Err(CheckedCoreBodyViewError::UnsupportedDeclarationKind {
            symbol: symbol.clone(),
            kind,
        });
    }

    let encoded_symbol =
        decode_stable_symbol(&mut cursor).map_err(|reason| malformed_body(symbol, reason))?;
    if &encoded_symbol != symbol {
        return Err(CheckedCoreBodyViewError::MismatchedDeclarationSymbol {
            expected: symbol.clone(),
            found: encoded_symbol,
        });
    }
    let level_params =
        decode_level_params(&mut cursor).map_err(|reason| malformed_body(symbol, reason))?;
    let checked_type =
        capture_canonical_term(&mut cursor).map_err(|reason| malformed_body(symbol, reason))?;
    let body = decode_supported_body_term(
        &mut cursor,
        semantic,
        selection,
        symbol,
        &[],
        Some(&checked_type),
    )?;
    if cursor.remaining() != 0 {
        return Err(CheckedCoreBodyViewError::TrailingCanonicalBytes {
            symbol: symbol.clone(),
            remaining: cursor.remaining(),
        });
    }

    Ok(CheckedCoreDeclarationBodyView {
        symbol: symbol.clone(),
        level_params,
        checked_type,
        body,
    })
}

fn malformed_body(symbol: &StableSymbol, reason: String) -> CheckedCoreBodyViewError {
    CheckedCoreBodyViewError::MalformedCanonicalBytes {
        symbol: symbol.clone(),
        reason,
    }
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
    symbols.extend(semantic.dependency_declaration_refs.keys().cloned());
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
    for (declaration, dependency) in &semantic.dependency_declaration_refs {
        refs.push(("dependency_declaration_refs", declaration.clone()));
        refs.push(("dependency_declaration_refs.dependency", dependency.clone()));
    }
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
            runtime_fields: BTreeSet::from(["eq".to_string()]),
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
            name,
            level_params,
            ty,
        } => {
            out.tag("opaque");
            encode_global(*id, symbols, out)?;
            out.str(name);
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
        Term::IntLit(value) => {
            out.tag("int_lit");
            out.bytes(&value.to_signed_bytes_be());
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

fn encode_symbol_map(
    tag: &'static str,
    map: &BTreeMap<StableSymbol, StableSymbol>,
    out: &mut CanonicalSink,
) {
    out.tag(tag);
    out.seq_len(map.len());
    for (symbol, value) in map {
        symbol.encode(out);
        value.encode(out);
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
        out.tag("runtime_fields");
        out.seq_len(meta.runtime_fields.len());
        for field in &meta.runtime_fields {
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

struct CanonicalCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> CanonicalCursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    fn read_tag(&mut self) -> Result<String, String> {
        self.read_str()
    }

    fn expect_tag(&mut self, expected: &'static str) -> Result<(), String> {
        let found = self.read_tag()?;
        if found == expected {
            Ok(())
        } else {
            Err(format!("expected tag {expected:?}, found {found:?}"))
        }
    }

    fn read_str(&mut self) -> Result<String, String> {
        let len = self.read_len()?;
        let bytes = self.read_exact(len)?;
        let value =
            std::str::from_utf8(bytes).map_err(|err| format!("invalid UTF-8 string: {err}"))?;
        Ok(value.to_string())
    }

    fn read_u64(&mut self) -> Result<u64, String> {
        let bytes = self.read_exact(8)?;
        let mut value = [0_u8; 8];
        value.copy_from_slice(bytes);
        Ok(u64::from_be_bytes(value))
    }

    fn read_len(&mut self) -> Result<usize, String> {
        let raw = self.read_u64()?;
        usize::try_from(raw).map_err(|_| format!("length {raw} does not fit usize"))
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| format!("canonical byte offset overflow at {}", self.pos))?;
        if end > self.bytes.len() {
            return Err(format!(
                "unexpected end of canonical bytes: need {len}, have {}",
                self.remaining()
            ));
        }
        let bytes = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }
}

fn decode_stable_symbol(cursor: &mut CanonicalCursor<'_>) -> Result<StableSymbol, String> {
    cursor.expect_tag("symbol")?;
    let namespace = match cursor.read_str()?.as_str() {
        "decl" => SymbolNamespace::Declaration,
        "ctor" => SymbolNamespace::Constructor,
        "prim" => SymbolNamespace::Primitive,
        "module" => SymbolNamespace::Module,
        "meta" => SymbolNamespace::Metadata,
        "obl" => SymbolNamespace::Obligation,
        "assume" => SymbolNamespace::Assumption,
        "dep" => SymbolNamespace::Dependency,
        "unsupported" => SymbolNamespace::Unsupported,
        other => return Err(format!("unknown stable symbol namespace {other:?}")),
    };
    let len = cursor.read_len()?;
    let mut components = Vec::with_capacity(len);
    for _ in 0..len {
        components.push(cursor.read_str()?);
    }
    Ok(StableSymbol {
        namespace,
        components,
    })
}

fn decode_level_params(cursor: &mut CanonicalCursor<'_>) -> Result<Vec<u64>, String> {
    cursor.expect_tag("level_params")?;
    let len = cursor.read_len()?;
    let mut params = Vec::with_capacity(len);
    for _ in 0..len {
        params.push(cursor.read_u64()?);
    }
    Ok(params)
}

fn capture_canonical_term(cursor: &mut CanonicalCursor<'_>) -> Result<Vec<u8>, String> {
    let start = cursor.pos;
    skip_term(cursor)?;
    Ok(cursor.bytes[start..cursor.pos].to_vec())
}

fn decode_supported_body_term(
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
    expected_type: Option<&[u8]>,
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    let tag = cursor
        .read_tag()
        .map_err(|reason| malformed_body(owner, reason))?;
    decode_supported_body_term_after_tag(
        tag,
        cursor,
        semantic,
        selection,
        owner,
        type_context,
        expected_type,
    )
}

fn decode_supported_body_term_after_tag(
    tag: String,
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
    expected_type: Option<&[u8]>,
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    match tag.as_str() {
        "var" => {
            let raw = cursor
                .read_u64()
                .map_err(|reason| malformed_body(owner, reason))?;
            let de_bruijn_index = usize::try_from(raw).map_err(|_| {
                malformed_body(owner, format!("variable index {raw} does not fit usize"))
            })?;
            Ok(CheckedCoreBodyTerm::Variable { de_bruijn_index })
        }
        "int_lit" => {
            let len = cursor
                .read_len()
                .map_err(|reason| malformed_body(owner, reason))?;
            let bytes = cursor
                .read_exact(len)
                .map_err(|reason| malformed_body(owner, reason))?;
            let value = BigInt::from_signed_bytes_be(bytes);
            Ok(CheckedCoreBodyTerm::IntegerLiteral { value })
        }
        "const" => {
            let symbol =
                decode_stable_symbol(cursor).map_err(|reason| malformed_body(owner, reason))?;
            let level_args =
                decode_levels(cursor).map_err(|reason| malformed_body(owner, reason))?;
            if semantic.primitive_metadata.contains_key(&symbol)
                || semantic.primitive_refs.contains_key(&symbol)
                || symbol.namespace == SymbolNamespace::Primitive
            {
                reject_primitive_level_args(owner, &symbol, &level_args)?;
                let primitive = checked_primitive_view(semantic, owner, &symbol)?;
                return match primitive.reduction {
                    PrimitiveReductionMetadata::Literal => {
                        Ok(CheckedCoreBodyTerm::PrimitiveLiteral(primitive))
                    }
                    PrimitiveReductionMetadata::Op => {
                        Ok(CheckedCoreBodyTerm::PrimitiveApplication(
                            CheckedCorePrimitiveApplicationView {
                                primitive,
                                arguments: Vec::new(),
                            },
                        ))
                    }
                    PrimitiveReductionMetadata::OpaqueType => {
                        Err(CheckedCoreBodyViewError::UnsupportedPrimitiveName {
                            symbol: owner.clone(),
                            primitive: symbol,
                        })
                    }
                };
            }
            if &symbol == owner {
                return checked_recursive_call_view(semantic, selection, owner, symbol, level_args)
                    .map(CheckedCoreBodyTerm::RecursiveDeclarationCall);
            }
            if is_same_recursive_group_reference(semantic, owner, &symbol) {
                return checked_recursive_call_view(semantic, selection, owner, symbol, level_args)
                    .map(CheckedCoreBodyTerm::RecursiveDeclarationCall);
            }
            if !selection.reachable_declarations.contains(&symbol) {
                if selection.external_symbols.contains(&symbol) {
                    return checked_imported_declaration_call_view(
                        semantic, selection, owner, symbol, level_args,
                    )
                    .map(CheckedCoreBodyTerm::ImportedDeclarationCall);
                }
                return Err(
                    CheckedCoreBodyViewError::BodyReferenceOutsideSelectedClosure {
                        owner: owner.clone(),
                        referenced: symbol,
                    },
                );
            }
            if !semantic.declarations.contains_key(&symbol) {
                return Err(CheckedCoreBodyViewError::BodyReferenceWithoutDeclaration {
                    owner: owner.clone(),
                    referenced: symbol,
                });
            }
            Ok(CheckedCoreBodyTerm::DirectDeclarationCall { symbol, level_args })
        }
        "constructor_ref" => {
            let symbol =
                decode_stable_symbol(cursor).map_err(|reason| malformed_body(owner, reason))?;
            let level_args =
                decode_levels(cursor).map_err(|reason| malformed_body(owner, reason))?;
            Ok(CheckedCoreBodyTerm::ConstructorReference(
                checked_constructor_view(semantic, owner, &symbol, level_args)?,
            ))
        }
        "elim" => decode_supported_match_view(cursor, semantic, selection, owner, type_context),
        "lam" => {
            let parameter_type =
                capture_canonical_term(cursor).map_err(|reason| malformed_body(owner, reason))?;
            let body_expected = expected_type
                .map(canonical_pi_codomain)
                .transpose()
                .map_err(|reason| malformed_body(owner, reason))?
                .flatten();
            let mut inner_context = Vec::with_capacity(type_context.len() + 1);
            inner_context.push(parameter_type.clone());
            inner_context.extend_from_slice(type_context);
            let body = Box::new(decode_supported_body_term(
                cursor,
                semantic,
                selection,
                owner,
                &inner_context,
                body_expected.as_deref(),
            )?);
            Ok(CheckedCoreBodyTerm::Lambda {
                parameter_type,
                body,
            })
        }
        "app" => {
            let function = Box::new(decode_supported_body_term(
                cursor,
                semantic,
                selection,
                owner,
                type_context,
                None,
            )?);
            let argument = if constructor_spine_needs_erased_family_argument(&function) {
                Box::new(CheckedCoreBodyTerm::ErasedConstructorArgument {
                    term: capture_canonical_term(cursor)
                        .map_err(|reason| malformed_body(owner, reason))?,
                })
            } else {
                Box::new(decode_supported_body_term(
                    cursor,
                    semantic,
                    selection,
                    owner,
                    type_context,
                    None,
                )?)
            };
            if let CheckedCoreBodyTerm::PrimitiveApplication(mut view) = *function {
                view.arguments.push(*argument);
                return Ok(CheckedCoreBodyTerm::PrimitiveApplication(view));
            }
            Ok(CheckedCoreBodyTerm::Application { function, argument })
        }
        "let" => {
            let value_type =
                capture_canonical_term(cursor).map_err(|reason| malformed_body(owner, reason))?;
            let value = Box::new(decode_supported_body_term(
                cursor,
                semantic,
                selection,
                owner,
                type_context,
                Some(&value_type),
            )?);
            let mut inner_context = Vec::with_capacity(type_context.len() + 1);
            inner_context.push(value_type.clone());
            inner_context.extend_from_slice(type_context);
            let body = Box::new(decode_supported_body_term(
                cursor,
                semantic,
                selection,
                owner,
                &inner_context,
                expected_type,
            )?);
            Ok(CheckedCoreBodyTerm::Let {
                value_type,
                value,
                body,
            })
        }
        "absurd" => Err(CheckedCoreBodyViewError::UnjustifiedImpossibleBranch {
            symbol: owner.clone(),
        }),
        "pair" => decode_supported_pair_construction(
            cursor,
            semantic,
            selection,
            owner,
            type_context,
            expected_type,
        ),
        "proj1" | "proj2" => decode_supported_record_sigma_projection(
            tag,
            cursor,
            semantic,
            selection,
            owner,
            type_context,
        ),
        _ => Err(CheckedCoreBodyViewError::UnsupportedTermShape {
            symbol: owner.clone(),
            tag,
        }),
    }
}

fn constructor_spine_needs_erased_family_argument(term: &CheckedCoreBodyTerm) -> bool {
    let mut applied_args = 0usize;
    let mut current = term;
    while let CheckedCoreBodyTerm::Application { function, .. } = current {
        applied_args += 1;
        current = function;
    }
    let CheckedCoreBodyTerm::ConstructorReference(constructor) = current else {
        return false;
    };
    applied_args < constructor.family_parameter_count
}

fn is_same_recursive_group_reference(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    referenced: &StableSymbol,
) -> bool {
    semantic.recursion_metadata.values().any(|meta| {
        meta.group_members.iter().any(|member| member == owner)
            && meta.group_members.iter().any(|member| member == referenced)
    })
}

fn checked_recursive_call_view(
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    referenced: StableSymbol,
    level_args: Vec<CheckedCoreLevelView>,
) -> Result<CheckedCoreRecursiveCallView, CheckedCoreBodyViewError> {
    if !selection.reachable_declarations.contains(&referenced) {
        return Err(
            CheckedCoreBodyViewError::BodyReferenceOutsideSelectedClosure {
                owner: owner.clone(),
                referenced,
            },
        );
    }
    if !semantic.declarations.contains_key(&referenced) {
        return Err(CheckedCoreBodyViewError::BodyReferenceWithoutDeclaration {
            owner: owner.clone(),
            referenced,
        });
    }
    let (group_symbol, meta) = semantic
        .recursion_metadata
        .iter()
        .find(|(_, meta)| {
            meta.group_members.iter().any(|member| member == owner)
                && meta
                    .group_members
                    .iter()
                    .any(|member| member == &referenced)
        })
        .ok_or_else(|| CheckedCoreBodyViewError::UnsupportedRecursiveShape {
            symbol: owner.clone(),
            referenced: referenced.clone(),
            reason: "missing recursive-group metadata for declaration reference".to_string(),
        })?;
    match meta.admission {
        RecursionAdmission::AcceptedStructural | RecursionAdmission::AcceptedSizeChange => {}
        RecursionAdmission::NonRecursive | RecursionAdmission::Rejected => {
            return Err(CheckedCoreBodyViewError::UnsupportedRecursiveShape {
                symbol: owner.clone(),
                referenced,
                reason: format!("recursive-group admission is {:?}", meta.admission),
            });
        }
    }
    if meta.lowerability.blocks_lowering() {
        return Err(CheckedCoreBodyViewError::UnsupportedRecursiveShape {
            symbol: owner.clone(),
            referenced,
            reason: format!(
                "recursive-group lowerability blocks runtime use: {:?}",
                meta.lowerability
            ),
        });
    }
    Ok(CheckedCoreRecursiveCallView {
        symbol: referenced,
        level_args,
        group_symbol: group_symbol.clone(),
        group_members: meta.group_members.clone(),
        admission: meta.admission.clone(),
        scc_index: meta.scc_index,
        lowerability: meta.lowerability.clone(),
    })
}

fn checked_imported_declaration_call_view(
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    referenced: StableSymbol,
    level_args: Vec<CheckedCoreLevelView>,
) -> Result<CheckedCoreImportedDeclarationCallView, CheckedCoreBodyViewError> {
    let dependency = semantic
        .dependency_declaration_refs
        .get(&referenced)
        .cloned()
        .ok_or_else(|| CheckedCoreBodyViewError::MissingDependencyIdentity {
            owner: owner.clone(),
            referenced: referenced.clone(),
        })?;
    let semantic_hash = semantic
        .dependency_semantic_hashes
        .get(&dependency)
        .ok_or_else(|| CheckedCoreBodyViewError::StaleDependencyIdentity {
            owner: owner.clone(),
            referenced: referenced.clone(),
            dependency: dependency.clone(),
            reason: "dependency declaration ref names a dependency without semantic hash"
                .to_string(),
        })?;
    let selection_hash = selection
        .dependency_semantic_hashes
        .get(&dependency)
        .ok_or_else(|| CheckedCoreBodyViewError::StaleDependencyIdentity {
            owner: owner.clone(),
            referenced: referenced.clone(),
            dependency: dependency.clone(),
            reason: "selected closure omits dependency semantic hash".to_string(),
        })?;
    if selection_hash != semantic_hash {
        return Err(CheckedCoreBodyViewError::StaleDependencyIdentity {
            owner: owner.clone(),
            referenced: referenced.clone(),
            dependency,
            reason: format!(
                "selected dependency hash {selection_hash:?} does not match package hash {semantic_hash:?}"
            ),
        });
    }
    Ok(CheckedCoreImportedDeclarationCallView {
        symbol: referenced,
        level_args,
        dependency,
        dependency_semantic_hash: semantic_hash.clone(),
    })
}

fn checked_primitive_view(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    symbol: &StableSymbol,
) -> Result<CheckedCorePrimitiveView, CheckedCoreBodyViewError> {
    let Some(meta) = semantic.primitive_metadata.get(symbol) else {
        return Err(CheckedCoreBodyViewError::UnsupportedPrimitiveName {
            symbol: owner.clone(),
            primitive: symbol.clone(),
        });
    };
    let registry_ref = semantic.primitive_refs.get(symbol).ok_or_else(|| {
        CheckedCoreBodyViewError::StalePrimitiveMetadata {
            symbol: owner.clone(),
            primitive: symbol.clone(),
            reason: "missing primitive_refs entry".to_string(),
        }
    })?;
    let expected_ref = format!("primitive-registry:{}", meta.registry_symbol);
    if registry_ref != &expected_ref {
        return Err(CheckedCoreBodyViewError::StalePrimitiveMetadata {
            symbol: owner.clone(),
            primitive: symbol.clone(),
            reason: format!(
                "primitive_refs entry {registry_ref:?} does not match {expected_ref:?}"
            ),
        });
    }
    if symbol.namespace != SymbolNamespace::Primitive
        || symbol.components.len() != 1
        || symbol.components[0] != meta.registry_symbol
    {
        return Err(CheckedCoreBodyViewError::StalePrimitiveMetadata {
            symbol: owner.clone(),
            primitive: symbol.clone(),
            reason: format!(
                "stable primitive identity must be prim:{}",
                meta.registry_symbol
            ),
        });
    }
    reject_unsupported_primitive_lowerability(owner, symbol, &meta.lowerability)?;
    justify_primitive_partiality(semantic, owner, symbol, &meta.partiality)?;

    Ok(CheckedCorePrimitiveView {
        symbol: symbol.clone(),
        registry_symbol: meta.registry_symbol.clone(),
        registry_ref: registry_ref.clone(),
        reduction: meta.reduction.clone(),
        partiality: meta.partiality.clone(),
        lowerability: meta.lowerability.clone(),
    })
}

fn reject_primitive_level_args(
    owner: &StableSymbol,
    primitive: &StableSymbol,
    level_args: &[CheckedCoreLevelView],
) -> Result<(), CheckedCoreBodyViewError> {
    if level_args.is_empty() {
        Ok(())
    } else {
        Err(CheckedCoreBodyViewError::StalePrimitiveMetadata {
            symbol: owner.clone(),
            primitive: primitive.clone(),
            reason: "primitive body-view terms do not support level arguments".to_string(),
        })
    }
}

fn reject_unsupported_primitive_lowerability(
    owner: &StableSymbol,
    primitive: &StableSymbol,
    lowerability: &LowerabilityStatus,
) -> Result<(), CheckedCoreBodyViewError> {
    match lowerability {
        LowerabilityStatus::Supported => Ok(()),
        LowerabilityStatus::RequiresFeature { feature, reason }
            if feature.contains("host") || reason.contains("host") =>
        {
            Err(CheckedCoreBodyViewError::HostDependentPrimitiveAttempt {
                symbol: owner.clone(),
                primitive: primitive.clone(),
                reason: reason.clone(),
            })
        }
        LowerabilityStatus::Unsupported { .. }
        | LowerabilityStatus::Deferred { .. }
        | LowerabilityStatus::RequiresFeature { .. }
        | LowerabilityStatus::Explicit { .. } => {
            Err(CheckedCoreBodyViewError::UnsupportedPrimitiveName {
                symbol: owner.clone(),
                primitive: primitive.clone(),
            })
        }
    }
}

fn justify_primitive_partiality(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    primitive: &StableSymbol,
    partiality: &PartialityMetadata,
) -> Result<(), CheckedCoreBodyViewError> {
    match partiality {
        PartialityMetadata::Total => Ok(()),
        PartialityMetadata::CheckedPartial { obligation } => {
            if semantic.obligations.contains_key(obligation)
                && semantic.obligation_metadata.contains_key(obligation)
            {
                Ok(())
            } else {
                Err(CheckedCoreBodyViewError::UnjustifiedPrimitivePartiality {
                    symbol: owner.clone(),
                    primitive: primitive.clone(),
                    reason: format!("missing checked-partial obligation {obligation}"),
                })
            }
        }
        PartialityMetadata::TrustedPartial { assumption } => {
            if semantic.assumptions.contains_key(assumption)
                && semantic.assumption_trust_metadata.contains_key(assumption)
            {
                Ok(())
            } else {
                Err(CheckedCoreBodyViewError::UnjustifiedPrimitivePartiality {
                    symbol: owner.clone(),
                    primitive: primitive.clone(),
                    reason: format!("missing trusted-partial assumption {assumption}"),
                })
            }
        }
    }
}

fn decode_supported_record_sigma_construction(
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
    expected_type: Option<&[u8]>,
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    let Some(expected_type) = expected_type else {
        return Err(CheckedCoreBodyViewError::StaleFieldIdentityOrder {
            symbol: owner.clone(),
            record: owner.clone(),
            reason: "pair body has no package-owned expected record/Sigma type".to_string(),
        });
    };
    if type_has_dependent_sigma(expected_type).map_err(|reason| malformed_body(owner, reason))? {
        return Err(CheckedCoreBodyViewError::UnsupportedDependentFieldShape {
            symbol: owner.clone(),
            reason: "expected type contains a dependent Sigma field".to_string(),
        });
    }

    let record_symbol = record_head_symbol_from_type(expected_type)
        .map_err(|reason| malformed_body(owner, reason))?
        .ok_or_else(|| CheckedCoreBodyViewError::StaleFieldIdentityOrder {
            symbol: owner.clone(),
            record: owner.clone(),
            reason: "expected type has no stable record/Sigma metadata head".to_string(),
        })?;
    let record = checked_record_sigma_view(semantic, owner, &record_symbol)?;
    if record.fields.is_empty() {
        return Err(CheckedCoreBodyViewError::StaleFieldIdentityOrder {
            symbol: owner.clone(),
            record: record.symbol,
            reason: "pair body cannot construct a zero-field record/Sigma".to_string(),
        });
    }

    let mut fields = Vec::with_capacity(record.fields.len());
    for (index, field) in record.fields.iter().enumerate() {
        if index > 0 {
            let tag = cursor
                .read_tag()
                .map_err(|reason| malformed_body(owner, reason))?;
            if tag != "pair" {
                return Err(CheckedCoreBodyViewError::StaleFieldIdentityOrder {
                    symbol: owner.clone(),
                    record: record.symbol.clone(),
                    reason: format!("field {} expected nested pair, found {tag:?}", field.name),
                });
            }
        }

        match field.runtime {
            RuntimeFieldStatus::Runtime => {
                let value = Box::new(decode_supported_body_term(
                    cursor,
                    semantic,
                    selection,
                    owner,
                    type_context,
                    None,
                )?);
                fields.push(CheckedCoreRecordSigmaFieldValue::Runtime {
                    field: field.clone(),
                    value,
                });
            }
            RuntimeFieldStatus::ErasedLaw | RuntimeFieldStatus::ErasedProof => {
                let term = capture_canonical_term(cursor)
                    .map_err(|reason| malformed_body(owner, reason))?;
                fields.push(CheckedCoreRecordSigmaFieldValue::Erased {
                    field: field.clone(),
                    term,
                });
            }
        }
    }

    let terminator_start = cursor.pos;
    let terminator_tag = cursor
        .read_tag()
        .map_err(|reason| malformed_body(owner, reason))?;
    cursor.pos = terminator_start;
    if terminator_tag == "pair" {
        return Err(CheckedCoreBodyViewError::StaleFieldIdentityOrder {
            symbol: owner.clone(),
            record: record.symbol.clone(),
            reason: "pair body carries more fields than package metadata".to_string(),
        });
    }
    let terminator =
        capture_canonical_term(cursor).map_err(|reason| malformed_body(owner, reason))?;

    Ok(CheckedCoreBodyTerm::RecordSigmaConstruction(
        CheckedCoreRecordSigmaConstructionView {
            record,
            fields,
            terminator,
        },
    ))
}

fn decode_supported_record_sigma_projection(
    first_tag: String,
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    if first_tag == "proj2" {
        return Err(CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
            symbol: owner.clone(),
            reason: "bare proj2 exposes a record/Sigma tail, not an executable field".to_string(),
        });
    }

    let mut skipped_count = 0usize;
    let mut base_tag = cursor
        .read_tag()
        .map_err(|reason| malformed_body(owner, reason))?;
    while base_tag == "proj2" {
        skipped_count += 1;
        base_tag = cursor
            .read_tag()
            .map_err(|reason| malformed_body(owner, reason))?;
    }

    let base = decode_supported_body_term_after_tag(
        base_tag,
        cursor,
        semantic,
        selection,
        owner,
        type_context,
        None,
    )?;
    let record_symbol = record_symbol_for_projection_base(semantic, owner, &base, type_context)?;
    let record = checked_record_sigma_view(semantic, owner, &record_symbol)?;
    let field = record.fields.get(skipped_count).cloned().ok_or_else(|| {
        CheckedCoreBodyViewError::StaleFieldIdentityOrder {
            symbol: owner.clone(),
            record: record.symbol.clone(),
            reason: format!(
                "projection skipped {skipped_count} fields but metadata has {}",
                record.fields.len()
            ),
        }
    })?;
    if field.runtime != RuntimeFieldStatus::Runtime {
        return Err(
            CheckedCoreBodyViewError::NonExecutableErasedFieldProjection {
                symbol: owner.clone(),
                record: record.symbol,
                field: field.name,
            },
        );
    }
    let skipped_fields = record.fields[..skipped_count].to_vec();

    Ok(CheckedCoreBodyTerm::RecordSigmaProjection(
        CheckedCoreRecordSigmaProjectionView {
            record,
            field,
            base: Box::new(base),
            skipped_fields,
        },
    ))
}

fn checked_record_sigma_view(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    symbol: &StableSymbol,
) -> Result<CheckedCoreRecordSigmaView, CheckedCoreBodyViewError> {
    let meta = semantic.record_sigma_metadata.get(symbol).ok_or_else(|| {
        CheckedCoreBodyViewError::StaleFieldIdentityOrder {
            symbol: owner.clone(),
            record: symbol.clone(),
            reason: "missing package record/Sigma metadata".to_string(),
        }
    })?;
    if meta.lowerability.blocks_lowering() {
        return Err(CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
            symbol: owner.clone(),
            reason: format!("record/Sigma {symbol} lowerability blocks runtime use"),
        });
    }

    Ok(CheckedCoreRecordSigmaView {
        symbol: symbol.clone(),
        kind: meta.kind.clone(),
        fields: meta
            .fields
            .iter()
            .enumerate()
            .map(|(position, field)| CheckedCoreRecordSigmaFieldView {
                position,
                name: field.name.clone(),
                ty: field.ty.clone(),
                runtime: field.runtime.clone(),
            })
            .collect(),
        lowerability: meta.lowerability.clone(),
    })
}

fn checked_dictionary_view(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    symbol: &StableSymbol,
) -> Result<CheckedCoreDictionaryView, CheckedCoreBodyViewError> {
    let meta = semantic
        .class_instance_metadata
        .get(symbol)
        .ok_or_else(|| CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
            symbol: owner.clone(),
            dictionary: symbol.clone(),
            reason: "missing package dictionary metadata".to_string(),
        })?;
    if meta.kind != ClassInstanceKind::Dictionary {
        return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
            symbol: owner.clone(),
            dictionary: symbol.clone(),
            reason: "class/instance metadata entry is not a dictionary".to_string(),
        });
    }
    if meta.lowerability.blocks_lowering() {
        return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
            symbol: owner.clone(),
            dictionary: symbol.clone(),
            reason: format!(
                "dictionary lowerability blocks runtime use: {:?}",
                meta.lowerability
            ),
        });
    }

    let mut fields = Vec::with_capacity(meta.field_order.len());
    for (position, name) in meta.field_order.iter().enumerate() {
        if meta.runtime_fields.contains(name) && meta.law_fields.contains(name) {
            return Err(CheckedCoreBodyViewError::NonExecutableDictionaryFieldUse {
                symbol: owner.clone(),
                dictionary: symbol.clone(),
                field: name.clone(),
            });
        }
        let runtime = if meta.runtime_fields.contains(name) {
            DictionaryFieldRuntimeStatus::Runtime
        } else {
            DictionaryFieldRuntimeStatus::ErasedLawProof
        };
        fields.push(CheckedCoreDictionaryFieldView {
            position,
            name: name.clone(),
            runtime,
        });
    }

    for field in meta.runtime_fields.union(&meta.law_fields) {
        if !meta.field_order.iter().any(|candidate| candidate == field) {
            return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
                symbol: owner.clone(),
                dictionary: symbol.clone(),
                reason: format!("field classification {field:?} is absent from field_order"),
            });
        }
    }

    Ok(CheckedCoreDictionaryView {
        symbol: symbol.clone(),
        class_symbol: meta.class_symbol.clone(),
        dictionary_symbol: meta.dictionary_symbol.clone(),
        head_symbol: meta.head_symbol.clone(),
        fields,
        lowerability: meta.lowerability.clone(),
    })
}

fn record_symbol_for_projection_base(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    base: &CheckedCoreBodyTerm,
    type_context: &[Vec<u8>],
) -> Result<StableSymbol, CheckedCoreBodyViewError> {
    match base {
        CheckedCoreBodyTerm::Variable { de_bruijn_index } => {
            let ty = type_context.get(*de_bruijn_index).ok_or_else(|| {
                CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
                    symbol: owner.clone(),
                    reason: format!(
                        "projection base variable {de_bruijn_index} has no checked type context"
                    ),
                }
            })?;
            record_head_symbol_from_type(ty)
                .map_err(|reason| malformed_body(owner, reason))?
                .ok_or_else(
                    || CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
                        symbol: owner.clone(),
                        reason: "projection base variable is not record/Sigma typed".to_string(),
                    },
                )
        }
        CheckedCoreBodyTerm::DirectDeclarationCall { symbol, .. } => {
            let ty = declaration_checked_type_bytes(semantic, symbol).map_err(|reason| {
                CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
                    symbol: owner.clone(),
                    reason,
                }
            })?;
            record_head_symbol_from_type(&ty)
                .map_err(|reason| malformed_body(owner, reason))?
                .ok_or_else(
                    || CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
                        symbol: owner.clone(),
                        reason: format!("direct call {symbol} is not record/Sigma typed"),
                    },
                )
        }
        CheckedCoreBodyTerm::RecordSigmaConstruction(view) => Ok(view.record.symbol.clone()),
        _ => Err(CheckedCoreBodyViewError::UnsupportedRecordProjectionShape {
            symbol: owner.clone(),
            reason: "projection base shape has no package-owned record/Sigma type".to_string(),
        }),
    }
}

fn declaration_checked_type_bytes(
    semantic: &CheckedCoreSemanticInputs,
    symbol: &StableSymbol,
) -> Result<Vec<u8>, String> {
    let bytes = semantic
        .declarations
        .get(symbol)
        .ok_or_else(|| format!("projection base {symbol} has no checked declaration bytes"))?;
    let mut cursor = CanonicalCursor::new(bytes);
    let kind = cursor.read_tag()?;
    if kind != "transparent" {
        return Err(format!(
            "projection base {symbol} has unsupported declaration kind {kind}"
        ));
    }
    let encoded_symbol = decode_stable_symbol(&mut cursor)?;
    if &encoded_symbol != symbol {
        return Err(format!(
            "projection base declaration bytes identify {encoded_symbol}, expected {symbol}"
        ));
    }
    decode_level_params(&mut cursor)?;
    capture_canonical_term(&mut cursor)
}

fn decode_supported_pair_construction(
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
    expected_type: Option<&[u8]>,
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    if let Some(expected_type) = expected_type {
        if let Some(symbol) = record_head_symbol_from_type(expected_type)
            .map_err(|reason| malformed_body(owner, reason))?
        {
            if semantic.class_instance_metadata.contains_key(&symbol) {
                return decode_supported_dictionary_construction(
                    cursor,
                    semantic,
                    selection,
                    owner,
                    type_context,
                    expected_type,
                    symbol,
                );
            }
        }
    }
    decode_supported_record_sigma_construction(
        cursor,
        semantic,
        selection,
        owner,
        type_context,
        expected_type,
    )
}

fn decode_supported_dictionary_construction(
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
    expected_type: &[u8],
    dictionary_symbol: StableSymbol,
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    if type_has_dependent_sigma(expected_type).map_err(|reason| malformed_body(owner, reason))? {
        return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
            symbol: owner.clone(),
            dictionary: dictionary_symbol,
            reason: "dictionary expected type contains a dependent Sigma field".to_string(),
        });
    }
    let dictionary = checked_dictionary_view(semantic, owner, &dictionary_symbol)?;
    if dictionary.fields.is_empty() {
        return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
            symbol: owner.clone(),
            dictionary: dictionary.symbol,
            reason: "dictionary body cannot construct a zero-field dictionary".to_string(),
        });
    }

    let mut fields = Vec::with_capacity(dictionary.fields.len());
    for (index, field) in dictionary.fields.iter().enumerate() {
        if index > 0 {
            let tag = cursor
                .read_tag()
                .map_err(|reason| malformed_body(owner, reason))?;
            if tag != "pair" {
                return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
                    symbol: owner.clone(),
                    dictionary: dictionary.symbol.clone(),
                    reason: format!("field {} expected nested pair, found {tag:?}", field.name),
                });
            }
        }

        match field.runtime {
            DictionaryFieldRuntimeStatus::Runtime => {
                let value = Box::new(decode_supported_body_term(
                    cursor,
                    semantic,
                    selection,
                    owner,
                    type_context,
                    None,
                )?);
                fields.push(CheckedCoreDictionaryFieldValue::Runtime {
                    field: field.clone(),
                    value,
                });
            }
            DictionaryFieldRuntimeStatus::ErasedLawProof => {
                let term = capture_canonical_term(cursor)
                    .map_err(|reason| malformed_body(owner, reason))?;
                fields.push(CheckedCoreDictionaryFieldValue::Erased {
                    field: field.clone(),
                    term,
                });
            }
        }
    }

    let terminator_start = cursor.pos;
    let terminator_tag = cursor
        .read_tag()
        .map_err(|reason| malformed_body(owner, reason))?;
    cursor.pos = terminator_start;
    if terminator_tag == "pair" {
        return Err(CheckedCoreBodyViewError::StaleDictionaryFieldSelection {
            symbol: owner.clone(),
            dictionary: dictionary.symbol.clone(),
            reason: "dictionary body carries more fields than package metadata".to_string(),
        });
    }
    let terminator =
        capture_canonical_term(cursor).map_err(|reason| malformed_body(owner, reason))?;

    Ok(CheckedCoreBodyTerm::DictionaryConstruction(
        CheckedCoreDictionaryConstructionView {
            dictionary,
            fields,
            terminator,
        },
    ))
}

fn canonical_pi_codomain(bytes: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut cursor = CanonicalCursor::new(bytes);
    if cursor.read_tag()?.as_str() != "pi" {
        return Ok(None);
    }
    skip_term(&mut cursor)?;
    let codomain = capture_canonical_term(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(format!(
            "function type bytes have {} trailing bytes",
            cursor.remaining()
        ));
    }
    Ok(Some(codomain))
}

fn record_head_symbol_from_type(bytes: &[u8]) -> Result<Option<StableSymbol>, String> {
    let mut cursor = CanonicalCursor::new(bytes);
    let symbol = record_head_symbol_from_type_cursor(&mut cursor)?;
    Ok(symbol)
}

fn record_head_symbol_from_type_cursor(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<Option<StableSymbol>, String> {
    match cursor.read_tag()?.as_str() {
        "const" | "ind_former" => {
            let symbol = decode_stable_symbol(cursor)?;
            skip_levels(cursor)?;
            Ok(Some(symbol))
        }
        "app" => {
            let symbol = record_head_symbol_from_type_cursor(cursor)?;
            skip_term(cursor)?;
            Ok(symbol)
        }
        _ => Ok(None),
    }
}

fn type_has_dependent_sigma(bytes: &[u8]) -> Result<bool, String> {
    let mut cursor = CanonicalCursor::new(bytes);
    let found = term_has_dependent_sigma(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(format!(
            "type bytes have {} trailing bytes",
            cursor.remaining()
        ));
    }
    Ok(found)
}

fn term_has_dependent_sigma(cursor: &mut CanonicalCursor<'_>) -> Result<bool, String> {
    match cursor.read_tag()?.as_str() {
        "sigma" => {
            if term_has_dependent_sigma(cursor)? {
                return Ok(true);
            }
            let codomain_start = cursor.pos;
            if term_contains_free_var(cursor, 0)? {
                return Ok(true);
            }
            let codomain = &cursor.bytes[codomain_start..cursor.pos];
            canonical_term_contains_free_var(codomain, 0)
        }
        "pi" | "lam" => {
            let left = term_has_dependent_sigma(cursor)?;
            let right = term_has_dependent_sigma(cursor)?;
            Ok(left || right)
        }
        "app" | "pair" | "ascript" | "quot" | "absurd" => {
            let left = term_has_dependent_sigma(cursor)?;
            let right = term_has_dependent_sigma(cursor)?;
            Ok(left || right)
        }
        "proj1" | "proj2" | "refl" | "quot_class" | "trunc" | "trunc_proj" => {
            term_has_dependent_sigma(cursor)
        }
        "let" | "eq" | "j" => {
            let a = term_has_dependent_sigma(cursor)?;
            let b = term_has_dependent_sigma(cursor)?;
            let c = term_has_dependent_sigma(cursor)?;
            Ok(a || b || c)
        }
        "cast" | "quot_elim" => {
            let a = term_has_dependent_sigma(cursor)?;
            let b = term_has_dependent_sigma(cursor)?;
            let c = term_has_dependent_sigma(cursor)?;
            let d = term_has_dependent_sigma(cursor)?;
            Ok(a || b || c || d)
        }
        "type" | "omega" => {
            skip_level(cursor)?;
            Ok(false)
        }
        "var" => {
            cursor.read_u64()?;
            Ok(false)
        }
        "const" | "ind_former" | "constructor_ref" => {
            decode_stable_symbol(cursor)?;
            skip_levels(cursor)?;
            Ok(false)
        }
        "elim" => {
            decode_stable_symbol(cursor)?;
            skip_levels(cursor)?;
            if terms_have_dependent_sigma(cursor)? {
                return Ok(true);
            }
            if term_has_dependent_sigma(cursor)? {
                return Ok(true);
            }
            if terms_have_dependent_sigma(cursor)? {
                return Ok(true);
            }
            if terms_have_dependent_sigma(cursor)? {
                return Ok(true);
            }
            term_has_dependent_sigma(cursor)
        }
        other => Err(format!("unsupported term tag {other:?}")),
    }
}

fn terms_have_dependent_sigma(cursor: &mut CanonicalCursor<'_>) -> Result<bool, String> {
    let len = cursor.read_len()?;
    for _ in 0..len {
        if term_has_dependent_sigma(cursor)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn decode_supported_match_view(
    cursor: &mut CanonicalCursor<'_>,
    semantic: &CheckedCoreSemanticInputs,
    selection: &CheckedCoreBodyViewSelection,
    owner: &StableSymbol,
    type_context: &[Vec<u8>],
) -> Result<CheckedCoreBodyTerm, CheckedCoreBodyViewError> {
    let family_symbol =
        decode_stable_symbol(cursor).map_err(|reason| malformed_body(owner, reason))?;
    let data = semantic.data_metadata.get(&family_symbol).ok_or_else(|| {
        CheckedCoreBodyViewError::UnsupportedEliminatorShape {
            symbol: owner.clone(),
            family: family_symbol.clone(),
            reason: "missing package data metadata".to_string(),
        }
    })?;
    if data.eliminator.blocks_lowering() {
        return Err(CheckedCoreBodyViewError::UnsupportedEliminatorShape {
            symbol: owner.clone(),
            family: family_symbol,
            reason: "eliminator lowerability blocks runtime use".to_string(),
        });
    }

    let level_args = decode_levels(cursor).map_err(|reason| malformed_body(owner, reason))?;
    let parameters =
        capture_canonical_terms(cursor).map_err(|reason| malformed_body(owner, reason))?;
    if parameters.len() != data.parameter_count {
        return Err(CheckedCoreBodyViewError::UnsupportedEliminatorShape {
            symbol: owner.clone(),
            family: family_symbol,
            reason: format!(
                "expected {} family parameters, got {}",
                data.parameter_count,
                parameters.len()
            ),
        });
    }

    let motive = capture_canonical_term(cursor).map_err(|reason| malformed_body(owner, reason))?;
    let computational_recursive_hypotheses =
        validate_supported_match_motive(semantic, owner, &family_symbol, data, &motive)?;

    let method_count = cursor
        .read_len()
        .map_err(|reason| malformed_body(owner, reason))?;
    if method_count != data.constructors.len() {
        return Err(CheckedCoreBodyViewError::MissingMatchBranchData {
            symbol: owner.clone(),
            family: family_symbol,
            expected: data.constructors.len(),
            found: method_count,
        });
    }
    let mut branches = Vec::with_capacity(method_count);
    for ctor in &data.constructors {
        let constructor =
            checked_constructor_view_from_metadata(&family_symbol, data, ctor, level_args.clone())
                .map_err(
                    |constructor| CheckedCoreBodyViewError::StaleConstructorIdentity {
                        owner: owner.clone(),
                        constructor,
                    },
                )?;
        let method =
            decode_supported_body_term(cursor, semantic, selection, owner, type_context, None)?;
        branches.push(CheckedCoreMatchBranchView {
            constructor,
            method,
        });
    }

    let indices =
        capture_canonical_terms(cursor).map_err(|reason| malformed_body(owner, reason))?;
    if indices.len() != data.index_count {
        return Err(CheckedCoreBodyViewError::UnsupportedEliminatorShape {
            symbol: owner.clone(),
            family: family_symbol,
            reason: format!(
                "expected {} family indices, got {}",
                data.index_count,
                indices.len()
            ),
        });
    }
    if data.index_count != 0 {
        return Err(CheckedCoreBodyViewError::UnsupportedDependentMotive {
            symbol: owner.clone(),
            family: family_symbol,
        });
    }

    let scrutinee = Box::new(decode_supported_body_term(
        cursor,
        semantic,
        selection,
        owner,
        type_context,
        None,
    )?);
    Ok(CheckedCoreBodyTerm::Match(CheckedCoreMatchView {
        family_symbol,
        level_args,
        parameters,
        motive,
        indices,
        scrutinee,
        branches,
        computational_recursive_hypotheses,
    }))
}

fn checked_constructor_view(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    symbol: &StableSymbol,
    level_args: Vec<CheckedCoreLevelView>,
) -> Result<CheckedCoreConstructorView, CheckedCoreBodyViewError> {
    for (family_symbol, data) in &semantic.data_metadata {
        for ctor in &data.constructors {
            if &ctor.symbol == symbol {
                return checked_constructor_view_from_metadata(
                    family_symbol,
                    data,
                    ctor,
                    level_args,
                )
                .map_err(|constructor| {
                    CheckedCoreBodyViewError::StaleConstructorIdentity {
                        owner: owner.clone(),
                        constructor,
                    }
                });
            }
        }
    }
    Err(CheckedCoreBodyViewError::StaleConstructorIdentity {
        owner: owner.clone(),
        constructor: symbol.clone(),
    })
}

fn checked_constructor_view_from_metadata(
    family_symbol: &StableSymbol,
    data: &DataMetadata,
    ctor: &ConstructorMetadata,
    level_args: Vec<CheckedCoreLevelView>,
) -> Result<CheckedCoreConstructorView, StableSymbol> {
    if ctor.symbol.namespace != SymbolNamespace::Constructor
        || !constructor_parent_matches_family(&ctor.symbol, family_symbol)
    {
        return Err(ctor.symbol.clone());
    }
    Ok(CheckedCoreConstructorView {
        symbol: ctor.symbol.clone(),
        family_symbol: family_symbol.clone(),
        level_args,
        family_parameter_count: data.parameter_count,
        family_index_count: data.index_count,
        argument_count: ctor.argument_count,
        target_index_count: ctor.target_index_count,
        recursive_positions: ctor.recursive_positions.clone(),
        constructor_lowerability: ctor.lowerability.clone(),
        family_lowerability: data.lowerability.clone(),
    })
}

fn constructor_parent_matches_family(constructor: &StableSymbol, family: &StableSymbol) -> bool {
    constructor.components.len() == family.components.len() + 1
        && constructor.components[..family.components.len()] == family.components[..]
}

fn validate_supported_match_motive(
    semantic: &CheckedCoreSemanticInputs,
    owner: &StableSymbol,
    family: &StableSymbol,
    data: &DataMetadata,
    motive: &[u8],
) -> Result<bool, CheckedCoreBodyViewError> {
    if data.index_count != 0 {
        return Err(CheckedCoreBodyViewError::UnsupportedDependentMotive {
            symbol: owner.clone(),
            family: family.clone(),
        });
    }
    match inspect_non_dependent_motive(motive).map_err(|reason| malformed_body(owner, reason))? {
        MotiveShape::ConstantType => Ok(true),
        MotiveShape::Dependent
            if semantic
                .metadata
                .values()
                .any(|bytes| bytes.starts_with(b"HostEffectSpineV1\0")) =>
        {
            Ok(true)
        }
        MotiveShape::Dependent => Err(CheckedCoreBodyViewError::UnsupportedDependentMotive {
            symbol: owner.clone(),
            family: family.clone(),
        }),
        MotiveShape::ProofOnly => Err(CheckedCoreBodyViewError::UnsupportedProofOnlyMatch {
            symbol: owner.clone(),
            family: family.clone(),
        }),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MotiveShape {
    ConstantType,
    Dependent,
    ProofOnly,
}

fn inspect_non_dependent_motive(motive: &[u8]) -> Result<MotiveShape, String> {
    let mut cursor = CanonicalCursor::new(motive);
    if cursor.read_tag()?.as_str() != "ascript" {
        return Ok(MotiveShape::Dependent);
    }

    if cursor.read_tag()?.as_str() != "lam" {
        return Ok(MotiveShape::Dependent);
    }
    skip_term(&mut cursor)?;
    let body_start = cursor.pos;
    skip_term(&mut cursor)?;
    let body = &cursor.bytes[body_start..cursor.pos];
    if canonical_term_contains_free_var(body, 0)? {
        return Ok(MotiveShape::Dependent);
    }

    let sort = inspect_non_indexed_motive_type_sort(&mut cursor)?;
    if cursor.remaining() != 0 {
        return Err(format!(
            "motive bytes have {} trailing bytes",
            cursor.remaining()
        ));
    }
    Ok(sort)
}

/// Return the exact checked result type of a constant, non-indexed Match
/// motive.  Native join planning consumes this checked fact before erasure;
/// erased Runtime syntax is never used to reconstruct it.
pub(crate) fn checked_constant_motive_result_type(
    motive: &[u8],
) -> Result<Option<Vec<u8>>, String> {
    let mut cursor = CanonicalCursor::new(motive);
    if cursor.read_tag()?.as_str() != "ascript" || cursor.read_tag()?.as_str() != "lam" {
        return Ok(None);
    }
    skip_term(&mut cursor)?;
    let body = capture_canonical_term(&mut cursor)?;
    if canonical_term_contains_free_var(&body, 0)? {
        return Ok(None);
    }
    if inspect_non_indexed_motive_type_sort(&mut cursor)? != MotiveShape::ConstantType
        || cursor.remaining() != 0
    {
        return Ok(None);
    }
    Ok(Some(body))
}

/// Exact stable head identity of one canonical checked type, when its head is
/// a constant/inductive former (possibly applied to arguments).
pub(crate) fn checked_type_head_symbol(
    checked_type: &[u8],
) -> Result<Option<StableSymbol>, String> {
    record_head_symbol_from_type(checked_type)
}

fn inspect_non_indexed_motive_type_sort(
    cursor: &mut CanonicalCursor<'_>,
) -> Result<MotiveShape, String> {
    if cursor.read_tag()?.as_str() != "pi" {
        return Ok(MotiveShape::Dependent);
    }
    skip_term(cursor)?;
    match cursor.read_tag()?.as_str() {
        "type" => {
            skip_level(cursor)?;
            Ok(MotiveShape::ConstantType)
        }
        "omega" => {
            skip_level(cursor)?;
            Ok(MotiveShape::ProofOnly)
        }
        _ => Ok(MotiveShape::Dependent),
    }
}

fn canonical_term_contains_free_var(bytes: &[u8], target: usize) -> Result<bool, String> {
    let mut cursor = CanonicalCursor::new(bytes);
    let contains = term_contains_free_var(&mut cursor, target)?;
    if cursor.remaining() != 0 {
        return Err(format!(
            "term bytes have {} trailing bytes",
            cursor.remaining()
        ));
    }
    Ok(contains)
}

fn term_contains_free_var(cursor: &mut CanonicalCursor<'_>, target: usize) -> Result<bool, String> {
    match cursor.read_tag()?.as_str() {
        "type" | "omega" => {
            skip_level(cursor)?;
            Ok(false)
        }
        "var" => {
            let raw = cursor.read_u64()?;
            let index = usize::try_from(raw)
                .map_err(|_| format!("variable index {raw} does not fit usize"))?;
            Ok(index == target)
        }
        "const" | "ind_former" | "constructor_ref" => {
            decode_stable_symbol(cursor)?;
            skip_levels(cursor)?;
            Ok(false)
        }
        "elim" => {
            decode_stable_symbol(cursor)?;
            skip_levels(cursor)?;
            if terms_contain_free_var(cursor, target)? {
                return Ok(true);
            }
            if term_contains_free_var(cursor, target)? {
                return Ok(true);
            }
            if terms_contain_free_var(cursor, target)? {
                return Ok(true);
            }
            if terms_contain_free_var(cursor, target)? {
                return Ok(true);
            }
            term_contains_free_var(cursor, target)
        }
        "pi" | "sigma" => {
            let left = term_contains_free_var(cursor, target)?;
            let right = term_contains_free_var(cursor, target + 1)?;
            Ok(left || right)
        }
        "lam" => {
            let parameter = term_contains_free_var(cursor, target)?;
            let body = term_contains_free_var(cursor, target + 1)?;
            Ok(parameter || body)
        }
        "app" | "pair" | "ascript" | "quot" | "absurd" => {
            let left = term_contains_free_var(cursor, target)?;
            let right = term_contains_free_var(cursor, target)?;
            Ok(left || right)
        }
        "proj1" | "proj2" | "refl" | "quot_class" | "trunc" | "trunc_proj" => {
            term_contains_free_var(cursor, target)
        }
        "let" => {
            let ty = term_contains_free_var(cursor, target)?;
            let value = term_contains_free_var(cursor, target)?;
            let body = term_contains_free_var(cursor, target + 1)?;
            Ok(ty || value || body)
        }
        "eq" | "j" => {
            let a = term_contains_free_var(cursor, target)?;
            let b = term_contains_free_var(cursor, target)?;
            let c = term_contains_free_var(cursor, target)?;
            Ok(a || b || c)
        }
        "cast" | "quot_elim" => {
            let a = term_contains_free_var(cursor, target)?;
            let b = term_contains_free_var(cursor, target)?;
            let c = term_contains_free_var(cursor, target)?;
            let d = term_contains_free_var(cursor, target)?;
            Ok(a || b || c || d)
        }
        other => Err(format!("unsupported term tag {other:?}")),
    }
}

fn terms_contain_free_var(cursor: &mut CanonicalCursor<'_>, target: usize) -> Result<bool, String> {
    let len = cursor.read_len()?;
    for _ in 0..len {
        if term_contains_free_var(cursor, target)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn decode_levels(cursor: &mut CanonicalCursor<'_>) -> Result<Vec<CheckedCoreLevelView>, String> {
    let len = cursor.read_len()?;
    let mut levels = Vec::with_capacity(len);
    for _ in 0..len {
        levels.push(decode_level_view(cursor)?);
    }
    Ok(levels)
}

fn decode_level_view(cursor: &mut CanonicalCursor<'_>) -> Result<CheckedCoreLevelView, String> {
    match cursor.read_tag()?.as_str() {
        "level_zero" => Ok(CheckedCoreLevelView::Zero),
        "level_suc" => Ok(CheckedCoreLevelView::Suc(Box::new(decode_level_view(
            cursor,
        )?))),
        "level_max" => Ok(CheckedCoreLevelView::Max(
            Box::new(decode_level_view(cursor)?),
            Box::new(decode_level_view(cursor)?),
        )),
        "level_var" => Ok(CheckedCoreLevelView::Var(cursor.read_u64()?)),
        other => Err(format!("unsupported level tag {other:?}")),
    }
}

fn skip_term(cursor: &mut CanonicalCursor<'_>) -> Result<(), String> {
    match cursor.read_tag()?.as_str() {
        "type" | "omega" => skip_level(cursor),
        "var" => {
            cursor.read_u64()?;
            Ok(())
        }
        "int_lit" => {
            let len = cursor.read_len()?;
            cursor.read_exact(len)?;
            Ok(())
        }
        "const" | "ind_former" | "constructor_ref" => {
            decode_stable_symbol(cursor)?;
            skip_levels(cursor)
        }
        "elim" => {
            decode_stable_symbol(cursor)?;
            skip_levels(cursor)?;
            skip_terms(cursor)?;
            skip_term(cursor)?;
            skip_terms(cursor)?;
            skip_terms(cursor)?;
            skip_term(cursor)
        }
        "pi" | "lam" | "app" | "sigma" | "pair" | "ascript" | "quot" | "absurd" => {
            skip_term(cursor)?;
            skip_term(cursor)
        }
        "proj1" | "proj2" | "refl" | "quot_class" | "trunc" | "trunc_proj" => skip_term(cursor),
        "let" | "eq" | "j" => {
            skip_term(cursor)?;
            skip_term(cursor)?;
            skip_term(cursor)
        }
        "cast" | "quot_elim" => {
            skip_term(cursor)?;
            skip_term(cursor)?;
            skip_term(cursor)?;
            skip_term(cursor)
        }
        other => Err(format!("unsupported term tag {other:?}")),
    }
}

fn skip_terms(cursor: &mut CanonicalCursor<'_>) -> Result<(), String> {
    let len = cursor.read_len()?;
    for _ in 0..len {
        skip_term(cursor)?;
    }
    Ok(())
}

fn capture_canonical_terms(cursor: &mut CanonicalCursor<'_>) -> Result<Vec<Vec<u8>>, String> {
    let len = cursor.read_len()?;
    let mut terms = Vec::with_capacity(len);
    for _ in 0..len {
        terms.push(capture_canonical_term(cursor)?);
    }
    Ok(terms)
}

fn skip_levels(cursor: &mut CanonicalCursor<'_>) -> Result<(), String> {
    let len = cursor.read_len()?;
    for _ in 0..len {
        skip_level(cursor)?;
    }
    Ok(())
}

fn skip_level(cursor: &mut CanonicalCursor<'_>) -> Result<(), String> {
    match cursor.read_tag()?.as_str() {
        "level_zero" => Ok(()),
        "level_suc" => skip_level(cursor),
        "level_max" => {
            skip_level(cursor)?;
            skip_level(cursor)
        }
        "level_var" => {
            cursor.read_u64()?;
            Ok(())
        }
        other => Err(format!("unsupported level tag {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use ken_kernel::env::{Decl, PrimReduction};
    use ken_kernel::{GlobalId, Level, LevelVar, Term};
    use num_bigint::BigInt;

    use super::*;

    /// Decode an `int_lit`-tagged canonical value back to its `BigInt`
    /// (`encode_term`'s `Term::IntLit` arm's exact inverse — length-prefixed
    /// `to_signed_bytes_be`, decoded via `from_signed_bytes_be`). Test-only:
    /// nothing in the shipped semantic view decoders consumes an `IntLit`
    /// yet (deciding what it MEANS to a checked-core-view consumer is a
    /// separate, later design question) — this exists solely to prove the
    /// syntactic round-trip itself is lossless, per the Architect's gate.
    fn decode_int_lit(cursor: &mut CanonicalCursor<'_>) -> Result<BigInt, String> {
        cursor.expect_tag("int_lit")?;
        let len = cursor.read_len()?;
        let bytes = cursor.read_exact(len)?;
        Ok(BigInt::from_signed_bytes_be(bytes))
    }

    /// `encode_term ∘ decode_int_lit = id` on `Term::IntLit` — the property
    /// that makes the serialization arm trustworthy, not just present. A
    /// wrong round-trip here would silently corrupt a literal's value on
    /// checked-core import (`Equal Int 5 5` reimporting as `Equal Int 5
    /// <other>`), which no other test in this WP would catch (the kernel
    /// tests never touch `checked_core.rs`; this file's own tests are the
    /// only place this specific encode/decode pair is exercised).
    #[test]
    fn int_lit_encode_decode_round_trips() {
        let table = StableSymbolTable::new();
        let values: Vec<BigInt> = vec![
            BigInt::from(0),
            BigInt::from(-1),
            BigInt::from(1),
            BigInt::from(5),
            BigInt::from(-123456789i64),
            BigInt::from(i64::MAX),
            BigInt::from(i64::MIN),
            "9".repeat(300).parse().unwrap(),
            format!("-{}", "1".repeat(300)).parse().unwrap(),
        ];
        for n in values {
            let mut out = CanonicalSink::new();
            encode_term(&Term::IntLit(n.clone()), &table, &mut out)
                .expect("encode_term must not fail on IntLit");
            let bytes = out.finish();

            let mut cursor = CanonicalCursor::new(&bytes);
            let decoded = decode_int_lit(&mut cursor).expect("decode_int_lit must not fail");
            assert_eq!(
                decoded, n,
                "round-trip must preserve the exact BigInt value"
            );
            assert_eq!(
                cursor.remaining(),
                0,
                "decode must consume exactly the bytes encode_term wrote, no more no less"
            );
        }
    }

    /// `skip_term` (the shared low-level cursor-advance primitive every
    /// other decoder builds on) must also consume exactly the `int_lit`
    /// payload's bytes — not too few (misparses whatever follows) and not
    /// too many (silently swallows sibling content in a larger term).
    #[test]
    fn int_lit_skip_term_consumes_exactly_its_own_bytes() {
        let table = StableSymbolTable::new();
        // Two IntLits back to back — skip_term on the first must land the
        // cursor exactly at the second's tag, not before or after it.
        let mut out = CanonicalSink::new();
        encode_term(&Term::IntLit(BigInt::from(42)), &table, &mut out).unwrap();
        let first_len = out.finish().len();
        let mut out = CanonicalSink::new();
        encode_term(&Term::IntLit(BigInt::from(42)), &table, &mut out).unwrap();
        encode_term(&Term::IntLit(BigInt::from(-7)), &table, &mut out).unwrap();
        let bytes = out.finish();

        let mut cursor = CanonicalCursor::new(&bytes);
        skip_term(&mut cursor).expect("skip_term must succeed on int_lit");
        assert_eq!(
            cursor.pos, first_len,
            "skip_term must stop exactly at the boundary between the two literals"
        );
        let second = decode_int_lit(&mut cursor).expect("second literal must decode cleanly");
        assert_eq!(second, BigInt::from(-7));
        assert_eq!(cursor.remaining(), 0);
    }

    fn decl_symbol(name: &str) -> StableSymbol {
        StableSymbol::declaration("pkg", &["M"], name)
    }

    fn table(id: GlobalId, symbol: StableSymbol) -> StableSymbolTable {
        let mut table = StableSymbolTable::new();
        table.insert_global(id, symbol);
        table
    }

    fn table_many(entries: &[(GlobalId, StableSymbol)]) -> StableSymbolTable {
        let mut table = StableSymbolTable::new();
        for (id, symbol) in entries {
            table.insert_global(*id, symbol.clone());
        }
        table
    }

    fn body_view_header() -> CheckedCorePackageHeader {
        CheckedCorePackageHeader::v0(
            "checked-core-body-view-test",
            "ken-kernel:test",
            "docs/program/wp/NC13-checked-core-body-view.md",
            "spec/10-kernel/18a-primitive-registry.md:test",
            StableSymbol::new(SymbolNamespace::Module, vec!["body_view_pkg".to_string()]),
        )
    }

    fn body_view_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
        let target = decl_symbol("target");
        let helper = decl_symbol("helper");
        let target_id = GlobalId(1);
        let helper_id = GlobalId(2);
        let table = table_many(&[(target_id, target.clone()), (helper_id, helper.clone())]);
        let ty = Term::Type(Level::zero());
        let helper_decl = Decl::Transparent {
            id: helper_id,
            level_params: Vec::new(),
            ty: ty.clone(),
            body: Term::Lam(Box::new(ty.clone()), Box::new(Term::Var(0))),
        };
        let target_decl = Decl::Transparent {
            id: target_id,
            level_params: Vec::new(),
            ty: ty.clone(),
            body: Term::Lam(
                Box::new(ty.clone()),
                Box::new(Term::Let {
                    ty: Box::new(ty),
                    val: Box::new(Term::App(
                        Box::new(Term::Const {
                            id: helper_id,
                            level_args: Vec::new(),
                        }),
                        Box::new(Term::Var(0)),
                    )),
                    body: Box::new(Term::Var(0)),
                }),
            ),
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        semantic.symbols.insert(target.clone());
        semantic.symbols.insert(helper.clone());
        semantic
            .lowerability
            .insert(target.clone(), LowerabilityStatus::Supported);
        semantic
            .lowerability
            .insert(helper.clone(), LowerabilityStatus::Supported);
        semantic.declarations.insert(
            target.clone(),
            canonical_decl_bytes(&target_decl, &table).unwrap(),
        );
        semantic.declarations.insert(
            helper.clone(),
            canonical_decl_bytes(&helper_decl, &table).unwrap(),
        );
        let package = emit_checked_core_package(
            body_view_header(),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, helper)
    }

    fn refresh_package_hashes(package: &mut CheckedCorePackage) {
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );
    }

    fn bool_data_symbols() -> (StableSymbol, StableSymbol, StableSymbol) {
        let bool_ty = decl_symbol("Bool");
        let false_ctor = StableSymbol::constructor(&bool_ty, "False");
        let true_ctor = StableSymbol::constructor(&bool_ty, "True");
        (bool_ty, false_ctor, true_ctor)
    }

    fn constant_type_motive(scrut_ty: Term, ret_ty: Term) -> Term {
        Term::Ascript(
            Box::new(Term::lam(scrut_ty.clone(), ret_ty)),
            Box::new(Term::pi(scrut_ty, Term::ty(Level::zero()))),
        )
    }

    fn proof_only_motive(scrut_ty: Term) -> Term {
        Term::Ascript(
            Box::new(Term::lam(scrut_ty.clone(), Term::Omega(Level::zero()))),
            Box::new(Term::pi(scrut_ty, Term::Omega(Level::zero()))),
        )
    }

    fn data_match_package() -> (CheckedCorePackage, StableSymbol, StableSymbol, StableSymbol) {
        let target = decl_symbol("target_data_match");
        let target_id = GlobalId(1);
        let bool_id = GlobalId(10);
        let false_id = GlobalId(11);
        let true_id = GlobalId(12);
        let (bool_ty, false_ctor, true_ctor) = bool_data_symbols();
        let table = table_many(&[
            (target_id, target.clone()),
            (bool_id, bool_ty.clone()),
            (false_id, false_ctor.clone()),
            (true_id, true_ctor.clone()),
        ]);
        let scrut_ty = Term::IndFormer {
            id: bool_id,
            level_args: Vec::new(),
        };
        let match_body = Term::Elim {
            fam: bool_id,
            level_args: Vec::new(),
            params: Vec::new(),
            motive: Box::new(constant_type_motive(
                scrut_ty.clone(),
                Term::Type(Level::zero()),
            )),
            methods: vec![
                Term::Constructor {
                    id: false_id,
                    level_args: Vec::new(),
                },
                Term::Constructor {
                    id: true_id,
                    level_args: Vec::new(),
                },
            ],
            indices: Vec::new(),
            scrut: Box::new(Term::Constructor {
                id: true_id,
                level_args: Vec::new(),
            }),
        };
        let decl = Decl::Transparent {
            id: target_id,
            level_params: Vec::new(),
            ty: scrut_ty,
            body: match_body,
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        for symbol in [&target, &bool_ty, &false_ctor, &true_ctor] {
            semantic.symbols.insert(symbol.clone());
            semantic
                .lowerability
                .insert(symbol.clone(), LowerabilityStatus::Supported);
        }
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic.data_metadata.insert(
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
        let package = emit_checked_core_package(
            body_view_header(),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, false_ctor, true_ctor)
    }

    fn record_sigma_symbols() -> (StableSymbol, StableSymbol, StableSymbol) {
        (
            decl_symbol("RecordPayload"),
            decl_symbol("RuntimePayload"),
            decl_symbol("record_value"),
        )
    }

    fn record_sigma_metadata(payload: &StableSymbol) -> RecordSigmaMetadata {
        RecordSigmaMetadata {
            kind: RecordSigmaKind::Record,
            fields: vec![
                FieldMetadata {
                    name: "runtime_payload".to_string(),
                    ty: payload.clone(),
                    runtime: RuntimeFieldStatus::Runtime,
                },
                FieldMetadata {
                    name: "law_payload".to_string(),
                    ty: payload.clone(),
                    runtime: RuntimeFieldStatus::ErasedLaw,
                },
                FieldMetadata {
                    name: "proof_payload".to_string(),
                    ty: payload.clone(),
                    runtime: RuntimeFieldStatus::ErasedProof,
                },
            ],
            lowerability: LowerabilityStatus::Supported,
        }
    }

    fn record_sigma_table(
        record: StableSymbol,
        payload: StableSymbol,
        target: StableSymbol,
    ) -> StableSymbolTable {
        table_many(&[
            (GlobalId(1), target),
            (GlobalId(10), record),
            (GlobalId(11), payload),
        ])
    }

    fn record_type() -> Term {
        Term::Const {
            id: GlobalId(10),
            level_args: Vec::new(),
        }
    }

    fn payload_type() -> Term {
        Term::Const {
            id: GlobalId(11),
            level_args: Vec::new(),
        }
    }

    fn record_pair_body() -> Term {
        Term::Pair(
            Box::new(Term::Var(0)),
            Box::new(Term::Pair(
                Box::new(Term::Type(Level::zero())),
                Box::new(Term::Pair(
                    Box::new(Term::Omega(Level::zero())),
                    Box::new(Term::Type(Level::zero())),
                )),
            )),
        )
    }

    fn record_sigma_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
        let (record, payload, target) = record_sigma_symbols();
        let table = record_sigma_table(record.clone(), payload.clone(), target.clone());
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::pi(payload_type(), record_type()),
            body: Term::Lam(Box::new(payload_type()), Box::new(record_pair_body())),
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        for symbol in [&target, &record, &payload] {
            semantic.symbols.insert(symbol.clone());
            semantic
                .lowerability
                .insert(symbol.clone(), LowerabilityStatus::Supported);
        }
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic
            .record_sigma_metadata
            .insert(record.clone(), record_sigma_metadata(&payload));
        let package = emit_checked_core_package(
            body_view_header(),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, record)
    }

    fn primitive_symbols() -> (StableSymbol, StableSymbol, StableSymbol) {
        (
            decl_symbol("primitive_target"),
            StableSymbol::primitive("lit_int_2"),
            StableSymbol::primitive("add_int"),
        )
    }

    fn primitive_table(
        target: StableSymbol,
        literal: StableSymbol,
        add: StableSymbol,
    ) -> StableSymbolTable {
        table_many(&[
            (GlobalId(1), target),
            (GlobalId(40), literal),
            (GlobalId(41), add),
        ])
    }

    fn primitive_application_body() -> Term {
        Term::App(
            Box::new(Term::App(
                Box::new(Term::Const {
                    id: GlobalId(41),
                    level_args: Vec::new(),
                }),
                Box::new(Term::Const {
                    id: GlobalId(40),
                    level_args: Vec::new(),
                }),
            )),
            Box::new(Term::Const {
                id: GlobalId(40),
                level_args: Vec::new(),
            }),
        )
    }

    fn primitive_body_view_package(
    ) -> (CheckedCorePackage, StableSymbol, StableSymbol, StableSymbol) {
        let (target, literal, add) = primitive_symbols();
        let table = primitive_table(target.clone(), literal.clone(), add.clone());
        let ty = Term::Type(Level::zero());
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: ty.clone(),
            body: primitive_application_body(),
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        for symbol in [&target, &literal, &add] {
            semantic.symbols.insert(symbol.clone());
            semantic
                .lowerability
                .insert(symbol.clone(), LowerabilityStatus::Supported);
        }
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic
            .primitive_refs
            .insert(literal.clone(), "primitive-registry:lit_int_2".to_string());
        semantic.primitive_metadata.insert(
            literal.clone(),
            PrimitiveMetadata {
                registry_symbol: "lit_int_2".to_string(),
                reduction: PrimitiveReductionMetadata::Literal,
                partiality: PartialityMetadata::Total,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        semantic
            .primitive_refs
            .insert(add.clone(), "primitive-registry:add_int".to_string());
        semantic.primitive_metadata.insert(
            add.clone(),
            PrimitiveMetadata {
                registry_symbol: "add_int".to_string(),
                reduction: PrimitiveReductionMetadata::Op,
                partiality: PartialityMetadata::Total,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        let package = emit_checked_core_package(
            body_view_header(),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, literal, add)
    }

    fn recursive_body_view_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
        let target = decl_symbol("recursive_target");
        let group = decl_symbol("recursive_target.group");
        let table = table(GlobalId(1), target.clone());
        let ty = Term::Type(Level::zero());
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: ty.clone(),
            body: Term::Const {
                id: GlobalId(1),
                level_args: Vec::new(),
            },
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        for symbol in [&target, &group] {
            semantic.symbols.insert(symbol.clone());
            semantic
                .lowerability
                .insert(symbol.clone(), LowerabilityStatus::Supported);
        }
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic.recursion_metadata.insert(
            group.clone(),
            RecursionMetadata {
                group_members: vec![target.clone()],
                admission: RecursionAdmission::AcceptedStructural,
                scc_index: 0,
                lowerability: LowerabilityStatus::Supported,
            },
        );
        let package = emit_checked_core_package(
            body_view_header(),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, group)
    }

    fn imported_body_view_package() -> (CheckedCorePackage, StableSymbol, StableSymbol, StableSymbol)
    {
        let target = decl_symbol("importing_target");
        let imported = StableSymbol::declaration("dependency-fixture", &["Dep"], "dep_value");
        let dependency = StableSymbol::new(
            SymbolNamespace::Dependency,
            vec!["dependency-fixture".to_string(), "checked-core".to_string()],
        );
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(90), imported.clone()),
        ]);
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::Type(Level::zero()),
            body: Term::Const {
                id: GlobalId(90),
                level_args: Vec::new(),
            },
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        for symbol in [&target, &imported, &dependency] {
            semantic.symbols.insert(symbol.clone());
        }
        for symbol in [&target, &imported] {
            semantic
                .lowerability
                .insert(symbol.clone(), LowerabilityStatus::Supported);
        }
        semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        semantic
            .dependency_semantic_hashes
            .insert(dependency.clone(), "sha256:dependency-body".to_string());
        semantic
            .dependency_declaration_refs
            .insert(imported.clone(), dependency.clone());
        let mut header = body_view_header();
        header.dependency_semantic_hashes = semantic.dependency_semantic_hashes.clone();
        let package = emit_checked_core_package(
            header,
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, imported, dependency)
    }

    fn dictionary_body_view_package(
    ) -> (CheckedCorePackage, StableSymbol, StableSymbol, StableSymbol) {
        let target = decl_symbol("dictionary_target");
        let helper = decl_symbol("dictionary_eq_field");
        let dictionary = decl_symbol("EqBoolDict");
        let class = decl_symbol("Eq");
        let head = decl_symbol("Bool");
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(2), helper.clone()),
            (GlobalId(10), dictionary.clone()),
        ]);
        let ty = Term::Type(Level::zero());
        let helper_decl = Decl::Transparent {
            id: GlobalId(2),
            level_params: Vec::new(),
            ty: ty.clone(),
            body: Term::Lam(Box::new(ty.clone()), Box::new(Term::Var(0))),
        };
        let target_decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::Const {
                id: GlobalId(10),
                level_args: Vec::new(),
            },
            body: Term::Pair(
                Box::new(Term::Const {
                    id: GlobalId(2),
                    level_args: Vec::new(),
                }),
                Box::new(Term::Pair(
                    Box::new(Term::Omega(Level::zero())),
                    Box::new(Term::Type(Level::zero())),
                )),
            ),
        };

        let mut semantic = CheckedCoreSemanticInputs::default();
        for symbol in [&target, &helper, &dictionary, &class, &head] {
            semantic.symbols.insert(symbol.clone());
        }
        for symbol in [&target, &helper, &dictionary] {
            semantic
                .lowerability
                .insert(symbol.clone(), LowerabilityStatus::Supported);
        }
        semantic.declarations.insert(
            target.clone(),
            canonical_decl_bytes(&target_decl, &table).unwrap(),
        );
        semantic.declarations.insert(
            helper.clone(),
            canonical_decl_bytes(&helper_decl, &table).unwrap(),
        );
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
        let package = emit_checked_core_package(
            body_view_header(),
            CheckedCoreArtifactInputs {
                semantic,
                source_identity: BTreeMap::new(),
                annotations: BTreeMap::new(),
            },
        )
        .unwrap();
        (package, target, helper, dictionary)
    }

    fn replace_body(
        package: &mut CheckedCorePackage,
        target: &StableSymbol,
        body: Term,
        table: &StableSymbolTable,
    ) {
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::Type(Level::zero()),
            body,
        };
        package
            .artifact
            .semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, table).unwrap());
        refresh_package_hashes(package);
    }

    fn body_view_selection(
        package: &CheckedCorePackage,
        target: StableSymbol,
        reachable_declarations: BTreeSet<StableSymbol>,
    ) -> CheckedCoreBodyViewSelection {
        CheckedCoreBodyViewSelection {
            package_identity: package.header.package_identity.clone(),
            package_core_semantic_hash: package.core_semantic_hash,
            package_artifact_hash: package.artifact_hash,
            target_symbol: target,
            reachable_declarations,
            external_symbols: BTreeSet::new(),
            dependency_semantic_hashes: package
                .artifact
                .semantic
                .dependency_semantic_hashes
                .clone(),
        }
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
    fn body_view_recovers_target_and_reachable_declaration_bodies() {
        let (package, target, helper) = body_view_package();
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper.clone()]),
        );

        let view = checked_core_body_view_for_selection(&package, &selection).unwrap();

        assert_eq!(view.package_identity, package.header.package_identity);
        assert_eq!(view.package_core_semantic_hash, package.core_semantic_hash);
        assert_eq!(view.package_artifact_hash, package.artifact_hash);
        assert_eq!(view.target_symbol, target);
        assert!(
            view.declarations.contains_key(&helper),
            "reachable declaration body must be recovered from package bytes"
        );
        let target_body = &view.declarations[&view.target_symbol].body;
        match target_body {
            CheckedCoreBodyTerm::Lambda { body, .. } => match body.as_ref() {
                CheckedCoreBodyTerm::Let { value, body, .. } => {
                    match value.as_ref() {
                        CheckedCoreBodyTerm::Application { function, argument } => {
                            assert_eq!(
                                function.as_ref(),
                                &CheckedCoreBodyTerm::DirectDeclarationCall {
                                    symbol: helper,
                                    level_args: Vec::new(),
                                }
                            );
                            assert_eq!(
                                argument.as_ref(),
                                &CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 }
                            );
                        }
                        other => panic!("expected helper application, got {other:?}"),
                    }
                    assert_eq!(
                        body.as_ref(),
                        &CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 }
                    );
                }
                other => panic!("expected let body, got {other:?}"),
            },
            other => panic!("expected lambda body, got {other:?}"),
        }
    }

    /// Durable invariant: the semantic body view preserves every Ken `Int`
    /// value accepted by the canonical checked-core encoding, including values
    /// outside the native `i64` range.
    #[test]
    fn body_view_preserves_arbitrary_precision_integer_literals() {
        let (mut package, target, _helper) = body_view_package();
        let value = BigInt::from(u64::MAX);
        replace_body(
            &mut package,
            &target,
            Term::IntLit(value.clone()),
            &table(GlobalId(1), target.clone()),
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let view = checked_core_declaration_body_view(&package, &selection, &target)
            .expect("an arbitrary-precision Int literal has a checked runtime body view");

        assert_eq!(
            view.body,
            CheckedCoreBodyTerm::IntegerLiteral { value },
            "the semantic view must not narrow the canonical BigInt payload"
        );
    }

    #[test]
    fn body_view_recovers_package_bound_primitive_literal_and_application() {
        let (package, target, literal, add) = primitive_body_view_package();
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::PrimitiveApplication(application) = view.body else {
            panic!("expected package-derived primitive application view");
        };
        assert_eq!(application.primitive.symbol, add);
        assert_eq!(application.primitive.registry_symbol, "add_int");
        assert_eq!(
            application.primitive.reduction,
            PrimitiveReductionMetadata::Op
        );
        assert_eq!(application.primitive.partiality, PartialityMetadata::Total);
        assert_eq!(
            application.primitive.lowerability,
            LowerabilityStatus::Supported
        );
        assert_eq!(application.arguments.len(), 2);
        for argument in application.arguments {
            let CheckedCoreBodyTerm::PrimitiveLiteral(literal_view) = argument else {
                panic!("expected primitive literal argument");
            };
            assert_eq!(literal_view.symbol, literal);
            assert_eq!(literal_view.registry_symbol, "lit_int_2");
            assert_eq!(literal_view.reduction, PrimitiveReductionMetadata::Literal);
            assert_eq!(literal_view.partiality, PartialityMetadata::Total);
            assert_eq!(literal_view.lowerability, LowerabilityStatus::Supported);
        }
    }

    #[test]
    fn body_view_recovers_recursive_imported_and_dictionary_package_seams() {
        let (package, target, group) = recursive_body_view_package();
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::RecursiveDeclarationCall(recursive) = view.body else {
            panic!("expected package-derived recursive declaration call view");
        };
        assert_eq!(recursive.symbol, target);
        assert_eq!(recursive.group_symbol, group);
        assert_eq!(recursive.group_members, vec![recursive.symbol.clone()]);
        assert_eq!(recursive.admission, RecursionAdmission::AcceptedStructural);
        assert_eq!(recursive.lowerability, LowerabilityStatus::Supported);

        let (package, target, imported, dependency) = imported_body_view_package();
        let mut selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));
        selection.external_symbols.insert(imported.clone());
        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::ImportedDeclarationCall(import_view) = view.body else {
            panic!("expected package-derived imported declaration call view");
        };
        assert_eq!(import_view.symbol, imported);
        assert_eq!(import_view.dependency, dependency);
        assert_eq!(
            import_view.dependency_semantic_hash,
            "sha256:dependency-body"
        );

        let (package, target, helper, dictionary) = dictionary_body_view_package();
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper.clone()]),
        );
        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::DictionaryConstruction(dictionary_view) = view.body else {
            panic!("expected package-derived dictionary construction view");
        };
        assert_eq!(dictionary_view.dictionary.symbol, dictionary);
        assert_eq!(dictionary_view.dictionary.fields.len(), 2);
        assert_eq!(
            dictionary_view.dictionary.fields[0].runtime,
            DictionaryFieldRuntimeStatus::Runtime
        );
        assert_eq!(
            dictionary_view.dictionary.fields[1].runtime,
            DictionaryFieldRuntimeStatus::ErasedLawProof
        );
        match &dictionary_view.fields[0] {
            CheckedCoreDictionaryFieldValue::Runtime { field, value } => {
                assert_eq!(field.name, "eq");
                assert_eq!(
                    value.as_ref(),
                    &CheckedCoreBodyTerm::DirectDeclarationCall {
                        symbol: helper,
                        level_args: Vec::new(),
                    }
                );
            }
            other => panic!("expected runtime dictionary field, got {other:?}"),
        }
        match &dictionary_view.fields[1] {
            CheckedCoreDictionaryFieldValue::Erased { field, term } => {
                assert_eq!(field.name, "law");
                assert!(
                    !term.is_empty(),
                    "erased law/proof dictionary field must remain report-visible bytes"
                );
            }
            other => panic!("expected erased dictionary law field, got {other:?}"),
        }
    }

    #[test]
    fn body_view_rejects_unsupported_recursive_shape() {
        let (mut package, target, group) = recursive_body_view_package();
        package
            .artifact
            .semantic
            .recursion_metadata
            .get_mut(&group)
            .unwrap()
            .admission = RecursionAdmission::Rejected;
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnsupportedRecursiveShape { .. }
        ));
        assert_eq!(err.lane(), "unsupported_recursive_shape");
    }

    #[test]
    fn body_view_rejects_stale_or_missing_dependency_identity() {
        let (package, target, imported, dependency) = imported_body_view_package();
        let mut selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));
        selection.external_symbols.insert(imported.clone());
        selection
            .dependency_semantic_hashes
            .insert(dependency.clone(), "sha256:stale".to_string());

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::StaleDependencyIdentity { .. }
        ));
        assert_eq!(err.lane(), "stale_dependency_identity");

        let (mut package, target, imported, _dependency) = imported_body_view_package();
        package
            .artifact
            .semantic
            .dependency_declaration_refs
            .clear();
        refresh_package_hashes(&mut package);
        let mut selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));
        selection.external_symbols.insert(imported);

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::MissingDependencyIdentity { .. }
        ));
        assert_eq!(err.lane(), "missing_dependency_identity");
    }

    #[test]
    fn body_view_rejects_non_executable_dictionary_field_use() {
        let (mut package, target, helper, dictionary) = dictionary_body_view_package();
        package
            .artifact
            .semantic
            .class_instance_metadata
            .get_mut(&dictionary)
            .unwrap()
            .runtime_fields
            .insert("law".to_string());
        refresh_package_hashes(&mut package);
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper]),
        );

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::NonExecutableDictionaryFieldUse { .. }
        ));
        assert_eq!(err.lane(), "non_executable_dictionary_field_use");
    }

    #[test]
    fn body_view_rejects_stale_primitive_metadata() {
        let (mut package, target, _literal, add) = primitive_body_view_package();
        package
            .artifact
            .semantic
            .primitive_refs
            .insert(add.clone(), "primitive-registry:stale_add".to_string());
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::StalePrimitiveMetadata { .. }
        ));
        assert_eq!(err.lane(), "stale_primitive_metadata");
        assert!(err.to_string().contains(&add.to_string()));
    }

    #[test]
    fn body_view_rejects_unsupported_primitive_name() {
        let (mut package, target, _literal, _add) = primitive_body_view_package();
        let missing = StableSymbol::primitive("missing_primitive");
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(99), missing.clone()),
        ]);
        replace_body(
            &mut package,
            &target,
            Term::Const {
                id: GlobalId(99),
                level_args: Vec::new(),
            },
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::UnsupportedPrimitiveName {
                symbol: target,
                primitive: missing,
            }
        );
        assert_eq!(err.lane(), "unsupported_primitive_name");
    }

    #[test]
    fn body_view_rejects_host_dependent_primitive_attempt() {
        let (mut package, target, _literal, add) = primitive_body_view_package();
        let meta = package
            .artifact
            .semantic
            .primitive_metadata
            .get_mut(&add)
            .unwrap();
        meta.lowerability = LowerabilityStatus::RequiresFeature {
            feature: "host-dependent-primitive".to_string(),
            reason: "depends on host locale".to_string(),
        };
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::HostDependentPrimitiveAttempt { .. }
        ));
        assert_eq!(err.lane(), "host_dependent_primitive_attempt");
        assert!(err.to_string().contains("host locale"));
    }

    #[test]
    fn body_view_rejects_unjustified_partial_primitive_contract() {
        let (mut package, target, _literal, add) = primitive_body_view_package();
        let obligation = StableSymbol::obligation("missing-add-int-overflow");
        package.artifact.semantic.symbols.insert(obligation.clone());
        let meta = package
            .artifact
            .semantic
            .primitive_metadata
            .get_mut(&add)
            .unwrap();
        meta.partiality = PartialityMetadata::CheckedPartial { obligation };
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnjustifiedPrimitivePartiality { .. }
        ));
        assert_eq!(err.lane(), "unjustified_primitive_partiality");
    }

    #[test]
    fn body_view_rejects_malformed_declaration_bytes() {
        let (mut package, target, helper) = body_view_package();
        package
            .artifact
            .semantic
            .declarations
            .insert(target.clone(), b"not canonical".to_vec());
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper]),
        );

        let err = checked_core_body_view_for_selection(&package, &selection).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::MalformedCanonicalBytes { .. }
        ));
        assert_eq!(err.lane(), "malformed_checked_declaration_body");
    }

    #[test]
    fn body_view_rejects_stale_package_identity_and_hash_facts() {
        let (package, target, helper) = body_view_package();
        let mut selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper.clone()]),
        );

        selection.package_identity =
            StableSymbol::new(SymbolNamespace::Module, vec!["other_pkg".to_string()]);
        let err = checked_core_body_view_for_selection(&package, &selection).unwrap_err();
        assert!(matches!(
            err,
            CheckedCoreBodyViewError::MismatchedPackageIdentity { .. }
        ));

        selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper.clone()]),
        );
        selection.package_core_semantic_hash ^= 1;
        let err = checked_core_body_view_for_selection(&package, &selection).unwrap_err();
        assert!(matches!(
            err,
            CheckedCoreBodyViewError::MismatchedCoreSemanticHash { .. }
        ));

        selection = body_view_selection(&package, target, BTreeSet::from([helper]));
        selection.package_artifact_hash ^= 1;
        let err = checked_core_body_view_for_selection(&package, &selection).unwrap_err();
        assert!(matches!(
            err,
            CheckedCoreBodyViewError::MismatchedArtifactHash { .. }
        ));
    }

    #[test]
    fn body_view_rejects_missing_reachable_body() {
        let (mut package, target, helper) = body_view_package();
        package.artifact.semantic.declarations.remove(&helper);
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target, helper.clone()]),
        );

        let err = checked_core_body_view_for_selection(&package, &selection).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::MissingDeclarationBody { symbol: helper }
        );
    }

    #[test]
    fn body_view_rejects_unsupported_declaration_kind() {
        let (mut package, target, helper) = body_view_package();
        let opaque_id = GlobalId(3);
        let opaque_decl = Decl::Opaque {
            id: opaque_id,
            name: "opaque_test".to_string(),
            level_params: Vec::new(),
            ty: Term::Type(Level::zero()),
        };
        let opaque_table = table(opaque_id, helper.clone());
        package.artifact.semantic.declarations.insert(
            helper.clone(),
            canonical_decl_bytes(&opaque_decl, &opaque_table).unwrap(),
        );
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target, helper.clone()]),
        );

        let err = checked_core_declaration_body_view(&package, &selection, &helper).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::UnsupportedDeclarationKind {
                symbol: helper,
                kind: "opaque".to_string(),
            }
        );
        assert_eq!(err.lane(), "unsupported_checked_declaration_kind");
    }

    #[test]
    fn body_view_rejects_unsupported_transparent_body_shape() {
        let (mut package, target, helper) = body_view_package();
        let target_id = GlobalId(1);
        let ty = Term::Type(Level::zero());
        let unsupported_decl = Decl::Transparent {
            id: target_id,
            level_params: Vec::new(),
            ty: ty.clone(),
            body: Term::Ascript(Box::new(Term::Var(0)), Box::new(ty)),
        };
        package.artifact.semantic.declarations.insert(
            target.clone(),
            canonical_decl_bytes(&unsupported_decl, &table(target_id, target.clone())).unwrap(),
        );
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );
        let selection = body_view_selection(
            &package,
            target.clone(),
            BTreeSet::from([target.clone(), helper]),
        );

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::UnsupportedTermShape {
                symbol: target,
                tag: "ascript".to_string(),
            }
        );
        assert_eq!(err.lane(), "unsupported_checked_body_shape");
    }

    #[test]
    fn body_view_rejects_body_outside_selected_closure() {
        let (package, target, helper) = body_view_package();
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &helper).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::RequestedBodyOutsideSelectedClosure {
                target,
                symbol: helper,
            }
        );
    }

    #[test]
    fn body_view_rejects_target_outside_selected_closure() {
        let (package, target, helper) = body_view_package();
        let selection = body_view_selection(&package, target.clone(), BTreeSet::from([helper]));

        let err = checked_core_body_view_for_selection(&package, &selection).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::TargetOutsideSelectedClosure { target }
        );
    }

    #[test]
    fn body_view_rejects_references_outside_selected_closure() {
        let (package, target, helper) = body_view_package();
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::BodyReferenceOutsideSelectedClosure {
                owner: target,
                referenced: helper,
            }
        );
    }

    #[test]
    fn body_view_recovers_package_bound_constructor_and_match_view() {
        let (package, target, false_ctor, true_ctor) = data_match_package();
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::Match(match_view) = view.body else {
            panic!("expected package-derived match view");
        };
        assert_eq!(match_view.branches.len(), 2);
        assert_eq!(match_view.branches[0].constructor.symbol, false_ctor);
        assert_eq!(match_view.branches[0].constructor.argument_count, 0);
        assert_eq!(match_view.branches[0].constructor.target_index_count, 0);
        assert_eq!(
            match_view.branches[0].constructor.constructor_lowerability,
            LowerabilityStatus::Supported
        );
        assert_eq!(match_view.branches[1].constructor.symbol, true_ctor.clone());
        assert!(match_view.parameters.is_empty());
        assert!(match_view.indices.is_empty());
        assert!(match_view.motive.len() > 8);
        assert_eq!(
            match_view.scrutinee.as_ref(),
            &CheckedCoreBodyTerm::ConstructorReference(CheckedCoreConstructorView {
                symbol: true_ctor.clone(),
                family_symbol: match_view.family_symbol.clone(),
                level_args: Vec::new(),
                family_parameter_count: 0,
                family_index_count: 0,
                argument_count: 0,
                target_index_count: 0,
                recursive_positions: Vec::new(),
                constructor_lowerability: LowerabilityStatus::Supported,
                family_lowerability: LowerabilityStatus::Supported,
            })
        );
        assert!(matches!(
            &match_view.branches[1].method,
            CheckedCoreBodyTerm::ConstructorReference(ctor) if ctor.symbol == true_ctor
        ));
    }

    #[test]
    fn body_view_rejects_stale_constructor_identity() {
        let (mut package, target, _, _) = data_match_package();
        let ghost = StableSymbol::constructor(&decl_symbol("Ghost"), "MadeUp");
        let table = table_many(&[(GlobalId(1), target.clone()), (GlobalId(99), ghost.clone())]);
        replace_body(
            &mut package,
            &target,
            Term::Constructor {
                id: GlobalId(99),
                level_args: Vec::new(),
            },
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::StaleConstructorIdentity {
                owner: target,
                constructor: ghost,
            }
        );
        assert_eq!(err.lane(), "stale_constructor_identity");
    }

    #[test]
    fn body_view_rejects_missing_match_branch_data() {
        let (mut package, target, _, _) = data_match_package();
        let (bool_ty, false_ctor, true_ctor) = bool_data_symbols();
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(10), bool_ty),
            (GlobalId(11), false_ctor),
            (GlobalId(12), true_ctor),
        ]);
        replace_body(
            &mut package,
            &target,
            Term::Elim {
                fam: GlobalId(10),
                level_args: Vec::new(),
                params: Vec::new(),
                motive: Box::new(constant_type_motive(
                    Term::IndFormer {
                        id: GlobalId(10),
                        level_args: Vec::new(),
                    },
                    Term::Type(Level::zero()),
                )),
                methods: vec![Term::Constructor {
                    id: GlobalId(11),
                    level_args: Vec::new(),
                }],
                indices: Vec::new(),
                scrut: Box::new(Term::Constructor {
                    id: GlobalId(12),
                    level_args: Vec::new(),
                }),
            },
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::MissingMatchBranchData {
                expected: 2,
                found: 1,
                ..
            }
        ));
        assert_eq!(err.lane(), "missing_match_branch_data");
    }

    #[test]
    fn body_view_rejects_unsupported_dependent_motive() {
        let (mut package, target, _, _) = data_match_package();
        let (bool_ty, false_ctor, true_ctor) = bool_data_symbols();
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(10), bool_ty),
            (GlobalId(11), false_ctor),
            (GlobalId(12), true_ctor),
        ]);
        let scrut_ty = Term::IndFormer {
            id: GlobalId(10),
            level_args: Vec::new(),
        };
        let motive = Term::Ascript(
            Box::new(Term::lam(scrut_ty.clone(), Term::Var(0))),
            Box::new(Term::pi(scrut_ty, Term::ty(Level::zero()))),
        );
        replace_body(
            &mut package,
            &target,
            Term::Elim {
                fam: GlobalId(10),
                level_args: Vec::new(),
                params: Vec::new(),
                motive: Box::new(motive),
                methods: vec![
                    Term::Constructor {
                        id: GlobalId(11),
                        level_args: Vec::new(),
                    },
                    Term::Constructor {
                        id: GlobalId(12),
                        level_args: Vec::new(),
                    },
                ],
                indices: Vec::new(),
                scrut: Box::new(Term::Constructor {
                    id: GlobalId(12),
                    level_args: Vec::new(),
                }),
            },
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnsupportedDependentMotive { .. }
        ));
        assert_eq!(err.lane(), "unsupported_dependent_motive");
    }

    #[test]
    fn body_view_rejects_unsupported_proof_only_match() {
        let (mut package, target, _, _) = data_match_package();
        let (bool_ty, false_ctor, true_ctor) = bool_data_symbols();
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(10), bool_ty),
            (GlobalId(11), false_ctor),
            (GlobalId(12), true_ctor),
        ]);
        replace_body(
            &mut package,
            &target,
            Term::Elim {
                fam: GlobalId(10),
                level_args: Vec::new(),
                params: Vec::new(),
                motive: Box::new(proof_only_motive(Term::IndFormer {
                    id: GlobalId(10),
                    level_args: Vec::new(),
                })),
                methods: vec![
                    Term::Constructor {
                        id: GlobalId(11),
                        level_args: Vec::new(),
                    },
                    Term::Constructor {
                        id: GlobalId(12),
                        level_args: Vec::new(),
                    },
                ],
                indices: Vec::new(),
                scrut: Box::new(Term::Constructor {
                    id: GlobalId(12),
                    level_args: Vec::new(),
                }),
            },
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnsupportedProofOnlyMatch { .. }
        ));
        assert_eq!(err.lane(), "unsupported_proof_only_match");
    }

    #[test]
    fn body_view_rejects_unsupported_eliminator_shape() {
        let (mut package, target, _, _) = data_match_package();
        let (bool_ty, _, _) = bool_data_symbols();
        package
            .artifact
            .semantic
            .data_metadata
            .get_mut(&bool_ty)
            .unwrap()
            .eliminator = LowerabilityStatus::Unsupported {
            reason: "dependent match lowering not selected".to_string(),
        };
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnsupportedEliminatorShape { .. }
        ));
        assert_eq!(err.lane(), "unsupported_eliminator_shape");
    }

    #[test]
    fn body_view_rejects_unjustified_impossible_branch() {
        let (mut package, target, _, _) = data_match_package();
        let (bool_ty, false_ctor, true_ctor) = bool_data_symbols();
        let table = table_many(&[
            (GlobalId(1), target.clone()),
            (GlobalId(10), bool_ty),
            (GlobalId(11), false_ctor),
            (GlobalId(12), true_ctor),
        ]);
        replace_body(
            &mut package,
            &target,
            Term::Elim {
                fam: GlobalId(10),
                level_args: Vec::new(),
                params: Vec::new(),
                motive: Box::new(constant_type_motive(
                    Term::IndFormer {
                        id: GlobalId(10),
                        level_args: Vec::new(),
                    },
                    Term::Type(Level::zero()),
                )),
                methods: vec![
                    Term::Absurd(Box::new(Term::Type(Level::zero())), Box::new(Term::Var(0))),
                    Term::Constructor {
                        id: GlobalId(12),
                        level_args: Vec::new(),
                    },
                ],
                indices: Vec::new(),
                scrut: Box::new(Term::Constructor {
                    id: GlobalId(12),
                    level_args: Vec::new(),
                }),
            },
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::UnjustifiedImpossibleBranch { symbol: target }
        );
        assert_eq!(err.lane(), "unjustified_impossible_branch");
    }

    #[test]
    fn body_view_recovers_package_bound_record_sigma_construction() {
        let (package, target, record) = record_sigma_package();
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::Lambda { body, .. } = view.body else {
            panic!("expected lambda returning record/Sigma construction");
        };
        let CheckedCoreBodyTerm::RecordSigmaConstruction(construction) = body.as_ref() else {
            panic!("expected package-derived record/Sigma construction view");
        };
        assert_eq!(construction.record.symbol, record);
        assert_eq!(construction.record.kind, RecordSigmaKind::Record);
        assert_eq!(
            construction
                .record
                .fields
                .iter()
                .map(|field| (field.position, field.name.as_str(), field.runtime.clone()))
                .collect::<Vec<_>>(),
            vec![
                (0, "runtime_payload", RuntimeFieldStatus::Runtime),
                (1, "law_payload", RuntimeFieldStatus::ErasedLaw),
                (2, "proof_payload", RuntimeFieldStatus::ErasedProof),
            ]
        );
        assert_eq!(construction.fields.len(), 3);
        assert!(matches!(
            &construction.fields[0],
            CheckedCoreRecordSigmaFieldValue::Runtime {
                field,
                value,
            } if field.name == "runtime_payload"
                && matches!(value.as_ref(), CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 })
        ));
        assert!(matches!(
            &construction.fields[1],
            CheckedCoreRecordSigmaFieldValue::Erased { field, term }
                if field.name == "law_payload" && !term.is_empty()
        ));
        assert!(matches!(
            &construction.fields[2],
            CheckedCoreRecordSigmaFieldValue::Erased { field, term }
                if field.name == "proof_payload" && !term.is_empty()
        ));
        assert!(!construction.terminator.is_empty());
    }

    #[test]
    fn body_view_recovers_runtime_record_sigma_projection() {
        let (mut package, target, record) = record_sigma_package();
        let (record_ty, payload, _) = record_sigma_symbols();
        let table = record_sigma_table(record_ty, payload, target.clone());
        replace_body(
            &mut package,
            &target,
            Term::Lam(
                Box::new(record_type()),
                Box::new(Term::Proj1(Box::new(Term::Var(0)))),
            ),
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let view = checked_core_declaration_body_view(&package, &selection, &target).unwrap();

        let CheckedCoreBodyTerm::Lambda { body, .. } = view.body else {
            panic!("expected projection lambda");
        };
        let CheckedCoreBodyTerm::RecordSigmaProjection(projection) = body.as_ref() else {
            panic!("expected package-derived record/Sigma projection view");
        };
        assert_eq!(projection.record.symbol, record);
        assert_eq!(projection.field.position, 0);
        assert_eq!(projection.field.name, "runtime_payload");
        assert_eq!(projection.field.runtime, RuntimeFieldStatus::Runtime);
        assert!(projection.skipped_fields.is_empty());
        assert!(matches!(
            projection.base.as_ref(),
            CheckedCoreBodyTerm::Variable { de_bruijn_index: 0 }
        ));
    }

    #[test]
    fn body_view_rejects_non_executable_erased_field_projection() {
        let (mut package, target, record) = record_sigma_package();
        let (record_ty, payload, _) = record_sigma_symbols();
        let table = record_sigma_table(record_ty, payload, target.clone());
        replace_body(
            &mut package,
            &target,
            Term::Lam(
                Box::new(record_type()),
                Box::new(Term::Proj1(Box::new(Term::Proj2(Box::new(Term::Var(0)))))),
            ),
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert_eq!(
            err,
            CheckedCoreBodyViewError::NonExecutableErasedFieldProjection {
                symbol: target,
                record,
                field: "law_payload".to_string(),
            }
        );
        assert_eq!(err.lane(), "non_executable_erased_field_projection");
    }

    #[test]
    fn body_view_rejects_stale_field_identity_order_mismatch() {
        let (mut package, target, record) = record_sigma_package();
        let (record_ty, payload, _) = record_sigma_symbols();
        let table = record_sigma_table(record_ty, payload, target.clone());
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::pi(payload_type(), record_type()),
            body: Term::Lam(
                Box::new(payload_type()),
                Box::new(Term::Pair(
                    Box::new(Term::Var(0)),
                    Box::new(Term::Type(Level::zero())),
                )),
            ),
        };
        package
            .artifact
            .semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::StaleFieldIdentityOrder { .. }
        ));
        assert_eq!(err.lane(), "stale_field_identity_order");
        assert!(err.to_string().contains(&record.to_string()));
    }

    #[test]
    fn body_view_rejects_unsupported_dependent_record_sigma_field_shape() {
        let (mut package, target, _) = record_sigma_package();
        let (record_ty, payload, _) = record_sigma_symbols();
        let table = record_sigma_table(record_ty, payload, target.clone());
        replace_body(
            &mut package,
            &target,
            Term::Lam(
                Box::new(payload_type()),
                Box::new(Term::Pair(
                    Box::new(Term::Var(0)),
                    Box::new(Term::Type(Level::zero())),
                )),
            ),
            &table,
        );
        let decl = Decl::Transparent {
            id: GlobalId(1),
            level_params: Vec::new(),
            ty: Term::pi(
                payload_type(),
                Term::sigma(Term::Type(Level::zero()), Term::Var(0)),
            ),
            body: Term::Lam(
                Box::new(payload_type()),
                Box::new(Term::Pair(
                    Box::new(Term::Var(0)),
                    Box::new(Term::Type(Level::zero())),
                )),
            ),
        };
        package
            .artifact
            .semantic
            .declarations
            .insert(target.clone(), canonical_decl_bytes(&decl, &table).unwrap());
        refresh_package_hashes(&mut package);
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnsupportedDependentFieldShape { .. }
        ));
        assert_eq!(err.lane(), "unsupported_dependent_field_shape");
    }

    #[test]
    fn body_view_rejects_unsupported_record_projection_shape() {
        let (mut package, target, _) = record_sigma_package();
        let (record_ty, payload, _) = record_sigma_symbols();
        let table = record_sigma_table(record_ty, payload, target.clone());
        replace_body(
            &mut package,
            &target,
            Term::Proj1(Box::new(Term::Var(0))),
            &table,
        );
        let selection =
            body_view_selection(&package, target.clone(), BTreeSet::from([target.clone()]));

        let err = checked_core_declaration_body_view(&package, &selection, &target).unwrap_err();

        assert!(matches!(
            err,
            CheckedCoreBodyViewError::UnsupportedRecordProjectionShape { .. }
        ));
        assert_eq!(err.lane(), "unsupported_record_projection_shape");
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
                runtime_fields: BTreeSet::from(["eq".to_string()]),
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
        // Rework (Q-RESIDUE, 2026-07-21): `representative_checked_core_
        // fixtures` is a growable representative corpus (today one entry);
        // pinning its length froze that growth for no semantic reason. This
        // only asserts the corpus is non-vacuous, so the loop below actually
        // validates something, and validates every entry it contains.
        let fixtures = representative_checked_core_fixtures().unwrap();

        assert!(
            !fixtures.is_empty(),
            "the representative corpus must not be vacuously empty"
        );
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
    fn validator_rejects_bad_kind_empty_header_and_incomplete_lowerability() {
        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;

        package.header.package_kind = "UncheckedCorePackage".to_string();
        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::UnsupportedPackageKind {
                found: "UncheckedCorePackage".to_string(),
            }
        );

        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        package.header.kernel_ref.clear();
        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::EmptyHeaderField {
                field: "kernel_ref",
            }
        );

        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        let bool_ty = StableSymbol::declaration("fixture", &["Core"], "Bool");
        package.artifact.semantic.lowerability.remove(&bool_ty);
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );

        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::MissingLowerability { symbol: bool_ty },
        );
    }

    #[test]
    fn validator_rejects_unsupported_entries_without_blocking_lowerability() {
        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        let bool_ty = StableSymbol::declaration("fixture", &["Core"], "Bool");
        package
            .artifact
            .semantic
            .unsupported
            .insert(bool_ty.clone(), b"must block if unsupported".to_vec());
        package
            .artifact
            .semantic
            .lowerability
            .insert(bool_ty.clone(), LowerabilityStatus::Supported);
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );

        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::UnsupportedEntryNotBlocking { symbol: bool_ty },
        );
    }

    #[test]
    fn validator_rejects_dependency_hash_lane_mismatch_after_hash_recompute() {
        let mut package = representative_checked_core_fixtures()
            .unwrap()
            .pop()
            .unwrap()
            .package;
        let dependency = StableSymbol::new(
            SymbolNamespace::Dependency,
            vec!["dep-pkg".to_string(), "checked-core".to_string()],
        );
        package.artifact.semantic.symbols.insert(dependency.clone());
        package
            .artifact
            .semantic
            .dependency_semantic_hashes
            .insert(dependency.clone(), "sha256:semantic-lane".to_string());
        package
            .header
            .dependency_semantic_hashes
            .insert(dependency, "sha256:header-lane".to_string());
        package.core_semantic_hash = semantic_fingerprint(&package.artifact.semantic);
        package.artifact_hash = package_artifact_fingerprint(
            &package.header,
            &package.artifact,
            package.core_semantic_hash,
        );

        assert_eq!(
            validate_checked_core_package(&package).unwrap_err(),
            CheckedCorePackageError::DependencySemanticHashesMismatch,
            "hash recomputation must not hide disagreement between header and semantic dependency closures"
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
