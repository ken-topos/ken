use std::collections::BTreeMap;

use ken_elaborator::checked_core::{
    canonical_decl_bytes, emit_checked_core_package, CheckedCoreArtifactInputs, CheckedCorePackage,
    CheckedCorePackageHeader, CheckedCoreSemanticInputs, LowerabilityStatus, PartialityMetadata,
    PrimitiveMetadata, PrimitiveReductionMetadata, StableSymbol, StableSymbolTable,
    SymbolNamespace,
};
use ken_elaborator::erasure::{erase_checked_core_package_for_target, ErasureError};
use ken_interp::{eval, EvalStore, EvalVal};
use ken_kernel::{declare_primitive, Decl, GlobalEnv, GlobalId, Level, PrimReduction, Term};
use ken_runtime::{
    compare_runtime_ir_with_interpreter_observation, evaluate_runtime_ir_expr,
    RuntimeArtifactIdentity, RuntimeDeclarationKind, RuntimeExample, RuntimeGroundValue,
    RuntimeInterpreterObservation, RuntimeIrDifferentialStage, RuntimeIrDifferentialVerdict,
    RuntimeIrSeedEnvironment, RuntimeIrTargetIdentity, RuntimeObservation, RuntimePartiality,
    RuntimePrimitive,
};

fn decl_symbol(package: &str, name: &str) -> StableSymbol {
    StableSymbol::declaration(package, &[], name)
}

fn header(package: &str) -> CheckedCorePackageHeader {
    CheckedCorePackageHeader::v0(
        "ken-elaborator:primitive-value-lowering-test",
        "ken-kernel:test",
        "docs/program/wp/NC16-primitive-value-lowering.md",
        "spec/10-kernel/18a-primitive-registry.md:test",
        StableSymbol::new(SymbolNamespace::Module, vec![package.to_string()]),
    )
}

fn stable_table(entries: &[(GlobalId, StableSymbol)]) -> StableSymbolTable {
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

fn primitive_const(id: GlobalId) -> Term {
    Term::Const {
        id,
        level_args: Vec::new(),
    }
}

fn app2(function: Term, left: Term, right: Term) -> Term {
    Term::App(
        Box::new(Term::App(Box::new(function), Box::new(left))),
        Box::new(right),
    )
}

fn add_primitive(
    semantic: &mut CheckedCoreSemanticInputs,
    registry_symbol: &str,
    reduction: PrimitiveReductionMetadata,
    partiality: PartialityMetadata,
    lowerability: LowerabilityStatus,
) -> StableSymbol {
    let symbol = StableSymbol::primitive(registry_symbol);
    semantic.symbols.insert(symbol.clone());
    semantic
        .lowerability
        .insert(symbol.clone(), lowerability.clone());
    semantic.primitive_refs.insert(
        symbol.clone(),
        format!("primitive-registry:{registry_symbol}"),
    );
    semantic.primitive_metadata.insert(
        symbol.clone(),
        PrimitiveMetadata {
            registry_symbol: registry_symbol.to_string(),
            reduction,
            partiality,
            lowerability,
        },
    );
    symbol
}

fn primitive_package(
    package: &str,
    body: Term,
    table: &StableSymbolTable,
    primitives: impl IntoIterator<
        Item = (
            &'static str,
            PrimitiveReductionMetadata,
            PartialityMetadata,
            LowerabilityStatus,
        ),
    >,
) -> (CheckedCorePackage, StableSymbol) {
    let target = decl_symbol(package, "target");
    let mut semantic = CheckedCoreSemanticInputs::default();
    semantic.symbols.insert(target.clone());
    semantic
        .lowerability
        .insert(target.clone(), LowerabilityStatus::Supported);
    semantic.declarations.insert(
        target.clone(),
        canonical_decl_bytes(&transparent(GlobalId(1), body), table)
            .expect("canonical primitive test declaration"),
    );
    for (registry_symbol, reduction, partiality, lowerability) in primitives {
        add_primitive(
            &mut semantic,
            registry_symbol,
            reduction,
            partiality,
            lowerability,
        );
    }

    let package = emit_checked_core_package(
        header(package),
        CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    )
    .expect("checked-core package emits");
    (package, target)
}

fn lowered_body(
    program: &ken_runtime::RuntimeProgram,
    symbol: &StableSymbol,
) -> ken_runtime::RuntimeExpr {
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

fn erasure_lane(package: &CheckedCorePackage, target: &StableSymbol) -> &'static str {
    let err = erase_checked_core_package_for_target(package, [target])
        .expect_err("package must reject before runtime program success");
    match err {
        ErasureError::ExpressionLowering { lane, .. } => lane,
        other => panic!("expected expression lowering error, got {other:?}"),
    }
}

fn interpreter_add_2_3_observation() -> RuntimeObservation {
    let mut globals = GlobalEnv::new();
    let int_id = declare_primitive(
        &mut globals,
        vec![],
        Term::Type(Level::zero()),
        PrimReduction::OpaqueType,
    )
    .expect("Int primitive type");
    let int_ty = Term::Const {
        id: int_id,
        level_args: Vec::new(),
    };
    let add_id = declare_primitive(
        &mut globals,
        vec![],
        Term::pi(int_ty.clone(), Term::pi(int_ty.clone(), int_ty.clone())),
        PrimReduction::Op { symbol: "add_int" },
    )
    .expect("add_int primitive");
    let lit_2 = declare_primitive(&mut globals, vec![], int_ty.clone(), PrimReduction::Literal)
        .expect("literal 2");
    let lit_3 =
        declare_primitive(&mut globals, vec![], int_ty, PrimReduction::Literal).expect("literal 3");

    let mut store = EvalStore::new();
    store.num_values.insert(lit_2, EvalVal::Int(2));
    store.num_values.insert(lit_3, EvalVal::Int(3));

    let term = app2(
        primitive_const(add_id),
        primitive_const(lit_2),
        primitive_const(lit_3),
    );
    match eval(&[], &term, &globals, &mut store) {
        EvalVal::Int(value) => RuntimeObservation::Returned(RuntimeGroundValue::Int(value)),
        other => panic!("interpreter fixture must return Int, got {other:?}"),
    }
}

#[test]
fn primitive_integer_application_lowers_evaluates_and_reports_agreement() {
    let package_name = "nc16_int_pkg";
    let target = decl_symbol(package_name, "target");
    let lit_two = StableSymbol::primitive("lit_int_2");
    let lit_three = StableSymbol::primitive("lit_int_3");
    let add = StableSymbol::primitive("add_int");
    let table = stable_table(&[
        (GlobalId(1), target.clone()),
        (GlobalId(40), lit_two),
        (GlobalId(41), lit_three),
        (GlobalId(42), add),
    ]);
    let (package, target) = primitive_package(
        package_name,
        app2(
            primitive_const(GlobalId(42)),
            primitive_const(GlobalId(40)),
            primitive_const(GlobalId(41)),
        ),
        &table,
        [
            (
                "lit_int_2",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "lit_int_3",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "add_int",
                PrimitiveReductionMetadata::Op,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
        ],
    );

    let mut program = erase_checked_core_package_for_target(&package, [&target])
        .expect("primitive integer body lowers");
    let body = lowered_body(&program, &target);

    let observation = evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
        .expect("primitive integer runtime IR evaluates");
    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(5))
    );
    let example = RuntimeExample {
        name: "nc16-int-add".to_string(),
        checked_core_shape: "add_int lit_int_2 lit_int_3".to_string(),
        ir: body,
        observation: observation.clone(),
    };
    program.examples.push(example.clone());
    let interpreter = RuntimeInterpreterObservation {
        artifact: RuntimeArtifactIdentity::from_program(&program),
        target: RuntimeIrTargetIdentity::from_example(&example),
        observation: interpreter_add_2_3_observation(),
        evidence_source: "ken-interp eval over GlobalEnv + closed core Term: add_int 2 3"
            .to_string(),
    };

    let report = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        interpreter,
    );

    assert_eq!(
        report.verdict,
        RuntimeIrDifferentialVerdict::InterpreterRuntimeIrAgreement {
            stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
        }
    );
}

#[test]
fn primitive_bool_string_and_bytes_values_lower_and_evaluate() {
    let package_name = "nc16_values_pkg";
    let target = decl_symbol(package_name, "target");
    let lit_string = StableSymbol::primitive("lit_string_ken");
    let byte_length = StableSymbol::primitive("byte_length");
    let table = stable_table(&[
        (GlobalId(1), target.clone()),
        (GlobalId(40), lit_string),
        (GlobalId(41), byte_length),
    ]);
    let (package, target) = primitive_package(
        package_name,
        Term::App(
            Box::new(primitive_const(GlobalId(41))),
            Box::new(primitive_const(GlobalId(40))),
        ),
        &table,
        [
            (
                "lit_string_ken",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "byte_length",
                PrimitiveReductionMetadata::Op,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
        ],
    );
    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("primitive string body lowers");
    let body = lowered_body(&program, &target);
    let observation = evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
        .expect("primitive string runtime IR evaluates");
    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(3))
    );

    let package_name = "nc16_bytes_pkg";
    let target = decl_symbol(package_name, "target");
    let left = StableSymbol::primitive("lit_bytes_hex_0a");
    let right = StableSymbol::primitive("lit_bytes_hex_ff");
    let concat = StableSymbol::primitive("bytes_concat");
    let table = stable_table(&[
        (GlobalId(1), target.clone()),
        (GlobalId(40), left),
        (GlobalId(41), right),
        (GlobalId(42), concat),
    ]);
    let (package, target) = primitive_package(
        package_name,
        app2(
            primitive_const(GlobalId(42)),
            primitive_const(GlobalId(40)),
            primitive_const(GlobalId(41)),
        ),
        &table,
        [
            (
                "lit_bytes_hex_0a",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "lit_bytes_hex_ff",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "bytes_concat",
                PrimitiveReductionMetadata::Op,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
        ],
    );
    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("primitive bytes body lowers");
    let body = lowered_body(&program, &target);
    let observation = evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
        .expect("primitive bytes runtime IR evaluates");
    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Bytes(vec![0x0a, 0xff]))
    );

    let package_name = "nc16_bool_pkg";
    let target = decl_symbol(package_name, "target");
    let lit_true = StableSymbol::primitive("lit_bool_true");
    let lit_false = StableSymbol::primitive("lit_bool_false");
    let and_bool = StableSymbol::primitive("and_bool");
    let table = stable_table(&[
        (GlobalId(1), target.clone()),
        (GlobalId(40), lit_true),
        (GlobalId(41), lit_false),
        (GlobalId(42), and_bool),
    ]);
    let (package, target) = primitive_package(
        package_name,
        app2(
            primitive_const(GlobalId(42)),
            primitive_const(GlobalId(40)),
            primitive_const(GlobalId(41)),
        ),
        &table,
        [
            (
                "lit_bool_true",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "lit_bool_false",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "and_bool",
                PrimitiveReductionMetadata::Op,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
        ],
    );
    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("primitive bool body lowers");
    let body = lowered_body(&program, &target);
    let observation = evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
        .expect("primitive bool runtime IR evaluates");
    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Bool(false))
    );
}

#[test]
fn safe_bytes_at_lowering_returns_none_and_carries_bounds_obligation() {
    let package_name = "nc16_partial_pkg";
    let target = decl_symbol(package_name, "target");
    let empty = StableSymbol::primitive("lit_bytes_hex_");
    let zero = StableSymbol::primitive("lit_int_0");
    let bytes_at = StableSymbol::primitive("bytes_at");
    let table = stable_table(&[
        (GlobalId(1), target.clone()),
        (GlobalId(40), empty),
        (GlobalId(41), zero),
        (GlobalId(42), bytes_at.clone()),
    ]);
    let obligation = StableSymbol::obligation("bytes_at.bounds");
    let (package, target) = primitive_package(
        package_name,
        app2(
            primitive_const(GlobalId(42)),
            primitive_const(GlobalId(40)),
            primitive_const(GlobalId(41)),
        ),
        &table,
        [
            (
                "lit_bytes_hex_",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "lit_int_0",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "bytes_at",
                PrimitiveReductionMetadata::Op,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
        ],
    );

    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("safe bytes_at primitive body lowers");
    let body = lowered_body(&program, &target);

    let observation = evaluate_runtime_ir_expr(&body, &RuntimeIrSeedEnvironment::empty())
        .expect("safe bytes_at evaluates to None");
    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
            constructor: "ctor:nc16_partial_pkg::Option::None".to_string(),
            args: Vec::new(),
        })
    );
    let ken_runtime::RuntimeExpr::PrimitiveCall { primitive, .. } = body else {
        panic!("expected primitive runtime call");
    };
    assert_eq!(
        primitive,
        RuntimePrimitive {
            symbol: "bytes_at".to_string(),
            partiality: RuntimePartiality::SafeOption {
                none: "ctor:nc16_partial_pkg::Option::None".to_string(),
                some: "ctor:nc16_partial_pkg::Option::Some".to_string(),
                obligation: Some(obligation.to_string()),
            },
        }
    );
}

#[test]
fn unsupported_literal_and_host_dependent_primitive_reject_before_runtime_success() {
    let package_name = "nc16_bad_lit_pkg";
    let target = decl_symbol(package_name, "target");
    let bad_lit = StableSymbol::primitive("lit_float_1_5");
    let table = stable_table(&[(GlobalId(1), target.clone()), (GlobalId(40), bad_lit)]);
    let (package, target) = primitive_package(
        package_name,
        primitive_const(GlobalId(40)),
        &table,
        [(
            "lit_float_1_5",
            PrimitiveReductionMetadata::Literal,
            PartialityMetadata::Total,
            LowerabilityStatus::Supported,
        )],
    );
    assert_eq!(
        erasure_lane(&package, &target),
        "unsupported_primitive_literal"
    );

    let package_name = "nc16_host_pkg";
    let target = decl_symbol(package_name, "target");
    let left = StableSymbol::primitive("lit_int_1");
    let right = StableSymbol::primitive("lit_int_2");
    let host_op = StableSymbol::primitive("host_locale_cmp");
    let table = stable_table(&[
        (GlobalId(1), target.clone()),
        (GlobalId(40), left),
        (GlobalId(41), right),
        (GlobalId(42), host_op),
    ]);
    let (package, target) = primitive_package(
        package_name,
        app2(
            primitive_const(GlobalId(42)),
            primitive_const(GlobalId(40)),
            primitive_const(GlobalId(41)),
        ),
        &table,
        [
            (
                "lit_int_1",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "lit_int_2",
                PrimitiveReductionMetadata::Literal,
                PartialityMetadata::Total,
                LowerabilityStatus::Supported,
            ),
            (
                "host_locale_cmp",
                PrimitiveReductionMetadata::Op,
                PartialityMetadata::Total,
                LowerabilityStatus::RequiresFeature {
                    feature: "host-dependent-primitive".to_string(),
                    reason: "depends on host locale".to_string(),
                },
            ),
        ],
    );
    assert_eq!(
        erasure_lane(&package, &target),
        "host_dependent_primitive_attempt"
    );
}

#[test]
fn unsupported_runtime_primitive_name_stays_loud_and_package_facing() {
    let expr = ken_runtime::RuntimeExpr::PrimitiveCall {
        primitive: RuntimePrimitive {
            symbol: "mystery_primitive".to_string(),
            partiality: RuntimePartiality::Total,
        },
        args: Vec::new(),
    };

    let err = evaluate_runtime_ir_expr(&expr, &RuntimeIrSeedEnvironment::empty())
        .expect_err("unsupported primitive must not evaluate successfully");

    assert_eq!(err.construct, "PrimitiveCall");
    assert!(err.reason.contains("mystery_primitive"));
}
