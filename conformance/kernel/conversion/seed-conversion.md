# Kernel-2c (conversion) conformance — series-1 seed cases

Format: `../../README.md`. These pin the K2c **series-1**
(conversion-hardening) acceptance criteria
(`../../../docs/program/wp/K2c-conversion.md`, "Acceptance") and the soundness
commitments from `../../../spec/10-kernel/README.md §6` (#15–16 plus the
δ-termination / SCT gate). Series-1 scope is the type-directed conversion
algorithm hardened to `17 §3` and the **SCT admission gate** `17 §4`. The three
K2 obs-reduction completeness seams are **series 2** (`seed-obs-completion.md`,
NbE-dependent); the closing section here tags them deferred so a series-1 run
does not mistake them for gaps.

Cases tagged **(oracle)** are to be confirmed against the prototype's observed
behavior by the Spec enclave at build time (`../../../CLEAN-ROOM.md`); cases
tagged **(soundness)** encode a kernel soundness commitment
(`../../../spec/10-kernel/README.md §6`) and must never regress; a
**(property)** case is a meta-property checked by the harness over the whole
corpus, not a single input.

**Grounding note.** SCT admit/reject verdicts are grounded in the size-change
*principle* (`17 §4`), computed deterministically — **not** in the prototype,
which has no SCT gate (SCT is K2c's net-new headline). Observational expected
results re-use the locked reductions of `observational/seed-observational.md`.
The expected results below are determined by the on-`main` spec (`16`/`17`/`18`)
and the settled `(oracle)` size order (primitives neutral, `cast`-under-
recursion conservatively `?` — pinned by spec-leader + Architect, `17 §4`); none
required reading the prototype to author. Citations to `17-conversion.md` are
reconciled against spec-author's **landed** elaboration (`spec-author/work`
`cd3b19d`): §3.2 whnf, §3.3 conv, §3.4 convSpine, §3.5 δ-unfold trigger, §3.6
level equality `convLevel`; §4.1 sct_check, §4.2 size_change_matrix, §4.3 matrix
composition + idempotent closure. (`18-judgments.md` keeps §2 (Conv), §3
bidirectional, §4 API, §5 trusted base.)

---

## Acceptance criterion: SCT-accept (frame Acceptance item 1)

The K1 lesson is load-bearing here (frame "The K1 lesson"; COORDINATION §7):
exercise **lexicographic**, **mutual**, **non-first-parameter**, and
**permuted / position-moving** descent — **not** single-argument structural
recursion, which is the obvious case that hides a too-weak check. Each verdict
is the conformance claim; the per-edge size-change matrix entries are the
spec/impl's to compute and the Architect reviews them.

### conversion/sct-accept-lexicographic (soundness)
- spec: `spec/10-kernel/17-conversion.md §4.1`, `§4.2`
- given: transparent `ack : Nat -> Nat -> Nat` with
  `ack zero n = suc n`,
  `ack (suc m) zero = ack m (suc zero)`,
  `ack (suc m) (suc n) = ack m (ack (suc m) n)` (Ackermann); admit via
  `declare_def`
- expect: **admitted transparent** (SCT-accepted; δ-reducible)
- why: lexicographic descent on the pair `(m, n)` — the outer calls strictly
  decrease `m`; the inner call `ack (suc m) n` holds `m` (`↓=`) and strictly
  decreases `n`. No single fixed argument decreases in every recursive call,
  so a "structural recursion on one fixed argument" check **rejects** it; SCT's
  idempotent closure **accepts** because every idempotent loop has a strict
  decrease thread. Exercises ≥2 parameters and the non-first-parameter (`n`)
  descent.

### conversion/sct-accept-mutual (soundness)
- spec: `17 §4.1` (call graph: cross-definition edges)
- given: mutually-recursive transparent pair
  `isEven zero = true`, `isEven (suc n) = isOdd n`;
  `isOdd  zero = false`, `isOdd (suc n) = isEven n`; admit both
- expect: **admitted transparent** (both)
- why: the call graph has cross edges `isEven -> isOdd -> isEven`; the
  idempotent loop `isEven -> isOdd -> isEven` strictly decreases the single
  argument by two `suc`s (a `↓` composed across two steps,
  `isEven (suc (suc j)) -> isOdd (suc j) -> isEven j`). Exercises the
  cross-call-graph matrix composition a single-definition check never reaches.

### conversion/sct-accept-second-parameter (soundness)
- spec: `17 §4.2` (size-change matrix; descent off the first diagonal)
- given: transparent `g : Nat -> Nat -> Nat`,
  `g acc zero = acc`, `g acc (suc n) = g (suc acc) n` (accumulator); admit
- expect: **admitted transparent**
- why: the strictly-decreasing parameter is the **second** (`n`); the **first**
  (`acc`) strictly **grows** (`suc acc`). A first-argument-only structural check
  rejects (acc grows); SCT accepts because the idempotent self-loop has `↓` on
  the second parameter's diagonal. Directly exercises the frame's
  "idempotent-loop that decreases on a **non-first** parameter."

### conversion/sct-accept-permuted (soundness)
- spec: `17 §4.1`, `§4.3` (matrix composition across a position-moving thread)
- given: mutually-recursive `foo bar` where the descending value moves
  parameter position each step:
  `foo zero b = b`, `foo (suc a) b = bar b a`;
  `bar b zero = b`, `bar b (suc a) = foo a b`; admit both
- expect: **admitted transparent** (both)
- why: the strict-decrease thread on the `a`-value runs through **different
  parameter positions** — `foo.param1 -> bar.param2 -> foo.param1` — so any
  check that fixes the descending argument to one position fails. SCT composes
  the size-change matrices around the idempotent loop and finds the `↓` thread
  regardless of position. Exercises permuted descent and mutual composition
  together.

---

## Acceptance criterion: SCT-reject (frame Acceptance item 2)

Both reject cases below are genuinely non-terminating — rejecting them is the
"never loops" half of kernel totality. (SCT is **sound but incomplete**: it also
refuses some terminating definitions whose descent is not a per-parameter thread
— e.g. a function terminating only by an aggregate `m + n` measure. Those
sound-incomplete refusals are real but their exact verdicts depend on the size
order; exhaustive sound-incomplete cases are deferred to build-time when the
reference interpreter can ground them, rather than locked unground here.)

### conversion/sct-reject-self-loop (soundness)
- spec: `17 §4.1`, `§4.3`
- given: transparent `loop : Nat -> Nat`, `loop x = loop x`; admit
- expect: **rejected at admission** (`KernelError`; never admitted transparent)
- why: the self-loop's size-change matrix is `↓=` on the diagonal (the argument
  is unchanged), never `↓`. The idempotent closure has an idempotent loop with
  no strict decrease ⇒ reject. Admitting it would let conversion δ-unfold
  `loop x` forever — a non-termination / decidability break.

### conversion/sct-reject-growing (soundness)
- spec: `17 §4.2`, `§4.3`
- given: transparent `up : Nat -> Nat`, `up n = up (suc n)`; admit
- expect: **rejected at admission**
- why: the recursive argument `suc n` is strictly **larger** than `n` (no
  descent on the structural subterm order), so the idempotent loop has no `↓`
  thread ⇒ reject. Guards against a check that mistakes "the argument changed"
  for "the argument decreased."

### conversion/sct-reject-ctor-wrap (soundness)
- spec: `17 §4.2` (sizeRel: a constructor-wrap is `?`, never `↓=`)
- given: transparent `f : Nat -> Nat`, `f (suc x) = f (suc (suc x))` — the
  recursive argument re-wraps the parameter's field in another constructor;
  admit
- expect: **rejected at admission**
- why: the argument `suc (suc x)` wraps `suc x` in a constructor, so it is
  strictly **larger** — the size relation is `?` (unknown / not-≤), **never**
  `↓=`. `↓=` means structurally ≤ (identity or a non-growing
  projection/permutation); a constructor application grows and must be `?`. The
  self-loop has no `↓` thread ⇒ reject. (Architect blocker companion: the `↓=`
  class must exclude constructor-wrapping.)

### conversion/sct-reject-ctor-wrap-compose (soundness)
- spec: `17 §4.2` (sizeRel `↓=` vs `?`), `§4.3` (`compose(↓=, ↓) = ↓`)
- given: a mutually-recursive transparent pair where one edge **unwraps** (a
  real `↓`) and the other **re-wraps** in a constructor (which must be `?`):
  `p (suc x) = q x`; `q x = p (suc (suc x))`; admit both
- expect: **rejected at admission** — the pair is non-terminating
  (`p 1 → q 0 → p 2 → q 1 → p 3 → …`, the `p`-argument grows without bound)
- why: the **discriminating** case for the `↓=` rule. The `q -> p` edge
  re-wraps `x` as `suc (suc x)` (grows) and MUST be `?`; composed with the real
  `↓` on the `p -> q` unwrap edge, `compose(↓, ?) = ?`, so the idempotent loop
  has no `↓` ⇒ **reject** (correct). If an implementation mis-records the
  constructor re-wrap as `↓=`, then `compose(↓, ↓=) = ↓` fabricates a spurious
  decreasing thread and the **non-terminating** pair is wrongly **admitted** —
  the exact unsoundness the Architect flagged. The single-function
  `sct-reject-ctor-wrap` is rejected under either classification, so it does not
  isolate the bug; this composed pair does.

---

## Acceptance criterion: δ-heavy convertibility terminates (frame item 3)

### conversion/delta-termination (soundness)
- spec: `17 §3.5` (lazy-δ trigger), `§5` (decidability)
- given: `ack` admitted (see `sct-accept-lexicographic`); the closed query
  `convert(Nat, ack (suc^3 zero) (suc^3 zero), suc^61 zero)`
- expect: **convertible (true)** — `ack 3 3` δ-reduces to the numeral `61` and
  conversion halts with yes
- why: forces substantial controlled δ-unfolding of a recursive transparent
  definition; SCT (which admitted `ack`) guarantees the unfolding bottoms out,
  so the query **halts** with the correct yes. Decidability payoff on a δ-heavy
  convertibility. (Numeral: `ack 3 3 = 2^6 - 3 = 61`.)

### conversion/delta-unfold-heads-differ (soundness)
- spec: `17 §3.5` (unfold when heads differ AND ≥1 head is transparent)
- given: two **distinct** transparent constants `f : Nat -> Nat := λ x. suc x`
  and `h : Nat -> Nat := λ x. suc x`; open `a : Nat`;
  `convert(Nat, f a, h a)`
- expect: **convertible (true)** — heads differ (`Const f` vs `Const h`), at
  least one is transparent, so the trigger fires: unfold both to `λ x. suc x`,
  β-reduce to `suc a` vs `suc a`, convert
- why: exercises controlled δ with **≥2 distinct transparent constants** whose
  heads differ (frame K1-lesson). A checker that never unfolds reports these
  unequal (a completeness failure). The Architect's high-value review point.

### conversion/delta-stuck-open-same-head (soundness)
- spec: `17 §3.3` (conv: equal heads compare spines), `§3.4` (convSpine)
- given: `ack` admitted; open `a a' b : Nat` with `a ≢ a'`;
  `convert(Nat, ack a b, ack a' b)`
- expect: **not convertible (false)**, and the query **halts**
- why: both sides share the same transparent head `ack` but it is **stuck** on
  open `a`/`a'` (no constructor to match — neutral). Conversion compares the
  spines position-by-position, finds `a ≢ a'`, and returns false **without**
  eagerly δ-unfolding `ack` (which on open arguments cannot reduce and whose
  eager unfolding would diverge). The observable consequence of the lazy-δ
  discipline on equal heads: correct verdict + termination.

---

## Acceptance criterion: full η + proof irrelevance decide (frame item 4)

### conversion/pi-eta-open (soundness)
- spec: `17 §2`, `§3.3` (η at Π)
- given: `A : Type 0`, `B : A -> Type 1` (≥2 distinct universe levels); open
  `f : (x:A) -> B x`; `convert((x:A) -> B x, f, λ (x:A). f x)`
- expect: **convertible (true)**
- why: Π-η — the algorithm η-expands the neutral `f` under a fresh variable and
  compares `f x ≡ (λ x. f x) x` at `B x`. Open `f`, dependent codomain `B x`,
  ≥2 distinct levels (`Type 0`, `Type 1`). K1 lesson: open + ≥2 type/level
  variables, not a closed single-variable instance.

### conversion/sigma-eta-open (soundness)
- spec: `17 §2`, `§3.3` (η at Σ)
- given: `A : Type 0`, `B : A -> Type 0`; open `p : (x:A) × B x`;
  `convert((x:A) × B x, p, (p.1 , p.2))`
- expect: **convertible (true)** — surjective pairing
- why: Σ-η via projection — `p.1 ≡ (p.1, p.2).1` at `A` and
  `p.2 ≡ (p.1, p.2).2` at `B[p.1/x]` (the dependent second component uses the
  WHNF of the first projection). Open `p`, dependent `B`.

### conversion/unit-eta (soundness)
- spec: `17 §2` (Unit-η / single-constructor record-η)
- given: open `u v : Unit` (two distinct variables); `convert(Unit, u, v)`
- expect: **convertible (true)** — any two `Unit` elements are equal
- why: Unit-η — the checker returns yes at `Unit` without comparing the terms.

### conversion/omega-proof-irrelevance (soundness)
- spec: `17 §2` (Ω proof irrelevance), `§3.3` (conv Step 1); `16 §1.2`
- given: `P : Omega_0`; open `p q : P` (distinct variables); `convert(P, p, q)`
- expect: **convertible (true)** — constant-time yes, contents not inspected
- why: proof irrelevance at `P : Ω` is definitional. The conversion algorithm
  short-circuits at an Ω-typed comparison **before** any structural work.
  Re-tests `observational/omega-pi-convertible` through the unified `17 §3.3`
  conv entry point (the K2c path).

### conversion/omega-universe-not-pi (soundness)
- spec: `17 §3.3` (conv step 1 fires only when `typeOf(A) is Omega_l`; step 5
  structural); `16 §1.2`, `16 §2.2` (propext is *propositional*, not
  definitional, equality)
- given: `Top : Omega_0` (the unit proposition) and `Bottom : Omega_0` (the
  empty proposition); `convert(Omega_0, Top, Bottom)` — the **governing type is
  the universe `Omega_0`**, so `Top`/`Bottom` are compared **as elements**
- expect: **not convertible (false)**
- why: proof irrelevance (`conv` step 1) fires only when the governing type is
  itself a proposition (`typeOf(A) is Omega_l` — comparing two *proofs* of one
  prop). Here the governing type **is the universe** `Omega_0` (a type, not a
  proposition), so the comparison falls through to structural (step 5): two
  **distinct** propositions are not definitionally equal — they are equal only
  when mutually implying (propext, the *propositional* `Eq Omega`, not
  definitional convertibility). If conversion equated all elements of `Omega_l`
  (the unsound `A is Omega_l` disjunct), then `Top ≡ Bottom`, giving a closed
  inhabitant of `Empty` — a consistency break. **Contrast with
  `omega-proof-irrelevance`**: proofs *of* a prop are all equal; propositions
  *as elements of the universe* are not. The existing `omega-pi` case tests the
  former and would not catch this — this case pins the latter (Architect
  blocker regression).

### conversion/omega-universe-distinct-props (soundness)
- spec: `17 §3.3` (conv step 1 guard; step 5 structural)
- given: open distinct propositions `P Q : Omega_0` (distinct variables);
  `convert(Omega_0, P, Q)`
- expect: **not convertible (false)**
- why: the open-term generalization — distinct propositions compared as
  elements of the universe are structurally distinct neutral heads ⇒ not
  convertible. Guards against a checker that special-cases the `Top`/`Bottom`
  literals but still collapses general props at `Omega_l`. (K1 lesson: open
  terms, not just the named-literal instance.)

### conversion/prop-arg-skip-spine (soundness)
- spec: `17 §3.4` (convSpine; prop-argument skip); `16 §8.2`
- given: `A : Type 0`, `P : A -> Omega_0`, `B : Type 0`; open
  `f : (x:A) -> P x -> B`; `a : A`; distinct `p q : P a`;
  `convert(B, f a p, f a q)`
- expect: **convertible (true)** — the propositional argument position
  (`P a : Ω`) is **skipped** during spine comparison; `p`/`q` never compared
- why: convSpine skips arguments whose binder type is in Ω. Exercises the
  per-position prop-skip inside the structural spine walk with an open
  prop-indexed family `P x`. Distinct `p`,`q` make the test fail on any checker
  that compares the skipped argument.

---

## Acceptance criterion: obs conversions decide via the unified path (item 5)

These re-test the K2 observational reductions **through** the `17 §3` conversion
entry point (not as standalone reductions — that is `seed-observational.md`):
conversion must consume the obs WHNF rules and decide.

### conversion/cast-refl-through-conv (soundness)
- spec: `17 §3.2` (whnf: cast-refl); `16 §3.2`
- given: `A : Type 0`, open `a : A`; `convert(A, cast A A refl a, a)`
- expect: **convertible (true)** — `cast A A refl a` whnf-reduces to `a`
- why: regularity through conversion — whnf applies `cast A A refl a → a`, then
  `a ≡ a`. The unified algorithm must invoke the obs reduction, not treat
  `cast` as an opaque neutral.

### conversion/eq-by-type-funext-through-conv (soundness)
- spec: `17 §3.2` (whnf: Eq-by-type); `16 §2.2`
- given: `A : Type 1`, `B : A -> Type 2` (≥2 distinct levels); open
  `f g : (x:A) -> B x`; `convert(Omega_2, Eq ((x:A) -> B x) f g,
  (x:A) -> Eq (B x) (f x) (g x))`
- expect: **convertible (true)** at `Omega_2` — `Eq`-at-Π whnf-reduces to the
  pointwise Π of equalities, and the two sides convert
- why: definitional funext is consumed by conversion. The result lands at
  `Omega_(max 1 2) = Omega_2` (level discipline: predicative `max`, the
  **exact** level, per the K2 retro). ≥2 distinct levels, open `f`,`g`,
  dependent `B x`.

### conversion/quotient-eq-through-conv (soundness)
- spec: `17 §3.2` (whnf: quotient equality); `16 §5`
- given: `A : Type 0`, `R : A -> A -> Omega_0`; open `a b : A`;
  `convert(Omega_0, Eq (A / R) [a] [b], R a b)`
- expect: **convertible (true)** — quotient equality **is** the relation
- why: `Eq (A/R) [a] [b]` whnf-reduces to `R a b`; conversion uses the reduct.
  Definitional, no setoid boilerplate. Open `a`,`b`,`R`.

---

## Acceptance criterion: decidable level equality (level-discipline, plan §6.1)

The conversion path decides universe equality via the decidable level
semilattice (`12 §1`). These pin `convLevel` against the settled level calculus
— predicative `max` (associative, commutative, idempotent, unit `0`),
`max l 0 = l`, and `suc (max l1 l2) = max (suc l1) (suc l2)` — **reconciled, not
merely cited**.

### conversion/level-max-computes (soundness)
- spec: `17 §3.6` (level equality); `12 §1`
- given: `convert(Type 3, Type (max 1 2), Type 2)`
- expect: **convertible (true)** — `max 1 2 = 2`, and `Type 2 : Type 3`
- why: `max` on closed levels computes; basic level normalization in conversion.

### conversion/level-max-unit-open (soundness)
- spec: `17 §3.6`; `12 §1` (`max l 0 = l`)
- given: open level variable `l`; `convert(Type (suc l), Type (max l 0),
  Type l)`
- expect: **convertible (true)** for **all** `l` — the unit law `max l 0 = l`
- why: level equality must hold on **open** levels via the semilattice
  equations, not only closed numerals. K1 lesson: open level variable.

### conversion/level-suc-distrib-open (soundness)
- spec: `17 §3.6`; `12 §1` (`suc (max l1 l2) = max (suc l1) (suc l2)`)
- given: open levels `l1 l2` (≥2 distinct level variables);
  `convert(_, Type (suc (max l1 l2)), Type (max (suc l1) (suc l2)))`
- expect: **convertible (true)** for all `l1, l2`
- why: the `suc`/`max` distribution law is part of decidable level equality;
  conversion normalizes both sides to one form. ≥2 distinct open level
  variables — the degree of freedom the K1 lesson demands.

### conversion/level-distinct-not-convertible
- spec: `17 §3.6`; `12 §1`
- given: distinct open levels `l1 l2`; `convert(_, Type l1, Type l2)`
- expect: **not convertible (false)** — distinct level variables are not equal
- why: the negative case — `convLevel` must **not** identify distinct level
  variables. A too-eager normalizer that collapses levels would license
  `Type:Type`-adjacent unsoundness. Pairs with the positive laws above.

---

## Acceptance criterion: structural neutral comparison (supports frame item 6)

### conversion/neutral-same-head-spine (soundness)
- spec: `17 §3.3` (conv Step 4), `§3.4` (convSpine)
- given: open `x : A -> A -> B`; open `a a' b : A` with `a ≢ a'`;
  `convert(B, x a b, x a b)` and `convert(B, x a b, x a' b)`
- expect: first **true**, second **false** — same head `x`, compare spines
  position-by-position
- why: neutral-vs-neutral with equal heads compares arguments at the head's
  type; a mismatch in one position (`a` vs `a'`) yields not-convertible. Spine
  length ≥2 with an open head.

### conversion/neutral-diff-head (soundness)
- spec: `17 §3.3` (conv Step 4: different heads, neither transparent)
- given: distinct variables `x y : A -> B`; open `a : A`;
  `convert(B, x a, y a)`
- expect: **not convertible (false)** — different **opaque** heads, neither a
  transparent constant, so no δ-unfold is possible
- why: distinct neutral heads are not convertible and the checker must **not**
  attempt to unfold a variable/opaque head. Pairs with
  `delta-unfold-heads-differ` (which **does** unfold, because there the heads
  are transparent constants).

---

## Acceptance criterion: decidability — every query halts (frame item 6)

### conversion/decidable-halts (property)
- spec: `17 §5`; `18 §6` (decidable conversion)
- given: every conversion and type-checking query exercised by this file and
  `../judgments/seed-judgments.md`, run by the conformance harness
- expect: **every query halts** with a definite yes/no (or typed error) — no
  infinite loop, no semi-decision, no stack overflow
- why: decidability is the K2c payoff and a meta-property of the whole corpus,
  not a single case — SCT bounds δ, the core reductions are SN, so conversion
  terminates on every well-typed input. A regression that makes any query
  diverge breaks the "kernel is a checker, always halts" contract.

---

## Series-2 (obs-completion) deferred seams — NOT series-1 cases

Per the K2c branch-series split (`docs/program/wp/K2c-elaboration-plan.md §0,
§5`), the three K2 obs-reduction completeness seams are **series 2**
(`seed-obs-completion.md`, NbE-dependent). They are tagged here so the series-1
build and review know these are **expected to remain sound-stuck** under
series-1 — each falls back to a neutral/stuck term, never a wrong result:

- **[K2c-series2] conversion/cast-at-inductive-index** — `cast (Vec A n)
  (Vec A m) e (vcons n a xs)` with `n ≢ m`: series-1 leaves the index `n` and
  the recursive cast stuck (sound); series-2 rewrites the index to `m` via NbE
  read-back. (`observational/seed-observational.md` already tags
  `cast-computes-inductive` `[K2c]`.)
- **[K2c-series2] conversion/j-non-constant-motive** — `J` on a non-`refl`
  equality with a **non-constant** motive: series-1 stays neutral (sound);
  series-2 builds the full `cong`/`sym` pair-equality transport.
- **[K2c-series2] conversion/quotient-respect-full** — `elim_/` into a non-Ω
  target: series-1 gates reduction on a raw-well-formed respect proof
  (sound-stuck when unverified); series-2 verifies the full `cong`/`cast`
  respect schema.

A series-1 conformance run MUST NOT mark these as passing reductions; they are
the series-2 acceptance surface.

---

## Regression: K1 + K2 conformance unchanged

### conversion/k1-k2-subset-still-green (soundness)
- spec: `spec/10-kernel/README.md §6`
- given: all K1-scoped cases (`../seed-k1.md`, `../seed-kernel.md`) and
  K2-scoped cases (`../observational/seed-observational.md`)
- expect: **all pass** — K2c hardens conversion; it does not change the
  observable equality K1/K2 already pinned
- why: the observable equality MUST be identical whichever way it is computed
  (frame Guardrails). K2c's lazy-WHNF + SCT must not regress any K1/K2 case.
