//! **`RT-FNSPLIT-B2F` `D1`/`D2` — the target code-unit population.**
//!
//! One closed Cranelift target function per `PredeclaredFunction` in the
//! validated `SemanticOwner` partition, forward-declared as a bundle and then
//! defined against `B2R`'s declared activation frame.
//!
//! ⛔ **This module does not derive the population and must never be made to.**
//! The unit set is `plan.entries` ∪ every `EdgeKind::StaticBody` **target**,
//! MINUS every declaration-owned pair (`RT-DECL-CLOSURE-PORT` `D2a`: a
//! closure-seed transparent declaration's entry seeds no function of its own),
//! seeded and validated by `B2O` and given one `AbiDescriptor` apiece by `B2R`.
//! `StaticTransitionPlan::emittable_units` projects that; this module consumes
//! the projection. In particular it never consults
//! `TransitionKind::ClosureBody`, which is a body's **return successor** and not
//! a unit head — the error that has appeared in successive drafts of the issue
//! file that warns against it.
//!
//! ⚠ The two shared exits (`SemanticOwner::Terminal`, `TrapTerminal`) are not
//! units and receive no target function. They are absent from
//! `emittable_units()` by construction, because `B2R` gives them no descriptor —
//! ⛔ **not** because this module filters them out. There is deliberately no
//! filter here to get wrong.

use super::*;
use super::core::{AmbientBodyAuthority, CheckedFrameFunctionScope};

use cranelift_module::FuncId;

/// **`RT-FNSPLIT-B2F` `AC-2` — what the compiled module ACTUALLY contains.**
///
/// ⭐ **This exists because the census pin in `control.rs` cannot answer the
/// question `AC-2` is really about.** That pin counts how many times three
/// spellings occur in seven source files; it is a source-TEXT oracle, so a call
/// split across lines evades it and a mention in a comment inflates it, and in
/// no configuration does it observe an emitted function. The property `B2F` owes
/// is *"this node adds exactly the emitted units its design predicts"*, and the
/// only way to see an emitted unit is to count them at the point of emission.
///
/// Records `(declared, defined)` for the most recent compile on this thread.
/// ⚠ Two numbers rather than one: a bundle that declares `n` and defines `n-1`
/// leaves an undefined symbol, and a single counter cannot tell that from a
/// smaller correct population.
#[cfg(test)]
thread_local! {
    static B2F_UNIT_EMISSION: std::cell::Cell<(usize, usize)> =
        const { std::cell::Cell::new((0, 0)) };
}

/// The `(declared, defined)` unit counts from the most recent compile.
///
/// ⚠ **"Most recent compile" is the whole limitation.** This reading carries no
/// statement about *which* compile produced it, so a compile that fails before
/// reaching the emission seam leaves the previous compile's numbers standing and
/// reads exactly like one that reached the seam and declared that many. Use it
/// only where a single compile is known to have run to emission; for a timing
/// question about a *failing* compile, use the attempt epoch below.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_last_unit_emission() -> (usize, usize) {
    B2F_UNIT_EMISSION.with(std::cell::Cell::get)
}

/// **`RT-FNSPLIT-B2F` `AC-11` clause 3 — the compile-attempt epoch.**
///
/// ⛔⛔ **This exists because the first timing instrument could not distinguish
/// the two outcomes it was built to separate, and reported a confident number
/// for the wrong one.** That version compiled a successful sentinel to force
/// `B2F_UNIT_EMISSION` to a nonzero value, then compiled the failing fixture and
/// read the cell back. But nothing on a pre-emission refusal path *writes* that
/// cell — so the read returned the **sentinel's** `1`, and:
///
/// - "refused before `declare_unit_bundle` ran" (the wanted `0`), and
/// - "declared one unit, then refused during lowering" (the feared `1`)
///
/// ⇒ produce the **identical reading**. ⭐ The in-source comment claimed the
/// sentinel made those cases distinguishable; it made them indistinguishable.
/// A measured `1` was therefore evidence of nothing, in **either** direction.
///
/// ⭐ **The repair is to stamp the reading with the attempt it belongs to**, so
/// a stale value is *detectable as stale* rather than readable as a count.
/// Three outcomes, all distinct:
///
/// | reading | meaning |
/// |---|---|
/// | `None` | ⚠ the compile never reached the emission seam at all — refused earlier still, or never ran. **Not** a zero |
/// | `Some(0)` | ✅ reached the seam, refused **before** any unit was declared — what clause 3 asks for |
/// | `Some(n > 0)` | ⛔ `n` units were already declared when the refusal came — a *later* guarantee, not clause 3's |
///
/// ⛔ **The stamp is written in `core.rs` immediately before
/// `validate_emitted_transfers_are_representable`, NOT inside
/// `declare_unit_bundle`.** Stamping inside the bundle would make `Some(0)`
/// unreachable — the only way to observe the epoch would be to declare a unit,
/// which is the very event the reading is supposed to detect the absence of.
#[cfg(test)]
thread_local! {
    /// The epoch a test opened; bumped once per `b2f_open_compile_attempt`.
    static B2F_ATTEMPT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// The epoch that was live when the emission seam was last reached.
    static B2F_ATTEMPT_AT_SEAM: std::cell::Cell<Option<u64>> =
        const { std::cell::Cell::new(None) };
}

/// Open a fresh compile attempt; the returned epoch identifies it.
///
/// ⚠ Deliberately does **not** clear `B2F_UNIT_EMISSION`: clearing it here would
/// hide a compile that never reached the seam behind a plausible `(0, 0)`, which
/// is the exact confusion this epoch exists to remove.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_open_compile_attempt() -> u64 {
    B2F_ATTEMPT.with(|cell| {
        let next = cell.get() + 1;
        cell.set(next);
        next
    })
}

/// Record that the emission seam was reached, and zero this attempt's counts.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_reached_emission_seam() {
    B2F_ATTEMPT_AT_SEAM.with(|cell| cell.set(Some(B2F_ATTEMPT.with(std::cell::Cell::get))));
    B2F_UNIT_EMISSION.with(|cell| cell.set((0, 0)));
}

/// How many units `epoch`'s compile had declared, or `None` if that compile
/// never reached the emission seam.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_units_declared_in_attempt(epoch: u64) -> Option<usize> {
    if B2F_ATTEMPT_AT_SEAM.with(std::cell::Cell::get) == Some(epoch) {
        Some(B2F_UNIT_EMISSION.with(std::cell::Cell::get).0)
    } else {
        None
    }
}

/// Every target function this artifact declares, keyed by its static identity.
///
/// ⭐ **Keyed by `PredeclaredFunctionId`, never by the iteration ordinal.** The
/// ordinal is an artifact of how this module happened to walk the plane; the id
/// is the planner's identity, and `D4`'s call edges resolve against *that*. A
/// map keyed on position would be an identity alias of exactly the kind `B2O`
/// removed from `SemanticDescriptor` (`semantic_ir.rs:651` records that
/// removal), reintroduced one layer out.
pub(in crate::cranelift_backend) struct UnitBundle {
    functions: BTreeMap<PredeclaredFunctionId, FuncId>,
    /// **`RT-CONTSPEC-ACTIVATE` `D2`** -- one declared target per planned
    /// continuation specialization, keyed by the planner's typed identity.
    ///
    /// Kept as its own map rather than folded into `functions`: a
    /// `ContinuationSpecializationId` is **not** a `PredeclaredFunctionId`, and
    /// admitting one there would alias two identities that the planner keeps
    /// apart. Nothing resolves a continuation by ordinal or by symbol name.
    continuations: BTreeMap<ContinuationSpecializationId, FuncId>,
    /// **`RT-DECL-CLOSURE-PORT` `D5a`** -- one declared target per planned
    /// generated producer execution context.
    ///
    /// A third map for the same reason there is a second: a
    /// `ContinuationContextId` is neither of the other two identities, and the
    /// ruling's "do not cast or alias one ID domain into the other" is enforced
    /// here by there being no map a caller could reach with the wrong key type.
    contexts: BTreeMap<ContinuationContextId, FuncId>,
}

impl UnitBundle {
    /// The declared target function for one unit.
    ///
    /// ⛔ `None` is a real answer and the caller must not substitute one of its
    /// own: a unit absent here was never declared, and emitting a call to a
    /// fabricated `FuncId` is the failure this return type exists to make
    /// visible.
    pub(in crate::cranelift_backend) fn function(
        &self,
        unit: PredeclaredFunctionId,
    ) -> Option<FuncId> {
        self.functions.get(&unit).copied()
    }

    /// The declared target for one continuation specialization.
    ///
    /// `None` is a real answer and must not be substituted for: a
    /// specialization absent here was never declared, and resolving a causal
    /// identity to a fabricated `FuncId` is exactly what this return type
    /// exists to make visible.
    pub(in crate::cranelift_backend) fn continuation(
        &self,
        specialization: ContinuationSpecializationId,
    ) -> Option<FuncId> {
        self.continuations.get(&specialization).copied()
    }

    /// The declared target for one generated producer execution context.
    ///
    /// `None` is a real answer, exactly as for the two maps above.
    pub(in crate::cranelift_backend) fn context(
        &self,
        context: ContinuationContextId,
    ) -> Option<FuncId> {
        self.contexts.get(&context).copied()
    }

    /// How many continuation targets this bundle declares.
    pub(in crate::cranelift_backend) fn continuation_len(&self) -> usize {
        self.continuations.len()
    }

    /// How many target functions this bundle declares.
    ///
    /// ⚠ This is the **emitted-unit** count, not a source-spelling count. `D8`'s
    /// growth verdict is about this number; the census pin in `control.rs`
    /// counts spellings and cannot see it.
    pub(in crate::cranelift_backend) fn len(&self) -> usize {
        self.functions.len()
    }
}

/// **`RT-FNSPLIT-B2F` `D4` — every cross-owner call edge, resolved to the target
/// function the bundle declared for it.**
///
/// ⭐ **Keyed by the planner's `PredeclaredFunctionId`, resolved to a `FuncId`,
/// and derived from nothing else.** The ordinal `declare_unit_bundle` used to
/// spell a symbol name never enters here; a call edge names its callee by static
/// identity and the bundle answers with the declared target or with `None`.
pub(in crate::cranelift_backend) struct CallEdgeTargets {
    edges: Vec<(PredeclaredFunctionId, ResolvedUnitTarget)>,
}

impl CallEdgeTargets {
    /// The resolved targets of every call emitted **into** `caller`.
    ///
    /// ⚠ Returns an empty iterator for a unit with no outgoing call edges, which
    /// is the common case: most units are leaves.
    pub(in crate::cranelift_backend) fn targets_in(
        &self,
        caller: PredeclaredFunctionId,
    ) -> impl Iterator<Item = &ResolvedUnitTarget> + '_ {
        self.edges
            .iter()
            .filter(move |(from, _)| *from == caller)
            .map(|(_, target)| target)
    }

    /// How many call edges were resolved.
    #[cfg(test)]
    pub(in crate::cranelift_backend) fn len(&self) -> usize {
        self.edges.len()
    }
}

#[derive(Clone)]
pub(in crate::cranelift_backend) struct ResolvedUnitTarget {
    function: FuncId,
    origin: StaticOriginId,
    call_site_origin: StaticOriginId,
    kind: EmittableCallKind,
    header: AbiFrameHeader,
    slots: Vec<AbiSlot>,
    offsets: Vec<u32>,
}

#[derive(Clone)]
pub(in crate::cranelift_backend) struct DeclaredUnitCall {
    pub(in crate::cranelift_backend) function: FuncRef,
    /// The callee's scheduling entry -- `RT-CONTSPEC-ACTIVATE` `D1b` keeps
    /// this as the target origin.
    pub(in crate::cranelift_backend) origin: StaticOriginId,
    /// The callable source body this call is keyed by. Both ends are retained
    /// in the declared record, not just the map key, so a consumer can check
    /// the pair rather than infer it.
    pub(in crate::cranelift_backend) call_site_origin: StaticOriginId,
    pub(in crate::cranelift_backend) header: AbiFrameHeader,
    pub(in crate::cranelift_backend) slots: Vec<AbiSlot>,
    pub(in crate::cranelift_backend) offsets: Vec<u32>,
}

pub(in crate::cranelift_backend) struct DeclaredUnitCalls {
    pub(in crate::cranelift_backend) static_bodies:
        BTreeMap<StaticOriginId, DeclaredUnitCall>,
    pub(in crate::cranelift_backend) declarations:
        BTreeMap<StaticOriginId, DeclaredUnitCall>,
}

impl CallEdgeTargets {
    pub(in crate::cranelift_backend) fn declare_in_func<M: Module>(
        &self,
        caller: PredeclaredFunctionId,
        module: &mut M,
        func: &mut Function,
    ) -> Result<DeclaredUnitCalls, CraneliftBackendError> {
        let mut static_bodies = BTreeMap::new();
        let mut declarations = BTreeMap::new();
        for target in self.targets_in(caller) {
            let call = DeclaredUnitCall {
                function: module.declare_func_in_func(target.function, func),
                origin: target.origin,
                call_site_origin: target.call_site_origin,
                header: target.header,
                slots: target.slots.clone(),
                offsets: target.offsets.clone(),
            };
            let (calls, duplicate) = match target.kind {
                EmittableCallKind::StaticBody => (
                    &mut static_bodies,
                    "one caller has two static-body calls to the same body origin",
                ),
                EmittableCallKind::Declaration => (
                    &mut declarations,
                    "one declaration-reference occurrence has two planner-derived call targets",
                ),
            };
            if calls.insert(target.call_site_origin, call).is_some() {
                return Err(backend_module(duplicate.to_string()));
            }
        }
        Ok(DeclaredUnitCalls {
            static_bodies,
            declarations,
        })
    }
}

/// **`D4` — resolve the derived call edges against the declared bundle, BEFORE
/// any body is defined.**
///
/// ⭐ **This runs between `D1`'s declaration pass and `D2`'s definition pass on
/// purpose, and the position is the point.** A call edge whose callee was never
/// forward-declared is a program that cannot be emitted; discovering that while
/// half the bodies are already defined leaves a partially emitted artifact whose
/// failure mode is a link error or worse. ⇒ Resolving the whole edge set first
/// makes the bundle's completeness a **precondition** of definition rather than
/// a discovery during it.
///
/// ⛔ **`None` from the bundle is a hard error and must never be replaced by a
/// fabricated `FuncId`.** That substitution is the exact failure
/// `UnitBundle::function`'s `Option` return type exists to make visible, and it
/// would emit a call to whatever function happened to share the id.
///
/// **MEASURED:** every `StaticBody` edge the planner validated resolves to a
/// target function the bundle declared, and the resolved population equals the
/// derived population.
/// **CLAIMED:** call sites reference target functions by their **static**
/// identity, with no indirect dispatch on a dynamic property and no runtime
/// lookup that re-derives which code to run from a value.
/// **THE GAP:** ⛔ **this resolves the edges; it does not yet EMIT the call
/// instructions.** A unit body today loads its result slot and returns, because
/// body emission does not descend until `S6` switches `lower_expr`'s consumers
/// over. ⇒ Until then this carries **rejection** authority, not emission
/// authority, and ⛔ the claim above is discharged for *resolution* only. The
/// `direct call rather than call_indirect` half of it has no control yet and is
/// **not** claimed.
///
/// ⛔⛔ **AND THE GAP IS WIDER THAN "no call instruction" — MEASURED, NOT
/// ESTIMATED.** Replacing `bundle.function(edge.callee())` with
/// `bundle.function(edge.caller())` — i.e. resolving every call edge to the
/// **calling** unit instead of the called one, the identity-alias defect
/// `UnitBundle`'s doc comment warns against — leaves the **entire suite green**:
/// 498 + 26 + 14, zero failures.
///
/// ⇒ ⭐ **Which unit an edge resolves to is currently unpinned.** The `FuncRef`
/// is declared in the caller's `Function` and never called, so a wrong target is
/// a reference nobody follows. ⛔ **`S6` must not read this as covered.** The
/// control that closes it is a *behavioural* one — a program whose answer
/// depends on which unit ran — and it cannot exist until the call is emitted.
/// ⚠ `the_resolved_call_edge_population_moves_with_the_program` pins the edge
/// **count** and is blind to the edge's **destination**; those are different
/// claims and only the first has a defender today.
/// **`D4` -- the static-body units, projected by exact body origin.**
///
/// This is a **projection of `emittable_units`**, not a new unit or call-edge
/// population: every entry is a unit the planner already emitted and the
/// bundle already declared, re-keyed by the body origin a static worker
/// binding names. Nothing here mints a unit, an edge, or a descriptor.
///
/// A body origin that appears twice is rejected rather than resolved by
/// last-writer, because a duplicate means two units claim one body and the
/// binding could not name either unambiguously.
pub(in crate::cranelift_backend) struct WorkerTargets {
    /// The EXECUTABLE targets: bodies with a declared `Function` to call.
    by_origin: BTreeMap<StaticOriginId, ResolvedUnitTarget>,
    /// **`D5a` checkpoint 1 -- the TEMPLATE population, over every emittable
    /// unit including the template-only ones.**
    ///
    /// Architect ruling `evt_5a0q3m9tnkh8e`: *"the constructor's raw
    /// identity/arity validation must be separate from the generated context
    /// `FuncRef` used by the call."* This map is that separation made
    /// structural -- it carries the descriptor facts and **no `FuncRef` at
    /// all**, so a template-only body can be validated against its own raw
    /// contract by code that has no way to call it.
    templates: BTreeMap<StaticOriginId, WorkerTemplate>,
}

/// **`D5a` checkpoint 1 -- one raw worker body's descriptor contract.**
///
/// ⛔ Deliberately has no `function: FuncRef` field. `construct_static_worker_
/// binding` reads only `call_site_origin`, `header`, `slots` and `offsets`, so
/// removing the callee from the record it validates against makes "validated
/// the raw contract, called the generated context" a fact about the types
/// rather than a discipline someone has to remember.
#[derive(Clone)]
pub(in crate::cranelift_backend) struct WorkerTemplate {
    pub(in crate::cranelift_backend) origin: StaticOriginId,
    pub(in crate::cranelift_backend) call_site_origin: StaticOriginId,
    pub(in crate::cranelift_backend) header: AbiFrameHeader,
    pub(in crate::cranelift_backend) slots: Vec<AbiSlot>,
    pub(in crate::cranelift_backend) offsets: Vec<u32>,
}

impl WorkerTargets {
    /// Declare every projected target **into one generated function**, and
    /// hand back that function's own `DeclaredUnitCall`s.
    ///
    /// The `FuncRef`s produced here belong to `func` alone. They are minted
    /// per function and never copied between functions -- which is why the
    /// binding stores origins and not a `FuncRef` (`D4`).
    ///
    /// This is also the operation a separately emitted caller uses: it takes
    /// any `Function`, so a caller emitted outside the main loop declares its
    /// own refs through the same route rather than borrowing another's.
    pub(in crate::cranelift_backend) fn declare_in_func<M: Module>(
        &self,
        module: &mut M,
        func: &mut Function,
    ) -> BTreeMap<StaticOriginId, DeclaredUnitCall> {
        self.by_origin
            .iter()
            .map(|(origin, target)| {
                (
                    *origin,
                    DeclaredUnitCall {
                        function: module.declare_func_in_func(target.function, func),
                        origin: target.origin,
                        call_site_origin: target.call_site_origin,
                        header: target.header,
                        slots: target.slots.clone(),
                        offsets: target.offsets.clone(),
                    },
                )
            })
            .collect()
    }

    /// The raw template contract for every emittable body, executable or not.
    pub(in crate::cranelift_backend) fn templates(
        &self,
    ) -> &BTreeMap<StaticOriginId, WorkerTemplate> {
        &self.templates
    }
}

/// Project the already-validated emittable units by exact body origin.
pub(in crate::cranelift_backend) fn resolve_worker_targets(
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
) -> Result<WorkerTargets, CraneliftBackendError> {
    let mut templates: BTreeMap<StaticOriginId, WorkerTemplate> = BTreeMap::new();
    // The TEMPLATE population is every emittable unit -- `D5a` checkpoint 1
    // keeps the raw worker's descriptor and source binding whether or not it
    // still receives a `Function`.
    // `D5a` checkpoint 4 step 3 -- the reaching mutation for "unchanged raw
    // worker ABI". Reading the EXECUTABLE population here instead of the
    // emittable one is the "template-only means deleted" mistake: it drops the
    // superseded body's descriptor while every consumer of that descriptor
    // remains. ⛔ The two populations are otherwise identical on a program with
    // no retarget, which is why this only became measurable at checkpoint 4.
    #[cfg(test)]
    let template_population =
        if crate::cranelift_backend::lowering::d5a_route_mutation()
            == crate::cranelift_backend::lowering::D5aRouteMutation::DropSupersededWorkerTemplates
        {
            crate::cranelift_backend::lowering::record_d5a_route_application();
            plan.executable_units()?
        } else {
            plan.emittable_units()?
        };
    #[cfg(not(test))]
    let template_population = plan.emittable_units()?;
    for unit in template_population {
        let (offsets, frame_bytes) = unit.slot_offsets()?;
        if frame_bytes != unit.header().frame_bytes {
            return Err(backend_module(
                "worker template frame size disagrees with its slot run".to_string(),
            ));
        }
        let origin = unit.origin();
        if templates
            .insert(
                origin,
                WorkerTemplate {
                    origin,
                    call_site_origin: origin,
                    header: unit.header(),
                    slots: unit.slots().to_vec(),
                    offsets,
                },
            )
            .is_some()
        {
            return Err(backend_module(
                "two emittable units claim the same body origin, so no worker template could \
                 name either unambiguously"
                    .to_string(),
            ));
        }
    }
    let mut by_origin: BTreeMap<StaticOriginId, ResolvedUnitTarget> = BTreeMap::new();
    for unit in plan.executable_units()? {
        let function = bundle.function(unit.function()).ok_or_else(|| {
            backend_module(
                "a planned unit has no forward-declared function to project as a worker target"
                    .to_string(),
            )
        })?;
        let (offsets, frame_bytes) = unit.slot_offsets()?;
        if frame_bytes != unit.header().frame_bytes {
            return Err(backend_module(
                "worker target frame size disagrees with its slot run".to_string(),
            ));
        }
        let origin = unit.origin();
        let target = ResolvedUnitTarget {
            function,
            origin,
            call_site_origin: origin,
            kind: EmittableCallKind::StaticBody,
            header: unit.header(),
            slots: unit.slots().to_vec(),
            offsets,
        };
        if by_origin.insert(origin, target).is_some() {
            return Err(backend_module(
                "two emittable units claim the same body origin, so no worker binding could                  name either unambiguously"
                    .to_string(),
            ));
        }
    }
    Ok(WorkerTargets {
        by_origin,
        templates,
    })
}

/// **`RT-DECL-CLOSURE-PORT` `D5` — the checked-call closeout ledger.**
///
/// One entry per checked same-SCC recursive call that reached a real direct
/// call to a declaration-owned unit, keyed by the `call_template_id` the
/// oriented plan issued.
///
/// ⭐ **The three populations are read independently.** `planned` is the
/// oriented plan's own `recursive_calls` identities; `consumed` is what the
/// affine marker machinery actually took; `emitted` is recorded only after the
/// `Inst` exists, from the emitted call itself. Set equality across the three
/// is what "every planned checked call became exactly one correct direct call,
/// and no other checked call was emitted" means.
///
/// ⛔ **Sets, not counts.** Two populations of the same size can differ, and a
/// length comparison would pass for one that swapped a template for another.
///
/// ⛔ **This closeout restates none of D5's other laws.** Interface, segment,
/// frame-template, occurrence-fingerprint, ABI descriptor, SCC, admission and
/// input-order checks all have their own authorities and keep them; duplicating
/// one here would put a second copy in a file where the two can disagree.
///
/// ⚠ **Scope, stated rather than left to be discovered.** The ledger is opened
/// and closed by `define_unit_bodies`, which runs **only** under
/// `BodyEmissionAuthority::FunctionizedUnits`. Production selects
/// `RecursiveDescent` until `D6` retires the `TransparentDeclarationClosure`
/// residual, so today this gate is reachable only under the `cfg(test)` selector
/// witness — the same reachability every other `D5` law has, and the thing `D6`
/// changes. It is live production code on the lane it guards, not a test hook.
#[derive(Debug, Default)]
pub(in crate::cranelift_backend) struct CheckedCallLedger {
    planned: BTreeSet<u64>,
    emitted: BTreeMap<u64, CheckedCallRecord>,
}

/// One emitted checked call, bound to the exact occurrence and target the
/// planner resolved for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) struct CheckedCallRecord {
    pub(in crate::cranelift_backend) reference: StaticOriginId,
    pub(in crate::cranelift_backend) target: StaticOriginId,
    /// The callee **decoded out of the emitted instruction**.
    pub(in crate::cranelift_backend) callee: cranelift_codegen::ir::FuncRef,
    /// The callee the planner-resolved target record carries. ⚠ Both are
    /// `FuncRef`s minted into the same defining function, so comparing them is
    /// lawful and is a comparison of two independently produced facts — one
    /// read from the CLIF, one from the resolved `DeclaredUnitCall`.
    pub(in crate::cranelift_backend) resolved: cranelift_codegen::ir::FuncRef,
}

impl CheckedCallLedger {
    /// ⚠ `planned` is taken **directly** from `plan.recursive_calls` — that IS
    /// the exact domain of same-SCC checked calls, so no classifier and no
    /// whitelist stands between the plan and this set.
    pub(in crate::cranelift_backend) fn open(
        plan: Option<&crate::OrientedSubcontinuationPlanV1>,
    ) -> Self {
        Self {
            planned: plan
                .map(|plan| {
                    plan.recursive_calls
                        .iter()
                        .map(|call| call.call_template_id)
                        .collect()
                })
                .unwrap_or_default(),
            emitted: BTreeMap::new(),
        }
    }

    /// Record one emitted checked call. ⛔ Called only **after** the `Inst`
    /// exists, so a template reaches this set only once its call is real.
    pub(in crate::cranelift_backend) fn record_emitted(
        &mut self,
        call_template_id: u64,
        record: CheckedCallRecord,
    ) -> Result<(), CraneliftBackendError> {
        if self.emitted.insert(call_template_id, record).is_some() {
            return Err(backend_module(
                "one checked recursive call template emitted more than one declaration-unit call"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// **Planned = consumed = emitted, before the artifact is published.**
    ///
    /// `consumed` is the affine marker machinery's own set, passed in rather
    /// than mirrored here, so the two cannot drift.
    pub(in crate::cranelift_backend) fn close(
        self,
        consumed: &BTreeSet<u64>,
    ) -> Result<(), CraneliftBackendError> {
        let emitted = self.emitted.keys().copied().collect::<BTreeSet<_>>();
        for (name, set) in [("consumed", consumed), ("emitted", &emitted)] {
            if *set != self.planned {
                let missing = self.planned.difference(set).count();
                let extra = set.difference(&self.planned).count();
                return Err(backend_module(format!(
                    "the {name} checked recursive call population does not equal the planned one:                      {missing} planned templates absent, {extra} unplanned templates present"
                )));
            }
        }
        // Each emitted call's ACTUAL callee against its exact resolved target.
        // ⛔ The callee is the one decoded from the emitted instruction; a
        // tuple that disagrees here is a call that went somewhere the planner
        // did not resolve.
        for (call_template_id, record) in &self.emitted {
            if record.callee != record.resolved {
                return Err(backend_module(format!(
                    "checked recursive call template {call_template_id} emitted a call to \
                     {:?} but the target resolved for occurrence {:?} is {:?} ({:?})",
                    record.callee, record.reference, record.resolved, record.target
                )));
            }
        }
        Ok(())
    }
}

pub(in crate::cranelift_backend) fn resolve_call_edges(
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
) -> Result<CallEdgeTargets, CraneliftBackendError> {
    // `D5a` checkpoint 1: the edges that SURVIVE the retarget. An edge into a
    // template-only body is the seeding edge whose realization moved to a
    // generated context; resolving it would demand a `FuncId` for a unit with
    // no emitted `Function`, and `bundle.function`'s `None` is a real answer
    // rather than a prompt to fabricate one.
    let derived = plan.executable_call_edges()?;
    // `RT-CONTSPEC-ACTIVATE` `D1b`: the exact-set source-body binding, joined
    // on validated caller + callee scheduling entry. A `StaticBody` edge's
    // resolved `call_site_origin` becomes the callable SOURCE BODY; the
    // scheduling entry stays the target origin.
    let source_bindings: BTreeMap<(PredeclaredFunctionId, StaticOriginId), StaticOriginId> = plan
        .static_body_source_bindings()?
        .into_iter()
        .map(|(caller, source_body, entry)| ((caller, entry), source_body))
        .collect();
    let mut edges = Vec::with_capacity(derived.len());
    for edge in derived {
        let target = bundle.function(edge.callee()).ok_or_else(|| {
            backend_module("a call edge names a unit that was never forward-declared".to_string())
        })?;
        let unit = plan
            .emittable_units()?
            .into_iter()
            .find(|unit| unit.function() == edge.callee())
            .ok_or_else(|| backend_module("call edge callee has no abi descriptor".to_string()))?;
        if unit.origin() != edge.callee_origin() {
            return Err(backend_module(
                "call edge callee origin disagrees with its abi descriptor".to_string(),
            ));
        }
        let (offsets, frame_bytes) = unit.slot_offsets()?;
        if frame_bytes != unit.header().frame_bytes {
            return Err(backend_module(
                "call edge target frame size disagrees with its slot run".to_string(),
            ));
        }
        edges.push((
            edge.caller(),
            ResolvedUnitTarget {
                function: target,
                origin: edge.callee_origin(),
                call_site_origin: match edge.kind() {
                    EmittableCallKind::StaticBody => *source_bindings
                        .get(&(edge.caller(), edge.callee_origin()))
                        .ok_or_else(|| {
                            backend_module(
                                "a static body call edge has no D1b source-body binding"
                                    .to_string(),
                            )
                        })?,
                    EmittableCallKind::Declaration => edge.call_site_origin(),
                },
                kind: edge.kind(),
                header: unit.header(),
                slots: unit.slots().to_vec(),
                offsets,
            },
        ));
    }
    #[cfg(test)]
    B2F_CALL_EDGE_RESOLUTION.with(|cell| cell.set(edges.len()));
    Ok(CallEdgeTargets { edges })
}

/// **`RT-DECL-CLOSURE-PORT` `D5`** — one mutation of the function-local
/// declared-call copy, per axis the ABI reconciliation claims to hold.
///
/// ⚠ Every variant leaves the plan's descriptor untouched. `Exact` is the
/// identity, and a control that ran only `Exact` would be asserting its own
/// setup ([[a-mutation-control-with-unwrap-or-exact-is-the-identity]]).
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum D5DeclaredCallMutation {
    Exact,
    /// Phase: the word class a slot is carried in.
    Carrier,
    /// Owner: whether the callee owns or borrows the slot.
    Ownership,
    /// Owner: whose storage the slot addresses.
    StorageOwner,
    /// The slot's position within its own kind-run.
    Ordinal,
    /// The declared frame size.
    Header,
    /// Where a slot sits in the frame.
    Offsets,
    /// The call resolves some other unit's record — the wrong-target class.
    Retarget,
}

#[cfg(test)]
thread_local! {
    static D5_DECLARED_CALL_MUTATION: std::cell::Cell<D5DeclaredCallMutation> =
        const { std::cell::Cell::new(D5DeclaredCallMutation::Exact) };
}

/// Run `body` with one declared-call mutation installed, restoring `Exact` on
/// the way out **including on panic** — a control asserts inside, and a leak
/// would silently mutate every later compile on this thread.
#[cfg(test)]
pub(in crate::cranelift_backend) fn with_d5_declared_call_mutation<T>(
    mutation: D5DeclaredCallMutation,
    body: impl FnOnce() -> T,
) -> T {
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            D5_DECLARED_CALL_MUTATION.with(|cell| cell.set(D5DeclaredCallMutation::Exact));
        }
    }
    D5_DECLARED_CALL_MUTATION.with(|cell| cell.set(mutation));
    let _restore = Restore;
    body()
}

#[cfg(test)]
fn d5_mutate_declared_calls(calls: &mut BTreeMap<StaticOriginId, DeclaredUnitCall>) {
    let mutation = D5_DECLARED_CALL_MUTATION.with(std::cell::Cell::get);
    if mutation == D5DeclaredCallMutation::Exact {
        return;
    }
    // ⚠ The retarget needs a DIFFERENT record to point at, so it is taken
    // before the loop below borrows the map mutably.
    let other = calls.values().next().cloned();
    for call in calls.values_mut() {
        match mutation {
            D5DeclaredCallMutation::Exact => {}
            D5DeclaredCallMutation::Carrier => {
                if let Some(slot) = call.slots.iter_mut().find(|slot| {
                    matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture)
                }) {
                    slot.carrier = match slot.carrier {
                        AbiCarrier::ValueWord => AbiCarrier::GroundValueCarrier,
                        _ => AbiCarrier::ValueWord,
                    };
                }
            }
            D5DeclaredCallMutation::Ownership => {
                if let Some(slot) = call.slots.iter_mut().find(|slot| {
                    matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture)
                }) {
                    slot.ownership = match slot.ownership {
                        AbiOwnership::OwnedByFrame => AbiOwnership::BorrowedForActivation,
                        _ => AbiOwnership::OwnedByFrame,
                    };
                }
            }
            D5DeclaredCallMutation::StorageOwner => {
                if let Some(slot) = call.slots.iter_mut().find(|slot| {
                    matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture)
                }) {
                    slot.storage_owner = match slot.storage_owner {
                        AbiStorageOwner::ActivationFrame => AbiStorageOwner::PersistentStore,
                        _ => AbiStorageOwner::ActivationFrame,
                    };
                }
            }
            D5DeclaredCallMutation::Ordinal => {
                if let Some(slot) = call.slots.iter_mut().find(|slot| {
                    matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture)
                }) {
                    slot.ordinal = slot.ordinal.wrapping_add(1);
                }
            }
            D5DeclaredCallMutation::Header => {
                call.header.frame_bytes = call.header.frame_bytes.wrapping_add(8);
            }
            D5DeclaredCallMutation::Offsets => {
                if let Some(offset) = call.offsets.first_mut() {
                    *offset = offset.wrapping_add(8);
                }
            }
            D5DeclaredCallMutation::Retarget => {
                if let Some(other) = other.clone() {
                    if other.origin != call.origin {
                        *call = other;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
thread_local! {
    /// How many call edges the most recent compile resolved.
    ///
    /// ⚠ Same limitation as [`b2f_last_unit_emission`]: it names no attempt, so
    /// read it only where one compile is known to have run to this seam.
    static B2F_CALL_EDGE_RESOLUTION: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The resolved call-edge count from the most recent compile.
#[cfg(test)]
pub(in crate::cranelift_backend) fn b2f_last_call_edge_resolution() -> usize {
    B2F_CALL_EDGE_RESOLUTION.with(std::cell::Cell::get)
}

/// The uniform internal call ABI for every target unit:
/// `(frame_ptr, services_ptr) -> i64`.
///
/// ⭐ **This is what "one fixed call-ABI scheme, not one fixed byte size"
/// means.** Every unit shares this signature; what varies per origin is the
/// *frame layout* the pointer addresses, which `B2R` declares per unit in
/// `AbiFrameHeader` + the slot run. ⛔ Reading "fixed frame" as one universal
/// byte size is the error that would reintroduce a boxed `Value` nobody asked
/// for, and `B2R` says so explicitly.
///
/// ⚠ The signature takes **no program-derived parameter**, which is the same
/// structural guarantee `AC-G0` accepts for `emit_native_int_local_graph`:
/// making a unit's *signature* vary with the program would require a visible
/// change here, so the compiler forbids that growth mode rather than a test
/// detecting it.
pub(super) fn unit_signature<M: Module>(module: &M) -> cranelift_codegen::ir::Signature {
    let mut sig = module.make_signature();
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.params
        .push(AbiParam::new(module.target_config().pointer_type()));
    sig.returns.push(AbiParam::new(types::I64));
    sig
}

/// **`D1` — forward-declare the whole bundle before any body is defined.**
///
/// ⭐ **The bundle is declared in one pass on purpose.** A unit's body may call
/// any other unit (that is what `D4`'s cross-owner call edges are), so a
/// declare-and-define-in-one-pass loop would be unable to emit a call to a unit
/// it has not reached yet. Declaring every signature first makes the call graph
/// order-independent, which is why the frame words `D1` as *"forward-declare the
/// whole bundle first, then define each body."*
pub(in crate::cranelift_backend) fn declare_unit_bundle<M: Module>(
    module: &mut M,
    plan: &StaticTransitionPlan<'_>,
) -> Result<UnitBundle, CraneliftBackendError> {
    let sig = unit_signature(module);
    let mut functions = BTreeMap::new();
    // `D5a` checkpoint 1: the EXECUTABLE population, not the template one. ⛔ A
    // template-only raw worker must not be declared here -- declaring it and
    // then not defining it is the undefined phantom the ruling names, and it
    // would falsify the declared/defined census below.
    for (ordinal, unit) in plan.executable_units()?.into_iter().enumerate() {
        // The symbol carries the dense ordinal purely so the linker sees
        // distinct names. ⛔ It is NOT an identity: nothing resolves a unit by
        // parsing this string, and `functions` is keyed by the planner's id.
        let name = format!("ken_unit_{ordinal}");
        let id = module
            .declare_function(&name, Linkage::Local, &sig)
            .map_err(|err| backend_module(err.to_string()))?;
        if functions.insert(unit.function(), id).is_some() {
            // ⛔ Fails closed rather than overwriting. `B2R` gives exactly one
            // descriptor per `PredeclaredFunction`, so a duplicate here means
            // the plane disagrees with that invariant, and silently keeping the
            // last one would emit a bundle whose call edges resolve to the
            // wrong body.
            return Err(backend_module(
                "two abi descriptors claim one predeclared function unit".to_string(),
            ));
        }
    }
    // `RT-CONTSPEC-ACTIVATE` `D2` -- forward-declare one target per planned
    // continuation specialization, before any body is defined. The symbol
    // carries a dense ordinal only so the linker sees distinct names; the map
    // is keyed by the planner's typed identity, never by that string.
    let mut continuations = BTreeMap::new();
    for (ordinal, unit) in plan.continuation_units()?.into_iter().enumerate() {
        let name = format!("ken_continuation_{ordinal}");
        let id = module
            .declare_function(&name, Linkage::Local, &sig)
            .map_err(|err| backend_module(err.to_string()))?;
        if continuations.insert(unit.id(), id).is_some() {
            return Err(backend_module(
                "two continuation descriptors claim one planned specialization".to_string(),
            ));
        }
    }
    // `RT-DECL-CLOSURE-PORT` `D5a` -- forward-declare one target per planned
    // generated producer execution context, in the same pre-definition pass and
    // for the same reason: a context is called from the enclosing
    // specialization's body, which is defined below.
    let mut contexts = BTreeMap::new();
    for (ordinal, context) in plan.continuation_contexts()?.into_iter().enumerate() {
        let name = format!("ken_continuation_context_{ordinal}");
        let id = module
            .declare_function(&name, Linkage::Local, &sig)
            .map_err(|err| backend_module(err.to_string()))?;
        if contexts.insert(context.id(), id).is_some() {
            return Err(backend_module(
                "two generated context descriptors claim one planned context".to_string(),
            ));
        }
    }
    #[cfg(test)]
    B2F_UNIT_EMISSION.with(|cell| cell.set((functions.len(), 0)));
    Ok(UnitBundle {
        functions,
        continuations,
        contexts,
    })
}

/// **`RT-CONTSPEC-ACTIVATE` `D2` — resolve every projected causal identity to
/// its typed declared target.**
///
/// Runs after declaration and before any body is defined, for the same reason
/// `resolve_call_edges` does: a causal edge whose target was never declared is
/// a program that cannot be emitted, and discovering that while half the
/// bodies exist leaves a partially emitted artifact.
///
/// The join is by the identity's `target()` alone. ⛔ Nothing here parses a
/// symbol name, indexes by ordinal, or aliases a `ContinuationSpecializationId`
/// to a `PredeclaredFunctionId` — a missing target rejects.
pub(in crate::cranelift_backend) fn resolve_continuation_targets(
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
) -> Result<BTreeMap<ContinuationCallIdentity, FuncId>, CraneliftBackendError> {
    let mut resolved = BTreeMap::new();
    for call in plan.continuation_calls()? {
        let identity = plan
            .continuation_call_binding_for(
                call.producer_construct_origin(),
                call.continuation_origin(),
                call.producer_alternative(),
                call.recursive_position(),
            )?
            .ok_or_else(|| {
                backend_module(
                    "a projected causal call has no binding under its own four-field selector"
                        .to_string(),
                )
            })?;
        let target = bundle.continuation(identity.target()).ok_or_else(|| {
            backend_module(
                "a projected causal identity names a continuation specialization that was never \
                 forward-declared"
                    .to_string(),
            )
        })?;
        if resolved.insert(identity, target).is_some() {
            return Err(backend_module(
                "two projected causal identities collide; the planner mints one call token per \
                 ruled recursive position, so this is a key-arity defect rather than a \
                 double-resolution"
                    .to_string(),
            ));
        }
    }
    Ok(resolved)
}

/// **`RT-DECL-CLOSURE-PORT` `D5a`** — one entry of a continuation
/// specialization body's case environment, named by **where its operand comes
/// from** rather than by the operand.
///
/// ⭐ The whole property under repair here is an **order**, so the order is
/// what this type makes observable. Cranelift `Value`s cannot be synthesized
/// without a live `FunctionBuilder`, so a control written against the assembled
/// operands could only ever re-run the pipeline and read a refusal; a control
/// written against this plan states the binding law directly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::cranelift_backend) enum ContinuationCaseBinderSource {
    /// The projected `StaticWorker`, standing for one recursive field's
    /// **induction hypothesis**.
    ///
    /// `D6a`: this binding takes the
    /// [`StaticWorkerCallRoute::GeneratedContext`] route -- the planner-issued
    /// execution context, which appends this frame's continuation-input suffix
    /// -- **iff** the planner issued such a context for this
    /// `(specialization, worker body)` pair and this unit resolved it.
    /// Otherwise it lawfully takes [`StaticWorkerCallRoute::RawWorker`], like
    /// every pre-`D5a` specialization, and appends nothing.
    InductionHypothesis,
    /// The ordinary-envelope operand at this index. ⛔ An index into the
    /// envelope, never a constructor source position -- the two coincide only
    /// when no `WorkerCapture` role precedes the field.
    Ordinary(usize),
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D6a`** -- the selected **recursive
    /// constructor argument**, at its own constructor source position.
    ///
    /// ⭐ This is a compiler-only member. It is not a new source occurrence,
    /// continuation input, ABI slot, carrier, tag or runtime descriptor: it is
    /// the *same closure* the induction hypothesis names. The unit already
    /// carries every fact needed to build it -- the closure occurrence, body,
    /// declared arity, ordered capture provenance and the worker-capture
    /// operands -- so nothing crosses the ABI to represent it.
    ///
    /// It carries [`StaticWorkerCallRoute::RawWorker`] **unconditionally**: the
    /// source scope binds the closure itself, so there is nothing to condition
    /// on. ⛔ That is *not* a claim that it differs in route from the induction
    /// hypothesis beside it. In a unit that resolved no generated context the
    /// hypothesis lawfully carries `RawWorker` too, and the two members are
    /// then separated by their positions in the run rather than by their
    /// routes. See [`StaticWorkerCallRoute`] for the asymmetric law.
    ///
    /// ⛔ Before `D6a` this position was **skipped**, and the induction
    /// hypothesis silently stood in for the argument as well. That is a wrong
    /// program, not a missing one: every later binder shifted down by one, so
    /// the case body's outer-frame references landed one slot early.
    SelectedRecursiveArgument { source_position: u32 },
    /// The continuation input at this ordinal.
    ContinuationInput(usize),
}

/// **`RT-DECL-CLOSURE-PORT` `D5a` — the ruled computational-case binding law,
/// as a plan.**
///
/// The established law, identical in the specialized and carried paths
/// (`lower_computational_match_value_composed` builds the same three segments
/// from `induction_hypotheses`, `extend_specialized(args)`, then the frame
/// environment):
///
/// ```text
/// [IH bindings, recursive-position order REVERSED]
///   ++ [ALL constructor arguments, source order]
///   ++ [frame/tail environment]
/// ```
///
/// ⭐ **`RT-CONTSRC-PRODUCER-LOCAL` `D6a` -- "ALL" is load-bearing and was the
/// defect.** The middle segment covers **every** constructor argument in source
/// order, the selected recursive one included. The pre-`D6a` construction
/// *replaced* the selected recursive argument with its own induction
/// hypothesis: it emitted the IH in segment 1 and then skipped that position in
/// segment 2, so a case body with one recursive field and one outer reference
/// got a two-member run where the source scope has three. Every binder after
/// the skipped position was off by one, which is why the measured symptom was
/// an out-of-range `Var` at the *tail* (`Var: no runtime binding for index 2`)
/// rather than anything at the position actually omitted.
///
/// ⭐ **`recursive_position` is a constructor SOURCE-FIELD coordinate, not a
/// lexical environment index.** Reading it as the latter is the exact defect
/// this function exists to prevent: it placed the worker at environment slot
/// `recursive_position`, which for a nonzero position moves the induction
/// hypothesis out of its established lexical prefix and rebinds `Var(0)` to an
/// ordinary field. The measured consequence on `px8tr_nested_post_effect` was
/// `Unsupported(Call, "callee is not a closure")` -- `Var(0)` reading a `Unit`.
///
/// The specialization eliminates **one** selected recursive callable, so its
/// projected `StaticWorker` is that position's IH-prefix binding.
///
/// The selected recursive field is **absent from the ordinary envelope** --
/// [`ContinuationOrdinaryEnvelopeRole::NonrecursiveConstructorField`] is by
/// construction the population that excludes it. ⛔ `D6a`: that is a fact about
/// the *envelope*, and reading it as a fact about the *binder run* is the
/// defect. The field has no envelope operand because it needs none: it is a
/// compiler-only [`ContinuationCaseBinderSource::SelectedRecursiveArgument`],
/// built from the unit's own worker provenance. So the run's length is
/// `argument_binders + recursive_positions.len()` -- one IH per recursive
/// position, then one member per constructor argument -- which is exactly the
/// binder count the planner's own demand walk uses for this case.
///
/// ⛔ Worker-capture `Parameter` slots are **not** case binders. They construct
/// the `StaticWorker`; they never enter this run.
///
/// Every gap is a hard stop rather than a hole: a gap-filled run would silently
/// shift every later binder, which is a wrong program rather than a refused one.
pub(super) fn continuation_case_binder_run(
    argument_binders: usize,
    recursive_positions: &[usize],
    worker_recursive_position: u32,
    envelope: &[ContinuationOrdinaryEnvelopeRole],
    continuation_inputs: usize,
) -> Result<Vec<ContinuationCaseBinderSource>, CraneliftBackendError> {
    for position in recursive_positions.iter().copied() {
        if position >= argument_binders {
            return Err(backend_module(format!(
                "the selected case names a recursive position {position} outside its own \
                 {argument_binders}-binder run"
            )));
        }
    }
    let worker_position = usize::try_from(worker_recursive_position).map_err(|_| {
        backend_module(
            "the ruled recursive position exceeds this platform's index width".to_string(),
        )
    })?;
    if !recursive_positions.contains(&worker_position) {
        return Err(backend_module(format!(
            "the ruled recursive position {worker_position} is not among the selected case's \
             recursive positions, so the projected worker stands for no induction hypothesis"
        )));
    }

    let mut run =
        Vec::with_capacity(argument_binders + recursive_positions.len() + continuation_inputs);

    // Segment 1 -- the IH prefix, recursive positions reversed.
    //
    // ⚠ **The reversal is not observable on any case this mechanism accepts.**
    // A specialization projects exactly one worker, so a second recursive
    // position has no IH to bind and hard-stops below; with one position,
    // reversed and forward order coincide. It is written as the law states it
    // rather than collapsed to the single-position case, so that admitting a
    // second worker later is a change to the projection and not to this order.
    // `D6c` — FABRICATED AVAILABILITY, under test only. A second recursive
    // position is claimed for this case, which the specialization projects no
    // worker for. ⛔ The claim is added to the segment's own input rather than
    // to its output: the loop below is untouched, so what refuses is the
    // production guard that owns availability, not a rewritten loop.
    #[cfg(test)]
    let fabricated: Vec<usize>;
    #[cfg(test)]
    let recursive_positions = if crate::cranelift_backend::lowering::d6c_selection_mutation()
        == crate::cranelift_backend::lowering::D6cSelectionMutation::FabricatedAvailability
    {
        crate::cranelift_backend::lowering::record_d6c_selection_application();
        fabricated = recursive_positions
            .iter()
            .copied()
            .chain(std::iter::once(worker_position.wrapping_add(1)))
            .collect();
        fabricated.as_slice()
    } else {
        recursive_positions
    };

    for position in recursive_positions.iter().rev().copied() {
        if position != worker_position {
            return Err(backend_module(format!(
                "the selected case has a recursive position {position} that the continuation \
                 specialization projects no worker for, so its induction-hypothesis prefix cannot \
                 be built"
            )));
        }
        run.push(ContinuationCaseBinderSource::InductionHypothesis);
    }

    // Segment 2 -- ALL the constructor arguments in SOURCE order.
    //
    // A nonrecursive field takes its operand from its own envelope role. A
    // recursive field takes the compiler-only `SelectedRecursiveArgument`
    // member: it is the same closure the IH prefix names, bound at a second
    // environment position and reached by its own call route, so it needs no
    // envelope operand and no ABI slot.
    //
    // ⛔ "Differing only in call route" would overstate it. The routes differ
    // only where the planner issued a generated context; where it issued none,
    // both members carry `RawWorker` and the difference is *which position of
    // the run they occupy* -- which is precisely the difference this segment
    // exists to restore.
    //
    // ⛔ `D6a`: this loop used to `continue` on a recursive position. The IH
    // then stood in for the argument as well as for the hypothesis, and every
    // later binder shifted down one slot.
    for position in 0..argument_binders {
        let source_position = u32::try_from(position).map_err(|_| {
            backend_module(
                "a continuation case binder position exceeds the planner's field width".to_string(),
            )
        })?;
        if recursive_positions.contains(&position) {
            // ⛔ The hard stop for an unprojected recursive position is
            // segment 1's, and it has already fired: a position that is not the
            // ruled `worker_position` never reaches here. So this member is
            // always the *selected* recursive argument, and `D6a` deliberately
            // does not generalize to a multi-worker population.
            //
            // `D6c` — the four run-shape mutations, under test only. Each moves
            // ONE producer input of this run and leaves the rest of the segment
            // exactly as it was, so a refusal downstream is attributable to that
            // input rather than to a rewritten builder.
            #[cfg(test)]
            match crate::cranelift_backend::lowering::d6c_selection_mutation() {
                // The pre-`D6a` defect exactly: the position is skipped and the
                // IH stands in for the argument as well.
                crate::cranelift_backend::lowering::D6cSelectionMutation::OmitSelectedArgument => {
                    crate::cranelift_backend::lowering::record_d6c_selection_application();
                    continue;
                }
                // One run naming two selected arguments.
                crate::cranelift_backend::lowering::D6cSelectionMutation::DuplicateSelectedArgument => {
                    crate::cranelift_backend::lowering::record_d6c_selection_application();
                    run.push(ContinuationCaseBinderSource::SelectedRecursiveArgument {
                        source_position,
                    });
                }
                // A source position the unit projects no worker for. ⛔ Chosen
                // by arithmetic on the ruled position rather than from the
                // envelope, so the value is not one the plan named anywhere.
                crate::cranelift_backend::lowering::D6cSelectionMutation::WrongSourcePosition => {
                    crate::cranelift_backend::lowering::record_d6c_selection_application();
                    run.push(ContinuationCaseBinderSource::SelectedRecursiveArgument {
                        source_position: source_position.wrapping_add(1),
                    });
                    continue;
                }
                _ => {}
            }
            run.push(ContinuationCaseBinderSource::SelectedRecursiveArgument { source_position });
            continue;
        }
        let index = envelope
            .iter()
            .position(|role| {
                matches!(
                    role,
                    ContinuationOrdinaryEnvelopeRole::NonrecursiveConstructorField {
                        source_position: candidate,
                    } if *candidate == source_position
                )
            })
            .ok_or_else(|| {
                backend_module(format!(
                    "the ordinary envelope has no nonrecursive field at source position \
                     {source_position}, so the selected case's binder run cannot be built"
                ))
            })?;
        run.push(ContinuationCaseBinderSource::Ordinary(index));
    }

    // `D6c` — WRONG ORDER, under test only. The IH prefix and the argument
    // segment are exchanged and nothing else moves: the same members, the same
    // count, the same continuation-input tail. ⛔ Applied before segment 3 so
    // the tail stays in its ruled place and the perturbation is confined to the
    // two segments whose order is the law under test.
    #[cfg(test)]
    if crate::cranelift_backend::lowering::d6c_selection_mutation()
        == crate::cranelift_backend::lowering::D6cSelectionMutation::WrongOrder
    {
        let hypotheses = run
            .iter()
            .filter(|source| {
                matches!(source, ContinuationCaseBinderSource::InductionHypothesis)
            })
            .count();
        if hypotheses > 0 && hypotheses < run.len() {
            crate::cranelift_backend::lowering::record_d6c_selection_application();
            run.rotate_left(hypotheses);
        }
    }

    // Segment 3 -- the tail environment: this frame's continuation inputs, in
    // their ruled ordinal order.
    for ordinal in 0..continuation_inputs {
        run.push(ContinuationCaseBinderSource::ContinuationInput(ordinal));
    }
    Ok(run)
}

/// **`RT-CONTSPEC-ACTIVATE` `D2` — define each declared continuation target
/// from its own projected contract.**
///
/// Operands come from the **descriptor**: each `Parameter` and `Capture` slot
/// is loaded at the offset `slot_offsets` assigns it. ⛔ Function parameter 0
/// is not the payload, and the `Result` slot is never read -- it is
/// caller-initialized, and this body only writes it.
///
/// The partition is the ruled one: `Parameter` operands are the ordinary
/// envelope (nonrecursive producer fields, then selected worker captures in
/// capture-ordinal order), and `Capture` operands are the continuation inputs
/// by ordinal. The worker binding is built by the **existing** static-worker
/// constructor, and the semantic environment is the sole
/// `LoweringEnvironmentBinding` authority -- no parallel operand map and no
/// worker-body de Bruijn table.
pub(super) fn define_continuation_bodies<M: Module>(
    module: &mut M,
    compiler: &mut Lowering<'_>,
    helpers: ArtifactHelpers<'_>,
    bundle: &UnitBundle,
) -> Result<usize, CraneliftBackendError> {
    // Own every projected fact BEFORE the loop: the projection borrows the
    // plan, and the definition below needs the compiler mutably.
    struct OwnedContinuationEmission {
        id: ContinuationSpecializationId,
        slots: Vec<AbiSlot>,
        offsets: Vec<u32>,
        envelope: Vec<ContinuationOrdinaryEnvelopeRole>,
        inputs: Vec<ContinuationInputView>,
        continuation_origin: StaticOriginId,
        producer_alternative: u32,
        recursive_position: u32,
        worker_closure_origin: StaticOriginId,
        worker_body_origin: StaticOriginId,
        worker_declared_arity: u32,
        worker_capture_count: usize,
        header_parameters: u32,
        header_captures: u32,
        /// `D8o` — the owner of the source body this specialization lowers: the
        /// continuation's own consumer. ⛔ Carried from the planner view rather
        /// than derived here; it is an existing planner fact reaching the site
        /// that needs it, not a new authority.
        consumer_owner: PredeclaredFunctionId,
    }
    // `RT-WORKER-BIND` `D4` exposes its local declaration operation for a
    // separately emitted caller; a continuation function is exactly that, so
    // it declares its own worker refs rather than borrowing another's.
    let worker_targets = resolve_worker_targets(&compiler.static_transition_plan, bundle)?;
    let emissions = compiler
        .static_transition_plan
        .continuation_units()?
        .into_iter()
        .map(|unit| {
            let (offsets, _frame_bytes) = unit.slot_offsets()?;
            Ok(OwnedContinuationEmission {
                id: unit.id(),
                slots: unit.slots().to_vec(),
                offsets,
                envelope: unit.ordinary_envelope()?,
                inputs: unit.continuation_inputs()?,
                continuation_origin: unit.continuation_origin(),
                producer_alternative: unit.producer_alternative(),
                recursive_position: unit.recursive_position(),
                worker_closure_origin: unit.worker_closure_origin(),
                worker_body_origin: unit.worker_body_origin(),
                worker_declared_arity: unit.worker_declared_arity(),
                worker_capture_count: unit.worker_capture_count(),
                header_parameters: unit.header().parameters,
                header_captures: unit.header().captures,
                consumer_owner: unit.consumer_owner(),
            })
        })
        .collect::<Result<Vec<_>, CraneliftBackendError>>()?;

    // Shape index for the `D4` redirect control: declared arity and capture
    // count per specialization, which is the only same-shaped definition.
    let mut defined = 0usize;
    for unit in emissions {
        // Resolve the EXACT target first; the control below perturbs only what
        // the definition is handed.
        let exact_id = bundle.continuation(unit.id).ok_or_else(|| {
            backend_module(
                "a planned continuation specialization was never forward-declared".to_string(),
            )
        })?;
        let id = exact_id;

        let offsets = unit.offsets.as_slice();
        let envelope = &unit.envelope;
        let inputs = &unit.inputs;
        let slots = unit.slots.as_slice();
        if slots.len() != offsets.len() {
            return Err(backend_module(
                "a continuation slot run disagrees with its own offset walk".to_string(),
            ));
        }

        // Reject BEFORE definition on partition incompleteness: the ordinary
        // envelope must cover every Parameter slot, and the continuation
        // inputs must cover every Capture slot densely by ordinal.
        let parameter_slots: Vec<_> = slots
            .iter()
            .zip(offsets)
            .filter(|(slot, _)| slot.kind == AbiSlotKind::Parameter)
            .collect();
        let capture_slots: Vec<_> = slots
            .iter()
            .zip(offsets)
            .filter(|(slot, _)| slot.kind == AbiSlotKind::Capture)
            .collect();
        if parameter_slots.len() != envelope.len() {
            return Err(backend_module(
                "the ruled ordinary envelope does not cover the Parameter slot run".to_string(),
            ));
        }
        if capture_slots.len() != inputs.len() {
            return Err(backend_module(
                "the projected continuation inputs do not cover the Capture slot run".to_string(),
            ));
        }
        for (position, input) in inputs.iter().enumerate() {
            if input.ordinal as usize != position {
                return Err(backend_module(
                    "continuation inputs are not dense in ordinal order".to_string(),
                ));
            }
        }
        // Provenance: every worker capture role must name the worker this key
        // selected, so an envelope built against another closure rejects.
        for role in envelope.iter() {
            if let ContinuationOrdinaryEnvelopeRole::WorkerCapture { closure_origin, .. } = role {
                if *closure_origin != unit.worker_closure_origin {
                    return Err(backend_module(
                        "an ordinary-envelope worker capture names a different closure than the \
                         selected worker"
                            .to_string(),
                    ));
                }
            }
        }

        compiler.open_aggregate_events(id)?;
        let sig = unit_signature(module);
        let mut func =
            Function::with_name_signature(UserFuncName::user(0, id.as_u32()), sig);
        // Set by the retarget below; `None` means this specialization calls the
        // raw worker unit directly, which is every pre-`D5a` case.
        let mut retargeted_worker_body: Option<StaticOriginId> = None;
        let result_offset = slots
            .iter()
            .zip(offsets)
            .find(|(slot, _)| slot.kind == AbiSlotKind::Result)
            .map(|(_, offset)| *offset)
            .ok_or_else(|| {
                backend_module("continuation frame declares no result slot".to_string())
            })?;
        let trap_offset = slots
            .iter()
            .zip(offsets)
            .find(|(slot, _)| slot.kind == AbiSlotKind::Trap)
            .map(|(_, offset)| *offset)
            .ok_or_else(|| {
                backend_module("continuation frame declares no trap slot".to_string())
            })?;

        let mut function_local = helpers.declare_in_func(module, &mut func, None);
        // ONE lawful declaration per continuation `Function`, retained whole
        // and seated in BOTH existing roles. The worker constructor validates
        // through `unit_calls`; the later callee-only consumer resolves
        // through `worker_calls`. Seating only the second is what made the
        // constructor refuse -- the objects are the same, the roles are not.
        //
        // Declared here, into THIS function: no `FuncRef` crosses a function.
        let declared_workers = worker_targets.declare_in_func(module, &mut func);
        function_local.unit_calls = declared_workers.clone();
        // `D6b` -- the RAW route's table, captured before the retarget below
        // rewrites `worker_calls`. This is the only point at which the raw
        // callee for a retargeted body is still in hand.
        function_local.raw_worker_calls = declared_workers.clone();
        function_local.worker_templates = worker_targets.templates().clone();
        function_local.context_calls = declare_context_calls_in_func(
            module,
            &mut func,
            &compiler.static_transition_plan,
            bundle,
        )?;
        // `RT-DECL-CLOSURE-PORT` `D5a` -- THE RETARGET.
        //
        // If this specialization's worker body has a generated execution
        // context, the worker call resolves to that context instead of to the
        // raw unit. ⛔ Only `worker_calls` moves: `unit_calls` above keeps the
        // raw target, because the static-worker CONSTRUCTOR validates against
        // the raw body's own contract and this retarget does not change what
        // that body is.
        //
        // ⭐ Why this is what makes the whole thing work: the continuation
        // inputs live in THIS frame's Capture slots. The raw unit's ABI has
        // nowhere to put them, so calling the raw unit drops them before the
        // nested producer is reached -- which is exactly the measured defect.
        // The context's ABI has a capture run for them, so the call carries
        // them across the checked-IH worker execution.
        let mut worker_calls = declared_workers;
        // `D5a` checkpoint 4 step 3 -- the binding's three reaching mutations.
        //
        // ⛔ `Suppress` and `Transplant` perturb WHICH context the retarget is
        // handed; the exact lookup itself is untouched, so a refusal downstream
        // is attributable to the binding and not to a rewritten resolver.
        // `Transplant` declines when this unit has no foreign context to be
        // given -- it bumps the application counter when it does fire, so a
        // control can require the perturbation actually reached the seat rather
        // than reading a green as a defence.
        #[cfg(test)]
        let resolved_context = match crate::cranelift_backend::lowering::d5a_route_mutation() {
            crate::cranelift_backend::lowering::D5aRouteMutation::SuppressContextBinding => {
                crate::cranelift_backend::lowering::record_d5a_route_application();
                None
            }
            crate::cranelift_backend::lowering::D5aRouteMutation::TransplantContextBinding => {
                let foreign = compiler
                    .static_transition_plan
                    .continuation_contexts()?
                    .into_iter()
                    .find(|context| context.enclosing_specialization() != unit.id);
                match foreign {
                    Some(context) => {
                        crate::cranelift_backend::lowering::record_d5a_route_application();
                        Some(context)
                    }
                    None => compiler
                        .static_transition_plan
                        .continuation_context_for(unit.id, unit.worker_body_origin)?,
                }
            }
            _ => compiler
                .static_transition_plan
                .continuation_context_for(unit.id, unit.worker_body_origin)?,
        };
        #[cfg(not(test))]
        let resolved_context = compiler
            .static_transition_plan
            .continuation_context_for(unit.id, unit.worker_body_origin)?;
        if let Some(context) = resolved_context {
            // `D5a` checkpoint 4 step 3 -- THE TRANSPLANT STOP.
            //
            // ⭐⭐ Added because a transplant was **measured to compile**. The
            // resolved context used to be trusted wholesale and the record
            // below took its `origin` from `unit.worker_body_origin` -- the
            // asking unit's own value. `call_static_worker`'s
            // `target.origin != worker.body_origin` check therefore compared
            // that value with itself on this path and could not see a foreign
            // context at all. Handed one context in place of another, lowering
            // emitted a call that type-checked (the capture suffix made the
            // operand run agree) and transferred to a function executing a
            // DIFFERENT body.
            //
            // ⛔ Production never reaches that state: `continuation_context_for`
            // is keyed by `(enclosing, worker_body)` and is the only producer.
            // ⇒ But "unreachable by construction" was carrying the whole
            // guarantee here, with no check able to observe a violation, so the
            // ruling's transplanted-binding stop had nothing to name. These two
            // comparisons cost nothing and turn it into a fact the code checks.
            if context.enclosing_specialization() != unit.id {
                return Err(backend_module(format!(
                    "the retarget resolved generated context {:?}, whose enclosing specialization \
                     is {:?} and not the {:?} now being defined; a context executes on behalf of \
                     the one identity that owns it, so this call would transfer another \
                     specialization's captures across this one's worker execution",
                    context.id(),
                    context.enclosing_specialization(),
                    unit.id,
                )));
            }
            if context.worker_body_origin() != unit.worker_body_origin {
                return Err(backend_module(format!(
                    "the retarget resolved a generated context executing body {:?}, but this \
                     specialization selected worker body {:?}",
                    context.worker_body_origin(),
                    unit.worker_body_origin,
                )));
            }
            let target = bundle.context(context.id()).ok_or_else(|| {
                backend_module(
                    "a planned generated context was never forward-declared".to_string(),
                )
            })?;
            let (context_offsets, _frame_bytes) = context.slot_offsets()?;
            worker_calls.insert(
                unit.worker_body_origin,
                DeclaredUnitCall {
                    function: module.declare_func_in_func(target, &mut func),
                    // The context EXECUTES that body, so the origin it answers
                    // for is unchanged. ⛔ Read from the CONTEXT, not from the
                    // asking unit: taking it from `unit` is what made
                    // `call_static_worker`'s origin check self-referential
                    // here. The equality above is what lets both readings agree.
                    origin: context.worker_body_origin(),
                    call_site_origin: context.worker_body_origin(),
                    header: context.header(),
                    slots: context.slots().to_vec(),
                    offsets: context_offsets,
                },
            );
            retargeted_worker_body = Some(unit.worker_body_origin);
        }
        // ⛔ Checked AFTER the retarget, not before it. `D5a` checkpoint 4
        // step 2 removes a fully retargeted raw worker from the emitted
        // `Function` population, so demanding an *emittable-unit* target here
        // would reject exactly the case the retarget exists to serve. What this
        // specialization actually needs is a declared callee for its worker
        // body -- raw or generated -- and that is what is required.
        if !worker_calls.contains_key(&unit.worker_body_origin) {
            return Err(backend_module(
                "the selected continuation worker body has no declared callee in this function,                  neither an emittable raw unit nor a generated execution context"
                    .to_string(),
            ));
        }
        function_local.worker_calls = worker_calls;
        // `D8n` — this generated Function's own checked-frame consumption
        // transaction, spanning the specialization body exactly. ⛔ Opened before the builder and
        // closed after it, so every branch scope inside nests within it.
        let frame_scope = CheckedFrameFunctionScope::open(compiler)?;
        // ⭐⭐ `D8o` — THE BINDING THIS PASS NEVER HAD. A specialization body
        // used to run with whatever the previously defined body left in both
        // ambient fields. The owner is exactly the planner's identity for this
        // specialization; the unit is the owner of the source body it lowers,
        // which is the continuation's own consumer.
        let ambient = AmbientBodyAuthority::bind(
            compiler,
            ContinuationEmissionOwner::Specialization(unit.id),
            unit.consumer_owner,
        );
        // `D8o` — the exact body key, supplied by the pass that knows it.
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d8o_body_key(
            compiler.defining_function_id,
            crate::cranelift_backend::lowering::D8oBodyKey::ContinuationSpecialization(unit.id),
        );
        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let envelope_pointer = builder.block_params(entry)[0];
            let frame = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                envelope_pointer,
                crate::activation_services::UNIT_CALL_FRAME_SLOTS,
            );
            // The same per-function activation-services preamble every
            // generated unit body binds, from the same envelope and services
            // record. Omitting it is what left this function with no boundary
            // arena; substituting the native-`Int` arena for the boundary one
            // is the defect the `S6`/`D6` ruling exists to remove, so the
            // record is read for both rather than one standing in for the
            // other.
            let host_dispatch_context = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                envelope_pointer,
                crate::activation_services::UNIT_CALL_FRAME_HOST_DISPATCH_CONTEXT,
            );
            let services = builder.block_params(entry)[1];
            let native_int_arena = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                services,
                crate::activation_services::SERVICES_NATIVE_INT_ARENA,
            );
            Lowering::require_nonzero(&mut builder, native_int_arena);
            let boundary_arena = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                services,
                crate::activation_services::SERVICES_BOUNDARY_ARENA,
            );
            Lowering::require_nonzero(&mut builder, boundary_arena);
            function_local.host_dispatch_context = Some(host_dispatch_context);
            function_local.native_int_arena = Some(native_int_arena);
            function_local.boundary_arena = Some(boundary_arena);
            function_local.services_pointer = Some(services);
            function_local.bind_unit_trap_frame(
                frame,
                i32::try_from(trap_offset).map_err(|_| {
                    backend_module("continuation trap slot offset exceeds range".to_string())
                })?,
            )?;
            compiler.function_local = function_local;

            // Descriptor-only loads. Each operand is read from the slot the
            // descriptor assigns it, and from nowhere else.
            let load_at = |builder: &mut FunctionBuilder<'_>, offset: u32| {
                let offset = i32::try_from(offset).map_err(|_| {
                    backend_module("continuation slot offset exceeds addressable range".to_string())
                })?;
                Ok::<_, CraneliftBackendError>(LoweringOperand::Carried(CarriedBoundaryWord {
                    word: builder.ins().load(types::I64, MemFlags::trusted(), frame, offset),
                }))
            };

            let mut ordinary = Vec::with_capacity(parameter_slots.len());
            for (_, offset) in &parameter_slots {
                ordinary.push(load_at(&mut builder, **offset)?);
            }
            let mut carried_inputs = Vec::with_capacity(capture_slots.len());
            for (_, offset) in &capture_slots {
                carried_inputs.push(load_at(&mut builder, **offset)?);
            }

            // `D5a` -- the operand suffix the retargeted worker call appends.
            //
            // These are THIS frame's own continuation inputs, in ordinal order,
            // which is exactly the capture run the generated context declares.
            // ⛔ Stashed rather than threaded through `construct_static_worker_
            // binding`: the worker binding is the raw body's contract and adding
            // a context's captures to it would make the raw contract vary with
            // its caller.
            if let Some(worker_body_origin) = retargeted_worker_body {
                compiler.function_local.generated_context_captures =
                    Some(GeneratedContextCaptures {
                        worker_body_origin,
                        operands: carried_inputs.clone(),
                    });
            }

            // The ordered capture segment for the selected worker: the
            // envelope's `WorkerCapture` roles, in capture-ordinal order,
            // taking each one's operand from its own Parameter position.
            let mut worker_captures = Vec::new();
            for (position, role) in envelope.iter().enumerate() {
                if matches!(role, ContinuationOrdinaryEnvelopeRole::WorkerCapture { .. }) {
                    worker_captures.push(ordinary[position].clone());
                }
            }
            if worker_captures.len() != unit.worker_capture_count {
                return Err(backend_module(
                    "the ordinary envelope's worker-capture segment disagrees with the selected \
                     worker's capture count"
                        .to_string(),
                ));
            }

            // `D6a` -- THE INDUCTION HYPOTHESIS'S ROUTE, and only its.
            //
            // `retargeted_worker_body` is `Some` exactly when
            // `continuation_context_for` issued a generated execution context
            // for this `(specialization, worker body)` pair AND the retarget
            // below resolved it into this function. Both halves are required,
            // which is why this reads the retarget's own outcome rather than
            // re-asking the planner: an issued context this unit did not
            // resolve is not a context this binding can name.
            //
            // ⭐ `None` is the ordinary, lawful answer -- every pre-`D5a`
            // specialization, and every unit in the governed-bracket witness.
            // The hypothesis then takes the raw route, appending nothing, and
            // the two bindings below are route-identical. That is a degenerate
            // route pair, not a collapsed one.
            //
            // ⛔ Not re-derived at the call site. Both bindings below name the
            // same body origin, so no comparison available there can tell them
            // apart -- see `StaticWorkerCallRoute`.
            let induction_route = match retargeted_worker_body {
                Some(_) => StaticWorkerCallRoute::GeneratedContext,
                None => StaticWorkerCallRoute::RawWorker,
            };
            // `D6c` — CROSS-ROUTING, the hypothesis half. It takes the raw route
            // while this unit DID resolve a context; the argument below takes
            // the context route. ⛔ Only where a context was actually resolved:
            // on a route-degenerate unit both members lawfully carry
            // `RawWorker`, so there is no crossing to make and the arm declines
            // rather than counting an application it did not perform.
            #[cfg(test)]
            let induction_route = if crate::cranelift_backend::lowering::d6c_selection_mutation()
                == crate::cranelift_backend::lowering::D6cSelectionMutation::CrossRouteTargets
                && retargeted_worker_body.is_some()
            {
                crate::cranelift_backend::lowering::record_d6c_selection_application();
                StaticWorkerCallRoute::RawWorker
            } else {
                induction_route
            };

            // The EXISTING constructor, with the projected identity and arity.
            let worker = compiler.construct_static_worker_binding(
                unit.worker_closure_origin,
                unit.worker_body_origin,
                unit.worker_declared_arity,
                unit.worker_capture_count,
                worker_captures.clone(),
                induction_route,
                // `D8i` — an induction hypothesis answers for no composed
                // source continuation. Stated explicitly: this is a positive
                // claim about the hypothesis's role, not the absence of one.
                ContinuationDischarge::DirectSpecializationCall,
            )?;

            // `D6a` -- the selected recursive constructor argument.
            //
            // ⭐ The SAME closure occurrence, body origin, declared arity and
            // ordered capture operands as the induction hypothesis above, built
            // through the same constructor and validated against the same raw
            // template contract. What the two represent still differs: the
            // argument is the closure the source scope binds, while the
            // hypothesis is that closure as this specialization eliminates it.
            //
            // ⛔ The ROUTE is `RawWorker` unconditionally here, and that is not
            // the same as saying it differs from the hypothesis's. When
            // `induction_route` above resolved to `RawWorker` -- no context
            // issued -- the two bindings are route-identical, and they are
            // still two bindings for two positions of the run. The route is
            // what will separate them at the call edge in `D6b` *where a
            // context exists*; it is not what makes them two.
            //
            // ⛔ Nothing new crosses the ABI. This adds no slot, carrier, tag,
            // descriptor or source occurrence -- it is a second compiler-only
            // binding over operands this frame has already loaded.
            // `D6c` — the three binding-construction mutations, under test only.
            // Each moves ONE argument handed to the existing constructor; the
            // constructor itself, the hypothesis above and the run below are all
            // untouched, so the guard that refuses is the one that owns the
            // moved input.
            #[cfg(test)]
            let (argument_body_origin, argument_captures, argument_route) = {
                use crate::cranelift_backend::lowering::D6cSelectionMutation as Mutation;
                match crate::cranelift_backend::lowering::d6c_selection_mutation() {
                    // A body this unit did not select. ⛔ The substituted value
                    // is a REAL planner-issued origin -- this continuation's own
                    // frame occurrence -- rather than an arithmetic neighbour.
                    // A fabricated id could be refused merely for being unknown;
                    // a real origin naming the wrong role is the case the guard
                    // actually has to catch. The control asserts it differs from
                    // the selected body.
                    Mutation::WrongClosureBody => {
                        crate::cranelift_backend::lowering::record_d6c_selection_application();
                        (
                            unit.continuation_origin,
                            worker_captures.clone(),
                            StaticWorkerCallRoute::RawWorker,
                        )
                    }
                    // A capture run that is not the envelope's worker-capture
                    // segment: drop an operand where there is one, otherwise add
                    // one the envelope holds.
                    //
                    // ⛔ The counter fires ONLY if the vector actually changed.
                    // A unit with no captures and no ordinary operand to borrow
                    // leaves this arm the IDENTITY, and counting an application
                    // there would report a perturbation that never happened --
                    // which is precisely how a control comes to prove the
                    // opposite of what it claims.
                    Mutation::WrongCaptureRun => {
                        let mut perturbed = worker_captures.clone();
                        if perturbed.pop().is_none() {
                            perturbed.extend(ordinary.first().cloned());
                        }
                        if perturbed.len() != worker_captures.len() {
                            crate::cranelift_backend::lowering::record_d6c_selection_application();
                        }
                        (
                            unit.worker_body_origin,
                            perturbed,
                            StaticWorkerCallRoute::RawWorker,
                        )
                    }
                    // The argument takes the context route. Paired with the
                    // hypothesis taking the raw route above, this is the
                    // cross-routing the two members must never permit.
                    //
                    // ⛔ Only where a context was actually resolved. On a
                    // route-degenerate unit both members lawfully carry
                    // `RawWorker`, so there is no crossing to perform; applying
                    // it there would move a route no law distinguishes and count
                    // an application for a perturbation with no content.
                    Mutation::CrossRouteTargets if retargeted_worker_body.is_some() => {
                        crate::cranelift_backend::lowering::record_d6c_selection_application();
                        (
                            unit.worker_body_origin,
                            worker_captures.clone(),
                            StaticWorkerCallRoute::GeneratedContext,
                        )
                    }
                    _ => (
                        unit.worker_body_origin,
                        worker_captures.clone(),
                        StaticWorkerCallRoute::RawWorker,
                    ),
                }
            };
            #[cfg(not(test))]
            let (argument_body_origin, argument_captures, argument_route) = (
                unit.worker_body_origin,
                worker_captures,
                StaticWorkerCallRoute::RawWorker,
            );
            let recursive_argument = compiler.construct_static_worker_binding(
                unit.worker_closure_origin,
                argument_body_origin,
                unit.worker_declared_arity,
                unit.worker_capture_count,
                argument_captures,
                argument_route,
                // `D8i` — the SPECIALIZATION's selected recursive argument.
                // ⛔ Direct, and the contrast with `D8d`'s composed argument is
                // the point: the same source closure at the same position
                // carries an authority on the composed path and none here,
                // because only the composed consumption stands in for a causal
                // call the producer never made.
                ContinuationDischarge::DirectSpecializationCall,
            )?;

            // Exact body recovery: the selected case of the computational
            // frame this continuation belongs to, by its own alternative.
            let frame_occurrence =
                compiler.retained_body_occurrence(unit.continuation_origin)?;
            let RuntimeExpr::ComputationalMatch { cases, .. } = frame_occurrence.expr else {
                return Err(backend_module(
                    "a continuation origin does not resolve to a computational frame".to_string(),
                ));
            };
            let alternative = unit.producer_alternative as usize;
            let case = cases.get(alternative).ok_or_else(|| {
                backend_module(
                    "the projected producer alternative is outside the frame's case run"
                        .to_string(),
                )
            })?;
            let body = compiler.case_body_occurrence(
                frame_occurrence.static_origin,
                alternative,
                &case.body,
            )?;
            // The semantic case environment, through the sole binding
            // authority, in the order `continuation_case_binder_run` states:
            // the IH prefix, then ALL the constructor arguments in source
            // order -- the selected recursive one included, as `D6a`'s
            // compiler-only member -- then this frame's continuation inputs.
            //
            // ⛔ This site chooses nothing. It maps a plan onto operands; the
            // order is the plan's, and the plan is a pure function of the
            // planner's own coordinates.
            let plan = continuation_case_binder_run(
                case.argument_binders,
                &case.recursive_positions,
                unit.recursive_position,
                envelope,
                carried_inputs.len(),
            )?;
            let mut env: Vec<LoweringEnvironmentBinding> = Vec::with_capacity(plan.len());
            for source in &plan {
                let binding = match *source {
                    ContinuationCaseBinderSource::InductionHypothesis => {
                        LoweringEnvironmentBinding::StaticWorker(worker.clone())
                    }
                    ContinuationCaseBinderSource::SelectedRecursiveArgument {
                        source_position,
                    } => {
                        // The plan only ever names the ruled position here;
                        // segment 1 hard-stops on any other. Re-checking it is
                        // what keeps that a fact this site verifies rather than
                        // one it inherits.
                        if source_position != unit.recursive_position {
                            return Err(backend_module(format!(
                                "the binder run names a selected recursive argument at source \
                                 position {source_position}, but this specialization projects a \
                                 worker for position {}",
                                unit.recursive_position
                            )));
                        }
                        LoweringEnvironmentBinding::StaticWorker(recursive_argument.clone())
                    }
                    ContinuationCaseBinderSource::Ordinary(index) => {
                        let operand = ordinary.get(index).ok_or_else(|| {
                            backend_module(
                                "the binder run names an ordinary-envelope index this frame loaded \
                                 no operand for"
                                    .to_string(),
                            )
                        })?;
                        LoweringEnvironmentBinding::Value(operand.clone())
                    }
                    ContinuationCaseBinderSource::ContinuationInput(ordinal) => {
                        let operand = carried_inputs.get(ordinal).ok_or_else(|| {
                            backend_module(
                                "the binder run names a continuation input ordinal this frame \
                                 loaded no operand for"
                                    .to_string(),
                            )
                        })?;
                        LoweringEnvironmentBinding::Value(operand.clone())
                    }
                };
                env.push(binding);
            }

            // `D6b` — the same instant, structured. The trace below renders the
            // ROUTE of each static-worker member; this record carries the body
            // origin beside it, which is what lets a control ask whether the
            // mixed pair is over ONE body rather than merely mixed.
            #[cfg(test)]
            crate::cranelift_backend::lowering::record_d6b_specialization_body(
                crate::cranelift_backend::lowering::D6bSpecializationBody {
                    unit: unit.id,
                    worker_body_origin: unit.worker_body_origin,
                    retargeted: retargeted_worker_body,
                    worker_call_targets: compiler
                        .function_local
                        .worker_calls
                        .keys()
                        .copied()
                        .collect(),
                    raw_worker_call_targets: compiler
                        .function_local
                        .raw_worker_calls
                        .keys()
                        .copied()
                        .collect(),
                    members: env
                        .iter()
                        .enumerate()
                        .filter_map(|(position, binding)| match binding {
                            LoweringEnvironmentBinding::StaticWorker(worker) => {
                                Some((position, worker.route, worker.body_origin))
                            }
                            LoweringEnvironmentBinding::Value(_) => None,
                        })
                        .collect(),
                },
            );
            #[cfg(test)]
            d5a_trace(format!(
                "  SPEC-BODY {:?} alt={} binders={} ordinary={} envelope={:?} env=[{}]",
                unit.id,
                unit.producer_alternative,
                case.argument_binders,
                ordinary.len(),
                envelope,
                env.iter()
                    // `D6a` -- the ROUTE is printed, not just the arm. Both
                    // static-worker members name the same closure, body and
                    // arity, so an arm-only rendering shows two identical
                    // entries and a change collapsing the two routes would be
                    // invisible in this log.
                    //
                    // ⛔ The converse does not hold, and a reader of this log
                    // must not assume it: two entries rendering the SAME route
                    // is the lawful route-degenerate case (no context issued),
                    // not evidence that one binding was reused for both
                    // members. Only a witness whose planner issues a context
                    // renders a mixed pair, and only there does this log
                    // discriminate the routes at all.
                    .map(|binding| match binding {
                        LoweringEnvironmentBinding::StaticWorker(worker) => match worker.route {
                            StaticWorkerCallRoute::RawWorker => "StaticWorker(RawWorker)",
                            StaticWorkerCallRoute::GeneratedContext =>
                                "StaticWorker(GeneratedContext)",
                        },
                        LoweringEnvironmentBinding::Value(LoweringOperand::Carried(_)) =>
                            "Carried",
                        LoweringEnvironmentBinding::Value(LoweringOperand::Specialized(_)) =>
                            "Specialized",
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            let lowered = compiler.lower_expr(&mut builder, body, &env)?;

            // The Result slot is WRITTEN here and never read.
            let word = match lowered {
                LoweringOperand::Carried(carried) => carried.word,
                LoweringOperand::Specialized(value) => {
                    compiler.emit_result(&mut builder, value)?.0
                }
            };
            let result_offset = i32::try_from(result_offset).map_err(|_| {
                backend_module("continuation result slot offset exceeds range".to_string())
            })?;
            builder
                .ins()
                .store(MemFlags::trusted(), word, frame, result_offset);
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().return_(&[zero]);
            builder.seal_all_blocks();
            builder.finalize();
        }
        ambient.release(compiler);
        frame_scope.close(compiler)?;
        // Verify, then define THIS function -- a fresh context here would
        // define an empty body and silently discard everything emitted above.
        verify_cranelift_function(&func, module.isa())?;
        compiler.commit_aggregate_events()?;
        let mut ctx = module.make_context();
        std::mem::swap(&mut ctx.func, &mut func);
        module
            .define_function(id, &mut ctx)
            .map_err(|error| backend_module(error.to_string()))?;
        defined += 1;
    }
    Ok(defined)
}

/// **`RT-DECL-CLOSURE-PORT` `D5a` — define each generated producer execution
/// context.**
///
/// The body lowered here is the **raw worker body**, unchanged. What differs
/// from that body's own ordinary unit is only the environment it runs in: the
/// context's frame carries the raw parameter run *and* the enclosing
/// specialization's continuation inputs, so a nested producer inside that body
/// can reach operands raw `fn2` provably never receives.
///
/// ⛔ **Two different questions about the raw unit, and they are decided by
/// different authorities. Do not answer one with the other.**
///
/// 1. **Descriptor, provenance and source-binding authority are RETAINED.**
///    The raw `ClosureBody` keeps its own descriptor, its own ABI, and its
///    status as the source binding for this body — the template lowered here
///    *is* that body, unchanged. ⛔ Nothing about this retarget mutates,
///    unions, or suffixes the raw descriptor.
/// 2. **Executable `Function` declaration/definition membership is NOT
///    retained by default.** It is decided from the **post-retarget final
///    graph**, by `StaticTransitionPlan::template_only_worker_bodies`: a raw
///    worker every selecting specialization has retargeted — and whose carried
///    invocation also binds a generated context — is **template-only**, and is
///    absent from the emitted-`Function` population. A raw worker with any
///    remaining final raw call stays executable.
///
/// ⚠ **This paragraph previously said the raw unit "is still emitted and is
/// not retired", keeping its body and "simply losing one caller".** That was
/// the pre-`D5a` reading and it is superseded: it answers question 2 with
/// question 1's answer, so it is true of the descriptor and false of the
/// emitted population whenever the retarget is total. The surviving half of
/// the old claim is the useful one — *"a new owner for a body does not retire
/// the old owner's unit"* remains correct about **authority**, and says
/// nothing about **membership**.
///
/// ⛔ The emission owner bound here is `Specialization(enclosing)`, and
/// `defining_unit` stays the **raw** owner. Deriving one from the other is the
/// conflation `evt_609am4v7cdt5b` ruled against, and it is precisely because
/// this function lowers someone else's body that the two must be supplied
/// independently.
pub(super) fn define_continuation_context_bodies<M: Module>(
    module: &mut M,
    compiler: &mut Lowering<'_>,
    helpers: ArtifactHelpers<'_>,
    bundle: &UnitBundle,
    call_edges: &CallEdgeTargets,
) -> Result<usize, CraneliftBackendError> {
    struct OwnedContext {
        id: ContinuationContextId,
        enclosing: ContinuationSpecializationId,
        worker_body_origin: StaticOriginId,
        raw_owner: PredeclaredFunctionId,
        slots: Vec<AbiSlot>,
        offsets: Vec<u32>,
        header_parameters: u32,
        header_captures: u32,
    }
    // Own every projected fact before the loop: the projection borrows the plan
    // and definition below needs the compiler mutably.
    let contexts = compiler
        .static_transition_plan
        .continuation_contexts()?
        .into_iter()
        .map(|context| {
            let (offsets, _frame_bytes) = context.slot_offsets()?;
            Ok(OwnedContext {
                id: context.id(),
                enclosing: context.enclosing_specialization(),
                worker_body_origin: context.worker_body_origin(),
                raw_owner: context.raw_owner(),
                slots: context.slots().to_vec(),
                offsets,
                header_parameters: context.header().parameters,
                header_captures: context.header().captures,
            })
        })
        .collect::<Result<Vec<_>, CraneliftBackendError>>()?;

    let worker_targets = resolve_worker_targets(&compiler.static_transition_plan, bundle)?;
    let mut defined = 0usize;
    for context in contexts {
        let id = bundle.context(context.id).ok_or_else(|| {
            backend_module(
                "a planned generated context was never forward-declared".to_string(),
            )
        })?;
        let slots = context.slots.as_slice();
        let offsets = context.offsets.as_slice();
        if slots.len() != offsets.len() {
            return Err(backend_module(
                "a generated context slot run disagrees with its own offset walk".to_string(),
            ));
        }
        let parameter_count = slots
            .iter()
            .filter(|slot| slot.kind == AbiSlotKind::Parameter)
            .count();
        let capture_count = slots
            .iter()
            .filter(|slot| slot.kind == AbiSlotKind::Capture)
            .count();
        // The header is the declared contract and the slot run is what the body
        // will actually walk; a disagreement here means the environment this
        // body binds is not the one its caller passes operands for.
        if u32::try_from(parameter_count).ok() != Some(context.header_parameters)
            || u32::try_from(capture_count).ok() != Some(context.header_captures)
        {
            return Err(backend_module(
                "a generated context's slot run disagrees with its declared frame header"
                    .to_string(),
            ));
        }
        let result_offset = slots
            .iter()
            .zip(offsets)
            .find(|(slot, _)| slot.kind == AbiSlotKind::Result)
            .map(|(_, offset)| *offset)
            .ok_or_else(|| {
                backend_module("generated context frame declares no result slot".to_string())
            })?;
        let trap_offset = slots
            .iter()
            .zip(offsets)
            .find(|(slot, _)| slot.kind == AbiSlotKind::Trap)
            .map(|(_, offset)| *offset)
            .ok_or_else(|| {
                backend_module("generated context frame declares no trap slot".to_string())
            })?;

        let sig = unit_signature(module);
        let mut func = Function::with_name_signature(UserFuncName::user(3, id.as_u32()), sig);
        let mut function_local = helpers.declare_in_func(module, &mut func, None);
        // The raw body's OWN call edges, declared into this function. They are
        // the raw owner's edges because the body is the raw owner's body; what
        // this context changes is the environment, never which callees the
        // source names.
        let declared_calls = call_edges.declare_in_func(context.raw_owner, module, &mut func)?;
        function_local.unit_calls = declared_calls.static_bodies;
        function_local.declaration_calls = declared_calls.declarations;
        function_local.worker_calls = worker_targets.declare_in_func(module, &mut func);
        // `D6b` -- no retarget happens in a generated context body, so the two
        // tables agree here. Populated anyway rather than left empty: the raw
        // route must resolve from ITS OWN table in every function, or the
        // resolution silently depends on which function it runs in.
        function_local.raw_worker_calls = function_local.worker_calls.clone();
        function_local.worker_templates = worker_targets.templates().clone();
        function_local.context_calls = declare_context_calls_in_func(
            module,
            &mut func,
            &compiler.static_transition_plan,
            bundle,
        )?;
        // `D5a`: this context's own causal call refs, selected by the EMISSION
        // owner. ⛔ Not by `raw_owner` -- that is the filter that would hand this
        // function the raw unit's tokens and leave its own undeclared.
        let emission_owner = ContinuationEmissionOwner::Specialization(context.enclosing);
        function_local.continuation_calls = match compiler.continuation_claims.as_ref() {
            Some(ledger) => ledger.declare_owned_in_func(
                emission_owner,
                module,
                &mut func,
                &compiler.static_transition_plan,
            )?,
            None => BTreeMap::new(),
        };
        if let Some(ledger) = compiler.continuation_claims.as_mut() {
            ledger.record_declared(function_local.continuation_calls.keys().cloned())?;
        }
        // Contract 3, on this context: the projection is taken before the
        // function is defined and keyed on the emission owner this pass is about
        // to bind.
        let result_edges = compiler
            .static_transition_plan
            .continuation_result_edges_owned_by(emission_owner)?;

        // `D8n` — this generated Function's own checked-frame consumption
        // transaction, spanning the generated-context body exactly. ⛔ Opened before the builder and
        // closed after it, so every branch scope inside nests within it.
        let frame_scope = CheckedFrameFunctionScope::open(compiler)?;
        let mut func_ctx = FunctionBuilderContext::new();
        {
            let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
            let entry = builder.create_block();
            builder.append_block_params_for_function_params(entry);
            builder.switch_to_block(entry);
            let envelope_pointer = builder.block_params(entry)[0];
            let frame = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                envelope_pointer,
                crate::activation_services::UNIT_CALL_FRAME_SLOTS,
            );
            let host_dispatch_context = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                envelope_pointer,
                crate::activation_services::UNIT_CALL_FRAME_HOST_DISPATCH_CONTEXT,
            );
            let services = builder.block_params(entry)[1];
            let native_int_arena = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                services,
                crate::activation_services::SERVICES_NATIVE_INT_ARENA,
            );
            Lowering::require_nonzero(&mut builder, native_int_arena);
            let boundary_arena = builder.ins().load(
                module.target_config().pointer_type(),
                MemFlags::trusted(),
                services,
                crate::activation_services::SERVICES_BOUNDARY_ARENA,
            );
            Lowering::require_nonzero(&mut builder, boundary_arena);
            function_local.host_dispatch_context = Some(host_dispatch_context);
            function_local.native_int_arena = Some(native_int_arena);
            function_local.boundary_arena = Some(boundary_arena);
            function_local.services_pointer = Some(services);
            function_local.bind_unit_trap_frame(
                frame,
                i32::try_from(trap_offset).map_err(|_| {
                    backend_module("generated context trap slot offset exceeds range".to_string())
                })?,
            )?;
            compiler.function_local = function_local;
            compiler.open_aggregate_events(id)?;
            // `D8o` — same binding, same unchanged domain: the emission owner is
            // the enclosing specialization and `defining_unit` stays the RAW
            // owner, which is the distinction the two fields exist to keep.
            // ⛔ After `open_aggregate_events`, so the observation this binding
            // writes is labelled with the Function it belongs to.
            let ambient = AmbientBodyAuthority::bind(compiler, emission_owner, context.raw_owner);
            // `D8o` — a generated CONTEXT body, whose owner is a Specialization
            // and whose kind is not.
            #[cfg(test)]
            crate::cranelift_backend::lowering::record_d8o_body_key(
                compiler.defining_function_id,
                crate::cranelift_backend::lowering::D8oBodyKey::GeneratedContext(context.id),
            );

            // The environment: Parameter slots then Capture slots, in slot
            // order. ⭐ This is the SAME walk `define_unit_body` uses, which is
            // why the raw body's binder positions resolve identically here --
            // the parameter prefix is byte-for-byte the run its own unit binds,
            // and the continuation inputs sit strictly after it.
            let mut env = Vec::new();
            for (slot, offset) in slots.iter().zip(offsets) {
                if !matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture) {
                    continue;
                }
                let offset = i32::try_from(*offset).map_err(|_| {
                    backend_module(
                        "generated context slot offset exceeds addressable range".to_string(),
                    )
                })?;
                let word = builder
                    .ins()
                    .load(types::I64, MemFlags::trusted(), frame, offset);
                env.push(LoweringEnvironmentBinding::Value(LoweringOperand::Carried(
                    CarriedBoundaryWord { word },
                )));
            }

            let body = compiler.retained_body_occurrence(context.worker_body_origin)?;
            let lowered = compiler.lower_expr(&mut builder, body, &env)?;
            // The detached-result seat, live in this context. Every operand its
            // capture projection names is reachable from `env` above.
            let lowered = compiler.eliminate_detached_producer_continuation(
                &mut builder,
                &result_edges,
                lowered,
                &env,
            )?;
            let word = match lowered {
                LoweringOperand::Carried(word) => Some(word.word),
                LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                    compiler.emit_current_trap(&mut builder, &trap)?;
                    None
                }
                LoweringOperand::Specialized(value) => Some(
                    compiler
                        .transfer_unit_result_into_carrier(
                            &mut builder,
                            context.worker_body_origin,
                            &value,
                        )?
                        .word,
                ),
            };
            if let Some(word) = word {
                builder.ins().store(
                    MemFlags::trusted(),
                    word,
                    frame,
                    i32::try_from(result_offset).map_err(|_| {
                        backend_module(
                            "generated context result slot offset exceeds range".to_string(),
                        )
                    })?,
                );
            }
            let status = builder.ins().iconst(types::I64, 0);
            builder.ins().return_(&[status]);
            builder.seal_all_blocks();
            builder.finalize();
            ambient.release(compiler);
        }
        frame_scope.close(compiler)?;
        // The same emission-seam gate every other generated function passes: the
        // callee of each recorded causal emission is decoded back out of THIS
        // finished CLIF and compared with the planner-issued target.
        compiler.verify_emitted_continuation_calls(&func, bundle)?;
        // `D8j` — the composed relation's own gate, beside the direct one and
        // never inside it: the two answer different questions about different
        // callees.
        compiler.verify_recorded_composed_discharges(&func, bundle)?;
        #[cfg(test)]
        crate::cranelift_backend::lowering::record_d8j_discharged(
            compiler.function_local.composed_discharges.keys().cloned(),
        );
        // `D8k` -- the composed half of the partition, accumulated from the
        // VERIFIED relation and never from the direct instruction map.
        if let Some(ledger) = compiler.continuation_claims.as_mut() {
            ledger.record_composed(
                compiler.function_local.composed_discharges.keys().cloned(),
                emission_owner,
            )?;
        }
        if let Some(ledger) = compiler.continuation_claims.as_mut() {
            ledger.record_emitted(
                compiler.function_local.continuation_emissions.keys().cloned(),
            )?;
        }
        verify_cranelift_function(&func, module.isa())?;
        compiler.commit_aggregate_events()?;
        let mut ctx = module.make_context();
        std::mem::swap(&mut ctx.func, &mut func);
        module
            .define_function(id, &mut ctx)
            .map_err(|error| backend_module(error.to_string()))?;
        defined += 1;
    }
    Ok(defined)
}

pub(super) fn define_root_adapter<M: Module>(
    module: &mut M,
    compiler: &mut Lowering<'_>,
    helpers: ArtifactHelpers<'_>,
    bundle: &UnitBundle,
    adapter_id: FuncId,
    process_mode: bool,
    project_public_scalar_root: bool,
) -> Result<(), CraneliftBackendError> {
    compiler.open_aggregate_events(adapter_id)?;
    let root = compiler.static_transition_plan.root_emittable_unit()?;
    let root_id = bundle.function(root.function()).ok_or_else(|| {
        backend_module("the recorded root unit was never forward-declared".to_string())
    })?;
    let (offsets, frame_bytes) = root.slot_offsets()?;
    if frame_bytes != root.header().frame_bytes {
        return Err(backend_module(
            "root adapter target frame size disagrees with its slot run".to_string(),
        ));
    }
    if process_mode {
        for role in [
            AbiProcessParameter::ProcessInput,
            AbiProcessParameter::Capability,
        ] {
            compiler
                .static_transition_plan
                .process_parameter_slot(role)?
                .ok_or_else(|| {
                    backend_module("process root has no declared role-keyed ingress slot".to_string())
                })?;
        }
    }

    let sig = unit_signature(module);
    let mut func =
        Function::with_name_signature(UserFuncName::user(0, adapter_id.as_u32()), sig);
    let mut function_local = helpers.declare_in_func(
        module,
        &mut func,
        Some(TrapExitAuthority::Root {
            process_sentinel: process_mode,
            source_authorized: false,
        }),
    );
    let root_origin = root.origin();
    function_local.unit_calls.insert(
        root_origin,
        DeclaredUnitCall {
            function: module.declare_func_in_func(root_id, &mut func),
            origin: root_origin,
            // The root adapter's own entry: source body and scheduling entry
            // are the same occurrence.
            call_site_origin: root_origin,
            header: root.header(),
            slots: root.slots().to_vec(),
            offsets,
        },
    );

    let mut func_ctx = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let ingress = builder.block_params(entry)[0];
        let services = builder.block_params(entry)[1];
        let pointer_type = module.target_config().pointer_type();
        let native_int_arena = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            services,
            crate::activation_services::SERVICES_NATIVE_INT_ARENA,
        );
        Lowering::require_nonzero(&mut builder, native_int_arena);
        let boundary_arena = builder.ins().load(
            pointer_type,
            MemFlags::trusted(),
            services,
            crate::activation_services::SERVICES_BOUNDARY_ARENA,
        );
        Lowering::require_nonzero(&mut builder, boundary_arena);
        function_local.services_pointer = Some(services);
        function_local.native_int_arena = Some(native_int_arena);
        function_local.boundary_arena = Some(boundary_arena);

        let mut inputs = Vec::new();
        if process_mode {
            let process_input = builder.ins().load(
                pointer_type,
                MemFlags::trusted(),
                ingress,
                crate::boundary_activation::ROOT_INGRESS_PROCESS_INPUT,
            );
            Lowering::require_nonzero(&mut builder, process_input);
            let host_dispatch_context = builder.ins().load(
                pointer_type,
                MemFlags::trusted(),
                ingress,
                crate::boundary_activation::ROOT_INGRESS_HOST_DISPATCH_CONTEXT,
            );
            Lowering::require_nonzero(&mut builder, host_dispatch_context);
            let capability = builder.ins().load(
                types::I64,
                MemFlags::trusted(),
                ingress,
                crate::boundary_activation::ROOT_INGRESS_CAPABILITY,
            );
            function_local.host_dispatch_context = Some(host_dispatch_context);
            inputs.push(LoweringOperand::Specialized(
                Lowered::BorrowedNativeValue {
                    pointer: process_input,
                },
            ));
            inputs.push(LoweringOperand::Specialized(Lowered::CapabilityToken {
                value: capability,
            }));
            #[cfg(test)]
            PROCESS_SLOT_MUTATION.with(|cell| match cell.get() {
                ProcessSlotMutation::Exact
                | ProcessSlotMutation::AttemptFixedContextOffsets
                | ProcessSlotMutation::ReintroduceLaunchIngress => {}
                ProcessSlotMutation::DeleteProcessInput => {
                    inputs.remove(0);
                }
                ProcessSlotMutation::DeleteCapability => {
                    inputs.pop();
                }
            });
        } else {
            function_local.host_dispatch_context =
                Some(builder.ins().iconst(pointer_type, 0));
        }

        compiler.function_local = function_local;
        let result = compiler.call_declared_unit(
            &mut builder,
            root_origin,
            &inputs,
            #[cfg(test)]
            Some(ingress),
        )?;
        let LoweringOperand::Carried(result) = result else {
            return Err(backend_module(
                "the internal root call did not return its result word".to_string(),
            ));
        };
        let public_result = if project_public_scalar_root {
            compiler.emit_public_carrier_scalar(&mut builder, result)?
        } else {
            result.word
        };
        builder.ins().return_(&[public_result]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&func, module.isa())?;
    compiler.commit_aggregate_events()?;
    #[cfg(test)]
    scale_b_record_functionized_root_adapter(&func);
    let mut ctx = module.make_context();
    std::mem::swap(&mut ctx.func, &mut func);
    module
        .define_function(adapter_id, &mut ctx)
        .map_err(|err| backend_module(err.to_string()))
}

/// **`D2` — define every declared unit against its declared activation frame.**
///
/// ⛔ **Every dynamic value crosses into a unit through the declared
/// `AbiFrameHeader` + `AbiSlot` layout, never through capture-by-construction.**
/// The frame pointer is the unit's sole parameter; each slot is addressed at the
/// offset `B2R`'s own walk assigns it.
///
/// **MEASURED:** each emitted body addresses its slots at the offsets
/// `abi::slot_offsets` assigns, and reads its result from the slot `B2R` marks
/// `AbiSlotKind::Result`.
/// **CLAIMED:** the emitted code obeys the declared frame layout.
/// **THE GAP:** ⚠ this establishes *layout* agreement only. It says nothing
/// about the **ownership modes** (`AC-12`) or about whether every transfer into
/// a slot is **representable** (`AC-11`) — those are separate obligations with
/// their own controls, and ⛔ a body that addresses the right offset while
/// violating an ownership mode satisfies everything asserted here.
pub(super) struct RootUnitResult {
    pub(super) decoder: Option<ResultDecoder>,
    pub(super) trap: Option<RuntimeTrap>,
}

/// **`RT-CONTSPEC-ACTIVATE` `D3` — the affine claim ledger over the exact
/// planned continuation-call tokens.**
///
/// Each projected causal identity is claimed **exactly once**, by the exact
/// producer unit the token itself names. This is affine on the *causal token*,
/// which is a different object from `RT-WORKER-BIND`'s worker binding -- that
/// one is deliberately NOT affine, and nothing here changes it.
///
/// ⛔ There is no `active_emission_owner`, no lowering-minted arm token, and no
/// second owner authority: the owner compared against is the token's own
/// immutable `producer_owner`, and the unit compared to it is the one
/// currently being defined.
///
/// ⚠ An affine rejection from here is **not self-explaining**. The identity is
/// four-field -- producer construct, alternative, call-site sequence, and
/// `recursive_position`. A key that lost `recursive_position` collides two
/// distinct tokens at one source position and this ledger will report a
/// double-consumption of the *right* token while the real defect is the key's
/// arity. Check the arity before believing the report.
pub(super) struct ContinuationClaimLedger {
    /// The RESOLVED target for each planned causal identity. Previously this
    /// kept only the keys and threw the `FuncId` away through `into_keys`,
    /// which is why no continuation target could ever be called: the join D1
    /// performed was discarded at the moment it became useful.
    resolved: BTreeMap<ContinuationCallIdentity, FuncId>,
    /// `None` until claimed; then the exact unit that claimed it, so owner
    /// agreement is a recorded fact rather than an inference.
    claims: BTreeMap<ContinuationCallIdentity, Option<ContinuationEmissionOwner>>,
    /// **`4b`** -- the PLANNED set, read straight off the plan's own causal call
    /// projection at open time and never derived from [`Self::resolved`].
    planned: BTreeSet<ContinuationCallIdentity>,
    /// **`4b`** -- every identity some generated function minted a `FuncRef`
    /// for, accumulated across all of them.
    declared: BTreeSet<ContinuationCallIdentity>,
    /// **`4b`** -- every identity a direct call was actually emitted for,
    /// accumulated across all generated functions after each one's CLIF has been
    /// checked.
    emitted: BTreeSet<ContinuationCallIdentity>,
    /// **`RT-CONTSRC-PRODUCER-LOCAL` `D8k`** -- every identity discharged by a
    /// VERIFIED composed source-continuation consumption, accumulated across
    /// every generated function after each one's CLIF has been checked.
    ///
    /// ⛔ Fed from `function_local.composed_discharges` and from nothing else.
    /// The direct instruction map is not a source of composed claims: its gate
    /// requires the recorded instruction to decode to `identity.target()`, and
    /// a composed instruction targets the raw worker, so an identity appearing
    /// in both would mean one of the two gates had been loosened.
    composed: BTreeSet<ContinuationCallIdentity>,
}

impl ContinuationClaimLedger {
    pub(super) fn open(
        plan: &StaticTransitionPlan<'_>,
        bundle: &UnitBundle,
    ) -> Result<Self, CraneliftBackendError> {
        let resolved = resolve_continuation_targets(plan, bundle)?;
        let claims = resolved.keys().cloned().map(|identity| (identity, None)).collect();
        // The PLANNED set, taken from the plan's causal call projection rather
        // than from `resolved`. ⚠ Honest note: `resolve_continuation_targets`
        // walks the same projection, so planned == resolved is structural today
        // and would only separate if resolution ever dropped or added a key. It
        // is recorded because `close()` asserts the four sets equal and a set
        // that is *implied* by another is not the same evidence as one that was
        // read independently -- the load-bearing pairs are declared and emitted.
        let planned = plan
            .continuation_calls()?
            .iter()
            .map(|call| {
                plan.continuation_call_binding_for(
                    call.producer_construct_origin(),
                    call.continuation_origin(),
                    call.producer_alternative(),
                    call.recursive_position(),
                )?
                .ok_or_else(|| {
                    backend_module(
                        "a projected causal call has no binding under its own four-field selector"
                            .to_string(),
                    )
                })
            })
            .collect::<Result<BTreeSet<_>, CraneliftBackendError>>()?;
        Ok(Self {
            resolved,
            claims,
            planned,
            declared: BTreeSet::new(),
            emitted: BTreeSet::new(),
            composed: BTreeSet::new(),
        })
    }

    /// **`4b`** -- record the causal tokens one generated function minted call
    /// refs for.
    pub(super) fn record_declared(
        &mut self,
        declared: impl IntoIterator<Item = ContinuationCallIdentity>,
    ) -> Result<(), CraneliftBackendError> {
        for identity in declared {
            if !self.declared.insert(identity) {
                return Err(backend_module(
                    "a causal token was declared into more than one generated function".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// **`4b`** -- record the causal tokens one generated function actually
    /// emitted a verified direct call for.
    ///
    /// ⛔ Called only *after* that function's CLIF has been checked, so a token
    /// reaches this set only once its emitted callee has been decoded from the
    /// instruction stream and matched against the planner-issued target.
    pub(super) fn record_emitted(
        &mut self,
        emitted: impl IntoIterator<Item = ContinuationCallIdentity>,
    ) -> Result<(), CraneliftBackendError> {
        for identity in emitted {
            if !self.emitted.insert(identity) {
                return Err(backend_module(
                    "a causal token emitted a direct call from more than one generated function"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    /// **`D8k`** -- record the causal tokens one generated function discharged
    /// through a VERIFIED composed source-continuation consumption, and claim
    /// them.
    ///
    /// ⛔ Called only after `verify_recorded_composed_discharges` has promoted
    /// them, so an identity reaches this set only once its recorded raw-worker
    /// call was found in the finished CLIF, its decoded callee matched the
    /// `D8b`/`D8d` target, its operand run matched that target's declared run,
    /// and its result was shown to return downstream into the unchanged
    /// continuation.
    ///
    /// ⭐⭐ **The claim is made HERE and not at the seat, and that is what makes
    /// the partition disjoint.** A composed consumption claims the same
    /// `claims` slot a direct emission would, so an identity claimed both ways
    /// is rejected as a double claim rather than silently satisfying both
    /// halves of the union.
    pub(super) fn record_composed(
        &mut self,
        discharged: impl IntoIterator<Item = ContinuationCallIdentity>,
        defining: ContinuationEmissionOwner,
    ) -> Result<(), CraneliftBackendError> {
        for identity in discharged {
            if identity.emission_owner() != defining {
                return Err(backend_module(
                    "a composed source continuation was discharged by a function that is not its \
                     emission owner"
                        .to_string(),
                ));
            }
            let consumed = self.claims.get_mut(&identity).ok_or_else(|| {
                backend_module(
                    "a composed source continuation discharged a causal token this ledger never \
                     planned"
                        .to_string(),
                )
            })?;
            if let Some(previous) = consumed {
                return Err(backend_module(format!(
                    "a causal token was claimed twice, first by {previous:?} and then by a \
                     composed source continuation; one causal obligation is discharged by one \
                     form, never by both"
                )));
            }
            *consumed = Some(defining);
            if !self.composed.insert(identity) {
                return Err(backend_module(
                    "a causal token was discharged by a composed source continuation in more \
                     than one generated function"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Declare THIS owning Function's own `FuncRef` for every causal token it
    /// owns, keyed by the complete four-field identity.
    ///
    /// Each `FuncRef` is minted into the `Function` passed here and belongs to
    /// it alone; ⛔ none is ever passed across functions. Declaring is not
    /// claiming -- a declared target is callable, and the affine claim happens
    /// later, at the exact producer occurrence.
    pub(super) fn declare_owned_in_func<M: Module>(
        &self,
        defining: ContinuationEmissionOwner,
        module: &mut M,
        func: &mut Function,
        plan: &StaticTransitionPlan<'_>,
    ) -> Result<BTreeMap<ContinuationCallIdentity, DeclaredUnitCall>, CraneliftBackendError> {
        // The declared target is the FULL contract, not a bare `FuncRef`:
        // `DeclaredUnitCall` already carries the function, the target origin,
        // and the projected header/slots/offsets, which is exactly what
        // `call_declared_unit_target` consumes. Reusing it is what keeps this
        // on the existing unit-call ABI instead of inventing a second one.
        let units = plan.continuation_units()?;
        self.resolved
            .iter()
            .filter(|(identity, _)| identity.emission_owner() == defining)
            .map(|(identity, target)| {
                let unit = units
                    .iter()
                    .find(|unit| unit.id() == identity.target())
                    .ok_or_else(|| {
                        backend_module(
                            "a resolved causal identity names a specialization with no projected \
                             unit"
                                .to_string(),
                        )
                    })?;
                let (offsets, _frame_bytes) = unit.slot_offsets()?;
                Ok((
                    identity.clone(),
                    DeclaredUnitCall {
                        function: module.declare_func_in_func(*target, func),
                        origin: unit.continuation_origin(),
                        call_site_origin: unit.continuation_origin(),
                        header: unit.header(),
                        slots: unit.slots().to_vec(),
                        offsets,
                    },
                ))
            })
            .collect()
    }

    /// Claim ONE exact token, at its producer occurrence.
    ///
    /// ⛔ This replaces a bulk discharge that consumed every token owned by a
    /// unit before any producer occurrence was reached. That was affine
    /// bookkeeping at the wrong seat: it could not tell a call that happened
    /// from one that did not, so it would have reported a clean ledger for a
    /// program that emitted no continuation call at all.
    ///
    /// Rejects an **absent** token, a **duplicate** claim, and a **wrong
    /// owner** -- and unlike the previous shape the owner check is reachable
    /// here, because the caller supplies the unit currently being defined
    /// rather than the token's own owner being used to select it.
    ///
    /// ⚠ An affine red is not self-explaining: confirm the identity still
    /// carries all four fields including `recursive_position` before believing
    /// a double-consumption, because a collided key reports it against the
    /// right token.
    pub(super) fn claim_exact(
        &mut self,
        identity: &ContinuationCallIdentity,
        defining: ContinuationEmissionOwner,
    ) -> Result<FuncId, CraneliftBackendError> {
        if identity.emission_owner() != defining {
            return Err(backend_module(
                "a continuation call token was claimed by a context that is not its emission \
                 owner; note this compares the EMISSION owner, not the raw source-occurrence \
                 provenance owner beside it"
                    .to_string(),
            ));
        }
        let consumed = self.claims.get_mut(identity).ok_or_else(|| {
            backend_module(
                "a continuation call token was claimed that this ledger never planned".to_string(),
            )
        })?;
        if let Some(previous) = consumed {
            return Err(backend_module(format!(
                "a continuation call token was claimed twice, first by {previous:?}; before \
                 reading this as a real double-consumption, confirm the causal identity still \
                 carries all four fields including recursive_position, because a collided key \
                 reports this against the right token"
            )));
        }
        *consumed = Some(defining);
        self.resolved.get(identity).copied().ok_or_else(|| {
            backend_module("a claimed causal token has no resolved target".to_string())
        })
    }

    /// No claim may be left over once every unit has been defined -- and, since
    /// `4b`, the exact planned population must equal the population that was
    /// resolved, declared, and **emitted**.
    ///
    /// ⭐ The emitted set is the one that is not bookkeeping: an identity enters
    /// it only after its emitted callee was decoded from the finished CLIF and
    /// matched the planner-issued target. Set equality against it is therefore
    /// "every planned continuation call became exactly one correct direct call,
    /// and no other continuation call was emitted."
    ///
    /// ⛔ Equality is asserted between sets, not between counts. Two sets of the
    /// same size can differ, and a length comparison here would pass for a
    /// population that swapped one token for another.
    pub(super) fn close(self) -> Result<(), CraneliftBackendError> {
        // `D8k` -- DECLARATION may remain over the full planned set. An unused
        // declaration is a `FuncRef` nobody called, not an emitted call, so the
        // declared population stays equal to planned even where the discharge
        // took the composed form.
        for (name, set) in [
            ("resolved", self.resolved.keys().cloned().collect::<BTreeSet<_>>()),
            ("declared", self.declared.clone()),
        ] {
            if set != self.planned {
                let missing = self.planned.difference(&set).count();
                let extra = set.difference(&self.planned).count();
                return Err(backend_module(format!(
                    "the {name} continuation call population does not equal the planned one: \
                     {missing} planned tokens absent, {extra} unplanned tokens present"
                )));
            }
        }
        // ⭐⭐ `D8k` -- THE PARTITION. `planned = direct-emitted ⊎
        // composed-consumed`, asserted as a disjoint union of two sets that
        // were accumulated from two different kinds of evidence: decoded direct
        // specialization emissions, and verified composed source-continuation
        // consumptions.
        //
        // ⛔ Not weakened to a count. Two sets of the right total size can still
        // be the wrong sets, and a program that emitted one token directly and
        // consumed a different one compositionally would satisfy any arithmetic
        // statement of this law.
        //
        // ⛔ Disjointness is asserted separately from coverage, because they
        // fail for different reasons: an overlap means one obligation was
        // answered twice, in two forms; a shortfall means one was never
        // answered at all.
        let both = self
            .emitted
            .intersection(&self.composed)
            .cloned()
            .collect::<BTreeSet<_>>();
        if !both.is_empty() {
            return Err(backend_module(format!(
                "{} causal tokens were discharged BOTH by a decoded direct emission and by a \
                 verified composed consumption; the two forms partition the planned population \
                 and an identity in both means one obligation was answered twice",
                both.len()
            )));
        }
        let discharged = self
            .emitted
            .union(&self.composed)
            .cloned()
            .collect::<BTreeSet<_>>();
        if discharged != self.planned {
            let missing = self.planned.difference(&discharged).count();
            let extra = discharged.difference(&self.planned).count();
            return Err(backend_module(format!(
                "the discharged continuation call population is not the planned one: {missing} \
                 planned tokens were neither directly emitted nor compositionally consumed, and \
                 {extra} discharged tokens were never planned. Direct: {}, composed: {}",
                self.emitted.len(),
                self.composed.len()
            )));
        }
        let leftover = self
            .claims
            .values()
            .filter(|consumed| consumed.is_none())
            .count();
        if leftover != 0 {
            return Err(backend_module(format!(
                "{leftover} planned continuation call tokens were never claimed by the unit that \
                 owns them"
            )));
        }
        // Owner agreement, asserted on the RECORDED consumer.
        //
        // Honest note: selection above is already by the token's own owner, so
        // this holds by construction today and cannot fire against the current
        // code. It is kept because it is the property `D3` actually promises,
        // and it is the check that would fire if selection were ever decoupled
        // from ownership. A wrong owner under today's structure does not reach
        // here -- it surfaces as a leftover claim above.
        for (identity, consumed) in &self.claims {
            if *consumed != Some(identity.emission_owner()) {
                return Err(backend_module(
                    "a continuation call token was claimed by a unit that does not own it"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// **`RT-DECL-CLOSURE-PORT` `D5a` checkpoint 2 — open the ONE cross-pass causal
/// ledger.**
///
/// Called from the orchestration before the first generated `Function`, so
/// every pass that can declare, claim or emit a causal call accumulates into
/// this single object.
///
/// ⛔ There is exactly one of these and it must not be re-opened. A second open
/// would discard whatever the first had accumulated and leave a partial
/// equality reading as a global one, which is the failure mode checkpoint 2
/// exists to remove — so re-opening rejects rather than replacing.
pub(super) fn open_continuation_claim_ledger(
    compiler: &mut Lowering<'_>,
    bundle: &UnitBundle,
) -> Result<(), CraneliftBackendError> {
    if compiler.continuation_claims.is_some() {
        return Err(backend_module(
            "the continuation claim ledger is already open; one artifact has exactly one ledger              and re-opening would silently discard every token recorded so far"
                .to_string(),
        ));
    }
    compiler.continuation_claims = Some(ContinuationClaimLedger::open(
        &compiler.static_transition_plan,
        bundle,
    )?);
    // `D7` — the aggregate allocation relation opens on the same boundary and
    // for the same reason: one artifact has exactly one relation, and every
    // body's events commit into it.
    compiler.aggregate_allocations = Some(AggregateAllocationLedger::default());
    // `D7` — the host-effect seat ledger opens on the same boundary: one
    // artifact has one consumed-seat evidence, and every body claims into it.
    compiler.host_effect_seats = Some(EffectSeatLedger::default());
    Ok(())
}

/// **`D5a` checkpoint 2 — close it, once, after every generated `Function`.**
///
/// The single global `planned = resolved = declared = claimed = emitted`
/// equality. ⛔ Not a per-pass partial: a pass that discharges nothing is
/// normal, and only the whole-artifact set answers whether every planned causal
/// token was discharged exactly once.
pub(super) fn close_continuation_claim_ledger(
    compiler: &mut Lowering<'_>,
) -> Result<(), CraneliftBackendError> {
    compiler
        .continuation_claims
        .take()
        .ok_or_else(|| backend_module("the continuation claim ledger went missing".to_string()))?
        .close()
}

/// **`D7` — close the aggregate allocation relation once, over the whole
/// compilation.**
///
/// ⛔ Not a per-body partial. One planner record may govern events in several
/// bodies -- a synthesized role at a seat reached under both a predeclared unit
/// and a generated specialization allocates in both -- so `image(R_f) = P` is
/// false for every individual body and imposing it would refuse lawful
/// programs. Only the whole-artifact relation answers whether every planned
/// record was allocated and every event was planned.
pub(super) fn close_aggregate_allocation_ledger(
    compiler: &mut Lowering<'_>,
) -> Result<AggregateRelationClosure, CraneliftBackendError> {
    let planned = compiler.static_transition_plan.aggregate_ownership_records();
    compiler
        .aggregate_allocations
        .take()
        .ok_or_else(|| {
            backend_module("the aggregate allocation ledger went missing".to_string())
        })?
        .close(planned)
}

/// **`D7` — close the host-effect seat authority once, over the whole
/// compilation.**
///
/// ⛔ Whole-artifact rather than per-body, because a seat inside a worker body
/// is consumed in its predeclared unit and again in each specialization that
/// contains it; no single body's claims equal the population.
///
/// ⭐ **`image(claims) ⊆ P`, exactly as the aggregate relation states it — NOT
/// an equality.** This comment previously claimed the opposite, on the reasoning
/// that a seat population derived from the source's own `Effect` occurrences
/// must be fully reached. That was measured false: an occurrence sitting in a
/// declaration body a compilation never emits takes its seats with it, and
/// requiring equality refused such a program. `P` authorizes; it does not
/// oblige, and an unreached member is lawful and reported. A half-read
/// occurrence cannot hide in that gap, because completeness is a group-local
/// equality that has already run at each visit's close.
pub(super) fn close_host_effect_seat_ledger(
    compiler: &mut Lowering<'_>,
) -> Result<EffectSeatClosure, CraneliftBackendError> {
    let planned = compiler
        .static_transition_plan
        .host_effect_seat_records()
        .to_vec();
    #[cfg(test)]
    if effect_seat_visit_mutation()
        == EffectSeatVisitMutation::DropCommittedGroupBeforeGlobalClose
    {
        if let Some(ledger) = compiler.host_effect_seats.as_mut() {
            ledger.drop_one_committed_group_for_tests();
        }
    }
    let closure = compiler
        .host_effect_seats
        .take()
        .ok_or_else(|| backend_module("the host effect seat ledger went missing".to_string()))?
        .close(&planned)?;
    #[cfg(test)]
    LAST_EFFECT_SEAT_CLOSURE.with(|cell| *cell.borrow_mut() = Some(closure.clone()));
    Ok(closure)
}

/// What the last completed seat closeout on this thread measured.
///
/// ⚠ Like `b2f_last_unit_emission`, this carries no statement about WHICH
/// compile produced it: a compile that fails before the closeout leaves the
/// previous reading standing. Read it only where one compile is known to have
/// closed.
#[cfg(test)]
thread_local! {
    static LAST_EFFECT_SEAT_CLOSURE: std::cell::RefCell<Option<EffectSeatClosure>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn last_effect_seat_closure() -> Option<EffectSeatClosure> {
    LAST_EFFECT_SEAT_CLOSURE.with(|cell| cell.borrow().clone())
}

/// How the `BufferAllocate` capacity seat was DISPATCHED, as `(specialized,
/// carried)`, since the last reset on this thread.
///
/// ⭐ **The premise a carried-capacity control cannot do without.** "The
/// compile succeeded and returned `InvalidBounds`" is the same green whether
/// the capacity took the carried route or the specialized one, so a fixture
/// that quietly stops carrying its capacity would leave the carried arm
/// untested and every assertion about it still passing. This is the only
/// instrument that separates those two worlds.
///
/// ⚠ It counts EMISSIONS, not executions: one compiled arm may run many times
/// or none. A control that wants "the carried route ran" needs the program's
/// own result, and this to know which route was compiled.
#[cfg(test)]
pub(in crate::cranelift_backend) fn capacity_phase_dispatch() -> (usize, usize) {
    super::CAPACITY_PHASE_DISPATCH.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::cranelift_backend) fn reset_capacity_phase_dispatch() {
    super::CAPACITY_PHASE_DISPATCH.with(|cell| cell.set((0, 0)));
}

/// **`D5a` checkpoint 4 step 1 — declare every generated context into ONE
/// generated function.**
///
/// Same per-function discipline as `WorkerTargets::declare_in_func`: the
/// `FuncRef`s belong to `func` alone and are never copied between functions.
/// ⛔ Keyed by the planner's `ContinuationContextId`, never by the body origin
/// the context executes -- that key is what would let a consumer resolve a
/// context from a body origin, which is the reconstruction the ruling forbids.
pub(in crate::cranelift_backend) fn declare_context_calls_in_func<M: Module>(
    module: &mut M,
    func: &mut Function,
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
) -> Result<BTreeMap<ContinuationContextId, DeclaredUnitCall>, CraneliftBackendError> {
    let mut calls = BTreeMap::new();
    for context in plan.continuation_contexts()? {
        let target = bundle.context(context.id()).ok_or_else(|| {
            backend_module(
                "a planned generated context was never forward-declared".to_string(),
            )
        })?;
        let (offsets, _frame_bytes) = context.slot_offsets()?;
        calls.insert(
            context.id(),
            DeclaredUnitCall {
                function: module.declare_func_in_func(target, func),
                // The context EXECUTES this body, so the origin it answers for
                // is unchanged and the source edge it serves is untouched.
                origin: context.worker_body_origin(),
                call_site_origin: context.worker_body_origin(),
                header: context.header(),
                slots: context.slots().to_vec(),
                offsets,
            },
        );
    }
    Ok(calls)
}

pub(super) fn define_unit_bodies<M: Module>(
    module: &mut M,
    compiler: &mut Lowering<'_>,
    helpers: ArtifactHelpers<'_>,
    bundle: &UnitBundle,
    call_edges: &CallEdgeTargets,
    staged_root_value: Option<&RuntimeValue>,
) -> Result<RootUnitResult, CraneliftBackendError> {
    let root = compiler.static_transition_plan.root_emittable_unit()?.function();
    // `D4`: projected once, declared afresh into each generated function below.
    let worker_targets = resolve_worker_targets(&compiler.static_transition_plan, bundle)?;
    // `RT-DECL-CLOSURE-PORT` `D5` — opened here because this bundle pass is the
    // only place a checked same-SCC call can reach a declaration-owned unit.
    compiler.checked_call_ledger = Some(CheckedCallLedger::open(
        compiler.oriented_subcontinuation_plan.as_ref(),
    ));
    let mut root_result = None;
    // `D5a` checkpoint 1: the SAME population `declare_unit_bundle` walked.
    // Both passes read `executable_units` so declared and defined cannot
    // disagree -- reading different methods here is exactly how a phantom
    // appears.
    let emissions = compiler
        .static_transition_plan
        .executable_units()?
        .into_iter()
        .map(|unit| {
            let (offsets, frame_bytes) = unit.slot_offsets()?;
            Ok(OwnedUnitEmission {
                function: unit.function(),
                origin: unit.origin(),
                header: unit.header(),
                slots: unit.slots().to_vec(),
                offsets,
                frame_bytes,
            })
        })
        .collect::<Result<Vec<_>, CraneliftBackendError>>()?;
    for unit in emissions {
        let id = bundle.function(unit.function).ok_or_else(|| {
            backend_module("a planned unit was never forward-declared".to_string())
        })?;
        let is_root = root == unit.function;
        let outcome = define_unit_body(
            module,
            compiler,
            helpers,
            unit,
            id,
            bundle,
            call_edges,
            &worker_targets,
            is_root,
            staged_root_value,
        )?;
        if let Some(outcome) = outcome {
            if root_result.replace(outcome).is_some() {
                return Err(backend_module(
                    "more than one emitted unit claimed root result authority".to_string(),
                ));
            }
        }
    }
    // ⛔ The continuation ledger is NOT closed here. `D5a` checkpoint 2 moved
    // its lifetime out to the orchestration in `core.rs`, because two more
    // passes after this one declare, claim and emit causal calls and closing
    // here reported their tokens absent before they could exist.
    // `D5` closeout, before the artifact is published: planned = consumed =
    // emitted, and every emitted actual callee equals its exact resolved
    // target. ⛔ `consumed` is the affine machinery's OWN set, passed in rather
    // than mirrored, so the two cannot drift apart.
    compiler
        .checked_call_ledger
        .take()
        .ok_or_else(|| backend_module("the checked call ledger went missing".to_string()))?
        .close(&compiler.consumed_recursive_call_templates)?;
    root_result.ok_or_else(|| {
        backend_module("the emitted unit bundle did not define its recorded root".to_string())
    })
}

struct OwnedUnitEmission {
    function: PredeclaredFunctionId,
    origin: StaticOriginId,
    header: AbiFrameHeader,
    slots: Vec<AbiSlot>,
    offsets: Vec<u32>,
    frame_bytes: u32,
}

fn define_unit_body<M: Module>(
    module: &mut M,
    compiler: &mut Lowering<'_>,
    helpers: ArtifactHelpers<'_>,
    unit: OwnedUnitEmission,
    id: FuncId,
    bundle: &UnitBundle,
    call_edges: &CallEdgeTargets,
    worker_targets: &WorkerTargets,
    is_root: bool,
    staged_root_value: Option<&RuntimeValue>,
) -> Result<Option<RootUnitResult>, CraneliftBackendError> {
    // ⭐ The declared size and the walked size must agree. They are the same
    // walk by construction (`abi::slot_offsets` totals for both), so this
    // rejects a corrupted descriptor rather than a divergent derivation.
    if unit.frame_bytes != unit.header.frame_bytes {
        return Err(backend_module(
            "abi frame size disagrees with its own slot run".to_string(),
        ));
    }
    let result_offset = unit
        .slots
        .iter()
        .zip(&unit.offsets)
        .find(|(slot, _)| slot.kind == AbiSlotKind::Result)
        .map(|(_, offset)| *offset)
        .ok_or_else(|| {
            // ⛔ Fails closed. `CONVENTION_SLOTS` puts a `Result` slot in every
            // unit, so its absence means the descriptor is not the one `B2R`
            // built, and returning a default word would fabricate a result.
            backend_module("unit frame declares no result slot".to_string())
        })?;
    let trap_offset = unit
        .slots
        .iter()
        .zip(&unit.offsets)
        .find(|(slot, _)| slot.kind == AbiSlotKind::Trap)
        .map(|(_, offset)| *offset)
        .ok_or_else(|| backend_module("unit frame declares no trap slot".to_string()))?;

    let sig = unit_signature(module);
    let mut func = Function::with_name_signature(UserFuncName::user(2, id.as_u32()), sig);
    // ⭐ `D4` — this unit's callees are referenced HERE, by the static identity
    // the planner assigned, before the body exists to call them.
    //
    // ⛔ **The call instructions themselves are `S6`'s**, because a unit body
    // does not descend into its own expression until `lower_expr`'s consumers
    // switch over. ⇒ What is live today is the **reference**: a `FuncRef`
    // resolved from a validated call edge through the declared bundle, with no
    // ordinal, no name parsing and no dynamic lookup anywhere on the path. ⚠ An
    // emitted `call` is not claimed and no control here asserts one.
    #[cfg(test)]
    let unit_trap_authority =
        match TRAP_FRAME_BINDING_MUTATION.with(std::cell::Cell::get) {
            TrapFrameBindingMutation::MisclassifyUnitAsRoot => Some(TrapExitAuthority::Root {
                process_sentinel: false,
                source_authorized: false,
            }),
            TrapFrameBindingMutation::Exact | TrapFrameBindingMutation::DeleteUnitLane => {
                None
            }
        };
    #[cfg(not(test))]
    let unit_trap_authority = None;
    let mut function_local =
        helpers.declare_in_func(module, &mut func, unit_trap_authority);
    let declared_calls = call_edges.declare_in_func(unit.function, module, &mut func)?;
    // `D3` — this ordinary Function declares its OWN `FuncRef` for every causal
    // token it owns, keyed by the four-field identity. Minted here, into this
    // `Function`; never passed across functions.
    function_local.continuation_calls = match compiler.continuation_claims.as_ref() {
        Some(ledger) => ledger.declare_owned_in_func(
            ContinuationEmissionOwner::Predeclared(unit.function),
            module,
            &mut func,
            &compiler.static_transition_plan,
        )?,
        None => BTreeMap::new(),
    };
    // `4b`: the DECLARED half of the four-set equality, recorded where the refs
    // are actually minted.
    if let Some(ledger) = compiler.continuation_claims.as_mut() {
        ledger.record_declared(function_local.continuation_calls.keys().cloned())?;
    }
    // `RT-DECL-CLOSURE-PORT` `D5a` contract 3 — AUTHORITY BEFORE EMISSION.
    //
    // Projected here, beside the declaration of this unit's own call refs and
    // **before the function is defined**. ⛔ It is a projection of authority the
    // planner already issued, keyed on the owner this pass is about to define;
    // lowering supplies only that owner and never searches, reverse-derives a
    // consumer, or reads an owner back off anything it emitted.
    let result_edges = compiler
        .static_transition_plan
        .continuation_result_edges_owned_by(ContinuationEmissionOwner::Predeclared(
            unit.function,
        ))?;
    // `D3`: the owner operand for the claim, supplied independently of any
    // token -- this is the ordinary producer unit currently being defined.
    #[cfg(test)]
    d5a_trace(format!(
        "UNIT-BODY entry function={:?} origin={:?}",
        unit.function, unit.origin
    ));
    compiler.open_aggregate_events(id)?;
    // `D8o` — bound for this body's lifetime and released on exit, so no later
    // Function inherits it. ⛔ The domain is unchanged: `D5a`'s rule that an
    // ordinary predeclared unit body emits as itself.
    let ambient = AmbientBodyAuthority::bind(
        compiler,
        ContinuationEmissionOwner::Predeclared(unit.function),
        unit.function,
    );
    #[cfg(test)]
    crate::cranelift_backend::lowering::record_d8o_body_key(
        compiler.defining_function_id,
        crate::cranelift_backend::lowering::D8oBodyKey::OrdinaryUnit(unit.function),
    );
    function_local.unit_calls = declared_calls.static_bodies;
    function_local.declaration_calls = declared_calls.declarations;
    // `RT-DECL-CLOSURE-PORT` `D5` — the causal control on the ABI half.
    //
    // ⛔⛔ **Injected HERE, on the function-local COPY, and never on the plan's
    // descriptor.** That asymmetry is the entire point: `D5`'s ABI
    // reconciliation claims the declared call record still agrees with the
    // immutable descriptor `D3` validated. A mutation applied to both sides
    // would leave them agreeing and prove nothing; a mutation applied to the
    // descriptor would trip the ABI plane upstream instead. ⇒ Only a mutation
    // of the copy measures the reconciliation itself.
    #[cfg(test)]
    d5_mutate_declared_calls(&mut function_local.declaration_calls);
    // `D4`: this function's own worker refs, minted here and never copied.
    function_local.worker_calls = worker_targets.declare_in_func(module, &mut func);
    // `D6b` -- an ordinary unit body performs no retarget, so the two tables
    // agree. Populated anyway, for the same reason as the context body above.
    function_local.raw_worker_calls = function_local.worker_calls.clone();
    // `D5a` checkpoint 1: the raw template contracts, beside the call targets
    // and deliberately not derived from them.
    function_local.worker_templates = worker_targets.templates().clone();
    // `D5a` checkpoint 4 step 1: this function's own context call targets.
    function_local.context_calls = declare_context_calls_in_func(
        module,
        &mut func,
        &compiler.static_transition_plan,
        bundle,
    )?;
    // `D8n` — this generated Function's own checked-frame consumption
    // transaction, spanning the ordinary unit body exactly. ⛔ Opened before the builder and
    // closed after it, so every branch scope inside nests within it.
    let frame_scope = CheckedFrameFunctionScope::open(compiler)?;
    let mut func_ctx = FunctionBuilderContext::new();
    let root_outcome;
    {
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let envelope = builder.block_params(entry)[0];
        let slots = builder.ins().load(
            module.target_config().pointer_type(),
            MemFlags::trusted(),
            envelope,
            crate::activation_services::UNIT_CALL_FRAME_SLOTS,
        );
        let host_dispatch_context = builder.ins().load(
            module.target_config().pointer_type(),
            MemFlags::trusted(),
            envelope,
            crate::activation_services::UNIT_CALL_FRAME_HOST_DISPATCH_CONTEXT,
        );
        let services = builder.block_params(entry)[1];
        let native_int_arena = builder.ins().load(
            module.target_config().pointer_type(),
            MemFlags::trusted(),
            services,
            crate::activation_services::SERVICES_NATIVE_INT_ARENA,
        );
        Lowering::require_nonzero(&mut builder, native_int_arena);
        let boundary_arena = builder.ins().load(
            module.target_config().pointer_type(),
            MemFlags::trusted(),
            services,
            crate::activation_services::SERVICES_BOUNDARY_ARENA,
        );
        Lowering::require_nonzero(&mut builder, boundary_arena);
        function_local.host_dispatch_context = Some(host_dispatch_context);
        function_local.native_int_arena = Some(native_int_arena);
        function_local.boundary_arena = Some(boundary_arena);
        function_local.services_pointer = Some(services);
        // The two fixed envelope loads are unconditional. Semantic frame
        // accesses below are relative only to the B2R payload base.
        #[cfg(test)]
        let bind_unit_trap_frame = TRAP_FRAME_BINDING_MUTATION.with(std::cell::Cell::get)
            != TrapFrameBindingMutation::DeleteUnitLane;
        #[cfg(not(test))]
        let bind_unit_trap_frame = true;
        if bind_unit_trap_frame {
            function_local.bind_unit_trap_frame(
                slots,
                i32::try_from(trap_offset).map_err(|_| {
                    backend_module("abi trap slot offset exceeds addressable range".to_string())
                })?,
            )?;
        }
        compiler.function_local = function_local;
        #[cfg(test)]
        if is_root
            && PROCESS_SLOT_MUTATION.with(std::cell::Cell::get)
                == ProcessSlotMutation::AttemptFixedContextOffsets
        {
            return Err(backend_module(
                "fixed host-dispatch context is not semantic process-pair storage".to_string(),
            ));
        }
        let mut env = Vec::new();
        for (slot, offset) in unit.slots.iter().zip(&unit.offsets) {
            if matches!(slot.kind, AbiSlotKind::Parameter | AbiSlotKind::Capture) {
                #[cfg(test)]
                let (base, offset) = if is_root
                    && slot.kind == AbiSlotKind::Parameter
                    && PROCESS_SLOT_MUTATION.with(std::cell::Cell::get)
                        == ProcessSlotMutation::ReintroduceLaunchIngress
                {
                    let offset = match slot.ordinal {
                        0 => crate::boundary_activation::ROOT_INGRESS_PROCESS_INPUT,
                        1 => crate::boundary_activation::ROOT_INGRESS_CAPABILITY,
                        _ => {
                            return Err(backend_module(
                                "the process-root mutation found an unknown parameter role"
                                    .to_string(),
                            ));
                        }
                    };
                    // The root adapter's companion mutation explicitly put
                    // launch ingress here. Without that producer-side change,
                    // the retained direct host context is not an admissible
                    // semantic-slot source.
                    (host_dispatch_context, offset)
                } else {
                    (
                        slots,
                        i32::try_from(*offset).map_err(|_| {
                            backend_module(
                                "abi input slot offset exceeds addressable range".to_string(),
                            )
                        })?,
                    )
                };
                #[cfg(not(test))]
                let (base, offset) = (
                    slots,
                    i32::try_from(*offset).map_err(|_| {
                        backend_module("abi input slot offset exceeds addressable range".to_string())
                    })?,
                );
                let word = builder.ins().load(
                    types::I64,
                    MemFlags::trusted(),
                    base,
                    offset,
                );
                let carried = CarriedBoundaryWord { word };
                // The process root's two ABI ordinals are closed semantic
                // roles, not generic ValueWord inputs. Recovering them here
                // prevents a borrowed process-input body from being emitted
                // twice behind a runtime carried-representation split.
                let operand = if is_root
                    && compiler.process_object
                    && slot.kind == AbiSlotKind::Parameter
                {
                    let value = compiler.emit_carrier_scalar(&mut builder, carried)?;
                    match slot.ordinal {
                        ordinal
                            if ordinal == AbiProcessParameter::ProcessInput.ordinal() =>
                        {
                            LoweringOperand::Specialized(Lowered::BorrowedNativeValue {
                                pointer: value,
                            })
                        }
                        ordinal
                            if ordinal == AbiProcessParameter::Capability.ordinal() =>
                        {
                            LoweringOperand::Specialized(Lowered::CapabilityToken {
                                value,
                            })
                        }
                        _ => {
                            return Err(backend_module(
                                "the process root has an unknown parameter role".to_string(),
                            ));
                        }
                    }
                } else {
                    LoweringOperand::Carried(carried)
                };
                // `D5a` checkpoint 4 step 1b: the SAME operand, recorded by ABI
                // position. ⭐ Taken from this one walk rather than rebuilt, so
                // "index i is ABI position i" holds by construction instead of
                // by two walks agreeing.
                compiler
                    .function_local
                    .defining_abi_operands
                    .push(operand.clone());
                env.push(LoweringEnvironmentBinding::Value(operand));
            }
        }
        // The in-process validation API historically stages one ground
        // `RuntimeValue` as the root environment.  It is compile-time material,
        // not launch ingress and not a generated-call transfer, so it does not
        // acquire an ABI slot.  The value is lowered exactly once inside the
        // selected root unit; descendants can receive it only through their
        // ordinary declared captures.
        if is_root {
            if let Some(value) = staged_root_value {
                env.push(LoweringEnvironmentBinding::Value(
                    LoweringOperand::Specialized(compiler.lower_value(&mut builder, value)?),
                ));
            }
        }
        if is_root {
            compiler.root_terminal_authority =
                compiler.take_distinguished_root_answer_authority()?;
        }
        // The explicit root *entry* selects the unit, but a root
        // `ComputationalMatch` deliberately schedules its scrutinee while its
        // source record belongs to the distinct root occurrence.  Body
        // selection therefore uses the recorded occurrence only after the
        // unmintable entry has selected the descriptor.
        let body_origin = if is_root {
            compiler.static_transition_plan.root_static_origin()?
        } else {
            unit.origin
        };
        let body = compiler.retained_body_occurrence(body_origin)?;
        compiler.select_terminal_result_origins(body_origin, body.expr)?;
        let lowered = compiler.lower_expr(&mut builder, body, &env)?;
        compiler.validate_join_plan_consumption(unit.function)?;
        let (result, outcome) = if is_root {
            match lowered {
                LoweringOperand::Carried(word) if !compiler.process_object => (
                    Some(word.word),
                    Some(RootUnitResult {
                        decoder: Some(ResultDecoder::Boundary),
                        trap: None,
                    }),
                ),
                LoweringOperand::Carried(word) => {
                    let tag = builder.ins().band_imm(
                        word.word,
                        crate::boundary_value::BOUNDARY_TAG_MASK as i64,
                    );
                    Lowering::require_i64(
                        &mut builder,
                        tag,
                        BoundaryTag::ImmediateExitStatus as i64,
                    );
                    let status = compiler.emit_carrier_scalar(&mut builder, word)?;
                    (
                        Some(status),
                        Some(RootUnitResult {
                            decoder: Some(ResultDecoder::ProcessStatus),
                            trap: None,
                        }),
                    )
                }
                LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                    #[cfg(test)]
                    if compiler.process_object {
                        px8tr_record_trap_provenance(
                            Px8trTrapProvenanceEvent::FinalProcessObjectTrap {
                                trap: trap.clone(),
                            },
                        );
                    }
                    compiler.emit_current_trap(&mut builder, &trap)?;
                    (
                        None,
                        Some(RootUnitResult {
                            decoder: Some(ResultDecoder::TrapOnly),
                            trap: None,
                        }),
                    )
                }
                LoweringOperand::Specialized(value) => {
                    let (token, decoder) = compiler.emit_result(&mut builder, value)?;
                    (
                        Some(token),
                        Some(RootUnitResult {
                            decoder: Some(decoder),
                            trap: None,
                        }),
                    )
                }
            }
        } else {
            // `RT-DECL-CLOSURE-PORT` `D5a` — THE DETACHED-RESULT SEAT.
            //
            // The exact retained result is lowered; nothing has been
            // transferred into a carrier, allocated, published or joined. This
            // is where the landed object fixture was measured to refuse, and it
            // is the seat for every producer owner the fixed point detached as
            // an ordinary unit result.
            //
            // ⛔ Applied on the non-root path only, because this is the only
            // result surface that reaches `transfer_unit_result_into_carrier`.
            // A root that owned an undischarged projected call would be caught
            // by the whole-pass claim closure, not silently dropped.
            let lowered = compiler.eliminate_detached_producer_continuation(
                &mut builder,
                &result_edges,
                lowered,
                &env,
            )?;
            let word = match lowered {
                LoweringOperand::Carried(word) => Some(word.word),
                LoweringOperand::Specialized(Lowered::Trap(trap)) => {
                    compiler.emit_current_trap(&mut builder, &trap)?;
                    None
                }
                LoweringOperand::Specialized(value) => Some(
                    compiler
                        .transfer_unit_result_into_carrier(&mut builder, unit.origin, &value)?
                        .word,
                ),
            };
            (word, None)
        };
        root_outcome = outcome;
        if let Some(result) = result {
            builder.ins().store(
                MemFlags::trusted(),
                result,
                slots,
                i32::try_from(result_offset).map_err(|_| {
                    backend_module("abi result slot offset exceeds addressable range".to_string())
                })?,
            );
        }
        let status = builder.ins().iconst(types::I64, 0);
        builder.ins().return_(&[status]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    ambient.release(compiler);
    frame_scope.close(compiler)?;
    #[cfg(test)]
    d5a_trace(format!(
        "UNIT-BODY done function={:?} origin={:?} root={:?}",
        unit.function, unit.origin, root_outcome.as_ref().map(|_| "root")
    ));
    compiler.validate_materialized_dead_join_cfg(unit.function, &func)?;
    // `4b` -- the emission-seam equality gate, on the FINISHED function and
    // before it is defined into the module. The callee of every recorded causal
    // emission is decoded out of this CLIF and compared with the planner-issued
    // target; a disagreement rejects here rather than being emitted.
    compiler.verify_emitted_continuation_calls(&func, bundle)?;
    // `D8j` — same placement in the ordinary unit-body pass. ⛔ Both passes
    // gate, because a composed discharge can be claimed in either.
    compiler.verify_recorded_composed_discharges(&func, bundle)?;
    #[cfg(test)]
    crate::cranelift_backend::lowering::record_d8j_discharged(
        compiler.function_local.composed_discharges.keys().cloned(),
    );
    // `D8k` -- same accumulation on the ordinary pass. ⛔ The owner supplied is
    // this unit's own emission owner, so `record_composed` can hold the
    // discharge to the identity it claims rather than trusting the seat.
    if let Some(ledger) = compiler.continuation_claims.as_mut() {
        ledger.record_composed(
            compiler.function_local.composed_discharges.keys().cloned(),
            ContinuationEmissionOwner::Predeclared(unit.function),
        )?;
    }
    // `4b` closeout control: verify this function's emissions but never
    // accumulate them, so whole-pass set equality has a population to miss.
    #[cfg(test)]
    let accumulate = CONTINUATION_EMISSION_MUTATION.with(std::cell::Cell::get)
        != ContinuationEmissionMutation::SuppressEmissionAccumulation;
    #[cfg(not(test))]
    let accumulate = true;
    if let Some(ledger) = compiler.continuation_claims.as_mut() {
        if accumulate {
            ledger.record_emitted(compiler.function_local.continuation_emissions.keys().cloned())?;
        }
    }
    verify_cranelift_function(&func, module.isa())?;
    compiler.commit_aggregate_events()?;
    #[cfg(test)]
    scale_b_record_unit_body(&func);
    let mut ctx = module.make_context();
    std::mem::swap(&mut ctx.func, &mut func);
    module
        .define_function(id, &mut ctx)
        .map_err(|err| backend_module(err.to_string()))?;
    // ⛔ Counted HERE, adjacent to `define_function`, and NOT at the call site
    // in the loop above -- where it was first written and where it was
    // worthless.
    //
    // ⭐ A mutation gating the `define_unit_body` call left this test GREEN,
    // because a counter incremented once per loop iteration compares the
    // bundle's length to the length of the collection the loop walks, which are
    // equal by construction. It proved the loop ran and CLAIMED bodies were
    // defined. Only an increment on the emitting path can tell those apart.
    #[cfg(test)]
    B2F_UNIT_EMISSION.with(|cell| {
        let (declared, defined) = cell.get();
        cell.set((declared, defined + 1));
    });
    Ok(root_outcome)
}
