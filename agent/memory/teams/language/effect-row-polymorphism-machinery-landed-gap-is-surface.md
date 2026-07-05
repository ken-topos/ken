---
scope: teams/language
audience: (see scope README)
source: private memory `effect-row-polymorphism-machinery-landed-gap-is-surface`
---

# Effect-row polymorphism machinery is landed; the D1 gap is the surface binder

**SURF-1 D1 ★ (`purity-keywords-effect-polymorphism.md @ cb90bcf`), my
Architect-owned mechanism lane (2026-07-04).** spec-leader routed the row
*variable* to me, same shape as CAT-1. Ruling cast `evt_53ybqtzjfv7yx` in
`thr_2czcdnr43ahy4`. I re-touch this at the SURF-1 Language build gate and at
**CAT-2** (`traverse` is the first surface effect-polymorphic def and the first
recursive row-poly consumer). Companion to higher kinded class param and funext
definitional (same "internal machinery present, surface path missing" shape as
CAT-1's `class` param) and enclave ruling in thread is not a durable deliverable
(owes a §36 transcription I fidelity-gate).

**The decisive grounding — machinery landed, gap is surface.** On `origin/main`,
`crates/ken-elaborator/src/effects/`:
- `row.rs`: `RowVar(u32)` (`:100`); `RowType = Concrete | Var | Join` (`:110`);
  `is_subset_of` with `Var(x)⊆Var(x)` (`:186`) + the conservative single-arm
  `x⊆Join(l,r) = x⊆l || x⊆r` (`:203`); `apply_subst` (`:238`); `Join` models an
  open row `[E|e]`.
- `row_poly.rs`: symbolic `infer_row_poly` (`:38`), `check_row_poly_escape`
  (`:70`) — the precise path that REPLACED L5's conservative
  `unknown_effectful_params`/`check_higher_order_guard` valve.
- **`extract.rs:64` is the SOLE `RowVar` construction site**
  (`RowVarAllocator::fresh`), fired **only** by a HOF-effectful parameter
  (`classify_telescope` → `ParamTy::HofEffectful|Unknown`). **No surface path
  `visits [e]` → `RowType::Var` exists** — this is spec §1.3's "row polymorphism
  falls out of substituting the argument's type… there is no surface
  row-variable binder," verbatim, in code. Today's row-poly is
  implicit/inferred-from-HOF; D1 makes it surface-writable.

**The four-part ruling.** (1) **Surface** — `[e]` bare row var (lowercase ident)
+ `[E|e]` open tail, **bound as an implicit param (§39-style, NOT a new
binder)**; adopt both (the landed `Join` + subset/escape/subst already handle
the tail). **Required, not optional**: §3.1 guarantee 1 = effects
manifest-in-the-type; a never-written inferred var violates it and D2's
`proc`/`fn` bidirectional check must read the poly row off the signature. `e`
bound once, referenced in both the HOF param codomain and the result row → same
`RowVar` (the `infer.rs:103` two-sided-same-var setup). (2) **Static closure
(AC3) is STRUCTURAL**: a `RowVar` is eliminated only by
`apply_subst(e:= concrete)` at a concrete-arg call or stays latent (deferred to
the arg); every `Vis` is a static `perform` or a typed HOF closure; the boundary
run (`run_io`/handler) is always concrete (can't run a variable). **Bounded
build seam:** `infer_all` (§1.3 fixpoint, `infer.rs:175`) is **concrete-only**
(`HashMap<String,EffectRow>`), so a **recursive** row-poly def — `traverse`
folds a `List`, self-calls — needs the fixpoint lifted to `RowType` (stable:
`Var(e)∪Var(e)=Var(e)`, monotone, terminates); row-poly analog of CAT-1's
four-piece extension. **Soundness residual (fail-closed):** the single-arm
`Join` subset rule under-accepts a straddling concrete row (completeness gap,
rejects-valid) — **never over-accepts** (no effect silently escapes); sound for
the escape gate. (3) **Locked features compose**: a handler folding a subset of
`[E|e]` leaves `ITree (Var e)` polymorphic; totality + single-consumption are
properties of the `elim_ITree` fold + the §5.2 tail-position clauses, invariant
under whether the residual row is concrete or a var. **No `Cap e` param under a
variable** — a row-poly `proc` performs poly effects only through its HOF arg (a
closure carrying its own caps); it never `perform`s `e` itself, just splices via
`bind`; its own direct-`perform` `ρ_open` is `∅`. Grounded: `cap_set`/`CapParam`
are concrete-only (`algebra.rs:11,43`). (4) **Register**: pure spec addition
(new §1.5 + the §1.3 recursive-fixpoint note), **not a fresh operator OQ** —
`OQ-8` already DECIDED the model (Koka rows the cited precedent; the row var is
"implied by the model"); record as an **`OQ-8` child pin**, DECIDED-by-
elaboration. Kernel-untouched; no Steward re-fork.
