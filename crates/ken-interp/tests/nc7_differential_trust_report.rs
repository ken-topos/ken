use std::collections::BTreeSet;

use ken_interp::{eval, EvalStore, EvalVal};
use ken_kernel::{declare_primitive, GlobalEnv, Level, PrimReduction, Term};
use ken_runtime::{
    nc5_seed_examples, ErasedExecutableCore, InterpreterOracleObservation, NativeArtifactIdentity,
    NativeDifferentialStage, NativeDifferentialVerdict, NativeEvidenceFact, NativeFidelity,
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr, RuntimeField, RuntimeFieldStatus,
    RuntimeGroundValue, RuntimeLowerabilityStatus, RuntimeMetadata, RuntimeObservation,
    RuntimeProgram, RuntimeSymbolMetadata, RuntimeValue,
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

fn artifact_identity(artifact_hash: u64) -> NativeArtifactIdentity {
    NativeArtifactIdentity {
        package_identity: "module:fixture::nc7".to_string(),
        core_semantic_hash: 0x7001,
        runtime_artifact_hash: artifact_hash,
    }
}

fn oracle_observation(artifact: NativeArtifactIdentity) -> InterpreterOracleObservation {
    let OracleFixture {
        globals,
        mut store,
        term,
    } = interpreter_add_2_3_fixture();
    let value = eval(&[], &term, &globals, &mut store);
    let observation = match value {
        EvalVal::Int(value) => RuntimeObservation::Returned(RuntimeGroundValue::Int((value).into())),
        other => panic!("NC7 oracle fixture must return Int, got {other:?}"),
    };
    InterpreterOracleObservation {
        artifact,
        observation,
        evidence_source: "ken-interp eval over GlobalEnv + closed core Term: add_int 2 3"
            .to_string(),
    }
}

fn runtime_program(example: ken_runtime::RuntimeExample, artifact_hash: u64) -> RuntimeProgram {
    let symbol = "decl:fixture::Main::main".to_string();
    let mut metadata = RuntimeMetadata::default();
    metadata
        .lowerability
        .insert(symbol.clone(), RuntimeLowerabilityStatus::Supported);
    RuntimeProgram {
        package_identity: "module:fixture::nc7".to_string(),
        core_semantic_hash: 0x7001,
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

#[test]
fn interpreter_backed_f1_report_uses_real_oracle_not_seed_observation() {
    let example = scalar_seed_example();
    let program = runtime_program(example.clone(), 0x7002);
    let artifact = artifact_identity(0x7002);

    let report = ken_runtime::run_example_with_interpreter_observation(
        &program,
        &example,
        &ken_runtime::NativeSeedEnvironment::empty(),
        oracle_observation(artifact.clone()),
    );

    assert_eq!(report.oracle.artifact, artifact);
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
    assert_eq!(report.artifact.package_identity, "module:fixture::nc7");
    assert_eq!(report.artifact.core_semantic_hash, 0x7001);
    assert_eq!(report.artifact.runtime_artifact_hash, 0x7002);
    assert!(matches!(
        native.trust.toolchain.cranelift,
        NativeEvidenceFact::Unavailable { .. }
    ));
    assert!(matches!(
        native.trust.toolchain.linker,
        NativeEvidenceFact::Unavailable { .. }
    ));
    assert!(matches!(
        native.trust.toolchain.runtime,
        NativeEvidenceFact::Available { .. }
    ));
    assert!(report
        .oracle
        .evidence_source
        .contains("ken-interp eval over GlobalEnv"));
    assert_eq!(
        native.observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int((5).into()))
    );
}

#[test]
fn mismatch_report_names_compare_stage_after_both_sides_run() {
    let mut example = scalar_seed_example();
    example.name = "mismatched-runtime-ir".to_string();
    example.ir = RuntimeExpr::Value(RuntimeValue::Int((4).into()));
    let program = runtime_program(example.clone(), 0x7003);
    let artifact = artifact_identity(0x7003);

    let report = ken_runtime::run_example_with_interpreter_observation(
        &program,
        &example,
        &ken_runtime::NativeSeedEnvironment::empty(),
        oracle_observation(artifact),
    );

    assert!(report.native.is_some(), "native side must have run");
    assert_eq!(report.artifact.runtime_artifact_hash, 0x7003);
    assert!(matches!(
        report.verdict,
        NativeDifferentialVerdict::Mismatch {
            stage: NativeDifferentialStage::InterpreterNativeCompare,
            interpreter: RuntimeObservation::Returned(RuntimeGroundValue::Int(
                ken_runtime::RuntimeIntV1::Small(5),
            )),
            native: RuntimeObservation::Returned(RuntimeGroundValue::Int(
                ken_runtime::RuntimeIntV1::Small(4),
            )),
        }
    ));
}

#[test]
fn unsupported_preflight_report_emits_no_differential_claim() {
    let example = scalar_seed_example();
    let mut program = runtime_program(example.clone(), 0x7004);
    let artifact = artifact_identity(0x7004);
    program
        .erased_core
        .metadata
        .effects
        .insert("Console".to_string());

    let report = ken_runtime::run_example_with_interpreter_observation(
        &program,
        &example,
        &ken_runtime::NativeSeedEnvironment::empty(),
        oracle_observation(artifact),
    );

    assert!(report.native.is_none());
    assert!(matches!(
        report.verdict,
        NativeDifferentialVerdict::Unsupported {
            stage: NativeDifferentialStage::BoundaryPreflight,
            construct: "RuntimeProgram",
            ..
        }
    ));
}

#[test]
fn oracle_identity_mismatch_emits_no_f1_and_does_not_run_native() {
    let example = scalar_seed_example();
    let program = runtime_program(example.clone(), 0x7005);
    let wrong_artifact = artifact_identity(0x7777);

    let report = ken_runtime::run_example_with_interpreter_observation(
        &program,
        &example,
        &ken_runtime::NativeSeedEnvironment::empty(),
        oracle_observation(wrong_artifact),
    );

    assert!(report.native.is_none());
    assert_eq!(report.artifact, artifact_identity(0x7005));
    assert_eq!(report.oracle.artifact, artifact_identity(0x7777));
    assert!(matches!(
        report.verdict,
        NativeDifferentialVerdict::Unsupported {
            stage: NativeDifferentialStage::BoundaryPreflight,
            construct: "InterpreterOracleObservation",
            ..
        }
    ));
}
