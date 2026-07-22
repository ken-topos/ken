//! Value and primitive lowering subject tests (RT-SPLIT §10.1/§10.2).
//!
//! Populated in slice 5 from the Architect's ruled row list
//! (`evt_3xvn8g7n5rv7m`): subject is the production mechanism a test
//! discriminates, not the entrypoint it enters through.

use super::*;

// Ruled test module: imports permitted here (AC-8 class 2).
use crate::nc5_seed_examples;

#[test]
fn cranelift_runs_scalar_seed_and_verifies_function() {
    let example = nc5_seed_examples()
        .into_iter()
        .find(|example| example.name == "closed-scalar-primitive")
        .expect("seed exists");

    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("native run succeeds");

    assert!(report.verifier_passed);
    assert_eq!(report.observation, example.observation);
    assert_eq!(
        report.trust.fidelity,
        NativeFidelity::F1SeedObservationAgreement
    );
}
#[test]
fn cranelift_reports_bytes_and_string_immediates_as_ground_values() {
    for (name, ir, observation) in [
        (
            "bytes-immediate",
            RuntimeExpr::Value(RuntimeValue::Bytes(vec![1, 2, 3])),
            RuntimeObservation::Returned(RuntimeGroundValue::Bytes(vec![1, 2, 3])),
        ),
        (
            "string-immediate",
            RuntimeExpr::Value(RuntimeValue::String("ken".to_string())),
            RuntimeObservation::Returned(RuntimeGroundValue::String("ken".to_string())),
        ),
    ] {
        let example = RuntimeExample {
            name: name.to_string(),
            checked_core_shape: "diagnostic label only".to_string(),
            ir,
            observation,
        };

        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("native run succeeds");

        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);
        assert!(
            report.native_returned.is_some(),
            "native function returns an opaque table token"
        );
    }
}
#[test]
fn cranelift_runs_closure_seed_with_explicit_runtime_capture_environment() {
    let example = nc5_seed_examples()
        .into_iter()
        .find(|example| example.name == "closure-capture-application")
        .expect("seed exists");

    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::nc5_seed())
        .expect("native run succeeds");

    assert!(report.verifier_passed);
    assert_eq!(report.observation, example.observation);

    let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect_err("missing capture must reject loudly");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Closure",
            ..
        })
    ));
}
#[test]
fn explicit_partial_primitive_reports_trap_not_backend_bug() {
    let example = nc5_seed_examples()
        .into_iter()
        .find(|example| example.name == "explicit-partial-primitive-trap")
        .expect("seed exists");

    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("trap report succeeds");

    assert!(report.verifier_passed);
    assert!(matches!(
        report.observation,
        RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            ..
        })
    ));
}
#[test]
fn safe_bytes_at_oob_lowers_to_none_with_bounds_obligation() {
    let none = "ctor:fixture::Option::None".to_string();
    let some = "ctor:fixture::Option::Some".to_string();
    let example = RuntimeExample {
        name: "safe-bytes-at-oob".to_string(),
        checked_core_shape: "bytes_at empty 0 : Option UInt8".to_string(),
        ir: RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "bytes_at".to_string(),
                partiality: RuntimePartiality::SafeOption {
                    none: none.clone(),
                    some,
                    obligation: Some("obl:bytes_at.bounds".to_string()),
                },
            },
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Bytes(Vec::new())),
                RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            ],
        },
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
            constructor: none,
            args: Vec::new(),
        }),
    };

    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("safe bytes_at native lowering succeeds");
    assert!(report.verifier_passed);
    assert_eq!(report.observation, example.observation);
}
#[test]
fn safe_bytes_slice_and_decode_native_results_are_explicit() {
    let none = "ctor:fixture::Option::None".to_string();
    let some = "ctor:fixture::Option::Some".to_string();
    let err = "ctor:fixture::Result::Err".to_string();
    let ok = "ctor:fixture::Result::Ok".to_string();
    let invalid = "ctor:fixture::Utf8Error::InvalidUtf8".to_string();
    let examples = [
        RuntimeExample {
            name: "safe-bytes-slice".to_string(),
            checked_core_shape: "bytes_slice [0,1,2] 1 2".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_slice".to_string(),
                    partiality: RuntimePartiality::SafeOption {
                        none,
                        some: some.clone(),
                        obligation: None,
                    },
                },
                args: vec![
                    RuntimeExpr::Value(RuntimeValue::Bytes(vec![0, 1, 2])),
                    RuntimeExpr::Value(RuntimeValue::Int((1).into())),
                    RuntimeExpr::Value(RuntimeValue::Int((2).into())),
                ],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
                constructor: some,
                args: vec![RuntimeGroundValue::Bytes(vec![1, 2])],
            }),
        },
        RuntimeExample {
            name: "safe-bytes-decode-invalid".to_string(),
            checked_core_shape: "bytes_decode [255]".to_string(),
            ir: RuntimeExpr::PrimitiveCall {
                primitive: RuntimePrimitive {
                    symbol: "bytes_decode".to_string(),
                    partiality: RuntimePartiality::SafeResult {
                        err: err.clone(),
                        ok,
                        error: invalid.clone(),
                    },
                },
                args: vec![RuntimeExpr::Value(RuntimeValue::Bytes(vec![0xff]))],
            },
            observation: RuntimeObservation::Returned(RuntimeGroundValue::Constructor {
                constructor: err,
                args: vec![RuntimeGroundValue::Constructor {
                    constructor: invalid,
                    args: Vec::new(),
                }],
            }),
        },
    ];

    for example in examples {
        let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
            .expect("safe Bytes native lowering succeeds");
        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);
    }
}
#[test]
fn checked_partial_primitive_still_rejects_unknown_arguments() {
    let example = RuntimeExample {
        name: "unknown-partial-arg".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::PrimitiveCall {
            primitive: RuntimePrimitive {
                symbol: "checked_index".to_string(),
                partiality: RuntimePartiality::CheckedTrap {
                    obligation: "obl:checked_index.bounds".to_string(),
                },
            },
            args: vec![RuntimeExpr::Value(RuntimeValue::Unknown)],
        },
        observation: RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "unused".to_string(),
        }),
    };

    let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect_err("unknown argument must reject before trap reporting");

    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Unknown",
            ..
        })
    ));
}
#[test]
fn overflowing_int_primitive_promotes_before_native_wrapping_semantics() {
    let example = RuntimeExample {
        name: "overflowing-add-int".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: total_primitive(
            "add_int",
            vec![
                RuntimeExpr::Value(RuntimeValue::Int((i64::MAX).into())),
                RuntimeExpr::Value(RuntimeValue::Int((1).into())),
            ],
        ),
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![1_u64 << 63],
            },
        )),
    };

    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("native lowering promotes before overflow");
    assert_eq!(report.observation, example.observation);
}
#[test]
fn px8i_big_small_big_mul_and_canonical_narrow_are_exact() {
    run_exact_int(
        total_primitive(
            "add_int",
            vec![
                big(crate::Sign::NonNegative, &[u64::MAX, 1]),
                RuntimeExpr::Value(RuntimeValue::Int(1.into())),
            ],
        ),
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::NonNegative,
            limbs: vec![0, 2],
        },
    );
    run_exact_int(
        total_primitive(
            "mul_int",
            vec![
                big(crate::Sign::NonNegative, &[0, 1]),
                big(crate::Sign::NonNegative, &[0, 1]),
            ],
        ),
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::NonNegative,
            limbs: vec![0, 0, 1],
        },
    );
    run_exact_int(
        total_primitive(
            "sub_int",
            vec![
                big(crate::Sign::NonNegative, &[1_u64 << 63]),
                RuntimeExpr::Value(RuntimeValue::Int(1.into())),
            ],
        ),
        crate::RuntimeIntV1::Small(i64::MAX),
    );
    run_exact_int(
        total_primitive(
            "add_int",
            vec![
                big(crate::Sign::Negative, &[0, 2]),
                RuntimeExpr::Value(RuntimeValue::Int(1.into())),
            ],
        ),
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::Negative,
            limbs: vec![u64::MAX, 1],
        },
    );
    run_exact_int(
        total_primitive(
            "sub_int",
            vec![
                RuntimeExpr::Value(RuntimeValue::Int(1.into())),
                big(crate::Sign::NonNegative, &[0, 2]),
            ],
        ),
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::Negative,
            limbs: vec![u64::MAX, 1],
        },
    );
}
#[test]
fn px8i_comparison_observes_high_limbs_and_dynamic_join_preserves_pair() {
    let lhs = big(crate::Sign::NonNegative, &[7, 1]);
    let rhs = big(crate::Sign::NonNegative, &[7, 2]);
    let condition = total_primitive("eq_int", vec![lhs.clone(), rhs]);
    run_exact_int(
        RuntimeExpr::If {
            scrutinee: Box::new(condition),
            then_expr: Box::new(big(crate::Sign::NonNegative, &[99, 9])),
            else_expr: Box::new(lhs),
        },
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::NonNegative,
            limbs: vec![7, 1],
        },
    );

    run_exact_int(
        RuntimeExpr::If {
            scrutinee: Box::new(total_primitive(
                "leq_int",
                vec![
                    RuntimeExpr::Value(RuntimeValue::Int(i64::MAX.into())),
                    big(crate::Sign::NonNegative, &[0, 1]),
                ],
            )),
            then_expr: Box::new(big(crate::Sign::NonNegative, &[17, 3])),
            else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((-1).into()))),
        },
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::NonNegative,
            limbs: vec![17, 3],
        },
    );
    run_exact_int(
        RuntimeExpr::If {
            scrutinee: Box::new(total_primitive(
                "leq_int",
                vec![
                    big(crate::Sign::Negative, &[0, 1]),
                    RuntimeExpr::Value(RuntimeValue::Int(i64::MIN.into())),
                ],
            )),
            then_expr: Box::new(big(crate::Sign::Negative, &[23, 4])),
            else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((-1).into()))),
        },
        crate::RuntimeIntV1::Big {
            sign: crate::Sign::Negative,
            limbs: vec![23, 4],
        },
    );
}
#[test]
fn px8i_wrapping_and_trap_mutations_are_causal_at_live_binop_lowering() {
    let expr = total_primitive(
        "add_int",
        vec![
            RuntimeExpr::Value(RuntimeValue::Int(i64::MAX.into())),
            RuntimeExpr::Value(RuntimeValue::Int(1.into())),
        ],
    );
    let example = RuntimeExample {
        name: "px8i-live-mutation".to_string(),
        checked_core_shape: "PX8-I live exact binop mutation".to_string(),
        ir: expr,
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![1_u64 << 63],
            },
        )),
    };

    NATIVE_INT_LOWERING_MUTATION.with(|mutation| mutation.set(NativeIntLoweringMutation::Wrapping));
    let wrapping = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("wrapping mutation still emits the live native expression");
    NATIVE_INT_LOWERING_MUTATION.with(|mutation| mutation.set(NativeIntLoweringMutation::Exact));
    assert_ne!(wrapping.observation, example.observation);
    assert_eq!(
        wrapping.observation,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(i64::MIN.into()))
    );

    NATIVE_INT_LOWERING_MUTATION.with(|mutation| mutation.set(NativeIntLoweringMutation::Trap));
    let trapped = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty());
    NATIVE_INT_LOWERING_MUTATION.with(|mutation| mutation.set(NativeIntLoweringMutation::Exact));
    assert!(matches!(
        trapped,
        Err(CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "PrimitiveCall",
            ..
        }))
    ));
}
#[test]
fn px8i_jit_terminal_requires_uncorrupted_local_export_evidence() {
    let example = RuntimeExample {
        name: "px8i-terminal-export".to_string(),
        checked_core_shape: "PX8-I terminal Big export".to_string(),
        ir: big(crate::Sign::NonNegative, &[5, 1]),
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(
            crate::RuntimeIntV1::Big {
                sign: crate::Sign::NonNegative,
                limbs: vec![5, 1],
            },
        )),
    };
    for mutation in [
        NativeIntLoweringMutation::SuppressTerminalExport,
        NativeIntLoweringMutation::CorruptTerminalExport,
    ] {
        NATIVE_INT_LOWERING_MUTATION.with(|cell| cell.set(mutation));
        let result = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty());
        NATIVE_INT_LOWERING_MUTATION.with(|cell| cell.set(NativeIntLoweringMutation::Exact));
        assert!(matches!(
            result,
            Err(CraneliftBackendError::Backend(
                BackendFailure::NativeResultDecode { .. }
            ))
        ));
    }
}
fn run_exact_int(expr: RuntimeExpr, expected: crate::RuntimeIntV1) {
    let direct = crate::evaluate_runtime_ir_expr(&expr, &crate::RuntimeIrSeedEnvironment::empty())
        .expect("backend-neutral Runtime IR evaluates exact Int expression");
    let example = RuntimeExample {
        name: "px8i-exact-int".to_string(),
        checked_core_shape: "PX8-I exact Int discriminator".to_string(),
        ir: expr,
        observation: RuntimeObservation::Returned(RuntimeGroundValue::Int(expected)),
    };
    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("exact Int expression lowers and executes");
    assert_eq!(direct, example.observation);
    assert_eq!(report.observation, example.observation);
}
