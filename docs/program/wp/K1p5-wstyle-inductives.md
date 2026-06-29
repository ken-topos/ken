# WP K1.5 — W-style (Π-bound) recursive inductives + their eliminator

> **Status:** Steward frame — **awaiting spec-leader elaboration** (newly active:
> surfaced as L5's hard dependency, Architect-confirmed 2026-06-29). spec-leader
> elaborates the admission rule + eliminator/ι semantics (extending
> `spec/10-kernel/14-inductive.md`), then the **Kernel team** builds.
>
> **Team:** Kernel · **Deps:** K1 (done — inductive machinery + structural
> eliminator), K2c series-1 (done — the ι-reduction feeds conversion) · **Size:**
> M · **Risk:** ★★★ (**trust root** — new admission + new eliminator + new ι in
> the TCB) · ► **High fan-out:** unblocks L5's denotation half, X1's effects, and
> the WS-Sec / WS-B workstreams (all ride the `ITree`).

## Objective

Admit **strictly-positive recursive inductives with a Π-bound (W-style) recursive
occurrence** — a constructor argument that is a *function into* the recursive
type — and generate their **dependent eliminator** with the Π-abstracted
induction hypothesis (plus its ι-reduction). The motivating shape is L5's
interaction tree:

```
ITree.Vis : (e : E.Op) → (E.Resp e → ITree E R) → ITree E R
```

whose second argument `(E.Resp e → ITree E R)` is a branching/W-style recursive
occurrence. Per `14 §2` this **is** strictly positive (`D` appears only as the
*target* of a function type — `(Nat → D) → D` is the listed "allowed" case), so
it is sound; the current kernel simply **defers admitting it**:
`check_no_pi_bound_recursive` rejects the shape, and `elim`'s ι does not yet build
the Π-abstracted IH such a constructor needs.

## The framing that sets the risk level

This is the **trusted kernel**. K1.5 adds (a) a new class of *admitted* inductive,
(b) a new *eliminator* whose motive/method types carry a Π-abstracted induction
hypothesis, and (c) a new *ι-reduction* that fires that eliminator on the
W-style constructor — all in the TCB. ★★★, the K1/K2 bar. The soundness rests on
strict positivity (already in `14 §2`/§8.2), but the **eliminator + ι are new
machinery** and must be derived and checked with the full discipline: the
eliminator's method for a W-style constructor receives the IH **as a function**
`(λ resp. ih (k resp))` over the branching argument; the ι-rule must apply it
correctly; and an adversarial test must confirm a **non-positive** sibling (e.g.
`(D → Bool) → D`) is still **rejected** (the guard must not over-open). Exercise
the property: a W-style elim that **uses** the Π-abstracted IH (not β-discards
it), at ≥2 levels, with a dependent motive.

## Scope

**IN:**
- **Admission:** replace the blanket `check_no_pi_bound_recursive` rejection with
  the precise strict-positivity test of `14 §2` — accept a recursive occurrence
  that is the **target** of a (dependent) function-typed argument; keep rejecting
  **negative** occurrences (`D` left of an arrow).
- **Eliminator generation:** the dependent eliminator for a W-style inductive —
  motive + per-constructor methods where a W-style argument `k : B → D`
  contributes a Π-abstracted IH `(b : B) → Motive (k b)`.
- **ι-reduction:** `elim` on a W-style constructor reduces to the method applied
  with the recursive results threaded through the branching function; integrate
  with the K2c conversion/normalization path so it computes for conversion.
- **`elim_ITree`** generated for L5's `ITree` as the concrete first client.
- Reconcile the `14 §2` erratum: its "allowed" prose is currently **stale** vs the
  on-`main` kernel (which rejects); once K1.5 lands, "allowed" is true again —
  land the `14 §2` wording (and remove the K1.5-deferral note) as part of this WP.

**OUT — do not build here:**
- General **coinduction** / guarded corecursion (`OQ-coinduction`, deferred) — K1.5
  is *inductive* W-style, finite trees, not infinite ones.
- The L5 **denotation/handlers** themselves (Team Language, once `elim_ITree`
  exists) and X1's **effect evaluation** (post-L5).
- Any relaxation of strict positivity — negative occurrences stay rejected.

## The elaboration this needs (spec-leader → spec-author + Architect)

Extend `14-inductive.md`: the exact admission predicate (positive Π-bound
occurrence vs. negative — pin the boundary), the **eliminator schema** for W-style
constructors (motive, method types with the Π-abstracted IH, the typing rule),
and the **ι-rule** with its soundness/termination argument (it must not threaten
the K2c decidability/SCT guarantee — eliminator-based recursion is total without
SCT per `14 §2`, but confirm the W-style ι terminates conversion). Conformance
(`conformance/kernel/inductive/`): a W-style inductive admitted + its elim
computing (discriminating: a correct ι vs. a mis-threaded IH reach different
results); a **negative** occurrence **rejected** (the adversarial guard); ≥2
levels; a dependent motive that uses the IH.

## Acceptance (testable)

1. **W-style admitted:** `ITree` (and a minimal `W`-type) type-check and an
   eliminator is generated; **negative** occurrences (`(D → Bool) → D`) are still
   **rejected**.
2. **Eliminator computes:** `elim` on a W-style constructor ι-reduces, threading
   the Π-abstracted IH; a dependent-motive elim that **uses** the IH evaluates
   correctly (not β-discarded).
3. **Conversion integrates:** the W-style ι participates in conversion/normal-
   ization and **decides** (halts) — no regression to the K2c SCT/decidability
   guarantee.
4. **No regression:** K1/K2/K2c suites stay green; the `14 §2` wording is
   reconciled ("allowed", K1.5-deferral note removed).
5. **L5 unblock:** `elim_ITree` exists so L5's `bind`/handlers/denotation become
   buildable (verified by Team Language picking up the denotation half).

## Sequencing

**Recommended priority: high** — after **X1-spec** (which closes the G1 vertical
slice with the merged V0 and needs only K1), K1.5 is the **effects-phase enabler**
(L5 denotation + X1 effects + Sec/B all gate on it). Independent of the obs-seam
completions (K2c series-2 → K-api), so those can interleave. Kernel is idle and
ready. Scope/boundary calls (the positivity predicate, the ι soundness argument)
→ Architect; behavioral contract → Spec.
