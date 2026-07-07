use std::collections::BTreeSet;

use ken_elaborator::checked_core::{StableSymbol, SymbolNamespace};
use ken_elaborator::compiler_driver::{
    compile_ken_source, CompilerSource, CompilerTargetKind, TargetSelector,
};
use ken_interp::{eval, EvalStore, EvalVal};
use ken_kernel::{declare_primitive, GlobalEnv, Level, PrimReduction, Term};
use ken_runtime::{
    compare_runtime_ir_with_interpreter_observation, evaluate_runtime_ir_example,
    nc5_seed_examples, ErasedExecutableCore, RuntimeArtifactIdentity, RuntimeAssumptionTrustKind,
    RuntimeAssumptionTrustMetadata, RuntimeDeclaration, RuntimeDeclarationKind, RuntimeField,
    RuntimeFieldStatus, RuntimeGroundValue, RuntimeInterpreterObservation,
    RuntimeIrDifferentialStage, RuntimeIrDifferentialVerdict, RuntimeIrEvidenceFact,
    RuntimeIrSeedEnvironment, RuntimeIrTargetIdentity, RuntimeIrTrustTier,
    RuntimeLowerabilityStatus, RuntimeMetadata, RuntimeObservation, RuntimeProgram,
    RuntimeSymbolMetadata, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
};

struct OracleFixture {
    globals: GlobalEnv,
    store: EvalStore,
    term: Term,
}

fn interpreter_add_2_3_fixture() -> OracleFixture {
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
        level_args: vec![],
    };
    let add_ty = Term::pi(int_ty.clone(), Term::pi(int_ty.clone(), int_ty.clone()));
    let add_id = declare_primitive(
        &mut globals,
        vec![],
        add_ty,
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

    let add = Term::Const {
        id: add_id,
        level_args: vec![],
    };
    let two = Term::Const {
        id: lit_2,
        level_args: vec![],
    };
    let three = Term::Const {
        id: lit_3,
        level_args: vec![],
    };
    let term = Term::app(Term::app(add, two), three);

    OracleFixture {
        globals,
        store,
        term,
    }
}

fn runtime_program(example: ken_runtime::RuntimeExample, artifact_hash: u64) -> RuntimeProgram {
    let symbol = "decl:fixture::Main::main".to_string();
    let mut metadata = RuntimeMetadata::default();
    metadata
        .lowerability
        .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
    RuntimeProgram {
        package_identity: "module:fixture::nc12".to_string(),
        core_semantic_hash: 0x1201,
        artifact_hash,
        erased_core: ErasedExecutableCore {
            symbols: BTreeSet::from([symbol.clone()]),
            metadata,
        },
        declarations: vec![RuntimeDeclaration {
            symbol,
            kind: RuntimeDeclarationKind::Record {
                fields: vec![RuntimeField {
                    name: "value".to_string(),
                    status: RuntimeFieldStatus::Runtime,
                }],
            },
            metadata: RuntimeSymbolMetadata {
                lowerability: Some(RuntimeLowerabilityStatus::Supported),
                ..RuntimeSymbolMetadata::empty()
            },
        }],
        examples: vec![example],
    }
}

fn scalar_seed_example() -> ken_runtime::RuntimeExample {
    nc5_seed_examples()
        .into_iter()
        .find(|example| example.name == "closed-scalar-primitive")
        .expect("closed scalar seed exists")
}

fn oracle_observation(
    program: &RuntimeProgram,
    example: &ken_runtime::RuntimeExample,
) -> RuntimeInterpreterObservation {
    let OracleFixture {
        globals,
        mut store,
        term,
    } = interpreter_add_2_3_fixture();
    let value = eval(&[], &term, &globals, &mut store);
    let observation = match value {
        EvalVal::Int(value) => RuntimeObservation::Returned(RuntimeGroundValue::Int(value)),
        other => panic!("NC12 oracle fixture must return Int, got {other:?}"),
    };
    RuntimeInterpreterObservation {
        artifact: RuntimeArtifactIdentity::from_program(program),
        target: RuntimeIrTargetIdentity::from_example(example),
        observation,
        evidence_source: "ken-interp eval over GlobalEnv + closed core Term: add_int 2 3"
            .to_string(),
    }
}

#[test]
fn runtime_ir_evaluator_runs_nc5_seed_subset_without_native_claims() {
    let mut saw_trap = false;
    for example in nc5_seed_examples() {
        let program = runtime_program(example.clone(), 0x1202);
        let env = if example.name == "closure-capture-application" {
            RuntimeIrSeedEnvironment::nc5_seed()
        } else {
            RuntimeIrSeedEnvironment::empty()
        };

        let report =
            evaluate_runtime_ir_example(&program, &example, &env).expect("runtime IR evaluates");

        assert_eq!(report.observation.observation, example.observation);
        assert_eq!(
            report.artifact,
            RuntimeArtifactIdentity::from_program(&program)
        );
        assert_eq!(
            report.target,
            RuntimeIrTargetIdentity::from_example(&example)
        );
        assert_eq!(
            report.trust.tier,
            RuntimeIrTrustTier::Nc12RuntimeIrObservation
        );
        assert!(matches!(
            report.trust.native_backend,
            RuntimeIrEvidenceFact::Unavailable { .. }
        ));
        assert!(matches!(
            report.trust.object_artifact,
            RuntimeIrEvidenceFact::Unavailable { .. }
        ));
        assert!(matches!(
            report.trust.linker,
            RuntimeIrEvidenceFact::Unavailable { .. }
        ));
        saw_trap |= matches!(example.observation, RuntimeObservation::Trapped(_));
    }
    assert!(
        saw_trap,
        "seed suite must exercise explicit trap observation"
    );
}

#[test]
fn caller_supplied_interpreter_agreement_binds_identities_without_live_path_claim() {
    let example = scalar_seed_example();
    let program = runtime_program(example.clone(), 0x1203);
    let oracle = oracle_observation(&program, &example);

    let report = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle.clone(),
    );

    assert_eq!(
        report.artifact,
        RuntimeArtifactIdentity::from_program(&program)
    );
    assert_eq!(
        report.target,
        RuntimeIrTargetIdentity::from_example(&example)
    );
    assert_eq!(report.interpreter, oracle);
    assert_eq!(
        report.verdict,
        RuntimeIrDifferentialVerdict::Nc12InterpreterRuntimeIrAgreement {
            stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
        }
    );
    assert_eq!(
        report.trust.tier,
        RuntimeIrTrustTier::Nc12InterpreterRuntimeIrAgreement
    );
    assert!(matches!(
        report.trust.native_backend,
        RuntimeIrEvidenceFact::Unavailable { .. }
    ));
    assert!(matches!(
        report.trust.source_level_proof,
        RuntimeIrEvidenceFact::Unavailable { .. }
    ));
    assert!(matches!(
        report.trust.interpreter_oracle,
        RuntimeIrEvidenceFact::Available {
            value,
            evidence_source,
        } if value == "caller-supplied interpreter observation"
            && evidence_source.contains("supplied by the caller")
            && evidence_source.contains("not interpreter provenance")
    ));
    let runtime_ir = report.runtime_ir.expect("runtime IR side ran");
    assert_eq!(
        runtime_ir.artifact,
        RuntimeArtifactIdentity::from_program(&program)
    );
    assert_eq!(
        runtime_ir.target,
        RuntimeIrTargetIdentity::from_example(&example)
    );
    assert_eq!(
        runtime_ir.observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(5))
    );
    assert!(runtime_ir.evidence_source.contains("RuntimeExpr evaluator"));
    assert!(report
        .interpreter
        .evidence_source
        .contains("ken-interp eval over GlobalEnv"));
}

#[test]
fn disagreement_report_names_artifact_target_and_both_observations() {
    let mut example = scalar_seed_example();
    example.name = "mismatched-runtime-ir".to_string();
    example.ir = ken_runtime::RuntimeExpr::Value(RuntimeValue::Int(4));
    let program = runtime_program(example.clone(), 0x1204);
    let oracle = oracle_observation(&program, &example);

    let report = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle,
    );

    assert_eq!(report.artifact.artifact_hash, 0x1204);
    assert_eq!(report.target.example, "mismatched-runtime-ir");
    assert!(report.runtime_ir.is_some(), "runtime side must have run");
    assert!(matches!(
        report.verdict,
        RuntimeIrDifferentialVerdict::Mismatch {
            stage: RuntimeIrDifferentialStage::InterpreterRuntimeIrCompare,
            interpreter: RuntimeObservation::Returned(RuntimeGroundValue::Int(5)),
            runtime_ir: RuntimeObservation::Returned(RuntimeGroundValue::Int(4)),
        }
    ));
    assert_ne!(
        report.trust.tier,
        RuntimeIrTrustTier::Nc12InterpreterRuntimeIrAgreement,
        "mismatch reports must not carry the agreement trust tier"
    );
    assert!(matches!(
        report.trust.interpreter_oracle,
        RuntimeIrEvidenceFact::Available {
            value,
            evidence_source,
        } if value == "caller-supplied interpreter observation"
            && evidence_source.contains("supplied by the caller")
    ));
}

#[test]
fn stale_oracle_identity_rejects_before_runtime_ir_evaluation() {
    let example = scalar_seed_example();
    let program = runtime_program(example.clone(), 0x1205);
    let mut oracle = oracle_observation(&program, &example);
    oracle.artifact.artifact_hash = 0x9999;

    let report = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle,
    );

    assert!(report.runtime_ir.is_none());
    assert!(matches!(
        report.verdict,
        RuntimeIrDifferentialVerdict::Unsupported {
            stage: RuntimeIrDifferentialStage::BoundaryPreflight,
            construct: "RuntimeInterpreterObservation",
            ..
        }
    ));
}

#[test]
fn unsupported_effect_metadata_rejects_without_success_observation() {
    let example = scalar_seed_example();
    let mut program = runtime_program(example.clone(), 0x1206);
    program
        .erased_core
        .metadata
        .effects
        .insert("Console".to_string());
    let oracle = oracle_observation(&program, &example);

    let report = compare_runtime_ir_with_interpreter_observation(
        &program,
        &example,
        &RuntimeIrSeedEnvironment::empty(),
        oracle,
    );

    assert!(report.runtime_ir.is_none());
    assert!(matches!(
        report.verdict,
        RuntimeIrDifferentialVerdict::Unsupported {
            stage: RuntimeIrDifferentialStage::BoundaryPreflight,
            construct: "RuntimeProgram",
            ..
        }
    ));
}

#[test]
fn package_level_trust_metadata_rejects_before_runtime_ir_success() {
    for lane in [
        "assumptions",
        "assumption_trust_metadata",
        "trusted_base_delta",
    ] {
        let example = scalar_seed_example();
        let mut program = runtime_program(example.clone(), 0x1208);
        match lane {
            "assumptions" => {
                program.erased_core.metadata.assumptions.insert(
                    "assume:fixture::pkg".to_string(),
                    b"package assumption".to_vec(),
                );
            }
            "assumption_trust_metadata" => {
                program
                    .erased_core
                    .metadata
                    .assumption_trust_metadata
                    .insert(
                        "assume:fixture::pkg".to_string(),
                        RuntimeAssumptionTrustMetadata {
                            kind: RuntimeAssumptionTrustKind::Postulate,
                            target: "decl:fixture::Main::main".to_string(),
                            affects_runtime_meaning: true,
                        },
                    );
            }
            "trusted_base_delta" => {
                program.erased_core.metadata.trusted_base_delta.insert(
                    "assume:fixture::pkg".to_string(),
                    b"package trust delta".to_vec(),
                );
            }
            _ => unreachable!("test lane list is exhaustive"),
        }
        let oracle = oracle_observation(&program, &example);

        let report = compare_runtime_ir_with_interpreter_observation(
            &program,
            &example,
            &RuntimeIrSeedEnvironment::empty(),
            oracle,
        );

        assert!(report.runtime_ir.is_none(), "{lane} must not evaluate");
        assert!(matches!(
            report.verdict,
            RuntimeIrDifferentialVerdict::Unsupported {
                stage: RuntimeIrDifferentialStage::BoundaryPreflight,
                construct: "RuntimeProgram",
                ref reason,
            } if reason.contains("package carries trust metadata")
        ));
    }
}

#[test]
fn source_derived_package_identity_survives_into_runtime_ir_report() {
    let package = "nc12_source_demo";
    let package_identity = StableSymbol::new(SymbolNamespace::Module, vec![package.to_string()]);
    let target = StableSymbol::declaration(package, &[], "main");
    let out = compile_ken_source(
        package,
        CompilerSource::new("src/main.ken", "const main : Bool = True"),
        TargetSelector::StableSymbol {
            package_identity,
            symbol: target,
            kind: CompilerTargetKind::Executable,
        },
    )
    .expect("NC10/NC11 source path emits checked-core package");

    let example = scalar_seed_example();
    let mut program = runtime_program(example.clone(), out.package.artifact_hash);
    program.package_identity = out.package.header.package_identity.to_string();
    program.core_semantic_hash = out.package.core_semantic_hash;

    let report =
        evaluate_runtime_ir_example(&program, &example, &RuntimeIrSeedEnvironment::empty())
            .expect("source-derived identity-bearing runtime artifact evaluates");

    assert_eq!(report.artifact.package_identity, program.package_identity);
    assert_eq!(
        report.artifact.core_semantic_hash,
        out.package.core_semantic_hash
    );
    assert_eq!(report.artifact.artifact_hash, out.package.artifact_hash);
    assert!(matches!(
        report.trust.source_level_proof,
        RuntimeIrEvidenceFact::Unavailable { .. }
    ));
}

#[test]
fn explicit_runtime_trap_is_an_observation_not_host_behavior() {
    let example = ken_runtime::RuntimeExample {
        name: "explicit-runtime-trap".to_string(),
        checked_core_shape: "diagnostic trap fixture".to_string(),
        ir: ken_runtime::RuntimeExpr::Trap(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "explicit fixture trap".to_string(),
        }),
        observation: RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "explicit fixture trap".to_string(),
        }),
    };
    let program = runtime_program(example.clone(), 0x1207);

    let report =
        evaluate_runtime_ir_example(&program, &example, &RuntimeIrSeedEnvironment::empty())
            .expect("explicit trap reports as a runtime observation");

    assert_eq!(report.observation.observation, example.observation);
}
