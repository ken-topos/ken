//! Cross-tree test support for the `cranelift_backend` decomposition.
//!
//! NOT a `mod tests` (RT-SPLIT §10.1): these fixtures are shared by test
//! trees in different subtrees of the facade, whose lowest common ancestor is
//! the facade itself. Declared here at facade scope under `#[cfg(test)]` so
//! no production item is widened to serve a test (§10.4).

use super::*;

pub(in crate::cranelift_backend) fn oriented_test_ih_plan() -> crate::OrientedSubcontinuationPlanV1
{
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

pub(in crate::cranelift_backend) fn oriented_test_instance_layer(
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

#[derive(Clone, Copy, Debug)]
pub(in crate::cranelift_backend) enum Px8dsEdgeMutation {
    Delete,
    Duplicate,
    StaleParent,
    CrossSibling,
    WrongStaticParent,
}

#[repr(C)]
pub(in crate::cranelift_backend) struct BorrowedFixtureValue {
    pub(in crate::cranelift_backend) kind: u64,
    pub(in crate::cranelift_backend) tag: u64,
    pub(in crate::cranelift_backend) data: *const std::ffi::c_void,
    pub(in crate::cranelift_backend) len: usize,
}

#[repr(C)]
pub(in crate::cranelift_backend) struct NativeInvocationFixture {
    pub(in crate::cranelift_backend) process_input: *const BorrowedFixtureValue,
    pub(in crate::cranelift_backend) host_context: *mut std::ffi::c_void,
    pub(in crate::cranelift_backend) capability: u64,
    pub(in crate::cranelift_backend) native_int_arena: *mut crate::NativeIntArenaV1,
}

pub(in crate::cranelift_backend) fn self_consistent_root_join_site(
    site_id: u64,
) -> crate::NativeJoinPlanSiteV1 {
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

pub(in crate::cranelift_backend) fn oriented_test_interface(
    name: u8,
) -> crate::CheckedAnswerInterfaceV1 {
    let mut bytes = crate::CHECKED_ANSWER_INTERFACE_V1_HEADER.to_vec();
    bytes.push(name);
    crate::CheckedAnswerInterfaceV1::new(bytes).unwrap()
}

pub(in crate::cranelift_backend) fn oriented_test_frame(
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

pub(in crate::cranelift_backend) fn oriented_test_layer(
    frame_id: u64,
    role: RecursorLayerRole,
) -> ComputationalRecursorLayer {
    ComputationalRecursorLayer {
        cases: Vec::new(),
        default: RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: format!("oriented frame {frame_id}"),
        },
        outer_env: Vec::new(),
        provenance: RecursorFrameProvenance(frame_id),
        role,
        checked_frame_id: Some(frame_id),
        checked_invocation_id: Some(0),
        checked_invocation_source: None,
        checked_invocation_depth: 0,
        semantic_pending: true,
    }
}

pub(in crate::cranelift_backend) fn oriented_test_plan() -> crate::OrientedSubcontinuationPlanV1 {
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

pub(in crate::cranelift_backend) fn root_authority_test_lowering<'a>(
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
