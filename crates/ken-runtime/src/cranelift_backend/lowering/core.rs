//! The indivisible lowering SCC (RT-SPLIT §10.1/§10.2).
//!
//! Moved verbatim from `cranelift_backend.rs` in RT-SPLIT slice 4; the
//! 29-method SCC plus `compile_expr_into_module`. Imports come only from
//! this module's parent, per §10.5, so slice 5 need not touch this file.

// Re-exported at facade scope so this module's `tests` subtree inherits the
// same names; a private `use` cannot be re-globbed by a descendant.
pub(in crate::cranelift_backend) use super::*;

#[cfg(test)]
mod tests;

pub(in crate::cranelift_backend) fn compile_expr_into_module<'a, M: Module>(
    mut module: M,
    function_name: &str,
    linkage: Linkage,
    expr: &RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    reset_partition_static_descriptors();
    validate_oriented_subcontinuation_transport(
        expr,
        &declarations,
        oriented_subcontinuation_plan.as_ref(),
    )?;
    let partition_source_bytes =
        partition_source_static_bytes(expr, declarations.values().copied());
    let partition_entry_cut_armed =
        process_mode && partition_entry_cut_should_arm(partition_source_bytes);
    let checked_join_count = native_join_plan.as_ref().map_or(0, |plan| plan.sites.len());
    let checked_plan_bytes = native_join_plan
        .as_ref()
        .map_or(0, |plan| plan.canonical_bytes().len())
        .saturating_add(
            oriented_subcontinuation_plan
                .as_ref()
                .map_or(0, |plan| plan.canonical_bytes().len()),
        );
    let checked_predecessor_count = oriented_subcontinuation_plan.as_ref().map_or(0, |plan| {
        plan.recursive_calls
            .len()
            .saturating_add(plan.computational_ih_calls.len())
    });
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function(function_name, linkage, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
    let mut partition_sig = module.make_signature();
    partition_sig
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    partition_sig
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    partition_sig
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    partition_sig.returns.push(AbiParam::new(types::I64));
    // A function can encounter several sequential binary fanouts before it
    // returns to the coordinator. Imports are created lazily, while function
    // IDs are replenished between work items.
    let initial_partition_helpers = if process_mode {
        PARTITION_HELPER_ID_RESERVE
    } else {
        0
    };
    let mut partition_helper_ids = Vec::with_capacity(initial_partition_helpers);
    for index in 0..initial_partition_helpers {
        let name = format!("{function_name}.__ken_partition_{index}");
        partition_helper_ids.push(
            module
                .declare_function(&name, Linkage::Local, &partition_sig)
                .map_err(|err| backend_module(err.to_string()))?,
        );
    }
    let native_int_wrapping_mutation = {
        #[cfg(test)]
        {
            NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get)
                == NativeIntLoweringMutation::Wrapping
        }
        #[cfg(not(test))]
        {
            false
        }
    };
    let native_int = crate::native_int_clif::emit_native_int_local_graph(
        &mut module,
        native_int_wrapping_mutation,
    )?;
    let host_dispatch_id = if process_mode {
        let mut host_sig = module.make_signature();
        host_sig
            .params
            .push(AbiParam::new(module.target_config().pointer_type()));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig
            .params
            .push(AbiParam::new(module.target_config().pointer_type()));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig.params.push(AbiParam::new(types::I64));
        host_sig.returns.push(AbiParam::new(types::I64));
        Some(
            module
                .declare_function("ken_host_dispatch_v1", Linkage::Import, &host_sig)
                .map_err(|err| backend_module(err.to_string()))?,
        )
    } else {
        None
    };
    let mut ctx = module.make_context();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);
    let host_dispatch = host_dispatch_id.map(|id| module.declare_func_in_func(id, &mut ctx.func));
    let int_binop = module.declare_func_in_func(native_int.binop, &mut ctx.func);
    let int_compare = module.declare_func_in_func(native_int.compare, &mut ctx.func);
    let int_intern = module.declare_func_in_func(native_int.intern, &mut ctx.func);
    let int_narrow = module.declare_func_in_func(native_int.narrow, &mut ctx.func);
    let int_export = module.declare_func_in_func(native_int.export, &mut ctx.func);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        declarations,
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
        native_join_plan,
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: None,
        oriented_subcontinuation_plan,
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
        process_object: process_mode,
        process_symbols: process_symbols
            .cloned()
            .unwrap_or_else(crate::NativeProcessSymbols::legacy_prelude),
        host_dispatch,
        invocation_pointer: None,
        native_int_arena: None,
        native_int_binop: Some(int_binop),
        native_int_compare: Some(int_compare),
        native_int_intern: Some(int_intern),
        native_int_narrow: Some(int_narrow),
        native_int_export: Some(int_export),
        native_int_tags: BTreeMap::new(),
        partition_helper_ids,
        partition_signature: partition_sig.clone(),
        partition_next_helper: 0,
        partition_queue: VecDeque::new(),
        partition_continuations: PartitionContinuationInterner::default(),
        partition_source_nodes: PartitionSourceNodeInterner::default(),
        partition_recursor_nodes: PartitionRecursorNodeInterner::default(),
        partition_recursor_qualifications: PartitionRecursorQualificationNodeInterner::default(),
        partition_open_control_obligations: PartitionOpenControlObligationNodeInterner::default(),
        partition_cleanup_suffixes: PartitionCleanupSuffixInterner::default(),
        partition_cleanup_transitions: PartitionCleanupTransitionLedger::default(),
        partition_cut_armed: partition_entry_cut_armed,
        partition_budget: active_partition_budget(),
        partition_measures: Vec::new(),
        partition_metrics: PartitionCompilationMetrics::default(),
        partition_next_site: 0,
        partition_branch_returns: PartitionBranchReturnLedger::default(),
        partition_producer_sites: BTreeMap::new(),
        partition_producer_site_interner: PartitionProducerKontSiteInterner::default(),
        partition_next_producer_site: 0,
        active_partition_producer_kont: None,
        active_partition_return_kind: None,
        active_partition_return_contract: None,
        partition_output_tag_pointer: None,
        partition_live_growth_ticks: 0,
        partition_join_site_union: BTreeSet::new(),
        partition_subcontinuation_frame_union: BTreeSet::new(),
        partition_recursive_call_template_union: BTreeSet::new(),
        #[cfg(test)]
        native_int_mutation: NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get),
        #[cfg(test)]
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let (mut maybe_trap, mut decoder) = {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.append_block_params_for_function_params(block);
        builder.switch_to_block(block);
        let invocation = builder.block_params(block)[0];
        compiler.native_int_arena = Some(invocation);
        let mut initial_env = Vec::new();
        if process_mode {
            compiler.invocation_pointer = Some(invocation);
            let pointer_type = builder.func.dfg.value_type(invocation);
            let process_input =
                builder
                    .ins()
                    .load(pointer_type, MemFlags::trusted(), invocation, 0);
            Lowering::require_nonzero(&mut builder, process_input);
            let capability = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), invocation, 16);
            let int_arena = builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), invocation, 24);
            Lowering::require_nonzero(&mut builder, int_arena);
            compiler.native_int_arena = Some(int_arena);
            initial_env.push(Lowered::BorrowedNativeValue {
                pointer: process_input,
            });
            initial_env.push(Lowered::CapabilityToken { value: capability });
        }
        if let Some(value) = staged_process_input {
            initial_env.push(compiler.lower_value(&mut builder, value)?);
        }
        compiler.root_terminal_authority = compiler.take_distinguished_root_answer_authority()?;
        let lowered = compiler.lower_expr(&mut builder, expr, &initial_env)?;
        let result = match lowered {
            Lowered::Trap(trap) => {
                #[cfg(test)]
                if process_mode {
                    px8tr_record_trap_provenance(
                        Px8trTrapProvenanceEvent::FinalProcessObjectTrap { trap: trap.clone() },
                    );
                }
                let status = builder
                    .ins()
                    .iconst(types::I64, if process_mode { -4 } else { 0 });
                builder.ins().return_(&[status]);
                (Some(trap), None)
            }
            Lowered::RecursiveBackedge if !compiler.partition_queue.is_empty() => {
                (None, Some(ResultDecoder::ProcessStatus))
            }
            value => {
                let (token, decoder) = compiler.emit_result(&mut builder, value)?;
                builder.ins().return_(&[token]);
                (None, Some(decoder))
            }
        };
        builder.seal_all_blocks();
        builder.finalize();
        result
    };

    let measure = PartitionFunctionMeasure::from_function(&ctx.func);
    compiler.partition_budget.check(measure)?;
    record_partition_measure(measure);
    compiler.partition_measures.push(measure);
    compiler.capture_partition_ledger_union();
    verify_cranelift_function(&ctx.func, module.isa())?;
    module
        .define_function(func_id, &mut ctx)
        .map_err(|err| backend_module(err.to_string()))?;

    // Compile partition work depth-first. Each work item can enqueue both
    // mutually-exclusive arms of another host-result fanout, and the owned
    // residuals contain deep RuntimeExpr/environment descriptors. Consuming
    // the queue breadth-first retains the complete fanout frontier even though
    // only one helper is being lowered. Helper identities are assigned when
    // the call sites are emitted, so LIFO consumption changes neither the
    // generated call graph nor runtime branch/effect order.
    while let Some(item) = compiler.partition_queue.pop_back() {
        let continuation_state_id = match &item {
            PartitionWorkItem::SourceArm(item) => {
                compiler
                    .partition_continuations
                    .begin_emitting(item.state_id)?;
                Some(item.state_id)
            }
            PartitionWorkItem::SourceKont(item) => {
                compiler
                    .partition_continuations
                    .begin_emitting(item.state_id)?;
                Some(item.state_id)
            }
            PartitionWorkItem::ProducerKont(item) => {
                compiler
                    .partition_continuations
                    .begin_emitting(item.state_id)?;
                Some(item.state_id)
            }
            PartitionWorkItem::CleanupStep(item) => {
                compiler
                    .partition_continuations
                    .begin_emitting(item.state_id)?;
                Some(item.state_id)
            }
            PartitionWorkItem::Arm(_) => None,
        };
        let required_helper_ids = compiler
            .partition_next_helper
            .checked_add(PARTITION_HELPER_ID_RESERVE)
            .ok_or_else(|| {
                unsupported(
                    "NativeFunctionPartition",
                    "partition helper identity exhausted",
                )
            })?;
        for index in compiler.partition_helper_ids.len()..required_helper_ids {
            let name = format!("{function_name}.__ken_partition_{index}");
            compiler.partition_helper_ids.push(
                module
                    .declare_function(&name, Linkage::Local, &partition_sig)
                    .map_err(|err| backend_module(err.to_string()))?,
            );
        }
        let helper_id = match &item {
            PartitionWorkItem::SourceArm(item) => item.function,
            PartitionWorkItem::SourceKont(item) => item.function,
            PartitionWorkItem::ProducerKont(item) => item.function,
            PartitionWorkItem::Arm(item) => item.function,
            PartitionWorkItem::CleanupStep(item) => item.function,
        };
        match &item {
            PartitionWorkItem::SourceArm(item) => {
                compiler.restore_partition_ledger_baseline(&item.ledger_baseline);
            }
            PartitionWorkItem::SourceKont(item) => {
                compiler.restore_partition_ledger_baseline(&item.ledger_baseline);
            }
            PartitionWorkItem::ProducerKont(item) => {
                compiler.restore_partition_ledger_baseline(&item.ledger_baseline);
            }
            PartitionWorkItem::Arm(item) => {
                compiler.restore_partition_ledger_baseline(&item.ledger_baseline);
            }
            PartitionWorkItem::CleanupStep(item) => {
                compiler.restore_partition_ledger_baseline(&item.ledger_baseline);
            }
        }
        let mut helper_ctx = module.make_context();
        helper_ctx.func = Function::with_name_signature(
            UserFuncName::user(0, helper_id.as_u32()),
            partition_sig.clone(),
        );
        compiler.host_dispatch =
            host_dispatch_id.map(|id| module.declare_func_in_func(id, &mut helper_ctx.func));
        compiler.native_int_binop =
            Some(module.declare_func_in_func(native_int.binop, &mut helper_ctx.func));
        compiler.native_int_compare =
            Some(module.declare_func_in_func(native_int.compare, &mut helper_ctx.func));
        compiler.native_int_intern =
            Some(module.declare_func_in_func(native_int.intern, &mut helper_ctx.func));
        compiler.native_int_narrow =
            Some(module.declare_func_in_func(native_int.narrow, &mut helper_ctx.func));
        compiler.native_int_export =
            Some(module.declare_func_in_func(native_int.export, &mut helper_ctx.func));
        compiler.invocation_pointer = None;
        compiler.native_int_arena = None;
        compiler.native_int_tags.clear();
        compiler.partition_cut_armed = false;
        compiler.partition_output_tag_pointer = None;
        compiler.active_partition_producer_kont = None;
        compiler.active_partition_return_contract = None;
        compiler.partition_live_growth_ticks = 0;
        compiler.declaration_stack.clear();
        compiler.active_recursive_declarations.clear();
        compiler.active_recursive_invocations.clear();
        compiler.pending_recursive_call = None;
        compiler.pending_computational_ih_call = None;

        let mut helper_func_ctx = FunctionBuilderContext::new();
        let helper_result = {
            let mut builder = FunctionBuilder::new(&mut helper_ctx.func, &mut helper_func_ctx);
            let block = builder.create_block();
            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);
            let invocation = builder.block_params(block)[0];
            let frame_pointer = builder.block_params(block)[1];
            let output_tag_pointer = builder.block_params(block)[2];
            compiler.invocation_pointer = Some(invocation);
            compiler.partition_output_tag_pointer = Some(output_tag_pointer);
            let pointer_type = builder.func.dfg.value_type(invocation);
            let int_arena = builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), invocation, 24);
            Lowering::require_nonzero(&mut builder, int_arena);
            compiler.native_int_arena = Some(int_arena);
            let result = match item {
                PartitionWorkItem::SourceArm(item) => compiler
                    .lower_source_arm_partition_work_item(&mut builder, item, frame_pointer)?,
                PartitionWorkItem::SourceKont(item) => compiler
                    .lower_source_kont_partition_work_item(&mut builder, item, frame_pointer)?,
                PartitionWorkItem::ProducerKont(item) => compiler
                    .lower_producer_kont_partition_work_item(&mut builder, item, frame_pointer)?,
                PartitionWorkItem::Arm(item) => {
                    compiler.lower_arm_partition_work_item(&mut builder, item, frame_pointer)?
                }
                PartitionWorkItem::CleanupStep(item) => compiler
                    .lower_cleanup_step_partition_work_item(&mut builder, item, frame_pointer)?,
            };
            builder.seal_all_blocks();
            builder.finalize();
            result
        };
        if maybe_trap.is_none() {
            maybe_trap = helper_result.0;
        }
        if decoder.is_none() {
            decoder = Some(helper_result.1);
        }
        let measure = PartitionFunctionMeasure::from_function(&helper_ctx.func);
        compiler.partition_budget.check(measure)?;
        record_partition_measure(measure);
        compiler.partition_measures.push(measure);
        compiler.capture_partition_ledger_union();
        verify_cranelift_function(&helper_ctx.func, module.isa())?;
        module
            .define_function(helper_id, &mut helper_ctx)
            .map_err(|err| backend_module(err.to_string()))?;
        if let Some(state_id) = continuation_state_id {
            compiler
                .partition_continuations
                .finish_definition(state_id)?;
        }
    }

    compiler.restore_partition_ledger_union();
    compiler.require_complete_partition_branch_returns()?;
    compiler.partition_cleanup_transitions.require_complete()?;
    compiler.partition_continuations.require_complete()?;
    compiler.require_complete_join_plan_consumption()?;
    compiler.require_complete_dynamic_splice_edge_consumption()?;

    if std::env::var_os("KEN_NATIVE_PARTITION_METRICS").is_some() {
        let (states, edges, defined_states) = compiler.partition_continuations.counts();
        let (
            state_key_bytes_constructed,
            state_key_bytes_retained,
            state_key_bucket_probes,
            state_key_exact_comparisons,
            state_key_exact_bytes_compared_upper_bound,
        ) = compiler.partition_continuations.representation_counts();
        let (
            static_descriptor_bytes_constructed,
            static_descriptor_bytes_retained,
            static_descriptor_bucket_probes,
            static_descriptor_exact_comparisons,
            static_descriptor_exact_bytes_compared_upper_bound,
        ) = partition_static_descriptor_counts();
        let (
            cleanup_suffixes,
            cleanup_key_bytes_constructed,
            cleanup_key_bytes_retained,
            cleanup_key_exact_comparisons,
        ) = compiler.partition_cleanup_suffixes.counts();
        let mut dfg_values_total = 0_usize;
        let mut dfg_instructions_total = 0_usize;
        let mut dfg_blocks_total = 0_usize;
        let mut dfg_values_max = 0_usize;
        let mut dfg_instructions_max = 0_usize;
        let mut dfg_blocks_max = 0_usize;
        for measure in &compiler.partition_measures {
            dfg_values_total = dfg_values_total.saturating_add(measure.values);
            dfg_instructions_total = dfg_instructions_total.saturating_add(measure.instructions);
            dfg_blocks_total = dfg_blocks_total.saturating_add(measure.blocks);
            dfg_values_max = dfg_values_max.max(measure.values);
            dfg_instructions_max = dfg_instructions_max.max(measure.instructions);
            dfg_blocks_max = dfg_blocks_max.max(measure.blocks);
        }
        eprintln!(
            "KEN_NATIVE_PARTITION_METRICS_V1 source_bytes={} checked_plan_bytes={} \
             checked_joins={} checked_predecessors={} states={} edges={} defined_states={} \
             helper_ids_declared={} helpers_defined={} frame_fields_total={} \
             frame_fields_max={} frame_stores={} frame_loads={} \
             cleanup_states={} cleanup_edges={} cleanup_suffixes={} \
             cleanup_frame_fields_total={} cleanup_frame_fields_max={} \
             cleanup_frame_stores={} cleanup_frame_loads={} \
             cleanup_key_bytes_constructed={} cleanup_key_bytes_retained={} \
             cleanup_key_exact_comparisons={} \
             source_env_fields_total={} source_env_fields_max={} \
             source_prefix_fields_total={} source_prefix_fields_max={} \
             source_scope_fields_total={} source_scope_fields_max={} \
             source_lineage_fields_total={} source_lineage_fields_max={} \
             state_key_bytes_constructed={} state_key_bytes_retained={} \
             state_key_bucket_probes={} state_key_exact_comparisons={} \
             state_key_exact_bytes_compared_upper_bound={} \
             static_descriptor_bytes_constructed={} static_descriptor_bytes_retained={} \
             static_descriptor_bucket_probes={} static_descriptor_exact_comparisons={} \
             static_descriptor_exact_bytes_compared_upper_bound={} \
             dfg_values_total={} dfg_values_max={} dfg_instructions_total={} \
             dfg_instructions_max={} dfg_blocks_total={} dfg_blocks_max={}",
            partition_source_bytes,
            checked_plan_bytes,
            checked_join_count,
            checked_predecessor_count,
            states,
            edges,
            defined_states,
            compiler.partition_helper_ids.len(),
            compiler.partition_measures.len().saturating_sub(1),
            compiler.partition_metrics.frame_fields_total,
            compiler.partition_metrics.frame_fields_max,
            compiler.partition_metrics.frame_stores,
            compiler.partition_metrics.frame_loads,
            compiler.partition_metrics.cleanup_states,
            compiler.partition_metrics.cleanup_edges,
            cleanup_suffixes,
            compiler.partition_metrics.cleanup_frame_fields_total,
            compiler.partition_metrics.cleanup_frame_fields_max,
            compiler.partition_metrics.cleanup_frame_stores,
            compiler.partition_metrics.cleanup_frame_loads,
            cleanup_key_bytes_constructed,
            cleanup_key_bytes_retained,
            cleanup_key_exact_comparisons,
            compiler.partition_metrics.source_env_fields_total,
            compiler.partition_metrics.source_env_fields_max,
            compiler.partition_metrics.source_prefix_fields_total,
            compiler.partition_metrics.source_prefix_fields_max,
            compiler.partition_metrics.source_scope_fields_total,
            compiler.partition_metrics.source_scope_fields_max,
            compiler.partition_metrics.source_lineage_fields_total,
            compiler.partition_metrics.source_lineage_fields_max,
            state_key_bytes_constructed,
            state_key_bytes_retained,
            state_key_bucket_probes,
            state_key_exact_comparisons,
            state_key_exact_bytes_compared_upper_bound,
            static_descriptor_bytes_constructed,
            static_descriptor_bytes_retained,
            static_descriptor_bucket_probes,
            static_descriptor_exact_comparisons,
            static_descriptor_exact_bytes_compared_upper_bound,
            dfg_values_total,
            dfg_values_max,
            dfg_instructions_total,
            dfg_instructions_max,
            dfg_blocks_total,
            dfg_blocks_max,
        );
    }

    Ok(CompiledModule::from_parts(
        module,
        func_id,
        decoder,
        compiler.result_table,
        maybe_trap,
        true,
        compiler.assumptions,
        compiler.unsupported,
    ))
}

impl<'a> Lowering<'a> {
    fn allocate_control_cursor_ref(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        parent: Option<ControlCursorRef>,
    ) -> Result<ControlCursorRef, CraneliftBackendError> {
        let invocation = self
            .invocation_pointer
            .expect("control-cell allocation owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES,
            3,
        ));
        let parent =
            parent.map_or_else(|| builder.ins().iconst(pointer_type, 0), |parent| parent.0);
        builder.ins().stack_store(parent, slot, 0);
        Ok(ControlCursorRef(builder.ins().stack_addr(
            pointer_type,
            slot,
            0,
        )))
    }

    fn allocate_selected_control_refs(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        parent_activation: Option<ActivationInstanceRef>,
        parent_cursor: Option<ControlCursorRef>,
        parent_scope: Option<ScopeInstanceRef>,
    ) -> Result<(ActivationInstanceRef, ControlCursorRef, ScopeInstanceRef), CraneliftBackendError>
    {
        let invocation = self
            .invocation_pointer
            .expect("selected-control cell allocation owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES * 3,
            3,
        ));
        let parent_activation = parent_activation
            .map_or_else(|| builder.ins().iconst(pointer_type, 0), |parent| parent.0);
        let parent_cursor =
            parent_cursor.map_or_else(|| builder.ins().iconst(pointer_type, 0), |parent| parent.0);
        let parent_scope =
            parent_scope.map_or_else(|| builder.ins().iconst(pointer_type, 0), |parent| parent.0);
        builder.ins().stack_store(parent_activation, slot, 0);
        builder
            .ins()
            .stack_store(parent_cursor, slot, PARTITION_FRAME_FIELD_BYTES as i32);
        builder
            .ins()
            .stack_store(parent_scope, slot, (PARTITION_FRAME_FIELD_BYTES * 2) as i32);
        let pointer = builder.ins().stack_addr(pointer_type, slot, 0);
        Ok((
            ActivationInstanceRef(pointer),
            ControlCursorRef(pointer),
            ScopeInstanceRef(pointer),
        ))
    }

    fn push_partition_source_cursor(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        control: &mut SourceControl<'_>,
    ) -> Result<(), CraneliftBackendError> {
        let mut current =
            partition_source_head_template(&control.continuation, control.terminal_outer)?;
        if let SourcePrefixTemplate::ReturnFromSelectedCase { parent_capture, .. } = &mut current {
            *parent_capture = Some(own_partition_selected(&control.selected).ok_or_else(|| {
                unsupported(
                    "NativeControlCellV1",
                    "selected return parent has no exact immediate capture schema",
                )
            })?);
        }
        self.push_partition_source_template(builder, control, current)
    }

    fn push_partition_source_prefix(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        control: &mut SourceControl<'_>,
    ) -> Result<(), CraneliftBackendError> {
        let mut current = &control.continuation;
        let mut prefix = Vec::new();
        while let Some(next) = partition_source_continuation_next(current) {
            prefix.push(partition_source_head_template(
                current,
                control.terminal_outer,
            )?);
            current = next;
        }
        for mut template in prefix.into_iter().rev() {
            if let SourcePrefixTemplate::ReturnFromSelectedCase { parent_capture, .. } =
                &mut template
            {
                *parent_capture =
                    Some(own_partition_selected(&control.selected).ok_or_else(|| {
                        unsupported(
                            "NativeControlCellV1",
                            "selected return parent has no exact immediate capture schema",
                        )
                    })?);
            }
            self.push_partition_source_template(builder, control, template)?;
        }
        Ok(())
    }

    fn push_partition_source_template(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        control: &mut SourceControl<'_>,
        current: SourcePrefixTemplate,
    ) -> Result<(), CraneliftBackendError> {
        let mut capture_values = Vec::new();
        append_partition_prefix_values(self, builder, &current, &mut capture_values)?;
        let capture_field_types = capture_values
            .iter()
            .map(|value| builder.func.dfg.value_type(*value))
            .collect::<Vec<_>>();
        let successor = control.partition_cursor.map(|cursor| cursor.node);
        let node = self
            .partition_source_nodes
            .intern(current, capture_field_types, successor);

        let invocation = self
            .invocation_pointer
            .expect("source continuation capture owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let field_count = capture_values.len().checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeSourceContinuationStepV1",
                "source-continuation cell width overflowed",
            )
        })?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(field_count)?,
            3,
        ));
        let parent = control.partition_cursor.map_or_else(
            || builder.ins().iconst(pointer_type, 0),
            |cursor| cursor.capture_pointer,
        );
        builder.ins().stack_store(parent, frame, 0);
        for (index, value) in capture_values.iter().copied().enumerate() {
            let offset = index
                .checked_add(1)
                .and_then(|field| field.checked_mul(PARTITION_FRAME_FIELD_BYTES as usize))
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeSourceContinuationStepV1",
                        "source-continuation cell store offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let capture_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        control.partition_cursor = Some(PartitionSourceCursor {
            node,
            capture_pointer,
        });
        Ok(())
    }

    fn push_partition_recursor_layer(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
        layer: ComputationalRecursorLayer,
    ) -> Result<(), CraneliftBackendError> {
        let obligation = match layer.role {
            RecursorLayerRole::ExitsScope {
                scope_instance,
                parent_scope_instance,
                ..
            } => Some((
                layer.checked_frame_id,
                layer.semantic_pending,
                scope_instance,
                parent_scope_instance,
            )),
            RecursorLayerRole::SelectsOccurrence { .. } => None,
        };
        let mut capture_values = Vec::new();
        append_partition_layer_values(self, builder, &layer, &mut capture_values)?;
        let capture_field_types = capture_values
            .iter()
            .map(|value| builder.func.dfg.value_type(*value))
            .collect::<Vec<_>>();
        let successor = stack.partition_cursor.map(|cursor| cursor.node);
        let node = self
            .partition_recursor_nodes
            .intern(layer, capture_field_types, successor);
        let invocation = self
            .invocation_pointer
            .expect("recursor continuation capture owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let field_count = capture_values.len().checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeRecursorContinuationStepV1",
                "recursor-continuation cell width overflowed",
            )
        })?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(field_count)?,
            3,
        ));
        let successor_pointer = stack.partition_cursor.map_or_else(
            || builder.ins().iconst(pointer_type, 0),
            |cursor| cursor.capture_pointer,
        );
        builder.ins().stack_store(successor_pointer, frame, 0);
        for (index, value) in capture_values.iter().copied().enumerate() {
            let offset = index
                .checked_add(1)
                .and_then(|field| field.checked_mul(PARTITION_FRAME_FIELD_BYTES as usize))
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeRecursorContinuationStepV1",
                        "recursor-continuation cell store offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        stack.partition_cursor = Some(PartitionRecursorCursor {
            node,
            capture_pointer: builder.ins().stack_addr(pointer_type, frame, 0),
        });
        if let Some((checked_frame_id, semantic_pending, scope, parent_scope)) = obligation {
            let target = stack
                .partition_cursor
                .expect("new recursor layer has an exact persistent cursor");
            self.push_partition_open_control_obligation(
                builder,
                stack,
                target,
                checked_frame_id,
                semantic_pending,
                scope,
                parent_scope,
            )?;
        }
        Ok(())
    }

    fn push_partition_open_control_obligation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
        target: PartitionRecursorCursor,
        checked_frame_id: Option<u64>,
        semantic_pending: bool,
        scope: ScopeInstanceRef,
        parent_scope: Option<ScopeInstanceRef>,
    ) -> Result<(), CraneliftBackendError> {
        let successor = stack.partition_open_obligation.map(|cursor| cursor.node);
        let node = self.partition_open_control_obligations.intern(
            target.node,
            checked_frame_id,
            semantic_pending,
            parent_scope.is_some(),
            successor,
        );
        let invocation = self
            .invocation_pointer
            .expect("open control obligation owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES * 4,
            3,
        ));
        let successor_pointer = stack.partition_open_obligation.map_or_else(
            || builder.ins().iconst(pointer_type, 0),
            |cursor| cursor.capture_pointer,
        );
        let null_parent = builder.ins().iconst(pointer_type, 0);
        builder.ins().stack_store(successor_pointer, frame, 0);
        builder.ins().stack_store(
            target.capture_pointer,
            frame,
            PARTITION_FRAME_FIELD_BYTES as i32,
        );
        builder
            .ins()
            .stack_store(scope.0, frame, (PARTITION_FRAME_FIELD_BYTES * 2) as i32);
        builder.ins().stack_store(
            parent_scope.map_or(null_parent, |parent| parent.0),
            frame,
            (PARTITION_FRAME_FIELD_BYTES * 3) as i32,
        );
        stack.partition_open_obligation = Some(PartitionOpenControlObligationCursor {
            node,
            capture_pointer: builder.ins().stack_addr(pointer_type, frame, 0),
        });
        Ok(())
    }

    pub(super) fn materialize_partition_recursor_stack(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
        checked_parent_frame: Option<u64>,
    ) -> Result<Option<PartitionRecursorCursor>, CraneliftBackendError> {
        let flat = std::mem::take(&mut stack.later_wrappers_in_construction_order);
        let mut checked_parent = None;
        for layer in flat {
            let is_checked_parent = checked_parent_frame.is_some_and(|frame| {
                layer.semantic_pending && layer.checked_frame_id == Some(frame)
            });
            self.push_partition_recursor_layer(builder, stack, layer)?;
            if is_checked_parent {
                if checked_parent
                    .replace(
                        stack
                            .partition_cursor
                            .expect("pushed recursor layer has a persistent cursor"),
                    )
                    .is_some()
                {
                    return Err(unsupported(
                        "NativeRecursorContinuationStepV1",
                        "persistent recursor stack has multiple exact checked parents",
                    ));
                }
            }
        }
        Ok(checked_parent)
    }

    pub(super) fn push_partition_recursor_qualification(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
        target: PartitionRecursorCursor,
        source: InvocationTemplateRef,
    ) -> Result<(), CraneliftBackendError> {
        let successor = stack.partition_qualification.map(|cursor| cursor.node);
        let node = self
            .partition_recursor_qualifications
            .intern(target.node, source, successor);
        let invocation = self
            .invocation_pointer
            .expect("recursor qualification capture owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES * 2,
            3,
        ));
        let successor_pointer = stack.partition_qualification.map_or_else(
            || builder.ins().iconst(pointer_type, 0),
            |cursor| cursor.capture_pointer,
        );
        builder.ins().stack_store(successor_pointer, frame, 0);
        builder.ins().stack_store(
            target.capture_pointer,
            frame,
            PARTITION_FRAME_FIELD_BYTES as i32,
        );
        stack.partition_qualification = Some(PartitionRecursorQualificationCursor {
            node,
            capture_pointer: builder.ins().stack_addr(pointer_type, frame, 0),
        });
        Ok(())
    }

    fn pop_partition_recursor_layer(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
    ) -> Result<Option<PoppedPartitionRecursorLayer>, CraneliftBackendError> {
        let Some(cursor) = stack.partition_cursor else {
            if stack.partition_qualification.is_some() {
                return Err(unsupported(
                    "NativeRecursorContinuationStepV1",
                    "recursor qualification outlived its exact target stack",
                ));
            }
            if stack.partition_open_obligation.is_some() {
                return Err(unsupported(
                    "NativeOpenControlObligationStepV1",
                    "open control obligation outlived its exact recursor stack",
                ));
            }
            return Ok(stack
                .later_wrappers_in_construction_order
                .pop()
                .map(|layer| PoppedPartitionRecursorLayer {
                    layer,
                    target: None,
                    exit_obligation: None,
                    exit_obligation_successor: None,
                }));
        };
        let mut definition = self.partition_recursor_nodes.definition(cursor.node)?;
        let mut captures = definition
            .capture_field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let offset = index
                    .checked_add(1)
                    .and_then(|field| field.checked_mul(PARTITION_FRAME_FIELD_BYTES as usize))
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeRecursorContinuationStepV1",
                            "recursor-continuation cell load offset overflowed",
                        )
                    })?;
                Ok(builder.ins().load(
                    *field_type,
                    MemFlags::trusted(),
                    cursor.capture_pointer,
                    offset,
                ))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?
            .into_iter();
        rebuild_partition_layer(
            &mut definition.current,
            &mut captures,
            &mut self.native_int_tags,
        )?;
        if captures.next().is_some() {
            return Err(unsupported(
                "NativeRecursorContinuationStepV1",
                "recursor-continuation cell has trailing fields",
            ));
        }
        let pointer_type = builder.func.dfg.value_type(cursor.capture_pointer);
        if let Some(qualification) = stack.partition_qualification {
            let qualification_definition = self
                .partition_recursor_qualifications
                .definition(qualification.node)?;
            if qualification_definition.target == cursor.node {
                let expected_pointer = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    qualification.capture_pointer,
                    PARTITION_FRAME_FIELD_BYTES as i32,
                );
                self.emit_control_cell_ref_guard(
                    builder,
                    &[(cursor.capture_pointer, expected_pointer)],
                );
                definition.current.checked_invocation_id =
                    Some((1_u64 << 63) | u64::from(qualification.node.0));
                definition.current.checked_invocation_source =
                    Some(qualification_definition.source);
                definition.current.checked_invocation_depth = 0;
                let successor_qualification_pointer = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    qualification.capture_pointer,
                    0,
                );
                stack.partition_qualification = qualification_definition.successor.map(|node| {
                    PartitionRecursorQualificationCursor {
                        node,
                        capture_pointer: successor_qualification_pointer,
                    }
                });
            }
        }
        let (exit_obligation, exit_obligation_successor) = self
            .reserve_partition_open_control_obligation(
                builder,
                stack,
                cursor,
                &definition.current,
            )?;
        let successor_pointer =
            builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), cursor.capture_pointer, 0);
        stack.partition_cursor = definition.successor.map(|node| PartitionRecursorCursor {
            node,
            capture_pointer: successor_pointer,
        });
        Ok(Some(PoppedPartitionRecursorLayer {
            layer: definition.current,
            target: Some(cursor),
            exit_obligation,
            exit_obligation_successor,
        }))
    }

    fn reserve_partition_open_control_obligation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
        target: PartitionRecursorCursor,
        layer: &ComputationalRecursorLayer,
    ) -> Result<
        (
            Option<PartitionOpenControlObligationCursor>,
            Option<PartitionOpenControlObligationCursor>,
        ),
        CraneliftBackendError,
    > {
        let RecursorLayerRole::ExitsScope {
            scope_instance,
            parent_scope_instance,
            ..
        } = layer.role
        else {
            return Ok((None, None));
        };
        let obligation = stack.partition_open_obligation.ok_or_else(|| {
            unsupported(
                "NativeOpenControlObligationStepV1",
                "scope-exit layer has no exact open control obligation",
            )
        })?;
        let definition = self
            .partition_open_control_obligations
            .definition(obligation.node)?;
        if definition.target != target.node
            || definition.checked_frame_id != layer.checked_frame_id
            || definition.semantic_pending != layer.semantic_pending
            || definition.has_parent_scope != parent_scope_instance.is_some()
        {
            return Err(unsupported(
                "NativeOpenControlObligationStepV1",
                "scope-exit layer does not own the current open control obligation",
            ));
        }
        let pointer_type = builder.func.dfg.value_type(obligation.capture_pointer);
        let expected_target = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            obligation.capture_pointer,
            PARTITION_FRAME_FIELD_BYTES as i32,
        );
        let expected_scope = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            obligation.capture_pointer,
            (PARTITION_FRAME_FIELD_BYTES * 2) as i32,
        );
        let expected_parent = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            obligation.capture_pointer,
            (PARTITION_FRAME_FIELD_BYTES * 3) as i32,
        );
        let actual_parent = parent_scope_instance
            .map_or_else(|| builder.ins().iconst(pointer_type, 0), |parent| parent.0);
        self.emit_control_cell_ref_guard(
            builder,
            &[
                (target.capture_pointer, expected_target),
                (scope_instance.0, expected_scope),
                (actual_parent, expected_parent),
            ],
        );
        let successor_pointer = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            obligation.capture_pointer,
            0,
        );
        let successor = definition
            .successor
            .map(|node| PartitionOpenControlObligationCursor {
                node,
                capture_pointer: successor_pointer,
            });
        // This advances only the compiler's planning cursor. Runtime ownership
        // remains with the exact obligation cell until ExitScopeComplete.
        stack.partition_open_obligation = successor;
        Ok((Some(obligation), successor))
    }

    fn drain_partition_exit_stack(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        stack: &mut RecursorUnwindStack,
    ) -> Result<Vec<PoppedPartitionRecursorLayer>, CraneliftBackendError> {
        if stack.partition_cursor.is_none() {
            return Ok(Vec::new());
        }
        let mut popped = Vec::new();
        while let Some(step) = self.pop_partition_recursor_layer(builder, stack)? {
            if !matches!(step.layer.role, RecursorLayerRole::ExitsScope { .. }) {
                return Err(unsupported(
                    "NativeExitScopeTransitionV1",
                    "persistent unwind retained a non-exit layer after its selected occurrence",
                ));
            }
            #[cfg(test)]
            if let RecursorLayerRole::ExitsScope {
                origin,
                scope_origin,
                parent_scope,
                ..
            } = step.layer.role
            {
                px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                    origin,
                    scope_origin,
                    parent_scope,
                });
            }
            popped.push(step);
        }
        Ok(popped)
    }

    fn reserve_partition_scope_exits_from_tail<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        continuation: &mut SourceContinuation<'b>,
    ) -> Result<Vec<PoppedPartitionRecursorLayer>, CraneliftBackendError> {
        let mut current = continuation;
        loop {
            match current {
                SourceContinuation::UnwindRecursorSegment { stack, .. } => {
                    return self.drain_partition_exit_stack(builder, stack);
                }
                SourceContinuation::Terminal(
                    SourceContinuationTerminal::ReturnToProducerHole { stack, .. },
                ) => return self.drain_partition_exit_stack(builder, stack),
                SourceContinuation::Partitioned { terminal, .. } => {
                    return match terminal {
                        SourceContinuationTerminal::ReturnToProducerHole { stack, .. } => {
                            self.drain_partition_exit_stack(builder, stack)
                        }
                        _ => Ok(Vec::new()),
                    };
                }
                SourceContinuation::Terminal(_) => return Ok(Vec::new()),
                SourceContinuation::CheckedRecursiveInvocationReturn { next, .. }
                | SourceContinuation::CheckedComputationalIHInvocationReturn { next, .. }
                | SourceContinuation::ReturnFromSelectedCase { next, .. }
                | SourceContinuation::LetBody { next, .. }
                | SourceContinuation::ApplyRecursorSelection { next, .. }
                | SourceContinuation::IfScrutinee { next, .. }
                | SourceContinuation::ConstructArgument { next, .. }
                | SourceContinuation::MatchScrutinee { next, .. }
                | SourceContinuation::ComputationalMatchScrutinee { next, .. }
                | SourceContinuation::ProjectRecord { next, .. }
                | SourceContinuation::CallCallee { next, .. }
                | SourceContinuation::CallArgument { next, .. } => current = next,
            }
        }
    }

    fn pop_partition_source_cursor(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        control: &mut SourceControl<'_>,
    ) -> Result<(), CraneliftBackendError> {
        let Some(cursor) = control.partition_cursor else {
            return Ok(());
        };
        let definition = self.partition_source_nodes.definition(cursor.node)?;
        let pointer_type = builder.func.dfg.value_type(cursor.capture_pointer);
        let successor_pointer =
            builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), cursor.capture_pointer, 0);
        control.partition_cursor = definition.successor.map(|node| PartitionSourceCursor {
            node,
            capture_pointer: successor_pointer,
        });
        Ok(())
    }

    fn reserve_partition_exit_source_cursor(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        cursor: &mut Option<PartitionSourceCursor>,
    ) -> Result<Option<ReservedPartitionExitCursor>, CraneliftBackendError> {
        let Some(head) = *cursor else {
            return Ok(None);
        };
        let mut definition = self.partition_source_nodes.definition(head.node)?;
        if !matches!(
            definition.current,
            SourcePrefixTemplate::UnwindRecursorSegment { .. }
        ) {
            return Ok(None);
        }
        let mut captures = definition
            .capture_field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let offset = index
                    .checked_add(1)
                    .and_then(|field| field.checked_mul(PARTITION_FRAME_FIELD_BYTES as usize))
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeExitScopeTransitionV1",
                            "exit source-cursor capture offset overflowed",
                        )
                    })?;
                Ok(builder.ins().load(
                    *field_type,
                    MemFlags::trusted(),
                    head.capture_pointer,
                    offset,
                ))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?
            .into_iter();
        rebuild_partition_prefix(
            &mut definition.current,
            &mut captures,
            &mut self.native_int_tags,
        )?;
        if captures.next().is_some() {
            return Err(unsupported(
                "NativeExitScopeTransitionV1",
                "exit source-cursor capture has trailing fields",
            ));
        }
        let SourcePrefixTemplate::UnwindRecursorSegment {
            mut stack,
            resume_cursor,
            resume_cursor_instance,
            ..
        } = definition.current
        else {
            unreachable!("unwind source cursor was checked before reconstruction");
        };
        if stack.partition_cursor.is_none() {
            return Ok(None);
        }
        let popped = self.drain_partition_exit_stack(builder, &mut stack)?;
        let pointer_type = builder.func.dfg.value_type(head.capture_pointer);
        let successor_pointer =
            builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), head.capture_pointer, 0);
        *cursor = definition.successor.map(|node| PartitionSourceCursor {
            node,
            capture_pointer: successor_pointer,
        });
        Ok(Some(ReservedPartitionExitCursor {
            popped,
            resume_cursor,
            resume_cursor_instance,
        }))
    }

    fn check_partition_live_growth(
        &mut self,
        builder: &FunctionBuilder<'_>,
    ) -> Result<(), CraneliftBackendError> {
        if self.active_partition_return_kind.is_none() {
            return Ok(());
        }
        self.partition_live_growth_ticks = self.partition_live_growth_ticks.wrapping_add(1);
        if !self.partition_live_growth_ticks.is_multiple_of(256) {
            return Ok(());
        }
        let measure = PartitionFunctionMeasure::from_function(builder.func);
        if measure.values > self.partition_budget.max_values
            || measure.instructions > self.partition_budget.max_instructions
            || measure.blocks > self.partition_budget.max_blocks
        {
            if std::env::var_os("KEN_NATIVE_PARTITION_METRICS").is_some() {
                eprintln!(
                    "KEN_NATIVE_PARTITION_BUDGET_CONTEXT declaration_stack={:?} \
                     active_recursive={:?}",
                    self.declaration_stack,
                    self.active_recursive_declarations
                        .iter()
                        .map(|active| active.symbol.as_str())
                        .collect::<Vec<_>>(),
                );
            }
            return Err(unsupported(
                "NativeFunctionPartition",
                format!(
                    "active semantic state crossed the native function budget before reaching \
                     another admissible checked scalar cut: actual values/instructions/blocks = \
                     {}/{}/{}, limits = {}/{}/{}",
                    measure.values,
                    measure.instructions,
                    measure.blocks,
                    self.partition_budget.max_values,
                    self.partition_budget.max_instructions,
                    self.partition_budget.max_blocks,
                ),
            ));
        }
        Ok(())
    }

    fn emit_partition_pair_return(
        &self,
        builder: &mut FunctionBuilder<'_>,
        pair: NativeScalarPairV1,
    ) {
        let output = self
            .partition_output_tag_pointer
            .expect("partition helper owns its scalar-pair tag output");
        builder
            .ins()
            .store(MemFlags::trusted(), pair.tag, output, 0);
        builder.ins().return_(&[pair.payload]);
    }

    fn emit_control_cell_ref_guard(
        &self,
        builder: &mut FunctionBuilder<'_>,
        pairs: &[(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value)],
    ) {
        let mut pairs = pairs.iter().copied();
        let Some((left, right)) = pairs.next() else {
            return;
        };
        let mut valid =
            builder
                .ins()
                .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, left, right);
        for (left, right) in pairs {
            let equal =
                builder
                    .ins()
                    .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, left, right);
            valid = builder.ins().band(valid, equal);
        }
        let pass = builder.create_block();
        let reject = builder.create_block();
        builder.ins().brif(valid, pass, &[], reject, &[]);
        builder.seal_block(reject);
        builder.switch_to_block(reject);
        let failure = builder.ins().iconst(types::I64, -4);
        builder.ins().return_(&[failure]);
        builder.seal_block(pass);
        builder.switch_to_block(pass);
    }

    fn partition_helper_ref(
        &self,
        builder: &mut FunctionBuilder<'_>,
        helper_index: usize,
        exhaustion_reason: &'static str,
    ) -> Result<(FuncId, FuncRef), CraneliftBackendError> {
        let function = *self
            .partition_helper_ids
            .get(helper_index)
            .ok_or_else(|| unsupported("NativeFunctionPartition", exhaustion_reason))?;
        let signature = builder
            .func
            .import_signature(self.partition_signature.clone());
        let user_name =
            builder
                .func
                .declare_imported_user_function(cranelift_codegen::ir::UserExternalName {
                    namespace: 0,
                    index: function.as_u32(),
                });
        let function_ref = builder
            .func
            .import_function(cranelift_codegen::ir::ExtFuncData {
                name: cranelift_codegen::ir::ExternalName::user(user_name),
                signature,
                colocated: true,
            });
        Ok((function, function_ref))
    }

    fn resume_active_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
        active: ActiveContinuationFrame<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let Some((head, tail)) = active.pending.split_first() else {
            return Ok(value);
        };
        let cursor = self.mint_continuation_cursor();
        let cursor_instance =
            self.allocate_control_cursor_ref(builder, Some(active.cursor_instance))?;
        let successor = EliminatorFrame::Active(ActiveContinuationFrame {
            activation: active.activation,
            activation_instance: active.activation_instance,
            cursor,
            cursor_instance,
            parent: Some(&active),
            pending: tail,
            selected_ancestry: active.selected_ancestry,
            source_lineage: active.source_lineage,
            source_selected_cursor: active.source_selected_cursor,
            selected_scope: active.selected_scope,
        });
        self.lower_computational_match_value_composed(builder, value, &[*head, successor])
    }

    fn lower_recursor_residual_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        residual: &Lowered,
        args: &[RuntimeExpr],
        argument_env: &[Lowered],
        saved_producer_env: &[Lowered],
        outer_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        if let Lowered::BoundedNat(predecessor) = residual {
            if !args.is_empty() {
                return Err(unsupported(
                    "BoundedNat",
                    "structural Nat recursive hypothesis takes no arguments",
                ));
            }
            return self.lower_bounded_nat_computational(
                builder,
                *predecessor,
                false,
                outer_eliminators,
            );
        }
        let Lowered::Closure {
            captures,
            params,
            body,
        } = residual
        else {
            return Err(unsupported(
                "ComputationalMatch",
                "recursive constructor field is not a closure",
            ));
        };
        let mut call_env = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, argument_env))
            .collect::<Result<Vec<_>, _>>()?;
        if params.len() != call_env.len() {
            return Err(unsupported(
                "ComputationalMatch",
                format!(
                    "recursive field expects {} args but call provides {}",
                    params.len(),
                    call_env.len()
                ),
            ));
        }
        call_env.extend_from_slice(captures);
        call_env.extend_from_slice(saved_producer_env);
        self.lower_computational_producer_expr(builder, body, &call_env, outer_eliminators)
    }

    fn lower_computational_match_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: &RuntimeExpr,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
        producer_env: &[Lowered],
        eliminator_env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let checked_frame_id = self.consume_checked_subcontinuation_frame(cases, default)?;
        let checked_invocation_id = checked_frame_id.map(|_| {
            self.active_recursive_invocations
                .last()
                .map_or(0, |instance| instance.invocation_instance_id)
        });
        let checked_invocation_depth = self
            .active_recursive_invocations
            .last()
            .map_or(0, |instance| instance.semantic_depth);
        let provenance = self.mint_recursor_frame_provenance();
        self.lower_computational_producer_expr(
            builder,
            scrutinee,
            producer_env,
            &[EliminatorFrame::Computational(
                ComputationalEliminatorFrame {
                    cases,
                    default,
                    env: eliminator_env,
                    retained_scrutinee_index: None,
                    deferred_constructor_case: None,
                    provenance,
                    checked_frame_id,
                    checked_invocation_id,
                    checked_invocation_source: self
                        .active_recursive_invocations
                        .last()
                        .map(|instance| instance.source),
                    checked_invocation_depth,
                },
            )],
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_computational_host_result(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        producer_cases: &[crate::RuntimeMatchCase],
        producer_default: &RuntimeTrap,
        producer_env: &[Lowered],
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        builder.ins().brif(success, ok_block, &[], err_block, &[]);
        let mut exit_merge = None;
        for (block, constructor, payload) in [
            (ok_block, ok_constructor, ok),
            (err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let lowered = if let Some(producer_case) =
                dynamic_host_result_producer_case(producer_cases, constructor)?
            {
                let mut case_env = vec![payload];
                case_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(
                    builder,
                    &producer_case.body,
                    &case_env,
                    eliminators,
                )?
            } else {
                Lowered::Trap(producer_default.clone())
            };
            let (value, is_exit) =
                self.merge_branch_value(builder, lowered, "ComputationalMatch")?;
            Self::record_merge_kind("ComputationalMatch", &mut exit_merge, is_exit)?;
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(if exit_merge == Some(true) {
            Lowered::ProcessExitStatus {
                value: pair.payload,
            }
        } else {
            self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn partition_ledger_baseline(&self) -> PartitionLedgerBaseline {
        PartitionLedgerBaseline {
            join_sites: self.consumed_join_sites.clone(),
            subcontinuation_frames: self.consumed_subcontinuation_frames.clone(),
            recursive_call_templates: self.consumed_recursive_call_templates.clone(),
        }
    }

    fn restore_partition_ledger_baseline(&mut self, baseline: &PartitionLedgerBaseline) {
        self.consumed_join_sites = baseline.join_sites.clone();
        self.consumed_subcontinuation_frames = baseline.subcontinuation_frames.clone();
        self.consumed_recursive_call_templates = baseline.recursive_call_templates.clone();
    }

    fn capture_partition_ledger_union(&mut self) {
        self.partition_join_site_union
            .extend(self.consumed_join_sites.iter().copied());
        self.partition_subcontinuation_frame_union
            .extend(self.consumed_subcontinuation_frames.iter().copied());
        self.partition_recursive_call_template_union
            .extend(self.consumed_recursive_call_templates.iter().copied());
    }

    fn restore_partition_ledger_union(&mut self) {
        self.consumed_join_sites = self.partition_join_site_union.clone();
        self.consumed_subcontinuation_frames = self.partition_subcontinuation_frame_union.clone();
        self.consumed_recursive_call_templates =
            self.partition_recursive_call_template_union.clone();
    }

    fn mint_partition_branch_return(
        &mut self,
        partition_site_id: u64,
        edge_index: u64,
        helper_index: usize,
        required_kind: ScalarMergeKind,
    ) -> Result<PartitionBranchReturnAuthority, CraneliftBackendError> {
        self.partition_branch_returns.mint(
            partition_site_id,
            edge_index,
            helper_index,
            required_kind,
        )
    }

    fn consume_partition_branch_return(
        &mut self,
        authority: PartitionBranchReturnAuthority,
        helper_index: usize,
        actual_kind: ScalarMergeKind,
    ) -> Result<(), CraneliftBackendError> {
        self.partition_branch_returns
            .consume(authority, helper_index, actual_kind)
    }

    fn require_complete_partition_branch_returns(&self) -> Result<(), CraneliftBackendError> {
        self.partition_branch_returns.require_complete()
    }

    #[allow(clippy::too_many_arguments)]
    fn outline_computational_host_result_arms(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        eliminators: Vec<OwnedPartitionEliminator>,
    ) -> Result<Lowered, CraneliftBackendError> {
        if !self.has_checked_root_exit_representation()
            || (self.root_terminal_authority.is_none()
                && !self
                    .active_partition_return_kind
                    .is_some_and(partition_helper_return_kind_is_admissible))
        {
            return Err(unsupported(
                "NativeFunctionPartition",
                "arm outlining requires the checked ExitCode root representation",
            ));
        }
        let partition_site_id = self.partition_next_site;
        self.partition_next_site = self.partition_next_site.checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "partition fanout identity exhausted",
            )
        })?;
        let ledger_baseline = self.partition_ledger_baseline();
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.ins().brif(success, ok_block, &[], err_block, &[]);

        for (arm_index, block, constructor, payload) in [
            (0_u8, ok_block, ok_constructor, ok),
            (1_u8, err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let (body, mut arm_env) =
                if let Some(case) = dynamic_host_result_producer_case(cases, constructor)? {
                    let mut arm_env = vec![payload];
                    arm_env.extend_from_slice(env);
                    (case.body.clone(), arm_env)
                } else {
                    (RuntimeExpr::Trap(default.clone()), env.to_vec())
                };
            let helper_index = self.partition_next_helper;
            if helper_index >= PartitionAggregateBudget::PRODUCTION.max_helpers {
                return Err(unsupported(
                    "NativeFunctionPartition",
                    "aggregate native partition graph exceeds its helper ceiling",
                ));
            }
            let (function, function_ref) = self.partition_helper_ref(
                builder,
                helper_index,
                "host-result fanout exhausted its predeclared helper pool",
            )?;
            self.partition_next_helper += 1;
            let return_authority = self.mint_partition_branch_return(
                partition_site_id,
                u64::from(arm_index),
                helper_index,
                ScalarMergeKind::ExitCode,
            )?;

            let mut fields = Vec::new();
            for value in &arm_env {
                append_partition_lowered_values(self, builder, value, &mut fields)?;
            }
            append_partition_eliminator_values(self, builder, &eliminators, &mut fields)?;
            if let Some(producer_kont) = self.active_partition_producer_kont {
                fields.push(producer_kont.capture_pointer);
            }
            let (frame_values, field_types, field_map) = partition_frame_layout(builder, &fields);
            self.partition_metrics.record_call_frame(frame_values.len());
            let frame_size = partition_frame_size(frame_values.len())?;
            let frame = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                frame_size,
                3,
            ));
            for (index, value) in frame_values.iter().copied().enumerate() {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeFunctionPartition",
                            "private arm frame offset overflowed",
                        )
                    })?;
                builder.ins().stack_store(value, frame, byte_offset);
            }
            let invocation = self
                .invocation_pointer
                .expect("process partition owns an invocation pointer");
            let pointer_type = builder.func.dfg.value_type(invocation);
            let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
            let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                PARTITION_FRAME_FIELD_BYTES,
                3,
            ));
            let zero_tag = builder.ins().iconst(types::I64, 0);
            builder.ins().stack_store(zero_tag, tag_output, 0);
            let tag_output_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
            let call = builder.ins().call(
                function_ref,
                &[invocation, frame_pointer, tag_output_pointer],
            );
            let status = builder.inst_results(call)[0];
            builder.ins().jump(merge, &[status.into()]);

            self.partition_queue
                .push_back(PartitionWorkItem::Arm(ArmPartitionWorkItem {
                    function,
                    helper_index,
                    field_types,
                    field_map,
                    body,
                    env: std::mem::take(&mut arm_env),
                    eliminators: eliminators.clone(),
                    producer_kont: self.active_partition_producer_kont,
                    ledger_baseline: ledger_baseline.clone(),
                    return_authority,
                }));
        }
        builder.switch_to_block(merge);
        self.partition_cut_armed = false;
        Ok(Lowered::ProcessExitStatus {
            value: builder.block_params(merge)[0],
        })
    }

    fn lower_computational_producer_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: &RuntimeExpr,
        producer_env: &[Lowered],
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        if eliminators.is_empty() {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        }
        if matches!(eliminators[0], EliminatorFrame::InvocationReturn) {
            return self.lower_expr(builder, scrutinee, producer_env);
        }
        if let EliminatorFrame::PendingLet(continuation) = eliminators[0] {
            let value = self.lower_expr(builder, scrutinee, producer_env)?;
            if matches!(value, Lowered::RecursiveBackedge) {
                return Ok(Lowered::RecursiveBackedge);
            }
            if let Lowered::Trap(trap) = value {
                return Ok(Lowered::Trap(trap));
            }
            let mut continuation_env = vec![value];
            continuation_env.extend_from_slice(continuation.env);
            return self.lower_recursor_residual_call(
                builder,
                continuation.residual,
                continuation.args,
                &continuation_env,
                continuation.env,
                &eliminators[1..],
            );
        }
        if let EliminatorFrame::Active(active) = eliminators[0] {
            if !matches!(
                scrutinee,
                RuntimeExpr::Let { .. }
                    | RuntimeExpr::Call { .. }
                    | RuntimeExpr::Match { .. }
                    | RuntimeExpr::ComputationalMatch { .. }
                    | RuntimeExpr::If { .. }
            ) {
                let value = self.lower_expr(builder, scrutinee, producer_env)?;
                return self.resume_active_continuation(builder, value, active);
            }
        }
        match scrutinee {
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let result = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                );
                if self.active_subcontinuation_frame.take().is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "checked subcontinuation marker was not consumed by its frame",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                body,
                ..
            } => {
                let instance = self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let result = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                );
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                self.lower_computational_producer_expr(builder, body, producer_env, eliminators)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                let explicit_return =
                    self.push_checked_computational_ih_producer_kont(builder, *call_template_id)?;
                let outer_producer_kont = explicit_return.map(|producer_kont| {
                    self.active_partition_producer_kont.replace(producer_kont)
                });
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let value = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                )?;
                if let Some(outer_producer_kont) = outer_producer_kont {
                    self.active_partition_producer_kont = outer_producer_kont;
                    self.pending_computational_ih_call = None;
                    Ok(value)
                } else {
                    self.finish_checked_computational_ih_marker(builder, value)
                }
            }
            RuntimeExpr::Let { value, body } => {
                if reaches_environment_computational_recursor(body, producer_env, 1) {
                    if let RuntimeExpr::Call { callee, args } = body.as_ref() {
                        if let RuntimeExpr::Var(index) = callee.as_ref() {
                            if let Some(index) = (*index as usize).checked_sub(1) {
                                if let Some(callee @ Lowered::ComputationalRecursorClosure { .. }) =
                                    producer_env.get(index)
                                {
                                    let (residual, boundary) =
                                        decompose_computational_recursor(callee.clone());
                                    let (activation, invocation) = boundary.expect(
                                        "recursor closure carries a continuation delimiter",
                                    );
                                    let resume_cursor = invocation.resume_cursor;
                                    let current =
                                        active_recursor_frame(eliminators).ok_or_else(|| {
                                            unsupported(
                                                "ComputationalRecursor",
                                                "recursive invocation has no active continuation",
                                            )
                                        })?;
                                    let _resume = find_continuation_cursor(current, resume_cursor)
                                        .ok_or_else(|| {
                                            unsupported(
                                                "ComputationalRecursor",
                                                "recursive invocation cursor is not active",
                                            )
                                        })?;
                                    if !recursor_invocation_is_checked(&invocation) {
                                        validate_recursor_invocation_segment(&invocation)?;
                                    }
                                    let dynamic_splice_edges =
                                        self.take_dynamic_splice_edges(&invocation)?;
                                    let installed = compose_oriented_subcontinuation(
                                        self.oriented_subcontinuation_plan.as_ref(),
                                        self.active_recursive_invocations.last().copied(),
                                        activation,
                                        invocation,
                                        dynamic_splice_edges,
                                    )?;
                                    let frames = installed_oriented_eliminator_frames(&installed);
                                    let mut composed = Vec::with_capacity(frames.len() + 2);
                                    composed.push(EliminatorFrame::PendingLet(
                                        PendingLetContinuationFrame {
                                            residual: &residual,
                                            args,
                                            env: producer_env,
                                        },
                                    ));
                                    composed.extend(frames);
                                    composed.push(EliminatorFrame::InvocationReturn);
                                    let producer_kont = self.push_oriented_producer_kont(
                                        builder,
                                        installed.checked,
                                        eliminators,
                                    )?;
                                    let outer_producer_kont = producer_kont.map(|producer_kont| {
                                        self.active_partition_producer_kont.replace(producer_kont)
                                    });
                                    self.enter_oriented_semantic_region(installed.checked);
                                    let returned = self.lower_computational_producer_expr(
                                        builder,
                                        value,
                                        producer_env,
                                        &composed,
                                    );
                                    self.leave_oriented_semantic_region(installed.checked);
                                    if let Some(outer_producer_kont) = outer_producer_kont {
                                        self.active_partition_producer_kont = outer_producer_kont;
                                    }
                                    let returned = returned?;
                                    if matches!(returned, Lowered::ProcessExitStatus { .. }) {
                                        return Ok(returned);
                                    }
                                    return self.lower_computational_match_value_composed(
                                        builder,
                                        returned,
                                        eliminators,
                                    );
                                }
                            }
                        }
                    }
                }
                let value = self.lower_expr(builder, value, producer_env)?;
                if let Lowered::Trap(trap) = value {
                    return Ok(Lowered::Trap(trap));
                }
                let mut body_env = vec![value];
                body_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(builder, body, &body_env, eliminators)
            }
            RuntimeExpr::Call { callee, args } => {
                let callee = self.lower_expr(builder, callee, producer_env)?;
                match callee {
                    Lowered::DeclarationClosure {
                        symbol,
                        captures,
                        params,
                        body,
                    } => self.lower_recursive_declaration_call(
                        builder,
                        &symbol,
                        &captures,
                        &params,
                        &body,
                        args,
                        producer_env,
                        Some(eliminators),
                    ),
                    Lowered::Closure {
                        captures,
                        params,
                        body,
                    } => {
                        if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                            if let Some((cases, default)) =
                                ordinary_match_continuation(&params, &body)
                            {
                                let mut frame_env = captures;
                                frame_env.extend_from_slice(producer_env);
                                let mut composed = Vec::with_capacity(eliminators.len() + 1);
                                composed.push(EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                    cases,
                                    default,
                                    env: &frame_env,
                                    retained_scrutinee_index: Some(0),
                                    deferred_constructor_case: None,
                                }));
                                composed.extend_from_slice(eliminators);
                                return self.lower_computational_producer_expr(
                                    builder,
                                    &args[0],
                                    producer_env,
                                    &composed,
                                );
                            }
                        }
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "tree producer expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, producer_env))
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures);
                        call_env.extend_from_slice(producer_env);
                        self.lower_computational_producer_expr(
                            builder,
                            &body,
                            &call_env,
                            eliminators,
                        )
                    }
                    mut callee @ Lowered::ComputationalRecursorClosure { .. } => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(builder, &mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(callee);
                        let (activation, invocation) =
                            boundary.expect("recursor closure carries an invocation segment");
                        let current = active_recursor_frame(eliminators).ok_or_else(|| {
                            unsupported(
                                "ComputationalRecursor",
                                "recursive producer invocation has no active continuation",
                            )
                        })?;
                        let _resume = find_continuation_cursor(current, invocation.resume_cursor)
                            .ok_or_else(|| {
                            unsupported(
                                "ComputationalRecursor",
                                "recursive producer invocation cursor is not active",
                            )
                        })?;
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let mut composed = installed_oriented_eliminator_frames(&installed);
                        composed.push(EliminatorFrame::InvocationReturn);
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            let producer_kont = self.push_oriented_producer_kont(
                                builder,
                                installed.checked,
                                eliminators,
                            )?;
                            let outer_producer_kont = producer_kont.map(|producer_kont| {
                                self.active_partition_producer_kont.replace(producer_kont)
                            });
                            self.enter_oriented_semantic_region(installed.checked);
                            let returned = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &composed,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            if let Some(outer_producer_kont) = outer_producer_kont {
                                self.active_partition_producer_kont = outer_producer_kont;
                            }
                            let returned = returned?;
                            if matches!(returned, Lowered::ProcessExitStatus { .. }) {
                                return Ok(returned);
                            }
                            return self.lower_computational_match_value_composed(
                                builder,
                                returned,
                                eliminators,
                            );
                        }
                        let Lowered::Closure {
                            captures,
                            params,
                            body,
                        } = base
                        else {
                            return Err(unsupported(
                                "ComputationalMatch",
                                "recursive constructor field is not a closure",
                            ));
                        };
                        if params.len() != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "recursive field expects {} args but call provides {}",
                                    params.len(),
                                    args.len()
                                ),
                            ));
                        }
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, producer_env))
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures);
                        call_env.extend_from_slice(producer_env);
                        let producer_kont = self.push_oriented_producer_kont(
                            builder,
                            installed.checked,
                            eliminators,
                        )?;
                        let outer_producer_kont = producer_kont.map(|producer_kont| {
                            self.active_partition_producer_kont.replace(producer_kont)
                        });
                        self.enter_oriented_semantic_region(installed.checked);
                        let returned = self.lower_computational_producer_expr(
                            builder, &body, &call_env, &composed,
                        );
                        self.leave_oriented_semantic_region(installed.checked);
                        if let Some(outer_producer_kont) = outer_producer_kont {
                            self.active_partition_producer_kont = outer_producer_kont;
                        }
                        let returned = returned?;
                        if matches!(returned, Lowered::ProcessExitStatus { .. }) {
                            return Ok(returned);
                        }
                        self.lower_computational_match_value_composed(
                            builder,
                            returned,
                            eliminators,
                        )
                    }
                    _ => Err(unsupported(
                        "ComputationalMatch",
                        "tree producer callee is not a closure",
                    )),
                }
            }
            RuntimeExpr::Construct { constructor, args } => {
                let eliminator = eliminators[0];
                let terminal_exit = constructor == &self.process_symbols.exit_success
                    || constructor == &self.process_symbols.exit_failure;
                let itree_frame = match eliminator {
                    EliminatorFrame::Computational(frame) => frame
                        .cases
                        .iter()
                        .any(|case| case.constructor.contains("::ITree::")),
                    EliminatorFrame::Ordinary(frame) => frame
                        .cases
                        .iter()
                        .any(|case| case.constructor.contains("::ITree::")),
                    EliminatorFrame::PendingLet(_) => {
                        unreachable!("pending Let continuations are consumed before dispatch")
                    }
                    EliminatorFrame::InvocationReturn => {
                        unreachable!("invocation returns are consumed before dispatch")
                    }
                    EliminatorFrame::Active(_) => {
                        unreachable!("active continuation cursors do not consume constructors")
                    }
                };
                if terminal_exit && itree_frame {
                    let lowered_args = args
                        .iter()
                        .map(|arg| self.lower_expr(builder, arg, producer_env))
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(Lowered::Constructor {
                        constructor: constructor.clone(),
                        args: lowered_args,
                    });
                }
                let (case_body, argument_binder_offset) = match eliminator {
                    EliminatorFrame::Computational(eliminator) => {
                        let case = match eliminator
                            .cases
                            .iter()
                            .find(|case| case.constructor == *constructor)
                        {
                            Some(case) => case,
                            None => return Ok(Lowered::Trap(eliminator.default.clone())),
                        };
                        if case.argument_binders != args.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "case {} expects {} constructor arguments but value has {}",
                                    case.constructor,
                                    case.argument_binders,
                                    args.len()
                                ),
                            ));
                        }
                        let mut seen = BTreeSet::new();
                        for position in case.recursive_positions.iter().copied() {
                            if !seen.insert(position) || position >= args.len() {
                                return Err(unsupported(
                                    "ComputationalMatch",
                                    format!(
                                        "case {} has malformed recursive position {position}",
                                        case.constructor
                                    ),
                                ));
                            }
                        }
                        (&case.body, case.recursive_positions.len())
                    }
                    EliminatorFrame::Ordinary(eliminator) => {
                        let case = match select_ordinary_case(eliminator, constructor) {
                            Ok(case) => case,
                            Err(trap) => return Ok(Lowered::Trap(trap)),
                        };
                        if case.binders != args.len() {
                            return Err(unsupported(
                                "Match",
                                format!(
                                    "case {} expects {} binders but constructor has {} args",
                                    case.constructor,
                                    case.binders,
                                    args.len()
                                ),
                            ));
                        }
                        (&case.body, 0)
                    }
                    EliminatorFrame::PendingLet(_) => {
                        unreachable!("pending Let continuations are consumed before dispatch")
                    }
                    EliminatorFrame::InvocationReturn => {
                        unreachable!("invocation returns are consumed before dispatch")
                    }
                    EliminatorFrame::Active(_) => {
                        unreachable!("active continuation cursors do not consume constructors")
                    }
                };

                let bridge =
                    immediate_binder_eliminator(case_body, argument_binder_offset, args.len());
                let bridge =
                    bridge.filter(|(field, _)| requires_heterogeneous_deforestation(&args[*field]));

                if let Some((field, consumer)) = bridge {
                    let lowered_prefix = args[..field]
                        .iter()
                        .map(|arg| self.lower_expr(builder, arg, producer_env))
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(Lowered::Trap(trap)) = lowered_prefix
                        .iter()
                        .find(|value| matches!(value, Lowered::Trap(_)))
                    {
                        return Ok(Lowered::Trap(trap.clone()));
                    }

                    let splice_caller = active_recursor_frame(&eliminators[1..]);
                    let mut selected_ancestry = splice_caller
                        .map(|active| active.selected_ancestry.to_vec())
                        .unwrap_or_default();
                    if let EliminatorFrame::Computational(frame) = eliminator {
                        selected_ancestry.push(frame.provenance);
                    }
                    let mut pending: Vec<_> = eliminators[1..]
                        .iter()
                        .copied()
                        .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                        .collect();
                    if let Some(active) = splice_caller {
                        pending.extend_from_slice(active.pending);
                    }
                    let (activation_instance, cursor_instance, _) = self
                        .allocate_selected_control_refs(
                            builder,
                            splice_caller.map(|active| active.activation_instance),
                            splice_caller.map(|active| active.cursor_instance),
                            splice_caller
                                .and_then(|active| active.selected_scope)
                                .map(|scope| scope.scope_instance),
                        )?;
                    let selected_active = ActiveContinuationFrame {
                        activation: self.mint_continuation_activation(),
                        activation_instance,
                        cursor: self.mint_continuation_cursor(),
                        cursor_instance,
                        parent: splice_caller.and_then(|active| active.parent),
                        pending: &pending,
                        selected_ancestry: &selected_ancestry,
                        source_lineage: splice_caller
                            .map(|active| active.source_lineage)
                            .unwrap_or(&[]),
                        source_selected_cursor: splice_caller
                            .and_then(|active| active.source_selected_cursor),
                        selected_scope: splice_caller.and_then(|active| active.selected_scope),
                    };
                    let deferred = DeferredConstructorCaseEnvironment {
                        constructor,
                        lowered_prefix: &lowered_prefix,
                        selected_field: field,
                        trailing_fields: &args[field + 1..],
                        producer_env,
                        outer_eliminator: eliminator,
                        splice_caller,
                        selected_active,
                    };
                    let mut composed = Vec::with_capacity(2);
                    composed.push(match consumer {
                        ImmediateBinderEliminator::Computational { cases, default } => {
                            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                                provenance: self.mint_recursor_frame_provenance(),
                                checked_frame_id: None,
                                checked_invocation_id: None,
                                checked_invocation_source: None,
                                checked_invocation_depth: 0,
                            })
                        }
                        ImmediateBinderEliminator::Ordinary { cases, default } => {
                            EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                cases,
                                default,
                                env: &[],
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                            })
                        }
                    });
                    composed.push(EliminatorFrame::Active(selected_active));
                    return self.lower_computational_producer_expr(
                        builder,
                        &args[field],
                        producer_env,
                        &composed,
                    );
                }

                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, producer_env))
                    .collect::<Result<Vec<_>, _>>()?;
                self.lower_computational_match_value_composed(
                    builder,
                    Lowered::Constructor {
                        constructor: constructor.clone(),
                        args: lowered_args,
                    },
                    eliminators,
                )
            }
            RuntimeExpr::Match {
                scrutinee,
                cases: producer_cases,
                default: producer_default,
            } => {
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                if let Lowered::Bool { value, known } = selected {
                    let true_case = producer_cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = producer_cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "Bool tree producer requires True and False cases",
                        ));
                    };
                    if let Some(known) = known {
                        return self.lower_computational_producer_expr(
                            builder,
                            if known {
                                &true_case.body
                            } else {
                                &false_case.body
                            },
                            producer_env,
                            eliminators,
                        );
                    }
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder.append_block_param(merge, types::I64);
                    builder.ins().brif(value, true_block, &[], false_block, &[]);
                    let mut exit_merge = None;
                    for (block, producer_case) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let lowered = self.lower_computational_producer_expr(
                            builder,
                            &producer_case.body,
                            producer_env,
                            eliminators,
                        )?;
                        let (value, is_exit) =
                            self.merge_branch_value(builder, lowered, "ComputationalMatch")?;
                        Self::record_merge_kind("ComputationalMatch", &mut exit_merge, is_exit)?;
                        builder
                            .ins()
                            .jump(merge, &[value.tag.into(), value.payload.into()]);
                    }
                    builder.switch_to_block(merge);
                    let pair = NativeScalarPairV1 {
                        tag: builder.block_params(merge)[0],
                        payload: builder.block_params(merge)[1],
                    };
                    return Ok(if exit_merge == Some(true) {
                        Lowered::ProcessExitStatus {
                            value: pair.payload,
                        }
                    } else {
                        self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
                    });
                }
                if let Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                } = selected
                {
                    let host_result_is_admissible =
                        partition_lowered_is_admissible(&Lowered::HostResult {
                            success,
                            error: error.clone(),
                            ok: ok.clone(),
                            err_constructor: err_constructor.clone(),
                            ok_constructor: ok_constructor.clone(),
                        });
                    if self.partition_cut_armed
                        && (self.root_terminal_authority.is_some()
                            || self.active_partition_return_kind == Some(ScalarMergeKind::ExitCode))
                        && self.has_checked_root_exit_representation()
                        && self.active_oriented_semantic_regions == 0
                        && self.active_join_site.is_none()
                        && self.active_subcontinuation_frame.is_none()
                        && self.pending_recursive_call.is_none()
                        && self.pending_computational_ih_call.is_none()
                        && self.dynamic_splice_edges.is_empty()
                        && host_result_is_admissible
                    {
                        if let Some(owned_eliminators) = own_partition_eliminators(eliminators) {
                            if partition_eliminators_are_admissible(&owned_eliminators) {
                                return self.outline_computational_host_result_arms(
                                    builder,
                                    success,
                                    *error,
                                    *ok,
                                    &err_constructor,
                                    &ok_constructor,
                                    producer_cases,
                                    producer_default,
                                    producer_env,
                                    owned_eliminators,
                                );
                            }
                        }
                    }
                    return self.lower_computational_host_result(
                        builder,
                        success,
                        *error,
                        *ok,
                        &err_constructor,
                        &ok_constructor,
                        producer_cases,
                        producer_default,
                        producer_env,
                        eliminators,
                    );
                }
                if let Lowered::DynamicConstructor(dynamic) = selected {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Producer {
                            cases: producer_cases,
                            default: producer_default,
                            env: producer_env,
                            eliminators,
                        },
                    );
                }
                if let Lowered::BoundedNat(nat) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(builder, nat, false, &composed);
                }
                if let Lowered::StructuralNat(nat) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        &composed,
                    );
                }
                let Lowered::Constructor { constructor, args } = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match scrutinee is not Bool or a constructor",
                    ));
                };
                let Some(producer_case) = producer_cases
                    .iter()
                    .find(|case| case.constructor == constructor)
                else {
                    return Ok(Lowered::Trap(producer_default.clone()));
                };
                if producer_case.binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match constructor arity changed",
                    ));
                }
                let mut case_env = args;
                case_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(
                    builder,
                    &producer_case.body,
                    &case_env,
                    eliminators,
                )
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee: inner_scrutinee,
                cases: inner_cases,
                default: inner_default,
            } => {
                // Fuse the inner eliminator ahead of the outer stack. Its
                // selected case body remains a producer for every outer frame;
                // no intermediate aggregate is materialized or exit-lowered.
                let mut composed = Vec::with_capacity(eliminators.len() + 1);
                let provenance = self.mint_recursor_frame_provenance();
                let checked_frame_id =
                    self.consume_checked_subcontinuation_frame(inner_cases, inner_default)?;
                let checked_invocation_id = checked_frame_id.map(|_| {
                    self.active_recursive_invocations
                        .last()
                        .map_or(0, |instance| instance.invocation_instance_id)
                });
                let checked_invocation_depth = self
                    .active_recursive_invocations
                    .last()
                    .map_or(0, |instance| instance.semantic_depth);
                composed.push(EliminatorFrame::Computational(
                    ComputationalEliminatorFrame {
                        cases: inner_cases,
                        default: inner_default,
                        env: producer_env,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                        provenance,
                        checked_frame_id,
                        checked_invocation_id,
                        checked_invocation_source: self
                            .active_recursive_invocations
                            .last()
                            .map(|instance| instance.source),
                        checked_invocation_depth,
                    },
                ));
                composed.extend_from_slice(eliminators);
                self.lower_computational_producer_expr(
                    builder,
                    inner_scrutinee,
                    producer_env,
                    &composed,
                )
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                let Lowered::Bool { value, known } = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing If scrutinee is not Bool",
                    ));
                };
                if let Some(known) = known {
                    return self.lower_computational_producer_expr(
                        builder,
                        if known { then_expr } else { else_expr },
                        producer_env,
                        eliminators,
                    );
                }
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                let mut exit_merge = None;
                for (block, branch) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_computational_producer_expr(
                        builder,
                        branch,
                        producer_env,
                        eliminators,
                    )?;
                    let (value, is_exit) =
                        self.merge_branch_value(builder, lowered, "ComputationalMatch")?;
                    Self::record_merge_kind("ComputationalMatch", &mut exit_merge, is_exit)?;
                    builder
                        .ins()
                        .jump(merge, &[value.tag.into(), value.payload.into()]);
                }
                builder.switch_to_block(merge);
                let pair = NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                };
                Ok(if exit_merge == Some(true) {
                    Lowered::ProcessExitStatus {
                        value: pair.payload,
                    }
                } else {
                    self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
                })
            }
            _ => {
                let value = self.lower_expr(builder, scrutinee, producer_env)?;
                self.lower_computational_match_value_composed(builder, value, eliminators)
            }
        }
    }

    fn push_oriented_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        checked: bool,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Option<PartitionProducerKontCursor>, CraneliftBackendError> {
        let owned = own_partition_eliminators(eliminators).ok_or_else(|| {
            unsupported(
                "NativeProducerContinuationStepV1",
                "oriented producer return has no exact eliminator schema",
            )
        })?;
        if !partition_eliminators_are_admissible(&owned) {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "oriented producer return carries an inadmissible eliminator",
            ));
        }
        let (checked_join, return_kind) =
            if let Some(return_contract) = self.active_partition_return_contract.clone() {
                (return_contract.checked_join, return_contract.required_kind)
            } else {
                let checked_join = self
                    .native_join_plan
                    .as_ref()
                    .and_then(|plan| {
                        plan.sites.iter().find(|site| {
                            site.runtime_frame_fingerprint
                                == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                                && site.checked_occurrence_path == [0]
                                && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
                        })
                    })
                    .map(PartitionCheckedJoinIdentity::from);
                let Some(checked_join) = checked_join else {
                    return Ok(None);
                };
                (checked_join, ScalarMergeKind::ExitCode)
            };
        let outer = if let Some(outer) = self.active_partition_producer_kont {
            outer
        } else {
            self.push_done_producer_kont(
                builder,
                checked_join.clone(),
                return_kind,
                PartitionProducerKontTerminalIdentity::CheckedJoin,
            )?
        };
        let successor = if owned.is_empty() {
            outer
        } else {
            self.push_owned_producer_kont(builder, owned, outer, checked_join.clone(), return_kind)?
        };
        self.push_oriented_return_producer_kont(
            builder,
            checked,
            successor,
            checked_join,
            return_kind,
        )
        .map(Some)
    }

    fn push_owned_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        owned: Vec<OwnedPartitionEliminator>,
        mut successor: PartitionProducerKontCursor,
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
    ) -> Result<PartitionProducerKontCursor, CraneliftBackendError> {
        for eliminator in owned.into_iter().rev() {
            successor = self.push_one_owned_producer_kont(
                builder,
                eliminator,
                successor,
                checked_join.clone(),
                return_kind,
            )?;
        }
        Ok(successor)
    }

    fn intern_producer_kont_site(
        &mut self,
        plan: PartitionProducerKontSitePlan,
    ) -> Result<usize, CraneliftBackendError> {
        let key = PartitionProducerKontSiteKey::new(
            &plan.action,
            plan.successor,
            plan.checked_join.clone(),
            plan.return_kind,
        );
        if let Some(site_id) = self.partition_producer_site_interner.lookup(&key) {
            return Ok(site_id);
        }
        let site_id = self.partition_next_producer_site;
        self.partition_next_producer_site = self
            .partition_next_producer_site
            .checked_add(1)
            .ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "producer continuation site identity exhausted",
                )
            })?;
        self.partition_producer_site_interner.insert(site_id, key)?;
        if self
            .partition_producer_sites
            .insert(site_id, plan)
            .is_some()
        {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "producer continuation site was planned twice",
            ));
        }
        Ok(site_id)
    }

    fn push_done_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
        terminal: PartitionProducerKontTerminalIdentity,
    ) -> Result<PartitionProducerKontCursor, CraneliftBackendError> {
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(0)?,
            3,
        ));
        self.partition_metrics.record_call_frame(0);
        let invocation = self
            .invocation_pointer
            .expect("producer terminal planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let capture_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let plan = PartitionProducerKontSitePlan {
            action: ProducerKontAction::Done { terminal },
            successor: None,
            checked_join,
            return_kind,
        };
        let site_id = self.intern_producer_kont_site(plan)?;
        Ok(PartitionProducerKontCursor {
            site_id,
            capture_pointer,
        })
    }

    fn push_one_owned_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        eliminator: OwnedPartitionEliminator,
        successor: PartitionProducerKontCursor,
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
    ) -> Result<PartitionProducerKontCursor, CraneliftBackendError> {
        let owned = vec![eliminator];
        let mut capture_fields = Vec::new();
        append_partition_eliminator_values(self, builder, &owned, &mut capture_fields)?;
        capture_fields.push(successor.capture_pointer);
        let capture_field_types = capture_fields
            .iter()
            .map(|value| builder.func.dfg.value_type(*value))
            .collect::<Vec<_>>();
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(capture_fields.len())?,
            3,
        ));
        self.partition_metrics
            .record_call_frame(capture_fields.len());
        for (index, value) in capture_fields.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeProducerContinuationStepV1",
                        "producer continuation capture-cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("producer continuation planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let capture_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let action = ProducerKontAction::ApplyEliminators {
            eliminators: owned,
            capture_field_types,
        };
        let plan = PartitionProducerKontSitePlan {
            action,
            successor: Some(successor),
            checked_join,
            return_kind,
        };
        let site_id = self.intern_producer_kont_site(plan)?;
        Ok(PartitionProducerKontCursor {
            site_id,
            capture_pointer,
        })
    }

    fn push_exit_scope_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        popped: &PoppedPartitionRecursorLayer,
        successor: PartitionProducerKontCursor,
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
    ) -> Result<PartitionProducerKontCursor, CraneliftBackendError> {
        let target = popped.target.ok_or_else(|| {
            unsupported(
                "NativeExitScopeTransitionV1",
                "partitioned scope exit lost its exact recursor cell",
            )
        })?;
        let obligation = popped.exit_obligation.ok_or_else(|| {
            unsupported(
                "NativeExitScopeTransitionV1",
                "partitioned scope exit lost its exact obligation cell",
            )
        })?;
        let RecursorLayerRole::ExitsScope {
            scope_instance,
            parent_scope_instance,
            ..
        } = popped.layer.role
        else {
            return Err(unsupported(
                "NativeExitScopeTransitionV1",
                "exit-scope producer transition received a non-exit layer",
            ));
        };
        let invocation = self
            .invocation_pointer
            .expect("scope-exit producer planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let null = builder.ins().iconst(pointer_type, 0);
        let parent_pointer = parent_scope_instance.map_or(null, |parent| parent.0);
        let obligation_successor_pointer = popped
            .exit_obligation_successor
            .map_or(null, |cursor| cursor.capture_pointer);

        let complete_fields = [
            obligation.capture_pointer,
            target.capture_pointer,
            scope_instance.0,
            parent_pointer,
            obligation_successor_pointer,
            successor.capture_pointer,
        ];
        let complete_frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(complete_fields.len())?,
            3,
        ));
        self.partition_metrics
            .record_call_frame(complete_fields.len());
        for (index, value) in complete_fields.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeExitScopeTransitionV1",
                        "scope-exit completion cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, complete_frame, offset);
        }
        let complete_capture_pointer = builder.ins().stack_addr(pointer_type, complete_frame, 0);
        let complete_plan = PartitionProducerKontSitePlan {
            action: ProducerKontAction::ExitScopeComplete {
                target: target.node,
                obligation: obligation.node,
                obligation_successor: popped.exit_obligation_successor.map(|cursor| cursor.node),
            },
            successor: Some(successor),
            checked_join: checked_join.clone(),
            return_kind,
        };
        let complete_site_id = self.intern_producer_kont_site(complete_plan)?;
        let complete = PartitionProducerKontCursor {
            site_id: complete_site_id,
            capture_pointer: complete_capture_pointer,
        };

        let start_fields = [
            obligation.capture_pointer,
            target.capture_pointer,
            scope_instance.0,
            parent_pointer,
            complete.capture_pointer,
        ];
        let start_frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(start_fields.len())?,
            3,
        ));
        self.partition_metrics.record_call_frame(start_fields.len());
        for (index, value) in start_fields.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeExitScopeTransitionV1",
                        "scope-exit start cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, start_frame, offset);
        }
        let start_capture_pointer = builder.ins().stack_addr(pointer_type, start_frame, 0);
        let start_plan = PartitionProducerKontSitePlan {
            action: ProducerKontAction::ExitScopeStart {
                target: target.node,
                obligation: obligation.node,
            },
            successor: Some(complete),
            checked_join,
            return_kind,
        };
        let start_site_id = self.intern_producer_kont_site(start_plan)?;
        Ok(PartitionProducerKontCursor {
            site_id: start_site_id,
            capture_pointer: start_capture_pointer,
        })
    }

    fn install_partition_exit_scope_chain<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        selected: &mut SourceSelectedContinuation<'b>,
        selected_lineage: &[SourceSelectedContinuation<'b>],
        producer_kont: &mut Option<PartitionProducerKontCursor>,
        popped: &[PoppedPartitionRecursorLayer],
    ) -> Result<(), CraneliftBackendError> {
        if popped.is_empty() {
            return Ok(());
        }
        let contract = self
            .active_partition_return_contract
            .clone()
            .ok_or_else(|| {
                unsupported(
                    "NativeExitScopeTransitionV1",
                    "persistent scope exit has no scalar return contract",
                )
            })?;
        let outer = producer_kont.ok_or_else(|| {
            unsupported(
                "NativeExitScopeTransitionV1",
                "persistent scope exit has no explicit parent producer successor",
            )
        })?;
        let owned_provenance = popped
            .iter()
            .map(|step| step.layer.provenance)
            .collect::<Vec<_>>();
        selected.pending.retain(|frame| match frame {
            EliminatorFrame::Computational(frame) => !owned_provenance
                .iter()
                .any(|owned| *owned == frame.provenance),
            _ => true,
        });
        if selected.selected_scope.as_ref().is_some_and(|scope| {
            owned_provenance
                .iter()
                .any(|owned| *owned == scope.frame.provenance)
        }) {
            selected.selected_scope = None;
        }

        let previous = self.active_partition_producer_kont.replace(outer);
        let generic_parent = self
            .push_selected_head_producer_kont(builder, selected.as_active(selected_lineage))?
            .ok_or_else(|| {
                unsupported(
                    "NativeExitScopeTransitionV1",
                    "scope-exit chain could not seal its generic parent successor",
                )
            })?;
        self.active_partition_producer_kont = previous;
        let mut head = generic_parent;
        for step in popped.iter().rev() {
            head = self.push_exit_scope_producer_kont(
                builder,
                step,
                head,
                contract.checked_join.clone(),
                contract.required_kind,
            )?;
        }
        // The exact remaining selected work is sealed behind the exit chain.
        // The eventual source terminal contributes only an obligation-free
        // pass-through selected head.
        selected.pending.clear();
        selected.selected_scope = None;
        *producer_kont = Some(head);
        Ok(())
    }

    fn install_source_recursor_invocation<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut control: SourceControl<'b>,
        activation: ContinuationActivationId,
        invocation: RecursorInvocationSegment,
        checked_ih_invocation: Option<CheckedRecursiveInvocationInstance>,
    ) -> Result<SourceControl<'b>, CraneliftBackendError> {
        if invocation.unwind.partition_cursor.is_none() {
            control.continuation = self.install_recursor_invocation(
                control.continuation,
                activation,
                invocation,
                checked_ih_invocation,
            )?;
            return Ok(control);
        }

        let _dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
        let resume_active = source_active_cursor(
            &control.selected,
            &control.selected_lineage,
            invocation.resume_cursor,
        )
        .ok_or_else(|| {
            unsupported(
                "NativeExitScopeTransitionV1",
                "partitioned recursor invocation lost its exact resume owner",
            )
        })?;
        self.emit_control_cell_ref_guard(
            builder,
            &[(
                resume_active.cursor_instance.0,
                invocation.resume_cursor_instance.0,
            )],
        );
        let RecursorInvocationSegment {
            selection,
            mut unwind,
            ..
        } = invocation;
        let popped = self.drain_partition_exit_stack(builder, &mut unwind)?;
        let mut producer_selected = control.selected.clone();
        let producer_lineage = control.selected_lineage.clone();
        self.install_partition_exit_scope_chain(
            builder,
            &mut producer_selected,
            &producer_lineage,
            &mut control.producer_kont,
            &popped,
        )?;
        control.continuation = SourceContinuation::ApplyRecursorSelection {
            layer: selection,
            next: Box::new(control.continuation),
        };
        Ok(control)
    }

    fn push_oriented_return_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        checked: bool,
        successor: PartitionProducerKontCursor,
        checked_join: PartitionCheckedJoinIdentity,
        return_kind: ScalarMergeKind,
    ) -> Result<PartitionProducerKontCursor, CraneliftBackendError> {
        let capture_fields = vec![successor.capture_pointer];
        let capture_field_types = capture_fields
            .iter()
            .map(|value| builder.func.dfg.value_type(*value))
            .collect::<Vec<_>>();
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(capture_fields.len())?,
            3,
        ));
        self.partition_metrics
            .record_call_frame(capture_fields.len());
        for (index, value) in capture_fields.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeProducerContinuationStepV1",
                        "oriented-return capture-cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("producer continuation planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let capture_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let plan = PartitionProducerKontSitePlan {
            action: ProducerKontAction::OrientedInvocationReturn {
                checked,
                capture_field_types,
            },
            successor: Some(successor),
            checked_join,
            return_kind,
        };
        let site_id = self.intern_producer_kont_site(plan)?;
        Ok(PartitionProducerKontCursor {
            site_id,
            capture_pointer,
        })
    }

    fn push_selected_head_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        active: ActiveContinuationFrame<'_>,
    ) -> Result<Option<PartitionProducerKontCursor>, CraneliftBackendError> {
        let Some(return_contract) = self.active_partition_return_contract.clone() else {
            return Ok(None);
        };
        if active.parent.is_some() {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "entry selected head still carries a borrowed parent frame",
            ));
        }
        let pending = own_partition_eliminators(active.pending).ok_or_else(|| {
            unsupported(
                "NativeProducerContinuationStepV1",
                "entry selected head pending frames have no exact schema",
            )
        })?;
        let selected_lineage =
            own_partition_selected_lineage(active.source_lineage).ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "entry selected head lineage has no exact schema",
                )
            })?;
        let selected_scope = active.selected_scope.cloned();
        if !partition_scope_is_admissible(&selected_scope)
            || !partition_eliminators_are_admissible(&pending)
        {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "entry selected head has no exact admissible capture schema",
            ));
        }
        let successor = self.active_partition_producer_kont.ok_or_else(|| {
            unsupported(
                "NativeProducerContinuationStepV1",
                "selected-head producer step has no explicit successor",
            )
        })?;
        let mut capture_fields = Vec::new();
        capture_fields.push(active.activation_instance.0);
        capture_fields.push(active.cursor_instance.0);
        append_partition_eliminator_values(self, builder, &pending, &mut capture_fields)?;
        append_partition_scope_values(self, builder, &selected_scope, &mut capture_fields)?;
        capture_fields.push(successor.capture_pointer);
        let capture_field_types = capture_fields
            .iter()
            .map(|value| builder.func.dfg.value_type(*value))
            .collect::<Vec<_>>();
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(capture_fields.len())?,
            3,
        ));
        self.partition_metrics
            .record_call_frame(capture_fields.len());
        for (index, value) in capture_fields.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeProducerContinuationStepV1",
                        "selected-head capture-cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("producer continuation planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let capture_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let plan = PartitionProducerKontSitePlan {
            action: ProducerKontAction::ApplyActiveEliminators {
                activation: active.activation,
                activation_instance: active.activation_instance,
                cursor: active.cursor,
                cursor_instance: active.cursor_instance,
                pending,
                selected_ancestry: active.selected_ancestry.to_vec(),
                selected_scope,
                selected_lineage,
                capture_field_types,
            },
            successor: Some(successor),
            checked_join: return_contract.checked_join,
            return_kind: return_contract.required_kind,
        };
        let site_id = self.intern_producer_kont_site(plan)?;
        Ok(Some(PartitionProducerKontCursor {
            site_id,
            capture_pointer,
        }))
    }

    fn call_restored_selected_producer_kont<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        successor: PartitionProducerKontCursor,
        value: Lowered,
        selected: &SourceSelectedContinuation<'b>,
        selected_lineage: &[SourceSelectedContinuation<'b>],
    ) -> Result<Lowered, CraneliftBackendError> {
        let previous = self.active_partition_producer_kont.replace(successor);
        let active = selected.as_active(selected_lineage);
        let head = self
            .push_selected_head_producer_kont(builder, active)?
            .ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "restored selected head has no explicit producer transition",
                )
            })?;
        self.active_partition_producer_kont = previous;
        self.call_partition_producer_kont(builder, head, value)
    }

    fn producer_kont_is_scope_exit_bridge(
        &self,
        cursor: PartitionProducerKontCursor,
    ) -> Result<bool, CraneliftBackendError> {
        let plan = self
            .partition_producer_sites
            .get(&cursor.site_id)
            .ok_or_else(|| {
                unsupported(
                    "NativeExitScopeTransitionV1",
                    "scope-exit source bridge lost its planned producer successor",
                )
            })?;
        Ok(matches!(
            plan.action,
            ProducerKontAction::ExitScopeStart { .. }
                | ProducerKontAction::ExitScopeComplete { .. }
        ))
    }

    fn producer_kont_starts_selected_scope_exit(
        &self,
        cursor: PartitionProducerKontCursor,
        selected_scope: Option<&OwnedSelectedScope>,
    ) -> Result<bool, CraneliftBackendError> {
        let plan = self
            .partition_producer_sites
            .get(&cursor.site_id)
            .ok_or_else(|| {
                unsupported(
                    "NativeExitScopeTransitionV1",
                    "selected return lost its planned scope-exit successor",
                )
            })?;
        let ProducerKontAction::ExitScopeStart { target, .. } = plan.action else {
            return Ok(false);
        };
        let target = self.partition_recursor_nodes.definition(target)?;
        let target_scope_origin = match target.current.role {
            RecursorLayerRole::ExitsScope { scope_origin, .. } => Some(scope_origin),
            RecursorLayerRole::SelectsOccurrence { .. } => None,
        };
        let matches = selected_scope.is_some_and(|scope| {
            scope.frame.provenance == target.current.provenance
                && scope.frame.checked_frame_id == target.current.checked_frame_id
                && Some(scope.scope_origin) == target_scope_origin
        });
        Ok(matches)
    }

    fn push_checked_computational_ih_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        call_template_id: u64,
    ) -> Result<Option<PartitionProducerKontCursor>, CraneliftBackendError> {
        let Some(return_contract) = self.active_partition_return_contract.clone() else {
            return Ok(None);
        };
        let Some(successor) = self.active_partition_producer_kont else {
            return Ok(None);
        };
        let capture_fields = vec![successor.capture_pointer];
        let capture_field_types = capture_fields
            .iter()
            .map(|value| builder.func.dfg.value_type(*value))
            .collect::<Vec<_>>();
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(capture_fields.len())?,
            3,
        ));
        self.partition_metrics
            .record_call_frame(capture_fields.len());
        for (index, value) in capture_fields.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeProducerContinuationStepV1",
                        "checked-marker capture-cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("producer continuation planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let capture_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let plan = PartitionProducerKontSitePlan {
            action: ProducerKontAction::CheckedComputationalIHReturn {
                call_template_id,
                capture_field_types,
            },
            successor: Some(successor),
            checked_join: return_contract.checked_join,
            return_kind: return_contract.required_kind,
        };
        let site_id = self.intern_producer_kont_site(plan)?;
        Ok(Some(PartitionProducerKontCursor {
            site_id,
            capture_pointer,
        }))
    }

    fn call_partition_producer_kont(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        cursor: PartitionProducerKontCursor,
        input: Lowered,
    ) -> Result<Lowered, CraneliftBackendError> {
        if !partition_lowered_is_admissible(&input) {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "producer continuation input has no exact closed schema",
            ));
        }
        let plan = self
            .partition_producer_sites
            .get(&cursor.site_id)
            .cloned()
            .ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "producer continuation head has no planned site",
                )
            })?;
        if !matches!(&plan.action, ProducerKontAction::Done { .. }) && plan.successor.is_none() {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "nonterminal producer continuation has no explicit successor",
            ));
        }
        let mut fields = Vec::new();
        append_partition_lowered_values(self, builder, &input, &mut fields)?;
        fields.push(cursor.capture_pointer);
        let (frame_values, field_types, field_map) = partition_frame_layout(builder, &fields);
        let key = PartitionSemanticStateKey::ProducerKont(match &plan.action {
            ProducerKontAction::Done { terminal } => PartitionContinuationKey::done(
                plan.checked_join.clone(),
                plan.return_kind,
                &input,
                *terminal,
                field_types.clone(),
                field_map.clone(),
            ),
            ProducerKontAction::OrientedInvocationReturn { checked, .. } => {
                PartitionContinuationKey::oriented_invocation_return(
                    plan.checked_join.clone(),
                    plan.return_kind,
                    &input,
                    *checked,
                    plan.successor.map(|successor| successor.site_id),
                    field_types.clone(),
                    field_map.clone(),
                )
            }
            ProducerKontAction::ApplyEliminators { eliminators, .. } => {
                PartitionContinuationKey::apply_eliminators(
                    plan.checked_join.clone(),
                    plan.return_kind,
                    &input,
                    eliminators,
                    plan.successor.map(|successor| successor.site_id),
                    field_types.clone(),
                    field_map.clone(),
                )
            }
            ProducerKontAction::CheckedComputationalIHReturn {
                call_template_id, ..
            } => PartitionContinuationKey::checked_computational_ih_return(
                plan.checked_join.clone(),
                plan.return_kind,
                &input,
                *call_template_id,
                plan.successor.map(|successor| successor.site_id),
                field_types.clone(),
                field_map.clone(),
            ),
            ProducerKontAction::ExitScopeStart { target, obligation } => {
                PartitionContinuationKey::exit_scope_start(
                    plan.checked_join.clone(),
                    plan.return_kind,
                    &input,
                    *target,
                    *obligation,
                    plan.successor.map(|successor| successor.site_id),
                    field_types.clone(),
                    field_map.clone(),
                )
            }
            ProducerKontAction::ExitScopeComplete {
                target,
                obligation,
                obligation_successor,
            } => PartitionContinuationKey::exit_scope_complete(
                plan.checked_join.clone(),
                plan.return_kind,
                &input,
                *target,
                *obligation,
                *obligation_successor,
                plan.successor.map(|successor| successor.site_id),
                field_types.clone(),
                field_map.clone(),
            ),
            ProducerKontAction::ApplyActiveEliminators {
                activation,
                cursor,
                pending,
                selected_ancestry,
                selected_scope,
                selected_lineage,
                ..
            } => PartitionContinuationKey::new(
                plan.checked_join.clone(),
                plan.return_kind,
                plan.return_kind,
                &input,
                *activation,
                *cursor,
                pending,
                selected_ancestry,
                selected_scope,
                selected_lineage,
                plan.successor.map(|successor| successor.site_id),
                field_types.clone(),
                field_map.clone(),
            ),
        });
        let expected_contract = key.return_contract();
        let existing = self
            .partition_continuations
            .lookup(&key, PartitionAggregateBudget::PRODUCTION)?;
        let (state_id, state, newly_reserved) = if let Some((state_id, state)) = existing {
            (state_id, state, false)
        } else {
            let helper_index = self.partition_next_helper;
            let function = *self.partition_helper_ids.get(helper_index).ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "producer continuation exhausted its predeclared helper pool",
                )
            })?;
            self.partition_next_helper += 1;
            let (state_id, state) = self.partition_continuations.reserve(
                key,
                function,
                helper_index,
                PartitionAggregateBudget::PRODUCTION,
            )?;
            (state_id, state, true)
        };
        self.partition_continuations
            .validate_call_contract(state_id, &expected_contract)?;
        let (_, function_ref) = self.partition_helper_ref(
            builder,
            state.helper_index,
            "interned producer continuation lost its helper identity",
        )?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(frame_values.len())?,
            3,
        ));
        self.partition_metrics.record_call_frame(frame_values.len());
        for (index, value) in frame_values.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeProducerContinuationStepV1",
                        "producer continuation input-cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("producer continuation call owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES,
            3,
        ));
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().stack_store(zero, tag_output, 0);
        let tag_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
        let call = builder
            .ins()
            .call(function_ref, &[invocation, frame_pointer, tag_pointer]);
        let tag = builder.ins().stack_load(types::I64, tag_output, 0);
        let payload = builder.inst_results(call)[0];
        if newly_reserved {
            self.partition_queue
                .push_back(PartitionWorkItem::ProducerKont(
                    ProducerKontPartitionWorkItem {
                        state_id,
                        site_id: cursor.site_id,
                        function: state.function,
                        field_types,
                        field_map,
                        value: input,
                        action: plan.action,
                        capture_pointer: Some(cursor.capture_pointer),
                        successor: plan.successor,
                        ledger_baseline: self.partition_ledger_baseline(),
                        declaration_stack: self.declaration_stack.clone(),
                        active_recursive_invocations: self.active_recursive_invocations.clone(),
                        checked_join: plan.checked_join,
                        return_kind: plan.return_kind,
                    },
                ));
        }
        Ok(self.lowered_from_scalar_pair(plan.return_kind, NativeScalarPairV1 { tag, payload }))
    }

    fn lower_computational_match_value_composed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: Lowered,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        let Some(eliminator) = eliminators.first().copied() else {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        };
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(scrutinee);
        }
        if let Lowered::BoundedNat(nat) = scrutinee {
            return self.lower_bounded_nat_computational(builder, nat, false, eliminators);
        }
        if let Lowered::StructuralNat(nat) = scrutinee {
            return self.lower_bounded_nat_computational(
                builder,
                BoundedNatV1::derived_from_validated(nat.value),
                true,
                eliminators,
            );
        }
        let Lowered::Constructor { constructor, args } = scrutinee else {
            return Err(unsupported(
                "ComputationalMatch",
                "scrutinee is not a constructor value after ordinary expression lowering",
            ));
        };
        let retained_scrutinee = Lowered::Constructor {
            constructor: constructor.clone(),
            args: args.clone(),
        };
        let remaining_eliminators = &eliminators[1..];
        let (body, case_env) = match eliminator {
            EliminatorFrame::Computational(eliminator) => {
                let (case, _) = match select_computational_case(
                    std::slice::from_ref(&eliminator),
                    &constructor,
                ) {
                    Ok(selected) => selected,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                if case.argument_binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} expects {} constructor arguments but value has {}",
                            case.constructor,
                            case.argument_binders,
                            args.len()
                        ),
                    ));
                }
                let mut seen = BTreeSet::new();
                for position in case.recursive_positions.iter().copied() {
                    if !seen.insert(position) || position >= args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "case {} has malformed recursive position {position}",
                                case.constructor
                            ),
                        ));
                    }
                }

                let splice_caller = active_recursor_frame(remaining_eliminators);
                let mut selected_ancestry = splice_caller
                    .map(|active| active.selected_ancestry.to_vec())
                    .unwrap_or_default();
                selected_ancestry.push(eliminator.provenance);
                let mut pending: Vec<_> = remaining_eliminators
                    .iter()
                    .copied()
                    .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                    .collect();
                if let Some(active) = splice_caller {
                    pending.extend_from_slice(active.pending);
                }
                let activation = self.mint_continuation_activation();
                let cursor = self.mint_continuation_cursor();
                let producer_origin = self.mint_recursor_producer_origin();
                let (activation_instance, cursor_instance, scope_instance) = self
                    .allocate_selected_control_refs(
                        builder,
                        splice_caller.map(|active| active.activation_instance),
                        splice_caller.map(|active| active.cursor_instance),
                        splice_caller
                            .and_then(|active| active.selected_scope)
                            .map(|scope| scope.scope_instance),
                    )?;
                let selected_scope = OwnedSelectedScope {
                    scope_origin: producer_origin,
                    scope_instance,
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                    parent_scope_instance: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_instance),
                    frame: ComputationalRecursorFramePayload {
                        cases: eliminator.cases.to_vec(),
                        default: eliminator.default.clone(),
                        outer_env: eliminator.env.to_vec(),
                        provenance: eliminator.provenance,
                        checked_frame_id: eliminator.checked_frame_id,
                        checked_invocation_id: eliminator.checked_invocation_id,
                        checked_invocation_source: eliminator.checked_invocation_source,
                        checked_invocation_depth: eliminator.checked_invocation_depth,
                    },
                };
                let selected_scope = Some(selected_scope);
                let active_state = ActiveContinuationFrame {
                    activation,
                    activation_instance,
                    cursor,
                    cursor_instance,
                    parent: splice_caller.and_then(|active| active.parent),
                    pending: &pending,
                    selected_ancestry: &selected_ancestry,
                    source_lineage: splice_caller
                        .map(|active| active.source_lineage)
                        .unwrap_or(&[]),
                    source_selected_cursor: splice_caller
                        .and_then(|active| active.source_selected_cursor),
                    selected_scope: selected_scope.as_ref(),
                };

                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::Composed,
                    origin: producer_origin,
                    cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                });
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                let ih_slots =
                    self.computational_ih_slots_for_case(case, eliminator.checked_frame_id)?;
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let induction_hypothesis = self.make_computational_recursor(
                        builder,
                        args[position].clone(),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.provenance,
                        eliminator.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                            origin_scope: scope_instance,
                        },
                        activation,
                        cursor,
                        cursor_instance,
                        Some(&active_state),
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
                    induction_hypotheses.push(induction_hypothesis);
                }
                let mut case_env = induction_hypotheses;
                case_env.extend(args);
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Computational(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                case_env.extend(frame_env);
                if !case.recursive_positions.is_empty() {
                    return self.lower_source_machine(
                        builder,
                        &case.body,
                        &case_env,
                        &active_state,
                    );
                }
                if remaining_eliminators.is_empty() {
                    return self.lower_expr(builder, &case.body, &case_env);
                }
                return self.lower_computational_producer_expr(
                    builder,
                    &case.body,
                    &case_env,
                    remaining_eliminators,
                );
            }
            EliminatorFrame::Ordinary(eliminator) => {
                let case = match select_ordinary_case(eliminator, &constructor) {
                    Ok(case) => case,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                if case.binders != args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            args.len()
                        ),
                    ));
                }
                let mut case_env = args;
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Ordinary(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(Lowered::Trap(trap)),
                };
                case_env.extend(frame_env);
                (&case.body, case_env)
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before value composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before value composition")
            }
            EliminatorFrame::Active(active) => {
                return self.resume_active_continuation(builder, retained_scrutinee, active);
            }
        };
        if remaining_eliminators.is_empty() {
            self.lower_expr(builder, body, &case_env)
        } else {
            self.lower_computational_producer_expr(builder, body, &case_env, remaining_eliminators)
        }
    }

    fn lower_bounded_nat_computational(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<Lowered, CraneliftBackendError> {
        let eliminator = eliminators[0];
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(if structural {
                Lowered::StructuralNat(StructuralNatV1 { value: nat.value })
            } else {
                Lowered::BoundedNat(nat)
            });
        }
        if let EliminatorFrame::Active(active) = eliminator {
            let value = if structural {
                Lowered::StructuralNat(StructuralNatV1 { value: nat.value })
            } else {
                Lowered::BoundedNat(nat)
            };
            return self.resume_active_continuation(builder, value, active);
        }
        let remaining = &eliminators[1..];
        let (zero_body, suc_body, computational) = match eliminator {
            EliminatorFrame::Computational(frame) => {
                let zero = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_zero
                        && case.argument_binders == 0
                        && case.recursive_positions.is_empty()
                });
                let suc = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_suc
                        && case.argument_binders == 1
                        && case.recursive_positions.as_slice() == [0]
                });
                let (Some(zero), Some(suc)) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "computational Nat requires Zero and one recursive Suc predecessor",
                    ));
                };
                (&zero.body, &suc.body, true)
            }
            EliminatorFrame::Ordinary(frame) => {
                let zero = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_zero && case.binders == 0
                });
                let suc = frame.cases.iter().find(|case| {
                    case.constructor == self.process_symbols.nat_suc && case.binders == 1
                });
                let (Some(zero), Some(suc)) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "ordinary Nat frame requires exact Zero and Suc predecessor arms",
                    ));
                };
                (&zero.body, &suc.body, false)
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before Nat composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before Nat composition")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors do not consume Nat values")
            }
        };

        let zero_value = builder.ins().iconst(types::I64, 0);
        let zero_nat = if structural {
            Lowered::StructuralNat(StructuralNatV1 { value: zero_value })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(zero_value))
        };
        let zero_frame_env =
            match self.materialize_eliminator_frame_env(builder, eliminator, &zero_nat)? {
                Ok(env) => env,
                Err(trap) => return Ok(Lowered::Trap(trap)),
            };
        let zero_lowered = if remaining.is_empty() {
            self.lower_expr(builder, zero_body, &zero_frame_env)?
        } else {
            self.lower_computational_producer_expr(builder, zero_body, &zero_frame_env, remaining)?
        };
        let (initial, result_kind) =
            self.merge_scalar_branch(builder, zero_lowered, "BoundedNat")?;

        let loop_block = builder.create_block();
        let step_block = builder.create_block();
        let done_block = builder.create_block();
        #[cfg(test)]
        let break_decrement =
            self.bounded_nat_mutation == BoundedNatLoweringMutation::BrokenDecrement;
        #[cfg(not(test))]
        let break_decrement = false;
        #[cfg(test)]
        let expose_raw_predecessor =
            self.bounded_nat_mutation == BoundedNatLoweringMutation::RawScalarPredecessor;
        #[cfg(not(test))]
        let expose_raw_predecessor = false;
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        if break_decrement {
            builder.append_block_param(loop_block, types::I64);
        }
        builder.append_block_param(done_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        if break_decrement {
            builder.ins().jump(
                loop_block,
                &[
                    zero_value.into(),
                    initial.tag.into(),
                    initial.payload.into(),
                    zero_value.into(),
                ],
            );
        } else {
            builder.ins().jump(
                loop_block,
                &[
                    zero_value.into(),
                    initial.tag.into(),
                    initial.payload.into(),
                ],
            );
        }

        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction = NativeScalarPairV1 {
            tag: builder.block_params(loop_block)[1],
            payload: builder.block_params(loop_block)[2],
        };
        if break_decrement {
            let fuel = builder.block_params(loop_block)[3];
            let compare_block = builder.create_block();
            let exhausted = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
                fuel,
                nat.value,
            );
            let nontermination = builder.ins().iconst(types::I64, -2);
            builder.ins().brif(
                exhausted,
                done_block,
                &[zero_value.into(), nontermination.into()],
                compare_block,
                &[],
            );
            builder.switch_to_block(compare_block);
        }
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            nat.value,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction.tag.into(), induction.payload.into()],
            step_block,
            &[],
        );

        builder.switch_to_block(step_block);
        let successor_value = if break_decrement {
            predecessor_value
        } else {
            builder.ins().iadd_imm(predecessor_value, 1)
        };
        let observed_predecessor = if expose_raw_predecessor {
            nat.value
        } else {
            predecessor_value
        };
        let predecessor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: observed_predecessor,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(observed_predecessor))
        };
        let retained = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: successor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(successor_value))
        };
        let frame_env =
            match self.materialize_eliminator_frame_env(builder, eliminator, &retained)? {
                Ok(env) => env,
                Err(trap) => return Ok(Lowered::Trap(trap)),
            };
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
        let mut suc_env = Vec::new();
        if computational {
            suc_env.push(induction);
        }
        suc_env.push(predecessor);
        suc_env.extend(frame_env);
        let suc_lowered = if remaining.is_empty() {
            self.lower_expr(builder, suc_body, &suc_env)?
        } else {
            self.lower_computational_producer_expr(builder, suc_body, &suc_env, remaining)?
        };
        let (next, next_kind) = self.merge_scalar_branch(builder, suc_lowered, "BoundedNat")?;
        if next_kind != result_kind {
            return Err(unsupported(
                "BoundedNat",
                "recursive Suc result disagrees with the Zero result kind",
            ));
        }
        if break_decrement {
            let fuel = builder.block_params(loop_block)[3];
            let next_fuel = builder.ins().iadd_imm(fuel, 1);
            builder.ins().jump(
                loop_block,
                &[
                    successor_value.into(),
                    next.tag.into(),
                    next.payload.into(),
                    next_fuel.into(),
                ],
            );
        } else {
            builder.ins().jump(
                loop_block,
                &[successor_value.into(), next.tag.into(), next.payload.into()],
            );
        }

        builder.switch_to_block(done_block);
        Ok(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        ))
    }

    fn materialize_eliminator_frame_env(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        eliminator: EliminatorFrame<'_>,
        retained_scrutinee: &Lowered,
    ) -> Result<Result<Vec<Lowered>, RuntimeTrap>, CraneliftBackendError> {
        let (env, retained_index, deferred, construct) = match eliminator {
            EliminatorFrame::Computational(frame) => (
                frame.env,
                frame.retained_scrutinee_index,
                frame.deferred_constructor_case,
                "ComputationalMatch",
            ),
            EliminatorFrame::Ordinary(frame) => (
                frame.env,
                frame.retained_scrutinee_index,
                frame.deferred_constructor_case,
                "Match",
            ),
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations do not materialize environments")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns do not materialize environments")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors do not materialize environments")
            }
        };
        let Some(deferred) = deferred else {
            let mut env = env.to_vec();
            if let Some(index) = retained_index {
                if index > env.len() {
                    return Err(unsupported(
                        construct,
                        "retained scrutinee index exceeds the frame environment",
                    ));
                }
                env.insert(index, retained_scrutinee.clone());
            }
            return Ok(Ok(env));
        };
        if deferred.lowered_prefix.len() != deferred.selected_field {
            return Err(unsupported(
                "Construct",
                "selected constructor field prefix does not match its binder index",
            ));
        }

        let mut constructor_args = deferred.lowered_prefix.to_vec();
        constructor_args.push(retained_scrutinee.clone());
        for field in deferred.trailing_fields {
            let lowered = self.lower_expr(builder, field, deferred.producer_env)?;
            if let Lowered::Trap(trap) = lowered {
                return Ok(Err(trap));
            }
            constructor_args.push(lowered);
        }
        let outer_scrutinee = Lowered::Constructor {
            constructor: deferred.constructor.to_string(),
            args: constructor_args.clone(),
        };
        let outer_tail = match self.materialize_eliminator_frame_env(
            builder,
            deferred.outer_eliminator,
            &outer_scrutinee,
        )? {
            Ok(env) => env,
            Err(trap) => return Ok(Err(trap)),
        };

        match deferred.outer_eliminator {
            EliminatorFrame::Computational(frame) => {
                let case = match frame
                    .cases
                    .iter()
                    .find(|case| case.constructor == deferred.constructor)
                {
                    Some(case) => case,
                    None => return Ok(Err(frame.default.clone())),
                };
                if case.argument_binders != constructor_args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} expects {} constructor arguments but value has {}",
                            case.constructor,
                            case.argument_binders,
                            constructor_args.len()
                        ),
                    ));
                }
                let mut seen = BTreeSet::new();
                for position in case.recursive_positions.iter().copied() {
                    if !seen.insert(position) || position >= constructor_args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "case {} has malformed recursive position {position}",
                                case.constructor
                            ),
                        ));
                    }
                }
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                let ih_slots =
                    self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                let producer_origin = self.mint_recursor_producer_origin();
                let producer_scope_instance = deferred
                    .selected_active
                    .selected_scope
                    .map(|scope| scope.scope_instance)
                    .ok_or_else(|| {
                        unsupported(
                            "NativeControlCellV1",
                            "deferred recursive producer has no owning selected-scope cell",
                        )
                    })?;
                #[cfg(test)]
                px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                    path: Px8jProducerPath::DeferredConstructor,
                    origin: producer_origin,
                    cursor: deferred.selected_active.cursor,
                    siblings: case.recursive_positions.len(),
                    parent_scope: deferred
                        .selected_active
                        .selected_scope
                        .map(|scope| scope.scope_origin),
                });
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let induction_hypothesis = self.make_computational_recursor(
                        builder,
                        constructor_args[position].clone(),
                        frame.cases.to_vec(),
                        frame.default.clone(),
                        outer_tail.clone(),
                        frame.provenance,
                        frame.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                            origin_scope: producer_scope_instance,
                        },
                        deferred.selected_active.activation,
                        deferred.selected_active.cursor,
                        deferred.selected_active.cursor_instance,
                        Some(&deferred.selected_active),
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(
                        Px8jProducerPath::DeferredConstructor,
                        &induction_hypothesis,
                    );
                    induction_hypotheses.push(induction_hypothesis);
                }
                induction_hypotheses.extend(constructor_args);
                induction_hypotheses.extend(outer_tail);
                Ok(Ok(induction_hypotheses))
            }
            EliminatorFrame::Ordinary(frame) => {
                let case = match select_ordinary_case(frame, deferred.constructor) {
                    Ok(case) => case,
                    Err(trap) => return Ok(Err(trap)),
                };
                if case.binders != constructor_args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            constructor_args.len()
                        ),
                    ));
                }
                constructor_args.extend(outer_tail);
                Ok(Ok(constructor_args))
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations cannot be deferred constructor frames")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns cannot be deferred constructor frames")
            }
            EliminatorFrame::Active(_) => {
                unreachable!("active continuation cursors cannot be deferred constructor frames")
            }
        }
    }

    fn lower_arm_partition_work_item(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut item: ArmPartitionWorkItem,
        frame_pointer: cranelift_codegen::ir::Value,
    ) -> Result<(Option<RuntimeTrap>, ResultDecoder), CraneliftBackendError> {
        self.partition_metrics
            .record_helper_frame_loads(item.field_types.len());
        let loaded = item
            .field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeFunctionPartition",
                            "private producer frame load offset overflowed",
                        )
                    })?;
                Ok(builder
                    .ins()
                    .load(*field_type, MemFlags::trusted(), frame_pointer, byte_offset))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let loaded = expand_partition_frame_values(&loaded, &item.field_map)?;
        let mut loaded = loaded.into_iter();
        for value in &mut item.env {
            rebuild_partition_lowered(value, &mut loaded, &mut self.native_int_tags)?;
        }
        rebuild_partition_eliminators(
            &mut item.eliminators,
            &mut loaded,
            &mut self.native_int_tags,
        )?;
        if let Some(producer_kont) = &mut item.producer_kont {
            producer_kont.capture_pointer = loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "outlined scalar arm lost its producer-continuation cell pointer",
                )
            })?;
        }
        if loaded.next().is_some() {
            return Err(unsupported(
                "NativeFunctionPartition",
                "private producer frame has trailing fields",
            ));
        }
        let checked_join = self
            .native_join_plan
            .as_ref()
            .and_then(|plan| {
                plan.sites.iter().find(|site| {
                    site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                        && site.checked_occurrence_path == [0]
                        && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
                })
            })
            .map(PartitionCheckedJoinIdentity::from)
            .ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "outlined scalar arm has no distinguished checked terminal",
                )
            })?;
        let terminal = if let Some(producer_kont) = item.producer_kont {
            producer_kont
        } else {
            self.push_done_producer_kont(
                builder,
                checked_join.clone(),
                item.return_authority.descriptor.required_kind,
                PartitionProducerKontTerminalIdentity::ScalarArmReturn {
                    edge_index: item.return_authority.descriptor.edge_index,
                },
            )?
        };
        let return_contract = PartitionStateReturnContract::producer_terminal(
            checked_join.clone(),
            item.return_authority.descriptor.required_kind,
        );
        self.active_partition_producer_kont = Some(terminal);
        let producer_kont = self.push_owned_producer_kont(
            builder,
            item.eliminators.clone(),
            terminal,
            checked_join,
            item.return_authority.descriptor.required_kind,
        )?;
        self.active_partition_producer_kont = Some(producer_kont);
        let invocation_return = [EliminatorFrame::InvocationReturn];
        self.active_partition_return_kind = Some(item.return_authority.descriptor.required_kind);
        self.active_partition_return_contract = Some(return_contract);
        let lowered = self.lower_computational_producer_expr(
            builder,
            &item.body,
            &item.env,
            &invocation_return,
        );
        self.active_partition_return_contract = None;
        self.active_partition_return_kind = None;
        self.active_partition_producer_kont = None;
        let lowered = lowered?;
        let lowered = match lowered {
            Lowered::Trap(_) | Lowered::ProcessExitStatus { .. } => lowered,
            value => self.call_partition_producer_kont(builder, producer_kont, value)?,
        };
        match lowered {
            Lowered::Trap(trap) => {
                self.consume_partition_branch_return(
                    item.return_authority,
                    item.helper_index,
                    ScalarMergeKind::ExitCode,
                )?;
                let payload = builder.ins().iconst(types::I64, -4);
                builder.ins().return_(&[payload]);
                Ok((Some(trap), ResultDecoder::ProcessStatus))
            }
            value => {
                let required_kind = item.return_authority.descriptor.required_kind;
                let (pair, actual_kind) = self.merge_planned_scalar_branch(
                    builder,
                    value,
                    required_kind,
                    "NativeFunctionPartition",
                )?;
                self.consume_partition_branch_return(
                    item.return_authority,
                    item.helper_index,
                    actual_kind,
                )?;
                self.emit_partition_pair_return(builder, pair);
                Ok((None, ResultDecoder::ProcessStatus))
            }
        }
    }

    fn lower_source_arm_partition_work_item(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut item: SourceArmPartitionWorkItem,
        frame_pointer: cranelift_codegen::ir::Value,
    ) -> Result<(Option<RuntimeTrap>, ResultDecoder), CraneliftBackendError> {
        self.partition_metrics
            .record_helper_frame_loads(item.field_types.len());
        let loaded = item
            .field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeFunctionPartition",
                            "private source-arm frame load offset overflowed",
                        )
                    })?;
                Ok(builder
                    .ins()
                    .load(*field_type, MemFlags::trusted(), frame_pointer, byte_offset))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let loaded = expand_partition_frame_values(&loaded, &item.field_map)?;
        let mut loaded = loaded.into_iter();
        for value in &mut item.env {
            rebuild_partition_lowered(value, &mut loaded, &mut self.native_int_tags)?;
        }
        if item.source_head.is_some() {
            item.source_capture_pointer = Some(loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "source Eval state lost its continuation capture pointer",
                )
            })?);
        }
        if let Some(producer_kont) = &mut item.producer_kont {
            producer_kont.capture_pointer = loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "source Eval state lost its producer-continuation cell pointer",
                )
            })?;
        }
        item.selected_activation_instance =
            ActivationInstanceRef(loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeControlCellV1",
                    "source Eval state lost its activation-instance reference",
                )
            })?);
        item.selected_cursor_instance = ControlCursorRef(loaded.next().ok_or_else(|| {
            unsupported(
                "NativeControlCellV1",
                "source Eval state lost its cursor-instance reference",
            )
        })?);
        rebuild_partition_scope(
            &mut item.selected_scope,
            &mut loaded,
            &mut self.native_int_tags,
        )?;
        rebuild_partition_eliminators(
            &mut item.selected_pending,
            &mut loaded,
            &mut self.native_int_tags,
        )?;
        if item.cleanup_head.is_some() {
            item.cleanup_capture_pointer = Some(loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeCleanupStepV1",
                    "source-arm cleanup capture pointer is missing",
                )
            })?);
        }
        if loaded.next().is_some() {
            return Err(unsupported(
                "NativeFunctionPartition",
                "private source-arm frame has trailing fields",
            ));
        }

        let mut body = item.body;
        self.declaration_stack = item.declaration_stack;
        self.active_recursive_invocations = item.active_recursive_invocations;
        self.pending_computational_ih_call = item.pending_computational_ih_call;
        let terminal = SourceContinuationTerminal::ReturnFromPartition {
            expected_outer: item.terminal_outer,
        };
        let continuation = match (item.source_head, item.source_capture_pointer) {
            (Some(_), Some(_)) | (None, None) => SourceContinuation::Terminal(terminal),
            _ => {
                return Err(unsupported(
                    "NativeSourceContinuationStepV1",
                    "source Eval state has an incomplete continuation cursor",
                ));
            }
        };
        let selected_lineage = Vec::new();
        let mut control = SourceControl {
            continuation,
            partition_cursor: item.source_head.zip(item.source_capture_pointer).map(
                |(node, capture_pointer)| PartitionSourceCursor {
                    node,
                    capture_pointer,
                },
            ),
            producer_kont: item.producer_kont,
            selected: SourceSelectedContinuation {
                activation: item.selected_activation,
                activation_instance: item.selected_activation_instance,
                cursor: item.selected_cursor,
                cursor_instance: item.selected_cursor_instance,
                parent: None,
                pending: borrow_partition_eliminators(&item.selected_pending),
                selected_ancestry: item.selected_ancestry,
                selected_scope: item.selected_scope,
            },
            selected_lineage,
            terminal_outer: item.terminal_outer,
        };
        if item.consume_checked_entry_marker {
            match body {
                RuntimeExpr::CheckedRecursiveInvocation {
                    call_template_id,
                    body: inner,
                    ..
                } => {
                    let instance =
                        self.enter_checked_recursive_invocation(call_template_id, &inner)?;
                    control.continuation = SourceContinuation::CheckedRecursiveInvocationReturn {
                        instance,
                        next: Box::new(control.continuation),
                    };
                    self.push_partition_source_cursor(builder, &mut control)?;
                    body = *inner;
                }
                RuntimeExpr::CheckedComputationalIHInvocation {
                    call_template_id,
                    body: inner,
                    ..
                } => {
                    self.enter_checked_computational_ih_invocation(call_template_id)?;
                    control.continuation =
                        SourceContinuation::CheckedComputationalIHInvocationReturn {
                            call_template_id,
                            next: Box::new(control.continuation),
                        };
                    self.push_partition_source_cursor(builder, &mut control)?;
                    body = *inner;
                }
                _ => {
                    return Err(unsupported(
                        "NativeSourceContinuationStepV1",
                        "checked-template Eval state lost its exact entry marker",
                    ));
                }
            }
        }
        let required_kind = item.return_contract.required_kind;
        self.active_partition_producer_kont = item.producer_kont;
        self.active_partition_return_kind = Some(required_kind);
        self.active_partition_return_contract = Some(item.return_contract.clone());
        let lowered = self.lower_source_machine_with_continuation(builder, body, item.env, control);
        self.active_partition_return_contract = None;
        self.active_partition_return_kind = None;
        self.active_partition_producer_kont = None;
        self.pending_computational_ih_call = None;
        let mut lowered = lowered?;
        if !matches!(lowered, Lowered::Trap(_)) {
            if let Some(cleanup_head) = item.cleanup_head {
                let capture_pointer = item.cleanup_capture_pointer.ok_or_else(|| {
                    unsupported(
                        "NativeCleanupStepV1",
                        "source arm lost its synchronous cleanup capture chain",
                    )
                })?;
                lowered = self.call_partition_cleanup_step(
                    builder,
                    None,
                    cleanup_head,
                    capture_pointer,
                    lowered,
                    item.return_contract.checked_join.clone(),
                    required_kind,
                    &item.ledger_baseline,
                )?;
            }
        }
        match lowered {
            Lowered::Trap(trap) => {
                let payload = builder.ins().iconst(types::I64, -4);
                builder.ins().return_(&[payload]);
                Ok((Some(trap), ResultDecoder::ProcessStatus))
            }
            value => {
                let (pair, _actual_kind) = self.merge_planned_scalar_branch(
                    builder,
                    value,
                    required_kind,
                    "NativeFunctionPartition",
                )?;
                self.emit_partition_pair_return(builder, pair);
                Ok((None, ResultDecoder::ProcessStatus))
            }
        }
    }

    fn lower_source_kont_partition_work_item(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut item: SourceKontPartitionWorkItem,
        frame_pointer: cranelift_codegen::ir::Value,
    ) -> Result<(Option<RuntimeTrap>, ResultDecoder), CraneliftBackendError> {
        self.partition_metrics
            .record_helper_frame_loads(item.field_types.len());
        let loaded = item
            .field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeSourceContinuationStepV1",
                            "private Kont input-cell load offset overflowed",
                        )
                    })?;
                Ok(builder
                    .ins()
                    .load(*field_type, MemFlags::trusted(), frame_pointer, byte_offset))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let loaded = expand_partition_frame_values(&loaded, &item.field_map)?;
        let mut loaded = loaded.into_iter();
        rebuild_partition_lowered(&mut item.input, &mut loaded, &mut self.native_int_tags)?;
        item.capture_pointer = loaded.next().ok_or_else(|| {
            unsupported(
                "NativeSourceContinuationStepV1",
                "Kont input cell lost its immediate-capture pointer",
            )
        })?;
        if let Some(producer_kont) = &mut item.producer_kont {
            producer_kont.capture_pointer = loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "source Kont state lost its producer-continuation cell pointer",
                )
            })?;
        }
        item.selected_activation_instance =
            ActivationInstanceRef(loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeControlCellV1",
                    "source Kont state lost its activation-instance reference",
                )
            })?);
        item.selected_cursor_instance = ControlCursorRef(loaded.next().ok_or_else(|| {
            unsupported(
                "NativeControlCellV1",
                "source Kont state lost its cursor-instance reference",
            )
        })?);
        rebuild_partition_eliminators(
            &mut item.selected_pending,
            &mut loaded,
            &mut self.native_int_tags,
        )?;
        rebuild_partition_scope(
            &mut item.selected_scope,
            &mut loaded,
            &mut self.native_int_tags,
        )?;
        if loaded.next().is_some() {
            return Err(unsupported(
                "NativeSourceContinuationStepV1",
                "private Kont input cell has trailing fields",
            ));
        }

        let mut definition = self.partition_source_nodes.definition(item.node)?;
        let mut captures = definition
            .capture_field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let offset = index
                    .checked_add(1)
                    .and_then(|field| field.checked_mul(PARTITION_FRAME_FIELD_BYTES as usize))
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeSourceContinuationStepV1",
                            "Kont immediate-capture load offset overflowed",
                        )
                    })?;
                Ok(builder.ins().load(
                    *field_type,
                    MemFlags::trusted(),
                    item.capture_pointer,
                    offset,
                ))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?
            .into_iter();
        rebuild_partition_prefix(
            &mut definition.current,
            &mut captures,
            &mut self.native_int_tags,
        )?;
        let restored_parent = if let SourcePrefixTemplate::ReturnFromSelectedCase {
            parent_capture: Some(parent),
            ..
        } = &definition.current
        {
            Some(parent.clone())
        } else {
            None
        };
        let selected_lineage = restored_parent
            .as_ref()
            .map(|parent| {
                vec![SourceSelectedContinuation {
                    activation: parent.activation,
                    activation_instance: parent.activation_instance,
                    cursor: parent.cursor,
                    cursor_instance: parent.cursor_instance,
                    parent: None,
                    pending: borrow_partition_eliminators(&parent.pending),
                    selected_ancestry: parent.selected_ancestry.clone(),
                    selected_scope: parent.selected_scope.clone(),
                }]
            })
            .unwrap_or_default();
        if captures.next().is_some() {
            return Err(unsupported(
                "NativeSourceContinuationStepV1",
                "Kont immediate-capture cell has trailing fields",
            ));
        }
        let terminal = SourceContinuationTerminal::ReturnFromPartition {
            expected_outer: item.terminal_outer,
        };
        let successor = SourceContinuation::Terminal(terminal);
        let continuation = instantiate_partition_source_node(definition.current, successor)?;
        let control = SourceControl {
            continuation,
            partition_cursor: Some(PartitionSourceCursor {
                node: item.node,
                capture_pointer: item.capture_pointer,
            }),
            producer_kont: item.producer_kont,
            selected: SourceSelectedContinuation {
                activation: item.selected_activation,
                activation_instance: item.selected_activation_instance,
                cursor: item.selected_cursor,
                cursor_instance: item.selected_cursor_instance,
                parent: None,
                pending: borrow_partition_eliminators(&item.selected_pending),
                selected_ancestry: item.selected_ancestry,
                selected_scope: item.selected_scope,
            },
            selected_lineage,
            terminal_outer: item.terminal_outer,
        };
        self.declaration_stack = item.declaration_stack;
        self.active_recursive_invocations = item.active_recursive_invocations;
        self.pending_computational_ih_call = item.pending_computational_ih_call;
        self.active_partition_producer_kont = item.producer_kont;
        self.active_partition_return_kind = Some(item.return_contract.required_kind);
        self.active_partition_return_contract = Some(item.return_contract.clone());
        let lowered = self.lower_source_machine_state_inner(
            builder,
            SourceMachineState::Value {
                value: item.input,
                control,
            },
        );
        self.active_partition_return_contract = None;
        self.active_partition_return_kind = None;
        self.active_partition_producer_kont = None;
        self.pending_computational_ih_call = None;
        match lowered? {
            Lowered::Trap(trap) => {
                let payload = builder.ins().iconst(types::I64, -4);
                builder.ins().return_(&[payload]);
                Ok((Some(trap), ResultDecoder::ProcessStatus))
            }
            value => {
                let (pair, _) = self.merge_planned_scalar_branch(
                    builder,
                    value,
                    item.return_contract.required_kind,
                    "NativeSourceContinuationStepV1",
                )?;
                self.emit_partition_pair_return(builder, pair);
                Ok((None, ResultDecoder::ProcessStatus))
            }
        }
    }

    fn call_partition_source_eval<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        body: RuntimeExpr,
        env: Vec<Lowered>,
        mut control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let popped =
            self.reserve_partition_scope_exits_from_tail(builder, &mut control.continuation)?;
        self.install_partition_exit_scope_chain(
            builder,
            &mut control.selected,
            &control.selected_lineage,
            &mut control.producer_kont,
            &popped,
        )?;
        if control.selected.parent.is_some()
            || !env.iter().all(partition_lowered_is_admissible)
            || !partition_scope_is_admissible(&control.selected.selected_scope)
        {
            return Err(unsupported(
                "NativeSourceContinuationStepV1",
                "planning completeness: Eval state has no exact closed input schema",
            ));
        }
        let return_contract = self
            .active_partition_return_contract
            .clone()
            .ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "Eval transfer has no checked scalar return contract",
                )
            })?;
        let selected_pending =
            own_partition_eliminators(&control.selected.pending).ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "planning completeness: Eval selected pending control has no exact schema",
                )
            })?;
        let selected_lineage = own_partition_selected_lineage(&control.selected_lineage)
            .ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "planning completeness: Eval selected lineage has no exact schema",
                )
            })?;
        let mut fields = Vec::new();
        for value in &env {
            append_partition_lowered_values(self, builder, value, &mut fields)?;
        }
        if let Some(cursor) = control.partition_cursor {
            fields.push(cursor.capture_pointer);
        }
        if let Some(producer_kont) = control.producer_kont {
            fields.push(producer_kont.capture_pointer);
        }
        fields.push(control.selected.activation_instance.0);
        fields.push(control.selected.cursor_instance.0);
        append_partition_scope_values(
            self,
            builder,
            &control.selected.selected_scope,
            &mut fields,
        )?;
        append_partition_eliminator_values(self, builder, &selected_pending, &mut fields)?;
        let (frame_values, field_types, field_map) = partition_frame_layout(builder, &fields);
        self.partition_metrics.record_call_frame(frame_values.len());
        let key = PartitionSemanticStateKey::SourceArm(PartitionSourceArmKey::new(
            return_contract.checked_join.clone(),
            return_contract.required_kind,
            false,
            self.pending_computational_ih_call,
            &body,
            &env,
            &self.declaration_stack,
            &self.active_recursive_invocations,
            control.partition_cursor.map(|cursor| cursor.node),
            control.producer_kont.map(|cursor| cursor.site_id),
            control.selected.activation,
            control.selected.cursor,
            &control.selected.selected_ancestry,
            &selected_pending,
            &control.selected.selected_scope,
            &selected_lineage,
            control.terminal_outer,
            None,
            field_types.clone(),
            field_map.clone(),
        ));
        let expected_contract = key.return_contract();
        let existing = self
            .partition_continuations
            .lookup(&key, PartitionAggregateBudget::PRODUCTION)?;
        let (state_id, state, newly_reserved) = if let Some((state_id, state)) = existing {
            (state_id, state, false)
        } else {
            let helper_index = self.partition_next_helper;
            let function = *self.partition_helper_ids.get(helper_index).ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "Eval state exhausted its predeclared helper pool",
                )
            })?;
            self.partition_next_helper += 1;
            let (state_id, state) = self.partition_continuations.reserve(
                key,
                function,
                helper_index,
                PartitionAggregateBudget::PRODUCTION,
            )?;
            (state_id, state, true)
        };
        self.partition_continuations
            .validate_call_contract(state_id, &expected_contract)?;
        if !newly_reserved {
            self.consume_reused_partition_dynamic_splice_edges(&env)?;
        }
        let (_, function_ref) = self.partition_helper_ref(
            builder,
            state.helper_index,
            "interned Eval state lost its helper identity",
        )?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(frame_values.len())?,
            3,
        ));
        for (index, value) in frame_values.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeSourceContinuationStepV1",
                        "Eval input-cell store offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("Eval transfer owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES,
            3,
        ));
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().stack_store(zero, tag_output, 0);
        let tag_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
        let call = builder
            .ins()
            .call(function_ref, &[invocation, frame_pointer, tag_pointer]);
        let tag = builder.ins().stack_load(types::I64, tag_output, 0);
        let payload = builder.inst_results(call)[0];
        if newly_reserved {
            self.partition_queue.push_back(PartitionWorkItem::SourceArm(
                SourceArmPartitionWorkItem {
                    state_id,
                    function: state.function,
                    field_types,
                    field_map,
                    body,
                    consume_checked_entry_marker: false,
                    pending_computational_ih_call: self.pending_computational_ih_call,
                    env,
                    declaration_stack: self.declaration_stack.clone(),
                    active_recursive_invocations: self.active_recursive_invocations.clone(),
                    source_head: control.partition_cursor.map(|cursor| cursor.node),
                    source_capture_pointer: control
                        .partition_cursor
                        .map(|cursor| cursor.capture_pointer),
                    producer_kont: control.producer_kont,
                    selected_activation: control.selected.activation,
                    selected_activation_instance: control.selected.activation_instance,
                    selected_cursor: control.selected.cursor,
                    selected_cursor_instance: control.selected.cursor_instance,
                    selected_ancestry: control.selected.selected_ancestry,
                    selected_pending,
                    selected_scope: control.selected.selected_scope,
                    selected_lineage,
                    terminal_outer: control.terminal_outer,
                    cleanup_head: None,
                    cleanup_capture_pointer: None,
                    ledger_baseline: self.partition_ledger_baseline(),
                    return_contract: expected_contract.clone(),
                },
            ));
        }
        Ok(self.lowered_from_scalar_pair(
            expected_contract.required_kind,
            NativeScalarPairV1 { tag, payload },
        ))
    }

    fn call_partition_source_kont<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        head: PartitionSourceCursor,
        input: Lowered,
        mut control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let popped =
            self.reserve_partition_scope_exits_from_tail(builder, &mut control.continuation)?;
        self.install_partition_exit_scope_chain(
            builder,
            &mut control.selected,
            &control.selected_lineage,
            &mut control.producer_kont,
            &popped,
        )?;
        if !partition_lowered_is_admissible(&input) {
            return Err(unsupported(
                "NativeSourceContinuationStepV1",
                "planning completeness: Kont input has no exact closed schema",
            ));
        }
        let return_contract = self
            .active_partition_return_contract
            .clone()
            .ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "Kont transfer has no checked scalar return contract",
                )
            })?;
        let selected_pending =
            own_partition_eliminators(&control.selected.pending).ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "planning completeness: selected pending control has no exact schema",
                )
            })?;
        let selected_lineage = own_partition_selected_lineage(&control.selected_lineage)
            .ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "planning completeness: selected lineage has no exact schema",
                )
            })?;
        let mut fields = Vec::new();
        append_partition_lowered_values(self, builder, &input, &mut fields)?;
        fields.push(head.capture_pointer);
        if let Some(producer_kont) = control.producer_kont {
            fields.push(producer_kont.capture_pointer);
        }
        fields.push(control.selected.activation_instance.0);
        fields.push(control.selected.cursor_instance.0);
        append_partition_eliminator_values(self, builder, &selected_pending, &mut fields)?;
        append_partition_scope_values(
            self,
            builder,
            &control.selected.selected_scope,
            &mut fields,
        )?;
        let (frame_values, field_types, field_map) = partition_frame_layout(builder, &fields);
        self.partition_metrics.record_call_frame(frame_values.len());
        let key = PartitionSemanticStateKey::SourceKont(PartitionSourceKontKey::new(
            return_contract.checked_join.clone(),
            return_contract.required_kind,
            head.node,
            control.producer_kont.map(|cursor| cursor.site_id),
            self.pending_computational_ih_call,
            &input,
            &self.declaration_stack,
            &self.active_recursive_invocations,
            control.selected.activation,
            control.selected.cursor,
            &control.selected.selected_ancestry,
            &selected_pending,
            &control.selected.selected_scope,
            &selected_lineage,
            control.terminal_outer,
            field_types.clone(),
            field_map.clone(),
        ));
        let expected_contract = key.return_contract();
        let existing = self
            .partition_continuations
            .lookup(&key, PartitionAggregateBudget::PRODUCTION)?;
        let (state_id, state, newly_reserved) = if let Some((state_id, state)) = existing {
            (state_id, state, false)
        } else {
            let helper_index = self.partition_next_helper;
            let function = *self.partition_helper_ids.get(helper_index).ok_or_else(|| {
                unsupported(
                    "NativeSourceContinuationStepV1",
                    "Kont state exhausted its predeclared helper pool",
                )
            })?;
            self.partition_next_helper += 1;
            let (state_id, state) = self.partition_continuations.reserve(
                key,
                function,
                helper_index,
                PartitionAggregateBudget::PRODUCTION,
            )?;
            (state_id, state, true)
        };
        self.partition_continuations
            .validate_call_contract(state_id, &expected_contract)?;
        let (_, function_ref) = self.partition_helper_ref(
            builder,
            state.helper_index,
            "interned Kont state lost its helper identity",
        )?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            partition_frame_size(frame_values.len())?,
            3,
        ));
        for (index, value) in frame_values.iter().copied().enumerate() {
            let offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeSourceContinuationStepV1",
                        "Kont input-cell store offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("Kont transfer owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES,
            3,
        ));
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().stack_store(zero, tag_output, 0);
        let tag_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
        let call = builder
            .ins()
            .call(function_ref, &[invocation, frame_pointer, tag_pointer]);
        let tag = builder.ins().stack_load(types::I64, tag_output, 0);
        let payload = builder.inst_results(call)[0];
        if newly_reserved {
            self.partition_queue
                .push_back(PartitionWorkItem::SourceKont(SourceKontPartitionWorkItem {
                    state_id,
                    function: state.function,
                    helper_index: state.helper_index,
                    field_types,
                    field_map,
                    input,
                    node: head.node,
                    capture_pointer: head.capture_pointer,
                    producer_kont: control.producer_kont,
                    pending_computational_ih_call: self.pending_computational_ih_call,
                    declaration_stack: self.declaration_stack.clone(),
                    active_recursive_invocations: self.active_recursive_invocations.clone(),
                    selected_activation: control.selected.activation,
                    selected_activation_instance: control.selected.activation_instance,
                    selected_cursor: control.selected.cursor,
                    selected_cursor_instance: control.selected.cursor_instance,
                    selected_ancestry: control.selected.selected_ancestry,
                    selected_pending,
                    selected_scope: control.selected.selected_scope,
                    selected_lineage,
                    terminal_outer: control.terminal_outer,
                    ledger_baseline: self.partition_ledger_baseline(),
                    return_contract: expected_contract.clone(),
                }));
        }
        Ok(self.lowered_from_scalar_pair(
            expected_contract.required_kind,
            NativeScalarPairV1 { tag, payload },
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn call_partition_cleanup_step(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        from: Option<PartitionCleanupSuffixId>,
        suffix: PartitionCleanupSuffixId,
        capture_pointer: cranelift_codegen::ir::Value,
        input: Lowered,
        checked_join: PartitionCheckedJoinIdentity,
        required_kind: ScalarMergeKind,
        ledger_baseline: &PartitionLedgerBaseline,
    ) -> Result<Lowered, CraneliftBackendError> {
        if !partition_lowered_is_admissible(&input) {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "planning-completeness gap: cleanup input has no exact closed schema",
            ));
        }
        let mut fields = Vec::new();
        append_partition_lowered_values(self, builder, &input, &mut fields)?;
        fields.push(capture_pointer);
        let (frame_values, field_types, field_map) = partition_frame_layout(builder, &fields);
        let key = PartitionSemanticStateKey::CleanupStep(PartitionCleanupStepKey::new(
            checked_join.clone(),
            required_kind,
            suffix,
            &input,
            field_types.clone(),
            field_map.clone(),
        ));
        let return_contract = key.return_contract();
        let existing = self
            .partition_continuations
            .lookup(&key, PartitionAggregateBudget::PRODUCTION)?;
        let (state_id, state, newly_reserved) = if let Some((state_id, state)) = existing {
            (state_id, state, false)
        } else {
            let helper_index = self.partition_next_helper;
            let function = *self.partition_helper_ids.get(helper_index).ok_or_else(|| {
                unsupported(
                    "NativeCleanupStepV1",
                    "cleanup step exhausted its predeclared helper pool",
                )
            })?;
            self.partition_next_helper =
                self.partition_next_helper.checked_add(1).ok_or_else(|| {
                    unsupported("NativeCleanupStepV1", "cleanup helper identity exhausted")
                })?;
            let (state_id, state) = self.partition_continuations.reserve(
                key,
                function,
                helper_index,
                PartitionAggregateBudget::PRODUCTION,
            )?;
            self.partition_metrics.cleanup_states =
                self.partition_metrics.cleanup_states.saturating_add(1);
            (state_id, state, true)
        };
        self.partition_continuations
            .validate_call_contract(state_id, &return_contract)?;
        if !newly_reserved {
            self.consume_reused_partition_dynamic_splice_edges(std::slice::from_ref(&input))?;
        }
        let (_, function_ref) = self.partition_helper_ref(
            builder,
            state.helper_index,
            "cleanup state lost its helper identity",
        )?;
        let frame_size = partition_frame_size(frame_values.len())?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            frame_size,
            3,
        ));
        self.partition_metrics
            .record_cleanup_call_frame(frame_values.len());
        for (index, value) in frame_values.iter().copied().enumerate() {
            let byte_offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeCleanupStepV1",
                        "cleanup input-cell offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, byte_offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("cleanup partition owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES,
            3,
        ));
        let zero_tag = builder.ins().iconst(types::I64, 0);
        builder.ins().stack_store(zero_tag, tag_output, 0);
        let tag_output_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
        let authority =
            self.partition_cleanup_transitions
                .mint(from, suffix, state.helper_index)?;
        let call = builder.ins().call(
            function_ref,
            &[invocation, frame_pointer, tag_output_pointer],
        );
        self.partition_cleanup_transitions
            .consume(authority, from, suffix, state.helper_index)?;
        let pair = NativeScalarPairV1 {
            tag: builder.ins().stack_load(types::I64, tag_output, 0),
            payload: builder.inst_results(call)[0],
        };
        if newly_reserved {
            self.partition_queue
                .push_back(PartitionWorkItem::CleanupStep(
                    CleanupStepPartitionWorkItem {
                        state_id,
                        function: state.function,
                        helper_index: state.helper_index,
                        field_types,
                        field_map,
                        input,
                        suffix,
                        checked_join,
                        required_kind,
                        ledger_baseline: ledger_baseline.clone(),
                    },
                ));
        }
        Ok(self.lowered_from_scalar_pair(required_kind, pair))
    }

    fn lower_cleanup_step_partition_work_item(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut item: CleanupStepPartitionWorkItem,
        frame_pointer: cranelift_codegen::ir::Value,
    ) -> Result<(Option<RuntimeTrap>, ResultDecoder), CraneliftBackendError> {
        self.partition_metrics
            .record_cleanup_helper_frame_loads(item.field_types.len());
        let loaded = item
            .field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeCleanupStepV1",
                            "cleanup input-cell load offset overflowed",
                        )
                    })?;
                Ok(builder
                    .ins()
                    .load(*field_type, MemFlags::trusted(), frame_pointer, byte_offset))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let loaded = expand_partition_frame_values(&loaded, &item.field_map)?;
        let mut loaded = loaded.into_iter();
        rebuild_partition_lowered(&mut item.input, &mut loaded, &mut self.native_int_tags)?;
        let capture_pointer = loaded.next().ok_or_else(|| {
            unsupported(
                "NativeCleanupStepV1",
                "cleanup input cell omitted its capture link",
            )
        })?;
        if loaded.next().is_some() {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "cleanup input cell has trailing fields",
            ));
        }

        let mut definition = self.partition_cleanup_suffixes.definition(item.suffix)?;
        let capture_values = definition
            .capture_field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeCleanupStepV1",
                            "cleanup capture-cell load offset overflowed",
                        )
                    })?;
                Ok(builder.ins().load(
                    *field_type,
                    MemFlags::trusted(),
                    capture_pointer,
                    byte_offset,
                ))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        self.partition_metrics.cleanup_frame_loads = self
            .partition_metrics
            .cleanup_frame_loads
            .saturating_add(capture_values.len());
        self.partition_metrics.frame_loads = self
            .partition_metrics
            .frame_loads
            .saturating_add(capture_values.len());
        let (next_capture_pointer, current_values) =
            capture_values.split_last().ok_or_else(|| {
                unsupported(
                    "NativeCleanupStepV1",
                    "cleanup capture cell has no parent link",
                )
            })?;
        let mut current_values = current_values.iter().copied();
        rebuild_partition_eliminators(
            std::slice::from_mut(&mut definition.current),
            &mut current_values,
            &mut self.native_int_tags,
        )?;
        if current_values.next().is_some() {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "cleanup capture cell has trailing dynamic fields",
            ));
        }
        let current = borrow_partition_eliminators(std::slice::from_ref(&definition.current));
        let one_node = [current[0], EliminatorFrame::InvocationReturn];
        self.active_partition_return_kind = Some(item.required_kind);
        let lowered = self.lower_computational_match_value_composed(builder, item.input, &one_node);
        self.active_partition_return_kind = None;
        let lowered = lowered?;
        if let Lowered::Trap(trap) = lowered {
            let payload = builder.ins().iconst(types::I64, -4);
            builder.ins().return_(&[payload]);
            return Ok((Some(trap), ResultDecoder::ProcessStatus));
        }
        let lowered = if let Some(successor) = definition.successor {
            self.call_partition_cleanup_step(
                builder,
                Some(item.suffix),
                successor,
                *next_capture_pointer,
                lowered,
                item.checked_join,
                item.required_kind,
                &item.ledger_baseline,
            )?
        } else {
            lowered
        };
        let (pair, actual_kind) = self.merge_planned_scalar_branch(
            builder,
            lowered,
            item.required_kind,
            "NativeCleanupStepV1",
        )?;
        debug_assert_eq!(actual_kind, item.required_kind);
        self.emit_partition_pair_return(builder, pair);
        let _ = item.helper_index;
        Ok((None, ResultDecoder::ProcessStatus))
    }

    fn lower_producer_kont_partition_work_item(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut item: ProducerKontPartitionWorkItem,
        frame_pointer: cranelift_codegen::ir::Value,
    ) -> Result<(Option<RuntimeTrap>, ResultDecoder), CraneliftBackendError> {
        self.declaration_stack = item.declaration_stack.clone();
        self.active_recursive_invocations = item.active_recursive_invocations.clone();
        self.partition_metrics
            .record_helper_frame_loads(item.field_types.len());
        let loaded = item
            .field_types
            .iter()
            .enumerate()
            .map(|(index, field_type)| {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeFunctionPartition",
                            "private resume frame load offset overflowed",
                        )
                    })?;
                Ok(builder
                    .ins()
                    .load(*field_type, MemFlags::trusted(), frame_pointer, byte_offset))
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let loaded = expand_partition_frame_values(&loaded, &item.field_map)?;
        let mut loaded = loaded.into_iter();
        rebuild_partition_lowered(&mut item.value, &mut loaded, &mut self.native_int_tags)?;
        let required_kind = item.return_kind;
        self.active_partition_return_kind = Some(required_kind);
        self.active_partition_return_contract =
            Some(PartitionStateReturnContract::producer_terminal(
                item.checked_join.clone(),
                required_kind,
            ));
        let action_has_capture_cell = match &item.action {
            ProducerKontAction::Done { .. } => true,
            ProducerKontAction::ApplyActiveEliminators {
                capture_field_types,
                ..
            } => !capture_field_types.is_empty() || item.site_id != usize::MAX,
            ProducerKontAction::OrientedInvocationReturn { .. }
            | ProducerKontAction::ApplyEliminators { .. }
            | ProducerKontAction::CheckedComputationalIHReturn { .. }
            | ProducerKontAction::ExitScopeStart { .. }
            | ProducerKontAction::ExitScopeComplete { .. } => true,
        };
        if action_has_capture_cell {
            item.capture_pointer = Some(loaded.next().ok_or_else(|| {
                unsupported(
                    "NativeProducerContinuationStepV1",
                    "producer continuation input lost its capture-cell pointer",
                )
            })?);
            self.active_partition_producer_kont =
                if matches!(&item.action, ProducerKontAction::ExitScopeStart { .. }) {
                    None
                } else {
                    item.successor
                };
        }
        let lowered = match &mut item.action {
            ProducerKontAction::Done { .. } => {
                if item.successor.is_some() {
                    return Err(unsupported(
                        "NativeProducerContinuationStepV1",
                        "explicit producer terminal unexpectedly has a successor",
                    ));
                }
                Ok(item.value)
            }
            ProducerKontAction::ApplyActiveEliminators {
                activation,
                activation_instance,
                cursor,
                cursor_instance,
                pending,
                selected_ancestry,
                selected_scope,
                selected_lineage,
                capture_field_types,
            } => {
                let captured = if action_has_capture_cell {
                    let capture_pointer = item.capture_pointer.expect("producer capture pointer");
                    capture_field_types
                        .iter()
                        .enumerate()
                        .map(|(index, field_type)| {
                            let offset = index
                                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                                .and_then(|offset| i32::try_from(offset).ok())
                                .ok_or_else(|| {
                                    unsupported(
                                        "NativeProducerContinuationStepV1",
                                        "selected-head capture-cell load offset overflowed",
                                    )
                                })?;
                            Ok(builder.ins().load(
                                *field_type,
                                MemFlags::trusted(),
                                capture_pointer,
                                offset,
                            ))
                        })
                        .collect::<Result<Vec<_>, CraneliftBackendError>>()?
                } else {
                    Vec::new()
                };
                let mut captured = captured.into_iter();
                let values = if action_has_capture_cell {
                    &mut captured
                } else {
                    &mut loaded
                };
                *activation_instance = ActivationInstanceRef(values.next().ok_or_else(|| {
                    unsupported(
                        "NativeControlCellV1",
                        "selected head lost its activation-instance reference",
                    )
                })?);
                *cursor_instance = ControlCursorRef(values.next().ok_or_else(|| {
                    unsupported(
                        "NativeControlCellV1",
                        "selected head lost its cursor-instance reference",
                    )
                })?);
                rebuild_partition_eliminators(pending, values, &mut self.native_int_tags)?;
                rebuild_partition_scope(selected_scope, values, &mut self.native_int_tags)?;
                if action_has_capture_cell {
                    if let Some(successor) = &mut item.successor {
                        successor.capture_pointer = captured.next().ok_or_else(|| {
                            unsupported(
                                "NativeProducerContinuationStepV1",
                                "selected head lost its successor-cell pointer",
                            )
                        })?;
                    }
                    if captured.next().is_some() {
                        return Err(unsupported(
                            "NativeProducerContinuationStepV1",
                            "selected-head capture cell has trailing fields",
                        ));
                    }
                }
                let pending = borrow_partition_eliminators(pending);
                let selected_lineage = selected_lineage
                    .iter()
                    .map(|selected| SourceSelectedContinuation {
                        activation: selected.activation,
                        activation_instance: selected.activation_instance,
                        cursor: selected.cursor,
                        cursor_instance: selected.cursor_instance,
                        parent: None,
                        pending: borrow_partition_eliminators(&selected.pending),
                        selected_ancestry: selected.selected_ancestry.clone(),
                        selected_scope: selected.selected_scope.clone(),
                    })
                    .collect::<Vec<_>>();
                let active = ActiveContinuationFrame {
                    activation: *activation,
                    activation_instance: *activation_instance,
                    cursor: *cursor,
                    cursor_instance: *cursor_instance,
                    parent: None,
                    pending: &pending,
                    selected_ancestry,
                    source_lineage: &selected_lineage,
                    source_selected_cursor: Some(*cursor),
                    selected_scope: selected_scope.as_ref(),
                };
                if let Some(scope) = selected_scope.as_ref() {
                    let selected_frame =
                        EliminatorFrame::Computational(ComputationalEliminatorFrame {
                            cases: &scope.frame.cases,
                            default: &scope.frame.default,
                            env: &scope.frame.outer_env,
                            retained_scrutinee_index: None,
                            deferred_constructor_case: None,
                            provenance: scope.frame.provenance,
                            checked_frame_id: scope.frame.checked_frame_id,
                            checked_invocation_id: scope.frame.checked_invocation_id,
                            checked_invocation_source: scope.frame.checked_invocation_source,
                            checked_invocation_depth: scope.frame.checked_invocation_depth,
                        });
                    let mut selected_then_pending = Vec::with_capacity(pending.len() + 1);
                    selected_then_pending.push(selected_frame);
                    selected_then_pending.extend(pending);
                    self.lower_computational_match_value_composed(
                        builder,
                        item.value,
                        &selected_then_pending,
                    )
                } else {
                    self.resume_active_continuation(builder, item.value, active)
                }
            }
            ProducerKontAction::ApplyEliminators {
                eliminators,
                capture_field_types,
            } => {
                let capture_pointer = item.capture_pointer.expect("producer capture pointer");
                let mut captures = capture_field_types
                    .iter()
                    .enumerate()
                    .map(|(index, field_type)| {
                        let offset = index
                            .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                            .and_then(|offset| i32::try_from(offset).ok())
                            .ok_or_else(|| {
                                unsupported(
                                    "NativeProducerContinuationStepV1",
                                    "producer capture-cell load offset overflowed",
                                )
                            })?;
                        Ok(builder.ins().load(
                            *field_type,
                            MemFlags::trusted(),
                            capture_pointer,
                            offset,
                        ))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?
                    .into_iter();
                rebuild_partition_eliminators(
                    eliminators,
                    &mut captures,
                    &mut self.native_int_tags,
                )?;
                if let Some(successor) = &mut item.successor {
                    successor.capture_pointer = captures.next().ok_or_else(|| {
                        unsupported(
                            "NativeProducerContinuationStepV1",
                            "producer continuation lost its successor-cell pointer",
                        )
                    })?;
                }
                if captures.next().is_some() {
                    return Err(unsupported(
                        "NativeProducerContinuationStepV1",
                        "producer continuation capture cell has trailing fields",
                    ));
                }
                let eliminators = borrow_partition_eliminators(eliminators);
                self.lower_computational_match_value_composed(builder, item.value, &eliminators)
            }
            ProducerKontAction::OrientedInvocationReturn {
                checked,
                capture_field_types,
            } => {
                let capture_pointer = item.capture_pointer.expect("producer capture pointer");
                let mut captures = capture_field_types
                    .iter()
                    .enumerate()
                    .map(|(index, field_type)| {
                        let offset = index
                            .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                            .and_then(|offset| i32::try_from(offset).ok())
                            .ok_or_else(|| {
                                unsupported(
                                    "NativeProducerContinuationStepV1",
                                    "oriented-return capture-cell load offset overflowed",
                                )
                            })?;
                        Ok(builder.ins().load(
                            *field_type,
                            MemFlags::trusted(),
                            capture_pointer,
                            offset,
                        ))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?
                    .into_iter();
                if let Some(successor) = &mut item.successor {
                    successor.capture_pointer = captures.next().ok_or_else(|| {
                        unsupported(
                            "NativeProducerContinuationStepV1",
                            "oriented return lost its successor-cell pointer",
                        )
                    })?;
                }
                if captures.next().is_some() {
                    return Err(unsupported(
                        "NativeProducerContinuationStepV1",
                        "oriented-return capture cell has trailing fields",
                    ));
                }
                self.enter_oriented_semantic_region(*checked);
                self.leave_oriented_semantic_region(*checked);
                Ok(item.value)
            }
            ProducerKontAction::CheckedComputationalIHReturn {
                call_template_id,
                capture_field_types,
            } => {
                let capture_pointer = item.capture_pointer.expect("producer capture pointer");
                let mut captures = capture_field_types
                    .iter()
                    .enumerate()
                    .map(|(index, field_type)| {
                        let offset = index
                            .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                            .and_then(|offset| i32::try_from(offset).ok())
                            .ok_or_else(|| {
                                unsupported(
                                    "NativeProducerContinuationStepV1",
                                    "checked-marker capture-cell load offset overflowed",
                                )
                            })?;
                        Ok(builder.ins().load(
                            *field_type,
                            MemFlags::trusted(),
                            capture_pointer,
                            offset,
                        ))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?
                    .into_iter();
                if let Some(successor) = &mut item.successor {
                    successor.capture_pointer = captures.next().ok_or_else(|| {
                        unsupported(
                            "NativeProducerContinuationStepV1",
                            "checked-marker return lost its successor-cell pointer",
                        )
                    })?;
                }
                if captures.next().is_some() {
                    return Err(unsupported(
                        "NativeProducerContinuationStepV1",
                        "checked-marker capture cell has trailing fields",
                    ));
                }
                self.pending_computational_ih_call = Some(*call_template_id);
                self.finish_checked_computational_ih_marker(builder, item.value)
            }
            ProducerKontAction::ExitScopeStart { target, obligation } => {
                let capture_pointer = item.capture_pointer.expect("producer capture pointer");
                let pointer_type = builder.func.dfg.value_type(capture_pointer);
                let fields = (0..5)
                    .map(|index| {
                        builder.ins().load(
                            pointer_type,
                            MemFlags::trusted(),
                            capture_pointer,
                            index * PARTITION_FRAME_FIELD_BYTES as i32,
                        )
                    })
                    .collect::<Vec<_>>();
                let obligation_pointer = fields[0];
                let target_pointer = fields[1];
                let scope_pointer = fields[2];
                let parent_pointer = fields[3];
                if let Some(successor) = &mut item.successor {
                    successor.capture_pointer = fields[4];
                }
                self.active_partition_producer_kont = item.successor;

                let obligation_definition = self
                    .partition_open_control_obligations
                    .definition(*obligation)?;
                if obligation_definition.target != *target {
                    return Err(unsupported(
                        "NativeExitScopeTransitionV1",
                        "exit start descriptor does not own its recursor node",
                    ));
                }
                let mut target_definition = self.partition_recursor_nodes.definition(*target)?;
                if target_definition.current.checked_frame_id
                    != obligation_definition.checked_frame_id
                    || target_definition.current.semantic_pending
                        != obligation_definition.semantic_pending
                {
                    return Err(unsupported(
                        "NativeExitScopeTransitionV1",
                        "exit start descriptor disagrees with its checked recursor frame",
                    ));
                }
                let mut captures = target_definition
                    .capture_field_types
                    .iter()
                    .enumerate()
                    .map(|(index, field_type)| {
                        let offset = index
                            .checked_add(1)
                            .and_then(|field| {
                                field.checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                            })
                            .and_then(|offset| i32::try_from(offset).ok())
                            .ok_or_else(|| {
                                unsupported(
                                    "NativeExitScopeTransitionV1",
                                    "exit start recursor-cell load offset overflowed",
                                )
                            })?;
                        Ok(builder.ins().load(
                            *field_type,
                            MemFlags::trusted(),
                            target_pointer,
                            offset,
                        ))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?
                    .into_iter();
                rebuild_partition_layer(
                    &mut target_definition.current,
                    &mut captures,
                    &mut self.native_int_tags,
                )?;
                if captures.next().is_some() {
                    return Err(unsupported(
                        "NativeExitScopeTransitionV1",
                        "exit start recursor cell has trailing fields",
                    ));
                }
                let RecursorLayerRole::ExitsScope {
                    scope_instance,
                    parent_scope_instance,
                    ..
                } = target_definition.current.role
                else {
                    return Err(unsupported(
                        "NativeExitScopeTransitionV1",
                        "exit start recursor node is not an ExitsScope edge",
                    ));
                };
                if obligation_definition.has_parent_scope != parent_scope_instance.is_some() {
                    return Err(unsupported(
                        "NativeExitScopeTransitionV1",
                        "exit start parent-scope schema changed",
                    ));
                }
                let obligation_target = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    obligation_pointer,
                    PARTITION_FRAME_FIELD_BYTES as i32,
                );
                let obligation_scope = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    obligation_pointer,
                    (PARTITION_FRAME_FIELD_BYTES * 2) as i32,
                );
                let obligation_parent = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    obligation_pointer,
                    (PARTITION_FRAME_FIELD_BYTES * 3) as i32,
                );
                let actual_parent = parent_scope_instance
                    .map_or_else(|| builder.ins().iconst(pointer_type, 0), |parent| parent.0);
                self.emit_control_cell_ref_guard(
                    builder,
                    &[
                        (target_pointer, obligation_target),
                        (scope_pointer, obligation_scope),
                        (parent_pointer, obligation_parent),
                        (scope_instance.0, scope_pointer),
                        (actual_parent, parent_pointer),
                    ],
                );
                let frame = EliminatorFrame::Computational(ComputationalEliminatorFrame {
                    cases: &target_definition.current.cases,
                    default: &target_definition.current.default,
                    env: &target_definition.current.outer_env,
                    retained_scrutinee_index: None,
                    deferred_constructor_case: None,
                    provenance: target_definition.current.provenance,
                    checked_frame_id: target_definition.current.checked_frame_id,
                    checked_invocation_id: target_definition.current.checked_invocation_id,
                    checked_invocation_source: target_definition.current.checked_invocation_source,
                    checked_invocation_depth: target_definition.current.checked_invocation_depth,
                });
                self.lower_computational_match_value_composed(
                    builder,
                    item.value,
                    std::slice::from_ref(&frame),
                )
            }
            ProducerKontAction::ExitScopeComplete {
                target,
                obligation,
                obligation_successor,
            } => {
                let capture_pointer = item.capture_pointer.expect("producer capture pointer");
                let pointer_type = builder.func.dfg.value_type(capture_pointer);
                let fields = (0..6)
                    .map(|index| {
                        builder.ins().load(
                            pointer_type,
                            MemFlags::trusted(),
                            capture_pointer,
                            index * PARTITION_FRAME_FIELD_BYTES as i32,
                        )
                    })
                    .collect::<Vec<_>>();
                let obligation_pointer = fields[0];
                let target_pointer = fields[1];
                let scope_pointer = fields[2];
                let parent_pointer = fields[3];
                let obligation_successor_pointer = fields[4];
                if let Some(successor) = &mut item.successor {
                    successor.capture_pointer = fields[5];
                }
                let obligation_definition = self
                    .partition_open_control_obligations
                    .definition(*obligation)?;
                if obligation_definition.target != *target
                    || obligation_definition.successor != *obligation_successor
                {
                    return Err(unsupported(
                        "NativeExitScopeTransitionV1",
                        "exit completion descriptor changed its exact obligation edge",
                    ));
                }
                let obligation_next =
                    builder
                        .ins()
                        .load(pointer_type, MemFlags::trusted(), obligation_pointer, 0);
                let obligation_target = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    obligation_pointer,
                    PARTITION_FRAME_FIELD_BYTES as i32,
                );
                let obligation_scope = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    obligation_pointer,
                    (PARTITION_FRAME_FIELD_BYTES * 2) as i32,
                );
                let obligation_parent = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    obligation_pointer,
                    (PARTITION_FRAME_FIELD_BYTES * 3) as i32,
                );
                self.emit_control_cell_ref_guard(
                    builder,
                    &[
                        (target_pointer, obligation_target),
                        (scope_pointer, obligation_scope),
                        (parent_pointer, obligation_parent),
                        (obligation_successor_pointer, obligation_next),
                    ],
                );
                Ok(item.value)
            }
        };
        self.active_partition_producer_kont = None;
        self.active_partition_return_contract = None;
        self.active_partition_return_kind = None;
        if loaded.next().is_some() {
            return Err(unsupported(
                "NativeProducerContinuationStepV1",
                "producer continuation input cell has trailing fields",
            ));
        }
        let lowered = lowered?;
        let lowered = match (&item.action, item.successor) {
            (ProducerKontAction::Done { .. }, None) => lowered,
            (ProducerKontAction::Done { .. }, Some(_)) => {
                unreachable!("explicit producer terminal successor was rejected before lowering")
            }
            (ProducerKontAction::ExitScopeStart { .. }, Some(_)) => lowered,
            (_, Some(successor)) => {
                self.call_partition_producer_kont(builder, successor, lowered)?
            }
            (_, None) => {
                return Err(unsupported(
                    "NativeProducerContinuationStepV1",
                    "nonterminal producer continuation reached scalarization without Done",
                ));
            }
        };
        match lowered {
            Lowered::Trap(trap) => {
                let payload = builder.ins().iconst(types::I64, -4);
                builder.ins().return_(&[payload]);
                Ok((Some(trap), ResultDecoder::ProcessStatus))
            }
            value => {
                let (pair, actual_kind) = self.merge_planned_scalar_branch(
                    builder,
                    value,
                    required_kind,
                    "NativeFunctionPartition",
                )?;
                debug_assert_eq!(actual_kind, required_kind);
                self.emit_partition_pair_return(builder, pair);
                Ok((None, ResultDecoder::ProcessStatus))
            }
        }
    }

    fn lower_source_machine(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: &RuntimeExpr,
        env: &[Lowered],
        active: &ActiveContinuationFrame<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let mut root_authority = self.root_terminal_authority.take();
        if let Some(authority) = &mut root_authority {
            match authority.outer_cursor {
                None => authority.outer_cursor = Some(active.cursor),
                Some(cursor) if cursor == active.cursor => {}
                Some(_) => {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked root answer authority was transplanted to another outer cursor",
                    ));
                }
            }
        }
        let producer_kont = self.active_partition_producer_kont;
        let control = SourceControl {
            continuation: SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected: active.cursor,
                active,
                root_authority,
            }),
            partition_cursor: None,
            producer_kont,
            selected: SourceSelectedContinuation {
                activation: active.activation,
                activation_instance: active.activation_instance,
                cursor: active.cursor,
                cursor_instance: active.cursor_instance,
                parent: active.parent,
                pending: active.pending.to_vec(),
                selected_ancestry: active.selected_ancestry.to_vec(),
                selected_scope: active.selected_scope.cloned(),
            },
            selected_lineage: Vec::new(),
            terminal_outer: active.cursor,
        };
        self.lower_source_machine_with_continuation(builder, expr.clone(), env.to_vec(), control)
    }

    fn lower_source_machine_with_continuation<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let previous_source_root = self.source_control_root.replace(control.terminal_outer);
        self.live_source_continuations = self
            .live_source_continuations
            .checked_add(1)
            .expect("compiler-private live source-continuation depth exhausted");
        let result = self.lower_source_machine_with_continuation_inner(builder, expr, env, control);
        self.live_source_continuations = self
            .live_source_continuations
            .checked_sub(1)
            .expect("source-continuation depth must balance");
        self.source_control_root = previous_source_root;
        result
    }

    fn lower_source_machine_with_continuation_inner<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        self.lower_source_machine_state_inner(
            builder,
            SourceMachineState::Eval { expr, env, control },
        )
    }

    fn lower_source_machine_state_inner<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        mut state: SourceMachineState<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let mut first_state = true;
        loop {
            self.check_partition_live_growth(builder)?;
            if !first_state && self.active_partition_return_contract.is_some() {
                match state {
                    SourceMachineState::Eval { expr, env, control } => {
                        return self.call_partition_source_eval(builder, expr, env, control);
                    }
                    SourceMachineState::Value { value, control }
                        if matches!(value, Lowered::Trap(_)) =>
                    {
                        return Ok(value);
                    }
                    SourceMachineState::Value { value, mut control } => {
                        if let Some(reserved) = self.reserve_partition_exit_source_cursor(
                            builder,
                            &mut control.partition_cursor,
                        )? {
                            let resume_active = source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                reserved.resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "NativeExitScopeTransitionV1",
                                    "reserved exit source cursor lost its resume owner",
                                )
                            })?;
                            self.emit_control_cell_ref_guard(
                                builder,
                                &[(
                                    resume_active.cursor_instance.0,
                                    reserved.resume_cursor_instance.0,
                                )],
                            );
                            self.install_partition_exit_scope_chain(
                                builder,
                                &mut control.selected,
                                &control.selected_lineage,
                                &mut control.producer_kont,
                                &reserved.popped,
                            )?;
                        }
                        if let Some(head) = control.partition_cursor {
                            return self.call_partition_source_kont(builder, head, value, control);
                        }
                        state = SourceMachineState::Value { value, control };
                    }
                }
            }
            first_state = false;
            state = match state {
                SourceMachineState::Eval {
                    expr,
                    env,
                    mut control,
                } => match expr {
                    RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                        self.enter_checked_subcontinuation_frame(frame_id)?;
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedRecursiveInvocation {
                        call_template_id,
                        checked_occurrence_path: _,
                        body,
                    } => {
                        let instance =
                            self.enter_checked_recursive_invocation(call_template_id, &body)?;
                        control.continuation =
                            SourceContinuation::CheckedRecursiveInvocationReturn {
                                instance,
                                next: Box::new(control.continuation),
                            };
                        self.push_partition_source_cursor(builder, &mut control)?;
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHInvocation {
                        call_template_id,
                        checked_occurrence_path: _,
                        body,
                    } => {
                        self.enter_checked_computational_ih_invocation(call_template_id)?;
                        control.continuation =
                            SourceContinuation::CheckedComputationalIHInvocationReturn {
                                call_template_id,
                                next: Box::new(control.continuation),
                            };
                        self.push_partition_source_cursor(builder, &mut control)?;
                        SourceMachineState::Eval {
                            expr: *body,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Value(value) => SourceMachineState::Value {
                        value: self.lower_value(builder, &value)?,
                        control,
                    },
                    RuntimeExpr::Var(index) => SourceMachineState::Value {
                        value: env.get(index as usize).cloned().ok_or_else(|| {
                            unsupported("Var", format!("no runtime binding for index {index}"))
                        })?,
                        control,
                    },
                    RuntimeExpr::Let { value, body } => {
                        control.continuation = SourceContinuation::LetBody {
                            body: *body,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        self.push_partition_source_cursor(builder, &mut control)?;
                        SourceMachineState::Eval {
                            expr: *value,
                            env: env.clone(),
                            control,
                        }
                    }
                    RuntimeExpr::Construct {
                        constructor,
                        mut args,
                    } => {
                        if args.is_empty() {
                            SourceMachineState::Value {
                                value: self.finish_source_constructor(
                                    builder,
                                    constructor,
                                    vec![],
                                )?,
                                control,
                            }
                        } else {
                            let first = args.remove(0);
                            control.continuation = SourceContinuation::ConstructArgument {
                                constructor,
                                remaining: args,
                                lowered: Vec::new(),
                                env: env.clone(),
                                next: Box::new(control.continuation),
                            };
                            self.push_partition_source_cursor(builder, &mut control)?;
                            SourceMachineState::Eval {
                                expr: first,
                                env,
                                control,
                            }
                        }
                    }
                    RuntimeExpr::Match {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        control.continuation = SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        self.push_partition_source_cursor(builder, &mut control)?;
                        SourceMachineState::Eval {
                            expr: *scrutinee,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Call { callee, args } => {
                        control.continuation = SourceContinuation::CallCallee {
                            args,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        self.push_partition_source_cursor(builder, &mut control)?;
                        SourceMachineState::Eval {
                            expr: *callee,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::ComputationalMatch {
                        scrutinee,
                        cases,
                        default,
                    } => {
                        let checked_frame_id =
                            self.consume_checked_subcontinuation_frame(&cases, &default)?;
                        control.continuation = SourceContinuation::ComputationalMatchScrutinee {
                            cases,
                            default,
                            env: env.clone(),
                            provenance: self.mint_recursor_frame_provenance(),
                            checked_frame_id,
                            answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                            next: Box::new(control.continuation),
                        };
                        self.push_partition_source_cursor(builder, &mut control)?;
                        SourceMachineState::Eval {
                            expr: *scrutinee,
                            env,
                            control,
                        }
                    }
                    other => SourceMachineState::Value {
                        value: self.lower_expr(builder, &other, &env)?,
                        control,
                    },
                },
                SourceMachineState::Value { value, mut control } => {
                    if matches!(value, Lowered::Trap(_)) {
                        control.continuation = Self::discard_source_prefix(control.continuation);
                        control.partition_cursor = None;
                    } else if !matches!(
                        &control.continuation,
                        SourceContinuation::Terminal(_) | SourceContinuation::Partitioned { .. }
                    ) {
                        self.pop_partition_source_cursor(builder, &mut control)?;
                    }
                    match control.continuation {
                        SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                            return Ok(value);
                        }
                        SourceContinuation::Terminal(
                            SourceContinuationTerminal::ReturnToProducerHole {
                                stack,
                                resume_cursor,
                                resume_cursor_instance,
                                expected,
                                active,
                                root_authority,
                            },
                        ) => {
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::ReturnHole {
                                cursor: resume_cursor,
                            });
                            if active.cursor != expected {
                                return Err(unsupported(
                                    "ComputationalRecursor",
                                    "producer-hole terminal cursor mismatch",
                                ));
                            }
                            if matches!(value, Lowered::Trap(_)) {
                                return Ok(value);
                            }
                            let resume_active = source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "producer-hole resume cursor is no longer active",
                                )
                            })?;
                            self.emit_control_cell_ref_guard(
                                builder,
                                &[(resume_active.cursor_instance.0, resume_cursor_instance.0)],
                            );
                            control.continuation = SourceContinuation::UnwindRecursorSegment {
                                stack,
                                resume_cursor,
                                resume_cursor_instance,
                                next: Box::new(SourceContinuation::Terminal(
                                    SourceContinuationTerminal::ResumeOuter {
                                        expected,
                                        active,
                                        root_authority,
                                    },
                                )),
                            };
                            self.push_partition_source_cursor(builder, &mut control)?;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                            expected,
                            active,
                            root_authority,
                        }) => {
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::ResumeOuter {
                                cursor: expected,
                            });
                            if active.cursor != expected {
                                return Err(unsupported(
                                    "ComputationalRecursor",
                                    "source continuation terminal cursor mismatch",
                                ));
                            }
                            if control.producer_kont.is_some()
                                && (control.selected.cursor != expected
                                    || !control.selected_lineage.is_empty())
                            {
                                return Err(unsupported(
                                    "NativeProducerContinuationStepV1",
                                    "source terminal reached producer transfer before restoring \
                                     its entry selected head",
                                ));
                            }
                            self.restore_root_terminal_authority(root_authority, expected)?;
                            if matches!(value, Lowered::Trap(_)) {
                                return Ok(value);
                            }
                            return if let Some(producer_kont) = control.producer_kont {
                                if self.producer_kont_is_scope_exit_bridge(producer_kont)? {
                                    if !control.selected_lineage.is_empty() {
                                        return Err(unsupported(
                                            "NativeExitScopeTransitionV1",
                                            "scope-exit completion retained a child selected head",
                                        ));
                                    }
                                    self.call_partition_producer_kont(builder, producer_kont, value)
                                } else {
                                    self.call_restored_selected_producer_kont(
                                        builder,
                                        producer_kont,
                                        value,
                                        &control.selected,
                                        &control.selected_lineage,
                                    )
                                }
                            } else {
                                self.resume_active_continuation(builder, value, *active)
                            };
                        }
                        SourceContinuation::Terminal(
                            SourceContinuationTerminal::ReturnFromPartition { expected_outer },
                        ) => {
                            if control.terminal_outer != expected_outer {
                                return Err(unsupported(
                                    "NativeFunctionPartition",
                                    "source arm returned through the wrong outer cursor",
                                ));
                            }
                            return if let Some(producer_kont) = control.producer_kont {
                                if self.producer_kont_is_scope_exit_bridge(producer_kont)? {
                                    if !control.selected_lineage.is_empty() {
                                        return Err(unsupported(
                                            "NativeExitScopeTransitionV1",
                                            "partition scope-exit completion retained a child \
                                             selected head",
                                        ));
                                    }
                                    self.call_partition_producer_kont(builder, producer_kont, value)
                                } else {
                                    self.call_restored_selected_producer_kont(
                                        builder,
                                        producer_kont,
                                        value,
                                        &control.selected,
                                        &control.selected_lineage,
                                    )
                                }
                            } else {
                                Ok(value)
                            };
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(
                            edge,
                        )) => {
                            if matches!(value, Lowered::Trap(_)) {
                                let failure = builder.ins().iconst(types::I64, -4);
                                builder.ins().return_(&[failure]);
                                return Ok(Lowered::RecursiveBackedge);
                            }
                            let value = if edge.target.terminal_active_prefix.is_empty() {
                                value
                            } else {
                                let mut prefix = edge.target.terminal_active_prefix;
                                prefix.push(EliminatorFrame::InvocationReturn);
                                self.lower_computational_match_value_composed(
                                    builder, value, &prefix,
                                )?
                            };
                            let (value, actual_kind) = self.merge_planned_scalar_branch(
                                builder,
                                value,
                                edge.target.required_kind,
                                "NativeJoinPlanV1",
                            )?;
                            if actual_kind != ScalarMergeKind::RecursiveBackedge
                                && actual_kind != edge.target.required_kind
                            {
                                return Err(unsupported(
                                    "NativeJoinPlanV1",
                                    format!(
                                        "predecessor {} for join {} produced {actual_kind:?}, planned {:?}",
                                        edge.predecessor_identity,
                                        edge.target.join_id,
                                        edge.target.required_kind
                                    ),
                                ));
                            }
                            builder
                                .ins()
                                .jump(edge.target.block, &[value.tag.into(), value.payload.into()]);
                            return Ok(Lowered::RecursiveBackedge);
                        }
                        SourceContinuation::LetBody { body, env, next } => {
                            control.continuation = *next;
                            if matches!(value, Lowered::RecursiveBackedge) {
                                SourceMachineState::Value { value, control }
                            } else if matches!(value, Lowered::Trap(_)) {
                                SourceMachineState::Value { value, control }
                            } else {
                                let mut body_env = vec![value];
                                body_env.extend(env);
                                SourceMachineState::Eval {
                                    expr: body,
                                    env: body_env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                            self.leave_checked_recursive_invocation(instance)?;
                            control.continuation = *next;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::CheckedComputationalIHInvocationReturn {
                            call_template_id,
                            next,
                        } => {
                            if self
                                .pending_computational_ih_call
                                .is_some_and(|pending| pending != call_template_id)
                            {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "computational IH invocation return crossed another marker",
                                ));
                            }
                            let value =
                                self.finish_checked_computational_ih_marker(builder, value)?;
                            control.continuation = *next;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                            let scope =
                                control.selected.selected_scope.as_ref().ok_or_else(|| {
                                    unsupported(
                                        "OrientedSubcontinuationPlanV1",
                                        "selected-case return has no open control obligation",
                                    )
                                })?;
                            if scope.frame.checked_frame_id != delimiter.frame_id {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    format!(
                                        "selected-case return delimiter does not match its open occurrence: \
                                         current activation={:?} cursor={:?} scope={:?} frame={:?} invocation={:?}; \
                                         delimiter activation={:?} cursor={:?} scope={:?} frame={:?} invocation={:?}",
                                        control.selected.activation,
                                        control.selected.cursor,
                                        scope.scope_origin,
                                        scope.frame.checked_frame_id,
                                        scope.frame.checked_invocation_id,
                                        delimiter.activation,
                                        delimiter.cursor,
                                        delimiter.scope_origin,
                                        delimiter.frame_id,
                                        delimiter.invocation_id,
                                    ),
                                ));
                            }
                            self.emit_control_cell_ref_guard(
                                builder,
                                &[
                                    (
                                        control.selected.activation_instance.0,
                                        delimiter.activation_instance.0,
                                    ),
                                    (
                                        control.selected.cursor_instance.0,
                                        delimiter.cursor_instance.0,
                                    ),
                                    (scope.scope_instance.0, delimiter.scope_instance.0),
                                ],
                            );
                            let mut previous = control.selected_lineage.pop().ok_or_else(|| {
                                unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return has no exact parent control state",
                                )
                            })?;
                            let pointer_type = builder
                                .func
                                .dfg
                                .value_type(control.selected.cursor_instance.0);
                            previous.activation_instance =
                                ActivationInstanceRef(builder.ins().load(
                                    pointer_type,
                                    MemFlags::trusted(),
                                    control.selected.cursor_instance.0,
                                    0,
                                ));
                            previous.cursor_instance = ControlCursorRef(builder.ins().load(
                                pointer_type,
                                MemFlags::trusted(),
                                control.selected.cursor_instance.0,
                                PARTITION_FRAME_FIELD_BYTES as i32,
                            ));
                            if let Some(parent_scope) = &mut previous.selected_scope {
                                parent_scope.scope_instance = ScopeInstanceRef(builder.ins().load(
                                    pointer_type,
                                    MemFlags::trusted(),
                                    control.selected.cursor_instance.0,
                                    (PARTITION_FRAME_FIELD_BYTES * 2) as i32,
                                ));
                                if parent_scope.parent_scope.is_some() {
                                    parent_scope.parent_scope_instance =
                                        Some(ScopeInstanceRef(builder.ins().load(
                                            pointer_type,
                                            MemFlags::trusted(),
                                            parent_scope.scope_instance.0,
                                            (PARTITION_FRAME_FIELD_BYTES * 2) as i32,
                                        )));
                                }
                            }
                            control.selected = previous;
                            let mut next = *next;
                            let popped =
                                self.reserve_partition_scope_exits_from_tail(builder, &mut next)?;
                            self.install_partition_exit_scope_chain(
                                builder,
                                &mut control.selected,
                                &control.selected_lineage,
                                &mut control.producer_kont,
                                &popped,
                            )?;
                            control.continuation = next;
                            if let Some(producer_kont) = control.producer_kont {
                                if self.producer_kont_starts_selected_scope_exit(
                                    producer_kont,
                                    control.selected.selected_scope.as_ref(),
                                )? {
                                    return self.call_partition_producer_kont(
                                        builder,
                                        producer_kont,
                                        value,
                                    );
                                }
                            }
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::ApplyRecursorSelection { layer, next } => {
                            #[cfg(test)]
                            match layer.role {
                                RecursorLayerRole::SelectsOccurrence { origin, .. } => {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Selection {
                                        origin,
                                    });
                                }
                                RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                    ..
                                } => px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                }),
                            }
                            let answer_route =
                                SourceComputationalAnswerRoute::for_recursor_layer(&layer);
                            control.continuation =
                                SourceContinuation::ComputationalMatchScrutinee {
                                    cases: layer.cases,
                                    default: layer.default,
                                    env: layer.outer_env,
                                    provenance: layer.provenance,
                                    checked_frame_id: layer.checked_frame_id,
                                    answer_route,
                                    next,
                                };
                            self.push_partition_source_cursor(builder, &mut control)?;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::UnwindRecursorSegment {
                            mut stack,
                            resume_cursor,
                            resume_cursor_instance,
                            next,
                        } => {
                            let resume_active = source_active_cursor(
                                &control.selected,
                                &control.selected_lineage,
                                resume_cursor,
                            )
                            .ok_or_else(|| {
                                unsupported(
                                    "ComputationalRecursor",
                                    "source recursor resume cursor is no longer active",
                                )
                            })?;
                            self.emit_control_cell_ref_guard(
                                builder,
                                &[(resume_active.cursor_instance.0, resume_cursor_instance.0)],
                            );
                            if stack.partition_cursor.is_some() {
                                let popped =
                                    self.drain_partition_exit_stack(builder, &mut stack)?;
                                self.install_partition_exit_scope_chain(
                                    builder,
                                    &mut control.selected,
                                    &control.selected_lineage,
                                    &mut control.producer_kont,
                                    &popped,
                                )?;
                                control.continuation = *next;
                                SourceMachineState::Value { value, control }
                            } else if let Some(popped) =
                                self.pop_partition_recursor_layer(builder, &mut stack)?
                            {
                                let layer = popped.layer;
                                if let RecursorLayerRole::ExitsScope {
                                    scope_origin,
                                    scope_instance,
                                    parent_scope,
                                    parent_scope_instance,
                                    ..
                                } = layer.role
                                {
                                    control.selected.activation_instance =
                                        ActivationInstanceRef(scope_instance.0);
                                    control.selected.cursor_instance =
                                        ControlCursorRef(scope_instance.0);
                                    control.selected.selected_scope = Some(OwnedSelectedScope {
                                        scope_origin,
                                        scope_instance,
                                        parent_scope,
                                        parent_scope_instance,
                                        frame: ComputationalRecursorFramePayload {
                                            cases: layer.cases.clone(),
                                            default: layer.default.clone(),
                                            outer_env: layer.outer_env.clone(),
                                            provenance: layer.provenance,
                                            checked_frame_id: layer.checked_frame_id,
                                            checked_invocation_id: layer.checked_invocation_id,
                                            checked_invocation_source: layer
                                                .checked_invocation_source,
                                            checked_invocation_depth: layer
                                                .checked_invocation_depth,
                                        },
                                    });
                                }
                                #[cfg(test)]
                                if let RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
                                    ..
                                } = layer.role
                                {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Exit {
                                        origin,
                                        scope_origin,
                                        parent_scope,
                                    });
                                }
                                let answer_route =
                                    SourceComputationalAnswerRoute::for_recursor_layer(&layer);
                                control.continuation = SourceContinuation::UnwindRecursorSegment {
                                    stack,
                                    resume_cursor,
                                    resume_cursor_instance,
                                    next,
                                };
                                self.push_partition_source_cursor(builder, &mut control)?;
                                control.continuation =
                                    SourceContinuation::ComputationalMatchScrutinee {
                                        cases: layer.cases,
                                        default: layer.default,
                                        env: layer.outer_env,
                                        provenance: layer.provenance,
                                        checked_frame_id: layer.checked_frame_id,
                                        answer_route,
                                        next: Box::new(control.continuation),
                                    };
                                self.push_partition_source_cursor(builder, &mut control)?;
                                SourceMachineState::Value { value, control }
                            } else {
                                control.continuation = *next;
                                SourceMachineState::Value { value, control }
                            }
                        }
                        SourceContinuation::ConstructArgument {
                            constructor,
                            mut remaining,
                            mut lowered,
                            env,
                            next,
                        } => {
                            lowered.push(value);
                            control.continuation = *next;
                            if remaining.is_empty() {
                                SourceMachineState::Value {
                                    value: self.finish_source_constructor(
                                        builder,
                                        constructor,
                                        lowered,
                                    )?,
                                    control,
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::ConstructArgument {
                                    constructor,
                                    remaining,
                                    lowered,
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                self.push_partition_source_cursor(builder, &mut control)?;
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::MatchScrutinee {
                            cases,
                            default,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            match value {
                                Lowered::BoundedNat(nat) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder, nat, false, &cases, &default, &env, control,
                                    );
                                }
                                Lowered::StructuralNat(nat) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder,
                                        BoundedNatV1::derived_from_validated(nat.value),
                                        true,
                                        &cases,
                                        &default,
                                        &env,
                                        control,
                                    );
                                }
                                Lowered::Bool { value, known } => {
                                    let true_case = cases.iter().find(|case| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::True")
                                    });
                                    let false_case = cases.iter().find(|case| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::False")
                                    });
                                    let (Some(true_case), Some(false_case)) =
                                        (true_case, false_case)
                                    else {
                                        return Err(unsupported(
                                            "Match",
                                            "Bool match requires zero-binder True and False cases",
                                        ));
                                    };
                                    if let Some(selected) = known {
                                        SourceMachineState::Eval {
                                            expr: if selected {
                                                true_case.body.clone()
                                            } else {
                                                false_case.body.clone()
                                            },
                                            env,
                                            control,
                                        }
                                    } else {
                                        return self.lower_source_dynamic_bool_match(
                                            builder,
                                            value,
                                            &true_case.body,
                                            &false_case.body,
                                            &env,
                                            control,
                                        );
                                    }
                                }
                                Lowered::HostResult {
                                    success,
                                    error,
                                    ok,
                                    err_constructor,
                                    ok_constructor,
                                } => {
                                    return self.lower_source_dynamic_host_result_match(
                                        builder,
                                        success,
                                        *error,
                                        *ok,
                                        &err_constructor,
                                        &ok_constructor,
                                        &cases,
                                        default,
                                        &env,
                                        control,
                                    );
                                }
                                Lowered::DynamicConstructor(dynamic) => {
                                    return self.lower_source_dynamic_constructor_match(
                                        builder, dynamic, &cases, &default, &env, control,
                                    );
                                }
                                Lowered::Constructor { constructor, args } => {
                                    let Some(case) =
                                        cases.iter().find(|case| case.constructor == constructor)
                                    else {
                                        return Ok(Lowered::Trap(default));
                                    };
                                    if case.binders != args.len() {
                                        return Err(unsupported(
                                            "Match",
                                            format!(
                                                "case {} expects {} binders but constructor has {} args",
                                                case.constructor,
                                                case.binders,
                                                args.len()
                                            ),
                                        ));
                                    }
                                    let mut case_env = args;
                                    case_env.extend(env);
                                    SourceMachineState::Eval {
                                        expr: case.body.clone(),
                                        env: case_env,
                                        control,
                                    }
                                }
                                _ => {
                                    return Err(unsupported(
                                        "Match",
                                        "scrutinee is not a constructor value",
                                    ));
                                }
                            }
                        }
                        SourceContinuation::ComputationalMatchScrutinee {
                            cases,
                            default,
                            env,
                            provenance,
                            checked_frame_id,
                            answer_route,
                            next,
                        } => {
                            let retained = value.clone();
                            #[cfg(test)]
                            let actual_constructor = match &value {
                                Lowered::Constructor { constructor, .. } => {
                                    Some(constructor.clone())
                                }
                                _ => None,
                            };
                            let selected = match &value {
                                Lowered::Constructor { constructor, .. } => {
                                    cases.iter().find(|case| case.constructor == *constructor)
                                }
                                _ => None,
                            };
                            let case = if let Some(case) = selected {
                                case
                            } else if answer_route
                                == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                && matches!(&value, Lowered::Constructor { .. })
                                && px8tr_deforested_answer_route_enabled()
                            {
                                let mut returns = cases.iter().filter(|case| {
                                    case.argument_binders == 1
                                        && case.constructor.ends_with("::ITree::Ret")
                                });
                                let return_case = returns.next();
                                let exact_return = returns.next().is_none();
                                let mut visible = cases
                                    .iter()
                                    .filter(|case| case.constructor.ends_with("::ITree::Vis"));
                                let exact_visible = visible.next().is_some()
                                    && visible.next().is_none()
                                    && cases.len() == 2;
                                let Some(return_case) = return_case.filter(|return_case| {
                                    exact_return
                                        && exact_visible
                                        && source_case_has_no_checked_control_markers(
                                            &return_case.body,
                                        )
                                }) else {
                                    #[cfg(test)]
                                    px8tr_record_trap_provenance(
                                        Px8trTrapProvenanceEvent::CheckedRecursorDefault {
                                            checked_frame_id: checked_frame_id.expect(
                                                "checked answer routes carry exact frame ids",
                                            ),
                                            actual_constructor,
                                            trap: default.clone(),
                                        },
                                    );
                                    return Ok(Lowered::Trap(default));
                                };
                                #[cfg(test)]
                                px8tr_record_trap_provenance(
                                    Px8trTrapProvenanceEvent::DeforestedAnswerResumed {
                                        checked_frame_id: checked_frame_id
                                            .expect("checked answer routes carry exact frame ids"),
                                        actual_constructor,
                                        return_constructor: return_case.constructor.clone(),
                                    },
                                );
                                let mut case_env = vec![retained];
                                case_env.extend(env);
                                control.continuation = *next;
                                return self.lower_source_machine_with_continuation(
                                    builder,
                                    return_case.body.clone(),
                                    case_env,
                                    control,
                                );
                            } else {
                                if !matches!(&value, Lowered::Constructor { .. }) {
                                    return Err(unsupported(
                                        "ComputationalMatch",
                                        "source scrutinee is not a constructor value",
                                    ));
                                }
                                #[cfg(test)]
                                if answer_route
                                    == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                {
                                    px8tr_record_trap_provenance(
                                        Px8trTrapProvenanceEvent::CheckedRecursorDefault {
                                            checked_frame_id: checked_frame_id.expect(
                                                "checked answer routes carry exact frame ids",
                                            ),
                                            actual_constructor,
                                            trap: default.clone(),
                                        },
                                    );
                                }
                                return Ok(Lowered::Trap(default));
                            };
                            let Lowered::Constructor { args, .. } = value else {
                                unreachable!("a selected source case has a constructor value")
                            };
                            if case.argument_binders != args.len() {
                                return Err(unsupported(
                                    "ComputationalMatch",
                                    format!(
                                        "case {} expects {} constructor arguments but value has {}",
                                        case.constructor,
                                        case.argument_binders,
                                        args.len()
                                    ),
                                ));
                            }
                            let mut seen = BTreeSet::new();
                            for position in case.recursive_positions.iter().copied() {
                                if !seen.insert(position) || position >= args.len() {
                                    return Err(unsupported(
                                        "ComputationalMatch",
                                        format!(
                                            "case {} has malformed recursive position {position}",
                                            case.constructor
                                        ),
                                    ));
                                }
                            }
                            let frame = ComputationalEliminatorFrame {
                                cases: &cases,
                                default: &default,
                                env: &env,
                                retained_scrutinee_index: None,
                                deferred_constructor_case: None,
                                provenance,
                                checked_frame_id,
                                checked_invocation_id: checked_frame_id.map(|_| {
                                    self.active_recursive_invocations
                                        .last()
                                        .map_or(0, |instance| instance.invocation_instance_id)
                                }),
                                checked_invocation_source: self
                                    .active_recursive_invocations
                                    .last()
                                    .map(|instance| instance.source),
                                checked_invocation_depth: self
                                    .active_recursive_invocations
                                    .last()
                                    .map_or(0, |instance| instance.semantic_depth),
                            };
                            let activation = self.mint_continuation_activation();
                            let cursor = self.mint_continuation_cursor();
                            let (activation_instance, cursor_instance, scope_instance) = self
                                .allocate_selected_control_refs(
                                    builder,
                                    Some(control.selected.activation_instance),
                                    Some(control.selected.cursor_instance),
                                    control
                                        .selected
                                        .selected_scope
                                        .as_ref()
                                        .map(|scope| scope.scope_instance),
                                )?;
                            let mut ancestry = control.selected.selected_ancestry.clone();
                            ancestry.push(provenance);
                            let mut induction_hypotheses =
                                Vec::with_capacity(case.recursive_positions.len());
                            let ih_slots =
                                self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                            let producer_origin = self.mint_recursor_producer_origin();
                            #[cfg(test)]
                            px8j_record_source_event(Px8jSourceTraceEvent::Mint {
                                path: Px8jProducerPath::SourceMachine,
                                origin: producer_origin,
                                cursor,
                                siblings: case.recursive_positions.len(),
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                            });
                            let parent = control.selected.parent;
                            {
                                let qold = control.selected.as_active(&control.selected_lineage);
                                for position in case.recursive_positions.iter().rev().copied() {
                                    let slot_template_id = case
                                        .recursive_positions
                                        .iter()
                                        .position(|candidate| *candidate == position)
                                        .and_then(|index| ih_slots[index]);
                                    let induction_hypothesis = self.make_computational_recursor(
                                        builder,
                                        args[position].clone(),
                                        cases.clone(),
                                        default.clone(),
                                        env.clone(),
                                        provenance,
                                        frame.checked_frame_id,
                                        slot_template_id,
                                        producer_origin,
                                        position,
                                        RecursorLayerRole::SelectsOccurrence {
                                            origin: producer_origin,
                                            origin_scope: scope_instance,
                                        },
                                        activation,
                                        cursor,
                                        cursor_instance,
                                        Some(&qold),
                                        Some((
                                            &control.selected,
                                            control.selected_lineage.as_slice(),
                                        )),
                                    )?;
                                    #[cfg(test)]
                                    px8j_record_recursor_carrier(
                                        Px8jProducerPath::SourceMachine,
                                        &induction_hypothesis,
                                    );
                                    induction_hypotheses.push(induction_hypothesis);
                                }
                            }
                            let frame_env = match self.materialize_eliminator_frame_env(
                                builder,
                                EliminatorFrame::Computational(frame),
                                &retained,
                            )? {
                                Ok(frame_env) => frame_env,
                                Err(trap) => return Ok(Lowered::Trap(trap)),
                            };
                            let mut case_env = induction_hypotheses;
                            case_env.extend(args);
                            case_env.extend(frame_env);
                            let previous_selected = control.selected.clone();
                            let pending = std::mem::take(&mut control.selected.pending);
                            let selected_scope = OwnedSelectedScope {
                                scope_origin: producer_origin,
                                scope_instance,
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                                parent_scope_instance: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_instance),
                                frame: ComputationalRecursorFramePayload {
                                    cases: cases.clone(),
                                    default: default.clone(),
                                    outer_env: env.clone(),
                                    provenance,
                                    checked_frame_id: frame.checked_frame_id,
                                    checked_invocation_id: frame.checked_invocation_id,
                                    checked_invocation_source: frame.checked_invocation_source,
                                    checked_invocation_depth: frame.checked_invocation_depth,
                                },
                            };
                            #[cfg(test)]
                            let selected_scope =
                                (!PX8J_DELETE_OWNED_SELECTED_SCOPE.get()).then_some(selected_scope);
                            #[cfg(not(test))]
                            let selected_scope = Some(selected_scope);
                            control.continuation = if frame.checked_frame_id.is_some() {
                                let selected_scope_ref =
                                    selected_scope.as_ref().ok_or_else(|| {
                                        unsupported(
                                            "OrientedSubcontinuationPlanV1",
                                            "checked selection has no owned open-control obligation",
                                        )
                                    })?;
                                SourceContinuation::ReturnFromSelectedCase {
                                    delimiter: SelectedCaseReturnDelimiter {
                                        activation,
                                        activation_instance,
                                        cursor,
                                        cursor_instance,
                                        scope_origin: selected_scope_ref.scope_origin,
                                        scope_instance,
                                        frame_id: selected_scope_ref.frame.checked_frame_id,
                                        invocation_id: selected_scope_ref
                                            .frame
                                            .checked_invocation_id,
                                    },
                                    next,
                                }
                            } else {
                                *next
                            };
                            if frame.checked_frame_id.is_some() {
                                self.push_partition_source_cursor(builder, &mut control)?;
                            }
                            control.selected = SourceSelectedContinuation {
                                activation,
                                activation_instance,
                                cursor,
                                cursor_instance,
                                parent,
                                pending,
                                selected_ancestry: ancestry,
                                selected_scope,
                            };
                            control.selected_lineage.push(previous_selected);
                            SourceMachineState::Eval {
                                expr: case.body.clone(),
                                env: case_env,
                                control,
                            }
                        }
                        SourceContinuation::CallCallee {
                            mut args,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            if args.is_empty() {
                                match self.source_call_state(
                                    builder,
                                    value,
                                    Vec::new(),
                                    env,
                                    control,
                                )? {
                                    SourceCallOutcome::Continue(state) => state,
                                    SourceCallOutcome::Complete(value) => return Ok(value),
                                }
                            } else {
                                let first = args.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee: value,
                                    remaining: args,
                                    lowered: Vec::new(),
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                self.push_partition_source_cursor(builder, &mut control)?;
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::CallArgument {
                            callee,
                            mut remaining,
                            mut lowered,
                            env,
                            next,
                        } => {
                            lowered.push(value);
                            control.continuation = *next;
                            if remaining.is_empty() {
                                match self
                                    .source_call_state(builder, callee, lowered, env, control)?
                                {
                                    SourceCallOutcome::Continue(state) => state,
                                    SourceCallOutcome::Complete(value) => return Ok(value),
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    callee,
                                    remaining,
                                    lowered,
                                    env: env.clone(),
                                    next: Box::new(control.continuation),
                                };
                                self.push_partition_source_cursor(builder, &mut control)?;
                                SourceMachineState::Eval {
                                    expr: first,
                                    env,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::IfScrutinee { .. }
                        | SourceContinuation::ProjectRecord { .. } => {
                            return Err(unsupported(
                                "ComputationalRecursor",
                                "source continuation frame is not implemented",
                            ));
                        }
                        SourceContinuation::Partitioned { head, terminal } => {
                            control.continuation = SourceContinuation::Terminal(terminal);
                            return self.call_partition_source_kont(builder, head, value, control);
                        }
                    }
                }
            };
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_bounded_nat_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let zero = cases
            .iter()
            .find(|case| case.constructor == self.process_symbols.nat_zero && case.binders == 0);
        let suc = cases
            .iter()
            .find(|case| case.constructor == self.process_symbols.nat_suc && case.binders == 1);
        let (Some(zero), Some(suc)) = (zero, suc) else {
            return Err(unsupported(
                "BoundedNat",
                "structural Nat source match requires exact Zero and Suc predecessor arms",
            ));
        };

        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let (source_prefix_template, target) = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => {
                let fanout = SourceBranchFanout {
                    source_prefix_template,
                    inherited_edge,
                };
                (fanout.source_prefix_template, fanout.inherited_edge.target)
            }
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                (
                    source_prefix_template,
                    SourceJoinTarget {
                        join_id,
                        checked_site_id: site_id,
                        block: merge,
                        expected_outer: suffix_control.terminal_outer,
                        required_kind,
                        terminal_active_prefix: prefix,
                    },
                )
            }
            SourcePrefixTerminal::ReturnFromPartition => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion =
                    Some((merge, suffix_pending.to_vec(), required_kind, site_id, None));
                (
                    source_prefix_template,
                    SourceJoinTarget {
                        join_id,
                        checked_site_id: site_id,
                        block: merge,
                        expected_outer: suffix_control.terminal_outer,
                        required_kind,
                        terminal_active_prefix: prefix,
                    },
                )
            }
        };

        let zero_block = builder.create_block();
        let suc_block = builder.create_block();
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);

        let owned_return_eliminators = own_partition_eliminators(&target.terminal_active_prefix);
        let owned_selected_lineage =
            own_partition_selected_lineage(&suffix_control.selected_lineage);
        let predecessor_partition_eligible = self.active_partition_return_kind.is_some()
            && partition_helper_return_kind_is_admissible(target.required_kind)
            && suffix_control.selected.parent.is_none()
            && suffix_control.selected.pending.is_empty()
            && partition_prefix_is_admissible(&source_prefix_template)
            && partition_scope_is_admissible(&suffix_control.selected.selected_scope)
            && env.iter().all(partition_lowered_is_admissible)
            && owned_return_eliminators
                .as_ref()
                .is_some_and(|frames| partition_eliminators_are_admissible(frames))
            && owned_selected_lineage.is_some();
        let partition_site_id = if predecessor_partition_eligible {
            let site_id = self.partition_next_site;
            self.partition_next_site =
                self.partition_next_site.checked_add(1).ok_or_else(|| {
                    unsupported(
                        "NativeFunctionPartition",
                        "Nat predecessor fanout identity exhausted",
                    )
                })?;
            Some(site_id)
        } else {
            None
        };
        let ledger_baseline = self.partition_ledger_baseline();
        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for (edge_index, arm_name, block, case, predecessor) in [
            (0_u64, "Zero", zero_block, zero, None),
            (1_u64, "Suc", suc_block, suc, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let mut arm_env = predecessor
                .map(|predecessor| {
                    vec![if structural {
                        Lowered::StructuralNat(StructuralNatV1 {
                            value: predecessor.value,
                        })
                    } else {
                        Lowered::BoundedNat(predecessor)
                    }]
                })
                .unwrap_or_default();
            arm_env.extend_from_slice(env);
            if let Some(partition_site_id) = partition_site_id {
                self.call_partition_source_predecessor(
                    builder,
                    partition_site_id,
                    edge_index,
                    case.body.clone(),
                    false,
                    arm_env,
                    &source_prefix_template,
                    suffix_control.partition_cursor,
                    &suffix_control.selected,
                    suffix_control.terminal_outer,
                    &target,
                    owned_selected_lineage
                        .as_deref()
                        .expect("eligibility proves owned selected lineage"),
                    owned_return_eliminators
                        .as_deref()
                        .expect("eligibility proves owned return eliminators"),
                    &ledger_baseline,
                )?;
                continue;
            }
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                partition_cursor: suffix_control.partition_cursor,
                producer_kont: suffix_control.producer_kont,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                case.body.clone(),
                arm_env,
                branch_control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                let detail = match &lowered {
                    Lowered::Trap(trap) => format!("Trap({}: {:?})", trap.message, trap.code),
                    other => lowered_value_kind(other).to_string(),
                };
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "bounded-Nat {arm_name} arm produced {detail} instead of sealing its distinct affine predecessor edge"
                    ),
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_union;

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            activation_instance: suffix_control.selected.activation_instance,
            cursor: suffix_control.selected.cursor,
            cursor_instance: suffix_control.selected.cursor_instance,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    /// Lower one mutually-exclusive match arm with the checked-subcontinuation-
    /// frame consumption set rewound to `frame_baseline`, then fold the arm's
    /// resulting consumptions into `frame_union`.
    ///
    /// A dynamic match lowers its shared post-match continuation once per arm —
    /// each arm inlines its own copy of the source-prefix template. The arms are
    /// mutually exclusive at run time (selected by one `brif`), so a checked
    /// subcontinuation frame occurring in that shared continuation is a *distinct
    /// lawful activation per arm*, not a repeated consumption of one activation.
    /// `consumed_subcontinuation_frames` is a single per-lowering set, so without
    /// this fork the second arm's lawful consume of the same
    /// `(invocation_id, frame_id)` is misreported as "consumed more than once"
    /// (RT-ESCAPE: e.g. an escaped resource used by a host op whose `Result`
    /// match fans out). Rewinding to the pre-match baseline before each arm
    /// preserves the affine check *within* a single control-flow path — a real
    /// double-consume on one path still collides — and is neither a set-clear nor
    /// a key-salt: it is per-branch scoping. Unioning the arms afterward keeps
    /// every frame consumed on any arm marked consumed for the post-join
    /// continuation, so a genuine revisit *across* the join still rejects.
    fn lower_forked_branch<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        frame_baseline: &std::collections::BTreeSet<(u64, u64)>,
        frame_union: &mut std::collections::BTreeSet<(u64, u64)>,
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        self.consumed_subcontinuation_frames = frame_baseline.clone();
        let lowered = self.lower_source_machine_with_continuation(builder, expr, env, control)?;
        frame_union.extend(self.consumed_subcontinuation_frames.iter().copied());
        Ok(lowered)
    }

    fn lower_source_dynamic_bool_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        condition: cranelift_codegen::ir::Value,
        true_body: &RuntimeExpr,
        false_body: &RuntimeExpr,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    checked_site_id: site_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
            SourcePrefixTerminal::ReturnFromPartition => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion =
                    Some((merge, suffix_pending.to_vec(), required_kind, site_id, None));
                SourceJoinTarget {
                    join_id,
                    checked_site_id: site_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let true_block = builder.create_block();
        let false_block = builder.create_block();
        builder
            .ins()
            .brif(condition, true_block, &[], false_block, &[]);
        let owned_return_eliminators = own_partition_eliminators(&target.terminal_active_prefix);
        let owned_selected_lineage =
            own_partition_selected_lineage(&suffix_control.selected_lineage);
        let predecessor_partition_eligible = self.active_partition_return_kind.is_some()
            && partition_helper_return_kind_is_admissible(target.required_kind)
            && suffix_control.selected.parent.is_none()
            && suffix_control.selected.pending.is_empty()
            && partition_prefix_is_admissible(&source_prefix_template)
            && partition_scope_is_admissible(&suffix_control.selected.selected_scope)
            && env.iter().all(partition_lowered_is_admissible)
            && owned_return_eliminators
                .as_ref()
                .is_some_and(|frames| partition_eliminators_are_admissible(frames))
            && owned_selected_lineage.is_some();
        let partition_site_id = if predecessor_partition_eligible {
            let site_id = self.partition_next_site;
            self.partition_next_site =
                self.partition_next_site.checked_add(1).ok_or_else(|| {
                    unsupported(
                        "NativeFunctionPartition",
                        "Bool predecessor fanout identity exhausted",
                    )
                })?;
            Some(site_id)
        } else {
            None
        };
        let ledger_baseline = self.partition_ledger_baseline();
        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for (predecessor_id, block, body) in [
            (0_u64, true_block, true_body),
            (1_u64, false_block, false_body),
        ] {
            builder.switch_to_block(block);
            if let Some(partition_site_id) = partition_site_id {
                self.call_partition_source_predecessor(
                    builder,
                    partition_site_id,
                    predecessor_id,
                    body.clone(),
                    false,
                    env.to_vec(),
                    &source_prefix_template,
                    suffix_control.partition_cursor,
                    &suffix_control.selected,
                    suffix_control.terminal_outer,
                    &target,
                    owned_selected_lineage
                        .as_deref()
                        .expect("eligibility proves owned selected lineage"),
                    owned_return_eliminators
                        .as_deref()
                        .expect("eligibility proves owned return eliminators"),
                    &ledger_baseline,
                )?;
                continue;
            }
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                partition_cursor: suffix_control.partition_cursor,
                producer_kont: suffix_control.producer_kont,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                body.clone(),
                env.to_vec(),
                branch_control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "Bool predecessor {predecessor_id} did not seal its distinct affine join edge"
                    ),
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_union;
        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            activation_instance: suffix_control.selected.activation_instance,
            cursor: suffix_control.selected.cursor,
            cursor_instance: suffix_control.selected.cursor_instance,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    fn build_partition_cleanup_capture_chain(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        checked_join_site_id: u64,
        eliminators: &[OwnedPartitionEliminator],
    ) -> Result<
        (
            Option<PartitionCleanupSuffixId>,
            Option<cranelift_codegen::ir::Value>,
        ),
        CraneliftBackendError,
    > {
        if eliminators.is_empty() {
            return Ok((None, None));
        }
        if eliminators
            .iter()
            .any(|frame| matches!(frame, OwnedPartitionEliminator::InvocationReturn))
        {
            return Err(unsupported(
                "NativeCleanupStepV1",
                "InvocationReturn must remain the scalar terminal, not a cleanup node",
            ));
        }
        let invocation = self
            .invocation_pointer
            .expect("cleanup capture planning owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let mut successor_id = None;
        let mut successor_pointer = builder.ins().iconst(pointer_type, 0);
        for (terminal_distance, current) in eliminators.iter().rev().enumerate() {
            let mut capture_fields = Vec::new();
            append_partition_eliminator_values(
                self,
                builder,
                std::slice::from_ref(current),
                &mut capture_fields,
            )?;
            capture_fields.push(successor_pointer);
            let capture_field_types = capture_fields
                .iter()
                .map(|value| builder.func.dfg.value_type(*value))
                .collect::<Vec<_>>();
            let suffix_id = self.partition_cleanup_suffixes.intern_step(
                checked_join_site_id,
                terminal_distance,
                current,
                capture_field_types,
                successor_id,
            );
            let frame_size = partition_frame_size(capture_fields.len())?;
            let frame = builder.create_sized_stack_slot(StackSlotData::new(
                StackSlotKind::ExplicitSlot,
                frame_size,
                3,
            ));
            self.partition_metrics
                .record_call_frame(capture_fields.len());
            for (index, value) in capture_fields.iter().copied().enumerate() {
                let byte_offset = index
                    .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                    .and_then(|offset| i32::try_from(offset).ok())
                    .ok_or_else(|| {
                        unsupported(
                            "NativeCleanupStepV1",
                            "cleanup capture-cell offset overflowed",
                        )
                    })?;
                builder.ins().stack_store(value, frame, byte_offset);
            }
            successor_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
            successor_id = Some(suffix_id);
        }
        Ok((successor_id, Some(successor_pointer)))
    }

    #[allow(clippy::too_many_arguments)]
    fn call_partition_source_predecessor<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        partition_site_id: u64,
        edge_index: u64,
        body: RuntimeExpr,
        consume_checked_entry_marker: bool,
        arm_env: Vec<Lowered>,
        _source_prefix_template: &SourcePrefixTemplate,
        source_cursor: Option<PartitionSourceCursor>,
        selected: &SourceSelectedContinuation<'b>,
        terminal_outer: ContinuationCursorId,
        target: &SourceJoinTarget<'b>,
        selected_lineage: &[OwnedSourceSelectedContinuation],
        return_eliminators: &[OwnedPartitionEliminator],
        ledger_baseline: &PartitionLedgerBaseline,
    ) -> Result<(), CraneliftBackendError> {
        let (cleanup_head, cleanup_capture_pointer) = self.build_partition_cleanup_capture_chain(
            builder,
            target.checked_site_id,
            return_eliminators,
        )?;
        let mut fields = Vec::new();
        for value in &arm_env {
            append_partition_lowered_values(self, builder, value, &mut fields)?;
        }
        let env_fields = fields.len();
        if let Some(cursor) = source_cursor {
            fields.push(cursor.capture_pointer);
        }
        if let Some(producer_kont) = self.active_partition_producer_kont {
            fields.push(producer_kont.capture_pointer);
        }
        fields.push(selected.activation_instance.0);
        fields.push(selected.cursor_instance.0);
        let prefix_fields = fields.len().saturating_sub(env_fields);
        append_partition_scope_values(self, builder, &selected.selected_scope, &mut fields)?;
        let selected_pending = own_partition_eliminators(&selected.pending).ok_or_else(|| {
            unsupported(
                "NativeSourceContinuationStepV1",
                "source Eval state selected pending control has no exact schema",
            )
        })?;
        append_partition_eliminator_values(self, builder, &selected_pending, &mut fields)?;
        let scope_fields = fields
            .len()
            .saturating_sub(env_fields)
            .saturating_sub(prefix_fields);
        let lineage_fields = 0;
        self.partition_metrics.record_source_frame_components(
            env_fields,
            prefix_fields,
            scope_fields,
            lineage_fields,
        );
        if let Some(pointer) = cleanup_capture_pointer {
            fields.push(pointer);
        }
        let (frame_values, field_types, field_map) = partition_frame_layout(builder, &fields);
        self.partition_metrics.record_call_frame(frame_values.len());
        let checked_join = self
            .native_join_plan
            .as_ref()
            .and_then(|plan| {
                plan.sites
                    .iter()
                    .find(|site| site.site_id == target.checked_site_id)
            })
            .map(PartitionCheckedJoinIdentity::from)
            .ok_or_else(|| {
                unsupported(
                    "NativeFunctionPartition",
                    "source predecessor has no exact checked join identity",
                )
            })?;
        let key = PartitionSemanticStateKey::SourceArm(PartitionSourceArmKey::new(
            checked_join,
            target.required_kind,
            consume_checked_entry_marker,
            self.pending_computational_ih_call,
            &body,
            &arm_env,
            &self.declaration_stack,
            &self.active_recursive_invocations,
            source_cursor.map(|cursor| cursor.node),
            self.active_partition_producer_kont
                .map(|cursor| cursor.site_id),
            selected.activation,
            selected.cursor,
            &selected.selected_ancestry,
            &selected_pending,
            &selected.selected_scope,
            selected_lineage,
            terminal_outer,
            cleanup_head,
            field_types.clone(),
            field_map.clone(),
        ));
        let return_contract = key.return_contract();
        let existing = self
            .partition_continuations
            .lookup(&key, PartitionAggregateBudget::PRODUCTION)?;
        let (state_id, state, newly_reserved) = if let Some((state_id, state)) = existing {
            (state_id, state, false)
        } else {
            let helper_index = self.partition_next_helper;
            if helper_index >= PartitionAggregateBudget::PRODUCTION.max_helpers {
                return Err(unsupported(
                    "NativeFunctionPartition",
                    "aggregate native partition graph exceeds its helper ceiling",
                ));
            }
            let function = *self.partition_helper_ids.get(helper_index).ok_or_else(|| {
                unsupported(
                    "NativeFunctionPartition",
                    "source predecessor exhausted its predeclared helper pool",
                )
            })?;
            self.partition_next_helper += 1;
            let (state_id, state) = self.partition_continuations.reserve(
                key,
                function,
                helper_index,
                PartitionAggregateBudget::PRODUCTION,
            )?;
            (state_id, state, true)
        };
        self.partition_continuations
            .validate_call_contract(state_id, &return_contract)?;
        if !newly_reserved {
            self.consume_reused_partition_dynamic_splice_edges(&arm_env)?;
        }
        let (_, function_ref) = self.partition_helper_ref(
            builder,
            state.helper_index,
            "interned source predecessor lost its helper identity",
        )?;
        let frame_size = partition_frame_size(frame_values.len())?;
        let frame = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            frame_size,
            3,
        ));
        for (index, value) in frame_values.iter().copied().enumerate() {
            let byte_offset = index
                .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                .and_then(|offset| i32::try_from(offset).ok())
                .ok_or_else(|| {
                    unsupported(
                        "NativeFunctionPartition",
                        "private source-predecessor frame offset overflowed",
                    )
                })?;
            builder.ins().stack_store(value, frame, byte_offset);
        }
        let invocation = self
            .invocation_pointer
            .expect("source partition owns an invocation pointer");
        let pointer_type = builder.func.dfg.value_type(invocation);
        let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
        let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            PARTITION_FRAME_FIELD_BYTES,
            3,
        ));
        let zero_tag = builder.ins().iconst(types::I64, 0);
        builder.ins().stack_store(zero_tag, tag_output, 0);
        let tag_output_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
        let call = builder.ins().call(
            function_ref,
            &[invocation, frame_pointer, tag_output_pointer],
        );
        let edge_authority = self.mint_partition_branch_return(
            partition_site_id,
            edge_index,
            state.helper_index,
            return_contract.required_kind,
        )?;
        self.consume_partition_branch_return(
            edge_authority,
            state.helper_index,
            return_contract.required_kind,
        )?;
        let result_tag = builder.ins().stack_load(types::I64, tag_output, 0);
        let result_payload = builder.inst_results(call)[0];
        builder
            .ins()
            .jump(target.block, &[result_tag.into(), result_payload.into()]);
        if newly_reserved {
            self.partition_queue.push_back(PartitionWorkItem::SourceArm(
                SourceArmPartitionWorkItem {
                    state_id,
                    function: state.function,
                    field_types,
                    field_map,
                    body,
                    consume_checked_entry_marker,
                    pending_computational_ih_call: self.pending_computational_ih_call,
                    env: arm_env,
                    declaration_stack: self.declaration_stack.clone(),
                    active_recursive_invocations: self.active_recursive_invocations.clone(),
                    source_head: source_cursor.map(|cursor| cursor.node),
                    source_capture_pointer: source_cursor.map(|cursor| cursor.capture_pointer),
                    producer_kont: self.active_partition_producer_kont,
                    selected_activation: selected.activation,
                    selected_activation_instance: selected.activation_instance,
                    selected_cursor: selected.cursor,
                    selected_cursor_instance: selected.cursor_instance,
                    selected_ancestry: selected.selected_ancestry.clone(),
                    selected_pending,
                    selected_scope: selected.selected_scope.clone(),
                    selected_lineage: selected_lineage.to_vec(),
                    terminal_outer,
                    cleanup_head,
                    cleanup_capture_pointer,
                    ledger_baseline: ledger_baseline.clone(),
                    return_contract,
                },
            ));
        }
        Ok(())
    }

    fn consume_reused_partition_dynamic_splice_edges(
        &mut self,
        values: &[Lowered],
    ) -> Result<(), CraneliftBackendError> {
        fn collect(value: &Lowered, output: &mut BTreeSet<DynamicSpliceEdgeId>) {
            match value {
                Lowered::HostResult { error, ok, .. } => {
                    collect(error, output);
                    collect(ok, output);
                }
                Lowered::DynamicConstructor(dynamic) => {
                    for field in dynamic
                        .alternatives
                        .iter()
                        .flat_map(|alternative| &alternative.fields)
                    {
                        collect(field, output);
                    }
                }
                Lowered::Constructor { args, .. } => {
                    for value in args {
                        collect(value, output);
                    }
                }
                Lowered::Record { fields } => {
                    for (_, value) in fields {
                        collect(value, output);
                    }
                }
                Lowered::Closure { captures, .. }
                | Lowered::DeclarationClosure { captures, .. } => {
                    for value in captures {
                        collect(value, output);
                    }
                }
                Lowered::ComputationalRecursorClosure {
                    residual,
                    invocation,
                    ..
                } => {
                    output.extend(invocation.dynamic_splice_edges.iter().copied());
                    collect(residual, output);
                    for value in &invocation.selection.outer_env {
                        collect(value, output);
                    }
                    for layer in &invocation.unwind.later_wrappers_in_construction_order {
                        for value in &layer.outer_env {
                            collect(value, output);
                        }
                    }
                }
                Lowered::Int { .. }
                | Lowered::Bool { .. }
                | Lowered::ProcessExitStatus { .. }
                | Lowered::CapabilityToken { .. }
                | Lowered::ResourceToken { .. }
                | Lowered::BoundedNat(_)
                | Lowered::StructuralNat(_)
                | Lowered::ResponseBytes { .. }
                | Lowered::Bytes(_)
                | Lowered::BorrowedNativeValue { .. }
                | Lowered::BorrowedOption { .. }
                | Lowered::String(_)
                | Lowered::Trap(_)
                | Lowered::RecursiveBackedge => {}
            }
        }

        let mut edge_ids = BTreeSet::new();
        for value in values {
            collect(value, &mut edge_ids);
        }
        for edge_id in edge_ids {
            let Some(edge) = self.dynamic_splice_edges.remove(&edge_id) else {
                // Lowered recursor carriers are cloneable inert handles. A
                // sibling clone may already have consumed the unique ledger
                // entry; only a still-live entry transfers to this call edge.
                continue;
            };
            if edge.edge_id != edge_id {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "reused partition call consumed a stale dynamic splice edge identity",
                ));
            }
        }
        Ok(())
    }

    fn lower_source_checked_template_predecessor<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        checked_expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) = Self::split_source_prefix(control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = control.selected.as_active(&control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    checked_site_id: site_id,
                    block: merge,
                    expected_outer: control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
            SourcePrefixTerminal::ReturnFromPartition => {
                let active = control.selected.as_active(&control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((merge, suffix_pending.to_vec(), required_kind, None));
                SourceJoinTarget {
                    join_id,
                    checked_site_id: site_id,
                    block: merge,
                    expected_outer: control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
        };
        let return_eliminators = own_partition_eliminators(&target.terminal_active_prefix)
            .filter(|frames| partition_eliminators_are_admissible(frames))
            .ok_or_else(|| {
                unsupported(
                    "NativeFunctionPartition",
                    "checked-template predecessor has a non-admissible planned scalar prefix",
                )
            })?;
        let selected_lineage = own_partition_selected_lineage(&control.selected_lineage)
            .ok_or_else(|| {
                unsupported(
                    "NativeFunctionPartition",
                    "checked-template predecessor has a non-admissible selected lineage",
                )
            })?;
        if !partition_helper_return_kind_is_admissible(target.required_kind)
            || control.selected.parent.is_some()
            || !control.selected.pending.is_empty()
            || !partition_prefix_is_admissible(&source_prefix_template)
            || !partition_scope_is_admissible(&control.selected.selected_scope)
            || !env.iter().all(partition_lowered_is_admissible)
        {
            return Err(unsupported(
                "NativeFunctionPartition",
                "checked-template predecessor reached a scalar join but its control/frame schema is not admissible",
            ));
        }
        let partition_site_id = self.partition_next_site;
        self.partition_next_site = self.partition_next_site.checked_add(1).ok_or_else(|| {
            unsupported(
                "NativeFunctionPartition",
                "checked-template predecessor identity exhausted",
            )
        })?;
        let ledger_baseline = self.partition_ledger_baseline();
        self.call_partition_source_predecessor(
            builder,
            partition_site_id,
            0,
            checked_expr,
            true,
            env,
            &source_prefix_template,
            control.partition_cursor,
            &control.selected,
            control.terminal_outer,
            &target,
            &selected_lineage,
            &return_eliminators,
            &ledger_baseline,
        )?;
        let Some((merge, suffix_pending, required_kind, root_authority)) = local_completion else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: control.selected.activation,
            activation_instance: control.selected.activation_instance,
            cursor: control.selected.cursor,
            cursor_instance: control.selected.cursor_instance,
            parent: control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &control.selected.selected_ancestry,
            source_lineage: &control.selected_lineage,
            source_selected_cursor: Some(control.selected.cursor),
            selected_scope: control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_dynamic_host_result_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        default: RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let mut local_completion = None;
        let target = match terminal {
            SourcePrefixTerminal::Join(inherited_edge) => inherited_edge.target,
            SourcePrefixTerminal::ResumeOuter { root_authority } => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion = Some((
                    merge,
                    suffix_pending.to_vec(),
                    required_kind,
                    site_id,
                    root_authority,
                ));
                SourceJoinTarget {
                    join_id,
                    checked_site_id: site_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
            SourcePrefixTerminal::ReturnFromPartition => {
                let active = suffix_control
                    .selected
                    .as_active(&suffix_control.selected_lineage);
                let (prefix, suffix_pending, required_kind, site_id) =
                    self.planned_active_scalar_cut(active)?;
                let join_id = self.next_source_join;
                self.next_source_join = self
                    .next_source_join
                    .checked_add(1)
                    .expect("compiler-private source join identity exhausted");
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                local_completion =
                    Some((merge, suffix_pending.to_vec(), required_kind, site_id, None));
                SourceJoinTarget {
                    join_id,
                    checked_site_id: site_id,
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
        };

        let owned_return_eliminators = own_partition_eliminators(&target.terminal_active_prefix);
        let owned_selected_lineage =
            own_partition_selected_lineage(&suffix_control.selected_lineage);
        let source_arm_partition_eligible = (self.partition_cut_armed
            || self.active_partition_return_kind.is_some())
            && partition_helper_return_kind_is_admissible(target.required_kind)
            && suffix_control.selected.parent.is_none()
            && suffix_control.selected.pending.is_empty()
            && partition_prefix_is_admissible(&source_prefix_template)
            && partition_scope_is_admissible(&suffix_control.selected.selected_scope)
            && env.iter().all(partition_lowered_is_admissible)
            && partition_lowered_is_admissible(&ok)
            && partition_lowered_is_admissible(&error)
            && owned_return_eliminators
                .as_ref()
                .is_some_and(|frames| partition_eliminators_are_admissible(frames))
            && owned_selected_lineage.is_some();
        if source_arm_partition_eligible {
            let return_eliminators =
                owned_return_eliminators.expect("eligibility proves owned return eliminators");
            let selected_lineage =
                owned_selected_lineage.expect("eligibility proves owned selected lineage");
            let partition_site_id = self.partition_next_site;
            self.partition_next_site =
                self.partition_next_site.checked_add(1).ok_or_else(|| {
                    unsupported(
                        "NativeFunctionPartition",
                        "source partition fanout identity exhausted",
                    )
                })?;
            let ledger_baseline = self.partition_ledger_baseline();
            let ok_block = builder.create_block();
            let err_block = builder.create_block();
            builder.ins().brif(success, ok_block, &[], err_block, &[]);
            for (edge_index, block, constructor, payload) in [
                (0_u64, ok_block, ok_constructor, ok),
                (1_u64, err_block, err_constructor, error),
            ] {
                builder.switch_to_block(block);
                let Some(case) = cases
                    .iter()
                    .find(|case| case.constructor == constructor && case.binders == 1)
                else {
                    // The dynamic test and its fail-closed default stay in the
                    // caller. Only a valid predecessor may cross the reusable
                    // state boundary; otherwise outlining would move trap
                    // identity into a helper and conceal malformed control.
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    continue;
                };
                let body = case.body.clone();
                let mut arm_env = vec![payload];
                arm_env.extend_from_slice(env);
                self.call_partition_source_predecessor(
                    builder,
                    partition_site_id,
                    edge_index,
                    body,
                    false,
                    arm_env,
                    &source_prefix_template,
                    suffix_control.partition_cursor,
                    &suffix_control.selected,
                    suffix_control.terminal_outer,
                    &target,
                    &selected_lineage,
                    &return_eliminators,
                    &ledger_baseline,
                )?;
            }
            self.partition_cut_armed = false;
            let Some((merge, suffix_pending, required_kind, site_id, root_authority)) =
                local_completion
            else {
                return Ok(Lowered::RecursiveBackedge);
            };
            builder.switch_to_block(merge);
            let merged = self.lowered_from_scalar_pair(
                required_kind,
                NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                },
            );
            if !suffix_pending.is_empty() {
                if let Some(pending) =
                    own_partition_eliminators(&suffix_pending).filter(|pending| {
                        partition_eliminators_are_admissible(pending)
                            && partition_lowered_is_admissible(&merged)
                    })
                {
                    let mut fields = Vec::new();
                    append_partition_lowered_values(self, builder, &merged, &mut fields)?;
                    fields.push(suffix_control.selected.activation_instance.0);
                    fields.push(suffix_control.selected.cursor_instance.0);
                    append_partition_eliminator_values(self, builder, &pending, &mut fields)?;
                    append_partition_scope_values(
                        self,
                        builder,
                        &suffix_control.selected.selected_scope,
                        &mut fields,
                    )?;
                    // The occurrence map records structural aliasing without
                    // admitting caller-local SSA identities into the state key.
                    let (frame_values, field_types, field_map) =
                        partition_frame_layout(builder, &fields);
                    self.partition_metrics.record_call_frame(frame_values.len());
                    let checked_join = self
                        .native_join_plan
                        .as_ref()
                        .and_then(|plan| plan.sites.iter().find(|site| site.site_id == site_id))
                        .map(PartitionCheckedJoinIdentity::from)
                        .ok_or_else(|| {
                            unsupported(
                                "NativeFunctionPartition",
                                "resume continuation has no exact checked join identity",
                            )
                        })?;
                    let key =
                        PartitionSemanticStateKey::ProducerKont(PartitionContinuationKey::new(
                            checked_join.clone(),
                            required_kind,
                            ScalarMergeKind::ExitCode,
                            &merged,
                            suffix_control.selected.activation,
                            suffix_control.selected.cursor,
                            &pending,
                            &suffix_control.selected.selected_ancestry,
                            &suffix_control.selected.selected_scope,
                            &selected_lineage,
                            None,
                            field_types.clone(),
                            field_map.clone(),
                        ));
                    let existing = self
                        .partition_continuations
                        .lookup(&key, PartitionAggregateBudget::PRODUCTION)?;
                    let (state_id, state, newly_reserved) =
                        if let Some((state_id, state)) = existing {
                            (state_id, state, false)
                        } else {
                            let helper_index = self.partition_next_helper;
                            let function =
                                *self.partition_helper_ids.get(helper_index).ok_or_else(|| {
                                    unsupported(
                                        "NativeFunctionPartition",
                                        "resume continuation exhausted its predeclared helper pool",
                                    )
                                })?;
                            self.partition_next_helper =
                                self.partition_next_helper.checked_add(1).ok_or_else(|| {
                                    unsupported(
                                        "NativeFunctionPartition",
                                        "partition helper identity exhausted",
                                    )
                                })?;
                            let (state_id, state) = self.partition_continuations.reserve(
                                key,
                                function,
                                helper_index,
                                PartitionAggregateBudget::PRODUCTION,
                            )?;
                            (state_id, state, true)
                        };
                    let (_, function_ref) = self.partition_helper_ref(
                        builder,
                        state.helper_index,
                        "interned resume continuation lost its helper identity",
                    )?;
                    let frame_size = partition_frame_size(frame_values.len())?;
                    let frame = builder.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        frame_size,
                        3,
                    ));
                    for (index, value) in frame_values.iter().copied().enumerate() {
                        let byte_offset = index
                            .checked_mul(PARTITION_FRAME_FIELD_BYTES as usize)
                            .and_then(|offset| i32::try_from(offset).ok())
                            .ok_or_else(|| {
                                unsupported(
                                    "NativeFunctionPartition",
                                    "private resume frame offset overflowed",
                                )
                            })?;
                        builder.ins().stack_store(value, frame, byte_offset);
                    }
                    let invocation = self
                        .invocation_pointer
                        .expect("resume partition owns an invocation pointer");
                    let pointer_type = builder.func.dfg.value_type(invocation);
                    let frame_pointer = builder.ins().stack_addr(pointer_type, frame, 0);
                    let tag_output = builder.create_sized_stack_slot(StackSlotData::new(
                        StackSlotKind::ExplicitSlot,
                        PARTITION_FRAME_FIELD_BYTES,
                        3,
                    ));
                    let zero_tag = builder.ins().iconst(types::I64, 0);
                    builder.ins().stack_store(zero_tag, tag_output, 0);
                    let tag_output_pointer = builder.ins().stack_addr(pointer_type, tag_output, 0);
                    let call = builder.ins().call(
                        function_ref,
                        &[invocation, frame_pointer, tag_output_pointer],
                    );
                    let result_tag = builder.ins().stack_load(types::I64, tag_output, 0);
                    let result_payload = builder.inst_results(call)[0];
                    if newly_reserved {
                        self.partition_queue
                            .push_back(PartitionWorkItem::ProducerKont(
                                ProducerKontPartitionWorkItem {
                                    state_id,
                                    site_id: usize::MAX,
                                    function: state.function,
                                    field_types,
                                    field_map,
                                    value: merged,
                                    action: ProducerKontAction::ApplyActiveEliminators {
                                        activation: suffix_control.selected.activation,
                                        activation_instance: suffix_control
                                            .selected
                                            .activation_instance,
                                        cursor: suffix_control.selected.cursor,
                                        cursor_instance: suffix_control.selected.cursor_instance,
                                        pending,
                                        selected_ancestry: suffix_control
                                            .selected
                                            .selected_ancestry
                                            .clone(),
                                        selected_scope: suffix_control
                                            .selected
                                            .selected_scope
                                            .clone(),
                                        selected_lineage,
                                        capture_field_types: Vec::new(),
                                    },
                                    capture_pointer: None,
                                    successor: None,
                                    ledger_baseline,
                                    declaration_stack: self.declaration_stack.clone(),
                                    active_recursive_invocations: self
                                        .active_recursive_invocations
                                        .clone(),
                                    checked_join,
                                    return_kind: ScalarMergeKind::ExitCode,
                                },
                            ));
                    }
                    self.restore_root_terminal_authority(
                        root_authority,
                        suffix_control.terminal_outer,
                    )?;
                    return Ok(self.lowered_from_scalar_pair(
                        ScalarMergeKind::ExitCode,
                        NativeScalarPairV1 {
                            tag: result_tag,
                            payload: result_payload,
                        },
                    ));
                }
            }
            let suffix_active = ActiveContinuationFrame {
                activation: suffix_control.selected.activation,
                activation_instance: suffix_control.selected.activation_instance,
                cursor: suffix_control.selected.cursor,
                cursor_instance: suffix_control.selected.cursor_instance,
                parent: suffix_control.selected.parent,
                pending: &suffix_pending,
                selected_ancestry: &suffix_control.selected.selected_ancestry,
                source_lineage: &suffix_control.selected_lineage,
                source_selected_cursor: Some(suffix_control.selected.cursor),
                selected_scope: suffix_control.selected.selected_scope.as_ref(),
            };
            self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
            return self.resume_active_continuation(builder, merged, suffix_active);
        }
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        builder.ins().brif(success, ok_block, &[], err_block, &[]);

        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for (predecessor_id, block, constructor, payload) in [
            (0, ok_block, ok_constructor, ok),
            (1, err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                partition_cursor: suffix_control.partition_cursor,
                producer_kont: suffix_control.producer_kont,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = if let Some(case) = cases
                .iter()
                .find(|case| case.constructor == constructor && case.binders == 1)
            {
                let mut arm_env = vec![payload];
                arm_env.extend_from_slice(env);
                self.lower_forked_branch(
                    builder,
                    &frame_baseline,
                    &mut frame_union,
                    case.body.clone(),
                    arm_env,
                    branch_control,
                )?
            } else {
                self.lower_forked_branch(
                    builder,
                    &frame_baseline,
                    &mut frame_union,
                    RuntimeExpr::Trap(default.clone()),
                    env.to_vec(),
                    branch_control,
                )?
            };
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "HostResult predecessor {predecessor_id} did not seal its distinct affine join edge"
                    ),
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_union;

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(Lowered::RecursiveBackedge);
        };
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            activation_instance: suffix_control.selected.activation_instance,
            cursor: suffix_control.selected.cursor,
            cursor_instance: suffix_control.selected.cursor_instance,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    fn lower_source_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;
        if Self::source_terminal_join(&suffix_control.continuation).is_some() {
            return self.lower_source_nested_dynamic_constructor_match(
                builder,
                dynamic,
                cases,
                default,
                env,
                suffix_control,
            );
        }
        self.lower_source_planned_dynamic_constructor_match(
            builder,
            dynamic,
            cases,
            default,
            env,
            suffix_control,
        )
    }

    fn lower_source_nested_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let SourcePrefixTerminal::Join(inherited_edge) = terminal else {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "nested dynamic constructor has no affine terminal edge",
            ));
        };
        let fanout = SourceBranchFanout {
            source_prefix_template,
            inherited_edge,
        };
        let target = fanout.inherited_edge.target;
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor source match block");
        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for alternative in dynamic.alternatives {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let case = match select_dynamic_constructor_case(cases, &alternative, default)? {
                Ok(case) => case,
                Err(_) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&fanout.source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                partition_cursor: suffix_control.partition_cursor,
                producer_kont: suffix_control.producer_kont,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                case.body.clone(),
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "nested dynamic constructor predecessor did not seal its edge",
                ));
            }
            test_block = next;
        }
        self.consumed_subcontinuation_frames = frame_union;
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        Ok(Lowered::RecursiveBackedge)
    }

    fn lower_source_planned_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        env: &[Lowered],
        suffix_control: SourceControl<'b>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let active = suffix_control
            .selected
            .as_active(&suffix_control.selected_lineage);
        let (prefix, suffix_pending, required_kind, site_id) =
            self.planned_active_scalar_cut(active)?;
        let suffix_pending = suffix_pending.to_vec();
        let join_id = self.next_source_join;
        self.next_source_join = self
            .next_source_join
            .checked_add(1)
            .expect("compiler-private source join identity exhausted");
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let target = SourceJoinTarget {
            join_id,
            checked_site_id: site_id,
            block: merge,
            expected_outer: suffix_control.terminal_outer,
            required_kind,
            terminal_active_prefix: prefix,
        };
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let root_authority = match terminal {
            SourcePrefixTerminal::ResumeOuter { root_authority } => root_authority,
            SourcePrefixTerminal::ReturnFromPartition => None,
            SourcePrefixTerminal::Join(_) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "planned dynamic-constructor cut unexpectedly inherited an executable edge",
                ));
            }
        };
        let owned_return_eliminators = own_partition_eliminators(&target.terminal_active_prefix);
        let owned_selected_lineage =
            own_partition_selected_lineage(&suffix_control.selected_lineage);
        let predecessor_partition_eligible = self.active_partition_return_kind.is_some()
            && partition_helper_return_kind_is_admissible(target.required_kind)
            && suffix_control.selected.parent.is_none()
            && suffix_control.selected.pending.is_empty()
            && partition_prefix_is_admissible(&source_prefix_template)
            && partition_scope_is_admissible(&suffix_control.selected.selected_scope)
            && env.iter().all(partition_lowered_is_admissible)
            && dynamic
                .alternatives
                .iter()
                .flat_map(|alternative| &alternative.fields)
                .all(partition_lowered_is_admissible)
            && owned_return_eliminators
                .as_ref()
                .is_some_and(|frames| partition_eliminators_are_admissible(frames))
            && owned_selected_lineage.is_some();
        let partition_site_id = if predecessor_partition_eligible {
            let site_id = self.partition_next_site;
            self.partition_next_site =
                self.partition_next_site.checked_add(1).ok_or_else(|| {
                    unsupported(
                        "NativeFunctionPartition",
                        "dynamic predecessor fanout identity exhausted",
                    )
                })?;
            Some(site_id)
        } else {
            None
        };
        let ledger_baseline = self.partition_ledger_baseline();
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor source match block");
        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for (predecessor_id, alternative) in dynamic.alternatives.into_iter().enumerate() {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let case = match select_dynamic_constructor_case(cases, &alternative, default)? {
                Ok(case) => case,
                Err(_) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let arm_env = materialize_dynamic_constructor_env(&alternative, env);
            if let Some(partition_site_id) = partition_site_id {
                self.call_partition_source_predecessor(
                    builder,
                    partition_site_id,
                    predecessor_id as u64,
                    case.body.clone(),
                    false,
                    arm_env,
                    &source_prefix_template,
                    suffix_control.partition_cursor,
                    &suffix_control.selected,
                    suffix_control.terminal_outer,
                    &target,
                    owned_selected_lineage
                        .as_deref()
                        .expect("eligibility proves owned selected lineage"),
                    owned_return_eliminators
                        .as_deref()
                        .expect("eligibility proves owned return eliminators"),
                    &ledger_baseline,
                )?;
                test_block = next;
                continue;
            }
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                partition_cursor: suffix_control.partition_cursor,
                producer_kont: suffix_control.producer_kont,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                case.body.clone(),
                arm_env,
                control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, Lowered::RecursiveBackedge) {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    format!(
                        "dynamic-constructor predecessor {predecessor_id} for checked site {site_id} did not seal its affine join edge"
                    ),
                ));
            }
            test_block = next;
        }
        self.consumed_subcontinuation_frames = frame_union;
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        builder.switch_to_block(merge);
        let merged = self.lowered_from_scalar_pair(
            required_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(merge)[0],
                payload: builder.block_params(merge)[1],
            },
        );
        let suffix_active = ActiveContinuationFrame {
            activation: suffix_control.selected.activation,
            activation_instance: suffix_control.selected.activation_instance,
            cursor: suffix_control.selected.cursor,
            cursor_instance: suffix_control.selected.cursor_instance,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, merged, suffix_active)
    }

    fn source_call_state<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        callee: Lowered,
        args: Vec<Lowered>,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        match callee {
            Lowered::Closure {
                captures,
                params,
                body,
            } => {
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                let mut call_env = args;
                call_env.extend(captures);
                call_env.extend(env);
                Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                    expr: body,
                    env: call_env,
                    control,
                }))
            }
            Lowered::DeclarationClosure {
                symbol,
                captures,
                params,
                body,
            } => {
                if params.len() != args.len() {
                    return Err(unsupported(
                        "Call",
                        format!(
                            "closure expects {} args but call provides {}",
                            params.len(),
                            args.len()
                        ),
                    ));
                }
                self.lower_source_declaration_call(
                    builder, symbol, captures, body, args, env, control,
                )
            }
            mut recursor @ Lowered::ComputationalRecursorClosure { .. } => {
                let checked_ih_invocation =
                    self.mint_checked_computational_ih_instance(builder, &mut recursor)?;
                if let Some(CheckedRecursiveInvocationInstance {
                    source: InvocationTemplateRef::ComputationalIHCall(call_template_id),
                    ..
                }) = checked_ih_invocation
                {
                    let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation has no oriented plan",
                        )
                    })?;
                    let call = plan
                        .computational_ih_call(call_template_id)
                        .ok_or_else(|| {
                            unsupported(
                                "OrientedSubcontinuationPlanV1",
                                "checked IH invocation has no exact call template",
                            )
                        })?;
                    let open = control.selected.selected_scope.as_ref().ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation has no selected/open parent occurrence",
                        )
                    })?;
                    self.validate_source_dynamic_splice_parent(
                        checked_ih_invocation.expect("matched checked IH invocation"),
                        open,
                    )?;
                    if call.parent_frame_template_id != open.frame.checked_frame_id
                        || call.parent_segment_site_id
                            != open.frame.checked_frame_id.and_then(|frame_id| {
                                plan.frame(frame_id).map(|frame| frame.segment_site_id)
                            })
                    {
                        return Err(unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "checked IH invocation parent edge does not match the active open occurrence",
                        ));
                    }
                }
                let (base, boundary) = decompose_computational_recursor(recursor);
                let (activation, invocation) =
                    boundary.expect("recursor closure carries an invocation segment");
                if source_active_cursor(
                    &control.selected,
                    &control.selected_lineage,
                    invocation.resume_cursor,
                )
                .is_none()
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "recursive invocation cursor is not live in source control",
                    ));
                }
                let armed = ArmedInvocation {
                    suspended: control,
                    expected_selected: invocation.resume_cursor,
                    expected_selected_instance: invocation.resume_cursor_instance,
                };
                if source_active_cursor(
                    &armed.suspended.selected,
                    &armed.suspended.selected_lineage,
                    armed.expected_selected,
                )
                .is_none()
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "armed invocation endpoint changed selected cursor",
                    ));
                }
                if let Lowered::BoundedNat(predecessor) = base {
                    if !args.is_empty() {
                        return Err(unsupported(
                            "BoundedNat",
                            "structural Nat recursive hypothesis takes no arguments",
                        ));
                    }
                    let mut suspended = self.install_source_recursor_invocation(
                        builder,
                        armed.suspended,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    if self.active_partition_return_contract.is_some() {
                        self.push_partition_source_prefix(builder, &mut suspended)?;
                    }
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: Lowered::BoundedNat(predecessor),
                        control: suspended,
                    }));
                } else {
                    let Lowered::Closure {
                        captures,
                        params,
                        body,
                    } = base
                    else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "recursive constructor field is not a closure",
                        ));
                    };
                    if params.len() != args.len() {
                        return Err(unsupported(
                            "ComputationalMatch",
                            format!(
                                "recursive field expects {} args but call provides {}",
                                params.len(),
                                args.len()
                            ),
                        ));
                    }
                    let mut call_env = args;
                    call_env.extend(captures);
                    call_env.extend(env);
                    let mut suspended = self.install_source_recursor_invocation(
                        builder,
                        armed.suspended,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    if self.active_partition_return_contract.is_some() {
                        self.push_partition_source_prefix(builder, &mut suspended)?;
                    }
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                        expr: body,
                        env: call_env,
                        control: suspended,
                    }));
                }
            }
            _ => Err(unsupported("Call", "callee is not a closure")),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_declaration_call<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: RuntimeSymbol,
        captures: Vec<Lowered>,
        body: RuntimeExpr,
        args: Vec<Lowered>,
        env: Vec<Lowered>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(&symbol)?;
        if !self.declaration_is_recursive(&symbol) {
            let mut call_env = args;
            call_env.extend(captures);
            call_env.extend(env);
            return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                expr: body,
                env: call_env,
                control,
            }));
        }

        if let Some(active) = self
            .active_recursive_declarations
            .iter()
            .rev()
            .find(|active| active.symbol == symbol)
            .cloned()
        {
            if !same_recursive_argument_shapes(&active.argument_templates, &args) {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "recursive declaration {symbol} changes its native argument representation: {:?} -> {:?}",
                        active
                            .argument_templates
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>(),
                        args.iter().map(lowered_value_kind).collect::<Vec<_>>()
                    ),
                ));
            }
            if let Some(induction) = active.induction {
                return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                    value: induction,
                    control,
                }));
            }
            let mut values = Vec::new();
            append_recursive_argument_values(builder, &args, &mut values, &self.native_int_tags)?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive source declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(SourceCallOutcome::Complete(Lowered::RecursiveBackedge));
        }

        let header = builder.create_block();
        let mut initial_values = Vec::new();
        append_recursive_argument_values(
            builder,
            &args,
            &mut initial_values,
            &self.native_int_tags,
        )?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
        builder.ins().jump(
            header,
            &initial_values
                .iter()
                .copied()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        builder.switch_to_block(header);

        let mut parameters = builder.block_params(header).iter().copied();
        let mut loop_args = Vec::with_capacity(args.len());
        for template in &args {
            loop_args.push(rebuild_recursive_argument(
                template,
                &mut parameters,
                &mut self.native_int_tags,
            )?);
        }
        if parameters.next().is_some() {
            return Err(unsupported(
                "DeclarationRef",
                "recursive source declaration loop parameter shape is not closed",
            ));
        }
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: Some(header),
                argument_templates: args,
                induction: None,
            });
        let mut call_env = loop_args.into_iter().rev().collect::<Vec<_>>();
        call_env.extend(captures);
        call_env.extend(env);
        let lowered = self.lower_source_machine_with_continuation(builder, body, call_env, control);
        self.active_recursive_declarations.pop();
        Ok(SourceCallOutcome::Complete(lowered?))
    }

    fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: &RuntimeExpr,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        self.check_partition_live_growth(builder)?;
        match expr {
            RuntimeExpr::Value(value) => self.lower_value(builder, value),
            RuntimeExpr::CheckedJoinSite { site_id, body } => {
                if self.active_join_site.replace(*site_id).is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "nested checked join occurrence marker",
                    ));
                }
                let result = self.lower_expr(builder, body, env);
                if self.active_join_site.take().is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence marker was not consumed",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let result = self.lower_expr(builder, body, env);
                if self.active_subcontinuation_frame.take().is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "checked subcontinuation marker was not consumed by its frame",
                    ));
                }
                result
            }
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id,
                body,
                ..
            } => {
                let instance = self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let result = self.lower_expr(builder, body, env);
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                self.lower_expr(builder, body, env)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let value = self.lower_expr(builder, body, env)?;
                self.finish_checked_computational_ih_marker(builder, value)
            }
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| unsupported("Var", format!("no runtime binding for index {index}"))),
            RuntimeExpr::PrimitiveCall { primitive, args } => {
                self.lower_primitive_call(builder, primitive, args, env)
            }
            RuntimeExpr::Let { value, body } => {
                let lowered_value = self.lower_expr(builder, value, env)?;
                if matches!(lowered_value, Lowered::RecursiveBackedge) {
                    return Ok(Lowered::RecursiveBackedge);
                }
                if let Lowered::Trap(trap) = lowered_value {
                    return Ok(Lowered::Trap(trap));
                }
                let mut body_env = vec![lowered_value];
                body_env.extend_from_slice(env);
                self.lower_expr(builder, body, &body_env)
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                if matches!(lowered_scrutinee, Lowered::RecursiveBackedge) {
                    return Ok(Lowered::RecursiveBackedge);
                }
                let Lowered::Bool { value, known } = lowered_scrutinee else {
                    return Err(unsupported(
                        "If",
                        "branch lowering requires a Bool scrutinee",
                    ));
                };
                if let Some(scrutinee) = known {
                    return if scrutinee {
                        self.lower_expr(builder, then_expr, env)
                    } else {
                        self.lower_expr(builder, else_expr, env)
                    };
                }
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = builder.create_block();
                builder.append_block_param(merge, types::I64);
                builder.append_block_param(merge, types::I64);
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                for (block, arm) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_expr(builder, arm, env)?;
                    let Lowered::Int { value, known } = lowered else {
                        return Err(unsupported(
                            "If",
                            "dynamic native If arms must produce scalar Int values",
                        ));
                    };
                    let tag = self.native_int_tag(builder, value, known)?;
                    builder.ins().jump(merge, &[tag.into(), value.into()]);
                }
                builder.switch_to_block(merge);
                let tag = builder.block_params(merge)[0];
                let value = builder.block_params(merge)[1];
                self.native_int_tags.insert(value, tag);
                Ok(Lowered::Int { value, known: None })
            }
            RuntimeExpr::Construct { constructor, args } => {
                let lowered_args = args
                    .iter()
                    .map(|arg| self.lower_expr(builder, arg, env))
                    .collect::<Result<Vec<_>, _>>()?;
                if lowered_args
                    .iter()
                    .any(|arg| matches!(arg, Lowered::RecursiveBackedge))
                {
                    return Ok(Lowered::RecursiveBackedge);
                }
                if lowered_args.is_empty()
                    && (constructor == &self.process_symbols.bool_true
                        || constructor == &self.process_symbols.bool_false)
                {
                    let known = constructor == &self.process_symbols.bool_true;
                    return Ok(Lowered::Bool {
                        value: builder.ins().iconst(types::I64, i64::from(known)),
                        known: Some(known),
                    });
                }
                if constructor == &self.process_symbols.nat_zero && lowered_args.is_empty() {
                    return Ok(Lowered::StructuralNat(StructuralNatV1 {
                        value: builder.ins().iconst(types::I64, 0),
                    }));
                }
                if constructor == &self.process_symbols.nat_suc {
                    if let [Lowered::StructuralNat(predecessor)] = lowered_args.as_slice() {
                        return Ok(Lowered::StructuralNat(StructuralNatV1 {
                            value: builder.ins().iadd_imm(predecessor.value, 1),
                        }));
                    }
                }
                Ok(Lowered::Constructor {
                    constructor: constructor.clone(),
                    args: lowered_args,
                })
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                if requires_heterogeneous_deforestation(scrutinee)
                    || self.declaration_call_produces_deforestable_aggregate(scrutinee)
                {
                    return self.lower_computational_producer_expr(
                        builder,
                        scrutinee,
                        env,
                        &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                            cases,
                            default,
                            env,
                            retained_scrutinee_index: None,
                            deferred_constructor_case: None,
                        })],
                    );
                }
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                if let Lowered::BorrowedNativeValue { pointer } = lowered_scrutinee {
                    return self.lower_borrowed_match(builder, pointer, cases, default, env);
                }
                if let Lowered::BorrowedOption {
                    present,
                    value,
                    none,
                    some,
                } = lowered_scrutinee
                {
                    return self.lower_borrowed_option_match(
                        builder, present, value, &none, &some, cases, default, env,
                    );
                }
                if let Lowered::BoundedNat(nat) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(builder, nat, false, cases, default, env);
                }
                if let Lowered::StructuralNat(nat) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        cases,
                        default,
                        env,
                    );
                }
                if let Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                } = lowered_scrutinee
                {
                    return self.lower_dynamic_host_result_match(
                        builder,
                        success,
                        *error,
                        *ok,
                        &err_constructor,
                        &ok_constructor,
                        cases,
                        env,
                    );
                }
                if let Lowered::DynamicConstructor(dynamic) = lowered_scrutinee {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Ordinary {
                            cases,
                            default,
                            env,
                        },
                    );
                }
                if let Lowered::Bool { value, known } = lowered_scrutinee {
                    let true_case = cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = cases.iter().find(|case| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "Match",
                            "Bool match requires zero-binder True and False cases",
                        ));
                    };
                    if let Some(selected) = known {
                        return self.lower_expr(
                            builder,
                            if selected {
                                &true_case.body
                            } else {
                                &false_case.body
                            },
                            env,
                        );
                    }
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder.append_block_param(merge, types::I64);
                    builder.ins().brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    for (block, case) in [(true_block, true_case), (false_block, false_case)] {
                        builder.switch_to_block(block);
                        let lowered = self.lower_expr(builder, &case.body, env)?;
                        let (value, branch_kind) =
                            self.merge_scalar_branch(builder, lowered, "Match")?;
                        Self::record_scalar_merge_kind("Match", &mut merge_kind, branch_kind)?;
                        builder
                            .ins()
                            .jump(merge, &[value.tag.into(), value.payload.into()]);
                    }
                    builder.switch_to_block(merge);
                    let pair = NativeScalarPairV1 {
                        tag: builder.block_params(merge)[0],
                        payload: builder.block_params(merge)[1],
                    };
                    return Ok(self.lowered_from_scalar_pair(
                        merge_kind.expect("Bool match emits both closed alternatives"),
                        pair,
                    ));
                }
                let Lowered::Constructor { constructor, args } = lowered_scrutinee else {
                    return Err(unsupported("Match", "scrutinee is not a constructor value"));
                };
                let Some(case) = cases.iter().find(|case| case.constructor == constructor) else {
                    return Ok(Lowered::Trap(default.clone()));
                };
                if case.binders != args.len() {
                    return Err(unsupported(
                        "Match",
                        format!(
                            "case {} expects {} binders but constructor has {} args",
                            case.constructor,
                            case.binders,
                            args.len()
                        ),
                    ));
                }
                let mut case_env = args;
                case_env.extend_from_slice(env);
                self.lower_expr(builder, &case.body, &case_env)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee,
                cases,
                default,
            } => self.lower_computational_match_expr(builder, scrutinee, cases, default, env, env),
            RuntimeExpr::Record { fields } => {
                let lowered_fields = fields
                    .iter()
                    .map(|(name, expr)| Ok((name.clone(), self.lower_expr(builder, expr, env)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                Ok(Lowered::Record {
                    fields: lowered_fields,
                })
            }
            RuntimeExpr::Project { record, field } => {
                let lowered_record = self.lower_expr(builder, record, env)?;
                let Lowered::Record { fields } = lowered_record else {
                    return Err(unsupported(
                        "Project",
                        "record projection needs a record value",
                    ));
                };
                fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *field).then_some(value))
                    .ok_or_else(|| unsupported("Project", format!("missing field {field}")))
            }
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let lowered_captures = captures
                    .iter()
                    .map(|symbol| self.lower_seed_capture(builder, symbol))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Lowered::Closure {
                    captures: lowered_captures,
                    params: params.clone(),
                    body: (**body).clone(),
                })
            }
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                let captures = captures
                    .iter()
                    .map(|capture| self.lower_expr(builder, capture, env))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Lowered::Closure {
                    captures,
                    params: params.clone(),
                    body: (**body).clone(),
                })
            }
            RuntimeExpr::DeclarationRef { symbol } => self.lower_declaration_ref(builder, symbol),
            RuntimeExpr::ImportedDeclarationRef {
                symbol,
                dependency,
                dependency_semantic_hash,
            } => Err(unsupported(
                "ImportedDeclarationRef",
                format!(
                    "imported declaration {symbol} from {dependency} @ {dependency_semantic_hash} requires dependency linking"
                ),
            )),
            RuntimeExpr::Call { callee, args } => {
                let lowered_callee = self.lower_expr(builder, callee, env)?;
                match lowered_callee {
                    Lowered::DeclarationClosure {
                        symbol,
                        captures,
                        params,
                        body,
                    } => self.lower_recursive_declaration_call(
                        builder, &symbol, &captures, &params, &body, args, env, None,
                    ),
                    Lowered::Closure {
                        captures,
                        params,
                        body,
                    } => {
                        if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                            if let Some((cases, default)) =
                                ordinary_match_continuation(&params, &body)
                            {
                                let mut frame_env = captures;
                                frame_env.extend_from_slice(env);
                                return self.lower_computational_producer_expr(
                                    builder,
                                    &args[0],
                                    env,
                                    &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                        cases,
                                        default,
                                        env: &frame_env,
                                        retained_scrutinee_index: Some(0),
                                        deferred_constructor_case: None,
                                    })],
                                );
                            }
                        }
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, env))
                            .collect::<Result<Vec<_>, _>>()?;
                        if params.len() != call_env.len() {
                            return Err(unsupported(
                                "Call",
                                format!(
                                    "closure expects {} args but call provides {}",
                                    params.len(),
                                    call_env.len()
                                ),
                            ));
                        }
                        call_env.extend(captures);
                        call_env.extend_from_slice(env);
                        self.lower_expr(builder, &body, &call_env)
                    }
                    mut callee @ Lowered::ComputationalRecursorClosure { .. } => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(builder, &mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(callee);
                        let (activation, invocation) =
                            boundary.expect("recursor closure carries an invocation segment");
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let mut frames = installed_oriented_eliminator_frames(&installed);
                        frames.push(EliminatorFrame::InvocationReturn);
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &frames,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
                        }
                        let Lowered::Closure {
                            captures,
                            params,
                            body,
                        } = base
                        else {
                            return Err(unsupported(
                                "ComputationalMatch",
                                "recursive constructor field is not a closure",
                            ));
                        };
                        let mut call_env = args
                            .iter()
                            .map(|arg| self.lower_expr(builder, arg, env))
                            .collect::<Result<Vec<_>, _>>()?;
                        if params.len() != call_env.len() {
                            return Err(unsupported(
                                "ComputationalMatch",
                                format!(
                                    "recursive field expects {} args but call provides {}",
                                    params.len(),
                                    call_env.len()
                                ),
                            ));
                        }
                        call_env.extend(captures);
                        call_env.extend_from_slice(env);
                        self.enter_oriented_semantic_region(installed.checked);
                        let result = self
                            .lower_computational_producer_expr(builder, &body, &call_env, &frames);
                        self.leave_oriented_semantic_region(installed.checked);
                        result
                    }
                    _ => Err(unsupported("Call", "callee is not a closure")),
                }
            }
            RuntimeExpr::Trap(trap) => Ok(Lowered::Trap(trap.clone())),
            RuntimeExpr::Effect {
                family,
                operation,
                capability,
                args,
            } if self.process_object => self.lower_process_host_effect(
                builder,
                family,
                *operation,
                capability.as_ref(),
                args,
                env,
            ),
            RuntimeExpr::Effect {
                family, operation, ..
            } => Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is not modeled in the supported native subset",
                    *operation as u16
                ),
            )),
        }
    }

    fn lower_process_host_effect(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        family: &RuntimeSymbol,
        operation: ken_host::HostOpV1,
        capability: Option<&crate::RuntimeCapabilityUse>,
        args: &[RuntimeExpr],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        if !CRANELIFT_HOST_EFFECT_CONSUMERS_V1.contains(&operation) {
            return Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is a represented unavailable lane",
                    operation as u16
                ),
            ));
        }
        let lowered = args
            .iter()
            .map(|argument| self.lower_expr(builder, argument, env))
            .collect::<Result<Vec<_>, _>>()?;
        let pointer_type = builder.func.dfg.value_type(
            self.invocation_pointer
                .expect("process effect lowering owns an invocation pointer"),
        );
        let wire = ken_host::host_effect_wire_layout_v1(operation).map_err(|error| {
            unsupported(
                "Effect",
                format!("generated HostEffectAbiV1 layout rejected: {error:?}"),
            )
        })?;
        let request_offset = |index: usize| {
            i32::try_from(wire.request_offsets[index])
                .expect("C-probed request offset was checked as u32")
        };
        let request = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            wire.request_size,
            wire.request_align_shift,
        ));
        let mut narrow_failure: Option<(
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        )> = None;
        let mut positioned_bounds: Option<(
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        )> = None;
        let mut record_narrow_failure =
            |builder: &mut FunctionBuilder<'_>, invalid, detail: i64| {
                let detail = builder.ins().iconst(types::I64, detail);
                narrow_failure = Some(match narrow_failure.take() {
                    Some((prior_invalid, prior_detail)) => (
                        builder.ins().bor(prior_invalid, invalid),
                        builder.ins().select(prior_invalid, prior_detail, detail),
                    ),
                    None => (invalid, detail),
                });
            };
        match operation {
            ken_host::HostOpV1::ConsoleWrite
            | ken_host::HostOpV1::ConsoleFlush
            | ken_host::HostOpV1::ConsoleIsTerminal => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "ambient Console carried a capability",
                    ));
                }
                let stream = lowered
                    .first()
                    .and_then(console_stream_tag)
                    .ok_or_else(|| {
                        unsupported("Effect", "Console operation has a malformed Stream operand")
                    })?;
                let stream = builder.ins().iconst(types::I64, stream);
                builder
                    .ins()
                    .stack_store(stream, request, request_offset(0));
                if operation == ken_host::HostOpV1::ConsoleWrite {
                    let (data, len) = self.wire_bytes(
                        builder,
                        lowered.get(1).ok_or_else(|| {
                            unsupported("Effect", "Console.Write is missing Bytes")
                        })?,
                    )?;
                    builder.ins().stack_store(data, request, request_offset(1));
                    builder.ins().stack_store(len, request, request_offset(2));
                }
            }
            ken_host::HostOpV1::FsReadFile
            | ken_host::HostOpV1::FsWriteFile
            | ken_host::HostOpV1::FsChangeMode
            | ken_host::HostOpV1::FsOpen => {
                let capability = capability
                    .ok_or_else(|| unsupported("Effect", "FS operation has no live capability"))?;
                let Lowered::CapabilityToken { value: token } =
                    self.lower_expr(builder, &capability.value, env)?
                else {
                    return Err(unsupported(
                        "Effect",
                        "FS capability operand is not the opaque invocation token",
                    ));
                };
                builder.ins().stack_store(token, request, request_offset(0));
                let (path, path_len) = self.wire_bytes(
                    builder,
                    lowered
                        .first()
                        .ok_or_else(|| unsupported("Effect", "FS operation is missing its path"))?,
                )?;
                builder.ins().stack_store(path, request, request_offset(1));
                builder
                    .ins()
                    .stack_store(path_len, request, request_offset(2));
                if operation == ken_host::HostOpV1::FsWriteFile {
                    let policy = lowered.get(1).and_then(create_policy_tag).ok_or_else(|| {
                        unsupported("Effect", "FS.WriteFile has a malformed CreatePolicy")
                    })?;
                    let (bytes, bytes_len) = self.wire_bytes(
                        builder,
                        lowered.get(2).ok_or_else(|| {
                            unsupported("Effect", "FS.WriteFile is missing contents")
                        })?,
                    )?;
                    let policy = builder.ins().iconst(types::I64, policy);
                    builder
                        .ins()
                        .stack_store(policy, request, request_offset(3));
                    builder.ins().stack_store(bytes, request, request_offset(4));
                    builder
                        .ins()
                        .stack_store(bytes_len, request, request_offset(5));
                } else if operation == ken_host::HostOpV1::FsChangeMode {
                    let mode = lowered.get(1).ok_or_else(|| {
                        unsupported("Effect", "FS.ChangeMode is missing its mode")
                    })?;
                    let (mode, valid_int) = self.narrow_native_int_u64(builder, mode)?;
                    let in_range = builder.ins().icmp_imm(
                        cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
                        mode,
                        0o7777,
                    );
                    let in_range = builder.ins().band(valid_int, in_range);
                    let narrowed = builder.ins().ireduce(types::I16, mode);
                    let invalid = builder.ins().iconst(types::I16, 0xffff);
                    let mode = builder.ins().select(in_range, narrowed, invalid);
                    builder.ins().stack_store(mode, request, request_offset(3));
                } else if operation == ken_host::HostOpV1::FsOpen {
                    let mode =
                        lowered
                            .get(1)
                            .and_then(resource_open_mode_tag)
                            .ok_or_else(|| {
                                unsupported("Effect", "FS.Open has a malformed ResourceOpenMode")
                            })?;
                    let mode = builder.ins().iconst(types::I64, mode);
                    builder.ins().stack_store(mode, request, request_offset(3));
                }
            }
            ken_host::HostOpV1::FsHandleMetadata | ken_host::HostOpV1::ResourceRelease => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "resource operation carried a capability",
                    ));
                }
                let Lowered::ResourceToken { value: token } = lowered.first().ok_or_else(|| {
                    unsupported("Effect", "resource operation is missing its token")
                })?
                else {
                    return Err(unsupported(
                        "Effect",
                        "resource operand is not an opaque resource token",
                    ));
                };
                builder
                    .ins()
                    .stack_store(*token, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferAllocate => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "buffer allocation carried a capability",
                    ));
                }
                let capacity = lowered.first().ok_or_else(|| {
                    unsupported("Effect", "BufferAllocate is missing its capacity")
                })?;
                let (capacity, valid) = self.narrow_native_int_u64(builder, capacity)?;
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                record_narrow_failure(builder, invalid, 7);
                builder
                    .ins()
                    .stack_store(capacity, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferFreeze => {
                if capability.is_some() {
                    return Err(unsupported("Effect", "BufferFreeze carried a capability"));
                }
                let Lowered::ResourceToken { value: token } = lowered
                    .first()
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its buffer"))?
                else {
                    return Err(unsupported(
                        "Effect",
                        "BufferFreeze buffer is not a resource",
                    ));
                };
                let start = lowered
                    .get(1)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its start"))?;
                let length = lowered
                    .get(2)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its length"))?;
                let (start, start_valid) = self.narrow_native_int_u64(builder, start)?;
                let (length, length_valid) = self.narrow_native_int_u64(builder, length)?;
                let valid = builder.ins().band(start_valid, length_valid);
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                record_narrow_failure(builder, invalid, 7);
                // PX8-SPAN-PROV: trailing `span_origin` acquisition token.
                let Lowered::ResourceToken { value: span_origin } =
                    lowered.get(3).ok_or_else(|| {
                        unsupported("Effect", "BufferFreeze is missing its span origin")
                    })?
                else {
                    return Err(unsupported(
                        "Effect",
                        "BufferFreeze span origin is not a resource",
                    ));
                };
                for (index, value) in [*token, start, length, *span_origin]
                    .into_iter()
                    .enumerate()
                {
                    builder
                        .ins()
                        .stack_store(value, request, request_offset(index));
                }
            }
            ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "positioned resource operation carried a capability",
                    ));
                }
                let resource = |index: usize, name: &str| {
                    let Some(Lowered::ResourceToken { value }) = lowered.get(index) else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not a resource"),
                        ));
                    };
                    Ok(*value)
                };
                let integer = |index: usize, name: &str| {
                    let Some(value @ Lowered::Int { .. }) = lowered.get(index) else {
                        return Err(unsupported(
                            "Effect",
                            format!("positioned {name} operand is not Int"),
                        ));
                    };
                    Ok(value)
                };
                let file = resource(0, "file")?;
                let (file_offset, file_offset_valid) =
                    self.narrow_native_int_u64(builder, integer(1, "file offset")?)?;
                let buffer = resource(2, "buffer")?;
                let (buffer_start, buffer_start_valid) =
                    self.narrow_native_int_u64(builder, integer(3, "buffer start")?)?;
                let (length, length_valid) =
                    self.narrow_native_int_u64(builder, integer(4, "length")?)?;
                positioned_bounds = Some((buffer_start, length));
                let file_offset_invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    file_offset_valid,
                    0,
                );
                record_narrow_failure(builder, file_offset_invalid, 6);
                let bounds_valid = builder.ins().band(buffer_start_valid, length_valid);
                let bounds_invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    bounds_valid,
                    0,
                );
                record_narrow_failure(builder, bounds_invalid, 7);
                if operation == ken_host::HostOpV1::FsWriteAt {
                    // PX8-SPAN-PROV: `FsWriteAt` carries the trailing
                    // `span_origin` acquisition token; `FsReadAt` mints the span
                    // and has no origin operand.
                    let span_origin = resource(5, "span origin")?;
                    for (index, value) in
                        [file, buffer, file_offset, buffer_start, length, span_origin]
                            .into_iter()
                            .enumerate()
                    {
                        builder
                            .ins()
                            .stack_store(value, request, request_offset(index));
                    }
                } else {
                    for (index, value) in [file, buffer, file_offset, buffer_start, length]
                        .into_iter()
                        .enumerate()
                    {
                        builder
                            .ins()
                            .stack_store(value, request, request_offset(index));
                    }
                }
            }
            _ => unreachable!("availability was checked above"),
        }
        let reply = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            wire.reply_size,
            wire.reply_align_shift,
        ));
        let invocation = self
            .invocation_pointer
            .expect("process effect lowering owns an invocation pointer");
        let op = builder.ins().iconst(types::I64, operation as i64);
        let request_pointer = builder.ins().stack_addr(pointer_type, request, 0);
        let request_size = builder
            .ins()
            .iconst(types::I64, i64::from(wire.request_size));
        let reply_pointer = builder.ins().stack_addr(pointer_type, reply, 0);
        if let Some((invalid, detail)) = narrow_failure {
            let dispatch = builder.create_block();
            let synthesize = builder.create_block();
            let decoded = builder.create_block();
            builder.ins().brif(invalid, synthesize, &[], dispatch, &[]);

            builder.switch_to_block(dispatch);
            let call = builder.ins().call(
                self.host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[invocation, op, request_pointer, request_size, reply_pointer],
            );
            let status = builder.inst_results(call)[0];
            Self::require_i64(builder, status, 0);
            builder.ins().jump(decoded, &[]);

            builder.switch_to_block(synthesize);
            let zero = builder.ins().iconst(types::I64, 0);
            for offset in [
                wire.reply_resource_error_schema_offset,
                wire.reply_resource_error_kind_offset,
                wire.reply_resource_error_identity_offset,
                wire.reply_resource_error_io_offset,
                wire.reply_resource_error_required_offset,
                wire.reply_resource_error_held_offset,
                wire.reply_resource_error_expected_kind_offset,
                wire.reply_resource_error_actual_kind_offset,
                wire.reply_bytes_data_offset,
                wire.reply_bytes_len_offset,
                wire.reply_effective_request_offset,
            ] {
                builder.ins().stack_store(
                    zero,
                    reply,
                    i32::try_from(offset).expect("reply field offset is u32"),
                );
            }
            let resource_error_tag = builder
                .ins()
                .iconst(types::I64, wire.reply_resource_error_tag as i64);
            builder.ins().stack_store(
                resource_error_tag,
                reply,
                i32::try_from(wire.reply_tag_offset).expect("reply tag offset is u32"),
            );
            builder.ins().stack_store(
                detail,
                reply,
                i32::try_from(wire.reply_detail_offset).expect("reply detail offset is u32"),
            );
            builder.ins().jump(decoded, &[]);
            builder.switch_to_block(decoded);
        } else {
            let call = builder.ins().call(
                self.host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[invocation, op, request_pointer, request_size, reply_pointer],
            );
            let status = builder.inst_results(call)[0];
            Self::require_i64(builder, status, 0);
        }
        let tag = builder.ins().stack_load(
            types::I64,
            reply,
            i32::try_from(wire.reply_tag_offset).expect("reply tag offset is u32"),
        );
        let detail = builder.ins().stack_load(
            types::I64,
            reply,
            i32::try_from(wire.reply_detail_offset).expect("reply detail offset is u32"),
        );
        self.partition_cut_armed |= self
            .partition_budget
            .should_partition(PartitionFunctionMeasure::from_function(builder.func));
        if operation == ken_host::HostOpV1::ConsoleIsTerminal {
            Self::require_i64(builder, tag, wire.reply_bool_tag as i64);
            Ok(Lowered::Bool {
                value: detail,
                known: None,
            })
        } else {
            let success_tag = match operation {
                ken_host::HostOpV1::FsReadFile => wire.reply_bytes_tag,
                ken_host::HostOpV1::FsOpen => wire.reply_resource_tag,
                ken_host::HostOpV1::FsHandleMetadata => wire.reply_metadata_tag,
                ken_host::HostOpV1::BufferAllocate => wire.reply_resource_tag,
                ken_host::HostOpV1::BufferFreeze => wire.reply_bytes_tag,
                ken_host::HostOpV1::FsReadAt => wire.reply_read_progress_tag,
                ken_host::HostOpV1::FsWriteAt => wire.reply_write_progress_tag,
                _ => wire.reply_unit_tag,
            } as i64;
            let accepted_tags = match operation {
                ken_host::HostOpV1::FsHandleMetadata => vec![
                    success_tag,
                    wire.reply_error_tag as i64,
                    wire.reply_resource_error_tag as i64,
                ],
                ken_host::HostOpV1::ResourceRelease => {
                    vec![success_tag, wire.reply_resource_error_tag as i64]
                }
                ken_host::HostOpV1::BufferAllocate | ken_host::HostOpV1::BufferFreeze => {
                    vec![success_tag, wire.reply_resource_error_tag as i64]
                }
                ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => vec![
                    success_tag,
                    wire.reply_error_tag as i64,
                    wire.reply_resource_error_tag as i64,
                ],
                _ => vec![success_tag, wire.reply_error_tag as i64],
            };
            Self::require_one_of_i64(builder, tag, &accepted_tags);
            let resource_schema = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_schema_offset)
                    .expect("resource error schema offset is u32"),
            );
            let resource_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_kind_offset)
                    .expect("resource error kind offset is u32"),
            );
            let resource_identity = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_identity_offset)
                    .expect("resource error identity offset is u32"),
            );
            let resource_io = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_io_offset)
                    .expect("resource error io offset is u32"),
            );
            let resource_required = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_required_offset)
                    .expect("resource error required offset is u32"),
            );
            let resource_held = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_held_offset)
                    .expect("resource error held offset is u32"),
            );
            let resource_expected_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_expected_kind_offset)
                    .expect("resource error expected-kind offset is u32"),
            );
            let resource_actual_kind = builder.ins().stack_load(
                types::I64,
                reply,
                i32::try_from(wire.reply_resource_error_actual_kind_offset)
                    .expect("resource error actual-kind offset is u32"),
            );
            Self::validate_resource_error_reply(
                builder,
                tag,
                wire.reply_resource_error_tag,
                detail,
                resource_schema,
                resource_kind,
                resource_identity,
                resource_io,
                resource_required,
                resource_held,
                resource_expected_kind,
                resource_actual_kind,
                wire.resource_error_reply_schema,
                wire.resource_kind_fs_handle,
                wire.resource_kind_buffer,
            );
            let payload = builder.ins().sshr_imm(detail, 32);
            let payload_int = self.lower_dynamic_small_int(builder, payload);
            let last = self.process_symbols.io_errors.len().saturating_sub(1);
            let io_error = Lowered::DynamicConstructor(DynamicConstructorV1 {
                discriminator: builder.ins().band_imm(detail, 0xff),
                alternatives: self
                    .process_symbols
                    .io_errors
                    .iter()
                    .enumerate()
                    .map(|(tag, constructor)| DynamicConstructorAlternativeV1 {
                        tag: tag as i64,
                        constructor: constructor.clone(),
                        fields: (tag == last)
                            .then(|| vec![payload_int.clone()])
                            .unwrap_or_default(),
                    })
                    .collect(),
            });
            let error = if matches!(
                operation,
                ken_host::HostOpV1::FsReadFile
                    | ken_host::HostOpV1::FsWriteFile
                    | ken_host::HostOpV1::FsChangeMode
                    | ken_host::HostOpV1::FsOpen
            ) {
                let path = lowered
                    .first()
                    .cloned()
                    .expect("validated FS operation has a path");
                Lowered::Constructor {
                    constructor: self.process_symbols.file_error.clone(),
                    args: vec![
                        Lowered::Constructor {
                            constructor: match operation {
                                ken_host::HostOpV1::FsReadFile => {
                                    self.process_symbols.file_operation_read.clone()
                                }
                                ken_host::HostOpV1::FsWriteFile => {
                                    self.process_symbols.file_operation_write.clone()
                                }
                                ken_host::HostOpV1::FsChangeMode => {
                                    self.process_symbols.file_operation_change_mode.clone()
                                }
                                ken_host::HostOpV1::FsOpen => {
                                    self.process_symbols.file_operation_read.clone()
                                }
                                _ => unreachable!("validated FS result operation"),
                            },
                            args: Vec::new(),
                        },
                        Lowered::Constructor {
                            constructor: self.process_symbols.option_some.clone(),
                            args: vec![path],
                        },
                        io_error,
                    ],
                }
            } else if matches!(
                operation,
                ken_host::HostOpV1::FsHandleMetadata
                    | ken_host::HostOpV1::ResourceRelease
                    | ken_host::HostOpV1::BufferAllocate
                    | ken_host::HostOpV1::BufferFreeze
                    | ken_host::HostOpV1::FsReadAt
                    | ken_host::HostOpV1::FsWriteAt
            ) {
                let generic = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    tag,
                    wire.reply_error_tag as i64,
                );
                let zero = builder.ins().iconst(types::I64, 0);
                let resource_surface_tag = builder.ins().iadd_imm(detail, 1);
                let surface_tag = builder.ins().select(generic, zero, resource_surface_tag);
                let surface_io = builder.ins().select(generic, detail, resource_io);
                let surface_io_payload = builder.ins().sshr_imm(surface_io, 32);
                let surface_io_payload_int =
                    self.lower_dynamic_small_int(builder, surface_io_payload);
                let resource_required_int =
                    self.lower_unsigned_u64_int(builder, resource_required)?;
                let resource_held_int = self.lower_unsigned_u64_int(builder, resource_held)?;
                let surface_io_error = Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: builder.ins().band_imm(surface_io, 0xff),
                    alternatives: self
                        .process_symbols
                        .io_errors
                        .iter()
                        .enumerate()
                        .map(|(tag, constructor)| DynamicConstructorAlternativeV1 {
                            tag: tag as i64,
                            constructor: constructor.clone(),
                            fields: (tag == last)
                                .then(|| vec![surface_io_payload_int.clone()])
                                .unwrap_or_default(),
                        })
                        .collect(),
                });
                let identity_low = builder.ins().band_imm(resource_identity, 0xffff_ffff);
                let identity_high = builder.ins().ushr_imm(resource_identity, 32);
                let identity_low_int = self.lower_dynamic_small_int(builder, identity_low);
                let identity_high_int = self.lower_dynamic_small_int(builder, identity_high);
                let resource_kind_value = |discriminator| {
                    Lowered::DynamicConstructor(DynamicConstructorV1 {
                        discriminator,
                        alternatives: vec![
                            DynamicConstructorAlternativeV1 {
                                tag: wire.resource_kind_fs_handle as i64,
                                constructor: self.process_symbols.resource_kind_fs_handle.clone(),
                                fields: Vec::new(),
                            },
                            DynamicConstructorAlternativeV1 {
                                tag: wire.resource_kind_buffer as i64,
                                constructor: self.process_symbols.resource_kind_buffer.clone(),
                                fields: Vec::new(),
                            },
                        ],
                    })
                };
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: surface_tag,
                    alternatives: vec![
                        DynamicConstructorAlternativeV1 {
                            tag: 0,
                            constructor: self.process_symbols.resource_host_io.clone(),
                            fields: vec![surface_io_error.clone()],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 1,
                            constructor: self.process_symbols.resource_closed.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 2,
                            constructor: self.process_symbols.resource_malformed.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 3,
                            constructor: self.process_symbols.resource_right_not_held.clone(),
                            fields: vec![resource_required_int, resource_held_int],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 4,
                            constructor: self.process_symbols.resource_release_failed.clone(),
                            fields: vec![
                                resource_kind_value(resource_kind),
                                Lowered::Constructor {
                                    constructor: self
                                        .process_symbols
                                        .resource_trace_identity
                                        .clone(),
                                    args: vec![identity_low_int, identity_high_int],
                                },
                                surface_io_error,
                            ],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 5,
                            constructor: self.process_symbols.resource_kind_mismatch.clone(),
                            fields: vec![
                                resource_kind_value(resource_expected_kind),
                                resource_kind_value(resource_actual_kind),
                            ],
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 6,
                            constructor: self.process_symbols.resource_buffer_limit.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 7,
                            constructor: self.process_symbols.resource_invalid_offset.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 8,
                            constructor: self.process_symbols.resource_invalid_bounds.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 9,
                            constructor: self.process_symbols.resource_no_progress.clone(),
                            fields: Vec::new(),
                        },
                    ],
                })
            } else {
                io_error
            };
            let success = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                success_tag,
            );
            let ok = if operation == ken_host::HostOpV1::FsReadFile {
                Lowered::ResponseBytes {
                    pointer: builder.ins().stack_load(
                        pointer_type,
                        reply,
                        i32::try_from(wire.reply_bytes_data_offset)
                            .expect("reply bytes data offset is u32"),
                    ),
                    len: builder.ins().stack_load(
                        types::I64,
                        reply,
                        i32::try_from(wire.reply_bytes_len_offset)
                            .expect("reply bytes len offset is u32"),
                    ),
                }
            } else if operation == ken_host::HostOpV1::FsOpen {
                Lowered::ResourceToken { value: detail }
            } else if operation == ken_host::HostOpV1::BufferAllocate {
                Lowered::ResourceToken { value: detail }
            } else if operation == ken_host::HostOpV1::BufferFreeze {
                Lowered::ResponseBytes {
                    pointer: builder.ins().stack_load(
                        pointer_type,
                        reply,
                        i32::try_from(wire.reply_bytes_data_offset)
                            .expect("reply bytes data offset is u32"),
                    ),
                    len: builder.ins().stack_load(
                        types::I64,
                        reply,
                        i32::try_from(wire.reply_bytes_len_offset)
                            .expect("reply bytes len offset is u32"),
                    ),
                }
            } else if operation == ken_host::HostOpV1::FsReadAt {
                let reply_data = builder.ins().stack_load(
                    pointer_type,
                    reply,
                    i32::try_from(wire.reply_bytes_data_offset)
                        .expect("reply bytes data offset is u32"),
                );
                let reply_start = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_bytes_len_offset)
                        .expect("reply bytes len offset is u32"),
                );
                let nonzero = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    detail,
                    0,
                );
                let read_some = builder.ins().band(success, nonzero);
                let zero = builder.ins().iconst(types::I64, 0);
                let eof_data = builder.ins().icmp(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    reply_data,
                    zero,
                );
                let eof_start = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    reply_start,
                    0,
                );
                let eof_valid = builder.ins().band(eof_data, eof_start);
                let is_zero = builder.ins().bnot(nonzero);
                let read_eof = builder.ins().band(success, is_zero);
                Self::require_when(builder, read_eof, eof_valid);
                Self::require_when(builder, read_some, eof_data);
                let (request_start, request_length) = positioned_bounds
                    .expect("positioned request bounds were narrowed before dispatch");
                let effective_request = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_effective_request_offset)
                        .expect("reply effective request offset is u32"),
                );
                let (count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    read_some,
                    detail,
                    request_start,
                    request_length,
                    effective_request,
                    Some(reply_start),
                );
                let reply_start_int = self.lower_unsigned_u64_int(builder, reply_start)?;
                // PX8-SPAN-PROV: bind the minted span to this `readAt`'s buffer
                // operand acquisition (lowered arg 2, the request seat).
                let Lowered::ResourceToken { value: span_origin } =
                    lowered.get(2).ok_or_else(|| {
                        unsupported("Effect", "FsReadAt is missing its buffer operand")
                    })?
                else {
                    return Err(unsupported(
                        "Effect",
                        "FsReadAt buffer operand is not a resource",
                    ));
                };
                let span_origin = *span_origin;
                let span = Lowered::Constructor {
                    constructor: self.process_symbols.private_buffer_span.clone(),
                    args: vec![
                        Lowered::ResourceToken { value: span_origin },
                        reply_start_int,
                        Lowered::BoundedNat(count),
                    ],
                };
                let transferred = Lowered::Constructor {
                    constructor: self.process_symbols.private_transfer_count.clone(),
                    args: vec![
                        Lowered::BoundedNat(predecessor),
                        Lowered::BoundedNat(remaining),
                    ],
                };
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: builder.ins().uextend(types::I64, nonzero),
                    alternatives: vec![
                        DynamicConstructorAlternativeV1 {
                            tag: 0,
                            constructor: self.process_symbols.read_eof.clone(),
                            fields: Vec::new(),
                        },
                        DynamicConstructorAlternativeV1 {
                            tag: 1,
                            constructor: self.process_symbols.read_some.clone(),
                            fields: vec![span, transferred],
                        },
                    ],
                })
            } else if operation == ken_host::HostOpV1::FsWriteAt {
                let (request_start, request_length) = positioned_bounds
                    .expect("positioned request bounds were narrowed before dispatch");
                let effective_request = builder.ins().stack_load(
                    types::I64,
                    reply,
                    i32::try_from(wire.reply_effective_request_offset)
                        .expect("reply effective request offset is u32"),
                );
                let (_count, predecessor, remaining) = Self::mint_validated_progress_nat(
                    builder,
                    success,
                    detail,
                    request_start,
                    request_length,
                    effective_request,
                    None,
                );
                Lowered::Constructor {
                    constructor: self.process_symbols.wrote.clone(),
                    args: vec![Lowered::Constructor {
                        constructor: self.process_symbols.private_transfer_count.clone(),
                        args: vec![
                            Lowered::BoundedNat(predecessor),
                            Lowered::BoundedNat(remaining),
                        ],
                    }],
                }
            } else if operation == ken_host::HostOpV1::FsHandleMetadata {
                self.lower_unsigned_u64_int(builder, detail)?
            } else {
                Lowered::Constructor {
                    constructor: self.process_symbols.unit.clone(),
                    args: Vec::new(),
                }
            };
            Ok(Lowered::HostResult {
                success,
                error: Box::new(error),
                ok: Box::new(ok),
                err_constructor: self.process_symbols.result_err.clone(),
                ok_constructor: self.process_symbols.result_ok.clone(),
            })
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_unary_recursive_nat_fold(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        argument: Lowered,
        zero_body: &RuntimeExpr,
        suc_body: &RuntimeExpr,
        producer_env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let (target, structural) = match argument {
            Lowered::StructuralNat(nat) => (nat.value, true),
            Lowered::BoundedNat(nat) => (nat.value, false),
            _ => {
                return Err(unsupported(
                    "DeclarationRef",
                    "unary Nat recursion received a non-Nat representation",
                ));
            }
        };
        let zero = builder.ins().iconst(types::I64, 0);
        let zero_nat = if structural {
            Lowered::StructuralNat(StructuralNatV1 { value: zero })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(zero))
        };
        let mut zero_env = vec![zero_nat];
        zero_env.extend_from_slice(captures);
        zero_env.extend_from_slice(producer_env);
        let zero_lowered = self.lower_expr(builder, zero_body, &zero_env)?;
        let (initial, result_kind) =
            self.merge_scalar_branch(builder, zero_lowered, "DeclarationRef")?;
        if result_kind == ScalarMergeKind::RecursiveBackedge {
            return Err(unsupported(
                "DeclarationRef",
                "unary Nat recursion has no finite base result",
            ));
        }

        let loop_block = builder.create_block();
        let step_block = builder.create_block();
        let done_block = builder.create_block();
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(loop_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder.append_block_param(done_block, types::I64);
        builder.ins().jump(
            loop_block,
            &[zero.into(), initial.tag.into(), initial.payload.into()],
        );
        builder.switch_to_block(loop_block);
        let predecessor_value = builder.block_params(loop_block)[0];
        let induction = NativeScalarPairV1 {
            tag: builder.block_params(loop_block)[1],
            payload: builder.block_params(loop_block)[2],
        };
        let complete = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            predecessor_value,
            target,
        );
        builder.ins().brif(
            complete,
            done_block,
            &[induction.tag.into(), induction.payload.into()],
            step_block,
            &[],
        );

        builder.switch_to_block(step_block);
        let successor_value = builder.ins().iadd_imm(predecessor_value, 1);
        let predecessor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: predecessor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(predecessor_value))
        };
        let successor = if structural {
            Lowered::StructuralNat(StructuralNatV1 {
                value: successor_value,
            })
        } else {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(successor_value))
        };
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: None,
                argument_templates: vec![predecessor.clone()],
                induction: Some(induction),
            });
        // A Suc case sees its predecessor first, followed by the retained
        // scrutinee and the declaration's outer environment.
        let mut suc_env = vec![predecessor, successor];
        suc_env.extend_from_slice(captures);
        suc_env.extend_from_slice(producer_env);
        let next = self.lower_expr(builder, suc_body, &suc_env);
        self.active_recursive_declarations.pop();
        let (next, next_kind) = self.merge_scalar_branch(builder, next?, "DeclarationRef")?;
        if next_kind != result_kind {
            return Err(unsupported(
                "DeclarationRef",
                "unary Nat recursion changes its native result representation",
            ));
        }
        builder.ins().jump(
            loop_block,
            &[successor_value.into(), next.tag.into(), next.payload.into()],
        );
        builder.switch_to_block(done_block);
        Ok(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        ))
    }

    fn lower_recursive_declaration_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        params: &[String],
        body: &RuntimeExpr,
        args: &[RuntimeExpr],
        producer_env: &[Lowered],
        eliminators: Option<&[EliminatorFrame<'_>]>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(symbol)?;
        let lowered_args = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, producer_env))
            .collect::<Result<Vec<_>, _>>()?;
        if params.len() != lowered_args.len() {
            return Err(unsupported(
                "DeclarationRef",
                format!(
                    "recursive declaration {symbol} expects {} args but call provides {}",
                    params.len(),
                    lowered_args.len()
                ),
            ));
        }

        if let Some(active) = self
            .active_recursive_declarations
            .iter()
            .rev()
            .find(|active| active.symbol == *symbol)
            .cloned()
        {
            if !same_recursive_argument_shapes(&active.argument_templates, &lowered_args) {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "recursive declaration {symbol} changes its native argument representation: {:?} -> {:?}",
                        active
                            .argument_templates
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>(),
                        lowered_args
                            .iter()
                            .map(lowered_value_kind)
                            .collect::<Vec<_>>()
                    ),
                ));
            }
            if let Some(induction) = active.induction {
                return Ok(induction);
            }
            let mut values = Vec::new();
            append_recursive_argument_values(
                builder,
                &lowered_args,
                &mut values,
                &self.native_int_tags,
            )?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );

            // Continue lowering only in a predecessor-free block. This keeps
            // the structured builder usable while the real recursive edge
            // returns directly to the loop header.
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(Lowered::RecursiveBackedge);
        }

        // Only declarations in an actual recursive SCC need the loop/result
        // closure below. Preserve the established direct-call lowering for
        // ordinary declarations, including constructor-valued HostIO trees.
        if !self.declaration_is_recursive(symbol) {
            let mut call_env = lowered_args.into_iter().rev().collect::<Vec<_>>();
            call_env.extend_from_slice(captures);
            call_env.extend_from_slice(producer_env);
            return if let Some(eliminators) = eliminators {
                self.lower_computational_producer_expr(builder, body, &call_env, eliminators)
            } else {
                self.lower_expr(builder, body, &call_env)
            };
        }

        if eliminators.is_none() && params.len() == 1 && lowered_args.len() == 1 {
            if let RuntimeExpr::Match {
                scrutinee, cases, ..
            } = body
            {
                if matches!(scrutinee.as_ref(), RuntimeExpr::Var(0)) {
                    let zero = cases.iter().find(|case| {
                        case.constructor == self.process_symbols.nat_zero && case.binders == 0
                    });
                    let suc = cases.iter().find(|case| {
                        case.constructor == self.process_symbols.nat_suc && case.binders == 1
                    });
                    if let (Some(zero), Some(suc)) = (zero, suc) {
                        return self.lower_unary_recursive_nat_fold(
                            builder,
                            symbol,
                            captures,
                            lowered_args
                                .into_iter()
                                .next()
                                .expect("unary recursion owns one argument"),
                            &zero.body,
                            &suc.body,
                            producer_env,
                        );
                    }
                }
            }
        }

        let header = builder.create_block();
        let done = builder.create_block();
        let mut initial_values = Vec::new();
        append_recursive_argument_values(
            builder,
            &lowered_args,
            &mut initial_values,
            &self.native_int_tags,
        )?;
        for value in &initial_values {
            builder.append_block_param(header, builder.func.dfg.value_type(*value));
        }
        builder.append_block_param(done, types::I64);
        builder.append_block_param(done, types::I64);
        builder.ins().jump(
            header,
            &initial_values
                .iter()
                .copied()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        builder.switch_to_block(header);

        let mut parameters = builder.block_params(header).iter().copied();
        let mut loop_args = Vec::with_capacity(lowered_args.len());
        for template in &lowered_args {
            loop_args.push(rebuild_recursive_argument(
                template,
                &mut parameters,
                &mut self.native_int_tags,
            )?);
        }
        if parameters.next().is_some() {
            return Err(unsupported(
                "DeclarationRef",
                "recursive declaration loop parameter shape is not closed",
            ));
        }
        self.active_recursive_declarations
            .push(ActiveRecursiveDeclarationV1 {
                symbol: symbol.clone(),
                header: Some(header),
                argument_templates: lowered_args,
                induction: None,
            });
        // Runtime environments are de Bruijn-nearest first: source arguments
        // are evaluated left-to-right, then installed in reverse binder order,
        // followed by captures and the producer environment.
        let mut call_env = loop_args.into_iter().rev().collect::<Vec<_>>();
        call_env.extend_from_slice(captures);
        call_env.extend_from_slice(producer_env);
        let lowered = if let Some(eliminators) = eliminators {
            self.lower_computational_producer_expr(builder, body, &call_env, eliminators)
        } else {
            self.lower_expr(builder, body, &call_env)
        };
        self.active_recursive_declarations.pop();
        let lowered = lowered?;
        let (value, result_kind) = self.merge_scalar_branch(builder, lowered, "DeclarationRef")?;
        builder
            .ins()
            .jump(done, &[value.tag.into(), value.payload.into()]);
        builder.switch_to_block(done);
        Ok(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done)[0],
                payload: builder.block_params(done)[1],
            },
        ))
    }

    fn lower_declaration_ref(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
    ) -> Result<Lowered, CraneliftBackendError> {
        let declaration = self
            .declarations
            .get(symbol.as_str())
            .copied()
            .ok_or_else(|| {
                unsupported(
                    "DeclarationRef",
                    format!("{symbol} is not present in the exact RuntimeProgram"),
                )
            })?;
        let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
            return Err(unsupported(
                "DeclarationRef",
                format!("{symbol} is not an executable transparent declaration"),
            ));
        };
        if let RuntimeExpr::Closure {
            captures,
            params,
            body,
        } = body
        {
            let captures = captures
                .iter()
                .map(|capture| self.lower_seed_capture(builder, capture))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Lowered::DeclarationClosure {
                symbol: symbol.clone(),
                captures,
                params: params.clone(),
                body: (**body).clone(),
            });
        }
        if self.declaration_stack.contains(symbol) {
            return Err(unsupported(
                "DeclarationRef",
                format!("recursive non-function declaration {symbol} is unsupported"),
            ));
        }
        self.declaration_stack.push(symbol.clone());
        let result = self.lower_expr(builder, body, &[]);
        self.declaration_stack.pop();
        result
    }

    fn lower_borrowed_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        pointer: cranelift_codegen::ir::Value,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let kind = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 0);
        Self::require_i64(builder, kind, 2);
        let tag = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 8);
        let arity = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), pointer, 24);
        let pointer_type = builder.func.dfg.value_type(pointer);
        let fields = builder
            .ins()
            .load(pointer_type, MemFlags::trusted(), pointer, 16);
        if let [case] = cases {
            let (expected_tag, expected_arity) =
                borrowed_constructor_identity(&self.process_symbols, &case.constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            format!("{} has no borrowed constructor identity", case.constructor),
                        )
                    })?;
            if case.binders != expected_arity {
                return Err(unsupported(
                    "Match",
                    format!("{} borrowed arity mismatch", case.constructor),
                ));
            }
            let arm = builder.create_block();
            let rejected = builder.create_block();
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                expected_tag,
            );
            builder.ins().brif(selected, arm, &[], rejected, &[]);
            builder.switch_to_block(rejected);
            let failure = builder.ins().iconst(types::I64, -1);
            builder.ins().return_(&[failure]);
            builder.switch_to_block(arm);
            Self::require_i64(builder, arity, expected_arity as i64);
            if expected_arity != 0 {
                Self::require_nonzero(builder, fields);
            }
            let mut arm_env = (0..expected_arity)
                .map(|index| {
                    let field = builder.ins().iadd_imm(fields, (index * 32) as i64);
                    Lowered::BorrowedNativeValue { pointer: field }
                })
                .collect::<Vec<_>>();
            arm_env.extend_from_slice(env);
            return self.lower_expr(builder, &case.body, &arm_env);
        }
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let mut test_block = builder.current_block().expect("borrowed match block");
        let mut merge_kind = None;
        for case in cases {
            let (expected_tag, expected_arity) =
                borrowed_constructor_identity(&self.process_symbols, &case.constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            format!("{} has no borrowed constructor identity", case.constructor),
                        )
                    })?;
            if case.binders != expected_arity {
                return Err(unsupported(
                    "Match",
                    format!("{} borrowed arity mismatch", case.constructor),
                ));
            }
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                expected_tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            Self::require_i64(builder, arity, expected_arity as i64);
            if expected_arity != 0 {
                Self::require_nonzero(builder, fields);
            }
            let mut arm_env = (0..expected_arity)
                .map(|index| {
                    let field = builder.ins().iadd_imm(fields, (index * 32) as i64);
                    Lowered::BorrowedNativeValue { pointer: field }
                })
                .collect::<Vec<_>>();
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, kind) = self.merge_scalar_branch(builder, lowered, "Match")?;
            Self::record_scalar_merge_kind("Match", &mut merge_kind, kind)?;
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("borrowed match emits at least one case"),
            pair,
        ))
    }

    fn lower_borrowed_option_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        present: cranelift_codegen::ir::Value,
        value: cranelift_codegen::ir::Value,
        none: &str,
        some: &str,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let some_block = builder.create_block();
        let none_block = builder.create_block();
        let mut exit_merge = None;
        builder
            .ins()
            .brif(present, some_block, &[], none_block, &[]);
        for (block, symbol, fields) in [
            (some_block, some, vec![Lowered::Int { value, known: None }]),
            (none_block, none, Vec::new()),
        ] {
            builder.switch_to_block(block);
            let case = cases.iter().find(|case| case.constructor == symbol);
            let Some(case) = case else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            if case.binders != fields.len() {
                return Err(unsupported("Match", "borrowed Option arity mismatch"));
            }
            let mut arm_env = fields;
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, is_exit) = self.merge_branch_value(builder, lowered, "Match")?;
            Self::record_merge_kind("Match", &mut exit_merge, is_exit)?;
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(if exit_merge == Some(true) {
            Lowered::ProcessExitStatus {
                value: pair.payload,
            }
        } else {
            self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_dynamic_host_result_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        error: Lowered,
        ok: Lowered,
        err_constructor: &str,
        ok_constructor: &str,
        cases: &[crate::RuntimeMatchCase],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let ok_block = builder.create_block();
        let err_block = builder.create_block();
        let mut merge_kind = None;
        builder.ins().brif(success, ok_block, &[], err_block, &[]);
        for (block, constructor, payload) in [
            (ok_block, ok_constructor, ok),
            (err_block, err_constructor, error),
        ] {
            builder.switch_to_block(block);
            let Some(case) = cases
                .iter()
                .find(|case| case.constructor == constructor && case.binders == 1)
            else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            let mut arm_env = vec![payload];
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, branch_kind) = self.merge_scalar_branch(builder, lowered, "Match")?;
            Self::record_scalar_merge_kind("Match", &mut merge_kind, branch_kind)?;
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("HostResult emits both closed alternatives"),
            pair,
        ))
    }

    fn lower_bounded_nat_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let zero = cases
            .iter()
            .find(|case| case.constructor == self.process_symbols.nat_zero && case.binders == 0);
        let suc = cases
            .iter()
            .find(|case| case.constructor == self.process_symbols.nat_suc && case.binders == 1);
        let (Some(zero), Some(suc)) = (zero, suc) else {
            return Err(unsupported(
                "BoundedNat",
                "structural Nat match requires exact Zero and Suc predecessor arms",
            ));
        };
        let zero_block = builder.create_block();
        let suc_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);
        let mut merge_kind = None;
        for (block, case, predecessor) in [
            (zero_block, zero, None),
            (suc_block, suc, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let mut arm_env = predecessor
                .map(|predecessor| {
                    vec![if structural {
                        Lowered::StructuralNat(StructuralNatV1 {
                            value: predecessor.value,
                        })
                    } else {
                        Lowered::BoundedNat(predecessor)
                    }]
                })
                .unwrap_or_default();
            arm_env.extend_from_slice(env);
            let lowered = self.lower_expr(builder, &case.body, &arm_env)?;
            let (value, kind) = self.merge_scalar_branch(builder, lowered, "BoundedNat")?;
            Self::record_scalar_merge_kind("BoundedNat", &mut merge_kind, kind)?;
            builder
                .ins()
                .jump(merge, &[value.tag.into(), value.payload.into()]);
        }
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("both structural Nat arms were emitted"),
            pair,
        ))
    }

    fn lower_dynamic_constructor_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        continuation: DynamicConstructorContinuation<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;

        let (source_cases, source_default) = match continuation {
            DynamicConstructorContinuation::Ordinary { cases, default, .. }
            | DynamicConstructorContinuation::Producer { cases, default, .. } => (cases, default),
        };
        let has_selected_case = dynamic.alternatives.iter().any(|alternative| {
            source_cases
                .iter()
                .any(|case| case.constructor == alternative.constructor)
        });
        let merge = has_selected_case.then(|| {
            let merge = builder.create_block();
            builder.append_block_param(merge, types::I64);
            builder.append_block_param(merge, types::I64);
            merge
        });
        let mut test_block = builder
            .current_block()
            .expect("dynamic constructor match block");
        let mut merge_kind = None;
        for alternative in dynamic.alternatives {
            let arm = builder.create_block();
            let next = builder.create_block();
            if builder.current_block() != Some(test_block) {
                builder.switch_to_block(test_block);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            let (cases, default, env) = match continuation {
                DynamicConstructorContinuation::Ordinary {
                    cases,
                    default,
                    env,
                }
                | DynamicConstructorContinuation::Producer {
                    cases,
                    default,
                    env,
                    ..
                } => (cases, default, env),
            };
            let case = match select_dynamic_constructor_case(cases, &alternative, default)? {
                Ok(case) => case,
                Err(_owned_default) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let arm_env = materialize_dynamic_constructor_env(&alternative, env);
            let lowered = match continuation {
                DynamicConstructorContinuation::Ordinary { .. } => {
                    self.lower_expr(builder, &case.body, &arm_env)?
                }
                DynamicConstructorContinuation::Producer { eliminators, .. } => self
                    .lower_computational_producer_expr(
                        builder,
                        &case.body,
                        &arm_env,
                        eliminators,
                    )?,
            };
            let (value, branch_kind) =
                self.merge_scalar_branch(builder, lowered, "DynamicConstructor")?;
            Self::record_scalar_merge_kind("DynamicConstructor", &mut merge_kind, branch_kind)?;
            builder.ins().jump(
                merge.expect("a selected dynamic constructor case owns the merge"),
                &[value.tag.into(), value.payload.into()],
            );
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(Lowered::Trap(source_default.clone()));
        };
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(self.lowered_from_scalar_pair(
            merge_kind.expect("a selected dynamic constructor case emits one arm"),
            pair,
        ))
    }

    fn lower_primitive_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        primitive: &RuntimePrimitive,
        args: &[RuntimeExpr],
        env: &[Lowered],
    ) -> Result<Lowered, CraneliftBackendError> {
        let lowered_args = args
            .iter()
            .map(|arg| self.lower_expr(builder, arg, env))
            .collect::<Result<Vec<_>, _>>()?;
        if lowered_args
            .iter()
            .any(|arg| matches!(arg, Lowered::RecursiveBackedge))
        {
            return Ok(Lowered::RecursiveBackedge);
        }

        match &primitive.partiality {
            RuntimePartiality::Total => {}
            RuntimePartiality::SafeOption { .. } | RuntimePartiality::SafeResult { .. } => {}
            RuntimePartiality::CheckedTrap { obligation } => {
                self.assumptions.insert(format!(
                    "checked partial obligation {obligation} not discharged"
                ));
                let message = if obligation.ends_with(".bounds") {
                    format!("{} bounds obligation failed", primitive.symbol)
                } else {
                    format!("{} checked partiality trapped", primitive.symbol)
                };
                return Ok(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message,
                }));
            }
            RuntimePartiality::TrustedTrap { assumption } => {
                self.assumptions.insert(format!(
                    "trusted partial assumption {assumption} remains visible"
                ));
                return Ok(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: format!("{} trusted partiality trapped", primitive.symbol),
                }));
            }
        }

        match primitive.symbol.as_str() {
            "add_int" => self.lower_int_binop(builder, "add_int", lowered_args, |lhs, rhs| {
                lhs.checked_add(rhs)
            }),
            "sub_int" => self.lower_int_binop(builder, "sub_int", lowered_args, |lhs, rhs| {
                lhs.checked_sub(rhs)
            }),
            "mul_int" => self.lower_int_binop(builder, "mul_int", lowered_args, |lhs, rhs| {
                lhs.checked_mul(rhs)
            }),
            "eq_int" => self.lower_int_cmp(
                builder,
                "eq_int",
                lowered_args,
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                |lhs, rhs| lhs == rhs,
            ),
            "leq_int" => self.lower_int_cmp(
                builder,
                "leq_int",
                lowered_args,
                cranelift_codegen::ir::condcodes::IntCC::SignedLessThanOrEqual,
                |lhs, rhs| lhs <= rhs,
            ),
            "uint8_to_int" | "int_to_uint8_raw" => {
                let [value]: [Lowered; 1] = lowered_args.try_into().map_err(|args: Vec<_>| {
                    unsupported(
                        "PrimitiveCall",
                        format!(
                            "{} expects one argument, got {}",
                            primitive.symbol,
                            args.len()
                        ),
                    )
                })?;
                let Lowered::Int { .. } = value else {
                    return Err(unsupported(
                        "PrimitiveCall",
                        format!("{} expects an Int-represented value", primitive.symbol),
                    ));
                };
                Ok(value)
            }
            "not_bool" => self.lower_bool_not(builder, lowered_args),
            "and_bool" => self.lower_bool_binop(
                builder,
                "and_bool",
                lowered_args,
                |builder, lhs, rhs| builder.ins().band(lhs, rhs),
                |lhs, rhs| lhs && rhs,
            ),
            "or_bool" => self.lower_bool_binop(
                builder,
                "or_bool",
                lowered_args,
                |builder, lhs, rhs| builder.ins().bor(lhs, rhs),
                |lhs, rhs| lhs || rhs,
            ),
            "bytes_length" => self.lower_bytes_length(builder, lowered_args),
            "bytes_at" => self.lower_bytes_at(builder, lowered_args, &primitive.partiality),
            "bytes_slice" => self.lower_bytes_slice(lowered_args, &primitive.partiality),
            "bytes_concat" => self.lower_bytes_concat(lowered_args),
            "bytes_encode" => self.lower_bytes_encode(lowered_args),
            "bytes_decode" => self.lower_bytes_decode(lowered_args, &primitive.partiality),
            "list_char_to_string" => {
                let [value]: [Lowered; 1] = lowered_args.try_into().map_err(|args: Vec<_>| {
                    unsupported(
                        "PrimitiveCall",
                        format!(
                            "list_char_to_string expects one argument, got {}",
                            args.len()
                        ),
                    )
                })?;
                let bytes = lowered_char_list(&value).ok_or_else(|| {
                    unsupported(
                        "PrimitiveCall",
                        "list_char_to_string requires a closed List Char",
                    )
                })?;
                let value = String::from_utf8(bytes).map_err(|_| {
                    unsupported(
                        "PrimitiveCall",
                        "list_char_to_string received non-UTF-8 Char values",
                    )
                })?;
                Ok(Lowered::String(value))
            }
            "byte_length" => self.lower_string_byte_length(builder, lowered_args),
            "char_length" => self.lower_string_char_length(builder, lowered_args),
            other => Err(unsupported(
                "PrimitiveCall",
                format!("primitive {other} is not in the supported native set"),
            )),
        }
    }
}
