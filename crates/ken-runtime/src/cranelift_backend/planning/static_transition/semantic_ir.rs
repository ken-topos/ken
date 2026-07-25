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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct SemanticProgramId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct CaptureLayoutId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct PredeclaredFunctionId(pub(super) u32);

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
}

impl SemanticMaterialArena {
    fn intern(&mut self, bytes: &[u8]) -> Result<DenseRange, CraneliftBackendError> {
        let span = DenseRange::at_end(&self.names, bytes.len(), "semantic name")?;
        self.names.extend_from_slice(bytes);
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
/// Traversal is over non-`StaticBody` edges only, so a `StaticBody` edge is the
/// one and only owner boundary. The two shared exits are never owned and never
/// traversed through — they have no outgoing edges by construction
/// (`static_transition.rs:1258`).
fn partition_function_units(
    nodes: &[StaticNode],
    edges: &[StaticEdge],
    entries: &[StaticNodeId],
) -> Result<OwnershipPartition, CraneliftBackendError> {
    let (terminal, trap_terminal) = shared_exits(nodes)?;

    // Seed class 1 is `entries`; seed class 2 is the `StaticBody` targets. The
    // three ways this can be malformed get three distinct failures on purpose —
    // one composite "the seeds are fine" check is discharged by any one of them
    // holding, so it could not distinguish the mutations AC-5 requires.
    #[derive(Clone, Copy, Eq, PartialEq)]
    enum SeedClass {
        SchedulingEntry,
        StaticBodyTarget,
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
        *slot = Some(SeedClass::SchedulingEntry);
        seeds.push(*entry);
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
            Some(SeedClass::SchedulingEntry) => {
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

    let mut outgoing = vec![Vec::new(); nodes.len()];
    for edge in edges {
        if edge.kind == EdgeKind::StaticBody {
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

    let partition = partition_function_units(nodes, edges, entries)?;

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

    plane.validate(nodes, edges, entries, sources, arena)?;
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
        let descriptor = self
            .descriptors
            .get(parent.0 as usize)
            .ok_or_else(|| planner_error("static origin is outside the semantic descriptors"))?;
        if descriptor.origin != parent {
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
        if record.origin != parent {
            return Err(planner_error(
                "semantic record origin is not its preallocated positional identity",
            ));
        }
        plane_slice(
            &self.child_origins,
            record.child_origins,
            "semantic child origin",
        )?
        .get(position)
        .copied()
        .ok_or_else(|| planner_error("static origin has no child at that source position"))
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
    fn validate_function_units(
        &self,
        nodes: &[StaticNode],
        edges: &[StaticEdge],
        entries: &[StaticNodeId],
    ) -> Result<(), CraneliftBackendError> {
        let partition = partition_function_units(nodes, edges, entries)?;

        // D5 prediction 1: the unit population is exactly the two seed classes.
        // Predicted from the design on 2026-07-25, before measuring:
        // `functions.len() == entries.len() + count(StaticBody edges)`.
        let static_body_edges = edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::StaticBody)
            .count();
        let expected_units = entries
            .len()
            .checked_add(static_body_edges)
            .ok_or_else(|| planner_capacity_error("function unit count exhausted"))?;
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
                if to_unit == from_unit {
                    return Err(planner_error(
                        "static body edge does not cross a function unit boundary",
                    ));
                }
                if self.functions[to_unit.0 as usize].planned_node != edge.to {
                    return Err(planner_error(
                        "static body edge target is not its function unit's seed",
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
        sources: &[SemanticSourceSeed],
        arena: &SemanticMaterialArena,
    ) -> Result<(), CraneliftBackendError> {
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
        self.validate_function_units(nodes, edges, entries)?;

        let source_by_node = positioned_sources(nodes, sources)?;
        let expected_operands = source_by_node.iter().try_fold(0usize, |total, source| {
            total
                .checked_add(source.source_material_elements as usize)
                .ok_or_else(|| planner_capacity_error("semantic operand count exhausted"))
        })?;
        // D4.4 — one-visit affine bound over the WHOLE material: this
        // occurrence's atoms plus its child references. The budget is unchanged
        // by the atom/child partition, so a superlinear arena still fails here.
        let expected_child_origins = source_by_node.iter().try_fold(0usize, |total, source| {
            total
                .checked_add(source.children.len as usize)
                .ok_or_else(|| planner_capacity_error("semantic child origin count exhausted"))
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
        let expected_capture_slots = source_by_node.iter().try_fold(0usize, |total, source| {
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
            let source = source_by_node[position];
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
