use std::collections::{BTreeMap, BTreeSet};

use ken_elaborator::checked_core::{
    canonical_decl_bytes, emit_checked_core_package, CheckedCoreArtifactInputs, CheckedCorePackage,
    CheckedCorePackageHeader, CheckedCoreSemanticInputs, ClassInstanceKind, ClassInstanceMetadata,
    LowerabilityStatus, PartialityMetadata, PrimitiveMetadata, PrimitiveReductionMetadata,
    RecursionAdmission, RecursionMetadata, StableSymbol, StableSymbolTable, SymbolNamespace,
};
use ken_elaborator::erasure::{erase_checked_core_package_for_target, ErasureError};
use ken_kernel::{Decl, GlobalId, Level, Term};
use ken_runtime::{
    evaluate_runtime_ir_example, RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExample,
    RuntimeExpr, RuntimeGroundValue, RuntimeIrSeedEnvironment, RuntimeLowerabilityStatus,
    RuntimeMetadata, RuntimeObservation, RuntimePartiality, RuntimePrimitive, RuntimeProgram,
    RuntimeSymbolMetadata, RuntimeValue,
};

fn decl_symbol(package: &str, name: &str) -> StableSymbol {
    StableSymbol::declaration(package, &[], name)
}

fn header(package: &str) -> CheckedCorePackageHeader {
    CheckedCorePackageHeader::v0(
        "ken-elaborator:recursion-dictionaries-modules-test",
        "ken-kernel:test",
        "docs/program/wp/NC17-recursion-dictionaries-modules.md",
        "spec/40-runtime/46-checked-core-package.md:test",
        StableSymbol::new(SymbolNamespace::Module, vec![package.to_string()]),
    )
}

fn table_many(entries: &[(GlobalId, StableSymbol)]) -> StableSymbolTable {
    let mut table = StableSymbolTable::new();
    for (id, symbol) in entries {
        table.insert_global(*id, symbol.clone());
    }
    table
}

fn transparent(id: GlobalId, body: Term) -> Decl {
    Decl::Transparent {
        id,
        level_params: Vec::new(),
        ty: Term::Type(Level::zero()),
        body,
    }
}

fn reemit(mut package: CheckedCorePackage) -> CheckedCorePackage {
    package.header.dependency_semantic_hashes =
        package.artifact.semantic.dependency_semantic_hashes.clone();
    emit_checked_core_package(package.header, package.artifact).expect("package re-emits")
}

fn lowered_body(program: &RuntimeProgram, symbol: &StableSymbol) -> RuntimeExpr {
    let declaration = program
        .declarations
        .iter()
        .find(|declaration| declaration.symbol == symbol.to_string())
        .expect("lowered declaration exists");
    let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
        panic!("expected transparent runtime declaration, got {declaration:?}");
    };
    body.clone()
}

fn recursive_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
    let package = "nc17_recursive_pkg";
    let target = decl_symbol(package, "loop_ref");
    let group = decl_symbol(package, "loop_ref.group");
    let table = table_many(&[(GlobalId(1), target.clone())]);
    let mut semantic = CheckedCoreSemanticInputs::default();
    for symbol in [&target, &group] {
        semantic.symbols.insert(symbol.clone());
        semantic
            .lowerability
            .insert(symbol.clone(), LowerabilityStatus::Supported);
    }
    semantic.declarations.insert(
        target.clone(),
        canonical_decl_bytes(
            &transparent(
                GlobalId(1),
                Term::Const {
                    id: GlobalId(1),
                    level_args: Vec::new(),
                },
            ),
            &table,
        )
        .expect("canonical recursive declaration"),
    );
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
        header(package),
        CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    )
    .expect("checked-core package emits");
    (package, target, group)
}

fn imported_package() -> (
    CheckedCorePackage,
    StableSymbol,
    StableSymbol,
    StableSymbol,
    String,
) {
    let package = "nc17_import_pkg";
    let target = decl_symbol(package, "target");
    let imported = StableSymbol::declaration("dep-pkg", &["Dep"], "answer");
    let dependency = StableSymbol::new(
        SymbolNamespace::Dependency,
        vec!["dep-pkg".to_string(), "checked-core".to_string()],
    );
    let dependency_hash = "sha256:dep-answer-v1".to_string();
    let table = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(90), imported.clone()),
    ]);
    let mut semantic = CheckedCoreSemanticInputs::default();
    for symbol in [&target, &imported, &dependency] {
        semantic.symbols.insert(symbol.clone());
    }
    for symbol in [&target, &imported] {
        semantic
            .lowerability
            .insert(symbol.clone(), LowerabilityStatus::Supported);
    }
    semantic.declarations.insert(
        target.clone(),
        canonical_decl_bytes(
            &transparent(
                GlobalId(1),
                Term::Const {
                    id: GlobalId(90),
                    level_args: Vec::new(),
                },
            ),
            &table,
        )
        .expect("canonical import declaration"),
    );
    semantic
        .dependency_semantic_hashes
        .insert(dependency.clone(), dependency_hash.clone());
    semantic
        .dependency_declaration_refs
        .insert(imported.clone(), dependency.clone());

    let package = reemit(CheckedCorePackage {
        header: header(package),
        core_semantic_hash: 0,
        artifact_hash: 0,
        artifact: CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    });
    (package, target, imported, dependency, dependency_hash)
}

fn dictionary_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
    let package = "nc17_dictionary_pkg";
    let target = decl_symbol(package, "target");
    let dictionary = decl_symbol(package, "EqBoolDict");
    let class = decl_symbol(package, "Eq");
    let head = decl_symbol(package, "Bool");
    let literal = StableSymbol::primitive("lit_int_7");
    let table = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(10), dictionary.clone()),
        (GlobalId(40), literal.clone()),
    ]);
    let mut semantic = CheckedCoreSemanticInputs::default();
    for symbol in [&target, &dictionary, &class, &head, &literal] {
        semantic.symbols.insert(symbol.clone());
    }
    for symbol in [&target, &dictionary, &literal] {
        semantic
            .lowerability
            .insert(symbol.clone(), LowerabilityStatus::Supported);
    }
    semantic.declarations.insert(
        target.clone(),
        canonical_decl_bytes(
            &Decl::Transparent {
                id: GlobalId(1),
                level_params: Vec::new(),
                ty: Term::Const {
                    id: GlobalId(10),
                    level_args: Vec::new(),
                },
                body: Term::Pair(
                    Box::new(Term::Const {
                        id: GlobalId(40),
                        level_args: Vec::new(),
                    }),
                    Box::new(Term::Pair(
                        Box::new(Term::Omega(Level::zero())),
                        Box::new(Term::Type(Level::zero())),
                    )),
                ),
            },
            &table,
        )
        .expect("canonical dictionary declaration"),
    );
    semantic
        .primitive_refs
        .insert(literal.clone(), "primitive-registry:lit_int_7".to_string());
    semantic.primitive_metadata.insert(
        literal,
        PrimitiveMetadata {
            registry_symbol: "lit_int_7".to_string(),
            reduction: PrimitiveReductionMetadata::Literal,
            partiality: PartialityMetadata::Total,
            lowerability: LowerabilityStatus::Supported,
        },
    );
    semantic.class_instance_metadata.insert(
        dictionary.clone(),
        ClassInstanceMetadata {
            kind: ClassInstanceKind::Dictionary,
            class_symbol: Some(class),
            dictionary_symbol: Some(dictionary.clone()),
            head_symbol: Some(head),
            field_order: vec!["value".to_string(), "law".to_string()],
            runtime_fields: BTreeSet::from(["value".to_string()]),
            law_fields: BTreeSet::from(["law".to_string()]),
            lowerability: LowerabilityStatus::Supported,
        },
    );
    let package = emit_checked_core_package(
        header(package),
        CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    )
    .expect("checked-core package emits");
    (package, target, dictionary)
}

fn supported_metadata() -> RuntimeSymbolMetadata {
    let mut metadata = RuntimeSymbolMetadata::empty();
    metadata.lowerability = Some(RuntimeLowerabilityStatus::Supported);
    metadata.obligations = BTreeSet::new();
    metadata
}

fn recursive_runtime_program() -> (RuntimeProgram, RuntimeExample) {
    let symbol = "decl:nc17_runtime::sum_to".to_string();
    let recursive_body = RuntimeExpr::Closure {
        captures: Vec::new(),
        params: vec!["n".to_string()],
        body: Box::new(RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "eq_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Var(0),
                    RuntimeExpr::Value(RuntimeValue::Int((0).into())),
                ],
            }),
            then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((0).into()))),
            else_expr: Box::new(RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "add_int".to_string(),
                    partiality: RuntimePartiality::Total,
                },
                args: vec![
                    RuntimeExpr::Var(0),
                    RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::DeclarationRef {
                            symbol: symbol.clone(),
                        }),
                        args: vec![RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "sub_int".to_string(),
                                partiality: RuntimePartiality::Total,
                            },
                            args: vec![
                                RuntimeExpr::Var(0),
                                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                            ],
                        }],
                    },
                ],
            }),
        }),
    };
    let example = RuntimeExample {
        name: "nc17-recursive-sum-to".to_string(),
        checked_core_shape: "sum_to 4".to_string(),
        ir: RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: symbol.clone(),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((4).into()))],
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((10).into())),
    };
    let program = RuntimeProgram {
        package_identity: "module:nc17_runtime".to_string(),
        core_semantic_hash: 17,
        artifact_hash: 18,
        erased_core: ken_runtime::ErasedExecutableCore {
            symbols: BTreeSet::from([symbol.clone()]),
            metadata: RuntimeMetadata {
                lowerability: BTreeMap::from([(
                    symbol.clone(),
                    RuntimeLowerabilityStatus::Supported,
                )]),
                runtime_declaration_targets: BTreeSet::from([symbol.clone()]),
                ..RuntimeMetadata::default()
            },
        },
        declarations: vec![RuntimeDeclaration {
            symbol: symbol.clone(),
            kind: RuntimeDeclarationKind::Transparent {
                body: recursive_body,
            },
            metadata: supported_metadata(),
        }],
        examples: vec![example.clone()],
    };
    (program, example)
}

#[test]
fn recursive_body_view_lowers_to_explicit_runtime_declaration_ref() {
    let (package, target, _group) = recursive_package();
    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("recursive declaration call lowers to runtime ref");
    let body = lowered_body(&program, &target);

    assert_eq!(
        body,
        RuntimeExpr::DeclarationRef {
            symbol: target.to_string()
        }
    );
}

#[test]
fn runtime_evaluator_executes_terminating_recursive_declaration_ref() {
    let (program, example) = recursive_runtime_program();

    let report =
        evaluate_runtime_ir_example(&program, &example, &RuntimeIrSeedEnvironment::empty())
            .expect("program-aware runtime evaluator executes recursive declaration ref");

    assert_eq!(report.observation.observation, example.observation);
}

#[test]
fn imported_declaration_ref_requires_exact_dependency_seed_identity() {
    let (mut package, target, imported, dependency, dependency_hash) = imported_package();
    let mut program = erase_checked_core_package_for_target(&package, [&target])
        .expect("imported declaration call lowers");
    let body = lowered_body(&program, &target);
    assert_eq!(
        body,
        RuntimeExpr::ImportedDeclarationRef {
            symbol: imported.to_string(),
            dependency: dependency.to_string(),
            dependency_semantic_hash: dependency_hash.clone(),
        }
    );

    let example = RuntimeExample {
        name: "nc17-imported-answer".to_string(),
        checked_core_shape: "dep-pkg.Dep.answer".to_string(),
        ir: body,
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into())),
    };
    program.examples.push(example.clone());
    let mut env = RuntimeIrSeedEnvironment::empty();
    env.insert_imported_declaration(
        imported.to_string(),
        dependency.to_string(),
        dependency_hash.clone(),
        RuntimeGroundValue::Int((9).into()),
    );
    let report = evaluate_runtime_ir_example(&program, &example, &env)
        .expect("exact dependency seed identity evaluates imported ref");
    assert_eq!(report.observation.observation, example.observation);

    let mut stale_env = RuntimeIrSeedEnvironment::empty();
    stale_env.insert_imported_declaration(
        imported.to_string(),
        dependency.to_string(),
        "sha256:stale",
        RuntimeGroundValue::Int((9).into()),
    );
    let err = evaluate_runtime_ir_example(&program, &example, &stale_env)
        .expect_err("stale dependency seed identity must reject loudly");
    assert_eq!(err.construct, "ImportedDeclarationRef");
    assert!(err.reason.contains(&dependency_hash));

    let mut blocked_package = package.clone();
    blocked_package.artifact.semantic.lowerability.insert(
        imported.clone(),
        LowerabilityStatus::Unsupported {
            reason: "qa probe imported declaration is not executable".to_string(),
        },
    );
    blocked_package = reemit(blocked_package);
    let err = erase_checked_core_package_for_target(&blocked_package, [&target])
        .expect_err("blocking imported declaration lowerability must reject before runtime IR");
    match err {
        ErasureError::ExpressionLowering { lane, .. } => {
            assert_eq!(lane, "imported_declaration_lowerability_blocked");
        }
        other => panic!("expected imported lowerability lane, got {other:?}"),
    }

    let mut blocked_program = program.clone();
    blocked_program.erased_core.metadata.lowerability.insert(
        imported.to_string(),
        RuntimeLowerabilityStatus::Unsupported {
            reason: "qa probe imported declaration is not executable".to_string(),
        },
    );
    let err = evaluate_runtime_ir_example(&blocked_program, &example, &env)
        .expect_err("runtime preflight must reject blocking imported lowerability");
    assert_eq!(err.construct, "ImportedDeclarationRef");
    assert!(err.reason.contains("blocking lowerability"));

    package
        .artifact
        .semantic
        .dependency_declaration_refs
        .clear();
    package = reemit(package);
    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("missing dependency identity must fail before runtime IR success");
    match err {
        ErasureError::ExpressionLowering { lane, .. } => {
            assert_eq!(lane, "missing_dependency_identity");
        }
        other => panic!("expected missing dependency identity lane, got {other:?}"),
    }
}

#[test]
fn dictionary_construction_lowers_only_runtime_fields_to_record_values() {
    let (package, target, dictionary) = dictionary_package();
    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("dictionary construction lowers");
    let body = lowered_body(&program, &target);

    let observation =
        ken_runtime::evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
            .expect("dictionary runtime record evaluates");
    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Record {
            fields: vec![("value".to_string(), RuntimeGroundValue::Int((7).into()))]
        })
    );

    let metadata = program
        .erased_core
        .metadata
        .checked_core
        .class_instance_metadata
        .get(&dictionary.to_string())
        .expect("dictionary audit metadata survives");
    assert_eq!(
        metadata.runtime_fields,
        BTreeSet::from(["value".to_string()])
    );
    assert_eq!(metadata.law_fields, BTreeSet::from(["law".to_string()]));
}
