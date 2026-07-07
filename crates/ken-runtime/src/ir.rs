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
    pub assumptions: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub trusted_base_delta: BTreeMap<RuntimeSymbol, Vec<u8>>,
    pub dependency_semantic_hashes: BTreeMap<RuntimeSymbol, String>,
    pub runtime_checks: BTreeSet<RuntimeSymbol>,
    pub capabilities: BTreeSet<RuntimeSymbol>,
    pub effects: BTreeSet<String>,
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
    pub assumptions: BTreeSet<RuntimeSymbol>,
    pub trusted_base_delta: BTreeSet<RuntimeSymbol>,
    pub runtime_checks: BTreeSet<RuntimeSymbol>,
    pub capabilities: BTreeSet<RuntimeSymbol>,
    pub effects: BTreeSet<String>,
}

impl RuntimeSymbolMetadata {
    pub fn empty() -> Self {
        Self {
            obligations: BTreeSet::new(),
            assumptions: BTreeSet::new(),
            trusted_base_delta: BTreeSet::new(),
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
    CheckedTrap { obligation: RuntimeSymbol },
    TrustedTrap { assumption: RuntimeSymbol },
}

/// Backend-neutral runtime expression language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeExpr {
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
    Call {
        callee: Box<RuntimeExpr>,
        args: Vec<RuntimeExpr>,
    },
    Effect {
        effect: String,
        capability: Option<RuntimeSymbol>,
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
pub enum RuntimeValue {
    Bool(bool),
    Int(i64),
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
    Int(i64),
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
                    RuntimeExpr::Value(RuntimeValue::Int(2)),
                    RuntimeExpr::Value(RuntimeValue::Int(3)),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(5)),
        },
        RuntimeExample {
            name: "adt-constructor-match".to_string(),
            checked_core_shape: "match Some 4 with Some x => x | None => 0".to_string(),
            ir: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::Core::Option::Some".to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(4))],
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
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(4)),
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
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(5))],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(7)),
        },
        RuntimeExample {
            name: "record-construction-projection".to_string(),
            checked_core_shape: "({ left = 1, right = 2 }).right".to_string(),
            ir: RuntimeExpr::Project {
                record: Box::new(RuntimeExpr::Record {
                    fields: vec![
                        ("left".to_string(), RuntimeExpr::Value(RuntimeValue::Int(1))),
                        (
                            "right".to_string(),
                            RuntimeExpr::Value(RuntimeValue::Int(2)),
                        ),
                    ],
                }),
                field: "right".to_string(),
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(2)),
        },
        RuntimeExample {
            name: "explicit-partial-primitive-trap".to_string(),
            checked_core_shape: "bytes_at empty 0".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_at".to_string(),
                    partiality: RuntimePartiality::CheckedTrap {
                        obligation: "obl:bytes_at.bounds".to_string(),
                    },
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Bytes(Vec::new())),
                    RuntimeExpr::Value(RuntimeValue::Int(0)),
                ],
            },
            observation: RuntimeObservation::Trapped(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "bytes_at bounds obligation failed".to_string(),
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
                    body: RuntimeExpr::Value(RuntimeValue::Int(1)),
                },
                metadata: RuntimeSymbolMetadata::empty(),
            }],
            examples: nc5_seed_examples(),
        };

        assert_eq!(program.package_identity, "module:fixture::nc5");
        assert_eq!(program.declarations[0].symbol, "decl:fixture::Main::f");
    }
}
