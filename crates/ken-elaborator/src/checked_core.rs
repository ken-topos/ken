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
    pub metadata: BTreeMap<StableSymbol, Vec<u8>>,
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
    encode_bytes_map("metadata", &inputs.metadata, &mut out);
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
}
