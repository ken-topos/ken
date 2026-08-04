//! Bounded-Nat, host-reply, IO, borrowed-ingress and native-int lowering
//! tests (RT-SPLIT §10.2 assigns these subjects to `effects`).

use super::*;

// RT-SPLIT slice 7, rule 8: dependency declarations carried in for the moved
// px8n fixture closure. These are used ONLY by that closure, so they travel
// with it (AC-9's "what travels with a moving item"). Ruled test module, so a
// `use` is permitted here (AC-8 class 2).
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::default_libcall_names;

use crate::cranelift_backend::artifact::native_isa_for_lowering_tests as native_isa;

/// Exercise the checked-reply mint without involving any resource operation.
/// The fixture deliberately enters through `mint_validated_progress_nat`, so
/// tests cannot manufacture the compact carrier through a second constructor.
#[cfg(test)]
fn run_checked_bounded_nat_fixture(
    count: u64,
    request_start: u64,
    request_length: u64,
    effective_request: u64,
    // `Some` mirrors the `ReadSome` call site (a reply-carried span start
    // distinct from the request); `None` mirrors `Wrote` (no reply span —
    // `mint_validated_progress_nat` falls back to `request_start`).
    reply_start: Option<u64>,
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
        body_emission_authority: BodyEmissionAuthority::FunctionizedUnits,
        continuation_claims: None,
        checked_call_ledger: None,
        defining_unit: None,
        defining_emission_owner: None,
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        // ⛔ `None` — a bare `Lowering` fixture emits into no module, so it has
        // no callable carrier refs. The `Carried` routes fail closed on this
        // rather than silently taking the `Specialized` path.
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: mutation,
        function_local: FunctionLocalRefs {
            defining_abi_operands: Vec::new(),
            context_calls: BTreeMap::new(),
            worker_templates: BTreeMap::new(),
            generated_context_captures: None,
            seed_material: crate::cranelift_backend::lowering::seed_material::SeedMaterialRefs::none_for_tests(),
            host_dispatch: None,
            host_dispatch_context: None,
            services_pointer: None,
            native_int_arena: None,
            boundary_arena: None,
            native_int_binop: None,
            native_int_compare: None,
            native_int_intern: None,
            native_int_narrow: None,
            native_int_export: None,
            native_int_export_parts: None,
            native_int_resolve: None,
            native_int_tags: BTreeMap::new(),
            unit_calls: BTreeMap::new(),
            worker_calls: BTreeMap::new(),
            continuation_calls: BTreeMap::new(),
            continuation_emissions: BTreeMap::new(),
            declaration_calls: BTreeMap::new(),
            trap_exit: None,
            terminal_result_origins: BTreeSet::new(),
            consumed_join_origins: BTreeSet::new(),
            dispositioned_join_origins: BTreeSet::new(),
            join_disposition_finalized: false,
            final_reachable_join_origins: BTreeSet::new(),
            materialized_join_blocks: BTreeMap::new(),
            emission_reachable_match_cases: BTreeMap::new(),
            boundary_carrier: None,
        },
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
        let effective_request = builder.ins().iconst(types::I64, effective_request as i64);
        let reply_start = reply_start.map(|start| builder.ins().iconst(types::I64, start as i64));
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
            effective_request,
            reply_start,
        );
        let nat = match observation {
            BoundedNatFixtureObservation::OrdinaryCount
            | BoundedNatFixtureObservation::ComputationalCount => count,
            BoundedNatFixtureObservation::OrdinaryRemaining
            | BoundedNatFixtureObservation::RawRemainingScalar => remaining,
        };
        let default = RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N exact structural Nat default".to_string(),
        };
        // BUDGET-EFF: `RawRemainingScalar` returns `nat`'s raw scalar
        // directly, bypassing the eliminator match below — the structural
        // zero/one/many buckets it produces can't distinguish a correct
        // capped-short `remaining` from one wrongly derived from the raw
        // (pre-clamp) length, since both are >= 2 and collapse to the same
        // bucket ("22"). Still enters solely through
        // `mint_validated_progress_nat`; no second constructor, just a
        // different tail on the one minted value.
        let value = if let BoundedNatFixtureObservation::RawRemainingScalar = observation {
            nat.value
        } else {
            let lowered = match observation {
                BoundedNatFixtureObservation::RawRemainingScalar => {
                    unreachable!("handled above, before eliminator lowering")
                }
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
                    // Real origins: plan the match these cases belong to.
                    //
                    // ⚠ Leaked deliberately, and only in this fixture family: since
                    // B2A-S the plan **borrows** the term it planned, and these
                    // cases are built out of `compiler.process_symbols`, so the term
                    // cannot be declared before the `Lowering` that holds the plan.
                    // `'static` is the honest way out in a test; the alternative is
                    // reshaping the fixture whose shape is the thing under test.
                    let source_match: &'static RuntimeExpr =
                        Box::leak(Box::new(RuntimeExpr::Match {
                            scrutinee: Box::new(RuntimeExpr::Var(0)),
                            cases: cases.clone(),
                            default: default.clone(),
                        }));
                    let (plan, match_origin) = planned_root_occurrence(source_match);
                    compiler.static_transition_plan = plan;
                    compiler.enter_source_occurrence_plan(match_origin)?;
                    compiler.lower_bounded_nat_match(
                        &mut builder,
                        nat,
                        false,
                        &cases,
                        &default,
                        match_origin,
                        &[],
                    )?
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
                                                    body: RuntimeExpr::Value(RuntimeValue::Bool(
                                                        true,
                                                    )),
                                                },
                                                crate::RuntimeMatchCase {
                                                    constructor: compiler
                                                        .process_symbols
                                                        .bool_true
                                                        .clone(),
                                                    binders: 0,
                                                    body: RuntimeExpr::Value(RuntimeValue::Bool(
                                                        false,
                                                    )),
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
                    // Leaked for the same reason as the ordinary-match fixture above.
                    let source_match: &'static RuntimeExpr =
                        Box::leak(Box::new(RuntimeExpr::ComputationalMatch {
                            scrutinee: Box::new(RuntimeExpr::Var(0)),
                            cases: cases.clone(),
                            default: default.clone(),
                        }));
                    let (plan, match_origin) = planned_root_occurrence(source_match);
                    compiler.static_transition_plan = plan;
                    compiler.enter_source_occurrence_plan(match_origin)?;
                    let frames = [EliminatorFrame::Computational(
                        ComputationalEliminatorFrame {
                            cases: &cases,
                            default: &default,
                            env: &[],
                            static_origin: match_origin,
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
            match lowered.specialized_at("this fixture's result")? {
                Lowered::Int { value, .. } => value,
                other => compiler.emit_result(&mut builder, other)?.0,
            }
        };
        builder.ins().return_(&[value]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&context.func, module.isa())?;
    module
        .define_function(func_id, &mut context)
        .map_err(|error| backend_module(error.to_string()))?;
    let trap_catalog = compiler.static_transition_plan.trap_catalog();
    let carrier_identity_catalog = compiler
        .static_transition_plan
        .carrier_identity_catalog()?;
    let compiled = CompiledModule::from_parts(
        module,
        func_id,
        Some(ResultDecoder::ProcessStatus),
        compiler.result_table,
        None,
        trap_catalog,
        carrier_identity_catalog,
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
            3,
            Some(7),
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
            5,
            Some(7),
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
            5,
            Some(7),
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
                length,
                Some(reply_start),
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
            5,
            Some(7),
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

// BUDGET-EFF native half (Architect ruling `dec_1m6xdwjp2ttyn`,
// `docs/program/issues/BUDGET-EFF.md`). `remaining` must derive from the
// post-clamp `effective_request`, never the raw pre-clamp request length —
// `mint_validated_progress_nat` is the exact native reification seat the
// WP's AC-3 rewrite requires a test at. `RawRemainingScalar` reads the
// minted value's magnitude directly (see its doc comment above), because the
// structural zero/one/many buckets the other observations use cannot tell a
// correct capped-short `remaining` (2) from a raw-derived one (6) — both
// land in the same "many" bucket.
//
// capped-full ALONE would be green under the wrong shortcut
// `effective := count` (remaining 0 either way) — capped-short is the
// discriminating shape and is not optional.

#[test]
fn budget_eff_native_read_some_capped_full_and_short_reify_effective_not_raw_remaining() {
    assert_eq!(
        run_checked_bounded_nat_fixture(
            4,
            0,
            8,
            4,
            Some(0),
            BoundedNatFixtureObservation::RawRemainingScalar,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        0,
        "ReadSome capped-full: raw 8, effective 4, count 4 -> remaining 0",
    );
    assert_eq!(
        run_checked_bounded_nat_fixture(
            2,
            0,
            8,
            4,
            Some(0),
            BoundedNatFixtureObservation::RawRemainingScalar,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        2,
        "ReadSome capped-short: raw 8, effective 4, count 2 -> remaining 2 \
         (NOT 6 == raw 8 - count 2, the pre-fix defect this WP closes)",
    );
}

#[test]
fn budget_eff_native_wrote_capped_full_and_short_reify_effective_not_raw_remaining() {
    assert_eq!(
        run_checked_bounded_nat_fixture(
            4,
            0,
            8,
            4,
            None,
            BoundedNatFixtureObservation::RawRemainingScalar,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        0,
        "Wrote capped-full: raw 8, effective 4, count 4 -> remaining 0",
    );
    assert_eq!(
        run_checked_bounded_nat_fixture(
            2,
            0,
            8,
            4,
            None,
            BoundedNatFixtureObservation::RawRemainingScalar,
            BoundedNatLoweringMutation::Exact,
        )
        .unwrap(),
        2,
        "Wrote capped-short: raw 8, effective 4, count 2 -> remaining 2 \
         (NOT 6 == raw 8 - count 2, the pre-fix defect this WP closes)",
    );
}

#[test]
fn budget_eff_native_fails_closed_on_effective_zero_below_count_and_above_raw() {
    // Boundary constraint 3: `0 < count <= effective_request <= raw_length`.
    // Each row violates exactly one conjunct; `mint_validated_progress_nat`
    // must reject all three rather than mint a carrier.
    for (label, count, request_length, effective_request) in [
        ("effective_request == 0", 2, 8, 0),
        ("effective_request(3) < count(4)", 4, 8, 3),
        ("effective_request(9) > raw request_length(8)", 2, 8, 9),
    ] {
        assert_eq!(
            run_checked_bounded_nat_fixture(
                count,
                0,
                request_length,
                effective_request,
                Some(0),
                BoundedNatFixtureObservation::OrdinaryCount,
                BoundedNatLoweringMutation::Exact,
            )
            .unwrap(),
            -1,
            "{label} must fail closed, not mint a carrier",
        );
    }
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
    let mut host_context = ();
    let invocation = RootIngressFixture {
        process_input: root,
        host_context: (&mut host_context as *mut ()).cast(),
        capability: 1_u64 << 32,
    };
    compiled
        .run(Some((&invocation as *const RootIngressFixture).cast()))
        .expect("borrowed fixture runs")
        .1
        .expect("borrowed fixture returns scalar")
}

thread_local! {
    static B2F_EXPECTED_HOST_CONTEXT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    static B2F_HOST_CONTEXT_OBSERVATION: std::cell::Cell<(usize, usize)> =
        const { std::cell::Cell::new((0, 0)) };
}

extern "C" fn b2f_host_context_probe(
    host_context: *const std::ffi::c_void,
    operation: i64,
    _request: *const std::ffi::c_void,
    _request_size: i64,
    reply: *mut std::ffi::c_void,
) -> i64 {
    let expected = B2F_EXPECTED_HOST_CONTEXT.with(std::cell::Cell::get);
    B2F_HOST_CONTEXT_OBSERVATION.with(|cell| {
        let (calls, mismatches) = cell.get();
        cell.set((
            calls + 1,
            mismatches + usize::from(host_context as usize != expected),
        ));
    });
    if host_context as usize != expected
        || operation != ken_host::HostOpV1::ConsoleWrite as i64
    {
        return -1;
    }
    let layout = ken_host::host_effect_wire_layout_v1(ken_host::HostOpV1::ConsoleWrite)
        .expect("ConsoleWrite has a generated wire layout");
    // SAFETY: the generated caller supplies the target-C-sized reply record.
    unsafe {
        std::ptr::write_bytes(reply.cast::<u8>(), 0, layout.reply_size as usize);
        *(reply
            .cast::<u8>()
            .add(layout.reply_tag_offset as usize)
            .cast::<u64>()) = layout.reply_unit_tag;
    }
    0
}

fn b2f_context_fixture() -> RuntimeExpr {
    RuntimeExpr::Let {
        value: Box::new(console_write_effect()),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(RuntimeExpr::Let {
                    value: Box::new(console_write_effect()),
                    body: Box::new(RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    }),
                }),
            }),
            args: Vec::new(),
        }),
    }
}

fn run_b2f_context_fixture(
    mutation: HostContextPropagationMutation,
) -> (i64, (usize, usize)) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            set_host_context_propagation_mutation(HostContextPropagationMutation::Exact);
            B2F_EXPECTED_HOST_CONTEXT.with(|cell| cell.set(0));
            B2F_HOST_CONTEXT_OBSERVATION.with(|cell| cell.set((0, 0)));
        }
    }
    let _reset = Reset;
    set_host_context_propagation_mutation(mutation);
    let isa = native_isa().expect("native ISA");
    let mut jit = JITBuilder::with_isa(isa, default_libcall_names());
    jit.symbol(
        "ken_host_dispatch_v1",
        b2f_host_context_probe as *const u8,
    );
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let expression = b2f_context_fixture();
    let compiled = compile_expr_into_module(
        JITModule::new(jit),
        "b2f_host_context_envelope",
        Linkage::Local,
        &expression,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(&symbols),
        Some(test_only_distinguished_root_join_plan()),
        None,
    )
    .expect("the context-propagation fixture compiles");
    let input = BorrowedFixtureValue {
        kind: 1,
        tag: 0,
        data: std::ptr::null(),
        len: 0,
    };
    let mut context = 0u64;
    let ingress = RootIngressFixture {
        process_input: &input,
        host_context: (&mut context as *mut u64).cast(),
        capability: 1_u64 << 32,
    };
    B2F_EXPECTED_HOST_CONTEXT.with(|cell| cell.set(ingress.host_context as usize));
    let result = compiled
        .run(Some((&ingress as *const RootIngressFixture).cast()))
        .expect("the context-propagation fixture runs")
        .1
        .expect("the process fixture returns a status");
    let observation = B2F_HOST_CONTEXT_OBSERVATION.with(std::cell::Cell::get);
    (result, observation)
}

#[test]
fn root_and_descendant_effects_share_only_the_direct_host_context() {
    let (status, exact) = run_b2f_context_fixture(HostContextPropagationMutation::Exact);
    assert_eq!(status, 0, "the descendant process answer must reach the root");
    assert_eq!(exact, (2, 0), "both host effects must see the direct context");

    for mutation in [
        HostContextPropagationMutation::ServicesPointer,
        HostContextPropagationMutation::NativeIntArena,
        HostContextPropagationMutation::BoundaryArena,
        HostContextPropagationMutation::Null,
        HostContextPropagationMutation::LaunchIngress,
    ] {
        let (_, (calls, mismatches)) = run_b2f_context_fixture(mutation);
        assert!(calls > 0, "{mutation:?} never reached the host assertion");
        assert!(
            mismatches > 0,
            "{mutation:?} did not perturb direct-context propagation"
        );
    }
}

fn b2f_process_pair_fixture() -> RuntimeExpr {
    RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
            params: Vec::new(),
            body: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases: vec![RuntimeMatchCase {
                    constructor: crate::PROCESS_INPUT_CONSTRUCTOR.to_string(),
                    binders: 3,
                    body: RuntimeExpr::Let {
                        value: Box::new(RuntimeExpr::Effect {
                            family: "FS".to_string(),
                            operation: ken_host::HostOpV1::FsReadFile,
                            capability: Some(crate::RuntimeCapabilityUse {
                                identity: "b2f.process.capability".to_string(),
                                value: Box::new(RuntimeExpr::Var(4)),
                            }),
                            args: vec![RuntimeExpr::Value(RuntimeValue::Bytes(
                                b"b2f-process-pair".to_vec(),
                            ))],
                        }),
                        body: Box::new(RuntimeExpr::Construct {
                            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                            args: Vec::new(),
                        }),
                    },
                }],
                default: RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "B2F process-pair capture default".to_string(),
                },
            }),
        }),
        args: Vec::new(),
    }
}

thread_local! {
    static B2F_PROCESS_PAIR_OBSERVATION: std::cell::Cell<(usize, u64)> =
        const { std::cell::Cell::new((0, 0)) };
}

extern "C" fn b2f_process_pair_probe(
    _host_context: *const std::ffi::c_void,
    operation: i64,
    request: *const std::ffi::c_void,
    _request_size: i64,
    reply: *mut std::ffi::c_void,
) -> i64 {
    if operation != ken_host::HostOpV1::FsReadFile as i64
        || request.is_null()
        || reply.is_null()
    {
        return -1;
    }
    let layout = ken_host::host_effect_wire_layout_v1(ken_host::HostOpV1::FsReadFile)
        .expect("FsReadFile has a generated wire layout");
    // SAFETY: the generated request has the layout selected above.
    let capability = unsafe {
        *(request
            .cast::<u8>()
            .add(layout.request_offsets[0] as usize)
            .cast::<u64>())
    };
    B2F_PROCESS_PAIR_OBSERVATION.with(|cell| {
        let (calls, _) = cell.get();
        cell.set((calls + 1, capability));
    });
    // SAFETY: the generated caller supplies the target-C-sized reply record.
    unsafe {
        std::ptr::write_bytes(reply.cast::<u8>(), 0, layout.reply_size as usize);
        *(reply
            .cast::<u8>()
            .add(layout.reply_tag_offset as usize)
            .cast::<u64>()) = layout.reply_bytes_tag;
    }
    0
}

fn compile_b2f_process_pair_fixture() -> Result<CompiledModule<JITModule>, CraneliftBackendError> {
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let isa = native_isa().expect("native ISA");
    let mut jit = JITBuilder::with_isa(isa, default_libcall_names());
    jit.symbol(
        "ken_host_dispatch_v1",
        b2f_process_pair_probe as *const u8,
    );
    compile_expr_into_module(
        JITModule::new(jit),
        "b2f_process_pair_slots",
        Linkage::Local,
        &b2f_process_pair_fixture(),
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(&symbols),
        Some(test_only_distinguished_root_join_plan()),
        None,
    )
}

#[test]
fn the_process_pair_reaches_a_retained_body_only_through_declared_slots() {
    const RECOVERY_ASSERTION: &str =
        "reintroduced launch ingress recovered the capability-using process answer";

    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            set_process_slot_mutation(ProcessSlotMutation::Exact);
            B2F_PROCESS_PAIR_OBSERVATION.with(|cell| cell.set((0, 0)));
        }
    }
    let _reset = Reset;

    for mutation in [
        ProcessSlotMutation::DeleteProcessInput,
        ProcessSlotMutation::DeleteCapability,
    ] {
        set_process_slot_mutation(mutation);
        let error = match compile_b2f_process_pair_fixture() {
            Ok(_) => panic!("deleting either declared root slot must refuse emission"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("declared input"),
            "{mutation:?} failed for the wrong reason: {error}"
        );
    }

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
            data: std::ptr::null(),
            len: 0,
        },
    ];
    let process_input = BorrowedFixtureValue {
        kind: 2,
        tag: 1,
        data: fields.as_ptr().cast(),
        len: fields.len(),
    };
    // The retained direct context is deliberately opaque. It has enough
    // addressable storage to make an accidental fixed-offset load harmless,
    // but it is not launch-ingress-shaped and contains no source-pair value.
    let mut direct_context = DirectHostContextFixture {
        opaque: [std::ptr::null_mut(); 3],
    };
    let ingress = RootIngressFixture {
        process_input: &process_input,
        host_context: (&mut direct_context as *mut DirectHostContextFixture).cast(),
        capability: 1_u64 << 32,
    };

    set_process_slot_mutation(ProcessSlotMutation::Exact);
    let exact = compile_b2f_process_pair_fixture()
        .expect("the exact declared pair compiles")
        .run(Some((&ingress as *const RootIngressFixture).cast()))
        .expect("the exact declared pair runs")
        .1
        .expect("the exact declared pair returns a status");
    assert_eq!(exact, 0, "the descendant process answer must reach the root");
    let exact_observation = B2F_PROCESS_PAIR_OBSERVATION.with(std::cell::Cell::get);
    assert_eq!(exact_observation, (1, ingress.capability));

    set_process_slot_mutation(ProcessSlotMutation::AttemptFixedContextOffsets);
    let fixed_context_error = match compile_b2f_process_pair_fixture() {
        Ok(_) => panic!("the old fixed-context offset evasion was accepted"),
        Err(error) => error,
    };
    assert_eq!(
        fixed_context_error.to_string(),
        "Cranelift backend failure: module operation failed: fixed \
         host-dispatch context is not semantic process-pair storage"
    );

    B2F_PROCESS_PAIR_OBSERVATION.with(|cell| cell.set((0, 0)));
    set_process_slot_mutation(ProcessSlotMutation::ReintroduceLaunchIngress);

    // MEASURED: the positive control changes both ends of the route: the root
    // caller explicitly injects its launch-ingress pointer, and the callee
    // reloads the pair from that reintroduced pointer.
    // CLAIMED: process input and capability reach a unit only through declared
    // slots.
    // THE GAP: the retained host context is independently exercised above as
    // opaque storage; merely substituting offsets against it is not this
    // mutation and has no source-pair value to recover.
    let recovery_red = std::panic::catch_unwind(|| {
        let recovered = compile_b2f_process_pair_fixture()
            .expect("the explicit launch-ingress recovery mutation still emits")
            .run(Some((&ingress as *const RootIngressFixture).cast()))
            .expect("the explicit launch-ingress recovery mutation runs")
            .1
            .expect("the explicit launch-ingress recovery mutation returns a status");
        assert_eq!(
            recovered,
            0,
            "{RECOVERY_ASSERTION}"
        );
    })
    .expect_err("the reintroduced launch-ingress mutation must red");
    let recovery_message = recovery_red
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| recovery_red.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        recovery_message.contains(RECOVERY_ASSERTION),
        "the mutation failed at the wrong assertion: {recovery_message}"
    );
    assert_eq!(
        B2F_PROCESS_PAIR_OBSERVATION.with(std::cell::Cell::get),
        (0, 0),
        "the invalid raw launch-ingress pair reached the host"
    );
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
    assert!(crate::object_linker_packaging::process_starter_c_stub(&crate::boundary_resource_profile::starter_smoke_profile())
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
                            // D8: make this predecessor a declared-unit result.
                            // The borrowed Option eliminator must consume the
                            // precomputed CarrierWord plan rather than forcing
                            // its historical two-native-word merge.
                            body: RuntimeExpr::Call {
                                callee: Box::new(RuntimeExpr::LexicalClosure {
                                    captures: Vec::new(),
                                    params: Vec::new(),
                                    body: Box::new(RuntimeExpr::Value(RuntimeValue::Int(
                                        (7).into(),
                                    ))),
                                }),
                                args: Vec::new(),
                            },
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
    RawRemainingScalar,
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
                    // PX8-SPAN-PROV: reply-start span field shifted +1 (origin is field 0).
                    RuntimeExpr::Var(2),
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
            // PX8-SPAN-PROV: span is now [origin, start, budget]; every span-field
            // reference shifts +1 (budget: Var(1) -> Var(2)).
            binders: 3,
            body: px8n_exact_nat(symbols, RuntimeExpr::Var(2), 1, exact),
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

// ── RT-SPLIT slice 7, rule 8 finalization ─────────────────────────────────
// Residual facade test fixtures whose final-user LCA is this module. Facade
// file scope was a TRANSITIONAL zero-widening holding position, never final
// ownership (Architect `evt_h69xwchqqxmj`); slice 7 discharges it. Moved
// verbatim -- ordered item-level identity, no body edits.

#[cfg(test)]
#[repr(C)]
struct BorrowedFixtureValue {
    kind: u64,
    tag: u64,
    data: *const std::ffi::c_void,
    len: usize,
}

#[cfg(test)]
#[repr(C)]
struct RootIngressFixture {
    process_input: *const BorrowedFixtureValue,
    host_context: *mut std::ffi::c_void,
    capability: u64,
}

#[cfg(test)]
#[repr(C)]
struct DirectHostContextFixture {
    opaque: [*mut std::ffi::c_void; 3],
}

// RT-SPLIT slice 5: shared test helpers whose final users span the
// lowering subject subtree AND the facade's residual artifact/api tests.
// Final-user LCA is the facade, so they sit at facade FILE SCOPE under
// item-level `#[cfg(test)]` -- ancestor-private, reachable by descendants
// with zero widening. A sibling `mod tests` could not be reached at all.
#[cfg(test)]
const PX8N_SHORT_WROTE: u64 = 0;

#[cfg(test)]
const PX8N_ZERO_WRITE: u64 = 1;

#[cfg(test)]
fn run_px8n_write_arm_fixture(scenario: u64) -> (i64, Px8nHostReplyFixture) {
    run_px8n_arm_fixture(scenario, px8n_write_arm_fixture)
}

#[cfg(test)]
#[repr(C)]
struct Px8nHostReplyFixture {
    scenario: u64,
    call_index: u64,
    malformed_request: u64,
}

#[cfg(test)]
fn px8n_write_arm_fixture(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    px8n_write_arm_fixture_with_start(symbols, RuntimeExpr::Value(RuntimeValue::Int((7).into())))
}

#[cfg(test)]
fn run_px8n_arm_fixture(
    scenario: u64,
    expression: fn(&crate::NativeProcessSymbols) -> RuntimeExpr,
) -> (i64, Px8nHostReplyFixture) {
    let isa = native_isa().unwrap();
    let mut builder = JITBuilder::with_isa(isa, default_libcall_names());
    builder.symbol(
        "ken_host_dispatch_v1",
        px8n_scripted_host_dispatch as *const u8,
    );
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let compiled = compile_expr_into_module(
        JITModule::new(builder),
        "px8n_fs_write_at",
        Linkage::Local,
        &expression(&symbols),
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(&symbols),
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
        None,
    )
    .unwrap();
    let input = BorrowedFixtureValue {
        kind: 1,
        tag: 0,
        data: std::ptr::null(),
        len: 0,
    };
    let mut fixture = Px8nHostReplyFixture {
        scenario,
        call_index: 0,
        malformed_request: 0,
    };
    let invocation = RootIngressFixture {
        process_input: &input,
        host_context: (&mut fixture as *mut Px8nHostReplyFixture).cast(),
        capability: 0,
    };
    let (_, result) = compiled
        .run(Some((&invocation as *const RootIngressFixture).cast()))
        .unwrap();
    (result.unwrap(), fixture)
}

#[cfg(test)]
extern "C" fn px8n_scripted_host_dispatch(
    host_context: *const std::ffi::c_void,
    operation: i64,
    request: *const std::ffi::c_void,
    request_size: i64,
    reply: *mut std::ffi::c_void,
) -> i64 {
    // SAFETY: the direct context points to the live fixture for the duration
    // of the compiled call and is never retained by the dispatcher.
    let fixture = unsafe {
        &mut *(host_context
            .cast_mut()
            .cast::<Px8nHostReplyFixture>())
    };
    let expected = if fixture.call_index == 0
        || (fixture.call_index == 1 && fixture.scenario != PX8I_METADATA_BIG)
    {
        ken_host::HostOpV1::BufferAllocate
    } else if fixture.scenario == PX8I_METADATA_BIG {
        ken_host::HostOpV1::FsHandleMetadata
    } else if fixture.scenario == PX8I_WRAPPING_WRITE_START {
        ken_host::HostOpV1::FsWriteAt
    } else if fixture.scenario >= PX8N_SHORT_READ {
        ken_host::HostOpV1::FsReadAt
    } else {
        ken_host::HostOpV1::FsWriteAt
    };
    if operation != expected as i64 {
        fixture.malformed_request = 1;
        return -1;
    }
    let wire = ken_host::host_effect_wire_layout_v1(expected)
        .expect("PX8-N scripted operation has a generated wire layout");
    if request_size != i64::from(wire.request_size) {
        fixture.malformed_request = 2;
        return -1;
    }
    let load = |offset: u32| {
        // SAFETY: each offset is generated from the target-C layout for
        // this exact request record and the lowering supplied its size.
        unsafe { *(request.cast::<u8>().add(offset as usize).cast::<u64>()) }
    };
    if expected == ken_host::HostOpV1::BufferAllocate {
        if load(wire.request_offsets[0]) != 8 {
            fixture.malformed_request = 3;
            return -1;
        }
    } else if expected == ken_host::HostOpV1::FsHandleMetadata {
        if load(wire.request_offsets[0]) != 11 {
            fixture.malformed_request = 5;
            return -1;
        }
    } else if [
        load(wire.request_offsets[0]),
        load(wire.request_offsets[1]),
        load(wire.request_offsets[2]),
        load(wire.request_offsets[3]),
        load(wire.request_offsets[4]),
    ] != [
        11,
        22,
        0,
        match fixture.scenario {
            PX8I_BIG_READ_START => PX8I_BIG_U64,
            PX8I_WRAPPING_WRITE_START => u64::MAX - 1,
            _ => 7,
        },
        4,
    ] {
        fixture.malformed_request = 4;
        return -1;
    }
    // PX8-SPAN-PROV native ABI discriminator: FsWriteAt carries a 6th request
    // field (span_origin) beyond FsReadAt's five. It must marshal the distinct
    // origin operand (Var(1) = identity 11), not the target buffer (22). This
    // reddens if the lowering drops span_origin or target-substitutes it —
    // closing the seam a same-token own-write fixture leaves open.
    if expected == ken_host::HostOpV1::FsWriteAt && load(wire.request_offsets[5]) != 11 {
        fixture.malformed_request = 6;
        return -1;
    }
    // SAFETY: the reply pointer names the target-C-sized stack record
    // supplied by the compiled caller for this exact operation.
    unsafe { std::ptr::write_bytes(reply.cast::<u8>(), 0, wire.reply_size as usize) };
    let store = |offset: u32, value: u64| {
        // SAFETY: generated offsets are aligned u64 fields within the
        // zeroed reply record above.
        unsafe {
            *(reply.cast::<u8>().add(offset as usize).cast::<u64>()) = value;
        }
    };
    if expected == ken_host::HostOpV1::BufferAllocate {
        store(wire.reply_tag_offset, wire.reply_resource_tag);
        store(
            wire.reply_detail_offset,
            if fixture.call_index == 0 { 11 } else { 22 },
        );
    } else if expected == ken_host::HostOpV1::FsHandleMetadata {
        store(wire.reply_tag_offset, wire.reply_metadata_tag);
        store(wire.reply_detail_offset, PX8I_BIG_U64);
    } else {
        // BUDGET-EFF: every scripted FsReadAt/FsWriteAt scenario here uses
        // the uniform, unclamped request length 4 (validated above at
        // `request_offsets[4]`) — this scripted host never exercises a
        // buffer-capacity clamp, so the effective request equals the raw
        // one. Without this the reply's `effective_request` field stays at
        // the write_bytes zero-fill above and every reply with a nonzero
        // transferred count fails the new `count <= effective_request`
        // bound this WP added.
        store(wire.reply_effective_request_offset, 4);
        match fixture.scenario {
            PX8N_SHORT_WROTE | PX8I_WRAPPING_WRITE_START => {
                store(wire.reply_tag_offset, wire.reply_write_progress_tag);
                store(wire.reply_detail_offset, 1);
            }
            PX8N_ZERO_WRITE => {
                store(wire.reply_tag_offset, wire.reply_resource_error_tag);
                store(wire.reply_detail_offset, wire.resource_error_no_progress);
            }
            PX8N_OVER_BOUND_WRITE => {
                store(wire.reply_tag_offset, wire.reply_write_progress_tag);
                store(wire.reply_detail_offset, 5);
            }
            PX8N_SHORT_READ => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                store(wire.reply_detail_offset, 1);
                store(wire.reply_bytes_len_offset, 7);
            }
            PX8N_READ_EOF => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
            }
            PX8N_OVER_BOUND_READ => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                store(wire.reply_detail_offset, 5);
                store(wire.reply_bytes_len_offset, 7);
            }
            PX8I_BIG_READ_START => {
                store(wire.reply_tag_offset, wire.reply_read_progress_tag);
                store(wire.reply_detail_offset, 1);
                store(wire.reply_bytes_len_offset, PX8I_BIG_U64);
            }
            _ => return -1,
        }
    }
    fixture.call_index += 1;
    0
}

#[cfg(test)]
fn px8n_write_arm_fixture_with_start(
    symbols: &crate::NativeProcessSymbols,
    start: RuntimeExpr,
) -> RuntimeExpr {
    let trap = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-N checked result default".to_string(),
    };
    let allocate = || RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::BufferAllocate,
        capability: None,
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
    };
    let write = RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsWriteAt,
        capability: None,
        args: vec![
            RuntimeExpr::Var(1),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Var(0),
            start,
            RuntimeExpr::Value(RuntimeValue::Int((4).into())),
            // PX8-SPAN-PROV native ABI discriminator: span_origin is a *distinct*
            // resource operand (Var(1), erased identity 11) from the target
            // buffer (Var(0), identity 22), so the scripted host below verifies
            // the 6th FsWriteAt request field carries the distinct origin token,
            // not the target. A native lowering/ABI bug that dropped or
            // target-substituted span_origin sends 22 to request_offsets[5] and
            // is caught. (The scripted host returns Wrote regardless of
            // provenance — it validates the marshalled request, not the check.)
            RuntimeExpr::Var(1),
        ],
    };
    let transfer_observation = px8n_exact_nat(
        symbols,
        RuntimeExpr::Var(0),
        0,
        px8n_exact_nat(
            symbols,
            RuntimeExpr::Var(1),
            3,
            RuntimeExpr::Value(RuntimeValue::Int((3).into())),
        ),
    );
    let success = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![crate::RuntimeMatchCase {
            constructor: symbols.wrote.clone(),
            binders: 1,
            body: RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Var(0)),
                cases: vec![crate::RuntimeMatchCase {
                    constructor: symbols.private_transfer_count.clone(),
                    binders: 2,
                    body: px8n_failure(symbols, transfer_observation),
                }],
                default: trap(),
            },
        }],
        default: trap(),
    };
    let error = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![crate::RuntimeMatchCase {
            constructor: symbols.resource_no_progress.clone(),
            binders: 0,
            body: px8n_failure(symbols, RuntimeExpr::Value(RuntimeValue::Int((70).into()))),
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-N expected exact NoProgress".to_string(),
        },
    };
    let write_result = RuntimeExpr::Match {
        scrutinee: Box::new(write),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: error,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: success,
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
                body: write_result,
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

#[cfg(test)]
const PX8N_SHORT_READ: u64 = 3;

#[cfg(test)]
const PX8I_METADATA_BIG: u64 = 6;

#[cfg(test)]
const PX8I_WRAPPING_WRITE_START: u64 = 8;

#[cfg(test)]
const PX8I_BIG_U64: u64 = i64::MAX as u64 + 97;

#[cfg(test)]
fn px8n_exact_nat(
    symbols: &crate::NativeProcessSymbols,
    nat: RuntimeExpr,
    depth: usize,
    exact: RuntimeExpr,
) -> RuntimeExpr {
    let mismatch = RuntimeExpr::Value(RuntimeValue::Int((99).into()));
    let cases = if depth == 0 {
        vec![
            crate::RuntimeMatchCase {
                constructor: symbols.nat_zero.clone(),
                binders: 0,
                body: exact,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.nat_suc.clone(),
                binders: 1,
                body: mismatch,
            },
        ]
    } else {
        vec![
            crate::RuntimeMatchCase {
                constructor: symbols.nat_zero.clone(),
                binders: 0,
                body: mismatch,
            },
            crate::RuntimeMatchCase {
                constructor: symbols.nat_suc.clone(),
                binders: 1,
                body: px8n_exact_nat(symbols, RuntimeExpr::Var(0), depth - 1, exact),
            },
        ]
    };
    RuntimeExpr::Match {
        scrutinee: Box::new(nat),
        cases,
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: format!("PX8-N expected exact structural Nat depth {depth}"),
        },
    }
}

#[cfg(test)]
fn px8n_failure(symbols: &crate::NativeProcessSymbols, code: RuntimeExpr) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: symbols.exit_failure.clone(),
        args: vec![code],
    }
}

#[cfg(test)]
const PX8N_OVER_BOUND_WRITE: u64 = 2;

#[cfg(test)]
const PX8N_READ_EOF: u64 = 4;

#[cfg(test)]
const PX8N_OVER_BOUND_READ: u64 = 5;

#[cfg(test)]
const PX8I_BIG_READ_START: u64 = 7;
