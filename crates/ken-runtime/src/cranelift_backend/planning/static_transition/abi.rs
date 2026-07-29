//! The representation and call-ABI contract for values crossing a generated
//! function boundary — declared, validated, and **inert**.
//!
//! `RT-FNSPLIT-B2O` landed the authority this plane consumes: the validated
//! `SemanticOwner` partition. This module attaches a **frame layout** to each
//! `PredeclaredFunction` in that partition.
//!
//! ⛔ **The population is the owner partition, never a source-text census.** The
//! authority for "what is a function unit" is the occurrence's
//! `StaticOriginId`, its validated `SemanticOwner`, and the planned edge kind —
//! never a Rust signature, name, visibility, or file. A pin over this module
//! that reddens because a Rust method was renamed, wrapped, made private, or
//! moved between files is measuring source topology and reporting success.
//!
//! ⛔ **Inert.** Nothing here emits. There is no `FunctionBuilder`, no
//! `define_function`, no call edge, no dispatch edge, no encoder and no decoder.
//! `RT-FNSPLIT-B2F` performs the atomic switch-over; this node only makes the
//! contract expressible and checkable before it does.

use super::semantic_ir::{
    positioned_sources, DenseRange, PredeclaredFunctionId, RuntimeExprShape, SemanticAtomKind,
    SemanticOperandElement, SemanticOwner,
};
use super::{
    planner_capacity_error, planner_error, unsupported, CraneliftBackendError, EdgeKind,
    SemanticPlane, SemanticSourceKind, SemanticSourceSeed, StaticEdge, StaticEdgeId, StaticNode,
    StaticNodeId, StaticOriginId, TransitionKind,
};

/// The exclusive end of a dense range, with its overflow named.
///
/// ⚠ `DenseRange::end` is `semantic_ir`-private on purpose, so this plane
/// computes its own rather than widening that surface for a convenience.
fn range_end(range: DenseRange) -> Result<usize, CraneliftBackendError> {
    (range.start as usize)
        .checked_add(range.len as usize)
        .ok_or_else(|| planner_capacity_error("abi dense range end exhausted"))
}

/// ⭐ **The closed carrier language.**
///
/// Every value that crosses a generated-function boundary travels in exactly
/// one of these. The enum is exhaustive and has no wildcard consumer: adding a
/// carrier must choose a width, an alignment, and an ownership mode explicitly,
/// so a new carrier cannot inherit another's contract by omission.
///
/// ⚠ **Why these are carriers and not derived types, stated honestly.** This
/// plane records `ParamName` and `CaptureSymbol` atoms — *names*, not types.
/// No per-slot static type is derivable from it. The frame therefore permits a
/// **closed handle/tag carrier** rather than a derived type lattice, which is
/// the sanctioned answer where a layout cannot be derived statically. Per-origin
/// variation is real and is carried by **arity and provenance mix**, not by
/// per-slot typing.
///
/// ⛔ **"Fixed frame" does not mean equal byte size across origins.** It means
/// one closed layout language and one common control/store/result/trap
/// convention, which is what this enum and `AbiFrameHeader` together are.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum AbiCarrier {
    /// One machine word holding a Ken value under this frame's ownership rules.
    /// Chosen for declared parameters and for **lexical** captures, whose static
    /// type is not derivable from this plane.
    ValueWord,
    /// ⭐ **The single fixed carrier for a SEED capture**, able to represent the
    /// entire permitted `RuntimeGroundValue` family — `Bool`, `Int`, `Bytes`,
    /// `String`, `Constructor`, `Record` — **without inspecting which variant a
    /// particular JIT-time value holds.**
    ///
    /// ⛔ This is the `C2` constraint made structural. The seed provenance's
    /// layout is a function of *provenance*, never of the value: the builder
    /// below cannot inspect a value because neither `SemanticPlane` nor
    /// `SemanticSourceSeed` contains one.
    GroundValueCarrier,
    /// The activation's single result word.
    ResultWord,
    /// The activation-frame control word: which normal successor the activation
    /// resumed into.
    ControlWord,
    /// The activation's trap word.
    TrapWord,
    /// A handle into the persistent store.
    StoreHandle,
}

impl AbiCarrier {
    /// Declared width, in bytes.
    ///
    /// ⛔ Exhaustive with no `_ =>` arm: a new carrier is a compile error here
    /// rather than a silent inheritance of some other carrier's width.
    const fn width_bytes(self) -> u16 {
        match self {
            Self::ValueWord
            | Self::GroundValueCarrier
            | Self::ResultWord
            | Self::ControlWord
            | Self::TrapWord
            | Self::StoreHandle => 8,
        }
    }

    /// Declared alignment, in bytes.
    ///
    /// ⭐ **Widened to the backend by `RT-FNSPLIT-B2F` `D3`, and the reason is
    /// the same one that makes this a declaration rather than a constant:** the
    /// artifact-static seed material an emitter mints must be aligned for the
    /// carrier that addresses it, and an emitter that hard-codes `8` instead of
    /// reading this would keep working right up until a carrier's declared
    /// alignment changed — at which point the material and the slot addressing
    /// it would disagree with nothing going red.
    ///
    /// ⛔ Widened as a **reader**, not as a setter: it is a `const fn` over a
    /// closed enum with no parameter, so a caller can learn a carrier's
    /// alignment and cannot choose one.
    pub(in crate::cranelift_backend) const fn align_bytes(self) -> u16 {
        match self {
            Self::ValueWord
            | Self::GroundValueCarrier
            | Self::ResultWord
            | Self::ControlWord
            | Self::TrapWord
            | Self::StoreHandle => 8,
        }
    }

    /// **`D4` — the ownership rule this carrier declares.**
    ///
    /// ⛔ An opaque pointer without a stated rule does not discharge the
    /// prerequisite, so every carrier answers here and the match is exhaustive.
    ///
    /// ⚠ A **borrow is only meaningful against an owner that outlives the
    /// borrower**, so this answer is incomplete on its own: read it together
    /// with `storage_owner`, which names who that owner is.
    const fn ownership(self) -> AbiOwnership {
        match self {
            // A parameter or lexical capture arrives owned by the frame for the
            // activation's extent and is reclaimed when the activation ends.
            Self::ValueWord => AbiOwnership::OwnedByFrame,
            // Borrowed from durable artifact-static material — see
            // `storage_owner`, which is where the corrected premise lives.
            Self::GroundValueCarrier => AbiOwnership::BorrowedForActivation,
            // A result leaves the callee for the caller at return.
            Self::ResultWord => AbiOwnership::TransferredToCaller,
            // Control and trap words are frame-local scalars with no reclamation
            // obligation beyond the frame itself.
            Self::ControlWord | Self::TrapWord => AbiOwnership::OwnedByFrame,
            // The persistent store outlives every activation; a frame never
            // reclaims it.
            Self::StoreHandle => AbiOwnership::BorrowedForActivation,
        }
    }

    /// **`D4` — WHO owns the storage this carrier names.**
    ///
    /// ⛔ **This dimension exists because an earlier revision of this file got
    /// the seed carrier's premise factually wrong**, and the error was exactly
    /// the kind a stated-but-unnamed owner hides. It said the seed carrier is
    /// *"minted from the seed environment, which outlives every activation
    /// reading it."* **It does not, and it cannot:**
    ///
    /// - `Lowering<'a>` holds `seed_env: &'a NativeSeedEnvironment`
    ///   (`lowering/mod.rs:267`) — a borrow that exists only for the duration of
    ///   **compilation**;
    /// - `CompiledModule<M>` has **no lifetime parameter** and takes only owned
    ///   data (`lowering/mod.rs:281-285`), so nothing borrowed can be stored in
    ///   it — the compiler rejects it, and
    ///   `escaping_a_source_borrow_into_the_compiled_artifact_does_not_typecheck`
    ///   pins precisely that.
    ///
    /// ⇒ **A runtime activation cannot borrow the seed environment.** An ABI
    /// that says it can is describing a program that cannot be written, and
    /// `B2F` would have inherited that as its calling convention.
    ///
    /// The corrected contract: a seed capture borrows **artifact-static**
    /// material — minted *before* execution begins and therefore outliving every
    /// activation. ⛔ **Minting that material is `B2F`'s work and is deliberately
    /// absent here**: this node declares the durable owner, it does not
    /// materialize anything. No encoder, no decoder, no second emission
    /// authority.
    const fn storage_owner(self) -> AbiStorageOwner {
        match self {
            // Parameters and lexical captures live in the activation's own frame
            // once the boundary transfer completes.
            Self::ValueWord | Self::ResultWord | Self::ControlWord | Self::TrapWord => {
                AbiStorageOwner::ActivationFrame
            }
            // ⭐ The corrected owner. NOT the seed environment.
            Self::GroundValueCarrier => AbiStorageOwner::ArtifactStatic,
            // The store outlives the whole execution, not merely the activation.
            Self::StoreHandle => AbiStorageOwner::PersistentStore,
        }
    }
}

/// **`D4` — the owner of the storage a carrier names.**
///
/// ⛔ Distinct from `AbiOwnership`, which is the *transfer discipline*. Keeping
/// them apart is what makes an impossible borrow expressible-and-rejectable
/// rather than merely unstated: `BorrowedForActivation` is not a claim at all
/// until you can say **borrowed from what**, and the thing it is borrowed from
/// must outlive the activation.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum AbiStorageOwner {
    /// The activation frame itself; reclaimed when the activation ends.
    ActivationFrame,
    /// Material minted into the compiled artifact **before execution begins**,
    /// therefore outliving every activation that reads it.
    ///
    /// ⚠ The seed environment is **not** this — it is compilation-only. See
    /// `AbiCarrier::storage_owner`.
    ArtifactStatic,
    /// The persistent store, which outlives the whole execution.
    PersistentStore,
}

/// **`D4` — the stated lifetime/aliasing/transfer/reclamation modes.**
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum AbiOwnership {
    /// The frame owns the value and reclaims it when the activation ends. May
    /// not alias a caller-visible value after return.
    OwnedByFrame,
    /// The activation borrows for its own extent; the producer reclaims. The
    /// borrow may not outlive the activation.
    BorrowedForActivation,
    /// Ownership transfers from callee to caller at return; the callee may not
    /// retain a reference.
    TransferredToCaller,
}

/// The role a slot plays in the activation frame.
///
/// ⛔ Closed on purpose: `AbiFrameHeader` accounts every slot against exactly
/// one of these, so a slot whose role is not named here cannot be laid out.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum AbiSlotKind {
    Parameter,
    Capture,
    Result,
    Control,
    Trap,
    Store,
}

/// **`D3` — which of the two capture provenances a unit's captures arrive by.**
///
/// ⚠ They differ **in kind**, and a pin keyed to one of them is a spelling
/// standing in for a population. Both are closed inputs to layout construction.
///
/// ⭐ This is recovered from `SemanticSourceKind::Expression(shape)` — planner
/// data — and never from source text.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(in crate::cranelift_backend) enum AbiCaptureProvenance {
    /// `RuntimeExpr::LexicalClosure`. Each capture is an **arbitrary source
    /// expression**, planned as a syntax child of the closure occurrence.
    Lexical,
    /// `RuntimeExpr::Closure`. Each capture is a symbol resolved against the
    /// seed environment to a **JIT-time `RuntimeGroundValue`**.
    Seed,
}

impl AbiCaptureProvenance {
    /// The carrier a capture of this provenance travels in.
    ///
    /// ⛔ **Determined by provenance alone.** There is deliberately no value
    /// parameter: the seed carrier must not be chosen by inspecting the
    /// particular runtime value, and the absence of a value argument is what
    /// makes that unrepresentable rather than merely untested.
    const fn carrier(self) -> AbiCarrier {
        match self {
            Self::Lexical => AbiCarrier::ValueWord,
            Self::Seed => AbiCarrier::GroundValueCarrier,
        }
    }
}

/// How a function unit came to be a function unit.
///
/// ⛔ Closed, and **derived from the graph**: the two arms are exactly `B2O`'s
/// two seed classes, which that node already validated to be disjoint and
/// exhaustive over the partition. `TransparentDeclarationClosure` is a closed
/// callable subkind of a scheduling entry, not a third seed class. A unit that
/// is neither seed class, or both, is a planner error rather than a defaulted
/// arm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(in crate::cranelift_backend) enum AbiUnitDefinition {
    /// A top-level scheduling entry — the root, or a transparent declaration.
    /// It has no defining closure occurrence and no captures. Only the
    /// explicitly recorded process root receives the closed ingress pair.
    SchedulingEntry {
        ingress: AbiSchedulingIngress,
    },
    /// A transparent declaration whose scheduling-entry occurrence is a
    /// closure seed. The declaration is already its own function unit; this
    /// arm gives that unit the closure's declared inputs and makes its body,
    /// rather than its callable seed value, the emitted unit body.
    TransparentDeclarationClosure {
        defining_origin: StaticOriginId,
        provenance: AbiCaptureProvenance,
    },
    /// A retained closure body. Its **defining occurrence** is the source of the
    /// unique `StaticBody` edge whose target is this unit's seed
    /// (`static_transition.rs:858`, `:884` build that edge as
    /// `closure_occurrence.entry -> body.entry`).
    ClosureBody {
        defining_origin: StaticOriginId,
        provenance: AbiCaptureProvenance,
    },
}

/// Static compilation mode for the explicitly recorded root scheduling entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum AbiRootIngress {
    Value,
    Process,
}

/// The closed source-valued ingress admitted by one scheduling entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum AbiSchedulingIngress {
    Empty,
    ProcessPair,
}

/// Role identity for the two process-root parameters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum AbiProcessParameter {
    ProcessInput,
    Capability,
}

impl AbiProcessParameter {
    const ALL: [Self; 2] = [Self::ProcessInput, Self::Capability];

    pub(in crate::cranelift_backend) const fn ordinal(self) -> u32 {
        match self {
            Self::ProcessInput => 0,
            Self::Capability => 1,
        }
    }
}

/// One declared frame slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(in crate::cranelift_backend) struct AbiSlot {
    pub(in crate::cranelift_backend) kind: AbiSlotKind,
    pub(in crate::cranelift_backend) carrier: AbiCarrier,
    pub(in crate::cranelift_backend) ownership: AbiOwnership,
    /// ⭐ Who owns the storage this slot borrows or holds. Recorded per slot so
    /// a borrow's counterparty is part of the ABI rather than prose.
    pub(in crate::cranelift_backend) storage_owner: AbiStorageOwner,
    pub(in crate::cranelift_backend) width_bytes: u16,
    pub(in crate::cranelift_backend) align_bytes: u16,
    /// Position within this slot's own kind-run, so a slot is recoverable
    /// positionally rather than by search.
    pub(in crate::cranelift_backend) ordinal: u32,
}

/// **`D1` — the common activation-frame header.**
///
/// Every unit carries the same header *fields*; the values differ per origin.
/// That is precisely what "one fixed call-ABI scheme, not one fixed byte size"
/// means.
///
/// ⛔ `frame_bytes` is **derived** from the slot run, never recorded
/// independently. A separately-recorded size would need its own agreement
/// checker, which is one more thing that can be green for the wrong reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(in crate::cranelift_backend) struct AbiFrameHeader {
    pub(in crate::cranelift_backend) parameters: u32,
    pub(in crate::cranelift_backend) captures: u32,
    pub(in crate::cranelift_backend) frame_bytes: u32,
    pub(in crate::cranelift_backend) align_bytes: u16,
}

/// The **shape** of a descriptor: everything except its positional identity.
///
/// ⭐ This exists because `AC-2`'s property is about *layout*, not about *where
/// in the node table a unit landed*. Adding an irrelevant binding to the caller
/// renumbers the node table, so `planned_node` and `origin` legitimately move
/// while the layout must not. Comparing whole descriptors would conflate the
/// two and report a false violation; comparing only the header and the slot run
/// asks the question the constraint actually poses.
///
/// ⚠ Recorded before measurement in `docs/program/rt-fnsplit-b2r-predictions.md`
/// (`P2`), so this narrowing is a stated design choice and not a red test
/// trimmed until it passed.
///
/// ⛔ `cfg(test)`: this is **probe infrastructure**, and `AC-6` requires
/// executable probes to be test-only. Production never needs a descriptor's
/// shape in isolation — the validator compares against a re-derivation instead.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AbiDescriptorShape {
    pub(super) definition_is_closure_body: bool,
    pub(super) provenance: Option<AbiCaptureProvenance>,
    pub(super) header: AbiFrameHeader,
    pub(super) slots: Vec<AbiSlot>,
}

/// One function unit's complete representation contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct AbiDescriptor {
    pub(super) function: PredeclaredFunctionId,
    /// This unit's entry node — its seed in the owner partition.
    pub(super) planned_node: StaticNodeId,
    pub(super) origin: StaticOriginId,
    pub(super) definition: AbiUnitDefinition,
    pub(super) header: AbiFrameHeader,
    /// This descriptor's dense run in `AbiPlane::slots`, laid out in kind order:
    /// parameters, captures, result, control, trap, store.
    pub(super) slots: DenseRange,
}

/// The ABI plane: one descriptor per `PredeclaredFunction`, and their slots.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct AbiPlane {
    pub(super) descriptors: Vec<AbiDescriptor>,
    pub(super) slots: Vec<AbiSlot>,
}

/// The fixed per-unit convention slots every activation carries, in layout
/// order after the parameters and captures.
///
/// ⛔ Named as a constant rather than spelled `4` at the arithmetic sites: the
/// "no implicit caller-environment tail" check below is *exactly* the statement
/// that a frame's slot count is `parameters + captures + CONVENTION_SLOTS`, and
/// a bare literal there would be a magic number in the one place the constraint
/// lives.
const CONVENTION_SLOTS: [(AbiSlotKind, AbiCarrier); 4] = [
    (AbiSlotKind::Result, AbiCarrier::ResultWord),
    (AbiSlotKind::Control, AbiCarrier::ControlWord),
    (AbiSlotKind::Trap, AbiCarrier::TrapWord),
    (AbiSlotKind::Store, AbiCarrier::StoreHandle),
];

#[cfg(test)]
impl AbiPlane {
    /// The shape of one descriptor, for the `AC-2`/`AC-4` invariance controls.
    pub(super) fn shape(
        &self,
        descriptor: &AbiDescriptor,
    ) -> Result<AbiDescriptorShape, CraneliftBackendError> {
        let slots = slot_slice(&self.slots, descriptor.slots)?;
        let (definition_is_closure_body, provenance) = match descriptor.definition {
            AbiUnitDefinition::SchedulingEntry { .. } => (false, None),
            AbiUnitDefinition::TransparentDeclarationClosure { provenance, .. } => {
                (false, Some(provenance))
            }
            AbiUnitDefinition::ClosureBody { provenance, .. } => (true, Some(provenance)),
        };
        Ok(AbiDescriptorShape {
            definition_is_closure_body,
            provenance,
            header: descriptor.header,
            slots: slots.to_vec(),
        })
    }

    /// Every descriptor's shape, in unit order.
    pub(super) fn shapes(&self) -> Result<Vec<AbiDescriptorShape>, CraneliftBackendError> {
        self.descriptors
            .iter()
            .map(|descriptor| self.shape(descriptor))
            .collect()
    }
}

/// **`D2` — descriptor construction from the owner partition.**
///
/// ⛔ The signature is load-bearing evidence for `AC-3`/`AC-4`. It takes the
/// semantic plane, the planner's source seeds, and the graph — and **nothing
/// that holds a runtime value.** Neither `SemanticPlane` nor
/// `SemanticSourceSeed` contains a `RuntimeGroundValue` or a `Lowered`, so
/// "the descriptor cannot vary with the particular runtime value" is enforced
/// by the type system here rather than observed by a test.
///
/// **MEASURED:** the builder's inputs contain no runtime value.
/// **CLAIMED:** a seed capture's layout is not chosen by inspecting its value.
/// **THE GAP:** this pins the **descriptor**. It does **not** pin that `B2F`'s
/// emission path stays value-independent — that obligation is `B2F`'s, and the
/// residual is recorded here rather than covered.
pub(super) fn build_abi_plane(
    plane: &SemanticPlane,
    nodes: &[StaticNode],
    sources_in: &[SemanticSourceSeed],
    edges: &[StaticEdge],
    entries: &[StaticNodeId],
    root_entry: StaticNodeId,
    root_ingress: AbiRootIngress,
) -> Result<AbiPlane, CraneliftBackendError> {
    // ⛔ The planner's `semantic_sources` are in **walk order**, not positional
    // by origin. Reading `sources[origin]` directly returns a plausible seed for
    // the wrong occurrence, so the positioning is done once, here, through the
    // same helper the semantic plane uses.
    let sources = positioned_sources(nodes, sources_in)?;
    let sources = sources.as_slice();

    let definitions =
        unit_definitions(plane, sources, edges, entries, root_entry, root_ingress)?;

    // `C4`, and deliberately before any descriptor is minted: an imported edge
    // must receive **no** callable descriptor at all, so the exclusion runs
    // before construction rather than as a filter afterwards.
    reject_imported_capture_edges(plane, sources, &definitions)?;

    let mut abi = AbiPlane::default();
    for (ordinal, function) in plane.functions.iter().enumerate() {
        let id = PredeclaredFunctionId(
            u32::try_from(ordinal)
                .map_err(|_| planner_capacity_error("abi descriptor identity exhausted"))?,
        );
        if function.id != id {
            return Err(planner_error("abi descriptor is not positional for its function unit"));
        }
        let definition = definitions[ordinal];
        let (parameters, captures) = declared_arity(plane, sources, definition)?;

        let slot_start = abi.slots.len();
        push_slots(&mut abi.slots, definition, parameters, captures)?;
        let slots = DenseRange {
            start: u32::try_from(slot_start)
                .map_err(|_| planner_capacity_error("abi slot identity exhausted"))?,
            len: u32::try_from(abi.slots.len() - slot_start)
                .map_err(|_| planner_capacity_error("abi slot range exhausted"))?,
        };
        let header = frame_header(&abi.slots[slot_start..], parameters, captures)?;

        abi.descriptors.push(AbiDescriptor {
            function: id,
            planned_node: function.planned_node,
            origin: function.origin,
            definition,
            header,
            slots,
        });
    }

    abi.validate(
        plane,
        nodes,
        sources_in,
        edges,
        entries,
        root_entry,
        root_ingress,
    )?;
    Ok(abi)
}

/// **`C4`/`AC-5` — cross-module linking is a CHECKED exclusion.**
///
/// An imported declaration receives **no callable descriptor** and fails here,
/// before emission, with the existing dependency-linking unsupported result —
/// not with a generic planner error, and not in a comment.
///
/// ⭐ **The scope is an imported EDGE, not an imported mention, and getting that
/// wrong is a real defect I shipped once.** My first implementation rejected
/// every occurrence whose result carrier is unrepresentable, which condemned any
/// plan that merely *contained* an `ImportedDeclarationRef` anywhere. That is
/// strictly stronger than `C4`, and
/// `every_expression_typed_field_is_a_reachable_positional_child_origin` — a
/// pre-existing property test that legitimately enumerates every expression
/// shape — caught it. `C4` excludes the position where an imported value would
/// have to **cross a frame boundary and be given a carrier**, which is a capture
/// slot, not an arbitrary evaluation site.
///
/// ⚠ **Non-vacuity is constructed, not assumed.** A lexical closure's captures
/// are arbitrary source expressions (`static_transition.rs:884`), so
/// `LexicalClosure { captures: [ImportedDeclarationRef { .. }], .. }` is a real,
/// buildable plan in which an imported value crosses into a frame. That is the
/// imported edge, and it is what the paired positive control varies against.
///
/// ⚠ The **seed** provenance cannot carry one at all: its captures resolve to a
/// `RuntimeGroundValue`, closed at six variants none of which is a declaration
/// reference. The asymmetry is stated rather than left to look like coverage.
fn reject_imported_capture_edges(
    plane: &SemanticPlane,
    sources: &[SemanticSourceSeed],
    definitions: &[AbiUnitDefinition],
) -> Result<(), CraneliftBackendError> {
    for definition in definitions {
        let (defining_origin, provenance) = match *definition {
            AbiUnitDefinition::ClosureBody {
                defining_origin,
                provenance,
            }
            | AbiUnitDefinition::TransparentDeclarationClosure {
                defining_origin,
                provenance,
            } => (defining_origin, provenance),
            AbiUnitDefinition::SchedulingEntry { .. } => continue,
        };
        if provenance != AbiCaptureProvenance::Lexical {
            continue;
        }
        for capture in lexical_capture_origins(plane, defining_origin)? {
            let seed = source_for(sources, capture)?;
            result_carrier(seed.source)?;
        }
    }
    Ok(())
}

/// The origins of a lexical closure's capture children.
///
/// A `LexicalClosure` occurrence's positional children are `[body, captures..]`
/// (`static_transition.rs:884` pushes the body first, then the capture
/// occurrences), so the captures are children `1..`.
fn lexical_capture_origins(
    plane: &SemanticPlane,
    defining_origin: StaticOriginId,
) -> Result<Vec<StaticOriginId>, CraneliftBackendError> {
    let descriptor = plane
        .descriptors
        .get(defining_origin.0 as usize)
        .ok_or_else(|| planner_error("defining occurrence has no semantic descriptor"))?;
    let program = plane
        .programs
        .get(descriptor.program.0 as usize)
        .ok_or_else(|| planner_error("defining occurrence names an unknown semantic program"))?;
    let records = plane
        .records
        .get(program.records.start as usize..range_end(program.records)?)
        .ok_or_else(|| planner_error("semantic program record range is outside the plane"))?;
    let [record] = records else {
        return Err(planner_error(
            "defining occurrence's program does not hold exactly one record",
        ));
    };
    let children = plane
        .child_origins
        .get(record.child_origins.start as usize..range_end(record.child_origins)?)
        .ok_or_else(|| planner_error("semantic child-origin range is outside the plane"))?;
    let [_body, captures @ ..] = children else {
        return Err(planner_error(
            "lexical closure occurrence has no body child origin",
        ));
    };
    Ok(captures.to_vec())
}

/// The carrier an occurrence's result travels in.
///
/// ⛔ Exhaustive over both source kinds with **no `_ =>` arm**. A new
/// `RuntimeExprShape` or `TransitionKind` must state its carrier explicitly; it
/// cannot inherit `ValueWord` by omission, which is how an unrepresentable
/// construct would otherwise acquire a representation silently.
fn result_carrier(source: SemanticSourceKind) -> Result<AbiCarrier, CraneliftBackendError> {
    Ok(match source {
        SemanticSourceKind::Expression(shape) => match shape {
            RuntimeExprShape::ImportedDeclarationRef => {
                // The one unrepresentable shape, and the reason `B2R` scopes to
                // the complete **intra-module** callable bundle.
                return Err(unsupported(
                    "ImportedDeclarationRef",
                    "imported declaration requires dependency linking, so it receives no callable \
                     descriptor in the intra-module representation contract",
                ));
            }
            RuntimeExprShape::CheckedJoinSite
            | RuntimeExprShape::CheckedSubcontinuationFrame
            | RuntimeExprShape::CheckedRecursiveInvocation
            | RuntimeExprShape::CheckedComputationalIHSlots
            | RuntimeExprShape::CheckedComputationalIHInvocation
            | RuntimeExprShape::Value
            | RuntimeExprShape::Var
            | RuntimeExprShape::Let
            | RuntimeExprShape::If
            | RuntimeExprShape::PrimitiveCall
            | RuntimeExprShape::Construct
            | RuntimeExprShape::Match
            | RuntimeExprShape::ComputationalMatch
            | RuntimeExprShape::Record
            | RuntimeExprShape::Project
            | RuntimeExprShape::Closure
            | RuntimeExprShape::LexicalClosure
            | RuntimeExprShape::DeclarationRef
            | RuntimeExprShape::Call
            | RuntimeExprShape::Effect => AbiCarrier::ValueWord,
            RuntimeExprShape::Trap => AbiCarrier::TrapWord,
        },
        SemanticSourceKind::Control(transition) => match transition {
            TransitionKind::TrapTerminal => AbiCarrier::TrapWord,
            TransitionKind::Terminal
            | TransitionKind::ClosureBody
            | TransitionKind::ProducerTail
            | TransitionKind::CompletedTail => AbiCarrier::ResultWord,
            TransitionKind::Evaluate
            | TransitionKind::Sequence
            | TransitionKind::Branch
            | TransitionKind::CaseTest
            | TransitionKind::ProducerWrapper
            | TransitionKind::SourceReturnResume => AbiCarrier::ControlWord,
        },
    })
}

/// Classifies every function unit into its definition arm, from the graph.
///
/// ⛔ **Derived, never hand-authored.** A unit is a `ClosureBody` iff its seed is
/// the target of a `StaticBody` edge, and a `SchedulingEntry` iff its seed is in
/// `entries`. `B2O` already validates those two classes are disjoint and cover
/// the partition; this function re-derives the classification rather than
/// trusting it, and a seed that is **neither** or **both** is a named planner
/// error instead of a defaulted arm.
fn unit_definitions(
    plane: &SemanticPlane,
    sources: &[SemanticSourceSeed],
    edges: &[StaticEdge],
    entries: &[StaticNodeId],
    root_entry: StaticNodeId,
    root_ingress: AbiRootIngress,
) -> Result<Vec<AbiUnitDefinition>, CraneliftBackendError> {
    // One pass over the edges rather than one per unit: the classification is
    // O(nodes + edges), not O(units × edges).
    let mut body_edge_from = vec![None; plane.descriptors.len()];
    for edge in edges {
        if edge.kind != EdgeKind::StaticBody {
            continue;
        }
        let slot = body_edge_from
            .get_mut(edge.to.0 as usize)
            .ok_or_else(|| planner_error("static body edge target is outside the planned nodes"))?;
        if slot.replace(edge.from).is_some() {
            return Err(planner_error(
                "function unit seed has more than one defining static body edge",
            ));
        }
    }
    let mut is_entry_node = vec![false; plane.descriptors.len()];
    for entry in entries {
        let slot = is_entry_node
            .get_mut(entry.0 as usize)
            .ok_or_else(|| planner_error("scheduling entry is outside the planned nodes"))?;
        *slot = true;
    }

    let mut definitions = Vec::with_capacity(plane.functions.len());
    for function in &plane.functions {
        let index = function.planned_node.0 as usize;
        let is_entry = *is_entry_node
            .get(index)
            .ok_or_else(|| planner_error("function unit seed is outside the planned nodes"))?;
        let body_edge = body_edge_from
            .get(index)
            .copied()
            .ok_or_else(|| planner_error("function unit seed is outside the planned nodes"))?;
        let definition = match (is_entry, body_edge) {
            (true, None) => {
                if function.planned_node == root_entry {
                    AbiUnitDefinition::SchedulingEntry {
                        ingress: match root_ingress {
                            AbiRootIngress::Value => AbiSchedulingIngress::Empty,
                            AbiRootIngress::Process => AbiSchedulingIngress::ProcessPair,
                        },
                    }
                } else {
                    let defining_origin = function.origin;
                    let seed = source_for(sources, defining_origin)?;
                    match seed.source {
                        SemanticSourceKind::Expression(RuntimeExprShape::Closure)
                        | SemanticSourceKind::Expression(RuntimeExprShape::LexicalClosure) => {
                            AbiUnitDefinition::TransparentDeclarationClosure {
                                defining_origin,
                                provenance: closure_provenance(seed.source)?,
                            }
                        }
                        SemanticSourceKind::Expression(_) | SemanticSourceKind::Control(_) => {
                            AbiUnitDefinition::SchedulingEntry {
                                ingress: AbiSchedulingIngress::Empty,
                            }
                        }
                    }
                }
            }
            (false, Some(from)) => {
                let defining_origin = StaticOriginId(from.0);
                let seed = source_for(sources, defining_origin)?;
                AbiUnitDefinition::ClosureBody {
                    defining_origin,
                    provenance: closure_provenance(seed.source)?,
                }
            }
            (true, Some(_)) => {
                return Err(planner_error(
                    "function unit seed is both a scheduling entry and a static body target",
                ));
            }
            (false, None) => {
                return Err(planner_error(
                    "function unit seed is neither a scheduling entry nor a static body target",
                ));
            }
        };
        definitions.push(definition);
    }
    Ok(definitions)
}

/// **`D3` — the provenance of a defining closure occurrence.**
///
/// ⛔ Read off `SemanticSourceKind`, which is planner data. A `StaticBody`
/// edge whose source is not a closure occurrence is a graph the planner did not
/// build, and is a named error rather than a defaulted provenance.
fn closure_provenance(
    source: SemanticSourceKind,
) -> Result<AbiCaptureProvenance, CraneliftBackendError> {
    match source {
        SemanticSourceKind::Expression(RuntimeExprShape::Closure) => Ok(AbiCaptureProvenance::Seed),
        SemanticSourceKind::Expression(RuntimeExprShape::LexicalClosure) => {
            Ok(AbiCaptureProvenance::Lexical)
        }
        SemanticSourceKind::Expression(_) | SemanticSourceKind::Control(_) => Err(planner_error(
            "static body edge source is not a closure occurrence",
        )),
    }
}

/// A unit's declared parameter and capture counts.
///
/// ⭐ **`C1` lives here.** The counts come from the **defining occurrence's own
/// declaration** — its `ParamName` atoms and its recorded `capture_slots` — and
/// from nothing else. No suffix of any caller's environment is consulted, so
/// caller depth cannot reach these numbers.
///
/// An ordinary `SchedulingEntry` declares only its ingress. A transparent
/// closure declaration instead derives its parameter and capture counts from
/// the defining closure occurrence.
fn declared_arity(
    plane: &SemanticPlane,
    sources: &[SemanticSourceSeed],
    definition: AbiUnitDefinition,
) -> Result<(u32, u32), CraneliftBackendError> {
    let defining_origin = match definition {
        AbiUnitDefinition::ClosureBody {
            defining_origin, ..
        }
        | AbiUnitDefinition::TransparentDeclarationClosure {
            defining_origin, ..
        } => defining_origin,
        AbiUnitDefinition::SchedulingEntry { ingress } => {
            return Ok(match ingress {
                AbiSchedulingIngress::Empty => (0, 0),
                AbiSchedulingIngress::ProcessPair => (
                    u32::try_from(AbiProcessParameter::ALL.len())
                        .map_err(|_| planner_capacity_error("process ingress arity exhausted"))?,
                    0,
                ),
            });
        }
    };

    let seed = source_for(sources, defining_origin)?;
    let descriptor = plane
        .descriptors
        .get(defining_origin.0 as usize)
        .ok_or_else(|| planner_error("defining occurrence has no semantic descriptor"))?;
    let layout = plane
        .capture_layouts
        .get(descriptor.capture_layout.0 as usize)
        .ok_or_else(|| planner_error("defining occurrence has no capture layout"))?;

    // The recorded layout and the source seed must agree on the capture count.
    // They are written by different code paths, so a disagreement is a real
    // detector rather than a restatement.
    if layout.slots.len != seed.capture_slots {
        return Err(planner_error(
            "capture layout slot count disagrees with its occurrence's declared captures",
        ));
    }

    let (parameters, _) = occurrence_atom_counts(plane, defining_origin)?;
    Ok((parameters, seed.capture_slots))
}

/// One occurrence's own non-child semantic atoms.
///
/// ⛔ Shared by `declared_arity` and `occurrence_atom_counts` so the two cannot
/// disagree about which atoms belong to an occurrence — a disagreement neither
/// would detect, since each would simply be reading its own answer.
fn occurrence_operands<'plane>(
    plane: &'plane SemanticPlane,
    origin: StaticOriginId,
) -> Result<&'plane [SemanticOperandElement], CraneliftBackendError> {
    let descriptor = plane
        .descriptors
        .get(origin.0 as usize)
        .ok_or_else(|| planner_error("defining occurrence has no semantic descriptor"))?;
    let program = plane
        .programs
        .get(descriptor.program.0 as usize)
        .ok_or_else(|| planner_error("defining occurrence names an unknown semantic program"))?;
    let records = plane
        .records
        .get(program.records.start as usize..range_end(program.records)?)
        .ok_or_else(|| planner_error("semantic program record range is outside the plane"))?;
    let [record] = records else {
        return Err(planner_error(
            "defining occurrence's program does not hold exactly one record",
        ));
    };
    plane
        .operands
        .get(record.operands.start as usize..range_end(record.operands)?)
        .ok_or_else(|| planner_error("semantic operand range is outside the plane"))
}

/// Lays one unit's slot run: parameters, then captures, then the fixed
/// convention slots, in that order.
fn push_slots(
    slots: &mut Vec<AbiSlot>,
    definition: AbiUnitDefinition,
    parameters: u32,
    captures: u32,
) -> Result<(), CraneliftBackendError> {
    for ordinal in 0..parameters {
        slots.push(slot(AbiSlotKind::Parameter, AbiCarrier::ValueWord, ordinal));
    }

    // ⛔ The capture carrier is a function of **provenance**, never of a value.
    // A `SchedulingEntry` has no captures at all, so its carrier question does
    // not arise rather than being answered with a default.
    let capture_carrier = match definition {
        AbiUnitDefinition::SchedulingEntry { .. } => {
            if captures != 0 {
                return Err(planner_error(
                    "scheduling entry unit declares captures, which it cannot have",
                ));
            }
            None
        }
        AbiUnitDefinition::TransparentDeclarationClosure { provenance, .. }
        | AbiUnitDefinition::ClosureBody { provenance, .. } => Some(provenance.carrier()),
    };
    if let Some(carrier) = capture_carrier {
        for ordinal in 0..captures {
            slots.push(slot(AbiSlotKind::Capture, carrier, ordinal));
        }
    }

    for (kind, carrier) in CONVENTION_SLOTS {
        slots.push(slot(kind, carrier, 0));
    }
    Ok(())
}

const fn slot(kind: AbiSlotKind, carrier: AbiCarrier, ordinal: u32) -> AbiSlot {
    AbiSlot {
        kind,
        carrier,
        ownership: carrier.ownership(),
        storage_owner: carrier.storage_owner(),
        width_bytes: carrier.width_bytes(),
        align_bytes: carrier.align_bytes(),
        ordinal,
    }
}

/// **`RT-FNSPLIT-B2F` `D2` — each slot's byte offset in its unit's frame, and
/// the frame's total size, from ONE walk.**
///
/// ⭐ **This exists so the emitter cannot own a second layout derivation.**
/// `B2F` has to know where a slot sits in order to load or store it, and
/// `AbiSlot` records a width but no offset. The obvious repair — let the
/// emitter prefix-sum the widths itself — would put the same arithmetic in two
/// files, where the two can disagree and only a test would notice. ⛔ Instead
/// the walk lives here once, [`frame_header`] totals *through* it, and the
/// emitter reads it. **A divergence is then unrepresentable rather than
/// merely untested.**
///
/// ⚠ There is no inter-slot padding, and that is a *consequence*, not an
/// assumption: `AbiCarrier::width_bytes` and `align_bytes` are `8` for every
/// variant, so each offset is already a multiple of every slot's alignment.
/// ⛔ Do not read this as a licence to assume 8 — if a future carrier is
/// narrower, this walk is the one place that has to learn about padding, which
/// is exactly why it is one place.
pub(in crate::cranelift_backend) fn slot_offsets(
    slots: &[AbiSlot],
) -> Result<(Vec<u32>, u32), CraneliftBackendError> {
    let mut offsets = Vec::with_capacity(slots.len());
    let mut frame_bytes = 0u32;
    for slot in slots {
        offsets.push(frame_bytes);
        frame_bytes = frame_bytes
            .checked_add(u32::from(slot.width_bytes))
            .ok_or_else(|| planner_capacity_error("abi frame size exhausted"))?;
    }
    Ok((offsets, frame_bytes))
}

/// Derives the frame header from the laid slot run.
fn frame_header(
    slots: &[AbiSlot],
    parameters: u32,
    captures: u32,
) -> Result<AbiFrameHeader, CraneliftBackendError> {
    // ⛔ The total comes from `slot_offsets`, not from a second sum here: the
    // emitter's offsets and this header's size are then the same walk by
    // construction.
    let (_, frame_bytes) = slot_offsets(slots)?;
    let mut align_bytes = 1u16;
    for slot in slots {
        align_bytes = align_bytes.max(slot.align_bytes);
    }
    Ok(AbiFrameHeader {
        parameters,
        captures,
        frame_bytes,
        align_bytes,
    })
}

// ─── RT-FNSPLIT-B2F AC-11 — per-transfer representability, at the EMITTED slots ─

/// The deepest value-flow chain the producer walk will follow before failing
/// closed.
///
/// ⛔ The origin graph is a tree of positional children, so this cannot be hit
/// by a cycle. It is a capacity guard: exceeding it **rejects**, because a walk
/// that gave up and returned "representable" would be a fail-open default, and
/// a fail-open default is the exact defect the amended `AC-2` was ruled on.
const MAX_PRODUCER_DEPTH: usize = 64;

/// **`AC-11` — every boundary transfer `B2F` emits is representable, established
/// HERE and not inherited from `C4`.**
///
/// ⛔ **`C4` does not establish this and must not be cited as though it did.**
/// `reject_imported_capture_edges` iterates a lexical closure's **direct capture
/// children** and asks `result_carrier(seed.source)` — which answers *"is this
/// capture expression's own top-level shape `ImportedDeclarationRef`?"*, not
/// *"can an imported value reach this frame slot?"*. Two consequences, both
/// buildable plans that plan green:
///
/// | | |
/// |---|---|
/// | **Hole A** | any wrapper defeats it — `If { Bool(true), imported, imported }` is **binder-free**, so no de Bruijn reading makes its result anything but the imported value, and it receives a full `Capture` slot |
/// | **Hole B** | needs no wrapper — `LexicalClosure { captures: [], body: ImportedDeclarationRef }`; the function iterates capture children only, so the unit's own **result** slot is never carrier-checked |
///
/// ⛔ **This is a NEW, `B2F`-owned check. It does not touch `C4`**, whose repair
/// rides `RT-FNSPLIT-B2O-CHECK` — an `L` node on an atomic boundary does not
/// absorb a checking-layer repair.
///
/// ⛔ **And it runs BEFORE any unit is declared or defined** (clause 3): its call
/// site in `compile_expr_into_module` precedes `declare_unit_bundle`, so no path
/// can treat `AbiPlane::validate`, `C4`, or descriptor existence as a substitute
/// for it.
///
/// **MEASURED:** for every emitted unit, the value-flow producers reaching its
/// `Capture` slots and its `Result` slot all have an admitted carrier; and its
/// `Control` / `Trap` / `Store` slots carry exactly the fixed protocol carrier.
/// **CLAIMED:** every transfer this node emits is representable.
/// **THE GAP:** ⛔ stated as a partition in `producers_of` below — the
/// pass-through relation covers `If` and `Let`, and a `Match` arm is **not**
/// traced.
pub(super) fn validate_emitted_transfers(
    plane: &SemanticPlane,
    nodes: &[StaticNode],
    sources_in: &[SemanticSourceSeed],
    descriptors: &[AbiDescriptor],
    slots: &[AbiSlot],
) -> Result<(), CraneliftBackendError> {
    // ⛔ The seeds must be POSITIONED before `source_for` can index them: it
    // resolves an origin by position and rejects a seed whose recorded origin is
    // not its own index. `build_abi_plane` does the same conversion at its own
    // entry, and skipping it here produced a planner error on every plan with
    // more than a trivial source order -- the failure was loud, but only because
    // `source_for` refuses to guess.
    let sources = &positioned_sources(nodes, sources_in)?;
    for descriptor in descriptors {
        let run = slot_slice(slots, descriptor.slots)?;
        for slot in run {
            match slot.kind {
                // ⭐ Clause 2. Protocol slots are **protocol-produced**, not the
                // result of any source expression, so `result_carrier` is the
                // wrong instrument for them and the AC says so. The expected
                // carrier is written out here rather than read back from
                // `AbiSlotKind`, so this compares two independent statements
                // instead of one statement with itself.
                AbiSlotKind::Control => require_protocol_carrier(slot, AbiCarrier::ControlWord)?,
                AbiSlotKind::Trap => require_protocol_carrier(slot, AbiCarrier::TrapWord)?,
                AbiSlotKind::Store => require_protocol_carrier(slot, AbiCarrier::StoreHandle)?,
                // ⚠ **The emitted population of parameter-argument transfers is
                // EMPTY**, and that is stated rather than passed over: an
                // argument is supplied by a **call site**, and `D4`'s call edges
                // are not emitted yet. ⛔ A vacuous pass is recorded as vacuous —
                // when `S5` emits call edges, each argument's producer joins this
                // walk, and until then there is nothing to trace.
                AbiSlotKind::Parameter => {}
                // Source-valued. Both are traced below, per unit rather than per
                // slot, because a capture's origin comes from the defining
                // occurrence and the result's from the unit's own.
                AbiSlotKind::Capture | AbiSlotKind::Result => {}
            }
        }

        // ⭐ Hole B: the unit's OWN result. `C4` never carrier-checks this, and
        // `LexicalClosure { captures: [], body: ImportedDeclarationRef }` needs
        // no wrapper at all to exploit it.
        require_representable_producers(plane, sources, descriptor.origin)?;

        // ⭐ Hole A: each capture, traced through binder-free wrappers rather
        // than read off the child's own top-level shape.
        if let AbiUnitDefinition::ClosureBody {
            defining_origin,
            provenance,
        }
        | AbiUnitDefinition::TransparentDeclarationClosure {
            defining_origin,
            provenance,
        } = descriptor.definition
        {
            // ⚠ The **seed** provenance cannot carry an imported value at all:
            // its captures resolve to a `RuntimeGroundValue`, closed at six
            // variants none of which is a declaration reference. The asymmetry
            // is stated rather than left to look like coverage.
            if provenance == AbiCaptureProvenance::Lexical {
                for capture in lexical_capture_origins(plane, defining_origin)? {
                    require_representable_producers(plane, sources, capture)?;
                }
            }
        }
    }
    Ok(())
}

/// A protocol slot must carry exactly the carrier the ABI fixes for its role.
fn require_protocol_carrier(
    slot: &AbiSlot,
    expected: AbiCarrier,
) -> Result<(), CraneliftBackendError> {
    if slot.carrier != expected {
        return Err(planner_error(
            "a protocol slot does not carry the fixed carrier its role declares",
        ));
    }
    Ok(())
}

/// Every value-flow producer reaching `origin` must have an admitted carrier.
fn require_representable_producers(
    plane: &SemanticPlane,
    sources: &[SemanticSourceSeed],
    origin: StaticOriginId,
) -> Result<(), CraneliftBackendError> {
    for producer in producers_of(plane, sources, origin, 0)? {
        let seed = source_for(sources, producer)?;
        result_carrier(seed.source)?;
    }
    Ok(())
}

/// The set of occurrences whose value can actually **reach** `origin`'s slot.
///
/// ⭐ **The whole content of `AC-11` clause 1 is that this is not the identity
/// function.** Checking `origin`'s own top-level shape is what `C4` does, and a
/// binder-free wrapper defeats it.
///
/// ⛔ **NOT CLAIMED, as a partition with its discriminator.** The pass-through
/// relation below covers `If` and `Let`, whose positional child layout is
/// measured (`If` = `[cond, then, else]`, `Let` = `[value, body]`). ⚠ **A
/// `Match` or `ComputationalMatch` arm is NOT traced** — its case bodies are
/// derived through `case_body_occurrence` rather than `child_occurrence`, and I
/// have not established that they land in `plane.child_origins` at the positions
/// this walk would read. ⇒ **The discriminator: does the value reach the slot
/// through a match arm?** If yes, only the match occurrence's own carrier is
/// checked and the arm is not. ⛔ That is a stated residual, not a covered case,
/// and widening it needs the case-body origin layout established first — over-
/// reading those positions would reject representable programs.
fn producers_of(
    plane: &SemanticPlane,
    sources: &[SemanticSourceSeed],
    origin: StaticOriginId,
    depth: usize,
) -> Result<Vec<StaticOriginId>, CraneliftBackendError> {
    if depth > MAX_PRODUCER_DEPTH {
        return Err(planner_error(
            "value-flow producer chain is deeper than the walk admits",
        ));
    }
    let seed = source_for(sources, origin)?;
    let SemanticSourceKind::Expression(shape) = seed.source else {
        // A transition-kind source is protocol-produced, not a source
        // expression; it is its own producer.
        return Ok(vec![origin]);
    };
    // ⛔ The pass-through set is an ALLOW-LIST of relations, not a deny-list of
    // shapes: an unrecognised shape is treated as its own producer and still has
    // its carrier checked, so a new `RuntimeExprShape` cannot acquire a
    // pass-through it was never given.
    let forwarded: &[usize] = match shape {
        // `[cond, then, else]` -- the value is one of the two branches, and
        // neither introduces a binder. This is the Architect's named
        // discriminator, `If { Bool(true), imported, imported }`.
        RuntimeExprShape::If => &[1, 2],
        // `[value, body]` -- the value that flows out is the body's.
        RuntimeExprShape::Let => &[1],
        _ => &[],
    };
    if forwarded.is_empty() {
        return Ok(vec![origin]);
    }
    let children = child_origins_of(plane, origin)?;
    let mut producers = Vec::new();
    for index in forwarded {
        let Some(child) = children.get(*index) else {
            // ⛔ Fails closed. A shape declared pass-through whose children are
            // not where its layout says they are means the two disagree, and
            // treating the occurrence as its own producer here would silently
            // restore exactly the top-level-shape check this walk replaces.
            return Err(planner_error(
                "a pass-through occurrence lacks the child its layout declares",
            ));
        };
        producers.extend(producers_of(plane, sources, *child, depth + 1)?);
    }
    Ok(producers)
}

/// One occurrence's positional syntax-child origins.
fn child_origins_of(
    plane: &SemanticPlane,
    origin: StaticOriginId,
) -> Result<Vec<StaticOriginId>, CraneliftBackendError> {
    let descriptor = plane
        .descriptors
        .get(origin.0 as usize)
        .ok_or_else(|| planner_error("occurrence has no semantic descriptor"))?;
    let program = plane
        .programs
        .get(descriptor.program.0 as usize)
        .ok_or_else(|| planner_error("occurrence names an unknown semantic program"))?;
    let records = plane
        .records
        .get(program.records.start as usize..range_end(program.records)?)
        .ok_or_else(|| planner_error("semantic program record range is outside the plane"))?;
    let [record] = records else {
        return Err(planner_error(
            "occurrence's program does not hold exactly one record",
        ));
    };
    Ok(plane
        .child_origins
        .get(record.child_origins.start as usize..range_end(record.child_origins)?)
        .ok_or_else(|| planner_error("semantic child-origin range is outside the plane"))?
        .to_vec())
}

fn source_for(
    sources: &[SemanticSourceSeed],
    origin: StaticOriginId,
) -> Result<SemanticSourceSeed, CraneliftBackendError> {
    let seed = sources
        .get(origin.0 as usize)
        .ok_or_else(|| planner_error("static origin is outside the planner's source seeds"))?;
    if seed.origin != origin {
        return Err(planner_error(
            "source seed origin is not its preallocated positional identity",
        ));
    }
    Ok(*seed)
}

fn slot_slice(slots: &[AbiSlot], range: DenseRange) -> Result<&[AbiSlot], CraneliftBackendError> {
    slots
        .get(range.start as usize..range_end(range)?)
        .ok_or_else(|| planner_error("abi slot range is outside the plane"))
}

impl AbiPlane {
    /// **`D5` — the fail-closed pre-emission validator.**
    ///
    /// ⛔ Deliberately **not** one composite check. A single "the ABI is fine"
    /// assertion is discharged by any one of its conjuncts holding, so the
    /// mutations `AC-1`–`AC-5` require would be indistinguishable from each
    /// other. Each law below has its own named failure.
    ///
    /// Everything the builder derived is **re-derived here and compared**. That
    /// is what makes a corrupted descriptor a planner error rather than a
    /// plausible wrong answer — a validator that only re-read what the builder
    /// wrote would be checking its own output against itself.
    ///
    /// ⛔ Failure is a planner error **before emission**. There is no fallback to
    /// the old specializer after partial emission, because nothing here emits at
    /// all.
    pub(super) fn validate(
        &self,
        plane: &SemanticPlane,
        nodes: &[StaticNode],
        sources: &[SemanticSourceSeed],
        edges: &[StaticEdge],
        entries: &[StaticNodeId],
        root_entry: StaticNodeId,
        root_ingress: AbiRootIngress,
    ) -> Result<(), CraneliftBackendError> {
        let sources = positioned_sources(nodes, sources)?;
        let sources = sources.as_slice();

        // `AC-1`, direction 1 — every function unit has exactly one descriptor.
        if self.descriptors.len() != plane.functions.len() {
            return Err(planner_error(
                "abi descriptor population is not exact for the function unit partition",
            ));
        }

        let definitions =
            unit_definitions(plane, sources, edges, entries, root_entry, root_ingress)?;

        for (ordinal, descriptor) in self.descriptors.iter().enumerate() {
            // `AC-1`, direction 2 — every descriptor names a member of the
            // partition, positionally. A one-directional check passes happily on
            // an orphan, so both directions are asserted.
            let function = plane.functions.get(ordinal).ok_or_else(|| {
                planner_error("abi descriptor names a function unit outside the partition")
            })?;
            let id = PredeclaredFunctionId(
                u32::try_from(ordinal)
                    .map_err(|_| planner_capacity_error("abi descriptor identity exhausted"))?,
            );
            if descriptor.function != id
                || descriptor.planned_node != function.planned_node
                || descriptor.origin != function.origin
            {
                return Err(planner_error(
                    "abi descriptor is not positional for its function unit",
                ));
            }

            // The definition arm is re-derived from the graph, not re-read.
            if descriptor.definition != definitions[ordinal] {
                return Err(planner_error(
                    "abi descriptor definition is not the unit's derived definition",
                ));
            }

            let (parameters, captures) = declared_arity(plane, sources, definitions[ordinal])?;
            if descriptor.header.parameters != parameters {
                return Err(planner_error(
                    "abi descriptor parameter count is not its origin's declared arity",
                ));
            }
            // `D5` — missing capture slots, and extra capture slots, each named.
            if descriptor.header.captures < captures {
                return Err(planner_error(
                    "abi descriptor is missing a declared capture slot",
                ));
            }
            if descriptor.header.captures > captures {
                return Err(planner_error(
                    "abi descriptor declares a capture slot its origin does not have",
                ));
            }

            let slots = slot_slice(&self.slots, descriptor.slots)?;

            // ⭐ `C1`/`AC-2` — **no implicit caller-environment tail.** A frame is
            // exactly its declared parameters, its declared captures, and the
            // fixed convention slots. Any additional slot is a suffix of
            // something the origin did not declare, which is the dependence on
            // caller depth this node exists to remove.
            let expected = (parameters as usize)
                .checked_add(captures as usize)
                .and_then(|total| total.checked_add(CONVENTION_SLOTS.len()))
                .ok_or_else(|| planner_capacity_error("abi frame slot count exhausted"))?;
            if slots.len() > expected {
                return Err(planner_error(
                    "abi frame carries an implicit caller-environment tail",
                ));
            }
            if slots.len() < expected {
                return Err(planner_error(
                    "abi frame is missing a declared or convention slot",
                ));
            }

            validate_slot_run(slots, parameters, captures, definitions[ordinal])?;

            // The header is derived from the slots, so it must agree with them.
            let derived = frame_header(slots, parameters, captures)?;
            if descriptor.header != derived {
                return Err(planner_error(
                    "abi frame header is not derived from its own slot run",
                ));
            }
        }

        reject_imported_capture_edges(plane, sources, &definitions)?;
        self.validate_boundary_layouts(plane, sources, edges)?;
        self.validate_declaration_call_layouts(plane, sources, edges)?;
        Ok(())
    }

    /// Every declaration-call edge that targets a closure declaration agrees
    /// with that declaration's separately derived callable frame.
    ///
    /// The edge supplies owner and target identity; the target occurrence
    /// independently supplies provenance and arity. A closure-shaped
    /// declaration that retained the zero-input scheduling-entry ABI therefore
    /// fails here before any function is declared.
    fn validate_declaration_call_layouts(
        &self,
        plane: &SemanticPlane,
        sources: &[SemanticSourceSeed],
        edges: &[StaticEdge],
    ) -> Result<(), CraneliftBackendError> {
        for (_, callee, callee_origin, _) in plane.declaration_call_edges(edges)? {
            let descriptor = self.descriptors.get(callee.0 as usize).ok_or_else(|| {
                planner_error(
                    "declaration call callee is not forward-declared in the abi plane",
                )
            })?;
            if descriptor.origin != callee_origin {
                return Err(planner_error(
                    "declaration call target and callee descriptor disagree on origin",
                ));
            }
            let seed = source_for(sources, callee_origin)?;
            let SemanticSourceKind::Expression(shape) = seed.source else {
                return Err(planner_error(
                    "declaration call target is not a source expression",
                ));
            };
            let is_closure =
                matches!(shape, RuntimeExprShape::Closure | RuntimeExprShape::LexicalClosure);
            match (is_closure, descriptor.definition) {
                (
                    true,
                    AbiUnitDefinition::TransparentDeclarationClosure {
                        defining_origin,
                        provenance,
                    },
                ) => {
                    if defining_origin != callee_origin {
                        return Err(planner_error(
                            "declaration closure descriptor names the wrong defining occurrence",
                        ));
                    }
                    if provenance != closure_provenance(seed.source)? {
                        return Err(planner_error(
                            "declaration closure descriptor has the wrong capture provenance",
                        ));
                    }
                    let (parameters, captures) =
                        declared_arity(plane, sources, descriptor.definition)?;
                    if descriptor.header.parameters != parameters
                        || descriptor.header.captures != captures
                    {
                        return Err(planner_error(
                            "declaration closure call edge disagrees with the callable input layout",
                        ));
                    }
                }
                (true, _) => {
                    return Err(planner_error(
                        "closure declaration call target has no callable declaration definition",
                    ));
                }
                (false, AbiUnitDefinition::SchedulingEntry { .. }) => {}
                (
                    false,
                    AbiUnitDefinition::TransparentDeclarationClosure { .. }
                    | AbiUnitDefinition::ClosureBody { .. },
                ) => {
                    return Err(planner_error(
                        "non-closure declaration call target has a closure callable definition",
                    ));
                }
            }
        }
        Ok(())
    }

    /// **`D5` — every dynamic edge agrees on caller/callee LAYOUT.**
    ///
    /// For each `StaticBody` boundary, the caller-side signature is compared
    /// against the callee descriptor **field by field and slot by slot** — kind,
    /// carrier, ownership, storage owner, width, alignment and ordinal — not
    /// merely that the boundary lands on the right frame.
    ///
    /// ⛔ Target identity is a *different* property and is checked elsewhere;
    /// conflating the two is the defect this method exists to repair.
    fn validate_boundary_layouts(
        &self,
        plane: &SemanticPlane,
        sources: &[SemanticSourceSeed],
        edges: &[StaticEdge],
    ) -> Result<(), CraneliftBackendError> {
        for signature in boundary_signatures(plane, sources, edges)? {
            let descriptor = self
                .descriptors
                .get(signature.callee.0 as usize)
                .ok_or_else(|| {
                    planner_error(
                        "static body edge callee is not forward-declared in the abi plane",
                    )
                })?;

            // The provenance the graph says, against the provenance the
            // descriptor recorded.
            let AbiUnitDefinition::ClosureBody {
                defining_origin,
                provenance,
            } = descriptor.definition
            else {
                return Err(planner_error(
                    "static body edge callee is not a closure-body unit",
                ));
            };
            if defining_origin != signature.defining_origin {
                return Err(planner_error(
                    "boundary signature and callee descriptor disagree on the defining occurrence",
                ));
            }
            if provenance != signature.provenance {
                return Err(planner_error(
                    "boundary signature and callee descriptor disagree on capture provenance",
                ));
            }

            // ⭐ The independent axis: the caller-side capture count against the
            // callee's declared capture slots.
            if descriptor.header.captures != signature.captures {
                return Err(planner_error(
                    "boundary signature and callee descriptor disagree on the transferred capture \
                     count",
                ));
            }
            if descriptor.header.parameters != signature.parameters {
                return Err(planner_error(
                    "boundary signature and callee descriptor disagree on the transferred \
                     parameter count",
                ));
            }

            // The full transfer layout, slot by slot.
            let mut expected = Vec::new();
            push_slots(
                &mut expected,
                descriptor.definition,
                signature.parameters,
                signature.captures,
            )?;
            let actual = slot_slice(&self.slots, descriptor.slots)?;
            if actual != expected.as_slice() {
                return Err(planner_error(
                    "boundary signature and callee descriptor disagree on the transfer slot layout",
                ));
            }
        }
        Ok(())
    }
}

/// **`D5` — the per-boundary ABI reference, derived CALLER-side from the graph.**
///
/// ⛔ **An earlier revision of this node deleted its edge-agreement check and
/// claimed the property was enforced by composition: `B2O` gives
/// `functions[callee].planned_node == edge.to`, this plane gives
/// `descriptors[i].planned_node == functions[i].planned_node`, therefore
/// `descriptors[callee].planned_node == edge.to`. That conclusion is TRUE and it
/// is NOT layout agreement.** It establishes *target identity* — that the
/// boundary lands on the callee's frame entry — and says nothing about parameter
/// count, capture count, slot kinds, carriers, widths, alignment, ownership, or
/// storage owner on the transfer. Two frames can agree on which one is being
/// entered and disagree about every slot in it.
///
/// ⇒ What follows is the actual check: a **signature derived from the caller
/// side of the boundary**, compared field-by-field against the callee's
/// descriptor.
///
/// ⭐ **Why this is not tautological, stated per axis rather than in general:**
///
/// | axis | caller-side source | callee-side source | independent? |
/// |---|---|---|---|
/// | captures | the graph — capture **child origins** for a lexical closure, `CaptureSymbol` **atoms** for a seed closure | the recorded `capture_slots` field | ⭐ **yes** — different encodings, written by different code paths |
/// | provenance | the defining occurrence's `RuntimeExprShape` | the descriptor's recorded `AbiUnitDefinition` | ⭐ **yes** — derived vs recorded |
/// | parameters | `ParamName` atom count | `ParamName` atom count | ⚠ **no** — same source, so this axis is a consistency check, not corroboration |
///
/// ⚠ The parameter row is stated as a limitation rather than left to look like
/// coverage. It cannot disagree, and a reader must not count it as a third
/// independent witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct AbiBoundarySignature {
    pub(super) edge: StaticEdgeId,
    pub(super) callee: PredeclaredFunctionId,
    pub(super) defining_origin: StaticOriginId,
    pub(super) provenance: AbiCaptureProvenance,
    pub(super) parameters: u32,
    pub(super) captures: u32,
}

/// Derives one boundary signature per `StaticBody` edge, from the graph alone.
fn boundary_signatures(
    plane: &SemanticPlane,
    sources: &[SemanticSourceSeed],
    edges: &[StaticEdge],
) -> Result<Vec<AbiBoundarySignature>, CraneliftBackendError> {
    let mut signatures = Vec::new();
    for edge in edges {
        if edge.kind != EdgeKind::StaticBody {
            continue;
        }
        let defining_origin = StaticOriginId(edge.from.0);
        let seed = source_for(sources, defining_origin)?;
        let provenance = closure_provenance(seed.source)?;

        let (parameters, capture_atoms) = occurrence_atom_counts(plane, defining_origin)?;
        // ⭐ The independent capture count, taken from the CALLER side.
        let captures = match provenance {
            // A lexical closure's captures are planned syntax children, laid out
            // after the body child.
            AbiCaptureProvenance::Lexical => u32::try_from(
                lexical_capture_origins(plane, defining_origin)?.len(),
            )
            .map_err(|_| planner_capacity_error("boundary capture count exhausted"))?,
            // A seed closure's captures are interned symbols, one atom each.
            AbiCaptureProvenance::Seed => capture_atoms,
        };

        let callee_owner = plane
            .descriptors
            .get(edge.to.0 as usize)
            .map(|descriptor| descriptor.owner)
            .ok_or_else(|| planner_error("static body edge target has no semantic descriptor"))?;
        let SemanticOwner::Function(callee) = callee_owner else {
            return Err(planner_error("static body edge targets a shared exit"));
        };

        signatures.push(AbiBoundarySignature {
            edge: edge.id,
            callee,
            defining_origin,
            provenance,
            parameters,
            captures,
        });
    }
    Ok(signatures)
}

/// `(ParamName count, CaptureSymbol count)` for one occurrence's own atoms.
fn occurrence_atom_counts(
    plane: &SemanticPlane,
    origin: StaticOriginId,
) -> Result<(u32, u32), CraneliftBackendError> {
    let operands = occurrence_operands(plane, origin)?;
    let count = |kind: SemanticAtomKind| -> Result<u32, CraneliftBackendError> {
        u32::try_from(operands.iter().filter(|atom| atom.kind == kind).count())
            .map_err(|_| planner_capacity_error("occurrence atom count exhausted"))
    };
    Ok((
        count(SemanticAtomKind::ParamName)?,
        count(SemanticAtomKind::CaptureSymbol)?,
    ))
}

/// Checks the slot run is in canonical kind order with the declared carriers.
///
/// ⛔ Exhaustive over `AbiSlotKind` with no `_ =>`: a new slot kind must be
/// placed in the layout order explicitly.
fn validate_slot_run(
    slots: &[AbiSlot],
    parameters: u32,
    captures: u32,
    definition: AbiUnitDefinition,
) -> Result<(), CraneliftBackendError> {
    let capture_carrier = match definition {
        AbiUnitDefinition::SchedulingEntry { .. } => None,
        AbiUnitDefinition::TransparentDeclarationClosure { provenance, .. }
        | AbiUnitDefinition::ClosureBody { provenance, .. } => Some(provenance.carrier()),
    };

    for (position, slot) in slots.iter().enumerate() {
        let position = u32::try_from(position)
            .map_err(|_| planner_capacity_error("abi slot position exhausted"))?;
        let (expected_kind, expected_carrier, expected_ordinal) = if position < parameters {
            (AbiSlotKind::Parameter, AbiCarrier::ValueWord, position)
        } else if position < parameters + captures {
            let carrier = capture_carrier.ok_or_else(|| {
                planner_error("scheduling entry unit declares captures, which it cannot have")
            })?;
            (AbiSlotKind::Capture, carrier, position - parameters)
        } else {
            let index = (position - parameters - captures) as usize;
            let (kind, carrier) = CONVENTION_SLOTS.get(index).copied().ok_or_else(|| {
                planner_error("abi frame carries an implicit caller-environment tail")
            })?;
            (kind, carrier, 0)
        };

        if slot.kind != expected_kind {
            return Err(planner_error("abi frame slot is not in canonical kind order"));
        }
        if slot.carrier != expected_carrier {
            return Err(planner_error(
                "abi frame slot does not carry its kind's declared carrier",
            ));
        }
        if slot.ordinal != expected_ordinal {
            return Err(planner_error("abi frame slot is not positional in its kind run"));
        }
        // `D2` — every slot carries a declared kind, width, alignment and
        // ownership mode, and each is the carrier's own declaration rather than
        // an independently recorded value that could drift from it.
        if slot.ownership != slot.carrier.ownership()
            || slot.storage_owner != slot.carrier.storage_owner()
            || slot.width_bytes != slot.carrier.width_bytes()
            || slot.align_bytes != slot.carrier.align_bytes()
        {
            return Err(planner_error(
                "abi frame slot does not declare its carrier's width, alignment, ownership and \
                 storage owner",
            ));
        }
    }
    Ok(())
}
