//! Bounded-Nat, host-reply, IO, borrowed-ingress and native-int lowering
//! tests (RT-SPLIT §10.2 assigns these subjects to `effects`).

use super::*;

/// Exercise the checked-reply mint without involving any resource operation.
/// The fixture deliberately enters through `mint_validated_progress_nat`, so
/// tests cannot manufacture the compact carrier through a second constructor.
#[cfg(test)]
fn run_checked_bounded_nat_fixture(
    count: u64,
    request_start: u64,
    request_length: u64,
    reply_start: u64,
    observation: BoundedNatFixtureObservation,
    mutation: BoundedNatLoweringMutation,
) -> Result<i64, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px8n_checked_bounded_nat", Linkage::Local, &signature)
        .map_err(|error| backend_module(error.to_string()))?;
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = Lowering {
        seed_env: &seed_env,
        declarations: BTreeMap::new(),
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
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: mutation,
    };
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let count = builder.ins().iconst(types::I64, count as i64);
        let request_start = builder.ins().iconst(types::I64, request_start as i64);
        let request_length = builder.ins().iconst(types::I64, request_length as i64);
        let reply_start = builder.ins().iconst(types::I64, reply_start as i64);
        let one = builder.ins().iconst(types::I64, 1);
        let success =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, one, 1);
        let (count, _predecessor, remaining) = Lowering::mint_validated_progress_nat(
            &mut builder,
            success,
            count,
            request_start,
            request_length,
            Some(reply_start),
        );
        let nat = match observation {
            BoundedNatFixtureObservation::OrdinaryCount
            | BoundedNatFixtureObservation::ComputationalCount => count,
            BoundedNatFixtureObservation::OrdinaryRemaining => remaining,
        };
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N exact structural Nat default".to_string(),
        };
        let lowered = match observation {
            BoundedNatFixtureObservation::OrdinaryCount
            | BoundedNatFixtureObservation::OrdinaryRemaining => {
                let cases = vec![
                    crate::RuntimeMatchCase {
                        constructor: compiler.process_symbols.nat_zero.clone(),
                        binders: 0,
                        body: RuntimeExpr::Value(RuntimeValue::Int((10).into())),
                    },
                    crate::RuntimeMatchCase {
                        constructor: compiler.process_symbols.nat_suc.clone(),
                        binders: 1,
                        body: RuntimeExpr::Match {
                            scrutinee: Box::new(RuntimeExpr::Var(0)),
                            cases: vec![
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_zero.clone(),
                                    binders: 0,
                                    body: RuntimeExpr::Value(RuntimeValue::Int((21).into())),
                                },
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_suc.clone(),
                                    binders: 1,
                                    body: RuntimeExpr::Value(RuntimeValue::Int((22).into())),
                                },
                            ],
                            default: default.clone(),
                        },
                    },
                ];
                compiler.lower_bounded_nat_match(&mut builder, nat, false, &cases, &default, &[])?
            }
            BoundedNatFixtureObservation::ComputationalCount => {
                let cases = vec![
                    crate::RuntimeComputationalMatchCase {
                        constructor: compiler.process_symbols.nat_zero.clone(),
                        argument_binders: 0,
                        recursive_positions: Vec::new(),
                        body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
                    },
                    crate::RuntimeComputationalMatchCase {
                        constructor: compiler.process_symbols.nat_suc.clone(),
                        argument_binders: 1,
                        recursive_positions: vec![0],
                        body: RuntimeExpr::Match {
                            scrutinee: Box::new(RuntimeExpr::Var(1)),
                            cases: vec![
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_zero.clone(),
                                    binders: 0,
                                    body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
                                },
                                crate::RuntimeMatchCase {
                                    constructor: compiler.process_symbols.nat_suc.clone(),
                                    binders: 1,
                                    body: RuntimeExpr::Match {
                                        scrutinee: Box::new(RuntimeExpr::Var(1)),
                                        cases: vec![
                                            crate::RuntimeMatchCase {
                                                constructor: compiler
                                                    .process_symbols
                                                    .bool_false
                                                    .clone(),
                                                binders: 0,
                                                body: RuntimeExpr::Value(RuntimeValue::Bool(true)),
                                            },
                                            crate::RuntimeMatchCase {
                                                constructor: compiler
                                                    .process_symbols
                                                    .bool_true
                                                    .clone(),
                                                binders: 0,
                                                body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
                                            },
                                        ],
                                        default: default.clone(),
                                    },
                                },
                            ],
                            default: default.clone(),
                        },
                    },
                ];
                let frames = [EliminatorFrame::Computational(
                    ComputationalEliminatorFrame {
                        cases: &cases,
                        default: &default,
                        env: &[],
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                        provenance: compiler.mint_recursor_frame_provenance(),
                        checked_frame_id: None,
                        checked_invocation_id: None,
                        checked_invocation_source: None,
                        checked_invocation_depth: 0,
                    },
                )];
                compiler.lower_bounded_nat_computational(&mut builder, nat, false, &frames)?
            }
        };
        let value = match lowered {
            Lowered::Int { value, .. } => value,
            other => compiler.emit_result(&mut builder, other)?.0,
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
        .map(|(_, value)| value.expect("PX8-N fixture returns one scalar"))
}

#[test]
fn px8n_bounded_nat_observes_exact_zero_successor_and_recursive_order() {
    assert_eq!(
        run_checked_bounded_nat_fixture(
            3,
            7,
            3,
            7,
            BoundedNatFixtureObservation::OrdinaryRemaining,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        10,
        "a zero remainder selects the structural Zero arm",
    );
    assert_eq!(
        run_checked_bounded_nat_fixture(
            3,
            7,
            5,
            7,
            BoundedNatFixtureObservation::OrdinaryCount,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        22,
        "Suc exposes predecessor 2 as a second structural successor",
    );
    assert_eq!(
        run_checked_bounded_nat_fixture(
            3,
            7,
            5,
            7,
            BoundedNatFixtureObservation::ComputationalCount,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        0,
        "the recursive Suc case consumes the ordered predecessor and retained IH",
    );
}

#[test]
fn px8n_bounded_nat_rejects_zero_over_bound_misaligned_and_wrapping_progress() {
    for (count, start, length, reply_start) in [
        (0, 7, 5, 7),
        (6, 7, 5, 7),
        (3, 7, 5, 8),
        (3, u64::MAX - 1, 5, u64::MAX - 1),
    ] {
        assert_eq!(
            run_checked_bounded_nat_fixture(
                count,
                start,
                length,
                reply_start,
                BoundedNatFixtureObservation::OrdinaryCount,
                BoundedNatLoweringMutation::Exact,
            )
            .unwrap(),
            -1,
            "invalid checked-host progress returns before carrier mint observation",
        );
    }
}

#[test]
fn px8n_decrement_and_raw_scalar_mutations_fail_the_structural_oracle() {
    let run = |mutation| {
        run_checked_bounded_nat_fixture(
            3,
            7,
            5,
            7,
            BoundedNatFixtureObservation::ComputationalCount,
            mutation,
        )
        .unwrap()
    };

    let exact = run(BoundedNatLoweringMutation::Exact);
    assert_eq!(exact, 0);
    assert_eq!(
        run(BoundedNatLoweringMutation::BrokenDecrement),
        -2,
        "the live production loop's test-only fuel guard detects nontermination",
    );
    assert_eq!(
            run(BoundedNatLoweringMutation::RawScalarPredecessor),
            1,
            "the live producer exposes the exact wrong result when its Suc binder receives the raw scalar",
        );
}

fn run_borrowed_fixture(expr: &RuntimeExpr, root: &BorrowedFixtureValue) -> i64 {
    let compiled = compile_expr_into_module(
        new_jit_module().expect("JIT module"),
        "px4_borrowed_fixture",
        Linkage::Local,
        expr,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        None,
        Some(test_only_distinguished_root_join_plan()),
        None,
    )
    .expect("borrowed fixture lowers");
    let mut native_int_arena = crate::NativeIntArenaV1::default();
    let invocation = NativeInvocationFixture {
        process_input: root,
        host_context: std::ptr::null_mut(),
        capability: 1_u64 << 32,
        native_int_arena: &mut native_int_arena,
    };
    compiled
        .run(Some((&invocation as *const NativeInvocationFixture).cast()))
        .expect("borrowed fixture runs")
        .1
        .expect("borrowed fixture returns scalar")
}

#[test]
fn borrowed_ingress_malformed_metadata_fails_closed() {
    let malformed = BorrowedFixtureValue {
        kind: 99,
        tag: 1,
        data: std::ptr::null(),
        len: 3,
    };
    let expr = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![RuntimeMatchCase {
            constructor: crate::PROCESS_INPUT_CONSTRUCTOR.to_string(),
            binders: 3,
            body: RuntimeExpr::Value(RuntimeValue::Int((0).into())),
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "malformed process root".to_string(),
        },
    };
    assert_eq!(run_borrowed_fixture(&expr, &malformed), -1);
    let null_fields = BorrowedFixtureValue {
        kind: 2,
        tag: 1,
        data: std::ptr::null(),
        len: 3,
    };
    assert_eq!(run_borrowed_fixture(&expr, &null_fields), -1);
    let wrong_arity = BorrowedFixtureValue {
        kind: 2,
        tag: 1,
        data: (&malformed as *const BorrowedFixtureValue).cast(),
        len: 2,
    };
    assert_eq!(run_borrowed_fixture(&expr, &wrong_arity), -1);
    assert!(crate::object_linker_packaging::process_starter_c_stub()
        .contains("ken native trap: malformed borrowed process input"));
}

#[test]
fn borrowed_ingress_bytes_at_preserves_safe_none_bounds() {
    let cwd = [0xff_u8];
    let fields = [
        BorrowedFixtureValue {
            kind: 2,
            tag: 2,
            data: std::ptr::null(),
            len: 0,
        },
        BorrowedFixtureValue {
            kind: 2,
            tag: 2,
            data: std::ptr::null(),
            len: 0,
        },
        BorrowedFixtureValue {
            kind: 1,
            tag: 0,
            data: cwd.as_ptr().cast(),
            len: cwd.len(),
        },
    ];
    let root = BorrowedFixtureValue {
        kind: 2,
        tag: 1,
        data: fields.as_ptr().cast(),
        len: 3,
    };
    let none = "ctor:fixture::Option::None";
    let some = "ctor:fixture::Option::Some";
    let expr = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![RuntimeMatchCase {
            constructor: crate::PROCESS_INPUT_CONSTRUCTOR.to_string(),
            binders: 3,
            body: RuntimeExpr::Construct {
                constructor: crate::EXIT_FAILURE_CONSTRUCTOR.to_string(),
                args: vec![RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::PrimitiveCall {
                        primitive: RuntimePrimitive {
                            symbol: "bytes_at".to_string(),
                            partiality: RuntimePartiality::SafeOption {
                                none: none.to_string(),
                                some: some.to_string(),
                                obligation: Some("obl:px4.bounds".to_string()),
                            },
                        },
                        args: vec![
                            RuntimeExpr::Var(2),
                            RuntimeExpr::Value(RuntimeValue::Int((99).into())),
                        ],
                    }),
                    cases: vec![
                        RuntimeMatchCase {
                            constructor: none.to_string(),
                            binders: 0,
                            body: RuntimeExpr::Value(RuntimeValue::Int((7).into())),
                        },
                        RuntimeMatchCase {
                            constructor: some.to_string(),
                            binders: 1,
                            body: RuntimeExpr::Value(RuntimeValue::Int((9).into())),
                        },
                    ],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "invalid bytes_at option".to_string(),
                    },
                }],
            },
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "invalid process input".to_string(),
        },
    };
    assert_eq!(run_borrowed_fixture(&expr, &root), 7);
}

#[test]
fn dynamic_host_result_producer_missing_case_routes_to_default() {
    assert!(
        dynamic_host_result_producer_case(&[], "ctor:prelude::Result::Ok")
            .expect("missing case is a fail-closed default route")
            .is_none()
    );
    emit_process_entrypoint_object_with_cranelift(
        &host_result_computational_fixture(1, false, true),
        "ken_px7m_missing_case_default",
    )
    .expect("the absent dynamic arm lowers through the producer default trap");
}
#[test]
fn px8n_fs_write_at_arm_constructs_short_wrote_and_exact_no_progress() {
    let (short, fixture) = run_px8n_write_arm_fixture(PX8N_SHORT_WROTE);
    assert_eq!(fixture.malformed_request, 0);
    assert_eq!(fixture.call_index, 3);
    assert_eq!(
        short, 3,
        "Wrote 1 of 4 exposes predecessor Zero and remaining structural Nat 3",
    );

    let (zero, fixture) = run_px8n_write_arm_fixture(PX8N_ZERO_WRITE);
    assert_eq!(fixture.malformed_request, 0);
    assert_eq!(fixture.call_index, 3);
    assert_eq!(
        zero, 70,
        "zero write reaches exact ResourceError.NoProgress"
    );
}
#[test]
fn live_effect_emitter_inventory_and_generated_layout_mutations_are_closed() {
    assert_eq!(
        CRANELIFT_HOST_EFFECT_CONSUMERS_V1,
        ken_host::NATIVE_TESTED_TARGETS_V1
    );
    for operation in CRANELIFT_HOST_EFFECT_CONSUMERS_V1 {
        let layout = ken_host::host_effect_wire_layout_v1(operation).unwrap();
        assert_eq!(
            ken_host::verify_host_effect_wire_layout_v1(operation, &layout),
            Ok(())
        );
        let mut mutations = Vec::new();
        let mut changed = layout.clone();
        changed.request_size ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.request_align_shift ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.request_offsets[0] ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_size ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_tag_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_error_tag ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_tag ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_schema_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_kind_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_identity_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_io_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_required_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_resource_error_held_offset ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.resource_error_closed ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.resource_error_malformed ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.resource_error_right_not_held ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.resource_error_release_failed ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.resource_kind_fs_handle ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.resource_error_reply_schema ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_unit_tag ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_bool_tag ^= 1;
        mutations.push(changed);
        let mut changed = layout.clone();
        changed.reply_bytes_tag ^= 1;
        mutations.push(changed);
        for mutation in mutations {
            assert!(ken_host::verify_host_effect_wire_layout_v1(operation, &mutation).is_err());
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum BoundedNatFixtureObservation {
    OrdinaryCount,
    OrdinaryRemaining,
    ComputationalCount,
}

#[test]
fn direct_host_result_closure_match_keeps_established_dynamic_lane() {
    emit_process_entrypoint_object_with_cranelift(
        &host_result_closure_match(console_write_effect()),
        "ken_px7o_direct_host_result_closure_match",
    )
    .expect("direct HostResult remains owned by ordinary dynamic matching");
}
#[test]
fn call_returned_host_result_keeps_established_dynamic_lane() {
    let effect_call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["ignored".to_string()],
            body: Box::new(console_write_effect()),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((0).into()))],
    };

    emit_process_entrypoint_object_with_cranelift(
        &host_result_closure_match(effect_call),
        "ken_px7o_call_returned_host_result_closure_match",
    )
    .expect("call-returned HostResult remains owned by ordinary dynamic matching");
}
#[test]
fn match_selected_call_returned_host_result_keeps_established_dynamic_lane() {
    let effect_call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["ignored".to_string()],
            body: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Bool::True".to_string(),
                    args: Vec::new(),
                }),
                cases: ["ctor:prelude::Bool::True", "ctor:prelude::Bool::False"]
                    .into_iter()
                    .map(|constructor| RuntimeMatchCase {
                        constructor: constructor.to_string(),
                        binders: 0,
                        body: console_write_effect(),
                    })
                    .collect(),
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "static Bool default".to_string(),
                },
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((0).into()))],
    };

    emit_process_entrypoint_object_with_cranelift(
        &host_result_closure_match(effect_call),
        "ken_px7o_match_selected_call_returned_host_result",
    )
    .expect("match-selected HostResult remains owned by ordinary dynamic matching");
}
#[test]
fn recursive_computational_host_result_keeps_established_dynamic_lane() {
    emit_process_entrypoint_object_with_cranelift(
        &host_result_closure_match(recursive_computational_result(console_write_effect())),
        "ken_px7o_recursive_computational_host_result",
    )
    .expect("recursive computational HostResult remains on ordinary dynamic matching");
}
#[test]
fn px8n_fs_write_at_arm_rejects_over_bound_reply_before_observation() {
    let (result, fixture) = run_px8n_write_arm_fixture(PX8N_OVER_BOUND_WRITE);
    assert_eq!(fixture.malformed_request, 0);
    assert_eq!(fixture.call_index, 3);
    assert_eq!(
        result, -1,
        "Wrote 5 for an effective request of 4 rejects before a Nat is observable",
    );
}
#[test]
fn px8n_fs_read_at_arm_distinguishes_eof_and_short_read_some() {
    let (eof, fixture) = run_px8n_read_arm_fixture(PX8N_READ_EOF);
    assert_eq!(fixture.malformed_request, 0);
    assert_eq!(fixture.call_index, 3);
    assert_eq!(eof, 10, "zero read constructs exact ReadEof");

    let (short, fixture) = run_px8n_read_arm_fixture(PX8N_SHORT_READ);
    assert_eq!(fixture.malformed_request, 0);
    assert_eq!(fixture.call_index, 3);
    assert_eq!(
        short, 12,
        "ReadSome 1 of 4 carries the same structural Nat 1 in BufferSpan",
    );
}
#[test]
fn px8n_fs_read_at_arm_rejects_over_bound_span_before_observation() {
    let (result, fixture) = run_px8n_read_arm_fixture(PX8N_OVER_BOUND_READ);
    assert_eq!(fixture.malformed_request, 0);
    assert_eq!(fixture.call_index, 3);
    assert_eq!(
        result, -1,
        "ReadSome 5 for an effective request of 4 rejects before a Nat is observable",
    );
}
#[test]
fn px8i_host_narrowing_rejects_negative_and_over_u64_before_dispatch() {
    let (negative, negative_fixture) =
        run_px8n_arm_fixture(PX8N_SHORT_WROTE, px8i_negative_narrow_fixture);
    assert_eq!(negative, 71);
    assert_eq!(negative_fixture.call_index, 0);

    let (oversize, oversize_fixture) =
        run_px8n_arm_fixture(PX8N_SHORT_WROTE, px8i_oversize_narrow_fixture);
    assert_eq!(oversize, 72);
    assert_eq!(oversize_fixture.call_index, 0);
}
#[test]
fn px8i_positioned_start_and_metadata_promote_u64_above_i64_max() {
    let (read, read_fixture) =
        run_px8n_arm_fixture(PX8I_BIG_READ_START, px8i_big_read_start_fixture);
    assert_eq!(read_fixture.malformed_request, 0);
    assert_eq!(read_fixture.call_index, 3);
    assert_eq!(
        read, 13,
        "ReadAt keeps the narrowed start through validation"
    );

    let (write, write_fixture) =
        run_px8n_arm_fixture(PX8I_WRAPPING_WRITE_START, px8i_wrapping_write_start_fixture);
    assert_eq!(write_fixture.malformed_request, 0);
    assert_eq!(write_fixture.call_index, 3);
    assert_eq!(
        write, -1,
        "WriteAt validates progress against the narrowed start and rejects wrap"
    );

    let (metadata, metadata_fixture) =
        run_px8n_arm_fixture(PX8I_METADATA_BIG, px8i_metadata_big_fixture);
    assert_eq!(metadata_fixture.malformed_request, 0);
    assert_eq!(metadata_fixture.call_index, 2);
    assert_eq!(
        metadata, 14,
        "metadata detail is promoted to canonical Big rather than a negative Small"
    );
}
#[test]
fn unsupported_effect_is_distinct_from_backend_failure() {
    let example = RuntimeExample {
        name: "unsupported-effect".to_string(),
        checked_core_shape: "diagnostic label only".to_string(),
        ir: RuntimeExpr::Effect {
            family: "Console".to_string(),
            operation: ken_host::HostOpV1::ConsoleRead,
            capability: None,
            args: vec![],
        },
        observation: RuntimeObservation::Trapped(RuntimeTrap {
            code: RuntimeTrapCode::UnsupportedErasure,
            message: "unsupported".to_string(),
        }),
    };

    let err = run_example_with_seed_observation(&example, &NativeSeedEnvironment::empty())
        .expect_err("effect must reject");

    assert!(matches!(
        err,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "Effect",
            ..
        })
    ));
}
fn px8i_negative_narrow_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8i_invalid_allocate(
        symbols,
        RuntimeExpr::Value(RuntimeValue::Int((-1).into())),
        71,
    )
}
fn px8i_oversize_narrow_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8i_invalid_allocate(symbols, big(crate::Sign::NonNegative, &[0, 1]), 72)
}
fn px8i_wrapping_write_start_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8n_write_arm_fixture_with_start(symbols, big(crate::Sign::NonNegative, &[u64::MAX - 1]))
}
fn px8i_big_read_start_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8n_read_arm_fixture_with_start(
        symbols,
        big(crate::Sign::NonNegative, &[PX8I_BIG_U64]),
        true,
    )
}
fn px8i_metadata_big_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    let trap = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-I metadata result default".to_string(),
    };
    let metadata = RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsHandleMetadata,
        capability: None,
        args: vec![RuntimeExpr::Var(0)],
    };
    let observe = RuntimeExpr::Match {
        scrutinee: Box::new(metadata),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((98).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: px8n_failure(
                    symbols,
                    RuntimeExpr::If {
                        scrutinee: Box::new(total_primitive(
                            "eq_int",
                            vec![
                                RuntimeExpr::Var(0),
                                big(crate::Sign::NonNegative, &[PX8I_BIG_U64]),
                            ],
                        )),
                        then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((14).into()))),
                        else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((99).into()))),
                    },
                ),
            },
        ],
        default: trap(),
    };
    RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
        }),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((97).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: observe,
            },
        ],
        default: trap(),
    }
}
fn run_px8n_read_arm_fixture(scenario: u64) -> (i64, Px8nHostReplyFixture) {
    run_px8n_arm_fixture(scenario, px8n_read_arm_fixture)
}
fn px8i_invalid_allocate(
    symbols: &crate::NativeProcessSymbols,
    capacity: RuntimeExpr,
    code: i64,
) -> RuntimeExpr {
    RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![capacity],
        }),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![crate::RuntimeMatchCase {
                        constructor: symbols.resource_invalid_bounds.clone(),
                        binders: 0,
                        body: px8n_failure(
                            symbols,
                            RuntimeExpr::Value(RuntimeValue::Int(code.into())),
                        ),
                    }],
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "PX8-I expected InvalidBounds".to_string(),
                    },
                },
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int(99.into()))),
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-I expected Result".to_string(),
        },
    }
}
fn px8n_read_arm_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8n_read_arm_fixture_with_start(
        symbols,
        RuntimeExpr::Value(RuntimeValue::Int((7).into())),
        false,
    )
}
fn px8n_read_arm_fixture_with_start(
    symbols: &crate::NativeProcessSymbols,
    start: RuntimeExpr,
    observe_big_start: bool,
) -> RuntimeExpr {
    let trap = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-N checked read result default".to_string(),
    };
    let allocate = || RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::BufferAllocate,
        capability: None,
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
    };
    let read = RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsReadAt,
        capability: None,
        args: vec![
            RuntimeExpr::Var(1),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Var(0),
            start,
            RuntimeExpr::Value(RuntimeValue::Int((4).into())),
        ],
    };
    let exact = if observe_big_start {
        RuntimeExpr::If {
            scrutinee: Box::new(total_primitive(
                "eq_int",
                vec![
                    RuntimeExpr::Var(1),
                    big(crate::Sign::NonNegative, &[PX8I_BIG_U64]),
                ],
            )),
            then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((13).into()))),
            else_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Int((99).into()))),
        }
    } else {
        RuntimeExpr::Value(RuntimeValue::Int((12).into()))
    };
    let read_some = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![crate::RuntimeMatchCase {
            constructor: symbols.private_buffer_span.clone(),
            binders: 2,
            body: px8n_exact_nat(symbols, RuntimeExpr::Var(1), 1, exact),
        }],
        default: trap(),
    };
    let read_some = px8n_failure(symbols, read_some);
    let progress = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.read_some.clone(),
                binders: 2,
                body: read_some,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.read_eof.clone(),
                binders: 0,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((10).into()))),
            },
        ],
        default: trap(),
    };
    let read_result = RuntimeExpr::Match {
        scrutinee: Box::new(read),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((82).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: progress,
            },
        ],
        default: trap(),
    };
    let second = RuntimeExpr::Match {
        scrutinee: Box::new(allocate()),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((81).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: read_result,
            },
        ],
        default: trap(),
    };
    RuntimeExpr::Match {
        scrutinee: Box::new(allocate()),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((80).into()))),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: second,
            },
        ],
        default: trap(),
    }
}
