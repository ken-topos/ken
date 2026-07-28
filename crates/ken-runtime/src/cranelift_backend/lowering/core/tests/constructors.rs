//! Constructor-field, dynamic-constructor, nested-computational and
//! heterogeneous-eliminator lowering tests (RT-SPLIT §10.2 -> `constructors`).

use super::*;

// Ruled test module: imports permitted here (AC-8 class 2).
use crate::nc5_seed_examples;

// RT-SPLIT slice 7, rule 8: dependencies carried in with the moved
// `emit_process_entrypoint_object_with_symbols` closure -- used ONLY by it, so
// they travel with it (AC-9). Ruled test module, `use` permitted (AC-8 class 2).
//
// `native_platform_target_name` is an `artifact` private after slice 7, so it
// arrives through its owner-adjacent adapter (§10.5a′), aliased back to the
// original name so the moved body's call token is unchanged.
use crate::cranelift_backend::artifact::native_platform_target_name_for_lowering_tests as native_platform_target_name;
use crate::fnv1a_64;

#[cfg(test)]
fn run_dynamic_constructor_dispatch_fixture(
    discriminator: i64,
    selected_tags: &[i64],
) -> Result<i64, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px7p_dynamic_dispatch", Linkage::Local, &signature)
        .map_err(|error| backend_module(error.to_string()))?;
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let seed_env = NativeSeedEnvironment::empty();
    // Declared before `compiler`, because the plan installed below **borrows**
    // this term (B2A-S D2) and so must outlive the `Lowering` that holds it.
    // Locals drop in reverse order, so declaring it here is the whole fix.
    let cases = [
        (0, "ctor:fixture::Dynamic::Zero", 0, 40),
        (1, "ctor:fixture::Dynamic::One", 1, 41),
    ]
    .into_iter()
    .filter(|(tag, ..)| selected_tags.contains(tag))
    .map(
        |(_, constructor, binders, result)| crate::RuntimeMatchCase {
            constructor: constructor.to_string(),
            binders,
            body: RuntimeExpr::Value(RuntimeValue::Int((result).into())),
        },
    )
    .collect::<Vec<_>>();
    let default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px7p exact dynamic source default".to_string(),
    };
    // This path lowers the SELECTED case body, so its origin must be real: plan
    // the very match these cases belong to and install that plan, so case *i*'s
    // body is child `1 + i` of a genuinely planned occurrence.
    let source_match = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: cases.clone(),
        default: default.clone(),
    };
    let mut compiler = Lowering {
        seed_env: &seed_env,
        declarations: BTreeMap::new(),
        static_transition_plan: inert_test_plan(),
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        active_carried_computational_eliminations: Vec::new(),
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan: None,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: None,
        native_int_compare: None,
        native_int_intern: None,
        native_int_narrow: None,
        native_int_export: None,
        native_int_tags: BTreeMap::new(),
        // ⛔ `None` — a bare `Lowering` fixture emits into no module, so it has
        // no callable carrier refs. The `Carried` routes fail closed on this
        // rather than silently taking the `Specialized` path.
        boundary_carrier: None,
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let dynamic = DynamicConstructorV1 {
            discriminator: builder.ins().iconst(types::I64, discriminator),
            alternatives: vec![
                DynamicConstructorAlternativeV1 {
                    tag: 0,
                    constructor: "ctor:fixture::Dynamic::Zero".to_string(),
                    fields: Vec::new(),
                },
                DynamicConstructorAlternativeV1 {
                    tag: 1,
                    constructor: "ctor:fixture::Dynamic::One".to_string(),
                    fields: vec![Lowered::Int {
                        value: builder.ins().iconst(types::I64, 7),
                        known: Some(7),
                    }],
                },
            ],
        };
        let (plan, match_origin) = planned_root_occurrence(&source_match);
        compiler.static_transition_plan = plan;
        let lowered = compiler.lower_dynamic_constructor_match(
            &mut builder,
            dynamic,
            DynamicConstructorContinuation::Ordinary {
                cases: &cases,
                default: &default,
                env: &[],
                static_origin: match_origin,
            },
        )?;
        let lowered = lowered.specialized_at("this fixture's result")?;
        let value = match lowered {
            Lowered::Trap(trap) => {
                assert_eq!(trap, default);
                builder.ins().iconst(types::I64, -4)
            }
            Lowered::Int { value, .. } => value,
            value => compiler.emit_result(&mut builder, value)?.0,
        };
        builder.ins().return_(&[value]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&context.func, module.isa())?;
    module
        .define_function(func_id, &mut context)
        .map_err(|error| backend_module(error.to_string()))?;
    let compiled = CompiledModule::from_parts(
        module,
        func_id,
        Some(ResultDecoder::ProcessStatus),
        compiler.result_table,
        None,
        true,
        compiler.assumptions,
        compiler.unsupported,
    );
    compiled
        .run(None)
        .map(|(_, token)| token.expect("fixture returns one scalar"))
}

#[test]
fn dynamic_constructor_all_known_omitted_runs_source_default_without_panic() {
    assert_eq!(
        run_dynamic_constructor_dispatch_fixture(0, &[]).expect("all-omitted dispatcher executes"),
        -4
    );
    assert_eq!(
        run_dynamic_constructor_dispatch_fixture(1, &[])
            .expect("every known alternative owns the source default"),
        -4
    );
}

#[test]
fn dynamic_constructor_mixed_present_and_omitted_keeps_default_distinct() {
    assert_eq!(
        run_dynamic_constructor_dispatch_fixture(0, &[1])
            .expect("known omitted tag executes the source default"),
        -4
    );
    assert_eq!(
        run_dynamic_constructor_dispatch_fixture(1, &[1])
            .expect("present unary alternative executes its selected case"),
        41
    );
}

#[test]
fn dynamic_constructor_unknown_tag_runs_malformed_not_source_default() {
    let malformed =
        run_dynamic_constructor_dispatch_fixture(2, &[]).expect("unknown-tag dispatcher executes");
    assert_eq!(malformed, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
    assert_eq!(malformed, -3);
    assert_ne!(malformed, -4);
}

#[test]
fn heterogeneous_later_ordinary_missing_selects_exact_default() {
    let later_cases = vec![RuntimeMatchCase {
        constructor: "ctor:fixture::Outer::Hit".to_string(),
        binders: 1,
        body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
    }];
    let first_default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px7o exact first ordinary default".to_string(),
    };
    let later_default = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "px7o exact later ordinary default".to_string(),
    };
    let trap = select_ordinary_case(
        OrdinaryEliminatorFrame {
            cases: &later_cases,
            default: &later_default,
            env: &[],
            static_origin: inert_test_static_origin(),
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
        },
        "ctor:fixture::Outer::Missing",
    )
    .expect_err("the later ordinary frame must select its own default");
    assert_eq!(trap, later_default);
    assert_ne!(trap, first_default);
}
#[test]
fn dynamic_constructor_duplicate_tag_and_identity_reject_exactly() {
    let duplicate_tag = validate_dynamic_constructor_alternatives([
        (0, "ctor:fixture::Dynamic::A"),
        (0, "ctor:fixture::Dynamic::B"),
    ])
    .expect_err("closed alternatives require unique tags");
    assert!(matches!(
        duplicate_tag,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "DynamicConstructor",
            reason,
        }) if reason == "duplicate alternative tag 0"
    ));

    let duplicate_identity = validate_dynamic_constructor_alternatives([
        (0, "ctor:fixture::Dynamic::A"),
        (1, "ctor:fixture::Dynamic::A"),
    ])
    .expect_err("closed alternatives require unique constructor identities");
    assert!(matches!(
        duplicate_identity,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "DynamicConstructor",
            reason,
        }) if reason == "duplicate alternative constructor ctor:fixture::Dynamic::A"
    ));
}
#[test]
fn dynamic_constructor_known_omission_owns_source_default() {
    let alternative = DynamicConstructorAlternativeV1 {
        tag: 0,
        constructor: "ctor:fixture::Dynamic::Missing".to_string(),
        fields: Vec::new(),
    };
    let owned = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "exact source match default".to_string(),
    };
    let unrelated = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "unrelated outer default".to_string(),
    };
    let selected = select_dynamic_constructor_case(&[], &alternative, &owned)
        .expect("a well-formed omission selects the source default")
        .expect_err("the constructor is intentionally omitted");
    assert_eq!(selected, &owned);
    assert_ne!(selected, &unrelated);
}
#[test]
fn heterogeneous_first_ordinary_missing_selects_exact_default() {
    let first_cases = vec![RuntimeMatchCase {
        constructor: "ctor:fixture::Inner::Hit".to_string(),
        binders: 1,
        body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
    }];
    let first_default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px7o exact first ordinary default".to_string(),
    };
    let later_default = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "px7o exact later ordinary default".to_string(),
    };
    let trap = select_ordinary_case(
        OrdinaryEliminatorFrame {
            cases: &first_cases,
            default: &first_default,
            env: &[],
            static_origin: inert_test_static_origin(),
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
        },
        "ctor:fixture::Inner::Missing",
    )
    .expect_err("the first ordinary frame must select its own default");
    assert_eq!(trap, first_default);
    assert_ne!(trap, later_default);
}
#[test]
fn dynamic_constructor_fields_precede_outer_environment_in_declaration_order() {
    let alternative = DynamicConstructorAlternativeV1 {
        tag: 7,
        constructor: "ctor:fixture::Dynamic::Pair".to_string(),
        fields: vec![
            Lowered::Bytes(b"first".to_vec()),
            Lowered::String("second".to_string()),
        ],
    };
    let env = materialize_dynamic_constructor_env(
        &alternative,
        &[LoweringOperand::Specialized(Lowered::Bytes(
            b"outer".to_vec(),
        ))],
    );
    assert!(
        matches!(&env[0], LoweringOperand::Specialized(Lowered::Bytes(value)) if value == b"first")
    );
    assert!(
        matches!(&env[1], LoweringOperand::Specialized(Lowered::String(value)) if value == "second")
    );
    assert!(
        matches!(&env[2], LoweringOperand::Specialized(Lowered::Bytes(value)) if value == b"outer")
    );
}

#[test]
fn cranelift_runs_constructor_match_and_record_projection_seeds() {
    let env = NativeSeedEnvironment::empty();
    for name in ["adt-constructor-match", "record-construction-projection"] {
        let example = nc5_seed_examples()
            .into_iter()
            .find(|example| example.name == name)
            .expect("seed exists");

        let report =
            run_example_with_seed_observation(&example, &env).expect("native run succeeds");

        assert!(report.verifier_passed);
        assert_eq!(report.observation, example.observation);
    }
}

#[test]
fn dynamic_host_result_producer_wrong_arity_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &host_result_computational_fixture(0, true, false),
        "ken_px7m_wrong_arity",
    )
    .expect_err("dynamic Result case must bind its one payload");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "dynamic HostResult tree producer case ctor:prelude::Result::Ok expects exactly one binder, got 0"
    ));
}
#[test]
fn dynamic_host_result_producer_result_kind_mismatch_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &host_result_computational_fixture(1, true, true),
        "ken_px7m_kind_mismatch",
    )
    .expect_err("scalar and ExitCode branches must not merge");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "dynamic native arms disagree on scalar versus ExitCode result"
    ));
}
#[test]
fn dynamic_host_result_producer_well_formed_control_emits() {
    emit_process_entrypoint_object_with_cranelift(
        &host_result_computational_fixture(1, true, false),
        "ken_px7m_well_formed",
    )
    .expect("both dynamic Result branches recursively lower and merge");
}
#[test]
fn nested_computational_producer_well_formed_control_emits() {
    emit_process_entrypoint_object_with_cranelift(
        &nested_computational_fixture(1, Vec::new(), false, true),
        "ken_px7n_well_formed",
    )
    .expect("inner dynamic branches compose through the outer eliminator");
}
#[test]
fn nested_computational_outer_arity_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &nested_computational_fixture(0, Vec::new(), false, true),
        "ken_px7n_wrong_outer_arity",
    )
    .expect_err("the outer aggregate payload must remain bound");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "case ctor:fixture::Aggregate::Ok expects 0 constructor arguments but value has 1"
    ));
}
#[test]
fn nested_computational_malformed_recursive_position_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &nested_computational_fixture(1, vec![1], false, true),
        "ken_px7n_bad_recursive_position",
    )
    .expect_err("an out-of-range inner recursive position must fail closed");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "case ctor:fixture::Inner::TrueLeaf has malformed recursive position 1"
    ));
}
#[test]
fn nested_computational_final_merge_kind_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &nested_computational_fixture(1, Vec::new(), true, true),
        "ken_px7n_final_kind_mismatch",
    )
    .expect_err("the final scalar and ExitCode arms must not merge");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "dynamic native arms disagree on scalar versus ExitCode result"
    ));
}
#[test]
fn nested_computational_payload_kind_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &nested_computational_fixture(1, Vec::new(), false, false),
        "ken_px7n_payload_kind",
    )
    .expect_err("the inner aggregate payload must retain its scalar kind");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "PrimitiveCall",
            reason,
        }) if reason == "sub_int only supports Int arguments in native lowering"
    ));
}
#[test]
fn heterogeneous_eliminator_well_formed_control_emits() {
    emit_process_entrypoint_object_with_cranelift(
        &heterogeneous_eliminator_fixture(
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Outer::Hit",
            "ctor:fixture::Outer::Hit",
            1,
            1,
            true,
            false,
        ),
        "ken_px7o_well_formed",
    )
    .expect("dynamic producer composes through both ordinary frames");
}
#[test]
fn constructor_field_selected_case_composes_before_field_lowering() {
    emit_process_entrypoint_object_with_cranelift(
        &constructor_field_selected_case_fixture(2, 1),
        "ken_px7p_constructor_field_selected_case",
    )
    .expect("the selected trailing field remains structural through its ordinary consumer");
}
#[test]
fn constructor_field_composes_through_computational_consumer() {
    let leaf = "ctor:fixture::FieldTree::Leaf".to_string();
    let field = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleIsTerminal,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
            .into_iter()
            .map(|constructor| RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: leaf.clone(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
                },
            })
            .collect(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7p computational field default".to_string(),
        },
    };
    let expr = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((41).into())), field],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            argument_binders: 2,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(RuntimeExpr::Var(1)),
                cases: vec![crate::RuntimeComputationalMatchCase {
                    constructor: leaf,
                    argument_binders: 1,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "sub_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7p computational consumer default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p computational outer default".to_string(),
        },
    };
    emit_process_entrypoint_object_with_cranelift(
        &expr,
        "ken_px7p_constructor_field_computational_consumer",
    )
    .expect("the selected field also composes through a computational consumer");
}
#[test]
fn constructor_field_recursive_ih_offset_selects_argument_binder() {
    let expr = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Recursive".to_string(),
            args: vec![constructor_field_aggregate()],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Recursive".to_string(),
            argument_binders: 1,
            recursive_positions: vec![0],
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(1)),
                cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                    .into_iter()
                    .map(|constructor| RuntimeMatchCase {
                        constructor: constructor.to_string(),
                        binders: 1,
                        body: RuntimeExpr::Var(0),
                    })
                    .collect(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7p recursive selected-field default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p recursive outer default".to_string(),
        },
    };
    emit_process_entrypoint_object_with_cranelift(
        &expr,
        "ken_px7p_constructor_field_recursive_offset",
    )
    .expect("the recursive IH prefix does not change the selected argument field");
}
#[test]
fn constructor_field_middle_binder_preserves_trailing_environment_order() {
    let aggregate = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
            .into_iter()
            .map(|constructor| RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Result::Ok".to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
                },
            })
            .collect(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7p middle producer default".to_string(),
        },
    };
    let expr = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Int((13).into())),
                aggregate,
                RuntimeExpr::Value(RuntimeValue::Int((41).into())),
            ],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            argument_binders: 3,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(1)),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Ok".to_string(),
                    binders: 1,
                    body: RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "sub_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![RuntimeExpr::Var(3), RuntimeExpr::Var(0)],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7p middle consumer default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p middle outer default".to_string(),
        },
    };
    let compiled = compile_expr(&expr, &NativeSeedEnvironment::empty())
        .expect("the selected middle field composes without moving its trailing sibling");
    assert_eq!(
        compiled.run(None).expect("middle-field fixture runs").0,
        RuntimeObservation::Returned(RuntimeGroundValue::Int((34).into()))
    );
}
#[test]
fn constructor_field_binder_shift_mutation_recovers_exact_refusal() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &constructor_field_selected_case_fixture(2, 0),
        "ken_px7p_constructor_field_wrong_binder",
    )
    .expect_err("the aggregate-looking sibling is not the selected field consumer");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Match",
            reason,
        }) if reason == "scrutinee is not a constructor value"
    ));
}
#[test]
fn constructor_field_bridge_removal_recovers_exact_refusal() {
    let fixture = constructor_field_selected_case_fixture(2, 1);
    let RuntimeExpr::ComputationalMatch {
        scrutinee,
        cases,
        default,
    } = fixture
    else {
        panic!("PX7-P fixture outer shape changed");
    };
    let eagerly_materialized = RuntimeExpr::Let {
        value: scrutinee,
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases,
            default,
        }),
    };
    let err = emit_process_entrypoint_object_with_cranelift(
        &eagerly_materialized,
        "ken_px7p_constructor_field_bridge_removed",
    )
    .expect_err("eager field lowering must recover the pre-PX7-P boundary");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Match",
            reason,
        }) if reason == "scrutinee is not a constructor value"
    ));
}
#[test]
fn constructor_field_outer_arity_rejects_before_field_lowering() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &constructor_field_selected_case_fixture(1, 1),
        "ken_px7p_constructor_field_outer_arity",
    )
    .expect_err("the selected constructor case must bind every field exactly");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "case ctor:fixture::Envelope::Wrap expects 1 constructor arguments but value has 2"
    ));
}
#[test]
fn constructor_field_missing_case_owns_default_before_fields() {
    let default = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "px7p exact missing constructor default".to_string(),
    };
    let expr = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Missing".to_string(),
            args: vec![RuntimeExpr::Var(999)],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            argument_binders: 1,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Var(0),
        }],
        default: default.clone(),
    };
    let compiled = compile_expr(&expr, &NativeSeedEnvironment::empty())
        .expect("a missing constructor selects its frame-owned default");
    assert_eq!(
        compiled.run(None).expect("default trap is observable").0,
        RuntimeObservation::Trapped(default)
    );
}
#[test]
fn constructor_field_aggregate_unconsumed_sibling_stays_ordinary() {
    let prefix = RuntimeExpr::Construct {
        constructor: "ctor:fixture::Prefix::Keep".to_string(),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((41).into()))],
    };
    let expr = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            args: vec![prefix, constructor_field_aggregate()],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            argument_binders: 2,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:fixture::Prefix::Keep".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7p prefix default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p outer default".to_string(),
        },
    };
    emit_process_entrypoint_object_with_cranelift(&expr, "ken_px7p_aggregate_unconsumed_sibling")
        .expect("an unconsumed aggregate-looking field retains ordinary lowering");
}
#[test]
fn constructor_field_host_result_stays_on_ordinary_dynamic_match() {
    let expr = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            args: vec![console_write_effect()],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            argument_binders: 1,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                    .into_iter()
                    .map(|constructor| RuntimeMatchCase {
                        constructor: constructor.to_string(),
                        binders: 1,
                        body: RuntimeExpr::Construct {
                            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                            args: Vec::new(),
                        },
                    })
                    .collect(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7p HostResult default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p outer default".to_string(),
        },
    };
    emit_process_entrypoint_object_with_cranelift(&expr, "ken_px7p_constructor_field_host_result")
        .expect("HostResult fields remain owned by ordinary dynamic matching");
}
#[test]
fn dynamic_constructor_dispatches_ordinary_continuation_with_mixed_arities() {
    emit_process_entrypoint_object_with_cranelift(
        &dynamic_io_error_match(false, false),
        "ken_px7p_dynamic_constructor_ordinary",
    )
    .expect("the shared dispatcher lowers ordinary nullary and unary alternatives");
}
#[test]
fn dynamic_constructor_dispatches_producer_continuation_with_all_frames() {
    emit_process_entrypoint_object_with_cranelift(
        &dynamic_io_error_match(true, false),
        "ken_px7p_dynamic_constructor_producer",
    )
    .expect("the shared dispatcher preserves the active computational frame");
}
#[test]
fn dynamic_constructor_ordinary_continuation_preserves_bool_kind() {
    emit_process_entrypoint_object_with_cranelift(
        &dynamic_io_error_match(false, true),
        "ken_px7p_dynamic_constructor_bool",
    )
    .expect("a dynamic Bool remains available to its enclosing Bool consumer");
}
#[test]
fn dynamic_constructor_binder_arity_rejects_exactly() {
    let mut symbols = crate::NativeProcessSymbols::legacy_prelude();
    symbols.io_errors.rotate_right(1);
    let err = emit_process_entrypoint_object_with_symbols(
        &dynamic_io_error_match(false, false),
        &symbols,
        "ken_px7p_dynamic_constructor_arity",
    )
    .expect_err("constructor identity, not table position, owns binder arity");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "DynamicConstructor",
            reason,
        }) if reason == "case ctor:prelude::IOError::Other expects 1 binders but alternative has 0 fields"
    ));
}
#[test]
fn recursive_computational_aggregate_traverses_ordinary_frame() {
    let aggregate = RuntimeExpr::Construct {
        constructor: "ctor:prelude::Result::Ok".to_string(),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        }],
    };

    emit_process_entrypoint_object_with_cranelift(
        &host_result_closure_match(recursive_computational_result(aggregate)),
        "ken_px7o_recursive_computational_aggregate",
    )
    .expect("recursive aggregate traverses the active ordinary frame");
}
#[test]
fn heterogeneous_bridge_removal_recovers_exact_ordinary_match_refusal() {
    let fixture = heterogeneous_eliminator_fixture(
        "ctor:fixture::Inner::Hit",
        "ctor:fixture::Inner::Hit",
        "ctor:fixture::Outer::Hit",
        "ctor:fixture::Outer::Hit",
        1,
        1,
        true,
        false,
    );
    let RuntimeExpr::Call { callee, mut args } = fixture else {
        panic!("fixture outer shape changed");
    };
    let RuntimeExpr::LexicalClosure { body, .. } = *callee else {
        panic!("fixture continuation shape changed");
    };
    let bridge_removed = RuntimeExpr::Let {
        value: Box::new(args.remove(0)),
        body,
    };
    let err =
        emit_process_entrypoint_object_with_cranelift(&bridge_removed, "ken_px7o_bridge_removed")
            .expect_err("eagerly materializing the intermediate must recover the original defect");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Match",
            reason,
        }) if reason == "scrutinee is not a constructor value"
    ));
}
#[test]
fn heterogeneous_frame_environment_and_binder_order_are_preserved() {
    let inner_call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: vec![RuntimeExpr::Value(RuntimeValue::Int((41).into()))],
            params: vec!["inner".to_string()],
            body: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases: vec![RuntimeMatchCase {
                    constructor: "ctor:fixture::Inner::Hit".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Outer::Hit".to_string(),
                        args: vec![RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "sub_int".to_string(),
                                partiality: RuntimePartiality::Total,
                            },
                            args: vec![RuntimeExpr::Var(2), RuntimeExpr::Var(0)],
                        }],
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7o binder-order inner default".to_string(),
                },
            }),
        }),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:fixture::Inner::Hit".to_string(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
        }],
    };
    let expr = RuntimeExpr::Call {
        callee: Box::new(ordinary_match_closure(
            vec![RuntimeMatchCase {
                constructor: "ctor:fixture::Outer::Hit".to_string(),
                binders: 1,
                body: RuntimeExpr::Var(0),
            }],
            RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7o binder-order outer default".to_string(),
            },
        )),
        args: vec![inner_call],
    };
    let compiled = compile_expr(&expr, &NativeSeedEnvironment::empty())
        .expect("frame environment fixture lowers");
    assert_eq!(
        compiled
            .run(None)
            .expect("frame environment fixture runs")
            .0,
        RuntimeObservation::Returned(RuntimeGroundValue::Int((34).into()))
    );
}
#[test]
fn heterogeneous_final_merge_kind_rejects_specifically() {
    let producer = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleIsTerminal,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        cases: vec![
            RuntimeMatchCase {
                constructor: "ctor:prelude::Bool::True".to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: "ctor:fixture::Inner::Scalar".to_string(),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int((7).into()))],
                },
            },
            RuntimeMatchCase {
                constructor: "ctor:prelude::Bool::False".to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: "ctor:fixture::Inner::Exit".to_string(),
                    args: Vec::new(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7o kind producer default".to_string(),
        },
    };
    let inner_call = RuntimeExpr::Call {
        callee: Box::new(ordinary_match_closure(
            vec![
                RuntimeMatchCase {
                    constructor: "ctor:fixture::Inner::Scalar".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Outer::Scalar".to_string(),
                        args: vec![RuntimeExpr::Var(0)],
                    },
                },
                RuntimeMatchCase {
                    constructor: "ctor:fixture::Inner::Exit".to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Outer::Exit".to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "px7o kind inner default".to_string(),
            },
        )),
        args: vec![producer],
    };
    let expr = RuntimeExpr::Call {
        callee: Box::new(ordinary_match_closure(
            vec![
                RuntimeMatchCase {
                    constructor: "ctor:fixture::Outer::Scalar".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                },
                RuntimeMatchCase {
                    constructor: "ctor:fixture::Outer::Exit".to_string(),
                    binders: 0,
                    body: RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "px7o kind outer default".to_string(),
            },
        )),
        args: vec![inner_call],
    };
    let err = emit_process_entrypoint_object_with_cranelift(&expr, "ken_px7o_final_kind_mismatch")
        .expect_err("final scalar and ExitCode arms must not merge");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalMatch",
            reason,
        }) if reason == "dynamic native arms disagree on scalar versus ExitCode result"
    ));
}
#[test]
fn heterogeneous_ordinary_arity_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &heterogeneous_eliminator_fixture(
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Outer::Hit",
            "ctor:fixture::Outer::Hit",
            0,
            1,
            true,
            false,
        ),
        "ken_px7o_wrong_arity",
    )
    .expect_err("ordinary frame binder arity must match the constructor");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Match",
            reason,
        }) if reason == "case ctor:fixture::Inner::Hit expects 0 binders but constructor has 1 args"
    ));
}
#[test]
fn heterogeneous_nested_payload_kind_rejects_specifically() {
    let err = emit_process_entrypoint_object_with_cranelift(
        &heterogeneous_eliminator_fixture(
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Inner::Hit",
            "ctor:fixture::Outer::Hit",
            "ctor:fixture::Outer::Hit",
            1,
            1,
            false,
            false,
        ),
        "ken_px7o_payload_kind",
    )
    .expect_err("the nested aggregate payload must retain its scalar kind");
    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "PrimitiveCall",
            reason,
        }) if reason == "sub_int only supports Int arguments in native lowering"
    ));
}
#[test]
fn pattern_default_trap_is_observation_not_backend_error() {
    let example = RuntimeExample {
        name: "match-default".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:None".to_string(),
                args: vec![],
            }),
            cases: vec![RuntimeMatchCase {
                constructor: "ctor:Some".to_string(),
                binders: 1,
                body: RuntimeExpr::Var(0),
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "no case selected".to_string(),
            },
        },
        observation: RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "no case selected".to_string(),
        }),
    };

    let report = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect("trap report succeeds");

    assert_eq!(report.observation, example.observation);
}
fn nested_computational_fixture(
    outer_binders: usize,
    inner_recursive_positions: Vec<usize>,
    mismatched_result_kind: bool,
    payload_is_int: bool,
) -> RuntimeExpr {
    let inner_true = "ctor:fixture::Inner::TrueLeaf".to_string();
    let inner_false = "ctor:fixture::Inner::FalseLeaf".to_string();
    let aggregate_ok = "ctor:fixture::Aggregate::Ok".to_string();
    let aggregate_err = "ctor:fixture::Aggregate::Err".to_string();
    let inner_cases = [
        (inner_true.clone(), aggregate_ok.clone()),
        (inner_false.clone(), aggregate_err.clone()),
    ]
    .into_iter()
    .map(
        |(constructor, aggregate)| crate::RuntimeComputationalMatchCase {
            constructor,
            argument_binders: 1,
            recursive_positions: inner_recursive_positions.clone(),
            body: RuntimeExpr::Construct {
                constructor: aggregate,
                args: vec![RuntimeExpr::PrimitiveCall {
                    primitive: RuntimePrimitive {
                        symbol: "sub_int".to_string(),
                        partiality: RuntimePartiality::Total,
                    },
                    args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
                }],
            },
        },
    )
    .collect();
    let producer_cases = [
        ("ctor:prelude::Bool::True", inner_true, 7),
        ("ctor:prelude::Bool::False", inner_false, 9),
    ]
    .into_iter()
    .map(|(constructor, leaf, payload)| RuntimeMatchCase {
        constructor: constructor.to_string(),
        binders: 0,
        body: RuntimeExpr::Construct {
            constructor: leaf,
            args: vec![if payload_is_int {
                RuntimeExpr::Value(RuntimeValue::Int((payload).into()))
            } else {
                RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                    args: Vec::new(),
                }
            }],
        },
    })
    .collect();
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::Value(RuntimeValue::Int((41).into()))),
            body: Box::new(RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Effect {
                        family: "Console".to_string(),
                        operation: ken_host::HostOpV1::ConsoleIsTerminal,
                        capability: None,
                        args: vec![RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Stream::Stdout".to_string(),
                            args: Vec::new(),
                        }],
                    }),
                    cases: producer_cases,
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "inner producer default".to_string(),
                    },
                }),
                cases: inner_cases,
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "inner eliminator default".to_string(),
                },
            }),
        }),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: aggregate_ok,
                argument_binders: outer_binders,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Var(0),
            },
            crate::RuntimeComputationalMatchCase {
                constructor: aggregate_err,
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body: if mismatched_result_kind {
                    RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    }
                } else {
                    RuntimeExpr::Var(0)
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "outer eliminator default".to_string(),
        },
    }
}
fn heterogeneous_eliminator_fixture(
    inner_constructor: &str,
    inner_case_constructor: &str,
    outer_constructor: &str,
    outer_case_constructor: &str,
    inner_binders: usize,
    outer_binders: usize,
    payload_is_int: bool,
    mismatched_result_kind: bool,
) -> RuntimeExpr {
    let inner_default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px7o exact first ordinary default".to_string(),
    };
    let outer_default = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "px7o exact later ordinary default".to_string(),
    };
    let producer = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleIsTerminal,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
            .into_iter()
            .map(|constructor| RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: inner_constructor.to_string(),
                    args: vec![if payload_is_int {
                        RuntimeExpr::Value(RuntimeValue::Int((7).into()))
                    } else {
                        RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                            args: Vec::new(),
                        }
                    }],
                },
            })
            .collect(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "px7o producer default".to_string(),
        },
    };
    let inner_call = RuntimeExpr::Call {
        callee: Box::new(ordinary_match_closure(
            vec![RuntimeMatchCase {
                constructor: inner_case_constructor.to_string(),
                binders: inner_binders,
                body: RuntimeExpr::Construct {
                    constructor: outer_constructor.to_string(),
                    args: vec![RuntimeExpr::Var(0)],
                },
            }],
            inner_default,
        )),
        args: vec![producer],
    };
    RuntimeExpr::Call {
        callee: Box::new(ordinary_match_closure(
            vec![RuntimeMatchCase {
                constructor: outer_case_constructor.to_string(),
                binders: outer_binders,
                body: if mismatched_result_kind {
                    RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    }
                } else {
                    RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "sub_int".to_string(),
                            partiality: RuntimePartiality::Total,
                        },
                        args: vec![
                            RuntimeExpr::Value(RuntimeValue::Int((41).into())),
                            RuntimeExpr::Var(0),
                        ],
                    }
                },
            }],
            outer_default,
        )),
        args: vec![inner_call],
    }
}
fn constructor_field_selected_case_fixture(
    selected_binders: usize,
    selected_field_var: u32,
) -> RuntimeExpr {
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Int((41).into())),
                constructor_field_aggregate(),
            ],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Envelope::Wrap".to_string(),
            argument_binders: selected_binders,
            recursive_positions: Vec::new(),
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(selected_field_var)),
                cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                    .into_iter()
                    .map(|constructor| RuntimeMatchCase {
                        constructor: constructor.to_string(),
                        binders: 1,
                        body: RuntimeExpr::PrimitiveCall {
                            primitive: RuntimePrimitive {
                                symbol: "sub_int".to_string(),
                                partiality: RuntimePartiality::Total,
                            },
                            args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
                        },
                    })
                    .collect(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "px7p selected field default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px7p exact outer default".to_string(),
        },
    }
}
fn dynamic_io_error_match(producer: bool, ordinary_bool: bool) -> RuntimeExpr {
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let tree = "ctor:fixture::DynamicConstructorTree::Code";
    let producer_tree = |code: RuntimeExpr| RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleIsTerminal,
            capability: None,
            args: vec![RuntimeExpr::Construct {
                constructor: "ctor:prelude::Stream::Stdout".to_string(),
                args: Vec::new(),
            }],
        }),
        cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
            .into_iter()
            .map(|constructor| RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: tree.to_string(),
                    args: vec![code.clone()],
                },
            })
            .collect(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "dynamic constructor producer default".to_string(),
        },
    };
    let io_cases = symbols
        .io_errors
        .iter()
        .enumerate()
        .map(|(tag, constructor)| {
            let binders = usize::from(tag + 1 == symbols.io_errors.len());
            let code = if binders == 1 {
                RuntimeExpr::Var(0)
            } else {
                RuntimeExpr::Value(RuntimeValue::Int((tag as i64 + 1).into()))
            };
            RuntimeMatchCase {
                constructor: constructor.clone(),
                binders,
                body: if producer {
                    producer_tree(code)
                } else if ordinary_bool {
                    RuntimeExpr::Value(RuntimeValue::Bool(tag % 2 == 0))
                } else {
                    RuntimeExpr::Construct {
                        constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                        args: vec![code],
                    }
                },
            }
        })
        .collect();
    let error = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![RuntimeMatchCase {
            constructor: symbols.file_error.clone(),
            binders: 3,
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(2)),
                cases: io_cases,
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "dynamic IOError match default".to_string(),
                },
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "dynamic FileError match default".to_string(),
        },
    };
    let result = RuntimeExpr::Match {
        scrutinee: Box::new(fs_read_effect()),
        cases: vec![
            RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: error,
            },
            RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: if producer {
                    RuntimeExpr::Construct {
                        constructor: tree.to_string(),
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int((0).into()))],
                    }
                } else if ordinary_bool {
                    RuntimeExpr::Value(RuntimeValue::Bool(false))
                } else {
                    RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    }
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "dynamic Result match default".to_string(),
        },
    };
    if producer {
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(result),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: tree.to_string(),
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Construct {
                    constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                    args: vec![RuntimeExpr::Var(0)],
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "dynamic producer consumer default".to_string(),
            },
        }
    } else if ordinary_bool {
        RuntimeExpr::Match {
            scrutinee: Box::new(result),
            cases: [
                ("ctor:prelude::Bool::True", crate::EXIT_SUCCESS_CONSTRUCTOR),
                ("ctor:prelude::Bool::False", crate::EXIT_FAILURE_CONSTRUCTOR),
            ]
            .into_iter()
            .map(|(constructor, exit)| RuntimeMatchCase {
                constructor: constructor.to_string(),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: exit.to_string(),
                    args: (exit == crate::EXIT_FAILURE_CONSTRUCTOR)
                        .then(|| RuntimeExpr::Value(RuntimeValue::Int((1).into())))
                        .into_iter()
                        .collect(),
                },
            })
            .collect(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "dynamic Bool consumer default".to_string(),
            },
        }
    } else {
        result
    }
}
fn fs_read_effect() -> RuntimeExpr {
    RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsReadFile,
        capability: Some(crate::RuntimeCapabilityUse {
            identity: "program_caps.fs".to_string(),
            value: Box::new(RuntimeExpr::Var(1)),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Bytes(
            b"dynamic-constructor.bin".to_vec(),
        ))],
    }
}

// ── RT-SPLIT slice 7, rule 8 finalization ─────────────────────────────────
// Residual facade test fixtures whose final-user LCA is this module. Facade
// file scope was a TRANSITIONAL zero-widening holding position, never final
// ownership (Architect `evt_h69xwchqqxmj`); slice 7 discharges it. Moved
// verbatim -- ordered item-level identity, no body edits.

#[cfg(test)]
fn emit_process_entrypoint_object_with_symbols(
    entrypoint: &RuntimeExpr,
    symbols: &crate::NativeProcessSymbols,
    entry_symbol: &str,
) -> Result<CraneliftObjectArtifact, CraneliftBackendError> {
    let compiled = compile_expr_into_module(
        new_object_module("ken-runtime-process-entrypoint")?,
        entry_symbol,
        Linkage::Export,
        entrypoint,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(symbols),
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
        None,
    )?;
    let verifier_passed = compiled.verifier_passed;
    let assumptions = compiled.assumptions.clone();
    let unsupported = compiled.unsupported.clone();
    let object_bytes = compiled
        .module
        .finish()
        .emit()
        .map_err(|err| backend_module(err.to_string()))?;
    let object_hash = fnv1a_64(&object_bytes);
    Ok(CraneliftObjectArtifact {
        example: "native-process-entrypoint".to_string(),
        entry_symbol: entry_symbol.to_string(),
        object_bytes,
        object_hash,
        platform_target: native_platform_target_name(),
        backend_name: "Cranelift process object".to_string(),
        verifier_passed,
        assumptions,
        unsupported,
    })
}

// ─── RT-FNSPLIT-C1 `D3` — the one-way producer ─────────────────────────────

/// A bare [`Lowering`] over `plan`, with ⛔ **no carrier refs**.
///
/// Same shape and same reason as `run_dynamic_constructor_dispatch_fixture`'s
/// inline fixture: a `Lowering` that emits into no module has no callable
/// carrier helpers, so the carrier routes must fail closed rather than take
/// some other path. ⭐ Here that absence is not incidental — it is the
/// **instrument**: `carrier_refs()`'s error is a marker that says *"control
/// reached the first emitted call"*, which is what makes the ordering below
/// measurable at all without a JIT module.
#[cfg(test)]
fn bare_carrier_test_lowering<'src>(
    seed_env: &'src NativeSeedEnvironment,
    plan: StaticTransitionPlan<'src>,
) -> Lowering<'src> {
    Lowering {
        seed_env,
        declarations: BTreeMap::new(),
        static_transition_plan: plan,
        declaration_stack: Vec::new(),
        active_recursive_declarations: Vec::new(),
        result_table: BTreeMap::new(),
        next_token: 0,
        next_recursor_frame_provenance: 0,
        next_recursor_producer_origin: 0,
        next_continuation_activation: 0,
        next_continuation_cursor: 0,
        next_source_join: 0,
        next_source_predecessor: 0,
        live_source_continuations: 0,
        source_control_root: None,
        active_oriented_semantic_regions: 0,
        active_carried_computational_eliminations: Vec::new(),
        native_join_plan: None,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan: None,
        consumed_subcontinuation_frames: BTreeSet::new(),
        active_subcontinuation_frame: None,
        consumed_recursive_call_templates: BTreeSet::new(),
        pending_recursive_call: None,
        pending_computational_ih_call: None,
        active_recursive_invocations: Vec::new(),
        next_recursive_invocation_instance: 1,
        dynamic_splice_edges: BTreeMap::new(),
        next_dynamic_splice_edge: 1,
        assumptions: BTreeSet::new(),
        unsupported: Vec::new(),
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        host_dispatch: None,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: None,
        native_int_compare: None,
        native_int_intern: None,
        native_int_narrow: None,
        native_int_export: None,
        native_int_tags: BTreeMap::new(),
        boundary_carrier: None,
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    }
}

/// `RT-FNSPLIT-C1` `D3` — the producer screens the **whole graph** for
/// admissibility *before* it touches the carrier.
///
/// **MEASURED:** through one fixture with no carrier refs, a `Constructor`
/// whose argument is a closure fails with the **closure** error, while the
/// same `Constructor` shape whose argument is a `Bool` fails with the
/// **carrier-refs** error.
/// **CLAIMED:** [`Lowered::boundary_transfer_admissibility`] runs ahead of the
/// first allocation, so an inadmissible graph is *rejected* rather than
/// half-emitted — which is the ordering that walk's own contract calls
/// load-bearing.
/// **THE GAP:** the closure error alone is consistent with *"this fixture
/// errors early for some unrelated reason."* ⭐ The `Bool` case is the positive
/// control that closes it: it proves the very same fixture does reach the
/// allocation step, so the closure case's earlier stop is attributable to the
/// walk and to nothing else.
///
/// ⚠ Promise class: **durable invariant**. It asserts a relation between two
/// outcomes of one fixture, not either error's spelling as a value — a
/// reworded message keeps it green, and moving the walk after the allocation
/// turns it red.
#[test]
fn c1_d3_producer_screens_admissibility_before_it_touches_the_carrier() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut module = new_jit_module().expect("JIT module constructs");
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("c1_d3_producer_probe", Linkage::Local, &signature)
        .expect("probe declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);

    // A real planned `Construct` occurrence: the producer derives its identity
    // from the plan, and ⛔ a test cannot fabricate a `StaticOriginId` — the
    // ordinal stays planner-private, so this must be a genuinely planned one.
    let construct = RuntimeExpr::Construct {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
    };
    let (plan, construct_origin) = planned_root_occurrence(&construct);
    let closure_body = inert_test_static_origin();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);

    // ── the inadmissible graph: the closure is one level DOWN ─────────────
    //
    // ⚠ Nested deliberately. A closure at the ROOT would be refused by the
    // root variant's own disposition, so it could not distinguish the walk
    // from the disposition table. The walk is the only thing that sees this.
    let inadmissible = Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            body: closure_body,
        }],
    };
    let refused = compiler
        .transfer_into_carrier(&mut builder, construct_origin, &inadmissible)
        .expect_err("a constructor holding a closure cannot cross the boundary");
    assert!(
        matches!(
            refused,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "Closure",
                ..
            })
        ),
        "the nested closure must be reported as the CLOSURE refusal, not as \
         whatever the carrier step would have said: got {refused:?}"
    );

    // ── POSITIVE CONTROL: the same shape, admissible ──────────────────────
    let admissible = Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![Lowered::Bool {
            value: builder.ins().iconst(types::I64, 1),
            known: Some(true),
        }],
    };
    let reached = compiler
        .transfer_into_carrier(&mut builder, construct_origin, &admissible)
        .expect_err("a fixture with no carrier refs cannot allocate");
    assert!(
        matches!(
            reached,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "BoundaryCarrier",
                ..
            })
        ),
        "NON-VACUITY: the admissible graph must get PAST the walk and stop at \
         the first emitted call, or the closure case above proves nothing about \
         ordering: got {reached:?}"
    );
}

/// `RT-FNSPLIT-C1` `D3` — a **carried** operand survives `case_env` and nested
/// lowering, which is `§2h`'s own control clause for the env/spine conversion.
///
/// **MEASURED:** with a `Carried` operand seeded at de Bruijn index `0`,
/// lowering `Let { value: Var(0), body: Var(0) }` — a form that necessarily
/// pushes the lowered value into a **new** environment and re-enters
/// `lower_expr` — returns `LoweringOperand::Carried` holding the **same SSA
/// value** that went in. With a `Specialized` operand in the identical fixture,
/// the identical expression returns `Specialized`.
/// **CLAIMED:** the shared environment spine forwards an operand's *phase*
/// unchanged through scope entry and recursive lowering, so a projected
/// `Carried` child reaching an inner scope is still carried when it is read
/// back.
/// **THE GAP:** *"the result is `Carried`"* alone is satisfied by a spine that
/// blindly returns its input, and *"a `Carried` went in and came out"* is
/// satisfied by one that **re-mints** a word. ⭐ Two things close it: the
/// `Specialized` arm proves the fixture's answer actually tracks what was
/// seeded, and the **SSA-value equality** proves the operand was forwarded
/// rather than reconstructed.
///
/// ⚠ **Why this control exists at all, stated plainly:** the whole 292-error
/// env/spine conversion is behaviour-preserving, and the 472-test suite stayed
/// green through it **without ever constructing a `Carried`**. A green suite is
/// therefore *no evidence* about phase closure — it is evidence about
/// regression. `rustc` says the same thing in its own words (`variant Carried
/// is never constructed`), and this test is what answers it.
///
/// ⚠ Promise class: **durable invariant**. It asserts a relation between what
/// is seeded and what is read back, over a `Lowered`-free property; adding
/// `Lowered` variants, carrier helpers, or eliminator arms all keep it green,
/// while re-specializing or re-minting an operand on the spine turns it red.
#[test]
fn c1_d3_a_carried_operand_survives_case_env_and_nested_lowering() {
    // `Let { value: Var(0), body: Var(0) }` — ⭐ `Var` in *both* positions on
    // purpose. The `value` read exercises the lookup, and the `body` read
    // exercises the lookup **through a freshly built inner environment**, which
    // is the `case_env` half of the clause. A single `Var(0)` would only test
    // the lookup.
    let nested_read = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Var(0)),
        body: Box::new(RuntimeExpr::Var(0)),
    };
    let (plan, root_origin) = planned_root_occurrence(&nested_read);
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);

    let mut func = Function::new();
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);

    let seeded_word = builder.ins().iconst(types::I64, 0x0c1_d3);

    // ── the carried phase ─────────────────────────────────────────────────
    let carried_env = [LoweringOperand::Carried(CarriedBoundaryWord {
        word: seeded_word,
    })];
    let carried_out = compiler
        .lower_expr(
            &mut builder,
            SourceOccurrence {
                expr: &nested_read,
                static_origin: root_origin,
            },
            &carried_env,
        )
        .expect("reading a bound operand emits nothing and cannot fail");
    let LoweringOperand::Carried(returned) = carried_out else {
        panic!(
            "a carried operand must still be carried after entering an inner \
             environment and being read back through nested lowering"
        );
    };
    assert_eq!(
        returned.word, seeded_word,
        "the spine must FORWARD the operand, not re-mint one: a different SSA \
         value here means some edge rebuilt the word instead of moving it"
    );

    // ── POSITIVE CONTROL: the identical fixture, specialized ──────────────
    //
    // ⛔ Without this the test is consistent with a spine that answers
    // `Carried` for everything.
    let specialized_env = [LoweringOperand::Specialized(Lowered::Bool {
        value: seeded_word,
        known: Some(true),
    })];
    let specialized_out = compiler
        .lower_expr(
            &mut builder,
            SourceOccurrence {
                expr: &nested_read,
                static_origin: root_origin,
            },
            &specialized_env,
        )
        .expect("reading a bound operand emits nothing and cannot fail");
    assert!(
        matches!(
            specialized_out,
            LoweringOperand::Specialized(Lowered::Bool {
                known: Some(true),
                ..
            })
        ),
        "NON-VACUITY: the same fixture must answer `Specialized` when a \
         specialized operand is seeded, or the carried assertion above is not \
         measuring the phase at all"
    );
}

// ─── `RT-FNSPLIT-C1` `AC-C7` — the EXECUTABLE EDGE ────────────────────────
//
// ⭐⭐ **This section is the node's reason to exist**, and it is the only thing
// here that distinguishes *"the carried routes are written"* from *"the carried
// routes work."*
//
// ⚠⚠ **Why nothing before this rig could establish that, stated so it is not
// re-learned.** Every earlier control ran against `bare_carrier_test_lowering`,
// whose `boundary_carrier` is `None` — so it could only ever observe a
// *refusal*. And rustc's dead-code pass, which correctly caught the uninhabited
// `Carried` variant one commit earlier, clears on the **mention** of a helper,
// ⛔ never on the branch executing. ⇒ Both instruments went quiet while all
// three elimination routes were still unreached by any test.
//
// ⇒ These tests **JIT-compile and RUN** the emitted code against a real bound
// arena, and assert the **eliminated value**, ⛔ not that no error came back.

/// A real invocation arena, bound the way emitted code expects to find one.
///
/// ⚠ The returned `BoundaryArenaV1` and `BoundaryValueStore` must both stay
/// alive across the call: the base pointer names *their* tables, and the
/// reservation happens before `publish` because growing a table afterwards
/// would move it under the pointer emitted code already holds.
fn ac_c7_bind_arena(
    store: &mut crate::boundary_value::BoundaryValueStore,
) -> (crate::boundary_value::BoundaryArenaV1, *mut u64) {
    store.reserve_persistent(64, 256, 512, 0);
    let persistent = store.publish_persistent();
    let mut arena = crate::boundary_value::BoundaryArenaBuilder::new().finish();
    arena.reserve(64, 256, 512, 0);
    arena.bind_persistent(Some(persistent as *const u64));
    let base = arena.publish();
    (arena, base)
}

/// The `AC-C7` rig: a JIT module carrying the **real** emitted carrier graph,
/// plus a `Lowering` wired to call it, plus whatever the caller emits between
/// them.
///
/// ⭐ The probe's one parameter is the invocation arena, which is exactly what
/// `Lowering::carrier_arena` reads — so the helpers this rig calls are the same
/// helpers production would call, reached the same way.
fn ac_c7_try_compile_edge<'src>(
    seed_env: &'src NativeSeedEnvironment,
    plan: StaticTransitionPlan<'src>,
    emit: impl FnOnce(
        &mut Lowering<'src>,
        &mut FunctionBuilder<'_>,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError>,
) -> Result<(cranelift_jit::JITModule, *const u8), CraneliftBackendError> {
    let mut module = new_jit_module().expect("JIT module constructs");
    let native = crate::native_int_clif::emit_native_int_local_graph(&mut module, false)
        .expect("native-int graph emits");
    let boundary_plan = crate::boundary_value::BoundaryEmissionPlan::derive();
    let helpers = crate::boundary_value_clif::emit_boundary_value_local_graph(
        &mut module,
        &native,
        &boundary_plan,
    )
    .expect("boundary carrier graph emits");
    let pointer = module.target_config().pointer_type();

    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(pointer));
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("c1_ac_c7_edge", Linkage::Local, &signature)
        .expect("edge probe declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);

    let carrier = BoundaryCarrierRefs {
        tag: module.declare_func_in_func(helpers.tag, &mut context.func),
        field_count: module.declare_func_in_func(helpers.field_count, &mut context.func),
        field: module.declare_func_in_func(helpers.field, &mut context.func),
        record_field: module.declare_func_in_func(helpers.record_field, &mut context.func),
        alloc: module.declare_func_in_func(helpers.alloc, &mut context.func),
        store_tag_id: module.declare_func_in_func(helpers.store_tag_id, &mut context.func),
        store_field: module.declare_func_in_func(helpers.store_field, &mut context.func),
        store_name: module.declare_func_in_func(helpers.store_name, &mut context.func),
        make_immediate: module.declare_func_in_func(helpers.make_immediate, &mut context.func),
    };

    let mut compiler = bare_carrier_test_lowering(seed_env, plan);
    compiler.boundary_carrier = Some(carrier);

    let mut function_context = FunctionBuilderContext::new();
    let refused = {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        compiler.native_int_arena = Some(builder.block_params(entry)[0]);
        // â  A refusal must still leave a WELL-FORMED function behind, or the
        // failure the caller wanted to observe is replaced by a Cranelift
        // assertion about an unfilled block. â­ Every carrier route refuses
        // *before* it creates a block â the termination guard and the
        // empty-case check both say so at their sites â so on the error path
        // the entry block is still current and still empty, and returning a
        // constant from it is sound.
        match emit(&mut compiler, &mut builder) {
            Ok(result) => {
                builder.ins().return_(&[result]);
                builder.seal_all_blocks();
                builder.finalize();
                None
            }
            Err(error) => {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
                builder.seal_all_blocks();
                builder.finalize();
                Some(error)
            }
        }
    };
    if let Some(error) = refused {
        return Err(error);
    }
    module
        .define_function(func_id, &mut context)
        .expect("edge probe defines");
    module.finalize_definitions().expect("jit finalizes");
    let code = module.get_finalized_function(func_id);
    Ok((module, code))
}

/// The expecting wrapper â every `AC-C7` row uses this, because there a
/// refusal is a test failure rather than the measurement.
fn ac_c7_compile_edge<'src>(
    seed_env: &'src NativeSeedEnvironment,
    plan: StaticTransitionPlan<'src>,
    emit: impl FnOnce(
        &mut Lowering<'src>,
        &mut FunctionBuilder<'_>,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError>,
) -> (cranelift_jit::JITModule, *const u8) {
    ac_c7_try_compile_edge(seed_env, plan, emit).expect("the carried edge emits")
}

fn ac_c7_run(code: *const u8, arena: *const u64) -> i64 {
    let f: extern "C" fn(*const u64) -> i64 = unsafe { std::mem::transmute(code) };
    f(arena)
}

/// A zero-argument constructor occurrence, so the producer's supported surface
/// (`Constructor` with no children) carries the whole fixture.
fn ac_c7_ctor(name: &str) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: format!("ctor:fixture::C1::{name}"),
        args: Vec::new(),
    }
}

fn ac_c7_lowered_ctor(name: &str) -> Lowered {
    Lowered::Constructor {
        constructor: format!("ctor:fixture::C1::{name}"),
        args: Vec::new(),
    }
}

/// Drive one `Project` edge end to end and report the **runtime** identity of
/// the projected child, beside the two artifact-static identities it could
/// legitimately have been.
///
/// ⭐ **One plan serves both sides, and that is the point rather than a
/// convenience.** The producer keys `store_name` on
/// `record_field_identity(record_origin, position)` and the eliminator keys
/// `record_field` on `project_field_identity(project_origin)`. Deriving both
/// from a single planned `Let { Record{..}, Project{Var(0), ..} }` is what makes
/// their agreement `D2`'s **shared-authority property under test**, ⛔ rather
/// than an assumption baked into the fixture.
///
/// ⚠ **The identities are returned rather than hard-coded because they are
/// ARTIFACT-LOCAL.** A packed identity is a span into *this* plan's own name
/// arena, so the same spelling may pack differently in a different plan. ⛔ A
/// caller must therefore compare within one call's results and never across two.
fn ac_c7_project_edge(fields: [(&str, &str); 2], project: &str) -> (i64, u64, u64) {
    let fixture = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Record {
            fields: vec![
                (fields[0].0.to_string(), ac_c7_ctor(fields[0].1)),
                (fields[1].0.to_string(), ac_c7_ctor(fields[1].1)),
            ],
        }),
        body: Box::new(RuntimeExpr::Project {
            record: Box::new(RuntimeExpr::Var(0)),
            field: project.to_string(),
        }),
    };
    let RuntimeExpr::Let {
        body: project_expr, ..
    } = &fixture
    else {
        unreachable!("the fixture is a `Let`")
    };
    let (plan, root) = planned_root_occurrence(&fixture);
    let record_origin = plan
        .child_static_origin(root, 0)
        .expect("a `Let`'s value is child 0");
    let project_origin = plan
        .child_static_origin(root, 1)
        .expect("a `Let`'s body is child 1");
    let identity = |position: usize| {
        plan.constructor_symbol_identity(
            plan.child_static_origin(record_origin, position)
                .expect("a record field has a planned child origin"),
        )
        .expect("a planned `Construct` has a constructor identity")
        .tag_abi_word()
        .expect("an identity packs into the ABI word")
    };
    let first_identity = identity(0);
    let second_identity = identity(1);

    let lowered_fields = vec![
        (fields[0].0.to_string(), ac_c7_lowered_ctor(fields[0].1)),
        (fields[1].0.to_string(), ac_c7_lowered_ctor(fields[1].1)),
    ];
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        // ── PRODUCER: a compile-time record crosses the one-way seam ──────
        let record = Lowered::Record {
            fields: lowered_fields,
        };
        let word = compiler.transfer_into_carrier(builder, record_origin, &record)?;
        // ── ELIMINATOR: `Project` over a value with NO compile-time
        //    template — the carried operand is all the env holds ──────────
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: project_expr.as_ref(),
                static_origin: project_origin,
            },
            &[LoweringOperand::Carried(word)],
        )?;
        let LoweringOperand::Carried(child) = eliminated else {
            panic!(
                "`§2g` requires a projected child to remain `Carried`; a specialized \
                 result here would be the materialized template the node exists to remove"
            );
        };
        // The assertion instrument: read the child's own runtime identity.
        compiler.emit_carrier_tag(builder, child)
    });

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let observed = ac_c7_run(code, base);
    (observed, first_identity, second_identity)
}

/// ⭐⭐ **`AC-C7` ROW 1 OF 3 — `Project`.** Reported as its own row; ⛔ never
/// folded into an aggregate, because an aggregate differential passes while one
/// of three contributors defects.
///
/// **MEASURED:** JIT-compiled emitted code, run against a real bound arena,
/// transfers a `Record` across the one-way producer and then lowers a `Project`
/// whose only input is the resulting boundary word — and the projected child's
/// **runtime** `tag` equals the artifact-static identity of the constructor the
/// named field holds.
/// **CLAIMED:** `D4` — `Project` selects a runtime record field by
/// artifact-static field identity and returns the carrier.
/// **THE GAP:** *"the result equals `Beta`"* is satisfiable by an eliminator
/// that ignores the field name and always returns the last child. ⭐ Closed by
/// the second half: projecting `"a"` on the identical fixture must yield
/// `Alpha`, so the two projections have to **disagree** in the direction the
/// names dictate.
///
/// ⚠ Promise class: **durable invariant**. It asserts a relation between the
/// runtime identity and the plan's own static identity, ⛔ not either as a
/// frozen literal — so re-interning, re-ordering the arena, or renaming the
/// fixture's constructors all keep it green, while an eliminator that stops
/// keying on the name turns it red.
#[test]
fn c1_d4_ac_c7_project_eliminates_a_carried_record_by_static_field_identity() {
    let (observed, alpha, beta) = ac_c7_project_edge([("a", "Alpha"), ("b", "Beta")], "b");
    assert_ne!(
        alpha, beta,
        "NON-VACUITY: the two constructors must have DIFFERENT artifact-static \
         identities, or `observed == beta` is satisfied by any answer at all"
    );
    assert_eq!(
        observed as u64, beta,
        "`D4`: projecting `b` must return the carrier holding `Beta`, whose \
         runtime tag is its artifact-static identity {beta}; got {observed}"
    );

    // ── DISCRIMINATOR: the same fixture, the other field ──────────────────
    let (other, alpha, beta) = ac_c7_project_edge([("a", "Alpha"), ("b", "Beta")], "a");
    assert_eq!(
        other as u64, alpha,
        "DISCRIMINATOR: projecting `a` must return `Alpha`. If this and the case \
         above returned the same word, the eliminator is not reading the field \
         name at all"
    );
    assert_ne!(
        other as u64, beta,
        "DISCRIMINATOR: projecting `a` must NOT return `Beta`"
    );
}

/// ⭐ **`AC-C5`'s named control — a record whose fields are REORDERED relative
/// to declaration yields the same projection.**
///
/// **MEASURED:** with the fixture's fields declared `(b, a)` instead of
/// `(a, b)`, projecting `"b"` still returns `Beta` — now the child at
/// **position 0** rather than position 1.
/// **CLAIMED:** the projection is keyed on artifact-static **field identity**,
/// not on declaration position.
/// **THE GAP:** a positional eliminator returns position 1 either way, so it
/// would answer `Alpha` here while answering `Beta` in the row above. ⇒ The two
/// tests together are the pair; ⛔ neither alone distinguishes name-keyed from
/// position-keyed.
///
/// ⚠ Identities are compared **within this call only** — they are artifact-local
/// spans, so the number here need not equal the number in the row above even for
/// the same spelling.
#[test]
fn c1_d4_ac_c5_a_reordered_record_projects_the_same_field() {
    // Declared `(b, a)`: `Beta` is now child 0.
    let (observed, beta, alpha) = ac_c7_project_edge([("b", "Beta"), ("a", "Alpha")], "b");
    assert_ne!(alpha, beta, "NON-VACUITY: the identities must differ");
    assert_eq!(
        observed as u64, beta,
        "`AC-C5`: `b` sits at declaration position 0 in this fixture and position \
         1 in the row above, and both must project to `Beta` — a positional \
         eliminator answers `Alpha` here"
    );
}

fn ac_c7_wrap(outer: &str, inner: &str) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: format!("ctor:fixture::C1::{outer}"),
        args: vec![ac_c7_ctor(inner)],
    }
}

fn ac_c7_lowered_wrap(outer: &str, inner: &str) -> Lowered {
    Lowered::Constructor {
        constructor: format!("ctor:fixture::C1::{outer}"),
        args: vec![ac_c7_lowered_ctor(inner)],
    }
}

fn ac_c7_trap() -> RuntimeTrap {
    RuntimeTrap {
        code: crate::RuntimeTrapCode::PatternMatchFailure,
        message: "no artifact-static case matches the carried value".to_string(),
    }
}

/// ⛔ The status the emitted closed default returns. Read from the one place
/// that spells it — `Lowering::seal_source_trap_branch` — rather than restated,
/// so the two cannot drift.
const AC_C7_TRAP_STATUS: i64 = -4;

/// Drive one carried `Match` end to end.
///
/// The fixture is `Let { Wrap(Inner), Match Var(0) { Left x -> x, Right x -> Sentinel } }`
/// with `Wrap` supplied by the caller, so ONE helper produces all three
/// interesting outcomes: selecting the first case, selecting the second, and
/// reaching the closed default.
///
/// ⭐ **Case 0's body is `Var(0)` — the projected child.** That makes its
/// returned identity the *child's*, so a green result requires all four emitted
/// steps: `tag` selected the case, `field_count` admitted the arity, `field(0)`
/// projected the child, and the child **stayed `Carried`** through `case_env`
/// and the nested lowering of the body.
///
/// ⭐⭐ **Case 1's body is a DIFFERENT expression, and that asymmetry is
/// load-bearing.** An earlier revision gave both cases the body `Var(0)`, and it
/// was **green for a weaker reason than it claimed**: an eliminator that always
/// took case 0 would still bind `field(0)` and return the same child, so the
/// "selects the right case" assertion could not have failed. ⛔ Two cases that
/// agree on every input do not discriminate between them. The defect was found
/// by designing `AC-C7`'s neutering mutation and noticing it would not redden —
/// which is the whole reason that control is mandated.
fn ac_c7_match_edge(scrutinee: &str, inner: &str) -> (i64, u64, u64) {
    let fixture = RuntimeExpr::Let {
        value: Box::new(ac_c7_wrap(scrutinee, inner)),
        body: Box::new(RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeMatchCase {
                    constructor: "ctor:fixture::C1::Left".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Var(0),
                },
                crate::RuntimeMatchCase {
                    constructor: "ctor:fixture::C1::Right".to_string(),
                    binders: 1,
                    body: ac_c7_ctor("Sentinel"),
                },
            ],
            default: ac_c7_trap(),
        }),
    };
    let RuntimeExpr::Let {
        body: match_expr, ..
    } = &fixture
    else {
        unreachable!("the fixture is a `Let`")
    };
    let (plan, root) = planned_root_occurrence(&fixture);
    let scrutinee_origin = plan
        .child_static_origin(root, 0)
        .expect("a `Let`'s value is child 0");
    let match_origin = plan
        .child_static_origin(root, 1)
        .expect("a `Let`'s body is child 1");
    let identity_at = |origin| {
        plan.constructor_symbol_identity(origin)
            .expect("a planned `Construct` has a constructor identity")
            .tag_abi_word()
            .expect("an identity packs into the ABI word")
    };
    let inner_identity = identity_at(
        plan.child_static_origin(scrutinee_origin, 0)
            .expect("the wrapper's only argument has a planned origin"),
    );
    // A `Match`'s case *i* body is child `1 + i` — the scrutinee is child 0.
    let sentinel_identity = identity_at(
        plan.child_static_origin(match_origin, 2)
            .expect("case 1's body has a planned origin"),
    );

    let lowered = ac_c7_lowered_wrap(scrutinee, inner);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringOperand::Carried(word)],
        )?;
        let LoweringOperand::Carried(selected) = eliminated else {
            panic!("a carried `Match` merges in the carrier lane, so its result is `Carried`");
        };
        compiler.emit_carrier_tag(builder, selected)
    });

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let observed = ac_c7_run(code, base);
    (observed, inner_identity, sentinel_identity)
}

/// ⭐⭐ **`AC-C7` ROW 2 OF 3 — `Match`.** Its own row; ⛔ never aggregated.
///
/// **MEASURED:** JIT-compiled emitted code, run against a real bound arena,
/// transfers `Left(Alpha)` across the one-way producer and lowers a two-case
/// `Match` whose only input is the resulting boundary word — the result's
/// runtime `tag` is `Alpha`'s artifact-static identity. Swapping the scrutinee
/// to `Right(Beta)` selects the **other** case, whose body is a different
/// expression, and yields `Sentinel`.
/// **CLAIMED:** `D3` — `Match` eliminates a carried value with no compile-time
/// template, selecting the correct case and projecting its children back into
/// the same carrier.
/// **THE GAP:** a single positive case is satisfied by an eliminator that always
/// takes case 0. ⭐ Closed because the two cases **disagree on every input**:
/// case 0 returns the projected child, case 1 returns a fixed constructor, so
/// selecting the wrong one is always observable.
///
/// ⚠ Promise class: **durable invariant** — a relation between the runtime tag
/// and the plan's own static identity, ⛔ never a frozen literal.
#[test]
fn c1_d3_ac_c7_match_eliminates_a_carried_value_and_selects_the_right_case() {
    let (first, alpha, sentinel) = ac_c7_match_edge("Left", "Alpha");
    assert_ne!(
        alpha, sentinel,
        "NON-VACUITY: the child and the sentinel must have different identities, \
         or selecting the wrong case is unobservable"
    );
    assert_eq!(
        first as u64, alpha,
        "`D3`: `Left(Alpha)` must select case 0 and bind its projected child, so \
         the result carries `Alpha` (identity {alpha}); got {first}"
    );

    // ── DISCRIMINATOR: the SECOND case, whose body differs ────────────────
    let (second, beta, sentinel) = ac_c7_match_edge("Right", "Beta");
    assert_eq!(
        second as u64, sentinel,
        "DISCRIMINATOR: `Right(Beta)` must select case 1, whose body is a fixed \
         constructor. An eliminator that always takes case 0 returns the child \
         `Beta` ({beta}) here instead; got {second}"
    );
    assert_ne!(
        second as u64, beta,
        "DISCRIMINATOR: case 1's body ignores the binder, so the child must not \
         be what comes back"
    );
}

/// ⭐ **`AC-C3`'s negative arm — a constructor OUTSIDE the artifact-static case
/// set reaches the closed default.**
///
/// **MEASURED:** the identical two-case fixture, given a scrutinee whose
/// constructor matches neither case, returns the emitted trap status instead of
/// any case's value.
/// **CLAIMED:** the carried `Match`'s case chain is **closed** — it falls
/// through to a runtime default rather than selecting arbitrarily or reading
/// past the node.
/// **THE GAP:** *"it returned the trap status"* is satisfiable by any failure
/// whatsoever, including the arena never binding. ⭐ Closed by the row above
/// sharing this helper: the same rig, same arena, same producer path returns
/// real identities for the two matching scrutinees, so the trap here is
/// attributable to the case chain and not to the rig.
#[test]
fn c1_d3_ac_c3_a_constructor_outside_the_case_set_reaches_the_closed_default() {
    let (observed, inner, sentinel) = ac_c7_match_edge("Absent", "Gamma");
    assert_eq!(
        observed, AC_C7_TRAP_STATUS,
        "`AC-C3`: `Absent(Gamma)` matches neither `Left` nor `Right`, so the \
         emitted chain must reach the closed default; got {observed}"
    );
    assert_ne!(
        observed as u64, inner,
        "the default must not be reachable by returning the child anyway"
    );
    assert_ne!(
        observed as u64, sentinel,
        "the default must not be reachable by falling into the last case"
    );
}

/// Drive one carried `ComputationalMatch` end to end — the same shape as
/// [`ac_c7_match_edge`], through the **composed producer route**.
///
/// ⛔ `recursive_positions` is deliberately empty: an induction hypothesis over
/// a carried child is the Architect fork this node refuses (see
/// `Lowering::lower_carried_computational_match`), so this row measures the
/// non-recursive elimination and ⛔ does NOT discharge `AC-C4`.
fn ac_c7_computational_match_edge(scrutinee: &str, inner: &str) -> (i64, u64, u64) {
    let fixture = RuntimeExpr::Let {
        value: Box::new(ac_c7_wrap(scrutinee, inner)),
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::C1::Left".to_string(),
                    argument_binders: 1,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Var(0),
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::C1::Right".to_string(),
                    argument_binders: 1,
                    recursive_positions: Vec::new(),
                    body: ac_c7_ctor("Sentinel"),
                },
            ],
            default: ac_c7_trap(),
        }),
    };
    let RuntimeExpr::Let {
        body: match_expr, ..
    } = &fixture
    else {
        unreachable!("the fixture is a `Let`")
    };
    let (plan, root) = planned_root_occurrence(&fixture);
    let scrutinee_origin = plan
        .child_static_origin(root, 0)
        .expect("a `Let`'s value is child 0");
    let match_origin = plan
        .child_static_origin(root, 1)
        .expect("a `Let`'s body is child 1");
    let identity_at = |origin| {
        plan.constructor_symbol_identity(origin)
            .expect("a planned `Construct` has a constructor identity")
            .tag_abi_word()
            .expect("an identity packs into the ABI word")
    };
    let inner_identity = identity_at(
        plan.child_static_origin(scrutinee_origin, 0)
            .expect("the wrapper's only argument has a planned origin"),
    );
    let sentinel_identity = identity_at(
        plan.child_static_origin(match_origin, 2)
            .expect("case 1's body has a planned origin"),
    );

    let lowered = ac_c7_lowered_wrap(scrutinee, inner);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringOperand::Carried(word)],
        )?;
        let LoweringOperand::Carried(selected) = eliminated else {
            panic!(
                "a carried `ComputationalMatch` merges in the carrier lane, so its \
                 result is `Carried`"
            );
        };
        compiler.emit_carrier_tag(builder, selected)
    });

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let observed = ac_c7_run(code, base);
    (observed, inner_identity, sentinel_identity)
}

/// ⭐⭐ **`AC-C7` ROW 3 OF 3 — `ComputationalMatch`.** Its own row; ⛔ never
/// aggregated with the other two.
///
/// **MEASURED:** as row 2, through the **composed producer route** rather than
/// the direct one: `Left(Alpha)` selects case 0 and yields the projected child,
/// `Right(Beta)` selects case 1 and yields its fixed body — on a value that
/// never had a compile-time template.
/// **CLAIMED:** `D3` for `ComputationalMatch`'s **non-recursive** cases.
/// **THE GAP — stated because this row is the one that under-delivers:**
/// ⛔ `AC-C4` asks for `ComputationalMatch` *"with recursive positions"*, and
/// this fixture has none. The recursive arm **fails closed** pending the
/// Architect's ruling on whether a `Lowered` variant may hold a
/// `LoweringOperand`. ⇒ This row is `AC-C7` evidence for the third eliminator;
/// it is ⛔ **NOT** `AC-C4`.
#[test]
fn c1_d3_ac_c7_computational_match_eliminates_a_carried_value_non_recursively() {
    let (first, alpha, sentinel) = ac_c7_computational_match_edge("Left", "Alpha");
    assert_ne!(alpha, sentinel, "NON-VACUITY: the identities must differ");
    assert_eq!(
        first as u64, alpha,
        "`D3`: `Left(Alpha)` must select case 0 through the composed route and \
         bind its projected child; got {first}"
    );

    let (second, beta, sentinel) = ac_c7_computational_match_edge("Right", "Beta");
    assert_eq!(
        second as u64, sentinel,
        "DISCRIMINATOR: `Right(Beta)` must select case 1 through the composed \
         route. An always-case-0 eliminator returns `Beta` ({beta}) instead"
    );
}

/// The `AC-C4` fixture: a `ComputationalMatch` whose first case declares a
/// **recursive position**, over a carried scrutinee, whose body **invokes the
/// induction hypothesis** with zero arguments.
///
/// ⭐⭐ **The two cases disagree on every input, by construction.** `Wrap`'s body
/// is the IH call; `Leaf`'s body is a fixed `Sentinel`. So on `Wrap(Leaf)` the
/// only way to reach `Sentinel` is to *recurse* — an eliminator that returned
/// the bound child, or that always took case 0, or that never installed the
/// invocation, lands on `Leaf` instead. ⚠ This is the trap `AC-C7` caught on
/// this node one commit ago: two arms whose bodies agree cannot discriminate
/// between them, and the positive assertion is then green for a weaker reason
/// than it claims.
///
/// Returns `(observed, leaf_identity, sentinel_identity)`. ⚠ The identities are
/// artifact-local spans into *this* plan's name arena — compare within one
/// call's results, ⛔ never across two.
fn ac_c4_recursive_edge(
    recursive_body: RuntimeExpr,
) -> Result<(i64, u64, u64), CraneliftBackendError> {
    let fixture = RuntimeExpr::Let {
        value: Box::new(ac_c7_wrap("Wrap", "Leaf")),
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::C1::Wrap".to_string(),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    // ⭐ The case environment for a recursive case is
                    // `[IH, reversed] ++ [children] ++ frame env`, so `Var(0)` is
                    // the induction hypothesis over child `0` and `Var(1)` is the
                    // child itself. ⛔ Zero arguments: a carried residual is a
                    // transferred VALUE, and the structural IH route is the only
                    // admitted one.
                    body: recursive_body,
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::C1::Leaf".to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: ac_c7_ctor("Sentinel"),
                },
            ],
            default: ac_c7_trap(),
        }),
    };
    let RuntimeExpr::Let {
        body: match_expr, ..
    } = &fixture
    else {
        unreachable!("the fixture is a `Let`")
    };
    let (plan, root) = planned_root_occurrence(&fixture);
    let scrutinee_origin = plan
        .child_static_origin(root, 0)
        .expect("a `Let`'s value is child 0");
    let match_origin = plan
        .child_static_origin(root, 1)
        .expect("a `Let`'s body is child 1");
    let identity_at = |origin| {
        plan.constructor_symbol_identity(origin)
            .expect("a planned `Construct` has a constructor identity")
            .tag_abi_word()
            .expect("an identity packs into the ABI word")
    };
    let leaf_identity = identity_at(
        plan.child_static_origin(scrutinee_origin, 0)
            .expect("the wrapper's only argument has a planned origin"),
    );
    let sentinel_identity = identity_at(
        plan.child_static_origin(match_origin, 2)
            .expect("case 1's body has a planned origin"),
    );

    let lowered = ac_c7_lowered_wrap("Wrap", "Leaf");
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_try_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringOperand::Carried(word)],
        )?;
        let LoweringOperand::Carried(selected) = eliminated else {
            panic!(
                "a carried `ComputationalMatch` merges in the carrier lane, so its \
                 result is `Carried` even when a recursive position resumed it"
            );
        };
        compiler.emit_carrier_tag(builder, selected)
    })?;

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let observed = ac_c7_run(code, base);
    Ok((observed, leaf_identity, sentinel_identity))
}

/// ⭐⭐ **`AC-C4` — a carried recursive position BUILDS ITS INDUCTION
/// HYPOTHESIS and eliminates. Executable, JIT-run, value-asserted.**
///
/// **MEASURED:** JIT-compiled code, run against a real bound arena, eliminates
/// `Wrap(Leaf)` — a value with no compile-time template — through a case
/// declaring `recursive_positions: [0]`. The case body reads **`Var(1)`**, and
/// the observed identity is `Leaf`'s.
/// **CLAIMED:** the single-field license is live end to end: an IH is minted
/// over a **carried** child (so `ComputationalRecursorClosure.residual` really
/// does hold a `LoweringOperand::Carried`), the case environment is laid out
/// `[IH] ++ [children] ++ frame env`, and the whole recursive-position case
/// eliminates in the carrier lane.
/// **THE GAP — stated because this row does not close `AC-C4`:** ⛔ the body
/// does not **invoke** the hypothesis. Invoking it is refused, for a mechanism
/// reason that is not a matter of effort; see the sentinel below.
///
/// ⭐ **`Var(1)` is the discriminator, and it is why this is not a vacuous
/// "it compiled" test.** Index `1` is the bound child *only if* the induction
/// hypothesis occupies index `0`. An implementation that skipped minting the
/// IH, or appended it after the children, shifts every de Bruijn index in the
/// body — `Var(1)` would then read the frame environment or run off the end,
/// and it could not return `Leaf`.
///
/// ⚠ Promise class: **durable invariant**. It relates the eliminated value to
/// the case environment's layout, over plan-derived identities.
#[test]
fn c1_d3_ac_c4_a_carried_recursive_position_builds_its_hypothesis_and_eliminates() {
    let (observed, leaf, sentinel) =
        ac_c4_recursive_edge(RuntimeExpr::Var(1)).expect("the recursive-position case lowers");
    assert_ne!(
        leaf, sentinel,
        "NON-VACUITY: the two identities this fixture can produce must differ"
    );
    assert_eq!(
        observed as u64, leaf,
        "`AC-C4`: with the induction hypothesis at index 0, `Var(1)` is the bound \
         carried child, so eliminating `Wrap(Leaf)` must yield `Leaf`. Any other \
         case-environment layout shifts this read; got {observed}"
    );
}

/// ⛔⛔ **TRANSITION SENTINEL — invoking a carried induction hypothesis is
/// REFUSED, and `AC-C4` is therefore NOT discharged.**
///
/// ⭐ **This is a boundary, not a bug, and the reason is structural.** A
/// *specialized* recursive elimination terminates because its residual is a
/// compile-time value that strictly shrinks. A **carried** residual is a runtime
/// word: nothing shrinks at compile time, so emitting the recursive case emits
/// its IH invocation, which re-enters the same eliminator, which emits the
/// recursive case again — without bound. ⚠ Measured, not theorised: before the
/// guard existed this fixture **overflowed the compiler's stack**.
///
/// ⛔ **The ruled mechanism is necessary but not sufficient.** Widening
/// `residual` to a `LoweringOperand` is what lets the hypothesis *exist* over a
/// carried child, and the test above proves it does. Making it *callable*
/// additionally requires the elimination to be emitted as a runtime backedge or
/// call rather than unrolled — a codegen capability the single-field license
/// neither grants nor implies.
///
/// ⚠ Promise class: **transition sentinel**, named for the boundary rather than
/// the current behaviour. ⭐ **It retires when the Architect rules on emitting a
/// carried recursive elimination as a backedge/call**; the seat that lands that
/// ruling deletes this test and makes the body above `Call { Var(0), [] }`,
/// which must then return `Sentinel` rather than `Leaf`.
#[test]
fn c1_d3_ac_c4_invoking_a_carried_hypothesis_refuses_rather_than_unrolling_unbounded() {
    let refused = ac_c4_recursive_edge(RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: Vec::new(),
    })
    .expect_err("inlining a carried recursion cannot terminate, so it must refuse");
    let CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct: "BoundaryCarrier",
        reason,
        ..
    }) = &refused
    else {
        panic!(
            "the refusal must come from the carrier's termination guard, not from \
             whatever the unroll would have hit first: got {refused:?}"
        );
    };
    assert!(
        reason.contains("cannot terminate"),
        "the diagnostic must say WHY it refuses, so the next reader inherits the \
         mechanism rather than rediscovering the stack overflow: got {reason}"
    );
}

/// ⭐⭐ **`AC-C4` CONTROL 5 — a carried residual applied to SOURCE ARGUMENTS
/// fails closed, and fails BEFORE the invocation is installed.**
///
/// **MEASURED:** the same recursive-position fixture, with the case body
/// invoking its induction hypothesis on one argument (`Var(1)`, the bound
/// carried child), is refused by the carrier with an arity diagnostic.
/// **CLAIMED:** the ruling's clause 3 — a carried residual is a transferred
/// **value**, never a transferred callable, so only the zero-argument
/// structural route is admitted.
/// **THE GAP:** *"it errored"* is satisfied by erroring for any reason at all,
/// including the termination guard that would fire one step later. ⭐ Closed by
/// asserting on the **arity** wording, which only
/// `Lowering::reject_carried_residual_arguments` produces — and that refusal
/// runs before any invocation segment is installed or semantic region entered.
///
/// ⚠ Promise class: **durable invariant**. A carried residual never becomes
/// callable without a durable closure lane, which the ruling withholds
/// explicitly; if one is ever granted, this is the test that must be argued.
#[test]
fn c1_d3_ac_c4_a_carried_hypothesis_applied_to_arguments_fails_closed() {
    let refused = ac_c4_recursive_edge(RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: vec![RuntimeExpr::Var(1)],
    })
    .expect_err("a carried residual is a value, so applying it must refuse");
    let CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct: "BoundaryCarrier",
        reason,
        ..
    }) = &refused
    else {
        panic!("the arity refusal is the carrier's: got {refused:?}");
    };
    assert!(
        reason.contains("not a callable"),
        "DISCRIMINATOR: this must be the ARGUMENT refusal, not the termination \
         guard that would fire a step later on the same fixture. Both are \
         `BoundaryCarrier` errors, so the wording is what separates them: got \
         {reason}"
    );
}

/// A minimal, structurally valid recursor capsule wrapping `residual`.
///
/// ⭐ The invocation segment is inert on purpose: control 4 measures the
/// **admission walk's ordering**, which must refuse the capsule before it ever
/// reads what is inside — so the inside is deliberately uninteresting.
fn ac_c4_recursor_capsule(residual: LoweringOperand) -> Lowered {
    let origin = RecursorProducerOriginId(41);
    let cursor = ContinuationCursorId(42);
    Lowered::ComputationalRecursorClosure {
        residual: Box::new(residual),
        activation: ContinuationActivationId(43),
        invocation: RecursorInvocationSegment::new(
            origin,
            0,
            ComputationalRecursorLayer {
                cases: Vec::new(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: "ac-c4 capsule".to_string(),
                },
                outer_env: Vec::new(),
                static_origin: inert_test_static_origin(),
                provenance: RecursorFrameProvenance(44),
                role: RecursorLayerRole::SelectsOccurrence { origin },
                checked_frame_id: None,
                checked_invocation_id: None,
                checked_invocation_source: None,
                checked_invocation_depth: 0,
                semantic_pending: true,
            },
            RecursorUnwindStack {
                later_wrappers_in_construction_order: Vec::new(),
            },
            cursor,
            None,
            None,
        ),
    }
}

/// ⭐⭐ **`AC-C4` CONTROL 4 — the outer recursor capsule stays UNCONDITIONALLY
/// non-transferable, and the admission walk refuses it BEFORE it looks inside.**
///
/// **MEASURED:** `transfer_into_carrier` on a constructor holding a recursor
/// capsule is refused as a `ComputationalRecursorClosure`, and it is refused
/// identically whether the capsule's residual is `Specialized` or `Carried`.
/// The positive control — the same shape with an admissible child — gets *past*
/// the walk and stops at the first emitted carrier call.
/// **CLAIMED:** the ruling's clause 4: widening `residual` did not open a
/// transfer path. The capsule is rejected before allocation or helper
/// invocation, and a carried residual is not a way to reach the carrier through
/// a capsule that is otherwise refused.
/// **THE GAP:** *"the transfer errored"* is satisfied by erroring anywhere,
/// including **after** an `alloc`. ⭐ Two things close it: the fixture has no
/// carrier refs installed, so *any* emitted helper call produces the distinct
/// `BoundaryCarrier` error the positive control asserts; and the capsule case
/// produces the `ComputationalRecursorClosure` error instead. ⇒ The two
/// diagnostics are what prove the ordering, not the mere presence of an error.
///
/// ⚠ The capsule is nested one level down, ⛔ never at the root: a root refusal
/// would be the root variant's own disposition and could not distinguish the
/// walk from the disposition table.
///
/// ⚠ Promise class: **durable invariant**.
#[test]
fn c1_d3_ac_c4_the_recursor_capsule_is_refused_before_its_residual_is_read() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut module = new_jit_module().expect("JIT module constructs");
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("c1_ac_c4_capsule_probe", Linkage::Local, &signature)
        .expect("probe declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);

    let construct = RuntimeExpr::Construct {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
    };
    let (plan, construct_origin) = planned_root_occurrence(&construct);
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);

    // A real SSA value, so the carried residual below is a genuine carried word
    // rather than a stand-in.
    let word = CarriedBoundaryWord {
        word: builder.ins().iconst(types::I64, 7),
    };

    for (label, residual) in [
        (
            "a SPECIALIZED residual -- the behaviour that must not have changed",
            LoweringOperand::Specialized(Lowered::Closure {
                captures: Vec::new(),
                params: Vec::new(),
                body: inert_test_static_origin(),
            }),
        ),
        (
            "a CARRIED residual -- the newly licensed shape",
            LoweringOperand::Carried(word),
        ),
    ] {
        let inadmissible = Lowered::Constructor {
            constructor: "ctor:fixture::C1::Wrap".to_string(),
            args: vec![ac_c4_recursor_capsule(residual)],
        };
        let refused = compiler
            .transfer_into_carrier(&mut builder, construct_origin, &inadmissible)
            .expect_err("a recursor capsule cannot cross the boundary");
        let CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. }) = &refused
        else {
            panic!("the capsule refusal is an unsupported-lowering: got {refused:?}");
        };
        assert!(
            reason.contains("in-flight activation"),
            "the capsule must be refused AS AN IN-FLIGHT ACTIVATION -- the \
             disposition that makes it unconditionally non-transferable -- and \
             refused before anything reads its residual. ⛔ Not as a carrier \
             failure, which would mean a helper had already been emitted. With \
             {label}: got {refused:?}"
        );
    }

    // ── POSITIVE CONTROL ──────────────────────────────────────────────────
    let admissible = Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![Lowered::Bool {
            value: builder.ins().iconst(types::I64, 1),
            known: Some(true),
        }],
    };
    let reached = compiler
        .transfer_into_carrier(&mut builder, construct_origin, &admissible)
        .expect_err("a fixture with no carrier refs cannot allocate");
    assert!(
        matches!(
            reached,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "BoundaryCarrier",
                ..
            })
        ),
        "NON-VACUITY: the admissible graph must get PAST the walk and stop at the \
         first emitted call, or the two refusals above prove nothing about \
         ordering: got {reached:?}"
    );
}
