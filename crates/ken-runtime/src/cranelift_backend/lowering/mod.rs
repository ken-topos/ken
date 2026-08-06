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

// `RT-FNSPLIT-B2F` `D1`/`D2` — the target code-unit population. A sibling of
// `core` rather than a region inside it: `core.rs` is the module whose recursive
// whole-configuration authority `D6` removes, and putting the replacement
// population in the same file would make the census that measures the removal
// unable to distinguish the two.
pub(in crate::cranelift_backend) mod units;

// `RT-FNSPLIT-B2F` `D3` — the artifact-static seed material. A sibling of
// `units` rather than a region inside it, because the two mint **different
// populations**: `units` mints code (`Theta(n)` in the program) and this mints
// data (`Theta(|seed environment|)`, independent of the program). ⛔ Folding
// them into one file would put two growth axes behind one census row, which is
// the population conflation this node has already had to repair three times.
pub(in crate::cranelift_backend) mod seed_material;

// --- external dependencies -------------------------------------------------
pub(in crate::cranelift_backend) use std::collections::{BTreeMap, BTreeSet};

// `RT-FNSPLIT-B2V` `D4`. Re-exported at facade scope like every other import in
// this header so the `tests` subtree inherits the names.
pub(in crate::cranelift_backend) use crate::boundary_value::{
    BoundaryClass, BoundaryReferentOwner, BoundaryTag, BOUNDARY_ERR_BOUNDS, BOUNDARY_OK,
};

pub(in crate::cranelift_backend) use cranelift_codegen::flowgraph::ControlFlowGraph;
pub(in crate::cranelift_backend) use cranelift_codegen::ir::{
    types, AbiParam, Block, FuncRef, Function, InstBuilder, MemFlags, StackSlotData, StackSlotKind,
    UserFuncName,
};
pub(in crate::cranelift_backend) use cranelift_codegen::verify_function;
pub(in crate::cranelift_backend) use cranelift_frontend::{
    FunctionBuilder, FunctionBuilderContext,
};
pub(in crate::cranelift_backend) use cranelift_module::{FuncId, Linkage, Module};

// --- crate root ------------------------------------------------------------
pub(in crate::cranelift_backend) use crate::{
    RuntimeDeclaration, RuntimeDeclarationKind, RuntimeExpr, RuntimeGroundValue, RuntimePartiality,
    RuntimePrimitive, RuntimeSymbol, RuntimeTrap, RuntimeTrapCode, RuntimeValue,
};

// --- sibling backend modules, named at their OWNERS -----------------------
// §10.3 DAG: `lowering support -> compiled, planning, surface`.
//
// These are SEMANTIC support edges, not namespace wiring for `core`:
// `Lowering::emit_result` returns and constructs `compiled::ResultDecoder`,
// and `source_case_has_no_checked_control_markers` calls the `planning`
// collectors and constructs `CheckedOrientedMarkerSets`. Acyclic, because
// `compiled` and `planning` each depend only on `surface` and neither imports
// lowering. No `artifact` / `artifact::api` edge, and no reverse edge.
// (Architect `evt_8vhe6rd6r80c`; the landed §10.3 line said support -> surface
// only, which these four imports and two production bodies refute.)
pub(in crate::cranelift_backend) use super::compiled::{CompiledModule, ResultDecoder};
pub(in crate::cranelift_backend) use super::planning::{
    collect_checked_oriented_markers, collect_checked_subcontinuation_frames,
    plan_static_transition_graph_with_symbols, validate_oriented_subcontinuation_transport,
    AbiCaptureProvenance, AbiCarrier, AbiFrameHeader, AbiOwnership, AbiProcessParameter,
    AbiRootIngress, AbiSlot, AbiSlotKind, AbiStorageOwner, AbiUnitDefinition,
    expected_capture_slot,
    CheckedOrientedMarkerSets, ConstructorIdentity, ContinuationCallIdentity, ContinuationCallView,
    DeclarationCallTargetClass,
    ContinuationContextId, ContinuationEmissionOwner,
    ContinuationInputView, ContinuationOrdinaryEnvelopeRole, ContinuationResultEdge,
    ContinuationAvailabilityViews, ContinuationEnvironmentClaim, ContinuationFrameIdentity,
    ContinuationSourceCoordinate,
    ContinuationSourceSlotAuthority,
    ContinuationSpecializationId,
    ContinuationUnitView, EmittableCallKind, EmittableUnit, FieldIdentity, JoinPlanToken,
    PlannedReferentLifetime,
    host_effect_seat_contract_of, EffectSeatNeed, EffectSeatOperation, EffectSeatPhase,
    EffectSeatSlot, PlannedEffectSeat,
    AggregateOccurrenceId, PlannedAggregateAllocation, PlannedAggregateShape,
    SynthesizedAggregateNode, SynthesizedAggregatePath, SynthesizedAggregateRoot, PlannedAggregateOwnership,
    JoinResultRepresentation, PredeclaredFunctionId, StaticOriginId, StaticTransitionPlan,
    verify_current_lexical_availability, verify_predeclared_entry_frame_membership,
    SynthesizedConstructorRole, SynthesizedFixedConstructorRole,
};
#[cfg(test)]
pub(in crate::cranelift_backend) use super::planning::{
    plan_static_transition_graph, with_last_io_error_role_omitted, ScaleBPlanCensus,
};
pub(in crate::cranelift_backend) use super::surface::{
    backend, backend_module, unsupported, BackendFailure, CraneliftBackendError,
    NativeSeedEnvironment,
};

// `#[cfg(test)]`-only: an unconditional `use` of this breaks the non-test
// build, which the test build cannot show you.
#[cfg(test)]
pub(in crate::cranelift_backend) use crate::RuntimeMatchCase;

/// One completed FunctionizedUnits emission row for RT-SCALE-B.
///
/// The fixed native-Int and boundary-value graphs, every unit body, and the
/// public root adapter all record through the same function seam.  Imports and
/// test-only probes never call that seam.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(in crate::cranelift_backend) struct ScaleBEmissionMetrics {
    pub(in crate::cranelift_backend) plan: ScaleBPlanCensus,
    pub(in crate::cranelift_backend) authority_functionized: bool,
    pub(in crate::cranelift_backend) emitted_helpers: usize,
    pub(in crate::cranelift_backend) production_functions: usize,
    pub(in crate::cranelift_backend) native_int_functions: usize,
    pub(in crate::cranelift_backend) boundary_value_functions: usize,
    pub(in crate::cranelift_backend) recursive_descent_roots: usize,
    pub(in crate::cranelift_backend) functionized_root_adapters: usize,
    pub(in crate::cranelift_backend) functionized_unit_bodies: usize,
    pub(in crate::cranelift_backend) clif_instructions: usize,
    pub(in crate::cranelift_backend) clif_bytes: usize,
    pub(in crate::cranelift_backend) total_dfg_values: usize,
    pub(in crate::cranelift_backend) total_instructions: usize,
    pub(in crate::cranelift_backend) total_blocks: usize,
}

#[cfg(test)]
#[derive(Clone, Debug)]
struct ScaleBEmissionAttempt {
    metrics: ScaleBEmissionMetrics,
    complete: bool,
}

#[cfg(test)]
thread_local! {
    static SCALE_B_EMISSION_ATTEMPT:
        std::cell::RefCell<Option<ScaleBEmissionAttempt>> =
            const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
fn scale_b_reset_emission_attempt() {
    SCALE_B_EMISSION_ATTEMPT.with(|attempt| *attempt.borrow_mut() = None);
}

#[cfg(test)]
fn scale_b_begin_emission_attempt(plan: &StaticTransitionPlan<'_>, authority_functionized: bool) {
    SCALE_B_EMISSION_ATTEMPT.with(|attempt| {
        *attempt.borrow_mut() = Some(ScaleBEmissionAttempt {
            metrics: ScaleBEmissionMetrics {
                plan: plan.scale_b_census(),
                authority_functionized,
                emitted_helpers: 0,
                production_functions: 0,
                native_int_functions: 0,
                boundary_value_functions: 0,
                recursive_descent_roots: 0,
                functionized_root_adapters: 0,
                functionized_unit_bodies: 0,
                clif_instructions: 0,
                clif_bytes: 0,
                total_dfg_values: 0,
                total_instructions: 0,
                total_blocks: 0,
            },
            complete: false,
        });
    });
}

#[cfg(test)]
fn scale_b_finish_emission_attempt() {
    SCALE_B_EMISSION_ATTEMPT.with(|attempt| {
        if let Some(attempt) = attempt.borrow_mut().as_mut() {
            attempt.complete = true;
        }
    });
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn scale_b_last_emission_metrics() -> Option<ScaleBEmissionMetrics>
{
    SCALE_B_EMISSION_ATTEMPT.with(|attempt| {
        attempt
            .borrow()
            .as_ref()
            .filter(|attempt| attempt.complete)
            .map(|attempt| attempt.metrics.clone())
    })
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum ScaleBEmitter {
    NativeInt,
    BoundaryValue,
    RecursiveDescentRoot,
    FunctionizedRootAdapter,
    FunctionizedUnitBody,
}

#[cfg(test)]
fn scale_b_record_function(function: &Function, emitter: ScaleBEmitter) {
    SCALE_B_EMISSION_ATTEMPT.with(|attempt| {
        let mut attempt = attempt.borrow_mut();
        let Some(attempt) = attempt.as_mut() else {
            return;
        };
        if attempt.complete {
            return;
        }
        let instructions = function.dfg.num_insts();
        attempt.metrics.production_functions += 1;
        attempt.metrics.clif_instructions += instructions;
        attempt.metrics.clif_bytes += function.display().to_string().len();
        attempt.metrics.total_dfg_values += function.dfg.num_values();
        attempt.metrics.total_instructions += instructions;
        attempt.metrics.total_blocks += function.dfg.num_blocks();
        match emitter {
            ScaleBEmitter::NativeInt => attempt.metrics.native_int_functions += 1,
            ScaleBEmitter::BoundaryValue => attempt.metrics.boundary_value_functions += 1,
            ScaleBEmitter::RecursiveDescentRoot => {
                attempt.metrics.recursive_descent_roots += 1;
            }
            ScaleBEmitter::FunctionizedRootAdapter => {
                attempt.metrics.functionized_root_adapters += 1;
            }
            ScaleBEmitter::FunctionizedUnitBody => {
                attempt.metrics.functionized_unit_bodies += 1;
                attempt.metrics.emitted_helpers += 1;
            }
        }
    });
}

#[cfg(test)]
pub(crate) fn scale_b_record_native_int(function: &Function) {
    scale_b_record_function(function, ScaleBEmitter::NativeInt);
}

#[cfg(test)]
pub(crate) fn scale_b_record_boundary_value(function: &Function) {
    scale_b_record_function(function, ScaleBEmitter::BoundaryValue);
}

#[cfg(test)]
fn scale_b_record_recursive_descent_root(function: &Function) {
    scale_b_record_function(function, ScaleBEmitter::RecursiveDescentRoot);
}

#[cfg(test)]
fn scale_b_record_functionized_root_adapter(function: &Function) {
    scale_b_record_function(function, ScaleBEmitter::FunctionizedRootAdapter);
}

#[cfg(test)]
fn scale_b_record_unit_body(function: &Function) {
    scale_b_record_function(function, ScaleBEmitter::FunctionizedUnitBody);
}

// ⭐ The admitted set now has ONE definition, in planning, because the seat
// population is derived from it there. This is a namespace re-export so both
// the emitter's admission check and the planner's population read the same
// list; a local copy could disagree with it silently.
use crate::cranelift_backend::planning::CRANELIFT_HOST_EFFECT_CONSUMERS_V1;
#[cfg(test)]
use crate::cranelift_backend::planning::{set_effect_seat_plan_mutation, EffectSeatPlanMutation};

/// **`D7` — perturbations of one VISIT, as distinct from perturbations of the
/// planned population.**
///
/// ⛔ These act on the emitter's side of the authority. The population
/// mutations ([`EffectSeatPlanMutation`]) act on the planner's. Keeping them
/// separate is what lets a control say which side a gate is actually reading —
/// a single enum spanning both would let a green row be attributed to either.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum EffectSeatVisitMutation {
    Exact,
    /// Omit one slot per visit, alternating which one across successive visits.
    /// ⭐ The masking discriminator: the omissions are COMPLEMENTARY, so a
    /// ledger that accumulated claims per occurrence rather than per visit would
    /// see a complete union and accept.
    OmitComplementary,
    /// Claim the visit's first slot a second time.
    DuplicateWithinVisit,
    /// Drop the open group instead of closing it.
    DiscardGroup,
    /// Report the opposite phase of the one the operand is actually in.
    PerturbObservedPhase,
    /// Drop one COMMITTED group after every body close has passed and before
    /// the whole-pass close. ⭐ The only way to ask whether the whole-pass
    /// backstop is still doing work now that the body close catches the same
    /// condition earlier — every ordinary route to a discarded group is now
    /// stopped before it can reach the backstop.
    DropCommittedGroupBeforeGlobalClose,
}

#[cfg(test)]
thread_local! {
    static EFFECT_SEAT_VISIT_MUTATION: std::cell::Cell<EffectSeatVisitMutation> =
        const { std::cell::Cell::new(EffectSeatVisitMutation::Exact) };
    /// Which visit this is, so `OmitComplementary` can alternate.
    static EFFECT_SEAT_VISIT_INDEX: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_effect_seat_visit_mutation(
    mutation: EffectSeatVisitMutation,
) {
    EFFECT_SEAT_VISIT_MUTATION.with(|cell| cell.set(mutation));
    EFFECT_SEAT_VISIT_INDEX.with(|cell| cell.set(0));
}

#[cfg(test)]
fn effect_seat_visit_mutation() -> EffectSeatVisitMutation {
    EFFECT_SEAT_VISIT_MUTATION.with(std::cell::Cell::get)
}

#[cfg(test)]
fn effect_seat_next_visit_index() -> usize {
    EFFECT_SEAT_VISIT_INDEX.with(|cell| {
        let index = cell.get();
        cell.set(index + 1);
        index
    })
}
/// **`RT-DECL-CLOSURE-PORT` `D7` — the two framed lowering-closure mutations.**
///
/// ⛔ Both name a REMOVAL of something this release added, not a corruption of
/// an input. That is what makes them closure evidence: each restores the state
/// the frame says must refuse, and the control asserts the refusal is the exact
/// one the frame names rather than any refusal.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EffectSeatDispatchMutation {
    Exact,
    /// Delete the carried arm of the capacity seat, leaving the specialized
    /// read the whole route -- the state that produced the exact
    /// `264 -> 262 / position 1` refusal.
    RemoveCarriedCapacityArm,
    /// Restore the eager all-argument projection in reply synthesis, so every
    /// argument is demanded as a template whether or not a synthesized node
    /// declares a use for it.
    RestoreBulkConversion,
}

#[cfg(test)]
thread_local! {
    static EFFECT_SEAT_DISPATCH_MUTATION: std::cell::Cell<EffectSeatDispatchMutation> =
        const { std::cell::Cell::new(EffectSeatDispatchMutation::Exact) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_effect_seat_dispatch_mutation(
    mutation: EffectSeatDispatchMutation,
) {
    EFFECT_SEAT_DISPATCH_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
fn effect_seat_dispatch_mutation() -> EffectSeatDispatchMutation {
    EFFECT_SEAT_DISPATCH_MUTATION.with(std::cell::Cell::get)
}

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
/// ⭐ **The phase of an induction hypothesis's residual, with the carried word
/// when there is one** (`RT-FNSPLIT-C1` `AC-C4`, `§2g-i`).
///
/// ⚠ **The raw `ir::Value` is recorded rather than a `CarriedBoundaryWord`, and
/// that is deliberate.** Recording the struct would require giving it
/// `PartialEq` — which would hand *production* a compile-time way to ask
/// whether two carried values are the same word. ⛔ That is exactly the
/// capability `CarriedBoundaryWord`'s emptiness exists to deny (see its doc
/// comment: every question about a carried value is answered by an emitted
/// helper at runtime). ⇒ The observation stays test-only in its **capability**,
/// not merely in its `#[cfg]`.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Px8jResidualPhase {
    /// The carried phase, together with the exact boundary word held.
    Carried(cranelift_codegen::ir::Value),
    /// The specialized phase. ⛔ Recorded as a phase only: `§2g-i`'s clause
    /// constrains the **carried** arm, and a specialized residual is a
    /// different route entirely.
    Specialized,
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
        /// ⭐⭐ **The RESIDUAL edge — `§2g-i`'s actual sentence.**
        ///
        /// ⛔ Every other field of this event observes the **metadata** edge:
        /// `origin`, `cursor` and `sibling_position` all say *who owns* the
        /// hypothesis. ⛔ **None of them says what is INSIDE it.** Substituting
        /// one projected child for another leaves all three byte-identical —
        /// which is precisely the compile-preserving evasion that defeated this
        /// control (`children[position]` → `children[0]`, `runtime-qa` on
        /// `b8d2922f`).
        residual: Px8jResidualPhase,
    },
    /// ⭐ **A child projected out of a carried scrutinee, as
    /// `(position, word)`.**
    ///
    /// ⭐⭐ **This is the INDEPENDENT ORACLE for the residual edge, and the
    /// independence is structural, not a matter of care.** It is written by the
    /// projection loop itself, keyed on **that loop's own counter** — so it
    /// records which field each word actually came from, and it is written
    /// *before* any selection among the children happens. ⇒ A test may name a
    /// position on its **fixture's** authority and ask this record which word
    /// that field produced, without ever reading the index the production path
    /// selected with.
    CarrierFieldProjection {
        path: Px8jProducerPath,
        position: usize,
        word: cranelift_codegen::ir::Value,
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
    /// **`RT-DECL-CLOSURE-PORT` `D6a` — the CARRIED answer route was emitted.**
    ///
    /// ⛔⛔ A separate event from [`Self::DeforestedAnswerResumed`], and the
    /// separation is the point. That one is recorded while lowering the
    /// **specialized** branch, where the scrutinee is a compile-time
    /// `Lowered::Constructor` and `actual_constructor` can name it. On the
    /// carried branch there is no such name to record and nothing at compile
    /// time knows which way a runtime word will go — so reusing that event here
    /// would dress a compile-time fact as runtime evidence.
    ///
    /// ⚠ This event says exactly one thing: **the carried route was emitted**
    /// into this frame's return case. It is an *emission* discriminator. The
    /// runtime half is the linked artifact's exit status, and the two are
    /// reported as a pair — neither substitutes for the other.
    CarriedAnswerRouteEmitted {
        checked_frame_id: u64,
        return_constructor: RuntimeSymbol,
    },
    FinalProcessObjectTrap {
        trap: RuntimeTrap,
    },
    /// **`RT-DECL-CLOSURE-PORT` `D6a` — EXACT trap provenance, recorded at the
    /// seat that emits the trap rather than at the seat that decided it.**
    ///
    /// ⭐ The frame requires the disabled checked-answer route to be proven
    /// through *"the planner trap identity at the unit `TrapWord` and root
    /// propagation seat"*, and states why: **the generic process `-4` string
    /// alone is not exact provenance.** That is not a stylistic preference.
    /// `-4` is the root adapter's single process-trap sentinel — it is the
    /// **same word for every trap in the program**, so a row asserting it
    /// cannot tell the checked-`ITree` default apart from any other trap the
    /// fixture could have reached, including one reached by a bug.
    ///
    /// ⚠ Both words are recorded, and the pair is the point:
    ///
    /// - `planned_identity` — what `StaticTransitionPlan::trap_identity` issued
    ///   for this exact `RuntimeTrap`. The authority.
    /// - `emitted_word` — what this seat actually put in the instruction
    ///   stream. Read according to `seat`.
    ///
    /// ⛔ Recording only the planned word would make this an assertion that the
    /// planner agrees with itself: `TrapIdentityMutation::{Zero,Substitute}`
    /// perturbs the *emitted* word and would leave such an event untouched.
    /// Recording the pair is what makes that existing mutation reach this row.
    PlannedTrapEmitted {
        trap: RuntimeTrap,
        seat: PlannedTrapSeat,
        planned_identity: i64,
        emitted_word: i64,
    },
    /// **`RT-DECL-CLOSURE-PORT` `D6a` — the ROOT PROPAGATION seat.**
    ///
    /// Recorded where a caller reads a callee unit's `TrapWord` and forwards
    /// it. ⛔ **Deliberately carries no identity**, and that is not an omission:
    /// the word here is a `stack_load`, a *runtime* value, so the compiler
    /// genuinely does not know which trap is flowing through. Recording a
    /// planned identity at this seat would be inventing one.
    ///
    /// ⭐ What the compiler *does* know is the authority, and therefore whether
    /// the callee's exact word survives the hop. That is the fact this event
    /// reports, and it is what makes the frame's rule legible rather than
    /// merely asserted: the identity is exact at [`PlannedTrapSeat::UnitTrapWord`]
    /// and is **collapsed** at [`PlannedTrapSeat::RootProcessSentinel`], which is
    /// exactly why the process `-4` string cannot stand in for provenance.
    UnitTrapWordPropagated {
        seat: PlannedTrapSeat,
        /// Whether the callee's exact trap word reaches the next frame
        /// recoverably. `false` only for the identity-free process sentinel.
        identity_preserved: bool,
    },
}
/// The three authorities [`Lowering::emit_current_trap`] can emit a trap under.
///
/// ⚠ **They are not interchangeable, and only the first two carry identity.**
/// `RootProcessSentinel` collapses every trap to one word by construction —
/// naming the seat is what stops a reader mistaking that collapse for a
/// measurement.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlannedTrapSeat {
    /// The generated unit's `TrapWord` lane. ⭐ The exact planner-issued
    /// identity word is stored here, so this is the seat where the checked
    /// default's provenance is genuinely exact.
    UnitTrapWord,
    /// The root adapter's process-object lane. ⛔ Identity-free: every trap
    /// emits the `-4` process sentinel.
    RootProcessSentinel,
    /// The root adapter's value lane, which shifts and tags the exact identity
    /// into a boundary trap token.
    RootTrapToken,
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
/// Record which child a carried scrutinee's projection loop produced at each
/// position. ⭐ See `Px8jSourceTraceEvent::CarrierFieldProjection` — this is the
/// residual edge's independent oracle.
#[cfg(test)]
fn px8j_record_carrier_field_projection(
    path: Px8jProducerPath,
    position: usize,
    word: CarriedBoundaryWord,
) {
    px8j_record_source_event(Px8jSourceTraceEvent::CarrierFieldProjection {
        path,
        position,
        word: word.word,
    });
}

#[cfg(test)]
fn px8j_record_recursor_carrier(path: Px8jProducerPath, value: &LoweringOperand) {
    // ⭐ A trace probe, so it is total over both phases and takes neither
    // boundary: a carried operand is simply not a recursor carrier, which is the
    // same "nothing to record" answer any other non-recursor value gets.
    let LoweringOperand::Specialized(Lowered::ComputationalRecursorClosure {
        residual,
        invocation,
        ..
    }) = value
    else {
        return;
    };
    // ⭐ Total over both phases here too, and wildcard-free: the residual's
    // phase is part of what is observed, so an unclassifiable arm would be a
    // silent hole in exactly the edge this field exists to expose.
    let residual = match residual.as_ref() {
        LoweringOperand::Carried(word) => Px8jResidualPhase::Carried(word.word),
        LoweringOperand::Specialized(_) => Px8jResidualPhase::Specialized,
    };
    px8j_record_source_event(Px8jSourceTraceEvent::Carrier {
        path,
        origin: invocation.origin,
        cursor: invocation.resume_cursor,
        sibling_position: invocation.sibling_position,
        residual,
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
/// One source occurrence the lowering is working on: the expression the planner
/// walked, paired with the `StaticOriginId` the planner preallocated for it.
///
/// The pair exists so that an occurrence's static name travels **with** the term
/// rather than beside it, so a child's origin can be derived positionally as the
/// walk descends.
///
/// ⚠ Since `RT-FNSPLIT-B2A-S` the origin is **no longer provenance only** — for a
/// retained closure body it is the *selector*, and `retained_body_occurrence` is
/// the one place a term is recovered from one. Be precise about what is still
/// true, because the previous blanket claim here is now half wrong:
///
/// - it **does** select a retained closure body (that is the point of the unit);
/// - since `RT-CONTSPEC-ACTIVATE` `D3` it **also keys a collection**: the
///   producer `Construct` occurrence's origin is the first field of the
///   four-field causal selector that resolves a continuation call binding. The
///   earlier "never keys a collection" clause is retired, and it is retired
///   rather than narrowed because a reader who trusted it would look for the
///   binding lookup somewhere other than the origin;
/// - it still **never** alters a branch or reaches emitted code by comparison,
///   ordering, or arithmetic — the binding lookup is an exact equality on a
///   planner-issued identity, not a decision computed from an origin's value.
#[derive(Clone, Copy)]
struct SourceOccurrence<'a> {
    expr: &'a RuntimeExpr,
    static_origin: StaticOriginId,
}

/// An owned source occurrence, for the points where the lowering **clones** a
/// `RuntimeExpr` into a pending frame or a cloneable prefix template.
///
/// ⛔ **No longer a retained closure**: `RT-FNSPLIT-B2A-S` removed that, and
/// `Lowered::Closure`/`DeclarationClosure` now name their body by origin alone. The
/// source machine's in-flight frames still own their terms, and that is a forced
/// boundary rather than a leftover — `lower_source_forked_match` synthesizes a
/// `Trap` that exists nowhere in the source tree, so it has no planned occurrence
/// to be resolved from and cannot be represented by a tag.
///
/// ⭐ The pair is one value on purpose: `SourcePrefixTemplate` is `Clone`, and a
/// clone that copied the term while dropping its origin would silently
/// reintroduce exactly the recoverability vacancy the pair exists to close.
/// Making them one field makes that drop unspellable.
#[derive(Clone)]
struct OwnedSourceOccurrence {
    expr: RuntimeExpr,
    static_origin: StaticOriginId,
}

impl OwnedSourceOccurrence {
    /// Clones a borrowed occurrence into an owned one, carrying the origin in
    /// the same constructor as the clone.
    fn cloned(occurrence: SourceOccurrence<'_>) -> Self {
        Self {
            expr: occurrence.expr.clone(),
            static_origin: occurrence.static_origin,
        }
    }
}


/// **Everything that is resolved into ONE generated `Function` and is
/// meaningless in any other.**
///
/// ⛔⛔ **Nothing in here is portable across functions, and the three kinds fail
/// differently.** `FuncRef`, `GlobalValue` and `ir::Value` are all
/// function-scoped entity references in Cranelift:
///
/// | field kind | what it is | moving it to another function |
/// |---|---|---|
/// | `FuncRef`s, `SeedMaterialRefs`' `GlobalValue`s, `BoundaryCarrierRefs` | an identity **resolved into** a function | ⚠ must be **re-resolved** — the identity survives, the handle does not |
/// | `host_dispatch_context`, `native_int_arena` | a **result of this function's own dataflow** | ⛔ must be **re-derived** — there is no identity to re-resolve |
/// | `native_int_tags` | a map **keyed on `ir::Value`** | ⛔⛔ **silently aliases** |
///
/// ⭐ **The `native_int_tags` row is the dangerous one, and it is why this
/// struct exists.** Entity references restart per `Function` — `v0`, `v1`, …
/// are reused — so a tag map carried from one function into another answers a
/// lookup for an unrelated value **that happens to share the numeric handle**.
/// ⛔ No type error, no panic, no verifier complaint: a wrong tag on a path that
/// still compiles.
///
/// ⇒ ⭐ **`RT-FNSPLIT-B2F` `S6` gives each unit body its own function, so this
/// state must be partitioned by construction.** ⛔ A `reset()` someone has to
/// remember to call is the same defect with an extra step. Gathering the fields
/// under one name is what makes *"is this per-function state?"* answerable by
/// reading one struct instead of auditing a hundred-field one.
///
/// ⚠ **This is the identity-alias class in a third substrate** — after `B2O`
/// removed it from `SemanticDescriptor` and `D1` kept it out of `UnitBundle`,
/// here it is again in Cranelift's own entity references.
///
/// ⛔ **NOT CLAIMED: no control demonstrates the alias.** Producing one requires
/// the `S6` switch-over that does not exist yet, so this is a read of Cranelift's
/// entity scoping, not a measurement. ⚠ What *is* structural is that a second
/// function cannot silently inherit this state without a visible second
/// construction of this struct.
/// **`RT-FNSPLIT-B2F` `S6`** — the module-level identities every generated
/// function must resolve for itself, and the sole producer of a
/// [`FunctionLocalRefs`].
///
/// ⭐ **This is the OTHER side of the construction boundary, and naming it is
/// what makes the boundary checkable.** [`FunctionLocalRefs`] says what is *not*
/// portable; this says what *is*. A `FuncId` and a `DataId` are module-scoped:
/// they mean the same thing in every function of the artifact, and
/// [`Self::declare_in_func`] is the one operation that turns them into the
/// function-scoped handles a body can actually reference.
///
/// ⛔ **There is deliberately no way to build a `FunctionLocalRefs` in
/// production except through [`Self::declare_in_func`].** The root and every
/// future unit body therefore cannot drift: a helper added here is resolved into
/// *every* generated function or into none, and *"the root has a carrier ref and
/// the unit does not"* stops being expressible by forgetting to copy a line.
/// ⚠ The `#[cfg(test)]` fixtures build the struct directly and are the stated
/// exception; they emit into no module and hold `None` throughout.
///
/// **MEASURED:** every resolved-handle field of `FunctionLocalRefs` is produced
/// by one function, from module-scoped identities held here.
/// **CLAIMED:** any second generated function emits against the same helper
/// surface the root does.
/// **THE GAP:** ⛔ **there is no second generated function yet** — `S6`'s
/// switch-over is not landed, so today this has exactly one caller and the claim
/// is about a population of one. ⚠ It is *not* claimed that this suffices for a
/// unit body: the two `ir::Value` fields are **not** resolvable from here,
/// because they are results of a function's own dataflow rather than identities,
/// and `declare_in_func` leaves them `None` for each caller to derive.
///
/// **THE TRAP DATAFLOW BOUNDARY:** a root's closed [`TrapExitAuthority`] is
/// supplied at construction. A unit's `slots` value is loaded from that
/// function's own envelope, so its authority is absent until
/// [`FunctionLocalRefs::bind_unit_trap_frame`] binds the exact frame.
#[derive(Clone, Copy)]
struct ArtifactHelpers<'h> {
    seed_material: &'h seed_material::SeedMaterial,
    host_dispatch: Option<FuncId>,
    native_int: &'h crate::native_int_clif::NativeIntLocalFuncs,
    boundary_value_abi: &'h crate::boundary_value_clif::BoundaryLocalFuncs,
}

impl ArtifactHelpers<'_> {
    /// Resolve every module-level identity into `func`.
    ///
    /// ⛔ **Call this once per `Function`, and never reuse the result.** The
    /// returned handles are function-scoped; see [`FunctionLocalRefs`] for what
    /// each of the three kinds does when it crosses a function boundary.
    fn declare_in_func<M: Module>(
        self,
        module: &mut M,
        func: &mut Function,
        trap_exit: Option<TrapExitAuthority>,
    ) -> FunctionLocalRefs {
        FunctionLocalRefs {
            seed_material: self.seed_material.declare_in_func(module, func),
            host_dispatch: self
                .host_dispatch
                .map(|id| module.declare_func_in_func(id, func)),
            // ⛔ Dataflow results, not identities: `None` here is correct, and
            // each function derives its own from its entry block.
            host_dispatch_context: None,
            services_pointer: None,
            native_int_arena: None,
            // ⛔ Sourced by the activation-services record, which the `S6`/`D6`
            // reland introduces. Fail closed until then.
            boundary_arena: None,
            native_int_binop: Some(module.declare_func_in_func(self.native_int.binop, func)),
            native_int_compare: Some(module.declare_func_in_func(self.native_int.compare, func)),
            native_int_intern: Some(module.declare_func_in_func(self.native_int.intern, func)),
            native_int_narrow: Some(module.declare_func_in_func(self.native_int.narrow, func)),
            native_int_export: Some(module.declare_func_in_func(self.native_int.export, func)),
            native_int_export_parts: Some(
                module.declare_func_in_func(self.native_int.export_parts, func),
            ),
            // ⭐ The **one** exact-`Int` decoder, resolved here so the
            // region-limbed spill copies through the landed representation
            // instead of growing a second one.
            native_int_resolve: Some(module.declare_func_in_func(self.native_int.resolve, func)),
            // ⛔ Empty, never inherited. This is the map whose `ir::Value` keys
            // alias across functions; starting it empty per function is why the
            // two structs are separate types rather than one with a `reset()`.
            native_int_tags: BTreeMap::new(),
            unit_calls: BTreeMap::new(),
            worker_calls: BTreeMap::new(),
            raw_worker_calls: BTreeMap::new(),
            worker_templates: BTreeMap::new(),
            context_calls: BTreeMap::new(),
            defining_abi_operands: Vec::new(),
            generated_context_captures: None,
            continuation_calls: BTreeMap::new(),
            continuation_emissions: BTreeMap::new(),
            pending_composed_discharges: Vec::new(),
            composed_discharges: BTreeMap::new(),
            declaration_calls: BTreeMap::new(),
            trap_exit,
            terminal_result_origins: BTreeSet::new(),
            consumed_join_origins: BTreeSet::new(),
            dispositioned_join_origins: BTreeSet::new(),
            join_disposition_finalized: false,
            final_reachable_join_origins: BTreeSet::new(),
            materialized_join_blocks: BTreeMap::new(),
            emission_reachable_match_cases: BTreeMap::new(),
            boundary_carrier: Some(BoundaryCarrierRefs {
                class: module.declare_func_in_func(self.boundary_value_abi.class, func),
                tag: module.declare_func_in_func(self.boundary_value_abi.tag, func),
                field_count: module.declare_func_in_func(self.boundary_value_abi.field_count, func),
                field: module.declare_func_in_func(self.boundary_value_abi.field, func),
                record_field: module
                    .declare_func_in_func(self.boundary_value_abi.record_field, func),
                scalar: module.declare_func_in_func(self.boundary_value_abi.scalar, func),
                host_success: module
                    .declare_func_in_func(self.boundary_value_abi.host_success, func),
                host_payload: module
                    .declare_func_in_func(self.boundary_value_abi.host_payload, func),
                alloc: module.declare_func_in_func(self.boundary_value_abi.alloc, func),
                store_tag_id: module
                    .declare_func_in_func(self.boundary_value_abi.store_tag_id, func),
                store_scalar: module
                    .declare_func_in_func(self.boundary_value_abi.store_scalar, func),
                store_field: module.declare_func_in_func(self.boundary_value_abi.store_field, func),
                store_name: module.declare_func_in_func(self.boundary_value_abi.store_name, func),
                make_immediate: module
                    .declare_func_in_func(self.boundary_value_abi.make_immediate, func),
                store_int_tag: module
                    .declare_func_in_func(self.boundary_value_abi.store_int_tag, func),
                store_bytes_len: module
                    .declare_func_in_func(self.boundary_value_abi.store_bytes_len, func),
                store_byte: module.declare_func_in_func(self.boundary_value_abi.store_byte, func),
                store_int_limbs: module
                    .declare_func_in_func(self.boundary_value_abi.store_int_limbs, func),
                store_int_limb: module
                    .declare_func_in_func(self.boundary_value_abi.store_int_limb, func),
                seal_int: module.declare_func_in_func(self.boundary_value_abi.seal_int, func),
                int_view: module.declare_func_in_func(self.boundary_value_abi.int_view, func),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrapExitAuthority {
    UnitFrame {
        slots: cranelift_codegen::ir::Value,
        trap_offset: i32,
    },
    Root {
        process_sentinel: bool,
        source_authorized: bool,
    },
}

/// **`RT-DECL-CLOSURE-PORT` `D5a`** -- the continuation-input operands one
/// enclosing specialization passes across a retargeted worker call.
struct GeneratedContextCaptures {
    /// The exact worker body origin whose call carries this suffix.
    worker_body_origin: StaticOriginId,
    /// The enclosing specialization's continuation inputs, in ordinal order.
    operands: Vec<LoweringOperand>,
}

struct FunctionLocalRefs {
    /// **`RT-FNSPLIT-B2F` `D3`** — the artifact-static seed material, resolved
    /// into this generated function.
    ///
    /// ⭐ Held **alongside** `Lowering::seed_env`, not instead of it, and the
    /// division is the point: `seed_env` answers *which* `RuntimeGroundValue` a
    /// symbol denotes — a compile-time question — while this answers *where the
    /// running artifact reads it from*. ⛔ Collapsing the two would either put a
    /// compilation-only borrow in the artifact (which does not typecheck) or
    /// re-fold the value into the instruction stream (which is the authority
    /// `D3` removes).
    seed_material: seed_material::SeedMaterialRefs,
    host_dispatch: Option<FuncRef>,
    host_dispatch_context: Option<cranelift_codegen::ir::Value>,
    services_pointer: Option<cranelift_codegen::ir::Value>,
    native_int_arena: Option<cranelift_codegen::ir::Value>,
    /// ⭐ **The BOUNDARY arena, and it is a different pointer from
    /// [`Self::native_int_arena`]** — Architect ruling via `evt_e300y2kjeb6k`.
    /// The two answer different questions and were wrongly merged into one
    /// field; see [`Lowering::carrier_arena`] for the retraction.
    ///
    /// ⚠ **`None` in production, because NOTHING PUBLISHES A BOUNDARY ARENA
    /// YET** — measured, not assumed: every `BoundaryRegion::reserve` /
    /// `reserve_persistent` call site in the crate is a test, so the activation
    /// owner the ruling assigns this to does not exist on either launcher path.
    /// ⇒ Every boundary-carrier call fails closed until then — ⭐ which is
    /// strictly better than the native arena it used to be handed silently.
    boundary_arena: Option<cranelift_codegen::ir::Value>,
    native_int_binop: Option<FuncRef>,
    native_int_compare: Option<FuncRef>,
    native_int_intern: Option<FuncRef>,
    native_int_narrow: Option<FuncRef>,
    native_int_export: Option<FuncRef>,
    native_int_export_parts: Option<FuncRef>,
    /// `(arena, tag, payload, out_view) -> status` — the sole exact-`Int`
    /// decoder, `ken_native_int_resolve_local`.
    native_int_resolve: Option<FuncRef>,
    native_int_tags: BTreeMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    /// The boundary-carrier helpers, made callable inside **this** generated
    /// function (`RT-FNSPLIT-C1` `D3`).
    ///
    /// ⛔ `FuncRef`s, not `FuncId`s — the ruling requires the helper IDs to be
    /// *"declared into each generated function as callable refs and actually
    /// called by all three routes"*, and a `FuncId` held here would be exactly
    /// the inert threading the node forbids: present, plausible, and never
    /// reaching an emitted call.
    boundary_carrier: Option<BoundaryCarrierRefs>,
    unit_calls: BTreeMap<StaticOriginId, units::DeclaredUnitCall>,
    /// **`D4`** -- this function's own static-worker call targets, keyed by
    /// exact body origin. Minted per generated function; a `FuncRef` here
    /// belongs to that function and is never copied to another.
    worker_calls: BTreeMap<StaticOriginId, units::DeclaredUnitCall>,
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D6b`** -- this function's own call targets
    /// for the **raw worker bodies**, keyed by exact body origin, as declared
    /// *before* any retarget.
    ///
    /// ⛔ A fifth map rather than a flag on `worker_calls`, because the two
    /// answer different questions for the **same** key. In a retargeted
    /// specialization `worker_calls[body]` has been overwritten with the
    /// generated context that executes `body`, so the raw callee is no longer
    /// reachable from that map at all -- and a
    /// [`StaticWorkerCallRoute::RawWorker`] binding for that same body still
    /// needs it. Keeping one map and choosing by route is impossible when one
    /// entry has been replaced; keeping both is the whole mechanism.
    ///
    /// ⛔ Same per-function discipline as every table here: minted by
    /// `WorkerTargets::declare_in_func` into *this* `Function`, never copied
    /// between functions, so no `FuncRef` crosses a boundary.
    ///
    /// ⚠ **A body absent here is a body with no emitted `Function`.** This map
    /// is declared from the *executable* population, so a body that a total
    /// retarget made template-only is legitimately missing. `call_static_worker`
    /// fails closed on the miss rather than falling back to `worker_calls` --
    /// a fallback would silently route a raw call through a generated context
    /// whose ABI expects a capture suffix this caller does not supply.
    raw_worker_calls: BTreeMap<StaticOriginId, units::DeclaredUnitCall>,
    /// **`RT-DECL-CLOSURE-PORT` `D5a` checkpoint 1** -- the RAW descriptor
    /// contract for every worker body, executable or template-only.
    ///
    /// ⛔ Separate from `worker_calls` on purpose, and carrying no `FuncRef`.
    /// `construct_static_worker_binding` validates identity and arity against
    /// **this**, while the call is emitted through `worker_calls`, which may
    /// have been retargeted to a generated context. Validating against the
    /// thing you are about to call would make the retarget invisible to the
    /// check; validating against the raw template is what "unchanged ordinary
    /// `fn2` ABI" actually means.
    worker_templates: BTreeMap<StaticOriginId, units::WorkerTemplate>,
    /// **`RT-DECL-CLOSURE-PORT` `D5a` checkpoint 4 step 1** -- this function's
    /// own call targets for the generated execution contexts, keyed by the
    /// planner's context identity.
    ///
    /// ⛔ A fourth map rather than an entry in `worker_calls` or `unit_calls`:
    /// a `ContinuationContextId` is its own identity domain, and keying a
    /// context by the body origin it executes is precisely the "reconstruct the
    /// binding from body origin" the ruling forbids. Minted per function; no
    /// `FuncRef` crosses a function.
    context_calls: BTreeMap<ContinuationContextId, units::DeclaredUnitCall>,
    /// **`RT-DECL-CLOSURE-PORT` `D5a` checkpoint 4 step 1b** -- this function's
    /// own ABI-slot operands, indexed by ABI position.
    ///
    /// The `Parameter` run followed by the `Capture` run, in the one slot walk
    /// that also builds the body's environment -- so index `i` here is ABI
    /// position `i` of the function being defined.
    ///
    /// ⭐ A retargeted carried invocation reads its context's capture suffix
    /// from here, at the **immediate slots** the planner assigned. It is stored
    /// rather than threaded because the call seam is six frames below the body
    /// walk; the alternative was passing an environment through every
    /// intermediate, which is how a seat ends up reading whichever environment
    /// happened to be nearest.
    ///
    /// ⛔ Reset per function, like every other field here: these are `ir::Value`
    /// operands of one `Function` and mean nothing in another.
    defining_abi_operands: Vec<LoweringOperand>,
    /// **`RT-DECL-CLOSURE-PORT` `D5a`** -- the operand suffix a **retargeted**
    /// worker call must append, and the one body origin it applies to.
    ///
    /// `None` in every function that calls raw worker units directly, which is
    /// every pre-`D5a` case. When present it is set by the enclosing
    /// specialization's own body, from that frame's Capture slots, so the
    /// operands are `ir::Value`s of *this* function and cannot be reused
    /// elsewhere -- the same per-function discipline `worker_calls` follows.
    ///
    /// ⛔ The body origin is retained beside the operands rather than being
    /// implicit: a suffix appended to the wrong worker call would be a silent
    /// arity error at a frame that happened to be big enough.
    generated_context_captures: Option<GeneratedContextCaptures>,
    /// **`RT-CONTSPEC-ACTIVATE` `D3`** -- this Function's own `FuncRef` per
    /// causal token it owns, keyed by the complete four-field identity.
    /// Minted into this `Function`; never passed across functions.
    continuation_calls: BTreeMap<ContinuationCallIdentity, units::DeclaredUnitCall>,
    /// **`RT-CONTSPEC-ACTIVATE` `4b`** -- the exact `Inst` this Function emitted
    /// for each causal token, recorded at the `builder.ins().call` that produced
    /// it.
    ///
    /// ⭐ This is an **anchor, not an answer**. It records *where* a call was
    /// emitted, never *what* it calls: the callee is decoded back out of the
    /// finished CLIF at
    /// [`Lowering::verify_emitted_continuation_calls`]. Recording the target
    /// here instead would make the gate compare the value it was handed with
    /// the value it was handed.
    ///
    /// ⛔ An entry exists only because a call instruction exists, so a token
    /// that was claimed and never called leaves no entry -- which is the whole
    /// reason the emission set is kept separately from the claim ledger.
    continuation_emissions: BTreeMap<ContinuationCallIdentity, cranelift_codegen::ir::Inst>,
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8j`** — composed discharges this function
    /// has CLAIMED but not yet verified.
    ///
    /// ⛔ A claim is not a discharge. Entries land here at the source-machine
    /// seat, after the raw-worker call is emitted and its result has returned
    /// to unchanged source-machine control, and they move into
    /// [`Self::composed_discharges`] only once the finished CLIF has been
    /// consulted. Nothing outside `verify_recorded_composed_discharges` may
    /// read this as a discharge.
    pending_composed_discharges: Vec<PendingComposedDischarge>,
    /// **`D8j` — the verified composed-discharge relation for this function.**
    ///
    /// ⛔⛔ **Deliberately NOT `continuation_emissions`.** That map's gate
    /// requires the recorded instruction to decode to `identity.target()` — the
    /// specialization the causal call names. A lawful composed instruction
    /// targets the **raw worker**, so putting a composed record there would
    /// either fail a gate it was never about or force that gate to be loosened
    /// for every direct emission. Two relations, two contracts.
    ///
    /// ⛔ Populated only by `verify_recorded_composed_discharges`, only from
    /// [`Self::pending_composed_discharges`], and only after all five
    /// verifications pass. `D8k` owns whatever global closure reads it.
    composed_discharges: BTreeMap<ContinuationCallIdentity, cranelift_codegen::ir::Inst>,
    declaration_calls: BTreeMap<StaticOriginId, units::DeclaredUnitCall>,
    /// The current function's closed trap-exit authority. Absence is an error
    /// state, never an implicit Root.
    trap_exit: Option<TrapExitAuthority>,
    /// Source occurrences reached only through the current unit's result
    /// position. Process-exit constructors are normalized only at these
    /// occurrences, never merely because an exit-shaped value appears nested.
    terminal_result_origins: BTreeSet<StaticOriginId>,
    /// Join-plan entries consumed while defining this function.  Each
    /// `FunctionLocalRefs` is freshly declared for one generated function, so
    /// this set cannot alias consumption across unit bodies.
    consumed_join_origins: BTreeSet<StaticOriginId>,
    /// Planned joins under source branches proven statically unselected while
    /// defining this function.
    ///
    /// Keeping these separate from consumed joins makes both mismatches loud:
    /// entering a dead join rejects, while failing to disposition one leaves
    /// the generated-function closure check red.
    dispositioned_join_origins: BTreeSet<StaticOriginId>,
    /// Whether the owner-bound reachable/dead partition has been closed.
    ///
    /// The boolean is separate from either partition because a generated
    /// function may lawfully own no reachable joins.
    join_disposition_finalized: bool,
    /// The final semantically reachable half of the owner-bound join
    /// population. This is derived only after every reached-case union closes;
    /// it is not inferred from structural token consumption.
    final_reachable_join_origins: BTreeSet<StaticOriginId>,
    /// Actual CLIF merge blocks materialized for each planned source join.
    ///
    /// Token consumption can precede semantic selection without producing a
    /// merge block. Keeping the CFG population separate is what lets closure
    /// distinguish metadata materialization from a live SSA join.
    materialized_join_blocks: BTreeMap<StaticOriginId, BTreeSet<Block>>,
    /// Case indices actually reached while emitting each statically selected
    /// source `Match`.
    ///
    /// Selection may revisit one source occurrence through a recursive
    /// producer and reach a different case later. We therefore close the dead
    /// population only after emission, from the union of reached indices,
    /// rather than treating the first observed constructor as globally final.
    emission_reachable_match_cases: BTreeMap<StaticOriginId, BTreeSet<usize>>,
}

impl FunctionLocalRefs {
    fn bind_unit_trap_frame(
        &mut self,
        slots: cranelift_codegen::ir::Value,
        trap_offset: i32,
    ) -> Result<(), CraneliftBackendError> {
        if self.trap_exit.is_some() {
            return Err(backend_module(
                "unit trap frame was bound to a function without unit authority".to_string(),
            ));
        }
        self.trap_exit = Some(TrapExitAuthority::UnitFrame { slots, trap_offset });
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyEmissionAuthority {
    RecursiveDescent,
    FunctionizedUnits,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HostContextPropagationMutation {
    Exact,
    ServicesPointer,
    NativeIntArena,
    BoundaryArena,
    Null,
    LaunchIngress,
}

#[cfg(test)]
thread_local! {
    static HOST_CONTEXT_PROPAGATION_MUTATION:
        std::cell::Cell<HostContextPropagationMutation> =
        const { std::cell::Cell::new(HostContextPropagationMutation::Exact) };
}

#[cfg(test)]
thread_local! {
    static D8_CARRIED_JOIN_UNCHANGED: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    static D8_SPECIALIZED_JOIN_PRODUCTIONS: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    static D8_JOIN_MERGES_CREATED: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
    static D8_JOIN_CONSUMPTION_MUTATION: std::cell::Cell<JoinConsumptionMutation> =
        const { std::cell::Cell::new(JoinConsumptionMutation::Exact) };
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JoinConsumptionMutation {
    Exact,
    SkipFirst,
    DuplicateFirst,
    IncludeStaticallyUnselected,
    OmitFirstStaticallyUnselectedMatchCase,
    OmitSourceMachineComputationalMatchSelection,
    MaterializeFirstUnselectedMatchJoin,
    AttachEntryToFirstMaterializedDead,
    DispositionDynamicHostResultMerge,
}

#[cfg(test)]
fn reset_d8_join_conversion_counts() {
    D8_CARRIED_JOIN_UNCHANGED.with(|count| count.set(0));
    D8_SPECIALIZED_JOIN_PRODUCTIONS.with(|count| count.set(0));
    D8_JOIN_MERGES_CREATED.with(|count| count.set(0));
}

#[cfg(test)]
fn d8_join_conversion_counts() -> (usize, usize) {
    (
        D8_CARRIED_JOIN_UNCHANGED.with(std::cell::Cell::get),
        D8_SPECIALIZED_JOIN_PRODUCTIONS.with(std::cell::Cell::get),
    )
}

#[cfg(test)]
fn d8_join_merge_count() -> usize {
    D8_JOIN_MERGES_CREATED.with(std::cell::Cell::get)
}

#[cfg(test)]
fn set_d8_join_consumption_mutation(mutation: JoinConsumptionMutation) {
    D8_JOIN_CONSUMPTION_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
fn set_host_context_propagation_mutation(mutation: HostContextPropagationMutation) {
    HOST_CONTEXT_PROPAGATION_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessSlotMutation {
    Exact,
    DeleteProcessInput,
    DeleteCapability,
    AttemptFixedContextOffsets,
    ReintroduceLaunchIngress,
}

#[cfg(test)]
thread_local! {
    static PROCESS_SLOT_MUTATION: std::cell::Cell<ProcessSlotMutation> =
        const { std::cell::Cell::new(ProcessSlotMutation::Exact) };
}

#[cfg(test)]
fn set_process_slot_mutation(mutation: ProcessSlotMutation) {
    PROCESS_SLOT_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrapFrameBindingMutation {
    Exact,
    DeleteUnitLane,
    MisclassifyUnitAsRoot,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrapIdentityMutation {
    Exact,
    Zero,
    Substitute,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrapCallerProtocolMutation {
    Exact,
    LeaveStaleTrap,
    ReadResultBeforeTrap,
}

/// **`RT-CONTSPEC-ACTIVATE` `D4` — the three executable controls for the
/// continuation emission seam.**
///
/// All `#[cfg(test)]`; production compiles as if they did not exist. Each sits
/// on the exact production branch it perturbs, so its red reproduces from the
/// committed tree by flipping the switch.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContinuationEmissionMutation {
    Exact,
    /// `D4` emission seam: substitute **only the function-local emitted
    /// `FuncRef`** with another callable already declared in this same
    /// function, retaining the continuation's header, slots, offsets, inputs,
    /// identity, and owner.
    ///
    /// ⭐ **This replaces the same-shaped two-target redirect, which was
    /// unreachable here and was therefore never a control at all.** That one
    /// searched `continuation_calls` for a *distinct same-shaped* entry; this
    /// generated function holds exactly one entry, so it refused with "found no
    /// distinct same-shaped call target" **before reaching the call seam**. A
    /// pre-call refusal proves a missing fixture precondition, not emitted-
    /// target equality. Architect `evt_6bf2mmehjzy3k`.
    ///
    /// ⛔ Substituting the ref and nothing else is what isolates the `ACTIVATE`
    /// property: the call is still emitted, on the same ABI, with the same
    /// inputs -- only the callee identity moves, so the finished-CLIF oracle
    /// must reject for exactly one reason.
    ///
    /// ⛔ No fall back to exact. If no other callable is declared in this
    /// function the control fails loudly, because a control that silently
    /// becomes the identity is vacuous.
    ///
    /// The **two-target same-shaped** redirect and its observable behavioural
    /// consequence live wholly in `RT-CONTSPEC-WITNESS` `D7`/`AC-9`, whose
    /// integrated fixture must supply two distinct same-shaped targets in one
    /// lawful callable population.
    SubstituteEmittedFuncRef,
    /// `4b` closure seam: emit the direct call but do not record it against its
    /// causal token, so the finished-CLIF sweep must notice an emission the
    /// records do not account for.
    SuppressEmissionRecord,
    /// `4b` closeout seam: verify each function's emissions but never
    /// accumulate them, so the whole-pass set equality must notice the missing
    /// population.
    SuppressEmissionAccumulation,
    /// `D3` affine seam: claim the same causal token twice.
    ClaimTokenTwice,
    /// `D3` owner seam: claim under a producer owner that does not own the
    /// token.
    ClaimUnderWrongOwner,
}

#[cfg(test)]
fn set_continuation_emission_mutation(mutation: ContinuationEmissionMutation) {
    CONTINUATION_EMISSION_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
thread_local! {
    static CONTINUATION_EMISSION_MUTATION: std::cell::Cell<ContinuationEmissionMutation> =
        const { std::cell::Cell::new(ContinuationEmissionMutation::Exact) };
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D3b` — the CONSUMER mutations.**
///
/// ⭐ `D4a`'s mutations proved the *instrument*: that the nearest-alias slot and
/// the locator slot hold different operands. These prove the *consumer*: that
/// the production emission seam refuses when it reads the wrong one. The
/// Architect's gate `evt_65xkzqppdqdaj` requires both, and passing the first
/// does not discharge the second.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D3bConsumerMutation {
    Exact,
    /// Consume the locator's scope-relative introduction index instead of the
    /// projection's nearest-alias index — the exact defect `D2b` reopened `D2` for,
    /// now expressed at the consumption boundary.
    ConsumeLocatorIndex,
    /// Move the resolved slot by one. ⛔ Distinct from the above rather than
    /// redundant with it: on a fixture whose locator and nearest-alias indices are
    /// adjacent the two coincide, but this one also perturbs an emission with a
    /// single producer-local input, where no collision exists to catch it.
    ShiftProducerLocalSlot,
}

/// **`D4b`** — how the generated-frame consumer's identity revalidation is
/// perturbed on the BEHAVIOURAL path. Test-only.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D4bFrameMutation {
    Exact,
    /// Present a claimed `ContinuationContextId` that is not the one this
    /// claim's own `(specialization, worker body)` key resolves to.
    WrongClaimedContext,
}

#[cfg(test)]
thread_local! {
    static D4B_FRAME_MUTATION: std::cell::Cell<D4bFrameMutation> =
        const { std::cell::Cell::new(D4bFrameMutation::Exact) };
    /// How many times the GENERATED-frame arm of `verify_entry_frame` was
    /// actually taken. ⛔ The non-vacuity counter: a behavioural control that
    /// only asserts a successful compile cannot tell "the generated route ran"
    /// from "the fixture never took it".
    static D4B_GENERATED_FRAME_CONSUMPTIONS: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d4b_frame_mutation(mutation: D4bFrameMutation) {
    D4B_FRAME_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4b_frame_mutation() -> D4bFrameMutation {
    D4B_FRAME_MUTATION.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d4b_generated_frame_consumption() {
    D4B_GENERATED_FRAME_CONSUMPTIONS.with(|cell| cell.set(cell.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4b_generated_frame_consumptions() -> usize {
    D4B_GENERATED_FRAME_CONSUMPTIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d4b_generated_frame_consumptions() {
    D4B_GENERATED_FRAME_CONSUMPTIONS.with(|cell| cell.set(0));
}

#[cfg(test)]
thread_local! {
    static D3B_CONSUMER_MUTATION: std::cell::Cell<D3bConsumerMutation> =
        const { std::cell::Cell::new(D3bConsumerMutation::Exact) };
    static D3B_CONSUMER_APPLIED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d3b_consumer_mutation(mutation: D3bConsumerMutation) {
    D3B_CONSUMER_MUTATION.with(|cell| cell.set(mutation));
    D3B_CONSUMER_APPLIED.with(|cell| cell.set(0));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3b_consumer_mutation() -> D3bConsumerMutation {
    D3B_CONSUMER_MUTATION.with(std::cell::Cell::get)
}

/// ⛔ A mutation that never fired is not a control. This counter is what lets a
/// row assert the perturbation actually reached the seam, rather than passing
/// because nothing happened.
#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d3b_consumer_application() {
    D3B_CONSUMER_APPLIED.with(|cell| cell.set(cell.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3b_consumer_applications() -> usize {
    D3B_CONSUMER_APPLIED.with(std::cell::Cell::get)
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D4a` — the lowering-side operand observatory.**
///
/// ⭐ **This is an INSTRUMENT, not a mechanism.** It is `#[cfg(test)]` in every
/// part; production compiles as if it did not exist, and nothing here is
/// consulted by any lowering decision. Its whole job is to answer one question
/// the planner cannot be asked without becoming its own oracle: *which actual
/// operand does the emitting context's environment hold at the nearest-alias
/// index, and is it the operand lowering built for that exact binding?*
///
/// ⛔ **The identity it reports is the Cranelift SSA `Value`, deliberately.**
/// A carrier, a phase, a length or a planner coordinate would all agree between
/// a correct index and a wrong one on a same-shaped decoy — which is exactly the
/// population this checkpoint's fixture supplies. The SSA word is the one thing
/// that cannot agree, so it is the only honest discriminator here.
///
/// ⛔ **No planner re-walk, no index arithmetic, no fixture-authored expected
/// index.** The creation half is keyed by the LOWERING's own occurrence id at
/// the seat where it constructs the binder; the seam half is keyed by index.
/// The two are joined by `binding_origin`, and a wrong index breaks the join.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D4aSlotSelection {
    /// Production's own answer: the projection's `nearest_alias_index`.
    Exact,
    /// `D4a` mutation 1 — consume the locator's scope-relative introduction
    /// index instead. This is the defect `D2b` reopened `D2` for, expressed at
    /// the lowering slot.
    UseLocatorIndex,
    /// `D4a` mutation 2 — read both slots, then exchange them. Distinct from
    /// mutation 1: it perturbs the *pairing* while both indices stay lawful, so
    /// it survives any repair that merely bounds-checks the index.
    SwapSlots,
}

#[cfg(test)]
thread_local! {
    /// ⛔ The observatory is **disarmed by default**, so every other test in
    /// this binary pays nothing and records nothing. Only `D4a`'s own control
    /// arms it, and it disarms again when that control takes its readings.
    static D4A_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static D4A_SLOT_SELECTION: std::cell::Cell<D4aSlotSelection> =
        const { std::cell::Cell::new(D4aSlotSelection::Exact) };
    static D4A_SEAM: std::cell::RefCell<Vec<D4aSeamObservation>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static D4A_CREATED: std::cell::RefCell<Vec<(StaticOriginId, String)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// One reaching producer-local continuation input, as lowering actually sees it.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(in crate::cranelift_backend) struct D4aSeamObservation {
    /// The binding this input names — the join key into the creation record.
    pub(in crate::cranelift_backend) binding_origin: StaticOriginId,
    /// The projection's nearest-alias index into the emitting environment.
    pub(in crate::cranelift_backend) nearest_alias_index: u32,
    /// The locator's scope-relative introduction index.
    pub(in crate::cranelift_backend) locator_index: u32,
    /// The actual operand at the selected nearest-alias slot.
    pub(in crate::cranelift_backend) nearest_alias_operand: String,
    /// The actual operand at the locator slot — the decoy.
    pub(in crate::cranelift_backend) locator_operand: String,
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_set_slot_selection(selection: D4aSlotSelection) {
    D4A_SLOT_SELECTION.with(|cell| cell.set(selection));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_set_armed(armed: bool) {
    D4A_ARMED.with(|cell| cell.set(armed));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_armed() -> bool {
    D4A_ARMED.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_slot_selection() -> D4aSlotSelection {
    D4A_SLOT_SELECTION.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_record_seam(observation: D4aSeamObservation) {
    if !d4a_armed() {
        return;
    }
    D4A_SEAM.with(|cell| cell.borrow_mut().push(observation));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_take_seam() -> Vec<D4aSeamObservation> {
    D4A_SEAM.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// The **binder-creation seat**: lowering has just built the operand for the
/// value bound at `origin`. ⭐ Recorded here and nowhere else, because this is
/// the only point at which the operand and the occurrence that creates it are
/// both in hand without consulting an environment index.
#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_record_created(
    origin: StaticOriginId,
    operand: String,
) {
    if !d4a_armed() {
        return;
    }
    D4A_CREATED.with(|cell| cell.borrow_mut().push((origin, operand)));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_take_created() -> Vec<(StaticOriginId, String)> {
    D4A_CREATED.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// A lowering-side description of one environment slot, carrying the Cranelift
/// SSA `Value` wherever the operand has one.
///
/// ⛔ Deliberately **not** a `Debug` impl on the operand types. `Lowered` has no
/// `Debug` on purpose (`RT-FNSPLIT-C1`), and adding one would be a second way to
/// read a compile-time template out of an operand. This is a test-only
/// projection to a string, and it reaches only the fields it names.
#[cfg(test)]
pub(in crate::cranelift_backend) fn d4a_describe_binding(
    binding: Option<&LoweringEnvironmentBinding>,
) -> String {
    match binding {
        None => "none".to_string(),
        Some(LoweringEnvironmentBinding::StaticWorker(..)) => "worker".to_string(),
        Some(LoweringEnvironmentBinding::Value(LoweringOperand::Carried(word))) => {
            format!("carried({:?})", word.word)
        }
        Some(LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(lowered))) => {
            match lowered {
                Lowered::Int { value, .. }
                | Lowered::Bool { value, .. }
                | Lowered::ProcessExitStatus { value }
                | Lowered::CapabilityToken { value }
                | Lowered::ResourceToken { value }
                | Lowered::BorrowedNativeValue { pointer: value } => {
                    format!("specialized-scalar({value:?})")
                }
                Lowered::Constructor { constructor, .. } => {
                    format!("specialized-ctor({constructor})")
                }
                Lowered::Record { .. } => "specialized-record".to_string(),
                Lowered::Closure { .. } => "specialized-closure".to_string(),
                Lowered::Trap(..) => "specialized-trap".to_string(),
                Lowered::HostResult {
                    success,
                    ok_constructor,
                    err_constructor,
                    ..
                } => format!("specialized-hostresult({success:?},{ok_constructor},{err_constructor})"),
                Lowered::ResponseBytes { pointer, len } => {
                    format!("specialized-responsebytes({pointer:?},{len:?})")
                }
                Lowered::Bytes(bytes) => format!("specialized-bytes({bytes:?})"),
                Lowered::String(text) => format!("specialized-string({text})"),
                _ => "specialized-other".to_string(),
            }
        }
    }
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D3c` — the entry-ABI immediate-availability
/// observatory.**
///
/// ⭐ **An INSTRUMENT, not a mechanism**, on the same terms as `D4a` above:
/// `#[cfg(test)]` throughout, disarmed by default, consulted by no lowering
/// decision. ⛔ **`D3c` authorizes no production edit**, and nothing here is one.
///
/// **The question.** `exact_continuation_projection`'s `RootIsImmediate` arm
/// copies an `EntryAbi` root's `source_abi_position` straight into
/// `immediate_slot`, and the emission seam then reads `producer_env` at that
/// slot. `D4a` established that `producer_env` at a predeclared seat is the
/// **current lexical environment, with intervening binders prepended** — not the
/// entry ABI operand run. So the copy is only sound at zero binder depth, and
/// every population before `D4a` was at zero binder depth.
///
/// **The oracle, and why it is independent.** Production already records the
/// entry ABI operands, in ABI-position order, at unit entry: `D5a` built
/// `defining_abi_operands` from the same single slot walk that seeds the entry
/// environment, so "index `i` is ABI position `i`" holds there by construction.
/// That record is keyed by ABI position and never by an environment index, so
/// comparing it against `producer_env[source_abi_position]` is a comparison of
/// two independently-derived answers to "which value is this", not a walk
/// checked against itself.
///
/// ⛔ **The identity reported is the Cranelift SSA `Value`**, for `D4a`'s reason:
/// the carrier, the phase and the lowering shape all agree between the entry
/// parameter and the local binding that displaces it, so only the SSA word can
/// discriminate.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D3cPositionSelection {
    /// Read the emission environment where the entry value **actually is**,
    /// located by operand identity against the entry oracle.
    MeasuredImmediate,
    /// `D3c` mutation — read at `source_abi_position` instead, which is exactly
    /// what production does today. ⭐ This is the Architect's condition 4: the
    /// substitution must flip the observed operand, and flip it because the two
    /// positions hold **different values**, not because one is out of bounds or
    /// of a different shape.
    SourceAbiPosition,
}

#[cfg(test)]
thread_local! {
    /// ⛔ Disarmed by default. Only `D3c`'s own control arms it.
    static D3C_ARMED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static D3C_POSITION_SELECTION: std::cell::Cell<D3cPositionSelection> =
        const { std::cell::Cell::new(D3cPositionSelection::MeasuredImmediate) };
    static D3C_SEAT: std::cell::RefCell<Vec<D3cSeatObservation>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// One entry-ABI continuation input at one real predeclared emission seat.
///
/// ⭐ The row is deliberately **raw**: it carries the whole emission environment
/// and the entry operand, and does no locating of its own beyond the selection
/// the mutation drives. The control re-derives the measured position from these
/// fields, so the instrument cannot quietly become the oracle.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(in crate::cranelift_backend) struct D3cSeatObservation {
    /// How many inputs of this seat's required vector carry an `EntryAbi` root.
    pub(in crate::cranelift_backend) entry_abi_inputs: usize,
    /// How many carry a `ProducerLocal` root — the Architect's condition 2 needs
    /// at least one of each **in the same vector**.
    pub(in crate::cranelift_backend) producer_local_inputs: usize,
    /// This input's root ABI position, which the projection also used as its
    /// `immediate_slot`.
    pub(in crate::cranelift_backend) source_abi_position: u32,
    /// Production's own entry-walk record at that ABI position — the oracle.
    pub(in crate::cranelift_backend) entry_operand: String,
    /// Length of the entry ABI operand run.
    pub(in crate::cranelift_backend) abi_operands: usize,
    /// The whole emission-seat environment, in order.
    pub(in crate::cranelift_backend) emission_environment: Vec<String>,
    /// The position this instrument read, under the active selection.
    pub(in crate::cranelift_backend) observed_position: Option<u32>,
    /// The operand found there.
    pub(in crate::cranelift_backend) observed_operand: String,
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3c_set_armed(armed: bool) {
    D3C_ARMED.with(|cell| cell.set(armed));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3c_armed() -> bool {
    D3C_ARMED.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3c_set_position_selection(selection: D3cPositionSelection) {
    D3C_POSITION_SELECTION.with(|cell| cell.set(selection));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3c_position_selection() -> D3cPositionSelection {
    D3C_POSITION_SELECTION.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3c_record_seat(observation: D3cSeatObservation) {
    if !d3c_armed() {
        return;
    }
    D3C_SEAT.with(|cell| cell.borrow_mut().push(observation));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d3c_take_seat() -> Vec<D3cSeatObservation> {
    D3C_SEAT.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// **`AC-5` -- the two executable mutation controls for the static-worker
/// substrate.**
///
/// These perturb **production resolution**, at the two seams the substrate
/// actually depends on, and they are committed rather than applied by hand so
/// the proof does not evaporate when a reviewer closes their terminal. Both
/// arms are `#[cfg(test)]`; production compiles as if they did not exist.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticWorkerMutation {
    Exact,
    /// `D2` seam: restore the pre-node carried-capture narrowing in
    /// `lower_binder`, so a carried capture again goes to the specialized-only
    /// fold instead of installing a `StaticWorker`.
    RestoreCarriedCaptureNarrowing,
    /// `D4` transport seam: hand the consumer a **distinct** already-resolved
    /// target from this function's own `worker_calls`, selected by `AC-6`'s
    /// same-shape definition -- same declared arity, same capture count -- and
    /// by nothing else.
    ///
    /// `AC-5` clause (c): this switch runs on the **two-same-shape-worker
    /// program and only there**. The ordinary single-worker witness has no
    /// distinct same-shape candidate at all, which is a property of that
    /// fixture's population; it discharges the carried-capture arm and nothing
    /// more. Widening the predicate back toward mere difference to make it
    /// find something there is clause (b)'s defect reintroduced as a repair.
    ///
    /// The binding and its construction are untouched -- this is a redirect
    /// performed *after* the worker was constructed, which is the only place
    /// it proves anything.
    RedirectResolvedWorkerTarget,
}

#[cfg(test)]
fn set_static_worker_mutation(mutation: StaticWorkerMutation) {
    STATIC_WORKER_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
thread_local! {
    static STATIC_WORKER_MUTATION: std::cell::Cell<StaticWorkerMutation> =
        const { std::cell::Cell::new(StaticWorkerMutation::Exact) };
    static TRAP_FRAME_BINDING_MUTATION: std::cell::Cell<TrapFrameBindingMutation> =
        const { std::cell::Cell::new(TrapFrameBindingMutation::Exact) };
    static TRAP_IDENTITY_MUTATION: std::cell::Cell<TrapIdentityMutation> =
        const { std::cell::Cell::new(TrapIdentityMutation::Exact) };
    static TRAP_CALLER_PROTOCOL_MUTATION: std::cell::Cell<TrapCallerProtocolMutation> =
        const { std::cell::Cell::new(TrapCallerProtocolMutation::Exact) };
    /// **`RT-DECL-CLOSURE-PORT` `D5` — every declaration-owned unit call this
    /// thread actually emitted**, as `(reference occurrence, target origin,
    /// emitted callee)`.
    ///
    /// ⛔ Appended at the emission site from the emitted `Inst` itself. Its
    /// point is to be an authority *independent of* `declaration_calls`, so a
    /// control can compare the planner-resolved target against what was really
    /// called. ⚠ It accumulates across a thread, so read it through
    /// [`d5_emitted_declaration_calls`] after
    /// [`reset_d5_emitted_declaration_calls`] — a bare read attributes an
    /// earlier compile's calls to the current one.
    /// **`RT-DECL-CLOSURE-PORT` `D5`** — the causal controls on the checked-call
    /// closeout. Each defeats exactly one of the three things the closeout
    /// claims: that every lawful emission is recorded, that no template records
    /// twice, and that the recorded callee is the one actually emitted.
    static D5_CLOSEOUT_MUTATION: std::cell::Cell<D5CloseoutMutation> =
        const { std::cell::Cell::new(D5CloseoutMutation::Exact) };
    static D5_EMITTED_DECLARATION_CALLS: std::cell::RefCell<
        Vec<(StaticOriginId, StaticOriginId, cranelift_codegen::ir::FuncRef)>,
    > = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D5CloseoutMutation {
    Exact,
    /// Emit the lawful call, then suppress its ledger entry.
    SuppressLedgerEntry,
    /// Record the entry twice under one template.
    DuplicateLedgerEntry,
    /// Record an entry under a template the plan never issued.
    ExtraLedgerEntry,
    /// Record a callee that is not the one the instruction actually calls.
    SubstituteEmittedCallee,
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn with_d5_closeout_mutation<T>(
    mutation: D5CloseoutMutation,
    body: impl FnOnce() -> T,
) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            D5_CLOSEOUT_MUTATION.with(|cell| cell.set(D5CloseoutMutation::Exact));
        }
    }
    D5_CLOSEOUT_MUTATION.with(|cell| cell.set(mutation));
    let _restore = Restore;
    body()
}

/// **`RT-DECL-CLOSURE-PORT` `D5a` — the outcome-complete localization trace.**
///
/// ⛔⛔ **Outcome-complete on purpose.** The previous localization instrumented
/// only `claim_and_call_continuation`'s two EARLY RETURNS and read zero hits as
/// "the helper is never entered". That inference is wrong: a *successful*
/// claim-and-call reaches neither early return, so zero is equally consistent
/// with the claim succeeding. A negative check passes for any reason, and that
/// one had no positive control. ⇒ Every path through the instrumented sites
/// records a terminal outcome here, so silence in the trace means the site was
/// not reached and nothing else.
#[cfg(test)]
thread_local! {
    static D5A_TRACE: std::cell::RefCell<Vec<String>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d5a_trace(entry: String) {
    D5A_TRACE.with(|trace| trace.borrow_mut().push(entry));
}

/// **`RT-DECL-CLOSURE-PORT` `D5a` — the checked-IH marker's ORDERED event log.**
///
/// ⭐ Two facts have to be separable and one of them is an **ordering**: the
/// marker is consumed *at the call edge, before emission*. A pair of counters
/// cannot say that -- both are `1` whether the consumption happened before or
/// after the call was written. One ordered log can, so this is one log.
///
/// ⚠ Accumulates across a thread; read it through [`d5a_marker_events`] after
/// [`reset_d5a_marker_events`], or an earlier compile's events are attributed
/// to this one.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D5aMarkerEvent {
    /// A checked-IH marker was consumed at the exact static-worker call edge,
    /// carrying every identity that had to agree for it to be consumed.
    Consumed {
        call_template_id: u64,
        slot_template_id: u64,
        binder_index: u64,
        arity: u64,
    },
    /// A static worker call instruction was actually written.
    ///
    /// `raw_operands` is the run the raw worker's own contract accounts for —
    /// source arguments plus stored captures — and `supplied_operands` is what
    /// the call actually carried. ⭐ Recording both separately is what makes the
    /// generated-context capture suffix a **measurable prefix relation** rather
    /// than a claim: a retargeted call is the raw run plus the enclosing
    /// frame's continuation inputs, and an unretargeted one is the raw run
    /// exactly. One total would conflate "no suffix" with "a suffix of zero".
    WorkerCallEmitted {
        body_origin: StaticOriginId,
        raw_operands: usize,
        supplied_operands: usize,
        /// **`RT-CONTSRC-PRODUCER-LOCAL` `D6b`** — the route this call was
        /// emitted on.
        ///
        /// ⭐ Recorded beside the operand counts because together they are the
        /// **call semantics**, and `D6b`'s claim is about the pair. `D6a` binds
        /// two workers over one `body_origin`, so the three pre-`D6b` fields
        /// cannot say which binding an event belongs to — two events for one
        /// body are indistinguishable without this. With it, "the raw route
        /// appends nothing and the context route appends the suffix" is one
        /// readable relation per event rather than an inference across events.
        route: StaticWorkerCallRoute,
    },
}

#[cfg(test)]
thread_local! {
    static D5A_MARKER_EVENTS: std::cell::RefCell<Vec<D5aMarkerEvent>> = const {
        std::cell::RefCell::new(Vec::new())
    };
    static D5A_MARKER_MUTATION: std::cell::Cell<D5aMarkerMutation> =
        const { std::cell::Cell::new(D5aMarkerMutation::Exact) };
}

/// **`RT-DECL-CLOSURE-PORT` `D5a`** — the two causal mutations on the checked-IH
/// marker seam. Each defeats exactly one claim the seam makes, and each is a
/// perturbation the plan alone cannot express.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D5aMarkerMutation {
    Exact,
    /// Skip the consumption without an error, leaving the call itself lawful
    /// and emitted. ⛔ The only way to ask whether the positive route *depends*
    /// on the consumption — every plan-level perturbation refuses at the
    /// consumer instead of reaching closeout.
    SuppressConsumption,
    /// Skip the ENTRY arity check only.
    ///
    /// ⚠ Without this, the consumer's own arity guard is **unreachable on this
    /// witness**: the marker wraps the very call that reaches the consumer, so
    /// entry and the consumer read the same two numbers and entry always
    /// refuses first. The guard is still ruled and still load-bearing where a
    /// marker's wrapped call is not the one that reaches a static worker — this
    /// switch is what lets a control red it rather than leave it asserted.
    RelaxEntryArity,
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn with_d5a_marker_mutation<T>(
    mutation: D5aMarkerMutation,
    body: impl FnOnce() -> T,
) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            D5A_MARKER_MUTATION.with(|cell| cell.set(D5aMarkerMutation::Exact));
        }
    }
    D5A_MARKER_MUTATION.with(|cell| cell.set(mutation));
    let _restore = Restore;
    body()
}

/// **`RT-DECL-CLOSURE-PORT` `D5a` checkpoint 4 step 3 — the REACHING mutations
/// for the ruled discriminators.**
///
/// ⭐⭐ Every variant here exists because the route is now **positive**. Before
/// checkpoint 4 the only fixture that reaches these seats refused further along,
/// so a control written then would have compared a red against a red and passed
/// for the wrong reason — which is why the frame forbids red-versus-red
/// evidence and why the detached seat carried an explicit *"UNEXERCISED
/// GUARDS — do not read these as tested"* block. Each variant below moves the
/// green compile to one named refusal.
///
/// ⛔ Each perturbs **what a seat is handed**, never the guard that inspects it.
/// A mutation of the guard would ask whether the guard agrees with itself.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D5aRouteMutation {
    Exact,
    // ── the detached-result seat's five formerly unexercised guards ─────────
    /// Present the one residual causal edge twice, so the seat sees a
    /// multi-member projection onto one unit result.
    DuplicateResidualEdge,
    /// Hand the seat a carried word instead of the specialized constructor the
    /// planned producer edge requires.
    CarryNonConstructorResult,
    /// Strip the lowered constructor's synthesized identity, so the result is
    /// no longer the planner's own constructor for that edge's construct
    /// origin.
    StripLoweredConstructorIdentity,
    /// Move the ruled recursive position past the planned constructor's field
    /// run.
    PerturbRecursivePosition,
    /// Perturb the continuation's declared ordinary-parameter count, so the
    /// field run and the declared run no longer differ by exactly the one
    /// omitted recursive field.
    PerturbOrdinaryParameterCount,
    // ── the generated-context binding (checkpoint 4 step 1) ────────────────
    /// Withhold the retarget's context lookup, leaving the specialization to
    /// call the raw worker unit that checkpoint 4 step 2 removed from the
    /// executable population.
    SuppressContextBinding,
    /// Bind a context the *enclosing specialization does not own* — the
    /// transplant the ruling's key exists to prevent.
    TransplantContextBinding,
    /// Make the identity-and-body key resolve TWICE, so the lookup sees two
    /// contexts claiming one specialization and worker body.
    ///
    /// ⚠ Applied inside the lookup because the planner derives its context
    /// population deterministically and interns on that same key: a duplicate
    /// is unreachable through any plan the planner will build, so this
    /// perturbs the guard's INPUT — the population it walks — which is the
    /// only way to ask the question at all.
    DuplicateContextBinding,
    // ── the capture projection: root provenance versus immediate slot ──────
    /// Index the emitting environment with the ROOT ABI position instead of the
    /// immediate slot. ⭐ On a specialization-owned edge those are different
    /// environments, which is the whole reason the two coordinates are kept
    /// apart.
    ///
    /// ⚠ Read the row that uses this before trusting it: on the witness both
    /// coordinates are IN RANGE, so the swap binds different operands without
    /// any refusal. That is a measured limit of the evidence, not a defence.
    ReadRootPositionAsImmediateSlot,
    /// Push the immediate slot one past the emitting environment, so the bounds
    /// guard that makes the planner's resolution *answerable here* has to fire.
    PerturbImmediateSlotOutOfRange,
    /// Break the predeclared emitter's consistency law: for an owner that IS
    /// its inputs' root provenance, move the immediate slot off the root ABI
    /// position it must equal.
    PerturbPredeclaredImmediateSlot,
    /// `RT-CONTSRC-PRODUCER-LOCAL` `D1` — present a **producer-local**
    /// coordinate at both emission seams.
    ///
    /// ⚠ Perturbs the seam's INPUT, deliberately: `D1` represents the
    /// producer-local domain but nothing constructs one yet, so no plan the
    /// planner will build reaches these arms. Without this the two refusals are
    /// unmeasured code, which is indistinguishable from absent code.
    PresentProducerLocalCoordinate,
    // ── the generated-context capture suffix ──────────────────────────────
    // ── the carried-invocation binding's retained source coordinates ───────
    /// Perturb the recursive position the carried invocation presents, so the
    /// lookup is asked for a coordinate the planner never issued.
    PerturbCarriedInvocationCoordinates,
    // ── the raw worker's unchanged descriptor authority ────────────────────
    /// Drop the superseded bodies from the raw template population, keeping
    /// only the executable ones — the "template-only means deleted" reading
    /// checkpoint 1 exists to refuse.
    DropSupersededWorkerTemplates,
    // ── the one cross-pass causal ledger's lifetime ────────────────────────
    /// Close the ledger after the FIRST definition pass, before any generated
    /// `Function` exists. ⭐ This is the checkpoint-2 defect reproduced
    /// deliberately: the equality is unchanged and only the window it is taken
    /// over moves, which is what makes the refusal attributable to the
    /// lifetime.
    CloseLedgerAfterTheFirstPass,
}

#[cfg(test)]
thread_local! {
    static D5A_ROUTE_MUTATION: std::cell::Cell<D5aRouteMutation> =
        const { std::cell::Cell::new(D5aRouteMutation::Exact) };
    /// How many times the live mutation actually **fired**.
    ///
    /// ⛔ A mutation whose precondition never holds on this fixture is
    /// indistinguishable from an inert one: the compile stays green and the
    /// control reads as "the mechanism defended itself". Every seat that can
    /// decline to apply its mutation bumps this, so a control can require that
    /// its perturbation reached the code at all.
    static D5A_ROUTE_APPLICATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d5a_route_mutation() -> D5aRouteMutation {
    D5A_ROUTE_MUTATION.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d5a_route_application() {
    D5A_ROUTE_APPLICATIONS.with(|cell| cell.set(cell.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d5a_route_applications() -> usize {
    D5A_ROUTE_APPLICATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn with_d5a_route_mutation<T>(
    mutation: D5aRouteMutation,
    body: impl FnOnce() -> T,
) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            D5A_ROUTE_MUTATION.with(|cell| cell.set(D5aRouteMutation::Exact));
        }
    }
    D5A_ROUTE_MUTATION.with(|cell| cell.set(mutation));
    D5A_ROUTE_APPLICATIONS.with(|cell| cell.set(0));
    let _restore = Restore;
    body()
}

/// **`RT-DECL-CLOSURE-PORT` `D6a` upstream — what the routed answer did.**
///
/// ⭐ The route is a **predecessor-edge fact**, so every control over it has to
/// name an *edge*: which producer raised it, and which consumer received it.
/// A control that could only see the consumer's final value would be unable to
/// distinguish the two producers, and the whole point of this checkpoint is
/// that they are distinct authorities.
///
/// ⛔ These carry the exact planner identities they observed, never counts —
/// *"planned, claimed and emitted call identity agree on the exact identity,
/// not merely on counts"* is one of the discriminators this checkpoint owes.
#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::cranelift_backend) enum D6aRouteEvent {
    /// **PRODUCER 2.** The result of an exactly claimed and emitted
    /// continuation-specialization call was raised to
    /// `CheckedSelectedRecursor`. Recorded only after the owner/affine claim
    /// succeeded and the emitted callee was checked against
    /// `identity.target()`, so `target` is the authority's own identity and
    /// not a guess about it.
    CallResultRaised {
        target: ContinuationSpecializationId,
    },
    /// **PRODUCER 1.** An exact recursor layer supplied a route.
    ///
    /// ⚠ Both answers are recorded, and `selects_occurrence` is recorded beside
    /// the answer, so a reader can see the decision rather than infer it. ⛔ On
    /// the governed witness this producer only ever fires **positively**
    /// (`checked_frame_id: Some(7)`, `selects_occurrence: true`), so the
    /// negative arm is not exercised here — the mutation
    /// [`D6aRouteMutation::DropRecursorLayerRoute`] is what supplies a
    /// `DirectScrutinee` answer at this seat.
    RecursorLayerSupplied {
        checked_frame_id: Option<u64>,
        selects_occurrence: bool,
        route: SourceComputationalAnswerRoute,
    },
    /// **THE CONSUMER.** The carried computational-match seat, with the route
    /// its predecessor handed it, the frame's own recursor-layer field, and
    /// the join of the two.
    ///
    /// ⚠ All three are recorded because the join is the mechanism: `incoming`
    /// alone cannot show that the frame's field failed to overwrite it, which
    /// is exactly the drop measured at `ae45e804`.
    ConsumerRoute {
        seat: D6aConsumerSeat,
        static_origin: StaticOriginId,
        incoming: SourceComputationalAnswerRoute,
        frame_field: SourceComputationalAnswerRoute,
        joined: SourceComputationalAnswerRoute,
    },
    /// The carried elimination this consumer handed its frame to was actually
    /// entered. ⭐ Distinct from `ConsumerRoute`, which records only that a
    /// seat computed a route — a seat can do that on a path the eliminator
    /// never runs, and a control that conflated the two would credit a route
    /// with an emission it had nothing to do with.
    CarriedEliminationEntered {
        static_origin: StaticOriginId,
        route: SourceComputationalAnswerRoute,
        cases: usize,
    },
    /// The carried checked-answer fallback was actually emitted, at this
    /// consumer. ⭐ Recorded into the *route* trace as well as the trap
    /// provenance so that one ordered sequence carries the whole edge —
    /// producer, consumer, emission — and a row does not have to correlate two
    /// traces to say which consumer acted.
    CarriedFallbackEmitted {
        static_origin: StaticOriginId,
    },
    /// The carried consumer took its closed default instead. ⛔ Recorded for
    /// **every** reason it can do so, including a `DirectScrutinee` route, so
    /// the trace can never show a consumer that neither emitted nor defaulted.
    CarriedDefaultSealed {
        static_origin: StaticOriginId,
        route: SourceComputationalAnswerRoute,
    },
}

/// Which lowering edge a carried computational-match consumer sits on.
///
/// ⚠ Both edges exist on the governed witness and they arrive at the **same**
/// `StaticOriginId`, which is why the seat has to be recorded: an assertion
/// keyed on the origin alone cannot say which predecessor it is about, and the
/// whole premise of this checkpoint is that the origin does not determine the
/// route.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::cranelift_backend) enum D6aConsumerSeat {
    /// The composed eliminator edge in `lower_computational_match_value_composed`.
    Composed,
    /// The source machine's computational-scrutinee edge.
    SourceMachine,
}

/// Perturbations of the `D6a` upstream transport, **one producer at a time**.
///
/// ⭐ Separating them is the requirement, not a convenience: the frame owes a
/// control showing the recursor-layer producer *stays green independently* of
/// the call-result producer, and a mutation that disabled both could not
/// distinguish "independent" from "jointly dead".
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::cranelift_backend) enum D6aRouteMutation {
    Exact,
    /// Drop **only** producer 2: the exact claimed call result returns
    /// `DirectScrutinee`. ⛔ The recursor-layer producer is untouched.
    DropCallResultRoute,
    /// Drop **only** producer 1: an exact selecting recursor layer supplies
    /// `DirectScrutinee`. ⛔ The call-result producer is untouched.
    DropRecursorLayerRoute,
    /// Let the frame's own field **overwrite** the incoming route instead of
    /// joining with it. ⚠ This reproduces the exact defect measured at
    /// `ae45e804`, and it is the control that the join is load-bearing.
    OverwriteIncomingWithFrameField,
}

#[cfg(test)]
thread_local! {
    static D6A_ROUTE_MUTATION: std::cell::Cell<D6aRouteMutation> =
        const { std::cell::Cell::new(D6aRouteMutation::Exact) };
    static D6A_ROUTE_TRACE: std::cell::RefCell<Vec<D6aRouteEvent>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// How many times the live `D6a` mutation actually fired — the same
    /// inertness guard `D5A_ROUTE_APPLICATIONS` provides, for the same reason.
    static D6A_ROUTE_APPLICATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d6a_route_mutation() -> D6aRouteMutation {
    D6A_ROUTE_MUTATION.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d6a_route_application() {
    D6A_ROUTE_APPLICATIONS.with(|cell| cell.set(cell.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d6a_route_applications() -> usize {
    D6A_ROUTE_APPLICATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d6a_route_event(event: D6aRouteEvent) {
    D6A_ROUTE_TRACE.with(|trace| trace.borrow_mut().push(event));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d6a_route_trace() -> Vec<D6aRouteEvent> {
    D6A_ROUTE_TRACE.with(|trace| trace.borrow().clone())
}

/// Runs `body` under `mutation` with a **cleared** route trace, restoring the
/// exact route on the way out even if `body` unwinds.
#[cfg(test)]
pub(in crate::cranelift_backend) fn with_d6a_route_mutation<T>(
    mutation: D6aRouteMutation,
    body: impl FnOnce() -> T,
) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            D6A_ROUTE_MUTATION.with(|cell| cell.set(D6aRouteMutation::Exact));
        }
    }
    D6A_ROUTE_MUTATION.with(|cell| cell.set(mutation));
    D6A_ROUTE_APPLICATIONS.with(|cell| cell.set(0));
    D6A_ROUTE_TRACE.with(|trace| trace.borrow_mut().clear());
    let _restore = Restore;
    body()
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d5a_marker_event(event: D5aMarkerEvent) {
    D5A_MARKER_EVENTS.with(|events| events.borrow_mut().push(event));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d5a_marker_events() {
    D5A_MARKER_EVENTS.with(|events| events.borrow_mut().clear());
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d5a_marker_events() -> Vec<D5aMarkerEvent> {
    D5A_MARKER_EVENTS.with(|events| events.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d5a_trace() {
    D5A_TRACE.with(|trace| trace.borrow_mut().clear());
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn take_d5a_trace() -> Vec<String> {
    D5A_TRACE.with(|trace| trace.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d5_emitted_declaration_calls() {
    D5_EMITTED_DECLARATION_CALLS.with(|calls| calls.borrow_mut().clear());
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d5_emitted_declaration_calls()
-> Vec<(StaticOriginId, StaticOriginId, cranelift_codegen::ir::FuncRef)> {
    D5_EMITTED_DECLARATION_CALLS.with(|calls| calls.borrow().clone())
}

#[cfg(test)]
fn set_trap_frame_binding_mutation(mutation: TrapFrameBindingMutation) {
    TRAP_FRAME_BINDING_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
fn set_trap_identity_mutation(mutation: TrapIdentityMutation) {
    TRAP_IDENTITY_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
fn set_trap_caller_protocol_mutation(mutation: TrapCallerProtocolMutation) {
    TRAP_CALLER_PROTOCOL_MUTATION.with(|cell| cell.set(mutation));
}

struct Lowering<'a> {
    seed_env: &'a NativeSeedEnvironment,
    /// Everything resolved into the ONE generated function this `Lowering`
    /// emits into. ⛔ See [`FunctionLocalRefs`] — none of it is portable.
    function_local: FunctionLocalRefs,
    declarations: BTreeMap<&'a str, &'a RuntimeDeclaration>,
    /// The closed static plan for this compilation.
    ///
    /// It lives here, rather than as a local of `compile_expr_into_module`,
    /// because every descent needs the checked positional child-origin table to
    /// derive the child's static name, and because a retained closure body is now
    /// resolved *out of this plan* by its static origin.
    ///
    /// ⚠ **The plan borrows the source trees** — `StaticTransitionPlan<'a>` holds
    /// each planned occurrence's term by reference, which is what lets a tag be
    /// resolved back to a body without any site retaining a cloned one. So the
    /// non-escape property is now **load-bearing** rather than incidental, and it
    /// is not argued from the absence of borrows:
    ///
    /// `CompiledModule<M>` has **no lifetime parameter** and takes only owned
    /// data, so nothing borrowed can be stored in it — the compiler rejects it.
    /// `escaping_a_source_borrow_into_the_compiled_artifact_does_not_typecheck`
    /// pins exactly that, by requiring `CompiledModule: 'static`; give the
    /// artifact a borrowed field and the pin stops compiling.
    static_transition_plan: StaticTransitionPlan<'a>,
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
    /// ⛔⛔ **`AC-C4`'s TERMINATION GUARD — the carried computational
    /// eliminations currently being emitted, by their frame's static origin.**
    ///
    /// ⚠ **Why a guard is needed at all, and it is not a scope choice.** A
    /// *specialized* recursive elimination terminates because its residual is a
    /// compile-time value that gets strictly smaller — `Suc(Suc(Zero))` reaches
    /// `Zero`. A **carried** residual is a runtime word, so nothing shrinks at
    /// compile time: emitting the recursive case emits its induction-hypothesis
    /// invocation, which re-enters the same eliminator, which emits the
    /// recursive case again. ⇒ Inlining a carried recursion **cannot
    /// terminate**, for any recursive datatype, and without this the compiler
    /// does not error — it **hangs and then overflows its stack**.
    ///
    /// ⭐ Keyed on the frame's own `static_origin` rather than a depth count, so
    /// it refuses exactly self-resumption and ⛔ still permits a case body to
    /// eliminate a *different* carried value. A bare depth bound would refuse
    /// legitimate nesting — an over-strengthened guard manufacturing defects.
    ///
    /// ⛔⛔ **This is a TRANSITION SENTINEL, ⛔ not closure — and the successor
    /// that retires it is named.** Steward decision 2026-07-28 (`C1 §2g-i`'s
    /// amendment block): `AC-C4` splits, `C1` keeps the representation half, and
    /// **`RT-FNSPLIT-B2F` owns the runtime invocation** inside its existing
    /// atomic target/switch boundary. ⇒ `B2F` emits one closed, recursively
    /// callable target per static computational-eliminator origin and turns a
    /// zero-argument structural IH into a **direct call to that same target**;
    /// at that point this stack, and the refusal it guards, come out.
    ///
    /// ⚠ **`B2F`'s termination premise is a DIFFERENT argument from this
    /// guard's, and that is the point.** This refuses because nothing shrinks at
    /// **compile** time. `B2F` may call because the producer→validator boundary
    /// has already established a **finite acyclic carrier graph** and the call
    /// rides a **declared recursive child edge** — so its measure is strict
    /// descent in that validated graph, ⛔ never compile-time shrinkage.
    active_carried_computational_eliminations: Vec<(StaticOriginId, cranelift_codegen::ir::Block)>,
    native_join_plan: Option<crate::NativeJoinPlanV1>,
    consumed_join_sites: BTreeSet<u64>,
    root_terminal_authority: Option<RootTerminalAnswerAuthority>,
    active_join_site: Option<u64>,
    oriented_subcontinuation_plan: Option<crate::OrientedSubcontinuationPlanV1>,
    consumed_subcontinuation_frames: BTreeSet<(u64, u64)>,
    active_subcontinuation_frame: Option<u64>,
    consumed_recursive_call_templates: BTreeSet<u64>,
    pending_recursive_call: Option<CheckedRecursiveInvocationInstance>,
    pending_computational_ih_call: Option<PendingCheckedIhCall>,
    active_recursive_invocations: Vec<CheckedRecursiveInvocationInstance>,
    next_recursive_invocation_instance: u64,
    dynamic_splice_edges: BTreeMap<DynamicSpliceEdgeId, DynamicSpliceEdge>,
    next_dynamic_splice_edge: u64,
    assumptions: BTreeSet<String>,
    unsupported: Vec<String>,
    body_emission_authority: BodyEmissionAuthority,
    /// **`RT-CONTSPEC-ACTIVATE` `D3`** -- the affine claim ledger, held across
    /// the whole unit-definition pass so a token claimed at one producer
    /// occurrence cannot be claimed again at another.
    continuation_claims: Option<units::ContinuationClaimLedger>,
    /// **`RT-DECL-CLOSURE-PORT` `D5a`** — the emission owner of the context
    /// currently being defined, in the generalized domain.
    ///
    /// ⛔ Deliberately a separate field from `defining_unit` rather than a
    /// function of it. A generated specialization context lowers a raw body, so
    /// deriving one from the other would make the raw body's predeclared owner
    /// stand in for the generated context — the exact conflation
    /// `evt_609am4v7cdt5b` ruled against.
    defining_emission_owner: Option<ContinuationEmissionOwner>,
    /// **`RT-DECL-CLOSURE-PORT` `D7`** — the exact declared function whose body
    /// is being emitted, which scopes this body's allocation EVENTS.
    ///
    /// ⛔ Evidence scope only, never planner authority. It is a separate field
    /// from `defining_unit` and `defining_emission_owner` because it answers a
    /// different question: those name who the planner says owns the emission,
    /// this names the module definition the events belong to. One
    /// `PredeclaredFunctionId` can be built as more than one `FuncId` body.
    defining_function_id: Option<FuncId>,
    /// **`RT-DECL-CLOSURE-PORT` `D7`** — the aggregate-allocation event ledger,
    /// held across the whole emission pass.
    ///
    /// `None` outside that pass, which is the only place an aggregate
    /// allocation can be attributed to a declared function.
    aggregate_allocations: Option<AggregateAllocationLedger>,
    /// `D7` — the consumed side of the host-effect seat authority. `None`
    /// outside the emission pass, where a bare rig defines no function and there
    /// is no population to close against.
    host_effect_seats: Option<EffectSeatLedger>,
    /// **`RT-DECL-CLOSURE-PORT` `D5`** — the checked-call closeout ledger.
    /// `None` outside the functionized unit-bundle pass, which is the only
    /// place a checked call can reach a declaration-owned unit.
    checked_call_ledger: Option<units::CheckedCallLedger>,
    /// The exact unit currently being defined, so `D3`'s owner check compares
    /// against a fact supplied independently of the token.
    defining_unit: Option<PredeclaredFunctionId>,
    process_object: bool,
    process_symbols: crate::NativeProcessSymbols,
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
    outer_env: Vec<LoweringEnvironmentBinding>,
    /// The origin of the computational-match occurrence these cases came from,
    /// cloned into this payload **in the same constructor as the cases** so a
    /// later resumption can still derive a case body's origin positionally
    ///.
    static_origin: StaticOriginId,
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
        /// The already-resolved constructor identity, carried with the template.
        ///
        /// `Some` for a compiler-synthesized constructor whose identity came
        /// from the semantic plane's closed role capability, **and also for a
        /// source constructor whose template may outlive its own occurrence** --
        /// which is every source constructor the `Construct` lowering arms
        /// build. The name is narrower than the population it now has; read the
        /// field as "identity resolved at the producer", not "this constructor
        /// is synthesized".
        synthesized_identity: Option<ConstructorIdentity>,
        /// **`RT-DECL-CLOSURE-PORT` `D7` -- the planner-issued aggregate
        /// occurrence this template will become.**
        ///
        /// Carried for exactly the reason `synthesized_identity` is, stated in
        /// that field's own words: a later unit boundary may receive this result
        /// after nested producer traversal, where the caller occurrence is not
        /// the constructor occurrence and therefore cannot lawfully re-query the
        /// plan. The allocation lane is the second fact with that property, so
        /// it travels the same way. Same shape as
        /// [`Lowered::DeclarationClosure::reference`].
        ///
        /// `None` is a REFUSAL, never a default. An emitter reaching an
        /// aggregate with no interned occurrence must fail as loudly as one
        /// reaching an origin with no ownership record: answering
        /// `PersistentGround` would reinstate the unproven persistent lane for
        /// precisely the producers the population still misses.
        occurrence: Option<AggregateOccurrenceId>,
        args: Vec<Lowered>,
    },
    Record {
        /// **`RT-DECL-CLOSURE-PORT` `D7` — the planner-issued aggregate
        /// occurrence this template will become.**
        ///
        /// ⭐ Exactly what `Lowered::Constructor::occurrence` is, and here for
        /// the same reason: **the aggregate's authority belongs to its
        /// PRODUCER, not to whatever coordinate happens to be in scope where it
        /// is used.** A record forwarded through a `Var` or handed to a call
        /// arrives somewhere that cannot lawfully re-query the plan for it.
        ///
        /// ⛔ Its absence is what made a source `Record` resolve its ownership
        /// record at the coordinate of its *use* — a callee's scheduling entry,
        /// a call argument's slot — and be refused on the shape cross-check
        /// against whatever record lived there. The `Constructor` variant never
        /// had that defect because it already carried this.
        ///
        /// `None` is a REFUSAL, never a default, exactly as on `Constructor`:
        /// an aggregate with no interned occurrence has no lifetime meet, and
        /// answering `PersistentGround` for it would reinstate the unproven
        /// persistent lane. A value-domain record — one built from a
        /// `RuntimeValue`, with no occurrence in the program — carries `None`
        /// and fails closed at the allocation rather than borrowing a lane.
        occurrence: Option<AggregateOccurrenceId>,
        fields: Vec<LoweredRecordField>,
    },
    /// A retained closure. ⭐ **The body is the static origin and nothing else.**
    ///
    /// `body` used to be an `OwnedSourceOccurrence` — a cloned `RuntimeExpr`
    /// carried beside its origin. That is the shape `RT-NATIVE-FNSPLIT` exists to
    /// remove: it let a *dynamic* property (which term this value happens to hold)
    /// name *static* code, so two authorities described one body and the cloned
    /// one won.
    ///
    /// Now the value names its body the way the planner already named it, and the
    /// term is recovered from the plan by that name alone
    /// (`Lowering::retained_body_occurrence`). ⛔ Do not reintroduce a term here,
    /// not even "for convenience": a carrier holding both would restore the two
    /// authorities, and the origin would be back to decorating a body rather than
    /// selecting one.
    ///
    /// ⚠ This does **not** change *when* a body is lowered. Each call site still
    /// re-lowers the resolved term in its own whole configuration — that is
    /// symptom-inventory entry 2, and it stays open for `RT-FNSPLIT-B2F`.
    Closure {
        /// **`D7` — the PHASE-BEARING capture edge.**
        ///
        /// A retained callable is an invocation-local compiler control capsule:
        /// the capsule itself is specialized, but the values it closes over
        /// reached it at their own phases, and a capture that arrived through a
        /// declared ABI slot is `Carried`. Storing `Lowered` here forced every
        /// such capture to be read as a compile-time template, which is why a
        /// lawfully mixed environment could not be represented at all.
        ///
        /// ⛔ **A carried capture does not become a `Lowered` value.** It gains
        /// no [`LoweredVariant`], no [`BoundaryDisposition`], no encoding
        /// policy, no inverse conversion, no carrier tag, no durable slot and no
        /// independent callable identity. The governing line is **phase
        /// identity, not transitive Rust containment**: the word stays typed as
        /// a [`LoweringOperand`] end to end.
        ///
        /// ⛔ **The capsule stays unconditionally non-transferable**, and
        /// [`Lowering::boundary_transfer_admissibility`] refuses it **before**
        /// descending into these captures — a carried capture must never become
        /// a way to reach the carrier through a callable that is itself refused.
        captures: Vec<LoweringOperand>,
        params: Vec<String>,
        body: StaticOriginId,
    },
    DeclarationClosure {
        /// **`RT-DECL-CLOSURE-PORT` `D4` — the planner-issued
        /// `DeclarationRef` occurrence this binding was produced at.**
        ///
        /// ⭐ Not a decoration and not derivable from `symbol`: the resolved
        /// declaration-call record is keyed by the **reference** occurrence, so
        /// two references to one declaration are two distinct call sites with
        /// their own targets. Carrying it here is what lets the `Call` consumer
        /// emit *this* reference's call instead of looking one up by name.
        ///
        /// ⛔ It is not a second body authority — `body` remains the sole
        /// callable-body identity, exactly as `AC-1` requires.
        reference: StaticOriginId,
        symbol: RuntimeSymbol,
        /// **`D7` — the same phase-bearing capture edge as
        /// [`Lowered::Closure::captures`], for the same reason.** A declaration
        /// closure's *lexical* captures reach it at their own phases; its *seed*
        /// captures are resolved to JIT-time ground values and are therefore
        /// always constructed as explicit `Specialized`.
        captures: Vec<LoweringOperand>,
        params: Vec<String>,
        body: StaticOriginId,
    },
    /// ⭐⭐ **The one `Lowered` child position that is a [`LoweringOperand`], by
    /// the Architect's `AC-C4` SINGLE-FIELD LICENSE — ⛔ not a precedent.**
    ///
    /// `residual` is the value the saved recursor **continues on**. `§2h`'s
    /// phase closure therefore requires this edge to preserve `Carried`:
    /// eliminating a carried scrutinee whose case declares a recursive position
    /// builds an induction hypothesis over a **carried** child, and treating
    /// that child as a compile-time template is precisely the defect that
    /// closure exists to forbid.
    ///
    /// ⭐ **Why this is not the `§2g` violation it looks like.** The governing
    /// line is **phase identity, not transitive Rust containment.** `§2g`
    /// forbids a carried word from *becoming* a `Lowered` inhabitant or
    /// acquiring a [`LoweredVariant`], a [`BoundaryDisposition`], an encoding
    /// policy, or an inverse conversion. None of those happens here: the word
    /// stays typed as a `LoweringOperand` end to end, and the outer object
    /// remains a specialization-only, in-flight **control capsule**.
    /// `Specialized` classifies how *that capsule* is consumed; it never
    /// asserted that every operand edge the capsule owns is itself specialized.
    ///
    /// ⛔ **The license is this field and nothing else.** `Constructor`,
    /// `Record`, `Closure`, `DeclarationClosure` and every other child position
    /// stay `Lowered`. ⛔ No third `LoweringOperand` variant, ⛔ no
    /// `Lowered::Boundary`, ⛔ no `Carried -> Lowered` conversion, ⛔ no durable
    /// closure lane, ⛔ no encoder/decoder row, ⛔ no carrier tag.
    ///
    /// ⚠ **The capsule itself stays unconditionally non-transferable, and the
    /// ORDERING is part of the ruling.** The admission walk must reject a
    /// `ComputationalRecursorClosure` *before* inspecting or emitting its
    /// residual — see [`Lowering::boundary_transfer_admissibility`] and the
    /// [`LoweredVariant::ComputationalRecursorClosure`] `FailClosedForbidden`
    /// row, both of which match `{ .. }` and refuse without descending. A
    /// carried residual must not become a way to reach the carrier through a
    /// capsule that is otherwise refused.
    ComputationalRecursorClosure {
        residual: Box<LoweringOperand>,
        activation: ContinuationActivationId,
        invocation: RecursorInvocationSegment,
    },
    /// A tail-recursive edge already emitted as a CFG jump. The current block
    /// is predecessor-free; enclosing scalar combinators propagate this
    /// marker so it cannot be confused with an ordinary or terminal value.
    RecursiveBackedge,
    Trap(RuntimeTrap),
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — one field of a lowered record, with its
/// producer-issued typed identity beside the value it names.**
///
/// ⭐ **The identity and the value are ONE member, never two lists.** A record's
/// field names are a producer fact — resolved where the record is built and
/// meaningless at any later coordinate — exactly as its ownership record is. A
/// parallel `Vec<FieldIdentity>` beside the values would carry that fact, and
/// would also make a desync spellable: a schema one element short, or ordered
/// differently from the values, still resolves real identities and still emits.
/// Pairing them makes the desync a type error instead of a wrong name.
///
/// ⛔ **`name` is NOT the identity, and the two may never be interchanged.** It
/// is the compile-time spelling, retained because value-domain records and
/// specialized projection are keyed on it; `identity` is the artifact-static
/// word the carrier ABI stores and `Project` looks up (`D1`/`D2`). Deriving one
/// from the other in either direction is the second derivation `D2` forbids —
/// there is no `&str -> FieldIdentity` mapping and none may be added.
#[derive(Clone)]
pub(in crate::cranelift_backend) struct LoweredRecordField {
    /// The compile-time spelling. ⛔ Never an identity.
    pub(in crate::cranelift_backend) name: String,
    /// The planner-issued typed identity, resolved at the record's producer.
    ///
    /// ⛔ `None` is a REFUSAL, never a default, for the same reason
    /// [`Lowered::Record::occurrence`]'s absence is: a value-domain record has
    /// no occurrence in the program and therefore no planned schema, and
    /// emitting a name for it would mean inventing one. It fails closed at the
    /// preflight rather than borrowing a name from the coordinate it is
    /// transferred at.
    pub(in crate::cranelift_backend) identity: Option<FieldIdentity>,
    pub(in crate::cranelift_backend) value: Lowered,
}

/// The boundary-carrier helpers this generated function may call.
///
/// ⛔ **Exactly the helpers the four carrier routes use, and no more.** A ref
/// declared here that no route calls is inert threading — the defect this node
/// exists to avoid — so the set is kept minimal and every member is reached by
/// the one-way producer ([`Lowering::transfer_into_carrier`]), the
/// `Match`/`ComputationalMatch` route, or the `Project` route.
///
/// ⚠ **The set grew by two when the producer landed, and the reason is worth
/// recording rather than absorbing.** The original comment said *"exactly the
/// helpers the three **eliminators** use"* — which under-counts by exactly the
/// producer, because `§2g` names it as a fourth route (*"the boundary producer
/// has a one-way typed seam ... it consumes the sole `BoundaryLocalFuncs`
/// authority"*). `make_immediate` and `store_name` are producer-only: an
/// eliminator never mints an immediate and never writes a field name.
#[derive(Clone, Copy, Debug)]
struct BoundaryCarrierRefs {
    /// `(arena, word, out) -> status` — the representation class that selects
    /// representation-specific consumers such as `HostResult`.
    class: FuncRef,
    /// `(arena, word, out) -> status` — the runtime constructor/record identity
    /// that `Match` discriminates against the artifact-static case set.
    tag: FuncRef,
    /// `(arena, word, out) -> status` — the child count a case's binder arity
    /// is checked against **at runtime**.
    field_count: FuncRef,
    /// `(arena, word, index, out) -> status` — positional child projection.
    /// Its result stays [`LoweringOperand::Carried`].
    field: FuncRef,
    /// `(arena, word, name_id, out) -> status` — `Project` by artifact-static
    /// field identity.
    record_field: FuncRef,
    /// Runtime scalar extraction for statically scalar consumer positions.
    scalar: FuncRef,
    /// HostResult runtime discriminant and selected payload.
    host_success: FuncRef,
    host_payload: FuncRef,
    /// `(arena, tag, class, field_count, out) -> status` — the one-way
    /// producer's allocation step.
    alloc: FuncRef,
    /// `(arena, word, tag_id) -> status` — the producer records the identity.
    store_tag_id: FuncRef,
    /// The producer records an inline scalar or HostResult discriminant.
    store_scalar: FuncRef,
    /// `(arena, word, index, child) -> status` — the producer writes children.
    store_field: FuncRef,
    /// `(arena, word, index, name_id) -> status` — the producer writes a
    /// record's field names, so [`Self::record_field`] can find one by
    /// artifact-static identity. ⛔ No `arena`-free shortcut: the name a
    /// `Project` looks up and the name the producer wrote must be the same
    /// word, which is `D2` at the field-identity namespace.
    store_name: FuncRef,
    /// `(tag, payload, out) -> status` — the producer's leaf step for a value
    /// whose payload rides in the tagged word. ⚠ Note the **absent `arena`**:
    /// an immediate names no referent, so there is nothing for an arena to own.
    make_immediate: FuncRef,
    /// `(arena, word, native_tag) -> status` — the spill arm records **how** the
    /// magnitude word is to be read, as a `NativeIntV1` marker.
    ///
    /// ⛔ **This is also the spill arm's region guard, and it is deliberately
    /// not re-derived here.** `BOUNDARY_INT_MARKER_OWNER` says a
    /// `NATIVE_INT_BIG_TAG_V1` payload is a slot in the *invocation's* native
    /// arena and therefore inadmissible on a persistent node. The helper
    /// enforces that from the one table; the producer neither restates the rule
    /// nor pre-empts it — see [`Lowering::emit_carrier_spillable_immediate`] for
    /// the residual that leaves.
    store_int_tag: FuncRef,
    /// `(arena, word, len, out) -> status` — claim `len` content bytes in the
    /// node's own region. ⭐ A **claim-then-fill** protocol: the span exists
    /// before a byte of it is written, so a length the region cannot satisfy
    /// fails before any address is formed.
    store_bytes_len: FuncRef,
    /// `(arena, word, index, byte) -> status` — write one content byte.
    store_byte: FuncRef,
    /// `(arena, word, sign, len, out) -> status` — claim `len` magnitude limbs
    /// in the node's **own** region for a region-limbed `Int`.
    store_int_limbs: FuncRef,
    /// `(arena, word, index, limb) -> status` — write one magnitude limb.
    store_int_limb: FuncRef,
    /// `(arena, word) -> status` — check a region-limbed `Int`'s magnitude
    /// canonical and seal it. ⛔ **Until this succeeds the node denotes
    /// nothing**, so it is the last step of the copy and never optional.
    seal_int: FuncRef,
    /// `(arena, word, out_view) -> status` — canonical exact-`Int` view.
    int_view: FuncRef,
}

/// A value that has crossed into the **operational carrier** — nothing but the
/// Cranelift SSA boundary word (`RT-FNSPLIT-C1` `D3`).
///
/// ⛔ **It holds the word and NOTHING ELSE, and the emptiness is the point.**
/// No constructor string, no field list, no body or template, no tag/class, no
/// reverse-decoding data. Every question about this value — which constructor,
/// how many fields, which child — is answered by **calling an emitted helper at
/// runtime**, never by reading a field of this struct. ⇒ The struct having room
/// for a compile-time answer is exactly how the wall would grow back.
#[derive(Clone, Copy, Debug)]
struct CarriedBoundaryWord {
    word: cranelift_codegen::ir::Value,
}

/// ⭐ **The closed PHASE sum — which phase a lowering operand is in, not what
/// kind of value it is** (`RT-FNSPLIT-C1` `D3`).
///
/// ⛔ **THE RULING IS `§2g` OF THE C1 FRAME — cite it, do not restate it.**
/// `docs/program/wp/RT-FNSPLIT-C1-operational-carrier.md §2g` carries Architect
/// Decision `dec_4te25repm33ph`'s resolution verbatim. Where the frame and any
/// restatement disagree, **the frame governs**.
///
/// ⚠ An earlier revision of this comment pasted the ruling text here. That was
/// a **second authority** — the exact defect this chain keeps paying for — and
/// it is deleted rather than kept "for convenience": two copies of a ruling
/// drift silently, and the copy nearest the code is the one a reader trusts.
///
/// ⭐ The clause that shapes the surface census, and the one a
/// three-eliminator reading misses: **environments and result surfaces that can
/// receive a transferred value carry this wrapper.** Every other clause is
/// prohibitive; this one *adds* obligated surface.
///
/// ⚠ The **name** `LoweringOperand` is this implementation's choice and is not
/// ruled; the **shape** is.
///
/// ⛔ **NOT a variant of [`Lowered`].** `Lowered` is a compile-time
/// specialization lattice (`§2f`); a `Lowered::Boundary` inhabitant is the
/// `B2E` shape the inertness rule rejects — the inhabitant is the easy half and
/// the three executable eliminations are the node.
///
/// ⛔ **One-way.** There is a producer into [`Self::Carried`] and deliberately
/// **no** inverse: a `Carried → Lowered` conversion would let a consumer
/// recover a compile-time template from a runtime value, which is the wall
/// itself wearing a different name.
// ⛔ No `Debug`: `Lowered` has none, and deriving one here would be a new,
// second way to read a compile-time template out of an operand.
#[derive(Clone)]
enum LoweringOperand {
    /// The compile-time specialization lattice — every route that existed
    /// before this node. ⛔ Kept as an **explicit** arm, never a fallback:
    /// a fallback arm is a wildcard with better manners.
    Specialized(Lowered),
    /// A runtime boundary word, eliminated only by emitted helpers.
    Carried(CarriedBoundaryWord),
}

/// **THE ONE BINDING AUTHORITY** for a lexical environment (`RT-WORKER-BIND`
/// judgment 4). Every lexical environment reaching [`Lowering::lower_expr`] is
/// a slice of these -- saved ordinary and computational eliminator
/// environments, pending-`Let` environments, and recursor outer environments
/// alike.
///
/// There is deliberately **no** parallel operand environment and **no**
/// de-Bruijn side map. Either would create a second binding authority, and then
/// the question *"what is bound here"* would have two answers.
///
/// This sum is compiler-only. It is **not** a [`Lowered`] variant, not a third
/// [`LoweringOperand`] arm, and it never becomes a runtime value.
#[derive(Clone)]
enum LoweringEnvironmentBinding {
    /// An ordinary bound value. Every binder that existed before this node
    /// installs this arm, and the outer spine forwards it unchanged.
    Value(LoweringOperand),
    /// A statically-bound worker: a lexical callable whose body is a declared
    /// static-body unit. Its sole admissible use is as the callee of a `Call`
    /// with an exact `Var` callee; every value-producing position rejects it.
    ///
    /// `D2` constructs this arm from a lexical closure's retained occurrence.
    /// The callee-only consumer is `D3`, so nothing *reads* the binding's
    /// fields in production yet -- see the note on [`StaticWorkerBinding`].
    StaticWorker(StaticWorkerBinding),
}

/// The compiler-only description of a static worker binding.
///
/// It carries no runtime word, tag, layout, vtable, descriptor or environment
/// pointer, and no callable identity. `captures` stay [`LoweringOperand`], so a
/// carried capture stays carried -- the binding never becomes a value.
///
/// A `FuncRef` deliberately does **not** live here: it belongs to one Cranelift
/// `Function`, and the target is declared afresh into each generated function
/// (`D4`).
///
/// `D3` reads `body_origin`, `declared_arity` and `captures`. It does **not**
/// read `closure_origin`, so the allowance is narrowed to that one field
/// rather than blanketing the struct.
///
/// `closure_origin` is preserved because judgment 1 rules these four fields
/// and the separation may not vary; it is the binding's own occurrence
/// identity, which no consumer needs today. Narrowing the allowance is what
/// keeps a future genuinely-unread field visible instead of silently covered.
#[derive(Clone)]
struct StaticWorkerBinding {
    #[allow(dead_code)]
    closure_origin: StaticOriginId,
    body_origin: StaticOriginId,
    declared_arity: u32,
    captures: Vec<LoweringOperand>,
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D6a`** — which callee this binding's call
    /// goes to, carried rather than reconstructed. See
    /// [`StaticWorkerCallRoute`].
    ///
    /// ⛔ **Deliberately NOT `#[allow(dead_code)]`.** `D6a` is the
    /// representation; the route's consumption at the call edge is `D6b`, so
    /// production reads nothing here yet and the compiler says so. That warning
    /// is the obligation staying visible, exactly as the narrowed allowance on
    /// `closure_origin` above intends — silencing it would turn an open
    /// checkpoint into an invisible one.
    route: StaticWorkerCallRoute,
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8i`** — which causal obligation this
    /// binding's consumption may satisfy. See [`ContinuationDischarge`].
    ///
    /// ⛔ **A separate facet from `route`, deliberately.** The route decides the
    /// callee and the operand run; the discharge decides which causal call, if
    /// any, a consumption of this binding is allowed to answer for. They are
    /// independent: the composed selected recursive argument is `RawWorker` and
    /// carries an authority, while an ordinary `RawWorker` induction hypothesis
    /// carries none. Folding the authority into the route would make "which
    /// callee" and "which obligation" one field, and `D6b` already showed what
    /// happens when two contracts share one discriminator.
    ///
    /// ⛔ **The open checkpoint is visible on the READER, not on this field.**
    /// The field is read by [`StaticWorkerBinding::
    /// composed_continuation_authority`], so the compiler does not name it
    /// unread — and it would be wrong to claim it does. That accessor is what
    /// carries the narrowed allowance until `D8j` supplies its production
    /// consumer, and it is the thing to check when asking whether `D8i` is
    /// still transport-only.
    discharge: ContinuationDischarge,
}

impl StaticWorkerBinding {
    /// **`D8i` — the composed causal authority this binding carries, or a
    /// refusal.**
    ///
    /// ⛔ **An ordinary binding is REJECTED, not answered with `None`.** A
    /// caller asking this question is asking "which composed obligation may I
    /// discharge with this", and for a [`ContinuationDischarge::
    /// DirectSpecializationCall`] binding the answer is not "none available" —
    /// it is that the question does not apply to it. `None` would let a caller
    /// treat the two cases alike with `unwrap_or_default`, `is_some`, or an
    /// `if let` that silently skips; a `Result` makes ignoring it a written
    /// decision.
    ///
    /// ⭐ Returned by reference and still opaque. A caller can compare it, key
    /// on it, and read its target specialization and emission owner. It cannot
    /// read the call-site sequence and cannot construct one.
    ///
    /// `D8j` is this method's production consumer, and the narrowed allowance
    /// that stood here is DELETED rather than widened: the composed discharge
    /// obtains its authority through this accessor, so a refusal is on the live
    /// path rather than behind a `cfg`.
    fn composed_continuation_authority(
        &self,
    ) -> Result<&ContinuationCallIdentity, CraneliftBackendError> {
        match &self.discharge {
            ContinuationDischarge::ComposedSourceContinuation(identity) => Ok(identity),
            ContinuationDischarge::DirectSpecializationCall => Err(unsupported(
                "StaticWorkerBinding",
                "an ordinary static-worker binding carries no composed causal authority: its \
                 discharge is a direct specialization call, which answers for no composed \
                 source continuation. This refuses rather than reporting an absence, because a \
                 consumption that reached here has already decided it is discharging a composed \
                 obligation and there is none to discharge",
            )),
        }
    }
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8f` — a pending checked-IH marker, and the
/// exact application occurrence it denotes.**
///
/// ⛔⛔ **The occurrence is the whole point of this type existing.** The marker
/// used to be a bare template id, so "a marker is pending" was the only fact a
/// consumption site had — and the site is the static-worker call arm, which
/// every static-worker call reaches. A pending marker therefore meant *the next
/// static-worker call consumes it*, and inside one checked wrapper the next
/// call need not be the checked one: the wrapped application's own ARGUMENTS
/// are evaluated first, and an argument can be an ordinary call on the selected
/// recursive argument.
///
/// ⭐ `application_origin` is the occurrence of the expression the marker
/// wraps, taken from the same `child_origin(marker, 0)` the lowering already
/// derives to lower it. That is the existing checked occurrence authority
/// projected faithfully — the marker denotes the complete application
/// occurrence, which this file already states — and it is not a route, an
/// arity, a binder index, a call ordinal, or any inferred shape.
#[derive(Clone, Copy)]
struct PendingCheckedIhCall {
    call_template_id: u64,
    /// The occurrence of the application this marker denotes. Only a call being
    /// lowered AT this occurrence may consume the marker.
    application_origin: StaticOriginId,
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8j` — what the shared static-worker emitter
/// hands back beside the call's value.**
///
/// ⭐ The instruction is returned rather than recorded by the emitter, because
/// the emitter serves BOTH consumers — direct descent and the source machine —
/// and only one of them may discharge a composed obligation. An emitter that
/// recorded would be deciding a question that belongs to its caller.
#[derive(Clone, Copy)]
struct StaticWorkerEmission {
    /// The raw-worker call instruction actually written.
    inst: cranelift_codegen::ir::Inst,
    /// The operand run supplied to it: explicit arguments plus stored captures
    /// plus any route suffix. Compared against the `D8b`/`D8d` target's own
    /// declared contract at verification.
    supplied_operands: usize,
}

/// **`D8j` — one CLAIMED composed discharge, awaiting the finished CLIF.**
///
/// ⛔ Every field is recorded at the seat from a value the seat already held.
/// Nothing here is re-derived at verification time from another field, because
/// then the verification would be comparing the record with itself.
struct PendingComposedDischarge {
    /// The opaque authority the binding transported.
    identity: ContinuationCallIdentity,
    /// The raw-worker call this discharge answers with.
    inst: cranelift_codegen::ir::Inst,
    /// The worker body the `D8b`/`D8d` target names — the callee the decoded
    /// instruction must resolve to.
    worker_body_origin: StaticOriginId,
    /// The target's declared contract: arity plus stored captures.
    declared_operands: usize,
    /// What the emitter actually supplied.
    supplied_operands: usize,
    /// The call's result word, as it was handed back to source-machine control.
    /// `None` when the result was not a carried word, which is itself a
    /// verification failure — recorded rather than rejected at the seat so the
    /// refusal happens in one place.
    result: Option<cranelift_codegen::ir::Value>,
    /// Live source continuations immediately before the emitter ran and
    /// immediately after the result was placed. Equal means the result returned
    /// into the continuation that was already in force.
    source_control: (usize, usize),
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8i` — the closed causal-discharge facet.**
///
/// ```text
/// ContinuationDischarge =
///   DirectSpecializationCall
/// | ComposedSourceContinuation(opaque ContinuationCallIdentity)
/// ```
///
/// ⭐ **Two arms, both explicit, no default and no `Option`.** Every
/// [`StaticWorkerBinding`] states its arm at construction because
/// [`Lowering::construct_static_worker_binding`] takes it as a required
/// argument — omission is a compile error, not a silent
/// [`Self::DirectSpecializationCall`]. That is the whole reason it is a
/// parameter rather than a field the composed path patches afterwards: a
/// defaulted facet would make "this binding carries no authority" and "nobody
/// said" the same value, and those are the two states this type exists to keep
/// apart.
///
/// ⛔ **Transport only, at this checkpoint.** Nothing here consumes, records or
/// closes the authority. The authority is carried opaquely — a
/// [`ContinuationCallIdentity`] has no sequence accessor and no lowering
/// constructor — so a binding can hand one on but can never mint one, and no
/// path can invent an obligation it is entitled to discharge.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ContinuationDischarge {
    /// The ordinary binding: its call is a direct specialization call and
    /// answers for **no** composed causal obligation.
    ///
    /// ⛔ This is a positive statement, not an absence. Induction hypotheses,
    /// selected recursive arguments built by a specialization context, and
    /// lexical-closure capsules all carry it explicitly.
    DirectSpecializationCall,
    /// **The composed selected recursive argument.** Carries the exact opaque
    /// identity `D8h` paired with the [`ComposedCallTarget`] this binding was
    /// derived from — the identity that target's own five-field coordinate
    /// selects, transported unchanged.
    ///
    /// [`ComposedCallTarget`]: crate::cranelift_backend::planning::ComposedCallTarget
    ComposedSourceContinuation(ContinuationCallIdentity),
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D6a` — the closed compiler-only call route.**
///
/// Two bindings can name the **same** closure occurrence, body origin, declared
/// arity and ordered captures and still be different callables, because they
/// reach that body by different routes. A specialization's induction hypothesis
/// and its selected recursive constructor argument are exactly that pair: the
/// same closure, distinguished by where the call goes.
///
/// ⭐ **THE ROUTE LAW, and it is not symmetric.**
///
/// - [`Self::RawWorker`] is what a **`SelectedRecursiveArgument`** carries,
///   **always**. The source scope binds the closure itself, so there is no
///   condition to evaluate and no case in which it carries the other arm.
/// - A **`InductionHypothesis`** carries [`Self::GeneratedContext`] **iff** the
///   planner issued a generated execution context for this
///   `(specialization, worker body)` pair *and* this unit resolved it.
///   Otherwise it **lawfully carries `RawWorker` too**.
///
/// ⛔ **So equal routes on the two members is a LAWFUL state, not a defect.** A
/// unit whose planner issued no context is *route-degenerate*: both members
/// render `RawWorker`, and they remain two distinct bindings for two distinct
/// environment positions. Reading "the two rendered routes are equal" as "one
/// binding was reused for both members" is an invalid inference, and prose or a
/// control that draws it is wrong about the mechanism rather than strict about
/// it. The membership-and-order facts are what separate the members in the
/// degenerate case; the route separates them only where a context exists.
///
/// ⛔ **It is carried, never derived at the call site.** The pre-`D6a` code
/// decided the suffix by comparing `generated_context_captures.worker_body_
/// origin` against the binding's `body_origin`, and selected the callee by
/// whichever entry the retarget had left in `worker_calls`. Both readings are
/// *blind here by construction*: the two bindings share a body origin, so a
/// body-origin comparison answers the same for both, and one map entry cannot
/// name two routes. Body shape, declared arity, use site, environment length
/// and "whichever target exists" are all equally blind for the same reason —
/// which is why this is a field and not a predicate.
///
/// The route is fixed at construction, from the role the binding is built for
/// together with the planner's own issuance — the planner either issued a
/// generated execution context for this `(specialization, worker body)` pair
/// and this unit resolved it, or it did not. Neither input is recoverable at
/// the call edge, which is why both are read here.
///
/// ⛔ No `FuncRef` lives here. The route names *which declared-call table* a
/// caller resolves its target from; the target itself is still minted into the
/// calling function, so nothing crosses a function boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticWorkerCallRoute {
    /// The exact **raw worker body**, at its own unretargeted contract.
    ///
    /// Its operand run is the raw run and nothing else: explicit arguments,
    /// then stored captures. It appends **no** continuation-input suffix.
    ///
    /// Three populations take it: every binding built for an ordinary lexical
    /// closure; `D6a`'s selected recursive constructor argument, **without
    /// exception**; and an induction hypothesis in a unit that resolved no
    /// generated execution context — the third being the route-degenerate case
    /// described on the type above, and lawful.
    RawWorker,
    /// The **planner-issued generated execution context** that executes that
    /// same body on behalf of one enclosing specialization.
    ///
    /// The context's ABI has a capture run for the enclosing specialization's
    /// continuation inputs, so a call on this route appends that suffix.
    ///
    /// ⛔ Reached by an induction hypothesis **only when a context was issued
    /// and resolved**, and by nothing else — never by a selected recursive
    /// constructor argument. "The route an induction hypothesis takes" states
    /// the implication in the wrong direction: this arm implies an induction
    /// hypothesis, an induction hypothesis does not imply this arm.
    GeneratedContext,
}

impl LoweringEnvironmentBinding {
    /// **THE FAIL-CLOSED READ** for a value-producing position.
    ///
    /// `Var` in every value-producing position accepts **only** [`Self::Value`]
    /// (`D3`). A [`Self::StaticWorker`] used as a result, aggregate field,
    /// primitive or effect argument, stored value, projection subject, match
    /// scrutinee, or ordinary call argument fails closed **here**, before any
    /// carrier transfer.
    ///
    /// The match is exhaustive with no wildcard, and that is the mechanism: a
    /// future third arm is a compile error at every value-producing read rather
    /// than a silent escape. `edge` names the call site in the diagnostic.
    fn value_at(&self, edge: &'static str) -> Result<&LoweringOperand, CraneliftBackendError> {
        match self {
            LoweringEnvironmentBinding::Value(operand) => Ok(operand),
            LoweringEnvironmentBinding::StaticWorker(_) => Err(unsupported(
                "StaticWorkerBinding",
                format!(
                    "{edge} is a value-producing position and a static worker binding has no \
                     value representation; its only admissible use is as the callee of a call \
                     with an exact Var callee"
                ),
            )),
        }
    }
}

/// A freshly bound operand entering the one binding authority. Every binder
/// that predates this node goes through here, which is what makes *"existing
/// binders install `Value`"* structural rather than tested.
impl From<LoweringOperand> for LoweringEnvironmentBinding {
    fn from(operand: LoweringOperand) -> Self {
        LoweringEnvironmentBinding::Value(operand)
    }
}

impl LoweringOperand {
    /// ⭐ **THE RULED TYPED PHASE BOUNDARY** in front of a specialized-only
    /// helper — `§2h` ¶2, verbatim: *"Every edge from `LoweringOperand` into
    /// such a specialized-only helper must first exhaustively classify both
    /// variants with no wildcard; `Carried` must take its ruled emitted-helper
    /// route **or fail closed**."*
    ///
    /// ⛔ **This is NOT the forbidden `Carried -> Lowered` conversion, and the
    /// difference is the whole point.** A conversion would *answer* a
    /// compile-time question about a runtime value; this **refuses** to. The
    /// `Carried` arm produces an error, never a `Lowered`, so no caller can
    /// recover a template through it.
    ///
    /// ⚠⚠ **THE MISUSE THIS INVITES, NAMED SO A REVIEWER CAN HUNT IT.** Nothing
    /// in production emits a `Carried` operand yet, so **every** call of this
    /// method is currently unreachable in its failing arm — which means putting
    /// it on a *phase-bearing* edge compiles, passes the whole suite, and still
    /// destroys the closure `§2h` mandates. ⇒ It is legal **only** where the
    /// callee inspects or destructures the **compile-time template** (a
    /// primitive's operands, a shape comparison, a constructor-name probe). It
    /// is ⛔ **illegal** on the three forwarding roles, which must stay
    /// `LoweringOperand` end to end:
    ///
    /// | role | why it may not fail closed here |
    /// |---|---|
    /// | environment insertion | `§2h`: a projected `Carried` child must survive `case_env` |
    /// | recursive lowering call | the callee is in the phase-bearing component, not a leaf |
    /// | result / join forwarding | a refused join is a lost `Carried`, not a rejected one |
    ///
    /// `edge` names the call site in the diagnostic. ⚠ It is a **label, not a
    /// mechanism** — it makes a misplaced boundary legible in a failure, and
    /// nothing more.
    /// **The phase this operand is ACTUALLY in.**
    ///
    /// ⛔ An observation, not a decision. It is the one fact the planner's
    /// derived `consumer_phase` is checked against, so it must read the operand
    /// in hand and nothing else — a version of this that consulted the plan
    /// would make the agreement true by construction.
    fn effect_seat_phase(&self) -> EffectSeatPhase {
        match self {
            LoweringOperand::Specialized(_) => EffectSeatPhase::SpecializedTemplate,
            LoweringOperand::Carried(_) => EffectSeatPhase::CarriedWord,
        }
    }

    fn specialized_at(self, edge: &'static str) -> Result<Lowered, CraneliftBackendError> {
        match self {
            LoweringOperand::Specialized(lowered) => Ok(lowered),
            LoweringOperand::Carried(_) => Err(unsupported(
                "BoundaryCarrier",
                format!(
                    "{edge} is a specialized-only surface and a carried boundary word has no \
                     compile-time template for it to read; the carrier's ruled route is an \
                     emitted helper call"
                ),
            )),
        }
    }

    /// ⭐ **A BRANCH/JOIN ARM's typed phase boundary — deliberately a distinct
    /// method from [`Self::specialized_at`], because it records a distinct
    /// fact.**
    ///
    /// `§2h` names *"branch/join forwarding"* phase-bearing, so a join is ⛔
    /// **not** a specialized-only leaf: the reason a `Carried` fails closed here
    /// is not *"this surface reads a template"* but *"this join merges native
    /// scalar lanes and `C1` has not built a carried lane for it."* ⇒ Every call
    /// of this method is an **inventory entry** for the join work, and
    /// `grep`ping the name is how that inventory is read back.
    ///
    /// ⚠ Collapsing the two into one helper would be the cheaper diff and the
    /// worse artifact: it would erase, at exactly the sites that need it, the
    /// difference between a boundary that is *final* and one that is *pending*.
    fn specialized_join_arm(self, join: &'static str) -> Result<Lowered, CraneliftBackendError> {
        match self {
            LoweringOperand::Specialized(lowered) => Ok(lowered),
            LoweringOperand::Carried(_) => Err(unsupported(
                "BoundaryCarrier",
                format!(
                    "{join} merges native scalar lanes and has no carried lane; a boundary word \
                     cannot cross it until that join carries the phase"
                ),
            )),
        }
    }

    /// [`Self::specialized_at`] without consuming the operand — same ruling,
    /// same prohibitions, for a callee that borrows its template.
    fn specialized_ref_at(&self, edge: &'static str) -> Result<&Lowered, CraneliftBackendError> {
        match self {
            LoweringOperand::Specialized(lowered) => Ok(lowered),
            LoweringOperand::Carried(_) => Err(unsupported(
                "BoundaryCarrier",
                format!(
                    "{edge} is a specialized-only surface and a carried boundary word has no \
                     compile-time template for it to read; the carrier's ruled route is an \
                     emitted helper call"
                ),
            )),
        }
    }
}

/// The spine's bulk phase boundary -- a **list of operands** rendered as the
/// specialized templates a specialized-only helper reads: a constructor's
/// arguments, a closure's captures, a primitive's operands.
///
/// Same ruling and the same prohibitions as
/// [`LoweringOperand::specialized_at`]; this exists because several leaves take
/// a whole `&[Lowered]` rather than one, and hand-writing the fold at each
/// would multiply the classification instead of sharing it.
///
/// This is deliberately **not** the environment form. An operand list has no
/// binding to classify, so there is nothing here to fail closed on; a lexical
/// environment goes through [`specialized_bindings_at`] instead, which crosses
/// the binding authority first. The two are named apart so that passing an
/// environment to the operand form is a type error rather than a silent
/// bypass of the value-producing rule.
fn specialized_operands_at(
    operands: &[LoweringOperand],
    edge: &'static str,
) -> Result<Vec<Lowered>, CraneliftBackendError> {
    operands
        .iter()
        .map(|operand| operand.specialized_ref_at(edge).cloned())
        .collect()
}

/// [`specialized_operands_at`] for a **lexical environment** rather than a bare
/// operand list.
///
/// The two are deliberately distinct functions. This one crosses the binding
/// authority first, so a [`LoweringEnvironmentBinding::StaticWorker`] fails
/// closed before any template is read; [`specialized_operands_at`] takes an operand
/// list -- a constructor's arguments, a closure's captures, a primitive's
/// operands -- which is not an environment and has no binding to classify.
/// Collapsing them would put a value-producing read on a surface that has no
/// binding to reject.
fn specialized_bindings_at(
    env: &[LoweringEnvironmentBinding],
    edge: &'static str,
) -> Result<Vec<Lowered>, CraneliftBackendError> {
    env.iter()
        .map(|binding| binding.value_at(edge)?.specialized_ref_at(edge).cloned())
        .collect()
}

/// ⭐ **THE ONE WAY A LOWERING ENVIRONMENT IS BUILT** — freshly bound
/// specialized values in front of the enclosing spine.
///
/// ⭐ **The spine is forwarded UNCHANGED, and that is the `§2h` property, not
/// an implementation detail.** The ruling's control clause asks that *"a
/// projected `Carried` child remains `Carried` through `case_env` and nested
/// lowering"*; that holds here **structurally** — `outer` is already
/// `[LoweringOperand]` and every element is cloned, so there is no arm in which
/// a carried operand could be re-specialized or dropped on the way in.
///
/// `bindings` are `Lowered` because a **binder introduces a compile-time
/// value**: a case's constructor fields, a `Let`'s value, a closure's captures.
/// ⚠ When a binder can be `Carried` — the projected child of an eliminated
/// carrier — it does **not** come through this parameter; it is already an
/// operand and is prepended as one.
fn env_with(
    bindings: impl IntoIterator<Item = Lowered>,
    outer: &[LoweringEnvironmentBinding],
) -> Vec<LoweringEnvironmentBinding> {
    bindings
        .into_iter()
        .map(|lowered| LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(lowered)))
        .chain(outer.iter().cloned())
        .collect()
}

/// ⭐ **The phase-PRESERVING environment constructor** — for bindings that are
/// already operands.
///
/// ⚠ This is the one that matters for `§2h`'s control clause. [`env_with`]'s
/// bindings are templates a binder just introduced; **these** are values the
/// lowering produced, and a projected `Carried` child arrives here. There is no
/// arm that re-specializes or drops one — it is moved into the environment as
/// it stands, which is what makes *"remains `Carried` through `case_env` and
/// nested lowering"* a structural fact rather than a tested one.
/// Bind already-lowered operands into the one binding authority, in order.
///
/// This is the phase-preserving entry every pre-existing binder takes: an
/// operand becomes [`LoweringEnvironmentBinding::Value`] and nothing else, so
/// a projected `Carried` child stays carried on the way in. It exists so the
/// many sites that build an environment out of lowered operands all spell the
/// installation the same way rather than each re-deriving it.
fn bound_values(
    operands: impl IntoIterator<Item = LoweringOperand>,
) -> Vec<LoweringEnvironmentBinding> {
    operands
        .into_iter()
        .map(LoweringEnvironmentBinding::Value)
        .collect()
}

fn env_with_operands(
    bindings: impl IntoIterator<Item = LoweringOperand>,
    outer: &[LoweringEnvironmentBinding],
) -> Vec<LoweringEnvironmentBinding> {
    bindings
        .into_iter()
        .map(LoweringEnvironmentBinding::Value)
        .chain(outer.iter().cloned())
        .collect()
}

/// Append specialized bindings **after** operands already in an environment —
/// the same rule as [`env_with`], for the sites whose leading bindings are
/// themselves lowered operands (a call's arguments) and whose trailing ones are
/// a closure's captured templates.
/// **`D8d` — how many target-derived static-worker bindings were installed.**
///
/// ⛔ An INSTALLATION counter, not a consumption one. `D8e` owns consumption and
/// this says nothing about it. It exists because "the binding is intentionally
/// unreadable" and "the binding was never built" are indistinguishable from the
/// outside, and only one of them is the checkpoint.
#[cfg(test)]
thread_local! {
    static D8D_STATIC_WORKER_BINDINGS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// **`D8d` — how many times the composed site reached a RECURSIVE position.**
///
/// The denominator to `D8D_STATIC_WORKER_BINDINGS`' numerator. Without it,
/// "no binding installed" cannot be told apart from "the site was never
/// reached", and those have completely different consequences for `D8e`.
#[cfg(test)]
thread_local! {
    static D8D_RECURSIVE_SITES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8d_record_site() {
    D8D_RECURSIVE_SITES.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8d_recursive_sites() -> usize {
    D8D_RECURSIVE_SITES.with(std::cell::Cell::get)
}

/// **`D8e` — how many source-machine `Var` callees resolved to a `D8d`
/// binding and were consumed through the shared emitter.**
///
/// The third counter in the pair `D8d` established, and it exists for the same
/// reason: "the consumer is correct but unreached" and "the consumer is wrong"
/// are indistinguishable from a green suite.
#[cfg(test)]
thread_local! {
    static D8E_CONSUMPTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8e_record_consumption() {
    D8E_CONSUMPTIONS.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8e_consumptions() -> usize {
    D8E_CONSUMPTIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8d_record_binding() {
    D8D_STATIC_WORKER_BINDINGS.with(|count| count.set(count.get() + 1));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d8d_bindings() {
    D8D_STATIC_WORKER_BINDINGS.with(|count| count.set(0));
    D8D_RECURSIVE_SITES.with(|count| count.set(0));
    D8E_CONSUMPTIONS.with(|count| count.set(0));
    D8I_DISCHARGES.with(|log| log.borrow_mut().clear());
    D8L2_CONSUMED_FACETS.with(|log| log.borrow_mut().clear());
}

/// **`D8i` — what discharge facet each constructed binding actually carried.**
///
/// ⛔ An OBSERVATION of the transported value, not a second authority: the
/// record is written inside the constructor, from the argument the call site
/// supplied, after every validation has passed. It exists because `D8i` is
/// transport with no production reader until `D8j`, so "both arms occur, each
/// at the site its role dictates" is otherwise unobservable — and a facet that
/// were defaulted or inferred rather than stated would be indistinguishable
/// from one that is stated correctly.
///
/// `None` is [`ContinuationDischarge::DirectSpecializationCall`]. The composed
/// arm records the identity's two readable facts; the call-site sequence stays
/// unread here as everywhere else.
#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct D8iDischargeRecord {
    pub(in crate::cranelift_backend) body_origin: StaticOriginId,
    pub(in crate::cranelift_backend) composed:
        Option<(ContinuationEmissionOwner, ContinuationSpecializationId)>,
}

#[cfg(test)]
thread_local! {
    static D8I_DISCHARGES: std::cell::RefCell<Vec<D8iDischargeRecord>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8i_discharges() -> Vec<D8iDischargeRecord> {
    D8I_DISCHARGES.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8i_discharge(record: D8iDischargeRecord) {
    D8I_DISCHARGES.with(|log| log.borrow_mut().push(record));
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8j` — the composed-discharge defects.**
///
/// ⛔ Every one perturbs an INPUT the seat or the verifier is handed, never the
/// check itself. Where an identity is substituted it is a real one taken from
/// the planned population, because `ContinuationCallIdentity` has no
/// constructor outside planning.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D8jMutation {
    Exact,
    /// Emit the raw call and record nothing. The relation stays empty on a
    /// program that should have populated it.
    SuppressDischargeAfterRealCall,
    /// Present a different exact identity from the same population — the answer
    /// a same-symbol shortcut would give, since the witness's targets share one
    /// constructor symbol.
    SubstituteAnotherExactIdentity,
    /// Record some other instruction than the raw-worker call that was written.
    RedirectRecordedInstruction,
    /// Claim from a function that is not the identity's emission owner.
    WrongClaimingOwner,
    /// Attempt the composed discharge on an ordinary binding.
    DischargeFromOrdinaryBinding,
    /// **Verification 4b's discriminator.** Move the emitter's reported operand
    /// run away from the run its `D8b`/`D8d` target declares, and move nothing
    /// else.
    ///
    /// ⛔ Applied to `StaticWorkerEmission.supplied_operands` AFTER the call has
    /// assembled and emitted its operand vector, so the vector that was written
    /// is the real one and only the evidence about it disagrees. Identity,
    /// paired target, owner, recorded instruction, decoded callee, downstream
    /// result and source control all stay correct, which is what makes 4b the
    /// FIRST refusal rather than a consequence of an earlier one.
    ///
    /// ⛔ A whole-target substitution is NOT a control for 4b: verification 1 or
    /// 4a would refuse first and mask it. That is why the perturbation is of the
    /// evidence and not of the target.
    ///
    /// ⚠ **The delta is arbitrary, and that is a property of this witness, not
    /// a choice.** Both of its workers declare arity 1 with no captures, so no
    /// adjacent real quantity differs from the true run to perturb toward. What
    /// carries the control is the isolation -- one field moves and every other
    /// verifier input stays exact -- not the value.
    SupplyOperandCountDisagreesWithTarget,
    /// Record a result value defined BEFORE the call, so the value the
    /// continuation is said to have received cannot have come from it.
    ///
    /// ⛔ A real value of the finished function, taken from the first
    /// instruction that defines one -- not a fabricated `Value`, which would be
    /// caught by a bounds check rather than by the downstream relation.
    RecordResultDefinedBeforeTheCall,
}

#[cfg(test)]
thread_local! {
    static D8J_MUTATION: std::cell::Cell<D8jMutation> =
        const { std::cell::Cell::new(D8jMutation::Exact) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8j_mutation(mutation: D8jMutation) {
    D8J_MUTATION.with(|cell| cell.set(mutation));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8j_mutation() -> D8jMutation {
    D8J_MUTATION.with(std::cell::Cell::get)
}

/// **`D8j`** — the verified composed-discharge relation of the function most
/// recently finalized, for the rows that assert it was populated.
#[cfg(test)]
thread_local! {
    static D8J_DISCHARGED: std::cell::RefCell<Vec<ContinuationCallIdentity>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// **`D8l2` — the discharge facet each source-machine consumption carried.**
///
/// ⛔ Recorded at the CONSUMPTION seat, not at construction. `D8I_DISCHARGES`
/// already says what each binding was built with; this says what was actually
/// consumed, and the two populations differ -- a binding can be built and never
/// consumed, and a program can reach the seat with bindings from several
/// sources. `true` is a composed authority.
#[cfg(test)]
thread_local! {
    static D8L2_CONSUMED_FACETS: std::cell::RefCell<Vec<bool>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8l2_consumed_facet(composed: bool) {
    D8L2_CONSUMED_FACETS.with(|log| log.borrow_mut().push(composed));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8l2_consumed_facets() -> Vec<bool> {
    D8L2_CONSUMED_FACETS.with(|log| log.borrow().clone())
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8n` — what the two checked seams actually
/// saw, per compilation.**
///
/// ⛔ Written AT the seams from state production already holds there, never
/// rebuilt from a fixture or an expected plan. The point of the split-body
/// witness is that one source body is lowered under two different defining
/// Functions; an observation that reconstructed the identity would be a second
/// authority for the very fact under test. Each record carries the **defining
/// `FuncId`** -- never an emission owner.
///
/// ⚠ The identity is `defining_function_id`, set by `open_aggregate_events` at
/// the START of every body -- the one per-`Function` fact in scope at both
/// seams.
///
/// **Pre-`D8o` history, retained because it is why this field was chosen:**
/// `defining_emission_owner` was then set only by the ordinary unit-body and
/// generated-context passes, so it was **stale** inside a specialization body,
/// and both consumptions reported the same owner while genuinely occurring in
/// two different Functions. ⭐ `D8o` repaired that: all three source-bearing
/// body kinds now bind both ambient fields for their own lifetime and restore
/// on exit, so the current invariant is that the owner is the planner's for the
/// body being emitted. `defining_function_id` remains the right identity here
/// regardless -- it answers "which module definition", which is the question
/// these two seams ask.
#[cfg(test)]
thread_local! {
    static D8N_FRAME_CONSUMPTIONS: std::cell::RefCell<Vec<(Option<FuncId>, u64, u64)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static D8N_SLOT_RECONCILIATIONS: std::cell::RefCell<Vec<(Option<FuncId>, u64)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// `D8m` — the pair the slot seam actually reconciled: the checked frame id
    /// the bridge transported, and the plan slot template it was held to. Kept
    /// separate from `D8N_SLOT_RECONCILIATIONS` so `D8n`'s accepted evidence
    /// keeps the exact tuple it was reviewed with.
    ///
    /// This is the relation a witness with ONE checked occurrence cannot
    /// discriminate: there, frame and slot are both singletons and any pairing
    /// of them is the same pairing. With two distinct occurrences the two
    /// subjects hold distinct values, so exchanging them is visible here and
    /// invisible to a bag of frames beside a bag of slots.
    static D8M_SLOT_FRAME_PAIRS: std::cell::RefCell<Vec<(Option<FuncId>, u64, u64)>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// `D8m` — which arm of the closed bridge descriptor each composed site
    /// actually took. The three arms are disjoint by source constructor, so this
    /// is how "the ordinary and unwrapped populations are unchanged" becomes a
    /// measurement rather than a reading of the match.
    static D8M_BRIDGE_ARMS: std::cell::RefCell<Vec<(Option<FuncId>, D8mBridgeArm)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// The arm of the closed immediate-binder bridge descriptor a composed site took.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum D8mBridgeArm {
    /// A bare `ComputationalMatch` -- the unwrapped bridge, all-None.
    Computational,
    /// A `CheckedSubcontinuationFrame` wrapping a `ComputationalMatch` -- the
    /// form `D8m` added.
    CheckedComputational,
    /// A bare `Match` -- the ordinary bridge.
    Ordinary,
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8n_frame_consumption(
    defining_function: Option<FuncId>,
    invocation_id: u64,
    frame_id: u64,
) {
    D8N_FRAME_CONSUMPTIONS
        .with(|log| log.borrow_mut().push((defining_function, invocation_id, frame_id)));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8n_slot_reconciliation(
    defining_function: Option<FuncId>,
    slot_template_id: u64,
) {
    D8N_SLOT_RECONCILIATIONS
        .with(|log| log.borrow_mut().push((defining_function, slot_template_id)));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8m_slot_frame_pair(
    defining_function: Option<FuncId>,
    checked_frame_id: u64,
    slot_template_id: u64,
) {
    D8M_SLOT_FRAME_PAIRS.with(|log| {
        log.borrow_mut()
            .push((defining_function, checked_frame_id, slot_template_id))
    });
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8m_slot_frame_pairs() -> Vec<(Option<FuncId>, u64, u64)> {
    D8M_SLOT_FRAME_PAIRS.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8m_bridge_arm(
    defining_function: Option<FuncId>,
    arm: D8mBridgeArm,
) {
    D8M_BRIDGE_ARMS.with(|log| log.borrow_mut().push((defining_function, arm)));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8m_bridge_arms() -> Vec<(Option<FuncId>, D8mBridgeArm)> {
    D8M_BRIDGE_ARMS.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8n_frame_consumptions() -> Vec<(Option<FuncId>, u64, u64)> {
    D8N_FRAME_CONSUMPTIONS.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8n_slot_reconciliations() -> Vec<(Option<FuncId>, u64)> {
    D8N_SLOT_RECONCILIATIONS.with(|log| log.borrow().clone())
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8o`** — what each emitted body was BOUND
/// with, and what it INHERITED.
///
/// ⛔ Written at the binding from the facts the caller supplied, which are the
/// planner's own for that body kind. The inherited pair is the half that proves
/// the release: with it in place every body inherits `None`.
#[cfg(test)]
#[allow(clippy::type_complexity)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct D8oBodyAuthority {
    pub(in crate::cranelift_backend) function: Option<FuncId>,
    pub(in crate::cranelift_backend) owner: ContinuationEmissionOwner,
    pub(in crate::cranelift_backend) unit: PredeclaredFunctionId,
    pub(in crate::cranelift_backend) inherited_owner: Option<ContinuationEmissionOwner>,
    pub(in crate::cranelift_backend) inherited_unit: Option<PredeclaredFunctionId>,
}

#[cfg(test)]
thread_local! {
    static D8O_BODY_AUTHORITIES: std::cell::RefCell<Vec<D8oBodyAuthority>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static D8O_INHERIT_RESIDUE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8o_body_authority(
    function: Option<FuncId>,
    owner: ContinuationEmissionOwner,
    unit: PredeclaredFunctionId,
    inherited_owner: Option<ContinuationEmissionOwner>,
    inherited_unit: Option<PredeclaredFunctionId>,
) {
    D8O_BODY_AUTHORITIES.with(|log| {
        log.borrow_mut().push(D8oBodyAuthority {
            function,
            owner,
            unit,
            inherited_owner,
            inherited_unit,
        })
    });
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8o_body_authorities() -> Vec<D8oBodyAuthority> {
    D8O_BODY_AUTHORITIES.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d8o_body_authorities() {
    D8O_BODY_AUTHORITIES.with(|log| log.borrow_mut().clear());
    D8O_COMPOSED_CLAIM_BODIES.with(|log| log.borrow_mut().clear());
    D8O_BODY_KEYS.with(|log| log.borrow_mut().clear());
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8o` — the exact body key.**
///
/// ⛔⛔ **Closed, and independently meaningful.** Each variant names the body by
/// the planner descriptor identity that pass was handed -- not by the ambient
/// owner, not by the `FuncId` alone, not by a raw origin, and not by a selected
/// composed identity. Supplied at each of the three body sites, which is the
/// only place that knows which kind it is without inferring it.
///
/// ⭐ **A generated context carries a `Specialization` OWNER and is not a
/// specialization BODY.** That distinction is the whole reason this key exists
/// separately from the owner: an owner-variant filter counts a context body as
/// a specialization body, which is exactly the mistake the previous evidence
/// made.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum D8oBodyKey {
    OrdinaryUnit(PredeclaredFunctionId),
    ContinuationSpecialization(ContinuationSpecializationId),
    GeneratedContext(ContinuationContextId),
}

#[cfg(test)]
thread_local! {
    static D8O_BODY_KEYS: std::cell::RefCell<Vec<(Option<FuncId>, D8oBodyKey)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8o_body_key(
    function: Option<FuncId>,
    key: D8oBodyKey,
) {
    D8O_BODY_KEYS.with(|log| log.borrow_mut().push((function, key)));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8o_body_keys() -> Vec<(Option<FuncId>, D8oBodyKey)> {
    D8O_BODY_KEYS.with(|log| log.borrow().clone())
}

/// **`D8o`** — the emitter body of every composed claim actually reached.
#[cfg(test)]
thread_local! {
    static D8O_COMPOSED_CLAIM_BODIES: std::cell::RefCell<
        Vec<(Option<FuncId>, Option<ContinuationEmissionOwner>)>,
    > = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8o_composed_claim_body(
    function: Option<FuncId>,
    body_owner: Option<ContinuationEmissionOwner>,
) {
    D8O_COMPOSED_CLAIM_BODIES.with(|log| log.borrow_mut().push((function, body_owner)));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8o_composed_claim_bodies(
) -> Vec<(Option<FuncId>, Option<ContinuationEmissionOwner>)> {
    D8O_COMPOSED_CLAIM_BODIES.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8o_inherit_residue(armed: bool) {
    D8O_INHERIT_RESIDUE.with(|cell| cell.set(armed));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8o_inherit_residue() -> bool {
    D8O_INHERIT_RESIDUE.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d8n_observations() {
    D8N_FRAME_CONSUMPTIONS.with(|log| log.borrow_mut().clear());
    D8N_SLOT_RECONCILIATIONS.with(|log| log.borrow_mut().clear());
    D8M_SLOT_FRAME_PAIRS.with(|log| log.borrow_mut().clear());
    D8M_BRIDGE_ARMS.with(|log| log.borrow_mut().clear());
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8j_discharged() -> Vec<ContinuationCallIdentity> {
    D8J_DISCHARGED.with(|log| log.borrow().clone())
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn record_d8j_discharged(
    identities: impl IntoIterator<Item = ContinuationCallIdentity>,
) {
    D8J_DISCHARGED.with(|log| log.borrow_mut().extend(identities));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_d8j_discharged() {
    D8J_DISCHARGED.with(|log| log.borrow_mut().clear());
}

/// **`D8i` — hand an ordinary binding site a REAL composed authority whose
/// emission owner is not this unit's.**
///
/// ⛔ The authority is taken from the plan's own target population, searched
/// for an emission owner that differs from the defining one. It is not
/// fabricated, and it could not be: `ContinuationCallIdentity` has no
/// constructor outside planning, which is exactly why this switch has to find a
/// real one rather than build a wrong one.
///
/// ⭐ It perturbs the constructor's INPUT — the facet a call site supplies —
/// and leaves the guard untouched, so the refusal is attributable to the guard
/// rather than to the mutation being the guard.
#[cfg(test)]
thread_local! {
    static D8I_FOREIGN_AUTHORITY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn set_d8i_foreign_authority(armed: bool) {
    D8I_FOREIGN_AUTHORITY.with(|cell| cell.set(armed));
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8i_foreign_authority() -> bool {
    D8I_FOREIGN_AUTHORITY.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn d8d_bindings() -> usize {
    D8D_STATIC_WORKER_BINDINGS.with(std::cell::Cell::get)
}

fn extend_specialized(
    env: &mut Vec<LoweringEnvironmentBinding>,
    bindings: impl IntoIterator<Item = Lowered>,
) {
    env.extend(
        bindings
            .into_iter()
            .map(|lowered| LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(lowered))),
    );
}

/// Append a retained callable's **phase-bearing captures** after operands
/// already in an environment.
///
/// ⭐ **The counterpart to [`extend_specialized`] for the `D7` capture edge, and
/// the distinction is the point.** `extend_specialized` takes templates and
/// *asserts* a phase by wrapping each one `Specialized`. A capture edge already
/// carries its own phase, so asserting one here would be exactly the narrowing
/// that made a mixed environment unrepresentable: a `Carried` capture would
/// either be refused or, worse, re-labelled as a template it has no bytes for.
///
/// ⛔ There is no classification to perform. An environment binding holds a
/// [`LoweringOperand`], so this is a total, phase-preserving move — **not** a
/// wildcard that silently absorbs the carried case.
fn extend_captures(
    env: &mut Vec<LoweringEnvironmentBinding>,
    captures: impl IntoIterator<Item = LoweringOperand>,
) {
    env.extend(captures.into_iter().map(LoweringEnvironmentBinding::Value));
}

/// ⭐ The inverse direction, which needs **no** boundary at all: a freshly
/// constructed specialized value entering the spine.
///
/// ⛔ There is deliberately no `From<CarriedBoundaryWord>` counterpart taking a
/// short cut around [`Lowering::transfer_into_carrier`] — the producer is the
/// one way in, and it screens admissibility first.
impl From<Lowered> for LoweringOperand {
    fn from(lowered: Lowered) -> Self {
        LoweringOperand::Specialized(lowered)
    }
}

/// ⭐ **The ONE-WAY PRODUCER** — `Lowered -> CarriedBoundaryWord`
/// (`RT-FNSPLIT-C1` `D3`; the seam is ruled in frame `§2g`, its authority for
/// `(tag, class)` in `§2h` ¶4).
///
/// ⛔ **There is deliberately no inverse in this block, and none may be added.**
/// A `Carried -> Lowered` conversion would let a consumer recover a compile-time
/// template from a runtime value, which is the wall this whole node exists to
/// remove, wearing a different name.
impl<'a> Lowering<'a> {
    /// Transfer a compile-time [`Lowered`] into the operational carrier.
    ///
    /// ⚠⚠ **The admissibility walk runs HERE and exactly once, before the first
    /// allocation.** [`Lowered::boundary_transfer_admissibility`]'s own contract
    /// says the ordering is load-bearing: a walk performed after the first child
    /// is published rejects a transfer it has already half-emitted, which is a
    /// **partial publication**, not a rejection. That is why the recursion lives
    /// in a separate private [`Self::emit_carrier_transfer`] — the entry point
    /// screens the whole graph, and the recursion never re-screens a subgraph
    /// whose parent is already allocated.
    fn transfer_into_carrier(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        value: &Lowered,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        value.boundary_transfer_admissibility()?;
        self.source_aggregate_preflight(value)?;
        self.emit_carrier_transfer(builder, origin, value)
    }

    /// **`RT-DECL-CLOSURE-PORT` `D7` — reconcile every aggregate in a template
    /// against its OWN producer's planned ownership record, before anything is
    /// allocated.**
    ///
    /// ⭐ **It takes no origin, and that absence is the mechanism.** Every other
    /// reconciliation on this path is handed a coordinate and is therefore only
    /// as right as the coordinate it was handed — which is precisely the defect
    /// class this subclosure exists to close. Here there is no coordinate to
    /// pass, so there is no wrong one to pass: each node is checked against the
    /// record its own producer occurrence names, and a template can only be
    /// admitted by agreeing with the plan about itself.
    ///
    /// ⛔ **A missing producer is a REFUSAL, never a fallback.** An aggregate
    /// with no interned occurrence has no lifetime meet, so "resolve it at
    /// wherever it is being transferred" would reinstate exactly the
    /// use-coordinate authority the `occurrence` fields were added to retire.
    /// The fallback in [`Self::aggregate_carrier_authority`] survives only for
    /// values this preflight never sees.
    ///
    /// ⚠ **Whole-graph, and it runs BEFORE `emit_carrier_transfer`.** A nested
    /// child is allocated during its parent's transfer, so a check that fired
    /// only at each node's own allocation would already have allocated the
    /// parent by the time a child was refused. Walking the spine up front is
    /// what makes "refuses before any allocation" true of the whole tree rather
    /// than of its root.
    fn source_aggregate_preflight(&self, value: &Lowered) -> Result<(), CraneliftBackendError> {
        match value {
            // ── the two source aggregates: reconciled AT THIS NODE ────────
            Lowered::Constructor { args, .. } => {
                let children: Vec<&Lowered> = args.iter().collect();
                self.reconcile_source_aggregate(
                    value,
                    PlannedAggregateShape::Constructor,
                    &children,
                    None,
                )?;
                for arg in args {
                    self.source_aggregate_preflight(arg)?;
                }
                Ok(())
            }
            Lowered::Record { fields, .. } => {
                let children: Vec<&Lowered> = fields.iter().map(|field| &field.value).collect();
                self.reconcile_source_aggregate(
                    value,
                    PlannedAggregateShape::Record,
                    &children,
                    Some(fields),
                )?;
                for field in fields {
                    self.source_aggregate_preflight(&field.value)?;
                }
                Ok(())
            }

            // ── recursive carriers that are not themselves aggregates ─────
            //
            // ⛔ These have no ownership record of their own and they are NOT
            // leaves. A walk that stopped here would leave every aggregate
            // below them unreconciled while reporting the tree admitted -- and
            // both of these positions are reached: a host result's two arms are
            // separate trees, and a dynamic alternative's fields are two levels
            // down inside a `Vec` of alternative structs.
            Lowered::HostResult { error, ok, .. } => {
                self.source_aggregate_preflight(error)?;
                self.source_aggregate_preflight(ok)
            }
            Lowered::DynamicConstructor(dynamic) => {
                for alternative in &dynamic.alternatives {
                    for field in &alternative.fields {
                        self.source_aggregate_preflight(field)?;
                    }
                }
                Ok(())
            }

            // ── values that cannot cross at all ───────────────────────────
            //
            // ⛔ Admitted HERE means "this walk has nothing to reconcile", not
            // "this value may cross". Whether a closure has a boundary
            // representation is `boundary_transfer_admissibility`'s question,
            // it is decided before this walk runs on the same value, and it
            // refuses all three. Re-deciding it here would put a second, weaker
            // authority on a question that already has one -- and would report
            // a nested closure as an ownership failure.
            Lowered::Closure { .. }
            | Lowered::DeclarationClosure { .. }
            | Lowered::ComputationalRecursorClosure { .. } => Ok(()),

            // ── true leaves: no `Lowered` child position exists ───────────
            //
            // ⛔ No `_` arm, by construction. A new variant with a child
            // position is a compile error here rather than a subtree that
            // silently stops being reconciled -- which is exactly how
            // `HostResult` and `DynamicConstructor` were missed.
            Lowered::Int { .. }
            | Lowered::Bool { .. }
            | Lowered::ProcessExitStatus { .. }
            | Lowered::CapabilityToken { .. }
            | Lowered::ResourceToken { .. }
            | Lowered::BoundedNat(_)
            | Lowered::StructuralNat(_)
            | Lowered::ResponseBytes { .. }
            | Lowered::Bytes(_)
            | Lowered::BorrowedNativeValue { .. }
            | Lowered::BorrowedOption { .. }
            | Lowered::String(_)
            | Lowered::RecursiveBackedge
            | Lowered::Trap(_) => Ok(()),
        }
    }

    /// Reconcile ONE source aggregate node against the ownership record its own
    /// producer occurrence names.
    ///
    /// ⛔ Takes no origin, for the reason [`Self::source_aggregate_preflight`]
    /// states: there is no coordinate to pass, so there is no wrong one.
    fn reconcile_source_aggregate(
        &self,
        value: &Lowered,
        shape: PlannedAggregateShape,
        children: &[&Lowered],
        record_fields: Option<&[LoweredRecordField]>,
    ) -> Result<(), CraneliftBackendError> {
        let Some(occurrence) = value.source_aggregate_producer() else {
            return Err(unsupported(
                lowered_value_kind(value),
                "a source aggregate reached the carrier with no planner-issued producer \
                 occurrence, so it would name no ownership record and could only be given \
                 the authority of wherever it happened to be transferred",
            ));
        };
        let planned = self
            .static_transition_plan
            .aggregate_record_view(occurrence)?;
        if planned.shape() != shape {
            return Err(unsupported(
                lowered_value_kind(value),
                format!(
                    "the template is a {shape:?} but its own producer occurrence names a \
                     {:?} ownership record",
                    planned.shape()
                ),
            ));
        }
        // ⭐ **The CLASS is a second, independent reading of the same fact.**
        // The variant match above is what the template *is*; this is what the
        // sole disposition authority says it *represents*. They are derived by
        // different code from different fields, so a template whose variant and
        // disposition ever disagree is caught here rather than allocating under
        // one and being emitted under the other.
        let (_, class) = Self::carrier_handle_disposition(value)?;
        let planned_class = CarrierAllocationRequest::aggregate_class(planned.shape());
        if class != planned_class {
            return Err(unsupported(
                lowered_value_kind(value),
                format!(
                    "the template's boundary class is {class:?} but its own producer \
                     occurrence names a {planned_class:?} ownership record"
                ),
            ));
        }
        if planned.children().len() != children.len() {
            return Err(unsupported(
                lowered_value_kind(value),
                format!(
                    "the template has {} children but its own producer occurrence names an \
                     ownership record planned with {}",
                    children.len(),
                    planned.children().len()
                ),
            ));
        }
        // ⭐ **The constructor SCHEMA, against the producer's own origin.**
        // A source constructor's `synthesized_identity` was resolved at the
        // producer and travels with the template for the same reason the
        // occurrence does; here the two carried facts are made to agree with
        // each other through the plan. A template that acquired one producer's
        // occurrence and another's symbol -- the exact shape of a grafted or
        // substituted certificate -- cannot satisfy both.
        //
        // ⚠ Gated to a SOURCE producer: a compiler-synthesized constructor's
        // identity comes from the semantic plane's closed role capability and
        // has no source origin to resolve against.
        if let Some(producer_origin) = planned.producer_origin() {
            if let Lowered::Constructor {
                synthesized_identity: Some(carried),
                ..
            } = value
            {
                let planned_identity = self
                    .static_transition_plan
                    .constructor_symbol_identity(producer_origin)?;
                if *carried != planned_identity {
                    return Err(unsupported(
                        lowered_value_kind(value),
                        "the template's carried constructor identity is not the one the \
                         planner resolved at its own producer occurrence's origin",
                    ));
                }
            }
        }
        for (position, (child, planned_child)) in
            children.iter().zip(planned.children()).enumerate()
        {
            // ⭐ **The RECORD half of the schema, compared EXACTLY and by
            // TYPE.** The template carries the identity its producer was issued
            // and the plan states the identity it planned at this position;
            // both are `FieldIdentity`, so the comparison is the identity
            // itself rather than a spelling. ⛔ There is no `&str ->
            // FieldIdentity` direction and none may be added: comparing the
            // template's field STRING against the plan would be the second
            // derivation `D2` forbids, and it is what left field naming
            // unreconciled while order and arity were covered.
            if let Some(fields) = record_fields {
                let held = fields[position].identity;
                match (held, planned_child.field_identity) {
                    // ⛔ Not `held != planned`. Two absences comparing equal is
                    // the shape that admits a record with no schema at all
                    // against a record the planner planned no schema for.
                    (Some(held), Some(planned)) if held == planned => {}
                    (held, planned) => {
                        return Err(unsupported(
                            lowered_value_kind(value),
                            format!(
                                "record field {position} carries identity {held:?} but its own \
                                 producer occurrence names a record planned with {planned:?} \
                                 at that position"
                            ),
                        ));
                    }
                }
            }
            // ⭐ **The child's own possible-owner set, against the set the meet
            // was taken over.** The planner's set is what the parent's lane was
            // DERIVED from; this is what the sole disposition authority and the
            // child's own static encoding say the emitter will actually build.
            // A child that can be owned by something the meet never considered
            // makes the parent's lane a conclusion from the wrong premises --
            // and when that something is the invocation arena, a persistent
            // parent ends up naming storage that dies first.
            // ⭐ **The planner's TWO fields about this position must agree,
            // before either is used.** `owners` is the set the meet was taken
            // over and `lifetime` is the lifetime it was taken under, and the
            // law relating them is one-directional: the invocation arena is a
            // possible owner ONLY IF the position is activation-owned.
            //
            // ⛔ **Not an equivalence, and the asymmetry is a measured fact
            // about the planner rather than a hedge.** `owners` is the
            // lifetime's affinity INTERSECTED with the child's representation,
            // so a child the emitter materializes as a native scalar pair is
            // recorded `ActivationOwned` over `[NoReferent]` -- it may be
            // short-lived and still have no boundary node for anything to own.
            // Stating this as `==` refuses that lawful record, measured on the
            // very fixture the owner control is built from.
            //
            // ⚠ Stated as a law rather than assumed, because everything below
            // reads whichever field answers its question: a record whose owners
            // admitted the arena under a persistent lifetime would let a
            // containment pass under one field while the lane was concluded
            // from the other.
            if planned_child
                .owners
                .contains(&BoundaryReferentOwner::InvocationArena)
                && planned_child.lifetime != PlannedReferentLifetime::ActivationOwned
            {
                return Err(unsupported(
                    lowered_value_kind(child),
                    format!(
                        "the ownership record's own fields disagree at child {}: it plans a \
                         {:?} referent lifetime and possible owners {:?}",
                        planned_child.position, planned_child.lifetime, planned_child.owners,
                    ),
                ));
            }
            // ⛔ **Containment, and the DIRECTION is the whole content.** A held
            // child may be LONGER-lived than the position planned for: a
            // persistent child sitting where the meet allowed an
            // activation-owned one dangles nothing. Requiring the two sets to be
            // EQUAL reds every spillable immediate, whose closed set is
            // `{NoReferent, PersistentStore}` at a position the planner
            // legitimately plans `ActivationOwned` -- measured on two fixtures
            // before the direction was fixed. It may never be SHORTER: that is
            // the parent naming storage that dies first, which is the edge this
            // subclosure exists to close.
            //
            // ⚠ The held set is read from the disposition and, for an aggregate,
            // from its own ruled lane -- never from the value's shape.
            // `Constructor` and `Record` are persistable SHAPES, which is
            // precisely why the shape cannot answer this.
            let held_owners = self.child_possible_referent_owners(child)?;
            if let Some(escaped) = held_owners
                .iter()
                .find(|owner| !planned_child.owners.contains(owner))
            {
                let held_lifetime = Self::possible_owners_lifetime(&held_owners);
                return Err(unsupported(
                    lowered_value_kind(child),
                    format!(
                        "child {} is held with a {held_lifetime:?} referent lifetime and can \
                         be owned by {escaped:?}, which its own producer occurrence's \
                         ownership record did not plan for that position (planned {:?} over \
                         {:?})",
                        planned_child.position, planned_child.lifetime, planned_child.owners,
                    ),
                ));
            }
            // ⚠ Gated to a SOURCE producer. A compiler-synthesized aggregate's
            // children have no occurrence in the program, and their agreement
            // with the plan is already established -- by path, role and
            // disposition -- in [`Self::reconcile_declared_children`].
            // Re-deriving it here from source origins the planner deliberately
            // recorded as absent would be a second, weaker authority for a
            // question that already has one.
            if planned.producer_origin().is_none() {
                continue;
            }
            let Some(child_shape) = Self::lowered_aggregate_shape(child) else {
                continue;
            };
            let Some(child_origin) = planned_child.origin else {
                return Err(unsupported(
                    lowered_value_kind(child),
                    "a source aggregate's child is an aggregate, but the planner recorded \
                     no source occurrence for it at that position",
                ));
            };
            let expected = self
                .static_transition_plan
                .source_aggregate_occurrence(child_origin, child_shape)?;
            // ⭐ The child must carry the occurrence the planner planned AT
            // THAT POSITION -- not merely some record of the same shape, and
            // not the same producer's record reached from elsewhere in the
            // tree. Swap two children, graft a sibling's subtree in, or hand a
            // forwarded aggregate a neighbour's certificate, and the carried
            // occurrence stops matching the position it sits at.
            if child.source_aggregate_producer() != Some(expected) {
                return Err(unsupported(
                    lowered_value_kind(child),
                    format!(
                        "child {} carries producer occurrence {:?} but the planner planned \
                         {expected:?} at that position",
                        planned_child.position,
                        child.source_aggregate_producer(),
                    ),
                ));
            }
        }
        Ok(())
    }

    /// The **closed** set of referent owners one held child can have.
    ///
    /// ⭐ Derived from the two authorities that already own the question and
    /// from nothing else: [`Lowered::boundary_disposition`] for what the value
    /// represents, and -- for an aggregate -- its OWN planned allocation lane
    /// for what it will be allocated in. ⛔ Never re-derived from a runtime tag
    /// and never guessed from the value's shape: `Constructor` and `Record` are
    /// persistable shapes, which is precisely why the shape cannot answer this.
    fn child_possible_referent_owners(
        &self,
        child: &Lowered,
    ) -> Result<Vec<BoundaryReferentOwner>, CraneliftBackendError> {
        match child.boundary_disposition() {
            // An immediate with no spill class has no boundary node at any
            // magnitude, so there is nothing for an arena or a store to own.
            BoundaryDisposition::RepresentedImmediate { spill: None, .. } => {
                Ok(vec![BoundaryReferentOwner::NoReferent])
            }
            // ⚠ A spillable immediate has TWO representations and the choice is
            // a runtime magnitude, so both owners are possible and the set says
            // so. Collapsing it to either one would be a determination this
            // static walk is not entitled to make.
            BoundaryDisposition::RepresentedImmediate { spill: Some(_), .. } => Ok(vec![
                BoundaryReferentOwner::NoReferent,
                BoundaryReferentOwner::PersistentStore,
            ]),
            BoundaryDisposition::RepresentedHandle { tag, .. } => {
                let Some(_) = Self::lowered_aggregate_shape(child) else {
                    // A non-aggregate handle's owner IS its tag's, exactly as
                    // the closed discriminator product reads it.
                    return Ok(vec![tag.referent_owner()]);
                };
                // ⭐ An aggregate's owner comes from its OWN ownership record's
                // ruled lane, never from the tag its shape reached for. This is
                // the same read the allocation itself makes, so a child whose
                // lane disagrees with its position in the parent is caught here
                // rather than after both are allocated.
                let Some(occurrence) = child.source_aggregate_producer() else {
                    return Err(unsupported(
                        lowered_value_kind(child),
                        "a source aggregate child reached the carrier with no planner-issued \
                         producer occurrence, so its allocation lane -- and therefore its \
                         referent owner -- has no authority",
                    ));
                };
                let allocation = self
                    .static_transition_plan
                    .aggregate_record_view(occurrence)?
                    .allocation();
                Ok(vec![match allocation {
                    PlannedAggregateAllocation::PersistentGround => {
                        BoundaryReferentOwner::PersistentStore
                    }
                    PlannedAggregateAllocation::InvocationAggregate => {
                        BoundaryReferentOwner::InvocationArena
                    }
                }])
            }
            BoundaryDisposition::ProtocolOnly { why }
            | BoundaryDisposition::FailClosedForbidden { why } => {
                Err(unsupported(lowered_value_kind(child), why))
            }
        }
    }

    /// The referent lifetime a closed possible-owner set encodes.
    ///
    /// ⛔ Membership, not a determination: a child is activation-owned exactly
    /// when the invocation arena is a possible owner of it, which is the same
    /// rule the planner's own meet is taken under.
    fn possible_owners_lifetime(owners: &[BoundaryReferentOwner]) -> PlannedReferentLifetime {
        if owners.contains(&BoundaryReferentOwner::InvocationArena) {
            PlannedReferentLifetime::ActivationOwned
        } else {
            PlannedReferentLifetime::Persistent
        }
    }

    /// The planned aggregate shape a template is, if it is an aggregate at all.
    fn lowered_aggregate_shape(value: &Lowered) -> Option<PlannedAggregateShape> {
        match value {
            Lowered::Constructor { .. } => Some(PlannedAggregateShape::Constructor),
            Lowered::Record { .. } => Some(PlannedAggregateShape::Record),
            _ => None,
        }
    }

    /// Transfer the terminal value returned by one declared generated unit.
    ///
    /// Process exit constructors are the one result-edge representation that
    /// differs from their nested carrier form: the root consumes a closed
    /// `ImmediateExitStatus`, not a constructor node. Keeping the conversion at
    /// this result surface prevents an ordinary nested exit-shaped constructor
    /// from being mistaken for the process answer.
    pub(super) fn transfer_unit_result_into_carrier(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        value: &Lowered,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        #[cfg(test)]
        d5a_trace(format!(
            "  UNIT-RESULT transfer origin={origin:?} value={}",
            lowered_value_kind(value)
        ));
        let process_exit = self.process_object
            && matches!(
                value,
                Lowered::Constructor { constructor, .. }
                    if constructor == &self.process_symbols.exit_success
                        || constructor == &self.process_symbols.exit_failure
            );
        if process_exit {
            let status = self.emit_process_exit_status(builder, value.clone());
            self.emit_carrier_immediate(builder, BoundaryTag::ImmediateExitStatus, status)
        } else {
            self.transfer_into_carrier(builder, origin, value)
        }
    }

    /// Select the exact source occurrences evaluated in result position for
    /// the generated unit currently being defined.
    pub(super) fn select_terminal_result_origins(
        &mut self,
        origin: StaticOriginId,
        _expr: &RuntimeExpr,
    ) -> Result<(), CraneliftBackendError> {
        self.function_local.terminal_result_origins = self
            .static_transition_plan
            .source_result_origins_in_owner_subtree(origin)?;
        Ok(())
    }

    /// Take the pre-emission result contract for this exact source join.
    ///
    /// Consumption is recorded before a merge block can be created. A second
    /// call to this method in one generated function is therefore a planner /
    /// lowering disagreement. Legitimate traversal re-entry goes through
    /// [`Self::enter_source_occurrence_plan`] and only reborrows the token.
    fn consume_join_plan(
        &mut self,
        origin: StaticOriginId,
    ) -> Result<JoinPlanToken, CraneliftBackendError> {
        let token = self.static_transition_plan.join_plan_token(origin)?;
        if token.origin != origin {
            return Err(backend_module(
                "source join consumed a result plan for a different origin".to_string(),
            ));
        }
        #[cfg(test)]
        match D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get) {
            JoinConsumptionMutation::Exact => {}
            JoinConsumptionMutation::SkipFirst
                if self.function_local.consumed_join_origins.is_empty() =>
            {
                return Ok(token);
            }
            JoinConsumptionMutation::DuplicateFirst
                if self.function_local.consumed_join_origins.is_empty() =>
            {
                self.function_local.consumed_join_origins.insert(origin);
            }
            JoinConsumptionMutation::SkipFirst
            | JoinConsumptionMutation::DuplicateFirst
            | JoinConsumptionMutation::IncludeStaticallyUnselected
            | JoinConsumptionMutation::OmitFirstStaticallyUnselectedMatchCase
            | JoinConsumptionMutation::OmitSourceMachineComputationalMatchSelection
            | JoinConsumptionMutation::MaterializeFirstUnselectedMatchJoin
            | JoinConsumptionMutation::AttachEntryToFirstMaterializedDead
            | JoinConsumptionMutation::DispositionDynamicHostResultMerge => {}
        }
        if !self.function_local.consumed_join_origins.insert(origin) {
            return Err(backend_module(
                "one source join consumed its static result plan more than once".to_string(),
            ));
        }
        Ok(token)
    }

    /// Disposition every planned join in a statically unselected source branch.
    ///
    /// The planner derives the subtree from its validated positional-child
    /// inventory and stops at declared-unit owner boundaries. Lowering supplies
    /// only the exact branch root it proved dead; it maintains no second source
    /// spelling inventory.
    fn disposition_statically_unselected_source_subtree(
        &mut self,
        root: StaticOriginId,
    ) -> Result<(), CraneliftBackendError> {
        #[cfg(test)]
        if D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get)
            == JoinConsumptionMutation::IncludeStaticallyUnselected
        {
            return Ok(());
        }
        let joins = self
            .static_transition_plan
            .source_join_origins_in_owner_subtree(root)?;
        for origin in joins {
            self.function_local
                .dispositioned_join_origins
                .insert(origin);
        }
        Ok(())
    }

    /// Record one case reached by static `Match` selection.
    ///
    /// `Match` lays its validated positional children out as the scrutinee
    /// followed by every case body. The complete root population comes from the
    /// planner's checked child inventory; lowering supplies only the reached
    /// case index. An empty selection records the default/no-match route.
    ///
    /// This deliberately defers disposition until generated-function closure.
    /// A recursive producer can revisit the same source occurrence and select a
    /// second case, so the emission-reachable population is the union of every
    /// observed selection, not the first constructor seen.
    fn disposition_statically_unselected_match_cases(
        &mut self,
        match_origin: StaticOriginId,
        selected_case: Option<usize>,
    ) -> Result<(), CraneliftBackendError> {
        let case_bodies = self
            .static_transition_plan
            .source_match_case_body_origins(match_origin)?;
        let case_count = case_bodies.len();
        if selected_case.is_some_and(|index| index >= case_count) {
            return Err(backend_module(
                "selected source Match case is outside the validated child population".to_string(),
            ));
        }
        #[cfg(test)]
        if matches!(
            D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get),
            JoinConsumptionMutation::MaterializeFirstUnselectedMatchJoin
                | JoinConsumptionMutation::AttachEntryToFirstMaterializedDead
        ) {
            let mut materialized = None;
            for (index, root) in case_bodies.iter().copied().enumerate() {
                if selected_case == Some(index) {
                    continue;
                }
                if let Some(origin) = self
                    .static_transition_plan
                    .source_join_origins_in_owner_subtree(root)?
                    .into_iter()
                    .next()
                {
                    materialized = Some(origin);
                    break;
                }
            }
            if let Some(origin) = materialized {
                if !self.function_local.consumed_join_origins.contains(&origin) {
                    self.consume_join_plan(origin)?;
                }
            }
        }
        let reached = self
            .function_local
            .emission_reachable_match_cases
            .entry(match_origin)
            .or_default();
        if let Some(index) = selected_case {
            reached.insert(index);
        }
        Ok(())
    }

    /// Record a specialized/default selection made after the source-machine
    /// continuation resumes a computational match.
    ///
    /// The separate seam is test-visible because an initial constructor
    /// selection and a recursive revisit use different lowering routes. A
    /// mutation must be able to remove only the revisit edge while preserving
    /// the initial population entry and the generated-function closure check.
    fn record_source_machine_computational_match_selection(
        &mut self,
        match_origin: StaticOriginId,
        selected_case: Option<usize>,
    ) -> Result<(), CraneliftBackendError> {
        #[cfg(test)]
        if D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get)
            == JoinConsumptionMutation::OmitSourceMachineComputationalMatchSelection
        {
            self.function_local
                .emission_reachable_match_cases
                .entry(match_origin)
                .or_default();
            return Ok(());
        }
        self.disposition_statically_unselected_match_cases(match_origin, selected_case)
    }

    /// Close every recorded static `Match` selection against its validated
    /// positional-child population.
    fn close_statically_unselected_match_cases(&mut self) -> Result<(), CraneliftBackendError> {
        let reached = self
            .function_local
            .emission_reachable_match_cases
            .iter()
            .map(|(origin, cases)| (*origin, cases.clone()))
            .collect::<Vec<_>>();
        // `RT-DECL-CLOSURE-PORT` `D5a` checkpoint 3 — the planner-issued
        // recursive predecessors. Read once, outside the loop, and from the
        // planner rather than from anything observed here.
        let recursive_predecessors = self
            .static_transition_plan
            .source_machine_recursive_predecessor_origins()?;
        for (match_origin, reached_cases) in reached {
            let case_bodies = self
                .static_transition_plan
                .source_match_case_body_origins(match_origin)?;
            // ⭐ **THE UNION `D5a` checkpoint 3 repairs.** Final reachability is
            // the initial selection PLUS every planner-issued source-machine
            // recursive predecessor -- not the initial selection alone.
            //
            // A predecessor's contribution here is the planner's **closed case
            // population**, because what re-enters the match is the *return* of
            // a generated call: a carried word with no compile-time constructor
            // template, so no case can be ruled out for it. Concretely, the
            // initial scrutinee selecting `Vis` used to disposition the `Ret`
            // arm's whole subtree, while the emitted causal call's return edge
            // makes `Ret` genuinely reachable -- lowering then both materialized
            // that arm's join and dispositioned it, and the finished CFG
            // correctly exposed the contradiction.
            //
            // ⚠ The ruling also says *"specialized re-entry keeps its exact
            // selected case"*. That describes the specialization **body**, which
            // lowers its own selected alternative directly and never re-enters
            // this match; it contributes to that generated function's own
            // population, not to this one. ⇒ There is deliberately **no**
            // exact-alternative narrowing at this seat: a narrowing that can
            // never fire is a branch that rots, and adding one here would read
            // as covering a case it could not reach.
            //
            // ⛔ The validator is untouched and this does not force any block
            // dead or delete origin 25. The repair is to stop asserting a
            // deadness that was never true.
            let final_reachable: BTreeSet<usize> =
                if recursive_predecessors.contains(&match_origin) {
                    (0..case_bodies.len()).collect()
                } else {
                    reached_cases
                };
            #[cfg(test)]
            let mut omitted_for_mutation = false;
            for (index, root) in case_bodies.into_iter().enumerate() {
                if final_reachable.contains(&index) {
                    continue;
                }
                #[cfg(test)]
                if !omitted_for_mutation
                    && D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get)
                        == JoinConsumptionMutation::OmitFirstStaticallyUnselectedMatchCase
                {
                    omitted_for_mutation = true;
                    continue;
                }
                self.disposition_statically_unselected_source_subtree(root)?;
            }
        }
        Ok(())
    }

    /// Enter one planned source occurrence on any lowering traversal.
    ///
    /// A recursive source-machine route may revisit the same occurrence while
    /// emitting one function. That is a reborrow of its already-consumed
    /// contract, not a second population member.
    fn enter_source_occurrence_plan(
        &mut self,
        origin: StaticOriginId,
    ) -> Result<(), CraneliftBackendError> {
        if self
            .static_transition_plan
            .join_plan_token_if_planned(origin)?
            .is_none()
        {
            return Ok(());
        }
        if self.function_local.consumed_join_origins.contains(&origin) {
            self.consumed_join_plan_token(origin)?;
        } else {
            self.consume_join_plan(origin)?;
        }
        Ok(())
    }

    /// Reborrow a contract after the source traversal has consumed it.
    ///
    /// Merge helpers may be reached long after a computational eliminator was
    /// installed. They do not constitute another source occurrence and must
    /// therefore neither mint nor consume a second contract.
    fn consumed_join_plan_token(
        &self,
        origin: StaticOriginId,
    ) -> Result<JoinPlanToken, CraneliftBackendError> {
        if !self.function_local.consumed_join_origins.contains(&origin) {
            return Err(backend_module(format!(
                "source join {origin:?} requested its static result plan before consumption",
            )));
        }
        self.static_transition_plan.join_plan_token(origin)
    }

    /// Close AC-14 at the generated-function boundary.
    ///
    /// Duplicate consumption already rejects at [`Self::consume_join_plan`].
    /// This equality supplies the missing other direction: every join in the
    /// planner's closed owner partition must either be reached exactly once by
    /// emission or be structurally dispositioned under a statically unselected
    /// branch, and no join owned by another function may appear here.
    fn validate_join_plan_consumption(
        &mut self,
        function: PredeclaredFunctionId,
    ) -> Result<(), CraneliftBackendError> {
        self.close_statically_unselected_match_cases()?;
        let required = self
            .static_transition_plan
            .required_join_origins(function)?;
        self.finalize_join_disposition(&required)
    }

    /// Close the one RecursiveDescent root over the recorded population it
    /// materializes or dispositions while deliberately inlining across planner
    /// owner boundaries.
    fn validate_recursive_descent_join_disposition(&mut self) -> Result<(), CraneliftBackendError> {
        self.close_statically_unselected_match_cases()?;
        let mut required = self.function_local.consumed_join_origins.clone();
        required.extend(
            self.function_local
                .dispositioned_join_origins
                .iter()
                .copied(),
        );
        self.finalize_join_disposition(&required)
    }

    fn finalize_join_disposition(
        &mut self,
        required: &BTreeSet<StaticOriginId>,
    ) -> Result<(), CraneliftBackendError> {
        #[cfg(test)]
        {
            let mutation = D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get);
            if matches!(
                mutation,
                JoinConsumptionMutation::OmitSourceMachineComputationalMatchSelection
            ) && self
                .function_local
                .consumed_join_origins
                .is_disjoint(&self.function_local.dispositioned_join_origins)
            {
                if let Some(origin) = self
                    .function_local
                    .consumed_join_origins
                    .iter()
                    .next()
                    .copied()
                {
                    self.function_local
                        .dispositioned_join_origins
                        .insert(origin);
                }
            }
        }

        let mut covered = self.function_local.consumed_join_origins.clone();
        covered.extend(
            self.function_local
                .dispositioned_join_origins
                .iter()
                .copied(),
        );
        if let Some(origin) = covered.difference(required).next() {
            return Err(backend_module(format!(
                "source join {origin:?} was classified outside its owning function",
            )));
        }
        if let Some(origin) = required.difference(&covered).next() {
            return Err(backend_module(format!(
                "function left planned source join {origin:?} neither emitted nor statically unselected",
            )));
        }
        if self.function_local.join_disposition_finalized {
            return Err(backend_module(
                "generated function finalized its source join disposition more than once"
                    .to_string(),
            ));
        }
        self.function_local.final_reachable_join_origins = required
            .difference(&self.function_local.dispositioned_join_origins)
            .copied()
            .collect();
        self.function_local.join_disposition_finalized = true;
        Ok(())
    }

    /// Validate the materialized-but-dead half of the final join disposition
    /// against the completed function CFG.
    ///
    /// A consumed token with no recorded merge block is metadata-only
    /// materialization and has no CFG repair obligation. Every recorded block
    /// for an origin later classified dead must be unreachable from entry,
    /// have no live predecessor, and contribute no block parameter to a
    /// reachable instruction. The ordinary Cranelift verifier subsequently
    /// closes the remaining SSA dominance and use-def obligations.
    fn validate_materialized_dead_join_cfg(
        &self,
        function: PredeclaredFunctionId,
        func: &Function,
    ) -> Result<(), CraneliftBackendError> {
        let required = self
            .static_transition_plan
            .required_join_origins(function)?;
        self.validate_materialized_dead_join_cfg_for(&required, func)
    }

    fn validate_recursive_descent_materialized_dead_join_cfg(
        &self,
        func: &Function,
    ) -> Result<(), CraneliftBackendError> {
        let mut required = self.function_local.final_reachable_join_origins.clone();
        required.extend(
            self.function_local
                .dispositioned_join_origins
                .iter()
                .copied(),
        );
        self.validate_materialized_dead_join_cfg_for(&required, func)
    }

    fn validate_materialized_dead_join_cfg_for(
        &self,
        required: &BTreeSet<StaticOriginId>,
        func: &Function,
    ) -> Result<(), CraneliftBackendError> {
        if !self.function_local.join_disposition_finalized {
            return Err(backend_module(
                "generated function checked materialized joins before final disposition"
                    .to_string(),
            ));
        }
        let mut final_covered = self.function_local.final_reachable_join_origins.clone();
        final_covered.extend(
            self.function_local
                .dispositioned_join_origins
                .iter()
                .copied(),
        );
        if &final_covered != required
            || !self
                .function_local
                .final_reachable_join_origins
                .is_disjoint(&self.function_local.dispositioned_join_origins)
        {
            return Err(backend_module(
                "generated function has an incomplete or overlapping final join disposition"
                    .to_string(),
            ));
        }
        let cfg = ControlFlowGraph::with_function(func);
        let entry = func
            .layout
            .entry_block()
            .ok_or_else(|| backend_module("generated function has no entry block".to_string()))?;
        let mut reachable = BTreeSet::from([entry]);
        let mut pending = vec![entry];
        while let Some(block) = pending.pop() {
            for successor in cfg.succ_iter(block) {
                if reachable.insert(successor) {
                    pending.push(successor);
                }
            }
        }

        let overlap = self
            .function_local
            .consumed_join_origins
            .intersection(&self.function_local.dispositioned_join_origins)
            .copied()
            .collect::<Vec<_>>();
        for origin in overlap {
            if !required.contains(&origin) {
                return Err(backend_module(format!(
                    "materialized-but-dead source join {origin:?} escaped its owning function",
                )));
            }
            let blocks = self
                .function_local
                .materialized_join_blocks
                .get(&origin)
                .into_iter()
                .flat_map(|blocks| blocks.iter().copied())
                .collect::<Vec<_>>();
            #[cfg(test)]
            let blocks = match D8_JOIN_CONSUMPTION_MUTATION.with(std::cell::Cell::get) {
                JoinConsumptionMutation::AttachEntryToFirstMaterializedDead
                | JoinConsumptionMutation::OmitSourceMachineComputationalMatchSelection => {
                    let mut blocks = blocks;
                    blocks.push(entry);
                    blocks
                }
                JoinConsumptionMutation::Exact
                | JoinConsumptionMutation::SkipFirst
                | JoinConsumptionMutation::DuplicateFirst
                | JoinConsumptionMutation::IncludeStaticallyUnselected
                | JoinConsumptionMutation::OmitFirstStaticallyUnselectedMatchCase
                | JoinConsumptionMutation::MaterializeFirstUnselectedMatchJoin
                | JoinConsumptionMutation::DispositionDynamicHostResultMerge => blocks,
            };
            for block in blocks {
                if reachable.contains(&block) {
                    return Err(backend_module(format!(
                        "materialized-but-dead source join {origin:?} retained a reachable block",
                    )));
                }
                if cfg
                    .pred_iter(block)
                    .any(|predecessor| reachable.contains(&predecessor.block))
                {
                    return Err(backend_module(format!(
                        "materialized-but-dead source join {origin:?} retained a live predecessor",
                    )));
                }
                let params = func.dfg.block_params(block);
                for reachable_block in &reachable {
                    for inst in func.layout.block_insts(*reachable_block) {
                        if func
                            .dfg
                            .inst_args(inst)
                            .iter()
                            .any(|argument| params.contains(argument))
                        {
                            return Err(backend_module(format!(
                                "materialized-but-dead source join {origin:?} retained a reachable use",
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Carry one generated-unit call input across the boundary.
    ///
    /// ⭐ A `Carried` input is already across and is returned untouched; a
    /// `Specialized` one crosses here.
    ///
    /// ⛔ **The origin is no longer the input's own, and that is a measured
    /// simplification rather than a regression.** It used to be, because an
    /// aggregate input's ownership record was resolved at whatever coordinate
    /// it was transferred at. It is not any more: every source aggregate now
    /// carries its producer occurrence and recovers its own schema from it, and
    /// the whole-graph preflight refuses one that does not. MEASURED: forcing
    /// every source-call input to cross at the program ROOT -- the maximally
    /// wrong coordinate -- leaves the suite at its exact baseline.
    fn carry_call_input(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        input: LoweringOperand,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        match input {
            LoweringOperand::Carried(word) => Ok(LoweringOperand::Carried(word)),
            LoweringOperand::Specialized(value) => Ok(LoweringOperand::Carried(
                self.transfer_into_carrier(builder, origin, &value)?,
            )),
        }
    }

    /// Carry a source-machine call's inputs across a **declared generated
    /// unit** boundary.
    ///
    /// ⛔ **No per-argument occurrence pairing, and its removal is the point.**
    /// The pairing existed so that an aggregate argument's planned ownership
    /// record would be resolved at the argument rather than at the callee's
    /// scheduling entry. That question no longer has a coordinate answer:
    /// ownership travels on the template and the schema is recovered from it,
    /// so the accumulator, the retained pending occurrence and the prefix
    /// template that mirrored it were carrying a fact nothing read.
    fn carry_source_call_inputs(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        inputs: Vec<LoweringOperand>,
    ) -> Result<Vec<LoweringOperand>, CraneliftBackendError> {
        // ⭐ **`D7` — the A/B AGGREGATE-OWNERSHIP discriminator's only seam.**
        // It moves the certificate the template carries; under it one argument
        // keeps its value, its callee, its parameter slot, its shape and its
        // lane, and takes a SIBLING argument's producer occurrence, so a
        // refusal is attributable to the certificate alone.
        #[cfg(test)]
        let inputs = self.substitute_sibling_aggregate_producer(inputs);
        // ⭐ **`D7` — the call-USE coordinate discriminator, at the same seam.**
        // It moves the coordinate every input is transferred at while the
        // certificate mutation above moves the certificate one input carries.
        // Same call, same arguments, same moment, two axes.
        #[cfg(test)]
        let origin = self.call_input_transfer_origin_under_mutation(origin)?;
        let mut carried = Vec::with_capacity(inputs.len());
        for input in inputs {
            carried.push(self.carry_call_input(builder, origin, input)?);
        }
        Ok(carried)
    }

    /// The coordinate source-call inputs are transferred at, under the
    /// call-use mutation only.
    ///
    /// ⚠ **The hit is counted only when the coordinate actually CHANGES.** A
    /// call already made at the root would otherwise report a substitution that
    /// substituted nothing, which is indistinguishable from a well-defended one.
    #[cfg(test)]
    fn call_input_transfer_origin_under_mutation(
        &self,
        origin: StaticOriginId,
    ) -> Result<StaticOriginId, CraneliftBackendError> {
        if GOVERNED_ALLOCATION_MUTATION.with(std::cell::Cell::get)
            != GovernedAllocationMutation::CallInputTransferOrigin
        {
            return Ok(origin);
        }
        let root = self.static_transition_plan.root_static_origin()?;
        if root != origin {
            governed_allocation_hit();
        }
        Ok(root)
    }

    /// Replace the FIRST argument's carried **producer occurrence** with the
    /// second's, under the A/B ownership mutation only.
    ///
    /// ⛔ Only the certificate moves. The template keeps its own constructor
    /// symbol, its own children, its own resolved identity, its own call use and
    /// its own parameter slot, so the emitter builds exactly what it built
    /// before and only the ownership record it claims differs.
    ///
    /// ⚠ **The hit is counted only when the occurrence actually CHANGES.** A
    /// call whose two arguments already share a record, or whose first argument
    /// is not a specialized aggregate, leaves the list untouched -- and a
    /// substitution that substitutes nothing is indistinguishable from a
    /// well-defended one if the counter fires anyway.
    #[cfg(test)]
    fn substitute_sibling_aggregate_producer(
        &self,
        mut inputs: Vec<LoweringOperand>,
    ) -> Vec<LoweringOperand> {
        if GOVERNED_ALLOCATION_MUTATION.with(std::cell::Cell::get)
            != GovernedAllocationMutation::SiblingAggregateProducer
        {
            return inputs;
        }
        if inputs.len() < 2 {
            return inputs;
        }
        let LoweringOperand::Specialized(sibling) = &inputs[1] else {
            return inputs;
        };
        let Some(sibling) = sibling.source_aggregate_producer() else {
            return inputs;
        };
        let LoweringOperand::Specialized(target) = &mut inputs[0] else {
            return inputs;
        };
        let carried = match target {
            Lowered::Constructor { occurrence, .. } | Lowered::Record { occurrence, .. } => {
                occurrence
            }
            _ => return inputs,
        };
        if *carried == Some(sibling) {
            return inputs;
        }
        let replaced = *carried;
        *carried = Some(sibling);
        let agreement = replaced.and_then(|replaced| {
            let before = self
                .static_transition_plan
                .aggregate_record_view(replaced)
                .ok()?;
            let after = self
                .static_transition_plan
                .aggregate_record_view(sibling)
                .ok()?;
            Some((
                before.shape() == after.shape(),
                before.allocation() == after.allocation(),
            ))
        });
        SIBLING_PRODUCER_SUBSTITUTION.with(|cell| {
            cell.set(Some(SiblingProducerSubstitution {
                from: replaced,
                to: sibling,
                same_shape: agreement.is_some_and(|(shape, _)| shape),
                same_lane: agreement.is_some_and(|(_, lane)| lane),
            }));
        });
        governed_allocation_hit();
        inputs
    }

    pub(super) fn call_declared_unit(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        body_origin: StaticOriginId,
        inputs: &[LoweringOperand],
        #[cfg(test)] launch_ingress: Option<cranelift_codegen::ir::Value>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let target = self
            .function_local
            .unit_calls
            .get(&body_origin)
            .cloned()
            .ok_or_else(|| {
                backend_module(format!(
                    "retained body {body_origin:?} has no graph-derived call target in this unit"
                ))
            })?;
        self.call_declared_unit_target(
            builder,
            target,
            inputs,
            #[cfg(test)]
            launch_ingress,
        )
        .map(|(operand, _inst)| operand)
    }

    /// **`RT-DECL-CLOSURE-PORT` `D4` — the call at a `DeclarationRef`, with its
    /// real inputs.**
    ///
    /// ⭐ `inputs` is the caller's ordered slice: the declaration's actual
    /// arguments in **parameter order**, followed by its retained captures in
    /// `D3` order. It is passed straight to the descriptor-driven emission
    /// below, which remains the sole authority for the exact
    /// `Parameter` + `Capture` slot run and rejects a slice that does not match
    /// it in either direction.
    ///
    /// ⛔ Nothing here re-derives the target: no callable identity word, no
    /// runtime lookup, no name parsing. The reference occurrence selects a
    /// record the planner already resolved and the bundle already declared.
    fn call_declared_declaration_unit(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        reference_origin: StaticOriginId,
        inputs: &[LoweringOperand],
        checked_template: Option<u64>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let target = self
            .function_local
            .declaration_calls
            .get(&reference_origin)
            .cloned()
            .ok_or_else(|| {
                backend_module(
                    "DeclarationRef has no planner-derived declaration call target".to_string(),
                )
            })?;
        let target_origin = target.origin;
        let target_function = target.function;
        let (operand, call) = self.call_declared_unit_target(
            builder,
            target,
            inputs,
            #[cfg(test)]
            None,
        )?;
        // `RT-DECL-CLOSURE-PORT` `D5` — the emitted-target oracle.
        //
        // ⭐ The callee is read back out of the **instruction that was actually
        // emitted**, not out of the declared map a second time. A control that
        // compared two reads of `declaration_calls` would agree with itself
        // whatever the emitter did; this disagrees the moment the emitted call
        // and the planner-resolved target diverge.
        // ⛔⛔ The callee is decoded out of the instruction that was ACTUALLY
        // emitted, never read back out of the declared map. That is what makes
        // the closeout's target comparison a comparison of two independently
        // produced facts.
        let emitted_callee = match builder.func.dfg.insts[call] {
            cranelift_codegen::ir::InstructionData::Call { func_ref, .. } => func_ref,
            _ => {
                return Err(backend_module(
                    "a declared unit call was not emitted as a direct call instruction".to_string(),
                ));
            }
        };
        #[cfg(test)]
        D5_EMITTED_DECLARATION_CALLS.with(|calls| {
            calls
                .borrow_mut()
                .push((reference_origin, target_origin, emitted_callee))
        });
        // `RT-DECL-CLOSURE-PORT` `D5` — one ledger entry per CHECKED call,
        // keyed by its template and bound to the exact reference occurrence and
        // resolved target. ⚠ An unchecked entry call carries no template id and
        // is deliberately outside this set.
        if let Some(call_template_id) = checked_template {
            #[cfg(test)]
            let mutation = D5_CLOSEOUT_MUTATION.with(std::cell::Cell::get);
            #[cfg(test)]
            if mutation == D5CloseoutMutation::SuppressLedgerEntry {
                // ⛔ The call itself is already emitted and lawful; only its
                // record is withheld. That is the whole point — the closeout
                // must notice a real call that no entry accounts for.
                return Ok(operand);
            }
            let ledger = self.checked_call_ledger.as_mut().ok_or_else(|| {
                backend_module(
                    "a checked declaration-unit call was emitted outside the unit bundle pass"
                        .to_string(),
                )
            })?;
            #[cfg(test)]
            let record = units::CheckedCallRecord {
                reference: reference_origin,
                target: target_origin,
                callee: if mutation == D5CloseoutMutation::SubstituteEmittedCallee {
                    target_function
                } else {
                    emitted_callee
                },
                resolved: if mutation == D5CloseoutMutation::SubstituteEmittedCallee {
                    // A ref this function certainly did not call.
                    builder
                        .func
                        .dfg
                        .ext_funcs
                        .keys()
                        .find(|candidate| *candidate != emitted_callee)
                        .unwrap_or(target_function)
                } else {
                    target_function
                },
            };
            #[cfg(not(test))]
            let record = units::CheckedCallRecord {
                reference: reference_origin,
                target: target_origin,
                callee: emitted_callee,
                resolved: target_function,
            };
            ledger.record_emitted(call_template_id, record)?;
            #[cfg(test)]
            match mutation {
                D5CloseoutMutation::DuplicateLedgerEntry => {
                    ledger.record_emitted(call_template_id, record)?;
                }
                D5CloseoutMutation::ExtraLedgerEntry => {
                    // ⚠ Keyed off the real template so each call site adds a
                    // DISTINCT unplanned entry. A single shared key would trip
                    // the duplicate check at the second call site instead, and
                    // this row would measure duplication rather than the
                    // planned-set membership it names.
                    ledger.record_emitted(call_template_id ^ u64::MAX, record)?;
                }
                D5CloseoutMutation::Exact
                | D5CloseoutMutation::SuppressLedgerEntry
                | D5CloseoutMutation::SubstituteEmittedCallee => {}
            }
        }
        Ok(operand)
    }

    /// Emit the direct call to a declared unit target.
    ///
    /// Returns the produced operand **and the exact `Inst` emitted for the
    /// call**. ⭐ The `Inst` is returned rather than kept in a `last_call` field
    /// (see also [`D5_EMITTED_DECLARATION_CALLS`])
    /// so that a caller which needs to attribute the emitted instruction has to
    /// take it from the emission itself; a stale side-channel would attribute
    /// one call site's instruction to another's token.
    fn call_declared_unit_target(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: units::DeclaredUnitCall,
        inputs: &[LoweringOperand],
        #[cfg(test)] launch_ingress: Option<cranelift_codegen::ir::Value>,
    ) -> Result<(LoweringOperand, cranelift_codegen::ir::Inst), CraneliftBackendError> {
        let payload = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            target.header.frame_bytes,
            3,
        ));
        let mut input = 0usize;
        let mut result_offset = None;
        let mut trap_offset = None;
        for (slot, offset) in target.slots.iter().zip(&target.offsets) {
            let offset = i32::try_from(*offset).map_err(|_| {
                backend_module("callee slot offset exceeds addressable range".to_string())
            })?;
            match slot.kind {
                AbiSlotKind::Parameter | AbiSlotKind::Capture => {
                    let value = inputs.get(input).ok_or_else(|| {
                        backend_module("callee frame is missing a declared input".to_string())
                    })?;
                    let word = match value {
                        LoweringOperand::Carried(word) => word.word,
                        LoweringOperand::Specialized(value) => {
                            // ⚠ **`target.origin` is the CALLEE's scheduling
                            // entry**, and what still arrives specialized here
                            // is what no earlier crossing took.
                            //
                            // ⛔ **The two earlier crossings are NOT the same
                            // mechanism, and conflating them is what this
                            // comment used to do.** `lower_expr`'s
                            // direct-closure-callee arm carries each input at
                            // its exact caller-side occurrence. The source
                            // machine's call path carries its inputs at ONE
                            // common transfer coordinate with no per-argument
                            // pairing — inert, because an aggregate carries and
                            // is preflighted against its own producer
                            // authority, and a non-aggregate queries no
                            // aggregate ownership.
                            //
                            // MEASURED after both, `--nocapture
                            // --test-threads=1` over the whole suite: 137
                            // `BorrowedNativeValue`, 137 `CapabilityToken`, 42
                            // `Int` and 1 `Bool` `Parameter`s, plus 55 `Int`
                            // `Capture`s — all non-aggregate, so a
                            // `NonAggregate` request takes the caller's tag,
                            // consults no planned record and enters neither `E`
                            // nor `R`. The origin is not load-bearing for any
                            // of them. **No aggregate `Capture` reaches here at
                            // all**, so the capture-authority witness does not
                            // exist.
                            //
                            // ⛔ The one remaining aggregate population is
                            // **`Constructor` `Parameter`s from
                            // `call_static_worker`** (traced by backtrace, not
                            // inferred). They reach this fallback and
                            // **self-authorize**: each carries its own producer
                            // occurrence, so the coordinate below is not the
                            // authority its ownership record is resolved at.
                            //
                            // ⚠ No guard here refusing aggregates: measured, it
                            // would refuse those 97 inputs, which compile today.
                            #[cfg(test)]
                            if value.source_aggregate_producer().is_some() {
                                SELF_AUTHORIZED_FALLBACK_REACHES
                                    .with(|n| n.set(n.get().saturating_add(1)));
                            }
                            self.transfer_into_carrier(
                                builder,
                                self.callee_scheduling_origin_under_mutation(target.origin),
                                value,
                            )?
                            .word
                        }
                    };
                    builder.ins().stack_store(word, payload, offset);
                    input += 1;
                }
                AbiSlotKind::Control | AbiSlotKind::Store => {
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().stack_store(zero, payload, offset);
                }
                AbiSlotKind::Trap => {
                    #[cfg(test)]
                    let zero = match TRAP_CALLER_PROTOCOL_MUTATION
                        .with(std::cell::Cell::get)
                    {
                        TrapCallerProtocolMutation::LeaveStaleTrap => {
                            builder.ins().iconst(types::I64, 1)
                        }
                        TrapCallerProtocolMutation::Exact
                        | TrapCallerProtocolMutation::ReadResultBeforeTrap => {
                            builder.ins().iconst(types::I64, 0)
                        }
                    };
                    #[cfg(not(test))]
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().stack_store(zero, payload, offset);
                    trap_offset = Some(offset);
                }
                AbiSlotKind::Result => {
                    #[cfg(test)]
                    if TRAP_CALLER_PROTOCOL_MUTATION.with(std::cell::Cell::get)
                        == TrapCallerProtocolMutation::ReadResultBeforeTrap
                    {
                        let false_word = builder.ins().iconst(types::I64, 0);
                        builder.ins().stack_store(false_word, payload, offset);
                    }
                    result_offset = Some(offset);
                }
            }
        }
        if input != inputs.len() {
            return Err(backend_module(
                "caller supplied inputs absent from the callee descriptor".to_string(),
            ));
        }
        let pointer_type = builder.func.dfg.value_type(
            self.function_local
                .services_pointer
                .ok_or_else(|| backend_module("unit call has no services pointer".to_string()))?,
        );
        let slots = builder.ins().stack_addr(pointer_type, payload, 0);
        let envelope = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            u32::try_from(crate::activation_services::UNIT_CALL_FRAME_BYTES)
                .expect("unit call frame byte count fits u32"),
            3,
        ));
        builder.ins().stack_store(
            slots,
            envelope,
            crate::activation_services::UNIT_CALL_FRAME_SLOTS,
        );
        let services = self
            .function_local
            .services_pointer
            .expect("services pointer checked above");
        let exact_host_dispatch_context =
            self.function_local.host_dispatch_context.ok_or_else(|| {
                backend_module("unit call has no direct host-dispatch context".to_string())
            })?;
        #[cfg(test)]
        let host_dispatch_context = if launch_ingress.is_some()
            && PROCESS_SLOT_MUTATION.with(std::cell::Cell::get)
                == ProcessSlotMutation::ReintroduceLaunchIngress
        {
            // This is the deliberately forbidden half of the AC-14 control:
            // unlike the retained direct context, this value is explicitly
            // sourced from the root adapter's launch-ingress parameter.
            launch_ingress.expect("the root adapter supplied launch ingress")
        } else {
            HOST_CONTEXT_PROPAGATION_MUTATION.with(|cell| match cell.get() {
                HostContextPropagationMutation::Exact => exact_host_dispatch_context,
                HostContextPropagationMutation::ServicesPointer if launch_ingress.is_none() => {
                    services
                }
                HostContextPropagationMutation::NativeIntArena if launch_ingress.is_none() => self
                    .function_local
                    .native_int_arena
                    .expect("unit native-int arena is bound"),
                HostContextPropagationMutation::BoundaryArena if launch_ingress.is_none() => self
                    .function_local
                    .boundary_arena
                    .expect("unit boundary arena is bound"),
                HostContextPropagationMutation::Null if launch_ingress.is_none() => {
                    builder.ins().iconst(pointer_type, 0)
                }
                HostContextPropagationMutation::LaunchIngress => {
                    launch_ingress.unwrap_or(exact_host_dispatch_context)
                }
                HostContextPropagationMutation::ServicesPointer
                | HostContextPropagationMutation::NativeIntArena
                | HostContextPropagationMutation::BoundaryArena
                | HostContextPropagationMutation::Null => exact_host_dispatch_context,
            })
        };
        #[cfg(not(test))]
        let host_dispatch_context = exact_host_dispatch_context;
        builder.ins().stack_store(
            host_dispatch_context,
            envelope,
            crate::activation_services::UNIT_CALL_FRAME_HOST_DISPATCH_CONTEXT,
        );
        let envelope = builder.ins().stack_addr(pointer_type, envelope, 0);
        let call = builder.ins().call(target.function, &[envelope, services]);
        let [unit_status] = builder.inst_results(call) else {
            return Err(backend_module(
                "internal unit call did not return exactly one word".to_string(),
            ));
        };
        let unit_status = *unit_status;
        let failed = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::NotEqual,
            unit_status,
            0,
        );
        let failure_block = builder.create_block();
        let trap_check_block = builder.create_block();
        builder
            .ins()
            .brif(failed, failure_block, &[], trap_check_block, &[]);
        builder.switch_to_block(failure_block);
        builder.ins().return_(&[unit_status]);
        builder.seal_block(failure_block);
        builder.switch_to_block(trap_check_block);
        builder.seal_block(trap_check_block);
        let trap_offset = trap_offset.ok_or_else(|| {
            backend_module("callee frame declares no trap slot".to_string())
        })?;
        let result_offset = result_offset.ok_or_else(|| {
            backend_module("callee frame declares no result slot".to_string())
        })?;
        #[cfg(test)]
        if TRAP_CALLER_PROTOCOL_MUTATION.with(std::cell::Cell::get)
            == TrapCallerProtocolMutation::ReadResultBeforeTrap
        {
            let word = builder.ins().stack_load(types::I64, payload, result_offset);
            return Ok((LoweringOperand::Carried(CarriedBoundaryWord { word }), call));
        }
        let trap_word = builder.ins().stack_load(types::I64, payload, trap_offset);
        let trapped = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::NotEqual,
            trap_word,
            0,
        );
        let trap_block = builder.create_block();
        let result_block = builder.create_block();
        builder.ins().brif(trapped, trap_block, &[], result_block, &[]);
        builder.switch_to_block(trap_block);
        match self.function_local.trap_exit {
            Some(TrapExitAuthority::UnitFrame { slots, trap_offset }) => {
                #[cfg(test)]
                px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::UnitTrapWordPropagated {
                    seat: PlannedTrapSeat::UnitTrapWord,
                    identity_preserved: true,
                });
                builder
                    .ins()
                    .store(MemFlags::trusted(), trap_word, slots, trap_offset);
                let no_result = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[no_result]);
            }
            Some(TrapExitAuthority::Root {
                process_sentinel: true,
                ..
            }) => {
                #[cfg(test)]
                px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::UnitTrapWordPropagated {
                    seat: PlannedTrapSeat::RootProcessSentinel,
                    identity_preserved: false,
                });
                let process_trap = builder.ins().iconst(types::I64, -4);
                builder.ins().return_(&[process_trap]);
            }
            Some(TrapExitAuthority::Root {
                process_sentinel: false,
                ..
            }) => {
                #[cfg(test)]
                px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::UnitTrapWordPropagated {
                    seat: PlannedTrapSeat::RootTrapToken,
                    identity_preserved: true,
                });
                let shifted = builder.ins().ishl_imm(
                    trap_word,
                    crate::cranelift_backend::compiled::ROOT_TRAP_TOKEN_SHIFT,
                );
                let root_token = builder.ins().bor_imm(
                    shifted,
                    crate::cranelift_backend::compiled::ROOT_TRAP_TOKEN_TAG,
                );
                builder.ins().return_(&[root_token]);
            }
            None => {
                return Err(backend_module(
                    "trap branch has no generated-unit TrapWord lane".to_string(),
                ));
            }
        }
        builder.seal_block(trap_block);
        builder.switch_to_block(result_block);
        builder.seal_block(result_block);
        let word = builder.ins().stack_load(types::I64, payload, result_offset);
        Ok((LoweringOperand::Carried(CarriedBoundaryWord { word }), call))
    }

    /// **`RT-CONTSPEC-ACTIVATE` `4b` — decode the callee of an emitted direct
    /// call out of the finished CLIF.**
    ///
    /// ⭐ **This is the independent side of the emission gate.** It reads the
    /// instruction stream that was actually built: the instruction's
    /// `func_ref`, that ref's `ExtFuncData` name, and the function's own
    /// imported-user-name table, which `Module::declare_func_in_func` populates
    /// with `UserExternalName { namespace: 0, index: func_id }`. ⛔ Nothing here
    /// consults `continuation_calls`, the claim ledger's `resolved` map, or the
    /// `DeclaredUnitCall` that was handed to the emitter -- those are all
    /// downstream of the same resolution and comparing against one of them
    /// would be a re-run of the builder under test.
    ///
    /// ⛔ A non-direct call, a non-user name, or a foreign namespace is a
    /// rejection rather than a skip: an unattributable callee must not read as
    /// agreement.
    fn decode_direct_callee(
        func: &Function,
        inst: cranelift_codegen::ir::Inst,
    ) -> Result<FuncId, CraneliftBackendError> {
        let cranelift_codegen::ir::InstructionData::Call { func_ref, .. } = func.dfg.insts[inst]
        else {
            return Err(backend_module(
                "an emitted continuation call site does not hold a direct call instruction"
                    .to_string(),
            ));
        };
        let cranelift_codegen::ir::ExternalName::User(name_ref) = func.dfg.ext_funcs[func_ref].name
        else {
            return Err(backend_module(
                "an emitted continuation call names a callee that is not a user function"
                    .to_string(),
            ));
        };
        let user = &func.params.user_named_funcs()[name_ref];
        if user.namespace != 0 {
            return Err(backend_module(
                "an emitted continuation call names a callee outside the module function namespace"
                    .to_string(),
            ));
        }
        Ok(FuncId::from_u32(user.index))
    }

    /// **`RT-CONTSPEC-ACTIVATE` `4b` — the emission-seam equality gate for one
    /// generated function.**
    ///
    /// For every causal token this function emitted a call for, prove
    ///
    /// ```text
    /// bundle.continuation(identity.target())  ==  callee decoded from the CLIF
    /// ```
    ///
    /// **The two sides come from different producers.** The left is the
    /// planner's own four-field projection (`continuation_call_binding_for`)
    /// carried through the forward-declaration table; the right is the finished
    /// instruction stream. ⛔ Neither is `continuation_calls` and neither is the
    /// ledger's `resolved` map -- those are what the emitter *was handed*, and a
    /// gate built on them would agree with a redirected emission.
    ///
    /// ⚠ **The honest residual:** `bundle` is the naming authority that decides
    /// which `FuncId` a specialization *is*, so a gate cannot get behind it --
    /// if the forward declaration itself named the wrong function, both sides
    /// move together. What this proves is that the routing from planned identity
    /// to emitted callee is exact, not that the declaration table is right.
    /// What the callee's **body** then computes is deferred to
    /// `RT-CONTSPEC-WITNESS` `D7`/`AC-9` and is not claimed here.
    ///
    /// The second half is closure: every direct call in the function whose
    /// callee is *any* planned continuation specialization must be one of the
    /// recorded ones. ⛔ Without it the gate would be complete only over the set
    /// it built itself -- an unrecorded emission would be invisible, and the
    /// records would look exhaustive because nothing disagreed with them.
    fn verify_emitted_continuation_calls(
        &self,
        func: &Function,
        bundle: &units::UnitBundle,
    ) -> Result<(), CraneliftBackendError> {
        let mut expected_by_callee: BTreeMap<FuncId, usize> = BTreeMap::new();
        for (identity, inst) in &self.function_local.continuation_emissions {
            let planned = bundle.continuation(identity.target()).ok_or_else(|| {
                backend_module(
                    "an emitted causal token names a specialization that was never \
                     forward-declared"
                        .to_string(),
                )
            })?;
            let emitted = Self::decode_direct_callee(func, *inst)?;
            if emitted != planned {
                return Err(backend_module(format!(
                    "the emitted direct-call target {emitted:?} disagrees with the planner-issued \
                     continuation target {planned:?} for a causal token; the call that was built \
                     is not the call that was planned"
                )));
            }
            *expected_by_callee.entry(planned).or_default() += 1;
        }

        // Closure: no continuation call may be emitted that was not recorded.
        let mut specialization_callees = BTreeSet::new();
        for unit in self.static_transition_plan.continuation_units()? {
            let id = bundle.continuation(unit.id()).ok_or_else(|| {
                backend_module(
                    "a planned continuation specialization was never forward-declared".to_string(),
                )
            })?;
            specialization_callees.insert(id);
        }
        // ⛔ Not a fast path around the check: with no planned specialization
        // there is no callee the scan could recognise, and the loop above has
        // already rejected any recorded emission naming one -- so `expected` is
        // necessarily empty here too. A program with no continuations skips an
        // instruction walk it could only ever conclude nothing from.
        if specialization_callees.is_empty() {
            return Ok(());
        }
        let mut observed_by_callee: BTreeMap<FuncId, usize> = BTreeMap::new();
        for block in func.layout.blocks() {
            for inst in func.layout.block_insts(block) {
                let cranelift_codegen::ir::InstructionData::Call { func_ref, .. } =
                    func.dfg.insts[inst]
                else {
                    continue;
                };
                let cranelift_codegen::ir::ExternalName::User(name_ref) =
                    func.dfg.ext_funcs[func_ref].name
                else {
                    continue;
                };
                let user = &func.params.user_named_funcs()[name_ref];
                if user.namespace != 0 {
                    continue;
                }
                let callee = FuncId::from_u32(user.index);
                if specialization_callees.contains(&callee) {
                    *observed_by_callee.entry(callee).or_default() += 1;
                }
            }
        }
        if observed_by_callee != expected_by_callee {
            return Err(backend_module(format!(
                "the continuation calls this function actually emitted {observed_by_callee:?} are \
                 not the ones recorded against planned causal tokens {expected_by_callee:?}"
            )));
        }
        Ok(())
    }

    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8j` — verifications 3, 4 and 5, against
    /// the FINISHED CLIF, and the only route into the composed relation.**
    ///
    /// A claim recorded at the source-machine seat proved what could be proved
    /// then: the authority came from the exact paired planner target, and the
    /// claiming function is that authority's own emission owner. Everything
    /// remaining is a fact about instructions, and instructions are not
    /// readable until the function is built.
    ///
    /// 3. **The finished stream contains the recorded call.** A record naming
    ///    an instruction that is not in the layout is a record about a call
    ///    this function did not make.
    /// 4. **The decoded callee and the operand contract agree with the exact
    ///    `D8b`/`D8d` target.** ⭐ The callee is decoded from the instruction
    ///    and compared against the `FuncId` the BUNDLE gives the emittable unit
    ///    whose origin is the target's worker body — two different producers.
    ///    ⚠ The operand half is **not** decoded: this call passes its run
    ///    through a frame slot rather than as call arguments, so what is
    ///    compared is the run the emitter reported supplying against the run
    ///    the planner target declares. That is still two producers, but it is
    ///    not a CLIF fact and this comment is where that limit is written down.
    /// 5. **The result returns into the unchanged source-machine
    ///    continuation.** The value handed on must be a result of the recorded
    ///    instruction — a CLIF fact — and the live source-continuation depth
    ///    must be what it was before the emitter ran.
    ///
    /// ⛔ Records are promoted one at a time and a duplicate identity refuses:
    /// one causal obligation cannot be answered twice.
    fn verify_recorded_composed_discharges(
        &mut self,
        func: &Function,
        bundle: &units::UnitBundle,
    ) -> Result<(), CraneliftBackendError> {
        let pending = std::mem::take(&mut self.function_local.pending_composed_discharges);
        if pending.is_empty() {
            return Ok(());
        }
        let mut emitted_insts = BTreeSet::new();
        for block in func.layout.blocks() {
            for inst in func.layout.block_insts(block) {
                emitted_insts.insert(inst);
            }
        }
        for record in pending {
            // ⛔ `D8j` — the REDIRECT switch, applied to the verifier's input
            // and only where a genuinely different call exists in the finished
            // function. It moves the record onto another real instruction,
            // which is what an attribution mistake would look like; a
            // fabricated `Inst` would be caught by clause 3 instead and would
            // prove the wrong guard.
            #[cfg(test)]
            let record = if d8j_mutation() == D8jMutation::RedirectRecordedInstruction {
                let mut other = None;
                'redirect: for block in func.layout.blocks() {
                    for inst in func.layout.block_insts(block) {
                        if inst == record.inst {
                            continue;
                        }
                        if matches!(
                            func.dfg.insts[inst],
                            cranelift_codegen::ir::InstructionData::Call { .. }
                        ) {
                            other = Some(inst);
                            break 'redirect;
                        }
                    }
                }
                match other {
                    Some(inst) => PendingComposedDischarge { inst, ..record },
                    None => record,
                }
            } else {
                record
            };
            // 3 — the recorded call is in the finished stream.
            if !emitted_insts.contains(&record.inst) {
                return Err(backend_module(
                    "a claimed composed discharge names an instruction the finished function \
                     does not contain, so it answers with a call this function never made"
                        .to_string(),
                ));
            }
            // 4a — the decoded callee is the planner's own raw worker.
            let unit = self
                .static_transition_plan
                .emittable_units()?
                .into_iter()
                .find(|unit| unit.origin() == record.worker_body_origin)
                .ok_or_else(|| {
                    backend_module(
                        "a claimed composed discharge names a worker body with no emittable unit"
                            .to_string(),
                    )
                })?;
            let planned = bundle.function(unit.function()).ok_or_else(|| {
                backend_module(
                    "a claimed composed discharge names a worker unit that was never \
                     forward-declared"
                        .to_string(),
                )
            })?;
            let decoded = Self::decode_direct_callee(func, record.inst)?;
            if decoded != planned {
                return Err(backend_module(format!(
                    "a composed discharge's recorded instruction calls {decoded:?}, but the \
                     D8b/D8d target's raw worker is {planned:?}; the call that was made is not \
                     the call the authority stands for"
                )));
            }
            // 4b — the operand contract, reported against declared.
            if record.supplied_operands != record.declared_operands {
                return Err(backend_module(format!(
                    "a composed discharge supplied {} operands but its D8b/D8d target declares \
                     {}; the raw worker's contract and the call that answers for it disagree",
                    record.supplied_operands, record.declared_operands
                )));
            }
            // 5 — the result returned into the unchanged continuation.
            #[cfg(test)]
            let record = if d8j_mutation() == D8jMutation::RecordResultDefinedBeforeTheCall {
                let mut earliest = None;
                'earliest: for block in func.layout.blocks() {
                    for inst in func.layout.block_insts(block) {
                        if inst == record.inst {
                            break 'earliest;
                        }
                        if let [value, ..] = func.dfg.inst_results(inst) {
                            earliest = Some(*value);
                            break 'earliest;
                        }
                    }
                }
                match earliest {
                    Some(value) => PendingComposedDischarge {
                        result: Some(value),
                        ..record
                    },
                    None => record,
                }
            } else {
                record
            };
            let result = record.result.ok_or_else(|| {
                backend_module(
                    "a composed discharge's call result was not a carried word, so nothing \
                     returned into source-machine control for it to be"
                        .to_string(),
                )
            })?;
            // ⚠ **Downstream, not the call's own SSA result, and the
            // difference is written here rather than papered over.** A declared
            // unit returns through its callee frame: the emitter writes the
            // operand run into a stack slot, calls, and LOADS the answer back.
            // So there is no SSA result on the call instruction to compare
            // against, and a check demanding one would assert a CLIF shape this
            // ABI does not have.
            //
            // ⭐ What IS a CLIF fact, and what is checked: the value handed to
            // source-machine control is defined at or after the recorded call,
            // in the function's block layout. ⚠ NOT in the call's own block:
            // the emitter branches on the callee's status word and loads the
            // answer in a successor it creates for that purpose, so demanding
            // the same block would refuse every lawful call. Nothing before the
            // call can have produced the value, which is the property.
            let mut reached = false;
            let mut downstream = false;
            'blocks: for block in func.layout.blocks() {
                for inst in func.layout.block_insts(block) {
                    if inst == record.inst {
                        reached = true;
                    }
                    if reached && func.dfg.inst_results(inst).contains(&result) {
                        downstream = true;
                        break 'blocks;
                    }
                }
            }
            if !downstream {
                return Err(backend_module(
                    "a composed discharge's recorded result is not defined at or after its \
                     recorded call in the finished block layout, so the value handed to \
                     source-machine control cannot have come from the call that answered the \
                     obligation"
                        .to_string(),
                ));
            }
            let (before, after) = record.source_control;
            if before != after {
                return Err(backend_module(format!(
                    "a composed discharge ran with {before} live source continuations and \
                     returned into {after}; the result did not return into the continuation \
                     that was in force"
                )));
            }
            if self
                .function_local
                .composed_discharges
                .insert(record.identity, record.inst)
                .is_some()
            {
                return Err(backend_module(
                    "one causal identity was discharged twice in a single function".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// The recursive emission step. ⛔ Private, and ⛔ never the entry point —
    /// see [`Self::transfer_into_carrier`] for why the split is not stylistic.
    ///
    /// ⭐ **The dispatch is an exhaustive `match` on the variant, and the
    /// `(tag, class)` comes from [`Lowered::boundary_disposition`].** Both
    /// halves are load-bearing and they answer different questions: the
    /// disposition is the **sole authority** for how a value is represented
    /// (`§2h` ¶4 makes reading it *required*), while the variant match is what
    /// supplies the **payload and children**, which a disposition cannot carry
    /// because it is a function of the variant tag alone.
    ///
    /// ⛔ **No wildcard arm.** A 22nd `Lowered` inhabitant is a compile error
    /// here, exactly as it is in `variant()` and
    /// `boundary_transfer_admissibility` — so a new carrier of children cannot
    /// be added without someone deciding whether it can cross.
    fn emit_carrier_transfer(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        value: &Lowered,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        match value {
            // ── the supported transfer surface ───────────────────────────
            Lowered::Bool { value: word, .. } => {
                let tag = Self::carrier_immediate_tag(value)?;
                self.emit_carrier_immediate(builder, tag, *word)
            }
            // ── the magnitude dispatch: `spill: Some(_)` immediates ───────
            //
            // ⭐ Four variants, ONE mechanism — which is the corrected `D9`
            // partition. ⛔ They are not four pieces of work and must not be
            // spelled as four arms with four bodies: the disposition supplies
            // the tag and the spill class, so the only thing that differs
            // between them is where the payload word and its `NativeIntV1`
            // marker come from.
            Lowered::Int {
                value: payload,
                known,
            } => {
                let (tag, spill) = Self::carrier_spillable_disposition(value)?;
                // ⛔ The marker travels with the payload; see
                // `carrier_small_marker` for why this is not a constant.
                let marker = self.native_int_tag(builder, *payload, *known)?;
                self.emit_carrier_native_int(builder, tag, spill, *payload, marker)
            }
            Lowered::ProcessExitStatus { value: payload }
            | Lowered::BoundedNat(BoundedNatV1 { value: payload })
            | Lowered::StructuralNat(StructuralNatV1 { value: payload }) => {
                let (tag, spill) = Self::carrier_spillable_disposition(value)?;
                let marker = Self::carrier_small_marker(builder);
                self.emit_carrier_spillable_immediate(builder, tag, spill, *payload, marker)
            }
            // ── byte-bodied handles ───────────────────────────────────────
            //
            // ⛔ Two arms, ONE emitter, and the class comes from the
            // disposition rather than from which arm we are in — see
            // `emit_carrier_bytes` for why a shared body driven by the class is
            // the thing that makes `String`'s guard reachable at all.
            Lowered::String(text) => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                self.emit_carrier_bytes(builder, tag, class, text.as_bytes())
            }
            Lowered::Bytes(content) => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                self.emit_carrier_bytes(builder, tag, class, content)
            }
            Lowered::Constructor {
                constructor,
                synthesized_identity,
                // Read from `value` by the disposition below, which needs the
                // whole template. Bound explicitly rather than swallowed by a
                // `..` so a further field is a compile error here.
                occurrence: _,
                args,
            } => {
                let (occurrence, class) = self.aggregate_carrier_authority(
                    origin,
                    value,
                    PlannedAggregateShape::Constructor,
                )?;
                let schema_origin = self.aggregate_schema_origin(occurrence, origin)?;
                // ⭐ `D2` — the identity comes from the ONE artifact-static
                // authority, via the typed newtype's own ABI-word method. ⛔ Not
                // `intern_symbol`, which is dense insertion-order numbering over
                // one store instance and therefore a *different* number in a
                // different store (`§2e`).
                let identity = match synthesized_identity {
                    Some(identity) => *identity,
                    None => self
                        .static_transition_plan
                        .constructor_symbol_identity(schema_origin)
                        .map_err(|error| {
                            backend_module(format!(
                                "constructor transfer for {constructor} at {:?} has no \
                                 resolved identity: {error}",
                                schema_origin
                            ))
                        })?,
                }
                .tag_abi_word()?;
                let word = self.emit_checked_aggregate_alloc(
                    builder,
                    GovernedAllocationSite::SourceConstructor,
                    occurrence,
                    PlannedAggregateShape::Constructor,
                    class,
                    args.len(),
                )?;
                self.emit_carrier_store_tag_id(builder, word, identity)?;
                for (position, argument) in args.iter().enumerate() {
                    let child = self.emit_carrier_transfer(builder, origin, argument)?;
                    self.emit_carrier_store_field(builder, word, position, child)?;
                }
                Ok(word)
            }
            Lowered::Record { fields, .. } => {
                let (occurrence, class) = self.aggregate_carrier_authority(
                    origin,
                    value,
                    PlannedAggregateShape::Record,
                )?;
                // ⛔ No schema coordinate here, and its absence is the point:
                // a record's field names now travel on the template, so this
                // arm resolves NOTHING at the coordinate it is transferred at.
                let word = self.emit_checked_aggregate_alloc(
                    builder,
                    GovernedAllocationSite::SourceRecord,
                    occurrence,
                    PlannedAggregateShape::Record,
                    class,
                    fields.len(),
                )?;
                for (position, field) in fields.iter().enumerate() {
                    // ⭐ `D2` at the field-identity namespace: the name written
                    // here and the name `Project` looks up are the same word
                    // from the same authority. ⚠ The `String` key on the field
                    // is deliberately NOT the identity — it is the compile-time
                    // spelling, and using it would be the second derivation
                    // `D2` forbids.
                    //
                    // ⭐⭐ **The identity EMITTED is the one the preflight
                    // COMPARED.** It travels on the template from the producer
                    // that was issued it, and the whole-graph walk has already
                    // made it agree with the plan at this exact position. ⛔ Not
                    // a fresh `record_field_identity` lookup: a second read at
                    // emission is a second authority, and it answers from
                    // whatever coordinate is in scope here rather than from the
                    // fact that was checked.
                    let Some(identity) = field.identity else {
                        return Err(unsupported(
                            "Record",
                            format!(
                                "record field {position} reached the carrier with no                                  planner-issued identity, so its name would have to be                                  invented at the coordinate it is transferred at"
                            ),
                        ));
                    };
                    self.emit_carrier_store_name(
                        builder,
                        word,
                        position,
                        identity.name_abi_word()?,
                    )?;
                    let child = self.emit_carrier_transfer(builder, origin, &field.value)?;
                    self.emit_carrier_store_field(builder, word, position, child)?;
                }
                Ok(word)
            }

            // ── ⛔ FAIL CLOSED — and these are DEFERRALS, said plainly ────
            //
            // ⚠ A deferral is honest; a deferral that reads as delivery is not.
            // Each arm below is a form the carrier ABI *can* represent and this
            // producer does not yet emit. ⛔ Do not read the fail-closed status
            // as "the boundary refuses this" — `boundary_disposition` admits
            // most of them. The refusal is **this producer's**, and it is
            // conservative rather than silent precisely so the gap cannot be
            // mistaken for coverage.
            Lowered::HostResult {
                success, error, ok, ..
            } => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                let ok = self.emit_carrier_transfer(builder, origin, ok)?;
                let error = self.emit_carrier_transfer(builder, origin, error)?;
                let word = self.emit_carrier_alloc(
                    builder,
                    CarrierAllocationRequest::NonAggregate { tag },
                    class,
                    2,
                )?;
                let success = if builder.func.dfg.value_type(*success) == types::I64 {
                    *success
                } else {
                    builder.ins().uextend(types::I64, *success)
                };
                self.emit_carrier_store_scalar(builder, word, success)?;
                self.emit_carrier_store_field(builder, word, 0, ok)?;
                self.emit_carrier_store_field(builder, word, 1, error)?;
                Ok(word)
            }
            Lowered::DynamicConstructor(dynamic) => {
                self.emit_carrier_dynamic_constructor(builder, origin, dynamic)
            }
            Lowered::ResourceToken { value: payload } => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                let word = self.emit_carrier_alloc(
                    builder,
                    CarrierAllocationRequest::NonAggregate { tag },
                    class,
                    0,
                )?;
                self.emit_carrier_store_scalar(builder, word, *payload)?;
                Ok(word)
            }
            Lowered::CapabilityToken { value: payload } => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                let word = self.emit_carrier_alloc(
                    builder,
                    CarrierAllocationRequest::NonAggregate { tag },
                    class,
                    0,
                )?;
                self.emit_carrier_store_scalar(builder, word, *payload)?;
                Ok(word)
            }
            Lowered::BorrowedNativeValue { pointer } => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                let word = self.emit_carrier_alloc(
                    builder,
                    CarrierAllocationRequest::NonAggregate { tag },
                    class,
                    0,
                )?;
                self.emit_carrier_store_scalar(builder, word, *pointer)?;
                Ok(word)
            }
            Lowered::ResponseBytes { pointer, len } => {
                let (tag, class) = Self::carrier_handle_disposition(value)?;
                let word = self.emit_carrier_alloc(
                    builder,
                    CarrierAllocationRequest::NonAggregate { tag },
                    class,
                    1,
                )?;
                self.emit_carrier_store_scalar(builder, word, *pointer)?;
                let len = self.emit_carrier_immediate(builder, BoundaryTag::ImmediateInt, *len)?;
                self.emit_carrier_store_field(builder, word, 0, len)?;
                Ok(word)
            }
            Lowered::BorrowedOption { .. } => Err(unsupported(
                lowered_value_kind(value),
                "the carrier producer does not yet emit borrowed ingress: an \
                 `InvocationBorrowed` handle is arena-owned and must clear \
                 `escape_check` before it may be written into a parent",
            )),

            // ── ⛔ REFUSED, not deferred — and structurally required here ──
            //
            // ⚠ Stated honestly: these arms are **unreachable in practice**,
            // because `boundary_transfer_admissibility` rejects the three
            // closure forms at the entry point and `boundary_disposition`
            // classifies the last two as `ProtocolOnly`. They are spelled
            // anyway because exhaustiveness is the mechanism that makes a 22nd
            // variant a compile error — ⛔ collapsing them into a `_` arm would
            // buy three lines and spend the whole closure property.
            Lowered::Closure { .. }
            | Lowered::DeclarationClosure { .. }
            | Lowered::ComputationalRecursorClosure { .. } => Err(unsupported(
                lowered_value_kind(value),
                "a closure has no durable lane and cannot cross the boundary; \
                 this arm is unreachable because the admissibility walk already \
                 refused the graph",
            )),
            Lowered::RecursiveBackedge | Lowered::Trap(_) => Err(unsupported(
                lowered_value_kind(value),
                "protocol machinery is never a source value at a boundary",
            )),
        }
    }

    /// The `(tag, class)` of a **handle**-represented value, read from the sole
    /// disposition authority (`§2h` ¶4).
    ///
    /// ⭐ **This is the typed boundary in front of the emission step**, and it
    /// is wildcard-free over [`BoundaryDisposition`] on purpose: a fifth
    /// disposition would break compilation here rather than silently taking
    /// whichever arm a `_` had swallowed.
    /// **`D7` — the aggregate allocation tag, taken from the planner record.**
    ///
    /// ⛔ The value-shape disposition answers a DIFFERENT question. It reports
    /// the lane a `Constructor`/`Record` takes *considered alone*, which is
    /// always the persistent one — the shape is persistable. Whether this
    /// particular aggregate may take it depends on its children's lifetimes,
    /// which the value in hand does not carry and this producer may not go
    /// looking for.
    ///
    /// ⚠ So this deliberately keeps the disposition's CLASS and replaces only
    /// its TAG. The class is a fact about the shape and the disposition is its
    /// authority; the lane is a fact about the meet and the planner is its.
    /// The record identity and class of one aggregate about to be allocated.
    ///
    /// ⭐ Returns the OCCURRENCE, not a lane. The lane is the checked wrapper's
    /// to read, so there is exactly one place a planned record becomes a
    /// `BoundaryTag` and exactly one place an event is recorded — a caller
    /// cannot obtain the lane and then allocate without leaving a pair.
    /// **`RT-DECL-CLOSURE-PORT` `D7` — the coordinate an aggregate's SCHEMA is
    /// resolved at, recovered from its own ownership record.**
    ///
    /// ⭐⭐ **Ownership was only half the defect.** Carrying the occurrence fixed
    /// which record an aggregate names; it did not fix where its constructor
    /// symbol, its field NAMES and its child positions are looked up, and those
    /// are keyed on a coordinate. A source record forwarded through a `Var` and
    /// handed to a call was still asking for its field names at the `Var` --
    /// measured, as `"static origin ... has no RecordFieldName atom at
    /// occurrence 0"`, on the released forwarded-record row.
    ///
    /// ⛔ The producer origin comes from the ownership record the template
    /// names, so it is recovered rather than transported. Nothing here searches
    /// for it, and no caller may pass one.
    ///
    /// ⚠ A compiler-synthesized aggregate has no source origin at all, so it
    /// keeps the transfer coordinate -- which for a synthesized subtree is the
    /// seat its whole tree is rooted at, and is the coordinate its children are
    /// reached under.
    ///
    /// ⛔ **There is no longer a `synthesized` flag here, and its removal is a
    /// measurement rather than a tidy-up.** It selected between two child
    /// coordinates that were genuinely DIFFERENT values -- 152 of 152 reached
    /// emissions took the two arms to different origins, 38 of them with a
    /// synthesized producer -- and the whole suite was green under either arm.
    /// The coordinate a child is transferred at is inert: a leaf never reads
    /// it, and an aggregate recovers its own from the record it carries. A
    /// decision whose two answers are indistinguishable is not a decision, and
    /// keeping it would have kept a `child_static_origin` lookup on the
    /// emission path that nothing consumed.
    fn aggregate_schema_origin(
        &self,
        occurrence: AggregateOccurrenceId,
        transfer: StaticOriginId,
    ) -> Result<StaticOriginId, CraneliftBackendError> {
        Ok(self
            .static_transition_plan
            .aggregate_record_view(occurrence)?
            .producer_origin()
            .unwrap_or(transfer))
    }

    fn aggregate_carrier_authority(
        &self,
        origin: StaticOriginId,
        value: &Lowered,
        shape: PlannedAggregateShape,
    ) -> Result<(AggregateOccurrenceId, BoundaryClass), CraneliftBackendError> {
        let (_, class) = Self::carrier_handle_disposition(value)?;
        // `D7` -- the carried occurrence is the authority whenever the template
        // has one, because it names the PRODUCER. `origin` names wherever the
        // template happened to be transferred, which after nested producer
        // traversal is a `Let`, `Match`, `Call` or `Effect` occurrence that
        // never built an aggregate at all.
        // ⭐ **The PRODUCER's occurrence, whichever aggregate shape this is.**
        // Both variants now carry one, so this reads the same answer for a
        // record as for a constructor rather than having a shape-shaped hole
        // that fell through to the use coordinate.
        let occurrence = match value.source_aggregate_producer() {
            Some(occurrence) => occurrence,
            // ⚠ Reached only by an aggregate with NO producer — a value-domain
            // record or constructor built from a `RuntimeValue`, which has no
            // occurrence in the program. `source_aggregate_occurrence` then
            // fails closed unless the transfer coordinate genuinely is a
            // producer, which for those is the rig that built them.
            None => self
                .static_transition_plan
                .source_aggregate_occurrence(origin, shape)?,
        };
        Ok((occurrence, class))
    }

    fn carrier_handle_disposition(
        value: &Lowered,
    ) -> Result<(BoundaryTag, BoundaryClass), CraneliftBackendError> {
        match value.boundary_disposition() {
            BoundaryDisposition::RepresentedHandle { tag, class } => Ok((tag, class)),
            // ⚠ Not dead defensive code: it fires if a variant's disposition is
            // ever retuned from handle to immediate while this arm still
            // allocates. The disposition is the authority, so the producer must
            // fail rather than out-vote it.
            BoundaryDisposition::RepresentedImmediate { .. } => Err(unsupported(
                lowered_value_kind(value),
                "the producer would allocate a handle for a value the sole \
                 disposition authority represents as an immediate",
            )),
            BoundaryDisposition::ProtocolOnly { why }
            | BoundaryDisposition::FailClosedForbidden { why } => {
                Err(unsupported(lowered_value_kind(value), why))
            }
        }
    }

    /// The tag of a **spill-free immediate**, read from the sole disposition
    /// authority.
    ///
    /// ⛔ **`spill: Some(_)` is still refused HERE, and that is not a leftover.**
    /// The refusal did not become unnecessary when the dispatch landed — it
    /// moved. A spillable payload has two possible representations, so a caller
    /// asking this question about one is asking a question with two answers;
    /// [`Self::carrier_spillable_disposition`] is the one that returns both, and
    /// this arm is what stops a spillable value reaching a bare `make_immediate`
    /// through an arm that forgot. ⚠ Deleting it would not reintroduce a
    /// truncation *today* — every spillable arm routes to the dispatch — which
    /// is exactly why it must stay: the next `RepresentedImmediate` variant is
    /// added by someone who copies the `Bool` arm.
    fn carrier_immediate_tag(value: &Lowered) -> Result<BoundaryTag, CraneliftBackendError> {
        match value.boundary_disposition() {
            BoundaryDisposition::RepresentedImmediate { tag, spill: None } => Ok(tag),
            BoundaryDisposition::RepresentedImmediate { spill: Some(_), .. } => Err(unsupported(
                lowered_value_kind(value),
                "a spillable immediate needs the runtime magnitude dispatch, \
                 not a single `make_immediate`",
            )),
            BoundaryDisposition::RepresentedHandle { .. } => Err(unsupported(
                lowered_value_kind(value),
                "the producer would mint an immediate for a value the sole \
                 disposition authority represents as a handle",
            )),
            BoundaryDisposition::ProtocolOnly { why }
            | BoundaryDisposition::FailClosedForbidden { why } => {
                Err(unsupported(lowered_value_kind(value), why))
            }
        }
    }

    /// The `(immediate tag, spill class)` of a **spillable** immediate, read
    /// from the sole disposition authority (`§2h` ¶4).
    ///
    /// ⛔ **`spill: None` is refused, and the refusal is the mirror of the one
    /// on [`Self::carrier_immediate_tag`].** Between them the two readers
    /// partition `RepresentedImmediate` on the `spill` field, so neither the
    /// dispatch nor the single-`make_immediate` path can be reached for a value
    /// the authority classified the other way — and a value with **no** reader
    /// is a compile error at the `match` in `emit_carrier_transfer`, not a
    /// silent default.
    fn carrier_spillable_disposition(
        value: &Lowered,
    ) -> Result<(BoundaryTag, BoundaryClass), CraneliftBackendError> {
        match value.boundary_disposition() {
            BoundaryDisposition::RepresentedImmediate {
                tag,
                spill: Some(class),
            } => Ok((tag, class)),
            BoundaryDisposition::RepresentedImmediate { spill: None, .. } => Err(unsupported(
                lowered_value_kind(value),
                "the producer would emit a magnitude dispatch for a value the \
                 sole disposition authority declares cannot overflow its field",
            )),
            BoundaryDisposition::RepresentedHandle { .. } => Err(unsupported(
                lowered_value_kind(value),
                "the producer would mint an immediate for a value the sole \
                 disposition authority represents as a handle",
            )),
            BoundaryDisposition::ProtocolOnly { why }
            | BoundaryDisposition::FailClosedForbidden { why } => {
                Err(unsupported(lowered_value_kind(value), why))
            }
        }
    }

    /// The carrier helpers, as refs callable inside **this** generated function.
    fn carrier_refs(&self) -> Result<BoundaryCarrierRefs, CraneliftBackendError> {
        self.function_local.boundary_carrier.ok_or_else(|| {
            unsupported(
                "BoundaryCarrier",
                "this generated function has no boundary-carrier helper refs",
            )
        })
    }

    /// The **boundary** arena the carrier helpers take as their first argument.
    ///
    /// ⛔⛔ **A CLAIM THAT STOOD HERE WAS FALSE, and it is retracted rather than
    /// deleted.** The retracted text asserted that the boundary arena and the
    /// native-`Int` arena *"are the same SSA value, and that is a fact about the
    /// ABI rather than a shortcut."* They are not — `CompiledModule::run` passed
    /// a **`NativeIntArenaV1`** as parameter 0, and in process mode the field was
    /// re-read from `invocation[24]`, which is the native arena again. ⇒ Every
    /// boundary-carrier helper reached through here was handed a native arena.
    /// ⚠ **It never fired only because the carrier was inert** — which is
    /// exactly why it must not be discovered by `S6` making it live.
    ///
    /// ⭐ **Repaired under the Architect's ruling (relayed `evt_e300y2kjeb6k`):**
    /// one runtime-owned [`crate::activation_services::GeneratedActivationServicesV1`]
    /// with **distinct typed fields**, a uniform `(frame_ptr, services_ptr) -> i64`
    /// signature for the root and every unit, `FunctionLocalRefs` split to match,
    /// and this accessor returning **only** `boundary_arena`. ⛔ Not a second
    /// answer to one question — two different questions that were wrongly merged
    /// into one.
    ///
    /// ⚠ A reader who met the false claim and believed it needs to see that it
    /// was withdrawn, and the next person to ask *"why two arena fields?"* needs
    /// the reason here.
    fn carrier_arena(&self) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        self.function_local.boundary_arena.ok_or_else(|| {
            unsupported(
                "BoundaryCarrier",
                "this generated function has no boundary arena: the activation \
                 -services record that sources it is `S6`/`D6` reland work, and \
                 substituting the native-`Int` arena is the defect that ruling \
                 exists to remove",
            )
        })
    }

    /// ⭐ **THE ONE SIGNEDNESS VIEW over `pack_identity`'s `u64`.**
    ///
    /// The planner's `pack_identity` is *"the ONE injective encoding"* and it
    /// yields a `u64`; Cranelift's `iconst` takes an `i64`. ⛔ This bit-preserving
    /// reinterpretation is a **view over that one encoding, never a sibling
    /// beside it** — a second packing spelled at a call site is precisely how
    /// `D2`'s single authority would be undone at the bridge rather than at the
    /// source.
    ///
    /// ⚠ It is deliberately spelled once, so there is exactly one `as i64` to
    /// review and exactly one to get wrong.
    fn carrier_identity_immediate(
        builder: &mut FunctionBuilder<'_>,
        identity: u64,
    ) -> cranelift_codegen::ir::Value {
        builder.ins().iconst(types::I64, identity as i64)
    }

    /// A one-word output slot plus its address, for a helper's `out` parameter.
    fn carrier_out_slot(
        builder: &mut FunctionBuilder<'_>,
        pointer_type: types::Type,
    ) -> (
        cranelift_codegen::ir::StackSlot,
        cranelift_codegen::ir::Value,
    ) {
        let slot =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 8, 3));
        let address = builder.ins().stack_addr(pointer_type, slot, 0);
        (slot, address)
    }

    /// `alloc(arena, tag, class, field_count, out) -> status`.
    /// Open a fresh local event set for the body about to be emitted.
    ///
    /// ⛔ Called at the START of a body, before any allocation, so an
    /// allocation cannot be attributed to whichever body happened to be open
    /// last. A missing open is a loud failure at the first allocation, not a
    /// silently unattributed event.
    fn open_aggregate_events(
        &mut self,
        function: cranelift_module::FuncId,
    ) -> Result<(), CraneliftBackendError> {
        self.defining_function_id = Some(function);
        match self.aggregate_allocations.as_mut() {
            Some(ledger) => ledger.open(function),
            // Outside the emission pass there is no relation to open into.
            None => Ok(()),
        }
    }

    /// Commit the open body's pairs, after finalization and verification and
    /// **before** `define_function`.
    /// ⭐ **`D7` — the effect-seat body close runs HERE, and this is the only
    /// place it can.** All four emitters reach this one boundary after
    /// finalization and verification and before `define_function`; a close
    /// installed in any single emitter would leave the other three ungated, and
    /// one installed at the whole-pass closeout would notice a discarded visit
    /// only after its body was already in the module.
    ///
    /// ⛔ `defining_function_id` is cleared only after BOTH closes succeed. It
    /// is the body the closes are asked about, so clearing it first would make
    /// the question unaskable at exactly the moment it is due.
    fn commit_aggregate_events(&mut self) -> Result<(), CraneliftBackendError> {
        if let Some(function) = self.defining_function_id {
            if let Some(ledger) = self.host_effect_seats.as_mut() {
                ledger.commit_body(function)?;
            }
        }
        let committed = match self.aggregate_allocations.as_mut() {
            Some(ledger) => ledger.commit(),
            None => Ok(()),
        };
        committed?;
        self.defining_function_id = None;
        Ok(())
    }

    /// **`D7` — ergonomic sugar over a `PlannedAggregate` request.**
    ///
    /// ⚠ **The governance no longer lives here.** The class/shape agreement,
    /// the lane derivation and the event-and-relation recording all happen at
    /// [`Self::emit_carrier_alloc`], because a check that lives in a wrapper
    /// holds only as long as every future caller remembers to reach for the
    /// wrapper rather than the raw helper beside it — an obligation nothing
    /// enforced and nothing measured. This function exists so the construction
    /// seats read the way they did.
    fn emit_checked_aggregate_alloc(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        site: GovernedAllocationSite,
        occurrence: AggregateOccurrenceId,
        shape: PlannedAggregateShape,
        class: BoundaryClass,
        field_count: usize,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let request = Self::governed_request(site, occurrence, shape);
        self.emit_carrier_alloc(builder, request, class, field_count)
    }

    /// The request one governed site hands the choke.
    ///
    /// ⛔ In a shipped compiler this is the `PlannedAggregate` construction and
    /// nothing else — the `#[cfg(test)]` arm compiles out entirely. It exists
    /// so a control can perturb ONE named site's request, which is the only way
    /// to show that site reaches the choke GOVERNED. Asserting the choke's
    /// refusal on a hand-built request proves the choke; it says nothing about
    /// whether the emitter's four real sites arrive there.
    fn governed_request(
        site: GovernedAllocationSite,
        occurrence: AggregateOccurrenceId,
        shape: PlannedAggregateShape,
    ) -> CarrierAllocationRequest {
        #[cfg(test)]
        if GOVERNED_ALLOCATION_MUTATION.with(std::cell::Cell::get)
            == GovernedAllocationMutation::Bypass(site)
        {
            governed_allocation_hit();
            return CarrierAllocationRequest::NonAggregate {
                tag: BoundaryTag::PersistentGround,
            };
        }
        let _ = site;
        CarrierAllocationRequest::PlannedAggregate { occurrence, shape }
    }

    /// Enter one governed allocation into `E`, then into `R`.
    ///
    /// ⚠ Outside the emission pass there is no relation and no declared
    /// function, so there is no event to record. That is not a bypass: a bare
    /// rig emits no artifact, and the relation's laws are about an artifact's
    /// bodies. Inside the pass the `FuncId` is REQUIRED, and its absence is a
    /// loud failure rather than an unattributed event.
    fn record_governed_allocation(
        &mut self,
        result: cranelift_codegen::ir::Value,
        occurrence: AggregateOccurrenceId,
    ) -> Result<(), CraneliftBackendError> {
        let function = self.defining_function_id;
        let Some(ledger) = self.aggregate_allocations.as_mut() else {
            return Ok(());
        };
        let function = function.ok_or_else(|| {
            backend_module(
                "a governed aggregate allocation ran inside the emission pass with no declared \
                 function open, so its event has no FuncId to be scoped by"
                    .to_string(),
            )
        })?;
        // ⛔ Event evidence FIRST, then the relation pair. `E` is what
        // allocated; deriving it from the relation would make `dom(R) = E` true
        // by construction and the law unstateable.
        ledger.record_event(function, result)?;
        ledger.relate(function, result, occurrence)
    }

    /// **`D7` — open the claim group for one visit to one effect occurrence.**
    ///
    /// ⛔ Called BEFORE any seat of the occurrence is observed, and after every
    /// operand has been lowered — an operand's own lowering may itself visit a
    /// nested effect, and a group open across that would take the nested
    /// visit's claims.
    ///
    /// ⚠ `None` outside the emission pass. A bare rig defines no function, so
    /// there is no body for a visit to belong to; the per-seat `Avail`
    /// membership below still runs there, because that is a property of the
    /// seat rather than of the ledger.
    fn open_host_effect_seat_group(
        &mut self,
        effect_origin: StaticOriginId,
        operation: ken_host::HostOpV1,
    ) -> Result<Option<EffectSeatGroupId>, CraneliftBackendError> {
        let planned = self
            .static_transition_plan
            .host_effect_seat_slots(effect_origin);
        let function = self.defining_function_id;
        let Some(ledger) = self.host_effect_seats.as_mut() else {
            return Ok(None);
        };
        let function = function.ok_or_else(|| {
            backend_module(
                "a host effect occurrence was visited inside the emission pass with no declared \
                 function open, so its claim group has no body to be scoped by"
                    .to_string(),
            )
        })?;
        ledger
            .open_group(function, effect_origin, operation, planned)
            .map(Some)
    }

    /// **`D7` — claim the ONE planned record for a seat, in the phase the
    /// operand is ACTUALLY in.**
    ///
    /// ⭐ **This is where `Need ⊆ Avail` is asked.** The need and the
    /// availability are the planner's, derived from the operation and the slot
    /// with no reference to any representation; the phase is read off the
    /// operand in hand, and cannot be reverse-derived from a child occurrence or
    /// an ABI result. A seat that fails the membership is refused as that exact
    /// seat of that exact operation — not as a generic specialized-only surface,
    /// which is the whole point of the record.
    ///
    /// ⚠ The returned record is bound to `operand` by construction: the phase it
    /// was proved against was read from that operand and no other. Binding it to
    /// the operation-specific ARM that performs the read needs the arms to take
    /// the claim in place of the bulk conversion, which is the next release —
    /// today the claim is made and the arms still read the bulk vector.
    fn claim_host_effect_seat(
        &mut self,
        group: Option<EffectSeatGroupId>,
        effect_origin: StaticOriginId,
        slot: EffectSeatSlot,
        operand: &LoweringOperand,
    ) -> Result<PlannedEffectSeat, CraneliftBackendError> {
        let record = self
            .static_transition_plan
            .host_effect_seat(effect_origin, slot)?;
        let observed = operand.effect_seat_phase();
        #[cfg(test)]
        let observed = match effect_seat_visit_mutation() {
            EffectSeatVisitMutation::PerturbObservedPhase => match observed {
                EffectSeatPhase::SpecializedTemplate => EffectSeatPhase::CarriedWord,
                EffectSeatPhase::CarriedWord => EffectSeatPhase::SpecializedTemplate,
            },
            _ => observed,
        };
        if !record.avail.admits(observed) {
            return Err(unsupported(
                "Effect",
                format!(
                    "seat {slot:?} of {:?} needs {:?}, which it cannot observe in {observed:?}",
                    record.operation, record.need
                ),
            ));
        }
        let Some(group) = group else {
            return Ok(record);
        };
        let Some(ledger) = self.host_effect_seats.as_mut() else {
            return Ok(record);
        };
        ledger.claim(group, record, observed)?;
        Ok(record)
    }

    /// **`D7` — close the visit, before host dispatch or any successful exit.**
    fn close_host_effect_seat_group(
        &mut self,
        group: Option<EffectSeatGroupId>,
    ) -> Result<(), CraneliftBackendError> {
        let Some(group) = group else {
            return Ok(());
        };
        let Some(ledger) = self.host_effect_seats.as_mut() else {
            return Ok(());
        };
        #[cfg(test)]
        if effect_seat_visit_mutation() == EffectSeatVisitMutation::DiscardGroup {
            ledger.discard_open_group_for_tests();
            return Ok(());
        }
        ledger.close_group(group)
    }

    /// **`D7` — THE choke point. Every carrier allocation in the backend is
    /// this call, and the REQUEST decides whether it is governed.**
    ///
    /// ⭐ The two arms are not two spellings of one thing:
    ///
    /// | request | class | lane | evidence |
    /// |---|---|---|---|
    /// | `NonAggregate` | must NOT be `Constructor`/`Record` | the caller's tag | none |
    /// | `PlannedAggregate` | must MATCH the shape | `aggregate_allocation_at` | `E` then `R` |
    ///
    /// ⛔ Both refusals happen **before the raw `alloc` call**, so a bypass
    /// cannot allocate and then fail: the arena's allocation count does not
    /// move and no artifact is ever defined. Refusing afterwards would leave
    /// the very half-governed state the request exists to make unspellable.
    ///
    /// ⛔ The event, by contrast, is recorded AFTER the raw allocation,
    /// because the result `Value` is half the event's identity and does not
    /// exist before it. That ordering is what makes "one allocation, one pair"
    /// checkable at all.
    fn emit_carrier_alloc(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        request: CarrierAllocationRequest,
        class: BoundaryClass,
        field_count: usize,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        // ── The request is settled before a single instruction is emitted ──
        let tag = match request {
            CarrierAllocationRequest::NonAggregate { tag } => {
                if matches!(class, BoundaryClass::Constructor | BoundaryClass::Record) {
                    return Err(backend_module(format!(
                        "a {class:?} carrier was allocated as non-aggregate, so it would name no \
                         planned ownership record and enter neither E nor R"
                    )));
                }
                tag
            }
            CarrierAllocationRequest::PlannedAggregate { occurrence, shape } => {
                let expected = CarrierAllocationRequest::aggregate_class(shape);
                if class != expected {
                    return Err(backend_module(format!(
                        "a planned {shape:?} aggregate was allocated at class {class:?} rather \
                         than {expected:?}"
                    )));
                }
                // ⛔ The LANE is read from the record, never from the caller.
                match self
                    .static_transition_plan
                    .aggregate_allocation_at(occurrence, shape)?
                {
                    PlannedAggregateAllocation::PersistentGround => BoundaryTag::PersistentGround,
                    PlannedAggregateAllocation::InvocationAggregate => {
                        BoundaryTag::InvocationAggregate
                    }
                }
            }
        };
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let tag = builder.ins().iconst(types::I64, i64::from(tag as u8));
        let class = builder.ins().iconst(types::I64, class as i64);
        let count = builder.ins().iconst(
            types::I64,
            i64::try_from(field_count).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a transferred aggregate has more fields than the ABI can name",
                )
            })?,
        );
        #[cfg(test)]
        CARRIER_RAW_ALLOCATIONS.with(|n| n.set(n.get().saturating_add(1)));
        let call = builder
            .ins()
            .call(refs.alloc, &[arena, tag, class, count, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        let word = CarriedBoundaryWord {
            word: builder.ins().stack_load(types::I64, slot, 0),
        };
        if let CarrierAllocationRequest::PlannedAggregate { occurrence, .. } = request {
            self.record_governed_allocation(word.word, occurrence)?;
        }
        Ok(word)
    }

    /// `make_immediate(tag, payload, out) -> status`. ⚠ No arena: an immediate
    /// names no referent.
    fn emit_carrier_immediate(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        tag: BoundaryTag,
        payload: cranelift_codegen::ir::Value,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let tag = builder.ins().iconst(types::I64, i64::from(tag as u8));
        let call = builder
            .ins()
            .call(refs.make_immediate, &[tag, payload, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(CarriedBoundaryWord {
            word: builder.ins().stack_load(types::I64, slot, 0),
        })
    }

    /// ⭐⭐ **THE MAGNITUDE DISPATCH** — the producer arm for a value whose
    /// disposition carries `spill: Some(_)` (`RT-FNSPLIT-B2F` `D9`; Architect
    /// ruling on the corrected producer partition).
    ///
    /// ⛔⛔ **The predicate is READ, never re-derived.**
    /// `ken_boundary_make_immediate_local` already tests the payload against the
    /// one `BOUNDARY_IMMEDIATE_DOMAIN` table and already reports the answer
    /// distinguishably — its own source says the errors are kept distinct *"so a
    /// control can tell which rule refused without reading the payload back"*.
    /// ⇒ A shift-and-compare here would be a **second answer to a question that
    /// already has one**, free to drift from the table silently. That is the
    /// second-representation-authority defect one layer down, and it is the same
    /// objection [`Self::carrier_identity_immediate`] raises about `pack_identity`.
    ///
    /// ⭐ **Three outcomes, ⛔ not two:**
    ///
    /// | status | outcome |
    /// |---|---|
    /// | `BOUNDARY_OK` | the immediate word `make_immediate` wrote |
    /// | `BOUNDARY_ERR_BOUNDS` | **the spill** — a handle of the declared class |
    /// | anything else | fail closed, via the same `require_i64` every other helper status takes |
    ///
    /// ⛔ Collapsing *"anything else"* into the spill would turn a shape, tag or
    /// capacity error into a **silent allocation** of a value nobody asked for.
    /// The third outcome is spelled as `require_i64(status, BOUNDARY_ERR_BOUNDS)`
    /// on the not-OK edge precisely so it cannot be written as a two-way branch
    /// by accident.
    ///
    /// ⭐ **`AC-2` — this is emitted code reading a RUNTIME value.** Nothing here
    /// inspects a JIT-time constant to choose a layout: one compiled body takes
    /// either arm depending on the payload it is handed. That is why the
    /// partition is a property of the value rather than of the compilation.
    ///
    /// ⛔⛔ **THIS ARM IS ONLY SOUND FOR A `Small`-MARKED PAYLOAD, and it is
    /// [`Self::emit_carrier_native_int`]'s job to guarantee that.** The payload
    /// of a `NativeIntV1` pair means different things under different markers —
    /// a `Big` payload is a **slot identity**, and asking `make_immediate` a
    /// magnitude question about a slot number is answered `OK` for a low slot.
    /// ⇒ Calling this directly on an unpartitioned `Lowered::Int` payload is a
    /// **silent corruption**, not a fail-closed gap.
    ///
    /// ⚠ An earlier revision of this comment claimed such a value would be
    /// refused by `store_int_tag`'s owner guard. **It never reaches that guard**
    /// — corrected under the Architect's ruling `evt_79xcj70p0qxjj`.
    ///
    /// **MEASURED:** the emitted body branches on `make_immediate`'s status and
    /// builds a `BoundaryClass::Int` handle on the bounds edge.
    /// **CLAIMED:** a `Small`-marked spillable value crosses without truncation.
    /// **THE GAP:** ⚠ **the marker partition is the caller's**, so this
    /// function's soundness is conditional on it. The non-`Int` spillables reach
    /// here directly because their payload *is* their magnitude with no pair and
    /// no second reading — see [`Self::carrier_small_marker`].
    ///
    /// ⚠ **A second residual, review-caught rather than mechanically detected:**
    /// swapping the status branch below for a hand-written magnitude test still
    /// round-trips every value, so no test in this suite would redden. ⛔ Its
    /// absence from a green run is not evidence about it.
    fn emit_carrier_spillable_immediate(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        tag: BoundaryTag,
        spill: BoundaryClass,
        payload: cranelift_codegen::ir::Value,
        native_marker: cranelift_codegen::ir::Value,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let immediate_tag = builder.ins().iconst(types::I64, i64::from(tag as u8));
        let call = builder
            .ins()
            .call(refs.make_immediate, &[immediate_tag, payload, out]);
        let status = builder.inst_results(call)[0];

        // ⛔ The ONE comparison this function makes, and it is against a status,
        // not against a magnitude.
        let fits = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            status,
            BOUNDARY_OK,
        );
        let immediate_block = builder.create_block();
        let spill_block = builder.create_block();
        let join = builder.create_block();
        builder.append_block_param(join, types::I64);
        builder
            .ins()
            .brif(fits, immediate_block, &[], spill_block, &[]);

        builder.switch_to_block(immediate_block);
        let word = builder.ins().stack_load(types::I64, slot, 0);
        builder.ins().jump(join, &[word.into()]);

        builder.switch_to_block(spill_block);
        // ⭐ The third outcome, spelled as a requirement rather than an `else`:
        // reaching here means the status was not `OK`, and anything that is also
        // not `ERR_BOUNDS` leaves the function fail-closed right here.
        Self::require_i64(builder, status, BOUNDARY_ERR_BOUNDS);
        // ⚠ `require_i64` splits the block; from here the builder is in its
        // `valid` successor, which is where the allocation belongs.
        let spilled = self.emit_carrier_alloc(
            builder,
            CarrierAllocationRequest::NonAggregate {
                tag: BoundaryTag::PersistentGround,
            },
            spill,
            0,
        )?;
        let store = builder
            .ins()
            .call(refs.store_scalar, &[arena, spilled.word, payload]);
        Self::require_i64(builder, builder.inst_results(store)[0], BOUNDARY_OK);
        let mark = builder
            .ins()
            .call(refs.store_int_tag, &[arena, spilled.word, native_marker]);
        Self::require_i64(builder, builder.inst_results(mark)[0], BOUNDARY_OK);
        builder.ins().jump(join, &[spilled.word.into()]);

        builder.switch_to_block(join);
        Ok(CarriedBoundaryWord {
            word: builder.block_params(join)[0],
        })
    }

    /// ⭐⭐ **THE `NativeIntV1` MARKER PARTITION** — the entry point for
    /// `Lowered::Int`, and the thing that must happen **before** any magnitude
    /// question is asked (Architect ruling, `evt_79xcj70p0qxjj`).
    ///
    /// ⛔⛔ **Why the marker comes first, and why the obvious order is a silent
    /// corruption rather than a residual.** `Lowered::Int`'s `value` is the
    /// **payload half of a `NativeIntV1` pair**, and what that word *means*
    /// depends on the marker: for `Small` it is the magnitude; for `Big` it is a
    /// **slot identity in the invocation's native arena**, and slots begin at
    /// `1`. ⇒ Calling `make_immediate` on a `Big` payload asks a magnitude
    /// question about a slot number — and a low slot **satisfies** the immediate
    /// domain, so the value crosses on the apparent-success arm encoded as the
    /// integer `1`. ⚠ Not a fail-closed gap: a wrong answer that looks like a
    /// right one.
    ///
    /// ⚠ **This corrects a residual I previously stated as fail-closed.** The
    /// earlier claim was that a `Big` would be refused by `store_int_tag`'s
    /// owner guard. It never reaches that guard.
    ///
    /// ⭐ **The branch is a read of the canonical transport tag, ⛔ not a
    /// sibling magnitude predicate**, so it does not weaken the ban on
    /// re-deriving the immediate-domain test: within the `Small` arm the ruled
    /// status-derived dispatch is unchanged.
    ///
    /// | marker | path |
    /// |---|---|
    /// | `NATIVE_INT_SMALL_TAG_V1` | the payload **is** the magnitude → [`Self::emit_carrier_spillable_immediate`] |
    /// | `NATIVE_INT_BIG_TAG_V1` | the payload is a slot → resolve, then an **owned deep copy** into the persistent region |
    /// | anything else | ⛔ fail closed |
    fn emit_carrier_native_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        tag: BoundaryTag,
        spill: BoundaryClass,
        payload: cranelift_codegen::ir::Value,
        marker: cranelift_codegen::ir::Value,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let small = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            marker,
            i64::try_from(crate::NATIVE_INT_SMALL_TAG_V1).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "the native `Small` marker is not an ABI word",
                )
            })?,
        );
        let small_block = builder.create_block();
        let wide_block = builder.create_block();
        let join = builder.create_block();
        builder.append_block_param(join, types::I64);
        builder.ins().brif(small, small_block, &[], wide_block, &[]);

        builder.switch_to_block(small_block);
        let immediate =
            self.emit_carrier_spillable_immediate(builder, tag, spill, payload, marker)?;
        builder.ins().jump(join, &[immediate.word.into()]);

        builder.switch_to_block(wide_block);
        let wide = self.emit_carrier_region_limbed_int(builder, spill, payload, marker)?;
        builder.ins().jump(join, &[wide.word.into()]);

        builder.switch_to_block(join);
        Ok(CarriedBoundaryWord {
            word: builder.block_params(join)[0],
        })
    }

    /// ⭐ **The owned deep copy** — a region-limbed `Int` crossing into the
    /// persistent region (Architect ruling, `evt_79xcj70p0qxjj`).
    ///
    /// ⛔ **No represented-unavailable lane, and no new error identity.** A valid
    /// wide `Int` crosses **successfully**; `ERR_ESCAPE` is not an admissible
    /// terminal result for one. The copy is *owned*, so nothing borrows the
    /// invocation arena past its extent and the escape question does not arise.
    ///
    /// ⭐ **The decode is `ken_native_int_resolve_local`'s, never ours.** It
    /// already yields canonical `sign`, `len` and `limbs` from the one native
    /// representation. ⛔ Deriving them here would be a second exact-integer
    /// decoder beside the first — the proliferation `docs/PRINCIPLES.md` forbids
    /// — and `boundary_value_clif`'s own int readers make the identical choice.
    ///
    /// ⛔ **The order is load-bearing and is the established wide-`Int`
    /// producer's:** allocate → region marker → claim → copy → **seal**. The
    /// marker written is [`BOUNDARY_INT_REGION_LIMBS`], ⛔ never the native
    /// `Big` marker: that marker names a slot in storage that dies with the
    /// invocation, which is exactly what `BOUNDARY_INT_MARKER_OWNER` refuses on
    /// a persistent node. And until `seal_int` succeeds **the node denotes
    /// nothing**, so the seal is the last step rather than an optional check.
    ///
    /// ⚠ The limb loop is over a **runtime** length: nothing about the magnitude
    /// is known when this body is compiled, which is `AC-2` at the wide arm.
    fn emit_carrier_region_limbed_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        spill: BoundaryClass,
        payload: cranelift_codegen::ir::Value,
        marker: cranelift_codegen::ir::Value,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        // ⛔ Any marker that is not `Big` fails closed HERE — the closed set is
        // `{Small, Big}` and `Small` was taken by the caller's branch.
        Self::require_i64(
            builder,
            marker,
            i64::try_from(crate::NATIVE_INT_BIG_TAG_V1).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "the native `Big` marker is not an ABI word",
                )
            })?,
        );

        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let decoder = self.function_local.native_int_resolve.ok_or_else(|| {
            unsupported(
                "BoundaryCarrier",
                "this generated function has no exact-Int decoder",
            )
        })?;
        let pointer_type = builder.func.dfg.value_type(arena);

        // ⭐⭐ **The native arena comes from the BOUNDARY arena's own binding
        // slot, and that choice is intrinsic rather than convenient.** The node
        // being built is read back by `int_sign` / `int_len` / `int_limb`, and
        // each of those decodes with exactly `load(arena, ARENA_NATIVE_INT)`.
        // ⇒ Reading the same slot makes producer and consumer agree **by
        // construction**; taking the pointer from anywhere else would let the
        // two decode a pair against different arenas, which is the drift the
        // one-decoder rule exists to prevent.
        //
        // ⛔ Not native-arena layout: this is the boundary arena's binding
        // field, read exactly as `boundary_value_clif` reads it, and the value
        // is handed straight to the decoder rather than walked.
        let native_arena = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            arena,
            crate::boundary_value::ARENA_NATIVE_INT,
        );
        Self::require_nonzero(builder, native_arena);

        // The decoder's `{sign, len, limbs, small}` view.
        let view_slot =
            builder.create_sized_stack_slot(StackSlotData::new(StackSlotKind::ExplicitSlot, 32, 3));
        let view = builder.ins().stack_addr(pointer_type, view_slot, 0);
        let decoded = builder
            .ins()
            .call(decoder, &[native_arena, marker, payload, view]);
        Self::require_i64(builder, builder.inst_results(decoded)[0], 0);
        let sign = builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            view,
            crate::native_int_clif::VIEW_SIGN,
        );
        let length = builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            view,
            crate::native_int_clif::VIEW_LEN,
        );
        let source = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            view,
            crate::native_int_clif::VIEW_LIMBS,
        );

        // allocate → region marker → claim → copy → seal.
        let word = self.emit_carrier_alloc(
            builder,
            CarrierAllocationRequest::NonAggregate {
                tag: BoundaryTag::PersistentGround,
            },
            spill,
            0,
        )?;
        let region = builder.ins().iconst(
            types::I64,
            i64::try_from(crate::boundary_value::BOUNDARY_INT_REGION_LIMBS).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "the region-limbs marker is not an ABI word",
                )
            })?,
        );
        let marked = builder
            .ins()
            .call(refs.store_int_tag, &[arena, word.word, region]);
        Self::require_i64(builder, builder.inst_results(marked)[0], BOUNDARY_OK);
        let (_span_slot, span) = Self::carrier_out_slot(builder, pointer_type);
        let claim = builder.ins().call(
            refs.store_int_limbs,
            &[arena, word.word, sign, length, span],
        );
        Self::require_i64(builder, builder.inst_results(claim)[0], BOUNDARY_OK);

        let head = builder.create_block();
        builder.append_block_param(head, types::I64);
        let body = builder.create_block();
        let done = builder.create_block();
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().jump(head, &[zero.into()]);

        builder.switch_to_block(head);
        let index = builder.block_params(head)[0];
        let more = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThan,
            index,
            length,
        );
        builder.ins().brif(more, body, &[], done, &[]);

        builder.switch_to_block(body);
        let offset = builder.ins().imul_imm(index, 8);
        let address = builder.ins().iadd(source, offset);
        let limb = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), address, 0);
        let write = builder
            .ins()
            .call(refs.store_int_limb, &[arena, word.word, index, limb]);
        Self::require_i64(builder, builder.inst_results(write)[0], BOUNDARY_OK);
        // ⚠ `require_i64` split the block, so the back edge is emitted from the
        // block the builder is in NOW, not from `body`.
        let next = builder.ins().iadd_imm(index, 1);
        builder.ins().jump(head, &[next.into()]);

        builder.switch_to_block(done);
        let sealed = builder.ins().call(refs.seal_int, &[arena, word.word]);
        Self::require_i64(builder, builder.inst_results(sealed)[0], BOUNDARY_OK);
        Ok(word)
    }

    /// ⭐ **The byte-bodied handle producer** — the `String` / `Bytes` arm of
    /// `RT-FNSPLIT-B2F` `D9`.
    ///
    /// ⭐ **ONE body, driven with the class the disposition supplies.** ⛔ Not
    /// two emitters and ⛔ not a `Bytes` emitter a `String` "shares every code
    /// path but the class" with — the class is exactly the axis `store_bytes_len`
    /// and `store_byte` guard on, so it is the one path the two do **not** share.
    /// `boundary_value_clif`'s own history records a `class_guard` narrowed to
    /// `Bytes` alone staying green because no test had ever asked emitted code to
    /// *build* a `String`.
    ///
    /// ⭐ **Claim-then-fill.** `store_bytes_len` reserves the whole span before a
    /// byte is written, so a length the region cannot satisfy fails **before any
    /// address is formed** rather than part-way through the content.
    ///
    /// **MEASURED:** the emitted body allocates a node of the declared class,
    /// claims a span of the literal's length, and writes every byte of it.
    /// **CLAIMED:** a byte-bodied literal crosses the boundary with its content.
    /// **THE GAP:** ⚠ the content is a **compile-time literal**, so this arm says
    /// nothing about a runtime-computed string — there is no `Lowered` variant
    /// that carries one, and when one exists it needs its own control. ⛔ Do not
    /// read this as coverage of the byte-bodied class in general.
    fn emit_carrier_bytes(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        tag: BoundaryTag,
        class: BoundaryClass,
        content: &[u8],
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let word = self.emit_carrier_alloc(
            builder,
            CarrierAllocationRequest::NonAggregate { tag },
            class,
            0,
        )?;
        let (_span_slot, span) = Self::carrier_out_slot(builder, pointer_type);
        let length = builder.ins().iconst(
            types::I64,
            i64::try_from(content.len()).map_err(|_| {
                unsupported(
                    "BoundaryCarrier",
                    "a transferred literal is longer than the ABI can name",
                )
            })?,
        );
        let claim = builder
            .ins()
            .call(refs.store_bytes_len, &[arena, word.word, length, span]);
        Self::require_i64(builder, builder.inst_results(claim)[0], BOUNDARY_OK);
        for (index, byte) in content.iter().enumerate() {
            let position = Self::carrier_position_immediate(builder, index)?;
            let byte = builder.ins().iconst(types::I64, i64::from(*byte));
            let write = builder
                .ins()
                .call(refs.store_byte, &[arena, word.word, position, byte]);
            Self::require_i64(builder, builder.inst_results(write)[0], BOUNDARY_OK);
        }
        Ok(word)
    }

    /// The `NativeIntV1` marker for a spillable immediate whose magnitude **is**
    /// its payload word.
    ///
    /// ⭐ `ProcessExitStatus`, `BoundedNat` and `StructuralNat` are one native
    /// scalar each — there is no second word and no arena slot — so `Small` is
    /// not a default for them, it is the only true answer.
    /// ⛔ `Lowered::Int` does **not** come here: its marker is carried alongside
    /// the payload by [`Self::native_int_tag`], and substituting a constant would
    /// be the producer out-voting the native-`Int` representation.
    fn carrier_small_marker(builder: &mut FunctionBuilder<'_>) -> cranelift_codegen::ir::Value {
        builder
            .ins()
            .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64)
    }

    /// `store_tag_id(arena, word, tag_id) -> status`.
    fn emit_carrier_store_tag_id(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
        identity: u64,
    ) -> Result<(), CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let identity = Self::carrier_identity_immediate(builder, identity);
        let call = builder
            .ins()
            .call(refs.store_tag_id, &[arena, target.word, identity]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(())
    }

    /// `store_scalar(arena, word, value) -> status`.
    fn emit_carrier_store_scalar(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
        payload: cranelift_codegen::ir::Value,
    ) -> Result<(), CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let call = builder
            .ins()
            .call(refs.store_scalar, &[arena, target.word, payload]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(())
    }

    fn emit_carrier_dynamic_constructor(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        origin: StaticOriginId,
        dynamic: &DynamicConstructorV1,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        validate_dynamic_constructor_alternatives(
            dynamic
                .alternatives
                .iter()
                .map(|alternative| (alternative.tag, alternative.constructor.as_str())),
        )?;
        let merge = builder.create_block();
        builder.append_block_param(merge, types::I64);

        for alternative in &dynamic.alternatives {
            let selected = builder.create_block();
            let next = builder.create_block();
            let matches = builder.ins().icmp_imm(
                cranelift_codegen::ir::condcodes::IntCC::Equal,
                dynamic.discriminator,
                alternative.tag,
            );
            builder.ins().brif(matches, selected, &[], next, &[]);

            builder.switch_to_block(selected);
            let disposition = Lowered::DynamicConstructor(DynamicConstructorV1 {
                discriminator: dynamic.discriminator,
                alternatives: vec![alternative.clone()],
            });
            // ⭐ **`D7` — the selected alternative's lane comes from ITS OWN
            // planner record.** The set is not an allocation; this is. The
            // value-shape disposition answers `PersistentGround` for every
            // `DynamicConstructor` because the shape is persistable, which is
            // the same unproven persistent lane the fixed-constructor arm
            // already stopped taking. Whether this alternative may take it
            // depends on its children's lifetimes, which the value in hand does
            // not carry.
            //
            // ⚠ The CLASS still comes from the disposition and only the TAG is
            // replaced — the class is a fact about the shape and the
            // disposition is its authority; the lane is a fact about the meet
            // and the planner is its.
            let (_, class) = Self::carrier_handle_disposition(&disposition)?;
            let occurrence = match alternative.occurrence {
                Some(occurrence) => occurrence,
                // ⛔ A refusal, not a default. An alternative with no carried
                // occurrence is one whose lifetime meet was never taken, and
                // answering `PersistentGround` for it would reinstate exactly
                // the unproven lane the record exists to replace — silently,
                // and only for the alternatives the population happened to
                // miss.
                None => {
                    return Err(unsupported(
                        "DynamicConstructor",
                        format!(
                            "the selected alternative {} carries no planned occurrence, so its \
                             allocation has no lifetime meet",
                            alternative.constructor
                        ),
                    ));
                }
            };
            let word = self.emit_checked_aggregate_alloc(
                builder,
                GovernedAllocationSite::DynamicAlternative,
                occurrence,
                PlannedAggregateShape::Constructor,
                class,
                alternative.fields.len(),
            )?;
            self.emit_carrier_store_tag_id(builder, word, alternative.identity.tag_abi_word()?)?;
            for (position, field) in alternative.fields.iter().enumerate() {
                let field = self.emit_carrier_transfer(builder, origin, field)?;
                self.emit_carrier_store_field(builder, word, position, field)?;
            }
            builder.ins().jump(merge, &[word.word.into()]);
            builder.switch_to_block(next);
        }

        let malformed = builder
            .ins()
            .iconst(types::I64, MALFORMED_DYNAMIC_CONSTRUCTOR_STATUS);
        builder.ins().return_(&[malformed]);
        builder.switch_to_block(merge);
        Ok(CarriedBoundaryWord {
            word: builder.block_params(merge)[0],
        })
    }

    /// `store_field(arena, word, index, child) -> status`.
    fn emit_carrier_store_field(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
        position: usize,
        child: CarriedBoundaryWord,
    ) -> Result<(), CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let index = Self::carrier_position_immediate(builder, position)?;
        let call = builder
            .ins()
            .call(refs.store_field, &[arena, target.word, index, child.word]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(())
    }

    /// `store_name(arena, word, index, name_id) -> status`.
    fn emit_carrier_store_name(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
        position: usize,
        identity: u64,
    ) -> Result<(), CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let index = Self::carrier_position_immediate(builder, position)?;
        let identity = Self::carrier_identity_immediate(builder, identity);
        let call = builder
            .ins()
            .call(refs.store_name, &[arena, target.word, index, identity]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(())
    }

    // ── the CONSUMER half of the carrier ABI (`D3` / `D4`) ──────────────
    //
    // ⭐ **Every question these answer is answered AT RUNTIME, by a call.**
    // That is the whole content of the node: a carried value has no
    // compile-time template, so *"which constructor"*, *"how many children"*
    // and *"which child"* cannot be read off a struct field — there is no
    // struct field to read. ⛔ Adding one is how the wall grows back
    // (see [`CarriedBoundaryWord`]).
    //
    // ⚠ Each of these emits a `require_i64(status, BOUNDARY_OK)`, which
    // **splits the current block**. A caller assembling its own control flow
    // must therefore take `builder`'s *current* block after the call, never
    // the block it switched to before it.

    /// `tag(arena, word, out) -> status` — the runtime constructor identity.
    ///
    /// ⭐ The returned word is comparable **only** against a
    /// `ConstructorIdentity::tag_abi_word()` from the same artifact's plane —
    /// it is the very word the producer wrote with `store_tag_id`, and that
    /// shared authority is `D2`. ⛔ It is not an ordinal, not a `LoweredVariant`
    /// discriminant, and not portable across artifacts.
    fn emit_carrier_tag(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let call = builder.ins().call(refs.tag, &[arena, target.word, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(builder.ins().stack_load(types::I64, slot, 0))
    }

    fn emit_carrier_class(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let call = builder.ins().call(refs.class, &[arena, target.word, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(builder.ins().stack_load(types::I64, slot, 0))
    }

    fn emit_carrier_host_success(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let call = builder
            .ins()
            .call(refs.host_success, &[arena, target.word, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(builder.ins().stack_load(types::I64, slot, 0))
    }

    fn emit_carrier_host_payload(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let call = builder
            .ins()
            .call(refs.host_payload, &[arena, target.word, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(CarriedBoundaryWord {
            word: builder.ins().stack_load(types::I64, slot, 0),
        })
    }

    pub(super) fn emit_carrier_scalar(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let call = builder.ins().call(refs.scalar, &[arena, target.word, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(builder.ins().stack_load(types::I64, slot, 0))
    }

    /// Project the object entry's public scalar result.
    ///
    /// Immediate carrier words expose their payload through the ordinary
    /// scalar helper. A persistent exact `Int` instead owns a magnitude in the
    /// boundary region; export that magnitude through the native-`Int`
    /// interner/exporter pair so the object launcher observes the same decimal
    /// value as the direct lowering path.
    pub(super) fn emit_public_carrier_scalar(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let boundary_arena = self.carrier_arena()?;
        let native_arena = self.function_local.native_int_arena.ok_or_else(|| {
            unsupported("NativeResult", "carried Int result has no invocation arena")
        })?;
        let export_parts = self
            .function_local
            .native_int_export_parts
            .ok_or_else(|| unsupported("NativeResult", "carried Int has no export function"))?;
        let pointer_type = builder.func.dfg.value_type(boundary_arena);

        let tag = builder.ins().band_imm(
            target.word,
            crate::boundary_value::BOUNDARY_TAG_MASK as i64,
        );
        let persistent = builder.ins().icmp_imm(
            cranelift_codegen::ir::condcodes::IntCC::Equal,
            tag,
            crate::boundary_value::BoundaryTag::PersistentGround as i64,
        );
        let exact_int = builder.create_block();
        let immediate = builder.create_block();
        let done = builder.create_block();
        builder.append_block_param(done, types::I64);
        builder
            .ins()
            .brif(persistent, exact_int, &[], immediate, &[]);

        builder.switch_to_block(exact_int);
        let view_slot = builder.create_sized_stack_slot(
            cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                24,
                3,
            ),
        );
        let view = builder.ins().stack_addr(pointer_type, view_slot, 0);
        let call = builder
            .ins()
            .call(refs.int_view, &[boundary_arena, target.word, view]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        let sign = builder.ins().stack_load(types::I64, view_slot, 0);
        let len = builder.ins().stack_load(types::I64, view_slot, 8);
        let limbs = builder.ins().stack_load(pointer_type, view_slot, 16);
        let call = builder
            .ins()
            .call(export_parts, &[native_arena, sign, limbs, len]);
        Self::require_i64(builder, builder.inst_results(call)[0], 0);
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().jump(done, &[zero.into()]);

        builder.switch_to_block(immediate);
        let scalar = self.emit_carrier_scalar(builder, target)?;
        builder.ins().jump(done, &[scalar.into()]);

        builder.switch_to_block(done);
        Ok(builder.block_params(done)[0])
    }

    /// `field_count(arena, word, out) -> status` — the child count a case's
    /// binder arity is checked against **at runtime**.
    ///
    /// ⚠ **Why this is a runtime check and not a compile-time one.** For a
    /// specialized scrutinee the arity check compares `case.binders` against
    /// `args.len()`, both known while compiling. A carried word knows neither,
    /// so the same guard has to be emitted — and it must still *be* a guard:
    /// binding *n* binders over a value with fewer children would read past the
    /// node.
    fn emit_carrier_field_count(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let call = builder
            .ins()
            .call(refs.field_count, &[arena, target.word, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(builder.ins().stack_load(types::I64, slot, 0))
    }

    /// `field(arena, word, index, out) -> status` — positional child projection.
    ///
    /// ⭐⭐ **The result is a [`CarriedBoundaryWord`], and that return type is
    /// the `§2g` property, not a convenience.** *"Projected children remain
    /// `Carried`."* A signature returning [`Lowered`] here would be the
    /// forbidden inverse conversion wearing the name of a projection — so the
    /// prohibition is carried by the **type**, where it cannot be forgotten at a
    /// call site.
    fn emit_carrier_field(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
        position: usize,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let index = Self::carrier_position_immediate(builder, position)?;
        let call = builder
            .ins()
            .call(refs.field, &[arena, target.word, index, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(CarriedBoundaryWord {
            word: builder.ins().stack_load(types::I64, slot, 0),
        })
    }

    /// `record_field(arena, word, name_id, out) -> status` — `Project` by
    /// **artifact-static field identity**.
    ///
    /// ⭐ The `name_id` is the same word the producer wrote with `store_name`,
    /// from the same `D1` authority — which is exactly why `AC-C5`'s reordered
    /// record still projects correctly: the lookup is keyed on the interned
    /// name, ⛔ never on declaration position.
    ///
    /// ⭐ Result stays carried, for the reason spelled out on
    /// [`Self::emit_carrier_field`].
    fn emit_carrier_record_field(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
        identity: u64,
    ) -> Result<CarriedBoundaryWord, CraneliftBackendError> {
        let refs = self.carrier_refs()?;
        let arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(arena);
        let (slot, out) = Self::carrier_out_slot(builder, pointer_type);
        let identity = Self::carrier_identity_immediate(builder, identity);
        let call = builder
            .ins()
            .call(refs.record_field, &[arena, target.word, identity, out]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        Ok(CarriedBoundaryWord {
            word: builder.ins().stack_load(types::I64, slot, 0),
        })
    }

    /// A child ordinal as an ABI immediate.
    fn carrier_position_immediate(
        builder: &mut FunctionBuilder<'_>,
        position: usize,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let position = i64::try_from(position).map_err(|_| {
            unsupported(
                "BoundaryCarrier",
                "a transferred aggregate's child ordinal is outside the ABI's range",
            )
        })?;
        Ok(builder.ins().iconst(types::I64, position))
    }
}

/// ⛔ **The `Lowered` variant TAG, without a value.**
///
/// `D4`'s policies are claims about a **whole variant**, never about a sampled
/// value — the frame says so in as many words, because assigning *immediate-only*
/// to a variant that has a spill arm is the vacuity route `AC-10` exists to
/// close. A disposition that takes `&Lowered` cannot be swept over the variants
/// without constructing 21 values, and a control that samples one value per
/// variant would be asserting the variant-level claim from value-level evidence.
///
/// ⭐ So the disposition is a function of **this** — the tag alone — and the tag
/// set is enumerable. `Lowered::variant` and
/// `LoweredVariant::boundary_disposition` are both `match`es with **no `_`
/// arm**, so a 22nd `Lowered` variant is a compile error in both.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum LoweredVariant {
    Int,
    Bool,
    ProcessExitStatus,
    CapabilityToken,
    ResourceToken,
    BoundedNat,
    StructuralNat,
    ResponseBytes,
    HostResult,
    DynamicConstructor,
    Bytes,
    BorrowedNativeValue,
    BorrowedOption,
    String,
    Constructor,
    Record,
    Closure,
    DeclarationClosure,
    ComputationalRecursorClosure,
    RecursiveBackedge,
    Trap,
}

impl LoweredVariant {
    /// Every variant, in declaration order.
    pub(in crate::cranelift_backend) const ALL: [LoweredVariant; 21] = [
        LoweredVariant::Int,
        LoweredVariant::Bool,
        LoweredVariant::ProcessExitStatus,
        LoweredVariant::CapabilityToken,
        LoweredVariant::ResourceToken,
        LoweredVariant::BoundedNat,
        LoweredVariant::StructuralNat,
        LoweredVariant::ResponseBytes,
        LoweredVariant::HostResult,
        LoweredVariant::DynamicConstructor,
        LoweredVariant::Bytes,
        LoweredVariant::BorrowedNativeValue,
        LoweredVariant::BorrowedOption,
        LoweredVariant::String,
        LoweredVariant::Constructor,
        LoweredVariant::Record,
        LoweredVariant::Closure,
        LoweredVariant::DeclarationClosure,
        LoweredVariant::ComputationalRecursorClosure,
        LoweredVariant::RecursiveBackedge,
        LoweredVariant::Trap,
    ];
}

impl Lowered {
    /// This value's variant tag. ⛔ Exhaustive, no `_` arm.
    pub(in crate::cranelift_backend) fn variant(&self) -> LoweredVariant {
        match self {
            Lowered::Int { .. } => LoweredVariant::Int,
            Lowered::Bool { .. } => LoweredVariant::Bool,
            Lowered::ProcessExitStatus { .. } => LoweredVariant::ProcessExitStatus,
            Lowered::CapabilityToken { .. } => LoweredVariant::CapabilityToken,
            Lowered::ResourceToken { .. } => LoweredVariant::ResourceToken,
            Lowered::BoundedNat(_) => LoweredVariant::BoundedNat,
            Lowered::StructuralNat(_) => LoweredVariant::StructuralNat,
            Lowered::ResponseBytes { .. } => LoweredVariant::ResponseBytes,
            Lowered::HostResult { .. } => LoweredVariant::HostResult,
            Lowered::DynamicConstructor(_) => LoweredVariant::DynamicConstructor,
            Lowered::Bytes(_) => LoweredVariant::Bytes,
            Lowered::BorrowedNativeValue { .. } => LoweredVariant::BorrowedNativeValue,
            Lowered::BorrowedOption { .. } => LoweredVariant::BorrowedOption,
            Lowered::String(_) => LoweredVariant::String,
            Lowered::Constructor { .. } => LoweredVariant::Constructor,
            Lowered::Record { .. } => LoweredVariant::Record,
            Lowered::Closure { .. } => LoweredVariant::Closure,
            Lowered::DeclarationClosure { .. } => LoweredVariant::DeclarationClosure,
            Lowered::ComputationalRecursorClosure { .. } => {
                LoweredVariant::ComputationalRecursorClosure
            }
            Lowered::RecursiveBackedge => LoweredVariant::RecursiveBackedge,
            Lowered::Trap(_) => LoweredVariant::Trap,
        }
    }
}

/// `RT-FNSPLIT-B2V` `D4` — the five STATIC ENCODING POLICIES, as a closed set.
///
/// ⛔ **Five policies, and the type says five.** They were previously readable
/// only by inspecting a `BoundaryDisposition`: *immediate-only* and
/// *immediate-with-declared-handle-spill* are the same constructor distinguished
/// by an `Option` field, so "every variant has exactly one of five" was a
/// **reading** of the type rather than a fact about it. `AC-3` requires the
/// assignment, and a claim a type cannot express is a claim a control has to
/// restate — which is how the misassignment it names would survive.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum StaticEncodingPolicy {
    /// Every value encodes in the tagged word; **no spill arm exists**.
    ImmediateOnly,
    /// Every value encodes as an opaque handle, with explicit lifetime and
    /// referent owner.
    HandleOnly,
    /// Values encode immediate **or**, on a declared closed condition, as a
    /// handle carrying the same lifetime/referent-owner obligations.
    ImmediateWithDeclaredHandleSpill,
    /// Never a source value at a boundary.
    ProtocolOnly,
    /// Rejected before emission, with an exact error.
    FailClosedForbidden,
}

impl StaticEncodingPolicy {
    /// Every policy, in the frame's order.
    pub(in crate::cranelift_backend) const ALL: [StaticEncodingPolicy; 5] = [
        StaticEncodingPolicy::ImmediateOnly,
        StaticEncodingPolicy::HandleOnly,
        StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill,
        StaticEncodingPolicy::ProtocolOnly,
        StaticEncodingPolicy::FailClosedForbidden,
    ];
}

impl BoundaryDisposition {
    /// The static encoding policy this disposition declares.
    ///
    /// ⛔ **A declared spill is the THIRD policy, not the first.**
    /// `RepresentedImmediate { spill: Some(_) }` claims that a value encodes
    /// immediate *or* spills to a handle on a declared condition — it does not
    /// claim every value of the variant is immediate, and calling it
    /// *immediate-only* would let a proof attach handle evidence to one sampled
    /// spill while never establishing the handle obligations for the partition.
    pub(in crate::cranelift_backend) fn policy(self) -> StaticEncodingPolicy {
        match self {
            BoundaryDisposition::RepresentedImmediate { spill: None, .. } => {
                StaticEncodingPolicy::ImmediateOnly
            }
            BoundaryDisposition::RepresentedImmediate { spill: Some(_), .. } => {
                StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill
            }
            BoundaryDisposition::RepresentedHandle { .. } => StaticEncodingPolicy::HandleOnly,
            BoundaryDisposition::ProtocolOnly { .. } => StaticEncodingPolicy::ProtocolOnly,
            BoundaryDisposition::FailClosedForbidden { .. } => {
                StaticEncodingPolicy::FailClosedForbidden
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `AC-10` — total classified-domain closure
// ---------------------------------------------------------------------------
//
// ⛔ **"One control total over every value" is not an executable oracle**, and
// the frame says so: the admitted domains include unbounded integers, arbitrary
// byte contents, ownership states, and recursive parent → child reachability. A
// finite runtime sweep dressed as a universal claim is worse than an honest
// sweep, because it reads as total.
//
// ⭐ **So totality is proved STRUCTURALLY, in two layers.** The sealed
// wildcard-free disposition closes the *variant* layer. Below it, every
// **value-dependent representation discriminator** is a closed finite partition,
// and the classifier is a total function from a cell of that product to exactly
// one actual outcome. A value reaches its cell through a *total* projection
// (`int_fits_immediate`, `referent_owner`, "does this aggregate hold an
// invocation-owned child") — so the infinite domain is covered by construction
// rather than by enumeration, and only the finitely many CELLS need controls.

/// Magnitude / shape — the discriminator an immediate-with-spill policy names.
///
/// The projection from a value is total: `BoundaryWord::int_fits_immediate`
/// answers for every `i64`, and there is no third answer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum MagnitudePartition {
    /// The payload encodes in the tagged word's 56-bit field.
    WithinImmediateField,
    /// The payload does not, so a declared spill arm must carry it.
    BeyondImmediateField,
}

/// Parent → child reachability — the discriminator that decides whether an
/// aggregate can be represented at all.
///
/// ⛔ **Total over nodes is not closed under parent → child reachability**, which
/// is why this is its own partition rather than a property of the parent's
/// variant: a persistent aggregate holding an invocation-owned child is a
/// surviving parent naming storage that dies first.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum ReachabilityPartition {
    /// No children — nothing to reach.
    Leaf,
    /// Every reachable child outlives the parent.
    ChildrenOutliveParent,
    /// Some reachable child dies before the parent.
    ChildDiesBeforeParent,
}

/// Whether a handle's referent carries the store's identity of record.
///
/// ⛔ **`NoStoreIdentity` is NOT a valid outcome for a persistent handle**, and
/// classifying it as one was the defect the Architect ruled on. A consumer can
/// recover the *absence* of an identity; it cannot thereby recover the same
/// identity **intact**. Worse, this ABI's own node contract says a null
/// `NODE_SLOT` denotes *invocation-arena* ownership — so a word claiming
/// `PersistentStore` over a null slot contradicts the layout it is written in.
/// Reserving persistent-region storage is storage governance, never adoption.
///
/// ⭐ An emitted-constructed persistent node is therefore a **pending** internal
/// state, not a published outcome: [`crate::boundary_value::BoundaryValueStore::adopt`]
/// validates the reachable graph, interns it, and mints or reuses the real
/// `SlotId` before the word can escape. `NoStoreIdentity` remains correct for an
/// **invocation** handle, where there is no store identity to have.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum HandleIdentity {
    /// The store minted or reused the referent's `SlotId` and the node names it.
    StoreMinted,
    /// An invocation-owned referent, which has no store identity by design.
    NoStoreIdentity,
}

/// Whether a persistent node has passed the store-owned adoption boundary.
///
/// ⛔ A closed partition, and the one that decides whether a persistent handle is
/// **published at all**. Emitted construction alone leaves
/// `PendingStoreAdoption`; only the store's `adopt` moves a node to
/// `StoreAdopted`, and the emitted escape gate refuses to let a pending word
/// cross a generated-function boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum AdoptionPartition {
    /// The store has minted or reused this referent's `SlotId`.
    StoreAdopted,
    /// Constructed and sealed by emitted code, but not adopted by the store.
    PendingStoreAdoption,
}

/// The **actual outcome** a boundary input receives — the closed set `AC-10`
/// quantifies over.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum BoundaryOutcome {
    /// The value rides in the tagged word.
    ///
    /// ⛔ `value_class` is what the uniform `class` helper must report for such
    /// a word — a *boundary-value* classification, deliberately NOT a node
    /// class (an immediate has no node). See
    /// [`BoundaryTag::immediate_value_class`].
    ImmediateWord {
        tag: BoundaryTag,
        value_class: Option<BoundaryClass>,
    },
    /// A handle, with every obligation the frame names discharged: class,
    /// referent owner, identity, and lifetime (the owner *is* the lifetime).
    HandleWord {
        tag: BoundaryTag,
        class: BoundaryClass,
        owner: BoundaryReferentOwner,
        identity: HandleIdentity,
    },
    /// Never a source value at a boundary.
    ProtocolOnly,
    /// Rejected before emission or publication, with an exact status.
    FailClosedForbidden,
}

/// One cell of the closed discriminator product — a boundary **input**, reduced
/// to the finitely many things its representation can depend on.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) struct BoundaryInput {
    pub(in crate::cranelift_backend) variant: LoweredVariant,
    pub(in crate::cranelift_backend) magnitude: MagnitudePartition,
    pub(in crate::cranelift_backend) reachability: ReachabilityPartition,
    pub(in crate::cranelift_backend) adoption: AdoptionPartition,
}

impl BoundaryInput {
    /// Every cell of the product, in a fixed order.
    pub(in crate::cranelift_backend) fn all() -> Vec<BoundaryInput> {
        let mut cells = Vec::new();
        for variant in LoweredVariant::ALL {
            for magnitude in [
                MagnitudePartition::WithinImmediateField,
                MagnitudePartition::BeyondImmediateField,
            ] {
                for reachability in [
                    ReachabilityPartition::Leaf,
                    ReachabilityPartition::ChildrenOutliveParent,
                    ReachabilityPartition::ChildDiesBeforeParent,
                ] {
                    for adoption in [
                        AdoptionPartition::StoreAdopted,
                        AdoptionPartition::PendingStoreAdoption,
                    ] {
                        cells.push(BoundaryInput {
                            variant,
                            magnitude,
                            reachability,
                            adoption,
                        });
                    }
                }
            }
        }
        cells
    }

    /// The actual outcome this input receives.
    ///
    /// ⛔ **Classification happens FIRST and the behaviour is entailed by the
    /// class.** The failure arm belongs to the *unrepresentable* class, never
    /// inside the admitted one — a predicate reading *"either round-trip or fail
    /// closed"* over the admitted set is satisfied vacuously by an
    /// implementation that rejects everything.
    ///
    /// ⛔ **No `_` arm anywhere below**, so a new variant, a new policy, or a new
    /// partition value is a compile error rather than a silent default.
    pub(in crate::cranelift_backend) fn outcome(self) -> BoundaryOutcome {
        let disposition = self.variant.boundary_disposition();
        match disposition {
            BoundaryDisposition::ProtocolOnly { .. } => BoundaryOutcome::ProtocolOnly,
            BoundaryDisposition::FailClosedForbidden { .. } => BoundaryOutcome::FailClosedForbidden,
            BoundaryDisposition::RepresentedImmediate { tag, spill } => {
                match (spill, self.magnitude) {
                    // Immediate-only: the outcome does not depend on magnitude,
                    // and that constancy is asserted rather than assumed.
                    (None, MagnitudePartition::WithinImmediateField)
                    | (None, MagnitudePartition::BeyondImmediateField) => {
                        BoundaryOutcome::ImmediateWord {
                            tag,
                            value_class: tag.immediate_value_class(),
                        }
                    }
                    (Some(_), MagnitudePartition::WithinImmediateField) => {
                        BoundaryOutcome::ImmediateWord {
                            tag,
                            value_class: tag.immediate_value_class(),
                        }
                    }
                    // ⛔ **The SPILL ARM is a handle outcome**, so it discharges the
                    // same class / owner / identity / lifetime obligations as
                    // handle-only. This is the arm the frame says a proof may not
                    // attach to one sampled value.
                    // ⛔ The spill arm is a PERSISTENT handle, so it publishes
                    // only once the store owns its identity.
                    (Some(class), MagnitudePartition::BeyondImmediateField) => {
                        match self.adoption {
                            AdoptionPartition::PendingStoreAdoption => {
                                BoundaryOutcome::FailClosedForbidden
                            }
                            AdoptionPartition::StoreAdopted => BoundaryOutcome::HandleWord {
                                tag: BoundaryTag::PersistentGround,
                                class,
                                owner: BoundaryReferentOwner::PersistentStore,
                                identity: Self::handle_identity(
                                    BoundaryReferentOwner::PersistentStore,
                                ),
                            },
                        }
                    }
                }
            }
            BoundaryDisposition::RepresentedHandle { tag, class } => {
                let owner = tag.referent_owner();
                match (owner, self.reachability) {
                    // ⭐⭐ **`RT-DECL-CLOSURE-PORT` `D7` — the aggregate lifetime
                    // MEET.** A `Constructor`/`Record` whose value shape is
                    // persistable but which has a child that dies first is not
                    // an error; it is an aggregate whose lifetime is the
                    // invocation. The parent takes
                    // [`BoundaryTag::InvocationAggregate`] and the whole edge
                    // becomes sound — an invocation-owned parent naming an
                    // invocation-owned child dangles nothing.
                    //
                    // ⛔ This is NOT a relaxation of the escape rule. It is the
                    // missing lane the rule was standing in for: the refusal
                    // below still fires for every non-aggregate shape, and this
                    // arm's own referent owner is the arena, so every escape
                    // check downstream governs it unchanged.
                    //
                    // ⚠ Keyed on the CLASS, not on the tag, and deliberately:
                    // the incoming `tag` is whatever the value-shape
                    // disposition reached for, and the question here is whether
                    // this shape has children to take a meet over. `Bytes`,
                    // `String` and `Int` do not, so they keep the refusal.
                    (
                        BoundaryReferentOwner::PersistentStore,
                        ReachabilityPartition::ChildDiesBeforeParent,
                    ) if matches!(
                        class,
                        BoundaryClass::Constructor | BoundaryClass::Record
                    ) =>
                    {
                        BoundaryOutcome::HandleWord {
                            tag: BoundaryTag::InvocationAggregate,
                            class,
                            owner: BoundaryReferentOwner::InvocationArena,
                            identity: Self::handle_identity(
                                BoundaryReferentOwner::InvocationArena,
                            ),
                        }
                    }
                    // ⛔ A surviving parent may not name storage that dies
                    // first. Rejected before publication, with `ERR_ESCAPE`.
                    (
                        BoundaryReferentOwner::PersistentStore,
                        ReachabilityPartition::ChildDiesBeforeParent,
                    ) => BoundaryOutcome::FailClosedForbidden,
                    // ⛔ A persistent handle publishes only after the store has
                    // adopted it. Until then the node carries `NULL_SLOT`, which
                    // this ABI reads as invocation ownership — a word claiming
                    // otherwise contradicts its own layout.
                    (BoundaryReferentOwner::PersistentStore, ReachabilityPartition::Leaf)
                    | (
                        BoundaryReferentOwner::PersistentStore,
                        ReachabilityPartition::ChildrenOutliveParent,
                    ) => match self.adoption {
                        AdoptionPartition::PendingStoreAdoption => {
                            BoundaryOutcome::FailClosedForbidden
                        }
                        AdoptionPartition::StoreAdopted => BoundaryOutcome::HandleWord {
                            tag,
                            class,
                            owner,
                            identity: Self::handle_identity(owner),
                        },
                    },
                    // An invocation handle has no store identity to have, and
                    // adoption is not its boundary.
                    (BoundaryReferentOwner::InvocationArena, ReachabilityPartition::Leaf)
                    | (
                        BoundaryReferentOwner::InvocationArena,
                        ReachabilityPartition::ChildrenOutliveParent,
                    )
                    | (
                        BoundaryReferentOwner::InvocationArena,
                        ReachabilityPartition::ChildDiesBeforeParent,
                    ) => BoundaryOutcome::HandleWord {
                        tag,
                        class,
                        owner,
                        identity: Self::handle_identity(owner),
                    },
                    // A handle whose referent nothing owns is not representable.
                    (BoundaryReferentOwner::NoReferent, ReachabilityPartition::Leaf)
                    | (
                        BoundaryReferentOwner::NoReferent,
                        ReachabilityPartition::ChildrenOutliveParent,
                    )
                    | (
                        BoundaryReferentOwner::NoReferent,
                        ReachabilityPartition::ChildDiesBeforeParent,
                    ) => BoundaryOutcome::FailClosedForbidden,
                }
            }
        }
    }

    /// The identity a **published** handle of this owner carries.
    ///
    /// ⛔ A persistent handle is only ever published `StoreMinted` — a pending
    /// one is not a handle outcome at all, and `outcome` routes it to
    /// `FailClosedForbidden` before reaching here.
    fn handle_identity(owner: BoundaryReferentOwner) -> HandleIdentity {
        match owner {
            BoundaryReferentOwner::PersistentStore => HandleIdentity::StoreMinted,
            BoundaryReferentOwner::InvocationArena | BoundaryReferentOwner::NoReferent => {
                HandleIdentity::NoStoreIdentity
            }
        }
    }
}

impl BoundaryOutcome {
    /// Whether this outcome is one the static policy permits.
    ///
    /// ⛔ The entailment `AC-10` requires: the outcome is not merely *some*
    /// classification, it is one the variant's declared policy allows. An
    /// immediate-only policy yielding a handle is the misassignment the frame
    /// names, seen from the value level.
    pub(in crate::cranelift_backend) fn permitted_by(self, policy: StaticEncodingPolicy) -> bool {
        match (policy, self) {
            (StaticEncodingPolicy::ImmediateOnly, BoundaryOutcome::ImmediateWord { .. }) => true,
            (StaticEncodingPolicy::HandleOnly, BoundaryOutcome::HandleWord { .. })
            | (StaticEncodingPolicy::HandleOnly, BoundaryOutcome::FailClosedForbidden) => true,
            (
                StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill,
                BoundaryOutcome::ImmediateWord { .. },
            )
            | (
                StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill,
                BoundaryOutcome::HandleWord { .. },
            )
            // ⛔ A spill arm is a PERSISTENT handle, so an unadopted one fails
            // closed before publication. That is an *unrepresentable-input*
            // outcome, not admission of a represented value — the vacuity guard
            // is that all four outcomes stay inhabited and that magnitude still
            // changes this policy's outcome.
            | (
                StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill,
                BoundaryOutcome::FailClosedForbidden,
            ) => true,
            (StaticEncodingPolicy::ProtocolOnly, BoundaryOutcome::ProtocolOnly) => true,
            (StaticEncodingPolicy::FailClosedForbidden, BoundaryOutcome::FailClosedForbidden) => {
                true
            }
            (StaticEncodingPolicy::ImmediateOnly, BoundaryOutcome::HandleWord { .. })
            | (StaticEncodingPolicy::ImmediateOnly, BoundaryOutcome::ProtocolOnly)
            | (StaticEncodingPolicy::ImmediateOnly, BoundaryOutcome::FailClosedForbidden)
            | (StaticEncodingPolicy::HandleOnly, BoundaryOutcome::ImmediateWord { .. })
            | (StaticEncodingPolicy::HandleOnly, BoundaryOutcome::ProtocolOnly)
            | (
                StaticEncodingPolicy::ImmediateWithDeclaredHandleSpill,
                BoundaryOutcome::ProtocolOnly,
            )
            | (StaticEncodingPolicy::ProtocolOnly, BoundaryOutcome::ImmediateWord { .. })
            | (StaticEncodingPolicy::ProtocolOnly, BoundaryOutcome::HandleWord { .. })
            | (StaticEncodingPolicy::ProtocolOnly, BoundaryOutcome::FailClosedForbidden)
            | (StaticEncodingPolicy::FailClosedForbidden, BoundaryOutcome::ImmediateWord { .. })
            | (StaticEncodingPolicy::FailClosedForbidden, BoundaryOutcome::HandleWord { .. })
            | (StaticEncodingPolicy::FailClosedForbidden, BoundaryOutcome::ProtocolOnly) => false,
        }
    }
}

// ---------------------------------------------------------------------------
// `RECUT 2` — representation authority-to-execution closure
// ---------------------------------------------------------------------------
//
// ⛔ **`AC-10` closes the CLASSIFICATION; this closes the EXECUTION.** The
// partition above proves every boundary input reaches exactly one outcome
// permitted by its variant's static policy. It does **not** ask whether the
// outcome's lifecycle is executable end to end — and that is the predicate the
// Architect named across blocks `#1`–`#6`: *every representation authority must
// be the sole authority actually consumed by the production path it governs,
// and every admitted partition must have one total executable lifecycle.*
//
// ⭐ **Why this is a type and not a table.** The proof shape RECUT 2 retires is
// a hand-maintained matrix that can drift from the production enums. So the row
// set here is not written down: it is **derived** by iterating
// [`BoundaryInput::all`] and classifying, and the phases are **struct fields
// with no default**. A row that cannot say what closes a phase does not
// compile, and a new outcome or a new phase is a compile error rather than a
// row nobody added.
//
// ⚠ **The honest boundary, stated once here rather than implied:** the
// compiler closes *completeness* — that every required phase is bound. It does
// **not** close *identity* — that the bound anchor is the real production item
// rather than a lookalike. Identity is closed by named causal controls, and
// [`ProductionAnchor::derived_witness`] exists so that the subset of anchors
// which can be evaluated without a JIT are checked against production values
// rather than against their own spelling.

/// One phase of the lifecycle `RECUT 2` requires every admitted row to close.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum LifecyclePhase {
    /// The authority that decides the representation.
    Authority,
    /// Emitted code that constructs a word of this outcome.
    Producer,
    /// The check that rejects a malformed or unrepresentable input.
    Validator,
    /// Canonicalization and store adoption / identity mint.
    CanonicalizerAdopter,
    /// The step that makes the word visible past the producer.
    Publisher,
    /// A separately compiled reader that recovers the value.
    Consumer,
}

impl LifecyclePhase {
    /// Every phase, in lifecycle order.
    ///
    /// ⛔ Bound to the enum by [`LifecyclePhase::index`]'s wildcard-free match,
    /// so a seventh phase cannot be added without extending this array.
    pub(in crate::cranelift_backend) const ALL: [LifecyclePhase; 6] = [
        LifecyclePhase::Authority,
        LifecyclePhase::Producer,
        LifecyclePhase::Validator,
        LifecyclePhase::CanonicalizerAdopter,
        LifecyclePhase::Publisher,
        LifecyclePhase::Consumer,
    ];

    /// This phase's position in [`LifecyclePhase::ALL`].
    ///
    /// ⛔ **This is the pin that binds `ALL`'s length to the type.** A seventh
    /// variant is a non-exhaustive-match compile error here, and the control
    /// `recut2_the_phase_inventory_is_bound_to_the_type` checks that every
    /// index round-trips through `ALL` — so `ALL` cannot silently omit one.
    pub(in crate::cranelift_backend) fn index(self) -> usize {
        match self {
            LifecyclePhase::Authority => 0,
            LifecyclePhase::Producer => 1,
            LifecyclePhase::Validator => 2,
            LifecyclePhase::CanonicalizerAdopter => 3,
            LifecyclePhase::Publisher => 4,
            LifecyclePhase::Consumer => 5,
        }
    }
}

/// The production item that closes a phase.
///
/// ⛔ **Every variant names a real item on the production path**, not a
/// description of one. The `derived_witness` below is what keeps that honest
/// for the anchors it can reach: it returns a value **computed by** the named
/// authority, so deleting or rewiring the authority changes the witness.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) enum ProductionAnchor {
    /// `boundary_value::NodeField` / `RegionHeaderField` — the sole layout
    /// authority, from which every extent is derived (`AC-1` layout closure).
    LayoutFieldInventory,
    /// `boundary_value::boundary_int_magnitude_is_canonical` — the canonical
    /// sign/limb contract, authoritative wherever a word is built.
    IntNormalizationAuthority,
    /// `boundary_value_clif::emit_boundary_value_local_graph` — the emitted
    /// producer helpers.
    EmittedProducerGraph,
    /// The emitted escape gate refusing a pending or invocation-owned word.
    EmittedEscapeGate,
    /// `BoundaryValueStore::adopt`'s iterative tri-colour reachable-graph walk.
    ReachableGraphValidator,
    /// `BoundaryValueStore::adopt` — postorder canonicalization and the
    /// store-only identity mint.
    StoreAdoption,
    /// `BoundaryArenaBuilder::publish` / `BoundaryValueStore::publish_persistent`.
    RegionPublication,
    /// `boundary_value_clif::capture_boundary_value_local_graph` — a separately
    /// compiled consumer.
    SeparatelyCompiledConsumer,
}

impl ProductionAnchor {
    /// A value **computed by the named production item**, where that is
    /// possible without a JIT module.
    ///
    /// ⛔ **`None` is not a waiver and not a residual** — it says this anchor's
    /// identity is closed by a named causal control instead, and
    /// `recut2_every_anchor_is_closed_by_a_witness_or_a_named_control` requires
    /// each `None` anchor to appear in [`ProductionAnchor::CONTROL_CLOSED`].
    /// Making "cannot determine" a third outcome that must be *accounted for*,
    /// rather than one that falls through to pass, is the point.
    pub(in crate::cranelift_backend) fn derived_witness(self) -> Option<i64> {
        match self {
            // Derived from the field inventory: if a field is added, removed or
            // reordered, this value moves.
            ProductionAnchor::LayoutFieldInventory => {
                Some(crate::boundary_value::NODE_EXTENT as i64)
            }
            // Computed by calling the normalization authority on a magnitude it
            // must reject — a leading-zero limb is non-canonical by contract.
            ProductionAnchor::IntNormalizationAuthority => Some(i64::from(
                !crate::boundary_value::boundary_int_magnitude_is_canonical(0, &[1, 0]),
            )),
            // Derived from the escape gate's exact status constant.
            ProductionAnchor::EmittedEscapeGate => Some(crate::boundary_value::BOUNDARY_ERR_ESCAPE),
            // Derived from the validator's exact malformed-shape status.
            ProductionAnchor::ReachableGraphValidator => {
                Some(crate::boundary_value::BOUNDARY_ERR_CYCLE)
            }
            // Derived from the seal/quiescence handoff's exact status.
            ProductionAnchor::RegionPublication => Some(crate::boundary_value::BOUNDARY_ERR_SEALED),
            // ⛔ These three need a live JIT module to evaluate, so their
            // identity is control-closed rather than witness-closed.
            ProductionAnchor::EmittedProducerGraph
            | ProductionAnchor::StoreAdoption
            | ProductionAnchor::SeparatelyCompiledConsumer => None,
        }
    }

    /// The anchors whose identity is closed by a named causal control rather
    /// than by a derived witness, each paired with that control.
    ///
    /// ⛔ **This list is the residual given a cell**, in the frame's sense: it
    /// records what is control-enforced instead of letting the absence read as
    /// enforcement.
    pub(in crate::cranelift_backend) const CONTROL_CLOSED: &'static [(
        ProductionAnchor,
        &'static str,
    )] = &[
        (
            ProductionAnchor::EmittedProducerGraph,
            "b2v_the_helper_inventory_is_closed_and_named",
        ),
        (
            ProductionAnchor::StoreAdoption,
            "b2v_adoption_mints_a_real_slot_and_equal_values_converge",
        ),
        (
            ProductionAnchor::SeparatelyCompiledConsumer,
            "b2v_a_separately_compiled_consumer_recovers_the_value",
        ),
    ];
}

/// How one phase is closed for one row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum PhaseBinding {
    /// Closed by a named production item.
    Closed(ProductionAnchor),
    /// The outcome class structurally has no such phase.
    ///
    /// ⛔ **Derived from the outcome by [`BoundaryOutcome::requires`], never
    /// chosen per row.** If a row could declare a phase absent on its own
    /// authority, this enum would be the drift-prone matrix again with an
    /// escape hatch — every uncomfortable cell would become `StructurallyAbsent`
    /// and the artifact would close vacuously.
    StructurallyAbsent,
}

/// One row of the closure artifact — all six phases, none optional.
///
/// ⛔ **There is no `Default` and no `Option`.** Omitting a field is a
/// missing-field compile error, which is RECUT 2's *"a missing lifecycle phase
/// must be a construction failure"* discharged by construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct PhaseClosure {
    authority: PhaseBinding,
    producer: PhaseBinding,
    validator: PhaseBinding,
    canonicalizer_adopter: PhaseBinding,
    publisher: PhaseBinding,
    consumer: PhaseBinding,
}

impl PhaseClosure {
    /// This row's binding for one phase.
    pub(in crate::cranelift_backend) fn binding(&self, phase: LifecyclePhase) -> PhaseBinding {
        match phase {
            LifecyclePhase::Authority => self.authority,
            LifecyclePhase::Producer => self.producer,
            LifecyclePhase::Validator => self.validator,
            LifecyclePhase::CanonicalizerAdopter => self.canonicalizer_adopter,
            LifecyclePhase::Publisher => self.publisher,
            LifecyclePhase::Consumer => self.consumer,
        }
    }
}

impl BoundaryOutcome {
    /// Whether this outcome's class **requires** a phase.
    ///
    /// ⛔ **Derived from the outcome, so a row cannot excuse itself.** A
    /// `ProtocolOnly` value never reaches a boundary, so it has no producer,
    /// adopter, publisher or consumer — but it still has an authority that says
    /// so and a validator that enforces it. An invocation handle has no
    /// *canonicalizer/adopter* because store adoption must **reject** it
    /// (Ruling B item 6); that absence is a contract, not a gap.
    pub(in crate::cranelift_backend) fn requires(self, phase: LifecyclePhase) -> bool {
        match (self, phase) {
            // An immediate rides in the word: produced, validated, published and
            // read, but never canonicalized or adopted by the store.
            (BoundaryOutcome::ImmediateWord { .. }, LifecyclePhase::Authority)
            | (BoundaryOutcome::ImmediateWord { .. }, LifecyclePhase::Producer)
            | (BoundaryOutcome::ImmediateWord { .. }, LifecyclePhase::Validator)
            | (BoundaryOutcome::ImmediateWord { .. }, LifecyclePhase::Publisher)
            | (BoundaryOutcome::ImmediateWord { .. }, LifecyclePhase::Consumer) => true,
            (BoundaryOutcome::ImmediateWord { .. }, LifecyclePhase::CanonicalizerAdopter) => false,

            // ⛔ A store-minted handle is the only outcome that requires ALL
            // SIX. This is the row the six blocks kept failing.
            (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::StoreMinted,
                    ..
                },
                _,
            ) => true,

            // An invocation handle has no store identity to mint, by design.
            (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::NoStoreIdentity,
                    ..
                },
                LifecyclePhase::CanonicalizerAdopter,
            ) => false,
            (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::NoStoreIdentity,
                    ..
                },
                LifecyclePhase::Authority,
            )
            | (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::NoStoreIdentity,
                    ..
                },
                LifecyclePhase::Producer,
            )
            | (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::NoStoreIdentity,
                    ..
                },
                LifecyclePhase::Validator,
            )
            | (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::NoStoreIdentity,
                    ..
                },
                LifecyclePhase::Publisher,
            )
            | (
                BoundaryOutcome::HandleWord {
                    identity: HandleIdentity::NoStoreIdentity,
                    ..
                },
                LifecyclePhase::Consumer,
            ) => true,

            // Never a source value at a boundary, and rejected before emission:
            // both are closed by an authority plus the validator that enforces
            // it, and neither ever produces, adopts, publishes or is read.
            (BoundaryOutcome::ProtocolOnly, LifecyclePhase::Authority)
            | (BoundaryOutcome::ProtocolOnly, LifecyclePhase::Validator)
            | (BoundaryOutcome::FailClosedForbidden, LifecyclePhase::Authority)
            | (BoundaryOutcome::FailClosedForbidden, LifecyclePhase::Validator) => true,
            (BoundaryOutcome::ProtocolOnly, LifecyclePhase::Producer)
            | (BoundaryOutcome::ProtocolOnly, LifecyclePhase::CanonicalizerAdopter)
            | (BoundaryOutcome::ProtocolOnly, LifecyclePhase::Publisher)
            | (BoundaryOutcome::ProtocolOnly, LifecyclePhase::Consumer)
            | (BoundaryOutcome::FailClosedForbidden, LifecyclePhase::Producer)
            | (BoundaryOutcome::FailClosedForbidden, LifecyclePhase::CanonicalizerAdopter)
            | (BoundaryOutcome::FailClosedForbidden, LifecyclePhase::Publisher)
            | (BoundaryOutcome::FailClosedForbidden, LifecyclePhase::Consumer) => false,
        }
    }

    /// The lifecycle closure for this outcome.
    ///
    /// ⛔ **Wildcard-free over the outcome, and every field is mandatory**, so
    /// a new outcome variant is a compile error and an existing one cannot ship
    /// a hole.
    pub(in crate::cranelift_backend) fn phase_closure(self) -> PhaseClosure {
        match self {
            BoundaryOutcome::ImmediateWord { .. } => PhaseClosure {
                authority: PhaseBinding::Closed(ProductionAnchor::LayoutFieldInventory),
                producer: PhaseBinding::Closed(ProductionAnchor::EmittedProducerGraph),
                validator: PhaseBinding::Closed(ProductionAnchor::IntNormalizationAuthority),
                canonicalizer_adopter: PhaseBinding::StructurallyAbsent,
                publisher: PhaseBinding::Closed(ProductionAnchor::RegionPublication),
                consumer: PhaseBinding::Closed(ProductionAnchor::SeparatelyCompiledConsumer),
            },
            BoundaryOutcome::HandleWord {
                identity: HandleIdentity::StoreMinted,
                ..
            } => PhaseClosure {
                authority: PhaseBinding::Closed(ProductionAnchor::LayoutFieldInventory),
                producer: PhaseBinding::Closed(ProductionAnchor::EmittedProducerGraph),
                validator: PhaseBinding::Closed(ProductionAnchor::ReachableGraphValidator),
                canonicalizer_adopter: PhaseBinding::Closed(ProductionAnchor::StoreAdoption),
                publisher: PhaseBinding::Closed(ProductionAnchor::RegionPublication),
                consumer: PhaseBinding::Closed(ProductionAnchor::SeparatelyCompiledConsumer),
            },
            BoundaryOutcome::HandleWord {
                identity: HandleIdentity::NoStoreIdentity,
                ..
            } => PhaseClosure {
                authority: PhaseBinding::Closed(ProductionAnchor::LayoutFieldInventory),
                producer: PhaseBinding::Closed(ProductionAnchor::EmittedProducerGraph),
                validator: PhaseBinding::Closed(ProductionAnchor::EmittedEscapeGate),
                canonicalizer_adopter: PhaseBinding::StructurallyAbsent,
                publisher: PhaseBinding::Closed(ProductionAnchor::RegionPublication),
                consumer: PhaseBinding::Closed(ProductionAnchor::SeparatelyCompiledConsumer),
            },
            BoundaryOutcome::ProtocolOnly => PhaseClosure {
                authority: PhaseBinding::Closed(ProductionAnchor::LayoutFieldInventory),
                producer: PhaseBinding::StructurallyAbsent,
                validator: PhaseBinding::Closed(ProductionAnchor::EmittedEscapeGate),
                canonicalizer_adopter: PhaseBinding::StructurallyAbsent,
                publisher: PhaseBinding::StructurallyAbsent,
                consumer: PhaseBinding::StructurallyAbsent,
            },
            BoundaryOutcome::FailClosedForbidden => PhaseClosure {
                authority: PhaseBinding::Closed(ProductionAnchor::LayoutFieldInventory),
                producer: PhaseBinding::StructurallyAbsent,
                validator: PhaseBinding::Closed(ProductionAnchor::ReachableGraphValidator),
                canonicalizer_adopter: PhaseBinding::StructurallyAbsent,
                publisher: PhaseBinding::StructurallyAbsent,
                consumer: PhaseBinding::StructurallyAbsent,
            },
        }
    }
}

/// `RT-FNSPLIT-B2V` `D4` — what a `Lowered` becomes when it crosses a boundary.
///
/// ⛔ **The population is closed by the compiler, not by a histogram.** The
/// `#10` evidence measured 41 source-valued transfers and 26-of-154 aggregate
/// root results; those numbers are *corroboration*. The proof is
/// [`Lowered::boundary_disposition`]'s exhaustive, wildcard-free `match` over
/// the 21 landed variants: a 22nd variant is a **compile error** until someone
/// dispositions it, never a silent default into `ValueWord`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum BoundaryDisposition {
    /// The payload rides in the tagged word itself.
    ///
    /// `spill` names the handle class used when a **runtime** magnitude test
    /// finds the payload too wide for the immediate field. ⭐ Spelling the
    /// spill out is the point: without it "represented immediate" would quietly
    /// claim that every `Int` fits 56 bits, which is false for exactly the
    /// values a bignum language exists to carry.
    RepresentedImmediate {
        tag: BoundaryTag,
        spill: Option<BoundaryClass>,
    },
    /// An opaque handle. The **referent** owner is part of the disposition
    /// because it is a different question from who owns the frame slot (`D2`).
    RepresentedHandle {
        tag: BoundaryTag,
        class: BoundaryClass,
    },
    /// Never a source value at a boundary — protocol machinery only.
    ProtocolOnly { why: &'static str },
    /// Rejected **before** emission, with an exact error.
    FailClosedForbidden { why: &'static str },
}

impl Lowered {
    /// The boundary disposition of this value.
    ///
    /// ⛔ **No `_` arm, by construction.** Every variant is named.
    /// The boundary disposition of this value's VARIANT.
    ///
    /// ⛔ A policy is a claim about the whole variant, so it is a function of
    /// the variant TAG and of nothing else — see [`LoweredVariant`]. Delegating
    /// makes that structural: this cannot come to depend on a payload without
    /// someone changing the signature, and the tag set is enumerable, so the
    /// `AC-3` assignment can be swept without constructing 21 values.
    pub(in crate::cranelift_backend) fn boundary_disposition(&self) -> BoundaryDisposition {
        self.variant().boundary_disposition()
    }

    /// `RT-FNSPLIT-C1` `D5` — whether this **whole value graph** may cross the
    /// boundary, decided before anything is allocated, written or published.
    ///
    /// ⭐⭐ **The root variant table is not sufficient, and that is the finding
    /// this walk exists to encode.** `boundary_disposition` is a function of the
    /// root tag alone, so it reports `RepresentedHandle` for a `Constructor`
    /// whose arguments contain a closure. Nothing in the lowering excludes that
    /// shape: `lower_expr`'s `Construct` arm lowers each argument through
    /// `lower_expr` and screens only for `RecursiveBackedge`, so a closure
    /// nested inside a constructor is constructible on the live path.
    ///
    /// ⇒ Admissibility is a property of the **graph**, not of the root.
    ///
    /// ⛔ **Total and wildcard-free by construction.** Every variant is named,
    /// so a 22nd `Lowered` inhabitant is a compile error here as well as in
    /// `variant()` — a new carrier of children cannot be added without deciding
    /// whether it can hide a closure.
    ///
    /// ⚠ **Ordering is load-bearing: this runs BEFORE any allocation, store
    /// write, adoption or publication.** A walk performed after the first child
    /// is published would reject the transfer having already emitted part of
    /// it, which is a partial publication rather than a rejection.
    ///
    /// ⚠ **The completeness cost, stated honestly and in its true size:** this
    /// rejects only graphs that *actually contain* a closure. ⛔ It does **not**
    /// reject the `Constructor` variant, and it does **not** follow that the 29
    /// of 41 measured `Constructor` transfers fail — only those whose actual
    /// argument graph holds a closure do. ⭐ And the measured zero-closure
    /// transfer census proves nothing either way, because the carrier is inert:
    /// that zero holds for every variant and cannot distinguish "closures never
    /// transfer" from "nothing transfers yet."
    /// **The producer-issued occurrence of a source aggregate, whichever shape
    /// it is.**
    ///
    /// ⭐ One reader over both aggregate variants, so a consumer asks *"what is
    /// this template's producer authority?"* without branching on shape and
    /// without a shape-specific spelling drifting from its sibling. `None` for
    /// every non-aggregate, and for an aggregate whose occurrence is absent —
    /// which is an explicit fail-closed absence at the allocation, never a
    /// signal to fall back to a use coordinate.
    pub(in crate::cranelift_backend) fn source_aggregate_producer(
        &self,
    ) -> Option<AggregateOccurrenceId> {
        match self {
            Lowered::Constructor { occurrence, .. } | Lowered::Record { occurrence, .. } => {
                *occurrence
            }
            _ => None,
        }
    }

    pub(in crate::cranelift_backend) fn boundary_transfer_admissibility(
        &self,
    ) -> Result<(), CraneliftBackendError> {
        match self {
            // ── closures: the rejection this walk exists for ──────────────
            //
            // ⛔ One exact typed error at every depth, so a nested rejection is
            // not reported as some enclosing variant's failure.
            Lowered::Closure { .. } | Lowered::DeclarationClosure { .. } => {
                #[cfg(test)]
                d5a_trace(format!(
                    "  BOUNDARY-REFUSAL first closure child variant={}",
                    lowered_value_kind(self)
                ));
                Err(unsupported(
                    "Closure",
                    "a closure cannot cross the boundary: it is runtime-local and \
                     live-domain only, and it has no durable lane",
                ))
            }
            Lowered::ComputationalRecursorClosure { .. } => Err(unsupported(
                "ComputationalMatch",
                "a computational recursor closure names an in-flight activation, \
                 not a transferable value",
            )),

            // ── recursive carriers: recurse into EVERY child position ─────
            Lowered::Constructor { args, .. } => {
                for arg in args {
                    arg.boundary_transfer_admissibility()?;
                }
                Ok(())
            }
            Lowered::Record { fields, .. } => {
                for field in fields {
                    field.value.boundary_transfer_admissibility()?;
                }
                Ok(())
            }
            Lowered::HostResult { error, ok, .. } => {
                error.boundary_transfer_admissibility()?;
                ok.boundary_transfer_admissibility()
            }
            // ⚠ **The child position most easily missed.** `DynamicConstructor`
            // looks like a leaf: its payload is a struct, and the children are
            // two levels down, in a `Vec` of alternative structs. Treating it
            // as a leaf would leave a closure nested in a dynamic alternative
            // completely unguarded while every other arm was correct.
            Lowered::DynamicConstructor(dynamic) => {
                for alternative in &dynamic.alternatives {
                    for field in &alternative.fields {
                        field.boundary_transfer_admissibility()?;
                    }
                }
                Ok(())
            }

            // ── leaves: no `Lowered` child position exists ────────────────
            //
            // ⛔ Admitted here means "holds no closure", NOT "is transferable".
            // Whether a leaf has a boundary representation at all is
            // `boundary_disposition`'s question and is decided separately; a
            // `ProtocolOnly` or otherwise forbidden leaf is still refused
            // there. Conflating the two would let this walk read as a transfer
            // authorization it is not.
            Lowered::Int { .. }
            | Lowered::Bool { .. }
            | Lowered::ProcessExitStatus { .. }
            | Lowered::CapabilityToken { .. }
            | Lowered::ResourceToken { .. }
            | Lowered::BoundedNat(_)
            | Lowered::StructuralNat(_)
            | Lowered::ResponseBytes { .. }
            | Lowered::Bytes(_)
            | Lowered::BorrowedNativeValue { .. }
            | Lowered::BorrowedOption { .. }
            | Lowered::String(_)
            | Lowered::RecursiveBackedge
            | Lowered::Trap(_) => Ok(()),
        }
    }
}

impl LoweredVariant {
    /// The boundary disposition of this variant.
    ///
    /// ⛔ **No `_` arm, by construction.** Every variant is named.
    pub(in crate::cranelift_backend) fn boundary_disposition(self) -> BoundaryDisposition {
        use BoundaryDisposition::{
            FailClosedForbidden, ProtocolOnly, RepresentedHandle, RepresentedImmediate,
        };
        match self {
            // ─── represented immediates ──────────────────────────────────
            //
            // Ken's `Int` is arbitrary precision, so the immediate field is a
            // fast path and the spill is the general case. The choice between
            // them is made by emitted code from the value's magnitude at
            // RUNTIME; nothing inspects a JIT-time value to pick a layout,
            // which is `AC-2`.
            LoweredVariant::Int => RepresentedImmediate {
                tag: BoundaryTag::ImmediateInt,
                spill: Some(BoundaryClass::Int),
            },
            // One bit. The only immediate that cannot overflow its field.
            LoweredVariant::Bool => RepresentedImmediate {
                tag: BoundaryTag::ImmediateBool,
                spill: None,
            },
            LoweredVariant::ProcessExitStatus => RepresentedImmediate {
                tag: BoundaryTag::ImmediateExitStatus,
                spill: Some(BoundaryClass::Int),
            },
            LoweredVariant::BoundedNat => RepresentedImmediate {
                tag: BoundaryTag::ImmediateBoundedNat,
                spill: Some(BoundaryClass::Int),
            },
            LoweredVariant::StructuralNat => RepresentedImmediate {
                tag: BoundaryTag::ImmediateStructuralNat,
                spill: Some(BoundaryClass::Int),
            },

            // ─── tokens: handles, NOT immediates ─────────────────────────
            //
            // ⛔ A capability or resource token is an opaque 64-bit identity,
            // and the immediate field is 56 bits. Truncating it would let two
            // distinct tokens compare equal — an authority forgery, not a
            // rounding error — so these take a handle whose node payload holds
            // the full word. Their owner is the invocation because that is
            // already the extent over which the token is valid.
            LoweredVariant::CapabilityToken | LoweredVariant::ResourceToken => RepresentedHandle {
                tag: BoundaryTag::InvocationBorrowed,
                class: BoundaryClass::BorrowedOpaque,
            },

            // ─── persistable ground values ───────────────────────────────
            //
            // `Constructor` is a REQUIRED live arm: 29 of the 41 measured
            // source-valued transfers are `Constructor` parameters, and a
            // disposition that parked it in `FailClosedForbidden` would reject
            // the dominant population — sound, and unable to satisfy `B2F`'s
            // `D6`/`D7`. That is the whole finding of `#10`.
            LoweredVariant::Constructor | LoweredVariant::DynamicConstructor => RepresentedHandle {
                tag: BoundaryTag::PersistentGround,
                class: BoundaryClass::Constructor,
            },
            LoweredVariant::Record => RepresentedHandle {
                tag: BoundaryTag::PersistentGround,
                class: BoundaryClass::Record,
            },
            LoweredVariant::String => RepresentedHandle {
                tag: BoundaryTag::PersistentGround,
                class: BoundaryClass::String,
            },
            LoweredVariant::Bytes => RepresentedHandle {
                tag: BoundaryTag::PersistentGround,
                class: BoundaryClass::Bytes,
            },

            // ─── borrowed ingress ────────────────────────────────────────
            //
            // ⛔ Invocation-owned: the referent is host storage that dies with
            // the native invocation. `AC-7`'s escape check keys on exactly this
            // owner, so a word naming one cannot silently outlive its buffer.
            //
            // `HostResult` is the second REQUIRED live arm. It carries a
            // RUNTIME success discriminant plus the two payloads it selects
            // between; the landed lowering holds those payloads as compile-time
            // templates, which is why a compiled-once callee cannot consume one
            // today.
            LoweredVariant::HostResult => RepresentedHandle {
                tag: BoundaryTag::InvocationHostResult,
                class: BoundaryClass::HostResult,
            },
            LoweredVariant::ResponseBytes
            | LoweredVariant::BorrowedNativeValue
            | LoweredVariant::BorrowedOption => RepresentedHandle {
                tag: BoundaryTag::InvocationBorrowed,
                class: BoundaryClass::BorrowedOpaque,
            },

            // ─── closures: FAIL CLOSED for `C1` ──────────────────────────
            //
            // ⛔ **Changed by `RT-FNSPLIT-C1` under Architect Decision
            // `dec_21aa95jbsznfh`, and the history is the point.**
            //
            // `B2V` landed this arm as `RepresentedHandle { tag:
            // BoundaryTag::PersistentClosure, class: BoundaryClass::Closure }`,
            // deliberately, reasoning that *"a `FailClosedForbidden` here would
            // guarantee that wall for `B2F`."* That reasoning was recorded and
            // never executed — the whole disposition was inert.
            //
            // ⭐ The conflict it hid: `PersistentClosure` is the DURABLE lane
            // (`referent_owner() == PersistentStore`; the word outlives the
            // invocation that minted it), and `C1`'s settled input is that
            // ordinary closures stay **runtime-local and live-domain only**.
            // Making the landed disposition execute would have restored exactly
            // the lane the `#11` ruling forbids.
            //
            // ⛔ So this is conditional rejection of a VALUE SHAPE, not of the
            // closure concept and not of `Constructor`: a closure-free
            // constructor is still admitted and has its own positive control.
            // ⛔ Do not "fix" this by adding a third closure tag or by
            // disguising a closure as `InvocationBorrowed` / `BorrowedOpaque` —
            // both were considered and refused; they violate the ownership and
            // self-evidence boundaries rather than respecting them.
            //
            // ⚠ A live-domain closure carrier is a real and expected future
            // mechanism, but it is **`B2F`'s design**: it needs invocation
            // ownership, static origin, captured `BoundaryWord`s, callable
            // dispatch and non-escape enforcement. ⛔ `C1` may not invent it,
            // and this arm is not the place to smuggle it in.
            LoweredVariant::Closure | LoweredVariant::DeclarationClosure => FailClosedForbidden {
                why: "an ordinary closure is runtime-local and live-domain only; it has \
                      no durable boundary lane, and a callable cross-owner carrier is \
                      B2F's design rather than this node's",
            },

            // ─── fail-closed ─────────────────────────────────────────────
            //
            // ⛔ Not a value: it names a `ContinuationActivationId` and a
            // `RecursorInvocationSegment`, which identify ONE in-flight
            // activation of the enclosing recursor. Transferring it to another
            // unit would hand over a cursor into a frame that unit does not
            // have. Rejected before emission, with an exact error.
            LoweredVariant::ComputationalRecursorClosure => FailClosedForbidden {
                why: "a computational recursor closure names an in-flight activation, \
                      not a transferable value",
            },

            // ─── protocol-only ───────────────────────────────────────────
            LoweredVariant::RecursiveBackedge => ProtocolOnly {
                why: "a tail-recursive edge is already a CFG jump; the block is \
                      predecessor-free and there is no value to transfer",
            },
            // The trap word is its own `AbiCarrier`, written by the protocol.
            // ⛔ `result_carrier` is not its producer — the `AC-11` correction
            // on `B2F` says exactly this, and it holds here too.
            LoweredVariant::Trap => ProtocolOnly {
                why: "a trap is written to the activation's trap word, which is a \
                      protocol carrier and not a source-expression result",
            },
        }
    }
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
    identity: ConstructorIdentity,
    /// **`D7` — the planner's occurrence for this exact alternative.**
    ///
    /// ⭐ A dynamic SET is not an allocation; a selected ALTERNATIVE is. It
    /// calls `emit_carrier_alloc` exactly as a fixed constructor does, so it
    /// needs the same path-keyed record — otherwise its allocation has no
    /// lifetime meet and cannot enter the event-to-record relation.
    ///
    /// `None` is a refusal, never a default: it means no context was being
    /// defined, and the allocation fails loudly rather than borrowing a lane.
    occurrence: Option<AggregateOccurrenceId>,
    fields: Vec<Lowered>,
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — the four sites that construct a governed
/// allocation request.**
///
/// ⭐ Named in production, not only under test. The domain is a real fact about
/// the emitter — these four are exactly the places an aggregate governed by a
/// planned record is allocated — and naming them is what lets a control act at
/// ONE of them while the other three stay honest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GovernedAllocationSite {
    /// A source `Construct`, carried through `emit_carrier_transfer`.
    SourceConstructor,
    /// A source `Record`, likewise.
    SourceRecord,
    /// The SELECTED alternative of a compiler-synthesized dynamic constructor.
    /// The set is not an allocation; the alternative is.
    DynamicAlternative,
    /// A constructor built from already-lowered operands at the process
    /// boundary (`transfer_constructor_operands`).
    CarriedConstructor,
}

/// **`D7` — the closed mutation surface for the governed-allocation controls.**
///
/// ⛔ `#[cfg(test)]`, so none of it exists in a shipped compiler. It is a
/// closed sum rather than a set of booleans for the same reason
/// [`CarrierAllocationRequest`] is: at most one perturbation can be installed
/// at a time, and "two bypasses at once" is not a state a control should be
/// able to reach by accident.
///
/// ⭐ Each variant acts at exactly ONE seam and increments the hit counter when
/// it does. A control that asserts only the refusal cannot tell "the site
/// bypassed and the choke caught it" from "the fixture never reached the site
/// and something else failed" — the hit count is what separates those, and it
/// is why every variant is required to prove it fired.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GovernedAllocationMutation {
    None,
    /// Hand the choke a `NonAggregate` request at the named site.
    Bypass(GovernedAllocationSite),
    /// Select the planned occurrence and record at a DIFFERENT live effect seat
    /// running the same host operation, retaining this seat's construction and
    /// operands. The A/B seat discriminator.
    SiblingEffectSeat,
    /// Transfer every source-call input at the program ROOT instead of at the
    /// coordinate the call supplies, keeping every value, callee, parameter
    /// slot, shape, lane and order. The call-USE coordinate discriminator.
    CallInputTransferOrigin,
    /// Give one source-call argument's TEMPLATE a sibling argument's
    /// planner-issued **producer occurrence**, keeping its own value, args,
    /// constructor symbol, call use, callee, parameter slot, shape, lane and
    /// order. The A/B aggregate-ownership discriminator.
    ///
    /// ⛔ **Not the same axis as [`Self::CallInputTransferOrigin`], and it may
    /// not be cited as this control.** That one moves the coordinate a value is
    /// *transferred at*; this one moves the certificate the value *carries*.
    /// Since `aggregate_carrier_authority` prefers the carried occurrence, a
    /// use-coordinate substitution is inert for exactly the templates that
    /// authorize themselves — which is every aggregate on this route.
    ///
    /// ⭐ The occurrence is taken from a **live sibling argument's own
    /// template**, never constructed. A hand-made identity would test that the
    /// record lookup rejects nonsense; taking a real one tests that ownership
    /// discriminates between two aggregates the plan considers equally real.
    SiblingAggregateProducer,
    /// Transfer a still-specialized call input at the PROGRAM ROOT's occurrence
    /// instead of the callee's scheduling entry, at the one call-input site that
    /// has no caller-side occurrence to carry.
    ///
    /// ⭐ The self-authority probe. A template that authorizes itself is
    /// unaffected by which coordinate it is transferred at; one that does not
    /// resolves a different record, or none at all.
    CalleeSchedulingOrigin,
}

#[cfg(test)]
thread_local! {
    static GOVERNED_ALLOCATION_MUTATION: std::cell::Cell<GovernedAllocationMutation> =
        const { std::cell::Cell::new(GovernedAllocationMutation::None) };
    static GOVERNED_ALLOCATION_HITS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// How many RAW carrier allocations have actually been emitted.
    ///
    /// ⭐ Counted at the `alloc` call itself, not at the choke's entry and not
    /// at the ledger. "Refused before any allocation" is a claim about emitted
    /// instructions, so the instrument has to sit where the instruction is
    /// emitted -- a counter one frame earlier would be satisfied by a refusal
    /// that had already allocated.
    static CARRIER_RAW_ALLOCATIONS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// What the ownership substitution actually moved, recorded at the seam.
    ///
    /// ⭐ Both records are read from the LIVE plan, at the moment of the swap.
    /// "The two arguments agree on shape and lane" is the premise that makes the
    /// negative a statement about IDENTITY rather than about shape, and a
    /// premise re-derived afterwards from a separately built plan is a premise
    /// about a different object.
    static SIBLING_PRODUCER_SUBSTITUTION: std::cell::Cell<Option<SiblingProducerSubstitution>> =
        const { std::cell::Cell::new(None) };
    /// How many SELF-AUTHORIZING aggregates reached the callee-scheduling-entry
    /// fallback -- the one call-input site with no caller-side occurrence.
    static SELF_AUTHORIZED_FALLBACK_REACHES: std::cell::Cell<u32> =
        const { std::cell::Cell::new(0) };
    /// The last `(passed in, actually used)` coordinate pair the self-authority
    /// probe returned.
    ///
    /// ⛔ Recorded at the RETURN, not at the decision. A hit counter proves the
    /// seam decided to substitute; only this proves it substituted. Measured:
    /// with the seam's return value reverted to its argument while the counter
    /// still fired, the control stayed green -- a no-op substitution is
    /// otherwise indistinguishable from a well-defended one.
    static CALLEE_SCHEDULING_ORIGIN_USED: std::cell::Cell<
        Option<(StaticOriginId, StaticOriginId)>,
    > = const { std::cell::Cell::new(None) };
}

/// The exact ownership substitution one A/B run performed.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SiblingProducerSubstitution {
    /// The certificate the first argument carried before the substitution.
    pub(in crate::cranelift_backend) from: Option<AggregateOccurrenceId>,
    /// The sibling certificate it was given.
    pub(in crate::cranelift_backend) to: AggregateOccurrenceId,
    /// Whether the two records agree on planned shape.
    pub(crate) same_shape: bool,
    /// Whether the two records agree on planned allocation lane.
    pub(crate) same_lane: bool,
}

/// RAII installation of one mutation.
///
/// ⛔ The restore is in `Drop` rather than at the end of the control, because a
/// control that asserts a refusal is a control whose happy path can `panic!`
/// mid-way. A hand-written reset after the assertion would be skipped exactly
/// when the assertion fails, leaving the mutation installed for every test that
/// runs afterwards on this thread — a whole-suite corruption produced by the
/// one failure you were trying to diagnose.
#[cfg(test)]
pub(crate) struct GovernedAllocationMutationGuard {
    previous: GovernedAllocationMutation,
    previous_hits: u32,
    previous_allocations: u32,
    previous_substitution: Option<SiblingProducerSubstitution>,
    previous_reaches: u32,
    previous_origin_used: Option<(StaticOriginId, StaticOriginId)>,
}

#[cfg(test)]
impl GovernedAllocationMutationGuard {
    pub(crate) fn install(mutation: GovernedAllocationMutation) -> Self {
        let guard = Self {
            previous: GOVERNED_ALLOCATION_MUTATION.with(std::cell::Cell::get),
            previous_hits: GOVERNED_ALLOCATION_HITS.with(std::cell::Cell::get),
            previous_allocations: CARRIER_RAW_ALLOCATIONS.with(std::cell::Cell::get),
            previous_substitution: SIBLING_PRODUCER_SUBSTITUTION.with(std::cell::Cell::get),
            previous_reaches: SELF_AUTHORIZED_FALLBACK_REACHES.with(std::cell::Cell::get),
            previous_origin_used: CALLEE_SCHEDULING_ORIGIN_USED.with(std::cell::Cell::get),
        };
        GOVERNED_ALLOCATION_MUTATION.with(|cell| cell.set(mutation));
        GOVERNED_ALLOCATION_HITS.with(|cell| cell.set(0));
        CARRIER_RAW_ALLOCATIONS.with(|cell| cell.set(0));
        SIBLING_PRODUCER_SUBSTITUTION.with(|cell| cell.set(None));
        SELF_AUTHORIZED_FALLBACK_REACHES.with(|cell| cell.set(0));
        CALLEE_SCHEDULING_ORIGIN_USED.with(|cell| cell.set(None));
        guard
    }

    /// What the ownership substitution moved, if one fired.
    pub(crate) fn substitution(&self) -> Option<SiblingProducerSubstitution> {
        SIBLING_PRODUCER_SUBSTITUTION.with(std::cell::Cell::get)
    }

    /// How many self-authorizing aggregates reached the callee-scheduling-entry
    /// fallback.
    pub(crate) fn self_authorized_fallback_reaches(&self) -> u32 {
        SELF_AUTHORIZED_FALLBACK_REACHES.with(std::cell::Cell::get)
    }

    /// The `(passed in, actually used)` coordinate pair the self-authority probe
    /// last returned.
    pub(in crate::cranelift_backend) fn callee_scheduling_origin_used(
        &self,
    ) -> Option<(StaticOriginId, StaticOriginId)> {
        CALLEE_SCHEDULING_ORIGIN_USED.with(std::cell::Cell::get)
    }

    /// How many times this mutation's seam actually fired.
    pub(crate) fn hits(&self) -> u32 {
        GOVERNED_ALLOCATION_HITS.with(std::cell::Cell::get)
    }

    /// How many raw carrier allocations were emitted since this guard installed.
    ///
    /// ⚠ Zero is only meaningful beside a baseline that is NON-zero. On its
    /// own it is equally consistent with "refused before allocating" and with
    /// "this fixture never allocates", and those are different claims.
    pub(crate) fn raw_allocations(&self) -> u32 {
        CARRIER_RAW_ALLOCATIONS.with(std::cell::Cell::get)
    }
}

#[cfg(test)]
impl Drop for GovernedAllocationMutationGuard {
    fn drop(&mut self) {
        GOVERNED_ALLOCATION_MUTATION.with(|cell| cell.set(self.previous));
        GOVERNED_ALLOCATION_HITS.with(|cell| cell.set(self.previous_hits));
        CARRIER_RAW_ALLOCATIONS.with(|cell| cell.set(self.previous_allocations));
        SIBLING_PRODUCER_SUBSTITUTION.with(|cell| cell.set(self.previous_substitution));
        SELF_AUTHORIZED_FALLBACK_REACHES.with(|cell| cell.set(self.previous_reaches));
        CALLEE_SCHEDULING_ORIGIN_USED.with(|cell| cell.set(self.previous_origin_used));
    }
}

#[cfg(test)]
fn governed_allocation_hit() {
    GOVERNED_ALLOCATION_HITS.with(|hits| hits.set(hits.get().saturating_add(1)));
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — the CLOSED request the deepest carrier
/// allocator accepts.**
///
/// ⭐ There is no way to hand the allocator a bare tag for an aggregate. A
/// caller either names a **planned record** — and the lane, the event and the
/// relation pair all follow from that record — or it declares the allocation
/// **non-aggregate**, in which case a `Constructor`/`Record` class is refused
/// before anything is emitted.
///
/// ⛔ This is what makes *"every governed allocation is in `E`"* a property of
/// the ALLOCATOR rather than of caller discipline. The predecessor paired a
/// checked wrapper that recorded evidence with a raw helper beside it that did
/// not, so the law held exactly as long as every future caller remembered which
/// one to reach for — an obligation nothing enforced and nothing measured. A
/// bypass was one plausible-looking line, and it would have produced an
/// allocation in no body's event set with no diagnostic anywhere.
///
/// ⛔ Not a `bool`, not an `Option<AggregateOccurrenceId>`. Both spellings
/// admit "governed, but with no record" and "ungoverned, but at an aggregate
/// class"; the sum admits neither, so the choke's two refusals are total over
/// the domain rather than over the cases someone enumerated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CarrierAllocationRequest {
    /// A scalar, spill, byte-bodied, `HostResult` or borrowed allocation. It
    /// names no record and never enters `E`.
    ///
    /// ⛔ The tag is the caller's precisely because there is no record to take
    /// one from — which is why this variant may not carry an aggregate class.
    NonAggregate { tag: BoundaryTag },
    /// An aggregate governed by a planned ownership record.
    ///
    /// ⛔ Carries no tag. The lane is read from the record at `occurrence`,
    /// never from the value in hand.
    PlannedAggregate {
        occurrence: AggregateOccurrenceId,
        shape: PlannedAggregateShape,
    },
}

impl CarrierAllocationRequest {
    /// The one node class an aggregate of this shape may be carried at.
    ///
    /// ⚠ Total over `PlannedAggregateShape`, so a third shape is a compile
    /// error here rather than a class that silently defaults.
    fn aggregate_class(shape: PlannedAggregateShape) -> BoundaryClass {
        match shape {
            PlannedAggregateShape::Constructor => BoundaryClass::Constructor,
            PlannedAggregateShape::Record => BoundaryClass::Record,
        }
    }
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — the identity of one aggregate allocation
/// EVENT within one compilation.**
///
/// ⭐ `FuncId` is the exact declared function handed to `define_function`, and
/// it scopes **event evidence only** — never planner authority. The planner
/// keys records by `owner + seat + path + role`; this keys the *emissions* of
/// those records, which is a different question with a different answer.
///
/// ## Why the result `Value` alone is not an identity
///
/// A CLIF `Value` is numbered **per function**, so two bodies allocate at
/// `v12` routinely. A first prototype keyed on the value alone and refused six
/// lawful allocations. Adding the emission owner and the raw defining unit did
/// not fix it either — the same function is *built more than once*, so those
/// two coordinates still aliased. `FuncId` is the coordinate that actually
/// separates them, because it is what the module identifies a definition by.
///
/// ⛔ Not a build counter. A counter would make the identity depend on the
/// order bodies happen to be emitted in, which is exactly the row-driven
/// discovery this domain refuses.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::cranelift_backend) struct AggregateAllocationEvent {
    function: FuncId,
    result: cranelift_codegen::ir::Value,
}

/// The events of one body, open while that body is being emitted.
///
/// ⭐ **`events` and `relation` are INDEPENDENTLY MUTABLE, on purpose.** `E` is
/// what actually allocated; `R` is what those allocations were related to. A
/// single map cannot tell the two apart — its keys would BE its domain by
/// construction — so "an event was recorded but never related" and "a relation
/// entry exists for no event" would both be unstateable, and the local close
/// below would have nothing to compare. The law is `dom(R) = E`, and a law
/// needs two sides.
#[derive(Clone, Debug)]
struct LocalAggregateEvents {
    function: FuncId,
    /// `E` — one entry per governed raw allocation that actually happened.
    events: BTreeSet<cranelift_codegen::ir::Value>,
    /// `R` — the pairing of those events to the records that govern them.
    relation: BTreeMap<cranelift_codegen::ir::Value, AggregateOccurrenceId>,
}

/// **The compilation's event-to-record relation `R`.**
///
/// Each emitted body opens a fresh local set bound to its `FuncId`; the checked
/// wrapper records one pair per allocation; the set is committed after the
/// function is finalized and verified and **before** `define_function`. The
/// whole-pass closeout then states the relation's laws once.
#[derive(Clone, Debug, Default)]
pub(in crate::cranelift_backend) struct AggregateAllocationLedger {
    local: Option<LocalAggregateEvents>,
    /// `E` over the compilation, appended at each body commit.
    committed_events: BTreeSet<AggregateAllocationEvent>,
    /// `R` over the compilation.
    committed: BTreeMap<AggregateAllocationEvent, AggregateOccurrenceId>,
    /// Bodies whose event set was opened, and those whose commit landed. The
    /// two are compared at the close, so a discarded commit cannot pass as a
    /// body that simply allocated nothing.
    opened_functions: BTreeSet<FuncId>,
    committed_functions: BTreeSet<FuncId>,
}

impl AggregateAllocationLedger {
    /// Open a fresh local set for one body.
    ///
    /// ⛔ A second build of one `FuncId` rejects. There is no
    /// rollback-and-continue: a body that is built twice has emitted its
    /// allocations twice, and the relation cannot say which emission the
    /// records govern.
    fn open(&mut self, function: FuncId) -> Result<(), CraneliftBackendError> {
        if self.committed_functions.contains(&function) {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {function} is built a second time, so \
                 its events would be recorded twice"
            )));
        }
        if let Some(open) = &self.local {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {} is still open while {function} \
                 starts, so an allocation could be attributed to the wrong body",
                open.function
            )));
        }
        if !self.opened_functions.insert(function) {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {function} opens a second time"
            )));
        }
        self.local = Some(LocalAggregateEvents {
            function,
            events: BTreeSet::new(),
            relation: BTreeMap::new(),
        });
        Ok(())
    }

    /// The open body, checked against the function the caller believes is open.
    fn open_body(
        &mut self,
        function: FuncId,
    ) -> Result<&mut LocalAggregateEvents, CraneliftBackendError> {
        let local = self.local.as_mut().ok_or_else(|| {
            backend_module(
                "aggregate allocation ledger: an allocation was emitted with no open body, so \
                 it belongs to no function's event set"
                    .to_string(),
            )
        })?;
        if local.function != function {
            return Err(backend_module(format!(
                "aggregate allocation ledger: an allocation in function {function} was recorded \
                 while {} is open",
                local.function
            )));
        }
        Ok(local)
    }

    /// Record that a governed allocation happened. This is `E`, and it is taken
    /// from the allocation itself — **never** derived from relation keys.
    fn record_event(
        &mut self,
        function: FuncId,
        result: cranelift_codegen::ir::Value,
    ) -> Result<(), CraneliftBackendError> {
        let local = self.open_body(function)?;
        if !local.events.insert(result) {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {function} value {result} is already an \
                 event, so one raw allocation produced two"
            )));
        }
        Ok(())
    }

    /// Relate one event to the record that governs it.
    fn relate(
        &mut self,
        function: FuncId,
        result: cranelift_codegen::ir::Value,
        occurrence: AggregateOccurrenceId,
    ) -> Result<(), CraneliftBackendError> {
        let local = self.open_body(function)?;
        match local.relation.insert(result, occurrence) {
            // ⛔ Both a duplicate and a conflict reject. One raw allocation
            // yields one result value, so a second pair at that value means
            // either the wrapper ran twice for one allocation or two
            // allocations share a result -- and neither can be reconciled.
            Some(previous) => Err(backend_module(format!(
                "aggregate allocation ledger: function {function} value {result} already maps to \
                 {previous:?}, so a second pair to {occurrence:?} is a duplicate or a conflict"
            ))),
            None => Ok(()),
        }
    }

    /// Commit the open body's pairs into the compilation relation.
    fn commit(&mut self) -> Result<(), CraneliftBackendError> {
        let local = self.local.take().ok_or_else(|| {
            backend_module(
                "aggregate allocation ledger: a body commit ran with no open event set"
                    .to_string(),
            )
        })?;
        if !self.committed_functions.insert(local.function) {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {} commits a second time",
                local.function
            )));
        }
        // ⛔ `dom(R) = E` at the LOCAL close, before anything is committed.
        // An event with no relation entry is an allocation nothing authorized;
        // a relation entry with no event is an authorization nothing allocated.
        // Both are refusals, and neither is visible from one side alone.
        let related = local.relation.keys().copied().collect::<BTreeSet<_>>();
        if related != local.events {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {} has {} events and {} relation keys, \
                 so dom(R) is not E",
                local.function,
                local.events.len(),
                related.len()
            )));
        }
        // Both evidences are appended, so the whole-pass close can restate the
        // same law over the compilation rather than trusting each body's.
        for result in local.events {
            self.committed_events.insert(AggregateAllocationEvent {
                function: local.function,
                result,
            });
        }
        for (result, occurrence) in local.relation {
            let event = AggregateAllocationEvent {
                function: local.function,
                result,
            };
            if let Some(previous) = self.committed.insert(event, occurrence) {
                return Err(backend_module(format!(
                    "aggregate allocation ledger: {event:?} already maps to {previous:?}, so a \
                     committed pair is not unique"
                )));
            }
        }
        Ok(())
    }

    /// Drop the open body's evidence without committing it, for the
    /// discarded-commit control. There is no production path that does this:
    /// a body either commits or the pass fails.
    #[cfg(test)]
    fn discard_open_body_for_tests(&mut self) {
        self.local = None;
    }

    /// Clear committed relation entries while leaving event evidence, for the
    /// cleared-relation control. No production path does this either.
    #[cfg(test)]
    fn clear_committed_relation_for_tests(&mut self) {
        self.committed.clear();
    }

    /// **Close the relation once, after every body is emitted.**
    ///
    /// ⛔ The laws are stated over the WHOLE compilation, never per function.
    /// One record may govern many function-local events — a synthesized role at
    /// a seat reached under both a predeclared unit and a generated
    /// specialization allocates in both bodies — so `image(R_f) = P` is false
    /// for every individual `f` and imposing it would refuse lawful programs.
    fn close(
        &mut self,
        planned: &[PlannedAggregateOwnership],
    ) -> Result<AggregateRelationClosure, CraneliftBackendError> {
        if let Some(open) = &self.local {
            return Err(backend_module(format!(
                "aggregate allocation ledger: function {} is still open at the whole-pass \
                 closeout, so its events were never committed",
                open.function
            )));
        }
        // ⭐ Every body that OPENED must have COMMITTED. A discarded commit
        // leaves its events uncommitted, and without this the artifact would
        // look like a body that simply allocated nothing.
        if self.opened_functions != self.committed_functions {
            return Err(backend_module(format!(
                "aggregate allocation ledger: {} bodies opened but {} committed, so a body's \
                 events were discarded",
                self.opened_functions.len(),
                self.committed_functions.len()
            )));
        }
        // ⭐ `dom(R) = E` over the whole compilation, restated from the two
        // independently accumulated evidences rather than trusted from the
        // per-body closes. Clearing committed relation entries between bodies
        // leaves the event evidence behind, and only this comparison sees it.
        let related = self.committed.keys().copied().collect::<BTreeSet<_>>();
        if related != self.committed_events {
            return Err(backend_module(format!(
                "aggregate allocation ledger: the compilation has {} events and {} relation \
                 keys, so dom(R) is not E",
                self.committed_events.len(),
                related.len()
            )));
        }
        let population = planned
            .iter()
            .map(|record| record.id)
            .collect::<BTreeSet<_>>();
        let image = self
            .committed
            .values()
            .copied()
            .collect::<BTreeSet<_>>();
        // ⭐ `image(R) ⊆ P`, and deliberately NOT equality.
        //
        // `P` is a closed AUTHORIZATION population, not an execution
        // obligation: it plans a record for every allocation-reachable node of
        // every seat's tree under every emission owner the seat may be lowered
        // by, while one compilation emits only the bodies it has. An unused
        // record is lawful. Measured before this was ruled: requiring equality
        // refused ordinary programs by 1 to 132 records.
        for occurrence in &image {
            if !population.contains(occurrence) {
                return Err(backend_module(format!(
                    "aggregate allocation ledger: {occurrence:?} is related by an event but is \
                     not in the planned population"
                )));
            }
        }
        Ok(AggregateRelationClosure {
            events: self.committed_events.len(),
            image: image.len(),
            population: population.len(),
            unused: population.difference(&image).count(),
        })
    }
}

/// What the whole-pass closeout measured.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct AggregateRelationClosure {
    pub(in crate::cranelift_backend) events: usize,
    pub(in crate::cranelift_backend) image: usize,
    pub(in crate::cranelift_backend) population: usize,
    /// Planned records no event related. **Lawful** — `P` authorizes, it does
    /// not oblige. Retained as a measurement, never as a failure condition.
    pub(in crate::cranelift_backend) unused: usize,
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — the claim-group identity.**
///
/// ⛔ **Its own module with a private field, so lowering cannot CONSTRUCT one.**
/// But construction is not where the guarantee lives — a fresh id minted out of
/// band names no open group, and every operation below requires an id the
/// ledger itself opened and is still holding. ⇒ The closure is REGISTRATION,
/// not opacity; opacity only removes the shortcut.
mod effect_seat_group {
    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub(in crate::cranelift_backend) struct EffectSeatGroupId(u64);

    /// Mint the next identity. ⛔ Callable only with the ledger's own counter.
    pub(super) fn mint(counter: &mut u64) -> EffectSeatGroupId {
        *counter += 1;
        EffectSeatGroupId(*counter)
    }
}
use effect_seat_group::EffectSeatGroupId;

/// One seat, as the emitter actually found it.
///
/// ⭐ **The observed phase is RETAINED, not checked and discarded.** Checking it
/// against `Avail` and then dropping it leaves the ledger unable to say what the
/// emitter saw — so a later reader has only the planner's admissible set, and
/// the one fact that distinguishes a specialized read from a carried one is
/// gone at exactly the point a reviewer would ask for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClaimedEffectSeat {
    record: PlannedEffectSeat,
    observed: EffectSeatPhase,
}

/// One compiler-side lowering VISIT to one effect occurrence.
///
/// ⭐ **The group is the unit of completeness, and that is the whole point.** A
/// ledger that accumulated claims per (body, occurrence) across visits would
/// accept two visits that each read half the seats, because their union is
/// complete — and two half-reads are exactly the defect. Completeness is asked
/// of each visit alone.
#[derive(Clone, Debug)]
struct OpenEffectSeatGroup {
    id: EffectSeatGroupId,
    function: FuncId,
    effect_origin: StaticOriginId,
    operation: ken_host::HostOpV1,
    /// The occurrence's planned slot population, bound at open so a later
    /// change to the plan cannot move the target the group closes against.
    planned: BTreeSet<EffectSeatSlot>,
    claims: BTreeMap<EffectSeatSlot, ClaimedEffectSeat>,
}

/// A visit that closed complete.
#[derive(Clone, Debug)]
struct CommittedEffectSeatGroup {
    function: FuncId,
    effect_origin: StaticOriginId,
    claims: BTreeMap<EffectSeatSlot, ClaimedEffectSeat>,
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — what the emitter ACTUALLY consumed, per
/// visit.**
///
/// ⭐ This is the independent second side of the seat authority. The planner
/// derives a population of seats and the admissible phases of each; this records
/// which seats a concrete visit reached and the phase it actually found them in.
/// A single structure holding both would make the agreement true by construction.
#[derive(Clone, Debug, Default)]
pub(in crate::cranelift_backend) struct EffectSeatLedger {
    next_group: u64,
    /// ⛔ At most ONE group open at a time. An effect's operands are lowered
    /// before its group opens, so a second open means a visit began inside
    /// another visit's window and their claims could interleave.
    open: Option<OpenEffectSeatGroup>,
    /// ⛔ Every opened group, keyed to the EXACT body it was opened for. A bare
    /// set of ids cannot answer "was every group this body opened committed
    /// before this body was defined" -- it can only answer that question over
    /// the whole compilation, which is too late: the body is already in the
    /// module. The `FuncId` is what makes the question askable per body.
    opened: BTreeMap<EffectSeatGroupId, FuncId>,
    committed: BTreeMap<EffectSeatGroupId, CommittedEffectSeatGroup>,
}

/// What the whole-pass seat closeout measured.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct EffectSeatClosure {
    /// Visits that closed complete.
    pub(in crate::cranelift_backend) groups: usize,
    /// Claims across all visits.
    pub(in crate::cranelift_backend) claims: usize,
    /// Distinct planned seats some visit reached — `image(claims)`.
    pub(in crate::cranelift_backend) image: usize,
    pub(in crate::cranelift_backend) population: usize,
    /// Members of `P` no visit reached. **Lawful** — `P` authorizes, it does not
    /// oblige, exactly as the aggregate relation's `P` does. A declaration body
    /// this compilation never emitted takes its occurrence's seats with it.
    /// Reported, never a failure condition.
    pub(in crate::cranelift_backend) unreached: usize,
}

impl EffectSeatLedger {
    /// Open the group for one visit, BEFORE any seat of it is observed.
    fn open_group(
        &mut self,
        function: FuncId,
        effect_origin: StaticOriginId,
        operation: ken_host::HostOpV1,
        planned: BTreeSet<EffectSeatSlot>,
    ) -> Result<EffectSeatGroupId, CraneliftBackendError> {
        if let Some(open) = &self.open {
            return Err(backend_module(format!(
                "host effect seat ledger: {:?} is still open while a visit to {effect_origin:?} \
                 starts, so a seat could be claimed into the wrong visit",
                open.effect_origin
            )));
        }
        if planned.is_empty() {
            return Err(backend_module(format!(
                "host effect seat ledger: {effect_origin:?} is visited but plans no seat at all"
            )));
        }
        let id = effect_seat_group::mint(&mut self.next_group);
        self.opened.insert(id, function);
        self.open = Some(OpenEffectSeatGroup {
            id,
            function,
            effect_origin,
            operation,
            planned,
            claims: BTreeMap::new(),
        });
        Ok(id)
    }

    /// The open group, checked against the id the caller believes it holds.
    fn open_group_mut(
        &mut self,
        group: EffectSeatGroupId,
    ) -> Result<&mut OpenEffectSeatGroup, CraneliftBackendError> {
        let open = self.open.as_mut().ok_or_else(|| {
            backend_module(
                "host effect seat ledger: a seat was claimed with no open visit, so it belongs \
                 to no group"
                    .to_string(),
            )
        })?;
        if open.id != group {
            return Err(backend_module(format!(
                "host effect seat ledger: a claim names {group:?} while {:?} is open",
                open.id
            )));
        }
        Ok(open)
    }

    /// Claim one seat into the open visit.
    fn claim(
        &mut self,
        group: EffectSeatGroupId,
        record: PlannedEffectSeat,
        observed: EffectSeatPhase,
    ) -> Result<(), CraneliftBackendError> {
        // ⛔ The contract, recomputed from the record's own operation and slot
        // and nothing else, before the record is admitted to a group. This is
        // what makes operation, ordinal and need load-bearing rather than
        // recorded and unread.
        let recomputed = host_effect_seat_contract_of(record.operation, record.slot);
        if recomputed != Some((record.semantic_operation, record.need, record.avail)) {
            return Err(backend_module(format!(
                "host effect seat ledger: {record:?} recomputes from its own operation and slot \
                 to {recomputed:?}, so its recorded contract is not the one its key names"
            )));
        }
        // ⛔ `observed ∈ Avail`, proved at the claim, of the operand in hand.
        if !record.avail.admits(observed) {
            return Err(backend_module(format!(
                "host effect seat ledger: {:?} seat {:?} of {:?} was observed as {observed:?}, \
                 which its planned availability does not admit",
                record.effect_origin, record.slot, record.operation
            )));
        }
        let open = self.open_group_mut(group)?;
        if record.effect_origin != open.effect_origin || record.operation != open.operation {
            return Err(backend_module(format!(
                "host effect seat ledger: {record:?} was claimed into the visit to {:?} {:?}, so \
                 one occurrence's seat carries another's authority",
                open.effect_origin, open.operation
            )));
        }
        if !open.planned.contains(&record.slot) {
            return Err(backend_module(format!(
                "host effect seat ledger: {:?} is not a planned slot of {:?}",
                record.slot, open.effect_origin
            )));
        }
        if let Some(previous) = open.claims.insert(record.slot, ClaimedEffectSeat { record, observed })
        {
            return Err(backend_module(format!(
                "host effect seat ledger: {:?} of {:?} is claimed twice in one visit (first as \
                 {previous:?})",
                record.slot, open.effect_origin
            )));
        }
        Ok(())
    }

    /// Close the visit, before host dispatch or any successful exit.
    ///
    /// ⛔ Group-local slot EQUALITY. Not "at least the ones it read", and not
    /// accumulated with any other visit.
    fn close_group(&mut self, group: EffectSeatGroupId) -> Result<(), CraneliftBackendError> {
        let open = self.open_group_mut(group)?.clone();
        let claimed = open.claims.keys().copied().collect::<BTreeSet<_>>();
        if claimed != open.planned {
            return Err(backend_module(format!(
                "host effect seat ledger: the visit to {:?} claimed {claimed:?} but its planned \
                 population is {:?}, so the occurrence was read incompletely",
                open.effect_origin, open.planned
            )));
        }
        self.committed.insert(
            open.id,
            CommittedEffectSeatGroup {
                function: open.function,
                effect_origin: open.effect_origin,
                claims: open.claims,
            },
        );
        self.open = None;
        Ok(())
    }

    /// Drop the open visit without closing it, for the discarded-group control.
    /// No production path does this: a visit either closes or the pass fails.
    #[cfg(test)]
    fn discard_open_group_for_tests(&mut self) {
        self.open = None;
    }

    /// Drop one committed group, leaving its `opened` entry, so the whole-pass
    /// `opened = committed` backstop can be asked whether it still fires on its
    /// own. No production path does this either.
    #[cfg(test)]
    fn drop_one_committed_group_for_tests(&mut self) {
        if let Some(id) = self.committed.keys().next().copied() {
            self.committed.remove(&id);
        }
    }

    /// **Close one BODY, before it is defined.**
    ///
    /// ⭐ **This is the gate the whole-pass close cannot be.** The whole-pass
    /// version states the same law over the compilation, but it runs after every
    /// `define_function` — so a body that discarded a visit's claims is already
    /// in the module when the contradiction is noticed. The artifact is refused
    /// either way; what changes is whether the defective body was ever defined.
    ///
    /// ⛔ Two clauses, and the second needs the `FuncId` association: no group
    /// for THIS body may still be open, and every group this body opened must be
    /// committed AND committed with this same body. A group opened here and
    /// committed under another `FuncId` would satisfy a bare id comparison.
    fn commit_body(&mut self, function: FuncId) -> Result<(), CraneliftBackendError> {
        if let Some(open) = &self.open {
            if open.function == function {
                return Err(backend_module(format!(
                    "host effect seat ledger: the visit to {:?} is still open as function \
                     {function} is defined, so its claims were never closed",
                    open.effect_origin
                )));
            }
        }
        for (id, opened_for) in &self.opened {
            if *opened_for != function {
                continue;
            }
            match self.committed.get(id) {
                Some(committed) if committed.function == function => {}
                Some(committed) => {
                    return Err(backend_module(format!(
                        "host effect seat ledger: {id:?} was opened for function {function} but \
                         committed under {}, so a visit's claims belong to a body that did not \
                         make them",
                        committed.function
                    )));
                }
                None => {
                    return Err(backend_module(format!(
                        "host effect seat ledger: {id:?} was opened for function {function} and \
                         never committed, so a visit's claims were discarded before the body was \
                         defined"
                    )));
                }
            }
        }
        Ok(())
    }

    /// **Close the whole compilation.**
    ///
    /// ⛔ Every opened group committed, and `image(claims) ⊆ P`.
    ///
    /// ⚠ **Deliberately NOT a group per member of `P`.** `P` is an
    /// authorization population — the same law the aggregate relation carries in
    /// this frame — so an unreached member is lawful and reported. It cannot
    /// hide a half-read occurrence, because completeness is a group-local
    /// equality that has already run.
    fn close(
        &mut self,
        planned: &[PlannedEffectSeat],
    ) -> Result<EffectSeatClosure, CraneliftBackendError> {
        if let Some(open) = &self.open {
            return Err(backend_module(format!(
                "host effect seat ledger: the visit to {:?} is still open at the close, so it \
                 was never committed",
                open.effect_origin
            )));
        }
        let committed = self.committed.keys().copied().collect::<BTreeSet<_>>();
        let opened = self.opened.keys().copied().collect::<BTreeSet<_>>();
        if committed != opened {
            return Err(backend_module(format!(
                "host effect seat ledger: {} visits opened but {} committed, so a visit's claims \
                 were discarded",
                opened.len(),
                committed.len()
            )));
        }
        let population = planned
            .iter()
            .map(|record| (record.effect_origin, record.slot))
            .collect::<BTreeSet<_>>();
        let mut image = BTreeSet::new();
        let mut claims = 0usize;
        for group in self.committed.values() {
            for claimed in group.claims.values() {
                claims += 1;
                let key = (claimed.record.effect_origin, claimed.record.slot);
                if !population.contains(&key) {
                    return Err(backend_module(format!(
                        "host effect seat ledger: function {} claimed {key:?}, which is not in \
                         the planned population",
                        group.function
                    )));
                }
                image.insert(key);
            }
        }
        Ok(EffectSeatClosure {
            groups: self.committed.len(),
            claims,
            image: image.len(),
            population: population.len(),
            unreached: population.difference(&image).count(),
        })
    }
}

/// **`RT-DECL-CLOSURE-PORT` `D7` — the seats of ONE visit, bound to the operands
/// they were claimed from.**
///
/// ⭐ **This is what replaces the bulk pre-operation conversion.** That
/// conversion crossed every operand to a specialized template before the
/// operation was known, so a seat that could not be read specialized failed as
/// "a host-effect operand" — a generic surface naming neither the operation nor
/// the seat. Here each arm names the exact slot it is reading, and a refusal
/// carries that seat's operation and need.
///
/// ⛔ The record comes from the claim, not from a fresh lookup. Re-resolving it
/// here would let the arm read a seat the visit never claimed, which is exactly
/// the binding the claim group exists to establish.
struct ClaimedEffectSeats<'a> {
    claimed: &'a BTreeMap<EffectSeatSlot, PlannedEffectSeat>,
    capability: Option<&'a LoweringOperand>,
    arguments: &'a [LoweringOperand],
}

impl<'a> ClaimedEffectSeats<'a> {
    /// The claimed record and the operand it was claimed from.
    fn operand(
        &self,
        slot: EffectSeatSlot,
    ) -> Result<(PlannedEffectSeat, &'a LoweringOperand), CraneliftBackendError> {
        let record = *self.claimed.get(&slot).ok_or_else(|| {
            unsupported(
                "Effect",
                format!("{slot:?} was not claimed in this visit, so nothing authorizes reading it"),
            )
        })?;
        let operand = match slot {
            EffectSeatSlot::Capability => self.capability.ok_or_else(|| {
                unsupported(
                    "Effect",
                    "the capability seat was claimed but no capability operand was lowered",
                )
            })?,
            EffectSeatSlot::Argument(ordinal) => {
                self.arguments.get(ordinal as usize).ok_or_else(|| {
                    unsupported(
                        "Effect",
                        format!("{slot:?} was claimed but no operand was lowered at it"),
                    )
                })?
            }
        };
        Ok((record, operand))
    }

    /// A view that authorizes NOTHING.
    ///
    /// ⭐ For the constructor-reconciliation tests, which build a synthesized
    /// node directly rather than through a host-effect visit. It is the honest
    /// spelling of what `&[]` used to say: this caller claimed no seat, so any
    /// declared `SiteOperand` child refuses. A tree with no site-bound child —
    /// which is every tree these tests build — asks it for nothing.
    #[cfg(test)]
    fn none() -> ClaimedEffectSeats<'static> {
        static NONE: BTreeMap<EffectSeatSlot, PlannedEffectSeat> = BTreeMap::new();
        ClaimedEffectSeats { claimed: &NONE, capability: None, arguments: &[] }
    }

    /// Read one seat's compile-time template.
    ///
    /// ⛔ **Exhaustive over the two phases, and the refusal names the SEAT.**
    /// The carried arm is not dead code that could be a wildcard: it is the arm
    /// that would fire if a seat's `Avail` were ever widened without a carried
    /// route being written for it, and it must say which seat and which need
    /// rather than "a host-effect operand".
    fn specialized(&self, slot: EffectSeatSlot) -> Result<&'a Lowered, CraneliftBackendError> {
        let (record, operand) = self.operand(slot)?;
        match operand {
            LoweringOperand::Specialized(lowered) => Ok(lowered),
            LoweringOperand::Carried(_) => Err(unsupported(
                "Effect",
                format!(
                    "seat {:?} of {:?} needs {:?}, which this release can observe only in a \
                     specialized template, but this visit holds a carried word",
                    record.slot, record.operation, record.need
                ),
            )),
        }
    }
}

#[cfg(test)]
thread_local! {
    /// `(specialized, carried)` emissions of the `BufferAllocate` capacity arm.
    static CAPACITY_PHASE_DISPATCH: std::cell::Cell<(usize, usize)> =
        const { std::cell::Cell::new((0, 0)) };
}

/// One argument to a compiler-synthesized constructor, in the FORM the tree
/// declares it.
///
/// ⭐ **The four forms are disjoint and the reconciliation matches on the pair
/// `(declared node, argument form)`.** A bare `Vec<Lowered>` cannot state which
/// form an operand is meant to be, so a site-bound child could only be checked
/// for position — and `SynthesizedAggregateNode::SiteOperand(_) => true` was
/// exactly that: arity proved the parent field position and nothing proved the
/// value in it was the operand whose lifetime justified the record. A different
/// value of the same shape and the same boundary disposition could inherit
/// operand `i`'s owner proof, which is the authority substitution `D7` exists
/// to prevent.
///
/// ⛔ Private to synthesized construction. Not a `Lowered` variant, not a
/// runtime tag, and nothing downstream sees it: the provenance is consumed by
/// the reconciliation and discarded, and the ordinary `Lowered` child is what
/// reaches the template.
enum SynthesizedArgument {
    /// A scalar the emitter materialized, for a `Scalar` node.
    Scalar(Lowered),
    /// A nested synthesized constructor, for a `Fixed` node.
    Nested(Lowered),
    /// A dynamic constructor, for a `Dynamic` node.
    Dynamic(Lowered),
    /// A value **projected from the seat's indexed operand**, for a
    /// `SiteOperand` node.
    ///
    /// All three axes are carried because all three are reconciled: the seat
    /// must be the one being lowered, the index must be the one the tree
    /// declares, and the value must still witness as the operand at that index.
    SiteOperand {
        seat: StaticOriginId,
        index: u32,
        value: Lowered,
    },
}

impl SynthesizedArgument {
    /// The `Lowered` child this argument becomes once its provenance has been
    /// reconciled and discarded.
    fn into_lowered(self) -> Lowered {
        match self {
            Self::Scalar(value) | Self::Nested(value) | Self::Dynamic(value) => value,
            Self::SiteOperand { value, .. } => value,
        }
    }

    fn lowered(&self) -> &Lowered {
        match self {
            Self::Scalar(value) | Self::Nested(value) | Self::Dynamic(value) => value,
            Self::SiteOperand { value, .. } => value,
        }
    }
}

/// What makes one lowered value distinguishable from another at a site.
///
/// ⛔ **`None` is a refusal.** A value this cannot witness is one whose identity
/// the reconciliation cannot establish, so a site-bound child holding it fails
/// rather than being accepted on the strength of its shape. The alternative —
/// a permissive fallback — would reopen the substitution for exactly the
/// variants nobody thought about.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SiteOperandWitness {
    Values(Vec<cranelift_codegen::ir::Value>),
    Bytes(Vec<u8>),
}

/// The witness of one lowered value, or `None` if it has none.
///
/// The CLIF `Value` numbering is per function, which is precisely what makes it
/// a discriminator here: two different operands of one seat, in one function
/// body, hold different values even when their shapes agree.
fn site_operand_witness(value: &Lowered) -> Option<SiteOperandWitness> {
    use SiteOperandWitness::{Bytes, Values};
    match value {
        Lowered::ResourceToken { value } | Lowered::CapabilityToken { value } => {
            Some(Values(vec![*value]))
        }
        Lowered::BorrowedNativeValue { pointer } => Some(Values(vec![*pointer])),
        Lowered::ResponseBytes { pointer, len } => Some(Values(vec![*pointer, *len])),
        Lowered::Int { value, .. } | Lowered::Bool { value, .. } => Some(Values(vec![*value])),
        // The nat wrappers carry a validated payload rather than a bare CLIF
        // value; their inner value is what distinguishes two of them.
        Lowered::BoundedNat(nat) => Some(Values(vec![nat.value])),
        Lowered::StructuralNat(nat) => Some(Values(vec![nat.value])),
        Lowered::Bytes(content) => Some(Bytes(content.clone())),
        Lowered::String(text) => Some(Bytes(text.as_bytes().to_vec())),
        _ => None,
    }
}

impl<'a> Lowering<'a> {
    /// Project the seat's operand at `index` into a site-bound argument.
    ///
    /// ⭐ The only way to build a [`SynthesizedArgument::SiteOperand`] in
    /// ordinary lowering, and it reads the visit's CLAIMED seat rather than
    /// accepting a value — so the emitter states *which operand* it means and
    /// cannot hand over a substitute by mistake.
    ///
    /// ⛔ **The sole template projection, and it is driven by an exact declared
    /// `SiteOperand(index)` use.** It used to read a dense `Vec<Lowered>` that
    /// the caller built by demanding a specialized template for *every*
    /// argument the operation has. That vector was the prohibited pre-operation
    /// bulk conversion relocated after dispatch: operation knowledge made the
    /// diagnostic narrower but did not authorize reading an unrelated seat, so
    /// `BufferAllocate`'s capacity — which no synthesized node uses — was
    /// re-read as a template here after its own arm had already consumed it,
    /// and a carried capacity was refused by a consumer that never wanted it.
    /// Demanding the template only at the seat a declared child names is what
    /// makes the projection exact-use-driven rather than dense.
    fn site_operand_argument(
        &self,
        seat: StaticOriginId,
        index: u32,
        seats: &ClaimedEffectSeats<'_>,
    ) -> Result<SynthesizedArgument, CraneliftBackendError> {
        let value = seats.specialized(EffectSeatSlot::Argument(index))?.clone();
        Ok(SynthesizedArgument::SiteOperand { seat, index, value })
    }

    fn synthesized_fixed_identity(
        &self,
        role: SynthesizedFixedConstructorRole,
    ) -> Result<ConstructorIdentity, CraneliftBackendError> {
        self.static_transition_plan
            .synthesized_constructor_identity(SynthesizedConstructorRole::Fixed(role))
    }

    /// Build one compiler-synthesized aggregate template at an exact producer
    /// seat.
    ///
    /// `seat` is the `Effect` occurrence whose lowering is making this use. It
    /// is passed explicitly rather than read from ambient state so that the
    /// occurrence this template carries is bound to the exact use — a role
    /// alone cannot select one, which is the whole point of the per-use key.
    fn synthesized_constructor(
        &self,
        seat: StaticOriginId,
        path: &SynthesizedAggregatePath,
        role: SynthesizedFixedConstructorRole,
        constructor: RuntimeSymbol,
        args: Vec<SynthesizedArgument>,
        seats: &ClaimedEffectSeats<'_>,
    ) -> Result<Lowered, CraneliftBackendError> {
        // ⚠ Every allocation-reachable use in an operation's tree HAS a record,
        // site-bound ones included -- `OptionSome`, `FileError`,
        // `PrivateBufferSpan`, `ReadSome`. None of them is lawfully unmodelled.
        // The `None` below is reached only when no context is being defined,
        // which is not an emission this population covers at all.
        // The exact `D5a` emission owner of the context doing the lowering.
        // Absent means no context is being defined, which is not an emission
        // this population covers -- so no occurrence, and the loud refusal at
        // the allocation stands rather than a borrowed owner being invented.
        let Some(owner) = self.defining_emission_owner else {
            return Ok(Lowered::Constructor {
                constructor,
                synthesized_identity: Some(self.synthesized_fixed_identity(role)?),
                occurrence: None,
                args: args.into_iter().map(SynthesizedArgument::into_lowered).collect(),
            });
        };
        // ⛔ **`?`, never `.ok()`.** With a live emission owner, every
        // allocation-reachable synthesized use HAS a record — that is the rule
        // this checkpoint closed. So a lookup that fails here is a missing or
        // wrong authority, not an absence to route around, and mapping it to
        // `None` silently skipped the child reconciliation below and emitted a
        // template that would then refuse only at its allocation.
        //
        // `None` survives on exactly one branch: the explicit
        // no-emission-owner early return above.
        // ⭐ **`D7` — the A/B seat discriminator's ONLY seam.** Under the
        // `SiblingEffectSeat` mutation this becomes a DIFFERENT live effect
        // seat running the same host operation, while the arguments and
        // operands already built for the real seat are retained unchanged. So
        // a refusal below is attributable to the seat coordinate and to
        // nothing else — not to an invalid seat, not to a different program.
        #[cfg(test)]
        let seat = self.sibling_effect_seat_under_mutation(seat);
        let occurrence = Some(self.static_transition_plan.synthesized_aggregate_occurrence(
            owner,
            seat,
            path,
            SynthesizedConstructorRole::Fixed(role),
        )?);
        // The recipe and this call site are two statements of one shape, so
        // they are cross-checked rather than trusted to agree. A recipe that
        // drifts from the code that builds the aggregate would otherwise pick
        // the lane for a different node than the one being allocated, and
        // nothing downstream could tell.
        {
            let declared = self
                .static_transition_plan
                .synthesized_aggregate_children(
                    owner,
                    seat,
                    path,
                    SynthesizedConstructorRole::Fixed(role),
                )?;
            self.reconcile_declared_children(owner, seat, path, declared, &args, seats)?;
        }
        Ok(Lowered::Constructor {
            constructor,
            synthesized_identity: Some(self.synthesized_fixed_identity(role)?),
            // `D7` — the planner's occurrence for this role, resolved here and
            // carried, exactly as a source constructor's is.
            occurrence,
            // The provenance has done its work; what the template holds is the
            // ordinary child, so nothing downstream sees a second carrier.
            args: args.into_iter().map(SynthesizedArgument::into_lowered).collect(),
        })
    }

    /// The program ROOT's occurrence in place of the callee's scheduling entry,
    /// under the self-authority probe only.
    ///
    /// ⚠ Returns the coordinate unchanged when the plan has no root, or when the
    /// root IS the callee's entry -- so a control must assert the hit count.
    ///
    /// ⛔ The root is a real, live, planned occurrence, never a fabricated one.
    /// A refusal driven by an unusable coordinate would be a claim about
    /// coordinate VALIDITY; the claim here is that a self-authorizing aggregate
    /// does not care WHICH live coordinate it crosses at.
    #[cfg(test)]
    fn callee_scheduling_origin_under_mutation(
        &self,
        origin: StaticOriginId,
    ) -> StaticOriginId {
        if GOVERNED_ALLOCATION_MUTATION.with(std::cell::Cell::get)
            != GovernedAllocationMutation::CalleeSchedulingOrigin
        {
            return origin;
        }
        let Ok(root) = self.static_transition_plan.root_static_origin() else {
            return origin;
        };
        if root == origin {
            return origin;
        }
        governed_allocation_hit();
        let used = root;
        CALLEE_SCHEDULING_ORIGIN_USED.with(|cell| cell.set(Some((origin, used))));
        used
    }

    #[cfg(not(test))]
    fn callee_scheduling_origin_under_mutation(
        &self,
        origin: StaticOriginId,
    ) -> StaticOriginId {
        origin
    }

    /// Swap in a sibling effect seat, under the A/B mutation only.
    ///
    /// ⚠ Returns the seat unchanged when no sibling exists. A control must
    /// therefore assert the HIT COUNT rather than the refusal alone: without
    /// it, "the fixture has no sibling seat so nothing was swapped" and "the
    /// swap happened and was caught" are the same green.
    #[cfg(test)]
    fn sibling_effect_seat_under_mutation(&self, seat: StaticOriginId) -> StaticOriginId {
        if GOVERNED_ALLOCATION_MUTATION.with(std::cell::Cell::get)
            != GovernedAllocationMutation::SiblingEffectSeat
        {
            return seat;
        }
        match self.static_transition_plan.sibling_effect_seat(seat) {
            Some(sibling) => {
                governed_allocation_hit();
                sibling
            }
            None => seat,
        }
    }

    /// Every operand must be the KIND the tree assumed when it took the meet.
    ///
    /// ⛔ Arity agreement is not sufficient and never was: a model that says
    /// `Scalar` where a referent-bearing child is passed has the right count
    /// and the wrong lane, and the aggregate is then allocated persistent over
    /// an operand that can be arena-owned -- the dangling parent this whole
    /// record exists to prevent.
    fn reconcile_declared_children(
        &self,
        owner: ContinuationEmissionOwner,
        seat: StaticOriginId,
        path: &SynthesizedAggregatePath,
        declared: &'static [SynthesizedAggregateNode],
        args: &[SynthesizedArgument],
        seats: &ClaimedEffectSeats<'_>,
    ) -> Result<(), CraneliftBackendError> {
        if declared.len() != args.len() {
            return Err(unsupported(
                "Constructor",
                format!(
                    "synthesized aggregate node is planned with {} children but the emitter \
                     built {}",
                    declared.len(),
                    args.len()
                ),
            ));
        }
        for (position, (child, argument)) in declared.iter().zip(args).enumerate() {
            let position = u32::try_from(position).map_err(|_| {
                unsupported(
                    "Constructor",
                    "synthesized aggregate arity exceeds the path step space",
                )
            })?;
            let agrees = match (child, argument) {
                // The EXACT planned disposition, spill class and presence
                // included -- not the broad `RepresentedImmediate` family.
                //
                // The family is not enough because it does not distinguish the
                // two owner sets the planner derived from: `spill: None` has no
                // boundary node at any magnitude, while `spill: Some(_)`
                // becomes a persistent-store handle at wide ones. Accepting any
                // immediate here would let a record justified by one of those
                // govern an operand that is the other.
                (
                    SynthesizedAggregateNode::Scalar { tag, spill },
                    SynthesizedArgument::Scalar(value),
                ) => matches!(
                    value.boundary_disposition(),
                    BoundaryDisposition::RepresentedImmediate {
                        tag: emitted_tag,
                        spill: emitted_spill,
                    } if emitted_tag == *tag && emitted_spill == *spill
                ),
                // ⭐ **A nested child's path EXTENDS its parent's.** The operand
                // must be the exact occurrence interned at
                // `path.field(position)` -- not merely a constructor of the same
                // role, and not that same role's occurrence somewhere else in
                // the tree. This is what makes the path key CHECKED rather than
                // merely declared: the emitter states where it put the child,
                // the planner states where it planned it, and the two are
                // compared. Collapse a step, drop one, or swap two, and the
                // occurrence resolved here stops matching the operand.
                //
                // ⛔ Every allocation-reachable nested child HAS a record --
                // `ReadSome`'s `PrivateBufferSpan` included, whose site-bound
                // `ResourceToken` gets an exact seat-derived one. So a failed
                // lookup here is missing authority, not a lawful absence, and
                // it propagates. The comparison is against `Some(expected)`;
                // under an `.ok()` mapping a missing record compared EQUAL to a
                // child carrying no occurrence and the pair passed on two
                // absences agreeing.
                (
                    SynthesizedAggregateNode::Fixed { role: inner, .. },
                    SynthesizedArgument::Nested(value),
                ) => {
                    // ⛔ Propagated, not mapped to `None`. Under `.ok()` a
                    // missing expected record compared EQUAL to a child
                    // carrying no occurrence, so two absences agreed and the
                    // pair passed. The comparison is against `Some(expected)`.
                    let expected = self.static_transition_plan.synthesized_aggregate_occurrence(
                        owner,
                        seat,
                        &path.field(position),
                        SynthesizedConstructorRole::Fixed(*inner),
                    )?;
                    matches!(
                        value,
                        Lowered::Constructor { occurrence, .. } if *occurrence == Some(expected)
                    )
                }
                // ⭐ **A dynamic child is checked ALTERNATIVE BY ALTERNATIVE.**
                //
                // ⛔ Not merely "is it a dynamic constructor". That weaker
                // check is what let the three `ResourceKind` uses be built at
                // ONE path with no complaint: the parent's own reconciliation
                // passed on shape, and because a `Lowered::DynamicConstructor`
                // carries no single occurrence, nothing compared the identities
                // its alternatives were holding. Three distinct allocations
                // then shared one record — exactly the aliasing the path key
                // exists to prevent, and invisible.
                //
                // The set has no occurrence, but each alternative does, and
                // each sits at `child_path.alternative(index)`. Comparing those
                // is what makes the parent-to-child path law hold through a
                // dynamic position as it already does through a fixed one.
                (
                    SynthesizedAggregateNode::Dynamic(_),
                    SynthesizedArgument::Dynamic(Lowered::DynamicConstructor(dynamic)),
                ) => self.dynamic_alternatives_agree(
                    owner,
                    seat,
                    &path.field(position),
                    dynamic,
                )?,
                // ⭐ **All three provenance axes, against the CLAIMED seat.**
                //
                // The planner derived this child's owners from the seat's
                // operand `index`. So the value here must BE that operand:
                // the seat must match, the index must be the declared one, and
                // the value must still witness as this visit's claimed
                // `Argument(index)`. Arity alone proves only the parent field
                // position, and a value of the same shape and the same boundary
                // disposition would otherwise inherit that operand's owner
                // proof.
                //
                // ⛔ **The projection happens HERE, at a declared use, and
                // nowhere else.** The two coordinate checks run first so that a
                // wrong seat or a wrong index is the ordinary child mismatch
                // below rather than a projection error about the seat the
                // emitter did not name.
                //
                // ⛔ A declared `SiteOperand` whose claimed operand is CARRIED
                // refuses at that exact seat, propagated from `specialized`. It
                // does not reconstruct a template, widen the carrier, borrow a
                // sibling, or fall back — reconciliation needs a compile-time
                // witness, and there is none.
                (
                    SynthesizedAggregateNode::SiteOperand(declared_index),
                    SynthesizedArgument::SiteOperand { seat: bound, index, value },
                ) => {
                    if *bound != seat || index != declared_index {
                        false
                    } else {
                        let projected =
                            seats.specialized(EffectSeatSlot::Argument(*declared_index))?;
                        site_operand_witness(value).is_some()
                            && site_operand_witness(value) == site_operand_witness(projected)
                    }
                }
                // ⛔ `Absent` marks a host-result arm that builds no aggregate,
                // so it is never a child of a planned record.
                (SynthesizedAggregateNode::Absent, _) => {
                    return Err(unsupported(
                        "Constructor",
                        "a synthesized aggregate is planned with an absent child, so the \
                         tree describes an allocation whose operand is not built",
                    ));
                }
                // ⛔ The FORMS ARE DISJOINT. A mismatched pair is a refusal, not
                // a fallthrough to a weaker check: passing a bare scalar where
                // the tree declares a site-bound operand is precisely the
                // substitution this typing exists to make unstateable.
                _ => false,
            };
            if !agrees {
                return Err(unsupported(
                    "Constructor",
                    format!(
                        "synthesized aggregate child {position} is planned as {child:?} but the \
                         emitter built a {}, so the meet was taken over a different node than \
                         the one being allocated",
                        lowered_value_kind(argument.lowered())
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Build one alternative of a compiler-synthesized dynamic constructor.
    ///
    /// ⭐ An alternative IS an allocation and has its own path-keyed ownership
    /// record; `emit_carrier_dynamic_constructor` takes its lane from that
    /// record rather than from the value-shape disposition. So this reconciles
    /// against the tree AND resolves the occurrence the emitter will carry: the
    /// node at `parent.alternative(position)` must be this exact role with this
    /// exact ordered child model.
    fn synthesized_dynamic_alternative(
        &self,
        seat: StaticOriginId,
        parent: &SynthesizedAggregatePath,
        position: u32,
        tag: i64,
        role: SynthesizedFixedConstructorRole,
        constructor: RuntimeSymbol,
        fields: Vec<SynthesizedArgument>,
        seats: &ClaimedEffectSeats<'_>,
    ) -> Result<DynamicConstructorAlternativeV1, CraneliftBackendError> {
        // Absent means no context is being defined, which is not an emission
        // this population covers -- the same boundary `synthesized_constructor`
        // draws, and for the same reason.
        let role = SynthesizedConstructorRole::Fixed(role);
        let occurrence = self.reconcile_dynamic_alternative(
            seat,
            parent,
            position,
            role,
            &fields,
            seats,
        )?;
        Ok(DynamicConstructorAlternativeV1 {
            tag,
            constructor,
            identity: self.static_transition_plan.synthesized_constructor_identity(role)?,
            occurrence,
            fields: fields.into_iter().map(SynthesizedArgument::into_lowered).collect(),
        })
    }

    /// Whether the dynamic alternative population at a path EQUALS the
    /// planner's.
    ///
    /// Used for a dynamic CHILD at `parent.field(i)` and for a dynamic ROOT at
    /// the bare root path. The two are the same contract at different seats,
    /// which is why they share one function rather than one being a weaker
    /// spelling of the other.
    ///
    /// ⭐ **Equality, not prefix agreement.** The expected population comes from
    /// `synthesized_dynamic_alternatives` — the planner's own ordered roles at
    /// this exact `seat + child_path` — and the emitter's cardinality is
    /// compared to it before anything else.
    ///
    /// ⛔ **The count is never inferred from the emitter's vector.** An earlier
    /// spelling iterated `dynamic.alternatives` and resolved each position. That
    /// rejects an EXTRA alternative, because its path does not exist — but a
    /// vector missing its last alternative, or an empty one, agrees with every
    /// prefix and returns true. A planner tree with two `ResourceKind`
    /// alternatives then accepted an emitter carrying only alternative 0, and
    /// the missing allocation would never surface at all.
    ///
    /// ⛔ **The earlier text here said it would surface at a future whole-pass
    /// `image(R) = P` closeout. There is no such closeout and there will not
    /// be** — `P` is an authorization population, so a record no event related
    /// is LAWFUL and the whole-pass close states `image(R) ⊆ P`. Exact
    /// construction cardinality therefore cannot defer to the ledger under any
    /// later WP: the ledger cannot tell a truncated emitter from a lawfully
    /// unused record. It has to be established here, which is what this
    /// function does.
    ///
    /// The set itself has no record — it is not an allocation. Its alternatives
    /// are, and each one's role and identity are checked at its own position.
    fn dynamic_alternatives_agree(
        &self,
        owner: ContinuationEmissionOwner,
        seat: StaticOriginId,
        child_path: &SynthesizedAggregatePath,
        dynamic: &DynamicConstructorV1,
    ) -> Result<bool, CraneliftBackendError> {
        let planned = self
            .static_transition_plan
            .synthesized_dynamic_alternatives(seat, child_path)?;
        if planned.len() != dynamic.alternatives.len() {
            return Ok(false);
        }
        for (index, (role, alternative)) in
            planned.iter().zip(&dynamic.alternatives).enumerate()
        {
            let index = u32::try_from(index).map_err(|_| {
                unsupported(
                    "DynamicConstructor",
                    "the alternative population exceeds the path step space",
                )
            })?;
            let position = child_path.alternative(index);
            // ⛔ Same fail-closed rule: missing planner authority must not
            // compare equal to an alternative carrying no occurrence.
            let expected = self.static_transition_plan.synthesized_aggregate_occurrence(
                owner,
                seat,
                &position,
                *role,
            )?;
            if alternative.occurrence != Some(expected) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// **Reconcile a host-result ROOT against the planner's tree.**
    ///
    /// ⭐ A root dynamic set is an allocation population exactly as a child one
    /// is — the resource surface at `HostResultError`, read progress at
    /// `HostResultOk`, the console `IOError` root. What makes it easy to miss
    /// is that **no node declares it**: a child is reached through its parent's
    /// ordered child model and is checked on the way, while a root is returned
    /// straight into `Lowered::HostResult` with nothing above it to compare
    /// against. The population equality had to be asked for here explicitly.
    ///
    /// ⛔ The check is BIDIRECTIONAL, and neither direction may be defaulted. A
    /// root the planner gives a dynamic set to must receive one; a root it does
    /// not must not. Treating a missing planned population as "no declaration,
    /// so nothing to check" would make an unplanned root set pass, which is the
    /// same shape as treating a short emitter vector as a prefix.
    fn reconcile_host_result_root(
        &self,
        seat: StaticOriginId,
        root: &SynthesizedAggregatePath,
        value: &Lowered,
    ) -> Result<(), CraneliftBackendError> {
        // Absent means no context is being defined, which is not an emission
        // this population covers -- the same boundary every other synthesized
        // reconciliation draws.
        let Some(owner) = self.defining_emission_owner else {
            return Ok(());
        };
        // ⛔ **`?`, never `.ok()`.** A lookup FAILURE and a lawful non-dynamic
        // root are different answers, and the planner types them apart. Merging
        // them here -- an absent or non-`Effect` seat, a walk that leaves the
        // tree, an `IOError` position outside the closed inventory, a malformed
        // population -- would make every one of those read as "the planner
        // plans no set at this root", and a non-dynamic emitted root would then
        // match the absent case and be accepted. That is the missing-authority
        // default this function's contract forbids, and no shape or truncation
        // mutation can find it: both of those keep the lookup working.
        let planned = self
            .static_transition_plan
            .synthesized_root_alternative_population(seat, root)?;
        match (planned, value) {
            (Some(_), Lowered::DynamicConstructor(dynamic)) => {
                if self.dynamic_alternatives_agree(owner, seat, root, dynamic)? {
                    Ok(())
                } else {
                    Err(unsupported(
                        "DynamicConstructor",
                        format!(
                            "the host-result root at {root:?} disagrees with the planner's \
                             closed alternative population, so an allocation at that root has \
                             no record or a record has no allocation"
                        ),
                    ))
                }
            }
            (Some(_), _) => Err(unsupported(
                "DynamicConstructor",
                format!(
                    "the planner plans a dynamic alternative set at {root:?} but the emitter \
                     built a {}",
                    lowered_value_kind(value)
                ),
            )),
            (None, Lowered::DynamicConstructor(_)) => Err(unsupported(
                "DynamicConstructor",
                format!(
                    "the emitter built a dynamic alternative set at {root:?}, where the \
                     planner plans none, so its alternatives allocate with no records"
                ),
            )),
            (None, _) => Ok(()),
        }
    }

    /// Reconcile one dynamic alternative against the tree and return its exact
    /// path-keyed occurrence.
    ///
    /// ⭐ The alternative allocates, so it needs a record — not merely a schema
    /// agreement. This resolves `owner + seat + parent.alternative(position) +
    /// role`, checks the role the tree has at that path and the ordered
    /// children the emitter built, and hands back the occurrence the emitter
    /// carries to its allocation.
    ///
    /// `None` when no context is being defined, which is the same boundary
    /// `synthesized_constructor` draws and for the same reason: that is not an
    /// emission this population covers, so the allocation refuses loudly rather
    /// than borrowing a lane.
    fn reconcile_dynamic_alternative(
        &self,
        seat: StaticOriginId,
        parent: &SynthesizedAggregatePath,
        position: u32,
        role: SynthesizedConstructorRole,
        fields: &[SynthesizedArgument],
        seats: &ClaimedEffectSeats<'_>,
    ) -> Result<Option<AggregateOccurrenceId>, CraneliftBackendError> {
        let Some(owner) = self.defining_emission_owner else {
            return Ok(None);
        };
        let path = parent.alternative(position);
        let (declared_role, declared) =
            self.static_transition_plan.synthesized_tree_node(seat, &path)?;
        if declared_role != role {
            return Err(unsupported(
                "DynamicConstructor",
                format!(
                    "alternative {position} is planned as {declared_role:?} but the emitter \
                     built {role:?}, so the path names a different node than the one being \
                     constructed"
                ),
            ));
        }
        self.reconcile_declared_children(owner, seat, &path, declared, fields, seats)?;
        Ok(Some(self.static_transition_plan.synthesized_aggregate_occurrence(
            owner,
            seat,
            &path,
            role,
        )?))
    }

    /// The closed `IOError` alternative set at one exact position in the tree.
    ///
    /// ⭐ Every alternative here is a real allocation and takes its own
    /// path-keyed record, keyed `IoError(role)`. The set is built at a `parent`
    /// path rather than a bare role because the same inventory appears three
    /// times in the measured trees — `FileError` field 2, `ResourceHostIo`
    /// field 0, `ResourceReleaseFailed` field 2 — and those are different
    /// allocations.
    fn synthesized_io_error_alternatives(
        &self,
        seat: StaticOriginId,
        parent: &SynthesizedAggregatePath,
        payload: Lowered,
        seats: &ClaimedEffectSeats<'_>,
    ) -> Result<Vec<DynamicConstructorAlternativeV1>, CraneliftBackendError> {
        let roles = self.static_transition_plan.synthesized_io_error_roles();
        if roles.len() != self.process_symbols.io_errors.len() {
            return Err(unsupported(
                "DynamicConstructor",
                "the closed IOError role inventory does not match the effect symbol population",
            ));
        }
        let last = roles.len().saturating_sub(1);
        self.process_symbols
            .io_errors
            .iter()
            .zip(roles)
            .enumerate()
            .map(|(position, (constructor, role))| {
                let role = SynthesizedConstructorRole::IoError(*role);
                let fields = (position == last)
                    .then(|| vec![SynthesizedArgument::Scalar(payload.clone())])
                    .unwrap_or_default();
                let occurrence = self.reconcile_dynamic_alternative(
                    seat,
                    parent,
                    u32::try_from(position).map_err(|_| {
                        unsupported(
                            "DynamicConstructor",
                            "the IOError alternative population exceeds the path step space",
                        )
                    })?,
                    role,
                    &fields,
                    seats,
                )?;
                Ok(DynamicConstructorAlternativeV1 {
                    tag: i64::try_from(position).map_err(|_| {
                        unsupported(
                            "DynamicConstructor",
                            "the IOError alternative population exceeds the ABI discriminator",
                        )
                    })?,
                    constructor: constructor.clone(),
                    identity: self
                        .static_transition_plan
                        .synthesized_constructor_identity(role)?,
                    occurrence,
                    fields: fields.into_iter().map(SynthesizedArgument::into_lowered).collect(),
                })
            })
            .collect()
    }
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
/// Selects the source case for one dynamic-constructor alternative.
///
/// ⭐ Returns the case's **index** alongside it: the selection is a search by
/// constructor name, and a search recovers no position. The caller needs the
/// index to derive the body's static origin positionally
///.
fn select_dynamic_constructor_case<'a>(
    cases: &'a [crate::RuntimeMatchCase],
    alternative: &DynamicConstructorAlternativeV1,
    default: &'a RuntimeTrap,
) -> Result<Result<(usize, &'a crate::RuntimeMatchCase), &'a RuntimeTrap>, CraneliftBackendError> {
    let mut selected = cases
        .iter()
        .enumerate()
        .filter(|(_, case)| case.constructor == alternative.constructor);
    let Some((index, case)) = selected.next() else {
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
    Ok(Ok((index, case)))
}
fn materialize_dynamic_constructor_env(
    alternative: &DynamicConstructorAlternativeV1,
    env: &[LoweringEnvironmentBinding],
) -> Vec<LoweringEnvironmentBinding> {
    env_with(alternative.fields.clone(), env)
}
fn console_stream_tag(value: &Lowered) -> Option<i64> {
    let Lowered::Constructor {
        constructor, args, ..
    } = value
    else {
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
    let Lowered::Constructor {
        constructor, args, ..
    } = value
    else {
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
    let Lowered::Constructor {
        constructor, args, ..
    } = value
    else {
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
    let Lowered::Constructor {
        constructor, args, ..
    } = value
    else {
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
) -> Result<Option<(usize, &'a crate::RuntimeMatchCase)>, CraneliftBackendError> {
    let Some((index, case)) = cases
        .iter()
        .enumerate()
        .find(|(_, case)| case.constructor == constructor)
    else {
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
    Ok(Some((index, case)))
}
#[derive(Clone, Copy)]
struct ComputationalEliminatorFrame<'a> {
    cases: &'a [crate::RuntimeComputationalMatchCase],
    default: &'a RuntimeTrap,
    env: &'a [LoweringEnvironmentBinding],
    /// The origin of the computational-match occurrence these cases belong to.
    /// Case *i*'s body is `child(static_origin, 1 + i)`.
    static_origin: StaticOriginId,
    retained_scrutinee_index: Option<usize>,
    deferred_constructor_case: Option<&'a DeferredConstructorCaseEnvironment<'a>>,
    provenance: RecursorFrameProvenance,
    checked_frame_id: Option<u64>,
    checked_invocation_id: Option<u64>,
    checked_invocation_source: Option<InvocationTemplateRef>,
    checked_invocation_depth: usize,
    /// **`RT-DECL-CLOSURE-PORT` `D6a` — the closed answer-route fact, carried
    /// rather than re-derived.**
    ///
    /// ⭐⭐ This field is the whole checkpoint. `SourceContinuation::
    /// ComputationalMatchScrutinee` already holds it, the **specialized** arm
    /// already reads it, and the **carried** arm used to build this frame
    /// without it — so a checked recursive answer arriving as a carried word was
    /// asked only whether its tag matched an ordinary case, and took the closed
    /// default when it did not. `D6` activation was the first execution of the
    /// functionized artifact and therefore the first time that drop could be
    /// observed at all.
    ///
    /// ⛔ **Threaded, never inferred.** It is not to be recovered from a tag, a
    /// body, a constructor spelling, a frame id, or the presence of a
    /// continuation unit. Every construction site that is not that source
    /// continuation supplies [`SourceComputationalAnswerRoute::DirectScrutinee`],
    /// which keeps the existing closed default exactly as it was.
    answer_route: SourceComputationalAnswerRoute,
}
/// **`RT-DECL-CLOSURE-PORT` `D5a` checkpoint 4 step 1 — one carried
/// invocation's RETAINED SOURCE COORDINATES.**
///
/// The exact computational-match occurrence the invocation resumes and the
/// ruled recursive position it occupies, both read straight off the invocation
/// segment that is already in scope at the call seam.
///
/// ⛔ Deliberately carries no body origin, no ABI shape and no context id.
/// Those are what the binding must NOT be keyed on, and a coordinate record
/// that cannot hold them is a stronger guarantee than a rule saying not to use
/// them.
#[derive(Clone, Copy)]
struct CarriedInvocationCoordinates {
    continuation_origin: StaticOriginId,
    recursive_position: u32,
}

impl CarriedInvocationCoordinates {
    /// Read the coordinates off the invocation segment.
    ///
    /// ⛔ Fails closed on a sibling position that does not fit the planner's
    /// width rather than truncating: a truncated position would silently select
    /// a different recursive field's binding.
    fn of(segment: &RecursorInvocationSegment) -> Result<Self, CraneliftBackendError> {
        Ok(Self {
            continuation_origin: segment.selection.static_origin,
            recursive_position: u32::try_from(segment.sibling_position).map_err(|_| {
                unsupported(
                    "ContinuationSpecialization",
                    "a carried invocation's sibling position exceeds the planner's recursive                      position width",
                )
            })?,
        })
    }
}

#[derive(Clone, Copy)]
struct OrdinaryEliminatorFrame<'a> {
    cases: &'a [crate::RuntimeMatchCase],
    default: &'a RuntimeTrap,
    env: &'a [LoweringEnvironmentBinding],
    /// The origin of the **match occurrence these cases belong to**. Case *i*'s
    /// body is `child(static_origin, 1 + i)`; see `SourceContinuation::
    /// MatchScrutinee` for why one parent origin beats a per-case vector.
    static_origin: StaticOriginId,
    retained_scrutinee_index: Option<usize>,
    deferred_constructor_case: Option<&'a DeferredConstructorCaseEnvironment<'a>>,
}
#[derive(Clone, Copy)]
struct PendingLetContinuationFrame<'a> {
    /// ⭐ The same phase-bearing edge as
    /// [`Lowered::ComputationalRecursorClosure::residual`], borrowed: a pending
    /// `Let` resumes on exactly the value the capsule it came from carries, so
    /// narrowing it back to `&Lowered` here would reintroduce the boundary one
    /// frame later.
    residual: &'a LoweringOperand,
    args: &'a [RuntimeExpr],
    /// The origin of the `Call` occurrence `args` belong to; argument *i* is
    /// `child(call_origin, 1 + i)`.
    call_origin: StaticOriginId,
    env: &'a [LoweringEnvironmentBinding],
    recursive_unit_body: Option<StaticOriginId>,
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
    outer_env: Vec<LoweringEnvironmentBinding>,
    /// The origin of the computational-match occurrence these cases came from,
    /// carried with the clone so a resumed selection can still derive a case
    /// body's origin positionally.
    static_origin: StaticOriginId,
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
    /// The declared retained-body unit for a callable recursive position.
    ///
    /// `None` is the ordinary structural-data IH: it resumes the eliminator
    /// over its carried value and accepts no source arguments.
    recursive_unit_body: Option<StaticOriginId>,
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
            recursive_unit_body: None,
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
/// Splits a recursor capsule into the value it continues on and its control
/// payload.
///
/// ⭐ **`AC-C4`: both halves of the signature are phase-bearing.** The input is
/// an operand because a capsule may itself arrive carried-adjacent; the output
/// is an operand because the residual may now be a carried word. ⛔ Every
/// invocation site must classify the returned base **exhaustively, with no
/// wildcard** — `Specialized(BoundedNat | Closure)` keep their existing
/// meanings byte-for-byte, and `Carried` installs the already-checked
/// invocation segment and resumes the same computational eliminator over that
/// word. ⛔ Never `specialized_at`, never a reconstructed `Lowered`, never back
/// through the producer.
fn decompose_computational_recursor(
    value: LoweringOperand,
) -> (
    LoweringOperand,
    Option<(ContinuationActivationId, RecursorInvocationSegment)>,
) {
    match value {
        LoweringOperand::Specialized(Lowered::ComputationalRecursorClosure {
            residual,
            activation,
            invocation,
        }) => (*residual, Some((activation, invocation))),
        // ⛔ Spelled rather than collapsed into a wildcard: a non-capsule
        // operand passes through in whichever phase it arrived, and the phase
        // set stays visibly closed.
        LoweringOperand::Specialized(value) => (LoweringOperand::Specialized(value), None),
        LoweringOperand::Carried(word) => (LoweringOperand::Carried(word), None),
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
    // ⭐ Set equality alone cannot see a PERMUTATION (`dec_s30rdnb1dvgk` item
    // 5): two header-identical frames of the same callee declaration can be
    // exchanged and still cover `expected` exactly. Record the ORDER in which
    // identities are instantiated so the exact occurrence binding — not merely
    // the set — is checked below.
    let mut instantiated_order: Vec<u64> = Vec::new();
    // ⚠ A layer carrying no checked identity is not an error *here*: a segment
    // may legitimately contain unchecked layers. It becomes one only if the
    // callee's expected templates are then not covered, which is what
    // distinguishes "this layer need not instantiate" from "this layer must
    // and cannot say which occurrence it is."
    let mut layers_without_identity = 0usize;
    let mut visit = |layer: &mut ComputationalRecursorLayer| {
        let Some(frame_id) = layer.checked_frame_id else {
            layers_without_identity += 1;
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
        instantiated_order.push(frame_id);
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
        // ⭐ Name the missing-identity cause explicitly rather than letting it
        // read as ordinary coverage drift. Since `dec_s30rdnb1dvgk` removed the
        // fingerprint recovery in `make_computational_recursor`, an absent
        // `checked_frame_id` is the expected shape of a layer whose transported
        // marker was dropped — and it must never be recovered by inference.
        if layers_without_identity > 0 {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                format!(
                    "computational invocation {:?} has {layers_without_identity} layer(s) with no checked frame identity and does not cover its expected templates: expected={expected:?} instantiated={instantiated:?} actual={actual:?}",
                    invocation.source,
                ),
            ));
        }
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "computational invocation {:?} does not carry its exact checked frame sequence: expected={expected:?} instantiated={instantiated:?} actual={actual:?}",
                invocation.source,
            ),
        ));
    }
    // ⛔ **The permutation net.** `instantiated == expected` is set equality and
    // is therefore blind to an exchange of two header-identical frames — the
    // exact hazard `AC-F1` creates by design. Requiring the *sequence* to agree
    // pins each occurrence to its own identity.
    //
    // ⭐ The expected sequence is re-derived here from the plan's
    // `semantic_position`, deliberately **not** inherited from the order
    // `erasure.rs:1149` happened to emit — an ordering claim taken from its own
    // producer cannot detect that producer drifting.
    //
    // ⚠ Direction: `finish_match` assigns `semantic_position` in checked
    // postorder, and a segment is traversed **selection first**, which is the
    // reverse of that order. `oriented_test_plan` documents exactly this — its
    // frames are postorder `p2, p1, p0` while composition visits `0, 1, 2` and
    // `oriented_segment_keeps_semantic_and_control_axes_independent` asserts
    // the resulting semantic order is `[2, 1, 0]`.
    //
    // ⚠ This uses Runtime traversal order only to VALIDATE the checked
    // plan/marker binding. It does not mint semantic order from Runtime: the
    // plan's `semantic_position` is the authority and Runtime order is the
    // thing being checked against it.
    let mut planned_visit_order = frame_templates.to_vec();
    planned_visit_order.sort_by_key(|frame_id| {
        std::cmp::Reverse(plan.frame(*frame_id).map(|frame| frame.semantic_position))
    });
    if instantiated_order != planned_visit_order {
        return Err(unsupported(
            "OrientedSubcontinuationPlanV1",
            format!(
                "computational invocation {:?} instantiates its checked frames out of their planned occurrence order: planned={planned_visit_order:?} instantiated={instantiated_order:?}",
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
                static_origin: layer.static_origin,
                retained_scrutinee_index: None,
                deferred_constructor_case: None,
                provenance: layer.provenance,
                checked_frame_id: layer.checked_frame_id,
                checked_invocation_id: layer.checked_invocation_id,
                checked_invocation_source: layer.checked_invocation_source,
                checked_invocation_depth: layer.checked_invocation_depth,
                answer_route: SourceComputationalAnswerRoute::DirectScrutinee,
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

/// **`RT-DECL-CLOSURE-PORT` `D6a` upstream half — one lowering predecessor's
/// operand paired with the route it arrived by.**
///
/// ⭐⭐ **The route is a property of the exact predecessor that supplied the
/// answer, not of the `ComputationalMatch` occurrence, its checked frame, or
/// its owner.** The census on the governed witness is what settles that:
/// `CSId(0)` and `CSId(1)` sit at the *same* continuation origin 10 and coexist
/// with the ordinary direct scrutinee of that origin, so any occurrence-global
/// projection marks the ordinary direct predecessor checked too.
///
/// ⛔ Compiler-path metadata only. It is **not** a field in the runtime word,
/// not a carrier lane, and not a runtime discriminator — nothing here reaches
/// the emitted CFG.
#[derive(Clone)]
struct RoutedAnswer {
    value: LoweringOperand,
    route: SourceComputationalAnswerRoute,
}

impl RoutedAnswer {
    /// Ordinary source evaluation **starts** here.
    fn direct(value: LoweringOperand) -> Self {
        Self {
            value,
            route: SourceComputationalAnswerRoute::DirectScrutinee,
        }
    }

    /// An exact producer **raises** it.
    ///
    /// ⛔ There are exactly two callers of this and there must not be a third
    /// without a ruling: the exact selecting recursor-layer path, and the
    /// result of an actually claimed and emitted continuation-specialization
    /// call. A static-worker call, a raw unit call, an ordinary expression
    /// result, or a merely matching continuation origin is **not** an exact
    /// producer.
    fn checked(value: LoweringOperand) -> Self {
        Self {
            value,
            route: SourceComputationalAnswerRoute::CheckedSelectedRecursor,
        }
    }

    /// Raise a frame's starting route with this predecessor's, never lower it.
    ///
    /// ⛔ The existing `SourceContinuation::ComputationalMatchScrutinee` field
    /// stays the recursor-layer producer's authority, but it is **not the sole
    /// authority** and must not overwrite a checked route carried in by an
    /// exact call result. That asymmetry is why this is a join and not an
    /// assignment.
    fn raise(self, existing: SourceComputationalAnswerRoute) -> SourceComputationalAnswerRoute {
        match (self.route, existing) {
            (SourceComputationalAnswerRoute::CheckedSelectedRecursor, _)
            | (_, SourceComputationalAnswerRoute::CheckedSelectedRecursor) => {
                SourceComputationalAnswerRoute::CheckedSelectedRecursor
            }
            _ => SourceComputationalAnswerRoute::DirectScrutinee,
        }
    }
}
impl SourceComputationalAnswerRoute {
    /// **`D6a` upstream — PRODUCER 1.** An exact recursor layer, and only the
    /// `SelectsOccurrence` role on a checked frame, supplies the checked route.
    fn for_recursor_layer(layer: &ComputationalRecursorLayer) -> Self {
        let selects_occurrence = matches!(layer.role, RecursorLayerRole::SelectsOccurrence { .. });
        let route = if layer.checked_frame_id.is_some() && selects_occurrence {
            Self::CheckedSelectedRecursor
        } else {
            Self::DirectScrutinee
        };
        #[cfg(test)]
        let route = if d6a_route_mutation() == D6aRouteMutation::DropRecursorLayerRoute
            && route == Self::CheckedSelectedRecursor
        {
            record_d6a_route_application();
            Self::DirectScrutinee
        } else {
            route
        };
        #[cfg(test)]
        record_d6a_route_event(D6aRouteEvent::RecursorLayerSupplied {
            checked_frame_id: layer.checked_frame_id,
            selects_occurrence,
            route,
        });
        route
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
        body: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
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
        then_expr: OwnedSourceOccurrence,
        else_expr: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourceContinuation<'a>>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        static_origin: StaticOriginId,
        remaining: Vec<OwnedSourceOccurrence>,
        lowered: Vec<Lowered>,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourceContinuation<'a>>,
    },
    /// ⭐ `static_origin` is the **match occurrence's own** origin, carried in
    /// the same constructor as the cloned cases. Case *i*'s body is derived from
    /// it positionally at the point of use (`child(static_origin, 1 + i)`).
    ///
    /// A parallel `Vec<StaticOriginId>` beside `cases` would be the obvious
    /// alternative and is worse: two vectors can desync, and a desync is
    /// undetectable here. One parent origin cannot.
    MatchScrutinee {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<LoweringEnvironmentBinding>,
        static_origin: StaticOriginId,
        next: Box<SourceContinuation<'a>>,
    },
    ComputationalMatchScrutinee {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<LoweringEnvironmentBinding>,
        static_origin: StaticOriginId,
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
        args: Vec<OwnedSourceOccurrence>,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourceContinuation<'a>>,
    },
    CallArgument {
        callee: SourceCallee,
        remaining: Vec<OwnedSourceOccurrence>,
        lowered: Vec<LoweringOperand>,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourceContinuation<'a>>,
    },
}

/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8e` — what a source-machine call is calling.**
///
/// ⛔ A sum rather than an operand, because a static worker **is not a value**.
/// The capsule has no value representation, so it cannot be carried in the
/// operand slot the value route uses, and widening that slot to hold it would
/// undo exactly the fail-closed property `D8d` installed it for.
///
/// ⭐ The two arms differ only in where the argument run goes once it is
/// evaluated. Argument evaluation itself is identical and happens under the
/// machine's own control and phase for both, which is why the split is at the
/// completion and not at the entry.
#[derive(Clone)]
enum SourceCallee {
    /// The pre-existing route: a lowered callee consumed by
    /// `source_call_state`.
    Value(LoweringOperand),
    /// **`D8e`** — an exact `Var` that resolved to a `D8d` target-derived
    /// binding. ⛔ Resolved once, at the `Call` occurrence, before the callee
    /// would otherwise have been evaluated as a value; there is no second
    /// lookup and no planner query on this path.
    StaticWorker {
        worker: StaticWorkerBinding,
        static_origin: StaticOriginId,
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
    join_plan: std::rc::Rc<JoinPlanToken>,
    result_origin: StaticOriginId,
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
        body: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
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
        then_expr: OwnedSourceOccurrence,
        else_expr: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourcePrefixTemplate>,
    },
    ConstructArgument {
        constructor: RuntimeSymbol,
        static_origin: StaticOriginId,
        remaining: Vec<OwnedSourceOccurrence>,
        lowered: Vec<Lowered>,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourcePrefixTemplate>,
    },
    MatchScrutinee {
        cases: Vec<crate::RuntimeMatchCase>,
        default: RuntimeTrap,
        env: Vec<LoweringEnvironmentBinding>,
        static_origin: StaticOriginId,
        next: Box<SourcePrefixTemplate>,
    },
    ComputationalMatchScrutinee {
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        env: Vec<LoweringEnvironmentBinding>,
        static_origin: StaticOriginId,
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
        args: Vec<OwnedSourceOccurrence>,
        env: Vec<LoweringEnvironmentBinding>,
        next: Box<SourcePrefixTemplate>,
    },
    CallArgument {
        callee: SourceCallee,
        remaining: Vec<OwnedSourceOccurrence>,
        lowered: Vec<LoweringOperand>,
        env: Vec<LoweringEnvironmentBinding>,
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
    /// A pending expression the machine will evaluate.
    ///
    /// ⭐ This is the state the source-machine fallback arm feeds
    /// (`core.rs:2074`, `other => …lower_expr(builder, &other, &env)`), which
    /// hands over every form the machine's own dispatcher does not handle —
    /// closures included. That is why the origin has to be here rather than in a
    /// guessed subset of the frames: the machine and the direct descent are the
    /// same population reached two ways.
    Eval {
        expr: OwnedSourceOccurrence,
        env: Vec<LoweringEnvironmentBinding>,
        control: SourceControl<'a>,
    },
    Value {
        /// **`D6a` upstream half** -- the operand **and the route it arrived
        /// by**. ⛔ Not two independent facts: the route is a property of this
        /// exact predecessor, so pairing them here is what stops a later seat
        /// from having to guess which predecessor a value came from.
        value: RoutedAnswer,
        control: SourceControl<'a>,
    },
}
enum SourceCallOutcome<'a> {
    Continue(SourceMachineState<'a>),
    Complete(LoweringOperand),
}
#[derive(Clone, Copy)]
enum DynamicConstructorContinuation<'a> {
    Ordinary {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
        env: &'a [LoweringEnvironmentBinding],
        static_origin: StaticOriginId,
    },
    Producer {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
        env: &'a [LoweringEnvironmentBinding],
        static_origin: StaticOriginId,
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
    /// The origin of the `Construct` occurrence the fields belong to. Field *i*
    /// of that constructor is its child *i*, so `trailing_fields[j]` is
    /// `child(construct_origin, selected_field + 1 + j)`.
    construct_origin: StaticOriginId,
    producer_env: &'a [LoweringEnvironmentBinding],
    outer_eliminator: EliminatorFrame<'a>,
    splice_caller: Option<&'a ActiveContinuationFrame<'a>>,
    selected_active: ActiveContinuationFrame<'a>,
}
/// **`D8m`** — the four checked facts a source `ComputationalMatch` frame
/// carries, kept together so no site can supply three of them.
#[derive(Clone, Copy)]
struct CheckedComputationalFrame {
    id: Option<u64>,
    invocation_id: Option<u64>,
    invocation_source: Option<InvocationTemplateRef>,
    invocation_depth: usize,
}

#[derive(Clone, Copy)]
/// **`RT-CONTSRC-PRODUCER-LOCAL` `D8m` — the closed bridge descriptor.**
///
/// Exactly three admissible shapes for the case body an
/// `immediate_binder_eliminator` bridge is built from, and no fourth:
///
/// 1. a direct [`RuntimeExpr::ComputationalMatch`];
/// 2. a direct ordinary [`RuntimeExpr::Match`];
/// 3. exactly `CheckedSubcontinuationFrame { frame_id, body: ComputationalMatch }`.
///
/// ⭐⭐ **The third exists because the bridge is an OPTIMIZATION of the source
/// match, not a new semantic frame.** The source declared one checked frame
/// there; deforesting the producer into it does not create a second frame and
/// must not lose the first. Before `D8m` the bridge always carried
/// `checked_frame_id: None`, so a checked IH slot inside a composed case body
/// refused as "detached from its checked frame" — the `D8f` hard stop.
///
/// ⛔ **Nothing here mints, borrows or infers a frame identity.** The id is the
/// one the source marker carries and is reached only by matching that exact
/// shape. There is deliberately no fingerprint lookup, no body-shape match, no
/// origin coincidence, no "the only frame in the plan", and no generic wrapper
/// peeling: a marker around anything but a `ComputationalMatch`, or any other
/// checked wrapper kind, simply is not a bridge.
enum ImmediateBinderEliminator<'a> {
    Computational {
        cases: &'a [crate::RuntimeComputationalMatchCase],
        default: &'a RuntimeTrap,
    },
    Ordinary {
        cases: &'a [crate::RuntimeMatchCase],
        default: &'a RuntimeTrap,
    },
    /// The wrapped form. ⛔ The match's own occurrence is **child 0 of the
    /// marker occurrence**, never the wrapper's origin — the wrapper is not the
    /// frame, it names it.
    CheckedComputational {
        frame_id: u64,
        cases: &'a [crate::RuntimeComputationalMatchCase],
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
        // `D8m` — the EXACT wrapped shape, and only it. ⛔ Not a loop, not a
        // helper that strips any checked wrapper: a `CheckedRecursiveInvocation`
        // or a `CheckedJoinSite` around a match is a different construct with a
        // different consumption law, and peeling it here would silently give the
        // bridge an identity nobody transported for it.
        RuntimeExpr::CheckedSubcontinuationFrame { frame_id, body } => {
            let RuntimeExpr::ComputationalMatch {
                scrutinee,
                cases,
                default,
            } = body.as_ref()
            else {
                return None;
            };
            (
                scrutinee.as_ref(),
                ImmediateBinderEliminator::CheckedComputational {
                    frame_id: *frame_id,
                    cases,
                    default,
                },
            )
        }
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
    env: &[LoweringEnvironmentBinding],
    introduced_binders: usize,
) -> bool {
    let recursive_hypotheses = env
        .iter()
        .enumerate()
        .filter_map(|(index, binding)| {
            // Exhaustive with no wildcard. A static worker is a lexical
            // callable, never a computational recursor closure, so it is not a
            // recursive hypothesis -- this is a classification, not a
            // value-producing read, so it answers rather than fails closed.
            let is_recursor = match binding {
                LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(
                    Lowered::ComputationalRecursorClosure { .. },
                )) => true,
                LoweringEnvironmentBinding::Value(_) => false,
                LoweringEnvironmentBinding::StaticWorker(_) => false,
            };
            is_recursor.then_some(index + introduced_binders)
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
/// Selects an ordinary case by constructor, **with its index**.
///
/// The index is what makes the selected body's origin derivable: the search
/// itself recovers no position.
fn select_ordinary_case<'a>(
    eliminator: OrdinaryEliminatorFrame<'a>,
    constructor: &str,
) -> Result<(usize, &'a crate::RuntimeMatchCase), RuntimeTrap> {
    eliminator
        .cases
        .iter()
        .enumerate()
        .find(|(_, case)| case.constructor == constructor)
        .ok_or_else(|| eliminator.default.clone())
}
/// Selects a computational case by constructor, **with its index**, plus the
/// remaining frames.
///
/// The index is load-bearing: the selected body's origin is `child(the frame's
/// `static_origin`, 1 + index)`, and the search alone recovers no position.
fn select_computational_case<'frames, 'data>(
    eliminators: &'frames [ComputationalEliminatorFrame<'data>],
    constructor: &str,
) -> Result<
    (
        usize,
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
        .enumerate()
        .find(|(_, case)| case.constructor == constructor)
        .map(|(index, case)| (index, case, &eliminators[1..]))
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

    /// **`RT-DECL-CLOSURE-PORT` `D5a` — the marker denotes the COMPLETE
    /// APPLICATION OCCURRENCE.**
    ///
    /// ⭐ That is what `CheckedComputationalIHCallTemplateV1::arity` is *about*:
    /// a call template describes one complete application, so the marker's
    /// wrapped expression must be exactly one `RuntimeExpr::Call` of that
    /// arity. Entry required only that a template exist, which admitted a marker
    /// wrapping anything at all -- and then the only place that could notice was
    /// closeout, on the produced *value*, long after emission.
    ///
    /// ⛔ Checked at ENTRY, so a marker that does not denote an application is
    /// refused before a single instruction is emitted for it.
    fn enter_checked_computational_ih_invocation(
        &mut self,
        call_template_id: u64,
        body: &RuntimeExpr,
        // `D8f` — the occurrence of the application this marker denotes,
        // supplied by the caller from the same `child_origin(marker, 0)` it
        // already derives to lower the body. ⛔ Not recomputed here: a second
        // derivation could disagree with the one the lowering actually uses,
        // and then the marker would be pending against an occurrence nobody
        // visits.
        application_origin: StaticOriginId,
    ) -> Result<(), CraneliftBackendError> {
        if self
            .pending_computational_ih_call
            .replace(PendingCheckedIhCall {
                call_template_id,
                application_origin,
            })
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
        let call = plan
            .computational_ih_call(call_template_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH invocation marker has no checked call template",
                )
            })?;
        let RuntimeExpr::Call { args, .. } = body else {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation marker does not wrap a complete application",
            ));
        };
        let supplied = u64::try_from(args.len()).map_err(|_| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "computational IH invocation argument count exceeds addressable range",
            )
        })?;
        #[cfg(test)]
        let relaxed =
            D5A_MARKER_MUTATION.with(std::cell::Cell::get) == D5aMarkerMutation::RelaxEntryArity;
        #[cfg(not(test))]
        let relaxed = false;
        if !relaxed && supplied != call.arity {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                format!(
                    "computational IH invocation marker wraps a call of {supplied} arguments but \
                     its checked template names arity {}",
                    call.arity
                ),
            ));
        }
        Ok(())
    }

    /// **`RT-DECL-CLOSURE-PORT` `D5a` — consume the checked-IH marker at the
    /// exact static-worker call edge.**
    ///
    /// ⭐⭐ **The marker denotes the application, not the applied value.** On the
    /// ported route the induction hypothesis *is* an emitted call, so its result
    /// is a boundary word and no compile-time recursor-closure template exists
    /// for closeout to read. Consuming here -- **before emission** -- is what
    /// makes the marker mean the same thing on both routes; consuming on the
    /// returned word would be a carrier decode, which is exactly what `§2h`
    /// forbids.
    ///
    /// Three independent identities must agree, and all three are read from the
    /// checked plan rather than from the operand:
    ///
    /// | requirement | what a mismatch would mean |
    /// |---|---|
    /// | the pending template resolves, and its slot template resolves | the marker names a plan this lowering does not hold |
    /// | `call.arity == args.len()` | the emitted call is not the application the template describes |
    /// | `slot.method_binder_ordinal == Var index` | the callee is not the binder the checked plan seated the IH at |
    ///
    /// ⭐ The third is an **independent oracle for the binder run**. The slot
    /// template carries `recursive_position` and `method_binder_ordinal` as
    /// *separate* fields -- the constructor source coordinate and the lexical
    /// one -- so a lowering that conflated them (as this checkpoint's
    /// predecessor did) disagrees with the plan here and refuses.
    ///
    /// ⛔ Returns `Ok(false)` when no marker is pending: an ordinary static
    /// worker call is untouched by this seam.
    fn consume_checked_ih_marker_at_static_worker_call(
        &mut self,
        binder_index: u64,
        supplied_arguments: usize,
        // `D8f` — the occurrence of the call being lowered.
        static_origin: StaticOriginId,
    ) -> Result<bool, CraneliftBackendError> {
        #[cfg(test)]
        if D5A_MARKER_MUTATION.with(std::cell::Cell::get) == D5aMarkerMutation::SuppressConsumption
        {
            // ⛔ The call below is still emitted, lawfully and unchanged; only
            // the consumption is withheld. That is the whole point — closeout
            // must notice a real application that no consumption accounts for.
            return Ok(false);
        }
        let Some(pending) = self.pending_computational_ih_call else {
            return Ok(false);
        };
        // ⭐⭐ `D8f` — OCCUPANCY. A pending marker does not mean "the next
        // static-worker call consumes it".
        //
        // Inside one checked wrapper the arguments of the wrapped application
        // are evaluated before the application itself, and an argument can be
        // an ordinary call on the selected recursive argument. That call
        // reaches this seat with a marker pending and must leave it pending:
        // it is not the occurrence the plan issued the marker for.
        //
        // ⛔ `Ok(false)`, not a refusal, and the difference is the checkpoint.
        // An ordinary call is *untouched* by this seam -- exactly as it is when
        // no marker is pending at all -- so the marker survives for the
        // occurrence that owns it. Refusing here would make a lawful program
        // fail; consuming here would attribute the checked application to a
        // call the planner never issued a template for.
        //
        // ⛔ The comparison is on the OCCURRENCE and on nothing else. Route,
        // arity, binder index, first-call order and callee shape are all
        // properties an ordinary selected-argument call can share with the
        // checked one -- they are the same worker at the same arity in the same
        // frame -- so every one of them is blind here by construction. The
        // arity and binder-ordinal agreements below stay, but as checks on the
        // call that has already been identified, never as the identification.
        // ⚠⚠ **UNWITNESSED, and measured to be so.** Mutating this comparison
        // to never admit reds four `D5a` rows, so the gate is on the live path
        // and its answer decides consumption. Mutating it to ALWAYS admit --
        // the pre-`D8f` behaviour -- leaves the whole suite green. ⇒ No landed
        // fixture poses the occupancy question, and this mechanism is
        // correct-and-unwitnessed rather than proved.
        //
        // ⛔⛔ **BOTH ROUTES TO A WITNESS ARE CLOSED, measured, and the second
        // is structural.** This is a hard stop, not an unfinished fixture.
        //
        // Route 1 -- a NON-COMPOSED checked wrapper (`px8tr`). An ordinary call
        // on the selected recursive argument refuses in the DECLARATION's own
        // lowering ("a source-machine call's callee is a specialized-only
        // surface"): that case body is lowered both there and in the
        // specialization, and the binder run's static workers exist only in the
        // second. The same shape with the induction hypothesis as the inner
        // callee refuses in plan validation ("oriented segment mixes checked and
        // inferred computational frames").
        //
        // Route 2 -- a COMPOSED checked wrapper, where both lowerings DO carry a
        // static worker at the callee position. Transport validation passes:
        // frame marker, slot and call templates, occurrence paths and
        // fingerprints all reconcile. Lowering then refuses with
        // **"computational IH slot marker is detached from its checked frame"**
        // (`computational_ih_slots_for_case`, which requires `checked_frame_id`
        // to be `Some`).
        //
        // ⭐ That refusal is structural and cannot be fixtured around. The frame
        // in force inside a composed case body is the BRIDGE, synthesized by
        // `immediate_binder_eliminator` with `checked_frame_id: None` -- it has
        // no source expression, so no `CheckedSubcontinuationFrame` marker can
        // ever supply it one. A checked IH wrapper inside a composed bridge case
        // body therefore needs a way for a lowering-synthesized frame to carry a
        // checked frame identity, which is planner/marker mechanism this
        // checkpoint must not add.
        //
        // ⇒ The occupancy property is correct and unwitnessable at this
        // checkpoint. The attempt is preserved rather than discarded.
        if static_origin != pending.application_origin {
            return Ok(false);
        }
        let call_template_id = pending.call_template_id;
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
        let slot = plan
            .computational_ih_slot(call.slot_template_id)
            .ok_or_else(|| {
                unsupported(
                    "OrientedSubcontinuationPlanV1",
                    "computational IH call template names a slot template the plan does not hold",
                )
            })?;
        let supplied = u64::try_from(supplied_arguments).map_err(|_| {
            unsupported(
                "OrientedSubcontinuationPlanV1",
                "static worker call argument count exceeds addressable range",
            )
        })?;
        if supplied != call.arity {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                format!(
                    "the checked computational-IH call template names arity {} but the static \
                     worker call applies {supplied} arguments",
                    call.arity
                ),
            ));
        }
        if slot.method_binder_ordinal != binder_index {
            return Err(unsupported(
                "OrientedSubcontinuationPlanV1",
                format!(
                    "the checked computational-IH slot seats its method binder at ordinal {} but \
                     the consuming call reads `Var({binder_index})`",
                    slot.method_binder_ordinal
                ),
            ));
        }
        // ⛔ Taken LAST. Every refusal above leaves the marker pending, so a
        // rejected consumption still reaches closeout's fail-closed arm rather
        // than silently becoming an unmarked call.
        self.pending_computational_ih_call = None;
        #[cfg(test)]
        record_d5a_marker_event(D5aMarkerEvent::Consumed {
            call_template_id,
            slot_template_id: call.slot_template_id,
            binder_index,
            arity: call.arity,
        });
        Ok(true)
    }

    fn mint_checked_computational_ih_instance(
        &mut self,
        value: &mut Lowered,
    ) -> Result<Option<CheckedRecursiveInvocationInstance>, CraneliftBackendError> {
        let Some(pending) = self.pending_computational_ih_call.take() else {
            return Ok(None);
        };
        let call_template_id = pending.call_template_id;
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
        value: LoweringOperand,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        // **`RT-DECL-CLOSURE-PORT` `D5a` — closeout is keyed on WHETHER THE
        // MARKER WAS CONSUMED, not on what the body produced.**
        //
        // ⭐ When it was already consumed -- at the exact static-worker call
        // edge, or by a recursor-closure callee -- the marker's obligations are
        // discharged and this operand is simply the application's result. It is
        // forwarded **unchanged, including `Carried`**: the marker denotes the
        // application, so once the application is bound there is nothing left
        // here to read a template for. ⛔ Refusing a `Carried` here would refuse
        // the ported route's *result* for a template it was never supposed to
        // carry.
        if self.pending_computational_ih_call.is_none() {
            return Ok(value);
        }
        // ⭐ A still-PENDING marker keeps the specialized-template path exactly
        // as it was: the marker consumes a **recursor closure template**, a
        // carried boundary word is not one and never becomes one, so this stays
        // a specialized-only surface with the ruled fail-closed arm.
        let mut value = value.specialized_at("a checked computational-IH marker")?;
        let Some(instance) = self.mint_checked_computational_ih_instance(&mut value)? else {
            return Ok(LoweringOperand::Specialized(value));
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
        Ok(LoweringOperand::Specialized(value))
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
        // `D8n` — the observation, at the REAL consumption seam and written from
        // production state that is in hand here: the pair the ledger just
        // accepted, and the defining `FuncId` of the function being emitted. ⛔ Nothing is reconstructed from a fixture or looked up in an
        // expected plan; a reader that rebuilt either side would agree with
        // itself and say nothing about what production did.
        #[cfg(test)]
        record_d8n_frame_consumption(self.defining_function_id, invocation_id, frame_id);
        Ok(Some(frame_id))
    }

    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8m` — the checked computational frame
    /// tuple, derived ONCE.**
    ///
    /// ⭐⭐ Every consumer of a source `ComputationalMatch` needs the same four
    /// facts, and they are not independent: the invocation id exists only when
    /// a frame id does, and the source and depth both come from the innermost
    /// active recursive invocation. Deriving them at each site is how they
    /// drift.
    ///
    /// ⛔ `D8m`'s first draft hard-coded three of the four at the checked bridge
    /// -- `checked_invocation_id: None`, `checked_invocation_source: None`,
    /// `checked_invocation_depth: 0` -- which made the bridge's tuple a
    /// different object from the one the direct path builds for the SAME source
    /// match. Sharing the derivation is what makes "the bridge is an
    /// optimization of the source match" true of the whole tuple rather than of
    /// its first field.
    ///
    /// ⛔ This is not a validator. It consumes the marker through the existing
    /// entry/consumption pair and reads the invocation stack; it checks nothing
    /// the callers did not already have checked for them.
    fn checked_computational_frame(
        &mut self,
        cases: &[crate::RuntimeComputationalMatchCase],
        default: &RuntimeTrap,
    ) -> Result<CheckedComputationalFrame, CraneliftBackendError> {
        let id = self.consume_checked_subcontinuation_frame(cases, default)?;
        Ok(CheckedComputationalFrame {
            id,
            invocation_id: id.map(|_| {
                self.active_recursive_invocations
                    .last()
                    .map_or(0, |instance| instance.invocation_instance_id)
            }),
            invocation_source: self
                .active_recursive_invocations
                .last()
                .map(|instance| instance.source),
            invocation_depth: self
                .active_recursive_invocations
                .last()
                .map_or(0, |instance| instance.semantic_depth),
        })
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
                // `D8n` — the observation at the REAL slot-reconciliation seam,
                // recorded only once the plan-named slot has been resolved and
                // held to its frame, constructor and position. The slot id is
                // the plan's own; the identity is the defining `FuncId`.
                #[cfg(test)]
                record_d8n_slot_reconciliation(self.defining_function_id, slot.slot_template_id);
                // `D8m` — the same seam, recording the PAIR: the frame the
                // bridge transported here beside the slot the plan held it to.
                // The two are only meaningfully related when a program has more
                // than one checked occurrence, which is what the two-occurrence
                // witness supplies.
                #[cfg(test)]
                record_d8m_slot_frame_pair(
                    self.defining_function_id,
                    frame_id,
                    slot.slot_template_id,
                );
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

    #[allow(clippy::too_many_arguments)]
    fn make_computational_recursor(
        &mut self,
        recursive: LoweringOperand,
        cases: Vec<crate::RuntimeComputationalMatchCase>,
        default: RuntimeTrap,
        outer_env: Vec<LoweringEnvironmentBinding>,
        static_origin: StaticOriginId,
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
        recursive_unit_body: Option<StaticOriginId>,
    ) -> Result<LoweringOperand, CraneliftBackendError> {
        let recursive_unit_body = recursive_unit_body.or_else(|| match &recursive {
            LoweringOperand::Specialized(Lowered::Closure { body, .. }) => Some(*body),
            LoweringOperand::Specialized(_) | LoweringOperand::Carried(_) => None,
        });
        let (residual, payload) = decompose_computational_recursor(recursive);
        let active_instance = self.active_recursive_invocations.last().copied();
        // ⛔ **The frame identity is TRANSPORTED, never inferred**
        // (`dec_s30rdnb1dvgk`). This site used to fall back, when
        // `checked_frame_id` was `None`, to `find`ing a `callee_frame_templates`
        // entry whose `runtime_frame_fingerprint` equalled one recomputed from
        // `cases`/`default`. `AC-F1` deliberately makes body-only differences
        // share a header fingerprint, so that `find` cannot discriminate a
        // callee declaration's two same-family computational frames — it
        // returns the first, silently.
        //
        // ⛔ Do not restore any recovery here, in any spelling: not header
        // equality, not body equality, not `StaticOriginId`, not vector
        // position, and not "the only remaining match." Each of those is
        // Runtime *inference*; the oriented plan's checked identity is the
        // authority. A missing identity is rejected in
        // `instantiate_checked_invocation_segment`, before CFG.
        let exact_frame_id = checked_frame_id;
        let invocation_id = exact_frame_id
            .and_then(|_| active_instance.map(|instance| instance.invocation_instance_id));
        let invocation_source = active_instance.map(|instance| instance.source);
        let invocation_depth = active_instance.map_or(0, |instance| instance.semantic_depth);
        let mut current_layer = ComputationalRecursorLayer {
            cases,
            default,
            outer_env,
            static_origin,
            provenance,
            role,
            checked_frame_id: exact_frame_id,
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
        let segment_recursive_unit_body = payload
            .as_ref()
            .and_then(|(_, invocation)| invocation.recursive_unit_body)
            .or(recursive_unit_body);
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
                                static_origin: scope.frame.static_origin,
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
        invocation.recursive_unit_body = segment_recursive_unit_body;
        invocation.dynamic_splice_edges = segment_dynamic_splice_edges;
        Ok(LoweringOperand::Specialized(
            Lowered::ComputationalRecursorClosure {
                residual: Box::new(residual),
                activation,
                invocation,
            },
        ))
    }

    /// ⭐ **A JOIN — `§2h` calls branch/join forwarding phase-bearing, so the
    /// arm arrives as a [`LoweringOperand`] and the phase boundary is taken
    /// HERE, once, rather than at each of the callers.**
    ///
    /// ⚠ the two-lane native scalar join merges `(tag, payload)` lanes of a native scalar. A carried
    /// boundary word has no such pair, so it fails closed via
    /// [`LoweringOperand::specialized_join_arm`] — ⛔ a *pending* boundary, not
    /// a final one; see that method for why the distinction is kept.
    fn merge_branch_value(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        join_plan: &JoinPlanToken,
        lowered: LoweringOperand,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, bool), CraneliftBackendError> {
        if join_plan.representation != JoinResultRepresentation::NativeScalarPair {
            return Err(backend_module(
                "carrier-result join reached a native-only branch merge consumer".to_string(),
            ));
        }
        let lowered = lowered.specialized_join_arm(construct)?;
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

    /// ⭐ **A JOIN — `§2h` calls branch/join forwarding phase-bearing, so the
    /// arm arrives as a [`LoweringOperand`] and the phase boundary is taken
    /// HERE, once, rather than at each of the callers.**
    ///
    /// ⚠ the tagged native scalar join merges `(tag, payload)` lanes of a native scalar. A carried
    /// boundary word has no such pair, so it fails closed via
    /// [`LoweringOperand::specialized_join_arm`] — ⛔ a *pending* boundary, not
    /// a final one; see that method for why the distinction is kept.
    fn merge_scalar_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        join_plan: &JoinPlanToken,
        lowered: LoweringOperand,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        if join_plan.representation != JoinResultRepresentation::NativeScalarPair {
            return Err(backend_module(
                "carrier-result join reached a native-only scalar merge consumer".to_string(),
            ));
        }
        self.merge_scalar_operand(builder, lowered, None, construct)
    }

    /// Consume one scalar-valued operand at a private typed CFG boundary.
    ///
    /// The surrounding source join may use `CarrierWord` storage. Once the
    /// consumer has established the scalar kind from a specialized arm, a
    /// carried sibling can be decoded back to that exact kind without changing
    /// constructor meaning. This is intentionally separate from
    /// [`Self::merge_scalar_branch`]: callers that own an ordinary source join
    /// must still obey its planned representation.
    fn merge_scalar_operand(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: LoweringOperand,
        required_kind: Option<ScalarMergeKind>,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        if let LoweringOperand::Carried(word) = lowered {
            let required_kind = required_kind.ok_or_else(|| {
                backend_module(
                    "a carried scalar reached an untyped private merge consumer".to_string(),
                )
            })?;
            let boundary_tag = builder
                .ins()
                .band_imm(word.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
            let (expected_tag, native_tag) = match required_kind {
                ScalarMergeKind::Int => (
                    BoundaryTag::ImmediateInt,
                    Self::carrier_small_marker(builder),
                ),
                ScalarMergeKind::Bool => (
                    BoundaryTag::ImmediateBool,
                    builder.ins().iconst(types::I64, 0),
                ),
                ScalarMergeKind::StructuralNat => (
                    BoundaryTag::ImmediateStructuralNat,
                    builder.ins().iconst(types::I64, 0),
                ),
                ScalarMergeKind::ExitCode => (
                    BoundaryTag::ImmediateExitStatus,
                    builder.ins().iconst(types::I64, 0),
                ),
                ScalarMergeKind::RecursiveBackedge => {
                    return Err(unsupported(
                        construct,
                        "a carried word cannot mint a recursive-backedge control marker",
                    ));
                }
            };
            Self::require_i64(builder, boundary_tag, expected_tag as i64);
            let payload = self.emit_carrier_scalar(builder, word)?;
            return Ok((
                NativeScalarPairV1 {
                    tag: native_tag,
                    payload,
                },
                required_kind,
            ));
        }
        let lowered = lowered.specialized_join_arm(construct)?;
        if required_kind == Some(ScalarMergeKind::ExitCode) {
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
            Lowered::Constructor {
                constructor, args, ..
            } if args.is_empty()
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
                    ..
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
    /// ⭐ A **planned** join — same phase-bearing role as
    /// [`Self::merge_scalar_branch`], same pending boundary, and named the same
    /// way so `grep specialized_join_arm` reaches all three.
    fn merge_planned_scalar_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        join_plan: &JoinPlanToken,
        lowered: LoweringOperand,
        required_kind: ScalarMergeKind,
        construct: &'static str,
    ) -> Result<(NativeScalarPairV1, ScalarMergeKind), CraneliftBackendError> {
        if join_plan.representation != JoinResultRepresentation::NativeScalarPair {
            return Err(backend_module(
                "carrier-result join reached a native checked-plan merge consumer".to_string(),
            ));
        }
        self.merge_scalar_operand(builder, lowered, Some(required_kind), construct)
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
                self.function_local
                    .native_int_tags
                    .insert(pair.payload, pair.tag);
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
                static_origin,
                remaining: arguments,
                lowered,
                env,
                next,
            } => SourceContinuation::ConstructArgument {
                constructor,
                static_origin,
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
                static_origin,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases,
                default,
                env,
                static_origin,
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
                static_origin,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                static_origin,
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
                static_origin,
                remaining,
                lowered,
                env,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::ConstructArgument {
                        constructor,
                        static_origin,
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
                static_origin,
                next,
            } => {
                let (next, terminal) = Self::split_source_prefix(*next)?;
                (
                    SourcePrefixTemplate::MatchScrutinee {
                        cases,
                        default,
                        env,
                        static_origin,
                        next: Box::new(next),
                    },
                    terminal,
                )
            }
            SourceContinuation::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                static_origin,
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
                        static_origin,
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
                static_origin,
                remaining,
                lowered,
                env,
                next,
            } => SourceContinuation::ConstructArgument {
                constructor: constructor.clone(),
                static_origin: *static_origin,
                remaining: remaining.clone(),
                lowered: lowered.clone(),
                env: env.clone(),
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::MatchScrutinee {
                cases,
                default,
                env,
                static_origin,
                next,
            } => SourceContinuation::MatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                // D4: the template clone carries the origin with the cases. A
                // clone that copied the terms and dropped this field would
                // silently reintroduce the vacancy this unit closes.
                static_origin: *static_origin,
                next: Box::new(Self::instantiate_source_prefix_template(next, edge)?),
            },
            SourcePrefixTemplate::ComputationalMatchScrutinee {
                cases,
                default,
                env,
                static_origin,
                provenance,
                checked_frame_id,
                answer_route,
                next,
            } => SourceContinuation::ComputationalMatchScrutinee {
                cases: cases.clone(),
                default: default.clone(),
                env: env.clone(),
                static_origin: *static_origin,
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

    /// ⭐ Takes the **operand**, not a template, and needs no phase boundary:
    /// *"is this branch a trap?"* has a total answer in both phases. A carried
    /// boundary word is never a trap — `Lowered::Trap` is a compile-time
    /// refusal, and the producer refuses to transfer one — so the `Carried`
    /// case answers `false` and the branch is left unsealed, which is the same
    /// answer any non-trap specialized value gets.
    fn seal_source_trap_branch(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        lowered: &LoweringOperand,
    ) -> Result<bool, CraneliftBackendError> {
        let LoweringOperand::Specialized(Lowered::Trap(trap)) = lowered else {
            return Ok(false);
        };
        let status = self.emit_current_trap(builder, trap)?;
        builder.ins().return_(&[status]);
        Ok(true)
    }

    fn emit_current_trap(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        trap: &RuntimeTrap,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        let identity = self.static_transition_plan.trap_identity(trap)?;
        match self.function_local.trap_exit {
            Some(TrapExitAuthority::UnitFrame { slots, trap_offset }) => {
                #[cfg(test)]
                let identity_word =
                    match TRAP_IDENTITY_MUTATION.with(std::cell::Cell::get) {
                        TrapIdentityMutation::Exact => identity.abi_word(),
                        TrapIdentityMutation::Zero => 0,
                        TrapIdentityMutation::Substitute => identity
                            .abi_word()
                            .checked_add(1)
                            .expect("planner trap identity fits below i64::MAX"),
                    };
                #[cfg(not(test))]
                let identity_word = identity.abi_word();
                #[cfg(test)]
                px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::PlannedTrapEmitted {
                    trap: trap.clone(),
                    seat: PlannedTrapSeat::UnitTrapWord,
                    planned_identity: identity.abi_word(),
                    emitted_word: identity_word,
                });
                let word = builder.ins().iconst(types::I64, identity_word);
                builder
                    .ins()
                    .store(MemFlags::trusted(), word, slots, trap_offset);
                Ok(builder.ins().iconst(types::I64, 0))
            }
            Some(TrapExitAuthority::Root {
                process_sentinel,
                source_authorized: true,
            }) => {
                if process_sentinel {
                    #[cfg(test)]
                    px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::PlannedTrapEmitted {
                        trap: trap.clone(),
                        seat: PlannedTrapSeat::RootProcessSentinel,
                        planned_identity: identity.abi_word(),
                        emitted_word: -4,
                    });
                    Ok(builder.ins().iconst(types::I64, -4))
                } else {
                    let token = (identity.abi_word()
                        << crate::cranelift_backend::compiled::ROOT_TRAP_TOKEN_SHIFT)
                        | crate::cranelift_backend::compiled::ROOT_TRAP_TOKEN_TAG;
                    #[cfg(test)]
                    px8tr_record_trap_provenance(Px8trTrapProvenanceEvent::PlannedTrapEmitted {
                        trap: trap.clone(),
                        seat: PlannedTrapSeat::RootTrapToken,
                        planned_identity: identity.abi_word(),
                        emitted_word: token,
                    });
                    let word = builder.ins().iconst(types::I64, identity.abi_word());
                    let shifted = builder.ins().ishl_imm(
                        word,
                        crate::cranelift_backend::compiled::ROOT_TRAP_TOKEN_SHIFT,
                    );
                    Ok(builder.ins().bor_imm(
                        shifted,
                        crate::cranelift_backend::compiled::ROOT_TRAP_TOKEN_TAG,
                    ))
                }
            }
            None => Err(backend_module(
                "trap branch has no generated-unit TrapWord lane".to_string(),
            )),
            Some(TrapExitAuthority::Root {
                source_authorized: false,
                ..
            }) => Err(backend_module(
                "generated function has no source-trap authority".to_string(),
            )),
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
        static_origin: StaticOriginId,
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
            synthesized_identity: Some(
                self.static_transition_plan
                    .constructor_symbol_identity(static_origin)?,
            ),
            // `D7` -- the allocation lane is the second fact resolved
            // at the producer and carried with the template.
            occurrence: Some(self.static_transition_plan.source_aggregate_occurrence(
                static_origin,
                PlannedAggregateShape::Constructor,
            )?),
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
            self.function_local
                .host_dispatch_context
                .expect("process byte lowering owns a direct host context"),
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
            .function_local
            .native_int_arena
            .ok_or_else(|| unsupported("Effect", "host-width Int has no invocation arena"))?;
        let helper = self.function_local.native_int_narrow.ok_or_else(|| {
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

    /// **`RT-DECL-CLOSURE-PORT` `D7` — the CARRIED exact-`Int` narrowing, over
    /// the existing carrier ABI.** `(u64, valid)`, emitted, for a capacity that
    /// reaches its seat as a boundary word rather than a compile-time template.
    ///
    /// ⭐ **One range rule, stated ONCE, over both carried representations.**
    /// The two decoders converge on `narrowed` as the same
    /// `(sign, len, limbs)` triple — the identical shape
    /// `ken_boundary_int_view_local` converges on internally — and the rule
    /// `sign == 0 && len == 1` is applied after they merge. An immediate is
    /// given a one-limb table of its own scalar rather than a second magnitude
    /// encoding, which is what lets the merged code be representation-blind.
    /// Testing the range twice would let the two spellings drift, and the drift
    /// would be invisible: each arm is exercised by different magnitudes, so a
    /// suite can be green with one of them wrong.
    ///
    /// ⛔ **`sign` is a BIT, not a number.** `0` is non-negative and `1` is
    /// negative — `ken_boundary_store_int_limbs_local` refuses anything else,
    /// and the native decoder writes `uextend(payload < 0)`. So the test is
    /// `sign == 0`; a signed `sign >= 0` is **always true** and would admit
    /// every negative `Int` at its magnitude. `len >= 1` always holds for the
    /// same reason — an empty magnitude denotes no integer — so `len == 1` is
    /// the exact "fits one unsigned limb" test rather than a bound.
    ///
    /// ⛔ **Limb 0 is loaded ONLY on the valid path.** A wide magnitude is
    /// refused on `len` before its table is read at all.
    ///
    /// ⛔ **`valid == 0` is the ONLY `InvalidBounds` outcome, and it means
    /// exactly one thing: a well-formed exact `Int` whose value does not fit
    /// `u64`.** Everything else — a word that is not an `Int`, a wrong class or
    /// owner, an unsealed magnitude, a helper that fails — leaves through
    /// `require_i64(.., BOUNDARY_OK)` as a carrier error. That separation is the
    /// framed contract: a caller must not be able to read "out of range" off a
    /// word that never denoted a number.
    ///
    /// ⚠ **The tag branch is a discrimination, not a validation.** It selects
    /// which decoder can read the word; it does not decide the word is good.
    /// Every non-`ImmediateInt` word goes to `int_view`, whose own guards are
    /// the authority — `resolve` rejects a wrong tag or owner, the class guard
    /// rejects a non-`Int` node, and the region path rejects an unsealed
    /// magnitude. Re-deriving any of those here would be a second copy of a rule
    /// that already has one.
    #[cfg(test)]
    fn record_capacity_phase_dispatch(carried: bool) {
        CAPACITY_PHASE_DISPATCH.with(|cell| {
            let (specialized, carried_count) = cell.get();
            cell.set(if carried {
                (specialized, carried_count + 1)
            } else {
                (specialized + 1, carried_count)
            });
        });
    }

    #[cfg(not(test))]
    fn record_capacity_phase_dispatch(_carried: bool) {}

    fn narrow_carried_int_u64(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        target: CarriedBoundaryWord,
    ) -> Result<(cranelift_codegen::ir::Value, cranelift_codegen::ir::Value), CraneliftBackendError>
    {
        use cranelift_codegen::ir::condcodes::IntCC;
        let refs = self.carrier_refs()?;
        let boundary_arena = self.carrier_arena()?;
        let pointer_type = builder.func.dfg.value_type(boundary_arena);

        let tag = builder
            .ins()
            .band_imm(target.word, crate::boundary_value::BOUNDARY_TAG_MASK as i64);
        let is_immediate_int = builder.ins().icmp_imm(
            IntCC::Equal,
            tag,
            crate::boundary_value::BoundaryTag::ImmediateInt as i64,
        );
        let immediate = builder.create_block();
        let viewed = builder.create_block();
        // `(sign, len, limbs)` — the canonical triple, from either decoder.
        let narrowed = builder.create_block();
        builder.append_block_param(narrowed, types::I64);
        builder.append_block_param(narrowed, types::I64);
        builder.append_block_param(narrowed, pointer_type);
        builder
            .ins()
            .brif(is_immediate_int, immediate, &[], viewed, &[]);

        // ── the immediate payload ────────────────────────────────────────
        //
        // The exact tag is validated by the branch above; the scalar comes from
        // the carrier's own helper rather than a shift written here, so the
        // immediate decode has one implementation and this is a caller of it.
        builder.switch_to_block(immediate);
        let scalar = self.emit_carrier_scalar(builder, target)?;
        // The sign BIT, spelled as the decoder spells it.
        let negative = builder.ins().icmp_imm(IntCC::SignedLessThan, scalar, 0);
        let immediate_sign = builder.ins().uextend(types::I64, negative);
        let immediate_len = builder.ins().iconst(types::I64, 1);
        // ⭐ A one-limb table holding the scalar. An immediate's whole
        // magnitude IS one limb, so giving it a table makes the merged rule
        // read it exactly as it reads a persistent one — no second encoding,
        // and no branch below that has to know which arm it came from.
        let immediate_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            8,
            3,
        ));
        builder.ins().stack_store(scalar, immediate_slot, 0);
        let immediate_limbs = builder.ins().stack_addr(pointer_type, immediate_slot, 0);
        builder.ins().jump(
            narrowed,
            &[immediate_sign.into(), immediate_len.into(), immediate_limbs.into()],
        );

        // ── the sealed persistent / native `Int` view ────────────────────
        builder.switch_to_block(viewed);
        let view_slot = builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            24,
            3,
        ));
        let view = builder.ins().stack_addr(pointer_type, view_slot, 0);
        let call = builder
            .ins()
            .call(refs.int_view, &[boundary_arena, target.word, view]);
        Self::require_i64(builder, builder.inst_results(call)[0], BOUNDARY_OK);
        let view_sign = builder.ins().stack_load(types::I64, view_slot, 0);
        let view_len = builder.ins().stack_load(types::I64, view_slot, 8);
        let view_limbs = builder.ins().stack_load(pointer_type, view_slot, 16);
        builder.ins().jump(
            narrowed,
            &[view_sign.into(), view_len.into(), view_limbs.into()],
        );

        // ── the shared rule ──────────────────────────────────────────────
        builder.switch_to_block(narrowed);
        let sign = builder.block_params(narrowed)[0];
        let len = builder.block_params(narrowed)[1];
        let limbs = builder.block_params(narrowed)[2];
        let non_negative = builder.ins().icmp_imm(IntCC::Equal, sign, 0);
        let one_limb = builder.ins().icmp_imm(IntCC::Equal, len, 1);
        let in_range = builder.ins().band(non_negative, one_limb);
        let read_limb = builder.create_block();
        let out_of_range = builder.create_block();
        let done = builder.create_block();
        builder.append_block_param(done, types::I64);
        builder.append_block_param(done, types::I8);
        builder
            .ins()
            .brif(in_range, read_limb, &[], out_of_range, &[]);

        builder.switch_to_block(read_limb);
        let value = builder
            .ins()
            .load(types::I64, MemFlags::trusted(), limbs, 0);
        let valid = builder.ins().iconst(types::I8, 1);
        builder.ins().jump(done, &[value.into(), valid.into()]);

        // ⛔ The magnitude is NOT read here. A wide `Int` is refused on its
        // length before its limb table is touched, which is what makes "loading
        // limb 0 only on the valid path" a property of the emitted code rather
        // than of the values a fixture happens to pass.
        builder.switch_to_block(out_of_range);
        let absent = builder.ins().iconst(types::I64, 0);
        let invalid = builder.ins().iconst(types::I8, 0);
        builder.ins().jump(done, &[absent.into(), invalid.into()]);

        builder.switch_to_block(done);
        Ok((builder.block_params(done)[0], builder.block_params(done)[1]))
    }

    fn lower_dynamic_small_int(
        &mut self,
        builder: &mut FunctionBuilder<'_>,
        value: cranelift_codegen::ir::Value,
    ) -> Lowered {
        let tag = builder
            .ins()
            .iconst(types::I64, crate::NATIVE_INT_SMALL_TAG_V1 as i64);
        self.function_local.native_int_tags.insert(value, tag);
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

    /// `request_length` is the RAW pre-clamp request length — an outer
    /// consistency ceiling and audit input only, never a progress bound
    /// (`BUDGET-EFF`, Architect ruling `dec_1m6xdwjp2ttyn`, boundary
    /// constraint 3). `effective_request` is the host's post-clamp bound,
    /// carried in the reply; range/no-wrap/span-containment and `remaining`
    /// are all derived from it, and `0 < count <= effective_request <=
    /// request_length` is asserted before minting.
    fn mint_validated_progress_nat(
        builder: &mut FunctionBuilder<'_>,
        success: cranelift_codegen::ir::Value,
        count: cranelift_codegen::ir::Value,
        request_start: cranelift_codegen::ir::Value,
        request_length: cranelift_codegen::ir::Value,
        effective_request: cranelift_codegen::ir::Value,
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
            effective_request,
        );
        let effective_within_raw = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedLessThanOrEqual,
            effective_request,
            request_length,
        );
        let effective_end = builder.ins().iadd(request_start, effective_request);
        let effective_no_wrap = builder.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::UnsignedGreaterThanOrEqual,
            effective_end,
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
            effective_end,
        );
        let valid = [
            positive,
            bounded,
            effective_within_raw,
            effective_no_wrap,
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
            BoundedNatV1::derived_from_validated(builder.ins().isub(effective_request, count));
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
                synthesized_identity: None,
                occurrence: None,
                args: args
                    .iter()
                    .map(|arg| self.lower_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            // ⚠ A VALUE-domain record has no occurrence in the program, so it
            // has no producer authority. Stated as an absence rather than
            // filled in: it fails closed at the allocation.
            RuntimeValue::Record { fields } => Ok(Lowered::Record {
                occurrence: None,
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        Ok(LoweredRecordField {
                            name: name.clone(),
                            // No occurrence, so no planned schema either. Both
                            // absences are stated at the same place.
                            identity: None,
                            value: self.lower_value(builder, value)?,
                        })
                    })
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
        // ⛔ Wildcard-free, because this match **is** the boundary between the
        // represented path and the compiler-only one. A `_` arm here would let
        // a future ground-value variant fall silently onto whichever side
        // happened to be last.
        match value {
            RuntimeGroundValue::Bool(flag) => Ok(Lowered::Bool {
                value: self.artifact_static_payload(builder, symbol)?,
                // ⚠ `known` is retained deliberately and it is NOT a second
                // authority: it is the *compile-time* answer used for
                // specialization decisions, while `value` is the word the
                // running artifact actually reads. ⛔ If a specialization ever
                // substitutes `known` for `value` in emitted code, the borrow
                // becomes unobservable — which is why the control for this is a
                // mutation of the minted bytes, not a count of loads.
                known: Some(*flag),
            }),
            RuntimeGroundValue::Int(crate::RuntimeIntV1::Small(small)) => Ok(Lowered::Int {
                value: self.artifact_static_payload(builder, symbol)?,
                known: Some(*small),
            }),
            // ⚠ **The stated boundary of `D3`'s represented path.** These five
            // still lower through the compiler-side specialization lattice:
            // `lower_ground_value` returns `Lowered::Bytes`/`String`/
            // `Constructor`/`Record`, which hold the compiler's own Rust values
            // and carry no `ir::Value` at all, and a big integer goes through
            // the interning helper rather than a frame word.
            //
            // ⛔ **This is a boundary, not a second authority.** No value has two
            // paths: a scalar has exactly the artifact-static one, and these
            // have exactly the specialization one. ⚠ Giving them an
            // artifact-static representation needs a *reader* for the encoded
            // aggregate — the encoding exists (`seed_material`), the consumer
            // does not — and the runtime-`alloc` carrier is not a substitute,
            // because it produces activation-time storage for a slot declared
            // `AbiStorageOwner::ArtifactStatic`.
            RuntimeGroundValue::Int(big @ crate::RuntimeIntV1::Big { .. }) => {
                self.lower_big_int_constant(builder, big)
            }
            RuntimeGroundValue::Bytes(_)
            | RuntimeGroundValue::String(_)
            | RuntimeGroundValue::Constructor { .. }
            | RuntimeGroundValue::Record { .. } => self.lower_ground_value(builder, value),
        }
    }

    /// Load a seed symbol's scalar payload out of artifact-static material.
    ///
    /// ⛔ Fails closed. A symbol present in the environment but absent from the
    /// minted material means the two populations disagree, and folding the
    /// compile-time value in as a fallback would silently restore exactly the
    /// authority `D3` removes — with nothing to observe that it had.
    fn artifact_static_payload(
        &self,
        builder: &mut FunctionBuilder<'_>,
        symbol: &str,
    ) -> Result<cranelift_codegen::ir::Value, CraneliftBackendError> {
        self.function_local
            .seed_material
            .payload_word(builder, symbol)
            .ok_or_else(|| {
                unsupported(
                    "Closure",
                    format!("seed capture {symbol} has no artifact-static material minted for it"),
                )
            })
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
                synthesized_identity: None,
                occurrence: None,
                args: args
                    .iter()
                    .map(|arg| self.lower_ground_value(builder, arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            // ⚠ Likewise a ground-value record: no occurrence, stated.
            RuntimeGroundValue::Record { fields } => Ok(Lowered::Record {
                occurrence: None,
                fields: fields
                    .iter()
                    .map(|(name, value)| {
                        Ok(LoweredRecordField {
                            name: name.clone(),
                            identity: None,
                            value: self.lower_ground_value(builder, value)?,
                        })
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
            self.function_local
                .native_int_arena
                .ok_or_else(|| unsupported("RuntimeValue::Int", "Big Int has no arena"))?,
        );
        let arena = self
            .function_local
            .native_int_arena
            .expect("Big Int arena was checked");
        let helper = self.function_local.native_int_intern.ok_or_else(|| {
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
        let arena = self.function_local.native_int_arena.ok_or_else(|| {
            unsupported("NativeInt", "unsigned Int producer has no invocation arena")
        })?;
        let helper = self.function_local.native_int_intern.ok_or_else(|| {
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
        let arena = self.function_local.native_int_arena.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int operation has no invocation arena",
            )
        })?;
        let helper = self.function_local.native_int_binop.ok_or_else(|| {
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
        self.function_local.native_int_tags.insert(value, tag);
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
        let arena = self.function_local.native_int_arena.ok_or_else(|| {
            unsupported(
                "PrimitiveCall",
                "exact Int comparison has no invocation arena",
            )
        })?;
        let helper = self.function_local.native_int_compare.ok_or_else(|| {
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
        if let Some(tag) = self.function_local.native_int_tags.get(&payload).copied() {
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
            self.function_local.native_int_tags.insert(value, tag);
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
            self.function_local.native_int_tags.insert(value, tag);
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
                synthesized_identity: None,
                occurrence: None,
                args: vec![Lowered::Int {
                    value: builder.ins().iconst(types::I64, i64::from(byte)),
                    known: Some(i64::from(byte)),
                }],
            },
            None => Lowered::Constructor {
                constructor: none.clone(),
                synthesized_identity: None,
                occurrence: None,
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
                synthesized_identity: None,
                occurrence: None,
                args: vec![Lowered::Bytes(bytes)],
            },
            None => Lowered::Constructor {
                constructor: none.clone(),
                synthesized_identity: None,
                occurrence: None,
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
                synthesized_identity: None,
                occurrence: None,
                args: vec![Lowered::String(value)],
            },
            Err(_) => Lowered::Constructor {
                constructor: err.clone(),
                synthesized_identity: None,
                occurrence: None,
                args: vec![Lowered::Constructor {
                    constructor: error.clone(),
                    synthesized_identity: None,
                    occurrence: None,
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

    pub(super) fn emit_result(
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
                let arena = self.function_local.native_int_arena.ok_or_else(|| {
                    unsupported("NativeResult", "Int result has no invocation arena")
                })?;
                let export = self.function_local.native_int_export.ok_or_else(|| {
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
        let Lowered::Constructor {
            constructor, args, ..
        } = value
        else {
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

    /// ⛔ **A typed boundary: raw [`Lowered`] only, and STRUCTURALLY so**
    /// (`RT-FNSPLIT-C1` frame `§2h` ¶2).
    ///
    /// `§2h` admits a raw-`Lowered` helper only where it is **structurally
    /// incapable** of receiving, forwarding or returning a
    /// [`LoweringOperand::Carried`]. ⛔ *"Nothing passes it one today"* is
    /// explicitly **not** that property — present-corpus reachability is the
    /// argument the ruling was written to reject. This helper is closed on all
    /// three counts **by signature**, not by census:
    ///
    /// - **cannot receive** — the parameter is `Lowered`, so handing it a
    ///   [`LoweringOperand`] is `E0308` at every call site, present or future;
    /// - **cannot return** — [`RuntimeGroundValue`] is a closed **compile-time
    ///   constant** domain (bool / int / bytes / string, and aggregates of
    ///   itself). Transitively it has no arm able to hold a Cranelift SSA word;
    /// - **cannot mint or forward** — ⚠ `&mut self` *does* reach
    ///   [`Lowering::boundary_carrier`], so the refs are in scope. What closes
    ///   the mint is that this takes **no `FunctionBuilder`**: it cannot emit
    ///   CLIF, so it cannot run the one-way producer, and a `FuncRef` with
    ///   nowhere to emit is inert. Its recursion descends into `Vec<Lowered>`
    ///   child positions only.
    ///
    /// ⭐ **The edge's fail-closed disposition is FORCED, not chosen.** This
    /// materializes a *compile-time constant*; a `Carried` is by construction a
    /// runtime word with **no** compile-time value. So when the caller's
    /// scrutinee becomes a `LoweringOperand`, the `Carried` arm has no sound
    /// answer but `Err` — the return type's domain settles it, not a preference.
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
            Lowered::Constructor {
                constructor, args, ..
            } => Ok(RuntimeGroundValue::Constructor {
                constructor,
                args: args
                    .into_iter()
                    .map(|arg| self.ground_value(arg))
                    .collect::<Result<Vec<_>, _>>()?,
            }),
            Lowered::Record { fields, .. } => Ok(RuntimeGroundValue::Record {
                fields: fields
                    .into_iter()
                    .map(|field| Ok((field.name, self.ground_value(field.value)?)))
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
                        ..
                    },
                    Lowered::Constructor {
                        constructor: right_constructor,
                        args: right_args,
                        ..
                    },
                ) => {
                    left_constructor == right_constructor
                        && same_recursive_argument_shapes(left_args, right_args)
                }
                (
                    Lowered::Record { fields: left, .. },
                    Lowered::Record { fields: right, .. },
                ) => {
                    left.len() == right.len()
                        && left
                            .iter()
                            .zip(right)
                            .all(|(left, right)| {
                                left.name == right.name
                                    && same_recursive_argument_shapes(
                                        std::slice::from_ref(&left.value),
                                        std::slice::from_ref(&right.value),
                                    )
                            })
                }
                _ => false,
            })
}
/// ⛔ **A typed boundary: raw [`Lowered`] only, and STRUCTURALLY so**
/// (`RT-FNSPLIT-C1` frame `§2h` ¶2).
///
/// Closed **more tightly** than [`Lowering::ground_value`], and worth stating
/// because the two were priced as one class and are not: this is a **free
/// function with no `self`**, so it cannot reach
/// [`Lowering::boundary_carrier`] at all — there is no `FunctionBuilder`
/// argument *and* no receiver through which one could be found. Its parameter
/// is `&Lowered` (⇒ `E0308` on a [`LoweringOperand`]) and it returns
/// `&'static str` (⇒ no arm can hold a Cranelift SSA word). It is a
/// one-per-variant dispatch table over the specialization lattice and nothing
/// else, so there is no reachable surface on which to attempt an evasion.
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
            Lowered::Record { fields, .. } => {
                for field in fields {
                    append_recursive_argument_values(
                        builder,
                        std::slice::from_ref(&field.value),
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
        Lowered::Constructor {
            constructor,
            synthesized_identity,
            occurrence,
            args,
        } => Lowered::Constructor {
            constructor: constructor.clone(),
            synthesized_identity: *synthesized_identity,
            // A rebuilt recursive argument is the same producer with its
            // children re-materialized, so the occurrence is preserved rather
            // than re-derived.
            occurrence: *occurrence,
            args: args
                .iter()
                .map(|arg| rebuild_recursive_argument(arg, values, native_int_tags))
                .collect::<Result<Vec<_>, _>>()?,
        },
        Lowered::Record { occurrence, fields } => Lowered::Record {
            // ⭐ The producer travels with the rebuilt template. Dropping it
            // here would silently return the record to use-coordinate
            // authority at exactly the boundary this field exists to cross.
            occurrence: *occurrence,
            fields: fields
                .iter()
                .map(|field| {
                    Ok(LoweredRecordField {
                        name: field.name.clone(),
                        // The schema travels with the rebuilt template for the
                        // same reason the occurrence above does: it is the
                        // producer's fact, and this is the same producer.
                        identity: field.identity,
                        value: rebuild_recursive_argument(
                            &field.value,
                            values,
                            native_int_tags,
                        )?,
                    })
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

impl crate::boundary_value::BoundaryEmissionPlan {
    /// Derive the emission plan from the representation authority.
    ///
    /// ⛔ **Nothing here is written down.** The admitted class set is collected
    /// by sweeping [`BoundaryInput::all`] through the wildcard-free classifier
    /// and keeping the classes that reach a published `HandleWord`; the
    /// per-storage sets are that set filtered by
    /// [`BoundaryClass::storage_shape`]. So a class the disposition stops
    /// admitting, or a class whose storage changes, changes what the emitter
    /// generates — which is the causal edge `RECUT 2` requires.
    ///
    /// ⚠ This lives in `lowering` rather than beside the struct because
    /// [`BoundaryInput`] is `pub(in crate::cranelift_backend)`: the authority is
    /// only visible here, which is precisely why the emitter cannot restate it.
    pub(crate) fn derive() -> Self {
        use crate::boundary_value::{
            BoundaryClass, BoundaryReferentOwner, BoundaryStorageShape, BoundaryTag,
            BoundaryTagAdmission,
        };
        use std::collections::{BTreeMap, BTreeSet};

        let mut admitted: BTreeSet<BoundaryClass> = BTreeSet::new();
        let mut immediate_tags: BTreeSet<BoundaryTag> = BTreeSet::new();
        let mut handle_tags: BTreeSet<BoundaryTag> = BTreeSet::new();
        let mut owner_bands: BTreeMap<BoundaryReferentOwner, BTreeSet<BoundaryTag>> =
            BTreeMap::new();
        let mut immediate_value_classes: BTreeMap<BoundaryTag, BoundaryClass> = BTreeMap::new();
        let mut handle_class_relation: BTreeMap<BoundaryTag, BTreeSet<BoundaryClass>> =
            BTreeMap::new();
        for cell in BoundaryInput::all() {
            // ⛔ Wildcard-free: a new outcome variant must decide here whether
            // its tag is admitted, rather than defaulting to "not emitted".
            match cell.outcome() {
                BoundaryOutcome::ImmediateWord { tag, value_class } => {
                    immediate_tags.insert(tag);
                    // ⛔ An immediate the authority cannot classify gets NO
                    // entry, so the emitted helper fails closed on it rather
                    // than inheriting a default arm.
                    if let Some(class) = value_class {
                        immediate_value_classes.insert(tag, class);
                    }
                }
                BoundaryOutcome::HandleWord {
                    tag, class, owner, ..
                } => {
                    admitted.insert(class);
                    handle_tags.insert(tag);
                    owner_bands.entry(owner).or_default().insert(tag);
                    // ⛔ Node-class legality, from `HandleWord` only. An
                    // `ImmediateWord` has no node, so it contributes no row.
                    handle_class_relation.entry(tag).or_default().insert(class);
                }
                BoundaryOutcome::ProtocolOnly | BoundaryOutcome::FailClosedForbidden => {}
            }
        }
        let of_shape = |shape: BoundaryStorageShape| -> Vec<BoundaryClass> {
            admitted
                .iter()
                .copied()
                .filter(|class| class.storage_shape() == shape)
                .collect()
        };
        let int_magnitude = of_shape(BoundaryStorageShape::IntMagnitude);
        let byte_span = of_shape(BoundaryStorageShape::ByteSpan);
        // The admitted set is the union, not a range: a tag admitted as an
        // immediate and a tag admitted as a handle are both legal words, and
        // nothing requires the two groups to be numerically adjacent.
        let admitted_tags: Vec<BoundaryTag> = immediate_tags
            .union(&handle_tags)
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        crate::boundary_value::BoundaryEmissionPlan::new(
            int_magnitude,
            byte_span,
            BoundaryTagAdmission::new(
                admitted_tags,
                immediate_tags.into_iter().collect(),
                handle_tags.into_iter().collect(),
                owner_bands
                    .into_iter()
                    .map(|(owner, tags)| (owner, tags.into_iter().collect()))
                    .collect(),
                immediate_value_classes.into_iter().collect(),
                handle_class_relation
                    .into_iter()
                    .map(|(tag, classes)| (tag, classes.into_iter().collect()))
                    .collect(),
            ),
        )
    }
}
