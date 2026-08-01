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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RecursiveDescentResidual {
    /// An ordinary producer match whose scrutinee is directly a call.
    ProducerMatchCall,
    /// A call whose callee is the retained non-lexical closure form.
    SeedClosureCall,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RecursiveDescentResidualOwner {
    RootExpression,
    Declaration(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RecursiveDescentResidualReport {
    by_owner: BTreeMap<RecursiveDescentResidualOwner, BTreeSet<RecursiveDescentResidual>>,
}

impl RecursiveDescentResidualReport {
    fn record(&mut self, owner: RecursiveDescentResidualOwner, residual: RecursiveDescentResidual) {
        self.by_owner.entry(owner).or_default().insert(residual);
    }

    fn variants(&self) -> BTreeSet<RecursiveDescentResidual> {
        self.by_owner
            .values()
            .flat_map(|residuals| residuals.iter().copied())
            .collect()
    }
}

/// Visit every retained reason from the exhaustive source walk.
///
/// Wrapper and child-producing forms propagate a reason from their children.
/// The exhaustive match is the fail-closed default: a new `RuntimeExpr` form
/// cannot compile until this production classifier assigns it to the
/// functionized population or to a typed retained reason.
///
/// `visit` returns whether traversal should continue. The selector stops on the
/// first reason; D1 continues through the same classifier.
fn walk_recursive_descent_residuals(
    expr: &RuntimeExpr,
    visit: &mut impl FnMut(RecursiveDescentResidual) -> bool,
) -> bool {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
        | RuntimeExpr::Closure { body, .. } => walk_recursive_descent_residuals(body, visit),
        RuntimeExpr::LexicalClosure { captures, body, .. } => {
            captures
                .iter()
                .all(|capture| walk_recursive_descent_residuals(capture, visit))
                && walk_recursive_descent_residuals(body, visit)
        }
        RuntimeExpr::Let { value, body } => {
            walk_recursive_descent_residuals(value, visit)
                && walk_recursive_descent_residuals(body, visit)
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            walk_recursive_descent_residuals(scrutinee, visit)
                && walk_recursive_descent_residuals(then_expr, visit)
                && walk_recursive_descent_residuals(else_expr, visit)
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => args
            .iter()
            .all(|argument| walk_recursive_descent_residuals(argument, visit)),
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            if matches!(scrutinee.as_ref(), RuntimeExpr::Call { .. })
                && !visit(RecursiveDescentResidual::ProducerMatchCall)
            {
                return false;
            }
            walk_recursive_descent_residuals(scrutinee, visit)
                && cases
                    .iter()
                    .all(|case| walk_recursive_descent_residuals(&case.body, visit))
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            walk_recursive_descent_residuals(scrutinee, visit)
                && cases
                    .iter()
                    .all(|case| walk_recursive_descent_residuals(&case.body, visit))
        }
        RuntimeExpr::Record { fields } => fields
            .iter()
            .all(|(_, value)| walk_recursive_descent_residuals(value, visit)),
        RuntimeExpr::Project { record, .. } => walk_recursive_descent_residuals(record, visit),
        RuntimeExpr::Call { callee, args } => {
            if matches!(callee.as_ref(), RuntimeExpr::Closure { .. })
                && !visit(RecursiveDescentResidual::SeedClosureCall)
            {
                return false;
            }
            walk_recursive_descent_residuals(callee, visit)
                && args
                    .iter()
                    .all(|argument| walk_recursive_descent_residuals(argument, visit))
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            capability.as_ref().map_or(true, |capability| {
                walk_recursive_descent_residuals(&capability.value, visit)
            }) && args
                .iter()
                .all(|argument| walk_recursive_descent_residuals(argument, visit))
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => true,
    }
}

fn recursive_descent_residual(expr: &RuntimeExpr) -> Option<RecursiveDescentResidual> {
    let mut first = None;
    walk_recursive_descent_residuals(expr, &mut |residual| {
        first = Some(residual);
        false
    });
    first
}

fn walk_declaration_recursive_descent_residuals(
    declaration: &RuntimeDeclaration,
    visit: &mut impl FnMut(RecursiveDescentResidual) -> bool,
) -> bool {
    match &declaration.kind {
        RuntimeDeclarationKind::Transparent { body } => {
            walk_recursive_descent_residuals(body, visit)
        }
        RuntimeDeclarationKind::Primitive { .. }
        | RuntimeDeclarationKind::Data { .. }
        | RuntimeDeclarationKind::Record { .. }
        | RuntimeDeclarationKind::RecursiveGroup { .. }
        | RuntimeDeclarationKind::EffectBoundary { .. }
        | RuntimeDeclarationKind::MetadataOnly => true,
    }
}

fn declaration_recursive_descent_residual(
    declaration: &RuntimeDeclaration,
) -> Option<RecursiveDescentResidual> {
    let mut first = None;
    walk_declaration_recursive_descent_residuals(declaration, &mut |residual| {
        first = Some(residual);
        false
    });
    first
}

fn recursive_descent_residual_report(
    expr: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
) -> RecursiveDescentResidualReport {
    let mut report = RecursiveDescentResidualReport::default();
    walk_recursive_descent_residuals(expr, &mut |residual| {
        report.record(RecursiveDescentResidualOwner::RootExpression, residual);
        true
    });
    for declaration in declarations.values() {
        let owner = RecursiveDescentResidualOwner::Declaration(declaration.symbol.clone());
        walk_declaration_recursive_descent_residuals(declaration, &mut |residual| {
            report.record(owner.clone(), residual);
            true
        });
    }
    report
}

fn emit_recursive_descent_residual_diagnostic(
    authority: BodyEmissionAuthority,
    report: &RecursiveDescentResidualReport,
) {
    eprintln!("RT_DECL_CLOSURE_PORT_D1 authority={authority:?}");
    if report.by_owner.is_empty() {
        eprintln!("RT_DECL_CLOSURE_PORT_D1 residuals=none");
        return;
    }
    for (owner, residuals) in &report.by_owner {
        let owner = match owner {
            RecursiveDescentResidualOwner::RootExpression => "<root>",
            RecursiveDescentResidualOwner::Declaration(symbol) => symbol,
        };
        for residual in residuals {
            eprintln!("RT_DECL_CLOSURE_PORT_D1 owner={owner} residual={residual:?}");
        }
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
    module: M,
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
    compile_expr_into_module_with_root_projection(
        module,
        function_name,
        linkage,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        process_mode,
        process_symbols,
        native_join_plan,
        oriented_subcontinuation_plan,
        false,
        false,
    )
}

/// Compile an object entry whose public scalar launcher consumes a scalar,
/// while generated-unit calls continue to exchange their planner-selected
/// carrier words internally.
pub(in crate::cranelift_backend) fn compile_expr_into_object_module<'a, M: Module>(
    module: M,
    function_name: &str,
    linkage: Linkage,
    expr: &'a RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
) -> Result<CompiledModule<M>, CraneliftBackendError> {
    compile_expr_into_module_with_root_projection(
        module,
        function_name,
        linkage,
        expr,
        seed_env,
        declarations,
        staged_process_input,
        process_mode,
        process_symbols,
        native_join_plan,
        oriented_subcontinuation_plan,
        !process_mode,
        process_mode,
    )
}

#[allow(clippy::too_many_arguments)]
fn compile_expr_into_module_with_root_projection<'a, M: Module>(
    mut module: M,
    function_name: &str,
    linkage: Linkage,
    expr: &'a RuntimeExpr,
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    staged_process_input: Option<&RuntimeValue>,
    process_mode: bool,
    process_symbols: Option<&crate::NativeProcessSymbols>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
    project_public_scalar_root: bool,
    root_trap_process_sentinel: bool,
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
    let body_emission_authority = select_body_emission_authority(expr, &declarations);
    if std::env::var_os("KEN_RT_DECL_CLOSURE_PORT_D1").is_some() {
        emit_recursive_descent_residual_diagnostic(
            body_emission_authority,
            &recursive_descent_residual_report(expr, &declarations),
        );
    }
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
    let static_transition_plan = plan_static_transition_graph_with_symbols_and_control(
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
        oriented_subcontinuation_plan.as_ref(),
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
            let units = super::units::declare_unit_bundle(&mut module, &static_transition_plan)?;
            // ⭐ `RT-FNSPLIT-B2F` `D4` — resolve every cross-owner call edge
            // against the bundle before a single body is defined.
            let calls = super::units::resolve_call_edges(&static_transition_plan, &units)?;
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
    let root_trap_exit = match body_emission_authority {
        BodyEmissionAuthority::RecursiveDescent => Some(TrapExitAuthority::Root {
            process_sentinel: root_trap_process_sentinel,
            source_authorized: true,
        }),
        BodyEmissionAuthority::FunctionizedUnits => None,
    };
    let root_function_local = helpers.declare_in_func(&mut module, &mut ctx.func, root_trap_exit);
    let mut func_ctx = FunctionBuilderContext::new();
    let mut compiler = Lowering {
        seed_env,
        active_emission_owner: None,
        active_static_recursor_result: None,
        active_static_recursor_selection: None,
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
        returned_source_continuation_result_origin: None,
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
            let staged_units = super::units::stage_unit_bodies(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                call_edges,
                staged_process_input,
            )?;
            compiler.require_complete_join_plan_consumption()?;
            compiler.require_complete_dynamic_splice_edge_consumption()?;
            let staged_adapter = super::units::stage_root_adapter(
                &mut module,
                &mut compiler,
                helpers,
                unit_bundle,
                func_id,
                process_mode,
                project_public_scalar_root,
            )?;
            compiler
                .static_transition_plan
                .validate_boundary_use_consumption()?;
            super::units::publish_functionized_bodies(
                &mut module,
                staged_units,
                func_id,
                staged_adapter,
            )?
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
                    compiler.function_local.host_dispatch_context = Some(host_dispatch_context);
                    initial_env.push(LoweringOperand::Specialized(Lowered::BorrowedNativeValue {
                        pointer: process_input,
                    }));
                    initial_env.push(LoweringOperand::Specialized(Lowered::CapabilityToken {
                        value: capability,
                    }));
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
                compiler.active_emission_owner = Some(
                    compiler
                        .static_transition_plan
                        .root_emittable_unit()?
                        .function(),
                );
                let root_origin = compiler.static_transition_plan.root_static_origin()?;
                let root = compiler.retained_body_occurrence(root_origin)?;
                compiler.select_terminal_result_origins(root_origin, root.expr)?;
                let lowered = compiler.lower_expr(&mut builder, root, &initial_env)?;
                // RecursiveDescent publishes the root's native result directly;
                // it emits no generated-unit result carrier. The root
                // CallableCapsuleEscape authority remains planner-visible for
                // direct producer tests, but this compilation causally
                // dispositions that absent ABI crossing.
                compiler.disposition_lowering_boundary_use_if_planned(
                    LoweringOnlyOperandEdge::CallableCapsuleEscape,
                    root_origin,
                    u32::MAX,
                )?;
                // RecursiveDescent still owns the explicit active-recursor
                // residual. It inlines across generated-unit owner boundaries,
                // so the function-owner equality used by FunctionizedUnits is
                // inapplicable here. Static Match reachability is nevertheless
                // closed at this generated root boundary: otherwise a recursive
                // source-machine revisit can emit one case and later classify
                // that same subtree as dead.
                compiler.validate_recursive_descent_join_disposition()?;
                compiler.require_complete_join_plan_consumption()?;
                compiler.require_complete_dynamic_splice_edge_consumption()?;
                match lowered {
                    LoweringOperand::Carried(word) if process_mode => {
                        let carrier_tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
                        let already_status = builder.ins().icmp_imm(
                            cranelift_codegen::ir::condcodes::IntCC::Equal,
                            carrier_tag,
                            BoundaryTag::ImmediateExitStatus as i64,
                        );
                        let retained = builder.create_block();
                        let decoded = builder.create_block();
                        let exit_word = builder.create_block();
                        builder.append_block_param(exit_word, types::I64);
                        builder
                            .ins()
                            .brif(already_status, retained, &[], decoded, &[]);
                        builder.switch_to_block(retained);
                        builder.ins().jump(exit_word, &[word.word.into()]);
                        builder.switch_to_block(decoded);
                        let decoded_word =
                            compiler.transfer_carried_exit_status(&mut builder, word)?;
                        builder.ins().jump(exit_word, &[decoded_word.word.into()]);
                        builder.switch_to_block(exit_word);
                        let word = CarriedBoundaryWord {
                            word: builder.block_params(exit_word)[0],
                        };
                        let tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
                        Lowering::require_i64(
                            &mut builder,
                            tag,
                            BoundaryTag::ImmediateExitStatus as i64,
                        );
                        let status = compiler.emit_carrier_scalar(&mut builder, word)?;
                        builder.ins().return_(&[status]);
                        decoder = Some(ResultDecoder::ProcessStatus);
                    }
                    LoweringOperand::Carried(word) => {
                        builder.ins().return_(&[word.word]);
                        decoder = Some(ResultDecoder::Boundary);
                    }
                    LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                        #[cfg(test)]
                        if process_mode {
                            px8tr_record_trap_provenance(
                                Px8trTrapProvenanceEvent::FinalProcessObjectTrap {
                                    trap: trap.clone(),
                                },
                            );
                        }
                        let status = builder
                            .ins()
                            .iconst(types::I64, if process_mode { -4 } else { 0 });
                        builder.ins().return_(&[status]);
                        maybe_trap = Some(trap);
                    }
                    LoweringOperand::Specialized(value) => {
                        let (token, result_decoder) = compiler.emit_result(&mut builder, value)?;
                        builder.ins().return_(&[token]);
                        decoder = Some(result_decoder);
                    }
                }
                builder.seal_all_blocks();
                builder.finalize();
            }
            compiler
                .static_transition_plan
                .validate_boundary_use_consumption()?;
            compiler.validate_recursive_descent_materialized_dead_join_cfg(&ctx.func)?;
            verify_cranelift_function(&ctx.func, module.isa())?;
            #[cfg(test)]
            scale_b_record_recursive_descent_root(&ctx.func);
            module
                .define_function(func_id, &mut ctx)
                .map_err(|err| backend_module(err.to_string()))?;
            super::units::RootUnitResult {
                decoder,
                trap: maybe_trap,
            }
        }
    };
    let trap_catalog = compiler.static_transition_plan.trap_catalog();
    let carrier_identity_catalog = compiler.static_transition_plan.carrier_identity_catalog()?;
    let compiled = CompiledModule::from_parts(
        module,
        func_id,
        root_result.decoder,
        compiler.result_table,
        root_result.trap,
        trap_catalog,
        carrier_identity_catalog,
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
    fn reject_carried_residual_arguments(arguments: usize) -> Result<(), CraneliftBackendError> {
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

    /// Read the closure-shaped worker retained by RecursiveDescent.
    ///
    /// The exact static-worker identity was consumed when the induction
    /// hypothesis was minted. RecursiveDescent emits no generated-unit
    /// crossing, so it retains that closure rather than consuming the generic
    /// `SpecializedOnlyLeaf` authority that the worker replaces.
    fn specialized_recursor_residual_ref<'b>(
        &self,
        residual: &'b LoweringOperand,
        recursor_parent: StaticOriginId,
        sibling_position: usize,
        recursive_worker: Option<StaticRecursorWorker>,
    ) -> Result<&'b Lowered, CraneliftBackendError> {
        if let Some(worker) = recursive_worker {
            if !matches!(
                self.body_emission_authority,
                BodyEmissionAuthority::RecursiveDescent
            ) || worker.parent_origin != recursor_parent
                || worker.sibling_position != sibling_position
            {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "inline static recursor worker disagrees with its exact authority".to_string(),
                )));
            }
            return match residual {
                LoweringOperand::Specialized(lowered) => Ok(lowered),
                LoweringOperand::Carried(_) => Err(backend(BackendFailure::PlannerInvariant(
                    "inline static recursor worker lost its closure residual".to_string(),
                ))),
            };
        }
        let edge = self
            .static_transition_plan
            .recursor_boundary_use_token(recursor_parent, sibling_position)?;
        residual.specialized_ref_at(edge)
    }

    fn specialized_recursor_residual(
        &self,
        residual: LoweringOperand,
        recursor_parent: StaticOriginId,
        sibling_position: usize,
        recursive_worker: Option<StaticRecursorWorker>,
    ) -> Result<Lowered, CraneliftBackendError> {
        if let Some(worker) = recursive_worker {
            if !matches!(
                self.body_emission_authority,
                BodyEmissionAuthority::RecursiveDescent
            ) || worker.parent_origin != recursor_parent
                || worker.sibling_position != sibling_position
            {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "inline static recursor worker disagrees with its exact authority".to_string(),
                )));
            }
            return match residual {
                LoweringOperand::Specialized(lowered) => Ok(lowered),
                LoweringOperand::Carried(_) => Err(backend(BackendFailure::PlannerInvariant(
                    "inline static recursor worker lost its closure residual".to_string(),
                ))),
            };
        }
        let edge = self
            .static_transition_plan
            .recursor_boundary_use_token(recursor_parent, sibling_position)?;
        residual.specialized_at(edge)
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
        recursor_parent: StaticOriginId,
        sibling_position: usize,
        recursive_worker: Option<StaticRecursorWorker>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // ⭐⭐ `AC-C4` — the carried residual, taken BEFORE the specialized
        // shapes so a carried word never reaches a template probe.
        if let LoweringOperand::Carried(word) = residual {
            if let Some(worker) = recursive_worker.filter(|_| {
                matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                )
            }) {
                let continuation_specialized =
                    self.producer_call_has_continuation_specialization(
                        worker.body_origin,
                        call_origin,
                    )?;
                let mut inputs = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(call_origin, 1 + position, arg)?;
                        self.lower_expr(builder, arg, argument_env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                self.append_static_recursor_worker_captures(builder, *word, worker, &mut inputs)?;
                let returned = self.call_declared_recursive_position_unit(
                    builder,
                    worker.body_origin,
                    &inputs,
                )?;
                if continuation_specialized {
                    self.consume_out_of_line_continuation_splice()?;
                    return self.apply_recursive_continuation_specialization_if_planned(
                        builder,
                        worker,
                        call_origin,
                        returned,
                    );
                }
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
        let residual = self.specialized_recursor_residual_ref(
            residual,
            recursor_parent,
            sibling_position,
            recursive_worker,
        )?;
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
        call_env.extend(captures.iter().cloned());
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
        self.function_local
            .continuation_specialization_environments
            .insert(static_origin, eliminator_env.to_vec());
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
            if matches!(
                value,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
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
                continuation.recursor_parent,
                continuation.sibling_position,
                continuation.recursive_worker,
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
                self.close_reached_operand_edge(static_origin, 0, SourceOperandRole::WrapperBody)?;
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
                self.close_reached_operand_edge(static_origin, 0, SourceOperandRole::WrapperBody)?;
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
                self.close_reached_operand_edge(static_origin, 0, SourceOperandRole::WrapperBody)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.lower_computational_producer_expr(builder, body, producer_env, eliminators)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.close_reached_operand_edge(static_origin, 0, SourceOperandRole::WrapperBody)?;
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let value = self.lower_computational_producer_expr(
                    builder,
                    body,
                    producer_env,
                    eliminators,
                )?;
                self.finish_checked_computational_ih_marker(static_origin, value)
            }
            RuntimeExpr::Let { value, body } => {
                // The `Let`'s own children: value `0`, body `1`. When the body
                // is itself the `Call` below, that `Call` occurrence's origin is
                // this body child — which is what the pending-let frame carries
                // so its arguments stay positionally derivable.
                let body_origin = self
                    .static_transition_plan
                    .child_static_origin(static_origin, 1)?;
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
                                    let recursive_worker = invocation.recursive_worker;
                                    let recursor_parent = invocation.selection.static_origin;
                                    let sibling_position = invocation.sibling_position;
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
                                    let prepared_residual = self
                                        .prepare_static_recursor_residual(residual, &invocation)?;
                                    let dynamic_splice_edges =
                                        self.take_dynamic_splice_edges(&invocation)?;
                                    let installed = compose_oriented_subcontinuation(
                                        self.oriented_subcontinuation_plan.as_ref(),
                                        self.active_recursive_invocations.last().copied(),
                                        activation,
                                        invocation,
                                        dynamic_splice_edges,
                                    )?;
                                    let residual = self.materialize_static_recursor_residual(
                                        builder,
                                        prepared_residual,
                                    )?;
                                    let frames = installed_oriented_eliminator_frames(&installed);
                                    let mut composed = Vec::with_capacity(frames.len() + 2);
                                    composed.push(EliminatorFrame::PendingLet(
                                        PendingLetContinuationFrame {
                                            residual: &residual,
                                            args,
                                            call_origin: body_origin,
                                            env: producer_env,
                                            recursor_parent,
                                            sibling_position,
                                            recursive_worker,
                                        },
                                    ));
                                    composed.extend(frames);
                                    composed.push(EliminatorFrame::InvocationReturn);
                                    self.enter_oriented_semantic_region(installed.checked);
                                    let value = self.child_occurrence(static_origin, 0, value)?;
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
                if matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                ) {
                    if let Some(call) = self
                        .function_local
                        .static_callable_calls
                        .get(&static_origin)
                        .cloned()
                    {
                        let edge = self.operand_edge_token(
                            static_origin,
                            0,
                            SourceOperandRole::CallCallee,
                        )?;
                        if edge.disposition() != OperandEdgeDisposition::SpecializedOnlyLeaf {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "static callable callee lost its semantic-inspection disposition"
                                    .to_string(),
                            )));
                        }
                        let value = self.lower_static_callable_specialization_call(
                            builder,
                            static_origin,
                            callee,
                            args,
                            producer_env,
                            call,
                        )?;
                        return self.lower_computational_match_value_composed(
                            builder,
                            value,
                            eliminators,
                        );
                    }
                }
                let callee = self.lower_expr(builder, callee, producer_env)?;
                let callee_edge = self.reached_operand_edge_token(
                    static_origin,
                    0,
                    SourceOperandRole::CallCallee,
                )?;
                if callee_edge.disposition() != OperandEdgeDisposition::SpecializedOnlyLeaf {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "producer callee lost its semantic-inspection disposition".to_string(),
                    )));
                }
                match callee {
                    LoweringOperand::Specialized(Lowered::DeclarationClosure {
                        reference_origin,
                        symbol,
                        captures,
                        params,
                        body,
                    }) => self.lower_recursive_declaration_call(
                        builder,
                        reference_origin,
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
                            if args.len() == 1 && requires_heterogeneous_deforestation(&args[0]) {
                                if let Some((cases, default)) =
                                    ordinary_match_continuation(&params, retained.expr)
                                {
                                    let argument =
                                        self.child_occurrence(static_origin, 1, &args[0])?;
                                    let frame_env =
                                        env_with_operands(captures.clone(), producer_env);
                                    let mut composed = Vec::with_capacity(eliminators.len() + 1);
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
                                    BodyEmissionAuthority::FunctionizedUnits => Ok(match lowered {
                                        LoweringOperand::Carried(word) => {
                                            LoweringOperand::Carried(word)
                                        }
                                        LoweringOperand::Specialized(value) => {
                                            let edge = self.operand_edge_token(
                                                static_origin,
                                                1 + position,
                                                SourceOperandRole::CallArgument,
                                            )?;
                                            LoweringOperand::Carried(
                                                self.transfer_into_carrier_on_edge(
                                                    builder,
                                                    arg.static_origin,
                                                    &value,
                                                    edge,
                                                )?,
                                            )
                                        }
                                    }),
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures);
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
                                let returned = self.call_declared_unit(
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
                        let (base, boundary) =
                            decompose_computational_recursor(LoweringOperand::Specialized(callee));
                        let (activation, invocation) =
                            boundary.expect("recursor closure carries an invocation segment");
                        let recursive_worker = invocation.recursive_worker;
                        let recursor_parent = invocation.selection.static_origin;
                        let sibling_position = invocation.sibling_position;
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
                        let prepared = self.prepare_static_recursor_residual(base, &invocation)?;
                        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                        let installed = compose_oriented_subcontinuation(
                            self.oriented_subcontinuation_plan.as_ref(),
                            checked_ih_invocation
                                .or_else(|| self.active_recursive_invocations.last().copied()),
                            activation,
                            invocation,
                            dynamic_splice_edges,
                        )?;
                        let base = self.materialize_static_recursor_residual(builder, prepared)?;
                        let mut composed = installed_oriented_eliminator_frames(&installed);
                        composed.push(EliminatorFrame::InvocationReturn);
                        // ⭐⭐ `AC-C4` — the carried residual resumes the SAME
                        // computational eliminator over the carried word, under
                        // the same semantic-region bracket the specialized
                        // `BoundedNat` arm below uses. ⛔ Not `specialized_at`,
                        // ⛔ not a reconstructed `Lowered`, ⛔ not the producer.
                        if let LoweringOperand::Carried(word) = base {
                            if let Some(worker) = recursive_worker.filter(|_| {
                                matches!(
                                    self.body_emission_authority,
                                    BodyEmissionAuthority::FunctionizedUnits
                                )
                            }) {
                                let continuation_specialized = self
                                    .producer_call_has_continuation_specialization(
                                        worker.body_origin,
                                        static_origin,
                                    )?;
                                let mut inputs = args
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
                                self.append_static_recursor_worker_captures(
                                    builder,
                                    word,
                                    worker,
                                    &mut inputs,
                                )?;
                                self.enter_oriented_semantic_region(installed.checked);
                                let returned = self
                                    .call_declared_recursive_position_unit(
                                        builder,
                                        worker.body_origin,
                                        &inputs,
                                    )
                                    .and_then(|value| {
                                        if continuation_specialized {
                                            self.apply_recursive_continuation_specialization_if_planned(
                                                builder, worker, static_origin, value,
                                            )
                                        } else {
                                            self.lower_computational_match_value_composed(
                                                builder, value, &composed,
                                            )
                                        }
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
                        let base = self.specialized_recursor_residual(
                            base,
                            recursor_parent,
                            sibling_position,
                            recursive_worker,
                        )?;
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
                                    BodyEmissionAuthority::FunctionizedUnits => Ok(match lowered {
                                        LoweringOperand::Carried(word) => {
                                            LoweringOperand::Carried(word)
                                        }
                                        LoweringOperand::Specialized(value) => {
                                            let edge = self.operand_edge_token(
                                                static_origin,
                                                1 + position,
                                                SourceOperandRole::CallArgument,
                                            )?;
                                            LoweringOperand::Carried(
                                                self.transfer_into_carrier_on_edge(
                                                    builder,
                                                    arg.static_origin,
                                                    &value,
                                                    edge,
                                                )?,
                                            )
                                        }
                                    }),
                                }
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        call_env.extend(captures);
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
                                    builder, body, &call_env, &composed,
                                )
                            }
                            BodyEmissionAuthority::FunctionizedUnits => {
                                let returned = self.call_declared_unit(
                                    builder,
                                    body,
                                    &call_env,
                                    #[cfg(test)]
                                    None,
                                )?;
                                self.lower_computational_match_value_composed(
                                    builder, returned, &composed,
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
                if let Some(call_token) =
                    self.continuation_specialization_result_origin(static_origin)
                {
                    if self.continuation_specialization_flattens_result(call_token)? {
                    // The producer branch still owns the exact worker at this
                    // source Construct. Transfer its ordinary fields directly
                    // to the planner-issued return-hole unit before the
                    // composed eliminator can erase that identity. The
                    // specialization embodies this eliminator and its checked
                    // suffix, so applying `eliminators` again would recreate
                    // the prohibited post-join lookup.
                    let lowered_args = args
                        .iter()
                        .enumerate()
                        .map(|(position, argument)| {
                            let argument =
                                self.child_occurrence(static_origin, position, argument)?;
                            self.lower_expr(builder, argument, producer_env)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    return self.call_known_constructor_continuation_specialization(
                        builder,
                        call_token,
                        static_origin,
                        lowered_args,
                    );
                    }
                }
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
                        aggregate_origin: Some(static_origin),
                        synthesized_identity: Some(ConstructorIdentityV1::Source(
                            self.static_transition_plan
                                .constructor_symbol_identity(static_origin)?,
                        )),
                        args: self.specialized_source_env_at(
                            &lowered_args,
                            static_origin,
                            0,
                            SourceOperandRole::ConstructArgument,
                        )?,
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
                            None => {
                                self.disposition_statically_unselected_match_cases(
                                    eliminator.static_origin,
                                    None,
                                )?;
                                return Ok(LoweringOperand::Specialized(Lowered::Trap(
                                    eliminator.default.clone(),
                                )));
                            }
                        };
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            Some(case_index),
                        )?;
                        let edge = self.operand_edge_token(
                            eliminator.static_origin,
                            1 + case_index,
                            SourceOperandRole::MatchArm,
                        )?;
                        if edge.disposition() != OperandEdgeDisposition::Forwarding {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "selected computational match arm lost its forwarding disposition"
                                    .to_string(),
                            )));
                        }
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
                            Err(trap) => {
                                self.disposition_statically_unselected_match_cases(
                                    eliminator.static_origin,
                                    None,
                                )?;
                                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                            }
                        };
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            Some(case_index),
                        )?;
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
                let bridge =
                    immediate_binder_eliminator(case_body.expr, argument_binder_offset, args.len());
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
                    if let Some(LoweringOperand::Specialized(Lowered::Trap(trap))) =
                        lowered_prefix.iter().find(|value| {
                            matches!(value, LoweringOperand::Specialized(Lowered::Trap(_)))
                        })
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
                    let lowered_prefix = self.specialized_source_env_at(
                        &lowered_prefix,
                        static_origin,
                        0,
                        SourceOperandRole::ConstructArgument,
                    )?;
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
                if let Some(call_token) =
                    self.continuation_specialization_result_origin(static_origin)
                {
                    if self.continuation_specialization_flattens_result(call_token)? {
                        return self.call_known_constructor_continuation_specialization(
                            builder,
                            call_token,
                            static_origin,
                            lowered_args,
                        );
                    }
                }
                if lowered_args
                    .iter()
                    .any(|argument| matches!(argument, LoweringOperand::Carried(_)))
                {
                    return self.lower_known_constructor_operands_composed(
                        builder,
                        static_origin,
                        constructor,
                        lowered_args,
                        eliminators,
                    );
                }
                let produced = LoweringOperand::Specialized(Lowered::Constructor {
                    constructor: constructor.clone(),
                    aggregate_origin: Some(static_origin),
                    // Carry the plan's already-resolved source identity with
                    // the template.  A later unit boundary may receive this
                    // result after nested producer traversal, where the caller
                    // occurrence is not the constructor occurrence and
                    // therefore cannot lawfully re-query its atom.
                    synthesized_identity: Some(ConstructorIdentityV1::Source(
                        self.static_transition_plan
                            .constructor_symbol_identity(static_origin)?,
                    )),
                    args: self.specialized_source_env_at(
                        &lowered_args,
                        static_origin,
                        0,
                        SourceOperandRole::ConstructArgument,
                    )?,
                });
                self.lower_computational_match_value_composed(builder, produced, eliminators)
            }
            RuntimeExpr::Match {
                scrutinee,
                cases: producer_cases,
                default: producer_default,
            } => {
                let scrutinee = self.child_occurrence(static_origin, 0, scrutinee)?;
                let _scrutinee_edge =
                    self.operand_edge_token(static_origin, 0, SourceOperandRole::MatchScrutinee)?;
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                if let LoweringOperand::Carried(word) = selected {
                    return self.lower_carried_match(
                        builder,
                        word,
                        CarriedMatchContinuation::Producer {
                            cases: producer_cases,
                            default: producer_default,
                            env: producer_env,
                            static_origin,
                            eliminators,
                        },
                    );
                }
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
                        self.disposition_statically_unselected_match_cases(
                            static_origin,
                            Some(index),
                        )?;
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        return self.lower_computational_producer_case_body(
                            builder,
                            body,
                            producer_env,
                            eliminators,
                        );
                    }
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = join_plan
                        .has_continuing_predecessor
                        .then(|| builder.create_block());
                    if let Some(merge) = merge {
                        self.append_planned_join_params(builder, merge, &join_plan);
                    }
                    builder.ins().brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    for (block, (index, producer_case)) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let body =
                            self.case_body_occurrence(static_origin, index, &producer_case.body)?;
                        let lowered = self.lower_computational_producer_case_body(
                            builder,
                            body,
                            producer_env,
                            eliminators,
                        )?;
                        if self.seal_source_trap_branch(builder, &lowered)? {
                            continue;
                        }
                        let merge = merge.ok_or_else(|| {
                            backend_module(
                                "join plan omitted a producer Bool Match merge despite a \
                                 continuing predecessor"
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
                            "ComputationalMatch",
                        )?;
                    }
                    let Some(merge) = merge else {
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(
                            producer_default.clone(),
                        )));
                    };
                    return self.finish_planned_join(
                        builder,
                        merge,
                        &join_plan,
                        merge_kind,
                        "ComputationalMatch",
                    );
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
                    let merge = join_plan
                        .has_continuing_predecessor
                        .then(|| builder.create_block());
                    if let Some(merge) = merge {
                        self.append_planned_join_params(builder, merge, &join_plan);
                    }
                    builder.ins().brif(success, ok_block, &[], err_block, &[]);
                    let mut merge_kind = None;
                    for (block, constructor, payload) in [
                        (ok_block, ok_constructor.as_str(), *ok),
                        (err_block, err_constructor.as_str(), *error),
                    ] {
                        builder.switch_to_block(block);
                        let (lowered, predecessor_origin) = if let Some((index, producer_case)) =
                            dynamic_host_result_producer_case(producer_cases, constructor)?
                        {
                            let arm_edge = self.operand_edge_token(
                                static_origin,
                                1 + index,
                                SourceOperandRole::MatchArm,
                            )?;
                            if arm_edge.disposition() != OperandEdgeDisposition::Forwarding {
                                return Err(backend(BackendFailure::PlannerInvariant(
                                    "dynamic producer HostResult arm lost its forwarding \
                                     disposition"
                                        .to_string(),
                                )));
                            }
                            let body = self.case_body_occurrence(
                                static_origin,
                                index,
                                &producer_case.body,
                            )?;
                            if !self
                                .static_transition_plan
                                .source_environment_slot_is_used(body.static_origin, 0)?
                            {
                                self.disposition_unobserved_lowered_value(&payload)?;
                            }
                            let case_env = env_with([payload], producer_env);
                            (
                                self.lower_computational_producer_case_body(
                                    builder,
                                    body,
                                    &case_env,
                                    eliminators,
                                )?,
                                body.static_origin,
                            )
                        } else {
                            (
                                LoweringOperand::Specialized(Lowered::Trap(
                                    producer_default.clone(),
                                )),
                                static_origin,
                            )
                        };
                        if self.seal_source_trap_branch(builder, &lowered)? {
                            continue;
                        }
                        let merge = merge.ok_or_else(|| {
                            backend_module(
                                "join plan omitted a producer HostResult merge despite a \
                                 continuing predecessor"
                                    .to_string(),
                            )
                        })?;
                        self.jump_planned_join_arm(
                            builder,
                            merge,
                            &join_plan,
                            predecessor_origin,
                            lowered,
                            &mut merge_kind,
                            "ComputationalMatch",
                        )?;
                    }
                    let Some(merge) = merge else {
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(
                            producer_default.clone(),
                        )));
                    };
                    return self.finish_planned_join(
                        builder,
                        merge,
                        &join_plan,
                        merge_kind,
                        "ComputationalMatch",
                    );
                }
                if let LoweringOperand::Specialized(Lowered::DynamicConstructor(dynamic)) = selected
                {
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
                    constructor, args, ..
                }) = selected
                else {
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
                    self.disposition_statically_unselected_match_cases(static_origin, None)?;
                    for eliminator in eliminators {
                        let frame_origin = match eliminator {
                            EliminatorFrame::Computational(frame) => frame.static_origin,
                            EliminatorFrame::Ordinary(frame) => frame.static_origin,
                            EliminatorFrame::PendingLet(_)
                            | EliminatorFrame::InvocationReturn
                            | EliminatorFrame::Active(_) => continue,
                        };
                        self.disposition_operand_edge(
                            frame_origin,
                            0,
                            SourceOperandRole::MatchScrutinee,
                        )?;
                        self.disposition_statically_unselected_match_cases(frame_origin, None)?;
                    }
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(
                        producer_default.clone(),
                    )));
                };
                self.disposition_statically_unselected_match_cases(
                    static_origin,
                    Some(case_index),
                )?;
                if producer_case.binders != args.len() {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing match constructor arity changed",
                    ));
                }
                let case_env = env_with(args, producer_env);
                let body =
                    self.case_body_occurrence(static_origin, case_index, &producer_case.body)?;
                self.lower_computational_producer_case_body(builder, body, &case_env, eliminators)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee: inner_scrutinee,
                cases: inner_cases,
                default: inner_default,
            } => {
                self.function_local
                    .continuation_specialization_environments
                    .insert(static_origin, producer_env.to_vec());
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
                let scrutinee_edge = self.reached_operand_edge_token(
                    static_origin,
                    0,
                    SourceOperandRole::IfScrutinee,
                )?;
                if scrutinee_edge.disposition() != OperandEdgeDisposition::SemanticEliminator {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "producer If scrutinee lost its semantic-eliminator disposition"
                            .to_string(),
                    )));
                }
                let selected = self.lower_expr(builder, scrutinee, producer_env)?;
                let LoweringOperand::Specialized(Lowered::Bool { value, known }) = selected else {
                    return Err(unsupported(
                        "ComputationalMatch",
                        "tree-producing If scrutinee is not Bool",
                    ));
                };
                if let Some(known) = known {
                    let unselected = if known { else_expr } else { then_expr };
                    let selected_position = if known { 1 } else { 2 };
                    let unselected_position = if known { 2 } else { 1 };
                    let selected_edge = self.reached_operand_edge_token(
                        static_origin,
                        selected_position,
                        SourceOperandRole::IfArm,
                    )?;
                    if selected_edge.disposition() != OperandEdgeDisposition::Forwarding {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "selected producer If arm lost its forwarding disposition".to_string(),
                        )));
                    }
                    self.disposition_operand_edge(
                        static_origin,
                        unselected_position,
                        SourceOperandRole::IfArm,
                    )?;
                    self.disposition_lowering_boundary_use_if_planned(
                        LoweringOnlyOperandEdge::JoinArm,
                        static_origin,
                        0,
                    )?;
                    self.disposition_lowering_boundary_use_if_planned(
                        LoweringOnlyOperandEdge::JoinArm,
                        if known {
                            then_expr.static_origin
                        } else {
                            else_expr.static_origin
                        },
                        0,
                    )?;
                    self.disposition_statically_unselected_source_subtree(
                        unselected.static_origin,
                    )?;
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
                let merge = join_plan
                    .has_continuing_predecessor
                    .then(|| builder.create_block());
                if let Some(merge) = merge {
                    self.append_planned_join_params(builder, merge, &join_plan);
                }
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                let mut merge_kind = None;
                let mut terminal_trap = None;
                for (block, branch) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let lowered = self.lower_computational_producer_expr(
                        builder,
                        branch,
                        producer_env,
                        eliminators,
                    )?;
                    if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                        terminal_trap.get_or_insert_with(|| trap.clone());
                    }
                    if self.seal_source_trap_branch(builder, &lowered)? {
                        continue;
                    }
                    let merge = merge.ok_or_else(|| {
                        backend_module(
                            "join plan omitted a producer If merge despite a continuing \
                             predecessor"
                                .to_string(),
                        )
                    })?;
                    self.jump_planned_join_arm(
                        builder,
                        merge,
                        &join_plan,
                        branch.static_origin,
                        lowered,
                        &mut merge_kind,
                        "ComputationalMatch",
                    )?;
                }
                let Some(merge) = merge else {
                    let unreachable = builder.create_block();
                    builder.switch_to_block(unreachable);
                    let trap = terminal_trap.ok_or_else(|| {
                        backend_module(
                            "producer If join omitted both a continuing predecessor and a \
                             source trap"
                                .to_string(),
                        )
                    })?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                };
                self.finish_planned_join(
                    builder,
                    merge,
                    &join_plan,
                    merge_kind,
                    "ComputationalMatch",
                )
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

    fn lower_computational_producer_case_body(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        body: SourceOccurrence<'_>,
        case_env: &[LoweringOperand],
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let continuation_source_machine = if let Some(owner) = self.active_emission_owner {
            self.static_transition_plan
                .continuation_specialization_in_owner_subtree(owner, body.static_origin)?
        } else {
            false
        };
        if !continuation_source_machine {
            return self.lower_computational_producer_expr(builder, body, case_env, eliminators);
        }

        let activation = self.mint_continuation_activation();
        let cursor = self.mint_continuation_cursor();
        let splice_caller = active_recursor_frame(eliminators);
        let mut pending: Vec<_> = eliminators
            .iter()
            .copied()
            .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
            .collect();
        if let Some(active) = splice_caller {
            pending.extend_from_slice(active.pending);
        }
        let selected_ancestry = splice_caller
            .map(|active| active.selected_ancestry.to_vec())
            .unwrap_or_default();
        let active_state = ActiveContinuationFrame {
            activation,
            cursor,
            parent: splice_caller.and_then(|active| active.parent),
            pending: &pending,
            selected_ancestry: &selected_ancestry,
            source_lineage: splice_caller
                .map(|active| active.source_lineage)
                .unwrap_or(&[]),
            source_selected_cursor: splice_caller.and_then(|active| active.source_selected_cursor),
            selected_scope: splice_caller.and_then(|active| active.selected_scope),
        };
        self.lower_source_machine_with_result_origin(builder, body, case_env, &active_state)
            .map(|(value, _)| value)
    }

    fn lower_known_constructor_operands_composed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        source_producer_origin: StaticOriginId,
        constructor: &str,
        args: Vec<LoweringOperand>,
        eliminators: &[EliminatorFrame<'_>],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let eliminator = eliminators[0];
        let remaining = &eliminators[1..];
        match eliminator {
            EliminatorFrame::Computational(frame) => {
                if frame.retained_scrutinee_index.is_some()
                    || frame.deferred_constructor_case.is_some()
                {
                    return Err(unsupported(
                        "BoundaryCarrier",
                        "a mixed-phase known constructor reached an eliminator that retains or \
                         rebuilds its whole scrutinee",
                    ));
                }
                let (case_index, case) = frame
                    .cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "ComputationalMatch",
                            "the selected known constructor has no computational case",
                        )
                    })?;
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
                self.record_source_machine_computational_match_selection(
                    frame.static_origin,
                    Some(case_index),
                )?;
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

                let splice_caller = active_recursor_frame(remaining);
                let mut selected_ancestry = splice_caller
                    .map(|active| active.selected_ancestry.to_vec())
                    .unwrap_or_default();
                selected_ancestry.push(frame.provenance);
                let mut pending: Vec<_> = remaining
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
                        cases: frame.cases.to_vec(),
                        default: frame.default.clone(),
                        outer_env: frame.env.to_vec(),
                        static_origin: frame.static_origin,
                        provenance: frame.provenance,
                        checked_frame_id: frame.checked_frame_id,
                        checked_invocation_id: frame.checked_invocation_id,
                        checked_invocation_source: frame.checked_invocation_source,
                        checked_invocation_depth: frame.checked_invocation_depth,
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

                let ih_slots =
                    self.computational_ih_slots_for_case(case, frame.checked_frame_id)?;
                let mut induction_hypotheses = Vec::with_capacity(case.recursive_positions.len());
                for position in case.recursive_positions.iter().rev().copied() {
                    let slot_template_id = case
                        .recursive_positions
                        .iter()
                        .position(|candidate| *candidate == position)
                        .and_then(|index| ih_slots[index]);
                    let recursive_worker = self.selected_static_recursor_worker_for_producer(
                        frame.static_origin,
                        position,
                        source_producer_origin,
                    )?;
                    let induction_hypothesis = self.make_computational_recursor(
                        args[position].clone(),
                        frame.cases.to_vec(),
                        frame.default.clone(),
                        frame.env.to_vec(),
                        frame.static_origin,
                        frame.provenance,
                        frame.checked_frame_id,
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
                        recursive_worker,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
                    induction_hypotheses.push(induction_hypothesis);
                }
                let mut case_env = induction_hypotheses;
                case_env.extend(args);
                case_env.extend(frame.env.iter().cloned());
                let body =
                    self.case_body_occurrence(frame.static_origin, case_index, &case.body)?;
                if !case.recursive_positions.is_empty() {
                    return self.lower_source_machine(builder, body, &case_env, &active_state);
                }
                if remaining.is_empty() {
                    self.lower_expr(builder, body, &case_env)
                } else {
                    self.lower_computational_producer_expr(builder, body, &case_env, remaining)
                }
            }
            EliminatorFrame::Ordinary(frame) => {
                if frame.retained_scrutinee_index.is_some()
                    || frame.deferred_constructor_case.is_some()
                {
                    return Err(unsupported(
                        "BoundaryCarrier",
                        "a mixed-phase known constructor reached an ordinary eliminator that \
                         retains or rebuilds its whole scrutinee",
                    ));
                }
                let (case_index, case) = frame
                    .cases
                    .iter()
                    .enumerate()
                    .find(|(_, case)| case.constructor == constructor)
                    .ok_or_else(|| {
                        unsupported(
                            "Match",
                            "the selected known constructor has no ordinary case",
                        )
                    })?;
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
                case_env.extend(frame.env.iter().cloned());
                let body =
                    self.case_body_occurrence(frame.static_origin, case_index, &case.body)?;
                if remaining.is_empty() {
                    self.lower_expr(builder, body, &case_env)
                } else {
                    self.lower_computational_producer_expr(builder, body, &case_env, remaining)
                }
            }
            EliminatorFrame::PendingLet(_)
            | EliminatorFrame::InvocationReturn
            | EliminatorFrame::Active(_) => Err(unsupported(
                "BoundaryCarrier",
                "a mixed-phase known constructor reached a non-eliminating continuation",
            )),
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
                EliminatorFrame::Computational(frame) => {
                    // Carrier dispatch bypasses `specialized_at` below, but it
                    // still emits and consumes this exact source scrutinee
                    // occurrence.
                    let _edge = self.operand_edge_token(
                        frame.static_origin,
                        0,
                        SourceOperandRole::MatchScrutinee,
                    )?;
                    let remaining = &eliminators[1..];
                    let closes_branch_local_specialization =
                        if let ([EliminatorFrame::Active(active)], Some(owner)) =
                            (remaining, self.active_emission_owner)
                        {
                            if active.pending.is_empty() {
                                false
                            } else {
                                let mut closes = false;
                                for body_origin in self
                                    .static_transition_plan
                                    .source_match_case_body_origins(frame.static_origin)?
                                {
                                    closes |= self
                                        .static_transition_plan
                                        .continuation_specialization_in_owner_subtree(
                                            owner,
                                            body_origin,
                                        )?;
                                }
                                closes
                            }
                        } else {
                            false
                        };
                    if closes_branch_local_specialization {
                        // A branch-local continuation-specialization call
                        // produces an ordinary result. Close this carried case
                        // join before lowering the next caller-owned suffix so
                        // that the suffix has one emitted occurrence rather
                        // than being cloned into every predecessor.
                        let [EliminatorFrame::Active(active)] = remaining else {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "continuation-specialized case split has no exact active suffix"
                                    .to_string(),
                            )));
                        };
                        let Some((branch_suffix, shared_suffix)) = active.pending.split_first()
                        else {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "continuation-specialized case split has an empty active suffix"
                                    .to_string(),
                            )));
                        };
                        let branch_active = ActiveContinuationFrame {
                            activation: active.activation,
                            cursor: active.cursor,
                            parent: active.parent,
                            pending: std::slice::from_ref(branch_suffix),
                            selected_ancestry: active.selected_ancestry,
                            source_lineage: active.source_lineage,
                            source_selected_cursor: active.source_selected_cursor,
                            selected_scope: active.selected_scope,
                        };
                        let value = self.lower_carried_computational_match(
                            builder,
                            word,
                            frame,
                            &[EliminatorFrame::Active(branch_active)],
                        )?;
                        let shared_active = ActiveContinuationFrame {
                            activation: active.activation,
                            cursor: active.cursor,
                            parent: active.parent,
                            pending: shared_suffix,
                            selected_ancestry: active.selected_ancestry,
                            source_lineage: active.source_lineage,
                            source_selected_cursor: active.source_selected_cursor,
                            selected_scope: active.selected_scope,
                        };
                        self.resume_active_continuation(builder, value, shared_active)
                    } else {
                        self.lower_carried_computational_match(builder, word, frame, remaining)
                    }
                }
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
        let scrutinee = match eliminator {
            EliminatorFrame::Computational(frame) => {
                let edge = self.operand_edge_token(
                    frame.static_origin,
                    0,
                    SourceOperandRole::MatchScrutinee,
                )?;
                scrutinee.specialized_at(edge)?
            }
            EliminatorFrame::Ordinary(frame) => {
                let edge = self.operand_edge_token(
                    frame.static_origin,
                    0,
                    SourceOperandRole::MatchScrutinee,
                )?;
                scrutinee.specialized_at(edge)?
            }
            EliminatorFrame::PendingLet(frame) => self.specialized_recursor_residual(
                scrutinee,
                frame.recursor_parent,
                frame.sibling_position,
                frame.recursive_worker,
            )?,
            EliminatorFrame::Active(_) => {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "active composed match has no exact planned scrutinee boundary".to_string(),
                )));
            }
            EliminatorFrame::InvocationReturn => {
                unreachable!("invocation return forwards before composed scrutinee classification")
            }
        };
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
            aggregate_origin,
            synthesized_identity,
            args,
        } = scrutinee
        else {
            return Err(unsupported(
                "ComputationalMatch",
                "scrutinee is not a constructor value after ordinary expression lowering",
            ));
        };
        let retained_scrutinee = Lowered::Constructor {
            constructor: constructor.clone(),
            aggregate_origin,
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
                    Err(trap) => {
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            None,
                        )?;
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                    }
                };
                self.disposition_statically_unselected_match_cases(
                    eliminator.static_origin,
                    Some(case_index),
                )?;
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
                    let recursive_worker = match aggregate_origin {
                        Some(producer_origin) => self
                            .selected_static_recursor_worker_for_producer(
                                eliminator.static_origin,
                                position,
                                producer_origin,
                            )?,
                        None => self
                            .selected_static_recursor_worker(eliminator.static_origin, position)?,
                    };
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
                        recursive_worker,
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
                    return self.lower_source_machine(builder, case_body, &case_env, &active_state);
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
                    Err(trap) => {
                        self.disposition_statically_unselected_match_cases(
                            eliminator.static_origin,
                            None,
                        )?;
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                    }
                };
                self.disposition_statically_unselected_match_cases(
                    eliminator.static_origin,
                    Some(case_index),
                )?;
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
                    self.case_body_occurrence(eliminator.static_origin, case_index, &case.body)?,
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
                return self.resume_active_continuation(
                    builder,
                    LoweringOperand::Specialized(retained_scrutinee),
                    active,
                );
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
                LoweringOperand::Specialized(Lowered::StructuralNat(StructuralNatV1 {
                    value: nat.value,
                }))
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
            return self.resume_active_continuation(
                builder,
                LoweringOperand::Specialized(value),
                active,
            );
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
                env.insert(
                    index,
                    LoweringOperand::Specialized(retained_scrutinee.clone()),
                );
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
            let edge = self.operand_edge_token(
                deferred.construct_origin,
                deferred.selected_field + 1 + offset,
                SourceOperandRole::ConstructArgument,
            )?;
            let lowered = if edge.disposition() == OperandEdgeDisposition::CallableCapture {
                lowered.callable_capture_ref_at(edge)?.clone()
            } else {
                lowered.specialized_at(edge)?
            };
            constructor_args.push(lowered);
        }
        let outer_scrutinee = Lowered::Constructor {
            constructor: deferred.constructor.to_string(),
            aggregate_origin: Some(deferred.construct_origin),
            synthesized_identity: Some(ConstructorIdentityV1::Source(
                self.static_transition_plan
                    .constructor_symbol_identity(deferred.construct_origin)?,
            )),
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
                    let recursive_worker = self.selected_static_recursor_worker_for_producer(
                        frame.static_origin,
                        position,
                        deferred.construct_origin,
                    )?;
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
                        recursive_worker,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(
                        Px8jProducerPath::DeferredConstructor,
                        &induction_hypothesis,
                    );
                    induction_hypotheses.push(induction_hypothesis);
                }
                induction_hypotheses.extend(
                    constructor_args
                        .into_iter()
                        .map(LoweringOperand::Specialized),
                );
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
                let edge = self.lowering_boundary_use_token(
                    LoweringOnlyOperandEdge::DeferredConstructorTrailingField,
                    deferred.construct_origin,
                    0,
                )?;
                constructor_args.extend(specialized_env_at(&outer_tail, edge)?);
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

    fn lower_source_machine_with_result_origin(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        occurrence: SourceOccurrence<'_>,
        env: &[LoweringOperand],
        active: &ActiveContinuationFrame<'_>,
    ) -> Result<
        (LoweringOperand, Option<ContinuationSpecializationCallToken>),
        CraneliftBackendError,
    > {
        let previous = self.returned_source_continuation_result_origin.take();
        let result = self.lower_source_machine(builder, occurrence, env, active);
        let result_origin = self.returned_source_continuation_result_origin.take();
        self.returned_source_continuation_result_origin = previous;
        result.map(|value| (value, result_origin))
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
                } => {
                    if !self
                        .function_local
                        .continuation_specialization_calls
                        .is_empty()
                    {
                    }
                    match {
                        // The owned source machine is the third traversal route for
                        // these joins. Record the source occurrence here; later
                        // continuation helpers may only reborrow its token.
                        self.enter_source_occurrence_plan(static_origin)?;
                        expr
                    } {
                        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::WrapperBody,
                            )?;
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
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::WrapperBody,
                            )?;
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
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::WrapperBody,
                            )?;
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
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::WrapperBody,
                            )?;
                            self.enter_checked_computational_ih_invocation(call_template_id)?;
                            control.continuation =
                                SourceContinuation::CheckedComputationalIHInvocationReturn {
                                    call_template_id,
                                    marker_origin: static_origin,
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
                            continuation_result_origin: None,
                            control,
                        },
                        RuntimeExpr::Var(index) => {
                            let value = env.get(index as usize).cloned().ok_or_else(|| {
                                unsupported("Var", format!("no runtime binding for index {index}"))
                            })?;
                            SourceMachineState::Value {
                                value,
                                continuation_result_origin: None,
                                control,
                            }
                        }
                        RuntimeExpr::Let { value, body } => {
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::LetValue,
                            )?;
                            control.continuation = SourceContinuation::LetBody {
                                let_origin: static_origin,
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
                                    value: LoweringOperand::Specialized(
                                        self.finish_source_constructor(
                                            builder,
                                            constructor,
                                            static_origin,
                                            vec![],
                                        )?,
                                    ),
                                    continuation_result_origin: self
                                        .continuation_specialization_result_origin(static_origin),
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
                                    continuation_result_origin: None,
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
                                call_origin: static_origin,
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
                            self.function_local
                                .continuation_specialization_environments
                                .insert(static_origin, env.clone());
                            let checked_frame_id =
                                self.consume_checked_subcontinuation_frame(&cases, &default)?;
                            control.continuation =
                                SourceContinuation::ComputationalMatchScrutinee {
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
                            continuation_result_origin: self
                                .continuation_specialization_result_origin(static_origin),
                            control,
                        },
                    }
                }
                SourceMachineState::Value {
                    value,
                    continuation_result_origin,
                    mut control,
                } => {
                    if continuation_result_origin.is_some() {
                    }
                    if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                        control.continuation = Self::discard_source_prefix(control.continuation);
                    }
                    match control.continuation {
                        SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                            self.returned_source_continuation_result_origin =
                                continuation_result_origin;
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
                            SourceMachineState::Value {
                                value,
                                continuation_result_origin,
                                control,
                            }
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
                            let mut active =
                                control.selected.as_active(&control.selected_lineage);
                            let value = if let Some(producer_origin) =
                                continuation_result_origin
                            {
                                let worker = self
                                    .continuation_specialization_worker_for_result(
                                        producer_origin,
                                    )?;
                                let exits_emitted_owner = self
                                    .continuation_specialization_exits_emitted_owner(
                                        producer_origin,
                                    )?;
                                active = self
                                    .bypass_active_continuation_specializations_for_result(
                                        producer_origin,
                                        active,
                                    )?;
                                let value = self.call_continuation_specialization_if_planned(
                                    builder,
                                    producer_origin,
                                    value,
                                )?;
                                if exits_emitted_owner {
                                    return Ok(value);
                                }
                                let previous = worker
                                    .map(|worker| {
                                        self.active_static_recursor_selection.replace(worker)
                                    })
                                    .flatten();
                                let result =
                                    self.resume_active_continuation(builder, value, active);
                                if worker.is_some() {
                                    self.active_static_recursor_selection = previous;
                                }
                                return result;
                            } else {
                                value
                            };
                            return self.resume_active_continuation(builder, value, active);
                        }
                        SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(
                            edge,
                        )) => {
                            if continuation_result_origin.is_some() {
                            }
                            if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                let failure = builder.ins().iconst(types::I64, -4);
                                builder.ins().return_(&[failure]);
                                return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                            }
                            if matches!(
                                value,
                                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
                            ) {
                                self.disposition_lowering_boundary_use_if_planned(
                                        LoweringOnlyOperandEdge::JoinArm,
                                        edge.producer_origin,
                                        0,
                                    )?;
                                return Ok(value);
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
                            let call_token = continuation_result_origin.or_else(|| {
                                self.continuation_specialization_result_origin(
                                    edge.producer_origin,
                                )
                            });
                            let exits_emitted_owner = match call_token {
                                Some(token) => {
                                    self.continuation_specialization_exits_emitted_owner(token)?
                                }
                                None => false,
                            };
                            let value = match call_token {
                                Some(token) => self
                                    .call_continuation_specialization_if_planned(
                                        builder, token, value,
                                    )?,
                                None => value,
                            };
                            if exits_emitted_owner {
                                self.disposition_lowering_boundary_use_if_planned(
                                    LoweringOnlyOperandEdge::JoinArm,
                                    edge.producer_origin,
                                    0,
                                )?;
                                return Ok(value);
                            }
                            match edge.target.join_plan.representation {
                                JoinResultRepresentation::NativeScalarPair => {
                                    let (value, actual_kind) =
                                        self.merge_planned_scalar_branch(
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
                                                "predecessor {} for join {} produced \
                                                 {actual_kind:?}, planned {:?}",
                                                edge.predecessor_identity,
                                                edge.target.join_id,
                                                edge.target.required_kind
                                            ),
                                        ));
                                    }
                                    builder.ins().jump(
                                        edge.target.block,
                                        &[value.tag.into(), value.payload.into()],
                                    );
                                }
                                JoinResultRepresentation::CarrierWord => {
                                    let word = self.carried_join_arm(
                                        builder,
                                        edge.producer_origin,
                                        LoweringOnlyOperandEdge::JoinArm,
                                        0,
                                        value,
                                        Some(edge.target.required_kind),
                                        "NativeJoinPlanV1",
                                    )?;
                                    builder
                                        .ins()
                                        .jump(edge.target.block, &[word.word.into()]);
                                }
                            }
                            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
                        }
                        SourceContinuation::LetBody {
                            let_origin,
                            body,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            if matches!(value, LoweringOperand::Specialized(Lowered::RecursiveBackedge)) {
                                SourceMachineState::Value {
                                    value,
                                    continuation_result_origin: None,
                                    control,
                                }
                            } else if matches!(value, LoweringOperand::Specialized(Lowered::Trap(_))) {
                                SourceMachineState::Value {
                                    value,
                                    continuation_result_origin: None,
                                    control,
                                }
                            } else {
                                self.close_reached_operand_edge(
                                    let_origin,
                                    1,
                                    SourceOperandRole::LetBody,
                                )?;
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
                            SourceMachineState::Value {
                                value,
                                continuation_result_origin,
                                control,
                            }
                        }
                        SourceContinuation::CheckedComputationalIHInvocationReturn {
                            call_template_id,
                            marker_origin,
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
                                self.finish_checked_computational_ih_marker(marker_origin, value)?;
                            control.continuation = *next;
                            SourceMachineState::Value {
                                value,
                                continuation_result_origin,
                                control,
                            }
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
                            SourceMachineState::Value {
                                value,
                                continuation_result_origin,
                                control,
                            }
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
                            SourceMachineState::Value {
                                value,
                                continuation_result_origin,
                                control,
                            }
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
                                SourceMachineState::Value {
                                    value,
                                    continuation_result_origin,
                                    control,
                                }
                            } else {
                                control.continuation = *next;
                                SourceMachineState::Value {
                                    value,
                                    continuation_result_origin,
                                    control,
                                }
                            }
                        }
                        SourceContinuation::ConstructArgument {
                            constructor,
                            static_origin,
                            mut remaining,
                            mut lowered,
                            continuation_result_origin:
                                mut constructed_continuation_result_origin,
                            env,
                            next,
                        } => {
                            constructed_continuation_result_origin =
                                Self::merge_continuation_result_origins(
                                    constructed_continuation_result_origin,
                                    continuation_result_origin,
                                )?;
                            lowered.push(value);
                            control.continuation = *next;
                            if remaining.is_empty() {
                                let own_continuation_result_origin = self
                                    .continuation_specialization_result_origin(static_origin);
                                let must_cross_specialization =
                                    own_continuation_result_origin.is_some();
                                let (
                                    nearest_continuation_result_origin,
                                    retained_continuation_result_origin,
                                ) = match (
                                    constructed_continuation_result_origin,
                                    own_continuation_result_origin,
                                ) {
                                    (Some(inner), Some(outer)) if inner != outer => {
                                        (Some(inner), Some(outer))
                                    }
                                    (retained, own) => {
                                        (
                                            Self::merge_continuation_result_origins(
                                                retained, own,
                                            )?,
                                            None,
                                        )
                                    }
                                };
                                let flattens_nearest = match
                                    nearest_continuation_result_origin
                                {
                                    Some(producer_origin) => self
                                        .continuation_specialization_flattens_result(
                                            producer_origin,
                                        )?,
                                    None => false,
                                };
                                if flattens_nearest {
                                    let producer_origin =
                                        nearest_continuation_result_origin
                                            .expect("flattened continuation result exists");
                                    let exits_emitted_owner = self
                                        .continuation_specialization_exits_emitted_owner(
                                        producer_origin,
                                    )?;
                                    let completed = self
                                        .call_known_constructor_continuation_specialization(
                                            builder,
                                            producer_origin,
                                            static_origin,
                                            lowered,
                                        )?;
                                    if retained_continuation_result_origin.is_some() {
                                        state = SourceMachineState::Value {
                                            value: completed,
                                            continuation_result_origin:
                                                retained_continuation_result_origin,
                                            control,
                                        };
                                        continue;
                                    }
                                    let continuation = control.continuation;
                                    if exits_emitted_owner {
                                        match continuation {
                                            SourceContinuation::Terminal(
                                                SourceContinuationTerminal::ResumeOuter {
                                                    active,
                                                    ..
                                                },
                                            ) => {
                                                self.bypass_active_continuation_specializations_for_result(
                                                    producer_origin,
                                                    *active,
                                                )?;
                                            }
                                            continuation @
                                            SourceContinuation::ComputationalMatchScrutinee {
                                                ..
                                            } => {
                                                self.bypass_continuation_specializations_for_result(
                                                    producer_origin,
                                                    continuation,
                                                )?;
                                            }
                                            _ => {
                                                return Err(backend(
                                                    BackendFailure::PlannerInvariant(
                                                        "known continuation result is not adjacent \
                                                         to its checked out-of-line suffix"
                                                            .to_string(),
                                                    ),
                                                ));
                                            }
                                        }
                                        return Ok(completed);
                                    }
                                    match continuation {
                                        SourceContinuation::Terminal(
                                            SourceContinuationTerminal::ResumeOuter {
                                                expected,
                                                active,
                                                root_authority,
                                            },
                                        ) => {
                                            if active.cursor != expected {
                                                return Err(unsupported(
                                                    "ComputationalRecursor",
                                                    "source continuation terminal cursor mismatch",
                                                ));
                                            }
                                            self.restore_root_terminal_authority(
                                                root_authority,
                                                expected,
                                            )?;
                                            let active = self
                                                .bypass_active_continuation_specializations_for_result(
                                                    producer_origin,
                                                    *active,
                                                )?;
                                            return self.resume_active_continuation(
                                                builder,
                                                completed,
                                                active,
                                            );
                                        }
                                        continuation => {
                                            control.continuation = self
                                                .bypass_continuation_specializations_for_result(
                                                    producer_origin,
                                                    continuation,
                                                )?;
                                            state = SourceMachineState::Value {
                                                value: completed,
                                                continuation_result_origin: None,
                                                control,
                                            };
                                            continue;
                                        }
                                    }
                                }
                                let mut completed = if must_cross_specialization
                                    || lowered.iter().any(|argument| {
                                        matches!(argument, LoweringOperand::Carried(_))
                                    })
                                {
                                    let lowered =
                                        self.split_static_recursor_worker_operands(
                                            builder, lowered,
                                        )?;
                                    let lowered = self
                                        .materialize_continuation_specialization_worker_operands(
                                            builder,
                                            static_origin,
                                            lowered,
                                        )?;
                                    LoweringOperand::Carried(
                                        self.transfer_constructor_operands(
                                            builder,
                                            static_origin,
                                            &constructor,
                                            &lowered,
                                        )?,
                                    )
                                } else {
                                    LoweringOperand::Specialized(
                                        self.finish_source_constructor(
                                            builder,
                                            constructor,
                                            static_origin,
                                            self.specialized_source_env_at(
                                                &lowered,
                                                static_origin,
                                                0,
                                                SourceOperandRole::ConstructArgument,
                                            )?,
                                        )?,
                                    )
                                };
                                let completed_continuation_result_origin =
                                    match (
                                        constructed_continuation_result_origin,
                                        own_continuation_result_origin,
                                    ) {
                                        (Some(inner), Some(outer)) if inner != outer => {
                                            completed = self
                                                .call_continuation_specialization_if_planned(
                                                    builder, inner, completed,
                                                )?;
                                            Some(outer)
                                        }
                                        (retained, own) => {
                                            Self::merge_continuation_result_origins(retained, own)?
                                        }
                                    };
                                if completed_continuation_result_origin.is_some() {
                                }
                                SourceMachineState::Value {
                                    value: completed,
                                    continuation_result_origin:
                                        completed_continuation_result_origin,
                                    control,
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::ConstructArgument {
                                    constructor,
                                    static_origin,
                                    remaining,
                                    lowered,
                                    continuation_result_origin:
                                        constructed_continuation_result_origin,
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
                            self.enter_source_occurrence_plan(static_origin)?;
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::MatchScrutinee,
                            )?;
                            control.continuation = *next;
                            if let LoweringOperand::Carried(word) = &value {
                                return self.lower_source_carried_match(
                                    builder,
                                    *word,
                                    &cases,
                                    &default,
                                    static_origin,
                                    &env,
                                    control,
                                );
                            }
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
                                        self.disposition_statically_unselected_match_cases(
                                            static_origin,
                                            Some(index),
                                        )?;
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
                                        self.disposition_statically_unselected_match_cases(
                                            static_origin,
                                            Some(true_index),
                                        )?;
                                        self.disposition_statically_unselected_match_cases(
                                            static_origin,
                                            Some(false_index),
                                        )?;
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
                                        self.disposition_statically_unselected_match_cases(
                                            static_origin,
                                            None,
                                        )?;
                                        return Ok(LoweringOperand::Specialized(Lowered::Trap(default)));
                                    };
                                    self.disposition_statically_unselected_match_cases(
                                        static_origin,
                                        Some(case_index),
                                    )?;
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
                            self.function_local
                                .continuation_specialization_environments
                                .insert(static_origin, env.clone());
                            if let Some(producer_origin) = continuation_result_origin {
                                let continuation =
                                    SourceContinuation::ComputationalMatchScrutinee {
                                        cases,
                                        default,
                                        env,
                                        static_origin,
                                        provenance,
                                        checked_frame_id,
                                        answer_route,
                                        next,
                                    };
                                let value = self
                                    .call_continuation_specialization_if_planned(
                                        builder,
                                        producer_origin,
                                        value,
                                    )?;
                                control.continuation = self
                                    .bypass_continuation_specializations_for_result(
                                        producer_origin,
                                        continuation,
                                    )?;
                                break 'computational_scrutinee SourceMachineState::Value {
                                    value,
                                    continuation_result_origin: None,
                                    control,
                                };
                            }
                            self.enter_source_occurrence_plan(static_origin)?;
                            self.close_reached_operand_edge(
                                static_origin,
                                0,
                                SourceOperandRole::MatchScrutinee,
                            )?;
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
                                    continuation_result_origin: None,
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
                                self.record_source_machine_computational_match_selection(
                                    static_origin,
                                    Some(selected.0),
                                )?;
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
                                    self.record_source_machine_computational_match_selection(
                                        static_origin,
                                        None,
                                    )?;
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
                                self.record_source_machine_computational_match_selection(
                                    static_origin,
                                    Some(return_index),
                                )?;
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
                                self.record_source_machine_computational_match_selection(
                                    static_origin,
                                    None,
                                )?;
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
                            let LoweringOperand::Specialized(Lowered::Constructor {
                                aggregate_origin,
                                args,
                                ..
                            }) = value else {
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
                                    let recursive_worker = match aggregate_origin {
                                        Some(producer_origin) => self
                                            .selected_static_recursor_worker_for_producer(
                                                static_origin,
                                                position,
                                                producer_origin,
                                            )?,
                                        None => self.selected_static_recursor_worker(
                                            static_origin,
                                            position,
                                        )?,
                                    };
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
                                        recursive_worker,
                                    )?;
                                    #[cfg(test)]
                                    px8j_record_recursor_carrier(
                                        Px8jProducerPath::SourceMachine,
                                        &induction_hypothesis,
                                    );
                                    induction_hypotheses.push(induction_hypothesis);
                                }
                            }
                            let edge = self.reached_operand_edge_token(
                                frame.static_origin,
                                0,
                                SourceOperandRole::MatchScrutinee,
                            )?;
                            let frame_env = match self.materialize_eliminator_frame_env(
                                builder,
                                EliminatorFrame::Computational(frame),
                                retained.specialized_ref_at(edge)?,
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
                            call_origin,
                            mut args,
                            env,
                            next,
                        } => {
                            control.continuation = *next;
                            if args.is_empty() {
                                match self.source_call_state(
                                    builder,
                                    call_origin,
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
                                    call_origin,
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
                            call_origin,
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
                                    .source_call_state(
                                        builder,
                                        call_origin,
                                        callee,
                                        lowered,
                                        env,
                                        control,
                                    )?
                                {
                                    SourceCallOutcome::Continue(state) => state,
                                    SourceCallOutcome::Complete(value) => return Ok(value),
                                }
                            } else {
                                let first = remaining.remove(0);
                                control.continuation = SourceContinuation::CallArgument {
                                    call_origin,
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

    fn prepare_static_recursor_residual(
        &self,
        residual: LoweringOperand,
        invocation: &RecursorInvocationSegment,
    ) -> Result<PreparedStaticRecursorResidual, CraneliftBackendError> {
        // RecursiveDescent keeps the established inline recursor. Its exact
        // worker or generic authority was consumed when the induction
        // hypothesis was minted; no generated carrier/frame/object exists to
        // prepare at invocation time.
        if matches!(
            self.body_emission_authority,
            BodyEmissionAuthority::RecursiveDescent
        ) {
            return Ok(PreparedStaticRecursorResidual::Passthrough(residual));
        }
        let Some(worker) = invocation.recursive_worker else {
            return Ok(PreparedStaticRecursorResidual::Passthrough(residual));
        };
        if worker.parent_origin != invocation.selection.static_origin
            || worker.sibling_position != invocation.sibling_position
        {
            return Err(unsupported(
                "StaticRecursorWorker",
                "the recursor invocation disagrees with its static worker edge",
            ));
        }
        if matches!(residual, LoweringOperand::Carried(_)) {
            self.static_transition_plan
                .validate_static_recursor_worker_residual_identity(
                    worker.boundary_identity,
                    worker.residual_id,
                    worker.parent_origin,
                    worker.producer_origin,
                    worker.sibling_position,
                    worker.closure_origin,
                    worker.body_origin,
                    worker.declared_arity,
                    worker.capture_count,
                )?;
            return Ok(PreparedStaticRecursorResidual::Passthrough(residual));
        }
        self.prepare_planned_static_recursor_worker(residual, worker)
    }

    fn prepare_planned_static_recursor_worker(
        &self,
        residual: LoweringOperand,
        worker: StaticRecursorWorker,
    ) -> Result<PreparedStaticRecursorResidual, CraneliftBackendError> {
        let LoweringOperand::Specialized(Lowered::Closure {
            captures,
            params,
            body,
        }) = residual
        else {
            return Err(unsupported(
                "StaticRecursorWorker",
                "a callable recursor worker has no closure residual",
            ));
        };
        self.static_transition_plan
            .validate_static_recursor_worker_residual_identity(
                worker.boundary_identity,
                worker.residual_id,
                worker.parent_origin,
                worker.producer_origin,
                worker.sibling_position,
                worker.closure_origin,
                worker.body_origin,
                worker.declared_arity,
                worker.capture_count,
            )?;
        if body != worker.body_origin
            || params.len() != worker.declared_arity
            || captures.len() != worker.capture_count
        {
            return Err(unsupported(
                "StaticRecursorWorker",
                "the residual environment disagrees with its static worker target",
            ));
        }
        let captures = captures
            .into_iter()
            .enumerate()
            .map(|(ordinal, capture)| {
                let token = self.static_transition_plan.static_recursor_capture_token(
                    worker.boundary_identity,
                    worker.residual_id,
                    worker.parent_origin,
                    worker.producer_origin,
                    worker.sibling_position,
                    worker.closure_origin,
                    ordinal,
                )?;
                if token.ordinal as usize != ordinal
                    || token.closure_origin != worker.closure_origin
                    || token.phase != OperandEdgeDisposition::CallableCapture
                    || token.lifetime != StaticRecursorCaptureLifetime::ActivationOwned
                {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "static recursor capture contract disagrees with its ordered worker"
                            .to_string(),
                    )));
                }
                match capture {
                    LoweringOperand::Carried(capture) => {
                        Ok(PreparedStaticRecursorCapture::Carried(capture))
                    }
                    LoweringOperand::Specialized(value) => {
                        value.boundary_transfer_admissibility(&token.edge)?;
                        Ok(PreparedStaticRecursorCapture::Specialized {
                            origin: token.source_origin,
                            value,
                        })
                    }
                }
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let environment = self
            .static_transition_plan
            .static_recursor_worker_environment_occurrence(
                worker.boundary_identity,
                worker.residual_id,
                self.active_emission_owner.ok_or_else(|| {
                    backend(BackendFailure::PlannerInvariant(
                        "static recursor worker environment has no emitted owner".to_string(),
                    ))
                })?,
                worker.parent_origin,
                worker.producer_origin,
                worker.sibling_position,
                worker.closure_origin,
                worker.capture_count,
            )
            .and_then(|occurrence| {
                self.static_transition_plan
                    .static_recursor_worker_environment_token(
                        occurrence,
                        BoundaryClass::Record,
                        worker.capture_count,
                    )
            })?;
        Ok(PreparedStaticRecursorResidual::Worker {
            worker,
            captures,
            environment,
        })
    }

    fn prepare_static_recursor_constructor_residual(
        &self,
        closure_origin: StaticOriginId,
        residual: LoweringOperand,
    ) -> Result<Option<PreparedStaticRecursorResidual>, CraneliftBackendError> {
        let Some(token) = self
            .static_transition_plan
            .static_recursor_worker_residual_token_for_closure(closure_origin)?
        else {
            return Ok(None);
        };
        if token.disposition() != OperandEdgeDisposition::CallableCapture {
            return Err(backend(BackendFailure::PlannerInvariant(
                "static recursor constructor residual is not callable-capture".to_string(),
            )));
        }
        let worker = StaticRecursorWorker {
            boundary_identity: token.identity(),
            residual_id: token.id,
            parent_origin: token.parent_origin,
            producer_origin: token.producer_origin,
            sibling_position: token.sibling_position as usize,
            closure_origin: token.closure_origin,
            body_origin: token.body_origin,
            declared_arity: token.declared_arity as usize,
            capture_count: token.capture_count as usize,
        };
        self.prepare_planned_static_recursor_worker(residual, worker)
            .map(Some)
    }

    fn materialize_static_recursor_residual(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        prepared: PreparedStaticRecursorResidual,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let PreparedStaticRecursorResidual::Worker {
            worker,
            captures,
            environment,
        } = prepared
        else {
            let PreparedStaticRecursorResidual::Passthrough(residual) = prepared else {
                unreachable!("prepared residual has two closed variants")
            };
            return Ok(residual);
        };
        if matches!(
            self.body_emission_authority,
            BodyEmissionAuthority::RecursiveDescent
        ) {
            let captures = captures
                .into_iter()
                .map(|capture| match capture {
                    PreparedStaticRecursorCapture::Carried(capture) => {
                        LoweringOperand::Carried(capture)
                    }
                    PreparedStaticRecursorCapture::Specialized { value, .. } => {
                        LoweringOperand::Specialized(value)
                    }
                })
                .collect();
            return Ok(LoweringOperand::Specialized(Lowered::Closure {
                captures,
                params: vec![String::new(); worker.declared_arity],
                body: worker.body_origin,
            }));
        }
        let captures = captures
            .into_iter()
            .map(|capture| match capture {
                PreparedStaticRecursorCapture::Carried(capture) => Ok(capture),
                PreparedStaticRecursorCapture::Specialized { origin, value } => {
                    self.emit_carrier_transfer(builder, origin, &value)
                }
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
        let environment = self.emit_carrier_alloc(
            builder,
            environment.tag(),
            environment.class(),
            worker.capture_count,
        )?;
        for (position, capture) in captures.into_iter().enumerate() {
            self.emit_carrier_store_field(builder, environment, position, capture)?;
        }
        Ok(LoweringOperand::Carried(environment))
    }

    pub(super) fn split_static_recursor_worker_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        operands: Vec<LoweringOperand>,
    ) -> Result<Vec<LoweringOperand>, CraneliftBackendError> {
        operands
            .into_iter()
            .map(|operand| {
                let LoweringOperand::Specialized(
                    recursor @ Lowered::ComputationalRecursorClosure { .. },
                ) = operand
                else {
                    return Ok(operand);
                };
                let (residual, boundary) =
                    decompose_computational_recursor(LoweringOperand::Specialized(recursor));
                let (activation, invocation) =
                    boundary.expect("matched recursor closure carries its invocation segment");
                if !recursor_invocation_is_checked(&invocation) {
                    validate_recursor_invocation_segment(&invocation)?;
                }
                let prepared = self.prepare_static_recursor_residual(residual, &invocation)?;
                let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                compose_oriented_subcontinuation(
                    self.oriented_subcontinuation_plan.as_ref(),
                    None,
                    activation,
                    invocation,
                    dynamic_splice_edges,
                )?;
                self.materialize_static_recursor_residual(builder, prepared)
            })
            .collect()
    }

    fn materialize_continuation_specialization_worker_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        producer_origin: StaticOriginId,
        mut operands: Vec<LoweringOperand>,
    ) -> Result<Vec<LoweringOperand>, CraneliftBackendError> {
        let Some(calls) = self
            .function_local
            .continuation_specialization_calls
            .get(&producer_origin)
            .cloned()
        else {
            return Ok(operands);
        };
        let token = self
            .static_transition_plan
            .continuation_specialization_worker_token(calls.plan.id())?;
            let position = usize::try_from(token.sibling_position).map_err(|_| {
                backend_module(
                    "continuation specialization worker position exceeds usize".to_string(),
                )
            })?;
            let operand = operands.get(position).cloned().ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "continuation specialization worker position exceeds producer payload"
                        .to_string(),
                ))
            })?;
            if matches!(operand, LoweringOperand::Carried(_)) {
                return Ok(operands);
            }
            let worker = StaticRecursorWorker {
                boundary_identity: token.identity(),
                residual_id: token.id,
                parent_origin: token.parent_origin,
                producer_origin: token.producer_origin,
                sibling_position: position,
                closure_origin: token.closure_origin,
                body_origin: token.body_origin,
                declared_arity: usize::try_from(token.declared_arity).map_err(|_| {
                    backend_module(
                        "continuation specialization worker arity exceeds usize".to_string(),
                    )
                })?,
                capture_count: usize::try_from(token.capture_count).map_err(|_| {
                    backend_module(
                        "continuation specialization capture count exceeds usize".to_string(),
                    )
                })?,
            };
            let prepared = self.prepare_planned_static_recursor_worker(operand, worker)?;
            operands[position] = self.materialize_static_recursor_residual(builder, prepared)?;
        Ok(operands)
    }

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
        self.disposition_statically_unselected_match_cases(static_origin, Some(zero_index))?;
        self.disposition_statically_unselected_match_cases(static_origin, Some(suc_index))?;
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
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
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
                        result_origin: static_origin,
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
            let edge = self.mint_source_predecessor(target.clone(), case_body.static_origin);
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
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
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
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
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
        self.resume_active_continuation(builder, merged?, suffix_active)
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
    /// `(emitted_owner, invocation_id, frame_id)` is misreported as "consumed
    /// more than once"
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
        frame_baseline: &std::collections::BTreeSet<(Option<PredeclaredFunctionId>, u64, u64)>,
        frame_union: &mut std::collections::BTreeSet<(Option<PredeclaredFunctionId>, u64, u64)>,
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
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
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
                    result_origin: static_origin,
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
            let edge = self.mint_source_predecessor(target.clone(), body.static_origin);
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
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
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
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
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
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    fn lower_source_carried_match<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        cases: &[crate::RuntimeMatchCase],
        default: &RuntimeTrap,
        static_origin: StaticOriginId,
        env: &[LoweringOperand],
        suffix_control: SourceControl<'b>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        if cases.is_empty() {
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
        }
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
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
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
                    result_origin: static_origin,
                    terminal_active_prefix: prefix,
                }
            }
        };

        let frame_baseline = self.consumed_subcontinuation_frames.clone();
        let mut frame_union = frame_baseline.clone();
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
            let owner = self.active_emission_owner.ok_or_else(|| {
                backend_module(
                    "carried Match emission has no exact generated-function owner".to_string(),
                )
            })?;
            for (index, _) in [ok_case, err_case] {
                let token =
                    self.static_transition_plan
                        .case_emission_token(owner, static_origin, index)?;
                if !token.is_reachable() {
                    return Err(backend_module(
                        "HostResult runtime selection targets a planner-eliminated case"
                            .to_string(),
                    ));
                }
                self.disposition_statically_unselected_match_cases(static_origin, Some(index))?;
            }
            let ok_block = builder.create_block();
            builder.append_block_param(ok_block, types::I64);
            let err_block = builder.create_block();
            builder.append_block_param(err_block, types::I64);

            let class = self.emit_carrier_class(builder, scrutinee)?;
            let is_host_result = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                class,
                BoundaryClass::HostResult as i64,
            );
            let host_result = builder.create_block();
            let represented = builder.create_block();
            builder
                .ins()
                .brif(is_host_result, host_result, &[], represented, &[]);

            builder.switch_to_block(host_result);
            let success = self.emit_carrier_host_success(builder, scrutinee)?;
            let payload = self.emit_carrier_host_payload(builder, scrutinee)?;
            builder.ins().brif(
                success,
                ok_block,
                &[payload.word.into()],
                err_block,
                &[payload.word.into()],
            );

            builder.switch_to_block(represented);
            let tag = self.emit_carrier_tag(builder, scrutinee)?;
            let field_count = self.emit_carrier_field_count(builder, scrutinee)?;
            for (block, (index, _)) in [(ok_block, ok_case), (err_block, err_case)] {
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
                builder.ins().jump(block, &[payload.word.into()]);
                builder.switch_to_block(next);
            }
            let defaulted = LoweringOperand::Specialized(Lowered::Trap(default.clone()));
            if !self.seal_source_trap_branch(builder, &defaulted)? {
                return Err(unsupported(
                    "Match",
                    "the source carried Result default did not seal its branch",
                ));
            }

            for (block, (index, case)) in [(ok_block, ok_case), (err_block, err_case)] {
                builder.switch_to_block(block);
                let payload = CarriedBoundaryWord {
                    word: builder.block_params(block)[0],
                };
                let body =
                    self.owned_case_body_occurrence(static_origin, index, case.body.clone())?;
                let edge = self.mint_source_predecessor(target.clone(), body.static_origin);
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
                    body,
                    env_with_operands([LoweringOperand::Carried(payload)], env),
                    branch_control,
                )?;
                self.require_source_branch_sealed(builder, &lowered, "carried Result predecessor")?;
            }
        } else {
            let tag = self.emit_carrier_tag(builder, scrutinee)?;
            let field_count = self.emit_carrier_field_count(builder, scrutinee)?;
            let owner = self.active_emission_owner.ok_or_else(|| {
                backend_module(
                    "carried Match emission has no exact generated-function owner".to_string(),
                )
            })?;
            for (index, case) in cases.iter().enumerate() {
                let token =
                    self.static_transition_plan
                        .case_emission_token(owner, static_origin, index)?;
                if !token.is_reachable() {
                    continue;
                }
                self.disposition_statically_unselected_match_cases(static_origin, Some(index))?;
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
                let binders = i64::try_from(case.binders).map_err(|_| {
                    unsupported(
                        "BoundaryCarrier",
                        "a case binds more fields than the carrier ABI can count",
                    )
                })?;
                Self::require_i64(builder, field_count, binders);
                let mut bindings = Vec::with_capacity(case.binders);
                for position in 0..case.binders {
                    bindings.push(LoweringOperand::Carried(
                        self.emit_carrier_field(builder, scrutinee, position)?,
                    ));
                }
                let body =
                    self.owned_case_body_occurrence(static_origin, index, case.body.clone())?;
                let edge = self.mint_source_predecessor(target.clone(), body.static_origin);
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
                    body,
                    env_with_operands(bindings, env),
                    branch_control,
                )?;
                self.require_source_branch_sealed(
                    builder,
                    &lowered,
                    "carried constructor predecessor",
                )?;
                builder.switch_to_block(next);
            }
            let defaulted = LoweringOperand::Specialized(Lowered::Trap(default.clone()));
            if !self.seal_source_trap_branch(builder, &defaulted)? {
                return Err(unsupported(
                    "Match",
                    "the source carried match default did not seal its branch",
                ));
            }
        }
        self.consumed_subcontinuation_frames = frame_union;

        let Some((merge, suffix_pending, required_kind, _site_id, root_authority)) =
            local_completion
        else {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        };
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
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
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    fn require_source_branch_sealed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: &LoweringOperand,
        label: &str,
    ) -> Result<(), CraneliftBackendError> {
        if self.seal_source_trap_branch(builder, lowered)?
            || matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            )
        {
            return Ok(());
        }
        Err(unsupported(
            "NativeJoinPlanV1",
            format!("{label} did not seal its distinct affine join edge"),
        ))
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
                let join_plan = std::rc::Rc::new(self.consumed_join_plan_token(static_origin)?);
                let merge = builder.create_block();
                self.append_planned_join_params(builder, merge, join_plan.as_ref());
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
                    result_origin: static_origin,
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
            let edge = self.mint_source_predecessor(target.clone(), static_origin);
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
                self.disposition_statically_unselected_match_cases(static_origin, Some(index))?;
                let body =
                    self.owned_case_body_occurrence(static_origin, index, case.body.clone())?;
                if !self
                    .static_transition_plan
                    .source_environment_slot_is_used(body.static_origin, 0)?
                {
                    self.disposition_unobserved_lowered_value(&payload)?;
                }
                let arm_env = env_with([payload], env);
                self.lower_forked_branch(
                    builder,
                    &frame_baseline,
                    &mut frame_union,
                    body,
                    arm_env,
                    branch_control,
                )?
            } else {
                self.disposition_statically_unselected_match_cases(static_origin, None)?;
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
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
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
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
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
        self.resume_active_continuation(builder, merged?, suffix_active)
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
                    Ok(selected) => {
                        self.disposition_statically_unselected_match_cases(
                            static_origin,
                            Some(selected.0),
                        )?;
                        selected
                    }
                    Err(_) => {
                        self.disposition_statically_unselected_match_cases(static_origin, None)?;
                        let failure = builder.ins().iconst(types::I64, -4);
                        builder.ins().return_(&[failure]);
                        test_block = next;
                        continue;
                    }
                };
            let body =
                self.owned_case_body_occurrence(static_origin, case_index, case.body.clone())?;
            let edge = self.mint_source_predecessor(target.clone(), body.static_origin);
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
                body,
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
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
        self.append_planned_join_params(builder, merge, join_plan.as_ref());
        let target = SourceJoinTarget {
            join_id,
            block: merge,
            expected_outer: suffix_control.terminal_outer,
            required_kind,
            join_plan,
            result_origin: static_origin,
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
                    Ok(selected) => {
                        self.disposition_statically_unselected_match_cases(
                            static_origin,
                            Some(selected.0),
                        )?;
                        selected
                    }
                    Err(_) => {
                        self.disposition_statically_unselected_match_cases(static_origin, None)?;
                        let failure = builder.ins().iconst(types::I64, -4);
                        builder.ins().return_(&[failure]);
                        test_block = next;
                        continue;
                    }
                };
            let body =
                self.owned_case_body_occurrence(static_origin, case_index, case.body.clone())?;
            let edge = self.mint_source_predecessor(target.clone(), body.static_origin);
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
                body,
                materialize_dynamic_constructor_env(&alternative, env),
                control,
            )?;
            if self.seal_source_trap_branch(builder, &lowered)? {
                // A trap terminates this mutually exclusive predecessor.
            } else if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
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
        let merged = self.finish_planned_join(
            builder,
            merge,
            target.join_plan.as_ref(),
            Some(required_kind),
            "NativeJoinPlanV1",
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
        self.resume_active_continuation(builder, merged?, suffix_active)
    }

    fn source_call_state<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        call_origin: StaticOriginId,
        callee: LoweringOperand,
        args: Vec<LoweringOperand>,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        // ⭐ A call needs a **callable template** — `params`, `captures`, a body
        // occurrence. A carried boundary word carries none of those and cannot
        // acquire them (`§2g`: the carrier holds the SSA word and nothing else),
        // so this is a specialized-only surface. ⛔ Fails closed.
        let edge =
            self.reached_operand_edge_token(call_origin, 0, SourceOperandRole::CallCallee)?;
        for position in 0..args.len() {
            let argument = self.reached_operand_edge_token(
                call_origin,
                1 + position,
                SourceOperandRole::CallArgument,
            )?;
            if argument.disposition() != OperandEdgeDisposition::Forwarding {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "source-machine call argument lost its forwarding disposition".to_string(),
                )));
            }
        }
        let callee = callee.specialized_at(edge)?;
        match callee {
            Lowered::Closure {
                captures,
                params,
                body,
            } => {
                self.disposition_recursive_declaration_call_alternatives(
                    call_origin,
                    false,
                    false,
                    false,
                )?;
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
                    expr: self.machine_body_occurrence(body)?,
                    env: call_env,
                    control,
                }))
            }
            Lowered::DeclarationClosure {
                reference_origin,
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
                    builder,
                    call_origin,
                    reference_origin,
                    symbol,
                    captures,
                    body,
                    args,
                    env,
                    control,
                )
            }
            mut recursor @ Lowered::ComputationalRecursorClosure { .. } => {
                self.disposition_recursive_declaration_call_alternatives(
                    call_origin,
                    false,
                    false,
                    false,
                )?;
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
                let (base, boundary) =
                    decompose_computational_recursor(LoweringOperand::Specialized(recursor));
                let (activation, invocation) =
                    boundary.expect("recursor closure carries an invocation segment");
                let recursive_worker = invocation.recursive_worker;
                let recursor_parent = invocation.selection.static_origin;
                let sibling_position = invocation.sibling_position;
                let prepared = self.prepare_static_recursor_residual(base, &invocation)?;
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
                let prepared_worker =
                    matches!(&prepared, PreparedStaticRecursorResidual::Worker { .. });
                let base = self.materialize_static_recursor_residual(builder, prepared)?;
                if prepared_worker
                    && matches!(
                        self.body_emission_authority,
                        BodyEmissionAuthority::FunctionizedUnits
                    )
                {
                    let worker = recursive_worker.expect("prepared worker retains static identity");
                    let continuation_specialized =
                        self.producer_call_has_continuation_specialization(
                            worker.body_origin,
                            call_origin,
                        )?;
                    let mut suspended = armed.suspended;
                    if continuation_specialized {
                        suspended.continuation =
                            Self::discard_source_prefix(suspended.continuation);
                        self.consume_out_of_line_continuation_splice()?;
                    } else {
                        suspended.continuation = self.install_recursor_invocation(
                            suspended.continuation,
                            activation,
                            invocation,
                            checked_ih_invocation,
                        )?;
                    }
                    let LoweringOperand::Carried(word) = base else {
                        unreachable!("a prepared worker materializes one carried environment")
                    };
                    let mut inputs = args;
                    self.append_static_recursor_worker_captures(
                        builder,
                        word,
                        worker,
                        &mut inputs,
                    )?;
                    let mut value = self.call_declared_recursive_position_unit(
                        builder,
                        worker.body_origin,
                        &inputs,
                    )?;
                    if continuation_specialized {
                        value = self.apply_recursive_continuation_specialization_if_planned(
                            builder,
                            worker,
                            call_origin,
                            value,
                        )?;
                    } else {
                        self.active_static_recursor_result = Some(worker);
                    }
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value,
                        continuation_result_origin: None,
                        control: suspended,
                    }));
                }
                // ⭐⭐ `AC-C4` — the carried residual on the source-machine
                // route. ⚠ This is the site where "installs the ALREADY-CHECKED
                // invocation segment" is literal: the refusal below runs
                // **before** `install_recursor_invocation`, which is exactly the
                // ordering control 5 measures.
                if let LoweringOperand::Carried(word) = base {
                    let mut suspended = armed.suspended;
                    if let Some(worker) = recursive_worker.filter(|_| {
                        matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::FunctionizedUnits
                        )
                    }) {
                        let continuation_specialized =
                            self.producer_call_has_continuation_specialization(
                                worker.body_origin,
                                call_origin,
                            )?;
                        if continuation_specialized {
                            suspended.continuation =
                                Self::discard_source_prefix(suspended.continuation);
                            self.consume_out_of_line_continuation_splice()?;
                        } else {
                            suspended.continuation = self.install_recursor_invocation(
                                suspended.continuation,
                                activation,
                                invocation,
                                checked_ih_invocation,
                            )?;
                        }
                        let mut inputs = args;
                        self.append_static_recursor_worker_captures(
                            builder,
                            word,
                            worker,
                            &mut inputs,
                        )?;
                        let mut value = self.call_declared_recursive_position_unit(
                            builder,
                            worker.body_origin,
                            &inputs,
                        )?;
                        if continuation_specialized {
                            value = self.apply_recursive_continuation_specialization_if_planned(
                                builder,
                                worker,
                                call_origin,
                                value,
                            )?;
                        } else {
                            self.active_static_recursor_result = Some(worker);
                        }
                        return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                            value,
                            continuation_result_origin: None,
                            control: suspended,
                        }));
                    }
                    suspended.continuation = self.install_recursor_invocation(
                        suspended.continuation,
                        activation,
                        invocation,
                        checked_ih_invocation,
                    )?;
                    Self::reject_carried_residual_arguments(args.len())?;
                    return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                        value: LoweringOperand::Carried(word),
                        continuation_result_origin: None,
                        control: suspended,
                    }));
                }
                let base = self.specialized_recursor_residual(
                    base,
                    recursor_parent,
                    sibling_position,
                    recursive_worker,
                )?;
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
                        continuation_result_origin: None,
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
                        let value =
                            self.call_declared_recursive_position_unit(builder, body, &call_env)?;
                        return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                            value,
                            continuation_result_origin: None,
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

    /// Close the three declaration-only alternatives planned for every
    /// RecursiveDescent `Call`.
    ///
    /// The planner cannot know which specialized callee the lowering will
    /// reach. Once it does, the selected declaration path consumes its exact
    /// edge(s) and this helper dispositions only the alternatives that cannot
    /// occur at that call site.
    fn disposition_recursive_declaration_call_alternatives(
        &self,
        call_origin: StaticOriginId,
        consumes_source_argument: bool,
        consumes_direct_argument: bool,
        consumes_capture_specialization: bool,
    ) -> Result<(), CraneliftBackendError> {
        for (edge, consumed) in [
            (
                LoweringOnlyOperandEdge::RecursiveSourceDeclarationArgument,
                consumes_source_argument,
            ),
            (
                LoweringOnlyOperandEdge::RecursiveDeclarationArgument,
                consumes_direct_argument,
            ),
            (
                LoweringOnlyOperandEdge::DeclarationCaptureSpecialization,
                consumes_capture_specialization,
            ),
        ] {
            if !consumed {
                self.disposition_lowering_boundary_use_if_planned(edge, call_origin, 0)?;
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn lower_source_declaration_call<'b>(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        call_origin: StaticOriginId,
        reference_origin: StaticOriginId,
        symbol: RuntimeSymbol,
        captures: Vec<LoweringOperand>,
        body: OwnedSourceOccurrence,
        args: Vec<LoweringOperand>,
        env: Vec<LoweringOperand>,
        control: SourceControl<'b>,
    ) -> Result<SourceCallOutcome<'b>, CraneliftBackendError> {
        let _checked_invocation = self.consume_checked_recursive_invocation_call(&symbol)?;
        if self.body_emission_authority == BodyEmissionAuthority::FunctionizedUnits {
            let mut inputs = args;
            inputs.extend(captures);
            let value = self.call_declared_declaration_unit(builder, reference_origin, &inputs)?;
            return Ok(SourceCallOutcome::Continue(SourceMachineState::Value {
                value,
                continuation_result_origin: None,
                control,
            }));
        }
        if !self.declaration_is_recursive(&symbol) {
            self.disposition_recursive_declaration_call_alternatives(
                call_origin,
                false,
                false,
                false,
            )?;
            let mut call_env = args;
            call_env.extend(captures);
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
        self.disposition_recursive_declaration_call_alternatives(call_origin, true, false, true)?;
        let args_edge = self.lowering_boundary_use_token(
            LoweringOnlyOperandEdge::RecursiveSourceDeclarationArgument,
            call_origin,
            0,
        )?;
        let args = specialized_env_at(&args, args_edge)?;
        let captures_edge = self.lowering_boundary_use_token(
            LoweringOnlyOperandEdge::DeclarationCaptureSpecialization,
            call_origin,
            0,
        )?;
        let captures = specialized_env_at(&captures, captures_edge)?;
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
                    continuation_result_origin: None,
                    control,
                }));
            }
            let mut values = Vec::new();
            append_recursive_argument_values(
                builder,
                &args,
                &mut values,
                &self.function_local.native_int_tags,
            )?;
            builder.ins().jump(
                active
                    .header
                    .expect("tail-recursive source declarations own a loop header"),
                &values.into_iter().map(Into::into).collect::<Vec<_>>(),
            );
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(SourceCallOutcome::Complete(LoweringOperand::Specialized(
                Lowered::RecursiveBackedge,
            )));
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
        boundary_edge: LoweringOnlyOperandEdge,
        boundary_position: u32,
        lowered: LoweringOperand,
        required_kind: Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let edge =
            self.reached_lowering_boundary_use_token(boundary_edge, origin, boundary_position)?;
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
                let exit_aggregate_origin = match &lowered {
                    Lowered::Constructor {
                        aggregate_origin: Some(origin),
                        constructor,
                        ..
                    } if constructor == &self.process_symbols.exit_success
                        || constructor == &self.process_symbols.exit_failure =>
                    {
                        Some(*origin)
                    }
                    _ => None,
                };
                let terminal_exit = self.process_object
                    && matches!(
                        &lowered,
                        Lowered::Constructor { constructor, .. }
                            if constructor == &self.process_symbols.exit_success
                                || constructor == &self.process_symbols.exit_failure
                    )
                    && (required_kind == Some(ScalarMergeKind::ExitCode)
                        || self
                            .function_local
                            .terminal_result_origins
                            .contains(&origin)
                        || exit_aggregate_origin.is_some_and(|origin| {
                            self.static_transition_plan
                                .is_terminal_exit_aggregate_origin(origin)
                        }));
                if terminal_exit {
                    let status = self.emit_process_exit_status(builder, lowered);
                    self.emit_carrier_immediate(builder, BoundaryTag::ImmediateExitStatus, status)
                } else {
                    self.transfer_into_carrier_on_planned_edge(builder, origin, &lowered, edge)
                }
            }
        }
    }

    /// Give one already-planned join exactly the lanes named by its D8 token.
    fn append_planned_join_params(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
    ) {
        self.function_local
            .materialized_join_blocks
            .entry(join_plan.origin)
            .or_default()
            .insert(merge);
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
        self.jump_planned_join_arm_on_edge(
            builder,
            merge,
            join_plan,
            origin,
            LoweringOnlyOperandEdge::JoinArm,
            0,
            lowered,
            merge_kind,
            join,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn jump_planned_join_arm_on_edge(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        merge: cranelift_codegen::ir::Block,
        join_plan: &JoinPlanToken,
        origin: StaticOriginId,
        boundary_edge: LoweringOnlyOperandEdge,
        boundary_position: u32,
        lowered: LoweringOperand,
        merge_kind: &mut Option<ScalarMergeKind>,
        join: &'static str,
    ) -> Result<(), CraneliftBackendError> {
        match join_plan.representation {
            JoinResultRepresentation::NativeScalarPair => {
                if matches!(lowered, LoweringOperand::Carried(_)) {
                    let planned = self.retained_body_occurrence(join_plan.origin)?;
                    return Err(backend_module(format!(
                        "{join} source join {:?} ({:?}) planned native scalar lanes but \
                         lowering produced a carried boundary word",
                        join_plan.origin, planned.expr
                    )));
                }
                let (value, kind) = self.merge_scalar_branch(builder, join_plan, lowered, join)?;
                Self::record_scalar_merge_kind(join, merge_kind, kind)?;
                builder
                    .ins()
                    .jump(merge, &[value.tag.into(), value.payload.into()]);
            }
            JoinResultRepresentation::CarrierWord => {
                let word = self.carried_join_arm(
                    builder,
                    origin,
                    boundary_edge,
                    boundary_position,
                    lowered,
                    None,
                    join,
                )?;
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
    #[track_caller]
    pub(super) fn transfer_constructor_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        constructor: &str,
        args: &[LoweringOperand],
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        self.transfer_constructor_operands_with_edge_mode(builder, origin, constructor, args, false)
    }

    pub(super) fn transfer_reached_constructor_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        constructor: &str,
        args: &[LoweringOperand],
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        self.transfer_constructor_operands_with_edge_mode(builder, origin, constructor, args, true)
    }

    #[track_caller]
    fn transfer_constructor_operands_with_edge_mode(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        constructor: &str,
        args: &[LoweringOperand],
        reborrow_source_edges: bool,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        if constructor == self.process_symbols.exit_failure {
            if let [LoweringOperand::Carried(code)] = args {
                let edge = if reborrow_source_edges {
                    self.reached_operand_edge_token(
                        origin,
                        0,
                        SourceOperandRole::ConstructArgument,
                    )?
                } else {
                    self.operand_edge_token(origin, 0, SourceOperandRole::ConstructArgument)?
                };
                if edge.disposition() != OperandEdgeDisposition::SemanticEliminator {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "transparent failure-result payload lost its semantic-eliminator \
                         disposition"
                            .to_string(),
                    )));
                }
                return self.transfer_carried_failure_exit_status(builder, *code);
            }
        }
        let representation =
            self.aggregate_representation_token(origin, BoundaryClass::Constructor, args.len())?;
        // Preflight the complete child graph before allocating either a child
        // environment or the parent constructor. An exact recursive-position
        // closure is the sole exception to whole-closure transfer: its planner
        // token proves the static worker and its captures are all ordinary.
        let mut prepared_children = Vec::with_capacity(args.len());
        for (position, argument) in args.iter().enumerate() {
            let child_origin = self
                .static_transition_plan
                .child_static_origin(origin, position)?;
            let edge = if reborrow_source_edges {
                self.reached_operand_edge_token(
                    origin,
                    position,
                    SourceOperandRole::ConstructArgument,
                )?
            } else {
                self.operand_edge_token(origin, position, SourceOperandRole::ConstructArgument)?
            };
            let prepared = match argument {
                LoweringOperand::Specialized(value @ Lowered::Closure { .. }) => {
                    if edge.disposition() == OperandEdgeDisposition::CallableCapture {
                        Some(
                            self.prepare_static_recursor_constructor_residual(
                                child_origin,
                                LoweringOperand::Specialized(value.clone()),
                            )?
                            .ok_or_else(|| {
                                backend(BackendFailure::PlannerInvariant(
                                    "callable-capture constructor edge has no exact static \
                                     recursor worker plan"
                                        .to_string(),
                                ))
                            })?,
                        )
                    } else {
                        value.boundary_transfer_admissibility(&edge)?;
                        None
                    }
                }
                LoweringOperand::Specialized(value) => {
                    if edge.disposition() == OperandEdgeDisposition::CallableCapture {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "callable-capture constructor edge does not carry a closure"
                                .to_string(),
                        )));
                    }
                    value.boundary_transfer_admissibility(&edge)?;
                    None
                }
                // A governed worker may already have crossed this exact edge
                // through `materialize_static_recursor_residual`. Its carried
                // form is the ordinary positional Record envelope, not a
                // callable capsule, so preserving it here neither reclassifies
                // the edge nor bypasses whole-Closure rejection.
                LoweringOperand::Carried(_) => None,
            };
            prepared_children.push((edge, prepared));
        }
        let mut children = Vec::with_capacity(args.len());
        for (position, (argument, (edge, prepared))) in
            args.iter().zip(prepared_children).enumerate()
        {
            let child_origin = self
                .static_transition_plan
                .child_static_origin(origin, position)?;
            let child = if let Some(prepared) = prepared {
                let LoweringOperand::Carried(child) =
                    self.materialize_static_recursor_residual(builder, prepared)?
                else {
                    unreachable!("a planned recursor residual materializes one environment")
                };
                child
            } else {
                match argument {
                    LoweringOperand::Carried(child) => *child,
                    LoweringOperand::Specialized(value) => {
                        self.transfer_into_carrier_on_edge(builder, child_origin, value, edge)?
                    }
                }
            };
            children.push(child);
        }
        let identity = self
            .static_transition_plan
            .constructor_symbol_identity(origin)?
            .tag_abi_word()?;
        let word = self.emit_carrier_alloc(
            builder,
            representation.tag(),
            representation.class(),
            args.len(),
        )?;
        self.emit_carrier_store_tag_id(builder, word, identity)?;
        for (position, child) in children.into_iter().enumerate() {
            self.emit_carrier_store_field(builder, word, position, child)?;
        }
        Ok(word)
    }

    /// Build one record directly in the boundary carrier when any field has
    /// already crossed a generated-unit edge.
    fn transfer_record_operands(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        fields: &[(String, LoweringOperand)],
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let representation =
            self.aggregate_representation_token(origin, BoundaryClass::Record, fields.len())?;
        let word = self.emit_carrier_alloc(
            builder,
            representation.tag(),
            representation.class(),
            fields.len(),
        )?;
        for (position, (_, field)) in fields.iter().enumerate() {
            let identity = self
                .static_transition_plan
                .record_field_identity(origin, position)?
                .name_abi_word()?;
            self.emit_carrier_store_name(builder, word, position, identity)?;
            let child_origin = self
                .static_transition_plan
                .child_static_origin(origin, position)?;
            let child = match field {
                LoweringOperand::Carried(child) => *child,
                LoweringOperand::Specialized(value) => {
                    let edge =
                        self.operand_edge_token(origin, position, SourceOperandRole::RecordField)?;
                    self.transfer_into_carrier_on_edge(builder, child_origin, value, edge)?
                }
            };
            self.emit_carrier_store_field(builder, word, position, child)?;
        }
        Ok(word)
    }

    /// Preserve the established process-exit mapping when the failure code
    /// crosses a unit edge before its enclosing constructor is lowered.
    ///
    /// A carried exact `Int` may be either immediate or persistent: crossing
    /// an emitted aggregate occurrence is allowed to preserve the semantic
    /// integer in the boundary region. Narrow both representations through
    /// the checked carrier view, then apply the same `0 -> 1`, `1..=255 ->
    /// self`, otherwise `-3` mapping used by `emit_process_exit_status`.
    fn transfer_carried_failure_exit_status(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        code: CarriedBoundaryWord,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let (value, representable) = self.narrow_carried_exact_int_u64(builder, code)?;
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
        let valid = builder.ins().band(representable, positive);
        let valid = builder.ins().band(valid, within_max);
        let nonzero = builder.ins().select(valid, value, malformed);
        let is_zero =
            builder
                .ins()
                .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, value, zero);
        let valid_zero = builder.ins().band(representable, is_zero);
        let status = builder.ins().select(valid_zero, one, nonzero);
        self.emit_carrier_immediate(builder, BoundaryTag::ImmediateExitStatus, status)
    }

    fn transfer_carried_exit_status(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: CarriedBoundaryWord,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let class = self.emit_carrier_class(builder, value)?;
        Self::require_i64(builder, class, BoundaryClass::Constructor as i64);
        let tag = self.emit_carrier_tag(builder, value)?;
        let field_count = self.emit_carrier_field_count(builder, value)?;
        let success = self
            .static_transition_plan
            .terminal_exit_constructor_identity(&self.process_symbols.exit_success)?
            .tag_abi_word()?;
        let failure = self
            .static_transition_plan
            .terminal_exit_constructor_identity(&self.process_symbols.exit_failure)?
            .tag_abi_word()?;
        let success = i64::try_from(success)
            .map_err(|_| unsupported("ExitCode", "Success identity exceeds the native ABI"))?;
        let failure = i64::try_from(failure)
            .map_err(|_| unsupported("ExitCode", "Failure identity exceeds the native ABI"))?;

        let success_block = builder.create_block();
        let non_success_block = builder.create_block();
        let failure_block = builder.create_block();
        let invalid_block = builder.create_block();
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);

        let is_success =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, tag, success);
        builder
            .ins()
            .brif(is_success, success_block, &[], non_success_block, &[]);

        builder.switch_to_block(success_block);
        Self::require_i64(builder, field_count, 0);
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().jump(merge, &[zero.into()]);

        builder.switch_to_block(non_success_block);
        let is_failure =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, tag, failure);
        builder
            .ins()
            .brif(is_failure, failure_block, &[], invalid_block, &[]);

        builder.switch_to_block(failure_block);
        Self::require_i64(builder, field_count, 1);
        let code = self.emit_carrier_field(builder, value, 0)?;
        let status = self.transfer_carried_failure_exit_status(builder, code)?;
        let status = self.emit_carrier_scalar(builder, status)?;
        builder.ins().jump(merge, &[status.into()]);

        builder.switch_to_block(invalid_block);
        let malformed = builder.ins().iconst(types::I64, -2);
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
    fn lower_carried_match_body(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        body: SourceOccurrence<'_>,
        env: &[LoweringOperand],
        producer_eliminators: Option<&[EliminatorFrame<'_>]>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        match producer_eliminators {
            Some(eliminators) => {
                self.lower_computational_producer_expr(builder, body, env, eliminators)
            }
            None => self.lower_expr(builder, body, env),
        }
    }

    fn specialized_source_env_at(
        &self,
        operands: &[LoweringOperand],
        parent: StaticOriginId,
        first_position: usize,
        role: SourceOperandRole,
    ) -> Result<Vec<Lowered>, CraneliftBackendError> {
        operands
            .iter()
            .enumerate()
            .map(|(offset, operand)| {
                let token =
                    self.reached_operand_edge_token(parent, first_position + offset, role)?;
                if token.disposition() == OperandEdgeDisposition::CallableCapture {
                    operand.callable_capture_ref_at(token).cloned()
                } else {
                    operand.specialized_ref_at(token).cloned()
                }
            })
            .collect()
    }

    fn retain_callable_capture(
        &self,
        parent: StaticOriginId,
        position: usize,
        operand: LoweringOperand,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let edge = self.operand_edge_token(parent, position, SourceOperandRole::LexicalCapture)?;
        if edge.disposition() != OperandEdgeDisposition::CallableCapture {
            return Err(backend(BackendFailure::PlannerInvariant(
                "lexical capture edge is not callable-capture".to_string(),
            )));
        }
        Ok(operand)
    }

    fn lower_static_callable_specialization_call(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        call_origin: StaticOriginId,
        callee: SourceOccurrence<'_>,
        args: &[RuntimeExpr],
        env: &[LoweringOperand],
        call: units::DeclaredStaticCallableCall,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        if call.plan.arguments().len() != args.len() {
            return Err(backend(BackendFailure::PlannerInvariant(
                "static callable call plan does not cover every parameter".to_string(),
            )));
        }
        let mut ordinary = Vec::new();
        let mut lifted = Vec::new();
        for (position, argument_plan) in call.plan.arguments().iter().enumerate() {
            if argument_plan.parameter_ordinal() as usize != position {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "static callable argument plan is not in parameter order".to_string(),
                )));
            }
            let argument = self.child_occurrence(call_origin, 1 + position, &args[position])?;
            if argument.static_origin != argument_plan.argument_origin() {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "static callable argument origin changed after planning".to_string(),
                )));
            }
            match argument_plan.kind() {
                EmittableStaticCallableArgumentKind::Ordinary => {
                    let edge = self.operand_edge_token(
                        call_origin,
                        1 + position,
                        SourceOperandRole::CallArgument,
                    )?;
                    if edge.disposition() != OperandEdgeDisposition::Forwarding {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "ordinary specialization argument is not forwarding".to_string(),
                        )));
                    }
                    ordinary.push(self.lower_expr(builder, argument, env)?);
                }
                EmittableStaticCallableArgumentKind::Erased => {
                    let edge = self.operand_edge_token(
                        call_origin,
                        1 + position,
                        SourceOperandRole::CallArgument,
                    )?;
                    if edge.disposition() != OperandEdgeDisposition::StaticCallableElimination {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "erased static callable argument lost its elimination disposition"
                                .to_string(),
                        )));
                    }
                }
                EmittableStaticCallableArgumentKind::Direct { closure_origin } => {
                    if closure_origin != argument.static_origin {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "static callable binding names the wrong closure occurrence"
                                .to_string(),
                        )));
                    }
                    let edge = self.operand_edge_token(
                        call_origin,
                        1 + position,
                        SourceOperandRole::CallArgument,
                    )?;
                    if edge.disposition() != OperandEdgeDisposition::StaticCallableElimination {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "static callable argument did not consume the sixth disposition"
                                .to_string(),
                        )));
                    }
                    let binding = argument_plan.binding().ok_or_else(|| {
                        backend(BackendFailure::PlannerInvariant(
                            "direct static callable argument has no recursive binding".to_string(),
                        ))
                    })?;
                    let callable = self.lower_expr(builder, argument, env)?;
                    lift_static_callable_binding(binding, &callable, &mut lifted)?;
                }
                EmittableStaticCallableArgumentKind::Forwarded {
                    body_origin,
                    declared_arity,
                } => {
                    let edge = self.operand_edge_token(
                        call_origin,
                        1 + position,
                        SourceOperandRole::CallArgument,
                    )?;
                    if edge.disposition() != OperandEdgeDisposition::StaticCallableElimination {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "forwarded callable argument lost its elimination disposition"
                                .to_string(),
                        )));
                    }
                    let binding = argument_plan.binding().ok_or_else(|| {
                        backend(BackendFailure::PlannerInvariant(
                            "forwarded static callable argument has no recursive binding"
                                .to_string(),
                        ))
                    })?;
                    if binding.body_origin() != body_origin
                        || binding.declared_arity() != declared_arity
                    {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "forwarded static callable identity changed after planning".to_string(),
                        )));
                    }
                    let callable = self.lower_expr(builder, argument, env)?;
                    lift_static_callable_binding(binding, &callable, &mut lifted)?;
                }
            }
        }
        let lowered_callee = self.lower_expr(builder, callee, env)?;
        let LoweringOperand::Specialized(Lowered::DeclarationClosure { captures, .. }) =
            lowered_callee
        else {
            return Err(backend(BackendFailure::PlannerInvariant(
                "static callable specialization target is not a declaration closure".to_string(),
            )));
        };
        let mut inputs = ordinary;
        inputs.extend(lifted);
        inputs.extend(captures);
        self.call_declared_unit_target(
            builder,
            call.target,
            &inputs,
            #[cfg(test)]
            None,
        )
    }

    fn lower_carried_match(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        scrutinee: CarriedBoundaryWord,
        continuation: CarriedMatchContinuation<'_>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let (cases, default, static_origin, env, producer_eliminators) = match continuation {
            CarriedMatchContinuation::Ordinary {
                cases,
                default,
                static_origin,
                env,
            } => (cases, default, static_origin, env, None),
            CarriedMatchContinuation::Producer {
                cases,
                default,
                static_origin,
                env,
                eliminators,
            } => (cases, default, static_origin, env, Some(eliminators)),
        };
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
                producer_eliminators,
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
            self.append_planned_join_params(builder, merge, &join_plan);
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
            producer_eliminators,
        )?;
        if self.seal_source_trap_branch(builder, &borrowed_result)? {
            // This runtime representation has no continuing predecessor.
        } else {
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm_on_edge(
                builder,
                merge,
                &join_plan,
                static_origin,
                LoweringOnlyOperandEdge::BorrowedMatchJoinArm,
                0,
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
            producer_eliminators,
        )?;
        if self.seal_source_trap_branch(builder, &represented_result)? {
            // This runtime representation has no continuing predecessor.
        } else {
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a merge despite a continuing predecessor".to_string(),
                )
            })?;
            self.jump_planned_join_arm_on_edge(
                builder,
                merge,
                &join_plan,
                static_origin,
                LoweringOnlyOperandEdge::BorrowedMatchJoinArm,
                1,
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
        producer_eliminators: Option<&[EliminatorFrame<'_>]>,
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
                self.append_planned_join_params(builder, merge, join_plan);
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
            for (body_block, (index, _case)) in [(ok_body, ok_case), (err_body, err_case)] {
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
            if !self.seal_source_trap_branch(builder, &defaulted)? {
                return Err(unsupported(
                    "Match",
                    "the carried Result match's closed default did not seal its branch",
                ));
            }

            for (block, (index, case)) in [(ok_body, ok_case), (err_body, err_case)] {
                builder.switch_to_block(block);
                let payload = CarriedBoundaryWord {
                    word: builder.block_params(block)[0],
                };
                let case_env = env_with_operands([LoweringOperand::Carried(payload)], env);
                let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                let body_origin = body.static_origin;
                let lowered =
                    self.lower_carried_match_body(builder, body, &case_env, producer_eliminators)?;
                if self.seal_source_trap_branch(builder, &lowered)? {
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
            producer_eliminators,
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
        producer_eliminators: Option<&[EliminatorFrame<'_>]>,
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
            self.append_planned_join_params(builder, merge, join_plan);
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
            let lowered =
                self.lower_carried_match_body(builder, body, &case_env, producer_eliminators)?;
            if self.seal_source_trap_branch(builder, &lowered)? {
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
        if !self.seal_source_trap_branch(builder, &defaulted)? {
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
    #[track_caller]
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

    /// Apply the exact caller-owned continuation selected by this producer
    /// occurrence before its value reaches an identity-erasing join.
    fn continuation_specialization_result_origin(
        &mut self,
        origin: StaticOriginId,
    ) -> Option<ContinuationSpecializationCallToken> {
        let reached = self
            .function_local
            .continuation_specialization_calls
            .get(&origin)
            .and_then(|call| call.plan.call_token());
        if reached.is_some() {
            self.function_local
                .reached_continuation_specialization_results
                .insert(origin);
        }
        reached
    }

    fn continuation_specialization_flattens_result(
        &self,
        token: ContinuationSpecializationCallToken,
    ) -> Result<bool, CraneliftBackendError> {
        let origin = token.producer_result_origin();
        let calls = self
            .function_local
            .continuation_specialization_calls
            .get(&origin)
            .ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "continuation result occurrence has no planned direct edges".to_string(),
                ))
            })?;
        if calls.plan.call_token() != Some(token) {
            return Err(backend(BackendFailure::PlannerInvariant(
                "continuation result token changed before branch lowering".to_string(),
            )));
        }
        Ok(calls.plan.producer_fields_flattened())
    }

    fn continuation_specialization_worker_for_result(
        &self,
        call_token: ContinuationSpecializationCallToken,
    ) -> Result<Option<StaticRecursorWorker>, CraneliftBackendError> {
        let producer_origin = call_token.producer_result_origin();
        let Some(calls) = self
            .function_local
            .continuation_specialization_calls
            .get(&producer_origin)
        else {
            return Ok(None);
        };
        if calls.plan.call_token() != Some(call_token) {
            return Err(backend(BackendFailure::PlannerInvariant(
                "continuation worker lookup received another branch token".to_string(),
            )));
        }
        let token = self
            .static_transition_plan
            .continuation_specialization_worker_token(calls.plan.id())?;
        let exact_worker = StaticRecursorWorker {
                boundary_identity: token.identity(),
                residual_id: token.id,
                parent_origin: token.parent_origin,
                producer_origin: token.producer_origin,
                sibling_position: usize::try_from(token.sibling_position).map_err(|_| {
                    backend_module(
                        "continuation specialization worker position exceeds usize".to_string(),
                    )
                })?,
                closure_origin: token.closure_origin,
                body_origin: token.body_origin,
                declared_arity: usize::try_from(token.declared_arity).map_err(|_| {
                    backend_module(
                        "continuation specialization worker arity exceeds usize".to_string(),
                    )
                })?,
                capture_count: usize::try_from(token.capture_count).map_err(|_| {
                    backend_module(
                        "continuation specialization capture count exceeds usize".to_string(),
                    )
                })?,
            };
        Ok(Some(exact_worker))
    }

    fn merge_continuation_result_origins(
        retained: Option<ContinuationSpecializationCallToken>,
        next: Option<ContinuationSpecializationCallToken>,
    ) -> Result<Option<ContinuationSpecializationCallToken>, CraneliftBackendError> {
        match (retained, next) {
            (None, next) | (next, None) => Ok(next),
            (Some(retained), Some(next)) if retained == next => Ok(Some(retained)),
            (Some(_), Some(_)) => Err(backend(BackendFailure::PlannerInvariant(
                "one aggregate carries two continuation-specialization producer alternatives"
                    .to_string(),
            ))),
        }
    }

    fn continuation_inputs_for_call(
        &self,
        call: &units::DeclaredContinuationSpecializationCall,
    ) -> Result<Vec<LoweringOperand>, CraneliftBackendError> {
        let projection = call.plan.continuation_inputs();
        let capture_count = usize::try_from(call.plan.input_capture_count()).map_err(|_| {
            backend_module("continuation specialization capture count exceeds usize".to_string())
        })?;
        if capture_count != projection.len()
            || u32::try_from(projection.len()).ok()
                != Some(call.plan.continuation_capture_count())
        {
            return Err(backend(BackendFailure::PlannerInvariant(
                "continuation call ABI is not the exact immutable input projection".to_string(),
            )));
        }
        let planned_input_start =
            usize::try_from(call.plan.continuation_input_start()).map_err(|_| {
                backend_module(
                    "continuation specialization input offset exceeds usize".to_string(),
                )
            })?;
        let local_environment = call
            .plan
            .continuation_environment_is_local()
            .then(|| {
                self.function_local
                    .continuation_specialization_environments
                    .get(&call.plan.continuation_origin())
            })
            .flatten();
        let environment =
            local_environment.unwrap_or(&self.function_local.active_unit_inputs);
        let input_start = if local_environment.is_some() {
            environment.len().checked_sub(projection.len()).ok_or_else(|| {
                backend_module(
                    "local continuation environment is shorter than its exact projection"
                        .to_string(),
                )
            })?
        } else {
            planned_input_start
        };
        let emitted_owner = self.emitted_owner()?;
        projection
            .iter()
            .enumerate()
            .map(|(ordinal, input)| {
                let ordinal = u32::try_from(ordinal).map_err(|_| {
                    backend_module("continuation projection ordinal exceeds u32".to_string())
                })?;
                let abi_position = call
                    .plan
                    .ordinary_parameter_count()
                    .checked_add(ordinal)
                    .ok_or_else(|| {
                        backend_module(
                            "continuation projection ABI position exhausted".to_string(),
                        )
                    })?;
                let target_slot = call
                    .target
                    .slots
                    .get(usize::try_from(abi_position).map_err(|_| {
                        backend_module(
                            "continuation projection ABI position exceeds usize".to_string(),
                        )
                    })?)
                    .ok_or_else(|| {
                        backend_module(
                            "continuation projection is outside the target ABI".to_string(),
                        )
                    })?;
                if input.producer_owner() != emitted_owner
                    || input.consumer_owner() != call.plan.consumer_owner()
                    || input.ordinal() != ordinal
                    || input.ordinary_abi_position() != abi_position
                    || target_slot.kind != AbiSlotKind::Capture
                    || target_slot.ordinal != ordinal
                    || target_slot.carrier != input.carrier()
                    || target_slot.ownership != input.ownership()
                    || target_slot.storage_owner != input.storage_owner()
                    || input.referent_affinity().is_empty()
                {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "continuation projection contract changed before lowering".to_string(),
                    )));
                }
                let source_position = input_start.checked_add(ordinal as usize).ok_or_else(|| {
                    backend_module("continuation source position exhausted".to_string())
                })?;
                environment.get(source_position).cloned().ok_or_else(|| {
                    backend_module(format!(
                        "continuation projection omits source owner {:?} slot {}",
                        input.source_owner(),
                        input.source_abi_position(),
                    ))
                })
            })
            .collect()
    }

    #[track_caller]
    fn call_continuation_specialization_if_planned(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        call_token: ContinuationSpecializationCallToken,
        mut value: LoweringOperand,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let producer_origin = call_token.producer_result_origin();
        let Some(calls) = self
            .function_local
            .continuation_specialization_calls
            .get(&producer_origin)
            .cloned()
        else {
            return Ok(value);
        };
        let call = calls;
            if call.plan.call_token() != Some(call_token) {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation call received another branch token".to_string(),
                )));
            }
            if call.plan.producer_result_origin() != producer_origin {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation specialization moved to another producer occurrence".to_string(),
                )));
            }
            self.disposition_out_of_line_continuation_subtree(
                call.plan.continuation_origin(),
            )?;
            let identity = (producer_origin, call.target.call_site_sequence);
            self.function_local
                .dispositioned_continuation_specialization_calls
                .remove(&identity);
            if !self
                .function_local
                .consumed_continuation_specialization_calls
                .insert(identity)
            {
                return Err(backend(BackendFailure::PlannerInvariant(format!(
                    "one causal continuation-specialization edge was emitted twice: \
                     {identity:?}"
                ))));
            }
            let continuation = self.continuation_inputs_for_call(&call)?;
            let ordinary_count =
                usize::try_from(call.plan.ordinary_parameter_count()).map_err(|_| {
                    backend_module(
                        "continuation specialization ordinary input count exceeds usize"
                            .to_string(),
                    )
                })?;
            let mut inputs =
                Vec::with_capacity(ordinary_count + continuation.len());
            if call.plan.producer_fields_flattened() {
                let LoweringOperand::Carried(producer) = value else {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "flattened continuation result is not carried".to_string(),
                    )));
                };
                let field_count = self.emit_carrier_field_count(builder, producer)?;
                Self::require_i64(
                    builder,
                    field_count,
                    i64::try_from(ordinary_count).map_err(|_| {
                        backend_module(
                            "continuation specialization field count exceeds i64"
                                .to_string(),
                        )
                    })?,
                );
                for position in 0..ordinary_count {
                    inputs.push(LoweringOperand::Carried(
                        self.emit_carrier_field(builder, producer, position)?,
                    ));
                }
            } else {
                if ordinary_count != 1 {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "carried continuation result has a non-scalar ordinary ABI"
                            .to_string(),
                    )));
                }
                inputs.push(value);
            }
            inputs.extend(continuation);
            value = self.call_declared_unit_target(
                builder,
                call.target,
                &inputs,
                #[cfg(test)]
                None,
            )?;
        Ok(value)
    }

    fn apply_recursive_continuation_specialization_if_planned(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        worker: StaticRecursorWorker,
        invocation_origin: StaticOriginId,
        value: LoweringOperand,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let owner = self.active_emission_owner.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "recursive continuation fold has no emitted owner".to_string(),
            ))
        })?;
        let Some(producer_origin) = self
            .static_transition_plan
            .recursive_continuation_specialization_result_origin(
                owner,
                worker.body_origin,
                invocation_origin,
            )?
        else {
            return Ok(value);
        };
        self.function_local
            .reached_continuation_specialization_results
            .insert(producer_origin);
        let call_token = self
            .continuation_specialization_result_origin(producer_origin)
            .ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "recursive continuation fold has no exact branch token".to_string(),
                ))
            })?;
        self.call_continuation_specialization_if_planned(builder, call_token, value)
    }

    /// Transfer one exact, statically known producer constructor to its
    /// out-of-line return hole without first allocating an identity-free
    /// wrapper carrier. The constructor's ordered fields are the unit's
    /// planner-declared ordinary parameters.
    #[track_caller]
    fn call_known_constructor_continuation_specialization(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        call_token: ContinuationSpecializationCallToken,
        producer_construct_origin: StaticOriginId,
        operands: Vec<LoweringOperand>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let producer_origin = call_token.producer_result_origin();
        let call = self
            .function_local
            .continuation_specialization_calls
            .get(&producer_origin)
            .cloned()
            .ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "known continuation result has no planned direct edge".to_string(),
                ))
            })?;
        let planned_call_token = call.plan.call_token().ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "known continuation result has no exact branch call token".to_string(),
            ))
        })?;
        if planned_call_token != call_token
            || call_token.producer_owner() != self.emitted_owner()?
            || call_token.producer_construct_origin() != producer_construct_origin
            || call_token.call_site_sequence() != call.target.call_site_sequence
            || call_token.target() != call.plan.id()
            || call_token.worker_body_origin() != call.plan.worker_body_origin()
        {
            return Err(backend(BackendFailure::PlannerInvariant(
                "known continuation branch disagrees with its exact call token".to_string(),
            )));
        }
        if call.plan.producer_result_origin() != producer_origin
            || call.plan.producer_construct_origin() != producer_construct_origin
            || !call.plan.producer_fields_flattened()
        {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "known continuation result disagrees with its planned ordinary ABI: \
                 result={producer_origin:?}, construct={producer_construct_origin:?}, \
                 planned_construct={:?}, planned_fields={}, actual_fields={}",
                call.plan.producer_construct_origin(),
                call.plan.ordinary_parameter_count(),
                operands.len(),
            ))));
        }
        self.disposition_out_of_line_continuation_subtree(
            call.plan.continuation_origin(),
        )?;
        let worker = self
            .static_transition_plan
            .continuation_specialization_worker_token(call.plan.id())?;
        let worker_position = usize::try_from(call.plan.worker_position()).map_err(|_| {
            backend_module(
                "continuation specialization worker position exceeds usize".to_string(),
            )
        })?;
        let mut flattened = Vec::new();
        for (position, operand) in operands.iter().cloned().enumerate() {
            if position != worker_position {
                flattened.push(operand);
                continue;
            }
            let (captures, params, body) = match operand {
                LoweringOperand::Specialized(Lowered::Closure {
                    captures,
                    params,
                    body,
                }) => {
                    self.static_transition_plan
                        .disposition_static_recursor_worker_environment_if_planned(
                            worker.identity(),
                            worker.id,
                            self.emitted_owner()?,
                        );
                    (captures, params, body)
                }
                LoweringOperand::Specialized(Lowered::ComputationalRecursorClosure {
                    residual,
                    activation,
                    invocation,
                }) => {
                    let invocation_owned_worker = invocation.recursive_worker.is_some();
                    let invocation_worker = StaticRecursorWorker {
                        boundary_identity: worker.identity(),
                        residual_id: worker.id,
                        parent_origin: worker.parent_origin,
                        producer_origin: worker.producer_origin,
                        sibling_position: worker.sibling_position as usize,
                        closure_origin: worker.closure_origin,
                        body_origin: worker.body_origin,
                        declared_arity: worker.declared_arity as usize,
                        capture_count: worker.capture_count as usize,
                    };
                    if invocation
                        .recursive_worker
                        .is_some_and(|observed| observed != invocation_worker)
                    {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "continuation specialization recursor field disagrees with its exact \
                             worker provenance"
                                .to_string(),
                        )));
                    }
                    let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
                    let installed = compose_oriented_subcontinuation(
                        self.oriented_subcontinuation_plan.as_ref(),
                        None,
                        activation,
                        invocation,
                        dynamic_splice_edges,
                    )?;
                    if !installed.checked
                        || (invocation_owned_worker
                            && !installed.semantic_frames.iter().any(|frame| {
                                frame.static_origin == invocation_worker.parent_origin
                            }))
                    {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "continuation specialization recursor does not contain its checked \
                             producer continuation"
                                .to_string(),
                        )));
                    }
                    match *residual {
                        LoweringOperand::Specialized(Lowered::Closure {
                            captures,
                            params,
                            body,
                        }) => {
                            self.static_transition_plan
                                .disposition_static_recursor_worker_environment_if_planned(
                                    invocation_worker.boundary_identity,
                                    invocation_worker.residual_id,
                                    self.emitted_owner()?,
                                );
                            (captures, params, body)
                        }
                        LoweringOperand::Carried(environment) => {
                            let class = self.emit_carrier_class(builder, environment)?;
                            Self::require_i64(builder, class, BoundaryClass::Record as i64);
                            let field_count =
                                self.emit_carrier_field_count(builder, environment)?;
                            Self::require_i64(
                                builder,
                                field_count,
                                i64::try_from(invocation_worker.capture_count).map_err(|_| {
                                    backend_module(
                                        "continuation specialization capture count exceeds i64"
                                            .to_string(),
                                    )
                                })?,
                            );
                            let captures = (0..invocation_worker.capture_count)
                                .map(|position| {
                                    self.emit_carrier_field(builder, environment, position)
                                        .map(LoweringOperand::Carried)
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            (
                                captures,
                                vec![String::new(); invocation_worker.declared_arity],
                                invocation_worker.body_origin,
                            )
                        }
                        LoweringOperand::Specialized(_) => {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "continuation specialization recursor has no closure residual"
                                    .to_string(),
                            )));
                        }
                    }
                }
                LoweringOperand::Specialized(_) | LoweringOperand::Carried(_) => {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "continuation specialization worker field is not a closure".to_string(),
                    )));
                }
            };
            if body != worker.body_origin
                || params.len() != worker.declared_arity as usize
                || captures.len() != worker.capture_count as usize
            {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation specialization worker field disagrees with its plan"
                        .to_string(),
                )));
            }
            for (ordinal, capture) in captures.into_iter().enumerate() {
                let token = self.static_transition_plan.static_recursor_capture_token(
                    worker.identity(),
                    worker.id,
                    worker.parent_origin,
                    worker.producer_origin,
                    worker.sibling_position as usize,
                    worker.closure_origin,
                    ordinal,
                )?;
                if token.ordinal as usize != ordinal
                    || token.phase != OperandEdgeDisposition::CallableCapture
                    || token.lifetime != StaticRecursorCaptureLifetime::ActivationOwned
                {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "continuation specialization capture contract changed after planning"
                            .to_string(),
                    )));
                }
                flattened.push(match capture {
                    LoweringOperand::Carried(capture) => {
                        LoweringOperand::Carried(capture)
                    }
                    LoweringOperand::Specialized(value) => {
                        LoweringOperand::Carried(self.emit_carrier_transfer(
                            builder,
                            token.source_origin,
                            &value,
                        )?)
                    }
                });
            }
        }
        if usize::try_from(call.plan.ordinary_parameter_count()).ok()
            != Some(flattened.len())
        {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "known continuation result flattened to the wrong ordinary ABI: \
                 planned_fields={}, actual_fields={}",
                call.plan.ordinary_parameter_count(),
                flattened.len(),
            ))));
        }
        for position in 0..operands.len() {
            self.disposition_unreached_operand_edge(
                producer_construct_origin,
                position,
                SourceOperandRole::ConstructArgument,
            )?;
        }
        if call.target.call_site_sequence == 0
            && self
                .static_transition_plan
                .continuation_specialization_for_function(
                    self.emitted_owner()?,
                )
                .is_some()
        {
            self.static_transition_plan
                .disposition_aggregate_representation_for_owner(
                    self.emitted_owner()?,
                    producer_construct_origin,
                )?;
        }
        let identity = (producer_origin, call.target.call_site_sequence);
        self.function_local
            .dispositioned_continuation_specialization_calls
            .remove(&identity);
        if !self
            .function_local
            .consumed_continuation_specialization_calls
            .insert(identity)
        {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "one causal continuation-specialization edge was emitted twice: {identity:?}"
            ))));
        }
        let continuation = self.continuation_inputs_for_call(&call)?;
        let mut inputs = Vec::with_capacity(flattened.len() + continuation.len());
        inputs.extend(flattened);
        inputs.extend(continuation);
        self.call_declared_unit_target(
            builder,
            call.target.clone(),
            &inputs,
            #[cfg(test)]
            None,
        )
    }

    fn continuation_specialization_exits_emitted_owner(
        &self,
        call_token: ContinuationSpecializationCallToken,
    ) -> Result<bool, CraneliftBackendError> {
        let producer_origin = call_token.producer_result_origin();
        let owner = self.active_emission_owner.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "continuation specialization has no emitted producer owner".to_string(),
            ))
        })?;
        let calls = self
            .function_local
            .continuation_specialization_calls
            .get(&producer_origin)
            .ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "continuation result occurrence has no planned direct edges".to_string(),
                ))
            })?;
        let active_specialization = self
            .static_transition_plan
            .continuation_specialization_for_function(owner);
        let call = calls;
                if call.plan.call_token() != Some(call_token) {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "continuation exit query received another branch token".to_string(),
                    )));
                }
                let Some(active_specialization) = active_specialization else {
                    return self
                        .static_transition_plan
                        .continuation_specialization_transitively_exits_owner(
                            call.plan.id(),
                            owner,
                        );
                };
                let active = self
                    .static_transition_plan
                    .emittable_continuation_specialization(
                        active_specialization,
                        owner,
                        producer_origin,
                    )?;
                let active_origin = active.continuation_origin();
                let target_origin = call.plan.continuation_origin();
                if active_origin == target_origin {
                    // A recursive fold at the same return hole already
                    // includes this generated unit's result. Resuming the
                    // caller-side suffix here would execute it once per
                    // recursive layer.
                    return Ok(true);
                }
                if self
                    .static_transition_plan
                    .source_origin_is_in_subtree(target_origin, active_origin)?
                {
                    // A strict outer return hole is lowered by another
                    // out-of-line unit, but the continuation after that hole
                    // remains owned by this call site's active source
                    // continuation. Resume it exactly once after the outer
                    // unit returns.
                    return Ok(false);
                }
                if self
                    .static_transition_plan
                    .source_origin_is_in_subtree(active_origin, target_origin)?
                {
                    // A strict inner return hole is merely one operation in
                    // this generated unit's source body. Resume the remaining
                    // caller-owned suffix after the out-of-line call returns.
                    return Ok(false);
                }
                return Err(backend(BackendFailure::PlannerInvariant(
                    "generated continuation call names an unrelated return hole".to_string(),
                )));
    }

    fn bypass_continuation_specializations_for_result<'b>(
        &mut self,
        call_token: ContinuationSpecializationCallToken,
        mut continuation: SourceContinuation<'b>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        let producer_origin = call_token.producer_result_origin();
        let owner = self.active_emission_owner.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "continuation result occurrence has no emitted owner".to_string(),
            ))
        })?;
        let suffixes = self
            .static_transition_plan
            .continuation_specialized_suffix_origins(owner, producer_origin)?;
        loop {
            let static_origin = match &continuation {
                SourceContinuation::ComputationalMatchScrutinee {
                    static_origin, ..
                } => *static_origin,
                _ => break,
            };
            if !suffixes.contains(&static_origin) {
                break;
            }
            self.disposition_out_of_line_continuation_subtree(static_origin)?;
            let SourceContinuation::ComputationalMatchScrutinee { next, .. } = continuation else {
                unreachable!("the continuation variant was checked above")
            };
            continuation = *next;
        }
        Ok(continuation)
    }

    fn bypass_active_continuation_specializations_for_result<'b>(
        &mut self,
        call_token: ContinuationSpecializationCallToken,
        mut active: ActiveContinuationFrame<'b>,
    ) -> Result<ActiveContinuationFrame<'b>, CraneliftBackendError> {
        let producer_origin = call_token.producer_result_origin();
        let owner = self.active_emission_owner.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "continuation result occurrence has no emitted owner".to_string(),
            ))
        })?;
        let suffixes = self
            .static_transition_plan
            .continuation_specialized_suffix_origins(owner, producer_origin)?;
        loop {
            if active.pending.is_empty() {
                let Some(parent) = active.parent else {
                    break;
                };
                active = *parent;
                continue;
            }
            let Some((frame, tail)) = active.pending.split_first() else {
                unreachable!("empty active continuation was handled above")
            };
            let EliminatorFrame::Computational(frame) = frame else {
                break;
            };
            let contained = suffixes.iter().try_fold(false, |contained, root| {
                Ok::<_, CraneliftBackendError>(
                    contained
                        || self
                            .static_transition_plan
                            .source_origin_is_in_subtree(*root, frame.static_origin)?,
                )
            })?;
            if !contained {
                break;
            }
            self.disposition_out_of_line_continuation_subtree(frame.static_origin)?;
            active.pending = tail;
        }
        Ok(active)
    }

    fn producer_call_has_continuation_specialization(
        &self,
        body_origin: StaticOriginId,
        invocation_origin: StaticOriginId,
    ) -> Result<bool, CraneliftBackendError> {
        let Some(owner) = self.active_emission_owner else {
            return Ok(false);
        };
        let target = self
            .function_local
            .unit_calls
            .get(&(body_origin, 0))
            .ok_or_else(|| {
                backend_module(format!(
                    "retained body {body_origin:?} has no graph-derived call target in emitted \
                     unit {:?}",
                    self.active_emission_owner,
                ))
            })?;
        Ok(self
            .static_transition_plan
            .producer_call_has_continuation_specialization(owner, target.unit)
            && self
                .static_transition_plan
                .recursive_continuation_specialization_result_origin(
                    owner,
                    body_origin,
                    invocation_origin,
                )?
                .is_some())
    }

    fn consume_out_of_line_continuation_splice(&mut self) -> Result<(), CraneliftBackendError> {
        let owner = self.active_emission_owner.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "out-of-line continuation splice has no emitted owner".to_string(),
            ))
        })?;
        let frame = self
            .static_transition_plan
            .continuation_specialization_checked_frame(owner)
            .ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "out-of-line continuation splice has no checked frame".to_string(),
                ))
            })?;
        let edges = self
            .dynamic_splice_edges
            .iter()
            .filter_map(|(id, edge)| (edge.parent_frame_template_id == frame).then_some(*id))
            .collect::<Vec<_>>();
        let [edge] = edges.as_slice() else {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "out-of-line continuation splice is not exact for frame {frame}: {edges:?}"
            ))));
        };
        self.dynamic_splice_edges.remove(edge).ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "out-of-line continuation splice disappeared before consumption".to_string(),
            ))
        })?;
        Ok(())
    }

    fn bypass_continuation_specialized_suffixes<'b>(
        &self,
        producer_body_origin: StaticOriginId,
        mut continuation: SourceContinuation<'b>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        let consumer = self.active_emission_owner.ok_or_else(|| {
            backend(BackendFailure::PlannerInvariant(
                "continuation-specialized producer call has no emitted consumer".to_string(),
            ))
        })?;
        let producer = self
            .function_local
            .unit_calls
            .get(&(producer_body_origin, 0))
            .ok_or_else(|| {
                backend_module(
                    "continuation-specialized producer body has no declared unit target"
                        .to_string(),
                )
            })?
            .unit;
        loop {
            let continuation_origin = match &continuation {
                SourceContinuation::ComputationalMatchScrutinee { static_origin, .. } => {
                    *static_origin
                }
                _ => break,
            };
            if !self
                .static_transition_plan
                .producer_call_specializes_continuation(consumer, producer, continuation_origin)
            {
                break;
            }
            let SourceContinuation::ComputationalMatchScrutinee { next, .. } = continuation else {
                unreachable!("the continuation variant was checked above")
            };
            continuation = *next;
        }
        Ok(continuation)
    }

    pub(super) fn validate_continuation_specialization_calls(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        let expected = self
            .function_local
            .continuation_specialization_calls
            .iter()
            .map(|(origin, call)| (*origin, call.target.call_site_sequence))
            .collect::<BTreeSet<_>>();
        let actual = &self
            .function_local
            .consumed_continuation_specialization_calls;
        let dispositioned = &self
            .function_local
            .dispositioned_continuation_specialization_calls;
        if !actual.is_disjoint(dispositioned) {
            return Err(backend(BackendFailure::PlannerInvariant(
                "one causal continuation-specialization edge was both emitted and transported"
                    .to_string(),
            )));
        }
        let covered = actual
            .union(dispositioned)
            .copied()
            .collect::<BTreeSet<_>>();
        if covered != expected {
            return Err(backend(BackendFailure::PlannerInvariant(format!(
                "causal continuation-specialization emitted/dispositioned edge population is \
                 not exact; expected={expected:?}; emitted={actual:?}; \
                 dispositioned={dispositioned:?}",
            ))));
        }
        Ok(())
    }

    /// Close planned direct edges whose producer result occurrence was absent
    /// from this emitted owner's completed lowering traversal.
    ///
    /// Producer reachability is recorded independently at the source-machine
    /// result seam. Consequently this can close genuinely absent alternatives
    /// without making an omitted direct call appear dead: if the producer was
    /// reached, its unconsumed identity remains missing and validation reds.
    pub(super) fn disposition_unreached_continuation_specialization_calls(
        &mut self,
    ) -> Result<(), CraneliftBackendError> {
        let calls = self
            .function_local
            .continuation_specialization_calls
            .iter()
            .filter_map(|(origin, call)| {
                    let identity = (*origin, call.target.call_site_sequence);
                    (!self
                        .function_local
                        .reached_continuation_specialization_results
                        .contains(origin)
                        && !self
                            .function_local
                            .consumed_continuation_specialization_calls
                            .contains(&identity)
                        && !self
                            .function_local
                            .dispositioned_continuation_specialization_calls
                            .contains(&identity))
                    .then_some((identity, call.clone()))
            })
            .collect::<Vec<_>>();
        for (identity, call) in calls {
            let input_count = call
                .plan
                .ordinary_parameter_count()
                .checked_add(call.plan.input_capture_count())
                .ok_or_else(|| {
                    backend_module(
                        "continuation specialization input population exhausted".to_string(),
                    )
                })?;
            for position in 0..input_count {
                self.static_transition_plan
                    .disposition_lowering_boundary_use_for_emitted_occurrence(
                        LoweringOnlyOperandEdge::CallableCapsuleEscape,
                        call.target.call_site_origin,
                        position,
                        call.target.call_site_sequence,
                        self.emitted_owner()?,
                    )?;
            }
            self.function_local
                .dispositioned_continuation_specialization_calls
                .insert(identity);
        }
        Ok(())
    }

    /// Lower one planner-interned continuation/return hole. Its ordinary
    /// operands are the exact fields of one statically known producer
    /// constructor; the remaining operands are the caller environment. Worker
    /// and constructor identity stay compiler-only in the plan.
    #[track_caller]
    pub(super) fn lower_continuation_specialization(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        specialization: EmittableContinuationSpecialization,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let continuation_capture_count =
            usize::try_from(specialization.continuation_capture_count()).map_err(|_| {
                backend_module(
                    "continuation specialization capture count exceeds usize".to_string(),
                )
            })?;
        let input_capture_count =
            usize::try_from(specialization.input_capture_count()).map_err(|_| {
                backend_module(
                    "continuation specialization input capture count exceeds usize".to_string(),
                )
            })?;
        let ordinary_count =
            usize::try_from(specialization.ordinary_parameter_count()).map_err(|_| {
                backend_module(
                    "continuation specialization ordinary input count exceeds usize".to_string(),
                )
            })?;
        if env.len() != ordinary_count + input_capture_count {
            return Err(backend(BackendFailure::PlannerInvariant(
                "continuation specialization inputs disagree with its ABI".to_string(),
            )));
        }
        let occurrence = self.retained_body_occurrence(specialization.continuation_origin())?;
        self.function_local
            .continuation_specialization_environments
            .insert(occurrence.static_origin, env.to_vec());
        let RuntimeExpr::ComputationalMatch { cases, default, .. } = occurrence.expr else {
            return Err(backend(BackendFailure::PlannerInvariant(
                "continuation specialization names a non-computational continuation".to_string(),
            )));
        };
        self.enter_source_occurrence_plan(occurrence.static_origin)?;
        self.close_reached_operand_edge(
            occurrence.static_origin,
            0,
            SourceOperandRole::MatchScrutinee,
        )?;
        if let Some(frame_id) = specialization.checked_frame_id() {
            self.enter_checked_subcontinuation_frame(frame_id)?;
        }
        let checked_frame_id = self.consume_checked_subcontinuation_frame(cases, default)?;
        let frame = ComputationalEliminatorFrame {
            cases,
            default,
            env: &env[ordinary_count..ordinary_count + continuation_capture_count],
            static_origin: occurrence.static_origin,
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
            provenance: self.mint_recursor_frame_provenance(),
            checked_frame_id,
            checked_invocation_id: None,
            checked_invocation_source: None,
            checked_invocation_depth: 0,
        };
        let result = if specialization.producer_fields_flattened() {
            let producer =
                self.retained_body_occurrence(specialization.producer_construct_origin())?;
            let RuntimeExpr::Construct { constructor, args } = producer.expr else {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation specialization names a non-constructor producer result"
                        .to_string(),
                )));
            };
            let worker_position =
                usize::try_from(specialization.worker_position()).map_err(|_| {
                    backend_module(
                        "continuation specialization worker position exceeds usize"
                            .to_string(),
                    )
                })?;
            let worker_capture_count =
                usize::try_from(specialization.worker_capture_count()).map_err(|_| {
                    backend_module(
                        "continuation specialization worker capture count exceeds usize"
                            .to_string(),
                    )
                })?;
            let flattened_count = args
                .len()
                .checked_sub(1)
                .and_then(|count| count.checked_add(worker_capture_count))
                .ok_or_else(|| {
                    backend_module(
                        "continuation specialization producer shape exhausted".to_string(),
                    )
                })?;
            if worker_position >= args.len() || flattened_count != ordinary_count {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation specialization producer shape disagrees with its ABI"
                        .to_string(),
                )));
            }
            let token = self
                .static_transition_plan
                .continuation_specialization_worker_token(specialization.id())?;
            let worker = StaticRecursorWorker {
                boundary_identity: token.identity(),
                residual_id: token.id,
                parent_origin: token.parent_origin,
                producer_origin: token.producer_origin,
                sibling_position: usize::try_from(token.sibling_position).map_err(|_| {
                    backend_module(
                        "continuation specialization worker position exceeds usize".to_string(),
                    )
                })?,
                closure_origin: token.closure_origin,
                body_origin: token.body_origin,
                declared_arity: usize::try_from(token.declared_arity).map_err(|_| {
                    backend_module(
                        "continuation specialization worker arity exceeds usize".to_string(),
                    )
                })?,
                capture_count: usize::try_from(token.capture_count).map_err(|_| {
                    backend_module(
                        "continuation specialization capture count exceeds usize".to_string(),
                    )
                })?,
            };
            let previous = self.active_static_recursor_result.replace(worker);
            let mut flattened = env[..ordinary_count].iter().cloned();
            let mut producer_operands = Vec::with_capacity(args.len());
            for position in 0..args.len() {
                if position == worker_position {
                    let captures = flattened
                        .by_ref()
                        .take(worker_capture_count)
                        .collect::<Vec<_>>();
                    if captures.len() != worker_capture_count {
                        return Err(backend(BackendFailure::PlannerInvariant(
                            "continuation specialization omits a worker capture".to_string(),
                        )));
                    }
                    producer_operands.push(LoweringOperand::Specialized(
                        Lowered::Closure {
                            captures,
                            params: vec![
                                String::new();
                                usize::try_from(
                                    specialization.worker_declared_arity(),
                                )
                                .map_err(|_| {
                                    backend_module(
                                        "continuation specialization worker arity exceeds usize"
                                            .to_string(),
                                    )
                                })?
                            ],
                            body: specialization.worker_body_origin(),
                        },
                    ));
                } else {
                    producer_operands.push(flattened.next().ok_or_else(|| {
                        backend(BackendFailure::PlannerInvariant(
                            "continuation specialization omits a producer field".to_string(),
                        ))
                    })?);
                }
            }
            if flattened.next().is_some() {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation specialization has excess producer fields".to_string(),
                )));
            }
            let result = self.lower_known_constructor_operands_composed(
                builder,
                producer.static_origin,
                constructor,
                producer_operands,
                &[EliminatorFrame::Computational(frame)],
            );
            if self.active_static_recursor_result == Some(worker) {
                self.active_static_recursor_result = previous;
            }
            result
        } else {
            let [LoweringOperand::Carried(scrutinee)] = &env[..ordinary_count] else {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "continuation specialization received a non-carried producer result"
                        .to_string(),
                )));
            };
            self.lower_carried_computational_match(builder, *scrutinee, frame, &[])
        }?;
        match self.continuation_specialization_result_origin(
            specialization.continuation_origin(),
        ) {
            Some(call_token) => {
                self.call_continuation_specialization_if_planned(builder, call_token, result)
            }
            None => Ok(result),
        }
    }

    fn lower_static_recursor_worker_result_composed(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: LoweringOperand,
        frames: &[EliminatorFrame<'_>],
        worker: StaticRecursorWorker,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let previous = self.active_static_recursor_result.replace(worker);
        let result = self.lower_computational_match_value_composed(builder, value, frames);
        // A nested worker call advances the direct static continuation. Do not
        // erase that advance while unwinding the compiler's composed
        // continuation stack; restore only when this worker produced no nested
        // successor.
        if self.active_static_recursor_result == Some(worker) {
            self.active_static_recursor_result = previous;
        }
        result
    }

    fn append_static_recursor_worker_captures(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        environment: CarriedBoundaryWord,
        worker: StaticRecursorWorker,
        inputs: &mut Vec<LoweringOperand>,
    ) -> Result<(), CraneliftBackendError> {
        if inputs.len() != worker.declared_arity {
            return Err(unsupported(
                "StaticRecursorWorker",
                "the direct worker invocation disagrees with its declared arity",
            ));
        }
        let class = self.emit_carrier_class(builder, environment)?;
        Self::require_i64(builder, class, BoundaryClass::Record as i64);
        let field_count = self.emit_carrier_field_count(builder, environment)?;
        let expected = i64::try_from(worker.capture_count).map_err(|_| {
            unsupported(
                "StaticRecursorWorker",
                "the static worker environment exceeds the carrier ABI",
            )
        })?;
        Self::require_i64(builder, field_count, expected);
        for position in 0..worker.capture_count {
            inputs.push(LoweringOperand::Carried(self.emit_carrier_field(
                builder,
                environment,
                position,
            )?));
        }
        Ok(())
    }

    fn selected_static_recursor_worker(
        &self,
        parent_origin: StaticOriginId,
        position: usize,
    ) -> Result<Option<StaticRecursorWorker>, CraneliftBackendError> {
        if let Some(owner) = self.active_emission_owner {
            if self
                .static_transition_plan
                .continuation_specialization_for_function(owner)
                .is_none()
                && self
                    .static_transition_plan
                    .continuation_position_is_specialized_for_consumer(
                        owner,
                        parent_origin,
                        position,
                    )?
            {
                return Ok(None);
            }
        }
        let continuation_specialization = self.active_emission_owner.and_then(|owner| {
            self.static_transition_plan
                .continuation_specialization_for_function(owner)
        });
        let token = if let Some(specialization) = continuation_specialization {
            Some(
                self.static_transition_plan
                    .continuation_specialization_worker_token(specialization)?,
            )
        } else if matches!(
            self.body_emission_authority,
            BodyEmissionAuthority::RecursiveDescent
        ) {
            self.static_transition_plan
                .reached_static_recursor_worker_residual_token(parent_origin, position)?
        } else {
            self.active_emission_owner.ok_or_else(|| {
                backend(BackendFailure::PlannerInvariant(
                    "functionized static recursor selection has no active emission owner"
                        .to_string(),
                ))
            })?;
            self.static_transition_plan
                .selected_static_recursor_worker_residual_token(
                    parent_origin,
                    position,
                    self.active_static_recursor_result
                        .or(self.active_static_recursor_selection)
                        .map(|worker| worker.body_origin),
                )?
        };
        token
            .map(|token| {
                if token.disposition() != OperandEdgeDisposition::CallableCapture {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "selected static recursor residual is not callable-capture".to_string(),
                    )));
                }
                Ok(StaticRecursorWorker {
                    boundary_identity: token.identity(),
                    residual_id: token.id,
                    parent_origin: token.parent_origin,
                    producer_origin: token.producer_origin,
                    sibling_position: token.sibling_position as usize,
                    closure_origin: token.closure_origin,
                    body_origin: token.body_origin,
                    declared_arity: token.declared_arity as usize,
                    capture_count: token.capture_count as usize,
                })
            })
            .transpose()
    }

    fn selected_static_recursor_worker_for_producer(
        &self,
        parent_origin: StaticOriginId,
        position: usize,
        producer_origin: StaticOriginId,
    ) -> Result<Option<StaticRecursorWorker>, CraneliftBackendError> {
        if let Some(worker) = self
            .active_static_recursor_result
            .or(self.active_static_recursor_selection)
        {
            if worker.parent_origin == parent_origin
                && worker.sibling_position == position
                && worker.producer_origin == producer_origin
            {
                return Ok(Some(worker));
            }
        }
        if matches!(
            self.body_emission_authority,
            BodyEmissionAuthority::RecursiveDescent
        ) {
            return self.selected_static_recursor_worker(parent_origin, position);
        }
        let token = self
            .static_transition_plan
            .selected_static_recursor_worker_residual_token_for_producer(
                parent_origin,
                position,
                producer_origin,
            )?;
        token
            .map(|token| {
                if token.disposition() != OperandEdgeDisposition::CallableCapture {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "selected static recursor residual is not callable-capture".to_string(),
                    )));
                }
                Ok(StaticRecursorWorker {
                    boundary_identity: token.identity(),
                    residual_id: token.id,
                    parent_origin: token.parent_origin,
                    producer_origin: token.producer_origin,
                    sibling_position: token.sibling_position as usize,
                    closure_origin: token.closure_origin,
                    body_origin: token.body_origin,
                    declared_arity: token.declared_arity as usize,
                    capture_count: token.capture_count as usize,
                })
            })
            .transpose()
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
    #[track_caller]
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
        let predecessor_body = self
            .active_static_recursor_result
            .map(|worker| worker.body_origin);
        if let Some((_, _, header)) = self
            .active_carried_computational_eliminations
            .iter()
            .rev()
            .find(|(origin, predecessor, _)| {
                *origin == eliminator.static_origin && *predecessor == predecessor_body
            })
        {
            builder.ins().jump(*header, &[scrutinee.word.into()]);
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        }

        let header = builder.create_block();
        builder.append_block_param(header, types::I64);
        builder.ins().jump(header, &[scrutinee.word.into()]);
        builder.switch_to_block(header);
        let scrutinee = CarriedBoundaryWord {
            word: builder.block_params(header)[0],
        };
        self.active_carried_computational_eliminations.push((
            eliminator.static_origin,
            predecessor_body,
            header,
        ));
        let lowered = self.lower_carried_computational_match_inner(
            builder,
            scrutinee,
            eliminator,
            remaining_eliminators,
        );
        let popped = self.active_carried_computational_eliminations.pop();
        debug_assert_eq!(
            popped,
            Some((eliminator.static_origin, predecessor_body, header)),
            "the carried elimination stack must unwind in the order it was pushed"
        );
        lowered
    }

    #[track_caller]
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
        let phase_constructors = if let Some(worker) = self.active_static_recursor_result {
            Some(self.static_transition_plan
                .source_result_constructor_identities_in_owner_subtree(worker.body_origin)?
            )
        } else if let Some(owner) = self.active_emission_owner {
            self.static_transition_plan
                .continuation_specialization_producer_constructor(owner)?
                .map(|identity| vec![identity])
        } else {
            None
        }
        .filter(|constructors: &Vec<_>| !constructors.is_empty());
        if phase_constructors.is_some() {
            self.disposition_statically_unselected_match_cases(eliminator.static_origin, None)?;
        }

        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);

        for (index, case) in eliminator.cases.iter().enumerate() {
            if let Some(constructors) = &phase_constructors {
                let identity = self
                    .static_transition_plan
                    .case_constructor_identity(eliminator.static_origin, index)?;
                if !constructors.iter().any(|candidate| candidate == &identity) {
                    continue;
                }
                self.disposition_statically_unselected_match_cases(
                    eliminator.static_origin,
                    Some(index),
                )?;
            }
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
                px8j_record_carrier_field_projection(Px8jProducerPath::Composed, position, child);
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
                    let recursive_worker =
                        self.selected_static_recursor_worker(eliminator.static_origin, position)?;
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
                        recursive_worker,
                    )?;
                    #[cfg(test)]
                    px8j_record_recursor_carrier(Px8jProducerPath::Composed, &induction_hypothesis);
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

            let body = self.case_body_occurrence(eliminator.static_origin, index, &case.body)?;
            let body_origin = body.static_origin;
            let continuation_source_machine = if active_scope.is_none() {
                if let Some(owner) = self.active_emission_owner {
                    self.static_transition_plan
                        .continuation_specialization_in_owner_subtree(owner, body.static_origin)?
                } else {
                    false
                }
            } else {
                false
            };
            let (lowered, continuation_result_origin) =
                if let Some((activation, cursor, producer_origin, splice_caller)) = active_scope {
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
                    self.lower_source_machine_with_result_origin(
                        builder,
                        body,
                        &case_env,
                        &active_state,
                    )?
                } else if continuation_source_machine {
                    // A non-recursive case can still contain one of the exact
                    // producer alternatives whose callable identity would be
                    // erased by this carried case join. Run that case through
                    // the owned source machine as well: its pending Construct
                    // frame retains the complete produced value while a
                    // branch-local Closure origin selects the planner-issued
                    // continuation unit.
                    let activation = self.mint_continuation_activation();
                    let cursor = self.mint_continuation_cursor();
                    let splice_caller = active_recursor_frame(remaining_eliminators);
                    let mut pending: Vec<_> = remaining_eliminators
                        .iter()
                        .copied()
                        .filter(|frame| !matches!(frame, EliminatorFrame::Active(_)))
                        .collect();
                    if let Some(active) = splice_caller {
                        pending.extend_from_slice(active.pending);
                    }
                    let selected_ancestry = splice_caller
                        .map(|active| active.selected_ancestry.to_vec())
                        .unwrap_or_default();
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
                        selected_scope: splice_caller.and_then(|active| active.selected_scope),
                    };
                    self.lower_source_machine_with_result_origin(
                        builder,
                        body,
                        &case_env,
                        &active_state,
                    )?
                } else if remaining_eliminators.is_empty() {
                    (self.lower_expr(builder, body, &case_env)?, None)
                } else {
                    (
                        self.lower_computational_producer_expr(
                            builder,
                            body,
                            &case_env,
                            remaining_eliminators,
                        )?,
                        None,
                    )
                };
            if !matches!(
                lowered,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            ) {
                let lowered = if let Some(producer_origin) = continuation_result_origin {
                    self.call_continuation_specialization_if_planned(
                        builder,
                        producer_origin,
                        lowered,
                    )?
                } else {
                    lowered
                };
                let word = self.carried_join_arm(
                    builder,
                    body_origin,
                    LoweringOnlyOperandEdge::JoinArm,
                    0,
                    lowered,
                    None,
                    "a carried `ComputationalMatch` arm",
                )?;
                builder.ins().jump(merge, &[word.word.into()]);
            }

            builder.switch_to_block(next);
        }

        if eliminator
            .cases
            .first()
            .is_some_and(|case| case.constructor.contains("::ResourceError::"))
        {
            if let Some(TrapExitAuthority::UnitFrame { slots, trap_offset }) =
                self.function_local.trap_exit
            {
                builder
                    .ins()
                    .store(MemFlags::trusted(), tag, slots, trap_offset);
                let returned = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[returned]);
                let unreachable = builder.create_block();
                builder.switch_to_block(unreachable);
            }
        }

        let defaulted = LoweringOperand::Specialized(Lowered::Trap(eliminator.default.clone()));
        if !self.seal_source_trap_branch(builder, &defaulted)? {
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
                self.close_reached_operand_edge(
                    static_origin,
                    0,
                    SourceOperandRole::WrapperBody,
                )?;
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
                self.close_reached_operand_edge(
                    static_origin,
                    0,
                    SourceOperandRole::WrapperBody,
                )?;
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
                self.close_reached_operand_edge(
                    static_origin,
                    0,
                    SourceOperandRole::WrapperBody,
                )?;
                let instance =
                    self.enter_checked_recursive_invocation(*call_template_id, body)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let result = self.lower_expr(builder, body, env);
                self.leave_checked_recursive_invocation(instance)?;
                result
            }
            RuntimeExpr::CheckedComputationalIHSlots { body, .. } => {
                self.close_reached_operand_edge(
                    static_origin,
                    0,
                    SourceOperandRole::WrapperBody,
                )?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                self.lower_expr(builder, body, env)
            }
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id,
                body,
                ..
            } => {
                self.close_reached_operand_edge(
                    static_origin,
                    0,
                    SourceOperandRole::WrapperBody,
                )?;
                self.enter_checked_computational_ih_invocation(*call_template_id)?;
                let body = self.child_occurrence(static_origin, 0, body)?;
                let value = self.lower_expr(builder, body, env)?;
                self.finish_checked_computational_ih_marker(static_origin, value)
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
                let _value_edge = self.operand_edge_token(
                    static_origin,
                    0,
                    SourceOperandRole::LetValue,
                )?;
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
                let _body_edge = self.operand_edge_token(
                    static_origin,
                    1,
                    SourceOperandRole::LetBody,
                )?;
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
                let _scrutinee_edge = self.operand_edge_token(
                    static_origin,
                    0,
                    SourceOperandRole::IfScrutinee,
                )?;
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
                    let unselected = if scrutinee { else_expr } else { then_expr };
                    let selected_position = if scrutinee { 1 } else { 2 };
                    let unselected_position = if scrutinee { 2 } else { 1 };
                    let _selected_edge = self.operand_edge_token(
                        static_origin,
                        selected_position,
                        SourceOperandRole::IfArm,
                    )?;
                    self.disposition_operand_edge(
                        static_origin,
                        unselected_position,
                        SourceOperandRole::IfArm,
                    )?;
                    self.disposition_lowering_boundary_use_if_planned(
                            LoweringOnlyOperandEdge::JoinArm,
                            static_origin,
                            0,
                        )?;
                    self.disposition_lowering_boundary_use_if_planned(
                            LoweringOnlyOperandEdge::JoinArm,
                            if scrutinee {
                                then_expr.static_origin
                            } else {
                                else_expr.static_origin
                            },
                            0,
                        )?;
                    self.disposition_statically_unselected_source_subtree(
                        unselected.static_origin,
                    )?;
                    return if scrutinee {
                        self.lower_expr(builder, then_expr, env)
                    } else {
                        self.lower_expr(builder, else_expr, env)
                    };
                }
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge = join_plan
                    .has_continuing_predecessor
                    .then(|| builder.create_block());
                if let Some(merge) = merge {
                    self.append_planned_join_params(builder, merge, &join_plan);
                }
                builder.ins().brif(value, then_block, &[], else_block, &[]);
                let mut merge_kind = None;
                let mut terminal_trap = None;
                for (block, arm) in [(then_block, then_expr), (else_block, else_expr)] {
                    builder.switch_to_block(block);
                    let position = if arm.static_origin == then_expr.static_origin {
                        1
                    } else {
                        2
                    };
                    let _arm_edge = self.operand_edge_token(
                        static_origin,
                        position,
                        SourceOperandRole::IfArm,
                    )?;
                    let lowered = self.lower_expr(builder, arm, env)?;
                    if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                        terminal_trap.get_or_insert_with(|| trap.clone());
                    }
                    if self.seal_source_trap_branch(builder, &lowered)? {
                        continue;
                    }
                    let merge = merge.ok_or_else(|| {
                        backend_module(
                            "join plan omitted an If merge despite a continuing predecessor"
                                .to_string(),
                        )
                    })?;
                    self.jump_planned_join_arm(
                        builder,
                        merge,
                        &join_plan,
                        arm.static_origin,
                        lowered,
                        &mut merge_kind,
                        "If",
                    )?;
                }
                let Some(merge) = merge else {
                    let unreachable = builder.create_block();
                    builder.switch_to_block(unreachable);
                    let trap = terminal_trap.ok_or_else(|| {
                        backend_module(
                            "If join omitted both a continuing predecessor and a source trap"
                                .to_string(),
                        )
                    })?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                };
                self.finish_planned_join(builder, merge, &join_plan, merge_kind, "If")
            }
            RuntimeExpr::Construct { constructor, args } => {
                let mut lower_construct = || -> Result<LoweringOperand, CraneliftBackendError> {
                let lowered_args = args
                    .iter()
                    .enumerate()
                    .map(|(position, arg)| {
                        let arg = self.child_occurrence(static_origin, position, arg)?;
                        self.lower_expr(builder, arg, env)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if let Some(call_token) =
                    self.continuation_specialization_result_origin(static_origin)
                {
                    if self.continuation_specialization_flattens_result(call_token)? {
                        return self.call_known_constructor_continuation_specialization(
                            builder,
                            call_token,
                            static_origin,
                            lowered_args,
                        );
                    }
                }
                if lowered_args
                    .iter()
                    .any(|arg| matches!(arg, LoweringOperand::Specialized(Lowered::RecursiveBackedge)))
                {
                    for position in 0..lowered_args.len() {
                        self.reached_operand_edge_token(
                            static_origin,
                            position,
                            SourceOperandRole::ConstructArgument,
                        )?;
                    }
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
                    aggregate_origin: Some(static_origin),
                    synthesized_identity: Some(ConstructorIdentityV1::Source(
                        self.static_transition_plan
                            .constructor_symbol_identity(static_origin)?,
                    )),
                    args: self.specialized_source_env_at(
                        &lowered_args,
                        static_origin,
                        0,
                        SourceOperandRole::ConstructArgument,
                    )?,
                }))
                };
                lower_construct()
            }
            RuntimeExpr::Match {
                scrutinee,
                cases,
                default,
            } => {
                // Keep the large multi-representation Match dispatcher in its
                // own stack frame. Deep generated-unit bodies alternate Match
                // and constructor layers; placing every arm's locals in
                // `lower_expr` itself exhausts the standard test-thread stack
                // before the source-machine continuation reaches its leaf.
                let mut lower_match = || -> Result<LoweringOperand, CraneliftBackendError> {
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
                let scrutinee_edge = self.operand_edge_token(
                    static_origin,
                    0,
                    SourceOperandRole::MatchScrutinee,
                )?;
                if scrutinee_edge.disposition() != OperandEdgeDisposition::SemanticEliminator {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "match scrutinee lost its semantic-eliminator disposition".to_string(),
                    )));
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
                        CarriedMatchContinuation::Ordinary {
                            cases,
                            default,
                            static_origin,
                            env,
                        },
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
                        None,
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
                        default,
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
                        self.disposition_statically_unselected_match_cases(
                            static_origin,
                            Some(index),
                        )?;
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        return self.lower_expr(builder, body, env);
                    }
                    let join_plan = self.consumed_join_plan_token(static_origin)?;
                    let true_block = builder.create_block();
                    let false_block = builder.create_block();
                    let merge = join_plan
                        .has_continuing_predecessor
                        .then(|| builder.create_block());
                    if let Some(merge) = merge {
                        self.append_planned_join_params(builder, merge, &join_plan);
                    }
                    builder
                        .ins()
                        .brif(value, true_block, &[], false_block, &[]);
                    let mut merge_kind = None;
                    let mut terminal_trap = None;
                    for (block, (index, case)) in
                        [(true_block, true_case), (false_block, false_case)]
                    {
                        builder.switch_to_block(block);
                        let body = self.case_body_occurrence(static_origin, index, &case.body)?;
                        let lowered = self.lower_expr(builder, body, env)?;
                        if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                            terminal_trap.get_or_insert_with(|| trap.clone());
                        }
                        if self.seal_source_trap_branch(builder, &lowered)? {
                            continue;
                        }
                        let merge = merge.ok_or_else(|| {
                            backend_module(
                                "join plan omitted a Bool Match merge despite a continuing \
                                 predecessor"
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
                        let unreachable = builder.create_block();
                        builder.switch_to_block(unreachable);
                        let trap = terminal_trap.ok_or_else(|| {
                            backend_module(
                                "Bool Match join omitted both a continuing predecessor and a \
                                 source trap"
                                    .to_string(),
                            )
                        })?;
                        return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
                    };
                    return self.finish_planned_join(
                        builder,
                        merge,
                        &join_plan,
                        merge_kind,
                        "Match",
                    );
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
                    self.disposition_statically_unselected_match_cases(
                        static_origin,
                        None,
                    )?;
                    return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
                };
                self.disposition_statically_unselected_match_cases(
                    static_origin,
                    Some(index),
                )?;
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
                };
                lower_match()
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
                if lowered_fields
                    .iter()
                    .any(|(_, value)| matches!(value, LoweringOperand::Carried(_)))
                {
                    return Ok(LoweringOperand::Carried(self.transfer_record_operands(
                        builder,
                        static_origin,
                        &lowered_fields,
                    )?));
                }
                let mut specialized_fields = Vec::with_capacity(lowered_fields.len());
                for (position, (name, value)) in lowered_fields.into_iter().enumerate() {
                    let token = self.operand_edge_token(
                        static_origin,
                        position,
                        SourceOperandRole::RecordField,
                    )?;
                    specialized_fields.push((name, value.specialized_at(token)?));
                }
                Ok(LoweringOperand::Specialized(Lowered::Record {
                    aggregate_origin: Some(static_origin),
                    fields: specialized_fields,
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
                        let Lowered::Record { fields, .. } = lowered else {
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
                    .map(|symbol| {
                        self.lower_seed_capture(builder, symbol)
                            .map(LoweringOperand::Specialized)
                    })
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
                        let lowered = self.lower_expr(builder, capture, env)?;
                        self.retain_callable_capture(static_origin, 1 + position, lowered)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(LoweringOperand::Specialized(Lowered::Closure {
                    captures,
                    params: params.clone(),
                    body: body.static_origin,
                }))
            }
            RuntimeExpr::DeclarationRef { symbol } => {
                self.lower_declaration_ref(builder, static_origin, symbol)
            }
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
                let mut lower_call = || -> Result<LoweringOperand, CraneliftBackendError> {
                let join_plan = self.consumed_join_plan_token(static_origin)?;
                if matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::RecursiveDescent
                ) {
                    // An inline call has no emitted call-result merge. Its
                    // result continues directly in the enclosing source
                    // traversal, so the native JoinArm alternative is absent.
                    self.disposition_lowering_boundary_use_if_planned(
                            LoweringOnlyOperandEdge::JoinArm,
                            static_origin,
                            0,
                        )?;
                }
                let callee = self.child_occurrence(static_origin, 0, callee)?;
                if matches!(
                    self.body_emission_authority,
                    BodyEmissionAuthority::FunctionizedUnits
                ) {
                    if let Some(call) = self
                        .function_local
                        .static_callable_calls
                        .get(&static_origin)
                        .cloned()
                    {
                        let edge = self.operand_edge_token(
                            static_origin,
                            0,
                            SourceOperandRole::CallCallee,
                        )?;
                        if edge.disposition() != OperandEdgeDisposition::SpecializedOnlyLeaf {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "static callable callee lost its semantic-inspection disposition"
                                    .to_string(),
                            )));
                        }
                        return self.lower_static_callable_specialization_call(
                            builder,
                            static_origin,
                            callee,
                            args,
                            env,
                            call,
                        );
                    }
                    if let RuntimeExpr::LexicalClosure {
                        captures,
                        params,
                        body,
                    } = callee.expr
                    {
                        let edge = self.operand_edge_token(
                            static_origin,
                            0,
                            SourceOperandRole::CallCallee,
                        )?;
                        if edge.disposition() != OperandEdgeDisposition::SpecializedOnlyLeaf {
                            return Err(backend(BackendFailure::PlannerInvariant(
                                "direct lexical callee lost its semantic-inspection disposition"
                                    .to_string(),
                            )));
                        }
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
                                let lowered = self.lower_expr(builder, argument, env)?;
                                let edge =
                                    self.reached_operand_edge_token(
                                        static_origin,
                                        1 + position,
                                        SourceOperandRole::CallArgument,
                                    )?;
                                if edge.disposition() != OperandEdgeDisposition::Forwarding {
                                    return Err(backend(BackendFailure::PlannerInvariant(
                                        "direct lexical argument lost its forwarding disposition"
                                            .to_string(),
                                    )));
                                }
                                Ok(lowered)
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
                let callee_edge = self.reached_operand_edge_token(
                    static_origin,
                    0,
                    SourceOperandRole::CallCallee,
                )?;
                if callee_edge.disposition() != OperandEdgeDisposition::SpecializedOnlyLeaf {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "callee lost its semantic-inspection disposition".to_string(),
                    )));
                }
                match lowered_callee {
                    LoweringOperand::Specialized(Lowered::DeclarationClosure {
                        reference_origin,
                        symbol,
                        captures,
                        params,
                        body,
                    }) => self.lower_recursive_declaration_call(
                        builder,
                        reference_origin,
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
                        self.disposition_recursive_declaration_call_alternatives(
                            static_origin,
                            false,
                            false,
                            false,
                        )?;
                        let mut call_env = args
                            .iter()
                            .enumerate()
                            .map(|(position, arg)| {
                                let arg =
                                    self.child_occurrence(static_origin, 1 + position, arg)?;
                                let lowered = self.lower_expr(builder, arg, env)?;
                                let edge =
                                    self.reached_operand_edge_token(
                                        static_origin,
                                        1 + position,
                                        SourceOperandRole::CallArgument,
                                    )?;
                                if edge.disposition() != OperandEdgeDisposition::Forwarding {
                                    return Err(backend(BackendFailure::PlannerInvariant(
                                        "closure call argument lost its forwarding disposition"
                                            .to_string(),
                                    )));
                                }
                                match self.body_emission_authority {
                                    BodyEmissionAuthority::RecursiveDescent => Ok(lowered),
                                    BodyEmissionAuthority::FunctionizedUnits => {
                                        Ok(match lowered {
                                            LoweringOperand::Carried(word) => {
                                                LoweringOperand::Carried(word)
                                            }
                                            LoweringOperand::Specialized(value) => {
                                                LoweringOperand::Carried(
                                                    self.transfer_into_carrier_on_edge(
                                                        builder,
                                                        arg.static_origin,
                                                        &value,
                                                        edge,
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
                        call_env.extend(captures);
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
                        self.disposition_recursive_declaration_call_alternatives(
                            static_origin,
                            false,
                            false,
                            false,
                        )?;
                        let checked_ih_invocation =
                            self.mint_checked_computational_ih_instance(&mut callee)?;
                        let (base, boundary) = decompose_computational_recursor(
                            LoweringOperand::Specialized(callee),
                        );
                        let (activation, invocation) = boundary.expect(
                            "recursor closure carries an invocation segment",
                        );
                        let recursive_worker = invocation.recursive_worker;
                        let recursor_parent = invocation.selection.static_origin;
                        let sibling_position = invocation.sibling_position;
                        if !recursor_invocation_is_checked(&invocation) {
                            validate_recursor_invocation_segment(&invocation)?;
                        }
                        let prepared =
                            self.prepare_static_recursor_residual(base, &invocation)?;
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
                        let base =
                            self.materialize_static_recursor_residual(builder, prepared)?;
                        let mut frames = installed_oriented_eliminator_frames(&installed);
                        frames.push(EliminatorFrame::InvocationReturn);
                        // ⭐⭐ `AC-C4` — the carried residual on the direct
                        // `lower_expr` call route.
                        if let LoweringOperand::Carried(word) = base {
                            if let Some(worker) = recursive_worker.filter(|_| {
                                matches!(
                                    self.body_emission_authority,
                                    BodyEmissionAuthority::FunctionizedUnits
                                )
                            }) {
                                let continuation_specialized = self
                                    .producer_call_has_continuation_specialization(
                                        worker.body_origin,
                                        static_origin,
                                    )?;
                                let mut inputs = args
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
                                self.append_static_recursor_worker_captures(
                                    builder,
                                    word,
                                    worker,
                                    &mut inputs,
                                )?;
                                self.enter_oriented_semantic_region(installed.checked);
                                let result = self
                                    .call_declared_recursive_position_unit(
                                        builder,
                                        worker.body_origin,
                                        &inputs,
                                    )
                                    .and_then(|value| {
                                        if continuation_specialized {
                                            // The producer branch has already
                                            // transferred this exact suffix
                                            // to its out-of-line
                                            // ContinuationSpecialization
                                            // edge. Reapplying the composed
                                            // frames here would recreate the
                                            // identity-erasing post-join
                                            // continuation that the unit
                                            // replaced.
                                            self.apply_recursive_continuation_specialization_if_planned(
                                                builder, worker, static_origin, value,
                                            )
                                        } else {
                                            self.lower_static_recursor_worker_result_composed(
                                                builder,
                                                value,
                                                &frames,
                                                worker,
                                            )
                                        }
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
                        let base = self.specialized_recursor_residual(
                            base,
                            recursor_parent,
                            sibling_position,
                            recursive_worker,
                        )?;
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
                        call_env.extend(captures);
                        if matches!(
                            self.body_emission_authority,
                            BodyEmissionAuthority::FunctionizedUnits
                        ) {
                            let continuation_specialized =
                                self.producer_call_has_continuation_specialization(
                                    body,
                                    static_origin,
                                )?;
                            self.enter_oriented_semantic_region(installed.checked);
                            let result = self
                                .call_declared_recursive_position_unit(
                                    builder,
                                    body,
                                    &call_env,
                                )
                                .and_then(|value| {
                                    if continuation_specialized {
                                        Ok(value)
                                    } else {
                                        self.lower_computational_match_value_composed(
                                            builder, value, &frames,
                                        )
                                    }
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
                };
                lower_call()
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
                Self::require_i64(builder, class, BoundaryClass::BorrowedOpaque as i64);
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
    fn effect_argument_operand(
        &self,
        lowered: &[LoweringOperand],
        static_origin: StaticOriginId,
        argument_base: usize,
        argument_ordinal: usize,
        operation: ken_host::HostOpV1,
        seat: EffectSemanticSeat,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        self.effect_operand_edge_token(
            static_origin,
            argument_base + argument_ordinal,
            SourceOperandRole::EffectArgument,
            operation,
            seat,
        )?;
        lowered
            .get(argument_ordinal)
            .cloned()
            .ok_or_else(|| unsupported("Effect", "host operation is missing a semantic argument"))
    }

    fn lower_effect_capability_operand(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        capability: &crate::RuntimeCapabilityUse,
        static_origin: StaticOriginId,
        operation: ken_host::HostOpV1,
        env: &[LoweringOperand],
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        self.effect_operand_edge_token(
            static_origin,
            0,
            SourceOperandRole::EffectCapability,
            operation,
            EffectSemanticSeat::Capability,
        )?;
        let value = self.child_occurrence(static_origin, 0, &capability.value)?;
        self.lower_expr(builder, value, env)
    }

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
        let mut file_path_operand: Option<LoweringOperand> = None;
        let mut read_buffer_operand: Option<LoweringOperand> = None;
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
                let stream = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    0,
                    operation,
                    EffectSemanticSeat::ConsoleStream,
                )?;
                let stream = self.effect_nullary_tag(
                    builder,
                    &stream,
                    console_stream_tag,
                    &[("::Stdin", 0), ("::Stdout", 1), ("::Stderr", 2)],
                    "Stream",
                )?;
                builder
                    .ins()
                    .stack_store(stream, request, request_offset(0));
                if operation == ken_host::HostOpV1::ConsoleWrite {
                    let bytes = self.effect_argument_operand(
                        &lowered,
                        static_origin,
                        argument_base,
                        1,
                        operation,
                        EffectSemanticSeat::Bytes,
                    )?;
                    let (data, len) = self.wire_effect_bytes(builder, &bytes)?;
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
                let capability = self.lower_effect_capability_operand(
                    builder,
                    capability,
                    static_origin,
                    operation,
                    env,
                )?;
                let token = self.effect_opaque_scalar(builder, &capability, true)?;
                builder.ins().stack_store(token, request, request_offset(0));
                let path = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    0,
                    operation,
                    EffectSemanticSeat::Bytes,
                )?;
                file_path_operand = Some(path.clone());
                let (path, path_len) = self.wire_effect_bytes(builder, &path)?;
                builder.ins().stack_store(path, request, request_offset(1));
                builder
                    .ins()
                    .stack_store(path_len, request, request_offset(2));
                if operation == ken_host::HostOpV1::FsWriteFile {
                    let policy = self.effect_argument_operand(
                        &lowered,
                        static_origin,
                        argument_base,
                        1,
                        operation,
                        EffectSemanticSeat::CreatePolicy,
                    )?;
                    let policy = self.effect_nullary_tag(
                        builder,
                        &policy,
                        create_policy_tag,
                        &[
                            ("::CreateNew", 0),
                            ("::CreateOrTruncate", 1),
                            ("::CreateOrKeep", 2),
                        ],
                        "CreatePolicy",
                    )?;
                    let contents = self.effect_argument_operand(
                        &lowered,
                        static_origin,
                        argument_base,
                        2,
                        operation,
                        EffectSemanticSeat::Bytes,
                    )?;
                    let (bytes, bytes_len) = self.wire_effect_bytes(builder, &contents)?;
                    builder
                        .ins()
                        .stack_store(policy, request, request_offset(3));
                    builder.ins().stack_store(bytes, request, request_offset(4));
                    builder
                        .ins()
                        .stack_store(bytes_len, request, request_offset(5));
                } else if operation == ken_host::HostOpV1::FsChangeMode {
                    let mode = self.effect_argument_operand(
                        &lowered,
                        static_origin,
                        argument_base,
                        1,
                        operation,
                        EffectSemanticSeat::ExactIntU64,
                    )?;
                    let (mode, valid_int) = self.narrow_effect_exact_int_u64(builder, &mode)?;
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
                    let mode = self.effect_argument_operand(
                        &lowered,
                        static_origin,
                        argument_base,
                        1,
                        operation,
                        EffectSemanticSeat::OpenMode,
                    )?;
                    let mode = self.effect_open_mode_tag(builder, &mode)?;
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
                let resource = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    0,
                    operation,
                    EffectSemanticSeat::Resource,
                )?;
                let token = self.effect_opaque_scalar(builder, &resource, false)?;
                builder.ins().stack_store(token, request, request_offset(0));
            }
            ken_host::HostOpV1::BufferAllocate => {
                if capability.is_some() {
                    return Err(unsupported(
                        "Effect",
                        "buffer allocation carried a capability",
                    ));
                }
                let capacity = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    0,
                    operation,
                    EffectSemanticSeat::ExactIntU64,
                )?;
                let (capacity, valid) = self.narrow_effect_exact_int_u64(builder, &capacity)?;
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
                let buffer = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    0,
                    operation,
                    EffectSemanticSeat::Resource,
                )?;
                let token = self.effect_opaque_scalar(builder, &buffer, false)?;
                let start = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    1,
                    operation,
                    EffectSemanticSeat::ExactIntU64,
                )?;
                let length = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    2,
                    operation,
                    EffectSemanticSeat::ExactIntU64,
                )?;
                let (start, start_valid) = self.narrow_effect_exact_int_u64(builder, &start)?;
                let (length, length_valid) = self.narrow_effect_exact_int_u64(builder, &length)?;
                let valid = builder.ins().band(start_valid, length_valid);
                let invalid = builder.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::Equal,
                    valid,
                    0,
                );
                record_narrow_failure(builder, invalid, 7);
                // PX8-SPAN-PROV: trailing `span_origin` acquisition token.
                let span_origin = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    3,
                    operation,
                    EffectSemanticSeat::Resource,
                )?;
                let span_origin = self.effect_opaque_scalar(builder, &span_origin, false)?;
                for (index, value) in [token, start, length, span_origin].into_iter().enumerate() {
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
                let file_operand = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    0,
                    operation,
                    EffectSemanticSeat::Resource,
                )?;
                let file = self.effect_opaque_scalar(builder, &file_operand, false)?;
                let file_offset_operand = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    1,
                    operation,
                    EffectSemanticSeat::ExactIntU64,
                )?;
                let (file_offset, file_offset_valid) =
                    self.narrow_effect_exact_int_u64(builder, &file_offset_operand)?;
                let buffer_operand = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    2,
                    operation,
                    EffectSemanticSeat::Resource,
                )?;
                let buffer = self.effect_opaque_scalar(builder, &buffer_operand, false)?;
                if operation == ken_host::HostOpV1::FsReadAt {
                    read_buffer_operand = Some(buffer_operand.clone());
                }
                let buffer_start_operand = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    3,
                    operation,
                    EffectSemanticSeat::ExactIntU64,
                )?;
                let (buffer_start, buffer_start_valid) =
                    self.narrow_effect_exact_int_u64(builder, &buffer_start_operand)?;
                let length_operand = self.effect_argument_operand(
                    &lowered,
                    static_origin,
                    argument_base,
                    4,
                    operation,
                    EffectSemanticSeat::ExactIntU64,
                )?;
                let (length, length_valid) =
                    self.narrow_effect_exact_int_u64(builder, &length_operand)?;
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
                    let span_origin_operand = self.effect_argument_operand(
                        &lowered,
                        static_origin,
                        argument_base,
                        5,
                        operation,
                        EffectSemanticSeat::Resource,
                    )?;
                    let span_origin =
                        self.effect_opaque_scalar(builder, &span_origin_operand, false)?;
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
                self.function_local
                    .host_dispatch
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
                self.function_local
                    .host_dispatch
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
                alternatives: self.synthesized_io_error_alternatives(static_origin, payload_int)?,
            });
            let error = if matches!(
                operation,
                ken_host::HostOpV1::FsReadFile
                    | ken_host::HostOpV1::FsWriteFile
                    | ken_host::HostOpV1::FsChangeMode
                    | ken_host::HostOpV1::FsOpen
            ) {
                let path = file_path_operand
                    .as_ref()
                    .cloned()
                    .expect("validated FS operation retained its exact path seat");
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
                let operation = self.synthesized_constructor(
                    static_origin,
                    SynthesizedAggregateSite::FileOperation,
                    operation_role,
                    operation_symbol,
                    Vec::new(),
                )?;
                match path {
                    LoweringOperand::Specialized(path) => {
                        let path = self.synthesized_constructor(
                            static_origin,
                            SynthesizedAggregateSite::FilePathSome,
                            SynthesizedFixedConstructorRole::OptionSome,
                            self.process_symbols.option_some.clone(),
                            vec![path],
                        )?;
                        LoweringOperand::Specialized(self.synthesized_constructor(
                            static_origin,
                            SynthesizedAggregateSite::FileError,
                            SynthesizedFixedConstructorRole::FileError,
                            self.process_symbols.file_error.clone(),
                            vec![operation, path, io_error],
                        )?)
                    }
                    carried @ LoweringOperand::Carried(_) => {
                        let path = self.synthesized_effect_carrier_constructor(
                            builder,
                            static_origin,
                            SynthesizedAggregateSite::FilePathSome,
                            SynthesizedFixedConstructorRole::OptionSome,
                            &[carried],
                        )?;
                        self.synthesized_effect_carrier_constructor(
                            builder,
                            static_origin,
                            SynthesizedAggregateSite::FileError,
                            SynthesizedFixedConstructorRole::FileError,
                            &[
                                LoweringOperand::Specialized(operation),
                                path,
                                LoweringOperand::Specialized(io_error),
                            ],
                        )?
                    }
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
                        .synthesized_io_error_alternatives(static_origin, surface_io_payload_int)?,
                });
                let identity_low = builder.ins().band_imm(resource_identity, 0xffff_ffff);
                let identity_high = builder.ins().ushr_imm(resource_identity, 32);
                let identity_low_int = self.lower_dynamic_small_int(builder, identity_low);
                let identity_high_int = self.lower_dynamic_small_int(builder, identity_high);
                let resource_kind_value = |this: &Self, discriminator, occurrence| {
                    Ok::<_, CraneliftBackendError>(Lowered::DynamicConstructor(
                        DynamicConstructorV1 {
                            discriminator,
                            alternatives: vec![
                                this.synthesized_dynamic_alternative(
                                    static_origin,
                                    SynthesizedAggregateSite::ResourceKind(occurrence, 0),
                                    wire.resource_kind_fs_handle as i64,
                                    SynthesizedFixedConstructorRole::ResourceKindFsHandle,
                                    this.process_symbols.resource_kind_fs_handle.clone(),
                                    Vec::new(),
                                )?,
                                this.synthesized_dynamic_alternative(
                                    static_origin,
                                    SynthesizedAggregateSite::ResourceKind(occurrence, 1),
                                    wire.resource_kind_buffer as i64,
                                    SynthesizedFixedConstructorRole::ResourceKindBuffer,
                                    this.process_symbols.resource_kind_buffer.clone(),
                                    Vec::new(),
                                )?,
                            ],
                        },
                    ))
                };
                let trace_identity = self.synthesized_constructor(
                    static_origin,
                    SynthesizedAggregateSite::ResourceTraceIdentity,
                    SynthesizedFixedConstructorRole::ResourceTraceIdentity,
                    self.process_symbols.resource_trace_identity.clone(),
                    vec![identity_low_int, identity_high_int],
                )?;
                LoweringOperand::Specialized(Lowered::DynamicConstructor(DynamicConstructorV1 {
                    discriminator: surface_tag,
                    alternatives: vec![
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceHostIo,
                            0,
                            SynthesizedFixedConstructorRole::ResourceHostIo,
                            self.process_symbols.resource_host_io.clone(),
                            vec![surface_io_error.clone()],
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceClosed,
                            1,
                            SynthesizedFixedConstructorRole::ResourceClosed,
                            self.process_symbols.resource_closed.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceMalformed,
                            2,
                            SynthesizedFixedConstructorRole::ResourceMalformed,
                            self.process_symbols.resource_malformed.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceRightNotHeld,
                            3,
                            SynthesizedFixedConstructorRole::ResourceRightNotHeld,
                            self.process_symbols.resource_right_not_held.clone(),
                            vec![resource_required_int, resource_held_int],
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceReleaseFailed,
                            4,
                            SynthesizedFixedConstructorRole::ResourceReleaseFailed,
                            self.process_symbols.resource_release_failed.clone(),
                            vec![
                                resource_kind_value(self, resource_kind, 0)?,
                                trace_identity,
                                surface_io_error,
                            ],
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceKindMismatch,
                            5,
                            SynthesizedFixedConstructorRole::ResourceKindMismatch,
                            self.process_symbols.resource_kind_mismatch.clone(),
                            vec![
                                resource_kind_value(self, resource_expected_kind, 1)?,
                                resource_kind_value(self, resource_actual_kind, 2)?,
                            ],
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceBufferLimit,
                            6,
                            SynthesizedFixedConstructorRole::ResourceBufferLimit,
                            self.process_symbols.resource_buffer_limit.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceInvalidOffset,
                            7,
                            SynthesizedFixedConstructorRole::ResourceInvalidOffset,
                            self.process_symbols.resource_invalid_offset.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceInvalidBounds,
                            8,
                            SynthesizedFixedConstructorRole::ResourceInvalidBounds,
                            self.process_symbols.resource_invalid_bounds.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ResourceNoProgress,
                            9,
                            SynthesizedFixedConstructorRole::ResourceNoProgress,
                            self.process_symbols.resource_no_progress.clone(),
                            Vec::new(),
                        )?,
                    ],
                }))
            } else {
                LoweringOperand::Specialized(io_error)
            };
            let success = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                tag,
                success_tag,
            );
            let ok = LoweringOperand::Specialized(if operation == ken_host::HostOpV1::FsReadFile {
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
                let span_origin = self.effect_opaque_scalar(
                    builder,
                    read_buffer_operand
                        .as_ref()
                        .expect("FsReadAt retained its exact buffer seat"),
                    false,
                )?;
                let span = self.synthesized_constructor(
                    static_origin,
                    SynthesizedAggregateSite::ReadBufferSpan,
                    SynthesizedFixedConstructorRole::PrivateBufferSpan,
                    self.process_symbols.private_buffer_span.clone(),
                    vec![
                        Lowered::ResourceToken { value: span_origin },
                        reply_start_int,
                        Lowered::BoundedNat(count),
                    ],
                )?;
                let transferred = self.synthesized_constructor(
                    static_origin,
                    SynthesizedAggregateSite::ReadTransferCount,
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
                            static_origin,
                            SynthesizedAggregateSite::ReadEof,
                            0,
                            SynthesizedFixedConstructorRole::ReadEof,
                            self.process_symbols.read_eof.clone(),
                            Vec::new(),
                        )?,
                        self.synthesized_dynamic_alternative(
                            static_origin,
                            SynthesizedAggregateSite::ReadSome,
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
                    static_origin,
                    SynthesizedAggregateSite::WriteTransferCount,
                    SynthesizedFixedConstructorRole::PrivateTransferCount,
                    self.process_symbols.private_transfer_count.clone(),
                    vec![
                        Lowered::BoundedNat(predecessor),
                        Lowered::BoundedNat(remaining),
                    ],
                )?;
                self.synthesized_constructor(
                    static_origin,
                    SynthesizedAggregateSite::Wrote,
                    SynthesizedFixedConstructorRole::Wrote,
                    self.process_symbols.wrote.clone(),
                    vec![transferred],
                )?
            } else if operation == ken_host::HostOpV1::FsHandleMetadata {
                self.lower_unsigned_u64_int(builder, detail)?
            } else {
                self.synthesized_constructor(
                    static_origin,
                    SynthesizedAggregateSite::Unit,
                    SynthesizedFixedConstructorRole::Unit,
                    self.process_symbols.unit.clone(),
                    Vec::new(),
                )?
            });
            match (&error, &ok) {
                (LoweringOperand::Specialized(error), LoweringOperand::Specialized(ok)) => {
                    Ok(LoweringOperand::Specialized(Lowered::HostResult {
                        success,
                        error: Box::new(error.clone()),
                        ok: Box::new(ok.clone()),
                        err_constructor: self.process_symbols.result_err.clone(),
                        ok_constructor: self.process_symbols.result_ok.clone(),
                    }))
                }
                _ => self.effect_carrier_host_result(builder, static_origin, success, &ok, &error),
            }
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
        let _join_plan = self.consumed_join_plan_token(join_origin)?;
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
            self.merge_scalar_operand(builder, join_origin, zero_lowered, None, "DeclarationRef")?;
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
        let (next, next_kind) = self.merge_scalar_operand(
            builder,
            join_origin,
            next?,
            Some(result_kind),
            "DeclarationRef",
        )?;
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
        reference_origin: StaticOriginId,
        symbol: &RuntimeSymbol,
        captures: &[LoweringOperand],
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
                let edge = self.operand_edge_token(
                    call_origin,
                    1 + position,
                    SourceOperandRole::CallArgument,
                )?;
                if edge.disposition() != OperandEdgeDisposition::Forwarding {
                    return Err(backend(BackendFailure::PlannerInvariant(
                        "ordinary declaration argument lost its forwarding disposition".to_string(),
                    )));
                }
                self.lower_expr(builder, arg, producer_env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if self.body_emission_authority == BodyEmissionAuthority::FunctionizedUnits {
            if params.len() != lowered_args.len() {
                return Err(unsupported(
                    "DeclarationRef",
                    format!(
                        "declaration {symbol} expects {} args but call provides {}",
                        params.len(),
                        lowered_args.len()
                    ),
                ));
            }
            let mut inputs = lowered_args;
            inputs.extend(captures.iter().cloned());
            return self.call_declared_declaration_unit(builder, reference_origin, &inputs);
        }
        self.disposition_recursive_declaration_call_alternatives(call_origin, false, true, true)?;
        let captures_edge = self.lowering_boundary_use_token(
            LoweringOnlyOperandEdge::DeclarationCaptureSpecialization,
            call_origin,
            0,
        )?;
        let captures = specialized_env_at(captures, captures_edge)?;
        // ⭐ A recursive declaration's arguments are its **loop-header
        // representation**: their shapes are compared across iterations
        // (`same_recursive_argument_shapes`) and lowered into block params. A
        // carried boundary word has no such shape, so this is a
        // specialized-only surface with the ruled fail-closed arm.
        let args_edge = self.lowering_boundary_use_token(
            LoweringOnlyOperandEdge::RecursiveDeclarationArgument,
            call_origin,
            0,
        )?;
        let lowered_args = specialized_env_at(&lowered_args, args_edge)?;
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
                        // The closed unary-fold fast path emits the declaration
                        // body's source `Match` without re-entering
                        // `lower_expr`; consume the same origin-keyed join plan
                        // here before its merge helper reborrows it.
                        self.enter_source_occurrence_plan(body.static_origin)?;
                        let zero_body =
                            self.case_body_occurrence(body.static_origin, zero_index, &zero.body)?;
                        let suc_body =
                            self.case_body_occurrence(body.static_origin, suc_index, &suc.body)?;
                        return self.lower_unary_recursive_nat_fold(
                            builder,
                            body.static_origin,
                            symbol,
                            &captures,
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
        reference_origin: StaticOriginId,
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
        match body {
            RuntimeExpr::Closure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(declaration_origin, 0, body)?;
                let captures = captures
                    .iter()
                    .map(|capture| {
                        self.lower_seed_capture(builder, capture)
                            .map(LoweringOperand::Specialized)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(LoweringOperand::Specialized(Lowered::DeclarationClosure {
                    reference_origin,
                    symbol: symbol.clone(),
                    captures,
                    params: params.clone(),
                    body: body.static_origin,
                }));
            }
            RuntimeExpr::LexicalClosure {
                captures,
                params,
                body,
            } => {
                let body = self.child_occurrence(declaration_origin, 0, body)?;
                let captures = captures
                    .iter()
                    .enumerate()
                    .map(|(position, capture)| {
                        let capture =
                            self.child_occurrence(declaration_origin, 1 + position, capture)?;
                        let lowered = self.lower_expr(builder, capture, &[])?;
                        self.retain_callable_capture(declaration_origin, 1 + position, lowered)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(LoweringOperand::Specialized(Lowered::DeclarationClosure {
                    reference_origin,
                    symbol: symbol.clone(),
                    captures,
                    params: params.clone(),
                    body: body.static_origin,
                }));
            }
            RuntimeExpr::CheckedJoinSite { .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { .. }
            | RuntimeExpr::CheckedRecursiveInvocation { .. }
            | RuntimeExpr::CheckedComputationalIHSlots { .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { .. }
            | RuntimeExpr::Value(_)
            | RuntimeExpr::Var(_)
            | RuntimeExpr::Let { .. }
            | RuntimeExpr::If { .. }
            | RuntimeExpr::PrimitiveCall { .. }
            | RuntimeExpr::Construct { .. }
            | RuntimeExpr::Match { .. }
            | RuntimeExpr::ComputationalMatch { .. }
            | RuntimeExpr::Record { .. }
            | RuntimeExpr::Project { .. }
            | RuntimeExpr::DeclarationRef { .. }
            | RuntimeExpr::ImportedDeclarationRef { .. }
            | RuntimeExpr::Call { .. }
            | RuntimeExpr::Effect { .. }
            | RuntimeExpr::Trap(_) => {}
        }
        if self.body_emission_authority == BodyEmissionAuthority::FunctionizedUnits {
            return self.call_declared_declaration_unit(builder, reference_origin, &[]);
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
        producer_eliminators: Option<&[EliminatorFrame<'_>]>,
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
            return self.lower_carried_match_body(builder, body, &arm_env, producer_eliminators);
        }
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, join_plan);
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
            let lowered =
                self.lower_carried_match_body(builder, body, &arm_env, producer_eliminators)?;
            if !self.seal_source_trap_branch(builder, &lowered)? {
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
            self.append_planned_join_params(builder, merge, &join_plan);
        }
        let some_block = builder.create_block();
        let none_block = builder.create_block();
        let mut merge_kind = None;
        let mut terminal_trap = None;
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
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                terminal_trap.get_or_insert_with(|| trap.clone());
            }
            if self.seal_source_trap_branch(builder, &lowered)? {
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
            let trap = terminal_trap.ok_or_else(|| {
                backend_module(
                    "borrowed Option join omitted both a continuing predecessor and a source \
                     trap"
                        .to_string(),
                )
            })?;
            return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
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
        default: &RuntimeTrap,
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
            self.append_planned_join_params(builder, merge, &join_plan);
            #[cfg(test)]
            if D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get)
                == JoinConsumptionMutation::DispositionDynamicHostResultMerge
            {
                self.function_local
                    .dispositioned_join_origins
                    .insert(static_origin);
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
            if !self
                .static_transition_plan
                .source_environment_slot_is_used(body.static_origin, 0)?
            {
                self.disposition_unobserved_operand(&arm_env[0])?;
            }
            let arm_edge =
                self.operand_edge_token(static_origin, 1 + index, SourceOperandRole::MatchArm)?;
            if arm_edge.disposition() != OperandEdgeDisposition::Forwarding {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "dynamic HostResult match arm lost its forwarding disposition".to_string(),
                )));
            }
            let lowered = self.lower_expr(builder, body, &arm_env)?;
            if self.seal_source_trap_branch(builder, &lowered)? {
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
                    let word = self.carried_join_arm(
                        builder,
                        body.static_origin,
                        LoweringOnlyOperandEdge::JoinArm,
                        0,
                        lowered,
                        None,
                        "Match",
                    )?;
                    builder.ins().jump(merge, &[word.word.into()]);
                }
            }
        }
        let Some(merge) = merge else {
            let unreachable_continuation = builder.create_block();
            builder.switch_to_block(unreachable_continuation);
            return Ok(LoweringOperand::Specialized(Lowered::Trap(default.clone())));
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
        let merge = join_plan
            .has_continuing_predecessor
            .then(|| builder.create_block());
        if let Some(merge) = merge {
            self.append_planned_join_params(builder, merge, &join_plan);
        }
        let predecessor = nat.predecessor(builder);
        let is_zero =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, nat.value, 0);
        builder.ins().brif(is_zero, zero_block, &[], suc_block, &[]);
        let mut merge_kind = None;
        let mut terminal_trap = None;
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
            if let LoweringOperand::Specialized(Lowered::Trap(trap)) = &lowered {
                terminal_trap.get_or_insert_with(|| trap.clone());
            }
            if self.seal_source_trap_branch(builder, &lowered)? {
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a BoundedNat merge despite a continuing predecessor"
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
                "BoundedNat",
            )?;
        }
        let Some(merge) = merge else {
            let unreachable = builder.create_block();
            builder.switch_to_block(unreachable);
            let trap = terminal_trap.ok_or_else(|| {
                backend_module(
                    "BoundedNat join omitted both a continuing predecessor and a source trap"
                        .to_string(),
                )
            })?;
            return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
        };
        self.finish_planned_join(builder, merge, &join_plan, merge_kind, "BoundedNat")
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

        let source_default = match continuation {
            DynamicConstructorContinuation::Ordinary { default, .. }
            | DynamicConstructorContinuation::Producer { default, .. } => default,
        };
        let static_origin = match continuation {
            DynamicConstructorContinuation::Ordinary { static_origin, .. }
            | DynamicConstructorContinuation::Producer { static_origin, .. } => static_origin,
        };
        let join_plan = self.consumed_join_plan_token(static_origin)?;
        let merge = join_plan.has_continuing_predecessor.then(|| {
            let merge = builder.create_block();
            self.append_planned_join_params(builder, merge, &join_plan);
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
            let arm_edge =
                self.operand_edge_token(static_origin, 1 + index, SourceOperandRole::MatchArm)?;
            if arm_edge.disposition() != OperandEdgeDisposition::Forwarding {
                return Err(backend(BackendFailure::PlannerInvariant(
                    "dynamic constructor match arm lost its forwarding disposition".to_string(),
                )));
            }
            let lowered = match continuation {
                DynamicConstructorContinuation::Ordinary { .. } => {
                    self.lower_expr(builder, body, &arm_env)?
                }
                DynamicConstructorContinuation::Producer { eliminators, .. } => {
                    self.lower_computational_producer_expr(builder, body, &arm_env, eliminators)?
                }
            };
            if self.seal_source_trap_branch(builder, &lowered)? {
                test_block = next;
                continue;
            }
            let merge = merge.ok_or_else(|| {
                backend_module(
                    "join plan omitted a DynamicConstructor merge despite a continuing \
                     predecessor"
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
                "DynamicConstructor",
            )?;
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
            return Ok(LoweringOperand::Specialized(Lowered::Trap(
                source_default.clone(),
            )));
        };
        self.finish_planned_join(builder, merge, &join_plan, merge_kind, "DynamicConstructor")
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
                let _edge = self.operand_edge_token(
                    static_origin,
                    position,
                    SourceOperandRole::PrimitiveArgument,
                )?;
                let arg = self.child_occurrence(static_origin, position, arg)?;
                self.lower_expr(builder, arg, env)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if lowered_args.iter().any(|arg| {
            matches!(
                arg,
                LoweringOperand::Specialized(Lowered::RecursiveBackedge)
            )
        }) {
            return Ok(LoweringOperand::Specialized(Lowered::RecursiveBackedge));
        }

        match &primitive.partiality {
            RuntimePartiality::Total => {}
            RuntimePartiality::SafeOption { .. } | RuntimePartiality::SafeResult { .. } => {}
            RuntimePartiality::CheckedTrap { obligation } => {
                self.assumptions.insert(format!(
                    "checked partial obligation {obligation} not discharged"
                ));
                let trap = crate::cranelift_backend::planning::planned_partiality_trap(primitive)
                    .expect("CheckedTrap has one planner-derived trap");
                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
            }
            RuntimePartiality::TrustedTrap { assumption } => {
                self.assumptions.insert(format!(
                    "trusted partial assumption {assumption} remains visible"
                ));
                let trap = crate::cranelift_backend::planning::planned_partiality_trap(primitive)
                    .expect("TrustedTrap has one planner-derived trap");
                return Ok(LoweringOperand::Specialized(Lowered::Trap(trap)));
            }
        }

        // A primitive's static symbol determines whether its operands are
        // scalar Ints or Bools. A carried word in one of those positions is
        // projected through the emitted scalar helper; no runtime tag chooses
        // which source type the operand is.
        let scalar_kind = match primitive.symbol.as_str() {
            "add_int" | "sub_int" | "mul_int" | "eq_int" | "leq_int" | "uint8_to_int"
            | "int_to_uint8_raw" => Some("Int"),
            "not_bool" | "and_bool" | "or_bool" => Some("Bool"),
            _ => None,
        };
        let lowered_args = if primitive.symbol == "bytes_length" {
            match lowered_args.as_slice() {
                [LoweringOperand::Specialized(_)] => self.specialized_source_env_at(
                    &lowered_args,
                    static_origin,
                    0,
                    SourceOperandRole::PrimitiveArgument,
                )?,
                [LoweringOperand::Carried(word)] => {
                    let class = self.emit_carrier_class(builder, *word)?;
                    Self::require_i64(builder, class, BoundaryClass::BorrowedOpaque as i64);
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
                        Self::require_i64(builder, class, BoundaryClass::BorrowedOpaque as i64);
                        let pointer = self.emit_carrier_scalar(builder, word)?;
                        Ok(Lowered::BorrowedNativeValue { pointer })
                    }
                    (1, LoweringOperand::Carried(word)) => {
                        let tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
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
                        let tag = builder
                            .ins()
                            .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
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
            self.specialized_source_env_at(
                &lowered_args,
                static_origin,
                0,
                SourceOperandRole::PrimitiveArgument,
            )?
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
