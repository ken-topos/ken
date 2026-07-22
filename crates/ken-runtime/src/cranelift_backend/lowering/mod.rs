//! Lowering state, support methods, and the lowering SCC (RT-SPLIT §10.1).
//!
//! Slice 4 creates this module as a **scaffold**. Only `core` — the indivisible
//! 29-method SCC plus `compile_expr_into_module` — lives here today; the
//! `Lowering` state and the 79 acyclic support methods arrive in slice 5.
//!
//! Until then this module privately imports the still-residual parent items so
//! that `core` imports only from its own parent (§10.5). That is what lets
//! slice 5 move the residual state in without touching `core.rs` again.

pub(crate) mod core;

// Re-exported at facade scope (never crate scope) so `core` and its tests can
// glob from their own parent. This is namespace wiring, not a widening: every
// underlying declaration keeps its visibility, and the re-export cannot escape
// `crate::cranelift_backend`. Slice 5 replaces it with the real declarations,
// which is what lets `core.rs` stay untouched (§10.5).
// Public/re-exported parent surface (surface.rs, compiled.rs, planning.rs
// and the parent's own `pub use`s) comes through the glob; ancestor-PRIVATE
// items cannot be glob-imported and are listed explicitly below.
pub(in crate::cranelift_backend) use super::*;

pub(in crate::cranelift_backend) use super::{
    active_recursor_frame, append_recursive_argument_values, backend_module,
    borrowed_constructor_identity, compose_oriented_subcontinuation, console_stream_tag,
    create_policy_tag, decompose_computational_recursor, dynamic_host_result_producer_case,
    find_continuation_cursor, immediate_binder_eliminator, installed_oriented_eliminator_frames,
    lowered_char_list, lowered_value_kind, materialize_dynamic_constructor_env,
    ordinary_match_continuation, px8tr_deforested_answer_route_enabled,
    reaches_environment_computational_recursor, rebuild_recursive_argument,
    recursor_invocation_is_checked, requires_heterogeneous_deforestation, resource_open_mode_tag,
    same_recursive_argument_shapes, select_computational_case, select_dynamic_constructor_case,
    select_ordinary_case, source_active_cursor, source_case_has_no_checked_control_markers, types,
    validate_dynamic_constructor_alternatives, validate_recursor_invocation_segment,
    verify_cranelift_function, AbiParam, ActiveContinuationFrame, ActiveRecursiveDeclarationV1,
    ArmedInvocation, BTreeMap, BTreeSet, BoundedNatV1, CheckedRecursiveInvocationInstance,
    CompiledModule, ComputationalEliminatorFrame, ComputationalRecursorFramePayload,
    CraneliftBackendError, DeferredConstructorCaseEnvironment, DynamicConstructorAlternativeV1,
    DynamicConstructorContinuation, DynamicConstructorV1, EliminatorFrame, Function,
    FunctionBuilder, FunctionBuilderContext, ImmediateBinderEliminator, InvocationTemplateRef,
    Linkage, Lowered, Lowering, MemFlags, Module, NativeScalarPairV1, NativeSeedEnvironment,
    OrdinaryEliminatorFrame, OwnedSelectedScope, PendingLetContinuationFrame, RecursorLayerRole,
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr, RuntimePartiality, RuntimePrimitive,
    RuntimeSymbol, RuntimeTrap, RuntimeTrapCode, RuntimeValue, ScalarMergeKind,
    SelectedCaseReturnDelimiter, SourceBranchFanout, SourceCallOutcome,
    SourceComputationalAnswerRoute, SourceContinuation, SourceContinuationTerminal, SourceControl,
    SourceJoinTarget, SourceMachineState, SourcePrefixTerminal, SourceSelectedContinuation,
    StackSlotData, StackSlotKind, StructuralNatV1, UserFuncName,
    CRANELIFT_HOST_EFFECT_CONSUMERS_V1, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS,
};

// `#[cfg(test)]`-only parent items: an unconditional `use` of these breaks
// the non-test build, which the test build cannot show you.
#[cfg(test)]
pub(in crate::cranelift_backend) use crate::RuntimeMatchCase;

#[cfg(test)]
pub(in crate::cranelift_backend) use super::{
    px8j_record_recursor_carrier, px8j_record_source_event, px8tr_record_trap_provenance,
    test_only_distinguished_root_join_plan, BoundedNatFixtureObservation,
    BoundedNatLoweringMutation, NativeIntLoweringMutation, Px8jDirectRecursorConsumer,
    Px8jProducerPath, Px8jRecursorMalformation, Px8jSourceTraceEvent, Px8trTrapProvenanceEvent,
};

#[cfg(test)]
pub(in crate::cranelift_backend) use super::test_support::*;
