use std::collections::BTreeMap;

use ken_elaborator::checked_core::{
    canonical_decl_bytes, emit_checked_core_package, CheckedCoreArtifactInputs, CheckedCorePackage,
    CheckedCorePackageHeader, CheckedCoreSemanticInputs, ConstructorMetadata, DataMetadata,
    LowerabilityStatus, StableSymbol, StableSymbolTable, SymbolNamespace,
};
use ken_elaborator::compiler_driver::{
    compile_ken_package_sources, CompilerManifest, CompilerSource, CompilerTargetKind,
    TargetSelector,
};
use ken_elaborator::erasure::{erase_checked_core_package_for_target, ErasureError};
use ken_elaborator::ElabEnv;
use ken_interp::eval::{eval, EvalStore, EvalVal};
use ken_kernel::{Decl, GlobalId, Level, Term};
use ken_runtime::{
    evaluate_runtime_ir_expr, RuntimeDeclarationKind, RuntimeExpr, RuntimeGroundValue,
    RuntimeIrSeedEnvironment, RuntimeObservation,
};

fn decl_symbol(package: &str, name: &str) -> StableSymbol {
    StableSymbol::declaration(package, &[], name)
}

fn header(package: &str) -> CheckedCorePackageHeader {
    CheckedCorePackageHeader::v0(
        "ken-elaborator:data-match-lowering-test",
        "ken-kernel:test",
        "docs/program/wp/NC14-data-match-lowering.md",
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

fn lowered_body(program: &ken_runtime::RuntimeProgram, symbol: &StableSymbol) -> RuntimeExpr {
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

fn runtime_observation_for_source(
    package_name: &str,
    target_name: &str,
    source: &str,
) -> RuntimeObservation {
    let target = decl_symbol(package_name, target_name);
    let out = compile_ken_package_sources(
        &CompilerManifest::new(package_name, Vec::new()),
        vec![CompilerSource::new("src/main.ken", source)],
        TargetSelector::StableSymbol {
            package_identity: StableSymbol::new(
                SymbolNamespace::Module,
                vec![package_name.to_string()],
            ),
            symbol: target.clone(),
            kind: CompilerTargetKind::Executable,
        },
    )
    .expect("source emits checked-core package and selected closure");
    let closure = out.closures.first().expect("selected target closure");
    let program =
        erase_checked_core_package_for_target(&out.package, closure.reachable_declarations.iter())
            .expect("source-derived data/match target lowers");
    let body = lowered_body(&program, &target);
    evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
        .expect("generated runtime IR evaluates")
}

fn interpreter_bool_for_source(source: &str, target_name: &str) -> bool {
    let mut env = ElabEnv::new().expect("prelude env");
    env.elaborate_file(source).expect("source elaborates");
    let target = env.globals[target_name];
    let body = match env.env.lookup(target) {
        Some(Decl::Transparent { body, .. }) => body.clone(),
        other => panic!("expected transparent target, got {:?}", other.map(|_| ())),
    };
    let mut store = EvalStore::new();
    match eval(&[], &body, &env.env, &mut store) {
        EvalVal::Ctor { id, args, .. } if id == env.globals["True"] && args.is_empty() => true,
        EvalVal::Ctor { id, args, .. } if id == env.globals["False"] && args.is_empty() => false,
        other => panic!("expected interpreter Bool constructor, got {other:?}"),
    }
}

fn assert_runtime_bool(observation: RuntimeObservation, expected: bool) {
    let expected_name = if expected { "::True" } else { "::False" };
    match observation {
        RuntimeObservation::Returned(RuntimeGroundValue::Constructor { constructor, args }) => {
            assert!(
                constructor.ends_with(expected_name),
                "expected runtime Bool constructor ending in {expected_name}, got {constructor}"
            );
            assert!(
                args.is_empty(),
                "Bool constructor must not carry runtime args"
            );
        }
        other => panic!("expected runtime Bool constructor observation, got {other:?}"),
    }
}

fn assert_runtime_and_interpreter_bool_agree(package_name: &str, target_name: &str, source: &str) {
    let interpreter = interpreter_bool_for_source(source, target_name);
    let runtime = runtime_observation_for_source(package_name, target_name, source);
    assert_runtime_bool(runtime, interpreter);
}

#[test]
fn option_match_payload_binding_lowers_and_matches_interpreter() {
    let source = "const target : Bool = \
        match Some Bool True { None |-> False ; Some x |-> x }";

    assert_runtime_and_interpreter_bool_agree("nc14_option_pkg", "target", source);
}

#[test]
fn result_match_payload_binding_lowers_and_matches_interpreter() {
    let source = "const target : Bool = \
        match Ok Unit Bool True { Err e |-> False ; Ok x |-> x }";

    assert_runtime_and_interpreter_bool_agree("nc14_result_pkg", "target", source);
}

#[test]
fn user_data_two_payload_binders_preserve_de_bruijn_order() {
    let source = "data PairBool = PairBoolMk Bool Bool\n\
        const target : Bool = \
        match PairBoolMk True False { PairBoolMk x y |-> x }";

    assert_runtime_and_interpreter_bool_agree("nc14_pair_pkg", "target", source);
}

fn bool_data_symbols(package: &str) -> (StableSymbol, StableSymbol, StableSymbol) {
    let bool_ty = decl_symbol(package, "Bool");
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

fn data_match_package(package_name: &str) -> (CheckedCorePackage, StableSymbol) {
    let target = decl_symbol(package_name, "target");
    let target_id = GlobalId(1);
    let bool_id = GlobalId(10);
    let false_id = GlobalId(11);
    let true_id = GlobalId(12);
    let (bool_ty, false_ctor, true_ctor) = bool_data_symbols(package_name);
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
    let body = Term::Elim {
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
        body,
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
    let package = emit_checked_core_package(
        header(package_name),
        CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    )
    .unwrap();
    (package, target)
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
    let header = package.header.clone();
    let artifact = package.artifact.clone();
    *package = emit_checked_core_package(header, artifact).expect("package re-emits");
}

fn erasure_lane(package: &CheckedCorePackage, target: &StableSymbol) -> &'static str {
    let err = erase_checked_core_package_for_target(package, [target])
        .expect_err("package must reject before runtime program success");
    match err {
        ErasureError::ExpressionLowering { lane, .. } => lane,
        other => panic!("expected expression lowering error, got {other:?}"),
    }
}

#[test]
fn stale_constructor_identity_rejects_before_runtime_program_success() {
    let (mut package, target) = data_match_package("nc14_stale_ctor_pkg");
    let ghost = StableSymbol::constructor(&decl_symbol("nc14_stale_ctor_pkg", "Ghost"), "MadeUp");
    let table = table_many(&[(GlobalId(1), target.clone()), (GlobalId(99), ghost)]);
    replace_body(
        &mut package,
        &target,
        Term::Constructor {
            id: GlobalId(99),
            level_args: Vec::new(),
        },
        &table,
    );

    assert_eq!(
        erasure_lane(&package, &target),
        "stale_constructor_identity"
    );
}

#[test]
fn missing_match_branch_data_rejects_before_runtime_program_success() {
    let (mut package, target) = data_match_package("nc14_missing_branch_pkg");
    let (bool_ty, false_ctor, true_ctor) = bool_data_symbols("nc14_missing_branch_pkg");
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

    assert_eq!(erasure_lane(&package, &target), "missing_match_branch_data");
}

#[test]
fn unsupported_dependent_motive_rejects_before_runtime_program_success() {
    let (mut package, target) = data_match_package("nc14_dependent_motive_pkg");
    let (bool_ty, _, _) = bool_data_symbols("nc14_dependent_motive_pkg");
    package
        .artifact
        .semantic
        .data_metadata
        .get_mut(&bool_ty)
        .unwrap()
        .index_count = 1;
    let header = package.header.clone();
    let artifact = package.artifact.clone();
    let package = emit_checked_core_package(header, artifact).expect("package re-emits");

    assert_eq!(
        erasure_lane(&package, &target),
        "unsupported_dependent_motive"
    );
}

#[test]
fn unsupported_proof_only_match_rejects_before_runtime_program_success() {
    let (mut package, target) = data_match_package("nc14_proof_match_pkg");
    let (bool_ty, false_ctor, true_ctor) = bool_data_symbols("nc14_proof_match_pkg");
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

    assert_eq!(
        erasure_lane(&package, &target),
        "unsupported_proof_only_match"
    );
}

#[test]
fn impossible_branch_misuse_rejects_before_runtime_program_success() {
    let (mut package, target) = data_match_package("nc14_impossible_branch_pkg");
    let (bool_ty, false_ctor, true_ctor) = bool_data_symbols("nc14_impossible_branch_pkg");
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

    assert_eq!(
        erasure_lane(&package, &target),
        "unjustified_impossible_branch"
    );
}
