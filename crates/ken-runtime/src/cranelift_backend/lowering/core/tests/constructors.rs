//! Constructor-field, dynamic-constructor, nested-computational and
//! heterogeneous-eliminator lowering tests (RT-SPLIT §10.2 -> `constructors`).

use super::*;

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
        let lowered = compiler.lower_dynamic_constructor_match(
            &mut builder,
            dynamic,
            DynamicConstructorContinuation::Ordinary {
                cases: &cases,
                default: &default,
                env: &[],
            },
        )?;
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
