# Kernel-2c series-2 conformance — observational-reduction completeness

Format: `../../README.md`. These pin the **three K2c series-2 seams** — the
observational reductions K2 left **sound-stuck** (a neutral fallback, never a
wrong result) and series-2 **completes** so they reduce. Each was placeheld in
`../conversion/seed-conversion.md` §"Series-2 deferred seams"; series-2 flips
each from sound-stuck → computed. Trust root (★★★): a wrong reduction here is a
wrong result inside the conversion checker (potential unsoundness), so every
case is **discriminating** and each seam carries the adversarial "would this
inhabit `Empty`?" probe.

Ground truth: `spec/10-kernel/16-observational.md` §2.2 (`Eq`-at-inductive),
§3.2 ("Index rewrite" + "Where the guard sits"), §3.3 (mutual termination),
§4.1 (`J` at a dependent motive). The quotient-`respect` schema (seam 3, an
**admission-time** gate, not a `whnf` reduction) lives under
`../conversion/seed-conversion.md` §"Series-2 — quotient respect schema". Clean-
room: grounded in the landed §-bodies + first principles; `yon` not consulted.

**Disciplines applied here** (the K1.5 carries, now standing): each
"reduces/stuck" claim is **independently re-derived** from first principles
(not just matched to the §-body), every "stuck" is verified to be a genuine
**guard-gated** stall (the finiteness-not-stuckness rule — "does this redex fire
when the variable is abstract?"), and an **internal-consistency pass** confirms
no two cases contradict on overlapping inputs (the positive reduction and the
open-index stuck case are discriminated purely by the §3.2 guard).

---

## Seam 1 — `cast` at an inductive index change (`16 §3.2` "Index rewrite")

### kernel/observational/cast-inductive-index-rewrite (soundness)
- spec: `16 §3.2` "Index rewrite", `§3.3`
- given: `Vec A : Nat → Type 0`; `cast (Vec A (suc n)) (Vec A (suc m)) e (vcons
  n a xs)`, where `e` is a canonical `Eq Type (Vec A (suc n)) (Vec A (suc m))`
  whose index equality `eq_idx : Eq Nat (suc n) (suc m)` decomposes suc-
  injectively (`§2.2`) to `eq' : Eq Nat n m`
- expect: **reduces-to** `vcons m a (cast (Vec A n) (Vec A m) (cong (Vec A) eq')
  xs)` — constructor-headed at the **target** index `suc m`, well-typed `: Vec A
  (suc m)`
- why: the suc-injectivity index rewrite (`§3.2`). The **forced** index argument
  `n ↦ m` is read off the target index `suc m`; the element `a : A` is unchanged
  (regularity); the recursive `xs : Vec A n` is **sub-cast** to `Vec A m` along
  `cong (Vec A) eq'`. Flips the carry-forward `cast-computes-inductive` from
  sound-stuck (series-1) → computed. **Verdict-flip (structural):** the removed
  naive rewrite — keep the index `n`, re-head `vcons` at `m` — yields the **ill-
  typed** reduct `vcons m a xs` with `xs : Vec A n` under a `Vec A (suc m)`
  head; the correct rule emits the `m`-indexed, well-typed form. The two reducts
  differ structurally (`cast (Vec A n) (Vec A m) … xs` vs bare `xs`). Exercised
  at ≥2 levels (`A : Type 0` and `A : Type 1` — the reduct's component types
  track the
  level; `cast`/`Eq` formation is level-polymorphic, `§3.1`). It supersedes
  the imprecise existing `cast-computes-inductive`, which wrote `cast (Vec A n)
  (Vec A m) … (vcons n a xs)` though `vcons n a xs : Vec A (suc n)` — the index
  was off by the `suc`.

### kernel/observational/cast-inductive-open-index-stuck (soundness)
- spec: `16 §3.2` "Where the guard sits"
- given: `cast (Vec A (suc n)) (Vec A k) e (vcons n a xs)` where `k : Nat` is a
  **variable** (open/neutral index), so `Eq Nat (suc n) k` is neutral and does
  **not** decompose canonically
- expect: **neutral / stuck** — the cast does **not** fire and does **not**
  fabricate a re-indexed constructor
- why: the index rewrite fires **only** when every index equality decomposes
  canonically — both sides headed by the **same** index constructor (`§3.2`
  guard). An open `k` is not constructor-headed, so suc-injectivity has nothing
  to invert; the cast is genuinely **stuck by the guard**. **Independently re-
  derived** (not lifted): with `k` abstract, `Eq Nat (suc n) k` reduces to a
  neutral `Eq` (`§2.2` neutral case) — the gate condition is underivable, so the
  redex truly cannot fire (a guard-gated stall, not a false-stuckness claim).
  **Internal-consistency:** discriminated from `cast-inductive-index-rewrite`
  purely on the guard axis (canonical same-index-ctor → reduces; neutral index →
  stuck) — the two do **not** contradict. This is the K2 closed-`Empty`
  discipline applied to index transport: gate on the index condition, never
  fabricate a constructor past an underivable guard.

---

## Seam 1 mutual sibling — `Eq` at an inductive dependent telescope (`16 §2.2`)

### kernel/observational/eq-inductive-dependent-telescope (soundness)
- spec: `16 §2.2` (Eq-at-inductive, Vec example), `§3.2`
- given: `Eq (Vec A (suc n)) (vcons A n a xs) (vcons A n' a' xs')` — a dependent
  telescope (index arg, element, recursive arg — length ≥2)
- expect: **reduces-to** `Eq Nat n n'  and  Eq A (cast A A refl a) a'  and  Eq
  (Vec A n) (cast (Vec A n) (Vec A n') (cong (Vec A) eq_n) xs) xs'`, where `eq_n
  : Eq Nat n n'` is the first argument equality
- why: same-constructor `Eq`-at-inductive is the conjunction of argument
  equalities, each later (dependent) argument **transported along the earlier
  equalities** before comparison (`§2.2`). The recursive `xs` conjunct's
  **dependent-telescope cast** (`cast (Vec A n) (Vec A n') (cong (Vec A) eq_n)
  xs`) needs the cast-at-inductive completion (seam 1) to reduce — so this
  carry-forward seed flips from sound-stuck → computed. The mutual sibling of
  seam 1:
  `Eq` calls `cast` on the dependent component (`§3.3` single mutual system).
  **Verdict-flip (structural):** assert the three-way conjunction with the
  **transported** third conjunct; a kernel that compared `xs` and `xs'` at
  mismatched types (no transport) or left the recursive conjunct stuck reaches a
  different (or stuck) result. Different-constructor remains `⇝ Bottom`
  (unchanged `eq-inductive-diff-ctor`).

---

## Seam 2 — `J` at a dependent (non-constant) motive (`16 §4.1`)

### kernel/observational/j-dependent-motive-fires (soundness)
- spec: `16 §4.1` (J-cast)
- given: `J A a P d b e` where `e : Eq A a b` is a canonical non-`refl` equality
  and `P` is a **dependent (non-constant)** motive landing in an indexed family
  (e.g. `P b e = Vec A (f b)`) with the index **closed and constructor-headed**
- expect: **reduces** — `J A a P d b e ⇝ cast (P a (refl a)) (P b e) pair-eq d`
  (J-cast) **fires** for every non-`refl` `e`, then the `cast` computes by cast-
  by-type, **bottoming through the index rewrite (`cast-inductive-index-
  rewrite`)** to a value
- why: `J` is `cast` at the singleton type (`§4.1`); the J-cast fires for
  **every** non-`refl` `e` — **motive constancy is not a gate** (`pair-eq` is a
  typing
  witness only, never inspected by `cast`, `§3.4`). Constant motive ⇒ `pair-eq ≡
  refl` ⇒ regularity ⇒ `d` (the headline `j-nonrefl`, **unchanged** — regression
  anchor). Dependent motive ⇒ cast-by-type descends through Π/Σ and the index
  rewrite. **Verdict-flip:** the K2-deferred behavior — gate `J`-reduction on
  motive constancy, leaving a dependent-motive `J` **neutral** — reaches a stuck
  `J`; the correct rule reduces to the transported value (green-vs-red on the
  reduct). **Finiteness-not-stuckness discipline (my K1.5 carry, applied):** do
  **not** assert a dependent-motive `J` is "stuck" — the J-cast **fires**; the
  only residual neutrality is the **inner cast** at an open index (the seam-1
  guard), never a stuck `J`. Independently verified by asking "does (J-cast)
  fire under an abstract proof `e`?" — yes; only the inner cast may stall. The
  case
  picks a **closed canonical** index so it fully computes (not green-vs-green).
  **`Empty` probe:** `cast` on mismatched endpoint-type heads is ill-typed (`Eq
  Type (P a (refl a)) (P b e) ⇝ Bottom`, `§2.2`), so a J-cast across
  incompatible types is never formed on a closed term — the transport cannot
  land a value in a wrong type.

---

## Regression — existing obs corpus unchanged

### kernel/observational/obs-completion-no-regression (soundness)
- spec: `16 §2.2`, `§3.2`, `§4.1`
- given: the existing `seed-observational.md` cases (`cast-refl`,
  `cast-computes-{pi,sigma,quotient}`, `j-on-refl`, `j-nonrefl` constant-motive
  headline, `eq-inductive-{same,diff}-ctor`, the Ω/quotient/trunc cases) re-run
  under series-2
- expect: **all unchanged-green** — series-2 **completes** stuck cases to
  reduce; it does not change any reduction that already fired
- why: the seams add reductions where K2 fell back to neutral; every reduction
  K2 already performed (regularity, `cast` at Π/Σ, `J`-β, constant-motive `J`,
  same/diff-ctor `Eq`) is untouched. The observable equality is identical
  whichever way computed (`§3.4`). Regression gate for the trust-root
  completion.
