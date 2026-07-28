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
    // `'a`, not an anonymous borrow: the plan files each planned occurrence's term
    // by reference, so the source tree must outlive the lowering that resolves
    // tags against it. Nothing borrowed reaches `CompiledModule`, which has no
    // lifetime parameter — see `Lowering::static_transition_plan`.
    expr: &'a RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    validate_oriented_subcontinuation_transport(
        expr,
        &declarations,
        oriented_subcontinuation_plan.as_ref(),
    )?;
    // Boundary A of RT-NATIVE-FNSPLIT: close and validate the factored static
    // graph before Cranelift sees any semantic body. The plan's positional
    // child-origin table is reachable from the lowering, so
    // every occurrence carries the static name the planner already gave it.
    //
    // ⚠ The plan also outlives this call's borrow of `expr` and holds each planned
    // occurrence BY REFERENCE, because a retained closure body is now selected by
    // its origin rather than carried as a clone. The emitter is otherwise
    // unchanged, and nothing borrowed reaches `CompiledModule`.
    let static_transition_plan = plan_static_transition_graph(expr, &declarations)?;
    let root_static_origin = static_transition_plan.root_static_origin()?;
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));

    let func_id = module
        .declare_function(function_name, linkage, &sig)
        .map_err(|err| backend_module(err.to_string()))?;
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
    // `RT-FNSPLIT-B2V` `D3` — the boundary-value interface is declared and
    // defined in EVERY module, a fixed Θ(1) population, so a unit can project a
    // transfer wherever it is emitted rather than only where some caller
    // arranged a decoder.
    //
    // ⛔ `D6` INERT: nothing calls these yet. This adds no generated function
    // for any semantic origin, no cross-owner call and no second body-emission
    // authority — `RT-FNSPLIT-B2F` performs the switch-over that consumes them.
    // The population is emitted unconditionally so that B2F's switch-over is a
    // change of caller, never a change of what a module contains.
    // ⛔ `RECUT 2` — the emission plan is DERIVED from the representation
    // authority here, at the single-owner seam, and passed into the emitter.
    // The emitter consumes it to build the helper bodies' legal class sets; it
    // does not restate the authority and cannot reach it (`BoundaryInput` is
    // private to `cranelift_backend::lowering`). Ruled in scope and required by
    // the Architect: production codegen consumption is not `B2F` activation.
    let boundary_plan = crate::boundary_value::BoundaryEmissionPlan::derive();
    // ⭐ `RT-FNSPLIT-C1` `AC-C8` — the emitted graph's result is **consumed**,
    // not bound to `_`. ⚠ Labelled honestly: this is *necessary, not
    // sufficient*. Consuming the handle only proves the helpers are reachable;
    // `AC-C7`'s three per-eliminator executable-edge tests are what make it
    // evidence that the carrier is live. ⛔ Do not report this line alone.
    let boundary_value_abi = crate::boundary_value_clif::emit_boundary_value_local_graph(
        &mut module,
        &native_int,
        &boundary_plan,
    )?;
    let host_dispatch = if process_mode {
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
    let host_dispatch = host_dispatch.map(|id| module.declare_func_in_func(id, &mut ctx.func));
    let int_binop = module.declare_func_in_func(native_int.binop, &mut ctx.func);
    let int_compare = module.declare_func_in_func(native_int.compare, &mut ctx.func);
    let int_intern = module.declare_func_in_func(native_int.intern, &mut ctx.func);
    let int_narrow = module.declare_func_in_func(native_int.narrow, &mut ctx.func);
    let int_export = module.declare_func_in_func(native_int.export, &mut ctx.func);
    // `RT-FNSPLIT-C1` `D3` — the carrier helpers become **callable refs inside
    // this generated function**, exactly as the native-int helpers above do.
    let boundary_carrier = BoundaryCarrierRefs {
        tag: module.declare_func_in_func(boundary_value_abi.tag, &mut ctx.func),
        field_count: module.declare_func_in_func(boundary_value_abi.field_count, &mut ctx.func),
        field: module.declare_func_in_func(boundary_value_abi.field, &mut ctx.func),
        record_field: module.declare_func_in_func(boundary_value_abi.record_field, &mut ctx.func),
        alloc: module.declare_func_in_func(boundary_value_abi.alloc, &mut ctx.func),
        store_tag_id: module.declare_func_in_func(boundary_value_abi.store_tag_id, &mut ctx.func),
        store_field: module.declare_func_in_func(boundary_value_abi.store_field, &mut ctx.func),
        store_name: module.declare_func_in_func(boundary_value_abi.store_name, &mut ctx.func),
        make_immediate: module
            .declare_func_in_func(boundary_value_abi.make_immediate, &mut ctx.func),
    };

    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        declarations,
        static_transition_plan,
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
        boundary_carrier: Some(boundary_carrier),
        #[cfg(test)]
        native_int_mutation: NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get),
        #[cfg(test)]
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    };
    let (maybe_trap, decoder) = {
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
            initial_env.push(LoweringOperand::Specialized(
                Lowered::BorrowedNativeValue {
                    pointer: process_input,
                },
            ));
            initial_env.push(LoweringOperand::Specialized(Lowered::CapabilityToken {
                value: capability,
            }));
        }
        if let Some(value) = staged_process_input {
            initial_env.push(LoweringOperand::Specialized(
                compiler.lower_value(&mut builder, value)?,
            ));
        }
        compiler.root_terminal_authority = compiler.take_distinguished_root_answer_authority()?;
        // D6/D9: the root lowering starts from the occurrence origin stored
        // during the planner's root visit — never derived from the plan's
        // scheduling `entries` — so the two walks start from one identity
        // rather than two.
        let lowered = compiler.lower_expr(
            &mut builder,
            SourceOccurrence {
                expr,
                static_origin: root_static_origin,
            },
            &initial_env,
        )?;
        compiler.require_complete_join_plan_consumption()?;
        compiler.require_complete_dynamic_splice_edge_consumption()?;
        // ⭐ `§2h` ¶2's typed phase boundary at the ROOT result surface.
        // ⛔ Not a `Carried -> Lowered` conversion: a carried root FAILS CLOSED.
        // A generated function's return is `emit_result`'s ground-value encoding,
        // which reads a compile-time template; `D6`'s executable edge runs
        // producer -> validator -> eliminator *inside* a body, and never asks the
        // root to decode a boundary word.
        let lowered = lowered.specialized_at("the generated function's root result")?;
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

    verify_cranelift_function(&ctx.func, module.isa())?;
    module
        .define_function(func_id, &mut ctx)
        .map_err(|err| backend_module(err.to_string()))?;
    // The plan is no longer dropped unused here: `compiler` owns it and it dies
    // with this call. `CompiledModule::from_parts` below takes only owned data
    // and the type has no lifetime parameter, so no part of the plan can reach
    // the artifact — the non-escape property is a fact about the types.

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
    fn resume_active_continuation(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: LoweringOperand,
        active: ActiveContinuationFrame<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let Some((head, tail)) = active.pending.split_first() else {
            return Ok(value);
        };
        let cursor = self.mint_continuation_cursor();
        let successor = EliminatorFrame::Active(ActiveContinuationFrame {
            activation: active.activation,
            cursor,
            parent: Some(&active),
            pending: tail,
            selected_ancestry: active.selected_ancestry,
            source_lineage: active.source_lineage,
            source_selected_cursor: active.source_selected_cursor,
            selected_scope: active.selected_scope,
        });
        self.lower_computational_match_value_composed(builder, value, &[*head, successor])
    }

    /// ⛔⛔ **`AC-C4` clause 3 — a carried residual is a transferred VALUE, never
    /// a transferred callable.** So an induction-hypothesis invocation against
    /// one may not carry source arguments, and this refuses **before** any
    /// invocation segment is installed or any semantic region entered.
    ///
    /// ⭐ **The zero-argument structural IH route is the admitted carried
    /// route**, and it is the only one. A function-valued recursive field would
    /// need the residual to *be* a closure over the carrier — the durable
    /// closure lane the ruling explicitly withholds — so it stays excluded by
    /// the existing closure-transfer prohibition rather than by a new check.
    ///
    /// ⚠ **Why this is a shared associated fn and not four inline `if`s.** Each
    /// of the four residual consumers reaches its carried arm by a different
    /// route, and a refusal that drifted at one of them would be a hole with
    /// three green siblings — the shape `AC-C7` already caught once on this
    /// node. One body, one message, one place to mutate.
    ///
    /// ⚠ Takes a **count**, not a slice: the four consumers hold their argument
    /// lists in two different forms (source `RuntimeExpr`s on three routes, an
    /// already-lowered operand vector on the source-machine route), and the
    /// property is about arity in both. A slice parameter would have forced one
    /// of them to spell its own refusal.
    fn reject_carried_residual_arguments(
        arguments: usize,
    ) -> Result<(), CraneliftBackendError> {
        if arguments == 0 {
            return Ok(());
        }
        Err(unsupported(
            "BoundaryCarrier",
            format!(
                "a carried recursive hypothesis is an eliminated value, not a callable, \
                 so it takes no arguments, but the call provides {arguments}"
            ),
        ))
    }

    /// `call_origin` is the origin of the `Call` occurrence `args` belong to.
    #[allow(clippy::too_many_arguments)]
    fn lower_recursor_residual_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        residual: &LoweringOperand,
        args: &[RuntimeExpr],
        call_origin: StaticOriginId,
        argument_env: &[LoweringOperand],
        saved_producer_env: &[LoweringOperand],
        outer_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐⭐ `AC-C4` — the carried residual, taken BEFORE the specialized
        // shapes so a carried word never reaches a template probe.
        if let LoweringOperand::Carried(word) = residual {
            Self::reject_carried_residual_arguments(args.len())?;
            return self.lower_computational_match_value_composed(
                builder,
                LoweringOperand::Carried(*word),
                outer_eliminators,
            );
        }
        let residual = residual.specialized_ref_at("a pending-let recursor residual")?;
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
            .enumerate()
            .map(|(position, arg)| {
                let arg = self.child_occurrence(call_origin, 1 + position, arg)?;
                self.lower_expr(builder, arg, argument_env)
            })
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
        extend_specialized(&mut call_env, captures.iter().cloned());
        call_env.extend_from_slice(saved_producer_env);
        self.lower_computational_producer_expr(
            builder,
            self.retained_body_occurrence(*body)?,
            &call_env,
            outer_eliminators,
        )
    }

    /// `static_origin` is the `ComputationalMatch` occurrence's own origin, so
    /// `scrutinee` is its child `0` and case *i*'s body is its child `1 + i`.
    #[allow(clippy::too_many_arguments)]
    fn lower_computational_match_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: SourceOccurrence<'_>,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        producer_env: &[LoweringOperand],
        eliminator_env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
                    static_origin,
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

    /// Lowers one source occurrence as a *producer* under a stack of eliminator
    /// frames.
    ///
    /// This is the second traversal of the same source population — the one that
    /// reaches occurrences the direct descent does not — so it threads origins by
    /// exactly the same table as `lower_expr`: no guessed subset, both routes or
    /// neither.
    fn lower_computational_producer_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        producer_env: &[LoweringOperand],
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let SourceOccurrence {
            expr: scrutinee,
            static_origin,
        } = occurrence;
        if eliminators.is_empty() {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        }
        if matches!(eliminators[0], EliminatorFrame::InvocationReturn) {
            return self.lower_expr(builder, occurrence, producer_env);
        }
        if let EliminatorFrame::PendingLet(continuation) = eliminators[0] {
            let value = self.lower_expr(builder, occurrence, producer_env)?;
            if matches!(value, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
            }
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = value {
                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
            }
            let mut continuation_env = vec![value];
            continuation_env.extend_from_slice(continuation.env);
            return self.lower_recursor_residual_call(
                builder,
                continuation.residual,
                continuation.args,
                continuation.call_origin,
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
                let value = self.lower_expr(builder, occurrence, producer_env)?;
                return self.resume_active_continuation(builder, value, active);
            }
        }
        match scrutinee {
            RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                self.enter_checked_subcontinuation_frame(*frame_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
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
                let body = self.child_occurrence(static_origin, 0, body)?;
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
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.lower_computational_producer_expr(builder, body, producer_env, eliminators)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let value = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                )?;
                self.finish_checked_computational_ih_marker(value)
            }
            RuntimeExpr::Let { value, body } => {
                // The `Let`'s own children: value `0`, body `1`. When the body
                // is itself the `Call` below, that `Call` occurrence's origin is
                // this body child — which is what the pending-let frame carries
                // so its arguments stay positionally derivable.
                let body_origin = self.static_transition_plan.child_static_origin(static_origin, 1)?;
                if reaches_environment_computational_recursor(body, producer_env, 1) {
                    if let RuntimeExpr::Call { callee, args } = body.as_ref() {
                        if let RuntimeExpr::Var(index) = callee.as_ref() {
                            if let Some(index) = (*index as usize).checked_sub(1) {
                                if let Some(LoweringOperand::Specialized(
                                    callee @ Lowered::ComputationalRecursorClosure { .. },
                                )) = producer_env.get(index)
                                {
                                    let (residual, boundary) = decompose_computational_recursor(
                                        LoweringOperand::Specialized(callee.clone()),
                                    );
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
                                            call_origin: body_origin,
                                            env: producer_env,
                                        },
                                    ));
                                    composed.extend(frames);
                                    composed.push(EliminatorFrame::InvocationReturn);
                                    self.enter_oriented_semantic_region(installed.checked);
                                    let value =
                                        self.child_occurrence(static_origin, 0, value)?;
                                    let returned = self.lower_computational_producer_expr(
                                        builder,
                                        value,
                                        producer_env,
                                        &composed,
                                    );
                                    self.leave_oriented_semantic_region(installed.checked);
                                    let returned = returned?;
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
                let value_occurrence = self.child_occurrence(static_origin, 0, value)?;
                let body_occurrence = SourceOccurrence {
                    expr: body,
                    static_origin: body_origin,
                };
                let value = self.lower_expr(builder, value_occurrence, producer_env)?;
                if let LoweringOperand::Specialized(Lowered::Trap(trap)) = value {
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                }
                let mut body_env = vec![value];
                body_env.extend_from_slice(producer_env);
                self.lower_computational_producer_expr(
                    builder,
                    body_occurrence,
                    &body_env,
                    eliminators,
                )
            }
            RuntimeExpr::Call { callee, args } => {
                let callee = self.child_occurrence(static_origin, 0, callee)?;
                let callee = self.lower_expr(builder, callee, producer_env)?;
                match callee {
                    LoweringOperand::Specialized(Lowered::DeclarationClosure {
                        symbol,
                        captures,
                        params,
                        body,
                    }) => self.lower_recursive_declaration_call(
                        builder,
                        &symbol,
                        &captures,
                        &params,
                        self.retained_body_occurrence(body)?,
                        args,
                        static_origin,
                        producer_env,
                        Some(eliminators),
                    ),
                    LoweringOperand::Specialized(Lowered::Closure {
                        captures,
                        params,
                        body,
                    }) => {
                        // Resolve the tag once, here, and shadow it: everything
                        // below reads the body's syntax or its origin, and both come
                        // from the one resolution rather than from a term the
                        // closure carried.
                        let body = self.retained_body_occurrence(body)?;
                        if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                            if let Some((cases, default)) =
                                ordinary_match_continuation(&params, body.expr)
                            {
                                let argument = self.child_occurrence(static_origin, 1, &args[0])?;
                                let frame_env = env_with(captures, producer_env);
                                let mut composed = Vec::with_capacity(eliminators.len() + 1);
                                composed.push(EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                    cases,
                                    default,
                                    env: &frame_env,
                                    static_origin: body.static_origin,
                                    retained_scrutinee_index: Some(0),
                                    deferred_constructor_case: None,
                                }));
                                composed.extend_from_slice(eliminators);
                                return self.lower_computational_producer_expr(
                                    builder,
                                    argument,
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
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                self.lower_expr(builder, arg, producer_env)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures.into_iter().map(LoweringOperand::Specialized));
                        call_env.extend_from_slice(producer_env);
                        self.lower_computational_producer_expr(
                            builder,
                            body,
                            &call_env,
                            eliminators,
                        )
                    }
                    LoweringOperand::Specialized(
                        mut callee @ Lowered::ComputationalRecursorClosure { .. },
                    ) => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(
                            LoweringOperand::Specialized(callee),
                        );
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
                        // ⭐⭐ `AC-C4` — the carried residual resumes the SAME
                        // computational eliminator over the carried word, under
                        // the same semantic-region bracket the specialized
                        // `BoundedNat` arm below uses. ⛔ Not `specialized_at`,
                        // ⛔ not a reconstructed `Lowered`, ⛔ not the producer.
                        if let LoweringOperand::Carried(word) = base {
                            Self::reject_carried_residual_arguments(args.len())?;
                            self.enter_oriented_semantic_region(installed.checked);
                            let returned = self.lower_computational_match_value_composed(
                                builder,
                                LoweringOperand::Carried(word),
                                &composed,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            let returned = returned?;
                            return self.lower_computational_match_value_composed(
                                builder,
                                returned,
                                eliminators,
                            );
                        }
                        let base =
                            base.specialized_at("a recursor residual in a producer call")?;
                        if let Lowered::BoundedNat(predecessor) = base {
                            if !args.is_empty() {
                                return Err(unsupported(
                                    "BoundedNat",
                                    "structural Nat recursive hypothesis takes no arguments",
                                ));
                            }
                            self.enter_oriented_semantic_region(installed.checked);
                            let returned = self.lower_bounded_nat_computational(
                                builder,
                                predecessor,
                                false,
                                &composed,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            let returned = returned?;
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
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                self.lower_expr(builder, arg, producer_env)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures.into_iter().map(LoweringOperand::Specialized));
                        call_env.extend_from_slice(producer_env);
                        self.enter_oriented_semantic_region(installed.checked);
                        let returned = self.lower_computational_producer_expr(
                            builder,
                            self.retained_body_occurrence(body)?,
                            &call_env,
                            &composed,
                        );
                        self.leave_oriented_semantic_region(installed.checked);
                        let returned = returned?;
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
                        .enumerate()
                        .map(|(position, arg)| {
                            let arg = self.child_occurrence(static_origin, position, arg)?;
                            self.lower_expr(builder, arg, producer_env)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return Ok(LoweringOperand::Specialized(Lowered::Constructor {
                        constructor: constructor.clone(),
                        args: specialized_env_at(&lowered_args, "a constructor argument")?,
                    }));
                }
                let (case_body, argument_binder_offset) = match eliminator {
                    EliminatorFrame::Computational(eliminator) => {
                        let (case_index, case) = match eliminator
                            .cases
                            .iter()
                            .enumerate()
                            .find(|(_, case)| case.constructor == *constructor)
                        {
                            Some(selected) => selected,
                            None => return Ok(LoweringOperand::Specialized(Lowered::Trap(eliminator.default.clone()))),
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
                        (
                            self.case_body_occurrence(
                                eliminator.static_origin,
                                case_index,
                                &case.body,
                            )?,
                            case.recursive_positions.len(),
                        )
                    }
                    EliminatorFrame::Ordinary(eliminator) => {
                        let (case_index, case) = match select_ordinary_case(eliminator, constructor)
                        {
                            Ok(selected) => selected,
                            Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
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
                        (
                            self.case_body_occurrence(
                                eliminator.static_origin,
                                case_index,
                                &case.body,
                            )?,
                            0,
                        )
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

                // The bridge eliminator's cases live in the selected case body
                // itself (`immediate_binder_eliminator` matches only a body that
                // IS a match), so that body's origin is their parent.
                let bridge = immediate_binder_eliminator(
                    case_body.expr,
                    argument_binder_offset,
                    args.len(),
                );
                let bridge =
                    bridge.filter(|(field, _)| requires_heterogeneous_deforestation(&args[*field]));

                if let Some((field, consumer)) = bridge {
                    let lowered_prefix = args[..field]
                        .iter()
                        .enumerate()
                        .map(|(position, arg)| {
                            let arg = self.child_occurrence(static_origin, position, arg)?;
                            self.lower_expr(builder, arg, producer_env)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(LoweringOperand::Specialized(Lowered::Trap(trap))) = lowered_prefix
                        .iter()
                        .find(|value| matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))))
                    {
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap.clone())));
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
                    let selected_active = ActiveContinuationFrame {
                        activation: self.mint_continuation_activation(),
                        cursor: self.mint_continuation_cursor(),
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
                    // ⭐ The prefix rebuilds the enclosing constructor's own
                    // **template** below (`outer_scrutinee`), so it is a
                    // specialized-only surface, not a spine edge.
                    let lowered_prefix =
                        specialized_env_at(&lowered_prefix, "a deferred constructor prefix")?;
                    let deferred = DeferredConstructorCaseEnvironment {
                        constructor,
                        lowered_prefix: &lowered_prefix,
                        selected_field: field,
                        trailing_fields: &args[field + 1..],
                        construct_origin: static_origin,
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
                                static_origin: case_body.static_origin,
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
                                static_origin: case_body.static_origin,
                                retained_scrutinee_index: None,
                                deferred_constructor_case: Some(&deferred),
                            })
                        }
                    });
                    composed.push(EliminatorFrame::Active(selected_active));
                    let selected = self.child_occurrence(static_origin, field, &args[field])?;
                    return self.lower_computational_producer_expr(
                        builder,
                        selected,
                        producer_env,
                        &composed,
                    );
                }

                let lowered_args = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(static_origin, position, arg)?;
                        self.lower_expr(builder, arg, producer_env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.lower_computational_match_value_composed(
                    builder,
                    LoweringOperand::Specialized(Lowered::Constructor {
                        constructor: constructor.clone(),
                        args: specialized_env_at(&lowered_args, "a constructor argument")?,
                    }),
                    eliminators,
                )
            }
            RuntimeExpr::Match {
                scrutinee,
                cases: producer_cases,
                default: producer_default,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                if let LoweringOperand::Specialized(Lowered::Bool { value, known }) = selected {
                    let true_case = producer_cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = producer_cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "ComputationalMatch",
                            "Bool tree producer requires True and False cases",
                        ));
                    };
                    if let Some(known) = known {
                        let (index, case) = if known { true_case } else { false_case };
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        return self.lower_computational_producer_expr(
                            builder,
                            body,
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
                    for (block, (index, producer_case)) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let body =
                            self.case_body_occurrence(static_origin, index, &producer_case.body)?;
                        let lowered = self.lower_computational_producer_expr(
                            builder,
                            body,
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
                        LoweringOperand::Specialized(Lowered::ProcessExitStatus {
                            value: pair.payload,
                        })
                    } else {
                        LoweringOperand::Specialized(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
                    });
                }
                if let LoweringOperand::Specialized(Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                }) = selected
                {
                    let ok_block = builder.create_block();
                    let err_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder.append_block_param(merge, types::I64);
                    builder.ins().brif(success, ok_block, &[], err_block, &[]);
                    let mut exit_merge = None;
                    for (block, constructor, payload) in [
                        (ok_block, ok_constructor.as_str(), *ok),
                        (err_block, err_constructor.as_str(), *error),
                    ] {
                        builder.switch_to_block(block);
                        let lowered = if let Some((index, producer_case)) =
                            dynamic_host_result_producer_case(producer_cases, constructor)?
                        {
                            let case_env = env_with([payload], producer_env);
                            let body = self.case_body_occurrence(
                                static_origin,
                                index,
                                &producer_case.body,
                            )?;
                            self.lower_computational_producer_expr(
                                builder,
                                body,
                                &case_env,
                                eliminators,
                            )?
                        } else {
                            LoweringOperand::Specialized(Lowered::Trap(producer_default.clone()))
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
                    return Ok(if exit_merge == Some(true) {
                        LoweringOperand::Specialized(Lowered::ProcessExitStatus {
                            value: pair.payload,
                        })
                    } else {
                        LoweringOperand::Specialized(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
                    });
                }
                if let LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) = selected {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Producer {
                            cases: producer_cases,
                            default: producer_default,
                            env: producer_env,
                            static_origin,
                            eliminators,
                        },
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BoundedNat(nat)) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        static_origin,
                        retained_scrutinee_index: None,
                        deferred_constructor_case: None,
                    };
                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
                    composed.push(EliminatorFrame::Ordinary(frame));
                    composed.extend_from_slice(eliminators);
                    return self.lower_bounded_nat_computational(builder, nat, false, &composed);
                }
                if let LoweringOperand::Specialized(Lowered::StructuralNat(nat)) = selected {
                    let frame = OrdinaryEliminatorFrame {
                        cases: producer_cases,
                        default: producer_default,
                        env: producer_env,
                        static_origin,
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
                let LoweringOperand::Specialized(Lowered::Constructor { constructor, args }) = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match scrutinee is not Bool or a constructor",
                    ));
                };
                let Some((case_index, producer_case)) = producer_cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == constructor)
                else {
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(producer_default.clone())));
                };
                if producer_case.binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match constructor arity changed",
                    ));
                }
                let case_env = env_with(args, producer_env);
                let body =
                    self.case_body_occurrence(static_origin, case_index, &producer_case.body)?;
                self.lower_computational_producer_expr(builder, body, &case_env, eliminators)
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
                        static_origin,
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
                let inner_scrutinee = self.child_occurrence(static_origin, 0, inner_scrutinee)?;
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
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let then_expr = self.child_occurrence(static_origin, 1, then_expr)?;
                let else_expr = self.child_occurrence(static_origin, 2, else_expr)?;
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                let LoweringOperand::Specialized(Lowered::Bool { value, known }) = selected else {
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
                    LoweringOperand::Specialized(Lowered::ProcessExitStatus {
                        value: pair.payload,
                    })
                } else {
                    LoweringOperand::Specialized(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
                })
            }
            _ => {
                // Everything this producer dispatcher does not special-case is
                // handed to `lower_expr` **as the same occurrence**, origin
                // included — the producer-side twin of the source machine's
                // fallback arm.
                let value = self.lower_expr(builder, occurrence, producer_env)?;
                self.lower_computational_match_value_composed(builder, value, eliminators)
            }
        }
    }

    fn lower_computational_match_value_composed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: LoweringOperand,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let Some(eliminator) = eliminators.first().copied() else {
            return Err(unsupported(
                "ComputationalMatch",
                "nested computational producer has no eliminator",
            ));
        };
        // ⭐ The forwarding arm comes FIRST and stays phase-preserving: an
        // invocation return hands the operand straight back, so a `Carried`
        // survives it untouched. Only the composition path below reads a
        // compile-time template, so only that path takes the boundary.
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(scrutinee);
        }
        // ⭐⭐ `D3`'s CARRIED arm for the composed route, ahead of the boundary
        // below — otherwise a carried scrutinee reaching a real eliminator
        // would fail closed at `specialized_at` even though `§2g` gives it a
        // route. ⛔ The phase is classified with no wildcard.
        if let LoweringOperand::Carried(word) = scrutinee {
            return match eliminator {
                EliminatorFrame::Computational(frame) => self
                    .lower_carried_computational_match(builder, word, frame, &eliminators[1..]),
                // ── ⛔ DEFERRED, and named rather than absorbed ────────────
                //
                // ⚠ A composed **ordinary** frame is reached only through
                // heterogeneous deforestation of a producer, and a deforestable
                // producer is by construction one whose shape was read at
                // compile time. So a carried scrutinee cannot arrive here from
                // today's corpus — ⛔ which is exactly why this is written as a
                // fail-closed arm and not omitted: `§2g` requires the phase to
                // be classified, and *"cannot occur"* is a disposition, never a
                // missing cell. The direct `RuntimeExpr::Match` route already
                // carries the real elimination (`Self::lower_carried_match`).
                EliminatorFrame::Ordinary(_) => Err(unsupported(
                    "BoundaryCarrier",
                    "a carried scrutinee reached an ordinary eliminator through the \
                     deforestation producer, which selects its case from a compile-time \
                     constructor shape the carrier does not have",
                )),
                EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => Err(unsupported(
                    "BoundaryCarrier",
                    "a carried scrutinee reached a continuation frame that resumes a \
                     compile-time value rather than eliminating one",
                )),
                // Answered above, before this match; spelled so the frame set
                // stays wildcard-free.
                EliminatorFrame::InvocationReturn => Ok(LoweringOperand::Carried(word)),
            };
        }
        let scrutinee = scrutinee.specialized_at("a composed computational-match scrutinee")?;
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
                let (case_index, case, _) = match select_computational_case(
                    std::slice::from_ref(&eliminator),
                    &constructor,
                ) {
                    Ok(selected) => selected,
                    Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
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
                let selected_scope = OwnedSelectedScope {
                    scope_origin: producer_origin,
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                    frame: ComputationalRecursorFramePayload {
                        cases: eliminator.cases.to_vec(),
                        default: eliminator.default.clone(),
                        outer_env: eliminator.env.to_vec(),
                        static_origin: eliminator.static_origin,
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
                    cursor,
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
                        // ⭐ `AC-C4` clause 1 — the SPECIALIZED caller wraps
                        // explicitly, so the phase is stated at the call site
                        // rather than inferred by the callee.
                        LoweringOperand::Specialized(args[position].clone()),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.static_origin,
                        eliminator.provenance,
                        eliminator.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        activation,
                        cursor,
                        splice_caller,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
                    induction_hypotheses.push(induction_hypothesis);
                }
                let mut case_env = induction_hypotheses;
                case_env.extend(args.into_iter().map(LoweringOperand::Specialized));
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Computational(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
                };
                case_env.extend(frame_env);
                let case_body =
                    self.case_body_occurrence(eliminator.static_origin, case_index, &case.body)?;
                if !case.recursive_positions.is_empty() {
                    return self.lower_source_machine(
                        builder,
                        case_body,
                        &case_env,
                        &active_state,
                    );
                }
                if remaining_eliminators.is_empty() {
                    return self.lower_expr(builder, case_body, &case_env);
                }
                return self.lower_computational_producer_expr(
                    builder,
                    case_body,
                    &case_env,
                    remaining_eliminators,
                );
            }
            EliminatorFrame::Ordinary(eliminator) => {
                let (case_index, case) = match select_ordinary_case(eliminator, &constructor) {
                    Ok(selected) => selected,
                    Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
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
                let mut case_env = env_with(args, &[]);
                let frame_env = match self.materialize_eliminator_frame_env(
                    builder,
                    EliminatorFrame::Ordinary(eliminator),
                    &retained_scrutinee,
                )? {
                    Ok(env) => env,
                    Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
                };
                case_env.extend(frame_env);
                (
                    self.case_body_occurrence(
                        eliminator.static_origin,
                        case_index,
                        &case.body,
                    )?,
                    case_env,
                )
            }
            EliminatorFrame::PendingLet(_) => {
                unreachable!("pending Let continuations are consumed before value composition")
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation returns are consumed before value composition")
            }
            EliminatorFrame::Active(active) => {
                return self.resume_active_continuation(builder, LoweringOperand::Specialized(retained_scrutinee), active);
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
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let eliminator = eliminators[0];
        if matches!(eliminator, EliminatorFrame::InvocationReturn) {
            return Ok(if structural {
                LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 { value: nat.value }))
            } else {
                LoweringOperand::Specialized(Lowered::BoundedNat(nat))
            });
        }
        if let EliminatorFrame::Active(active) = eliminator {
            let value = if structural {
                Lowered::StructuralNat(StructuralNatV1 { value: nat.value })
            } else {
                Lowered::BoundedNat(nat)
            };
            return self.resume_active_continuation(builder, LoweringOperand::Specialized(value), active);
        }
        let remaining = &eliminators[1..];
        let (zero_body, suc_body, computational) = match eliminator {
            EliminatorFrame::Computational(frame) => {
                let zero = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_zero
                        && case.argument_binders == 0
                        && case.recursive_positions.is_empty()
                });
                let suc = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_suc
                        && case.argument_binders == 1
                        && case.recursive_positions.as_slice() == [0]
                });
                let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "computational Nat requires Zero and one recursive Suc predecessor",
                    ));
                };
                (
                    self.case_body_occurrence(frame.static_origin, zero_index, &zero.body)?,
                    self.case_body_occurrence(frame.static_origin, suc_index, &suc.body)?,
                    true,
                )
            }
            EliminatorFrame::Ordinary(frame) => {
                let zero = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_zero && case.binders == 0
                });
                let suc = frame.cases.iter().enumerate().find(|(_, case)| {
                    case.constructor == self.process_symbols.nat_suc && case.binders == 1
                });
                let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) else {
                    return Err(unsupported(
                        "BoundedNat",
                        "ordinary Nat frame requires exact Zero and Suc predecessor arms",
                    ));
                };
                (
                    self.case_body_occurrence(frame.static_origin, zero_index, &zero.body)?,
                    self.case_body_occurrence(frame.static_origin, suc_index, &suc.body)?,
                    false,
                )
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
                Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
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
                Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
            };
        let induction = self.lowered_from_scalar_pair(result_kind, induction);
        let mut suc_env = Vec::new();
        if computational {
            suc_env.push(LoweringOperand::Specialized(induction));
        }
        suc_env.push(LoweringOperand::Specialized(predecessor));
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
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        )))
    }

    fn materialize_eliminator_frame_env(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        eliminator: EliminatorFrame<'_>,
        retained_scrutinee: &Lowered,
    ) -> Result<Result<Vec<LoweringOperand>, RuntimeTrap>, CraneliftBackendError> {
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
                env.insert(index, LoweringOperand::Specialized(retained_scrutinee.clone()));
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
        // The trailing fields are the constructor's own children, continuing
        // past the selected one: `child(construct_origin, selected_field + 1 + j)`.
        for (offset, field) in deferred.trailing_fields.iter().enumerate() {
            let field = self.child_occurrence(
                deferred.construct_origin,
                deferred.selected_field + 1 + offset,
                field,
            )?;
            let lowered = self.lower_expr(builder, field, deferred.producer_env)?;
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = lowered {
                return Ok(Err(trap));
            }
            // ⭐ These become `outer_scrutinee`'s constructor **template** below,
            // so this is a specialized-only surface, not a spine edge.
            constructor_args.push(lowered.specialized_at("a deferred constructor field")?);
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
                        LoweringOperand::Specialized(constructor_args[position].clone()),
                        frame.cases.to_vec(),
                        frame.default.clone(),
                        outer_tail.clone(),
                        frame.static_origin,
                        frame.provenance,
                        frame.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        deferred.selected_active.activation,
                        deferred.selected_active.cursor,
                        deferred.splice_caller,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(
                        Px8jProducerPath::DeferredConstructor,
                        &induction_hypothesis,
                    );
                    induction_hypotheses.push(induction_hypothesis);
                }
                induction_hypotheses
                    .extend(constructor_args.into_iter().map(LoweringOperand::Specialized));
                induction_hypotheses.extend(outer_tail);
                Ok(Ok(induction_hypotheses))
            }
            EliminatorFrame::Ordinary(frame) => {
                let (_case_index, case) = match select_ordinary_case(frame, deferred.constructor) {
                    Ok(selected) => selected,
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
                constructor_args.extend(specialized_env_at(
                    &outer_tail,
                    "a deferred constructor's trailing field",
                )?);
                Ok(Ok(constructor_args
                    .into_iter()
                    .map(LoweringOperand::Specialized)
                    .collect()))
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

    fn lower_source_machine(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringOperand],
        active: &ActiveContinuationFrame<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
        let control = SourceControl {
            continuation: SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected: active.cursor,
                active,
                root_authority,
            }),
            selected: SourceSelectedContinuation {
                activation: active.activation,
                cursor: active.cursor,
                parent: active.parent,
                pending: active.pending.to_vec(),
                selected_ancestry: active.selected_ancestry.to_vec(),
                selected_scope: active.selected_scope.cloned(),
            },
            selected_lineage: Vec::new(),
            terminal_outer: active.cursor,
        };
        self.lower_source_machine_with_continuation(
            builder,
            OwnedSourceOccurrence::cloned(occurrence),
            env.to_vec(),
            control,
        )
    }

    fn lower_source_machine_with_continuation<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let mut state = SourceMachineState::Eval { expr, env, control };
        loop {
            state = match state {
                SourceMachineState::Eval {
                    expr:
                        OwnedSourceOccurrence {
                            expr,
                            static_origin,
                        },
                    env,
                    mut control,
                } => match expr {
                    RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                        self.enter_checked_subcontinuation_frame(frame_id)?;
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedRecursiveInvocation {
                        call_template_id,
                        body,
                        ..
                    } => {
                        let instance =
                            self.enter_checked_recursive_invocation(call_template_id, &body)?;
                        control.continuation =
                            SourceContinuation::CheckedRecursiveInvocationReturn {
                                instance,
                                next: Box::new(control.continuation),
                            };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::CheckedComputationalIHInvocation {
                        call_template_id,
                        body,
                        ..
                    } => {
                        self.enter_checked_computational_ih_invocation(call_template_id)?;
                        control.continuation =
                            SourceContinuation::CheckedComputationalIHInvocationReturn {
                                call_template_id,
                                next: Box::new(control.continuation),
                            };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *body)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Value(value) => SourceMachineState::Value {
                        value: LoweringOperand::Specialized(self.lower_value(builder, &value)?),
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
                            body: self.owned_child_occurrence(static_origin, 1, *body)?,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *value)?,
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
                                value: LoweringOperand::Specialized(self.finish_source_constructor(
                                    builder,
                                    constructor,
                                    vec![],
                                )?),
                                control,
                            }
                        } else {
                            // Argument *i* is child *i*; the suffix keeps each
                            // pending term paired with its own origin, so the
                            // machine's positions cannot drift as it consumes them.
                            let first = args.remove(0);
                            let remaining = args
                                .into_iter()
                                .enumerate()
                                .map(|(offset, arg)| {
                                    self.owned_child_occurrence(static_origin, 1 + offset, arg)
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            control.continuation = SourceContinuation::ConstructArgument {
                                constructor,
                                remaining,
                                lowered: Vec::new(),
                                env: env.clone(),
                                next: Box::new(control.continuation),
                            };
                            SourceMachineState::Eval {
                                expr: self.owned_child_occurrence(static_origin, 0, first)?,
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
                            static_origin,
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *scrutinee)?,
                            env,
                            control,
                        }
                    }
                    RuntimeExpr::Call { callee, args } => {
                        let args = args
                            .into_iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                self.owned_child_occurrence(static_origin, 1 + position, arg)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        control.continuation = SourceContinuation::CallCallee {
                            args,
                            env: env.clone(),
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *callee)?,
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
                            static_origin,
                            provenance: self.mint_recursor_frame_provenance(),
                            checked_frame_id,
                            answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
                            next: Box::new(control.continuation),
                        };
                        SourceMachineState::Eval {
                            expr: self.owned_child_occurrence(static_origin, 0, *scrutinee)?,
                            env,
                            control,
                        }
                    }
                    // ⭐ The delegation point. Every form this dispatcher does not
                    // handle — closures included — goes to `lower_expr` here, and
                    // it now goes **as the same occurrence**: same term, same
                    // origin. This arm is why a "machine-only" subset could never
                    // have been threaded soundly.
                    other => SourceMachineState::Value {
                        value: self.lower_expr(
                            builder,
                            SourceOccurrence {
                                expr: &other,
                                static_origin,
                            },
                            &env,
                        )?,
                        control,
                    },
                },
                SourceMachineState::Value { value, mut control } => {
                    if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                        control.continuation = Self::discard_source_prefix(control.continuation);
                    }
                    match control.continuation {
                        SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                            return Ok(value);
                        }
                        SourceContinuation::Terminal(
                            SourceContinuationTerminal::ReturnToProducerHole {
                                stack,
                                resume_cursor,
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
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                return Ok(value);
                            }
                            source_active_cursor(
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
                            control.continuation = SourceContinuation::UnwindRecursorSegment {
                                stack,
                                resume_cursor,
                                next: Box::new(SourceContinuation::Terminal(
                                    SourceContinuationTerminal::ResumeOuter {
                                        expected,
                                        active,
                                        root_authority,
                                    },
                                )),
                            };
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
                            self.restore_root_terminal_authority(root_authority, expected)?;
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                return Ok(value);
                            }
                            return self.resume_active_continuation(builder, value, *active);
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(
                            edge,
                        )) => {
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                let failure = builder.ins().iconst(types::I64, -4);
                                builder.ins().return_(&[failure]);
                                return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
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
                            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                        }
                        SourceContinuation::LetBody { body, env, next } => {
                            control.continuation = *next;
                            if matches!(value, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                                SourceMachineState::Value { value, control }
                            } else if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                SourceMachineState::Value { value, control }
                            } else {
                                let body_env = env_with_operands([value], &env);
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
                            let value = self.finish_checked_computational_ih_marker(value)?;
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
                            if control.selected.activation != delimiter.activation
                                || control.selected.cursor != delimiter.cursor
                                || scope.scope_origin != delimiter.scope_origin
                                || scope.frame.checked_frame_id != delimiter.frame_id
                                || scope.frame.checked_invocation_id != delimiter.invocation_id
                            {
                                return Err(unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return delimiter does not match its open occurrence",
                                ));
                            }
                            let previous = control.selected_lineage.pop().ok_or_else(|| {
                                unsupported(
                                    "OrientedSubcontinuationPlanV1",
                                    "selected-case return has no exact parent control state",
                                )
                            })?;
                            control.selected = previous;
                            control.continuation = *next;
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::ApplyRecursorSelection { layer, next } => {
                            #[cfg(test)]
                            match layer.role {
                                RecursorLayerRole::SelectsOccurrence { origin } => {
                                    px8j_record_source_event(Px8jSourceTraceEvent::Selection {
                                        origin,
                                    });
                                }
                                RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
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
                                    static_origin: layer.static_origin,
                                    provenance: layer.provenance,
                                    checked_frame_id: layer.checked_frame_id,
                                    answer_route,
                                    next,
                                };
                            SourceMachineState::Value { value, control }
                        }
                        SourceContinuation::UnwindRecursorSegment {
                            mut stack,
                            resume_cursor,
                            next,
                        } => {
                            source_active_cursor(
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
                            if let Some(layer) = stack.later_wrappers_in_construction_order.pop() {
                                #[cfg(test)]
                                if let RecursorLayerRole::ExitsScope {
                                    origin,
                                    scope_origin,
                                    parent_scope,
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
                                control.continuation =
                                    SourceContinuation::ComputationalMatchScrutinee {
                                        cases: layer.cases,
                                        default: layer.default,
                                        env: layer.outer_env,
                                        static_origin: layer.static_origin,
                                        provenance: layer.provenance,
                                        checked_frame_id: layer.checked_frame_id,
                                        answer_route,
                                        next: Box::new(SourceContinuation::UnwindRecursorSegment {
                                            stack,
                                            resume_cursor,
                                            next,
                                        }),
                                    };
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
                            // ⭐ A source `Construct` builds a constructor
                            // **template**, so its arguments take the ruled
                            // fail-closed boundary here.
                            lowered.push(
                                value.specialized_at("a source constructor argument")?,
                            );
                            control.continuation = *next;
                            if remaining.is_empty() {
                                SourceMachineState::Value {
                                    value: LoweringOperand::Specialized(self.finish_source_constructor(
                                        builder,
                                        constructor,
                                        lowered,
                                    )?),
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
                            static_origin,
                            next,
                        } => {
                            control.continuation = *next;
                            match value {
                                LoweringOperand::Specialized(Lowered::BoundedNat(nat)) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder,
                                        nat,
                                        false,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::StructuralNat(nat)) => {
                                    return self.lower_source_bounded_nat_match(
                                        builder,
                                        BoundedNatV1::derived_from_validated(nat.value),
                                        true,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::Bool { value, known }) => {
                                    let true_case = cases.iter().enumerate().find(|(_, case)| {
                                        case.binders == 0
                                            && case.constructor.ends_with("::Bool::True")
                                    });
                                    let false_case = cases.iter().enumerate().find(|(_, case)| {
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
                                        let (index, case) =
                                            if selected { true_case } else { false_case };
                                        SourceMachineState::Eval {
                                            expr: self.owned_case_body_occurrence(
                                                static_origin,
                                                index,
                                                case.body.clone(),
                                            )?,
                                            env,
                                            control,
                                        }
                                    } else {
                                        let (true_index, true_case) = true_case;
                                        let (false_index, false_case) = false_case;
                                        let true_body = self.case_body_occurrence(
                                            static_origin,
                                            true_index,
                                            &true_case.body,
                                        )?;
                                        let false_body = self.case_body_occurrence(
                                            static_origin,
                                            false_index,
                                            &false_case.body,
                                        )?;
                                        return self.lower_source_dynamic_bool_match(
                                            builder,
                                            value,
                                            true_body,
                                            false_body,
                                            &env,
                                            control,
                                        );
                                    }
                                }
                                LoweringOperand::Specialized(Lowered::HostResult {
                                    success,
                                    error,
                                    ok,
                                    err_constructor,
                                    ok_constructor,
                                }) => {
                                    return self.lower_source_dynamic_host_result_match(
                                        builder,
                                        success,
                                        *error,
                                        *ok,
                                        &err_constructor,
                                        &ok_constructor,
                                        &cases,
                                        default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) => {
                                    return self.lower_source_dynamic_constructor_match(
                                        builder,
                                        dynamic,
                                        &cases,
                                        &default,
                                        static_origin,
                                        &env,
                                        control,
                                    );
                                }
                                LoweringOperand::Specialized(Lowered::Constructor { constructor, args }) => {
                                    let Some((case_index, case)) = cases
                                        .iter()
                                        .enumerate()
                                        .find(|(_, case)| case.constructor == constructor)
                                    else {
                                        return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
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
                                    let mut case_env = env_with(args, &[]);
                                    case_env.extend(env);
                                    SourceMachineState::Eval {
                                        expr: self.owned_case_body_occurrence(
                                            static_origin,
                                            case_index,
                                            case.body.clone(),
                                        )?,
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
                            static_origin,
                            provenance,
                            checked_frame_id,
                            answer_route,
                            next,
                        } => 'computational_scrutinee: {
                            // ⭐⭐ `AC-C4` — THE RESUMPTION POINT. An induction
                            // hypothesis over a carried child hands its word back
                            // as the machine's value, and it lands **here**: this
                            // continuation is what "resumes the same computational
                            // eliminator over that carried word" means on the
                            // source-machine route.
                            //
                            // ⛔ Taken before the specialized selection below,
                            // which reads `Lowered::Constructor` — a compile-time
                            // template the carried value does not have and must
                            // not be asked for. Without this arm the resumed word
                            // reaches `"source scrutinee is not a constructor
                            // value"`, which is a **true sentence about the wrong
                            // thing**: the value is fine, the question is.
                            if let LoweringOperand::Carried(word) = &value {
                                let word = *word;
                                let frame = ComputationalEliminatorFrame {
                                    cases: &cases,
                                    default: &default,
                                    env: &env,
                                    static_origin,
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
                                let eliminated = self
                                    .lower_carried_computational_match(builder, word, frame, &[])?;
                                control.continuation = *next;
                                break 'computational_scrutinee SourceMachineState::Value {
                                    value: eliminated,
                                    control,
                                };
                            }
                            let retained = value.clone();
                            #[cfg(test)]
                            let actual_constructor = match &value {
                                LoweringOperand::Specialized(Lowered::Constructor {
                                    constructor,
                                    ..
                                }) => Some(constructor.clone()),
                                LoweringOperand::Specialized(_) | LoweringOperand::Carried(_) => {
                                    None
                                }
                            };
                            let selected = match &value {
                                LoweringOperand::Specialized(Lowered::Constructor { constructor, .. }) => cases
                                    .iter()
                                    .enumerate()
                                    .find(|(_, case)| case.constructor == *constructor),
                                _ => None,
                            };
                            let (case_index, case) = if let Some(selected) = selected {
                                selected
                            } else if answer_route
                                == SourceComputationalAnswerRoute::CheckedSelectedRecursor
                                && matches!(&value, LoweringOperand::Specialized(Lowered::Constructor { .. }))
                                && px8tr_deforested_answer_route_enabled()
                            {
                                let mut returns = cases.iter().enumerate().filter(|(_, case)| {
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
                                let Some((return_index, return_case)) =
                                    return_case.filter(|(_, return_case)| {
                                        exact_return
                                            && exact_visible
                                            && source_case_has_no_checked_control_markers(
                                                &return_case.body,
                                            )
                                    })
                                else {
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
                                    return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
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
                                let case_env = env_with_operands([retained], &env);
                                control.continuation = *next;
                                let body = self.owned_case_body_occurrence(
                                    static_origin,
                                    return_index,
                                    return_case.body.clone(),
                                )?;
                                return self.lower_source_machine_with_continuation(
                                    builder,
                                    body,
                                    case_env,
                                    control,
                                );
                            } else {
                                if !matches!(&value, LoweringOperand::Specialized(Lowered::Constructor { .. })) {
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
                                return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
                            };
                            let LoweringOperand::Specialized(Lowered::Constructor { args, .. }) = value else {
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
                                static_origin,
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
                                        LoweringOperand::Specialized(
                                            args[position].clone(),
                                        ),
                                        cases.clone(),
                                        default.clone(),
                                        env.clone(),
                                        static_origin,
                                        provenance,
                                        frame.checked_frame_id,
                                        slot_template_id,
                                        producer_origin,
                                        position,
                                        RecursorLayerRole::SelectsOccurrence {
                                            origin: producer_origin,
                                        },
                                        activation,
                                        cursor,
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
                                retained.specialized_ref_at("an eliminator frame's scrutinee")?,
                            )? {
                                Ok(frame_env) => frame_env,
                                Err(trap) => return Ok(LoweringOperand::Specialized(Lowered::Trap(trap))),
                            };
                            let mut case_env = induction_hypotheses;
                            case_env.extend(args.into_iter().map(LoweringOperand::Specialized));
                            case_env.extend(frame_env);
                            let previous_selected = control.selected.clone();
                            let pending = std::mem::take(&mut control.selected.pending);
                            let selected_scope = OwnedSelectedScope {
                                scope_origin: producer_origin,
                                parent_scope: control
                                    .selected
                                    .selected_scope
                                    .as_ref()
                                    .map(|scope| scope.scope_origin),
                                frame: ComputationalRecursorFramePayload {
                                    cases: cases.clone(),
                                    default: default.clone(),
                                    outer_env: env.clone(),
                                    static_origin,
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
                                        cursor,
                                        scope_origin: selected_scope_ref.scope_origin,
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
                            control.selected = SourceSelectedContinuation {
                                activation,
                                cursor,
                                parent,
                                pending,
                                selected_ancestry: ancestry,
                                selected_scope,
                            };
                            control.selected_lineage.push(previous_selected);
                            let body = self.owned_case_body_occurrence(
                                static_origin,
                                case_index,
                                case.body.clone(),
                            )?;
                            SourceMachineState::Eval {
                                expr: body,
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
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let zero = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_zero && case.binders == 0
        });
        let suc = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_suc && case.binders == 1
        });
        let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) else {
            return Err(unsupported(
                "BoundedNat",
                "structural Nat source match requires exact Zero and Suc predecessor arms",
            ));
        };
        let zero_body = self.case_body_occurrence(static_origin, zero_index, &zero.body)?;
        let suc_body = self.case_body_occurrence(static_origin, suc_index, &suc.body)?;

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

        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for (arm_name, block, case_body, predecessor) in [
            ("Zero", zero_block, zero_body, None),
            ("Suc", suc_block, suc_body, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let arm_env = predecessor
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
            let mut arm_env = env_with(arm_env, &[]);
            arm_env.extend_from_slice(env);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                OwnedSourceOccurrence::cloned(case_body),
                arm_env,
                branch_control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                let detail = match &lowered {
                    LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                        format!("Trap({}: {:?})", trap.message, trap.code)
                    }
                    LoweringOperand::Specialized(other) => lowered_value_kind(other).to_string(),
                    // ⛔ No wildcard: a carried operand reaching a join
                    // diagnostic must name itself, not fall into `other`.
                    LoweringOperand::Carried(_) => "BoundaryCarrier".to_string(),
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
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
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
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, LoweringOperand::Specialized(merged), suffix_active)
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
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        self.consumed_subcontinuation_frames = frame_baseline.clone();
        let lowered = self.lower_source_machine_with_continuation(builder, expr, env, control)?;
        frame_union.extend(self.consumed_subcontinuation_frames.iter().copied());
        Ok(lowered)
    }

    fn lower_source_dynamic_bool_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        condition: cranelift_codegen::ir::Value,
        true_body: SourceOccurrence<'_>,
        false_body: SourceOccurrence<'_>,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
        for (predecessor_id, block, body) in
            [(0, true_block, true_body), (1, false_block, false_body)]
        {
            builder.switch_to_block(block);
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let branch_control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                OwnedSourceOccurrence::cloned(body),
                env.to_vec(),
                branch_control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
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
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
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
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, LoweringOperand::Specialized(merged), suffix_active)
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
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
                    block: merge,
                    expected_outer: suffix_control.terminal_outer,
                    required_kind,
                    terminal_active_prefix: prefix,
                }
            }
        };
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
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = if let Some((index, case)) = cases
                .iter()
                .enumerate()
                .find(|(_, case)| case.constructor == constructor && case.binders == 1)
            {
                let arm_env = env_with([payload], env);
                let body =
                    self.owned_case_body_occurrence(static_origin, index, case.body.clone())?;
                self.lower_forked_branch(
                    builder,
                    &frame_baseline,
                    &mut frame_union,
                    body,
                    arm_env,
                    branch_control,
                )?
            } else {
                // ⚠ THE ONE SYNTHESIZED TERM in the whole lowering: no source
                // occurrence exists for "this alternative has no case", so the
                // machine is handed a fresh `Trap` built from the match's own
                // `default`. `default` is an ATOM of the match occurrence, not a
                // child of it, so the honest origin for this term is the match
                // occurrence's own — and `Trap` is a leaf, so no child is ever
                // derived from it. ⛔ Do not mint an origin here.
                self.lower_forked_branch(
                    builder,
                    &frame_baseline,
                    &mut frame_union,
                    OwnedSourceOccurrence {
                        expr: RuntimeExpr::Trap(default.clone()),
                        static_origin,
                    },
                    env.to_vec(),
                    branch_control,
                )?
            };
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
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
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
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
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, LoweringOperand::Specialized(merged), suffix_active)
    }

    fn lower_source_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
                static_origin,
                env,
                suffix_control,
            );
        }
        self.lower_source_planned_dynamic_constructor_match(
            builder,
            dynamic,
            cases,
            default,
            static_origin,
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
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
            let (case_index, case) =
                match select_dynamic_constructor_case(cases, &alternative, default)? {
                    Ok(selected) => selected,
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
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                self.owned_case_body_occurrence(static_origin, case_index, case.body.clone())?,
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
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
        Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge))
    }

    fn lower_source_planned_dynamic_constructor_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
            block: merge,
            expected_outer: suffix_control.terminal_outer,
            required_kind,
            terminal_active_prefix: prefix,
        };
        let (source_prefix_template, terminal) =
            Self::split_source_prefix(suffix_control.continuation)?;
        let root_authority = match terminal {
            SourcePrefixTerminal::ResumeOuter { root_authority } => root_authority,
            SourcePrefixTerminal::Join(_) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "planned dynamic-constructor cut unexpectedly inherited an executable edge",
                ));
            }
        };
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
            let (case_index, case) =
                match select_dynamic_constructor_case(cases, &alternative, default)? {
                    Ok(selected) => selected,
                    Err(_) => {
                        let failure = builder.ins().iconst(types::I64, -4);
                        builder.ins().return_(&[failure]);
                        test_block = next;
                        continue;
                    }
                };
            let edge = self.mint_source_predecessor(target.clone());
            let continuation =
                Self::instantiate_source_prefix_template(&source_prefix_template, edge)?;
            let control = SourceControl {
                continuation,
                selected: suffix_control.selected.clone(),
                selected_lineage: suffix_control.selected_lineage.clone(),
                terminal_outer: suffix_control.terminal_outer,
            };
            let lowered = self.lower_forked_branch(
                builder,
                &frame_baseline,
                &mut frame_union,
                self.owned_case_body_occurrence(static_origin, case_index, case.body.clone())?,
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if Self::seal_source_trap_branch(builder, &lowered) {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(lowered, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
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
            cursor: suffix_control.selected.cursor,
            parent: suffix_control.selected.parent,
            pending: &suffix_pending,
            selected_ancestry: &suffix_control.selected.selected_ancestry,
            source_lineage: &suffix_control.selected_lineage,
            source_selected_cursor: Some(suffix_control.selected.cursor),
            selected_scope: suffix_control.selected.selected_scope.as_ref(),
        };
        self.restore_root_terminal_authority(root_authority, suffix_control.terminal_outer)?;
        self.resume_active_continuation(builder, LoweringOperand::Specialized(merged), suffix_active)
    }

    fn source_call_state<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        callee: LoweringOperand,
        args: Vec<LoweringOperand>,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        // ⭐ A call needs a **callable template** — `params`, `captures`, a body
        // occurrence. A carried boundary word carries none of those and cannot
        // acquire them (`§2g`: the carrier holds the SSA word and nothing else),
        // so this is a specialized-only surface. ⛔ Fails closed.
        let callee = callee.specialized_at("a source-machine call's callee")?;
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
                extend_specialized(&mut call_env, captures);
                call_env.extend(env);
                Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                    expr: self.machine_body_occurrence(body)?,
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
                let body = self.machine_body_occurrence(body)?;
                self.lower_source_declaration_call(
                    builder, symbol, captures, body, args, env, control,
                )
            }
            mut recursor @ Lowered::ComputationalRecursorClosure { .. } => {
                let checked_ih_invocation =
                    self.mint_checked_computational_ih_instance(&mut recursor)?;
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
                let (base, boundary) = decompose_computational_recursor(
                    LoweringOperand::Specialized(recursor),
                );
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
                // ⭐⭐ `AC-C4` — the carried residual on the source-machine
                // route. ⚠ This is the site where "installs the ALREADY-CHECKED
                // invocation segment" is literal: the refusal below runs
                // **before** `install_recursor_invocation`, which is exactly the
                // ordering control 5 measures.
                if let LoweringOperand::Carried(word) = base {
                    Self::reject_carried_residual_arguments(args.len())?;
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: LoweringOperand::Carried(word),
                        control: suspended,
                    }));
                }
                let base = base.specialized_at("a recursor residual in a source call")?;
                if let Lowered::BoundedNat(predecessor) = base {
                    if !args.is_empty() {
                        return Err(unsupported(
                            "BoundedNat",
                            "structural Nat recursive hypothesis takes no arguments",
                        ));
                    }
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: LoweringOperand::Specialized(Lowered::BoundedNat(predecessor)),
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
                    extend_specialized(&mut call_env, captures);
                    call_env.extend(env);
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                        expr: self.machine_body_occurrence(body)?,
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
        body: OwnedSourceOccurrence,
        args: Vec<LoweringOperand>,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(&symbol)?;
        if !self.declaration_is_recursive(&symbol) {
            let mut call_env = args;
            extend_specialized(&mut call_env, captures);
            call_env.extend(env);
            return Ok(SourceCallOutcome::Continue(SourceMachineState::Eval {
                expr: body,
                env: call_env,
                control,
            }));
        }

        // ⭐ Past this point the call is genuinely recursive, and its arguments
        // become the **loop header's representation** — compared across
        // iterations by `same_recursive_argument_shapes` and lowered into block
        // params. A carried boundary word has no such shape, so this is a
        // specialized-only surface with the ruled fail-closed arm.
        //
        // ⚠ The boundary sits HERE and not at the parameter, because the
        // non-recursive direct call above forwards `args` into `call_env`
        // untouched — that path stays phase-preserving and must not be made to
        // fail closed for a property only the loop needs.
        let args = specialized_env_at(&args, "a recursive source-declaration argument")?;
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
                    value: LoweringOperand::Specialized(induction),
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
            return Ok(SourceCallOutcome::Complete(LoweringOperand::Specialized(Lowered::RecursiveBackedge)));
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
        let mut call_env = loop_args
            .into_iter()
            .rev()
            .map(LoweringOperand::Specialized)
            .collect::<Vec<_>>();
        extend_specialized(&mut call_env, captures);
        call_env.extend(env);
        let lowered = self.lower_source_machine_with_continuation(builder, body, call_env, control);
        self.active_recursive_declarations.pop();
        Ok(SourceCallOutcome::Complete(lowered?))
    }

    /// Resolves a retained closure body's static origin back to its source term.
    ///
    /// ⭐ **This is the only `origin -> expression` lookup in the backend, and the
    /// only place `StaticTransitionPlan::source_occurrence` is called.**
    /// `RT-FNSPLIT-B2A-C` shipped a pin (its N3) asserting that no such lookup
    /// existed, because at that point the origin was provenance and a lookup would
    /// have been an unaudited second authority. B2A-S **retires that pin on
    /// purpose** and replaces it with the opposite one: exactly one lookup,
    /// reachable from exactly one consumer. Zero to one, never zero to unbounded —
    /// see `exactly_one_plan_origin_to_expression_lookup_exists`.
    ///
    /// ⛔ Do not call the plan's resolver anywhere else, and do not widen this to
    /// take anything but an origin. The moment a caller can pass a term, a
    /// pointer, or a hash alongside the tag, the tag has stopped being the
    /// authority and this is decoration again.
    ///
    /// ⚠ Stated precisely, **with its window**, because the two counts differ and
    /// conflating them would overclaim: there is **one** lookup (this function),
    /// and `grep -c 'self.retained_body_occurrence('` in this file returns
    /// **eight** — **seven** retained-closure consumption sites (application,
    /// resume, declaration-call) plus **one** internal composition by
    /// `machine_body_occurrence`, which is this function's own caller rather than
    /// a further consumer.
    ///
    /// Those seven do not share a single lowering *entry point*, because a retained
    /// body is lowered by whichever specialized path the call shape selects. What
    /// makes selection closed is therefore not one caller but that
    /// `Lowered::Closure`/`DeclarationClosure` carry only a tag — so this is the
    /// *only* way any of them can reach a term at all.
    fn retained_body_occurrence(
        &self,
        static_origin: StaticOriginId,
    ) -> Result<SourceOccurrence<'a>, CraneliftBackendError> {
        Ok(SourceOccurrence {
            expr: self
                .static_transition_plan
                .source_occurrence(static_origin)?,
            static_origin,
        })
    }

    /// The source machine's **owned working copy** of a retained body.
    ///
    /// ⚠ The machine's pending frames own their terms (`OwnedSourceOccurrence`)
    /// and must keep doing so — this is the population boundary of B2A-S, and it
    /// is forced rather than chosen. `lower_source_forked_match` hands the machine
    /// a **synthesized** `RuntimeExpr::Trap` that exists nowhere in the source
    /// tree and therefore has no planned occurrence to be resolved from; a frame
    /// that could only hold a borrowed view of a planned term could not represent
    /// it. So the frames stay owned, and this is where a tag becomes one.
    ///
    /// ⛔ That is **not** a surviving retained-body carrier. The distinction is
    /// which value is authoritative: a `Lowered::Closure` names its body by origin
    /// and holds no term, and this copy is made *at the point of use* from that
    /// name. Re-lowering the resolved term per call site is symptom-inventory
    /// entry 2, which `RT-FNSPLIT-B2F` owns and this WP does not claim.
    fn machine_body_occurrence(
        &self,
        static_origin: StaticOriginId,
    ) -> Result<OwnedSourceOccurrence, CraneliftBackendError> {
        Ok(OwnedSourceOccurrence::cloned(
            self.retained_body_occurrence(static_origin)?,
        ))
    }

    /// Derives one **positional** child occurrence of `parent`.
    ///
    /// This is the lowering's sole route to a child's static name. `position` is
    /// the child's source-field ordinal in the planner's own child order (see the
    /// table on `lower_expr`), and the value comes out of B1R's checked
    /// positional child-origin range. There is deliberately no other route: not
    /// pointer identity, not the term's content or hash, not clone order, not
    /// visit order, and no arithmetic that mints an origin
    /// for it.
    fn child_occurrence<'x>(
        &self,
        parent: StaticOriginId,
        position: usize,
        child: &'x RuntimeExpr,
    ) -> Result<SourceOccurrence<'x>, CraneliftBackendError> {
        Ok(SourceOccurrence {
            expr: child,
            static_origin: self
                .static_transition_plan
                .child_static_origin(parent, position)?,
        })
    }

    /// The owned form of `child_occurrence`, for the source machine's pending
    /// frames: it takes the child term **by value** and pairs it with its origin
    /// in one constructor, so no step of the machine can hold a term whose origin
    /// was dropped was dropped.
    fn owned_child_occurrence(
        &self,
        parent: StaticOriginId,
        position: usize,
        child: RuntimeExpr,
    ) -> Result<OwnedSourceOccurrence, CraneliftBackendError> {
        Ok(OwnedSourceOccurrence {
            expr: child,
            static_origin: self
                .static_transition_plan
                .child_static_origin(parent, position)?,
        })
    }

    /// The owned form of `case_body_occurrence`, for the source machine.
    fn owned_case_body_occurrence(
        &self,
        parent: StaticOriginId,
        index: usize,
        body: RuntimeExpr,
    ) -> Result<OwnedSourceOccurrence, CraneliftBackendError> {
        self.owned_child_occurrence(parent, 1 + index, body)
    }

    /// Derives the occurrence of case *index*'s body under a match occurrence.
    ///
    /// Both match variants lay their children out as `[scrutinee, case 0 body,
    /// case 1 body, …]`, so a case body is child `1 + index`. Cases are the one
    /// place the lowering reaches a body by *searching* (by constructor name),
    /// and a search recovers no position — so every such site enumerates to
    /// recover the index rather than deriving identity from the match it found.
    fn case_body_occurrence<'x>(
        &self,
        parent: StaticOriginId,
        index: usize,
        body: &'x RuntimeExpr,
    ) -> Result<SourceOccurrence<'x>, CraneliftBackendError> {
        self.child_occurrence(parent, 1 + index, body)
    }

    /// ⭐ **The dual of [`LoweringOperand::specialized_join_arm`]** — a join
    /// whose single lane is the **carrier word**, not a native scalar pair.
    ///
    /// ⚠⚠ **Read the two together: they refuse in OPPOSITE directions, and the
    /// asymmetry is the point.** `specialized_join_arm` guards a join that has
    /// no carried lane, so its `Carried` arm fails closed. This guards a join
    /// that has *only* a carried lane, so a **specialized** arm must cross into
    /// it. ⛔ Neither is a `Carried -> Lowered` conversion; this one moves
    /// `Lowered -> CarriedBoundaryWord`, which is precisely the direction `§2g`
    /// rules as the producer's one-way seam.
    ///
    /// ⭐ **Why a carried match's join has one lane and it is this one.** An arm
    /// of a carried `Match` may return a **projected child**, which `§2g`
    /// requires to stay `Carried` and which has no compile-time template to
    /// re-specialize. So the merge cannot be a `Lowered` join, and every arm
    /// must arrive as a carrier word.
    ///
    /// ⚠ **The producer's coverage is partial and this inherits that
    /// deliberately.** An arm whose value is a form `transfer_into_carrier`
    /// defers — a spillable `Int`, a `String`, borrowed ingress — fails closed
    /// with **the producer's own message**, ⛔ never a second refusal invented
    /// here. ⇒ The carried match's arm coverage widens exactly when the
    /// producer's does, with no list to keep in sync and no second authority to
    /// let drift.
    fn carried_join_arm(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        lowered: LoweringOperand,
        join: &'static str,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        match lowered {
            LoweringOperand::Carried(word) => Ok(word),
            // ── ⛔ DEFERRED, said plainly ──────────────────────────────────
            //
            // ⚠ A deferral is honest; a deferral that reads as delivery is not.
            // A compile-time trap arm must **not** reach the merge at all — it
            // returns instead (`seal_source_trap_branch`) — so the merge block
            // would have fewer predecessors than the case chain has arms. That
            // is a control-flow shape this route does not build yet, and
            // refusing is strictly better than emitting a half-formed merge.
            LoweringOperand::Specialized(Lowered::Trap(trap)) => Err(unsupported(
                "BoundaryCarrier",
                format!(
                    "{join} resolves at compile time to a trap ({}), and the carried join \
                     does not yet build a merge with a trapping predecessor",
                    trap.message
                ),
            )),
            LoweringOperand::Specialized(lowered) => {
                self.transfer_into_carrier(builder, origin, &lowered)
            }
        }
    }

    /// ⭐⭐ **`D3` — `Match` eliminating a value that has NO compile-time
    /// template.** This is the second, *executable* route the whole node exists
    /// to build.
    ///
    /// The specialized route answers *"which constructor?"* by reading a
    /// `Lowered::Constructor`'s own `constructor` field while compiling. Here
    /// there is no such value and no such field — only a boundary word — so
    /// **every** question becomes a call into the emitted carrier ABI:
    ///
    /// | question | specialized route | this route |
    /// |---|---|---|
    /// | which constructor? | `case.constructor == constructor` | `tag(word)` vs `case_constructor_identity` |
    /// | how many children? | `args.len()` | `field_count(word)` |
    /// | child *i*? | `args[i]` | `field(word, i)` — ⭐ **stays `Carried`** |
    /// | nothing matched? | a compile-time `Lowered::Trap` | a **runtime** closed default |
    ///
    /// ⭐ **Both columns read ONE identity authority** (`D2`). The producer
    /// wrote `constructor_symbol_identity(..).tag_abi_word()`; this compares
    /// against `case_constructor_identity(..).tag_abi_word()`, and equal
    /// spellings intern to one canonical span, so the two agree **because they
    /// are the same number**, not because two derivations happen to coincide.
    /// ⛔ There is no decode step and no reverse table: the comparison is word
    /// against word, ⛔ never word against a reconstructed name.
    ///
    /// ⚠ **This changes no production behaviour today.** Nothing in production
    /// emits a `Carried` scrutinee (`AC-C10` — zero `B2F` activation), so this
    /// route is reached only by a test that seeds one. Stated here so the
    /// reachability is not overclaimed by a later reader.
    fn lower_carried_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐ Handled before any block is created, and that ordering matters: a
        // case-free match reaches the default unconditionally, so building a
        // merge block for it would leave one with no predecessor.
        if cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        }

        // Read identity and arity ONCE, ahead of the chain: both are properties
        // of the scrutinee, not of any case, and re-reading per case would be a
        // second answer to a question that has one.
        let tag = self.emit_carrier_tag(builder, scrutinee)?;
        let field_count = self.emit_carrier_field_count(builder, scrutinee)?;

        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);

        for (index, case) in cases.iter().enumerate() {
            // ⭐ `D1` — the case's identity, keyed on this `Match` occurrence's
            // origin and the case's ordinal. ⚠ `case.constructor`, the
            // **string**, is deliberately not the key: keying on the spelling
            // would be the second derivation `D2` forbids.
            let identity = self
                .static_transition_plan
                .case_constructor_identity(static_origin, index)?
                .tag_abi_word()?;
            let identity = Self::carrier_identity_immediate(builder, identity);
            let selected = builder.create_block();
            let next = builder.create_block();
            let matched = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                identity,
            );
            builder.ins().brif(matched, selected, &[], next, &[]);

            builder.switch_to_block(selected);
            // ⚠ **The arity check the specialized route performs while
            // compiling has to be EMITTED here**, because neither operand is
            // known until the value exists. It is a real guard, not ceremony:
            // binding *n* binders over a node with fewer children would read
            // past the node. A mismatch means the producer's `field_count` and
            // the elaborator's binder count disagree — corruption, not an input
            // condition — so it takes the same failure status as every other
            // carrier ABI violation.
            let binders = i64::try_from(case.binders).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a case binds more binders than the carrier ABI can count",
                )
            })?;
            Self::require_i64(builder, field_count, binders);

            // ⭐ `§2g`: *"projected children remain `Carried`."* Each binder is
            // a runtime projection, and it enters `case_env` **in the carried
            // phase** — which is the exact clause `§2h`'s control demands.
            let mut bindings = Vec::with_capacity(case.binders);
            for position in 0..case.binders {
                bindings.push(LoweringOperand::Carried(
                    self.emit_carrier_field(builder, scrutinee, position)?,
                ));
            }
            let case_env = env_with_operands(bindings, env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let body_origin = body.static_origin;
            let lowered = self.lower_expr(builder, body, &case_env)?;
            let word = self.carried_join_arm(
                builder,
                body_origin,
                lowered,
                "a carried `Match` arm",
            )?;
            builder.ins().jump(merge, &[word.word.into()]);

            builder.switch_to_block(next);
        }

        // ── ⛔ THE CLOSED DEFAULT — `AC-C3`'s negative arm ─────────────────
        //
        // ⭐ Routed through the existing [`Self::seal_source_trap_branch`]
        // rather than spelling the trap encoding a second time: if the encoding
        // ever changes, both move together. A constructor outside the
        // artifact-static case set lands here, at runtime, and returns.
        let defaulted = LoweringOperand::Specialized(Lowered::Trap(default.clone()));
        if !Self::seal_source_trap_branch(builder, &defaulted) {
            return Err(unsupported(
                "Match",
                "the carried match's closed default did not seal its branch",
            ));
        }

        builder.switch_to_block(merge);
        Ok(LoweringOperand::Carried(CarriedBoundaryWord {
            word: builder.block_params(merge)[0],
        }))
    }

    /// ⭐⭐ **`D3` — `ComputationalMatch` eliminating a carried value.**
    ///
    /// Structurally the same three runtime questions as
    /// [`Self::lower_carried_match`] — identity, arity, positional child — over
    /// the same one authority. The differences are the computational frame's:
    /// the arity compared is `argument_binders`, the frame contributes its own
    /// environment, and a case may declare **recursive positions**.
    ///
    /// ## ⭐⭐ Recursive positions are BUILT here — `AC-C4`, on the Architect's
    /// ## single-field license
    ///
    /// A recursive position builds an *induction hypothesis* over the child at
    /// that position. Over a carried scrutinee that child is a **carried word**,
    /// so the IH's residual must hold one — which
    /// [`Lowered::ComputationalRecursorClosure::residual`] now does, as a
    /// `Box<LoweringOperand>`.
    ///
    /// ⚠ **This function previously refused recursive positions and named the
    /// fork for the Architect. That refusal was SCAFFOLD, and the ruling
    /// rejected the branch it was holding open** — *"the recursive-position
    /// refusal is not an acceptable `C1` residual."* It is recorded here rather
    /// than deleted silently, because the arm and a shipped boundary are
    /// textually the same thing and only the prose says which one this is.
    ///
    /// ⭐ **The metadata is MINTED exactly as the specialized composed path
    /// mints it, and ⛔ none of it is derived from the carried word.** Static
    /// origin, checked-frame id, IH slot templates, activation, cursor and
    /// producer origin all come from `eliminator` and the compiler's own
    /// counters — the carried word contributes the *value* and nothing else.
    /// That separation is the ruling's clause 5, and it is what control 3
    /// perturbs.
    fn lower_carried_computational_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        eliminator: ComputationalEliminatorFrame<'_>,
        remaining_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⛔⛔ TERMINATION — refused BEFORE any block is created, so a hang can
        // never be half-emitted. See
        // `Lowering::active_carried_computational_eliminations` for why inlining
        // a carried recursion cannot terminate.
        //
        // ⚠ **This bounds INVOKING an induction hypothesis, ⛔ not declaring a
        // recursive position.** A case with `recursive_positions` still mints
        // its IH over the carried child, still puts it in `case_env`, and still
        // eliminates — everything below runs. Only re-entering *this same*
        // eliminator refuses.
        if self
            .active_carried_computational_eliminations
            .contains(&eliminator.static_origin)
        {
            return Err(unsupported(
                "BoundaryCarrier",
                "a carried induction hypothesis resumed the same computational \
                 eliminator, and inlining that recursion cannot terminate: the \
                 residual is a runtime word, so no operand shrinks at compile \
                 time and the recursive case re-emits itself without bound. \
                 The invocation half is RT-FNSPLIT-B2F's, which emits one \
                 closed recursively callable target per static \
                 computational-eliminator origin; until it lands, a carried \
                 induction hypothesis is built and eliminated but never called",
            ));
        }
        self.active_carried_computational_eliminations
            .push(eliminator.static_origin);
        let lowered = self.lower_carried_computational_match_inner(
            builder,
            scrutinee,
            eliminator,
            remaining_eliminators,
        );
        let popped = self.active_carried_computational_eliminations.pop();
        debug_assert_eq!(
            popped,
            Some(eliminator.static_origin),
            "the carried elimination stack must unwind in the order it was pushed"
        );
        lowered
    }

    fn lower_carried_computational_match_inner(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        eliminator: ComputationalEliminatorFrame<'_>,
        remaining_eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        if eliminator.cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(
                eliminator.default.clone(),
            )));
        }
        // ⛔ A deferred constructor case rebuilds a `Lowered::Constructor`
        // *around* the scrutinee, which needs a compile-time template for the
        // parent. Refused rather than approximated.
        if eliminator.deferred_constructor_case.is_some() {
            return Err(unsupported(
                "BoundaryCarrier",
                "a carried scrutinee reached a deferred constructor case, which \
                 reconstructs a compile-time constructor around the eliminated value",
            ));
        }

        let tag = self.emit_carrier_tag(builder, scrutinee)?;
        let field_count = self.emit_carrier_field_count(builder, scrutinee)?;

        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);

        for (index, case) in eliminator.cases.iter().enumerate() {
            // ⛔ Malformed recursive positions are rejected before any code is
            // emitted for this case, exactly as the specialized composed path
            // rejects them. ⚠ The bound is `argument_binders` — the case's own
            // declared arity — and ⛔ NOT anything read off the carried word:
            // the word's `field_count` is checked against that same arity
            // below, at runtime, which is where a disagreement belongs.
            let mut seen = BTreeSet::new();
            for position in case.recursive_positions.iter().copied() {
                if !seen.insert(position) || position >= case.argument_binders {
                    return Err(unsupported(
                        "ComputationalMatch",
                        format!(
                            "case {} has malformed recursive position {position}",
                            case.constructor
                        ),
                    ));
                }
            }
            let identity = self
                .static_transition_plan
                .case_constructor_identity(eliminator.static_origin, index)?
                .tag_abi_word()?;
            let identity = Self::carrier_identity_immediate(builder, identity);
            let selected = builder.create_block();
            let next = builder.create_block();
            let matched = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                identity,
            );
            builder.ins().brif(matched, selected, &[], next, &[]);

            builder.switch_to_block(selected);
            let binders = i64::try_from(case.argument_binders).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a case binds more constructor arguments than the carrier ABI can count",
                )
            })?;
            Self::require_i64(builder, field_count, binders);

            let mut children = Vec::with_capacity(case.argument_binders);
            for position in 0..case.argument_binders {
                // ⭐ `§2g` — the projected child stays `Carried` into `case_env`.
                children.push(LoweringOperand::Carried(
                    self.emit_carrier_field(builder, scrutinee, position)?,
                ));
            }

            // ── ⭐⭐ `AC-C4` — the induction hypotheses over carried children ──
            //
            // ⚠ **Order is load-bearing and matches the specialized composed
            // path exactly:** `[IHs, reversed] ++ [children] ++ [frame env]`.
            // De Bruijn indices in the case body are positional, so a different
            // order here would silently rebind every recursive-position body.
            let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
            let mut active_scope = None;
            if !case.recursive_positions.is_empty() {
                let ih_slots =
                    self.computational_ih_slots_for_case(case, eliminator.checked_frame_id)?;
                let activation = self.mint_continuation_activation();
                let cursor = self.mint_continuation_cursor();
                let producer_origin = self.mint_recursor_producer_origin();
                let splice_caller = active_recursor_frame(remaining_eliminators);
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
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    // ⭐ Clause 1 — the CARRIED arm passes its projected operand
                    // **directly**. ⛔ No wrap, no `specialized_at`, no template.
                    let induction_hypothesis = self.make_computational_recursor(
                        children[position].clone(),
                        eliminator.cases.to_vec(),
                        eliminator.default.clone(),
                        eliminator.env.to_vec(),
                        eliminator.static_origin,
                        eliminator.provenance,
                        eliminator.checked_frame_id,
                        slot_template_id,
                        producer_origin,
                        position,
                        RecursorLayerRole::SelectsOccurrence {
                            origin: producer_origin,
                        },
                        activation,
                        cursor,
                        splice_caller,
                        None,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(
                        Px8jProducerPath::Composed,
                        &induction_hypothesis,
                    );
                    induction_hypotheses.push(induction_hypothesis);
                }
                active_scope = Some((activation, cursor, producer_origin, splice_caller));
            }

            let mut case_env = induction_hypotheses;
            case_env.extend(children);
            // The frame's own environment, with the retained scrutinee inserted
            // where the frame asked for it. ⭐ Retention is phase-preserving:
            // the retained value is the **same carried word**, ⛔ never a
            // materialized template of it.
            let mut frame_env = eliminator.env.to_vec();
            if let Some(retained) = eliminator.retained_scrutinee_index {
                if retained > frame_env.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "retained scrutinee index exceeds the frame environment",
                    ));
                }
                frame_env.insert(retained, LoweringOperand::Carried(scrutinee));
            }
            case_env.extend(frame_env);

            let body =
                self.case_body_occurrence(eliminator.static_origin, index, &case.body)?;
            let body_origin = body.static_origin;
            let lowered = if let Some((activation, cursor, producer_origin, splice_caller)) =
                active_scope
            {
                // ⭐ A case with recursive positions descends through the SOURCE
                // MACHINE, as the specialized composed path does — the body's IH
                // call needs a live continuation to resume into, and that is the
                // machinery that supplies one. ⛔ The only difference between
                // this block and its specialized twin is the phase of the
                // children; every identity below is the frame's.
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
                let selected_scope = OwnedSelectedScope {
                    scope_origin: producer_origin,
                    parent_scope: splice_caller
                        .and_then(|active| active.selected_scope)
                        .map(|scope| scope.scope_origin),
                    frame: ComputationalRecursorFramePayload {
                        cases: eliminator.cases.to_vec(),
                        default: eliminator.default.clone(),
                        outer_env: eliminator.env.to_vec(),
                        static_origin: eliminator.static_origin,
                        provenance: eliminator.provenance,
                        checked_frame_id: eliminator.checked_frame_id,
                        checked_invocation_id: eliminator.checked_invocation_id,
                        checked_invocation_source: eliminator.checked_invocation_source,
                        checked_invocation_depth: eliminator.checked_invocation_depth,
                    },
                };
                let active_state = ActiveContinuationFrame {
                    activation,
                    cursor,
                    parent: splice_caller.and_then(|active| active.parent),
                    pending: &pending,
                    selected_ancestry: &selected_ancestry,
                    source_lineage: splice_caller
                        .map(|active| active.source_lineage)
                        .unwrap_or(&[]),
                    source_selected_cursor: splice_caller
                        .and_then(|active| active.source_selected_cursor),
                    selected_scope: Some(&selected_scope),
                };
                self.lower_source_machine(builder, body, &case_env, &active_state)?
            } else if remaining_eliminators.is_empty() {
                self.lower_expr(builder, body, &case_env)?
            } else {
                self.lower_computational_producer_expr(
                    builder,
                    body,
                    &case_env,
                    remaining_eliminators,
                )?
            };
            let word = self.carried_join_arm(
                builder,
                body_origin,
                lowered,
                "a carried `ComputationalMatch` arm",
            )?;
            builder.ins().jump(merge, &[word.word.into()]);

            builder.switch_to_block(next);
        }

        let defaulted =
            LoweringOperand::Specialized(Lowered::Trap(eliminator.default.clone()));
        if !Self::seal_source_trap_branch(builder, &defaulted) {
            return Err(unsupported(
                "ComputationalMatch",
                "the carried computational match's closed default did not seal its branch",
            ));
        }

        builder.switch_to_block(merge);
        Ok(LoweringOperand::Carried(CarriedBoundaryWord {
            word: builder.block_params(merge)[0],
        }))
    }

    /// Lowers one source occurrence.
    ///
    /// ## The per-variant child-position table
    ///
    /// ⭐ **A child's position is its index in the `children: &[StaticNodeId]`
    /// slice the planner hands to `expression_node` / `expression_seed`** — *not*
    /// the `ordinal` parameter of `plan_expr`, which keys the frame's syntax and
    /// path stores instead. Reading positions off the `plan_expr` call sites
    /// gives the wrong table for every multi-child variant.
    ///
    /// | variant | positions (planner `children` order) |
    /// |---|---|
    /// | `CheckedJoinSite` / `CheckedSubcontinuationFrame` / `CheckedRecursiveInvocation` / `CheckedComputationalIHSlots` / `CheckedComputationalIHInvocation` | `0` = body |
    /// | `Value` / `Var` / `DeclarationRef` / `ImportedDeclarationRef` / `Trap` | no expression children |
    /// | `Let` | `0` = value, `1` = body |
    /// | `If` | `0` = scrutinee, `1` = then, `2` = else |
    /// | `PrimitiveCall` / `Construct` | `i` = `args[i]` |
    /// | `Record` | `i` = `fields[i]`'s value |
    /// | `Project` | `0` = record |
    /// | `Match` | `0` = scrutinee, `1 + i` = `cases[i].body` |
    /// | `ComputationalMatch` | `0` = scrutinee, `1 + i` = `cases[i].body` — ⚠ and it is the **sole** variant whose `entry != occurrence.node` (second axis below) |
    /// | `Closure` | `0` = body |
    /// | `LexicalClosure` | ⚠ `0` = **body**, `1 + i` = `captures[i]` |
    /// | `Call` | `0` = callee, `1 + i` = `args[i]` |
    /// | `Effect` | ⚠ capability present: `0` = `capability.value`, `1 + i` = `args[i]`; absent: `i` = `args[i]` |
    ///
    /// The planner's order and this walk's traversal **agree** on every variant.
    /// Two of them disagree with *declaration field order*, which is the trap a
    /// future author would fall into, so they are marked ⚠ above and again at
    /// their arms:
    ///
    /// 1. `LexicalClosure` declares `captures, params, body` but plans **body
    ///    first**, because the body is planned before the capture sequence.
    /// 2. `Effect`'s capability takes position `0` **only when present**, so the
    ///    argument base is a conditional offset rather than a constant.
    ///
    /// ## ⭐ THE SECOND AXIS: `entry` vs `occurrence`
    ///
    /// Positional agreement does **not** imply that the identity a parent
    /// schedules is the identity that owns the child record. `plan_expr` returns
    /// both (`PlannedExpr { entry, occurrence }`), and the positions above are
    /// always indexed by the **occurrence**.
    ///
    /// | | `entry == occurrence.node`? |
    /// |---|---|
    /// | every variant except `ComputationalMatch` | **yes**, by construction — they all return through `expression_node` |
    /// | `ComputationalMatch` | **no**, and deliberately: its record is seeded on its `SourceReturnResume` while a parent still schedules its scrutinee. It is the SOLE split. |
    ///
    /// ⛔ Passing an `entry` where an `occurrence` belongs is a category error, not
    /// an off-by-one. The seed API takes `&[StaticOriginId]`
    /// so the type now prevents it; do not re-conflate the two axes.
    ///
    /// ⛔ Where the two orders could ever disagree the **planner's** position
    /// wins: the plane's records are already laid out against it.
    fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let SourceOccurrence {
            expr,
            static_origin,
        } = occurrence;
        match expr {
            RuntimeExpr::Value(value) => self
                .lower_value(builder, value)
                .map(LoweringOperand::Specialized),
            RuntimeExpr::CheckedJoinSite { site_id, body } => {
                if self.active_join_site.replace(*site_id).is_some() {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "nested checked join occurrence marker",
                    ));
                }
                let body = self.child_occurrence(static_origin, 0, body)?;
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
                let body = self.child_occurrence(static_origin, 0, body)?;
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
                let instance =
                    self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_expr(builder, body, env);
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.lower_expr(builder, body, env)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let value = self.lower_expr(builder, body, env)?;
                self.finish_checked_computational_ih_marker(value)
            }
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .cloned()
                .ok_or_else(|| unsupported("Var", format!("no runtime binding for index {index}"))),
            RuntimeExpr::PrimitiveCall { primitive, args } => {
                self.lower_primitive_call(builder, primitive, args, static_origin, env)
            }
            RuntimeExpr::Let { value, body } => {
                let value = self.child_occurrence(static_origin, 0, value)?;
                let lowered_value = self.lower_expr(builder, value, env)?;
                if matches!(lowered_value, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                    return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                }
                if let LoweringOperand::Specialized(Lowered::Trap(trap)) = lowered_value {
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                }
                let mut body_env = vec![lowered_value];
                body_env.extend_from_slice(env);
                let body = self.child_occurrence(static_origin, 1, body)?;
                self.lower_expr(builder, body, &body_env)
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let then_expr = self.child_occurrence(static_origin, 1, then_expr)?;
                let else_expr = self.child_occurrence(static_origin, 2, else_expr)?;
                let lowered_scrutinee = self.lower_expr(builder, scrutinee, env)?;
                if matches!(lowered_scrutinee, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                    return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                }
                let LoweringOperand::Specialized(Lowered::Bool { value, known }) = lowered_scrutinee else {
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
                    let LoweringOperand::Specialized(Lowered::Int { value, known }) = lowered else {
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
                Ok(LoweringOperand::Specialized(Lowered::Int {
                    value,
                    known: None,
                }))
            }
            RuntimeExpr::Construct { constructor, args } => {
                let lowered_args = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(static_origin, position, arg)?;
                        self.lower_expr(builder, arg, env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if lowered_args
                    .iter()
                    .any(|arg| matches!(arg, LoweringOperand::Specialized(Lowered::RecursiveBackedge)))
                {
                    return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                }
                if lowered_args.is_empty()
                    && (constructor == &self.process_symbols.bool_true
                        || constructor == &self.process_symbols.bool_false)
                {
                    let known = constructor == &self.process_symbols.bool_true;
                    return Ok(LoweringOperand::Specialized(Lowered::Bool {
                        value: builder.ins().iconst(types::I64, i64::from(known)),
                        known: Some(known),
                    }));
                }
                if constructor == &self.process_symbols.nat_zero && lowered_args.is_empty() {
                    return Ok(LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 {
                        value: builder.ins().iconst(types::I64, 0),
                    })));
                }
                if constructor == &self.process_symbols.nat_suc {
                    if let [LoweringOperand::Specialized(Lowered::StructuralNat(predecessor))] = lowered_args.as_slice() {
                        return Ok(LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 {
                            value: builder.ins().iadd_imm(predecessor.value, 1),
                        })));
                    }
                }
                Ok(LoweringOperand::Specialized(Lowered::Constructor {
                    constructor: constructor.clone(),
                    args: specialized_env_at(&lowered_args, "a constructor argument")?,
                }))
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                let scrutinee_occurrence = self.child_occurrence(static_origin, 0, scrutinee)?;
                if requires_heterogeneous_deforestation(scrutinee)
                    || self.declaration_call_produces_deforestable_aggregate(scrutinee)
                {
                    return self.lower_computational_producer_expr(
                        builder,
                        scrutinee_occurrence,
                        env,
                        &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                            cases,
                            default,
                            env,
                            static_origin,
                            retained_scrutinee_index: None,
                            deferred_constructor_case: None,
                        })],
                    );
                }
                let lowered_scrutinee = self.lower_expr(builder, scrutinee_occurrence, env)?;
                // ⭐⭐ `D3`'s CARRIED arm, and it MUST come first.
                //
                // ⚠ Every test below asks for a specific `Lowered` shape, and
                // the chain ends in *"scrutinee is not a constructor value"*. A
                // carried scrutinee would fall past all of them and land on
                // that refusal — a **true sentence about the wrong thing**,
                // which is worse than an error, because it names a cause that
                // is not the cause. Classifying the phase first is what makes
                // the rest of the chain a statement about `Lowered` only.
                if let LoweringOperand::Carried(word) = lowered_scrutinee {
                    return self.lower_carried_match(
                        builder,
                        word,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer }) = lowered_scrutinee {
                    return self
                        .lower_borrowed_match(builder, pointer, cases, default, static_origin, env);
                }
                if let LoweringOperand::Specialized(Lowered::BorrowedOption {
                    present,
                    value,
                    none,
                    some,
                }) = lowered_scrutinee
                {
                    return self.lower_borrowed_option_match(
                        builder,
                        present,
                        value,
                        &none,
                        &some,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::BoundedNat(nat)) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(
                        builder,
                        nat,
                        false,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::StructuralNat(nat)) = lowered_scrutinee {
                    return self.lower_bounded_nat_match(
                        builder,
                        BoundedNatV1::derived_from_validated(nat.value),
                        true,
                        cases,
                        default,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::HostResult {
                    success,
                    error,
                    ok,
                    err_constructor,
                    ok_constructor,
                }) = lowered_scrutinee
                {
                    return self.lower_dynamic_host_result_match(
                        builder,
                        success,
                        *error,
                        *ok,
                        &err_constructor,
                        &ok_constructor,
                        cases,
                        static_origin,
                        env,
                    );
                }
                if let LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) = lowered_scrutinee {
                    return self.lower_dynamic_constructor_match(
                        builder,
                        dynamic,
                        DynamicConstructorContinuation::Ordinary {
                            cases,
                            default,
                            env,
                            static_origin,
                        },
                    );
                }
                if let LoweringOperand::Specialized(Lowered::Bool { value, known }) = lowered_scrutinee {
                    // ⭐ These two cases are found by CONSTRUCTOR NAME, and a
                    // search yields no position — so both lookups enumerate and
                    // keep the index. The index, not the found body, is what the
                    // origin is derived from.
                    let true_case = cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::True")
                    });
                    let false_case = cases.iter().enumerate().find(|(_, case)| {
                        case.binders == 0 && case.constructor.ends_with("::Bool::False")
                    });
                    let (Some(true_case), Some(false_case)) = (true_case, false_case) else {
                        return Err(unsupported(
                            "Match",
                            "Bool match requires zero-binder True and False cases",
                        ));
                    };
                    if let Some(selected) = known {
                        let (index, case) = if selected { true_case } else { false_case };
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        return self.lower_expr(builder, body, env);
                    }
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = builder.create_block();
                    builder.append_block_param(merge, types::I64);
                    builder.append_block_param(merge, types::I64);
                    builder
                        .ins()
                        .brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    for (block, (index, case)) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        let lowered = self.lower_expr(builder, body, env)?;
                        let (value, branch_kind) =
                            self.merge_scalar_branch(builder, lowered, "Match")?;
                        Self::record_scalar_merge_kind(
                            "Match",
                            &mut merge_kind,
                            branch_kind,
                        )?;
                        builder
                            .ins()
                            .jump(merge, &[value.tag.into(), value.payload.into()]);
                    }
                    builder.switch_to_block(merge);
                    let pair = NativeScalarPairV1 {
                        tag: builder.block_params(merge)[0],
                        payload: builder.block_params(merge)[1],
                    };
                    return Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
                        merge_kind.expect("Bool match emits both closed alternatives"),
                        pair,
                    )));
                }
                let LoweringOperand::Specialized(Lowered::Constructor { constructor, args }) = lowered_scrutinee else {
                    return Err(unsupported("Match", "scrutinee is not a constructor value"));
                };
                let Some((index, case)) = cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == constructor)
                else {
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
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
                let case_env = env_with(args, env);
                let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                self.lower_expr(builder, body, &case_env)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee,
                cases,
                default,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                self.lower_computational_match_expr(
                    builder,
                    scrutinee,
                    cases,
                    default,
                    static_origin,
                    env,
                    env,
                )
            }
            RuntimeExpr::Record { fields } => {
                let lowered_fields = fields
                    .iter()
                    .enumerate()
                    .map(|(position, (name, expr))| {
                        let expr = self.child_occurrence(static_origin, position, expr)?;
                        Ok((name.clone(), self.lower_expr(builder, expr, env)?))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                let lowered_fields = lowered_fields
                    .into_iter()
                    .map(|(name, value)| {
                        Ok((name, value.specialized_at("a record field's value")?))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
                Ok(LoweringOperand::Specialized(Lowered::Record {
                    fields: lowered_fields,
                }))
            }
            RuntimeExpr::Project { record, field } => {
                let record = self.child_occurrence(static_origin, 0, record)?;
                let lowered_record = self.lower_expr(builder, record, env)?;
                // ⭐ `D4`'s two-phase arm. ⛔ No wildcard over the phase: a
                // third `LoweringOperand` inhabitant must break compilation
                // here rather than silently taking whichever arm a `_` had
                // swallowed (`§2g`, `D5`).
                match lowered_record {
                    // ── the CARRIED route — `record_field` at runtime ──────
                    LoweringOperand::Carried(word) => {
                        // ⭐ `D1`/`D2`: the key is the artifact-static field
                        // identity of **this `Project` occurrence**, from the
                        // one authority the producer's `store_name` also used.
                        //
                        // ⚠ The `field` **string** is deliberately NOT the key.
                        // It is the compile-time spelling; keying on it would be
                        // the second derivation `D2` forbids — and it is also
                        // what makes `AC-C5` work, because a record whose fields
                        // are reordered relative to declaration still projects
                        // correctly when the lookup is by interned name rather
                        // than by position.
                        let identity = self
                            .static_transition_plan
                            .project_field_identity(static_origin)?
                            .name_abi_word()?;
                        let selected = self.emit_carrier_record_field(builder, word, identity)?;
                        // ⭐ `§2g`, verbatim: *"projected children remain
                        // `Carried`."* ⛔ Not materialized into a `Lowered`
                        // template — that is the wall itself.
                        Ok(LoweringOperand::Carried(selected))
                    }
                    // ── the pre-existing SPECIALIZED route, unchanged ──────
                    LoweringOperand::Specialized(lowered) => {
                        let Lowered::Record { fields } = lowered else {
                            return Err(unsupported(
                                "Project",
                                "record projection needs a record value",
                            ));
                        };
                        fields
                            .into_iter()
                            .find_map(|(name, value)| (name == *field).then_some(value))
                            .map(LoweringOperand::Specialized)
                            .ok_or_else(|| unsupported("Project", format!("missing field {field}")))
                    }
                }
            }
            // Site 1 of 3. The occurrence's own origin is in scope here, so the
            // body's origin is `child(_, 0)` — determined, not searched for.
            // ⭐ The origin is now the *whole* carrier: the body is not cloned into
            // the closure at all, and the term is recovered from the plan by this
            // name when a call site re-lowers it. The clone this site used to make
            // was the second authority the chain exists to remove.
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                let lowered_captures = captures
                    .iter()
                    .map(|symbol| self.lower_seed_capture(builder, symbol))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(LoweringOperand::Specialized(Lowered::Closure {
                    captures: lowered_captures,
                    params: params.clone(),
                    body: body.static_origin,
                }))
            }
            // D7, site 2 of 3.
            //
            // ⚠ HAZARD 1 (D3): the planner plans the **body first** and the
            // capture sequence after it, so body is position `0` and capture *i*
            // is `1 + i` — the declaration order (`captures, params, body`) is
            // NOT the child order. Evaluation order below is unchanged: the
            // captures are still lowered before the body is retained.
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(static_origin, 0, body)?;
                let captures = captures
                    .iter()
                    .enumerate()
                    .map(|(position, capture)| {
                        let capture = self.child_occurrence(static_origin, 1 + position, capture)?;
                        self.lower_expr(builder, capture, env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(LoweringOperand::Specialized(Lowered::Closure {
                    captures: specialized_env_at(&captures, "a closure capture")?,
                    params: params.clone(),
                    body: body.static_origin,
                }))
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
                let callee = self.child_occurrence(static_origin, 0, callee)?;
                let lowered_callee = self.lower_expr(builder, callee, env)?;
                match lowered_callee {
                    LoweringOperand::Specialized(Lowered::DeclarationClosure {
                        symbol,
                        captures,
                        params,
                        body,
                    }) => self.lower_recursive_declaration_call(
                        builder,
                        &symbol,
                        &captures,
                        &params,
                        self.retained_body_occurrence(body)?,
                        args,
                        static_origin,
                        env,
                        None,
                    ),
                    LoweringOperand::Specialized(Lowered::Closure {
                        captures,
                        params,
                        body,
                    }) => {
                        // Resolved once and shadowed, as in the producer arm above.
                        let body = self.retained_body_occurrence(body)?;
                        if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                            // The cases here belong to the closure BODY
                            // occurrence: `ordinary_match_continuation` matches
                            // only a body that *is* a `Match` (over `Var(0)`),
                            // so the body's own origin is their parent.
                            if let Some((cases, default)) =
                                ordinary_match_continuation(&params, body.expr)
                            {
                                let argument =
                                    self.child_occurrence(static_origin, 1, &args[0])?;
                                let frame_env = env_with(captures, env);
                                return self.lower_computational_producer_expr(
                                    builder,
                                    argument,
                                    env,
                                    &[EliminatorFrame::Ordinary(OrdinaryEliminatorFrame {
                                        cases,
                                        default,
                                        env: &frame_env,
                                        static_origin: body.static_origin,
                                        retained_scrutinee_index: Some(0),
                                        deferred_constructor_case: None,
                                    })],
                                );
                            }
                        }
                        let mut call_env = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                self.lower_expr(builder, arg, env)
                            })
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
                        call_env.extend(captures.into_iter().map(LoweringOperand::Specialized));
                        call_env.extend_from_slice(env);
                        self.lower_expr(builder, body, &call_env)
                    }
                    LoweringOperand::Specialized(
                        mut callee @ Lowered::ComputationalRecursorClosure { .. },
                    ) => {
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(
                            LoweringOperand::Specialized(callee),
                        );
                        let (activation, invocation) = boundary.expect(
                            "recursor closure carries an invocation segment",
                        );
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let dynamic_splice_edges =
                            self.take_dynamic_splice_edges(&invocation)?;
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
                        // ⭐⭐ `AC-C4` — the carried residual on the direct
                        // `lower_expr` call route.
                        if let LoweringOperand::Carried(word) = base {
                            Self::reject_carried_residual_arguments(args.len())?;
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self.lower_computational_match_value_composed(
                                builder,
                                LoweringOperand::Carried(word),
                                &frames,
                            );
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
                        }
                        let base =
                            base.specialized_at("a recursor residual in a direct call")?;
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
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                self.lower_expr(builder, arg, env)
                            })
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
                        call_env.extend(captures.into_iter().map(LoweringOperand::Specialized));
                        call_env.extend_from_slice(env);
                        self.enter_oriented_semantic_region(installed.checked);
                        let result = self.lower_computational_producer_expr(
                            builder,
                            self.retained_body_occurrence(body)?,
                            &call_env,
                            &frames,
                        );
                        self.leave_oriented_semantic_region(installed.checked);
                        result
                    }
                    _ => Err(unsupported("Call", "callee is not a closure")),
                }
            }
            RuntimeExpr::Trap(trap) => Ok(LoweringOperand::Specialized(Lowered::Trap(trap.clone()))),
            // ⚠ HAZARD 2 (D3): the capability occupies child position `0` **only
            // when present**, so the argument base is `1` with a capability and
            // `0` without it. A constant base would mis-key every argument of
            // every capability-carrying effect, and nothing in the types would
            // notice.
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
                static_origin,
                env,
            ),
            RuntimeExpr::Effect { family, operation, .. } => Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is not modeled in the supported native subset",
                    *operation as u16
                ),
            )),
        }
    }

    /// `static_origin` is the `Effect` occurrence's own origin.
    ///
    /// ⚠ HAZARD 2 (D3): the planner plans `capability.value` **first when it is
    /// present**, so the argument base is `1` with a capability and `0` without
    /// one. `argument_base` below is that conditional offset, computed once from
    /// the same `Option` the planner tested (`static_transition.rs` `Effect`
    /// arm) rather than assumed.
    #[allow(clippy::too_many_arguments)]
    fn lower_process_host_effect(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        family: &RuntimeSymbol,
        operation: ken_host::HostOpV1,
        capability: Option<&crate::RuntimeCapabilityUse>,
        args: &[RuntimeExpr],
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        if !CRANELIFT_HOST_EFFECT_CONSUMERS_V1.contains(&operation) {
            return Err(unsupported(
                "Effect",
                format!(
                    "effect {family}.{} is a represented unavailable lane",
                    operation as u16
                ),
            ));
        }
        let argument_base = usize::from(capability.is_some());
        let lowered = args
            .iter()
            .enumerate()
            .map(|(position, argument)| {
                let argument =
                    self.child_occurrence(static_origin, argument_base + position, argument)?;
                self.lower_expr(builder, argument, env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // ⭐ `§2h` ¶2's typed phase boundary in front of the host-effect wire.
        // The wire encoder reads templates to fill a request slot — a `Stream`
        // constructor's tag, a `Bytes` body's pointer and length, an `Int`'s
        // narrowed u64. ⛔ A carried operand fails closed rather than being
        // written into a host request under a guessed encoding.
        let lowered = specialized_env_at(&lowered, "a host-effect operand")?;
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
                // Present ⇒ the capability value is child 0 of this occurrence.
                let capability_value =
                    self.child_occurrence(static_origin, 0, &capability.value)?;
                let LoweringOperand::Specialized(Lowered::CapabilityToken { value: token }) =
                    self.lower_expr(builder, capability_value, env)?
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
                let Lowered::ResourceToken { value: span_origin } = lowered.get(3).ok_or_else(
                    || unsupported("Effect", "BufferFreeze is missing its span origin"),
                )?
                else {
                    return Err(unsupported(
                        "Effect",
                        "BufferFreeze span origin is not a resource",
                    ));
                };
                for (index, value) in [*token, start, length, *span_origin].into_iter().enumerate() {
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
        if operation == ken_host::HostOpV1::ConsoleIsTerminal {
            Self::require_i64(builder, tag, wire.reply_bool_tag as i64);
            Ok(LoweringOperand::Specialized(Lowered::Bool {
                value: detail,
                known: None,
            }))
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
                let Lowered::ResourceToken { value: span_origin } = lowered
                    .get(2)
                    .ok_or_else(|| unsupported("Effect", "FsReadAt is missing its buffer operand"))?
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
            Ok(LoweringOperand::Specialized(Lowered::HostResult {
                success,
                error: Box::new(error),
                ok: Box::new(ok),
                err_constructor: self.process_symbols.result_err.clone(),
                ok_constructor: self.process_symbols.result_ok.clone(),
            }))
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn lower_unary_recursive_nat_fold(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        argument: Lowered,
        zero_body: SourceOccurrence<'_>,
        suc_body: SourceOccurrence<'_>,
        producer_env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
        let mut zero_env = env_with([zero_nat], &[]);
        extend_specialized(&mut zero_env, captures.iter().cloned());
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
        let mut suc_env = env_with([predecessor, successor], &[]);
        extend_specialized(&mut suc_env, captures.iter().cloned());
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
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done_block)[0],
                payload: builder.block_params(done_block)[1],
            },
        )))
    }

    /// `body` is the declaration closure's body occurrence (reachable by symbol,
    /// D6); `call_origin` is the origin of the **`Call` occurrence** whose
    /// arguments these are, so argument *i* is `child(call_origin, 1 + i)`.
    #[allow(clippy::too_many_arguments)]
    fn lower_recursive_declaration_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        params: &[String],
        body: SourceOccurrence<'_>,
        args: &[RuntimeExpr],
        call_origin: StaticOriginId,
        producer_env: &[LoweringOperand],
        eliminators: Option<&[EliminatorFrame<'_>]>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(symbol)?;
        let lowered_args = args
            .iter()
            .enumerate()
            .map(|(position, arg)| {
                let arg = self.child_occurrence(call_origin, 1 + position, arg)?;
                self.lower_expr(builder, arg, producer_env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // ⭐ A recursive declaration's arguments are its **loop-header
        // representation**: their shapes are compared across iterations
        // (`same_recursive_argument_shapes`) and lowered into block params. A
        // carried boundary word has no such shape, so this is a
        // specialized-only surface with the ruled fail-closed arm.
        let lowered_args = specialized_env_at(&lowered_args, "a recursive declaration argument")?;
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
                return Ok(LoweringOperand::Specialized(induction));
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
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        }

        // Only declarations in an actual recursive SCC need the loop/result
        // closure below. Preserve the established direct-call lowering for
        // ordinary declarations, including constructor-valued HostIO trees.
        if !self.declaration_is_recursive(symbol) {
            let mut call_env = lowered_args
                .into_iter()
                .rev()
                .map(LoweringOperand::Specialized)
                .collect::<Vec<_>>();
            extend_specialized(&mut call_env, captures.iter().cloned());
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
            } = body.expr
            {
                if matches!(scrutinee.as_ref(), RuntimeExpr::Var(0)) {
                    // These two arms are found by constructor name under the
                    // BODY occurrence's match, so their bodies are its children
                    // `1 + index` — the index the search would otherwise discard.
                    let zero = cases.iter().enumerate().find(|(_, case)| {
                        case.constructor == self.process_symbols.nat_zero && case.binders == 0
                    });
                    let suc = cases.iter().enumerate().find(|(_, case)| {
                        case.constructor == self.process_symbols.nat_suc && case.binders == 1
                    });
                    if let (Some((zero_index, zero)), Some((suc_index, suc))) = (zero, suc) {
                        let zero_body = self.case_body_occurrence(
                            body.static_origin,
                            zero_index,
                            &zero.body,
                        )?;
                        let suc_body =
                            self.case_body_occurrence(body.static_origin, suc_index, &suc.body)?;
                        return self.lower_unary_recursive_nat_fold(
                            builder,
                            symbol,
                            captures,
                            lowered_args
                                .into_iter()
                                .next()
                                .expect("unary recursion owns one argument"),
                            zero_body,
                            suc_body,
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
        let mut call_env = loop_args
            .into_iter()
            .rev()
            .map(LoweringOperand::Specialized)
            .collect::<Vec<_>>();
        extend_specialized(&mut call_env, captures.iter().cloned());
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
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            result_kind,
            NativeScalarPairV1 {
                tag: builder.block_params(done)[0],
                payload: builder.block_params(done)[1],
            },
        )))
    }

    fn lower_declaration_ref(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &RuntimeSymbol,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
        // D6/D7: a `DeclarationRef` is a childless leaf, and the declaration's
        // body is a **separately planned** source occurrence, reachable by name.
        // That is why this construction site needs no threading — and why it must
        // not suggest the two `lower_expr` closure arms are nearly done. Only
        // transparent declarations are planned, which is exactly the set that
        // survives the rejection above, so a missing occurrence is a planner bug.
        let declaration_origin = self
            .static_transition_plan
            .declaration_occurrence_origin(symbol.as_str())
            .ok_or_else(|| {
                // A planner invariant, not a capacity limit: this declaration is
                // transparent, so the planner planned it.
                backend(BackendFailure::PlannerInvariant(format!(
                    "transparent declaration {symbol} has no planned source occurrence"
                )))
            })?;
        let declaration_body = SourceOccurrence {
            expr: body,
            static_origin: declaration_origin,
        };
        if let RuntimeExpr::Closure {
            captures,
            params,
            body,
        } = body
        {
            let body = self.child_occurrence(declaration_origin, 0, body)?;
            let captures = captures
                .iter()
                .map(|capture| self.lower_seed_capture(builder, capture))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(LoweringOperand::Specialized(Lowered::DeclarationClosure {
                symbol: symbol.clone(),
                captures,
                params: params.clone(),
                body: body.static_origin,
            }));
        }
        if self.declaration_stack.contains(symbol) {
            return Err(unsupported(
                "DeclarationRef",
                format!("recursive non-function declaration {symbol} is unsupported"),
            ));
        }
        self.declaration_stack.push(symbol.clone());
        let result = self.lower_expr(builder, declaration_body, &[]);
        self.declaration_stack.pop();
        result
    }

    /// `static_origin` is the origin of the **match occurrence** whose cases
    /// these are; case *i*'s body is `child(static_origin, 1 + i)`.
    fn lower_borrowed_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        pointer: cranelift_codegen::ir::Value,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
                    LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer: field })
                })
                .collect::<Vec<_>>();
            arm_env.extend_from_slice(env);
            // The single-case fast path is still case 0 of this match.
            let body = self.case_body_occurrence(static_origin, 0, &case.body)?;
            return self.lower_expr(builder, body, &arm_env);
        }
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let mut test_block = builder.current_block().expect("borrowed match block");
        let mut merge_kind = None;
        for (index, case) in cases.iter().enumerate() {
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
                    LoweringOperand::Specialized(Lowered::BorrowedNativeValue { pointer: field })
                })
                .collect::<Vec<_>>();
            arm_env.extend_from_slice(env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
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
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            merge_kind.expect("borrowed match emits at least one case"),
            pair,
        )))
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_borrowed_option_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        present: cranelift_codegen::ir::Value,
        value: cranelift_codegen::ir::Value,
        none: &str,
        some: &str,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
            let case = cases
                .iter()
                .enumerate()
                .find(|(_, case)| case.constructor == symbol);
            let Some((index, case)) = case else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            if case.binders != fields.len() {
                return Err(unsupported("Match", "borrowed Option arity mismatch"));
            }
            let arm_env = env_with(fields, env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
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
            LoweringOperand::Specialized(Lowered::ProcessExitStatus {
                value: pair.payload,
            })
        } else {
            LoweringOperand::Specialized(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
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
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
            let Some((index, case)) = cases
                .iter()
                .enumerate()
                .find(|(_, case)| case.constructor == constructor && case.binders == 1)
            else {
                let failure = builder.ins().iconst(types::I64, -1);
                builder.ins().return_(&[failure]);
                continue;
            };
            let arm_env = env_with([payload], env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
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
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            merge_kind.expect("HostResult emits both closed alternatives"),
            pair,
        )))
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_bounded_nat_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        nat: BoundedNatV1,
        structural: bool,
        cases: &[crate::RuntimeMatchCase],
        _default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let zero = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_zero && case.binders == 0
        });
        let suc = cases.iter().enumerate().find(|(_, case)| {
            case.constructor == self.process_symbols.nat_suc && case.binders == 1
        });
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
        for (block, (index, case), predecessor) in [
            (zero_block, zero, None),
            (suc_block, suc, Some(predecessor)),
        ] {
            builder.switch_to_block(block);
            let arm_env = predecessor
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
            let mut arm_env = env_with(arm_env, &[]);
            arm_env.extend_from_slice(env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = self.lower_expr(builder, body, &arm_env)?;
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
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            merge_kind.expect("both structural Nat arms were emitted"),
            pair,
        )))
    }

    fn lower_dynamic_constructor_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        dynamic: DynamicConstructorV1,
        continuation: DynamicConstructorContinuation<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
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
            let (cases, default, env, static_origin) = match continuation {
                DynamicConstructorContinuation::Ordinary {
                    cases,
                    default,
                    env,
                    static_origin,
                }
                | DynamicConstructorContinuation::Producer {
                    cases,
                    default,
                    env,
                    static_origin,
                    ..
                } => (cases, default, env, static_origin),
            };
            let (index, case) = match select_dynamic_constructor_case(cases, &alternative, default)?
            {
                Ok(selected) => selected,
                Err(_owned_default) => {
                    let failure = builder.ins().iconst(types::I64, -4);
                    builder.ins().return_(&[failure]);
                    test_block = next;
                    continue;
                }
            };
            let arm_env = materialize_dynamic_constructor_env(&alternative, env);
            let body = self.case_body_occurrence(static_origin, index, &case.body)?;
            let lowered = match continuation {
                DynamicConstructorContinuation::Ordinary { .. } => {
                    self.lower_expr(builder, body, &arm_env)?
                }
                DynamicConstructorContinuation::Producer { eliminators, .. } => {
                    self.lower_computational_producer_expr(builder, body, &arm_env, eliminators)?
                }
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
            return Ok(LoweringOperand::Specialized(Lowered::Trap(source_default.clone())));
        };
        builder.switch_to_block(merge);
        let pair = NativeScalarPairV1 {
            tag: builder.block_params(merge)[0],
            payload: builder.block_params(merge)[1],
        };
        Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
            merge_kind.expect("a selected dynamic constructor case emits one arm"),
            pair,
        )))
    }

    /// `static_origin` is the `PrimitiveCall` occurrence's own origin; argument
    /// *i* is child *i* (a primitive symbol is an atom, not a child).
    fn lower_primitive_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        primitive: &RuntimePrimitive,
        args: &[RuntimeExpr],
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let lowered_args = args
            .iter()
            .enumerate()
            .map(|(position, arg)| {
                let arg = self.child_occurrence(static_origin, position, arg)?;
                self.lower_expr(builder, arg, env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if lowered_args
            .iter()
            .any(|arg| matches!(arg, LoweringOperand::Specialized(Lowered::RecursiveBackedge)))
        {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
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
                return Ok(LoweringOperand::Specialized(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message,
                })));
            }
            RuntimePartiality::TrustedTrap { assumption } => {
                self.assumptions.insert(format!(
                    "trusted partial assumption {assumption} remains visible"
                ));
                return Ok(LoweringOperand::Specialized(Lowered::Trap(RuntimeTrap {
                    code: RuntimeTrapCode::ExplicitTrap,
                    message: format!("{} trusted partiality trapped", primitive.symbol),
                })));
            }
        }

        // ⭐ `§2h` ¶2's typed phase boundary in front of the primitive leaves.
        // Every arm below builds a **fresh specialized value** out of templates
        // it reads — an `Int`'s known-ness, a `Bytes` body, a `Bool`'s constant
        // — which is the ruling's own example of a helper that keeps raw
        // `Lowered`. ⛔ A carried operand fails closed here instead of reaching
        // one of them; its route is an emitted helper call.
        let lowered_args = specialized_env_at(&lowered_args, "a primitive-call operand")?;
        let lowered = match primitive.symbol.as_str() {
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
        };
        // ⭐ Back onto the spine: a primitive's result is a fresh specialized
        // value re-entering the phase sum.
        lowered.map(LoweringOperand::Specialized)
    }
}
