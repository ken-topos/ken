//! Closed semantic-IR representation referenced by the static transition plan.
//!
//! Boundary A owns scheduling and authority. This plane owns only semantic
//! programs for already-planned nodes; static edges remain body-free transfer
//! contracts.

use super::{
    planner_capacity_error, planner_error, CraneliftBackendError, StaticEdge, StaticEdgeId,
    StaticNode, StaticNodeId, TransitionKind,
};
use crate::{RuntimeExpr, RuntimeValue};

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
/// later activation exists. Variable source material is counted once here; the
/// builder flattens exactly that many positional elements into its operand
/// arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticSourceSeed {
    pub(super) planned_node: StaticNodeId,
    pub(super) origin: StaticOriginId,
    pub(super) source: SemanticSourceKind,
    pub(super) source_material_elements: u32,
    pub(super) capture_slots: u32,
}

impl SemanticSourceSeed {
    pub(super) fn expression(
        planned_node: StaticNodeId,
        expr: &RuntimeExpr,
    ) -> Result<Self, CraneliftBackendError> {
        Ok(Self {
            planned_node,
            origin: StaticOriginId(planned_node.0),
            source: SemanticSourceKind::Expression(RuntimeExprShape::of(expr)),
            source_material_elements: source_material_elements(expr)?,
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

    pub(super) const fn control(planned_node: StaticNodeId, transition: TransitionKind) -> Self {
        Self {
            planned_node,
            origin: StaticOriginId(planned_node.0),
            source: SemanticSourceKind::Control(transition),
            source_material_elements: 0,
            capture_slots: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticOperandElement {
    pub(super) source_ordinal: u32,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub(super) struct SemanticRecord {
    pub(super) opcode: SemanticOpcode,
    pub(super) origin: StaticOriginId,
    pub(super) operands: DenseRange,
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
    pub(super) operands: Vec<SemanticOperandElement>,
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

        let operand_range = DenseRange::at_end(
            &plane.operands,
            source.source_material_elements as usize,
            "semantic operand",
        )?;
        plane.operands.extend(
            (0..source.source_material_elements)
                .map(|source_ordinal| SemanticOperandElement { source_ordinal }),
        );

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
    plane.validate(nodes, edges, sources)?;
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
        if self.operands.len() != expected_operands {
            return Err(planner_error(
                "semantic operand arena exceeds the one-visit source-material budget",
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
            if record.operands.len != source.source_material_elements {
                return Err(planner_error(
                    "semantic record does not own its exact source-material range",
                ));
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

    #[cfg(test)]
    pub(super) fn all_out_of_line_operand_elements(&self) -> usize {
        self.operands.len() + self.capture_slots.len() + self.ruled_children.len()
    }
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
