//! Ken runtime IR seed (NC5).
//!
//! This module defines the first backend-neutral operational artifact below
//! `CheckedCorePackage v0`. It deliberately names checked-core symbols by
//! stable strings and carries semantic metadata by hash/input bytes; it does not
//! assign native layout, ABI slots, Cranelift operations, pointer identities, or
//! backend poison values.

use std::collections::{BTreeMap, BTreeSet};

/// Stable checked-core symbol rendered at the package boundary.
pub type RuntimeSymbol = String;

/// A capability operand carried by an effectful runtime node.
///
/// `identity` is observation-only provenance. `value` is the live, opaque
/// credential and is the only field allowed to authorize a host operation.
/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub struct RuntimeCapabilityUse {
    pub identity: RuntimeSymbol,
    pub value: Box<RuntimeExpr>,
}

/// Complete NC5 runtime artifact for one checked-core package subset.
/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub struct RuntimeProgram {
    pub package_identity: RuntimeSymbol,
    pub core_semantic_hash: u64,
    pub artifact_hash: u64,
    pub erased_core: ErasedExecutableCore,
    pub declarations: Vec<RuntimeDeclaration>,
    pub examples: Vec<RuntimeExample>,
}

/// Intermediate semantic artifact between checked core and runtime IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ErasedExecutableCore {
    pub symbols: BTreeSet<RuntimeSymbol>,
    pub metadata: RuntimeMetadata,
}

/// Metadata that remains authoritative after proof erasure.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeMetadata {
    pub obligations: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub obligation_metadata: BTreeMap<RuntimeSymbol, RuntimeObligationMetadata>,
    pub assumptions: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub assumption_trust_metadata: BTreeMap<RuntimeSymbol, RuntimeAssumptionTrustMetadata>,
    pub trusted_base_delta: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub dependency_semantic_hashes: BTreeMap<RuntimeSymbol, String>,
    pub lowerability: BTreeMap<RuntimeSymbol, RuntimeLowerabilityStatus>,
    pub unsupported: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub runtime_declaration_targets: BTreeSet<RuntimeSymbol>,
    pub checked_core: RuntimeCheckedCoreMetadata,
    pub runtime_checks: BTreeSet<RuntimeSymbol>,
    pub capabilities: BTreeSet<RuntimeSymbol>,
    pub effects: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeObligationMetadata {
    pub status: RuntimeObligationStatus,
    pub origin: RuntimeSymbol,
    pub affects_runtime_meaning: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeObligationStatus {
    Proved,
    Tested,
    Delegated,
    Unknown,
    Disproved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAssumptionTrustMetadata {
    pub kind: RuntimeAssumptionTrustKind,
    pub target: RuntimeSymbol,
    pub affects_runtime_meaning: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeAssumptionTrustKind {
    Postulate,
    Hole,
    Foreign,
    Declassify,
    PrimitiveAssumption,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeLowerabilityStatus {
    Supported,
    Unsupported { reason: String },
    Deferred { later_stage: String, reason: String },
    RequiresFeature { feature: String, reason: String },
    Explicit { state: String, reason: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeCheckedCoreMetadata {
    pub primitive_metadata: BTreeMap<RuntimeSymbol, RuntimePrimitiveAuditMetadata>,
    pub data_metadata: BTreeMap<RuntimeSymbol, RuntimeDataAuditMetadata>,
    pub record_sigma_metadata: BTreeMap<RuntimeSymbol, RuntimeRecordSigmaAuditMetadata>,
    pub class_instance_metadata: BTreeMap<RuntimeSymbol, RuntimeClassInstanceAuditMetadata>,
    pub recursion_metadata: BTreeMap<RuntimeSymbol, RuntimeRecursionAuditMetadata>,
    pub effects_foreign_metadata: BTreeMap<RuntimeSymbol, RuntimeEffectsForeignAuditMetadata>,
    pub metadata: BTreeMap<RuntimeSymbol, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePrimitiveAuditMetadata {
    pub registry_symbol: String,
    pub reduction: RuntimePrimitiveReductionMetadata,
    pub partiality: RuntimePartialityMetadata,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePrimitiveReductionMetadata {
    OpaqueType,
    Literal,
    Op,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePartialityMetadata {
    Total,
    CheckedPartial { obligation: RuntimeSymbol },
    TrustedPartial { assumption: RuntimeSymbol },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDataAuditMetadata {
    pub parameter_count: usize,
    pub index_count: usize,
    pub constructors: Vec<RuntimeConstructorAuditMetadata>,
    pub eliminator: RuntimeLowerabilityStatus,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConstructorAuditMetadata {
    pub symbol: RuntimeSymbol,
    pub argument_count: usize,
    pub target_index_count: usize,
    pub recursive_positions: Vec<usize>,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecordSigmaAuditMetadata {
    pub kind: RuntimeRecordSigmaKind,
    pub fields: Vec<RuntimeFieldAuditMetadata>,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeRecordSigmaKind {
    Record,
    Sigma,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFieldAuditMetadata {
    pub name: String,
    pub ty: RuntimeSymbol,
    pub runtime: RuntimeFieldStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeClassInstanceAuditMetadata {
    pub kind: RuntimeClassInstanceKind,
    pub class_symbol: Option<RuntimeSymbol>,
    pub dictionary_symbol: Option<RuntimeSymbol>,
    pub head_symbol: Option<RuntimeSymbol>,
    pub field_order: Vec<String>,
    pub runtime_fields: BTreeSet<String>,
    pub law_fields: BTreeSet<String>,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeClassInstanceKind {
    Class,
    Instance,
    Dictionary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecursionAuditMetadata {
    pub group_members: Vec<RuntimeSymbol>,
    pub admission: RuntimeRecursionAdmission,
    pub scc_index: usize,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeRecursionAdmission {
    NonRecursive,
    AcceptedStructural,
    AcceptedSizeChange,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeEffectsForeignAuditMetadata {
    pub declared_effects: BTreeSet<String>,
    pub capabilities: BTreeSet<RuntimeSymbol>,
    pub foreign_symbol: Option<String>,
    pub boundary: RuntimeEffectBoundary,
    pub runtime_checks: BTreeSet<RuntimeSymbol>,
    pub lowerability: RuntimeLowerabilityStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEffectBoundary {
    Pure,
    Effectful,
    Foreign,
}

/// Runtime declaration lowered from a checked-core symbol.
/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub struct RuntimeDeclaration {
    pub symbol: RuntimeSymbol,
    pub kind: RuntimeDeclarationKind,
    pub metadata: RuntimeSymbolMetadata,
}

/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub enum RuntimeDeclarationKind {
    Transparent {
        body: RuntimeExpr,
    },
    Primitive {
        op: RuntimePrimitive,
    },
    Data {
        constructors: Vec<RuntimeConstructor>,
    },
    Record {
        fields: Vec<RuntimeField>,
    },
    RecursiveGroup {
        members: Vec<RuntimeSymbol>,
    },
    EffectBoundary {
        effects: BTreeSet<String>,
    },
    MetadataOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSymbolMetadata {
    pub obligations: BTreeSet<RuntimeSymbol>,
    pub obligation_metadata: BTreeMap<RuntimeSymbol, RuntimeObligationMetadata>,
    pub assumptions: BTreeSet<RuntimeSymbol>,
    pub assumption_trust_metadata: BTreeMap<RuntimeSymbol, RuntimeAssumptionTrustMetadata>,
    pub trusted_base_delta: BTreeSet<RuntimeSymbol>,
    pub lowerability: Option<RuntimeLowerabilityStatus>,
    pub unsupported: Option<Vec<u8>>,
    pub runtime_checks: BTreeSet<RuntimeSymbol>,
    pub capabilities: BTreeSet<RuntimeSymbol>,
    pub effects: BTreeSet<String>,
}

impl RuntimeSymbolMetadata {
    pub fn empty() -> Self {
        Self {
            obligations: BTreeSet::new(),
            obligation_metadata: BTreeMap::new(),
            assumptions: BTreeSet::new(),
            assumption_trust_metadata: BTreeMap::new(),
            trusted_base_delta: BTreeSet::new(),
            lowerability: None,
            unsupported: None,
            runtime_checks: BTreeSet::new(),
            capabilities: BTreeSet::new(),
            effects: BTreeSet::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConstructor {
    pub symbol: RuntimeSymbol,
    pub runtime_arg_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeField {
    pub name: String,
    pub status: RuntimeFieldStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeFieldStatus {
    Runtime,
    ErasedLaw,
    ErasedProof,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePrimitive {
    pub symbol: String,
    pub partiality: RuntimePartiality,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePartiality {
    Total,
    /// A checked operation whose failure is represented by `None`, never a
    /// trap. `obligation` names the native bounds check when one is required.
    SafeOption {
        none: RuntimeSymbol,
        some: RuntimeSymbol,
        obligation: Option<RuntimeSymbol>,
    },
    /// A checked operation whose failure is represented by `Err error`.
    SafeResult {
        err: RuntimeSymbol,
        ok: RuntimeSymbol,
        error: RuntimeSymbol,
    },
    CheckedTrap {
        obligation: RuntimeSymbol,
    },
    TrustedTrap {
        assumption: RuntimeSymbol,
    },
}

/// Backend-neutral runtime expression language.
/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub enum RuntimeExpr {
    #[doc(hidden)]
    CheckedJoinSite {
        site_id: u64,
        body: Box<RuntimeExpr>,
    },
    /// Exact checked pre-erasure frame marker for native oriented
    /// subcontinuation validation. This is compiler-private metadata, not a
    /// source-visible operation or a runtime semantic effect.
    #[doc(hidden)]
    CheckedSubcontinuationFrame {
        frame_id: u64,
        body: Box<RuntimeExpr>,
    },
    /// Exact checked marker for one complete same-SCC recursive application.
    /// The marker names a reusable static call template; native lowering mints
    /// a fresh affine invocation identity when it consumes the marker.
    #[doc(hidden)]
    CheckedRecursiveInvocation {
        call_template_id: u64,
        checked_occurrence_path: Vec<u64>,
        body: Box<RuntimeExpr>,
    },
    /// Exact checked slot templates aligned with the recursive positions of
    /// one computational match case.
    #[doc(hidden)]
    CheckedComputationalIHSlots {
        slot_template_ids: Vec<u64>,
        checked_occurrence_paths: Vec<Vec<u64>>,
        body: Box<RuntimeExpr>,
    },
    /// Exact checked marker for one complete application of a bound
    /// computational induction hypothesis.
    #[doc(hidden)]
    CheckedComputationalIHInvocation {
        call_template_id: u64,
        checked_occurrence_path: Vec<u64>,
        body: Box<RuntimeExpr>,
    },
    Value(RuntimeValue),
    Var(u32),
    Let {
        value: Box<RuntimeExpr>,
        body: Box<RuntimeExpr>,
    },
    If {
        scrutinee: Box<RuntimeExpr>,
        then_expr: Box<RuntimeExpr>,
        else_expr: Box<RuntimeExpr>,
    },
    PrimitiveCall {
        primitive: RuntimePrimitive,
        args: Vec<RuntimeExpr>,
    },
    Construct {
        constructor: RuntimeSymbol,
        args: Vec<RuntimeExpr>,
    },
    Match {
        scrutinee: Box<RuntimeExpr>,
        cases: Vec<RuntimeMatchCase>,
        default: RuntimeTrap,
    },
    /// A computational eliminator whose recursive hypotheses are runtime
    /// values.  Each recursive constructor field produces one lazily-applied
    /// recursive hypothesis before the branch body runs.
    ComputationalMatch {
        scrutinee: Box<RuntimeExpr>,
        cases: Vec<RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
    },
    Record {
        fields: Vec<(String, RuntimeExpr)>,
    },
    Project {
        record: Box<RuntimeExpr>,
        field: String,
    },
    Closure {
        captures: Vec<RuntimeSymbol>,
        params: Vec<String>,
        body: Box<RuntimeExpr>,
    },
    /// An ordinary lexical closure.  Capture expressions are evaluated in the
    /// closure-creation environment and precede no implicit/dynamic bindings.
    LexicalClosure {
        captures: Vec<RuntimeExpr>,
        params: Vec<String>,
        body: Box<RuntimeExpr>,
    },
    DeclarationRef {
        symbol: RuntimeSymbol,
    },
    ImportedDeclarationRef {
        symbol: RuntimeSymbol,
        dependency: RuntimeSymbol,
        dependency_semantic_hash: String,
    },
    Call {
        callee: Box<RuntimeExpr>,
        args: Vec<RuntimeExpr>,
    },
    Effect {
        family: RuntimeSymbol,
        operation: ken_host::HostOpV1,
        capability: Option<RuntimeCapabilityUse>,
        args: Vec<RuntimeExpr>,
    },
    Trap(RuntimeTrap),
}

/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub struct RuntimeMatchCase {
    pub constructor: RuntimeSymbol,
    pub binders: usize,
    pub body: RuntimeExpr,
}

/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub struct RuntimeComputationalMatchCase {
    pub constructor: RuntimeSymbol,
    pub argument_binders: usize,
    pub recursive_positions: Vec<usize>,
    pub body: RuntimeExpr,
}

/// The closure-free **header** of one ordinary eliminator case.
///
/// ⭐ This carrier exists so the frame-fingerprint core has a parameter type
/// that **excludes closures by construction** (`dec_16n1t4b92463g`, route C).
/// It deliberately has **no `body` field** — a [`RuntimeMatchCase`]'s `body` is
/// an unrestricted [`RuntimeExpr`] and may contain `Closure`,
/// `LexicalClosure`, or `Value(ClosureRef)`, and
/// `spec/40-runtime/41-values.md §2.1` denies ordinary closures structural
/// equality, ordering, and a canonical hash. Hashing a `Debug` rendering of a
/// full case is that same forbidden equality verdict under another spelling.
///
/// ⚠ Adding a body-bearing field here re-opens the defect. See
/// [`MatchFrameHeaders`] for the `AC-F4` controls that make that a **compile**
/// failure rather than a review obligation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrdinaryFrameHeader<'a> {
    pub constructor: &'a RuntimeSymbol,
    pub binders: usize,
}

/// The closure-free **header** of one computational eliminator case.
///
/// See [`OrdinaryFrameHeader`] — same rationale, plus the recursive-position
/// vector that distinguishes a computational frame's induction hypotheses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComputationalFrameHeader<'a> {
    pub constructor: &'a RuntimeSymbol,
    pub argument_binders: usize,
    pub recursive_positions: &'a [usize],
}

/// The fingerprint core's **only** input shape, and its explicit
/// ordinary/computational domain separator.
///
/// ⛔ **No [`RuntimeExpr`], [`RuntimeValue`], case `body`, full case value, or
/// `Debug` rendering of any such value may reach the core.** The enum has
/// exactly two arms, each carrying a closure-free header slice, so that
/// requirement is discharged by the type rather than by a convention.
///
/// # `AC-F4` — a full case CANNOT reach the core
///
/// ⚠ **These blocks carry no `EXXXX` code, and that is deliberate.** A
/// ```` ```compile_fail,E0277 ```` fence passes when the block fails to compile
/// for **any** reason — rustdoc does not bind the code — so an error-code
/// annotation reads as a reason-pin and is only documentation. Attribution
/// here comes from the **compiling sibling** in the last block, which shares
/// every import and constructor with the negatives and differs from them by
/// exactly the forbidden operation.
///
/// **MEASURED:** each negative block fails to compile, for some reason.
/// **CLAIMED:** it fails *because a body-bearing type is not a header*.
/// **THE GAP:** closed by the sibling — a malformed fixture (bad path, missing
/// import, wrong literal) reddens *it* instead of silently greening these.
///
/// An ordinary **case slice** is not a header slice:
///
/// ```compile_fail
/// use ken_runtime::{MatchFrameHeaders, RuntimeMatchCase, RuntimeTrap, RuntimeTrapCode};
/// let cases: Vec<RuntimeMatchCase> = Vec::new();
/// let default = RuntimeTrap {
///     code: RuntimeTrapCode::PatternMatchFailure,
///     message: String::new(),
/// };
/// let _ = ken_runtime::compiler_private_match_frame_header_fingerprint(
///     MatchFrameHeaders::Ordinary(&cases),
///     &default,
/// );
/// ```
///
/// A computational **case slice** is not a header slice:
///
/// ```compile_fail
/// use ken_runtime::{MatchFrameHeaders, RuntimeComputationalMatchCase, RuntimeTrap, RuntimeTrapCode};
/// let cases: Vec<RuntimeComputationalMatchCase> = Vec::new();
/// let default = RuntimeTrap {
///     code: RuntimeTrapCode::PatternMatchFailure,
///     message: String::new(),
/// };
/// let _ = ken_runtime::compiler_private_match_frame_header_fingerprint(
///     MatchFrameHeaders::Computational(&cases),
///     &default,
/// );
/// ```
///
/// The header carrier has **no body-bearing field** to smuggle one through:
///
/// ```compile_fail
/// use ken_runtime::{OrdinaryFrameHeader, RuntimeExpr, RuntimeTrapCode, RuntimeTrap};
/// let constructor = String::from("Cons");
/// let _ = OrdinaryFrameHeader {
///     constructor: &constructor,
///     binders: 2,
///     body: RuntimeExpr::Trap(RuntimeTrap {
///         code: RuntimeTrapCode::ExplicitTrap,
///         message: String::new(),
///     }),
/// };
/// ```
///
/// ⭐ **The sibling** — identical imports and constructors, differing only in
/// that the operand is a genuine header sequence. It **must compile and run**:
///
/// ```
/// use ken_runtime::{MatchFrameHeaders, OrdinaryFrameHeader, RuntimeTrap, RuntimeTrapCode};
/// let constructor = String::from("Cons");
/// let headers = vec![OrdinaryFrameHeader { constructor: &constructor, binders: 2 }];
/// let default = RuntimeTrap {
///     code: RuntimeTrapCode::PatternMatchFailure,
///     message: String::new(),
/// };
/// let _ = ken_runtime::compiler_private_match_frame_header_fingerprint(
///     MatchFrameHeaders::Ordinary(&headers),
///     &default,
/// );
/// ```
#[derive(Clone, Copy, Debug)]
pub enum MatchFrameHeaders<'a> {
    Ordinary(&'a [OrdinaryFrameHeader<'a>]),
    Computational(&'a [ComputationalFrameHeader<'a>]),
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// Length-prefixed, so no two distinct field sequences can share an encoding by
/// running their bytes together at a boundary.
fn push_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    push_u64(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

/// Exhaustive and **wildcard-free**: a new [`RuntimeTrapCode`] variant must
/// stop this compiling rather than silently colliding with an existing tag.
fn runtime_trap_code_tag(code: &RuntimeTrapCode) -> u64 {
    match code {
        RuntimeTrapCode::UnsupportedErasure => 0,
        RuntimeTrapCode::UnsupportedPrimitivePartiality => 1,
        RuntimeTrapCode::MissingRuntimeMetadata => 2,
        RuntimeTrapCode::PatternMatchFailure => 3,
        RuntimeTrapCode::ExplicitTrap => 4,
    }
}

/// The frame-fingerprint **core** — the operation that forms the equality
/// verdict, and the only place the hash is computed.
///
/// It consumes one ordered [`MatchFrameHeaders`] sequence plus the closure-free
/// [`RuntimeTrap`] default. ⛔ Nothing body-bearing can be passed; see
/// [`MatchFrameHeaders`] for the `AC-F4` compile-failure controls.
///
/// ⚠ **Body-change staleness is deliberately NOT an invariant of this
/// fingerprint** (`dec_16n1t4b92463g`). `site_id` plus the closure-free checked
/// occurrence binding remains the authoritative identity; this value checks
/// only eliminator-family / header / default compatibility. A body change at
/// the same checked occurrence is program semantics, not a licence to inspect
/// forbidden closure structure.
#[doc(hidden)]
pub fn compiler_private_match_frame_header_fingerprint(
    headers: MatchFrameHeaders<'_>,
    default: &RuntimeTrap,
) -> u64 {
    let mut encoded = Vec::new();
    match headers {
        MatchFrameHeaders::Ordinary(headers) => {
            push_len_prefixed(&mut encoded, b"ordinary");
            push_u64(&mut encoded, headers.len() as u64);
            for header in headers {
                push_len_prefixed(&mut encoded, header.constructor.as_bytes());
                push_u64(&mut encoded, header.binders as u64);
            }
        }
        MatchFrameHeaders::Computational(headers) => {
            push_len_prefixed(&mut encoded, b"computational");
            push_u64(&mut encoded, headers.len() as u64);
            for header in headers {
                push_len_prefixed(&mut encoded, header.constructor.as_bytes());
                push_u64(&mut encoded, header.argument_binders as u64);
                push_u64(&mut encoded, header.recursive_positions.len() as u64);
                for position in header.recursive_positions {
                    push_u64(&mut encoded, *position as u64);
                }
            }
        }
    }
    push_u64(&mut encoded, runtime_trap_code_tag(&default.code));
    push_len_prefixed(&mut encoded, default.message.as_bytes());
    crate::fnv1a_64(&encoded)
}

/// Compiler-private structural identity for one erased ordinary eliminator
/// frame.  The checked join plan binds this fingerprint to a distinct checked
/// occurrence; native lowering refuses ambiguity rather than treating equal
/// frame shapes as interchangeable sites.
///
/// ⭐ **This is the projecting wrapper, and it keeps the existing signature.**
/// It accepts the full case slice **solely to project** each case into
/// [`OrdinaryFrameHeader`] and call
/// [`compiler_private_match_frame_header_fingerprint`]. ⛔ It must not itself
/// serialize, hash, or compare full cases.
#[doc(hidden)]
pub fn compiler_private_ordinary_match_frame_fingerprint(
    cases: &[RuntimeMatchCase],
    default: &RuntimeTrap,
) -> u64 {
    let headers = cases
        .iter()
        .map(|case| OrdinaryFrameHeader {
            constructor: &case.constructor,
            binders: case.binders,
        })
        .collect::<Vec<_>>();
    compiler_private_match_frame_header_fingerprint(MatchFrameHeaders::Ordinary(&headers), default)
}

/// Compiler-private structural identity for one erased computational
/// eliminator frame.  See
/// [`compiler_private_ordinary_match_frame_fingerprint`] — this is the same
/// projecting wrapper for the computational domain.
#[doc(hidden)]
pub fn compiler_private_computational_match_frame_fingerprint(
    cases: &[RuntimeComputationalMatchCase],
    default: &RuntimeTrap,
) -> u64 {
    let headers = cases
        .iter()
        .map(|case| ComputationalFrameHeader {
            constructor: &case.constructor,
            argument_binders: case.argument_binders,
            recursive_positions: &case.recursive_positions,
        })
        .collect::<Vec<_>>();
    compiler_private_match_frame_header_fingerprint(
        MatchFrameHeaders::Computational(&headers),
        default,
    )
}

/// The **operational** carrier — where an ordinary closure lives.
///
/// ⛔ **`PartialEq`/`Eq` are deliberately NOT derived, and that is `D2`.** This
/// enum has a `ClosureRef` arm, and `spec/40-runtime/41-values.md §2.1` denies
/// ordinary closures structural equality. A blanket derive would have granted
/// Ken-semantic equality to *every* arm — closures included — merely because
/// they share one Rust enum, which is the "half an AC that reads as whole"
/// failure `AC-V4` names.
///
/// ⭐ **The property is reachability, not the absence of a caller:** generic
/// code requiring `PartialEq<RuntimeValue>` **fails to compile**. There is no
/// detector to evade, because the capability is absent from the type.
///
/// ⚠ Comparison of *closure-free* runtime values is still available and
/// explicitly named: [`RuntimeGroundValue`] and [`RuntimeObservation`] keep
/// their derives, because neither has a closure arm. Route a comparison through
/// those rather than re-adding one here.
///
/// # `AC-V4` — the forbidden capabilities are UNREACHABLE on this carrier
///
/// ⭐ Same discipline as [`crate::values::Value`]'s block, and it is required
/// **separately**: discharging `AC-V4` on the canonical carrier alone is "half
/// an AC that reads as whole," because that carrier has no closure arm to
/// begin with while this one does. ⚠ The subject here is therefore a
/// `ClosureRef` in every block — the forbidden *value*, not the enum in
/// general.
///
/// ⛔ **The `EXXXX` codes are documentation, not a check** — rustdoc was
/// measured not to bind them (see [`crate::values::Value`]'s block for the
/// probe). Reason-attribution here comes from the two siblings below.
///
/// **No structural equality (`D2` — the derive was removed):**
///
/// ```compile_fail,E0277
/// use ken_runtime::RuntimeValue;
/// fn requires_eq<T: PartialEq>(_: &T) {}
/// let c = RuntimeValue::ClosureRef { symbol: "f".to_string(), captured: vec![] };
/// requires_eq(&c);
/// ```
///
/// **No ordering:**
///
/// ```compile_fail,E0277
/// use ken_runtime::RuntimeValue;
/// fn requires_ord<T: Ord>(_: &T) {}
/// let c = RuntimeValue::ClosureRef { symbol: "f".to_string(), captured: vec![] };
/// requires_ord(&c);
/// ```
///
/// **No canonical hash:**
///
/// ```compile_fail,E0277
/// use ken_runtime::RuntimeValue;
/// fn requires_hash<T: std::hash::Hash>(_: &T) {}
/// let c = RuntimeValue::ClosureRef { symbol: "f".to_string(), captured: vec![] };
/// requires_hash(&c);
/// ```
///
/// ⚠ **Stated honestly:** only the first of those three was ever derived here.
/// `Ord` and `Hash` were never present, so their blocks pin an absence that
/// `D2` did not create — they are kept because `AC-V4` names all three
/// capabilities, and an unpinned one is indistinguishable from an unexamined
/// one.
///
/// ⭐ **Two siblings that MUST compile.** ⛔ A `compile_fail` block passes for
/// *any* compilation error, so the negative blocks above are worthless alone.
/// The first sibling is the same fixture with the bound check removed, which is
/// what makes each failure attributable to the missing impl rather than to a
/// malformed `ClosureRef` literal:
///
/// ```
/// use ken_runtime::RuntimeValue;
/// let c = RuntimeValue::ClosureRef { symbol: "f".to_string(), captured: vec![] };
/// assert!(matches!(c, RuntimeValue::ClosureRef { .. }));
/// ```
///
/// The second shows the three capabilities are genuinely **available** for a
/// closure-free operational value, through the one sanctioned route —
/// [`crate::canonical::project_operational_to_canonical`] onto the sealed
/// witness. ⇒ The carrier is not merely comparison-hostile; it routes:
///
/// ```
/// use ken_runtime::RuntimeValue;
/// use ken_runtime::canonical::{project_operational_to_canonical, CanonicalWitness};
/// fn requires_eq<T: PartialEq>(_: &T) {}
/// fn requires_ord<T: Ord>(_: &T) {}
/// fn requires_hash<T: std::hash::Hash>(_: &T) {}
/// let mut intern = |s: &str| s.len() as u32;
/// let free = RuntimeValue::Record {
///     fields: vec![("n".to_string(), RuntimeValue::Bool(true))],
/// };
/// let w = CanonicalWitness::of(
///     &project_operational_to_canonical(&free, &mut intern).expect("closure-free"),
/// );
/// requires_eq(&w);
/// requires_ord(&w);
/// requires_hash(&w);
/// let again = CanonicalWitness::of(
///     &project_operational_to_canonical(&free, &mut intern).expect("closure-free"),
/// );
/// assert_eq!(w, again);
/// ```
#[derive(Clone, Debug)]
pub enum RuntimeValue {
    Bool(bool),
    Int(crate::RuntimeIntV1),
    Bytes(Vec<u8>),
    String(String),
    Constructor {
        constructor: RuntimeSymbol,
        args: Vec<RuntimeValue>,
    },
    Record {
        fields: Vec<(String, RuntimeValue)>,
    },
    ClosureRef {
        symbol: RuntimeSymbol,
        captured: Vec<RuntimeValue>,
    },
    Unknown,
}

/// The only NC5 comparison observations: returned ground values or traps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeObservation {
    Returned(RuntimeGroundValue),
    Trapped(RuntimeTrap),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeGroundValue {
    Bool(bool),
    Int(crate::RuntimeIntV1),
    Bytes(Vec<u8>),
    String(String),
    Constructor {
        constructor: RuntimeSymbol,
        args: Vec<RuntimeGroundValue>,
    },
    Record {
        fields: Vec<(String, RuntimeGroundValue)>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTrap {
    pub code: RuntimeTrapCode,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeTrapCode {
    UnsupportedErasure,
    UnsupportedPrimitivePartiality,
    MissingRuntimeMetadata,
    PatternMatchFailure,
    ExplicitTrap,
}

/// ⚠ **`PartialEq`/`Eq` are not derived** — this type transitively contains
/// [`RuntimeValue`], whose closure arm may not expose structural equality
/// (`spec/40-runtime/41-values.md §2.1`, `D2`). Compare a closure-free
/// projection such as [`RuntimeGroundValue`] instead.
#[derive(Clone, Debug)]
pub struct RuntimeExample {
    pub name: String,
    pub checked_core_shape: String,
    pub ir: RuntimeExpr,
    pub observation: RuntimeObservation,
}

pub fn nc5_seed_examples() -> Vec<RuntimeExample> {
    vec![
        RuntimeExample {
            name: "closed-scalar-primitive".to_string(),
            checked_core_shape: "add_int 2 3".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "add_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((3).into())),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((5).into())),
        },
        RuntimeExample {
            name: "adt-constructor-match".to_string(),
            checked_core_shape: "match Some 4 with Some x => x | None => 0".to_string(),
            ir: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::Core::Option::Some".to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int((4).into()))],
                }),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:fixture::Core::Option::Some".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "no Option case selected".to_string(),
                },
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((4).into())),
        },
        RuntimeExample {
            name: "closure-capture-application".to_string(),
            checked_core_shape: "let y = 2 in (\\x . add_int x y) 5".to_string(),
            ir: RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Closure {
                    captures: vec!["decl:fixture::Local::y".to_string()],
                    params: vec!["x".to_string()],
                    body: Box::new(RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "add_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
                    }),
                }),
                args: vec![RuntimeExpr::Value(RuntimeValue::Int((5).into()))],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((7).into())),
        },
        RuntimeExample {
            name: "record-construction-projection".to_string(),
            checked_core_shape: "({ left = 1, right = 2 }).right".to_string(),
            ir: RuntimeExpr::Project {
                record: Box::new(RuntimeExpr::Record {
                    fields: vec![
                        (
                            "left".to_string(),
                            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                        ),
                        (
                            "right".to_string(),
                            RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                        ),
                    ],
                }),
                field: "right".to_string(),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
        },
        RuntimeExample {
            name: "explicit-partial-primitive-trap".to_string(),
            checked_core_shape: "checked_index empty 0".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "checked_index".to_string(),
                    partiality: RuntimePartiality::CheckedTrap {
                        obligation: "obl:checked_index.bounds".to_string(),
                    },
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Bytes(Vec::new())),
                    RuntimeExpr::Value(RuntimeValue::Int((0).into())),
                ],
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "checked_index bounds obligation failed".to_string(),
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_examples_are_observation_limited() {
        // The exhaustive match below (no `_` arm) is what actually pins
        // "observation-limited": adding a third `RuntimeObservation` variant
        // is a compile error here, not a silent gap. The seed corpus itself
        // is a growable illustrative set, so this only asserts it is
        // non-vacuous (the loop below must actually exercise something), not
        // any particular size (Q-RESIDUE: `examples.len() == 5` froze a
        // growable corpus at today's size for no semantic reason).
        let examples = nc5_seed_examples();
        assert!(
            !examples.is_empty(),
            "the seed corpus must not be vacuously empty"
        );
        for example in examples {
            match example.observation {
                RuntimeObservation::Returned(_) | RuntimeObservation::Trapped(_) => {}
            }
        }
    }

    #[test]
    fn primitive_partiality_is_explicit_in_ir() {
        let partial = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == "explicit-partial-primitive-trap")
            .expect("partial primitive example present");

        let RuntimeExpr::PrimitiveCall { primitive, .. } = partial.ir else {
            panic!("partial primitive example must be a primitive call");
        };
        assert!(matches!(
            primitive.partiality,
            RuntimePartiality::CheckedTrap { .. }
        ));
        assert!(matches!(
            partial.observation,
            RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                ..
            })
        ));
    }

    #[test]
    fn ir_has_no_backend_layout_surface() {
        let program = RuntimeProgram {
            package_identity: "module:fixture::nc5".to_string(),
            core_semantic_hash: 1,
            artifact_hash: 2,
            erased_core: ErasedExecutableCore {
                symbols: BTreeSet::from(["decl:fixture::Main::f".to_string()]),
                metadata: RuntimeMetadata::default(),
            },
            declarations: vec![RuntimeDeclaration {
                symbol: "decl:fixture::Main::f".to_string(),
                kind: RuntimeDeclarationKind::Transparent {
                    body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                },
                metadata: RuntimeSymbolMetadata::empty(),
            }],
            examples: nc5_seed_examples(),
        };

        assert_eq!(program.package_identity, "module:fixture::nc5");
        assert_eq!(program.declarations[0].symbol, "decl:fixture::Main::f");
    }

    // ---- RT-MATCH-FRAME-FP: `AC-F1`–`AC-F3` --------------------------------
    //
    // `AC-F4` is a *compile* failure and therefore cannot live here; it is the
    // doc-test block on `MatchFrameHeaders`, which `--doc` runs.

    /// A body that genuinely contains a closure — the value class
    /// `spec/40-runtime/41-values.md §2.1` denies structural equality.
    fn closure_body(param: &str) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec![param.to_string()],
            body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((0).into()))),
        }
    }

    fn trap(message: &str) -> RuntimeTrap {
        RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: message.to_string(),
        }
    }

    fn ordinary_case(constructor: &str, binders: usize, body: RuntimeExpr) -> RuntimeMatchCase {
        RuntimeMatchCase {
            constructor: constructor.to_string(),
            binders,
            body,
        }
    }

    fn computational_case(
        constructor: &str,
        argument_binders: usize,
        recursive_positions: Vec<usize>,
        body: RuntimeExpr,
    ) -> RuntimeComputationalMatchCase {
        RuntimeComputationalMatchCase {
            constructor: constructor.to_string(),
            argument_binders,
            recursive_positions,
            body,
        }
    }

    /// The **retired** route, kept only as this file's non-vacuity oracle: it
    /// is what the fingerprint used to be, and it *is* sensitive to the body.
    ///
    /// ⛔ Not a production path. It exists so `AC-F1` can prove its two frames
    /// genuinely differ — otherwise "the body is ignored" and "the fixture
    /// never varied the body" are the same green.
    fn debug_rendering_differs<T: std::fmt::Debug>(left: &T, right: &T) -> bool {
        format!("{left:?}") != format!("{right:?}")
    }

    #[test]
    fn acf1_ordinary_frames_differing_only_in_a_closure_body_share_a_fingerprint() {
        let left = vec![ordinary_case("Cons", 2, closure_body("x"))];
        let right = vec![ordinary_case("Cons", 2, closure_body("y"))];
        assert!(
            debug_rendering_differs(&left, &right),
            "positive control: the two bodies must actually differ, or AC-F1 is vacuous"
        );

        let default = trap("no match");
        assert_eq!(
            compiler_private_ordinary_match_frame_fingerprint(&left, &default),
            compiler_private_ordinary_match_frame_fingerprint(&right, &default),
            "AC-F1: the header carrier must not observe a closure-bearing body"
        );
    }

    #[test]
    fn acf1_computational_frames_differing_only_in_a_closure_body_share_a_fingerprint() {
        let left = vec![computational_case("Succ", 1, vec![0], closure_body("x"))];
        let right = vec![computational_case("Succ", 1, vec![0], closure_body("y"))];
        assert!(
            debug_rendering_differs(&left, &right),
            "positive control: the two bodies must actually differ, or AC-F1 is vacuous"
        );

        let default = trap("no match");
        assert_eq!(
            compiler_private_computational_match_frame_fingerprint(&left, &default),
            compiler_private_computational_match_frame_fingerprint(&right, &default),
            "AC-F1: the header carrier must not observe a closure-bearing body"
        );
    }

    // `AC-F2` — one mutation per load-bearing field, each fired SEPARATELY.
    // ⛔ Deliberately not an aggregate "the header matters" pass: a single
    // field silently dropped from the encoding survives that.

    #[test]
    fn acf2_ordinary_constructor_is_load_bearing() {
        let base = vec![ordinary_case("Cons", 2, closure_body("x"))];
        let mutated = vec![ordinary_case("Nil", 2, closure_body("x"))];
        let default = trap("no match");
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&base, &default),
            compiler_private_ordinary_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_ordinary_binders_is_load_bearing() {
        let base = vec![ordinary_case("Cons", 2, closure_body("x"))];
        let mutated = vec![ordinary_case("Cons", 3, closure_body("x"))];
        let default = trap("no match");
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&base, &default),
            compiler_private_ordinary_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_ordinary_case_order_is_load_bearing() {
        let base = vec![
            ordinary_case("Cons", 2, closure_body("x")),
            ordinary_case("Nil", 0, closure_body("x")),
        ];
        let mutated = vec![
            ordinary_case("Nil", 0, closure_body("x")),
            ordinary_case("Cons", 2, closure_body("x")),
        ];
        let default = trap("no match");
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&base, &default),
            compiler_private_ordinary_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_ordinary_case_count_is_load_bearing() {
        let base = vec![ordinary_case("Cons", 2, closure_body("x"))];
        let mutated = vec![
            ordinary_case("Cons", 2, closure_body("x")),
            ordinary_case("Nil", 0, closure_body("x")),
        ];
        let default = trap("no match");
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&base, &default),
            compiler_private_ordinary_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_default_trap_code_is_load_bearing() {
        let cases = vec![ordinary_case("Cons", 2, closure_body("x"))];
        let base = trap("no match");
        let mutated = RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: base.message.clone(),
        };
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&cases, &base),
            compiler_private_ordinary_match_frame_fingerprint(&cases, &mutated),
            "AC-F2: the default trap's CODE is a load-bearing field"
        );
    }

    #[test]
    fn acf2_default_trap_message_is_load_bearing() {
        let cases = vec![ordinary_case("Cons", 2, closure_body("x"))];
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&cases, &trap("no match")),
            compiler_private_ordinary_match_frame_fingerprint(&cases, &trap("other")),
            "AC-F2: the default trap's MESSAGE is a load-bearing field"
        );
    }

    #[test]
    fn acf2_computational_constructor_is_load_bearing() {
        let base = vec![computational_case("Succ", 1, vec![0], closure_body("x"))];
        let mutated = vec![computational_case("Zero", 1, vec![0], closure_body("x"))];
        let default = trap("no match");
        assert_ne!(
            compiler_private_computational_match_frame_fingerprint(&base, &default),
            compiler_private_computational_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_computational_argument_binders_is_load_bearing() {
        let base = vec![computational_case("Succ", 1, vec![0], closure_body("x"))];
        let mutated = vec![computational_case("Succ", 2, vec![0], closure_body("x"))];
        let default = trap("no match");
        assert_ne!(
            compiler_private_computational_match_frame_fingerprint(&base, &default),
            compiler_private_computational_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_computational_recursive_position_values_are_load_bearing() {
        let base = vec![computational_case("Node", 2, vec![0], closure_body("x"))];
        let mutated = vec![computational_case("Node", 2, vec![1], closure_body("x"))];
        let default = trap("no match");
        assert_ne!(
            compiler_private_computational_match_frame_fingerprint(&base, &default),
            compiler_private_computational_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf2_computational_recursive_position_arity_is_load_bearing() {
        let base = vec![computational_case("Node", 2, vec![0], closure_body("x"))];
        let mutated = vec![computational_case("Node", 2, vec![0, 1], closure_body("x"))];
        let default = trap("no match");
        assert_ne!(
            compiler_private_computational_match_frame_fingerprint(&base, &default),
            compiler_private_computational_match_frame_fingerprint(&mutated, &default),
        );
    }

    #[test]
    fn acf3_ordinary_and_computational_frames_are_domain_separated() {
        // Coinciding field-for-field: same constructor, `binders ==
        // argument_binders`, no recursive positions, same default.
        let default = trap("no match");
        let ordinary = vec![ordinary_case("Cons", 2, closure_body("x"))];
        let computational = vec![computational_case("Cons", 2, Vec::new(), closure_body("x"))];
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&ordinary, &default),
            compiler_private_computational_match_frame_fingerprint(&computational, &default),
            "AC-F3: the two eliminator families must not collide"
        );
    }

    #[test]
    fn frame_header_encoding_is_unambiguous_at_field_boundaries() {
        // A count-and-concatenate encoding would let `["ab"]` and `["a","b"]`
        // (or a constructor absorbing the next field's bytes) collide. The
        // length prefix is what forbids that, so pin it directly rather than
        // trusting that no such pair exists.
        let default = trap("");
        let joined = vec![ordinary_case("ab", 0, closure_body("x"))];
        let split = vec![
            ordinary_case("a", 0, closure_body("x")),
            ordinary_case("b", 0, closure_body("x")),
        ];
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&joined, &default),
            compiler_private_ordinary_match_frame_fingerprint(&split, &default),
        );

        // The constructor must not be able to swallow the default message.
        let short = vec![ordinary_case("Cons", 0, closure_body("x"))];
        let absorbed = vec![ordinary_case("Constail", 0, closure_body("x"))];
        assert_ne!(
            compiler_private_ordinary_match_frame_fingerprint(&short, &trap("tail")),
            compiler_private_ordinary_match_frame_fingerprint(&absorbed, &trap("")),
        );
    }
}
