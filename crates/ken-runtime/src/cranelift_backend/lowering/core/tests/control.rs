//! Oriented control, PX8J/PX8DS recursor-consumer, root-authority and
//! source-install lowering tests (RT-SPLIT §10.2: `oriented_*`, `px8j_*`,
//! root-authority, join-site, source-install and recursor tests -> `control`).

use super::*;

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
fn root_authority_test_lowering<'a>(
    seed_env: &'a NativeSeedEnvironment,
) -> Lowering<'a> {
    Lowering {
        seed_env,
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
        native_int_mutation: NativeIntLoweringMutation::Exact,
        bounded_nat_mutation: BoundedNatLoweringMutation::Exact,
    }
}

#[cfg(test)]
fn run_px8j_malformed_recursor_consumer(
    consumer: Px8jDirectRecursorConsumer,
    malformation: Px8jRecursorMalformation,
) -> Result<Lowered, CraneliftBackendError> {
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
    let origin = RecursorProducerOriginId(7);
    let cursor = ContinuationCursorId(9);
    let layer = |role| ComputationalRecursorLayer {
        cases: Vec::new(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: "px8j malformed recursor role".to_string(),
        },
        outer_env: Vec::new(),
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
        residual: Box::new(Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            body: RuntimeExpr::Construct {
                constructor: "ctor:fixture::PX8J::Done".to_string(),
                args: Vec::new(),
            },
        }),
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
    let env = [recursor];
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
    let mut function_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut function_context);
    let entry = builder.create_block();
    builder.switch_to_block(entry);
    match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer => compiler
            .lower_computational_producer_expr(&mut builder, &pending_let, &env, &active_frames),
        Px8jDirectRecursorConsumer::ProducerCall => {
            compiler.lower_computational_producer_expr(&mut builder, &call, &env, &active_frames)
        }
        Px8jDirectRecursorConsumer::OrdinaryCall => compiler.lower_expr(&mut builder, &call, &env),
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
) -> Result<Lowered, CraneliftBackendError> {
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
        residual: Box::new(Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            body: RuntimeExpr::Construct {
                constructor: "ctor:fixture::PX8DS::Done".to_string(),
                args: Vec::new(),
            },
        }),
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
    let env = [recursor];
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
    match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer => compiler
            .lower_computational_producer_expr(&mut builder, &pending_let, &env, &active_frames),
        Px8jDirectRecursorConsumer::ProducerCall => {
            compiler.lower_computational_producer_expr(&mut builder, &call, &env, &active_frames)
        }
        Px8jDirectRecursorConsumer::OrdinaryCall => compiler.lower_expr(&mut builder, &call, &env),
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
        let error = match run_px8j_source_machine_install(Some(
            Px8jInstallMalformation::RepeatedScopeIdentity,
        )) {
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

