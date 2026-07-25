//! Closed semantic-IR representation referenced by the static transition plan.
//!
//! Boundary A owns scheduling and authority. This plane owns only semantic
//! programs for already-planned nodes; static edges remain body-free transfer
//! contracts.

use super::{
    planner_capacity_error, planner_error, CraneliftBackendError, StaticEdge, StaticEdgeId,
    StaticNode, StaticNodeId, TransitionKind,
};
use crate::{RuntimeExpr, RuntimeIntV1, RuntimeTrap, RuntimeTrapCode, RuntimeValue, Sign};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct StaticOriginId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct SemanticProgramId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct CaptureLayoutId(pub(super) u32);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct PredeclaredFunctionId(pub(super) u32);

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
    pub(super) fn expression(
        planned_node: StaticNodeId,
        expr: &RuntimeExpr,
        children: &[StaticNodeId],
        arena: &mut SemanticMaterialArena,
    ) -> Result<Self, CraneliftBackendError> {
        let atom_start = arena.atoms.len();
        let child_start = arena.child_origins.len();
        emit_expression_atoms(expr, arena)?;
        for child in children {
            arena.child_origins.push(StaticOriginId(child.0));
        }
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
    /// Symbol atoms: `content` spans the interned name bytes.
    PrimitiveSymbol,
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
    pub(super) function: PredeclaredFunctionId,
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

/// The sole semantic-definition constructor. It positions seeds by their
/// already-planned node ID, visits each node once, visits each edge once, and
/// flattens each variable source/capture collection once.
pub(super) fn build_semantic_plane(
    nodes: &[StaticNode],
    edges: &[StaticEdge],
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
        let function = PredeclaredFunctionId(planned_node.0);

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

        plane.functions.push(PredeclaredFunction {
            id: function,
            planned_node,
            origin,
            program,
        });
        plane.descriptors.push(SemanticDescriptor {
            planned_node,
            origin,
            program,
            capture_layout,
            function,
            ruled_children: child_range,
        });
    }
    plane.validate(nodes, edges, sources, arena)?;
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
    pub(super) fn validate(
        &self,
        nodes: &[StaticNode],
        edges: &[StaticEdge],
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
        if self.programs.len() != nodes.len()
            || self.records.len() != nodes.len()
            || self.capture_layouts.len() != nodes.len()
            || self.functions.len() != nodes.len()
        {
            return Err(planner_error(
                "semantic program arena contains a post-origin clone",
            ));
        }

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
            let function = self.functions[position];
            let source = source_by_node[position];
            if descriptor.planned_node != node
                || descriptor.origin != source.origin
                || descriptor.program != SemanticProgramId(node.0)
                || descriptor.capture_layout != CaptureLayoutId(node.0)
                || descriptor.function != PredeclaredFunctionId(node.0)
                || program.id != SemanticProgramId(node.0)
                || layout.id != CaptureLayoutId(node.0)
                || function.id != PredeclaredFunctionId(node.0)
                || function.planned_node != node
                || function.origin != source.origin
                || function.program != SemanticProgramId(node.0)
            {
                return Err(planner_error(
                    "node, descriptor, program, capture layout, and function are not positional",
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

fn positioned_sources(
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
            arena.push_named(SemanticAtomKind::PrimitiveSymbol, &primitive.symbol, 0)?;
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
