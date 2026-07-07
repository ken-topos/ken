use std::collections::BTreeMap;

use ken_elaborator::checked_core::{
    canonical_decl_bytes, emit_checked_core_package, CheckedCoreArtifactInputs, CheckedCorePackage,
    CheckedCorePackageHeader, CheckedCoreSemanticInputs, FieldMetadata, LowerabilityStatus,
    RecordSigmaKind, RecordSigmaMetadata, RuntimeFieldStatus as CheckedRuntimeFieldStatus,
    StableSymbol, StableSymbolTable, SymbolNamespace,
};
use ken_elaborator::erasure::{
    emit_proof_erasure_boundary_witness_for_targets, erase_checked_core_package_for_target,
    ErasureError,
};
use ken_kernel::{Decl, GlobalId, Level, Term};
use ken_runtime::{
    evaluate_runtime_ir_expr, validate_proof_erasure_boundary_witness,
    ProofErasureBoundaryWitnessStage, ProofErasureBoundaryWitnessTier, RuntimeDeclarationKind,
    RuntimeExpr, RuntimeFieldStatus, RuntimeGroundValue, RuntimeIrSeedEnvironment,
    RuntimeObservation, RuntimeValue,
};

fn decl_symbol(package: &str, name: &str) -> StableSymbol {
    StableSymbol::declaration(package, &[], name)
}

fn header(package: &str) -> CheckedCorePackageHeader {
    CheckedCorePackageHeader::v0(
        "ken-elaborator:record-sigma-proof-erasure-lowering-test",
        "ken-kernel:test",
        "docs/program/wp/NC15-record-sigma-proof-erasure-lowering.md",
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

fn record_symbols(package: &str) -> (StableSymbol, StableSymbol, StableSymbol) {
    (
        decl_symbol(package, "RecordPayload"),
        decl_symbol(package, "RuntimePayload"),
        decl_symbol(package, "record_value"),
    )
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

fn record_metadata(payload: &StableSymbol) -> RecordSigmaMetadata {
    RecordSigmaMetadata {
        kind: RecordSigmaKind::Record,
        fields: vec![
            FieldMetadata {
                name: "runtime_payload".to_string(),
                ty: payload.clone(),
                runtime: CheckedRuntimeFieldStatus::Runtime,
            },
            FieldMetadata {
                name: "law_payload".to_string(),
                ty: payload.clone(),
                runtime: CheckedRuntimeFieldStatus::ErasedLaw,
            },
            FieldMetadata {
                name: "proof_payload".to_string(),
                ty: payload.clone(),
                runtime: CheckedRuntimeFieldStatus::ErasedProof,
            },
        ],
        lowerability: LowerabilityStatus::Supported,
    }
}

fn record_package(package: &str) -> (CheckedCorePackage, StableSymbol, StableSymbol) {
    let (record, payload, target) = record_symbols(package);
    let symbols = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(10), record.clone()),
        (GlobalId(11), payload.clone()),
    ]);
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
    semantic.declarations.insert(
        target.clone(),
        canonical_decl_bytes(&decl, &symbols).expect("canonical target declaration"),
    );
    semantic
        .record_sigma_metadata
        .insert(record.clone(), record_metadata(&payload));

    let package = emit_checked_core_package(
        header(package),
        CheckedCoreArtifactInputs {
            semantic,
            source_identity: BTreeMap::new(),
            annotations: BTreeMap::new(),
        },
    )
    .expect("checked-core package emits");
    (package, target, record)
}

fn reemit(mut package: CheckedCorePackage) -> CheckedCorePackage {
    package.header.dependency_semantic_hashes =
        package.artifact.semantic.dependency_semantic_hashes.clone();
    emit_checked_core_package(package.header, package.artifact).expect("package re-emits")
}

fn replace_target(
    package: &mut CheckedCorePackage,
    target: &StableSymbol,
    ty: Term,
    body: Term,
    symbols: &StableSymbolTable,
) {
    let decl = Decl::Transparent {
        id: GlobalId(1),
        level_params: Vec::new(),
        ty,
        body,
    };
    package.artifact.semantic.declarations.insert(
        target.clone(),
        canonical_decl_bytes(&decl, symbols).expect("canonical replacement declaration"),
    );
    *package = reemit(package.clone());
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

fn erasure_lane(package: &CheckedCorePackage, target: &StableSymbol) -> &'static str {
    let err = erase_checked_core_package_for_target(package, [target])
        .expect_err("package must reject before runtime program success");
    match err {
        ErasureError::ExpressionLowering { lane, .. } => lane,
        other => panic!("expected expression lowering error, got {other:?}"),
    }
}

#[test]
fn record_construction_lowers_runtime_fields_and_preserves_erased_statuses() {
    let (package, target, record) = record_package("nc15_record_construct_pkg");
    let program = erase_checked_core_package_for_target(&package, [&target, &record])
        .expect("record construction target lowers");
    let body = lowered_body(&program, &target);

    let observation = evaluate_runtime_ir_expr(
        &RuntimeExpr::Call {
            callee: Box::new(body),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(41))],
        },
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("generated record runtime IR evaluates");

    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Record {
            fields: vec![("runtime_payload".to_string(), RuntimeGroundValue::Int(41))]
        })
    );
    let record_decl = program
        .declarations
        .iter()
        .find(|declaration| declaration.symbol == record.to_string())
        .expect("record metadata declaration is reachable");
    let RuntimeDeclarationKind::Record { fields } = &record_decl.kind else {
        panic!("expected record declaration, got {record_decl:?}");
    };
    assert_eq!(
        fields
            .iter()
            .map(|field| (field.name.as_str(), field.status.clone()))
            .collect::<Vec<_>>(),
        vec![
            ("runtime_payload", RuntimeFieldStatus::Runtime),
            ("law_payload", RuntimeFieldStatus::ErasedLaw),
            ("proof_payload", RuntimeFieldStatus::ErasedProof),
        ]
    );

    let witness =
        emit_proof_erasure_boundary_witness_for_targets(&package, [&target, &record], &program)
            .expect("record field-status witness emits");
    let report = validate_proof_erasure_boundary_witness(&program, &witness)
        .expect("runtime witness checker validates the concrete program lanes");
    assert_eq!(
        report.tier,
        ProofErasureBoundaryWitnessTier::Nc9BoundedProofErasureBoundary
    );
    assert_eq!(
        report
            .facts
            .record_field_statuses
            .get(&record.to_string())
            .expect("runtime record field statuses"),
        report
            .facts
            .checked_core_record_field_statuses
            .get(&record.to_string())
            .expect("checked-core record field statuses")
    );
}

#[test]
fn runtime_record_projection_from_constructed_sigma_lowers_and_evaluates() {
    let (mut package, target, record) = record_package("nc15_record_project_pkg");
    let (_, payload, _) = record_symbols("nc15_record_project_pkg");
    let symbols = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(10), record),
        (GlobalId(11), payload),
    ]);
    replace_target(
        &mut package,
        &target,
        Term::pi(payload_type(), payload_type()),
        Term::Lam(
            Box::new(payload_type()),
            Box::new(Term::Let {
                ty: Box::new(record_type()),
                val: Box::new(record_pair_body()),
                body: Box::new(Term::Proj1(Box::new(Term::Var(0)))),
            }),
        ),
        &symbols,
    );
    let program = erase_checked_core_package_for_target(&package, [&target])
        .expect("record projection target lowers");
    let body = lowered_body(&program, &target);

    let observation = evaluate_runtime_ir_expr(
        &RuntimeExpr::Call {
            callee: Box::new(body),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(7))],
        },
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("generated projection runtime IR evaluates");

    assert_eq!(
        observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(7))
    );
}

#[test]
fn erased_law_field_projection_rejects_before_runtime_program_success() {
    let (mut package, target, record) = record_package("nc15_erased_projection_pkg");
    let (_, payload, _) = record_symbols("nc15_erased_projection_pkg");
    let symbols = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(10), record),
        (GlobalId(11), payload),
    ]);
    replace_target(
        &mut package,
        &target,
        Term::pi(record_type(), payload_type()),
        Term::Lam(
            Box::new(record_type()),
            Box::new(Term::Proj1(Box::new(Term::Proj2(Box::new(Term::Var(0)))))),
        ),
        &symbols,
    );

    assert_eq!(
        erasure_lane(&package, &target),
        "non_executable_erased_field_projection"
    );
}

#[test]
fn stale_record_field_order_rejects_before_runtime_program_success() {
    let (mut package, target, record) = record_package("nc15_stale_field_pkg");
    let (_, payload, _) = record_symbols("nc15_stale_field_pkg");
    let symbols = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(10), record),
        (GlobalId(11), payload),
    ]);
    replace_target(
        &mut package,
        &target,
        Term::pi(payload_type(), record_type()),
        Term::Lam(
            Box::new(payload_type()),
            Box::new(Term::Pair(
                Box::new(Term::Var(0)),
                Box::new(Term::Type(Level::zero())),
            )),
        ),
        &symbols,
    );

    assert_eq!(
        erasure_lane(&package, &target),
        "stale_field_identity_order"
    );
}

#[test]
fn unsupported_dependent_record_field_shape_rejects_before_runtime_ir() {
    let (mut package, target, record) = record_package("nc15_dependent_field_pkg");
    let (_, payload, _) = record_symbols("nc15_dependent_field_pkg");
    let symbols = table_many(&[
        (GlobalId(1), target.clone()),
        (GlobalId(10), record),
        (GlobalId(11), payload),
    ]);
    replace_target(
        &mut package,
        &target,
        Term::pi(
            payload_type(),
            Term::sigma(Term::Type(Level::zero()), Term::Var(0)),
        ),
        Term::Lam(
            Box::new(payload_type()),
            Box::new(Term::Pair(
                Box::new(Term::Var(0)),
                Box::new(Term::Type(Level::zero())),
            )),
        ),
        &symbols,
    );

    assert_eq!(
        erasure_lane(&package, &target),
        "unsupported_dependent_field_shape"
    );
}

#[test]
fn witness_rejects_runtime_or_checked_core_field_status_drift() {
    for lane in [
        "record_field_statuses",
        "checked_core_record_field_statuses",
    ] {
        let (package, target, record) = record_package("nc15_status_drift_pkg");
        let mut program = erase_checked_core_package_for_target(&package, [&target, &record])
            .expect("record construction target lowers");
        match lane {
            "record_field_statuses" => {
                let declaration = program
                    .declarations
                    .iter_mut()
                    .find(|declaration| declaration.symbol == record.to_string())
                    .expect("record declaration exists");
                let RuntimeDeclarationKind::Record { fields } = &mut declaration.kind else {
                    panic!("expected record declaration, got {declaration:?}");
                };
                fields[1].status = RuntimeFieldStatus::Runtime;
            }
            "checked_core_record_field_statuses" => {
                let metadata = program
                    .erased_core
                    .metadata
                    .checked_core
                    .record_sigma_metadata
                    .get_mut(&record.to_string())
                    .expect("checked-core record metadata survives");
                metadata.fields[2].runtime = RuntimeFieldStatus::Runtime;
            }
            _ => unreachable!("test lanes are exhaustive"),
        }

        let err =
            emit_proof_erasure_boundary_witness_for_targets(&package, [&target, &record], &program)
                .expect_err("field-status drift must reject");

        assert!(matches!(
            err,
            ErasureError::ProofErasureBoundaryWitness(witness)
                if witness.stage == ProofErasureBoundaryWitnessStage::WitnessMismatch
                    && witness.lane == lane
        ));
    }
}
