//! Factored, pre-emission native transition planner.
//!
//! Node code identity is `(transition kind, static node id)` and edge code
//! identity is `(edge kind, static edge id)`. Dynamic environment,
//! continuation, cleanup, source, and affine state travels as constant-width
//! IDs into hash-consed persistent stores.

mod abi;
mod semantic_ir;

#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use super::{
    backend, unsupported, BackendFailure, CraneliftBackendError, RuntimeDeclaration,
    RuntimeDeclarationKind,
};
use crate::boundary_value::{BoundaryClass, BoundaryReferentOwner, BoundaryTag};
use crate::{RuntimeExpr, RuntimePartiality, RuntimeSymbol, RuntimeTrap, RuntimeTrapCode};
use abi::{build_abi_plane, extend_static_callable_abi, AbiPlane};
use semantic_ir::{
    build_semantic_plane, build_synthesized_constructor_inventory, SemanticMaterialArena,
    SemanticPlane, SemanticSourceKind, SemanticSourceSeed,
};

// ⭐ `D1`'s capability surface. The two identity types cross into
// `crate::cranelift_backend` so `lowering` can hold and compare them; ⛔
// `SemanticPlane`, `SemanticMaterialArena` and the `names` arena stay on the
// `use` above, visible only inside this planner. Widening either of those to
// serve a consumer is the move `§2d` forbids.
pub(in crate::cranelift_backend) use abi::{
    AbiCaptureProvenance, AbiCarrier, AbiFrameHeader, AbiOwnership, AbiProcessParameter,
    AbiRootIngress, AbiSchedulingIngress, AbiSlot, AbiSlotKind, AbiStorageOwner, AbiUnitDefinition,
};
#[cfg(test)]
pub(in crate::cranelift_backend) use semantic_ir::with_last_io_error_role_omitted;
pub(in crate::cranelift_backend) use semantic_ir::{
    ConstructorIdentity, FieldIdentity, PredeclaredFunctionId, StaticOriginId,
    SynthesizedConstructorRole, SynthesizedFixedConstructorRole, SynthesizedIoErrorRole,
};

pub(super) const MAX_HELPERS_PER_STATIC_SOURCE: usize = 8;

#[cfg(test)]
thread_local! {
    static ACTIVE_RECURSIVE_LOWERING_FRAMES: Cell<usize> = const { Cell::new(0) };
    static MAX_RECURSIVE_LOWERING_FRAMES: Cell<usize> = const { Cell::new(0) };
    static DENY_LOWERING_BOUNDARY_USE_ISSUANCE: Cell<bool> = const { Cell::new(false) };
}

/// Run one control with planner issuance for lowering-only boundary uses denied.
///
/// A production lowering path that still reaches its boundary walk under this
/// mutation has bypassed the planner-issued token ledger.
#[cfg(test)]
pub(in crate::cranelift_backend) fn with_lowering_boundary_use_issuance_denied<R>(
    control: impl FnOnce() -> R,
) -> R {
    struct Reset(bool);

    impl Drop for Reset {
        fn drop(&mut self) {
            DENY_LOWERING_BOUNDARY_USE_ISSUANCE.with(|denied| denied.set(self.0));
        }
    }

    let previous = DENY_LOWERING_BOUNDARY_USE_ISSUANCE.with(|denied| denied.replace(true));
    let _reset = Reset(previous);
    control()
}

/// Test-only observation of the actual `plan_expr` call stack.
///
/// The guard is entered inside `plan_expr`, so `Drop` runs on every `?` return
/// as well as the ordinary path. This measures production recursion rather than
/// deriving a proxy from bracket depth or expression-node counts.
#[cfg(test)]
struct RecursiveLoweringFrameGuard;

#[cfg(test)]
impl RecursiveLoweringFrameGuard {
    fn enter() -> Self {
        ACTIVE_RECURSIVE_LOWERING_FRAMES.with(|active| {
            let depth = active
                .get()
                .checked_add(1)
                .expect("recursive lowering frame count fits usize");
            active.set(depth);
            MAX_RECURSIVE_LOWERING_FRAMES.with(|maximum| {
                maximum.set(maximum.get().max(depth));
            });
        });
        Self
    }
}

#[cfg(test)]
impl Drop for RecursiveLoweringFrameGuard {
    fn drop(&mut self) {
        ACTIVE_RECURSIVE_LOWERING_FRAMES.with(|active| {
            active.set(
                active
                    .get()
                    .checked_sub(1)
                    .expect("recursive lowering frame guard is balanced"),
            );
        });
    }
}

#[cfg(test)]
fn reset_recursive_lowering_frame_count() {
    ACTIVE_RECURSIVE_LOWERING_FRAMES.with(|active| active.set(0));
    MAX_RECURSIVE_LOWERING_FRAMES.with(|maximum| maximum.set(0));
}

#[cfg(test)]
fn max_recursive_lowering_frame_count() -> usize {
    MAX_RECURSIVE_LOWERING_FRAMES.with(Cell::get)
}

/// ⭐ The dual result of planning one expression.
///
/// One `StaticNodeId` was previously made to mean two different things:
///
/// - **`entry`** — the first node the transfer graph *schedules* for the
///   expression;
/// - **`occurrence`** — the node on which `SemanticSourceSeed::expression`
///   registered that `RuntimeExpr`, and from which its positional child-origin
///   record is read.
///
/// They coincide for every ordinary form and **deliberately do not** for
/// `ComputationalMatch`, whose occurrence is registered on its
/// `SourceReturnResume` while the parent must still schedule its scrutinee.
/// Returning one value for both made a parent record the scrutinee's identity as
/// its child's origin — a category error, not an off-by-one.
///
/// ⛔ **The two fields have disjoint consumers, and that is the whole mechanism.**
/// Transfer topology consumes **only `.entry`**; source correspondence consumes
/// **only `.occurrence`**. This adds no node, no origin, no search and no
/// arithmetic: both values are outputs of the same recursive visit, and
/// `occurrence` is the origin already assigned to the already-existing semantic
/// seed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlannedExpr {
    entry: StaticNodeId,
    occurrence: StaticOriginId,
}

/// The occurrence origin of a node the planner has just allocated a semantic
/// seed for.
///
/// ⛔ Formed **only** inside the planner. `StaticOriginId`'s ordinal is
/// planner-private precisely so no consumer outside this module can mint one, and
/// this is the single function that does.
fn origin_of(node: StaticNodeId) -> StaticOriginId {
    StaticOriginId(node.0)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct StaticNodeId(u32);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct StaticEdgeId(u32);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct StaticSourceId(u32);
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
struct PersistentNodeId(u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
enum TransitionKind {
    Terminal,
    TrapTerminal,
    Evaluate,
    Sequence,
    Branch,
    CaseTest,
    ClosureBody,
    ProducerWrapper,
    SourceReturnResume,
    ProducerTail,
    CompletedTail,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
enum EdgeKind {
    Continue,
    Select,
    Reject,
    InvokeProducerWrapper,
    SourceReturnOwnedResume,
    InvokeProducerTail,
    CompleteProducerTail,
    StaticBody,
    DeclarationCall,
    Trap,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
enum StoreKind {
    Syntax,
    Environment,
    Continuation,
    Path,
    Cleanup,
    Affine,
    SourceReturn,
}

/// The complete fixed-width helper identity. It contains no activation or
/// occurrence path.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(C)]
enum PlannedHelperKey {
    Node(TransitionKind, StaticNodeId),
    Edge(EdgeKind, StaticEdgeId),
}

impl PlannedHelperKey {
    const fn node(transition: TransitionKind, node: StaticNodeId) -> Self {
        Self::Node(transition, node)
    }

    const fn edge(kind: EdgeKind, edge: StaticEdgeId) -> Self {
        Self::Edge(kind, edge)
    }
}

/// Fixed ABI shape carried between helpers. Every field is one dense ID.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
struct DynamicActivationFrame {
    syntax: PersistentNodeId,
    environment: PersistentNodeId,
    normal: PersistentNodeId,
    abrupt: PersistentNodeId,
    path: PersistentNodeId,
    cleanup: PersistentNodeId,
    affine: PersistentNodeId,
    source_return: PersistentNodeId,
}

/// The sole persistent-node schema. `local` and `aux` are dense IDs/tags, and
/// `child` is the shared suffix. No vector or recursive payload is inline.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(C)]
struct PersistentStoreNode {
    kind: StoreKind,
    local: u32,
    aux: u32,
    child: PersistentNodeId,
}

#[derive(Clone, Copy, Debug)]
struct StaticNode {
    id: StaticNodeId,
    transition: TransitionKind,
    owner: StaticSourceId,
    frame: DynamicActivationFrame,
}

#[derive(Clone, Copy, Debug)]
struct StaticEdge {
    id: StaticEdgeId,
    from: StaticNodeId,
    to: StaticNodeId,
    kind: EdgeKind,
}

/// Exact graph evidence is deliberately out of line and keyed by one edge ID.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct EdgeEvidence {
    edge: u32,
    owner: StaticSourceId,
    from: StaticNodeId,
    to: StaticNodeId,
    kind: EdgeKind,
}

#[derive(Clone, Copy)]
struct PlanContext {
    environment: PersistentNodeId,
    continuation: PersistentNodeId,
    path: PersistentNodeId,
    cleanup: PersistentNodeId,
    affine: PersistentNodeId,
    source_return: PersistentNodeId,
}

/// One planned source occurrence: the borrowed term, paired with the origin the
/// planner gave it in the very same visit.
///
/// ⭐ The origin is stored **beside** the term rather than left implicit in the
/// table position. A dense table whose entries only ever say "whatever lives at
/// this index" cannot detect an entry written at the wrong index; storing the
/// origin makes that failure observable, and `source_occurrence` rejects it
/// instead of returning a plausible wrong body.
#[derive(Clone, Copy)]
struct PlannedOccurrence<'src> {
    static_origin: StaticOriginId,
    expr: &'src RuntimeExpr,
}

/// The complete, pre-emission result representation of one source join.
///
/// This is deliberately a two-way type rather than a phase bit threaded through
/// lowering.  In particular, lowering cannot add a third representation or
/// select one from an emitted predecessor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum JoinResultRepresentation {
    NativeScalarPair,
    CarrierWord,
}

/// Move-only evidence that a particular source join was planned.
///
/// Fields and construction stay in the planner.  Lowering can consume the
/// token and inspect the closed representation, but cannot manufacture a token
/// from an origin or a diagnostic label.
#[derive(Debug)]
pub(in crate::cranelift_backend) struct JoinPlanToken {
    pub(in crate::cranelift_backend) origin: StaticOriginId,
    pub(in crate::cranelift_backend) representation: JoinResultRepresentation,
    pub(in crate::cranelift_backend) has_continuing_predecessor: bool,
}

/// The closed disposition of one edge that receives a [`LoweringOperand`].
///
/// This is planner authority, not a diagnostic label. Lowering may consume the
/// token for an edge, but cannot manufacture a disposition from the value it
/// happens to observe.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum OperandEdgeDisposition {
    Forwarding,
    CallableCapture,
    StaticCallableElimination,
    SemanticEliminator,
    SpecializedOnlyLeaf,
    EscapeForbidden,
}

/// The exhaustive source-child roles that can carry a lowering operand.
///
/// Repeated roles are distinguished by their checked child position in
/// [`PlannedOperandEdge`]. A new `RuntimeExpr` variant must extend the
/// wildcard-free derivation below before the planner compiles.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum SourceOperandRole {
    WrapperBody,
    LetValue,
    LetBody,
    IfScrutinee,
    IfArm,
    PrimitiveArgument,
    ConstructArgument,
    MatchScrutinee,
    MatchArm,
    RecordField,
    ProjectRecord,
    LexicalCapture,
    CallCallee,
    CallArgument,
    EffectCapability,
    EffectArgument,
}

fn role_only_disposition(
    role: SourceOperandRole,
) -> Result<OperandEdgeDisposition, CraneliftBackendError> {
    Ok(match role {
        SourceOperandRole::WrapperBody
        | SourceOperandRole::LetValue
        | SourceOperandRole::LetBody
        | SourceOperandRole::IfArm
        | SourceOperandRole::MatchArm => OperandEdgeDisposition::Forwarding,
        SourceOperandRole::LexicalCapture => OperandEdgeDisposition::CallableCapture,
        SourceOperandRole::IfScrutinee
        | SourceOperandRole::PrimitiveArgument
        | SourceOperandRole::ConstructArgument
        | SourceOperandRole::MatchScrutinee
        | SourceOperandRole::RecordField
        | SourceOperandRole::ProjectRecord
        | SourceOperandRole::EffectCapability
        | SourceOperandRole::EffectArgument => OperandEdgeDisposition::SemanticEliminator,
        SourceOperandRole::CallCallee => OperandEdgeDisposition::SpecializedOnlyLeaf,
        SourceOperandRole::CallArgument => {
            return Err(planner_error(
                "call-argument disposition requires exact callee, parameter, and use-closure",
            ));
        }
    })
}

/// Lowering-only consumer edges have no positional source child. Keeping them
/// in a separate closed enum prevents a free-form string from becoming a
/// second, unvalidated consumer inventory.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum LoweringOnlyOperandEdge {
    CheckedComputationalIhMarker,
    PendingLetRecursorResidual,
    ProducerCallRecursorResidual,
    DeferredConstructorPrefix,
    ComposedComputationalMatchScrutinee,
    DeferredConstructorField,
    DeferredConstructorTrailingField,
    EliminatorFrameScrutinee,
    SourceMachineCallee,
    SourceCallRecursorResidual,
    RecursiveSourceDeclarationArgument,
    JoinArm,
    DirectCallRecursorResidual,
    RecursiveDeclarationArgument,
    DeclarationCaptureSpecialization,
    CallableCapsuleEscape,
    #[cfg(test)]
    TestFixtureResult,
}

/// Exact identity of one planned owner/phase crossing.
///
/// A source crossing names both endpoints and the structural child position.
/// A synthesized crossing names a planner-interned identity allocated while the
/// generated-unit fixed point is still open.  There is deliberately no
/// anonymous form: a lowering consumer cannot substitute a diagnostic label
/// for the crossing it is about to perform.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum BoundaryUseIdentity {
    Source {
        parent: StaticOriginId,
        child: StaticOriginId,
        position: u32,
    },
    Synthesized(u32),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum BoundaryUsePhase {
    SpecializedValue,
    CallableEnvironment,
    OperationalCarrier,
    Eliminated,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum BoundaryUseOperation {
    Forward,
    Capture,
    Inspect,
    Eliminate,
    RejectEscape,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum BoundaryUseNeed {
    PreserveValue,
    PreserveCallableProvenance,
    ReadSpecializedTemplate,
    ObserveSemanticValue,
    ObserveEffectCapability,
    ObserveEffectBytes,
    ObserveEffectConstructorTag,
    ObserveEffectExactIntU64,
    ObserveEffectResource,
    EliminateRuntimeValue,
    ForbidCallableCapsule,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum BoundaryUseAvail {
    Value,
    CallableProvenance,
    SpecializedTemplate,
    SemanticObservation,
    NoRuntimeValue,
    FailClosed,
}

fn non_semantic_boundary_contract(
    disposition: OperandEdgeDisposition,
) -> (
    BoundaryUsePhase,
    BoundaryUseOperation,
    BoundaryUseNeed,
    BoundaryUseAvail,
) {
    match disposition {
        OperandEdgeDisposition::Forwarding => (
            BoundaryUsePhase::OperationalCarrier,
            BoundaryUseOperation::Forward,
            BoundaryUseNeed::PreserveValue,
            BoundaryUseAvail::Value,
        ),
        OperandEdgeDisposition::CallableCapture => (
            BoundaryUsePhase::CallableEnvironment,
            BoundaryUseOperation::Capture,
            BoundaryUseNeed::PreserveCallableProvenance,
            BoundaryUseAvail::CallableProvenance,
        ),
        OperandEdgeDisposition::StaticCallableElimination => (
            BoundaryUsePhase::Eliminated,
            BoundaryUseOperation::Eliminate,
            BoundaryUseNeed::EliminateRuntimeValue,
            BoundaryUseAvail::NoRuntimeValue,
        ),
        OperandEdgeDisposition::SemanticEliminator => (
            BoundaryUsePhase::OperationalCarrier,
            BoundaryUseOperation::Inspect,
            BoundaryUseNeed::ObserveSemanticValue,
            BoundaryUseAvail::SemanticObservation,
        ),
        OperandEdgeDisposition::SpecializedOnlyLeaf => (
            BoundaryUsePhase::SpecializedValue,
            BoundaryUseOperation::Inspect,
            BoundaryUseNeed::ReadSpecializedTemplate,
            BoundaryUseAvail::SpecializedTemplate,
        ),
        OperandEdgeDisposition::EscapeForbidden => (
            BoundaryUsePhase::Eliminated,
            BoundaryUseOperation::RejectEscape,
            BoundaryUseNeed::ForbidCallableCapsule,
            BoundaryUseAvail::FailClosed,
        ),
    }
}

/// The exact semantic meaning of one admitted host-effect child.
///
/// `EffectArgument` remains source-inventory data. This closed key is the
/// consumer authority: equal structural roles may require different emitted
/// observations, and the operation plus semantic seat makes that distinction
/// before a disposition is selected.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum EffectSemanticSeat {
    Capability,
    ConsoleStream,
    Bytes,
    CreatePolicy,
    OpenMode,
    ExactIntU64,
    Resource,
}

impl EffectSemanticSeat {
    fn need(self) -> BoundaryUseNeed {
        match self {
            Self::Capability => BoundaryUseNeed::ObserveEffectCapability,
            Self::ConsoleStream | Self::CreatePolicy | Self::OpenMode => {
                BoundaryUseNeed::ObserveEffectConstructorTag
            }
            Self::Bytes => BoundaryUseNeed::ObserveEffectBytes,
            Self::ExactIntU64 => BoundaryUseNeed::ObserveEffectExactIntU64,
            Self::Resource => BoundaryUseNeed::ObserveEffectResource,
        }
    }
}

fn effect_semantic_seats(
    operation: ken_host::HostOpV1,
    has_capability: bool,
) -> Result<Option<Vec<EffectSemanticSeat>>, CraneliftBackendError> {
    use ken_host::HostOpV1 as Op;
    use EffectSemanticSeat as Seat;

    let (expects_capability, arguments): (bool, &[Seat]) = match operation {
        Op::ConsoleWrite => (false, &[Seat::ConsoleStream, Seat::Bytes]),
        Op::ConsoleFlush | Op::ConsoleIsTerminal => (false, &[Seat::ConsoleStream]),
        Op::FsReadFile => (true, &[Seat::Bytes]),
        Op::FsWriteFile => (true, &[Seat::Bytes, Seat::CreatePolicy, Seat::Bytes]),
        Op::FsChangeMode => (true, &[Seat::Bytes, Seat::ExactIntU64]),
        Op::FsOpen => (true, &[Seat::Bytes, Seat::OpenMode]),
        Op::FsHandleMetadata | Op::ResourceRelease => (false, &[Seat::Resource]),
        Op::FsReadAt => (
            false,
            &[
                Seat::Resource,
                Seat::ExactIntU64,
                Seat::Resource,
                Seat::ExactIntU64,
                Seat::ExactIntU64,
            ],
        ),
        Op::FsWriteAt => (
            false,
            &[
                Seat::Resource,
                Seat::ExactIntU64,
                Seat::Resource,
                Seat::ExactIntU64,
                Seat::ExactIntU64,
                Seat::Resource,
            ],
        ),
        Op::BufferAllocate => (false, &[Seat::ExactIntU64]),
        Op::BufferFreeze => (
            false,
            &[
                Seat::Resource,
                Seat::ExactIntU64,
                Seat::ExactIntU64,
                Seat::Resource,
            ],
        ),
        Op::ConsoleRead
        | Op::ClockWallNow
        | Op::ClockMonotonicNow
        | Op::ClockSleepUntil
        | Op::FsAppendFile
        | Op::FsMetadata
        | Op::FsReadDirectory
        | Op::FsCreateDirectory
        | Op::FsRemoveFile
        | Op::FsRemoveDirectory
        | Op::FsRename
        | Op::EntropyRandomBytes => {
            return Ok(None);
        }
    };
    if has_capability != expects_capability {
        return Err(planner_error(
            "host operation capability base does not match its semantic-seat contract",
        ));
    }
    let mut seats = Vec::with_capacity(arguments.len() + usize::from(has_capability));
    if has_capability {
        seats.push(Seat::Capability);
    }
    seats.extend_from_slice(arguments);
    Ok(Some(seats))
}

fn operand_edge_contract(
    disposition: OperandEdgeDisposition,
    effect_seat: Option<EffectSemanticSeat>,
) -> (
    BoundaryUsePhase,
    BoundaryUseOperation,
    BoundaryUseNeed,
    BoundaryUseAvail,
) {
    match effect_seat {
        Some(seat) => (
            BoundaryUsePhase::OperationalCarrier,
            BoundaryUseOperation::Inspect,
            seat.need(),
            BoundaryUseAvail::SemanticObservation,
        ),
        None => non_semantic_boundary_contract(disposition),
    }
}

fn effect_edge_contract(
    expr: &RuntimeExpr,
    position: usize,
    role: SourceOperandRole,
) -> Result<(Option<ken_host::HostOpV1>, Option<EffectSemanticSeat>), CraneliftBackendError> {
    let RuntimeExpr::Effect {
        operation,
        capability,
        args,
        ..
    } = expr
    else {
        return Ok((None, None));
    };
    let Some(seats) = effect_semantic_seats(*operation, capability.is_some())? else {
        return Ok((None, None));
    };
    if seats.len() != args.len() + usize::from(capability.is_some()) {
        return Err(planner_error(
            "host operation semantic-seat population is not exact",
        ));
    }
    let seat = seats
        .get(position)
        .copied()
        .ok_or_else(|| planner_error("effect child has no exact semantic seat"))?;
    let expected_role = if seat == EffectSemanticSeat::Capability {
        SourceOperandRole::EffectCapability
    } else {
        SourceOperandRole::EffectArgument
    };
    if role != expected_role {
        return Err(planner_error(
            "effect semantic seat does not match its source-inventory role",
        ));
    }
    Ok((Some(*operation), Some(seat)))
}

impl LoweringOnlyOperandEdge {
    fn disposition(self) -> OperandEdgeDisposition {
        match self {
            Self::JoinArm => OperandEdgeDisposition::Forwarding,
            Self::CallableCapsuleEscape => OperandEdgeDisposition::EscapeForbidden,
            Self::CheckedComputationalIhMarker
            | Self::PendingLetRecursorResidual
            | Self::ProducerCallRecursorResidual
            | Self::DeferredConstructorPrefix
            | Self::ComposedComputationalMatchScrutinee
            | Self::DeferredConstructorField
            | Self::DeferredConstructorTrailingField
            | Self::EliminatorFrameScrutinee
            | Self::SourceMachineCallee
            | Self::SourceCallRecursorResidual
            | Self::RecursiveSourceDeclarationArgument
            | Self::DirectCallRecursorResidual
            | Self::RecursiveDeclarationArgument
            | Self::DeclarationCaptureSpecialization => OperandEdgeDisposition::SpecializedOnlyLeaf,
            #[cfg(test)]
            Self::TestFixtureResult => OperandEdgeDisposition::SpecializedOnlyLeaf,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::CheckedComputationalIhMarker => "a checked computational-IH marker",
            Self::PendingLetRecursorResidual => "a pending-let recursor residual",
            Self::ProducerCallRecursorResidual => "a recursor residual in a producer call",
            Self::DeferredConstructorPrefix => "a deferred constructor prefix",
            Self::ComposedComputationalMatchScrutinee => "a composed computational-match scrutinee",
            Self::DeferredConstructorField => "a deferred constructor field",
            Self::DeferredConstructorTrailingField => "a deferred constructor's trailing field",
            Self::EliminatorFrameScrutinee => "an eliminator frame's scrutinee",
            Self::SourceMachineCallee => "a source-machine call's callee",
            Self::SourceCallRecursorResidual => "a recursor residual in a source call",
            Self::RecursiveSourceDeclarationArgument => "a recursive source-declaration argument",
            Self::JoinArm => "a planned join arm",
            Self::DirectCallRecursorResidual => "a recursor residual in a direct call",
            Self::RecursiveDeclarationArgument => "a recursive declaration argument",
            Self::DeclarationCaptureSpecialization => "a recursive-descent declaration capture",
            Self::CallableCapsuleEscape => "a whole callable capsule",
            #[cfg(test)]
            Self::TestFixtureResult => "a test fixture result",
        }
    }
}

/// Move-only evidence that one exact consumer edge has a planner-owned
/// disposition. Construction remains in this module.
#[derive(Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct OperandEdgeToken {
    disposition: OperandEdgeDisposition,
    label: &'static str,
    identity: BoundaryUseIdentity,
    producer_owner: Option<PredeclaredFunctionId>,
    consumer_owner: Option<PredeclaredFunctionId>,
    producer_phase: BoundaryUsePhase,
    consumer_phase: BoundaryUsePhase,
    operation: BoundaryUseOperation,
    need: BoundaryUseNeed,
    avail: BoundaryUseAvail,
    effect_operation: Option<ken_host::HostOpV1>,
    effect_seat: Option<EffectSemanticSeat>,
}

impl OperandEdgeToken {
    pub(in crate::cranelift_backend) fn disposition(&self) -> OperandEdgeDisposition {
        self.disposition
    }

    pub(in crate::cranelift_backend) fn label(&self) -> &'static str {
        self.label
    }

    pub(in crate::cranelift_backend) fn identity(&self) -> BoundaryUseIdentity {
        self.identity
    }

    pub(in crate::cranelift_backend) fn need(&self) -> BoundaryUseNeed {
        self.need
    }

    pub(in crate::cranelift_backend) fn effect_seat(&self) -> Option<EffectSemanticSeat> {
        self.effect_seat
    }
}

/// Dense identity of one planner-proved callable residual owned by a
/// computational recursor edge.
///
/// Capture values are deliberately absent. The identity is static source
/// provenance only.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct StaticRecursorWorkerResidualId(u32);

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum StaticRecursorCaptureSource {
    Seed(RuntimeSymbol),
    Lexical(StaticOriginId),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum StaticRecursorCaptureLifetime {
    ActivationOwned,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StaticRecursorCaptureProvenance {
    ordinal: u32,
    owner: PredeclaredFunctionId,
    closure_origin: StaticOriginId,
    source: StaticRecursorCaptureSource,
    phase: OperandEdgeDisposition,
}

/// Move-only authority for one ordered capture of one exact static worker.
///
/// The source value is deliberately absent. Lowering must present the capture
/// at `ordinal` and use `edge` to cross a specialized ordinary value.
#[derive(Debug)]
pub(in crate::cranelift_backend) struct StaticRecursorCaptureToken {
    pub(in crate::cranelift_backend) ordinal: u32,
    pub(in crate::cranelift_backend) owner: PredeclaredFunctionId,
    pub(in crate::cranelift_backend) closure_origin: StaticOriginId,
    pub(in crate::cranelift_backend) source_origin: StaticOriginId,
    pub(in crate::cranelift_backend) phase: OperandEdgeDisposition,
    pub(in crate::cranelift_backend) lifetime: StaticRecursorCaptureLifetime,
    pub(in crate::cranelift_backend) edge: OperandEdgeToken,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlannedStaticRecursorWorkerResidual {
    id: StaticRecursorWorkerResidualId,
    parent_origin: StaticOriginId,
    producer_origin: StaticOriginId,
    sibling_position: u32,
    closure_origin: StaticOriginId,
    body_origin: StaticOriginId,
    declared_arity: u32,
    captures: Vec<StaticRecursorCaptureProvenance>,
    disposition: OperandEdgeDisposition,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlannedRecursorBoundaryUse {
    identity: BoundaryUseIdentity,
    owner: PredeclaredFunctionId,
    parent_origin: StaticOriginId,
    sibling_position: u32,
    producer_phase: BoundaryUsePhase,
    consumer_phase: BoundaryUsePhase,
    operation: BoundaryUseOperation,
    need: BoundaryUseNeed,
    disposition: OperandEdgeDisposition,
    avail: BoundaryUseAvail,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlannedLoweringBoundaryUse {
    identity: BoundaryUseIdentity,
    edge: LoweringOnlyOperandEdge,
    origin: StaticOriginId,
    position: u32,
    owner: PredeclaredFunctionId,
    producer_phase: BoundaryUsePhase,
    consumer_phase: BoundaryUsePhase,
    operation: BoundaryUseOperation,
    need: BoundaryUseNeed,
    disposition: OperandEdgeDisposition,
    avail: BoundaryUseAvail,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum PlannedBoundaryUsePath {
    Source {
        parent: StaticOriginId,
        child: StaticOriginId,
        position: u32,
        effect_operation: Option<ken_host::HostOpV1>,
        effect_seat: Option<EffectSemanticSeat>,
    },
    Synthesized {
        origin: StaticOriginId,
        position: u32,
    },
    StaticRecursorWorker {
        parent_origin: StaticOriginId,
        producer_origin: StaticOriginId,
        sibling_position: u32,
        closure_origin: StaticOriginId,
        body_origin: StaticOriginId,
        declared_arity: u32,
        captures: Vec<StaticRecursorCaptureProvenance>,
    },
    StaticRecursorCapture {
        worker_identity: BoundaryUseIdentity,
        residual_id: StaticRecursorWorkerResidualId,
        parent_origin: StaticOriginId,
        producer_origin: StaticOriginId,
        sibling_position: u32,
        closure_origin: StaticOriginId,
        ordinal: u32,
        capture: StaticRecursorCaptureProvenance,
    },
}

/// The single planner-owned population behind every phase-bearing transition.
///
/// Source-child indexes and synthesized-edge indexes remain useful for deriving
/// exact lookup keys, but neither is lowering authority. Tokens are issued only
/// from this closed population after the generated-unit fixed point completes.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlannedBoundaryUse {
    identity: BoundaryUseIdentity,
    path: PlannedBoundaryUsePath,
    producer_owner: PredeclaredFunctionId,
    consumer_owner: PredeclaredFunctionId,
    producer_phase: BoundaryUsePhase,
    consumer_phase: BoundaryUsePhase,
    operation: BoundaryUseOperation,
    need: BoundaryUseNeed,
    disposition: OperandEdgeDisposition,
    avail: BoundaryUseAvail,
}

/// Move-only planner evidence for one exact static-worker residual edge.
///
/// Lowering can inspect this token only after naming the parent and recursive
/// position. It cannot construct or reclassify one.
#[derive(Debug)]
pub(in crate::cranelift_backend) struct StaticRecursorWorkerResidualToken {
    identity: BoundaryUseIdentity,
    pub(in crate::cranelift_backend) id: StaticRecursorWorkerResidualId,
    pub(in crate::cranelift_backend) parent_origin: StaticOriginId,
    pub(in crate::cranelift_backend) producer_origin: StaticOriginId,
    pub(in crate::cranelift_backend) sibling_position: u32,
    pub(in crate::cranelift_backend) closure_origin: StaticOriginId,
    pub(in crate::cranelift_backend) body_origin: StaticOriginId,
    pub(in crate::cranelift_backend) declared_arity: u32,
    pub(in crate::cranelift_backend) capture_count: u32,
    disposition: OperandEdgeDisposition,
}

impl StaticRecursorWorkerResidualToken {
    pub(in crate::cranelift_backend) fn disposition(&self) -> OperandEdgeDisposition {
        self.disposition
    }

    pub(in crate::cranelift_backend) fn identity(&self) -> BoundaryUseIdentity {
        self.identity
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SourceChildRole {
    StaticBody,
    Operand(SourceOperandRole),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PlannedOperandEdge {
    owner: PredeclaredFunctionId,
    producer_owner: PredeclaredFunctionId,
    parent: StaticOriginId,
    child: StaticOriginId,
    position: u32,
    role: SourceOperandRole,
    effect_operation: Option<ken_host::HostOpV1>,
    effect_seat: Option<EffectSemanticSeat>,
    disposition: OperandEdgeDisposition,
    producer_phase: BoundaryUsePhase,
    consumer_phase: BoundaryUsePhase,
    operation: BoundaryUseOperation,
    need: BoundaryUseNeed,
    avail: BoundaryUseAvail,
}

/// Dense identity of one planner-interned static callable specialization.
///
/// Capture values cannot enter this identity. The full key is retained in the
/// planner and consists only of source owners/origins, parameter ordinals,
/// callable body origins, arities, and capture provenance.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct StaticCallableSpecializationId(u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum StaticCallableCapturePhase {
    SpecializedOnly,
    CarrierRequired,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StaticCallableCaptureProvenance {
    owner: PredeclaredFunctionId,
    closure_origin: StaticOriginId,
    capture_origin: StaticOriginId,
    ordinal: u32,
    phase: StaticCallableCapturePhase,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StaticCallableBindingKey {
    parameter_ordinal: u32,
    closure_origin: StaticOriginId,
    body_origin: StaticOriginId,
    declared_arity: u32,
    captures: Vec<StaticCallableCaptureBinding>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum StaticCallableCaptureBinding {
    Value(StaticCallableCaptureProvenance),
    Callable(Box<StaticCallableBindingKey>),
}

impl StaticCallableBindingKey {
    fn lifted_capture_count(&self) -> Result<u32, CraneliftBackendError> {
        self.captures.iter().try_fold(0u32, |total, capture| {
            let count = match capture {
                StaticCallableCaptureBinding::Value(_) => 1,
                StaticCallableCaptureBinding::Callable(binding) => {
                    binding.lifted_capture_count()?
                }
            };
            total.checked_add(count).ok_or_else(|| {
                planner_capacity_error("recursive lifted callable capture count exhausted")
            })
        })
    }
}

fn validate_static_callable_binding(
    plan: &StaticTransitionPlan<'_>,
    binding: &StaticCallableBindingKey,
) -> Result<u32, CraneliftBackendError> {
    let body_owner = plan
        .semantic
        .function_owner(binding.body_origin)?
        .ok_or_else(|| planner_error("static callable body has no owner"))?;
    let body_unit = plan
        .semantic
        .functions
        .get(body_owner.0 as usize)
        .ok_or_else(|| planner_error("static callable body owner is absent"))?;
    if body_unit.origin != binding.body_origin {
        return Err(planner_error(
            "static callable body origin does not name its unit",
        ));
    }
    let mut lifted = 0u32;
    for (ordinal, capture) in binding.captures.iter().enumerate() {
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| planner_capacity_error("static callable capture ordinal exhausted"))?;
        let count = match capture {
            StaticCallableCaptureBinding::Value(capture) => {
                if capture.closure_origin != binding.closure_origin || capture.ordinal != ordinal {
                    return Err(planner_error(
                        "static callable capture provenance is not ordered and closed",
                    ));
                }
                if plan.semantic.function_owner(capture.capture_origin)? != Some(capture.owner) {
                    return Err(planner_error(
                        "static callable capture provenance has the wrong owner",
                    ));
                }
                1
            }
            StaticCallableCaptureBinding::Callable(nested) => {
                if nested.parameter_ordinal != ordinal {
                    return Err(planner_error(
                        "recursive static callable capture is not in declaration order",
                    ));
                }
                validate_static_callable_binding(plan, nested)?
            }
        };
        lifted = lifted.checked_add(count).ok_or_else(|| {
            planner_capacity_error("recursive lifted capture population exhausted")
        })?;
    }
    Ok(lifted)
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StaticCallableSpecializationKey {
    base_owner: PredeclaredFunctionId,
    base_origin: StaticOriginId,
    bindings: Vec<StaticCallableBindingKey>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum StaticCallableArgumentKind {
    Ordinary,
    Erased,
    Direct {
        closure_origin: StaticOriginId,
    },
    Forwarded {
        body_origin: StaticOriginId,
        declared_arity: u32,
    },
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct StaticCallableArgument {
    parameter_ordinal: u32,
    argument_origin: StaticOriginId,
    kind: StaticCallableArgumentKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedStaticCallableSpecialization {
    id: StaticCallableSpecializationId,
    function: PredeclaredFunctionId,
    base_function: PredeclaredFunctionId,
    body_function: PredeclaredFunctionId,
    base_origin: StaticOriginId,
    base_body_origin: StaticOriginId,
    key: StaticCallableSpecializationKey,
    ordinary_parameters: u32,
    lifted_captures: u32,
    kind: PlannedStaticCallableSpecializationKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PlannedStaticCallableSpecializationKind {
    Declaration,
    CallableBody { binding: StaticCallableBindingKey },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedStaticCallableCall {
    caller: PredeclaredFunctionId,
    call_origin: StaticOriginId,
    callee_reference_origin: StaticOriginId,
    specialization: StaticCallableSpecializationId,
    arguments: Vec<StaticCallableArgument>,
}

fn source_child_roles(expr: &RuntimeExpr) -> Vec<SourceChildRole> {
    match expr {
        RuntimeExpr::CheckedJoinSite { .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { .. }
        | RuntimeExpr::CheckedRecursiveInvocation { .. }
        | RuntimeExpr::CheckedComputationalIHSlots { .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { .. } => {
            vec![SourceChildRole::Operand(SourceOperandRole::WrapperBody)]
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => Vec::new(),
        RuntimeExpr::Let { .. } => vec![
            SourceChildRole::Operand(SourceOperandRole::LetValue),
            SourceChildRole::Operand(SourceOperandRole::LetBody),
        ],
        RuntimeExpr::If { .. } => vec![
            SourceChildRole::Operand(SourceOperandRole::IfScrutinee),
            SourceChildRole::Operand(SourceOperandRole::IfArm),
            SourceChildRole::Operand(SourceOperandRole::IfArm),
        ],
        RuntimeExpr::PrimitiveCall { args, .. } => args
            .iter()
            .map(|_| SourceChildRole::Operand(SourceOperandRole::PrimitiveArgument))
            .collect(),
        RuntimeExpr::Construct { args, .. } => args
            .iter()
            .map(|_| SourceChildRole::Operand(SourceOperandRole::ConstructArgument))
            .collect(),
        RuntimeExpr::Match { cases, .. } => {
            let mut roles = Vec::with_capacity(cases.len() + 1);
            roles.push(SourceChildRole::Operand(SourceOperandRole::MatchScrutinee));
            roles.extend(
                cases
                    .iter()
                    .map(|_| SourceChildRole::Operand(SourceOperandRole::MatchArm)),
            );
            roles
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            let mut roles = Vec::with_capacity(cases.len() + 1);
            roles.push(SourceChildRole::Operand(SourceOperandRole::MatchScrutinee));
            roles.extend(
                cases
                    .iter()
                    .map(|_| SourceChildRole::Operand(SourceOperandRole::MatchArm)),
            );
            roles
        }
        RuntimeExpr::Record { fields } => fields
            .iter()
            .map(|_| SourceChildRole::Operand(SourceOperandRole::RecordField))
            .collect(),
        RuntimeExpr::Project { .. } => {
            vec![SourceChildRole::Operand(SourceOperandRole::ProjectRecord)]
        }
        RuntimeExpr::Closure { .. } => vec![SourceChildRole::StaticBody],
        RuntimeExpr::LexicalClosure { captures, .. } => {
            let mut roles = Vec::with_capacity(captures.len() + 1);
            roles.push(SourceChildRole::StaticBody);
            roles.extend(
                captures
                    .iter()
                    .map(|_| SourceChildRole::Operand(SourceOperandRole::LexicalCapture)),
            );
            roles
        }
        RuntimeExpr::Call { args, .. } => {
            let mut roles = Vec::with_capacity(args.len() + 1);
            roles.push(SourceChildRole::Operand(SourceOperandRole::CallCallee));
            roles.extend(
                args.iter()
                    .map(|_| SourceChildRole::Operand(SourceOperandRole::CallArgument)),
            );
            roles
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            let mut roles = Vec::with_capacity(args.len() + usize::from(capability.is_some()));
            if capability.is_some() {
                roles.push(SourceChildRole::Operand(
                    SourceOperandRole::EffectCapability,
                ));
            }
            roles.extend(
                args.iter()
                    .map(|_| SourceChildRole::Operand(SourceOperandRole::EffectArgument)),
            );
            roles
        }
    }
}

fn source_operand_role_label(role: SourceOperandRole) -> &'static str {
    match role {
        SourceOperandRole::WrapperBody => "a checked wrapper body",
        SourceOperandRole::LetValue => "a let value",
        SourceOperandRole::LetBody => "a let body",
        SourceOperandRole::IfScrutinee => "an if scrutinee",
        SourceOperandRole::IfArm => "an if arm",
        SourceOperandRole::PrimitiveArgument => "a primitive-call argument",
        SourceOperandRole::ConstructArgument => "a constructor argument",
        SourceOperandRole::MatchScrutinee => "a match scrutinee",
        SourceOperandRole::MatchArm => "a match arm",
        SourceOperandRole::RecordField => "a record field",
        SourceOperandRole::ProjectRecord => "a projected record",
        SourceOperandRole::LexicalCapture => "a lexical closure capture",
        SourceOperandRole::CallCallee => "a call callee",
        SourceOperandRole::CallArgument => "a call argument",
        SourceOperandRole::EffectCapability => "an effect capability",
        SourceOperandRole::EffectArgument => "an effect argument",
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CallableParameterUse {
    invoked: bool,
    forwarded: bool,
    observed_as_value: bool,
}

impl CallableParameterUse {
    const UNUSED: Self = Self {
        invoked: false,
        forwarded: false,
        observed_as_value: false,
    };

    fn combine(&mut self, other: Self) {
        self.invoked |= other.invoked;
        self.forwarded |= other.forwarded;
        self.observed_as_value |= other.observed_as_value;
    }

    fn is_callable(self) -> bool {
        self.invoked || self.forwarded
    }
}

/// Classify every occurrence of one declaration parameter.
///
/// The target is a de Bruijn index in the current environment. Binder-producing
/// forms shift it explicitly. A parameter is callable only when every use is a
/// direct invocation or exact forwarding argument to another transparent
/// declaration. Any other occurrence is a value observation and makes static
/// elimination unlawful before an ABI descriptor can be added.
fn classify_callable_parameter_use(
    expr: &RuntimeExpr,
    target: u32,
) -> Result<CallableParameterUse, CraneliftBackendError> {
    fn observed(
        expr: &RuntimeExpr,
        target: u32,
    ) -> Result<CallableParameterUse, CraneliftBackendError> {
        let mut use_ = CallableParameterUse::UNUSED;
        collect(expr, target, &mut use_)?;
        if use_.invoked || use_.forwarded {
            use_.observed_as_value = true;
        }
        Ok(use_)
    }

    fn shifted(target: u32, binders: usize) -> Result<u32, CraneliftBackendError> {
        target
            .checked_add(
                u32::try_from(binders)
                    .map_err(|_| planner_capacity_error("callable binder depth exhausted"))?,
            )
            .ok_or_else(|| planner_capacity_error("callable binder depth exhausted"))
    }

    fn collect(
        expr: &RuntimeExpr,
        target: u32,
        use_: &mut CallableParameterUse,
    ) -> Result<(), CraneliftBackendError> {
        match expr {
            RuntimeExpr::Var(index) => {
                if *index == target {
                    use_.observed_as_value = true;
                }
            }
            RuntimeExpr::CheckedJoinSite { body, .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
            | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
            | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { body, .. } => {
                collect(body, target, use_)?;
            }
            RuntimeExpr::Let { value, body } => {
                collect(value, target, use_)?;
                collect(body, shifted(target, 1)?, use_)?;
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                use_.combine(observed(scrutinee, target)?);
                collect(then_expr, target, use_)?;
                collect(else_expr, target, use_)?;
            }
            RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
                for arg in args {
                    use_.combine(observed(arg, target)?);
                }
            }
            RuntimeExpr::Match {
                scrutinee, cases, ..
            } => {
                use_.combine(observed(scrutinee, target)?);
                for case in cases {
                    collect(&case.body, shifted(target, case.binders)?, use_)?;
                }
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee, cases, ..
            } => {
                use_.combine(observed(scrutinee, target)?);
                for case in cases {
                    let binders = case
                        .argument_binders
                        .saturating_add(case.recursive_positions.len());
                    collect(&case.body, shifted(target, binders)?, use_)?;
                }
            }
            RuntimeExpr::Record { fields } => {
                for (_, field) in fields {
                    use_.combine(observed(field, target)?);
                }
            }
            RuntimeExpr::Project { record, .. } => {
                use_.combine(observed(record, target)?);
            }
            RuntimeExpr::Closure { .. } => {}
            RuntimeExpr::LexicalClosure { captures, .. } => {
                for capture in captures {
                    use_.combine(observed(capture, target)?);
                }
            }
            RuntimeExpr::Call { callee, args } => {
                match callee.as_ref() {
                    RuntimeExpr::Var(index) if *index == target => {
                        use_.invoked = true;
                    }
                    other => collect(other, target, use_)?,
                }
                let transparent = matches!(callee.as_ref(), RuntimeExpr::DeclarationRef { .. });
                for arg in args {
                    match arg {
                        RuntimeExpr::Var(index) if *index == target && transparent => {
                            use_.forwarded = true;
                        }
                        other => collect(other, target, use_)?,
                    }
                }
            }
            RuntimeExpr::Effect {
                capability, args, ..
            } => {
                if let Some(capability) = capability {
                    use_.combine(observed(&capability.value, target)?);
                }
                for arg in args {
                    use_.combine(observed(arg, target)?);
                }
            }
            RuntimeExpr::Value(_)
            | RuntimeExpr::DeclarationRef { .. }
            | RuntimeExpr::ImportedDeclarationRef { .. }
            | RuntimeExpr::Trap(_) => {}
        }
        Ok(())
    }

    let mut use_ = CallableParameterUse::UNUSED;
    collect(expr, target, &mut use_)?;
    Ok(use_)
}

#[derive(Clone)]
struct CallableDeclarationPlan {
    origin: StaticOriginId,
    function: PredeclaredFunctionId,
    body_function: PredeclaredFunctionId,
    body_origin: StaticOriginId,
    parameter_uses: Vec<CallableParameterUse>,
    declaration_captures: u32,
}

fn callable_declaration_plan(
    plan: &StaticTransitionPlan<'_>,
    symbol: &str,
) -> Result<Option<CallableDeclarationPlan>, CraneliftBackendError> {
    let Some(origin) = plan.declaration_occurrences.get(symbol).copied() else {
        return Ok(None);
    };
    let occurrence = plan
        .source_occurrences
        .get(origin.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| planner_error("transparent declaration has no source occurrence"))?;
    let (params, body, declaration_captures) = match occurrence.expr {
        RuntimeExpr::Closure {
            captures,
            params,
            body,
        } => (
            params,
            body.as_ref(),
            u32::try_from(captures.len())
                .map_err(|_| planner_capacity_error("declaration capture count exhausted"))?,
        ),
        RuntimeExpr::LexicalClosure {
            captures,
            params,
            body,
        } => (
            params,
            body.as_ref(),
            u32::try_from(captures.len())
                .map_err(|_| planner_capacity_error("declaration capture count exhausted"))?,
        ),
        _ => return Ok(None),
    };
    let body_origin = plan.semantic.child_origin(origin, 0)?;
    let function = plan
        .semantic
        .function_owner(origin)?
        .ok_or_else(|| planner_error("transparent declaration has no function owner"))?;
    let body_function = plan
        .semantic
        .function_owner(body_origin)?
        .ok_or_else(|| planner_error("transparent declaration body has no function owner"))?;
    let parameter_uses = (0..params.len())
        .map(|ordinal| {
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| planner_capacity_error("callable parameter ordinal exhausted"))?;
            classify_callable_parameter_use(body, ordinal)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(CallableDeclarationPlan {
        origin,
        function,
        body_function,
        body_origin,
        parameter_uses,
        declaration_captures,
    }))
}

fn static_binding_from_closure(
    plan: &StaticTransitionPlan<'_>,
    parameter_ordinal: u32,
    closure_origin: StaticOriginId,
    environment: &[Option<StaticCallableBindingKey>],
) -> Result<Option<StaticCallableBindingKey>, CraneliftBackendError> {
    let occurrence = plan
        .source_occurrences
        .get(closure_origin.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| planner_error("static callable argument has no source occurrence"))?;
    let (declared_arity, mut capture_origins) = match occurrence.expr {
        RuntimeExpr::Closure {
            captures, params, ..
        } => {
            let captures = (0..captures.len())
                .map(|ordinal| {
                    let ordinal = u32::try_from(ordinal).map_err(|_| {
                        planner_capacity_error("static callable capture ordinal exhausted")
                    })?;
                    Ok(StaticCallableCaptureBinding::Value(
                        StaticCallableCaptureProvenance {
                            owner: plan.semantic.function_owner(closure_origin)?.ok_or_else(
                                || planner_error("static callable closure has no owner"),
                            )?,
                            closure_origin,
                            capture_origin: closure_origin,
                            ordinal,
                            phase: StaticCallableCapturePhase::SpecializedOnly,
                        },
                    ))
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
            (
                u32::try_from(params.len())
                    .map_err(|_| planner_capacity_error("callable arity exhausted"))?,
                captures,
            )
        }
        RuntimeExpr::LexicalClosure {
            captures, params, ..
        } => {
            let phase_environment = result_phase_environment_for_owner(plan, closure_origin, true)?;
            let mut joins = plan.join_results.clone();
            let mut phases = vec![None; plan.source_occurrences.len()];
            let mut provenance = Vec::with_capacity(captures.len());
            for (ordinal, capture) in captures.iter().enumerate() {
                let capture_origin = plan.semantic.child_origin(closure_origin, 1 + ordinal)?;
                let capture_ordinal = u32::try_from(ordinal).map_err(|_| {
                    planner_capacity_error("static callable capture ordinal exhausted")
                })?;
                let callable = match capture {
                    RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. } => {
                        static_binding_from_closure(
                            plan,
                            capture_ordinal,
                            capture_origin,
                            environment,
                        )?
                    }
                    RuntimeExpr::Var(index) => environment
                        .get(*index as usize)
                        .and_then(Clone::clone)
                        .map(|mut binding| {
                            binding.parameter_ordinal = capture_ordinal;
                            binding
                        }),
                    _ => None,
                };
                if let Some(binding) = callable {
                    provenance.push(StaticCallableCaptureBinding::Callable(Box::new(binding)));
                    continue;
                }
                let summary = summarize_result_phase(
                    plan,
                    capture_origin,
                    true,
                    &phase_environment,
                    &mut joins,
                    &mut phases,
                )?;
                if summary.callable_result.is_some() {
                    return Err(planner_error(
                        "dynamically selected callable capture cannot enter static callable \
                         elimination",
                    ));
                }
                provenance.push(StaticCallableCaptureBinding::Value(
                    StaticCallableCaptureProvenance {
                        owner: plan
                            .semantic
                            .function_owner(capture_origin)?
                            .ok_or_else(|| planner_error("static callable capture has no owner"))?,
                        closure_origin,
                        capture_origin,
                        ordinal: capture_ordinal,
                        phase: match summary.phase {
                            ResultPhase::SpecializedOnly => {
                                StaticCallableCapturePhase::SpecializedOnly
                            }
                            ResultPhase::CarrierRequired => {
                                StaticCallableCapturePhase::CarrierRequired
                            }
                        },
                    },
                ));
            }
            (
                u32::try_from(params.len())
                    .map_err(|_| planner_capacity_error("callable arity exhausted"))?,
                provenance,
            )
        }
        _ => return Ok(None),
    };
    let body_origin = plan.semantic.child_origin(closure_origin, 0)?;
    let body = match occurrence.expr {
        RuntimeExpr::Closure { body, .. } | RuntimeExpr::LexicalClosure { body, .. } => {
            body.as_ref()
        }
        _ => unreachable!("the closure shape was matched above"),
    };
    for (ordinal, capture) in capture_origins.iter_mut().enumerate() {
        let StaticCallableCaptureBinding::Callable(binding) = capture else {
            continue;
        };
        let capture_index =
            declared_arity
                .checked_add(u32::try_from(ordinal).map_err(|_| {
                    planner_capacity_error("static callable capture ordinal exhausted")
                })?)
                .ok_or_else(|| planner_capacity_error("static callable capture index exhausted"))?;
        let use_ = classify_callable_parameter_use(body, capture_index)?;
        if use_.observed_as_value {
            return Err(planner_error(
                "captured callable is returned, stored, constructed, effect-passed, compared, \
                 or otherwise observed as a runtime value",
            ));
        }
        if !use_.is_callable() {
            binding.captures.clear();
        }
    }
    Ok(Some(StaticCallableBindingKey {
        parameter_ordinal,
        closure_origin,
        body_origin,
        declared_arity,
        captures: capture_origins,
    }))
}

fn binding_has_callable_capture(binding: &StaticCallableBindingKey) -> bool {
    binding
        .captures
        .iter()
        .any(|capture| matches!(capture, StaticCallableCaptureBinding::Callable(_)))
}

fn intern_static_callable_body(
    plan: &StaticTransitionPlan<'_>,
    binding: &StaticCallableBindingKey,
    interned: &mut BTreeMap<StaticCallableSpecializationKey, StaticCallableSpecializationId>,
    specializations: &mut Vec<PlannedStaticCallableSpecialization>,
) -> Result<StaticCallableSpecializationId, CraneliftBackendError> {
    let body_function = plan
        .semantic
        .function_owner(binding.body_origin)?
        .ok_or_else(|| planner_error("static callable body has no function owner"))?;
    let mut normalized = binding.clone();
    normalized.parameter_ordinal = 0;
    let key = StaticCallableSpecializationKey {
        base_owner: body_function,
        base_origin: binding.body_origin,
        bindings: vec![normalized.clone()],
    };
    if let Some(id) = interned.get(&key).copied() {
        return Ok(id);
    }
    let id = StaticCallableSpecializationId(
        u32::try_from(specializations.len())
            .map_err(|_| planner_capacity_error("static callable body units exhausted"))?,
    );
    interned.insert(key.clone(), id);
    let function_ordinal = plan
        .semantic
        .functions
        .len()
        .checked_add(specializations.len())
        .ok_or_else(|| planner_capacity_error("callable body function identity exhausted"))?;
    let function = PredeclaredFunctionId(
        u32::try_from(function_ordinal)
            .map_err(|_| planner_capacity_error("callable body function identity exhausted"))?,
    );
    specializations.push(PlannedStaticCallableSpecialization {
        id,
        function,
        base_function: body_function,
        body_function,
        base_origin: binding.body_origin,
        base_body_origin: binding.body_origin,
        key,
        ordinary_parameters: binding.declared_arity,
        lifted_captures: binding.lifted_capture_count()?,
        kind: PlannedStaticCallableSpecializationKind::CallableBody {
            binding: normalized.clone(),
        },
    });
    for capture in &normalized.captures {
        if let StaticCallableCaptureBinding::Callable(nested) = capture {
            if binding_has_callable_capture(nested) {
                intern_static_callable_body(plan, nested, interned, specializations)?;
            }
        }
    }
    Ok(id)
}

fn intern_static_callable_specialization(
    plan: &StaticTransitionPlan<'_>,
    key: StaticCallableSpecializationKey,
    declaration: &CallableDeclarationPlan,
    interned: &mut BTreeMap<StaticCallableSpecializationKey, StaticCallableSpecializationId>,
    specializations: &mut Vec<PlannedStaticCallableSpecialization>,
) -> Result<StaticCallableSpecializationId, CraneliftBackendError> {
    if let Some(id) = interned.get(&key).copied() {
        return Ok(id);
    }
    let id = StaticCallableSpecializationId(
        u32::try_from(specializations.len())
            .map_err(|_| planner_capacity_error("static callable specialization exhausted"))?,
    );
    // Intern before enqueue. Recursive discovery of this same key observes the
    // entry and cannot clone the state.
    interned.insert(key.clone(), id);
    let function_ordinal = plan
        .semantic
        .functions
        .len()
        .checked_add(specializations.len())
        .ok_or_else(|| planner_capacity_error("specialization function identity exhausted"))?;
    let function = PredeclaredFunctionId(
        u32::try_from(function_ordinal)
            .map_err(|_| planner_capacity_error("specialization function identity exhausted"))?,
    );
    let eliminated = u32::try_from(key.bindings.len())
        .map_err(|_| planner_capacity_error("eliminated callable count exhausted"))?;
    let total_parameters = u32::try_from(declaration.parameter_uses.len())
        .map_err(|_| planner_capacity_error("declaration parameter count exhausted"))?;
    let ordinary_parameters = total_parameters
        .checked_sub(eliminated)
        .ok_or_else(|| planner_error("callable binding population exceeds declaration arity"))?;
    let lifted_callable_captures = key.bindings.iter().try_fold(0u32, |total, binding| {
        total
            .checked_add(binding.lifted_capture_count()?)
            .ok_or_else(|| planner_capacity_error("lifted callable capture count exhausted"))
    })?;
    let lifted_captures = lifted_callable_captures
        .checked_add(declaration.declaration_captures)
        .ok_or_else(|| planner_capacity_error("specialization capture count exhausted"))?;
    specializations.push(PlannedStaticCallableSpecialization {
        id,
        function,
        base_function: declaration.function,
        body_function: declaration.body_function,
        base_origin: declaration.origin,
        base_body_origin: declaration.body_origin,
        key,
        ordinary_parameters,
        lifted_captures,
        kind: PlannedStaticCallableSpecializationKind::Declaration,
    });
    let bindings = specializations[id.0 as usize].key.bindings.clone();
    for binding in &bindings {
        if binding_has_callable_capture(binding) {
            intern_static_callable_body(plan, binding, interned, specializations)?;
        }
    }
    Ok(id)
}

fn plan_static_callable_call(
    plan: &StaticTransitionPlan<'_>,
    call_origin: StaticOriginId,
    caller: PredeclaredFunctionId,
    environment: &[Option<StaticCallableBindingKey>],
    declarations: &BTreeMap<String, CallableDeclarationPlan>,
    interned: &mut BTreeMap<StaticCallableSpecializationKey, StaticCallableSpecializationId>,
    specializations: &mut Vec<PlannedStaticCallableSpecialization>,
) -> Result<Option<PlannedStaticCallableCall>, CraneliftBackendError> {
    let occurrence = plan
        .source_occurrences
        .get(call_origin.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| planner_error("static callable call has no source occurrence"))?;
    let RuntimeExpr::Call { args, .. } = occurrence.expr else {
        return Err(planner_error(
            "static callable call planner received a non-call occurrence",
        ));
    };
    let callee_reference_origin = plan.semantic.child_origin(call_origin, 0)?;
    let callee = plan
        .source_occurrences
        .get(callee_reference_origin.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| planner_error("call callee has no source occurrence"))?;
    let RuntimeExpr::DeclarationRef { symbol } = callee.expr else {
        return Ok(None);
    };
    let Some(declaration) = declarations.get(symbol.as_str()) else {
        return Ok(None);
    };
    if args.len() != declaration.parameter_uses.len() {
        return Err(planner_error(
            "transparent declaration call arity disagrees with its declared closure",
        ));
    }

    let mut bindings = Vec::new();
    let mut arguments = Vec::with_capacity(args.len());
    for (ordinal, (argument, use_)) in args.iter().zip(&declaration.parameter_uses).enumerate() {
        let parameter_ordinal = u32::try_from(ordinal)
            .map_err(|_| planner_capacity_error("callable parameter ordinal exhausted"))?;
        let argument_origin = plan.semantic.child_origin(call_origin, 1 + ordinal)?;
        let mut binding = match argument {
            RuntimeExpr::Var(index) => {
                environment
                    .get(*index as usize)
                    .and_then(Clone::clone)
                    .map(|mut binding| {
                        binding.parameter_ordinal = parameter_ordinal;
                        binding
                    })
            }
            RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. } => {
                static_binding_from_closure(plan, parameter_ordinal, argument_origin, environment)?
            }
            _ => None,
        };
        if let Some(mut binding) = binding.take() {
            if use_.observed_as_value {
                return Err(planner_error(
                    "callable parameter is returned, stored, constructed, effect-passed, compared, \
                     or otherwise observed as a runtime value",
                ));
            }
            let kind = if !use_.is_callable() {
                binding.captures.clear();
                StaticCallableArgumentKind::Erased
            } else {
                match argument {
                    RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. } => {
                        StaticCallableArgumentKind::Direct {
                            closure_origin: argument_origin,
                        }
                    }
                    RuntimeExpr::Var(_) => StaticCallableArgumentKind::Forwarded {
                        body_origin: binding.body_origin,
                        declared_arity: binding.declared_arity,
                    },
                    _ => {
                        return Err(planner_error(
                            "static callable binding source is not closed",
                        ));
                    }
                }
            };
            arguments.push(StaticCallableArgument {
                parameter_ordinal,
                argument_origin,
                kind,
            });
            bindings.push(binding);
        } else {
            if use_.is_callable() {
                return Err(planner_error(
                    "callable parameter does not resolve to one static closure body",
                ));
            }
            arguments.push(StaticCallableArgument {
                parameter_ordinal,
                argument_origin,
                kind: StaticCallableArgumentKind::Ordinary,
            });
        }
    }
    if bindings.is_empty() {
        return Ok(None);
    }
    bindings.sort_by_key(|binding| binding.parameter_ordinal);
    let key = StaticCallableSpecializationKey {
        base_owner: declaration.function,
        base_origin: declaration.origin,
        bindings,
    };
    let specialization =
        intern_static_callable_specialization(plan, key, declaration, interned, specializations)?;
    Ok(Some(PlannedStaticCallableCall {
        caller,
        call_origin,
        callee_reference_origin,
        specialization,
        arguments,
    }))
}

#[allow(clippy::too_many_arguments)]
fn discover_specialized_body_calls(
    plan: &StaticTransitionPlan<'_>,
    origin: StaticOriginId,
    caller: PredeclaredFunctionId,
    environment: &[Option<StaticCallableBindingKey>],
    declarations: &BTreeMap<String, CallableDeclarationPlan>,
    interned: &mut BTreeMap<StaticCallableSpecializationKey, StaticCallableSpecializationId>,
    specializations: &mut Vec<PlannedStaticCallableSpecialization>,
    calls: &mut Vec<PlannedStaticCallableCall>,
) -> Result<(), CraneliftBackendError> {
    let occurrence = plan
        .source_occurrences
        .get(origin.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| planner_error("specialized body walk has no source occurrence"))?;
    if matches!(occurrence.expr, RuntimeExpr::Call { .. }) {
        if let Some(call) = plan_static_callable_call(
            plan,
            origin,
            caller,
            environment,
            declarations,
            interned,
            specializations,
        )? {
            calls.push(call);
        }
    }
    let child = |position| plan.semantic.child_origin(origin, position);
    match occurrence.expr {
        RuntimeExpr::CheckedJoinSite { .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { .. }
        | RuntimeExpr::CheckedRecursiveInvocation { .. }
        | RuntimeExpr::CheckedComputationalIHSlots { .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { .. } => {
            discover_specialized_body_calls(
                plan,
                child(0)?,
                caller,
                environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
        }
        RuntimeExpr::Let { .. } => {
            discover_specialized_body_calls(
                plan,
                child(0)?,
                caller,
                environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
            let mut body_environment = Vec::with_capacity(environment.len() + 1);
            body_environment.push(None);
            body_environment.extend_from_slice(environment);
            discover_specialized_body_calls(
                plan,
                child(1)?,
                caller,
                &body_environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
        }
        RuntimeExpr::If { .. } => {
            for position in 0..3 {
                discover_specialized_body_calls(
                    plan,
                    child(position)?,
                    caller,
                    environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::Match { cases, .. } => {
            discover_specialized_body_calls(
                plan,
                child(0)?,
                caller,
                environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
            for (index, case) in cases.iter().enumerate() {
                let mut case_environment = Vec::with_capacity(case.binders + environment.len());
                case_environment.extend((0..case.binders).map(|_| None));
                case_environment.extend_from_slice(environment);
                discover_specialized_body_calls(
                    plan,
                    child(1 + index)?,
                    caller,
                    &case_environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            discover_specialized_body_calls(
                plan,
                child(0)?,
                caller,
                environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
            for (index, case) in cases.iter().enumerate() {
                let binders = case
                    .argument_binders
                    .checked_add(case.recursive_positions.len())
                    .ok_or_else(|| planner_capacity_error("callable case arity exhausted"))?;
                let mut case_environment = Vec::with_capacity(binders + environment.len());
                case_environment.extend((0..binders).map(|_| None));
                case_environment.extend_from_slice(environment);
                discover_specialized_body_calls(
                    plan,
                    child(1 + index)?,
                    caller,
                    &case_environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::Closure { .. } => {}
        RuntimeExpr::LexicalClosure { captures, .. } => {
            for position in 0..captures.len() {
                discover_specialized_body_calls(
                    plan,
                    child(1 + position)?,
                    caller,
                    environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            for position in 0..args.len() {
                discover_specialized_body_calls(
                    plan,
                    child(position)?,
                    caller,
                    environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::Record { fields } => {
            for position in 0..fields.len() {
                discover_specialized_body_calls(
                    plan,
                    child(position)?,
                    caller,
                    environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::Project { .. } => {
            discover_specialized_body_calls(
                plan,
                child(0)?,
                caller,
                environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
        }
        RuntimeExpr::Call { args, .. } => {
            discover_specialized_body_calls(
                plan,
                child(0)?,
                caller,
                environment,
                declarations,
                interned,
                specializations,
                calls,
            )?;
            for position in 0..args.len() {
                let argument_origin = child(1 + position)?;
                let argument = plan
                    .source_occurrences
                    .get(argument_origin.0 as usize)
                    .and_then(Option::as_ref)
                    .ok_or_else(|| planner_error("call argument has no source occurrence"))?;
                match argument.expr {
                    RuntimeExpr::Closure { .. } => {}
                    RuntimeExpr::LexicalClosure { captures, .. } => {
                        for capture in 0..captures.len() {
                            discover_specialized_body_calls(
                                plan,
                                plan.semantic.child_origin(argument_origin, 1 + capture)?,
                                caller,
                                environment,
                                declarations,
                                interned,
                                specializations,
                                calls,
                            )?;
                        }
                    }
                    _ => discover_specialized_body_calls(
                        plan,
                        argument_origin,
                        caller,
                        environment,
                        declarations,
                        interned,
                        specializations,
                        calls,
                    )?,
                }
            }
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => {
            let child_count = args.len() + usize::from(capability.is_some());
            for position in 0..child_count {
                discover_specialized_body_calls(
                    plan,
                    child(position)?,
                    caller,
                    environment,
                    declarations,
                    interned,
                    specializations,
                    calls,
                )?;
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::Var(_)
        | RuntimeExpr::DeclarationRef { .. }
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Trap(_) => {}
    }
    Ok(())
}

fn build_static_callable_specializations(
    plan: &StaticTransitionPlan<'_>,
) -> Result<
    (
        Vec<PlannedStaticCallableSpecialization>,
        Vec<PlannedStaticCallableCall>,
    ),
    CraneliftBackendError,
> {
    let mut declarations = BTreeMap::new();
    for symbol in plan.declaration_occurrences.keys() {
        if let Some(declaration) = callable_declaration_plan(plan, symbol)? {
            declarations.insert(symbol.clone(), declaration);
        }
    }
    let callable_base_functions = declarations
        .values()
        .filter(|declaration| {
            declaration
                .parameter_uses
                .iter()
                .any(|use_| use_.is_callable())
        })
        .flat_map(|declaration| [declaration.function, declaration.body_function])
        .collect::<BTreeSet<_>>();

    let mut interned = BTreeMap::new();
    let mut specializations = Vec::new();
    let mut calls = Vec::new();
    for occurrence in plan.source_occurrences.iter().flatten() {
        let RuntimeExpr::Call { args, .. } = occurrence.expr else {
            continue;
        };
        let owner = plan
            .semantic
            .function_owner(occurrence.static_origin)?
            .ok_or_else(|| planner_error("call occurrence has no function owner"))?;
        let has_direct_callable = args.iter().any(|argument| {
            matches!(
                argument,
                RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. }
            )
        });
        if !has_direct_callable && callable_base_functions.contains(&owner) {
            continue;
        }
        if let Some(call) = plan_static_callable_call(
            plan,
            occurrence.static_origin,
            owner,
            &[],
            &declarations,
            &mut interned,
            &mut specializations,
        )? {
            calls.push(call);
        }
    }

    let mut cursor = 0usize;
    while cursor < specializations.len() {
        let specialization = specializations[cursor].clone();
        let environment = match &specialization.kind {
            PlannedStaticCallableSpecializationKind::Declaration => {
                let declaration = declarations
                    .values()
                    .find(|declaration| declaration.origin == specialization.base_origin)
                    .ok_or_else(|| planner_error("specialization base declaration is absent"))?;
                let mut environment = vec![None; declaration.parameter_uses.len()];
                for binding in &specialization.key.bindings {
                    let slot = environment
                        .get_mut(binding.parameter_ordinal as usize)
                        .ok_or_else(|| {
                            planner_error("specialization binding exceeds base arity")
                        })?;
                    *slot = Some(binding.clone());
                }
                environment.extend((0..declaration.declaration_captures).map(|_| None));
                environment
            }
            PlannedStaticCallableSpecializationKind::CallableBody { binding } => {
                let mut environment = vec![None; binding.declared_arity as usize];
                environment.extend(binding.captures.iter().map(|capture| match capture {
                    StaticCallableCaptureBinding::Value(_) => None,
                    StaticCallableCaptureBinding::Callable(binding) => Some((**binding).clone()),
                }));
                environment
            }
        };
        discover_specialized_body_calls(
            plan,
            specialization.base_body_origin,
            specialization.function,
            &environment,
            &declarations,
            &mut interned,
            &mut specializations,
            &mut calls,
        )?;
        cursor += 1;
    }

    calls.sort_by_key(|call| (call.caller, call.call_origin));
    if calls
        .windows(2)
        .any(|pair| pair[0].caller == pair[1].caller && pair[0].call_origin == pair[1].call_origin)
    {
        return Err(planner_error(
            "one caller has two static callable targets for one call occurrence",
        ));
    }
    Ok((specializations, calls))
}

fn derive_operand_edge_disposition(
    plan: &StaticTransitionPlan<'_>,
    parent: StaticOriginId,
    child: StaticOriginId,
    position: u32,
    role: SourceOperandRole,
) -> Result<OperandEdgeDisposition, CraneliftBackendError> {
    if role == SourceOperandRole::ConstructArgument
        && is_static_recursor_construct_argument(plan, parent, child, position)?
    {
        return Ok(OperandEdgeDisposition::CallableCapture);
    }
    if role == SourceOperandRole::ConstructArgument
        && plan
            .source_occurrences
            .get(child.0 as usize)
            .and_then(Option::as_ref)
            .is_some_and(|occurrence| {
                matches!(
                    occurrence.expr,
                    RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. }
                )
            })
        && construct_crosses_carrier_edge(plan, parent)?
    {
        return Ok(OperandEdgeDisposition::EscapeForbidden);
    }
    if role != SourceOperandRole::CallArgument {
        return role_only_disposition(role);
    }
    let parameter_ordinal = position
        .checked_sub(1)
        .ok_or_else(|| planner_error("call argument occupies the callee position"))?;
    let eliminated = plan.static_callable_calls.iter().any(|call| {
        call.call_origin == parent
            && call.arguments.iter().any(|argument| {
                argument.parameter_ordinal == parameter_ordinal
                    && argument.kind != StaticCallableArgumentKind::Ordinary
            })
    });
    Ok(if eliminated {
        OperandEdgeDisposition::StaticCallableElimination
    } else {
        OperandEdgeDisposition::Forwarding
    })
}

fn construct_crosses_carrier_edge(
    plan: &StaticTransitionPlan<'_>,
    parent: StaticOriginId,
) -> Result<bool, CraneliftBackendError> {
    for child in plan.semantic.child_origins(parent)? {
        let Some(phase) = plan
            .result_phases
            .get(child.0 as usize)
            .and_then(Option::as_ref)
        else {
            // A child outside the functionized result-flow walk has no
            // generated carrier transition. Its source edge remains a
            // specialized semantic read; absence here is not permission to
            // invent a boundary.
            continue;
        };
        if phase.phase == ResultPhase::CarrierRequired {
            return Ok(true);
        }
    }
    Ok(false)
}

fn is_static_recursor_construct_argument(
    plan: &StaticTransitionPlan<'_>,
    parent: StaticOriginId,
    child: StaticOriginId,
    position: u32,
) -> Result<bool, CraneliftBackendError> {
    for occurrence in plan.source_occurrences.iter().flatten() {
        let RuntimeExpr::ComputationalMatch { cases, .. } = occurrence.expr else {
            continue;
        };
        if !cases
            .iter()
            .any(|case| case.recursive_positions.contains(&(position as usize)))
        {
            continue;
        }
        let scrutinee_origin = plan.semantic.child_origin(occurrence.static_origin, 0)?;
        if !plan
            .source_result_origins_in_owner_subtree(scrutinee_origin)?
            .contains(&parent)
        {
            continue;
        }
        if plan.semantic.child_origin(parent, position as usize)? == child {
            return Ok(true);
        }
    }
    Ok(false)
}

fn build_operand_edge_matrix(
    plan: &StaticTransitionPlan<'_>,
) -> Result<Vec<PlannedOperandEdge>, CraneliftBackendError> {
    let mut edges = Vec::new();
    for occurrence in plan.source_occurrences.iter().flatten() {
        let parent = occurrence.static_origin;
        let children = plan.semantic.child_origins(parent)?;
        let roles = source_child_roles(occurrence.expr);
        if roles.len() != children.len() {
            return Err(planner_error(
                "operand-edge role inventory is not exact for positional source children",
            ));
        }
        let owner = plan
            .semantic
            .function_owner(parent)?
            .ok_or_else(|| planner_error("source operand edge has no function owner"))?;
        for (position, (child, role)) in children.iter().copied().zip(roles).enumerate() {
            let SourceChildRole::Operand(role) = role else {
                continue;
            };
            let (effect_operation, effect_seat) =
                effect_edge_contract(occurrence.expr, position, role)?;
            let position = u32::try_from(position)
                .map_err(|_| planner_capacity_error("operand-edge position exhausted"))?;
            let producer_owner = plan
                .semantic
                .function_owner(child)?
                .ok_or_else(|| planner_error("source operand producer has no function owner"))?;
            let disposition = derive_operand_edge_disposition(plan, parent, child, position, role)?;
            let (consumer_phase, operation, need, avail) =
                operand_edge_contract(disposition, effect_seat);
            edges.push(PlannedOperandEdge {
                owner,
                producer_owner,
                parent,
                child,
                position,
                role,
                effect_operation,
                effect_seat,
                disposition,
                producer_phase: BoundaryUsePhase::SpecializedValue,
                consumer_phase,
                operation,
                need,
                avail,
            });
        }
    }
    #[cfg(test)]
    if D7_OMIT_LEXICAL_CAPTURE_EDGE.with(Cell::get) {
        if let Some(position) = edges
            .iter()
            .position(|edge| edge.role == SourceOperandRole::LexicalCapture)
        {
            edges.remove(position);
        }
    }
    Ok(edges)
}

fn build_static_recursor_worker_residuals(
    plan: &StaticTransitionPlan<'_>,
) -> Result<Vec<PlannedStaticRecursorWorkerResidual>, CraneliftBackendError> {
    let mut residuals = Vec::new();
    for parent in plan.source_occurrences.iter().flatten() {
        let RuntimeExpr::ComputationalMatch { cases, .. } = parent.expr else {
            continue;
        };
        let scrutinee_origin = plan.semantic.child_origin(parent.static_origin, 0)?;
        let scrutinee_results = plan.source_result_origins_in_owner_subtree(scrutinee_origin)?;
        for position in cases
            .iter()
            .flat_map(|case| case.recursive_positions.iter().copied())
        {
            for constructor_origin in &scrutinee_results {
                let Some(constructor) = plan
                    .source_occurrences
                    .get(constructor_origin.0 as usize)
                    .and_then(Option::as_ref)
                else {
                    return Err(planner_error(
                        "static recursor scrutinee descendant has no source occurrence",
                    ));
                };
                let RuntimeExpr::Construct { args, .. } = constructor.expr else {
                    continue;
                };
                let Some(candidate) = args.get(position) else {
                    continue;
                };
                if !matches!(
                    candidate,
                    RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. }
                ) {
                    continue;
                }
                let closure_origin = plan.semantic.child_origin(*constructor_origin, position)?;
                let closure = plan
                    .source_occurrences
                    .get(closure_origin.0 as usize)
                    .and_then(Option::as_ref)
                    .ok_or_else(|| {
                        planner_error("static recursor child has no source occurrence")
                    })?;
                residuals.push(build_static_recursor_worker_residual(
                    plan,
                    parent.static_origin,
                    *constructor_origin,
                    position,
                    closure,
                )?);
            }
        }
    }
    residuals.sort();
    if residuals.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(planner_error(
            "static recursor worker residual population contains a duplicate",
        ));
    }
    Ok(residuals)
}

fn build_recursor_boundary_uses(
    plan: &StaticTransitionPlan<'_>,
) -> Result<Vec<PlannedRecursorBoundaryUse>, CraneliftBackendError> {
    let mut uses = BTreeSet::new();
    for occurrence in plan.source_occurrences.iter().flatten() {
        let RuntimeExpr::ComputationalMatch { cases, .. } = occurrence.expr else {
            continue;
        };
        let owner = plan
            .semantic
            .function_owner(occurrence.static_origin)?
            .ok_or_else(|| planner_error("computational recursor boundary has no owner"))?;
        for position in cases
            .iter()
            .flat_map(|case| case.recursive_positions.iter().copied())
        {
            let sibling_position = u32::try_from(position)
                .map_err(|_| planner_capacity_error("recursor boundary position exhausted"))?;
            if plan
                .static_recursor_worker_residuals
                .iter()
                .any(|residual| {
                    residual.parent_origin == occurrence.static_origin
                        && residual.sibling_position == sibling_position
                })
            {
                continue;
            }
            uses.insert((owner, occurrence.static_origin, sibling_position));
        }
    }
    uses.into_iter()
        .enumerate()
        .map(|(ordinal, (owner, parent_origin, sibling_position))| {
            let disposition = OperandEdgeDisposition::SpecializedOnlyLeaf;
            let (consumer_phase, operation, need, avail) =
                non_semantic_boundary_contract(disposition);
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| planner_capacity_error("recursor boundary identity exhausted"))?;
            let identity =
                BoundaryUseIdentity::Synthesized(0x4800_0000u32.checked_add(ordinal).ok_or_else(
                    || planner_capacity_error("recursor boundary identity exhausted"),
                )?);
            Ok(PlannedRecursorBoundaryUse {
                identity,
                owner,
                parent_origin,
                sibling_position,
                producer_phase: BoundaryUsePhase::SpecializedValue,
                consumer_phase,
                operation,
                need,
                disposition,
                avail,
            })
        })
        .collect()
}

fn build_lowering_boundary_uses(
    plan: &StaticTransitionPlan<'_>,
) -> Result<Vec<PlannedLoweringBoundaryUse>, CraneliftBackendError> {
    let mut keys = BTreeSet::new();
    let mut owner_bound_keys = BTreeSet::new();
    for occurrence in plan.source_occurrences.iter().flatten() {
        let origin = occurrence.static_origin;
        #[cfg(test)]
        if INCLUDE_TEST_FIXTURE_BOUNDARY_USE.with(Cell::get) {
            keys.insert((LoweringOnlyOperandEdge::TestFixtureResult, origin, 0));
        }
        match occurrence.expr {
            RuntimeExpr::CheckedComputationalIHInvocation { .. } => {
                keys.insert((
                    LoweringOnlyOperandEdge::CheckedComputationalIhMarker,
                    origin,
                    0,
                ));
            }
            RuntimeExpr::Call { .. } => {
                if !plan.functionized_units {
                    for edge in [
                        LoweringOnlyOperandEdge::RecursiveSourceDeclarationArgument,
                        LoweringOnlyOperandEdge::RecursiveDeclarationArgument,
                        LoweringOnlyOperandEdge::DeclarationCaptureSpecialization,
                    ] {
                        keys.insert((edge, origin, 0));
                    }
                }
            }
            RuntimeExpr::Construct { .. } => {}
            _ => {}
        }
        if let Some(join) = plan
            .join_results
            .get(origin.0 as usize)
            .and_then(Option::as_ref)
        {
            if !join.has_continuing_predecessor {
                continue;
            }
            if join.representation == JoinResultRepresentation::CarrierWord {
                let predecessor_positions: Box<dyn Iterator<Item = usize>> = match occurrence.expr {
                    RuntimeExpr::CheckedJoinSite { .. } => Box::new(0..1),
                    RuntimeExpr::If { .. } => Box::new(1..3),
                    RuntimeExpr::Match { cases, .. } => Box::new(1..1 + cases.len()),
                    RuntimeExpr::ComputationalMatch { cases, .. } => Box::new(1..1 + cases.len()),
                    RuntimeExpr::Call { .. } => Box::new(std::iter::empty()),
                    _ => Box::new(std::iter::empty()),
                };
                for position in predecessor_positions {
                    let predecessor = plan.semantic.child_origin(origin, position)?;
                    keys.insert((LoweringOnlyOperandEdge::JoinArm, predecessor, 0));
                }
            } else {
                keys.insert((LoweringOnlyOperandEdge::JoinArm, origin, 0));
            }
        }
    }
    let root_owner = plan
        .root_occurrence
        .map(|origin| plan.semantic.function_owner(origin))
        .transpose()?
        .flatten();
    let emitted_functions = if plan.functionized_units {
        plan.emittable_units()?
            .into_iter()
            .map(|unit| unit.function())
            .collect::<BTreeSet<_>>()
    } else {
        BTreeSet::new()
    };
    for descriptor in &plan.abi.descriptors {
        if (!plan.functionized_units
            || (Some(descriptor.function) != root_owner
                && emitted_functions.contains(&descriptor.function)))
            && plan
                .result_phases
                .get(descriptor.origin.0 as usize)
                .and_then(Option::as_ref)
                .is_some_and(|phase| phase.continues)
        {
            if plan.functionized_units {
                owner_bound_keys.insert((
                    LoweringOnlyOperandEdge::CallableCapsuleEscape,
                    descriptor.origin,
                    u32::MAX,
                    descriptor.function,
                ));
            } else {
                keys.insert((
                    LoweringOnlyOperandEdge::CallableCapsuleEscape,
                    descriptor.origin,
                    u32::MAX,
                ));
            }
        }
        let start = descriptor.slots.start as usize;
        let end = start
            .checked_add(descriptor.slots.len as usize)
            .ok_or_else(|| planner_capacity_error("ABI boundary-use range exhausted"))?;
        let slots = plan
            .abi
            .slots
            .get(start..end)
            .ok_or_else(|| planner_error("ABI boundary-use range is outside the slot plane"))?;
        if Some(descriptor.function) == root_owner {
            let mut input = 0u32;
            for slot in slots {
                if matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture) {
                    owner_bound_keys.insert((
                        LoweringOnlyOperandEdge::CallableCapsuleEscape,
                        descriptor.origin,
                        input,
                        descriptor.function,
                    ));
                    input = input.checked_add(1).ok_or_else(|| {
                        planner_capacity_error("ABI input boundary-use exhausted")
                    })?;
                }
            }
        }
    }
    for call in plan.emittable_call_edges()? {
        let descriptor = plan
            .abi
            .descriptors
            .iter()
            .find(|descriptor| descriptor.function == call.callee)
            .ok_or_else(|| planner_error("planned call has no callee ABI descriptor"))?;
        let start = descriptor.slots.start as usize;
        let end = start
            .checked_add(descriptor.slots.len as usize)
            .ok_or_else(|| planner_capacity_error("ABI call boundary-use range exhausted"))?;
        let slots = plan.abi.slots.get(start..end).ok_or_else(|| {
            planner_error("ABI call boundary-use range is outside the slot plane")
        })?;
        let mut input = 0u32;
        for slot in slots {
            if matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture) {
                owner_bound_keys.insert((
                    LoweringOnlyOperandEdge::CallableCapsuleEscape,
                    call.call_site_origin,
                    input,
                    call.caller,
                ));
                input = input
                    .checked_add(1)
                    .ok_or_else(|| planner_capacity_error("ABI input boundary-use exhausted"))?;
            }
        }
    }
    let mut resolved_keys = keys
        .into_iter()
        .map(|(edge, origin, position)| {
            let owner = plan
                .semantic
                .function_owner(origin)?
                .ok_or_else(|| planner_error("lowering boundary use has no function owner"))?;
            Ok((edge, origin, position, owner))
        })
        .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
    resolved_keys.extend(owner_bound_keys);
    resolved_keys.sort();
    let static_worker_bodies = plan
        .static_recursor_worker_residuals
        .iter()
        .map(|residual| residual.body_origin)
        .collect::<BTreeSet<_>>();
    let mut exact_keys = Vec::new();
    for key @ (edge, origin, position, _) in resolved_keys {
        exact_keys.push(key);
        if edge == LoweringOnlyOperandEdge::CallableCapsuleEscape
            && position != u32::MAX
            && static_worker_bodies.contains(&origin)
        {
            // A static worker has two distinct emitted call-input crossings:
            // the initial specialized closure materialization and the carried
            // recursive revisit. They require two identities; collapsing this
            // pair would turn a real repeated emission into an `or_insert(1)`.
            exact_keys.push(key);
        }
    }
    exact_keys
        .into_iter()
        .enumerate()
        .map(|(ordinal, (edge, origin, position, owner))| {
            let disposition = edge.disposition();
            let (consumer_phase, operation, need, avail) =
                non_semantic_boundary_contract(disposition);
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| planner_capacity_error("lowering boundary identity exhausted"))?;
            let identity =
                BoundaryUseIdentity::Synthesized(0x5000_0000u32.checked_add(ordinal).ok_or_else(
                    || planner_capacity_error("lowering boundary identity exhausted"),
                )?);
            Ok(PlannedLoweringBoundaryUse {
                identity,
                edge,
                origin,
                position,
                owner,
                producer_phase: BoundaryUsePhase::SpecializedValue,
                consumer_phase,
                operation,
                need,
                disposition,
                avail,
            })
        })
        .collect()
}

fn build_boundary_uses(
    plan: &StaticTransitionPlan<'_>,
) -> Result<Vec<PlannedBoundaryUse>, CraneliftBackendError> {
    let mut uses = plan
        .operand_edges
        .iter()
        .map(|edge| PlannedBoundaryUse {
            identity: BoundaryUseIdentity::Source {
                parent: edge.parent,
                child: edge.child,
                position: edge.position,
            },
            path: PlannedBoundaryUsePath::Source {
                parent: edge.parent,
                child: edge.child,
                position: edge.position,
                effect_operation: edge.effect_operation,
                effect_seat: edge.effect_seat,
            },
            producer_owner: edge.producer_owner,
            consumer_owner: edge.owner,
            producer_phase: edge.producer_phase,
            consumer_phase: edge.consumer_phase,
            operation: edge.operation,
            need: edge.need,
            disposition: edge.disposition,
            avail: edge.avail,
        })
        .chain(
            plan.static_recursor_worker_residuals
                .iter()
                .enumerate()
                .map(|(ordinal, residual)| {
                    let producer_owner = plan
                        .semantic
                        .function_owner(residual.producer_origin)?
                        .ok_or_else(|| {
                            planner_error("static recursor producer has no function owner")
                        })?;
                    let consumer_owner = plan
                        .semantic
                        .function_owner(residual.parent_origin)?
                        .ok_or_else(|| {
                            planner_error("static recursor consumer has no function owner")
                        })?;
                    let ordinal = u32::try_from(ordinal).map_err(|_| {
                        planner_capacity_error("static recursor boundary identity exhausted")
                    })?;
                    let identity = BoundaryUseIdentity::Synthesized(
                        0x4000_0000u32.checked_add(ordinal).ok_or_else(|| {
                            planner_capacity_error("static recursor boundary identity exhausted")
                        })?,
                    );
                    let (consumer_phase, operation, need, avail) =
                        non_semantic_boundary_contract(residual.disposition);
                    Ok(PlannedBoundaryUse {
                        identity,
                        path: PlannedBoundaryUsePath::StaticRecursorWorker {
                            parent_origin: residual.parent_origin,
                            producer_origin: residual.producer_origin,
                            sibling_position: residual.sibling_position,
                            closure_origin: residual.closure_origin,
                            body_origin: residual.body_origin,
                            declared_arity: residual.declared_arity,
                            captures: residual.captures.clone(),
                        },
                        producer_owner,
                        consumer_owner,
                        producer_phase: BoundaryUsePhase::SpecializedValue,
                        consumer_phase,
                        operation,
                        need,
                        disposition: residual.disposition,
                        avail,
                    })
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?
                .into_iter(),
        )
        .chain(
            plan.recursor_boundary_uses
                .iter()
                .map(|edge| PlannedBoundaryUse {
                    identity: edge.identity,
                    path: PlannedBoundaryUsePath::Synthesized {
                        origin: edge.parent_origin,
                        position: edge.sibling_position,
                    },
                    producer_owner: edge.owner,
                    consumer_owner: edge.owner,
                    producer_phase: edge.producer_phase,
                    consumer_phase: edge.consumer_phase,
                    operation: edge.operation,
                    need: edge.need,
                    disposition: edge.disposition,
                    avail: edge.avail,
                }),
        )
        .chain(
            plan.lowering_boundary_uses
                .iter()
                .map(|edge| PlannedBoundaryUse {
                    identity: edge.identity,
                    path: PlannedBoundaryUsePath::Synthesized {
                        origin: edge.origin,
                        position: edge.position,
                    },
                    producer_owner: edge.owner,
                    consumer_owner: edge.owner,
                    producer_phase: edge.producer_phase,
                    consumer_phase: edge.consumer_phase,
                    operation: edge.operation,
                    need: edge.need,
                    disposition: edge.disposition,
                    avail: edge.avail,
                }),
        )
        .collect::<Vec<_>>();
    let mut capture_ordinal = 0u32;
    for (worker_ordinal, residual) in plan.static_recursor_worker_residuals.iter().enumerate() {
        let worker_ordinal = u32::try_from(worker_ordinal)
            .map_err(|_| planner_capacity_error("static recursor boundary identity exhausted"))?;
        let worker_identity = BoundaryUseIdentity::Synthesized(
            0x4000_0000u32.checked_add(worker_ordinal).ok_or_else(|| {
                planner_capacity_error("static recursor boundary identity exhausted")
            })?,
        );
        let consumer_owner = plan
            .semantic
            .function_owner(residual.parent_origin)?
            .ok_or_else(|| planner_error("static recursor consumer has no function owner"))?;
        for capture in &residual.captures {
            let identity = BoundaryUseIdentity::Synthesized(
                0x4400_0000u32.checked_add(capture_ordinal).ok_or_else(|| {
                    planner_capacity_error("static recursor capture identity exhausted")
                })?,
            );
            capture_ordinal = capture_ordinal.checked_add(1).ok_or_else(|| {
                planner_capacity_error("static recursor capture identity exhausted")
            })?;
            let disposition = OperandEdgeDisposition::Forwarding;
            let (consumer_phase, operation, need, avail) =
                non_semantic_boundary_contract(disposition);
            uses.push(PlannedBoundaryUse {
                identity,
                path: PlannedBoundaryUsePath::StaticRecursorCapture {
                    worker_identity,
                    residual_id: residual.id,
                    parent_origin: residual.parent_origin,
                    producer_origin: residual.producer_origin,
                    sibling_position: residual.sibling_position,
                    closure_origin: residual.closure_origin,
                    ordinal: capture.ordinal,
                    capture: capture.clone(),
                },
                producer_owner: capture.owner,
                consumer_owner,
                producer_phase: BoundaryUsePhase::SpecializedValue,
                consumer_phase,
                operation,
                need,
                disposition,
                avail,
            });
        }
    }
    uses.sort_by_key(|edge| edge.identity);
    if uses
        .windows(2)
        .any(|pair| pair[0].identity == pair[1].identity)
    {
        return Err(planner_error(
            "planned boundary-use population contains a duplicate identity",
        ));
    }
    Ok(uses)
}

fn build_static_recursor_worker_residual(
    plan: &StaticTransitionPlan<'_>,
    parent_origin: StaticOriginId,
    producer_origin: StaticOriginId,
    sibling_position: usize,
    closure: &PlannedOccurrence<'_>,
) -> Result<PlannedStaticRecursorWorkerResidual, CraneliftBackendError> {
    let closure_origin = closure.static_origin;
    let body_origin = plan.semantic.child_origin(closure_origin, 0)?;
    let closure_owner = plan
        .semantic
        .function_owner(closure_origin)?
        .ok_or_else(|| planner_error("static recursor closure has no owner"))?;
    let (params, captures) = match closure.expr {
        RuntimeExpr::Closure {
            captures, params, ..
        } => {
            let captures = captures
                .iter()
                .enumerate()
                .map(|(ordinal, symbol)| {
                    Ok(StaticRecursorCaptureProvenance {
                        ordinal: u32::try_from(ordinal).map_err(|_| {
                            planner_capacity_error("static recursor capture ordinal exhausted")
                        })?,
                        owner: closure_owner,
                        closure_origin,
                        source: StaticRecursorCaptureSource::Seed(symbol.clone()),
                        phase: OperandEdgeDisposition::CallableCapture,
                    })
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
            (params, captures)
        }
        RuntimeExpr::LexicalClosure {
            captures, params, ..
        } => {
            let captures = captures
                .iter()
                .enumerate()
                .map(|(ordinal, _)| {
                    let child_position = 1 + ordinal;
                    let capture_origin =
                        plan.semantic.child_origin(closure_origin, child_position)?;
                    let edge = plan
                        .operand_edges
                        .iter()
                        .find(|edge| {
                            edge.parent == closure_origin
                                && edge.child == capture_origin
                                && edge.position as usize == child_position
                                && edge.role == SourceOperandRole::LexicalCapture
                        })
                        .ok_or_else(|| {
                            planner_error("static recursor capture has no planned callable edge")
                        })?;
                    if edge.disposition != OperandEdgeDisposition::CallableCapture {
                        return Err(planner_error(
                            "static recursor capture is not callable-capture phase",
                        ));
                    }
                    Ok(StaticRecursorCaptureProvenance {
                        ordinal: u32::try_from(ordinal).map_err(|_| {
                            planner_capacity_error("static recursor capture ordinal exhausted")
                        })?,
                        owner: edge.owner,
                        closure_origin,
                        source: StaticRecursorCaptureSource::Lexical(capture_origin),
                        phase: edge.disposition,
                    })
                })
                .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
            (params, captures)
        }
        _ => {
            return Err(planner_error(
                "static recursor child is not a closure occurrence",
            ));
        }
    };
    Ok(PlannedStaticRecursorWorkerResidual {
        id: StaticRecursorWorkerResidualId(closure_origin.0),
        parent_origin,
        producer_origin,
        sibling_position: u32::try_from(sibling_position)
            .map_err(|_| planner_capacity_error("static recursor sibling exhausted"))?,
        closure_origin,
        body_origin,
        declared_arity: u32::try_from(params.len())
            .map_err(|_| planner_capacity_error("static recursor arity exhausted"))?,
        captures,
        disposition: OperandEdgeDisposition::CallableCapture,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PlannedJoinResult {
    representation: JoinResultRepresentation,
    has_continuing_predecessor: bool,
}

/// The complete producer population for one exact carried `Match` scrutinee.
///
/// `Open` is a positive fact: at least one value-flow ingress is opaque,
/// untracked, or otherwise not closed by the planner. It is never inferred from
/// an empty observation. `Closed` is the monotone union of canonical
/// constructor identities supplied by exact source producers.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ScrutineeProducerSet {
    Open,
    Closed(Vec<ConstructorIdentity>),
}

impl ScrutineeProducerSet {
    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Open, _) | (_, Self::Open) => Self::Open,
            (Self::Closed(left), Self::Closed(right)) => {
                let mut union = left.clone();
                for identity in right {
                    if !union.contains(identity) {
                        union.push(*identity);
                    }
                }
                Self::Closed(union)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProducerFlowKind {
    Construct,
    Forward,
    Environment,
    Alternative,
    CallArgument,
    CallResult,
    Capture,
    Recursor,
    OpaqueIngress,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProducerFlowEdge {
    from: StaticOriginId,
    to: StaticOriginId,
    kind: ProducerFlowKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScrutineeProducerAuthority {
    producers: ScrutineeProducerSet,
    producer_origins: Vec<(ConstructorIdentity, BTreeSet<StaticOriginId>)>,
    flow: BTreeSet<ProducerFlowEdge>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum CaseEmissionStatus {
    Reachable,
    Eliminated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedCaseEmission {
    match_origin: StaticOriginId,
    scrutinee_origin: StaticOriginId,
    owner: PredeclaredFunctionId,
    phase: ResultPhase,
    ordinal: u32,
    body_origin: StaticOriginId,
    constructor: ConstructorIdentity,
    authority: ScrutineeProducerAuthority,
    status: CaseEmissionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum AggregateIdentity {
    Constructor(ConstructorIdentity),
    Record(Vec<FieldIdentity>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedAggregateChild {
    origin: StaticOriginId,
    position: u32,
    possible_owners: Vec<BoundaryReferentOwner>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedAggregateRepresentation {
    origin: StaticOriginId,
    owner: PredeclaredFunctionId,
    phase: ResultPhase,
    class: BoundaryClass,
    identity: AggregateIdentity,
    arity: u32,
    children: Vec<PlannedAggregateChild>,
    selected_owner: BoundaryReferentOwner,
    selected_tag: BoundaryTag,
}

/// The compiler-emission site of one synthesized aggregate occurrence.
///
/// Roles identify constructor semantics; sites distinguish repeated
/// occurrences of the same role in one effect lowering.  This closed sum is
/// planner input, not allocation authority.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum SynthesizedAggregateSite {
    IoError(u32),
    FileOperation,
    FilePathSome,
    FileError,
    ResourceKind(u8, u8),
    ResourceTraceIdentity,
    ResourceHostIo,
    ResourceClosed,
    ResourceMalformed,
    ResourceRightNotHeld,
    ResourceReleaseFailed,
    ResourceKindMismatch,
    ResourceBufferLimit,
    ResourceInvalidOffset,
    ResourceInvalidBounds,
    ResourceNoProgress,
    ReadBufferSpan,
    ReadTransferCount,
    ReadEof,
    ReadSome,
    WriteTransferCount,
    Wrote,
    Unit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlannedSynthesizedAggregateRepresentation {
    effect_origin: StaticOriginId,
    owner: PredeclaredFunctionId,
    phase: ResultPhase,
    site: SynthesizedAggregateSite,
    role: SynthesizedConstructorRole,
    arity: u32,
    children: Vec<Vec<BoundaryReferentOwner>>,
    selected_owner: BoundaryReferentOwner,
    selected_tag: BoundaryTag,
}

/// Opaque identity of one pre-planned compiler-synthesized aggregate.
///
/// Lowering may retain and clone the identity while it builds alternatives,
/// but only the planner can construct it or exchange it for the move-only
/// allocation token below.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) struct SynthesizedAggregateOccurrence {
    effect_origin: StaticOriginId,
    owner: PredeclaredFunctionId,
    site: SynthesizedAggregateSite,
}

/// Move-only authority for allocating one exact aggregate occurrence.
///
/// Lowering can read the selected row but cannot mint, transplant, or relabel
/// it. The exact owner/origin pair is consumed in the pre-publication ledger.
#[derive(Debug)]
pub(in crate::cranelift_backend) struct AggregateRepresentationToken {
    tag: BoundaryTag,
    class: BoundaryClass,
}

impl AggregateRepresentationToken {
    pub(in crate::cranelift_backend) fn tag(&self) -> BoundaryTag {
        self.tag
    }

    pub(in crate::cranelift_backend) fn class(&self) -> BoundaryClass {
        self.class
    }
}

/// Move-only authority for lowering one exact source case.
///
/// Fields are readers only and construction remains planner-private. Lowering
/// cannot mint a case, change its status, or substitute a case from another
/// match occurrence.
#[derive(Debug)]
pub(in crate::cranelift_backend) struct CaseEmissionToken {
    match_origin: StaticOriginId,
    ordinal: u32,
    body_origin: StaticOriginId,
    status: CaseEmissionStatus,
}

impl CaseEmissionToken {
    pub(in crate::cranelift_backend) fn is_reachable(&self) -> bool {
        self.status == CaseEmissionStatus::Reachable
    }
}

/// A nonzero identity for one exact trap value interned by the planner.
///
/// The word travels only through [`AbiCarrier::TrapWord`]. It is not a source
/// value and cannot be constructed by lowering.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct PlannedTrapIdentity(u32);

impl PlannedTrapIdentity {
    pub(in crate::cranelift_backend) fn abi_word(self) -> i64 {
        i64::from(self.0)
    }
}

#[derive(Clone)]
pub(in crate::cranelift_backend) struct StaticTransitionPlan<'src> {
    entries: Vec<StaticNodeId>,
    nodes: Vec<StaticNode>,
    edges: Vec<StaticEdge>,
    stores: Vec<PersistentStoreNode>,
    store_depths: Vec<u32>,
    evidence: Vec<EdgeEvidence>,
    planned_helpers: Vec<PlannedHelperKey>,
    semantic_sources: Vec<SemanticSourceSeed>,
    semantic_material: SemanticMaterialArena,
    semantic: SemanticPlane,
    /// `RT-FNSPLIT-B2R` — one representation/call-ABI descriptor per function
    /// unit in `semantic`'s validated owner partition.
    ///
    /// ⛔ **Inert.** This plane is *declared and validated*, never emitted from.
    /// It carries no `FunctionBuilder`, no `define_function`, no call edge and no
    /// encoder; `RT-FNSPLIT-B2F` performs the atomic switch-over that makes it
    /// live.
    abi: AbiPlane,
    /// The scheduling entry returned by the root visit. Kept separately from
    /// the root occurrence because computational matches make them differ.
    root_entry: Option<StaticNodeId>,
    root_ingress: AbiRootIngress,
    /// The **occurrence** origin of the whole program's root, stored at planning
    /// time.
    ///
    /// ⛔ It is not recoverable from `entries`, which holds scheduling entries: a
    /// root whose body is a `ComputationalMatch` schedules its scrutinee while
    /// its occurrence lives on the resume (D9/AC-15). Deriving one from the other
    /// afterwards is the conflation this field exists to prevent.
    root_occurrence: Option<StaticOriginId>,
    /// The **occurrence** origin of each transparent declaration, keyed by its
    /// symbol — likewise stored at planning time, not recovered from an entry.
    ///
    /// A declaration is planned as its own source occurrence, so its body's
    /// static name is reachable **by name** and needs no origin threaded into it.
    /// This is what makes `Lowered::DeclarationClosure`'s construction site
    /// asymmetric with the two `lower_expr` closure arms
    ///.
    declaration_occurrences: BTreeMap<String, StaticOriginId>,
    /// Exact trap values interned during the same occurrence visit that records
    /// their source semantics. Identity zero is reserved for "no trap".
    trap_catalog: Vec<RuntimeTrap>,
    /// Every planned source occurrence, **dense by origin ordinal**.
    ///
    /// `origin_of` is `StaticOriginId(node.0)`, so a node's origin *is* its index
    /// here. The table is written in the same visit that allocates the node's
    /// semantic seed — `expression_seed`, the one function every occurrence's
    /// seed passes through — so there is no second walk that could disagree with
    /// the first, and totality is a property of the construction rather than of
    /// an enumeration someone has to keep current.
    ///
    /// ⛔ `None` is a real answer, not a gap to paper over: a control node is a
    /// planned node with no source term, so its slot stays empty and a lookup on
    /// one is a **loud planner failure** rather than a substituted body.
    source_occurrences: Vec<Option<PlannedOccurrence<'src>>>,
    /// The closed result contract for every source occurrence that can create a
    /// lowering join.  Absence is meaningful for non-join occurrences.
    join_results: Vec<Option<PlannedJoinResult>>,
    /// The phase of every source result as derived by the same lexical
    /// value-flow walk that plans joins.
    result_phases: Vec<Option<ResultPhaseSummary>>,
    /// One exact, planner-derived case partition for every carried ordinary
    /// `Match`. The population closes after generated units and continuations.
    case_emissions: Vec<PlannedCaseEmission>,
    /// Exact reachable-case consumption ledger. It closes with the boundary
    /// ledger before any staged function is published.
    case_emission_consumption: RefCell<BTreeMap<(PredeclaredFunctionId, StaticOriginId, u32), u32>>,
    /// Owner-parametric representation selected for every reached source
    /// Constructor/Record occurrence after producer flow closes.
    aggregate_representations: Vec<PlannedAggregateRepresentation>,
    /// Exact compiler-synthesized Constructor occurrences derived from each
    /// carrier-required effect's closed emission schema.
    synthesized_aggregate_representations: Vec<PlannedSynthesizedAggregateRepresentation>,
    /// Exact source constructors whose process-result representation is the
    /// immediate exit-status lane rather than an aggregate allocation.
    terminal_exit_aggregate_origins: BTreeSet<StaticOriginId>,
    /// Exact aggregate allocations emitted by lowering.
    aggregate_representation_consumption:
        RefCell<BTreeMap<(PredeclaredFunctionId, StaticOriginId), u32>>,
    aggregate_representation_dispositions:
        RefCell<BTreeSet<(PredeclaredFunctionId, StaticOriginId)>>,
    synthesized_aggregate_representation_consumption:
        RefCell<BTreeMap<SynthesizedAggregateOccurrence, u32>>,
    /// The selected emission authority that determines whether owner crossings
    /// require operational carrier results.
    functionized_units: bool,
    /// One planner-derived entry for every operand-bearing positional source
    /// child. Lowering-only consumers live in [`LoweringOnlyOperandEdge`].
    operand_edges: Vec<PlannedOperandEdge>,
    /// Exact lowering consumption ledger. The plan is immutable during
    /// emission; only this monotone ledger changes. The complete ledger closes
    /// before any staged function definition is handed to the object module.
    operand_edge_consumption: RefCell<BTreeMap<BoundaryUseIdentity, u32>>,
    /// Planned uses proven unreachable by a lowering-time static selection.
    /// These identities are closed as dispositions, never masqueraded as
    /// emitted consumption.
    boundary_use_dispositions: RefCell<BTreeSet<BoundaryUseIdentity>>,
    /// The finite, interned population of out-of-line units that eliminate a
    /// statically known callable parameter from a transparent declaration ABI.
    static_callable_specializations: Vec<PlannedStaticCallableSpecialization>,
    /// One exact call-site edge per use of an interned specialization.
    static_callable_calls: Vec<PlannedStaticCallableCall>,
    /// Exact residual crossings materialized before the generated-unit fixed
    /// point closes. Lowering can only consume this population; it cannot
    /// recover a residual by searching source occurrences.
    static_recursor_worker_residuals: Vec<PlannedStaticRecursorWorkerResidual>,
    recursor_boundary_uses: Vec<PlannedRecursorBoundaryUse>,
    lowering_boundary_uses: Vec<PlannedLoweringBoundaryUse>,
    boundary_uses: Vec<PlannedBoundaryUse>,
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct BoundaryACensus {
    static_nodes: usize,
    edges: usize,
    planned_helpers: usize,
    persistent_store_nodes: usize,
    out_of_line_evidence_records: usize,
    max_helpers_per_static_source: usize,
    helper_key_bytes: usize,
    activation_frame_bytes: usize,
    store_node_bytes: usize,
    helper_key_schemas: usize,
    frame_schemas: usize,
    store_node_schemas: usize,
    static_node_id_bytes: usize,
    persistent_node_id_bytes: usize,
    max_logical_chain_depth: u32,
    max_environment_depth: u32,
    max_continuation_depth: u32,
    max_path_depth: u32,
    max_cleanup_depth: u32,
    max_affine_depth: u32,
    max_source_return_depth: u32,
    source_return_resume_nodes: usize,
    source_return_owned_resume_edges: usize,
    terminal_outgoing_edges: usize,
    recursive_lowering_frames: usize,
}

/// The planner-side material retained until one completed FunctionizedUnits
/// emission can be measured at Boundary B.
///
/// Unlike [`BoundaryACensus`], this is not itself a result row.  The lowering
/// collector takes this snapshot from the exact plan it subsequently emits and
/// publishes it only after every production CLIF body has been defined.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct ScaleBPlanCensus {
    pub(in crate::cranelift_backend) static_nodes: usize,
    pub(in crate::cranelift_backend) edges: usize,
    pub(in crate::cranelift_backend) planned_helpers: usize,
    pub(in crate::cranelift_backend) persistent_store_nodes: usize,
    pub(in crate::cranelift_backend) out_of_line_evidence_records: usize,
    pub(in crate::cranelift_backend) max_helpers_per_static_source: usize,
    pub(in crate::cranelift_backend) helper_key_bytes: usize,
    pub(in crate::cranelift_backend) activation_frame_bytes: usize,
    pub(in crate::cranelift_backend) store_node_bytes: usize,
    pub(in crate::cranelift_backend) helper_key_schemas: usize,
    pub(in crate::cranelift_backend) frame_schemas: usize,
    pub(in crate::cranelift_backend) store_node_schemas: usize,
    pub(in crate::cranelift_backend) static_node_id_bytes: usize,
    pub(in crate::cranelift_backend) persistent_node_id_bytes: usize,
    pub(in crate::cranelift_backend) max_logical_chain_depth: u32,
    pub(in crate::cranelift_backend) max_environment_depth: u32,
    pub(in crate::cranelift_backend) max_continuation_depth: u32,
    pub(in crate::cranelift_backend) max_path_depth: u32,
    pub(in crate::cranelift_backend) max_cleanup_depth: u32,
    pub(in crate::cranelift_backend) max_affine_depth: u32,
    pub(in crate::cranelift_backend) max_source_return_depth: u32,
    pub(in crate::cranelift_backend) source_return_resume_nodes: usize,
    pub(in crate::cranelift_backend) source_return_owned_resume_edges: usize,
    pub(in crate::cranelift_backend) terminal_outgoing_edges: usize,
    pub(in crate::cranelift_backend) recursive_lowering_frames: usize,
    pub(in crate::cranelift_backend) distinct_interned_semantic_states: usize,
    pub(in crate::cranelift_backend) defined_helpers: usize,
    pub(in crate::cranelift_backend) descriptor_construction_work: usize,
    pub(in crate::cranelift_backend) descriptor_comparison_work: usize,
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct BoundaryB1Census {
    opcode_vocabulary: usize,
    distinct_origins: usize,
    ir_records: usize,
    semantic_edges: usize,
    /// The number of **function units** — `entries.len() + count(StaticBody
    /// edges)`. Renamed from `helper_definitions` by `RT-FNSPLIT-B2O` `AC-6`.
    ///
    /// ⚠ **This field is the one re-baselined quantity that cannot fail
    /// loudly, which is why the rename is an acceptance criterion and not
    /// tidiness.** It is a *reported metric*: its only consumer asserts that the
    /// second finite difference across `n = 3..7` is zero — affine scaling — and
    /// asserts **no absolute value**. `RT-FNSPLIT-B2O` changed this quantity from
    /// "one definition per planned node" to "one per function unit"; both are
    /// affine in `n`, so that assertion passed before and after. Nothing failed
    /// and nothing warned. A name still reporting `helper_definitions` for a
    /// number whose meaning had changed underneath it would be worse than a
    /// rename precisely because there is no red to notice.
    function_units: usize,
    definitions_per_origin: usize,
    all_out_of_line_operand_elements: usize,
    duplicate_origin_definitions: usize,
    post_origin_clones: usize,
    max_definitions_per_origin: usize,
    descriptor_bytes: usize,
    program_bytes: usize,
    record_bytes: usize,
    operand_element_bytes: usize,
    capture_layout_bytes: usize,
    capture_slot_bytes: usize,
    ruled_child_bytes: usize,
    function_bytes: usize,
}

struct Planner<'src> {
    plan: StaticTransitionPlan<'src>,
    store_interner: BTreeMap<PersistentStoreNode, PersistentNodeId>,
    next_source: u32,
    terminal: StaticNodeId,
    trap_terminal: StaticNodeId,
}

fn planner_error(detail: impl Into<String>) -> CraneliftBackendError {
    backend(BackendFailure::PlannerInvariant(detail.into()))
}

fn planner_capacity_error(detail: impl Into<String>) -> CraneliftBackendError {
    unsupported("NativeStaticTransitionPlanner", detail)
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ResultPhase {
    SpecializedOnly,
    CarrierRequired,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticRecursorResidualMatrixMutation {
    Exact,
    OmitFirst,
    ReclassifyFirst,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum SynthesizedConsumptionMutation {
    Exact,
    OmitFirst,
    RepeatFirst,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum StaticRecursorConsumptionMutation {
    Exact,
    OmitFirst,
    RepeatFirst,
}

#[cfg(test)]
thread_local! {
    static D8_FORCE_VARIABLE_SPECIALIZED: Cell<bool> = const { Cell::new(false) };
    static D8_REMOVE_VARIABLE_CALLABLE_SEED: Cell<bool> = const { Cell::new(false) };
    static DECLARATION_CLOSURE_DROP_CALLABLE_PHASE: Cell<bool> =
        const { Cell::new(false) };
    static D7_OMIT_LEXICAL_CAPTURE_EDGE: Cell<bool> = const { Cell::new(false) };
    static STATIC_RECURSOR_RESIDUAL_MATRIX_MUTATION:
        Cell<StaticRecursorResidualMatrixMutation> =
        const { Cell::new(StaticRecursorResidualMatrixMutation::Exact) };
    static INCLUDE_TEST_FIXTURE_BOUNDARY_USE: Cell<bool> = const { Cell::new(false) };
    static SYNTHESIZED_CONSUMPTION_MUTATION: Cell<SynthesizedConsumptionMutation> =
        const { Cell::new(SynthesizedConsumptionMutation::Exact) };
    static SYNTHESIZED_CONSUMPTION_MUTATED: Cell<bool> = const { Cell::new(false) };
    static STATIC_RECURSOR_CONSUMPTION_MUTATION: Cell<StaticRecursorConsumptionMutation> =
        const { Cell::new(StaticRecursorConsumptionMutation::Exact) };
    static STATIC_RECURSOR_CONSUMPTION_MUTATED: Cell<bool> = const { Cell::new(false) };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ResultPhaseSummary {
    /// Representation of this value itself. This is the earlier bound-value
    /// seed: inserting the summary into an environment preserves a carried
    /// value recovered through `Var`.
    phase: ResultPhase,
    continues: bool,
    /// Representation produced by invoking this value, when it is callable.
    /// Keeping this orthogonal to `phase` closes the bound-lexical-closure form
    /// without weakening the value's specialized closure representation.
    callable_result: Option<ResultPhase>,
}

impl ResultPhaseSummary {
    const TRAP: Self = Self {
        phase: ResultPhase::SpecializedOnly,
        continues: false,
        callable_result: None,
    };

    const SPECIALIZED: Self = Self {
        phase: ResultPhase::SpecializedOnly,
        continues: true,
        callable_result: None,
    };

    fn carrier() -> Self {
        Self {
            phase: ResultPhase::CarrierRequired,
            continues: true,
            callable_result: None,
        }
    }

    fn callable(result: ResultPhase) -> Self {
        Self {
            phase: ResultPhase::SpecializedOnly,
            continues: true,
            callable_result: Some(result),
        }
    }

    fn join(self, other: Self) -> Self {
        Self {
            phase: self.phase.max(other.phase),
            continues: self.continues || other.continues,
            callable_result: self.callable_result.max(other.callable_result),
        }
    }

    fn sequence(self, other: Self) -> Self {
        Self {
            phase: self.phase.max(other.phase),
            continues: self.continues && other.continues,
            // A sequence returns its right-hand value. Callable provenance is
            // therefore not an effect to accumulate from the left.
            callable_result: other.callable_result,
        }
    }
}

fn is_source_join(expr: &RuntimeExpr) -> bool {
    matches!(
        expr,
        RuntimeExpr::CheckedJoinSite { .. }
            | RuntimeExpr::If { .. }
            | RuntimeExpr::Match { .. }
            | RuntimeExpr::ComputationalMatch { .. }
            | RuntimeExpr::Call { .. }
    )
}

pub(in crate::cranelift_backend) fn planned_partiality_trap(
    primitive: &crate::RuntimePrimitive,
) -> Option<RuntimeTrap> {
    match &primitive.partiality {
        RuntimePartiality::CheckedTrap { obligation } => {
            let message = if obligation.ends_with(".bounds") {
                format!("{} bounds obligation failed", primitive.symbol)
            } else {
                format!("{} checked partiality trapped", primitive.symbol)
            };
            Some(RuntimeTrap {
                code: RuntimeTrapCode::ExplicitTrap,
                message,
            })
        }
        RuntimePartiality::TrustedTrap { .. } => Some(RuntimeTrap {
            code: RuntimeTrapCode::ExplicitTrap,
            message: format!("{} trusted partiality trapped", primitive.symbol),
        }),
        RuntimePartiality::Total
        | RuntimePartiality::SafeOption { .. }
        | RuntimePartiality::SafeResult { .. } => None,
    }
}

/// Compute the result phase from semantic result edges, never from arm order or
/// an emitted operand.  The match is intentionally exhaustive: a new source
/// form cannot silently inherit `SpecializedOnly`.
fn summarize_result_phase(
    plan: &StaticTransitionPlan<'_>,
    origin: StaticOriginId,
    functionized_units: bool,
    environment: &[ResultPhaseSummary],
    joins: &mut [Option<PlannedJoinResult>],
    phases: &mut [Option<ResultPhaseSummary>],
) -> Result<ResultPhaseSummary, CraneliftBackendError> {
    let occurrence = plan
        .source_occurrences
        .get(origin.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| planner_error("phase plan names no source occurrence"))?;
    if occurrence.static_origin != origin {
        return Err(planner_error(
            "phase plan occurrence disagrees with its preallocated origin",
        ));
    }
    let expr = occurrence.expr;
    let child = |position| plan.semantic.child_origin(origin, position);
    let mut summarize_child = |position: usize,
                               environment: &[ResultPhaseSummary],
                               joins: &mut [Option<PlannedJoinResult>]|
     -> Result<ResultPhaseSummary, CraneliftBackendError> {
        let child_origin = child(position)?;
        let crosses_owner = plan.semantic.crosses_function_owner(origin, child_origin)?;
        let child_environment = if crosses_owner {
            result_phase_environment_for_owner(plan, child_origin, functionized_units)?
        } else {
            environment.to_vec()
        };
        let mut summary = summarize_result_phase(
            plan,
            child_origin,
            functionized_units,
            &child_environment,
            joins,
            phases,
        )?;
        if functionized_units && summary.continues && crosses_owner {
            summary.phase = ResultPhase::CarrierRequired;
        }
        Ok(summary)
    };
    let summary = match expr {
        RuntimeExpr::Trap(_) => ResultPhaseSummary::TRAP,
        RuntimeExpr::CheckedJoinSite { .. }
        | RuntimeExpr::CheckedSubcontinuationFrame { .. }
        | RuntimeExpr::CheckedComputationalIHSlots { .. } => {
            summarize_child(0, environment, joins)?
        }
        // These markers are the static call-template seeds consumed by the
        // functionized emitter. Their result is a declared-unit carrier even
        // when the wrapped source spelling itself is specialized.
        RuntimeExpr::CheckedRecursiveInvocation { .. }
        | RuntimeExpr::CheckedComputationalIHInvocation { .. } => {
            let nested = summarize_child(0, environment, joins)?;
            if functionized_units && nested.continues {
                ResultPhaseSummary::carrier()
            } else {
                nested
            }
        }
        RuntimeExpr::Let { .. } => {
            let value = summarize_child(0, environment, joins)?;
            let mut body_environment = Vec::with_capacity(1 + environment.len());
            body_environment.push(ResultPhaseSummary {
                continues: true,
                ..value
            });
            body_environment.extend_from_slice(environment);
            value.sequence(summarize_child(1, &body_environment, joins)?)
        }
        RuntimeExpr::If { .. } => {
            summarize_child(1, environment, joins)?.join(summarize_child(2, environment, joins)?)
        }
        RuntimeExpr::Match { cases, .. } => {
            let scrutinee = summarize_child(0, environment, joins)?;
            let mut result = ResultPhaseSummary::TRAP;
            for (index, case) in cases.iter().enumerate() {
                // A case projection preserves the scrutinee's representation:
                // fields of a carried constructor remain carried, while native
                // and borrowed specialized scrutinees yield specialized fields.
                let mut case_environment = Vec::with_capacity(case.binders + environment.len());
                case_environment.extend((0..case.binders).map(|_| ResultPhaseSummary {
                    continues: true,
                    ..scrutinee
                }));
                case_environment.extend_from_slice(environment);
                result = result.join(summarize_child(1 + index, &case_environment, joins)?);
            }
            result
        }
        RuntimeExpr::ComputationalMatch {
            scrutinee, cases, ..
        } => {
            let scrutinee_summary = summarize_child(0, environment, joins)?;
            let mut result = ResultPhaseSummary::TRAP;
            for (index, case) in cases.iter().enumerate() {
                let case_binders = case
                    .argument_binders
                    .checked_add(case.recursive_positions.len())
                    .ok_or_else(|| planner_capacity_error("phase-plan case arity exhausted"))?;
                let mut case_environment = Vec::with_capacity(case_binders + environment.len());
                // Lowering installs `[IHs, argument binders, outer env]`.
                // Functionized IHs are declared-unit results; argument binders
                // preserve the scrutinee's representation.
                case_environment.extend(case.recursive_positions.iter().map(|_| {
                    if functionized_units {
                        ResultPhaseSummary::carrier()
                    } else {
                        ResultPhaseSummary::SPECIALIZED
                    }
                }));
                case_environment.extend((0..case.argument_binders).map(|_| ResultPhaseSummary {
                    phase: scrutinee_summary.phase,
                    continues: true,
                    callable_result: scrutinee_summary.callable_result,
                }));
                case_environment.extend_from_slice(environment);
                result = result.join(summarize_child(1 + index, &case_environment, joins)?);
            }
            let scrutinee_origin = child(0)?;
            if let RuntimeExpr::Construct { args, .. } = scrutinee.as_ref() {
                let mut carries_recursive_unit = false;
                'cases: for case in cases {
                    for position in case.recursive_positions.iter().copied() {
                        let Some(RuntimeExpr::LexicalClosure { captures, .. }) = args.get(position)
                        else {
                            continue;
                        };
                        if !captures.is_empty() {
                            continue;
                        }
                        let argument_origin =
                            plan.semantic.child_origin(scrutinee_origin, position)?;
                        let body_origin = plan.semantic.child_origin(argument_origin, 0)?;
                        if plan.semantic.crosses_function_owner(origin, body_origin)? {
                            carries_recursive_unit = true;
                            break 'cases;
                        }
                    }
                }
                if functionized_units && result.continues && carries_recursive_unit {
                    result.phase = ResultPhase::CarrierRequired;
                }
            }
            // Producer-local result joins forward the value after this
            // computational eliminator has run, not the raw producer syntax.
            // Raise only the shared result-position population; argument and
            // let-value joins still carry their own independently summarized
            // representation.
            if functionized_units {
                for join_origin in plan.source_result_origins_in_owner_subtree(scrutinee_origin)? {
                    if joins
                        .get(join_origin.0 as usize)
                        .and_then(Option::as_ref)
                        .is_none()
                    {
                        continue;
                    }
                    let join = joins
                        .get_mut(join_origin.0 as usize)
                        .and_then(Option::as_mut)
                        .ok_or_else(|| {
                            planner_error(
                                "computational result flow names an unplanned source join",
                            )
                        })?;
                    join.representation = JoinResultRepresentation::CarrierWord;
                }
            }
            result
        }
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            let mut result = (0..args.len()).try_fold(
                ResultPhaseSummary::SPECIALIZED,
                |summary, position| {
                    Ok(summary.sequence(summarize_child(position, environment, joins)?))
                },
            )?;
            result.callable_result = None;
            result
        }
        RuntimeExpr::Record { fields } => {
            let mut result = (0..fields.len()).try_fold(
                ResultPhaseSummary::SPECIALIZED,
                |summary, position| {
                    Ok(summary.sequence(summarize_child(position, environment, joins)?))
                },
            )?;
            result.callable_result = None;
            result
        }
        RuntimeExpr::Project { .. } => summarize_child(0, environment, joins)?,
        RuntimeExpr::Call { args, .. } => {
            let callee = summarize_child(0, environment, joins)?;
            let mut result = callee;
            for position in 0..args.len() {
                result = result.sequence(summarize_child(1 + position, environment, joins)?);
            }
            if functionized_units && callee.callable_result == Some(ResultPhase::CarrierRequired) {
                result.phase = ResultPhase::CarrierRequired;
            }
            // This planner tracks the representation of the call result, not
            // higher-order provenance of values returned by an opaque call.
            result.callable_result = None;
            result
        }
        RuntimeExpr::Var(index) => {
            let phase = environment
                .get(*index as usize)
                .copied()
                .unwrap_or(ResultPhaseSummary::SPECIALIZED);
            #[cfg(test)]
            if D8_FORCE_VARIABLE_SPECIALIZED.with(Cell::get) {
                ResultPhaseSummary::SPECIALIZED
            } else {
                if D8_REMOVE_VARIABLE_CALLABLE_SEED.with(Cell::get) {
                    ResultPhaseSummary {
                        callable_result: None,
                        ..phase
                    }
                } else {
                    phase
                }
            }
            #[cfg(not(test))]
            phase
        }
        RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. } => {
            let body_origin = child(0)?;
            ResultPhaseSummary::callable(
                if functionized_units
                    && plan.semantic.crosses_function_owner(origin, body_origin)?
                {
                    ResultPhase::CarrierRequired
                } else {
                    ResultPhase::SpecializedOnly
                },
            )
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            let Some(declaration_origin) =
                plan.declaration_occurrences.get(symbol.as_str()).copied()
            else {
                return Ok(ResultPhaseSummary::SPECIALIZED);
            };
            let declaration = plan
                .source_occurrences
                .get(declaration_origin.0 as usize)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    planner_error(
                        "declaration reference phase has no planned declaration occurrence",
                    )
                })?;
            match declaration.expr {
                RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. } => {
                    #[cfg(test)]
                    if DECLARATION_CLOSURE_DROP_CALLABLE_PHASE.with(Cell::get) {
                        ResultPhaseSummary::SPECIALIZED
                    } else {
                        ResultPhaseSummary::callable(if functionized_units {
                            ResultPhase::CarrierRequired
                        } else {
                            ResultPhase::SpecializedOnly
                        })
                    }
                    #[cfg(not(test))]
                    ResultPhaseSummary::callable(if functionized_units {
                        ResultPhase::CarrierRequired
                    } else {
                        ResultPhase::SpecializedOnly
                    })
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
                | RuntimeExpr::Trap(_) => ResultPhaseSummary::SPECIALIZED,
            }
        }
        RuntimeExpr::Value(_)
        | RuntimeExpr::ImportedDeclarationRef { .. }
        | RuntimeExpr::Effect { .. } => ResultPhaseSummary::SPECIALIZED,
    };
    if is_source_join(expr) {
        let result = PlannedJoinResult {
            representation: match summary.phase {
                ResultPhase::SpecializedOnly => JoinResultRepresentation::NativeScalarPair,
                ResultPhase::CarrierRequired => JoinResultRepresentation::CarrierWord,
            },
            has_continuing_predecessor: summary.continues,
        };
        let entry = joins
            .get_mut(origin.0 as usize)
            .ok_or_else(|| planner_error("phase-plan join origin is outside the plan"))?;
        *entry = Some(match *entry {
            Some(previous) => PlannedJoinResult {
                representation: if previous.representation == JoinResultRepresentation::CarrierWord
                    || result.representation == JoinResultRepresentation::CarrierWord
                {
                    JoinResultRepresentation::CarrierWord
                } else {
                    JoinResultRepresentation::NativeScalarPair
                },
                has_continuing_predecessor: previous.has_continuing_predecessor
                    || result.has_continuing_predecessor,
            },
            None => result,
        });
    }
    let slot = phases
        .get_mut(origin.0 as usize)
        .ok_or_else(|| planner_error("result phase origin is outside the plan"))?;
    if slot.is_some_and(|previous| previous != summary) {
        return Err(planner_error(
            "one source occurrence has conflicting result-phase authorities",
        ));
    }
    *slot = Some(summary);
    Ok(summary)
}

fn result_phase_environment_for_owner(
    plan: &StaticTransitionPlan<'_>,
    origin: StaticOriginId,
    functionized_units: bool,
) -> Result<Vec<ResultPhaseSummary>, CraneliftBackendError> {
    let Some(function) = plan.semantic.function_owner(origin)? else {
        return Ok(Vec::new());
    };
    let descriptor = plan
        .abi
        .descriptors
        .iter()
        .find(|descriptor| descriptor.function == function)
        .ok_or_else(|| planner_error("phase plan owner has no ABI descriptor"))?;
    let start = descriptor.slots.start as usize;
    let end = start
        .checked_add(descriptor.slots.len as usize)
        .ok_or_else(|| planner_capacity_error("phase-plan ABI slot range exhausted"))?;
    let slots = plan
        .abi
        .slots
        .get(start..end)
        .ok_or_else(|| planner_error("phase plan ABI slot range is outside the plane"))?;
    Ok(slots
        .iter()
        .filter(|slot| matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture))
        .map(|slot| {
            // The ABI plane remains validated while RecursiveDescent is
            // selected, but it is inert there. Its slots therefore cannot
            // impose carrier storage on the retained lowering authority.
            if !functionized_units {
                return ResultPhaseSummary::SPECIALIZED;
            }
            // The process pair is the closed exception to generic ValueWord
            // inputs: the root unit recovers these two role-keyed values as a
            // borrowed process input and a capability token. Every other
            // parameter/capture remains an opaque carried word.
            if matches!(
                descriptor.definition,
                AbiUnitDefinition::SchedulingEntry {
                    ingress: AbiSchedulingIngress::ProcessPair,
                }
            ) && slot.kind == AbiSlotKind::Parameter
            {
                ResultPhaseSummary::SPECIALIZED
            } else {
                ResultPhaseSummary::carrier()
            }
        })
        .collect())
}

fn build_join_result_plan(
    plan: &StaticTransitionPlan<'_>,
    functionized_units: bool,
) -> Result<
    (
        Vec<Option<PlannedJoinResult>>,
        Vec<Option<ResultPhaseSummary>>,
    ),
    CraneliftBackendError,
> {
    let mut joins = vec![None; plan.source_occurrences.len()];
    let mut phases = vec![None; plan.source_occurrences.len()];
    for descriptor in &plan.abi.descriptors {
        let mut root = descriptor.origin;
        if let Some(root_occurrence) = plan.root_occurrence {
            if plan.semantic.function_owner(root_occurrence)? == Some(descriptor.function) {
                root = root_occurrence;
            }
        }
        let environment = result_phase_environment_for_owner(plan, root, functionized_units)?;
        summarize_result_phase(
            plan,
            root,
            functionized_units,
            &environment,
            &mut joins,
            &mut phases,
        )?;
    }
    for occurrence in plan.source_occurrences.iter().flatten() {
        if is_source_join(occurrence.expr) && joins[occurrence.static_origin.0 as usize].is_none() {
            let environment = result_phase_environment_for_owner(
                plan,
                occurrence.static_origin,
                functionized_units,
            )?;
            summarize_result_phase(
                plan,
                occurrence.static_origin,
                functionized_units,
                &environment,
                &mut joins,
                &mut phases,
            )?;
        }
    }
    Ok((joins, phases))
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProducerFact {
    population: ScrutineeProducerSet,
    producer_origins: Vec<(ConstructorIdentity, BTreeSet<StaticOriginId>)>,
    flow: BTreeSet<ProducerFlowEdge>,
    frontier: BTreeSet<StaticOriginId>,
}

impl ProducerFact {
    fn empty() -> Self {
        Self {
            population: ScrutineeProducerSet::Closed(Vec::new()),
            producer_origins: Vec::new(),
            flow: BTreeSet::new(),
            frontier: BTreeSet::new(),
        }
    }

    fn open(origin: StaticOriginId) -> Self {
        Self {
            population: ScrutineeProducerSet::Open,
            producer_origins: Vec::new(),
            flow: BTreeSet::from([ProducerFlowEdge {
                from: origin,
                to: origin,
                kind: ProducerFlowKind::OpaqueIngress,
            }]),
            frontier: BTreeSet::from([origin]),
        }
    }

    fn constructor(origin: StaticOriginId, identity: ConstructorIdentity) -> Self {
        Self {
            population: ScrutineeProducerSet::Closed(vec![identity]),
            producer_origins: vec![(identity, BTreeSet::from([origin]))],
            flow: BTreeSet::from([ProducerFlowEdge {
                from: origin,
                to: origin,
                kind: ProducerFlowKind::Construct,
            }]),
            frontier: BTreeSet::from([origin]),
        }
    }

    fn join(&self, other: &Self) -> Self {
        let mut producer_origins = self.producer_origins.clone();
        for (identity, origins) in &other.producer_origins {
            if let Some((_, known)) = producer_origins
                .iter_mut()
                .find(|(candidate, _)| candidate == identity)
            {
                known.extend(origins.iter().copied());
            } else {
                producer_origins.push((*identity, origins.clone()));
            }
        }
        Self {
            population: self.population.join(&other.population),
            producer_origins,
            flow: self.flow.union(&other.flow).copied().collect(),
            frontier: self.frontier.union(&other.frontier).copied().collect(),
        }
    }

    fn forwarded(mut self, to: StaticOriginId, kind: ProducerFlowKind) -> Self {
        for from in self.frontier.iter().copied() {
            self.flow.insert(ProducerFlowEdge { from, to, kind });
        }
        self.frontier.clear();
        self.frontier.insert(to);
        self
    }

    fn authority(&self) -> ScrutineeProducerAuthority {
        ScrutineeProducerAuthority {
            producers: self.population.clone(),
            producer_origins: self.producer_origins.clone(),
            flow: self.flow.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct ProducerCallable {
    closure_origin: StaticOriginId,
    body_origin: StaticOriginId,
    recursor_origin: Option<StaticOriginId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProducerCallableSet {
    Open,
    Closed(BTreeSet<ProducerCallable>),
}

impl ProducerCallableSet {
    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Open, _) | (_, Self::Open) => Self::Open,
            (Self::Closed(left), Self::Closed(right)) => {
                Self::Closed(left.union(right).copied().collect())
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ReferentOwnerFact {
    Bottom,
    Closed(BTreeSet<BoundaryReferentOwner>),
    Unrepresented,
}

impl ReferentOwnerFact {
    fn owner(owner: BoundaryReferentOwner) -> Self {
        Self::Closed(BTreeSet::from([owner]))
    }

    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Bottom, value) | (value, Self::Bottom) => value.clone(),
            (Self::Unrepresented, _) | (_, Self::Unrepresented) => Self::Unrepresented,
            (Self::Closed(left), Self::Closed(right)) => {
                Self::Closed(left.union(right).copied().collect())
            }
        }
    }

    fn aggregate(children: &[ProducerValue]) -> Self {
        if children
            .iter()
            .any(|child| child.referent_owners == Self::Unrepresented)
        {
            return Self::Unrepresented;
        }
        if children
            .iter()
            .any(|child| child.referent_owners == Self::Bottom)
        {
            return Self::Bottom;
        }
        let invocation_owned = children.iter().any(|child| {
            matches!(
                &child.referent_owners,
                Self::Closed(owners)
                    if owners.contains(&BoundaryReferentOwner::InvocationArena)
            )
        });
        Self::owner(if invocation_owned {
            BoundaryReferentOwner::InvocationArena
        } else {
            BoundaryReferentOwner::PersistentStore
        })
    }

    fn closed_owners(&self) -> Option<Vec<BoundaryReferentOwner>> {
        match self {
            Self::Closed(owners) => Some(owners.iter().copied().collect()),
            Self::Bottom | Self::Unrepresented => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ProducerValue {
    constructors: ProducerFact,
    constructor_payloads: Vec<(ConstructorIdentity, Vec<ProducerValue>)>,
    record_fields: Vec<(FieldIdentity, ProducerValue)>,
    callables: ProducerCallableSet,
    referent_owners: ReferentOwnerFact,
    aggregate_origins: BTreeSet<StaticOriginId>,
    effect_origins: BTreeSet<StaticOriginId>,
    carried: bool,
}

impl ProducerValue {
    fn empty() -> Self {
        Self {
            constructors: ProducerFact::empty(),
            constructor_payloads: Vec::new(),
            record_fields: Vec::new(),
            callables: ProducerCallableSet::Closed(BTreeSet::new()),
            referent_owners: ReferentOwnerFact::Bottom,
            aggregate_origins: BTreeSet::new(),
            effect_origins: BTreeSet::new(),
            carried: false,
        }
    }

    fn open(origin: StaticOriginId) -> Self {
        Self {
            constructors: ProducerFact::open(origin),
            constructor_payloads: Vec::new(),
            record_fields: Vec::new(),
            callables: ProducerCallableSet::Open,
            referent_owners: ReferentOwnerFact::Unrepresented,
            aggregate_origins: BTreeSet::new(),
            effect_origins: BTreeSet::new(),
            carried: false,
        }
    }

    fn owner(owner: BoundaryReferentOwner) -> Self {
        Self {
            constructors: ProducerFact::empty(),
            constructor_payloads: Vec::new(),
            record_fields: Vec::new(),
            callables: ProducerCallableSet::Closed(BTreeSet::new()),
            referent_owners: ReferentOwnerFact::owner(owner),
            aggregate_origins: BTreeSet::new(),
            effect_origins: BTreeSet::new(),
            carried: false,
        }
    }

    fn join(&self, other: &Self) -> Self {
        let mut constructor_payloads = self.constructor_payloads.clone();
        for (identity, incoming) in &other.constructor_payloads {
            if let Some((_, known)) = constructor_payloads
                .iter_mut()
                .find(|(candidate, _)| candidate == identity)
            {
                if known.len() == incoming.len() {
                    for (slot, value) in known.iter_mut().zip(incoming) {
                        *slot = slot.join(value);
                    }
                } else {
                    known.clear();
                }
            } else {
                constructor_payloads.push((*identity, incoming.clone()));
            }
        }
        let mut record_fields = self.record_fields.clone();
        for (identity, incoming) in &other.record_fields {
            if let Some((_, known)) = record_fields
                .iter_mut()
                .find(|(candidate, _)| candidate == identity)
            {
                *known = known.join(incoming);
            } else {
                record_fields.push((*identity, incoming.clone()));
            }
        }
        Self {
            constructors: self.constructors.join(&other.constructors),
            constructor_payloads,
            record_fields,
            callables: self.callables.join(&other.callables),
            referent_owners: self.referent_owners.join(&other.referent_owners),
            aggregate_origins: self
                .aggregate_origins
                .union(&other.aggregate_origins)
                .copied()
                .collect(),
            effect_origins: self
                .effect_origins
                .union(&other.effect_origins)
                .copied()
                .collect(),
            carried: self.carried || other.carried,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AggregateObservation {
    class: BoundaryClass,
    identity: AggregateIdentity,
    children: Vec<(StaticOriginId, ReferentOwnerFact)>,
    carried: bool,
}

impl AggregateObservation {
    fn join(&self, other: &Self) -> Result<Self, CraneliftBackendError> {
        if self.class != other.class
            || self.identity != other.identity
            || self.children.len() != other.children.len()
            || self
                .children
                .iter()
                .zip(&other.children)
                .any(|((left, _), (right, _))| left != right)
        {
            return Err(planner_error(
                "aggregate occurrence changed class, identity, arity, or child origin",
            ));
        }
        Ok(Self {
            class: self.class,
            identity: self.identity.clone(),
            children: self
                .children
                .iter()
                .zip(&other.children)
                .map(|((origin, left), (_, right))| (*origin, left.join(right)))
                .collect(),
            carried: self.carried || other.carried,
        })
    }
}

struct ProducerAnalysis<'plan, 'src> {
    plan: &'plan StaticTransitionPlan<'src>,
    static_recursor_worker_residuals: Vec<PlannedStaticRecursorWorkerResidual>,
    active_owner: PredeclaredFunctionId,
    inputs: BTreeMap<PredeclaredFunctionId, Vec<ProducerValue>>,
    results: BTreeMap<PredeclaredFunctionId, ProducerValue>,
    captures: BTreeMap<StaticOriginId, Vec<ProducerValue>>,
    computational_scrutinees: BTreeMap<(StaticOriginId, PredeclaredFunctionId), ProducerValue>,
    computational_results: BTreeMap<(StaticOriginId, PredeclaredFunctionId), ProducerValue>,
    match_scrutinees: BTreeMap<(StaticOriginId, PredeclaredFunctionId), ProducerFact>,
    aggregate_occurrences: BTreeMap<(StaticOriginId, PredeclaredFunctionId), AggregateObservation>,
    effect_operands: BTreeMap<(StaticOriginId, PredeclaredFunctionId), Vec<ReferentOwnerFact>>,
    carried_aggregates: BTreeSet<StaticOriginId>,
    carried_effects: BTreeSet<StaticOriginId>,
    reached_owners: BTreeSet<PredeclaredFunctionId>,
    changed: bool,
}

fn runtime_value_owner(value: &crate::RuntimeValue) -> ReferentOwnerFact {
    match value {
        crate::RuntimeValue::Bool(_) | crate::RuntimeValue::Int(_) => {
            ReferentOwnerFact::owner(BoundaryReferentOwner::NoReferent)
        }
        crate::RuntimeValue::Bytes(_) | crate::RuntimeValue::String(_) => {
            ReferentOwnerFact::owner(BoundaryReferentOwner::PersistentStore)
        }
        crate::RuntimeValue::Constructor { args, .. } => {
            let children = args
                .iter()
                .map(|value| {
                    let mut child = ProducerValue::empty();
                    child.referent_owners = runtime_value_owner(value);
                    child
                })
                .collect::<Vec<_>>();
            ReferentOwnerFact::aggregate(&children)
        }
        crate::RuntimeValue::Record { fields } => {
            let children = fields
                .iter()
                .map(|(_, value)| {
                    let mut child = ProducerValue::empty();
                    child.referent_owners = runtime_value_owner(value);
                    child
                })
                .collect::<Vec<_>>();
            ReferentOwnerFact::aggregate(&children)
        }
        crate::RuntimeValue::ClosureRef { .. } | crate::RuntimeValue::Unknown => {
            ReferentOwnerFact::Unrepresented
        }
    }
}

impl ProducerAnalysis<'_, '_> {
    fn mark_carried(&mut self, value: &mut ProducerValue) {
        value.carried = true;
        for origin in &value.aggregate_origins {
            self.changed |= self.carried_aggregates.insert(*origin);
        }
        for origin in &value.effect_origins {
            self.changed |= self.carried_effects.insert(*origin);
        }
    }

    fn merge_value(slot: &mut ProducerValue, incoming: &ProducerValue) -> bool {
        let joined = slot.join(incoming);
        if *slot == joined {
            false
        } else {
            *slot = joined;
            true
        }
    }

    fn merge_values(
        slots: &mut Vec<ProducerValue>,
        incoming: &[ProducerValue],
    ) -> Result<bool, CraneliftBackendError> {
        if slots.len() != incoming.len() {
            return Err(planner_error(
                "producer-flow environment arity disagrees with the planned ABI",
            ));
        }
        let mut changed = false;
        for (slot, value) in slots.iter_mut().zip(incoming) {
            changed |= Self::merge_value(slot, value);
        }
        Ok(changed)
    }

    fn record_aggregate(
        &mut self,
        origin: StaticOriginId,
        class: BoundaryClass,
        identity: AggregateIdentity,
        child_origins: Vec<StaticOriginId>,
        children: &[ProducerValue],
        carried: bool,
    ) -> Result<(), CraneliftBackendError> {
        if child_origins.len() != children.len() {
            return Err(planner_error(
                "aggregate occurrence child authority is not positional",
            ));
        }
        let observation = AggregateObservation {
            class,
            identity,
            children: child_origins
                .into_iter()
                .zip(children)
                .map(|(origin, value)| (origin, value.referent_owners.clone()))
                .collect(),
            carried: carried || self.carried_aggregates.contains(&origin),
        };
        let key = (origin, self.active_owner);
        match self.aggregate_occurrences.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(observation);
                self.changed = true;
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                let joined = entry.get().join(&observation)?;
                if *entry.get() != joined {
                    entry.insert(joined);
                    self.changed = true;
                }
            }
        }
        Ok(())
    }

    fn closure_value(
        &mut self,
        origin: StaticOriginId,
        env: &[ProducerValue],
    ) -> Result<ProducerValue, CraneliftBackendError> {
        let occurrence = self
            .plan
            .source_occurrences
            .get(origin.0 as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| planner_error("producer-flow closure names no occurrence"))?;
        let (capture_values, body_origin) = match occurrence.expr {
            RuntimeExpr::Closure { captures, .. } => (
                captures
                    .iter()
                    .map(|_| ProducerValue::open(origin))
                    .collect::<Vec<_>>(),
                self.plan.semantic.child_origin(origin, 0)?,
            ),
            RuntimeExpr::LexicalClosure { captures, .. } => {
                let mut values = Vec::with_capacity(captures.len());
                for index in 0..captures.len() {
                    let capture_origin = self.plan.semantic.child_origin(origin, 1 + index)?;
                    let mut value = self.eval(capture_origin, env)?;
                    value.constructors = value
                        .constructors
                        .forwarded(origin, ProducerFlowKind::Capture);
                    values.push(value);
                }
                (values, self.plan.semantic.child_origin(origin, 0)?)
            }
            _ => {
                return Err(planner_error(
                    "producer-flow callable identity is not a closure",
                ));
            }
        };
        let referent_owners = ReferentOwnerFact::aggregate(&capture_values);
        let aggregate_origins = capture_values
            .iter()
            .flat_map(|capture| capture.aggregate_origins.iter().copied())
            .collect();
        let effect_origins = capture_values
            .iter()
            .flat_map(|capture| capture.effect_origins.iter().copied())
            .collect();
        match self.captures.entry(origin) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(capture_values);
                self.changed = true;
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                self.changed |= Self::merge_values(entry.get_mut(), &capture_values)?;
            }
        }
        Ok(ProducerValue {
            constructors: ProducerFact::empty(),
            constructor_payloads: Vec::new(),
            record_fields: Vec::new(),
            callables: ProducerCallableSet::Closed(BTreeSet::from([ProducerCallable {
                closure_origin: origin,
                body_origin,
                recursor_origin: None,
            }])),
            referent_owners,
            aggregate_origins,
            effect_origins,
            carried: false,
        })
    }

    fn bound_callable_value(
        &mut self,
        binding: &StaticCallableBindingKey,
        lifted: &[ProducerValue],
        cursor: &mut usize,
    ) -> Result<ProducerValue, CraneliftBackendError> {
        let mut captures = Vec::with_capacity(binding.captures.len());
        for capture in &binding.captures {
            captures.push(match capture {
                StaticCallableCaptureBinding::Value(_) => {
                    let value = lifted.get(*cursor).cloned().ok_or_else(|| {
                        planner_error("producer-flow callable binding omits a lifted capture")
                    })?;
                    *cursor = cursor.checked_add(1).ok_or_else(|| {
                        planner_capacity_error("producer-flow lifted capture cursor exhausted")
                    })?;
                    value
                }
                StaticCallableCaptureBinding::Callable(nested) => {
                    self.bound_callable_value(nested, lifted, cursor)?
                }
            });
        }
        let referent_owners = ReferentOwnerFact::aggregate(&captures);
        let aggregate_origins = captures
            .iter()
            .flat_map(|capture| capture.aggregate_origins.iter().copied())
            .collect();
        let effect_origins = captures
            .iter()
            .flat_map(|capture| capture.effect_origins.iter().copied())
            .collect();
        match self.captures.entry(binding.closure_origin) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(captures);
                self.changed = true;
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                self.changed |= Self::merge_values(entry.get_mut(), &captures)?;
            }
        }
        Ok(ProducerValue {
            constructors: ProducerFact::empty(),
            constructor_payloads: Vec::new(),
            record_fields: Vec::new(),
            callables: ProducerCallableSet::Closed(BTreeSet::from([ProducerCallable {
                closure_origin: binding.closure_origin,
                body_origin: binding.body_origin,
                recursor_origin: None,
            }])),
            referent_owners,
            aggregate_origins,
            effect_origins,
            carried: false,
        })
    }

    fn source_environment(
        &mut self,
        owner: PredeclaredFunctionId,
        definition: AbiUnitDefinition,
    ) -> Result<Vec<ProducerValue>, CraneliftBackendError> {
        let raw = self
            .inputs
            .get(&owner)
            .cloned()
            .ok_or_else(|| planner_error("producer-flow owner has no ABI environment"))?;
        let AbiUnitDefinition::StaticCallableSpecialization { specialization, .. } = definition
        else {
            return Ok(raw);
        };
        let specialization = self
            .plan
            .static_callable_specializations
            .get(specialization.0 as usize)
            .ok_or_else(|| planner_error("producer-flow specialization is outside the plan"))?;
        match &specialization.kind {
            PlannedStaticCallableSpecializationKind::Declaration => {
                let occurrence = self
                    .plan
                    .source_occurrences
                    .get(specialization.base_origin.0 as usize)
                    .and_then(Option::as_ref)
                    .ok_or_else(|| {
                        planner_error("producer-flow specialization base has no occurrence")
                    })?;
                let (parameter_count, declaration_capture_count) = match occurrence.expr {
                    RuntimeExpr::Closure {
                        params, captures, ..
                    } => (params.len(), captures.len()),
                    RuntimeExpr::LexicalClosure {
                        params, captures, ..
                    } => (params.len(), captures.len()),
                    _ => {
                        return Err(planner_error(
                            "producer-flow specialization base is not a closure",
                        ));
                    }
                };
                let ordinary_count = specialization.ordinary_parameters as usize;
                let declaration_start = raw
                    .len()
                    .checked_sub(declaration_capture_count)
                    .ok_or_else(|| {
                        planner_error("producer-flow specialization omits declaration captures")
                    })?;
                if ordinary_count > declaration_start {
                    return Err(planner_error(
                        "producer-flow specialization ordinary inputs exceed its ABI",
                    ));
                }
                let lifted = &raw[ordinary_count..declaration_start];
                let mut lifted_cursor = 0usize;
                let mut ordinary_cursor = 0usize;
                let mut environment =
                    Vec::with_capacity(parameter_count + declaration_capture_count);
                for parameter in 0..parameter_count {
                    if let Some(binding) = specialization
                        .key
                        .bindings
                        .iter()
                        .find(|binding| binding.parameter_ordinal as usize == parameter)
                    {
                        environment.push(self.bound_callable_value(
                            binding,
                            lifted,
                            &mut lifted_cursor,
                        )?);
                    } else {
                        environment.push(raw.get(ordinary_cursor).cloned().ok_or_else(|| {
                            planner_error("producer-flow specialization omits an ordinary input")
                        })?);
                        ordinary_cursor += 1;
                    }
                }
                if ordinary_cursor != ordinary_count || lifted_cursor != lifted.len() {
                    return Err(planner_error(
                        "producer-flow specialization input partition is not exact",
                    ));
                }
                environment.extend_from_slice(&raw[declaration_start..]);
                Ok(environment)
            }
            PlannedStaticCallableSpecializationKind::CallableBody { binding } => {
                let parameter_count = binding.declared_arity as usize;
                if parameter_count > raw.len() {
                    return Err(planner_error(
                        "producer-flow callable body omits declared parameters",
                    ));
                }
                let mut cursor = 0usize;
                let callable =
                    self.bound_callable_value(binding, &raw[parameter_count..], &mut cursor)?;
                if cursor != raw.len() - parameter_count {
                    return Err(planner_error(
                        "producer-flow callable body left a lifted capture unused",
                    ));
                }
                let mut environment = raw[..parameter_count].to_vec();
                environment.extend(
                    self.captures
                        .get(&binding.closure_origin)
                        .cloned()
                        .ok_or_else(|| {
                            planner_error("producer-flow callable body has no captures")
                        })?,
                );
                let _ = callable;
                Ok(environment)
            }
        }
    }

    fn invoke(
        &mut self,
        call_origin: StaticOriginId,
        callables: ProducerCallableSet,
        arguments: Vec<ProducerValue>,
    ) -> Result<ProducerValue, CraneliftBackendError> {
        let ProducerCallableSet::Closed(callables) = callables else {
            return Ok(ProducerValue::open(call_origin));
        };
        if callables.is_empty() {
            return Ok(ProducerValue::open(call_origin));
        }
        let mut result: Option<ProducerValue> = None;
        for callable in callables {
            let owner = self
                .plan
                .semantic
                .function_owner(callable.body_origin)?
                .ok_or_else(|| planner_error("producer-flow callable body has no owner"))?;
            let captures = self
                .captures
                .get(&callable.closure_origin)
                .cloned()
                .ok_or_else(|| planner_error("producer-flow callable has no capture authority"))?;
            // Source calls list arguments left-to-right, while a declaration
            // body observes its parameters de-Bruijn-nearest first. Preserve
            // the same reversal the emitted declaration ABI performs; failing
            // to do so attributes the first source argument to `Var(0)` and
            // turns a closed constructor producer into an unrelated opaque
            // ingress.
            let mut incoming = arguments.iter().rev().cloned().collect::<Vec<_>>();
            incoming.extend(captures);
            if let Some(recursor_origin) = callable.recursor_origin {
                let mut residuals =
                    self.static_recursor_worker_residuals
                        .iter()
                        .filter(|residual| {
                            residual.parent_origin == recursor_origin
                                && residual.closure_origin == callable.closure_origin
                                && residual.body_origin == callable.body_origin
                        });
                let residual = residuals.next().ok_or_else(|| {
                    planner_error("static recursor call has no exact worker residual")
                })?;
                if residuals.next().is_some() {
                    return Err(planner_error(
                        "static recursor call has ambiguous worker residual authority",
                    ));
                }
                let expected = usize::try_from(residual.declared_arity)
                    .ok()
                    .and_then(|arity| arity.checked_add(residual.captures.len()))
                    .ok_or_else(|| {
                        planner_capacity_error("static recursor worker input count exhausted")
                    })?;
                if incoming.len() != expected {
                    return Err(planner_error(
                        "static recursor call disagrees with its worker residual ABI",
                    ));
                }
                for value in &mut incoming {
                    value.referent_owners =
                        ReferentOwnerFact::owner(BoundaryReferentOwner::InvocationArena);
                }
            }
            // A source call crosses one generated-unit result edge even when a
            // recursive call returns to the currently analysed owner. Never
            // recurse through the Rust evaluator for that case: merge its
            // inputs and read the previous monotone result approximation. The
            // outer fixed point re-evaluates the owner after either changes.
            // Treating the active owner specially makes source recursion a
            // host-stack recursion and bypasses the very cycle closure this
            // analysis is required to prove.
            for value in &mut incoming {
                self.mark_carried(value);
            }
            let slots = self
                .inputs
                .get_mut(&owner)
                .ok_or_else(|| planner_error("producer-flow callable owner has no ABI inputs"))?;
            self.changed |= Self::merge_values(slots, &incoming)?;
            self.changed |= self.reached_owners.insert(owner);
            let mut value = self
                .results
                .get(&owner)
                .cloned()
                .unwrap_or_else(ProducerValue::empty);
            if let Some(recursor_origin) = callable.recursor_origin {
                self.mark_carried(&mut value);
                value.constructors = value
                    .constructors
                    .forwarded(recursor_origin, ProducerFlowKind::Recursor);
                let key = (recursor_origin, self.active_owner);
                match self.computational_scrutinees.entry(key) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(value);
                        self.changed = true;
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        self.changed |= Self::merge_value(entry.get_mut(), &value);
                    }
                }
                value = self
                    .computational_results
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(ProducerValue::empty);
                self.mark_carried(&mut value);
            }
            result = Some(match result {
                Some(previous) => previous.join(&value),
                None => value,
            });
        }
        let mut result = result.unwrap_or_else(ProducerValue::empty);
        self.mark_carried(&mut result);
        result.constructors = result
            .constructors
            .forwarded(call_origin, ProducerFlowKind::CallResult);
        Ok(result)
    }

    fn eval(
        &mut self,
        origin: StaticOriginId,
        env: &[ProducerValue],
    ) -> Result<ProducerValue, CraneliftBackendError> {
        let occurrence = self
            .plan
            .source_occurrences
            .get(origin.0 as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| planner_error("producer-flow names no source occurrence"))?;
        let child = |position| self.plan.semantic.child_origin(origin, position);
        let value = match occurrence.expr {
            RuntimeExpr::CheckedJoinSite { .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { .. }
            | RuntimeExpr::CheckedRecursiveInvocation { .. }
            | RuntimeExpr::CheckedComputationalIHSlots { .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { .. } => self
                .eval(child(0)?, env)?
                .with_forward(origin, ProducerFlowKind::Forward),
            RuntimeExpr::Value(value @ crate::RuntimeValue::Constructor { .. }) => {
                // Runtime values have their own semantic atom kind. Until that
                // canonical identity is exposed, this is an explicit opaque
                // producer rather than a fabricated symbol lookup.
                let mut result = ProducerValue::open(origin);
                result.referent_owners = runtime_value_owner(value);
                result
            }
            RuntimeExpr::Value(value) => {
                let mut result = ProducerValue::empty();
                result.referent_owners = runtime_value_owner(value);
                result
            }
            RuntimeExpr::Trap(_) => ProducerValue::empty(),
            RuntimeExpr::Var(index) => env
                .get(*index as usize)
                .cloned()
                .unwrap_or_else(|| ProducerValue::open(origin))
                .with_forward(origin, ProducerFlowKind::Environment),
            RuntimeExpr::Let { .. } => {
                let bound = self.eval(child(0)?, env)?;
                let mut body_env = Vec::with_capacity(env.len() + 1);
                body_env.push(bound);
                body_env.extend_from_slice(env);
                self.eval(child(1)?, &body_env)?
                    .with_forward(origin, ProducerFlowKind::Forward)
            }
            RuntimeExpr::If { .. } => {
                let left = self.eval(child(1)?, env)?;
                let right = self.eval(child(2)?, env)?;
                left.join(&right)
                    .with_forward(origin, ProducerFlowKind::Alternative)
            }
            RuntimeExpr::Construct { args, .. } => {
                let mut payload = Vec::with_capacity(args.len());
                let mut child_origins = Vec::with_capacity(args.len());
                for index in 0..args.len() {
                    let child_origin = child(index)?;
                    child_origins.push(child_origin);
                    payload.push(self.eval(child_origin, env)?);
                }
                let identity = self.plan.semantic.constructor_symbol_identity(origin)?;
                let carried = payload.iter().any(|child| child.carried);
                self.record_aggregate(
                    origin,
                    BoundaryClass::Constructor,
                    AggregateIdentity::Constructor(identity),
                    child_origins,
                    &payload,
                    carried,
                )?;
                let referent_owners = ReferentOwnerFact::aggregate(&payload);
                let mut aggregate_origins = payload
                    .iter()
                    .flat_map(|child| child.aggregate_origins.iter().copied())
                    .collect::<BTreeSet<_>>();
                aggregate_origins.insert(origin);
                let effect_origins = payload
                    .iter()
                    .flat_map(|child| child.effect_origins.iter().copied())
                    .collect();
                ProducerValue {
                    constructors: ProducerFact::constructor(origin, identity),
                    constructor_payloads: vec![(identity, payload)],
                    record_fields: Vec::new(),
                    callables: ProducerCallableSet::Closed(BTreeSet::new()),
                    referent_owners,
                    aggregate_origins,
                    effect_origins,
                    carried,
                }
            }
            RuntimeExpr::Match { cases, .. } => {
                let scrutinee_origin = child(0)?;
                let scrutinee = self.eval(scrutinee_origin, env)?;
                if scrutinee.carried {
                    self.match_scrutinees
                        .entry((origin, self.active_owner))
                        .and_modify(|known| *known = known.join(&scrutinee.constructors))
                        .or_insert_with(|| scrutinee.constructors.clone());
                }
                let mut result = ProducerValue::empty();
                for (index, case) in cases.iter().enumerate() {
                    let identity = self
                        .plan
                        .semantic
                        .case_constructor_identity(origin, index)?;
                    let reachable = match &scrutinee.constructors.population {
                        ScrutineeProducerSet::Open => true,
                        ScrutineeProducerSet::Closed(constructors) => {
                            constructors.contains(&identity)
                        }
                    };
                    if !reachable {
                        continue;
                    }
                    let mut case_env = match &scrutinee.constructors.population {
                        ScrutineeProducerSet::Open => (0..case.binders)
                            .map(|_| {
                                let mut value = ProducerValue::open(origin);
                                value.referent_owners = scrutinee.referent_owners.clone();
                                value
                            })
                            .collect(),
                        ScrutineeProducerSet::Closed(_) => scrutinee
                            .constructor_payloads
                            .iter()
                            .find(|(constructor, _)| *constructor == identity)
                            .filter(|(_, payload)| payload.len() == case.binders)
                            .map(|(_, payload)| payload.clone())
                            .unwrap_or_else(|| {
                                (0..case.binders)
                                    .map(|_| {
                                        let mut value = ProducerValue::open(origin);
                                        value.referent_owners = scrutinee.referent_owners.clone();
                                        value
                                    })
                                    .collect()
                            }),
                    };
                    for binder in &mut case_env {
                        if scrutinee.carried {
                            self.mark_carried(binder);
                        }
                        binder.constructors = binder
                            .constructors
                            .clone()
                            .forwarded(origin, ProducerFlowKind::Forward);
                    }
                    case_env.extend_from_slice(env);
                    result = result.join(&self.eval(child(1 + index)?, &case_env)?);
                }
                result.with_forward(origin, ProducerFlowKind::Alternative)
            }
            RuntimeExpr::ComputationalMatch { cases, .. } => {
                let incoming_scrutinee = self.eval(child(0)?, env)?;
                let key = (origin, self.active_owner);
                match self.computational_scrutinees.entry(key) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(incoming_scrutinee);
                        self.changed = true;
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        self.changed |= Self::merge_value(entry.get_mut(), &incoming_scrutinee);
                    }
                }
                let scrutinee = self
                    .computational_scrutinees
                    .get(&key)
                    .cloned()
                    .ok_or_else(|| {
                        planner_error("producer-flow computational scrutinee is absent")
                    })?;
                let mut result = ProducerValue::empty();
                for (index, case) in cases.iter().enumerate() {
                    let identity = self
                        .plan
                        .semantic
                        .case_constructor_identity(origin, index)?;
                    let reachable = match &scrutinee.constructors.population {
                        ScrutineeProducerSet::Open => true,
                        ScrutineeProducerSet::Closed(constructors) => {
                            constructors.contains(&identity)
                        }
                    };
                    if !reachable {
                        continue;
                    }
                    let mut arguments = match &scrutinee.constructors.population {
                        ScrutineeProducerSet::Open => (0..case.argument_binders)
                            .map(|_| {
                                let mut value = ProducerValue::open(origin);
                                value.referent_owners = scrutinee.referent_owners.clone();
                                value
                            })
                            .collect(),
                        ScrutineeProducerSet::Closed(_) => scrutinee
                            .constructor_payloads
                            .iter()
                            .find(|(constructor, _)| *constructor == identity)
                            .filter(|(_, payload)| payload.len() == case.argument_binders)
                            .map(|(_, payload)| payload.clone())
                            .unwrap_or_else(|| {
                                (0..case.argument_binders)
                                    .map(|_| {
                                        let mut value = ProducerValue::open(origin);
                                        value.referent_owners = scrutinee.referent_owners.clone();
                                        value
                                    })
                                    .collect()
                            }),
                    };
                    for argument in &mut arguments {
                        if scrutinee.carried {
                            self.mark_carried(argument);
                        }
                        argument.constructors = argument
                            .constructors
                            .clone()
                            .forwarded(origin, ProducerFlowKind::Forward);
                    }
                    let producer_origins = scrutinee
                        .constructors
                        .producer_origins
                        .iter()
                        .find(|(constructor, _)| *constructor == identity)
                        .map(|(_, origins)| origins)
                        .cloned()
                        .unwrap_or_default();
                    let mut induction_hypotheses =
                        Vec::with_capacity(case.recursive_positions.len());
                    for position in case.recursive_positions.iter().rev().copied() {
                        let callables = self
                            .plan
                            .static_recursor_worker_residuals
                            .iter()
                            .filter(|residual| {
                                residual.parent_origin == origin
                                    && residual.sibling_position
                                        == u32::try_from(position).unwrap_or(u32::MAX)
                                    && producer_origins.contains(&residual.producer_origin)
                            })
                            .map(|residual| ProducerCallable {
                                closure_origin: residual.closure_origin,
                                body_origin: residual.body_origin,
                                recursor_origin: Some(origin),
                            })
                            .collect::<BTreeSet<_>>();
                        induction_hypotheses.push(if callables.is_empty() {
                            ProducerValue::open(origin)
                        } else {
                            ProducerValue {
                                constructors: ProducerFact::empty(),
                                constructor_payloads: Vec::new(),
                                record_fields: Vec::new(),
                                callables: ProducerCallableSet::Closed(callables),
                                referent_owners: ReferentOwnerFact::Unrepresented,
                                aggregate_origins: BTreeSet::new(),
                                effect_origins: BTreeSet::new(),
                                carried: false,
                            }
                        });
                    }
                    let mut case_env = induction_hypotheses;
                    case_env.extend(arguments);
                    case_env.extend_from_slice(env);
                    result = result.join(&self.eval(child(1 + index)?, &case_env)?);
                }
                result = result.with_forward(origin, ProducerFlowKind::Alternative);
                match self.computational_results.entry(key) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(result.clone());
                        self.changed = true;
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        self.changed |= Self::merge_value(entry.get_mut(), &result);
                        result = entry.get().clone();
                    }
                }
                result
            }
            RuntimeExpr::Closure { .. } | RuntimeExpr::LexicalClosure { .. } => {
                self.closure_value(origin, env)?
            }
            RuntimeExpr::DeclarationRef { symbol } => {
                if let Some(declaration) = self.plan.declaration_occurrences.get(symbol) {
                    self.closure_value(*declaration, &[])?
                } else {
                    ProducerValue::open(origin)
                }
            }
            RuntimeExpr::Call { args, .. } => {
                let callee = self.eval(child(0)?, env)?;
                let mut arguments = Vec::with_capacity(args.len());
                for index in 0..args.len() {
                    let mut argument = self.eval(child(1 + index)?, env)?;
                    argument.constructors = argument
                        .constructors
                        .forwarded(origin, ProducerFlowKind::CallArgument);
                    arguments.push(argument);
                }
                self.invoke(origin, callee.callables, arguments)?
            }
            RuntimeExpr::PrimitiveCall { args, .. } => {
                for index in 0..args.len() {
                    let _ = self.eval(child(index)?, env)?;
                }
                // Every admitted primitive result is immediate or persistable
                // ground data. Producer identity can remain open while its
                // referent lifetime is closed and durable.
                let mut result = ProducerValue::open(origin);
                result.referent_owners =
                    ReferentOwnerFact::owner(BoundaryReferentOwner::PersistentStore);
                result
            }
            RuntimeExpr::Record { fields } => {
                let mut payload = Vec::with_capacity(fields.len());
                let mut child_origins = Vec::with_capacity(fields.len());
                let mut record_fields: Vec<(FieldIdentity, ProducerValue)> = Vec::new();
                let mut identities = Vec::with_capacity(fields.len());
                for index in 0..fields.len() {
                    let child_origin = child(index)?;
                    let value = self.eval(child_origin, env)?;
                    let identity = self.plan.semantic.record_field_identity(origin, index)?;
                    child_origins.push(child_origin);
                    identities.push(identity);
                    if let Some((_, known)) = record_fields
                        .iter_mut()
                        .find(|(candidate, _)| *candidate == identity)
                    {
                        *known = known.join(&value);
                    } else {
                        record_fields.push((identity, value.clone()));
                    }
                    payload.push(value);
                }
                let carried = payload.iter().any(|child| child.carried);
                self.record_aggregate(
                    origin,
                    BoundaryClass::Record,
                    AggregateIdentity::Record(identities),
                    child_origins,
                    &payload,
                    carried,
                )?;
                let mut aggregate_origins = payload
                    .iter()
                    .flat_map(|child| child.aggregate_origins.iter().copied())
                    .collect::<BTreeSet<_>>();
                aggregate_origins.insert(origin);
                let effect_origins = payload
                    .iter()
                    .flat_map(|child| child.effect_origins.iter().copied())
                    .collect();
                ProducerValue {
                    constructors: ProducerFact::empty(),
                    constructor_payloads: Vec::new(),
                    record_fields,
                    callables: ProducerCallableSet::Closed(BTreeSet::new()),
                    referent_owners: ReferentOwnerFact::aggregate(&payload),
                    aggregate_origins,
                    effect_origins,
                    carried,
                }
            }
            RuntimeExpr::Project { .. } => {
                let record = self.eval(child(0)?, env)?;
                let identity = self.plan.semantic.project_field_identity(origin)?;
                record
                    .record_fields
                    .iter()
                    .find(|(candidate, _)| *candidate == identity)
                    .map(|(_, value)| value.clone())
                    .unwrap_or_else(|| ProducerValue::open(origin))
            }
            RuntimeExpr::Effect {
                capability, args, ..
            } => {
                let child_count = args.len() + usize::from(capability.is_some());
                let mut operands = Vec::with_capacity(child_count);
                for index in 0..child_count {
                    operands.push(self.eval(child(index)?, env)?.referent_owners);
                }
                let key = (origin, self.active_owner);
                match self.effect_operands.entry(key) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(operands);
                        self.changed = true;
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        if entry.get().len() != operands.len() {
                            return Err(planner_error(
                                "effect operand-owner population changed arity",
                            ));
                        }
                        let joined = entry
                            .get()
                            .iter()
                            .zip(&operands)
                            .map(|(left, right)| left.join(right))
                            .collect::<Vec<_>>();
                        if entry.get() != &joined {
                            entry.insert(joined);
                            self.changed = true;
                        }
                    }
                }
                let mut result = ProducerValue::open(origin);
                result.referent_owners =
                    ReferentOwnerFact::owner(BoundaryReferentOwner::InvocationArena);
                result.effect_origins.insert(origin);
                // Invocation ownership answers how long the result's referent
                // lives; it does not itself prove that this source occurrence
                // crosses a generated-unit boundary.  An actual call,
                // capture, result, or recursor flow marks the value carried.
                // Conflating the two promotes source-machine control
                // constructors around an effect into carrier allocations even
                // when their exact SemanticEliminator consumes them locally.
                result.carried = false;
                result
            }
            RuntimeExpr::ImportedDeclarationRef { .. } => ProducerValue::open(origin),
        };
        Ok(value)
    }
}

trait ProducerValueForward {
    fn with_forward(self, origin: StaticOriginId, kind: ProducerFlowKind) -> Self;
}

impl ProducerValueForward for ProducerValue {
    fn with_forward(mut self, origin: StaticOriginId, kind: ProducerFlowKind) -> Self {
        self.constructors = self.constructors.forwarded(origin, kind);
        self
    }
}

#[derive(Clone)]
struct SynthesizedAggregateSpec {
    site: SynthesizedAggregateSite,
    role: SynthesizedConstructorRole,
    children: Vec<Vec<BoundaryReferentOwner>>,
}

fn closed_owner_set(
    fact: &ReferentOwnerFact,
    context: &'static str,
) -> Result<Vec<BoundaryReferentOwner>, CraneliftBackendError> {
    fact.closed_owners()
        .ok_or_else(|| planner_error(format!("{context} has no closed referent-owner authority")))
}

fn synthesized_effect_aggregate_specs(
    operation: ken_host::HostOpV1,
    has_capability: bool,
    operands: &[ReferentOwnerFact],
    io_error_roles: &[SynthesizedIoErrorRole],
) -> Result<Vec<SynthesizedAggregateSpec>, CraneliftBackendError> {
    use ken_host::HostOpV1 as Op;
    use BoundaryReferentOwner as Owner;
    use SynthesizedAggregateSite as Site;
    use SynthesizedConstructorRole as Role;
    use SynthesizedFixedConstructorRole as Fixed;

    let none = || vec![Owner::NoReferent];
    let persistent = || vec![Owner::PersistentStore];
    let invocation = || vec![Owner::InvocationArena];
    let mut specs = io_error_roles
        .iter()
        .copied()
        .enumerate()
        .map(|(index, role)| {
            Ok(SynthesizedAggregateSpec {
                site: Site::IoError(u32::try_from(index).map_err(|_| {
                    planner_capacity_error("synthesized IOError occurrence ordinal exhausted")
                })?),
                role: Role::IoError(role),
                children: if index + 1 == io_error_roles.len() {
                    vec![none()]
                } else {
                    Vec::new()
                },
            })
        })
        .collect::<Result<Vec<_>, CraneliftBackendError>>()?;

    let fixed = |site, role, children| SynthesizedAggregateSpec {
        site,
        role: Role::Fixed(role),
        children,
    };
    if matches!(
        operation,
        Op::FsReadFile | Op::FsWriteFile | Op::FsChangeMode | Op::FsOpen
    ) {
        let path_position = usize::from(has_capability);
        let path = operands
            .get(path_position)
            .ok_or_else(|| planner_error("file effect has no exact path owner fact"))?;
        let path = closed_owner_set(path, "file path operand")?;
        let wrapper_owner = if path.contains(&Owner::InvocationArena) {
            invocation()
        } else {
            persistent()
        };
        let operation_role = match operation {
            Op::FsReadFile | Op::FsOpen => Fixed::FileOperationRead,
            Op::FsWriteFile => Fixed::FileOperationWrite,
            Op::FsChangeMode => Fixed::FileOperationChangeMode,
            _ => unreachable!("guarded file operation"),
        };
        specs.extend([
            fixed(Site::FileOperation, operation_role, Vec::new()),
            fixed(Site::FilePathSome, Fixed::OptionSome, vec![path]),
            fixed(
                Site::FileError,
                Fixed::FileError,
                vec![persistent(), wrapper_owner, persistent()],
            ),
        ]);
    } else if matches!(
        operation,
        Op::FsHandleMetadata
            | Op::ResourceRelease
            | Op::BufferAllocate
            | Op::BufferFreeze
            | Op::FsReadAt
            | Op::FsWriteAt
    ) {
        for occurrence in 0..3 {
            specs.extend([
                fixed(
                    Site::ResourceKind(occurrence, 0),
                    Fixed::ResourceKindFsHandle,
                    Vec::new(),
                ),
                fixed(
                    Site::ResourceKind(occurrence, 1),
                    Fixed::ResourceKindBuffer,
                    Vec::new(),
                ),
            ]);
        }
        specs.extend([
            fixed(
                Site::ResourceTraceIdentity,
                Fixed::ResourceTraceIdentity,
                vec![none(), none()],
            ),
            fixed(
                Site::ResourceHostIo,
                Fixed::ResourceHostIo,
                vec![persistent()],
            ),
            fixed(Site::ResourceClosed, Fixed::ResourceClosed, Vec::new()),
            fixed(
                Site::ResourceMalformed,
                Fixed::ResourceMalformed,
                Vec::new(),
            ),
            fixed(
                Site::ResourceRightNotHeld,
                Fixed::ResourceRightNotHeld,
                vec![none(), none()],
            ),
            fixed(
                Site::ResourceReleaseFailed,
                Fixed::ResourceReleaseFailed,
                vec![persistent(), persistent(), persistent()],
            ),
            fixed(
                Site::ResourceKindMismatch,
                Fixed::ResourceKindMismatch,
                vec![persistent(), persistent()],
            ),
            fixed(
                Site::ResourceBufferLimit,
                Fixed::ResourceBufferLimit,
                Vec::new(),
            ),
            fixed(
                Site::ResourceInvalidOffset,
                Fixed::ResourceInvalidOffset,
                Vec::new(),
            ),
            fixed(
                Site::ResourceInvalidBounds,
                Fixed::ResourceInvalidBounds,
                Vec::new(),
            ),
            fixed(
                Site::ResourceNoProgress,
                Fixed::ResourceNoProgress,
                Vec::new(),
            ),
        ]);
    }

    match operation {
        Op::FsReadAt => specs.extend([
            fixed(
                Site::ReadBufferSpan,
                Fixed::PrivateBufferSpan,
                vec![invocation(), none(), none()],
            ),
            fixed(
                Site::ReadTransferCount,
                Fixed::PrivateTransferCount,
                vec![none(), none()],
            ),
            fixed(Site::ReadEof, Fixed::ReadEof, Vec::new()),
            fixed(
                Site::ReadSome,
                Fixed::ReadSome,
                vec![invocation(), persistent()],
            ),
        ]),
        Op::FsWriteAt => specs.extend([
            fixed(
                Site::WriteTransferCount,
                Fixed::PrivateTransferCount,
                vec![none(), none()],
            ),
            fixed(Site::Wrote, Fixed::Wrote, vec![persistent()]),
        ]),
        Op::FsReadFile
        | Op::FsOpen
        | Op::BufferAllocate
        | Op::BufferFreeze
        | Op::FsHandleMetadata => {}
        _ => specs.push(fixed(Site::Unit, Fixed::Unit, Vec::new())),
    }
    Ok(specs)
}

fn build_producer_flow_plans(
    plan: &StaticTransitionPlan<'_>,
) -> Result<
    (
        Vec<PlannedCaseEmission>,
        Vec<PlannedAggregateRepresentation>,
        Vec<PlannedSynthesizedAggregateRepresentation>,
    ),
    CraneliftBackendError,
> {
    let mut computational_scrutinee_results = BTreeSet::new();
    for occurrence in plan.source_occurrences.iter().flatten() {
        if !matches!(occurrence.expr, RuntimeExpr::ComputationalMatch { .. }) {
            continue;
        }
        let scrutinee = plan.semantic.child_origin(occurrence.static_origin, 0)?;
        computational_scrutinee_results
            .extend(plan.source_result_origins_in_owner_subtree(scrutinee)?);
    }
    let mut pending = computational_scrutinee_results
        .iter()
        .copied()
        .collect::<Vec<_>>();
    while let Some(parent) = pending.pop() {
        for child in plan
            .operand_edges
            .iter()
            .filter(|edge| {
                edge.parent == parent
                    && edge.disposition == OperandEdgeDisposition::SemanticEliminator
            })
            .map(|edge| edge.child)
        {
            if computational_scrutinee_results.insert(child) {
                pending.push(child);
            }
        }
    }
    let emittable_owners = plan
        .emittable_units()?
        .into_iter()
        .map(|unit| unit.function())
        .collect::<BTreeSet<_>>();
    let first_owner = plan
        .abi
        .descriptors
        .first()
        .map(|descriptor| descriptor.function)
        .ok_or_else(|| planner_error("case-emission analysis has no function population"))?;
    let root_owner = plan
        .root_occurrence
        .ok_or_else(|| planner_error("producer-flow analysis has no root occurrence"))
        .and_then(|origin| {
            plan.semantic
                .function_owner(origin)?
                .ok_or_else(|| planner_error("producer-flow root has no function owner"))
        })?;
    let mut inputs = BTreeMap::new();
    let mut bodies = Vec::new();
    for descriptor in &plan.abi.descriptors {
        let input_count = descriptor
            .header
            .parameters
            .checked_add(descriptor.header.captures)
            .ok_or_else(|| planner_capacity_error("producer-flow ABI input count exhausted"))?;
        let input_count = usize::try_from(input_count)
            .map_err(|_| planner_capacity_error("producer-flow ABI input count exhausted"))?;
        let start = descriptor.slots.start as usize;
        let end = start
            .checked_add(descriptor.slots.len as usize)
            .ok_or_else(|| planner_capacity_error("producer-flow ABI slot range exhausted"))?;
        let input_slots = plan
            .abi
            .slots
            .get(start..end)
            .ok_or_else(|| planner_error("producer-flow ABI slot range is outside the plan"))?
            .iter()
            .filter(|slot| matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture))
            .collect::<Vec<_>>();
        if input_slots.len() != input_count {
            return Err(planner_error(
                "producer-flow ABI input slots disagree with the frame header",
            ));
        }
        let mut environment = input_slots
            .into_iter()
            .map(|slot| {
                let mut value = ProducerValue::empty();
                value.referent_owners = match slot.carrier {
                    AbiCarrier::ValueWord => ReferentOwnerFact::Closed(BTreeSet::from([
                        BoundaryReferentOwner::NoReferent,
                        BoundaryReferentOwner::PersistentStore,
                        BoundaryReferentOwner::InvocationArena,
                    ])),
                    AbiCarrier::GroundValueCarrier => ReferentOwnerFact::Closed(BTreeSet::from([
                        BoundaryReferentOwner::NoReferent,
                        BoundaryReferentOwner::PersistentStore,
                    ])),
                    _ => ReferentOwnerFact::Unrepresented,
                };
                value.carried = true;
                value
            })
            .collect::<Vec<_>>();
        if matches!(
            descriptor.definition,
            AbiUnitDefinition::SchedulingEntry {
                ingress: AbiSchedulingIngress::ProcessPair,
            }
        ) {
            for value in &mut environment {
                *value = ProducerValue::open(descriptor.origin);
                value.referent_owners =
                    ReferentOwnerFact::owner(BoundaryReferentOwner::InvocationArena);
                value.carried = true;
            }
        }
        inputs.insert(descriptor.function, environment);
        let body_origin = match descriptor.definition {
            // This unit only forwards into its separately owned closure body.
            // Producer flow follows the statically known callable directly in
            // `invoke`; evaluating the body here as well fabricates a second,
            // input-less execution and can derive an unlawful `Closed({})`
            // fact from "no producer observed".
            AbiUnitDefinition::TransparentDeclarationClosure { .. } => continue,
            AbiUnitDefinition::StaticCallableSpecialization { specialization, .. } => {
                plan.static_callable_specializations
                    .get(specialization.0 as usize)
                    .ok_or_else(|| {
                        planner_error("producer-flow specialization is outside the plan")
                    })?
                    .base_body_origin
            }
            AbiUnitDefinition::SchedulingEntry { .. } => {
                if plan.root_occurrence.is_some_and(|root| {
                    plan.semantic.function_owner(root).ok() == Some(Some(descriptor.function))
                }) {
                    plan.root_occurrence
                        .ok_or_else(|| planner_error("producer-flow root is absent"))?
                } else {
                    descriptor.origin
                }
            }
            AbiUnitDefinition::ClosureBody { .. } => descriptor.origin,
        };
        bodies.push((descriptor.function, body_origin, descriptor.definition));
    }

    // Re-derive the source graph population independently of the stored plan.
    // Besides being the producer-flow authority, this keeps the residual
    // omission/reclassification mutations from failing at an unrelated
    // aggregate observation before exact population validation can reject
    // them.
    let static_recursor_worker_residuals = build_static_recursor_worker_residuals(plan)?;

    // A static recursor worker is entered only through its exact planned
    // residual. Its ordinary arguments are recursive children extracted from
    // a carried scrutinee, and its captures are appended from the
    // activation-owned residual environment. Seed that one separately emitted
    // body with the exact owner proved by the residual instead of retaining the
    // ABI carrier's conservative represented-owner population. This is
    // deliberately residual-keyed; an actually unrepresented incoming value
    // still poisons the fact through the ordinary monotone join and fails
    // closed.
    for residual in &static_recursor_worker_residuals {
        let owner = plan
            .semantic
            .function_owner(residual.body_origin)?
            .ok_or_else(|| planner_error("static recursor worker body has no owner"))?;
        let descriptor = plan
            .abi
            .descriptors
            .iter()
            .find(|descriptor| descriptor.function == owner)
            .ok_or_else(|| planner_error("static recursor worker body has no ABI descriptor"))?;
        let capture_count = u32::try_from(residual.captures.len())
            .map_err(|_| planner_capacity_error("static recursor capture count exhausted"))?;
        if descriptor.origin != residual.body_origin
            || descriptor.header.parameters != residual.declared_arity
            || descriptor.header.captures != capture_count
        {
            return Err(planner_error(
                "static recursor worker residual disagrees with its body ABI",
            ));
        }
        let environment = inputs
            .get_mut(&owner)
            .ok_or_else(|| planner_error("static recursor worker body has no ABI environment"))?;
        for value in environment {
            *value = ProducerValue::owner(BoundaryReferentOwner::InvocationArena);
        }
    }

    // Every emittable unit needs complete aggregate occurrence authority
    // before any unit can be defined. Call reachability still flows through
    // the producer values themselves; it cannot suppress representation
    // planning for a separately emitted body.
    let reached_owners = emittable_owners.clone();
    let mut analysis = ProducerAnalysis {
        plan,
        static_recursor_worker_residuals,
        active_owner: first_owner,
        inputs,
        results: BTreeMap::new(),
        captures: BTreeMap::new(),
        computational_scrutinees: BTreeMap::new(),
        computational_results: BTreeMap::new(),
        match_scrutinees: BTreeMap::new(),
        aggregate_occurrences: BTreeMap::new(),
        effect_operands: BTreeMap::new(),
        carried_aggregates: BTreeSet::new(),
        carried_effects: BTreeSet::new(),
        reached_owners,
        changed: true,
    };
    let population_bound = plan
        .source_occurrences
        .len()
        .checked_mul(plan.abi.descriptors.len().max(1))
        .and_then(|bound| bound.checked_add(1))
        .ok_or_else(|| planner_capacity_error("producer-flow fixed-point bound exhausted"))?;
    for _ in 0..population_bound {
        analysis.changed = false;
        for (owner, body_origin, definition) in bodies.iter().copied() {
            if !analysis.reached_owners.contains(&owner) {
                continue;
            }
            analysis.active_owner = owner;
            let environment = analysis.source_environment(owner, definition)?;
            let mut value = analysis.eval(body_origin, &environment)?;
            // Every non-root emittable unit returns across its declared ABI,
            // even when no currently reached caller observes it. The root
            // scheduling result has no such unit-return crossing: it enters
            // the aggregate ledger only when the result-phase planner already
            // requires a carrier. Treating every root result as carried would
            // promote a SpecializedOnly source aggregate into a nonexistent
            // emitted representation occurrence.
            let result_phase = plan
                .result_phases
                .get(body_origin.0 as usize)
                .and_then(Option::as_ref)
                .map(|summary| summary.phase);
            let root_result_is_carried = result_phase == Some(ResultPhase::CarrierRequired);
            #[cfg(test)]
            // The synthesized-ledger control asks the planner to insert one
            // exact root-result crossing. That planned crossing is causal
            // carrier authority; it is not a blanket root promotion.
            let root_result_is_carried = root_result_is_carried
                || plan.lowering_boundary_uses.iter().any(|use_| {
                    use_.edge == LoweringOnlyOperandEdge::TestFixtureResult
                        && use_.origin == body_origin
                });
            if owner != root_owner || root_result_is_carried {
                analysis.mark_carried(&mut value);
            }
            let result = analysis
                .results
                .entry(owner)
                .or_insert_with(ProducerValue::empty);
            analysis.changed |= ProducerAnalysis::merge_value(result, &value);
        }
        if !analysis.changed {
            break;
        }
    }
    if analysis.changed {
        return Err(planner_error(
            "constructor producer-flow fixed point did not close monotonically",
        ));
    }
    let mut aggregate_records = Vec::new();
    for ((origin, owner), observation) in &analysis.aggregate_occurrences {
        if !emittable_owners.contains(owner) {
            continue;
        }
        let is_control = computational_scrutinee_results.contains(origin)
            || plan.terminal_exit_aggregate_origins.contains(origin);
        let is_carried = observation.carried || analysis.carried_aggregates.contains(origin);
        let planned_phase = plan
            .result_phases
            .get(origin.0 as usize)
            .and_then(Option::as_ref)
            .map(|summary| summary.phase);
        if !is_control && !is_carried && planned_phase.is_none() {
            // The producer walk also observes syntax-local aggregates that
            // neither flow to a generated-unit result nor serve as semantic
            // control. They have no carrier crossing, so inventing a
            // representation record for them would turn containment into
            // value-flow authority.
            continue;
        }
        let mut children = Vec::with_capacity(observation.children.len());
        let mut invocation_owned = false;
        for (position, (child_origin, owners)) in observation.children.iter().enumerate() {
            let possible_owners = match owners.closed_owners() {
                Some(owners) => owners,
                None if !observation.carried => Vec::new(),
                None => {
                    let parent_expr = plan
                        .source_occurrence(*origin)
                        .map(|expr| format!("{expr:?}"))
                        .unwrap_or_else(|_| "<missing>".to_string());
                    let child_expr = plan
                        .source_occurrence(*child_origin)
                        .map(|expr| format!("{expr:?}"))
                        .unwrap_or_else(|_| "<missing>".to_string());
                    return Err(planner_error(format!(
                        "aggregate occurrence has an unrepresented, forbidden, or unclosed \
                         carried child; owner={owner:?}; origin={origin:?}; position={position}; \
                         parent={parent_expr}; child={child_expr}"
                    )));
                }
            };
            invocation_owned |= possible_owners.contains(&BoundaryReferentOwner::InvocationArena);
            children.push(PlannedAggregateChild {
                origin: *child_origin,
                position: u32::try_from(position)
                    .map_err(|_| planner_capacity_error("aggregate child position exhausted"))?,
                possible_owners,
            });
        }
        let (selected_owner, selected_tag) = if invocation_owned {
            (
                BoundaryReferentOwner::InvocationArena,
                BoundaryTag::InvocationAggregate,
            )
        } else {
            (
                BoundaryReferentOwner::PersistentStore,
                BoundaryTag::PersistentGround,
            )
        };
        let phase = if is_control {
            // A computational scrutinee is consumed by the source-machine
            // continuation. Its result-position constructors are semantic
            // control, not aggregate allocations, even when the enclosing
            // callable result crosses a unit boundary.
            ResultPhase::SpecializedOnly
        } else if is_carried {
            ResultPhase::CarrierRequired
        } else {
            planned_phase.ok_or_else(|| {
                let expression = plan
                    .source_occurrence(*origin)
                    .map(|expr| format!("{expr:?}"))
                    .unwrap_or_else(|_| "<missing>".to_string());
                planner_error(format!(
                    "aggregate occurrence has no exact result-phase authority; \
                         owner={owner:?}; origin={origin:?}; expression={expression}"
                ))
            })?
        };
        aggregate_records.push(PlannedAggregateRepresentation {
            origin: *origin,
            owner: *owner,
            phase,
            class: observation.class,
            identity: observation.identity.clone(),
            arity: u32::try_from(observation.children.len())
                .map_err(|_| planner_capacity_error("aggregate arity exhausted"))?,
            children,
            selected_owner,
            selected_tag,
        });
    }
    aggregate_records.sort_by_key(|record| (record.owner, record.origin));
    let mut synthesized_aggregate_records = Vec::new();
    for ((effect_origin, owner), operands) in &analysis.effect_operands {
        if !emittable_owners.contains(owner) {
            continue;
        }
        let occurrence = plan.source_occurrence(*effect_origin)?;
        let RuntimeExpr::Effect {
            operation,
            capability,
            ..
        } = occurrence
        else {
            return Err(planner_error(
                "synthesized aggregate authority names a non-effect occurrence",
            ));
        };
        let phase = if analysis.carried_effects.contains(effect_origin) {
            ResultPhase::CarrierRequired
        } else {
            ResultPhase::SpecializedOnly
        };
        for spec in synthesized_effect_aggregate_specs(
            *operation,
            capability.is_some(),
            operands,
            plan.semantic.synthesized_io_error_roles(),
        )? {
            let invocation_owned = spec
                .children
                .iter()
                .flatten()
                .any(|owner| *owner == BoundaryReferentOwner::InvocationArena);
            let (selected_owner, selected_tag) = if invocation_owned {
                (
                    BoundaryReferentOwner::InvocationArena,
                    BoundaryTag::InvocationAggregate,
                )
            } else {
                (
                    BoundaryReferentOwner::PersistentStore,
                    BoundaryTag::PersistentGround,
                )
            };
            synthesized_aggregate_records.push(PlannedSynthesizedAggregateRepresentation {
                effect_origin: *effect_origin,
                owner: *owner,
                phase,
                site: spec.site,
                role: spec.role,
                arity: u32::try_from(spec.children.len())
                    .map_err(|_| planner_capacity_error("synthesized aggregate arity exhausted"))?,
                children: spec.children,
                selected_owner,
                selected_tag,
            });
        }
    }
    synthesized_aggregate_records
        .sort_by_key(|record| (record.owner, record.effect_origin, record.site));
    let mut records = Vec::new();
    for ((match_origin, owner), fact) in analysis.match_scrutinees {
        if !emittable_owners.contains(&owner) {
            continue;
        }
        let occurrence = plan
            .source_occurrences
            .get(match_origin.0 as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| planner_error("case-emission match names no occurrence"))?;
        let RuntimeExpr::Match { cases, .. } = occurrence.expr else {
            return Err(planner_error(
                "case-emission producer authority names a non-Match occurrence",
            ));
        };
        let scrutinee_origin = plan.semantic.child_origin(match_origin, 0)?;
        let phase = ResultPhase::CarrierRequired;
        let authority = fact.authority();
        for (ordinal, _case) in cases.iter().enumerate() {
            let constructor = plan
                .semantic
                .case_constructor_identity(match_origin, ordinal)?;
            let status = match &authority.producers {
                ScrutineeProducerSet::Open => CaseEmissionStatus::Reachable,
                ScrutineeProducerSet::Closed(constructors)
                    if constructors.contains(&constructor) =>
                {
                    CaseEmissionStatus::Reachable
                }
                ScrutineeProducerSet::Closed(_) => CaseEmissionStatus::Eliminated,
            };
            records.push(PlannedCaseEmission {
                match_origin,
                scrutinee_origin,
                owner,
                phase,
                ordinal: u32::try_from(ordinal)
                    .map_err(|_| planner_capacity_error("case-emission ordinal exhausted"))?,
                body_origin: plan.semantic.child_origin(match_origin, 1 + ordinal)?,
                constructor,
                authority: authority.clone(),
                status,
            });
        }
    }
    records.sort_by_key(|record| (record.owner, record.match_origin, record.ordinal));
    Ok((records, aggregate_records, synthesized_aggregate_records))
}

impl<'src> Planner<'src> {
    fn new() -> Result<Self, CraneliftBackendError> {
        let empty = PersistentNodeId(0);
        let frame = DynamicActivationFrame {
            syntax: empty,
            environment: empty,
            normal: empty,
            abrupt: empty,
            path: empty,
            cleanup: empty,
            affine: empty,
            source_return: empty,
        };
        let mut planner = Self {
            plan: StaticTransitionPlan {
                entries: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                stores: Vec::new(),
                store_depths: Vec::new(),
                evidence: Vec::new(),
                planned_helpers: Vec::new(),
                semantic_sources: Vec::new(),
                semantic_material: SemanticMaterialArena::default(),
                abi: AbiPlane::default(),
                root_entry: None,
                root_ingress: AbiRootIngress::Value,
                semantic: SemanticPlane::default(),
                root_occurrence: None,
                declaration_occurrences: BTreeMap::new(),
                trap_catalog: Vec::new(),
                source_occurrences: Vec::new(),
                join_results: Vec::new(),
                result_phases: Vec::new(),
                case_emissions: Vec::new(),
                case_emission_consumption: RefCell::new(BTreeMap::new()),
                aggregate_representations: Vec::new(),
                synthesized_aggregate_representations: Vec::new(),
                terminal_exit_aggregate_origins: BTreeSet::new(),
                aggregate_representation_consumption: RefCell::new(BTreeMap::new()),
                aggregate_representation_dispositions: RefCell::new(BTreeSet::new()),
                synthesized_aggregate_representation_consumption: RefCell::new(BTreeMap::new()),
                functionized_units: false,
                operand_edges: Vec::new(),
                static_callable_specializations: Vec::new(),
                static_callable_calls: Vec::new(),
                static_recursor_worker_residuals: Vec::new(),
                recursor_boundary_uses: Vec::new(),
                lowering_boundary_uses: Vec::new(),
                boundary_uses: Vec::new(),
                operand_edge_consumption: RefCell::new(BTreeMap::new()),
                boundary_use_dispositions: RefCell::new(BTreeSet::new()),
            },
            store_interner: BTreeMap::new(),
            next_source: 0,
            terminal: StaticNodeId(0),
            trap_terminal: StaticNodeId(0),
        };
        let terminal_owner = planner.source()?;
        planner.terminal = planner.control_node(TransitionKind::Terminal, terminal_owner, frame)?;
        let trap_owner = planner.source()?;
        planner.trap_terminal =
            planner.control_node(TransitionKind::TrapTerminal, trap_owner, frame)?;
        Ok(planner)
    }

    fn source(&mut self) -> Result<StaticSourceId, CraneliftBackendError> {
        let id = self.next_source;
        self.next_source = self
            .next_source
            .checked_add(1)
            .ok_or_else(|| planner_capacity_error("static source identity exhausted"))?;
        Ok(StaticSourceId(id))
    }

    fn push_node(
        &mut self,
        kind: TransitionKind,
        owner: StaticSourceId,
        frame: DynamicActivationFrame,
    ) -> Result<StaticNodeId, CraneliftBackendError> {
        let id = u32::try_from(self.plan.nodes.len())
            .map_err(|_| planner_capacity_error("static node identity exhausted"))?;
        let id = StaticNodeId(id);
        self.plan.nodes.push(StaticNode {
            id,
            transition: kind,
            owner,
            frame,
        });
        self.plan
            .planned_helpers
            .push(PlannedHelperKey::node(kind, id));
        Ok(id)
    }

    fn control_node(
        &mut self,
        kind: TransitionKind,
        owner: StaticSourceId,
        frame: DynamicActivationFrame,
    ) -> Result<StaticNodeId, CraneliftBackendError> {
        let node = self.push_node(kind, owner, frame)?;
        self.plan
            .semantic_sources
            .push(SemanticSourceSeed::control(node, kind));
        Ok(node)
    }

    /// Registers an expression occurrence whose syntax children are already
    /// planned. `children` is in source position order, and holds each child's
    /// **occurrence** origin — never its scheduling entry (D9).
    ///
    /// The returned `PlannedExpr` has `entry == occurrence.node`: an ordinary
    /// form is scheduled at the very node its occurrence is registered on.
    /// `ComputationalMatch` is the sole variant that does not go through here for
    /// that reason.
    fn expression_node(
        &mut self,
        kind: TransitionKind,
        owner: StaticSourceId,
        frame: DynamicActivationFrame,
        expr: &'src RuntimeExpr,
        children: &[StaticOriginId],
    ) -> Result<PlannedExpr, CraneliftBackendError> {
        let node = self.push_node(kind, owner, frame)?;
        self.expression_seed(node, expr, children)?;
        Ok(PlannedExpr {
            entry: node,
            occurrence: origin_of(node),
        })
    }

    /// Emits an already-pushed node's semantic material. Split out for the one
    /// occurrence whose node must exist before its children are planned (a
    /// computational match's source-return resume owns the outer edges).
    fn expression_seed(
        &mut self,
        node: StaticNodeId,
        expr: &'src RuntimeExpr,
        children: &[StaticOriginId],
    ) -> Result<(), CraneliftBackendError> {
        match expr {
            RuntimeExpr::Trap(trap)
            | RuntimeExpr::Match { default: trap, .. }
            | RuntimeExpr::ComputationalMatch { default: trap, .. } => {
                self.intern_trap(trap)?;
            }
            RuntimeExpr::PrimitiveCall { primitive, .. } => {
                if let Some(trap) = planned_partiality_trap(primitive) {
                    self.intern_trap(&trap)?;
                }
            }
            _ => {}
        }
        let seed =
            SemanticSourceSeed::expression(node, expr, children, &mut self.plan.semantic_material)?;
        self.plan.semantic_sources.push(seed);
        self.record_source_occurrence(node, expr)
    }

    fn intern_trap(
        &mut self,
        trap: &RuntimeTrap,
    ) -> Result<PlannedTrapIdentity, CraneliftBackendError> {
        if let Some(index) = self
            .plan
            .trap_catalog
            .iter()
            .position(|candidate| candidate == trap)
        {
            return u32::try_from(index + 1)
                .map(PlannedTrapIdentity)
                .map_err(|_| planner_capacity_error("trap identity exhausted"));
        }
        self.plan.trap_catalog.push(trap.clone());
        u32::try_from(self.plan.trap_catalog.len())
            .map(PlannedTrapIdentity)
            .map_err(|_| planner_capacity_error("trap identity exhausted"))
    }

    /// Add the cross-owner edges represented by source `DeclarationRef`
    /// occurrences after all transparent declaration entries exist.
    ///
    /// These are deliberately not `StaticBody` edges: that edge kind is the
    /// closure-body owner boundary and also seeds a function unit. A transparent
    /// declaration entry is already a scheduling-entry seed.
    fn connect_declaration_calls(
        &mut self,
        declaration_entries: &BTreeMap<String, StaticNodeId>,
    ) -> Result<(), CraneliftBackendError> {
        let calls = self
            .plan
            .source_occurrences
            .iter()
            .flatten()
            .filter_map(|occurrence| match occurrence.expr {
                RuntimeExpr::DeclarationRef { symbol } => declaration_entries
                    .get(symbol.as_str())
                    .copied()
                    .map(|target| (StaticNodeId(occurrence.static_origin.0), target)),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (caller, callee) in calls {
            self.edge(caller, callee, EdgeKind::DeclarationCall)?;
        }
        Ok(())
    }

    /// Files this occurrence's term under the origin the planner just gave it.
    ///
    /// ⭐ This is deliberately the *same* function that emits the semantic seed,
    /// not a companion pass: the term and its static name are recorded in one
    /// step, so no ordering between two walks can put them out of agreement. Every
    /// planned occurrence reaches `expression_seed`, so the table is total over
    /// the occurrence population by construction.
    fn record_source_occurrence(
        &mut self,
        node: StaticNodeId,
        expr: &'src RuntimeExpr,
    ) -> Result<(), CraneliftBackendError> {
        let index = node.0 as usize;
        if self.plan.source_occurrences.len() <= index {
            self.plan.source_occurrences.resize(index + 1, None);
        }
        // A second occurrence filed under one origin would make selection
        // ambiguous, and ambiguity here is a compiler bug rather than a program
        // the backend cannot handle — so it is a `PlannerInvariant`, not a
        // capacity refusal (`RT-PLANNER-ATTRIB-K`).
        if self.plan.source_occurrences[index].is_some() {
            return Err(planner_error(
                "static origin was given more than one source occurrence",
            ));
        }
        self.plan.source_occurrences[index] = Some(PlannedOccurrence {
            static_origin: origin_of(node),
            expr,
        });
        Ok(())
    }

    fn edge(
        &mut self,
        from: StaticNodeId,
        to: StaticNodeId,
        kind: EdgeKind,
    ) -> Result<(), CraneliftBackendError> {
        let edge = u32::try_from(self.plan.edges.len())
            .map(StaticEdgeId)
            .map_err(|_| planner_capacity_error("static edge identity exhausted"))?;
        let owner = self.plan.nodes[from.0 as usize].owner;
        self.plan.edges.push(StaticEdge {
            id: edge,
            from,
            to,
            kind,
        });
        self.plan.evidence.push(EdgeEvidence {
            edge: edge.0,
            owner,
            from,
            to,
            kind,
        });
        self.plan
            .planned_helpers
            .push(PlannedHelperKey::edge(kind, edge));
        Ok(())
    }

    fn store(
        &mut self,
        kind: StoreKind,
        local: u32,
        aux: u32,
        child: PersistentNodeId,
    ) -> Result<PersistentNodeId, CraneliftBackendError> {
        let node = PersistentStoreNode {
            kind,
            local,
            aux,
            child,
        };
        if let Some(id) = self.store_interner.get(&node) {
            return Ok(*id);
        }
        let id = u32::try_from(self.plan.stores.len() + 1)
            .map(PersistentNodeId)
            .map_err(|_| planner_capacity_error("persistent store identity exhausted"))?;
        let child_depth = if child.0 == 0 {
            0
        } else {
            *self
                .plan
                .store_depths
                .get(child.0 as usize - 1)
                .ok_or_else(|| planner_error("persistent store child is not closed"))?
        };
        self.plan.stores.push(node);
        self.plan.store_depths.push(
            child_depth
                .checked_add(1)
                .ok_or_else(|| planner_capacity_error("persistent chain depth exhausted"))?,
        );
        self.store_interner.insert(node, id);
        Ok(id)
    }

    fn frame(
        &mut self,
        tag: u32,
        ordinal: u32,
        ctx: PlanContext,
        successor: StaticNodeId,
    ) -> Result<DynamicActivationFrame, CraneliftBackendError> {
        let syntax = self.store(StoreKind::Syntax, tag, ordinal, PersistentNodeId(0))?;
        let path = self.store(StoreKind::Path, ordinal, 0, ctx.path)?;
        let normal = self.store(StoreKind::Continuation, successor.0, 0, ctx.continuation)?;
        Ok(DynamicActivationFrame {
            syntax,
            environment: ctx.environment,
            normal,
            abrupt: PersistentNodeId(0),
            path,
            cleanup: ctx.cleanup,
            affine: ctx.affine,
            source_return: ctx.source_return,
        })
    }

    /// Plans a positional operand sequence. Returns the chain **entry** — what the
    /// parent schedules — and each element's **occurrence** origin indexed by its
    /// source position, which is what the parent records as its positional child
    ///. The two are different values for a
    /// `ComputationalMatch` element, and mixing them is a category error.
    fn plan_sequence(
        &mut self,
        expressions: &[&'src RuntimeExpr],
        ctx: PlanContext,
        successor: StaticNodeId,
        exit_kind: EdgeKind,
    ) -> Result<(StaticNodeId, Vec<StaticOriginId>), CraneliftBackendError> {
        let mut next = successor;
        let mut next_kind = exit_kind;
        let mut occurrences = vec![None; expressions.len()];
        for (ordinal, expression) in expressions.iter().enumerate().rev() {
            let planned = self.plan_expr(expression, ctx, next, next_kind, ordinal as u32)?;
            next = planned.entry;
            occurrences[ordinal] = Some(planned.occurrence);
            next_kind = EdgeKind::Continue;
        }
        let occurrences = occurrences
            .into_iter()
            .map(|occurrence| {
                occurrence.ok_or_else(|| {
                    planner_error("operand sequence position has no planned occurrence")
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((next, occurrences))
    }

    /// Plans one eliminator's case bodies. Returns the dispatch **entry** and each
    /// body's **occurrence** origin by source position (D9): the case test edges
    /// to the body's `entry`, while the parent records the body's `occurrence`.
    fn plan_cases(
        &mut self,
        bodies: &[(&'src RuntimeExpr, usize)],
        ctx: PlanContext,
        successor: StaticNodeId,
        exit_kind: EdgeKind,
        default: StaticNodeId,
    ) -> Result<(StaticNodeId, Vec<StaticOriginId>), CraneliftBackendError> {
        let mut reject = default;
        let mut occurrences = vec![None; bodies.len()];
        for (ordinal, (body, binders)) in bodies.iter().enumerate().rev() {
            let mut body_ctx = ctx;
            for binder in 0..*binders {
                body_ctx.environment = self.store(
                    StoreKind::Environment,
                    binder as u32,
                    0,
                    body_ctx.environment,
                )?;
            }
            let planned = self.plan_expr(body, body_ctx, successor, exit_kind, ordinal as u32)?;
            occurrences[ordinal] = Some(planned.occurrence);
            let owner = self.source()?;
            let frame = self.frame(0x80, ordinal as u32, ctx, reject)?;
            let test = self.control_node(TransitionKind::CaseTest, owner, frame)?;
            // Topology: the test selects the body's SCHEDULING entry.
            self.edge(test, planned.entry, EdgeKind::Select)?;
            self.edge(test, reject, EdgeKind::Reject)?;
            reject = test;
        }
        let occurrences = occurrences
            .into_iter()
            .map(|occurrence| {
                occurrence.ok_or_else(|| planner_error("case position has no planned occurrence"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok((reject, occurrences))
    }

    /// Plans one expression occurrence and returns **both** of its identities
    /// (D9): the `entry` the parent schedules and the `occurrence` the parent
    /// records at its source position. Every arm but `ComputationalMatch` returns
    /// them equal, by going through `expression_node`.
    fn plan_expr(
        &mut self,
        expr: &'src RuntimeExpr,
        ctx: PlanContext,
        successor: StaticNodeId,
        exit_kind: EdgeKind,
        ordinal: u32,
    ) -> Result<PlannedExpr, CraneliftBackendError> {
        #[cfg(test)]
        let _recursive_lowering_frame = RecursiveLoweringFrameGuard::enter();
        let owner = self.source()?;
        let tag = runtime_expr_tag(expr);
        let frame = self.frame(tag, ordinal, ctx, successor)?;
        let ctx = PlanContext {
            continuation: frame.normal,
            path: frame.path,
            ..ctx
        };
        match expr {
            RuntimeExpr::Trap(_) => {
                let node =
                    self.expression_node(TransitionKind::Evaluate, owner, frame, expr, &[])?;
                self.edge(node.entry, self.trap_terminal, EdgeKind::Trap)?;
                Ok(node)
            }
            RuntimeExpr::Value(_)
            | RuntimeExpr::Var(_)
            | RuntimeExpr::DeclarationRef { .. }
            | RuntimeExpr::ImportedDeclarationRef { .. } => {
                let node =
                    self.expression_node(TransitionKind::Evaluate, owner, frame, expr, &[])?;
                self.edge(node.entry, successor, exit_kind)?;
                Ok(node)
            }
            RuntimeExpr::CheckedJoinSite { body, .. }
            | RuntimeExpr::CheckedSubcontinuationFrame { body, .. }
            | RuntimeExpr::CheckedRecursiveInvocation { body, .. }
            | RuntimeExpr::CheckedComputationalIHSlots { body, .. }
            | RuntimeExpr::CheckedComputationalIHInvocation { body, .. }
            | RuntimeExpr::Project { record: body, .. } => {
                let body = self.plan_expr(body, ctx, successor, exit_kind, 0)?;
                let node = self.expression_node(
                    TransitionKind::Sequence,
                    owner,
                    frame,
                    expr,
                    &[body.occurrence],
                )?;
                self.edge(node.entry, body.entry, EdgeKind::Continue)?;
                Ok(node)
            }
            RuntimeExpr::Let { value, body } => {
                let environment = self.store(StoreKind::Environment, 0, 0, ctx.environment)?;
                let body = self.plan_expr(
                    body,
                    PlanContext { environment, ..ctx },
                    successor,
                    exit_kind,
                    1,
                )?;
                let value = self.plan_expr(value, ctx, body.entry, EdgeKind::Continue, 0)?;
                let node = self.expression_node(
                    TransitionKind::Sequence,
                    owner,
                    frame,
                    expr,
                    &[value.occurrence, body.occurrence],
                )?;
                self.edge(node.entry, value.entry, EdgeKind::Continue)?;
                Ok(node)
            }
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => {
                let then_entry = self.plan_expr(then_expr, ctx, successor, exit_kind, 1)?;
                let else_entry = self.plan_expr(else_expr, ctx, successor, exit_kind, 2)?;
                let branch_owner = self.source()?;
                let branch = self.control_node(TransitionKind::Branch, branch_owner, frame)?;
                self.edge(branch, then_entry.entry, EdgeKind::Select)?;
                self.edge(branch, else_entry.entry, EdgeKind::Reject)?;
                let scrutinee = self.plan_expr(scrutinee, ctx, branch, EdgeKind::Continue, 0)?;
                let node = self.expression_node(
                    TransitionKind::Evaluate,
                    owner,
                    frame,
                    expr,
                    &[
                        scrutinee.occurrence,
                        then_entry.occurrence,
                        else_entry.occurrence,
                    ],
                )?;
                self.edge(node.entry, scrutinee.entry, EdgeKind::Continue)?;
                Ok(node)
            }
            RuntimeExpr::Match {
                scrutinee, cases, ..
            } => {
                let default_owner = self.source()?;
                let default = self.control_node(TransitionKind::Evaluate, default_owner, frame)?;
                self.edge(default, self.trap_terminal, EdgeKind::Trap)?;
                let bodies = cases
                    .iter()
                    .map(|case| (&case.body, case.binders))
                    .collect::<Vec<_>>();
                let (dispatch, case_bodies) =
                    self.plan_cases(&bodies, ctx, successor, exit_kind, default)?;
                let scrutinee = self.plan_expr(scrutinee, ctx, dispatch, EdgeKind::Continue, 0)?;
                let mut children = Vec::with_capacity(1 + case_bodies.len());
                children.push(scrutinee.occurrence);
                children.extend(case_bodies);
                let node =
                    self.expression_node(TransitionKind::Evaluate, owner, frame, expr, &children)?;
                self.edge(node.entry, scrutinee.entry, EdgeKind::Continue)?;
                Ok(node)
            }
            RuntimeExpr::ComputationalMatch {
                scrutinee, cases, ..
            } => {
                let cleanup = self.store(StoreKind::Cleanup, owner.0, 0, ctx.cleanup)?;
                let affine = self.store(StoreKind::Affine, owner.0, 0, ctx.affine)?;
                let control_ctx = PlanContext {
                    cleanup,
                    affine,
                    ..ctx
                };
                let completed = self.control_node(TransitionKind::CompletedTail, owner, frame)?;
                let tail = self.control_node(TransitionKind::ProducerTail, owner, frame)?;
                let wrapper = self.control_node(TransitionKind::ProducerWrapper, owner, frame)?;
                let resume = self.push_node(TransitionKind::SourceReturnResume, owner, frame)?;
                self.edge(resume, wrapper, EdgeKind::InvokeProducerWrapper)?;
                self.edge(wrapper, tail, EdgeKind::InvokeProducerTail)?;
                self.edge(tail, completed, EdgeKind::CompleteProducerTail)?;
                self.edge(completed, successor, exit_kind)?;
                let source_return = self.store(
                    StoreKind::SourceReturn,
                    wrapper.0,
                    tail.0,
                    ctx.source_return,
                )?;
                let control_ctx = PlanContext {
                    source_return,
                    ..control_ctx
                };
                for id in [completed, tail, wrapper, resume] {
                    self.plan.nodes[id.0 as usize].frame.source_return = source_return;
                    self.plan.nodes[id.0 as usize].frame.cleanup = cleanup;
                    self.plan.nodes[id.0 as usize].frame.affine = affine;
                }
                let default_owner = self.source()?;
                let default = self.control_node(TransitionKind::Evaluate, default_owner, frame)?;
                self.edge(default, self.trap_terminal, EdgeKind::Trap)?;
                let bodies = cases
                    .iter()
                    .map(|case| {
                        (
                            &case.body,
                            case.argument_binders + case.recursive_positions.len(),
                        )
                    })
                    .collect::<Vec<_>>();
                let (dispatch, case_bodies) = self.plan_cases(
                    &bodies,
                    control_ctx,
                    resume,
                    EdgeKind::SourceReturnOwnedResume,
                    default,
                )?;
                let scrutinee =
                    self.plan_expr(scrutinee, control_ctx, dispatch, EdgeKind::Continue, 0)?;
                let mut children = Vec::with_capacity(1 + case_bodies.len());
                children.push(scrutinee.occurrence);
                children.extend(case_bodies);
                self.expression_seed(resume, expr, &children)?;
                // ⭐ THE SOLE SPLIT. This occurrence's record
                // lives on `resume`, because the resume owns the outer edges and
                // must exist before the cases are planned — but the transfer
                // graph still schedules the SCRUTINEE, exactly as before. So the
                // two identities genuinely differ here, and returning one value
                // for both is what made a parent record the scrutinee's identity
                // as this match's. ⛔ Do not "fix" this by returning `resume` as
                // the entry: that would change the approved Boundary-A topology.
                Ok(PlannedExpr {
                    entry: scrutinee.entry,
                    occurrence: origin_of(resume),
                })
            }
            RuntimeExpr::Closure { body, .. } => {
                let body_return_owner = self.source()?;
                let body_return =
                    self.control_node(TransitionKind::ClosureBody, body_return_owner, frame)?;
                self.edge(body_return, self.terminal, EdgeKind::Continue)?;
                let body = self.plan_expr(body, ctx, body_return, EdgeKind::Continue, 0)?;
                let node = self.expression_node(
                    TransitionKind::Evaluate,
                    owner,
                    frame,
                    expr,
                    &[body.occurrence],
                )?;
                self.edge(node.entry, successor, exit_kind)?;
                self.edge(node.entry, body.entry, EdgeKind::StaticBody)?;
                Ok(node)
            }
            RuntimeExpr::LexicalClosure { captures, body, .. } => {
                let body_return_owner = self.source()?;
                let body_return =
                    self.control_node(TransitionKind::ClosureBody, body_return_owner, frame)?;
                self.edge(body_return, self.terminal, EdgeKind::Continue)?;
                let body = self.plan_expr(body, ctx, body_return, EdgeKind::Continue, 0)?;
                let captures = captures.iter().collect::<Vec<_>>();
                let (capture_entry, capture_occurrences) =
                    self.plan_sequence(&captures, ctx, successor, exit_kind)?;
                let mut children = Vec::with_capacity(1 + capture_occurrences.len());
                children.push(body.occurrence);
                children.extend(capture_occurrences);
                let node =
                    self.expression_node(TransitionKind::Evaluate, owner, frame, expr, &children)?;
                self.edge(
                    node.entry,
                    capture_entry,
                    if captures.is_empty() {
                        exit_kind
                    } else {
                        EdgeKind::Continue
                    },
                )?;
                self.edge(node.entry, body.entry, EdgeKind::StaticBody)?;
                Ok(node)
            }
            RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
                let expressions = args.iter().collect::<Vec<_>>();
                let (first, operand_occurrences) =
                    self.plan_sequence(&expressions, ctx, successor, exit_kind)?;
                let node = self.expression_node(
                    TransitionKind::Sequence,
                    owner,
                    frame,
                    expr,
                    &operand_occurrences,
                )?;
                self.edge(
                    node.entry,
                    first,
                    if expressions.is_empty() {
                        exit_kind
                    } else {
                        EdgeKind::Continue
                    },
                )?;
                Ok(node)
            }
            RuntimeExpr::Record { fields } => {
                let expressions = fields.iter().map(|(_, value)| value).collect::<Vec<_>>();
                let (first, operand_occurrences) =
                    self.plan_sequence(&expressions, ctx, successor, exit_kind)?;
                let node = self.expression_node(
                    TransitionKind::Sequence,
                    owner,
                    frame,
                    expr,
                    &operand_occurrences,
                )?;
                self.edge(
                    node.entry,
                    first,
                    if expressions.is_empty() {
                        exit_kind
                    } else {
                        EdgeKind::Continue
                    },
                )?;
                Ok(node)
            }
            RuntimeExpr::Call { callee, args } => {
                let mut expressions = Vec::with_capacity(args.len() + 1);
                expressions.push(callee.as_ref());
                expressions.extend(args);
                let (first, operand_occurrences) =
                    self.plan_sequence(&expressions, ctx, successor, exit_kind)?;
                let node = self.expression_node(
                    TransitionKind::Sequence,
                    owner,
                    frame,
                    expr,
                    &operand_occurrences,
                )?;
                self.edge(
                    node.entry,
                    first,
                    if expressions.is_empty() {
                        exit_kind
                    } else {
                        EdgeKind::Continue
                    },
                )?;
                Ok(node)
            }
            RuntimeExpr::Effect {
                capability, args, ..
            } => {
                let mut expressions =
                    Vec::with_capacity(args.len() + usize::from(capability.is_some()));
                if let Some(capability) = capability {
                    expressions.push(capability.value.as_ref());
                }
                expressions.extend(args);
                let (first, operand_occurrences) =
                    self.plan_sequence(&expressions, ctx, successor, exit_kind)?;
                let node = self.expression_node(
                    TransitionKind::Sequence,
                    owner,
                    frame,
                    expr,
                    &operand_occurrences,
                )?;
                self.edge(
                    node.entry,
                    first,
                    if expressions.is_empty() {
                        exit_kind
                    } else {
                        EdgeKind::Continue
                    },
                )?;
                Ok(node)
            }
        }
    }

    fn finish(
        mut self,
        symbols: &crate::NativeProcessSymbols,
        root_ingress: AbiRootIngress,
        functionized_units: bool,
    ) -> Result<StaticTransitionPlan<'src>, CraneliftBackendError> {
        let (synthesized_identities, synthesized_io_roles) =
            build_synthesized_constructor_inventory(&mut self.plan.semantic_material, symbols)?;
        self.plan.semantic = build_semantic_plane(
            &self.plan.nodes,
            &self.plan.edges,
            &self.plan.entries,
            &self.plan.semantic_sources,
            &self.plan.semantic_material,
        )?;
        self.plan
            .semantic
            .install_synthesized_constructor_inventory(
                synthesized_identities,
                synthesized_io_roles,
            );
        self.plan
            .semantic
            .validate_synthesized_constructor_inventory()?;
        self.plan.terminal_exit_aggregate_origins = self
            .plan
            .source_occurrences
            .iter()
            .flatten()
            .filter_map(|occurrence| match occurrence.expr {
                RuntimeExpr::Construct { constructor, .. }
                    if constructor == &symbols.exit_success
                        || constructor == &symbols.exit_failure =>
                {
                    Some(occurrence.static_origin)
                }
                _ => None,
            })
            .collect();
        // `B2R` — the representation contract is built from the owner partition
        // the line above just validated, and it fails **before** anything is
        // emitted. It is deliberately not deferred to lowering: a contract that
        // is only checked at emission time cannot be a *pre*-emission gate.
        let root_entry = self
            .plan
            .root_entry
            .ok_or_else(|| planner_error("plan has no root scheduling entry"))?;
        self.plan.root_ingress = root_ingress;
        self.plan.abi = build_abi_plane(
            &self.plan.semantic,
            &self.plan.nodes,
            &self.plan.semantic_sources,
            &self.plan.edges,
            &self.plan.entries,
            root_entry,
            root_ingress,
        )?;
        self.plan.functionized_units = functionized_units;
        let (join_results, result_phases) = build_join_result_plan(&self.plan, functionized_units)?;
        self.plan.join_results = join_results;
        self.plan.result_phases = result_phases;
        let (specializations, calls) = if functionized_units {
            build_static_callable_specializations(&self.plan)?
        } else {
            (Vec::new(), Vec::new())
        };
        self.plan.static_callable_specializations = specializations;
        self.plan.static_callable_calls = calls;
        extend_static_callable_abi(
            &mut self.plan.abi,
            &self.plan.semantic,
            &self.plan.static_callable_specializations,
        )?;
        self.plan.operand_edges = build_operand_edge_matrix(&self.plan)?;
        self.plan.static_recursor_worker_residuals =
            build_static_recursor_worker_residuals(&self.plan)?;
        self.plan.recursor_boundary_uses = build_recursor_boundary_uses(&self.plan)?;
        self.plan.lowering_boundary_uses = build_lowering_boundary_uses(&self.plan)?;
        self.plan.boundary_uses = build_boundary_uses(&self.plan)?;
        let (case_emissions, aggregate_representations, synthesized_aggregate_representations) =
            build_producer_flow_plans(&self.plan)?;
        self.plan.case_emissions = case_emissions;
        self.plan.aggregate_representations = aggregate_representations;
        self.plan.synthesized_aggregate_representations = synthesized_aggregate_representations;
        #[cfg(test)]
        STATIC_RECURSOR_RESIDUAL_MATRIX_MUTATION.with(|mutation| match mutation.get() {
            StaticRecursorResidualMatrixMutation::Exact => {}
            StaticRecursorResidualMatrixMutation::OmitFirst => {
                if !self.plan.static_recursor_worker_residuals.is_empty() {
                    self.plan.static_recursor_worker_residuals.remove(0);
                }
            }
            StaticRecursorResidualMatrixMutation::ReclassifyFirst => {
                if let Some(first) = self.plan.static_recursor_worker_residuals.first_mut() {
                    first.disposition = OperandEdgeDisposition::Forwarding;
                }
            }
        });
        self.plan.validate()?;
        Ok(self.plan)
    }
}

/// **`RT-FNSPLIT-B2F` `D1` — the emitter's read-only view of ONE validated
/// function unit.**
///
/// ⭐ **This is the `case_constructor_identity` precedent, not a widened
/// field.** What crosses into `crate::cranelift_backend` is a *question about
/// a unit* and an answer the asker cannot mint: `AbiPlane`, `AbiDescriptor`,
/// `build_abi_plane` and `AbiPlane::validate` all stay `pub(super)`, so the
/// emitter can neither construct a plane, mutate a descriptor, nor reach the
/// pre-emission validator to bypass it.
///
/// ⛔ **The fields are private and there is no constructor**, so a unit
/// cannot be forged in `lowering`. That is the load-bearing half: `B2F`
/// drives emission from units, so an unmintable unit means emission cannot
/// be driven from anything but the validated plane.
///
/// **MEASURED:** `lowering` can read a unit's declared identity, origin,
/// definition, header and slot run, and can construct none of them.
/// **CLAIMED:** emission is driven by `B2R`'s validated authority rather than
/// by a second table `B2F` derives for itself.
/// **THE GAP:** ⚠ `AbiSlot` and `AbiFrameHeader` are plain `Copy` data whose
/// fields are now readable in `cranelift_backend`, so `lowering` **can**
/// spell a *local* `AbiSlot` literal — Rust cannot forbid struct-literal
/// construction inside one crate. ⛔ **This is not claimed to be detected.**
/// What closes it is that a forged slot has no route into a unit: the only
/// producer of an `EmittableUnit` is [`Self::emittable_units`], which reads
/// `self.abi`. A control that emission consumes only unit-supplied slots is
/// `AC-12`'s, and it is not discharged here.
/// **One cross-owner call edge, as the emitter is allowed to see it.**
///
/// ⭐ **Both ends are `PredeclaredFunctionId`s and nothing else.** There is no
/// node id, no origin and no expression here, because a call edge's whole
/// content at emission time is *which unit calls which unit* — and resolving a
/// callee to a target function must go through the planner's identity, never
/// through the ordinal some emission loop happened to assign.
///
/// ⛔ **Unmintable in `lowering`:** the fields are private and the sole producer
/// is [`StaticTransitionPlan::emittable_call_edges`]. ⇒ The emitter cannot
/// invent a call to a unit the planner did not connect, which is the property
/// that makes "no indirect dispatch on a dynamic property" structural rather
/// than a coding convention.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) struct EmittableCallEdge {
    caller: PredeclaredFunctionId,
    callee: PredeclaredFunctionId,
    callee_origin: StaticOriginId,
    call_site_origin: StaticOriginId,
    kind: EmittableCallKind,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum EmittableCallKind {
    StaticBody,
    Declaration,
    StaticCallableSpecialization,
}

impl EmittableCallEdge {
    /// The unit this call is emitted **into**.
    pub(in crate::cranelift_backend) fn caller(self) -> PredeclaredFunctionId {
        self.caller
    }

    /// The unit this call transfers **to**. ⛔ Resolve it through
    /// `UnitBundle::function`, whose `None` is a real answer.
    pub(in crate::cranelift_backend) fn callee(self) -> PredeclaredFunctionId {
        self.callee
    }

    pub(in crate::cranelift_backend) fn callee_origin(self) -> StaticOriginId {
        self.callee_origin
    }

    /// The source occurrence which owns this call operation.
    ///
    /// For a closure-body call this is the body target, preserving the
    /// established lookup. For a declaration call it is the exact
    /// `DeclarationRef` occurrence, so two references to one declaration remain
    /// distinct typed edges without emitter-side symbol lookup.
    pub(in crate::cranelift_backend) fn call_site_origin(self) -> StaticOriginId {
        self.call_site_origin
    }

    pub(in crate::cranelift_backend) fn kind(self) -> EmittableCallKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum EmittableStaticCallableArgumentKind {
    Ordinary,
    Erased,
    Direct {
        closure_origin: StaticOriginId,
    },
    Forwarded {
        body_origin: StaticOriginId,
        declared_arity: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct EmittableStaticCallableArgument {
    parameter_ordinal: u32,
    argument_origin: StaticOriginId,
    kind: EmittableStaticCallableArgumentKind,
    binding: Option<EmittableStaticCallableBinding>,
}

impl EmittableStaticCallableArgument {
    pub(in crate::cranelift_backend) fn parameter_ordinal(&self) -> u32 {
        self.parameter_ordinal
    }

    pub(in crate::cranelift_backend) fn argument_origin(&self) -> StaticOriginId {
        self.argument_origin
    }

    pub(in crate::cranelift_backend) fn kind(&self) -> EmittableStaticCallableArgumentKind {
        self.kind
    }

    pub(in crate::cranelift_backend) fn binding(&self) -> Option<&EmittableStaticCallableBinding> {
        self.binding.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct EmittableStaticCallableCall {
    arguments: Vec<EmittableStaticCallableArgument>,
}

impl EmittableStaticCallableCall {
    pub(in crate::cranelift_backend) fn arguments(&self) -> &[EmittableStaticCallableArgument] {
        &self.arguments
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum EmittableStaticCallableCapture {
    Value,
    Callable(EmittableStaticCallableBinding),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct EmittableStaticCallableBinding {
    parameter_ordinal: u32,
    body_origin: StaticOriginId,
    declared_arity: u32,
    capture_count: u32,
    captures: Vec<EmittableStaticCallableCapture>,
}

impl EmittableStaticCallableBinding {
    pub(in crate::cranelift_backend) fn parameter_ordinal(&self) -> u32 {
        self.parameter_ordinal
    }

    pub(in crate::cranelift_backend) fn body_origin(&self) -> StaticOriginId {
        self.body_origin
    }

    pub(in crate::cranelift_backend) fn declared_arity(&self) -> u32 {
        self.declared_arity
    }

    pub(in crate::cranelift_backend) fn capture_count(&self) -> u32 {
        self.capture_count
    }

    pub(in crate::cranelift_backend) fn captures(&self) -> &[EmittableStaticCallableCapture] {
        &self.captures
    }
}

fn emittable_static_callable_binding(
    binding: &StaticCallableBindingKey,
) -> Result<EmittableStaticCallableBinding, CraneliftBackendError> {
    Ok(EmittableStaticCallableBinding {
        parameter_ordinal: binding.parameter_ordinal,
        body_origin: binding.body_origin,
        declared_arity: binding.declared_arity,
        capture_count: binding.lifted_capture_count()?,
        captures: binding
            .captures
            .iter()
            .map(|capture| match capture {
                StaticCallableCaptureBinding::Value(_) => Ok(EmittableStaticCallableCapture::Value),
                StaticCallableCaptureBinding::Callable(binding) => {
                    Ok(EmittableStaticCallableCapture::Callable(
                        emittable_static_callable_binding(binding)?,
                    ))
                }
            })
            .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct EmittableStaticCallableUnit {
    base_origin: StaticOriginId,
    base_body_origin: StaticOriginId,
    parameter_count: u32,
    declaration_captures: u32,
    bindings: Vec<EmittableStaticCallableBinding>,
    body_binding: Option<EmittableStaticCallableBinding>,
}

impl EmittableStaticCallableUnit {
    pub(in crate::cranelift_backend) fn base_origin(&self) -> StaticOriginId {
        self.base_origin
    }

    pub(in crate::cranelift_backend) fn base_body_origin(&self) -> StaticOriginId {
        self.base_body_origin
    }

    pub(in crate::cranelift_backend) fn parameter_count(&self) -> u32 {
        self.parameter_count
    }

    pub(in crate::cranelift_backend) fn declaration_captures(&self) -> u32 {
        self.declaration_captures
    }

    pub(in crate::cranelift_backend) fn bindings(&self) -> &[EmittableStaticCallableBinding] {
        &self.bindings
    }

    pub(in crate::cranelift_backend) fn body_binding(
        &self,
    ) -> Option<&EmittableStaticCallableBinding> {
        self.body_binding.as_ref()
    }
}

#[derive(Clone, Copy, Debug)]
pub(in crate::cranelift_backend) struct EmittableUnit<'plan> {
    function: PredeclaredFunctionId,
    origin: StaticOriginId,
    definition: AbiUnitDefinition,
    header: AbiFrameHeader,
    slots: &'plan [AbiSlot],
}

impl<'plan> EmittableUnit<'plan> {
    /// This unit's static identity. ⛔ Unmintable in `lowering`: the newtype's
    /// field stays `pub(super)`, so the emitter can key and compare an id but
    /// cannot fabricate one or do arithmetic on it.
    pub(in crate::cranelift_backend) fn function(self) -> PredeclaredFunctionId {
        self.function
    }

    /// The occurrence origin of this unit's body, for
    /// [`StaticTransitionPlan::source_occurrence`].
    pub(in crate::cranelift_backend) fn origin(self) -> StaticOriginId {
        self.origin
    }

    /// Whether this unit is a scheduling entry or a retained closure body,
    /// with the closure body's defining origin and capture provenance.
    pub(in crate::cranelift_backend) fn definition(self) -> AbiUnitDefinition {
        self.definition
    }

    /// The declared activation-frame header. ⚠ `frame_bytes` is derived from
    /// the slot run by `B2R`; do not recompute it from [`Self::slots`].
    pub(in crate::cranelift_backend) fn header(self) -> AbiFrameHeader {
        self.header
    }

    /// This unit's declared slots, in `B2R`'s layout order: parameters,
    /// captures, result, control, trap, store.
    pub(in crate::cranelift_backend) fn slots(self) -> &'plan [AbiSlot] {
        self.slots
    }

    /// Each slot's byte offset in this unit's activation frame, paired with the
    /// frame's total size.
    ///
    /// ⛔ **Delegated to `abi::slot_offsets`, never re-derived here.** The
    /// emitter needs offsets to load and store slots, and prefix-summing the
    /// widths at the emission site would put the same arithmetic in a second
    /// file where the two can silently disagree. `AbiFrameHeader::frame_bytes`
    /// is totalled *through* the same walk, so the offsets the emitter uses and
    /// the size the ABI declares cannot diverge.
    ///
    /// ⚠ The returned total is checked against [`Self::header`]'s `frame_bytes`
    /// by `AC-3`, not here — this accessor's job is to have one walk, not to
    /// assert about it.
    pub(in crate::cranelift_backend) fn slot_offsets(
        self,
    ) -> Result<(Vec<u32>, u32), CraneliftBackendError> {
        abi::slot_offsets(self.slots)
    }
}

impl StaticTransitionPlan<'_> {
    /// Resolve one process-root parameter by its closed semantic role.
    ///
    /// The caller cannot restate ordinals: only the scheduling entry whose
    /// validated definition carries `ProcessPair` can answer, and the slot
    /// offset comes from B2R's sole offset walk.
    pub(in crate::cranelift_backend) fn process_parameter_slot(
        &self,
        role: AbiProcessParameter,
    ) -> Result<Option<(AbiSlot, u32)>, CraneliftBackendError> {
        let mut answer = None;
        for unit in self.emittable_units()? {
            if unit.definition()
                != (AbiUnitDefinition::SchedulingEntry {
                    ingress: AbiSchedulingIngress::ProcessPair,
                })
            {
                continue;
            }
            let (offsets, _) = unit.slot_offsets()?;
            let found = unit
                .slots()
                .iter()
                .copied()
                .zip(offsets)
                .find(|(slot, _)| {
                    slot.kind == AbiSlotKind::Parameter && slot.ordinal == role.ordinal()
                })
                .ok_or_else(|| planner_error("process ingress role has no declared root slot"))?;
            if answer.replace(found).is_some() {
                return Err(planner_error(
                    "more than one scheduling entry declares process ingress",
                ));
            }
        }
        Ok(answer)
    }
}

// **`RT-FNSPLIT-B2A-S` `AC-4` — the route counters.**
//
// ⛔⛔ **These exist because the instrument that used to carry `AC-4` cannot
// carry it through `B2F`.** That instrument reads this file's source text and
// asserts a list of exported signatures; it constrains the *identifier*
// `source_occurrence` and says nothing about **who calls the route** — and
// `B2F` `S6` widens `Lowering::retained_body_occurrence` from private-to-`core`
// to all of `lowering` so a unit body can resolve its own origin. ⚠ A
// source-text oracle also reddens on a reflow that changes nothing about how
// any program behaves, which is why the replacement is a behavioural one.
//
// ⭐ **The property is a RATIO, not a count, and that is what makes it durable.**
// `retained_body_occurrence` calls [`StaticTransitionPlan::source_occurrence`]
// exactly once, so the two counters move together **for as long as that route
// is the only caller**. Any second call site — a convenience resolver, a
// "just this once" direct call from an emission site — makes resolutions
// exceed route invocations, and nothing else can.
//
// ⚠ **Deliberately NOT a bound on how many times the route is used.** Seven
// consumption sites call it today and more may; `AC-4` holds the number of
// **routes** at one, never the number of resolutions. ⛔ A pin that froze the
// call count would go red on legitimate work and would be a snapshot wearing an
// invariant's name.
#[cfg(test)]
thread_local! {
    /// Resolutions performed by `source_occurrence`, since the last window open.
    static AC4_RESOLUTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Invocations of the single route, since the last window open.
    static AC4_ROUTE_INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Zero both counters. ⛔ Call this immediately before the compile under
/// measurement: without a per-window reset a reading cannot distinguish this
/// compile's resolutions from an earlier one's, and a stale equal pair reads
/// exactly like the outcome the pin wants.
#[cfg(test)]
pub(in crate::cranelift_backend) fn ac4_open_route_window() {
    AC4_RESOLUTIONS.with(|cell| cell.set(0));
    AC4_ROUTE_INVOCATIONS.with(|cell| cell.set(0));
}

/// Record one invocation of the single `origin -> expression` route.
///
/// ⚠ Called by `Lowering::retained_body_occurrence` and by nothing else — that
/// is the whole point. A second route that recorded itself here would be
/// *claiming* to be the single route, which is a visible lie rather than a
/// silent one.
#[cfg(test)]
pub(in crate::cranelift_backend) fn ac4_note_route_invocation() {
    AC4_ROUTE_INVOCATIONS.with(|cell| cell.set(cell.get() + 1));
}

/// `(resolutions, route invocations)` since the window opened.
#[cfg(test)]
pub(in crate::cranelift_backend) fn ac4_route_counts() -> (usize, usize) {
    (
        AC4_RESOLUTIONS.with(std::cell::Cell::get),
        AC4_ROUTE_INVOCATIONS.with(std::cell::Cell::get),
    )
}

impl<'src> StaticTransitionPlan<'src> {
    fn callable_base_functions(
        &self,
    ) -> Result<BTreeSet<PredeclaredFunctionId>, CraneliftBackendError> {
        let mut functions = BTreeSet::new();
        for symbol in self.declaration_occurrences.keys() {
            let Some(declaration) = callable_declaration_plan(self, symbol)? else {
                continue;
            };
            if declaration
                .parameter_uses
                .iter()
                .any(|use_| use_.is_callable())
            {
                functions.insert(declaration.function);
                functions.insert(declaration.body_function);
            }
        }
        for specialization in &self.static_callable_specializations {
            if matches!(
                specialization.kind,
                PlannedStaticCallableSpecializationKind::CallableBody { .. }
            ) {
                functions.insert(specialization.body_function);
            }
        }
        Ok(functions)
    }

    /// Resolves a static origin to the source term the planner filed under it.
    ///
    /// ⭐ **This is the sole `origin -> expression` route in the backend**, and it
    /// is what `RT-FNSPLIT-B2A-C`'s N3 pin asserted did not exist. B2A-S retires
    /// that pin deliberately: the count goes from zero to **exactly one**, so a
    /// retained body is selected by its static name and by nothing else.
    ///
    /// Three distinct ways to be wrong, each rejected separately so a mutation
    /// that breaks one is distinguishable from a mutation that breaks another:
    ///
    /// 1. an origin past the end of the table — outside the planned population;
    /// 2. an origin naming a planned node that is **not** a source occurrence
    ///    (a control node), whose slot is legitimately empty;
    /// 3. an entry whose **stored** origin disagrees with the index it was found
    ///    at — the table itself is corrupt, and returning a term from a
    ///    mis-indexed entry is exactly the wrong-body substitution this WP
    ///    exists to make impossible.
    ///
    /// ⛔ The returned lifetime is the **plan's** `'src`, not `&self`'s: the
    /// borrow outlives this call, which is what lets a `&mut self` lowering step
    /// resolve a tag and then lower the result. That is also why the plan cannot
    /// escape — see `Lowering::static_transition_plan`.
    pub(in crate::cranelift_backend) fn source_occurrence(
        &self,
        static_origin: StaticOriginId,
    ) -> Result<&'src RuntimeExpr, CraneliftBackendError> {
        // ⭐ Counted at ENTRY, not on the success path. A resolution that fails
        // is still a resolution *attempted through this route*, and `AC-4` is a
        // claim about routes, not about outcomes — counting only successes would
        // let a second caller hide behind a bad origin.
        #[cfg(test)]
        AC4_RESOLUTIONS.with(|cell| cell.set(cell.get() + 1));
        let index = static_origin.0 as usize;
        let slot = self.source_occurrences.get(index).ok_or_else(|| {
            planner_error("static origin is outside the planned occurrence table")
        })?;
        let occurrence = slot
            .as_ref()
            .ok_or_else(|| planner_error("static origin names no planned source occurrence"))?;
        if occurrence.static_origin != static_origin {
            return Err(planner_error(
                "planned occurrence's stored origin disagrees with its table position",
            ));
        }
        Ok(occurrence.expr)
    }

    /// The preallocated origin of one positional syntax child of `parent`.
    ///
    /// This is the **sole** production point for a child's static name, and the
    /// only admissible one: the position is the child's source-field ordinal and
    /// the value comes out of B1R's checked positional child-origin range. There
    /// is deliberately no pointer, content, hash, clone-order, or visit-order
    /// route to an origin, and no arithmetic that mints one
    ///.
    pub(in crate::cranelift_backend) fn child_static_origin(
        &self,
        parent: StaticOriginId,
        position: usize,
    ) -> Result<StaticOriginId, CraneliftBackendError> {
        self.semantic.child_origin(parent, position)
    }

    #[cfg(test)]
    pub(in crate::cranelift_backend) fn function_owner_for_test(
        &self,
        origin: StaticOriginId,
    ) -> Result<PredeclaredFunctionId, CraneliftBackendError> {
        self.semantic
            .function_owner(origin)?
            .ok_or_else(|| planner_error("test fixture origin has no function owner"))
    }

    /// Consume the planner-owned disposition for one positional source edge.
    ///
    /// The caller must name the role it is implementing. Parent, checked
    /// position, child, owner and disposition were all closed before emission;
    /// a second lowering-side edge inventory cannot mint a token.
    pub(in crate::cranelift_backend) fn operand_edge_token(
        &self,
        parent: StaticOriginId,
        position: usize,
        role: SourceOperandRole,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let child = self.semantic.child_origin(parent, position)?;
        let position = u32::try_from(position)
            .map_err(|_| planner_capacity_error("operand-edge position exhausted"))?;
        let owner = self
            .semantic
            .function_owner(parent)?
            .ok_or_else(|| planner_error("source operand edge has no function owner"))?;
        let edge = self
            .operand_edges
            .iter()
            .find(|edge| {
                edge.owner == owner
                    && edge.parent == parent
                    && edge.child == child
                    && edge.position == position
                    && edge.role == role
            })
            .ok_or_else(|| planner_error("source operand edge has no planned disposition"))?;
        let identity = BoundaryUseIdentity::Source {
            parent: edge.parent,
            child: edge.child,
            position: edge.position,
        };
        let token =
            self.planned_boundary_use_token(identity, source_operand_role_label(edge.role))?;
        self.record_boundary_use_consumption(identity)?;
        Ok(token)
    }

    /// Consume one exact admitted host-operation semantic seat.
    ///
    /// The structural role is checked as inventory, while operation and seat
    /// are the semantic consumer key. A lowering arm therefore cannot borrow a
    /// same-role token from another host operation or another argument.
    pub(in crate::cranelift_backend) fn effect_operand_edge_token(
        &self,
        parent: StaticOriginId,
        position: usize,
        role: SourceOperandRole,
        operation: ken_host::HostOpV1,
        seat: EffectSemanticSeat,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let child = self.semantic.child_origin(parent, position)?;
        let position = u32::try_from(position)
            .map_err(|_| planner_capacity_error("effect-seat position exhausted"))?;
        let owner = self
            .semantic
            .function_owner(parent)?
            .ok_or_else(|| planner_error("effect-seat edge has no function owner"))?;
        let edge = self
            .operand_edges
            .iter()
            .find(|edge| {
                edge.owner == owner
                    && edge.parent == parent
                    && edge.child == child
                    && edge.position == position
                    && edge.role == role
                    && edge.effect_operation == Some(operation)
                    && edge.effect_seat == Some(seat)
            })
            .ok_or_else(|| planner_error("effect use has no exact planned semantic seat"))?;
        let identity = BoundaryUseIdentity::Source {
            parent: edge.parent,
            child: edge.child,
            position: edge.position,
        };
        let token =
            self.planned_boundary_use_token(identity, source_operand_role_label(edge.role))?;
        if token.effect_operation != Some(operation)
            || token.effect_seat != Some(seat)
            || token.need != seat.need()
            || token.avail != BoundaryUseAvail::SemanticObservation
        {
            return Err(planner_error(
                "effect semantic-seat authority does not satisfy its exact consumer need",
            ));
        }
        if !self
            .operand_edge_consumption
            .borrow()
            .contains_key(&identity)
        {
            self.record_boundary_use_consumption(identity)?;
        }
        Ok(token)
    }

    fn record_boundary_use_consumption(
        &self,
        identity: BoundaryUseIdentity,
    ) -> Result<(), CraneliftBackendError> {
        let mut ledger = self.operand_edge_consumption.borrow_mut();
        let count = ledger.entry(identity).or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or_else(|| planner_capacity_error("boundary-use consumption count exhausted"))?;
        Ok(())
    }

    pub(in crate::cranelift_backend) fn disposition_operand_edge(
        &self,
        parent: StaticOriginId,
        position: usize,
        role: SourceOperandRole,
    ) -> Result<(), CraneliftBackendError> {
        let child = self.semantic.child_origin(parent, position)?;
        let position = u32::try_from(position)
            .map_err(|_| planner_capacity_error("operand-edge position exhausted"))?;
        let edge = self
            .operand_edges
            .iter()
            .find(|edge| {
                edge.parent == parent
                    && edge.child == child
                    && edge.position == position
                    && edge.role == role
            })
            .ok_or_else(|| planner_error("dead source edge has no planned disposition"))?;
        let identity = BoundaryUseIdentity::Source {
            parent: edge.parent,
            child: edge.child,
            position: edge.position,
        };
        self.record_boundary_use_disposition(identity)
    }

    pub(in crate::cranelift_backend) fn close_reached_operand_edge(
        &self,
        parent: StaticOriginId,
        position: usize,
        role: SourceOperandRole,
    ) -> Result<OperandEdgeDisposition, CraneliftBackendError> {
        self.reached_operand_edge_token(parent, position, role)
            .map(|token| token.disposition())
    }

    /// Return one exact source-occurrence token across repeated lowering
    /// specializations, recording the source identity only on its first reach.
    ///
    /// This does not collapse emitted lowering-only transitions: source
    /// identities name semantic syntax occurrences, and one occurrence can be
    /// revisited while staging more than one generated unit.
    pub(in crate::cranelift_backend) fn reached_operand_edge_token(
        &self,
        parent: StaticOriginId,
        position: usize,
        role: SourceOperandRole,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let child = self.semantic.child_origin(parent, position)?;
        let position = u32::try_from(position)
            .map_err(|_| planner_capacity_error("operand-edge position exhausted"))?;
        let edge = self
            .operand_edges
            .iter()
            .find(|edge| {
                edge.parent == parent
                    && edge.child == child
                    && edge.position == position
                    && edge.role == role
            })
            .ok_or_else(|| planner_error("reached source edge has no planned disposition"))?;
        let identity = BoundaryUseIdentity::Source {
            parent: edge.parent,
            child: edge.child,
            position: edge.position,
        };
        let token =
            self.planned_boundary_use_token(identity, source_operand_role_label(edge.role))?;
        if !self
            .operand_edge_consumption
            .borrow()
            .contains_key(&identity)
        {
            self.record_boundary_use_consumption(identity)?;
        }
        Ok(token)
    }

    pub(in crate::cranelift_backend) fn disposition_boundary_uses_in_owner_subtree(
        &self,
        root: StaticOriginId,
    ) -> Result<(), CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(root)?
            .ok_or_else(|| planner_error("dead source subtree has no function owner"))?;
        let mut pending = vec![root];
        let mut origins = BTreeSet::new();
        while let Some(origin) = pending.pop() {
            if !origins.insert(origin) {
                continue;
            }
            for child in self.semantic.child_origins(origin)?.iter().copied() {
                if self.semantic.function_owner(child)? == Some(owner) {
                    pending.push(child);
                }
            }
        }
        let dead_worker_bodies = self
            .static_recursor_worker_residuals
            .iter()
            .filter(|residual| origins.contains(&residual.producer_origin))
            .map(|residual| residual.body_origin)
            .collect::<BTreeSet<_>>();
        let mut dead_worker_origins = BTreeMap::new();
        for body in &dead_worker_bodies {
            let worker_owner = self
                .semantic
                .function_owner(*body)?
                .ok_or_else(|| planner_error("dead static worker body has no function owner"))?;
            let mut worker_pending = vec![*body];
            while let Some(origin) = worker_pending.pop() {
                if dead_worker_origins.insert(origin, worker_owner).is_some() {
                    continue;
                }
                for child in self.semantic.child_origins(origin)?.iter().copied() {
                    if self.semantic.function_owner(child)? == Some(worker_owner) {
                        worker_pending.push(child);
                    }
                }
            }
        }
        let consumption = self.operand_edge_consumption.borrow();
        let identities = self
            .boundary_uses
            .iter()
            .filter_map(|planned| {
                let (origin, exact_static_worker_path) = match &planned.path {
                    PlannedBoundaryUsePath::Source { parent, .. } => (*parent, false),
                    PlannedBoundaryUsePath::Synthesized { origin, .. } => (*origin, false),
                    PlannedBoundaryUsePath::StaticRecursorWorker {
                        producer_origin, ..
                    } => (*producer_origin, true),
                    PlannedBoundaryUsePath::StaticRecursorCapture {
                        producer_origin, ..
                    } => (*producer_origin, true),
                };
                let dead_worker_call_input = dead_worker_origins.contains_key(&origin)
                    && self.lowering_boundary_uses.iter().any(|lowering| {
                        lowering.identity == planned.identity
                            && lowering.edge == LoweringOnlyOperandEdge::CallableCapsuleEscape
                            && lowering.position != u32::MAX
                    });
                let exact_static_worker = exact_static_worker_path || dead_worker_call_input;
                let in_dead_population = (planned.producer_owner == owner
                    && origins.contains(&origin))
                    || dead_worker_origins
                        .get(&origin)
                        .is_some_and(|worker_owner| {
                            exact_static_worker
                                && (*worker_owner == planned.producer_owner
                                    || dead_worker_call_input)
                        });
                (in_dead_population
                    && (exact_static_worker || !consumption.contains_key(&planned.identity)))
                .then_some(planned.identity)
            })
            .collect::<Vec<_>>();
        drop(consumption);
        for identity in identities {
            self.record_boundary_use_disposition(identity)?;
        }
        let aggregate_keys = self
            .aggregate_representations
            .iter()
            .filter(|record| {
                record.phase == ResultPhase::CarrierRequired
                    && record.owner == owner
                    && origins.contains(&record.origin)
            })
            .map(|record| (record.owner, record.origin))
            .collect::<Vec<_>>();
        let consumption = self.aggregate_representation_consumption.borrow();
        if let Some(key) = aggregate_keys
            .iter()
            .find(|key| consumption.contains_key(key))
        {
            return Err(planner_error(format!(
                "one aggregate occurrence was both emitted and statically dispositioned: \
                 {key:?}"
            )));
        }
        drop(consumption);
        self.aggregate_representation_dispositions
            .borrow_mut()
            .extend(aggregate_keys);
        Ok(())
    }

    pub(in crate::cranelift_backend) fn disposition_lowering_boundary_use_if_planned(
        &self,
        edge: LoweringOnlyOperandEdge,
        origin: StaticOriginId,
        position: u32,
    ) -> Result<(), CraneliftBackendError> {
        let Some(planned) = self.lowering_boundary_uses.iter().find(|planned| {
            planned.edge == edge && planned.origin == origin && planned.position == position
        }) else {
            return Ok(());
        };
        if self
            .operand_edge_consumption
            .borrow()
            .contains_key(&planned.identity)
        {
            return Ok(());
        }
        self.record_boundary_use_disposition(planned.identity)
    }

    fn record_boundary_use_disposition(
        &self,
        identity: BoundaryUseIdentity,
    ) -> Result<(), CraneliftBackendError> {
        if !self
            .boundary_uses
            .iter()
            .any(|planned| planned.identity == identity)
        {
            return Err(planner_error(
                "dead boundary disposition names no exact planned use",
            ));
        }
        if self
            .operand_edge_consumption
            .borrow()
            .contains_key(&identity)
        {
            let planned = self
                .boundary_uses
                .iter()
                .find(|planned| planned.identity == identity);
            return Err(planner_error(format!(
                "one boundary use was both emitted and statically dispositioned: \
                 {identity:?}; planned={planned:?}"
            )));
        }
        self.boundary_use_dispositions.borrow_mut().insert(identity);
        Ok(())
    }

    /// Close one generated function's source-boundary ledger before its
    /// definition is published to the object module.
    pub(in crate::cranelift_backend) fn validate_boundary_use_consumption_for_owner(
        &self,
        owner: PredeclaredFunctionId,
    ) -> Result<(), CraneliftBackendError> {
        let dispositions = self.boundary_use_dispositions.borrow();
        let expected = self
            .operand_edges
            .iter()
            .filter(|edge| edge.owner == owner)
            .map(|edge| BoundaryUseIdentity::Source {
                parent: edge.parent,
                child: edge.child,
                position: edge.position,
            })
            .filter(|identity| !dispositions.contains(identity))
            .collect::<BTreeSet<_>>();
        let ledger = self.operand_edge_consumption.borrow();
        let actual = ledger
            .iter()
            .filter_map(|(identity, count)| match identity {
                BoundaryUseIdentity::Source { parent, .. }
                    if self
                        .semantic
                        .function_owner(*parent)
                        .is_ok_and(|candidate| candidate == Some(owner)) =>
                {
                    (!dispositions.contains(identity)).then_some((*identity, *count))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let duplicates = actual
            .iter()
            .filter(|(_, count)| **count != 1)
            .map(|(identity, count)| (*identity, *count))
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            return Err(planner_error(format!(
                "source boundary-use ledger contains duplicate consumption; \
                 duplicates={duplicates:?}"
            )));
        }
        let actual_set = actual.keys().copied().collect::<BTreeSet<_>>();
        if actual_set != expected {
            let missing = expected
                .difference(&actual_set)
                .copied()
                .collect::<Vec<_>>();
            let extra = actual_set
                .difference(&expected)
                .copied()
                .collect::<Vec<_>>();
            let missing_edges = self
                .operand_edges
                .iter()
                .filter(|edge| {
                    missing.contains(&BoundaryUseIdentity::Source {
                        parent: edge.parent,
                        child: edge.child,
                        position: edge.position,
                    })
                })
                .collect::<Vec<_>>();
            return Err(planner_error(format!(
                "source boundary-use ledger is not exact; missing={missing:?}; \
                 missing_edges={missing_edges:?}; extra={extra:?}"
            )));
        }
        Ok(())
    }

    pub(in crate::cranelift_backend) fn validate_boundary_use_consumption(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        self.validate_case_emission_consumption()?;
        self.validate_aggregate_representation_consumption()?;
        self.validate_synthesized_aggregate_representation_consumption()?;
        let owners = self
            .operand_edges
            .iter()
            .map(|edge| edge.owner)
            .collect::<BTreeSet<_>>();
        for owner in owners {
            self.validate_boundary_use_consumption_for_owner(owner)?;
        }
        let dispositions = self.boundary_use_dispositions.borrow();
        let expected = self
            .boundary_uses
            .iter()
            .map(|edge| edge.identity)
            .filter(|identity| !dispositions.contains(identity))
            .collect::<BTreeSet<_>>();
        let ledger = self.operand_edge_consumption.borrow();
        let duplicates = ledger
            .iter()
            .filter(|(_, count)| **count != 1)
            .map(|(identity, count)| (*identity, *count))
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            let duplicate_uses = self
                .boundary_uses
                .iter()
                .filter(|planned| {
                    duplicates
                        .iter()
                        .any(|(identity, _)| *identity == planned.identity)
                })
                .collect::<Vec<_>>();
            let duplicate_lowering_uses = self
                .lowering_boundary_uses
                .iter()
                .filter(|planned| {
                    duplicates
                        .iter()
                        .any(|(identity, _)| *identity == planned.identity)
                })
                .collect::<Vec<_>>();
            return Err(planner_error(format!(
                "boundary-use ledger contains duplicate consumption; \
                 duplicates={duplicates:?}; duplicate_uses={duplicate_uses:?}; \
                 duplicate_lowering_uses={duplicate_lowering_uses:?}"
            )));
        }
        let actual = ledger
            .keys()
            .filter(|identity| !dispositions.contains(identity))
            .copied()
            .collect::<BTreeSet<_>>();
        if actual != expected {
            let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
            let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
            let missing_uses = self
                .boundary_uses
                .iter()
                .filter(|planned| missing.contains(&planned.identity))
                .collect::<Vec<_>>();
            let extra_uses = self
                .boundary_uses
                .iter()
                .filter(|planned| extra.contains(&planned.identity))
                .collect::<Vec<_>>();
            let missing_lowering_uses = self
                .lowering_boundary_uses
                .iter()
                .filter(|planned| missing.contains(&planned.identity))
                .collect::<Vec<_>>();
            return Err(planner_error(format!(
                "boundary-use ledger is not exact; missing={missing:?}; \
                 missing_uses={missing_uses:?}; \
                 missing_lowering_uses={missing_lowering_uses:?}; extra={extra:?}; \
                 extra_uses={extra_uses:?}"
            )));
        }
        let emitted_and_dispositioned = ledger
            .keys()
            .filter(|identity| {
                matches!(identity, BoundaryUseIdentity::Synthesized(_))
                    && dispositions.contains(identity)
            })
            .copied()
            .collect::<Vec<_>>();
        if !emitted_and_dispositioned.is_empty() {
            return Err(planner_error(format!(
                "boundary uses were both emitted and statically dispositioned; \
                 identities={emitted_and_dispositioned:?}"
            )));
        }
        Ok(())
    }

    fn validate_case_emission_consumption(&self) -> Result<(), CraneliftBackendError> {
        let ledger = self.case_emission_consumption.borrow();
        let emitted_matches = ledger
            .keys()
            .map(|(owner, match_origin, _)| (*owner, *match_origin))
            .collect::<BTreeSet<_>>();
        let expected = self
            .case_emissions
            .iter()
            .filter(|record| {
                record.status == CaseEmissionStatus::Reachable
                    && emitted_matches.contains(&(record.owner, record.match_origin))
            })
            .map(|record| (record.owner, record.match_origin, record.ordinal))
            .collect::<BTreeSet<_>>();
        let duplicates = ledger
            .iter()
            .filter(|(_, count)| **count != 1)
            .map(|(identity, count)| (*identity, *count))
            .collect::<Vec<_>>();
        if !duplicates.is_empty() {
            return Err(planner_error(format!(
                "case-emission ledger contains duplicate consumption; \
                 duplicates={duplicates:?}"
            )));
        }
        let actual = ledger.keys().copied().collect::<BTreeSet<_>>();
        if actual != expected {
            let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
            let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
            return Err(planner_error(format!(
                "case-emission ledger is not exact; missing={missing:?}; extra={extra:?}"
            )));
        }
        drop(ledger);
        let inactive_arms = self
            .case_emissions
            .iter()
            .filter(|record| !emitted_matches.contains(&(record.owner, record.match_origin)))
            .map(|record| BoundaryUseIdentity::Source {
                parent: record.match_origin,
                child: record.body_origin,
                position: 1 + record.ordinal,
            })
            .collect::<BTreeSet<_>>();
        for identity in inactive_arms {
            if self
                .operand_edge_consumption
                .borrow()
                .contains_key(&identity)
                || self.boundary_use_dispositions.borrow().contains(&identity)
            {
                continue;
            }
            self.record_boundary_use_disposition(identity)?;
        }
        Ok(())
    }

    fn validate_aggregate_representation_consumption(&self) -> Result<(), CraneliftBackendError> {
        let expected = self
            .aggregate_representations
            .iter()
            .filter(|record| record.phase == ResultPhase::CarrierRequired)
            .map(|record| (record.owner, record.origin))
            .collect::<BTreeSet<_>>();
        let ledger = self.aggregate_representation_consumption.borrow();
        let dispositions = self.aggregate_representation_dispositions.borrow();
        let overlap = ledger
            .keys()
            .filter(|key| dispositions.contains(key))
            .copied()
            .collect::<Vec<_>>();
        if !overlap.is_empty() {
            return Err(planner_error(format!(
                "aggregate representation ledger overlaps static dispositions; \
                 overlap={overlap:?}"
            )));
        }
        // The key is one semantic aggregate occurrence in one owner. Lowering
        // can visit that occurrence in more than one mutually exclusive
        // source-machine clone, but those visits consume the same planned
        // representation crossing rather than inventing additional
        // occurrences. Exactness therefore closes over the semantic key set;
        // duplicate planned records are rejected independently when the plan
        // is validated.
        let actual = ledger
            .keys()
            .copied()
            .chain(dispositions.iter().copied())
            .collect::<BTreeSet<_>>();
        if actual != expected {
            let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
            let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
            let missing_expressions = missing
                .iter()
                .map(|(_, origin)| {
                    (
                        *origin,
                        self.source_occurrence(*origin)
                            .map(|expr| format!("{expr:?}"))
                            .unwrap_or_else(|_| "<missing>".to_string()),
                        self.result_phases
                            .get(origin.0 as usize)
                            .and_then(Option::as_ref)
                            .map(|summary| summary.phase),
                        self.operand_edges
                            .iter()
                            .filter(|edge| edge.child == *origin)
                            .map(|edge| {
                                (
                                    edge.parent,
                                    self.source_occurrence(edge.parent)
                                        .map(|expr| format!("{expr:?}"))
                                        .unwrap_or_else(|_| "<missing>".to_string()),
                                    edge.position,
                                    edge.disposition,
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>();
            return Err(planner_error(format!(
                "aggregate representation ledger is not exact; missing={missing:?}; \
                 extra={extra:?}; missing_expressions={missing_expressions:?}"
            )));
        }
        Ok(())
    }

    fn validate_synthesized_aggregate_representation_consumption(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        let expected = self
            .synthesized_aggregate_representations
            .iter()
            .filter(|record| record.phase == ResultPhase::CarrierRequired)
            .map(|record| SynthesizedAggregateOccurrence {
                effect_origin: record.effect_origin,
                owner: record.owner,
                site: record.site,
            })
            .collect::<BTreeSet<_>>();
        let ledger = self
            .synthesized_aggregate_representation_consumption
            .borrow();
        let actual = ledger.keys().copied().collect::<BTreeSet<_>>();
        if actual != expected {
            let missing = expected.difference(&actual).copied().collect::<Vec<_>>();
            let extra = actual.difference(&expected).copied().collect::<Vec<_>>();
            return Err(planner_error(format!(
                "synthesized aggregate ledger is not exact; missing={missing:?}; \
                 extra={extra:?}"
            )));
        }
        Ok(())
    }

    fn planned_boundary_use_token(
        &self,
        identity: BoundaryUseIdentity,
        label: &'static str,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let planned = self
            .boundary_uses
            .iter()
            .find(|planned| planned.identity == identity)
            .ok_or_else(|| planner_error("boundary transition has no exact unified planned use"))?;
        Ok(OperandEdgeToken {
            disposition: planned.disposition,
            label,
            identity: planned.identity,
            producer_owner: Some(planned.producer_owner),
            consumer_owner: Some(planned.consumer_owner),
            producer_phase: planned.producer_phase,
            consumer_phase: planned.consumer_phase,
            operation: planned.operation,
            need: planned.need,
            avail: planned.avail,
            effect_operation: match &planned.path {
                PlannedBoundaryUsePath::Source {
                    effect_operation, ..
                } => *effect_operation,
                _ => None,
            },
            effect_seat: match &planned.path {
                PlannedBoundaryUsePath::Source { effect_seat, .. } => *effect_seat,
                _ => None,
            },
        })
    }

    pub(in crate::cranelift_backend) fn lowering_boundary_use_token(
        &self,
        edge: LoweringOnlyOperandEdge,
        origin: StaticOriginId,
        position: u32,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(origin)?
            .ok_or_else(|| planner_error("lowering boundary use has no function owner"))?;
        self.lowering_boundary_use_token_for_owner(edge, origin, position, owner)
    }

    /// Re-enter one lowering transition attached to a source occurrence.
    ///
    /// Source-machine recursion can revisit an already emitted semantic
    /// occurrence while building one generated function. The first visit
    /// consumes the planner-issued identity. A later visit reborrows that same
    /// authority; it is not a second planned population member. Callers must
    /// choose this API explicitly—ordinary lowering-only transitions continue
    /// to count every request and therefore red on duplicate consumption.
    pub(in crate::cranelift_backend) fn reached_lowering_boundary_use_token(
        &self,
        edge: LoweringOnlyOperandEdge,
        origin: StaticOriginId,
        position: u32,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(origin)?
            .ok_or_else(|| planner_error("lowering boundary use has no function owner"))?;
        let ledger = self.operand_edge_consumption.borrow();
        let planned = self
            .lowering_boundary_uses
            .iter()
            .find(|planned| {
                planned.edge == edge
                    && planned.origin == origin
                    && planned.position == position
                    && planned.owner == owner
                    && !ledger.contains_key(&planned.identity)
            })
            .or_else(|| {
                self.lowering_boundary_uses.iter().find(|planned| {
                    planned.edge == edge
                        && planned.origin == origin
                        && planned.position == position
                        && planned.owner == owner
                })
            })
            .ok_or_else(|| {
                planner_error(
                    "reached lowering transition has no exact planner-issued boundary use",
                )
            })?;
        let already_consumed = ledger.contains_key(&planned.identity);
        let identity = planned.identity;
        let label = planned.edge.label();
        drop(ledger);
        if already_consumed {
            self.planned_boundary_use_token(identity, label)
        } else {
            self.lowering_boundary_use_token_for_owner(edge, origin, position, owner)
        }
    }

    pub(in crate::cranelift_backend) fn lowering_boundary_use_token_for_owner(
        &self,
        edge: LoweringOnlyOperandEdge,
        origin: StaticOriginId,
        position: u32,
        owner: PredeclaredFunctionId,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        #[cfg(test)]
        if DENY_LOWERING_BOUNDARY_USE_ISSUANCE.with(Cell::get) {
            return Err(planner_error(
                "test mutation denied planner-issued lowering boundary use",
            ));
        }
        let ledger = self.operand_edge_consumption.borrow();
        let planned = self
            .lowering_boundary_uses
            .iter()
            .find(|planned| {
                planned.edge == edge
                    && planned.origin == origin
                    && planned.position == position
                    && planned.owner == owner
                    && !ledger.contains_key(&planned.identity)
            })
            .or_else(|| {
                self.lowering_boundary_uses.iter().find(|planned| {
                    planned.edge == edge
                        && planned.origin == origin
                        && planned.position == position
                        && planned.owner == owner
                })
            })
            .ok_or_else(|| {
                let source = self
                    .source_occurrences
                    .get(origin.0 as usize)
                    .and_then(Option::as_ref)
                    .map(|occurrence| occurrence.expr);
                planner_error(format!(
                    "lowering transition has no exact planner-issued boundary use: \
                     edge={edge:?}, origin={origin:?}, position={position}, owner={owner:?}, \
                     source={source:?}"
                ))
            })?;
        let identity = planned.identity;
        let label = planned.edge.label();
        drop(ledger);
        let token = self.planned_boundary_use_token(identity, label)?;
        #[cfg(test)]
        {
            let mutation = SYNTHESIZED_CONSUMPTION_MUTATION.with(Cell::get);
            let apply = mutation != SynthesizedConsumptionMutation::Exact
                && SYNTHESIZED_CONSUMPTION_MUTATED.with(|cell| {
                    if cell.get() {
                        false
                    } else {
                        cell.set(true);
                        true
                    }
                });
            if apply {
                match mutation {
                    SynthesizedConsumptionMutation::OmitFirst => return Ok(token),
                    SynthesizedConsumptionMutation::RepeatFirst => {
                        self.record_boundary_use_consumption(identity)?;
                    }
                    SynthesizedConsumptionMutation::Exact => {}
                }
            }
        }
        self.record_boundary_use_consumption(identity)?;
        Ok(token)
    }

    pub(in crate::cranelift_backend) fn recursor_boundary_use_token(
        &self,
        parent_origin: StaticOriginId,
        sibling_position: usize,
    ) -> Result<OperandEdgeToken, CraneliftBackendError> {
        let sibling_position = u32::try_from(sibling_position)
            .map_err(|_| planner_capacity_error("recursor boundary position exhausted"))?;
        let edge = self
            .recursor_boundary_uses
            .iter()
            .find(|edge| {
                edge.parent_origin == parent_origin && edge.sibling_position == sibling_position
            })
            .ok_or_else(|| {
                planner_error("computational recursor use has no planned boundary edge")
            })?;
        let token = self.planned_boundary_use_token(
            edge.identity,
            "a planned computational recursor residual",
        )?;
        // This is a semantic source crossing. The same recursive-position
        // authority may be revisited while staging multiple generated units,
        // but it is one planned use rather than repeated emitted lowering.
        if !self
            .operand_edge_consumption
            .borrow()
            .contains_key(&edge.identity)
        {
            self.record_boundary_use_consumption(edge.identity)?;
        }
        Ok(token)
    }

    /// Consume the planner-owned callable-capture disposition for one exact
    /// computational-recursor worker residual.
    pub(in crate::cranelift_backend) fn static_recursor_worker_residual_token(
        &self,
        parent_origin: StaticOriginId,
        sibling_position: usize,
        body_origin: StaticOriginId,
    ) -> Result<Option<StaticRecursorWorkerResidualToken>, CraneliftBackendError> {
        let sibling_position = u32::try_from(sibling_position)
            .map_err(|_| planner_capacity_error("static recursor sibling exhausted"))?;
        let Some(residual) = self
            .static_recursor_worker_residuals
            .iter()
            .find(|residual| {
                residual.parent_origin == parent_origin
                    && residual.sibling_position == sibling_position
                    && residual.body_origin == body_origin
            })
        else {
            return Ok(None);
        };
        self.issue_static_recursor_worker_residual_token(residual)
            .map(Some)
    }

    /// Recover the exact recursor-owned residual edge from its closure source
    /// occurrence.
    ///
    /// This reverse projection is used only while constructing the source
    /// constructor that owns the recursive position. It follows the same
    /// planned parent/scrutinee/position relation as the forward token lookup;
    /// a closure that is not such a child has no exception to whole-capsule
    /// rejection.
    pub(in crate::cranelift_backend) fn static_recursor_worker_residual_token_for_closure(
        &self,
        closure_origin: StaticOriginId,
    ) -> Result<Option<StaticRecursorWorkerResidualToken>, CraneliftBackendError> {
        let Some(residual) = self
            .static_recursor_worker_residuals
            .iter()
            .find(|residual| residual.closure_origin == closure_origin)
        else {
            return Ok(None);
        };
        self.issue_static_recursor_worker_residual_token(residual)
            .map(Some)
    }

    /// Recover the one exact static worker left live by causal source-result
    /// selection for a reached computational-recursion position.
    ///
    /// The constructor-side closure lookup owns ledger consumption. This
    /// consumer-side lookup returns the same planner identity without recording
    /// a second emission. A statically dead result subtree has already
    /// dispositioned its identity, so two same-shape producers remain distinct
    /// here without placing callable identity in the runtime carrier.
    pub(in crate::cranelift_backend) fn selected_static_recursor_worker_residual_token(
        &self,
        parent_origin: StaticOriginId,
        sibling_position: usize,
    ) -> Result<Option<StaticRecursorWorkerResidualToken>, CraneliftBackendError> {
        let sibling_position = u32::try_from(sibling_position)
            .map_err(|_| planner_capacity_error("static recursor sibling exhausted"))?;
        let candidates = self
            .static_recursor_worker_residuals
            .iter()
            .filter(|residual| {
                residual.parent_origin == parent_origin
                    && residual.sibling_position == sibling_position
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return Ok(None);
        }
        let dispositions = self.boundary_use_dispositions.borrow();
        let mut selected = Vec::new();
        for residual in candidates {
            let planned = self
                .boundary_uses
                .iter()
                .find(|planned| {
                    matches!(
                        &planned.path,
                        PlannedBoundaryUsePath::StaticRecursorWorker {
                            parent_origin: planned_parent,
                            producer_origin,
                            sibling_position: planned_position,
                            closure_origin,
                            body_origin,
                            declared_arity,
                            captures,
                        } if *planned_parent == residual.parent_origin
                            && *producer_origin == residual.producer_origin
                            && *planned_position == residual.sibling_position
                            && *closure_origin == residual.closure_origin
                            && *body_origin == residual.body_origin
                            && *declared_arity == residual.declared_arity
                            && captures == &residual.captures
                    )
                })
                .ok_or_else(|| {
                    planner_error(
                        "static recursor worker residual has no exact unified boundary use",
                    )
                })?;
            if !dispositions.contains(&planned.identity) {
                selected.push((residual, planned.identity));
            }
        }
        drop(dispositions);
        let [(residual, identity)] = selected.as_slice() else {
            return Err(planner_error(
                "reached static recursor position does not select exactly one result worker",
            ));
        };
        self.static_recursor_worker_residual_token_value(residual, *identity)
            .map(Some)
    }

    fn issue_static_recursor_worker_residual_token(
        &self,
        residual: &PlannedStaticRecursorWorkerResidual,
    ) -> Result<StaticRecursorWorkerResidualToken, CraneliftBackendError> {
        let planned = self
            .boundary_uses
            .iter()
            .find(|planned| {
                matches!(
                    &planned.path,
                    PlannedBoundaryUsePath::StaticRecursorWorker {
                        parent_origin,
                        producer_origin,
                        sibling_position,
                        closure_origin,
                        body_origin,
                        declared_arity,
                        captures,
                    } if *parent_origin == residual.parent_origin
                        && *producer_origin == residual.producer_origin
                        && *sibling_position == residual.sibling_position
                        && *closure_origin == residual.closure_origin
                        && *body_origin == residual.body_origin
                        && *declared_arity == residual.declared_arity
                        && captures == &residual.captures
                )
            })
            .ok_or_else(|| {
                planner_error("static recursor worker residual has no exact unified boundary use")
            })?;
        if planned.disposition != residual.disposition {
            return Err(planner_error(
                "static recursor worker residual disagrees with its unified disposition",
            ));
        }
        #[cfg(test)]
        {
            let mutation = STATIC_RECURSOR_CONSUMPTION_MUTATION.with(Cell::get);
            let apply = mutation != StaticRecursorConsumptionMutation::Exact
                && STATIC_RECURSOR_CONSUMPTION_MUTATED.with(|cell| {
                    if cell.get() {
                        false
                    } else {
                        cell.set(true);
                        true
                    }
                });
            if apply {
                match mutation {
                    StaticRecursorConsumptionMutation::OmitFirst => {
                        return self.static_recursor_worker_residual_token_value(
                            residual,
                            planned.identity,
                        );
                    }
                    StaticRecursorConsumptionMutation::RepeatFirst => {
                        self.record_boundary_use_consumption(planned.identity)?;
                    }
                    StaticRecursorConsumptionMutation::Exact => {}
                }
            }
        }
        self.record_boundary_use_consumption(planned.identity)?;
        self.static_recursor_worker_residual_token_value(residual, planned.identity)
    }

    fn static_recursor_worker_residual_token_value(
        &self,
        residual: &PlannedStaticRecursorWorkerResidual,
        identity: BoundaryUseIdentity,
    ) -> Result<StaticRecursorWorkerResidualToken, CraneliftBackendError> {
        Ok(StaticRecursorWorkerResidualToken {
            identity,
            id: residual.id,
            parent_origin: residual.parent_origin,
            producer_origin: residual.producer_origin,
            sibling_position: residual.sibling_position,
            closure_origin: residual.closure_origin,
            body_origin: residual.body_origin,
            declared_arity: residual.declared_arity,
            capture_count: u32::try_from(residual.captures.len())
                .map_err(|_| planner_capacity_error("static recursor capture count exhausted"))?,
            disposition: residual.disposition,
        })
    }

    pub(in crate::cranelift_backend) fn validate_static_recursor_worker_residual_identity(
        &self,
        identity: BoundaryUseIdentity,
        id: StaticRecursorWorkerResidualId,
        parent_origin: StaticOriginId,
        producer_origin: StaticOriginId,
        sibling_position: usize,
        closure_origin: StaticOriginId,
        body_origin: StaticOriginId,
        declared_arity: usize,
        capture_count: usize,
    ) -> Result<(), CraneliftBackendError> {
        let sibling_position = u32::try_from(sibling_position)
            .map_err(|_| planner_capacity_error("static recursor sibling exhausted"))?;
        let declared_arity = u32::try_from(declared_arity)
            .map_err(|_| planner_capacity_error("static recursor arity exhausted"))?;
        let residual = self
            .static_recursor_worker_residuals
            .iter()
            .find(|residual| {
                residual.id == id
                    && residual.parent_origin == parent_origin
                    && residual.producer_origin == producer_origin
                    && residual.sibling_position == sibling_position
                    && residual.closure_origin == closure_origin
                    && residual.body_origin == body_origin
                    && residual.declared_arity == declared_arity
                    && residual.captures.len() == capture_count
                    && residual.disposition == OperandEdgeDisposition::CallableCapture
            })
            .ok_or_else(|| {
                planner_error("static recursor worker metadata has no exact planned residual")
            })?;
        let planned = self
            .boundary_uses
            .iter()
            .find(|planned| planned.identity == identity)
            .ok_or_else(|| {
                planner_error("static recursor worker metadata has no exact unified identity")
            })?;
        let exact = matches!(
            &planned.path,
            PlannedBoundaryUsePath::StaticRecursorWorker {
                parent_origin: planned_parent,
                producer_origin: planned_producer,
                sibling_position: planned_position,
                closure_origin: planned_closure,
                body_origin: planned_body,
                declared_arity: planned_arity,
                captures,
            } if *planned_parent == residual.parent_origin
                && *planned_producer == residual.producer_origin
                && *planned_position == residual.sibling_position
                && *planned_closure == residual.closure_origin
                && *planned_body == residual.body_origin
                && *planned_arity == residual.declared_arity
                && captures == &residual.captures
        );
        if !exact || planned.disposition != OperandEdgeDisposition::CallableCapture {
            return Err(planner_error(
                "static recursor worker metadata disagrees with its unified authority",
            ));
        }
        Ok(())
    }

    /// Consume the exact one-way producer authority for one ordered capture of
    /// one already-validated static worker.
    pub(in crate::cranelift_backend) fn static_recursor_capture_token(
        &self,
        worker_identity: BoundaryUseIdentity,
        residual_id: StaticRecursorWorkerResidualId,
        parent_origin: StaticOriginId,
        producer_origin: StaticOriginId,
        sibling_position: usize,
        closure_origin: StaticOriginId,
        ordinal: usize,
    ) -> Result<StaticRecursorCaptureToken, CraneliftBackendError> {
        let sibling_position = u32::try_from(sibling_position)
            .map_err(|_| planner_capacity_error("static recursor sibling exhausted"))?;
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| planner_capacity_error("static recursor capture ordinal exhausted"))?;
        let planned = self
            .boundary_uses
            .iter()
            .find(|planned| {
                matches!(
                    &planned.path,
                    PlannedBoundaryUsePath::StaticRecursorCapture {
                        worker_identity: planned_worker,
                        residual_id: planned_residual,
                        parent_origin: planned_parent,
                        producer_origin: planned_producer,
                        sibling_position: planned_position,
                        closure_origin: planned_closure,
                        ordinal: planned_ordinal,
                        capture,
                    } if *planned_worker == worker_identity
                        && *planned_residual == residual_id
                        && *planned_parent == parent_origin
                        && *planned_producer == producer_origin
                        && *planned_position == sibling_position
                        && *planned_closure == closure_origin
                        && *planned_ordinal == ordinal
                        && capture.ordinal == ordinal
                        && capture.closure_origin == closure_origin
                        && capture.phase == OperandEdgeDisposition::CallableCapture
                )
            })
            .ok_or_else(|| {
                planner_error("static recursor capture has no exact planned producer authority")
            })?;
        let capture = match &planned.path {
            PlannedBoundaryUsePath::StaticRecursorCapture { capture, .. } => capture,
            _ => unreachable!("selected static recursor capture path"),
        };
        let edge =
            self.planned_boundary_use_token(planned.identity, "a static recursor capture")?;
        self.record_boundary_use_consumption(planned.identity)?;
        Ok(StaticRecursorCaptureToken {
            ordinal,
            owner: capture.owner,
            closure_origin: capture.closure_origin,
            source_origin: match capture.source {
                StaticRecursorCaptureSource::Seed(_) => capture.closure_origin,
                StaticRecursorCaptureSource::Lexical(origin) => origin,
            },
            phase: capture.phase,
            lifetime: StaticRecursorCaptureLifetime::ActivationOwned,
            edge,
        })
    }

    /// Consume the planner-owned result contract for one source join.
    ///
    /// The token is keyed only by the opaque origin. Diagnostic labels and
    /// lowered values do not participate in selection.
    pub(in crate::cranelift_backend) fn join_plan_token(
        &self,
        origin: StaticOriginId,
    ) -> Result<JoinPlanToken, CraneliftBackendError> {
        self.join_plan_token_if_planned(origin)?
            .ok_or_else(|| planner_error("static origin has no planned source join"))
    }

    /// Project the authoritative join population onto one traversal entry.
    ///
    /// `None` means this validated source occurrence is not a join. Lowering
    /// therefore never maintains a second spelling inventory of join forms.
    pub(in crate::cranelift_backend) fn join_plan_token_if_planned(
        &self,
        origin: StaticOriginId,
    ) -> Result<Option<JoinPlanToken>, CraneliftBackendError> {
        let planned = self
            .join_results
            .get(origin.0 as usize)
            .ok_or_else(|| planner_error("static origin is outside the join result plan"))?;
        Ok(planned.map(|planned| JoinPlanToken {
            origin,
            representation: planned.representation,
            has_continuing_predecessor: planned.has_continuing_predecessor,
        }))
    }

    /// Every source-join contract owned by one generated function.
    ///
    /// This is a projection of the already-validated occurrence population and
    /// semantic owner partition. Lowering uses it only as the closed expected
    /// set for its end-of-function consumption check; it cannot add or omit a
    /// join by maintaining a second caller inventory.
    pub(in crate::cranelift_backend) fn required_join_origins(
        &self,
        function: PredeclaredFunctionId,
    ) -> Result<BTreeSet<StaticOriginId>, CraneliftBackendError> {
        let source_function = self
            .static_callable_specializations
            .iter()
            .find(|specialization| specialization.function == function)
            .map_or(function, |specialization| specialization.body_function);
        let mut required = BTreeSet::new();
        for (index, (occurrence, join)) in self
            .source_occurrences
            .iter()
            .zip(&self.join_results)
            .enumerate()
        {
            let (Some(occurrence), Some(_)) = (occurrence, join) else {
                continue;
            };
            if occurrence.static_origin.0 as usize != index {
                return Err(planner_error(
                    "join consumption population is not keyed by source origin",
                ));
            }
            if self.semantic.function_owner(occurrence.static_origin)? == Some(source_function) {
                required.insert(occurrence.static_origin);
            }
        }
        Ok(required)
    }

    /// Planned joins in one source subtree that remain in its function owner.
    ///
    /// This is the structural population used when lowering proves that a
    /// branch is statically unselected. The traversal follows the semantic
    /// plane's validated positional-child inventory, never a second
    /// `RuntimeExpr` spelling list, and stops at declared-unit owner
    /// boundaries. A closure body in a dead outer branch is still validated
    /// when its own generated function is emitted.
    pub(in crate::cranelift_backend) fn source_join_origins_in_owner_subtree(
        &self,
        root: StaticOriginId,
    ) -> Result<BTreeSet<StaticOriginId>, CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(root)?
            .ok_or_else(|| planner_error("source subtree root has no function owner"))?;
        let mut pending = vec![root];
        let mut visited = BTreeSet::new();
        let mut joins = BTreeSet::new();
        while let Some(origin) = pending.pop() {
            if !visited.insert(origin) {
                continue;
            }
            if self.semantic.function_owner(origin)? != Some(owner) {
                continue;
            }
            let index = origin.0 as usize;
            let occurrence = self
                .source_occurrences
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(|| planner_error("source subtree names no planned occurrence"))?;
            if occurrence.static_origin != origin {
                return Err(planner_error(
                    "source subtree occurrence disagrees with its positional origin",
                ));
            }
            if is_source_join(occurrence.expr) {
                joins.insert(origin);
            }
            pending.extend(self.semantic.child_origins(origin)?.iter().copied());
        }
        Ok(joins)
    }

    /// Result-position source occurrences below one root in one function owner.
    ///
    /// This is the sole exhaustive inventory for source result flow. Lowering
    /// uses it to recognize terminal process results, while phase planning uses
    /// the same population when an enclosing computational eliminator changes
    /// the representation forwarded by a producer-local join.
    pub(in crate::cranelift_backend) fn source_result_origins_in_owner_subtree(
        &self,
        root: StaticOriginId,
    ) -> Result<BTreeSet<StaticOriginId>, CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(root)?
            .ok_or_else(|| planner_error("source result root has no function owner"))?;
        let mut pending = vec![root];
        let mut results = BTreeSet::new();
        while let Some(origin) = pending.pop() {
            if !results.insert(origin) {
                continue;
            }
            if self.semantic.function_owner(origin)? != Some(owner) {
                results.remove(&origin);
                continue;
            }
            let occurrence = self
                .source_occurrences
                .get(origin.0 as usize)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    planner_error("source result traversal names no planned occurrence")
                })?;
            if occurrence.static_origin != origin {
                return Err(planner_error(
                    "source result occurrence disagrees with its positional origin",
                ));
            }
            let expr = occurrence.expr;
            let child = |position| self.semantic.child_origin(origin, position);
            match expr {
                RuntimeExpr::CheckedJoinSite { .. }
                | RuntimeExpr::CheckedSubcontinuationFrame { .. }
                | RuntimeExpr::CheckedRecursiveInvocation { .. }
                | RuntimeExpr::CheckedComputationalIHSlots { .. }
                | RuntimeExpr::CheckedComputationalIHInvocation { .. } => {
                    pending.push(child(0)?);
                }
                RuntimeExpr::Let { .. } => pending.push(child(1)?),
                RuntimeExpr::If { .. } => {
                    pending.push(child(1)?);
                    pending.push(child(2)?);
                }
                RuntimeExpr::Match { cases, .. } => {
                    for index in 0..cases.len() {
                        pending.push(child(1 + index)?);
                    }
                }
                RuntimeExpr::ComputationalMatch { cases, .. } => {
                    for index in 0..cases.len() {
                        pending.push(child(1 + index)?);
                    }
                }
                RuntimeExpr::Value(_)
                | RuntimeExpr::Var(_)
                | RuntimeExpr::PrimitiveCall { .. }
                | RuntimeExpr::Construct { .. }
                | RuntimeExpr::Record { .. }
                | RuntimeExpr::Project { .. }
                | RuntimeExpr::Closure { .. }
                | RuntimeExpr::LexicalClosure { .. }
                | RuntimeExpr::DeclarationRef { .. }
                | RuntimeExpr::ImportedDeclarationRef { .. }
                | RuntimeExpr::Call { .. }
                | RuntimeExpr::Effect { .. }
                | RuntimeExpr::Trap(_) => {}
            }
        }
        Ok(results)
    }

    /// The case-body roots of a source `Match` occurrence.
    ///
    /// Both ordinary and computational matches have one scrutinee followed by
    /// their case bodies in the semantic plane. Static selection consumes this
    /// validated positional population directly; lowering does not supply or
    /// recount the source cases.
    pub(in crate::cranelift_backend) fn source_match_case_body_origins(
        &self,
        origin: StaticOriginId,
    ) -> Result<Vec<StaticOriginId>, CraneliftBackendError> {
        let occurrence = self
            .source_occurrences
            .get(origin.0 as usize)
            .and_then(Option::as_ref)
            .ok_or_else(|| planner_error("source match names no planned occurrence"))?;
        if occurrence.static_origin != origin
            || !matches!(
                occurrence.expr,
                RuntimeExpr::Match { .. } | RuntimeExpr::ComputationalMatch { .. }
            )
        {
            return Err(planner_error(
                "source match population was requested for a different occurrence kind",
            ));
        }
        let children = self.semantic.child_origins(origin)?;
        let Some((_scrutinee, case_bodies)) = children.split_first() else {
            return Err(planner_error(
                "source match occurrence has no validated scrutinee child",
            ));
        };
        Ok(case_bodies.to_vec())
    }

    /// The artifact-static constructor identity of one case of the `Match` /
    /// `ComputationalMatch` occurrence at `origin` (`D1`).
    ///
    /// ⭐ **This is the capability export, not the plane.** `SemanticPlane` and
    /// its `names` arena stay `pub(super)`; what crosses into
    /// `crate::cranelift_backend` is an occurrence-keyed *question* and an
    /// unmintable answer. That is `RT-FNSPLIT-B2E`'s surviving `R3` shape —
    /// *"expose the capability, not the plane internals"* — and it is why `D1`
    /// is not discharged by widening a field.
    ///
    /// ⭐ The returned identity **is** occurrence-independent: equal spellings
    /// intern to one canonical span, so a producer's identity for `Cons` and an
    /// eliminator's identity for `Cons` are the same value even at different
    /// occurrences. That is `D2`'s shared-authority property.
    ///
    /// ⚠ **Artifact-local.** The identity is stable within one artifact's plane
    /// and carries no cross-artifact meaning. ⛔ Do not persist or compare it
    /// across artifacts.
    pub(in crate::cranelift_backend) fn case_constructor_identity(
        &self,
        origin: StaticOriginId,
        case_index: usize,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        self.semantic.case_constructor_identity(origin, case_index)
    }

    pub(in crate::cranelift_backend) fn case_emission_token(
        &self,
        owner: PredeclaredFunctionId,
        match_origin: StaticOriginId,
        case_index: usize,
    ) -> Result<CaseEmissionToken, CraneliftBackendError> {
        let ordinal = u32::try_from(case_index)
            .map_err(|_| planner_capacity_error("case-emission ordinal exhausted"))?;
        let body_origin = self.semantic.child_origin(match_origin, 1 + case_index)?;
        let record = self
            .case_emissions
            .iter()
            .find(|record| {
                record.owner == owner
                    && record.match_origin == match_origin
                    && record.ordinal == ordinal
            })
            .ok_or_else(|| {
                let candidates = self
                    .case_emissions
                    .iter()
                    .filter(|record| {
                        record.match_origin == match_origin && record.ordinal == ordinal
                    })
                    .map(|record| record.owner)
                    .collect::<Vec<_>>();
                planner_error(format!(
                    "carried Match case has no planned emission record; owner={owner:?}; \
                     match={match_origin:?}; ordinal={ordinal}; candidates={candidates:?}"
                ))
            })?;
        if record.body_origin != body_origin
            || record.scrutinee_origin != self.semantic.child_origin(match_origin, 0)?
            || record.constructor
                != self
                    .semantic
                    .case_constructor_identity(match_origin, case_index)?
            || record.phase != ResultPhase::CarrierRequired
        {
            return Err(planner_error(
                "case-emission record disagrees with its exact source case",
            ));
        }
        if record.status == CaseEmissionStatus::Reachable {
            let key = (owner, match_origin, ordinal);
            let mut ledger = self.case_emission_consumption.borrow_mut();
            let count = ledger.entry(key).or_insert(0);
            *count = count
                .checked_add(1)
                .ok_or_else(|| planner_capacity_error("case-emission ledger exhausted"))?;
        }
        Ok(CaseEmissionToken {
            match_origin,
            ordinal,
            body_origin,
            status: record.status,
        })
    }

    pub(in crate::cranelift_backend) fn aggregate_representation_token(
        &self,
        origin: StaticOriginId,
        class: BoundaryClass,
        arity: usize,
    ) -> Result<AggregateRepresentationToken, CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(origin)?
            .ok_or_else(|| planner_error("aggregate occurrence has no function owner"))?;
        let arity = u32::try_from(arity)
            .map_err(|_| planner_capacity_error("aggregate arity exhausted"))?;
        let record = self
            .aggregate_representations
            .iter()
            .find(|record| record.owner == owner && record.origin == origin)
            .ok_or_else(|| {
                let expression = self
                    .source_occurrence(origin)
                    .map(|expr| format!("{expr:?}"))
                    .unwrap_or_else(|_| "<missing>".to_string());
                let candidates = self
                    .aggregate_representations
                    .iter()
                    .filter(|record| record.origin == origin)
                    .map(|record| record.owner)
                    .collect::<Vec<_>>();
                planner_error(format!(
                    "aggregate allocation has no exact representation record; \
                     owner={owner:?}; origin={origin:?}; class={class:?}; arity={arity}; \
                     candidates={candidates:?}; expression={expression}"
                ))
            })?;
        if record.class != class
            || record.arity != arity
            || record.phase != ResultPhase::CarrierRequired
            || record.children.iter().enumerate().any(|(position, child)| {
                child.position != u32::try_from(position).unwrap_or(u32::MAX)
                    || self.semantic.child_origin(origin, position).ok() != Some(child.origin)
            })
        {
            let expression = self
                .source_occurrence(origin)
                .map(|expr| format!("{expr:?}"))
                .unwrap_or_else(|_| "<missing>".to_string());
            return Err(planner_error(format!(
                "aggregate representation record disagrees with its exact source occurrence; \
                 requested_class={class:?}; requested_arity={arity}; record={record:?}; \
                 expression={expression}"
            )));
        }
        let key = (owner, origin);
        if self
            .aggregate_representation_dispositions
            .borrow()
            .contains(&key)
        {
            return Err(planner_error(format!(
                "one aggregate occurrence was both statically dispositioned and emitted: \
                 {key:?}"
            )));
        }
        let mut ledger = self.aggregate_representation_consumption.borrow_mut();
        let count = ledger.entry(key).or_insert(0);
        *count = count
            .checked_add(1)
            .ok_or_else(|| planner_capacity_error("aggregate representation ledger exhausted"))?;
        Ok(AggregateRepresentationToken {
            tag: record.selected_tag,
            class: record.class,
        })
    }

    pub(in crate::cranelift_backend) fn synthesized_aggregate_occurrence(
        &self,
        effect_origin: StaticOriginId,
        site: SynthesizedAggregateSite,
        role: SynthesizedConstructorRole,
        arity: usize,
    ) -> Result<SynthesizedAggregateOccurrence, CraneliftBackendError> {
        let owner = self
            .semantic
            .function_owner(effect_origin)?
            .ok_or_else(|| planner_error("synthesized aggregate has no function owner"))?;
        let arity = u32::try_from(arity)
            .map_err(|_| planner_capacity_error("synthesized aggregate arity exhausted"))?;
        let record = self
            .synthesized_aggregate_representations
            .iter()
            .find(|record| {
                record.owner == owner
                    && record.effect_origin == effect_origin
                    && record.site == site
            })
            .ok_or_else(|| {
                let candidates = self
                    .synthesized_aggregate_representations
                    .iter()
                    .filter(|record| record.effect_origin == effect_origin && record.site == site)
                    .map(|record| (record.owner, record.phase, record.role, record.arity))
                    .collect::<Vec<_>>();
                planner_error(format!(
                    "compiler-synthesized aggregate has no exact planned occurrence; \
                     owner={owner:?}; effect={effect_origin:?}; site={site:?}; \
                     role={role:?}; arity={arity}; candidates={candidates:?}"
                ))
            })?;
        if record.role != role || record.arity != arity || record.children.len() != arity as usize {
            return Err(planner_error(
                "compiler-synthesized aggregate request disagrees with its planned occurrence",
            ));
        }
        Ok(SynthesizedAggregateOccurrence {
            effect_origin,
            owner,
            site,
        })
    }

    pub(in crate::cranelift_backend) fn synthesized_aggregate_representation_token(
        &self,
        occurrence: SynthesizedAggregateOccurrence,
        class: BoundaryClass,
        arity: usize,
    ) -> Result<AggregateRepresentationToken, CraneliftBackendError> {
        let arity = u32::try_from(arity)
            .map_err(|_| planner_capacity_error("synthesized aggregate arity exhausted"))?;
        let record = self
            .synthesized_aggregate_representations
            .iter()
            .find(|record| {
                record.owner == occurrence.owner
                    && record.effect_origin == occurrence.effect_origin
                    && record.site == occurrence.site
            })
            .ok_or_else(|| {
                planner_error("compiler-synthesized aggregate token names no planned occurrence")
            })?;
        if class != BoundaryClass::Constructor
            || record.arity != arity
            || record.phase != ResultPhase::CarrierRequired
        {
            return Err(planner_error(
                "compiler-synthesized aggregate allocation disagrees with its planned record",
            ));
        }
        let mut ledger = self
            .synthesized_aggregate_representation_consumption
            .borrow_mut();
        let count = ledger.entry(occurrence).or_insert(0);
        *count = count.checked_add(1).ok_or_else(|| {
            planner_capacity_error("synthesized aggregate representation ledger exhausted")
        })?;
        Ok(AggregateRepresentationToken {
            tag: record.selected_tag,
            class,
        })
    }

    /// The artifact-static constructor identity of a `Construct` occurrence —
    /// the producer side of [`Self::case_constructor_identity`] (`D2`).
    pub(in crate::cranelift_backend) fn constructor_symbol_identity(
        &self,
        origin: StaticOriginId,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        self.semantic.constructor_symbol_identity(origin)
    }

    /// Resolve one already-interned constructor spelling by a closed semantic
    /// suffix used by an effect seat.
    ///
    /// This does not intern or pack a second identity: it filters the semantic
    /// plane's existing carrier catalog and requires exactly one result.
    pub(in crate::cranelift_backend) fn effect_constructor_tag_word(
        &self,
        suffix: &'static str,
    ) -> Result<u64, CraneliftBackendError> {
        let mut matches = self
            .semantic
            .carrier_identity_catalog()?
            .into_iter()
            .filter(|(spelling, _)| spelling.ends_with(suffix));
        if let Some((_, identity)) = matches.next() {
            if matches.next().is_some() {
                return Err(planner_error(
                    "effect constructor suffix names more than one planned identity",
                ));
            }
            return Ok(identity);
        }
        let fallback = match suffix {
            "::Stdin" => SynthesizedFixedConstructorRole::EffectConsoleStdin,
            "::Stdout" => SynthesizedFixedConstructorRole::EffectConsoleStdout,
            "::Stderr" => SynthesizedFixedConstructorRole::EffectConsoleStderr,
            "::CreateNew" => SynthesizedFixedConstructorRole::EffectCreateNew,
            "::CreateOrTruncate" => SynthesizedFixedConstructorRole::EffectCreateOrTruncate,
            "::CreateOrKeep" => SynthesizedFixedConstructorRole::EffectCreateOrKeep,
            "::ResourceRead" => SynthesizedFixedConstructorRole::EffectResourceRead,
            "::ResourceMetadata" => SynthesizedFixedConstructorRole::EffectResourceMetadata,
            "::ResourceWriteCreate" => SynthesizedFixedConstructorRole::EffectResourceWriteCreate,
            _ => {
                return Err(planner_error(format!(
                    "effect constructor identity ending in {suffix:?} is absent"
                )));
            }
        };
        let identity = self
            .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(fallback))?
            .tag_abi_word()?;
        if matches.next().is_some() {
            return Err(planner_error(
                "effect constructor suffix names more than one planned identity",
            ));
        }
        Ok(identity)
    }

    /// The existing semantic-plane identity for one compiler-synthesized
    /// constructor role.
    ///
    /// The key is a closed sum.  In particular, dynamic IOError alternatives
    /// can only be named with opaque tokens returned by
    /// [`Self::synthesized_io_error_roles`].
    pub(in crate::cranelift_backend) fn synthesized_constructor_identity(
        &self,
        role: SynthesizedConstructorRole,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        self.semantic.synthesized_constructor_identity(role)
    }

    pub(in crate::cranelift_backend) fn synthesized_io_error_roles(
        &self,
    ) -> &[SynthesizedIoErrorRole] {
        self.semantic.synthesized_io_error_roles()
    }

    /// The artifact-static field identity a `Project` occurrence selects (`D1`).
    pub(in crate::cranelift_backend) fn project_field_identity(
        &self,
        origin: StaticOriginId,
    ) -> Result<FieldIdentity, CraneliftBackendError> {
        self.semantic.project_field_identity(origin)
    }

    /// The artifact-static field identity of one field of a `Record` occurrence
    /// — the producer side of [`Self::project_field_identity`] (`D2`).
    pub(in crate::cranelift_backend) fn record_field_identity(
        &self,
        origin: StaticOriginId,
        position: usize,
    ) -> Result<FieldIdentity, CraneliftBackendError> {
        self.semantic.record_field_identity(origin, position)
    }

    /// The **occurrence** origin of the whole program's root.
    ///
    /// Read from the value stored during the root's own planning visit (D9), not
    /// derived from `entries.first()` — that is a *scheduling* entry, and for a
    /// root whose body is a `ComputationalMatch` it names the scrutinee.
    pub(in crate::cranelift_backend) fn root_static_origin(
        &self,
    ) -> Result<StaticOriginId, CraneliftBackendError> {
        self.root_occurrence
            .ok_or_else(|| planner_error("plan has no root occurrence"))
    }

    /// The **occurrence** origin of a transparent declaration, by symbol.
    ///
    /// `None` is a real answer, not a failure: a declaration that is not
    /// transparent has no planned body, and the lowering rejects it on its own
    /// terms. The caller must not substitute an origin of its own when this is
    /// `None`.
    pub(in crate::cranelift_backend) fn declaration_occurrence_origin(
        &self,
        symbol: &str,
    ) -> Option<StaticOriginId> {
        self.declaration_occurrences.get(symbol).copied()
    }

    pub(in crate::cranelift_backend) fn trap_identity(
        &self,
        trap: &RuntimeTrap,
    ) -> Result<PlannedTrapIdentity, CraneliftBackendError> {
        self.trap_catalog
            .iter()
            .position(|candidate| candidate == trap)
            .ok_or_else(|| {
                planner_error(format!(
                    "trap outcome has no planner-bound identity: {trap:?}"
                ))
            })
            .and_then(|index| {
                u32::try_from(index + 1)
                    .map(PlannedTrapIdentity)
                    .map_err(|_| planner_capacity_error("trap identity exhausted"))
            })
    }

    pub(in crate::cranelift_backend) fn trap_catalog(&self) -> Vec<RuntimeTrap> {
        self.trap_catalog.clone()
    }

    pub(in crate::cranelift_backend) fn carrier_identity_catalog(
        &self,
    ) -> Result<Vec<(String, u64)>, CraneliftBackendError> {
        self.semantic.carrier_identity_catalog()
    }

    /// **`RT-FNSPLIT-B2F` `D1` — every function unit this artifact must emit, in
    /// unit order.**
    ///
    /// ⛔ **This does not derive the population and must never be made to.** The
    /// set is `plan.entries` ∪ every `EdgeKind::StaticBody` **target**, already
    /// seeded and validated by `B2O` (`semantic_ir.rs`
    /// `validate_function_units`) and already given one descriptor apiece by
    /// `B2R`. This walks `self.abi.descriptors` and projects; it re-seeds
    /// nothing, and in particular it does **not** consult
    /// `TransitionKind::ClosureBody`, which is a body's *return successor* and
    /// not a unit head.
    ///
    /// ⚠ The two shared exits (`SemanticOwner::Terminal`, `TrapTerminal`) are not
    /// units and are absent here by construction — they never receive a
    /// descriptor.
    /// **`RT-FNSPLIT-B2F` `AC-11` — prove every transfer this node will emit is
    /// representable, BEFORE any unit is declared, defined or called.**
    ///
    /// ⛔ Exposed as a **verdict**, not as the plane: the semantic plane and its
    /// source seeds stay private, so an emitter can obtain the answer and cannot
    /// re-derive a different one. ⭐ That is what keeps this a single authority
    /// rather than a check the emitter could route around.
    ///
    /// ⛔ Clause 3 is discharged by the CALL SITE's position, not by this
    /// method's contents: it runs before `declare_unit_bundle` in
    /// `compile_expr_into_module`. Moving the call after emission would satisfy
    /// every assertion inside it and discharge nothing.
    pub(in crate::cranelift_backend) fn validate_emitted_transfers_are_representable(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        abi::validate_emitted_transfers(
            &self.semantic,
            &self.nodes,
            &self.semantic_sources,
            &self.abi.descriptors,
            &self.abi.slots,
        )
    }

    /// **`RT-FNSPLIT-B2F` `D4` — the cross-owner call edges, DERIVED.**
    ///
    /// ⛔ **Nothing here decides what a call edge is.** The classification is
    /// `B2O`'s and it is enforced as `return Err` arms inside
    /// `SemanticPlane::validate_function_units`: a `StaticBody` edge crosses to
    /// a **distinct** unit and lands on that unit's **seed**; every other edge
    /// either stays inside one unit or exits to a shared exit; anything else is
    /// refused during planning. ⇒ ⭐ **A plan that reaches this method cannot
    /// carry a violating edge**, so this is a projection of facts already
    /// validated, not a second classification.
    ///
    /// ⚠ **Which is exactly why `B2F` must not re-assert those four laws.** A
    /// control here asserting "a `StaticBody` edge crosses owners" is green on
    /// every input that can reach emission and tests nothing. ⭐ What `B2F` owes
    /// instead is **one-for-one consumption** — that emission is driven by this
    /// view and does not build a second table beside it — which is a property
    /// the inert node could not check about itself.
    ///
    /// ⛔ **Fails closed on a missing descriptor** rather than skipping the
    /// edge: a dropped call edge is a unit that is never called, which is
    /// silent at emission and wrong at run time.
    /// ⛔ **The owner classification is NOT named here**, and that is enforced:
    /// `the_owner_classification_has_a_closed_production_naming_inventory` reds
    /// if this file starts spelling `SemanticOwner`. ⇒ The `StaticBody` walk
    /// lives in `semantic_ir.rs`, beside the validation that makes it sound, and
    /// this method only wraps the resulting id pairs in the emitter's view type.
    ///
    /// ⭐ That pin caught a real defect in this deliverable's first draft, which
    /// destructured `SemanticOwner::Function(..)` right here — a third file
    /// naming the classification is how a second, divergent classification
    /// authority starts.
    pub(in crate::cranelift_backend) fn emittable_call_edges(
        &self,
    ) -> Result<Vec<EmittableCallEdge>, CraneliftBackendError> {
        let specialized_bases = self.callable_base_functions()?;
        let mut eliminated_literal_bodies = BTreeSet::new();
        for call in &self.static_callable_calls {
            for argument in &call.arguments {
                if matches!(
                    argument.kind,
                    StaticCallableArgumentKind::Direct { .. } | StaticCallableArgumentKind::Erased
                ) {
                    eliminated_literal_bodies.insert((
                        call.caller,
                        self.semantic.child_origin(argument.argument_origin, 0)?,
                    ));
                }
            }
        }
        let mut calls = self
            .semantic
            .static_body_call_edges(&self.edges)?
            .into_iter()
            .filter(|(caller, _, callee_origin)| {
                !specialized_bases.contains(caller)
                    && !eliminated_literal_bodies.contains(&(*caller, *callee_origin))
            })
            .map(|(caller, callee, callee_origin)| EmittableCallEdge {
                caller,
                callee,
                callee_origin,
                call_site_origin: callee_origin,
                kind: EmittableCallKind::StaticBody,
            })
            .collect::<Vec<_>>();
        calls.extend(
            self.semantic
                .declaration_call_edges(&self.edges)?
                .into_iter()
                .filter(|(caller, _, _, call_site_origin)| {
                    !specialized_bases.contains(caller)
                        && !self.static_callable_calls.iter().any(|call| {
                            call.caller == *caller
                                && call.callee_reference_origin == *call_site_origin
                        })
                })
                .map(
                    |(caller, callee, callee_origin, call_site_origin)| EmittableCallEdge {
                        caller,
                        callee,
                        callee_origin,
                        call_site_origin,
                        kind: EmittableCallKind::Declaration,
                    },
                ),
        );
        let callable_body_callee =
            |binding: &StaticCallableBindingKey|
             -> Result<PredeclaredFunctionId, CraneliftBackendError> {
                if binding_has_callable_capture(binding) {
                    let mut normalized = binding.clone();
                    normalized.parameter_ordinal = 0;
                    return self
                        .static_callable_specializations
                        .iter()
                        .find_map(|specialization| match &specialization.kind {
                            PlannedStaticCallableSpecializationKind::CallableBody {
                                binding,
                            } if binding == &normalized => Some(specialization.function),
                            PlannedStaticCallableSpecializationKind::Declaration
                            | PlannedStaticCallableSpecializationKind::CallableBody { .. } => {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            planner_error(
                                "recursive callable binding has no out-of-line body unit",
                            )
                        });
                }
                self.semantic
                    .function_owner(binding.body_origin)?
                    .ok_or_else(|| {
                        planner_error("static callable body has no function owner")
                    })
            };
        for specialization in &self.static_callable_specializations {
            match &specialization.kind {
                PlannedStaticCallableSpecializationKind::Declaration => {
                    let base_declaration = self
                        .declaration_occurrences
                        .iter()
                        .find_map(|(symbol, origin)| {
                            (*origin == specialization.base_origin).then_some(symbol.as_str())
                        })
                        .ok_or_else(|| {
                            planner_error("specialization base has no declaration symbol")
                        })
                        .and_then(|symbol| {
                            callable_declaration_plan(self, symbol)?
                                .ok_or_else(|| planner_error("specialization base is not callable"))
                        })?;
                    for binding in &specialization.key.bindings {
                        if !base_declaration
                            .parameter_uses
                            .get(binding.parameter_ordinal as usize)
                            .is_some_and(|use_| use_.invoked)
                        {
                            continue;
                        }
                        calls.push(EmittableCallEdge {
                            caller: specialization.function,
                            callee: callable_body_callee(binding)?,
                            callee_origin: binding.body_origin,
                            call_site_origin: binding.body_origin,
                            kind: EmittableCallKind::StaticBody,
                        });
                    }
                }
                PlannedStaticCallableSpecializationKind::CallableBody { binding } => {
                    let body = self
                        .source_occurrences
                        .get(binding.body_origin.0 as usize)
                        .and_then(Option::as_ref)
                        .ok_or_else(|| {
                            planner_error("callable body specialization has no occurrence")
                        })?
                        .expr;
                    for (ordinal, capture) in binding.captures.iter().enumerate() {
                        let StaticCallableCaptureBinding::Callable(nested) = capture else {
                            continue;
                        };
                        let capture_index = binding
                            .declared_arity
                            .checked_add(u32::try_from(ordinal).map_err(|_| {
                                planner_capacity_error("callable capture ordinal exhausted")
                            })?)
                            .ok_or_else(|| {
                                planner_capacity_error("callable capture index exhausted")
                            })?;
                        if !classify_callable_parameter_use(body, capture_index)?.invoked {
                            continue;
                        }
                        calls.push(EmittableCallEdge {
                            caller: specialization.function,
                            callee: callable_body_callee(nested)?,
                            callee_origin: nested.body_origin,
                            call_site_origin: nested.body_origin,
                            kind: EmittableCallKind::StaticBody,
                        });
                    }
                }
            }
            for (caller, callee, callee_origin) in
                self.semantic.static_body_call_edges(&self.edges)?
            {
                let eliminated_literal =
                    eliminated_literal_bodies.contains(&(specialization.function, callee_origin));
                if caller == specialization.body_function && !eliminated_literal {
                    calls.push(EmittableCallEdge {
                        caller: specialization.function,
                        callee,
                        callee_origin,
                        call_site_origin: callee_origin,
                        kind: EmittableCallKind::StaticBody,
                    });
                }
            }
            for (caller, callee, callee_origin, call_site_origin) in
                self.semantic.declaration_call_edges(&self.edges)?
            {
                if caller == specialization.body_function
                    && !self.static_callable_calls.iter().any(|call| {
                        call.caller == specialization.function
                            && call.callee_reference_origin == call_site_origin
                    })
                {
                    calls.push(EmittableCallEdge {
                        caller: specialization.function,
                        callee,
                        callee_origin,
                        call_site_origin,
                        kind: EmittableCallKind::Declaration,
                    });
                }
            }
        }
        for call in &self.static_callable_calls {
            let specialization = self
                .static_callable_specializations
                .get(call.specialization.0 as usize)
                .ok_or_else(|| planner_error("static callable call names no specialization"))?;
            calls.push(EmittableCallEdge {
                caller: call.caller,
                callee: specialization.function,
                callee_origin: specialization.base_origin,
                call_site_origin: call.call_origin,
                kind: EmittableCallKind::StaticCallableSpecialization,
            });
        }
        calls.sort();
        calls.dedup();
        let mut sites = BTreeSet::new();
        for call in &calls {
            if !sites.insert((call.caller, call.call_site_origin, call.kind)) {
                return Err(planner_error(
                    "one generated function has two targets for one typed call edge",
                ));
            }
        }
        Ok(calls)
    }

    pub(in crate::cranelift_backend) fn emittable_static_callable_call(
        &self,
        caller: PredeclaredFunctionId,
        call_origin: StaticOriginId,
    ) -> Result<Option<EmittableStaticCallableCall>, CraneliftBackendError> {
        self.static_callable_calls
            .iter()
            .find(|call| call.caller == caller && call.call_origin == call_origin)
            .map(|call| {
                let specialization = self
                    .static_callable_specializations
                    .get(call.specialization.0 as usize)
                    .ok_or_else(|| planner_error("static callable call names no specialization"))?;
                Ok(EmittableStaticCallableCall {
                    arguments: call
                        .arguments
                        .iter()
                        .map(|argument| {
                            Ok(EmittableStaticCallableArgument {
                                parameter_ordinal: argument.parameter_ordinal,
                                argument_origin: argument.argument_origin,
                                kind: match argument.kind {
                                    StaticCallableArgumentKind::Ordinary => {
                                        EmittableStaticCallableArgumentKind::Ordinary
                                    }
                                    StaticCallableArgumentKind::Erased => {
                                        EmittableStaticCallableArgumentKind::Erased
                                    }
                                    StaticCallableArgumentKind::Direct { closure_origin } => {
                                        EmittableStaticCallableArgumentKind::Direct {
                                            closure_origin,
                                        }
                                    }
                                    StaticCallableArgumentKind::Forwarded {
                                        body_origin,
                                        declared_arity,
                                    } => EmittableStaticCallableArgumentKind::Forwarded {
                                        body_origin,
                                        declared_arity,
                                    },
                                },
                                binding: specialization
                                    .key
                                    .bindings
                                    .iter()
                                    .find(|binding| {
                                        binding.parameter_ordinal == argument.parameter_ordinal
                                    })
                                    .map(emittable_static_callable_binding)
                                    .transpose()?,
                            })
                        })
                        .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
                })
            })
            .transpose()
    }

    pub(in crate::cranelift_backend) fn emittable_static_callable_unit(
        &self,
        id: StaticCallableSpecializationId,
    ) -> Result<EmittableStaticCallableUnit, CraneliftBackendError> {
        let specialization = self
            .static_callable_specializations
            .get(id.0 as usize)
            .ok_or_else(|| planner_error("callable specialization id is outside the plan"))?;
        if specialization.id != id {
            return Err(planner_error(
                "callable specialization is not keyed by its dense identity",
            ));
        }
        let (parameter_count, declaration_captures, bindings, body_binding) = match &specialization
            .kind
        {
            PlannedStaticCallableSpecializationKind::Declaration => {
                let declaration = self
                    .source_occurrences
                    .get(specialization.base_origin.0 as usize)
                    .and_then(Option::as_ref)
                    .ok_or_else(|| planner_error("specialization base has no source occurrence"))?;
                let (parameter_count, declaration_captures) = match declaration.expr {
                    RuntimeExpr::Closure {
                        captures, params, ..
                    } => (params.len(), captures.len()),
                    RuntimeExpr::LexicalClosure {
                        captures, params, ..
                    } => (params.len(), captures.len()),
                    _ => {
                        return Err(planner_error(
                            "specialization base is not a transparent closure declaration",
                        ));
                    }
                };
                (
                    u32::try_from(parameter_count)
                        .map_err(|_| planner_capacity_error("specialization arity exhausted"))?,
                    u32::try_from(declaration_captures).map_err(|_| {
                        planner_capacity_error("specialization declaration captures exhausted")
                    })?,
                    specialization
                        .key
                        .bindings
                        .iter()
                        .map(emittable_static_callable_binding)
                        .collect::<Result<Vec<_>, CraneliftBackendError>>()?,
                    None,
                )
            }
            PlannedStaticCallableSpecializationKind::CallableBody { binding } => (
                binding.declared_arity,
                0,
                Vec::new(),
                Some(emittable_static_callable_binding(binding)?),
            ),
        };
        Ok(EmittableStaticCallableUnit {
            base_origin: specialization.base_origin,
            base_body_origin: specialization.base_body_origin,
            parameter_count,
            declaration_captures,
            bindings,
            body_binding,
        })
    }

    pub(in crate::cranelift_backend) fn root_emittable_unit(
        &self,
    ) -> Result<EmittableUnit<'_>, CraneliftBackendError> {
        let root_entry = self
            .root_entry
            .ok_or_else(|| planner_error("plan has no recorded root entry"))?;
        let root_function = self.semantic.function_for_node(root_entry)?;
        self.emittable_units()?
            .into_iter()
            .find(|unit| unit.function() == root_function)
            .ok_or_else(|| planner_error("recorded root has no abi descriptor"))
    }

    pub(in crate::cranelift_backend) fn emittable_units(
        &self,
    ) -> Result<Vec<EmittableUnit<'_>>, CraneliftBackendError> {
        let specialized_bases = self.callable_base_functions()?;
        self.abi
            .descriptors
            .iter()
            .filter(|descriptor| {
                !specialized_bases.contains(&descriptor.function)
                    || matches!(
                        descriptor.definition,
                        AbiUnitDefinition::StaticCallableSpecialization { .. }
                    )
            })
            .map(|descriptor| {
                let start = descriptor.slots.start as usize;
                let end = start
                    .checked_add(descriptor.slots.len as usize)
                    .ok_or_else(|| planner_error("abi slot range overflows"))?;
                let slots = self
                    .abi
                    .slots
                    .get(start..end)
                    .ok_or_else(|| planner_error("abi slot range is outside the plane"))?;
                Ok(EmittableUnit {
                    function: descriptor.function,
                    origin: descriptor.origin,
                    definition: descriptor.definition,
                    header: descriptor.header,
                    slots,
                })
            })
            .collect()
    }

    fn helper_key_for_activation(
        &self,
        node: StaticNodeId,
        activation: DynamicActivationFrame,
    ) -> Result<PlannedHelperKey, CraneliftBackendError> {
        let static_node = self
            .nodes
            .get(node.0 as usize)
            .ok_or_else(|| planner_error("activation names an unknown static node"))?;
        let store_is_closed =
            |id: PersistentNodeId| id.0 == 0 || id.0 as usize <= self.stores.len();
        for id in [
            activation.syntax,
            activation.environment,
            activation.normal,
            activation.abrupt,
            activation.path,
            activation.cleanup,
            activation.affine,
            activation.source_return,
        ] {
            if !store_is_closed(id) {
                return Err(planner_error(
                    "activation frame references an unclosed persistent node",
                ));
            }
        }
        Ok(PlannedHelperKey::node(static_node.transition, node))
    }

    fn validate(&self) -> Result<(), CraneliftBackendError> {
        if self.entries.is_empty() {
            return Err(planner_error("closed graph has no entry"));
        }
        self.validate_operand_edge_matrix()?;
        self.validate_static_recursor_worker_residuals()?;
        self.validate_producer_flow_plans()?;
        self.validate_recursor_boundary_uses()?;
        self.validate_lowering_boundary_uses()?;
        self.validate_boundary_uses()?;
        self.validate_static_callable_specializations()?;
        if self.evidence.len() != self.edges.len() {
            return Err(planner_error("edge evidence is incomplete"));
        }
        if self.store_depths.len() != self.stores.len() {
            return Err(planner_error(
                "persistent store depth table does not match the store",
            ));
        }
        let mut unique_stores = BTreeSet::new();
        for (index, node) in self.stores.iter().enumerate() {
            if !unique_stores.insert(*node) {
                return Err(planner_error("persistent store contains a duplicate node"));
            }
            let child_depth = if node.child.0 == 0 {
                0
            } else {
                let child_index = node.child.0 as usize - 1;
                if child_index >= index {
                    return Err(planner_error(
                        "persistent store child is not an earlier closed node",
                    ));
                }
                self.store_depths[child_index]
            };
            let depth = child_depth
                .checked_add(1)
                .ok_or_else(|| planner_capacity_error("persistent chain depth exhausted"))?;
            if self.store_depths[index] != depth {
                return Err(planner_error(
                    "persistent store depth does not match its child chain",
                ));
            }
        }

        let mut expected_helpers = BTreeSet::new();
        for (index, node) in self.nodes.iter().enumerate() {
            if node.id.0 as usize != index {
                return Err(planner_error(
                    "static node identity does not match its closed position",
                ));
            }
            expected_helpers.insert(PlannedHelperKey::node(node.transition, node.id));
        }
        let closed_nodes = self
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<BTreeSet<_>>();
        if self
            .entries
            .iter()
            .any(|entry| entry.0 as usize >= self.nodes.len())
        {
            return Err(planner_error("graph entry is outside the closed node set"));
        }
        if self.entries.iter().copied().collect::<BTreeSet<_>>().len() != self.entries.len() {
            return Err(planner_error("closed graph contains a duplicate entry"));
        }
        for (index, edge) in self.edges.iter().enumerate() {
            if edge.id.0 as usize != index {
                return Err(planner_error(
                    "static edge identity does not match its closed position",
                ));
            }
            if edge.from.0 as usize >= self.nodes.len() || edge.to.0 as usize >= self.nodes.len() {
                return Err(planner_error("edge endpoint is outside the closed graph"));
            }
            expected_helpers.insert(PlannedHelperKey::edge(edge.kind, edge.id));
        }
        let actual_helpers = self
            .planned_helpers
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if actual_helpers.len() != self.planned_helpers.len() {
            return Err(planner_error(
                "planned helper inventory contains a duplicate identity",
            ));
        }
        if actual_helpers != expected_helpers
            || self.planned_helpers.len() != self.nodes.len() + self.edges.len()
        {
            return Err(planner_error(
                "planned helper inventory is not exact for the closed graph",
            ));
        }
        let mut helpers = BTreeMap::<StaticSourceId, usize>::new();
        for helper in &self.planned_helpers {
            let owner =
                match *helper {
                    PlannedHelperKey::Node(transition, id) => {
                        let node = self.nodes.get(id.0 as usize).ok_or_else(|| {
                            planner_error("planned node helper is outside the graph")
                        })?;
                        if transition != node.transition || id != node.id {
                            return Err(planner_error(
                                "planned node helper does not match its static node",
                            ));
                        }
                        node.owner
                    }
                    PlannedHelperKey::Edge(kind, id) => {
                        let edge = self.edges.get(id.0 as usize).ok_or_else(|| {
                            planner_error("planned edge helper is outside the graph")
                        })?;
                        if kind != edge.kind || id != edge.id {
                            return Err(planner_error(
                                "planned edge helper does not match its static edge",
                            ));
                        }
                        self.nodes[edge.from.0 as usize].owner
                    }
                };
            *helpers.entry(owner).or_default() += 1;
        }

        for (index, (edge, evidence)) in self.edges.iter().zip(&self.evidence).enumerate() {
            if evidence.edge as usize != index
                || evidence.owner != self.nodes[edge.from.0 as usize].owner
                || evidence.from != edge.from
                || evidence.to != edge.to
                || evidence.kind != edge.kind
            {
                return Err(planner_error("out-of-line edge evidence is not exact"));
            }
        }
        for node in &self.nodes {
            if self.helper_key_for_activation(node.id, node.frame)?
                != PlannedHelperKey::node(node.transition, node.id)
            {
                return Err(planner_error(
                    "dynamic activation changed static helper identity",
                ));
            }
        }

        let terminals = self
            .nodes
            .iter()
            .filter(|node| node.transition == TransitionKind::Terminal)
            .map(|node| node.id)
            .collect::<Vec<_>>();
        let trap_terminals = self
            .nodes
            .iter()
            .filter(|node| node.transition == TransitionKind::TrapTerminal)
            .map(|node| node.id)
            .collect::<Vec<_>>();
        if terminals.len() != 1 || trap_terminals.len() != 1 {
            return Err(planner_error(
                "closed graph must have exactly one Terminal and TrapTerminal",
            ));
        }
        let terminal = terminals[0];
        let trap_terminal = trap_terminals[0];
        if self
            .edges
            .iter()
            .any(|edge| edge.from == terminal || edge.from == trap_terminal)
        {
            return Err(planner_error(
                "Terminal and TrapTerminal must have no outgoing edges",
            ));
        }

        self.validate_source_return_topology()?;
        if helpers.values().copied().max().unwrap_or(0) > MAX_HELPERS_PER_STATIC_SOURCE {
            return Err(planner_error(
                "fixed K helpers per static source was exceeded",
            ));
        }

        let mut reachable = self.entries.iter().copied().collect::<BTreeSet<_>>();
        reachable.extend([terminal, trap_terminal]);
        loop {
            let before = reachable.len();
            for edge in &self.edges {
                if reachable.contains(&edge.from) {
                    reachable.insert(edge.to);
                }
            }
            if reachable.len() == before {
                break;
            }
        }
        if reachable != closed_nodes {
            return Err(planner_error(
                "closed graph contains unreachable transitions",
            ));
        }
        self.semantic.validate(
            &self.nodes,
            &self.edges,
            &self.entries,
            &self.semantic_sources,
            &self.semantic_material,
        )?;
        self.abi.validate(
            &self.semantic,
            &self.nodes,
            &self.semantic_sources,
            &self.edges,
            &self.entries,
            self.root_entry
                .ok_or_else(|| planner_error("plan has no root scheduling entry"))?,
            self.root_ingress,
            &self.static_callable_specializations,
        )?;
        self.validate_source_occurrence_table()?;
        self.validate_join_result_plan()?;
        Ok(())
    }

    fn validate_producer_flow_plans(&self) -> Result<(), CraneliftBackendError> {
        let (expected_cases, expected_aggregates, expected_synthesized_aggregates) =
            build_producer_flow_plans(self)?;
        if self.case_emissions != expected_cases {
            return Err(planner_error(
                "case-emission partition is not the exact producer-flow derivation",
            ));
        }
        if self.aggregate_representations != expected_aggregates {
            return Err(planner_error(
                "aggregate representation plan is not the exact producer-flow derivation",
            ));
        }
        if self.synthesized_aggregate_representations != expected_synthesized_aggregates {
            return Err(planner_error(
                "synthesized aggregate representation plan is not the exact effect-schema \
                 derivation",
            ));
        }
        let mut keys = BTreeSet::new();
        for record in &self.case_emissions {
            if !keys.insert((record.owner, record.match_origin, record.ordinal)) {
                return Err(planner_error(
                    "case-emission partition contains a duplicate source case",
                ));
            }
        }
        let mut aggregate_keys = BTreeSet::new();
        for record in &self.aggregate_representations {
            if !aggregate_keys.insert((record.owner, record.origin)) {
                return Err(planner_error(
                    "aggregate representation plan contains a duplicate occurrence",
                ));
            }
            if record.selected_tag.referent_owner() != record.selected_owner
                || !matches!(
                    (record.selected_tag, record.class),
                    (
                        BoundaryTag::PersistentGround,
                        BoundaryClass::Constructor | BoundaryClass::Record
                    ) | (
                        BoundaryTag::InvocationAggregate,
                        BoundaryClass::Constructor | BoundaryClass::Record
                    )
                )
            {
                return Err(planner_error(
                    "aggregate representation selected an unlawful tag, class, or owner",
                ));
            }
        }
        let mut synthesized_keys = BTreeSet::new();
        for record in &self.synthesized_aggregate_representations {
            if !synthesized_keys.insert((record.owner, record.effect_origin, record.site)) {
                return Err(planner_error(
                    "synthesized aggregate plan contains a duplicate occurrence",
                ));
            }
            if !matches!(
                record.phase,
                ResultPhase::SpecializedOnly | ResultPhase::CarrierRequired
            ) || record.selected_tag.referent_owner() != record.selected_owner
                || !matches!(
                    (record.selected_tag, BoundaryClass::Constructor),
                    (BoundaryTag::PersistentGround, BoundaryClass::Constructor)
                        | (BoundaryTag::InvocationAggregate, BoundaryClass::Constructor)
                )
            {
                return Err(planner_error(
                    "synthesized aggregate record selects an unrepresented row",
                ));
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn validate_case_emissions(&self) -> Result<(), CraneliftBackendError> {
        self.validate_producer_flow_plans()
    }

    fn validate_static_callable_specializations(&self) -> Result<(), CraneliftBackendError> {
        let base_count = self.semantic.functions.len();
        let mut keys = BTreeSet::new();
        for (ordinal, specialization) in self.static_callable_specializations.iter().enumerate() {
            let id =
                StaticCallableSpecializationId(u32::try_from(ordinal).map_err(|_| {
                    planner_capacity_error("static callable specialization exhausted")
                })?);
            let function = PredeclaredFunctionId(
                u32::try_from(base_count.checked_add(ordinal).ok_or_else(|| {
                    planner_capacity_error("static callable function population exhausted")
                })?)
                .map_err(|_| {
                    planner_capacity_error("static callable function identity exhausted")
                })?,
            );
            if specialization.id != id || specialization.function != function {
                return Err(planner_error(
                    "static callable specialization is not densely interned",
                ));
            }
            if !keys.insert(specialization.key.clone()) {
                return Err(planner_error(
                    "static callable specialization key was cloned instead of reused",
                ));
            }
            if specialization.key.base_owner != specialization.base_function
                || specialization.key.base_origin != specialization.base_origin
            {
                return Err(planner_error(
                    "static callable specialization key lost its base owner/origin",
                ));
            }
            if self
                .semantic
                .function_owner(specialization.base_body_origin)?
                != Some(specialization.body_function)
            {
                return Err(planner_error(
                    "static callable specialization lost its body owner/origin",
                ));
            }
            let mut last_parameter = None;
            let mut lifted = 0u32;
            for binding in &specialization.key.bindings {
                if last_parameter.is_some_and(|last| last >= binding.parameter_ordinal) {
                    return Err(planner_error(
                        "static callable bindings are not strictly ordered by parameter",
                    ));
                }
                last_parameter = Some(binding.parameter_ordinal);
                lifted = lifted
                    .checked_add(validate_static_callable_binding(self, binding)?)
                    .ok_or_else(|| planner_capacity_error("lifted capture population exhausted"))?;
            }
            if lifted > specialization.lifted_captures {
                return Err(planner_error(
                    "static callable descriptor omits a lifted capture",
                ));
            }
        }
        let mut call_sites = BTreeSet::new();
        for call in &self.static_callable_calls {
            if !call_sites.insert((call.caller, call.call_origin)) {
                return Err(planner_error(
                    "static callable call-site mapping is not one-to-one",
                ));
            }
            let specialization = self
                .static_callable_specializations
                .get(call.specialization.0 as usize)
                .ok_or_else(|| {
                    planner_error("static callable call names no interned specialization")
                })?;
            let caller_environment = if let Some(caller_specialization) = self
                .static_callable_specializations
                .iter()
                .find(|specialization| specialization.function == call.caller)
            {
                match &caller_specialization.kind {
                    PlannedStaticCallableSpecializationKind::Declaration => {
                        let declaration = self
                            .source_occurrences
                            .get(caller_specialization.base_origin.0 as usize)
                            .and_then(Option::as_ref)
                            .ok_or_else(|| {
                                planner_error("caller specialization base has no occurrence")
                            })?;
                        let (parameter_count, capture_count) = match declaration.expr {
                            RuntimeExpr::Closure {
                                params, captures, ..
                            } => (params.len(), captures.len()),
                            RuntimeExpr::LexicalClosure {
                                params, captures, ..
                            } => (params.len(), captures.len()),
                            _ => {
                                return Err(planner_error(
                                    "caller declaration specialization base is not a closure",
                                ));
                            }
                        };
                        let mut environment = vec![None; parameter_count];
                        for binding in &caller_specialization.key.bindings {
                            let slot = environment
                                .get_mut(binding.parameter_ordinal as usize)
                                .ok_or_else(|| {
                                    planner_error(
                                        "caller specialization binding exceeds declaration arity",
                                    )
                                })?;
                            *slot = Some(binding.clone());
                        }
                        environment.extend((0..capture_count).map(|_| None));
                        environment
                    }
                    PlannedStaticCallableSpecializationKind::CallableBody { binding } => {
                        let mut environment = vec![None; binding.declared_arity as usize];
                        environment.extend(binding.captures.iter().map(|capture| match capture {
                            StaticCallableCaptureBinding::Value(_) => None,
                            StaticCallableCaptureBinding::Callable(binding) => {
                                Some((**binding).clone())
                            }
                        }));
                        environment
                    }
                }
            } else {
                Vec::new()
            };
            let occurrence = self
                .source_occurrences
                .get(call.call_origin.0 as usize)
                .and_then(Option::as_ref)
                .ok_or_else(|| planner_error("static callable call has no source occurrence"))?;
            let RuntimeExpr::Call { args, .. } = occurrence.expr else {
                return Err(planner_error(
                    "static callable call-site mapping names a non-call",
                ));
            };
            if call.arguments.len() != args.len()
                || call
                    .arguments
                    .iter()
                    .enumerate()
                    .any(|(ordinal, argument)| {
                        argument.parameter_ordinal as usize != ordinal
                            || self
                                .semantic
                                .child_origin(call.call_origin, 1 + ordinal)
                                .ok()
                                != Some(argument.argument_origin)
                    })
            {
                return Err(planner_error(
                    "static callable call-site argument mapping is not exact",
                ));
            }
            let eliminated = call
                .arguments
                .iter()
                .filter(|argument| argument.kind != StaticCallableArgumentKind::Ordinary)
                .map(|argument| argument.parameter_ordinal)
                .collect::<Vec<_>>();
            let keyed = specialization
                .key
                .bindings
                .iter()
                .map(|binding| binding.parameter_ordinal)
                .collect::<Vec<_>>();
            if eliminated != keyed {
                return Err(planner_error(
                    "static callable call-site use closure disagrees with its interned key",
                ));
            }
            for argument in &call.arguments {
                let Some(binding) = specialization
                    .key
                    .bindings
                    .iter()
                    .find(|binding| binding.parameter_ordinal == argument.parameter_ordinal)
                else {
                    continue;
                };
                match argument.kind {
                    StaticCallableArgumentKind::Direct { closure_origin } => {
                        let derived = static_binding_from_closure(
                            self,
                            argument.parameter_ordinal,
                            closure_origin,
                            &caller_environment,
                        )?
                        .ok_or_else(|| {
                            planner_error(
                                "direct callable argument no longer resolves to a closure",
                            )
                        })?;
                        if &derived != binding {
                            return Err(planner_error(
                                "direct callable argument disagrees with its body/arity/capture \
                                 provenance key",
                            ));
                        }
                    }
                    StaticCallableArgumentKind::Erased => {
                        if !binding.captures.is_empty() {
                            return Err(planner_error(
                                "unused callable binding retained runtime captures",
                            ));
                        }
                    }
                    StaticCallableArgumentKind::Forwarded {
                        body_origin,
                        declared_arity,
                    } => {
                        if binding.body_origin != body_origin
                            || binding.declared_arity != declared_arity
                        {
                            return Err(planner_error(
                                "forwarded callable binding changed body identity or arity",
                            ));
                        }
                    }
                    StaticCallableArgumentKind::Ordinary => {
                        return Err(planner_error(
                            "ordinary argument appears in the callable binding key",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_operand_edge_matrix(&self) -> Result<(), CraneliftBackendError> {
        let mut expected = BTreeSet::new();
        for occurrence in self.source_occurrences.iter().flatten() {
            let parent = occurrence.static_origin;
            let children = self.semantic.child_origins(parent)?;
            let roles = source_child_roles(occurrence.expr);
            if roles.len() != children.len() {
                return Err(planner_error(
                    "operand-edge role inventory is not exact for positional source children",
                ));
            }
            let owner = self
                .semantic
                .function_owner(parent)?
                .ok_or_else(|| planner_error("source operand edge has no function owner"))?;
            for (position, (child, role)) in children.iter().copied().zip(roles).enumerate() {
                let SourceChildRole::Operand(role) = role else {
                    continue;
                };
                let (effect_operation, effect_seat) =
                    effect_edge_contract(occurrence.expr, position, role)?;
                let position = u32::try_from(position)
                    .map_err(|_| planner_capacity_error("operand-edge position exhausted"))?;
                let producer_owner = self.semantic.function_owner(child)?.ok_or_else(|| {
                    planner_error("source operand producer has no function owner")
                })?;
                let disposition =
                    derive_operand_edge_disposition(self, parent, child, position, role)?;
                let (consumer_phase, operation, need, avail) =
                    operand_edge_contract(disposition, effect_seat);
                expected.insert(PlannedOperandEdge {
                    owner,
                    producer_owner,
                    parent,
                    child,
                    position,
                    role,
                    effect_operation,
                    effect_seat,
                    disposition,
                    producer_phase: BoundaryUsePhase::SpecializedValue,
                    consumer_phase,
                    operation,
                    need,
                    avail,
                });
            }
        }
        let actual = self.operand_edges.iter().copied().collect::<BTreeSet<_>>();
        if actual.len() != self.operand_edges.len() {
            return Err(planner_error(
                "operand-edge matrix contains a duplicate consumer edge",
            ));
        }
        if actual != expected {
            return Err(planner_error(
                "operand-edge matrix is not exact for positional source consumers",
            ));
        }
        Ok(())
    }

    fn validate_static_recursor_worker_residuals(&self) -> Result<(), CraneliftBackendError> {
        let expected = build_static_recursor_worker_residuals(self)?;
        if self.static_recursor_worker_residuals != expected {
            return Err(planner_error(
                "static recursor worker residual population is not exact",
            ));
        }
        Ok(())
    }

    fn validate_recursor_boundary_uses(&self) -> Result<(), CraneliftBackendError> {
        if self.recursor_boundary_uses != build_recursor_boundary_uses(self)? {
            return Err(planner_error(
                "computational recursor boundary-use population is not exact",
            ));
        }
        Ok(())
    }

    fn validate_lowering_boundary_uses(&self) -> Result<(), CraneliftBackendError> {
        if self.lowering_boundary_uses != build_lowering_boundary_uses(self)? {
            return Err(planner_error(
                "lowering boundary-use population is not exact",
            ));
        }
        if self
            .lowering_boundary_uses
            .iter()
            .any(|edge| matches!(edge.identity, BoundaryUseIdentity::Synthesized(0)))
        {
            return Err(planner_error(
                "lowering boundary-use population contains an anonymous identity",
            ));
        }
        Ok(())
    }

    fn validate_boundary_uses(&self) -> Result<(), CraneliftBackendError> {
        if self.boundary_uses != build_boundary_uses(self)? {
            return Err(planner_error(
                "planned boundary-use population is not the exact unified set",
            ));
        }
        if self
            .boundary_uses
            .iter()
            .any(|edge| matches!(edge.identity, BoundaryUseIdentity::Synthesized(0)))
        {
            return Err(planner_error(
                "planned boundary-use population contains an anonymous identity",
            ));
        }
        Ok(())
    }

    fn validate_join_result_plan(&self) -> Result<(), CraneliftBackendError> {
        if self.join_results.len() != self.source_occurrences.len() {
            return Err(planner_error(
                "join result plan is not dense over the occurrence table",
            ));
        }
        if self.result_phases.len() != self.source_occurrences.len() {
            return Err(planner_error(
                "result-phase plan is not dense over the occurrence table",
            ));
        }
        for (index, (occurrence, join)) in self
            .source_occurrences
            .iter()
            .zip(&self.join_results)
            .enumerate()
        {
            match (occurrence, join) {
                (Some(occurrence), Some(_)) if is_source_join(occurrence.expr) => {
                    if occurrence.static_origin.0 as usize != index {
                        return Err(planner_error(
                            "join result entry is not keyed by its source origin",
                        ));
                    }
                }
                (Some(occurrence), None) if !is_source_join(occurrence.expr) => {}
                (Some(_), None) => {
                    return Err(planner_error(
                        "source join occurrence has no result representation",
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err(planner_error(
                        "join result entry names a non-join source occurrence",
                    ));
                }
                (None, Some(_)) => {
                    return Err(planner_error(
                        "join result entry names no source occurrence",
                    ));
                }
                (None, None) => {}
            }
        }
        let (expected_joins, expected_phases) =
            build_join_result_plan(self, self.functionized_units)?;
        if self.join_results != expected_joins || self.result_phases != expected_phases {
            return Err(planner_error(
                "join and result-phase plans disagree with lexical value flow",
            ));
        }
        Ok(())
    }

    /// The occurrence table's three properties, each as its own failure.
    ///
    /// ⛔ Deliberately **not** one composite check. A single "the table is fine"
    /// assertion is discharged by any one of these holding, so a mutation that
    /// breaks exactly one would still be reported as the same failure; three
    /// named failures make three different mutations distinguishable.
    ///
    /// The cross-check is against `semantic_sources`, a population produced by a
    /// *different* mechanism in the same visit. Checking the table against itself
    /// could only ever confirm its internal shape — it could not notice that an
    /// occurrence the planner registered is missing from it.
    fn validate_source_occurrence_table(&self) -> Result<(), CraneliftBackendError> {
        // 1. Self-consistency: an entry's stored origin is the index it sits at.
        for (index, slot) in self.source_occurrences.iter().enumerate() {
            let Some(occurrence) = slot else {
                continue;
            };
            if occurrence.static_origin.0 as usize != index {
                return Err(planner_error(
                    "occurrence table entry is filed under an origin that is not its index",
                ));
            }
        }

        // 2. Totality over the occurrence population: every expression seed the
        //    walk registered has an entry, filed under that seed's own origin.
        let mut expression_seeds = 0usize;
        for seed in &self.semantic_sources {
            if !matches!(seed.source, SemanticSourceKind::Expression(_)) {
                continue;
            }
            expression_seeds += 1;
            let filed = self
                .source_occurrences
                .get(seed.origin.0 as usize)
                .and_then(|slot| slot.as_ref())
                .ok_or_else(|| {
                    planner_error("planned source occurrence is missing from the occurrence table")
                })?;
            if filed.static_origin != seed.origin {
                return Err(planner_error(
                    "occurrence table entry does not match its semantic seed's origin",
                ));
            }
        }

        // 3. No surplus: the table holds nothing no seed accounts for. With (2)
        //    this is injectivity — one entry per registered occurrence, and no
        //    entry without one.
        let filed = self
            .source_occurrences
            .iter()
            .filter(|slot| slot.is_some())
            .count();
        if filed != expression_seeds {
            return Err(planner_error(
                "occurrence table holds an entry no semantic seed accounts for",
            ));
        }
        Ok(())
    }

    fn validate_source_return_topology(&self) -> Result<(), CraneliftBackendError> {
        let special = |transition| {
            matches!(
                transition,
                TransitionKind::SourceReturnResume
                    | TransitionKind::ProducerWrapper
                    | TransitionKind::ProducerTail
                    | TransitionKind::CompletedTail
            )
        };
        let owners = self
            .nodes
            .iter()
            .filter(|node| special(node.transition))
            .map(|node| node.owner)
            .collect::<BTreeSet<_>>();
        for owner in owners {
            let one = |transition| {
                let nodes = self
                    .nodes
                    .iter()
                    .filter(|node| node.owner == owner && node.transition == transition)
                    .collect::<Vec<_>>();
                match nodes.as_slice() {
                    [node] => Ok(*node),
                    _ => Err(planner_error(
                        "computational source owner lacks one R/W/T/CompletedTail quartet",
                    )),
                }
            };
            let resume = one(TransitionKind::SourceReturnResume)?;
            let wrapper = one(TransitionKind::ProducerWrapper)?;
            let tail = one(TransitionKind::ProducerTail)?;
            let completed = one(TransitionKind::CompletedTail)?;
            let descriptor = wrapper.frame.source_return;
            if descriptor.0 == 0
                || [resume, tail, completed]
                    .iter()
                    .any(|node| node.frame.source_return != descriptor)
            {
                return Err(planner_error(
                    "computational quartet does not share one source-return descriptor",
                ));
            }
            let stored = self
                .stores
                .get(descriptor.0 as usize - 1)
                .ok_or_else(|| planner_error("source-return descriptor is not closed"))?;
            if stored.kind != StoreKind::SourceReturn
                || stored.local != wrapper.id.0
                || stored.aux != tail.id.0
            {
                return Err(planner_error(
                    "source-return descriptor does not name its exact W and T",
                ));
            }
            self.require_only_outgoing_edge(
                resume.id,
                wrapper.id,
                EdgeKind::InvokeProducerWrapper,
                "source-return resume must have only its exact wrapper invocation",
            )?;
            self.require_only_incoming_edge(
                wrapper.id,
                resume.id,
                EdgeKind::InvokeProducerWrapper,
                "producer wrapper must have only its exact resume invocation",
            )?;
            self.require_only_outgoing_edge(
                wrapper.id,
                tail.id,
                EdgeKind::InvokeProducerTail,
                "producer wrapper must have only its exact tail invocation",
            )?;
            self.require_only_incoming_edge(
                tail.id,
                wrapper.id,
                EdgeKind::InvokeProducerTail,
                "producer tail must have only its exact wrapper invocation",
            )?;
            self.require_only_outgoing_edge(
                tail.id,
                completed.id,
                EdgeKind::CompleteProducerTail,
                "producer tail must have only its exact completion edge",
            )?;
            self.require_only_incoming_edge(
                completed.id,
                tail.id,
                EdgeKind::CompleteProducerTail,
                "CompletedTail must have only its exact producer-tail completion",
            )?;
            if self.entries.contains(&wrapper.id) {
                return Err(planner_error(
                    "producer wrapper cannot be a pre-source graph entry",
                ));
            }

            let successor = self.activation_successor(completed)?;
            let completed_edges = self
                .edges
                .iter()
                .filter(|edge| edge.from == completed.id)
                .collect::<Vec<_>>();
            if !matches!(completed_edges.as_slice(), [edge] if edge.to == successor) {
                return Err(planner_error(
                    "CompletedTail must have only its activation-named successor",
                ));
            }
            let completed_edge = completed_edges[0];
            let successor_transition = self.nodes[successor.0 as usize].transition;
            let expected_kind = if successor_transition == TransitionKind::SourceReturnResume {
                EdgeKind::SourceReturnOwnedResume
            } else {
                EdgeKind::Continue
            };
            if completed_edge.kind != expected_kind {
                return Err(planner_error(
                    "CompletedTail successor does not use its normal-resume edge kind",
                ));
            }
        }
        for edge in self
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::SourceReturnOwnedResume)
        {
            let from = &self.nodes[edge.from.0 as usize];
            let to = &self.nodes[edge.to.0 as usize];
            let edge_descriptor = if from.transition == TransitionKind::CompletedTail {
                let descriptor_index =
                    from.frame.source_return.0.checked_sub(1).ok_or_else(|| {
                        planner_error(
                            "CompletedTail source return does not name a closed parent descriptor",
                        )
                    })? as usize;
                self.stores
                    .get(descriptor_index)
                    .filter(|descriptor| descriptor.kind == StoreKind::SourceReturn)
                    .map(|descriptor| descriptor.child)
                    .ok_or_else(|| {
                        planner_error(
                            "CompletedTail source return does not name a closed parent descriptor",
                        )
                    })?
            } else {
                from.frame.source_return
            };
            if to.transition != TransitionKind::SourceReturnResume
                || edge_descriptor.0 == 0
                || edge_descriptor != to.frame.source_return
            {
                return Err(planner_error(
                    "source-return-owned edge targets a resume from another descriptor",
                ));
            }
        }
        for edge in self
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::CompleteProducerTail)
        {
            let from = &self.nodes[edge.from.0 as usize];
            let to = &self.nodes[edge.to.0 as usize];
            if from.transition != TransitionKind::ProducerTail
                || to.transition != TransitionKind::CompletedTail
                || from.owner != to.owner
            {
                return Err(planner_error(
                    "producer completion is not owned by one computational source",
                ));
            }
        }
        Ok(())
    }

    fn activation_successor(
        &self,
        node: &StaticNode,
    ) -> Result<StaticNodeId, CraneliftBackendError> {
        let continuation_index =
            node.frame.normal.0.checked_sub(1).ok_or_else(|| {
                planner_error("activation does not name a closed normal continuation")
            })? as usize;
        let continuation = self
            .stores
            .get(continuation_index)
            .filter(|store| store.kind == StoreKind::Continuation)
            .ok_or_else(|| {
                planner_error("activation does not name a closed normal continuation")
            })?;
        let successor = StaticNodeId(continuation.local);
        if successor.0 as usize >= self.nodes.len() {
            return Err(planner_error(
                "activation normal continuation is outside the closed graph",
            ));
        }
        Ok(successor)
    }

    fn require_only_outgoing_edge(
        &self,
        from: StaticNodeId,
        to: StaticNodeId,
        kind: EdgeKind,
        error: &'static str,
    ) -> Result<(), CraneliftBackendError> {
        let edges = self
            .edges
            .iter()
            .filter(|edge| edge.from == from)
            .collect::<Vec<_>>();
        if !matches!(edges.as_slice(), [edge] if edge.to == to && edge.kind == kind) {
            return Err(planner_error(error));
        }
        Ok(())
    }

    fn require_only_incoming_edge(
        &self,
        to: StaticNodeId,
        from: StaticNodeId,
        kind: EdgeKind,
        error: &'static str,
    ) -> Result<(), CraneliftBackendError> {
        let edges = self
            .edges
            .iter()
            .filter(|edge| edge.to == to)
            .collect::<Vec<_>>();
        if !matches!(edges.as_slice(), [edge] if edge.from == from && edge.kind == kind) {
            return Err(planner_error(error));
        }
        Ok(())
    }

    #[cfg(test)]
    fn census(&self) -> BoundaryACensus {
        let max_depth = |kind| {
            self.stores
                .iter()
                .zip(&self.store_depths)
                .filter_map(|(node, depth)| (node.kind == kind).then_some(*depth))
                .max()
                .unwrap_or(0)
        };
        let mut helpers = BTreeMap::<StaticSourceId, usize>::new();
        for helper in &self.planned_helpers {
            let owner = match *helper {
                PlannedHelperKey::Node(_, id) => self.nodes[id.0 as usize].owner,
                PlannedHelperKey::Edge(_, id) => {
                    let edge = self.edges[id.0 as usize];
                    self.nodes[edge.from.0 as usize].owner
                }
            };
            *helpers.entry(owner).or_default() += 1;
        }
        BoundaryACensus {
            static_nodes: self.nodes.len(),
            edges: self.edges.len(),
            planned_helpers: self.planned_helpers.len(),
            persistent_store_nodes: self.stores.len(),
            out_of_line_evidence_records: self.evidence.len(),
            max_helpers_per_static_source: helpers.values().copied().max().unwrap_or(0),
            helper_key_bytes: std::mem::size_of::<PlannedHelperKey>(),
            activation_frame_bytes: std::mem::size_of::<DynamicActivationFrame>(),
            store_node_bytes: std::mem::size_of::<PersistentStoreNode>(),
            helper_key_schemas: 1,
            frame_schemas: 1,
            store_node_schemas: 1,
            static_node_id_bytes: std::mem::size_of::<StaticNodeId>(),
            persistent_node_id_bytes: std::mem::size_of::<PersistentNodeId>(),
            max_logical_chain_depth: self.store_depths.iter().copied().max().unwrap_or(0),
            max_environment_depth: max_depth(StoreKind::Environment),
            max_continuation_depth: max_depth(StoreKind::Continuation),
            max_path_depth: max_depth(StoreKind::Path),
            max_cleanup_depth: max_depth(StoreKind::Cleanup),
            max_affine_depth: max_depth(StoreKind::Affine),
            max_source_return_depth: max_depth(StoreKind::SourceReturn),
            source_return_resume_nodes: self
                .nodes
                .iter()
                .filter(|node| node.transition == TransitionKind::SourceReturnResume)
                .count(),
            source_return_owned_resume_edges: self
                .edges
                .iter()
                .filter(|edge| edge.kind == EdgeKind::SourceReturnOwnedResume)
                .count(),
            terminal_outgoing_edges: self
                .edges
                .iter()
                .filter(|edge| {
                    matches!(
                        self.nodes[edge.from.0 as usize].transition,
                        TransitionKind::Terminal | TransitionKind::TrapTerminal
                    )
                })
                .count(),
            recursive_lowering_frames: max_recursive_lowering_frame_count(),
        }
    }

    /// Capture the complete plan/semantic/ABI material that the emission
    /// collector will bind to its completed-object row.
    ///
    /// Descriptor work is counted in explicit representation work units: one
    /// descriptor header plus each slot it constructs, and the same closed
    /// population compared by `AbiPlane::validate`.  This keeps the metric tied
    /// to the actual descriptor/slot population rather than to wall-clock
    /// sampling or a source-text proxy.
    #[cfg(test)]
    pub(in crate::cranelift_backend) fn scale_b_census(&self) -> ScaleBPlanCensus {
        let outer = self.census();
        let descriptor_work = self
            .abi
            .descriptors
            .len()
            .checked_add(self.abi.slots.len())
            .expect("the descriptor work population fits usize");
        ScaleBPlanCensus {
            static_nodes: outer.static_nodes,
            edges: outer.edges,
            planned_helpers: outer.planned_helpers,
            persistent_store_nodes: outer.persistent_store_nodes,
            out_of_line_evidence_records: outer.out_of_line_evidence_records,
            max_helpers_per_static_source: outer.max_helpers_per_static_source,
            helper_key_bytes: outer.helper_key_bytes,
            activation_frame_bytes: outer.activation_frame_bytes,
            store_node_bytes: outer.store_node_bytes,
            helper_key_schemas: outer.helper_key_schemas,
            frame_schemas: outer.frame_schemas,
            store_node_schemas: outer.store_node_schemas,
            static_node_id_bytes: outer.static_node_id_bytes,
            persistent_node_id_bytes: outer.persistent_node_id_bytes,
            max_logical_chain_depth: outer.max_logical_chain_depth,
            max_environment_depth: outer.max_environment_depth,
            max_continuation_depth: outer.max_continuation_depth,
            max_path_depth: outer.max_path_depth,
            max_cleanup_depth: outer.max_cleanup_depth,
            max_affine_depth: outer.max_affine_depth,
            max_source_return_depth: outer.max_source_return_depth,
            source_return_resume_nodes: outer.source_return_resume_nodes,
            source_return_owned_resume_edges: outer.source_return_owned_resume_edges,
            terminal_outgoing_edges: outer.terminal_outgoing_edges,
            recursive_lowering_frames: outer.recursive_lowering_frames,
            distinct_interned_semantic_states: self.semantic.records.len(),
            defined_helpers: self.semantic.functions.len(),
            descriptor_construction_work: descriptor_work,
            descriptor_comparison_work: descriptor_work,
        }
    }

    #[cfg(test)]
    fn semantic_census(&self) -> BoundaryB1Census {
        use semantic_ir::{
            CaptureLayout, CaptureSlot, PredeclaredFunction, RuledChild, SemanticDescriptor,
            SemanticOperandElement, SemanticProgram, SemanticRecord,
        };

        let opcode_vocabulary = self
            .semantic
            .records
            .iter()
            .map(|record| record.opcode)
            .collect::<BTreeSet<_>>()
            .len();
        let mut definitions = BTreeMap::new();
        for descriptor in &self.semantic.descriptors {
            *definitions.entry(descriptor.origin).or_insert(0usize) += 1;
        }
        let distinct_origins = definitions.len();
        let duplicate_origin_definitions = definitions
            .values()
            .map(|count| count.saturating_sub(1))
            .sum();
        let max_definitions_per_origin = definitions.values().copied().max().unwrap_or(0);
        let definitions_per_origin = if definitions
            .values()
            .all(|count| *count == max_definitions_per_origin)
        {
            max_definitions_per_origin
        } else {
            0
        };

        BoundaryB1Census {
            opcode_vocabulary,
            distinct_origins,
            ir_records: self.semantic.records.len(),
            semantic_edges: self.semantic.ruled_children.len(),
            function_units: self.semantic.functions.len(),
            definitions_per_origin,
            all_out_of_line_operand_elements: self.semantic.all_out_of_line_operand_elements(),
            duplicate_origin_definitions,
            post_origin_clones: self
                .semantic
                .programs
                .len()
                .saturating_sub(distinct_origins),
            max_definitions_per_origin,
            descriptor_bytes: std::mem::size_of::<SemanticDescriptor>(),
            program_bytes: std::mem::size_of::<SemanticProgram>(),
            record_bytes: std::mem::size_of::<SemanticRecord>(),
            operand_element_bytes: std::mem::size_of::<SemanticOperandElement>(),
            capture_layout_bytes: std::mem::size_of::<CaptureLayout>(),
            capture_slot_bytes: std::mem::size_of::<CaptureSlot>(),
            ruled_child_bytes: std::mem::size_of::<RuledChild>(),
            function_bytes: std::mem::size_of::<PredeclaredFunction>(),
        }
    }
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn plan_static_transition_graph<'src>(
    entry: &'src RuntimeExpr,
    declarations: &BTreeMap<&str, &'src RuntimeDeclaration>,
) -> Result<StaticTransitionPlan<'src>, CraneliftBackendError> {
    // The legacy direct-lowering fixtures exercise the retained authority and
    // do not install a UnitBundle. Production passes its selected authority at
    // the call site; D8's functionized controls do the same explicitly.
    plan_static_transition_graph_with_symbols(
        entry,
        declarations,
        &crate::NativeProcessSymbols::legacy_prelude(),
        AbiRootIngress::Value,
        false,
    )
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn plan_static_transition_graph_with_test_fixture_boundary_use<
    'src,
>(
    entry: &'src RuntimeExpr,
    declarations: &BTreeMap<&str, &'src RuntimeDeclaration>,
) -> Result<StaticTransitionPlan<'src>, CraneliftBackendError> {
    INCLUDE_TEST_FIXTURE_BOUNDARY_USE.with(|cell| cell.set(true));
    let result = plan_static_transition_graph(entry, declarations);
    INCLUDE_TEST_FIXTURE_BOUNDARY_USE.with(|cell| cell.set(false));
    result
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_synthesized_consumption_mutation(
    mutation: SynthesizedConsumptionMutation,
) {
    SYNTHESIZED_CONSUMPTION_MUTATION.with(|cell| cell.set(mutation));
    SYNTHESIZED_CONSUMPTION_MUTATED.with(|cell| cell.set(false));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_static_recursor_consumption_mutation(
    mutation: StaticRecursorConsumptionMutation,
) {
    STATIC_RECURSOR_CONSUMPTION_MUTATION.with(|cell| cell.set(mutation));
    STATIC_RECURSOR_CONSUMPTION_MUTATED.with(|cell| cell.set(false));
}

pub(in crate::cranelift_backend) fn plan_static_transition_graph_with_symbols<'src>(
    entry: &'src RuntimeExpr,
    declarations: &BTreeMap<&str, &'src RuntimeDeclaration>,
    symbols: &crate::NativeProcessSymbols,
    root_ingress: AbiRootIngress,
    functionized_units: bool,
) -> Result<StaticTransitionPlan<'src>, CraneliftBackendError> {
    #[cfg(test)]
    reset_recursive_lowering_frame_count();
    let mut planner = Planner::new()?;
    let empty = PersistentNodeId(0);
    let context = PlanContext {
        environment: empty,
        continuation: empty,
        path: empty,
        cleanup: empty,
        affine: empty,
        source_return: empty,
    };
    // D9/AC-15: `entries` keeps the SCHEDULING entry; the occurrence is stored
    // separately, from the same visit. For a root or declaration body that is a
    // `ComputationalMatch` these are different nodes, and that case is the
    // required discriminator.
    let root = planner.plan_expr(entry, context, planner.terminal, EdgeKind::Continue, 0)?;
    planner.plan.entries.push(root.entry);
    planner.plan.root_entry = Some(root.entry);
    planner.plan.root_occurrence = Some(root.occurrence);
    let mut declaration_entries = BTreeMap::new();
    for (symbol, declaration) in declarations {
        if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
            let planned =
                planner.plan_expr(body, context, planner.terminal, EdgeKind::Continue, 0)?;
            planner.plan.entries.push(planned.entry);
            // A declaration body is its own planned source occurrence, so its
            // occurrence origin is reachable by name. Two occurrences under one
            // symbol would make that lookup ambiguous, which is a planner bug
            // rather than an input condition.
            if planner
                .plan
                .declaration_occurrences
                .insert((*symbol).to_owned(), planned.occurrence)
                .is_some()
            {
                return Err(planner_error(
                    "transparent declaration planned more than one occurrence origin",
                ));
            }
            if declaration_entries
                .insert((*symbol).to_owned(), planned.entry)
                .is_some()
            {
                return Err(planner_error(
                    "transparent declaration planned more than one scheduling entry",
                ));
            }
        }
    }
    planner.connect_declaration_calls(&declaration_entries)?;
    planner.finish(symbols, root_ingress, functionized_units)
}

fn runtime_expr_tag(expr: &RuntimeExpr) -> u32 {
    match expr {
        RuntimeExpr::CheckedJoinSite { .. } => 0,
        RuntimeExpr::CheckedSubcontinuationFrame { .. } => 1,
        RuntimeExpr::CheckedRecursiveInvocation { .. } => 2,
        RuntimeExpr::CheckedComputationalIHSlots { .. } => 3,
        RuntimeExpr::CheckedComputationalIHInvocation { .. } => 4,
        RuntimeExpr::Value(_) => 5,
        RuntimeExpr::Var(_) => 6,
        RuntimeExpr::Let { .. } => 7,
        RuntimeExpr::If { .. } => 8,
        RuntimeExpr::PrimitiveCall { .. } => 9,
        RuntimeExpr::Construct { .. } => 10,
        RuntimeExpr::Match { .. } => 11,
        RuntimeExpr::ComputationalMatch { .. } => 12,
        RuntimeExpr::Record { .. } => 13,
        RuntimeExpr::Project { .. } => 14,
        RuntimeExpr::Closure { .. } => 15,
        RuntimeExpr::LexicalClosure { .. } => 16,
        RuntimeExpr::DeclarationRef { .. } => 17,
        RuntimeExpr::ImportedDeclarationRef { .. } => 18,
        RuntimeExpr::Call { .. } => 19,
        RuntimeExpr::Effect { .. } => 20,
        RuntimeExpr::Trap(_) => 21,
    }
}

/// The governed nested-bracket source shared by the planning and emission
/// controls. Keeping one constructor prevents the emission gate from silently
/// measuring a trap-free or non-recursive surrogate.
#[cfg(test)]
pub(in crate::cranelift_backend) fn governed_nested_resource_bracket(depth: usize) -> RuntimeExpr {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum BinderRole {
        AllocatedBuffer,
        ScopeArgument,
        InductionHypothesis,
        RecursiveResult,
    }

    #[derive(Clone, Debug, Default)]
    struct BinderScope(Vec<BinderRole>);

    impl BinderScope {
        fn bind(&self, role: BinderRole) -> Self {
            let mut roles = self.0.clone();
            roles.push(role);
            Self(roles)
        }

        fn var(&self, role: BinderRole) -> RuntimeExpr {
            let index = self
                .0
                .iter()
                .rev()
                .position(|candidate| *candidate == role)
                .unwrap_or_else(|| panic!("governed bracket role {role:?} is not in scope"));
            RuntimeExpr::Var(
                u32::try_from(index).expect("governed bracket binder depth fits RuntimeExpr::Var"),
            )
        }
    }

    fn trap(message: &str) -> crate::RuntimeTrap {
        crate::RuntimeTrap {
            code: crate::RuntimeTrapCode::PatternMatchFailure,
            message: message.to_string(),
        }
    }

    fn unit() -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        }
    }

    if depth == 0 {
        return unit();
    }
    let recursive_body = governed_nested_resource_bracket(depth - 1);
    let closure_scope = BinderScope::default().bind(BinderRole::AllocatedBuffer);
    let release_scope = closure_scope.bind(BinderRole::RecursiveResult);
    let release = RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferFreeze,
            capability: None,
            args: vec![
                release_scope.var(BinderRole::AllocatedBuffer),
                RuntimeExpr::Value(crate::RuntimeValue::Int(0.into())),
                RuntimeExpr::Value(crate::RuntimeValue::Int(1.into())),
                release_scope.var(BinderRole::AllocatedBuffer),
            ],
        }),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: "ctor:prelude::Result::Err".to_string(),
                binders: 1,
                body: RuntimeExpr::Trap(trap("release failed")),
            },
            crate::RuntimeMatchCase {
                constructor: "ctor:prelude::Result::Ok".to_string(),
                binders: 1,
                body: unit(),
            },
        ],
        default: trap("release result"),
    };
    let closure_body = RuntimeExpr::Let {
        value: Box::new(recursive_body),
        body: Box::new(release),
    };
    let allocation_scope = BinderScope::default().bind(BinderRole::AllocatedBuffer);
    let bracket_case_scope = allocation_scope
        .bind(BinderRole::ScopeArgument)
        .bind(BinderRole::InductionHypothesis);
    let bracket = RuntimeExpr::ComputationalMatch {
        scrutinee: Box::new(RuntimeExpr::Construct {
            constructor: "ctor:fixture::Bracket::Scope".to_string(),
            args: vec![RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["buffer".to_string()],
                body: Box::new(closure_body),
            }],
        }),
        cases: vec![crate::RuntimeComputationalMatchCase {
            constructor: "ctor:fixture::Bracket::Scope".to_string(),
            argument_binders: 1,
            recursive_positions: vec![0],
            body: RuntimeExpr::Call {
                callee: Box::new(bracket_case_scope.var(BinderRole::InductionHypothesis)),
                args: vec![bracket_case_scope.var(BinderRole::AllocatedBuffer)],
            },
        }],
        default: trap("bracket scope"),
    };
    RuntimeExpr::Match {
        scrutinee: Box::new(RuntimeExpr::Effect {
            family: "FS".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(crate::RuntimeValue::Int(1.into()))],
        }),
        cases: vec![
            crate::RuntimeMatchCase {
                constructor: "ctor:prelude::Result::Err".to_string(),
                binders: 1,
                body: RuntimeExpr::Trap(trap("allocate failed")),
            },
            crate::RuntimeMatchCase {
                constructor: "ctor:prelude::Result::Ok".to_string(),
                binders: 1,
                body: bracket,
            },
        ],
        default: trap("allocate result"),
    }
}

#[cfg(test)]
mod tests {
    use super::abi::{AbiCarrier, AbiSlot, AbiSlotKind};
    use super::semantic_ir::{
        build_semantic_plane, DenseRange, PredeclaredFunctionId, RuntimeExprShape,
        SemanticAtomKind, SemanticOperandElement, SemanticOwner, SemanticSourceKind,
        StaticOriginId,
    };
    use super::*;
    use crate::cranelift_backend::surface::NativeSeedEnvironment;
    use crate::RuntimeGroundValue;
    use crate::{
        RuntimeComputationalMatchCase, RuntimeMatchCase, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
    };

    fn trap(message: &str) -> RuntimeTrap {
        RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: message.to_string(),
        }
    }

    fn unit() -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: "ctor:prelude::Unit::MkUnit".to_string(),
            args: Vec::new(),
        }
    }

    fn consume_lowering_boundary_uses(
        plan: &StaticTransitionPlan<'_>,
        omitted: Option<BoundaryUseIdentity>,
    ) -> Vec<BoundaryUseIdentity> {
        let planned = plan.lowering_boundary_uses.clone();
        planned
            .into_iter()
            .filter(|use_| Some(use_.identity) != omitted)
            .map(|use_| {
                plan.lowering_boundary_use_token(use_.edge, use_.origin, use_.position)
                    .expect("the exact synthesized authority is consumed")
                    .identity()
            })
            .collect()
    }

    fn d7_functionized_plan<'a>(
        entry: &'a RuntimeExpr,
        declarations: &BTreeMap<&str, &'a RuntimeDeclaration>,
    ) -> Result<StaticTransitionPlan<'a>, CraneliftBackendError> {
        plan_static_transition_graph_with_symbols(
            entry,
            declarations,
            &crate::NativeProcessSymbols::legacy_prelude(),
            AbiRootIngress::Value,
            true,
        )
    }

    fn d7_callable_declaration(symbol: &str, body: RuntimeExpr) -> RuntimeDeclaration {
        RuntimeDeclaration {
            symbol: symbol.to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["callable".to_string()],
                    body: Box::new(body),
                },
            },
            metadata: crate::RuntimeSymbolMetadata::empty(),
        }
    }

    fn d7_static_closure(tag: &str) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
            params: Vec::new(),
            body: Box::new(RuntimeExpr::Construct {
                constructor: tag.to_string(),
                args: Vec::new(),
            }),
        }
    }

    fn d7_declaration_call(symbol: &str, argument: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: symbol.to_string(),
            }),
            args: vec![argument],
        }
    }

    #[test]
    fn d7_static_callable_identity_keys_distinct_bodies_and_no_identity_slot() {
        let symbol = "decl:fixture::d7::identity";
        let declaration = d7_callable_declaration(
            symbol,
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: Vec::new(),
            },
        );
        let entry = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Pair".to_string(),
            args: vec![
                d7_declaration_call(symbol, d7_static_closure("ctor:fixture::Left")),
                d7_declaration_call(symbol, d7_static_closure("ctor:fixture::Right")),
            ],
        };
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations)
            .expect("two static callable bodies plan out of line");
        assert_eq!(plan.static_callable_specializations.len(), 2);
        let body_origins = plan
            .static_callable_specializations
            .iter()
            .map(|specialization| {
                assert_eq!(specialization.ordinary_parameters, 0);
                assert_eq!(specialization.lifted_captures, 1);
                specialization.key.bindings[0].body_origin
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            body_origins.len(),
            2,
            "different callable bodies aliased to one specialization key"
        );
        let specialization_edges = plan
            .emittable_call_edges()
            .expect("call edges project")
            .into_iter()
            .filter(|edge| edge.kind() == EmittableCallKind::StaticCallableSpecialization)
            .count();
        assert_eq!(
            specialization_edges, 2,
            "the two out-of-line units do not have two planner-derived call edges"
        );
        for specialization in &plan.static_callable_specializations {
            let descriptor_index = specialization.function.0 as usize;
            let descriptor = &plan.abi.descriptors[descriptor_index];
            assert!(matches!(
                descriptor.definition,
                AbiUnitDefinition::StaticCallableSpecialization { .. }
            ));
            assert_eq!(descriptor.header.parameters, 0);
            assert_eq!(descriptor.header.captures, 1);
            assert_eq!(
                descriptor.slots.len, 5,
                "callable identity leaked into the specialization ABI"
            );
        }

        let mut selector_slot = plan.clone();
        let descriptor_index = selector_slot.static_callable_specializations[0].function.0 as usize;
        selector_slot.abi.descriptors[descriptor_index]
            .header
            .parameters += 1;
        assert!(
            selector_slot.validate().is_err(),
            "adding a callable selector slot survived exact ABI validation"
        );

        let mut inlined = plan.clone();
        inlined.static_callable_specializations[0].body_function =
            inlined.static_callable_specializations[0].function;
        assert!(
            inlined.validate().is_err(),
            "collapsing the out-of-line body owner into the specialization survived validation"
        );

        let mut swapped = plan.clone();
        let left = swapped.static_callable_calls[0].specialization;
        let right = swapped.static_callable_calls[1].specialization;
        swapped.static_callable_calls[0].specialization = right;
        swapped.static_callable_calls[1].specialization = left;
        assert!(
            swapped.validate().is_err(),
            "swapping callable body keys survived exact call-edge validation"
        );

        let mut collapsed = plan;
        let first = collapsed.static_callable_calls[0].specialization;
        collapsed.static_callable_calls[1].specialization = first;
        assert!(
            collapsed.validate().is_err(),
            "collapsing two callable bodies to one global binding survived validation"
        );
    }

    #[test]
    fn d7_static_callable_capture_values_do_not_enter_the_key() {
        let target_symbol = "decl:fixture::d7::capture_target";
        let outer_symbol = "decl:fixture::d7::capture_outer";
        let target = d7_callable_declaration(
            target_symbol,
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: Vec::new(),
            },
        );
        let outer = RuntimeDeclaration {
            symbol: outer_symbol.to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["value".to_string()],
                    body: Box::new(d7_declaration_call(
                        target_symbol,
                        RuntimeExpr::LexicalClosure {
                            captures: vec![RuntimeExpr::Var(0)],
                            params: Vec::new(),
                            body: Box::new(RuntimeExpr::Var(0)),
                        },
                    )),
                },
            },
            metadata: crate::RuntimeSymbolMetadata::empty(),
        };
        let entry = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Pair".to_string(),
            args: vec![
                RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::DeclarationRef {
                        symbol: outer_symbol.to_string(),
                    }),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Bool(false))],
                },
                RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::DeclarationRef {
                        symbol: outer_symbol.to_string(),
                    }),
                    args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
                },
            ],
        };
        let declarations = BTreeMap::from([(target_symbol, &target), (outer_symbol, &outer)]);
        let plan = d7_functionized_plan(&entry, &declarations)
            .expect("one body with runtime capture inputs plans once");
        assert_eq!(plan.static_callable_specializations.len(), 1);
        let specialization = &plan.static_callable_specializations[0];
        assert_eq!(specialization.ordinary_parameters, 0);
        assert_eq!(specialization.lifted_captures, 1);
        assert_eq!(
            specialization.key.bindings[0]
                .lifted_capture_count()
                .expect("capture count is finite"),
            1
        );

        let mut deleted_capture = plan;
        deleted_capture.static_callable_specializations[0].lifted_captures = 0;
        assert!(
            deleted_capture.validate().is_err(),
            "deleting one lifted capture survived exact ABI validation"
        );
    }

    #[test]
    fn d7_static_callable_use_closure_and_runtime_selection_reject_pre_emission() {
        let returned_symbol = "decl:fixture::d7::returned";
        let returned = d7_callable_declaration(returned_symbol, RuntimeExpr::Var(0));
        let returned_entry =
            d7_declaration_call(returned_symbol, d7_static_closure("ctor:fixture::Body"));
        let returned_declarations = BTreeMap::from([(returned_symbol, &returned)]);
        assert!(
            d7_functionized_plan(&returned_entry, &returned_declarations).is_err(),
            "returning a callable parameter entered specialization"
        );

        let invoked_symbol = "decl:fixture::d7::selected";
        let invoked = d7_callable_declaration(
            invoked_symbol,
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: Vec::new(),
            },
        );
        let selected = RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            then_expr: Box::new(d7_static_closure("ctor:fixture::Left")),
            else_expr: Box::new(d7_static_closure("ctor:fixture::Right")),
        };
        let selected_entry = d7_declaration_call(invoked_symbol, selected);
        let selected_declarations = BTreeMap::from([(invoked_symbol, &invoked)]);
        assert!(
            d7_functionized_plan(&selected_entry, &selected_declarations).is_err(),
            "a runtime-selected callable entered specialization"
        );
    }

    #[test]
    fn d7_static_callable_recursion_interns_before_enqueue() {
        let symbol = "decl:fixture::d7::recursive";
        let declaration = d7_callable_declaration(
            symbol,
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::DeclarationRef {
                    symbol: symbol.to_string(),
                }),
                args: vec![RuntimeExpr::Var(0)],
            },
        );
        let entry = d7_declaration_call(symbol, d7_static_closure("ctor:fixture::Body"));
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations)
            .expect("recursive static binding reaches a finite fixed point");
        assert_eq!(plan.static_callable_specializations.len(), 1);
        assert_eq!(plan.static_callable_calls.len(), 2);
        assert_eq!(
            plan.static_callable_calls[0].specialization,
            plan.static_callable_calls[1].specialization
        );

        let mut cloned_state = plan;
        let duplicate = cloned_state.static_callable_specializations[0].clone();
        cloned_state.static_callable_specializations.push(duplicate);
        assert!(
            cloned_state.validate().is_err(),
            "cloning one recursively reused specialization survived the finite-state census"
        );
    }

    #[test]
    fn d7_static_callable_matrix_mutations_fail_validation() {
        let symbol = "decl:fixture::d7::matrix";
        let declaration = d7_callable_declaration(
            symbol,
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: Vec::new(),
            },
        );
        let entry = d7_declaration_call(symbol, d7_static_closure("ctor:fixture::Body"));
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan =
            d7_functionized_plan(&entry, &declarations).expect("static callable fixture plans");
        let edge_index = plan
            .operand_edges
            .iter()
            .position(|edge| edge.disposition == OperandEdgeDisposition::StaticCallableElimination)
            .expect("the real static callable edge is present");

        let mut omitted = plan.clone();
        omitted.operand_edges.remove(edge_index);
        assert!(omitted.validate().is_err());

        let mut reclassified = plan.clone();
        reclassified.operand_edges[edge_index].disposition = OperandEdgeDisposition::Forwarding;
        assert!(reclassified.validate().is_err());

        let mut deleted_use_closure = plan;
        deleted_use_closure.static_callable_calls[0].arguments[0].kind =
            StaticCallableArgumentKind::Ordinary;
        assert!(deleted_use_closure.validate().is_err());
    }

    #[test]
    fn d7_static_callable_capture_provenance_mutations_fail_validation() {
        let target_symbol = "decl:fixture::d7::capture_provenance_target";
        let outer_symbol = "decl:fixture::d7::capture_provenance_outer";
        let target = d7_callable_declaration(
            target_symbol,
            RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: Vec::new(),
            },
        );
        let outer = RuntimeDeclaration {
            symbol: outer_symbol.to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["left".to_string(), "right".to_string()],
                    body: Box::new(d7_declaration_call(
                        target_symbol,
                        RuntimeExpr::LexicalClosure {
                            captures: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
                            params: Vec::new(),
                            body: Box::new(RuntimeExpr::Construct {
                                constructor: "ctor:fixture::CapturePair".to_string(),
                                args: vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)],
                            }),
                        },
                    )),
                },
            },
            metadata: crate::RuntimeSymbolMetadata::empty(),
        };
        let entry = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: outer_symbol.to_string(),
            }),
            args: vec![
                RuntimeExpr::Value(RuntimeValue::Bool(false)),
                RuntimeExpr::Value(RuntimeValue::Bool(true)),
            ],
        };
        let declarations = BTreeMap::from([(target_symbol, &target), (outer_symbol, &outer)]);
        let plan = d7_functionized_plan(&entry, &declarations)
            .expect("two-capture callable fixture plans");
        let specialization = plan
            .static_callable_specializations
            .iter()
            .position(|specialization| specialization.lifted_captures == 2)
            .expect("one specialization lifts both captures");

        let mut reordered = plan.clone();
        reordered.static_callable_specializations[specialization]
            .key
            .bindings[0]
            .captures
            .swap(0, 1);
        assert!(reordered.validate().is_err());

        let mut wrong_owner = plan.clone();
        let StaticCallableCaptureBinding::Value(capture) = &mut wrong_owner
            .static_callable_specializations[specialization]
            .key
            .bindings[0]
            .captures[0]
        else {
            panic!("fixture capture is ordinary");
        };
        capture.owner = PredeclaredFunctionId(u32::MAX);
        assert!(wrong_owner.validate().is_err());

        let mut wrong_phase = plan.clone();
        let StaticCallableCaptureBinding::Value(capture) = &mut wrong_phase
            .static_callable_specializations[specialization]
            .key
            .bindings[0]
            .captures[0]
        else {
            panic!("fixture capture is ordinary");
        };
        capture.phase = match capture.phase {
            StaticCallableCapturePhase::SpecializedOnly => {
                StaticCallableCapturePhase::CarrierRequired
            }
            StaticCallableCapturePhase::CarrierRequired => {
                StaticCallableCapturePhase::SpecializedOnly
            }
        };
        assert!(wrong_phase.validate().is_err());

        let mut wrong_lifetime = plan;
        let StaticCallableCaptureBinding::Value(capture) = &mut wrong_lifetime
            .static_callable_specializations[specialization]
            .key
            .bindings[0]
            .captures[0]
        else {
            panic!("fixture capture is ordinary");
        };
        capture.closure_origin = StaticOriginId(u32::MAX);
        assert!(wrong_lifetime.validate().is_err());
    }

    #[test]
    fn d7_operand_edge_matrix_separates_static_bodies_from_callable_captures() {
        let expr = RuntimeExpr::LexicalClosure {
            captures: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
            params: Vec::new(),
            body: Box::new(unit()),
        };
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let root = plan.root_occurrence.expect("root occurrence");
        assert_eq!(plan.operand_edges.len(), 1);
        let edge = plan
            .operand_edge_token(root, 1, SourceOperandRole::LexicalCapture)
            .unwrap();
        assert_eq!(edge.disposition(), OperandEdgeDisposition::CallableCapture);
        assert_eq!(
            edge.identity(),
            BoundaryUseIdentity::Source {
                parent: root,
                child: plan.child_static_origin(root, 1).unwrap(),
                position: 1,
            }
        );
        assert!(plan
            .operand_edge_token(root, 0, SourceOperandRole::LexicalCapture)
            .is_err());
    }

    #[test]
    fn d7_omitting_one_real_callable_capture_edge_rejects_before_emission() {
        let expr = RuntimeExpr::LexicalClosure {
            captures: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
            params: Vec::new(),
            body: Box::new(unit()),
        };
        D7_OMIT_LEXICAL_CAPTURE_EDGE.with(|mutation| mutation.set(true));
        let result = plan_static_transition_graph(&expr, &BTreeMap::new());
        D7_OMIT_LEXICAL_CAPTURE_EDGE.with(|mutation| mutation.set(false));
        let error = match result {
            Ok(_) => panic!("omitting a real matrix member must reject"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("operand-edge matrix is not exact for positional source consumers"),
            "unexpected pre-emission rejection: {error}"
        );
    }

    fn static_recursor_worker_fixture(body_constructor: &str) -> RuntimeExpr {
        let constructor = "ctor:fixture::StaticRecursor::Node".to_string();
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: constructor.clone(),
                args: vec![RuntimeExpr::Closure {
                    captures: vec![
                        "seed:fixture::left".to_string(),
                        "seed:fixture::right".to_string(),
                    ],
                    params: vec!["argument".to_string()],
                    body: Box::new(RuntimeExpr::Construct {
                        constructor: body_constructor.to_string(),
                        args: vec![RuntimeExpr::Var(0)],
                    }),
                }],
            }),
            cases: vec![RuntimeComputationalMatchCase {
                constructor,
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::Var(0)),
                    args: vec![RuntimeExpr::Var(1)],
                },
            }],
            default: trap("static recursor fixture"),
        }
    }

    fn recursor_worker_constructor(constructor: &str, body_constructor: &str) -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: constructor.to_string(),
            args: vec![RuntimeExpr::Closure {
                captures: Vec::new(),
                params: vec!["argument".to_string()],
                body: Box::new(RuntimeExpr::Construct {
                    constructor: body_constructor.to_string(),
                    args: vec![RuntimeExpr::Var(0)],
                }),
            }],
        }
    }

    fn branched_static_recursor_worker_fixture(selected: bool) -> RuntimeExpr {
        let constructor = "ctor:fixture::BranchedWorker::Node";
        RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::If {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(selected))),
                then_expr: Box::new(recursor_worker_constructor(
                    constructor,
                    "ctor:fixture::BranchedWorker::Left",
                )),
                else_expr: Box::new(recursor_worker_constructor(
                    constructor,
                    "ctor:fixture::BranchedWorker::Right",
                )),
            }),
            cases: vec![RuntimeComputationalMatchCase {
                constructor: constructor.to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Var(0),
            }],
            default: trap("branched static recursor fixture"),
        }
    }

    #[test]
    fn static_recursor_population_follows_result_flow_not_syntax_containment() {
        let constructor = "ctor:fixture::ResultFlow::Node";
        let decoy_tree = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Decoy::Outer".to_string(),
            args: vec![
                RuntimeExpr::Closure {
                    captures: Vec::new(),
                    params: vec!["argument".to_string()],
                    body: Box::new(recursor_worker_constructor(
                        "ctor:fixture::Decoy::ClosureBody",
                        "ctor:fixture::Decoy::ClosureBodyWorker",
                    )),
                },
                recursor_worker_constructor(
                    "ctor:fixture::Decoy::NestedField",
                    "ctor:fixture::Decoy::NestedFieldWorker",
                ),
            ],
        };
        let scrutinee = RuntimeExpr::Let {
            value: Box::new(decoy_tree),
            body: Box::new(RuntimeExpr::If {
                scrutinee: Box::new(RuntimeExpr::Let {
                    value: Box::new(recursor_worker_constructor(
                        "ctor:fixture::Decoy::Condition",
                        "ctor:fixture::Decoy::ConditionWorker",
                    )),
                    body: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                }),
                then_expr: Box::new(recursor_worker_constructor(
                    constructor,
                    "ctor:fixture::ResultFlow::Left",
                )),
                else_expr: Box::new(recursor_worker_constructor(
                    constructor,
                    "ctor:fixture::ResultFlow::Right",
                )),
            }),
        };
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(scrutinee),
            cases: vec![RuntimeComputationalMatchCase {
                constructor: constructor.to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Var(0),
            }],
            default: trap("result-flow fixture"),
        };
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("result-flow recursor population plans");
        let body_constructors = plan
            .static_recursor_worker_residuals
            .iter()
            .map(|residual| {
                let body = plan
                    .source_occurrences
                    .get(residual.body_origin.0 as usize)
                    .and_then(Option::as_ref)
                    .expect("residual body has a source occurrence");
                let RuntimeExpr::Construct { constructor, .. } = body.expr else {
                    panic!("fixture worker body remains a constructor")
                };
                constructor.clone()
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            body_constructors,
            BTreeSet::from([
                "ctor:fixture::ResultFlow::Left".to_string(),
                "ctor:fixture::ResultFlow::Right".to_string(),
            ]),
            "conditions, unused let values, nested fields, and closure bodies \
             are not result-flow recursor authorities"
        );
    }

    #[test]
    fn static_recursor_worker_residual_is_an_exact_callable_capture_member() {
        // Promise class: durable invariant.
        //
        // MEASURED: the pre-emission plan returns one token keyed by the
        // computational parent and recursive position, with the fixture's
        // arity and ordered capture count.
        // CLAIMED: the actual constructor crossing, not a lowering-only label,
        // selects CallableCapture from its recursor provenance.
        // THE GAP: the token must also be consumed against the affine
        // invocation segment and runtime capture phases; lowering owns those
        // independent checks.
        let expr = static_recursor_worker_fixture("ctor:fixture::Worker::Left");
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("captured static recursor worker plans");
        let parent = plan.root_occurrence.expect("fixture has a root occurrence");
        let scrutinee = plan.child_static_origin(parent, 0).unwrap();
        let closure = plan.child_static_origin(scrutinee, 0).unwrap();
        let body = plan.child_static_origin(closure, 0).unwrap();
        let source_edge = plan
            .operand_edge_token(scrutinee, 0, SourceOperandRole::ConstructArgument)
            .expect("recursive constructor crossing is planned");
        assert_eq!(
            source_edge.identity(),
            BoundaryUseIdentity::Source {
                parent: scrutinee,
                child: closure,
                position: 0,
            }
        );
        assert_eq!(
            source_edge.disposition(),
            OperandEdgeDisposition::CallableCapture
        );
        let token = plan
            .static_recursor_worker_residual_token(parent, 0, body)
            .expect("token lookup is valid")
            .expect("the exact residual member exists");
        assert_eq!(token.disposition(), OperandEdgeDisposition::CallableCapture);
        assert_eq!(token.parent_origin, parent);
        assert_eq!(token.sibling_position, 0);
        assert_eq!(token.declared_arity, 1);
        assert_eq!(token.capture_count, 2);
        let forward_identity = token.identity();
        let reverse_plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("the exact residual replans");
        let reverse = reverse_plan
            .static_recursor_worker_residual_token_for_closure(closure)
            .expect("closure lookup is valid")
            .expect("closure lookup finds the same residual");
        assert_eq!(
            reverse.identity(),
            forward_identity,
            "forward and closure lookup must issue the same unified identity"
        );
        let ordinary = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Ordinary::Node".to_string(),
            args: vec![RuntimeExpr::Closure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(unit()),
            }],
        };
        let ordinary_plan = plan_static_transition_graph(&ordinary, &BTreeMap::new())
            .expect("ordinary closure-bearing constructor plans fail-closed");
        let ordinary_parent = ordinary_plan
            .root_occurrence
            .expect("ordinary constructor has an occurrence");
        let forbidden = ordinary_plan
            .operand_edge_token(ordinary_parent, 0, SourceOperandRole::ConstructArgument)
            .expect("ordinary constructor crossing is planned");
        assert_eq!(
            forbidden.disposition(),
            OperandEdgeDisposition::SemanticEliminator,
            "a specialized-only constructor reads an ordinary closure template \
             without inventing a boundary crossing"
        );

        let crossing = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Crossing::Node".to_string(),
            args: vec![
                RuntimeExpr::CheckedRecursiveInvocation {
                    call_template_id: 17,
                    checked_occurrence_path: vec![3],
                    body: Box::new(unit()),
                },
                RuntimeExpr::Closure {
                    captures: Vec::new(),
                    params: Vec::new(),
                    body: Box::new(unit()),
                },
            ],
        };
        let mut crossing_plan = d7_functionized_plan(&crossing, &BTreeMap::new())
            .expect("carrier-crossing closure-bearing constructor plans");
        let crossing_parent = crossing_plan
            .root_occurrence
            .expect("crossing constructor has an occurrence");
        let crossing_edge = crossing_plan
            .operand_edge_token(crossing_parent, 1, SourceOperandRole::ConstructArgument)
            .expect("crossing constructor edge is planned");
        assert_eq!(
            crossing_edge.disposition(),
            OperandEdgeDisposition::EscapeForbidden,
            "the same source role becomes fail-closed only when the exact \
             constructor crosses a generated carrier edge"
        );
        let carrier_child = crossing_plan
            .child_static_origin(crossing_parent, 0)
            .expect("crossing constructor has its carrier-producing child");
        crossing_plan.result_phases[carrier_child.0 as usize] =
            Some(ResultPhaseSummary::SPECIALIZED);
        let closure_child = crossing_plan
            .child_static_origin(crossing_parent, 1)
            .expect("crossing constructor has its closure child");
        let edge = crossing_plan
            .operand_edges
            .iter_mut()
            .find(|edge| edge.parent == crossing_parent && edge.child == closure_child)
            .expect("crossing closure edge remains planned");
        edge.disposition = OperandEdgeDisposition::SemanticEliminator;
        let (phase, operation, need, avail) =
            operand_edge_contract(edge.disposition, edge.effect_seat);
        edge.consumer_phase = phase;
        edge.operation = operation;
        edge.need = need;
        edge.avail = avail;
        assert!(
            crossing_plan.validate().is_err(),
            "mutating both the value-flow authority and its derived verdict \
             cannot make the paired drift self-consistent"
        );
    }

    #[test]
    fn static_worker_recursor_use_is_one_exact_unified_member() {
        // Promise class: durable invariant.
        //
        // The four lowering routes for one recursive-position residual consume
        // one planner-interned identity.  Deleting that identity or changing
        // its phase contract must fail the independent population derivation.
        let expr = static_recursor_worker_fixture("ctor:fixture::Worker::Left");
        let mut plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("captured static recursor worker plans");
        let parent = plan.root_occurrence.expect("fixture has a root occurrence");
        assert!(
            plan.recursor_boundary_uses.is_empty(),
            "the generic SpecializedOnlyLeaf authority must not compete with \
             an exact static worker"
        );
        let residual = plan
            .static_recursor_worker_residuals
            .first()
            .expect("the exact worker residual is planned");
        let token = plan
            .static_recursor_worker_residual_token(parent, 0, residual.body_origin)
            .expect("recursive position lookup is valid")
            .expect("recursive position has one planned boundary use");
        assert!(matches!(
            token.identity(),
            BoundaryUseIdentity::Synthesized(_)
        ));
        assert_eq!(token.producer_origin, residual.producer_origin);
        assert_eq!(
            plan.boundary_uses
                .iter()
                .filter(|planned| planned.identity == token.identity())
                .count(),
            1
        );

        plan.boundary_uses
            .retain(|planned| planned.identity != token.identity());
        assert!(plan.validate_boundary_uses().is_err());

        let mut plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("captured static recursor worker replans");
        let planned = plan
            .boundary_uses
            .iter_mut()
            .find(|planned| {
                matches!(
                    planned.path,
                    PlannedBoundaryUsePath::StaticRecursorWorker { .. }
                )
            })
            .expect("the exact worker is in the unified population");
        planned.consumer_phase = BoundaryUsePhase::OperationalCarrier;
        assert!(plan.validate_boundary_uses().is_err());
    }

    #[test]
    fn same_parent_position_result_workers_keep_distinct_unified_identities() {
        let expr = branched_static_recursor_worker_fixture(true);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("both result-flow workers plan");
        let parent = plan.root_occurrence.expect("fixture has a root occurrence");
        assert_eq!(plan.static_recursor_worker_residuals.len(), 2);
        assert!(plan
            .static_recursor_worker_residuals
            .iter()
            .all(|residual| {
                residual.parent_origin == parent && residual.sibling_position == 0
            }));
        assert!(
            plan.recursor_boundary_uses.is_empty(),
            "one generic parent/position authority would collapse the pair"
        );
        let worker_uses = plan
            .boundary_uses
            .iter()
            .filter_map(|planned| match &planned.path {
                PlannedBoundaryUsePath::StaticRecursorWorker {
                    parent_origin,
                    producer_origin,
                    sibling_position,
                    body_origin,
                    ..
                } => Some((
                    planned.identity,
                    *parent_origin,
                    *producer_origin,
                    *sibling_position,
                    *body_origin,
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(worker_uses.len(), 2);
        assert_ne!(worker_uses[0].0, worker_uses[1].0);
        assert_eq!(worker_uses[0].1, worker_uses[1].1);
        assert_eq!(worker_uses[0].3, worker_uses[1].3);
        assert_ne!(worker_uses[0].2, worker_uses[1].2);
        assert_ne!(worker_uses[0].4, worker_uses[1].4);
    }

    #[test]
    fn static_unselected_result_dispositions_only_its_worker_identity() {
        let expr = branched_static_recursor_worker_fixture(true);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("both result-flow workers plan");
        let parent = plan.root_occurrence.expect("fixture has a root occurrence");
        let scrutinee = plan.child_static_origin(parent, 0).unwrap();
        let unselected = plan.child_static_origin(scrutinee, 2).unwrap();
        let selected = plan.child_static_origin(scrutinee, 1).unwrap();
        plan.disposition_boundary_uses_in_owner_subtree(unselected)
            .expect("the exact statically dead result subtree closes");
        let worker_identity = |producer_origin| {
            plan.boundary_uses
                .iter()
                .find_map(|planned| match &planned.path {
                    PlannedBoundaryUsePath::StaticRecursorWorker {
                        producer_origin: candidate,
                        ..
                    } if *candidate == producer_origin => Some(planned.identity),
                    _ => None,
                })
                .expect("result producer owns one exact worker use")
        };
        let worker_identities = plan
            .boundary_uses
            .iter()
            .filter_map(|planned| {
                matches!(
                    planned.path,
                    PlannedBoundaryUsePath::StaticRecursorWorker { .. }
                )
                .then_some(planned.identity)
            })
            .collect::<BTreeSet<_>>();
        let dispositioned_workers = plan
            .boundary_use_dispositions
            .borrow()
            .intersection(&worker_identities)
            .copied()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            dispositioned_workers,
            BTreeSet::from([worker_identity(unselected)]),
            "causal branch disposition reached another worker identity"
        );
        assert!(!plan
            .boundary_use_dispositions
            .borrow()
            .contains(&worker_identity(selected)));
    }

    #[test]
    fn boundary_use_source_and_synthesized_share_one_planned_emitted_ledger() {
        let expr = RuntimeExpr::Construct {
            constructor: "ctor:fixture::BoundaryUse::Node".to_string(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
        };
        let mut plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("the mixed source/synthesized population plans");
        let root = plan
            .root_static_origin()
            .expect("the fixture has a root occurrence");

        let source = plan
            .operand_edge_token(root, 0, SourceOperandRole::ConstructArgument)
            .expect("the exact source child has unified authority");
        let synthesized = consume_lowering_boundary_uses(&plan, None);
        let expected = std::iter::once(source.identity())
            .chain(synthesized)
            .collect::<BTreeSet<_>>();
        let ledger = plan.operand_edge_consumption.borrow();
        assert_eq!(
            ledger.keys().copied().collect::<BTreeSet<_>>(),
            expected,
            "source and synthesized emission must enter one identity ledger"
        );
        drop(ledger);
        plan.validate_boundary_use_consumption()
            .expect("the shared planned/emitted ledger closes");

        plan.boundary_uses.pop();
        assert!(
            plan.validate().is_err(),
            "omitting either identity class from the unified planned set must \
             reject before lowering"
        );
    }

    #[test]
    fn source_boundary_use_consumption_is_exactly_once() {
        // Promise class: durable invariant with population-side mutations.
        //
        // MEASURED: omitting and repeating issuance of the real ConstructArgument
        // authority reach distinct exact pre-emission ledger errors.
        // CLAIMED: every planned source crossing is consumed exactly once.
        // THE GAP: the synthesized use must already be consumed so neither
        // control can pass or fail because the other identity class is absent.
        let expr = RuntimeExpr::Construct {
            constructor: "ctor:fixture::BoundaryUse::SourceExact".to_string(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
        };
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("the exact source-consumption fixture plans");
        let root = plan
            .root_static_origin()
            .expect("the fixture has a root occurrence");
        let source_identity = BoundaryUseIdentity::Source {
            parent: root,
            child: plan
                .child_static_origin(root, 0)
                .expect("the fixture has one exact child"),
            position: 0,
        };
        consume_lowering_boundary_uses(&plan, None);
        let missing = plan
            .validate_boundary_use_consumption()
            .expect_err("an unconsumed planned source use must reject");
        assert!(matches!(
            missing,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(ref detail))
                if detail.starts_with(&format!(
                    "source boundary-use ledger is not exact; missing={:?};",
                    [source_identity]
                ))
        ));

        plan.operand_edge_token(root, 0, SourceOperandRole::ConstructArgument)
            .expect("the exact source authority is consumed once");
        plan.operand_edge_token(root, 0, SourceOperandRole::ConstructArgument)
            .expect("the same issued authority is consumed a second time");
        let duplicate = plan
            .validate_boundary_use_consumption()
            .expect_err("repeated source consumption must reject");
        assert!(matches!(
            duplicate,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(ref detail))
                if detail == &format!(
                    "source boundary-use ledger contains duplicate consumption; \
                     duplicates={:?}",
                    [(source_identity, 2)]
                )
        ));
    }

    #[test]
    fn synthesized_boundary_use_consumption_is_exactly_once() {
        // Promise class: durable invariant with population-side mutations.
        //
        // MEASURED: omitting and repeating issuance of the real
        // CallableCapsuleEscape authority reach distinct exact pre-emission
        // ledger errors.
        // CLAIMED: every planner-interned synthesized crossing is consumed
        // exactly once.
        // THE GAP: the source use must already be consumed so neither control
        // can pass or fail because the source identity class is absent.
        let expr = RuntimeExpr::Construct {
            constructor: "ctor:fixture::BoundaryUse::SynthExact".to_string(),
            args: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
        };
        let plan =
            plan_static_transition_graph_with_test_fixture_boundary_use(&expr, &BTreeMap::new())
                .expect("the exact synthesized-consumption fixture plans");
        let root = plan
            .root_static_origin()
            .expect("the fixture has a root occurrence");
        plan.operand_edge_token(root, 0, SourceOperandRole::ConstructArgument)
            .expect("the independent source authority is consumed");
        plan.aggregate_representation_token(root, BoundaryClass::Constructor, 1)
            .expect("the independent aggregate authority is consumed");
        let synthesized = *plan
            .lowering_boundary_uses
            .iter()
            .find(|use_| {
                use_.edge == LoweringOnlyOperandEdge::TestFixtureResult
                    && use_.origin == root
                    && use_.position == 0
            })
            .expect("the result crossing is planner-interned");
        consume_lowering_boundary_uses(&plan, Some(synthesized.identity));
        let missing = plan
            .validate_boundary_use_consumption()
            .expect_err("an unconsumed planned synthesized use must reject");
        assert!(matches!(
            missing,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(ref detail))
                if detail.starts_with(&format!(
                    "boundary-use ledger is not exact; missing={:?};",
                    [synthesized.identity]
                ))
        ));

        plan.lowering_boundary_use_token(
            synthesized.edge,
            synthesized.origin,
            synthesized.position,
        )
        .expect("the synthesized authority is consumed once");
        plan.lowering_boundary_use_token(
            synthesized.edge,
            synthesized.origin,
            synthesized.position,
        )
        .expect("the same issued synthesized authority is consumed again");
        let duplicate = plan
            .validate_boundary_use_consumption()
            .expect_err("repeated synthesized consumption must reject");
        assert!(matches!(
            duplicate,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(ref detail))
                if detail.starts_with(&format!(
                    "boundary-use ledger contains duplicate consumption; duplicates={:?};",
                    [(synthesized.identity, 2)]
                ))
        ));
    }

    #[test]
    fn static_recursor_worker_residual_matrix_omission_and_reclassification_fail_planning() {
        // Promise class: durable invariant with population-side mutation.
        //
        // MEASURED: deleting or reclassifying the real member makes plan
        // construction return the exact population-closure error.
        // CLAIMED: omission cannot fall through to lowering's late Closure
        // refusal.
        // THE GAP: the fixture must actually contribute a member; the positive
        // control above and the pre-mutation assertion below establish that.
        let expr = static_recursor_worker_fixture("ctor:fixture::Worker::Left");
        for mutation in [
            StaticRecursorResidualMatrixMutation::OmitFirst,
            StaticRecursorResidualMatrixMutation::ReclassifyFirst,
        ] {
            STATIC_RECURSOR_RESIDUAL_MATRIX_MUTATION.with(|cell| cell.set(mutation));
            let result = plan_static_transition_graph(&expr, &BTreeMap::new());
            STATIC_RECURSOR_RESIDUAL_MATRIX_MUTATION
                .with(|cell| cell.set(StaticRecursorResidualMatrixMutation::Exact));
            let error = match result {
                Ok(_) => panic!("the real static recursor member mutation survived planning"),
                Err(error) => error,
            };
            assert!(
                error
                    .to_string()
                    .contains("static recursor worker residual population is not exact"),
                "unexpected pre-emission rejection: {error}"
            );
        }
    }

    #[test]
    fn boundary_use_relation_rejects_duplicate_transplant_owner_phase_and_position() {
        // Promise class: durable invariant with population-side mutations.
        //
        // MEASURED: each mutation changes one field of an actual planned
        // crossing and the independently re-derived relation rejects it.
        // CLAIMED: nominal role/cardinality cannot substitute for exact
        // producer, consumer, path, phase, Need, and Avail agreement.
        // THE GAP: emitted exact-once consumption is a separate lowering
        // ledger control; this pin covers the planner-side operand only.
        let expr = static_recursor_worker_fixture("ctor:fixture::Worker::Ledger");
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("boundary-use mutation fixture plans");
        let edge_index = plan
            .operand_edges
            .iter()
            .position(|edge| {
                edge.role == SourceOperandRole::ConstructArgument
                    && edge.disposition == OperandEdgeDisposition::CallableCapture
            })
            .expect("fixture has a real constructor crossing");

        let mut duplicate = plan.clone();
        duplicate
            .operand_edges
            .push(duplicate.operand_edges[edge_index]);
        assert!(duplicate.validate().is_err());

        let mut transplant = plan.clone();
        transplant.operand_edges[edge_index].child = StaticOriginId(u32::MAX);
        assert!(transplant.validate().is_err());

        let mut wrong_owner = plan.clone();
        wrong_owner.operand_edges[edge_index].producer_owner = PredeclaredFunctionId(u32::MAX);
        assert!(wrong_owner.validate().is_err());

        let mut wrong_phase = plan.clone();
        wrong_phase.operand_edges[edge_index].consumer_phase = BoundaryUsePhase::OperationalCarrier;
        assert!(wrong_phase.validate().is_err());

        let mut wrong_position = plan.clone();
        wrong_position.operand_edges[edge_index].position = u32::MAX;
        assert!(wrong_position.validate().is_err());

        let mut wrong_need = plan.clone();
        wrong_need.operand_edges[edge_index].need = BoundaryUseNeed::ReadSpecializedTemplate;
        assert!(wrong_need.validate().is_err());

        let mut wrong_avail = plan;
        wrong_avail.operand_edges[edge_index].avail = BoundaryUseAvail::SpecializedTemplate;
        assert!(wrong_avail.validate().is_err());
    }

    #[test]
    fn same_shape_static_recursor_workers_keep_distinct_body_targets() {
        // Promise class: durable invariant.
        //
        // MEASURED: two same-arity, same-capture-shape workers in one plan have
        // distinct body origins and residual identities.
        // CLAIMED: body identity, not capture shape or values, selects the
        // direct worker target.
        // THE GAP: emitted result parity independently proves those distinct
        // static targets are called.
        let expr = RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            then_expr: Box::new(static_recursor_worker_fixture("ctor:fixture::Worker::Left")),
            else_expr: Box::new(static_recursor_worker_fixture(
                "ctor:fixture::Worker::Right",
            )),
        };
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new())
            .expect("both static recursor workers plan");
        let root = plan.root_occurrence.expect("if has a root occurrence");
        let worker = |position| {
            let parent = plan.child_static_origin(root, position).unwrap();
            let scrutinee = plan.child_static_origin(parent, 0).unwrap();
            let closure = plan.child_static_origin(scrutinee, 0).unwrap();
            let body = plan.child_static_origin(closure, 0).unwrap();
            plan.static_recursor_worker_residual_token(parent, 0, body)
                .unwrap()
                .unwrap()
        };
        let left = worker(1);
        let right = worker(2);
        assert_eq!(left.declared_arity, right.declared_arity);
        assert_eq!(left.capture_count, right.capture_count);
        assert_ne!(left.body_origin, right.body_origin);
        assert_ne!(left.id, right.id);
    }

    fn nested_resource_bracket(depth: usize) -> RuntimeExpr {
        governed_nested_resource_bracket(depth)
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum GovernedBracketRole {
        AllocatedBuffer,
        ScopeArgument,
        InductionHypothesis,
        RecursiveResult,
    }

    fn role_at(index: u32, outer_to_inner: &[GovernedBracketRole]) -> GovernedBracketRole {
        outer_to_inner[outer_to_inner.len() - 1 - index as usize]
    }

    fn assert_governed_bracket_shape(expr: &RuntimeExpr, depth: usize) {
        if depth == 0 {
            assert!(matches!(
                expr,
                RuntimeExpr::Construct { constructor, args }
                    if constructor == "ctor:prelude::Unit::MkUnit" && args.is_empty()
            ));
            return;
        }

        let RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } = expr
        else {
            panic!("depth {depth} is not allocation-result match");
        };
        assert!(matches!(
            scrutinee.as_ref(),
            RuntimeExpr::Effect {
                operation: ken_host::HostOpV1::BufferAllocate,
                capability: None,
                args,
                ..
            } if matches!(
                args.as_slice(),
                [RuntimeExpr::Value(RuntimeValue::Int(value))] if *value == 1.into()
            )
        ));
        assert_eq!(
            cases.len(),
            2,
            "allocation match lost a trap or success arm"
        );
        assert!(matches!(
            &cases[0],
            RuntimeMatchCase {
                constructor,
                binders: 1,
                body: RuntimeExpr::Trap(_),
            } if constructor == "ctor:prelude::Result::Err"
        ));
        assert_eq!(default.message, "allocate result");

        let RuntimeMatchCase {
            constructor,
            binders: 1,
            body:
                RuntimeExpr::ComputationalMatch {
                    scrutinee,
                    cases,
                    default,
                },
        } = &cases[1]
        else {
            panic!("allocation success arm lost its recursive computational match");
        };
        assert_eq!(constructor, "ctor:prelude::Result::Ok");
        assert_eq!(default.message, "bracket scope");
        assert_eq!(
            cases.len(),
            1,
            "recursive computational match is not closed"
        );

        let RuntimeExpr::Construct { constructor, args } = scrutinee.as_ref() else {
            panic!("recursive scrutinee is not the governed Scope constructor");
        };
        assert_eq!(constructor, "ctor:fixture::Bracket::Scope");
        let [RuntimeExpr::LexicalClosure {
            captures,
            params,
            body,
        }] = args.as_slice()
        else {
            panic!("Scope does not carry exactly one lexical closure");
        };
        assert!(captures.is_empty());
        assert_eq!(params, &["buffer"]);

        let RuntimeComputationalMatchCase {
            constructor,
            argument_binders: 1,
            recursive_positions,
            body: case_body,
        } = &cases[0]
        else {
            panic!("recursive Scope arm changed binder arity");
        };
        assert_eq!(constructor, "ctor:fixture::Bracket::Scope");
        assert_eq!(recursive_positions, &[0]);
        let RuntimeExpr::Call { callee, args } = case_body else {
            panic!("recursive Scope arm no longer invokes its induction hypothesis");
        };
        let (RuntimeExpr::Var(callee), [RuntimeExpr::Var(argument)]) =
            (callee.as_ref(), args.as_slice())
        else {
            panic!("induction-hypothesis call lost its two semantic binder roles");
        };
        let case_roles = [
            GovernedBracketRole::AllocatedBuffer,
            GovernedBracketRole::ScopeArgument,
            GovernedBracketRole::InductionHypothesis,
        ];
        assert_eq!(
            role_at(*callee, &case_roles),
            GovernedBracketRole::InductionHypothesis
        );
        assert_eq!(
            role_at(*argument, &case_roles),
            GovernedBracketRole::AllocatedBuffer,
            "the induction-hypothesis argument is not the allocation result"
        );

        let RuntimeExpr::Let {
            value: recursive_body,
            body: release,
        } = body.as_ref()
        else {
            panic!("lexical closure lost its recursive-before-release ordering");
        };
        assert_governed_bracket_shape(recursive_body, depth - 1);

        let RuntimeExpr::Match {
            scrutinee,
            cases,
            default,
        } = release.as_ref()
        else {
            panic!("lexical closure release is not a result match");
        };
        assert_eq!(default.message, "release result");
        assert_eq!(cases.len(), 2, "release match lost a trap or success arm");
        assert!(matches!(
            &cases[0],
            RuntimeMatchCase {
                constructor,
                binders: 1,
                body: RuntimeExpr::Trap(_),
            } if constructor == "ctor:prelude::Result::Err"
        ));
        assert!(matches!(
            &cases[1],
            RuntimeMatchCase {
                constructor,
                binders: 1,
                body: RuntimeExpr::Construct {
                    constructor: unit,
                    args,
                },
            } if constructor == "ctor:prelude::Result::Ok"
                && unit == "ctor:prelude::Unit::MkUnit"
                && args.is_empty()
        ));

        let RuntimeExpr::Effect {
            operation: ken_host::HostOpV1::BufferFreeze,
            capability: None,
            args,
            ..
        } = scrutinee.as_ref()
        else {
            panic!("release scrutinee is not BufferFreeze");
        };
        let [RuntimeExpr::Var(buffer), RuntimeExpr::Value(RuntimeValue::Int(start)), RuntimeExpr::Value(RuntimeValue::Int(length)), RuntimeExpr::Var(span_origin)] =
            args.as_slice()
        else {
            panic!("BufferFreeze does not have its canonical four operands");
        };
        let release_roles = [
            GovernedBracketRole::AllocatedBuffer,
            GovernedBracketRole::RecursiveResult,
        ];
        assert_eq!(
            role_at(*buffer, &release_roles),
            GovernedBracketRole::AllocatedBuffer
        );
        assert_eq!(
            role_at(*span_origin, &release_roles),
            GovernedBracketRole::AllocatedBuffer
        );
        assert_eq!(
            buffer, span_origin,
            "resource seats do not name the same closure parameter"
        );
        assert_eq!(*start, 0.into());
        assert_eq!(*length, 1.into());
    }

    #[test]
    fn governed_nested_bracket_uses_canonical_four_seat_binder_roles() {
        for depth in 3..=7 {
            assert_governed_bracket_shape(&nested_resource_bracket(depth), depth);
        }
    }

    fn assert_fixed_helper_identity_shape(key: PlannedHelperKey) {
        fn require_copy<T: Copy>() {}
        require_copy::<PlannedHelperKey>();
        match key {
            PlannedHelperKey::Node(_transition, StaticNodeId(_ordinal)) => {}
            PlannedHelperKey::Edge(_kind, StaticEdgeId(_ordinal)) => {}
        }
    }

    fn census(depth: usize) -> BoundaryACensus {
        let expr = nested_resource_bracket(depth);
        plan_static_transition_graph(&expr, &BTreeMap::new())
            .map(|plan| {
                for key in &plan.planned_helpers {
                    assert_fixed_helper_identity_shape(*key);
                }
                plan.census()
            })
            .unwrap_or_else(|error| {
                panic!("RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine n={depth}: {error}")
            })
    }

    fn values(rows: &[BoundaryACensus], field: impl Fn(&BoundaryACensus) -> usize) -> Vec<isize> {
        rows.iter().map(|row| field(row) as isize).collect()
    }

    fn differences(values: &[isize]) -> (Vec<isize>, Vec<isize>) {
        let first = values.windows(2).map(|v| v[1] - v[0]).collect::<Vec<_>>();
        let second = first.windows(2).map(|v| v[1] - v[0]).collect::<Vec<_>>();
        (first, second)
    }

    fn semantic_census(depth: usize) -> (BoundaryACensus, BoundaryB1Census) {
        let expr = nested_resource_bracket(depth);
        plan_static_transition_graph(&expr, &BTreeMap::new())
            .map(|plan| (plan.census(), plan.semantic_census()))
            .unwrap_or_else(|error| {
                panic!("RT_NATIVE_FNSPLIT_B1 could_not_determine n={depth}: {error}")
            })
    }

    fn semantic_values(
        rows: &[BoundaryB1Census],
        field: impl Fn(&BoundaryB1Census) -> usize,
    ) -> Vec<isize> {
        rows.iter().map(|row| field(row) as isize).collect()
    }

    fn index_of_edge_helper(plan: &StaticTransitionPlan, edge: StaticEdgeId) -> usize {
        plan.planned_helpers
            .iter()
            .position(|helper| matches!(helper, PlannedHelperKey::Edge(_, id) if *id == edge))
            .expect("edge has a planned helper")
    }

    fn rewrite_edge(
        plan: &mut StaticTransitionPlan,
        edge: StaticEdgeId,
        from: StaticNodeId,
        to: StaticNodeId,
        kind: EdgeKind,
    ) {
        let index = edge.0 as usize;
        plan.edges[index] = StaticEdge {
            id: edge,
            from,
            to,
            kind,
        };
        plan.evidence[index] = EdgeEvidence {
            edge: edge.0,
            owner: plan.nodes[from.0 as usize].owner,
            from,
            to,
            kind,
        };
        let helper = index_of_edge_helper(plan, edge);
        plan.planned_helpers[helper] = PlannedHelperKey::edge(kind, edge);
    }

    fn append_edge(
        plan: &mut StaticTransitionPlan,
        from: StaticNodeId,
        to: StaticNodeId,
        kind: EdgeKind,
    ) {
        let id = StaticEdgeId(plan.edges.len() as u32);
        plan.edges.push(StaticEdge { id, from, to, kind });
        plan.evidence.push(EdgeEvidence {
            edge: id.0,
            owner: plan.nodes[from.0 as usize].owner,
            from,
            to,
            kind,
        });
        plan.planned_helpers.push(PlannedHelperKey::edge(kind, id));
    }

    #[test]
    fn boundary_b1_nested_resource_brackets_n3_through_n7_are_closed_and_affine() {
        // Promise class: durable invariant. The finite differences corroborate
        // the builder's structural one-node/one-edge/one-flattening traversal;
        // they are not the asymptotic proof.
        let rows = (3..=7).map(semantic_census).collect::<Vec<_>>();
        let outer = rows
            .iter()
            .map(|(outer, _)| outer.clone())
            .collect::<Vec<_>>();
        let semantic = rows
            .iter()
            .map(|(_, semantic)| semantic.clone())
            .collect::<Vec<_>>();

        for (depth, (outer, row)) in (3..=7).zip(&rows) {
            eprintln!(
                "RT_NATIVE_FNSPLIT_B1 n={depth} opcode_vocabulary={} origins={} \
                 ir_records={} semantic_edges={} function_units={} \
                 definitions_per_origin={} operand_elements={} duplicate_origins={} \
                 clone_count={} max_definitions_per_origin={} fixed_k={} \
                 descriptor_bytes={} program_bytes={} record_bytes={} \
                 operand_element_bytes={} capture_layout_bytes={} capture_slot_bytes={} \
                 ruled_child_bytes={} function_bytes={}",
                row.opcode_vocabulary,
                row.distinct_origins,
                row.ir_records,
                row.semantic_edges,
                row.function_units,
                row.definitions_per_origin,
                row.all_out_of_line_operand_elements,
                row.duplicate_origin_definitions,
                row.post_origin_clones,
                row.max_definitions_per_origin,
                outer.max_helpers_per_static_source,
                row.descriptor_bytes,
                row.program_bytes,
                row.record_bytes,
                row.operand_element_bytes,
                row.capture_layout_bytes,
                row.capture_slot_bytes,
                row.ruled_child_bytes,
                row.function_bytes,
            );
        }

        for (name, metric) in [
            (
                "distinct_origins",
                semantic_values(&semantic, |row| row.distinct_origins),
            ),
            (
                "ir_records",
                semantic_values(&semantic, |row| row.ir_records),
            ),
            (
                "semantic_edges",
                semantic_values(&semantic, |row| row.semantic_edges),
            ),
            (
                "function_units",
                semantic_values(&semantic, |row| row.function_units),
            ),
            (
                "all_out_of_line_operand_elements",
                semantic_values(&semantic, |row| row.all_out_of_line_operand_elements),
            ),
        ] {
            let (first, second) = differences(&metric);
            eprintln!(
                "RT_NATIVE_FNSPLIT_B1_DIFF metric={name} values={metric:?} \
                 first={first:?} second={second:?}"
            );
            assert!(
                second.iter().all(|difference| *difference == 0),
                "{name} is not affine across n=3..7"
            );
        }

        for (name, metric) in [
            (
                "opcode_vocabulary",
                semantic_values(&semantic, |row| row.opcode_vocabulary),
            ),
            (
                "definitions_per_origin",
                semantic_values(&semantic, |row| row.definitions_per_origin),
            ),
            (
                "max_definitions_per_origin",
                semantic_values(&semantic, |row| row.max_definitions_per_origin),
            ),
            (
                "duplicate_origin_definitions",
                semantic_values(&semantic, |row| row.duplicate_origin_definitions),
            ),
            (
                "post_origin_clones",
                semantic_values(&semantic, |row| row.post_origin_clones),
            ),
            (
                "descriptor_bytes",
                semantic_values(&semantic, |row| row.descriptor_bytes),
            ),
            (
                "program_bytes",
                semantic_values(&semantic, |row| row.program_bytes),
            ),
            (
                "record_bytes",
                semantic_values(&semantic, |row| row.record_bytes),
            ),
            (
                "operand_element_bytes",
                semantic_values(&semantic, |row| row.operand_element_bytes),
            ),
            (
                "capture_layout_bytes",
                semantic_values(&semantic, |row| row.capture_layout_bytes),
            ),
            (
                "capture_slot_bytes",
                semantic_values(&semantic, |row| row.capture_slot_bytes),
            ),
            (
                "ruled_child_bytes",
                semantic_values(&semantic, |row| row.ruled_child_bytes),
            ),
            (
                "function_bytes",
                semantic_values(&semantic, |row| row.function_bytes),
            ),
        ] {
            let (first, second) = differences(&metric);
            eprintln!(
                "RT_NATIVE_FNSPLIT_B1_DIFF metric={name} values={metric:?} \
                 first={first:?} second={second:?}"
            );
            assert!(
                metric.windows(2).all(|pair| pair[0] == pair[1]),
                "{name} is not pairwise constant across n=3..7"
            );
        }

        let fixed_k = outer
            .iter()
            .map(|row| row.max_helpers_per_static_source as isize)
            .collect::<Vec<_>>();
        let (fixed_k_first, fixed_k_second) = differences(&fixed_k);
        eprintln!(
            "RT_NATIVE_FNSPLIT_B1_DIFF metric=fixed_k values={fixed_k:?} \
             first={fixed_k_first:?} second={fixed_k_second:?}"
        );
        assert_eq!(
            fixed_k,
            vec![8, 8, 8, 8, 8],
            "B1 grew or obscured the already-full outer helper inventory"
        );
        assert!(semantic.iter().all(|row| {
            row.opcode_vocabulary == 6
                && row.definitions_per_origin == 1
                && row.max_definitions_per_origin == 1
                && row.duplicate_origin_definitions == 0
                && row.post_origin_clones == 0
        }));
    }

    #[test]
    fn boundary_b1_preserves_equal_occurrences_and_reuses_one_activation_program() {
        // Promise class: durable invariant. Equal source text is the
        // discriminating counterexample to semantic hash-consing.
        let equal_occurrences = RuntimeExpr::If {
            scrutinee: Box::new(unit()),
            then_expr: Box::new(unit()),
            else_expr: Box::new(unit()),
        };
        let plan = plan_static_transition_graph(&equal_occurrences, &BTreeMap::new()).unwrap();
        let equal_nodes = plan
            .semantic_sources
            .iter()
            .filter_map(|source| {
                (source.source == SemanticSourceKind::Expression(RuntimeExprShape::Construct))
                    .then_some(source.planned_node)
            })
            .collect::<Vec<_>>();
        assert_eq!(equal_nodes.len(), 3);
        let descriptors = equal_nodes
            .iter()
            .map(|node| plan.semantic.descriptors[node.0 as usize])
            .collect::<Vec<_>>();
        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| descriptor.origin)
                .collect::<BTreeSet<_>>()
                .len(),
            3,
            "equal source occurrences were semantic-hash-consed"
        );
        assert_eq!(
            descriptors
                .iter()
                .map(|descriptor| descriptor.program)
                .collect::<BTreeSet<_>>()
                .len(),
            3,
            "equal source occurrences lost positional programs"
        );
        let records = descriptors
            .iter()
            .map(|descriptor| plan.semantic.records[descriptor.program.0 as usize])
            .collect::<Vec<_>>();
        assert!(records.windows(2).all(|pair| {
            pair[0].opcode == pair[1].opcode
                && pair[0].operands.len == pair[1].operands.len
                && pair[0].origin != pair[1].origin
        }));

        let node = equal_nodes[0];
        let static_node = plan.nodes[node.0 as usize];
        let other_activation = plan
            .nodes
            .iter()
            .map(|candidate| candidate.frame)
            .find(|frame| *frame != static_node.frame)
            .expect("fixture has another closed activation frame");
        let descriptor_before = plan.semantic.descriptors[node.0 as usize];
        assert_eq!(
            plan.helper_key_for_activation(node, static_node.frame)
                .unwrap(),
            plan.helper_key_for_activation(node, other_activation)
                .unwrap()
        );
        assert_eq!(
            plan.semantic.descriptors[node.0 as usize], descriptor_before,
            "another activation minted a program or origin"
        );
    }

    #[test]
    fn boundary_b1_semantics_are_discovery_order_and_dynamic_state_independent() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let mut reversed_sources = plan.semantic_sources.clone();
        reversed_sources.reverse();
        let mut reordered = build_semantic_plane(
            &plan.nodes,
            &plan.edges,
            &plan.entries,
            &reversed_sources,
            &plan.semantic_material,
        )
        .unwrap();
        let mut reordered_material = plan.semantic_material.clone();
        let (reordered_roles, reordered_io_roles) = build_synthesized_constructor_inventory(
            &mut reordered_material,
            &crate::NativeProcessSymbols::legacy_prelude(),
        )
        .unwrap();
        reordered.install_synthesized_constructor_inventory(reordered_roles, reordered_io_roles);
        assert_eq!(reordered, plan.semantic);

        let mut changed_frames = plan.nodes.clone();
        let frames = plan.nodes.iter().map(|node| node.frame).collect::<Vec<_>>();
        assert!(
            frames.iter().any(|frame| *frame != frames[0]),
            "frame rotation is a no-op: all frames are equal, so this control proves nothing"
        );
        for (index, node) in changed_frames.iter_mut().enumerate() {
            node.frame = frames[(index + 1) % frames.len()];
        }
        let mut changed = build_semantic_plane(
            &changed_frames,
            &plan.edges,
            &plan.entries,
            &reversed_sources,
            &plan.semantic_material,
        )
        .unwrap();
        let mut changed_material = plan.semantic_material.clone();
        let (changed_roles, changed_io_roles) = build_synthesized_constructor_inventory(
            &mut changed_material,
            &crate::NativeProcessSymbols::legacy_prelude(),
        )
        .unwrap();
        changed.install_synthesized_constructor_inventory(changed_roles, changed_io_roles);
        assert_eq!(
            changed, plan.semantic,
            "dynamic activation state changed semantic programs or bodies"
        );
        assert_eq!(plan.semantic.descriptors.len(), plan.nodes.len());
        // RT-FNSPLIT-B2O, re-baselined 2026-07-25 from `plan.nodes.len()`.
        //
        // PREDICTED FROM THE DESIGN BEFORE MEASURING, and this is the reason:
        // the function table is no longer a positional alias of the node table,
        // so it is seed-exact rather than node-exact. The unit set is
        // `plan.entries` (root plus each transparent declaration) union every
        // `EdgeKind::StaticBody` target (each retained closure-body entry), and
        // those two classes are disjoint, so the count is their sum.
        //
        // ⛔ Asserted RELATIONALLY against the two seed classes, never against
        // the observed number. A count re-fit to whatever the code now emits
        // measures nothing; this form goes red if either seed class stops being
        // enumerated, which is the property `D1` actually claims.
        assert_eq!(
            plan.semantic.functions.len(),
            plan.entries.len()
                + plan
                    .edges
                    .iter()
                    .filter(|edge| edge.kind == EdgeKind::StaticBody)
                    .count()
        );
        assert_eq!(plan.semantic.ruled_children.len(), plan.edges.len());
    }

    #[test]
    fn boundary_b1_negative_controls_fail_at_named_semantic_artifacts() {
        // Promise class: durable mutation proof.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let pointer_origins = plan
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                (
                    std::ptr::from_ref(node) as usize,
                    StaticOriginId(((index + 1) % plan.nodes.len()) as u32),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut pointer_recovery = plan.semantic.clone();
        pointer_recovery.descriptors[0].origin =
            pointer_origins[&(std::ptr::from_ref(&plan.nodes[0]) as usize)];
        assert_eq!(
            pointer_recovery
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("descriptor origin is not its preallocated positional identity")
        );

        let equal_occurrences = RuntimeExpr::If {
            scrutinee: Box::new(unit()),
            then_expr: Box::new(unit()),
            else_expr: Box::new(unit()),
        };
        let equal_plan =
            plan_static_transition_graph(&equal_occurrences, &BTreeMap::new()).unwrap();
        let equal_nodes = equal_plan
            .semantic_sources
            .iter()
            .filter_map(|source| {
                (source.source == SemanticSourceKind::Expression(RuntimeExprShape::Construct))
                    .then_some(source.planned_node)
            })
            .collect::<Vec<_>>();
        let mut hash_cons = equal_plan.semantic.clone();
        hash_cons.descriptors[equal_nodes[1].0 as usize].origin =
            hash_cons.descriptors[equal_nodes[0].0 as usize].origin;
        assert_eq!(
            hash_cons
                .validate(
                    &equal_plan.nodes,
                    &equal_plan.edges,
                    &equal_plan.entries,
                    &equal_plan.semantic_sources,
                    &equal_plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic hash-consing merged distinct static origins")
        );

        let mut second_definition = plan.semantic.clone();
        second_definition
            .descriptors
            .push(second_definition.descriptors[0]);
        assert_eq!(
            second_definition
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("planned node has more than one semantic definition")
        );

        let mut post_origin_clone = plan.semantic.clone();
        post_origin_clone
            .programs
            .push(post_origin_clone.programs[0]);
        assert_eq!(
            post_origin_clone
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic program arena contains a post-origin clone")
        );

        let mut superlinear_material = plan.semantic.clone();
        let deliberate_square = plan.nodes.len().checked_mul(plan.nodes.len()).unwrap();
        superlinear_material
            .operands
            .extend(
                (0..deliberate_square).map(|ordinal| SemanticOperandElement {
                    kind: SemanticAtomKind::LocalIndex,
                    content: DenseRange { start: 0, len: 0 },
                    payload: ordinal as u64,
                }),
            );
        superlinear_material.records[0].operands.len = superlinear_material.records[0]
            .operands
            .len
            .checked_add(deliberate_square as u32)
            .unwrap();
        assert_eq!(
            superlinear_material
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic operand arena exceeds the one-visit source-material budget")
        );
    }

    /// Two occurrences of the same shape whose material differs. `Var(0)` and
    /// `Var(1)` agree on shape, opcode, atom count and child count, so shape and
    /// count checks cannot separate them: only occurrence-exact material can.
    fn equal_shaped_atom_fixture() -> RuntimeExpr {
        RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::Var(0)),
            body: Box::new(RuntimeExpr::Var(1)),
        }
    }

    /// Three occurrences that are **equal as terms**. A content or hash lookup
    /// cannot tell them apart; their origins can.
    fn content_equal_occurrences() -> RuntimeExpr {
        RuntimeExpr::If {
            scrutinee: Box::new(unit()),
            then_expr: Box::new(unit()),
            else_expr: Box::new(unit()),
        }
    }

    /// `RT-FNSPLIT-B2A-S` D6 — the occurrence table's negative controls, each red
    /// at its **own named artifact**.
    ///
    /// ⛔ The four expected errors are deliberately distinct. A single "the table
    /// is invalid" verdict would be discharged by any one of these mutations, so
    /// it could not tell a swapped entry from a missing one from a surplus one —
    /// and the whole point of storing each entry's origin beside its term is that
    /// those failures are distinguishable.
    #[test]
    fn occurrence_table_negative_controls_fail_at_named_artifacts() {
        // Promise class: durable mutation proof.
        let expr = equal_shaped_atom_fixture();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let vars = nodes_of_shape(&plan, RuntimeExprShape::Var);
        assert_eq!(
            vars.len(),
            2,
            "fixture must hold two equal-shaped occurrences"
        );

        // Control 1 — SWAP two equal-shaped entries. Each now sits at an index
        // that is not its stored origin, so the wrong-body substitution is
        // REFUSED rather than performed. This is the control that matters: the
        // pair agrees on shape and counts, so nothing but the stored origin
        // distinguishes them.
        let mut swapped = plan.clone();
        swapped
            .source_occurrences
            .swap(vars[0].0 as usize, vars[1].0 as usize);
        assert_eq!(
            swapped.validate_source_occurrence_table().unwrap_err(),
            planner_error("occurrence table entry is filed under an origin that is not its index")
        );
        // ⭐ And the lookup refuses on its own, not only via the whole-plan
        // validator — a consumer cannot reach a swapped body even in a plan that
        // was never re-validated.
        assert_eq!(
            swapped.source_occurrence(origin_of(vars[0])).unwrap_err(),
            planner_error("planned occurrence's stored origin disagrees with its table position")
        );

        // Control 2 — MISSING: a control node is a planned node with no source
        // term, so its slot is legitimately empty and a lookup on it is loud
        // rather than a substituted neighbour.
        assert_eq!(
            plan.source_occurrence(origin_of(plan.terminal_id()))
                .unwrap_err(),
            planner_error("static origin names no planned source occurrence")
        );

        // Control 3 — OUT OF RANGE: past the end of the planned population.
        assert_eq!(
            plan.source_occurrence(StaticOriginId(plan.nodes.len() as u32 + 7))
                .unwrap_err(),
            planner_error("static origin is outside the planned occurrence table")
        );

        // Control 4 — SURPLUS/DUPLICATE: an entry no semantic seed accounts for.
        // Well-formed in isolation (its stored origin *is* its index), so only the
        // cross-check against the independently produced seed population sees it.
        let mut surplus = plan.clone();
        let terminal = plan.terminal_id();
        surplus.source_occurrences[terminal.0 as usize] = Some(PlannedOccurrence {
            static_origin: origin_of(terminal),
            expr: &expr,
        });
        assert_eq!(
            surplus.validate_source_occurrence_table().unwrap_err(),
            planner_error("occurrence table holds an entry no semantic seed accounts for")
        );
    }

    /// `RT-FNSPLIT-B2A-S` D6/AC-3 — **identity is the ordinal, not the content.**
    ///
    /// ⭐ This is the chain's predicate as an executable test. The fixture's three
    /// occurrences are equal as terms, so a content or hash lookup would have to
    /// pick one of them arbitrarily — while the tag resolves each to its own
    /// occurrence. If this test ever passes with the origins compared equal, a
    /// dynamic property has started naming static code again.
    #[test]
    fn content_equal_occurrences_resolve_to_distinct_occurrences() {
        // Promise class: durable invariant.
        let expr = content_equal_occurrences();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let units = nodes_of_shape(&plan, RuntimeExprShape::Construct);
        // ⚠ Load-bearing: without it the loop below is vacuous and this test
        // passes while checking nothing at all.
        assert_eq!(units.len(), 3, "fixture must hold three equal terms");

        let resolved = units
            .iter()
            .map(|node| {
                (
                    origin_of(*node),
                    plan.source_occurrence(origin_of(*node)).unwrap(),
                )
            })
            .collect::<Vec<_>>();

        for (index, (origin, term)) in resolved.iter().enumerate() {
            for (other_origin, other_term) in resolved.iter().skip(index + 1) {
                assert_ne!(origin, other_origin, "each occurrence has its own origin");
                // ⛔ `RuntimeExpr: PartialEq` is gone (`D2`), and a `Debug`-text
                // proxy is barred. The claim here does not need term-to-term
                // comparison at all: the fixture builds all three occurrences
                // from `unit()`, so *"a CONTENT lookup could not have told them
                // apart"* is established by asserting each resolves to that one
                // known content — a direct property, checked against a value
                // this test states rather than against its sibling.
                for resolved in [term, other_term] {
                    let RuntimeExpr::Construct { constructor, args } = resolved else {
                        panic!("occurrence resolved to {resolved:?}, not a Construct");
                    };
                    assert_eq!(constructor, "ctor:prelude::Unit::MkUnit");
                    assert!(args.is_empty(), "unit takes no arguments");
                }
                assert!(
                    !std::ptr::eq(*term, *other_term),
                    "distinct occurrences resolve to distinct subterms"
                );
            }
        }
    }

    /// `RT-FNSPLIT-B2A-S` D6/AC-3 — perturbing the borrowed **address** while the
    /// ordinal mapping is unchanged does not move any identity.
    ///
    /// Two structurally equal source trees at different addresses plan to the same
    /// origins, and each plan resolves its own origins into its own tree. So the
    /// table's key is the planner's ordinal and the borrow is payload — which is
    /// exactly what makes a lifetime admissible here rather than dangerous.
    #[test]
    fn a_source_tree_at_a_different_address_yields_identical_origins() {
        // Promise class: durable invariant.
        let first = equal_shaped_child_fixture();
        let second = equal_shaped_child_fixture();
        assert!(
            !std::ptr::eq(&first, &second),
            "the two fixtures must live at different addresses"
        );

        let first_plan = plan_static_transition_graph(&first, &BTreeMap::new()).unwrap();
        let second_plan = plan_static_transition_graph(&second, &BTreeMap::new()).unwrap();

        let origins = |plan: &StaticTransitionPlan<'_>| {
            plan.semantic_sources
                .iter()
                .map(|seed| (seed.origin, seed.source))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            origins(&first_plan),
            origins(&second_plan),
            "identity must not depend on where the source tree happens to live"
        );

        // And the payload follows the borrow it was planned from, rather than
        // leaking across plans.
        for (origin, _) in origins(&first_plan) {
            if let Ok(term) = first_plan.source_occurrence(origin) {
                let other = second_plan.source_occurrence(origin).unwrap();
                // ⛔ This asserted only `discriminant` equality, which CANNOT
                // establish the property: it passes if `Var(0)` were resolved
                // as `Var(3)`, or if the two equal-shaped `Let` children were
                // exchanged — the exact occurrence-identity defects this
                // fixture exists to catch. Recursive comparison is genuinely
                // required here, through the closure-refusing witness.
                let (lhs, rhs) = (fixture_witness(term), fixture_witness(other));
                assert!(
                    lhs.is_some() && rhs.is_some(),
                    "both occurrences must lie in the fixture grammar; a \
                     refusal is a failure, not a skip"
                );
                assert_eq!(
                    lhs, rhs,
                    "equal trees resolve to structurally identical terms, \
                     including every Var index and child position"
                );
                assert!(
                    !std::ptr::eq(term, other),
                    "but each plan resolves into its own tree"
                );
            }
        }
    }

    /// `RT-FNSPLIT-B2A-S` AC-2 — the table is **total** over the planned expression
    /// population, positively and not by the absence of a failure.
    #[test]
    fn the_occurrence_table_is_total_over_every_planned_expression() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let mut expressions = 0usize;
        for seed in &plan.semantic_sources {
            match seed.source {
                SemanticSourceKind::Expression(_) => {
                    expressions += 1;
                    plan.source_occurrence(seed.origin).expect(
                        "every planned expression occurrence resolves through its own origin",
                    );
                }
                SemanticSourceKind::Control(_) => {
                    plan.source_occurrence(seed.origin)
                        .expect_err("a control node has no source term to resolve");
                }
            }
        }
        assert!(
            expressions > 1,
            "the fixture must plan more than one expression for totality to mean anything"
        );
        assert_eq!(
            expressions,
            plan.source_occurrences
                .iter()
                .filter(|slot| slot.is_some())
                .count(),
            "the table holds exactly one entry per planned expression occurrence"
        );
    }

    /// Two `Let` occurrences of identical shape and counts whose positional
    /// children are different occurrences.
    /// ⛔ **Test-local, closure-REFUSING witness for exactly this fixture's
    /// grammar** — `If`, the unit `Construct`, `Let`, and `Var(index)`.
    ///
    /// `D2` removed `RuntimeExpr: PartialEq` because it reached
    /// `RuntimeValue::ClosureRef`. The address-independence control below
    /// genuinely needs **recursive** comparison — `discriminant` cannot express
    /// it — so this is the narrow route the Architect's ruling permits: an input
    /// grammar that **refuses** anything closure-capable before producing a
    /// verdict.
    ///
    /// ⛔ Deliberately NOT a shared `RuntimeExpr` projection. It lives in this
    /// test module, covers four forms, and every other variant — including
    /// `Closure`, `LexicalClosure`, and `Value(ClosureRef)` — returns `None`.
    /// `None` is a **refusal that fails the test**, never a skip.
    #[derive(Debug, PartialEq, Eq)]
    enum FixtureWitness {
        Unit,
        Var(u32),
        Let(Box<FixtureWitness>, Box<FixtureWitness>),
        If(
            Box<FixtureWitness>,
            Box<FixtureWitness>,
            Box<FixtureWitness>,
        ),
    }

    fn fixture_witness(expr: &RuntimeExpr) -> Option<FixtureWitness> {
        Some(match expr {
            RuntimeExpr::Construct { constructor, args }
                if constructor == "ctor:prelude::Unit::MkUnit" && args.is_empty() =>
            {
                FixtureWitness::Unit
            }
            RuntimeExpr::Var(index) => FixtureWitness::Var(*index),
            RuntimeExpr::Let { value, body } => FixtureWitness::Let(
                Box::new(fixture_witness(value)?),
                Box::new(fixture_witness(body)?),
            ),
            RuntimeExpr::If {
                scrutinee,
                then_expr,
                else_expr,
            } => FixtureWitness::If(
                Box::new(fixture_witness(scrutinee)?),
                Box::new(fixture_witness(then_expr)?),
                Box::new(fixture_witness(else_expr)?),
            ),
            // ⛔ Every other form REFUSES. Closure-bearing ones are the reason
            // this arm exists, and `p2_the_fixture_witness_refuses_closures`
            // is the negative control proving they cannot slip through.
            _ => return None,
        })
    }

    /// ⚠ **NEGATIVE CONTROL for [`fixture_witness`]** — without it, "the
    /// witnesses compared equal" and "the witness silently admitted a closure"
    /// are indistinguishable.
    #[test]
    fn p2_the_fixture_witness_refuses_closures() {
        // Promise class: durable invariant.
        assert!(
            fixture_witness(&RuntimeExpr::Closure {
                captures: vec![],
                params: vec!["x".to_string()],
                body: Box::new(RuntimeExpr::Var(0)),
            })
            .is_none(),
            "a Closure must not produce a witness"
        );
        assert!(
            fixture_witness(&RuntimeExpr::Value(RuntimeValue::ClosureRef {
                symbol: "decl:fixture::f".to_string(),
                captured: vec![],
            }))
            .is_none(),
            "a ClosureRef value must not produce a witness"
        );
        // ⛔ And transitively: a closure NESTED inside admitted grammar refuses
        // the whole tree, rather than the parent succeeding around it.
        assert!(
            fixture_witness(&RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Closure {
                    captures: vec![],
                    params: vec!["x".to_string()],
                    body: Box::new(RuntimeExpr::Var(0)),
                }),
                body: Box::new(RuntimeExpr::Var(0)),
            })
            .is_none(),
            "refusal is transitive through admitted parents"
        );

        // ⚠ POSITIVE CONTROL — the fixture grammar itself DOES produce a
        // witness, so the three refusals above are not a witness that refuses
        // everything.
        assert!(
            fixture_witness(&equal_shaped_child_fixture()).is_some(),
            "the fixture grammar produces a witness"
        );
    }

    fn equal_shaped_child_fixture() -> RuntimeExpr {
        RuntimeExpr::If {
            scrutinee: Box::new(unit()),
            then_expr: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Var(0)),
                body: Box::new(RuntimeExpr::Var(1)),
            }),
            else_expr: Box::new(RuntimeExpr::Let {
                value: Box::new(RuntimeExpr::Var(2)),
                body: Box::new(RuntimeExpr::Var(3)),
            }),
        }
    }

    fn nodes_of_shape(plan: &StaticTransitionPlan, shape: RuntimeExprShape) -> Vec<StaticNodeId> {
        plan.semantic_sources
            .iter()
            .filter_map(|source| {
                (source.source == SemanticSourceKind::Expression(shape))
                    .then_some(source.planned_node)
            })
            .collect()
    }

    #[test]
    fn boundary_b1r_control_1_swapping_equal_shaped_occurrence_material_is_rejected() {
        // Promise class: durable mutation proof. This is the load-bearing
        // control: the swapped pair agrees on shape, opcode and every count, so
        // it is exactly the case B1's counted placeholders could not see.
        let expr = equal_shaped_atom_fixture();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let vars = nodes_of_shape(&plan, RuntimeExprShape::Var);
        assert_eq!(
            vars.len(),
            2,
            "fixture must hold two equal-shaped occurrences"
        );

        let first = plan.semantic.records[vars[0].0 as usize];
        let second = plan.semantic.records[vars[1].0 as usize];
        assert_eq!(
            (first.opcode, first.operands.len, first.child_origins.len),
            (second.opcode, second.operands.len, second.child_origins.len),
            "the pair is not equal-shaped, so this control would prove nothing"
        );
        let before = (
            plan.semantic.operands[first.operands.start as usize],
            plan.semantic.operands[second.operands.start as usize],
        );
        assert_ne!(
            before.0, before.1,
            "the pair's material is identical, so a swap is a no-op and this \
             control would prove nothing"
        );

        let mut swapped = plan.semantic.clone();
        for offset in 0..first.operands.len as usize {
            swapped.operands.swap(
                first.operands.start as usize + offset,
                second.operands.start as usize + offset,
            );
        }
        assert_eq!(
            swapped
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic material record is not occurrence-exact for its origin")
        );

        // The same swap on positional children of an equal-shaped pair.
        let child_expr = equal_shaped_child_fixture();
        let child_plan = plan_static_transition_graph(&child_expr, &BTreeMap::new()).unwrap();
        let lets = nodes_of_shape(&child_plan, RuntimeExprShape::Let);
        assert_eq!(lets.len(), 2);
        let first = child_plan.semantic.records[lets[0].0 as usize];
        let second = child_plan.semantic.records[lets[1].0 as usize];
        assert_eq!(
            (first.opcode, first.operands.len, first.child_origins.len),
            (second.opcode, second.operands.len, second.child_origins.len),
            "the pair is not equal-shaped, so this control would prove nothing"
        );
        assert_eq!(first.child_origins.len, 2, "a Let owns value and body");
        let mut swapped_children = child_plan.semantic.clone();
        for offset in 0..first.child_origins.len as usize {
            swapped_children.child_origins.swap(
                first.child_origins.start as usize + offset,
                second.child_origins.start as usize + offset,
            );
        }
        assert_eq!(
            swapped_children
                .validate(
                    &child_plan.nodes,
                    &child_plan.edges,
                    &child_plan.entries,
                    &child_plan.semantic_sources,
                    &child_plan.semantic_material,
                )
                .unwrap_err(),
            planner_error(
                "semantic child origins are not occurrence-exact for their source positions"
            )
        );
    }

    fn primitive_call(symbol: &str, partiality: crate::RuntimePartiality) -> RuntimeExpr {
        RuntimeExpr::PrimitiveCall {
            primitive: crate::RuntimePrimitive {
                symbol: symbol.to_string(),
                partiality,
            },
            args: Vec::new(),
        }
    }

    /// Two `PrimitiveCall` occurrences sharing one symbol and one (empty)
    /// argument shape, differing only in the partiality that lowering branches on.
    fn equal_shaped_primitive_pair(
        left: crate::RuntimePartiality,
        right: crate::RuntimePartiality,
    ) -> RuntimeExpr {
        RuntimeExpr::Let {
            value: Box::new(primitive_call("ken.bytes.at", left)),
            body: Box::new(primitive_call("ken.bytes.at", right)),
        }
    }

    /// Decodes one record's single descriptor atom back out of the closed name
    /// arena, so a control asserts on the material's CONTENT and not on the
    /// incidental fact that two occurrences interned at different offsets.
    fn descriptor_bytes(plan: &StaticTransitionPlan, node: StaticNodeId) -> Vec<u8> {
        let record = plan.semantic.records[node.0 as usize];
        assert_eq!(record.operands.len, 1, "a primitive owns one atom");
        let atom = plan.semantic.operands[record.operands.start as usize];
        assert_eq!(atom.kind, SemanticAtomKind::PrimitiveDescriptor);
        let start = atom.content.start as usize;
        plan.semantic.names[start..start + atom.content.len as usize].to_vec()
    }

    /// Asserts that an equal-shaped primitive pair differing only in partiality
    /// has genuinely different material, and that cross-wiring one occurrence's
    /// descriptor onto the other reddens at occurrence-exactness.
    fn assert_partiality_is_occurrence_exact(
        left: crate::RuntimePartiality,
        right: crate::RuntimePartiality,
        case: &str,
    ) {
        let expr = equal_shaped_primitive_pair(left, right);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let calls = nodes_of_shape(&plan, RuntimeExprShape::PrimitiveCall);
        assert_eq!(calls.len(), 2, "{case}: fixture must hold two occurrences");

        let first = plan.semantic.records[calls[0].0 as usize];
        let second = plan.semantic.records[calls[1].0 as usize];
        assert_eq!(
            (first.opcode, first.operands.len, first.child_origins.len),
            (second.opcode, second.operands.len, second.child_origins.len),
            "{case}: the pair is not equal-shaped, so this control proves nothing"
        );
        assert_ne!(
            descriptor_bytes(&plan, calls[0]),
            descriptor_bytes(&plan, calls[1]),
            "{case}: the two primitives encode identical material, so the plane \
             cannot tell them apart and B2a would emit the wrong behaviour"
        );

        // Cross-wire: point the first occurrence's descriptor at the second's
        // encoded content. Shape, opcode, counts and atom kind all still agree.
        let mut cross_wired = plan.semantic.clone();
        let victim = first.operands.start as usize;
        cross_wired.operands[victim].content =
            cross_wired.operands[second.operands.start as usize].content;
        assert_eq!(
            cross_wired
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic material record is not occurrence-exact for its origin"),
            "{case}: a cross-wired primitive descriptor was not caught"
        );
    }

    #[test]
    fn boundary_b1r_primitive_partiality_is_occurrence_exact_material() {
        // Promise class: durable mutation proof. Partiality changes what
        // lowering emits (immediate trap versus continue, plus distinct
        // constructor/obligation/assumption material), so a symbol-only atom
        // would let these occurrences share one body while lowering differently.
        assert_partiality_is_occurrence_exact(
            crate::RuntimePartiality::Total,
            crate::RuntimePartiality::CheckedTrap {
                obligation: "ken.bytes.at.inBounds".to_string(),
            },
            "distinct partiality variants",
        );

        // A variant-tag-only encoding would pass the case above and fail here:
        // same variant, one differing field.
        assert_partiality_is_occurrence_exact(
            crate::RuntimePartiality::SafeOption {
                none: "None".to_string(),
                some: "Some".to_string(),
                obligation: None,
            },
            crate::RuntimePartiality::SafeOption {
                none: "Nothing".to_string(),
                some: "Some".to_string(),
                obligation: None,
            },
            "same variant, one differing field",
        );

        // The optional field must also discriminate, so its presence byte is
        // load-bearing rather than decorative.
        assert_partiality_is_occurrence_exact(
            crate::RuntimePartiality::SafeOption {
                none: "None".to_string(),
                some: "Some".to_string(),
                obligation: None,
            },
            crate::RuntimePartiality::SafeOption {
                none: "None".to_string(),
                some: "Some".to_string(),
                obligation: Some("ken.bytes.at.inBounds".to_string()),
            },
            "same variant, optional field present versus absent",
        );
    }

    const IDENTITY_CTOR: &str = "ctor:prelude::Pair::MkPair";
    const IDENTITY_OTHER_CTOR: &str = "ctor:prelude::Triple::MkTriple";
    const IDENTITY_FIELD: &str = "field:fst";

    /// A `Match` whose scrutinee **constructs** the same constructor one of its
    /// cases **eliminates**, and whose case body projects a field the record
    /// beneath it **declares**.
    ///
    /// ⭐ The point of the shape is that each spelling appears at **two distinct
    /// occurrences** with different atom kinds — `ConstructorSymbol` vs
    /// `CaseConstructor`, `RecordFieldName` vs `ProjectField`. That is what makes
    /// the equality assertions below non-trivial: they are comparing a
    /// *producer's* identity against a *consumer's*, which is exactly `D2`.
    ///
    /// Positional child layout, verified against the planner's own construction
    /// rather than assumed: `Match` pushes the scrutinee then the case bodies
    /// (`children.push(scrutinee); children.extend(case_bodies)`), and `Project`
    /// plans its record at position 0.
    fn identity_fixture() -> RuntimeExpr {
        RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: IDENTITY_CTOR.to_string(),
                args: Vec::new(),
            }),
            cases: vec![
                RuntimeMatchCase {
                    constructor: IDENTITY_CTOR.to_string(),
                    binders: 0,
                    body: RuntimeExpr::Project {
                        record: Box::new(RuntimeExpr::Record {
                            fields: vec![(IDENTITY_FIELD.to_string(), unit())],
                        }),
                        field: IDENTITY_FIELD.to_string(),
                    },
                },
                RuntimeMatchCase {
                    constructor: IDENTITY_OTHER_CTOR.to_string(),
                    binders: 0,
                    body: unit(),
                },
            ],
            default: trap("identity fixture default"),
        }
    }

    /// `RT-FNSPLIT-C1` `D2` — the producer and the eliminator derive **one**
    /// constructor identity, and different spellings stay distinct.
    ///
    /// **MEASURED:** `constructor_symbol_identity` at the `Construct` occurrence
    /// equals `case_constructor_identity` at the `Match` occurrence, and differs
    /// from the identity of a differently-spelled case.
    /// **CLAIMED:** producer and consumer share one authority (`D2`).
    /// **THE GAP:** the two readings must come from **different occurrences** —
    /// otherwise equality is trivially true of any scheme at all, including the
    /// per-occurrence spans this node replaced. That is asserted, not assumed.
    #[test]
    fn boundary_c1_producer_and_eliminator_share_one_constructor_identity() {
        // Promise class: durable invariant.
        let expr = identity_fixture();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let match_origin = plan.root_static_origin().unwrap();
        let construct_origin = plan.child_static_origin(match_origin, 0).unwrap();

        assert_ne!(
            match_origin, construct_origin,
            "NON-VACUITY: the produced and eliminated identities must be read at \
             two different occurrences, or their equality says nothing about \
             sharing an authority."
        );

        let produced = plan.constructor_symbol_identity(construct_origin).unwrap();
        let eliminated = plan.case_constructor_identity(match_origin, 0).unwrap();
        assert_eq!(
            produced, eliminated,
            "the constructor built at one occurrence and matched at another have \
             different artifact-static identities, so producer and consumer are \
             not sharing one authority"
        );

        // Discriminator: a different spelling must not collide.
        let other = plan.case_constructor_identity(match_origin, 1).unwrap();
        assert_ne!(
            produced, other,
            "two differently-spelled constructors share an identity"
        );
    }

    /// `RT-FNSPLIT-C1` `D2` — the record's declared field and the projection's
    /// selected field are one identity.
    #[test]
    fn boundary_c1_declared_and_projected_field_share_one_identity() {
        // Promise class: durable invariant.
        let expr = identity_fixture();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let match_origin = plan.root_static_origin().unwrap();
        let project_origin = plan.child_static_origin(match_origin, 1).unwrap();
        let record_origin = plan.child_static_origin(project_origin, 0).unwrap();

        assert_ne!(
            project_origin, record_origin,
            "NON-VACUITY: the declared and selected field identities must be read \
             at two different occurrences."
        );

        let selected = plan.project_field_identity(project_origin).unwrap();
        let declared = plan.record_field_identity(record_origin, 0).unwrap();
        assert_eq!(
            selected, declared,
            "the field declared by a record and the field selected by a projection \
             over it have different artifact-static identities"
        );
    }

    /// `RT-FNSPLIT-C1` `D1` — the capability refuses a wrong-kind or
    /// out-of-cardinality access rather than returning a plausible identity.
    #[test]
    fn boundary_c1_identity_capability_refuses_wrong_kind_and_cardinality() {
        // Promise class: durable invariant.
        let expr = identity_fixture();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let match_origin = plan.root_static_origin().unwrap();
        let construct_origin = plan.child_static_origin(match_origin, 0).unwrap();

        // Cardinality: the fixture has two cases, so index 2 does not exist.
        assert!(plan.case_constructor_identity(match_origin, 2).is_err());

        // Wrong kind: a `Construct` occurrence has a `ConstructorSymbol` atom
        // and no `ProjectField` atom, so asking it for a field identity must
        // fail rather than fall back to whatever named atom it does hold.
        assert!(plan.project_field_identity(construct_origin).is_err());

        // The positive direction, so the two refusals above are attributable to
        // the kind/cardinality checks and not to an origin that resolves nothing.
        assert!(plan.constructor_symbol_identity(construct_origin).is_ok());
    }

    /// `RT-FNSPLIT-C1` `D2` — equal name bytes have exactly one canonical span.
    ///
    /// **MEASURED:** across every atom of a real plan, atoms whose interned
    /// bytes are equal have equal `content` spans.
    /// **CLAIMED:** a producer and an eliminator at *different occurrences*
    /// derive the *same* artifact-static identity for the same spelling.
    /// **THE GAP:** the identity must be a function of the span alone — which
    /// is why `pack_identity` is the sole encoding and why the newtypes wrap the
    /// span rather than carrying any second field.
    ///
    /// ⛔ **The non-vacuity guard is the load-bearing half of this test.** A
    /// fixture in which no spelling repeats satisfies the canonicalization
    /// assertion trivially and would stay green against an interner that never
    /// deduplicates at all — which is precisely the pre-`C1` behaviour this
    /// test exists to detect. So the repeat count is asserted, not assumed.
    #[test]
    fn boundary_c1_equal_name_bytes_have_one_canonical_span() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let mut spans_by_bytes: BTreeMap<Vec<u8>, Vec<DenseRange>> = BTreeMap::new();
        for atom in &plan.semantic.operands {
            let start = atom.content.start as usize;
            let bytes = plan.semantic.names[start..start + atom.content.len as usize].to_vec();
            spans_by_bytes.entry(bytes).or_default().push(atom.content);
        }

        let repeated = spans_by_bytes
            .iter()
            .filter(|(bytes, spans)| !bytes.is_empty() && spans.len() > 1)
            .count();
        assert!(
            repeated > 0,
            "NON-VACUITY: no non-empty spelling occurs twice in this fixture, so the \
             canonicalization assertion below is trivially satisfied and would not \
             detect an interner that never deduplicates."
        );

        for (bytes, spans) in &spans_by_bytes {
            let first = spans[0];
            let deviant = spans.iter().find(|span| **span != first);
            assert!(
                deviant.is_none(),
                "spelling {:?} is interned at both {:?} and {:?}, so one symbol has more \
                 than one artifact-static identity",
                String::from_utf8_lossy(bytes),
                first,
                deviant.unwrap()
            );
        }
    }

    /// `RT-FNSPLIT-C1` `D2` — the plane refuses a two-identity symbol.
    ///
    /// ⭐ This is the control for the *validator*, not for `intern`. An `intern`
    /// that regressed to an unconditional append leaves every span in bounds and
    /// every budget exact, so every pre-existing check stays green while
    /// producer and consumer quietly stop sharing an identity. The plane has to
    /// assert canonicality itself rather than trusting the function that is
    /// supposed to maintain it.
    #[test]
    fn boundary_c1_validate_rejects_equal_bytes_interned_at_two_spans() {
        // Promise class: durable mutation proof.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        // The unmutated plane is green — the inverse half, so a red result below
        // is attributable to the mutation and not to a fixture that never validated.
        plan.semantic
            .validate(
                &plan.nodes,
                &plan.edges,
                &plan.entries,
                &plan.semantic_sources,
                &plan.semantic_material,
            )
            .expect("the unmutated plane validates");

        // ⛔ The mutation must NOT grow `names`.
        //
        // `validate` already requires `plane.names == arena.names`, and that
        // check runs first. Appending a duplicate copy of a spelling therefore
        // trips the arena-equality check and never reaches the canonicality
        // one — the first draft of this control did exactly that and proved
        // nothing about the property it names.
        //
        // ⭐ So the two equal-byte spans are manufactured *inside* the existing
        // arena: whole symbols are unique after canonicalization, but their
        // BYTES are not — two distinct symbols routinely share a first byte.
        // Pointing two atoms at one-byte spans over the same byte value at
        // different offsets yields equal content at unequal spans with `names`
        // byte-for-byte untouched.
        let mut duplicated = plan.semantic.clone();
        let candidates = duplicated
            .operands
            .iter()
            .filter(|atom| atom.content.len > 0)
            .map(|atom| atom.content)
            .collect::<Vec<_>>();
        let (first, second) = candidates
            .iter()
            .enumerate()
            .find_map(|(i, a)| {
                candidates[i + 1..]
                    .iter()
                    .find(|b| {
                        b.start != a.start
                            && duplicated.names[a.start as usize]
                                == duplicated.names[b.start as usize]
                    })
                    .map(|b| (*a, *b))
            })
            .expect(
                "NON-VACUITY: the fixture has no two out-of-line atoms starting at \
                 different offsets with the same first byte, so this mutation cannot \
                 manufacture equal bytes at unequal spans and the control is vacuous.",
            );

        for atom in duplicated.operands.iter_mut() {
            if atom.content == first {
                atom.content = DenseRange {
                    start: first.start,
                    len: 1,
                };
            } else if atom.content == second {
                atom.content = DenseRange {
                    start: second.start,
                    len: 1,
                };
            }
        }
        assert_eq!(
            duplicated.names, plan.semantic.names,
            "the mutation must leave the name arena byte-identical, or the \
             arena-equality check fires before the canonicality check and this \
             control measures the wrong rejection"
        );

        assert_eq!(
            duplicated
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error(
                "equal semantic name bytes are interned at two different spans, \
                 so one symbol has two identities"
            )
        );
    }

    /// `RT-FNSPLIT-C1` `D1` — the one identity ABI encoding is injective and
    /// reserves zero.
    ///
    /// ⚠ `start = 0, len = 0` is a **legitimate** identity (the empty name at
    /// offset zero), which is the entire reason the encoding adds one. Without
    /// the `+1` that identity would encode as `0` and be indistinguishable from
    /// uninitialized ABI memory.
    #[test]
    fn boundary_c1_identity_abi_word_round_trips_and_reserves_zero() {
        // Promise class: normative compatibility vector — the encoding is the
        // contract between the planner and the carrier's emitted ABI.
        for (start, len) in [
            (0u32, 0u32),
            (0, 1),
            (1, 0),
            (7, 3),
            (u32::MAX, u32::MAX - 1),
        ] {
            let span = DenseRange { start, len };
            let packed = ConstructorIdentity(span).tag_abi_word().unwrap();
            assert_ne!(packed, 0, "({start},{len}) encoded as the invalid sentinel");
            assert_eq!(
                super::semantic_ir::unpack_identity(packed).unwrap(),
                span,
                "({start},{len}) did not round trip"
            );
        }

        // Both namespaces share the one encoding, so a field identity and a
        // constructor identity over the same span agree numerically. That is
        // intended: the separation is carried by the *type*, not by the number.
        let span = DenseRange { start: 9, len: 4 };
        assert_eq!(
            ConstructorIdentity(span).tag_abi_word().unwrap(),
            FieldIdentity(span).name_abi_word().unwrap()
        );

        assert_eq!(
            super::semantic_ir::unpack_identity(0).unwrap_err(),
            planner_error("semantic identity is the reserved invalid sentinel")
        );

        // ⭐ Capacity loudness. The `+1` that reserves zero costs exactly one
        // encodable span at the very top of the range, and the refusal must be
        // a loud capacity error rather than a wrap to the sentinel — a silent
        // wrap would hand emitted code the "invalid" word for a valid symbol.
        assert_eq!(
            ConstructorIdentity(DenseRange {
                start: u32::MAX,
                len: u32::MAX
            })
            .tag_abi_word()
            .unwrap_err(),
            planner_capacity_error("semantic identity encoding exhausted")
        );
    }

    #[test]
    fn boundary_b1r_atom_content_must_stay_inside_the_closed_name_arena() {
        // Promise class: durable mutation proof. B2a decodes atom content, so a
        // structurally well-formed atom whose span escapes the arena, or whose
        // bytes are not the ones the walk interned, is undecodable material.
        let expr = equal_shaped_primitive_pair(
            crate::RuntimePartiality::Total,
            crate::RuntimePartiality::CheckedTrap {
                obligation: "ken.bytes.at.inBounds".to_string(),
            },
        );
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let mut escaped = plan.semantic.clone();
        let atom = escaped
            .operands
            .iter_mut()
            .find(|atom| atom.content.len > 0)
            .expect("fixture has an atom with out-of-line content");
        atom.content.start = u32::try_from(plan.semantic.names.len()).unwrap();
        assert_eq!(
            escaped
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic atom content range is outside its closed name arena")
        );

        let mut retagged = plan.semantic.clone();
        retagged.names.push(0xff);
        assert_eq!(
            retagged
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error(
                "semantic atom content arena is not the material the source walk interned"
            )
        );
    }

    #[test]
    fn boundary_b1r_control_2_dropping_one_origins_material_record_is_rejected() {
        // Promise class: durable mutation proof.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let carrier = plan
            .semantic
            .records
            .iter()
            .position(|record| record.operands.len > 0)
            .expect("fixture has an occurrence with non-child material");

        // Drop this origin's ownership of its material while leaving the atom
        // arena intact, so the global one-visit budget still balances and only
        // the per-record artifact can catch it. Removing the atoms instead would
        // redden at the arena-budget artifact the superlinear control already
        // owns, which would not discriminate this fault.
        let mut dropped = plan.semantic.clone();
        assert!(dropped.records[carrier].operands.len > 0);
        dropped.records[carrier].operands.len = 0;
        assert_eq!(
            dropped.operands.len(),
            plan.semantic.operands.len(),
            "the atom arena must be untouched, or a different artifact fires"
        );
        assert_eq!(
            dropped
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic record does not own its exact source-material range")
        );
    }

    #[test]
    fn boundary_b1r_control_3_duplicating_a_material_record_origin_is_rejected() {
        // Promise class: durable mutation proof.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let mut duplicated = plan.semantic.clone();
        duplicated.records[1].origin = duplicated.records[0].origin;
        assert_eq!(
            duplicated
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic program is not the exhaustive lowering of its source")
        );
    }

    #[test]
    fn boundary_a_nested_resource_brackets_n3_through_n7_are_closed_and_affine() {
        const WORKER_ENV: &str = "KEN_RT_SCALE_A_CENSUS_WORKER";
        const FORCE_INDETERMINATE_ENV: &str = "KEN_RT_SCALE_A_FORCE_INDETERMINATE";
        const OMIT_RESULT_ENV: &str = "KEN_RT_SCALE_A_OMIT_RESULT";
        const COMPLETE_RESULT: &str = "RT_NATIVE_FNSPLIT_BOUNDARY_A_RESULT \
             status=measured_complete rows=5 stack_bytes=8388608";
        if std::env::var_os(WORKER_ENV).is_none() {
            let run_worker = |force_indeterminate: bool, omit_result: bool| {
                let executable = std::env::current_exe().unwrap_or_else(|error| {
                    panic!("RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: {error}")
                });
                let test_name = std::thread::current()
                    .name()
                    .expect("libtest names every test thread")
                    .to_string();
                let mut command = std::process::Command::new("prlimit");
                command
                    .args([
                        "--cpu=30:30",
                        "--as=4294967296:4294967296",
                        "--stack=8388608:8388608",
                        "--",
                    ])
                    .arg(executable)
                    .args(["--exact", &test_name, "--nocapture", "--test-threads=1"])
                    .env(WORKER_ENV, "1")
                    // This isolated process's incidental libtest thread only
                    // dispatches the deliberately-created 8 MiB planner
                    // thread below. `prlimit` bounds the process and catches
                    // aborts; no recursive planning runs on libtest's stack.
                    // Do not inherit the repository's 256 MiB convention.
                    .env_remove("RUST_MIN_STACK");
                if force_indeterminate {
                    command.env(FORCE_INDETERMINATE_ENV, "1");
                }
                if omit_result {
                    command.env(OMIT_RESULT_ENV, "1");
                }
                command
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                let mut child = command.spawn().unwrap_or_else(|error| {
                    panic!(
                        "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                         prlimit worker could not start: {error}"
                    )
                });
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
                loop {
                    match child.try_wait() {
                        Ok(Some(_)) => {
                            break child.wait_with_output().unwrap_or_else(|error| {
                                panic!(
                                    "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                                     worker result could not be collected: {error}"
                                )
                            });
                        }
                        Ok(None) if std::time::Instant::now() < deadline => {
                            std::thread::sleep(std::time::Duration::from_millis(25));
                        }
                        Ok(None) => {
                            let _ = child.kill();
                            break child.wait_with_output().unwrap_or_else(|error| {
                                panic!(
                                    "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                                     timed-out worker could not be reaped: {error}"
                                )
                            });
                        }
                        Err(error) => {
                            let _ = child.kill();
                            panic!(
                                "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                                 worker status could not be observed: {error}"
                            );
                        }
                    }
                }
            };

            // AC-A1 positive control: the third outcome must be observable and
            // must fail. This is not merely a successful-worker smoke test.
            let forced = run_worker(true, false);
            let forced_report = format!(
                "{}{}",
                String::from_utf8_lossy(&forced.stdout),
                String::from_utf8_lossy(&forced.stderr)
            );
            assert!(
                !forced.status.success() && forced_report.contains("could_not_determine"),
                "AC-A1: forced indeterminacy must fail with the stable third-outcome spelling; \
                 status={:?}, report={forced_report}",
                forced.status
            );

            // A zero exit is not enough: missing/malformed result data is the
            // same third outcome, not a silent pass.
            let omitted = run_worker(false, true);
            let omitted_report = format!(
                "{}{}",
                String::from_utf8_lossy(&omitted.stdout),
                String::from_utf8_lossy(&omitted.stderr)
            );
            assert!(
                omitted.status.success() && !omitted_report.contains(COMPLETE_RESULT),
                "AC-A1: the missing-result control must reach a zero exit without \
                 accidentally emitting a complete census"
            );

            let measured = run_worker(false, false);
            let measured_report = format!(
                "{}{}",
                String::from_utf8_lossy(&measured.stdout),
                String::from_utf8_lossy(&measured.stderr)
            );
            eprint!("{measured_report}");
            assert!(
                measured.status.success() && measured_report.contains(COMPLETE_RESULT),
                "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: bounded worker \
                 stack_bytes=8388608 status={:?}, complete result sentinel missing or malformed",
                measured.status,
            );
            return;
        }

        if std::env::var_os(FORCE_INDETERMINATE_ENV).is_some() {
            panic!(
                "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                 stack_bytes=8388608 forced fail-closed positive control"
            );
        }
        if std::env::var_os(OMIT_RESULT_ENV).is_some() {
            return;
        }

        let planner_worker = std::thread::Builder::new()
            .name("rt-scale-a-planner-8-mib".to_string())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
        eprintln!(
            "RT_NATIVE_FNSPLIT_BOUNDARY_A_STACK \
             worker=rt-scale-a-planner-8-mib stack=nominal_8_MiB stack_bytes=8388608 \
             process_main_stack_limit=8_MiB cpu_limit=30_s address_space_limit=4_GiB \
             claim=explicit_product_stack_measurement"
        );

        // Promise class: durable invariant. Counts remain relational; the
        // historic literals below are comparison data, never a re-baseline or
        // an exponent inferred from five points.
        let rows = (3..=7).map(census).collect::<Vec<_>>();
        for (depth, row) in (3..=7).zip(&rows) {
            eprintln!(
                "RT_NATIVE_FNSPLIT_BOUNDARY_A n={depth} static_nodes={} edges={} \
                 planned_helpers={} persistent_store_nodes={} evidence_records={} \
                 fixed_k={} observed_max_helpers_per_source={} key_bytes={} \
                 key_schemas={} frame_schemas={} store_schemas={} \
                 static_node_id_bytes={} persistent_node_id_bytes={} \
                 max_logical_chain_depth={} env_depth={} continuation_depth={} \
                 path_depth={} cleanup_depth={} affine_depth={} source_return_depth={} \
                 source_return_resume_nodes={} source_return_owned_resume_edges={} \
                 terminal_outgoing_edges={} recursive_lowering_frames={} \
                 stack_bytes=8388608 \
                 node_payload_width=\"DEFERRED — NEEDS B2V/B2F\" \
                 frame_schema_width=\"DEFERRED — NEEDS B2V/B2F\" \
                 store_node_schema_width=\"DEFERRED — NEEDS B2V/B2F\"",
                row.static_nodes,
                row.edges,
                row.planned_helpers,
                row.persistent_store_nodes,
                row.out_of_line_evidence_records,
                MAX_HELPERS_PER_STATIC_SOURCE,
                row.max_helpers_per_static_source,
                row.helper_key_bytes,
                row.helper_key_schemas,
                row.frame_schemas,
                row.store_node_schemas,
                row.static_node_id_bytes,
                row.persistent_node_id_bytes,
                row.max_logical_chain_depth,
                row.max_environment_depth,
                row.max_continuation_depth,
                row.max_path_depth,
                row.max_cleanup_depth,
                row.max_affine_depth,
                row.max_source_return_depth,
                row.source_return_resume_nodes,
                row.source_return_owned_resume_edges,
                row.terminal_outgoing_edges,
                row.recursive_lowering_frames,
            );
        }
        for (name, values) in [
            ("static_nodes", values(&rows, |r| r.static_nodes)),
            ("edges", values(&rows, |r| r.edges)),
            ("planned_helpers", values(&rows, |r| r.planned_helpers)),
            (
                "persistent_store_nodes",
                values(&rows, |r| r.persistent_store_nodes),
            ),
            (
                "evidence_records",
                values(&rows, |r| r.out_of_line_evidence_records),
            ),
            (
                "fixed_k",
                values(&rows, |_| MAX_HELPERS_PER_STATIC_SOURCE),
            ),
            (
                "observed_max_helpers_per_source",
                values(&rows, |r| r.max_helpers_per_static_source),
            ),
            (
                "source_return_resume_nodes",
                values(&rows, |r| r.source_return_resume_nodes),
            ),
            (
                "source_return_owned_resume_edges",
                values(&rows, |r| r.source_return_owned_resume_edges),
            ),
            (
                "terminal_outgoing_edges",
                values(&rows, |r| r.terminal_outgoing_edges),
            ),
            (
                "recursive_lowering_frames",
                values(&rows, |r| r.recursive_lowering_frames),
            ),
            ("helper_key_bytes", values(&rows, |r| r.helper_key_bytes)),
            (
                "static_node_id_bytes",
                values(&rows, |r| r.static_node_id_bytes),
            ),
            (
                "persistent_node_id_bytes",
                values(&rows, |r| r.persistent_node_id_bytes),
            ),
            (
                "helper_key_schemas",
                values(&rows, |r| r.helper_key_schemas),
            ),
            ("frame_schemas", values(&rows, |r| r.frame_schemas)),
            (
                "store_node_schemas",
                values(&rows, |r| r.store_node_schemas),
            ),
            (
                "max_logical_chain_depth",
                values(&rows, |r| r.max_logical_chain_depth as usize),
            ),
            (
                "environment_depth",
                values(&rows, |r| r.max_environment_depth as usize),
            ),
            (
                "continuation_depth",
                values(&rows, |r| r.max_continuation_depth as usize),
            ),
            ("path_depth", values(&rows, |r| r.max_path_depth as usize)),
            (
                "cleanup_depth",
                values(&rows, |r| r.max_cleanup_depth as usize),
            ),
            (
                "affine_depth",
                values(&rows, |r| r.max_affine_depth as usize),
            ),
            (
                "source_return_depth",
                values(&rows, |r| r.max_source_return_depth as usize),
            ),
        ] {
            let (first, second) = differences(&values);
            eprintln!(
                "RT_NATIVE_FNSPLIT_BOUNDARY_A_DIFF metric={name} first={first:?} second={second:?}"
            );
            assert!(
                second.iter().all(|difference| *difference == 0),
                "{name} is not affine across n=3..7"
            );
        }
        for (name, field) in [
            (
                "helper_key_bytes",
                (|r: &BoundaryACensus| r.helper_key_bytes) as fn(&BoundaryACensus) -> usize,
            ),
            ("static_node_id_bytes", |r: &BoundaryACensus| {
                r.static_node_id_bytes
            }),
            ("persistent_node_id_bytes", |r: &BoundaryACensus| {
                r.persistent_node_id_bytes
            }),
            ("helper_key_schemas", |r: &BoundaryACensus| {
                r.helper_key_schemas
            }),
            ("frame_schemas", |r: &BoundaryACensus| r.frame_schemas),
            ("store_node_schemas", |r: &BoundaryACensus| {
                r.store_node_schemas
            }),
        ] {
            let values = values(&rows, field);
            assert!(
                values.windows(2).all(|pair| pair[0] == pair[1]),
                "{name} is not constant across n=3..7"
            );
        }
        assert!(rows
            .iter()
            .all(|row| row.max_helpers_per_static_source <= MAX_HELPERS_PER_STATIC_SOURCE));
        assert!(rows
            .iter()
            .zip(3..=7)
            .all(|(row, depth)| {
                row.source_return_resume_nodes == depth
                    && row.source_return_owned_resume_edges == depth
                    && row.terminal_outgoing_edges == 0
                    && row.recursive_lowering_frames > depth
            }));
        assert!(rows.iter().all(|row| {
            row.planned_helpers == row.static_nodes + row.edges
                && row.out_of_line_evidence_records == row.edges
                && row.max_environment_depth <= row.persistent_store_nodes as u32
                && row.max_continuation_depth <= row.persistent_store_nodes as u32
                && row.max_path_depth <= row.persistent_store_nodes as u32
                && row.max_logical_chain_depth <= row.persistent_store_nodes as u32
        }));

        let measured_static_nodes = values(&rows, |row| row.static_nodes);
        let provisional_static_nodes = [87, 115, 143, 171, 199];
        let static_nodes_agree = measured_static_nodes
            .iter()
            .copied()
            .eq(provisional_static_nodes);
        let measured_k_is_eight = rows
            .iter()
            .all(|row| row.max_helpers_per_static_source == 8);
        let measured_key_bytes_are_twelve = rows.iter().all(|row| row.helper_key_bytes == 12);
        eprintln!(
            "RT_NATIVE_FNSPLIT_BOUNDARY_A_PROVISIONAL relation_static_nodes={} \
             relation_observed_k={} relation_key_width={} \
             provisional_frame_store_widths=32/16 \
             current_frame_store_widths=\"DEFERRED — NEEDS B2V/B2F\" \
             stack_bytes=8388608 verdict=agreement_is_a_finding_not_confirmation",
            if static_nodes_agree {
                "agrees_with_87/115/143/171/199"
            } else {
                "differs_from_87/115/143/171/199"
            },
            if measured_k_is_eight {
                "agrees_with_8"
            } else {
                "differs_from_8"
            },
            if measured_key_bytes_are_twelve {
                "agrees_with_12"
            } else {
                "differs_from_12"
            },
        );
        eprintln!(
            "RT_NATIVE_FNSPLIT_BOUNDARY_A_EXPONENT_VERDICT \
             five_points_do_not_prove_an_exponent=true \
             historic_n4_fits=370n,93n²,product_switching_on_at_n5 \
             discriminator=structural_invariants table=corroboration_only \
             stack_bytes=8388608"
        );

        const AC_CONTROLS: [(&str, &str); 8] = [
            (
                "AC-A1",
                "prlimit worker plus forced failure and missing-result positive controls",
            ),
            (
                "AC-A2",
                "one emitted row per n with every due D2 field and three spelled deferrals",
            ),
            (
                "AC-A3",
                "first and second finite differences emitted for every due numeric row",
            ),
            (
                "AC-A4",
                "closed Copy helper-key patterns, constant ID/key/schema checks, affine stores/depth",
            ),
            (
                "AC-A5",
                "explicit five-points-do-not-prove-exponent verdict",
            ),
            (
                "AC-A6",
                "test-only guard measures maximum simultaneous production plan_expr calls",
            ),
            (
                "AC-A7",
                "computed provisional relation with agreement-not-confirmation verdict",
            ),
            (
                "AC-A8",
                "exact eight-row AC control inventory asserted below",
            ),
        ];
        assert_eq!(
            AC_CONTROLS.map(|(criterion, _)| criterion),
            [
                "AC-A1", "AC-A2", "AC-A3", "AC-A4", "AC-A5", "AC-A6", "AC-A7", "AC-A8"
            ]
        );
        for (criterion, control) in AC_CONTROLS {
            let control = if control.is_empty() {
                "NO CONTROL — open residual"
            } else {
                control
            };
            eprintln!(
                "RT_NATIVE_FNSPLIT_BOUNDARY_A_CONTROL criterion={criterion} control={control}"
            );
        }
        eprintln!(
            "RT_NATIVE_FNSPLIT_BOUNDARY_A_RESULT \
             status=measured_complete rows=5 stack_bytes=8388608"
        );
            })
            .unwrap_or_else(|error| {
                panic!(
                    "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                     exact 8 MiB planner worker could not start: {error}"
                )
            });
        if planner_worker.join().is_err() {
            panic!(
                "RT_NATIVE_FNSPLIT_BOUNDARY_A could_not_determine: \
                 stack_bytes=8388608 exact 8 MiB planner worker panicked"
            );
        }
    }

    #[test]
    fn planner_invariant_failures_have_compiler_bug_attribution() {
        // Promise class: durable invariant. These distinct planner
        // self-consistency failures are compiler bugs. The former fixed-K capacity arm is
        // not input-reachable because fixed K is a structural planner invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let mut missing_helper = plan.clone();
        missing_helper.planned_helpers.pop();
        let invariant = missing_helper.validate().unwrap_err();
        assert!(matches!(
            &invariant,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(detail))
                if detail == "planned helper inventory is not exact for the closed graph"
        ));
        assert_eq!(
            invariant.to_string(),
            "Cranelift backend failure: native static transition planner invariant failed; \
             please report this compiler bug: planned helper inventory is not exact for the \
             closed graph"
        );
        assert!(!invariant.to_string().contains("unsupported"));

        let mut helpers_per_source = BTreeMap::<StaticSourceId, usize>::new();
        for helper in &plan.planned_helpers {
            let owner = match *helper {
                PlannedHelperKey::Node(_, id) => plan.nodes[id.0 as usize].owner,
                PlannedHelperKey::Edge(_, id) => {
                    let edge = plan.edges[id.0 as usize];
                    plan.nodes[edge.from.0 as usize].owner
                }
            };
            *helpers_per_source.entry(owner).or_default() += 1;
        }
        let owner = helpers_per_source
            .iter()
            .find_map(|(owner, count)| (*count == MAX_HELPERS_PER_STATIC_SOURCE).then_some(*owner))
            .expect("nested bracket plan has a source at the fixed K capacity");
        let frame = plan
            .nodes
            .iter()
            .find(|node| node.owner == owner)
            .expect("capacity owner has a node")
            .frame;
        let mut over_capacity = plan.clone();
        let id = StaticNodeId(over_capacity.nodes.len() as u32);
        over_capacity.nodes.push(StaticNode {
            id,
            transition: TransitionKind::Evaluate,
            owner,
            frame,
        });
        over_capacity
            .planned_helpers
            .push(PlannedHelperKey::node(TransitionKind::Evaluate, id));

        let fixed_k_invariant = over_capacity.validate().unwrap_err();
        assert!(matches!(
            &fixed_k_invariant,
            CraneliftBackendError::Backend(BackendFailure::PlannerInvariant(detail))
                if detail == "fixed K helpers per static source was exceeded"
        ));
        assert_eq!(
            fixed_k_invariant.to_string(),
            "Cranelift backend failure: native static transition planner invariant failed; \
             please report this compiler bug: fixed K helpers per static source was exceeded"
        );
        assert!(!fixed_k_invariant.to_string().contains("unsupported"));
    }

    #[test]
    fn distinct_activations_share_one_helper_key_and_source_return_is_not_terminal() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let wrapper = plan
            .nodes
            .iter()
            .find(|node| node.transition == TransitionKind::ProducerWrapper)
            .unwrap();
        let other_activation = plan
            .nodes
            .iter()
            .map(|node| node.frame)
            .find(|frame| {
                frame.environment != wrapper.frame.environment && frame.path != wrapper.frame.path
            })
            .expect("nested bracket plan has a distinct valid activation");
        assert_ne!(wrapper.frame, other_activation);
        let helpers_before = plan.census().planned_helpers;
        let first = plan
            .helper_key_for_activation(wrapper.id, wrapper.frame)
            .unwrap();
        let second = plan
            .helper_key_for_activation(wrapper.id, other_activation)
            .unwrap();
        assert_eq!(
            BTreeSet::from([first, second]).len(),
            1,
            "distinct dynamic activations multiplied one static helper"
        );
        assert_eq!(
            plan.census().planned_helpers,
            helpers_before,
            "flowing another activation through a static node grew planned code"
        );
        assert!(plan
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::SourceReturnOwnedResume)
            .all(|edge| {
                plan.nodes[edge.to.0 as usize].transition == TransitionKind::SourceReturnResume
                    && edge.to != plan.terminal_id()
            }));
    }

    #[test]
    fn source_return_ownership_guards_fail_closed_on_exact_cross_wires() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let wrappers = plan
            .nodes
            .iter()
            .filter(|node| node.transition == TransitionKind::ProducerWrapper)
            .collect::<Vec<_>>();
        let first_wrapper = wrappers[0];
        let second_wrapper = wrappers[1];
        let node_for = |owner, transition| {
            plan.nodes
                .iter()
                .find(|node| node.owner == owner && node.transition == transition)
                .unwrap()
                .id
        };
        let first_resume = node_for(first_wrapper.owner, TransitionKind::SourceReturnResume);
        let second_resume = node_for(second_wrapper.owner, TransitionKind::SourceReturnResume);
        let first_tail = node_for(first_wrapper.owner, TransitionKind::ProducerTail);
        let second_tail = node_for(second_wrapper.owner, TransitionKind::ProducerTail);

        let source_return_edge = *plan
            .edges
            .iter()
            .find(|edge| edge.to == first_resume && edge.kind == EdgeKind::SourceReturnOwnedResume)
            .unwrap();
        let mut crossed_resume = plan.clone();
        rewrite_edge(
            &mut crossed_resume,
            source_return_edge.id,
            source_return_edge.from,
            second_resume,
            source_return_edge.kind,
        );
        assert_eq!(
            crossed_resume.validate().unwrap_err(),
            planner_error("source-return-owned edge targets a resume from another descriptor")
        );

        let resume_edge = *plan
            .edges
            .iter()
            .find(|edge| edge.from == first_resume && edge.kind == EdgeKind::InvokeProducerWrapper)
            .unwrap();
        let mut crossed_wrapper = plan.clone();
        rewrite_edge(
            &mut crossed_wrapper,
            resume_edge.id,
            resume_edge.from,
            second_wrapper.id,
            resume_edge.kind,
        );
        assert_eq!(
            crossed_wrapper.validate().unwrap_err(),
            planner_error("source-return resume must have only its exact wrapper invocation")
        );

        let wrapper_edge = *plan
            .edges
            .iter()
            .find(|edge| edge.from == first_wrapper.id && edge.kind == EdgeKind::InvokeProducerTail)
            .unwrap();
        let mut crossed_tail = plan.clone();
        rewrite_edge(
            &mut crossed_tail,
            wrapper_edge.id,
            wrapper_edge.from,
            second_tail,
            wrapper_edge.kind,
        );
        assert_eq!(
            crossed_tail.validate().unwrap_err(),
            planner_error("producer wrapper must have only its exact tail invocation")
        );

        let descriptor = first_wrapper.frame.source_return.0 as usize - 1;
        let mut crossed_descriptor_wrapper = plan.clone();
        crossed_descriptor_wrapper.stores[descriptor].local = second_wrapper.id.0;
        assert_eq!(
            crossed_descriptor_wrapper.validate().unwrap_err(),
            planner_error("source-return descriptor does not name its exact W and T")
        );

        let mut crossed_descriptor_tail = plan.clone();
        crossed_descriptor_tail.stores[descriptor].aux = second_tail.0;
        assert_eq!(
            crossed_descriptor_tail.validate().unwrap_err(),
            planner_error("source-return descriptor does not name its exact W and T")
        );

        let tail_edge = *plan
            .edges
            .iter()
            .find(|edge| edge.from == first_tail && edge.kind == EdgeKind::CompleteProducerTail)
            .unwrap();
        let mut duplicate_wrapper = plan.clone();
        rewrite_edge(
            &mut duplicate_wrapper,
            tail_edge.id,
            first_resume,
            first_wrapper.id,
            EdgeKind::InvokeProducerWrapper,
        );
        assert_eq!(
            duplicate_wrapper.validate().unwrap_err(),
            planner_error("source-return resume must have only its exact wrapper invocation")
        );

        let mut terminal_resume = plan.clone();
        rewrite_edge(
            &mut terminal_resume,
            source_return_edge.id,
            source_return_edge.from,
            plan.terminal_id(),
            source_return_edge.kind,
        );
        assert_eq!(
            terminal_resume.validate().unwrap_err(),
            planner_error("source-return-owned edge targets a resume from another descriptor")
        );

        let mut wrapper_entry = plan.clone();
        wrapper_entry.entries[0] = first_wrapper.id;
        assert_eq!(
            wrapper_entry.validate().unwrap_err(),
            planner_error("producer wrapper cannot be a pre-source graph entry")
        );
    }

    #[test]
    fn quartet_edge_sets_and_completed_successor_reject_alternate_calls() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();
        let wrapper = plan
            .nodes
            .iter()
            .find(|node| node.transition == TransitionKind::ProducerWrapper)
            .unwrap();
        let node_for = |transition| {
            plan.nodes
                .iter()
                .find(|node| node.owner == wrapper.owner && node.transition == transition)
                .unwrap()
                .id
        };
        let resume = node_for(TransitionKind::SourceReturnResume);
        let tail = node_for(TransitionKind::ProducerTail);
        let completed = node_for(TransitionKind::CompletedTail);
        let ordinary = plan
            .nodes
            .iter()
            .find(|node| node.owner != wrapper.owner && node.transition == TransitionKind::Evaluate)
            .unwrap()
            .id;

        let mut alternate_tail_incoming = plan.clone();
        append_edge(
            &mut alternate_tail_incoming,
            ordinary,
            tail,
            EdgeKind::Continue,
        );
        assert_eq!(
            alternate_tail_incoming.validate().unwrap_err(),
            planner_error("producer tail must have only its exact wrapper invocation")
        );

        let mut alternate_completed_incoming = plan.clone();
        append_edge(
            &mut alternate_completed_incoming,
            ordinary,
            completed,
            EdgeKind::Continue,
        );
        assert_eq!(
            alternate_completed_incoming.validate().unwrap_err(),
            planner_error("CompletedTail must have only its exact producer-tail completion")
        );

        for (from, expected) in [
            (
                resume,
                "source-return resume must have only its exact wrapper invocation",
            ),
            (
                wrapper.id,
                "producer wrapper must have only its exact tail invocation",
            ),
            (
                tail,
                "producer tail must have only its exact completion edge",
            ),
        ] {
            let mut alternate_outgoing = plan.clone();
            append_edge(
                &mut alternate_outgoing,
                from,
                plan.terminal_id(),
                EdgeKind::Continue,
            );
            assert_eq!(
                alternate_outgoing.validate().unwrap_err(),
                planner_error(expected)
            );
        }

        let completed_edge = *plan
            .edges
            .iter()
            .find(|edge| edge.from == completed)
            .unwrap();
        let mut wrong_successor = plan.clone();
        rewrite_edge(
            &mut wrong_successor,
            completed_edge.id,
            completed,
            plan.trap_terminal_id(),
            completed_edge.kind,
        );
        assert_eq!(
            wrong_successor.validate().unwrap_err(),
            planner_error("CompletedTail must have only its activation-named successor")
        );

        let mut wrong_resume_kind = plan.clone();
        rewrite_edge(
            &mut wrong_resume_kind,
            completed_edge.id,
            completed,
            completed_edge.to,
            EdgeKind::Trap,
        );
        assert_eq!(
            wrong_resume_kind.validate().unwrap_err(),
            planner_error("CompletedTail successor does not use its normal-resume edge kind")
        );
    }

    #[test]
    fn entry_and_reachability_closure_rejects_balancing_invalid_root() {
        // Promise class: durable invariant.
        let expr = unit();
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let mut outside = plan.clone();
        outside.entries[0] = StaticNodeId(u32::MAX);
        assert_eq!(
            outside.validate().unwrap_err(),
            planner_error("graph entry is outside the closed node set")
        );

        let mut duplicate = plan.clone();
        duplicate.entries.push(duplicate.entries[0]);
        assert_eq!(
            duplicate.validate().unwrap_err(),
            planner_error("closed graph contains a duplicate entry")
        );
    }

    #[test]
    fn closed_identity_terminal_and_store_guards_reject_exact_mutations() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).unwrap();

        let mut wrong_node_identity = plan.clone();
        let evaluate = wrong_node_identity
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.transition == TransitionKind::Evaluate)
            .map(|(index, node)| (index, node.id))
            .take(2)
            .collect::<Vec<_>>();
        wrong_node_identity.nodes[evaluate[0].0].id = evaluate[1].1;
        assert_eq!(
            wrong_node_identity.validate().unwrap_err(),
            planner_error("static node identity does not match its closed position")
        );

        let mut terminal_outgoing = plan.clone();
        let resume = terminal_outgoing
            .nodes
            .iter()
            .find(|node| node.transition == TransitionKind::SourceReturnResume)
            .unwrap()
            .id;
        append_edge(
            &mut terminal_outgoing,
            plan.terminal_id(),
            resume,
            EdgeKind::Continue,
        );
        assert_eq!(
            terminal_outgoing.validate().unwrap_err(),
            planner_error("Terminal and TrapTerminal must have no outgoing edges")
        );

        let mut unclosed_store = plan.clone();
        unclosed_store.stores[0].child = PersistentNodeId(unclosed_store.stores.len() as u32 + 1);
        assert_eq!(
            unclosed_store.validate().unwrap_err(),
            planner_error("persistent store child is not an earlier closed node")
        );

        let mut wrong_depth = plan.clone();
        wrong_depth.store_depths[0] += 1;
        assert_eq!(
            wrong_depth.validate().unwrap_err(),
            planner_error("persistent store depth does not match its child chain")
        );

        let mut duplicate_store = plan.clone();
        duplicate_store.stores[1] = duplicate_store.stores[0];
        assert_eq!(
            duplicate_store.validate().unwrap_err(),
            planner_error("persistent store contains a duplicate node")
        );

        let mut missing_helper = plan.clone();
        missing_helper.planned_helpers.pop();
        assert_eq!(
            missing_helper.validate().unwrap_err(),
            planner_error("planned helper inventory is not exact for the closed graph")
        );
    }

    impl StaticTransitionPlan<'_> {
        fn terminal_id(&self) -> StaticNodeId {
            self.nodes
                .iter()
                .find(|node| node.transition == TransitionKind::Terminal)
                .expect("closed graph has Terminal")
                .id
        }

        fn trap_terminal_id(&self) -> StaticNodeId {
            self.nodes
                .iter()
                .find(|node| node.transition == TransitionKind::TrapTerminal)
                .expect("closed graph has TrapTerminal")
                .id
        }
    }
    #[cfg(test)]
    fn b2ac_topology_fixtures() -> Vec<(&'static str, RuntimeExpr)> {
        let leaf = || RuntimeExpr::Value(RuntimeValue::Bool(true));
        let trap = || RuntimeTrap {
            code: RuntimeTrapCode::PatternMatchFailure,
            message: "b2ac topology".to_string(),
        };
        let computational = |body: RuntimeExpr| RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::B2AC::Node".to_string(),
                args: vec![leaf()],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::B2AC::Node".to_string(),
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body,
            }],
            default: trap(),
        };
        vec![
            ("leaf", leaf()),
            (
                "let-if",
                RuntimeExpr::Let {
                    value: Box::new(RuntimeExpr::If {
                        scrutinee: Box::new(leaf()),
                        then_expr: Box::new(leaf()),
                        else_expr: Box::new(leaf()),
                    }),
                    body: Box::new(RuntimeExpr::Var(0)),
                },
            ),
            (
                "match",
                RuntimeExpr::Match {
                    scrutinee: Box::new(leaf()),
                    cases: vec![RuntimeMatchCase {
                        constructor: "ctor:fixture::B2AC::A".to_string(),
                        binders: 0,
                        body: leaf(),
                    }],
                    default: trap(),
                },
            ),
            (
                "lexical-closure-call",
                RuntimeExpr::Call {
                    callee: Box::new(RuntimeExpr::LexicalClosure {
                        captures: vec![leaf()],
                        params: vec!["x".to_string()],
                        body: Box::new(RuntimeExpr::Var(0)),
                    }),
                    args: vec![leaf()],
                },
            ),
            ("computational", computational(RuntimeExpr::Var(0))),
            (
                "computational-nested",
                computational(computational(RuntimeExpr::Var(0))),
            ),
            (
                "computational-under-let",
                RuntimeExpr::Let {
                    value: Box::new(computational(RuntimeExpr::Var(0))),
                    body: Box::new(RuntimeExpr::Var(0)),
                },
            ),
        ]
    }

    /// A canonical digest of the Boundary-A transfer graph: node transitions in
    /// order, then every edge as `(from, to, kind)` in order.
    #[cfg(test)]
    fn b2ac_topology_digest(expr: &RuntimeExpr) -> String {
        let plan = plan_static_transition_graph(expr, &BTreeMap::new()).expect("plannable");
        let mut digest = String::new();
        digest.push_str(&format!(
            "nodes={} edges={}",
            plan.nodes.len(),
            plan.edges.len()
        ));
        for node in &plan.nodes {
            digest.push_str(&format!("|n{}:{:?}", node.id.0, node.transition));
        }
        for edge in &plan.edges {
            digest.push_str(&format!(
                "|e{}:{}->{}:{:?}",
                edge.id.0, edge.from.0, edge.to.0, edge.kind
            ));
        }
        digest.push_str(&format!("|entries={:?}", plan.entries));
        digest
    }

    /// **AC-11 — every transfer edge is unchanged and consumes `.entry`.**
    ///
    /// These digests were captured by running the identical probe against the
    /// WP's base commit `70bd2c74` — before `PlannedExpr` existed — in a scratch
    /// worktree, and are asserted here against the post-D9 planner. Equality is
    /// the mechanical proof that the Boundary-A graph is topologically
    /// identical: same nodes in the same order, same edges with the same
    /// `(from, to, kind)`, same scheduling entries.
    ///
    /// ## ⚠ Reproducing the baseline — the recipe, because equality hides its own
    /// provenance
    ///
    /// ⛔ The asserted property is *equality against committed constants*, so a
    /// re-capture taken **after** the change would have produced byte-identical
    /// values. **Nothing in this file distinguishes a genuine pre-change baseline
    /// from a re-recording**, and the scratch worktree it was taken in is gone. So
    /// the binding is demonstrated here rather than testified to — anyone can
    /// redo it in about two minutes:
    ///
    /// ```text
    /// git worktree add --detach /tmp/b2ac-base 70bd2c74
    /// # port these two functions into that tree's test module verbatim:
    /// #   `b2ac_topology_fixtures`  (the seven fixtures, by name)
    /// #   `b2ac_topology_digest`    (nodes, edges, entries -- it reads nothing
    /// #                              that postdates the base, which is why it
    /// #                              compiles there at all)
    /// cd /tmp/b2ac-base
    /// scripts/ken-cargo test -p ken-runtime --lib -- b2ac_topology
    /// git worktree remove /tmp/b2ac-base
    /// ```
    ///
    /// ⛔ `scripts/ken-cargo`, never raw `cargo` — `COORDINATION §12`, and it binds
    /// inside a copied recipe exactly as it binds anywhere else. A recipe that
    /// spells the raw command teaches the next reader to bypass the build lock.
    ///
    /// Verified this way by the adversary on `2db29abe`: **all seven rows
    /// reproduce byte-for-byte** from `70bd2c74`, including
    /// `computational-under-let`, which is the row carrying the load.
    ///
    /// ⭐ Read `computational-under-let`: the parent `Sequence` (n12) edges to
    /// **n11**, the computational match's scrutinee, and *not* to the
    /// `SourceReturnResume` (n6). That is D9's promise — the occurrence moved to
    /// the resume while the schedule stayed on the scrutinee — and this row is
    /// what would redden if a future change returned the resume as the entry.
    #[cfg(test)]
    const B2AC_BASE_TOPOLOGY: &[(&str, &str)] = &[
        ("leaf", "nodes=3 edges=1|n0:Terminal|n1:TrapTerminal|n2:Evaluate|e0:2->0:Continue|entries=[StaticNodeId(2)]"),
        ("let-if", "nodes=9 edges=8|n0:Terminal|n1:TrapTerminal|n2:Evaluate|n3:Evaluate|n4:Evaluate|n5:Branch|n6:Evaluate|n7:Evaluate|n8:Sequence|e0:2->0:Continue|e1:3->2:Continue|e2:4->2:Continue|e3:5->3:Select|e4:5->4:Reject|e5:6->5:Continue|e6:7->6:Continue|e7:8->7:Continue|entries=[StaticNodeId(8)]"),
        ("match", "nodes=7 edges=6|n0:Terminal|n1:TrapTerminal|n2:Evaluate|n3:Evaluate|n4:CaseTest|n5:Evaluate|n6:Evaluate|e0:2->1:Trap|e1:3->0:Continue|e2:4->3:Select|e3:4->2:Reject|e4:5->4:Continue|e5:6->5:Continue|entries=[StaticNodeId(6)]"),
        ("lexical-closure-call", "nodes=8 edges=7|n0:Terminal|n1:TrapTerminal|n2:Evaluate|n3:ClosureBody|n4:Evaluate|n5:Evaluate|n6:Evaluate|n7:Sequence|e0:2->0:Continue|e1:3->0:Continue|e2:4->3:Continue|e3:5->2:Continue|e4:6->5:Continue|e5:6->4:StaticBody|e6:7->6:Continue|entries=[StaticNodeId(7)]"),
        ("computational", "nodes=11 edges=10|n0:Terminal|n1:TrapTerminal|n2:CompletedTail|n3:ProducerTail|n4:ProducerWrapper|n5:SourceReturnResume|n6:Evaluate|n7:Evaluate|n8:CaseTest|n9:Evaluate|n10:Sequence|e0:5->4:InvokeProducerWrapper|e1:4->3:InvokeProducerTail|e2:3->2:CompleteProducerTail|e3:2->0:Continue|e4:6->1:Trap|e5:7->5:SourceReturnOwnedResume|e6:8->7:Select|e7:8->6:Reject|e8:9->8:Continue|e9:10->9:Continue|entries=[StaticNodeId(10)]"),
        ("computational-nested", "nodes=19 edges=19|n0:Terminal|n1:TrapTerminal|n2:CompletedTail|n3:ProducerTail|n4:ProducerWrapper|n5:SourceReturnResume|n6:Evaluate|n7:CompletedTail|n8:ProducerTail|n9:ProducerWrapper|n10:SourceReturnResume|n11:Evaluate|n12:Evaluate|n13:CaseTest|n14:Evaluate|n15:Sequence|n16:CaseTest|n17:Evaluate|n18:Sequence|e0:5->4:InvokeProducerWrapper|e1:4->3:InvokeProducerTail|e2:3->2:CompleteProducerTail|e3:2->0:Continue|e4:6->1:Trap|e5:10->9:InvokeProducerWrapper|e6:9->8:InvokeProducerTail|e7:8->7:CompleteProducerTail|e8:7->5:SourceReturnOwnedResume|e9:11->1:Trap|e10:12->10:SourceReturnOwnedResume|e11:13->12:Select|e12:13->11:Reject|e13:14->13:Continue|e14:15->14:Continue|e15:16->15:Select|e16:16->6:Reject|e17:17->16:Continue|e18:18->17:Continue|entries=[StaticNodeId(18)]"),
        ("computational-under-let", "nodes=13 edges=12|n0:Terminal|n1:TrapTerminal|n2:Evaluate|n3:CompletedTail|n4:ProducerTail|n5:ProducerWrapper|n6:SourceReturnResume|n7:Evaluate|n8:Evaluate|n9:CaseTest|n10:Evaluate|n11:Sequence|n12:Sequence|e0:2->0:Continue|e1:6->5:InvokeProducerWrapper|e2:5->4:InvokeProducerTail|e3:4->3:CompleteProducerTail|e4:3->2:Continue|e5:7->1:Trap|e6:8->6:SourceReturnOwnedResume|e7:9->8:Select|e8:9->7:Reject|e9:10->9:Continue|e10:11->10:Continue|e11:12->11:Continue|entries=[StaticNodeId(12)]"),
    ];

    #[test]
    fn boundary_a_topology_is_identical_to_the_pre_d9_planner() {
        let expected: BTreeMap<&str, &str> = B2AC_BASE_TOPOLOGY.iter().copied().collect();
        for (name, expr) in b2ac_topology_fixtures() {
            let digest = b2ac_topology_digest(&expr);
            let base = expected
                .get(name)
                .expect("every fixture has a recorded base digest");
            assert_eq!(
                &digest.as_str(),
                base,
                "AC-11: `{name}` changed the Boundary-A transfer graph. D9 must move \
                 only which identity is RECORDED at a source position, never which \
                 node is SCHEDULED."
            );
        }
    }

    /// **AC-13 — the split is exactly one variant.**
    #[test]
    fn computational_match_is_the_sole_entry_occurrence_split() {
        let mut split = Vec::new();
        for (name, expr) in b2ac_topology_fixtures() {
            let mut planner = Planner::new().expect("planner");
            let empty = PersistentNodeId(0);
            let context = PlanContext {
                environment: empty,
                continuation: empty,
                path: empty,
                cleanup: empty,
                affine: empty,
                source_return: empty,
            };
            let planned = planner
                .plan_expr(&expr, context, planner.terminal, EdgeKind::Continue, 0)
                .expect("plannable");
            if planned.occurrence != origin_of(planned.entry) {
                split.push(name);
            }
        }
        assert_eq!(
            split,
            vec!["computational", "computational-nested"],
            "AC-13: only a `ComputationalMatch` result may split entry from \
             occurrence, and every such result must. `computational-under-let` is \
             a `Let` at the root, so its own result does not split."
        );
    }

    /// **AC-14 — nested computational matches stay INJECTIVE even when several
    /// occurrences share a scheduling entry.**
    ///
    /// ⭐ This is the row a shallow test omits. In `computational-nested` the
    /// outer and inner matches are scheduled through the same chain, so a key
    /// taken from the *entry* would look unique while naming the wrong
    /// occurrence. The occurrences must differ, and each must resolve its own
    /// children.
    #[test]
    fn nested_computational_occurrences_stay_injective_under_a_shared_entry() {
        let (_, nested) = b2ac_topology_fixtures()
            .into_iter()
            .find(|(name, _)| *name == "computational-nested")
            .expect("the nested computational fixture");
        let plan = plan_static_transition_graph(&nested, &BTreeMap::new()).expect("plannable");
        let outer = plan.root_static_origin().expect("root occurrence");
        // The outer match's case body IS the inner match: child `1 + 0`.
        let inner = plan
            .child_static_origin(outer, 1)
            .expect("the outer match's case body resolves");
        assert_ne!(
            outer, inner,
            "AC-14: two computational occurrences must not share an origin"
        );
        // Each resolves its OWN children -- scrutinee at 0, case body at 1.
        for occurrence in [outer, inner] {
            plan.child_static_origin(occurrence, 0)
                .expect("scrutinee position resolves");
            plan.child_static_origin(occurrence, 1)
                .expect("case-body position resolves");
        }
        // And the shared scheduling entry is genuinely shared: the plan's single
        // root entry is a scrutinee chain node, not either occurrence.
        let entry = *plan.entries.first().expect("a root entry");
        assert_ne!(origin_of(entry), outer, "the entry is not the occurrence");
    }

    /// **`RT-FNSPLIT-B2A-S` AC-5 — keying selection by the scheduling ENTRY
    /// resolves to the WRONG body. Demonstrated, not forbidden by a grep.**
    ///
    /// ⛔ The first candidate discharged AC-5 by scanning for four container
    /// spellings keyed by `StaticNodeId`. The Architect rejected that
    /// (`evt_6sq2tq3v9jcd0`) and was right: a `Vec` indexed by `planned.entry.0`, a
    /// type alias, or a bespoke collection all violate the ruled property while
    /// such a scan stays green. **The property is about which value selects a body,
    /// so the control has to be about that too.**
    #[test]
    fn keying_selection_by_the_scheduling_entry_does_not_resolve_the_body() {
        // Promise class: durable invariant.
        let (_, computational) = b2ac_topology_fixtures()
            .into_iter()
            .find(|(name, _)| *name == "computational")
            .expect("the computational fixture");
        let plan =
            plan_static_transition_graph(&computational, &BTreeMap::new()).expect("plannable");

        let occurrence = plan.root_static_origin().expect("root occurrence");
        let entry = *plan.entries.first().expect("a root entry");
        assert_ne!(
            occurrence,
            origin_of(entry),
            "AC-5: the fixture must actually exhibit the split, or this test is vacuous"
        );

        // What the TAG resolves to: this match.
        let by_tag = plan
            .source_occurrence(occurrence)
            .expect("the occurrence resolves its own body");
        assert!(
            matches!(by_tag, RuntimeExpr::ComputationalMatch { .. }),
            "AC-5: the occurrence must resolve to the match itself"
        );

        // What an ENTRY-keyed lookup would resolve to: anything but this body. It
        // is either a different term or no source occurrence at all -- both are
        // wrong answers for "the body of this match", which is the point.
        let by_entry = plan.source_occurrence(origin_of(entry));
        assert!(
            !matches!(by_entry, Ok(term) if std::ptr::eq(term, by_tag)),
            "AC-5: the scheduling entry must not resolve to the occurrence's body; \
             if it does, entry and occurrence have been conflated again and \
             hard-stop #8 is back"
        );
    }

    /// **`RT-FNSPLIT-B2A-S` AC-5 — and entry-keying cannot be introduced QUIETLY,
    /// because filing two occurrences under one origin is refused.**
    ///
    /// ⭐ This is the mechanism that makes the property enforceable rather than
    /// merely stated. A `ComputationalMatch` shares its scheduling entry with its
    /// scrutinee chain, so a table keyed by `.entry` files two terms under one
    /// index — and `record_source_occurrence` rejects that outright.
    ///
    /// **Measured, not assumed:** replacing `expression_seed(resume, …)` with
    /// `expression_seed(scrutinee.entry, …)` — a compile-preserving mutation, and
    /// exactly the "key selection by `.entry`" change the Architect asked for —
    /// reddens **48** tests, **36** of them naming this invariant.
    #[test]
    fn filing_two_occurrences_under_one_origin_is_refused() {
        // Promise class: durable mutation proof.
        let (_, computational) = b2ac_topology_fixtures()
            .into_iter()
            .find(|(name, _)| *name == "computational")
            .expect("the computational fixture");

        let mut planner = Planner::new().expect("planner");
        let empty = PersistentNodeId(0);
        let context = PlanContext {
            environment: empty,
            continuation: empty,
            path: empty,
            cleanup: empty,
            affine: empty,
            source_return: empty,
        };
        planner
            .plan_expr(
                &computational,
                context,
                planner.terminal,
                EdgeKind::Continue,
                0,
            )
            .expect("plannable");

        // Any node that already owns an occurrence: re-filing it is the collision
        // an entry-keyed table would produce.
        let taken = planner
            .plan
            .semantic_sources
            .iter()
            .find_map(|seed| {
                matches!(seed.source, SemanticSourceKind::Expression(_))
                    .then_some(seed.planned_node)
            })
            .expect("the fixture plans at least one expression occurrence");
        assert_eq!(
            planner
                .record_source_occurrence(taken, &computational)
                .unwrap_err(),
            planner_error("static origin was given more than one source occurrence"),
            "AC-5: a second occurrence under one origin must be a loud planner \
             invariant, since that is what silently merges two bodies"
        );
    }

    /// **AC-15 — a root or transparent-declaration `ComputationalMatch` body
    /// receives the RESUME occurrence, not the scrutinee origin.**
    #[test]
    fn root_and_declaration_computational_bodies_take_the_resume_occurrence() {
        let (_, computational) = b2ac_topology_fixtures()
            .into_iter()
            .find(|(name, _)| *name == "computational")
            .expect("the computational fixture");

        // Root: the stored occurrence is the resume seed, and the scheduling
        // entry is the scrutinee -- so they must differ, and the occurrence must
        // resolve its own positional children.
        let plan =
            plan_static_transition_graph(&computational, &BTreeMap::new()).expect("plannable");
        let root = plan.root_static_origin().expect("root occurrence");
        let entry = *plan.entries.first().expect("a root entry");
        assert_ne!(
            root,
            origin_of(entry),
            "AC-15: a root computational match must not take its scrutinee's origin"
        );
        plan.child_static_origin(root, 0)
            .expect("the root occurrence resolves its scrutinee position");

        // Transparent declaration: same discriminator, by symbol.
        let declaration = RuntimeDeclaration {
            symbol: "decl:fixture::b2ac".to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: computational.clone(),
            },
            metadata: crate::RuntimeSymbolMetadata {
                obligations: Default::default(),
                obligation_metadata: Default::default(),
                assumptions: Default::default(),
                assumption_trust_metadata: Default::default(),
                trusted_base_delta: Default::default(),
                lowerability: None,
                unsupported: None,
                runtime_checks: Default::default(),
                capabilities: Default::default(),
                effects: Default::default(),
            },
        };
        let mut declarations = BTreeMap::new();
        declarations.insert("decl:fixture::b2ac", &declaration);
        let plan =
            plan_static_transition_graph(&RuntimeExpr::Var(0), &declarations).expect("plannable");
        let occurrence = plan
            .declaration_occurrence_origin("decl:fixture::b2ac")
            .expect("the transparent declaration has an occurrence origin");
        let declaration_entry = plan.entries[1];
        assert_ne!(
            occurrence,
            origin_of(declaration_entry),
            "AC-15: a declaration whose body is a computational match must not \
             take its scrutinee's origin"
        );
        plan.child_static_origin(occurrence, 0)
            .expect("the declaration occurrence resolves its scrutinee position");
    }

    /// Hard-stop #18 row 2 — declaration-call validation consumes the canonical
    /// node-indexed source view, never the planner's walk order.
    #[test]
    fn declaration_call_validation_positions_out_of_order_sources_once() {
        // Promise class: durable invariant plus a durable mutation proof.
        //
        // MEASURED: the exact DeclarationCall edge source names a
        // `DeclarationRef` in the canonical positioned view while the raw
        // walk-order slot at the same ordinal names a different source.
        // CLAIMED: validation indexes source semantics by StaticOriginId.
        // THE GAP: a fixture whose two views happen to agree cannot distinguish
        // positioned indexing from the rejected raw indexing, so the mismatch
        // assertions below are load-bearing.
        let symbol = "decl:fixture::b2o".to_string();
        let declaration =
            b2o_transparent_declaration(RuntimeExpr::Value(RuntimeValue::Int((73).into())));
        let declarations = BTreeMap::from([(symbol.as_str(), &declaration)]);
        let expr = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Row2::Node".to_string(),
                args: vec![unit()],
            }),
            cases: vec![crate::RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Row2::Node".to_string(),
                argument_binders: 1,
                recursive_positions: Vec::new(),
                body: RuntimeExpr::DeclarationRef {
                    symbol: symbol.clone(),
                },
            }],
            default: RuntimeTrap {
                code: RuntimeTrapCode::PatternMatchFailure,
                message: "row-2 fixture is total".to_string(),
            },
        };
        let plan = plan_static_transition_graph(&expr, &declarations)
            .expect("the out-of-order declaration call validates");
        let (edge_index, edge) = plan
            .edges
            .iter()
            .copied()
            .enumerate()
            .find(|(_, edge)| edge.kind == EdgeKind::DeclarationCall)
            .expect("the fixture has one declaration call edge");
        let node_indexed_sources =
            super::semantic_ir::positioned_sources(&plan.nodes, &plan.semantic_sources)
                .expect("the source population positions");
        assert_ne!(
            plan.semantic_sources[edge.from.0 as usize].source,
            SemanticSourceKind::Expression(RuntimeExprShape::DeclarationRef),
            "the fixture's raw walk-order slot agrees with node order, so it \
             cannot discriminate the rejected indexing"
        );
        assert_eq!(
            node_indexed_sources[edge.from.0 as usize].source,
            SemanticSourceKind::Expression(RuntimeExprShape::DeclarationRef),
            "the canonical positioned source does not name the call occurrence"
        );

        let call = plan
            .emittable_call_edges()
            .expect("the validated call edge projects")
            .into_iter()
            .find(|call| call.kind() == EmittableCallKind::Declaration)
            .expect("the declaration call remains separately typed");
        assert_eq!(call.call_site_origin(), origin_of(edge.from));
        assert_eq!(
            call.callee_origin(),
            plan.declaration_occurrence_origin(symbol.as_str())
                .expect("the transparent declaration owns one exact origin")
        );

        // Ordinary in-order control: positioning is not a special case for
        // ComputationalMatch and leaves an already positional call unchanged.
        let ordinary = RuntimeExpr::DeclarationRef {
            symbol: symbol.clone(),
        };
        let ordinary_plan = plan_static_transition_graph(&ordinary, &declarations)
            .expect("an ordinary declaration call remains valid");
        let ordinary_edge = ordinary_plan
            .edges
            .iter()
            .find(|edge| edge.kind == EdgeKind::DeclarationCall)
            .expect("the ordinary fixture has a declaration call edge");
        assert_eq!(
            ordinary_plan.semantic_sources[ordinary_edge.from.0 as usize].source,
            SemanticSourceKind::Expression(RuntimeExprShape::DeclarationRef)
        );

        // Redirect only the call-site source to a non-DeclarationRef occurrence
        // under the same owner. The source-shape invariant must still be the
        // exact detector; positioning repairs indexing, not validation.
        let caller_owner = plan.semantic.descriptors[edge.from.0 as usize].owner;
        let non_declaration_source = plan
            .nodes
            .iter()
            .map(|node| node.id)
            .find(|node| {
                *node != edge.from
                    && plan.semantic.descriptors[node.0 as usize].owner == caller_owner
                    && node_indexed_sources[node.0 as usize].source
                        != SemanticSourceKind::Expression(RuntimeExprShape::DeclarationRef)
            })
            .expect("the caller owns a non-DeclarationRef occurrence");
        let mut redirected_edges = plan.edges.clone();
        redirected_edges[edge_index].from = non_declaration_source;
        assert_eq!(
            plan.semantic
                .validate(
                    &plan.nodes,
                    &redirected_edges,
                    &plan.entries,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("declaration call edge source is not a DeclarationRef occurrence")
        );
    }

    /// RT-DECL-CLOSURE-PORT D2/D3/D5 — the declaration scheduling unit and its
    /// retained body are distinct owners with one phase-closed callable ABI.
    #[test]
    fn declaration_closure_units_derive_callable_layout_owner_and_phase_before_emission() {
        let symbol = "decl:fixture::decl_port::lexical".to_string();
        let declaration = RuntimeDeclaration {
            symbol: symbol.clone(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::LexicalClosure {
                    captures: vec![RuntimeExpr::Value(RuntimeValue::Bool(true))],
                    params: vec!["x".to_string()],
                    body: Box::new(RuntimeExpr::Var(0)),
                },
            },
            metadata: crate::RuntimeSymbolMetadata::empty(),
        };
        let expr = RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: symbol.clone(),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Bool(false))],
        };
        let declarations = BTreeMap::from([(symbol.as_str(), &declaration)]);
        let plan = plan_static_transition_graph_with_symbols(
            &expr,
            &declarations,
            &crate::NativeProcessSymbols::legacy_prelude(),
            AbiRootIngress::Value,
            true,
        )
        .expect("the declaration closure callable plan validates");
        let declaration_origin = plan
            .declaration_occurrence_origin(symbol.as_str())
            .expect("the declaration has one exact occurrence");
        let units = plan.emittable_units().expect("the units project");
        let wrapper = units
            .iter()
            .copied()
            .find(|unit| unit.origin() == declaration_origin)
            .expect("the declaration scheduling entry owns one unit");
        assert_eq!(
            wrapper.definition(),
            AbiUnitDefinition::TransparentDeclarationClosure {
                defining_origin: declaration_origin,
                provenance: AbiCaptureProvenance::Lexical,
            }
        );
        assert_eq!(wrapper.header().parameters, 1);
        assert_eq!(wrapper.header().captures, 1);

        let body_origin = plan
            .child_static_origin(declaration_origin, 0)
            .expect("the closure body has one exact origin");
        let body = units
            .iter()
            .copied()
            .find(|unit| unit.origin() == body_origin)
            .expect("the retained closure body owns one unit");
        assert_eq!(
            body.definition(),
            AbiUnitDefinition::ClosureBody {
                defining_origin: declaration_origin,
                provenance: AbiCaptureProvenance::Lexical,
            }
        );
        assert_eq!(body.header().parameters, wrapper.header().parameters);
        assert_eq!(body.header().captures, wrapper.header().captures);

        let root = plan.root_static_origin().expect("the call has an origin");
        assert_eq!(
            plan.join_plan_token(root)
                .expect("the call has a phase plan")
                .representation,
            JoinResultRepresentation::CarrierWord,
            "the DeclarationRef callable result did not close through the call phase"
        );

        DECLARATION_CLOSURE_DROP_CALLABLE_PHASE.with(|cell| cell.set(true));
        let dropped = plan_static_transition_graph_with_symbols(
            &expr,
            &declarations,
            &crate::NativeProcessSymbols::legacy_prelude(),
            AbiRootIngress::Value,
            true,
        )
        .expect("the phase mutation remains structurally plannable");
        DECLARATION_CLOSURE_DROP_CALLABLE_PHASE.with(|cell| cell.set(false));
        assert_eq!(
            dropped
                .join_plan_token(dropped.root_static_origin().expect("mutated call origin"))
                .expect("the mutated call still has a phase plan")
                .representation,
            JoinResultRepresentation::NativeScalarPair,
            "dropping callable phase left the result plan green, so the phase control is non-causal"
        );

        let wrapper_index = plan
            .abi
            .descriptors
            .iter()
            .position(|descriptor| descriptor.origin == declaration_origin)
            .expect("the wrapper has one descriptor");
        let mut wrong_owner = plan.abi.clone();
        wrong_owner.descriptors[wrapper_index].origin = body_origin;
        assert_eq!(
            wrong_owner
                .validate(
                    &plan.semantic,
                    &plan.nodes,
                    &plan.semantic_sources,
                    &plan.edges,
                    &plan.entries,
                    plan.root_entry.expect("root entry"),
                    plan.root_ingress,
                    &plan.static_callable_specializations,
                )
                .unwrap_err(),
            planner_error("abi descriptor is not positional for its function unit"),
            "a declaration callable assigned to the body owner survived validation"
        );

        let mut wrong_layout = plan.abi.clone();
        wrong_layout.descriptors[wrapper_index].header.parameters = 0;
        assert_eq!(
            wrong_layout
                .validate(
                    &plan.semantic,
                    &plan.nodes,
                    &plan.semantic_sources,
                    &plan.edges,
                    &plan.entries,
                    plan.root_entry.expect("root entry"),
                    plan.root_ingress,
                    &plan.static_callable_specializations,
                )
                .unwrap_err(),
            planner_error("abi descriptor parameter count is not its origin's declared arity"),
            "a declaration callable with the wrong parameter layout survived validation"
        );

        let mut zero_input = plan.abi.clone();
        zero_input.descriptors[wrapper_index].definition = AbiUnitDefinition::SchedulingEntry {
            ingress: AbiSchedulingIngress::Empty,
        };
        assert_eq!(
            zero_input
                .validate(
                    &plan.semantic,
                    &plan.nodes,
                    &plan.semantic_sources,
                    &plan.edges,
                    &plan.entries,
                    plan.root_entry.expect("root entry"),
                    plan.root_ingress,
                    &plan.static_callable_specializations,
                )
                .unwrap_err(),
            planner_error("abi descriptor definition is not the unit's derived definition"),
            "restoring the old zero-input scheduling ABI did not fail closed before emission"
        );
    }

    /// **AC-12 — every semantic child position consumes `.occurrence`.**
    ///
    /// Pinned at the type rather than by auditing call sites: both seed entry
    /// points take `&[StaticOriginId]`, and `StaticOriginId` can only be formed
    /// by `origin_of` inside this module, so a `StaticNodeId` cannot reach a
    /// child position at all.
    #[test]
    fn the_semantic_seed_api_accepts_only_occurrence_origins() {
        let source = include_str!("static_transition.rs");
        // ⚠ Count DECLARATION lines, not substring hits: this test's own
        // assertion text mentions both spellings, and a substring oracle would
        // fire on the prose that denies them.
        let declarations = source
            .lines()
            .filter(|line| line.trim() == "children: &[StaticOriginId],")
            .count();
        assert_eq!(
            declarations, 2,
            "AC-12: `expression_node` and `expression_seed` must both take \
             occurrence origins; a `&[StaticNodeId]` parameter here is the exact \
             conflation this parameter type exists to prevent"
        );
        assert!(
            !source
                .lines()
                .any(|line| line.trim() == "children: &[StaticNodeId],"),
            "AC-12: no semantic child list may be typed as scheduling nodes"
        );
    }

    // ---- RT-FNSPLIT-B2O — static body ownership -----------------------------

    fn b2o_transparent_declaration(body: RuntimeExpr) -> RuntimeDeclaration {
        RuntimeDeclaration {
            symbol: "decl:fixture::b2o".to_string(),
            kind: RuntimeDeclarationKind::Transparent { body },
            metadata: crate::RuntimeSymbolMetadata {
                obligations: Default::default(),
                obligation_metadata: Default::default(),
                assumptions: Default::default(),
                assumption_trust_metadata: Default::default(),
                trusted_base_delta: Default::default(),
                lowerability: None,
                unsupported: None,
                runtime_checks: Default::default(),
                capabilities: Default::default(),
                effects: Default::default(),
            },
        }
    }

    fn b2o_retained_closure(body: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures: Vec::new(),
            params: vec!["x".to_string()],
            body: Box::new(body),
        }
    }

    fn b2o_units(expr: &RuntimeExpr, declarations: &BTreeMap<&str, &RuntimeDeclaration>) -> usize {
        plan_static_transition_graph(expr, declarations)
            .expect("plannable")
            .semantic
            .functions
            .len()
    }

    /// `AC-4` — the unit set is exactly `plan.entries` ∪ `StaticBody` targets,
    /// with **three positive** controls.
    ///
    /// ⚠ A negative check ("no extra units") passes for any reason, including
    /// because nothing reached the checker. Each control here asserts a **delta**
    /// against a base fixture differing in exactly one way, so the count is
    /// attributable and no absolute number is frozen.
    #[test]
    fn b2o_ac4_each_seed_class_adds_exactly_one_function_unit() {
        // Promise class: durable invariant — a relation between fixtures.
        let base = unit();
        let none = BTreeMap::new();
        let base_units = b2o_units(&base, &none);
        // Non-vacuity: the base must already carry the root unit, or every "+1"
        // below could be measuring the root's arrival instead of the new seed.
        assert_eq!(base_units, 1, "the base fixture is the root unit alone");

        // Control 1 — one retained closure adds exactly one unit.
        assert_eq!(
            b2o_units(&b2o_retained_closure(unit()), &none),
            base_units + 1,
            "AC-4: one retained closure must add exactly one function unit"
        );

        // Control 2 — one transparent declaration adds exactly one unit.
        //
        // ⚠ This is the row an obvious test set omits. A closure/non-closure pair
        // does not exercise the **second top-level seed class** at all, so a
        // declaration-entry bug would pass every other control here. Two seed
        // classes require two positive controls.
        let declaration = b2o_transparent_declaration(unit());
        let mut declarations = BTreeMap::new();
        declarations.insert("decl:fixture::b2o", &declaration);
        assert_eq!(
            b2o_units(&base, &declarations),
            base_units + 1,
            "AC-4: one transparent declaration must add exactly one function unit"
        );

        // Control 3 — a non-closure expression inside an existing unit adds zero.
        let interior = RuntimeExpr::Let {
            value: Box::new(unit()),
            body: Box::new(unit()),
        };
        assert_eq!(
            b2o_units(&interior, &none),
            base_units,
            "AC-4: an expression inside an existing unit must add no unit"
        );
        // ...and it genuinely added planned nodes, so control 3 is not vacuous.
        let interior_nodes = plan_static_transition_graph(&interior, &none)
            .expect("plannable")
            .nodes
            .len();
        let base_nodes = plan_static_transition_graph(&base, &none)
            .expect("plannable")
            .nodes
            .len();
        assert!(
            interior_nodes > base_nodes,
            "control 3 proves nothing unless the interior expression added nodes"
        );
    }

    /// `AC-2` — totality and exclusivity are **pinned**, not merely structural.
    ///
    /// ⭐ "It is total by construction" is exactly the claim hard-stop #5 was
    /// defeated on: the carrier existed and the property did not follow. A
    /// structural guarantee still needs a check that *fires* if the construction
    /// changes.
    #[test]
    fn b2o_ac2_every_non_sentinel_node_has_exactly_one_in_range_function_owner() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).expect("plannable");

        let mut owned = 0usize;
        let mut terminals = 0usize;
        let mut trap_terminals = 0usize;
        for descriptor in &plan.semantic.descriptors {
            match descriptor.owner {
                SemanticOwner::Function(id) => {
                    assert!(
                        (id.0 as usize) < plan.semantic.functions.len(),
                        "an owner names a function unit outside the closed table"
                    );
                    owned += 1;
                }
                SemanticOwner::Terminal => terminals += 1,
                SemanticOwner::TrapTerminal => trap_terminals += 1,
            }
        }
        // The shared-exit population is EXACTLY the two sentinels — not "at
        // least", and not "whichever nodes ended up unowned".
        assert_eq!(
            (terminals, trap_terminals),
            (1, 1),
            "AC-2: the shared-exit population must be exactly one Terminal and \
             one TrapTerminal"
        );
        assert_eq!(
            owned,
            plan.nodes.len() - 2,
            "AC-2: every non-sentinel node must resolve to one Function owner"
        );
        // Non-vacuity: a single-unit fixture satisfies every line above while
        // proving nothing about exclusivity.
        assert!(
            plan.semantic.functions.len() >= 2,
            "this fixture has one unit, so exclusivity is untested here"
        );
    }

    /// `AC-3` — composition, **bidirectionally**, per `SemanticOpcode` variant.
    ///
    /// ⛔ Hard-stop #8 was predictable from the question its frame asked: the
    /// census answered `TOTAL` and was *true*, but the mechanism needed **closure
    /// under parent→child reachability**, a different property.
    /// `ComputationalMatch` files its occurrence on a different node from the
    /// entry its parent points at, so totality held while composition failed.
    /// This composes the child accessor with the owner map instead of measuring
    /// totality a second time.
    #[test]
    fn b2o_ac3_ownership_composes_down_and_up_for_every_opcode_variant() {
        // Promise class: durable invariant.
        let expr = nested_resource_bracket(3);
        let plan = plan_static_transition_graph(&expr, &BTreeMap::new()).expect("plannable");
        let owner_of = |origin: StaticOriginId| plan.semantic.descriptors[origin.0 as usize].owner;

        // The retained-body boundaries, read off the graph rather than assumed.
        let static_body = plan
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::StaticBody)
            .map(|edge| (edge.from, edge.to))
            .collect::<BTreeMap<_, _>>();

        let mut variants = BTreeSet::new();
        let mut boundary_children = 0usize;
        let mut interior_children = 0usize;
        for (position, record) in plan.semantic.records.iter().enumerate() {
            let parent = StaticOriginId(position as u32);
            variants.insert(record.opcode);
            let parent_owner = owner_of(parent);
            let crossing = static_body.get(&StaticNodeId(parent.0)).copied();
            for index in 0..record.child_origins.len as usize {
                let child = plan
                    .child_static_origin(parent, index)
                    .expect("a positional child origin");
                let child_owner = owner_of(child);
                match crossing {
                    // Child 0 of a closure occurrence IS its retained body. ⚠ Its
                    // *occurrence* may sit on a different node from the entry the
                    // StaticBody edge targets — that is the #8 shape — so this
                    // asserts the OWNER agrees, not that the nodes are equal.
                    Some(callee_seed) if index == 0 => {
                        let callee = match owner_of(StaticOriginId(callee_seed.0)) {
                            SemanticOwner::Function(id) => id,
                            other => panic!("a callee seed cannot be a shared exit: {other:?}"),
                        };
                        assert_eq!(
                            child_owner,
                            SemanticOwner::Function(callee),
                            "the retained body child must be owned by the callee unit"
                        );
                        assert_ne!(
                            child_owner, parent_owner,
                            "the retained body child must not stay in the caller's unit"
                        );
                        boundary_children += 1;
                    }
                    // Every other child — a capture, or any child of a
                    // non-closure — stays inside the parent's own unit.
                    _ => {
                        assert_eq!(
                            child_owner, parent_owner,
                            "descending to a non-boundary child left the parent's unit"
                        );
                        interior_children += 1;
                    }
                }
            }
        }

        // ⭐ The "up" half. The boundary crossed on descent is represented by the
        // **callee seed**, and the body's return node stays inside the
        // **callee's** owner rather than being handed back to the caller. This is
        // AC-5 control 8's property stated positively.
        //
        // ⛔ `B2O` invents no static edge back to the caller — `B2R` carries the
        // dynamic return continuation — so "up" is checked as *the return node is
        // callee-owned and exits only through a shared exit*, never as a
        // cross-owner edge this node manufactured.
        let mut returns = 0usize;
        for node in &plan.nodes {
            if node.transition != TransitionKind::ClosureBody {
                continue;
            }
            returns += 1;
            let SemanticOwner::Function(unit) = plan.semantic.descriptors[node.id.0 as usize].owner
            else {
                panic!("a ClosureBody return successor must be owned by a function unit");
            };
            let seed = plan.semantic.functions[unit.0 as usize].planned_node;
            assert!(
                static_body.values().any(|target| *target == seed),
                "the ClosureBody return successor is owned by a unit that is not a callee"
            );
            let exits = plan
                .edges
                .iter()
                .filter(|edge| edge.from == node.id)
                .collect::<Vec<_>>();
            assert!(
                !exits.is_empty(),
                "a return successor with no exit proves nothing"
            );
            for edge in exits {
                assert!(
                    matches!(
                        plan.semantic.descriptors[edge.to.0 as usize].owner,
                        SemanticOwner::Terminal | SemanticOwner::TrapTerminal
                    ),
                    "a ClosureBody return successor must exit only through a shared exit"
                );
            }
        }

        // ⛔ No silent caps. Say what was exercised and fail if a class never
        // appeared — an assertion nothing reached is green for the wrong reason.
        assert_eq!(
            variants.len(),
            6,
            "AC-3 requires EVERY SemanticOpcode variant, not a sampled few; this \
             fixture exercised {variants:?}"
        );
        assert!(boundary_children > 0, "no boundary child was exercised");
        assert!(interior_children > 0, "no interior child was exercised");
        assert!(returns > 0, "no ClosureBody return successor was exercised");
    }

    /// A fixture with two retained closures **and** a transparent declaration, so
    /// every seed class and both `AC-5` duplicate/overlap shapes are constructible.
    fn b2o_two_closure_fixture() -> RuntimeExpr {
        RuntimeExpr::Let {
            value: Box::new(b2o_retained_closure(unit())),
            body: Box::new(b2o_retained_closure(RuntimeExpr::Var(0))),
        }
    }

    fn b2o_err(
        plane: &SemanticPlane,
        nodes: &[StaticNode],
        edges: &[StaticEdge],
        entries: &[StaticNodeId],
        plan: &StaticTransitionPlan,
    ) -> CraneliftBackendError {
        plane
            .validate(
                nodes,
                edges,
                entries,
                &plan.semantic_sources,
                &plan.semantic_material,
            )
            .expect_err("the control must redden")
    }

    /// `AC-5` — every ownership law is enforced, each with its own **independent**
    /// redden control, constructed and confirmed to error **before emission**.
    ///
    /// ⛔ A pin that enumerates spellings is not a proof of the property. Each
    /// control below mutates the *graph, the seeds, or the recorded owner* — never
    /// a string — and every mutation keeps the code compiling.
    ///
    /// ⚠ **Honest residual, and it is a finding about the AC rather than a gap in
    /// the mechanism.** Rows 5 and 6 of `AC-5` land on the **same** detector, and
    /// they must: because ownership is *derived* by traversal from seeds over
    /// non-`StaticBody` edges and then compared against the record, any ordinary
    /// cross-owner edge necessarily makes some node reachable from two seeds. So
    /// "a non-`StaticBody` cross-owner edge" **is** an overlap, and no data
    /// mutation can produce one without producing the other. Both are constructed
    /// below and both redden; what cannot be claimed is that they exercise two
    /// independent checks. The `D3` edge laws are still checked, because they
    /// constrain the **algorithm** — but as **defense in depth behind overlap,
    /// not as the primary detector.** Measured: a traversal edited to cross
    /// `StaticBody` reddens at **overlap** (mutation M1), because the callee's
    /// seed is claimed by the caller; the "crosses to a *distinct* unit" law is
    /// the sole detector only once overlap is **also** disabled (mutation M2).
    /// The genuinely independent edge-law control is the sentinel one (5b).
    ///
    /// ⭐ Note the shape of my own error here, since it is the reusable part: I
    /// identified this exact detector-collapse for the *data*-mutation route in
    /// the paragraph above, then asserted the opposite for the *code*-mutation
    /// route one sentence later. Having found one collapse, sweep every route to
    /// the property before writing prose about any of them.
    #[test]
    fn b2o_ac5_each_ownership_law_reddens_on_its_own() {
        // Promise class: durable mutation proof.
        let declaration = b2o_transparent_declaration(unit());
        let mut declarations = BTreeMap::new();
        declarations.insert("decl:fixture::b2o", &declaration);
        let expr = b2o_two_closure_fixture();
        let plan = plan_static_transition_graph(&expr, &declarations).expect("plannable");

        // Non-vacuity of the fixture itself, before any control runs.
        assert!(
            plan.entries.len() >= 2,
            "controls 1 and 2 need a root AND a declaration entry"
        );
        let static_body = plan
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::StaticBody)
            .map(|edge| (edge.from, edge.to))
            .collect::<Vec<_>>();
        assert!(
            static_body.len() >= 2,
            "control 4 needs two static body edges to alias"
        );
        plan.semantic
            .validate(
                &plan.nodes,
                &plan.edges,
                &plan.entries,
                &plan.semantic_sources,
                &plan.semantic_material,
            )
            .expect("the unmutated plane must validate, or every control below is vacuous");

        let unowned = planner_error("planned node has no function unit owner");
        let population = planner_error(
            "function unit population is not the scheduling entries and static body targets",
        );

        // 1 — a missing ROOT entry. The root's subgraph is then reachable from no
        //     seed at all.
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &plan.edges,
                &plan.entries[1..],
                &plan
            ),
            unowned,
            "AC-5.1: dropping the root entry must redden"
        );

        // 2 — a missing TRANSPARENT DECLARATION entry. ⚠ Independent of control 1:
        //     a checker that only knew about the root would pass 1 and fail here.
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &plan.edges,
                &plan.entries[..1],
                &plan
            ),
            unowned,
            "AC-5.2: dropping a transparent declaration entry must redden"
        );

        // 3 — a missing StaticBody TARGET: demote one StaticBody edge to an
        //     ordinary transfer, so its body stops being a seed.
        let mut demoted = plan.clone();
        let victim = demoted
            .edges
            .iter()
            .find(|edge| edge.kind == EdgeKind::StaticBody)
            .copied()
            .expect("a static body edge");
        rewrite_edge(
            &mut demoted,
            victim.id,
            victim.from,
            victim.to,
            EdgeKind::Continue,
        );
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &demoted.edges,
                &plan.entries,
                &plan
            ),
            population,
            "AC-5.3: dropping a static body target must redden"
        );

        // 4 — a DUPLICATE StaticBody target: point the second boundary edge at
        //     the first one's body.
        let mut aliased = plan.clone();
        let second = aliased
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::StaticBody)
            .nth(1)
            .copied()
            .expect("a second static body edge");
        rewrite_edge(
            &mut aliased,
            second.id,
            second.from,
            static_body[0].1,
            EdgeKind::StaticBody,
        );
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &aliased.edges,
                &plan.entries,
                &plan
            ),
            planner_error("static body target has more than one incoming static body edge"),
            "AC-5.4: a duplicate static body target must redden, not be deduplicated"
        );

        // 5 — a non-StaticBody CROSS-OWNER edge: reach from the root's unit
        //     straight into a body-owned node. See the honest residual above —
        //     this reddens at the overlap detector, by construction.
        let mut crossed = plan.clone();
        let root_entry = plan.entries[0];
        append_edge(
            &mut crossed,
            root_entry,
            static_body[0].1,
            EdgeKind::Continue,
        );
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &crossed.edges,
                &plan.entries,
                &plan
            ),
            planner_error("planned node is owned by more than one function unit"),
            "AC-5.5: a non-static-body cross-owner edge must redden"
        );

        // 5b — the genuinely independent EDGE-LAW control: an outgoing edge from
        //      a shared exit. A sentinel is never traversed from, so this creates
        //      no overlap and reaches the edge law itself.
        let mut exiting = plan.clone();
        let terminal = plan
            .nodes
            .iter()
            .find(|node| node.transition == TransitionKind::Terminal)
            .expect("the shared terminal")
            .id;
        append_edge(&mut exiting, terminal, root_entry, EdgeKind::Continue);
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &exiting.edges,
                &plan.entries,
                &plan
            ),
            planner_error("shared exit has an outgoing transfer edge"),
            "AC-5.5b: the edge law must reject an edge leaving a shared exit"
        );

        // 6 — OVERLAP by a spurious extra seed: name an already-owned interior
        //     node as a scheduling entry. ⚠ A different construction from control
        //     5 — a bad seed rather than a bad edge.
        //
        // ⚠ Select it by its OWNER, not by excluding the node kinds I happen to
        // think of. Picking "not the terminal, not an entry, not a static body
        // target" selected the **trap** terminal, which is never traversed from,
        // so the control created no overlap and reddened at the population check
        // instead — green-for-the-wrong-reason, caught only because this control
        // asserts the exact error rather than merely `is_err`.
        let root_unit = SemanticOwner::Function(PredeclaredFunctionId(0));
        let interior = plan
            .nodes
            .iter()
            .map(|node| node.id)
            .find(|id| {
                *id != root_entry && plan.semantic.descriptors[id.0 as usize].owner == root_unit
            })
            .expect("an interior node inside the root unit");
        assert!(
            !plan.entries.contains(&interior)
                && !static_body.iter().any(|(_, target)| *target == interior),
            "control 6 needs a node that is not already a seed"
        );
        let mut extra_entries = plan.entries.clone();
        extra_entries.push(interior);
        assert_eq!(
            b2o_err(
                &plan.semantic,
                &plan.nodes,
                &plan.edges,
                &extra_entries,
                &plan
            ),
            planner_error("planned node is owned by more than one function unit"),
            "AC-5.6: an ordinary node owned by two seeds must redden"
        );

        // 7 — a SENTINEL misclassified as a Function.
        let mut misclassified = plan.semantic.clone();
        misclassified.descriptors[terminal.0 as usize].owner =
            SemanticOwner::Function(PredeclaredFunctionId(0));
        assert_eq!(
            b2o_err(
                &misclassified,
                &plan.nodes,
                &plan.edges,
                &plan.entries,
                &plan
            ),
            planner_error("semantic descriptor owner is not the node's derived function unit"),
            "AC-5.7: a shared exit recorded as a function unit must redden"
        );

        // 8 — a `ClosureBody` return successor assigned to the CALLER.
        //
        // ⚠ This is the one that would otherwise ship green: assigning the return
        // node to the caller is the *intuitive* reading of "the caller resumes
        // here", it produces a coherent-looking partition, and only the down/up
        // invariant catches it.
        let return_node = plan
            .nodes
            .iter()
            .find(|node| node.transition == TransitionKind::ClosureBody)
            .expect("a ClosureBody return successor")
            .id;
        let caller_owner = plan.semantic.descriptors[static_body[0].0 .0 as usize].owner;
        let callee_owner = plan.semantic.descriptors[return_node.0 as usize].owner;
        assert_ne!(
            caller_owner, callee_owner,
            "control 8 proves nothing unless the caller and callee units differ"
        );
        let mut handed_back = plan.semantic.clone();
        handed_back.descriptors[return_node.0 as usize].owner = caller_owner;
        assert_eq!(
            b2o_err(&handed_back, &plan.nodes, &plan.edges, &plan.entries, &plan),
            planner_error("semantic descriptor owner is not the node's derived function unit"),
            "AC-5.8: a return successor assigned to the caller must redden"
        );
    }

    /// The semantic disposition of a plan, as the ruling's four classification
    /// laws project it. **This is the authority** — an occurrence's owner and the
    /// planned edge kind — and it is deliberately computed from nothing else.
    ///
    /// ⚠ There is no Rust identifier, file name, method name or source offset
    /// anywhere in this function, and that absence is the point: it is why a Rust
    /// wrapper cannot move the result.
    fn b2o_disposition(plan: &StaticTransitionPlan) -> (usize, usize, usize, usize) {
        let owner_of = |node: StaticNodeId| plan.semantic.descriptors[node.0 as usize].owner;
        let (mut cross_owner, mut intra_owner, mut shared_exit, mut other) = (0, 0, 0, 0);
        for edge in &plan.edges {
            match (owner_of(edge.from), owner_of(edge.to), edge.kind) {
                // Law 1 — a `StaticBody` edge between DISTINCT function owners is
                // a cross-owner call boundary.
                (SemanticOwner::Function(a), SemanticOwner::Function(b), EdgeKind::StaticBody)
                    if a != b =>
                {
                    cross_owner += 1
                }
                // Law 3 — a function edge to either shared exit is the validated
                // return/trap, never a call.
                (
                    SemanticOwner::Function(_),
                    SemanticOwner::Terminal | SemanticOwner::TrapTerminal,
                    _,
                ) => shared_exit += 1,
                // Law 2 — an ordinary edge inside one owner is local traversal.
                (SemanticOwner::Function(a), SemanticOwner::Function(b), _) if a == b => {
                    intra_owner += 1
                }
                // Law 4 — everything else is a graph planning refuses to build.
                _ => other += 1,
            }
        }
        (
            plan.semantic.functions.len(),
            cross_owner,
            intra_owner,
            shared_exit + other,
        )
    }

    /// `AC-10a` / `AC-10b` — **the harness that must stay GREEN under a Rust
    /// refactor.** Architect ruling `evt_5yxjd1zqnyvcq`.
    ///
    /// ⛔ **The verdict here is INVERTED from the four withdrawn folds.** Those
    /// spent four candidate SHAs making a relocation redden. A Rust wrapper, a
    /// nested `fn`, or a same-named method in a second `impl` creates **no Ken
    /// function-unit boundary**, so a pin that reddens on one is measuring
    /// implementation topology and reporting success.
    ///
    /// - **MEASURED:** the unit count and the three edge-classification counts,
    ///   derived from owners and edge kinds alone.
    /// - **CLAIMED:** that semantic disposition is a function of the plan graph,
    ///   so no source-level reorganisation can move it.
    /// - **THE GAP:** ⚠ a green here proves invariance only for mutations that
    ///   were actually *applied*. That is why `AC-10a`/`10b` are recorded
    ///   **mutation proofs against this pin**, and why `AC-10c` exists at all —
    ///   without it, deleting the assertion below would leave this green forever.
    ///
    /// Promise class: **transition sentinel** — the four numbers are a frozen
    /// snapshot of *this fixture*; the durable claim is their **invariance under
    /// source refactoring**, which only the recorded mutation proofs discharge.
    #[test]
    fn b2o_ac10_semantic_disposition_is_a_function_of_the_plan_graph_alone() {
        let declaration = b2o_transparent_declaration(unit());
        let mut declarations = BTreeMap::new();
        declarations.insert("decl:fixture::b2o", &declaration);
        let expr = b2o_two_closure_fixture();
        let plan = plan_static_transition_graph(&expr, &declarations).expect("plannable");

        let (units, cross_owner, intra_owner, exits) = b2o_disposition(&plan);

        // Non-vacuity before the snapshot: a fixture with no boundary would make
        // every claim below true for the wrong reason.
        assert!(
            cross_owner > 0,
            "the fixture has no cross-owner boundary, so 10a/10b would be green \
             on a harness that observes nothing"
        );
        assert_eq!(
            units,
            plan.entries.len()
                + plan
                    .edges
                    .iter()
                    .filter(|edge| edge.kind == EdgeKind::StaticBody)
                    .count(),
            "the unit population is the ruled seed set"
        );
        // ⚠ PREDICTED (2, 6, 3) before running; MEASURED (2, 4, 4). The
        // cross-owner count was right and the intra/exit split was not — I had
        // both retained-closure bodies reaching the terminal through one more
        // ordinary hop than they do. Recorded as a miss rather than silently
        // re-fitted, because a number edited to match an observation measures
        // nothing (`AC-11`, and the `D5` predictions before it).
        assert_eq!(
            (cross_owner, intra_owner, exits),
            (2, 4, 4),
            "AC-10: the semantic disposition of this fixture moved.\n\
             ⚠ If you reached this by RELOCATING A RUST CALL, adding a wrapper, \
             or adding a same-named method in another `impl`, the pin is not the \
             thing that is wrong -- the ruling is explicit that such a refactor \
             creates no Ken function-unit boundary and this MUST stay green. \
             Investigate why the plan graph moved.\n\
             If you reached it by changing the planner's edges or seeds, that IS \
             a semantic change and belongs in review."
        );
    }

    /// `AC-10c` — **the RED twin that makes `AC-10a`/`10b` mean something.**
    ///
    /// ⭐ Without this, `b2o_ac10_...` is green on a harness that observes
    /// nothing at all: delete its assertion body and the relocation proofs stay
    /// green forever. This control mutates the **one axis that IS authority** —
    /// the planned edge's owner endpoints — and requires both that the projection
    /// moves and that validation refuses the graph.
    #[test]
    fn b2o_ac10c_repointing_a_static_body_edge_changes_the_disposition() {
        let declaration = b2o_transparent_declaration(unit());
        let mut declarations = BTreeMap::new();
        declarations.insert("decl:fixture::b2o", &declaration);
        let expr = b2o_two_closure_fixture();
        let plan = plan_static_transition_graph(&expr, &declarations).expect("plannable");

        let before = b2o_disposition(&plan);

        // Repoint one `StaticBody` edge at a node owned by the SAME unit as its
        // source. Compile-preserving, and it is exactly the "moved boundary" a
        // source-text oracle could never see.
        let (index, edge) = plan
            .edges
            .iter()
            .enumerate()
            .find(|(_, edge)| edge.kind == EdgeKind::StaticBody)
            .map(|(i, e)| (i, *e))
            .expect("the fixture must carry a static body edge");
        let source_owner = plan.semantic.descriptors[edge.from.0 as usize].owner;
        let same_unit_target = plan
            .nodes
            .iter()
            .map(|node| node.id)
            .find(|id| {
                plan.semantic.descriptors[id.0 as usize].owner == source_owner && *id != edge.to
            })
            .expect("the caller unit must hold a second node to repoint at");

        let mut edges = plan.edges.clone();
        edges[index].to = same_unit_target;

        let after = {
            let mut repointed = plan.clone();
            repointed.edges = edges.clone();
            b2o_disposition(&repointed)
        };
        assert_ne!(
            before, after,
            "AC-10c: repointing a static body edge left the projection unchanged, \
             so the harness does not observe semantic disposition and 10a/10b are \
             vacuous"
        );

        // ⭐ I named the WRONG detector here and the exact-error assertion caught
        // it. I predicted `"static body edge does not cross a function unit
        // boundary"` -- the edge law. It reddens at **overlap** instead, and that
        // is correct and already documented: repointing the edge inside one unit
        // makes the target reachable from the caller's seed *while still being a
        // seed itself*, so the partition sees two owners before any edge law is
        // consulted. This independently re-confirms the corrected `D3` note that
        // overlap is the primary detector and the edge law is defense in depth.
        //
        // ⚠ `expect_err` would have been GREEN here and would have taught the
        // next reader that the edge law is load-bearing. Asserting the exact
        // error is the only reason this was visible.
        assert_eq!(
            b2o_err(&plan.semantic, &plan.nodes, &edges, &plan.entries, &plan),
            planner_error("planned node is owned by more than one function unit"),
            "AC-10c: a static body edge repointed inside one unit must be REFUSED \
             by planning, not merely reclassified"
        );
    }

    // ================================================================
    // `RT-FNSPLIT-B2R` — the representation and call-ABI contract.
    //
    // ⛔ Every control below mutates the **graph, the owner partition, or the
    // recorded descriptor** — never a Rust spelling. `AC-8` inverts the usual
    // reflex: a rename, a wrapper, a visibility change, or a `fn` moved between
    // files MUST leave these green, and a pin that reddens on one of those is a
    // defect in the pin, reported as such rather than repaired into greenness.
    // ================================================================

    /// A closure whose captures arrive by the **seed** provenance: the captures
    /// are symbols resolved against the seed environment at JIT time.
    fn b2r_seed_closure(captures: &[&str], body: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Closure {
            captures: captures.iter().map(|c| (*c).to_string()).collect(),
            params: vec!["x".to_string()],
            body: Box::new(body),
        }
    }

    /// A closure whose captures arrive by the **lexical** provenance: each
    /// capture is an arbitrary source expression, planned as a syntax child.
    fn b2r_lexical_closure(captures: Vec<RuntimeExpr>, body: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::LexicalClosure {
            captures,
            params: vec!["x".to_string()],
            body: Box::new(body),
        }
    }

    fn b2r_plan(expr: &RuntimeExpr) -> StaticTransitionPlan<'_> {
        let declarations = BTreeMap::new();
        plan_static_transition_graph(expr, &declarations).expect("plannable")
    }

    /// `AC-1` — descriptor totality over the owner partition, **both
    /// directions**.
    ///
    /// ⚠ A one-directional check passes happily on an orphan, so both are
    /// asserted: every unit has exactly one descriptor, and every descriptor
    /// names a member of the partition.
    ///
    /// Promise class: **durable invariant** — a relation between two populations,
    /// not a frozen count.
    #[test]
    fn b2r_ac1_every_function_unit_has_exactly_one_descriptor_and_conversely() {
        let expr = b2r_lexical_closure(Vec::new(), RuntimeExpr::Var(0));
        let plan = b2r_plan(&expr);

        // Non-vacuity FIRST: a plane with one unit would make both directions
        // true for the wrong reason, and every claim below would be green on a
        // fixture that never exercised a boundary.
        assert!(
            plan.semantic.functions.len() > 1,
            "the fixture has only one function unit, so totality is trivially \
             true and this control observes nothing"
        );

        // Direction 1 — every unit is covered.
        assert_eq!(
            plan.abi.descriptors.len(),
            plan.semantic.functions.len(),
            "AC-1: the descriptor population is not exact for the function unit \
             partition"
        );
        // Direction 2 — every descriptor names a member, positionally.
        for (ordinal, descriptor) in plan.abi.descriptors.iter().enumerate() {
            let function = &plan.semantic.functions[ordinal];
            assert_eq!(descriptor.function, function.id, "AC-1: descriptor/unit id");
            assert_eq!(
                descriptor.planned_node, function.planned_node,
                "AC-1: a descriptor names a node that is not its unit's seed"
            );
        }

        // And an ORPHAN must be refused, so direction 2 is a real detector
        // rather than a restatement of how the builder happens to loop.
        let mut orphaned = plan.abi.clone();
        orphaned.descriptors.pop();
        let err = orphaned
            .validate(
                &plan.semantic,
                &plan.nodes,
                &plan.semantic_sources,
                &plan.edges,
                &plan.entries,
                plan.root_entry.expect("root entry"),
                plan.root_ingress,
                &plan.static_callable_specializations,
            )
            .expect_err("AC-1: dropping a descriptor must be refused");
        // ⛔ The EXACT failure, not `is_err()`. A control that reddens does not
        // confirm which detector caught it, and `is_err()` would stay green if
        // some unrelated law started firing first.
        assert!(
            format!("{err:?}")
                .contains("not exact for graph units plus static callable specializations"),
            "AC-1: the orphan was refused, but not by the totality law. Got: {err:?}"
        );
    }

    /// `AC-2` / `C1` — **a descriptor post-condition, not a census of the 44
    /// caller-environment append sites.**
    ///
    /// ⭐ The site census is a spelling standing in for a population: the frame
    /// measured 44 sites across two spellings, and the site `C1` names is in the
    /// spelling a sweep written against the other one **excludes**. This control
    /// is mechanism-independent instead — it holds whether the environment is
    /// appended, cloned, threaded or restructured, and it still holds at the
    /// 45th site.
    ///
    /// Promise class: **durable invariant.**
    #[test]
    fn b2r_ac2_an_irrelevant_caller_binding_does_not_change_the_callee_descriptor() {
        let inner = b2r_lexical_closure(Vec::new(), RuntimeExpr::Var(0));
        let wrapped = RuntimeExpr::Let {
            value: Box::new(unit()),
            body: Box::new(b2r_lexical_closure(Vec::new(), RuntimeExpr::Var(0))),
        };

        let bare = b2r_plan(&inner);
        let deeper = b2r_plan(&wrapped);

        // ⚠ Non-vacuity: the extra binding must actually have changed the plan.
        // Comparing two identical plans would pass for the wrong reason.
        assert!(
            deeper.nodes.len() > bare.nodes.len(),
            "AC-2: the irrelevant binding did not change the plan, so descriptor \
             invariance is being asserted against an unchanged input"
        );

        // The unit count is unchanged: an irrelevant binding adds no scheduling
        // entry and no static body edge.
        assert_eq!(
            deeper.semantic.functions.len(),
            bare.semantic.functions.len(),
            "AC-2: an irrelevant caller binding changed the function unit count"
        );

        // ⭐ SHAPE, not identity. `planned_node`/`origin` are positional over the
        // node table and legitimately move when the table grows; the LAYOUT must
        // not. This narrowing was recorded in the predictions file (`P2`) BEFORE
        // measuring, so it is a stated design choice rather than a red assertion
        // trimmed until it passed.
        assert_eq!(
            deeper.abi.shapes().expect("shapes"),
            bare.abi.shapes().expect("shapes"),
            "AC-2/C1: adding an irrelevant caller binding changed a callee \
             descriptor's slot count or layout, which is the caller-depth \
             dependence this node exists to remove"
        );
    }

    /// `AC-3` / `C2` — **both** capture provenances produce a declared slot with
    /// a declared layout, and they are a **non-degenerate discriminator pair.**
    ///
    /// ⚠ A single positive case is green-vs-green under the exact swap it should
    /// catch. The two provenances are exercised on the **same** closure shape,
    /// differing only in how their captures arrive, so a collapse of the two into
    /// one carrier fails **both** sides rather than neither.
    ///
    /// ⭐ **Where the real enforcement lives.** That a seed layout is not chosen
    /// by inspecting the particular runtime value is enforced by the
    /// **signature**: `AbiCaptureProvenance::carrier` takes no value, and
    /// `build_abi_plane`'s inputs contain no `RuntimeGroundValue` and no
    /// `Lowered`. There is nothing to inspect. This test is a positive control
    /// that the mechanism is reachable and discriminating — not the enforcement.
    ///
    /// Promise class: **durable invariant.**
    #[test]
    fn b2r_ac3_both_capture_provenances_declare_slots_and_select_distinct_carriers() {
        let seeded_expr = b2r_seed_closure(&["c"], RuntimeExpr::Var(0));
        let lexical_expr = b2r_lexical_closure(vec![unit()], RuntimeExpr::Var(0));
        let seeded = b2r_plan(&seeded_expr);
        let lexical = b2r_plan(&lexical_expr);

        let seed_capture = b2r_only_capture_slot(&seeded);
        let lexical_capture = b2r_only_capture_slot(&lexical);

        // Both declare a slot with a declared layout — kind, carrier, ownership,
        // width and alignment, none of them absent or defaulted.
        for (label, slot) in [("seed", seed_capture), ("lexical", lexical_capture)] {
            assert_eq!(slot.kind, AbiSlotKind::Capture, "AC-3: {label} slot kind");
            assert_eq!(slot.width_bytes, 8, "AC-3: {label} declared width");
            assert_eq!(slot.align_bytes, 8, "AC-3: {label} declared alignment");
        }

        // ⭐ The discriminator: the two provenances select DIFFERENT carriers on
        // the same closure shape. Collapsing them would fail this, and a swap of
        // the two would fail it too.
        assert_eq!(
            seed_capture.carrier,
            AbiCarrier::GroundValueCarrier,
            "AC-3/C2: a seed capture must travel in the fixed closed carrier for \
             the permitted ground-value family"
        );
        assert_eq!(
            lexical_capture.carrier,
            AbiCarrier::ValueWord,
            "AC-3/C2: a lexical capture travels in the ordinary value carrier"
        );
        assert_ne!(
            seed_capture.carrier, lexical_capture.carrier,
            "AC-3/C2: the two provenances collapsed to one carrier, so a pin \
             keyed to either one would be a spelling standing in for the \
             population"
        );
    }

    /// `AC-4` / `C3` — *the transported payload may change; the ABI may not.*
    ///
    /// Two required controls, plus the positive rejection control that stops the
    /// negative half from passing because nothing reached the checker.
    ///
    /// Promise class: **durable invariant** for the invariance halves; **durable
    /// mutation proof** for the rejection half.
    #[test]
    fn b2r_ac4_the_abi_is_invariant_under_payload_and_depth_and_rejects_an_implicit_tail() {
        // Control 1 — caller DEPTH changes, per-origin descriptor is identical.
        let shallow_expr = b2r_seed_closure(&["c"], RuntimeExpr::Var(0));
        let deep_expr = RuntimeExpr::Let {
            value: Box::new(unit()),
            body: Box::new(RuntimeExpr::Let {
                value: Box::new(unit()),
                body: Box::new(b2r_seed_closure(&["c"], RuntimeExpr::Var(0))),
            }),
        };
        let shallow = b2r_plan(&shallow_expr);
        let deep = b2r_plan(&deep_expr);
        assert!(
            deep.nodes.len() > shallow.nodes.len() + 1,
            "AC-4: the depth control did not actually deepen the caller"
        );
        assert_eq!(
            deep.abi.shapes().expect("shapes"),
            shallow.abi.shapes().expect("shapes"),
            "AC-4/C3: the per-origin descriptor varied with CALLER DEPTH"
        );

        // Control 2 — the seed capture's payload changes within its declared
        // carrier class. ⭐ Renaming the captured symbol changes WHICH ground
        // value the seed environment will supply at JIT time; the descriptor's
        // shape must not move. The carrier cannot vary with the value because no
        // value is in scope to vary it — this control observes that the arity and
        // layout are likewise untouched.
        let other_payload_expr = b2r_seed_closure(&["a-different-capture"], RuntimeExpr::Var(0));
        let other_payload = b2r_plan(&other_payload_expr);
        assert_eq!(
            other_payload.abi.shapes().expect("shapes"),
            shallow.abi.shapes().expect("shapes"),
            "AC-4/C3: the descriptor shape moved when the transported payload did"
        );

        // ⚠ Control 3 — the POSITIVE control. "The validator rejects an implicit
        // caller-env tail" passes for any reason, including that nothing ever
        // reached the checker. So construct one and observe the rejection.
        let mut tailed = shallow.abi.clone();
        let last = tailed
            .descriptors
            .last_mut()
            .expect("the fixture has at least one descriptor");
        last.slots.len += 1;
        let tail_slot = *tailed.slots.last().expect("the fixture has slots");
        tailed.slots.push(tail_slot);
        let err = tailed
            .validate(
                &shallow.semantic,
                &shallow.nodes,
                &shallow.semantic_sources,
                &shallow.edges,
                &shallow.entries,
                shallow.root_entry.expect("root entry"),
                shallow.root_ingress,
                &shallow.static_callable_specializations,
            )
            .expect_err("AC-4/C3: an implicit caller-environment tail must be REFUSED");
        assert!(
            format!("{err:?}").contains("implicit caller-environment tail"),
            "AC-4/C3: the tail was refused, but not by the tail law -- a control \
             that reddens does not confirm WHICH detector caught it. Got: {err:?}"
        );
    }

    /// `AC-5` / `C4` — cross-module linking is a **checked** exclusion, paired
    /// with a positive intra-module control so the exclusion is distinguishable
    /// from a gap.
    ///
    /// Promise class: **durable mutation proof** plus a positive control.
    #[test]
    fn b2r_ac5_an_imported_capture_edge_is_refused_and_intra_module_recursion_is_not() {
        // The exclusion. A lexical closure's captures are arbitrary source
        // expressions, so this is a real plan in which an imported value would
        // have to cross into a frame and be given a carrier.
        let imported = b2r_lexical_closure(
            vec![RuntimeExpr::ImportedDeclarationRef {
                symbol: "decl:other::thing".to_string(),
                dependency: "other".to_string(),
                dependency_semantic_hash: "hash".to_string(),
            }],
            RuntimeExpr::Var(0),
        );
        let declarations = BTreeMap::new();
        let err = match plan_static_transition_graph(&imported, &declarations) {
            Ok(_) => panic!(
                "AC-5/C4: an imported capture edge must be REFUSED before emission, \
                 and it planned green instead"
            ),
            Err(err) => err,
        };
        assert!(
            matches!(err, CraneliftBackendError::Unsupported(ref u) if u.construct == "ImportedDeclarationRef"),
            "AC-5/C4: the refusal must be the EXISTING dependency-linking \
             unsupported result, not a generic planner error. Got: {err:?}"
        );

        // ⚠ The positive control. Without it, the assertion above is
        // indistinguishable from a planner that refuses closures generally.
        let intra = b2r_lexical_closure(
            vec![RuntimeExpr::DeclarationRef {
                symbol: "decl:fixture::b2o".to_string(),
            }],
            RuntimeExpr::Var(0),
        );
        let plan = plan_static_transition_graph(&intra, &declarations)
            .expect("AC-5/C4: an INTRA-module declaration capture must plan green");
        assert!(
            plan.abi.descriptors.len() > 1,
            "AC-5/C4: the positive control produced no boundary, so it does not \
             discriminate"
        );
    }

    /// `AC-6` — **inert.** The ABI plane declares and validates; it never emits.
    ///
    /// ⚠ MEASURED: the production region of `abi.rs` contains no emission
    /// construct. CLAIMED: exactly that. THE GAP: a source census cannot see an
    /// executable edge, and inertness is pinned BEHAVIOURALLY by
    /// `correspondence_adds_no_emitted_unit_to_the_production_census`. This is a
    /// declaration inventory that makes a new emission construct loud.
    ///
    /// Promise class: **durable invariant.**
    #[test]
    fn b2r_ac6_the_abi_plane_declares_no_emission_construct() {
        let abi = include_str!("static_transition/abi.rs");
        let production = abi
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(abi, |(before, _)| before);

        // ⚠ POSITIVE CONTROL FIRST. Every assertion below is a NEGATIVE check,
        // and a negative check passes for any reason -- including a broken
        // comment-stripper that returns 0 for everything. So prove the
        // instrument can SEE before trusting what it does not see.
        assert!(
            b2r_code_identifier_occurrences(production, "AbiCarrier") > 0,
            "AC-6: the instrument reports zero occurrences of a token that is \
             certainly present in the production region, so its zeros below mean \
             nothing"
        );
        // And prove it reads CODE rather than comments: `FunctionBuilder` appears
        // in this module's doc comments (denying that it emits one), so a
        // stripper that failed to strip would report a non-zero count for it and
        // the real assertion below would redden for the wrong reason.
        assert!(
            abi.contains("FunctionBuilder"),
            "AC-6: the module no longer MENTIONS the construct it disclaims, so \
             the comment-stripping half of this instrument is untested"
        );

        // Comment-stripped and tokenized, so the doc comments that DENY emitting
        // (and must keep saying so) do not fire the oracle that checks it.
        for forbidden in [
            "FunctionBuilder",
            "define_function",
            "declare_function",
            "ins",
            "Signature",
        ] {
            assert_eq!(
                b2r_code_identifier_occurrences(production, forbidden),
                0,
                "AC-6: `{forbidden}` appears in the ABI plane's production code. \
                 This node is INERT: no new callable target unit, call edge, \
                 dispatch edge, callback, flag, alternate entry, encoder or \
                 decoder lands here -- `RT-FNSPLIT-B2F` performs the atomic \
                 switch-over."
            );
        }
    }

    /// `AC-7` — no oracle, no dependency. The ABI plane parses no source text.
    ///
    /// Promise class: **durable invariant.**
    #[test]
    fn b2r_ac7_the_abi_plane_adds_no_parser_and_no_dependency_edge() {
        let abi = include_str!("static_transition/abi.rs");
        let production = abi
            .split_once("\n#[cfg(test)]\nmod tests {")
            .map_or(abi, |(before, _)| before);

        // ⚠ POSITIVE CONTROL. Same reasoning as `AC-6`: without it, a broken
        // instrument reports a clean bill of health it never measured.
        assert!(
            b2r_code_identifier_occurrences(production, "AbiPlane") > 0,
            "AC-7: the instrument reports zero occurrences of a token that is \
             certainly present, so its zeros below mean nothing"
        );

        for forbidden in ["syn", "proc_macro2", "quote", "include_str"] {
            assert_eq!(
                b2r_code_identifier_occurrences(production, forbidden),
                0,
                "AC-7: `{forbidden}` appears in the ABI plane. The population is \
                 the owner partition consumed as DATA; a source-parsing oracle \
                 is exactly the mechanism `B2O` spent four candidate SHAs ruling \
                 out."
            );
        }
    }

    /// The single capture slot of the fixture's one closure unit.
    fn b2r_only_capture_slot(plan: &StaticTransitionPlan<'_>) -> AbiSlot {
        let mut found = Vec::new();
        for descriptor in &plan.abi.descriptors {
            let start = descriptor.slots.start as usize;
            let end = start + descriptor.slots.len as usize;
            for slot in &plan.abi.slots[start..end] {
                if slot.kind == AbiSlotKind::Capture {
                    found.push(*slot);
                }
            }
        }
        assert_eq!(
            found.len(),
            1,
            "the fixture must declare exactly one capture slot, or this helper \
             is silently picking one of several"
        );
        found[0]
    }

    /// Whole-token occurrences of `needle` in `source`'s **code**, with line and
    /// block comments stripped.
    ///
    /// ⛔ Tokenized rather than substring-matched: `line.contains("ins")` is a
    /// claim about formatting and fires on `instruction`, `against`, and every
    /// other word containing those letters.
    fn b2r_code_identifier_occurrences(source: &str, needle: &str) -> usize {
        let mut code = String::with_capacity(source.len());
        let mut rest = source;
        let mut depth = 0usize;
        while !rest.is_empty() {
            if depth > 0 {
                if let Some(open) = rest.find("/*") {
                    if rest.find("*/").is_none_or(|close| open < close) {
                        depth += 1;
                        rest = &rest[open + 2..];
                        continue;
                    }
                }
                match rest.find("*/") {
                    Some(close) => {
                        depth -= 1;
                        rest = &rest[close + 2..];
                    }
                    None => break,
                }
                continue;
            }
            let block = rest.find("/*");
            let line = rest.find("//");
            match (block, line) {
                (Some(b), None) => {
                    code.push_str(&rest[..b]);
                    code.push(' ');
                    depth = 1;
                    rest = &rest[b + 2..];
                }
                (Some(b), l) if l.is_none_or(|l| b < l) => {
                    code.push_str(&rest[..b]);
                    code.push(' ');
                    depth = 1;
                    rest = &rest[b + 2..];
                }
                (_, Some(l)) => {
                    code.push_str(&rest[..l]);
                    code.push(' ');
                    rest = match rest[l..].find('\n') {
                        Some(nl) => &rest[l + nl..],
                        None => "",
                    };
                }
                (None, None) => {
                    code.push_str(rest);
                    rest = "";
                }
            }
        }
        code.split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|token| *token == needle)
            .count()
    }

    /// `AC-10` — the **predicted** descriptor population, measured.
    ///
    /// ⭐ The numbers below were written into
    /// `docs/program/rt-fnsplit-b2r-predictions.md` (`P1`) and committed at
    /// `b7aacd03`, **before** `abi.rs` existed. A count re-fit to what the code
    /// happens to produce measures nothing; the commit graph orders the
    /// prediction ahead of the measurement so a miss stays legible as a miss.
    ///
    /// Promise class: **durable invariant** — the assertion is the RELATION
    /// `descriptors == entries + StaticBody edges`, which survives any change
    /// preserving the contract. The per-fixture table beside it is a
    /// **transition sentinel**: it is a snapshot of these seven fixtures and is
    /// retired when the fixture set changes.
    #[test]
    fn b2r_ac10_the_descriptor_population_matches_the_prediction_on_every_fixture() {
        let mut measured = Vec::new();
        for (name, expr) in b2ac_topology_fixtures() {
            let declarations = BTreeMap::new();
            let plan = plan_static_transition_graph(&expr, &declarations).expect("plannable");

            // The durable relation, asserted per fixture.
            let static_body = plan
                .edges
                .iter()
                .filter(|edge| edge.kind == EdgeKind::StaticBody)
                .count();
            assert_eq!(
                plan.abi.descriptors.len(),
                plan.entries.len() + static_body,
                "AC-10/AC-1: `{name}` -- descriptors are not the scheduling \
                 entries plus the static body targets"
            );
            measured.push((name, plan.abi.descriptors.len()));
        }

        assert_eq!(
            measured,
            vec![
                ("leaf", 1),
                ("let-if", 1),
                ("match", 1),
                ("lexical-closure-call", 2),
                ("computational", 1),
                ("computational-nested", 1),
                ("computational-under-let", 1),
            ],
            "AC-10: the measured descriptor population differs from the value \
             predicted at `b7aacd03` before this module was written"
        );
        assert_eq!(
            measured.iter().map(|(_, n)| n).sum::<usize>(),
            8,
            "AC-10: predicted 8 descriptors over the seven fixtures"
        );

        // And the richer B2O fixture: 2 scheduling entries (root + the
        // transparent declaration) and 2 static body edges.
        let declaration = b2o_transparent_declaration(unit());
        let mut declarations = BTreeMap::new();
        declarations.insert("decl:fixture::b2o", &declaration);
        let expr = b2o_two_closure_fixture();
        let plan = plan_static_transition_graph(&expr, &declarations).expect("plannable");
        assert_eq!(
            plan.abi.descriptors.len(),
            4,
            "AC-10: predicted 4 descriptors on the two-closure fixture"
        );
    }

    /// `AC-11` — **every rejection class `D5` advertises has a witness that
    /// reaches THAT arm.**
    ///
    /// ⛔ This is not `AC-4`'s positive control and `AC-4` does not cover it.
    /// `AC-4` proves *the checker was reached*; `AC-11` proves *which arm
    /// rejected*. In the failure mode the input **is** constructed, the validator
    /// **does** reject, the test **is** green — and an **earlier** arm returned
    /// the error while the arm you meant to exercise is unreachable code.
    ///
    /// ⭐ Asserting the **exact** message rather than `is_err`/`expect_err` is
    /// the entire mechanism. With `expect_err` every row below reads green and
    /// teaches nothing.
    ///
    /// Promise class: **durable mutation proof.** Each row names the arm that
    /// actually fired, so a re-ordering of the validator reddens here rather than
    /// silently changing which law is load-bearing.
    #[test]
    fn b2r_ac11_every_advertised_d5_rejection_class_names_the_arm_that_actually_fires() {
        let expr = b2r_seed_closure(&["c"], RuntimeExpr::Var(0));
        let plan = b2r_plan(&expr);
        let base = &plan.abi;

        let check = |abi: &super::abi::AbiPlane| -> String {
            match abi.validate(
                &plan.semantic,
                &plan.nodes,
                &plan.semantic_sources,
                &plan.edges,
                &plan.entries,
                plan.root_entry.expect("root entry"),
                plan.root_ingress,
                &plan.static_callable_specializations,
            ) {
                Ok(()) => "NO WITNESS -- the mutation was accepted".to_string(),
                Err(err) => format!("{err:?}"),
            }
        };

        let closure_unit = base
            .descriptors
            .iter()
            .position(|d| d.header.captures == 1)
            .expect("the fixture must have a unit with exactly one capture");

        let mut measured = Vec::new();

        // D5 class 1 -- a MISSING capture slot.
        let mut missing = base.clone();
        missing.descriptors[closure_unit].header.captures = 0;
        measured.push(("missing capture slot", check(&missing)));

        // D5 class 2 -- an EXTRA capture slot.
        let mut extra = base.clone();
        extra.descriptors[closure_unit].header.captures = 2;
        measured.push(("extra capture slot", check(&extra)));

        // D5 class 3 -- an implicit caller-environment TAIL.
        let mut tailed = base.clone();
        let tail = *tailed.slots.last().expect("slots");
        tailed
            .descriptors
            .last_mut()
            .expect("descriptors")
            .slots
            .len += 1;
        tailed.slots.push(tail);
        measured.push(("implicit caller-env tail", check(&tailed)));

        // D5 class 4 -- caller/callee dynamic-edge LAYOUT DISAGREEMENT.
        //
        // ⛔ An earlier revision mutated `planned_node` here, which tests TARGET
        // IDENTITY while naming layout agreement -- the Architect's finding. A
        // real witness must leave identity intact and make the CALLER-side
        // transfer layout disagree with the callee's declared frame.
        //
        // This grows the defining occurrence's capture-child count in the graph
        // while leaving its recorded `capture_slots` alone. The per-descriptor
        // checks compare against `capture_slots` and so still pass; only the
        // boundary comparison, which counts capture children caller-side, can
        // see the divergence. That is exactly the independence the signature
        // claims.
        let lexical_expr = b2r_lexical_closure(vec![unit()], RuntimeExpr::Var(0));
        let lexical_plan = b2r_plan(&lexical_expr);
        let mut skewed_plane = lexical_plan.semantic.clone();
        let defining = lexical_plan
            .edges
            .iter()
            .find(|edge| edge.kind == EdgeKind::StaticBody)
            .map(|edge| edge.from.0 as usize)
            .expect("the lexical fixture has a static body boundary");
        let program = skewed_plane.descriptors[defining].program.0 as usize;
        let record = skewed_plane.programs[program].records.start as usize;
        let extra = skewed_plane.records[record].child_origins;
        let borrowed = skewed_plane.child_origins[extra.start as usize];
        skewed_plane
            .child_origins
            .insert((extra.start + extra.len) as usize, borrowed);
        skewed_plane.records[record].child_origins.len += 1;
        let layout_arm = match lexical_plan.abi.validate(
            &skewed_plane,
            &lexical_plan.nodes,
            &lexical_plan.semantic_sources,
            &lexical_plan.edges,
            &lexical_plan.entries,
            lexical_plan.root_entry.expect("root entry"),
            lexical_plan.root_ingress,
            &lexical_plan.static_callable_specializations,
        ) {
            Ok(()) => "NO WITNESS -- the layout skew was accepted".to_string(),
            Err(err) => format!("{err:?}"),
        };
        measured.push(("edge layout disagreement", layout_arm));

        // D5 class 5 -- a recursive-bundle member that is NOT forward-declared.
        let mut unforward = base.clone();
        unforward.descriptors.truncate(closure_unit);
        measured.push(("callee not forward-declared", check(&unforward)));

        // D5 class 6 -- representability / the imported-edge exclusion. This one
        // is a GRAPH witness, not a plane mutation: it is checked during
        // construction, before any descriptor is minted.
        let imported = b2r_lexical_closure(
            vec![RuntimeExpr::ImportedDeclarationRef {
                symbol: "decl:other::thing".to_string(),
                dependency: "other".to_string(),
                dependency_semantic_hash: "hash".to_string(),
            }],
            RuntimeExpr::Var(0),
        );
        let declarations = BTreeMap::new();
        let imported_arm = match plan_static_transition_graph(&imported, &declarations) {
            Ok(_) => "NO WITNESS -- the imported capture edge planned green".to_string(),
            Err(err) => format!("{err:?}"),
        };
        measured.push(("imported capture edge", imported_arm));

        let report = measured
            .iter()
            .map(|(class, arm)| format!("{class} => {arm}"))
            .collect::<Vec<_>>();
        // ⭐ MEASURED, not predicted-then-fitted. The arm named on each row is
        // the one that actually returned. **Five of the six classes reach an arm
        // of this validator's own; exactly ONE is enforced by an earlier arm**
        // and is recorded as such rather than counted as a law of its own.
        //
        // ⚠ **Row 5 — recursive-bundle forward declaration — is the subsumed
        // one.** Descriptors are dense and complete over the partition before any
        // edge resolves, which *is* forward-declaration, so the dense population
        // check sees a gap first and the class never reaches an arm of its own.
        // It is reported as subsumed, not counted.
        //
        // ⭐ **Row 4 — edge-layout disagreement — reaches its OWN arm**,
        // `"boundary signature ... transferred capture count"`, supplied by
        // `AbiBoundarySignature` / `validate_boundary_layouts` in `abi.rs`.
        //
        // ⛔ **This paragraph previously said rows 4 AND 5 were subsumed and that
        // the edge-agreement code had been deleted. That was true of an earlier
        // revision and false of this one**, and the assertion directly below
        // proved it false while the comment still said it. The deletion was
        // reverted once the Architect established that the composition I cited
        // proves target IDENTITY and never layout AGREEMENT; a real caller-side
        // boundary check now exists. `B2F` is told to read this validator as its
        // guarantee, so a stale count of live laws here is exactly the
        // silently-inherited defect `AC-11` exists to prevent -- which makes a
        // stale governing comment the same defect one layer up from the code.
        assert_eq!(
            report,
            vec![
                // -- reach an arm of their own --
                "missing capture slot => Backend(PlannerInvariant(\"abi descriptor \
                 is missing a declared capture slot\"))"
                    .to_string(),
                "extra capture slot => Backend(PlannerInvariant(\"abi descriptor \
                 declares a capture slot its origin does not have\"))"
                    .to_string(),
                "implicit caller-env tail => Backend(PlannerInvariant(\"abi frame \
                 carries an implicit caller-environment tail\"))"
                    .to_string(),
                // -- reaches the boundary-layout arm --
                "edge layout disagreement => Backend(PlannerInvariant(\"boundary \
                 signature and callee descriptor disagree on the transferred \
                 capture count\"))"
                    .to_string(),
                // -- SUBSUMED: descriptors are dense over the partition before
                //    any edge resolves, which IS forward-declaration --
                "callee not forward-declared => Backend(PlannerInvariant(\"abi \
                 descriptor population is not exact for graph units plus static \
                 callable specializations\"))"
                    .to_string(),
                // -- reaches its own arm, with the EXISTING unsupported result --
                "imported capture edge => Unsupported(UnsupportedLowering { \
                 construct: \"ImportedDeclarationRef\", reason: \"imported \
                 declaration requires dependency linking, so it receives no \
                 callable descriptor in the intra-module representation \
                 contract\" })"
                    .to_string(),
            ],
            "AC-11: the arm that actually fired differs from the one recorded \
             for this class. Either the validator was re-ordered -- in which \
             case which law is load-bearing has changed and that is the point of \
             this test -- or a previously subsumed arm became reachable."
        );
    }

    /// `AC-3` positive control, as the frame words it — **a seed capture whose
    /// ground value is a `Constructor`, a `Record`, or a `String` must still
    /// yield one FIXED carrier, and the descriptor must not vary with the
    /// value.**
    ///
    /// ⛔ An earlier revision discharged this by renaming a capture symbol. That
    /// is not the discriminator the frame asks for: it never constructs a value
    /// from the family, so it cannot observe representability across it. The
    /// Architect's finding, and this is the repair.
    ///
    /// ⭐ **Two mechanisms, and they answer different questions.**
    ///
    /// 1. The **closed-family map** below is exhaustive over `RuntimeGroundValue`
    ///    with no `_ =>` arm, so a seventh variant is a **compile error** here
    ///    rather than a value that silently acquires a carrier. That is the
    ///    representability half.
    /// 2. Planning the same closure against three seed environments — each
    ///    binding the capture to a different variant of the family — must give
    ///    **byte-identical descriptors**. That is the invariance half.
    ///
    /// ⚠ **Why the second is stronger than it looks, stated honestly.** The
    /// descriptors are identical because the planner never receives a seed
    /// environment at all: `build_abi_plane`'s inputs contain no
    /// `RuntimeGroundValue`. So this control does not *discover* invariance — it
    /// **exhibits** that the family is real, constructible, and inert to the
    /// contract. The enforcement remains the signature. Recorded this way rather
    /// than presented as a measurement that could have come out otherwise.
    ///
    /// Promise class: **durable invariant.**
    #[test]
    fn b2r_ac3_the_closed_ground_value_family_yields_one_fixed_carrier() {
        // (1) Representability across the closed family. No `_ =>` arm.
        fn carrier_for(value: &RuntimeGroundValue) -> AbiCarrier {
            match value {
                RuntimeGroundValue::Bool(_)
                | RuntimeGroundValue::Int(_)
                | RuntimeGroundValue::Bytes(_)
                | RuntimeGroundValue::String(_)
                | RuntimeGroundValue::Constructor { .. }
                | RuntimeGroundValue::Record { .. } => AbiCarrier::GroundValueCarrier,
            }
        }

        let family = vec![
            ("String", RuntimeGroundValue::String("seeded".to_string())),
            (
                "Constructor",
                RuntimeGroundValue::Constructor {
                    constructor: "ctor:prelude::Unit::MkUnit".to_string(),
                    args: Vec::new(),
                },
            ),
            (
                "Record",
                RuntimeGroundValue::Record {
                    fields: vec![("f".to_string(), RuntimeGroundValue::Bool(true))],
                },
            ),
        ];

        // Every member of the family maps to the ONE carrier.
        for (label, value) in &family {
            assert_eq!(
                carrier_for(value),
                AbiCarrier::GroundValueCarrier,
                "AC-3/C2: a seed ground value of kind {label} did not land on the \
                 single fixed carrier"
            );
        }

        // (2) Invariance: the same closure, seeded with each member in turn.
        let expr = b2r_seed_closure(&["c"], RuntimeExpr::Var(0));
        let mut shapes = Vec::new();
        for (label, value) in &family {
            let mut seed_env = NativeSeedEnvironment::default();
            seed_env.insert("c", value.clone());
            // ⚠ The environment is constructed and bound, and is deliberately
            // NOT threaded into planning -- because planning has no parameter to
            // thread it into. That absence IS the contract.
            assert!(
                seed_env.values.contains_key("c"),
                "AC-3: the {label} seed binding did not materialise, so the \
                 invariance rows below would compare three empty environments"
            );
            let plan = b2r_plan(&expr);
            shapes.push(plan.abi.shapes().expect("shapes"));
        }

        assert_eq!(shapes.len(), 3, "AC-3: the family must have three members");
        assert_eq!(
            shapes[0], shapes[1],
            "AC-3/C3: the descriptor differed between a String and a Constructor \
             seed capture"
        );
        assert_eq!(
            shapes[1], shapes[2],
            "AC-3/C3: the descriptor differed between a Constructor and a Record \
             seed capture"
        );

        // Non-vacuity: the shapes being compared must actually contain a seed
        // capture slot carrying the fixed carrier, or all three are equal
        // because all three are empty.
        let seeded = b2r_plan(&expr);
        assert_eq!(
            b2r_only_capture_slot(&seeded).carrier,
            AbiCarrier::GroundValueCarrier,
            "AC-3: the fixture declares no seed capture slot, so the invariance \
             rows above compare descriptors that never exercised the carrier"
        );
    }

    /// Promise class: durable invariant — process mode changes only the
    /// explicitly recorded root scheduling entry's declared source ingress.
    #[test]
    fn process_ingress_is_role_keyed_and_absent_from_value_roots() {
        let expr = RuntimeExpr::Value(RuntimeValue::Bool(true));
        let symbols = crate::NativeProcessSymbols::legacy_prelude();
        let transparent = RuntimeDeclaration {
            symbol: "decl:fixture::process_ingress::transparent".to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::Value(RuntimeValue::Bool(false)),
            },
            metadata: crate::RuntimeSymbolMetadata {
                lowerability: Some(crate::RuntimeLowerabilityStatus::Supported),
                ..crate::RuntimeSymbolMetadata::empty()
            },
        };
        let declarations = BTreeMap::from([(transparent.symbol.as_str(), &transparent)]);
        let process = plan_static_transition_graph_with_symbols(
            &expr,
            &declarations,
            &symbols,
            AbiRootIngress::Process,
            true,
        )
        .expect("process root plans");
        let input = process
            .process_parameter_slot(AbiProcessParameter::ProcessInput)
            .expect("lookup validates")
            .expect("process input slot exists");
        let capability = process
            .process_parameter_slot(AbiProcessParameter::Capability)
            .expect("lookup validates")
            .expect("capability slot exists");
        assert_eq!(input.0.kind, AbiSlotKind::Parameter);
        assert_eq!(input.0.ordinal, 0);
        assert_eq!(capability.0.kind, AbiSlotKind::Parameter);
        assert_eq!(capability.0.ordinal, 1);
        assert_ne!(input.1, capability.1);
        let scheduling = process
            .emittable_units()
            .expect("validated units")
            .into_iter()
            .filter_map(|unit| match unit.definition() {
                AbiUnitDefinition::SchedulingEntry { ingress } => {
                    Some((ingress, unit.header().parameters))
                }
                AbiUnitDefinition::TransparentDeclarationClosure { .. }
                | AbiUnitDefinition::ClosureBody { .. }
                | AbiUnitDefinition::StaticCallableSpecialization { .. } => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            scheduling,
            vec![
                (AbiSchedulingIngress::ProcessPair, 2),
                (AbiSchedulingIngress::Empty, 0),
            ],
            "only the explicitly recorded process root acquires parameters"
        );

        let value = plan_static_transition_graph_with_symbols(
            &expr,
            &BTreeMap::new(),
            &symbols,
            AbiRootIngress::Value,
            true,
        )
        .expect("value root plans");
        assert_eq!(
            value
                .process_parameter_slot(AbiProcessParameter::ProcessInput)
                .expect("lookup validates"),
            None
        );
        assert_eq!(
            value
                .process_parameter_slot(AbiProcessParameter::Capability)
                .expect("lookup validates"),
            None
        );

        let closure = |captures| RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures,
                params: Vec::new(),
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            }),
            args: Vec::new(),
        };
        let captured_expr = closure(vec![RuntimeExpr::Var(0), RuntimeExpr::Var(1)]);
        let captured = plan_static_transition_graph_with_symbols(
            &captured_expr,
            &BTreeMap::new(),
            &symbols,
            AbiRootIngress::Process,
            true,
        )
        .expect("capturing process closure plans");
        let capture_counts = captured
            .emittable_units()
            .expect("validated units")
            .into_iter()
            .filter_map(|unit| match unit.definition() {
                AbiUnitDefinition::ClosureBody { .. } => Some(unit.header().captures),
                AbiUnitDefinition::TransparentDeclarationClosure { .. }
                | AbiUnitDefinition::SchedulingEntry { .. }
                | AbiUnitDefinition::StaticCallableSpecialization { .. } => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(capture_counts, vec![2]);

        let uncaptured_expr = closure(Vec::new());
        let uncaptured = plan_static_transition_graph_with_symbols(
            &uncaptured_expr,
            &BTreeMap::new(),
            &symbols,
            AbiRootIngress::Process,
            true,
        )
        .expect("non-capturing process closure plans");
        let capture_counts = uncaptured
            .emittable_units()
            .expect("validated units")
            .into_iter()
            .filter_map(|unit| match unit.definition() {
                AbiUnitDefinition::ClosureBody { .. } => Some(unit.header().captures),
                AbiUnitDefinition::TransparentDeclarationClosure { .. }
                | AbiUnitDefinition::SchedulingEntry { .. }
                | AbiUnitDefinition::StaticCallableSpecialization { .. } => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            capture_counts,
            vec![0],
            "an otherwise identical body without a free binding acquired a slot"
        );
    }

    fn d8_mixed_join(swapped: bool) -> RuntimeExpr {
        let carried = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Carried".to_string(),
            binders: 0,
            body: RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: Vec::new(),
                    body: Box::new(RuntimeExpr::Value(RuntimeValue::Int(11.into()))),
                }),
                args: Vec::new(),
            },
        };
        let specialized = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Specialized".to_string(),
            binders: 0,
            body: RuntimeExpr::Value(RuntimeValue::Int(7.into())),
        };
        RuntimeExpr::Match {
            // Deliberately specialized: the scrutinee phase is not the result
            // representation selector.
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Constructor {
                constructor: "ctor:fixture::D8::Carried".to_string(),
                args: Vec::new(),
            })),
            cases: if swapped {
                vec![specialized, carried]
            } else {
                vec![carried, specialized]
            },
            default: trap("D8 mixed join default"),
        }
    }

    fn d8_functionized_plan(
        expr: &RuntimeExpr,
    ) -> Result<StaticTransitionPlan<'_>, CraneliftBackendError> {
        plan_static_transition_graph_with_symbols(
            expr,
            &BTreeMap::new(),
            &crate::NativeProcessSymbols::legacy_prelude(),
            AbiRootIngress::Value,
            true,
        )
    }

    fn d8_environment_join(swapped: bool) -> RuntimeExpr {
        let carried = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Carried".to_string(),
            binders: 0,
            body: RuntimeExpr::Var(0),
        };
        let specialized = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Specialized".to_string(),
            binders: 0,
            body: RuntimeExpr::Value(RuntimeValue::Int(7.into())),
        };
        RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: Vec::new(),
                    body: Box::new(RuntimeExpr::Value(RuntimeValue::Int(11.into()))),
                }),
                args: Vec::new(),
            }),
            body: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Constructor {
                    constructor: "ctor:fixture::D8::Carried".to_string(),
                    args: Vec::new(),
                })),
                cases: if swapped {
                    vec![specialized, carried]
                } else {
                    vec![carried, specialized]
                },
                default: trap("D8 environment join default"),
            }),
        }
    }

    fn assert_d8_environment_join_is_carrier(swapped: bool) {
        let expr = d8_environment_join(swapped);
        let plan = d8_functionized_plan(&expr).expect("environment join plans");
        let root = plan.root_static_origin().expect("root origin");
        let join = plan
            .semantic
            .child_origin(root, 1)
            .expect("let body origin");
        let token = plan
            .join_plan_token(join)
            .expect("nested environment join has one plan entry");
        assert_eq!(
            token.representation,
            JoinResultRepresentation::CarrierWord,
            "the exact nested join lost its let-bound declared-unit result"
        );
    }

    fn d8_bound_callable_join(swapped: bool) -> RuntimeExpr {
        let carried_call = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Call".to_string(),
            binders: 0,
            body: RuntimeExpr::Call {
                callee: Box::new(RuntimeExpr::Var(0)),
                args: Vec::new(),
            },
        };
        let specialized = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Specialized".to_string(),
            binders: 0,
            body: RuntimeExpr::Value(RuntimeValue::Int(7.into())),
        };
        RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: Vec::new(),
                body: Box::new(RuntimeExpr::Value(RuntimeValue::Int(11.into()))),
            }),
            body: Box::new(RuntimeExpr::Match {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Constructor {
                    constructor: "ctor:fixture::D8::Call".to_string(),
                    args: Vec::new(),
                })),
                cases: if swapped {
                    vec![specialized, carried_call]
                } else {
                    vec![carried_call, specialized]
                },
                default: trap("D8 bound callable join default"),
            }),
        }
    }

    fn assert_d8_bound_callable_join_is_carrier(swapped: bool) {
        let expr = d8_bound_callable_join(swapped);
        let plan = d8_functionized_plan(&expr).expect("bound callable join plans");
        let root = plan.root_static_origin().expect("root origin");
        let join = plan
            .semantic
            .child_origin(root, 1)
            .expect("let body origin");
        let token = plan
            .join_plan_token(join)
            .expect("nested bound-callable join has one plan entry");
        assert_eq!(
            token.representation,
            JoinResultRepresentation::CarrierWord,
            "the exact nested join lost the bound closure's callable result"
        );
    }

    fn d8_abi_parameter_join(swapped: bool) -> RuntimeExpr {
        let carried = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Carried".to_string(),
            binders: 0,
            body: RuntimeExpr::Var(0),
        };
        let specialized = RuntimeMatchCase {
            constructor: "ctor:fixture::D8::Specialized".to_string(),
            binders: 0,
            body: RuntimeExpr::Value(RuntimeValue::Int(7.into())),
        };
        RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::LexicalClosure {
                captures: Vec::new(),
                params: vec!["carried".to_string()],
                body: Box::new(RuntimeExpr::Match {
                    scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Constructor {
                        constructor: "ctor:fixture::D8::Carried".to_string(),
                        args: Vec::new(),
                    })),
                    cases: if swapped {
                        vec![specialized, carried]
                    } else {
                        vec![carried, specialized]
                    },
                    default: trap("D8 ABI parameter join default"),
                }),
            }),
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(11.into()))],
        }
    }

    fn d8_abi_parameter_join_origin(
        plan: &StaticTransitionPlan<'_>,
    ) -> Result<StaticOriginId, CraneliftBackendError> {
        let root = plan.root_static_origin()?;
        let callee = plan.semantic.child_origin(root, 0)?;
        plan.semantic.child_origin(callee, 0)
    }

    #[test]
    fn d8_mixed_join_plan_is_carrier_and_arm_order_independent() {
        for swapped in [false, true] {
            let expr = d8_mixed_join(swapped);
            let plan = d8_functionized_plan(&expr).expect("mixed join plans");
            let token = plan
                .join_plan_token(plan.root_static_origin().expect("root origin"))
                .expect("root join has one plan entry");
            assert_eq!(
                token.representation,
                JoinResultRepresentation::CarrierWord,
                "specialized scrutinee or first-arm order selected a native merge"
            );
            assert!(
                token.has_continuing_predecessor,
                "mixed join lost both continuing predecessors"
            );
        }
    }

    /// MEASURED: the exact nested source join receives `CarrierWord` when one
    /// arm forwards a declared-unit result through a de Bruijn environment
    /// insertion, independently of arm order.
    ///
    /// CLAIMED: D8 phase propagation is monotone through `Let`; a join cannot
    /// be planned as native merely because its carrier reaches it through a
    /// variable.
    ///
    /// GAP: this fixture does not exercise ABI parameters; the adjacent control
    /// pins that independent environment seed.
    #[test]
    fn d8_let_environment_provenance_reaches_the_exact_nested_join() {
        for swapped in [false, true] {
            assert_d8_environment_join_is_carrier(swapped);
        }
    }

    /// MEASURED: a lexical closure bound by `Let`, recovered as `Var(0)`, and
    /// invoked in one mixed-join arm makes that exact join `CarrierWord`
    /// independently of arm order.
    ///
    /// CLAIMED: D8's binder environment retains both a bound value's own phase
    /// and the phase produced when that value is invoked.
    ///
    /// GAP: this is the lexical-binder route. ABI parameters remain governed by
    /// the adjacent closed-slot control.
    #[test]
    fn d8_bound_lexical_callable_provenance_reaches_the_exact_nested_join() {
        for swapped in [false, true] {
            assert_d8_bound_callable_join_is_carrier(swapped);
        }
    }

    /// MEASURED: a functionized unit parameter is seeded from the unit ABI and
    /// reaches the exact join occurrence in the unit body.
    ///
    /// CLAIMED: phase planning uses the closed ABI slot inventory for
    /// parameter/capture environment provenance, rather than classifying every
    /// `Var` as specialized.
    ///
    /// GAP: captures share the same ABI-slot seed path and are not duplicated
    /// in this control.
    #[test]
    fn d8_abi_parameter_provenance_reaches_the_exact_nested_join() {
        for swapped in [false, true] {
            let expr = d8_abi_parameter_join(swapped);
            let plan = d8_functionized_plan(&expr).expect("ABI parameter join plans");
            let join =
                d8_abi_parameter_join_origin(&plan).expect("closure body join has an origin");
            let token = plan
                .join_plan_token(join)
                .expect("nested parameter join has one plan entry");
            assert_eq!(
                token.representation,
                JoinResultRepresentation::CarrierWord,
                "the exact nested join lost its function-unit parameter"
            );
        }
    }

    /// MEASURED: the same closed ABI-parameter fixture is `CarrierWord` under
    /// FunctionizedUnits and `NativeScalarPair` under RecursiveDescent,
    /// independently of arm order.
    ///
    /// CLAIMED: the validated but inert ABI plane cannot impose carrier
    /// storage on the retained RecursiveDescent lowering authority.
    ///
    /// GAP: this pins the planner boundary and the native/interpreter parity
    /// suite pins the resulting public observations; it does not compare every
    /// emitted block in the two authorities.
    #[test]
    fn d8_inert_abi_slots_do_not_change_recursive_descent_join_storage() {
        for swapped in [false, true] {
            let expr = d8_abi_parameter_join(swapped);
            let functionized =
                d8_functionized_plan(&expr).expect("functionized ABI parameter join plans");
            let retained = plan_static_transition_graph_with_symbols(
                &expr,
                &BTreeMap::new(),
                &crate::NativeProcessSymbols::legacy_prelude(),
                AbiRootIngress::Value,
                false,
            )
            .expect("retained ABI parameter join plans");
            for (plan, expected) in [
                (&functionized, JoinResultRepresentation::CarrierWord),
                (&retained, JoinResultRepresentation::NativeScalarPair),
            ] {
                let join =
                    d8_abi_parameter_join_origin(plan).expect("closure body join has an origin");
                assert_eq!(
                    plan.join_plan_token(join)
                        .expect("nested parameter join has one plan entry")
                        .representation,
                    expected,
                    "inert and live ABI slots selected the same join storage"
                );
            }
        }
    }

    /// Reversible mutation: forcing all variable seeds back to the rejected
    /// `SpecializedOnly` behavior must red at the plan assertion, before any
    /// lowering or emitted block can influence the result.
    #[test]
    fn d8_variable_seed_mutation_reds_at_the_plan_boundary() {
        D8_FORCE_VARIABLE_SPECIALIZED.with(|forced| forced.set(true));
        let result = std::panic::catch_unwind(|| {
            assert_d8_environment_join_is_carrier(false);
        });
        D8_FORCE_VARIABLE_SPECIALIZED.with(|forced| forced.set(false));
        assert!(
            result.is_err(),
            "forcing the rejected variable seed did not red the plan assertion"
        );
    }

    /// Reversible population-side mutation: removing only the callable-result
    /// seed from `Var` must red at the exact plan assertion before lowering.
    #[test]
    fn d8_callable_seed_removal_reds_at_the_plan_boundary() {
        D8_REMOVE_VARIABLE_CALLABLE_SEED.with(|forced| forced.set(true));
        let result = std::panic::catch_unwind(|| {
            assert_d8_bound_callable_join_is_carrier(false);
        });
        D8_REMOVE_VARIABLE_CALLABLE_SEED.with(|forced| forced.set(false));
        assert!(
            result.is_err(),
            "removing the bound callable seed did not red the plan assertion"
        );
    }

    fn d7_aggregate_identity_declaration(symbol: &str) -> RuntimeDeclaration {
        RuntimeDeclaration {
            symbol: symbol.to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["value".to_string()],
                    body: Box::new(RuntimeExpr::Var(0)),
                },
            },
            metadata: crate::RuntimeSymbolMetadata::empty(),
        }
    }

    fn d7_aggregate_identity_call(symbol: &str, value: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: symbol.to_string(),
            }),
            args: vec![value],
        }
    }

    fn d7_invocation_value() -> RuntimeExpr {
        RuntimeExpr::Effect {
            family: "Buffer".to_string(),
            operation: ken_host::HostOpV1::BufferAllocate,
            capability: None,
            args: vec![RuntimeExpr::Value(RuntimeValue::Int(1.into()))],
        }
    }

    fn d7_boxed_value(constructor: &str, value: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: constructor.to_string(),
            args: vec![value],
        }
    }

    fn d7_single_field_record(value: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Record {
            fields: vec![("field:fixture::value".to_string(), value)],
        }
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: two occurrences with the same constructor identity and arity,
    /// and two records with the same field identity and arity, select different
    /// representation owners solely from their exact child occurrence facts.
    /// CLAIMED: aggregate spelling and shape are not referent-lifetime
    /// authority.
    /// THE GAP: emitted semantic parity is covered by the carrier tests; this
    /// pin measures the pre-definition planner decision.
    #[test]
    fn d7_aggregate_owner_is_keyed_by_occurrence_not_nominal_shape() {
        let symbol = "decl:fixture::d7-aggregate-identity";
        let declaration = d7_aggregate_identity_declaration(symbol);
        let entry = RuntimeExpr::Construct {
            constructor: "ctor:fixture::AggregatePair".to_string(),
            args: vec![
                d7_aggregate_identity_call(
                    symbol,
                    d7_boxed_value(
                        "ctor:fixture::SameBox",
                        RuntimeExpr::Value(RuntimeValue::Bool(true)),
                    ),
                ),
                d7_aggregate_identity_call(
                    symbol,
                    d7_boxed_value("ctor:fixture::SameBox", d7_invocation_value()),
                ),
                d7_aggregate_identity_call(
                    symbol,
                    d7_single_field_record(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                ),
                d7_aggregate_identity_call(symbol, d7_single_field_record(d7_invocation_value())),
            ],
        };
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations)
            .expect("same-shape aggregate owner pair plans");

        let constructor_identity = plan
            .aggregate_representations
            .iter()
            .find_map(|record| match record.identity {
                AggregateIdentity::Constructor(identity)
                    if record.arity == 1 && record.phase == ResultPhase::CarrierRequired =>
                {
                    Some(identity)
                }
                _ => None,
            })
            .expect("fixture has one carried unary constructor identity");
        let constructor_pair = plan
            .aggregate_representations
            .iter()
            .filter(|record| {
                record.identity == AggregateIdentity::Constructor(constructor_identity)
                    && record.arity == 1
                    && record.phase == ResultPhase::CarrierRequired
            })
            .map(|record| (record.selected_owner, record.selected_tag))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            constructor_pair,
            BTreeSet::from([
                (
                    BoundaryReferentOwner::PersistentStore,
                    BoundaryTag::PersistentGround,
                ),
                (
                    BoundaryReferentOwner::InvocationArena,
                    BoundaryTag::InvocationAggregate,
                ),
            ]),
            "same constructor identity/arity collapsed to one lifetime"
        );

        let record_pair = plan
            .aggregate_representations
            .iter()
            .filter(|record| {
                record.class == BoundaryClass::Record
                    && record.arity == 1
                    && record.phase == ResultPhase::CarrierRequired
            })
            .map(|record| (record.selected_owner, record.selected_tag))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            record_pair,
            BTreeSet::from([
                (
                    BoundaryReferentOwner::PersistentStore,
                    BoundaryTag::PersistentGround,
                ),
                (
                    BoundaryReferentOwner::InvocationArena,
                    BoundaryTag::InvocationAggregate,
                ),
            ]),
            "same record identity/arity collapsed to one lifetime"
        );
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: an invocation child below mixed Constructor/Record depth two
    /// makes every ancestor invocation-owned, while an all-durable twin stays
    /// persistent; a runtime alternative conservatively selects invocation.
    /// CLAIMED: ownership is the transitive lifetime meet over every possible
    /// child, not a direct-child or first-arm walk.
    /// THE GAP: cross-invocation replay is a boundary-store property covered by
    /// the arena lifecycle controls rather than this producer-flow pin.
    #[test]
    fn d7_aggregate_owner_meet_is_transitive_and_joins_all_alternatives() {
        let symbol = "decl:fixture::d7-aggregate-transitive";
        let declaration = d7_aggregate_identity_declaration(symbol);
        let nested = |leaf| {
            d7_boxed_value(
                "ctor:fixture::Outer",
                d7_single_field_record(d7_boxed_value("ctor:fixture::Inner", leaf)),
            )
        };
        let alternative = RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            then_expr: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            else_expr: Box::new(d7_invocation_value()),
        };
        let entry = RuntimeExpr::Construct {
            constructor: "ctor:fixture::AggregateTriple".to_string(),
            args: vec![
                d7_aggregate_identity_call(
                    symbol,
                    nested(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                ),
                d7_aggregate_identity_call(symbol, nested(d7_invocation_value())),
                d7_aggregate_identity_call(
                    symbol,
                    d7_boxed_value("ctor:fixture::AlternativeBox", alternative),
                ),
            ],
        };
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations)
            .expect("transitive aggregate owner meet plans");

        let outer_records = plan
            .aggregate_representations
            .iter()
            .filter(|record| {
                matches!(record.identity, AggregateIdentity::Constructor(_))
                    && record.arity == 1
                    && record.phase == ResultPhase::CarrierRequired
            })
            .collect::<Vec<_>>();
        assert!(
            outer_records.iter().any(|record| {
                record.selected_tag == BoundaryTag::PersistentGround
                    && record.selected_owner == BoundaryReferentOwner::PersistentStore
            }),
            "all-durable transitive twin did not remain persistent"
        );
        assert!(
            outer_records
                .iter()
                .filter(|record| {
                    record.selected_tag == BoundaryTag::InvocationAggregate
                        && record.selected_owner == BoundaryReferentOwner::InvocationArena
                })
                .count()
                >= 2,
            "depth-two or durable/invocation alternative lost invocation ownership"
        );

        let exact = plan.aggregate_representations.clone();
        let invocation_index = exact
            .iter()
            .position(|record| record.selected_tag == BoundaryTag::InvocationAggregate)
            .expect("fixture has an invocation aggregate");
        let reject = |name: &str, mutate: &dyn Fn(&mut PlannedAggregateRepresentation)| {
            let mut changed = plan.clone();
            mutate(&mut changed.aggregate_representations[invocation_index]);
            assert_ne!(
                changed.aggregate_representations, exact,
                "{name} was vacuous"
            );
            assert_eq!(
                changed.validate_producer_flow_plans().unwrap_err(),
                planner_error(
                    "aggregate representation plan is not the exact producer-flow derivation"
                ),
                "{name} survived the pre-definition exact derivation"
            );
        };
        reject("wrong selected tag", &|record| {
            record.selected_tag = BoundaryTag::PersistentGround
        });
        reject("wrong owner", &|record| {
            record.selected_owner = BoundaryReferentOwner::PersistentStore
        });
        reject("wrong phase", &|record| {
            record.phase = ResultPhase::SpecializedOnly
        });
        reject("wrong arity", &|record| record.arity += 1);
        reject("missing child", &|record| {
            record.children.pop();
        });
        reject("stale child provenance", &|record| {
            record.children[0].possible_owners = vec![BoundaryReferentOwner::PersistentStore];
        });
    }

    #[test]
    fn d8_trap_predecessors_do_not_create_a_result_edge() {
        let mixed = d8_mixed_join(false);
        let mixed_plan = d8_functionized_plan(&mixed).expect("mixed join plans");
        let mixed_token = mixed_plan
            .join_plan_token(mixed_plan.root_static_origin().expect("mixed root"))
            .expect("mixed join token");
        assert!(mixed_token.has_continuing_predecessor);

        let all_trap = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Constructor {
                constructor: "ctor:fixture::D8::Left".to_string(),
                args: Vec::new(),
            })),
            cases: ["Left", "Right"]
                .into_iter()
                .map(|name| RuntimeMatchCase {
                    constructor: format!("ctor:fixture::D8::{name}"),
                    binders: 0,
                    body: RuntimeExpr::Trap(trap("D8 terminal arm")),
                })
                .collect(),
            default: trap("D8 all-trap default"),
        };
        let all_trap_plan = d8_functionized_plan(&all_trap).expect("all-trap plans");
        let all_trap_token = all_trap_plan
            .join_plan_token(all_trap_plan.root_static_origin().expect("all-trap root"))
            .expect("all-trap join token");
        assert!(!all_trap_token.has_continuing_predecessor);
    }

    #[test]
    fn d8_join_plan_is_a_bijection_with_source_join_occurrences() {
        let expr = governed_nested_resource_bracket(3);
        let plan = d8_functionized_plan(&expr).expect("bracket plans");
        for (occurrence, join) in plan.source_occurrences.iter().zip(&plan.join_results) {
            assert_eq!(
                occurrence
                    .as_ref()
                    .is_some_and(|occurrence| is_source_join(occurrence.expr)),
                join.is_some(),
                "join-plan population differs from the source-join population"
            );
        }

        let mut missing = plan.clone();
        let index = missing
            .join_results
            .iter()
            .position(Option::is_some)
            .expect("governed bracket has a source join");
        missing.join_results[index] = None;
        assert_eq!(
            missing.validate_join_result_plan().unwrap_err(),
            planner_error("source join occurrence has no result representation")
        );
    }

    fn d7_case_partition_declaration(
        symbol: &str,
        cases: Vec<RuntimeMatchCase>,
    ) -> RuntimeDeclaration {
        RuntimeDeclaration {
            symbol: symbol.to_string(),
            kind: RuntimeDeclarationKind::Transparent {
                body: RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["scrutinee".to_string()],
                    body: Box::new(RuntimeExpr::Match {
                        scrutinee: Box::new(RuntimeExpr::Var(0)),
                        cases,
                        default: trap("closed case partition"),
                    }),
                },
            },
            metadata: crate::RuntimeSymbolMetadata::empty(),
        }
    }

    fn d7_case_partition_cases() -> Vec<RuntimeMatchCase> {
        ["A", "B", "Append"]
            .into_iter()
            .map(|name| RuntimeMatchCase {
                constructor: format!("ctor:fixture::D7Case::{name}"),
                binders: 0,
                body: RuntimeExpr::Construct {
                    constructor: format!("ctor:fixture::D7Answer::{name}"),
                    args: Vec::new(),
                },
            })
            .collect()
    }

    fn d7_case_partition_call(symbol: &str, producer: RuntimeExpr) -> RuntimeExpr {
        RuntimeExpr::Call {
            callee: Box::new(RuntimeExpr::DeclarationRef {
                symbol: symbol.to_string(),
            }),
            args: vec![producer],
        }
    }

    fn d7_case_partition_records<'plan>(
        plan: &'plan StaticTransitionPlan<'_>,
    ) -> Vec<&'plan PlannedCaseEmission> {
        let mut grouped =
            BTreeMap::<(PredeclaredFunctionId, StaticOriginId), Vec<&PlannedCaseEmission>>::new();
        for record in &plan.case_emissions {
            grouped
                .entry((record.owner, record.match_origin))
                .or_default()
                .push(record);
        }
        grouped
            .into_values()
            .find(|records| records.len() == 3)
            .expect("the fixture has one three-case carried match")
    }

    fn d7_case_constructor(name: &str) -> RuntimeExpr {
        RuntimeExpr::Construct {
            constructor: format!("ctor:fixture::D7Case::{name}"),
            args: Vec::new(),
        }
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: a runtime `If` contributes both constructor identities and
    /// their producer paths to one exact carried Match partition.
    /// CLAIMED: alternatives union monotonically instead of taking the first
    /// producer found.
    /// THE GAP: this fixture does not exercise recursive cycles; the fixed-point
    /// closure is independently bounded and rejects non-convergence.
    #[test]
    fn d7_case_emission_unions_every_runtime_alternative() {
        let symbol = "decl:fixture::d7-case-union";
        let declaration = d7_case_partition_declaration(symbol, d7_case_partition_cases());
        let producer = RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            then_expr: Box::new(d7_case_constructor("A")),
            else_expr: Box::new(d7_case_constructor("B")),
        };
        let entry = d7_case_partition_call(symbol, producer);
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations).expect("case union plans");
        let records = d7_case_partition_records(&plan);
        assert_eq!(
            records
                .iter()
                .map(|record| record.status)
                .collect::<Vec<_>>(),
            vec![
                CaseEmissionStatus::Reachable,
                CaseEmissionStatus::Reachable,
                CaseEmissionStatus::Eliminated,
            ]
        );
        let ScrutineeProducerSet::Closed(constructors) = &records[0].authority.producers else {
            panic!("a source-only conditional must close");
        };
        assert_eq!(constructors.len(), 2);
        assert_eq!(records[0].authority.producer_origins.len(), 2);
        assert!(
            records[0]
                .authority
                .flow
                .iter()
                .any(|edge| edge.kind == ProducerFlowKind::Alternative),
            "the union has no alternative-flow provenance"
        );

        // Population-side evasion: removing the B-producing alternative must
        // move B from reachable to eliminated.
        let single = d7_case_partition_call(
            symbol,
            RuntimeExpr::If {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                then_expr: Box::new(d7_case_constructor("A")),
                else_expr: Box::new(d7_case_constructor("A")),
            },
        );
        let single_plan =
            d7_functionized_plan(&single, &declarations).expect("single producer plans");
        assert_eq!(
            d7_case_partition_records(&single_plan)[1].status,
            CaseEmissionStatus::Eliminated
        );
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: opaque ingress dominates a closed constructor alternative and
    /// leaves every source case reachable.
    /// CLAIMED: lack of producer closure never authorizes pruning.
    /// THE GAP: the unchanged host-operation gate supplies the later refusal;
    /// this pin guards the planner-side precondition for reaching it.
    #[test]
    fn d7_case_emission_open_ingress_prunes_nothing() {
        let symbol = "decl:fixture::d7-case-open";
        let declaration = d7_case_partition_declaration(symbol, d7_case_partition_cases());
        let producer = RuntimeExpr::If {
            scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
            then_expr: Box::new(d7_case_constructor("A")),
            else_expr: Box::new(RuntimeExpr::PrimitiveCall {
                primitive: crate::RuntimePrimitive {
                    symbol: "opaque_fixture".to_string(),
                    partiality: crate::RuntimePartiality::Total,
                },
                args: Vec::new(),
            }),
        };
        let entry = d7_case_partition_call(symbol, producer);
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations).expect("open producer plans");
        let records = d7_case_partition_records(&plan);
        assert!(matches!(
            records[0].authority.producers,
            ScrutineeProducerSet::Open
        ));
        assert!(
            records
                .iter()
                .all(|record| record.status == CaseEmissionStatus::Reachable),
            "Open unlawfully eliminated a source case"
        );
        assert!(records[0]
            .authority
            .flow
            .iter()
            .any(|edge| edge.kind == ProducerFlowKind::OpaqueIngress));
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: a real `Append` constructor in producer flow makes the exact
    /// `Append` case reachable even though siblings are eliminated.
    /// CLAIMED: the capability catalog is not used as a reachability oracle.
    /// THE GAP: object lowering's literal 13-operation gate is separately
    /// responsible for rejecting the retained unsupported body.
    #[test]
    fn d7_case_emission_retains_a_real_unavailable_producer() {
        let symbol = "decl:fixture::d7-case-unavailable";
        let declaration = d7_case_partition_declaration(symbol, d7_case_partition_cases());
        let entry = d7_case_partition_call(symbol, d7_case_constructor("Append"));
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations).expect("unavailable producer plans");
        let records = d7_case_partition_records(&plan);
        assert_eq!(
            records
                .iter()
                .map(|record| record.status)
                .collect::<Vec<_>>(),
            vec![
                CaseEmissionStatus::Eliminated,
                CaseEmissionStatus::Eliminated,
                CaseEmissionStatus::Reachable,
            ],
            "the unavailable producer was filtered by case spelling"
        );
        assert_eq!(
            records[2].authority.producer_origins.len(),
            1,
            "the retained case has no concrete producer provenance"
        );
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: two syntactically identical case lists at distinct Match
    /// occurrences receive distinct exact partitions from distinct producers.
    /// CLAIMED: match occurrence, owner, and scrutinee edge—not family or case
    /// spelling—key reachability.
    /// THE GAP: phase differs only for carried versus specialized planning;
    /// phase mutation is covered by the bijection pin below.
    #[test]
    fn d7_case_emission_is_keyed_by_exact_match_occurrence() {
        let left_symbol = "decl:fixture::d7-case-left";
        let right_symbol = "decl:fixture::d7-case-right";
        let left = d7_case_partition_declaration(left_symbol, d7_case_partition_cases());
        let right = d7_case_partition_declaration(right_symbol, d7_case_partition_cases());
        let entry = RuntimeExpr::Construct {
            constructor: "ctor:fixture::Pair".to_string(),
            args: vec![
                d7_case_partition_call(left_symbol, d7_case_constructor("A")),
                d7_case_partition_call(right_symbol, d7_case_constructor("B")),
            ],
        };
        let declarations = BTreeMap::from([(left_symbol, &left), (right_symbol, &right)]);
        let plan = d7_functionized_plan(&entry, &declarations).expect("two matches plan");
        let mut groups =
            BTreeMap::<(PredeclaredFunctionId, StaticOriginId), Vec<CaseEmissionStatus>>::new();
        for record in &plan.case_emissions {
            groups
                .entry((record.owner, record.match_origin))
                .or_default()
                .push(record.status);
        }
        let populations = groups.into_values().collect::<BTreeSet<_>>();
        assert!(populations.contains(&vec![
            CaseEmissionStatus::Reachable,
            CaseEmissionStatus::Eliminated,
            CaseEmissionStatus::Eliminated,
        ]));
        assert!(populations.contains(&vec![
            CaseEmissionStatus::Eliminated,
            CaseEmissionStatus::Reachable,
            CaseEmissionStatus::Eliminated,
        ]));
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: each field in the exact case record participates in
    /// pre-emission equality against a fresh producer-flow derivation.
    /// CLAIMED: no missing, duplicate, transplanted, stale, or forged record can
    /// survive to function definition.
    /// THE GAP: move-only construction prevents production lowering from
    /// fabricating a record; this pin exercises corrupt planner state.
    #[test]
    fn d7_case_emission_bijection_rejects_every_identity_axis() {
        let symbol = "decl:fixture::d7-case-bijection";
        let declaration = d7_case_partition_declaration(symbol, d7_case_partition_cases());
        let entry = d7_case_partition_call(symbol, d7_case_constructor("A"));
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations).expect("case partition plans");
        let exact = plan.case_emissions.clone();
        assert!(exact.len() >= 3, "the mutation population is vacuous");

        let reject = |name: &str, records: Vec<PlannedCaseEmission>| {
            assert_ne!(records, exact, "{name} mutation did not change the subject");
            let mut mutated = plan.clone();
            mutated.case_emissions = records;
            assert_eq!(
                mutated.validate_case_emissions().unwrap_err(),
                planner_error("case-emission partition is not the exact producer-flow derivation"),
                "{name} was rejected by the wrong gate"
            );
        };

        reject("omit", exact[1..].to_vec());
        let mut duplicate = exact.clone();
        duplicate.push(exact[0].clone());
        reject("duplicate", duplicate);
        for (name, mutation) in [
            (
                "match",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.match_origin = StaticOriginId(u32::MAX)
                }) as Box<dyn Fn(&mut PlannedCaseEmission)>,
            ),
            (
                "scrutinee",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.scrutinee_origin = StaticOriginId(u32::MAX)
                }),
            ),
            (
                "body",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.body_origin = StaticOriginId(u32::MAX)
                }),
            ),
            (
                "ordinal",
                Box::new(|record: &mut PlannedCaseEmission| record.ordinal = u32::MAX),
            ),
            (
                "constructor",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.constructor = exact[1].constructor
                }),
            ),
            (
                "owner",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.owner = PredeclaredFunctionId(u32::MAX)
                }),
            ),
            (
                "phase",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.phase = ResultPhase::SpecializedOnly
                }),
            ),
            (
                "provenance",
                Box::new(|record: &mut PlannedCaseEmission| record.authority.flow.clear()),
            ),
            (
                "false-elimination",
                Box::new(|record: &mut PlannedCaseEmission| {
                    record.status = CaseEmissionStatus::Eliminated
                }),
            ),
        ] {
            let mut records = exact.clone();
            mutation(&mut records[0]);
            reject(name, records);
        }

        let open_entry = d7_case_partition_call(
            symbol,
            RuntimeExpr::PrimitiveCall {
                primitive: crate::RuntimePrimitive {
                    symbol: "opaque_fixture".to_string(),
                    partiality: crate::RuntimePartiality::Total,
                },
                args: Vec::new(),
            },
        );
        let open_plan =
            d7_functionized_plan(&open_entry, &declarations).expect("open partition plans");
        let mut forged = open_plan.clone();
        for record in &mut forged.case_emissions {
            record.authority.producers = ScrutineeProducerSet::Closed(Vec::new());
            record.status = CaseEmissionStatus::Eliminated;
        }
        assert_eq!(
            forged.validate_case_emissions().unwrap_err(),
            planner_error("case-emission partition is not the exact producer-flow derivation"),
            "Open-to-Closed forgery reached a later gate"
        );
    }

    /// Promise class: durable invariant.
    ///
    /// MEASURED: eliminating one exact case dispositions every planned
    /// boundary use below its same-owner body, including an effect seat, while
    /// retaining the nested source join in the structural accounting
    /// population.
    /// CLAIMED: omitted case bodies emit no anonymous boundary transition and
    /// cannot hide a nested join from the pre-definition ledgers.
    /// THE GAP: helper-call allocation is covered by the object-emission
    /// no-publication controls; this pin closes the planner's causal subtree.
    #[test]
    fn d7_case_emission_eliminated_subtree_is_fully_accounted() {
        let symbol = "decl:fixture::d7-case-eliminated-subtree";
        let mut cases = d7_case_partition_cases();
        cases[1].body = RuntimeExpr::Let {
            value: Box::new(RuntimeExpr::Effect {
                family: "Buffer".to_string(),
                operation: ken_host::HostOpV1::BufferAllocate,
                capability: None,
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(1.into()))],
            }),
            body: Box::new(RuntimeExpr::If {
                scrutinee: Box::new(RuntimeExpr::Value(RuntimeValue::Bool(true))),
                then_expr: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::D7Answer::B".to_string(),
                    args: vec![RuntimeExpr::Var(0)],
                }),
                else_expr: Box::new(RuntimeExpr::Construct {
                    constructor: "ctor:fixture::D7Answer::B".to_string(),
                    args: vec![RuntimeExpr::Var(0)],
                }),
            }),
        };
        let declaration = d7_case_partition_declaration(symbol, cases);
        let entry = d7_case_partition_call(symbol, d7_case_constructor("A"));
        let declarations = BTreeMap::from([(symbol, &declaration)]);
        let plan = d7_functionized_plan(&entry, &declarations).expect("nested dead case plans");
        let records = d7_case_partition_records(&plan);
        let eliminated = records[1];
        assert_eq!(eliminated.status, CaseEmissionStatus::Eliminated);

        let owner = eliminated.owner;
        let root = eliminated.body_origin;
        let mut pending = vec![root];
        let mut subtree = BTreeSet::new();
        while let Some(origin) = pending.pop() {
            if !subtree.insert(origin) {
                continue;
            }
            for child in plan.semantic.child_origins(origin).unwrap() {
                if plan.semantic.function_owner(*child).unwrap() == Some(owner) {
                    pending.push(*child);
                }
            }
        }
        let expected_boundary_uses = plan
            .boundary_uses
            .iter()
            .filter_map(|planned| {
                let origin = match &planned.path {
                    PlannedBoundaryUsePath::Source { parent, .. } => *parent,
                    PlannedBoundaryUsePath::Synthesized { origin, .. } => *origin,
                    PlannedBoundaryUsePath::StaticRecursorWorker {
                        producer_origin, ..
                    }
                    | PlannedBoundaryUsePath::StaticRecursorCapture {
                        producer_origin, ..
                    } => *producer_origin,
                };
                (planned.producer_owner == owner && subtree.contains(&origin))
                    .then_some(planned.identity)
            })
            .collect::<BTreeSet<_>>();
        assert!(
            expected_boundary_uses.iter().any(|identity| {
                plan.boundary_uses.iter().any(|planned| {
                    planned.identity == *identity
                        && matches!(
                            planned.path,
                            PlannedBoundaryUsePath::Source {
                                effect_operation: Some(ken_host::HostOpV1::BufferAllocate),
                                ..
                            }
                        )
                })
            }),
            "the eliminated subtree has no planned effect-seat boundary use"
        );
        let joins = plan
            .source_join_origins_in_owner_subtree(root)
            .expect("dead subtree join population closes");
        assert!(
            !joins.is_empty(),
            "the eliminated subtree has no planned join"
        );

        plan.disposition_boundary_uses_in_owner_subtree(root)
            .expect("the eliminated subtree dispositions causally");
        assert!(
            plan.operand_edge_consumption.borrow().is_empty(),
            "the eliminated body emitted a source boundary use"
        );
        assert!(
            expected_boundary_uses.is_subset(&plan.boundary_use_dispositions.borrow()),
            "one nested planned boundary use escaped elimination accounting"
        );
    }
}
