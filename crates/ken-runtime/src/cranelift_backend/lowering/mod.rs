//! Lowering state, the acyclic support methods, and the lowered-value,
//! continuation/control, source-machine, bounded-Nat and dynamic-constructor
//! data model, plus their free helpers (RT-SPLIT §10.1/§10.2).
//!
//! The indivisible 29-method lowering SCC lives in the child module `core`,
//! which consumes this module's private items as its ancestor and therefore
//! needs no widening (§10.4, "hierarchy is load-bearing").
//!
//! Imports below name their **owning** module, crate root, or external
//! dependency directly. This module must never import through the facade:
//! §10.3 forbids an implementation module doing so, and an omnibus
//! `use super::*` would hide the real `compiled` / `planning` / `surface`
//! edges behind a namespace. The `pub(in crate::cranelift_backend)` on each
//! is namespace wiring, not a widening — it re-exports names at their existing
//! visibility so `core` and its subject tests inherit them, and it cannot
//! escape `crate::cranelift_backend`.

pub(in crate::cranelift_backend) mod core;

// --- external dependencies -------------------------------------------------
pub(in crate::cranelift_backend) use std::collections::{BTreeMap, BTreeSet};

pub(in crate::cranelift_backend) use cranelift_codegen::ir::{
    types, AbiParam, FuncRef, Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind,
    UserFuncName,
};
pub(in crate::cranelift_backend) use cranelift_codegen::verify_function;
pub(in crate::cranelift_backend) use cranelift_frontend::{
    FunctionBuilder, FunctionBuilderContext,
};
pub(in crate::cranelift_backend) use cranelift_module::{Linkage, Module};

// --- crate root ------------------------------------------------------------
pub(in crate::cranelift_backend) use crate::{
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr, RuntimeGroundValue, RuntimePartiality,
    RuntimePrimitive, RuntimeSymbol, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
};

// --- sibling backend modules, named at their OWNERS (§10.3 DAG:
//     `lowering support -> surface`, `lowering::core -> compiled`) ----------
pub(in crate::cranelift_backend) use super::compiled::{CompiledModule, ResultDecoder};
pub(in crate::cranelift_backend) use super::planning::{
    collect_checked_oriented_markers, collect_checked_subcontinuation_frames,
    validate_oriented_subcontinuation_transport, CheckedOrientedMarkerSets,
};
pub(in crate::cranelift_backend) use super::surface::{
    backend, backend_module, unsupported, BackendFailure, CraneliftBackendError,
    NativeSeedEnvironment,
};

// `#[cfg(test)]`-only: an unconditional `use` of this breaks the non-test
// build, which the test build cannot show you.
#[cfg(test)]
pub(in crate::cranelift_backend) use crate::RuntimeMatchCase;

const CRANELIFT_HOST_EFFECT_CONSUMERS_V1: [ken_host::HostOpV1; 13] = [
    ken_host::HostOpV1::ConsoleWrite,
    ken_host::HostOpV1::ConsoleFlush,
    ken_host::HostOpV1::ConsoleIsTerminal,
    ken_host::HostOpV1::FsReadFile,
    ken_host::HostOpV1::FsWriteFile,
    ken_host::HostOpV1::FsChangeMode,
    ken_host::HostOpV1::FsOpen,
    ken_host::HostOpV1::FsHandleMetadata,
    ken_host::HostOpV1::FsReadAt,
    ken_host::HostOpV1::FsWriteAt,
    ken_host::HostOpV1::ResourceRelease,
    ken_host::HostOpV1::BufferAllocate,
    ken_host::HostOpV1::BufferFreeze,
];
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoundedNatLoweringMutation {
    Exact,
    BrokenDecrement,
    RawScalarPredecessor,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Px8jProducerPath {
    Composed,
    DeferredConstructor,
    SourceMachine,
}
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Px8jSourceTraceEvent {
    Mint {
        path: Px8jProducerPath,
        origin: RecursorProducerOriginId,
        cursor: ContinuationCursorId,
        siblings: usize,
        parent_scope: Option<RecursorProducerOriginId>,
    },
    Carrier {
        path: Px8jProducerPath,
        origin: RecursorProducerOriginId,
        cursor: ContinuationCursorId,
        sibling_position: usize,
    },
    Install {
        origin: RecursorProducerOriginId,
        selection_cursor: ContinuationCursorId,
        sibling_position: usize,
        exits: Vec<(RecursorProducerOriginId, Option<RecursorProducerOriginId>)>,
    },
    DirectConsume {
        origin: RecursorProducerOriginId,
        selection_cursor: ContinuationCursorId,
        sibling_position: usize,
        exits: Vec<(RecursorProducerOriginId, Option<RecursorProducerOriginId>)>,
    },
    Selection {
        origin: RecursorProducerOriginId,
    },
    Exit {
        origin: RecursorProducerOriginId,
        scope_origin: RecursorProducerOriginId,
        parent_scope: Option<RecursorProducerOriginId>,
    },
    ReturnHole {
        cursor: ContinuationCursorId,
    },
    ResumeOuter {
        cursor: ContinuationCursorId,
    },
}
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Px8trTrapProvenanceEvent {
    CheckedRecursorDefault {
        checked_frame_id: u64,
        actual_constructor: Option<RuntimeSymbol>,
        trap: RuntimeTrap,
    },
    DeforestedAnswerResumed {
        checked_frame_id: u64,
        actual_constructor: Option<RuntimeSymbol>,
        return_constructor: RuntimeSymbol,
    },
    FinalProcessObjectTrap {
        trap: RuntimeTrap,
    },
}
#[cfg(test)]
fn px8j_record_source_event(event: Px8jSourceTraceEvent) {
    PX8J_SOURCE_TRACE.with(|trace| trace.borrow_mut().push(event));
}
#[cfg(test)]
fn px8tr_record_trap_provenance(event: Px8trTrapProvenanceEvent) {
    PX8TR_TRAP_PROVENANCE.with(|trace| trace.borrow_mut().push(event));
}
#[cfg(test)]
fn px8tr_deforested_answer_route_enabled() -> bool {
    !PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE.get()
}

#[cfg(not(test))]
fn px8tr_deforested_answer_route_enabled() -> bool {
    true
}
#[cfg(test)]
fn px8j_record_recursor_carrier(path: Px8jProducerPath, value: &Lowered) {
    let Lowered::ComputationalRecursorClosure { invocation, .. } = value else {
        return;
    };
    px8j_record_source_event(Px8jSourceTraceEvent::Carrier {
        path,
        origin: invocation.origin,
        cursor: invocation.resume_cursor,
        sibling_position: invocation.sibling_position,
    });
}
fn verify_cranelift_function(
    func: &Function,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<(), CraneliftBackendError> {
    verify_function(func, isa).map_err(|err| backend(BackendFailure::Verifier(err.to_string())))
}

// RT-SPLIT slice 5 (Architect `evt_3tgaw9ws44fqg`): test-only adapter letting
// the two artifact-subject `px8i_*` tests reach the private original across the
// ownership boundary. No ISA flags, validation, defaults, transformation, or
// error remapping — a single delegating call. Test scaffolding: absent from
// production builds, zero AC-7 production seams.
// Same shape, same rationale: the artifact-subject `px8i_local_helpers_*` test
// discriminates this lowering-private helper. Single delegating call, no policy.
#[cfg(test)]
pub(super) fn require_i64_for_artifact_tests(
    builder: &mut FunctionBuilder<'_>,
    actual: cranelift_codegen::ir::Value,
    expected: i64,
) {
    Lowering::require_i64(builder, actual, expected)
}

#[cfg(test)]
pub(super) fn verify_cranelift_function_for_artifact_tests(
    func: &Function,
    isa: &dyn cranelift_codegen::isa::TargetIsa,
) -> Result<(), CraneliftBackendError> {
    verify_cranelift_function(func, isa)
}
struct Lowering<'a> {
    seed_env: &'a NativeSeedEnvironment,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    declaration_stack: Vec<RuntimeSymbol>,
    active_recursive_declarations: Vec<ActiveRecursiveDeclarationV1>,
    result_table: BTreeMap<i64, RuntimeGroundValue>,
    next_token: i64,
    next_recursor_frame_provenance: u64,
    next_recursor_producer_origin: u64,
    next_continuation_activation: u64,
    next_continuation_cursor: u64,
    next_source_join: u64,
    next_source_predecessor: u64,
    live_source_continuations: usize,
    source_control_root: Option<ContinuationCursorId>,
    active_oriented_semantic_regions: usize,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    consumed_join_sites: BTreeSet<u64>,
    root_terminal_authority: Option<RootTerminalAnswerAuthority>,
    active_join_site: Option<u64>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
    consumed_subcontinuation_frames: BTreeSet<(u64, u64)>,
    active_subcontinuation_frame: Option<u64>,
    consumed_recursive_call_templates: BTreeSet<u64>,
    pending_recursive_call: Option<CheckedRecursiveInvocationInstance>,
    pending_computational_ih_call: Option<u64>,
    active_recursive_invocations: Vec<CheckedRecursiveInvocationInstance>,
    next_recursive_invocation_instance: u64,
    dynamic_splice_edges: BTreeMap<DynamicSpliceEdgeId, DynamicSpliceEdge>,
    next_dynamic_splice_edge: u64,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
    process_object: bool,
    process_symbols: crate::NativeProcessSymbols,
    host_dispatch: Option<FuncRef>,
    invocation_pointer: Option<cranelift_codegen::ir::Value>,
    native_int_arena: Option<cranelift_codegen::ir::Value>,
    native_int_binop: Option<FuncRef>,
    native_int_compare: Option<FuncRef>,
    native_int_intern: Option<FuncRef>,
    native_int_narrow: Option<FuncRef>,
    native_int_export: Option<FuncRef>,
    native_int_tags: BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    #[cfg(test)]
    native_int_mutation: NativeIntLoweringMutation,
    #[cfg(test)]
    bounded_nat_mutation: BoundedNatLoweringMutation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecursorFrameProvenance(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InvocationTemplateRef {
    SameSccCall(u64),
    ComputationalIHCall(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CheckedRecursiveInvocationInstance {
    source: InvocationTemplateRef,
    invocation_instance_id: u64,
    semantic_depth: usize,
    dynamic_splice_edge: Option<DynamicSpliceEdgeId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DynamicSpliceEdgeId(u64);
/// The unique compiler-owned authority to splice one completed dynamic child
/// invocation into one exact open parent occurrence. Lowered values retain
/// only the inert `DynamicSpliceEdgeId`; this non-`Clone` ledger entry is
/// removed and consumed before any CFG is emitted.
struct DynamicSpliceEdge {
    edge_id: DynamicSpliceEdgeId,
    child_invocation_instance_id: u64,
    parent_invocation_instance_id: u64,
    checked_call_template_id: u64,
    parent_frame_template_id: u64,
    segment_site_id: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContinuationActivationId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ContinuationCursorId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RecursorProducerOriginId(u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecursorLayerRole {
    SelectsOccurrence {
        origin: RecursorProducerOriginId,
    },
    ExitsScope {
        origin: RecursorProducerOriginId,
        scope_origin: RecursorProducerOriginId,
        parent_scope: Option<RecursorProducerOriginId>,
    },
}
#[derive(Clone)]
struct ComputationalRecursorFramePayload {
    cases: Vec<crate::RuntimeComputationalMatchCase>,
    default: RuntimeTrap,
    outer_env: Vec<Lowered>,
    provenance: RecursorFrameProvenance,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
}
#[derive(Clone)]
struct OwnedSelectedScope {
    scope_origin: RecursorProducerOriginId,
    parent_scope: Option<RecursorProducerOriginId>,
    frame: ComputationalRecursorFramePayload,
}
#[derive(Clone, Copy)]
struct NativeScalarPairV1 {
    tag: cranelift_codegen::ir::Value,
    payload: cranelift_codegen::ir::Value,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeIntLoweringMutation {
    Exact,
    Wrapping,
    Trap,
    SuppressTerminalExport,
    CorruptTerminalExport,
}
#[derive(Clone)]
enum Lowered {
    Int {
        value: cranelift_codegen::ir::Value,
        known: Option<i64>,
    },
    Bool {
        value: cranelift_codegen::ir::Value,
        known: Option<bool>,
    },
    ProcessExitStatus {
        value: cranelift_codegen::ir::Value,
    },
    CapabilityToken {
        value: cranelift_codegen::ir::Value,
    },
    ResourceToken {
        value: cranelift_codegen::ir::Value,
    },
    BoundedNat(BoundedNatV1),
    /// A structural `Nat` constructed by checked Ken. Unlike `BoundedNat`,
    /// this value is not a host-reply proof carrier; it is the ordinary unary
    /// constructor representation deforested to one native scalar.
    StructuralNat(StructuralNatV1),
    ResponseBytes {
        pointer: cranelift_codegen::ir::Value,
        len: cranelift_codegen::ir::Value,
    },
    HostResult {
        success: cranelift_codegen::ir::Value,
        error: Box<Lowered>,
        ok: Box<Lowered>,
        err_constructor: String,
        ok_constructor: String,
    },
    DynamicConstructor(DynamicConstructorV1),
    Bytes(Vec<u8>),
    BorrowedNativeValue {
        pointer: cranelift_codegen::ir::Value,
    },
    BorrowedOption {
        present: cranelift_codegen::ir::Value,
        value: cranelift_codegen::ir::Value,
        none: String,
        some: String,
    },
    String(String),
    Constructor {
        constructor: String,
        args: Vec<Lowered>,
    },
    Record {
        fields: Vec<(String, Lowered)>,
    },
    Closure {
        captures: Vec<Lowered>,
        params: Vec<String>,
        body: RuntimeExpr,
    },
    DeclarationClosure {
        symbol: RuntimeSymbol,
        captures: Vec<Lowered>,
        params: Vec<String>,
        body: RuntimeExpr,
    },
    ComputationalRecursorClosure {
        residual: Box<Lowered>,
        activation: ContinuationActivationId,
        invocation: RecursorInvocationSegment,
    },
    /// A tail-recursive edge already emitted as a CFG jump. The current block
    /// is predecessor-free; enclosing scalar combinators propagate this
    /// marker so it cannot be confused with an ordinary or terminal value.
    RecursiveBackedge,
    Trap(RuntimeTrap),
}
#[derive(Clone)]
struct ActiveRecursiveDeclarationV1 {
    symbol: RuntimeSymbol,
    header: Option<cranelift_codegen::ir::Block>,
    argument_templates: Vec<Lowered>,
    induction: Option<Lowered>,
}
#[derive(Clone, Copy)]
struct StructuralNatV1 {
    value: cranelift_codegen::ir::Value,
}
/// Compact private observation of a structural Nat minted from a checked host
/// reply. The scalar never enters Runtime IR or the Ken surface: only the
/// Zero/Suc eliminators below can observe it.
#[derive(Clone, Copy)]
struct BoundedNatV1 {
    value: cranelift_codegen::ir::Value,
}
impl BoundedNatV1 {
    fn mint_after_reply_validation(value: cranelift_codegen::ir::Value) -> Self {
        Self { value }
    }

    fn predecessor(self, builder: &mut FunctionBuilder<'_>) -> Self {
        Self::derived_from_validated(builder.ins().iadd_imm(self.value, -1))
    }

    fn derived_from_validated(value: cranelift_codegen::ir::Value) -> Self {
        Self { value }
    }
}
#[derive(Clone)]
struct DynamicConstructorV1 {
    discriminator: cranelift_codegen::ir::Value,
    alternatives: Vec<DynamicConstructorAlternativeV1>,
}
#[derive(Clone)]
struct DynamicConstructorAlternativeV1 {
    tag: i64,
    constructor: RuntimeSymbol,
    fields: Vec<Lowered>,
}
const MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS: i64 = -3;
fn validate_dynamic_constructor_alternatives<'a>(
    alternatives: impl IntoIterator<Item = (i64, &'a str)>,
) -> Result<(), CraneliftBackendError> {
    let mut tags = BTreeSet::new();
    let mut constructors = BTreeSet::new();
    let mut count = 0;
    for (tag, constructor) in alternatives {
        count += 1;
        if !tags.insert(tag) {
            return Err(unsupported(
                "DynamicConstructor",
                format!("duplicate alternative tag {tag}"),
            ));
        }
        if !constructors.insert(constructor) {
            return Err(unsupported(
                "DynamicConstructor",
                format!("duplicate alternative constructor {constructor}"),
            ));
        }
    }
    if count == 0 {
        return Err(unsupported(
            "DynamicConstructor",
            "closed alternative table is empty",
        ));
    }
    Ok(())
}
fn select_dynamic_constructor_case<'a>(
    cases: &'a [crate::RuntimeMatchCase],
    alternative: &DynamicConstructorAlternativeV1,
    default: &'a RuntimeTrap,
) -> Result<Result<&'a crate::RuntimeMatchCase, &'a RuntimeTrap>, CraneliftBackendError> {
    let mut selected = cases
        .iter()
        .filter(|case| case.constructor == alternative.constructor);
    let Some(case) = selected.next() else {
        return Ok(Err(default));
    };
    if selected.next().is_some() {
        return Err(unsupported(
            "DynamicConstructor",
            format!(
                "source match duplicates constructor {}",
                alternative.constructor
            ),
        ));
    }
    if case.binders != alternative.fields.len() {
        return Err(unsupported(
            "DynamicConstructor",
            format!(
                "case {} expects {} binders but alternative has {} fields",
                case.constructor,
                case.binders,
                alternative.fields.len()
            ),
        ));
    }
    Ok(Ok(case))
}
fn materialize_dynamic_constructor_env(
    alternative: &DynamicConstructorAlternativeV1,
    env: &[Lowered],
) -> Vec<Lowered> {
    let mut arm_env = alternative.fields.clone();
    arm_env.extend_from_slice(env);
    arm_env
}
fn console_stream_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if !args.is_empty() {
        return None;
    }
    if constructor.ends_with("::Stdin") {
        Some(0)
    } else if constructor.ends_with("::Stdout") {
        Some(1)
    } else if constructor.ends_with("::Stderr") {
        Some(2)
    } else {
        None
    }
}
fn create_policy_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if !args.is_empty() {
        return None;
    }
    if constructor.ends_with("::CreateNew") {
        Some(0)
    } else if constructor.ends_with("::CreateOrTruncate") {
        Some(1)
    } else if constructor.ends_with("::CreateOrKeep") {
        Some(2)
    } else {
        None
    }
}
fn resource_open_mode_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if constructor.ends_with("::ResourceRead") && args.is_empty() {
        Some(0)
    } else if constructor.ends_with("::ResourceMetadata") && args.is_empty() {
        Some(1)
    } else if constructor.ends_with("::ResourceWriteCreate") && args.len() == 1 {
        create_policy_tag(&args[0]).map(|tag| tag + 2)
    } else {
        None
    }
}
fn lowered_char_list(value: &Lowered) -> Option<Vec<u8>> {
    let Lowered::Constructor { constructor, args } = value else {
        return None;
    };
    if constructor.ends_with("::Nil") && args.is_empty() {
        return Some(Vec::new());
    }
    if !constructor.ends_with("::Cons") || args.len() != 2 {
        return None;
    }
    let Lowered::Int {
        known: Some(head), ..
    } = &args[0]
    else {
        return None;
    };
    let head = u8::try_from(*head).ok()?;
    let mut tail = lowered_char_list(&args[1])?;
    tail.insert(0, head);
    Some(tail)
}
fn dynamic_host_result_producer_case<'a>(
    cases: &'a [crate::RuntimeMatchCase],
    constructor: &str,
) -> Result<Option<&'a crate::RuntimeMatchCase>, CraneliftBackendError> {
    let Some(case) = cases.iter().find(|case| case.constructor == constructor) else {
        return Ok(None);
    };
    if case.binders != 1 {
        return Err(unsupported(
            "ComputationalMatch",
            format!(
                "dynamic HostResult tree producer case {} expects exactly one binder, got {}",
                case.constructor, case.binders
            ),
        ));
    }
    Ok(Some(case))
}
#[derive(Clone, Copy)]
struct ComputationalEliminatorFrame<'a> {
    cases: &'a [crate::RuntimeComputationalMatchCase],
    default: &'a RuntimeTrap,
    env: &'a [Lowered],
    retained_scrutinee_index: Option<usize>,
    deferred_constructor_case: Option<&'a DeferredConstructorCaseEnvironment<'a>>,
    provenance: RecursorFrameProvenance,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
}
#[derive(Clone, Copy)]
struct OrdinaryEliminatorFrame<'a> {
    cases: &'a [crate::RuntimeMatchCase],
    default: &'a RuntimeTrap,
    env: &'a [Lowered],
    retained_scrutinee_index: Option<usize>,
    deferred_constructor_case: Option<&'a DeferredConstructorCaseEnvironment<'a>>,
}
#[derive(Clone, Copy)]
struct PendingLetContinuationFrame<'a> {
    residual: &'a Lowered,
    args: &'a [RuntimeExpr],
    env: &'a [Lowered],
}
#[derive(Clone, Copy)]
struct ActiveContinuationFrame<'a> {
    activation: ContinuationActivationId,
    cursor: ContinuationCursorId,
    parent: Option<&'a ActiveContinuationFrame<'a>>,
    pending: &'a [EliminatorFrame<'a>],
    selected_ancestry: &'a [RecursorFrameProvenance],
    source_lineage: &'a [SourceSelectedContinuation<'a>],
    source_selected_cursor: Option<ContinuationCursorId>,
    selected_scope: Option<&'a OwnedSelectedScope>,
}
#[derive(Clone)]
struct ComputationalRecursorLayer {
    cases: Vec<crate::RuntimeComputationalMatchCase>,
    default: RuntimeTrap,
    outer_env: Vec<Lowered>,
    provenance: RecursorFrameProvenance,
    role: RecursorLayerRole,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
    semantic_pending: bool,
}
#[derive(Clone)]
struct RecursorInvocationSegment {
    origin: RecursorProducerOriginId,
    /// Declaration-order field position inside the one selected constructor
    /// case. Siblings share `origin`; this position distinguishes their
    /// immutable carriers through the consumer boundary.
    sibling_position: usize,
    selection: ComputationalRecursorLayer,
    unwind: RecursorUnwindStack,
    resume_cursor: ContinuationCursorId,
    checked_invocation: Option<CheckedRecursiveInvocationInstance>,
    computational_ih_slot_template_id: Option<u64>,
    /// Inert handles into `Lowering::dynamic_splice_edges`. Cloning a lowered
    /// recursor can copy a handle, but only one clone can consume the unique
    /// compiler-owned edge; every replay rejects before CFG.
    dynamic_splice_edges: Vec<DynamicSpliceEdgeId>,
    /// Immutable mint-time witness for every already-open control extent.
    /// Qualification may attach a fresh invocation identity later, but it may
    /// not delete, duplicate, reorder, or transplant an exit obligation.
    open_control_obligations: Vec<OpenControlObligation>,
}
#[derive(Clone)]
struct RecursorUnwindStack {
    later_wrappers_in_construction_order: Vec<ComputationalRecursorLayer>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpenControlObligation {
    scope_origin: RecursorProducerOriginId,
    parent_scope: Option<RecursorProducerOriginId>,
    checked_frame_id: Option<u64>,
    semantic_pending: bool,
}
fn open_control_obligations(unwind: &RecursorUnwindStack) -> Vec<OpenControlObligation> {
    unwind
        .later_wrappers_in_construction_order
        .iter()
        .filter_map(|layer| match layer.role {
            RecursorLayerRole::ExitsScope {
                scope_origin,
                parent_scope,
                ..
            } => Some(OpenControlObligation {
                scope_origin,
                parent_scope,
                checked_frame_id: layer.checked_frame_id,
                semantic_pending: layer.semantic_pending,
            }),
            RecursorLayerRole::SelectsOccurrence { .. } => None,
        })
        .collect()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AffineSpliceState {
    Open,
    Consumed,
}
/// Move-only compiler capability for one oriented segment splice.  It is
/// deliberately not `Clone`; validation consumes the sole open state before
/// any CFG or consumer lowering begins.
struct AffineSpliceCapability {
    state: AffineSpliceState,
}
impl AffineSpliceCapability {
    fn consume(&mut self) -> Result<(), CraneliftBackendError> {
        if std::mem::replace(&mut self.state, AffineSpliceState::Consumed)
            == AffineSpliceState::Consumed
        {
            return Err(unsupported(
                "OrientedSubcontinuation",
                "affine splice capability was consumed more than once",
            ));
        }
        Ok(())
    }
}
#[derive(Clone)]
struct OrientedControlLedgerEntry {
    frame_id: Option<u64>,
    invocation_id: Option<u64>,
    role: RecursorLayerRole,
    checked_witness: Option<crate::OrientedControlWitnessV1>,
}
fn oriented_layer_is_pending_semantic(layer: &ComputationalRecursorLayer) -> bool {
    layer.semantic_pending
}
fn validate_oriented_control_projection(
    producer_origin: RecursorProducerOriginId,
    layers: &[ComputationalRecursorLayer],
) -> Result<(), CraneliftBackendError> {
    let mut invocation_sources = BTreeMap::new();
    let mut open_scopes = BTreeMap::new();
    for layer in layers {
        let role_origin = match layer.role {
            RecursorLayerRole::SelectsOccurrence { origin }
            | RecursorLayerRole::ExitsScope { origin, .. } => origin,
        };
        if role_origin != producer_origin {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "control occurrence was transplanted across producer regions",
            ));
        }
        match (layer.checked_invocation_id, layer.checked_invocation_source) {
            (Some(instance), Some(source)) => {
                if invocation_sources
                    .insert(instance, source)
                    .is_some_and(|old| old != source)
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "one invocation instance is shared by distinct checked templates",
                    ));
                }
            }
            (None, Some(_)) => {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked invocation source has no affine instance identity",
                ));
            }
            _ => {}
        }
        match (layer.role, layer.semantic_pending) {
            (RecursorLayerRole::SelectsOccurrence { .. }, false) => {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "pending selection was misclassified as control-only",
                ));
            }
            _ => {}
        }
        if let RecursorLayerRole::ExitsScope {
            scope_origin,
            parent_scope,
            ..
        } = layer.role
        {
            if layer.checked_invocation_id.is_some() {
                if open_scopes.insert(scope_origin, parent_scope).is_some() {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "open control obligation is duplicated",
                    ));
                }
            }
        }
    }
    for parent in open_scopes.values().flatten() {
        if !open_scopes.contains_key(parent) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "open control obligation has a stale or cross-region parent",
            ));
        }
    }
    Ok(())
}
struct OwnedOrientedSubcontinuationSegment {
    producer_origin: RecursorProducerOriginId,
    sibling_position: usize,
    activation: ContinuationActivationId,
    segment_site_id: Option<u64>,
    input_interface: Option<crate::CheckedAnswerInterfaceV1>,
    output_interface: Option<crate::CheckedAnswerInterfaceV1>,
    semantic_frames: Vec<ComputationalRecursorLayer>,
    control_ledger: Vec<OrientedControlLedgerEntry>,
    resume_cursor: ContinuationCursorId,
    capability: AffineSpliceCapability,
}
struct InstalledOrientedSubcontinuationSegment {
    checked: bool,
    producer_origin: RecursorProducerOriginId,
    sibling_position: usize,
    activation: ContinuationActivationId,
    semantic_frames: Vec<ComputationalRecursorLayer>,
    control_ledger: Vec<OrientedControlLedgerEntry>,
    resume_cursor: ContinuationCursorId,
}
impl RecursorInvocationSegment {
    fn new(
        origin: RecursorProducerOriginId,
        sibling_position: usize,
        selection: ComputationalRecursorLayer,
        unwind: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        checked_invocation: Option<CheckedRecursiveInvocationInstance>,
        computational_ih_slot_template_id: Option<u64>,
    ) -> Self {
        let open_control_obligations = open_control_obligations(&unwind);
        Self {
            origin,
            sibling_position,
            selection,
            unwind,
            resume_cursor,
            checked_invocation,
            computational_ih_slot_template_id,
            dynamic_splice_edges: Vec::new(),
            open_control_obligations,
        }
    }

    fn validate_open_control_obligations(&self) -> Result<(), CraneliftBackendError> {
        if open_control_obligations(&self.unwind) != self.open_control_obligations {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "open control obligation set changed after affine mint",
            ));
        }
        Ok(())
    }
}
fn decompose_computational_recursor(
    value: Lowered,
) -> (
    Lowered,
    Option<(ContinuationActivationId, RecursorInvocationSegment)>,
) {
    match value {
        Lowered::ComputationalRecursorClosure {
            residual,
            activation,
            invocation,
        } => (*residual, Some((activation, invocation))),
        value => (value, None),
    }
}
fn checked_invocation_frame_templates(
    plan: &crate::OrientedSubcontinuationPlanV1,
    source: InvocationTemplateRef,
) -> Result<&[u64], CraneliftBackendError> {
    match source {
        InvocationTemplateRef::SameSccCall(call_template_id) => plan
            .recursive_call(call_template_id)
            .map(|call| call.callee_frame_templates.as_slice())
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic invocation has no checked same-SCC call template",
                )
            }),
        InvocationTemplateRef::ComputationalIHCall(call_template_id) => plan
            .computational_ih_call(call_template_id)
            .map(|call| call.callee_frame_templates.as_slice())
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic invocation has no checked computational IH call template",
                )
            }),
    }
}
fn instantiate_checked_invocation_segment(
    plan: &crate::OrientedSubcontinuationPlanV1,
    invocation: CheckedRecursiveInvocationInstance,
    segment: &mut RecursorInvocationSegment,
) -> Result<(), CraneliftBackendError> {
    let frame_templates = checked_invocation_frame_templates(plan, invocation.source)?;
    let expected = frame_templates.iter().copied().collect::<BTreeSet<_>>();
    let mut instantiated = BTreeSet::new();
    let mut visit = |layer: &mut ComputationalRecursorLayer| {
        let Some(frame_id) = layer.checked_frame_id else {
            return Ok(());
        };
        let frame = plan.frame(frame_id).ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic recursive layer has no checked frame entry",
            )
        })?;
        if frame.runtime_frame_fingerprint
            != crate::compiler_private_computational_match_frame_fingerprint(
                &layer.cases,
                &layer.default,
            )
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic recursive layer does not match its checked frame template",
            ));
        }
        if !expected.contains(&frame_id) {
            return Ok(());
        }
        match layer.checked_invocation_id {
            None => {
                layer.checked_invocation_id = Some(invocation.invocation_instance_id);
                layer.checked_invocation_source = Some(invocation.source);
                layer.checked_invocation_depth = invocation.semantic_depth;
            }
            Some(existing) if existing == invocation.invocation_instance_id => {
                if layer.checked_invocation_source != Some(invocation.source) {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic invocation source changed after qualification",
                    ));
                }
            }
            Some(_) => return Ok(()),
        }
        if !instantiated.insert(frame_id) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "one invocation instantiates a checked frame template more than once",
            ));
        }
        Ok(())
    };
    visit(&mut segment.selection)?;
    for layer in &mut segment.unwind.later_wrappers_in_construction_order {
        visit(layer)?;
    }
    if instantiated != expected {
        let actual = std::iter::once(&segment.selection)
            .chain(segment.unwind.later_wrappers_in_construction_order.iter())
            .map(|layer| (layer.checked_frame_id, layer.checked_invocation_id))
            .collect::<Vec<_>>();
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "computational invocation {:?} does not carry its exact checked frame sequence: expected={expected:?} instantiated={instantiated:?} actual={actual:?}",
                invocation.source,
            ),
        ));
    }
    segment.checked_invocation = None;
    Ok(())
}
/// Test-only causal switch for the retired cross-instance flat ordering.
///
/// This is feature-gated so ordinary Runtime and CLI artifacts cannot select
/// the invalid ordering. PX8-DS integration tests use it to drive the exact
/// checked source through the former production consumer.
#[cfg(feature = "px8-ds-test-support")]
#[doc(hidden)]
pub fn with_px8ds_retired_flat_order<R>(run: impl FnOnce() -> R) -> R {
    struct Restore(bool);

    impl Drop for Restore {
        fn drop(&mut self) {
            PX8DS_RETIRED_FLAT_ORDER.with(|enabled| enabled.set(self.0));
        }
    }

    let previous = PX8DS_RETIRED_FLAT_ORDER.with(|enabled| enabled.replace(true));
    let _restore = Restore(previous);
    run()
}
fn px8ds_retired_flat_order_enabled() -> bool {
    #[cfg(any(test, feature = "px8-ds-test-support"))]
    {
        return PX8DS_RETIRED_FLAT_ORDER.with(std::cell::Cell::get);
    }
    #[cfg(not(any(test, feature = "px8-ds-test-support")))]
    {
        false
    }
}
fn compose_oriented_subcontinuation(
    plan: Option<&crate::OrientedSubcontinuationPlanV1>,
    invocation: Option<CheckedRecursiveInvocationInstance>,
    activation: ContinuationActivationId,
    mut segment: RecursorInvocationSegment,
    dynamic_splice_edges: Vec<DynamicSpliceEdge>,
) -> Result<InstalledOrientedSubcontinuationSegment, CraneliftBackendError> {
    segment.validate_open_control_obligations()?;
    let invocation = invocation.or(segment.checked_invocation);
    if let Some(invocation) = invocation {
        let plan = plan.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic invocation has no checked oriented plan",
            )
        })?;
        instantiate_checked_invocation_segment(plan, invocation, &mut segment)?;
    }
    let producer_origin = segment.origin;
    let sibling_position = segment.sibling_position;
    let resume_cursor = segment.resume_cursor;
    let mut control_layers =
        Vec::with_capacity(1 + segment.unwind.later_wrappers_in_construction_order.len());
    control_layers.push(segment.selection);
    control_layers.extend(
        segment
            .unwind
            .later_wrappers_in_construction_order
            .into_iter()
            .rev(),
    );
    let mut control_ledger = control_layers
        .iter()
        .map(|layer| OrientedControlLedgerEntry {
            frame_id: layer.checked_frame_id,
            invocation_id: layer.checked_invocation_id,
            role: layer.role,
            checked_witness: None,
        })
        .collect::<Vec<_>>();
    validate_oriented_control_projection(producer_origin, &control_layers)?;
    #[cfg(test)]
    px8j_record_source_event(Px8jSourceTraceEvent::DirectConsume {
        origin: segment.origin,
        selection_cursor: segment.resume_cursor,
        sibling_position: segment.sibling_position,
        exits: control_layers
            .iter()
            .rev()
            .filter_map(|layer| match layer.role {
                RecursorLayerRole::ExitsScope {
                    scope_origin,
                    parent_scope,
                    ..
                } => Some((scope_origin, parent_scope)),
                RecursorLayerRole::SelectsOccurrence { .. } => None,
            })
            .collect(),
    });

    // A selected frame is pending semantic work. A freshly instantiated IH
    // layer remains semantic even when its control projection is already in
    // ExitsScope phase. Inherited exit rows carry no fresh invocation source:
    // their transformer was consumed at selection and they remain only as
    // affine open-extent obligations in the control ledger.
    let semantic_layers = control_layers
        .iter()
        .filter(|layer| oriented_layer_is_pending_semantic(layer))
        .cloned()
        .collect::<Vec<_>>();

    let planned = semantic_layers
        .iter()
        .map(|layer| (layer.checked_invocation_id, layer.checked_frame_id))
        .collect::<Vec<_>>();
    let has_planned = planned.iter().any(|(_, frame)| frame.is_some());
    if has_planned
        && planned
            .iter()
            .any(|(invocation, frame)| invocation.is_none() || frame.is_none())
    {
        let detail = semantic_layers
            .iter()
            .map(|layer| {
                (
                    layer.checked_frame_id,
                    layer.checked_invocation_id,
                    layer.checked_invocation_depth,
                    layer.provenance.0,
                    layer
                        .cases
                        .iter()
                        .map(|case| case.constructor.as_str())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "oriented segment mixes checked and inferred computational frames: {detail:?}; recursive templates: {:?}",
                plan.map(|plan| (
                    plan.recursive_calls
                        .iter()
                        .map(|call| (call.call_template_id, call.declaration.as_str(), call.callee.as_str()))
                        .collect::<Vec<_>>(),
                    plan.computational_ih_calls
                        .iter()
                        .map(|call| (call.call_template_id, call.declaration.as_str(), call.slot_template_id))
                        .collect::<Vec<_>>()
                ))
            ),
        ));
    }

    let (segment_site_id, input_interface, output_interface, semantic_frames) = if has_planned {
        let plan = plan.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "oriented segment has no checked plan metadata",
            )
        })?;
        plan.validate()
            .map_err(|reason| unsupported("OrientedSubcontinuationPlanV1", reason))?;
        for entry in &mut control_ledger {
            if entry.invocation_id.is_none() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked control occurrence has no affine invocation identity",
                ));
            }
            let frame_id = entry.frame_id.expect("all control entries are checked");
            entry.checked_witness = Some(
                plan.frame(frame_id)
                    .expect("checked control entry has a validated plan row")
                    .control_witness
                    .clone(),
            );
        }
        let mut by_id = BTreeMap::<u64, Vec<u64>>::new();
        let mut layers_by_key = BTreeMap::new();
        for layer in semantic_layers {
            let frame_id = layer.checked_frame_id.expect("all frames are checked");
            let invocation_id = layer
                .checked_invocation_id
                .expect("all checked frames have an invocation instance");
            if layers_by_key
                .insert((invocation_id, frame_id), layer)
                .is_some()
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "oriented segment repeats a checked dynamic frame key",
                ));
            }
            by_id.entry(invocation_id).or_default().push(frame_id);
        }
        for frame_ids in by_id.values_mut() {
            frame_ids.sort_by_key(|frame_id| {
                plan.frame(*frame_id)
                    .expect("checked frame exists after plan validation")
                    .semantic_position
            });
            for pair in frame_ids.windows(2) {
                let left = plan.frame(pair[0]).expect("validated frame");
                let right = plan.frame(pair[1]).expect("validated frame");
                if left.segment_site_id != right.segment_site_id {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "invocation-local oriented segment crosses checked prompt regions",
                    ));
                }
                if left.output_interface != right.input_interface {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "invocation-local oriented segment endpoints do not compose",
                    ));
                }
            }
        }

        if px8ds_retired_flat_order_enabled() {
            let mut retired = layers_by_key
                .iter()
                .map(|((invocation_id, frame_id), layer)| {
                    (
                        *invocation_id,
                        plan.frame(*frame_id).expect("validated checked frame"),
                        layer,
                    )
                })
                .collect::<Vec<_>>();
            retired.sort_by_key(|(_, frame, layer)| {
                (
                    std::cmp::Reverse(layer.checked_invocation_depth),
                    frame.semantic_position,
                )
            });
            for pair in retired.windows(2) {
                if pair[0].1.output_interface != pair[1].1.input_interface {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        format!(
                            "retired flat oriented splice answer endpoints do not compose: left=(instance={}, frame={}, depth={}) right=(instance={}, frame={}, depth={})",
                            pair[0].0,
                            pair[0].1.frame_id,
                            pair[0].2.checked_invocation_depth,
                            pair[1].0,
                            pair[1].1.frame_id,
                            pair[1].2.checked_invocation_depth,
                        ),
                    ));
                }
            }
        }

        let mut edges_by_child = BTreeMap::new();
        let mut child_by_parent_frame = BTreeMap::new();
        for edge in dynamic_splice_edges {
            if edge.child_invocation_instance_id == edge.parent_invocation_instance_id {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge forms a self-parent cycle",
                ));
            }
            let child_frames = by_id
                .get(&edge.child_invocation_instance_id)
                .ok_or_else(|| {
                    unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic splice edge names a stale child invocation",
                    )
                })?;
            let parent_frames = by_id.get(&edge.parent_invocation_instance_id);
            if parent_frames.is_some_and(|frames| !frames.contains(&edge.parent_frame_template_id))
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge names the wrong static parent frame",
                ));
            }
            let call = plan
                .computational_ih_call(edge.checked_call_template_id)
                .ok_or_else(|| {
                    unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic splice edge names a stale checked call template",
                    )
                })?;
            if call.parent_frame_template_id != Some(edge.parent_frame_template_id)
                || call.parent_segment_site_id != Some(edge.segment_site_id)
                || call.callee_segment_site_id != edge.segment_site_id
                || call.callee_frame_templates != *child_frames
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge disagrees with its checked static parent",
                ));
            }
            if call.result_interface != call.caller_interface {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice call result does not match its checked caller interface",
                ));
            }
            if edges_by_child
                .insert(edge.child_invocation_instance_id, edge)
                .is_some()
            {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic child invocation carries duplicate affine splice edges",
                ));
            }
        }
        let mut external_children = BTreeMap::new();
        for edge in edges_by_child.values() {
            if by_id.contains_key(&edge.parent_invocation_instance_id) {
                let key = (
                    edge.parent_invocation_instance_id,
                    edge.parent_frame_template_id,
                );
                if child_by_parent_frame
                    .insert(key, edge.child_invocation_instance_id)
                    .is_some()
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "sibling dynamic invocations contend for one affine parent edge",
                    ));
                }
            } else {
                if edge.parent_invocation_instance_id != 0 {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "dynamic splice edge names a stale parent invocation",
                    ));
                }
                if external_children
                    .insert(
                        edge.parent_frame_template_id,
                        edge.child_invocation_instance_id,
                    )
                    .is_some()
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "sibling dynamic invocations compete for one external parent edge",
                    ));
                }
            }
        }
        let roots = if !external_children.is_empty() {
            if edges_by_child.len() != by_id.len() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge deletion leaves an unparented invocation-local segment",
                ));
            }
            let mut roots = external_children.into_iter().collect::<Vec<_>>();
            roots.sort_by_key(|(parent_frame, _)| {
                plan.frame(*parent_frame)
                    .expect("validated external parent frame")
                    .semantic_position
            });
            roots.into_iter().map(|(_, child)| child).collect()
        } else {
            by_id
                .keys()
                .filter(|instance| !edges_by_child.contains_key(instance))
                .copied()
                .collect::<Vec<_>>()
        };
        if roots.is_empty() || (edges_by_child.len() < by_id.len() && roots.len() != 1) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic splice edges do not form one exact invocation-local tree",
            ));
        }
        fn append_invocation_local_segment(
            invocation_id: u64,
            by_id: &BTreeMap<u64, Vec<u64>>,
            child_by_parent_frame: &BTreeMap<(u64, u64), u64>,
            visiting: &mut BTreeSet<u64>,
            completed: &mut BTreeSet<u64>,
            order: &mut Vec<(u64, u64)>,
        ) -> Result<(), CraneliftBackendError> {
            if completed.contains(&invocation_id) {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge is shared across sibling invocation paths",
                ));
            }
            if !visiting.insert(invocation_id) {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edges form a parent cycle",
                ));
            }
            for frame_id in by_id
                .get(&invocation_id)
                .expect("validated invocation-local segment exists")
            {
                if let Some(child) = child_by_parent_frame.get(&(invocation_id, *frame_id)) {
                    append_invocation_local_segment(
                        *child,
                        by_id,
                        child_by_parent_frame,
                        visiting,
                        completed,
                        order,
                    )?;
                }
                order.push((invocation_id, *frame_id));
            }
            visiting.remove(&invocation_id);
            completed.insert(invocation_id);
            Ok(())
        }
        let mut order = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut completed = BTreeSet::new();
        for root in roots {
            append_invocation_local_segment(
                root,
                &by_id,
                &child_by_parent_frame,
                &mut visiting,
                &mut completed,
                &mut order,
            )?;
        }
        if completed.len() != by_id.len() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic splice tree leaves an invocation-local segment unreachable",
            ));
        }
        let mut ordered = order
            .into_iter()
            .map(|key| {
                let layer = layers_by_key
                    .remove(&key)
                    .expect("validated dynamic frame key exists");
                let frame = plan.frame(key.1).expect("validated checked frame exists");
                (key.0, frame, layer)
            })
            .collect::<Vec<_>>();
        let site = ordered
            .first()
            .expect("checked oriented segment is nonempty")
            .1
            .segment_site_id;
        if ordered
            .iter()
            .any(|(_, frame, _)| frame.segment_site_id != site)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "oriented dynamic splice crosses checked prompt regions",
            ));
        }
        let input = ordered.first().unwrap().1.input_interface.clone();
        let output = ordered.last().unwrap().1.output_interface.clone();
        (
            Some(site),
            Some(input),
            Some(output),
            ordered.drain(..).map(|(_, _, layer)| layer).collect(),
        )
    } else {
        (None, None, None, semantic_layers)
    };

    let mut owned = OwnedOrientedSubcontinuationSegment {
        producer_origin,
        sibling_position,
        activation,
        segment_site_id,
        input_interface,
        output_interface,
        semantic_frames,
        control_ledger,
        resume_cursor,
        capability: AffineSpliceCapability {
            state: AffineSpliceState::Open,
        },
    };
    owned.capability.consume()?;
    debug_assert_eq!(owned.capability.state, AffineSpliceState::Consumed);
    debug_assert!(owned.control_ledger.len() >= owned.semantic_frames.len());
    debug_assert_eq!(
        owned.segment_site_id.is_some(),
        owned.input_interface.is_some()
    );
    debug_assert_eq!(
        owned.segment_site_id.is_some(),
        owned.output_interface.is_some()
    );
    Ok(InstalledOrientedSubcontinuationSegment {
        checked: owned.segment_site_id.is_some(),
        producer_origin: owned.producer_origin,
        sibling_position: owned.sibling_position,
        activation: owned.activation,
        semantic_frames: owned.semantic_frames,
        control_ledger: owned.control_ledger,
        resume_cursor: owned.resume_cursor,
    })
}
fn recursor_invocation_is_checked(segment: &RecursorInvocationSegment) -> bool {
    segment.selection.checked_frame_id.is_some()
        || segment
            .unwind
            .later_wrappers_in_construction_order
            .iter()
            .any(|layer| layer.checked_frame_id.is_some())
}
fn installed_oriented_eliminator_frames(
    segment: &InstalledOrientedSubcontinuationSegment,
) -> Vec<EliminatorFrame<'_>> {
    segment
        .semantic_frames
        .iter()
        .map(|layer| {
            EliminatorFrame::Computational(ComputationalEliminatorFrame {
                cases: &layer.cases,
                default: &layer.default,
                env: &layer.outer_env,
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: layer.provenance,
                checked_frame_id: layer.checked_frame_id,
                checked_invocation_id: layer.checked_invocation_id,
                checked_invocation_source: layer.checked_invocation_source,
                checked_invocation_depth: layer.checked_invocation_depth,
            })
        })
        .collect()
}
/// Validate the control shape available at source-machine installation.
/// Parent adjacency is established by the return-hole continuation and belongs
/// only to the flattened-consumer validator below.
fn validate_recursor_invocation_install_shape(
    segment: &RecursorInvocationSegment,
) -> Result<(), CraneliftBackendError> {
    if !matches!(
        segment.selection.role,
        RecursorLayerRole::SelectsOccurrence { origin } if origin == segment.origin
    ) {
        return Err(unsupported(
            "ComputationalRecursor",
            "recursor selection role does not select the invocation origin",
        ));
    }
    let mut scope_origins = BTreeSet::new();
    for layer in &segment.unwind.later_wrappers_in_construction_order {
        let RecursorLayerRole::ExitsScope {
            origin,
            scope_origin,
            ..
        } = layer.role
        else {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        };
        if origin != segment.origin {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        }
        if !scope_origins.insert(scope_origin) {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind repeats a selected scope identity",
            ));
        }
    }
    Ok(())
}
fn validate_recursor_invocation_segment(
    segment: &RecursorInvocationSegment,
) -> Result<(), CraneliftBackendError> {
    if !matches!(
        segment.selection.role,
        RecursorLayerRole::SelectsOccurrence { origin } if origin == segment.origin
    ) {
        return Err(unsupported(
            "ComputationalRecursor",
            "recursor selection role does not select the invocation origin",
        ));
    }
    // Construction order is outer-to-inner, while execution pops the vector
    // inner-to-outer. An outermost scope may name a parent owned by the caller;
    // every carried successor must link to the immediately preceding scope.
    let mut scope_origins = BTreeSet::new();
    let mut previous_scope = None;
    for layer in &segment.unwind.later_wrappers_in_construction_order {
        let RecursorLayerRole::ExitsScope {
            origin,
            scope_origin,
            parent_scope,
        } = layer.role
        else {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        };
        if origin != segment.origin {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind role does not exit the invocation origin",
            ));
        }
        if !scope_origins.insert(scope_origin) {
            return Err(unsupported(
                "ComputationalRecursor",
                "recursor unwind repeats a selected scope identity",
            ));
        }
        if let Some(previous_scope) = previous_scope {
            if parent_scope != Some(previous_scope) {
                return Err(unsupported(
                    "ComputationalRecursor",
                    "recursor unwind has a broken selected-scope parent link",
                ));
            }
        }
        previous_scope = Some(scope_origin);
    }
    Ok(())
}
fn active_recursor_frame<'a>(
    eliminators: &'a [EliminatorFrame<'a>],
) -> Option<&'a ActiveContinuationFrame<'a>> {
    eliminators.iter().find_map(|eliminator| match eliminator {
        EliminatorFrame::Active(frame) => Some(frame),
        EliminatorFrame::Computational(_)
        | EliminatorFrame::Ordinary(_)
        | EliminatorFrame::PendingLet(_)
        | EliminatorFrame::InvocationReturn => None,
    })
}
fn find_continuation_cursor<'a>(
    active: &'a ActiveContinuationFrame<'a>,
    cursor: ContinuationCursorId,
) -> Option<&'a ActiveContinuationFrame<'a>> {
    if active.cursor == cursor {
        Some(active)
    } else {
        active
            .parent
            .and_then(|parent| find_continuation_cursor(parent, cursor))
    }
}
fn active_context_contains_cursor(
    active: &ActiveContinuationFrame<'_>,
    cursor: ContinuationCursorId,
) -> bool {
    find_continuation_cursor(active, cursor).is_some()
        || active.source_selected_cursor == Some(cursor)
        || active.source_lineage.iter().rev().any(|candidate| {
            let candidate = candidate.as_active(active.source_lineage);
            find_continuation_cursor(&candidate, cursor).is_some()
        })
}
#[derive(Clone, Copy)]
enum EliminatorFrame<'a> {
    Computational(ComputationalEliminatorFrame<'a>),
    Ordinary(OrdinaryEliminatorFrame<'a>),
    PendingLet(PendingLetContinuationFrame<'a>),
    InvocationReturn,
    Active(ActiveContinuationFrame<'a>),
}
/// The source-evaluation continuation above a recursive-IH invocation.  This
/// is deliberately distinct from `EliminatorFrame`: source evaluation drains
/// this owned chain before its terminal may resume the outer eliminator cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceComputationalAnswerRoute {
    DirectScrutinee,
    CheckedSelectedRecursor,
}
impl SourceComputationalAnswerRoute {
    fn for_recursor_layer(layer: &ComputationalRecursorLayer) -> Self {
        if layer.checked_frame_id.is_some()
            && matches!(layer.role, RecursorLayerRole::SelectsOccurrence { .. })
        {
            Self::CheckedSelectedRecursor
        } else {
            Self::DirectScrutinee
        }
    }
}
fn source_case_has_no_checked_control_markers(expr: &RuntimeExpr) -> bool {
    let mut frames = BTreeMap::new();
    if collect_checked_subcontinuation_frames(expr, &mut frames).is_err() || !frames.is_empty() {
        return false;
    }
    let mut markers = CheckedOrientedMarkerSets::default();
    collect_checked_oriented_markers(expr, &mut markers, "<source-case>", &mut Vec::new()).is_ok()
        && markers.recursive_calls.is_empty()
        && markers.computational_ih_slots.is_empty()
        && markers.computational_ih_calls.is_empty()
}
enum SourceContinuation<'a> {
    Terminal(SourceContinuationTerminal<'a>),
    CheckedRecursiveInvocationReturn {
        instance: CheckedRecursiveInvocationInstance,
        next: Box<SourceContinuation<'a>>,
    },
    CheckedComputationalIHInvocationReturn {
        call_template_id: u64,
        next: Box<SourceContinuation<'a>>,
    },
    ReturnFromSelectedCase {
        delimiter: SelectedCaseReturnDelimiter,
        next: Box<SourceContinuation<'a>>,
    },
    LetBody {
        body: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ApplyRecursorSelection {
        layer: ComputationalRecursorLayer,
        next: Box<SourceContinuation<'a>>,
    },
    UnwindRecursorSegment {
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        next: Box<SourceContinuation<'a>>,
    },
    IfScrutinee {
        then_expr: RuntimeExpr,
        else_expr: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    MatchScrutinee {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    ComputationalMatchScrutinee {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        checked_frame_id: Option<u64>,
        answer_route: SourceComputationalAnswerRoute,
        next: Box<SourceContinuation<'a>>,
    },
    ProjectRecord {
        field: String,
        next: Box<SourceContinuation<'a>>,
    },
    CallCallee {
        args: Vec<RuntimeExpr>,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
    CallArgument {
        callee: Lowered,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourceContinuation<'a>>,
    },
}
enum SourceContinuationTerminal<'a> {
    ReturnValue,
    /// The unique affine handoff from source evaluation back to the producer.
    /// The stored unwind segment is consumed here; it is not inferred from
    /// provenance or reconstructed from the cursor.
    ReturnToProducerHole {
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        expected: ContinuationCursorId,
        active: &'a ActiveContinuationFrame<'a>,
        root_authority: Option<RootTerminalAnswerAuthority>,
    },
    ResumeOuter {
        expected: ContinuationCursorId,
        active: &'a ActiveContinuationFrame<'a>,
        root_authority: Option<RootTerminalAnswerAuthority>,
    },
    JumpToJoin(SourcePredecessorEdge<'a>),
}
#[derive(Clone)]
struct SourceJoinTarget<'a> {
    join_id: u64,
    block: cranelift_codegen::ir::Block,
    expected_outer: ContinuationCursorId,
    required_kind: ScalarMergeKind,
    terminal_active_prefix: Vec<EliminatorFrame<'a>>,
}
/// An affine capability for one mutually exclusive predecessor of a checked
/// source join. The target description is shareable; this edge deliberately is
/// not `Clone`, so a predecessor can either seal its edge or consume it into a
/// branch fan-out, never replay it.
struct SourcePredecessorEdge<'a> {
    target: SourceJoinTarget<'a>,
    predecessor_identity: u64,
}
/// A cloneable source-evaluation prefix with its terminal edge removed. A
/// branch fan-out may materialize this prefix once per mutually exclusive CFG
/// arm, but the post-cut suffix and executable predecessor edge never live in
/// the template.
#[derive(Clone)]
enum SourcePrefixTemplate {
    Terminal {
        expected_outer: ContinuationCursorId,
    },
    CheckedRecursiveInvocationReturn {
        instance: CheckedRecursiveInvocationInstance,
        next: Box<SourcePrefixTemplate>,
    },
    CheckedComputationalIHInvocationReturn {
        call_template_id: u64,
        next: Box<SourcePrefixTemplate>,
    },
    ReturnFromSelectedCase {
        delimiter: SelectedCaseReturnDelimiter,
        next: Box<SourcePrefixTemplate>,
    },
    LetBody {
        body: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    ApplyRecursorSelection {
        layer: ComputationalRecursorLayer,
        next: Box<SourcePrefixTemplate>,
    },
    UnwindRecursorSegment {
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
        next: Box<SourcePrefixTemplate>,
    },
    IfScrutinee {
        then_expr: RuntimeExpr,
        else_expr: RuntimeExpr,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    MatchScrutinee {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    ComputationalMatchScrutinee {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        checked_frame_id: Option<u64>,
        answer_route: SourceComputationalAnswerRoute,
        next: Box<SourcePrefixTemplate>,
    },
    ProjectRecord {
        field: String,
        next: Box<SourcePrefixTemplate>,
    },
    CallCallee {
        args: Vec<RuntimeExpr>,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
    CallArgument {
        callee: Lowered,
        remaining: Vec<RuntimeExpr>,
        lowered: Vec<Lowered>,
        env: Vec<Lowered>,
        next: Box<SourcePrefixTemplate>,
    },
}
enum SourcePrefixTerminal<'a> {
    ResumeOuter {
        root_authority: Option<RootTerminalAnswerAuthority>,
    },
    Join(SourcePredecessorEdge<'a>),
}
struct SourceBranchFanout<'a> {
    source_prefix_template: SourcePrefixTemplate,
    inherited_edge: SourcePredecessorEdge<'a>,
}
struct ArmedInvocation<'a> {
    suspended: SourceControl<'a>,
    expected_selected: ContinuationCursorId,
}
#[derive(Clone)]
struct SourceSelectedContinuation<'a> {
    activation: ContinuationActivationId,
    cursor: ContinuationCursorId,
    parent: Option<&'a ActiveContinuationFrame<'a>>,
    pending: Vec<EliminatorFrame<'a>>,
    selected_ancestry: Vec<RecursorFrameProvenance>,
    selected_scope: Option<OwnedSelectedScope>,
}
impl<'a> SourceSelectedContinuation<'a> {
    fn as_active<'b>(
        &'b self,
        source_lineage: &'b [SourceSelectedContinuation<'a>],
    ) -> ActiveContinuationFrame<'b>
    where
        'a: 'b,
    {
        ActiveContinuationFrame {
            activation: self.activation,
            cursor: self.cursor,
            parent: self.parent,
            pending: &self.pending,
            selected_ancestry: &self.selected_ancestry,
            source_lineage,
            source_selected_cursor: Some(self.cursor),
            selected_scope: self.selected_scope.as_ref(),
        }
    }
}
fn source_active_cursor<'a: 'b, 'b>(
    selected: &'b SourceSelectedContinuation<'a>,
    lineage: &'b [SourceSelectedContinuation<'a>],
    cursor: ContinuationCursorId,
) -> Option<ActiveContinuationFrame<'b>> {
    std::iter::once(selected)
        .chain(lineage.iter().rev())
        .find_map(|candidate| {
            let mut active = candidate.as_active(lineage);
            active.source_selected_cursor = Some(selected.cursor);
            if active.cursor == cursor {
                Some(active)
            } else {
                let mut parent = active.parent;
                while let Some(frame) = parent {
                    if frame.cursor == cursor {
                        let mut frame = *frame;
                        frame.source_lineage = lineage;
                        frame.source_selected_cursor = Some(selected.cursor);
                        return Some(frame);
                    }
                    parent = frame.parent;
                }
                None
            }
        })
}
struct SourceControl<'a> {
    continuation: SourceContinuation<'a>,
    selected: SourceSelectedContinuation<'a>,
    selected_lineage: Vec<SourceSelectedContinuation<'a>>,
    terminal_outer: ContinuationCursorId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SelectedCaseReturnDelimiter {
    activation: ContinuationActivationId,
    cursor: ContinuationCursorId,
    scope_origin: RecursorProducerOriginId,
    frame_id: Option<u64>,
    invocation_id: Option<u64>,
}
enum SourceMachineState<'a> {
    Eval {
        expr: RuntimeExpr,
        env: Vec<Lowered>,
        control: SourceControl<'a>,
    },
    Value {
        value: Lowered,
        control: SourceControl<'a>,
    },
}
enum SourceCallOutcome<'a> {
    Continue(SourceMachineState<'a>),
    Complete(Lowered),
}
#[derive(Clone, Copy)]
enum DynamicConstructorContinuation<'a> {
    Ordinary {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
        env: &'a [Lowered],
    },
    Producer {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
        env: &'a [Lowered],
        eliminators: &'a [EliminatorFrame<'a>],
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScalarMergeKind {
    Int,
    Bool,
    StructuralNat,
    ExitCode,
    RecursiveBackedge,
}
/// Proof token for the legacy closed-expression merge sites. It can only be
/// minted when source evaluation has no live continuation. Checked source joins
/// use their explicit `SourceJoinTarget.required_kind` instead.
/// Move-only proof that the native lowering machine has reached the checked
/// invocation root with no semantic or control continuation left to consume
/// the value.
struct RootTerminalAnswerAuthority {
    site_id: u64,
    checked_result_type_fingerprint: u64,
    occurrence_binding_fingerprint: u64,
    outer_cursor: Option<ContinuationCursorId>,
}
struct TerminalAnswerAuthority;
struct DeferredConstructorCaseEnvironment<'a> {
    constructor: &'a str,
    lowered_prefix: &'a [Lowered],
    selected_field: usize,
    trailing_fields: &'a [RuntimeExpr],
    producer_env: &'a [Lowered],
    outer_eliminator: EliminatorFrame<'a>,
    splice_caller: Option<&'a ActiveContinuationFrame<'a>>,
    selected_active: ActiveContinuationFrame<'a>,
}
#[derive(Clone, Copy)]
enum ImmediateBinderEliminator<'a> {
    Computational {
        cases: &'a [crate::RuntimeComputationalMatchCase],
        default: &'a RuntimeTrap,
    },
    Ordinary {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
    },
}
fn immediate_binder_eliminator(
    body: &RuntimeExpr,
    argument_binder_offset: usize,
    argument_binders: usize,
) -> Option<(usize, ImmediateBinderEliminator<'_>)> {
    let (scrutinee, eliminator) = match body {
        RuntimeExpr::ComputationalMatch {
            scrutinee,
            cases,
            default,
        } => (
            scrutinee.as_ref(),
            ImmediateBinderEliminator::Computational { cases, default },
        ),
        RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } => (
            scrutinee.as_ref(),
            ImmediateBinderEliminator::Ordinary { cases, default },
        ),
        _ => return None,
    };
    let RuntimeExpr::Var(index) = scrutinee else {
        return None;
    };
    let index = usize::try_from(*index).ok()?;
    let field = index.checked_sub(argument_binder_offset)?;
    (field < argument_binders).then_some((field, eliminator))
}
fn ordinary_match_continuation<'a>(
    params: &[String],
    body: &'a RuntimeExpr,
) -> Option<(&'a [crate::RuntimeMatchCase], &'a RuntimeTrap)> {
    if params.len() != 1 {
        return None;
    }
    let RuntimeExpr::Match {
        scrutinee,
        cases,
        default,
    } = body
    else {
        return None;
    };
    matches!(scrutinee.as_ref(), RuntimeExpr::Var(0)).then_some((cases, default))
}
fn requires_heterogeneous_deforestation(expr: &RuntimeExpr) -> bool {
    matches!(
        expr,
        RuntimeExpr::Match { .. }
            | RuntimeExpr::ComputationalMatch { .. }
            | RuntimeExpr::If { .. }
            | RuntimeExpr::Call { .. }
    ) && produces_deforestable_aggregate_with_ih(expr, &BTreeSet::new())
}
fn reaches_environment_computational_recursor(
    expr: &RuntimeExpr,
    env: &[Lowered],
    introduced_binders: usize,
) -> bool {
    let recursive_hypotheses = env
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            matches!(value, Lowered::ComputationalRecursorClosure { .. })
                .then_some(index + introduced_binders)
        })
        .collect();
    produces_deforestable_aggregate_with_ih(expr, &recursive_hypotheses)
        && !produces_deforestable_aggregate_with_ih(expr, &BTreeSet::new())
}
fn shifted_aggregate_ihs(aggregate_ihs: &BTreeSet<usize>, by: usize) -> BTreeSet<usize> {
    aggregate_ihs.iter().map(|index| index + by).collect()
}
fn produces_deforestable_aggregate_with_ih(
    expr: &RuntimeExpr,
    aggregate_ihs: &BTreeSet<usize>,
) -> bool {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. } => {
            produces_deforestable_aggregate_with_ih(body, aggregate_ihs)
        }
        RuntimeExpr::Construct { .. } => true,
        RuntimeExpr::Let { body, .. } => {
            produces_deforestable_aggregate_with_ih(body, &shifted_aggregate_ihs(aggregate_ihs, 1))
        }
        RuntimeExpr::Match { cases, .. } => {
            !cases.is_empty()
                && cases.iter().all(|case| {
                    produces_deforestable_aggregate_with_ih(
                        &case.body,
                        &shifted_aggregate_ihs(aggregate_ihs, case.binders),
                    )
                })
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            !cases.is_empty()
                && cases.iter().all(|case| {
                    let mut case_ihs = (0..case.recursive_positions.len()).collect::<BTreeSet<_>>();
                    case_ihs.extend(aggregate_ihs.iter().map(|index| {
                        index + case.recursive_positions.len() + case.argument_binders
                    }));
                    produces_deforestable_aggregate_with_ih(&case.body, &case_ihs)
                })
        }
        RuntimeExpr::If {
            then_expr,
            else_expr,
            ..
        } => {
            produces_deforestable_aggregate_with_ih(then_expr, aggregate_ihs)
                && produces_deforestable_aggregate_with_ih(else_expr, aggregate_ihs)
        }
        RuntimeExpr::Call { callee, .. } => {
            if let RuntimeExpr::Var(index) = callee.as_ref() {
                return usize::try_from(*index).is_ok_and(|index| aggregate_ihs.contains(&index));
            }
            match callee.as_ref() {
                RuntimeExpr::Closure {
                    captures,
                    params,
                    body,
                } => produces_deforestable_aggregate_with_ih(
                    body,
                    &shifted_aggregate_ihs(aggregate_ihs, params.len() + captures.len()),
                ),
                RuntimeExpr::LexicalClosure {
                    captures,
                    params,
                    body,
                } => produces_deforestable_aggregate_with_ih(
                    body,
                    &shifted_aggregate_ihs(aggregate_ihs, params.len() + captures.len()),
                ),
                _ => false,
            }
        }
        _ => false,
    }
}
fn produces_recursive_deforestable_aggregate(expr: &RuntimeExpr, symbol: &str) -> bool {
    match expr {
        RuntimeExpr::Construct { .. } => true,
        RuntimeExpr::Let { body, .. } => produces_recursive_deforestable_aggregate(body, symbol),
        RuntimeExpr::Match { cases, .. } => {
            !cases.is_empty()
                && cases
                    .iter()
                    .all(|case| produces_recursive_deforestable_aggregate(&case.body, symbol))
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            !cases.is_empty()
                && cases
                    .iter()
                    .all(|case| produces_recursive_deforestable_aggregate(&case.body, symbol))
        }
        RuntimeExpr::If {
            then_expr,
            else_expr,
            ..
        } => {
            produces_recursive_deforestable_aggregate(then_expr, symbol)
                && produces_recursive_deforestable_aggregate(else_expr, symbol)
        }
        RuntimeExpr::Call { callee, .. } => {
            matches!(callee.as_ref(), RuntimeExpr::DeclarationRef { symbol: callee } if callee == symbol)
        }
        _ => false,
    }
}
fn collect_runtime_declaration_refs(expr: &RuntimeExpr, output: &mut BTreeSet<RuntimeSymbol>) {
    match expr {
        RuntimeExpr::CheckedJoinSite { body, .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
        | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
        | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
            collect_runtime_declaration_refs(body, output)
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            output.insert(symbol.clone());
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Let { value, body } => {
            collect_runtime_declaration_refs(value, output);
            collect_runtime_declaration_refs(body, output);
        }
        RuntimeExpr::If {
            scrutinee,
            then_expr,
            else_expr,
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            collect_runtime_declaration_refs(then_expr, output);
            collect_runtime_declaration_refs(else_expr, output);
        }
        RuntimeExpr::Match {
            scrutinee, cases, ..
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            for case in cases {
                collect_runtime_declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            collect_runtime_declaration_refs(scrutinee, output);
            for case in cases {
                collect_runtime_declaration_refs(&case.body, output);
            }
        }
        RuntimeExpr::Record { fields } => {
            for (_, field) in fields {
                collect_runtime_declaration_refs(field, output);
            }
        }
        RuntimeExpr::Project { record, .. }
        | RuntimeExpr::Closure { body: record, .. }
        | RuntimeExpr::LexicalClosure { body: record, .. } => {
            collect_runtime_declaration_refs(record, output);
        }
        RuntimeExpr::Call { callee, args } => {
            collect_runtime_declaration_refs(callee, output);
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            if let Some(capability) = capability {
                collect_runtime_declaration_refs(&capability.value, output);
            }
            for arg in args {
                collect_runtime_declaration_refs(arg, output);
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
}
fn select_ordinary_case<'a>(
    eliminator: OrdinaryEliminatorFrame<'a>,
    constructor: &str,
) -> Result<&'a crate::RuntimeMatchCase, RuntimeTrap> {
    eliminator
        .cases
        .iter()
        .find(|case| case.constructor == constructor)
        .ok_or_else(|| eliminator.default.clone())
}
fn select_computational_case<'frames, 'data>(
    eliminators: &'frames [ComputationalEliminatorFrame<'data>],
    constructor: &str,
) -> Result<
    (
        &'data crate::RuntimeComputationalMatchCase,
        &'frames [ComputationalEliminatorFrame<'data>],
    ),
    RuntimeTrap,
> {
    let Some(eliminator) = eliminators.first() else {
        return Err(RuntimeTrap {
            code: RuntimeTrapCode::UnsupportedErasure,
            message: "nested computational producer has no eliminator".to_string(),
        });
    };
    eliminator
        .cases
        .iter()
        .find(|case| case.constructor == constructor)
        .map(|case| (case, &eliminators[1..]))
        .ok_or_else(|| eliminator.default.clone())
}
impl<'a> Lowering<'a> {
    fn mint_recursor_producer_origin(&mut self) -> RecursorProducerOriginId {
        let origin = RecursorProducerOriginId(self.next_recursor_producer_origin);
        self.next_recursor_producer_origin = self
            .next_recursor_producer_origin
            .checked_add(1)
            .expect("compiler-private recursor producer origin exhausted");
        origin
    }

    fn mint_recursor_frame_provenance(&mut self) -> RecursorFrameProvenance {
        let provenance = RecursorFrameProvenance(self.next_recursor_frame_provenance);
        self.next_recursor_frame_provenance = self
            .next_recursor_frame_provenance
            .checked_add(1)
            .expect("compiler-private recursor provenance exhausted");
        provenance
    }

    fn mint_continuation_activation(&mut self) -> ContinuationActivationId {
        let activation = ContinuationActivationId(self.next_continuation_activation);
        self.next_continuation_activation = self
            .next_continuation_activation
            .checked_add(1)
            .expect("compiler-private continuation activation exhausted");
        activation
    }

    fn mint_continuation_cursor(&mut self) -> ContinuationCursorId {
        let cursor = ContinuationCursorId(self.next_continuation_cursor);
        self.next_continuation_cursor = self
            .next_continuation_cursor
            .checked_add(1)
            .expect("compiler-private continuation cursor exhausted");
        cursor
    }

    fn enter_checked_subcontinuation_frame(
        &mut self,
        frame_id: u64,
    ) -> Result<(), CraneliftBackendError> {
        if self
            .active_subcontinuation_frame
            .replace(frame_id)
            .is_some()
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "nested checked subcontinuation occurrence marker",
            ));
        }
        Ok(())
    }

    fn enter_checked_recursive_invocation(
        &mut self,
        call_template_id: u64,
        body: &RuntimeExpr,
    ) -> Result<CheckedRecursiveInvocationInstance, CraneliftBackendError> {
        if self.pending_recursive_call.is_some() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "nested unchecked recursive invocation marker",
            ));
        }
        let call = self
            .oriented_subcontinuation_plan
            .as_ref()
            .and_then(|plan| plan.recursive_call(call_template_id))
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "recursive invocation marker has no checked call template",
                )
            })?;
        let RuntimeExpr::Call { callee, args } = body else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker does not wrap one complete call",
            ));
        };
        if !matches!(callee.as_ref(), RuntimeExpr::DeclarationRef { symbol } if symbol == &call.callee)
            || args.len() as u64 != call.arity
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker callee or arity is stale",
            ));
        }
        if !self
            .consumed_recursive_call_templates
            .insert(call_template_id)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation occurrence marker was consumed twice",
            ));
        }
        let instance = CheckedRecursiveInvocationInstance {
            source: InvocationTemplateRef::SameSccCall(call_template_id),
            invocation_instance_id: self.next_recursive_invocation_instance,
            semantic_depth: self.active_recursive_invocations.len() + 1,
            dynamic_splice_edge: None,
        };
        self.next_recursive_invocation_instance = self
            .next_recursive_invocation_instance
            .checked_add(1)
            .expect("compiler-private recursive invocation identity exhausted");
        self.pending_recursive_call = Some(instance);
        self.active_recursive_invocations.push(instance);
        Ok(instance)
    }

    fn leave_checked_recursive_invocation(
        &mut self,
        instance: CheckedRecursiveInvocationInstance,
    ) -> Result<(), CraneliftBackendError> {
        if self.pending_recursive_call == Some(instance) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker was not consumed by its call",
            ));
        }
        if self.active_recursive_invocations.pop() != Some(instance) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation instance stack is not affine",
            ));
        }
        Ok(())
    }

    fn enter_checked_computational_ih_invocation(
        &mut self,
        call_template_id: u64,
    ) -> Result<(), CraneliftBackendError> {
        if self
            .pending_computational_ih_call
            .replace(call_template_id)
            .is_some()
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "nested computational IH invocation marker",
            ));
        }
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation marker has no checked plan",
            )
        })?;
        plan.computational_ih_call(call_template_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH invocation marker has no checked call template",
                )
            })?;
        Ok(())
    }

    fn mint_checked_computational_ih_instance(
        &mut self,
        value: &mut Lowered,
    ) -> Result<Option<CheckedRecursiveInvocationInstance>, CraneliftBackendError> {
        let Some(call_template_id) = self.pending_computational_ih_call.take() else {
            return Ok(None);
        };
        let Lowered::ComputationalRecursorClosure { invocation, .. } = value else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH marker was applied to an ordinary value",
            ));
        };
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked plan",
            )
        })?;
        let call = plan
            .computational_ih_call(call_template_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH invocation has no checked call template",
                )
            })?;
        if invocation.computational_ih_slot_template_id != Some(call.slot_template_id) {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation marker names a different slot",
            ));
        }
        let parent_frame_template_id = call.parent_frame_template_id.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked static parent",
            )
        })?;
        let segment_site_id = call.parent_segment_site_id.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked parent segment",
            )
        })?;
        let mut parents = std::iter::once(&invocation.selection)
            .chain(
                invocation
                    .unwind
                    .later_wrappers_in_construction_order
                    .iter(),
            )
            .filter(|layer| {
                layer.semantic_pending && layer.checked_frame_id == Some(parent_frame_template_id)
            });
        let selected = parents.next().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH closure has no exact checked open parent occurrence",
            )
        })?;
        if parents.next().is_some() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH closure has multiple candidate dynamic parent occurrences",
            ));
        }
        let parent_invocation_instance_id = match selected.checked_invocation_id {
            Some(instance_id) => instance_id,
            None if selected.checked_invocation_source.is_none() => 0,
            None => {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    format!(
                        "computational IH closure-selected occurrence has no dynamic parent identity: frame={:?} source={:?} depth={} handles={:?}",
                        selected.checked_frame_id,
                        selected.checked_invocation_source,
                        selected.checked_invocation_depth,
                        invocation.dynamic_splice_edges,
                    ),
                ))
            }
        };
        let selected_site = plan
            .frame(parent_frame_template_id)
            .map(|frame| frame.segment_site_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH closure-selected occurrence names a stale parent frame",
                )
            })?;
        if selected_site != segment_site_id {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH closure-selected occurrence crosses its checked segment",
            ));
        }
        let edge_id = DynamicSpliceEdgeId(self.next_dynamic_splice_edge);
        self.next_dynamic_splice_edge = self
            .next_dynamic_splice_edge
            .checked_add(1)
            .expect("compiler-private dynamic splice edge identity exhausted");
        let instance = CheckedRecursiveInvocationInstance {
            source: InvocationTemplateRef::ComputationalIHCall(call_template_id),
            invocation_instance_id: self.next_recursive_invocation_instance,
            semantic_depth: self.active_recursive_invocations.len() + 1,
            dynamic_splice_edge: Some(edge_id),
        };
        self.next_recursive_invocation_instance = self
            .next_recursive_invocation_instance
            .checked_add(1)
            .expect("compiler-private invocation identity exhausted");
        if self
            .dynamic_splice_edges
            .insert(
                edge_id,
                DynamicSpliceEdge {
                    edge_id,
                    child_invocation_instance_id: instance.invocation_instance_id,
                    parent_invocation_instance_id,
                    checked_call_template_id: call_template_id,
                    parent_frame_template_id,
                    segment_site_id,
                },
            )
            .is_some()
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "dynamic splice edge identity was minted twice",
            ));
        }
        invocation.dynamic_splice_edges.push(edge_id);
        Ok(Some(instance))
    }

    fn validate_source_dynamic_splice_parent(
        &self,
        instance: CheckedRecursiveInvocationInstance,
        open: &OwnedSelectedScope,
    ) -> Result<(), CraneliftBackendError> {
        let edge_id = instance.dynamic_splice_edge.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "source IH invocation has no affine dynamic splice edge",
            )
        })?;
        let edge = self.dynamic_splice_edges.get(&edge_id).ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "source IH invocation names a deleted or already-consumed dynamic splice edge",
            )
        })?;
        if edge.child_invocation_instance_id != instance.invocation_instance_id
            || edge.parent_invocation_instance_id != open.frame.checked_invocation_id.unwrap_or(0)
            || Some(edge.parent_frame_template_id) != open.frame.checked_frame_id
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "source open occurrence disagrees with the closure-selected dynamic parent",
            ));
        }
        Ok(())
    }

    fn take_dynamic_splice_edges(
        &mut self,
        segment: &RecursorInvocationSegment,
    ) -> Result<Vec<DynamicSpliceEdge>, CraneliftBackendError> {
        let mut seen = BTreeSet::new();
        let mut edges = Vec::with_capacity(segment.dynamic_splice_edges.len());
        for edge_id in &segment.dynamic_splice_edges {
            if !seen.insert(*edge_id) {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge handle is duplicated in one invocation carrier",
                ));
            }
            let edge = self.dynamic_splice_edges.remove(edge_id).ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge was deleted, replayed, or consumed by a sibling",
                )
            })?;
            if edge.edge_id != *edge_id {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "dynamic splice edge ledger identity is stale",
                ));
            }
            edges.push(edge);
        }
        Ok(edges)
    }

    fn finish_checked_computational_ih_marker(
        &mut self,
        mut value: Lowered,
    ) -> Result<Lowered, CraneliftBackendError> {
        let Some(instance) = self.mint_checked_computational_ih_instance(&mut value)? else {
            return Ok(value);
        };
        let Lowered::ComputationalRecursorClosure { invocation, .. } = &mut value else {
            unreachable!("IH instance mint validates one recursor closure")
        };
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation has no checked plan",
            )
        })?;
        // Qualify the exact reusable template sequence at marker consumption.
        // Existing child-qualified layers remain untouched when later parent
        // wrappers are added to the same flattened carrier.
        instantiate_checked_invocation_segment(plan, instance, invocation)?;
        Ok(value)
    }

    fn consume_checked_recursive_invocation_call(
        &mut self,
        symbol: &RuntimeSymbol,
    ) -> Result<Option<CheckedRecursiveInvocationInstance>, CraneliftBackendError> {
        let Some(instance) = self.pending_recursive_call.take() else {
            return Ok(None);
        };
        let InvocationTemplateRef::SameSccCall(call_template_id) = instance.source else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "same-SCC call consumer received a computational IH invocation",
            ));
        };
        let call = self
            .oriented_subcontinuation_plan
            .as_ref()
            .and_then(|plan| plan.recursive_call(call_template_id))
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "active recursive invocation has no checked template",
                )
            })?;
        if &call.callee != symbol {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "recursive invocation marker was transplanted to another callee",
            ));
        }
        Ok(Some(instance))
    }

    fn consume_checked_subcontinuation_frame(
        &mut self,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
    ) -> Result<Option<u64>, CraneliftBackendError> {
        let Some(frame_id) = self.active_subcontinuation_frame.take() else {
            return Ok(None);
        };
        let frame = self
            .oriented_subcontinuation_plan
            .as_ref()
            .and_then(|plan| plan.frame(frame_id))
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked Runtime marker has no transported frame entry",
                )
            })?;
        if frame.runtime_frame_fingerprint
            != crate::compiler_private_computational_match_frame_fingerprint(cases, default)
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked Runtime marker no longer denotes its planned frame",
            ));
        }
        let invocation_id = self
            .active_recursive_invocations
            .last()
            .map_or(0, |instance| instance.invocation_instance_id);
        if !self
            .consumed_subcontinuation_frames
            .insert((invocation_id, frame_id))
        {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "checked Runtime frame marker was consumed more than once",
            ));
        }
        Ok(Some(frame_id))
    }

    fn computational_ih_slots_for_case(
        &self,
        case: &crate::RuntimeComputationalMatchCase,
        checked_frame_id: Option<u64>,
    ) -> Result<Vec<Option<u64>>, CraneliftBackendError> {
        let RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids, ..
        } = &case.body
        else {
            if checked_frame_id.is_some() && !case.recursive_positions.is_empty() {
                return Err(unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "checked computational case is missing its IH slot marker",
                ));
            }
            return Ok(vec![None; case.recursive_positions.len()]);
        };
        let frame_id = checked_frame_id.ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH slot marker is detached from its checked frame",
            )
        })?;
        if slot_template_ids.len() != case.recursive_positions.len() {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH slot marker is not bijective with recursive positions",
            ));
        }
        let plan = self.oriented_subcontinuation_plan.as_ref().ok_or_else(|| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH slot marker has no checked plan",
            )
        })?;
        let mut seen = BTreeSet::new();
        slot_template_ids
            .iter()
            .copied()
            .zip(case.recursive_positions.iter().copied())
            .map(|(slot_template_id, recursive_position)| {
                if !seen.insert(slot_template_id) {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "computational IH case repeats a checked slot template",
                    ));
                }
                let slot = plan
                    .computational_ih_slot(slot_template_id)
                    .ok_or_else(|| {
                        unsupported(
                            "OrientedSubcontinuationPlanV1",
                            "computational IH case names a stale slot template",
                        )
                    })?;
                if slot.frame_template_id != frame_id
                    || slot.constructor != case.constructor
                    || slot.recursive_position != recursive_position as u64
                {
                    return Err(unsupported(
                        "OrientedSubcontinuationPlanV1",
                        "computational IH slot constructor/position/frame binding is stale",
                    ));
                }
                Ok(Some(slot_template_id))
            })
            .collect()
    }

    fn enter_oriented_semantic_region(&mut self, checked: bool) {
        if checked {
            self.active_oriented_semantic_regions = self
                .active_oriented_semantic_regions
                .checked_add(1)
                .expect("compiler-private oriented segment depth exhausted");
        }
    }

    fn leave_oriented_semantic_region(&mut self, checked: bool) {
        if checked {
            self.active_oriented_semantic_regions = self
                .active_oriented_semantic_regions
                .checked_sub(1)
                .expect("oriented semantic region must be entered exactly once");
        }
    }

    fn make_computational_recursor(
        &mut self,
        recursive: Lowered,
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        outer_env: Vec<Lowered>,
        provenance: RecursorFrameProvenance,
        checked_frame_id: Option<u64>,
        computational_ih_slot_template_id: Option<u64>,
        origin: RecursorProducerOriginId,
        sibling_position: usize,
        role: RecursorLayerRole,
        activation: ContinuationActivationId,
        resume_cursor: ContinuationCursorId,
        splice_caller: Option<&ActiveContinuationFrame<'_>>,
        source_control: Option<(
            &SourceSelectedContinuation<'_>,
            &[SourceSelectedContinuation<'_>],
        )>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (residual, payload) = decompose_computational_recursor(recursive);
        let active_instance = self.active_recursive_invocations.last().copied();
        let inferred_frame_id = if checked_frame_id.is_none() {
            active_instance.and_then(|instance| {
                let fingerprint =
                    crate::compiler_private_computational_match_frame_fingerprint(&cases, &default);
                self.oriented_subcontinuation_plan
                    .as_ref()
                    .and_then(|plan| checked_invocation_frame_templates(plan, instance.source).ok())
                    .and_then(|frame_templates| {
                        frame_templates.iter().copied().find(|frame_id| {
                            self.oriented_subcontinuation_plan
                                .as_ref()
                                .and_then(|plan| plan.frame(*frame_id))
                                .is_some_and(|frame| frame.runtime_frame_fingerprint == fingerprint)
                        })
                    })
            })
        } else {
            checked_frame_id
        };
        let invocation_id = inferred_frame_id
            .and_then(|_| active_instance.map(|instance| instance.invocation_instance_id));
        let invocation_source = active_instance.map(|instance| instance.source);
        let invocation_depth = active_instance.map_or(0, |instance| instance.semantic_depth);
        let mut current_layer = ComputationalRecursorLayer {
            cases,
            default,
            outer_env,
            provenance,
            role,
            checked_frame_id: inferred_frame_id,
            checked_invocation_id: invocation_id,
            checked_invocation_source: invocation_source,
            checked_invocation_depth: invocation_depth,
            semantic_pending: true,
        };
        let segment_origin = payload
            .as_ref()
            .map(|(_, invocation)| invocation.origin)
            .unwrap_or(origin);
        let segment_sibling_position = payload
            .as_ref()
            .map(|(_, invocation)| invocation.sibling_position)
            .unwrap_or(sibling_position);
        let segment_checked_invocation = payload
            .as_ref()
            .and_then(|(_, invocation)| invocation.checked_invocation)
            .or(active_instance);
        let segment_dynamic_splice_edges = payload
            .as_ref()
            .map(|(_, invocation)| invocation.dynamic_splice_edges.clone())
            .unwrap_or_default();
        let (selection, unwind) =
            if let Some((_, invocation)) = payload {
                let splice_caller = splice_caller.ok_or_else(|| {
                    unsupported(
                        "ComputationalRecursor",
                        "recursive payload splice has no active continuation",
                    )
                })?;
                let source_cursor_is_live = source_control.is_some_and(|(selected, lineage)| {
                    source_active_cursor(selected, lineage, invocation.resume_cursor).is_some()
                });
                if !active_context_contains_cursor(splice_caller, invocation.resume_cursor)
                    && !source_cursor_is_live
                    && !recursor_invocation_is_checked(&invocation)
                {
                    return Err(unsupported(
                        "ComputationalRecursor",
                        "recursive payload resume cursor is not active",
                    ));
                }
                let mut unwind = invocation.unwind;
                let parent_scope = unwind.later_wrappers_in_construction_order.last().and_then(
                    |layer| match layer.role {
                        RecursorLayerRole::ExitsScope { scope_origin, .. } => Some(scope_origin),
                        RecursorLayerRole::SelectsOccurrence { .. } => None,
                    },
                );
                let unwind_role = match role {
                    RecursorLayerRole::SelectsOccurrence { origin: _ } => {
                        RecursorLayerRole::ExitsScope {
                            origin: segment_origin,
                            scope_origin: origin,
                            parent_scope,
                        }
                    }
                    RecursorLayerRole::ExitsScope {
                        origin: _,
                        scope_origin,
                        parent_scope,
                    } => RecursorLayerRole::ExitsScope {
                        origin: segment_origin,
                        scope_origin,
                        parent_scope,
                    },
                };
                current_layer.role = unwind_role;
                unwind
                    .later_wrappers_in_construction_order
                    .push(current_layer);
                if let Some((selected, lineage)) = source_control {
                    if selected.selected_scope.is_none() {
                        return Err(unsupported(
                            "ComputationalRecursor",
                            "source recursor invocation is missing its owned selected scope",
                        ));
                    }
                    for scope in lineage
                        .iter()
                        .filter_map(|selected| selected.selected_scope.as_ref())
                        .chain(selected.selected_scope.iter())
                    {
                        if unwind
                            .later_wrappers_in_construction_order
                            .iter()
                            .any(|layer| {
                                matches!(
                                    layer.role,
                                    RecursorLayerRole::ExitsScope { scope_origin, .. }
                                        if scope_origin == scope.scope_origin
                                )
                            })
                        {
                            continue;
                        }
                        unwind.later_wrappers_in_construction_order.push(
                            ComputationalRecursorLayer {
                                cases: scope.frame.cases.clone(),
                                default: scope.frame.default.clone(),
                                outer_env: scope.frame.outer_env.clone(),
                                provenance: scope.frame.provenance,
                                checked_frame_id: scope.frame.checked_frame_id,
                                checked_invocation_id: scope.frame.checked_invocation_id,
                                checked_invocation_source: scope.frame.checked_invocation_source,
                                checked_invocation_depth: scope.frame.checked_invocation_depth,
                                semantic_pending: false,
                                role: RecursorLayerRole::ExitsScope {
                                    origin: segment_origin,
                                    scope_origin: scope.scope_origin,
                                    parent_scope: scope.parent_scope,
                                },
                            },
                        );
                    }
                }
                (invocation.selection, unwind)
            } else {
                (
                    current_layer,
                    RecursorUnwindStack {
                        later_wrappers_in_construction_order: Vec::new(),
                    },
                )
            };
        let mut invocation = RecursorInvocationSegment::new(
            segment_origin,
            segment_sibling_position,
            selection,
            unwind,
            resume_cursor,
            segment_checked_invocation,
            computational_ih_slot_template_id,
        );
        invocation.dynamic_splice_edges = segment_dynamic_splice_edges;
        Ok(Lowered::ComputationalRecursorClosure {
            residual: Box::new(residual),
            activation,
            invocation,
        })
    }

    fn merge_branch_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: Lowered,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, bool), CraneliftBackendError> {
        let checked_root_exit_representation = self.has_checked_root_exit_representation();
        let lowered = if checked_root_exit_representation {
            Self::unwrap_terminal_ret(lowered)
        } else {
            lowered
        };
        let zero_tag = builder.ins().iconst(types::I64, 0);
        match lowered {
            Lowered::Int { value, known } => Ok((
                NativeScalarPairV1 {
                    tag: self.native_int_tag(builder, value, known)?,
                    payload: value,
                },
                false,
            )),
            Lowered::ProcessExitStatus { value } => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: value,
                },
                true,
            )),
            lowered if checked_root_exit_representation => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: self.emit_process_exit_status(builder, lowered),
                },
                true,
            )),
            _ => Err(unsupported(
                construct,
                "dynamic native arms must produce scalar Int values",
            )),
        }
    }

    fn merge_scalar_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: Lowered,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        let checked_root_exit_representation = self.has_checked_root_exit_representation();
        let lowered = if checked_root_exit_representation {
            Self::unwrap_terminal_ret(lowered)
        } else {
            lowered
        };
        let zero_tag = builder.ins().iconst(types::I64, 0);
        match lowered {
            Lowered::RecursiveBackedge => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: builder.ins().iconst(types::I64, 0),
                },
                ScalarMergeKind::RecursiveBackedge,
            )),
            Lowered::Int { value, known } => Ok((
                NativeScalarPairV1 {
                    tag: self.native_int_tag(builder, value, known)?,
                    payload: value,
                },
                ScalarMergeKind::Int,
            )),
            Lowered::Bool { value, .. } => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: value,
                },
                ScalarMergeKind::Bool,
            )),
            Lowered::StructuralNat(nat) => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: nat.value,
                },
                ScalarMergeKind::StructuralNat,
            )),
            Lowered::Constructor { constructor, args }
                if args.is_empty()
                    && (constructor == self.process_symbols.bool_true
                        || constructor == self.process_symbols.bool_false) =>
            {
                Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: builder.ins().iconst(
                            types::I64,
                            i64::from(constructor == self.process_symbols.bool_true),
                        ),
                    },
                    ScalarMergeKind::Bool,
                ))
            }
            Lowered::ProcessExitStatus { value } => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: value,
                },
                ScalarMergeKind::ExitCode,
            )),
            lowered if checked_root_exit_representation => Ok((
                NativeScalarPairV1 {
                    tag: zero_tag,
                    payload: self.emit_process_exit_status(builder, lowered),
                },
                ScalarMergeKind::ExitCode,
            )),
            _ => Err(unsupported(
                construct,
                "dynamic arms must produce scalar Int or Bool values",
            )),
        }
    }

    fn restore_root_terminal_authority(
        &mut self,
        authority: Option<RootTerminalAnswerAuthority>,
        expected_outer: ContinuationCursorId,
    ) -> Result<(), CraneliftBackendError> {
        let Some(mut authority) = authority else {
            return Ok(());
        };
        if authority.outer_cursor != Some(expected_outer) {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "checked root answer authority returned through the wrong outer cursor",
            ));
        }
        // The exact source-machine delimiter consumes this cursor binding.
        // A later source-machine episode may bind the same affine root token
        // to its own exact outer cursor; retaining the old cursor would turn a
        // lawful sequential episode into an apparent transplant.
        authority.outer_cursor = None;
        if self.root_terminal_authority.replace(authority).is_some() {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "checked root answer authority was duplicated across source control",
            ));
        }
        Ok(())
    }

    /// The checked root cut determines the temporary scalar representation
    /// used at internal CFG joins. This is validation metadata only: it cannot
    /// mint or consume terminal authority, which remains affine in
    /// `RootTerminalAnswerAuthority` until `emit_result`.
    fn has_checked_root_exit_representation(&self) -> bool {
        self.process_object
            && self.native_join_plan.as_ref().is_some_and(|plan| {
                plan.sites.iter().any(|site| {
                    site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                        && site.checked_occurrence_path == [0]
                        && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
                        && self.consumed_join_sites.contains(&site.site_id)
                })
            })
    }

    fn mint_terminal_answer_authority(
        &mut self,
    ) -> Result<TerminalAnswerAuthority, CraneliftBackendError> {
        debug_assert_eq!(
            self.live_source_continuations == 0,
            self.source_control_root.is_none(),
            "source-control ownership and diagnostic depth must agree"
        );
        let authority = self.root_terminal_authority.take().ok_or_else(|| {
            unsupported(
                "NativeJoinPlanV1",
                "terminal answer has no affine checked-root authority",
            )
        })?;
        let site = self
            .native_join_plan
            .as_ref()
            .and_then(|plan| {
                plan.sites
                    .iter()
                    .find(|site| site.site_id == authority.site_id)
            })
            .ok_or_else(|| {
                unsupported(
                    "NativeJoinPlanV1",
                    "terminal answer authority names a missing checked-root site",
                )
            })?;
        if !self.process_object
            || site.runtime_frame_fingerprint != crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
            || site.checked_occurrence_path != [0]
            || site.answer_kind != crate::NativeJoinAnswerKindV1::ExitCode
            || site.checked_result_type_fingerprint != authority.checked_result_type_fingerprint
            || site.occurrence_binding_fingerprint != authority.occurrence_binding_fingerprint
            || !self.consumed_join_sites.contains(&authority.site_id)
            || authority.outer_cursor.is_some()
            || self.source_control_root.is_some()
            || self.active_oriented_semantic_regions != 0
            || self.active_subcontinuation_frame.is_some()
            || self.active_join_site.is_some()
        {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "terminal answer authority does not match the exhausted checked root",
            ));
        }
        Ok(TerminalAnswerAuthority)
    }

    fn unwrap_terminal_ret(mut lowered: Lowered) -> Lowered {
        loop {
            match lowered {
                Lowered::Constructor {
                    constructor,
                    mut args,
                } if constructor.ends_with("::ITree::Ret") && args.len() == 1 => {
                    lowered = args.remove(0);
                }
                lowered => return lowered,
            }
        }
    }

    /// Scalarize only under the answer kind carried by an already-consumed
    /// checked join site. In particular, process-object mode is not evidence
    /// that an arbitrary constructor is terminal: only an `ExitCode` plan may
    /// invoke the terminal process decoder.
    fn merge_planned_scalar_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: Lowered,
        required_kind: ScalarMergeKind,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        if required_kind == ScalarMergeKind::ExitCode {
            let lowered = Self::unwrap_terminal_ret(lowered);
            let zero_tag = builder.ins().iconst(types::I64, 0);
            return match lowered {
                Lowered::RecursiveBackedge => Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: builder.ins().iconst(types::I64, 0),
                    },
                    ScalarMergeKind::RecursiveBackedge,
                )),
                Lowered::ProcessExitStatus { value } => Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: value,
                    },
                    ScalarMergeKind::ExitCode,
                )),
                lowered if self.process_object => Ok((
                    NativeScalarPairV1 {
                        tag: zero_tag,
                        payload: self.emit_process_exit_status(builder, lowered),
                    },
                    ScalarMergeKind::ExitCode,
                )),
                _ => Err(unsupported(
                    construct,
                    "checked ExitCode join is unavailable outside process-object lowering",
                )),
            };
        }
        self.merge_scalar_branch(builder, lowered, construct)
    }

    fn record_merge_kind(
        construct: &'static str,
        expected: &mut Option<bool>,
        exit_status: bool,
    ) -> Result<(), CraneliftBackendError> {
        match expected {
            Some(expected) if *expected != exit_status => Err(unsupported(
                construct,
                "dynamic native arms disagree on scalar versus ExitCode result",
            )),
            Some(_) => Ok(()),
            None => {
                *expected = Some(exit_status);
                Ok(())
            }
        }
    }

    fn lowered_from_scalar_pair(
        &mut self,
        kind: ScalarMergeKind,
        pair: NativeScalarPairV1,
    ) -> Lowered {
        match kind {
            ScalarMergeKind::Int => {
                self.native_int_tags.insert(pair.payload, pair.tag);
                Lowered::Int {
                    value: pair.payload,
                    known: None,
                }
            }
            ScalarMergeKind::Bool => Lowered::Bool {
                value: pair.payload,
                known: None,
            },
            ScalarMergeKind::StructuralNat => Lowered::StructuralNat(StructuralNatV1 {
                value: pair.payload,
            }),
            ScalarMergeKind::ExitCode => Lowered::ProcessExitStatus {
                value: pair.payload,
            },
            ScalarMergeKind::RecursiveBackedge => {
                unreachable!("backedges do not establish a merge result kind")
            }
        }
    }

    fn record_scalar_merge_kind(
        construct: &'static str,
        expected: &mut Option<ScalarMergeKind>,
        kind: ScalarMergeKind,
    ) -> Result<(), CraneliftBackendError> {
        if kind == ScalarMergeKind::RecursiveBackedge {
            return Ok(());
        }
        match expected {
            Some(expected) if *expected != kind => Err(unsupported(
                construct,
                "dynamic native arms disagree on scalar result kind",
            )),
            Some(_) => Ok(()),
            None => {
                *expected = Some(kind);
                Ok(())
            }
        }
    }

    fn planned_join_site_for_frame(
        &mut self,
        frame: EliminatorFrame<'_>,
    ) -> Result<Option<crate::NativeJoinPlanSiteV1>, CraneliftBackendError> {
        let fingerprint = match frame {
            EliminatorFrame::Computational(frame) => {
                crate::compiler_private_computational_match_frame_fingerprint(
                    frame.cases,
                    frame.default,
                )
            }
            EliminatorFrame::Ordinary(frame) => {
                crate::compiler_private_ordinary_match_frame_fingerprint(frame.cases, frame.default)
            }
            EliminatorFrame::InvocationReturn => crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1,
            EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => return Ok(None),
        };
        let Some(plan) = &self.native_join_plan else {
            return Ok(None);
        };
        if matches!(frame, EliminatorFrame::InvocationReturn) && self.active_join_site.is_some() {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "distinguished root cannot consume an active match occurrence marker",
            ));
        }
        let matches = match frame {
            EliminatorFrame::InvocationReturn => plan
                .sites
                .iter()
                .filter(|site| {
                    site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                        && site.checked_occurrence_path == [0]
                        && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
                })
                .cloned()
                .collect::<Vec<_>>(),
            EliminatorFrame::Computational(_) | EliminatorFrame::Ordinary(_) => {
                let Some(site_id) = self.active_join_site else {
                    return Ok(None);
                };
                plan.sites
                    .iter()
                    .filter(|site| site.site_id == site_id)
                    .cloned()
                    .collect::<Vec<_>>()
            }
            EliminatorFrame::PendingLet(_) | EliminatorFrame::Active(_) => unreachable!(),
        };
        match matches.as_slice() {
            [] if self.active_join_site.is_some() => Err(unsupported(
                "NativeJoinPlanV1",
                "runtime occurrence has no exact checked join site",
            )),
            [] => Ok(None),
            [site] => {
                if site.runtime_frame_fingerprint != fingerprint
                    || site.occurrence_binding_fingerprint
                        != crate::compiler_private_join_occurrence_binding_fingerprint(
                            site.site_id,
                            &site.declaration,
                            &site.checked_occurrence_path,
                            site.checked_result_type_fingerprint,
                        )
                {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence binding is stale or inconsistent",
                    ));
                }
                if !self.consumed_join_sites.insert(site.site_id)
                    && !matches!(frame, EliminatorFrame::InvocationReturn)
                {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "checked join occurrence was consumed twice",
                    ));
                }
                if !matches!(frame, EliminatorFrame::InvocationReturn) {
                    self.active_join_site = None;
                }
                Ok(Some(site.clone()))
            }
            _ => Err(unsupported(
                "NativeJoinPlanV1",
                "checked cut identity resolves to multiple plan sites",
            )),
        }
    }

    fn require_complete_join_plan_consumption(&self) -> Result<(), CraneliftBackendError> {
        let Some(plan) = &self.native_join_plan else {
            return Ok(());
        };
        let planned = plan
            .sites
            .iter()
            .map(|site| site.site_id)
            .collect::<BTreeSet<_>>();
        if planned != self.consumed_join_sites {
            return Err(unsupported(
                "NativeJoinPlanV1",
                format!(
                    "checked join plan contains an unconsumed or orphan site: planned {planned:?}, consumed {:?}",
                    self.consumed_join_sites
                ),
            ));
        }
        Ok(())
    }

    fn require_complete_dynamic_splice_edge_consumption(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        if self.dynamic_splice_edges.is_empty() {
            return Ok(());
        }
        Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "checked lowering left affine dynamic splice edges unconsumed: {:?}",
                self.dynamic_splice_edges.keys().collect::<Vec<_>>(),
            ),
        ))
    }

    fn take_distinguished_root_answer_authority(
        &mut self,
    ) -> Result<Option<RootTerminalAnswerAuthority>, CraneliftBackendError> {
        let Some(plan) = &self.native_join_plan else {
            return if self.process_object {
                Err(unsupported(
                    "NativeJoinPlanV1",
                    "process-object lowering has no checked distinguished-root answer authority",
                ))
            } else {
                Ok(None)
            };
        };
        let roots = plan
            .sites
            .iter()
            .filter(|site| {
                site.runtime_frame_fingerprint == crate::NATIVE_JOIN_INVOCATION_RETURN_FRAME_V1
                    && site.checked_occurrence_path == [0]
                    && site.answer_kind == crate::NativeJoinAnswerKindV1::ExitCode
            })
            .cloned()
            .collect::<Vec<_>>();
        let site = match roots.as_slice() {
            [] if !self.process_object => return Ok(None),
            [] => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "process-object lowering has no checked distinguished-root answer authority",
                ));
            }
            [site] => site,
            _ => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "checked package contains multiple distinguished root join sites",
                ));
            }
        };
        if site.occurrence_binding_fingerprint
            != crate::compiler_private_join_occurrence_binding_fingerprint(
                site.site_id,
                &site.declaration,
                &site.checked_occurrence_path,
                site.checked_result_type_fingerprint,
            )
        {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "distinguished root join occurrence binding is stale or inconsistent",
            ));
        }
        if !self.consumed_join_sites.insert(site.site_id) {
            return Err(unsupported(
                "NativeJoinPlanV1",
                "checked distinguished-root answer authority was consumed more than once",
            ));
        }
        Ok(Some(RootTerminalAnswerAuthority {
            site_id: site.site_id,
            checked_result_type_fingerprint: site.checked_result_type_fingerprint,
            occurrence_binding_fingerprint: site.occurrence_binding_fingerprint,
            outer_cursor: None,
        }))
    }

    fn scalar_kind_from_plan(kind: crate::NativeJoinAnswerKindV1) -> ScalarMergeKind {
        match kind {
            crate::NativeJoinAnswerKindV1::Int => ScalarMergeKind::Int,
            crate::NativeJoinAnswerKindV1::Bool => ScalarMergeKind::Bool,
            crate::NativeJoinAnswerKindV1::StructuralNat => ScalarMergeKind::StructuralNat,
            crate::NativeJoinAnswerKindV1::ExitCode => ScalarMergeKind::ExitCode,
        }
    }

    fn declaration_call_produces_deforestable_aggregate(&self, expr: &RuntimeExpr) -> bool {
        let RuntimeExpr::Call { callee, .. } = expr else {
            return false;
        };
        let RuntimeExpr::DeclarationRef { symbol } = callee.as_ref() else {
            return false;
        };
        let Some(declaration) = self.declarations.get(symbol.as_str()).copied() else {
            return false;
        };
        let RuntimeDeclarationKind::Transparent {
            body:
                RuntimeExpr::Closure {
                    body: declaration_body,
                    ..
                },
        } = &declaration.kind
        else {
            return false;
        };
        produces_recursive_deforestable_aggregate(declaration_body, symbol)
    }

    fn source_terminal_join<'b, 'c>(
        continuation: &'b SourceContinuation<'c>,
    ) -> Option<&'b SourceJoinTarget<'c>> {
        match continuation {
            SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)) => {
                Some(&edge.target)
            }
            SourceContinuation::Terminal(
                SourceContinuationTerminal::ReturnValue
                | SourceContinuationTerminal::ReturnToProducerHole { .. }
                | SourceContinuationTerminal::ResumeOuter { .. },
            ) => None,
            SourceContinuation::LetBody { next, .. }
            | SourceContinuation::CheckedRecursiveInvocationReturn { next, .. }
            | SourceContinuation::CheckedComputationalIHInvocationReturn { next, .. }
            | SourceContinuation::ReturnFromSelectedCase { next, .. }
            | SourceContinuation::ApplyRecursorSelection { next, .. }
            | SourceContinuation::UnwindRecursorSegment { next, .. }
            | SourceContinuation::IfScrutinee { next, .. }
            | SourceContinuation::ConstructArgument { next, .. }
            | SourceContinuation::MatchScrutinee { next, .. }
            | SourceContinuation::ComputationalMatchScrutinee { next, .. }
            | SourceContinuation::ProjectRecord { next, .. }
            | SourceContinuation::CallCallee { next, .. }
            | SourceContinuation::CallArgument { next, .. } => Self::source_terminal_join(next),
        }
    }

    fn discard_source_prefix<'b>(continuation: SourceContinuation<'b>) -> SourceContinuation<'b> {
        match continuation {
            terminal @ SourceContinuation::Terminal(_) => terminal,
            SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                SourceContinuation::CheckedRecursiveInvocationReturn {
                    instance,
                    next: Box::new(Self::discard_source_prefix(*next)),
                }
            }
            SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next: Box::new(Self::discard_source_prefix(*next)),
            },
            SourceContinuation::ReturnFromSelectedCase { next, .. } => {
                Self::discard_source_prefix(*next)
            }
            SourceContinuation::LetBody { next, .. }
            | SourceContinuation::ApplyRecursorSelection { next, .. }
            | SourceContinuation::UnwindRecursorSegment { next, .. }
            | SourceContinuation::IfScrutinee { next, .. }
            | SourceContinuation::ConstructArgument { next, .. }
            | SourceContinuation::MatchScrutinee { next, .. }
            | SourceContinuation::ComputationalMatchScrutinee { next, .. }
            | SourceContinuation::ProjectRecord { next, .. }
            | SourceContinuation::CallCallee { next, .. }
            | SourceContinuation::CallArgument { next, .. } => Self::discard_source_prefix(*next),
        }
    }

    fn replace_source_terminal_with_unwind<'b>(
        continuation: SourceContinuation<'b>,
        stack: RecursorUnwindStack,
        resume_cursor: ContinuationCursorId,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        Ok(match continuation {
            SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                SourceContinuation::CheckedRecursiveInvocationReturn {
                    instance,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                SourceContinuation::ReturnFromSelectedCase {
                    delimiter,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::LetBody { body, env, next } => SourceContinuation::LetBody {
                body,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ApplyRecursorSelection { layer, next } => {
                SourceContinuation::ApplyRecursorSelection {
                    layer,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::UnwindRecursorSegment {
                stack: outer_stack,
                resume_cursor: outer_cursor,
                next,
            } => SourceContinuation::UnwindRecursorSegment {
                stack: outer_stack,
                resume_cursor: outer_cursor,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ConstructArgument {
                constructor,
                remaining: arguments,
                lowered,
                env,
                next,
            } => SourceContinuation::ConstructArgument {
                constructor,
                remaining: arguments,
                lowered,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::ProjectRecord { field, next } => {
                SourceContinuation::ProjectRecord {
                    field,
                    next: Box::new(Self::replace_source_terminal_with_unwind(
                        *next,
                        stack,
                        resume_cursor,
                    )?),
                }
            }
            SourceContinuation::CallCallee { args, env, next } => SourceContinuation::CallCallee {
                args,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::CallArgument {
                callee,
                remaining: arguments,
                lowered,
                env,
                next,
            } => SourceContinuation::CallArgument {
                callee,
                remaining: arguments,
                lowered,
                env,
                next: Box::new(Self::replace_source_terminal_with_unwind(
                    *next,
                    stack,
                    resume_cursor,
                )?),
            },
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected,
                active,
                root_authority,
            }) => SourceContinuation::Terminal(SourceContinuationTerminal::ReturnToProducerHole {
                stack,
                resume_cursor,
                expected,
                active,
                root_authority,
            }),
            terminal @ SourceContinuation::Terminal(_) => terminal,
        })
    }

    fn install_recursor_invocation<'b>(
        &mut self,
        continuation: SourceContinuation<'b>,
        activation: ContinuationActivationId,
        invocation: RecursorInvocationSegment,
        checked_ih_invocation: Option<CheckedRecursiveInvocationInstance>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        if !recursor_invocation_is_checked(&invocation) {
            validate_recursor_invocation_install_shape(&invocation)?;
        }
        #[cfg(test)]
        px8j_record_source_event(Px8jSourceTraceEvent::Install {
            origin: invocation.origin,
            selection_cursor: invocation.resume_cursor,
            sibling_position: invocation.sibling_position,
            exits: invocation
                .unwind
                .later_wrappers_in_construction_order
                .iter()
                .filter_map(|layer| match layer.role {
                    RecursorLayerRole::ExitsScope {
                        scope_origin,
                        parent_scope,
                        ..
                    } => Some((scope_origin, parent_scope)),
                    RecursorLayerRole::SelectsOccurrence { .. } => None,
                })
                .collect(),
        });
        let sibling_position = invocation.sibling_position;
        let dynamic_splice_edges = self.take_dynamic_splice_edges(&invocation)?;
        let installed = compose_oriented_subcontinuation(
            self.oriented_subcontinuation_plan.as_ref(),
            checked_ih_invocation.or_else(|| self.active_recursive_invocations.last().copied()),
            activation,
            invocation,
            dynamic_splice_edges,
        )?;
        debug_assert_eq!(installed.activation, activation);
        debug_assert!(installed
            .control_ledger
            .iter()
            .all(|entry| match entry.role {
                RecursorLayerRole::SelectsOccurrence { origin }
                | RecursorLayerRole::ExitsScope { origin, .. } => {
                    origin == installed.producer_origin
                }
            }));
        debug_assert_eq!(installed.sibling_position, sibling_position);
        debug_assert!(installed.control_ledger.len() >= installed.semantic_frames.len());
        debug_assert!(installed.control_ledger.iter().all(|entry| {
            entry.frame_id.is_some() == entry.checked_witness.is_some()
                && (entry.frame_id.is_none()
                    || matches!(
                        entry.role,
                        RecursorLayerRole::SelectsOccurrence { .. }
                            | RecursorLayerRole::ExitsScope { .. }
                    ))
        }));
        if !installed.checked {
            let mut frames = installed.semantic_frames.into_iter();
            let selection = frames
                .next()
                .expect("validated recursor invocation has a selection frame");
            let stack = RecursorUnwindStack {
                later_wrappers_in_construction_order: frames.rev().collect(),
            };
            let continuation = Self::replace_source_terminal_with_unwind(
                continuation,
                stack,
                installed.resume_cursor,
            )?;
            return Ok(SourceContinuation::ApplyRecursorSelection {
                layer: selection,
                next: Box::new(continuation),
            });
        }
        let mut continuation = continuation;
        for layer in installed.semantic_frames.into_iter().rev() {
            continuation = SourceContinuation::ApplyRecursorSelection {
                layer,
                next: Box::new(continuation),
            };
        }
        Ok(continuation)
    }

    fn split_source_prefix<'b>(
        source: SourceContinuation<'b>,
    ) -> Result<(SourcePrefixTemplate, SourcePrefixTerminal<'b>), CraneliftBackendError> {
        Ok(match source {
            SourceContinuation::CheckedRecursiveInvocationReturn { instance, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CheckedRecursiveInvocationReturn {
                        instance,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
                        call_template_id,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ReturnFromSelectedCase { delimiter, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ReturnFromSelectedCase {
                        delimiter,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnValue) => {
                return Err(unsupported(
                    "NativeJoinPlanV1",
                    "source prefix has no exact outer terminal to split",
                ));
            }
            SourceContinuation::Terminal(SourceContinuationTerminal::ResumeOuter {
                expected,
                root_authority,
                ..
            }) => (
                SourcePrefixTemplate::Terminal {
                    expected_outer: expected,
                },
                SourcePrefixTerminal::ResumeOuter { root_authority },
            ),
            SourceContinuation::Terminal(SourceContinuationTerminal::ReturnToProducerHole {
                expected,
                root_authority,
                ..
            }) => (
                SourcePrefixTemplate::Terminal {
                    expected_outer: expected,
                },
                SourcePrefixTerminal::ResumeOuter { root_authority },
            ),
            SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge)) => (
                SourcePrefixTemplate::Terminal {
                    expected_outer: edge.target.expected_outer,
                },
                SourcePrefixTerminal::Join(edge),
            ),
            SourceContinuation::LetBody { body, env, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::LetBody {
                        body,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ApplyRecursorSelection { layer, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ApplyRecursorSelection {
                        layer,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::UnwindRecursorSegment {
                stack,
                resume_cursor,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::UnwindRecursorSegment {
                        stack,
                        resume_cursor,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::IfScrutinee {
                        then_expr,
                        else_expr,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ConstructArgument {
                constructor,
                remaining,
                lowered,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ConstructArgument {
                        constructor,
                        remaining,
                        lowered,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::MatchScrutinee {
                        cases,
                        default,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
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
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ComputationalMatchScrutinee {
                        cases,
                        default,
                        env,
                        provenance,
                        checked_frame_id,
                        answer_route,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ProjectRecord { field, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ProjectRecord {
                        field,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::CallCallee { args, env, next } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CallCallee {
                        args,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::CallArgument {
                callee,
                remaining,
                lowered,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::CallArgument {
                        callee,
                        remaining,
                        lowered,
                        env,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
        })
    }

    fn instantiate_source_prefix_template<'b>(
        template: &SourcePrefixTemplate,
        edge: SourcePredecessorEdge<'b>,
    ) -> Result<SourceContinuation<'b>, CraneliftBackendError> {
        Ok(match template {
            SourcePrefixTemplate::Terminal { expected_outer } => {
                if *expected_outer != edge.target.expected_outer {
                    return Err(unsupported(
                        "NativeJoinPlanV1",
                        "source prefix terminal does not match the planned outer cursor",
                    ));
                }
                SourceContinuation::Terminal(SourceContinuationTerminal::JumpToJoin(edge))
            }
            SourcePrefixTemplate::CheckedRecursiveInvocationReturn { instance, next } => {
                SourceContinuation::CheckedRecursiveInvocationReturn {
                    instance: *instance,
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::CheckedComputationalIHInvocationReturn {
                call_template_id,
                next,
            } => SourceContinuation::CheckedComputationalIHInvocationReturn {
                call_template_id: *call_template_id,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ReturnFromSelectedCase { delimiter, next } => {
                SourceContinuation::ReturnFromSelectedCase {
                    delimiter: *delimiter,
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::LetBody { body, env, next } => SourceContinuation::LetBody {
                body: body.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ApplyRecursorSelection { layer, next } => {
                SourceContinuation::ApplyRecursorSelection {
                    layer: layer.clone(),
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::UnwindRecursorSegment {
                stack,
                resume_cursor,
                next,
            } => SourceContinuation::UnwindRecursorSegment {
                stack: stack.clone(),
                resume_cursor: *resume_cursor,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::IfScrutinee {
                then_expr,
                else_expr,
                env,
                next,
            } => SourceContinuation::IfScrutinee {
                then_expr: then_expr.clone(),
                else_expr: else_expr.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ConstructArgument {
                constructor,
                remaining,
                lowered,
                env,
                next,
            } => SourceContinuation::ConstructArgument {
                constructor: constructor.clone(),
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::MatchScrutinee {
                cases,
                default,
                env,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                provenance: *provenance,
                checked_frame_id: *checked_frame_id,
                answer_route: *answer_route,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ProjectRecord { field, next } => {
                SourceContinuation::ProjectRecord {
                    field: field.clone(),
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::CallCallee { args, env, next } => {
                SourceContinuation::CallCallee {
                    args: args.clone(),
                    env: env.clone(),
                    next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
                }
            }
            SourcePrefixTemplate::CallArgument {
                callee,
                remaining,
                lowered,
                env,
                next,
            } => SourceContinuation::CallArgument {
                callee: callee.clone(),
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
        })
    }

    fn mint_source_predecessor<'b>(
        &mut self,
        target: SourceJoinTarget<'b>,
    ) -> SourcePredecessorEdge<'b> {
        let predecessor_identity = self.next_source_predecessor;
        self.next_source_predecessor = self
            .next_source_predecessor
            .checked_add(1)
            .expect("compiler-private source predecessor identity exhausted");
        SourcePredecessorEdge {
            target,
            predecessor_identity,
        }
    }

    fn seal_source_trap_branch(builder: &mut FunctionBuilder<'_>, lowered: &Lowered) -> bool {
        if matches!(lowered, Lowered::Trap(_)) {
            let failure = builder.ins().iconst(types::I64, -4);
            builder.ins().return_(&[failure]);
            true
        } else {
            false
        }
    }

    fn planned_active_scalar_cut<'b>(
        &mut self,
        active: ActiveContinuationFrame<'b>,
    ) -> Result<
        (
            Vec<EliminatorFrame<'b>>,
            &'b [EliminatorFrame<'b>],
            ScalarMergeKind,
            u64,
        ),
        CraneliftBackendError,
    > {
        for (index, frame) in active.pending.iter().copied().enumerate() {
            if let Some(site) = self.planned_join_site_for_frame(frame)? {
                let prefix_end = if matches!(frame, EliminatorFrame::InvocationReturn) {
                    index
                } else {
                    index + 1
                };
                return Ok((
                    active.pending[..prefix_end].to_vec(),
                    &active.pending[prefix_end..],
                    Self::scalar_kind_from_plan(site.answer_kind),
                    site.site_id,
                ));
            }
        }
        if active.pending.is_empty() {
            if let Some(site) =
                self.planned_join_site_for_frame(EliminatorFrame::InvocationReturn)?
            {
                return Ok((
                    Vec::new(),
                    active.pending,
                    Self::scalar_kind_from_plan(site.answer_kind),
                    site.site_id,
                ));
            }
        }
        Err(unsupported(
            "NativeJoinPlanV1",
            "active checked continuation has no planned scalar cut before its outer suffix",
        ))
    }

    fn finish_source_constructor(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        constructor: RuntimeSymbol,
        lowered_args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        if lowered_args
            .iter()
            .any(|arg| matches!(arg, Lowered::RecursiveBackedge))
        {
            return Ok(Lowered::RecursiveBackedge);
        }
        if lowered_args.is_empty()
            && (constructor == self.process_symbols.bool_true
                || constructor == self.process_symbols.bool_false)
        {
            let known = constructor == self.process_symbols.bool_true;
            return Ok(Lowered::Bool {
                value: builder.ins().iconst(types::I64, i64::from(known)),
                known: Some(known),
            });
        }
        if constructor == self.process_symbols.nat_zero && lowered_args.is_empty() {
            return Ok(Lowered::StructuralNat(StructuralNatV1 {
                value: builder.ins().iconst(types::I64, 0),
            }));
        }
        if constructor == self.process_symbols.nat_suc {
            if let [Lowered::StructuralNat(predecessor)] = lowered_args.as_slice() {
                return Ok(Lowered::StructuralNat(StructuralNatV1 {
                    value: builder.ins().iadd_imm(predecessor.value, 1),
                }));
            }
        }
        Ok(Lowered::Constructor {
            constructor,
            args: lowered_args,
        })
    }

    fn wire_bytes(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value), CraneliftBackendError>
    {
        let pointer_type = builder.func.dfg.value_type(
            self.invocation_pointer
                .expect("process byte lowering owns an invocation pointer"),
        );
        match value {
            Lowered::BorrowedNativeValue { pointer } => {
                let kind = builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), *pointer, 0);
                Self::require_i64(builder, kind, 1);
                Ok((
                    builder
                        .ins()
                        .load(pointer_type, MemFlags::trusted(), *pointer, 16),
                    builder
                        .ins()
                        .load(types::I64, MemFlags::trusted(), *pointer, 24),
                ))
            }
            Lowered::ResponseBytes { pointer, len } => Ok((*pointer, *len)),
            Lowered::Bytes(bytes) => {
                if bytes.is_empty() {
                    return Ok((
                        builder.ins().iconst(pointer_type, 0),
                        builder.ins().iconst(types::I64, 0),
                    ));
                }
                let size = u32::try_from(bytes.len())
                    .map_err(|_| unsupported("Effect", "Bytes exceed native stack slot"))?;
                let slot = builder.create_sized_stack_slot(StackSlotData::new(
                    StackSlotKind::ExplicitSlot,
                    size,
                    0,
                ));
                for (offset, byte) in bytes.iter().enumerate() {
                    let byte = builder.ins().iconst(types::I8, i64::from(*byte));
                    builder.ins().stack_store(byte, slot, offset as i32);
                }
                Ok((
                    builder.ins().stack_addr(pointer_type, slot, 0),
                    builder.ins().iconst(types::I64, bytes.len() as i64),
                ))
            }
            _ => Err(unsupported("Effect", "operand is not a Bytes value")),
        }
    }

    fn narrow_native_int_u64(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value), CraneliftBackendError>
    {
        let Lowered::Int { value, known } = value else {
            return Err(unsupported("Effect", "host-width operand is not Int"));
        };
        let arena = self
            .native_int_arena
            .ok_or_else(|| unsupported("Effect", "host-width Int has no invocation arena"))?;
        let helper = self.native_int_narrow.ok_or_else(|| {
            unsupported("Effect", "host-width Int has no checked narrowing helper")
        })?;
        let tag = self.native_int_tag(builder, *value, *known)?;
        let output_slot =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        let pointer_type = builder.func.dfg.value_type(arena);
        let output = builder.ins().stack_addr(pointer_type, output_slot, 0);
        let call = builder.ins().call(helper, &[arena, tag, *value, output]);
        let status = builder.inst_results(call)[0];
        Self::require_one_of_i64(builder, status, &[0, 1]);
        let valid =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::Equal, status, 0);
        let value = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), output, 0);
        Ok((value, valid))
    }

    fn lower_dynamic_small_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: cranelift_codegen::ir::Value,
    ) -> Lowered {
        let tag = builder
            .ins()
            .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
        self.native_int_tags.insert(value, tag);
        Lowered::Int { value, known: None }
    }

    fn declaration_is_recursive(&self, symbol: &RuntimeSymbol) -> bool {
        let Some(declaration) = self.declarations.get(symbol.as_str()).copied() else {
            return false;
        };
        let RuntimeDeclarationKind::Transparent { body } = &declaration.kind else {
            return false;
        };

        let mut frontier = BTreeSet::new();
        let mut visited = BTreeSet::new();
        collect_runtime_declaration_refs(body, &mut frontier);
        while let Some(candidate) = frontier.pop_first() {
            if candidate == *symbol {
                return true;
            }
            if !visited.insert(candidate.clone()) {
                continue;
            }
            let Some(declaration) = self.declarations.get(candidate.as_str()).copied() else {
                continue;
            };
            if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
                collect_runtime_declaration_refs(body, &mut frontier);
            }
        }
        false
    }

    fn require_i64(
        builder: &mut FunctionBuilder<'_>,
        actual: cranelift_codegen::ir::Value,
        expected: i64,
    ) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let matches = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            actual,
            expected,
        );
        builder.ins().brif(matches, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_one_of_i64(
        builder: &mut FunctionBuilder<'_>,
        actual: cranelift_codegen::ir::Value,
        expected: &[i64],
    ) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let mut matches = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            actual,
            expected[0],
        );
        for expected in &expected[1..] {
            let next = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                actual,
                *expected,
            );
            matches = builder.ins().bor(matches, next);
        }
        builder.ins().brif(matches, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_nonzero(builder: &mut FunctionBuilder<'_>, value: cranelift_codegen::ir::Value) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let present =
            builder
                .ins()
                .icmp_imm(cranelift_codegen::ir::condcodes::IntCC::NotEqual, value, 0);
        builder.ins().brif(present, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_u8(builder: &mut FunctionBuilder<'_>, value: cranelift_codegen::ir::Value) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        let in_range = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            value,
            i64::from(u8::MAX),
        );
        builder.ins().brif(in_range, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_true(builder: &mut FunctionBuilder<'_>, condition: cranelift_codegen::ir::Value) {
        let valid = builder.create_block();
        let invalid = builder.create_block();
        builder.ins().brif(condition, valid, &[], invalid, &[]);
        builder.switch_to_block(invalid);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(valid);
    }

    fn require_when(
        builder: &mut FunctionBuilder<'_>,
        enabled: cranelift_codegen::ir::Value,
        condition: cranelift_codegen::ir::Value,
    ) {
        let validate = builder.create_block();
        let done = builder.create_block();
        builder.ins().brif(enabled, validate, &[], done, &[]);
        builder.switch_to_block(validate);
        Self::require_true(builder, condition);
        builder.ins().jump(done, &[]);
        builder.switch_to_block(done);
    }

    fn mint_validated_progress_nat(
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        count: cranelift_codegen::ir::Value,
        request_start: cranelift_codegen::ir::Value,
        request_length: cranelift_codegen::ir::Value,
        reply_start: Option<cranelift_codegen::ir::Value>,
    ) -> (BoundedNatV1, BoundedNatV1, BoundedNatV1) {
        let positive = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            count,
            0,
        );
        let bounded = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            count,
            request_length,
        );
        let request_end = builder.ins().iadd(request_start, request_length);
        let request_no_wrap = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThanOrEqual,
            request_end,
            request_start,
        );
        let span_start = reply_start.unwrap_or(request_start);
        let span_end = builder.ins().iadd(span_start, count);
        let span_no_wrap = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThanOrEqual,
            span_end,
            span_start,
        );
        let starts_at_request = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            span_start,
            request_start,
        );
        let inside = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            span_end,
            request_end,
        );
        let valid = [
            positive,
            bounded,
            request_no_wrap,
            span_no_wrap,
            starts_at_request,
            inside,
        ]
        .into_iter()
        .reduce(|left, right| builder.ins().band(left, right))
        .expect("progress validation has fixed clauses");
        Self::require_when(builder, success, valid);

        let minted = BoundedNatV1::mint_after_reply_validation(count);
        let predecessor = minted.predecessor(builder);
        let remaining =
            BoundedNatV1::derived_from_validated(builder.ins().isub(request_length, count));
        (minted, predecessor, remaining)
    }

    fn validate_resource_io(
        builder: &mut FunctionBuilder<'_>,
        encoded: cranelift_codegen::ir::Value,
    ) {
        let discriminator = builder.ins().band_imm(encoded, 0xff);
        let other = builder.create_block();
        let ordinary = builder.create_block();
        let valid = builder.create_block();
        let is_other = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            discriminator,
            11,
        );
        builder.ins().brif(is_other, other, &[], ordinary, &[]);
        builder.switch_to_block(other);
        let middle = builder
            .ins()
            .band_imm(encoded, 0x0000_0000_ffff_ff00u64 as i64);
        Self::require_i64(builder, middle, 0);
        builder.ins().jump(valid, &[]);
        builder.switch_to_block(ordinary);
        let upper = builder.ins().ushr_imm(encoded, 8);
        Self::require_i64(builder, upper, 0);
        Self::require_one_of_i64(builder, discriminator, &[0, 1, 3, 4, 5, 6, 7, 8, 9, 10]);
        builder.ins().jump(valid, &[]);
        builder.switch_to_block(valid);
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_resource_error_reply(
        builder: &mut FunctionBuilder<'_>,
        reply_tag: cranelift_codegen::ir::Value,
        resource_reply_tag: u64,
        discriminator: cranelift_codegen::ir::Value,
        schema: cranelift_codegen::ir::Value,
        kind: cranelift_codegen::ir::Value,
        identity: cranelift_codegen::ir::Value,
        io: cranelift_codegen::ir::Value,
        required: cranelift_codegen::ir::Value,
        held: cranelift_codegen::ir::Value,
        actual_expected_kind: cranelift_codegen::ir::Value,
        actual_actual_kind: cranelift_codegen::ir::Value,
        expected_schema: u64,
        expected_kind: u64,
        buffer_kind: u64,
    ) {
        let resource = builder.create_block();
        let done = builder.create_block();
        let is_resource = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            reply_tag,
            resource_reply_tag as i64,
        );
        builder.ins().brif(is_resource, resource, &[], done, &[]);
        builder.switch_to_block(resource);
        let arms = (0..9).map(|_| builder.create_block()).collect::<Vec<_>>();
        let mut test = builder
            .current_block()
            .expect("resource reply validation block");
        for (index, arm) in arms.into_iter().enumerate() {
            let next = builder.create_block();
            if builder.current_block() != Some(test) {
                builder.switch_to_block(test);
            }
            let selected = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                discriminator,
                index as i64,
            );
            builder.ins().brif(selected, arm, &[], next, &[]);
            builder.switch_to_block(arm);
            match index {
                0 | 1 => {
                    for field in [
                        schema,
                        kind,
                        identity,
                        io,
                        required,
                        held,
                        actual_expected_kind,
                        actual_actual_kind,
                    ] {
                        Self::require_i64(builder, field, 0);
                    }
                }
                2 => {
                    Self::require_i64(builder, schema, expected_schema as i64);
                    Self::require_i64(builder, kind, 0);
                    Self::require_i64(builder, identity, 0);
                    Self::require_i64(builder, io, 0);
                    Self::require_i64(builder, actual_expected_kind, 0);
                    Self::require_i64(builder, actual_actual_kind, 0);
                    Self::require_u8(builder, required);
                    Self::require_u8(builder, held);
                }
                3 => {
                    Self::require_i64(builder, schema, expected_schema as i64);
                    Self::require_one_of_i64(
                        builder,
                        kind,
                        &[expected_kind as i64, buffer_kind as i64],
                    );
                    Self::require_i64(builder, required, 0);
                    Self::require_i64(builder, held, 0);
                    Self::require_i64(builder, actual_expected_kind, 0);
                    Self::require_i64(builder, actual_actual_kind, 0);
                    Self::validate_resource_io(builder, io);
                }
                4 => {
                    for field in [schema, kind, identity, io, required, held] {
                        Self::require_i64(builder, field, 0);
                    }
                    Self::require_one_of_i64(
                        builder,
                        actual_expected_kind,
                        &[expected_kind as i64, buffer_kind as i64],
                    );
                    Self::require_one_of_i64(
                        builder,
                        actual_actual_kind,
                        &[expected_kind as i64, buffer_kind as i64],
                    );
                    let distinct = builder.ins().icmp(
                        cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                        actual_expected_kind,
                        actual_actual_kind,
                    );
                    Self::require_true(builder, distinct);
                }
                5..=8 => {
                    for field in [
                        schema,
                        kind,
                        identity,
                        io,
                        required,
                        held,
                        actual_expected_kind,
                        actual_actual_kind,
                    ] {
                        Self::require_i64(builder, field, 0);
                    }
                }
                _ => unreachable!(),
            }
            builder.ins().jump(done, &[]);
            test = next;
        }
        builder.switch_to_block(test);
        let failure = builder.ins().iconst(types::I64, -1);
        builder.ins().return_(&[failure]);
        builder.switch_to_block(done);
    }

    fn lower_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeValue::Bool(value) => Ok(Lowered::Bool {
                value: builder.ins().iconst(types::I64, i64::from(*value)),
                known: Some(*value),
            }),
            RuntimeValue::Int(crate::RuntimeIntV1::Small(value)) => Ok(Lowered::Int {
                value: builder.ins().iconst(types::I64, *value),
                known: Some(*value),
            }),
            RuntimeValue::Int(value @ crate::RuntimeIntV1::Big { .. }) => {
                self.lower_big_int_constant(builder, value)
            }
            RuntimeValue::Bytes(value) => Ok(Lowered::Bytes(value.clone())),
            RuntimeValue::String(value) => Ok(Lowered::String(value.clone())),
            RuntimeValue::Constructor { constructor, args } => Ok(Lowered::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeValue::Record { fields } => Ok(Lowered::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| Ok((name.clone(), self.lower_value(builder, value)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
            RuntimeValue::ClosureRef { .. } => Err(unsupported(
                "ClosureRef",
                "pre-existing closure references are not lowered by the native backend",
            )),
            RuntimeValue::Unknown => Err(unsupported(
                "Unknown",
                "unknown runtime values must reject before backend lowering",
            )),
        }
    }

    fn lower_seed_capture(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &str,
    ) -> Result<Lowered, CraneliftBackendError> {
        let value = self.seed_env.values.get(symbol).ok_or_else(|| {
            unsupported(
                "Closure",
                format!("capture {symbol} has no runtime value in the seed environment"),
            )
        })?;
        self.lower_ground_value(builder, value)
    }

    fn lower_ground_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &RuntimeGroundValue,
    ) -> Result<Lowered, CraneliftBackendError> {
        match value {
            RuntimeGroundValue::Bool(value) => Ok(Lowered::Bool {
                value: builder.ins().iconst(types::I64, i64::from(*value)),
                known: Some(*value),
            }),
            RuntimeGroundValue::Int(crate::RuntimeIntV1::Small(value)) => Ok(Lowered::Int {
                value: builder.ins().iconst(types::I64, *value),
                known: Some(*value),
            }),
            RuntimeGroundValue::Int(value @ crate::RuntimeIntV1::Big { .. }) => {
                self.lower_big_int_constant(builder, value)
            }
            RuntimeGroundValue::Bytes(value) => Ok(Lowered::Bytes(value.clone())),
            RuntimeGroundValue::String(value) => Ok(Lowered::String(value.clone())),
            RuntimeGroundValue::Constructor { constructor, args } => Ok(Lowered::Constructor {
                constructor: constructor.clone(),
                args: args
                    .iter()
                    .map(|arg| self.lower_ground_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            RuntimeGroundValue::Record { fields } => Ok(Lowered::Record {
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        Ok((name.clone(), self.lower_ground_value(builder, value)?))
                    })
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
        }
    }

    fn lower_big_int_constant(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: &crate::RuntimeIntV1,
    ) -> Result<Lowered, CraneliftBackendError> {
        let crate::RuntimeIntV1::Big { sign, limbs } = value else {
            unreachable!("Big constant lowering is called only for Big Int values")
        };
        let limb_count = limbs.len();
        let byte_len = u32::try_from(limbs.len().saturating_mul(std::mem::size_of::<u64>()))
            .map_err(|_| unsupported("RuntimeValue::Int", "Big Int literal is too large"))?;
        let limbs_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            byte_len,
            3,
        ));
        for (index, limb) in limbs.iter().enumerate() {
            let limb = builder.ins().iconst(types::I64, *limb as i64);
            builder.ins().stack_store(
                limb,
                limbs_slot,
                i32::try_from(index * std::mem::size_of::<u64>()).expect("Big limb offset is u32"),
            );
        }
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let pointer_type = builder.func.dfg.value_type(
            self.native_int_arena
                .ok_or_else(|| unsupported("RuntimeValue::Int", "Big Int has no arena"))?,
        );
        let arena = self.native_int_arena.expect("Big Int arena was checked");
        let helper = self.native_int_intern.ok_or_else(|| {
            unsupported("RuntimeValue::Int", "Big Int has no local intern helper")
        })?;
        let sign = builder
            .ins()
            .iconst(types::I64, i64::from(matches!(sign, crate::Sign::Negative)));
        let limbs = builder.ins().stack_addr(pointer_type, limbs_slot, 0);
        let len = builder.ins().iconst(
            types::I64,
            i64::try_from(limb_count).expect("Big limb count fits i64"),
        );
        let output_ptr = builder.ins().stack_addr(pointer_type, output, 0);
        let call = builder
            .ins()
            .call(helper, &[arena, sign, limbs, len, output_ptr]);
        Self::require_i64(builder, builder.inst_results(call)[0], 0);
        let pair = NativeScalarPairV1 {
            tag: builder.ins().stack_load(types::I64, output, 0),
            payload: builder.ins().stack_load(types::I64, output, 8),
        };
        Ok(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
    }

    /// Reify a host-owned unsigned word into the exact native Int carrier.
    /// The shared local interner chooses Small or canonical Big; callers never
    /// reinterpret the raw `u64` bits as a signed scalar.
    fn lower_unsigned_u64_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: cranelift_codegen::ir::Value,
    ) -> Result<Lowered, CraneliftBackendError> {
        let arena = self.native_int_arena.ok_or_else(|| {
            unsupported("NativeInt", "unsigned Int producer has no invocation arena")
        })?;
        let helper = self.native_int_intern.ok_or_else(|| {
            unsupported(
                "NativeInt",
                "unsigned Int producer has no local intern helper",
            )
        })?;
        let limb =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        builder.ins().stack_store(value, limb, 0);
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let pointer_type = builder.func.dfg.value_type(arena);
        let limb = builder.ins().stack_addr(pointer_type, limb, 0);
        let output_pointer = builder.ins().stack_addr(pointer_type, output, 0);
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let call = builder
            .ins()
            .call(helper, &[arena, zero, limb, one, output_pointer]);
        Self::require_i64(builder, builder.inst_results(call)[0], 0);
        let pair = NativeScalarPairV1 {
            tag: builder.ins().stack_load(types::I64, output, 0),
            payload: builder.ins().stack_load(types::I64, output, 8),
        };
        Self::require_one_of_i64(
            builder,
            pair.tag,
            &[
                crate::NATIVE_INT_SMALL_TAG_V1 as i64,
                crate::NATIVE_INT_BIG_TAG_V1 as i64,
            ],
        );
        Ok(self.lowered_from_scalar_pair(ScalarMergeKind::Int, pair))
    }

    fn lower_int_binop(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        eval: impl FnOnce(i64, i64) -> Option<i64>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args(symbol, args)?;
        let (
            Lowered::Int {
                value: lhs,
                known: lhs_known,
            },
            Lowered::Int {
                value: rhs,
                known: rhs_known,
            },
        ) = (lhs, rhs)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                format!("{symbol} only supports Int arguments in native lowering"),
            ));
        };
        #[cfg(test)]
        match self.native_int_mutation {
            NativeIntLoweringMutation::Exact => {}
            NativeIntLoweringMutation::Wrapping => {}
            NativeIntLoweringMutation::Trap => {
                return Err(unsupported(
                    "PrimitiveCall",
                    "PX8-I mutation traps before exact Int support",
                ));
            }
            NativeIntLoweringMutation::SuppressTerminalExport
            | NativeIntLoweringMutation::CorruptTerminalExport => {}
        }
        let lhs_tag = self.native_int_tag(builder, lhs, lhs_known)?;
        let rhs_tag = self.native_int_tag(builder, rhs, rhs_known)?;
        let arena = self.native_int_arena.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int operation has no invocation arena",
            )
        })?;
        let helper = self.native_int_binop.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int operation has no local support function",
            )
        })?;
        let output =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 16, 3));
        let pointer_type = builder.func.dfg.value_type(arena);
        let output_pointer = builder.ins().stack_addr(pointer_type, output, 0);
        let operation = builder.ins().iconst(
            types::I64,
            match symbol {
                "add_int" => 0,
                "sub_int" => 1,
                "mul_int" => 2,
                _ => unreachable!("caller supplies exact Int arithmetic symbol"),
            },
        );
        let call = builder.ins().call(
            helper,
            &[arena, operation, lhs_tag, lhs, rhs_tag, rhs, output_pointer],
        );
        let status = builder.inst_results(call)[0];
        Self::require_i64(builder, status, 0);
        let tag = builder.ins().stack_load(types::I64, output, 0);
        let value = builder.ins().stack_load(types::I64, output, 8);
        Self::require_one_of_i64(
            builder,
            tag,
            &[
                crate::NATIVE_INT_SMALL_TAG_V1 as i64,
                crate::NATIVE_INT_BIG_TAG_V1 as i64,
            ],
        );
        self.native_int_tags.insert(value, tag);
        let known = lhs_known.and_then(|lhs| rhs_known.and_then(|rhs| eval(lhs, rhs)));
        Ok(Lowered::Int { value, known })
    }

    fn lower_int_cmp(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        _cc: cranelift_codegen::ir::condcodes::IntCC,
        eval: impl FnOnce(i64, i64) -> bool,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args(symbol, args)?;
        let (
            Lowered::Int {
                value: lhs,
                known: lhs_known,
            },
            Lowered::Int {
                value: rhs,
                known: rhs_known,
            },
        ) = (lhs, rhs)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                format!("{symbol} only supports Int arguments in native lowering"),
            ));
        };
        let lhs_tag = self.native_int_tag(builder, lhs, lhs_known)?;
        let rhs_tag = self.native_int_tag(builder, rhs, rhs_known)?;
        let arena = self.native_int_arena.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int comparison has no invocation arena",
            )
        })?;
        let helper = self.native_int_compare.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int comparison has no local support function",
            )
        })?;
        let operation = builder.ins().iconst(
            types::I64,
            match symbol {
                "eq_int" => 0,
                "leq_int" => 1,
                _ => unreachable!("caller supplies exact Int comparison symbol"),
            },
        );
        let call = builder
            .ins()
            .call(helper, &[arena, operation, lhs_tag, lhs, rhs_tag, rhs]);
        let value = builder.inst_results(call)[0];
        Self::require_one_of_i64(builder, value, &[0, 1]);
        Ok(Lowered::Bool {
            value,
            known: lhs_known.and_then(|lhs| rhs_known.map(|rhs| eval(lhs, rhs))),
        })
    }

    fn native_int_tag(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        payload: cranelift_codegen::ir::Value,
        known: Option<i64>,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        if let Some(tag) = self.native_int_tags.get(&payload).copied() {
            return Ok(tag);
        }
        if known.is_some() {
            return Ok(builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64));
        }
        Err(unsupported(
            "NativeInt",
            "dynamic Int value lost its two-word tag transport",
        ))
    }

    fn lower_bool_not(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("not_bool expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::Bool { value, known } = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "not_bool only supports Bool arguments in native lowering",
            ));
        };
        let one = builder.ins().iconst(types::I64, 1);
        Ok(Lowered::Bool {
            value: builder.ins().bxor(value, one),
            known: known.map(|value| !value),
        })
    }

    fn lower_bool_binop(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &'static str,
        args: Vec<Lowered>,
        emit: impl FnOnce(
            &mut FunctionBuilder<'_>,
            cranelift_codegen::ir::Value,
            cranelift_codegen::ir::Value,
        ) -> cranelift_codegen::ir::Value,
        eval: impl FnOnce(bool, bool) -> bool,
    ) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args(symbol, args)?;
        let (
            Lowered::Bool {
                value: lhs,
                known: lhs_known,
            },
            Lowered::Bool {
                value: rhs,
                known: rhs_known,
            },
        ) = (lhs, rhs)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                format!("{symbol} only supports Bool arguments in native lowering"),
            ));
        };
        Ok(Lowered::Bool {
            value: emit(builder, lhs, rhs),
            known: lhs_known.and_then(|lhs| rhs_known.map(|rhs| eval(lhs, rhs))),
        })
    }

    fn lower_bytes_length(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_length expects 1 arg, got {}", args.len()),
            )
        })?;
        if let Lowered::ResponseBytes { len, .. } = arg {
            return self.lower_unsigned_u64_int(builder, len);
        }
        if let Lowered::BorrowedNativeValue { pointer } = arg {
            let kind = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 0);
            Self::require_i64(builder, kind, 1);
            let len = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 24);
            return self.lower_unsigned_u64_int(builder, len);
        }
        let Lowered::Bytes(bytes) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_length only supports Bytes arguments in native lowering",
            ));
        };
        let len = i64::try_from(bytes.len()).map_err(|_| {
            unsupported(
                "PrimitiveCall",
                "bytes_length result does not fit the runtime Int representation",
            )
        })?;
        Ok(Lowered::Int {
            value: builder.ins().iconst(types::I64, len),
            known: Some(len),
        })
    }

    fn lower_bytes_at(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
        partiality: &RuntimePartiality,
    ) -> Result<Lowered, CraneliftBackendError> {
        let RuntimePartiality::SafeOption { none, some, .. } = partiality else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_at requires safe Option result metadata",
            ));
        };
        let (bytes, index) = expect_two_args("bytes_at", args)?;
        let Lowered::Int {
            known: Some(index), ..
        } = index
        else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_at requires a statically known Int index",
            ));
        };
        if let Lowered::ResponseBytes { pointer: data, len } = bytes {
            let index_value = builder.ins().iconst(types::I64, index);
            let present = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                index_value,
                len,
            );
            let in_bounds = builder.create_block();
            let out_of_bounds = builder.create_block();
            let merge = builder.create_block();
            builder.append_block_param(merge, types::I64);
            builder.append_block_param(merge, types::I64);
            builder
                .ins()
                .brif(present, in_bounds, &[], out_of_bounds, &[]);
            builder.switch_to_block(in_bounds);
            let address = builder.ins().iadd_imm(data, index);
            let byte = builder
                .ins()
                .load(types::I8, MemFlags::trusted(), address, 0);
            let yes = builder.ins().iconst(types::I64, 1);
            let byte = builder.ins().uextend(types::I64, byte);
            builder.ins().jump(merge, &[yes.into(), byte.into()]);
            builder.switch_to_block(out_of_bounds);
            let no = builder.ins().iconst(types::I64, 0);
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().jump(merge, &[no.into(), zero.into()]);
            builder.switch_to_block(merge);
            let value = builder.block_params(merge)[1];
            let tag = builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
            self.native_int_tags.insert(value, tag);
            return Ok(Lowered::BorrowedOption {
                present: builder.block_params(merge)[0],
                value,
                none: none.clone(),
                some: some.clone(),
            });
        }
        if let Lowered::BorrowedNativeValue { pointer } = bytes {
            let kind = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 0);
            Self::require_i64(builder, kind, 1);
            let pointer_type = builder.func.dfg.value_type(pointer);
            let data = builder
                .ins()
                .load(pointer_type, MemFlags::trusted(), pointer, 16);
            let len = builder
                .ins()
                .load(types::I64, MemFlags::trusted(), pointer, 24);
            let index_value = builder.ins().iconst(types::I64, index);
            let present = builder.ins().icmp(
                cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
                index_value,
                len,
            );
            let in_bounds = builder.create_block();
            let out_of_bounds = builder.create_block();
            let merge = builder.create_block();
            builder.append_block_param(merge, types::I64);
            builder
                .ins()
                .brif(present, in_bounds, &[], out_of_bounds, &[]);
            builder.switch_to_block(in_bounds);
            Self::require_nonzero(builder, data);
            let address = builder.ins().iadd_imm(data, index);
            let byte = builder
                .ins()
                .load(types::I8, MemFlags::trusted(), address, 0);
            let byte = builder.ins().uextend(types::I64, byte);
            builder.ins().jump(merge, &[byte.into()]);
            builder.switch_to_block(out_of_bounds);
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().jump(merge, &[zero.into()]);
            builder.switch_to_block(merge);
            let value = builder.block_params(merge)[0];
            let tag = builder
                .ins()
                .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
            self.native_int_tags.insert(value, tag);
            return Ok(Lowered::BorrowedOption {
                present,
                value,
                none: none.clone(),
                some: some.clone(),
            });
        }
        let Lowered::Bytes(bytes) = bytes else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_at requires Bytes in native lowering",
            ));
        };
        let byte = usize::try_from(index)
            .ok()
            .and_then(|index| bytes.get(index).copied());
        Ok(match byte {
            Some(byte) => Lowered::Constructor {
                constructor: some.clone(),
                args: vec![Lowered::Int {
                    value: builder.ins().iconst(types::I64, i64::from(byte)),
                    known: Some(i64::from(byte)),
                }],
            },
            None => Lowered::Constructor {
                constructor: none.clone(),
                args: Vec::new(),
            },
        })
    }

    fn lower_bytes_slice(
        &mut self,
        args: Vec<Lowered>,
        partiality: &RuntimePartiality,
    ) -> Result<Lowered, CraneliftBackendError> {
        let RuntimePartiality::SafeOption { none, some, .. } = partiality else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_slice requires safe Option result metadata",
            ));
        };
        let [bytes, start, len]: [Lowered; 3] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_slice expects 3 args, got {}", args.len()),
            )
        })?;
        let (
            Lowered::Bytes(bytes),
            Lowered::Int {
                known: Some(start), ..
            },
            Lowered::Int {
                known: Some(len), ..
            },
        ) = (bytes, start, len)
        else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_slice requires Bytes and statically known Int bounds",
            ));
        };
        let value = usize::try_from(start)
            .ok()
            .zip(usize::try_from(len).ok())
            .and_then(|(start, len)| {
                start
                    .checked_add(len)
                    .filter(|end| *end <= bytes.len())
                    .map(|end| bytes[start..end].to_vec())
            });
        Ok(match value {
            Some(bytes) => Lowered::Constructor {
                constructor: some.clone(),
                args: vec![Lowered::Bytes(bytes)],
            },
            None => Lowered::Constructor {
                constructor: none.clone(),
                args: Vec::new(),
            },
        })
    }

    fn lower_bytes_concat(&mut self, args: Vec<Lowered>) -> Result<Lowered, CraneliftBackendError> {
        let (lhs, rhs) = expect_two_args("bytes_concat", args)?;
        let (Lowered::Bytes(mut lhs), Lowered::Bytes(rhs)) = (lhs, rhs) else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_concat only supports Bytes arguments in native lowering",
            ));
        };
        lhs.extend(rhs);
        Ok(Lowered::Bytes(lhs))
    }

    fn lower_bytes_encode(&mut self, args: Vec<Lowered>) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_encode expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::String(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_encode only supports String arguments in native lowering",
            ));
        };
        Ok(Lowered::Bytes(value.into_bytes()))
    }

    fn lower_bytes_decode(
        &mut self,
        args: Vec<Lowered>,
        partiality: &RuntimePartiality,
    ) -> Result<Lowered, CraneliftBackendError> {
        let RuntimePartiality::SafeResult { err, ok, error } = partiality else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_decode requires safe Result metadata",
            ));
        };
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("bytes_decode expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::Bytes(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "bytes_decode only supports Bytes arguments in native lowering",
            ));
        };
        Ok(match String::from_utf8(value) {
            Ok(value) => Lowered::Constructor {
                constructor: ok.clone(),
                args: vec![Lowered::String(value)],
            },
            Err(_) => Lowered::Constructor {
                constructor: err.clone(),
                args: vec![Lowered::Constructor {
                    constructor: error.clone(),
                    args: Vec::new(),
                }],
            },
        })
    }

    fn lower_string_byte_length(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("byte_length expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::String(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "byte_length only supports String arguments in native lowering",
            ));
        };
        let len = i64::try_from(value.len()).map_err(|_| {
            unsupported(
                "PrimitiveCall",
                "byte_length result does not fit the runtime Int representation",
            )
        })?;
        Ok(Lowered::Int {
            value: builder.ins().iconst(types::I64, len),
            known: Some(len),
        })
    }

    fn lower_string_char_length(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        args: Vec<Lowered>,
    ) -> Result<Lowered, CraneliftBackendError> {
        let [arg]: [Lowered; 1] = args.try_into().map_err(|args: Vec<Lowered>| {
            unsupported(
                "PrimitiveCall",
                format!("char_length expects 1 arg, got {}", args.len()),
            )
        })?;
        let Lowered::String(value) = arg else {
            return Err(unsupported(
                "PrimitiveCall",
                "char_length only supports String arguments in native lowering",
            ));
        };
        let len = i64::try_from(value.chars().count()).map_err(|_| {
            unsupported(
                "PrimitiveCall",
                "char_length result does not fit the runtime Int representation",
            )
        })?;
        Ok(Lowered::Int {
            value: builder.ins().iconst(types::I64, len),
            known: Some(len),
        })
    }

    fn emit_result(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
    ) -> Result<(cranelift_codegen::ir::Value, ResultDecoder), CraneliftBackendError> {
        if self.process_object {
            let _authority = self.mint_terminal_answer_authority()?;
            let value = Self::unwrap_terminal_ret(value);
            let value = match value {
                Lowered::ProcessExitStatus { value } => value,
                value => self.emit_process_exit_status(builder, value),
            };
            return Ok((value, ResultDecoder::ProcessStatus));
        }
        match value {
            Lowered::Int { value, known } => {
                let tag = self.native_int_tag(builder, value, known)?;
                let arena = self.native_int_arena.ok_or_else(|| {
                    unsupported("NativeResult", "Int result has no invocation arena")
                })?;
                let export = self.native_int_export.ok_or_else(|| {
                    unsupported("NativeResult", "Int result has no export support function")
                })?;
                #[cfg(test)]
                if self.native_int_mutation == NativeIntLoweringMutation::SuppressTerminalExport {
                    return Ok((value, ResultDecoder::Int));
                }
                let call = builder.ins().call(export, &[arena, tag, value]);
                Self::require_i64(builder, builder.inst_results(call)[0], 0);
                #[cfg(test)]
                if self.native_int_mutation == NativeIntLoweringMutation::CorruptTerminalExport {
                    let invalid = builder.ins().iconst(types::I64, 7);
                    builder.ins().store(
                        MemFlags::trusted(),
                        invalid,
                        arena,
                        crate::native_int_clif::ARENA_FINAL_TAG,
                    );
                }
                Ok((value, ResultDecoder::Int))
            }
            Lowered::Bool { value, .. } => Ok((value, ResultDecoder::Bool)),
            value => {
                let ground = self.ground_value(value)?;
                let token = self.intern_result(ground);
                Ok((
                    builder.ins().iconst(types::I64, token),
                    ResultDecoder::Table,
                ))
            }
        }
    }

    fn emit_process_exit_status(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: Lowered,
    ) -> cranelift_codegen::ir::Value {
        let Lowered::Constructor { constructor, args } = value else {
            return builder.ins().iconst(types::I64, -2);
        };
        if constructor == self.process_symbols.exit_success {
            return if args.is_empty() {
                builder.ins().iconst(types::I64, 0)
            } else {
                builder.ins().iconst(types::I64, -2)
            };
        }
        if constructor != self.process_symbols.exit_failure {
            return builder.ins().iconst(types::I64, -2);
        }
        let Ok([payload]) = <Vec<Lowered> as TryInto<[Lowered; 1]>>::try_into(args) else {
            return builder.ins().iconst(types::I64, -3);
        };
        let Lowered::Int { known, .. } = &payload else {
            return builder.ins().iconst(types::I64, -3);
        };
        if let Some(code) = *known {
            let mapping = crate::process_exit_status(crate::ProcessExitCode::Failure(code));
            return builder.ins().iconst(
                types::I64,
                if mapping.trap_report.is_some() {
                    -3
                } else {
                    i64::from(mapping.status)
                },
            );
        }
        let Ok((value, valid_int)) = self.narrow_native_int_u64(builder, &payload) else {
            return builder.ins().iconst(types::I64, -3);
        };
        let zero = builder.ins().iconst(types::I64, 0);
        let one = builder.ins().iconst(types::I64, 1);
        let max = builder.ins().iconst(types::I64, 255);
        let malformed = builder.ins().iconst(types::I64, -3);
        let is_zero =
            builder
                .ins()
                .icmp(cranelift_codegen::ir::condcodes::IntCC::Equal, value, zero);
        let positive = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThan,
            value,
            zero,
        );
        let within_max = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            value,
            max,
        );
        let valid = builder.ins().band(valid_int, positive);
        let valid = builder.ins().band(valid, within_max);
        let nonzero = builder.ins().select(valid, value, malformed);
        builder.ins().select(is_zero, one, nonzero)
    }

    fn ground_value(
        &mut self,
        value: Lowered,
    ) -> Result<RuntimeGroundValue, CraneliftBackendError> {
        match value {
            Lowered::Int {
                known: Some(value), ..
            } => Ok(RuntimeGroundValue::Int((value).into())),
            Lowered::Int { known: None, .. } => Err(unsupported(
                "Result",
                "native aggregate result contains a non-constant Int field",
            )),
            Lowered::Bool {
                known: Some(value), ..
            } => Ok(RuntimeGroundValue::Bool(value)),
            Lowered::Bool { known: None, .. } => Err(unsupported(
                "Result",
                "native aggregate result contains a non-constant Bool field",
            )),
            Lowered::ProcessExitStatus { .. } => Err(unsupported(
                "Result",
                "process exit status cannot escape a native process call",
            )),
            Lowered::Bytes(value) => Ok(RuntimeGroundValue::Bytes(value)),
            Lowered::BorrowedNativeValue { .. }
            | Lowered::BorrowedOption { .. }
            | Lowered::ResponseBytes { .. }
            | Lowered::CapabilityToken { .. }
            | Lowered::ResourceToken { .. }
            | Lowered::BoundedNat(_)
            | Lowered::StructuralNat(_)
            | Lowered::HostResult { .. }
            | Lowered::DynamicConstructor(_) => Err(unsupported(
                "Result",
                "borrowed ingress values cannot escape the native call",
            )),
            Lowered::String(value) => Ok(RuntimeGroundValue::String(value)),
            Lowered::Constructor { constructor, args } => Ok(RuntimeGroundValue::Constructor {
                constructor,
                args: args
                    .into_iter()
                    .map(|arg| self.ground_value(arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            Lowered::Record { fields } => Ok(RuntimeGroundValue::Record {
                fields: fields
                    .into_iter()
                    .map(|(name, value)| Ok((name, self.ground_value(value)?)))
                    .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
            }),
            Lowered::Closure { .. } | Lowered::DeclarationClosure { .. } => Err(unsupported(
                "Closure",
                "closures are callable but not observable ground values in native lowering",
            )),
            Lowered::ComputationalRecursorClosure { .. } => Err(unsupported(
                "ComputationalMatch",
                "recursive hypotheses are callable but not observable ground values",
            )),
            Lowered::RecursiveBackedge => Err(unsupported(
                "DeclarationRef",
                "a recursive CFG edge cannot escape as a ground value",
            )),
            Lowered::Trap(trap) => Err(unsupported(
                "Trap",
                format!("trap result must be reported as trapped: {}", trap.message),
            )),
        }
    }

    fn intern_result(&mut self, ground: RuntimeGroundValue) -> i64 {
        let token = self.next_token;
        self.next_token += 1;
        self.result_table.insert(token, ground);
        token
    }
}
fn same_recursive_argument_shapes(left: &[Lowered], right: &[Lowered]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (Lowered::Int { .. }, Lowered::Int { .. })
                | (Lowered::Bool { .. }, Lowered::Bool { .. })
                | (Lowered::ProcessExitStatus { .. }, Lowered::ProcessExitStatus { .. })
                | (Lowered::CapabilityToken { .. }, Lowered::CapabilityToken { .. })
                | (Lowered::ResourceToken { .. }, Lowered::ResourceToken { .. })
                | (Lowered::BoundedNat(_), Lowered::BoundedNat(_))
                | (Lowered::StructuralNat(_), Lowered::StructuralNat(_))
                | (Lowered::ResponseBytes { .. }, Lowered::ResponseBytes { .. })
                | (Lowered::BorrowedNativeValue { .. }, Lowered::BorrowedNativeValue { .. }) => {
                    true
                }
                (Lowered::Bytes(left), Lowered::Bytes(right)) => left == right,
                (Lowered::String(left), Lowered::String(right)) => left == right,
                (
                    Lowered::Constructor {
                        constructor: left_constructor,
                        args: left_args,
                    },
                    Lowered::Constructor {
                        constructor: right_constructor,
                        args: right_args,
                    },
                ) => {
                    left_constructor == right_constructor
                        && same_recursive_argument_shapes(left_args, right_args)
                }
                (Lowered::Record { fields: left }, Lowered::Record { fields: right }) => {
                    left.len() == right.len()
                        && left
                            .iter()
                            .zip(right)
                            .all(|((left_name, left), (right_name, right))| {
                                left_name == right_name
                                    && same_recursive_argument_shapes(
                                        std::slice::from_ref(left),
                                        std::slice::from_ref(right),
                                    )
                            })
                }
                _ => false,
            })
}
fn lowered_value_kind(value: &Lowered) -> &'static str {
    match value {
        Lowered::Int { .. } => "Int",
        Lowered::Bool { .. } => "Bool",
        Lowered::ProcessExitStatus { .. } => "ProcessExitStatus",
        Lowered::CapabilityToken { .. } => "CapabilityToken",
        Lowered::ResourceToken { .. } => "ResourceToken",
        Lowered::BoundedNat(_) => "BoundedNat",
        Lowered::StructuralNat(_) => "StructuralNat",
        Lowered::ResponseBytes { .. } => "ResponseBytes",
        Lowered::HostResult { .. } => "HostResult",
        Lowered::DynamicConstructor(_) => "DynamicConstructor",
        Lowered::Bytes(_) => "Bytes",
        Lowered::BorrowedNativeValue { .. } => "BorrowedNativeValue",
        Lowered::BorrowedOption { .. } => "BorrowedOption",
        Lowered::String(_) => "String",
        Lowered::Constructor { .. } => "Constructor",
        Lowered::Record { .. } => "Record",
        Lowered::Closure { .. } => "Closure",
        Lowered::DeclarationClosure { .. } => "DeclarationClosure",
        Lowered::ComputationalRecursorClosure { .. } => "ComputationalRecursorClosure",
        Lowered::RecursiveBackedge => "RecursiveBackedge",
        Lowered::Trap(_) => "Trap",
    }
}
fn append_recursive_argument_values(
    builder: &mut FunctionBuilder<'_>,
    values: &[Lowered],
    output: &mut Vec<cranelift_codegen::ir::Value>,
    native_int_tags: &BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
) -> Result<(), CraneliftBackendError> {
    for value in values {
        match value {
            Lowered::Int { value, known } => {
                let tag = match native_int_tags.get(value).copied() {
                    Some(tag) => tag,
                    None if known.is_some() => builder
                        .ins()
                        .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64),
                    None => {
                        return Err(unsupported(
                            "DeclarationRef",
                            "recursive Int argument lost its two-word tag transport",
                        ));
                    }
                };
                output.push(tag);
                output.push(*value);
            }
            Lowered::Bool { value, .. }
            | Lowered::ProcessExitStatus { value }
            | Lowered::CapabilityToken { value }
            | Lowered::ResourceToken { value } => output.push(*value),
            Lowered::BoundedNat(nat) => output.push(nat.value),
            Lowered::StructuralNat(nat) => output.push(nat.value),
            Lowered::ResponseBytes { pointer, len } => {
                output.push(*pointer);
                output.push(*len);
            }
            Lowered::BorrowedNativeValue { pointer } => output.push(*pointer),
            Lowered::Bytes(_) | Lowered::String(_) => {}
            Lowered::Constructor { args, .. } => {
                append_recursive_argument_values(builder, args, output, native_int_tags)?;
            }
            Lowered::Record { fields } => {
                for (_, field) in fields {
                    append_recursive_argument_values(
                        builder,
                        std::slice::from_ref(field),
                        output,
                        native_int_tags,
                    )?;
                }
            }
            _ => {
                return Err(unsupported(
                    "DeclarationRef",
                    "recursive declaration argument has an unsupported native representation",
                ));
            }
        }
    }
    Ok(())
}
fn rebuild_recursive_argument(
    template: &Lowered,
    values: &mut impl Iterator<Item = cranelift_codegen::ir::Value>,
    native_int_tags: &mut BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
) -> Result<Lowered, CraneliftBackendError> {
    let next = |values: &mut dyn Iterator<Item = cranelift_codegen::ir::Value>| {
        values.next().ok_or_else(|| {
            unsupported(
                "DeclarationRef",
                "recursive declaration loop parameter shape is truncated",
            )
        })
    };
    Ok(match template {
        Lowered::Int { .. } => {
            let tag = next(values)?;
            let value = next(values)?;
            native_int_tags.insert(value, tag);
            Lowered::Int { value, known: None }
        }
        Lowered::Bool { .. } => Lowered::Bool {
            value: next(values)?,
            known: None,
        },
        Lowered::ProcessExitStatus { .. } => Lowered::ProcessExitStatus {
            value: next(values)?,
        },
        Lowered::CapabilityToken { .. } => Lowered::CapabilityToken {
            value: next(values)?,
        },
        Lowered::ResourceToken { .. } => Lowered::ResourceToken {
            value: next(values)?,
        },
        Lowered::BoundedNat(_) => {
            Lowered::BoundedNat(BoundedNatV1::derived_from_validated(next(values)?))
        }
        Lowered::StructuralNat(_) => Lowered::StructuralNat(StructuralNatV1 {
            value: next(values)?,
        }),
        Lowered::ResponseBytes { .. } => Lowered::ResponseBytes {
            pointer: next(values)?,
            len: next(values)?,
        },
        Lowered::BorrowedNativeValue { .. } => Lowered::BorrowedNativeValue {
            pointer: next(values)?,
        },
        Lowered::Bytes(bytes) => Lowered::Bytes(bytes.clone()),
        Lowered::String(string) => Lowered::String(string.clone()),
        Lowered::Constructor { constructor, args } => Lowered::Constructor {
            constructor: constructor.clone(),
            args: args
                .iter()
                .map(|arg| rebuild_recursive_argument(arg, values, native_int_tags))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Lowered::Record { fields } => Lowered::Record {
            fields: fields
                .iter()
                .map(|(name, value)| {
                    Ok((
                        name.clone(),
                        rebuild_recursive_argument(value, values, native_int_tags)?,
                    ))
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
        },
        _ => {
            return Err(unsupported(
                "DeclarationRef",
                "recursive declaration argument has an unsupported native representation",
            ));
        }
    })
}
fn expect_two_args(
    symbol: &'static str,
    args: Vec<Lowered>,
) -> Result<(Lowered, Lowered), CraneliftBackendError> {
    let [lhs, rhs]: [Lowered; 2] = args.try_into().map_err(|args: Vec<Lowered>| {
        unsupported(
            "PrimitiveCall",
            format!("{symbol} expects 2 args, got {}", args.len()),
        )
    })?;
    Ok((lhs, rhs))
}
fn borrowed_constructor_identity(
    symbols: &crate::NativeProcessSymbols,
    symbol: &str,
) -> Option<(i64, usize)> {
    if symbol == symbols.process_input {
        Some((1, 3))
    } else if symbol == symbols.list_nil {
        Some((2, 0))
    } else if symbol == symbols.list_cons {
        Some((3, 2))
    } else if symbol == symbols.prod {
        Some((4, 2))
    } else {
        None
    }
}

#[cfg(test)]
thread_local! {
    static PX8J_SOURCE_TRACE: std::cell::RefCell<Vec<Px8jSourceTraceEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static PX8J_DELETE_OWNED_SELECTED_SCOPE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
    pub(super) static PX8TR_TRAP_PROVENANCE: std::cell::RefCell<Vec<Px8trTrapProvenanceEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    pub(super) static PX8TR_DISABLE_DEFORESTED_ANSWER_ROUTE: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}
#[cfg(test)]
thread_local! {
    pub(crate) static NATIVE_INT_LOWERING_MUTATION: std::cell::Cell<NativeIntLoweringMutation> =
        const { std::cell::Cell::new(NativeIntLoweringMutation::Exact) };
}
#[cfg(any(test, feature = "px8-ds-test-support"))]
thread_local! {
    static PX8DS_RETIRED_FLAT_ORDER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
