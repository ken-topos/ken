//! Oriented control, PX8J/PX8DS recursor-consumer, root-authority and
//! source-install lowering tests (RT-SPLIT §10.2: `oriented_*`, `px8j_*`,
//! root-authority, join-site, source-install and recursor tests -> `control`).

use super::*;
use crate::RuntimeSymbolMetadata;

#[derive(Clone, Copy)]
enum Px8jInstallMalformation {
    SelectionRole,
    UnwindRole,
    UnwindOrigin,
    RepeatedScopeIdentity,
}

#[derive(Clone, Copy, Debug)]
enum Px8dsEdgeMutation {
    Delete,
    Duplicate,
    StaleParent,
    CrossSibling,
    WrongStaticParent,
}
/// ⚠ The plan here is a minimal inert one: every test that uses this builder
/// exercises a ledger, authority, or frame validator and never lowers an
/// expression through it, so no child origin is ever derived. A test that DOES
/// lower a fixture builds its own `Lowering` with that fixture's plan.
fn root_authority_test_lowering<'a>(seed_env: &'a NativeSeedEnvironment) -> Lowering<'a> {
    Lowering {
        seed_env,
        seed_material: crate::cranelift_backend::lowering::seed_material::SeedMaterialRefs::none_for_tests(),
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
        native_join_plan: Some(crate::NativeJoinPlanV1 {
            representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: vec![self_consistent_root_join_site(0)],
        }),
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
        process_object: true,
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
    }
}

#[cfg(test)]
fn run_px8j_malformed_recursor_consumer(
    consumer: Px8jDirectRecursorConsumer,
    malformation: Px8jRecursorMalformation,
) -> Result<LoweringOperand, CraneliftBackendError> {
    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px8j_malformed_recursor", Linkage::Local, &signature)
        .map_err(|error| backend_module(error.to_string()))?;
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let seed_env = NativeSeedEnvironment::empty();
    // The consumer under test lowers exactly one of these two fixtures, so the
    // plan is that fixture's own: every origin the lowering derives below is a
    // real positional child of a really-planned occurrence.
    let call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: Vec::new(),
    };
    let pending_let = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Var(1)),
            args: Vec::new(),
        }),
    };
    let lowered_fixture = match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer => &pending_let,
        Px8jDirectRecursorConsumer::ProducerCall | Px8jDirectRecursorConsumer::OrdinaryCall => {
            &call
        }
    };
    let (static_transition_plan, fixture_origin) = planned_root_occurrence(lowered_fixture);
    let mut compiler = Lowering {
        seed_env: &seed_env,
        seed_material: crate::cranelift_backend::lowering::seed_material::SeedMaterialRefs::none_for_tests(),
        declarations: BTreeMap::new(),
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
    let origin = RecursorProducerOriginId(7);
    let cursor = ContinuationCursorId(9);
    let layer = |role| ComputationalRecursorLayer {
        cases: Vec::new(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px8j malformed recursor role".to_string(),
        },
        outer_env: Vec::new(),
        static_origin: inert_test_static_origin(),
        provenance: RecursorFrameProvenance(6),
        role,
        checked_frame_id: None,
        checked_invocation_id: None,
        checked_invocation_source: None,
        checked_invocation_depth: 0,
        semantic_pending: matches!(role, RecursorLayerRole::SelectsOccurrence { .. }),
    };
    let selection = layer(match malformation {
        Px8jRecursorMalformation::SelectionRole => RecursorLayerRole::ExitsScope {
            origin,
            scope_origin: origin,
            parent_scope: None,
        },
        Px8jRecursorMalformation::RepeatedScopeIdentity
        | Px8jRecursorMalformation::BrokenScopeParent => {
            RecursorLayerRole::SelectsOccurrence { origin }
        }
    });
    let unwind = match malformation {
        Px8jRecursorMalformation::SelectionRole => Vec::new(),
        Px8jRecursorMalformation::RepeatedScopeIdentity => vec![
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(11),
                parent_scope: None,
            }),
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(11),
                parent_scope: Some(RecursorProducerOriginId(11)),
            }),
        ],
        Px8jRecursorMalformation::BrokenScopeParent => vec![
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(11),
                parent_scope: None,
            }),
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(12),
                parent_scope: Some(RecursorProducerOriginId(99)),
            }),
        ],
    };
    let recursor = Lowered::ComputationalRecursorClosure {
        residual: Box::new(LoweringOperand::Specialized(Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            // An inert residual body. This test drives the recursor-malformation
            // validator and never lowers the body, so the inert planned origin is
            // the whole of it — and since B2A-S the carrier *is* the origin, the
            // fixture can no longer pair an arbitrary term with an unrelated tag.
            body: inert_test_static_origin(),
        })),
        activation: ContinuationActivationId(8),
        invocation: RecursorInvocationSegment::new(
            origin,
            0,
            selection,
            RecursorUnwindStack {
                later_wrappers_in_construction_order: unwind,
            },
            cursor,
            None,
            None,
        ),
    };
    let active = ActiveContinuationFrame {
        activation: ContinuationActivationId(8),
        cursor,
        parent: None,
        pending: &[],
        selected_ancestry: &[],
        source_lineage: &[],
        source_selected_cursor: None,
        selected_scope: None,
    };
    let active_frames = [EliminatorFrame::Active(active)];
    let env = [LoweringOperand::Specialized(recursor)];
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    let occurrence = SourceOccurrence {
        expr: lowered_fixture,
        static_origin: fixture_origin,
    };
    match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer
        | Px8jDirectRecursorConsumer::ProducerCall => compiler.lower_computational_producer_expr(
            &mut builder,
            occurrence,
            &env,
            &active_frames,
        ),
        Px8jDirectRecursorConsumer::OrdinaryCall => {
            compiler.lower_expr(&mut builder, occurrence, &env)
        }
    }
}

fn oriented_dynamic_sibling_fixture() -> (
    crate::OrientedSubcontinuationPlanV1,
    RecursorInvocationSegment,
    Vec<DynamicSpliceEdge>,
) {
    let plan = oriented_test_ih_plan();
    let origin = RecursorProducerOriginId(60);
    let mut segment = RecursorInvocationSegment::new(
        origin,
        0,
        oriented_test_instance_layer(
            2,
            11,
            1,
            true,
            RecursorLayerRole::SelectsOccurrence { origin },
        ),
        RecursorUnwindStack {
            later_wrappers_in_construction_order: vec![oriented_test_instance_layer(
                0,
                12,
                1,
                true,
                RecursorLayerRole::ExitsScope {
                    origin,
                    scope_origin: RecursorProducerOriginId(61),
                    parent_scope: None,
                },
            )],
        },
        ContinuationCursorId(13),
        None,
        None,
    );
    segment.dynamic_splice_edges = vec![DynamicSpliceEdgeId(71), DynamicSpliceEdgeId(72)];
    let edges = vec![
        DynamicSpliceEdge {
            edge_id: DynamicSpliceEdgeId(71),
            child_invocation_instance_id: 11,
            parent_invocation_instance_id: 0,
            checked_call_template_id: 102,
            parent_frame_template_id: 2,
            segment_site_id: 9,
        },
        DynamicSpliceEdge {
            edge_id: DynamicSpliceEdgeId(72),
            child_invocation_instance_id: 12,
            parent_invocation_instance_id: 0,
            checked_call_template_id: 100,
            parent_frame_template_id: 0,
            segment_site_id: 9,
        },
    ];
    (plan, segment, edges)
}

#[test]
fn oriented_same_depth_siblings_require_exact_dynamic_edges() {
    let (plan, segment, edges) = oriented_dynamic_sibling_fixture();

    let mut old_flat = std::iter::once(&segment.selection)
        .chain(segment.unwind.later_wrappers_in_construction_order.iter())
        .filter(|layer| layer.semantic_pending)
        .collect::<Vec<_>>();
    old_flat.sort_by_key(|layer| {
        (
            std::cmp::Reverse(layer.checked_invocation_depth),
            plan.frame(layer.checked_frame_id.unwrap())
                .unwrap()
                .semantic_position,
        )
    });
    let [left, right] = old_flat.as_slice() else {
        panic!("the discriminator must carry exactly two same-depth siblings")
    };
    assert_eq!(left.checked_invocation_depth, 1);
    assert_eq!(right.checked_invocation_depth, 1);
    let left = plan.frame(left.checked_frame_id.unwrap()).unwrap();
    let right = plan.frame(right.checked_frame_id.unwrap()).unwrap();
    assert_ne!(
        left.output_interface, right.input_interface,
        "the retired flat ordering must invent the non-composable sibling adjacency"
    );

    let installed = compose_oriented_subcontinuation(
        Some(&plan),
        None,
        ContinuationActivationId(14),
        segment,
        edges,
    )
    .expect("exact child-to-parent edges keep same-depth siblings separate");
    assert_eq!(
        installed
            .semantic_frames
            .iter()
            .map(|frame| (
                frame.checked_invocation_id.unwrap(),
                frame.checked_frame_id.unwrap(),
            ))
            .collect::<Vec<_>>(),
        vec![(11, 2), (12, 0)],
    );
}

#[test]
fn oriented_dynamic_edge_mutations_reject_through_named_lanes() {
    let reject =
        |segment: RecursorInvocationSegment, edges: Vec<DynamicSpliceEdge>, expected: &str| {
            let plan = oriented_test_ih_plan();
            let error = match compose_oriented_subcontinuation(
                Some(&plan),
                None,
                ContinuationActivationId(14),
                segment,
                edges,
            ) {
                Ok(_) => panic!("a malformed dynamic splice graph must reject before CFG"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error,
                    CraneliftBackendError::Unsupported(UnsupportedLowering {
                        construct: "OrientedSubcontinuationPlanV1",
                        ref reason,
                    }) if reason.contains(expected)
                ),
                "expected {expected:?}, got {error:?}"
            );
        };

    let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
    edges.pop();
    reject(segment, edges, "deletion leaves an unparented");

    let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
    edges.push(DynamicSpliceEdge {
        edge_id: DynamicSpliceEdgeId(73),
        child_invocation_instance_id: 11,
        parent_invocation_instance_id: 0,
        checked_call_template_id: 102,
        parent_frame_template_id: 2,
        segment_site_id: 9,
    });
    reject(segment, edges, "duplicate affine splice edges");

    let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
    edges[0].parent_invocation_instance_id = 99;
    reject(segment, edges, "stale parent invocation");

    let (_, segment, mut edges) = oriented_dynamic_sibling_fixture();
    edges[0].parent_frame_template_id = 1;
    reject(segment, edges, "disagrees with its checked static parent");
}

#[test]
fn oriented_dynamic_edge_ledger_is_affine_and_sibling_isolated() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut lowering = root_authority_test_lowering(&seed_env);
    let (_, mut segment, mut edges) = oriented_dynamic_sibling_fixture();
    let edge = edges.remove(0);
    segment.dynamic_splice_edges = vec![edge.edge_id];
    lowering.dynamic_splice_edges.insert(edge.edge_id, edge);

    let consumed = lowering
        .take_dynamic_splice_edges(&segment)
        .expect("the owning invocation consumes its edge exactly once");
    assert_eq!(consumed.len(), 1);
    let stolen = match lowering.take_dynamic_splice_edges(&segment) {
        Ok(_) => panic!("a sibling cannot steal an already-consumed edge"),
        Err(error) => error,
    };
    assert!(matches!(
        stolen,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason.contains("consumed by a sibling")
    ));

    let (_, mut duplicated, mut edges) = oriented_dynamic_sibling_fixture();
    let edge = edges.remove(0);
    duplicated.dynamic_splice_edges = vec![edge.edge_id, edge.edge_id];
    lowering.dynamic_splice_edges.insert(edge.edge_id, edge);
    let duplicate = match lowering.take_dynamic_splice_edges(&duplicated) {
        Ok(_) => panic!("one carrier cannot duplicate an affine edge handle"),
        Err(error) => error,
    };
    assert!(matches!(
        duplicate,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason.contains("handle is duplicated")
    ));
}

fn run_px8ds_edge_consumer(
    consumer: Px8jDirectRecursorConsumer,
    mutation: Px8dsEdgeMutation,
) -> Result<LoweringOperand, CraneliftBackendError> {
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = root_authority_test_lowering(&seed_env);
    compiler.native_join_plan = None;
    compiler.root_terminal_authority = None;
    compiler.process_object = false;
    let (plan, mut segment, mut edges) = oriented_dynamic_sibling_fixture();
    compiler.oriented_subcontinuation_plan = Some(plan);

    match mutation {
        Px8dsEdgeMutation::Delete => {
            edges.remove(0);
        }
        Px8dsEdgeMutation::Duplicate => {
            segment
                .dynamic_splice_edges
                .push(segment.dynamic_splice_edges[0]);
        }
        Px8dsEdgeMutation::StaleParent => {
            edges[0].parent_invocation_instance_id = 99;
        }
        Px8dsEdgeMutation::CrossSibling => {
            let stolen = RecursorInvocationSegment {
                dynamic_splice_edges: vec![segment.dynamic_splice_edges[0]],
                ..segment.clone()
            };
            for edge in edges.drain(..) {
                compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
            }
            compiler.take_dynamic_splice_edges(&stolen)?;
        }
        Px8dsEdgeMutation::WrongStaticParent => {
            edges[0].parent_frame_template_id = 1;
        }
    }
    for edge in edges {
        compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
    }

    let cursor = segment.resume_cursor;
    let activation = ContinuationActivationId(90);
    let recursor = Lowered::ComputationalRecursorClosure {
        residual: Box::new(LoweringOperand::Specialized(Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            // An inert residual body, as in the PX8J fixture above: the carrier is
            // the origin, and this test never lowers the body.
            body: inert_test_static_origin(),
        })),
        activation,
        invocation: segment,
    };
    let active = ActiveContinuationFrame {
        activation,
        cursor,
        parent: None,
        pending: &[],
        selected_ancestry: &[],
        source_lineage: &[],
        source_selected_cursor: None,
        selected_scope: None,
    };
    let active_frames = [EliminatorFrame::Active(active)];
    let env = [LoweringOperand::Specialized(recursor)];
    let call = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::Var(0)),
        args: Vec::new(),
    };
    let pending_let = RuntimeExpr::Let {
        value: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        body: Box::new(RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::Var(1)),
            args: Vec::new(),
        }),
    };
    // Plan the fixture this consumer actually lowers, and install that plan on
    // the compiler under test.
    let lowered_fixture = match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer => &pending_let,
        Px8jDirectRecursorConsumer::ProducerCall | Px8jDirectRecursorConsumer::OrdinaryCall => {
            &call
        }
    };
    let (static_transition_plan, fixture_origin) = planned_root_occurrence(lowered_fixture);
    compiler.static_transition_plan = static_transition_plan;

    let mut module = new_jit_module()?;
    let mut signature = module.make_signature();
    signature.returns.push(AbiParam::new(types::I64));
    let func_id = module
        .declare_function("px8ds_edge_consumer", Linkage::Local, &signature)
        .map_err(|error| backend_module(error.to_string()))?;
    let mut context = module.make_context();
    context.func =
        Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), signature);
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    let occurrence = SourceOccurrence {
        expr: lowered_fixture,
        static_origin: fixture_origin,
    };
    match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer
        | Px8jDirectRecursorConsumer::ProducerCall => compiler.lower_computational_producer_expr(
            &mut builder,
            occurrence,
            &env,
            &active_frames,
        ),
        Px8jDirectRecursorConsumer::OrdinaryCall => {
            compiler.lower_expr(&mut builder, occurrence, &env)
        }
    }
}

#[test]
fn oriented_edge_mutations_reject_in_all_three_direct_consumers() {
    for consumer in [
        Px8jDirectRecursorConsumer::PendingLetProducer,
        Px8jDirectRecursorConsumer::ProducerCall,
        Px8jDirectRecursorConsumer::OrdinaryCall,
    ] {
        for (mutation, expected) in [
            (Px8dsEdgeMutation::Delete, "deleted, replayed"),
            (Px8dsEdgeMutation::Duplicate, "handle is duplicated"),
            (Px8dsEdgeMutation::StaleParent, "stale parent invocation"),
            (Px8dsEdgeMutation::CrossSibling, "consumed by a sibling"),
            (
                Px8dsEdgeMutation::WrongStaticParent,
                "disagrees with its checked static parent",
            ),
        ] {
            let error = match run_px8ds_edge_consumer(consumer, mutation) {
                Ok(_) => panic!("{consumer:?}/{mutation:?} must reject before CFG"),
                Err(error) => error,
            };
            assert!(
                matches!(
                    error,
                    CraneliftBackendError::Unsupported(UnsupportedLowering {
                        construct: "OrientedSubcontinuationPlanV1",
                        ref reason,
                    }) if reason.contains(expected)
                ),
                "{consumer:?}/{mutation:?}: expected {expected:?}, got {error:?}"
            );
        }
    }
}

fn run_px8ds_source_consumer(mutation: Px8dsEdgeMutation) -> Result<(), CraneliftBackendError> {
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = root_authority_test_lowering(&seed_env);
    compiler.native_join_plan = None;
    compiler.root_terminal_authority = None;
    compiler.process_object = false;
    let (plan, mut segment, mut edges) = oriented_dynamic_sibling_fixture();
    compiler.oriented_subcontinuation_plan = Some(plan);

    match mutation {
        Px8dsEdgeMutation::Delete => {
            edges.remove(0);
        }
        Px8dsEdgeMutation::Duplicate => {
            segment
                .dynamic_splice_edges
                .push(segment.dynamic_splice_edges[0]);
        }
        Px8dsEdgeMutation::StaleParent => {
            edges[0].parent_invocation_instance_id = 99;
        }
        Px8dsEdgeMutation::CrossSibling => {
            let stolen = RecursorInvocationSegment {
                dynamic_splice_edges: vec![segment.dynamic_splice_edges[0]],
                ..segment.clone()
            };
            for edge in edges.drain(..) {
                compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
            }
            compiler.take_dynamic_splice_edges(&stolen)?;
        }
        Px8dsEdgeMutation::WrongStaticParent => {
            edges[0].parent_frame_template_id = 1;
        }
    }
    for edge in edges {
        compiler.dynamic_splice_edges.insert(edge.edge_id, edge);
    }
    compiler
        .install_recursor_invocation(
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue),
            ContinuationActivationId(90),
            segment,
            None,
        )
        .map(|_| ())
}

#[test]
fn oriented_edge_mutations_reject_in_the_source_machine_consumer() {
    for (mutation, expected) in [
        (Px8dsEdgeMutation::Delete, "deleted, replayed"),
        (Px8dsEdgeMutation::Duplicate, "handle is duplicated"),
        (Px8dsEdgeMutation::StaleParent, "stale parent invocation"),
        (Px8dsEdgeMutation::CrossSibling, "consumed by a sibling"),
        (
            Px8dsEdgeMutation::WrongStaticParent,
            "disagrees with its checked static parent",
        ),
    ] {
        let error = match run_px8ds_source_consumer(mutation) {
            Ok(()) => panic!("source {mutation:?} must reject before CFG"),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "OrientedSubcontinuationPlanV1",
                    ref reason,
                }) if reason.contains(expected)
            ),
            "source {mutation:?}: expected {expected:?}, got {error:?}"
        );
    }
}

#[test]
fn rt_escape_within_path_duplicate_frame_consume_still_rejects() {
    // RT-ESCAPE: forking `consumed_subcontinuation_frames` per mutually-exclusive
    // arm must not weaken the same-path affine guard. On a straight-line path
    // (no branch, so `lower_forked_branch`'s per-arm reset never applies),
    // consuming one checked frame twice must still reject before CFG. Direct-API
    // PX8DS-fixture style; exercises the frame consume the dynamic-splice-edge
    // mutation suite does not reach.
    let seed_env = NativeSeedEnvironment::empty();
    let (_expr, decl, plan) = occurrence_exact_marker_fixture(false, false);
    let RuntimeDeclarationKind::Transparent { body } = decl.kind else {
        panic!("fixture declaration is transparent");
    };
    let RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } = body else {
        panic!("declaration body is a checked subcontinuation frame");
    };
    let RuntimeExpr::ComputationalMatch { cases, default, .. } = *body else {
        panic!("checked frame wraps a computational match");
    };
    let mut compiler = root_authority_test_lowering(&seed_env);
    compiler.native_join_plan = None;
    compiler.root_terminal_authority = None;
    compiler.process_object = false;
    compiler.oriented_subcontinuation_plan = Some(plan);

    // First consume on the path succeeds.
    compiler
        .enter_checked_subcontinuation_frame(frame_id)
        .expect("first enter of the checked frame");
    assert_eq!(
        compiler
            .consume_checked_subcontinuation_frame(&cases, &default)
            .expect("first consume of the checked frame succeeds"),
        Some(frame_id)
    );

    // A second enter + consume of the same frame on the same path rejects.
    compiler
        .enter_checked_subcontinuation_frame(frame_id)
        .expect("second enter re-marks the active frame");
    let err = compiler
        .consume_checked_subcontinuation_frame(&cases, &default)
        .expect_err("a same-path duplicate consume must reject before CFG");
    assert!(
        matches!(
            err,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "OrientedSubcontinuationPlanV1",
                ref reason,
            }) if reason.contains("consumed more than once")
        ),
        "expected 'consumed more than once', got {err:?}"
    );
}

#[test]
fn oriented_source_open_occurrence_cross_checks_the_closure_selected_parent() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = root_authority_test_lowering(&seed_env);
    let (_, _, mut edges) = oriented_dynamic_sibling_fixture();
    let edge = edges.remove(0);
    let edge_id = edge.edge_id;
    compiler.dynamic_splice_edges.insert(edge_id, edge);
    let instance = CheckedRecursiveInvocationInstance {
        source: InvocationTemplateRef::ComputationalIHCall(102),
        invocation_instance_id: 11,
        semantic_depth: 1,
        dynamic_splice_edge: Some(edge_id),
    };
    let mut open = OwnedSelectedScope {
        scope_origin: RecursorProducerOriginId(70),
        parent_scope: None,
        frame: ComputationalRecursorFramePayload {
            cases: Vec::new(),
            default: RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message: "PX8-DS source parent".to_string(),
            },
            outer_env: Vec::new(),
            static_origin: inert_test_static_origin(),
            provenance: RecursorFrameProvenance(71),
            checked_frame_id: Some(2),
            checked_invocation_id: Some(0),
            checked_invocation_source: None,
            checked_invocation_depth: 0,
        },
    };
    compiler
        .validate_source_dynamic_splice_parent(instance, &open)
        .expect("the source open occurrence agrees with closure selection");
    open.frame.checked_frame_id = Some(0);
    let mismatch = compiler
        .validate_source_dynamic_splice_parent(instance, &open)
        .expect_err("source and closure parent identities must agree before CFG");
    assert!(matches!(
        mismatch,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason.contains("source open occurrence disagrees")
    ));
}

#[test]
fn distinguished_root_authority_is_checked_affine_and_cursor_bound() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut lowering = root_authority_test_lowering(&seed_env);
    let authority = lowering
        .take_distinguished_root_answer_authority()
        .expect("the exact checked root site validates")
        .expect("process lowering carries root authority");
    lowering.root_terminal_authority = Some(authority);
    lowering
        .mint_terminal_answer_authority()
        .expect("the first exhausted-root mint consumes the authority");
    let repeated = match lowering.mint_terminal_answer_authority() {
        Ok(_) => panic!("the affine root authority cannot mint twice"),
        Err(error) => error,
    };
    assert!(matches!(
        repeated,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "NativeJoinPlanV1",
            reason,
        }) if reason == "terminal answer has no affine checked-root authority"
    ));

    let mut lowering = root_authority_test_lowering(&seed_env);
    let mut authority = lowering
        .take_distinguished_root_answer_authority()
        .unwrap()
        .unwrap();
    authority.outer_cursor = Some(ContinuationCursorId(7));
    let transplanted = lowering
        .restore_root_terminal_authority(Some(authority), ContinuationCursorId(8))
        .expect_err("a root token cannot cross the wrong source cursor");
    assert!(matches!(
        transplanted,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "NativeJoinPlanV1",
            reason,
        }) if reason == "checked root answer authority returned through the wrong outer cursor"
    ));
}

#[test]
fn px8j_all_three_direct_consumers_propagate_the_role_validator() {
    for consumer in [
        Px8jDirectRecursorConsumer::PendingLetProducer,
        Px8jDirectRecursorConsumer::ProducerCall,
        Px8jDirectRecursorConsumer::OrdinaryCall,
    ] {
        let error = match run_px8j_malformed_recursor_consumer(
            consumer,
            Px8jRecursorMalformation::SelectionRole,
        ) {
            Ok(_) => panic!("each live recursor consumer must reject the malformed selection"),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "ComputationalRecursor",
                    ref reason,
                }) if reason == "recursor selection role does not select the invocation origin"
            ),
            "{consumer:?}: {error:?}"
        );
    }
}

#[test]
fn px8j_release_validator_rejects_repeated_and_broken_scope_lineage() {
    for (malformation, expected_reason) in [
        (
            Px8jRecursorMalformation::RepeatedScopeIdentity,
            "recursor unwind repeats a selected scope identity",
        ),
        (
            Px8jRecursorMalformation::BrokenScopeParent,
            "recursor unwind has a broken selected-scope parent link",
        ),
    ] {
        let error = match run_px8j_malformed_recursor_consumer(
            Px8jDirectRecursorConsumer::OrdinaryCall,
            malformation,
        ) {
            Ok(_) => panic!("the real direct consumer must propagate release validation"),
            Err(error) => error,
        };
        assert!(
            matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "ComputationalRecursor",
                    ref reason,
                }) if reason == expected_reason
            ),
            "{malformation:?}: {error:?}"
        );
    }
}

fn run_px8j_source_machine_install(
    malformation: Option<Px8jInstallMalformation>,
) -> Result<SourceContinuation<'static>, CraneliftBackendError> {
    let seed_env = NativeSeedEnvironment::empty();
    let mut compiler = root_authority_test_lowering(&seed_env);
    compiler.native_join_plan = None;
    compiler.root_terminal_authority = None;
    compiler.process_object = false;

    let origin = RecursorProducerOriginId(17);
    let layer = |role| ComputationalRecursorLayer {
        cases: Vec::new(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "PX8-J-ERR source install".to_string(),
        },
        outer_env: Vec::new(),
        static_origin: inert_test_static_origin(),
        provenance: RecursorFrameProvenance(18),
        role,
        checked_frame_id: None,
        checked_invocation_id: None,
        checked_invocation_source: None,
        checked_invocation_depth: 0,
        semantic_pending: matches!(role, RecursorLayerRole::SelectsOccurrence { .. }),
    };
    let selection = match malformation {
        Some(Px8jInstallMalformation::SelectionRole) => layer(RecursorLayerRole::ExitsScope {
            origin,
            scope_origin: RecursorProducerOriginId(18),
            parent_scope: None,
        }),
        _ => layer(RecursorLayerRole::SelectsOccurrence { origin }),
    };
    let unwind = match malformation {
        None => Vec::new(),
        Some(Px8jInstallMalformation::SelectionRole) => Vec::new(),
        Some(Px8jInstallMalformation::UnwindRole) => {
            vec![layer(RecursorLayerRole::SelectsOccurrence { origin })]
        }
        Some(Px8jInstallMalformation::UnwindOrigin) => {
            vec![layer(RecursorLayerRole::ExitsScope {
                origin: RecursorProducerOriginId(99),
                scope_origin: RecursorProducerOriginId(19),
                parent_scope: None,
            })]
        }
        Some(Px8jInstallMalformation::RepeatedScopeIdentity) => vec![
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(19),
                parent_scope: None,
            }),
            layer(RecursorLayerRole::ExitsScope {
                origin,
                scope_origin: RecursorProducerOriginId(19),
                parent_scope: Some(RecursorProducerOriginId(19)),
            }),
        ],
    };
    let invocation = RecursorInvocationSegment::new(
        origin,
        0,
        selection,
        RecursorUnwindStack {
            later_wrappers_in_construction_order: unwind,
        },
        ContinuationCursorId(20),
        None,
        None,
    );
    assert!(!recursor_invocation_is_checked(&invocation));

    compiler.install_recursor_invocation(
        SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue),
        ContinuationActivationId(21),
        invocation,
        None,
    )
}

#[test]
fn px8j_source_machine_install_rejects_repeated_scope_identity() {
    let error =
        match run_px8j_source_machine_install(Some(Px8jInstallMalformation::RepeatedScopeIdentity))
        {
            Ok(_) => panic!("the unchecked source-machine install must validate before CFG"),
            Err(error) => error,
        };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalRecursor",
            reason,
        }) if reason == "recursor unwind repeats a selected scope identity"
    ));
}

#[test]
fn px8j_source_machine_install_rejects_wrong_control_roles_and_origins() {
    for (malformation, expected_reason) in [
        (
            Px8jInstallMalformation::SelectionRole,
            "recursor selection role does not select the invocation origin",
        ),
        (
            Px8jInstallMalformation::UnwindRole,
            "recursor unwind role does not exit the invocation origin",
        ),
        (
            Px8jInstallMalformation::UnwindOrigin,
            "recursor unwind role does not exit the invocation origin",
        ),
    ] {
        let error = match run_px8j_source_machine_install(Some(malformation)) {
            Ok(_) => panic!("the unchecked source-machine install must validate before CFG"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "ComputationalRecursor",
                ref reason,
            }) if reason == expected_reason
        ));
    }
}

#[test]
fn px8j_source_machine_install_accepts_valid_unchecked_segment() {
    let installed = run_px8j_source_machine_install(None)
        .expect("a valid unchecked source-machine invocation still installs");
    assert!(matches!(
        installed,
        SourceContinuation::ApplyRecursorSelection { .. }
    ));
}

#[test]
fn oriented_open_control_obligations_are_affine_and_mint_exact() {
    let plan = oriented_test_ih_plan();
    let mut deleted = oriented_five_control_invocation();
    deleted
        .unwind
        .later_wrappers_in_construction_order
        .remove(0);
    let deleted = match compose_oriented_subcontinuation(
        Some(&plan),
        None,
        ContinuationActivationId(8),
        deleted,
        Vec::new(),
    ) {
        Ok(_) => panic!("deleting only an inherited exit obligation must reject"),
        Err(error) => error,
    };
    assert!(matches!(
        deleted,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason == "open control obligation set changed after affine mint"
    ));

    let mut duplicated = oriented_five_control_invocation();
    let duplicate = duplicated.unwind.later_wrappers_in_construction_order[0].clone();
    duplicated
        .unwind
        .later_wrappers_in_construction_order
        .push(duplicate);
    let duplicated = match compose_oriented_subcontinuation(
        Some(&plan),
        None,
        ContinuationActivationId(8),
        duplicated,
        Vec::new(),
    ) {
        Ok(_) => panic!("duplicating an inherited exit obligation must reject"),
        Err(error) => error,
    };
    assert!(matches!(
        duplicated,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason == "open control obligation set changed after affine mint"
    ));
}
#[test]
fn oriented_endpoint_corruption_and_affine_reuse_fail_closed() {
    let mut plan = oriented_test_plan();
    plan.frames[2].output_interface = oriented_test_interface(9);
    plan.frames[2].occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&plan.frames[2]);
    let error = match compose_oriented_subcontinuation(
        Some(&plan),
        None,
        ContinuationActivationId(8),
        oriented_test_invocation(),
        Vec::new(),
    ) {
        Ok(_) => panic!("endpoint corruption must reject before installation"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "OrientedSubcontinuationPlanV1",
            reason,
        }) if reason.contains("endpoints do not compose")
    ));

    let mut capability = AffineSpliceCapability {
        state: AffineSpliceState::Open,
    };
    capability.consume().unwrap();
    assert!(capability.consume().is_err());
}
fn oriented_five_control_invocation() -> RecursorInvocationSegment {
    let origin = RecursorProducerOriginId(40);
    let mut invocation = RecursorInvocationSegment::new(
        origin,
        0,
        oriented_test_instance_layer(
            2,
            0,
            0,
            true,
            RecursorLayerRole::SelectsOccurrence { origin },
        ),
        RecursorUnwindStack {
            later_wrappers_in_construction_order: vec![
                oriented_test_instance_layer(
                    2,
                    1,
                    0,
                    false,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(41),
                        parent_scope: None,
                    },
                ),
                oriented_test_instance_layer(
                    0,
                    1,
                    0,
                    false,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(42),
                        parent_scope: Some(RecursorProducerOriginId(41)),
                    },
                ),
                oriented_test_instance_layer(
                    0,
                    0,
                    0,
                    true,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(43),
                        parent_scope: Some(RecursorProducerOriginId(42)),
                    },
                ),
                oriented_test_instance_layer(
                    1,
                    0,
                    0,
                    true,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(44),
                        parent_scope: Some(RecursorProducerOriginId(43)),
                    },
                ),
            ],
        },
        ContinuationCursorId(7),
        None,
        None,
    );
    for layer in &mut invocation.unwind.later_wrappers_in_construction_order[..2] {
        layer.checked_invocation_source = Some(InvocationTemplateRef::SameSccCall(999));
    }
    invocation.selection.checked_invocation_source = None;
    for layer in &mut invocation.unwind.later_wrappers_in_construction_order {
        if layer.semantic_pending {
            layer.checked_invocation_source = None;
        }
    }
    invocation
}
#[test]
fn px8j_owned_scope_deletion_fails_closed_before_another_frame_is_emitted() {
    let expression = host_result_closure_match(px8j_layered_recursive_result(1, 1));
    let (exact_result, exact_trace) =
        px8j_capture_source_trace(&expression, false, "ken_px8j_scope_exact");
    exact_result.expect("the exact owned-scope path lowers");
    let (deleted_result, deleted_trace) =
        px8j_capture_source_trace(&expression, true, "ken_px8j_scope_deleted");
    let error = deleted_result.expect_err("deleting the owned scope must fail closed");
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "ComputationalRecursor",
            ref reason,
        }) if reason == "source recursor invocation is missing its owned selected scope"
    ));
    let deleted_terminal = deleted_trace
        .last()
        .expect("deletion must leave its terminal mint observation");
    let exact_terminal_index = exact_trace
        .iter()
        .position(|event| match (event, deleted_terminal) {
            (
                Px8jSourceTraceEvent::Mint {
                    path: exact_path,
                    origin: exact_origin,
                    cursor: exact_cursor,
                    siblings: exact_siblings,
                    ..
                },
                Px8jSourceTraceEvent::Mint {
                    path: deleted_path,
                    origin: deleted_origin,
                    cursor: deleted_cursor,
                    siblings: deleted_siblings,
                    ..
                },
            ) => {
                exact_path == deleted_path
                    && exact_origin == deleted_origin
                    && exact_cursor == deleted_cursor
                    && exact_siblings == deleted_siblings
            }
            _ => false,
        })
        .expect("the exact run reaches the deleted run's terminal mint");
    assert_eq!(
        &deleted_trace[..deleted_trace.len() - 1],
        &exact_trace[..exact_terminal_index]
    );
    assert!(matches!(
        (exact_trace.get(exact_terminal_index), deleted_trace.last()),
        (
            Some(Px8jSourceTraceEvent::Mint {
                path: exact_path,
                origin: exact_origin,
                cursor: exact_cursor,
                siblings: exact_siblings,
                parent_scope: Some(_),
            }),
            Some(Px8jSourceTraceEvent::Mint {
                path: deleted_path,
                origin: deleted_origin,
                cursor: deleted_cursor,
                siblings: deleted_siblings,
                parent_scope: None,
            }),
        ) if exact_path == deleted_path
            && exact_origin == deleted_origin
            && exact_cursor == deleted_cursor
            && exact_siblings == deleted_siblings
    ));
    let deleted_origin = match deleted_trace.last() {
        Some(Px8jSourceTraceEvent::Mint { origin, .. }) => *origin,
        event => panic!("deletion must stop immediately after the nested mint: {event:?}"),
    };
    assert!(!deleted_trace.iter().any(|event| matches!(
        event,
        Px8jSourceTraceEvent::Install { origin, .. }
            if *origin == deleted_origin
    )));
}
#[test]
fn px8j_all_three_producer_paths_reach_real_consumers() {
    let aggregate = RuntimeExpr::Construct {
        constructor: "ctor:prelude::Result::Ok".to_string(),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        }],
    };
    let expression = host_result_closure_match(recursive_computational_result_depth(2, aggregate));
    let (result, trace) =
        px8j_capture_source_trace(&expression, false, "ken_px8j_live_source_paths");
    result.expect("the composed and source-machine producer paths lower");
    for path in [Px8jProducerPath::Composed, Px8jProducerPath::SourceMachine] {
        let (origin, cursor) = trace
            .iter()
            .find_map(|event| match event {
                Px8jSourceTraceEvent::Mint {
                    path: actual,
                    origin,
                    cursor,
                    siblings,
                    ..
                } if *actual == path && *siblings > 0 => Some((*origin, *cursor)),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{path:?} must mint a recursive IH"));
        assert!(trace.iter().any(|event| matches!(
            event,
            Px8jSourceTraceEvent::Install {
                origin: actual_origin,
                selection_cursor,
                ..
            } if *actual_origin == origin && *selection_cursor == cursor
        )));
        assert!(trace.iter().any(|event| matches!(
            event,
            Px8jSourceTraceEvent::Selection { origin: actual } if *actual == origin
        )));
    }

    let deferred = host_result_closure_match(px8j_deferred_recursive_field_fixture());
    let (result, trace) =
        px8j_capture_source_trace(&deferred, false, "ken_px8j_live_deferred_path");
    result.expect("the deferred-constructor producer path lowers");
    let (origin, cursor) = trace
        .iter()
        .find_map(|event| match event {
            Px8jSourceTraceEvent::Mint {
                path: Px8jProducerPath::DeferredConstructor,
                origin,
                cursor,
                siblings: 1,
                ..
            } => Some((*origin, *cursor)),
            _ => None,
        })
        .expect("the deferred constructor mints its recursive IH");
    assert!(trace.iter().any(|event| matches!(
        event,
        Px8jSourceTraceEvent::DirectConsume {
            origin: actual_origin,
            selection_cursor,
            ..
        } if *actual_origin == origin && *selection_cursor == cursor
    )));
}
#[test]
fn px8j_siblings_share_an_origin_and_nested_ih_gets_a_child_origin() {
    let expression =
        host_result_closure_match(px8j_recursive_sibling_result(1, 2, px8j_aggregate_result()));
    let (result, trace) =
        px8j_capture_source_trace(&expression, false, "ken_px8j_live_sibling_origins");
    result.expect("the sibling and nested recursive IH path lowers");
    let (sibling_origin, sibling_cursor) = trace
        .iter()
        .find_map(|event| match event {
            Px8jSourceTraceEvent::Mint {
                origin,
                cursor,
                siblings: 2,
                ..
            } => Some((*origin, *cursor)),
            _ => None,
        })
        .expect("the selected case owns the sibling IH origin");
    let sibling_carriers: BTreeSet<_> = trace
        .iter()
        .filter_map(|event| match event {
            Px8jSourceTraceEvent::Carrier {
                origin,
                cursor,
                sibling_position,
                ..
            } if *origin == sibling_origin && *cursor == sibling_cursor => Some(*sibling_position),
            _ => None,
        })
        .collect();
    assert_eq!(sibling_carriers, BTreeSet::from([0, 1]));
    let sibling_consumers: BTreeSet<_> = trace
        .iter()
        .filter_map(|event| match event {
            Px8jSourceTraceEvent::Install {
                origin,
                selection_cursor,
                sibling_position,
                ..
            } if *origin == sibling_origin && *selection_cursor == sibling_cursor => {
                Some(*sibling_position)
            }
            _ => None,
        })
        .collect();
    assert_eq!(sibling_consumers, sibling_carriers);
    assert!(
        trace.iter().any(|event| matches!(
            event,
            Px8jSourceTraceEvent::Mint {
                origin,
                parent_scope: Some(parent),
                ..
            } if *origin != sibling_origin && *parent == sibling_origin
        )),
        "{trace:#?}"
    );
}
fn px8j_capture_source_trace(
    expression: &RuntimeExpr,
    delete_owned_scope: bool,
    symbol: &str,
) -> (
    Result<CraneliftObjectArtifact, CraneliftBackendError>,
    Vec<Px8jSourceTraceEvent>,
) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            PX8J_DELETE_OWNED_SELECTED_SCOPE.set(false);
            PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().clear());
        }
    }
    PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().clear());
    PX8J_DELETE_OWNED_SELECTED_SCOPE.set(delete_owned_scope);
    let _reset = Reset;
    let result = emit_process_entrypoint_object_with_cranelift(expression, symbol);
    let trace = PX8J_SOURCE_TRACE.with(|trace| trace.borrow().clone());
    (result, trace)
}
#[test]
fn oriented_phase_misclassification_recovers_endpoint_and_missing_semantic_rejections() {
    let plan = oriented_test_ih_plan();
    let mut replayed = oriented_five_control_invocation();
    replayed.unwind.later_wrappers_in_construction_order[0].semantic_pending = true;
    replayed.open_control_obligations = open_control_obligations(&replayed.unwind);
    let replayed = match compose_oriented_subcontinuation(
        Some(&plan),
        None,
        ContinuationActivationId(8),
        replayed,
        Vec::new(),
    ) {
        Ok(_) => panic!("an inherited open scope cannot replay its semantic transformer"),
        Err(error) => error,
    };
    assert!(matches!(
        replayed,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason.contains("exact invocation-local tree")
    ));

    let mut omitted = oriented_five_control_invocation();
    omitted.selection.semantic_pending = false;
    let omitted = match compose_oriented_subcontinuation(
        Some(&plan),
        None,
        ContinuationActivationId(8),
        omitted,
        Vec::new(),
    ) {
        Ok(_) => panic!("a pending selection cannot be omitted from semantic work"),
        Err(error) => error,
    };
    assert!(matches!(
        omitted,
        CraneliftBackendError::Unsupported(UnsupportedLowering { reason, .. })
            if reason == "pending selection was misclassified as control-only"
    ));
}
#[test]
fn nested_computational_inner_missing_selects_exact_inner_default() {
    let inner_cases = vec![crate::RuntimeComputationalMatchCase {
        constructor: "ctor:fixture::Inner::Hit".to_string(),
        argument_binders: 0,
        recursive_positions: Vec::new(),
        body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
    }];
    let outer_cases = vec![crate::RuntimeComputationalMatchCase {
        constructor: "ctor:fixture::Outer::Hit".to_string(),
        argument_binders: 0,
        recursive_positions: Vec::new(),
        body: RuntimeExpr::Value(RuntimeValue::Int((2).into())),
    }];
    let inner_default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px7n exact inner default".to_string(),
    };
    let outer_default = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "px7n exact outer default".to_string(),
    };
    let frames = [
        ComputationalEliminatorFrame {
            cases: &inner_cases,
            default: &inner_default,
            env: &[],
            static_origin: inert_test_static_origin(),
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
            provenance: RecursorFrameProvenance(1),
            checked_frame_id: None,
            checked_invocation_id: None,
            checked_invocation_source: None,
            checked_invocation_depth: 0,
        },
        ComputationalEliminatorFrame {
            cases: &outer_cases,
            default: &outer_default,
            env: &[],
            static_origin: inert_test_static_origin(),
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
            provenance: RecursorFrameProvenance(0),
            checked_frame_id: None,
            checked_invocation_id: None,
            checked_invocation_source: None,
            checked_invocation_depth: 0,
        },
    ];

    let trap = match select_computational_case(&frames, "ctor:fixture::Inner::Missing") {
        Err(trap) => trap,
        Ok(_) => panic!("a missing inner case must select the inner frame default"),
    };
    assert_eq!(trap.code, RuntimeTrapCode::PatternMatchFailure);
    assert_eq!(trap.message, "px7n exact inner default");
    assert_ne!(trap.code, outer_default.code);
    assert_ne!(trap.message, outer_default.message);
}
#[test]
fn unmarked_equal_shape_frame_cannot_consume_retained_join_site() {
    let cases = vec![RuntimeMatchCase {
        constructor: "ctor:fixture::PX8H::Only".to_string(),
        binders: 0,
        body: RuntimeExpr::Value(RuntimeValue::Int((7).into())),
    }];
    let default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px8h unmarked equal-shape default".to_string(),
    };
    let fingerprint = crate::compiler_private_ordinary_match_frame_fingerprint(&cases, &default);
    let expression = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::PX8H::Only".to_string(),
            args: Vec::new(),
        }),
        cases,
        default,
    };
    let result = compile_expr_into_module(
        new_object_module("px8h-unmarked-equal-shape").unwrap(),
        "ken_px8h_unmarked_equal_shape",
        Linkage::Export,
        &expression,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        false,
        None,
        Some(crate::NativeJoinPlanV1 {
            representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: vec![self_consistent_join_site(51, fingerprint)],
        }),
        None,
    );
    let error = match result {
        Ok(_) => panic!("an unmarked equal-shape frame must not consume a plan row"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "NativeJoinPlanV1",
            reason,
        }) if reason.contains("unconsumed or orphan site")
    ));
}
fn px8j_scope_chain_observation_result(transform_layers: usize, input_depth: usize) -> RuntimeExpr {
    let tree_constructor =
        |_layer: usize, constructor: &str| format!("ctor:fixture::PX8JScopeTree::{constructor}");
    fn child(depth: usize, node: &str, leaf: &str) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["unit".to_string()],
            body: Box::new(if depth == 0 {
                RuntimeExpr::Construct {
                    constructor: leaf.to_string(),
                    args: Vec::new(),
                }
            } else {
                RuntimeExpr::Construct {
                    constructor: node.to_string(),
                    args: vec![child(depth - 1, node, leaf)],
                }
            }),
        }
    }
    let input_node = tree_constructor(0, "Node");
    let input_leaf = tree_constructor(0, "Leaf");
    let mut producer = RuntimeExpr::Construct {
        constructor: input_node.clone(),
        args: vec![child(input_depth, &input_node, &input_leaf)],
    };
    for layer in 0..transform_layers {
        producer = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(producer),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(layer, "Node"),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Construct {
                        constructor: tree_constructor(layer + 1, "Node"),
                        args: vec![RuntimeExpr::Var(0)],
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(layer, "Leaf"),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: tree_constructor(layer + 1, "Leaf"),
                        args: Vec::new(),
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: format!("PX8-J transform {layer} default"),
            },
        };
    }
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(producer),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: tree_constructor(transform_layers, "Node"),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Let {
                    value: Box::new(RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                            args: Vec::new(),
                        }],
                    }),
                    body: Box::new(RuntimeExpr::Construct {
                        constructor: tree_constructor(transform_layers, "Node"),
                        args: vec![child(
                            0,
                            &tree_constructor(transform_layers, "Node"),
                            &tree_constructor(transform_layers, "Leaf"),
                        )],
                    }),
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: tree_constructor(transform_layers, "Leaf"),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Construct {
                    constructor: tree_constructor(transform_layers, "Leaf"),
                    args: Vec::new(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-J terminal transform default".to_string(),
        },
    }
}
fn px8j_recursive_sibling_result(
    depth: usize,
    siblings: usize,
    leaf_body: RuntimeExpr,
) -> RuntimeExpr {
    assert!(siblings > 0);
    let node = "ctor:fixture::PX8JSiblingTree::Node";
    let leaf = "ctor:fixture::PX8JSiblingTree::Leaf";
    fn child(depth: usize, siblings: usize, node: &str, leaf: &str) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["unit".to_string()],
            body: Box::new(if depth == 0 {
                RuntimeExpr::Construct {
                    constructor: leaf.to_string(),
                    args: Vec::new(),
                }
            } else {
                RuntimeExpr::Construct {
                    constructor: node.to_string(),
                    args: (0..siblings)
                        .map(|_| child(depth - 1, siblings, node, leaf))
                        .collect(),
                }
            }),
        }
    }
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: node.to_string(),
            args: (0..siblings)
                .map(|_| child(depth, siblings, node, leaf))
                .collect(),
        }),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: node.to_string(),
                argument_binders: siblings,
                recursive_positions: (0..siblings).collect(),
                body: if siblings == 1 {
                    RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![RuntimeExpr::Construct {
                            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                            args: Vec::new(),
                        }],
                    }
                } else {
                    RuntimeExpr::Let {
                        value: Box::new(RuntimeExpr::Call {
                            callee: Box::new(RuntimeExpr::Var(0)),
                            args: vec![RuntimeExpr::Construct {
                                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                args: Vec::new(),
                            }],
                        }),
                        body: Box::new(RuntimeExpr::Call {
                            callee: Box::new(RuntimeExpr::Var(2)),
                            args: vec![RuntimeExpr::Construct {
                                constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                args: Vec::new(),
                            }],
                        }),
                    }
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: leaf.to_string(),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: leaf_body,
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-J sibling tree default".to_string(),
        },
    }
}
fn oriented_test_invocation() -> RecursorInvocationSegment {
    let origin = RecursorProducerOriginId(40);
    RecursorInvocationSegment::new(
        origin,
        0,
        oriented_test_layer(0, RecursorLayerRole::SelectsOccurrence { origin }),
        RecursorUnwindStack {
            later_wrappers_in_construction_order: vec![
                oriented_test_layer(
                    1,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(41),
                        parent_scope: None,
                    },
                ),
                oriented_test_layer(
                    2,
                    RecursorLayerRole::ExitsScope {
                        origin,
                        scope_origin: RecursorProducerOriginId(42),
                        parent_scope: Some(RecursorProducerOriginId(41)),
                    },
                ),
            ],
        },
        ContinuationCursorId(7),
        None,
        None,
    )
}
#[test]
fn px8j_one_two_three_scope_segments_reach_selection_hole_and_unwind() {
    for depth in 1..=3 {
        let expression = host_result_closure_match(px8j_scope_chain_observation_result(depth, 0));
        let (result, trace) = px8j_capture_source_trace(
            &expression,
            false,
            &format!("ken_px8j_live_scope_depth_{depth}"),
        );
        result.unwrap_or_else(|error| panic!("scope depth {depth} must lower: {error:?}"));
        let (origin, cursor, exits) = trace
            .iter()
            .find_map(|event| match event {
                Px8jSourceTraceEvent::Install {
                    origin,
                    selection_cursor,
                    exits,
                    ..
                } if exits.len() == depth => Some((*origin, *selection_cursor, exits)),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!("scope depth {depth} must install one exact segment: {trace:#?}")
            });
        let unique_scope_origins: BTreeSet<_> = exits
            .iter()
            .map(|(scope_origin, _)| *scope_origin)
            .collect();
        assert_eq!(unique_scope_origins.len(), depth);
        assert_eq!(exits.first().and_then(|(_, parent)| *parent), None);
        for pair in exits.windows(2) {
            let (outer_scope, _) = pair[0];
            let (_, inner_parent) = pair[1];
            assert_eq!(inner_parent, Some(outer_scope));
        }
        let selection = trace
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Px8jSourceTraceEvent::Selection { origin: actual } if *actual == origin
                )
            })
            .expect("selection is consumed");
        let hole = trace
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Px8jSourceTraceEvent::ReturnHole { cursor: actual } if *actual == cursor
                )
            })
            .expect("the complete caller source K reaches its return hole");
        let first_exit = trace
            .iter()
            .position(|event| {
                matches!(
                    event,
                    Px8jSourceTraceEvent::Exit { origin: actual, .. } if *actual == origin
                )
            })
            .expect("the installed unwind stack begins consumption");
        assert!(selection < hole && hole < first_exit);
        let consumed_exits: Vec<_> = trace[hole + 1..]
            .iter()
            .filter_map(|event| match event {
                Px8jSourceTraceEvent::Exit {
                    origin: actual_origin,
                    scope_origin,
                    parent_scope,
                } if *actual_origin == origin => Some((*scope_origin, *parent_scope)),
                _ => None,
            })
            .collect();
        assert_eq!(
            consumed_exits,
            exits.iter().rev().copied().collect::<Vec<_>>(),
            "depth {depth}: {trace:#?}"
        );
    }
}
#[test]
fn px8j_selected_scope_partitions_differ_across_the_real_return_hole() {
    let before = host_result_closure_match(px8j_equal_payload_hole_placement(
        Px8jSelectedScopePlacement::BeforeReturnHole,
    ));
    let after = host_result_closure_match(px8j_equal_payload_hole_placement(
        Px8jSelectedScopePlacement::AfterReturnHole,
    ));
    let (before_result, before_trace) =
        px8j_capture_source_trace(&before, false, "ken_px8j_scope_before_hole");
    let (after_result, after_trace) =
        px8j_capture_source_trace(&after, false, "ken_px8j_scope_after_hole");
    before_result.expect("the before-hole selected scope lowers");
    after_result.expect("the after-hole selected scope lowers");

    let partition = |trace: &[Px8jSourceTraceEvent]| {
        let hole = trace
            .iter()
            .position(|event| matches!(event, Px8jSourceTraceEvent::ReturnHole { .. }))
            .expect("the real source path reaches its return hole");
        let selections_before = trace[..hole]
            .iter()
            .filter(|event| matches!(event, Px8jSourceTraceEvent::Selection { .. }))
            .count();
        let exits_after = trace[hole + 1..]
            .iter()
            .filter(|event| matches!(event, Px8jSourceTraceEvent::Exit { .. }))
            .count();
        (selections_before, exits_after)
    };
    assert_eq!(partition(&before_trace), (2, 0));
    assert_eq!(partition(&after_trace), (1, 1));
}
#[test]
fn nested_computational_outer_missing_selects_exact_outer_default() {
    let inner_cases = vec![crate::RuntimeComputationalMatchCase {
        constructor: "ctor:fixture::Inner::Hit".to_string(),
        argument_binders: 0,
        recursive_positions: Vec::new(),
        body: RuntimeExpr::Value(RuntimeValue::Int((1).into())),
    }];
    let outer_cases = vec![crate::RuntimeComputationalMatchCase {
        constructor: "ctor:fixture::Outer::Hit".to_string(),
        argument_binders: 0,
        recursive_positions: Vec::new(),
        body: RuntimeExpr::Value(RuntimeValue::Int((2).into())),
    }];
    let inner_default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "px7n exact inner default".to_string(),
    };
    let outer_default = RuntimeTrap {
        code: RuntimeTrapCode::ExplicitTrap,
        message: "px7n exact outer default".to_string(),
    };
    let frames = [
        ComputationalEliminatorFrame {
            cases: &inner_cases,
            default: &inner_default,
            env: &[],
            static_origin: inert_test_static_origin(),
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
            provenance: RecursorFrameProvenance(1),
            checked_frame_id: None,
            checked_invocation_id: None,
            checked_invocation_source: None,
            checked_invocation_depth: 0,
        },
        ComputationalEliminatorFrame {
            cases: &outer_cases,
            default: &outer_default,
            env: &[],
            static_origin: inert_test_static_origin(),
            retained_scrutinee_index: None,
            deferred_constructor_case: None,
            provenance: RecursorFrameProvenance(0),
            checked_frame_id: None,
            checked_invocation_id: None,
            checked_invocation_source: None,
            checked_invocation_depth: 0,
        },
    ];

    let (_, _, outer_frames) = select_computational_case(&frames, "ctor:fixture::Inner::Hit")
        .expect("the inner case succeeds before the outer miss");
    let trap = match select_computational_case(outer_frames, "ctor:fixture::Outer::Missing") {
        Err(trap) => trap,
        Ok(_) => panic!("a missing outer case must select the outer frame default"),
    };
    assert_eq!(trap.code, RuntimeTrapCode::ExplicitTrap);
    assert_eq!(trap.message, "px7n exact outer default");
    assert_ne!(trap.code, inner_default.code);
    assert_ne!(trap.message, inner_default.message);
}
#[test]
fn distinguished_root_cannot_discharge_missing_match_site_marker() {
    let seed_env = NativeSeedEnvironment::empty();
    let mut lowering = Lowering {
        seed_env: &seed_env,
        seed_material: crate::cranelift_backend::lowering::seed_material::SeedMaterialRefs::none_for_tests(),
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
        native_join_plan: Some(crate::NativeJoinPlanV1 {
            representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: vec![self_consistent_root_join_site(0)],
        }),
        consumed_join_sites: BTreeSet::new(),
        root_terminal_authority: None,
        active_join_site: Some(41),
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
    let error = lowering
        .planned_join_site_for_frame(EliminatorFrame::InvocationReturn)
        .expect_err("the distinguished root must not discharge an unrelated live marker");
    assert!(
        matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                ref reason,
            }) if reason.contains("root cannot consume an active match occurrence marker")
        ),
        "{error:?}"
    );
    assert_eq!(lowering.active_join_site, Some(41));
    assert!(lowering.consumed_join_sites.is_empty());
}
#[test]
fn oriented_segment_keeps_semantic_and_control_axes_independent() {
    let installed = compose_oriented_subcontinuation(
        Some(&oriented_test_plan()),
        None,
        ContinuationActivationId(8),
        oriented_test_invocation(),
        Vec::new(),
    )
    .unwrap();
    assert_eq!(
        installed
            .semantic_frames
            .iter()
            .map(|frame| frame.checked_frame_id.unwrap())
            .collect::<Vec<_>>(),
        vec![2, 1, 0],
        "checked composition order is p2, p1, p0"
    );
    assert_eq!(
        installed
            .control_ledger
            .iter()
            .map(|entry| entry.frame_id.unwrap())
            .collect::<Vec<_>>(),
        vec![0, 2, 1],
        "delimiter order remains independently o0, o4, o3"
    );
}
#[derive(Clone, Copy)]
enum Px8jSelectedScopePlacement {
    BeforeReturnHole,
    AfterReturnHole,
}
fn px8j_aggregate_result() -> RuntimeExpr {
    RuntimeExpr::Construct {
        constructor: "ctor:prelude::Result::Ok".to_string(),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        }],
    }
}
#[test]
fn oriented_fresh_ih_semantics_retain_all_inherited_control_obligations() {
    let installed = compose_oriented_subcontinuation(
        Some(&oriented_test_ih_plan()),
        None,
        ContinuationActivationId(8),
        oriented_five_control_invocation(),
        Vec::new(),
    )
    .unwrap();
    assert_eq!(
        installed
            .semantic_frames
            .iter()
            .map(|frame| {
                (
                    frame.checked_invocation_id.unwrap(),
                    frame.checked_frame_id.unwrap(),
                )
            })
            .collect::<Vec<_>>(),
        vec![(0, 2), (0, 1), (0, 0)],
    );
    assert_eq!(installed.control_ledger.len(), 5);
    assert_eq!(
        installed
            .control_ledger
            .iter()
            .filter(|entry| matches!(entry.role, RecursorLayerRole::ExitsScope { .. }))
            .count(),
        4,
    );
}
fn px8j_deferred_recursive_field_fixture() -> RuntimeExpr {
    let wrap = "ctor:fixture::PX8JDeferred::Wrap";
    let done = "ctor:fixture::PX8JDeferred::Done";
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: wrap.to_string(),
            args: vec![
                RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["unit".to_string()],
                    body: Box::new(RuntimeExpr::Construct {
                        constructor: done.to_string(),
                        args: Vec::new(),
                    }),
                },
                constructor_field_aggregate(),
            ],
        }),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: wrap.to_string(),
                argument_binders: 2,
                recursive_positions: vec![0],
                body: RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Var(2)),
                    cases: ["ctor:prelude::Result::Err", "ctor:prelude::Result::Ok"]
                        .into_iter()
                        .map(|constructor| RuntimeMatchCase {
                            constructor: constructor.to_string(),
                            binders: 1,
                            body: RuntimeExpr::Call {
                                callee: Box::new(RuntimeExpr::Var(1)),
                                args: vec![RuntimeExpr::Construct {
                                    constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                                    args: Vec::new(),
                                }],
                            },
                        })
                        .collect(),
                    default: RuntimeTrap {
                        code: RuntimeTrapCode::PatternMatchFailure,
                        message: "PX8-J deferred selected-field default".to_string(),
                    },
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: done.to_string(),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Construct {
                    constructor: "ctor:prelude::Result::Ok".to_string(),
                    args: vec![RuntimeExpr::Construct {
                        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                        args: Vec::new(),
                    }],
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-J deferred outer default".to_string(),
        },
    }
}
fn px8j_layered_recursive_result(transform_layers: usize, input_depth: usize) -> RuntimeExpr {
    let tree_constructor =
        |layer: usize, constructor: &str| format!("ctor:fixture::PX8JTree{layer}::{constructor}");
    let unit = || RuntimeExpr::Construct {
        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
        args: Vec::new(),
    };
    let aggregate = || RuntimeExpr::Construct {
        constructor: "ctor:prelude::Result::Ok".to_string(),
        args: vec![unit()],
    };
    fn child(depth: usize, node: &str, leaf: &str) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["unit".to_string()],
            body: Box::new(if depth == 0 {
                RuntimeExpr::Construct {
                    constructor: leaf.to_string(),
                    args: Vec::new(),
                }
            } else {
                RuntimeExpr::Construct {
                    constructor: node.to_string(),
                    args: vec![child(depth - 1, node, leaf)],
                }
            }),
        }
    }
    let input_node = tree_constructor(0, "Node");
    let input_leaf = tree_constructor(0, "Leaf");
    let mut producer = RuntimeExpr::Construct {
        constructor: input_node.clone(),
        args: vec![child(input_depth, &input_node, &input_leaf)],
    };
    for layer in 0..transform_layers {
        producer = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(producer),
            cases: vec![
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(layer, "Node"),
                    argument_binders: 1,
                    recursive_positions: vec![0],
                    body: RuntimeExpr::Construct {
                        constructor: tree_constructor(layer + 1, "Node"),
                        args: vec![RuntimeExpr::Var(0)],
                    },
                },
                crate::RuntimeComputationalMatchCase {
                    constructor: tree_constructor(layer, "Leaf"),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: RuntimeExpr::Construct {
                        constructor: tree_constructor(layer + 1, "Leaf"),
                        args: Vec::new(),
                    },
                },
            ],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: format!("PX8-J transform {layer} default"),
            },
        };
    }
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(producer),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: tree_constructor(transform_layers, "Node"),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![unit()],
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: tree_constructor(transform_layers, "Leaf"),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: aggregate(),
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-J terminal transform default".to_string(),
        },
    }
}
fn px8j_equal_payload_hole_placement(placement: Px8jSelectedScopePlacement) -> RuntimeExpr {
    let input_node = "ctor:fixture::PX8JHoleInput::Node";
    let input_leaf = "ctor:fixture::PX8JHoleInput::Leaf";
    let output_node = "ctor:fixture::PX8JHoleOutput::Node";
    let output_leaf = "ctor:fixture::PX8JHoleOutput::Leaf";
    let unit = || RuntimeExpr::Construct {
        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
        args: Vec::new(),
    };
    let recursive_child = || RuntimeExpr::LexicalClosure {
        captures: Vec::new(),
        params: vec!["unit".to_string()],
        body: Box::new(RuntimeExpr::Construct {
            constructor: input_leaf.to_string(),
            args: Vec::new(),
        }),
    };
    let scoped_payload = || RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: input_node.to_string(),
            args: vec![recursive_child()],
        }),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: input_node.to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Construct {
                    constructor: output_node.to_string(),
                    args: vec![RuntimeExpr::Var(0)],
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: input_leaf.to_string(),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::Construct {
                    constructor: output_leaf.to_string(),
                    args: Vec::new(),
                },
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-J equal-payload inner default".to_string(),
        },
    };
    let outer_scrutinee = match placement {
        Px8jSelectedScopePlacement::BeforeReturnHole => RuntimeExpr::Construct {
            constructor: output_node.to_string(),
            args: vec![RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["unit".to_string()],
                body: Box::new(scoped_payload()),
            }],
        },
        Px8jSelectedScopePlacement::AfterReturnHole => scoped_payload(),
    };
    RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(outer_scrutinee),
        cases: vec![
            crate::RuntimeComputationalMatchCase {
                constructor: output_node.to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![unit()],
                },
            },
            crate::RuntimeComputationalMatchCase {
                constructor: output_leaf.to_string(),
                argument_binders: 0,
                recursive_positions: Vec::new(),
                body: px8j_aggregate_result(),
            },
        ],
        default: RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "PX8-J equal-payload outer default".to_string(),
        },
    }
}
#[cfg(test)]
fn oriented_test_ih_plan() -> crate::OrientedSubcontinuationPlanV1 {
    let mut plan = oriented_test_plan();
    for frame_id in 0..=2 {
        let slot_template_id = 200 + frame_id;
        let mut slot = crate::CheckedComputationalIHSlotTemplateV1 {
            slot_template_id,
            declaration: "decl:fixture::oriented".to_string(),
            checked_match_ordinal: frame_id,
            checked_occurrence_path: vec![20, frame_id],
            frame_template_id: frame_id,
            constructor: format!("Ctor{frame_id}"),
            recursive_position: 0,
            method_binder_ordinal: 0,
            local_telescope: Vec::new(),
            ih_interface: oriented_test_interface(frame_id as u8),
            segment_site_id: 9,
            frame_templates: vec![frame_id],
            input_interface: oriented_test_interface(frame_id as u8),
            output_interface: oriented_test_interface(frame_id as u8 + 1),
            runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
                declaration: "decl:fixture::oriented".to_string(),
                runtime_path: vec![0, frame_id],
            }],
            occurrence_binding_fingerprint: 0,
        };
        slot.occurrence_binding_fingerprint =
            crate::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
        plan.computational_ih_slots.push(slot);

        let mut call = crate::CheckedComputationalIHCallTemplateV1 {
            call_template_id: 100 + frame_id,
            declaration: "decl:fixture::oriented".to_string(),
            checked_occurrence_path: vec![30, frame_id],
            slot_template_id,
            arity: 1,
            local_telescope: Vec::new(),
            result_interface: oriented_test_interface(frame_id as u8 + 1),
            callee_segment_site_id: 9,
            callee_frame_templates: vec![frame_id],
            parent_frame_template_id: Some(frame_id),
            parent_segment_site_id: Some(9),
            caller_interface: oriented_test_interface(frame_id as u8 + 1),
            runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
                declaration: "decl:fixture::oriented".to_string(),
                runtime_path: vec![1, frame_id],
            }],
            occurrence_binding_fingerprint: 0,
        };
        call.occurrence_binding_fingerprint =
            crate::compiler_private_computational_ih_call_binding_fingerprint(&call);
        plan.computational_ih_calls.push(call);
    }
    plan.validate().unwrap();
    plan
}
#[cfg(test)]
fn oriented_test_instance_layer(
    frame_id: u64,
    invocation_id: u64,
    semantic_depth: usize,
    semantic_pending: bool,
    role: RecursorLayerRole,
) -> ComputationalRecursorLayer {
    let mut layer = oriented_test_layer(frame_id, role);
    layer.checked_invocation_id = Some(invocation_id);
    layer.checked_invocation_source =
        Some(InvocationTemplateRef::ComputationalIHCall(100 + frame_id));
    layer.checked_invocation_depth = semantic_depth;
    layer.semantic_pending = semantic_pending;
    layer
}
#[cfg(test)]
fn oriented_test_layer(frame_id: u64, role: RecursorLayerRole) -> ComputationalRecursorLayer {
    ComputationalRecursorLayer {
        cases: Vec::new(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: format!("oriented frame {frame_id}"),
        },
        outer_env: Vec::new(),
        static_origin: inert_test_static_origin(),
        provenance: RecursorFrameProvenance(frame_id),
        role,
        checked_frame_id: Some(frame_id),
        checked_invocation_id: Some(0),
        checked_invocation_source: None,
        checked_invocation_depth: 0,
        semantic_pending: true,
    }
}
#[cfg(test)]
fn oriented_test_plan() -> crate::OrientedSubcontinuationPlanV1 {
    crate::OrientedSubcontinuationPlanV1 {
        representation_rule_version:
            crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
        // Checked postorder is p2, p1, p0 even though control returns
        // through o0, o4, o3 below.
        frames: vec![
            oriented_test_frame(0, 2, 2, 3, None),
            oriented_test_frame(1, 1, 1, 2, Some(0)),
            oriented_test_frame(2, 0, 0, 1, Some(1)),
        ],
        recursive_calls: Vec::new(),
        computational_ih_slots: Vec::new(),
        computational_ih_calls: Vec::new(),
    }
}
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum Px8jDirectRecursorConsumer {
    PendingLetProducer,
    ProducerCall,
    OrdinaryCall,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum Px8jRecursorMalformation {
    SelectionRole,
    RepeatedScopeIdentity,
    BrokenScopeParent,
}

#[test]
fn recursive_declaration_shape_change_hits_typed_boundary() {
    let symbol = "decl:fixture::Loop::run".to_string();
    let declaration = RuntimeDeclaration {
        symbol: symbol.clone(),
        kind: RuntimeDeclarationKind::Transparent {
            body: RuntimeExpr::Closure {
                captures: Vec::new(),
                params: vec!["state".to_string()],
                body: Box::new(RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::DeclarationRef {
                        symbol: symbol.clone(),
                    }),
                    args: vec![RuntimeExpr::Construct {
                        constructor: "ctor:fixture::Option::Some".to_string(),
                        args: vec![RuntimeExpr::Value(RuntimeValue::Int((1).into()))],
                    }],
                }),
            },
        },
        metadata: RuntimeSymbolMetadata {
            lowerability: Some(RuntimeLowerabilityStatus::Supported),
            ..RuntimeSymbolMetadata::empty()
        },
    };
    let entry = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::DeclarationRef {
            symbol: symbol.clone(),
        }),
        args: vec![RuntimeExpr::Construct {
            constructor: "ctor:fixture::Option::None".to_string(),
            args: Vec::new(),
        }],
    };
    let declarations = BTreeMap::from([(symbol.as_str(), &declaration)]);
    let result = compile_expr_into_module(
        new_object_module("px8l-recursive-shape").unwrap(),
        "ken_px8l_recursive_shape",
        Linkage::Export,
        &entry,
        &NativeSeedEnvironment::empty(),
        declarations,
        None,
        true,
        None,
        Some(test_only_distinguished_root_join_plan()),
        None,
    );
    let error = match result {
        Ok(_) => panic!("a changing recursive native representation must fail closed"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "DeclarationRef",
            reason,
        }) if reason.contains("changes its native argument representation")
    ));
}
#[test]
fn checked_join_marker_without_exact_plan_site_rejects_before_emission() {
    let expression = RuntimeExpr::CheckedJoinSite {
        site_id: 41,
        body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((7).into()))),
    };
    let result = compile_expr_into_module(
        new_object_module("px8h-missing-join-site").unwrap(),
        "ken_px8h_missing_join_site",
        Linkage::Export,
        &expression,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        false,
        None,
        None,
        None,
    );
    let error = match result {
        Ok(_) => panic!("a live checked occurrence without its plan site must reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "NativeJoinPlanV1",
            reason,
        }) if reason.contains("marker was not consumed")
    ));
}
#[test]
fn process_lowering_without_checked_root_authority_rejects_before_cfg() {
    let result = compile_expr_into_module(
        new_object_module("px8ta-missing-root-authority").unwrap(),
        "ken_px8ta_missing_root_authority",
        Linkage::Export,
        &RuntimeExpr::Construct {
            constructor: crate::EXIT_SUCCESS_CONSTRUCTOR.to_string(),
            args: Vec::new(),
        },
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        None,
        None,
        None,
    );
    let error = match result {
        Ok(_) => panic!("process lowering must not invent root authority from process mode"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "NativeJoinPlanV1",
            reason,
        }) if reason == "process-object lowering has no checked distinguished-root answer authority"
    ));
}
#[test]
fn checked_marker_census_rejects_duplicate_call_and_slot_occurrences_before_cfg() {
    let (entry, declaration, plan) = occurrence_exact_marker_fixture(false, false);
    let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
    validate_oriented_subcontinuation_transport(&entry, &declarations, Some(&plan))
        .expect("the exact checked Runtime marker occurrence ledger closes");

    for (duplicate_call, duplicate_slot, expected) in [
        (
            true,
            false,
            "computational-IH call Runtime occurrences differ",
        ),
        (
            false,
            true,
            "computational-IH slot Runtime occurrences differ",
        ),
    ] {
        let (entry, declaration, plan) =
            occurrence_exact_marker_fixture(duplicate_call, duplicate_slot);
        let declarations = BTreeMap::from([(declaration.symbol.as_str(), &declaration)]);
        let error = validate_oriented_subcontinuation_transport(&entry, &declarations, Some(&plan))
            .expect_err("an extra static marker occurrence must reject before CFG emission");
        assert!(
            matches!(
                error,
                CraneliftBackendError::Unsupported(UnsupportedLowering {
                    construct: "OrientedSubcontinuationPlanV1",
                    ref reason,
                }) if reason.contains(expected)
            ),
            "{error:?}"
        );
    }
}
#[test]
fn valid_root_plus_missing_marked_scalar_cut_rejects_before_emission() {
    let expression = RuntimeExpr::CheckedJoinSite {
        site_id: 41,
        body: Box::new(host_result_computational_fixture(1, true, false)),
    };
    let symbols = crate::NativeProcessSymbols::legacy_prelude();
    let result = compile_expr_into_module(
        new_object_module("px8h-root-marker-class-separation").unwrap(),
        "ken_px8h_root_marker_class_separation",
        Linkage::Export,
        &expression,
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        true,
        Some(&symbols),
        Some(crate::NativeJoinPlanV1 {
            representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: vec![self_consistent_root_join_site(0)],
        }),
        None,
    );
    let error = match result {
        Ok(_) => panic!("the root must not discharge a missing marked scalar-cut site"),
        Err(error) => error,
    };
    assert!(
        matches!(
            error,
            CraneliftBackendError::Unsupported(UnsupportedLowering {
                construct: "NativeJoinPlanV1",
                ref reason,
            }) if reason.contains("marker was not consumed")
        ),
        "{error:?}"
    );
}
#[test]
fn self_consistent_appended_orphan_join_site_rejects_before_emission() {
    let result = compile_expr_into_module(
        new_object_module("px8h-orphan-join-site").unwrap(),
        "ken_px8h_orphan_join_site",
        Linkage::Export,
        &RuntimeExpr::Value(RuntimeValue::Int((7).into())),
        &NativeSeedEnvironment::empty(),
        BTreeMap::new(),
        None,
        false,
        None,
        Some(crate::NativeJoinPlanV1 {
            representation_rule_version: crate::NativeJoinPlanV1::REPRESENTATION_RULE_VERSION,
            sites: vec![
                self_consistent_root_join_site(0),
                self_consistent_join_site(52, 23),
            ],
        }),
        None,
    );
    let error = match result {
        Ok(_) => panic!("a self-consistent orphan plan row must reject"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        CraneliftBackendError::Unsupported(UnsupportedLowering {
            construct: "NativeJoinPlanV1",
            reason,
        }) if reason.contains("unconsumed or orphan site")
    ));
}
fn occurrence_exact_marker_fixture(
    duplicate_call: bool,
    duplicate_slot: bool,
) -> (
    RuntimeExpr,
    RuntimeDeclaration,
    crate::OrientedSubcontinuationPlanV1,
) {
    let declaration = "decl:fixture::PX8TA::markers".to_string();
    let slot_marker = RuntimeExpr::CheckedComputationalIHSlots {
        slot_template_ids: vec![200],
        checked_occurrence_paths: vec![vec![20]],
        body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((1).into()))),
    };
    let call_marker = RuntimeExpr::CheckedComputationalIHInvocation {
        call_template_id: 100,
        checked_occurrence_path: vec![30],
        body: Box::new(RuntimeExpr::Value(RuntimeValue::Int((2).into()))),
    };
    let slot_value = if duplicate_slot {
        RuntimeExpr::Construct {
            constructor: "ctor:fixture::Pair".to_string(),
            args: vec![slot_marker.clone(), slot_marker],
        }
    } else {
        slot_marker
    };
    let call_body = if duplicate_call {
        RuntimeExpr::Construct {
            constructor: "ctor:fixture::Pair".to_string(),
            args: vec![call_marker.clone(), call_marker],
        }
    } else {
        call_marker
    };
    let cases = vec![crate::RuntimeComputationalMatchCase {
        constructor: "ctor:fixture::Only".to_string(),
        argument_binders: 0,
        recursive_positions: Vec::new(),
        body: RuntimeExpr::Let {
            value: Box::new(slot_value),
            body: Box::new(call_body),
        },
    }];
    let default = RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "PX8-TA marker fixture default".to_string(),
    };
    let runtime_frame_fingerprint =
        crate::compiler_private_computational_match_frame_fingerprint(&cases, &default);
    let body = RuntimeExpr::CheckedSubcontinuationFrame {
        frame_id: 0,
        body: Box::new(RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Only".to_string(),
                args: Vec::new(),
            }),
            cases,
            default,
        }),
    };
    let runtime_declaration = RuntimeDeclaration {
        symbol: declaration.clone(),
        kind: RuntimeDeclarationKind::Transparent { body },
        metadata: RuntimeSymbolMetadata::empty(),
    };
    let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
        frame_id: 0,
        segment_site_id: 9,
        declaration: declaration.clone(),
        checked_occurrence_path: vec![10],
        semantic_position: 0,
        input_interface: oriented_test_interface(0),
        output_interface: oriented_test_interface(1),
        runtime_frame_fingerprint,
        occurrence_binding_fingerprint: 0,
        control_witness: crate::OrientedControlWitnessV1::DistinguishedRoot,
    };
    frame.occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
    let mut slot = crate::CheckedComputationalIHSlotTemplateV1 {
        slot_template_id: 200,
        declaration: declaration.clone(),
        checked_match_ordinal: 0,
        checked_occurrence_path: vec![20],
        frame_template_id: 0,
        constructor: "ctor:fixture::Only".to_string(),
        recursive_position: 0,
        method_binder_ordinal: 0,
        local_telescope: Vec::new(),
        ih_interface: oriented_test_interface(0),
        segment_site_id: 9,
        frame_templates: vec![0],
        input_interface: oriented_test_interface(0),
        output_interface: oriented_test_interface(1),
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration: declaration.clone(),
            runtime_path: vec![0, 1, 0],
        }],
        occurrence_binding_fingerprint: 0,
    };
    slot.occurrence_binding_fingerprint =
        crate::compiler_private_computational_ih_slot_binding_fingerprint(&slot);
    let mut call = crate::CheckedComputationalIHCallTemplateV1 {
        call_template_id: 100,
        declaration: declaration.clone(),
        checked_occurrence_path: vec![30],
        slot_template_id: 200,
        arity: 1,
        local_telescope: Vec::new(),
        result_interface: oriented_test_interface(1),
        callee_segment_site_id: 9,
        callee_frame_templates: vec![0],
        parent_frame_template_id: Some(0),
        parent_segment_site_id: Some(9),
        caller_interface: oriented_test_interface(1),
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration,
            runtime_path: vec![0, 1, 1],
        }],
        occurrence_binding_fingerprint: 0,
    };
    call.occurrence_binding_fingerprint =
        crate::compiler_private_computational_ih_call_binding_fingerprint(&call);
    (
        RuntimeExpr::Value(RuntimeValue::Int((0).into())),
        runtime_declaration,
        crate::OrientedSubcontinuationPlanV1 {
            representation_rule_version:
                crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
            frames: vec![frame],
            recursive_calls: Vec::new(),
            computational_ih_slots: vec![slot],
            computational_ih_calls: vec![call],
        },
    )
}

// ── RT-SPLIT slice 7, rule 8 finalization ─────────────────────────────────
// Residual facade test fixtures whose final-user LCA is this module. Facade
// file scope was a TRANSITIONAL zero-widening holding position, never final
// ownership (Architect `evt_h69xwchqqxmj`); slice 7 discharges it. Moved
// verbatim -- ordered item-level identity, no body edits.

#[cfg(test)]
fn self_consistent_root_join_site(site_id: u64) -> crate::NativeJoinPlanSiteV1 {
    let declaration = "decl:fixture::PX8H::main".to_string();
    let checked_occurrence_path = vec![0];
    let checked_result_type_fingerprint = 19;
    crate::NativeJoinPlanSiteV1 {
        site_id,
        occurrence_binding_fingerprint: crate::compiler_private_join_occurrence_binding_fingerprint(
            site_id,
            &declaration,
            &checked_occurrence_path,
            checked_result_type_fingerprint,
        ),
        declaration,
        checked_occurrence_path,
        checked_result_type_fingerprint,
        runtime_frame_fingerprint: crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
        answer_kind: crate::NativeJoinAnswerKindV1::ExitCode,
    }
}

#[cfg(test)]
fn oriented_test_interface(name: u8) -> crate::CheckedAnswerInterfaceV1 {
    let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    bytes.push(name);
    crate::CheckedAnswerInterfaceV1::new(bytes).unwrap()
}

#[cfg(test)]
fn oriented_test_frame(
    frame_id: u64,
    semantic_position: u64,
    input: u8,
    output: u8,
    parent: Option<u64>,
) -> crate::OrientedSubcontinuationFramePlanV1 {
    let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
        frame_id,
        segment_site_id: 9,
        declaration: "decl:fixture::oriented".to_string(),
        checked_occurrence_path: vec![frame_id],
        semantic_position,
        input_interface: oriented_test_interface(input),
        output_interface: oriented_test_interface(output),
        runtime_frame_fingerprint: frame_id + 100,
        occurrence_binding_fingerprint: 0,
        control_witness: parent.map_or(
            crate::OrientedControlWitnessV1::DistinguishedRoot,
            crate::OrientedControlWitnessV1::ParentFrame,
        ),
    };
    frame.occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
    frame
}

#[cfg(test)]
fn self_consistent_join_site(
    site_id: u64,
    runtime_frame_fingerprint: u64,
) -> crate::NativeJoinPlanSiteV1 {
    let declaration = "decl:fixture::PX8H::main".to_string();
    let checked_occurrence_path = vec![1, site_id];
    let checked_result_type_fingerprint = 17;
    crate::NativeJoinPlanSiteV1 {
        site_id,
        occurrence_binding_fingerprint: crate::compiler_private_join_occurrence_binding_fingerprint(
            site_id,
            &declaration,
            &checked_occurrence_path,
            checked_result_type_fingerprint,
        ),
        declaration,
        checked_occurrence_path,
        checked_result_type_fingerprint,
        runtime_frame_fingerprint,
        answer_kind: crate::NativeJoinAnswerKindV1::Int,
    }
}

// ─── RT-FNSPLIT-B2A-C D5 — the coverage guard ─────────────────────────────
//
// ⭐ This is the deliverable with the longest half-life in the chain: it is what
// stops inventory entry 3 recurring the next time `RuntimeExpr` grows a field.
// It has TWO independent failure modes, and the first is a COMPILE error rather
// than an assertion, which is strictly stronger:
//
//  1. `expression_children` below matches every `RuntimeExpr` variant with its
//     fields spelled out and **no `..` and no `_ =>` arm**. Add a field to any
//     variant and this stops compiling (E0027 "pattern does not mention field");
//     add a variant and it stops compiling (E0004). A wildcard here is what
//     would let a new expression-typed field become silently originless, so the
//     absence of one is the mechanism, not a style preference.
//  2. Even with the pattern updated, the guard asserts that the plane holds
//     **exactly** the enumerated children for a planned instance of every
//     variant — no more, no fewer — so a field that is enumerated here but not
//     planned, or planned but not enumerated, is still red.
//
// ⛔ A test that merely enumerates today's variants and passes is NOT this
// guard (AC-3). The demonstration that it reddens on an *added* field is in the
// handoff.

/// Every expression-typed field of one occurrence, **in the planner's child
/// order** — the order of the `children` slice handed to `expression_node` /
/// `expression_seed`, which is what the positional child-origin range is laid
/// out against.
///
/// ⚠ Two variants order their children differently from their declaration:
/// `LexicalClosure` plans **body first** (position 0) with capture *i* at
/// `1 + i`, and `Effect` gives position 0 to `capability.value` **only when it
/// is present**, shifting every argument by one.
#[cfg(test)]
fn expression_children(expr: &RuntimeExpr) -> Vec<&RuntimeExpr> {
    match expr {
        RuntimeExpr::CheckedJoinSite { site_id: _, body } => vec![body],
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id: _, body } => vec![body],
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id: _,
            checked_occurrence_path: _,
            body,
        } => vec![body],
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids: _,
            checked_occurrence_paths: _,
            body,
        } => vec![body],
        RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id: _,
            checked_occurrence_path: _,
            body,
        } => vec![body],
        RuntimeExpr::Value(_) => Vec::new(),
        RuntimeExpr::Var(_) => Vec::new(),
        RuntimeExpr::Let { value, body } => vec![value, body],
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => vec![scrutinee, then_expr, else_expr],
        RuntimeExpr::PrimitiveCall { primitive: _, args } => args.iter().collect(),
        RuntimeExpr::Construct {
            constructor: _,
            args,
        } => args.iter().collect(),
        RuntimeExpr::Match {
            scrutinee,
            cases,
            default: _,
        } => std::iter::once(scrutinee.as_ref())
            .chain(cases.iter().map(|case| &case.body))
            .collect(),
        RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default: _,
        } => std::iter::once(scrutinee.as_ref())
            .chain(cases.iter().map(|case| &case.body))
            .collect(),
        RuntimeExpr::Record { fields } => fields.iter().map(|(_, value)| value).collect(),
        RuntimeExpr::Project { record, field: _ } => vec![record],
        RuntimeExpr::Closure {
            captures: _,
            params: _,
            body,
        } => vec![body],
        RuntimeExpr::LexicalClosure {
            captures,
            params: _,
            body,
        } => std::iter::once(body.as_ref())
            .chain(captures.iter())
            .collect(),
        RuntimeExpr::DeclarationRef { symbol: _ } => Vec::new(),
        RuntimeExpr::ImportedDeclarationRef {
            symbol: _,
            dependency: _,
            dependency_semantic_hash: _,
        } => Vec::new(),
        RuntimeExpr::Call { callee, args } => std::iter::once(callee.as_ref())
            .chain(args.iter())
            .collect(),
        RuntimeExpr::Effect {
            family: _,
            operation: _,
            capability,
            args,
        } => capability
            .iter()
            .map(|capability| capability.value.as_ref())
            .chain(args.iter())
            .collect(),
        RuntimeExpr::Trap(_) => Vec::new(),
    }
}

/// One planned instance of **every** `RuntimeExpr` variant, each carrying at
/// least one expression-typed field where the variant has any, so a dropped
/// position cannot hide behind an empty list.
#[cfg(test)]
fn every_variant_occurrence() -> Vec<(&'static str, RuntimeExpr)> {
    let leaf = || RuntimeExpr::Value(RuntimeValue::Bool(true));
    let trap = || RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "b2ac coverage guard".to_string(),
    };
    vec![
        (
            "CheckedJoinSite",
            RuntimeExpr::CheckedJoinSite {
                site_id: 1,
                body: Box::new(leaf()),
            },
        ),
        (
            "CheckedSubcontinuationFrame",
            RuntimeExpr::CheckedSubcontinuationFrame {
                frame_id: 2,
                body: Box::new(leaf()),
            },
        ),
        (
            "CheckedRecursiveInvocation",
            RuntimeExpr::CheckedRecursiveInvocation {
                call_template_id: 3,
                checked_occurrence_path: vec![1],
                body: Box::new(leaf()),
            },
        ),
        (
            "CheckedComputationalIHSlots",
            RuntimeExpr::CheckedComputationalIHSlots {
                slot_template_ids: vec![4],
                checked_occurrence_paths: vec![vec![1]],
                body: Box::new(leaf()),
            },
        ),
        (
            "CheckedComputationalIHInvocation",
            RuntimeExpr::CheckedComputationalIHInvocation {
                call_template_id: 5,
                checked_occurrence_path: vec![1],
                body: Box::new(leaf()),
            },
        ),
        ("Value", leaf()),
        ("Var", RuntimeExpr::Var(0)),
        (
            "Let",
            RuntimeExpr::Let {
                value: Box::new(leaf()),
                body: Box::new(RuntimeExpr::Var(0)),
            },
        ),
        (
            "If",
            RuntimeExpr::If {
                scrutinee: Box::new(leaf()),
                then_expr: Box::new(leaf()),
                else_expr: Box::new(leaf()),
            },
        ),
        (
            "PrimitiveCall",
            total_primitive("prim:fixture::b2ac", vec![leaf(), leaf()]),
        ),
        (
            "Construct",
            RuntimeExpr::Construct {
                constructor: "ctor:fixture::B2AC::Pair".to_string(),
                args: vec![leaf(), leaf()],
            },
        ),
        (
            "Match",
            RuntimeExpr::Match {
                scrutinee: Box::new(leaf()),
                cases: vec![
                    RuntimeMatchCase {
                        constructor: "ctor:fixture::B2AC::A".to_string(),
                        binders: 0,
                        body: leaf(),
                    },
                    RuntimeMatchCase {
                        constructor: "ctor:fixture::B2AC::B".to_string(),
                        binders: 0,
                        body: leaf(),
                    },
                ],
                default: trap(),
            },
        ),
        (
            "ComputationalMatch",
            RuntimeExpr::ComputationalMatch {
                scrutinee: Box::new(leaf()),
                cases: vec![crate::RuntimeComputationalMatchCase {
                    constructor: "ctor:fixture::B2AC::A".to_string(),
                    argument_binders: 0,
                    recursive_positions: Vec::new(),
                    body: leaf(),
                }],
                default: trap(),
            },
        ),
        (
            "Record",
            RuntimeExpr::Record {
                fields: vec![("l".to_string(), leaf()), ("r".to_string(), leaf())],
            },
        ),
        (
            "Project",
            RuntimeExpr::Project {
                record: Box::new(RuntimeExpr::Record {
                    fields: vec![("l".to_string(), leaf())],
                }),
                field: "l".to_string(),
            },
        ),
        (
            "Closure",
            RuntimeExpr::Closure {
                captures: Vec::new(),
                params: vec!["x".to_string()],
                body: Box::new(leaf()),
            },
        ),
        (
            "LexicalClosure",
            RuntimeExpr::LexicalClosure {
                captures: vec![leaf(), leaf()],
                params: vec!["x".to_string()],
                body: Box::new(leaf()),
            },
        ),
        (
            "DeclarationRef",
            RuntimeExpr::DeclarationRef {
                symbol: "decl:fixture::b2ac".to_string(),
            },
        ),
        (
            "ImportedDeclarationRef",
            RuntimeExpr::ImportedDeclarationRef {
                symbol: "decl:fixture::b2ac".to_string(),
                dependency: "pkg:fixture".to_string(),
                dependency_semantic_hash: "hash".to_string(),
            },
        ),
        (
            "Call",
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: vec![leaf(), leaf()],
            },
        ),
        (
            "Effect (capability present)",
            RuntimeExpr::Effect {
                family: "Fs".to_string(),
                operation: ken_host::HostOpV1::FsReadFile,
                capability: Some(crate::RuntimeCapabilityUse {
                    identity: "cap:fixture::fs".to_string(),
                    value: Box::new(RuntimeExpr::Var(0)),
                }),
                args: vec![leaf()],
            },
        ),
        (
            "Effect (capability absent)",
            RuntimeExpr::Effect {
                family: "Console".to_string(),
                operation: ken_host::HostOpV1::ConsoleWrite,
                capability: None,
                args: vec![leaf(), leaf()],
            },
        ),
        ("Trap", RuntimeExpr::Trap(trap())),
    ]
}

#[test]
fn every_expression_typed_field_is_a_reachable_positional_child_origin() {
    let mut unreachable = Vec::new();
    for (name, occurrence) in every_variant_occurrence() {
        let (plan, origin) = planned_root_occurrence(&occurrence);
        let children = expression_children(&occurrence);

        // Every enumerated position resolves to a real preallocated origin.
        for position in 0..children.len() {
            if plan.child_static_origin(origin, position).is_err() {
                unreachable.push(format!("{name}: position {position} does not resolve"));
            }
        }
        // And there is no position beyond them: the plane holds exactly the
        // enumerated children, so an unenumerated field is red too.
        if plan.child_static_origin(origin, children.len()).is_ok() {
            unreachable.push(format!(
                "{name}: the plane holds a child at position {} that no field enumerates",
                children.len()
            ));
        }
    }
    assert!(
        unreachable.is_empty(),
        "every expression-typed field must be a reachable positional child origin: {unreachable:#?}"
    );
}

// ─── RT-FNSPLIT-B2A-C AC-4/AC-6 — the positional-derivation controls ──────
//
// ★ AC-4's second control is the chain's own predicate as an executable test:
// if identity moves when only the ADDRESS moved, the tag is not authoritative.

/// Two same-shaped children distinguishable **only** by how many children they
/// themselves have — so which one a position resolves to is observable through
/// the positional accessor alone, with no origin→expression lookup (N3).
#[cfg(test)]
fn one_child_record() -> RuntimeExpr {
    RuntimeExpr::Record {
        fields: vec![(
            "l".to_string(),
            RuntimeExpr::Value(RuntimeValue::Bool(true)),
        )],
    }
}

#[cfg(test)]
fn two_child_record() -> RuntimeExpr {
    RuntimeExpr::Record {
        fields: vec![
            (
                "l".to_string(),
                RuntimeExpr::Value(RuntimeValue::Bool(true)),
            ),
            (
                "r".to_string(),
                RuntimeExpr::Value(RuntimeValue::Bool(false)),
            ),
        ],
    }
}

#[test]
fn swapping_two_same_shaped_children_swaps_their_derived_origins() {
    let branch = |then_expr: RuntimeExpr, else_expr: RuntimeExpr| RuntimeExpr::If {
        scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
    };
    // `arity_at(position)` reads how many children the occurrence at that
    // position has, using nothing but the positional accessor.
    let arity_at = |expr: &RuntimeExpr, position: usize| {
        let (plan, root) = planned_root_occurrence(expr);
        let child = plan
            .child_static_origin(root, position)
            .expect("If has three positional children");
        (0..)
            .take_while(|inner| plan.child_static_origin(child, *inner).is_ok())
            .count()
    };

    let straight = branch(one_child_record(), two_child_record());
    let swapped = branch(two_child_record(), one_child_record());

    assert_eq!(
        arity_at(&straight, 1),
        1,
        "then_expr is the one-child record"
    );
    assert_eq!(
        arity_at(&straight, 2),
        2,
        "else_expr is the two-child record"
    );
    // The children swapped in the source; the derived origins swapped with them.
    assert_eq!(
        arity_at(&swapped, 1),
        2,
        "then_expr is now the two-child record"
    );
    assert_eq!(
        arity_at(&swapped, 2),
        1,
        "else_expr is now the one-child record"
    );
}

#[test]
fn perturbing_a_borrowed_address_does_not_move_any_derived_origin() {
    let expr = RuntimeExpr::If {
        scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        then_expr: Box::new(one_child_record()),
        else_expr: Box::new(two_child_record()),
    };
    // A clone is the same syntax at different addresses, and boxing it again
    // moves every interior node. No ordinal changes.
    let relocated = Box::new(expr.clone());

    let derive = |expr: &RuntimeExpr| {
        let (plan, root) = planned_root_occurrence(expr);
        (0..3)
            .map(|position| {
                plan.child_static_origin(root, position)
                    .expect("If has three positional children")
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(
        derive(&expr),
        derive(relocated.as_ref()),
        "identity must not move when only the address moved"
    );
}

#[test]
fn an_out_of_range_child_position_is_a_loud_planner_invariant() {
    let record = one_child_record();
    let (plan, root) = planned_root_occurrence(&record);
    let error = plan
        .child_static_origin(root, 7)
        .expect_err("a record with one field has no child at position 7");
    // AC-6: an invariant violation is a compiler bug, never a capacity limit --
    // so the specific variant is asserted, not `is_err()`.
    match error {
        CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(ref reason)) => {
            assert_eq!(reason, "static origin has no child at that source position");
        }
        other => panic!("expected a loud PlannerInvariant, got {other:?}"),
    }
}

// ─── RT-FNSPLIT-B2A-C N1/N2 — the emission census, pinned mechanically ────
//
// AC-7 wants each negative-boundary pin discharged by a committed check rather
// than by review reading. N1 and N2 are counting properties over the PRODUCTION
// lowering and planning sources, so they are pinned by counting call
// expressions in those exact files. The test sources live in a sibling
// directory, so no `#[cfg(test)]` region has to be parsed out: the partition is
// at file level.

/// **`RT-FNSPLIT-B2F` `AC-2` — WHICH POPULATION THIS CENSUS COVERS, and why the
/// rest is excluded.**
///
/// ⛔ **The population is the production LOWERING AND PLANNING sources** — the
/// seven rows below. It is deliberately **not** "every Cranelift emitter in
/// `ken-runtime`", and stating that boundary is `AC-2`'s second clause: a census
/// whose scope is implicit reads as covering everything.
///
/// **Excluded, measured at base `6534e4a6`, each with its reason:**
///
/// | emitter | measured | why it is out of scope here |
/// |---|---|---|
/// | `native_int_clif.rs` | 5 / 1 / 3 | Θ(1) per native module. Its emitted population is already pinned behaviourally as `LOCAL_HELPER_COUNT = 6` (`artifact/tests.rs:56`) — ⛔ cite that, do not duplicate it |
/// | `boundary_value_clif.rs` | 23 / 3 / 3 | ⭐ a live production emitter that was in **neither** this census nor `BACKEND_PRODUCTION_SOURCES`; same Θ(1)-per-module shape |
///
/// ⭐ **Why they are recorded as reasoned exclusions rather than pinned rows,
/// which is a judgement and is stated as one:** freezing `23` and `5` here would
/// redden this file whenever a *sibling* node legitimately changes an emitter it
/// owns — landing the failure on whoever is unlucky, in a test they have never
/// read, rather than on whoever changed the thing. Their growth is `AC-G0`/`D8`'s
/// obligation and is discharged there **behaviourally**, against emitted counts,
/// not against source spellings.
///
/// ⚠ **MEASURED:** how many times five spellings occur in eight files.
/// **CLAIMED:** exactly that. **THE GAP:** ⛔ this is a source-TEXT oracle and
/// it is retained as a **tripwire, not as the evidence**. A call split across
/// lines evades every needle; a mention inside a string or a block comment
/// inflates them; and nothing here observes what a compiled module actually
/// contains.
///
/// # ⛔⛔ WHICH INSTRUMENT CARRIES THE CLAIM — and they are NOT corroboration
///
/// **`AC-2` requires this division of labour to be stated in-source, because two
/// counts sitting side by side read as corroboration and these two are not: one
/// of them is fail-open by construction.**
///
/// | instrument | what it does | what it carries |
/// |---|---|---|
/// | ⭐ the behavioural counters — `units::b2f_last_unit_emission`, `seed_material::b2f_last_seed_material_emission` | count what the compiled module **actually contains**, at the point of emission | ⭐ **the population claim, entirely** |
/// | ⚠ this census | searches source text for spellings someone enumerated | ⛔ **nothing.** A tripwire only |
///
/// ⛔ **This census's default branch is *"needle not found ⇒ nothing
/// emitted"*, so it fails OPEN for every emission spelling nobody thought of.**
/// It was repaired three times on this node — missing rows, then missing
/// sibling emitters, then a missing needle class — and each repair found the
/// next thing it was not looking for, because a needle-list census can only
/// ever be one discovery behind the code. ⛔ **Adding `.declare_data(` /
/// `.define_data(` did not make it sound and nothing here claims it did.** It
/// is retained, unweakened, because a defeat count never licenses removing a
/// gate — not because it is evidence.
#[test]
fn correspondence_adds_no_emitted_unit_to_the_production_census() {
    struct Census {
        file: &'static str,
        source: &'static str,
        builders: usize,
        definitions: usize,
        declarations: usize,
        /// ⭐ **`RT-FNSPLIT-B2F` `AC-2`, third population defect.** Data objects
        /// are declared and defined by `.declare_data(` / `.define_data(`, and
        /// the three needles above cannot see either. That is a strictly worse
        /// shape than a missing row: a missing *row* leaves one file unmeasured
        /// and the gap is visible, while a missing *needle class* leaves the
        /// census reading **complete across every row** while `n` data objects
        /// sit in the artifact — `D3`'s entire deliverable, invisible, with
        /// nothing looking wrong.
        data_declarations: usize,
        data_definitions: usize,
    }
    let census = [
        Census {
            file: "lowering/core.rs",
            source: include_str!("../../core.rs"),
            // N1: exactly one root `FunctionBuilder::new` and one root
            // `define_function`. The two declarations are the entry point and
            // the IMPORTED host-dispatch symbol -- an import, not a definition.
            builders: 1,
            definitions: 1,
            declarations: 2,
            data_declarations: 0,
            data_definitions: 0,
        },
        Census {
            file: "lowering/mod.rs",
            source: include_str!("../../mod.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
            data_declarations: 0,
            data_definitions: 0,
        },
        Census {
            file: "planning.rs",
            source: include_str!("../../../planning.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
            data_declarations: 0,
            data_definitions: 0,
        },
        Census {
            file: "planning/static_transition.rs",
            source: include_str!("../../../planning/static_transition.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
            data_declarations: 0,
            data_definitions: 0,
        },
        Census {
            file: "planning/static_transition/semantic_ir.rs",
            source: include_str!("../../../planning/static_transition/semantic_ir.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
            data_declarations: 0,
            data_definitions: 0,
        },
        // ⭐ `RT-FNSPLIT-B2F` `AC-2` — THE PREDICTED ROW, and it is predicted
        // rather than fitted.
        //
        // Recorded in `docs/program/rt-fnsplit-b2f-predictions.md` (`P1`) at
        // base `6534e4a6`, committed BEFORE the module was written, and then
        // measured: 1 / 1 / 1, exactly as predicted. A census re-fitted to
        // whatever the output happened to be measures nothing, so the order is
        // the evidence.
        //
        // ⛔ ONE of each spelling, for a population of Θ(n) emitted units. The
        // needles count SPELLINGS, never units: `declare_unit_bundle` holds one
        // `declare_function` inside a loop over every unit, and
        // `define_unit_body` is called once per unit from one site. That gap is
        // the whole content of `AC-G0`'s narrative — `native_int_clif` emits 6
        // definitions from 5 builder source sites — and it is why this row
        // cannot be read as an emitted-unit count. `D8`'s growth verdict is
        // about `UnitBundle::len`, which this pin cannot see.
        Census {
            file: "lowering/units.rs",
            source: include_str!("../../units.rs"),
            builders: 1,
            definitions: 1,
            declarations: 1,
            data_declarations: 0,
            data_definitions: 0,
        },
        // `RT-FNSPLIT-B2R`'s ABI plane, added as an explicit ZERO row because
        // the frame flagged its absence: it is in `BACKEND_PRODUCTION_SOURCES`
        // and was not in this census, and an absent row and a zero row read
        // identically to a reader while only one of them is a claim.
        //
        // ⭐ The zero is the load-bearing part: `abi.rs` DECLARES the
        // representation contract and must never emit against it. If this row
        // ever moves, the planner has started emitting, which is the one thing
        // the ownership/representation split exists to prevent.
        Census {
            file: "planning/static_transition/abi.rs",
            source: include_str!("../../../planning/static_transition/abi.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
            data_declarations: 0,
            data_definitions: 0,
        },
        // ⭐ `RT-FNSPLIT-B2F` `D3`/`AC-2` — THE SECOND PREDICTED ROW, and the
        // prediction was recorded before the module existed for the same reason
        // the first one was.
        //
        // Recorded in `docs/program/rt-fnsplit-b2f-predictions.md` (`P6`) at
        // base `6534e4a6`: **1 `declare_data` / 1 `define_data`, every other row
        // 0/0.** Measured: exactly that.
        //
        // ⚠ **AND `P6` WAS WRONG ABOUT WHERE, WHICH IS RECORDED RATHER THAN
        // QUIETLY CORRECTED.** It named `lowering/units.rs` as the file carrying
        // the two needles; the material is minted in `lowering/seed_material.rs`
        // instead, because units and seed material are two populations on two
        // growth axes (Θ(n) in the program vs Θ(|seed environment|), which the
        // program does not affect) and one census row cannot carry both. ⇒ The
        // *counts* held; the *row* moved. A prediction file that only ever
        // agrees with the outcome is a transcription, and `P4` said in advance
        // that the row placement was the likeliest thing to move.
        //
        // ⛔ ONE of each spelling for a population of Θ(|seed environment|)
        // objects: `mint_seed_material` holds one `declare_data` and one
        // `define_data` inside a loop over every entry. Same spellings-not-units
        // gap as the row above, and the same consequence — ⛔ **this row is not
        // an object count and must never be read as one.**
        Census {
            file: "lowering/seed_material.rs",
            source: include_str!("../../seed_material.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
            data_declarations: 1,
            data_definitions: 1,
        },
    ];
    for row in census {
        assert_eq!(
            row.source.matches("FunctionBuilder::new(").count(),
            row.builders,
            "{}: N1 -- the production root builder census moved",
            row.file
        );
        assert_eq!(
            row.source.matches(".define_function(").count(),
            row.definitions,
            "{}: N1/N2 -- a definition was added or removed",
            row.file
        );
        assert_eq!(
            row.source.matches(".declare_function(").count(),
            row.declarations,
            "{}: N2 -- a function declaration was added or removed",
            row.file
        );
        assert_eq!(
            row.source.matches(".declare_data(").count(),
            row.data_declarations,
            "{}: N3 -- an artifact-static data declaration was added or removed",
            row.file
        );
        assert_eq!(
            row.source.matches(".define_data(").count(),
            row.data_definitions,
            "{}: N3 -- an artifact-static data definition was added or removed",
            row.file
        );
    }
}

/// **`RT-FNSPLIT-B2A-S` D4/AC-4 — the `origin -> expression` lookup count is
/// EXACTLY ONE.** This pin *replaces* `RT-FNSPLIT-B2A-C`'s N3, and the transition
/// is stated here rather than in a commit message so it stays auditable:
///
/// | | B2A-C's N3 (retired) | this pin |
/// |---|---|---|
/// | lookups that exist | **0** — `!source.contains("-> Result<&'src RuntimeExpr")` | **1** |
/// | consumers that call one | 0 (none existed to call) | **1** |
///
/// ⛔ N3 was **not** violated — it was **retired by design**. B2A-C asserted zero
/// because at that point the origin was provenance, and any lookup would have been
/// an unaudited second authority. B2A-S's whole job is to introduce the lookup so
/// that a retained body is selected by its static name. ⭐ The count therefore goes
/// `0 -> 1`, never `0 -> unbounded`: one producer, one consumer.
///
/// A reviewer reading the new lookup against B2A-C's AC list without this table
/// would reject a correct diff.
#[test]
fn exactly_one_plan_origin_to_expression_lookup_exists() {
    let planner = include_str!("../../../planning/static_transition.rs");

    // The PRODUCING end, pinned as a whole exported surface rather than by
    // searching for one name: a second resolver added here would redden this even
    // if it were never called.
    let exported: Vec<&str> = planner
        .lines()
        .filter(|line| {
            line.trim_start()
                .starts_with("pub(in crate::cranelift_backend) fn ")
        })
        .map(|line| line.trim())
        .collect();
    assert_eq!(
        exported,
        vec![
            // `RT-FNSPLIT-B2F` `D1` — the emitter's read-only view of ONE
            // validated function unit. Six accessors, one type, no constructor.
            //
            // ⭐ Same shape as `C1`'s four below: a *question* about a planned
            // object with an answer the asker cannot mint. `EmittableUnit`'s
            // fields are private and its sole producer is `emittable_units`, so
            // a unit cannot be forged in `lowering` — and since `B2F` drives
            // emission from units, emission cannot be driven from anything but
            // the validated plane.
            //
            // ⛔ `AbiPlane`, `AbiDescriptor`, `build_abi_plane` and
            // `AbiPlane::validate` stay `pub(super)` and are NOT here. The
            // emitter reads a unit; it cannot construct the plane, mutate a
            // descriptor, or reach the pre-emission validator to bypass it. One
            // of those names appearing in this list is the violation.
            //
            // ⚠ None of the six returns a source term, so the `-> Result<&'src
            // RuntimeExpr` count below is still exactly one and `B2A-S`'s `AC-4`
            // is untouched. A unit carries an ORIGIN; resolving that origin to a
            // term still goes through `source_occurrence`, which is why `B2F`
            // adds no second `origin -> expression` lookup.
            "pub(in crate::cranelift_backend) fn function(self) -> PredeclaredFunctionId {",
            "pub(in crate::cranelift_backend) fn origin(self) -> StaticOriginId {",
            "pub(in crate::cranelift_backend) fn definition(self) -> AbiUnitDefinition {",
            "pub(in crate::cranelift_backend) fn header(self) -> AbiFrameHeader {",
            "pub(in crate::cranelift_backend) fn slots(self) -> &'plan [AbiSlot] {",
            "pub(in crate::cranelift_backend) fn slot_offsets(",
            "pub(in crate::cranelift_backend) fn source_occurrence(",
            "pub(in crate::cranelift_backend) fn child_static_origin(",
            // `RT-FNSPLIT-C1` `D1` — the artifact-static identity capability.
            //
            // ⭐ These four are the whole of `D1`, and they are the shape the
            // Architect's ruling requires: an occurrence-keyed *question* with
            // an unmintable answer. ⛔ `SemanticPlane` and its `names` arena
            // stay `pub(super)`; widening either to serve a consumer is what
            // this pin exists to catch, and adding a capability is not that.
            //
            // ⚠ None of them returns a source term, so the `-> Result<&'src
            // RuntimeExpr` count below is still exactly one. That assertion is
            // the one carrying B2A-S's AC-4; this list is the surrounding
            // allowed-inventory.
            "pub(in crate::cranelift_backend) fn case_constructor_identity(",
            "pub(in crate::cranelift_backend) fn constructor_symbol_identity(",
            // `RT-FNSPLIT-C2-SYNTH-ID` adds one closed synthesized-role
            // identity route plus the opaque dynamic-role population. Neither
            // accepts a spelling, origin, hash, or ordinal from lowering.
            "pub(in crate::cranelift_backend) fn synthesized_constructor_identity(",
            "pub(in crate::cranelift_backend) fn synthesized_io_error_roles(",
            "pub(in crate::cranelift_backend) fn project_field_identity(",
            "pub(in crate::cranelift_backend) fn record_field_identity(",
            "pub(in crate::cranelift_backend) fn root_static_origin(",
            "pub(in crate::cranelift_backend) fn declaration_occurrence_origin(",
            // `RT-FNSPLIT-B2F` `D1` — the sole producer of an `EmittableUnit`,
            // and therefore the sole route by which emission can be driven.
            //
            // ⛔ It projects `self.abi.descriptors`; it does not re-seed the
            // population and must never be made to. The unit set is
            // `plan.entries` ∪ every `EdgeKind::StaticBody` TARGET, already
            // enforced by `validate_function_units`. In particular it does not
            // consult `TransitionKind::ClosureBody`, which is a body's return
            // successor and not a unit head.
            "pub(in crate::cranelift_backend) fn emittable_units(",
            "pub(in crate::cranelift_backend) fn plan_static_transition_graph<'src>(",
            "pub(in crate::cranelift_backend) fn plan_static_transition_graph_with_symbols<'src>(",
        ],
        "AC-4 -- the planner's exported surface changed; exactly one of these may \
         return a source term"
    );
    assert_eq!(
        planner
            .lines()
            .filter(|line| line.contains("-> Result<&'src RuntimeExpr"))
            .count(),
        1,
        "AC-4 -- exactly one accessor may return a borrowed source expression \
         (B2A-C's N3 required zero; B2A-S requires one)"
    );

    // The CONSUMING end, over the WHOLE backend production surface.
    //
    // ⛔ The first candidate scanned only `lowering/core.rs` and `lowering/mod.rs`
    // and argued closure from `Lowering::static_transition_plan` being private.
    // The Architect rejected that (`evt_6sq2tq3v9jcd0`) and was right: the
    // resolver is `pub(in crate::cranelift_backend)` and `planning.rs` re-exports
    // `plan_static_transition_graph` to the backend parent, so ANY backend sibling
    // can build its own plan and call the resolver without owning a `Lowering` at
    // all. A second call in `artifact/**`, `compiled.rs` or `planning.rs` would
    // have stayed green. Privacy of one field was never the closure.
    let mut mentions = Vec::new();
    for (file, source) in BACKEND_PRODUCTION_SOURCES {
        // `static_transition.rs` carries its tests inline; the census is about the
        // production surface, and the planner's own tests legitimately call the
        // resolver to exercise it.
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(*source, |(before, _)| before);
        let n = identifier_occurrences(production, "source_occurrence");
        if n > 0 {
            mentions.push((*file, n));
        }
    }
    assert_eq!(
        mentions,
        vec![
            ("lowering/core.rs", 1),
            ("planning/static_transition.rs", 1)
        ],
        "AC-4 -- the resolver may be NAMED exactly twice in production: its \
         definition in the planner, and its single call from \
         `retained_body_occurrence`. Any third mention is a second lookup"
    );
}

/// Counts whole-identifier occurrences in production source, comments removed.
///
/// ⛔ **Neither a substring scan nor a line scan is sound, and the Architect proved
/// both against me** (`evt_6sq2tq3v9jcd0`, `evt_1p11krxny4wny`). The first census
/// tested `line.contains(".source_occurrence(")`, which a second lookup evaded by
/// formatting alone:
///
/// ```text
/// let _second = self
///     .static_transition_plan
///     .source_occurrence
///     (static_origin)?;
/// ```
///
/// No line contains `.source_occurrence(`, so the pin passed with two lookups
/// present. ⇒ **A text pattern is a claim about layout; the property is about
/// code.** Tokenizing is the fix, not a longer list of spellings:
///
/// - splitting on every non-identifier character makes newlines, dots and spaces
///   all separators, so **no formatting can hide a mention**;
/// - matching a **whole token** distinguishes `source_occurrence` from the
///   `source_occurrences` field, which a substring scan would conflate;
/// - counting the **identifier** rather than a call shape also catches a path-form
///   or aliased call (`StaticTransitionPlan::source_occurrence(plan, o)`), because
///   a method cannot be called without naming it;
/// - comments are stripped per line **before** tokenizing, because the resolver's
///   own doc comment and these very notes name it — an oracle that greps a name
///   otherwise fires on the prose describing it.
///
/// ⚠ Residual: a call synthesized by a macro would not name the identifier in this
/// source. There is no such macro in the backend, and one would be visible in the
/// same review; this is a stated limit, not a silent one.
#[cfg(test)]
/// **Every production source carrying an `impl Lowering` block.**
///
/// ⛔ `core.rs` alone is NOT the routing surface, and assuming it was is the
/// defect this constant exists to prevent: `lowering/mod.rs:2473` carries a
/// **second** `impl<'a> Lowering<'a>` block. A retained-body route added there
/// would have sat entirely outside a `core.rs`-scoped inventory.
///
/// ⛔ **An earlier revision of this comment continued: *"today `mod.rs` cannot
/// reach `retained_body_occurrence` … that privacy is therefore load-bearing …
/// this list is what makes the inventory still correct after a deliberate
/// widening."* That is the REACHABILITY entailment the Architect ruling
/// (`evt_5yxjd1zqnyvcq`) struck, and it is withdrawn here too.**
///
/// The list is now a **declaration inventory only**: it names the files that
/// carry an `impl Lowering` block, so a declaration appearing in a second one is
/// *visible*. It supports no claim about who can **call** anything — that is the
/// plan graph's to answer, via an occurrence's `SemanticOwner` and the planned
/// edge kind.
const LOWERING_IMPL_SOURCES: &[(&str, &str)] = &[
    ("lowering/core.rs", include_str!("../../core.rs")),
    ("lowering/mod.rs", include_str!("../../mod.rs")),
];

/// Is the retained-body helper still declared private to module `core`?
///
/// Matched as the exact unqualified form, so **any** visibility qualifier is a
/// miss. That is intentional and is not a spelling list: the property is "no
/// qualifier at all", which has exactly one spelling.
fn retained_body_helper_is_private(core: &str) -> bool {
    core.lines()
        .any(|line| line.trim() == "fn retained_body_occurrence(")
}

/// **`RT-FNSPLIT-B2O` `AC-12` split row — the DECLARATION survives, the
/// REACHABILITY entailment does not.** Architect ruling `evt_5yxjd1zqnyvcq`.
///
/// This pin is what remains of the withdrawn route oracle, and the boundary is
/// the point of it:
///
/// - **MEASURED:** `retained_body_occurrence` is declared in `lowering/core.rs`
///   with **no visibility qualifier**. Source text is authoritative for its own
///   declarations, so this is a fact the scan can settle.
/// - **CLAIMED:** exactly that, and nothing further.
/// - **THE GAP:** ⛔ this does **not** establish which functions can *reach* the
///   helper. The withdrawn oracle made that inference — *"`mod.rs` therefore
///   cannot reach it, so the route inventory is still correct"* — and
///   reachability is not a property of declaration text. Name resolution, macro
///   expansion, and indirect calls all sit outside what any source scan sees.
///
/// ⇒ **The authority for boundaries is the plan graph** — an occurrence's
/// `StaticOriginId`, its validated `SemanticOwner`, and the planned edge kind —
/// **not this file's text.** A Rust wrapper or a same-named method in another
/// `impl` creates no Ken function-unit boundary, so no pin here should redden
/// when one is added; see `b2o_ac10c_repointing_a_static_body_edge_changes_the_
/// disposition` for the axis that *is* authority.
///
/// Promise class: **normative compatibility vector** — the unqualified spelling
/// is the contract, and widening it is a deliberate review event.
#[test]
fn the_retained_body_helper_carries_no_visibility_qualifier() {
    let core = LOWERING_IMPL_SOURCES
        .iter()
        .find(|(file, _)| *file == "lowering/core.rs")
        .map(|(_, source)| *source)
        .expect("the impl-source list must carry core.rs");
    assert!(
        retained_body_helper_is_private(core),
        "`retained_body_occurrence` no longer declares as the exact unqualified \
         form in `lowering/core.rs`.\n\
         ⛔ DO NOT 'fix' this by accepting the new spelling -- that is the \
         evasion this WP paid five review folds to learn. A visibility \
         qualifier here is a DELIBERATE widening and belongs in review, with \
         the frozen evidence in the D6 report updated to match.\n\
         ⚠ This pin makes NO claim about who can reach the helper; that is the \
         plan graph's to answer, not this file's."
    );

    // The helper is declared in exactly one of the `impl Lowering` sources. This
    // is a DECLARATION inventory over both files -- it says where the helper is
    // written, never who can call it.
    let declaring = LOWERING_IMPL_SOURCES
        .iter()
        .filter(|(_, source)| retained_body_helper_is_private(source))
        .map(|(file, _)| *file)
        .collect::<Vec<_>>();
    assert_eq!(
        declaring,
        vec!["lowering/core.rs"],
        "the retained-body helper's DECLARING file set changed. ⚠ A declaration \
         in a second `impl Lowering` source is a review event; this pin reports \
         it and draws no conclusion about reachability."
    );

    // Non-vacuity: the needles must be real files, or both assertions above are
    // satisfied by an empty read.
    for (file, source) in LOWERING_IMPL_SOURCES {
        assert!(
            source.len() > 10_000,
            "`{file}` did not load; the assertions above would pass vacuously"
        );
    }
}

fn identifier_occurrences(source: &str, identifier: &str) -> usize {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(|code| code.split(|c: char| !c.is_alphanumeric() && c != '_'))
        .filter(|token| *token == identifier)
        .count()
}

#[test]
fn the_identifier_census_survives_the_evasions_that_defeated_the_text_scan() {
    // ⭐ Positive control built from the Architect's own two mutations. If the
    // census cannot see these, the count above asserts nothing.
    let split_across_lines = "let _second = self\n    .static_transition_plan\n    .source_occurrence\n    (static_origin)?;\n";
    assert_eq!(
        identifier_occurrences(split_across_lines, "source_occurrence"),
        1,
        "a mention split across lines must still be counted"
    );
    let path_form = "let _ = StaticTransitionPlan::source_occurrence(plan, origin)?;\n";
    assert_eq!(identifier_occurrences(path_form, "source_occurrence"), 1);
    // The plural FIELD must not be conflated with the resolver.
    assert_eq!(
        identifier_occurrences("self.plan.source_occurrences.len()\n", "source_occurrence"),
        0,
        "`source_occurrences` is a different identifier"
    );
    // Prose must not satisfy or inflate the census.
    assert_eq!(
        identifier_occurrences("// calls source_occurrence here\n", "source_occurrence"),
        0
    );
    assert_eq!(
        identifier_occurrences(
            "/// `source_occurrence` is the sole route\n",
            "source_occurrence"
        ),
        0
    );
}

/// The backend's complete production source surface — the census's **closure
/// proof**, not a convenience list.
///
/// ⭐ Why this is a proof rather than an enumeration someone must remember: a Rust
/// file is compiled only if an ancestor module declares it with `mod`. So pinning
/// every production `mod` declaration across the backend pins the *file set*, and
/// a thirteenth backend file cannot be compiled without reddening
/// `the_backend_production_surface_inventory_is_closed` below — which is what
/// forces whoever adds it to extend this list.
#[cfg(test)]
const BACKEND_PRODUCTION_SOURCES: &[(&str, &str)] = &[
    (
        "cranelift_backend.rs",
        include_str!("../../../../cranelift_backend.rs"),
    ),
    ("artifact/api.rs", include_str!("../../../artifact/api.rs")),
    ("artifact/mod.rs", include_str!("../../../artifact/mod.rs")),
    ("compiled.rs", include_str!("../../../compiled.rs")),
    ("lowering/core.rs", include_str!("../../core.rs")),
    ("lowering/mod.rs", include_str!("../../mod.rs")),
    // `RT-FNSPLIT-B2F` `D1`/`D2` — the target code-unit population. Registered
    // here the moment the module exists, because every pin that iterates this
    // roster is closed only over the files it lists: a production emitter absent
    // from it is invisible to all of them at once, which is precisely how
    // `boundary_value_clif.rs` and `native_int_clif.rs` came to sit outside
    // both this roster and the emitted-unit census.
    ("lowering/units.rs", include_str!("../../units.rs")),
    // `RT-FNSPLIT-B2F` `D3` — the artifact-static seed material. Registered for
    // the same reason as `units.rs` above, and ⭐ **it is the file that made the
    // reason concrete**: this module mints DATA objects, and until `AC-2` was
    // amended no needle in the census could see a data object at all. A file
    // outside this roster is invisible to every pin that iterates it; a file
    // inside it whose emission spelling nobody enumerated is invisible to the
    // census while looking fully measured.
    (
        "lowering/seed_material.rs",
        include_str!("../../seed_material.rs"),
    ),
    ("planning.rs", include_str!("../../../planning.rs")),
    (
        "planning/static_transition.rs",
        include_str!("../../../planning/static_transition.rs"),
    ),
    (
        "planning/static_transition/abi.rs",
        include_str!("../../../planning/static_transition/abi.rs"),
    ),
    (
        "planning/static_transition/semantic_ir.rs",
        include_str!("../../../planning/static_transition/semantic_ir.rs"),
    ),
    ("surface.rs", include_str!("../../../surface.rs")),
    ("test_objects.rs", include_str!("../../../test_objects.rs")),
    ("test_support.rs", include_str!("../../../test_support.rs")),
];

#[test]
fn the_backend_production_surface_inventory_is_closed() {
    // Every production `mod` declaration reachable in the backend, paired with the
    // file that declares it. `mod tests;` is excluded: a sibling test module is not
    // production surface, and its absence from the census is the point.
    let mut declared = Vec::new();
    for (file, source) in BACKEND_PRODUCTION_SOURCES {
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(*source, |(before, _)| before);
        for line in production.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || !trimmed.ends_with(';') {
                continue;
            }
            let Some(rest) = trimmed.strip_suffix(';') else {
                continue;
            };
            let Some(name) = rest.rsplit_once("mod ").map(|(_, name)| name) else {
                continue;
            };
            if name == "tests" || name.contains(' ') {
                continue;
            }
            declared.push((*file, name));
        }
    }
    assert_eq!(
        declared,
        vec![
            ("cranelift_backend.rs", "artifact"),
            ("cranelift_backend.rs", "compiled"),
            ("cranelift_backend.rs", "lowering"),
            ("cranelift_backend.rs", "planning"),
            ("cranelift_backend.rs", "surface"),
            ("cranelift_backend.rs", "test_objects"),
            ("cranelift_backend.rs", "test_support"),
            ("artifact/mod.rs", "api"),
            ("lowering/mod.rs", "core"),
            // `RT-FNSPLIT-B2F` `D1`/`D2`. A sibling of `core` rather than a
            // region inside it: `core.rs` is the module whose recursive
            // whole-configuration authority `D6` removes, and putting the
            // replacement population in the same file would leave the census
            // that measures the removal unable to tell the two apart.
            ("lowering/mod.rs", "units"),
            // `RT-FNSPLIT-B2F` `D3`. A sibling of `units` rather than a region
            // inside it, because the two mint DIFFERENT POPULATIONS on
            // different growth axes: `units` mints code, Θ(n) in the program;
            // this mints data, Θ(|seed environment|) and independent of the
            // program. Folding them into one file would put two growth axes
            // behind one census row.
            ("lowering/mod.rs", "seed_material"),
            ("planning.rs", "static_transition"),
            ("planning/static_transition.rs", "abi"),
            ("planning/static_transition.rs", "semantic_ir"),
        ],
        "AC-4 -- the backend's module inventory changed, so \
         BACKEND_PRODUCTION_SOURCES is no longer the whole production surface and \
         the sole-consumer census above has stopped being closed. Add the new \
         file to that list."
    );
    assert_eq!(
        declared.len() + 1,
        BACKEND_PRODUCTION_SOURCES.len(),
        "AC-4 -- every declared module must appear in the census list exactly once \
         (+1 for `cranelift_backend.rs`, the root, which no `mod` line declares)"
    );
}

// ─── RT-FNSPLIT-B2A-C AC-1 — uniform threading, shown not asserted ────────
//
// ⛔ A prose claim that "the fallback is covered" does not discharge AC-1. This
// reads the DECLARATIONS of the three source-term carriers and pins two
// properties structurally:
//
//  1. no field in any of them is a bare `RuntimeExpr` / `Vec<RuntimeExpr>` --
//     every carried term is an occurrence pair, so a frame cannot hold a term
//     whose origin was dropped;
//  2. every variant that carries a `cases` list also declares the parent
//     `static_origin` its case bodies are derived from.
//
// Both are declaration-level, not substring-level: a mention of `RuntimeExpr` in
// a comment or in this test's own message cannot satisfy or break them.

#[cfg(test)]
fn declaration_span(source: &'static str, header: &str) -> Vec<&'static str> {
    let start = source
        .find(header)
        .unwrap_or_else(|| panic!("{header} is declared in the lowering facade"));
    let mut depth = 0usize;
    let mut span = Vec::new();
    for line in source[start..].lines() {
        span.push(line);
        depth += line.matches('{').count();
        depth -= line.matches('}').count();
        if depth == 0 && span.len() > 1 {
            break;
        }
    }
    span
}

/// The bare-source-term predicate, factored out so it can be given a positive
/// control. ⚠ Without one, this whole pin would pass for the trivial reason that
/// it finds nothing — a negative check passes for any reason.
#[cfg(test)]
fn is_bare_source_term_field(line: &str) -> bool {
    let field = line.trim();
    field == "expr: RuntimeExpr,"
        || field == "body: RuntimeExpr,"
        || field == "then_expr: RuntimeExpr,"
        || field == "else_expr: RuntimeExpr,"
        || field == "remaining: Vec<RuntimeExpr>,"
        || field == "args: Vec<RuntimeExpr>,"
}

#[test]
fn the_bare_source_term_detector_catches_the_shape_it_is_looking_for() {
    // The pre-B2A-C declarations, verbatim. If the detector cannot see these it
    // is asserting nothing about the post-B2A-C ones.
    for pre_amendment in [
        "        expr: RuntimeExpr,",
        "        body: RuntimeExpr,",
        "        then_expr: RuntimeExpr,",
        "        remaining: Vec<RuntimeExpr>,",
        "        args: Vec<RuntimeExpr>,",
    ] {
        assert!(
            is_bare_source_term_field(pre_amendment),
            "the AC-1 detector must catch {pre_amendment:?}"
        );
    }
    assert!(!is_bare_source_term_field(
        "        expr: OwnedSourceOccurrence,"
    ));
    assert!(!is_bare_source_term_field(
        "    // a comment naming RuntimeExpr"
    ));
}

#[test]
fn every_source_term_carrier_holds_an_occurrence_and_never_a_bare_expression() {
    let source = include_str!("../../mod.rs");
    for header in [
        "enum SourceContinuation<'a> {",
        "enum SourcePrefixTemplate {",
        "enum SourceMachineState<'a> {",
    ] {
        let span = declaration_span(source, header);
        let bare: Vec<&str> = span
            .iter()
            .copied()
            .filter(|line| is_bare_source_term_field(line))
            .collect();
        assert!(
            bare.is_empty(),
            "AC-1: {header} still carries a bare source term without its origin: {bare:?}"
        );

        // Every `cases`-bearing variant declares its parent origin. The variant
        // boundary is a field list, so scan forward from each `cases:` line to
        // the variant's closing brace.
        let mut index = 0;
        while index < span.len() {
            if span[index].trim().starts_with("cases: Vec<") {
                let variant_tail = span[index..]
                    .iter()
                    .take_while(|line| !line.trim().starts_with("},"))
                    .any(|line| line.trim() == "static_origin: StaticOriginId,");
                assert!(
                    variant_tail,
                    "AC-1: {header} has a `cases` variant with no `static_origin`; \
                     its case bodies would have no parent to derive from"
                );
            }
            index += 1;
        }
    }
}

// ─── RT-FNSPLIT-B2A-S AC-1/AC-6 — the retained-body carrier holds a NAME ──────
//
// ⛔ AC-1 asks for this structurally, not asserted. It reads the DECLARATIONS of
// the retained-closure variants, so a mention of `OwnedSourceOccurrence` in a
// comment can neither satisfy nor break it, and it states the covered population
// (AC-6) per variant in the assertions themselves.

/// Every field a variant declares, in order.
///
/// ⛔ The **whole inventory**, not a search for known-bad spellings. The first
/// candidate matched three exact `body:` spellings, and the Architect rejected it
/// (`evt_6sq2tq3v9jcd0`): a compile-preserving `cached_body: RuntimeExpr` or
/// `retained: Box<RuntimeExpr>` beside `body: StaticOriginId` evaded it entirely
/// once the construction and pattern sites were updated. A detector enumerating
/// what it forbids can only ever be as complete as the enumeration; pinning what
/// is **allowed** rejects every added field regardless of name or type.
#[cfg(test)]
fn declared_fields(source: &'static str, header: &str) -> Vec<&'static str> {
    declaration_span(source, header)
        .into_iter()
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('}'))
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with("///"))
        .collect()
}

#[test]
fn the_field_inventory_extractor_sees_an_added_term_field() {
    // Positive control on the extractor, using the exact evasion the Architect
    // named. If the extractor cannot see this field, the equality assertion below
    // is not actually closed over "no additional term carrier".
    let synthetic =
        "    Evasion {\n        body: StaticOriginId,\n        cached_body: RuntimeExpr,\n    },\n";
    let fields = declared_fields(
        Box::leak(synthetic.to_string().into_boxed_str()),
        "    Evasion {",
    );
    assert_eq!(
        fields,
        vec!["body: StaticOriginId,", "cached_body: RuntimeExpr,"],
        "the extractor must report EVERY declared field, so an added one breaks \
         the inventory equality"
    );
}

#[test]
fn retained_closures_carry_a_static_origin_and_no_body_term() {
    let source = include_str!("../../mod.rs");

    // AC-6, the COVERED population: both variants that retained a body. Pinned as
    // a complete field inventory, so ANY added field -- term-bearing or not --
    // reddens and has to be justified here.
    assert_eq!(
        declared_fields(source, "    Closure {"),
        vec![
            "captures: Vec<Lowered>,",
            "params: Vec<String>,",
            "body: StaticOriginId,",
        ],
        "AC-1: `Lowered::Closure`'s field inventory changed. A second body \
         authority beside the tag is exactly what this WP removed, so an added \
         field must be argued, not absorbed"
    );
    assert_eq!(
        declared_fields(source, "    DeclarationClosure {"),
        vec![
            "symbol: RuntimeSymbol,",
            "captures: Vec<Lowered>,",
            "params: Vec<String>,",
            "body: StaticOriginId,",
        ],
        "AC-1: `Lowered::DeclarationClosure`'s field inventory changed"
    );

    // AC-6, the EXCLUDED variant, and why — a fact about the declaration rather
    // than a judgement call: it carries no source term at all. Pinned as an
    // inventory for the same reason as above, so it cannot quietly acquire one.
    assert_eq!(
        declared_fields(source, "    ComputationalRecursorClosure {"),
        vec![
            // ⚠ `RT-FNSPLIT-C1 AC-C4` widened this field from `Box<Lowered>` on
            // the Architect's SINGLE-FIELD license, and the pin's own property
            // is UNCHANGED by that: a `LoweringOperand` residual is still not a
            // body carrier — no `StaticOriginId`, no `RuntimeExpr`, nothing this
            // variant could be re-lowered from. It stays out of the covered
            // population for exactly the reason stated below. ⛔ What would move
            // it in is a field naming a source body, and that is still what this
            // inventory equality catches.
            "residual: Box<LoweringOperand>,",
            "activation: ContinuationActivationId,",
            "invocation: RecursorInvocationSegment,",
        ],
        "AC-6: ComputationalRecursorClosure is out of the covered population \
         because it declares no body carrier. If it acquires one it JOINS the \
         population, and this test is where that has to be said"
    );
}

/// **`RT-FNSPLIT-B2A-S` D2 — the plan cannot escape into the compiled artifact,
/// shown by the type system rather than asserted in prose.**
///
/// The plan now **borrows** the source trees, so non-escape stopped being
/// incidental and became load-bearing. `CompiledModule<M>` has no lifetime
/// parameter, so it cannot store a borrow of them; requiring `'static` is exactly
/// that claim, checked by the compiler.
///
/// ⭐ Falsifiable by mutation in the ordinary D6 way: give `CompiledModule` a
/// `&'src RuntimeExpr` field and this test **stops compiling**. That is a stronger
/// failure mode than a red assertion — the escape cannot be introduced and then
/// argued about.
#[test]
fn escaping_a_source_borrow_into_the_compiled_artifact_does_not_typecheck() {
    fn holds_no_borrowed_state<T: 'static>() {}
    holds_no_borrowed_state::<CompiledModule<cranelift_jit::JITModule>>();
}

// ─── RT-FNSPLIT-B2A-S AC-5 — nothing is KEYED by a scheduling entry ───────────
//
// ⭐ Why this needs a pin even though the resolver takes a `StaticOriginId`:
// hard-stop #8 was a category error in which a scheduling entry stood in for a
// source occurrence, and a `ComputationalMatch` SHARES its scheduling entry with
// its scrutinee chain. So a collection keyed by an entry looks perfectly injective
// on every fixture without one — the wrong key still looks unique — and then
// silently merges two occurrences on the fixture that has one.
//
// ⛔ **This scan is a tripwire, not a discharge — and neither are the behavioural
// controls, on their own.** Two Architect blocks established that, and the second
// (`evt_1p11krxny4wny`) is the one that settles it: a real
// `Vec<Option<&RuntimeExpr>>` indexed by `usize::try_from(scrutinee.entry.0)` at
// the `ComputationalMatch` seam **compiles and passes all three nets**.
//
// ⇒ The framed property, *"no collection is keyed by `.entry`, and a mutation
// introducing one reddens,"* is a **global negative over arbitrary code shapes**.
// No test enforces that: detecting it needs dataflow, not a scan, and a scan can
// always be spelled around. So the honest split is recorded here rather than
// papered over with a longer list:
//
// The authoritative AC-5 is the Architect's four clauses (`origin/main`
// `d0b6e064`, transcribed verbatim there); this is what discharges each:
//
//   (a) concrete entry-carrying types stay module-private
//         -> `the_entry_carrying_types_are_module_private`
//   (b) a non-vacuous split fixture proves entry-keying selects the wrong body
//         -> `keying_selection_by_the_scheduling_entry_does_not_resolve_the_body`
//   (c) a compile-preserving re-key of the sanctioned table reddens at the
//       collision/invariant controls
//         -> `filing_two_occurrences_under_one_origin_is_refused`
//   (d) Architect review of the closed two-file planner surface and its exports
//       confirms the stated residual -- review, not a test.
//
// ⛔ **BOTH residual arms, because recording one reads as if the other were
// covered:**
//
//   RESIDUAL 1 — an independently maintained entry-keyed collection INSIDE the
//     two planner files. Inside the planner, entry-keying is the planner's own
//     job and is NOT prohibited; what is unenforceable is detecting a *second*
//     selection authority built from it.
//   RESIDUAL 2 — exported / inferred / ordinal entry exposure. A future method
//     could hand out an entry as `impl Ord` (`StaticNodeId` already derives
//     `Ord`) or as a derived `u32` ordinal, **naming neither private type**, so
//     (a) would still hold while an outside consumer keyed on an entry anyway.
//
// ⛔ **Do not claim that an arbitrary independently maintained entry-keyed
// collection is mechanically detected.** No test enforces that: detecting it
// needs dataflow, not a scan, and a scan can always be spelled around.

/// ⚠ Positive control for the AC-5 detector: it must actually recognise the shape
/// it claims nothing matches, or "no matches" means nothing.
#[cfg(test)]
fn declares_collection_keyed_by_node_id(line: &str) -> bool {
    [
        "BTreeMap<StaticNodeId",
        "BTreeSet<StaticNodeId",
        "HashMap<StaticNodeId",
        "HashSet<StaticNodeId",
    ]
    .iter()
    .any(|shape| line.contains(shape))
}

#[test]
fn the_entry_keyed_collection_detector_catches_the_shape_it_is_looking_for() {
    assert!(declares_collection_keyed_by_node_id(
        "    scheduled: BTreeMap<StaticNodeId, RuntimeExpr>,"
    ));
    assert!(declares_collection_keyed_by_node_id(
        "    seen: BTreeSet<StaticNodeId>,"
    ));
    // The admissible neighbour: keyed by the OCCURRENCE, which B1R's
    // `origin.0 == planned_node.0` bijection makes safe.
    assert!(!declares_collection_keyed_by_node_id(
        "    occurrences: BTreeMap<StaticOriginId, RuntimeExpr>,"
    ));
}

/// **AC-5(a) — the concrete entry-carrying types are module-private.**
///
/// `PlannedExpr` and `StaticNodeId` are declared with **no `pub` modifier**, so
/// they are private to `planning::static_transition` (`StaticNodeId` reaching its
/// own `semantic_ir` child through `use super::`). The set of production files
/// that can *name* either type is therefore exactly those two.
///
/// ## ⚠ What is measured, and what is NOT claimed
///
/// **Measured:** the privacy of two concrete types, i.e. which files can name
/// them. **Not claimed:** that selection authority is confined, or that no
/// outside code can key on a scheduling entry.
///
/// ⛔ **The implication between those two is invalid, and asserting it was my
/// defect** (struck by Steward ruling `evt_4dh098a49cbze`; the earlier version of
/// this test said privacy meant *"none can key on one"* and encoded that claim in
/// its own name). Privacy of a *name* does not confine a *value*: a future method
/// could hand an entry out as `impl Ord` — `StaticNodeId` already derives `Ord` —
/// or as a derived `u32` ordinal, **naming neither private type**, and this test
/// would still pass while an outside consumer keyed on an entry.
///
/// ⇒ This pin is clause **(a)** of four. (b) and (c) are the behavioural and
/// collision controls in the planner; **(d) is Architect review**, and the two
/// residual arms above are what that review covers.
#[test]
fn the_entry_carrying_types_are_module_private() {
    let mut naming = Vec::new();
    for (file, source) in BACKEND_PRODUCTION_SOURCES {
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(*source, |(before, _)| before);
        // Tokenized with comments stripped, so a doc comment MENTIONING the type
        // (as `lowering/core.rs` does, twice, while being unable to name it) does
        // not count.
        let mentions = identifier_occurrences(production, "PlannedExpr")
            + identifier_occurrences(production, "StaticNodeId");
        if mentions > 0 {
            naming.push(*file);
        }
    }
    assert_eq!(
        naming,
        vec![
            "planning/static_transition.rs",
            "planning/static_transition/abi.rs",
            "planning/static_transition/semantic_ir.rs",
        ],
        "AC-5(a): another backend file now NAMES an entry-carrying type. That is \
         the measured fact only -- it does not by itself decide whether anything \
         keys on an entry, which is residual arm 2 and Architect review.\n\
         `abi.rs` joined this inventory in `RT-FNSPLIT-B2R`: the ABI plane names \
         `StaticNodeId` because a function unit's frame entry IS its seed node, \
         and the descriptor records which node that is. It remains module-private \
         and is not widened."
    );

    // The naming set is what it is because the declarations are module-private. A
    // `pub` of any width would widen it without changing any call.
    let planner = include_str!("../../../planning/static_transition.rs");
    assert!(
        planner.contains("\nstruct PlannedExpr {"),
        "AC-5: `PlannedExpr` must stay module-private"
    );
    assert!(
        planner.contains("\nstruct StaticNodeId(u32);"),
        "AC-5: `StaticNodeId` must stay module-private"
    );
}

#[test]
fn no_collection_is_keyed_by_a_scheduling_entry() {
    // Over the CLOSED backend surface, not a hand-picked four files: the resolver
    // and the plan are reachable from every backend sibling, so a tripwire scoped
    // to `lowering/` and `planning/` would miss `artifact/**` and `compiled.rs`.
    for (file, source) in BACKEND_PRODUCTION_SOURCES {
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(*source, |(before, _)| before);
        let keyed: Vec<&str> = production
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .filter(|line| {
                declares_collection_keyed_by_node_id(line)
                    // The index form the Architect named: a positional table
                    // subscripted by a scheduling entry rather than an occurrence.
                    || line.contains(".entry.0 as usize")
                    || line.contains("[entry.0 as usize]")
            })
            .collect();
        assert!(
            keyed.is_empty(),
            "{file} keys or indexes by a scheduling entry {keyed:?}; a \
             ComputationalMatch shares its entry with its scrutinee chain, so this \
             merges two occurrences on exactly the fixture that has one. \
             ⚠ NOT an AC-5 clause: this is an early tripwire over enumerated \
             forms, and AC-5 is discharged by (a)-(d) above"
        );
    }
}

// ─── RT-FNSPLIT-B2O D6/D7 — the call population, and inertness ─────────────

/// **`RT-FNSPLIT-B2O` `D6` — the `lower_expr` call population, and why its
/// disposition is now BY OWNER rather than by source site.**
///
/// ⛔ **This is a report, not the authority, and this pin is FROZEN DECLARATION
/// EVIDENCE.** The authority is the ownership mapping in the semantic plane —
/// an occurrence's `StaticOriginId`, its validated `SemanticOwner`, and the
/// planned edge kind.
///
/// ⚠ An earlier revision said this pin existed so the population *"cannot drift
/// silently."* **It does not establish that**, and the claim is withdrawn: the
/// census counts textual occurrences of an identifier, which is a declaration
/// fact. It observes nothing about which Rust functions can reach a retained
/// body. See the `D6` report's UNMECHANIZED section for the four residuals.
///
/// ⚠ **The census is TOKENIZED, not `self.`-spelled.** `grep -c
/// 'self\.lower_expr('` returns **58** and silently loses the program's entry
/// point: the root call is spelled `compiler.lower_expr(` (`core.rs:188`) and
/// takes `root_static_origin`, so it *seeds* the descent rather than traversing.
/// A receiver spelling is a census of the RECEIVER, and the call it misses is the
/// one that matters most.
///
/// ⭐ **Why the count is asserted as two measurements and the 59 is DERIVED.**
/// Freezing "59" directly would be a snapshot. Instead this pins the token total
/// and the definition count, and subtracts — so the pin states the *relation*
/// `calls = tokens - definitions`, and a call added or removed reddens with an
/// arithmetic explanation rather than a bare number mismatch.
///
/// ### The disposition, derived from the ownership mapping
///
/// `B2O` makes a `StaticBody` edge the **one and only** owner boundary. So a call
/// into `lower_expr` crosses an owner boundary **iff the occurrence it lowers is
/// a `StaticBody` target — that is, iff it lowers a retained body.** ⇒ The test
/// is on the **occurrence's owner and the planned edge kind**, and on nothing
/// else.
///
/// ⛔ **Withdrawn here:** that retained bodies are *"reachable only through the
/// single `origin -> expression` route"* and that the population is
/// *"characterised structurally, by one pinned route."*
/// `exactly_one_plan_origin_to_expression_lookup_exists` constrains the
/// identifier `source_occurrence` **only** — it says nothing about who may call
/// `retained_body_occurrence`, so it never supported either sentence.
///
/// ⇒ **The boundary-crossing population is derived from the validated owner
/// partition**, instead of enumerated as a table of source sites. That is the
/// repair for the withdrawn `AC-5`: its two-way site classification had no cell
/// for "depends on the reaching path", so it could have been filled in completely
/// and still been wrong. For the 14 caller-dependent sites the answer genuinely
/// *is* a function of the reaching path — the same parameter carries both a
/// retained body and ordinary sub-expressions — and no per-site row can say that.
/// The **validated owner partition** can: an occurrence's `StaticOriginId`, its
/// `SemanticOwner`, and the planned edge kind answer it per occurrence, which is
/// the only authority here.
#[test]
fn the_lower_expr_call_population_is_dispositioned_by_owner_not_by_site() {
    // Promise class: durable invariant — a relation over the production source,
    // not a frozen count. `tokens` and `definitions` each move for a stated
    // reason; `calls` is their difference.
    let core = include_str!("../../core.rs");
    let tokens = identifier_occurrences(core, "lower_expr");
    let definitions = core
        .lines()
        .filter(|line| line.trim() == "fn lower_expr(")
        .count();
    assert_eq!(
        definitions, 1,
        "D6: there must be exactly one `lower_expr` definition for the call \
         count to be `tokens - definitions`"
    );
    let calls = tokens - definitions;
    // ⭐ **59 -> 61 on `RT-FNSPLIT-C1` `D3`, then 61 -> 62 on
    // `RT-FNSPLIT-C2-SYNTH-ID`, and the arithmetic is the whole
    // report the pin asks for.** The two added calls are the case-body descents
    // of the two *carried* elimination routes — `lower_carried_match` and
    // `lower_carried_computational_match` — each lowering a case body under a
    // `case_env` whose binders are runtime projections rather than compile-time
    // constructor arguments. C2 adds the HostResult-specific carried case-body
    // descent: the runtime success bit chooses the Result case, while the
    // selected payload remains a carried operand in that case's environment.
    //
    // ⭐ **Neither is a new owner boundary**, which is the disposition this pin
    // actually reports. A carried case body is reached by ordinary descent from
    // the eliminator's own occurrence — `case_body_occurrence(static_origin,
    // index, ..)`, the identical accessor the specialized routes use — so its
    // occurrence's `SemanticOwner` and planned edge kind are unchanged. ⛔ No
    // `StaticBody` edge is introduced, and no retained body is reached by a new
    // path.
    assert_eq!(
        calls, 62,
        "D6: the tokenized production call population into `lower_expr` moved. \
         ⚠ If you reached this by counting `self.lower_expr(` you will have got \
         one fewer -- the root call at `core.rs:188` is spelled \
         `compiler.lower_expr(`"
    );

    // Non-vacuity: the tokenizer must actually see the root call's receiver
    // spelling, or the paragraph above is describing something the pin cannot
    // measure.
    assert!(
        core.contains("compiler.lower_expr("),
        "D6: the root call's spelling is gone, so this census no longer \
         distinguishes the entry point from traversal"
    );

    // ⭐ The DISCRIMINATOR, on a shared input: a non-degenerate pair where the
    // tokenizer and the receiver-spelled scan give different answers. Without
    // this, "use the tokenizer" is advice rather than a checked property — and a
    // positive control that only exercises `self.` would be spelling-scoped in
    // exactly the way that produced 58.
    let both_receivers =
        "let a = self.lower_expr(b, o, e)?;\nlet c = compiler.lower_expr(b, o, e)?;\n";
    assert_eq!(
        identifier_occurrences(both_receivers, "lower_expr"),
        2,
        "the census must count a call regardless of its receiver"
    );
    assert_eq!(
        both_receivers.matches("self.lower_expr(").count(),
        1,
        "if the receiver-spelled scan agreed with the tokenizer here, this pair \
         would not discriminate and would prove nothing about the 58/59 gap"
    );

    // ⚠ Honest limit, recorded next to the enforced statement rather than left
    // for the next reader to discover: this census does NOT partition out
    // `core.rs`'s 22 inline `#[cfg(test)]` regions, so a call added inside one
    // would be counted as production. That errs toward a FALSE RED, never a
    // false green, so it is the safe direction — but it is a limit, not a
    // property, and "production" here means "textually in the production file".
    assert!(
        core.contains("#[cfg(test)]"),
        "the caveat above describes inline cfg(test) regions that are no longer \
         present, so it has gone stale and must be re-derived"
    );
}

/// **`RT-FNSPLIT-B2O` `D7`/`AC-1` — inertness, as reach rather than as a builder
/// count.**
///
/// The emitted-unit census (`correspondence_adds_no_emitted_unit_to_the_production_census`)
/// already pins `1` builder / `1` definition / `2` declarations in `core.rs` and
/// zero everywhere else, and it counts **source text**, which is why it discharges
/// `AC-1`'s "in BOTH configurations" rather than needing a per-`cfg` variant:
///
/// > **MEASURED:** text occurrences of the builder/definition/declaration forms
/// > across each whole production file, `#[cfg(test)]` regions included.
/// > **CLAIMED:** production emits no new unit under `cfg(test)` or without it.
/// > **THE GAP:** none in the strict direction — any unit emitted in *either*
/// > configuration must appear in the text, so a text census is a superset of
/// > both. It is stricter than the AC, not weaker.
///
/// ⛔ **But a builder census cannot see an executable edge, and it was already
/// zero before this node**, so on its own it is a check that would pass whether
/// or not `B2O` stayed inert.
///
/// ⛔ **Withdrawn:** an earlier revision presented what follows as *"two
/// mechanisms"* proving *"no emission edge is representable."* Neither
/// establishes that, and the pin does not claim it. What this pin is:
///
/// 1. **A visibility inventory (declaration).** `SemanticOwner` is
///    `pub(super)`, and this pin asserts the **allowed inventory** of widened
///    items rather than a forbidden list, so *any* new widening reddens —
///    including one nobody imagined. ⚠ The hatch is not hypothetical:
///    `StaticOriginId` went through it deliberately. ⚠ But visibility bounds
///    **naming**, not reaching: a type is reachable through a method that
///    returns it, an `impl Trait`, or a re-export without ever being named.
/// 2. **A naming inventory (declaration).** `SemanticOwner` appears **zero**
///    times in the production region of every backend source except the file
///    that defines it. This makes a new mention **visible to review**; it is not
///    a proof of unreachability.
///
/// ⇒ **Inertness itself is pinned BEHAVIORALLY**, by
/// `correspondence_adds_no_emitted_unit_to_the_production_census` — that is the
/// mechanism that would actually observe an emission edge. These two are
/// declaration inventories that make a change loud, and that is their whole
/// claim.
#[test]
fn the_owner_classification_has_a_closed_production_naming_inventory() {
    // Promise class: durable invariant — a DECLARATION inventory.
    //
    // ⚠ RENAMED AGAIN by `RT-FNSPLIT-B2R`, and the rename is the honest part.
    // The previous name was `..._is_named_in_production_only_by_the_module_that_
    // defines_it`, and `B2R` **falsified that claim legitimately**: the ABI plane
    // consumes the validated owner partition, which is precisely what the `B2R`
    // frame mandates ("the population is `B2O`'s owner partition, consumed as
    // data"). A pin whose name asserts sole-consumership cannot survive the node
    // that adds the second consumer, and quietly widening the expected list while
    // keeping that name would leave a corrected body under an uncorrected name.
    //
    // ⇒ What is pinned now is the **closed allowed inventory** of production
    // files naming the classification. It still reddens on a *third* consumer —
    // including one nobody imagined — which is the property worth guarding. What
    // it no longer claims is that there is only one.
    //
    // ⚠ RENAMED under the Architect ruling (`evt_5yxjd1zqnyvcq`). This pin was
    // called `..._has_no_reach_into_any_emission_path`, and that name asserted an
    // inference the mechanism cannot make: a type can be *reached* without being
    // *named* — through a method that returns it, an `impl Trait`, a re-export,
    // or a derived ordinal. Naming is not capability. The name now states what
    // is actually measured, because the name is the part future readers quote.
    let mut naming = Vec::new();
    for (file, source) in BACKEND_PRODUCTION_SOURCES {
        // `static_transition.rs` carries its tests inline, and those tests
        // legitimately name the owner classification to exercise it.
        let production = source
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(*source, |(before, _)| before);
        let n = identifier_occurrences(production, "SemanticOwner");
        if n > 0 {
            naming.push(*file);
        }
    }
    assert_eq!(
        naming,
        vec![
            "planning/static_transition/abi.rs",
            "planning/static_transition/semantic_ir.rs",
        ],
        "D7: the owner classification's production naming inventory changed.\n\
         The two permitted members are the module that DEFINES it \
         (`semantic_ir`) and the `B2R` ABI plane (`abi`), which names it to \
         resolve a static-body boundary's CALLEE unit when deriving that \
         boundary's caller-side signature. A third file is a review event: say \
         why that consumer must name the classification rather than take a \
         descriptor.\n\
         ⚠ This membership moved twice inside `RT-FNSPLIT-B2R` and the history \
         is worth one line, because the second move is the load-bearing one. \
         `abi.rs` first named the type in a redundant edge-agreement check that \
         `AC-11` measured as unreachable and deleted -- at which point it left \
         this inventory. The Architect then established that the deleted \
         composition proved target IDENTITY and never layout AGREEMENT, so a \
         real per-boundary signature replaced it, and that mechanism genuinely \
         needs the classification. The name is here now for a live reason, not \
         a vestigial one.\n\
         ⚠ MEASURED: which production files mention the identifier. CLAIMED: \
         exactly that. THE GAP: a mention is not an executable edge and the \
         absence of one is not proof there is none -- a type can be reached \
         without being named. Inertness itself is pinned behaviorally by \
         `correspondence_adds_no_emitted_unit_to_the_production_census`; this \
         pin is a declaration inventory that makes a new mention VISIBLE to \
         review, not a proof of unreachability."
    );

    // The allowed inventory of widened visibility in the plane. ⛔ Asserted as
    // the exact permitted set, not as a scan for a forbidden spelling, so that
    // ANY new widening reddens -- including one nobody imagined.
    //
    // ⚠ This is a DECLARATION inventory. It records which items are widened; it
    // does not entail anything about what is representable or reachable, because
    // visibility bounds NAMING, not reaching.
    let plane = include_str!("../../../planning/static_transition/semantic_ir.rs");
    let widened = plane
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("//"))
        .filter(|line| line.contains("pub(in crate"))
        .collect::<Vec<_>>();
    assert_eq!(
        widened,
        vec![
            "pub(in crate::cranelift_backend) struct StaticOriginId(pub(super) u32);",
            "pub(in crate::cranelift_backend) struct ConstructorIdentity(pub(super) DenseRange);",
            "pub(in crate::cranelift_backend) enum SynthesizedFixedConstructorRole {",
            "pub(in crate::cranelift_backend) struct SynthesizedIoErrorRole(pub(super) u32);",
            "pub(in crate::cranelift_backend) enum SynthesizedConstructorRole {",
            "pub(in crate::cranelift_backend) struct FieldIdentity(pub(super) DenseRange);",
            "pub(in crate::cranelift_backend) fn tag_abi_word(self) -> Result<u64, CraneliftBackendError> {",
            "pub(in crate::cranelift_backend) fn name_abi_word(self) -> Result<u64, CraneliftBackendError> {",
            // ⭐ `RT-FNSPLIT-B2F` `D1` adds ONE member, and it is argued here
            // rather than absorbed, because this pin exists to make a widening a
            // review event.
            //
            // `B2F` emits one closed target function per `PredeclaredFunction`
            // in the validated owner partition. To do that the emitter must be
            // able to NAME a unit — to key its declared `FuncId` and to resolve
            // `D4`'s call edges against the planner's identity rather than
            // against an iteration ordinal. It must NOT be able to mint one.
            //
            // ⭐ The `pub(super)` field is what makes that a fact about the type
            // system: the newtype is widened, its `u32` is not, so `lowering`
            // can hold, compare, order and pass a unit identity and cannot
            // fabricate one or do arithmetic on it. That is the identical
            // argument `StaticOriginId` was widened on, and it transfers because
            // it is the same shape, not because it is nearby.
            //
            // ⛔ What is deliberately NOT widened, and is the thing this row
            // must not be read as licensing: `AbiPlane`, `AbiDescriptor`,
            // `build_abi_plane` and `AbiPlane::validate` all stay `pub(super)`.
            // The emitter reads one unit's projection; it cannot construct the
            // plane, mutate a descriptor, or reach the pre-emission validator to
            // bypass it. A future `AbiPlane` or `build_abi_plane` line appearing
            // in this list is the violation, not a further capability.
            "pub(in crate::cranelift_backend) struct PredeclaredFunctionId(pub(super) u32);",
            "pub(in crate::cranelift_backend) fn with_last_io_error_role_omitted<T>(",
        ],
        "D7: the plane's widened-visibility inventory changed. `StaticOriginId` \
         is widened deliberately so the lowering can carry an occurrence's \
         static name.\n\
         ⭐ `RT-FNSPLIT-C1` `D1`/`D2` adds four members, and the argument for \
         each is the same one that justifies `StaticOriginId`: the widened item \
         is a NAME the lowering may hold, never a CONSTRUCTOR it may use. Both \
         identity newtypes wrap a `pub(super)` field, so a consumer can hold, \
         compare and pass an identity but CANNOT MINT one -- which is what \
         makes `D2`'s single-authority property a fact about the type system \
         rather than about reviewer vigilance. `tag_abi_word`/`name_abi_word` \
         are widened because the carrier's emitted ABI takes a word; they are \
         METHODS ON THE TYPED IDENTITY rather than a shared `u64` conversion, \
         so neither namespace can be erased before the tag-vs-name ABI \
         operation is chosen.\n\
         ⭐ `RT-FNSPLIT-C2-SYNTH-ID` adds the closed fixed-role sum, the opaque \
         dynamic-role token, their closed key sum, and a cfg(test) omission \
         seam. The source tripwire cannot distinguish cfg(test), so it records \
         that seam without claiming production reachability. The IO token's \
         field remains parent-private and lowering can only receive one from \
         the plan.\n\
         ⛔ What is NOT widened, and is the thing this pin most needs to keep \
         catching: `SemanticPlane` and its `names` arena stay `pub(super)`. The \
         Architect's ruling forbids resolving a consumer's need by widening the \
         plane, and `D1` is deliberately a capability export instead. A future \
         `SemanticPlane` or `names` line appearing in this list is the \
         violation, not a fifth capability.\n\
         ⚠ This is a DECLARATION inventory, not a proof of inertness: a \
         widening of the OWNER surface is a DELIBERATE REVIEW EVENT that must \
         be argued here, not absorbed. It entails nothing by itself about what \
         is representable or reachable -- inertness is pinned behaviorally by \
         `correspondence_adds_no_emitted_unit_to_the_production_census`"
    );

    // Non-vacuity: the needle must occur somewhere, or both assertions above are
    // satisfied by a typo.
    assert!(
        identifier_occurrences(plane, "SemanticOwner") > 0,
        "the owner classification is not in the plane at all, so this pin is \
         measuring nothing"
    );
}

// ─── RT-FNSPLIT-B2V AC-3 — the Lowered disposition is exhaustive, no wildcard ─
//
// **MEASURED:** the `boundary_disposition` region of `lowering/mod.rs` contains
// no `_ =>` arm and names all 21 `Lowered` variants.
// **CLAIMED:** adding a 22nd variant is a COMPILE ERROR, so the transfer
// population is closed structurally rather than by the `#10` histogram.
// **THE GAP:** the compiler already guarantees exhaustiveness — what it cannot
// guarantee is that nobody *silences* it. A `_ =>` arm would make a new variant
// compile straight into whatever that arm returned. This pin exists for that
// one job, and it also checks the dispatch is single, since a second
// wildcarded dispatch elsewhere would be outside the compiler's guarantee too.

#[test]
fn b2v_ac3_the_lowered_boundary_disposition_has_no_wildcard_arm() {
    let source = include_str!("../../mod.rs");
    let region = source
        .split_once("fn boundary_disposition(self) -> BoundaryDisposition {")
        .map(|(_, after)| after)
        .and_then(|after| {
            after
                .split_once("#[derive(Clone)]\nstruct ActiveRecursiveDeclarationV1")
                .map(|(body, _)| body)
        })
        .expect("AC-3: the disposition region was not found, so every check below is vacuous");

    // ⚠ POSITIVE CONTROL FIRST. A negative check passes for any reason,
    // including an extractor that returned an empty region.
    assert!(
        region.contains("LoweredVariant::Constructor"),
        "AC-3: the extracted region does not contain a token that is certainly \
         in it, so its silence about `_ =>` means nothing"
    );

    // ⛔ Every arm head must name a `Lowered` variant.
    //
    // ⚠ This started as `!region.contains("_ =>")` and a compile-preserving
    // evasion DEFEATED it in one line: `unhandled => ...` is a binding
    // catch-all, so it silences exhaustiveness exactly like `_` while matching
    // no `_ =>` substring. The pin was green with the catch-all in place. What
    // the two evasions share is a GRANULARITY error — the check was a claim
    // about one spelling where the property is about the SHAPE of every arm —
    // so the repair is to enumerate arm heads rather than to add the second
    // spelling to a forbidden list that is open at the top.
    for line in region.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("=>") {
            continue;
        }
        assert!(
            trimmed.starts_with("LoweredVariant::") || trimmed.starts_with('|'),
            "AC-3: `{trimmed}` is a match arm whose head does not name a \
             `LoweredVariant`. A catch-all — `_` or a binding — silences \
             exhaustiveness, so a new variant would compile into it instead of \
             failing until someone dispositions it."
        );
    }

    // Every one of the 21 landed variants is named. Pinned as the ALLOWED
    // inventory: a variant renamed or removed reddens here with its own name in
    // the message, where a bare count would only say that something moved.
    for variant in [
        "Int",
        "Bool",
        "ProcessExitStatus",
        "CapabilityToken",
        "ResourceToken",
        "BoundedNat",
        "StructuralNat",
        "ResponseBytes",
        "HostResult",
        "DynamicConstructor",
        "Bytes",
        "BorrowedNativeValue",
        "BorrowedOption",
        "String",
        "Constructor",
        "Record",
        "Closure",
        "DeclarationClosure",
        "ComputationalRecursorClosure",
        "RecursiveBackedge",
        "Trap",
    ] {
        assert!(
            region.contains(&format!("LoweredVariant::{variant}")),
            "AC-3: `LoweredVariant::{variant}` has no disposition"
        );
    }

    // ⛔ `Constructor` and `HostResult` are REQUIRED LIVE ARMS. A disposition
    // that parked either in `FailClosedForbidden` would reject the dominant
    // measured population — sound, and unable to satisfy `B2F`'s `D6`/`D7`.
    // Checked positionally, so moving one into the forbidden block reddens.
    let forbidden_block = region
        .split_once("FailClosedForbidden {")
        .map(|(_, after)| after)
        .expect("AC-3: there is no fail-closed arm at all, which is itself wrong");
    for required_live in ["Constructor", "HostResult"] {
        assert!(
            !forbidden_block.contains(&format!("LoweredVariant::{required_live}")),
            "AC-3: `LoweredVariant::{required_live}` is a REQUIRED LIVE ARM and has \
             been moved behind the fail-closed boundary"
        );
    }

    // The dispatch is single: one definition, so the compiler's exhaustiveness
    // guarantee covers the whole question and not just this copy of it.
    assert_eq!(
        source.matches("fn boundary_disposition(self)").count(),
        1,
        "AC-3: a second disposition exists, and the compiler cannot promise \
         the two agree"
    );
}

// ─── RT-FNSPLIT-C1 D5 — closure admissibility is a property of the GRAPH ───

/// A real `StaticOriginId` for a closure fixture.
///
/// ⭐ One cannot be minted outside the planner — its ordinal is `pub(super)`,
/// which is exactly the unmintability `D1`/`D2` rely on. So a control that needs
/// a closure value must source a genuine origin from a genuine plan rather than
/// fabricating one, and that constraint is a feature reaching into the tests.
fn c1_closure_fixture_origin() -> StaticOriginId {
    let expr = RuntimeExpr::Construct {
        constructor: "ctor:prelude::Unit::MkUnit".to_string(),
        args: Vec::new(),
    };
    let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
        .expect("the fixture expression plans");
    plan.root_static_origin()
        .expect("the plan has a root occurrence origin")
}

fn c1_closure(origin: StaticOriginId) -> Lowered {
    Lowered::Closure {
        captures: Vec::new(),
        params: Vec::new(),
        body: origin,
    }
}

/// **`RT-FNSPLIT-C1` `D5` — a closure is inadmissible at the root and at every
/// depth, and the rejection is the same exact typed error at each.**
///
/// **MEASURED:** `boundary_transfer_admissibility` returns the closure-transfer
/// error for a bare closure, a bare declaration closure, a closure nested one
/// level inside a `Constructor`, and one nested two levels inside a
/// `Constructor` -> `Record`.
/// **CLAIMED:** admissibility is a property of the whole value graph.
/// **THE GAP:** the root variant table cannot see any of the nested cases —
/// `boundary_disposition` reports `RepresentedHandle` for every one of the
/// nested fixtures below, because it is a function of the root tag alone. That
/// disagreement is asserted here rather than described, so the walk cannot be
/// deleted in favour of the table without reddening.
#[test]
fn c1_d5_a_closure_is_inadmissible_at_the_root_and_at_every_depth() {
    // Promise class: durable invariant.
    let origin = c1_closure_fixture_origin();
    let expected = unsupported(
        "Closure",
        "a closure cannot cross the boundary: it is runtime-local and \
         live-domain only, and it has no durable lane",
    );

    let bare = c1_closure(origin);
    let bare_declaration = Lowered::DeclarationClosure {
        symbol: "decl:fixture::f".to_string(),
        captures: Vec::new(),
        params: Vec::new(),
        body: origin,
    };
    let depth_1 = Lowered::Constructor {
        constructor: "ctor:fixture::Box::MkBox".to_string(),
        synthesized_identity: None,
        args: vec![c1_closure(origin)],
    };
    let depth_2 = Lowered::Constructor {
        constructor: "ctor:fixture::Box::MkBox".to_string(),
        synthesized_identity: None,
        args: vec![Lowered::Record {
            fields: vec![("field:held".to_string(), c1_closure(origin))],
        }],
    };

    for (label, value) in [
        ("bare closure", &bare),
        ("bare declaration closure", &bare_declaration),
        ("closure nested at depth 1", &depth_1),
        ("closure nested at depth 2", &depth_2),
    ] {
        assert_eq!(
            value.boundary_transfer_admissibility().unwrap_err(),
            expected,
            "{label}: the graph holds a closure and must be refused with the \
             exact closure-transfer error"
        );
    }

    // ⭐ THE GAP, asserted. The two nested fixtures are exactly the cases the
    // root table cannot see, and it must be shown to disagree — otherwise this
    // whole walk could be replaced by `boundary_disposition` and nothing would
    // redden.
    for (label, value) in [
        ("closure nested at depth 1", &depth_1),
        ("closure nested at depth 2", &depth_2),
    ] {
        assert!(
            matches!(
                value.boundary_disposition(),
                BoundaryDisposition::RepresentedHandle { .. }
            ),
            "{label}: the ROOT table already refuses this, so the graph walk is \
             not what is catching it and this control proves nothing about depth"
        );
    }
}

/// **`RT-FNSPLIT-C1` `D5` — the positive path: a closure-free constructor is
/// still admitted.**
///
/// ⛔ This is the control that keeps the rejection **conditional**. Without it,
/// an implementation that refused every `Constructor` outright would satisfy
/// every negative control above — and it would be a capability removal wearing
/// a soundness fix's clothing.
///
/// ⚠ Admitted here means *"this graph holds no closure"*, **not** *"this value
/// is transferable"*. Whether the root has a boundary representation at all is
/// `boundary_disposition`'s separate question.
#[test]
fn c1_d5_a_closure_free_constructor_is_admissible() {
    // Promise class: durable invariant.
    let closure_free = Lowered::Constructor {
        constructor: "ctor:fixture::Pair::MkPair".to_string(),
        synthesized_identity: None,
        args: vec![
            Lowered::String("left".to_string()),
            Lowered::Record {
                fields: vec![("field:right".to_string(), Lowered::Bytes(vec![7, 8]))],
            },
        ],
    };
    assert!(
        closure_free.boundary_transfer_admissibility().is_ok(),
        "a constructor whose graph holds no closure must remain admissible; \
         D5 rejects closure-bearing GRAPHS, not the Constructor variant"
    );

    // Non-vacuity: the same shape with one leaf swapped for a closure must be
    // refused, so the `is_ok` above is attributable to the absence of a closure
    // rather than to the walk admitting everything it is handed.
    let origin = c1_closure_fixture_origin();
    let closure_bearing = Lowered::Constructor {
        constructor: "ctor:fixture::Pair::MkPair".to_string(),
        synthesized_identity: None,
        args: vec![
            Lowered::String("left".to_string()),
            Lowered::Record {
                fields: vec![("field:right".to_string(), c1_closure(origin))],
            },
        ],
    };
    assert!(
        closure_bearing.boundary_transfer_admissibility().is_err(),
        "NON-VACUITY: the walk admits a graph differing only by a closure in one \
         leaf position, so it is not discriminating on closures at all"
    );
}

// ─── RT-FNSPLIT-B2V AC-3 — exactly one of the FIVE static encoding policies ───

/// **`AC-3` — every `Lowered` variant carries exactly one of `D4`'s five static
/// encoding policies, and a declared spill is the SPILL policy.**
///
/// ⛔ The prior control proved wildcard-freedom and nothing else. Exhaustiveness
/// says every variant has *a* disposition; it says nothing about *which*, and
/// the frame names the misassignment it cares about: a variant with a declared
/// spill arm assigned *immediate-only* would let a proof attach handle evidence
/// to one sampled spill while never establishing the handle obligations for the
/// whole partition. That is the vacuity route `AC-10` exists to close, and no
/// amount of "no `_` arm" detects it.
///
/// ⚠ MEASURED: the policy of every one of the 21 variant **tags**. CLAIMED:
/// each variant has exactly one of five policies. THE GAP: that a policy is a
/// claim about the *variant* and not about a sampled value — closed structurally
/// rather than asserted, because `boundary_disposition` now takes
/// `LoweredVariant` and has no value to sample.
#[test]
fn b2v_ac3_every_variant_carries_exactly_one_of_the_five_static_policies() {
    use std::collections::{BTreeMap, BTreeSet};

    // ⛔ The sweep is over the tag set, so it is TOTAL by construction — there
    // are no 21 values to build and therefore no sampling to get wrong.
    let assigned: BTreeMap<LoweredVariant, StaticEncodingPolicy> = LoweredVariant::ALL
        .iter()
        .map(|variant| (*variant, variant.boundary_disposition().policy()))
        .collect();
    assert_eq!(
        assigned.len(),
        LoweredVariant::ALL.len(),
        "AC-3: a variant is listed twice, so the sweep is not over the tag set"
    );
    assert_eq!(
        assigned.len(),
        21,
        "AC-3: the landed variant count has moved"
    );

    // Every assigned policy is one of the five, and the five are the closed set.
    let five: BTreeSet<StaticEncodingPolicy> = StaticEncodingPolicy::ALL.iter().copied().collect();
    assert_eq!(
        five.len(),
        5,
        "AC-3: the policy set is not five distinct policies"
    );
    for (variant, policy) in &assigned {
        assert!(
            five.contains(policy),
            "AC-3: {variant:?} carries a policy outside the closed set"
        );
    }

    // ⛔ **THE misassignment the frame names.** A disposition that declares a
    // spill must be the third policy, never the first.
    for variant in LoweredVariant::ALL {
        let disposition = variant.boundary_disposition();
        if let BoundaryDisposition::RepresentedImmediate { spill, .. } = disposition {
            let expected = if spill.is_some() {
                StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill
            } else {
                StaticEncodingPolicy::ImmediateOnly
            };
            assert_eq!(
                disposition.policy(),
                expected,
                "AC-3: {variant:?} declares spill {spill:?} and must carry the \
                 matching policy — assigning immediate-only to a variant with a \
                 spill arm is the vacuity route AC-10 exists to close"
            );
        }
    }
    // ⚠ NON-DEGENERATE PAIR on that exact boundary: `Int` declares a spill and
    // `Bool` does not, and they must land in DIFFERENT policies. A checker that
    // ignored `spill` would put both in one and pass the loop above.
    assert_eq!(
        assigned[&LoweredVariant::Int],
        StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill,
        "AC-3: Int declares a PersistentGround/Int spill, so it is the third policy"
    );
    assert_eq!(
        assigned[&LoweredVariant::Bool],
        StaticEncodingPolicy::ImmediateOnly,
        "AC-3: Bool has no spill arm, so it is the first policy"
    );
    assert_ne!(
        assigned[&LoweredVariant::Int],
        assigned[&LoweredVariant::Bool],
        "AC-3: the spill boundary must separate them, or neither assertion means \
         anything"
    );

    // `Constructor` and `HostResult` are REQUIRED LIVE arms — represented, in
    // policy terms, not merely absent from the forbidden block.
    for required in [LoweredVariant::Constructor, LoweredVariant::HostResult] {
        assert_eq!(
            assigned[&required],
            StaticEncodingPolicy::HandleOnly,
            "AC-3: {required:?} is a required LIVE represented arm"
        );
    }

    // ⚠ POSITIVE CONTROL over the policy set: every policy the frame declares
    // must actually be inhabited. A policy nobody uses is unreachable surface
    // that reads as supported — the same defect that removed `ImmediateCapability`
    // from the tag set — and a policy holding all 21 would make every check
    // above vacuous.
    let mut population: BTreeMap<StaticEncodingPolicy, usize> = BTreeMap::new();
    for policy in assigned.values() {
        *population.entry(*policy).or_default() += 1;
    }
    for policy in StaticEncodingPolicy::ALL {
        let count = population.get(&policy).copied().unwrap_or(0);
        assert!(
            count > 0,
            "AC-3: no variant carries {policy:?}, so it is unreachable surface"
        );
        assert!(
            count < LoweredVariant::ALL.len(),
            "AC-3: {policy:?} holds every variant, so the assignment is degenerate"
        );
    }

    // ⛔ Every fail-closed arm names an EXACT reason, never a bare rejection.
    for variant in LoweredVariant::ALL {
        match variant.boundary_disposition() {
            BoundaryDisposition::FailClosedForbidden { why }
            | BoundaryDisposition::ProtocolOnly { why } => assert!(
                !why.is_empty(),
                "AC-3: {variant:?} rejects without an exact reason"
            ),
            BoundaryDisposition::RepresentedImmediate { .. }
            | BoundaryDisposition::RepresentedHandle { .. } => {}
        }
    }
}

// ─── RT-FNSPLIT-B2V AC-10 — total classified-domain closure ──────────────────

/// **`AC-10` — every boundary input receives exactly one actual outcome, and
/// that outcome is entailed by its variant's static policy.**
///
/// ⛔ **This is a STRUCTURAL totality proof, and it is not one dynamic test
/// pretending to enumerate an infinite domain.** The admitted domains include
/// unbounded integers, arbitrary byte contents, ownership states and recursive
/// parent → child reachability; no finite runtime sweep covers them, and one
/// wearing a universal name would be worse than an honest sweep. The closure has
/// two layers:
///
/// 1. the sealed wildcard-free disposition closes the **variant** layer
///    (`b2v_ac3_…`), and
/// 2. every **value-dependent discriminator** is a closed finite partition —
///    magnitude/shape, lifetime/owner, parent → child reachability, and the
///    producer that minted the referent — reached from a value by a **total**
///    projection (`int_fits_immediate`, `referent_owner`, "does this aggregate
///    hold an invocation-owned child").
///
/// ⭐ **So the infinite domain is covered by construction and only the finitely
/// many CELLS need controls.** This sweeps the whole product.
///
/// ⚠ MEASURED: every cell maps to exactly one outcome, permitted by its policy.
/// CLAIMED: no input or encoding outcome is unclassified. THE GAP: that a value
/// reaches its cell — which is the totality of the projections named above, and
/// is why they are named rather than implied.
#[test]
fn b2v_ac10_every_boundary_input_receives_one_policy_entailed_outcome() {
    use std::collections::BTreeSet;

    let cells = BoundaryInput::all();
    // The product is closed and finite: 21 variants x 2 magnitudes x 3
    // reachabilities x 2 producers.
    assert_eq!(
        cells.len(),
        21 * 2 * 3 * 2,
        "AC-10: the cell product has moved"
    );
    assert_eq!(
        cells.iter().collect::<BTreeSet<_>>().len(),
        cells.len(),
        "AC-10: a cell is enumerated twice, so the sweep is not over the product"
    );

    let mut outcomes = BTreeSet::new();
    for cell in &cells {
        let policy = cell.variant.boundary_disposition().policy();
        let outcome = cell.outcome();
        // ⛔ **Entailment, not merely classification.** An outcome the policy
        // does not permit is the misassignment AC-3 names, seen from the value
        // level.
        assert!(
            outcome.permitted_by(policy),
            "AC-10: {cell:?} receives {outcome:?}, which {policy:?} does not permit"
        );
        // ⛔ Every handle outcome discharges class, referent owner, identity and
        // lifetime — including the SPILL ARM of an immediate policy, which is
        // the arm a proof may not attach to one sampled value.
        if let BoundaryOutcome::HandleWord { tag, owner, .. } = outcome {
            assert_eq!(
                tag.referent_owner(),
                owner,
                "AC-10: {cell:?} declares an owner its tag does not carry — the \
                 lifetime obligation is the owner"
            );
            assert_ne!(
                owner,
                BoundaryReferentOwner::NoReferent,
                "AC-10: a handle whose referent nothing owns has no lifetime"
            );
        }
        outcomes.insert(outcome);
    }

    // ⚠ POSITIVE CONTROL over the outcome set: all four actual outcomes must be
    // inhabited. A classifier that answered `FailClosedForbidden` everywhere
    // satisfies "exactly one outcome" and every entailment above — that is the
    // vacuity the frame's own AC-10 wording was rewritten to exclude.
    let kinds: BTreeSet<&str> = outcomes
        .iter()
        .map(|outcome| match outcome {
            BoundaryOutcome::ImmediateWord { .. } => "immediate",
            BoundaryOutcome::HandleWord { .. } => "handle",
            BoundaryOutcome::ProtocolOnly => "protocol-only",
            BoundaryOutcome::FailClosedForbidden => "fail-closed",
        })
        .collect();
    assert_eq!(
        kinds,
        ["fail-closed", "handle", "immediate", "protocol-only"]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        "AC-10: an actual outcome is uninhabited, so the classification is degenerate"
    );

    // ⛔ **A policy's outcome varies ONLY in the discriminators it declares.**
    // An immediate-only policy whose outcome moved with magnitude would be a
    // spill arm nobody declared; a handle policy indifferent to reachability
    // would be admitting the parent → child escape.
    for variant in LoweredVariant::ALL {
        let policy = variant.boundary_disposition().policy();
        let at = |magnitude, reachability| {
            BoundaryInput {
                variant,
                magnitude,
                reachability,
                adoption: AdoptionPartition::StoreAdopted,
            }
            .outcome()
        };
        let within = at(
            MagnitudePartition::WithinImmediateField,
            ReachabilityPartition::Leaf,
        );
        let beyond = at(
            MagnitudePartition::BeyondImmediateField,
            ReachabilityPartition::Leaf,
        );
        match policy {
            StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill => assert_ne!(
                within, beyond,
                "AC-10: {variant:?} declares a spill, so magnitude MUST change \
                 its outcome — a constant one is a spill arm that never fires"
            ),
            StaticEncodingPolicy::ImmediateOnly
            | StaticEncodingPolicy::HandleOnly
            | StaticEncodingPolicy::ProtocolOnly
            | StaticEncodingPolicy::FailClosedForbidden => assert_eq!(
                within, beyond,
                "AC-10: {variant:?} declares no spill, so magnitude must NOT \
                 change its outcome"
            ),
        }
    }

    // ⛔ Parent → child reachability is a real discriminator: at least one
    // persistent aggregate must reject the child that dies first, and the same
    // variant must be admitted when its children outlive it. A nondegenerate
    // pair on one variant, so "rejects everything" cannot pass.
    let escaping = BoundaryInput {
        variant: LoweredVariant::Constructor,
        magnitude: MagnitudePartition::WithinImmediateField,
        reachability: ReachabilityPartition::ChildDiesBeforeParent,
        adoption: AdoptionPartition::StoreAdopted,
    };
    let sound = BoundaryInput {
        reachability: ReachabilityPartition::ChildrenOutliveParent,
        ..escaping
    };
    assert_eq!(
        escaping.outcome(),
        BoundaryOutcome::FailClosedForbidden,
        "AC-10: a persistent parent naming a child that dies first must reject"
    );
    assert!(
        matches!(sound.outcome(), BoundaryOutcome::HandleWord { .. }),
        "AC-10: the same variant with sound children must be admitted, or the \
         rejection above is about the variant and not about reachability"
    );

    // ⛔ Identity is CLASSIFIED, not assumed. A store-materialized persistent
    // handle carries the store's identity; an emitted-constructed one carries
    // none, by AC-6's design — and recording which is what makes identity
    // recoverable rather than unasked.
    let adopted = BoundaryInput {
        variant: LoweredVariant::Constructor,
        magnitude: MagnitudePartition::WithinImmediateField,
        reachability: ReachabilityPartition::Leaf,
        adoption: AdoptionPartition::StoreAdopted,
    };
    let pending = BoundaryInput {
        adoption: AdoptionPartition::PendingStoreAdoption,
        ..adopted
    };
    assert!(
        matches!(
            adopted.outcome(),
            BoundaryOutcome::HandleWord {
                identity: HandleIdentity::StoreMinted,
                ..
            }
        ),
        "AC-10: an adopted persistent handle carries the store's identity"
    );
    // ⛔ **A pending node is not a published handle at all.** Classifying it as
    // one with `NoStoreIdentity` was the defect: a consumer recovering the
    // ABSENCE of an identity has not recovered the same identity intact, and a
    // null `NODE_SLOT` denotes invocation ownership in this very layout.
    assert_eq!(
        pending.outcome(),
        BoundaryOutcome::FailClosedForbidden,
        "AC-10: a persistent node the store has not adopted must not publish"
    );

    // ⛔ **owner ⟺ identity, over the WHOLE product**: every published handle
    // declaring `PersistentStore` carries a real store identity, and no other
    // owner does.
    for cell in &cells {
        if let BoundaryOutcome::HandleWord {
            owner, identity, ..
        } = cell.outcome()
        {
            assert_eq!(
                owner == BoundaryReferentOwner::PersistentStore,
                identity == HandleIdentity::StoreMinted,
                "AC-10: {cell:?} publishes owner {owner:?} with identity \
                 {identity:?} — a persistent handle has a store identity and \
                 nothing else does"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// `RECUT 2` — representation authority-to-execution closure
// ---------------------------------------------------------------------------

/// **`RECUT 2`.** Every admitted row closes every phase its outcome requires.
///
/// ⛔ **This is the artifact RECUT 2 demands, and its row set is DERIVED.**
/// Nothing here enumerates rows: the cells come from [`BoundaryInput::all`], the
/// outcome from the wildcard-free classifier, the required phases from the
/// outcome's own class, and the bindings from a struct with six mandatory
/// fields. A hand-maintained matrix can drift from the production enums; this
/// cannot, because there is no matrix to maintain.
///
/// ⚠ MEASURED: for all 252 cells, every phase the outcome's class requires is
/// bound to a named production anchor, and every phase it does not require is
/// `StructurallyAbsent`. CLAIMED: every admitted partition has one total
/// executable lifecycle. THE GAP: that each anchor **is** the production item it
/// names — closed for five anchors by `derived_witness` below, and for the other
/// three by the named controls in `ProductionAnchor::CONTROL_CLOSED`.
#[test]
fn recut2_every_admitted_row_closes_every_required_phase() {
    use std::collections::BTreeSet;

    let cells = BoundaryInput::all();
    // Positive control FIRST: an empty sweep satisfies every `for` below.
    assert_eq!(
        cells.len(),
        21 * 2 * 3 * 2,
        "RECUT 2: the cell product moved, so this sweep is not over the partition"
    );

    let mut bound_anchors = BTreeSet::new();
    let mut absent_seen = false;
    for cell in &cells {
        let outcome = cell.outcome();
        let closure = outcome.phase_closure();
        for phase in LifecyclePhase::ALL {
            match (outcome.requires(phase), closure.binding(phase)) {
                // Required and bound — the row closes this phase.
                (true, PhaseBinding::Closed(anchor)) => {
                    bound_anchors.insert(anchor);
                }
                // Not required and absent — a contract, derived from the class.
                (false, PhaseBinding::StructurallyAbsent) => absent_seen = true,
                // ⛔ The two failures RECUT 2 exists to make loud.
                (true, PhaseBinding::StructurallyAbsent) => panic!(
                    "RECUT 2: {cell:?} receives {outcome:?}, whose class REQUIRES \
                     {phase:?}, but the row declares it structurally absent — an \
                     authority with no closed execution is the named predicate's \
                     failure"
                ),
                (false, PhaseBinding::Closed(anchor)) => panic!(
                    "RECUT 2: {cell:?} receives {outcome:?}, whose class does not \
                     require {phase:?}, yet the row binds {anchor:?} — a phase \
                     nothing entails is a claim with no authority behind it"
                ),
            }
        }
    }

    // ⚠ TWO-SIDED. Without these, a `requires` that answered `false` everywhere
    // would pass the loop above with every row structurally absent, and a
    // `requires` that answered `true` everywhere would pass a row that bound one
    // anchor to all six phases. Both must be inhabited.
    assert!(
        absent_seen,
        "RECUT 2: no phase is ever structurally absent, so `requires` is constant \
         and the derivation is degenerate"
    );
    assert!(
        bound_anchors.len() >= 5,
        "RECUT 2: only {} distinct anchors are reachable across the whole \
         partition, so most of the lifecycle is closed by one item standing in \
         for the rest",
        bound_anchors.len()
    );
}

/// **`RECUT 2`.** The phase inventory is bound to the type, not restated beside it.
///
/// ⛔ **This is the `AC-V1b` defect, refused.** That pin froze `25` next to an
/// enum and was invariant under adding a variant by construction. Here the
/// index is produced by a wildcard-free match on the enum, so a seventh phase
/// is a compile error there, and this control proves `ALL` did not silently
/// drop one.
#[test]
fn recut2_the_phase_inventory_is_bound_to_the_type() {
    for (position, phase) in LifecyclePhase::ALL.into_iter().enumerate() {
        assert_eq!(
            phase.index(),
            position,
            "RECUT 2: {phase:?} sits at {position} in ALL but indexes {}",
            phase.index()
        );
    }
    let distinct: std::collections::BTreeSet<_> = LifecyclePhase::ALL.into_iter().collect();
    assert_eq!(
        distinct.len(),
        LifecyclePhase::ALL.len(),
        "RECUT 2: ALL repeats a phase, so a missing one is masked by a duplicate"
    );
}

/// **`RECUT 2`.** No anchor is silently unclosed.
///
/// ⛔ **"Cannot determine" is a third outcome that must be ACCOUNTED FOR**, not
/// one that falls through to pass. An anchor with no derived witness must name
/// the causal control that closes its identity instead; an anchor that does
/// neither is the residual with no cell.
#[test]
fn recut2_every_anchor_is_closed_by_a_witness_or_a_named_control() {
    use std::collections::BTreeSet;

    let mut anchors = BTreeSet::new();
    for cell in BoundaryInput::all() {
        let closure = cell.outcome().phase_closure();
        for phase in LifecyclePhase::ALL {
            if let PhaseBinding::Closed(anchor) = closure.binding(phase) {
                anchors.insert(anchor);
            }
        }
    }
    assert!(
        !anchors.is_empty(),
        "RECUT 2: no anchor is bound anywhere, so every check below is vacuous"
    );

    let control_closed: BTreeSet<_> = ProductionAnchor::CONTROL_CLOSED
        .iter()
        .map(|(anchor, _)| *anchor)
        .collect();
    for anchor in &anchors {
        match anchor.derived_witness() {
            Some(_) => assert!(
                !control_closed.contains(anchor),
                "RECUT 2: {anchor:?} has BOTH a derived witness and a control \
                 row — one of the two is not describing this anchor"
            ),
            None => assert!(
                control_closed.contains(anchor),
                "RECUT 2: {anchor:?} has no derived witness and names no causal \
                 control, so nothing closes its identity — this is exactly the \
                 residual that reads as enforcement"
            ),
        }
    }

    // ⛔ Every declared control row must correspond to an anchor the partition
    // actually reaches. A control for a dead anchor is a row that can never fail.
    for (anchor, control) in ProductionAnchor::CONTROL_CLOSED {
        assert!(
            anchors.contains(anchor),
            "RECUT 2: {anchor:?} names control `{control}` but is bound by no row \
             in the partition, so that control guards nothing"
        );
    }
}

/// **`RECUT 2`.** The derived witnesses are computed by production, not restated.
///
/// ⛔ **A witness that agrees with a constant written beside it proves nothing**
/// — that is two hand-maintained authorities agreeing, the `AC-1` defect. Each
/// value below is checked against the *behaviour* of the authority that
/// produces it, so rewiring the authority moves the witness.
#[test]
fn recut2_derived_witnesses_come_from_the_production_authority() {
    // The layout witness is the node extent DERIVED from the field inventory.
    // ⛔ Asserted as a relation to the inventory, never as a frozen byte count:
    // a reviewed layout delta is predicate delta, not a regression.
    let extent = ProductionAnchor::LayoutFieldInventory
        .derived_witness()
        .expect("the layout authority is evaluable without a JIT");
    assert!(
        extent > 0,
        "RECUT 2: the derived node extent is {extent}, so the field inventory \
         computes nothing and the layout authority has no content"
    );

    // ⛔ **The expected value is COMPUTED BY the authority here, not written as
    // a literal.** Mutation `M-E` deleted the production call inside
    // `derived_witness` and replaced it with `Some(1)`; against a frozen `1`
    // that evasion stayed green, because a hardcoded constant and a live call
    // are indistinguishable while they happen to agree. Computing the expected
    // side from the authority does not make them distinguishable *today* — see
    // the residual below — but it does mean the two diverge the moment the
    // contract moves, which is the drift a frozen literal cannot see.
    let normalization_rejects_leading_zero =
        !crate::boundary_value::boundary_int_magnitude_is_canonical(0, &[1, 0]);
    assert_eq!(
        ProductionAnchor::IntNormalizationAuthority.derived_witness(),
        Some(i64::from(normalization_rejects_leading_zero)),
        "RECUT 2: the normalization witness no longer tracks the canonical \
         sign/limb authority, so it is measuring its own spelling"
    );
    // ⚠ The two-sided half: the same authority must ACCEPT a canonical
    // magnitude, or "rejects a leading zero" is just "rejects everything" and
    // the witness above is `1` for a reason that has nothing to do with the
    // contract.
    assert!(
        crate::boundary_value::boundary_int_magnitude_is_canonical(0, &[1]),
        "RECUT 2: the normalization authority rejects a canonical magnitude too, \
         so its rejection above is not discriminating"
    );
    assert!(
        normalization_rejects_leading_zero,
        "RECUT 2: the authority now ACCEPTS a leading-zero magnitude — the \
         canonical sign/limb contract has changed and this anchor's meaning \
         changed with it"
    );

    // The status witnesses are the exact codes the production paths return.
    assert_eq!(
        ProductionAnchor::EmittedEscapeGate.derived_witness(),
        Some(crate::boundary_value::BOUNDARY_ERR_ESCAPE)
    );
    assert_eq!(
        ProductionAnchor::ReachableGraphValidator.derived_witness(),
        Some(crate::boundary_value::BOUNDARY_ERR_CYCLE)
    );
    assert_eq!(
        ProductionAnchor::RegionPublication.derived_witness(),
        Some(crate::boundary_value::BOUNDARY_ERR_SEALED)
    );
    // ⛔ And the statuses must be DISTINCT: three phases sharing one code would
    // make a control that fires for one indistinguishable from the others.
    let statuses = [
        crate::boundary_value::BOUNDARY_ERR_ESCAPE,
        crate::boundary_value::BOUNDARY_ERR_CYCLE,
        crate::boundary_value::BOUNDARY_ERR_SEALED,
    ];
    let distinct: std::collections::BTreeSet<_> = statuses.iter().collect();
    assert_eq!(
        distinct.len(),
        statuses.len(),
        "RECUT 2: two lifecycle phases report the same exact status, so a \
         control cannot tell which one fired"
    );
}

/// **`RECUT 2`.** The store-minted handle is the only outcome requiring all six.
///
/// ⛔ **This is the row the six blocks kept failing**, so it is pinned as a
/// relation to the phase inventory rather than as the number six — the count is
/// derived from `LifecyclePhase::ALL`, so adding a phase strengthens this
/// automatically instead of leaving a stale literal behind.
#[test]
fn recut2_only_the_store_minted_handle_requires_the_whole_lifecycle() {
    let full: Vec<BoundaryOutcome> = BoundaryInput::all()
        .into_iter()
        .map(|cell| cell.outcome())
        .filter(|outcome| {
            LifecyclePhase::ALL
                .into_iter()
                .all(|phase| outcome.requires(phase))
        })
        .collect();
    assert!(
        !full.is_empty(),
        "RECUT 2: no outcome requires the whole lifecycle, so the artifact never \
         asks the question the six blocks failed"
    );
    for outcome in full {
        assert!(
            matches!(
                outcome,
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::StoreMinted,
                    ..
                }
            ),
            "RECUT 2: {outcome:?} requires every phase, but only a store-minted \
             handle should — a non-persistent outcome demanding adoption means \
             the requirement is not derived from the class"
        );
    }
}

/// **`RECUT 2`, causal.** The emitter CONSUMES the representation authority.
///
/// ⛔ **The ruling's bar, literally:** *"mutate or bypass the authority and the
/// captured/emitted helper graph must change or reject; an emitter that ignores
/// the plan must redden."* A test that only checks the plan is *passed* would be
/// the `let _ = plan` the ruling excludes, so this feeds a **perturbed plan**
/// and requires the emitted CLIF to differ.
///
/// ⚠ MEASURED: the emitted helper graph under the derived plan differs from the
/// graph under a plan whose class sets differ. CLAIMED: the helper bodies are
/// generated from the authority. THE GAP: that the derived plan is the
/// authority's real answer — closed by
/// `recut2_the_plan_is_derived_from_the_partition_not_restated` below.
#[test]
fn recut2_the_emitted_helper_graph_changes_when_the_authority_changes() {
    use crate::boundary_value::{BoundaryClass, BoundaryEmissionPlan};

    let derived = BoundaryEmissionPlan::derive();
    let real = crate::boundary_value_clif::tests::capture_with_plan(&derived);

    // Positive control FIRST: the capture must be non-empty, or every
    // comparison below is between two empty strings and means nothing.
    assert!(
        real.contains("function"),
        "RECUT 2: the capture is empty, so the difference below is not evidence"
    );

    // ⛔ Perturb ONLY the axis the plan names: the class set a limb helper may
    // touch. Nothing else about the emitter or the module changes.
    let perturbed = BoundaryEmissionPlan::new(
        vec![BoundaryClass::Record],
        derived.byte_span_classes().to_vec(),
        derived.tags().clone(),
    );
    let other = crate::boundary_value_clif::tests::capture_with_plan(&perturbed);
    assert_ne!(
        real, other,
        "RECUT 2: the emitted helper graph is IDENTICAL under a plan whose \
         int-magnitude class set is different — the emitter is not consuming \
         the authority, it is only receiving it"
    );

    // ⛔ And the difference must be the CLASS COMPARISON, not incidental. The
    // real graph compares against `Int`; the perturbed one against `Record`.
    assert!(
        real.contains(&format!("{}", BoundaryClass::Int as i64))
            && other.contains(&format!("{}", BoundaryClass::Record as i64)),
        "RECUT 2: the graphs differ, but not in the class constant the plan \
         supplies — the difference is not attributable to the authority"
    );

    // ⚠ Two-sided: the SAME plan must produce the SAME graph, or `assert_ne!`
    // above would pass for any two captures and prove nothing about the plan.
    let again = crate::boundary_value_clif::tests::capture_with_plan(&derived);
    assert_eq!(
        real, again,
        "RECUT 2: two captures under the same plan differ, so emission is not \
         a function of the plan and the inequality above is noise"
    );
}

/// **`RECUT 2`.** The plan is derived from the partition, not restated beside it.
///
/// ⛔ This is the half that keeps the causal test above honest: it would still
/// pass if `derive()` returned a hand-written set. Here the expected sets are
/// recomputed from the authority *in the test*, by the same two total
/// projections — the classifier and `storage_shape` — so a `derive()` that
/// stopped consulting either reddens.
#[test]
fn recut2_the_plan_is_derived_from_the_partition_not_restated() {
    use crate::boundary_value::{BoundaryClass, BoundaryEmissionPlan, BoundaryStorageShape};
    use std::collections::BTreeSet;

    let mut admitted: BTreeSet<BoundaryClass> = BTreeSet::new();
    for cell in BoundaryInput::all() {
        if let BoundaryOutcome::HandleWord { class, .. } = cell.outcome() {
            admitted.insert(class);
        }
    }
    assert!(
        !admitted.is_empty(),
        "RECUT 2: the partition admits no handle class at all, so the plan is \
         vacuous and every set below is trivially equal"
    );

    let plan = BoundaryEmissionPlan::derive();
    // ⛔ There is no whole-admitted-class assertion here, because the plan no
    // longer carries that set: no emitted helper ever read it. The per-shape
    // sets below are recomputed from `admitted` in this test, so the classifier
    // is still the thing being pinned — dropping an unconsumed accessor removed
    // a declaration, not a control.
    for (shape, got) in [
        (
            BoundaryStorageShape::IntMagnitude,
            plan.int_magnitude_classes(),
        ),
        (BoundaryStorageShape::ByteSpan, plan.byte_span_classes()),
    ] {
        let want: Vec<BoundaryClass> = admitted
            .iter()
            .copied()
            .filter(|class| class.storage_shape() == shape)
            .collect();
        assert_eq!(
            got, want,
            "RECUT 2: the plan's {shape:?} set is not the admitted classes of \
             that storage shape"
        );
        assert!(
            !want.is_empty(),
            "RECUT 2: no admitted class has storage shape {shape:?}, so the \
             guard built from it would name nothing"
        );
    }
}

/// **`RECUT 2`, causal — the TAG axis.** The emitted helpers branch on the
/// plan's derived tag sets, not on an ordinal band.
///
/// ⛔ **The ruling's bar is causation, not agreement:** *"mutate or bypass the
/// authority and the captured/emitted helper graph must change or reject."* An
/// assertion that the plan and the emitted bytes agree is explicitly listed as
/// not counting, so this feeds a **perturbed tag admission** and requires the
/// CLIF to differ.
///
/// ⚠ MEASURED: the emitted graph under the derived tag sets differs from the
/// graph under tag sets that admit a different set of tags. CLAIMED: the
/// emitted validity, handle-ness and immediacy tests are generated from the
/// authority. THE GAP: that the derived sets are the authority's real answer —
/// closed by `recut2_the_tag_admission_is_derived_from_the_partition_not_restated`.
#[test]
fn recut2_the_emitted_helper_graph_changes_when_the_tag_sets_change() {
    use crate::boundary_value::{BoundaryEmissionPlan, BoundaryTag, BoundaryTagAdmission};

    let derived = BoundaryEmissionPlan::derive();
    let real = crate::boundary_value_clif::tests::capture_with_plan(&derived);
    assert!(
        real.contains("function"),
        "RECUT 2: the capture is empty, so the difference below is not evidence"
    );

    // ⛔ Perturb ONLY the tag axis: drop one admitted tag. Nothing about the
    // class sets, the emitter, or the module changes.
    let dropped = *derived
        .tags()
        .admitted()
        .last()
        .expect("the partition admits at least one tag");
    let thinner: Vec<BoundaryTag> = derived
        .tags()
        .admitted()
        .iter()
        .copied()
        .filter(|tag| *tag != dropped)
        .collect();
    assert!(
        thinner.len() + 1 == derived.tags().admitted().len(),
        "RECUT 2: the perturbation removed nothing, so the comparison below is \
         between two identical plans"
    );
    let perturbed = BoundaryEmissionPlan::new(
        derived.int_magnitude_classes().to_vec(),
        derived.byte_span_classes().to_vec(),
        BoundaryTagAdmission::new(
            thinner,
            derived.tags().immediate().to_vec(),
            derived.tags().handle().to_vec(),
            derived.tags().owner_bands().to_vec(),
            derived.tags().immediate_value_classes().to_vec(),
            derived.tags().handle_class_relation().to_vec(),
        ),
    );
    let other = crate::boundary_value_clif::tests::capture_with_plan(&perturbed);
    assert_ne!(
        real, other,
        "RECUT 2: the emitted helper graph is IDENTICAL under a plan admitting \
         one fewer tag — the emitter is not consuming the tag admission, it is \
         only receiving it"
    );

    // ⛔ And the difference must be the DROPPED TAG's own membership tests,
    // not something incidental. Presence/absence is the wrong needle: tag 8 is
    // also in the handle set and in an owner band, so it keeps being compared
    // for those. What must move is the COUNT — dropping it from exactly one
    // derived set removes exactly the comparisons that set generates, and
    // nothing in the perturbation can add one.
    let compares_for = |clif: &str| {
        let suffix = format!(", {}", dropped as i64);
        clif.lines()
            .filter(|line| line.contains("icmp_imm") && line.trim_end().ends_with(&suffix))
            .count()
    };
    let (before, after) = (compares_for(&real), compares_for(&other));
    assert!(
        before > 0,
        "RECUT 2: the real graph never compares against {dropped:?} at all, so \
         its disappearance below would not be evidence"
    );
    assert!(
        after < before,
        "RECUT 2: the graphs differ, but the emitted membership tests for \
         {dropped:?} did not decrease ({before} -> {after}) — the difference \
         is not attributable to the tag the plan stopped admitting"
    );

    // ⚠ Two-sided: the SAME plan must produce the SAME graph, or `assert_ne!`
    // above would pass for any two captures.
    let again = crate::boundary_value_clif::tests::capture_with_plan(&derived);
    assert_eq!(
        real, again,
        "RECUT 2: two captures under the same plan differ, so emission is not \
         a function of the plan and the inequality above is noise"
    );
}

/// **`RECUT 2`, causal — the OWNER axis.** The emitted region selection, the
/// node's recorded owner, and the escape gate all branch on the plan's owner
/// bands.
///
/// ⚠ MEASURED: moving a tag from one owner band to another changes the emitted
/// CLIF. CLAIMED: the emitted owner decisions are generated from the authority
/// rather than from a threshold on tag order. THE GAP: that the bands are the
/// partition's real answer — closed by the derivation test below.
///
/// ⛔ This axis needs its own perturbation because the tag test above holds the
/// bands fixed: a plan could consume the admitted set and still decide
/// ownership from a hardcoded threshold, and every assertion there would pass.
#[test]
fn recut2_the_emitted_helper_graph_changes_when_the_owner_bands_change() {
    use crate::boundary_value::{BoundaryEmissionPlan, BoundaryTagAdmission};

    let derived = BoundaryEmissionPlan::derive();
    let real = crate::boundary_value_clif::tests::capture_with_plan(&derived);
    assert!(
        real.contains("function"),
        "RECUT 2: the capture is empty, so the difference below is not evidence"
    );

    // Move the first band's first tag into the second band — the owners keep
    // their identities, only the tag-to-owner assignment moves.
    let mut bands = derived.tags().owner_bands().to_vec();
    assert!(
        bands.len() >= 2 && !bands[0].1.is_empty(),
        "RECUT 2: fewer than two non-empty owner bands, so a reassignment \
         cannot be expressed and this test proves nothing"
    );
    let moved = bands[0].1.remove(0);
    bands[1].1.push(moved);
    bands[1].1.sort();
    let perturbed = BoundaryEmissionPlan::new(
        derived.int_magnitude_classes().to_vec(),
        derived.byte_span_classes().to_vec(),
        BoundaryTagAdmission::new(
            derived.tags().admitted().to_vec(),
            derived.tags().immediate().to_vec(),
            derived.tags().handle().to_vec(),
            bands,
            derived.tags().immediate_value_classes().to_vec(),
            derived.tags().handle_class_relation().to_vec(),
        ),
    );
    let other = crate::boundary_value_clif::tests::capture_with_plan(&perturbed);
    assert_ne!(
        real, other,
        "RECUT 2: the emitted helper graph is IDENTICAL after moving {moved:?} \
         to another owner band — the owner decisions are not derived from the \
         bands"
    );

    let again = crate::boundary_value_clif::tests::capture_with_plan(&derived);
    assert_eq!(
        real, again,
        "RECUT 2: two captures under the same plan differ, so the inequality \
         above is noise"
    );
}

/// **`RECUT 2`.** The tag admission is derived from the partition, not restated
/// beside it.
///
/// ⛔ The half that keeps the two causal tests honest: they would both still
/// pass if `derive()` returned hand-written sets. Here the expected sets are
/// recomputed from the authority *in the test*, by the same total projection —
/// sweeping `BoundaryInput::all()` through the wildcard-free classifier — so a
/// `derive()` that stopped consulting it reddens.
#[test]
fn recut2_the_tag_admission_is_derived_from_the_partition_not_restated() {
    use crate::boundary_value::{BoundaryEmissionPlan, BoundaryReferentOwner, BoundaryTag};
    use std::collections::{BTreeMap, BTreeSet};

    let mut immediate: BTreeSet<BoundaryTag> = BTreeSet::new();
    let mut handle: BTreeSet<BoundaryTag> = BTreeSet::new();
    let mut bands: BTreeMap<BoundaryReferentOwner, BTreeSet<BoundaryTag>> = BTreeMap::new();
    let mut value_classes: BTreeMap<BoundaryTag, crate::boundary_value::BoundaryClass> =
        BTreeMap::new();
    for cell in BoundaryInput::all() {
        match cell.outcome() {
            BoundaryOutcome::ImmediateWord { tag, value_class } => {
                immediate.insert(tag);
                if let Some(class) = value_class {
                    value_classes.insert(tag, class);
                }
            }
            BoundaryOutcome::HandleWord { tag, owner, .. } => {
                handle.insert(tag);
                bands.entry(owner).or_default().insert(tag);
            }
            BoundaryOutcome::ProtocolOnly | BoundaryOutcome::FailClosedForbidden => {}
        }
    }
    // Positive controls: each population is non-empty, so the equalities below
    // are not agreements between empty sets.
    assert!(
        !immediate.is_empty() && !handle.is_empty() && bands.len() >= 2,
        "RECUT 2: the partition yields immediate={}, handle={}, bands={} — a \
         plan derived from it would be vacuous",
        immediate.len(),
        handle.len(),
        bands.len()
    );

    let plan = BoundaryEmissionPlan::derive();
    assert_eq!(
        plan.tags().immediate(),
        immediate.iter().copied().collect::<Vec<_>>(),
        "RECUT 2: the plan's immediate tag set is not the partition's"
    );
    assert_eq!(
        plan.tags().handle(),
        handle.iter().copied().collect::<Vec<_>>(),
        "RECUT 2: the plan's handle tag set is not the partition's"
    );
    assert_eq!(
        plan.tags().admitted(),
        immediate
            .union(&handle)
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
        "RECUT 2: the plan's admitted tag set is not the union of the two"
    );
    assert_eq!(
        plan.tags().owner_bands(),
        bands
            .into_iter()
            .map(|(owner, tags)| (owner, tags.into_iter().collect::<Vec<_>>()))
            .collect::<Vec<_>>(),
        "RECUT 2: the plan's owner bands are not the partition's"
    );
    // ⛔ The immediate-class projection, swept from the same outcomes. Kept
    // separate from the node-class relation on purpose: this is what the
    // `class` helper reports for an immediate word, not a node's `NODE_CLASS`.
    assert!(
        !value_classes.is_empty(),
        "RECUT 2: the partition classifies no immediate, so the equality below \
         is between empty relations"
    );
    assert_eq!(
        plan.tags().immediate_value_classes(),
        value_classes.into_iter().collect::<Vec<_>>(),
        "RECUT 2: the plan's immediate value-class relation is not the \
         partition's"
    );
}

/// **`RECUT 2`, identity.** The predicted-and-then-measured half: emitted code
/// makes no identity decision the authority does not supply.
///
/// ⚠ **The prediction, stated before it was measured** (recorded in the
/// evidence doc at `ab11a3d2`): `HandleIdentity` is computed by
/// `BoundaryInput::handle_identity` **from the owner alone**, so once the owner
/// bands are derived, identity needs no separate wiring.
///
/// ⚠ MEASURED: identity is a total function of owner across every admitted
/// handle outcome, and the sole identity the emitted graph can mint is the
/// absent one — every `alloc`ed node is written `NULL_SLOT`, which this ABI
/// reads as "no store identity". CLAIMED: no emitted decision assigns identity.
/// ⛔ **THE GAP, stated rather than closed:** this shows emitted code cannot
/// mint a *store* identity, not that no future helper could. The residual is
/// review-enforced — `escape_check`'s adoption gate is the mechanism that keeps
/// it honest at runtime, and it is tested separately.
#[test]
fn recut2_identity_is_a_function_of_owner_and_needs_no_second_wiring() {
    use std::collections::BTreeMap;

    let mut by_owner: BTreeMap<BoundaryReferentOwner, BTreeSet<HandleIdentity>> = BTreeMap::new();
    for cell in BoundaryInput::all() {
        if let BoundaryOutcome::HandleWord {
            owner, identity, ..
        } = cell.outcome()
        {
            by_owner.entry(owner).or_default().insert(identity);
        }
    }
    assert!(
        by_owner.len() >= 2,
        "RECUT 2: fewer than two owners publish handles, so 'identity is a \
         function of owner' cannot be distinguished from 'identity is constant'"
    );
    for (owner, identities) in &by_owner {
        assert_eq!(
            identities.len(),
            1,
            "RECUT 2: {owner:?} publishes {} distinct identities, so identity \
             is NOT a function of owner and the emitted side would need a \
             decision the bands cannot supply",
            identities.len()
        );
    }
    // ⛔ Non-vacuity: the function must actually distinguish, or a constant
    // identity would satisfy every assertion above.
    let distinct: BTreeSet<HandleIdentity> = by_owner.values().flatten().copied().collect();
    assert!(
        distinct.len() >= 2,
        "RECUT 2: every owner yields the same identity, so the agreement above \
         is vacuous"
    );
}

// ── RT-MATCH-FRAME-FP: the identity selector and its permutation net ───────
//
// `dec_s30rdnb1dvgk`. `AC-F1` makes two frames that differ only in a
// closure-bearing body share one header fingerprint, so a fingerprint can no
// longer say *which* occurrence is being checked. Identity is transported;
// the fingerprint is a compatibility check only.
//
// ⚠ The fixture below is the reachable shape, not a contrived one: `erasure.rs`
// derives the case headers from the eliminated family and builds the default as
// `format!("no runtime match case selected for {family_symbol}")`. Two
// eliminations of one family in one declaration therefore agree on every field
// a header fingerprint can see.

#[cfg(test)]
const RTFP_DECLARATION: &str = "decl:fixture::RTFP::twice";
#[cfg(test)]
const RTFP_CALL_TEMPLATE: u64 = 700;
/// `semantic_position` 1 — the outermost checked frame, visited FIRST.
#[cfg(test)]
const RTFP_OUTER_FRAME: u64 = 10;
/// `semantic_position` 0 — checked postorder's first frame, visited LAST.
#[cfg(test)]
const RTFP_INNER_FRAME: u64 = 11;

#[cfg(test)]
fn rtfp_cases(body: i64) -> Vec<crate::RuntimeComputationalMatchCase> {
    vec![crate::RuntimeComputationalMatchCase {
        constructor: "ctor:fixture::Succ".to_string(),
        argument_binders: 1,
        recursive_positions: vec![0],
        // ⭐ The ONLY field that differs between the two frames.
        body: RuntimeExpr::Value(RuntimeValue::Int(body.into())),
    }]
}

#[cfg(test)]
fn rtfp_default() -> RuntimeTrap {
    RuntimeTrap {
        code: RuntimeTrapCode::PatternMatchFailure,
        message: "no runtime match case selected for ind:fixture::Nat".to_string(),
    }
}

#[cfg(test)]
fn rtfp_frame(
    frame_id: u64,
    semantic_position: u64,
    input: u8,
    output: u8,
    parent: Option<u64>,
) -> crate::OrientedSubcontinuationFramePlanV1 {
    let mut frame = crate::OrientedSubcontinuationFramePlanV1 {
        frame_id,
        segment_site_id: 9,
        declaration: RTFP_DECLARATION.to_string(),
        checked_occurrence_path: vec![frame_id],
        semantic_position,
        input_interface: oriented_test_interface(input),
        output_interface: oriented_test_interface(output),
        // ⭐ Identical for both frames — computed from the shared header, never
        // from the body. That equality is `AC-F1`, and it is what makes the
        // fingerprint useless as a selector.
        runtime_frame_fingerprint: crate::compiler_private_computational_match_frame_fingerprint(
            &rtfp_cases(0),
            &rtfp_default(),
        ),
        occurrence_binding_fingerprint: 0,
        control_witness: parent.map_or(
            crate::OrientedControlWitnessV1::DistinguishedRoot,
            crate::OrientedControlWitnessV1::ParentFrame,
        ),
    };
    frame.occurrence_binding_fingerprint =
        crate::compiler_private_oriented_occurrence_binding_fingerprint(&frame);
    frame
}

#[cfg(test)]
fn rtfp_plan() -> crate::OrientedSubcontinuationPlanV1 {
    let mut call = crate::CheckedRecursiveInvocationTemplateV1 {
        call_template_id: RTFP_CALL_TEMPLATE,
        declaration: RTFP_DECLARATION.to_string(),
        checked_occurrence_path: vec![5],
        callee: RTFP_DECLARATION.to_string(),
        level_instantiation: Vec::new(),
        recursion_group: "scc:fixture::RTFP".to_string(),
        scc_index: 0,
        admission: 0,
        arity: 1,
        local_telescope: Vec::new(),
        result_interface: oriented_test_interface(2),
        callee_segment_site_id: 9,
        // ⚠ Ascending `semantic_position`, exactly as `erasure.rs:1149` sorts.
        callee_frame_templates: vec![RTFP_INNER_FRAME, RTFP_OUTER_FRAME],
        caller_interface: oriented_test_interface(2),
        // ⚠ `validate_marker_locations` rejects an empty occurrence list, so an
        // empty one would make every REJECTION control below green on a fixture
        // that could never have lowered in the first place. The positive
        // control is what surfaced that.
        runtime_marker_locations: vec![crate::CheckedRuntimeMarkerLocationV1 {
            declaration: RTFP_DECLARATION.to_string(),
            runtime_path: vec![0, 1],
        }],
        occurrence_binding_fingerprint: 0,
    };
    call.occurrence_binding_fingerprint =
        crate::compiler_private_recursive_call_binding_fingerprint(&call);
    crate::OrientedSubcontinuationPlanV1 {
        representation_rule_version:
            crate::OrientedSubcontinuationPlanV1::REPRESENTATION_RULE_VERSION,
        frames: vec![
            rtfp_frame(RTFP_OUTER_FRAME, 1, 1, 2, None),
            rtfp_frame(RTFP_INNER_FRAME, 0, 0, 1, Some(RTFP_OUTER_FRAME)),
        ],
        recursive_calls: vec![call],
        computational_ih_slots: Vec::new(),
        computational_ih_calls: Vec::new(),
    }
}

#[cfg(test)]
fn rtfp_layer(
    frame_id: Option<u64>,
    body: i64,
    role: RecursorLayerRole,
) -> ComputationalRecursorLayer {
    ComputationalRecursorLayer {
        cases: rtfp_cases(body),
        default: rtfp_default(),
        outer_env: Vec::new(),
        static_origin: inert_test_static_origin(),
        provenance: RecursorFrameProvenance(frame_id.unwrap_or(0)),
        role,
        checked_frame_id: frame_id,
        checked_invocation_id: None,
        checked_invocation_source: None,
        checked_invocation_depth: 0,
        semantic_pending: true,
    }
}

#[cfg(test)]
fn rtfp_invocation() -> CheckedRecursiveInvocationInstance {
    CheckedRecursiveInvocationInstance {
        source: InvocationTemplateRef::SameSccCall(RTFP_CALL_TEMPLATE),
        invocation_instance_id: 0,
        semantic_depth: 0,
        dynamic_splice_edge: None,
    }
}

/// `selection_frame` is visited first, `wrapper_frame` second.
#[cfg(test)]
fn rtfp_segment(
    selection_frame: Option<u64>,
    wrapper_frame: Option<u64>,
) -> RecursorInvocationSegment {
    let origin = RecursorProducerOriginId(70);
    RecursorInvocationSegment::new(
        origin,
        0,
        rtfp_layer(
            selection_frame,
            1,
            RecursorLayerRole::SelectsOccurrence { origin },
        ),
        RecursorUnwindStack {
            later_wrappers_in_construction_order: vec![rtfp_layer(
                wrapper_frame,
                2,
                RecursorLayerRole::ExitsScope {
                    origin,
                    scope_origin: RecursorProducerOriginId(71),
                    parent_scope: None,
                },
            )],
        },
        ContinuationCursorId(7),
        None,
        None,
    )
}

#[cfg(test)]
fn rtfp_compose(
    plan: &crate::OrientedSubcontinuationPlanV1,
    segment: RecursorInvocationSegment,
) -> Result<InstalledOrientedSubcontinuationSegment, CraneliftBackendError> {
    compose_oriented_subcontinuation(
        Some(plan),
        Some(rtfp_invocation()),
        ContinuationActivationId(8),
        segment,
        Vec::new(),
    )
}

#[cfg(test)]
fn rtfp_reason(
    result: Result<InstalledOrientedSubcontinuationSegment, CraneliftBackendError>,
) -> String {
    match result {
        Ok(_) => panic!("this fixture must reject"),
        Err(CraneliftBackendError::Unsupported(UnsupportedLowering { construct, reason })) => {
            assert_eq!(construct, "OrientedSubcontinuationPlanV1");
            reason
        }
        Err(other) => panic!("unexpected error class: {other:?}"),
    }
}

#[test]
fn rtfp_the_two_frames_are_header_identical_and_body_distinct() {
    // ⭐ Non-vacuity for every control below. If the fingerprints differed, the
    // permutation control would redden at the *fingerprint* check and prove
    // nothing about the order check; if the bodies agreed, there would be no
    // permutation to catch.
    let plan = rtfp_plan();
    let outer = plan.frame(RTFP_OUTER_FRAME).expect("outer frame");
    let inner = plan.frame(RTFP_INNER_FRAME).expect("inner frame");
    assert_eq!(
        outer.runtime_frame_fingerprint, inner.runtime_frame_fingerprint,
        "AC-F1: two same-family frames must share one header fingerprint"
    );
    assert_ne!(
        format!("{:?}", rtfp_cases(1)),
        format!("{:?}", rtfp_cases(2)),
        "the two bodies must actually differ"
    );
    assert_ne!(RTFP_OUTER_FRAME, RTFP_INNER_FRAME, "identities are distinct");
}

#[test]
fn rtfp_both_exact_occurrences_lower_under_equal_header_fingerprints() {
    let plan = rtfp_plan();
    let installed = rtfp_compose(&plan, rtfp_segment(Some(RTFP_OUTER_FRAME), Some(RTFP_INNER_FRAME)))
        .expect("two header-identical frames with distinct transported ids must both lower");
    assert_eq!(
        installed
            .semantic_frames
            .iter()
            .map(|frame| frame.checked_frame_id.unwrap())
            .collect::<Vec<_>>(),
        vec![RTFP_INNER_FRAME, RTFP_OUTER_FRAME],
        "checked composition order is postorder: inner then outer"
    );
}

#[test]
fn rtfp_a_cleared_transported_identity_rejects_before_cfg() {
    let plan = rtfp_plan();
    let reason = rtfp_reason(rtfp_compose(&plan, rtfp_segment(None, Some(RTFP_INNER_FRAME))));
    assert!(
        reason.contains("no checked frame identity"),
        "a dropped identity must be named as such, not recovered by inference: {reason}"
    );
}

#[test]
fn rtfp_exchanging_the_two_occurrence_identities_rejects() {
    // ⭐ THE PERMUTATION NET. The exchanged set is still exactly `expected`, and
    // both layers still pass the fingerprint compatibility check because the
    // two frames are header-identical by construction. ⛔ So a set-only check
    // CANNOT fail this fixture — only the occurrence-order check can.
    let plan = rtfp_plan();
    let reason = rtfp_reason(rtfp_compose(
        &plan,
        rtfp_segment(Some(RTFP_INNER_FRAME), Some(RTFP_OUTER_FRAME)),
    ));
    assert!(
        reason.contains("out of their planned occurrence order"),
        "the ORDER check must be the detector that fires, not coverage or the \
         fingerprint: {reason}"
    );
}

#[test]
fn rtfp_header_drift_after_identity_selection_rejects_by_fingerprint() {
    let plan = rtfp_plan();
    let mut segment = rtfp_segment(Some(RTFP_OUTER_FRAME), Some(RTFP_INNER_FRAME));
    // Identity is still exact and correctly ordered; only the header moved.
    segment.selection.default.message.push_str(" (drifted)");
    let reason = rtfp_reason(rtfp_compose(&plan, segment));
    assert!(
        reason.contains("does not match its checked frame template"),
        "post-selection header drift must still reject by fingerprint: {reason}"
    );
}

// ─── RT-FNSPLIT-B2F AC-2 — the emitted-unit population, measured BEHAVIOURALLY ─

/// **`AC-2`'s real property, defended by an oracle that source text cannot
/// move.**
///
/// ⭐ **Why this test exists next to a census that already "covers" `AC-2`.**
/// `correspondence_adds_no_emitted_unit_to_the_production_census` counts how
/// many times three spellings occur in seven files. That is a claim about
/// *repository text*: splitting a call across lines evades every needle, a
/// mention inside a comment inflates them, and in no configuration does it
/// observe a single emitted function. ⇒ It is a **tripwire**. This test is the
/// evidence: it counts units at the point of emission, so the number it asserts
/// is a property of the compiled module.
///
/// **MEASURED:** for two programs that differ *only* in whether they contain a
/// retained closure body, the `(declared, defined)` unit counts `B2F` actually
/// emitted.
/// **CLAIMED:** every declared target unit is defined, and the population tracks
/// the program's static structure rather than being a constant.
/// **THE GAP:** ⛔ this says nothing about whether a unit's *body* is correct,
/// nor that the population equals `entries ∪ StaticBody targets` — the latter is
/// `B2O`'s enforced equality (`validate_function_units`), consumed here rather
/// than re-asserted, because planning refuses to build a graph that violates it
/// and a re-assertion would be green on every input that can reach `B2F`.
#[test]
fn b2f_emits_one_defined_target_unit_per_planned_function_unit() {
    fn units_emitted(expr: &RuntimeExpr) -> (usize, usize) {
        let module = new_jit_module().expect("jit module");
        compile_expr_into_module(
            module,
            "b2f_unit_population_probe",
            Linkage::Local,
            expr,
            &NativeSeedEnvironment::empty(),
            BTreeMap::new(),
            None,
            false,
            None,
            None,
            None,
        )
        .expect("compile");
        crate::cranelift_backend::lowering::units::b2f_last_unit_emission()
    }

    // The two fixtures differ in exactly one thing: the second reaches the same
    // leaf value through a *called* lexical closure, which is what mints a
    // `StaticBody` edge and therefore a second function unit.
    //
    // ⚠ The closure is CALLED rather than returned, and that is required rather
    // than stylistic: a closure at the root is rejected outright
    // ("closures are callable but not observable ground values in native
    // lowering"), so a fixture that merely mentions one never reaches emission
    // and would have measured nothing while looking like a discriminator.
    let leaf = RuntimeExpr::Value(RuntimeValue::Bool(true));
    let with_closure = RuntimeExpr::Call {
        callee: Box::new(RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: Vec::new(),
            body: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
        }),
        args: Vec::new(),
    };

    let (leaf_declared, leaf_defined) = units_emitted(&leaf);
    let (closure_declared, closure_defined) = units_emitted(&with_closure);

    // ⛔ Every declared unit is defined. A bundle that declares `n` and defines
    // `n-1` leaves an undefined symbol, which is why the recorder carries two
    // numbers instead of one.
    assert_eq!(
        leaf_declared, leaf_defined,
        "AC-2 -- a declared target unit was never defined (leaf program)"
    );
    assert_eq!(
        closure_declared, closure_defined,
        "AC-2 -- a declared target unit was never defined (closure program)"
    );

    // ⭐ POSITIVE CONTROL / NON-VACUITY. Without this the assertions above are
    // satisfied by emitting nothing at all, for any program, forever -- a
    // negative check passes for any reason. The discriminator is that the count
    // MOVES with the program's static structure.
    assert!(
        leaf_declared >= 1,
        "AC-2 -- even a leaf program has a root scheduling entry, so the \
         population is never empty; measured {leaf_declared}"
    );
    assert!(
        closure_declared > leaf_declared,
        "AC-2 -- NON-VACUITY: a retained closure body mints a `StaticBody` edge \
         and therefore an additional function unit. If these are equal the \
         population is not tracking the program and every count above is \
         satisfied by a constant. measured leaf={leaf_declared} \
         closure={closure_declared}"
    );
}
