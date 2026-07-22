//! Certificate, preflight, differential and outward-runner subject tests
//! (RT-SPLIT §10.1/§10.2).
//!
//! Moved from the residual facade in slice 6, alongside `artifact::api`.
//! Ruled test module: imports are permitted here (AC-8 class 2).

use super::*;

use std::collections::BTreeSet;

use crate::cranelift_backend::total_primitive;
use crate::{
    evaluate_runtime_ir_example, nc5_seed_examples, ErasedExecutableCore,
    RuntimeArtifactValidationStage, RuntimeArtifactValidationTier, RuntimeAssumptionTrustKind,
    RuntimeAssumptionTrustMetadata, RuntimeDeclaration, RuntimeEffectsForeignAuditMetadata,
    RuntimeFieldStatus, RuntimeGroundValue, RuntimeIrSeedEnvironment, RuntimeMatchCase,
    RuntimeMetadata, RuntimeObservation, RuntimePartiality, RuntimePrimitive,
    RuntimeSymbolMetadata, RuntimeTrap, RuntimeTrapCode, UnsupportedLowering,
};

fn seed_program_with_lowerability(status: Option<RuntimeLowerabilityStatus>) -> RuntimeProgram {
    let symbol = "decl:fixture::Main::main".to_string();
    let mut metadata = RuntimeMetadata::default();
    if let Some(status) = status.clone() {
        metadata.lowerability.insert(symbol.clone(), status);
    }
    RuntimeProgram {
        package_identity: "module:fixture::nc6".to_string(),
        core_semantic_hash: 1,
        artifact_hash: 2,
        erased_core: ErasedExecutableCore {
            symbols: BTreeSet::from([symbol.clone()]),
            metadata,
        },
        declarations: vec![RuntimeDeclaration {
            symbol,
            kind: RuntimeDeclarationKind::Record {
                fields: vec![crate::RuntimeField {
                    name: "value".to_string(),
                    status: RuntimeFieldStatus::Runtime,
                }],
            },
            metadata: RuntimeSymbolMetadata {
                lowerability: status,
                ..RuntimeSymbolMetadata::empty()
            },
        }],
        examples: nc5_seed_examples(),
    }
}
fn nc22_program_with_body(body: RuntimeExpr, observation: RuntimeObservation) -> RuntimeProgram {
    let symbol = "decl:fixture::Main::main".to_string();
    let mut metadata = RuntimeMetadata::default();
    metadata
        .lowerability
        .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
    RuntimeProgram {
        package_identity: "module:fixture::nc22".to_string(),
        core_semantic_hash: 22,
        artifact_hash: 2200,
        erased_core: ErasedExecutableCore {
            symbols: BTreeSet::from([symbol.clone()]),
            metadata,
        },
        declarations: vec![RuntimeDeclaration {
            symbol: symbol.clone(),
            kind: RuntimeDeclarationKind::Transparent { body },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        }],
        examples: vec![RuntimeExample {
            name: "main-entrypoint".to_string(),
            checked_core_shape: "compiler-produced declaration ref".to_string(),
            ir: RuntimeExpr::DeclarationRef { symbol },
            observation,
        }],
    }
}
#[test]
fn program_runner_preflights_metadata_before_backend_lowering() {
    let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));

    let reports = run_nc6_seed_examples(&program).expect("seed program runs");

    assert_eq!(reports.len(), 5);
    assert!(reports
        .iter()
        .all(|report| report.trust.fidelity == NativeFidelity::F1SeedObservationAgreement));
}
#[test]
fn nc22_cranelift_agrees_with_runtime_ir_report_for_broad_starter_shapes() {
    let body = RuntimeExpr::Let {
        value: Box::new(total_primitive(
            "add_int",
            vec![
                RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                RuntimeExpr::Value(RuntimeValue::Int((3).into())),
            ],
        )),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Closure {
                captures: Vec::new(),
                params: vec!["x".to_string()],
                body: Box::new(RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Box::Box".to_string(),
                        args: vec![RuntimeExpr::Var(0)],
                    }),
                    cases: vec![RuntimeMatchCase {
                        constructor: "ctor:fixture::Box::Box".to_string(),
                        binders: 1,
                        body: RuntimeExpr::Record {
                            fields: vec![
                                (
                                    "ok".to_string(),
                                    RuntimeExpr::If {
                                        scrutinee: Box::new(total_primitive(
                                            "eq_int",
                                            vec![
                                                RuntimeExpr::Var(0),
                                                RuntimeExpr::Value(RuntimeValue::Int((5).into())),
                                            ],
                                        )),
                                        then_expr: Box::new(RuntimeExpr::Value(
                                            RuntimeValue::Bool(true),
                                        )),
                                        else_expr: Box::new(RuntimeExpr::Value(
                                            RuntimeValue::Bool(false),
                                        )),
                                    },
                                ),
                                (
                                    "value".to_string(),
                                    total_primitive(
                                        "sub_int",
                                        vec![
                                            total_primitive(
                                                "mul_int",
                                                vec![
                                                    RuntimeExpr::Var(0),
                                                    RuntimeExpr::Value(RuntimeValue::Int(
                                                        (2).into(),
                                                    )),
                                                ],
                                            ),
                                            RuntimeExpr::Value(RuntimeValue::Int((3).into())),
                                        ],
                                    ),
                                ),
                            ],
                        },
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "unexpected constructor".to_string(),
                    },
                }),
            }),
            args: vec![RuntimeExpr::Var(0)],
        }),
    };
    let observation = RuntimeObservation::Returned(RuntimeGroundValue::Record {
        fields: vec![
            ("ok".to_string(), RuntimeGroundValue::Bool(true)),
            ("value".to_string(), RuntimeGroundValue::Int((7).into())),
        ],
    });
    let program = nc22_program_with_body(body, observation.clone());
    let run_report = evaluate_runtime_ir_example(
        &program,
        &program.examples[0],
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("runtime-IR evaluator runs the compiler-produced artifact");

    let report =
        run_runtime_ir_report_with_cranelift(&program, run_report, &NativeSeedEnvironment::empty());

    assert_eq!(
        report.verdict,
        NativeRuntimeIrComparisonVerdict::RuntimeIrNativeAgreement {
            stage: NativeDifferentialStage::RuntimeIrNativeCompare,
        }
    );
    let native = report.native.expect("native side ran");
    assert_eq!(native.observation, observation);
    assert_eq!(
        native.trust.fidelity,
        NativeFidelity::F1RuntimeIrEvaluatorAgreement
    );
    assert_eq!(
        native.trust.evidence.runtime_artifact_hash,
        Some(program.artifact_hash)
    );
}
#[test]
fn nc22_imported_dependency_lowers_as_stable_unsupported_native_lane() {
    let symbol = "decl:fixture::Main::main".to_string();
    let dependency = "dep:fixture".to_string();
    let imported = "decl:dep::value".to_string();
    let dependency_hash = "hash:dep".to_string();
    let mut program = nc22_program_with_body(
        RuntimeExpr::ImportedDeclarationRef {
            symbol: imported.clone(),
            dependency: dependency.clone(),
            dependency_semantic_hash: dependency_hash.clone(),
        },
        RuntimeObservation::Returned(RuntimeGroundValue::Int((9).into())),
    );
    program.declarations[0].symbol = symbol.clone();
    program.erased_core.symbols.insert(imported.clone());
    program
        .erased_core
        .metadata
        .lowerability
        .insert(imported.clone(), RuntimeLowerabilityStatus::Supported);
    program
        .erased_core
        .metadata
        .dependency_semantic_hashes
        .insert(dependency.clone(), dependency_hash.clone());
    let mut runtime_env = RuntimeIrSeedEnvironment::empty();
    runtime_env.insert_imported_declaration(
        imported,
        dependency,
        dependency_hash,
        RuntimeGroundValue::Int((9).into()),
    );
    let run_report = evaluate_runtime_ir_example(&program, &program.examples[0], &runtime_env)
        .expect("runtime-IR evaluator can use an exact imported seed binding");

    let report =
        run_runtime_ir_report_with_cranelift(&program, run_report, &NativeSeedEnvironment::empty());

    assert!(matches!(
        report.verdict,
        NativeRuntimeIrComparisonVerdict::Unsupported {
            stage: NativeDifferentialStage::NativeLoweringOrExecution,
            construct: "ImportedDeclarationRef",
            ..
        }
    ));
    assert!(report.native.is_none());
}
#[test]
fn nc22_runtime_ir_report_identity_mismatch_rejects_before_native_lowering() {
    let program = nc22_program_with_body(
        RuntimeExpr::Value(RuntimeValue::Int((1).into())),
        RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
    );
    let mut run_report = evaluate_runtime_ir_example(
        &program,
        &program.examples[0],
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("runtime-IR evaluator runs");
    run_report.evidence.runtime_artifact_hash = 0xdead_beef;

    let report =
        run_runtime_ir_report_with_cranelift(&program, run_report, &NativeSeedEnvironment::empty());

    assert!(matches!(
        report.verdict,
        NativeRuntimeIrComparisonVerdict::Unsupported {
            stage: NativeDifferentialStage::BoundaryPreflight,
            construct: "RuntimeIrRunReport",
            ..
        }
    ));
    assert!(report.native.is_none());
}
#[test]
fn nc22_ambiguous_runtime_ir_report_target_rejects_before_native_lowering() {
    let mut program = nc22_program_with_body(
        RuntimeExpr::Value(RuntimeValue::Int((1).into())),
        RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
    );
    program.examples.push(program.examples[0].clone());
    let mut run_report = evaluate_runtime_ir_example(
        &nc22_program_with_body(
            RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
        ),
        &program.examples[0],
        &RuntimeIrSeedEnvironment::empty(),
    )
    .expect("runtime-IR evaluator runs");
    run_report.artifact = RuntimeArtifactIdentity::from_program(&program);
    run_report.observation.artifact = RuntimeArtifactIdentity::from_program(&program);
    run_report.evidence.package_identity = program.package_identity.clone();
    run_report.evidence.core_semantic_hash = program.core_semantic_hash;
    run_report.evidence.runtime_artifact_hash = program.artifact_hash;

    let report =
        run_runtime_ir_report_with_cranelift(&program, run_report, &NativeSeedEnvironment::empty());

    assert!(matches!(
        report.verdict,
        NativeRuntimeIrComparisonVerdict::Unsupported {
            stage: NativeDifferentialStage::BoundaryPreflight,
            construct: "RuntimeIrRunReport",
            ..
        }
    ));
    assert!(report.native.is_none());
}
#[test]
fn nc8_valid_certificate_records_f2_validation_separate_from_f1() {
    let example = nc5_seed_examples()
        .into_iter()
        .find(|example| example.name == "closed-scalar-primitive")
        .expect("seed exists");
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![example.clone()];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
    let oracle = InterpreterOracleObservation {
        artifact: NativeArtifactIdentity::from_program(&program),
        observation: example.observation.clone(),
        evidence_source: "test oracle over matching RuntimeProgram identity".to_string(),
    };

    let report = run_validated_example_with_interpreter_observation(
        &program,
        &example,
        &NativeSeedEnvironment::empty(),
        oracle,
        &certificate,
    )
    .expect("certificate validates");

    assert_eq!(
        report.verdict,
        NativeDifferentialVerdict::F1InterpreterAgreement {
            stage: NativeDifferentialStage::InterpreterNativeCompare,
        }
    );
    let native = report.native.expect("native side ran");
    assert_eq!(
        native.trust.fidelity,
        NativeFidelity::F1InterpreterDifferentialAgreement
    );
    let validation = native
        .trust
        .artifact_validation
        .expect("validated artifact fact is report-visible");
    assert_eq!(
        validation.tier,
        RuntimeArtifactValidationTier::F2BoundedRuntimeArtifactValidation
    );
    assert_eq!(
        validation.artifact.package_identity,
        program.package_identity
    );
    assert_eq!(
        validation.artifact.core_semantic_hash,
        program.core_semantic_hash
    );
    assert_eq!(validation.artifact.artifact_hash, program.artifact_hash);
    assert!(validation
        .evidence_source
        .contains("recomputed supported-subset facts"));
}
#[test]
fn nc8_certificate_wrong_identity_rejects_before_native_run() {
    let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
    certificate.artifact_hash = Some(0xdead_beef);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("wrong artifact identity rejects");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ArtifactIdentity);
    assert_eq!(err.fact, "runtime_artifact_identity");
}
#[test]
fn nc8_certificate_missing_fields_rejects_loudly() {
    let program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
    certificate.core_semantic_hash = None;

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("missing identity field rejects");

    assert_eq!(
        err.stage,
        RuntimeArtifactValidationStage::MalformedCertificate
    );
    assert_eq!(err.fact, "core_semantic_hash");

    let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
    certificate.claim.as_mut().expect("claim exists").facts = None;
    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("missing facts reject");

    assert_eq!(
        err.stage,
        RuntimeArtifactValidationStage::MalformedCertificate
    );
    assert_eq!(err.fact, "facts");
}
#[test]
fn nc8_certificate_contradictory_claim_rejects() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![nc5_seed_examples()
        .into_iter()
        .find(|example| example.name == "closed-scalar-primitive")
        .expect("seed exists")];
    let mut certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);
    certificate
        .claim
        .as_mut()
        .expect("claim exists")
        .facts
        .as_mut()
        .expect("facts exist")
        .declaration_count = Some(program.declarations.len() + 1);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("contradictory count rejects");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimMismatch);
    assert_eq!(err.fact, "declaration_count");
}
#[test]
fn nc8_certificate_false_supported_claim_rejects_by_recomputation() {
    let mut program =
        seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Unsupported {
            reason: "not lowerable".to_string(),
        }));
    let symbol = program.declarations[0].symbol.clone();
    program.declarations[0].metadata.lowerability = Some(RuntimeLowerabilityStatus::Unsupported {
        reason: "not lowerable".to_string(),
    });
    program
        .erased_core
        .metadata
        .unsupported
        .insert(symbol, b"hidden blocker".to_vec());
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("false supported-subset claim rejects");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert!(matches!(
        err.fact,
        "no_reachable_unsupported_entries" | "all_reachable_lowerability_supported"
    ));
}
#[test]
fn nc8_certificate_rejects_unknown_runtime_value_by_recomputation() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "unknown-runtime-value".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Value(RuntimeValue::Unknown),
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((0).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("unknown runtime values are outside the supported subset");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_values_supported");
    assert!(err.reason.contains("unknown runtime data"));
}
#[test]
fn nc8_certificate_rejects_let_expression_in_validated_example() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "let-outside-supported-subset".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
            body: Box::new(RuntimeExpr::Var(0)),
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("let expressions are outside the NC6 supported subset");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("Let"));
}
#[test]
fn nc8_certificate_rejects_if_expression_in_reachable_transparent_declaration() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
        body: RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
            else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((0).into()))),
        },
    };
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("if expressions are outside the NC6 supported subset");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("If"));
}
#[test]
fn nc8_certificate_rejects_unsupported_total_primitive_in_validated_example() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "unsupported-total-primitive".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "sub_int".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            ],
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("unsupported total primitives are outside the NC6 supported subset");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_primitives_supported");
    assert!(err.reason.contains("sub_int"));
}
#[test]
fn nc8_certificate_rejects_add_int_wrong_arity_in_reachable_transparent_declaration() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
        body: RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "add_int".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((1).into()))],
        },
    };
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("add_int arity mismatch is outside the NC6 supported subset");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_primitives_supported");
    assert!(err.reason.contains("arity 1"));
}
#[test]
fn nc8_certificate_rejects_add_int_non_literal_int_operand_shape() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "add-int-non-int-operand".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "add_int".to_string(),
                partiality: RuntimePartiality::Total,
            },
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Bool(true)),
                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            ],
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("add_int non-literal-Int operands are outside the NC8 subset");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_primitives_supported");
    assert!(err.reason.contains("non-literal-Int operand"));
}
#[test]
fn nc8_certificate_rejects_add_int_var_bound_to_bool_payload() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "add-int-var-bound-to-bool".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::BoolBox::Box".to_string(),
                args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
            }),
            cases: vec![RuntimeMatchCase {
                constructor: "ctor:fixture::BoolBox::Box".to_string(),
                binders: 1,
                body: RuntimeExpr::PrimitiveCall {
                    primitive: RuntimePrimitive {
                        symbol: "add_int".to_string(),
                        partiality: RuntimePartiality::Total,
                    },
                    args: vec![
                        RuntimeExpr::Var(0),
                        RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                    ],
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "unused default".to_string(),
            },
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((2).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("add_int variable operands are outside the first NC8 validator");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("Match"));
}
#[test]
fn nc8_certificate_rejects_top_level_var_example() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "top-level-var".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Var(0),
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((0).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("unbound var is outside the first NC8 validator");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("Var"));
}
#[test]
fn nc8_certificate_rejects_project_from_non_record_example() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "project-from-int".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Project {
            record: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
            field: "x".to_string(),
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("project is outside the first NC8 validator");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("Project"));
}
#[test]
fn nc8_certificate_rejects_top_level_observable_closure() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.examples = vec![RuntimeExample {
        name: "top-level-closure".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int((1).into())),
    }];
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("closure is outside the first NC8 validator");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("Closure"));
}
#[test]
fn nc8_certificate_rejects_var_in_reachable_transparent_declaration() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    program.declarations[0].kind = RuntimeDeclarationKind::Transparent {
        body: RuntimeExpr::Var(0),
    };
    let certificate = RuntimeArtifactCertificate::supported_runtime_artifact_for(&program);

    let err = validate_supported_runtime_artifact_certificate(&program, &certificate)
        .expect_err("transparent declaration var is outside the first NC8 validator");

    assert_eq!(err.stage, RuntimeArtifactValidationStage::ClaimRecompute);
    assert_eq!(err.fact, "all_runtime_expressions_supported");
    assert!(err.reason.contains("Var"));
}
#[test]
fn missing_lowerability_metadata_rejects_before_backend_lowering() {
    let program = seed_program_with_lowerability(None);

    let err = run_nc6_seed_examples(&program).expect_err("missing metadata rejects");

    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "RuntimeProgram",
            ..
        })
    ));
}
#[test]
fn reachable_unsupported_metadata_rejects_before_backend_lowering() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    let symbol = program.declarations[0].symbol.clone();
    program
        .erased_core
        .metadata
        .unsupported
        .insert(symbol, b"unsupported target".to_vec());

    let err = run_nc6_seed_examples(&program).expect_err("unsupported metadata rejects");

    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "RuntimeProgram",
            ..
        })
    ));
}
#[test]
fn reachable_declaration_effect_metadata_rejects_before_backend_lowering() {
    for lane in [
        "effects",
        "capabilities",
        "runtime_checks",
        "assumptions",
        "assumption_trust_metadata",
        "trusted_base_delta",
    ] {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        let target = program.declarations[0].symbol.clone();
        match lane {
            "effects" => {
                program.declarations[0]
                    .metadata
                    .effects
                    .insert("Console".to_string());
            }
            "capabilities" => {
                program.declarations[0]
                    .metadata
                    .capabilities
                    .insert("cap:Console".to_string());
            }
            "runtime_checks" => {
                program.declarations[0]
                    .metadata
                    .runtime_checks
                    .insert("check:Console".to_string());
            }
            "assumptions" => {
                program.declarations[0]
                    .metadata
                    .assumptions
                    .insert("assume:Console".to_string());
            }
            "assumption_trust_metadata" => {
                program.declarations[0]
                    .metadata
                    .assumption_trust_metadata
                    .insert(
                        "assume:Console".to_string(),
                        RuntimeAssumptionTrustMetadata {
                            kind: RuntimeAssumptionTrustKind::Declassify,
                            target,
                            affects_runtime_meaning: true,
                        },
                    );
            }
            "trusted_base_delta" => {
                program.declarations[0]
                    .metadata
                    .trusted_base_delta
                    .insert("assume:Console".to_string());
            }
            _ => unreachable!("test lanes are exhaustive"),
        }

        let err = match run_nc6_seed_examples(&program) {
            Ok(_) => panic!("expected {lane} metadata to reject"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }
}
#[test]
fn reachable_package_effect_metadata_rejects_before_backend_lowering() {
    for lane in ["effects", "capabilities", "runtime_checks"] {
        let mut program =
            seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
        match lane {
            "effects" => {
                program
                    .erased_core
                    .metadata
                    .effects
                    .insert("Console".to_string());
            }
            "capabilities" => {
                program
                    .erased_core
                    .metadata
                    .capabilities
                    .insert("cap:Console".to_string());
            }
            "runtime_checks" => {
                program
                    .erased_core
                    .metadata
                    .runtime_checks
                    .insert("check:Console".to_string());
            }
            _ => unreachable!("test lanes are exhaustive"),
        }

        let err = match run_nc6_seed_examples(&program) {
            Ok(_) => panic!("expected package {lane} metadata to reject"),
            Err(err) => err,
        };

        assert!(matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "RuntimeProgram",
                ..
            })
        ));
    }
}
#[test]
fn reachable_effectful_checked_core_metadata_rejects_before_backend_lowering() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    let symbol = program.declarations[0].symbol.clone();
    program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .insert(
            symbol,
            RuntimeEffectsForeignAuditMetadata {
                declared_effects: BTreeSet::from(["Console".to_string()]),
                capabilities: BTreeSet::from(["cap:Console".to_string()]),
                foreign_symbol: None,
                boundary: RuntimeEffectBoundary::Effectful,
                runtime_checks: BTreeSet::from(["check:Console".to_string()]),
                lowerability: RuntimeLowerabilityStatus::Supported,
            },
        );

    let err =
        run_nc6_seed_examples(&program).expect_err("effectful checked-core metadata must reject");

    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "RuntimeProgram",
            ..
        })
    ));
}
#[test]
fn reachable_foreign_checked_core_metadata_rejects_before_backend_lowering() {
    let mut program = seed_program_with_lowerability(Some(RuntimeLowerabilityStatus::Supported));
    let symbol = program.declarations[0].symbol.clone();
    program
        .erased_core
        .metadata
        .checked_core
        .effects_foreign_metadata
        .insert(
            symbol,
            RuntimeEffectsForeignAuditMetadata {
                declared_effects: BTreeSet::new(),
                capabilities: BTreeSet::new(),
                foreign_symbol: Some("host.fixture.foreign".to_string()),
                boundary: RuntimeEffectBoundary::Foreign,
                runtime_checks: BTreeSet::new(),
                lowerability: RuntimeLowerabilityStatus::Supported,
            },
        );

    let err =
        run_nc6_seed_examples(&program).expect_err("foreign checked-core metadata must reject");

    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "RuntimeProgram",
            ..
        })
    ));
}
