use std::collections::BTreeMap;

use ken_elaborator::checked_core::{
    canonical_decl_bytes, emit_checked_core_package, CheckedCoreArtifactInputs, CheckedCorePackage,
    CheckedCorePackageHeader, CheckedCoreSemanticInputs, LowerabilityStatus, StableSymbol,
    StableSymbolTable, SymbolNamespace,
};
use ken_elaborator::compiler_driver::{
    compile_ken_package_sources, CompilerManifest, CompilerSource, CompilerTargetKind,
    TargetSelector,
};
use ken_elaborator::erasure::{erase_checked_core_package_for_target, ErasureError};
use ken_kernel::{Decl, GlobalId, Level, Term};
use ken_runtime::{
    evaluate_runtime_ir_expr, RuntimeDeclarationKind, RuntimeExpr, RuntimeGroundValue,
    RuntimeIrSeedEnvironment, RuntimeObservation, RuntimeValue,
};

fn decl_symbol(package: &str, name: &str) -> StableSymbol {
    StableSymbol::declaration(package, &[], name)
}

fn header(package: &str) -> CheckedCorePackageHeader {
    CheckedCorePackageHeader::v0(
        "ken-elaborator:core-expression-lowering-test",
        "ken-kernel:test",
        "docs/program/wp/NC13-core-expression-lowering.md",
        "spec/10-kernel/18a-primitive-registry.md:test",
        StableSymbol::new(SymbolNamespace::Module, vec![package.to_string()]),
    )
}

fn table(entries: &[(GlobalId, StableSymbol)]) -> StableSymbolTable {
    let mut table = StableSymbolTable::new();
    for (id, symbol) in entries {
        table.insert_global(*id, symbol.clone());
    }
    table
}

fn transparent(id: GlobalId, body: Term) -> Decl {
    let ty = Term::Type(Level::zero());
    Decl::Transparent {
        id,
        level_params: Vec::new(),
        ty: ty.clone(),
        body,
    }
}

fn package_from_decls(
    package: &str,
    decls: Vec<(StableSymbol, Decl)>,
    symbols: &StableSymbolTable,
) -> CheckedCorePackage {
    let mut semantic = CheckedCoreSemanticInputs::default();
    for (symbol, decl) in decls {
        semantic.symbols.insert(symbol.clone());
        semantic
            .lowerability
            .insert(symbol.clone(), LowerabilityStatus::Supported);
        semantic.declarations.insert(
            symbol,
            canonical_decl_bytes(&decl, symbols).expect("canonical declaration bytes"),
        );
    }
    emit_checked_core_package(
        header(package),
        CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    )
    .expect("checked-core package emits")
}

fn expression_package() -> (CheckedCorePackage, StableSymbol, StableSymbol) {
    let package = "expr_pkg";
    let helper = decl_symbol(package, "helper");
    let target = decl_symbol(package, "target");
    let helper_id = GlobalId(1);
    let target_id = GlobalId(2);
    let table = table(&[(helper_id, helper.clone()), (target_id, target.clone())]);
    let ty = Term::Type(Level::zero());
    let helper_decl = transparent(
        helper_id,
        Term::Lam(Box::new(ty.clone()), Box::new(Term::Var(0))),
    );
    let target_decl = transparent(
        target_id,
        Term::Let {
            ty: Box::new(ty),
            val: Box::new(Term::Const {
                id: helper_id,
                level_args: Vec::new(),
            }),
            body: Box::new(Term::Var(0)),
        },
    );

    (
        package_from_decls(
            package,
            vec![(helper.clone(), helper_decl), (target.clone(), target_decl)],
            &table,
        ),
        target,
        helper,
    )
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

#[test]
fn source_derived_closed_function_target_lowers_to_runtime_ir() {
    let package_name = "source_expr_pkg";
    let target = decl_symbol(package_name, "target");
    let out = compile_ken_package_sources(
        &CompilerManifest::new(package_name, Vec::new()),
        vec![CompilerSource::new(
            "src/main.ken",
            "fn helper (x : Bool) : Bool = x\nconst target : Bool -> Bool = helper",
        )],
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
            .expect("source-derived closed function target lowers");
    let body = lowered_body(&program, &target);

    let observation = evaluate_runtime_ir_expr(
        &RuntimeExpr::Call {
            callee: Box::new(body),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((11).into()))],
        },
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("source-derived generated runtime IR evaluates");

    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int((11).into()))
    );
    assert_eq!(
        program.package_identity,
        out.package.header.package_identity.to_string()
    );
    assert_eq!(program.core_semantic_hash, out.package.core_semantic_hash);
    assert_eq!(program.artifact_hash, out.package.artifact_hash);
}

#[test]
fn transparent_declaration_body_lowers_and_evaluates_through_runtime_ir() {
    let (package, target, helper) = expression_package();
    let program = erase_checked_core_package_for_target(&package, [&target, &helper])
        .expect("transparent declarations lower");
    let body = lowered_body(&program, &target);

    let observation = evaluate_runtime_ir_expr(
        &RuntimeExpr::Call {
            callee: Box::new(body),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((9).into()))],
        },
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("generated runtime IR evaluates");

    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into()))
    );
    assert_eq!(
        program.package_identity,
        package.header.package_identity.to_string()
    );
    assert_eq!(program.core_semantic_hash, package.core_semantic_hash);
    assert_eq!(program.artifact_hash, package.artifact_hash);
    assert!(program
        .erased_core
        .metadata
        .runtime_declaration_targets
        .contains(&target.to_string()));
    assert!(program
        .erased_core
        .metadata
        .runtime_declaration_targets
        .contains(&helper.to_string()));
}

#[test]
fn unresolved_direct_call_rejects_with_body_view_lane() {
    let (package, target, _) = expression_package();

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("missing helper in selected closure must reject");

    assert!(matches!(
        err,
        ErasureError::ExpressionLowering { symbol, lane, .. }
            if symbol == target && lane == "body_reference_outside_selected_closure"
    ));
}

#[test]
fn unbound_variable_rejects_before_runtime_program_success() {
    let package = "unbound_pkg";
    let target = decl_symbol(package, "target");
    let target_id = GlobalId(1);
    let table = table(&[(target_id, target.clone())]);
    let package = package_from_decls(
        package,
        vec![(target.clone(), transparent(target_id, Term::Var(0)))],
        &table,
    );

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("top-level de Bruijn variable is unbound");

    assert!(matches!(
        err,
        ErasureError::ExpressionLowering { symbol, lane, .. }
            if symbol == target && lane == "unbound_de_bruijn_variable"
    ));
}

#[test]
fn implicit_lexical_capture_rejects_loudly() {
    let package = "capture_pkg";
    let target = decl_symbol(package, "target");
    let target_id = GlobalId(1);
    let table = table(&[(target_id, target.clone())]);
    let ty = Term::Type(Level::zero());
    let package = package_from_decls(
        package,
        vec![(
            target.clone(),
            transparent(target_id, Term::Lam(Box::new(ty), Box::new(Term::Var(1)))),
        )],
        &table,
    );

    let err = erase_checked_core_package_for_target(&package, [&target])
        .expect_err("implicit closure capture must not be silently lowered");

    assert!(matches!(
        err,
        ErasureError::ExpressionLowering { symbol, lane, .. }
            if symbol == target && lane == "implicit_closure_capture"
    ));
}

#[test]
fn mutual_direct_call_cycle_rejects_loudly() {
    let package = "cycle_pkg";
    let left = decl_symbol(package, "left");
    let right = decl_symbol(package, "right");
    let left_id = GlobalId(1);
    let right_id = GlobalId(2);
    let table = table(&[(left_id, left.clone()), (right_id, right.clone())]);
    let left_decl = transparent(
        left_id,
        Term::Const {
            id: right_id,
            level_args: Vec::new(),
        },
    );
    let right_decl = transparent(
        right_id,
        Term::Const {
            id: left_id,
            level_args: Vec::new(),
        },
    );
    let package = package_from_decls(
        package,
        vec![(left.clone(), left_decl), (right.clone(), right_decl)],
        &table,
    );

    let err = erase_checked_core_package_for_target(&package, [&left, &right])
        .expect_err("direct-call cycles must be explicit");

    assert!(matches!(
        err,
        ErasureError::ExpressionLowering { symbol, lane, .. }
            if symbol == left && lane == "direct_call_cycle"
    ));
}

#[test]
fn generated_closure_arity_mismatch_stays_runtime_ir_error() {
    let (package, target, helper) = expression_package();
    let program = erase_checked_core_package_for_target(&package, [&target, &helper])
        .expect("transparent declarations lower");
    let body = lowered_body(&program, &target);

    let err = evaluate_runtime_ir_expr(
        &RuntimeExpr::Call {
            callee: Box::new(body),
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                RuntimeExpr::Value(RuntimeValue::Int((2).into())),
            ],
        },
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect_err("generated one-argument closure must reject two runtime args");

    assert_eq!(err.construct, "Call");
    assert!(err.reason.contains("expects 1 args"));
}
