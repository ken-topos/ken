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

#[cfg(test)]
thread_local! {
    static C2_UNIT_EMISSION_EPOCH: std::cell::Cell<Option<u64>> =
        const { std::cell::Cell::new(None) };
    static RECURSIVE_POSITION_UNIT_CALLS: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn c2_unit_emission_epoch() -> Option<u64> {
    C2_UNIT_EMISSION_EPOCH.with(std::cell::Cell::get)
}

#[cfg(test)]
fn recursive_position_unit_calls() -> usize {
    RECURSIVE_POSITION_UNIT_CALLS.with(std::cell::Cell::get)
}

/// The closed production routes that still require retained recursive descent.
///
/// This type is the D5 accounting: the selector produces one of these reasons
/// rather than consulting a second spelling list. D1/D2/D3/D6/D8 ported and
/// admitted recursive positions, trap terminals, carried host-effect seats, and
/// result-directed joins; D7/S4 exercise their corrected governed composition.
/// S4's completed-emission rows establish collection capability only; they are
/// not an asymptotic verdict about those rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecursiveDescentResidual {
    /// An ordinary producer match whose scrutinee is directly a call.
    ProducerMatchCall,
    /// An ordinary match consuming an active computational recursor.
    MatchScrutineeRecursor,
    /// A lexical unit call whose argument is an active computational recursor.
    ///
    /// The recursive result still carries invocation-local scope/return-hole
    /// state. Passing it through a separately declared lexical unit is not one
    /// of the completed functionized ports, so the established recursive
    /// descent lane retains the whole call.
    LexicalCallArgumentRecursor,
    /// A call whose callee is the retained non-lexical closure form.
    SeedClosureCall,
    /// A transparent declaration whose body is a closure seed.
    TransparentDeclarationClosure,
}

/// Produce the retained reason, if any, from the exhaustive source walk.
///
/// Wrapper and child-producing forms propagate a reason from their children.
/// The exhaustive match is the fail-closed default: a new `RuntimeExpr` form
/// cannot compile until this production classifier assigns it to the
/// functionized population or to a typed retained reason.
fn recursive_descent_residual(expr: &RuntimeExpr) -> Option<RecursiveDescentResidual> {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Closure { body, .. } => recursive_descent_residual(body),
        RuntimeExpr::LexicalClosure { captures, body, .. } => captures
            .iter()
            .find_map(recursive_descent_residual)
            .or_else(|| recursive_descent_residual(body)),
        RuntimeExpr::Let { value, body } => {
            recursive_descent_residual(value).or_else(|| recursive_descent_residual(body))
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => recursive_descent_residual(scrutinee)
            .or_else(|| recursive_descent_residual(then_expr))
            .or_else(|| recursive_descent_residual(else_expr)),
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            args.iter().find_map(recursive_descent_residual)
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => matches!(scrutinee.as_ref(), RuntimeExpr::Call { .. })
            .then_some(RecursiveDescentResidual::ProducerMatchCall)
            .or_else(|| {
                matches!(
                    scrutinee.as_ref(),
                    RuntimeExpr::ComputationalMatch { cases, .. }
                        if cases
                            .iter()
                            .any(|case| !case.recursive_positions.is_empty())
                )
                .then_some(RecursiveDescentResidual::MatchScrutineeRecursor)
            })
            .or_else(|| recursive_descent_residual(scrutinee))
            .or_else(|| {
                cases
                    .iter()
                    .find_map(|case| recursive_descent_residual(&case.body))
            }),
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => recursive_descent_residual(scrutinee).or_else(|| {
            cases
                .iter()
                .find_map(|case| recursive_descent_residual(&case.body))
        }),
        RuntimeExpr::Record { fields } => fields
            .iter()
            .find_map(|(_, value)| recursive_descent_residual(value)),
        RuntimeExpr::Project { record, .. } => recursive_descent_residual(record),
        RuntimeExpr::Call { callee, args } => {
            matches!(callee.as_ref(), RuntimeExpr::Closure { .. })
                .then_some(RecursiveDescentResidual::SeedClosureCall)
                .or_else(|| {
                    (matches!(callee.as_ref(), RuntimeExpr::LexicalClosure { .. })
                        && args.iter().any(|argument| {
                            matches!(
                                argument,
                                RuntimeExpr::ComputationalMatch { cases, .. }
                                    if cases
                                        .iter()
                                        .any(|case| !case.recursive_positions.is_empty())
                            )
                        }))
                    .then_some(RecursiveDescentResidual::LexicalCallArgumentRecursor)
                })
                .or_else(|| recursive_descent_residual(callee))
                .or_else(|| args.iter().find_map(recursive_descent_residual))
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => capability
            .as_ref()
            .and_then(|capability| recursive_descent_residual(&capability.value))
            .or_else(|| args.iter().find_map(recursive_descent_residual)),
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => None,
    }
}

/// Produce the retained reason from the exhaustive declaration-kind route.
fn declaration_recursive_descent_residual(
    declaration: &RuntimeDeclaration,
) -> Option<RecursiveDescentResidual> {
    match &declaration.kind {
        RuntimeDeclarationKind::Transparent { body } => matches!(
            body,
            RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. }
        )
        .then_some(RecursiveDescentResidual::TransparentDeclarationClosure)
        .or_else(|| recursive_descent_residual(body)),
        RuntimeDeclarationKind::Primitive { .. }
        | RuntimeDeclarationKind::Data { .. }
        | RuntimeDeclarationKind::Record { .. }
        | RuntimeDeclarationKind::RecursiveGroup { .. }
        | RuntimeDeclarationKind::EffectBoundary { .. }
        | RuntimeDeclarationKind::MetadataOnly => None,
    }
}

/// The one temporary B2F migration selector, evaluated once at compilation
/// entry from source syntax and declaration kinds only.
///
/// `FunctionizedUnits` is selected only after both exhaustive production
/// classifiers produce no typed retained reason. No runtime value, carrier
/// class, walk result, or emission failure can change this answer after it is
/// chosen.
fn select_body_emission_authority(
    expr: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
) -> BodyEmissionAuthority {
    if recursive_descent_residual(expr)
        .or_else(|| {
            declarations
                .values()
                .find_map(|declaration| declaration_recursive_descent_residual(declaration))
        })
        .is_some()
    {
        BodyEmissionAuthority::RecursiveDescent
    } else {
        BodyEmissionAuthority::FunctionizedUnits
    }
}

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
    #[cfg(test)]
    {
        scale_b_reset_emission_attempt();
        C2_UNIT_EMISSION_EPOCH.with(|epoch| epoch.set(Some(0)));
        RECURSIVE_POSITION_UNIT_CALLS.with(|calls| calls.set(0));
        reset_d8_join_conversion_counts();
    }
    validate_oriented_subcontinuation_transport(
        expr,
        &declarations,
        oriented_subcontinuation_plan.as_ref(),
    )?;
    let body_emission_authority =
        select_body_emission_authority(expr, &declarations);
    // Boundary A of RT-NATIVE-FNSPLIT: close and validate the factored static
    // graph before Cranelift sees any semantic body. The plan's positional
    // child-origin table is reachable from the lowering, so
    // every occurrence carries the static name the planner already gave it.
    //
    // ⚠ The plan also outlives this call's borrow of `expr` and holds each planned
    // occurrence BY REFERENCE, because a retained closure body is now selected by
    // its origin rather than carried as a clone. The emitter is otherwise
    // unchanged, and nothing borrowed reaches `CompiledModule`.
    let process_symbols = process_symbols
        .cloned()
        .unwrap_or_else(crate::NativeProcessSymbols::legacy_prelude);
    let static_transition_plan = plan_static_transition_graph_with_symbols(
        expr,
        &declarations,
        &process_symbols,
        if process_mode {
            AbiRootIngress::Process
        } else {
            AbiRootIngress::Value
        },
        matches!(
            body_emission_authority,
            BodyEmissionAuthority::FunctionizedUnits
        ),
    )?;
    #[cfg(test)]
    scale_b_begin_emission_attempt(
        &static_transition_plan,
        matches!(
            body_emission_authority,
            BodyEmissionAuthority::FunctionizedUnits
        ),
    );
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));

    #[cfg(test)]
    C2_UNIT_EMISSION_EPOCH.with(|epoch| {
        epoch.set(Some(
            epoch
                .get()
                .expect("the C2 compilation epoch was initialized")
                .checked_add(1)
                .expect("the C2 compilation epoch fits u64"),
        ));
    });
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
    // ⭐ `RT-FNSPLIT-B2F` `D1` — forward-declare the WHOLE target-unit bundle
    // before any body (root or unit) is defined. A unit body may call any other
    // unit, so declaring every signature first is what makes the call graph
    // order-independent; a declare-and-define-in-one-pass loop could not emit a
    // call to a unit it had not reached yet.
    //
    // ⛔ The population is `B2O`'s validated owner partition as `B2R` described
    // it. This call does not derive it and must never be made to.
    // ⭐ `RT-FNSPLIT-B2F` `AC-11` clause 3 — the per-transfer representability
    // proof runs HERE, before a single unit is declared, defined or called.
    //
    // ⛔ Its position is the discharge. Moving this call below
    // `declare_unit_bundle` would satisfy everything the check asserts and prove
    // nothing about emission, and ⛔ no path may substitute `AbiPlane::validate`,
    // `C4`, or descriptor existence for it.
    // ⭐ The attempt epoch is stamped HERE, on the last statement before the
    // proof, so that "this compile reached emission and declared zero units" is
    // observable as a distinct outcome from "this compile never got here".
    // ⛔ Not inside `declare_unit_bundle`: stamping there would make the zero
    // reading unreachable, because observing the epoch would require declaring
    // the unit whose absence is the thing being measured.
    let functionized_bundle = match body_emission_authority {
        BodyEmissionAuthority::RecursiveDescent => None,
        BodyEmissionAuthority::FunctionizedUnits => {
            #[cfg(test)]
            super::units::b2f_reached_emission_seam();
            static_transition_plan.validate_emitted_transfers_are_representable()?;
            let units =
                super::units::declare_unit_bundle(&mut module, &static_transition_plan)?;
            // ⭐ `RT-FNSPLIT-B2F` `D4` — resolve every cross-owner call edge
            // against the bundle before a single body is defined.
            let calls =
                super::units::resolve_call_edges(&static_transition_plan, &units)?;
            Some((units, calls))
        }
    };
    // ⭐ `RT-FNSPLIT-B2F` `D3` — mint the artifact-static seed material before
    // any function context exists. `B2R` declared `GroundValueCarrier` as
    // `BorrowedForActivation` from `ArtifactStatic` and deliberately minted
    // nothing; this is the counterpart that gives the borrow an owner which
    // outlives every activation.
    //
    // ⛔ Minted from the environment, never from the plan: resolving which
    // symbols a unit captures would add an `origin -> expression` lookup, and
    // `AC-4` holds that count at exactly one.
    let seed_material = super::seed_material::mint_seed_material(&mut module, seed_env)?;
    let mut ctx = module.make_context();
    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);
    // ⭐ `RT-FNSPLIT-B2F` `S6` — the module-level identities, gathered under one
    // name so the root and every future unit body resolve them through **one**
    // operation. What stood here was twenty inline `declare_*_in_func` calls;
    // the point of the move is that a helper cannot be present in the root's
    // function and absent from a unit's by someone forgetting to copy a line.
    //
    // ⚠ `seed_material` is `D3`'s minted artifact-static material: a `DataId` is
    // a module-level identity and cannot be addressed from inside a body, so it
    // is resolved into this `Function` exactly as the native-int and
    // boundary-carrier helpers are.
    let helpers = ArtifactHelpers {
        seed_material: &seed_material,
        host_dispatch,
        native_int: &native_int,
        boundary_value_abi: &boundary_value_abi,
    };
    let root_function_local = helpers.declare_in_func(&mut module, &mut ctx.func);
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
        body_emission_authority,
        process_object: process_mode,
        process_symbols,
        #[cfg(test)]
        native_int_mutation: NATIVE_INT_LOWERING_MUTATION.with(std::cell::Cell::get),
        #[cfg(test)]
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
        function_local: root_function_local,
    };
    let root_result = match body_emission_authority {
        BodyEmissionAuthority::FunctionizedUnits => {
            let (unit_bundle, call_edges) = functionized_bundle
                .as_ref()
                .expect("the functionized selector arm owns its bundle");
            let root_result = super::units::define_unit_bodies(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                call_edges,
                staged_process_input,
            )?;
            compiler.require_complete_join_plan_consumption()?;
            compiler.require_complete_dynamic_splice_edge_consumption()?;
            super::units::define_root_adapter(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                func_id,
                process_mode,
            )?;
            root_result
        }
        BodyEmissionAuthority::RecursiveDescent => {
            let mut maybe_trap = None;
            let mut decoder = None;
            {
                let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
                let block = builder.create_block();
                builder.append_block_params_for_function_params(block);
                builder.switch_to_block(block);
                let ingress = builder.block_params(block)[0];
                let services = builder.block_params(block)[1];
                let pointer_type = module.target_config().pointer_type();
                let native_int_arena = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    services,
                    crate::activation_services::SERVICES_NATIVE_INT_ARENA,
                );
                Lowering::require_nonzero(&mut builder, native_int_arena);
                let boundary_arena = builder.ins().load(
                    pointer_type,
                    MemFlags::trusted(),
                    services,
                    crate::activation_services::SERVICES_BOUNDARY_ARENA,
                );
                Lowering::require_nonzero(&mut builder, boundary_arena);
                compiler.function_local.services_pointer = Some(services);
                compiler.function_local.native_int_arena = Some(native_int_arena);
                compiler.function_local.boundary_arena = Some(boundary_arena);

                let mut initial_env = Vec::new();
                if process_mode {
                    let process_input = builder.ins().load(
                        pointer_type,
                        MemFlags::trusted(),
                        ingress,
                        crate::boundary_activation::ROOT_INGRESS_PROCESS_INPUT,
                    );
                    Lowering::require_nonzero(&mut builder, process_input);
                    let host_dispatch_context = builder.ins().load(
                        pointer_type,
                        MemFlags::trusted(),
                        ingress,
                        crate::boundary_activation::ROOT_INGRESS_HOST_DISPATCH_CONTEXT,
                    );
                    Lowering::require_nonzero(&mut builder, host_dispatch_context);
                    let capability = builder.ins().load(
                        types::I64,
                        MemFlags::trusted(),
                        ingress,
                        crate::boundary_activation::ROOT_INGRESS_CAPABILITY,
                    );
                    compiler.function_local.host_dispatch_context =
                        Some(host_dispatch_context);
                    initial_env.push(LoweringOperand::Specialized(
                        Lowered::BorrowedNativeValue {
                            pointer: process_input,
                        },
                    ));
                    initial_env.push(LoweringOperand::Specialized(
                        Lowered::CapabilityToken { value: capability },
                    ));
                } else {
                    compiler.function_local.host_dispatch_context =
                        Some(builder.ins().iconst(pointer_type, 0));
                }
                if let Some(value) = staged_process_input {
                    initial_env.push(LoweringOperand::Specialized(
                        compiler.lower_value(&mut builder, value)?,
                    ));
                }
                compiler.root_terminal_authority =
                    compiler.take_distinguished_root_answer_authority()?;
                let root_origin =
                    compiler.static_transition_plan.root_static_origin()?;
                let root = compiler.retained_body_occurrence(root_origin)?;
                let lowered =
                    compiler.lower_expr(&mut builder, root, &initial_env)?;
                compiler.require_complete_join_plan_consumption()?;
                compiler.require_complete_dynamic_splice_edge_consumption()?;
                let lowered =
                    lowered.specialized_at("the recursive-descent root result")?;
                match lowered {
                    Lowered::Trap(trap) => {
                        #[cfg(test)]
                        if process_mode {
                            px8tr_record_trap_provenance(
                                Px8trTrapProvenanceEvent::FinalProcessObjectTrap {
                                    trap: trap.clone(),
                                },
                            );
                        }
                        let status = builder.ins().iconst(
                            types::I64,
                            if process_mode { -4 } else { 0 },
                        );
                        builder.ins().return_(&[status]);
                        maybe_trap = Some(trap);
                    }
                    value => {
                        let (token, result_decoder) =
                            compiler.emit_result(&mut builder, value)?;
                        builder.ins().return_(&[token]);
                        decoder = Some(result_decoder);
                    }
                }
                builder.seal_all_blocks();
                builder.finalize();
            }
            verify_cranelift_function(&ctx.func, module.isa())?;
            #[cfg(test)]
            scale_b_record_root_adapter(&ctx.func);
            module
                .define_function(func_id, &mut ctx)
                .map_err(|err| backend_module(err.to_string()))?;
            super::units::RootUnitResult {
                decoder,
                trap: maybe_trap,
            }
        }
    };
    let compiled = CompiledModule::from_parts(
        module,
        func_id,
        root_result.decoder,
        compiler.result_table,
        root_result.trap,
        true,
        compiler.assumptions,
        compiler.unsupported,
    );
    #[cfg(test)]
    scale_b_finish_emission_attempt();
    Ok(compiled)
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
        recursive_unit_body: Option<StaticOriginId>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐⭐ `AC-C4` — the carried residual, taken BEFORE the specialized
        // shapes so a carried word never reaches a template probe.
        if let LoweringOperand::Carried(word) = residual {
            if let Some(body) = recursive_unit_body.filter(|_| {
                matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                )
            }) {
                let inputs = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg =
                            self.child_occurrence(call_origin, 1 + position, arg)?;
                        self.lower_expr(builder, arg, argument_env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let returned =
                    self.call_declared_recursive_position_unit(builder, body, &inputs)?;
                return self.lower_computational_match_value_composed(
                    builder,
                    returned,
                    outer_eliminators,
                );
            }
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
                continuation.recursive_unit_body,
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
        self.enter_source_occurrence_plan(static_origin)?;
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
                                    let recursive_unit_body =
                                        invocation.recursive_unit_body;
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
                                            recursive_unit_body,
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
                let join_plan = self.consumed_join_plan_token(static_origin)?;
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
                        join_plan,
                    ),
                    LoweringOperand::Specialized(Lowered::Closure {
                        captures,
                        params,
                        body,
                    }) => {
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::RecursiveDescent
                        ) {
                            let retained = self.retained_body_occurrence(body)?;
                            if args.len() == 1
                                && requires_heterogeneous_deforestation(&args[0])
                            {
                                if let Some((cases, default)) =
                                    ordinary_match_continuation(&params, retained.expr)
                                {
                                    let argument =
                                        self.child_occurrence(static_origin, 1, &args[0])?;
                                    let frame_env = env_with(captures.clone(), producer_env);
                                    let mut composed =
                                        Vec::with_capacity(eliminators.len() + 1);
                                    composed.push(EliminatorFrame::Ordinary(
                                        OrdinaryEliminatorFrame {
                                            cases,
                                            default,
                                            env: &frame_env,
                                            static_origin: retained.static_origin,
                                            retained_scrutinee_index: Some(0),
                                            deferred_constructor_case: None,
                                        },
                                    ));
                                    composed.extend_from_slice(eliminators);
                                    return self.lower_computational_producer_expr(
                                        builder,
                                        argument,
                                        producer_env,
                                        &composed,
                                    );
                                }
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
                                let lowered = self.lower_expr(builder, arg, producer_env)?;
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => {
                                        Ok(match lowered {
                                            LoweringOperand::Carried(word) => {
                                                LoweringOperand::Carried(word)
                                            }
                                            LoweringOperand::Specialized(value) => {
                                                LoweringOperand::Carried(
                                                    self.transfer_into_carrier(
                                                        builder,
                                                        arg.static_origin,
                                                        &value,
                                                    )?,
                                                )
                                            }
                                        })
                                    }
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures.into_iter().map(LoweringOperand::Specialized));
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::RecursiveDescent
                        ) {
                            call_env.extend_from_slice(producer_env);
                        }
                        match self.body_emission_authority {
                            BodyEmissionAuthority::RecursiveDescent => {
                                let body = self.retained_body_occurrence(body)?;
                                self.lower_computational_producer_expr(
                                    builder,
                                    body,
                                    &call_env,
                                    eliminators,
                                )
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                let returned =
                                    self.call_declared_unit(
                                        builder,
                                        body,
                                        &call_env,
                                        #[cfg(test)]
                                        None,
                                    )?;
                                self.lower_computational_match_value_composed(
                                    builder,
                                    returned,
                                    eliminators,
                                )
                            }
                        }
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
                        let recursive_unit_body = invocation.recursive_unit_body;
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
                            if let Some(body) = recursive_unit_body.filter(|_| {
                                matches!(
                                    self.body_emission_authority,
                                    BodyEmissionAuthority::FunctionizedUnits
                                )
                            }) {
                                let inputs = args
                                    .iter()
                                    .enumerate()
                                    .map(|(position, arg)| {
                                        let arg = self.child_occurrence(
                                            static_origin,
                                            1 + position,
                                            arg,
                                        )?;
                                        self.lower_expr(builder, arg, producer_env)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                self.enter_oriented_semantic_region(installed.checked);
                                let returned = self
                                    .call_declared_recursive_position_unit(
                                        builder,
                                        body,
                                        &inputs,
                                    )
                                    .and_then(|value| {
                                        self.lower_computational_match_value_composed(
                                            builder,
                                            value,
                                            &composed,
                                        )
                                    });
                                self.leave_oriented_semantic_region(installed.checked);
                                let returned = returned?;
                                return self.lower_computational_match_value_composed(
                                    builder,
                                    returned,
                                    eliminators,
                                );
                            }
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
                                let lowered = self.lower_expr(builder, arg, producer_env)?;
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => {
                                        Ok(match lowered {
                                            LoweringOperand::Carried(word) => {
                                                LoweringOperand::Carried(word)
                                            }
                                            LoweringOperand::Specialized(value) => {
                                                LoweringOperand::Carried(
                                                    self.transfer_into_carrier(
                                                        builder,
                                                        arg.static_origin,
                                                        &value,
                                                    )?,
                                                )
                                            }
                                        })
                                    }
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures.into_iter().map(LoweringOperand::Specialized));
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::RecursiveDescent
                        ) {
                            call_env.extend_from_slice(producer_env);
                        }
                        self.enter_oriented_semantic_region(installed.checked);
                        let returned = match self.body_emission_authority {
                            BodyEmissionAuthority::RecursiveDescent => {
                                let body = self.retained_body_occurrence(body)?;
                                self.lower_computational_producer_expr(
                                    builder,
                                    body,
                                    &call_env,
                                    &composed,
                                )
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                let returned =
                                    self.call_declared_unit(
                                        builder,
                                        body,
                                        &call_env,
                                        #[cfg(test)]
                                        None,
                                    )?;
                                self.lower_computational_match_value_composed(
                                    builder,
                                    returned,
                                    &composed,
                                )
                            }
                        };
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
                        synthesized_identity: Some(
                            self.static_transition_plan
                                .constructor_symbol_identity(static_origin)?,
                        ),
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
                let produced = if lowered_args
                    .iter()
                    .any(|argument| matches!(argument, LoweringOperand::Carried(_)))
                {
                    LoweringOperand::Carried(self.transfer_constructor_operands(
                        builder,
                        static_origin,
                        constructor,
                        &lowered_args,
                    )?)
                } else {
                    LoweringOperand::Specialized(Lowered::Constructor {
                        constructor: constructor.clone(),
                        // Carry the plan's already-resolved source identity with
                        // the template.  A later unit boundary may receive this
                        // result after nested producer traversal, where the
                        // caller occurrence is not the constructor occurrence
                        // and therefore cannot lawfully re-query its atom.
                        synthesized_identity: Some(
                            self.static_transition_plan
                                .constructor_symbol_identity(static_origin)?,
                        ),
                        args: specialized_env_at(&lowered_args, "a constructor argument")?,
                    })
                };
                self.lower_computational_match_value_composed(
                    builder,
                    produced,
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
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
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
                            self.merge_branch_value(
                                builder,
                                &join_plan,
                                lowered,
                                "ComputationalMatch",
                            )?;
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
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
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
                            self.merge_branch_value(
                                builder,
                                &join_plan,
                                lowered,
                                "ComputationalMatch",
                            )?;
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
                let LoweringOperand::Specialized(Lowered::Constructor {
                    constructor,
                    args,
                    ..
                }) = selected else {
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
                let join_plan = self.consumed_join_plan_token(static_origin)?;
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
                        self.merge_branch_value(
                            builder,
                            &join_plan,
                            lowered,
                            "ComputationalMatch",
                        )?;
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
        match eliminator {
            EliminatorFrame::Computational(frame) => {
                self.enter_source_occurrence_plan(frame.static_origin)?;
            }
            EliminatorFrame::Ordinary(frame) => {
                self.enter_source_occurrence_plan(frame.static_origin)?;
            }
            EliminatorFrame::PendingLet(_)
            | EliminatorFrame::InvocationReturn
            | EliminatorFrame::Active(_) => {}
        }
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
        let Lowered::Constructor {
            constructor,
            synthesized_identity,
            args,
        } = scrutinee else {
            return Err(unsupported(
                "ComputationalMatch",
                "scrutinee is not a constructor value after ordinary expression lowering",
            ));
        };
        let retained_scrutinee = Lowered::Constructor {
            constructor: constructor.clone(),
            synthesized_identity,
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
        let join_origin = match eliminator {
            EliminatorFrame::Computational(frame) => frame.static_origin,
            EliminatorFrame::Ordinary(frame) => frame.static_origin,
            EliminatorFrame::PendingLet(_)
            | EliminatorFrame::InvocationReturn
            | EliminatorFrame::Active(_) => {
                unreachable!("non-join eliminators returned before bounded-Nat emission")
            }
        };
        let join_plan = self.consumed_join_plan_token(join_origin)?;

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
            self.merge_scalar_branch(builder, &join_plan, zero_lowered, "BoundedNat")?;

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
        let (next, next_kind) =
            self.merge_scalar_branch(builder, &join_plan, suc_lowered, "BoundedNat")?;
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
            synthesized_identity: Some(
                self.static_transition_plan
                    .constructor_symbol_identity(deferred.construct_origin)?,
            ),
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
                } => match {
                    // The owned source machine is the third traversal route for
                    // these joins. Record the source occurrence here; later
                    // continuation helpers may only reborrow its token.
                    self.enter_source_occurrence_plan(static_origin)?;
                    expr
                } {
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
                                    static_origin,
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
                                static_origin,
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
                                edge.target.join_plan.as_ref(),
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
                            static_origin,
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
                                        static_origin,
                                        lowered,
                                    )?),
                                    control,
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::ConstructArgument {
                                    constructor,
                                    static_origin,
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
                                            static_origin,
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
                                LoweringOperand::Specialized(Lowered::Constructor {
                                    constructor,
                                    args,
                                    ..
                                }) => {
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
                                        None,
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
                let join_plan =
                    std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
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
                        join_plan,
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
                let join_plan =
                    std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
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
                    join_plan,
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
                let join_plan =
                    std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
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
                    join_plan,
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
        let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder.append_block_param(merge, types::I64);
        let target = SourceJoinTarget {
            join_id,
            block: merge,
            expected_outer: suffix_control.terminal_outer,
            required_kind,
            join_plan,
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
                let recursive_unit_body = invocation.recursive_unit_body;
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
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    if let Some(body) = recursive_unit_body.filter(|_| {
                        matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::FunctionizedUnits
                        )
                    }) {
                        let value = self.call_declared_recursive_position_unit(
                            builder,
                            body,
                            &args,
                        )?;
                        return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                            value,
                            control: suspended,
                        }));
                    }
                    Self::reject_carried_residual_arguments(args.len())?;
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
                    let mut suspended = armed.suspended;
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    if matches!(
                        self.body_emission_authority,
                        BodyEmissionAuthority::FunctionizedUnits
                    ) {
                        let value = self.call_declared_recursive_position_unit(
                            builder,
                            body,
                            &call_env,
                        )?;
                        return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                            value,
                            control: suspended,
                        }));
                    }
                    call_env.extend(env);
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
            append_recursive_argument_values(builder, &args, &mut values, &self.function_local.native_int_tags)?;
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
            &self.function_local.native_int_tags,
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
                &mut self.function_local.native_int_tags,
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
    /// purpose** and replaces it with the opposite one: one resolution route,
    /// observed behaviorally by
    /// `every_origin_to_expression_resolution_goes_through_the_single_route`.
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
    pub(super) fn retained_body_occurrence(
        &self,
        static_origin: StaticOriginId,
    ) -> Result<SourceOccurrence<'a>, CraneliftBackendError> {
        // ⭐ `AC-4`'s behavioural half. This route and
        // `StaticTransitionPlan::source_occurrence` are counted separately, and
        // the claim is that they move **together**: a resolution performed
        // without passing through here is the second route `AC-4` forbids, and
        // it shows up as `resolutions > invocations`.
        #[cfg(test)]
        crate::cranelift_backend::planning::ac4_note_route_invocation();
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
            LoweringOperand::Carried(word) => {
                #[cfg(test)]
                D8_CARRIED_JOIN_UNCHANGED.with(|count| count.set(count.get() + 1));
                Ok(word)
            }
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
                #[cfg(test)]
                D8_SPECIALIZED_JOIN_PRODUCTIONS.with(|count| count.set(count.get() + 1));
                let terminal_exit = self.process_object
                    && self
                        .function_local
                        .terminal_result_origins
                        .contains(&origin)
                    && matches!(
                        &lowered,
                        Lowered::Constructor { constructor, .. }
                            if constructor == &self.process_symbols.exit_success
                                || constructor == &self.process_symbols.exit_failure
                    );
                if terminal_exit {
                    let status = self.emit_process_exit_status(builder, lowered);
                    self.emit_carrier_immediate(
                        builder,
                        BoundaryTag::ImmediateExitStatus,
                        status,
                    )
                } else {
                    self.transfer_into_carrier(builder, origin, &lowered)
                }
            }
        }
    }

    /// Give one already-planned join exactly the lanes named by its D8 token.
    fn append_planned_join_params(
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
    ) {
        builder.append_block_param(merge, types::I64);
        if join_plan.representation == JoinResultRepresentation::NativeScalarPair {
            builder.append_block_param(merge, types::I64);
        }
    }

    /// Send one continuing predecessor through the representation selected
    /// before CFG emission. Source traps are sealed by the caller and never
    /// reach this value-only operation.
    fn jump_planned_join_arm(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
        origin: StaticOriginId,
        lowered: LoweringOperand,
        merge_kind: &mut Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<(), CraneliftBackendError> {
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                let (value, kind) =
                    self.merge_scalar_branch(builder, join_plan, lowered, join)?;
                Self::record_scalar_merge_kind(join, merge_kind, kind)?;
                builder
                    .ins()
                    .jump(merge, &[value.tag.into(), value.payload.into()]);
            }
            JoinResultRepresentation::CarrierWord => {
                let word = self.carried_join_arm(builder, origin, lowered, join)?;
                builder.ins().jump(merge, &[word.word.into()]);
            }
        }
        Ok(())
    }

    /// Recover the typed result of a planned join after all continuing
    /// predecessors have been emitted.
    fn finish_planned_join(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
        merge_kind: Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        builder.switch_to_block(merge);
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                let pair = NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                };
                let kind = merge_kind.ok_or_else(|| {
                    backend_module(format!(
                        "{join} has a continuing native predecessor without a result kind"
                    ))
                })?;
                Ok(LoweringOperand::Specialized(
                    self.lowered_from_scalar_pair(kind, pair),
                ))
            }
            JoinResultRepresentation::CarrierWord => {
                Ok(LoweringOperand::Carried(CarriedBoundaryWord {
                    word: builder.block_params(merge)[0],
                }))
            }
        }
    }

    /// Build one source constructor directly in the boundary carrier when at
    /// least one child has already crossed a generated-unit edge.
    ///
    /// The constructor identity and child origins still come exclusively from
    /// the static plan.  A carried child is stored unchanged; a specialized
    /// sibling crosses through the sole producer before both are joined in the
    /// same runtime node.
    fn transfer_constructor_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        constructor: &str,
        args: &[LoweringOperand],
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        if constructor == self.process_symbols.exit_failure {
            if let [LoweringOperand::Carried(code)] = args {
                return self.transfer_carried_failure_exit_status(builder, *code);
            }
        }
        let identity = self
            .static_transition_plan
            .constructor_symbol_identity(origin)?
            .tag_abi_word()?;
        let word = self.emit_carrier_alloc(
            builder,
            BoundaryTag::PersistentGround,
            BoundaryClass::Constructor,
            args.len(),
        )?;
        self.emit_carrier_store_tag_id(builder, word, identity)?;
        for (position, argument) in args.iter().enumerate() {
            let child_origin = self
                .static_transition_plan
                .child_static_origin(origin, position)?;
            let child = match argument {
                LoweringOperand::Carried(child) => *child,
                LoweringOperand::Specialized(value) => {
                    self.transfer_into_carrier(builder, child_origin, value)?
                }
            };
            self.emit_carrier_store_field(builder, word, position, child)?;
        }
        Ok(word)
    }

    /// Preserve the established process-exit mapping when the failure code
    /// crosses a unit edge before its enclosing constructor is lowered.
    ///
    /// Every valid native exit code is inside the immediate-Int domain. A
    /// non-immediate Int is therefore invalid without decoding its magnitude;
    /// the immediate arm reads the scalar through the carrier ABI and applies
    /// the same `0 -> 1`, `1..=255 -> self`, otherwise `-3` mapping used by
    /// `emit_process_exit_status`.
    fn transfer_carried_failure_exit_status(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        code: CarriedBoundaryWord,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let tag = builder.ins().band_imm(
            code.word,
            crate::boundary_value::BOUNDARY_TAG_MASK as i64,
        );
        let immediate = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            tag,
            BoundaryTag::ImmediateInt as i64,
        );
        let immediate_block = builder.create_block();
        let invalid_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);
        builder
            .ins()
            .brif(immediate, immediate_block, &[], invalid_block, &[]);

        builder.switch_to_block(immediate_block);
        let value = self.emit_carrier_scalar(builder, code)?;
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let malformed = builder.ins().iconst(types::I64, -3);
        let positive = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            value,
            zero,
        );
        let within_max = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            value,
            255,
        );
        let valid = builder.ins().band(positive, within_max);
        let nonzero = builder.ins().select(valid, value, malformed);
        let is_zero = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            value,
            zero,
        );
        let status = builder.ins().select(is_zero, one, nonzero);
        builder.ins().jump(merge, &[status.into()]);

        builder.switch_to_block(invalid_block);
        let malformed = builder.ins().iconst(types::I64, -3);
        builder.ins().jump(merge, &[malformed.into()]);

        builder.switch_to_block(merge);
        self.emit_carrier_immediate(
            builder,
            BoundaryTag::ImmediateExitStatus,
            builder.block_params(merge)[0],
        )
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
    /// | which Result carrier representation? | compile-time `Lowered` variant | `class(word)` |
    /// | which constructor? | `case.constructor == constructor` | `tag(word)` vs `case_constructor_identity` |
    /// | how many children? | `args.len()` | `field_count(word)` |
    /// | child *i*? | `args[i]` | `field(word, i)` — ⭐ **stays `Carried`** |
    /// | nothing matched? | a compile-time `Lowered::Trap` | a **runtime** closed default |
    ///
    /// ⭐ **Both columns read ONE identity authority** (`D2`). The producer
    /// wrote either `constructor_symbol_identity(..)` for a source occurrence
    /// or `synthesized_constructor_identity(..)` for a closed compiler role;
    /// this compares against `case_constructor_identity(..).tag_abi_word()`.
    /// Equal spellings intern to one canonical span, so the two agree **because
    /// they are the same number**, not because two derivations happen to
    /// coincide.
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
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        if cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        }
        // Only the closed process-input constructor family can arrive through
        // the borrowed-opaque lane.  Do not materialize that branch for an
        // ordinary carried match: Cranelift must compile both successors, so a
        // runtime class test alone would incorrectly require borrowed
        // identities for unrelated source constructors.
        let admits_borrowed_input = self.process_object
            && cases.iter().all(|case| {
                borrowed_constructor_identity(&self.process_symbols, &case.constructor).is_some()
            });
        if !admits_borrowed_input {
            return self.lower_nonborrowed_carried_match(
                builder,
                scrutinee,
                cases,
                default,
                static_origin,
                env,
                &join_plan,
            );
        }
        let class = self.emit_carrier_class(builder, scrutinee)?;
        let is_borrowed = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            class,
            BoundaryClass::BorrowedOpaque as i64,
        );
        let borrowed = builder.create_block();
        let represented = builder.create_block();
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            Self::append_planned_join_params(builder, merge, &join_plan);
        }
        let mut merge_kind = None;
        builder
            .ins()
            .brif(is_borrowed, borrowed, &[], represented, &[]);

        builder.switch_to_block(borrowed);
        let pointer = self.emit_carrier_scalar(builder, scrutinee)?;
        let borrowed_result = self.lower_borrowed_match(
            builder,
            pointer,
            cases,
            default,
            static_origin,
            env,
            &join_plan,
        )?;
        if Self::seal_source_trap_branch(builder, &borrowed_result) {
            // This runtime representation has no continuing predecessor.
        } else {
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                static_origin,
                borrowed_result,
                &mut merge_kind,
                "a carried borrowed-input match",
            )?;
        }

        builder.switch_to_block(represented);
        let represented_result = self.lower_nonborrowed_carried_match(
            builder,
            scrutinee,
            cases,
            default,
            static_origin,
            env,
            &join_plan,
        )?;
        if Self::seal_source_trap_branch(builder, &represented_result) {
            // This runtime representation has no continuing predecessor.
        } else {
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                static_origin,
                represented_result,
                &mut merge_kind,
                "a carried represented-value match",
            )?;
        }

        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        self.finish_planned_join(
            builder,
            merge,
            &join_plan,
            merge_kind,
            "a carried representation split",
        )
    }

    fn lower_nonborrowed_carried_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        join_plan: &JoinPlanToken,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐ Handled before any block is created, and that ordering matters: a
        // case-free match reaches the default unconditionally, so building a
        // merge block for it would leave one with no predecessor.
        if cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        }
        let ok_case = cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.constructor == self.process_symbols.result_ok);
        let err_case = cases
            .iter()
            .enumerate()
            .find(|(_, case)| case.constructor == self.process_symbols.result_err);
        if ok_case.is_some() || err_case.is_some() {
            let (Some(ok_case), Some(err_case)) = (ok_case, err_case) else {
                return Err(unsupported(
                    "HostResult",
                    "a carried HostResult match requires both closed Result cases",
                ));
            };
            if ok_case.1.binders != 1 || err_case.1.binders != 1 {
                return Err(unsupported(
                    "HostResult",
                    "carried Result cases must each bind exactly one selected payload",
                ));
            }
            // Dispatch both carried representations into one pair of source
            // case blocks. A nested source join is therefore emitted exactly
            // once even though either representation can select its owner.
            let ok_body = builder.create_block();
            builder.append_block_param(ok_body, types::I64);
            let err_body = builder.create_block();
            builder.append_block_param(err_body, types::I64);
            let merge = join_plan
                .has_continuing_predecessor
                .then(|| builder.create_block());
            if let Some(merge) = merge {
                Self::append_planned_join_params(builder, merge, join_plan);
            }
            let mut merge_kind = None;

            let class = self.emit_carrier_class(builder, scrutinee)?;
            let is_host_result = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                class,
                BoundaryClass::HostResult as i64,
            );
            let host_result = builder.create_block();
            let constructor = builder.create_block();
            builder
                .ins()
                .brif(is_host_result, host_result, &[], constructor, &[]);

            builder.switch_to_block(host_result);
            let success = self.emit_carrier_host_success(builder, scrutinee)?;
            let payload = self.emit_carrier_host_payload(builder, scrutinee)?;
            builder.ins().brif(
                success,
                ok_body,
                &[payload.word.into()],
                err_body,
                &[payload.word.into()],
            );

            builder.switch_to_block(constructor);
            let tag = self.emit_carrier_tag(builder, scrutinee)?;
            let field_count = self.emit_carrier_field_count(builder, scrutinee)?;
            for (body_block, (index, _case)) in
                [(ok_body, ok_case), (err_body, err_case)]
            {
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
                Self::require_i64(builder, field_count, 1);
                let payload = self.emit_carrier_field(builder, scrutinee, 0)?;
                builder.ins().jump(body_block, &[payload.word.into()]);
                builder.switch_to_block(next);
            }
            let defaulted = LoweringOperand::Specialized(Lowered::Trap(default.clone()));
            if !Self::seal_source_trap_branch(builder, &defaulted) {
                return Err(unsupported(
                    "Match",
                    "the carried Result match's closed default did not seal its branch",
                ));
            }

            for (block, (index, case)) in
                [(ok_body, ok_case), (err_body, err_case)]
            {
                builder.switch_to_block(block);
                let payload = CarriedBoundaryWord {
                    word: builder.block_params(block)[0],
                };
                let case_env = env_with_operands([LoweringOperand::Carried(payload)], env);
                let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                let body_origin = body.static_origin;
                let lowered = self.lower_expr(builder, body, &case_env)?;
                if Self::seal_source_trap_branch(builder, &lowered) {
                    continue;
                }
                let merge = merge.ok_or_else(|| {
                    backend_module(
                        "join plan omitted a merge despite a continuing predecessor".to_string(),
                    )
                })?;
                self.jump_planned_join_arm(
                    builder,
                    merge,
                    join_plan,
                    body_origin,
                    lowered,
                    &mut merge_kind,
                    "a carried Result arm",
                )?;
            }

            let Some(merge) = merge else {
                let unreachable_continuation = builder.create_block();
                builder.switch_to_block(unreachable_continuation);
                return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
            };
            return self.finish_planned_join(
                builder,
                merge,
                join_plan,
                merge_kind,
                "a carried Result join",
            );
        }

        self.lower_carried_constructor_match(
            builder,
            scrutinee,
            cases,
            default,
            static_origin,
            env,
            join_plan,
        )
    }

    fn lower_carried_constructor_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        join_plan: &JoinPlanToken,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // Read identity and arity ONCE, ahead of the chain: both are properties
        // of the scrutinee, not of any case, and re-reading per case would be a
        // second answer to a question that has one.
        let tag = self.emit_carrier_tag(builder, scrutinee)?;
        let field_count = self.emit_carrier_field_count(builder, scrutinee)?;

        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            Self::append_planned_join_params(builder, merge, join_plan);
        }
        let mut merge_kind = None;

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
            if Self::seal_source_trap_branch(builder, &lowered) {
                builder.switch_to_block(next);
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                join_plan,
                body_origin,
                lowered,
                &mut merge_kind,
                "a carried `Match` arm",
            )?;

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

        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        self.finish_planned_join(
            builder,
            merge,
            join_plan,
            merge_kind,
            "a carried `Match` join",
        )
    }

    /// Emit the declared call that evaluates one computational recursive
    /// position on the functionized path.
    ///
    /// Keeping this as a distinct operation makes the S1 boundary mechanical:
    /// a recursive position cannot accidentally return to source-body
    /// re-lowering without bypassing the one operation that emits its call.
    fn call_declared_recursive_position_unit(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        body_origin: StaticOriginId,
        inputs: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let result = self.call_declared_unit(
            builder,
            body_origin,
            inputs,
            #[cfg(test)]
            None,
        )?;
        #[cfg(test)]
        RECURSIVE_POSITION_UNIT_CALLS.with(|calls| calls.set(calls.get() + 1));
        Ok(result)
    }

    /// Resolve the declared body unit of a callable recursive position in the
    /// source form that owns the carried child.
    ///
    /// Structural-data recursive positions return `None`; they resume the
    /// eliminator directly and take no arguments. A lexical closure with
    /// captures also returns `None` because its carried value does not expose
    /// those capture operands to a generated call frame.
    fn recursive_position_unit_body(
        &self,
        eliminator_origin: StaticOriginId,
        position: usize,
    ) -> Result<Option<StaticOriginId>, CraneliftBackendError> {
        let eliminator = self.retained_body_occurrence(eliminator_origin)?;
        let RuntimeExpr::ComputationalMatch { scrutinee, .. } = eliminator.expr else {
            return Err(backend_module(
                "recursive-position metadata names a non-computational eliminator".to_string(),
            ));
        };
        let scrutinee =
            self.child_occurrence(eliminator_origin, 0, scrutinee)?;
        let RuntimeExpr::Construct { args, .. } = scrutinee.expr else {
            return Ok(None);
        };
        let Some(argument) = args.get(position) else {
            return Err(backend_module(
                "recursive position is outside its source constructor".to_string(),
            ));
        };
        let argument =
            self.child_occurrence(scrutinee.static_origin, position, argument)?;
        match argument.expr {
            RuntimeExpr::LexicalClosure {
                captures, body, ..
            } if captures.is_empty() => Ok(Some(
                self.child_occurrence(argument.static_origin, 0, body)?
                    .static_origin,
            )),
            RuntimeExpr::Closure { body, .. } => Ok(Some(
                self.child_occurrence(argument.static_origin, 0, body)?
                    .static_origin,
            )),
            RuntimeExpr::Value(_)
            | RuntimeExpr::Var(_)
            | RuntimeExpr::Let { .. }
            | RuntimeExpr::If { .. }
            | RuntimeExpr::PrimitiveCall { .. }
            | RuntimeExpr::Construct { .. }
            | RuntimeExpr::Match { .. }
            | RuntimeExpr::ComputationalMatch { .. }
            | RuntimeExpr::Record { .. }
            | RuntimeExpr::Project { .. }
            | RuntimeExpr::LexicalClosure { .. }
            | RuntimeExpr::DeclarationRef { .. }
            | RuntimeExpr::ImportedDeclarationRef { .. }
            | RuntimeExpr::Call { .. }
            | RuntimeExpr::Effect { .. }
            | RuntimeExpr::Trap(_)
            | RuntimeExpr::CheckedJoinSite { .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { .. }
            | RuntimeExpr::CheckedRecursiveInvocation { .. }
            | RuntimeExpr::CheckedComputationalIHSlots { .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { .. } => Ok(None),
        }
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
        if let Some((_, header)) = self
            .active_carried_computational_eliminations
            .iter()
            .rev()
            .find(|(origin, _)| *origin == eliminator.static_origin)
        {
            builder.ins().jump(*header, &[scrutinee.word.into()]);
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(LoweringOperand::Specialized(
                Lowered::RecursiveBackedge,
            ));
        }

        let header = builder.create_block();
        builder.append_block_param(header, types::I64);
        builder.ins().jump(header, &[scrutinee.word.into()]);
        builder.switch_to_block(header);
        let scrutinee = CarriedBoundaryWord {
            word: builder.block_params(header)[0],
        };
        self.active_carried_computational_eliminations
            .push((eliminator.static_origin, header));
        let lowered = self.lower_carried_computational_match_inner(
            builder,
            scrutinee,
            eliminator,
            remaining_eliminators,
        );
        let popped = self.active_carried_computational_eliminations.pop();
        debug_assert_eq!(
            popped,
            Some((eliminator.static_origin, header)),
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
                let child = self.emit_carrier_field(builder, scrutinee, position)?;
                // ⭐ The residual edge's oracle, written here and keyed on THIS
                // loop's own counter — before any selection among the children
                // happens. ⛔ Not derived from `recursive_positions`.
                #[cfg(test)]
                px8j_record_carrier_field_projection(
                    Px8jProducerPath::Composed,
                    position,
                    child,
                );
                children.push(LoweringOperand::Carried(child));
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
                        self.recursive_position_unit_body(
                            eliminator.static_origin,
                            position,
                        )?,
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
            if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                let word = self.carried_join_arm(
                    builder,
                    body_origin,
                    lowered,
                    "a carried `ComputationalMatch` arm",
                )?;
                builder.ins().jump(merge, &[word.word.into()]);
            }

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
    pub(super) fn lower_expr(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let SourceOccurrence {
            expr,
            static_origin,
        } = occurrence;
        self.enter_source_occurrence_plan(static_origin)?;
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
                    let (value, tag) = match lowered {
                        LoweringOperand::Specialized(Lowered::Int { value, known }) => {
                            (value, self.native_int_tag(builder, value, known)?)
                        }
                        LoweringOperand::Carried(word) => {
                            let carrier_tag = builder.ins().band_imm(
                                word.word,
                                crate::boundary_value::BOUNDARY_TAG_MASK as i64,
                            );
                            Self::require_i64(
                                builder,
                                carrier_tag,
                                crate::boundary_value::BoundaryTag::ImmediateInt as i64,
                            );
                            let value = self.emit_carrier_scalar(builder, word)?;
                            let _ = self.lower_dynamic_small_int(builder, value);
                            let tag = self.native_int_tag(builder, value, None)?;
                            (value, tag)
                        }
                        LoweringOperand::Specialized(_) => {
                            return Err(unsupported(
                                "If",
                                "dynamic native If arms must produce scalar Int values",
                            ));
                        }
                    };
                    builder.ins().jump(merge, &[tag.into(), value.into()]);
                }
                builder.switch_to_block(merge);
                let tag = builder.block_params(merge)[0];
                let value = builder.block_params(merge)[1];
                self.function_local.native_int_tags.insert(value, tag);
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
                if lowered_args
                    .iter()
                    .any(|argument| matches!(argument, LoweringOperand::Carried(_)))
                {
                    return Ok(LoweringOperand::Carried(
                        self.transfer_constructor_operands(
                            builder,
                            static_origin,
                            constructor,
                            &lowered_args,
                        )?,
                    ));
                }
                Ok(LoweringOperand::Specialized(Lowered::Constructor {
                    constructor: constructor.clone(),
                    synthesized_identity: Some(
                        self.static_transition_plan
                            .constructor_symbol_identity(static_origin)?,
                    ),
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
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    return self.lower_borrowed_match(
                        builder,
                        pointer,
                        cases,
                        default,
                        static_origin,
                        env,
                        &join_plan,
                    );
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
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
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
                            self.merge_scalar_branch(builder, &join_plan, lowered, "Match")?;
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
                let LoweringOperand::Specialized(Lowered::Constructor {
                    constructor,
                    args,
                    ..
                }) = lowered_scrutinee else {
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
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                let callee = self.child_occurrence(static_origin, 0, callee)?;
                if matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                ) {
                    if let RuntimeExpr::LexicalClosure {
                        captures,
                        params,
                        body,
                    } = callee.expr
                    {
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
                        let mut inputs = args
                            .iter()
                            .enumerate()
                            .map(|(position, argument)| {
                                let argument = self.child_occurrence(
                                    static_origin,
                                    1 + position,
                                    argument,
                                )?;
                                self.lower_expr(builder, argument, env)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let closure_origin = callee.static_origin;
                        inputs.extend(
                            captures
                                .iter()
                                .enumerate()
                                .map(|(position, capture)| {
                                    let capture = self.child_occurrence(
                                        closure_origin,
                                        1 + position,
                                        capture,
                                    )?;
                                    self.lower_expr(builder, capture, env)
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        );
                        let body = self
                            .child_occurrence(closure_origin, 0, body)?
                            .static_origin;
                        return self.call_declared_unit(
                            builder,
                            body,
                            &inputs,
                            #[cfg(test)]
                            None,
                        );
                    }
                }
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
                        join_plan,
                    ),
                    LoweringOperand::Specialized(Lowered::Closure {
                        captures,
                        params,
                        body,
                    }) => {
                        let mut call_env = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                let lowered = self.lower_expr(builder, arg, env)?;
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => {
                                        Ok(match lowered {
                                            LoweringOperand::Carried(word) => {
                                                LoweringOperand::Carried(word)
                                            }
                                            LoweringOperand::Specialized(value) => {
                                                LoweringOperand::Carried(
                                                    self.transfer_into_carrier(
                                                        builder,
                                                        arg.static_origin,
                                                        &value,
                                                    )?,
                                                )
                                            }
                                        })
                                    }
                                }
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
                        match self.body_emission_authority {
                            BodyEmissionAuthority::RecursiveDescent => {
                                let body = self.retained_body_occurrence(body)?;
                                self.lower_expr(builder, body, &call_env)
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                self.call_declared_unit(
                                    builder,
                                    body,
                                    &call_env,
                                    #[cfg(test)]
                                    None,
                                )
                            }
                        }
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
                        let recursive_unit_body = invocation.recursive_unit_body;
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
                            if let Some(body) = recursive_unit_body.filter(|_| {
                                matches!(
                                    self.body_emission_authority,
                                    BodyEmissionAuthority::FunctionizedUnits
                                )
                            }) {
                                let inputs = args
                                    .iter()
                                    .enumerate()
                                    .map(|(position, arg)| {
                                        let arg = self.child_occurrence(
                                            static_origin,
                                            1 + position,
                                            arg,
                                        )?;
                                        self.lower_expr(builder, arg, env)
                                    })
                                    .collect::<Result<Vec<_>, _>>()?;
                                self.enter_oriented_semantic_region(installed.checked);
                                let result = self
                                    .call_declared_recursive_position_unit(
                                        builder,
                                        body,
                                        &inputs,
                                    )
                                    .and_then(|value| {
                                        self.lower_computational_match_value_composed(
                                            builder,
                                            value,
                                            &frames,
                                        )
                                    });
                                self.leave_oriented_semantic_region(installed.checked);
                                return result;
                            }
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
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::FunctionizedUnits
                        ) {
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self
                                .call_declared_recursive_position_unit(
                                    builder,
                                    body,
                                    &call_env,
                                )
                                .and_then(|value| {
                                    self.lower_computational_match_value_composed(
                                        builder,
                                        value,
                                        &frames,
                                    )
                                });
                            self.leave_oriented_semantic_region(installed.checked);
                            return result;
                        }
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

    fn lower_buffer_freeze_resource_seat(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        operand: &LoweringOperand,
        seat: &'static str,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        match operand {
            LoweringOperand::Specialized(Lowered::ResourceToken { value }) => Ok(*value),
            LoweringOperand::Specialized(_) => Err(unsupported(
                "Effect",
                format!("BufferFreeze {seat} is not a resource"),
            )),
            LoweringOperand::Carried(word) => {
                let tag = self.emit_carrier_tag(builder, *word)?;
                Self::require_i64(
                    builder,
                    tag,
                    crate::boundary_value::BoundaryTag::InvocationBorrowed as i64,
                );
                let class = self.emit_carrier_class(builder, *word)?;
                Self::require_i64(
                    builder,
                    class,
                    BoundaryClass::BorrowedOpaque as i64,
                );
                self.emit_carrier_scalar(builder, *word)
            }
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
        // BufferFreeze has two ruled phase-bearing resource seats. Every other
        // host operation remains specialized-only and crosses the typed phase
        // boundary only after the checked operation is known.
        let specialized_lowered = if operation == ken_host::HostOpV1::BufferFreeze {
            None
        } else {
            Some(specialized_env_at(&lowered, "a host-effect operand")?)
        };
        let pointer_type = builder.func.dfg.value_type(
            self.function_local
                .host_dispatch_context
                .expect("process effect lowering owns a direct host context"),
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
                let lowered = specialized_lowered
                    .as_deref()
                    .expect("non-BufferFreeze operands crossed the specialized boundary");
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
                let lowered = specialized_lowered
                    .as_deref()
                    .expect("non-BufferFreeze operands crossed the specialized boundary");
                let capability = capability
                    .ok_or_else(|| unsupported("Effect", "FS operation has no live capability"))?;
                // Present ⇒ the capability value is child 0 of this occurrence.
                let capability_value =
                    self.child_occurrence(static_origin, 0, &capability.value)?;
                let token = match self.lower_expr(builder, capability_value, env)? {
                    LoweringOperand::Specialized(Lowered::CapabilityToken { value }) => value,
                    LoweringOperand::Carried(word) => {
                        self.emit_carrier_scalar(builder, word)?
                    }
                    _ => {
                        return Err(unsupported(
                            "Effect",
                            "FS capability operand is not the opaque invocation token",
                        ));
                    }
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
                let lowered = specialized_lowered
                    .as_deref()
                    .expect("non-BufferFreeze operands crossed the specialized boundary");
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
                let lowered = specialized_lowered
                    .as_deref()
                    .expect("non-BufferFreeze operands crossed the specialized boundary");
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
                let token = self.lower_buffer_freeze_resource_seat(
                    builder,
                    lowered
                    .first()
                        .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its buffer"))?,
                    "buffer",
                )?;
                let start = lowered
                    .get(1)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its start"))?;
                let length = lowered
                    .get(2)
                    .ok_or_else(|| unsupported("Effect", "BufferFreeze is missing its length"))?;
                let (LoweringOperand::Specialized(start), LoweringOperand::Specialized(length)) =
                    (start, length)
                else {
                    return Err(unsupported(
                        "Effect",
                        "BufferFreeze start and length must remain specialized Int operands",
                    ));
                };
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
                let span_origin = self.lower_buffer_freeze_resource_seat(
                    builder,
                    lowered.get(3).ok_or_else(|| {
                        unsupported("Effect", "BufferFreeze is missing its span origin")
                    })?,
                    "span origin",
                )?;
                for (index, value) in [token, start, length, span_origin].into_iter().enumerate() {
                    builder
                        .ins()
                        .stack_store(value, request, request_offset(index));
                }
            }
            ken_host::HostOpV1::FsReadAt | ken_host::HostOpV1::FsWriteAt => {
                let lowered = specialized_lowered
                    .as_deref()
                    .expect("non-BufferFreeze operands crossed the specialized boundary");
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
        let host_context = self
            .function_local
            .host_dispatch_context
            .expect("process effect lowering owns a direct host context");
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
                self.function_local.host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[
                    host_context,
                    op,
                    request_pointer,
                    request_size,
                    reply_pointer,
                ],
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
                self.function_local.host_dispatch
                    .expect("process effect lowering owns one host dispatch import"),
                &[
                    host_context,
                    op,
                    request_pointer,
                    request_size,
                    reply_pointer,
                ],
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
            let io_error = Lowered::DynamicConstructor(DynamicConstructorV1 {
                discriminator: builder.ins().band_imm(detail, 0xff),
                alternatives: self.synthesized_io_error_alternatives(payload_int)?,
            });
            let error = if matches!(
                operation,
                ken_host::HostOpV1::FsReadFile
                    | ken_host::HostOpV1::FsWriteFile
                    | ken_host::HostOpV1::FsChangeMode
                    | ken_host::HostOpV1::FsOpen
            ) {
                let path = specialized_lowered
                    .as_ref()
                    .expect("file-result synthesis follows a specialized-only operation")
                    .first()
                    .cloned()
                    .expect("validated FS operation has a path");
                let (operation_role, operation_symbol) = match operation {
                    ken_host::HostOpV1::FsReadFile | ken_host::HostOpV1::FsOpen => (
                        SynthesizedFixedConstructorRole::FileOperationRead,
                        self.process_symbols.file_operation_read.clone(),
                    ),
                    ken_host::HostOpV1::FsWriteFile => (
                        SynthesizedFixedConstructorRole::FileOperationWrite,
                        self.process_symbols.file_operation_write.clone(),
                    ),
                    ken_host::HostOpV1::FsChangeMode => (
                        SynthesizedFixedConstructorRole::FileOperationChangeMode,
                        self.process_symbols.file_operation_change_mode.clone(),
                    ),
                    _ => unreachable!("validated FS result operation"),
                };
                let operation =
                    self.synthesized_constructor(operation_role, operation_symbol, Vec::new())?;
                let path = self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::OptionSome,
                    self.process_symbols.option_some.clone(),
                    vec![path],
                )?;
                self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::FileError,
                    self.process_symbols.file_error.clone(),
                    vec![operation, path, io_error],
                )?
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
                        .synthesized_io_error_alternatives(surface_io_payload_int)?,
                });
                let identity_low = builder.ins().band_imm(resource_identity, 0xffff_ffff);
                let identity_high = builder.ins().ushr_imm(resource_identity, 32);
                let identity_low_int = self.lower_dynamic_small_int(builder, identity_low);
                let identity_high_int = self.lower_dynamic_small_int(builder, identity_high);
                let resource_kind_value = |this: &Self, discriminator| {
                    Ok::<_, CraneliftBackendError>(Lowered::DynamicConstructor(
                        DynamicConstructorV1 {
                        discriminator,
                        alternatives: vec![
                            this.synthesized_dynamic_alternative(
                                wire.resource_kind_fs_handle as i64,
                                SynthesizedFixedConstructorRole::ResourceKindFsHandle,
                                this.process_symbols.resource_kind_fs_handle.clone(),
                                Vec::new(),
                            )?,
                            this.synthesized_dynamic_alternative(
                                wire.resource_kind_buffer as i64,
                                SynthesizedFixedConstructorRole::ResourceKindBuffer,
                                this.process_symbols.resource_kind_buffer.clone(),
                                Vec::new(),
                            )?,
                        ],
                    }))
                };
                let trace_identity = self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::ResourceTraceIdentity,
                    self.process_symbols.resource_trace_identity.clone(),
                    vec![identity_low_int, identity_high_int],
                )?;
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: surface_tag,
                    alternatives: vec![
                        self.synthesized_dynamic_alternative(
                            0,
                            SynthesizedFixedConstructorRole::ResourceHostIo,
                            self.process_symbols.resource_host_io.clone(),
                            vec![surface_io_error.clone()],
                        )?,
                        self.synthesized_dynamic_alternative(
                            1,
                            SynthesizedFixedConstructorRole::ResourceClosed,
                            self.process_symbols.resource_closed.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            2,
                            SynthesizedFixedConstructorRole::ResourceMalformed,
                            self.process_symbols.resource_malformed.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            3,
                            SynthesizedFixedConstructorRole::ResourceRightNotHeld,
                            self.process_symbols.resource_right_not_held.clone(),
                            vec![resource_required_int, resource_held_int],
                        )?,
                        self.synthesized_dynamic_alternative(
                            4,
                            SynthesizedFixedConstructorRole::ResourceReleaseFailed,
                            self.process_symbols.resource_release_failed.clone(),
                            vec![
                                resource_kind_value(self, resource_kind)?,
                                trace_identity,
                                surface_io_error,
                            ],
                        )?,
                        self.synthesized_dynamic_alternative(
                            5,
                            SynthesizedFixedConstructorRole::ResourceKindMismatch,
                            self.process_symbols.resource_kind_mismatch.clone(),
                            vec![
                                resource_kind_value(self, resource_expected_kind)?,
                                resource_kind_value(self, resource_actual_kind)?,
                            ],
                        )?,
                        self.synthesized_dynamic_alternative(
                            6,
                            SynthesizedFixedConstructorRole::ResourceBufferLimit,
                            self.process_symbols.resource_buffer_limit.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            7,
                            SynthesizedFixedConstructorRole::ResourceInvalidOffset,
                            self.process_symbols.resource_invalid_offset.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            8,
                            SynthesizedFixedConstructorRole::ResourceInvalidBounds,
                            self.process_symbols.resource_invalid_bounds.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            9,
                            SynthesizedFixedConstructorRole::ResourceNoProgress,
                            self.process_symbols.resource_no_progress.clone(),
                            Vec::new(),
                        )?,
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
                let Lowered::ResourceToken { value: span_origin } = specialized_lowered
                    .as_ref()
                    .expect("FsReadAt result synthesis follows a specialized-only operation")
                    .get(2)
                    .ok_or_else(|| unsupported("Effect", "FsReadAt is missing its buffer operand"))?
                else {
                    return Err(unsupported(
                        "Effect",
                        "FsReadAt buffer operand is not a resource",
                    ));
                };
                let span_origin = *span_origin;
                let span = self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::PrivateBufferSpan,
                    self.process_symbols.private_buffer_span.clone(),
                    vec![
                        Lowered::ResourceToken { value: span_origin },
                        reply_start_int,
                        Lowered::BoundedNat(count),
                    ],
                )?;
                let transferred = self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::PrivateTransferCount,
                    self.process_symbols.private_transfer_count.clone(),
                    vec![
                        Lowered::BoundedNat(predecessor),
                        Lowered::BoundedNat(remaining),
                    ],
                )?;
                Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: builder.ins().uextend(types::I64, nonzero),
                    alternatives: vec![
                        self.synthesized_dynamic_alternative(
                            0,
                            SynthesizedFixedConstructorRole::ReadEof,
                            self.process_symbols.read_eof.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            1,
                            SynthesizedFixedConstructorRole::ReadSome,
                            self.process_symbols.read_some.clone(),
                            vec![span, transferred],
                        )?,
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
                let transferred = self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::PrivateTransferCount,
                    self.process_symbols.private_transfer_count.clone(),
                    vec![
                            Lowered::BoundedNat(predecessor),
                            Lowered::BoundedNat(remaining),
                    ],
                )?;
                self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::Wrote,
                    self.process_symbols.wrote.clone(),
                    vec![transferred],
                )?
            } else if operation == ken_host::HostOpV1::FsHandleMetadata {
                self.lower_unsigned_u64_int(builder, detail)?
            } else {
                self.synthesized_constructor(
                    SynthesizedFixedConstructorRole::Unit,
                    self.process_symbols.unit.clone(),
                    Vec::new(),
                )?
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
        join_origin: StaticOriginId,
        symbol: &RuntimeSymbol,
        captures: &[Lowered],
        argument: Lowered,
        zero_body: SourceOccurrence<'_>,
        suc_body: SourceOccurrence<'_>,
        producer_env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let join_plan = self.consumed_join_plan_token(join_origin)?;
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
            self.merge_scalar_branch(builder, &join_plan, zero_lowered, "DeclarationRef")?;
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
        let (next, next_kind) =
            self.merge_scalar_branch(builder, &join_plan, next?, "DeclarationRef")?;
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
        join_plan: JoinPlanToken,
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
                &self.function_local.native_int_tags,
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
                            body.static_origin,
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
            &self.function_local.native_int_tags,
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
                &mut self.function_local.native_int_tags,
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
        let (value, result_kind) =
            self.merge_scalar_branch(builder, &join_plan, lowered, "DeclarationRef")?;
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
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        join_plan: &JoinPlanToken,
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
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            Self::append_planned_join_params(builder, merge, join_plan);
        }
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
            if !Self::seal_source_trap_branch(builder, &lowered) {
                let merge = merge.ok_or_else(|| {
                    backend_module(
                        "join plan omitted a merge despite a continuing predecessor".to_string(),
                    )
                })?;
                self.jump_planned_join_arm(
                    builder,
                    merge,
                    join_plan,
                    body.static_origin,
                    lowered,
                    &mut merge_kind,
                    "a borrowed `Match` arm",
                )?;
            }
            test_block = next;
        }
        builder.switch_to_block(test_block);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        };
        self.finish_planned_join(
            builder,
            merge,
            join_plan,
            merge_kind,
            "a borrowed `Match` join",
        )
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
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            Self::append_planned_join_params(builder, merge, &join_plan);
        }
        let some_block = builder.create_block();
        let none_block = builder.create_block();
        let mut merge_kind = None;
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
            if Self::seal_source_trap_branch(builder, &lowered) {
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "borrowed Option join omitted a merge despite a continuing predecessor"
                        .to_string(),
                )
            })?;
            self.jump_planned_join_arm(
                builder,
                merge,
                &join_plan,
                body.static_origin,
                lowered,
                &mut merge_kind,
                "Match",
            )?;
        }
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "all borrowed Option alternatives trap".to_string(),
            })));
        };
        self.finish_planned_join(builder, merge, &join_plan, merge_kind, "Match")
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
        // D8: the source traversal consumed the origin-keyed contract before
        // reaching this helper. Reborrow it before creating a block or lowering
        // either arm. The specialized HostResult scrutinee is not a selector
        // for the result representation.
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            #[cfg(test)]
            D8_JOIN_MERGES_CREATED.with(|count| count.set(count.get() + 1));
            builder.append_block_param(merge, types::I64);
            if join_plan.representation == JoinResultRepresentation::NativeScalarPair {
                builder.append_block_param(merge, types::I64);
            }
        }
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
            if Self::seal_source_trap_branch(builder, &lowered) {
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            match join_plan.representation {
                JoinResultRepresentation::NativeScalarPair => {
                    let (value, branch_kind) =
                        self.merge_scalar_branch(builder, &join_plan, lowered, "Match")?;
                    Self::record_scalar_merge_kind("Match", &mut merge_kind, branch_kind)?;
                    builder
                        .ins()
                        .jump(merge, &[value.tag.into(), value.payload.into()]);
                }
                JoinResultRepresentation::CarrierWord => {
                    let word =
                        self.carried_join_arm(builder, body.static_origin, lowered, "Match")?;
                    builder.ins().jump(merge, &[word.word.into()]);
                }
            }
        }
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(
                RuntimeTrap {
                    code: RuntimeTrapCode::PatternMatchFailure,
                    message: "all HostResult match alternatives trap".to_string(),
                },
            )));
        };
        builder.switch_to_block(merge);
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                let pair = NativeScalarPairV1 {
                    tag: builder.block_params(merge)[0],
                    payload: builder.block_params(merge)[1],
                };
                Ok(LoweringOperand::Specialized(self.lowered_from_scalar_pair(
                    merge_kind.expect("HostResult emits a continuing closed alternative"),
                    pair,
                )))
            }
            JoinResultRepresentation::CarrierWord => {
                Ok(LoweringOperand::Carried(CarriedBoundaryWord {
                    word: builder.block_params(merge)[0],
                }))
            }
        }
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
        let join_plan = self.consumed_join_plan_token(static_origin)?;
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
            let (value, kind) =
                self.merge_scalar_branch(builder, &join_plan, lowered, "BoundedNat")?;
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
        let static_origin = match continuation {
            DynamicConstructorContinuation::Ordinary { static_origin, .. }
            | DynamicConstructorContinuation::Producer { static_origin, .. } => static_origin,
        };
        let join_plan = self.consumed_join_plan_token(static_origin)?;
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
                self.merge_scalar_branch(builder, &join_plan, lowered, "DynamicConstructor")?;
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

        // A primitive's static symbol determines whether its operands are
        // scalar Ints or Bools. A carried word in one of those positions is
        // projected through the emitted scalar helper; no runtime tag chooses
        // which source type the operand is.
        let scalar_kind = match primitive.symbol.as_str() {
            "add_int" | "sub_int" | "mul_int" | "eq_int" | "leq_int"
            | "uint8_to_int" | "int_to_uint8_raw" => Some("Int"),
            "not_bool" | "and_bool" | "or_bool" => Some("Bool"),
            _ => None,
        };
        let lowered_args = if primitive.symbol == "bytes_length" {
            match lowered_args.as_slice() {
                [LoweringOperand::Specialized(_)] => specialized_env_at(
                    &lowered_args,
                    "the bytes_length operand",
                )?,
                [LoweringOperand::Carried(word)] => {
                    let class = self.emit_carrier_class(builder, *word)?;
                    Self::require_i64(
                        builder,
                        class,
                        BoundaryClass::BorrowedOpaque as i64,
                    );
                    let pointer = self.emit_carrier_scalar(builder, *word)?;
                    vec![Lowered::BorrowedNativeValue { pointer }]
                }
                _ => {
                    return Err(unsupported(
                        "PrimitiveCall",
                        "bytes_length requires exactly one bytes operand",
                    ));
                }
            }
        } else if primitive.symbol == "bytes_at" {
            lowered_args
                .into_iter()
                .enumerate()
                .map(|(position, arg)| match (position, arg) {
                    (_, LoweringOperand::Specialized(value)) => Ok(value),
                    (0, LoweringOperand::Carried(word)) => {
                        let class = self.emit_carrier_class(builder, word)?;
                        Self::require_i64(
                            builder,
                            class,
                            BoundaryClass::BorrowedOpaque as i64,
                        );
                        let pointer = self.emit_carrier_scalar(builder, word)?;
                        Ok(Lowered::BorrowedNativeValue { pointer })
                    }
                    (1, LoweringOperand::Carried(word)) => {
                        let tag = builder.ins().band_imm(
                            word.word,
                            crate::boundary_value::BOUNDARY_TAG_MASK as i64,
                        );
                        Self::require_i64(
                            builder,
                            tag,
                            crate::boundary_value::BoundaryTag::ImmediateInt as i64,
                        );
                        let value = self.emit_carrier_scalar(builder, word)?;
                        Ok(self.lower_dynamic_small_int(builder, value))
                    }
                    (_, LoweringOperand::Carried(_)) => Err(unsupported(
                        "PrimitiveCall",
                        "bytes_at received more operands than its closed static signature",
                    )),
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?
        } else if let Some(kind) = scalar_kind {
            lowered_args
                .into_iter()
                .map(|arg| match arg {
                    LoweringOperand::Specialized(value) => Ok(value),
                    LoweringOperand::Carried(word) => {
                        let value = self.emit_carrier_scalar(builder, word)?;
                        let tag = builder.ins().band_imm(
                            word.word,
                            crate::boundary_value::BOUNDARY_TAG_MASK as i64,
                        );
                        Ok(match kind {
                            "Int" => {
                                Self::require_i64(
                                    builder,
                                    tag,
                                    crate::boundary_value::BoundaryTag::ImmediateInt as i64,
                                );
                                self.lower_dynamic_small_int(builder, value)
                            }
                            "Bool" => {
                                Self::require_i64(
                                    builder,
                                    tag,
                                    crate::boundary_value::BoundaryTag::ImmediateBool as i64,
                                );
                                Lowered::Bool { value, known: None }
                            }
                            _ => unreachable!("closed primitive scalar kind"),
                        })
                    }
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?
        } else {
            specialized_env_at(&lowered_args, "a primitive-call operand")?
        };
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
