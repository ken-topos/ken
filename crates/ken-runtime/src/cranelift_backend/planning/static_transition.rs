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
use std::collections::{BTreeMap, BTreeSet};

use super::{
    backend, unsupported, BackendFailure, CraneliftBackendError, RuntimeDeclaration,
    RuntimeDeclarationKind,
};
use crate::RuntimeExpr;
use abi::{build_abi_plane, AbiPlane};
use semantic_ir::{
    build_semantic_plane, SemanticMaterialArena, SemanticPlane, SemanticSourceKind,
    SemanticSourceSeed,
};

// ⭐ `D1`'s capability surface. The two identity types cross into
// `crate::cranelift_backend` so `lowering` can hold and compare them; ⛔
// `SemanticPlane`, `SemanticMaterialArena` and the `names` arena stay on the
// `use` above, visible only inside this planner. Widening either of those to
// serve a consumer is the move `§2d` forbids.
pub(in crate::cranelift_backend) use semantic_ir::{
    ConstructorIdentity, FieldIdentity, StaticOriginId,
};

pub(super) const MAX_HELPERS_PER_STATIC_SOURCE: usize = 8;

#[cfg(test)]
thread_local! {
    static ACTIVE_RECURSIVE_LOWERING_FRAMES: Cell<usize> = const { Cell::new(0) };
    static MAX_RECURSIVE_LOWERING_FRAMES: Cell<usize> = const { Cell::new(0) };
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
                semantic: SemanticPlane::default(),
                root_occurrence: None,
                declaration_occurrences: BTreeMap::new(),
                source_occurrences: Vec::new(),
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
        let seed =
            SemanticSourceSeed::expression(node, expr, children, &mut self.plan.semantic_material)?;
        self.plan.semantic_sources.push(seed);
        self.record_source_occurrence(node, expr)
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

    fn finish(mut self) -> Result<StaticTransitionPlan<'src>, CraneliftBackendError> {
        self.plan.semantic = build_semantic_plane(
            &self.plan.nodes,
            &self.plan.edges,
            &self.plan.entries,
            &self.plan.semantic_sources,
            &self.plan.semantic_material,
        )?;
        // `B2R` — the representation contract is built from the owner partition
        // the line above just validated, and it fails **before** anything is
        // emitted. It is deliberately not deferred to lowering: a contract that
        // is only checked at emission time cannot be a *pre*-emission gate.
        self.plan.abi = build_abi_plane(
            &self.plan.semantic,
            &self.plan.nodes,
            &self.plan.semantic_sources,
            &self.plan.edges,
            &self.plan.entries,
        )?;
        self.plan.validate()?;
        Ok(self.plan)
    }
}

impl<'src> StaticTransitionPlan<'src> {
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

    /// The artifact-static constructor identity of a `Construct` occurrence —
    /// the producer side of [`Self::case_constructor_identity`] (`D2`).
    pub(in crate::cranelift_backend) fn constructor_symbol_identity(
        &self,
        origin: StaticOriginId,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        self.semantic.constructor_symbol_identity(origin)
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
        )?;
        self.validate_source_occurrence_table()?;
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

pub(in crate::cranelift_backend) fn plan_static_transition_graph<'src>(
    entry: &'src RuntimeExpr,
    declarations: &BTreeMap<&str, &'src RuntimeDeclaration>,
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
    planner.plan.root_occurrence = Some(root.occurrence);
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
    use super::abi::{AbiCarrier, AbiSlot, AbiSlotKind};
    use crate::cranelift_backend::surface::NativeSeedEnvironment;
    use crate::RuntimeGroundValue;
    use super::semantic_ir::{
        build_semantic_plane, DenseRange, PredeclaredFunctionId, RuntimeExprShape, SemanticAtomKind,
        SemanticOperandElement, SemanticOwner, SemanticSourceKind, StaticOriginId,
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
        let reordered = build_semantic_plane(
            &plan.nodes,
            &plan.edges,
            &plan.entries,
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
            &plan.entries,
            &reversed_sources,
            &plan.semantic_material,
        )
        .unwrap();
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

        let mut duplicated = plan.semantic.clone();
        let victim = duplicated
            .operands
            .iter()
            .find(|atom| atom.content.len > 0)
            .map(|atom| atom.content)
            .expect("fixture has an atom with out-of-line content");
        let start = victim.start as usize;
        let bytes = duplicated.names[start..start + victim.len as usize].to_vec();

        // Append a byte-identical second copy and repoint one atom at it: same
        // spelling, different span. Nothing else about the plane changes.
        let copy = DenseRange {
            start: u32::try_from(duplicated.names.len()).unwrap(),
            len: victim.len,
        };
        duplicated.names.extend_from_slice(&bytes);
        let atom = duplicated
            .operands
            .iter_mut()
            .find(|atom| atom.content == victim)
            .expect("the victim atom is still present");
        atom.content = copy;

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
            let SemanticOwner::Function(unit) =
                plan.semantic.descriptors[node.id.0 as usize].owner
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
        append_edge(&mut crossed, root_entry, static_body[0].1, EdgeKind::Continue);
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
                *id != root_entry
                    && plan.semantic.descriptors[id.0 as usize].owner == root_unit
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
            b2o_err(
                &handed_back,
                &plan.nodes,
                &plan.edges,
                &plan.entries,
                &plan
            ),
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
        (plan.semantic.functions.len(), cross_owner, intra_owner, shared_exit + other)
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
            )
            .expect_err("AC-1: dropping a descriptor must be refused");
        // ⛔ The EXACT failure, not `is_err()`. A control that reddens does not
        // confirm which detector caught it, and `is_err()` would stay green if
        // some unrelated law started firing first.
        assert!(
            format!("{err:?}").contains("not exact for the function unit partition"),
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
        tailed.descriptors.last_mut().expect("descriptors").slots.len += 1;
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
        skewed_plane.child_origins.insert(
            (extra.start + extra.len) as usize,
            borrowed,
        );
        skewed_plane.records[record].child_origins.len += 1;
        let layout_arm = match lexical_plan.abi.validate(
            &skewed_plane,
            &lexical_plan.nodes,
            &lexical_plan.semantic_sources,
            &lexical_plan.edges,
            &lexical_plan.entries,
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
                 descriptor population is not exact for the function unit \
                 partition\"))"
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
            (
                "String",
                RuntimeGroundValue::String("seeded".to_string()),
            ),
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
}
