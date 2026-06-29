# X1 (reference interpreter) conformance — seed cases

Format: `../../README.md`. These pin the **reference interpreter** that **X1**
delivers (`docs/program/wp/X1-interpreter.md`,
`spec/40-runtime/42-evaluation.md`): evaluate **core terms** (`10-kernel/11`,
the elaborator's output) to **values** (`41`, the K3 content-addressed model),
realizing exactly the kernel's reductions (`17 §1`), **deterministically**, with
**canonicity** for closed ground computations, and `unknown` propagation.
**Pure-core (G1) scope** — effects are deferred (below). They extend — and must
not regress — the two on-`main` evaluation anchors in `../seed-runtime.md`
(`runtime/evaluation/canonicity`, `runtime/evaluation/unknown-propagates`).

**Trust posture.** X1 is **not in the TCB for type soundness** — it evaluates
already-kernel-checked core terms; it does not decide typing (`42 §5`, `frame`).
A bug here is a **wrong answer**, silently propagated to every backend validated
against the interpreter (★★, a notch below the kernel). So correctness is by
**agreement with the kernel's own reductions** (`17 §1`, `42 §1`) — the **CAN5
kernel-agreement** cases are the load-bearing oracle anchor — plus the
canonicity/determinism corpus, not a separate trust argument. **`(soundness)`**
tags the **canonicity** cases: a closed well-typed ground term *getting stuck*
would break the kernel's canonicity commitment (`16 §9`, `42 §3.6`) that X1 must
realize; those must never regress. (X1 cannot make the *kernel* unsound — it
runs post-check — but a stuck or divergent X1 fails the metatheoretic guarantee
end-to-end, so the cases carry the tag and the never-regress bar.)

**Tags.** `(oracle)` — confirmed at build time against Ken's interpreter (safe:
X1 not in the type-soundness TCB): **interpreter-internal** observations (the
evaluation *trace*, which subterms are interned, slot-id specifics), and — per
`42 §3.3`/`16 §9.1` — the `cast Type Type` non-`refl` reduction and certain
quotient-transport edge cases, which X1 **inherits as `(oracle)`** and does
**not** lock (ground from `16` + the conformance oracle at build time; the
AGPLv3 prototype is **not** mounted). `(property)` — an invariant over a corpus,
not a single trace.

**Effects are OUT (pure-core G1 seam, `42 §6`).** Primitive effects
(`FS`/`Net`/`Clock`/`Console`/`Rand`) and `space`/`becomes` mutable cells are
**out-of-scope stuck forms** — a `perform`/effect node and a `space` cell op are
deliberately **not reduced**, to be wired by the **L5 follow-on** (their
denotation rides `36`'s `ITree`, K1.5-gated, `36 §7.0`). **No effect-evaluation
case is authored here.** So CAN1's "no stuck closed ground term" is scoped to
the **pure fragment**: an effect node legitimately sticks (the deferred seam),
not a canonicity violation. Two deferred strengthenings are flagged at their
cases: branch-laziness becomes *value-observable* only once an effect (L5)
**or** an opaque-non-total divergent branch (`42 §3.3` escape hatch) can sit in
the untaken arm; and the `Lazy a` thunk's force/memo lands only if `41`/`42` pin
it (`42 §2`).

**Reconcile note (content-verified against landed `42` §1–§7, `ef7d55d`).**
Authored in parallel with spec-author, then reconciled against the *bodies* (not
headings/the draft — the L5 `§2.1` lesson,
`conformance-oracle-grounding-fallback`). Findings folded: (a) branch laziness
is **the eliminator's methods held unevaluated** — `if`/`match`/`&&`/`||` all
elaborate to `elim_D`, and only the scrutinee-selected method fires (ι); it is
**not** a special rule (`42 §2`, `§3.3`), so CAN3 cites that mechanism and its
bug ("methods evaluated strictly before selection"); (b) dedup is asserted by
**slot identity**, not `==` (`42 §3.4`, `§3.7`); (c) the `unknown` absorbing
connectives **are** the untaken-eliminator-arm rule (`42 §4`); (d) functions
evaluate to **closures** (WHNF), data to **full** normal form, and
kernel-agreement on functions is **up to η at compare time** (`42 §3.5`); (e)
**level-reconcile is N/A** — evaluation carries the kernel's explicit levels and
forms no new types (`42 §3.5`; spec-leader's trivial-PASS). Expected values are
grounded in the landed `42` + the **K2 `seed-observational` locked reductions**
(reused — kernel-agreement means X1 reaches the value the kernel already
computes) + `17 §1`/`16 §9`.

**Citations.** `42-evaluation.md` §1 (reduction table: β/Σ-β/ι/δ/prim/obs, the
kernel-agreement source), §2 (CBV-with-sharing; the eliminator-branch non-strict
position), §3.2 (`eval`/`apply`), §3.3 (per-form reduction; obs with the
C-numbers), §3.4 (sharing + dedup, O(1) slot equality), §3.5 (WHNF-vs-full
boundary, η-at-compare, levels carried), §3.6 (canonicity), §3.7 (determinism),
§4 (`unknown` propagation), §6 (effects deferred / stuck forms). Kernel: `17 §1`
(β, Σ-projection, ι, δ, obs, prim), `16 §2.2`/`§3.2`/`§5`/`§6` (observational
computations, C2–C6/C9/C10), `16 §9.1` (the `(oracle)` cast/transport edges),
`14 §5` (audited prim). The **K2 locked reductions**
`kernel/observational/seed-observational.md` (`cast-refl`, `cast-computes-*`,
`quotient-eq`, `quotient-elim`, `eq-inductive-*`). Heap:
`41 §2`/`§3a`/`§4`/`§6`. V0: `surface/elaboration/seed-elaboration.md` (the G1
elaboration X1 runs).

---

## CAN1 — canonicity: closed ground terms compute, none get stuck (frame AC1)

A closed, well-typed, **ground** term evaluates to a **value**, never a stuck
neutral (`42 §3.6`): an inductive → a **constructor form** (data goes to
**full** normal form, recursively, `42 §3.5`); the closed observational
computations compute. These realize the kernel's canonicity commitment, so
`(soundness)`. (Scoped to the **pure fragment** — effect nodes are the intended
deferred stuck forms, `42 §6`.)

### runtime/evaluation/can-closed-inductive-to-constructor (soundness)
- spec: `42 §3.6`, `§1` (ι, δ, β, prim table); `14 §5` (prim)
- given: a closed `Nat` computation, e.g. `add 2 3` (δ-unfold `add`, ι on the
  `Nat` eliminator, or `prim` where `Nat`/`Int` is primitive, `42 §3.3`).
- expect: **reduces-to** the constructor form `5`
  (`suc (suc (suc (suc (suc zero))))`, or the `Int`-immediate `5` if primitive —
  exact representation `(oracle)`, `41 §1`); **fully** evaluated (`42 §3.5`),
  **not** stuck/neutral.
- why: the canonicity headline — a closed inductive computation reaches a
  constructor. A bug that leaves the eliminator neutral (un-fired ι) yields a
  stuck term, not `5` — the verdict flips value-vs-stuck. (soundness; AC1.)

### runtime/evaluation/can-cast-refl-to-value (soundness)
- spec: `42 §3.3`/`§3.6` (C5), `16 §3.2`; anchor `seed-observational`
  `cast-refl`
- given: `A : Type 0`, `a : A` closed; `cast A A refl a`.
- expect: **reduces-to** `a` — exactly the value the kernel's `cast-refl`
  reduction locks (**regularity**: cast on a reflexive type equality is the
  identity).
- why: **kernel-agreement on a `(soundness)` reduction** — X1 must reach the
  K2-locked value `a`, not a re-wrapped `cast … a`. A bug that fails cast-refl
  regularity leaves `cast A A refl a` stuck/neutral — flips vs `a`. (soundness.)

### runtime/evaluation/can-eq-by-type-computes (soundness)
- spec: `42 §3.3` (C2–C4, C9), `16 §2.2`/`§5`; anchors `seed-observational`
  `eq-inductive-same-ctor`, `quotient-eq`
- given: closed `Eq`-by-type computations: (a) `Eq Bool true true`; (b)
  `Eq (A/R) [a] [b]` for a quotient `A/R`.
- expect: (a) **reduces-to** the kernel value for same-constructor `Eq` (the
  conjunction of field equalities, here trivial — exact form `(oracle)`,
  anchored to the locked `eq-inductive-same-ctor`; `Eq` lands in `Ω`,
  proof-irrelevant at the value layer, `42 §3.3`); (b) **reduces-to** `R a b`
  (`quotient-eq`: quotient equality *is* the user relation).
- why: `Eq`-by-type computes on closed terms (no setoid boilerplate) — X1 agrees
  with the kernel's locked values. A bug that leaves `Eq …` neutral flips
  value-vs-stuck. (soundness; AC1 "`Eq`-by-type → its computed value".)

### runtime/evaluation/can-quotient-elim-computes (soundness)
- spec: `42 §3.3` (C9), `16 §5`; anchor `seed-observational` `quotient-elim`
- given: `M : (z : A/R) → Type 0`, `f : (x:A) → M [x]` respecting `R`, closed
  `a : A`; the quotient eliminator applied to the class `[a]`.
- expect: **reduces-to** `f a` (`elim_/ M f r [a] → f a` — the eliminator on a
  canonical class computes the representative branch) — the kernel's locked
  `quotient-elim` value.
- why: quotient elimination computes on a closed class. A bug that leaves the
  eliminator stuck on `[a]` flips vs `f a`. (soundness; AC1.)

### runtime/evaluation/can-no-stuck-closed-ground (soundness, property)
- spec: `42 §3.6` ("a closed well-typed ground term evaluates to a value, never
  a stuck neutral"); `42 §6` (effect nodes excepted)
- given: a corpus of closed **pure** ground terms mixing β, Σ-projection, ι, δ,
  and a prim (e.g. `fst (pair (add 1 1) true)`, `(\x. x) ((\y. y) 0)`).
- expect: **each reduces-to a value** (constructor form / immediate); **no**
  sub-term remains neutral/stuck. The **only** marked non-value outcomes are
  `unknown` (open hole, CAN4) and an opt-in opaque-non-total divergence
  (`42 §3.3`) — both listed; an **effect node** is the deferred out-of-scope
  stuck form (`42 §6`), not a canonicity failure.
- why: the canonicity *property* over the reduction forms, not one
  representative (COORDINATION §7). A bug stuck on any one reduction (a missing
  δ-unfold, an un-projected Σ) is caught by the corpus member that exercises it.
  (soundness; property.)

---

## CAN2 — determinism + sharing by slot identity (frame AC2)

Evaluation of a closed term is a **function** (same term → same value,
`42 §3.7`); results are **shared** via the content-addressed heap — equal
subcomputations dedup to the **same slot id** (`42 §3.4`), making `==` an O(1)
slot compare (`41 §4`). AC2 asserts **slot identity**, not just `==`
(`42 §3.7`).

### runtime/evaluation/det-same-term-same-value (property)
- spec: `42 §3.7` (determinism)
- given: the same closed term evaluated twice (independent runs).
- expect: **identical value** — same constructor form and, for a compound, the
  **same slot id** (`42 §3.7`, `41 §4`).
- why: determinism is what makes X1 a usable oracle. A non-deterministic
  evaluator (e.g. iteration-order-dependent) flips the second run's value/slot.
  (property; the AC2 baseline.)

### runtime/evaluation/det-sharing-dedups-by-slot (oracle)
- spec: `42 §3.4` (representation sharing / dedup), `41 §2`/`§4`; extends
  `runtime/values/dedup-shares-slot`
- given: a closed term producing the **same** compound value by two independent
  subcomputations, e.g. `pair (big_expr) (big_expr)`.
- expect: both components intern to the **same slot id** (stored once); `==` is
  **true** by O(1) slot compare, with **no recomputation** of the shared value
  (`(oracle)` on the trace). **Assert slot identity, not just `==`**
  (`42 §3.7`).
- why: content-addressed sharing at the eval layer — equal results share a slot.
  A recompute-without-dedup bug yields an equal value at a *different* slot
  (`42 §3.4`); the slot-identity assertion catches it where a bare `==` would
  pass vacuously. (oracle; extends the values anchor.)

### runtime/evaluation/det-canonical-order-independent (oracle)
- spec: `41 §3a` (`Map`/`Set` canonical order), `§4`; `42 §3.7`
- given: a `Map` (or `Set`) value produced by two evaluation paths with
  **different insertion orders**, e.g. `{1↦a, 2↦b}` built insert-1-then-2 vs
  insert-2-then-1.
- expect: the **same slot id** → `==` is **true** — the canonical encoding sorts
  entries by key bytes (`41 §3a`), so construction order is invisible.
- why: **verdict-flip** (the `42 §3.7` "shared vs recompute-divergence"): a
  canonicalization bug that encodes in *insertion* order gives two slots for
  equal `Map`s → `==` flips to **false** (and dedup silently fails). Correct →
  `true`, bug → `false`. (The `values/` corpus owns encoding-determinism proper;
  this pins the evaluation consequence: equal results share regardless of how
  evaluation built them.) (oracle; verdict-flip.)

---

## CAN3 — branch laziness = the eliminator fires one method (frame AC3)

`if`/`match`/`&&`/`||` all elaborate to `elim_D` (`14 §3`, `34`); the **one**
non-strict position is an eliminator's **methods** — `elimReduce` forces the
scrutinee (CBV), then evaluates **only** the method the head constructor selects
(ι), discarding the others **unevaluated** (`42 §2`, `§3.3`). "Evaluate only the
taken arm" and short-circuit **fall out of ι**, not a special rule.

**Honesty note (why these are structural, not value-flips).** Ken's pure core is
**total** (`17 §4`): an untaken method cannot diverge, and — effects being
**out** (`42 §6`) — has no side effect to skip, so forcing it would waste work
but **change no observable value** (`42 §3.6`: a branch is selected away from,
so even a would-be-`unknown` arm does not contaminate the result). These cases
therefore assert the **structural/trace** property — the untaken method is **not
evaluated** (its subterms never reach the interner) — tagged `(oracle)`.
**Deferred strengthening:** the result becomes value-observable only once an
**effect** (L5 follow-on) **or** an **opaque-non-total divergent** branch
(`42 §3.3` escape hatch) sits in the untaken arm; flagged, not forced here.

### runtime/evaluation/lazy-if-taken-arm-only (oracle)
- spec: `42 §2`/`§3.3` (ι fires one method; others held unevaluated)
- given: `if true then x else Y` (≡ `elim_Bool _ x Y true`), where `Y`
  constructs a distinct compound value (e.g. `pair (big_a) (big_b)`).
- expect: **reduces-to** `x`; the `else` method `Y` and its subterms are **not
  evaluated** — not interned, absent from the evaluation trace (`(oracle)`).
- why: branch laziness as a structural assertion. The targeted bug — "methods
  evaluated strictly *before* selection" (`42 §2`) — would intern `Y`'s value
  (an extra slot / trace entry) while still returning `x`, so the *value* is
  unchanged but the **structural** assertion catches it. (oracle; AC3.)

### runtime/evaluation/lazy-match-taken-branch-only (oracle)
- spec: `42 §2`/`§3.3`
- given: `match (inl a) { inl x → x ; inr y → Y }` (≡ `elim_Sum`), `Y` a
  distinct compound.
- expect: **reduces-to** `a`; the `inr` method `Y` is **not evaluated** (not
  interned / not in the trace).
- why: only the constructor-selected method evaluates (ι). A strict-all-methods
  bug interns `Y` — caught structurally. (oracle; AC3.)

### runtime/evaluation/shortcircuit-and-or (oracle)
- spec: `42 §2`/`§3.3` (`&&`/`||` are `elim_Bool`)
- given: `false && Y` and `true || Y`, `Y` a distinct compound.
- expect: `false && Y` **reduces-to** `false`, `true || Y` **reduces-to**
  `true`; in both, `Y` is **not evaluated** (not interned / not in the trace).
- why: short-circuit is the `elim_Bool` scrutinee selecting the determined
  method (`42 §2`). (Note: a Kleene `∧`/`∨` over `unknown` does *not*
  discriminate strict-vs-short — `false ∧ x = false` either way, `41 §6` — so
  the structural trace is the only probe in the pure fragment; see CAN4 for the
  `unknown` rows.) (oracle; AC3.)

---

## CAN4 — `unknown` propagation (frame AC4)

A term depending on an **open verification hole** yields `unknown` (`42 §4`,
`41 §6`); a **hole-free** term **never** does. The absorbing connectives are the
untaken-eliminator-arm rule (CAN3) in disguise (`42 §4`). Extends
`runtime/evaluation/unknown-propagates`.

### runtime/evaluation/unknown-from-hole-dependent (oracle)
- spec: `42 §4` (`hole h → unknown`), `41 §6`; extends
  `runtime/evaluation/unknown-propagates`
- given: a value whose computation depends on an **open hole** (`24 §2`) — e.g.
  a result guarded by an unproven proposition.
- expect: evaluates to **`unknown`**; the program **runs** (does not fail
  closed), with `unknown` marking exactly where the gap bites.
- why: the operational face of partial verification. A bug that fails closed
  (errors instead of `unknown`) or substitutes a default is caught. (oracle;
  AC4.)

### runtime/evaluation/unknown-strict-and-kleene-table (oracle)
- spec: `42 §4`, `41 §6`
- given: `unknown` in strict-elimination positions and across the connectives.
- expect: **strict elimination on `unknown` → `unknown`**: `apply unknown u`,
  `elimReduce … unknown` (an `unknown` *scrutinee*),
  `primReduce op (… unknown …)`, `cast`/`Eq` on `unknown`, `fst`/`snd` of
  `unknown` — all `unknown` (`42 §4`). **Absorbing connectives**:
  `unknown ∧ false = false`, `unknown ∨ true = true`; **propagating**:
  `unknown ∧ true = unknown`, `unknown ∨ false = unknown`,
  `¬ unknown = unknown`.
- why: each row pinned independently (COORDINATION §7) — the *annihilator* rows
  (`∧false`/`∨true`) resolve despite `unknown` (the `elim_Bool` scrutinee is the
  *known* operand selecting the absorbing method, `42 §4`), the others propagate
  (`unknown` scrutinee). A bug that propagates `unknown` through an annihilator
  (`unknown ∧ false → unknown`), or fails to propagate through a strict
  position, is caught by that exact row. (oracle; AC4 table.)

### runtime/evaluation/unknown-absent-when-hole-free (oracle)
- spec: `42 §4` ("a hole-free program never yields `unknown`"), `41 §6`
- given: **one term shape**, two instantiations: (a) a proposition left as an
  **open hole**; (b) the **same** proposition **discharged by a proof**.
- expect: (a) → **`unknown`**; (b) → the **concrete value** (the proven branch's
  result), **never `unknown`**.
- why: **the verdict-flip** (`42 §4` "hole-present vs hole-absent"), catching
  **both** error directions: over-approximation (a bug emitting `unknown` for
  the hole-free (b) is caught by (b)'s concrete expectation) and
  under-propagation (a bug giving a default for the holed (a) is caught by (a)'s
  `unknown`). (oracle; verdict-flip.)

---

## CAN5 — kernel agreement + G1 end-to-end (frame AC5, AC6)

The interpreter is "the kernel's evaluator, run to completion" (`42 §1`); for a
closed-term corpus its value **matches the kernel's own reduction** — on
**data** exactly, on **functions** up to the kernel's **η** at compare time
(`42 §3.5`). The G1 slice runs V0-elaborated core through X1.

### runtime/evaluation/agree-with-kernel-reduction (property)
- spec: `42 §1` (reduction table), `§3.5` (η-at-compare); `17 §1`
- given: a corpus of closed terms, one per reduction form (a β-redex, a
  Σ-projection, an ι on a constructor, a δ-unfold, a prim).
- expect: X1's value **equals** the kernel's reduction result for each — the
  interpreter is WHNF-run-to-completion to the full value on **data**; a
  **function** stops at a closure `⟨λ ; ρ⟩`, agreeing with the kernel **up to
  η** (`42 §3.5`), which the interpreter need not implement (it produces values;
  the kernel compares them).
- why: the **oracle invariant** — disagreement means X1 is wrong by definition
  (`frame Objective`). A bug in any reduction form flips that corpus member's
  value away from the kernel's. (property; AC5.)

### runtime/evaluation/agree-observational-corpus (soundness, property)
- spec: `16 §2.2`/`§3.2`/`§5`/`§6` (C2–C6, C9, C10); anchors
  `seed-observational` `cast-refl`,
  `cast-computes-pi`/`-sigma`/`-inductive`/`-quotient`, `quotient-eq`,
  `quotient-elim`, `eq-inductive-same-ctor`/`-diff-ctor`
- given: the **closed observational** terms whose kernel reductions K2 locked.
- expect: X1 evaluates **each** to the **same value** the K2 seed locked (e.g.
  `cast A A refl a → a` C5; `Eq (A/R) [a] [b] → R a b`; `cast`-by-type at an
  inductive → the constructor form with recursive casts C6). **Exception —
  `(oracle)`:** the `cast Type Type` non-`refl` reduction and certain
  quotient-transport edges (`16 §9.1`, `42 §3.3`) are **not locked** — X1
  inherits the tag and realizes whatever `16` settles, grounded from `16` + the
  build-time oracle (yon not mounted).
- why: the load-bearing **kernel-agreement on the `(soundness)` observational
  reductions** — X1 reuses the kernel's locked values, never a divergent one. A
  bug evaluating any observational form differently from the kernel is a silent
  oracle corruption. (soundness; property; AC5.)

### runtime/evaluation/g1-end-to-end (property)
- spec: `frame AC6`; `surface/elaboration/seed-elaboration.md` (V0), `42 §1`
- given: a surface program elaborated by **V0** to core, then run by **X1** —
  e.g. `(\ x . add x 1) 2` (V0 elaborates to the core `App`/`Lam`; X1 evaluates
  via β + prim).
- expect: the value **`3`** (a constructor/immediate) — the vertical slice
  closes: V0 (surface → core) ∘ X1 (core → value) = the expected result.
- why: **G1 end-to-end** — the V0 elaboration anchor feeds X1, and the composed
  pipeline produces the expected value. A bug in either stage (mis-elaboration,
  or mis-evaluation) flips the end value. (property; AC6, the G1 close.)

---

## Regression — X1 extends the on-`main` runtime anchors

### runtime/evaluation/existing-anchors-still-green (property)
- spec: `../seed-runtime.md` (`runtime/evaluation/canonicity`,
  `runtime/evaluation/unknown-propagates`)
- given: the two on-`main` evaluation invariants.
- expect: **unchanged** — X1's granular CAN1 (canonicity) and CAN4 (`unknown`)
  cases **refine** these anchors; they must continue to hold. Pure-core: no
  effect case is added that would alter the existing pure expectations.
- why: X1 conformance is **additive** over the seed-runtime anchors. Pins that
  the granular cases extend, never contradict, the merged invariants. (property;
  regression guard.)
