//! **`RT-FNSPLIT-B2F` `D1`/`D2` — the target code-unit population.**
//!
//! One closed Cranelift target function per `PredeclaredFunction` in the
//! validated `SemanticOwner` partition, forward-declared as a bundle and then
//! defined against `B2R`'s declared activation frame.
//!
//! ⛔ **This module does not derive the population and must never be made to.**
//! The unit set is `plan.entries` ∪ every `EdgeKind::StaticBody` **target**,
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
    pub(in crate::cranelift_backend) origin: StaticOriginId,
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
    by_origin: BTreeMap<StaticOriginId, ResolvedUnitTarget>,
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
                        header: target.header,
                        slots: target.slots.clone(),
                        offsets: target.offsets.clone(),
                    },
                )
            })
            .collect()
    }
}

/// Project the already-validated emittable units by exact body origin.
pub(in crate::cranelift_backend) fn resolve_worker_targets(
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
) -> Result<WorkerTargets, CraneliftBackendError> {
    let mut by_origin: BTreeMap<StaticOriginId, ResolvedUnitTarget> = BTreeMap::new();
    for unit in plan.emittable_units()? {
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
    Ok(WorkerTargets { by_origin })
}

pub(in crate::cranelift_backend) fn resolve_call_edges(
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
) -> Result<CallEdgeTargets, CraneliftBackendError> {
    let derived = plan.emittable_call_edges()?;
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
                call_site_origin: edge.call_site_origin(),
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
    for (ordinal, unit) in plan.emittable_units()?.into_iter().enumerate() {
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
    #[cfg(test)]
    B2F_UNIT_EMISSION.with(|cell| cell.set((functions.len(), 0)));
    Ok(UnitBundle {
        functions,
        continuations,
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
        worker_closure_origin: StaticOriginId,
        worker_body_origin: StaticOriginId,
        worker_declared_arity: u32,
        worker_capture_count: usize,
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
                worker_closure_origin: unit.worker_closure_origin(),
                worker_body_origin: unit.worker_body_origin(),
                worker_declared_arity: unit.worker_declared_arity(),
                worker_capture_count: unit.worker_capture_count(),
            })
        })
        .collect::<Result<Vec<_>, CraneliftBackendError>>()?;

    let mut defined = 0usize;
    for unit in emissions {
        let id = bundle.continuation(unit.id).ok_or_else(|| {
            backend_module(
                "a planned continuation specialization was never forward-declared".to_string(),
            )
        })?;
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

        let sig = unit_signature(module);
        let mut func =
            Function::with_name_signature(UserFuncName::user(0, id.as_u32()), sig);
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
        if !declared_workers.contains_key(&unit.worker_body_origin) {
            return Err(backend_module(
                "the selected continuation worker body has no projected emittable-unit target"
                    .to_string(),
            ));
        }
        function_local.unit_calls = declared_workers.clone();
        function_local.worker_calls = declared_workers;
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

            // The EXISTING constructor, with the projected identity and arity.
            let worker = compiler.construct_static_worker_binding(
                unit.worker_closure_origin,
                unit.worker_body_origin,
                unit.worker_declared_arity,
                unit.worker_capture_count,
                worker_captures,
            )?;

            // The semantic case environment, through the sole binding
            // authority: the continuation inputs in ordinal order, then the
            // worker installed in the selected case's binder order.
            let mut env: Vec<LoweringEnvironmentBinding> = carried_inputs
                .into_iter()
                .map(LoweringEnvironmentBinding::Value)
                .collect();
            env.insert(0, LoweringEnvironmentBinding::StaticWorker(worker));

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
        // Verify, then define THIS function -- a fresh context here would
        // define an empty body and silently discard everything emitted above.
        verify_cranelift_function(&func, module.isa())?;
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
    let mut root_result = None;
    let emissions = compiler
        .static_transition_plan
        .emittable_units()?
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
    function_local.unit_calls = declared_calls.static_bodies;
    function_local.declaration_calls = declared_calls.declarations;
    // `D4`: this function's own worker refs, minted here and never copied.
    function_local.worker_calls = worker_targets.declare_in_func(module, &mut func);
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
    compiler.validate_materialized_dead_join_cfg(unit.function, &func)?;
    verify_cranelift_function(&func, module.isa())?;
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
