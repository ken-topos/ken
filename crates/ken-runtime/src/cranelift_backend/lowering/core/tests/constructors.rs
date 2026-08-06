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
use crate::cranelift_backend::artifact::native_isa_for_lowering_tests as native_isa;
use crate::cranelift_backend::artifact::native_platform_target_name_for_lowering_tests as native_platform_target_name;
use crate::fnv1a_64;

fn test_synthesized_constructor_identity() -> ConstructorIdentity {
    inert_test_plan()
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::Unit,
        ))
        .expect("the inert plan inventories the fixed Unit role")
}

#[test]
fn c2_ac2_closed_roles_are_injective_by_spelling_and_canonical_for_duplicates() {
    let expr = RuntimeExpr::Value(RuntimeValue::Bool(true));
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let distinct = plan_static_transition_graph_with_symbols(
        &expr,
        &BTreeMap::new(),
        &symbols,
        AbiRootIngress::Value,
        true,
    )
    .expect("the distinct-role fixture plans");
    let file_error = distinct
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::FileError,
        ))
        .expect("FileError is inventoried");
    let unit = distinct
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::Unit,
        ))
        .expect("Unit is inventoried");
    assert_ne!(
        file_error, unit,
        "distinct synthesized-role spellings must not alias"
    );
    assert_eq!(
        distinct.synthesized_io_error_roles().len(),
        symbols.io_errors.len(),
        "the dynamic inventory must be derived from every IOError alternative"
    );
    for role in distinct.synthesized_io_error_roles() {
        distinct
            .synthesized_constructor_identity(SynthesizedConstructorRole::IoError(*role))
            .expect("every minted IOError role resolves");
    }

    let mut duplicate_symbols = symbols;
    duplicate_symbols.unit = duplicate_symbols.file_error.clone();
    let duplicate = plan_static_transition_graph_with_symbols(
        &expr,
        &BTreeMap::new(),
        &duplicate_symbols,
        AbiRootIngress::Value,
        true,
    )
    .expect("the duplicate-spelling fixture plans");
    let duplicate_file_error = duplicate
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::FileError,
        ))
        .expect("FileError is inventoried");
    let duplicate_unit = duplicate
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::Unit,
        ))
        .expect("Unit is inventoried");
    assert_eq!(
        duplicate_file_error, duplicate_unit,
        "duplicate role spellings must converge through the plane's one interner"
    );
}

#[test]
fn c2_ac3_missing_dynamic_role_refuses_at_some_zero_unit_epoch() {
    let expr = RuntimeExpr::Value(RuntimeValue::Bool(true));
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let result = with_last_io_error_role_omitted(|| {
        compile_expr_into_module(
            new_jit_module().expect("JIT module constructs"),
            "c2_missing_role",
            Linkage::Local,
            &expr,
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            false,
            Some(&symbols),
            None,
            None,
        )
    });
    let error = match result {
        Ok(_) => panic!("an omitted dynamic role must refuse compilation"),
        Err(error) => error,
    };
    assert!(
        matches!(
            error,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(ref reason))
                if reason.contains("IoError")
                    && reason.contains("absent from the closed inventory")
        ),
        "the exact omitted dynamic role must own the refusal: {error:?}"
    );
    assert_eq!(
        c2_unit_emission_epoch(),
        Some(0),
        "Some(0) proves compilation reached the pre-emission seam and declared \
         no unit; None would mean the seam was never observed"
    );
}

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
        body_emission_authority: BodyEmissionAuthority::FunctionizedUnits,
        continuation_claims: None,
        checked_call_ledger: None,
        defining_unit: None,
        defining_emission_owner: None,
        defining_function_id: None,
        aggregate_allocations: None,
        host_effect_seats: None,
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        // ⛔ `None` — a bare `Lowering` fixture emits into no module, so it has
        // no callable carrier refs. The `Carried` routes fail closed on this
        // rather than silently taking the `Specialized` path.
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
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
            raw_worker_calls: BTreeMap::new(),
            continuation_calls: BTreeMap::new(),
            continuation_emissions: BTreeMap::new(),
            pending_composed_discharges: Vec::new(),
            composed_discharges: BTreeMap::new(),
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
        let dynamic = DynamicConstructorV1 {
            discriminator: builder.ins().iconst(types::I64, discriminator),
            alternatives: vec![
                DynamicConstructorAlternativeV1 {
                    tag: 0,
                    constructor: "ctor:fixture::Dynamic::Zero".to_string(),
                    identity: test_synthesized_constructor_identity(),
                    occurrence: None,
                    fields: Vec::new(),
                },
                DynamicConstructorAlternativeV1 {
                    tag: 1,
                    constructor: "ctor:fixture::Dynamic::One".to_string(),
                    identity: test_synthesized_constructor_identity(),
                    occurrence: None,
                    fields: vec![Lowered::Int {
                        value: builder.ins().iconst(types::I64, 7),
                        known: Some(7),
                    }],
                },
            ],
        };
        let (plan, match_origin) = planned_root_occurrence(&source_match);
        compiler.static_transition_plan = plan;
        compiler.enter_source_occurrence_plan(match_origin)?;
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
        identity: test_synthesized_constructor_identity(),
                    occurrence: None,
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
        identity: test_synthesized_constructor_identity(),
                    occurrence: None,
        fields: vec![
            Lowered::Bytes(b"first".to_vec()),
            Lowered::String("second".to_string()),
        ],
    };
    let env = materialize_dynamic_constructor_env(
        &alternative,
        &[LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(
            Lowered::Bytes(b"outer".to_vec()),
        ))],
    );
    assert!(
        matches!(
            &env[0],
            LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(Lowered::Bytes(
                value,
            ))) if value == b"first"
        )
    );
    assert!(
        matches!(
            &env[1],
            LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(Lowered::String(
                value,
            ))) if value == "second"
        )
    );
    assert!(
        matches!(
            &env[2],
            LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(Lowered::Bytes(
                value,
            ))) if value == b"outer"
        )
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

extern "C" fn final_kind_discriminator_host_probe(
    host_context: *const std::ffi::c_void,
    operation: i64,
    _request: *const std::ffi::c_void,
    _request_size: i64,
    reply: *mut std::ffi::c_void,
) -> i64 {
    if host_context.is_null() || reply.is_null() {
        return -1;
    }
    // SAFETY: `run_final_kind_discriminator_fixture` supplies this exact
    // call-scoped `u64` as the direct host context.
    let observation = host_context.cast::<u64>().cast_mut();
    // Mark the exact call-scoped selector as observed. The caller checks this
    // after execution so a lost host-context edge cannot masquerade as a
    // discriminator result.
    unsafe {
        *observation |= 2;
    }
    let Ok(operation) = ken_host::HostOpV1::try_from(operation as u16) else {
        return -1;
    };
    let Ok(layout) = ken_host::host_effect_wire_layout_v1(operation) else {
        return -1;
    };
    // SAFETY: the generated caller supplies the target-C-sized reply record.
    unsafe {
        std::ptr::write_bytes(reply.cast::<u8>(), 0, layout.reply_size as usize);
        let reply_tag = reply
            .cast::<u8>()
            .add(layout.reply_tag_offset as usize)
            .cast::<u64>();
        let reply_detail = reply
            .cast::<u8>()
            .add(layout.reply_detail_offset as usize)
            .cast::<u64>();
        match operation {
            ken_host::HostOpV1::ConsoleWrite => {
                *reply_tag = layout.reply_unit_tag;
            }
            ken_host::HostOpV1::ConsoleIsTerminal => {
                *reply_tag = layout.reply_bool_tag;
                *reply_detail = 1;
            }
            _ => return -1,
        }
        *observation |= 4;
    }
    0
}

fn run_final_kind_discriminator_fixture(fixture: &RuntimeExpr, symbol: &str) -> i64 {
    let isa = native_isa().expect("native ISA");
    let mut jit =
        cranelift_jit::JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    jit.symbol(
        "ken_host_dispatch_v1",
        final_kind_discriminator_host_probe as *const u8,
    );
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let compiled = compile_expr_into_module(
        cranelift_jit::JITModule::new(jit),
        symbol,
        Linkage::Local,
        fixture,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(&symbols),
        Some(test_only_distinguished_root_join_plan()),
        None,
    )
    .expect("the CarrierWord final-kind fixture emits");
    let process_input = 0_u8;
    let mut host_observation = 0_u64;
    let ingress = crate::boundary_activation::GeneratedRootIngressV1 {
        process_input: (&process_input as *const u8).cast(),
        host_dispatch_context: (&mut host_observation as *mut u64).cast(),
        capability: 1_u64 << 32,
    };
    let status = compiled
        .run(Some(
            (&ingress as *const crate::boundary_activation::GeneratedRootIngressV1).cast(),
        ))
        .expect("the CarrierWord final-kind fixture runs")
        .1
        .expect("the process root returns a status");
    assert_eq!(
        host_observation, 6,
        "the direct host context must complete the intended runtime scalar arm"
    );
    status
}

fn assert_runtime_final_kind_discriminator_rejects_scalar(fixture: &RuntimeExpr, symbol: &str) {
    // Promise class: durable invariant. CarrierWord may change storage, but the
    // process root must still reject an Int-tagged word as an exit status.
    //
    // MEASURED: the same source fixture emits, then a runtime host reply selects
    // its scalar alternative; the process root returns the wrong-tag guard -1.
    // CLAIMED: the heterogeneous CarrierWord join defers final-kind validation
    // to the emitted process-root discriminator without accepting Int as status.
    // THE GAP: this pin observes the wrong-tag arm only. The companion object
    // emission above establishes that the heterogeneous source population is
    // accepted; this assertion does not re-prove every well-tagged exit route.
    let scalar_status = run_final_kind_discriminator_fixture(fixture, &format!("{symbol}_scalar"));
    assert_eq!(
        scalar_status, -1,
        "the emitted process-root discriminator must reject the wrong Int tag"
    );
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
fn dynamic_host_result_producer_carrier_final_kind_is_runtime_guarded() {
    let fixture = host_result_computational_fixture(1, true, true);
    emit_process_entrypoint_object_with_cranelift(&fixture, "ken_px7m_kind_mismatch")
        .expect("the CarrierWord result join emits its runtime final-kind discriminator");
    assert_runtime_final_kind_discriminator_rejects_scalar(
        &fixture,
        "ken_px7m_kind_mismatch_runtime",
    );
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
fn nested_computational_carrier_final_kind_is_runtime_guarded() {
    let fixture = nested_computational_fixture(1, Vec::new(), true, true);
    emit_process_entrypoint_object_with_cranelift(&fixture, "ken_px7n_final_kind_mismatch")
        .expect("the CarrierWord result join emits its runtime final-kind discriminator");
    assert_runtime_final_kind_discriminator_rejects_scalar(
        &fixture,
        "ken_px7n_final_kind_mismatch_runtime",
    );
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
fn heterogeneous_bridge_removal_uses_the_runtime_constructor_route() {
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
    emit_process_entrypoint_object_with_cranelift(&bridge_removed, "ken_px7o_bridge_removed")
        .expect("the functionized carrier retains the runtime constructor discriminator");
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
fn heterogeneous_final_merge_kind_is_deferred_to_the_runtime_discriminator() {
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
    emit_process_entrypoint_object_with_cranelift(&expr, "ken_px7o_final_kind_mismatch")
        .expect("the functionized route emits the dynamic final-kind discriminator");
}
#[test]
fn heterogeneous_ordinary_arity_is_guarded_in_the_emitted_consumer() {
    emit_process_entrypoint_object_with_cranelift(
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
    .expect("the functionized consumer emits its runtime binder-arity guard");
}
#[test]
fn heterogeneous_nested_payload_kind_is_guarded_in_the_emitted_consumer() {
    emit_process_entrypoint_object_with_cranelift(
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
    .expect("the functionized consumer preserves the runtime payload-kind guard");
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
        body_emission_authority: BodyEmissionAuthority::FunctionizedUnits,
        continuation_claims: None,
        checked_call_ledger: None,
        defining_unit: None,
        defining_emission_owner: None,
        defining_function_id: None,
        aggregate_allocations: None,
        host_effect_seats: None,
        process_object: false,
        process_symbols: crate::NativeProcessSymbols::legacy_prelude(),
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
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
            raw_worker_calls: BTreeMap::new(),
            continuation_calls: BTreeMap::new(),
            continuation_emissions: BTreeMap::new(),
            pending_composed_discharges: Vec::new(),
            composed_discharges: BTreeMap::new(),
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
    }
}

fn bind_bare_test_trap_lane(
    compiler: &mut Lowering<'_>,
    builder: &mut FunctionBuilder<'_>,
) {
    let lane = builder.create_sized_stack_slot(StackSlotData::new(
        StackSlotKind::ExplicitSlot,
        8,
        3,
    ));
    compiler.function_local.trap_exit = None;
    compiler
        .function_local
        .bind_unit_trap_frame(builder.ins().stack_addr(types::I64, lane, 0), 0)
        .expect("the bare fixture owns its synthetic unit trap lane");
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
    // ⭐ `D7` — resolved BEFORE the plan is moved into the compiler, because a
    // hand-built source aggregate owes its own producer occurrence exactly as
    // the `Construct` arm's does. ⛔ Both graphs below get it: the ordering
    // claim under test is about the ADMISSIBILITY walk beating the carrier
    // step, so neither arm may be decided by a missing-producer refusal that is
    // an artifact of the rig.
    let wrap_occurrence = plan
        .source_aggregate_occurrence(construct_origin, PlannedAggregateShape::Constructor)
        .expect("the planned `Construct` has an ownership record at its own origin");
    let closure_body = inert_test_static_origin();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    bind_bare_test_trap_lane(&mut compiler, &mut builder);

    // ── the inadmissible graph: the closure is one level DOWN ─────────────
    //
    // ⚠ Nested deliberately. A closure at the ROOT would be refused by the
    // root variant's own disposition, so it could not distinguish the walk
    // from the disposition table. The walk is the only thing that sees this.
    let inadmissible = Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        synthesized_identity: None,
        occurrence: Some(wrap_occurrence),
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
        synthesized_identity: None,
        occurrence: Some(wrap_occurrence),
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
    bind_bare_test_trap_lane(&mut compiler, &mut builder);

    let seeded_word = builder.ins().iconst(types::I64, 0x0c1_d3);

    // ── the carried phase ─────────────────────────────────────────────────
    let carried_env = [LoweringEnvironmentBinding::Value(LoweringOperand::Carried(
        CarriedBoundaryWord { word: seeded_word },
    ))];
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
    let specialized_env = [LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(
        Lowered::Bool {
            value: seeded_word,
            known: Some(true),
        },
    ))];
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
    ac_c7_try_compile_edge_with_operands(seed_env, plan, 0, |compiler, builder, _| {
        emit(compiler, builder)
    })
}

/// The same rig, with `operands` extra `i64` parameters after the arena.
///
/// ⭐⭐ **Why a rig with RUNTIME operands exists at all.** Every row above
/// compiles one body per fixture, so a body that specialized on a JIT-time
/// constant would be indistinguishable from one that decided at run time — the
/// two compilations differ, and either could be what produced the two answers.
/// ⇒ For any claim of the form *"emitted code makes this choice from the
/// **value**"* the discriminator has to be **one compiled body driven with two
/// payloads**, which is what these parameters are for. ⛔ Nothing else in this
/// file can establish `AC-2`.
fn ac_c7_try_compile_edge_with_operands<'src>(
    seed_env: &'src NativeSeedEnvironment,
    plan: StaticTransitionPlan<'src>,
    operands: usize,
    emit: impl FnOnce(
        &mut Lowering<'src>,
        &mut FunctionBuilder<'_>,
        &[cranelift_codegen::ir::Value],
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
    if operands == 0 {
        signature.params.push(AbiParam::new(pointer));
    }
    for _ in 0..operands {
        signature.params.push(AbiParam::new(types::I64));
    }
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("c1_ac_c7_edge", Linkage::Local, &signature)
        .expect("edge probe declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);

    let carrier = BoundaryCarrierRefs {
        class: module.declare_func_in_func(helpers.class, &mut context.func),
        tag: module.declare_func_in_func(helpers.tag, &mut context.func),
        field_count: module.declare_func_in_func(helpers.field_count, &mut context.func),
        field: module.declare_func_in_func(helpers.field, &mut context.func),
        record_field: module.declare_func_in_func(helpers.record_field, &mut context.func),
        scalar: module.declare_func_in_func(helpers.scalar, &mut context.func),
        host_success: module.declare_func_in_func(helpers.host_success, &mut context.func),
        host_payload: module.declare_func_in_func(helpers.host_payload, &mut context.func),
        alloc: module.declare_func_in_func(helpers.alloc, &mut context.func),
        store_tag_id: module.declare_func_in_func(helpers.store_tag_id, &mut context.func),
        store_scalar: module.declare_func_in_func(helpers.store_scalar, &mut context.func),
        store_field: module.declare_func_in_func(helpers.store_field, &mut context.func),
        store_name: module.declare_func_in_func(helpers.store_name, &mut context.func),
        make_immediate: module.declare_func_in_func(helpers.make_immediate, &mut context.func),
        store_int_tag: module.declare_func_in_func(helpers.store_int_tag, &mut context.func),
        store_bytes_len: module.declare_func_in_func(helpers.store_bytes_len, &mut context.func),
        store_byte: module.declare_func_in_func(helpers.store_byte, &mut context.func),
        store_int_limbs: module.declare_func_in_func(helpers.store_int_limbs, &mut context.func),
        store_int_limb: module.declare_func_in_func(helpers.store_int_limb, &mut context.func),
        seal_int: module.declare_func_in_func(helpers.seal_int, &mut context.func),
        int_view: module.declare_func_in_func(helpers.int_view, &mut context.func),
    };

    let mut compiler = bare_carrier_test_lowering(seed_env, plan);
    compiler.function_local.boundary_carrier = Some(carrier);
    // ⭐ The native-`Int` authority, resolved into THIS function. ⛔ Without
    // these the wide-`Int` arm cannot decode a pair and the rig would measure a
    // refusal rather than the copy.
    compiler.function_local.native_int_intern =
        Some(module.declare_func_in_func(native.intern, &mut context.func));
    compiler.function_local.native_int_resolve =
        Some(module.declare_func_in_func(native.resolve, &mut context.func));

    let mut function_context = FunctionBuilderContext::new();
    let refused = {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let parameters = builder.block_params(entry).to_vec();
        // ⭐ In THIS rig parameter 0 is genuinely the boundary arena — the test
        // passes `BoundaryArenaV1::publish()` — and the native arena is its
        // `ARENA_NATIVE_INT` binding. ⛔ Setting both from one value would
        // reinstate the equality the Architect's ruling deletes; the native
        // field is left for the fixtures that bind one.
        compiler.function_local.boundary_arena = Some(parameters[0]);
        let emitted_operands = if operands == 0 {
            compiler.function_local.trap_exit = None;
            compiler
                .function_local
                .bind_unit_trap_frame(parameters[1], 0)
                .expect("the zero-operand fixture owns its unit trap lane");
            &parameters[2..]
        } else {
            bind_bare_test_trap_lane(&mut compiler, &mut builder);
            &parameters[1..]
        };
        // â  A refusal must still leave a WELL-FORMED function behind, or the
        // failure the caller wanted to observe is replaced by a Cranelift
        // assertion about an unfilled block. â­ Every carrier route refuses
        // *before* it creates a block â the termination guard and the
        // empty-case check both say so at their sites â so on the error path
        // the entry block is still current and still empty, and returning a
        // constant from it is sound.
        match emit(&mut compiler, &mut builder, emitted_operands) {
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
    let f: extern "C" fn(*const u64, *mut i64) -> i64 =
        unsafe { std::mem::transmute(code) };
    let mut trap_identity = 0;
    let result = f(arena, &mut trap_identity);
    if trap_identity == 0 {
        result
    } else {
        -4
    }
}

fn c2_compile_edge_with_arg<'src>(
    name: &str,
    seed_env: &'src NativeSeedEnvironment,
    plan: StaticTransitionPlan<'src>,
    emit: impl FnOnce(
        &mut Lowering<'src>,
        &mut FunctionBuilder<'_>,
        cranelift_codegen::ir::Value,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError>,
) -> (cranelift_jit::JITModule, *const u8) {
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
    signature.params.push(AbiParam::new(types::I64));
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function(name, Linkage::Local, &signature)
        .expect("C2 edge declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let carrier = BoundaryCarrierRefs {
        class: module.declare_func_in_func(helpers.class, &mut context.func),
        tag: module.declare_func_in_func(helpers.tag, &mut context.func),
        field_count: module.declare_func_in_func(helpers.field_count, &mut context.func),
        field: module.declare_func_in_func(helpers.field, &mut context.func),
        record_field: module.declare_func_in_func(helpers.record_field, &mut context.func),
        scalar: module.declare_func_in_func(helpers.scalar, &mut context.func),
        host_success: module.declare_func_in_func(helpers.host_success, &mut context.func),
        host_payload: module.declare_func_in_func(helpers.host_payload, &mut context.func),
        alloc: module.declare_func_in_func(helpers.alloc, &mut context.func),
        store_tag_id: module.declare_func_in_func(helpers.store_tag_id, &mut context.func),
        store_scalar: module.declare_func_in_func(helpers.store_scalar, &mut context.func),
        store_field: module.declare_func_in_func(helpers.store_field, &mut context.func),
        store_name: module.declare_func_in_func(helpers.store_name, &mut context.func),
        make_immediate: module.declare_func_in_func(helpers.make_immediate, &mut context.func),
        store_int_tag: module.declare_func_in_func(helpers.store_int_tag, &mut context.func),
        store_bytes_len: module.declare_func_in_func(helpers.store_bytes_len, &mut context.func),
        store_byte: module.declare_func_in_func(helpers.store_byte, &mut context.func),
        store_int_limbs: module.declare_func_in_func(helpers.store_int_limbs, &mut context.func),
        store_int_limb: module.declare_func_in_func(helpers.store_int_limb, &mut context.func),
        seal_int: module.declare_func_in_func(helpers.seal_int, &mut context.func),
        int_view: module.declare_func_in_func(helpers.int_view, &mut context.func),
    };
    let mut compiler = bare_carrier_test_lowering(seed_env, plan);
    compiler.function_local.boundary_carrier = Some(carrier);
    let mut function_context = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let parameters = builder.block_params(entry).to_vec();
        // This rig receives the published boundary arena directly.  Its
        // carrier producer/consumer paths do not use native-Int services.
        compiler.function_local.boundary_arena = Some(parameters[0]);
        bind_bare_test_trap_lane(&mut compiler, &mut builder);
        let result = emit(&mut compiler, &mut builder, parameters[1])
            .expect("the C2 carrier edge emits");
        builder.ins().return_(&[result]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    module
        .define_function(func_id, &mut context)
        .expect("C2 edge defines");
    module.finalize_definitions().expect("JIT finalizes");
    let code = module.get_finalized_function(func_id);
    (module, code)
}

fn c2_run_edge_with_arg(code: *const u8, arena: *const u64, argument: i64) -> i64 {
    let function: extern "C" fn(*const u64, i64) -> i64 =
        unsafe { std::mem::transmute(code) };
    function(arena, argument)
}

#[test]
fn c2_ac4_runtime_host_result_selects_a_separately_generated_nested_payload() {
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let nested_default = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "C2 nested constructor default".to_string(),
    };
    let match_expr = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![
            RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![RuntimeMatchCase {
                        constructor: symbols.wrote.clone(),
                        binders: 1,
                        body: RuntimeExpr::Var(0),
                    }],
                    default: nested_default(),
                },
            },
            RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(0)),
                    cases: vec![RuntimeMatchCase {
                        constructor: symbols.read_some.clone(),
                        binders: 1,
                        body: RuntimeExpr::Var(0),
                    }],
                    default: nested_default(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "C2 HostResult default".to_string(),
        },
    };
    let ordinary_producer_expr = RuntimeExpr::Construct {
        constructor: symbols.result_ok.clone(),
        args: vec![RuntimeExpr::Construct {
            constructor: symbols.read_some.clone(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
        }],
    };
    let planned_fixture = RuntimeExpr::Let {
        // The separate producer is a declared unit, so its result reaches the
        // consumer through the carrier ABI. Keep that source fact in the plan
        // instead of relying on the test rig's later manual carrier injection.
        value: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(ordinary_producer_expr),
            }),
            args: Vec::new(),
        }),
        body: Box::new(match_expr.clone()),
    };
    let plan = plan_static_transition_graph_with_symbols(
        &planned_fixture,
        &BTreeMap::new(),
        &symbols,
        AbiRootIngress::Value,
        true,
    )
    .expect("the C2 producer/consumer fixture plans");
    let root = plan.root_static_origin().expect("root occurrence exists");
    let producer_call_origin = plan
        .child_static_origin(root, 0)
        .expect("the ordinary Result producer call exists");
    let producer_closure_origin = plan
        .child_static_origin(producer_call_origin, 0)
        .expect("the ordinary Result producer closure exists");
    let ordinary_producer_origin = plan
        .child_static_origin(producer_closure_origin, 0)
        .expect("the ordinary Result producer body exists");
    let match_origin = plan
        .child_static_origin(root, 1)
        .expect("the shared Result consumer occurrence exists");
    let read_some = plan
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::ReadSome,
        ))
        .expect("ReadSome is inventoried")
        .tag_abi_word()
        .expect("ReadSome identity projects");
    let wrote = plan
        .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(
            SynthesizedFixedConstructorRole::Wrote,
        ))
        .expect("Wrote is inventoried")
        .tag_abi_word()
        .expect("Wrote identity projects");
    assert_ne!(
        read_some, wrote,
        "the two runtime arms need distinct identities or selection is vacuous"
    );

    let seed_env = NativeSeedEnvironment::empty();
    let producer_plan = plan.clone();
    let producer_symbols = symbols.clone();
    let (_producer_module, producer) = c2_compile_edge_with_arg(
        "c2_host_result_producer",
        &seed_env,
        producer_plan,
        move |compiler, builder, success| {
            let true_word = builder.ins().iconst(types::I64, 1);
            let false_word = builder.ins().iconst(types::I64, 0);
            let discriminator = builder.ins().iconst(types::I64, 0);
            let ok_identity = compiler
                .synthesized_fixed_identity(SynthesizedFixedConstructorRole::ReadSome)?;
            let ok = Lowered::DynamicConstructor(DynamicConstructorV1 {
                discriminator,
                alternatives: vec![DynamicConstructorAlternativeV1 {
                    tag: 0,
                    constructor: producer_symbols.read_some.clone(),
                    identity: ok_identity,
                    occurrence: None,
                    fields: vec![Lowered::Bool {
                        value: true_word,
                        known: Some(true),
                    }],
                }],
            });
            // `D7` — this fixture has no `Effect` occurrence, so `match_origin`
            // is not a producer seat and carries no per-use record. That is
            // correct and leaves the row's existing refusal unchanged: the
            // template gets no occurrence and refuses at the allocation, which
            // is where it already fails.
            let error = compiler.synthesized_constructor(
                match_origin,
                &SynthesizedAggregatePath::root(SynthesizedAggregateRoot::HostResultOk),
                SynthesizedFixedConstructorRole::Wrote,
                producer_symbols.wrote.clone(),
                vec![SynthesizedArgument::Scalar(Lowered::Bool {
                    value: false_word,
                    known: Some(false),
                })],
                &ClaimedEffectSeats::none(),
            )?;
            let host_result = Lowered::HostResult {
                success,
                error: Box::new(error),
                ok: Box::new(ok),
                err_constructor: producer_symbols.result_err.clone(),
                ok_constructor: producer_symbols.result_ok.clone(),
            };
            Ok(compiler
                .transfer_into_carrier(builder, match_origin, &host_result)?
                .word)
        },
    );

    let ordinary_producer_plan = plan.clone();
    assert_eq!(
        ordinary_producer_plan
            .constructor_symbol_identity(ordinary_producer_origin)
            .expect("the ordinary Result producer identity exists")
            .tag_abi_word()
            .expect("the ordinary Result producer identity projects"),
        plan.case_constructor_identity(match_origin, 1)
            .expect("the consumer Result::Ok identity exists")
            .tag_abi_word()
            .expect("the consumer Result::Ok identity projects"),
        "separately generated producer and consumer occurrences in one plan \
         must converge for Result::Ok"
    );
    let ordinary_symbols = symbols.clone();
    let (_ordinary_producer_module, ordinary_producer) = c2_compile_edge_with_arg(
        "c2_ordinary_result_producer",
        &seed_env,
        ordinary_producer_plan,
        move |compiler, builder, _| {
            let true_word = builder.ins().iconst(types::I64, 1);
            let ordinary_result = Lowered::Constructor {
                constructor: ordinary_symbols.result_ok.clone(),
                synthesized_identity: None,
                occurrence: None,
                args: vec![Lowered::Constructor {
                    constructor: ordinary_symbols.read_some.clone(),
                    synthesized_identity: None,
                    occurrence: None,
                    args: vec![Lowered::Bool {
                        value: true_word,
                        known: Some(true),
                    }],
                }],
            };
            Ok(compiler
                .transfer_into_carrier(
                    builder,
                    ordinary_producer_origin,
                    &ordinary_result,
                )?
                .word)
        },
    );

    let consumer_plan = plan;
    let (_consumer_module, consumer) = c2_compile_edge_with_arg(
        "c2_host_result_consumer",
        &seed_env,
        consumer_plan,
        |compiler, builder, word| {
            compiler.enter_source_occurrence_plan(match_origin)?;
            let lowered = compiler.lower_carried_match(
                builder,
                CarriedBoundaryWord { word },
                match match_expr {
                    RuntimeExpr::Match { ref cases, .. } => cases,
                    _ => unreachable!("fixture is a Match"),
                },
                &RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "C2 HostResult default".to_string(),
                },
                match_origin,
                &[],
            )?;
            let LoweringOperand::Carried(observed) = lowered else {
                return Err(unsupported(
                    "HostResult",
                    "the separately generated consumer recovered a compile-time template",
                ));
            };
            Ok(observed.word)
        },
    );

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let success_word = c2_run_edge_with_arg(producer, base, 1);
    let success_observed = c2_run_edge_with_arg(consumer, base, success_word);
    let true_boundary_word =
        (1u64 << crate::boundary_value::BOUNDARY_TAG_BITS)
            | BoundaryTag::ImmediateBool as u64;
    assert_eq!(
        success_observed as u64,
        true_boundary_word,
        "runtime success must select the DynamicConstructor payload, preserve \
         its D2 identity, match it through the ordinary tag helper, and project \
         its field"
    );

    let error_word = c2_run_edge_with_arg(producer, base, 0);
    let error_observed = c2_run_edge_with_arg(consumer, base, error_word);
    assert_eq!(
        error_observed as u64,
        BoundaryTag::ImmediateBool as u64,
        "runtime error must select the synthesized Constructor payload, preserve \
         its D2 identity, match it through the ordinary tag helper, and project \
         its field"
    );
    assert_ne!(
        success_observed, error_observed,
        "the runtime success bit must change the separately generated consumer's answer"
    );

    let ordinary_word = c2_run_edge_with_arg(ordinary_producer, base, 0);
    assert!(
        ordinary_word >= 0,
        "the separately generated ordinary Result producer must emit a carrier \
         word, got {ordinary_word}"
    );
    let ordinary_observed = c2_run_edge_with_arg(consumer, base, ordinary_word);
    assert_eq!(
        ordinary_observed as u64, true_boundary_word,
        "an ordinary source Result constructor must use the ordinary tag/field \
         route through the same consumer and project its nested payload"
    );
}

#[test]
fn c2_ac6_host_result_covers_resource_token_and_response_bytes_payloads() {
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let expr = RuntimeExpr::Value(RuntimeValue::Bool(true));
    let plan = plan_static_transition_graph_with_symbols(
        &expr,
        &BTreeMap::new(),
        &symbols,
        AbiRootIngress::Value,
        true,
    )
    .expect("the C2 covered-class fixture plans");
    let origin = plan.root_static_origin().expect("root occurrence exists");
    let seed_env = NativeSeedEnvironment::empty();
    let resource = 0x1020_3040_5060_7080_i64;
    let response_pointer = 0x1122_3344_5566_7788_i64;
    let response_len = 23_i64;

    let producer_plan = plan.clone();
    let (_producer_module, producer) = c2_compile_edge_with_arg(
        "c2_borrowed_payload_producer",
        &seed_env,
        producer_plan,
        move |compiler, builder, success| {
            let resource = builder.ins().iconst(types::I64, resource);
            let response_pointer =
                builder.ins().iconst(types::I64, response_pointer);
            let response_len = builder.ins().iconst(types::I64, response_len);
            let result = Lowered::HostResult {
                success,
                error: Box::new(Lowered::ResponseBytes {
                    pointer: response_pointer,
                    len: response_len,
                }),
                ok: Box::new(Lowered::ResourceToken { value: resource }),
                err_constructor: symbols.result_err.clone(),
                ok_constructor: symbols.result_ok.clone(),
            };
            Ok(compiler
                .transfer_into_carrier(builder, origin, &result)?
                .word)
        },
    );

    let resource_plan = plan.clone();
    let (_resource_module, read_resource) = c2_compile_edge_with_arg(
        "c2_resource_token_consumer",
        &seed_env,
        resource_plan,
        |compiler, builder, word| {
            let payload = compiler.emit_carrier_host_payload(
                builder,
                CarriedBoundaryWord { word },
            )?;
            compiler.emit_carrier_scalar(builder, payload)
        },
    );

    let (_response_module, read_response) = c2_compile_edge_with_arg(
        "c2_response_bytes_consumer",
        &seed_env,
        plan,
        |compiler, builder, word| {
            let payload = compiler.emit_carrier_host_payload(
                builder,
                CarriedBoundaryWord { word },
            )?;
            let pointer = compiler.emit_carrier_scalar(builder, payload)?;
            let len = compiler.emit_carrier_field(builder, payload, 0)?;
            let len = compiler.emit_carrier_scalar(builder, len)?;
            Ok(builder.ins().bxor(pointer, len))
        },
    );

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let ok_word = c2_run_edge_with_arg(producer, base, 1);
    assert_eq!(
        c2_run_edge_with_arg(read_resource, base, ok_word),
        resource,
        "the success arm must preserve and expose the full ResourceToken scalar"
    );

    let err_word = c2_run_edge_with_arg(producer, base, 0);
    assert_eq!(
        c2_run_edge_with_arg(read_response, base, err_word),
        response_pointer ^ response_len,
        "the error arm must preserve and expose ResponseBytes pointer and length"
    );
}

/// A zero-argument constructor occurrence, so the producer's supported surface
/// (`Constructor` with no children) carries the whole fixture.
fn ac_c7_ctor(name: &str) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: format!("ctor:fixture::C1::{name}"),
        args: Vec::new(),
    }
}

/// ⭐ **`D7` — a hand-built source aggregate owes the producer occurrence the
/// planner issued AT ITS OWN ORIGIN**, and takes that origin as an argument
/// rather than inheriting whatever coordinate its eventual transfer uses.
///
/// This rig stands in for the `Construct` lowering arm, so it owes exactly what
/// that arm resolves. ⛔ Leaving it absent is not a shortcut that happens to
/// work: it is the fail-closed hole the subclosure exists to close, and it read
/// as harmless only because these rigs transfer at the producer origin, where
/// the two coordinates coincide.
fn ac_c7_lowered_ctor(
    plan: &StaticTransitionPlan<'_>,
    origin: StaticOriginId,
    name: &str,
) -> Lowered {
    Lowered::Constructor {
        constructor: format!("ctor:fixture::C1::{name}"),
        synthesized_identity: None,
        occurrence: Some(
            plan.source_aggregate_occurrence(origin, PlannedAggregateShape::Constructor)
                .expect("a planned `Construct` has an ownership record at its own origin"),
        ),
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

    let field_origin = |position| {
        plan.child_static_origin(record_origin, position)
            .expect("a record field has a planned child origin")
    };
    // ⭐ `D7` — the field SCHEMA travels on the template exactly as the
    // producer occurrence does, resolved here for the same reason: this rig
    // hand-builds what the `Record` lowering arm would otherwise have built,
    // and a hand-built field with no planner-issued identity is a fail-closed
    // absence rather than a licence to name it at the transfer coordinate.
    let field_identity = |position: usize| {
        plan.record_field_identity(record_origin, position)
            .expect("a planned `Record` field has a planner-issued identity")
    };
    let lowered_fields = vec![
        LoweredRecordField {
            name: fields[0].0.to_string(),
            identity: Some(field_identity(0)),
            value: ac_c7_lowered_ctor(&plan, field_origin(0), fields[0].1),
        },
        LoweredRecordField {
            name: fields[1].0.to_string(),
            identity: Some(field_identity(1)),
            value: ac_c7_lowered_ctor(&plan, field_origin(1), fields[1].1),
        },
    ];
    // ⭐ `D7` — the record's PRODUCER occurrence, resolved here because this
    // rig hand-builds the template that the `Record` lowering arm would
    // otherwise have resolved it in. A hand-built aggregate with no producer
    // is a fail-closed absence, not a licence to fall back to the transfer
    // coordinate.
    let record_occurrence = plan
        .source_aggregate_occurrence(record_origin, PlannedAggregateShape::Record)
        .expect("the planned record has an ownership record at its own origin");
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        // ── PRODUCER: a compile-time record crosses the one-way seam ──────
        let record = Lowered::Record {
            occurrence: Some(record_occurrence),
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
            &[LoweringEnvironmentBinding::Value(LoweringOperand::Carried(word))],
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

/// ⭐ **The child's origin is derived from the parent's, not passed in.** The
/// preflight checks each child against the record the planner planned at THAT
/// POSITION, so a rig that took two independent origins could satisfy it with
/// two unrelated ones. Reading child 0 off the parent is the same derivation the
/// producer arm performs, which is what makes the agreement a property under
/// test rather than a coincidence the caller arranged.
fn ac_c7_lowered_wrap(
    plan: &StaticTransitionPlan<'_>,
    origin: StaticOriginId,
    outer: &str,
    inner: &str,
) -> Lowered {
    let inner_origin = plan
        .child_static_origin(origin, 0)
        .expect("the wrapper's only argument has a planned origin");
    Lowered::Constructor {
        constructor: format!("ctor:fixture::C1::{outer}"),
        synthesized_identity: None,
        occurrence: Some(
            plan.source_aggregate_occurrence(origin, PlannedAggregateShape::Constructor)
                .expect("the planned wrapper has an ownership record at its own origin"),
        ),
        args: vec![ac_c7_lowered_ctor(plan, inner_origin, inner)],
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
/// The fixture is
/// `Let { Call { || Wrap(Inner) }, Match Var(0) { Left x -> x, Right x ->
/// Sentinel } }` with `Wrap` supplied by the caller, so ONE helper produces all
/// three interesting outcomes: selecting the first case, selecting the second,
/// and reaching the closed default. The zero-argument lexical call makes the
/// fixture's source agree with the carrier result the focused JIT rig injects.
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
        value: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(ac_c7_wrap(scrutinee, inner)),
            }),
            args: Vec::new(),
        }),
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
    let plan = plan_static_transition_graph_with_symbols(
        &fixture,
        &BTreeMap::new(),
        &crate::NativeProcessSymbols::legacy_prelude(),
        AbiRootIngress::Value,
        true,
    )
    .expect("the functionized carrier fixture plans");
    let root = plan
        .root_static_origin()
        .expect("the functionized carrier fixture has a root occurrence");
    let producer_call_origin = plan
        .child_static_origin(root, 0)
        .expect("a `Let`'s value is child 0");
    let producer_closure_origin = plan
        .child_static_origin(producer_call_origin, 0)
        .expect("the producer call's callee is child 0");
    let scrutinee_origin = plan
        .child_static_origin(producer_closure_origin, 0)
        .expect("the producer closure's body is child 0");
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

    let lowered = ac_c7_lowered_wrap(&plan, scrutinee_origin, scrutinee, inner);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringEnvironmentBinding::Value(LoweringOperand::Carried(word))],
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

    let lowered = ac_c7_lowered_wrap(&plan, scrutinee_origin, scrutinee, inner);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringEnvironmentBinding::Value(LoweringOperand::Carried(word))],
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

    let lowered = ac_c7_lowered_wrap(&plan, scrutinee_origin, "Wrap", "Leaf");
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_try_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringEnvironmentBinding::Value(LoweringOperand::Carried(word))],
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

/// An ordinary `Match` over the bound CARRIED child, for the source-machine
/// seat.
///
/// The case body of [`ac_c4_recursive_edge`]'s recursive case is lowered through
/// `Lowering::lower_source_machine`, so a `Match` placed here arrives at
/// `SourceContinuation::MatchScrutinee` -- a DIFFERENT seat from the generic
/// `lower_expr` emitter that [`ac_c7_match_edge`] drives. That distinction is
/// the whole point: the generic seat classified a carried scrutinee by phase and
/// this one did not, so a carried value reaching it fell past every
/// `Lowered`-shape test onto `"scrutinee is not a constructor value"`.
fn ac1_source_machine_carried_match(cases: Vec<crate::RuntimeMatchCase>) -> RuntimeExpr {
    RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(1)),
        cases,
        default: ac_c7_trap(),
    }
}

fn ac1_unmatchable_case() -> crate::RuntimeMatchCase {
    crate::RuntimeMatchCase {
        constructor: "ctor:fixture::C1::Sentinel".to_string(),
        binders: 0,
        body: ac_c7_ctor("Sentinel"),
    }
}

/// `AC-1` -- an ordinary `Match` at the SOURCE-MACHINE seat CLASSIFIES A
/// CARRIED SCRUTINEE BY PHASE instead of asking it for a `Lowered` shape.
///
/// MEASURED: the fixture is refused by `Lowering::lower_source_carried_match`'s
/// own join acquisition, because this rig's enclosing continuation has no
/// planned scalar cut. Pre-repair the identical fixture was refused earlier and
/// elsewhere, by `Match: "scrutinee is not a constructor value"`.
///
/// CLAIMED: `SourceContinuation::MatchScrutinee` classifies a `LoweringOperand`
/// by PHASE before any arm asks for a `Lowered` shape, and dispatches a carried
/// scrutinee into the source-machine carried route.
///
/// THE GAP, stated plainly because this row asserts on a REFUSAL: "it errored"
/// is satisfied by erroring for any reason at all. Closed by the identity of the
/// error rather than its presence -- the refusal below is raised only INSIDE the
/// carried route, after the phase classification and the dispatch, so it cannot
/// be produced by the pre-repair code path, which never reached that route.
///
/// ⛔ What this row does NOT claim, and what is still OWED: it does not exercise
/// a nontrivial post-match continuation, a nested or inherited source join,
/// exact-once join consumption, distinct predecessor edges, or the carried
/// identity / arity / field-order negatives. They are not claimed here and are
/// not discharged anywhere else.
///
/// ⛔ AND THE OBVIOUS NEXT STEP IS MEASURED CLOSED. An earlier draft of this
/// paragraph said those controls "need a whole-process fixture with a planned
/// scalar cut", which reads as an instruction to go build one. A probe over the
/// whole `ken-runtime` lib suite (779 tests) measured otherwise:
///
///   * `SourceContinuation::MatchScrutinee` is reached **7 times total**, and
///     with a CARRIED operand **exactly once** -- this row.
///   * `lower_source_carried_match` is entered **exactly once** -- this row --
///     and refuses at join acquisition before emitting any selector.
///   * The whole-process capacity family (`run_capacity_fixture` and every
///     sibling in `effects.rs`) reaches the seat **zero** times. A closure
///     parameter bound to a `Construct` stays `Specialized(Lowered::Constructor)`
///     and is selected at COMPILE time, so the carried arm is never taken.
///
/// ⇒ Writing the owed controls is not a fixture-authoring task on the existing
/// rigs. It needs a producer that delivers a genuinely carried word into a
/// source-machine `Match` under a planned scalar cut, and no rig in this crate
/// does that today. Do not read the list above as buildable work.
///
/// Promise class: transition sentinel. It is pinned to the join-acquisition
/// refusal this rig currently raises; a rig that supplies a planned scalar cut
/// retires this row in favour of the real positive.
#[test]
fn ac1_source_machine_match_classifies_a_carried_scrutinee_by_phase() {
    let refused = ac_c4_recursive_edge(ac1_source_machine_carried_match(vec![
        ac1_unmatchable_case(),
        crate::RuntimeMatchCase {
            constructor: "ctor:fixture::C1::Leaf".to_string(),
            binders: 0,
            body: ac_c7_ctor("Sentinel"),
        },
    ]))
    .expect_err("this rig's continuation has no planned scalar cut");
    let CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct, reason, ..
    }) = &refused
    else {
        panic!("the refusal is an unsupported-lowering: got {refused:?}");
    };
    assert_ne!(
        *reason, "scrutinee is not a constructor value",
        "DISCRIMINATOR: this is the pre-repair refusal. Reaching it means the seat \
         asked a carried value for a compile-time template again"
    );
    assert_eq!(
        (*construct, reason.as_str()),
        (
            "NativeJoinPlanV1",
            "active checked continuation has no planned scalar cut before its outer suffix"
        ),
        "AC-1: the refusal must come from INSIDE the source-machine carried route, \
         which is reachable only once the seat has classified the scrutinee's \
         phase and dispatched into it"
    );
}

/// A two-argument constructor, so the recursive position can be declared
/// somewhere **other than 0**.
fn ac_c4_wrap2(outer: &str, first: &str, second: &str) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: format!("ctor:fixture::C1::{outer}"),
        args: vec![ac_c7_ctor(first), ac_c7_ctor(second)],
    }
}

/// The two-argument sibling of [`ac_c7_lowered_wrap`], with each child's origin
/// derived from its own position for the reason stated there.
fn ac_c4_lowered_wrap2(
    plan: &StaticTransitionPlan<'_>,
    origin: StaticOriginId,
    outer: &str,
    first: &str,
    second: &str,
) -> Lowered {
    let child_origin = |position| {
        plan.child_static_origin(origin, position)
            .expect("the wrapper's argument has a planned origin")
    };
    Lowered::Constructor {
        constructor: format!("ctor:fixture::C1::{outer}"),
        synthesized_identity: None,
        occurrence: Some(
            plan.source_aggregate_occurrence(origin, PlannedAggregateShape::Constructor)
                .expect("the planned wrapper has an ownership record at its own origin"),
        ),
        args: vec![
            ac_c7_lowered_ctor(plan, child_origin(0), first),
            ac_c7_lowered_ctor(plan, child_origin(1), second),
        ],
    }
}

/// Drive a carried recursive-position elimination whose recursive position is
/// **1 of 2**, capturing the `PX8J` producer trace alongside the eliminated
/// value.
///
/// ⭐⭐ **Position 1, not 0, and that is the whole design.** `sibling_position: 0`
/// is what a *positionally defaulted* implementation produces for free — an
/// ownership claim measured on a fixture whose right answer is also the default
/// cannot fail. ⚠ This is the `AC-C5` hazard from `AC-C7` in a new dress: that
/// control stayed green under its mutation precisely because its field sat at
/// position 0.
fn ac_c4_ownership_edge() -> (i64, u64, u64, Vec<Px8jSourceTraceEvent>) {
    // `[IH] ++ [child0, child1] ++ frame env` -- so `Var(1)` is `Alpha`.
    ac_c4_ownership_edge_with_case_body(RuntimeExpr::Var(1))
}

/// The same position-1 recursive edge, with the recursive case's body supplied
/// by the caller so a control can read a **chosen** case binder.
///
/// ⭐ The parameter exists because `Var(1)` alone cannot see a defect in the
/// projection loop's own field index: it reads `child0`, and a loop that
/// projected field 0 for *every* binder would still answer `Alpha`. Reading
/// `Var(2)` is what makes that class visible.
fn ac_c4_ownership_edge_with_case_body(
    case_body: RuntimeExpr,
) -> (i64, u64, u64, Vec<Px8jSourceTraceEvent>) {
    let fixture = RuntimeExpr::Let {
        value: Box::new(ac_c4_wrap2("Wrap2", "Alpha", "Leaf")),
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::C1::Wrap2".to_string(),
                    argument_binders: 2,
                    // ⛔ The SECOND argument is the recursive one.
                    recursive_positions: vec![1],
                    // `[IH] ++ [child0, child1] ++ frame env`. ⭐ A layout
                    // discriminator: without the IH at index 0, `Var(1)` reads
                    // `Leaf` instead of `Alpha`.
                    body: case_body,
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
    let alpha_identity = identity_at(
        plan.child_static_origin(scrutinee_origin, 0)
            .expect("the wrapper's first argument has a planned origin"),
    );
    let leaf_identity = identity_at(
        plan.child_static_origin(scrutinee_origin, 1)
            .expect("the wrapper's second argument has a planned origin"),
    );

    let lowered = ac_c4_lowered_wrap2(&plan, scrutinee_origin, "Wrap2", "Alpha", "Leaf");
    let seed_env = NativeSeedEnvironment::empty();

    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().clear());
        }
    }
    PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().clear());
    let _reset = Reset;

    let (_module, code) = ac_c7_try_compile_edge(&seed_env, plan, move |compiler, builder| {
        let word = compiler.transfer_into_carrier(builder, scrutinee_origin, &lowered)?;
        let eliminated = compiler.lower_expr(
            builder,
            SourceOccurrence {
                expr: match_expr.as_ref(),
                static_origin: match_origin,
            },
            &[LoweringEnvironmentBinding::Value(LoweringOperand::Carried(word))],
        )?;
        let LoweringOperand::Carried(selected) = eliminated else {
            panic!("a carried `ComputationalMatch` merges in the carrier lane")
        };
        compiler.emit_carrier_tag(builder, selected)
    })
    .expect("the position-1 recursive case lowers");
    let trace = PX8J_SOURCE_TRACE.with(|trace| trace.borrow().clone());

    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let observed = ac_c7_run(code, base);
    (observed, alpha_identity, leaf_identity, trace)
}

/// ⭐⭐ **`AC-C4` CONTROL 3 — the recursive position's OWNERSHIP comes from the
/// frame, not from the carried word or from a positional default.**
///
/// **MEASURED:** eliminating `Wrap2(Alpha, Leaf)` through a case declaring
/// `recursive_positions: [1]` mints exactly one induction hypothesis, whose
/// recorded `sibling_position` is **1**, under a producer origin that matches
/// the mint's; and the eliminated value is `Alpha`.
/// **CLAIMED:** the ruling's clause 5 — static-origin, slot-template, activation
/// and invocation ownership all stay on the existing recursor metadata, and ⛔
/// none of it is derived from the carried word.
/// **THE GAP:** a trace assertion says an IH was *recorded*, not that the right
/// one was built.
///
/// ⛔⛔ **AN EARLIER REVISION OF THIS COMMENT CLAIMED THAT GAP WAS CLOSED HERE,
/// AND IT WAS NOT.** It read: *"closed by pairing it with the value — the trace
/// fixes which position owns the hypothesis and `Var(1)` fixes where the
/// children sit, and no single wrong answer satisfies both."* ⚠ Both halves of
/// that pairing observe the **metadata** edge. `runtime-qa` defeated it on
/// `b8d2922f` with a compile-preserving substitution of the residual's operand
/// — `children[position]` → `children[0]` — which leaves `sibling_position`,
/// the producer origin and the `Var(1)` route all intact, and this control
/// stayed **green**.
///
/// ⇒ ⭐ **This control measures OWNERSHIP and nothing else.** The residual's
/// *content* — `§2g-i`'s "passes its projected `Carried(child)` directly" — is
/// a different edge, and it is measured by
/// [`c1_d3_ac_c4_the_residual_holds_the_declared_positions_projected_child`].
///
/// ⚠ Promise class: **durable invariant**.
#[test]
fn c1_d3_ac_c4_the_recursive_positions_ownership_comes_from_the_frame() {
    let (observed, alpha, leaf, trace) = ac_c4_ownership_edge();
    assert_ne!(alpha, leaf, "NON-VACUITY: the two children must be distinguishable");

    let mints: Vec<_> = trace
        .iter()
        .filter_map(|event| match event {
            Px8jSourceTraceEvent::Mint {
                origin, siblings, ..
            } => Some((*origin, *siblings)),
            _ => None,
        })
        .collect();
    assert_eq!(
        mints.len(),
        1,
        "exactly one recursive producer is minted for one recursive position: \
         {trace:#?}"
    );
    let (mint_origin, siblings) = mints[0];
    assert_eq!(siblings, 1, "the case declares one recursive position");

    let carriers: Vec<_> = trace
        .iter()
        .filter_map(|event| match event {
            Px8jSourceTraceEvent::Carrier {
                origin,
                sibling_position,
                ..
            } => Some((*origin, *sibling_position)),
            _ => None,
        })
        .collect();
    assert_eq!(
        carriers,
        vec![(mint_origin, 1)],
        "DISCRIMINATOR: the hypothesis must be owned by the DECLARED recursive \
         position 1 under the minting producer's own origin. ⛔ `0` here is the \
         positional default, which is exactly why this fixture declares its \
         recursive position somewhere else: {trace:#?}"
    );

    assert_eq!(
        observed as u64, alpha,
        "the value route must stay intact while ownership is measured: with the \
         hypothesis at index 0, `Var(1)` is the FIRST child. Reading `Leaf` \
         ({leaf}) means the case environment lost its hypothesis; got {observed}"
    );
}

/// ⭐⭐ **`AC-C4` CONTROL 6 — the induction hypothesis's residual holds the
/// child projected at the case's DECLARED recursive position.**
///
/// **MEASURED:** eliminating `Wrap2(Alpha, Leaf)` through a case declaring
/// `recursive_positions: [1]`, the boundary word recorded inside the minted
/// hypothesis's residual is **identical to the word the projection loop
/// produced for field 1**, and the two fields produced different words.
/// **CLAIMED:** `§2g-i` clause 1 — the carried `ComputationalMatch` arm passes
/// its projected `Carried(child)` **directly** into the licensed residual edge.
/// **THE GAP:** identity of SSA words shows the residual holds *that
/// projection*, not that the projection itself reads the right **memory**. ⛔
/// This control's oracle is the projection loop's own record, so it is blind by
/// construction to a defect in the loop's field index — a loop projecting field
/// `0` for every binder still records two distinct words and still satisfies
/// "the residual holds the word recorded at position 1."
///
/// ⚠⚠ **I first wrote here that the second half was "measured by the `AC-C7`
/// field-projection controls." I then mutated the loop to check, and it was
/// FALSE:** `emit_carrier_field(builder, scrutinee, position)` →
/// `..., 0)` was green across the entire `ken-runtime` suite. ⇒ That half is
/// closed by
/// [`c1_d3_ac_c4_each_case_binder_reads_its_own_constructor_field`], written
/// for this gap, ⛔ not by a neighbour that happened to exist.
///
/// ## ⭐ Why this is NOT the positionally-derived assertion I flagged as the risk
///
/// The expected index is the literal `DECLARED_RECURSIVE_POSITION`, ⛔ **not**
/// read from the production path's `position` variable. The distinction is
/// where the number comes from, and it is the whole difference between a
/// control and a tautology:
///
/// - ⛔ **circular:** expected index sourced from the same production variable
///   the mutation perturbs ⇒ expected moves *with* production, stays green.
/// - ✅ **sound (this control):** expected index is the fixture's own
///   declaration, chosen by the fixture author ⇒ under
///   `children[position]` → `children[0]` the expectation stays at field 1
///   while production moves to field 0, and this **reds**.
///
/// The oracle it compares against — `CarrierFieldProjection` — is written by
/// the projection loop keyed on that loop's counter, *before* any selection
/// among the children occurs. ⇒ It records ground truth about which field
/// yielded which word and cannot move with a selection defect.
///
/// ⚠ Promise class: **durable invariant**. Any future case shape keeps it green
/// so long as the residual holds the declared position's child; it reddens
/// exactly when that stops being true.
#[test]
fn c1_d3_ac_c4_the_residual_holds_the_declared_positions_projected_child() {
    // ⭐ THE FIXTURE'S OWN DECLARATION, restated on this test's authority.
    // `ac_c4_ownership_edge` builds `recursive_positions: vec![1]` over two
    // binders; these literals are the independent half of the comparison.
    const DECLARED_RECURSIVE_POSITION: usize = 1;
    const ARGUMENT_BINDERS: usize = 2;

    let (_observed, _alpha, _leaf, trace) = ac_c4_ownership_edge();

    let projections: Vec<(usize, cranelift_codegen::ir::Value)> = trace
        .iter()
        .filter_map(|event| match event {
            Px8jSourceTraceEvent::CarrierFieldProjection { position, word, .. } => {
                Some((*position, *word))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        projections.iter().map(|(position, _)| *position).collect::<Vec<_>>(),
        (0..ARGUMENT_BINDERS).collect::<Vec<_>>(),
        "the recursive case projects exactly its {ARGUMENT_BINDERS} binders, in \
         order, and the `Leaf` case projects none: {trace:#?}"
    );
    assert_ne!(
        projections[0].1, projections[1].1,
        "NON-VACUITY: the two projected fields must be DISTINGUISHABLE words, \
         or 'the residual holds field 1' is satisfied by field 0 as well and \
         this control proves nothing: {trace:#?}"
    );

    let expected = projections
        .iter()
        .find(|(position, _)| *position == DECLARED_RECURSIVE_POSITION)
        .map(|(_, word)| *word)
        .expect("the declared recursive position is among the projected binders");

    let residuals: Vec<_> = trace
        .iter()
        .filter_map(|event| match event {
            Px8jSourceTraceEvent::Carrier { residual, .. } => Some(*residual),
            _ => None,
        })
        .collect();
    assert_eq!(
        residuals,
        vec![Px8jResidualPhase::Carried(expected)],
        "DISCRIMINATOR: the one minted hypothesis must hold, IN THE CARRIED \
         PHASE, the exact word field {DECLARED_RECURSIVE_POSITION} projected. ⛔ \
         `Carried({:?})` here is field 0 — the positional default, and the \
         compile-preserving evasion this control exists to redden. ⛔ \
         `Specialized` here means the residual was wrapped or templated rather \
         than passed directly, which `§2g-i` forbids: {trace:#?}",
        projections[0].1
    );
}

/// ⭐⭐ **`AC-C4` CONTROL 7 — each case binder reads ITS OWN constructor field.**
///
/// **MEASURED:** eliminating `Wrap2(Alpha, Leaf)` through the two-binder
/// recursive case, a body of `Var(1)` evaluates to **`Alpha`** and a body of
/// `Var(2)` evaluates to **`Leaf`** — the complete positional map of the case
/// environment's child region, run end-to-end through emitted code.
/// **CLAIMED:** `§2g` — the carried projection loop projects field `p` for
/// binder `p`, so `[IH] ++ [child0, child1] ++ frame env` means what it says.
/// **THE GAP:** two binders witness a two-field constructor exactly; a wider
/// arity could permute fields `≥2` undetected. ⛔ Recorded, not claimed away.
///
/// ## ⚠ This control exists because I falsified my OWN coverage claim
///
/// `c1_d3_ac_c4_the_residual_holds_the_declared_positions_projected_child`
/// closes *which child* the residual selects, but its oracle is the projection
/// loop's own record — so it cannot see the loop projecting the **wrong field**
/// for every binder. I asserted that case was covered elsewhere, then mutated
/// `emit_carrier_field(builder, scrutinee, position)` → `..., 0)` and found it
/// **green across all 485 + 26 + 14 tests**.
///
/// ⭐ **`Var(1)` alone is structurally incapable of catching it**: it reads
/// `child0`, whose field index *is* `0`, so the mutation's answer and the
/// correct answer coincide. ⇒ **`Var(2)` is the load-bearing half of this
/// control** — it is the only assertion here that the mutation moves.
///
/// ⚠ Promise class: **durable invariant**.
#[test]
fn c1_d3_ac_c4_each_case_binder_reads_its_own_constructor_field() {
    let (first, alpha, leaf, _trace) =
        ac_c4_ownership_edge_with_case_body(RuntimeExpr::Var(1));
    let (second, _alpha, _leaf, _trace) =
        ac_c4_ownership_edge_with_case_body(RuntimeExpr::Var(2));
    assert_ne!(
        alpha, leaf,
        "NON-VACUITY: the two children must be distinguishable identities"
    );

    assert_eq!(
        first as u64, alpha,
        "binder 0 (`Var(1)`, after the hypothesis at index 0) must read the \
         constructor's FIRST field: expected Alpha ({alpha}), got {first}"
    );
    assert_eq!(
        second as u64, leaf,
        "DISCRIMINATOR: binder 1 (`Var(2)`) must read the constructor's SECOND \
         field. ⛔ Reading Alpha ({alpha}) here means the projection loop \
         projected field 0 for every binder — the positional default, invisible \
         to `Var(1)` because field 0 is its right answer too. Got {second}, \
         expected Leaf ({leaf})"
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
    // ⭐ `D7` — as in the admissibility-ordering row: both graphs carry the
    // producer occurrence, so the refusal each one earns is the one the row is
    // about rather than a rig artifact.
    let wrap_occurrence = plan
        .source_aggregate_occurrence(construct_origin, PlannedAggregateShape::Constructor)
        .expect("the planned `Construct` has an ownership record at its own origin");
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    bind_bare_test_trap_lane(&mut compiler, &mut builder);

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
            synthesized_identity: None,
            occurrence: Some(wrap_occurrence),
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
        synthesized_identity: None,
        occurrence: Some(wrap_occurrence),
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

// ─── `RT-FNSPLIT-B2F` `D9` — THE MAGNITUDE DISPATCH ───────────────────────
//
// ⭐⭐ **One compiled body, two runtime payloads, both arms.** The claim under
// test is `AC-2`: the choice between the immediate field and the spilled handle
// is made by *emitted code from the value*, ⛔ never by a JIT-time inspection
// picking a layout. ⇒ Two separate compilations, each with its own constant,
// cannot establish that — a body that specialized on the constant would produce
// the same two answers. Every row below therefore drives **one** compiled
// function with the payload as a **parameter**.

/// `(arena, payload) -> boundary word` — the dispatch, compiled once.
///
/// ⚠ The `Lowered::Int` is built over the function's own **block parameter**,
/// and its `NativeIntV1` marker is registered the way `lower_dynamic_small_int`
/// registers one in production. ⛔ `known` is `None`: a `Some` here would hand
/// the producer a compile-time magnitude and is exactly the input this rig
/// exists to withhold.
fn b2f_d9_dispatch(payloads: &[i64]) -> Vec<crate::boundary_value::BoundaryWord> {
    let fixture = ac_c7_ctor("Alpha");
    let (plan, root) = planned_root_occurrence(&fixture);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_try_compile_edge_with_operands(
        &seed_env,
        plan,
        1,
        |compiler, builder, operands| {
            let payload = operands[0];
            let marker = builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
            compiler
                .function_local
                .native_int_tags
                .insert(payload, marker);
            let value = Lowered::Int {
                value: payload,
                known: None,
            };
            Ok(compiler.transfer_into_carrier(builder, root, &value)?.word)
        },
    )
    .expect("the magnitude dispatch emits");

    let run: extern "C" fn(*const u64, i64) -> i64 = unsafe { std::mem::transmute(code) };
    payloads
        .iter()
        .map(|payload| {
            // ⚠ A fresh store and arena per payload: the spill ALLOCATES, and
            // sharing one arena would let the second row's answer depend on the
            // first row's residency.
            let mut store = crate::boundary_value::BoundaryValueStore::new();
            let (_arena, base) = ac_c7_bind_arena(&mut store);
            let word = crate::boundary_value::BoundaryWord(run(base, *payload) as u64);
            // The node's own recorded content, read from the persistent image
            // the emitted code wrote into.
            if word.tag() == Some(BoundaryTag::PersistentGround) {
                let image = store.image();
                assert_eq!(
                    image.0.node_field(word.payload(), crate::boundary_value::NODE_CLASS),
                    Some(BoundaryClass::Int as u64),
                    "the spill arm must allocate the class the disposition \
                     declares in `spill: Some(_)`"
                );
                assert_eq!(
                    image
                        .0
                        .node_field(word.payload(), crate::boundary_value::NODE_PAYLOAD),
                    Some(*payload as u64),
                    "⛔ the spill must carry the magnitude WORD UNTRUNCATED — \
                     that is the entire reason the arm exists"
                );
                assert_eq!(
                    image
                        .0
                        .node_field(word.payload(), crate::boundary_value::NODE_EXTENT),
                    Some(crate::NATIVE_INT_SMALL_TAG_V1),
                    "the spill must record HOW the word is to be read"
                );
            }
            word
        })
        .collect()
}

/// ⭐ **`D9` ROW 1 — a value inside the immediate field takes the immediate
/// arm.**
///
/// **MEASURED:** JIT-compiled emitted code, handed `BOUNDARY_IMMEDIATE_INT_MAX`
/// at run time, returns a word tagged [`BoundaryTag::ImmediateInt`] whose signed
/// payload is that value.
/// **CLAIMED:** the dispatch's `BOUNDARY_OK` arm uses the word `make_immediate`
/// wrote, rather than allocating.
/// **THE GAP:** ⚠ *"it is an immediate"* alone is satisfiable by a body that is
/// **always** an immediate — which is the pre-dispatch defect, truncation and
/// all. ⇒ Closed only by row 2, on the same compiled body.
///
/// ⚠ Promise class: **durable invariant.** The literal is the ABI's own field
/// limit rather than a captured number, so widening the payload field moves the
/// fixture with the contract instead of reddening it.
#[test]
fn b2f_d9_a_value_inside_the_field_takes_the_immediate_arm() {
    let max = crate::boundary_value::BOUNDARY_IMMEDIATE_INT_MAX;
    assert!(
        crate::boundary_value::BoundaryWord::int_fits_immediate(max),
        "NON-VACUITY: the fixture must actually be inside the field, or this row \
         is testing the other arm"
    );
    let [word]: [_; 1] = b2f_d9_dispatch(&[max]).try_into().expect("one payload");
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::ImmediateInt),
        "`D9`: a value the field can hold crosses as an immediate word"
    );
    assert_eq!(
        word.signed_payload(),
        max,
        "`D9`: and it carries the value, not a truncation of it"
    );
}

/// ⭐ **`D9` ROW 2 — a value past the immediate field takes the SPILL arm, and
/// the spill is a handle that carries the magnitude.**
///
/// **MEASURED:** the same emitted body, handed `BOUNDARY_IMMEDIATE_INT_MAX + 1`,
/// returns a [`BoundaryTag::PersistentGround`] handle whose node records class
/// `Int`, the exact magnitude word, and the `Small` marker (asserted inside
/// [`b2f_d9_dispatch`]).
/// **CLAIMED:** `make_immediate`'s `BOUNDARY_ERR_BOUNDS` status is what selects
/// the spill, and the spill preserves the value.
/// **THE GAP:** ⛔ this row does not show the producer READS the status rather
/// than re-deriving the predicate — a hand-written shift-and-compare would
/// answer identically on every value. That residual is **review-caught, not
/// mechanically detected**, and it is recorded as such on
/// `Lowering::emit_carrier_spillable_immediate`; ⚠ this test passing is not
/// evidence about it.
///
/// ⚠ Promise class: **durable invariant** — `MAX + 1` is derived from the ABI's
/// own limit, so it tracks the field rather than freezing a magnitude.
#[test]
fn b2f_d9_a_value_past_the_field_takes_the_spill_arm() {
    let over = crate::boundary_value::BOUNDARY_IMMEDIATE_INT_MAX + 1;
    assert!(
        !crate::boundary_value::BoundaryWord::int_fits_immediate(over),
        "NON-VACUITY: the fixture must actually overflow the field"
    );
    let [word]: [_; 1] = b2f_d9_dispatch(&[over]).try_into().expect("one payload");
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::PersistentGround),
        "`D9`: a value the field cannot hold crosses as a HANDLE — the tag the \
         ABI's own `ImmediateInt` doc names as the overflow representation"
    );
}

/// ⭐⭐ **`D9` POSITIVE CONTROL — the spill arm is genuinely TAKEN, by one
/// compiled body, at run time.**
///
/// ⛔ **This is the row the other two cannot replace, and the reason is the
/// defect it is designed to catch.** Rows 1 and 2 each compile their own
/// function, so both would still pass if the producer inspected a JIT-time
/// magnitude and emitted a *different body* for each — which is precisely the
/// compile-time specialization `AC-2` forbids. Here **one** compiled function is
/// driven with both payloads, so the only thing that can differ between the two
/// answers is the run-time value.
///
/// ⚠ The pair is **adjacent** — `MAX` against `MAX + 1` — so nothing but the
/// partition itself can separate them. A body that always took one arm returns
/// two words with the same tag.
///
/// **MEASURED:** one function, two payloads one apart, two different tags.
/// **CLAIMED:** the arm is selected from the payload at run time.
/// **THE GAP:** that the selecting quantity is `make_immediate`'s status — see
/// row 2's residual.
#[test]
fn b2f_d9_one_compiled_body_takes_both_arms_at_runtime() {
    let max = crate::boundary_value::BOUNDARY_IMMEDIATE_INT_MAX;
    let words = b2f_d9_dispatch(&[max, max + 1]);
    assert_ne!(
        words[0].tag(),
        words[1].tag(),
        "POSITIVE CONTROL: one compiled body handed two ADJACENT payloads must \
         take DIFFERENT arms. Equal tags mean the dispatch is not reading the \
         value at all"
    );
    assert_eq!(
        words[0].tag(),
        Some(BoundaryTag::ImmediateInt),
        "and the direction must be the one the field dictates"
    );
    assert_eq!(
        words[1].tag(),
        Some(BoundaryTag::PersistentGround),
        "⛔ the larger value is the one that spills"
    );
}

/// ⭐⭐ **`D9` — WHY THE THIRD OUTCOME IS NOT PINNED BY A FIXTURE, checked
/// rather than asserted.**
///
/// ⛔ **A mutation deleting `require_i64(status, BOUNDARY_ERR_BOUNDS)` from the
/// dispatch leaves all three rows above GREEN, and that is measured.** The
/// honest reading is not *"the controls are weak"* — it is that the arm is
/// **structurally unreachable through this producer**, and the reason is a
/// relation between two authority tables that nothing else states:
///
/// - `ken_boundary_make_immediate_local` refuses with `BOUNDARY_ERR_SHAPE`
///   in exactly two situations: a **handle** tag, and a payload outside a
///   **`Bit`** domain. Every other refusal is `BOUNDARY_ERR_BOUNDS`.
/// - The dispatch is only ever reached with a tag from a
///   `RepresentedImmediate { spill: Some(_) }` disposition.
///
/// ⇒ If no spillable variant's tag carries the `Bit` domain, `make_immediate`
/// on this path can answer only `OK` or `ERR_BOUNDS`, and no fixture can drive
/// the third arm without first changing one of those tables.
///
/// **MEASURED:** every `LoweredVariant` whose disposition declares a spill has
/// an immediate tag present in `BOUNDARY_IMMEDIATE_DOMAIN` with a domain other
/// than `Bit`.
/// **CLAIMED:** the dispatch's *"anything else → fail closed"* arm is a backstop
/// against a future table change, ⛔ not dead code and ⛔ not a live branch some
/// test forgot to cover.
/// **THE GAP:** ⚠ this pins the **premise**, not the backstop. If the premise
/// is ever broken — a spillable tag given the `Bit` domain, or a handle tag
/// reaching the call — this test reddens and the branch becomes reachable, at
/// which point it needs a fixture. ⇒ That is the intended coupling: ⛔ the
/// backstop must never be removed on the grounds that "no test covers it."
///
/// ⚠ Promise class: **durable invariant.** It quantifies over
/// `LoweredVariant::ALL` and reads both tables, so a new spillable variant is
/// covered without editing this test — and a new one with a `Bit` domain is
/// exactly the change that should stop the world.
#[test]
fn b2f_d9_no_spillable_tag_can_make_the_immediate_producer_answer_shape() {
    let mut spillable = 0usize;
    for variant in LoweredVariant::ALL {
        let BoundaryDisposition::RepresentedImmediate {
            tag,
            spill: Some(_),
        } = variant.boundary_disposition()
        else {
            continue;
        };
        spillable += 1;
        let domain = crate::boundary_value::BOUNDARY_IMMEDIATE_DOMAIN
            .iter()
            .find(|(candidate, _)| *candidate == tag)
            .map(|(_, domain)| *domain);
        // ⛔ Two ways the premise can break, and they are different failures.
        assert!(
            domain.is_some(),
            "⛔ {variant:?}'s immediate tag {tag:?} is absent from \
             `BOUNDARY_IMMEDIATE_DOMAIN`, so `make_immediate` refuses it as a \
             HANDLE tag with ERR_SHAPE — the third outcome, reachable"
        );
        assert_ne!(
            domain,
            Some(crate::boundary_value::BoundaryImmediateDomain::Bit),
            "⛔ {variant:?} declares a spill and carries the `Bit` domain, so \
             `make_immediate` can now answer ERR_SHAPE on the dispatch path. \
             The fail-closed arm is REACHABLE and needs a fixture"
        );
    }
    assert!(
        spillable > 0,
        "NON-VACUITY: a loop over zero spillable variants asserts nothing, and \
         would stay green if the disposition table lost its `spill` arm entirely"
    );
}

// ─── `RT-FNSPLIT-B2F` `D9` — THE BYTE-BODIED HANDLE PRODUCER ──────────────

/// Transfer one byte-bodied literal through the real emitted carrier graph and
/// report `(word, node class, node content)`.
///
/// ⚠ The `Lowered` is handed in by the caller so that **one** helper drives both
/// classes: `String` and `Bytes` differ by the class the disposition supplies,
/// and that class is the axis `store_bytes_len` and `store_byte` guard on. ⛔ A
/// `Bytes`-only fixture leaves `String`'s guard arm unreached — the defect
/// `boundary_value_clif`'s own history records.
fn b2f_d9_bytes_edge(literal: Lowered) -> (crate::boundary_value::BoundaryWord, Option<u64>, Vec<u8>) {
    let fixture = ac_c7_ctor("Alpha");
    let (plan, root) = planned_root_occurrence(&fixture);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_compile_edge(&seed_env, plan, move |compiler, builder| {
        Ok(compiler.transfer_into_carrier(builder, root, &literal)?.word)
    });
    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let word = crate::boundary_value::BoundaryWord(ac_c7_run(code, base) as u64);
    let class = store
        .image()
        .0
        .node_field(word.payload(), crate::boundary_value::NODE_CLASS);
    let content = store
        .image()
        .0
        .node_data(word.payload())
        .map(<[u8]>::to_vec)
        .unwrap_or_default();
    (word, class, content)
}

/// ⭐ **`D9` — a `Bytes` literal crosses as a handle carrying its content.**
///
/// **MEASURED:** JIT-compiled emitted code claims a span of the literal's length
/// in the node's own region and writes every byte; the persistent image reads
/// back the exact content.
/// **CLAIMED:** the byte-bodied producer arm emits the claim-then-fill protocol.
/// **THE GAP:** ⚠ the content is a **compile-time literal**. ⛔ This says nothing
/// about a runtime-computed byte body — no `Lowered` variant carries one today,
/// and the arm must not be read as covering the class in general.
///
/// ⚠ Promise class: **durable invariant** — it asserts the round trip of a
/// fixture it owns, not a frozen node index or length.
#[test]
fn b2f_d9_a_bytes_literal_crosses_with_its_content() {
    // ⚠ Deliberately NOT ASCII-only and not a palindrome: a producer that wrote
    // the length as content, or filled the span in reverse, must be visible.
    let literal: Vec<u8> = vec![0x00, 0x7f, 0x80, 0xff, 0x01];
    let (word, class, content) = b2f_d9_bytes_edge(Lowered::Bytes(literal.clone()));
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::PersistentGround),
        "`D9`: a byte-bodied literal crosses as the handle its disposition declares"
    );
    assert_eq!(
        class,
        Some(BoundaryClass::Bytes as u64),
        "`D9`: the class comes from the sole disposition authority"
    );
    assert_eq!(
        content, literal,
        "`D9`: ⛔ the whole content, in order — a claim-then-fill that stopped \
         early, reversed, or wrote the length would differ here"
    );
}

/// ⭐⭐ **`D9` — the SAME emitter drives the `String` class, and that is the
/// discriminating row.**
///
/// ⛔ **Why this is not a duplicate of the `Bytes` row.** The two arms share
/// every line of the producer except the class the disposition hands it — and
/// the class is precisely what `store_bytes_len` and `store_byte` guard on. ⇒ A
/// guard narrowed to `Bytes` alone would leave the `Bytes` row green and this
/// one red, which is the whole reason both exist.
///
/// **MEASURED:** the identical emitter, given a `String`, produces a node whose
/// class is `String` and whose content is the literal's UTF-8 bytes.
/// **CLAIMED:** the byte-bodied arm is reached for both classes, not one.
/// **THE GAP:** ⚠ same literal-content caveat as the row above.
///
/// ⚠ Promise class: **durable invariant.**
#[test]
fn b2f_d9_the_same_emitter_builds_the_string_class() {
    // ⚠ Multi-byte on purpose: a producer writing `char`s rather than bytes, or
    // truncating to ASCII, differs here and agrees on a plain-ASCII fixture.
    let text = "kΩ→";
    let (word, class, content) = b2f_d9_bytes_edge(Lowered::String(text.to_string()));
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::PersistentGround),
        "`D9`: a `String` crosses as the handle its disposition declares"
    );
    assert_eq!(
        class,
        Some(BoundaryClass::String as u64),
        "⛔ the `String` CLASS — not `Bytes`. This is the axis the two arms do \
         NOT share, and the only thing this row adds over the `Bytes` row"
    );
    assert_eq!(
        content,
        text.as_bytes(),
        "`D9`: the content is the literal's UTF-8 bytes, all {} of them",
        text.len()
    );
    assert_ne!(
        content.len(),
        text.chars().count(),
        "NON-VACUITY: the fixture must be multi-byte, or `bytes` and `chars` \
         agree and the length assertion above discriminates nothing"
    );
}

// ─── `RT-FNSPLIT-B2F` `D9` — THE REGION-LIMBED (`Big`) `Int` PRODUCER ─────
//
// ⛔⛔ **Why a synthetic `(Big, payload)` pair would not do.** A `Big` payload is
// a **slot identity** in the invocation's native arena, and slots are small
// integers. ⇒ Handing `make_immediate` a low slot answers `BOUNDARY_OK` and
// encodes the integer `1` — the silent-corruption path. A fixture that invented
// a large payload would take the bounds edge and never exercise it. ⭐ So the
// pair here is minted by **`ken_native_int_intern_local` itself**, from limbs
// supplied at run time, exactly as production mints one.

/// A bound invocation whose boundary arena also names a native-`Int` arena and
/// reserves limb capacity in the persistent region.
///
/// ⚠ Both the `NativeIntArenaV1` and the store must outlive the call: the base
/// pointer names their tables, and the binding is published before the pointer
/// is taken because growing a table afterwards would move it.
fn b2f_d9_bind_wide_arena(
    store: &mut crate::boundary_value::BoundaryValueStore,
    native: &crate::native_int::NativeIntArenaV1,
) -> (crate::boundary_value::BoundaryArenaV1, *mut u64) {
    store.reserve_persistent(64, 256, 512, 64);
    let persistent = store.publish_persistent();
    let mut arena = crate::boundary_value::BoundaryArenaBuilder::new().finish();
    arena.reserve(64, 256, 512, 64);
    arena.bind_persistent(Some(persistent as *const u64));
    arena.bind_native_int(Some(native as *const _ as *const u64));
    let base = arena.publish();
    (arena, base)
}

/// `(arena, limb0, limb1) -> boundary word` — intern a native `Int` from
/// **run-time** limbs, then transfer it across the producer.
///
/// ⭐⭐ **One compiled body, and the marker is a RUNTIME value.** `intern` trims
/// leading zero limbs, so `(x, 0)` comes back `Small` and `(x, 1)` comes back
/// `Big` from the *same* call. ⇒ The marker partition is exercised as a run-time
/// branch, ⛔ not as two compilations that could each have specialized.
#[allow(clippy::type_complexity)]
fn b2f_d9_wide_int(
    limbs: [u64; 2],
) -> (
    crate::boundary_value::BoundaryWord,
    Vec<u64>,
    Option<u64>,
    Option<u64>,
    Option<u64>,
) {
    let fixture = ac_c7_ctor("Alpha");
    let (plan, root) = planned_root_occurrence(&fixture);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_try_compile_edge_with_operands(
        &seed_env,
        plan,
        2,
        |compiler, builder, operands| {
            let arena = compiler
                .function_local
                .boundary_arena
                .expect("the rig binds a boundary arena");
            let pointer_type = builder.func.dfg.value_type(arena);
            let native_arena = builder.ins().load(
                pointer_type,
                MemFlags::trusted(),
                arena,
                crate::boundary_value::ARENA_NATIVE_INT,
            );
            // The limb array, filled from the function's own parameters.
            let source = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                16,
                3,
            ));
            builder.ins().stack_store(operands[0], source, 0);
            builder.ins().stack_store(operands[1], source, 8);
            let source_address = builder.ins().stack_addr(pointer_type, source, 0);
            let pair = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                16,
                3,
            ));
            let pair_address = builder.ins().stack_addr(pointer_type, pair, 0);
            let sign = builder.ins().iconst(types::I64, 0);
            let length = builder.ins().iconst(types::I64, 2);
            let intern = compiler
                .function_local
                .native_int_intern
                .expect("the rig declares intern");
            let call = builder.ins().call(
                intern,
                &[native_arena, sign, source_address, length, pair_address],
            );
            Lowering::require_i64(builder, builder.inst_results(call)[0], 0);
            let marker = builder.ins().stack_load(types::I64, pair, 0);
            let payload = builder.ins().stack_load(types::I64, pair, 8);
            // ⛔ Registered exactly as production registers one — the marker is
            // the pair's own transport tag, not a constant chosen here.
            compiler
                .function_local
                .native_int_tags
                .insert(payload, marker);
            let value = Lowered::Int {
                value: payload,
                known: None,
            };
            Ok(compiler.transfer_into_carrier(builder, root, &value)?.word)
        },
    )
    .expect("the wide-Int producer emits");

    let run: extern "C" fn(*const u64, i64, i64) -> i64 = unsafe { std::mem::transmute(code) };
    let native = crate::native_int::NativeIntArenaV1::default();
    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = b2f_d9_bind_wide_arena(&mut store, &native);
    let word = crate::boundary_value::BoundaryWord(
        run(base, limbs[0] as i64, limbs[1] as i64) as u64,
    );
    let image = store.image();
    let copied = image.0.node_limbs(word.payload()).map(<[u64]>::to_vec);
    let sign = image
        .0
        .node_field(word.payload(), crate::boundary_value::NODE_PAYLOAD);
    let extent = image
        .0
        .node_field(word.payload(), crate::boundary_value::NODE_EXTENT);
    let sealed = image
        .0
        .node_field(word.payload(), crate::boundary_value::NODE_INT_SEALED);
    (word, copied.unwrap_or_default(), sign, extent, sealed)
}

/// ⭐⭐ **`D9` — a REAL native `Big` crosses as an owned deep copy, with its
/// exact sign and every limb.**
///
/// ⛔ **This is the row the `ERR_ESCAPE` residual was standing in for, and the
/// residual was false.** The claim was that a wide `Int` would fail closed at
/// `store_int_tag`'s owner guard. It never reaches that guard: a `Big` payload
/// is a **slot identity**, `make_immediate` answers `OK` for a low slot, and the
/// value crossed as the integer `1`. ⇒ The marker must partition the path
/// *before* any magnitude question is asked.
///
/// **MEASURED:** one compiled body interns a native `Int` from run-time limbs
/// through `ken_native_int_intern_local`, transfers it, and the persistent node
/// carries the `BOUNDARY_INT_REGION_LIMBS` marker, sign `0`, and **both** limbs.
/// **CLAIMED:** a valid region-limbed `Int` crosses a unit result boundary
/// successfully, by owned deep copy, with no borrow escaping.
/// **THE GAP:** ⚠ this fixture's magnitude is two limbs. The copy loop is over a
/// **runtime** length, so nothing here is specialized to two — but a defect that
/// only appears past some larger limb count is not measured by it.
///
/// ⚠ Promise class: **durable invariant** — it asserts the round trip of limbs
/// it supplies at run time, not a frozen node index or encoding.
#[test]
fn b2f_d9_a_real_native_big_crosses_as_an_owned_region_limbed_copy() {
    // ⚠ The top limb is non-zero, so `intern` cannot trim this to a `Small`.
    // The low limb is deliberately NOT the value a slot identity would be.
    let (word, copied, sign, extent, sealed) = b2f_d9_wide_int([0xdead_beef_0000_0001, 3]);
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::PersistentGround),
        "⛔ a wide `Int` must cross as a persistent handle. An `ImmediateInt` \
         here is the silent-corruption path: the SLOT was encoded as an integer"
    );
    assert_eq!(
        extent,
        Some(crate::boundary_value::BOUNDARY_INT_REGION_LIMBS),
        "⛔ the persistent node carries the REGION-LIMBS marker — never the \
         native `Big` marker, which names storage that dies with the invocation"
    );
    assert_eq!(sign, Some(0), "the sign is copied, not assumed");
    // ⭐ Asserted on its OWN field, before the limbs. `node_limbs` returns
    // `None` for an unsealed node, so without this row an omitted `seal_int`
    // reddens the *limb* assertion and reports a dropped limb — a true failure
    // under a message that names the wrong cause.
    assert_eq!(
        sealed,
        Some(1),
        "⛔ the copy must END in `seal_int`: until it succeeds the node DENOTES \
         NOTHING, so an unsealed node is not a value that crossed"
    );
    assert_eq!(
        copied,
        vec![0xdead_beef_0000_0001u64, 3],
        "⛔ EVERY limb, in order — a dropped, substituted or reordered limb is a \
         different integer"
    );
}

/// ⭐⭐ **`D9` POSITIVE CONTROL — the SAME compiled body takes the `Small` arm
/// when the interned pair comes back `Small`.**
///
/// ⛔ **Why this is the discriminator and not a repeat.** `intern` trims leading
/// zero limbs, so `(x, 0)` and `(x, 1)` differ only in a **run-time** operand and
/// come back with different markers from the same call. ⇒ If the producer had
/// specialized the marker at compile time, one compiled body could not answer
/// both ways. A body that always took the wide arm passes the row above and
/// fails here.
///
/// **MEASURED:** one body, two run-time limb pairs, two different outcomes —
/// a region-limbed persistent copy and an immediate word.
/// **CLAIMED:** the marker partition is emitted code reading a runtime tag.
/// **THE GAP:** the `Small` value here also fits the immediate field, so this
/// row does not separately re-establish the `Small` spill — the adjacent
/// `MAX`/`MAX + 1` rows do that.
#[test]
fn b2f_d9_the_same_body_takes_the_small_arm_on_a_trimmed_pair() {
    // Top limb zero ⇒ `intern` trims to one limb ⇒ a `Small` pair.
    let (word, copied, _sign, _extent, _sealed) = b2f_d9_wide_int([7, 0]);
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::ImmediateInt),
        "POSITIVE CONTROL: a trimmed pair is `Small`, and 7 fits the immediate \
         field — the SAME body that region-copied the wide value must take the \
         immediate arm here"
    );
    assert_eq!(
        word.signed_payload(),
        7,
        "and it must carry the value, not the slot and not a truncation"
    );
    assert!(
        copied.is_empty(),
        "NON-VACUITY: an immediate word names no node, so there are no limbs to \
         read — if this had limbs, the readback is looking at the wrong node"
    );
}

/// ⭐⭐ **`D9` / `AC-13` item 2 — THE NO-PAIR ROUTE into the spillable dispatch.**
///
/// ⛔ **The spillable arm has TWO ENTRY ROUTES with different preconditions, and
/// a fixture on one says nothing about the other.** On the **pair-bearing**
/// route the `NativeIntV1` marker partition governs and both `Small` and `Big`
/// are live; on the **no-pair** route that partition **never engages** and the
/// `Small` marker comes from [`Lowering::carrier_small_marker`]. ⇒ This row is
/// the required discharge of the second route, ⛔ not an extra class.
///
/// ⛔ **Why the `Int` rows do not cover it.** `ProcessExitStatus`,
/// `BoundedNat` and `StructuralNat` reach the dispatch by a *different route*:
/// they have no `NativeIntV1` pair, so they skip the marker partition entirely
/// and are handed a `Small` marker by [`Lowering::carrier_small_marker`]. ⇒ A
/// suite whose only spillable fixture is an `Int` measures the marker partition
/// and leaves the three no-pair variants unexecuted — three of the four
/// contributors to *"63 of 69"* silently untested.
///
/// ⭐ **And the tag is the discriminator.** Each of the three has its own
/// `BoundaryTag`, and reading it back proves the disposition's tag reached
/// `make_immediate` rather than a hardcoded `ImmediateInt`.
///
/// **MEASURED:** a `ProcessExitStatus` transferred through the producer returns
/// a word tagged `ImmediateExitStatus` carrying its value.
/// **CLAIMED:** the **no-pair route** into the dispatch is exercised, and it
/// carries the disposition's own tag.
/// **THE GAP:** ⚠ this row executes **one** of the three no-pair classes.
///
/// ⛔⛔ **And the other two are NOT discharged by that.** `BoundedNat` and
/// `StructuralNat` share this arm and this emitter, but *"covered by the
/// neighbour that went green"* is a **pin claim, not a measurement** — a class
/// that never executed is not evidence about itself, whatever its arm-mate did.
/// Their constructors are private to the lowering, so **no behavioural fixture
/// can reach them at all.**
///
/// ✅ **What discharges them is a different mechanism, not this test:** the
/// producer's `match` over `Lowered` is **exhaustive and wildcard-free**, so a
/// class that is silently unhandled is a **compile error**. That is a *compiler*
/// proof, and it is strictly stronger here than a fixture would be — ⛔ it is
/// not this row reaching further than it does. (`AC-13` item 1; Steward
/// `evt_3k37x62bj040x`.)
///
/// ⚠ Promise class: **durable invariant** — it relates the returned tag to the
/// disposition's own declared tag, not to a frozen number.
#[test]
fn b2f_d9_a_no_pair_spillable_crosses_on_its_own_tag() {
    let fixture = ac_c7_ctor("Alpha");
    let (plan, root) = planned_root_occurrence(&fixture);
    let seed_env = NativeSeedEnvironment::empty();
    let (_module, code) = ac_c7_try_compile_edge_with_operands(
        &seed_env,
        plan,
        1,
        |compiler, builder, operands| {
            let status = Lowered::ProcessExitStatus { value: operands[0] };
            Ok(compiler.transfer_into_carrier(builder, root, &status)?.word)
        },
    )
    .expect("the no-pair spillable emits");
    let run: extern "C" fn(*const u64, i64) -> i64 = unsafe { std::mem::transmute(code) };
    let mut store = crate::boundary_value::BoundaryValueStore::new();
    let (_arena, base) = ac_c7_bind_arena(&mut store);
    let word = crate::boundary_value::BoundaryWord(run(base, 42) as u64);
    assert_eq!(
        word.tag(),
        Some(BoundaryTag::ImmediateExitStatus),
        "⛔ its OWN tag — an `ImmediateInt` here means the emitter took the \
         disposition's tag from the wrong place"
    );
    assert_eq!(
        word.signed_payload(),
        42,
        "and it carries the status it was handed at run time"
    );
    assert_ne!(
        BoundaryTag::ImmediateExitStatus as u8,
        BoundaryTag::ImmediateInt as u8,
        "NON-VACUITY: the two tags must differ, or the assertion above cannot \
         tell a per-variant tag from a hardcoded one"
    );
}

// ─── RT-WORKER-BIND `D2` — the construction route's pre-installation facts ───

/// A planned `Let` whose bound value is a lexical closure with one capture.
///
/// The origins come from the plan, positionally, exactly as `D2` projects
/// them: the closure is the `Let`'s child `0`, the worker body is the
/// closure's child `0`, and capture `i` is the closure's child `1 + i`.
#[cfg(test)]
fn worker_source() -> RuntimeExpr {
    RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::LexicalClosure {
            captures: vec![RuntimeExpr::Value(RuntimeValue::Int(7.into()))],
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Var(0)),
        }),
        body: Box::new(RuntimeExpr::Var(0)),
    }
}

/// A descriptor that agrees with itself: `parameters` parameters, `captures`
/// captures, one slot per declared item, and an offset per slot.
#[cfg(test)]
fn worker_descriptor(
    origin: StaticOriginId,
    parameters: u32,
    captures: u32,
) -> units::WorkerTemplate {
    let mut slots = Vec::new();
    for ordinal in 0..parameters {
        slots.push(AbiSlot {
            kind: AbiSlotKind::Parameter,
            carrier: AbiCarrier::ValueWord,
            ownership: AbiOwnership::OwnedByFrame,
            storage_owner: AbiStorageOwner::ActivationFrame,
            width_bytes: 8,
            align_bytes: 8,
            ordinal,
        });
    }
    for ordinal in 0..captures {
        slots.push(AbiSlot {
            kind: AbiSlotKind::Capture,
            carrier: AbiCarrier::ValueWord,
            ownership: AbiOwnership::OwnedByFrame,
            storage_owner: AbiStorageOwner::ActivationFrame,
            width_bytes: 8,
            align_bytes: 8,
            ordinal,
        });
    }
    let offsets = (0..slots.len() as u32).map(|index| index * 8).collect();
    // `D5a` checkpoint 1: the constructor now validates against the RAW
    // TEMPLATE, so these controls follow it there. ⭐ The move is the point --
    // a `WorkerTemplate` has no `FuncRef` field at all, so these seven controls
    // now measure a record that could not name a callee even if the fixture
    // wanted it to.
    units::WorkerTemplate {
        origin,
        // The D2 worker-descriptor fixture keys on the same origin it targets;
        // the D1b pair is exercised by the production join, not here.
        call_site_origin: origin,
        header: AbiFrameHeader {
            parameters,
            captures,
            frame_bytes: (slots.len() as u32) * 8,
            align_bytes: 8,
        },
        slots,
        offsets,
    }
}

/// Wrap a raw template as a CALL TARGET for the `worker_calls` axis.
///
/// ⚠ Two axes, two record types, and this helper is where they meet:
/// `worker_templates` carries the raw contract the constructor validates, and
/// `worker_calls` carries the callee `call_static_worker` emits. ⛔ In
/// production those two may name different functions -- that is the `D5a`
/// retarget -- so a fixture that needs both must build both rather than reuse
/// one for the other.
#[cfg(test)]
fn worker_call_target(template: units::WorkerTemplate) -> units::DeclaredUnitCall {
    units::DeclaredUnitCall {
        function: cranelift_codegen::ir::FuncRef::from_u32(0),
        origin: template.origin,
        call_site_origin: template.call_site_origin,
        header: template.header,
        slots: template.slots,
        offsets: template.offsets,
    }
}

/// Drives one construction attempt against a descriptor the caller shapes.
///
/// Returns the route's own verdict, so a test asserts on the construction
/// rather than on some later emission.
#[cfg(test)]
fn attempt_worker_construction(
    install: impl FnOnce(StaticOriginId, StaticOriginId) -> Option<units::WorkerTemplate>,
    declared_arity: u32,
    source_capture_count: usize,
    capture_operands: usize,
) -> Result<StaticWorkerBinding, CraneliftBackendError> {
    let source = worker_source();
    let (plan, root) = planned_root_occurrence(&source);
    let closure_origin = plan
        .child_static_origin(root, 0)
        .expect("the Let's bound value is planned as child 0");
    let body_origin = plan
        .child_static_origin(closure_origin, 0)
        .expect("a lexical closure plans its body as child 0");
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);
    if let Some(target) = install(body_origin, closure_origin) {
        compiler
            .function_local
            .worker_templates
            .insert(body_origin, target);
    }
    // `Lowered::Bytes` needs no emitted value, so the fixture builds captures
    // without a builder. The route is phase-agnostic by design -- it stores
    // operands unchanged -- so the descriptor facts below are what is under
    // test, not the capture phase.
    let captures = (0..capture_operands)
        .map(|index| {
            LoweringOperand::Specialized(Lowered::Bytes(format!("capture{index}").into_bytes()))
        })
        .collect::<Vec<_>>();
    compiler.construct_static_worker_binding(
        closure_origin,
        body_origin,
        declared_arity,
        source_capture_count,
        captures,
        // `D6a` — these rows are about the descriptor contract, which is
        // validated identically on both routes. The raw route is the one an
        // ordinary lexical closure takes, so it is the honest default here.
        StaticWorkerCallRoute::RawWorker,
        // `D8i` — likewise route-independent: these rows validate the
        // descriptor contract, which runs before the discharge facet is even
        // looked at. The ordinary arm is what an ordinary lexical closure
        // carries.
        ContinuationDischarge::DirectSpecializationCall,
    )
}

/// `StaticWorkerBinding` deliberately has no `Debug` (it holds
/// `LoweringOperand`, which has none), so the tests below destructure rather
/// than reach for `expect`/`expect_err`.
#[cfg(test)]
fn expect_worker_rejection(
    result: Result<StaticWorkerBinding, CraneliftBackendError>,
) -> CraneliftBackendError {
    match result {
        Ok(_) => panic!("the construction route installed a binding where it must reject"),
        Err(error) => error,
    }
}

#[cfg(test)]
fn expect_worker_binding(
    result: Result<StaticWorkerBinding, CraneliftBackendError>,
) -> StaticWorkerBinding {
    match result {
        Ok(binding) => binding,
        Err(error) => panic!("an agreeing descriptor must install: {error:?}"),
    }
}

/// The route succeeds when every declared fact agrees, and stores exactly the
/// projected origins, arity and captures.
#[test]
fn static_worker_construction_installs_on_agreeing_descriptor() {
    let binding = expect_worker_binding(attempt_worker_construction(
        |origin, _| Some(worker_descriptor(origin, 1, 1)),
        1,
        1,
        1,
    ));
    assert_eq!(binding.declared_arity, 1);
    assert_eq!(binding.captures.len(), 1);
    assert!(
        matches!(&binding.captures[0], LoweringOperand::Specialized(Lowered::Bytes(value))
            if value == b"capture0"),
        "captures are stored unchanged, in order"
    );
    assert_ne!(
        binding.closure_origin, binding.body_origin,
        "the closure occurrence and its child-0 body are distinct origins"
    );
}

/// A worker body with no declared static-body target in this function rejects
/// before installation, rather than yielding a binding that could later be
/// called.
#[test]
fn static_worker_construction_rejects_missing_target() {
    let error = expect_worker_rejection(attempt_worker_construction(|_, _| None, 1, 1, 1));
    assert!(
        // `D5a` checkpoint 1 moved the constructor's authority from the
        // declared call target to the raw worker template, and the diagnostic
        // moved with it. Same seam, and a sharper reason: what is missing is
        // the RAW CONTRACT, which a function has whether or not it also has a
        // callee to reach.
        format!("{error:?}").contains("no raw worker template"),
        "rejects for the missing-template reason, not some later one: {error:?}"
    );
}

/// A declared unit call recorded against a different body origin is a
/// wrong-body fact and rejects.
#[test]
fn static_worker_construction_rejects_wrong_body_origin() {
    let error = expect_worker_rejection(attempt_worker_construction(
        |origin, other| {
            let mut target = worker_descriptor(origin, 1, 1);
            // `D1b` moved the wrong-body fact onto the end that names the
            // source body. The declared record is keyed by `call_site_origin`,
            // so perturbing THAT is what a wrong body now is; `origin` carries
            // the scheduling entry and is a different fact.
            target.call_site_origin = other;
            Some(target)
        },
        1,
        1,
        1,
    ));
    assert!(
        format!("{error:?}").contains("but the worker body origin"),
        "rejects for the wrong-body reason: {error:?}"
    );
}

/// A descriptor whose parameter count disagrees with the source closure's
/// declared arity rejects.
#[test]
fn static_worker_construction_rejects_wrong_arity() {
    let error = expect_worker_rejection(attempt_worker_construction(
        |origin, _| Some(worker_descriptor(origin, 2, 1)),
        1,
        1,
        1,
    ));
    assert!(
        format!("{error:?}").contains("parameters but the source closure declares"),
        "rejects for the wrong-arity reason: {error:?}"
    );
}

/// A descriptor whose capture count disagrees with the projected capture
/// vector rejects.
#[test]
fn static_worker_construction_rejects_wrong_capture_count() {
    let error = expect_worker_rejection(attempt_worker_construction(
        |origin, _| Some(worker_descriptor(origin, 1, 2)),
        1,
        1,
        1,
    ));
    assert!(
        format!("{error:?}").contains("captures but"),
        "rejects for the wrong-capture reason: {error:?}"
    );
}

/// A capture vector that disagrees with the retained definition rejects before
/// the descriptor is even consulted.
#[test]
fn static_worker_construction_rejects_capture_count_against_definition() {
    let error = expect_worker_rejection(attempt_worker_construction(
        |origin, _| Some(worker_descriptor(origin, 1, 1)),
        1,
        2,
        1,
    ));
    assert!(
        format!("{error:?}").contains("were projected"),
        "rejects against the retained definition: {error:?}"
    );
}

/// A descriptor whose slot run disagrees with its own offsets rejects, so the
/// binding never carries a layout it did not take unchanged.
#[test]
fn static_worker_construction_rejects_slot_offset_disagreement() {
    let error = expect_worker_rejection(attempt_worker_construction(
        |origin, _| {
            let mut target = worker_descriptor(origin, 1, 1);
            target.offsets.pop();
            Some(target)
        },
        1,
        1,
        1,
    ));
    assert!(
        format!("{error:?}").contains("offsets"),
        "rejects for the layout-agreement reason: {error:?}"
    );
}

/// A descriptor whose slot run disagrees with its header's counts rejects.
#[test]
fn static_worker_construction_rejects_slot_run_against_header() {
    let error = expect_worker_rejection(attempt_worker_construction(
        |origin, _| {
            let mut target = worker_descriptor(origin, 1, 1);
            // Header still claims one parameter and one capture; the slot run
            // now carries two captures and no parameter.
            target.slots[0].kind = AbiSlotKind::Capture;
            Some(target)
        },
        1,
        1,
        1,
    ));
    assert!(
        format!("{error:?}").contains("slot run declares"),
        "rejects for the slot-run reason: {error:?}"
    );
}

// ─── RT-WORKER-BIND `D3`/`D4` — the callee-only consumer and its escapes ────

/// Drives one lowering of `subject` in a function whose environment binds a
/// static worker at de Bruijn index 0.
///
/// `declare_target` decides whether this function has a worker call target
/// declared for the binding's body origin, which is the `D4` axis; the
/// binding's own arity is the `D3` axis.
#[cfg(test)]
fn lower_against_static_worker(
    subject: &RuntimeExpr,
    declared_arity: u32,
    declare_target: bool,
) -> Result<LoweringOperand, CraneliftBackendError> {
    let source = worker_source();
    let (plan, root) = planned_root_occurrence(&source);
    let closure_origin = plan
        .child_static_origin(root, 0)
        .expect("the Let's bound value is planned as child 0");
    let body_origin = plan
        .child_static_origin(closure_origin, 0)
        .expect("a lexical closure plans its body as child 0");
    let (subject_plan, subject_origin) = planned_root_occurrence(subject);
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = bare_carrier_test_lowering(&seed_env, subject_plan);
    if declare_target {
        compiler
            .function_local
            .worker_calls
            .insert(
                body_origin,
                worker_call_target(worker_descriptor(body_origin, declared_arity, 1)),
            );
    }
    let env = [LoweringEnvironmentBinding::StaticWorker(StaticWorkerBinding {
        closure_origin,
        body_origin,
        declared_arity,
        captures: vec![LoweringOperand::Specialized(Lowered::Bytes(b"cap".to_vec()))],
        route: StaticWorkerCallRoute::RawWorker,
        // `D8i` — a hand-built ordinary binding. ⛔ The composed arm is not
        // constructible here even in a test: it needs a planner-issued
        // `ContinuationCallIdentity`, which has no constructor outside planning.
        discharge: ContinuationDischarge::DirectSpecializationCall,
    })];
    let mut func = Function::with_name_signature(
        UserFuncName::user(0, 0),
        cranelift_codegen::ir::Signature::new(cranelift_codegen::isa::CallConv::SystemV),
    );
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    compiler.lower_expr(
        &mut builder,
        SourceOccurrence {
            expr: subject,
            static_origin: subject_origin,
        },
        &env,
    )
}

/// `LoweringOperand` has no `Debug` either, so worker-consumer rejections are
/// destructured rather than reached for with `expect_err`.
#[cfg(test)]
fn expect_lowering_rejection(
    result: Result<LoweringOperand, CraneliftBackendError>,
) -> CraneliftBackendError {
    match result {
        Ok(_) => panic!("lowering produced an operand where it must fail closed"),
        Err(error) => error,
    }
}

/// A bare `Var` naming the worker is a value-producing position and fails
/// closed: a static worker binding has no value representation.
#[test]
fn static_worker_fails_closed_in_value_position() {
    let subject = RuntimeExpr::Var(0);
    let error = expect_lowering_rejection(lower_against_static_worker(&subject, 1, true));
    assert!(
        format!("{error:?}").contains("value-producing position"),
        "fails closed for the value-position reason: {error:?}"
    );
}

/// The same binding used as an aggregate field fails closed before any
/// carrier transfer, rather than entering the constructor's argument list.
#[test]
fn static_worker_fails_closed_as_aggregate_field() {
    let subject = RuntimeExpr::Construct {
        constructor: "ctor:fixture::Box::Wrap".to_string(),
        args: vec![RuntimeExpr::Var(0)],
    };
    let error = expect_lowering_rejection(lower_against_static_worker(&subject, 1, true));
    assert!(
        format!("{error:?}").contains("value-producing position"),
        "fails closed for the value-position reason: {error:?}"
    );
}

/// The same binding as a match scrutinee fails closed.
///
/// This control sits on the scrutinee rather than on an ordinary call's
/// argument, and the reason is a measured one: with a non-closure callee the
/// `Call` arm rejects the callee before it lowers any argument, so a worker in
/// that position is never reached and a control there would pass for the
/// wrong reason. The scrutinee is reached directly.
#[test]
fn static_worker_fails_closed_as_match_scrutinee() {
    let subject = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Var(0)),
        cases: vec![crate::RuntimeMatchCase {
            constructor: "ctor:fixture::Box::Wrap".to_string(),
            binders: 0,
            body: RuntimeExpr::Value(RuntimeValue::Int(1.into())),
        }],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "worker scrutinee control".to_string(),
        },
    };
    let error = expect_lowering_rejection(lower_against_static_worker(&subject, 1, true));
    assert!(
        format!("{error:?}").contains("value-producing position"),
        "fails closed for the value-position reason: {error:?}"
    );
}

/// The consumer is reached through the exact `Var` callee, and validates the
/// supplied argument count against the binding's declared arity.
#[test]
fn static_worker_call_rejects_arity_disagreement() {
    let subject = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: vec![
            RuntimeExpr::Value(RuntimeValue::Int(1.into())),
            RuntimeExpr::Value(RuntimeValue::Int(2.into())),
        ],
    };
    let error = expect_lowering_rejection(lower_against_static_worker(&subject, 1, true));
    assert!(
        format!("{error:?}").contains("static worker expects"),
        "reaches the consumer and rejects on arity: {error:?}"
    );
}

/// `D4`: a worker whose body origin was never declared into this function
/// rejects, rather than reaching for another function's target.
#[test]
fn static_worker_call_rejects_undeclared_target() {
    let subject = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(1.into()))],
    };
    let error = expect_lowering_rejection(lower_against_static_worker(&subject, 1, false));
    assert!(
        format!("{error:?}").contains("was declared into this"),
        "rejects for the undeclared-target reason: {error:?}"
    );
}

// ─── RT-WORKER-BIND `D8` — the independent ordinary witness ─────────────────

/// **The `D8` witness program, and it contains ZERO continuation machinery.**
///
/// An ordinary `FunctionizedUnits` program: no `ComputationalMatch`, no
/// continuation specialization, identity, descriptor or token anywhere in it.
/// `FunctionizedUnits` is the *default* authority -- it is selected whenever
/// the source carries no recursive-descent residual -- so this fixture reaches
/// it by being ordinary, not by asking for it.
///
/// Shape:
///
/// - a normal unit receives a real ABI input, so `x` arrives **`Carried`**;
/// - an ordinary `Let` binds a lexical closure capturing two operands in
///   order -- the carried `x` first, a specialized constant second;
/// - the `Let` body calls `Var(0)`, which is that binding.
///
/// The carried capture is what routes the binder to `StaticWorker`; the call
/// through the exact `Var(0)` callee is what consumes it.
#[cfg(test)]
pub(super) fn static_worker_witness(capture_first: bool) -> RuntimeExpr {
    let carried = RuntimeExpr::Var(0);
    let constant = RuntimeExpr::Value(RuntimeValue::Int(3.into()));
    let captures = if capture_first {
        vec![carried, constant]
    } else {
        vec![constant, carried]
    };
    // Inside the worker body the environment is the unit's slot run: the
    // parameter first, then the captures in declared order.
    let worker = RuntimeExpr::LexicalClosure {
        captures,
        params: vec!["y".to_string()],
        body: Box::new(RuntimeExpr::Var(1)),
    };
    let outer_body = RuntimeExpr::Let {
        value: Box::new(worker),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Var(0)),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(100.into()))],
        }),
    };
    RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(outer_body),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    }
}

/// The witness contains no continuation spelling. This is `AC-4`'s first half,
/// asserted over the fixture the test actually runs.
#[test]
fn static_worker_witness_contains_no_continuation_machinery() {
    let witness = static_worker_witness(true);
    let rendered = format!("{witness:?}");
    for spelling in [
        "ComputationalMatch",
        "ContinuationSpecializationId",
        "ContinuationCallIdentity",
        "ContinuationDescriptor",
        "ContinuationToken",
    ] {
        assert!(
            !rendered.contains(spelling),
            "the witness must contain zero continuation machinery, found {spelling}"
        );
    }
}

/// The witness compiles and executes end to end, and its result distinguishes
/// capture order.
#[test]
fn static_worker_witness_runs_and_distinguishes_capture_order() {
    let ordered = static_worker_witness(true);
    let compiled = crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        &ordered,
        &NativeSeedEnvironment::empty(),
    )
    .expect("the ordinary witness compiles");
    let observed = compiled.run(None).expect("the witness runs").0;
    let swapped = static_worker_witness(false);
    let swapped_observed =
        crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
            &swapped,
            &NativeSeedEnvironment::empty(),
        )
        .expect("the capture-swapped witness compiles")
        .run(None)
        .expect("the swapped witness runs")
        .0;
    assert_ne!(
        observed, swapped_observed,
        "swapping the capture order must change the linked result"
    );
}

/// `AC-8`/judgment 3 -- **the binding is NOT affine.** An installed worker
/// that is never called must still compile and run.
///
/// If any consumed-set, once-token or required-empty ledger existed, this
/// would fail; it is the companion that would catch one being introduced.
#[test]
fn static_worker_unused_binding_succeeds() {
    let expr = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::LexicalClosure {
                    captures: vec![
                        RuntimeExpr::Var(0),
                        RuntimeExpr::Value(RuntimeValue::Int(3.into())),
                    ],
                    params: vec!["y".to_string()],
                    body: Box::new(RuntimeExpr::Var(1)),
                }),
                // The binding is installed and simply never called.
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Int(42.into()))),
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    };
    let compiled = crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        &expr,
        &NativeSeedEnvironment::empty(),
    )
    .expect("an unused worker binding is lawful and must compile");
    assert_eq!(
        compiled.run(None).expect("the unused-binding fixture runs").0,
        RuntimeObservation::Returned(RuntimeGroundValue::Int(42.into()))
    );
}

/// `AC-8`/judgment 3 -- a binding called **twice** is lawful too. Nothing
/// consumes the binding on first use.
#[test]
fn static_worker_twice_called_binding_succeeds() {
    // The inner `Let` shifts de Bruijn indices, so the second call names the
    // worker at `Var(1)` while the first names it at `Var(0)`. Same binding,
    // called twice.
    let call = |index: u32| RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(index)),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(100.into()))],
    };
    let expr = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::LexicalClosure {
                    captures: vec![
                        RuntimeExpr::Var(0),
                        RuntimeExpr::Value(RuntimeValue::Int(3.into())),
                    ],
                    params: vec!["y".to_string()],
                    body: Box::new(RuntimeExpr::Var(1)),
                }),
                // Called once, then called again in the same scope.
                body: Box::new(RuntimeExpr::Let {
                    value: Box::new(call(0)),
                    body: Box::new(call(1)),
                }),
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    };
    let compiled = crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        &expr,
        &NativeSeedEnvironment::empty(),
    )
    .expect("a twice-called worker binding is lawful and must compile");
    compiled
        .run(None)
        .expect("the twice-called fixture runs");
}

// ─── RT-WORKER-BIND `D5`/`D6`/`D7` — multiple, nested, and completion ───────

/// Two same-shape workers -- same arity, same capture count -- at distinct de
/// Bruijn slots, with distinct bodies and distinct capture orders, both
/// called. The result is an aggregate of both calls, so it depends on each
/// worker's body **and** its capture order independently.
#[cfg(test)]
fn two_same_shape_workers(first_body: u32, second_body: u32, swap_second: bool) -> RuntimeExpr {
    let cap_a = vec![
        RuntimeExpr::Var(0),
        RuntimeExpr::Value(RuntimeValue::Int(3.into())),
    ];
    // `x` sits at index 1 here, not 0: worker A is already bound at 0 by the
    // enclosing `Let`. Naming `Var(0)` would capture the WORKER as a value,
    // which fails closed -- the guard caught exactly that while this fixture
    // was being written.
    let cap_b = if swap_second {
        vec![
            RuntimeExpr::Value(RuntimeValue::Int(7.into())),
            RuntimeExpr::Var(1),
        ]
    } else {
        vec![
            RuntimeExpr::Var(1),
            RuntimeExpr::Value(RuntimeValue::Int(7.into())),
        ]
    };
    // Two `Let`s in one environment: worker A ends at index 1 once B is bound,
    // so the pair also exercises binder-order preservation at distinct slots.
    let inner = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::LexicalClosure {
            captures: cap_b,
            params: vec!["y".to_string()],
            body: Box::new(RuntimeExpr::Var(second_body)),
        }),
        body: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Pair::Both".to_string(),
            args: vec![
                RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(1)),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(100.into()))],
                },
                RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(200.into()))],
                },
            ],
        }),
    };
    RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::LexicalClosure {
                    captures: cap_a,
                    params: vec!["y".to_string()],
                    body: Box::new(RuntimeExpr::Var(first_body)),
                }),
                body: Box::new(inner),
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    }
}

#[cfg(test)]
fn run_worker_fixture(expr: &RuntimeExpr) -> RuntimeObservation {
    crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        expr,
        &NativeSeedEnvironment::empty(),
    )
    .expect("the worker fixture compiles")
    .run(None)
    .expect("the worker fixture runs")
    .0
}

/// `D5` -- two same-shape workers in one environment are genuinely
/// distinguished, and swapping either one's body or its capture order changes
/// the linked result.
///
/// This is also `AC-5`'s target-redirect red: the two workers are same-shape,
/// so a call resolving to the other one's body is exactly a redirected target.
#[test]
fn two_same_shape_workers_are_distinguished() {
    let baseline = run_worker_fixture(&two_same_shape_workers(1, 1, false));
    let body_swapped = run_worker_fixture(&two_same_shape_workers(2, 1, false));
    let capture_swapped = run_worker_fixture(&two_same_shape_workers(1, 1, true));
    assert_ne!(
        baseline, body_swapped,
        "changing which capture the first worker's body selects must move the result"
    );
    assert_ne!(
        baseline, capture_swapped,
        "swapping the second worker's capture order must move the result"
    );
    assert_ne!(
        body_swapped, capture_swapped,
        "the two mutations must be distinguishable from each other, not merely from the baseline"
    );
}

/// `D6` -- a static worker body that binds and calls **another** static
/// worker.
///
/// The inner closure's captures are the outer worker function's own value
/// operands, carried ones included: capture 0 is the outer worker's parameter
/// and capture 1 is the outer worker's own first capture. Both are carried
/// inside that function, so the inner binder installs a second `StaticWorker`
/// whose target must be declared afresh **into the outer worker's function**.
///
/// `outer_body` and `inner_body` select which operand each level returns, so
/// the result depends on both levels independently.
#[cfg(test)]
fn nested_workers(inner_body: u32, swap_inner_captures: bool) -> RuntimeExpr {
    let inner_captures = if swap_inner_captures {
        vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)]
    } else {
        vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)]
    };
    // Inside the OUTER worker body: [y(param), cap0 = x, cap1 = 3].
    let outer_worker_body = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::LexicalClosure {
            captures: inner_captures,
            params: vec!["z".to_string()],
            body: Box::new(RuntimeExpr::Var(inner_body)),
        }),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Var(0)),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(500.into()))],
        }),
    };
    RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::LexicalClosure {
                    captures: vec![
                        RuntimeExpr::Var(0),
                        RuntimeExpr::Value(RuntimeValue::Int(3.into())),
                    ],
                    params: vec!["y".to_string()],
                    body: Box::new(outer_worker_body),
                }),
                body: Box::new(RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(100.into()))],
                }),
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    }
}

/// `D6`/`AC-7` -- the nested positive depends on BOTH levels, and each
/// mutation moves the result independently.
///
/// This is also `AC-9`'s evidence: the inner worker's target is declared into
/// the **outer worker's** function, which is a different `Function` from the
/// root. A `FuncRef` copied across functions would not verify, so a green
/// nested run is exactly the fresh-per-function declaration working.
#[test]
fn nested_worker_depends_on_both_levels() {
    let baseline = run_worker_fixture(&nested_workers(1, false));
    let inner_body_moved = run_worker_fixture(&nested_workers(2, false));
    let inner_captures_swapped = run_worker_fixture(&nested_workers(1, true));
    assert_ne!(
        baseline, inner_body_moved,
        "moving which operand the inner body selects must move the result"
    );
    assert_ne!(
        baseline, inner_captures_swapped,
        "swapping the inner worker's capture order must move the result"
    );
}

/// `D8` companion -- **capture omission.** Dropping a capture the body reads
/// must not silently succeed with a shifted environment.
///
/// The witness body reads capture 0 at `Var(1)`; with only one capture
/// declared, `Var(2)` names nothing and the lowering fails closed rather than
/// reading past the worker's environment.
#[test]
fn static_worker_capture_omission_fails_closed() {
    let omitted = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::LexicalClosure {
                    // One capture declared, but the body reads a second.
                    captures: vec![RuntimeExpr::Var(0)],
                    params: vec!["y".to_string()],
                    body: Box::new(RuntimeExpr::Var(2)),
                }),
                body: Box::new(RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Int(100.into()))],
                }),
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    };
    let error = crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        &omitted,
        &NativeSeedEnvironment::empty(),
    )
    .err()
    .expect("omitting a capture the body reads must fail closed");
    assert!(
        format!("{error:?}").contains("no runtime binding for index"),
        "fails closed on the missing binding rather than reading past it: {error:?}"
    );
}

// ─── RT-WORKER-BIND `AC-5` — the two executable production-seam mutations ───

/// Runs `body` with a static-worker mutation installed, restoring `Exact`
/// afterwards **even if `body` panics**, so one failing control cannot leak a
/// mutation into every later test in the thread.
#[cfg(test)]
fn with_static_worker_mutation<T>(mutation: StaticWorkerMutation, body: impl FnOnce() -> T) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            set_static_worker_mutation(StaticWorkerMutation::Exact);
        }
    }
    set_static_worker_mutation(mutation);
    let _restore = Restore;
    body()
}

/// `AC-5` mutation 1, at the real `D2` binder seam.
///
/// The **same** ordinary witness is green under `Exact` and red with the
/// pre-node carried-capture narrowing restored. No fixture is substituted:
/// the source program is identical in both runs and only production
/// resolution moves.
#[test]
fn ac5_restoring_carried_capture_narrowing_reds_the_ordinary_witness() {
    let witness = static_worker_witness(true);
    // Positive control first: without the mutation this exact program runs.
    let baseline = crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        &witness,
        &NativeSeedEnvironment::empty(),
    );
    assert!(
        baseline.is_ok(),
        "the witness must be green at the same seam the mutation reddens"
    );
    let error = with_static_worker_mutation(
        StaticWorkerMutation::RestoreCarriedCaptureNarrowing,
        || {
            crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
                &witness,
                &NativeSeedEnvironment::empty(),
            )
            .err()
        },
    )
    .expect("restoring the carried-capture narrowing must red the witness");
    assert!(
        format!("{error:?}").contains("specialized-only surface"),
        "reds at the D2 carried-capture seam, not somewhere else: {error:?}"
    );
    // The mutation is scoped: the same program is green again immediately.
    assert!(
        crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
            &witness,
            &NativeSeedEnvironment::empty(),
        )
        .is_ok(),
        "the mutation must not leak past its scope"
    );
}

/// `AC-5` mutation 2, at the real `D4` transport seam.
///
/// The **same** planned two-same-shape-worker program is green under `Exact`
/// and red when the already-resolved worker target is redirected to the other
/// same-shape worker in that same function. The binding and its construction
/// are untouched; only transport resolution moves, which is the whole point of
/// the control.
#[test]
fn ac5_redirecting_the_resolved_worker_target_reds_the_same_shape_witness() {
    let program = two_same_shape_workers(1, 1, false);
    let baseline = crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
        &program,
        &NativeSeedEnvironment::empty(),
    );
    assert!(
        baseline.is_ok(),
        "the same-shape witness must be green at the seam the mutation reddens"
    );
    let error = with_static_worker_mutation(
        StaticWorkerMutation::RedirectResolvedWorkerTarget,
        || {
            crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
                &program,
                &NativeSeedEnvironment::empty(),
            )
            .err()
        },
    )
    .expect("redirecting the resolved worker target must red the same-shape witness");
    assert!(
        format!("{error:?}").contains("worker call target carries origin"),
        "reds at the D4 transport seam's own origin check: {error:?}"
    );
    assert!(
        crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
            &program,
            &NativeSeedEnvironment::empty(),
        )
        .is_ok(),
        "the mutation must not leak past its scope"
    );
}

/// **`D7` — every construction-time occurrence lookup FAILS CLOSED.**
///
/// MEASURED, on the three consumers this row drives:
///
/// | consumer | no emission owner | live owner, unanswerable lookup |
/// |---|---|---|
/// | `synthesized_constructor` | accepts | refuses |
/// | `reconcile_declared_children`, nested `Fixed` child | not exercised | refuses |
/// | `reconcile_host_result_root` | accepts | refuses |
///
/// ⚠ The middle row's permissive cell is deliberately blank rather than
/// asserted: `reconcile_declared_children` takes its owner as an argument and
/// has no no-emission-owner branch of its own, so there is nothing there to
/// exercise. The two consumers that DO draw that boundary are the two asserted.
///
/// CLAIMED: none of these converts a failed authority lookup into an absence.
/// `None` is lawful only on the explicit no-emission-owner early return, which
/// is what the permissive column exercises.
///
/// THE GAP — and it is why these are driven at the CONSUMER rather than at the
/// planner API. The planner-side row
/// `a_lawful_non_dynamic_root_is_not_a_failed_lookup` proves the API types
/// absence apart from failure, and stays green if a consumer reintroduces
/// `.ok()`.
///
/// ⚠ Not every assertion below is a single-line discriminator.
/// `synthesized_constructor`'s repair closed its hole TWICE — with `?` and by
/// making the child reconciliation unconditional — so reverting either half
/// alone stays green and only the full predecessor is caught. That is a
/// redundancy, not a gap, and it is recorded so a green single-line revert is
/// not misread as an unpinned property.
///
/// The fourth consumer, `dynamic_alternatives_agree`, has its own row:
/// `a_dynamic_alternative_with_no_planned_record_refuses`.
#[test]
fn a_construction_time_occurrence_lookup_fails_closed() {
    use crate::cranelift_backend::planning::{
        SynthesizedAggregateNode, SynthesizedAggregatePath, SynthesizedAggregateRoot,
    };

    let source = RuntimeExpr::Construct {
        constructor: "ctor:fixture::FailClosed::Seed".to_string(),
        args: Vec::new(),
    };
    let (plan, root_origin) = planned_root_occurrence(&source);
    // A real emission owner. The seat below is deliberately NOT one this owner
    // has synthesized records at, so every lookup is unanswerable — which is
    // the state `.ok()` used to convert into "there is nothing planned here".
    let owner = ContinuationEmissionOwner::Predeclared(
        plan.emittable_units()
            .expect("a planned graph enumerates its units")
            .first()
            .copied()
            .expect("a planned graph has an emittable unit")
            .function(),
    );
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);
    let ok_root = SynthesizedAggregatePath::root(SynthesizedAggregateRoot::HostResultOk);
    let symbols = crate::NativeProcessSymbols::legacy_prelude();

    // ── The lawful absence: NO emission owner ──
    //
    // This is the one branch on which a missing occurrence is correct, and it
    // runs first so the refusals below cannot be read as "this consumer
    // refuses everything".
    compiler.defining_emission_owner = None;
    assert!(
        compiler
            .synthesized_constructor(
                root_origin,
                &ok_root,
                SynthesizedFixedConstructorRole::Wrote,
                symbols.wrote.clone(),
                Vec::new(),
                &ClaimedEffectSeats::none(),
            )
            .is_ok(),
        "with no emission owner there is no emission this population covers, \
         so the template is built carrying no occurrence"
    );
    assert!(
        compiler
            .reconcile_host_result_root(
                root_origin,
                &ok_root,
                &Lowered::Constructor {
                    constructor: symbols.wrote.clone(),
                    synthesized_identity: None,
                    occurrence: None,
                    args: Vec::new(),
                },
            )
            .is_ok(),
        "the root consumer draws the same no-emission-owner boundary"
    );

    // ── Now with a live owner: every unanswerable lookup REFUSES ──
    compiler.defining_emission_owner = Some(owner);

    // 1. The construction's own exact record. Under `.ok()` this became
    //    `occurrence: None` and the child reconciliation was SKIPPED entirely,
    //    emitting a template that would refuse only later at its allocation.
    assert!(
        compiler
            .synthesized_constructor(
                root_origin,
                &ok_root,
                SynthesizedFixedConstructorRole::Wrote,
                symbols.wrote.clone(),
                Vec::new(),
                &ClaimedEffectSeats::none(),
            )
            .is_err(),
        "a synthesized construction whose exact record does not exist must \
         refuse, not carry `None` and skip its own child reconciliation"
    );

    // 2. A nested `Fixed` child's expected record. Under `.ok()` the
    //    expectation became `None`, which compared EQUAL to a child carrying no
    //    occurrence — two absences agreed and the pair passed.
    const NESTED: &[SynthesizedAggregateNode] = &[SynthesizedAggregateNode::Fixed {
        role: SynthesizedFixedConstructorRole::PrivateTransferCount,
        children: &[],
    }];
    assert!(
        compiler
            .reconcile_declared_children(
                owner,
                root_origin,
                &ok_root,
                NESTED,
                &[SynthesizedArgument::Nested(Lowered::Constructor {
                    constructor: symbols.private_transfer_count.clone(),
                    synthesized_identity: None,
                    occurrence: None,
                    args: Vec::new(),
                })],
                &ClaimedEffectSeats::none(),
            )
            .is_err(),
        "a nested child whose expected record does not exist must refuse; two \
         absences must not compare equal"
    );

    // 3. The host-result root. The emitted root here is lawfully NON-dynamic,
    //    which is exactly the case `.ok()` let through: the failed lookup read
    //    as "the planner plans no set at this root", and `(None, non-dynamic)`
    //    returned `Ok(())`.
    assert!(
        compiler
            .reconcile_host_result_root(
                root_origin,
                &ok_root,
                &Lowered::Constructor {
                    constructor: symbols.wrote.clone(),
                    synthesized_identity: None,
                    occurrence: None,
                    args: Vec::new(),
                },
            )
            .is_err(),
        "a root whose authority lookup cannot be answered must refuse, even \
         when the emitted root is lawfully non-dynamic"
    );
}

/// The first `Effect` occurrence in a planned graph, found by walking the
/// occurrence tree with the accessors lowering itself uses.
///
/// `StaticOriginId` is unmintable here — its field is `pub(super)` — so a seat
/// has to be *discovered* rather than fabricated, which is also the honest
/// shape: the control below is about a seat the planner really issued records
/// for.
fn first_effect_seat(plan: &StaticTransitionPlan<'_>) -> Option<StaticOriginId> {
    let mut stack = vec![plan.root_static_origin().ok()?];
    let mut seen = 0usize;
    while let Some(origin) = stack.pop() {
        seen += 1;
        if seen > 4096 {
            return None;
        }
        if matches!(plan.source_occurrence(origin), Ok(RuntimeExpr::Effect { .. })) {
            return Some(origin);
        }
        let mut position = 0;
        while let Ok(child) = plan.child_static_origin(origin, position) {
            stack.push(child);
            position += 1;
        }
    }
    None
}

/// **`D7` — the dynamic-alternative consumer fails closed on a missing record.**
///
/// MEASURED: at a real `FsWriteAt` seat whose error root is the ten-alternative
/// resource surface, `dynamic_alternatives_agree` accepts the alternatives
/// carrying the occurrences the planner issued **under the seat's own emission
/// owner**, and refuses under a *different* enumerated unit's owner — at which
/// no per-alternative record exists — even though every emitted alternative
/// carries `occurrence: None` and the population cardinality still matches.
///
/// CLAIMED: the per-alternative record lookup propagates rather than mapping to
/// `None`, so missing planner authority cannot compare equal to an alternative
/// that carries no occurrence.
///
/// THE GAP: this is the one consumer cell the earlier fail-closed row could not
/// reach, and I previously reported it as unreachable from a test. That was
/// wrong in a specific way worth recording — I checked whether a
/// `PredeclaredFunctionId` could be **minted** (it cannot; the field is
/// `pub(super)`) and concluded no second owner was obtainable, without checking
/// whether a fixture could **enumerate** two. `emittable_units()` returns them,
/// and this fixture has more than one.
///
/// ⚠ The negative's discriminating power is exactly the `?`: restoring the
/// predecessor `.ok()` makes `expected` become `None`, which compares equal to
/// the emitted `None`, and the negative half passes. Both halves are asserted
/// because the positive is what stops the row degenerating into "this consumer
/// refuses everything".
#[test]
fn a_dynamic_alternative_with_no_planned_record_refuses() {
    use crate::cranelift_backend::planning::{
        SynthesizedAggregatePath, SynthesizedAggregateRoot,
    };

    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let write = RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsWriteAt,
        capability: None,
        args: vec![
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Value(RuntimeValue::Int((4).into())),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
        ],
    };
    let source = host_result_closure_match(write);
    let (plan, _) = planned_root_occurrence(&source);
    let seat = first_effect_seat(&plan).expect("the fixture has an effect seat");

    // Two ENUMERATED units, which is where the alternate owner comes from.
    let units = plan
        .emittable_units()
        .expect("a planned graph enumerates its units");
    assert!(
        units.len() > 1,
        "this control needs two enumerated units to obtain an owner the seat \
         has no records under; the fixture yielded {}",
        units.len()
    );
    let owners = units
        .iter()
        .map(|unit| ContinuationEmissionOwner::Predeclared(unit.function()))
        .collect::<Vec<_>>();

    let error_root = SynthesizedAggregatePath::root(SynthesizedAggregateRoot::HostResultError);
    let population = plan
        .synthesized_dynamic_alternatives(seat, &error_root)
        .expect("the error root is the resource surface");
    assert_eq!(population.len(), 10, "the resource surface has ten alternatives");

    let seed_env = NativeSeedEnvironment::empty();
    let compiler = bare_carrier_test_lowering(&seed_env, plan);
    let plan = &compiler.static_transition_plan;

    // The owner the seat's records were actually issued under. Found by asking
    // which enumerated owner resolves alternative 0, rather than assumed.
    let live = owners
        .iter()
        .copied()
        .find(|owner| {
            plan.synthesized_aggregate_occurrence(
                *owner,
                seat,
                &error_root.alternative(0),
                population[0],
            )
            .is_ok()
        })
        .expect("some enumerated owner holds this seat's records");
    let absent = owners
        .iter()
        .copied()
        .find(|owner| *owner != live)
        .expect("a second enumerated unit supplies the alternate owner");

    let alternative = |occurrence| DynamicConstructorAlternativeV1 {
        tag: 0,
        constructor: symbols.resource_host_io.clone(),
        identity: test_synthesized_constructor_identity(),
        occurrence,
        fields: Vec::new(),
    };

    // ── POSITIVE: the real occurrences under the live owner agree ──
    let carried = population
        .iter()
        .enumerate()
        .map(|(index, role)| {
            let occurrence = plan
                .synthesized_aggregate_occurrence(
                    live,
                    seat,
                    &error_root.alternative(index as u32),
                    *role,
                )
                .expect("every planned alternative has a record under the live owner");
            alternative(Some(occurrence))
        })
        .collect::<Vec<_>>();
    let mut builder_context = FunctionBuilderContext::new();
    let mut function = Function::new();
    let mut builder = FunctionBuilder::new(&mut function, &mut builder_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    let discriminator = builder.ins().iconst(types::I64, 0);
    assert!(
        compiler
            .dynamic_alternatives_agree(
                live,
                seat,
                &error_root,
                &DynamicConstructorV1 {
                    discriminator,
                    alternatives: carried,
                },
            )
            .expect("the live owner's lookup is answerable"),
        "alternatives carrying the planner's own occurrences must agree, or \
         the negative below is not discriminating"
    );

    // ── NEGATIVE: an owner with no records here, alternatives carrying None ──
    //
    // The population still resolves — it is read from the tree, which has no
    // owner — and the cardinality still matches, so nothing but the
    // per-alternative record lookup can catch this. Under the predecessor
    // `.ok()` the expectation became `None`, compared equal to the emitted
    // `None`, and all ten alternatives agreed.
    let refused = compiler.dynamic_alternatives_agree(
        absent,
        seat,
        &error_root,
        &DynamicConstructorV1 {
            discriminator,
            alternatives: (0..population.len()).map(|_| alternative(None)).collect(),
        },
    );
    assert!(
        refused.is_err(),
        "a missing per-alternative record must refuse, not compare equal to an \
         alternative carrying no occurrence"
    );
}

/// A program that reaches all four governed allocation sites, and that carries
/// TWO live effect seats of the same operation for the A/B row.
///
/// ⚠ One fixture rather than four, deliberately. Four fixtures would let a row
/// pass because its own fixture happened to avoid the other three sites, and
/// the hit counter could not tell the difference. With one program every row
/// runs against the same emission and the counter is the only thing that
/// distinguishes them.
///
/// ⭐ Built on the shape `d7_fs_write_at_carrier_fixture` already established:
/// a host effect only reaches the aggregate arm when it is CARRIED, which means
/// crossing a generated-unit boundary as a call argument. Matching a host
/// result directly keeps it specialized and it never reaches the carrier at
/// all.
fn d7_governed_sites_program(symbols: &crate::NativeProcessSymbols) -> RuntimeExpr {
    let exit_success = || RuntimeExpr::Construct {
        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
        args: Vec::new(),
    };
    let trap = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "D7 governed-sites default".to_string(),
    };
    let allocate = || RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::BufferAllocate,
        capability: None,
        args: vec![RuntimeExpr::Value(RuntimeValue::Int((8).into()))],
    };
    // `origin` and `target` are the two bound buffer resources, named at
    // whatever de Bruijn depth the caller sits at.
    let write = |origin: u32, target: u32| RuntimeExpr::Effect {
        family: "FS".to_string(),
        operation: ken_host::HostOpV1::FsWriteAt,
        capability: None,
        args: vec![
            RuntimeExpr::Var(origin),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Var(target),
            RuntimeExpr::Value(RuntimeValue::Int((0).into())),
            RuntimeExpr::Value(RuntimeValue::Int((4).into())),
            RuntimeExpr::Var(origin),
        ],
    };
    let bind = |body: RuntimeExpr| RuntimeExpr::Match {
        scrutinee: Box::new(allocate()),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: symbols.result_err.clone(),
                binders: 1,
                body: exit_success(),
            },
            crate::RuntimeMatchCase {
                constructor: symbols.result_ok.clone(),
                binders: 1,
                body,
            },
        ],
        default: trap(),
    };
    // Depth bookkeeping, stated because a wrong index is a silently different
    // fixture rather than an error: inside `bind(bind(_))` the inner buffer is
    // `Var(0)` and the outer is `Var(1)`; each enclosing `Let` shifts both.
    bind(bind(RuntimeExpr::Let {
        value: Box::new(host_result_closure_match(write(1, 0))),
        body: Box::new(host_result_closure_match(write(2, 1))),
    }))
}

/// ⚠ The artifact is DISCARDED. These rows only ever ask whether one was
/// defined at all, and keeping it would leak a backend type into the fixture
/// for nothing.
fn d7_compile_governed_sites(program: &RuntimeExpr) -> Result<(), CraneliftBackendError> {
    emit_process_entrypoint_object_with_cranelift(program, "ken_d7_governed_sites").map(|_| ())
}

/// A source `Record` with DISTINGUISHABLE fields, passed as an ordinary
/// generated-unit call argument.
///
/// ⭐ The two fields are different constructors on purpose. A record whose
/// fields are interchangeable cannot show that the right field identities
/// travelled; this one can.
fn d7_record_as_call_argument() -> RuntimeExpr {
    RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["record".to_string()],
            body: Box::new(RuntimeExpr::Construct {
                constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                args: Vec::new(),
            }),
        }),
        args: vec![RuntimeExpr::Record {
            fields: vec![
                (
                    "first".to_string(),
                    RuntimeExpr::Construct {
                        constructor: "ctor:fixture::CallInput::First".to_string(),
                        args: Vec::new(),
                    },
                ),
                (
                    "second".to_string(),
                    RuntimeExpr::Construct {
                        constructor: "ctor:fixture::CallInput::Second".to_string(),
                        args: Vec::new(),
                    },
                ),
            ],
        }],
    }
}

/// **`D7` — a source `Record` passed as an ordinary generated-unit call
/// argument is marshalled at ITS OWN caller-side occurrence.**
///
/// MEASURED, by instrumenting `source_aggregate_occurrence` on this exact
/// fixture before the repair. The planned population is
///
/// | occurrence | shape | producer |
/// |---|---|---|
/// | 0 | `Constructor` | origin 2 — `CallInput::First` |
/// | 1 | `Constructor` | origin 3 — `CallInput::Second` |
/// | 2 | **`Record`** | **origin 4 — the record itself** |
/// | 3 | `Constructor` | origin 6 — the callee closure's `ExitCode::Success` body |
///
/// and the emitter asked for the `Record`'s ownership record **at origin 6**,
/// found occurrence 3, and refused on the shape cross-check. Origin 6 is
/// `DeclaredUnitCall::origin` — the callee's scheduling entry — because
/// `call_declared_unit_target` transferred every input still specialized at
/// that one coordinate.
///
/// ⭐ **This row was committed asserting that refusal**, as
/// `..._is_marshalled_at_the_callee_origin`, so the repair had a discriminator
/// to flip rather than a claim to make. The rename and the flipped assertion
/// are that flip.
///
/// ⛔ The shape check was never what was wrong. It was correct and doing its
/// job; the coordinate it was asked about was wrong. Nothing in the repair
/// weakens it, accepts both shapes, or defaults an occurrence — the caller-side
/// occurrence `child_occurrence` had already issued is simply carried through
/// to the transfer instead of being discarded.
///
/// THE GAP: a successful compile shows the coordinate resolves and the shape
/// agrees. It does not show the field identities themselves round-trip — that
/// is `c1_d4_ac_c7`'s differential, which reads each projected child's runtime
/// identity back.
#[test]
fn a_source_record_as_a_call_argument_is_marshalled_at_its_own_occurrence() {
    emit_process_entrypoint_object_with_cranelift(
        &d7_record_as_call_argument(),
        "ken_d7_record_call_input",
    )
    .expect("a record call argument resolves its ownership record at its own occurrence");
}

/// The source-`Record` site, driven by an ordinary program.
///
/// ⭐ This is the real route: a source `Record` passed as a generated-unit call
/// argument, the same fixture
/// [`a_source_record_as_a_call_argument_is_marshalled_at_its_own_occurrence`]
/// drives. Before the call-input coordinate was repaired no ordinary program
/// reached `emit_carrier_transfer`'s `Lowered::Record` arm at all — the
/// transfer resolved the ownership record at the callee's scheduling entry and
/// was refused on the shape cross-check first — so this row was driven at the
/// lower-level carrier edge instead. That edge control still exists on its own
/// (`c1_d4_ac_c7`); this row no longer needs it.
fn d7_transfer_a_carried_record() -> Result<(), CraneliftBackendError> {
    emit_process_entrypoint_object_with_cranelift(
        &d7_record_as_call_argument(),
        "ken_d7_record_site_bypass",
    )
    .map(|_| ())
}

/// The carried-constructor site, driven by the heterogeneous eliminator.
///
/// ⚠ A third driver, and measured for the same reason as the second.
/// `transfer_constructor_operands` builds a constructor from ALREADY-LOWERED
/// operands; the host-effect fixture never does, because its constructors are
/// built from source. The heterogeneous eliminator composes a dynamic producer
/// through two ordinary frames, which is the shape that carries operands into
/// a constructor at the process boundary.
fn d7_transfer_carried_constructor_operands() -> Result<(), CraneliftBackendError> {
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
        "ken_d7_carried_constructor_operands",
    )
    .map(|_| ())
}

/// **`D7` — each of the four governed sites reaches the choke, and cannot
/// bypass it.**
///
/// MEASURED, one fixture, one variant installed at a time:
///
/// | site | driver | hits | outcome |
/// |---|---|---|---|
/// | source `Constructor` | host-effect object | > 0 | refused at the choke's non-aggregate check |
/// | source `Record` | record-as-call-argument | > 0 | refused at the choke's non-aggregate check |
/// | selected dynamic alternative | host-effect object | > 0 | refused at the choke's non-aggregate check |
/// | carried `transfer_constructor_operands` | heterogeneous eliminator | > 0 | refused at the choke's non-aggregate check |
///
/// ⚠ **Three drivers, not one, and that is measured rather than chosen.** I
/// wrote this as a single fixture first, on the argument that one emission
/// makes the counter the only thing separating the rows. The counter falsified
/// the argument twice, and both findings turned out to be about the emitter
/// rather than about the fixture: no ordinary program reached the
/// source-`Record` arm, because the call-input coordinate resolved its
/// ownership record at the callee's scheduling entry — since repaired, so that
/// row is now an ordinary program — and nothing in a host-effect program builds
/// a constructor from already-lowered operands, so that row is the
/// heterogeneous eliminator.
///
/// ⭐ **The hit count is not decoration.** Without it a row is green whenever
/// its driver fails to reach the site — a vacuous pass that looks exactly like
/// a caught bypass, because the assertion it satisfies is "this did not
/// compile". The counter is what makes each row a statement about ITS site,
/// and it is what caught the first spelling of this fixture, which reached
/// three sites and silently skipped the fourth.
///
/// ⭐ Each unmutated baseline runs first for the same reason in the other
/// direction: without it, four refusals could all be some unrelated defect in
/// the driver rather than the perturbation.
///
/// CLAIMED: the emitter's four governed allocation sites all arrive at
/// `emit_carrier_alloc` carrying a `PlannedAggregate` request, and a
/// `NonAggregate` one substituted at any of them is refused **before the raw
/// allocation and before any artifact is defined** — the compile returns `Err`,
/// so there is no module.
///
/// THE GAP: this says nothing about the ORDER of the refusal within the choke.
/// That is
/// `the_carrier_allocation_choke_refuses_both_ungoverned_requests`'s positive
/// control, which reaches the allocation on a rig that has none.
#[test]
fn each_governed_site_reaches_the_choke_and_cannot_bypass_it() {
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let program = d7_governed_sites_program(&symbols);
    let drive = |site| match site {
        GovernedAllocationSite::SourceRecord => d7_transfer_a_carried_record(),
        GovernedAllocationSite::CarriedConstructor => d7_transfer_carried_constructor_operands(),
        _ => d7_compile_governed_sites(&program),
    };

    for site in [
        GovernedAllocationSite::SourceConstructor,
        GovernedAllocationSite::SourceRecord,
        GovernedAllocationSite::DynamicAlternative,
        GovernedAllocationSite::CarriedConstructor,
    ] {
        drive(site).unwrap_or_else(|error| {
            panic!("{site:?}: this row's driver must compile unmutated, got {error:?}")
        });

        let guard =
            GovernedAllocationMutationGuard::install(GovernedAllocationMutation::Bypass(site));
        let result = drive(site);
        let hits = guard.hits();
        drop(guard);

        assert!(
            hits > 0,
            "{site:?}: the fixture never reached this site, so its row is vacuous"
        );
        let Err(error) = result else {
            panic!("{site:?}: a bypassed request must refuse, and no artifact may be defined");
        };
        assert!(
            format!("{error:?}").contains("was allocated as non-aggregate"),
            "{site:?}: must refuse at the choke's own pre-allocation check, got {error:?}"
        );
    }
}

/// **`D7` — a SIBLING effect seat's coordinate is rejected at construction.**
///
/// MEASURED: the fixture carries two live `FsWriteAt` seats. Because they run
/// the same operation they share every synthesized role, path and shape, so the
/// only thing that differs between them is which occurrence is being lowered.
/// With the emitted construction and the seat's own operands retained
/// unchanged, selecting the planned occurrence and record at the SIBLING is
/// refused before `emit_carrier_alloc`.
///
/// ⭐ **B is a live effect seat, never an invalid or non-`Effect` origin.** A
/// refusal driven by an unusable seat would be a claim about seat VALIDITY —
/// much weaker, and satisfiable by a check that merely resolves the origin. The
/// claim here is that the seat coordinate is *load-bearing between two seats
/// that are equally real and structurally identical*.
///
/// ⭐ The hit count is asserted because `sibling_effect_seat` returns the seat
/// unchanged when no sibling exists. Without it, "this fixture has one seat so
/// nothing was swapped" and "the swap happened and was caught" are the same
/// green.
///
/// CLAIMED: the seat is a real coordinate of the record lookup, not a
/// pass-through. This retires the row-12 seat cell from residual to held.
#[test]
fn a_sibling_effect_seats_coordinate_is_rejected_at_construction() {
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let program = d7_governed_sites_program(&symbols);
    let (plan, _) = planned_root_occurrence(&program);
    let seat = first_effect_seat(&plan).expect("the fixture has an effect seat");
    let sibling = plan
        .sibling_effect_seat(seat)
        .expect("the fixture has a SECOND live effect seat of the same operation");
    assert_ne!(seat, sibling, "an A/B discriminator needs two distinct seats");

    d7_compile_governed_sites(&program).expect("the fixture compiles at its own seats");

    let guard =
        GovernedAllocationMutationGuard::install(GovernedAllocationMutation::SiblingEffectSeat);
    let result = d7_compile_governed_sites(&program);
    let hits = guard.hits();
    drop(guard);

    assert!(
        hits > 0,
        "the sibling swap never fired, so this row says nothing about the seat"
    );
    let Err(error) = result else {
        panic!(
            "a sibling seat's coordinate was ACCEPTED after {hits} swaps. Stated as measured: \
             the seat is not load-bearing at the sites this fixture reaches, and row 12's seat \
             cell stays a residual"
        );
    };
    // MEASURED: it refuses at `reconcile_declared_children`'s nested-`Fixed`
    // comparison — the child carries the occurrence the planner issued at seat
    // A and the expected one is B's, so the two disagree. That is a
    // CONSTRUCTION refusal, reached before any request is built and therefore
    // before `emit_carrier_alloc`; the choke's own message would mean the seat
    // had been carried all the way to the allocation.
    let reason = format!("{error:?}");
    assert!(
        !reason.contains("was allocated as non-aggregate"),
        "the refusal must come from CONSTRUCTION, before emit_carrier_alloc: {reason}"
    );
    assert!(
        reason.contains("the meet was taken over a different node than the one being allocated"),
        "the refusal must be the parent-to-child record disagreement the seat \
         coordinate produces: {reason}"
    );
}

/// **`D7` — the deepest carrier allocator refuses BOTH ungoverned requests, and
/// refuses them before it emits anything.**
///
/// MEASURED, on a bare rig whose function carries no boundary-carrier refs —
/// which is exactly what makes the ORDER observable:
///
/// | request | class | outcome |
/// |---|---|---|
/// | `NonAggregate` | `Constructor` | refused, naming the class |
/// | `NonAggregate` | `Record` | refused, naming the class |
/// | `PlannedAggregate { Constructor }` | `Record` | refused, naming the shape's class |
/// | `PlannedAggregate { Constructor }` | `Constructor` | reaches the allocation and dies THERE |
///
/// ⭐ **The last row is the positive control and it is the load-bearing one.** A
/// blanket refusal would satisfy the first three exactly as well; only a request
/// that gets *past* both checks and fails at `carrier_refs` shows the checks
/// discriminate rather than wall. It is also what witnesses the ordering the
/// correction asks for: on this rig there is no `alloc` to reach, so a refusal
/// that arrived after the raw call could not be observed at all — and the
/// agreeing request proves the raw call is genuinely downstream of both.
///
/// CLAIMED: an aggregate class cannot be allocated without naming a planned
/// record, and a planned record cannot be allocated at a class its shape does
/// not authorize. Neither refusal can be reached after the arena has moved.
///
/// THE GAP: this row is about the CHOKE, on hand-built requests. That the
/// emitter's four governed sites actually ARRIVE here carrying a
/// `PlannedAggregate` request is a different claim, and nothing below asserts
/// it — it is held by
/// `each_governed_site_reaches_the_choke_and_cannot_bypass_it`.
#[test]
fn the_carrier_allocation_choke_refuses_both_ungoverned_requests() {
    let source = RuntimeExpr::Construct {
        constructor: "ctor:fixture::Choke::Outer".to_string(),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:fixture::Choke::Inner".to_string(),
            args: Vec::new(),
        }],
    };
    let (plan, _) = planned_root_occurrence(&source);
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);
    // Found by SHAPE rather than by position, so the agreeing row below cannot
    // fail at the planner's own shape cross-check and be misread as the class
    // check biting.
    let occurrence = compiler
        .static_transition_plan
        .aggregate_ownership_records()
        .iter()
        .find(|record| record.shape == PlannedAggregateShape::Constructor)
        .expect("the fixture plans a constructor-shaped record")
        .id;

    let mut builder_context = FunctionBuilderContext::new();
    let mut function = Function::new();
    let mut builder = FunctionBuilder::new(&mut function, &mut builder_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);

    for class in [BoundaryClass::Constructor, BoundaryClass::Record] {
        let error = compiler
            .emit_carrier_alloc(
                &mut builder,
                CarrierAllocationRequest::NonAggregate {
                    tag: BoundaryTag::PersistentGround,
                },
                class,
                0,
            )
            .expect_err("an aggregate class may not be allocated ungoverned");
        assert!(
            format!("{error:?}").contains("non-aggregate"),
            "the refusal must name the ungoverned request: {error:?}"
        );
    }

    let error = compiler
        .emit_carrier_alloc(
            &mut builder,
            CarrierAllocationRequest::PlannedAggregate {
                occurrence,
                shape: PlannedAggregateShape::Constructor,
            },
            BoundaryClass::Record,
            0,
        )
        .expect_err("a constructor-shaped record may not be carried as a Record");
    assert!(
        format!("{error:?}").contains("rather than"),
        "the refusal must name the class the shape authorizes: {error:?}"
    );

    // ── POSITIVE CONTROL — past both checks, dead at the allocation ──
    let error = compiler
        .emit_carrier_alloc(
            &mut builder,
            CarrierAllocationRequest::PlannedAggregate {
                occurrence,
                shape: PlannedAggregateShape::Constructor,
            },
            BoundaryClass::Constructor,
            0,
        )
        .expect_err("this rig has no boundary-carrier helper refs to allocate through");
    assert!(
        format!("{error:?}").contains("boundary-carrier helper refs"),
        "an agreeing request must reach the ALLOCATION and fail there, not at \
         either request check: {error:?}"
    );
}

/// **`D7` — the aggregate allocation relation's laws, driven at the ledger.**
///
/// MEASURED, as the numbered control matrix:
///
/// | # | case | verdict |
/// |---|---|---|
/// | 1 | same numeric `Value` under distinct `FuncId`s | lawful |
/// | 2 | one planner record governing several events | lawful |
/// | 3 | an unused planned record, closeout still succeeds | lawful |
/// | 4 | duplicate/conflicting pairing | rejects |
/// | 5 | event recorded, relation suppressed | rejects at local close |
/// | 6 | relation entry with no event | rejects at local close |
/// | 7 | committed relation entries cleared between bodies | rejects at close |
/// | 8 | a body commit discarded | rejects via opened-vs-committed |
/// | 9 | second build/commit of one `FuncId` | rejects |
/// | 10 | related occurrence absent from `P` | rejects |
/// | 11 | allocation with no open body | rejects |
///
/// CLAIMED: `dom(R) = E` exactly, `R` single-valued with unique pairs, and
/// `image(R) ⊆ P` — with `P` authorizing rather than obliging, so rows 2 and 3
/// are the lawful shapes that a surjectivity requirement would have refused.
///
/// THE GAP: rows 12 and 13 of the released matrix are NOT here, and neither is
/// inherited by gesture. This row is about the LEDGER, and driving it there is
/// what lets rows 5 and 6 exist at all: they are only stateable because the
/// event set and the relation are separate evidence.
///
/// **Row 12 — wrong owner/seat/path/role/shape/lane/child — is inherited only
/// through this explicit axis mapping:**
///
/// | axis | held by | where |
/// |---|---|---|
/// | owner | `a_dynamic_alternative_with_no_planned_record_refuses` | a second ENUMERATED unit's owner, at which the seat has no records |
/// | seat | `a_sibling_effect_seats_coordinate_is_rejected_at_construction` | a second LIVE effect seat of the same operation |
/// | path | `a_records_lookup_key_includes_its_path_not_only_its_role`, `a_path_step_kind_is_load_bearing_not_an_index` | planner-side, both directions of the key |
/// | role | `a_repeated_role_at_one_seat_gets_distinct_real_records` | one role twice at one seat gets two records |
/// | shape | `the_carrier_allocation_choke_refuses_both_ungoverned_requests` | class-vs-shape at the choke; `aggregate_allocation_at` cross-checks the record's own shape beneath it |
/// | lane | structural — see below | no test, and none is possible |
/// | child | `a_construction_time_occurrence_lookup_fails_closed`, nested `Fixed` row | the child's own path is resolved, not the parent's |
///
/// ⭐ **The SEAT cell was a residual and is not one any more.** I had recorded
/// that a wrong seat was not falsifiable, reasoning that producing a different
/// seat would mean lowering a different expression and every consumer would
/// then agree with it consistently. That was wrong in a specific way: it
/// treated the seat as travelling with the whole construction, when the
/// construction and its operands can be RETAINED while only the coordinate used
/// to select the planned record moves. Two live effect seats of the same
/// operation share every role, path and shape, so the swap is an A/B in which
/// nothing else differs — and it is refused at
/// `reconcile_declared_children`, before any allocation request exists.
///
/// ⛔ **The LANE cell is a type-and-API property, not an unattainable test.**
/// `CarrierAllocationRequest::PlannedAggregate` carries no lane and offers no
/// field for one; the lane is derived inside the choke from
/// `aggregate_allocation_at(occurrence, shape)`. There is no caller input that
/// could be wrong, so "a wrong lane" is not a state the emitter can express —
/// which is a stronger guarantee than a test, not a weaker one. A test would
/// have to construct the unconstructible to fail.
///
/// **Row 13 is the four-site bypass matrix**, and it is committed rather than
/// recorded in a commit message:
/// `each_governed_site_reaches_the_choke_and_cannot_bypass_it` installs a
/// closed `#[cfg(test)]` mutation at one governed site's request construction
/// at a time, proves the site was actually reached with a hit counter, and
/// asserts the choke's own pre-allocation refusal. The choke's two refusals are
/// asserted separately in
/// `the_carrier_allocation_choke_refuses_both_ungoverned_requests`.
#[test]
fn the_aggregate_allocation_relation_holds_its_laws() {
    // Two nested constructors, so the population has two records: one is used
    // below and the other is left unused, which is what row 3 needs.
    let source = RuntimeExpr::Construct {
        constructor: "ctor:fixture::Relation::Outer".to_string(),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:fixture::Relation::Inner".to_string(),
            args: Vec::new(),
        }],
    };
    let (plan, _) = planned_root_occurrence(&source);
    let population = plan.aggregate_ownership_records();
    assert!(
        population.len() > 1,
        "this row needs at least two planned records; got {}",
        population.len()
    );
    let first = population[0].id;
    let second = population[1].id;
    let planned = population.to_vec();

    // ── 1. The same numeric Value under distinct FuncIds is two events ──
    //
    // A CLIF `Value` is numbered per function, so this is the ordinary case,
    // not an edge one. A ledger keyed on the value alone refused six lawful
    // allocations before `FuncId` scoped them.
    // ⚠ `from_u32` is a TEST-only way to name an identity. Production never
    // constructs one: it carries the `FuncId` the module handed it and the
    // `Value` the allocation produced.
    let func = FuncId::from_u32;
    let val = cranelift_codegen::ir::Value::from_u32;

    let mut ledger = AggregateAllocationLedger::default();
    for function in [func(7), func(9)] {
        ledger.open(function).expect("each body opens");
        ledger.record_event(function, val(12)).expect("value 12 in each");
        // ── 2. ONE record governing several events is lawful ──
        ledger
            .relate(function, val(12), first)
            .expect("the same record governs both");
        ledger.commit().expect("each body commits");
    }
    let closure = ledger.close(&planned).expect("the relation closes");
    assert_eq!(closure.events, 2, "same value, two functions, two events");
    assert_eq!(closure.image, 1, "one record governed both");
    // ── 3. Unused planned records are LAWFUL ──
    assert!(
        closure.unused > 0,
        "this fixture must leave a planned record unused, or row 3 is vacuous"
    );

    // ── 4. BOTH a duplicate and a conflicting pair reject ──
    //
    // ⭐ Two assertions, not one. `relation.insert` returning `Some` is the
    // single condition behind both, but "the wrapper ran twice for one
    // allocation" and "two allocations share a result value" are different
    // defects, and a spelling that rejected only a CHANGED occurrence would
    // satisfy the conflicting half while letting the duplicate half through.
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.record_event(func(1), val(3)).unwrap();
    ledger.relate(func(1), val(3), first).unwrap();
    assert!(
        ledger.relate(func(1), val(3), first).is_err(),
        "a second pair at one event to the SAME record is a duplicate"
    );
    assert!(
        ledger.relate(func(1), val(3), second).is_err(),
        "a second pair at one event to a DIFFERENT record is a conflict"
    );

    // ── 5. An event recorded but never related rejects at the local close ──
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.record_event(func(1), val(3)).unwrap();
    assert!(
        ledger.commit().is_err(),
        "an allocation nothing authorized must not commit"
    );

    // ── 6. A relation entry with no event rejects at the local close ──
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.relate(func(1), val(3), first).unwrap();
    assert!(
        ledger.commit().is_err(),
        "an authorization nothing allocated must not commit"
    );

    // ── 7. Clearing committed relation entries between bodies rejects ──
    //
    // The event evidence remains, so only the two-sided comparison sees it.
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.record_event(func(1), val(3)).unwrap();
    ledger.relate(func(1), val(3), first).unwrap();
    ledger.commit().unwrap();
    ledger.clear_committed_relation_for_tests();
    assert!(
        ledger.close(&planned).is_err(),
        "cleared relation entries leave dom(R) short of E"
    );

    // ── 8. A discarded body commit rejects via opened-vs-committed ──
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.record_event(func(1), val(3)).unwrap();
    ledger.relate(func(1), val(3), first).unwrap();
    ledger.discard_open_body_for_tests();
    assert!(
        ledger.close(&planned).is_err(),
        "a body that opened and never committed must not pass as one that \
         allocated nothing"
    );

    // ── 9. BOTH a second build and a second commit of one FuncId reject ──
    //
    // ⚠ The two are guarded independently — `open` by `committed_functions`,
    // `commit` by there being no open set — and the second assertion is what
    // shows a body cannot commit its events twice even if the open guard were
    // relaxed. It is the reachable spelling of a double commit: the
    // `committed_functions` guard inside `commit` sits behind the `open` guard
    // and cannot be reached while that one holds.
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.commit().unwrap();
    assert!(
        ledger.open(func(1)).is_err(),
        "a second build of one FuncId rejects"
    );
    assert!(
        ledger.commit().is_err(),
        "a second commit of one FuncId has no open event set to commit"
    );

    // ── 10. A related occurrence absent from P rejects ──
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    ledger.record_event(func(1), val(3)).unwrap();
    ledger.relate(func(1), val(3), second).unwrap();
    ledger.commit().unwrap();
    assert!(
        ledger.close(&planned[..1]).is_err(),
        "an event related to a record outside P must reject"
    );

    // ── 11. An allocation with no open body rejects ──
    let mut ledger = AggregateAllocationLedger::default();
    assert!(
        ledger.record_event(func(1), val(3)).is_err(),
        "an allocation with no open body belongs to no function's event set"
    );

    // And an open body left open at the whole-pass close rejects.
    let mut ledger = AggregateAllocationLedger::default();
    ledger.open(func(1)).unwrap();
    assert!(
        ledger.close(&planned).is_err(),
        "a body still open at the closeout never committed its events"
    );
}



// ─── `D7` — the A/B aggregate-ownership controls ────────────────────────────

/// The declared two-parameter callee the ownership controls call.
///
/// ⭐⭐ **A `DeclarationRef`, and that is measured rather than stylistic.** The
/// source machine's multi-argument crossing arm is reached only by a
/// `Lowered::DeclarationClosure` under `FunctionizedUnits`; a `LexicalClosure`
/// callee takes the continuing-evaluation arm and never crosses a unit boundary
/// at all, so its arguments never reach the seam these controls act at. Both
/// spellings were compiled and traced, and only this one arrives.
///
/// ⚠ Its body is a bare exit constructor. A body that did anything with its
/// parameters would put a second aggregate route between the mutation and the
/// refusal, and the row would no longer be about the arguments.
const D7_PAIR_CALLEE: &str = "fixture::d7::pair";

fn d7_pair_callee() -> RuntimeDeclaration {
    RuntimeDeclaration {
        symbol: D7_PAIR_CALLEE.to_string(),
        kind: RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["a".to_string(), "b".to_string()],
                body: Box::new(RuntimeExpr::Construct {
                    constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                    args: Vec::new(),
                }),
            },
        },
        metadata: crate::RuntimeSymbolMetadata {
            lowerability: Some(crate::RuntimeLowerabilityStatus::Supported),
            ..crate::RuntimeSymbolMetadata::empty()
        },
    }
}

fn d7_wrap(inner: &str) -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: "ctor:fixture::CallInput::Wrap".to_string(),
        args: vec![RuntimeExpr::Construct {
            constructor: inner.to_string(),
            args: Vec::new(),
        }],
    }
}

fn d7_pair_record(first: &str, second: &str) -> RuntimeExpr {
    RuntimeExpr::Record {
        fields: vec![
            (
                "first".to_string(),
                RuntimeExpr::Construct {
                    constructor: first.to_string(),
                    args: Vec::new(),
                },
            ),
            (
                "second".to_string(),
                RuntimeExpr::Construct {
                    constructor: second.to_string(),
                    args: Vec::new(),
                },
            ),
        ],
    }
}

fn d7_named_pair_record(names: [&str; 2], constructors: [&str; 2]) -> RuntimeExpr {
    RuntimeExpr::Record {
        fields: names
            .into_iter()
            .zip(constructors)
            .map(|(name, constructor)| {
                (
                    name.to_string(),
                    RuntimeExpr::Construct {
                        constructor: constructor.to_string(),
                        args: Vec::new(),
                    },
                )
            })
            .collect(),
    }
}

/// Two records with the same arity, the same field CONSTRUCTORS and therefore
/// the same shape and lane, differing in exactly one thing: their ordered field
/// NAMES.
///
/// ⭐⭐ **Differing only in the names is what makes the row about the schema.**
/// The existing record rows share `first`/`second`, so two records at different
/// origins intern the same identities and a substituted certificate is caught by
/// the per-position child occurrence instead. Here the field identities differ
/// and nothing else does, so the field-schema comparison is the only check that
/// can separate them.
fn d7_field_identity_arguments() -> RuntimeExpr {
    d7_ownership_recursor(vec![
        d7_named_pair_record(
            ["alpha", "beta"],
            [
                "ctor:fixture::CallInput::Alpha",
                "ctor:fixture::CallInput::Beta",
            ],
        ),
        d7_named_pair_record(
            ["gamma", "delta"],
            [
                "ctor:fixture::CallInput::Alpha",
                "ctor:fixture::CallInput::Beta",
            ],
        ),
    ])
}

/// A **recursor** whose case body calls the declared two-parameter unit with
/// the two aggregates given.
///
/// ⭐⭐ **`recursive_positions: vec![0]` is load-bearing.** The source machine is
/// entered only from a computational-match case that declares a recursive
/// position; with an empty list the case body is lowered by ordinary descent and
/// the call never reaches the source-machine crossing. Measured both ways: with
/// the position removed the seam records zero reaches on this exact fixture.
///
/// ⚠ The `Seed` case is the base case and must stay. Without it the match is
/// one-armed and the scrutinee's own child has nowhere to go.
fn d7_ownership_recursor(args: Vec<RuntimeExpr>) -> RuntimeExpr {
    RuntimeExpr::Let {
        value: Box::new(d7_wrap("ctor:fixture::CallInput::Seed")),
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::CallInput::Wrap".to_string(),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::DeclarationRef {
                            symbol: D7_PAIR_CALLEE.to_string(),
                        }),
                        args,
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::CallInput::Seed".to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            default: ac_c7_trap(),
        }),
    }
}

/// Two `Wrap`s with different children: same outer symbol, same shape, same
/// arity, ⭐ **distinct nested producer facts**. That combination is what makes
/// the negative below a statement about identity — everything a shape-only read
/// can see agrees.
fn d7_constructor_arguments() -> RuntimeExpr {
    d7_ownership_recursor(vec![
        d7_wrap("ctor:fixture::CallInput::Alpha"),
        d7_wrap("ctor:fixture::CallInput::Beta"),
    ])
}

/// The same design one shape over: two records with the same field names and
/// arity and different field constructors.
fn d7_record_arguments() -> RuntimeExpr {
    d7_ownership_recursor(vec![
        d7_pair_record(
            "ctor:fixture::CallInput::Alpha",
            "ctor:fixture::CallInput::Beta",
        ),
        d7_pair_record(
            "ctor:fixture::CallInput::Gamma",
            "ctor:fixture::CallInput::Delta",
        ),
    ])
}

/// Two aggregates bound by nested `Let`s and reached as `Var`s — so each
/// arrives at the call already lowered and FORWARDED rather than written in
/// argument position.
///
/// ⭐⭐ **This is the shape the producer field exists for.** A record literal in
/// argument position is transferred at a coordinate that happens to be its own,
/// so a use-coordinate lookup gets the right answer by accident. Forwarded
/// through a binder it does not, and the ownership record has to come from the
/// template or not at all.
/// The literal-constructor row's exact fixture, differing ONLY in that each
/// argument is bound and reached as a `Var`. A pair that differs in one thing
/// is what makes forwarding the attributable difference.
fn d7_forwarded_constructor_arguments() -> RuntimeExpr {
    d7_forwarded_pair(
        d7_wrap("ctor:fixture::CallInput::Alpha"),
        d7_wrap("ctor:fixture::CallInput::Beta"),
    )
}

fn d7_forwarded_pair(first: RuntimeExpr, second: RuntimeExpr) -> RuntimeExpr {
    let call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::DeclarationRef {
            symbol: D7_PAIR_CALLEE.to_string(),
        }),
        args: vec![RuntimeExpr::Var(1), RuntimeExpr::Var(0)],
    };
    let body = RuntimeExpr::Let {
        value: Box::new(first),
        body: Box::new(RuntimeExpr::Let {
            value: Box::new(second),
            body: Box::new(call),
        }),
    };
    RuntimeExpr::Let {
        value: Box::new(d7_wrap("ctor:fixture::CallInput::Seed")),
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Var(0)),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::CallInput::Wrap".to_string(),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body,
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::CallInput::Seed".to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
                        args: Vec::new(),
                    },
                },
            ],
            default: ac_c7_trap(),
        }),
    }
}

fn d7_forwarded_record_arguments() -> RuntimeExpr {
    d7_forwarded_pair(
        d7_pair_record(
            "ctor:fixture::CallInput::Alpha",
            "ctor:fixture::CallInput::Beta",
        ),
        d7_pair_record(
            "ctor:fixture::CallInput::Gamma",
            "ctor:fixture::CallInput::Delta",
        ),
    )
}

fn d7_compile_ownership(program: &RuntimeExpr) -> Result<(), CraneliftBackendError> {
    let declaration = d7_pair_callee();
    let mut declarations = BTreeMap::new();
    declarations.insert(D7_PAIR_CALLEE, &declaration);
    compile_expr_into_module(
        new_object_module("ken-runtime-process-entrypoint")?,
        "ken_d7_aggregate_ownership",
        Linkage::Export,
        program,
        &NativeSeedEnvironment::empty(),
        declarations,
        None,
        true,
        None,
        Some(crate::cranelift_backend::test_support::test_only_distinguished_root_join_plan()),
        None,
    )
    .map(|_| ())
}

/// One run under one mutation: the outcome, the seam's hit count, the raw
/// allocations it emitted, and what it substituted.
fn d7_ownership_run(
    program: &RuntimeExpr,
    mutation: GovernedAllocationMutation,
) -> (
    Result<(), CraneliftBackendError>,
    u32,
    u32,
    Option<SiblingProducerSubstitution>,
) {
    let guard = GovernedAllocationMutationGuard::install(mutation);
    let result = d7_compile_ownership(program);
    let hits = guard.hits();
    let allocations = guard.raw_allocations();
    let substitution = guard.substitution();
    drop(guard);
    (result, hits, allocations, substitution)
}

/// A **static worker** — a closure bound in a `Let` and called through its
/// binder — handed an aggregate argument.
///
/// ⭐ This is the one call-input site with no caller-side occurrence to carry:
/// `call_static_worker` computes each argument's exact origin with
/// `child_occurrence` and discards it, so the input arrives still specialized at
/// `call_declared_unit_target` and is transferred at `target.origin`, the
/// CALLEE's scheduling entry. Every other caller now carries its inputs across
/// at their own occurrences.
fn d7_static_worker_with_aggregate_argument() -> RuntimeExpr {
    RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::LexicalClosure {
                    captures: vec![RuntimeExpr::Var(0)],
                    params: vec!["y".to_string()],
                    body: Box::new(RuntimeExpr::Var(0)),
                }),
                body: Box::new(RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![d7_wrap("ctor:fixture::CallInput::Alpha")],
                }),
            }),
        }),
        args: vec![RuntimeExpr::Value(RuntimeValue::Int(10.into()))],
    }
}

/// **`D7` — an aggregate reaching the callee-scheduling-entry fallback is
/// SELF-AUTHORIZING: which live coordinate it is transferred at is inert.**
///
/// MEASURED, over the whole suite with the site instrumented
/// (`--nocapture --test-threads=1`): 89 `Constructor` parameters reach this
/// fallback and **every one of them carries a producer occurrence** — zero
/// carry `None`. Of the origin-consuming lookups reachable from there,
///
/// | consumer | reaches |
/// |---|---|
/// | `aggregate_carrier_authority`'s no-producer fallback | **0** |
/// | `constructor_symbol_identity(origin)` | **0** |
/// | `child_static_origin(origin, position)` for a constructor | **0** |
/// | `record_field_identity(origin, position)` | **0** |
/// | `child_static_origin(origin, position)` for a record | **0** |
/// | parent origin passed to a synthesized child (not a lookup) | 1 |
///
/// ⭐⭐ **So no origin-dependent aggregate lookup remains, and the release's
/// static-worker caller-origin transport is not required.** Every aggregate
/// there carries its own occurrence and its own resolved constructor identity,
/// and the whole-graph preflight has already reconciled its tree against the
/// plan using no coordinate at all.
///
/// ⛔ **A census is not a pin.** The rows above were true when measured and
/// nothing re-runs them, so this control states the same property as behaviour:
/// with the coordinate replaced by the program root's — a real, live, planned
/// occurrence, never a fabricated one — the artifact still builds.
///
/// ⭐ The hit count and the reach count are both asserted, and they answer
/// different questions. The hit count says the coordinate was actually
/// replaced; the reach count says an AGGREGATE actually arrived at the site. A
/// green run missing either is green for not having happened.
///
/// CLAIMED: the callee's scheduling entry is a pass-through for a source
/// aggregate at this site, not an authority.
///
/// THE GAP: inertness is measured for the aggregates this fixture and the suite
/// drive there. A future input arriving with no producer occurrence would take
/// `aggregate_carrier_authority`'s fallback and the coordinate would become
/// load-bearing again — which is why that fallback still exists and why
/// `an_aggregate_with_no_producer_certificate_cannot_reach_the_carrier` pins
/// the arm that refuses one at the preflight.
#[test]
fn an_aggregate_at_the_callee_scheduling_fallback_authorizes_itself() {
    let program = d7_static_worker_with_aggregate_argument();
    let compile = || {
        crate::cranelift_backend::artifact::compile_expr_for_lowering_tests(
            &program,
            &NativeSeedEnvironment::empty(),
        )
        .map(|_| ())
    };

    let baseline_guard = GovernedAllocationMutationGuard::install(GovernedAllocationMutation::None);
    let baseline = compile();
    let baseline_reaches = baseline_guard.self_authorized_fallback_reaches();
    let baseline_hits = baseline_guard.hits();
    drop(baseline_guard);
    baseline.expect("the static-worker fixture compiles at its own coordinates");
    assert_eq!(
        baseline_hits, 0,
        "no coordinate may be replaced without the mutation installed"
    );
    assert!(
        baseline_reaches > 0,
        "NON-VACUITY: a self-authorizing aggregate must actually REACH the \
         callee-scheduling fallback, or the substitution below is inert for the \
         trivial reason that nothing was there"
    );

    let guard =
        GovernedAllocationMutationGuard::install(GovernedAllocationMutation::CalleeSchedulingOrigin);
    let result = compile();
    let hits = guard.hits();
    let reaches = guard.self_authorized_fallback_reaches();
    let guard_origin_used = guard.callee_scheduling_origin_used();
    drop(guard);
    assert!(
        hits > 0,
        "the coordinate substitution never fired, so this row says nothing about \
         whether the coordinate matters"
    );
    // ⛔ The hit count proves the seam DECIDED to substitute; this proves it
    // substituted. Measured: with the seam's return value reverted to its
    // argument while the counter still fired, everything below stayed green.
    let (passed_in, used) = guard_origin_used
        .expect("a fired coordinate substitution records the pair it returned");
    assert_ne!(
        passed_in, used,
        "the substitution returned the coordinate it was given, so this row is a \
         no-op wearing a hit count"
    );
    assert_eq!(
        reaches, baseline_reaches,
        "the substitution must not change WHICH values reach the site, only the \
         coordinate they cross at"
    );
    result.unwrap_or_else(|error| {
        panic!(
            "an aggregate at the callee-scheduling fallback was refused after {hits} \
             coordinate substitutions. Stated as measured: it is NOT self-authorizing \
             there, some lookup still consumes the callee's scheduling entry, and the \
             static-worker caller-origin transport is owed after all: {error:?}"
        )
    });
}

/// **`D7` — an aggregate with NO producer certificate is refused at the
/// preflight, and the SAME aggregate carrying one gets past it.**
///
/// MEASURED, on a bare rig whose function carries no boundary-carrier refs —
/// the same device the admissibility-ordering row uses, and for the same
/// reason: with nothing to allocate into, the first thing the carrier step does
/// is fail, so where a graph stops is observable.
///
/// | template | outcome |
/// |---|---|
/// | `Constructor` with `occurrence: None` | refused, naming the missing producer |
/// | the SAME `Constructor` with its planned occurrence | past the preflight, stops at `BoundaryCarrier` |
///
/// ⭐⭐ **This is the arm the ownership controls cannot reach, and it is the one
/// a weakening would take.** Those controls substitute a certificate that is
/// PRESENT and wrong; adding a fallback for a certificate that is ABSENT leaves
/// every one of them green. Twelve evidence rigs demonstrated this arm firing
/// when the preflight landed, but that evidence was spent by the commit that
/// repaired them — a refusal proven only by the reds it once caused is not
/// pinned, because nothing re-runs it.
///
/// ⛔ The two rows differ in ONE field. If they differed in anything else the
/// comparison would be between two fixtures rather than between two answers to
/// one question.
///
/// ⚠ Promise class: **durable invariant** — a relation between two outcomes of
/// one template, not either message as a value. A fallback that answered
/// `None` from the transfer coordinate turns the first row green, which is
/// exactly the regression this exists to catch.
///
/// CLAIMED: a missing producer occurrence is a refusal and never a licence to
/// resolve ownership at wherever the value happens to be transferred.
#[test]
fn an_aggregate_with_no_producer_certificate_cannot_reach_the_carrier() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut module = new_jit_module().expect("JIT module constructs");
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("d7_missing_producer_probe", Linkage::Local, &signature)
        .expect("probe declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);

    let construct = RuntimeExpr::Construct {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
    };
    let (plan, construct_origin) = planned_root_occurrence(&construct);
    // ⛔ Resolved at the TRANSFER coordinate the rows below use, on purpose.
    // That makes the first row's refusal unattributable to the coordinate being
    // unusable: the plan can answer at exactly this origin, and the template
    // is refused anyway because it does not carry the answer itself.
    let wrap_occurrence = plan
        .source_aggregate_occurrence(construct_origin, PlannedAggregateShape::Constructor)
        .expect("the planned `Construct` has an ownership record at its own origin");
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    bind_bare_test_trap_lane(&mut compiler, &mut builder);

    // One SSA value, shared by both rows -- the child must be identical, or the
    // two rows differ in more than the certificate.
    let child = builder.ins().iconst(types::I64, 1);
    let template = |occurrence| Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        synthesized_identity: None,
        occurrence,
        args: vec![Lowered::Bool {
            value: child,
            known: Some(true),
        }],
    };

    let refused = compiler
        .transfer_into_carrier(&mut builder, construct_origin, &template(None))
        .expect_err("an aggregate carrying no producer occurrence cannot cross");
    let reason = format!("{refused:?}");
    assert!(
        reason.contains("no planner-issued producer occurrence"),
        "the refusal must name the MISSING PRODUCER, not whatever a later step \
         would have said: got {reason}"
    );

    let reached = compiler
        .transfer_into_carrier(
            &mut builder,
            construct_origin,
            &template(Some(wrap_occurrence)),
        )
        .expect_err("a fixture with no carrier refs cannot allocate");
    assert!(
        matches!(
            reached,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "BoundaryCarrier",
                ..
            })
        ),
        "NON-VACUITY: the SAME template carrying its producer occurrence must get \
         PAST the preflight and stop at the first emitted call, or the row above \
         proves nothing about the certificate: got {reached:?}"
    );
}

/// **`D7` — an aggregate's PRODUCER CERTIFICATE is load-bearing, and a sibling's
/// is refused before a single allocation is emitted.**
///
/// MEASURED, on a real two-argument declared-unit call reached through the
/// source machine, one shape per row:
///
/// | row | baseline | negative |
/// |---|---|---|
/// | two `Wrap` constructors | compiles, 6 raw allocations | 1 hit, refused, **0 raw allocations** |
/// | two records | compiles, 8 raw allocations | 1 hit, refused, **0 raw allocations** |
///
/// and in both negatives the substitution was recorded, from the live plan, as
/// `same_shape: true, same_lane: true` — so the only thing that moved was the
/// identity.
///
/// ⭐⭐ **The refusal names a NESTED child, not the outer node.** The preflight
/// checks presence, then shape, then boundary class, then arity, then the
/// carried constructor identity, and only then the children. Reaching the child
/// arm therefore *proves* that everything an outer, shape-only read can see
/// agreed. ⛔ That is exactly what the current `aggregate_allocation_at` reads —
/// identity existence plus shape — so restoring it as the only check makes this
/// negative go green. The preflight is what refuses here, and nothing else
/// would.
///
/// ⭐ **Zero raw allocations is asserted against a non-zero baseline.** On its
/// own, zero is equally consistent with "refused before allocating" and with
/// "this fixture never allocates"; the baseline row is what separates them, and
/// it is why each row runs unmutated first.
///
/// ⭐ **The hit count is asserted because the seam is a no-op on a call with
/// fewer than two arguments, on a non-specialized operand, and on two arguments
/// that already share a record.** Without it, all three of those and a caught
/// substitution are the same green. The counter increments only when the
/// occurrence actually CHANGES, so a substitution that substituted nothing
/// cannot satisfy it.
///
/// ⛔ The certificate is taken from a **live sibling argument's own template**,
/// never constructed. A hand-made identity would show that the record lookup
/// rejects nonsense; a real one shows that ownership discriminates between two
/// aggregates the plan considers equally real.
///
/// CLAIMED: a source aggregate is self-authenticating by its producer-issued
/// occurrence. A call transports the value; it does not become its producer, and
/// it cannot lend one argument's ownership to another.
///
/// THE GAP: this measures the argument position of one crossing. It says nothing
/// about a certificate substituted somewhere the source machine does not reach —
/// `call_static_worker`'s fallback is the open one, and it is measured
/// separately.
#[test]
fn a_sibling_aggregates_producer_certificate_is_refused_before_any_allocation() {
    for (label, program) in [
        ("two constructors", d7_constructor_arguments()),
        ("two records", d7_record_arguments()),
        ("two forwarded constructors", d7_forwarded_constructor_arguments()),
        ("two forwarded records", d7_forwarded_record_arguments()),
    ] {
        let (baseline, baseline_hits, baseline_allocations, _) =
            d7_ownership_run(&program, GovernedAllocationMutation::None);
        baseline.unwrap_or_else(|error| {
            panic!("{label}: the unmutated route must compile, or the negative below \
                    is measuring a broken fixture: {error:?}")
        });
        assert_eq!(
            baseline_hits, 0,
            "{label}: no substitution may fire without the mutation installed"
        );
        assert!(
            baseline_allocations > 0,
            "{label}: NON-VACUITY -- the unmutated route must actually allocate, or \
             `0 allocations` under the mutation says nothing"
        );

        let (mutated, hits, allocations, substitution) = d7_ownership_run(
            &program,
            GovernedAllocationMutation::SiblingAggregateProducer,
        );
        assert_eq!(
            hits, 1,
            "{label}: the certificate substitution must fire exactly once, or this row \
             is green for not having happened"
        );
        let substitution = substitution
            .unwrap_or_else(|| panic!("{label}: a fired substitution records what it moved"));
        assert_ne!(
            substitution.from,
            Some(substitution.to),
            "{label}: an A/B control needs two DISTINCT certificates"
        );
        assert!(
            substitution.same_shape,
            "{label}: the two records must agree on shape, or the refusal is about \
             shape rather than about ownership: {substitution:?}"
        );
        assert!(
            substitution.same_lane,
            "{label}: the two records must agree on allocation lane, or the refusal \
             is about the lane rather than about ownership: {substitution:?}"
        );
        let Err(error) = mutated else {
            panic!(
                "{label}: a sibling's producer certificate was ACCEPTED after {hits} \
                 substitution. Stated as measured: the certificate is not load-bearing \
                 on this route"
            );
        };
        assert_eq!(
            allocations, 0,
            "{label}: the whole-graph preflight must finish before the FIRST raw \
             allocation, and the baseline emitted {baseline_allocations}: got {error:?}"
        );
        let reason = format!("{error:?}");
        assert!(
            reason.contains("at that position"),
            "{label}: the refusal must be the per-position child disagreement -- an \
             outer shape, class, arity or identity refusal would mean the row never \
             exercised the nested authority a shape-only read cannot see: {reason}"
        );
    }
}

/// **`D7` — the call-USE coordinate is INERT, and that is why it cannot be
/// cited as the ownership control.**
///
/// MEASURED, on the same two fixtures: with `CallInputTransferOrigin` installed
/// every input is transferred at the program ROOT -- the maximally wrong
/// coordinate -- the seam fires, and **both programs still compile**, emitting
/// exactly their baselines' raw allocations.
///
/// ⭐⭐ **This is the positive statement of a negative result, and it is the
/// point of the row.** An aggregate's ownership record is the one its template
/// CARRIES, and its schema -- constructor symbol, record field identities -- is
/// recovered from that record rather than looked up at the coordinate. Nothing
/// on this route reads the coordinate, so moving it changes nothing any check
/// can see. ⛔ A control built on this mutation would be unbuildable rather than
/// merely unwritten: no fixture makes it refuse.
///
/// ⚠ **It is what retired the per-argument occurrence pairing.** The pairing
/// existed to give each argument its own transfer coordinate. This row is the
/// measurement that the coordinate decides nothing, so the accumulator, the
/// retained pending occurrence and the prefix template that mirrored it were
/// carrying a fact nothing read.
///
/// ⚠ The two mutations act at the SAME seam and are still different axes. That
/// is what makes the pair informative: same call, same arguments, same moment,
/// and only the one that moves the certificate is observable.
///
/// CLAIMED: use and production are distinct authorities, and only production
/// governs ownership.
#[test]
fn a_call_use_coordinate_substitution_is_inert_for_a_self_authorizing_aggregate() {
    for (label, program) in [
        ("two constructors", d7_constructor_arguments()),
        ("two records", d7_record_arguments()),
        ("forwarded constructors", d7_forwarded_constructor_arguments()),
        ("forwarded records", d7_forwarded_record_arguments()),
    ] {
        let (_, baseline_hits, baseline_allocations, _) =
            d7_ownership_run(&program, GovernedAllocationMutation::None);
        assert_eq!(baseline_hits, 0, "{label}: the baseline installs no mutation");
        assert!(
            baseline_allocations > 0,
            "{label}: the baseline must actually allocate, or 'the mutated run emits \
             the baseline's allocations' is a comparison between two zeroes"
        );

        let (result, hits, allocations, _) = d7_ownership_run(
            &program,
            GovernedAllocationMutation::CallInputTransferOrigin,
        );
        assert!(
            hits > 0,
            "{label}: the use-coordinate substitution must FIRE, or its inertness is \
             a statement about a call it never reached"
        );
        result.unwrap_or_else(|error| {
            panic!(
                "{label}: a use-coordinate substitution refused. If this ever goes red \
                 the two axes have merged, and the ownership control above is no longer \
                 measuring something the use coordinate cannot: {error:?}"
            )
        });
        assert_eq!(
            allocations, baseline_allocations,
            "{label}: an inert substitution must emit exactly the baseline's allocations"
        );
    }
}

/// The one bare-rig device the direct-API `D7` rows share.
///
/// ⭐ A function with NO boundary-carrier refs: the first thing an emitted
/// carrier step does is fail, so **where a graph stops is observable**. A row
/// that stops at `BoundaryCarrier` got past the whole-graph preflight and
/// reached the allocator; a row that stops with a preflight reason did not.
/// ⛔ That pairing is what makes "refused before any allocation" a measurement
/// rather than an assertion about a fixture that never allocates.
fn d7_bare_preflight_rig<'plan>(
    seed_env: &'plan NativeSeedEnvironment,
    plan: StaticTransitionPlan<'plan>,
    name: &str,
) -> (impl Module, cranelift_codegen::Context, Lowering<'plan>) {
    let mut module = new_jit_module().expect("JIT module constructs");
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function(name, Linkage::Local, &signature)
        .expect("probe declares");
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let compiler = bare_carrier_test_lowering(seed_env, plan);
    (module, context, compiler)
}

/// **`D7` — a mismatch below EVERY recursive container is refused, and the
/// containers are the two the walk used to stop at.**
///
/// MEASURED, on a bare rig, one row per container position. Each row is a pair
/// differing in ONE field -- the nested aggregate's carried producer occurrence:
///
/// | container position | with `occurrence: None` | with the planned occurrence |
/// |---|---|---|
/// | `HostResult.error` | refused, naming the missing producer, 0 raw allocations | reaches the allocator |
/// | `HostResult.ok` | refused, likewise | reaches the allocator |
/// | `DynamicConstructor` alternative field | refused, likewise | reaches the allocator |
///
/// ⭐⭐ **These three positions were invisible to the previous walk.** It matched
/// `Constructor` and `Record` and returned `Ok` on a `_` arm, so a host result's
/// two arms and a dynamic alternative's fields -- which are two levels down,
/// inside a `Vec` of alternative structs -- were admitted without being
/// reconciled at all. ⛔ The refusal is not "this container is unsupported": the
/// twin row proves the SAME container with a correct nested certificate walks
/// straight through to the allocation.
///
/// ⚠ Promise class: **durable invariant** -- a relation between the two answers
/// to one question at each position, never either message as a value. Restoring
/// a wildcard arm turns all three refusing rows green, which is the regression
/// this exists to catch.
///
/// CLAIMED: the preflight is total over `Lowered`, so no aggregate reaches the
/// allocator through a container the walk declined to enter.
#[test]
fn a_mismatch_below_every_recursive_container_is_refused_before_any_allocation() {
    let seed_env = NativeSeedEnvironment::empty();
    let construct = RuntimeExpr::Construct {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
    };
    let (plan, construct_origin) = planned_root_occurrence(&construct);
    let wrap_occurrence = plan
        .source_aggregate_occurrence(construct_origin, PlannedAggregateShape::Constructor)
        .expect("the planned `Construct` has an ownership record at its own origin");
    let wrap_identity = plan
        .constructor_symbol_identity(construct_origin)
        .expect("the planned `Construct` has a constructor identity");
    let (_module, mut context, mut compiler) =
        d7_bare_preflight_rig(&seed_env, plan, "d7_container_probe");

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    bind_bare_test_trap_lane(&mut compiler, &mut builder);

    // One SSA value per role, shared by every row: the rows must differ in the
    // carried occurrence and in nothing else.
    let word = builder.ins().iconst(types::I64, 1);
    let leaf = || Lowered::Bool {
        value: word,
        known: Some(true),
    };
    let nested = |occurrence| Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        synthesized_identity: None,
        occurrence,
        args: vec![leaf()],
    };
    let host_result = |error: Lowered, ok: Lowered| Lowered::HostResult {
        success: word,
        error: Box::new(error),
        ok: Box::new(ok),
        err_constructor: "ctor:fixture::C1::Err".to_string(),
        ok_constructor: "ctor:fixture::C1::Ok".to_string(),
    };
    let dynamic = |field: Lowered| {
        Lowered::DynamicConstructor(DynamicConstructorV1 {
            discriminator: word,
            alternatives: vec![DynamicConstructorAlternativeV1 {
                tag: 0,
                constructor: "ctor:fixture::C1::Wrap".to_string(),
                identity: wrap_identity,
                occurrence: Some(wrap_occurrence),
                fields: vec![field],
            }],
        })
    };

    for (label, build) in [
        (
            "HostResult.error",
            &(|occurrence| host_result(nested(occurrence), leaf()))
                as &dyn Fn(Option<AggregateOccurrenceId>) -> Lowered,
        ),
        (
            "HostResult.ok",
            &|occurrence| host_result(leaf(), nested(occurrence)),
        ),
        ("DynamicConstructor field", &|occurrence| {
            dynamic(nested(occurrence))
        }),
    ] {
        let guard = GovernedAllocationMutationGuard::install(GovernedAllocationMutation::None);
        let refused = compiler
            .transfer_into_carrier(&mut builder, construct_origin, &build(None))
            .expect_err("a nested aggregate carrying no producer occurrence cannot cross");
        let reason = format!("{refused:?}");
        assert!(
            reason.contains("no planner-issued producer occurrence"),
            "{label}: the refusal must be the PREFLIGHT's, not whatever the container \
             emission would have said next: got {reason}"
        );
        assert_eq!(
            guard.raw_allocations(),
            0,
            "{label}: the whole-graph walk must finish before the first raw allocation"
        );
        drop(guard);

        let reached = compiler
            .transfer_into_carrier(
                &mut builder,
                construct_origin,
                &build(Some(wrap_occurrence)),
            )
            .expect_err("a fixture with no carrier refs cannot allocate");
        let reached = format!("{reached:?}");
        assert!(
            !reached.contains("no planner-issued producer occurrence"),
            "NON-VACUITY ({label}): the SAME container carrying a correct nested \
             certificate must get PAST the preflight, or the row above says nothing \
             about the container: got {reached}"
        );
    }
}

/// **`D7` — a child whose possible OWNERS escape the set the meet was taken
/// over is refused, at the same outer shape, lane and arity.**
///
/// MEASURED, on a bare rig over one planned parent whose position 0 the planner
/// planned with owners `[NoReferent]` -- a child it will materialize as a native
/// scalar pair, which has no boundary node for anything to own:
///
/// | held child at position 0 | derived closed owner set | outcome |
/// |---|---|---|
/// | `Bool` (spill-free immediate) | `[NoReferent]` | reaches the allocator |
/// | `Bytes` (a handle) | `[PersistentStore]` | refused, 0 raw allocations |
///
/// ⭐⭐ **The two rows share the parent template exactly** -- same producer
/// occurrence, same constructor symbol, same arity, therefore the same planned
/// shape, class and allocation lane. The only difference is what the child can
/// be owned by, which is the fact the parent's lane was DERIVED from. A parent
/// whose lane was concluded from a set that does not contain its actual child's
/// owner is a conclusion from the wrong premises.
///
/// ⛔ Not a check that the child is "the right kind". It is a containment: the
/// held set must be inside the planned one, and the direction is the content. A
/// held child may be LONGER-lived than its position planned for -- a persistent
/// child where the meet allowed an activation-owned one dangles nothing --
/// never shorter.
///
/// ⚠ **The refusal is the SECOND of two arms at this position, and the first
/// has its own witness.** Before the containment, the planner's two fields about
/// the position are made to agree: the invocation arena is a possible owner only
/// if the position is activation-owned. Not an equivalence -- `owners` is the
/// lifetime's affinity intersected with the child's representation, so THIS
/// fixture's position 0 is legitimately `ActivationOwned` over `[NoReferent]`,
/// and stating the law as `==` refuses it. Mutating the planner to admit the
/// arena under a persistent lifetime reds the first arm; dropping the
/// containment reds this row.
///
/// ⚠ Promise class: **durable invariant** -- a relation between two answers to
/// one question, not either message.
///
/// CLAIMED: the meet's premises are checked against the operands actually held,
/// not merely recorded.
#[test]
fn a_child_owner_set_outside_the_planned_meet_is_refused_before_any_allocation() {
    let seed_env = NativeSeedEnvironment::empty();
    // ⭐ The `Call` child is what makes position 0's planned owner set the
    // singleton `[NoReferent]`: its join result is a native scalar pair, so the
    // planner records that nothing can own it. Measured on this exact fixture.
    let fixture = RuntimeExpr::Construct {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        args: vec![RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            }),
            args: Vec::new(),
        }],
    };
    let (plan, construct_origin) = planned_root_occurrence(&fixture);
    let wrap_occurrence = plan
        .source_aggregate_occurrence(construct_origin, PlannedAggregateShape::Constructor)
        .expect("the planned `Construct` has an ownership record at its own origin");
    assert_eq!(
        plan.aggregate_record_view(wrap_occurrence)
            .expect("the record is readable")
            .children()[0]
            .owners,
        vec![BoundaryReferentOwner::NoReferent],
        "the row is only about owner containment if the planned set is a strict \
         subset of what a handle child would derive"
    );
    let (_module, mut context, mut compiler) =
        d7_bare_preflight_rig(&seed_env, plan, "d7_child_owner_probe");

    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    bind_bare_test_trap_lane(&mut compiler, &mut builder);

    let word = builder.ins().iconst(types::I64, 1);
    let template = |child: Lowered| Lowered::Constructor {
        constructor: "ctor:fixture::C1::Wrap".to_string(),
        synthesized_identity: None,
        occurrence: Some(wrap_occurrence),
        args: vec![child],
    };

    let guard = GovernedAllocationMutationGuard::install(GovernedAllocationMutation::None);
    let refused = compiler
        .transfer_into_carrier(
            &mut builder,
            construct_origin,
            &template(Lowered::Bytes(vec![7, 8])),
        )
        .expect_err("a child that can be owned outside the planned set cannot cross");
    let reason = format!("{refused:?}");
    assert!(
        reason.contains("can be owned by"),
        "the refusal must be the OWNER containment, not the arity, class or \
         identity checks that precede it: got {reason}"
    );
    assert_eq!(
        guard.raw_allocations(),
        0,
        "the walk must refuse before the first raw allocation"
    );
    drop(guard);

    let reached = compiler
        .transfer_into_carrier(
            &mut builder,
            construct_origin,
            &template(Lowered::Bool {
                value: word,
                known: Some(true),
            }),
        )
        .expect_err("a fixture with no carrier refs cannot allocate");
    let reached = format!("{reached:?}");
    assert!(
        !reached.contains("can be owned by"),
        "NON-VACUITY: the SAME parent holding a child whose closed owner set IS \
         inside the planned one must get past the owner check: got {reached}"
    );
}

/// **`D7` — a record's ordered FIELD SCHEMA is load-bearing, and a sibling's is
/// refused before a single allocation is emitted.**
///
/// MEASURED, on the same real two-argument declared-unit call reached through
/// the source machine, with two records that differ in **exactly one thing** —
/// their ordered field names. Same arity, same field constructors, therefore the
/// same shape and the same ruled lane, recorded from the live plan as
/// `same_shape: true, same_lane: true`:
///
/// | run | outcome |
/// |---|---|
/// | baseline | compiles, non-zero raw allocations |
/// | B's certificate for A | 1 hit, refused naming the field identity, **0 raw allocations** |
///
/// ⭐⭐ **The refusal is the FIELD SCHEMA arm, not the child occurrence.** The
/// preflight compares presence, shape, class, arity and then, per position, the
/// field identity — before it reaches the nested occurrence. Asserting the
/// message names the identity is what distinguishes this row from the ownership
/// control it sits beside: with the same field names on both records, the
/// substitution is caught by the child occurrence and this arm is never
/// exercised.
///
/// ⛔ The identity compared is the TYPED `FieldIdentity` the producer was
/// issued, on both sides. There is no `&str -> FieldIdentity` direction and none
/// may be added — comparing the template's field strings against the plan would
/// be the second derivation `D2` forbids, and it is what left field naming
/// unreconciled while order and arity were covered.
///
/// ⚠ Promise class: **durable invariant** — a relation between a baseline and a
/// negative differing in one certificate, never a message as a value.
///
/// CLAIMED: a record's field names are its producer's fact, they travel with the
/// template, and a template carrying another producer's schema cannot be
/// emitted.
#[test]
fn a_sibling_records_field_schema_is_refused_before_any_allocation() {
    let program = d7_field_identity_arguments();
    let (baseline, baseline_hits, baseline_allocations, _) =
        d7_ownership_run(&program, GovernedAllocationMutation::None);
    baseline.expect("the two-record field-identity fixture compiles unmutated");
    assert_eq!(baseline_hits, 0, "the baseline installs no mutation");
    assert!(
        baseline_allocations > 0,
        "the baseline must actually allocate, or 'the negative emitted zero' is a \
         comparison between two zeroes"
    );

    let (result, hits, allocations, substitution) = d7_ownership_run(
        &program,
        GovernedAllocationMutation::SiblingAggregateProducer,
    );
    assert_eq!(
        hits, 1,
        "the certificate substitution must FIRE exactly once, or a refusal is not \
         attributable to it"
    );
    let substitution = substitution.expect("a firing substitution records what it moved");
    assert_ne!(
        substitution.from,
        Some(substitution.to),
        "a substitution that substituted nothing is indistinguishable from a \
         well-defended one"
    );
    assert!(
        substitution.same_shape && substitution.same_lane,
        "the two records must agree on shape and lane, or the refusal is \
         attributable to something coarser than the schema: {substitution:?}"
    );
    let error = result.expect_err("a record carrying a sibling's field schema cannot be emitted");
    assert_eq!(
        allocations, 0,
        "the whole-graph preflight must finish before the FIRST raw allocation, and \
         the baseline emitted {baseline_allocations}: got {error:?}"
    );
    let reason = format!("{error:?}");
    assert!(
        reason.contains("carries identity"),
        "the refusal must be the FIELD SCHEMA disagreement -- a shape, class, arity \
         or nested-occurrence refusal would mean the row never exercised the field \
         identities at all: {reason}"
    );
}

// -- `D7` checkpoint 1: the retained-callable capture contract ---------------
//
// The gate these exercise is the one the framed `#23` reaching row stops at.
// Its subject is a MIXED-PHASE environment, so every control below builds a
// real carried word rather than asserting about specialized captures only --
// a run whose every capture is specialized cannot distinguish a gate that
// preserves phase from one that never sees a carried capture at all.

/// One capture's intended phase, so a fixture states its environment rather
/// than deriving it.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CapturePhase {
    Specialized,
    Carried,
}

/// A capture slot the caller shapes, so each axis of the contract can be
/// perturbed on its own.
///
/// ⛔ Defaults to the descriptor the planner would actually lay. A fixture that
/// had to spell every field to get the EXACT case would make the exact case the
/// unusual one, and a control is only as trustworthy as its unmutated twin.
#[cfg(test)]
fn lexical_capture_slot(ordinal: u32) -> AbiSlot {
    AbiSlot {
        kind: AbiSlotKind::Capture,
        carrier: AbiCarrier::ValueWord,
        ownership: AbiOwnership::OwnedByFrame,
        storage_owner: AbiStorageOwner::ActivationFrame,
        width_bytes: 8,
        align_bytes: 8,
        ordinal,
    }
}

/// Drive the capture-contract gate against a descriptor and an environment the
/// caller shapes independently.
///
/// The two are separate parameters on purpose: the gate's whole claim is that
/// it relates a capture RUN to a planner-issued CONTRACT, so a fixture must be
/// able to move one while holding the other.
#[cfg(test)]
fn attempt_capture_contract(
    phases: &[CapturePhase],
    capture_slots: impl FnOnce() -> Option<Vec<AbiSlot>>,
    declared_arity: u32,
) -> Result<StaticWorkerBinding, CraneliftBackendError> {
    let source = worker_source();
    let (plan, root) = planned_root_occurrence(&source);
    let closure_origin = plan
        .child_static_origin(root, 0)
        .expect("the Let's bound value is planned as child 0");
    let body_origin = plan
        .child_static_origin(closure_origin, 0)
        .expect("a lexical closure plans its body as child 0");
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = bare_carrier_test_lowering(&seed_env, plan);
    if let Some(capture_slots) = capture_slots() {
        let mut slots = (0..declared_arity)
            .map(|ordinal| AbiSlot {
                kind: AbiSlotKind::Parameter,
                carrier: AbiCarrier::ValueWord,
                ownership: AbiOwnership::OwnedByFrame,
                storage_owner: AbiStorageOwner::ActivationFrame,
                width_bytes: 8,
                align_bytes: 8,
                ordinal,
            })
            .collect::<Vec<_>>();
        let declared_captures = capture_slots.len() as u32;
        slots.extend(capture_slots);
        let offsets = (0..slots.len() as u32).map(|index| index * 8).collect();
        compiler.function_local.worker_templates.insert(
            body_origin,
            units::WorkerTemplate {
                origin: body_origin,
                call_site_origin: body_origin,
                header: AbiFrameHeader {
                    parameters: declared_arity,
                    captures: declared_captures,
                    frame_bytes: (slots.len() as u32) * 8,
                    align_bytes: 8,
                },
                slots,
                offsets,
            },
        );
    }

    // A carried capture is an SSA word, so the environment needs a real
    // function under construction to mint one. ⛔ Standing in a specialized
    // value for it would make every control below measure the specialized path.
    let mut func = Function::with_name_signature(
        UserFuncName::user(0, 0),
        cranelift_codegen::ir::Signature::new(cranelift_codegen::isa::CallConv::SystemV),
    );
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    let captures = phases
        .iter()
        .enumerate()
        .map(|(index, phase)| match phase {
            CapturePhase::Specialized => {
                LoweringOperand::Specialized(Lowered::Bytes(format!("capture{index}").into_bytes()))
            }
            CapturePhase::Carried => LoweringOperand::Carried(CarriedBoundaryWord {
                word: builder.ins().iconst(types::I64, i64::from(index as i32)),
            }),
        })
        .collect::<Vec<_>>();

    compiler.construct_static_worker_binding(
        closure_origin,
        body_origin,
        declared_arity,
        phases.len(),
        captures,
        // `D6a` — the capture-phase contract is route-independent.
        StaticWorkerCallRoute::RawWorker,
        // `D8i` — and discharge-independent, for the same reason.
        ContinuationDischarge::DirectSpecializationCall,
    )
}

/// The positive control, and the property checkpoint 1 exists to establish: a
/// MIXED environment is representable, and each capture keeps the phase it
/// arrived at.
///
/// ⭐ The phase assertion is per-capture, not a count. "Two captures survived"
/// is satisfied by a run that specialized both.
#[test]
fn a_mixed_phase_capture_run_installs_and_keeps_each_capture_at_its_own_phase() {
    let binding = expect_worker_binding(attempt_capture_contract(
        &[
            CapturePhase::Specialized,
            CapturePhase::Carried,
            CapturePhase::Carried,
        ],
        || Some((0..3).map(lexical_capture_slot).collect()),
        1,
    ));
    assert_eq!(binding.captures.len(), 3);
    assert!(
        matches!(&binding.captures[0], LoweringOperand::Specialized(Lowered::Bytes(value))
            if value == b"capture0"),
        "the specialized capture is stored unchanged, in position"
    );
    assert!(
        matches!(&binding.captures[1], LoweringOperand::Carried(_)),
        "a carried capture stays carried rather than being refused or re-specialized"
    );
    assert!(
        matches!(&binding.captures[2], LoweringOperand::Carried(_)),
        "and so does the second one -- one surviving carried capture would not \
         show the run is phase-preserving"
    );
}

/// Omission: the planner issued no contract for this body. `worker_templates`
/// membership is the fact the gate needs, and its absence is the half a keyed
/// map cannot make unrepresentable.
#[test]
fn a_mixed_phase_capture_run_without_a_planner_issued_contract_rejects() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Specialized, CapturePhase::Carried],
        || None,
        1,
    ));
    assert!(
        format!("{error:?}").contains("no raw worker template"),
        "rejects for the omitted-contract reason: {error:?}"
    );
}

/// Order: the capture run's slots are permuted, so slot *i* no longer carries
/// ordinal *i*. Every other fact -- count, carrier, owner, lifetime -- agrees,
/// so the refusal is attributable to order alone.
#[test]
fn a_capture_contract_whose_slot_ordinals_are_permuted_rejects() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Carried, CapturePhase::Specialized],
        || Some(vec![lexical_capture_slot(1), lexical_capture_slot(0)]),
        1,
    ));
    assert!(
        format!("{error:?}").contains("provenance projects"),
        "rejects against the projected slot: {error:?}"
    );
}

/// Order, the duplication case: two slots claim ordinal 0, so one capture has
/// no slot of its own and another is named twice. A gate that only counted
/// slots would accept this.
#[test]
fn a_capture_contract_with_a_duplicated_slot_ordinal_rejects() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Carried, CapturePhase::Specialized],
        || Some(vec![lexical_capture_slot(0), lexical_capture_slot(0)]),
        1,
    ));
    assert!(
        format!("{error:?}").contains("provenance projects"),
        "rejects against the projected slot: {error:?}"
    );
}

/// Provenance / lane: the descriptor declares the SEED capture lane for a
/// callable whose captures are lexical. The capture that meets it is
/// specialized, so this isolates provenance from the phase axis below.
#[test]
fn a_capture_contract_declaring_the_seed_lane_for_a_lexical_callable_rejects() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Specialized],
        || {
            Some(vec![AbiSlot {
                carrier: AbiCarrier::GroundValueCarrier,
                ownership: AbiOwnership::BorrowedForActivation,
                storage_owner: AbiStorageOwner::ArtifactStatic,
                ..lexical_capture_slot(0)
            }])
        },
        1,
    ));
    assert!(
        format!("{error:?}").contains("provenance projects"),
        "rejects against the projected lexical slot: {error:?}"
    );
}

/// Phase / lifetime: the same seed-lane descriptor, met by a CARRIED capture.
///
/// ⭐ This is the pair that makes the phase arm reachable at all. The two
/// fixtures differ in exactly one thing -- the capture's phase -- and they must
/// produce DIFFERENT refusals: the specialized one above cannot get past slot
/// equality, and this one is stopped earlier, by the lifetime rule, because an
/// invocation-time word cannot inhabit artifact-static storage.
#[test]
fn a_carried_capture_meeting_artifact_static_storage_rejects_on_lifetime() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Carried],
        || {
            Some(vec![AbiSlot {
                carrier: AbiCarrier::GroundValueCarrier,
                ownership: AbiOwnership::BorrowedForActivation,
                storage_owner: AbiStorageOwner::ArtifactStatic,
                ..lexical_capture_slot(0)
            }])
        },
        1,
    ));
    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("arrived carried") && rendered.contains("ArtifactStatic"),
        "rejects for the lifetime reason, naming the storage owner: {rendered}"
    );
    assert!(
        !rendered.contains("provenance projects"),
        "and is stopped BEFORE slot equality -- if it fell through to that, the \
         lifetime arm would be dead code that no descriptor could reach: {rendered}"
    );
}

/// Count: the contract declares fewer capture slots than the run supplies. The
/// header and the slot run are two recorded facts, and this moves both together
/// so the refusal is about the run rather than about their disagreement.
#[test]
fn a_capture_contract_declaring_too_few_slots_rejects() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Carried, CapturePhase::Specialized],
        || Some(vec![lexical_capture_slot(0)]),
        1,
    ));
    assert!(
        format!("{error:?}").contains("captures"),
        "rejects on the capture count: {error:?}"
    );
}

/// Owner: the slot names the right carrier and the right storage owner but
/// claims the wrong TRANSFER discipline.
///
/// ⭐ Isolated from the two axes above on purpose. `AbiOwnership` and
/// `AbiStorageOwner` are deliberately distinct facts -- one is "who reclaims
/// this", the other is "borrowed from what" -- and a gate that compared only the
/// carrier would accept a descriptor promising the caller a value the frame
/// actually reclaims.
#[test]
fn a_capture_contract_claiming_the_wrong_transfer_discipline_rejects() {
    let error = expect_worker_rejection(attempt_capture_contract(
        &[CapturePhase::Carried],
        || {
            Some(vec![AbiSlot {
                ownership: AbiOwnership::TransferredToCaller,
                ..lexical_capture_slot(0)
            }])
        },
        1,
    ));
    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("provenance projects"),
        "rejects against the projected slot: {rendered}"
    );
    assert!(
        !rendered.contains("arrived carried"),
        "and NOT on the lifetime arm -- this slot's storage is still \
         activation-frame owned, so only the transfer discipline moved: {rendered}"
    );
}

/// The one fixture in this crate that reaches `lower_source_carried_match`.
///
/// ⛔ Its independent abort is JOIN ACQUISITION -- this rig's enclosing
/// continuation has no planned scalar cut -- not the `origin 264` byte-span
/// effect seat. That matters for what the control below may assert, and it is
/// not a choice: no `ken-runtime` lib fixture reaches the carried route through
/// an effect seat, and the seam is `#[cfg(test)]`, so the one fixture that does
/// (`ken-cli`'s `rt_parity_native`) links a build where the seam does not exist.
#[cfg(test)]
fn ac1_carried_route_fixture() -> Result<(), CraneliftBackendError> {
    ac_c4_recursive_edge(ac1_source_machine_carried_match(vec![
        ac1_unmatchable_case(),
        crate::RuntimeMatchCase {
            constructor: "ctor:fixture::C1::Leaf".to_string(),
            binders: 0,
            body: ac_c7_ctor("Sentinel"),
        },
    ]))
    .map(|_| ())
}

#[cfg(test)]
fn ac1_refusal_of(outcome: &Result<(), CraneliftBackendError>) -> (String, String) {
    let Err(CraneliftBackendError::Unsupported(UnsupportedLowering {
        construct, reason, ..
    })) = outcome
    else {
        panic!("the carried-route fixture refuses: got {outcome:?}");
    };
    ((*construct).to_string(), reason.clone())
}

/// The refusal this rig raises when nothing is mutated -- its INDEPENDENT abort.
#[cfg(test)]
const AC1_INDEPENDENT_ABORT: (&str, &str) = (
    "NativeJoinPlanV1",
    "active checked continuation has no planned scalar cut before its outer suffix",
);

/// `AC-1` control family 5 -- the source-machine `Match` seat DISPATCHES an
/// already-classified carried operand into the carried route, and the exact
/// pre-repair refusal comes back when that dispatch is removed.
///
/// MEASURED, three runs of ONE fixture:
/// 1. `Exact` -- the mutation fires **0** times and the fixture refuses at its
///    independent abort, [`AC1_INDEPENDENT_ABORT`].
/// 2. `RefuseClassifiedCarried` -- the mutation fires **exactly once** and the
///    fixture refuses with exactly `Match: "scrutinee is not a constructor
///    value"`, which is the pre-repair refusal, and which is raised strictly
///    BEFORE join acquisition.
/// 3. `Exact` again, after the scope -- back to **0** applications and the
///    original refusal, so the mutation left nothing behind.
///
/// ⭐ The APPLICATION COUNT is what makes run 2 evidence. A mutated run that
/// produced the right message with a count of zero would have refused for some
/// other reason entirely, and the message alone cannot tell those apart. The
/// count is taken from the hook itself, which cannot fire without incrementing.
///
/// ⛔ SCOPE -- this claims CARRIED DISPATCH BEFORE JOIN, and nothing else. It
/// does not claim selector correctness, leaf lowering, physical predecessor
/// distinction, exact-once completion, or anything about the suffix. Those are
/// decided after this rig's abort and are not measured anywhere.
///
/// ⛔ The independent abort here is join acquisition, NOT the `origin 264`
/// byte-span effect seat named in the activation gate. See
/// [`ac1_carried_route_fixture`] for why that substitution is forced rather than
/// chosen.
///
/// Promise class: durable invariant. It asserts a relation between a mutation
/// and the refusal it produces, not a snapshot of either.
#[test]
fn ac1_the_seat_dispatches_a_classified_carried_operand_before_join_acquisition() {
    let (exact, exact_applications) = with_source_carried_control_mutation(
        SourceCarriedControlMutation::Exact,
        ac1_carried_route_fixture,
    );
    assert_eq!(
        exact_applications, 0,
        "an unmutated run must not apply the mutation"
    );
    let (construct, reason) = ac1_refusal_of(&exact);
    assert_eq!(
        (construct.as_str(), reason.as_str()),
        AC1_INDEPENDENT_ABORT,
        "the unmutated fixture must refuse at its own independent abort"
    );

    let (mutated, mutated_applications) = with_source_carried_control_mutation(
        SourceCarriedControlMutation::RefuseClassifiedCarried,
        ac1_carried_route_fixture,
    );
    assert_eq!(
        mutated_applications, 1,
        "ANTI-VACUITY: the carried dispatch is taken exactly once by this \
         fixture, so the mutation must fire exactly once. Zero means the run \
         refused for a different reason and proves nothing"
    );
    let (construct, reason) = ac1_refusal_of(&mutated);
    assert_eq!(
        (construct.as_str(), reason.as_str()),
        ("Match", "scrutinee is not a constructor value"),
        "removing the carried dispatch must restore the EXACT pre-repair refusal"
    );
    assert_ne!(
        (construct.as_str(), reason.as_str()),
        AC1_INDEPENDENT_ABORT,
        "DISCRIMINATOR: the mutated refusal must REPLACE the independent abort, \
         which is what shows the dispatch is decided before it"
    );

    let (restored, restored_applications) = with_source_carried_control_mutation(
        SourceCarriedControlMutation::Exact,
        ac1_carried_route_fixture,
    );
    assert_eq!(
        restored_applications, 0,
        "the scoped mutation must not survive its scope"
    );
    assert_eq!(
        ac1_refusal_of(&restored),
        (
            AC1_INDEPENDENT_ABORT.0.to_string(),
            AC1_INDEPENDENT_ABORT.1.to_string()
        ),
        "after the scope the fixture must refuse exactly as it did before it"
    );
}

/// Why family 2a ships no control ON THE CURRENT CARRIED-ROUTE FIXTURE.
///
/// MEASURED: under `RefuseSplitInheritedJoin`, [`ac1_carried_route_fixture`]
/// applies the mutation **0** times and refuses exactly as `Exact` does. That
/// fixture's prefix split classifies its terminal as `ResumeOuter`, so the
/// inherited-join arm is not on ITS path.
///
/// ⛔ This is NOT a discharge of family 2a, and must never be read as one. It is
/// the anti-vacuity evidence for why that control is absent here: an
/// inherited-join control written against this fixture would assert a refusal it
/// did not cause.
///
/// ⛔ SCOPE -- this row is LOCAL to [`ac1_carried_route_fixture`]. It is not a
/// crate-wide `cfg(test)` census, and an earlier heading and test name said it
/// was. It measures one fixture and says nothing whatever about any other.
///
/// Promise class: LOCAL transition sentinel. This row reds if
/// [`ac1_carried_route_fixture`] itself begins reaching the inherited-Join arm.
/// It does not census other `cfg(test)` fixtures; any new reaching fixture must
/// land family 2a's gate-satisfying control.
#[test]
fn ac1_the_current_carried_route_fixture_does_not_reach_the_inherited_join_arm() {
    let (outcome, applications) = with_source_carried_control_mutation(
        SourceCarriedControlMutation::RefuseSplitInheritedJoin,
        ac1_carried_route_fixture,
    );
    assert_eq!(
        applications, 0,
        "if this is nonzero THIS FIXTURE began reaching the inherited-join arm, \
         and family 2a's gate-satisfying control can and must now be written \
         against it"
    );
    assert_eq!(
        ac1_refusal_of(&outcome),
        (
            AC1_INDEPENDENT_ABORT.0.to_string(),
            AC1_INDEPENDENT_ABORT.1.to_string()
        ),
        "with the mutation never applied THIS FIXTURE must refuse exactly as \
         an unmutated run of it does"
    );
}
