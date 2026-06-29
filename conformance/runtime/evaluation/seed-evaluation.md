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
already-kernel-checked core terms; it does not decide typing
(`frame "the framing that sets the risk level"`). A bug here is a **wrong
answer**, silently propagated to every backend validated against the interpreter
(★★, a notch below the kernel). So correctness is by **agreement with the
kernel's own reductions** (`17 §1`, `42 §1`) — the **CAN5 kernel-agreement**
cases are the load-bearing oracle anchor — plus the canonicity/determinism
corpus, not a separate trust argument. **`(soundness)`** tags the **canonicity**
cases: a closed well-typed ground term *getting stuck* would break the kernel's
canonicity commitment (`16 §9`, `10-kernel/README §5`) that X1 must realize;
those must never regress. (X1 cannot make the *kernel* unsound — it runs
post-check — but a stuck or divergent X1 fails the metatheoretic guarantee
end-to-end, so the cases carry the tag and the never-regress bar.)

**Tags.** `(oracle)` — confirmed at build time against Ken's interpreter (safe:
X1 not in the type-soundness TCB): **interpreter-internal** observations (the
evaluation *trace*, which subterms are interned, slot-id specifics), the exact
heap dedup behavior, and any free strategy detail `42` leaves as latitude (`42`
is *normative for what evaluation computes*, the strategy is implementation
latitude `§2`). `(property)` — an invariant over a corpus, not a single trace.

**Effects are OUT (pure-core G1 seam).** `42 §3` (primitive effects
`FS`/`Net`/`Clock`/`Console`/`Rand`; `space`/`becomes` mutable cells) has its
operational meaning **deferred to the L5 follow-on** — its denotation rides
`36`/`ITree`, itself K1.5-gated. **No effect-evaluation case is authored here**;
the G1 interpreter is the pure, effect-free fragment (`frame Scope OUT`). Two
deferred strengthenings are flagged at their cases: branch-laziness becomes
*value-observable* once an effect can sit in the untaken arm (CAN3), and the
`Lazy a` thunk's force/memo semantics lands only if `41`/`42` pin it
(`frame Scope OUT`).

**Reconcile note.** Authored in parallel with spec-author's `42` elaboration
(`wp/X1-spec`). Expected values are grounded in the **current** normative `42`
(*what evaluation computes*) + the **elaborated** `41` + the **K2
`seed-observational` locked reductions** (reused, not re-derived —
kernel-agreement means X1 must reach the value the kernel already computes) +
`17 §1`. Before the merge Decision I run a **content-verified reconcile**
against the landed `42` §-bodies (not headings/this draft — the L5 `§2.1`
`perform`-vs-`Vis` lesson, `conformance-oracle-grounding-fallback`).

**Citations.** `42-evaluation.md` §1 (kernel-reduction agreement, determinism,
canonicity), §2 (CBV-with-sharing, branch laziness / short-circuit), §4
(`unknown` at runtime); `41-values.md` §2 (content-addressed heap, dedup), §3a
(canonical byte encoding — `Map`/`Set` order-independence), §4 (O(1) structural
equality = slot-id compare), §6 (`unknown` Kleene/Heyting). Kernel: `17 §1` (β,
Σ-projection, ι, δ, observational, prim), `16 §9` (observational computations),
`14 §5` (audited prim reductions); the **K2 locked reductions**
`kernel/observational/seed-observational.md` (`cast-refl`, `cast-computes-*`,
`quotient-eq`, `quotient-elim`, `eq-inductive-*`). V0:
`surface/elaboration/seed-elaboration.md` (the G1 elaboration X1 runs).

---

## CAN1 — canonicity: closed ground terms compute, none get stuck (frame AC1)

A closed term of an inductive type evaluates to a **constructor form**; closed
`Eq`/`cast`/quotient computations compute to their kernel values; **no closed
well-typed ground term gets stuck** (`42 §1`, `16 §9`). These realize the
kernel's canonicity commitment, so `(soundness)`.

### runtime/evaluation/can-closed-inductive-to-constructor (soundness)
- spec: `42 §1`, `17 §1` (ι, δ, β); `14 §5` (prim)
- given: a closed `Nat` computation, e.g. `add 2 3` (δ-unfold `add`, ι on the
  `Nat` eliminator, prim where `Nat` is primitive).
- expect: **reduces-to** the constructor form `5`
  (`suc (suc (suc (suc (suc zero))))`, or the `Int`-immediate `5` if `Nat`/`Int`
  is a prim — exact representation `(oracle)`, `41 §1`). **Not stuck**, not a
  neutral.
- why: the canonicity headline — a closed inductive computation reaches a
  constructor. A bug that leaves the eliminator neutral (un-fired ι) yields a
  stuck term, not `5` — the verdict flips value-vs-stuck. (soundness; AC1.)

### runtime/evaluation/can-cast-refl-to-value (soundness)
- spec: `42 §1`, `16 §9`; anchor `kernel/observational/seed-observational.md`
  `cast-refl`
- given: `A : Type 0`, `a : A` closed; `cast A A refl a`.
- expect: **reduces-to** `a` — exactly the value the kernel's `cast-refl`
  reduction locks (regularity: cast on a reflexive type equality is the
  identity).
- why: **kernel-agreement on a `(soundness)` reduction** — X1 must reach the K2-
  locked value `a`, not a re-wrapped `cast … a`. A bug that fails cast-refl
  regularity leaves `cast A A refl a` stuck/neutral — flips vs `a`. (soundness.)

### runtime/evaluation/can-eq-by-type-computes (soundness)
- spec: `42 §1`, `16 §9`; anchors `seed-observational` `eq-inductive-same-ctor`,
  `quotient-eq`
- given: closed `Eq`-by-type computations: (a) `Eq Bool true true`; (b)
  `Eq (A/R) [a] [b]` for a quotient `A/R`.
- expect: (a) **reduces-to** the kernel value for same-constructor `Eq` (its
  computed `Unit`/component form, per `eq-inductive-same-ctor` — exact form
  `(oracle)`, anchored to the locked K2 case); (b) **reduces-to** `R a b`
  (`quotient-eq`: quotient equality *is* the user relation).
- why: `Eq`-by-type computes on closed terms (no setoid boilerplate) — X1 agrees
  with the kernel's locked values. A bug that leaves `Eq …` neutral flips
  value-vs-stuck. (soundness; AC1 "`Eq`-by-type → its computed value".)

### runtime/evaluation/can-quotient-elim-computes (soundness)
- spec: `42 §1`, `16 §9`; anchor `seed-observational` `quotient-elim`
- given: `M : (z : A/R) → Type 0`, `f : (x:A) → M [x]` respecting `R`, closed
  `a : A`; the quotient eliminator applied to the class `[a]`.
- expect: **reduces-to** `f a` (the eliminator on a canonical class computes to
  the representative branch) — the kernel's locked `quotient-elim` value.
- why: quotient elimination computes on a closed class. A bug that leaves the
  eliminator stuck on `[a]` flips vs `f a`. (soundness; AC1.)

### runtime/evaluation/can-no-stuck-closed-ground (soundness, property)
- spec: `42 §1` ("no closed program gets stuck on a well-typed ground
  computation")
- given: a corpus of closed ground terms mixing β, Σ-projection, ι, δ, and a
  prim (e.g. `fst (pair (add 1 1) true)`, `(\x. x) ((\y. y) 0)`).
- expect: **each reduces-to a value** (constructor form / immediate); **no**
  sub-term remains neutral/stuck.
- why: the canonicity *property* over the reduction forms, not one
  representative (COORDINATION §7). A bug stuck on any one reduction (a missing
  δ-unfold, an un-projected Σ) is caught by the corpus member that exercises it.
  (soundness; property.)

---

## CAN2 — determinism + sharing (frame AC2)

Evaluation of a closed term is a **function** (same term → same value, `42 §1`);
results are **shared** via the content-addressed heap — equal subcomputations
deduplicate to one slot (`41 §2`), making `==` an O(1) slot compare (`41 §4`).

### runtime/evaluation/det-same-term-same-value (property)
- spec: `42 §1` (determinism)
- given: the same closed term evaluated twice (independent runs).
- expect: **identical value** — same constructor form and the **same slot id**
  for the compound result (`41 §4`).
- why: determinism is what makes X1 a usable oracle. A non-deterministic
  evaluator (e.g. iteration-order-dependent) flips the second run's value/slot.
  (property; the AC2 baseline.)

### runtime/evaluation/det-sharing-dedups-equal-subcomputations (oracle)
- spec: `41 §2` (global dedup), `§4` (O(1) equality); extends
  `runtime/values/dedup-shares-slot`
- given: a closed term producing the **same** compound value by two independent
  subcomputations, e.g. `pair (big_expr) (big_expr)` where both components
  evaluate to equal content.
- expect: both components resolve to the **same slot** (interned once); `==` on
  them is **true** by O(1) slot compare, with **no recomputation** of the shared
  value (a single intern, `(oracle)` on the trace).
- why: the evaluation-level manifestation of content-addressed sharing — equal
  results share a slot, not just compare equal. A bug that stores duplicates (no
  intern-on-hit) breaks dedup; caught by the single-slot assertion. (oracle;
  extends the values anchor at the eval layer.)

### runtime/evaluation/det-canonical-order-independent (oracle)
- spec: `41 §3a` (`Map`/`Set` canonical order), `§4`; `42 §1`
- given: a `Map` (or `Set`) value produced by two evaluation paths with
  **different insertion orders**, e.g. `{1↦a, 2↦b}` built insert-1-then-2 vs
  insert-2-then-1.
- expect: **same slot** → `==` is **true** — the canonical encoding sorts
  entries by key bytes (`41 §3a`), so construction order is invisible.
- why: **verdict-flip** (the frame's "correct-shared vs recompute-divergence"):
  a canonicalization bug that encodes in *insertion* order gives two slots for
  equal `Map`s → `==` flips to **false** (and dedup silently fails). Correct →
  `true`, bug → `false`. (The `values/` corpus owns encoding-determinism proper;
  this pins the evaluation consequence: equal results share regardless of how
  evaluation built them.) (oracle; verdict-flip.)

---

## CAN3 — branch laziness / short-circuit (frame AC3)

`if`/`match` evaluate the scrutinee then **only the taken arm**; `&&`/`||`
**short-circuit** (`42 §2`).

**Honesty note (why these are structural, not value-flips).** Ken's pure core is
**total** (`17 §4`): the untaken arm cannot diverge, and — effects being **out**
of G1 — it has no side effect to skip. So forcing it would waste work but
**change no observable value** (it computes and discards). These cases therefore
assert a **structural/trace** property — the untaken arm is **not evaluated**
(its subterms never reach the interner / the trace) — tagged `(oracle)` as an
interpreter-internal observation. **Deferred strengthening (L5 follow-on):**
once an effect can sit in the untaken arm, "the effect did not fire" becomes a
*value/world*-observable verdict-flip; flagged, not forced here.

### runtime/evaluation/lazy-if-taken-arm-only (oracle)
- spec: `42 §2` ("`if`/`match` evaluate only the taken arm")
- given: `if true then x else Y`, where `Y` constructs a distinct compound value
  (e.g. `pair (big_a) (big_b)`).
- expect: **reduces-to** `x`; `Y` and its subterms are **not evaluated** — not
  interned, absent from the evaluation trace (`(oracle)`).
- why: branch laziness as a structural assertion. A strict-both-arms bug would
  intern `Y`'s value (an extra slot / trace entry) while still returning `x` —
  so the *value* is unchanged but the **structural** assertion catches it.
  (oracle; AC3.)

### runtime/evaluation/lazy-match-taken-branch-only (oracle)
- spec: `42 §2`
- given: `match (inl a) { inl x → x ; inr y → Y }`, `Y` a distinct compound.
- expect: **reduces-to** `a`; the `inr` branch `Y` is **not evaluated** (not
  interned / not in the trace).
- why: only the matched branch evaluates. A bug evaluating all branches interns
  `Y` — caught structurally. (oracle; AC3.)

### runtime/evaluation/shortcircuit-and-or (oracle)
- spec: `42 §2` ("`&&`/`||` short-circuit")
- given: `false && Y` and `true || Y`, `Y` a distinct compound.
- expect: `false && Y` **reduces-to** `false`, `true || Y` **reduces-to**
  `true`; in both, `Y` is **not evaluated** (not interned / not in the trace).
- why: short-circuit skips the determined operand. A strict bug evaluating `Y`
  interns it — caught structurally. (Note: a Kleene `∧`/`∨` over `unknown` does
  *not* discriminate strict-vs-short here — `false ∧ x = false` either way,
  `41 §6` — so the structural trace is the only probe in the pure fragment.)
  (oracle; AC3.)

---

## CAN4 — `unknown` propagation (frame AC4)

A term depending on an **open verification hole** yields `unknown`, propagated
by the Kleene/Heyting rules (`41 §6`, `42 §4`); a **hole-free** term **never**
yields `unknown`. Extends `runtime/evaluation/unknown-propagates`.

### runtime/evaluation/unknown-from-hole-dependent (oracle)
- spec: `42 §4`, `41 §6`; extends `runtime/evaluation/unknown-propagates`
- given: a value whose computation depends on an **open hole** (`24 §2`) — e.g.
  a branch guarded by an unproven proposition.
- expect: evaluates to **`unknown`**; the program **runs** (does not fail
  closed), with `unknown` marking exactly where the gap bites.
- why: the operational face of partial verification. A bug that fails closed
  (errors instead of `unknown`) or substitutes a default is caught. (oracle;
  AC4.)

### runtime/evaluation/unknown-kleene-table (oracle)
- spec: `41 §6` (Kleene/Heyting)
- given: `unknown` combined with concrete operands across the connectives.
- expect: `unknown ∧ false = false`; `unknown ∨ true = true`;
  `unknown ∧ true = unknown`; `unknown ∨ false = unknown`; a **strict** operator
  on an `unknown` operand = `unknown`.
- why: each row pinned independently (COORDINATION §7) — the *annihilator* rows
  (`∧ false`, `∨ true`) resolve to a concrete value despite `unknown`, the
  others propagate. A bug that propagates `unknown` through an annihilator
  (`unknown ∧ false → unknown`) is caught by that exact row. (oracle; AC4
  table.)

### runtime/evaluation/unknown-absent-when-hole-free (oracle)
- spec: `42 §4` ("a hole-free program never yields `unknown`"), `41 §6`
- given: **one term shape**, two instantiations: (a) a proposition left as an
  **open hole**; (b) the **same** proposition **discharged by a proof**.
- expect: (a) → **`unknown`**; (b) → the **concrete value** (the proven branch's
  result), **never `unknown`**.
- why: **the verdict-flip** (frame AC4 "hole-present vs hole-absent"), catching
  **both** error directions: over-approximation (a bug emitting `unknown` for
  the hole-free (b) is caught by (b)'s concrete expectation) and
  under-propagation (a bug giving a default for the holed (a) is caught by (a)'s
  `unknown`). (oracle; verdict-flip.)

---

## CAN5 — kernel agreement + G1 end-to-end (frame AC5, AC6)

The interpreter is "the kernel's evaluator, run to completion" (`42 §1`); for a
closed-term corpus its value **matches the kernel's own reduction**. The G1
slice runs V0-elaborated core through X1.

### runtime/evaluation/agree-with-kernel-reduction (property)
- spec: `42 §1`, `17 §1` (β, Σ-projection, ι, δ, prim)
- given: a corpus of closed terms, one per reduction form (a β-redex, a
  Σ-projection, an ι on a constructor, a δ-unfold, a prim).
- expect: X1's value **equals** the kernel's reduction result for each — the
  interpreter is WHNF-run-to-completion to the full value (`42 §1`).
- why: the **oracle invariant** — disagreement means X1 is wrong by definition
  (`frame Objective`). A bug in any reduction form flips that corpus member's
  value away from the kernel's. (property; AC5.)

### runtime/evaluation/agree-observational-corpus (soundness, property)
- spec: `16 §9`; anchors `seed-observational` `cast-refl`, `cast-computes-pi`/
  `-sigma`/`-inductive`/`-quotient`, `quotient-eq`, `quotient-elim`,
  `eq-inductive-same-ctor`/`-diff-ctor`
- given: the **closed observational** terms whose kernel reductions K2 locked.
- expect: X1 evaluates **each** to the **same value** the K2 seed locked (e.g.
  `cast A A refl a → a`; `Eq (A/R) [a] [b] → R a b`; `cast`-by-type at an
  inductive → the constructor form with recursive casts).
- why: the load-bearing **kernel-agreement on the `(soundness)` observational
  reductions** — X1 reuses the kernel's locked values, never a divergent one. A
  bug evaluating any observational form differently from the kernel is a silent
  oracle corruption. (soundness; property; AC5.)

### runtime/evaluation/g1-end-to-end (property)
- spec: `frame AC6`; `surface/elaboration/seed-elaboration.md` (V0), `42 §1`
- given: a surface program elaborated by **V0** to core, then run by **X1** —
  e.g. `(\ x . add x 1) 2` (V0 elaborates to the core `App`/`Lam`; X1
  evaluates).
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
