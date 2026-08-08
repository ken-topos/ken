//! Closed semantic-IR representation referenced by the static transition plan.
//!
//! Boundary A owns scheduling and authority. This plane owns only semantic
//! programs for already-planned nodes; static edges remain body-free transfer
//! contracts.

use super::{
    planner_capacity_error, planner_error, CraneliftBackendError, EdgeKind, StaticEdge,
    StaticEdgeId, StaticNode, StaticNodeId, TransitionKind,
};
use crate::{
    RuntimeExpr, RuntimeIntV1, RuntimePartiality, RuntimePrimitive, RuntimeTrap, RuntimeTrapCode,
    RuntimeValue, Sign,
};
use std::collections::BTreeMap;
#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static OMIT_LAST_IO_ERROR_ROLE: Cell<bool> = const { Cell::new(false) };
}

/// The preallocated positional identity of one planned occurrence.
///
/// Widened to `pub(in crate::cranelift_backend)` so the
/// lowering can carry an occurrence's static name to the site that lowers it.
/// The wrapped ordinal stays `pub(super)` deliberately: a consumer outside this
/// planner can hold, compare, and pass an origin, but **cannot mint one** from
/// an arbitrary integer, so the tag population can only ever be the planner's
/// own.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct StaticOriginId(pub(super) u32);

/// The artifact-static identity of a **constructor** symbol (`D1`).
///
/// ⭐ The wrapped word is `pub(super)` for the same reason `StaticOriginId`'s
/// ordinal is: `lowering` can hold one, compare two, and hand one to the
/// carrier's emitted `tag` / `store_tag_id` helpers, but **cannot mint one**.
/// That is what makes "one identity authority shared by producer and consumer"
/// (`D2`) a property of the *type system* rather than of reviewer vigilance —
/// there is no second derivation available to a consumer, because a consumer
/// has no constructor for this type at all.
///
/// ⛔ **Distinct from [`FieldIdentity`] on purpose.** Constructor identity and
/// record-field identity are different namespaces, and a comparison across them
/// is meaningless. Two newtypes over one authority make that error **fail to
/// compile** rather than merely never happening to be written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct ConstructorIdentity(pub(super) DenseRange);

/// A fixed constructor role synthesized by effect lowering.
///
/// This is a closed capability vocabulary, not a name lookup.  Lowering may
/// ask for one of these roles, but it cannot submit a `RuntimeSymbol`, an
/// ordinal, or an origin and thereby mint a second constructor identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum SynthesizedFixedConstructorRole {
    FileError,
    FileOperationRead,
    FileOperationWrite,
    FileOperationChangeMode,
    OptionSome,
    ResourceHostIo,
    ResourceClosed,
    ResourceMalformed,
    ResourceRightNotHeld,
    ResourceReleaseFailed,
    ResourceKindMismatch,
    ResourceBufferLimit,
    ResourceAllocationFailed,
    ResourceInvalidOffset,
    ResourceInvalidBounds,
    ResourceNoProgress,
    ResourceKindFsHandle,
    ResourceKindBuffer,
    ResourceTraceIdentity,
    PrivateBufferSpan,
    PrivateTransferCount,
    ReadSome,
    ReadEof,
    Wrote,
    Unit,
}

impl SynthesizedFixedConstructorRole {
    pub(super) const ALL: [Self; 25] = [
        Self::FileError,
        Self::FileOperationRead,
        Self::FileOperationWrite,
        Self::FileOperationChangeMode,
        Self::OptionSome,
        Self::ResourceHostIo,
        Self::ResourceClosed,
        Self::ResourceMalformed,
        Self::ResourceRightNotHeld,
        Self::ResourceReleaseFailed,
        Self::ResourceKindMismatch,
        Self::ResourceBufferLimit,
        Self::ResourceAllocationFailed,
        Self::ResourceInvalidOffset,
        Self::ResourceInvalidBounds,
        Self::ResourceNoProgress,
        Self::ResourceKindFsHandle,
        Self::ResourceKindBuffer,
        Self::ResourceTraceIdentity,
        Self::PrivateBufferSpan,
        Self::PrivateTransferCount,
        Self::ReadSome,
        Self::ReadEof,
        Self::Wrote,
        Self::Unit,
    ];

    fn spelling<'a>(self, symbols: &'a crate::NativeProcessSymbols) -> &'a str {
        match self {
            Self::FileError => &symbols.file_error,
            Self::FileOperationRead => &symbols.file_operation_read,
            Self::FileOperationWrite => &symbols.file_operation_write,
            Self::FileOperationChangeMode => &symbols.file_operation_change_mode,
            Self::OptionSome => &symbols.option_some,
            Self::ResourceHostIo => &symbols.resource_host_io,
            Self::ResourceClosed => &symbols.resource_closed,
            Self::ResourceMalformed => &symbols.resource_malformed,
            Self::ResourceRightNotHeld => &symbols.resource_right_not_held,
            Self::ResourceReleaseFailed => &symbols.resource_release_failed,
            Self::ResourceKindMismatch => &symbols.resource_kind_mismatch,
            Self::ResourceBufferLimit => &symbols.resource_buffer_limit,
            Self::ResourceAllocationFailed => &symbols.resource_allocation_failed,
            Self::ResourceInvalidOffset => &symbols.resource_invalid_offset,
            Self::ResourceInvalidBounds => &symbols.resource_invalid_bounds,
            Self::ResourceNoProgress => &symbols.resource_no_progress,
            Self::ResourceKindFsHandle => &symbols.resource_kind_fs_handle,
            Self::ResourceKindBuffer => &symbols.resource_kind_buffer,
            Self::ResourceTraceIdentity => &symbols.resource_trace_identity,
            Self::PrivateBufferSpan => &symbols.private_buffer_span,
            Self::PrivateTransferCount => &symbols.private_transfer_count,
            Self::ReadSome => &symbols.read_some,
            Self::ReadEof => &symbols.read_eof,
            Self::Wrote => &symbols.wrote,
            Self::Unit => &symbols.unit,
        }
    }
}

/// One dynamic `IOError` role minted by semantic-plane construction.
///
/// The wrapped position is planner-private.  Lowering can only receive these
/// values from `StaticTransitionPlan::synthesized_io_error_roles`; it cannot
/// forge an alternative by supplying a vector index.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) struct SynthesizedIoErrorRole(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum SynthesizedConstructorRole {
    Fixed(SynthesizedFixedConstructorRole),
    IoError(SynthesizedIoErrorRole),
}

/// The artifact-static identity of a **record field** name (`D1`).
///
/// See [`ConstructorIdentity`] for why the word is unmintable outside this
/// planner and why the two namespaces are separate types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct FieldIdentity(pub(super) DenseRange);

/// The **one** injective `DenseRange -> u64` encoding, for handing an identity
/// to the carrier's emitted ABI (`store_tag_id`, `tag`, `record_field`,
/// `store_name`).
///
/// ⛔ **No per-site packing arithmetic.** Every identity that reaches emitted
/// code goes through this function, so there is exactly one spelling to review
/// and exactly one to get wrong.
///
/// ⭐ **Zero is retained as the invalid sentinel**, which is why the encoding
/// adds one: a span of `start = 0, len = 0` is a legitimate identity (the empty
/// name at offset zero) and would otherwise be indistinguishable from
/// uninitialized ABI memory.
///
/// ⚠ **Artifact-local only.** This number is stable within one artifact's plane
/// and carries no cross-artifact meaning — spans depend on that artifact's own
/// interning order. ⛔ Do not persist it, compare it across artifacts, or read
/// it as a portable name.
fn pack_identity(span: DenseRange) -> Result<u64, CraneliftBackendError> {
    ((u64::from(span.start) << 32) | u64::from(span.len))
        .checked_add(1)
        .ok_or_else(|| planner_capacity_error("semantic identity encoding exhausted"))
}

/// Inverse of [`pack_identity`]. ⛔ Zero is the invalid sentinel and is refused.
///
/// ⚠ **`cfg(test)` deliberately, and this is a statement about the current
/// state rather than a permanent one.** Nothing in production decodes an
/// identity yet — `S1`/`S2` only ever *hand* one to the carrier ABI. `D3`/`D4`
/// are the consumers that read one back, and they promote this to production.
///
/// ⛔ Not `#[allow(dead_code)]`: the dead-code warning is a free and accurate
/// oracle for "does production consume this?", and silencing it with an
/// attribute would hide exactly the inertness this node exists to remove.
#[cfg(test)]
pub(super) fn unpack_identity(packed: u64) -> Result<DenseRange, CraneliftBackendError> {
    let raw = packed
        .checked_sub(1)
        .ok_or_else(|| planner_error("semantic identity is the reserved invalid sentinel"))?;
    Ok(DenseRange {
        start: (raw >> 32) as u32,
        len: (raw & u64::from(u32::MAX)) as u32,
    })
}

impl ConstructorIdentity {
    /// This identity as the carrier ABI's `tag_id` word.
    ///
    /// ⛔ Deliberately a method on the **typed** identity rather than a free
    /// `u64` conversion: erasing both namespaces to `u64` before choosing
    /// between the tag ABI and the name ABI is exactly the confusion the two
    /// newtypes exist to prevent.
    pub(in crate::cranelift_backend) fn tag_abi_word(self) -> Result<u64, CraneliftBackendError> {
        pack_identity(self.0)
    }
}

impl FieldIdentity {
    /// This identity as the carrier ABI's `name_id` word. See
    /// [`ConstructorIdentity::tag_abi_word`] for why this is not a shared
    /// `u64` conversion.
    pub(in crate::cranelift_backend) fn name_abi_word(self) -> Result<u64, CraneliftBackendError> {
        pack_identity(self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct SemanticProgramId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct CaptureLayoutId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(in crate::cranelift_backend) struct PredeclaredFunctionId(pub(super) u32);

/// Which function unit a planned node belongs to.
///
/// ⛔ **Exhaustive and closed on purpose, and deliberately NOT
/// `Option<PredeclaredFunctionId>`.** The unique `Terminal` and `TrapTerminal`
/// are **shared exit templates**: they are reachable from every unit by
/// construction (`static_transition.rs:835`, `:852`), so they sit outside the
/// exclusive owner partition. They are neither target functions nor *missing
/// data* — an `Option` would say "absent", which is a third thing and is false.
/// A reserved "invalid" id would be worse still, because it type-checks as a
/// function.
///
/// ⭐ This is the withdrawn `AC-5` defect relocated into a **type**. That AC's
/// two-way site classification had no cell for the honest answer, so it could
/// have been filled in *completely* and still been wrong. Here the same defect
/// would have lived in a field whose every value is a `PredeclaredFunctionId`,
/// where the code compiles and exactly two rows are lies. With this enum those
/// two rows cannot be spelled.
///
/// ⚠ Distinct from `StaticNode.owner`, which is a `StaticSourceId` — Boundary
/// A's authority attribution, not a function unit.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum SemanticOwner {
    Function(PredeclaredFunctionId),
    Terminal,
    TrapTerminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct DenseRange {
    pub(super) start: u32,
    pub(super) len: u32,
}

impl DenseRange {
    fn at_end<T>(
        arena: &[T],
        len: usize,
        what: &'static str,
    ) -> Result<Self, CraneliftBackendError> {
        Ok(Self {
            start: u32::try_from(arena.len())
                .map_err(|_| planner_capacity_error(format!("{what} identity exhausted")))?,
            len: u32::try_from(len)
                .map_err(|_| planner_capacity_error(format!("{what} range exhausted")))?,
        })
    }

    fn end(self) -> Option<usize> {
        (self.start as usize).checked_add(self.len as usize)
    }
}

/// The six semantic lowering primitives. This is deliberately independent of
/// Boundary A's scheduling/authority vocabulary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum SemanticOpcode {
    EvaluateExpression,
    TransferValueOrControl,
    SelectBranchOrCase,
    InvokeOrResume,
    ReturnOrComplete,
    RunAffineCleanup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum RuntimeExprShape {
    CheckedJoinSite,
    CheckedSubcontinuationFrame,
    CheckedRecursiveInvocation,
    CheckedComputationalIHSlots,
    CheckedComputationalIHInvocation,
    Value,
    Var,
    Let,
    If,
    PrimitiveCall,
    Construct,
    Match,
    ComputationalMatch,
    Record,
    Project,
    Closure,
    LexicalClosure,
    DeclarationRef,
    ImportedDeclarationRef,
    Call,
    Effect,
    Trap,
}

impl RuntimeExprShape {
    fn of(expr: &RuntimeExpr) -> Self {
        match expr {
            RuntimeExpr::CheckedJoinSite { .. } => Self::CheckedJoinSite,
            RuntimeExpr::CheckedSubcontinuationFrame { .. } => Self::CheckedSubcontinuationFrame,
            RuntimeExpr::CheckedRecursiveInvocation { .. } => Self::CheckedRecursiveInvocation,
            RuntimeExpr::CheckedComputationalIHSlots { .. } => Self::CheckedComputationalIHSlots,
            RuntimeExpr::CheckedComputationalIHInvocation { .. } => {
                Self::CheckedComputationalIHInvocation
            }
            RuntimeExpr::Value(_) => Self::Value,
            RuntimeExpr::Var(_) => Self::Var,
            RuntimeExpr::Let { .. } => Self::Let,
            RuntimeExpr::If { .. } => Self::If,
            RuntimeExpr::PrimitiveCall { .. } => Self::PrimitiveCall,
            RuntimeExpr::Construct { .. } => Self::Construct,
            RuntimeExpr::Match { .. } => Self::Match,
            RuntimeExpr::ComputationalMatch { .. } => Self::ComputationalMatch,
            RuntimeExpr::Record { .. } => Self::Record,
            RuntimeExpr::Project { .. } => Self::Project,
            RuntimeExpr::Closure { .. } => Self::Closure,
            RuntimeExpr::LexicalClosure { .. } => Self::LexicalClosure,
            RuntimeExpr::DeclarationRef { .. } => Self::DeclarationRef,
            RuntimeExpr::ImportedDeclarationRef { .. } => Self::ImportedDeclarationRef,
            RuntimeExpr::Call { .. } => Self::Call,
            RuntimeExpr::Effect { .. } => Self::Effect,
            RuntimeExpr::Trap(_) => Self::Trap,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) enum SemanticSourceKind {
    Expression(RuntimeExprShape),
    Control(TransitionKind),
}

/// Fixed-width occurrence registered during the planner's source walk. Its
/// origin is allocated with the planned node, before the semantic plane or any
/// later activation exists.
///
/// `source_material_elements` is the occurrence's **total** one-visit material
/// budget, and it is partitioned exactly: `material` spans this occurrence's
/// non-child atoms and `children` spans its positional syntax-child origins,
/// with `material.len + children.len == source_material_elements`. Both ranges
/// point into the walk's `SemanticMaterialArena`, so the seed itself stays
/// fixed-width and `Copy`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticSourceSeed {
    pub(super) planned_node: StaticNodeId,
    pub(super) origin: StaticOriginId,
    pub(super) source: SemanticSourceKind,
    pub(super) source_material_elements: u32,
    pub(super) capture_slots: u32,
    pub(super) material: DenseRange,
    pub(super) children: DenseRange,
}

impl SemanticSourceSeed {
    /// Registers one expression occurrence and emits its material in the same
    /// visit. `children` are the occurrence's syntax children in **source
    /// position order**, already planned by the walk; their origins are the
    /// children's own preallocated positional identities, never minted here.
    ///
    /// ⭐ They are each child's **occurrence** origin, never its scheduling
    /// entry. The parameter is `&[StaticOriginId]` rather than
    /// `&[StaticNodeId]` so the type prevents that conflation instead of the
    /// call sites having to remember it: for a `ComputationalMatch` child the
    /// two are deliberately different nodes, and passing a scheduling entry
    /// here is a category error, not an off-by-one.
    pub(super) fn expression(
        planned_node: StaticNodeId,
        expr: &RuntimeExpr,
        children: &[StaticOriginId],
        arena: &mut SemanticMaterialArena,
    ) -> Result<Self, CraneliftBackendError> {
        let atom_start = arena.atoms.len();
        let child_start = arena.child_origins.len();
        emit_expression_atoms(expr, arena)?;
        arena.child_origins.extend_from_slice(children);
        let material = arena.atoms_since(atom_start)?;
        let child_range = arena.children_since(child_start)?;

        // The emitted partition must exhaust exactly the same one-visit budget
        // the walk has always counted. A disagreement is a compiler bug in the
        // emitter or the budget, never an input condition.
        let budget = source_material_elements(expr)?;
        let emitted = material
            .len
            .checked_add(child_range.len)
            .ok_or_else(|| planner_capacity_error("semantic source material exhausted"))?;
        if emitted != budget {
            return Err(planner_error(
                "emitted semantic material does not exhaust its one-visit source-material budget",
            ));
        }

        Ok(Self {
            planned_node,
            origin: StaticOriginId(planned_node.0),
            source: SemanticSourceKind::Expression(RuntimeExprShape::of(expr)),
            source_material_elements: budget,
            material,
            children: child_range,
            capture_slots: match expr {
                RuntimeExpr::Closure { captures, .. } => checked_len(captures.len())?,
                RuntimeExpr::LexicalClosure { captures, .. } => checked_len(captures.len())?,
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
                | RuntimeExpr::Trap(_) => 0,
            },
        })
    }

    /// A generated outer control occurrence. It has no source material and no
    /// syntax children: its transfer topology is the ruled-children graph.
    pub(super) const fn control(planned_node: StaticNodeId, transition: TransitionKind) -> Self {
        Self {
            planned_node,
            origin: StaticOriginId(planned_node.0),
            source: SemanticSourceKind::Control(transition),
            source_material_elements: 0,
            capture_slots: 0,
            material: DenseRange { start: 0, len: 0 },
            children: DenseRange { start: 0, len: 0 },
        }
    }
}

/// One occurrence-local **non-child** semantic atom.
///
/// Fixed width: the atom names its own kind, an out-of-line content span (empty
/// when the atom is purely numeric), and a numeric payload. Atoms are
/// self-describing, so a consumer recovers the occurrence's material by walking
/// the record's atom range in position order.
///
/// ⛔ A syntax child is **not** an atom. Child positions live in the record's
/// positional child-origin range, so child *k* is recoverable as child *k* —
/// never by search, shape-matching, pointer, or clone order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticOperandElement {
    pub(super) kind: SemanticAtomKind,
    pub(super) content: DenseRange,
    pub(super) payload: u64,
}

/// The closed vocabulary of non-child semantic atoms. There is deliberately no
/// wildcard consumer of this enum: a new atom kind must be handled explicitly
/// wherever material is interpreted.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum SemanticAtomKind {
    /// Checked compiler-private site/frame identity.
    CheckedSiteId,
    CheckedFrameId,
    /// Reusable static call-template identity, and one checked occurrence-path
    /// step of the path that selects it.
    CallTemplateId,
    OccurrencePathLen,
    OccurrencePathStep,
    SlotTemplateId,
    /// De Bruijn local index of a `Var` occurrence.
    LocalIndex,
    /// One complete primitive: `content` spans an injective tagged encoding of
    /// the symbol **and** its `RuntimePartiality`, including every variant field.
    /// Partiality changes what lowering emits, so a symbol-only atom would let
    /// two same-shaped occurrences share one body while lowering differently.
    PrimitiveDescriptor,
    /// Symbol atoms: `content` spans the interned name bytes.
    ConstructorSymbol,
    DeclarationSymbol,
    DependencySymbol,
    DependencyHash,
    RecordFieldName,
    ProjectField,
    CaptureSymbol,
    ParamName,
    EffectFamily,
    EffectOperation,
    /// Eliminator material: the default trap, then per case its constructor,
    /// its binder count, one atom per binder, and one per recursive position.
    MatchDefault,
    CaseConstructor,
    CaseBinders,
    CaseBinder,
    CaseRecursivePosition,
    /// Trap material.
    TrapCode,
    TrapMessage,
    /// Flattened `RuntimeValue` material, emitted in source pre-order.
    ValueBool,
    ValueIntSmall,
    ValueIntBig,
    ValueBytes,
    ValueString,
    ValueConstructor,
    ValueRecord,
    ValueClosureRef,
    ValueUnknown,
    /// One literal byte of a `Bytes`/`String` value.
    ByteLiteral,
}

/// Out-of-line material accumulated by the planner's single source walk.
///
/// The walk that allocates a planned node and its origin also emits that
/// occurrence's atoms and child origins here, in one visit. `build_semantic_plane`
/// re-lays this material positionally into the plane; it never re-derives it.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct SemanticMaterialArena {
    atoms: Vec<SemanticOperandElement>,
    child_origins: Vec<StaticOriginId>,
    names: Vec<u8>,
    /// ⛔ **Construction-local only.** Exact-byte lookup so `intern` can return
    /// the *canonical* span for content it has already seen.
    ///
    /// ⛔ This is **not** a second identity authority and must never become one:
    /// it is never copied into the plane, never exported, and never consulted
    /// after plane construction. `build_semantic_plane` copies `names` and
    /// nothing else. It is a *memo* of the one derivation, discarded with the
    /// arena — deleting it would change performance and not meaning.
    ///
    /// ⚠ It participates in the derived `PartialEq` harmlessly: the map is a
    /// pure function of the interning history, so two arenas with equal `names`
    /// necessarily have equal caches.
    canonical_names: BTreeMap<Vec<u8>, DenseRange>,
}

impl SemanticMaterialArena {
    /// Intern `bytes` **content-addressed**: equal bytes always yield the same
    /// canonical [`DenseRange`], which is the artifact-static identity of that
    /// symbol and the sole stored form of it.
    ///
    /// ⭐ This is what makes a producer's identity for `Cons` and an
    /// eliminator's identity for `Cons` the *same value* even when they are
    /// different occurrences — the property `D2` requires and the reason the
    /// span, rather than an added id, is the identity. `names[span]` remains the
    /// diagnostic view over that one authority.
    ///
    /// ⛔ Before `RT-FNSPLIT-C1` this appended unconditionally, so equal content
    /// produced *unequal* spans and no shared identity existed. Restoring the
    /// unconditional append re-breaks `D2` while leaving every span in bounds —
    /// which is why `SemanticPlane::validate` rejects unequal spans for equal
    /// bytes rather than trusting this function.
    fn intern(&mut self, bytes: &[u8]) -> Result<DenseRange, CraneliftBackendError> {
        // ⛔ The empty span is canonically `(0, 0)`, matching what `push_numeric`
        // stores for an atom with no out-of-line content.
        //
        // ⚠ Without this, empty content has **two** spellings: `push_numeric`'s
        // `(0, 0)` and `at_end`'s `(names.len(), 0)`. Both denote the same empty
        // byte string, so the canonicality invariant — equal bytes, equal span —
        // would be violated by construction on any plan holding both a numeric
        // atom and an empty string/bytes literal, and the validator would report
        // a two-identity symbol that is really just two spellings of "nothing".
        if bytes.is_empty() {
            return Ok(DenseRange { start: 0, len: 0 });
        }
        if let Some(span) = self.canonical_names.get(bytes) {
            return Ok(*span);
        }
        let span = DenseRange::at_end(&self.names, bytes.len(), "semantic name")?;
        self.names.extend_from_slice(bytes);
        self.canonical_names.insert(bytes.to_vec(), span);
        Ok(span)
    }

    fn push_atom(
        &mut self,
        kind: SemanticAtomKind,
        content: DenseRange,
        payload: u64,
    ) -> Result<(), CraneliftBackendError> {
        if self.atoms.len() == u32::MAX as usize {
            return Err(planner_capacity_error("semantic atom identity exhausted"));
        }
        self.atoms.push(SemanticOperandElement {
            kind,
            content,
            payload,
        });
        Ok(())
    }

    fn push_numeric(
        &mut self,
        kind: SemanticAtomKind,
        payload: u64,
    ) -> Result<(), CraneliftBackendError> {
        self.push_atom(kind, DenseRange { start: 0, len: 0 }, payload)
    }

    fn push_named(
        &mut self,
        kind: SemanticAtomKind,
        name: &str,
        payload: u64,
    ) -> Result<(), CraneliftBackendError> {
        let span = self.intern(name.as_bytes())?;
        self.push_atom(kind, span, payload)
    }

    fn atoms_since(&self, start: usize) -> Result<DenseRange, CraneliftBackendError> {
        range_since(start, self.atoms.len(), "semantic operand")
    }

    fn children_since(&self, start: usize) -> Result<DenseRange, CraneliftBackendError> {
        range_since(start, self.child_origins.len(), "semantic child origin")
    }
}

pub(super) fn build_synthesized_constructor_inventory(
    arena: &mut SemanticMaterialArena,
    symbols: &crate::NativeProcessSymbols,
) -> Result<
    (
        BTreeMap<SynthesizedConstructorRole, DenseRange>,
        Vec<SynthesizedIoErrorRole>,
    ),
    CraneliftBackendError,
> {
    let mut identities = BTreeMap::new();
    for role in SynthesizedFixedConstructorRole::ALL {
        let span = arena.intern(role.spelling(symbols).as_bytes())?;
        identities.insert(SynthesizedConstructorRole::Fixed(role), span);
    }

    let mut io_roles = Vec::with_capacity(symbols.io_errors.len());
    for (position, spelling) in symbols.io_errors.iter().enumerate() {
        let role = SynthesizedIoErrorRole(u32::try_from(position).map_err(|_| {
            planner_capacity_error("synthesized IOError role population exhausted")
        })?);
        let span = arena.intern(spelling.as_bytes())?;
        #[cfg(test)]
        let omit = OMIT_LAST_IO_ERROR_ROLE.with(Cell::get)
            && position + 1 == symbols.io_errors.len();
        #[cfg(not(test))]
        let omit = false;
        if !omit {
            identities.insert(SynthesizedConstructorRole::IoError(role), span);
        }
        io_roles.push(role);
    }
    Ok((identities, io_roles))
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn with_last_io_error_role_omitted<T>(
    body: impl FnOnce() -> T,
) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            OMIT_LAST_IO_ERROR_ROLE.with(|flag| flag.set(false));
        }
    }
    OMIT_LAST_IO_ERROR_ROLE.with(|flag| flag.set(true));
    let _reset = Reset;
    body()
}

fn range_since(
    start: usize,
    end: usize,
    what: &'static str,
) -> Result<DenseRange, CraneliftBackendError> {
    let len = end
        .checked_sub(start)
        .ok_or_else(|| planner_error("semantic material range moved backwards"))?;
    Ok(DenseRange {
        start: u32::try_from(start)
            .map_err(|_| planner_capacity_error(format!("{what} identity exhausted")))?,
        len: u32::try_from(len)
            .map_err(|_| planner_capacity_error(format!("{what} range exhausted")))?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct CaptureSlot {
    pub(super) ordinal: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct RuledChild {
    pub(super) node: StaticNodeId,
    pub(super) edge: StaticEdgeId,
}

/// One canonical occurrence-local material record, exactly one per
/// `StaticOriginId`. `operands` spans this occurrence's non-child atoms;
/// `child_origins` is its **positional** dense range of syntax-child origins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticRecord {
    pub(super) opcode: SemanticOpcode,
    pub(super) origin: StaticOriginId,
    pub(super) operands: DenseRange,
    pub(super) child_origins: DenseRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticProgram {
    pub(super) id: SemanticProgramId,
    pub(super) records: DenseRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct CaptureLayout {
    pub(super) id: CaptureLayoutId,
    pub(super) slots: DenseRange,
}

/// One function unit: a **seed** of the ownership partition, not a planned node.
///
/// ⛔ This table used to be a positional alias of the node table — one row per
/// planned node, `PredeclaredFunctionId(planned_node.0)` — so a type whose name
/// claimed "function" was populated with abstract-machine transition states. It
/// is now populated from the ruled seeds:
///
/// ```text
/// all scheduling entries in plan.entries   (root + each transparent declaration)
///   UNION
/// all TARGETS of EdgeKind::StaticBody edges  (each retained closure-body entry)
/// ```
///
/// `planned_node` is this unit's **entry** node. ⚠ `id.0` is a dense ordinal over
/// the seeds and is **no longer** equal to `planned_node.0`; any code that
/// recovers one from the other is reintroducing the alias.
///
/// ⛔ There is exactly **one** table in this plane whose name claims "function",
/// because `RT-FNSPLIT-B2R` attaches signatures and frame layouts to it and
/// cannot be told which of two to use. Node-scoped semantic material stays in
/// `SemanticDescriptor` + `SemanticProgram` + `SemanticRecord`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct PredeclaredFunction {
    pub(super) id: PredeclaredFunctionId,
    pub(super) planned_node: StaticNodeId,
    pub(super) origin: StaticOriginId,
    pub(super) program: SemanticProgramId,
}

/// One positional semantic definition for one already-planned node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticDescriptor {
    pub(super) planned_node: StaticNodeId,
    pub(super) origin: StaticOriginId,
    pub(super) program: SemanticProgramId,
    pub(super) capture_layout: CaptureLayoutId,
    /// The function unit this occurrence belongs to.
    ///
    /// ⛔ Formerly `function: PredeclaredFunctionId`, filled with
    /// `PredeclaredFunctionId(planned_node.0)` — an identity alias carrying no
    /// information. It now names the **owning** unit, which is what makes the
    /// 59-call population dispositionable by owner and reaching path instead of
    /// by source site.
    pub(super) owner: SemanticOwner,
    pub(super) ruled_children: DenseRange,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct SemanticPlane {
    pub(super) descriptors: Vec<SemanticDescriptor>,
    pub(super) programs: Vec<SemanticProgram>,
    pub(super) records: Vec<SemanticRecord>,
    /// This plane's non-child semantic atoms, laid out positionally per record.
    pub(super) operands: Vec<SemanticOperandElement>,
    /// Positional syntax-child origins, laid out per record. Distinct from
    /// `ruled_children`, which is the transfer graph and not a source-child map.
    pub(super) child_origins: Vec<StaticOriginId>,
    /// Interned atom content (symbols, literal bytes, big-integer limbs).
    pub(super) names: Vec<u8>,
    pub(super) capture_layouts: Vec<CaptureLayout>,
    pub(super) capture_slots: Vec<CaptureSlot>,
    pub(super) ruled_children: Vec<RuledChild>,
    pub(super) functions: Vec<PredeclaredFunction>,
    /// Constructor identities for the exact role population effect lowering
    /// may synthesize.  Values are spans in `names`, produced by the same
    /// private interner as source constructor atoms.
    synthesized_constructor_roles: BTreeMap<SynthesizedConstructorRole, DenseRange>,
    synthesized_io_error_roles: Vec<SynthesizedIoErrorRole>,
}

/// The unique pair of shared exit templates, located and checked as a pair.
///
/// ⚠ `StaticTransitionPlan::validate` also requires exactly one of each. That is
/// not redundant: this check runs during plane construction, *before* the plan's
/// own validation, and a mutation control that hands a corrupted plane straight
/// to `SemanticPlane::validate` never reaches the plan-level check at all.
fn shared_exits(
    nodes: &[StaticNode],
) -> Result<(StaticNodeId, StaticNodeId), CraneliftBackendError> {
    let mut terminal = None;
    let mut trap_terminal = None;
    for node in nodes {
        match node.transition {
            TransitionKind::Terminal => {
                if terminal.replace(node.id).is_some() {
                    return Err(planner_error(
                        "closed graph has more than one Terminal shared exit",
                    ));
                }
            }
            TransitionKind::TrapTerminal => {
                if trap_terminal.replace(node.id).is_some() {
                    return Err(planner_error(
                        "closed graph has more than one TrapTerminal shared exit",
                    ));
                }
            }
            _ => {}
        }
    }
    let terminal =
        terminal.ok_or_else(|| planner_error("closed graph has no Terminal shared exit"))?;
    let trap_terminal = trap_terminal
        .ok_or_else(|| planner_error("closed graph has no TrapTerminal shared exit"))?;
    Ok((terminal, trap_terminal))
}

/// The derived ownership partition: the function-unit seeds, and one owner per
/// planned node.
struct OwnershipPartition {
    /// Seed entry nodes, dense by `PredeclaredFunctionId` ordinal: `entries`
    /// first, in order, then every `StaticBody` target in edge order.
    seeds: Vec<StaticNodeId>,
    /// One owner per planned node, dense by node index.
    owners: Vec<SemanticOwner>,
}

/// Derives the function-unit partition from the plan graph.
///
/// ⛔ **Derived, never hand-authored.** A map read off the graph cannot drift
/// from it; a parallel table would need its own agreement checker, which is one
/// more thing that can be green for the wrong reason.
///
/// The seeds are the ruled ones (Architect `evt_48dxvb2yrwpad`):
///
/// ```text
/// every scheduling entry in plan.entries    (root + each transparent declaration)
///   UNION
/// every TARGET of an EdgeKind::StaticBody edge   (each retained closure-body entry)
/// ```
///
/// ⛔ **`TransitionKind::ClosureBody` is NOT a head.** It is the body's *return
/// successor*: `static_transition.rs:833-836` makes the `ClosureBody` control
/// node **first**, wires it to the shared terminal, plans the body **toward** it,
/// and only then adds the `StaticBody` edge to `body.entry`. Seeding on
/// `ClosureBody` nodes would pick return nodes instead of entries **and** make
/// the edge law unsatisfiable, because that terminal edge is a non-`StaticBody`
/// edge out of a body-owned node.
///
/// Traversal excludes both typed call-edge kinds. `StaticBody` seeds a new
/// closure-body unit; `DeclarationCall` targets an already-seeded scheduling
/// entry. The two shared exits are never owned and never traversed through —
/// they have no outgoing edges by construction (`static_transition.rs:1258`).
/// **`RT-DECL-CLOSURE-PORT` `D2a` — the declaration-owned pairs.**
///
/// A transparent declaration whose body is a closure seed is planned as an
/// `Evaluate` node for the closure (which is what `plan.entries` holds) plus one
/// forward `EdgeKind::StaticBody` edge to the closure body's entry. `D2`
/// classifies that *target* as the declaration-owned `AbiUnitDefinition::
/// CallableDeclaration`, and `D4` resolves every `DeclarationRef` to it.
///
/// ⛔⛔ **The pair is ONE function, not two.** Before `D2a` both ends were
/// seeds, so one source declaration contributed two emitted functions: the new
/// parameter/capture-bearing callable unit, and the old zero-input scheduling
/// entry at the closure occurrence. That second function has **no lawful
/// runtime meaning** — it cannot call the callable unit without the missing
/// parameters and captures, cannot return the closure (there is deliberately no
/// carrier for one), and cannot become a no-op without changing program
/// meaning. It is refused at `boundary_transfer_admissibility` the moment the
/// functionized lane is actually entered.
///
/// ⭐ **Derived solely from the declaration occurrence and its one forward
/// `StaticBody` relation** (Architect `evt_3twrm71vck49d`). ⛔ No source
/// whitelist, no reverse body search, no call-site reachability or
/// "referenced declaration" filter — each of those would make the population
/// depend on something other than the declaration's own shape.
///
/// ⚠ **The root is excluded explicitly, not by position.** The ruled partition
/// keeps the process/root entry as one `SchedulingEntry`, and a root whose
/// expression is itself a closure would otherwise match this rule exactly.
/// Relying on `entries[0]` would encode the push order at
/// `static_transition.rs`, which is not a law anyone states.
///
/// ⚠ Returned POSITIONALLY (`declaration_owned_body[node] = Some(body)`), never
/// as a map keyed by the entry node. A `ComputationalMatch` shares its entry
/// node with its scrutinee chain, so a node-keyed collection can silently merge
/// two occurrences — the standing tripwire in `control.rs` exists for exactly
/// that, and indexing is both cheaper and immune to it.
/// **`RT-DECL-CLOSURE-PORT` `D2a` — the causal control on the substitution.**
///
/// `Retained` restores the pre-`D2a` population: the closure-seed declaration's
/// scheduling entry seeds a function again, so one source declaration
/// contributes two emitted functions. ⛔ Without this, the population assertions
/// are consistent with `D2a` never having been installed on a fixture that
/// happened to agree.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D2aPopulationMutation {
    Exact,
    RetainObsoleteSchedulingUnit,
}

#[cfg(test)]
thread_local! {
    static D2A_POPULATION_MUTATION: std::cell::Cell<D2aPopulationMutation> =
        const { std::cell::Cell::new(D2aPopulationMutation::Exact) };
}

/// Run `body` with the pre-`D2a` population restored, restoring `Exact` on the
/// way out **including on panic**.
#[cfg(test)]
pub(in crate::cranelift_backend) fn with_d2a_population_mutation<T>(
    mutation: D2aPopulationMutation,
    body: impl FnOnce() -> T,
) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            D2A_POPULATION_MUTATION.with(|cell| cell.set(D2aPopulationMutation::Exact));
        }
    }
    D2A_POPULATION_MUTATION.with(|cell| cell.set(mutation));
    let _restore = Restore;
    body()
}

fn declaration_owned_pairs(
    node_count: usize,
    edges: &[StaticEdge],
    entries: &[StaticNodeId],
    root_entry: Option<StaticNodeId>,
) -> Result<(Vec<Option<StaticNodeId>>, usize), CraneliftBackendError> {
    let mut declaration_owned_body = vec![None; node_count];
    let mut pairs = 0usize;
    #[cfg(test)]
    if D2A_POPULATION_MUTATION.with(std::cell::Cell::get)
        == D2aPopulationMutation::RetainObsoleteSchedulingUnit
    {
        return Ok((declaration_owned_body, pairs));
    }
    for entry in entries {
        if Some(*entry) == root_entry {
            continue;
        }
        let mut body = None;
        for edge in edges {
            if edge.kind != EdgeKind::StaticBody || edge.from != *entry {
                continue;
            }
            if body.is_some() {
                return Err(planner_error(
                    "a declaration scheduling entry has two forward static body edges",
                ));
            }
            body = Some(edge.to);
        }
        if let Some(body) = body {
            let slot = declaration_owned_body
                .get_mut(entry.0 as usize)
                .ok_or_else(|| planner_error("scheduling entry is outside the planned nodes"))?;
            *slot = Some(body);
            pairs += 1;
        }
    }
    Ok((declaration_owned_body, pairs))
}

fn partition_function_units(
    nodes: &[StaticNode],
    edges: &[StaticEdge],
    entries: &[StaticNodeId],
    root_entry: Option<StaticNodeId>,
) -> Result<OwnershipPartition, CraneliftBackendError> {
    let (terminal, trap_terminal) = shared_exits(nodes)?;
    let (declaration_owned_body, _pairs) =
        declaration_owned_pairs(nodes.len(), edges, entries, root_entry)?;
    let is_declaration_owned =
        |node: StaticNodeId| declaration_owned_body[node.0 as usize].is_some();

    // Seed class 1 is `entries`; seed class 2 is the `StaticBody` targets. The
    // three ways this can be malformed get three distinct failures on purpose —
    // one composite "the seeds are fine" check is discharged by any one of them
    // holding, so it could not distinguish the mutations AC-5 requires.
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum SeedClass {
        SchedulingEntry,
        StaticBodyTarget,
        /// `D2a`: a closure-seed declaration's occurrence. Marked, owned, and
        /// deliberately NOT a seed.
        DeclarationOwnedEntry,
    }
    let mut seed_class = vec![None; nodes.len()];
    let mut seeds = Vec::with_capacity(entries.len());
    for entry in entries {
        let index = entry.0 as usize;
        let slot = seed_class
            .get_mut(index)
            .ok_or_else(|| planner_error("scheduling entry is outside the planned nodes"))?;
        if slot.is_some() {
            return Err(planner_error(
                "closed graph contains a duplicate scheduling entry",
            ));
        }
        // `D2a`: a declaration-owned entry is still marked, so a duplicate or a
        // collision with a static-body target is still caught — but it seeds no
        // function of its own. Its subgraph joins the callable unit below.
        *slot = Some(if is_declaration_owned(*entry) {
            SeedClass::DeclarationOwnedEntry
        } else {
            SeedClass::SchedulingEntry
        });
        if !is_declaration_owned(*entry) {
            seeds.push(*entry);
        }
    }
    for edge in edges {
        if edge.kind != EdgeKind::StaticBody {
            continue;
        }
        let index = edge.to.0 as usize;
        let slot = seed_class
            .get_mut(index)
            .ok_or_else(|| planner_error("static body target is outside the planned nodes"))?;
        match *slot {
            Some(SeedClass::SchedulingEntry) | Some(SeedClass::DeclarationOwnedEntry) => {
                return Err(planner_error(
                    "scheduling entry is also a static body target",
                ));
            }
            Some(SeedClass::StaticBodyTarget) => {
                return Err(planner_error(
                    "static body target has more than one incoming static body edge",
                ));
            }
            None => *slot = Some(SeedClass::StaticBodyTarget),
        }
        seeds.push(edge.to);
    }
    // `D2a`: the declaration occurrence traverses WITH its callable unit. ⭐ It
    // remains that unit's ownership, provenance and `D3` signature authority —
    // the ruling forbids it becoming an unowned semantic node just as firmly as
    // it forbids a second emitted definition.
    let mut extra_roots = vec![Vec::new(); nodes.len()];
    for (index, body) in declaration_owned_body.iter().enumerate() {
        if let Some(body) = body {
            extra_roots[body.0 as usize].push(StaticNodeId(index as u32));
        }
    }

    let mut outgoing = vec![Vec::new(); nodes.len()];
    for edge in edges {
        if matches!(
            edge.kind,
            EdgeKind::StaticBody | EdgeKind::DeclarationCall
        ) {
            continue;
        }
        if edge.to.0 as usize >= nodes.len() {
            return Err(planner_error("transfer edge target is outside the planned nodes"));
        }
        outgoing
            .get_mut(edge.from.0 as usize)
            .ok_or_else(|| planner_error("transfer edge source is outside the planned nodes"))?
            .push(edge.to);
    }

    let is_shared_exit = |node: StaticNodeId| node == terminal || node == trap_terminal;
    let mut owners = vec![None; nodes.len()];
    owners[terminal.0 as usize] = Some(SemanticOwner::Terminal);
    owners[trap_terminal.0 as usize] = Some(SemanticOwner::TrapTerminal);

    for (ordinal, seed) in seeds.iter().enumerate() {
        let unit = SemanticOwner::Function(PredeclaredFunctionId(
            u32::try_from(ordinal)
                .map_err(|_| planner_capacity_error("function unit identity exhausted"))?,
        ));
        let mut frontier = vec![*seed];
        frontier.extend_from_slice(&extra_roots[seed.0 as usize]);
        while let Some(node) = frontier.pop() {
            if is_shared_exit(node) {
                // A shared exit is this unit's local return or trap, never a
                // node it owns and never a node to traverse through.
                continue;
            }
            match owners[node.0 as usize] {
                Some(existing) if existing == unit => continue,
                Some(SemanticOwner::Function(_)) => {
                    return Err(planner_error(
                        "planned node is owned by more than one function unit",
                    ));
                }
                Some(SemanticOwner::Terminal) | Some(SemanticOwner::TrapTerminal) => {
                    return Err(planner_error("shared exit was reached as an owned node"));
                }
                None => {
                    owners[node.0 as usize] = Some(unit);
                    frontier.extend_from_slice(&outgoing[node.0 as usize]);
                }
            }
        }
    }

    let owners = owners
        .into_iter()
        .map(|owner| owner.ok_or_else(|| planner_error("planned node has no function unit owner")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OwnershipPartition { seeds, owners })
}

/// The sole semantic-definition constructor. It positions seeds by their
/// already-planned node ID, visits each node once, visits each edge once, and
/// flattens each variable source/capture collection once.
///
/// ⚠ `entries` is threaded in because it is **planner state, not graph
/// structure**: the scheduling entries are pushed at `static_transition.rs:1728`
/// and `:1734` and cannot be recovered from `nodes`/`edges`. "A node with no
/// incoming `StaticBody`" is *not* the same set — every ordinary node satisfies
/// that too.
pub(super) fn build_semantic_plane(
    nodes: &[StaticNode],
    edges: &[StaticEdge],
    entries: &[StaticNodeId],
    root_entry: Option<StaticNodeId>,
    sources: &[SemanticSourceSeed],
    arena: &SemanticMaterialArena,
) -> Result<SemanticPlane, CraneliftBackendError> {
    let mut positioned = vec![None; nodes.len()];
    for source in sources {
        let slot = positioned
            .get_mut(source.planned_node.0 as usize)
            .ok_or_else(|| planner_error("semantic source names an unknown planned node"))?;
        if slot.replace(*source).is_some() {
            return Err(planner_error(
                "planned node has more than one semantic source",
            ));
        }
    }
    if positioned.iter().any(Option::is_none) {
        return Err(planner_error(
            "planned node lacks its preallocated semantic source",
        ));
    }

    let mut outgoing = vec![Vec::new(); nodes.len()];
    for edge in edges {
        let children = outgoing
            .get_mut(edge.from.0 as usize)
            .ok_or_else(|| planner_error("semantic edge source is outside the planned nodes"))?;
        children.push(RuledChild {
            node: edge.to,
            edge: edge.id,
        });
    }

    let partition = partition_function_units(nodes, edges, entries, root_entry)?;

    let mut plane = SemanticPlane::default();
    // Atom content is referenced by absolute span, so the interned bytes move
    // across whole; only the atom and child-origin arenas are re-laid per record.
    plane.names.extend_from_slice(&arena.names);
    for (position, source) in positioned.into_iter().enumerate() {
        let source = source.expect("all source positions checked above");
        let planned_node = StaticNodeId(
            u32::try_from(position)
                .map_err(|_| planner_capacity_error("semantic node identity exhausted"))?,
        );
        let origin = source.origin;
        let program = SemanticProgramId(planned_node.0);
        let capture_layout = CaptureLayoutId(planned_node.0);
        let owner = partition.owners[position];

        // Positional re-lay of the material the source walk already emitted for
        // this origin. Nothing is re-derived here, and no placeholder is minted.
        let operand_range =
            DenseRange::at_end(&plane.operands, source.material.len as usize, "semantic operand")?;
        plane
            .operands
            .extend_from_slice(arena_slice(&arena.atoms, source.material, "semantic operand")?);

        let child_origin_range = DenseRange::at_end(
            &plane.child_origins,
            source.children.len as usize,
            "semantic child origin",
        )?;
        plane.child_origins.extend_from_slice(arena_slice(
            &arena.child_origins,
            source.children,
            "semantic child origin",
        )?);

        let slot_range = DenseRange::at_end(
            &plane.capture_slots,
            source.capture_slots as usize,
            "capture slot",
        )?;
        plane
            .capture_slots
            .extend((0..source.capture_slots).map(|ordinal| CaptureSlot { ordinal }));
        plane.capture_layouts.push(CaptureLayout {
            id: capture_layout,
            slots: slot_range,
        });

        let record_range = DenseRange::at_end(&plane.records, 1, "semantic record")?;
        plane.records.push(SemanticRecord {
            opcode: opcode_for_source(source.source),
            origin,
            operands: operand_range,
            child_origins: child_origin_range,
        });
        plane.programs.push(SemanticProgram {
            id: program,
            records: record_range,
        });

        let node_children = &outgoing[position];
        let child_range =
            DenseRange::at_end(&plane.ruled_children, node_children.len(), "ruled child")?;
        plane.ruled_children.extend(node_children);

        plane.descriptors.push(SemanticDescriptor {
            planned_node,
            origin,
            program,
            capture_layout,
            owner,
            ruled_children: child_range,
        });
    }

    // One row per function unit — NOT one per planned node. The seeds carry
    // their own entry node, so a unit's identity is its entry rather than a
    // position in the node table.
    for (ordinal, seed) in partition.seeds.iter().enumerate() {
        let id = PredeclaredFunctionId(
            u32::try_from(ordinal)
                .map_err(|_| planner_capacity_error("function unit identity exhausted"))?,
        );
        plane.functions.push(PredeclaredFunction {
            id,
            planned_node: *seed,
            origin: StaticOriginId(seed.0),
            program: SemanticProgramId(seed.0),
        });
    }

    plane.validate(nodes, edges, entries, root_entry, sources, arena)?;
    Ok(plane)
}

/// One exhaustive source/control-to-IR derivation. There is intentionally no
/// wildcard or fallback arm: adding a source or outer control kind must choose
/// one of the six semantic primitives here.
fn opcode_for_source(source: SemanticSourceKind) -> SemanticOpcode {
    match source {
        SemanticSourceKind::Expression(shape) => match shape {
            RuntimeExprShape::Value
            | RuntimeExprShape::Var
            | RuntimeExprShape::DeclarationRef
            | RuntimeExprShape::ImportedDeclarationRef => SemanticOpcode::EvaluateExpression,
            RuntimeExprShape::CheckedJoinSite
            | RuntimeExprShape::CheckedSubcontinuationFrame
            | RuntimeExprShape::CheckedComputationalIHSlots
            | RuntimeExprShape::Let
            | RuntimeExprShape::PrimitiveCall
            | RuntimeExprShape::Construct
            | RuntimeExprShape::Record
            | RuntimeExprShape::Project
            | RuntimeExprShape::Closure
            | RuntimeExprShape::LexicalClosure => SemanticOpcode::TransferValueOrControl,
            RuntimeExprShape::If | RuntimeExprShape::Match => SemanticOpcode::SelectBranchOrCase,
            RuntimeExprShape::CheckedRecursiveInvocation
            | RuntimeExprShape::CheckedComputationalIHInvocation
            | RuntimeExprShape::Call
            | RuntimeExprShape::Effect => SemanticOpcode::InvokeOrResume,
            RuntimeExprShape::Trap => SemanticOpcode::ReturnOrComplete,
            RuntimeExprShape::ComputationalMatch => SemanticOpcode::RunAffineCleanup,
        },
        SemanticSourceKind::Control(transition) => match transition {
            TransitionKind::Evaluate => SemanticOpcode::EvaluateExpression,
            TransitionKind::Sequence => SemanticOpcode::TransferValueOrControl,
            TransitionKind::Branch | TransitionKind::CaseTest => SemanticOpcode::SelectBranchOrCase,
            TransitionKind::ProducerWrapper | TransitionKind::SourceReturnResume => {
                SemanticOpcode::InvokeOrResume
            }
            TransitionKind::Terminal
            | TransitionKind::TrapTerminal
            | TransitionKind::ClosureBody
            | TransitionKind::ProducerTail
            | TransitionKind::CompletedTail => SemanticOpcode::ReturnOrComplete,
        },
    }
}

impl SemanticPlane {
    /// Closed diagnostic/runtime view of the planner-owned carrier identity
    /// namespace. The returned words are the existing packed spans; this
    /// accessor neither interns nor derives an identity.
    pub(super) fn carrier_identity_catalog(
        &self,
    ) -> Result<Vec<(String, u64)>, CraneliftBackendError> {
        let mut catalog = BTreeMap::<u64, String>::new();
        let mut record = |span: DenseRange| -> Result<(), CraneliftBackendError> {
            validate_range(
                span,
                self.names.len(),
                "carrier identity catalog span is outside the closed name arena",
            )?;
            let bytes = plane_slice(&self.names, span, "carrier identity spelling")?;
            let spelling = std::str::from_utf8(bytes)
                .map_err(|_| planner_error("carrier identity spelling is not UTF-8"))?
                .to_string();
            let identity = pack_identity(span)?;
            if catalog
                .insert(identity, spelling.clone())
                .is_some_and(|prior| prior != spelling)
            {
                return Err(planner_error(
                    "one carrier identity names two different spellings",
                ));
            }
            Ok(())
        };
        for atom in &self.operands {
            if matches!(
                atom.kind,
                SemanticAtomKind::ConstructorSymbol
                    | SemanticAtomKind::CaseConstructor
                    | SemanticAtomKind::RecordFieldName
                    | SemanticAtomKind::ProjectField
                    | SemanticAtomKind::ValueConstructor
            ) {
                record(atom.content)?;
            }
        }
        for span in self.synthesized_constructor_roles.values().copied() {
            record(span)?;
        }
        Ok(catalog
            .into_iter()
            .map(|(identity, spelling)| (spelling, identity))
            .collect())
    }

    pub(super) fn install_synthesized_constructor_inventory(
        &mut self,
        identities: BTreeMap<SynthesizedConstructorRole, DenseRange>,
        io_roles: Vec<SynthesizedIoErrorRole>,
    ) {
        self.synthesized_constructor_roles = identities;
        self.synthesized_io_error_roles = io_roles;
    }

    pub(super) fn synthesized_constructor_identity(
        &self,
        role: SynthesizedConstructorRole,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        let span = self
            .synthesized_constructor_roles
            .get(&role)
            .copied()
            .ok_or_else(|| {
                planner_error(format!(
                    "synthesized constructor role {role:?} is absent from the closed inventory"
                ))
            })?;
        validate_range(
            span,
            self.names.len(),
            "synthesized constructor identity is outside the closed name arena",
        )?;
        Ok(ConstructorIdentity(span))
    }

    pub(super) fn synthesized_io_error_roles(&self) -> &[SynthesizedIoErrorRole] {
        &self.synthesized_io_error_roles
    }

    pub(super) fn validate_synthesized_constructor_inventory(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        for role in SynthesizedFixedConstructorRole::ALL {
            self.synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(role))?;
        }
        for role in &self.synthesized_io_error_roles {
            self.synthesized_constructor_identity(SynthesizedConstructorRole::IoError(*role))?;
        }
        let expected = SynthesizedFixedConstructorRole::ALL
            .len()
            .checked_add(self.synthesized_io_error_roles.len())
            .ok_or_else(|| planner_capacity_error("synthesized constructor role count exhausted"))?;
        if self.synthesized_constructor_roles.len() != expected {
            return Err(planner_error(
                "synthesized constructor inventory is not exact for its closed role population",
            ));
        }
        Ok(())
    }

    /// The single record an origin resolves to, with both positional-identity
    /// checks applied.
    ///
    /// ⭐ Extracted so that `child_origin` and the `D1` identity accessors share
    /// **one** descriptor -> program -> record walk. Two copies of this walk
    /// would be two authorities on "the record for this origin", and a
    /// divergence between them is exactly the wrong-occurrence substitution the
    /// origin-vs-index checks below exist to make impossible.
    fn record_for(&self, origin: StaticOriginId) -> Result<&SemanticRecord, CraneliftBackendError> {
        let descriptor = self
            .descriptors
            .get(origin.0 as usize)
            .ok_or_else(|| planner_error("static origin is outside the semantic descriptors"))?;
        if descriptor.origin != origin {
            return Err(planner_error(
                "descriptor origin is not its preallocated positional identity",
            ));
        }
        let program = self
            .programs
            .get(descriptor.program.0 as usize)
            .ok_or_else(|| planner_error("descriptor names an unknown semantic program"))?;
        let [record] = plane_slice(&self.records, program.records, "semantic record")? else {
            return Err(planner_error(
                "semantic program does not hold exactly one record",
            ));
        };
        if record.origin != origin {
            return Err(planner_error(
                "semantic record origin is not its preallocated positional identity",
            ));
        }
        Ok(record)
    }

    /// This occurrence's own non-child semantic atoms, in emission order.
    fn operands_of(
        &self,
        origin: StaticOriginId,
    ) -> Result<&[SemanticOperandElement], CraneliftBackendError> {
        let record = self.record_for(origin)?;
        plane_slice(&self.operands, record.operands, "semantic operand")
    }

    /// The `occurrence`-th atom of `kind` among this origin's own operands.
    ///
    /// ⚠ Selection is by **atom kind**, not by absolute operand position: a
    /// `Match`'s operand run begins with a `MatchDefault` atom and interleaves
    /// `CaseBinders` / `CaseBinder` between the `CaseConstructor`s, so an
    /// absolute index would silently track the binder counts of earlier cases.
    fn named_atom(
        &self,
        origin: StaticOriginId,
        kind: SemanticAtomKind,
        occurrence: usize,
    ) -> Result<&SemanticOperandElement, CraneliftBackendError> {
        self.operands_of(origin)?
            .iter()
            .filter(|atom| atom.kind == kind)
            .nth(occurrence)
            .ok_or_else(|| {
                planner_error(format!(
                    "static origin {origin:?} has no {kind:?} atom at occurrence {occurrence}"
                ))
            })
    }

    /// The canonical identity span of a named atom of `kind`.
    ///
    /// ⭐ **The atom-kind check is `named_atom`'s `filter`, not a second test
    /// here.** Selecting by kind and then re-asserting the kind would be a check
    /// that can never fire, which reads as safety and supplies none. What makes
    /// the returned identity's *type* a consequence of the atom's kind is that
    /// the four callers below each pass a fixed kind and wrap in the newtype
    /// that kind belongs to — there is no path that takes the kind from a
    /// caller's intent.
    ///
    /// ⚠ The span is canonical because `SemanticMaterialArena::intern` is
    /// content-addressed; that, not this function, is what makes equal spellings
    /// equal identities. The bounds check is genuine and *can* fire: it is the
    /// per-access half of the arena-closure invariant `validate` asserts
    /// wholesale, and it fires on a corrupt plane handed straight to an
    /// accessor without a validation pass.
    fn identity_span(
        &self,
        origin: StaticOriginId,
        kind: SemanticAtomKind,
        occurrence: usize,
    ) -> Result<DenseRange, CraneliftBackendError> {
        let atom = self.named_atom(origin, kind, occurrence)?;
        validate_range(
            atom.content,
            self.names.len(),
            "semantic identity atom span is outside its closed name arena",
        )?;
        Ok(atom.content)
    }

    /// The artifact-static constructor identity of case `case_index` of the
    /// `Match` / `ComputationalMatch` occurrence at `origin` (`D1`).
    pub(super) fn case_constructor_identity(
        &self,
        origin: StaticOriginId,
        case_index: usize,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        Ok(ConstructorIdentity(self.identity_span(
            origin,
            SemanticAtomKind::CaseConstructor,
            case_index,
        )?))
    }

    /// The artifact-static constructor identity of the `Construct` occurrence at
    /// `origin` — the **producer** side of the same authority (`D2`).
    pub(super) fn constructor_symbol_identity(
        &self,
        origin: StaticOriginId,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        Ok(ConstructorIdentity(self.identity_span(
            origin,
            SemanticAtomKind::ConstructorSymbol,
            0,
        )?))
    }

    /// The artifact-static field identity selected by the `Project` occurrence
    /// at `origin` (`D1`).
    pub(super) fn project_field_identity(
        &self,
        origin: StaticOriginId,
    ) -> Result<FieldIdentity, CraneliftBackendError> {
        Ok(FieldIdentity(self.identity_span(
            origin,
            SemanticAtomKind::ProjectField,
            0,
        )?))
    }

    /// The artifact-static field identity of field `position` of the `Record`
    /// occurrence at `origin` — the **producer** side of `project_field_identity`.
    pub(super) fn record_field_identity(
        &self,
        origin: StaticOriginId,
        position: usize,
    ) -> Result<FieldIdentity, CraneliftBackendError> {
        Ok(FieldIdentity(self.identity_span(
            origin,
            SemanticAtomKind::RecordFieldName,
            position,
        )?))
    }

    /// The preallocated origin of one **positional** syntax child.
    ///
    /// Child *k* is recovered as child *k* out of the occurrence's own
    /// child-origin range — never by search, shape-matching, pointer, or clone
    /// order. This is the accessor the emitter descends with: knowing an
    /// occurrence's origin, its children's origins are already determined, so
    /// no second identity space is minted for them.
    ///
    /// Every planned origin resolves here: `build_semantic_plane` lays the
    /// descriptors, programs, and records dense over the planned nodes and
    /// indexed by planned-node id, and an origin *is* its node's ordinal. The
    /// only failing lookup is a position past this occurrence's own child count,
    /// which is a compiler bug in the caller's ordinal, not an input condition.
    pub(super) fn child_origin(
        &self,
        parent: StaticOriginId,
        position: usize,
    ) -> Result<StaticOriginId, CraneliftBackendError> {
        self.child_origins(parent)?
            .get(position)
            .copied()
            .ok_or_else(|| planner_error("static origin has no child at that source position"))
    }

    /// Every preallocated positional child of one source occurrence.
    ///
    /// The slice is the same validated range used by [`Self::child_origin`].
    /// Keeping the population projection here lets planner-side structural
    /// traversals consume the closed child inventory without spelling the
    /// `RuntimeExpr` variants a second time.
    pub(super) fn child_origins(
        &self,
        parent: StaticOriginId,
    ) -> Result<&[StaticOriginId], CraneliftBackendError> {
        let record = self.record_for(parent)?;
        plane_slice(
            &self.child_origins,
            record.child_origins,
            "semantic child origin",
        )
    }

    /// The validated function-unit owner of one source occurrence.
    ///
    /// Result-phase planning uses this capability rather than exposing the
    /// descriptor arena: an owner transition is a declared-unit boundary, but
    /// neither the lowering nor the phase planner may derive an owner from an
    /// origin ordinal.
    pub(super) fn owner(
        &self,
        origin: StaticOriginId,
    ) -> Result<SemanticOwner, CraneliftBackendError> {
        let descriptor = self
            .descriptors
            .get(origin.0 as usize)
            .ok_or_else(|| planner_error("static origin is outside the semantic descriptors"))?;
        if descriptor.origin != origin {
            return Err(planner_error(
                "descriptor origin is not its preallocated positional identity",
            ));
        }
        Ok(descriptor.owner)
    }

    pub(super) fn function_owner(
        &self,
        origin: StaticOriginId,
    ) -> Result<Option<PredeclaredFunctionId>, CraneliftBackendError> {
        Ok(match self.owner(origin)? {
            SemanticOwner::Function(function) => Some(function),
            SemanticOwner::Terminal | SemanticOwner::TrapTerminal => None,
        })
    }

    /// Whether a result edge crosses between two distinct function units.
    ///
    /// The closed owner vocabulary stays private to this module; consumers ask
    /// the semantic plane the one question phase propagation needs.
    pub(super) fn crosses_function_owner(
        &self,
        from: StaticOriginId,
        to: StaticOriginId,
    ) -> Result<bool, CraneliftBackendError> {
        Ok(matches!(
            (self.owner(from)?, self.owner(to)?),
            (SemanticOwner::Function(left), SemanticOwner::Function(right)) if left != right
        ))
    }

    /// The function-unit population, the ownership partition, and the edge laws
    /// — each as its own named failure.
    ///
    /// ⛔ Deliberately not one composite check, for the same reason
    /// `validate_source_occurrence_table` is not: a single "ownership is fine"
    /// assertion is discharged by any one of these holding, so the eight
    /// mutations `AC-5` requires would be indistinguishable from each other.
    ///
    /// The partition is **recomputed from the graph** here and compared against
    /// what the plane recorded. That is what makes a corrupted owner field a
    /// planner error rather than a plausible wrong answer.
    ///
    /// The edge laws are checked *on top of* that comparison because they
    /// constrain the **algorithm** and not just the record. ⚠ But they are
    /// **defense in depth behind the overlap check, not the primary detector** —
    /// measured, not assumed: a traversal edited to cross `StaticBody` is caught
    /// by **overlap** first, because the callee's seed gets claimed by the caller
    /// (mutation M1). The `StaticBody` law becomes the *sole* detector only once
    /// the overlap check is **also** disabled (mutation M2).
    ///
    /// ⛔ An earlier revision of this comment said such a traversal "would
    /// produce a self-consistent partition, and only the distinct-unit law
    /// catches it." That was wrong, and wrong in the direction that matters: it
    /// credited this law with work the overlap check is doing, and a reader who
    /// believed it might weaken overlap thinking the edge law still covered them.
    /// **`RT-FNSPLIT-B2F` `D4` — the cross-owner call edges, as caller/callee id
    /// pairs.**
    ///
    /// ⭐ **Derived here because this is where the classification already
    /// lives.** [`Self::validate_function_units`] enforces all four edge laws as
    /// `return Err` arms, so a plane that exists cannot carry a `StaticBody`
    /// edge that fails to cross into a distinct unit's seed. ⇒ This walk re-reads
    /// validated facts; it does not re-decide them, and ⛔ it must never grow an
    /// arm that classifies an edge the validator would have rejected.
    ///
    /// ⛔ **Deliberately kept out of `static_transition.rs`.** Spelling
    /// `SemanticOwner` in a third production file is how a second classification
    /// authority begins, and
    /// `the_owner_classification_has_a_closed_production_naming_inventory`
    /// reddens on exactly that.
    ///
    /// ⛔ Fails closed on an endpoint with no descriptor, and on a `StaticBody`
    /// edge that does not join two function units. ⚠ The latter is unreachable
    /// through the validator — it is here so that a future caller which reaches
    /// this method on an unvalidated plane is refused rather than silently given
    /// a short edge list.
    /// **`RT-DECL-CLOSURE-PORT` `D2a`** — is this `StaticBody` edge the
    /// declaration-owned pair's definition relation rather than an emitted call?
    ///
    /// ⭐ Exposed from here, and only from here, because the answer is a
    /// statement about the **owner classification** — and that classification
    /// has exactly one home. `static_transition.rs` must not spell
    /// `SemanticOwner`; the standing source tripwire reds if it starts.
    pub(super) fn is_declaration_owned_static_body(
        &self,
        edge: &StaticEdge,
    ) -> Result<bool, CraneliftBackendError> {
        if edge.kind != EdgeKind::StaticBody {
            return Ok(false);
        }
        let owner_of = |node: StaticNodeId| -> Result<SemanticOwner, CraneliftBackendError> {
            self.descriptors
                .get(node.0 as usize)
                .map(|descriptor| descriptor.owner)
                .ok_or_else(|| planner_error("call edge endpoint has no semantic descriptor"))
        };
        Ok(owner_of(edge.from)? == owner_of(edge.to)?)
    }

    pub(super) fn static_body_call_edges(
        &self,
        edges: &[StaticEdge],
    ) -> Result<
        Vec<(
            PredeclaredFunctionId,
            PredeclaredFunctionId,
            StaticOriginId,
        )>,
        CraneliftBackendError,
    > {
        let owner_of = |node: StaticNodeId| -> Result<SemanticOwner, CraneliftBackendError> {
            self.descriptors
                .get(node.0 as usize)
                .map(|descriptor| descriptor.owner)
                .ok_or_else(|| planner_error("call edge endpoint has no semantic descriptor"))
        };
        let mut call_edges = Vec::new();
        for edge in edges {
            if edge.kind != EdgeKind::StaticBody {
                continue;
            }
            let (SemanticOwner::Function(caller), SemanticOwner::Function(callee)) =
                (owner_of(edge.from)?, owner_of(edge.to)?)
            else {
                return Err(planner_error(
                    "static body call edge does not join two function units",
                ));
            };
            // ⭐⭐ `RT-DECL-CLOSURE-PORT` `D2a` — the declaration-owned pair is a
            // DEFINITION relation, not an emitted call.
            //
            // Both ends are one unit: the declaration occurrence is the callable
            // unit's ownership, provenance and `D3` signature authority, and the
            // body is what that unit emits. Emitting a call here would
            // reintroduce, from the call side, exactly the phantom `D2a` removes
            // from the unit side — the semantic partition would say one function
            // while the emitted call population said two.
            //
            // ⛔ **`caller == callee` is a sound discriminator only because the
            // edge law upstream already refused every OTHER intra-unit
            // `StaticBody` edge.** An anonymous closure body's edge crosses
            // units and is unaffected; a same-unit edge that is not a
            // declaration-owned pair never reaches this walk. ⇒ This is not a
            // second classification authority, and `D3`'s boundary-layout
            // validation over the relation is untouched.
            if caller == callee {
                continue;
            }
            let callee_origin = self
                .functions
                .get(callee.0 as usize)
                .ok_or_else(|| planner_error("call edge callee has no function descriptor"))?
                .origin;
            call_edges.push((caller, callee, callee_origin));
        }
        Ok(call_edges)
    }

    /// The separately typed call edges from `DeclarationRef` occurrences to
    /// already-owned transparent-declaration scheduling entries.
    pub(super) fn declaration_call_edges(
        &self,
        edges: &[StaticEdge],
    ) -> Result<
        Vec<(
            PredeclaredFunctionId,
            PredeclaredFunctionId,
            StaticOriginId,
            StaticOriginId,
        )>,
        CraneliftBackendError,
    > {
        let owner_of = |node: StaticNodeId| -> Result<SemanticOwner, CraneliftBackendError> {
            self.descriptors
                .get(node.0 as usize)
                .map(|descriptor| descriptor.owner)
                .ok_or_else(|| {
                    planner_error("declaration call endpoint has no semantic descriptor")
                })
        };
        let mut call_edges = Vec::new();
        for edge in edges {
            if edge.kind != EdgeKind::DeclarationCall {
                continue;
            }
            let (SemanticOwner::Function(caller), SemanticOwner::Function(callee)) =
                (owner_of(edge.from)?, owner_of(edge.to)?)
            else {
                return Err(planner_error(
                    "declaration call edge does not join two function units",
                ));
            };
            let callee_origin = self
                .functions
                .get(callee.0 as usize)
                .ok_or_else(|| {
                    planner_error("declaration call callee has no function descriptor")
                })?
                .origin;
            call_edges.push((
                caller,
                callee,
                callee_origin,
                StaticOriginId(edge.from.0),
            ));
        }
        Ok(call_edges)
    }

    pub(super) fn function_for_node(
        &self,
        node: StaticNodeId,
    ) -> Result<PredeclaredFunctionId, CraneliftBackendError> {
        let descriptor = self
            .descriptors
            .get(node.0 as usize)
            .ok_or_else(|| planner_error("root entry has no semantic descriptor"))?;
        match descriptor.owner {
            SemanticOwner::Function(function) => Ok(function),
            SemanticOwner::Terminal | SemanticOwner::TrapTerminal => {
                Err(planner_error("root entry is owned by a shared exit"))
            }
        }
    }

    fn validate_function_units(
        &self,
        nodes: &[StaticNode],
        edges: &[StaticEdge],
        entries: &[StaticNodeId],
        root_entry: Option<StaticNodeId>,
        node_indexed_sources: &[SemanticSourceSeed],
    ) -> Result<(), CraneliftBackendError> {
        let partition = partition_function_units(nodes, edges, entries, root_entry)?;
        let (declaration_owned_body, pairs) =
            declaration_owned_pairs(nodes.len(), edges, entries, root_entry)?;

        // **The unit population, corrected by `D2a`.**
        //
        // ⛔ It was `functions.len() == entries.len() + count(StaticBody edges)`,
        // predicted from the design on 2026-07-25. `D2a` FALSIFIES that equality
        // for the declaration-owned class: a closure-seed transparent
        // declaration contributes its `StaticBody` target and **not** its own
        // scheduling entry, so the two ends of that pair are one function, not
        // two. The subtraction is the whole correction, stated here rather than
        // annotated onto the old claim.
        let static_body_edges = edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::StaticBody)
            .count();
        let expected_units = entries
            .len()
            .checked_add(static_body_edges)
            .ok_or_else(|| planner_capacity_error("function unit count exhausted"))?
            .checked_sub(pairs)
            .ok_or_else(|| planner_error("more declaration-owned pairs than seeded units"))?;
        if self.functions.len() != expected_units || partition.seeds.len() != expected_units {
            return Err(planner_error(
                "function unit population is not the scheduling entries and static body targets",
            ));
        }

        // Dense, positional, and seeded on the node the partition seeded on.
        for (ordinal, function) in self.functions.iter().enumerate() {
            let id = PredeclaredFunctionId(
                u32::try_from(ordinal)
                    .map_err(|_| planner_capacity_error("function unit identity exhausted"))?,
            );
            let seed = partition.seeds[ordinal];
            if function.id != id
                || function.planned_node != seed
                || function.origin != StaticOriginId(seed.0)
                || function.program != SemanticProgramId(seed.0)
            {
                return Err(planner_error(
                    "function unit is not positional for its seed",
                ));
            }
        }

        // AC-2: totality and exclusivity are PINNED, not merely structural. The
        // owner field is one field rather than a list, so "owned by two units" is
        // unrepresentable in the record — but a *wrongly assigned* owner is very
        // representable, which is what this comparison catches.
        if self.descriptors.len() != partition.owners.len() {
            return Err(planner_error(
                "semantic descriptor population is not exact for the ownership partition",
            ));
        }
        let mut terminals = 0usize;
        let mut trap_terminals = 0usize;
        for (position, descriptor) in self.descriptors.iter().enumerate() {
            if descriptor.owner != partition.owners[position] {
                return Err(planner_error(
                    "semantic descriptor owner is not the node's derived function unit",
                ));
            }
            match descriptor.owner {
                SemanticOwner::Function(id) => {
                    if id.0 as usize >= self.functions.len() {
                        return Err(planner_error(
                            "semantic descriptor names an unknown function unit",
                        ));
                    }
                }
                SemanticOwner::Terminal => terminals += 1,
                SemanticOwner::TrapTerminal => trap_terminals += 1,
            }
        }
        // AC-2: the shared-exit population is EXACTLY the two sentinels — not
        // "at least", and not "whichever nodes ended up unowned".
        if terminals != 1 || trap_terminals != 1 {
            return Err(planner_error(
                "shared exit population is not exactly one Terminal and one TrapTerminal",
            ));
        }

        // D3, the edge laws.
        let owner_of = |node: StaticNodeId| -> Result<SemanticOwner, CraneliftBackendError> {
            self.descriptors
                .get(node.0 as usize)
                .map(|descriptor| descriptor.owner)
                .ok_or_else(|| planner_error("ownership edge endpoint has no semantic descriptor"))
        };
        for edge in edges {
            let from = owner_of(edge.from)?;
            let to = owner_of(edge.to)?;
            let SemanticOwner::Function(from_unit) = from else {
                // Sentinels have no outgoing edges (`static_transition.rs:1258`),
                // so an edge leaving one is a graph the planner did not build.
                return Err(planner_error("shared exit has an outgoing transfer edge"));
            };
            if edge.kind == EdgeKind::StaticBody {
                // A StaticBody edge crosses from one unit to a DISTINCT unit,
                // and its target is that unit's seed.
                let SemanticOwner::Function(to_unit) = to else {
                    return Err(planner_error("static body edge targets a shared exit"));
                };
                // ⭐⭐ `RT-DECL-CLOSURE-PORT` `D2a` — the ONE intra-unit
                // `StaticBody` edge, and why it is not a hole in this law.
                //
                // For a closure-seed transparent declaration the pair
                // `(declaration occurrence, body entry)` is **one** function.
                // Its `StaticBody` edge is therefore a **definition/signature
                // relation**, not a cross-unit call: it is what binds the
                // declaration occurrence — the callable unit's ownership,
                // provenance and `D3` signature authority — to the body that
                // unit emits.
                //
                // ⛔ The exemption is NOT "the two ends happen to share a unit".
                // It is granted only where `declaration_owned_pairs` derived the
                // pair, from the declaration occurrence plus its one forward
                // relation. An anonymous closure body's edge, or any other
                // same-unit `StaticBody` edge, still fails closed here.
                let declaration_owned =
                    declaration_owned_body[edge.from.0 as usize] == Some(edge.to);
                if to_unit == from_unit && !declaration_owned {
                    return Err(planner_error(
                        "static body edge does not cross a function unit boundary",
                    ));
                }
                if self.functions[to_unit.0 as usize].planned_node != edge.to {
                    return Err(planner_error(
                        "static body edge target is not its function unit's seed",
                    ));
                }
            } else if edge.kind == EdgeKind::DeclarationCall {
                let SemanticOwner::Function(to_unit) = to else {
                    return Err(planner_error(
                        "declaration call edge targets a shared exit",
                    ));
                };
                // ⭐ `RT-DECL-CLOSURE-PORT` `D4` — the target is a **function
                // unit head**, and there are exactly two classes of those.
                //
                // The ruled seed set is `entries` ∪ every `StaticBody` target,
                // minus every `D2a` declaration-owned pair.
                // Before `D4` a declaration call could only reach the first
                // class, because every declaration reference targeted its own
                // scheduling entry. The selective retarget makes a closure-seed
                // declaration's reference reach the second — its
                // declaration-owned callable unit — so the law is widened to
                // the full head set and no further.
                //
                // ⛔ It is NOT relaxed to "any node in a distinct unit": the
                // seed check below is what keeps a call from landing in the
                // middle of a unit, and it runs on both classes.
                let scheduling_entry = entries.contains(&edge.to);
                // ⚠⚠ **NAMED FOR WHAT IT MEASURES.** This is "the endpoint is a
                // static-body unit head" — the WHOLE such population, anonymous
                // `ClosureBody` units included. It is **not** the
                // callable-declaration discriminator, and must never be
                // described as one: reading it that way would make this file a
                // second, weaker classification authority for a class the ABI
                // plane already decides exactly.
                //
                // ⭐ The layering, stated so it is not collapsed by a later
                // edit: the **semantic plane** establishes that the endpoint is
                // a unit head and therefore that a self-loop is structurally
                // possible; the **ABI plane** —
                // `AbiPlane::validate_declaration_call_targets` — reads the
                // exact `AbiUnitDefinition` and is the sole authority on the
                // callable class. ⛔ Do not reverse-search the semantic graph to
                // duplicate that.
                let static_body_head = edges
                    .iter()
                    .any(|body| body.kind == EdgeKind::StaticBody && body.to == edge.to);
                if !scheduling_entry && !static_body_head {
                    return Err(planner_error(
                        "declaration call edge target is neither a scheduling entry nor a \
                         static body unit head",
                    ));
                }
                if self.functions[to_unit.0 as usize].planned_node != edge.to {
                    return Err(planner_error(
                        "declaration call edge target is not its function unit's seed",
                    ));
                }
                // ⭐⭐ **`D4`: a call to the caller's OWN unit is lawful for the
                // callable-declaration class and for that class only.**
                //
                // A closure-seed declaration that refers to itself does so from
                // inside its own body — which, after the retarget, IS the unit
                // being called. That edge is direct recursion, and it is the
                // one case where a declaration call legitimately does not cross
                // a unit boundary.
                //
                // ⛔ The ban is kept intact for the scheduling-entry class,
                // where it still means what it always meant: a declaration
                // whose reference resolves back into the unit it already sits
                // in has no second unit to call, so the edge would be an
                // intra-unit transfer misfiled as a call.
                //
                // ⚠ **What this test does and does not decide.** Admitting the
                // self-loop here is a *structural* permission keyed on the
                // broad head predicate above; it does not certify that the
                // target is the exact declaration-owned `CallableDeclaration`
                // of the referenced symbol. That certification is
                // `validate_declaration_call_targets`'s, and it is what keeps
                // an anonymous `ClosureBody` self-loop out. ⇒ The plan is
                // fail-closed across the two layers together, never by this
                // line alone.
                if to_unit == from_unit && !static_body_head {
                    return Err(planner_error(
                        "declaration call edge does not cross a function unit boundary",
                    ));
                }
                let source = node_indexed_sources
                    .get(edge.from.0 as usize)
                    .ok_or_else(|| planner_error("declaration call source has no semantic seed"))?;
                if source.source
                    != SemanticSourceKind::Expression(RuntimeExprShape::DeclarationRef)
                {
                    return Err(planner_error(
                        "declaration call edge source is not a DeclarationRef occurrence",
                    ));
                }
            } else {
                // A non-StaticBody edge stays inside one unit, or exits to a
                // shared exit — which lowers as this unit's own return or trap,
                // never as a cross-owner call.
                match to {
                    SemanticOwner::Terminal | SemanticOwner::TrapTerminal => {}
                    SemanticOwner::Function(to_unit) if to_unit == from_unit => {}
                    SemanticOwner::Function(_) => {
                        return Err(planner_error(
                            "transfer edge crosses a function unit boundary without a static body edge",
                        ));
                    }
                }
            }
        }

        // Each top-level scheduling entry has NO incoming static body edge.
        // ⚠ Not "every head except the root": a transparent declaration entry is
        // a top-level seed too, so the root is not the only entry.
        let scheduling_entries = entries.iter().copied().collect::<Vec<_>>();
        for edge in edges {
            if edge.kind == EdgeKind::StaticBody && scheduling_entries.contains(&edge.to) {
                return Err(planner_error(
                    "scheduling entry has an incoming static body edge",
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate(
        &self,
        nodes: &[StaticNode],
        edges: &[StaticEdge],
        entries: &[StaticNodeId],
        root_entry: Option<StaticNodeId>,
        semantic_sources: &[SemanticSourceSeed],
        arena: &SemanticMaterialArena,
    ) -> Result<(), CraneliftBackendError> {
        // `semantic_sources` is recorded in walk order. Position it exactly once
        // before any validator indexes the population by `StaticOriginId`, then
        // reuse that canonical view for every node-indexed check below.
        let node_indexed_sources = positioned_sources(nodes, semantic_sources)?;

        let mut seen_nodes = vec![false; nodes.len()];
        let mut seen_origins = vec![false; nodes.len()];
        for descriptor in &self.descriptors {
            let node_index = descriptor.planned_node.0 as usize;
            let origin_index = descriptor.origin.0 as usize;
            if node_index >= nodes.len() {
                return Err(planner_error(
                    "semantic descriptor names an unknown planned node",
                ));
            }
            if seen_nodes[node_index] {
                return Err(planner_error(
                    "planned node has more than one semantic definition",
                ));
            }
            seen_nodes[node_index] = true;
            if origin_index < seen_origins.len() && seen_origins[origin_index] {
                return Err(planner_error(
                    "semantic hash-consing merged distinct static origins",
                ));
            }
            if origin_index < seen_origins.len() {
                seen_origins[origin_index] = true;
            }
            if descriptor.origin.0 != descriptor.planned_node.0 {
                return Err(planner_error(
                    "descriptor origin is not its preallocated positional identity",
                ));
            }
        }
        if seen_nodes.iter().any(|seen| !seen) {
            return Err(planner_error(
                "planned node lacks exactly one semantic definition",
            ));
        }
        if self.descriptors.len() != nodes.len() {
            return Err(planner_error(
                "semantic descriptor population is not exact for planned nodes",
            ));
        }
        // ⚠ `functions` is deliberately NOT in this list any more. The
        // node-exact arenas stay node-exact — that is what keeps `child_origin`'s
        // one-record-per-program requirement and `B2A-C`'s correspondence
        // working — but the function table is now seed-exact, and asserting it
        // against `nodes.len()` is the alias this node exists to remove.
        if self.programs.len() != nodes.len()
            || self.records.len() != nodes.len()
            || self.capture_layouts.len() != nodes.len()
        {
            return Err(planner_error(
                "semantic program arena contains a post-origin clone",
            ));
        }
        self.validate_function_units(nodes, edges, entries, root_entry, &node_indexed_sources)?;

        let expected_operands =
            node_indexed_sources
                .iter()
                .try_fold(0usize, |total, source| {
                    total
                        .checked_add(source.source_material_elements as usize)
                        .ok_or_else(|| planner_capacity_error("semantic operand count exhausted"))
                })?;
        // D4.4 — one-visit affine bound over the WHOLE material: this
        // occurrence's atoms plus its child references. The budget is unchanged
        // by the atom/child partition, so a superlinear arena still fails here.
        let expected_child_origins =
            node_indexed_sources
                .iter()
                .try_fold(0usize, |total, source| {
                    total
                        .checked_add(source.children.len as usize)
                        .ok_or_else(|| {
                            planner_capacity_error("semantic child origin count exhausted")
                        })
                })?;
        let expected_atoms = expected_operands
            .checked_sub(expected_child_origins)
            .ok_or_else(|| {
                planner_error("semantic child references exceed the source-material budget")
            })?;
        if self.operands.len() != expected_atoms {
            return Err(planner_error(
                "semantic operand arena exceeds the one-visit source-material budget",
            ));
        }
        if self.child_origins.len() != expected_child_origins {
            return Err(planner_error(
                "semantic child-origin arena is not exact for its positional source children",
            ));
        }
        // Atom content is what B2a will decode. A structurally well-formed atom
        // whose span escapes the closed name arena, or whose bytes are not the
        // ones the walk interned, is undecodable material — reject both.
        if self.names != arena.names {
            return Err(planner_error(
                "semantic atom content arena is not the material the source walk interned",
            ));
        }
        for atom in &self.operands {
            validate_range(
                atom.content,
                self.names.len(),
                "semantic atom content range is outside its closed name arena",
            )?;
        }
        // ⭐ `D2`'s shared-identity property, checked rather than trusted.
        //
        // Every identity in the backend is a canonical name span, and "canonical"
        // means exactly this: equal bytes have equal spans. `intern` establishes
        // it, but an `intern` that regressed to an unconditional append would
        // leave every span *in bounds* and every budget *exact* — the checks
        // above all stay green while producer and consumer silently stop sharing
        // an identity. ⛔ So this is asserted here, at the plane, and not
        // delegated to the function that is supposed to maintain it.
        let mut canonical: BTreeMap<&[u8], DenseRange> = BTreeMap::new();
        for atom in &self.operands {
            let bytes = plane_slice(&self.names, atom.content, "semantic atom content")?;
            match canonical.get(bytes) {
                Some(seen) if *seen != atom.content => {
                    return Err(planner_error(
                        "equal semantic name bytes are interned at two different spans, \
                         so one symbol has two identities",
                    ));
                }
                Some(_) => {}
                None => {
                    canonical.insert(bytes, atom.content);
                }
            }
        }
        if self
            .operands
            .len()
            .checked_add(self.child_origins.len())
            .ok_or_else(|| planner_capacity_error("semantic material count exhausted"))?
            != expected_operands
        {
            return Err(planner_error(
                "semantic material does not partition the one-visit source-material budget",
            ));
        }
        let expected_capture_slots =
            node_indexed_sources
                .iter()
                .try_fold(0usize, |total, source| {
                    total
                        .checked_add(source.capture_slots as usize)
                        .ok_or_else(|| planner_capacity_error("capture slot count exhausted"))
                })?;
        if self.capture_slots.len() != expected_capture_slots {
            return Err(planner_error(
                "capture layout does not flatten each source capture exactly once",
            ));
        }
        if self.ruled_children.len() != edges.len() {
            return Err(planner_error(
                "semantic child arena is not exact for body-free transfer edges",
            ));
        }

        let mut expected_children = vec![Vec::new(); nodes.len()];
        for edge in edges {
            let children = expected_children
                .get_mut(edge.from.0 as usize)
                .ok_or_else(|| planner_error("body-free transfer edge has an unknown source"))?;
            children.push((edge.id, edge.to));
        }
        let mut seen_edges = vec![false; edges.len()];
        for position in 0..nodes.len() {
            let node = StaticNodeId(position as u32);
            let descriptor = self.descriptors[position];
            let program = self.programs[position];
            let record = self.records[position];
            let layout = self.capture_layouts[position];
            let source = node_indexed_sources[position];
            if descriptor.planned_node != node
                || descriptor.origin != source.origin
                || descriptor.program != SemanticProgramId(node.0)
                || descriptor.capture_layout != CaptureLayoutId(node.0)
                || program.id != SemanticProgramId(node.0)
                || layout.id != CaptureLayoutId(node.0)
            {
                return Err(planner_error(
                    "node, descriptor, program, and capture layout are not positional",
                ));
            }
            if program.records.len != 1
                || program.records.start as usize != position
                || record.origin != descriptor.origin
                || record.opcode != opcode_for_source(source.source)
            {
                return Err(planner_error(
                    "semantic program is not the exhaustive lowering of its source",
                ));
            }
            validate_range(
                record.operands,
                self.operands.len(),
                "semantic operand range is outside its closed arena",
            )?;
            // D4.2 — the record's shape/opcode and its operand range agree with
            // the occurrence: exactly this occurrence's non-child atom count.
            if record.operands.len != source.material.len
                || record
                    .operands
                    .len
                    .checked_add(record.child_origins.len)
                    .ok_or_else(|| planner_capacity_error("semantic material count exhausted"))?
                    != source.source_material_elements
            {
                return Err(planner_error(
                    "semantic record does not own its exact source-material range",
                ));
            }
            // D4.1 — one canonical material record per origin, and it carries
            // THIS occurrence's atoms. Equal-shaped occurrences agree on shape
            // and counts, so only content comparison discriminates them.
            let record_atoms = plane_slice(
                &self.operands,
                record.operands,
                "semantic operand range is outside its closed arena",
            )?;
            let expected_atoms = arena_slice(&arena.atoms, source.material, "semantic operand")?;
            if record_atoms != expected_atoms {
                return Err(planner_error(
                    "semantic material record is not occurrence-exact for its origin",
                ));
            }
            // D4.3 — child-origin range is in bounds AND occurrence-exact:
            // positional child k is this occurrence's syntax child k.
            validate_range(
                record.child_origins,
                self.child_origins.len(),
                "semantic child-origin range is outside its closed arena",
            )?;
            if record.child_origins.len != source.children.len {
                return Err(planner_error(
                    "semantic record does not own its exact positional child-origin range",
                ));
            }
            let record_child_origins = plane_slice(
                &self.child_origins,
                record.child_origins,
                "semantic child-origin range is outside its closed arena",
            )?;
            let expected_child_origins =
                arena_slice(&arena.child_origins, source.children, "semantic child origin")?;
            if record_child_origins != expected_child_origins {
                return Err(planner_error(
                    "semantic child origins are not occurrence-exact for their source positions",
                ));
            }
            for child in record_child_origins {
                if child.0 as usize >= nodes.len() {
                    return Err(planner_error(
                        "semantic child origin is outside the planned occurrences",
                    ));
                }
            }
            validate_range(
                layout.slots,
                self.capture_slots.len(),
                "capture slot range is outside its closed arena",
            )?;
            if layout.slots.len != source.capture_slots {
                return Err(planner_error(
                    "capture layout does not match its source occurrence",
                ));
            }
            validate_range(
                descriptor.ruled_children,
                self.ruled_children.len(),
                "ruled child range is outside its closed arena",
            )?;
            let start = descriptor.ruled_children.start as usize;
            let end = descriptor
                .ruled_children
                .end()
                .expect("validated range end");
            let actual = self.ruled_children[start..end]
                .iter()
                .map(|child| (child.edge, child.node))
                .collect::<Vec<_>>();
            if actual != expected_children[position] {
                return Err(planner_error(
                    "descriptor ruled children are not exact for its body-free edges",
                ));
            }
            for child in &self.ruled_children[start..end] {
                let edge_index = child.edge.0 as usize;
                if edge_index >= seen_edges.len() || seen_edges[edge_index] {
                    return Err(planner_error(
                        "body-free transfer edge is owned by more than one descriptor",
                    ));
                }
                seen_edges[edge_index] = true;
            }
        }
        if seen_edges.iter().any(|seen| !seen) {
            return Err(planner_error(
                "body-free transfer edge lacks its ruled source descriptor",
            ));
        }
        Ok(())
    }

    /// Every out-of-line material element the plane holds. The atom/child-origin
    /// partition is a refinement of one budget, so this total is exactly the
    /// occurrence material plus captures and transfer edges.
    #[cfg(test)]
    pub(super) fn all_out_of_line_operand_elements(&self) -> usize {
        self.operands.len()
            + self.child_origins.len()
            + self.capture_slots.len()
            + self.ruled_children.len()
    }
}

fn arena_slice<'a, T>(
    arena: &'a [T],
    range: DenseRange,
    what: &'static str,
) -> Result<&'a [T], CraneliftBackendError> {
    let start = range.start as usize;
    let end = range
        .end()
        .ok_or_else(|| planner_capacity_error(format!("{what} range exhausted")))?;
    arena
        .get(start..end)
        .ok_or_else(|| planner_error("semantic material range is outside its closed arena"))
}

fn plane_slice<'a, T>(
    arena: &'a [T],
    range: DenseRange,
    error: &'static str,
) -> Result<&'a [T], CraneliftBackendError> {
    let start = range.start as usize;
    let end = range.end().ok_or_else(|| planner_error(error))?;
    arena.get(start..end).ok_or_else(|| planner_error(error))
}

/// Lays the planner's walk-ordered source seeds out **positionally by origin**.
///
/// ⚠ `semantic_sources` is pushed in **walk order**, not by node id — which is
/// exactly why this function exists and why `build_semantic_plane` calls it
/// before reading a seed by position. `pub(super)` so the `B2R` ABI plane reuses
/// this one definition rather than re-deriving the positioning: two planes that
/// disagree about "the seed for this origin" is a defect neither would detect.
pub(super) fn positioned_sources(
    nodes: &[StaticNode],
    sources: &[SemanticSourceSeed],
) -> Result<Vec<SemanticSourceSeed>, CraneliftBackendError> {
    let mut positioned = vec![None; nodes.len()];
    for source in sources {
        if source.origin.0 != source.planned_node.0 {
            return Err(planner_error(
                "semantic source origin is not its preallocated positional identity",
            ));
        }
        let slot = positioned
            .get_mut(source.planned_node.0 as usize)
            .ok_or_else(|| planner_error("semantic source names an unknown planned node"))?;
        if slot.replace(*source).is_some() {
            return Err(planner_error(
                "planned node has more than one semantic source",
            ));
        }
    }
    positioned
        .into_iter()
        .map(|source| source.ok_or_else(|| planner_error("planned node lacks its semantic source")))
        .collect()
}

fn validate_range(
    range: DenseRange,
    arena_len: usize,
    error: &'static str,
) -> Result<(), CraneliftBackendError> {
    if range.end().is_none_or(|end| end > arena_len) {
        return Err(planner_error(error));
    }
    Ok(())
}

fn checked_len(len: usize) -> Result<u32, CraneliftBackendError> {
    u32::try_from(len).map_err(|_| planner_capacity_error("semantic source material exhausted"))
}

fn add_material(total: &mut usize, amount: usize) -> Result<(), CraneliftBackendError> {
    *total = total
        .checked_add(amount)
        .ok_or_else(|| planner_capacity_error("semantic source material exhausted"))?;
    Ok(())
}

fn add_material_sum(total: &mut usize, amounts: &[usize]) -> Result<(), CraneliftBackendError> {
    for amount in amounts {
        add_material(total, *amount)?;
    }
    Ok(())
}

fn runtime_value_material_elements(value: &RuntimeValue) -> Result<usize, CraneliftBackendError> {
    let mut total = 0usize;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        add_material(&mut total, 1)?;
        match value {
            RuntimeValue::Bool(_) | RuntimeValue::Int(_) | RuntimeValue::Unknown => {}
            RuntimeValue::Bytes(bytes) => add_material(&mut total, bytes.len())?,
            RuntimeValue::String(value) => add_material(&mut total, value.len())?,
            RuntimeValue::Constructor { args, .. } => pending.extend(args),
            RuntimeValue::Record { fields } => {
                pending.extend(fields.iter().map(|(_, value)| value))
            }
            RuntimeValue::ClosureRef { captured, .. } => pending.extend(captured),
        }
    }
    Ok(total)
}

/// Emits one occurrence's **non-child** atoms, in source position order.
///
/// There is intentionally no wildcard arm: a new `RuntimeExpr` shape must state
/// its own atoms here. A shape whose material is entirely syntax children (`Let`,
/// `If`, `Call`) correctly emits none — its positions live in the child-origin
/// range, and emitting a placeholder for them is what B1 did wrong.
fn emit_expression_atoms(
    expr: &RuntimeExpr,
    arena: &mut SemanticMaterialArena,
) -> Result<(), CraneliftBackendError> {
    match expr {
        RuntimeExpr::CheckedJoinSite { site_id, .. } => {
            arena.push_numeric(SemanticAtomKind::CheckedSiteId, *site_id)?;
        }
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, .. } => {
            arena.push_numeric(SemanticAtomKind::CheckedFrameId, *frame_id)?;
        }
        RuntimeExpr::CheckedRecursiveInvocation {
            call_template_id,
            checked_occurrence_path,
            ..
        }
        | RuntimeExpr::CheckedComputationalIHInvocation {
            call_template_id,
            checked_occurrence_path,
            ..
        } => {
            arena.push_numeric(SemanticAtomKind::CallTemplateId, *call_template_id)?;
            for step in checked_occurrence_path {
                arena.push_numeric(SemanticAtomKind::OccurrencePathStep, *step)?;
            }
        }
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            ..
        } => {
            for id in slot_template_ids {
                arena.push_numeric(SemanticAtomKind::SlotTemplateId, *id)?;
            }
            for path in checked_occurrence_paths {
                arena.push_numeric(
                    SemanticAtomKind::OccurrencePathLen,
                    checked_u64(path.len())?,
                )?;
                for step in path {
                    arena.push_numeric(SemanticAtomKind::OccurrencePathStep, *step)?;
                }
            }
        }
        RuntimeExpr::Value(value) => emit_value_atoms(value, arena)?,
        RuntimeExpr::Var(index) => {
            arena.push_numeric(SemanticAtomKind::LocalIndex, u64::from(*index))?;
        }
        // `Let`, `If` and `Call` are entirely positional: value/body,
        // scrutinee/then/else, and callee/args are syntax children.
        RuntimeExpr::Let { .. } | RuntimeExpr::If { .. } | RuntimeExpr::Call { .. } => {}
        RuntimeExpr::PrimitiveCall { primitive, .. } => {
            // One budgeted element, widened content: the whole primitive, not
            // just its symbol.
            let descriptor = primitive_descriptor_bytes(primitive)?;
            let span = arena.intern(&descriptor)?;
            arena.push_atom(SemanticAtomKind::PrimitiveDescriptor, span, 0)?;
        }
        RuntimeExpr::Construct { constructor, .. } => {
            arena.push_named(SemanticAtomKind::ConstructorSymbol, constructor, 0)?;
        }
        RuntimeExpr::Match { cases, default, .. } => {
            emit_trap_default(default, arena)?;
            for case in cases {
                arena.push_named(SemanticAtomKind::CaseConstructor, &case.constructor, 0)?;
                arena.push_numeric(SemanticAtomKind::CaseBinders, checked_u64(case.binders)?)?;
                for binder in 0..case.binders {
                    arena.push_numeric(SemanticAtomKind::CaseBinder, checked_u64(binder)?)?;
                }
            }
        }
        RuntimeExpr::ComputationalMatch { cases, default, .. } => {
            emit_trap_default(default, arena)?;
            for case in cases {
                arena.push_named(SemanticAtomKind::CaseConstructor, &case.constructor, 0)?;
                arena.push_numeric(
                    SemanticAtomKind::CaseBinders,
                    checked_u64(case.argument_binders)?,
                )?;
                for binder in 0..case.argument_binders {
                    arena.push_numeric(SemanticAtomKind::CaseBinder, checked_u64(binder)?)?;
                }
                for position in &case.recursive_positions {
                    arena.push_numeric(
                        SemanticAtomKind::CaseRecursivePosition,
                        checked_u64(*position)?,
                    )?;
                }
            }
        }
        RuntimeExpr::Record { fields } => {
            for (name, _) in fields {
                arena.push_named(SemanticAtomKind::RecordFieldName, name, 0)?;
            }
        }
        RuntimeExpr::Project { field, .. } => {
            arena.push_named(SemanticAtomKind::ProjectField, field, 0)?;
        }
        RuntimeExpr::Closure {
            captures, params, ..
        } => {
            for capture in captures {
                arena.push_named(SemanticAtomKind::CaptureSymbol, capture, 0)?;
            }
            for param in params {
                arena.push_named(SemanticAtomKind::ParamName, param, 0)?;
            }
        }
        // A lexical closure's captures are evaluated, so they are syntax
        // children; only its parameter names are atoms.
        RuntimeExpr::LexicalClosure { params, .. } => {
            for param in params {
                arena.push_named(SemanticAtomKind::ParamName, param, 0)?;
            }
        }
        RuntimeExpr::DeclarationRef { symbol } => {
            arena.push_named(SemanticAtomKind::DeclarationSymbol, symbol, 0)?;
        }
        RuntimeExpr::ImportedDeclarationRef {
            symbol,
            dependency,
            dependency_semantic_hash,
        } => {
            arena.push_named(SemanticAtomKind::DeclarationSymbol, symbol, 0)?;
            arena.push_named(SemanticAtomKind::DependencySymbol, dependency, 0)?;
            arena.push_named(
                SemanticAtomKind::DependencyHash,
                dependency_semantic_hash,
                0,
            )?;
        }
        RuntimeExpr::Effect {
            family,
            operation,
            capability,
            ..
        } => {
            // The capability, when present, is child 0; record its presence so
            // the positional child range stays interpretable.
            arena.push_named(
                SemanticAtomKind::EffectFamily,
                family,
                u64::from(capability.is_some()),
            )?;
            arena.push_numeric(SemanticAtomKind::EffectOperation, *operation as u64)?;
        }
        RuntimeExpr::Trap(trap) => {
            arena.push_numeric(SemanticAtomKind::TrapCode, trap_code_ordinal(&trap.code))?;
            arena.push_named(SemanticAtomKind::TrapMessage, &trap.message, 0)?;
        }
    }
    Ok(())
}

/// Length-prefixed field, so a concatenation of fields is injective: `"ab"+"c"`
/// and `"a"+"bc"` encode differently.
fn push_encoded_field(bytes: &mut Vec<u8>, value: &str) -> Result<(), CraneliftBackendError> {
    bytes.extend_from_slice(&checked_u32(value.len())?.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
    Ok(())
}

/// Injective tagged encoding of one complete `RuntimePrimitive`: the
/// length-prefixed symbol, an explicit `RuntimePartiality` variant tag, and
/// every field of that variant.
///
/// ⛔ Deliberately hand-written and exhaustive with no wildcard arm: a new
/// `RuntimePartiality` variant must choose its own tag and fields here. It is
/// **not** derived from `Debug`, a hash, a pointer, or clone order, and it costs
/// no extra material element — the primitive's single budgeted atom simply
/// carries wider content.
fn primitive_descriptor_bytes(
    primitive: &RuntimePrimitive,
) -> Result<Vec<u8>, CraneliftBackendError> {
    let mut bytes = Vec::new();
    push_encoded_field(&mut bytes, &primitive.symbol)?;
    match &primitive.partiality {
        RuntimePartiality::Total => bytes.push(0),
        RuntimePartiality::SafeOption {
            none,
            some,
            obligation,
        } => {
            bytes.push(1);
            push_encoded_field(&mut bytes, none)?;
            push_encoded_field(&mut bytes, some)?;
            match obligation {
                Some(obligation) => {
                    bytes.push(1);
                    push_encoded_field(&mut bytes, obligation)?;
                }
                None => bytes.push(0),
            }
        }
        RuntimePartiality::SafeResult { err, ok, error } => {
            bytes.push(2);
            push_encoded_field(&mut bytes, err)?;
            push_encoded_field(&mut bytes, ok)?;
            push_encoded_field(&mut bytes, error)?;
        }
        RuntimePartiality::CheckedTrap { obligation } => {
            bytes.push(3);
            push_encoded_field(&mut bytes, obligation)?;
        }
        RuntimePartiality::TrustedTrap { assumption } => {
            bytes.push(4);
            push_encoded_field(&mut bytes, assumption)?;
        }
    }
    Ok(bytes)
}

/// One eliminator's default trap collapses to a single atom: its code, with the
/// message interned out of line.
fn emit_trap_default(
    trap: &RuntimeTrap,
    arena: &mut SemanticMaterialArena,
) -> Result<(), CraneliftBackendError> {
    let span = arena.intern(trap.message.as_bytes())?;
    arena.push_atom(
        SemanticAtomKind::MatchDefault,
        span,
        trap_code_ordinal(&trap.code),
    )
}

fn trap_code_ordinal(code: &RuntimeTrapCode) -> u64 {
    match code {
        RuntimeTrapCode::UnsupportedErasure => 0,
        RuntimeTrapCode::UnsupportedPrimitivePartiality => 1,
        RuntimeTrapCode::MissingRuntimeMetadata => 2,
        RuntimeTrapCode::PatternMatchFailure => 3,
        RuntimeTrapCode::ExplicitTrap => 4,
    }
}

/// Flattens one `RuntimeValue` into atoms in source pre-order, emitting exactly
/// one atom per element the material budget counts.
fn emit_value_atoms(
    value: &RuntimeValue,
    arena: &mut SemanticMaterialArena,
) -> Result<(), CraneliftBackendError> {
    match value {
        RuntimeValue::Bool(flag) => arena.push_numeric(SemanticAtomKind::ValueBool, u64::from(*flag)),
        RuntimeValue::Int(int) => match int {
            RuntimeIntV1::Small(value) => {
                arena.push_numeric(SemanticAtomKind::ValueIntSmall, *value as u64)
            }
            // A big integer is one budgeted element, so its sign and limbs are
            // interned out of line rather than spread over extra atoms.
            RuntimeIntV1::Big { sign, limbs } => {
                let mut bytes = Vec::with_capacity(1 + limbs.len() * 8);
                bytes.push(match sign {
                    Sign::NonNegative => 0,
                    Sign::Negative => 1,
                });
                for limb in limbs {
                    bytes.extend_from_slice(&limb.to_le_bytes());
                }
                let span = arena.intern(&bytes)?;
                arena.push_atom(SemanticAtomKind::ValueIntBig, span, checked_u64(limbs.len())?)
            }
        },
        RuntimeValue::Bytes(bytes) => {
            arena.push_numeric(SemanticAtomKind::ValueBytes, checked_u64(bytes.len())?)?;
            for byte in bytes {
                arena.push_numeric(SemanticAtomKind::ByteLiteral, u64::from(*byte))?;
            }
            Ok(())
        }
        RuntimeValue::String(text) => {
            arena.push_numeric(SemanticAtomKind::ValueString, checked_u64(text.len())?)?;
            for byte in text.as_bytes() {
                arena.push_numeric(SemanticAtomKind::ByteLiteral, u64::from(*byte))?;
            }
            Ok(())
        }
        RuntimeValue::Constructor { constructor, args } => {
            arena.push_named(
                SemanticAtomKind::ValueConstructor,
                constructor,
                checked_u64(args.len())?,
            )?;
            for arg in args {
                emit_value_atoms(arg, arena)?;
            }
            Ok(())
        }
        RuntimeValue::Record { fields } => {
            // Field names are length-prefixed so the concatenation is injective:
            // `["ab","c"]` and `["a","bc"]` intern to different spans.
            let mut names = Vec::new();
            for (name, _) in fields {
                names.extend_from_slice(&checked_u32(name.len())?.to_le_bytes());
                names.extend_from_slice(name.as_bytes());
            }
            let span = arena.intern(&names)?;
            arena.push_atom(
                SemanticAtomKind::ValueRecord,
                span,
                checked_u64(fields.len())?,
            )?;
            for (_, field) in fields {
                emit_value_atoms(field, arena)?;
            }
            Ok(())
        }
        RuntimeValue::ClosureRef { symbol, captured } => {
            arena.push_named(
                SemanticAtomKind::ValueClosureRef,
                symbol,
                checked_u64(captured.len())?,
            )?;
            for capture in captured {
                emit_value_atoms(capture, arena)?;
            }
            Ok(())
        }
        RuntimeValue::Unknown => arena.push_numeric(SemanticAtomKind::ValueUnknown, 0),
    }
}

fn checked_u64(value: usize) -> Result<u64, CraneliftBackendError> {
    u64::try_from(value).map_err(|_| planner_capacity_error("semantic source material exhausted"))
}

fn checked_u32(value: usize) -> Result<u32, CraneliftBackendError> {
    u32::try_from(value).map_err(|_| planner_capacity_error("semantic source material exhausted"))
}

fn source_material_elements(expr: &RuntimeExpr) -> Result<u32, CraneliftBackendError> {
    let mut total = 0usize;
    match expr {
        RuntimeExpr::CheckedJoinSite { .. } | RuntimeExpr::CheckedSubcontinuationFrame { .. } => {
            add_material(&mut total, 2)?
        }
        RuntimeExpr::CheckedRecursiveInvocation {
            checked_occurrence_path,
            ..
        }
        | RuntimeExpr::CheckedComputationalIHInvocation {
            checked_occurrence_path,
            ..
        } => add_material_sum(&mut total, &[2, checked_occurrence_path.len()])?,
        RuntimeExpr::CheckedComputationalIHSlots {
            slot_template_ids,
            checked_occurrence_paths,
            ..
        } => {
            add_material_sum(&mut total, &[1, slot_template_ids.len()])?;
            add_material(&mut total, checked_occurrence_paths.len())?;
            for path in checked_occurrence_paths {
                add_material(&mut total, path.len())?;
            }
        }
        RuntimeExpr::Value(value) => {
            total = runtime_value_material_elements(value)?;
        }
        RuntimeExpr::Var(_) => add_material(&mut total, 1)?,
        RuntimeExpr::Let { .. } => add_material(&mut total, 2)?,
        RuntimeExpr::If { .. } => add_material(&mut total, 3)?,
        RuntimeExpr::PrimitiveCall { args, .. } | RuntimeExpr::Construct { args, .. } => {
            add_material_sum(&mut total, &[1, args.len()])?;
        }
        RuntimeExpr::Match { cases, .. } => {
            add_material(&mut total, 2)?;
            for case in cases {
                add_material_sum(&mut total, &[3, case.binders])?;
            }
        }
        RuntimeExpr::ComputationalMatch { cases, .. } => {
            add_material(&mut total, 2)?;
            for case in cases {
                add_material_sum(
                    &mut total,
                    &[3, case.argument_binders, case.recursive_positions.len()],
                )?;
            }
        }
        RuntimeExpr::Record { fields } => {
            add_material(&mut total, fields.len())?;
            add_material(&mut total, fields.len())?;
        }
        RuntimeExpr::Project { .. } => add_material(&mut total, 2)?,
        RuntimeExpr::Closure {
            captures, params, ..
        } => add_material_sum(&mut total, &[1, captures.len(), params.len()])?,
        RuntimeExpr::LexicalClosure {
            captures, params, ..
        } => add_material_sum(&mut total, &[1, captures.len(), params.len()])?,
        RuntimeExpr::DeclarationRef { .. } => add_material(&mut total, 1)?,
        RuntimeExpr::ImportedDeclarationRef { .. } => add_material(&mut total, 3)?,
        RuntimeExpr::Call { args, .. } => {
            add_material_sum(&mut total, &[1, args.len()])?;
        }
        RuntimeExpr::Effect {
            capability, args, ..
        } => add_material_sum(
            &mut total,
            &[2, usize::from(capability.is_some()), args.len()],
        )?,
        RuntimeExpr::Trap(_) => add_material(&mut total, 2)?,
    }
    checked_len(total)
}
