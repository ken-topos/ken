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
    edges: Vec<(PredeclaredFunctionId, FuncId)>,
}

impl CallEdgeTargets {
    /// The resolved targets of every call emitted **into** `caller`.
    ///
    /// ⚠ Returns an empty iterator for a unit with no outgoing call edges, which
    /// is the common case: most units are leaves.
    pub(in crate::cranelift_backend) fn targets_in(
        &self,
        caller: PredeclaredFunctionId,
    ) -> impl Iterator<Item = FuncId> + '_ {
        self.edges
            .iter()
            .filter(move |(from, _)| *from == caller)
            .map(|(_, id)| *id)
    }

    /// How many call edges were resolved.
    #[cfg(test)]
    pub(in crate::cranelift_backend) fn len(&self) -> usize {
        self.edges.len()
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
        edges.push((edge.caller(), target));
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

/// The uniform call ABI for every target unit: one pointer to the activation
/// frame, returning one `i64`.
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
fn unit_signature<M: Module>(module: &M) -> cranelift_codegen::ir::Signature {
    let mut sig = module.make_signature();
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
    #[cfg(test)]
    B2F_UNIT_EMISSION.with(|cell| cell.set((functions.len(), 0)));
    Ok(UnitBundle { functions })
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
pub(in crate::cranelift_backend) fn define_unit_bodies<M: Module>(
    module: &mut M,
    plan: &StaticTransitionPlan<'_>,
    bundle: &UnitBundle,
    call_edges: &CallEdgeTargets,
) -> Result<(), CraneliftBackendError> {
    for unit in plan.emittable_units()? {
        let id = bundle.function(unit.function()).ok_or_else(|| {
            backend_module("a planned unit was never forward-declared".to_string())
        })?;
        let targets = call_edges.targets_in(unit.function()).collect::<Vec<_>>();
        define_unit_body(module, unit, id, &targets)?;
    }
    Ok(())
}

fn define_unit_body<M: Module>(
    module: &mut M,
    unit: EmittableUnit<'_>,
    id: FuncId,
    call_targets: &[FuncId],
) -> Result<(), CraneliftBackendError> {
    let (offsets, frame_bytes) = unit.slot_offsets()?;
    // ⭐ The declared size and the walked size must agree. They are the same
    // walk by construction (`abi::slot_offsets` totals for both), so this
    // rejects a corrupted descriptor rather than a divergent derivation.
    if frame_bytes != unit.header().frame_bytes {
        return Err(backend_module(
            "abi frame size disagrees with its own slot run".to_string(),
        ));
    }
    let result_offset = unit
        .slots()
        .iter()
        .zip(&offsets)
        .find(|(slot, _)| slot.kind == AbiSlotKind::Result)
        .map(|(_, offset)| *offset)
        .ok_or_else(|| {
            // ⛔ Fails closed. `CONVENTION_SLOTS` puts a `Result` slot in every
            // unit, so its absence means the descriptor is not the one `B2R`
            // built, and returning a default word would fabricate a result.
            backend_module("unit frame declares no result slot".to_string())
        })?;

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
    for target in call_targets {
        module.declare_func_in_func(*target, &mut func);
    }
    let mut func_ctx = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        let frame = builder.block_params(entry)[0];
        let result = builder.ins().load(
            types::I64,
            MemFlags::trusted(),
            frame,
            i32::try_from(result_offset).map_err(|_| {
                backend_module("abi result slot offset exceeds addressable range".to_string())
            })?,
        );
        builder.ins().return_(&[result]);
        builder.seal_all_blocks();
        builder.finalize();
    }
    verify_cranelift_function(&func, module.isa())?;
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
    Ok(())
}
