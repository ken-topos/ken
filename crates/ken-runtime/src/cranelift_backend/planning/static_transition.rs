//! Factored, pre-emission native transition planner.
//!
//! Node code identity is `(transition kind, static node id)` and edge code
//! identity is `(edge kind, static edge id)`. Dynamic environment,
//! continuation, cleanup, source, and affine state travels as constant-width
//! IDs into hash-consed persistent stores.

mod semantic_ir;

use std::collections::{BTreeMap, BTreeSet};

use super::{
    backend, unsupported, BackendFailure, CraneliftBackendError, RuntimeDeclaration,
    RuntimeDeclarationKind,
};
use crate::RuntimeExpr;
use semantic_ir::{build_semantic_plane, SemanticMaterialArena, SemanticPlane, SemanticSourceSeed};

pub(in crate::cranelift_backend) use semantic_ir::StaticOriginId;

pub(super) const MAX_HELPERS_PER_STATIC_SOURCE: usize = 8;

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

#[derive(Clone)]
pub(in crate::cranelift_backend) struct StaticTransitionPlan {
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
    max_environment_depth: u32,
    max_continuation_depth: u32,
    max_path_depth: u32,
    max_cleanup_depth: u32,
    max_affine_depth: u32,
    max_source_return_depth: u32,
    source_return_resume_nodes: usize,
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
struct BoundaryB1Census {
    opcode_vocabulary: usize,
    distinct_origins: usize,
    ir_records: usize,
    semantic_edges: usize,
    helper_definitions: usize,
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

struct Planner {
    plan: StaticTransitionPlan,
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

impl Planner {
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
                semantic: SemanticPlane::default(),
                root_occurrence: None,
                declaration_occurrences: BTreeMap::new(),
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
        expr: &RuntimeExpr,
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
        expr: &RuntimeExpr,
        children: &[StaticOriginId],
    ) -> Result<(), CraneliftBackendError> {
        let seed = SemanticSourceSeed::expression(
            node,
            expr,
            children,
            &mut self.plan.semantic_material,
        )?;
        self.plan.semantic_sources.push(seed);
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
        expressions: &[&RuntimeExpr],
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
        bodies: &[(&RuntimeExpr, usize)],
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
                occurrence
                    .ok_or_else(|| planner_error("case position has no planned occurrence"))
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
        expr: &RuntimeExpr,
        ctx: PlanContext,
        successor: StaticNodeId,
        exit_kind: EdgeKind,
        ordinal: u32,
    ) -> Result<PlannedExpr, CraneliftBackendError> {
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
                let node = self.expression_node(TransitionKind::Evaluate, owner, frame, expr, &[])?;
                self.edge(node.entry, self.trap_terminal, EdgeKind::Trap)?;
                Ok(node)
            }
            RuntimeExpr::Value(_)
            | RuntimeExpr::Var(_)
            | RuntimeExpr::DeclarationRef { .. }
            | RuntimeExpr::ImportedDeclarationRef { .. } => {
                let node = self.expression_node(TransitionKind::Evaluate, owner, frame, expr, &[])?;
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
                let node = self.expression_node(
                    TransitionKind::Evaluate,
                    owner,
                    frame,
                    expr,
                    &children,
                )?;
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
                let node = self.expression_node(
                    TransitionKind::Evaluate,
                    owner,
                    frame,
                    expr,
                    &children,
                )?;
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

    fn finish(mut self) -> Result<StaticTransitionPlan, CraneliftBackendError> {
        self.plan.semantic = build_semantic_plane(
            &self.plan.nodes,
            &self.plan.edges,
            &self.plan.semantic_sources,
            &self.plan.semantic_material,
        )?;
        self.plan.validate()?;
        Ok(self.plan)
    }
}

impl StaticTransitionPlan {
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
            &self.semantic_sources,
            &self.semantic_material,
        )?;
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
            helper_definitions: self.semantic.functions.len(),
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

pub(in crate::cranelift_backend) fn plan_static_transition_graph(
    entry: &RuntimeExpr,
    declarations: &BTreeMap<&str, &RuntimeDeclaration>,
) -> Result<StaticTransitionPlan, CraneliftBackendError> {
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
    planner.plan.root_occurrence = Some(root.occurrence);
    for (symbol, declaration) in declarations {
        if let RuntimeDeclarationKind::Transparent { body } = &declaration.kind {
            let planned =
                planner.plan_expr(body, context, planner.terminal, EdgeKind::Continue, 0)?;
            planner.plan.entries.push(planned.entry);
            // A declaration body is its own planned source occurrence, so its
            // occurrence origin is reachable by name. Two entries under one
            // symbol would make that lookup ambiguous, which is a planner bug
            // rather than an input condition.
            if planner
                .plan
                .declaration_occurrences
                .insert((*symbol).to_owned(), planned.occurrence)
                .is_some()
            {
                return Err(planner_error(
                    "transparent declaration planned more than one entry origin",
                ));
            }
        }
    }
    planner.finish()
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

#[cfg(test)]
mod tests {
    use super::semantic_ir::{
        build_semantic_plane, DenseRange, RuntimeExprShape, SemanticAtomKind,
        SemanticOperandElement, SemanticSourceKind, StaticOriginId,
    };
    use super::*;
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

    fn nested_resource_bracket(depth: usize) -> RuntimeExpr {
        if depth == 0 {
            return unit();
        }
        let body = nested_resource_bracket(depth - 1);
        let release = RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "FS".to_string(),
                operation: ken_host::HostOpV1::BufferFreeze,
                capability: None,
                args: vec![RuntimeExpr::Var(0)],
            }),
            cases: vec![
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Err".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Trap(trap("release failed")),
                },
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Ok".to_string(),
                    binders: 1,
                    body: unit(),
                },
            ],
            default: trap("release result"),
        };
        let bracket = RuntimeExpr::ComputationalMatch {
            scrutinee: Box::new(RuntimeExpr::Construct {
                constructor: "ctor:fixture::Bracket::Scope".to_string(),
                args: vec![RuntimeExpr::LexicalClosure {
                    captures: Vec::new(),
                    params: vec!["buffer".to_string()],
                    body: Box::new(body),
                }],
            }),
            cases: vec![RuntimeComputationalMatchCase {
                constructor: "ctor:fixture::Bracket::Scope".to_string(),
                argument_binders: 1,
                recursive_positions: vec![0],
                body: RuntimeExpr::Let {
                    value: Box::new(RuntimeExpr::Call {
                        callee: Box::new(RuntimeExpr::Var(0)),
                        args: vec![unit()],
                    }),
                    body: Box::new(release),
                },
            }],
            default: trap("bracket scope"),
        };
        RuntimeExpr::Match {
            scrutinee: Box::new(RuntimeExpr::Effect {
                family: "FS".to_string(),
                operation: ken_host::HostOpV1::BufferAllocate,
                capability: None,
                args: vec![RuntimeExpr::Value(RuntimeValue::Int(1.into()))],
            }),
            cases: vec![
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Err".to_string(),
                    binders: 1,
                    body: RuntimeExpr::Trap(trap("allocate failed")),
                },
                RuntimeMatchCase {
                    constructor: "ctor:prelude::Result::Ok".to_string(),
                    binders: 1,
                    body: bracket,
                },
            ],
            default: trap("allocate result"),
        }
    }

    fn census(depth: usize) -> BoundaryACensus {
        let expr = nested_resource_bracket(depth);
        plan_static_transition_graph(&expr, &BTreeMap::new())
            .map(|plan| plan.census())
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
                 ir_records={} semantic_edges={} helper_definitions={} \
                 definitions_per_origin={} operand_elements={} duplicate_origins={} \
                 clone_count={} max_definitions_per_origin={} fixed_k={} \
                 descriptor_bytes={} program_bytes={} record_bytes={} \
                 operand_element_bytes={} capture_layout_bytes={} capture_slot_bytes={} \
                 ruled_child_bytes={} function_bytes={}",
                row.opcode_vocabulary,
                row.distinct_origins,
                row.ir_records,
                row.semantic_edges,
                row.helper_definitions,
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
                "helper_definitions",
                semantic_values(&semantic, |row| row.helper_definitions),
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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();
        let mut reversed_sources = plan.semantic_sources.clone();
        reversed_sources.reverse();
        let reordered = build_semantic_plane(
            &plan.nodes,
            &plan.edges,
            &reversed_sources,
            &plan.semantic_material,
        )
        .unwrap();
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
        let changed = build_semantic_plane(
            &changed_frames,
            &plan.edges,
            &reversed_sources,
            &plan.semantic_material,
        )
        .unwrap();
        assert_eq!(
            changed, plan.semantic,
            "dynamic activation state changed semantic programs or bodies"
        );
        assert_eq!(plan.semantic.descriptors.len(), plan.nodes.len());
        assert_eq!(plan.semantic.functions.len(), plan.nodes.len());
        assert_eq!(plan.semantic.ruled_children.len(), plan.edges.len());
    }

    #[test]
    fn boundary_b1_negative_controls_fail_at_named_semantic_artifacts() {
        // Promise class: durable mutation proof.
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();

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
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic program arena contains a post-origin clone")
        );

        let mut superlinear_material = plan.semantic.clone();
        let deliberate_square = plan.nodes.len().checked_mul(plan.nodes.len()).unwrap();
        superlinear_material.operands.extend((0..deliberate_square).map(|ordinal| {
            SemanticOperandElement {
                kind: SemanticAtomKind::LocalIndex,
                content: DenseRange { start: 0, len: 0 },
                payload: ordinal as u64,
            }
        }));
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

    /// Two `Let` occurrences of identical shape and counts whose positional
    /// children are different occurrences.
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
        let plan =
            plan_static_transition_graph(&equal_shaped_atom_fixture(), &BTreeMap::new()).unwrap();
        let vars = nodes_of_shape(&plan, RuntimeExprShape::Var);
        assert_eq!(vars.len(), 2, "fixture must hold two equal-shaped occurrences");

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
            swapped
                .operands
                .swap(first.operands.start as usize + offset, second.operands.start as usize + offset);
        }
        assert_eq!(
            swapped
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic material record is not occurrence-exact for its origin")
        );

        // The same swap on positional children of an equal-shaped pair.
        let child_plan =
            plan_static_transition_graph(&equal_shaped_child_fixture(), &BTreeMap::new()).unwrap();
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
        let plan = plan_static_transition_graph(
            &equal_shaped_primitive_pair(left, right),
            &BTreeMap::new(),
        )
        .unwrap();
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

    #[test]
    fn boundary_b1r_atom_content_must_stay_inside_the_closed_name_arena() {
        // Promise class: durable mutation proof. B2a decodes atom content, so a
        // structurally well-formed atom whose span escapes the arena, or whose
        // bytes are not the ones the walk interned, is undecodable material.
        let plan = plan_static_transition_graph(
            &equal_shaped_primitive_pair(
                crate::RuntimePartiality::Total,
                crate::RuntimePartiality::CheckedTrap {
                    obligation: "ken.bytes.at.inBounds".to_string(),
                },
            ),
            &BTreeMap::new(),
        )
        .unwrap();

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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();
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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();
        let mut duplicated = plan.semantic.clone();
        duplicated.records[1].origin = duplicated.records[0].origin;
        assert_eq!(
            duplicated
                .validate(
                    &plan.nodes,
                    &plan.edges,
                    &plan.semantic_sources,
                    &plan.semantic_material,
                )
                .unwrap_err(),
            planner_error("semantic program is not the exhaustive lowering of its source")
        );
    }

    #[test]
    fn boundary_a_nested_resource_brackets_n3_through_n7_are_closed_and_affine() {
        // Promise class: durable invariant. Counts remain relational; this
        // does not freeze a current literal or infer an exponent from points.
        let rows = (3..=7).map(census).collect::<Vec<_>>();
        for (depth, row) in (3..=7).zip(&rows) {
            eprintln!(
                "RT_NATIVE_FNSPLIT_BOUNDARY_A n={depth} static_nodes={} edges={} \
                 planned_helpers={} persistent_store_nodes={} evidence_records={} \
                 fixed_k={} key_bytes={} frame_bytes={} store_node_bytes={} \
                 key_schemas={} frame_schemas={} store_schemas={} env_depth={} \
                 continuation_depth={} path_depth={} cleanup_depth={} affine_depth={} \
                 source_return_depth={} source_return_resume_nodes={}",
                row.static_nodes,
                row.edges,
                row.planned_helpers,
                row.persistent_store_nodes,
                row.out_of_line_evidence_records,
                row.max_helpers_per_static_source,
                row.helper_key_bytes,
                row.activation_frame_bytes,
                row.store_node_bytes,
                row.helper_key_schemas,
                row.frame_schemas,
                row.store_node_schemas,
                row.max_environment_depth,
                row.max_continuation_depth,
                row.max_path_depth,
                row.max_cleanup_depth,
                row.max_affine_depth,
                row.max_source_return_depth,
                row.source_return_resume_nodes,
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
                "source_return_resume_nodes",
                values(&rows, |r| r.source_return_resume_nodes),
            ),
            ("helper_key_bytes", values(&rows, |r| r.helper_key_bytes)),
            (
                "activation_frame_bytes",
                values(&rows, |r| r.activation_frame_bytes),
            ),
            ("store_node_bytes", values(&rows, |r| r.store_node_bytes)),
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
                "fixed_k",
                (|r: &BoundaryACensus| r.max_helpers_per_static_source)
                    as fn(&BoundaryACensus) -> usize,
            ),
            ("helper_key_bytes", |r: &BoundaryACensus| r.helper_key_bytes),
            ("activation_frame_bytes", |r: &BoundaryACensus| {
                r.activation_frame_bytes
            }),
            ("store_node_bytes", |r: &BoundaryACensus| r.store_node_bytes),
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
            .all(|(row, depth)| row.source_return_resume_nodes == depth));
        assert!(rows.iter().all(|row| {
            row.out_of_line_evidence_records == row.edges
                && row.max_environment_depth <= row.persistent_store_nodes as u32
                && row.max_continuation_depth <= row.persistent_store_nodes as u32
                && row.max_path_depth <= row.persistent_store_nodes as u32
        }));
    }

    #[test]
    fn planner_invariant_failures_have_compiler_bug_attribution() {
        // Promise class: durable invariant. These distinct planner
        // self-consistency failures are compiler bugs. The former fixed-K capacity arm is
        // not input-reachable because fixed K is a structural planner invariant.
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();

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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();
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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();
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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();
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
        let plan = plan_static_transition_graph(&unit(), &BTreeMap::new()).unwrap();

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
        let plan =
            plan_static_transition_graph(&nested_resource_bracket(3), &BTreeMap::new()).unwrap();

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

    impl StaticTransitionPlan {
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
        digest.push_str(&format!("nodes={} edges={}", plan.nodes.len(), plan.edges.len()));
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
        let plan = plan_static_transition_graph(&computational, &BTreeMap::new())
            .expect("plannable");
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
        let plan = plan_static_transition_graph(&RuntimeExpr::Var(0), &declarations)
            .expect("plannable");
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
}
