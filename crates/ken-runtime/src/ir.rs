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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeCapabilityUse {
    pub identity: RuntimeSymbol,
    pub value: Box<RuntimeExpr>,
}

/// Complete NC5 runtime artifact for one checked-core package subset.
#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeclaration {
    pub symbol: RuntimeSymbol,
    pub kind: RuntimeDeclarationKind,
    pub metadata: RuntimeSymbolMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeExpr {
    #[doc(hidden)]
    CheckedJoinSite {
        site_id: u64,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMatchCase {
    pub constructor: RuntimeSymbol,
    pub binders: usize,
    pub body: RuntimeExpr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeComputationalMatchCase {
    pub constructor: RuntimeSymbol,
    pub argument_binders: usize,
    pub recursive_positions: Vec<usize>,
    pub body: RuntimeExpr,
}

/// Compiler-private structural identity for one erased ordinary eliminator
/// frame.  The checked join plan binds this fingerprint to a distinct checked
/// occurrence; native lowering refuses ambiguity rather than treating equal
/// frame shapes as interchangeable sites.
#[doc(hidden)]
pub fn compiler_private_ordinary_match_frame_fingerprint(
    cases: &[RuntimeMatchCase],
    default: &RuntimeTrap,
) -> u64 {
    crate::fnv1a_64(format!("ordinary\0{cases:?}\0{default:?}").as_bytes())
}

/// Compiler-private structural identity for one erased computational
/// eliminator frame.  See
/// [`compiler_private_ordinary_match_frame_fingerprint`].
#[doc(hidden)]
pub fn compiler_private_computational_match_frame_fingerprint(
    cases: &[RuntimeComputationalMatchCase],
    default: &RuntimeTrap,
) -> u64 {
    crate::fnv1a_64(format!("computational\0{cases:?}\0{default:?}").as_bytes())
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
        let examples = nc5_seed_examples();
        assert_eq!(examples.len(), 5);
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
}
