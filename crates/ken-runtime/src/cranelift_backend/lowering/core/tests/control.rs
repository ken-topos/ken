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
        Px8jDirectRecursorConsumer::ProducerCall
        | Px8jDirectRecursorConsumer::OrdinaryCall => &call,
    };
    let (static_transition_plan, fixture_origin) = planned_root_occurrence(lowered_fixture);
    let mut compiler = Lowering {
        seed_env: &seed_env,
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
        residual: Box::new(Lowered::Closure {
            captures: Vec::new(),
            params: Vec::new(),
            // A childless residual: `Construct` with no arguments derives no
            // child origin, so an inert planned origin is honest here.
            body: OwnedSourceOccurrence {
                expr: RuntimeExpr::Construct {
                    constructor: "ctor:fixture::PX8J::Done".to_string(),
                    args: Vec::new(),
                },
                static_origin: inert_test_static_origin(),
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
        | Px8jDirectRecursorConsumer::ProducerCall => compiler
            .lower_computational_producer_expr(&mut builder, occurrence, &env, &active_frames),
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
            // A childless residual: `Construct` with no arguments derives no
            // child origin, so an inert planned origin is honest here.
            body: OwnedSourceOccurrence {
                expr: RuntimeExpr::Construct {
                    constructor: "ctor:fixture::PX8DS::Done".to_string(),
                    args: Vec::new(),
                },
                static_origin: inert_test_static_origin(),
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
    // Plan the fixture this consumer actually lowers, and install that plan on
    // the compiler under test.
    let lowered_fixture = match consumer {
        Px8jDirectRecursorConsumer::PendingLetProducer => &pending_let,
        Px8jDirectRecursorConsumer::ProducerCall
        | Px8jDirectRecursorConsumer::OrdinaryCall => &call,
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
        | Px8jDirectRecursorConsumer::ProducerCall => compiler
            .lower_computational_producer_expr(&mut builder, occurrence, &env, &active_frames),
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
        } => std::iter::once(body.as_ref()).chain(captures.iter()).collect(),
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
        fields: vec![("l".to_string(), RuntimeExpr::Value(RuntimeValue::Bool(true)))],
    }
}

#[cfg(test)]
fn two_child_record() -> RuntimeExpr {
    RuntimeExpr::Record {
        fields: vec![
            ("l".to_string(), RuntimeExpr::Value(RuntimeValue::Bool(true))),
            ("r".to_string(), RuntimeExpr::Value(RuntimeValue::Bool(false))),
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

    assert_eq!(arity_at(&straight, 1), 1, "then_expr is the one-child record");
    assert_eq!(arity_at(&straight, 2), 2, "else_expr is the two-child record");
    // The children swapped in the source; the derived origins swapped with them.
    assert_eq!(arity_at(&swapped, 1), 2, "then_expr is now the two-child record");
    assert_eq!(arity_at(&swapped, 2), 1, "else_expr is now the one-child record");
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
    let (plan, root) = planned_root_occurrence(&one_child_record());
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

#[test]
fn correspondence_adds_no_emitted_unit_to_the_production_census() {
    struct Census {
        file: &'static str,
        source: &'static str,
        builders: usize,
        definitions: usize,
        declarations: usize,
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
        },
        Census {
            file: "lowering/mod.rs",
            source: include_str!("../../mod.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
        },
        Census {
            file: "planning.rs",
            source: include_str!("../../../planning.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
        },
        Census {
            file: "planning/static_transition.rs",
            source: include_str!("../../../planning/static_transition.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
        },
        Census {
            file: "planning/static_transition/semantic_ir.rs",
            source: include_str!("../../../planning/static_transition/semantic_ir.rs"),
            builders: 0,
            definitions: 0,
            declarations: 0,
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
    }
}

/// N3 — **no plan `origin -> expr` lookup from a lowering consumer.**
///
/// Pinned at the producing end rather than by searching for callers: the plan
/// exposes exactly three origin accessors to the rest of the backend, and none
/// of them returns an expression. `RT-FNSPLIT-B2A-S`'s occurrence table was
/// deliberately NOT folded in for this reason (D8), so the lookup this pin
/// forbids does not exist to be called.
#[test]
fn the_plan_exposes_no_origin_to_expression_lookup() {
    let source = include_str!("../../../planning/static_transition.rs");
    let exported: Vec<&str> = source
        .lines()
        .filter(|line| line.trim_start().starts_with("pub(in crate::cranelift_backend) fn "))
        .map(|line| line.trim())
        .collect();
    assert_eq!(
        exported,
        vec![
            "pub(in crate::cranelift_backend) fn child_static_origin(",
            "pub(in crate::cranelift_backend) fn root_static_origin(",
            "pub(in crate::cranelift_backend) fn declaration_occurrence_origin(",
            "pub(in crate::cranelift_backend) fn plan_static_transition_graph(",
        ],
        "N3 -- the planner's exported surface changed; an origin->expression \
         accessor here would let a lowering consumer select a body by tag, \
         which is RT-FNSPLIT-B2A-S and not this unit"
    );
    assert!(
        !source.contains("-> Result<&'src RuntimeExpr"),
        "N3 -- no accessor may return a borrowed source expression"
    );
}
