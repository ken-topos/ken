# Definitional equality, conversion, and termination

> Status: **K2c elaborated** (series 1 — conversion-hardening). Normative.
> Defines what the kernel treats as *the same* — the reductions (β/ι/δ/obs), the
> type-directed η + proof irrelevance, the conversion algorithm (lazy-WHNF +
> on-the-fly structural comparison + lazy δ, with NbE as the declarative
> reference, §3), and the **size-change termination (SCT)** gate (§4) that keeps
> δ-unfolding — and therefore type-checking — **decidable** (§5). The contract
> for K2c. (The three K2 obs-completion seams — `cast`-at-inductive index
> rewrite, non-constant-motive `J`, full quotient `respect` — are **series 2**,
> elaborated in `16 §3.2`/`§4.1`/`§5.1`; §3 consumes the `16` obs reductions as
> they stand. **Series 2 changes no rule in this chapter:** the seams compute
> *inside* `whnfObs`/`whnfJ` (§3.2) and each terminates within a single `whnf`
> by structural descent on the finite type (`16 §3.3`); the quotient-`respect`
> completion is an **admission-time** check (`16 §5.1`), adding no reduction.
> The **SCT gate (§4) is unchanged** — it already scores `cast` under recursion
> as `?` (§4.2), the conservative measure that bounds these reductions without
> needing to track them.)

Two terms that are **definitionally equal** are interchangeable everywhere with
no proof obligation; equalities that are *not* definitional are propositions to
prove via `Eq` (`15`). Getting this boundary right is most of what a dependent
kernel *is*.

## 1. The reductions

Definitional equality `Γ ⊢ a ≡ b : A` is the least typed congruence closed under
the following reductions and the η rules (§2):

| Rule | Redex → reduct | From |
|---|---|---|
| **β** | `(λ (x:A). t) u → t[u/x]` | `13 §1` |
| **Σ-β** | `(a,b).1 → a`, `(a,b).2 → b` | `13 §2` |
| **ι** | `elim_D M m̄ … (cₖ ā) → mₖ …` (structural) | `14 §3` |
| **δ** | `c → t` for `(c : A := t) ∈ Σ` (transparent) | `11 §4` |
| **prim** | `op lit̄ → lit` (registered primitive reduction) | `14 §5` |
| **obs** | `Eq`-by-type; `cast A A refl a → a` + `cast`-by-type; quotient elim | `16` |

- δ (constant unfolding) is **controlled**: the conversion algorithm unfolds a
  definition only when needed to make progress (§3), never eagerly. Opaque
  constants (`11 §4`) never δ-reduce.
- **prim** reductions are the trusted boundary (`14 §5`): `add 2 3 → 5` lets
  proofs compute over literals.
- A term with no applicable reduction at its head is **neutral** (a variable, an
  opaque constant, a primitive on non-literals, an `elim`/`cast`/quotient-elim
  on a neutral target). Conversion compares neutrals structurally (§3).

**Confluence.** The reduction system is confluent (Church–Rosser); normal forms
are unique up to α (de Bruijn identity) and Ω proof-irrelevance. This is a
metatheoretic commitment (`18 §6`), tested behaviorally by the conformance
corpus.

## 2. η (type-directed)

η-equalities are **not** plain reductions (they depend on the *type*) and are
applied by the conversion algorithm at the relevant type:

- **Π-η:** at a Π-type, `f ≡ g` iff `f x ≡ g x` for a fresh `x` (`13 §1`). The
  algorithm η-expands both sides under a fresh binder.
- **Σ-η:** at a Σ-type, `p ≡ q` iff `p.1 ≡ q.1` and `p.2 ≡ q.2` (`13 §2`).
- **Proof irrelevance (Ω):** at a type `P : Ω`, **any** two terms are equal — `p
  ≡ q` with no comparison of contents (`16 §1`). This is definitional and is
  what makes equality (`Eq : Ω`) and the whole logic proof-irrelevant; the
  checker **skips propositional arguments** entirely.
- **Unit-η** (and record-η for single-constructor types with η, `14 §4`): any
  two elements of `Unit` are equal; the η for records follows from Σ-η.

Type-directed η is why conversion needs the type, not just the two terms — the
algorithm is `conv Γ A a b`, not `conv a b`.

## 3. The conversion algorithm

**`OQ-eval-strategy` — DECIDED (operator, 2026-06-27): follow Lean's kernel.**
The *operational* algorithm is **lazy weak-head normalization with on-the-fly
structural conversion and lazy δ-unfolding** — Lean 4's battle-tested,
heavily-scrutinised approach (consistent with Ken already adopting Lean's
small-trusted-kernel model, ADR 0001): reduce only enough to expose a head,
compare heads incrementally, and unfold a transparent definition (δ) **only when
forced** (heads differ and at least one is transparent), preferring *not* to
unfold. **Normalization by evaluation (NbE)** is the **declarative reference** —
the meaning of "equal" (§3.7) — and the observable equality MUST be identical
whichever way it is computed.

The algorithm has three mutually-recursive entry points, all relative to the
global environment `Σ` (written `env`) and a local context `Γ` (written `ctx`):

- **`whnf env ctx t`** (§3.2) — reduce `t` to **weak-head normal form**: enough
  reduction to expose the head former, not under binders.
- **`conv env ctx A a b`** (§3.3) — decide `Γ ⊢ a ≡ b : A`, **type-directed** so
  η and Ω proof-irrelevance can fire.
- **`convSpine env ctx hty sp_a sp_b`** (§3.4) — compare the argument spines of
  two neutrals with equal heads, at the head's type telescope `hty`.

The pseudocode below is the **normative contract** for the observable yes/no.
Where it fixes a step that affects only *performance* (the exact δ-unfold
heuristic, memoization, the content-hash fast path), that step is
**non-normative** and flagged as such — an implementation MAY choose differently
as long as the decided equality is unchanged.

### 3.1 Notation for the pseudocode

`t` ranges over core terms (`11 §1`); the term formers are matched by the
grammar of `11 §1` (`Type ℓ`, `Ω_l`, variables `x`, constants `c`, `(x:A)→B`,
`λ(x:A).t`, `t u`, `(x:A)×B`, `(t,u)`, `t.1`, `t.2`, `Eq A a b`, `refl t`,
`cast A B e t`, `J M d e`, the inductive former/constructor/eliminator, `A/R`,
`[t]`, `elim_/ M f s`, `‖A‖`, `|t|`, `let`, ascription). `subst t u` is the
capture-avoiding single substitution `t[u/x]` of `11 §5`. `head t` is the
outermost former of an already-whnf'd `t`; `spine t` is the list of arguments
applied to that head. `fresh ctx A` is a fresh variable `x` (de Bruijn level =
`|ctx|`, read back as the index `Var(|ctx|)` of `13 §1`) extending `ctx` with
`x : A`. Transparency of a constant is read from `env` (`11 §4`): a transparent
definition `c : A := t` is δ-unfoldable; an opaque constant / postulate is not.

### 3.2 `whnf` — lazy weak-head normalization

`whnf` applies the §1 reductions at the head until none applies (the head is a
canonical former or a neutral). It never descends under a binder and never
forces an argument it does not need.

```
function whnf(env, ctx, t):
  loop:
    t := stripAscription(t)              // (t : A) ⇝ t   (11 §1; erased)
    match t:

      // β  (13 §1)
      App(f, u):
        f := whnf(env, ctx, f)
        if f is Lam(x, A, body): t := subst(body, u); continue
        else: return App(f, u)           // neutral application — f is stuck

      // Σ-β  (13 §2)
      Proj(p, i):
        p := whnf(env, ctx, p)
        if p is Pair(a1, a2): t := (i == 1 ? a1 : a2); continue
        else: return Proj(p, i)          // neutral projection

      // ι  (14 §3) — eliminator on a constructor
      Elim(D, M, methods, s):
        s := whnf(env, ctx, s)
        if s is Ctor(k, args): t := applyMethod(methods[k], args, M); continue
        else: return Elim(D, M, methods, s)   // neutral scrutinee

      // obs: Eq-by-type / cast / quotient-elim / truncation-elim  (16 §2,3,5,6)
      Eq(A, a, b):       return whnfObs(env, ctx, Eq(A, a, b))    // 16 §2.2 dispatch on whnf(A)
      Cast(A, B, e, a):  return whnfObs(env, ctx, Cast(A, B, e, a)) // 16 §3.2 incl. cast-refl regularity
      QuotElim(M, f, s): if whnf(s) is Class(a): t := App(f, a); continue
                         else: return QuotElim(M, f, whnf(s))
      TruncElim(P, f, s):if whnf(s) is Trunc(a): t := App(f, a); continue
                         else: return TruncElim(P, f, whnf(s))

      // J — derived; reduces via cast  (15 §4, 16 §4)
      J(M, d, e):        return whnfJ(env, ctx, J(M, d, e))   // J-β to d on refl; non-refl via cast

      // prim — only on literals  (14 §5)
      Prim(op, args):
        if all args are literals: t := primReduce(op, args); continue
        else: return Prim(op, args)      // neutral primitive

      // let  (11 §1) — definitional unfolding of a local binding
      Let(x, ty, rhs, body): t := subst(body, rhs); continue

      // δ — unfold a transparent global constant at the head (lazy; see §3.5)
      Const(c):
        if env[c] is a transparent definition AND deltaWanted:
          t := env[c].body; continue
        else: return Const(c)            // opaque, or δ deferred — neutral head

      // already weak-head normal (Type, Ω, Π, Σ, λ, pair, refl, ctor, neutral, …)
      _ : return t
```

Notes.

- **`whnfObs`** is the observational reduction engine of `16 §8.1`: it computes
  `whnf(A)` (and `whnf(B)` for `cast`), dispatches on the head type former per
  the `Eq`-by-type table (`16 §2.2`) and the `cast`-by-type table (`16 §3.2`,
  including the `cast A A refl a ⇝ a` regularity rule), and returns a neutral
  when the governing type is neutral. K2c **consumes** these rules; it does not
  re-derive them (frame: "the obs conversions are K2's").
- **`deltaWanted`** governs the *laziness* of δ at the head. In a bare `whnf`
  call it is `true` (a transparent head unfolds). In conversion, `whnf` is first
  run with δ **deferred** so heads can be compared before unfolding; δ fires
  only on the `conv` retry path of §3.5. This split is the whole point of "lazy
  δ" and is detailed in §3.5.
- **Termination.** Every branch except `Const`/`Let` strictly shrinks the term
  (β/Σ-β/ι/prim contract it; obs descends on the type, `16 §3.3`). `Let` is
  acyclic (`11 §4`). The `Const` branch is the only source of unbounded
  unfolding, and it is exactly what the **SCT gate (§4)** bounds — see §5.

### 3.3 `conv` — type-directed conversion

`conv` decides `Γ ⊢ a ≡ b : A`. It normalizes the governing type to expose η /
proof-irrelevance opportunities, then either drives an η rule or falls through
to a structural head-and-spine comparison.

```
function conv(env, ctx, A, a, b):
  A := whnf(env, ctx, A)

  // (1) Ω proof-irrelevance (16 §1.2): any two proofs of a strict prop are equal.
  if typeOf(A) is Ω_l:                     // A is a *proposition* (A : Ω_l), NOT the universe Ω_l itself
    return true                            // constant-time "yes"; contents unread
  // NB: do NOT short-circuit when A *is* the universe Ω_l — then a, b are
  // propositions compared as elements, and must fall to (5)/convStruct. Equating
  // them here would make Top ≡ Bottom and inhabit Empty (unsound).

  // (2) η at Π (13 §1): compare bodies under a fresh argument.
  if A is Pi(x, A1, B1):
    let y = fresh(ctx, A1)
    return conv(env, ctx⊕(y:A1), B1[y], App(a, y), App(b, y))

  // (3) η at Σ (13 §2): compare projections.
  if A is Sigma(x, A1, B1):
    if not conv(env, ctx, A1, Proj(a,1), Proj(b,1)): return false
    let a1 = whnf(env, ctx, Proj(a,1))
    return conv(env, ctx, B1[a1], Proj(a,2), Proj(b,2))

  // (4) Unit-η / single-ctor record-η (14 §4): any two elements are equal.
  if A is Unit or a record type with η:
    return true

  // (5) Structural: no η rule applies — reduce both sides and compare.
  a := whnf_deferδ(env, ctx, a)            // whnf with δ deferred (§3.5)
  b := whnf_deferδ(env, ctx, b)
  return convStruct(env, ctx, A, a, b)


function convStruct(env, ctx, A, a, b):
  match (head a, head b):

    // same canonical former — compare components at their types
    (Type, Type):     return convLevel(levelOf a, levelOf b)         // §3.6
    (Ω, Ω):           return convLevel(levelOf a, levelOf b)
    (Pi,  Pi):        return conv(env,ctx,Type,dom a,dom b)
                          and conv(env, ctx⊕dom a, Type, cod a, cod b)
    (Sigma, Sigma):   return conv(env,ctx,Type,fst a,fst b)
                          and conv(env, ctx⊕fst a, Type, snd a, snd b)
    (Ctor k, Ctor k): return convSpine at the constructor's arg telescope  // skips Ω args
    (Eq, Eq):         return conv(env,ctx,Type,tyA a,tyA b)
                          and conv(env,ctx,tyA a,lhs a,lhs b)
                          and conv(env,ctx,tyA a,rhs a,rhs b)
    (Quot, Quot):     return conv(carrier a, carrier b) and conv(rel a, rel b)
    (Trunc, Trunc):   return conv(env,ctx,Type,arg a,arg b)
    // refl carries no content beyond its endpoints; equal once the endpoints are.
    (Refl, Refl):     return true

    // neutral vs neutral with the same head — compare spines at the head's type
    (h_a, h_b) where neutral(a) and neutral(b):
      if h_a == h_b:                       // same variable / opaque const / stuck elim
        let hty = headType(env, ctx, h_a)
        return convSpine(env, ctx, hty, spine a, spine b)
      // (6) δ-unfold trigger (§3.5): heads differ — unfold a transparent one and retry
      if isTransparent(env, h_a) or isTransparent(env, h_b):
        return conv(env, ctx, A, whnf_unfoldδ(env,ctx,a), whnf_unfoldδ(env,ctx,b))
      return false                         // distinct rigid heads — not convertible

    _ : return false                       // canonical/neutral head mismatch
```

The Ω test at (1) uses the membership check of `16 §8.2`: it fires **only** when
`A` is a *proposition* — `Γ ⊢ A : Ω_l` — never when `A` *is* the universe `Ω_l`.
The level `l` here is the **predicative** Ω-level of the proposition (`12 §2`,
`16 §1.1`): the shortcut fires at *every* `Ω_l`, and is sound at each because
proof irrelevance (`16 §1.2`) holds level-by-level — there is **no
cumulativity** collapsing the `Ω_l` (`12 §3`), and none is needed, since the
verdict (`true`) does not depend on `l`.

**The universe case is deliberately excluded.** When the governing type is
itself `Ω_l` — i.e. `a`, `b` are *propositions*, compared as elements of the
universe Ω — proof irrelevance does **not** apply (it equates proofs *of* a
fixed prop, not distinct props), and the comparison falls through to
(5)/`convStruct`. Two propositions are convertible iff **structurally equal**,
exactly like two types at `Type ℓ`, with `convLevel` (§3.6) deciding any
embedded level equality. This is load-bearing: short-circuiting `true` at the
universe would equate every pair of propositions — `conv(Ω_l, Top, Bottom)`
would succeed, `Top ≡ Bottom`, and a closed inhabitant of `Empty` would follow.
Distinct props (`Top` vs `Bottom`, distinct `Eq`/`Π`-props) have distinct heads
and are correctly **not** convertible; mutual implication is *propositional*
equality (propext, `16 §2.2`), never definitional. The conformance corpus pins
this with `conversion/omega-universe-not-pi` (`Top ≢ Bottom` at `Ω_0`).

### 3.4 `convSpine` — argument-by-argument at the head's type

```
function convSpine(env, ctx, hty, sp_a, sp_b):
  if sp_a and sp_b are both empty: return true
  if one is empty and the other is not: return false    // arity mismatch ⇒ not equal
  let (arg_a, rest_a) = sp_a, (arg_b, rest_b) = sp_b
  hty := whnf(env, ctx, hty)
  require hty is Pi(x, A_i, B):                          // head applied to arg_i : A_i
    if A_i is a proposition (typeOf A_i is Ω_l):
      skip                                               // prop-arg skip (16 §1.2, §8.2)
    else if not conv(env, ctx, A_i, arg_a, arg_b):
      return false
  return convSpine(env, ctx, B[arg_a], rest_a, rest_b)   // continue at the instantiated codomain
```

The **propositional-argument skip** is the same shortcut as `16 §8.2`: when a
spine position has a type in Ω, the two arguments are definitionally equal by
proof irrelevance and are not compared. This is what frees agents from
synthesising coherence/transport terms (`16 §1.2`).

### 3.5 The δ-unfold trigger (lazy discipline)

δ is the only reduction that can *grow* a term, so the algorithm unfolds a
transparent definition as little as possible. Two `whnf` modes realise this:

- **`whnf_deferδ`** — weak-head-normalize but treat every transparent `Const(c)`
  at the head as **neutral** (do not unfold). Used by `conv` step (5) so the two
  sides expose their *outermost shared structure* before any unfolding.
- **`whnf_unfoldδ`** — `whnf` with δ enabled at the head (the `deltaWanted =
  true` path of §3.2). Used only on the retry of `convStruct` step (6).

**The trigger.** Unfold a transparent definition during conversion **only when**
(i) the two heads differ after `whnf_deferδ`, **and** (ii) at least one head is
a transparent constant. Consequences:

- `f ā` vs `f ā` where `f` is transparent: heads are *equal* (`Const f`), so
  **no unfolding** — compare the spines `ā` structurally (`convSpine`). This is
  the congruence shortcut that avoids unfolding a shared definition on both
  sides.
- `f ā` vs `g b̄` with `f` transparent and `g` distinct: heads differ and `f` is
  transparent, so unfold (at least `f`) and retry from `conv` step (5).
- a transparent head against an *opaque* / variable head: unfold the transparent
  side and retry; if they still differ after the transparent side is fully
  unfolded, they are not convertible.

**Resolution of the `whnf`/`conv` δ split.** A bare `whnf` (the API entry of
`18 §4`, used by the interpreter and the prover's checker) unfolds transparent
heads eagerly (`deltaWanted = true`) — that is what "transparent" means
operationally. *Conversion* is the one caller that defers δ, to keep the lazy
congruence shortcut above. Both observe the same equality; the deferral only
changes *when* the work is done, never the yes/no. (This reconciles the two
δ paths the K2/K1 build surfaced; it is the Lean discipline, and is settled by
`OQ-eval-strategy` — not an open question.)

**(perf, non-normative).** *Which* side to unfold first when both heads are
transparent, and how far, is a performance heuristic (Lean uses definitional
height; an implementation MAY memoize, or take the §3.9 content-hash fast path
first). The observable result is fixed by completeness for definitional
equality; the heuristic is **(oracle)** only in the sense that the reference
interpreter fixes a concrete order — it can never change a
convertible/not-convertible verdict.

### 3.6 Level equality `convLevel`

Universe and Ω comparisons reduce to **decidable level equality** (`12 §1`).
Levels `ℓ ::= 0 | suc ℓ | max ℓ₁ ℓ₂ | u` form a commutative idempotent
semilattice; `convLevel` decides `ℓ₁ = ℓ₂` modulo the `12 §1` laws (`max`
associative / commutative / idempotent, `max ℓ 0 = ℓ`, `suc (max ℓ₁ ℓ₂) = max
(suc ℓ₁) (suc ℓ₂)`):

```
function convLevel(l1, l2):
  return normLevel(l1) == normLevel(l2)        // syntactic equality of normal forms

function normLevel(l):
  // push suc inward and flatten max into a set of (variable, offset) atoms:
  //   normLevel l = max over atoms of (u + k)  ∪  a constant offset c
  // where each atom is a level variable u with an added suc-depth k, and c is
  // the maximal pure-numeral summand. Drop dominated atoms (same u, smaller k);
  // drop c if some atom is present and c is not larger. The result is canonical.
```

`convLevel` is **equality, not subtyping**: Ken is **non-cumulative** (`12 §3`),
so `Type ℓ` and `Type (suc ℓ)` are *not* convertible and there is no `Type ℓ ≤
Type ℓ'` subsumption in the kernel — any genuine lift is an explicit elaborator
insertion (`12 §3`), never a conversion success. The kernel receives
**explicit** level arguments on every level-polymorphic use (`12 §4`);
`convLevel` re-checks the induced constraints as ordinary level equalities at
each instantiation.

### 3.7 NbE as the declarative reference

NbE is the **meaning** of "equal" against which the operational algorithm above
is judged sound and complete. The reference read-back:

1. **Evaluate** each side into a semantic domain of **values** — weak-head
   normal forms with closures for binders and **neutrals** for stuck
   computations. Evaluation performs β/Σ-β/ι/δ/prim/obs reductions lazily to
   weak-head normal form; it does *not* go under binders.
2. **Compare** values type-directed, head-first — the same case split as `conv`
   (§3.3): η at Π/Σ, proof-irrelevance at Ω, structural at canonical/neutral
   heads, level equality at universes, controlled δ on head mismatch.
3. **Read-back (quote)** to a normal form is used where a syntactic normal form
   is genuinely needed (storing an elaborated term, the conformance corpus's
   normal-form checks); η-long, δ-short normal forms are the reference output.

The algorithm is **sound and complete** for definitional equality and
**terminates** (§4, §5). The **recommended implementation** is the Lean-style
lazy-WHNF + on-the-fly conversion of §3.2–§3.5 (avoid full normalization;
compare incrementally; unfold δ lazily); NbE read-back is used only where a
syntactic normal form is genuinely needed. The observable equality MUST be
identical whichever way it is computed.

### 3.8 Where Ken's theory differs from Lean's

**Where Ken's *theory* differs from Lean's** (its engine is shared; ADR 0005):
`J`/`subst` reduce on **non-`refl`** equalities via the observational `cast`
rules (`15`, `16 §3`), where Lean's `Eq.rec` is stuck off `refl`. **Canonicity
is kept** — and Ken assumes **none** of Lean's axioms: in the observational
foundation **funext and propext are *definitional*** and **quotient soundness is
definitional** (quotient equality *is* the relation, `16 §5`), so Ken needs no
axiom where Lean postulates `propext`/`Quot.sound`, and assumes no `choice`; the
reflective prover (`../20-verification/23 §3`) relies on closed terms computing.
**Definitional proof irrelevance** — which Lean gets from its impredicative
`Prop` — Ken **also has**, from the *predicative* strict-prop universe Ω (`16
§1`, `OQ-Prop`), without impredicativity.

### 3.9 Fast paths (non-normative, for performance)

Because the runtime is content-addressed (`../40-runtime/41-values.md`), two
closed terms with the same content hash are equal — an O(1) shortcut conversion
MAY take before structural comparison. Memoizing whnf and sharing via the heap
make repeated conversions cheap. These are optimizations and are explicitly
**out of the decidability-critical TCB** (frame: the content-hash fast path is
non-normative); they must never report unequal terms equal or vice versa.

## 4. Termination of conversion — the SCT gate

Type-checking calls conversion; conversion unfolds δ; an unrestricted recursive
definition would make δ-unfolding (and hence conversion, and hence
type-checking) **loop**. Ken keeps decidability with a **size-change termination
(SCT)** check at definition-admission time (`11 §4`).

**What SCT checks.** When admitting a (possibly recursive, possibly mutually
recursive) transparent definition, the kernel:

1. Extracts the **call graph**: nodes are the definitions, edges are calls, each
   edge annotated with a **size-change matrix** recording, for each pair (caller
   parameter, callee argument), whether the argument is **strictly smaller**
   (`↓`), **not larger** (`↓=`), or **unrelated** (`?`) than the parameter, in
   the structural (subterm) order on canonical/inductive values.
2. Forms the **idempotent closure** of the call graph under composition of
   size-change matrices.
3. **Accepts** iff every idempotent loop (every way a call can return to itself)
   has at least one parameter that **strictly decreases** (`↓` on the diagonal)
   — i.e. every infinite call sequence would have an infinitely-decreasing
   thread, which is impossible over a well-founded order. This is the
   size-change principle (Lee, Jones & Ben-Amram 2001).

### 4.1 `sct_check` — the admission gate

`declare_def` (`18 §4`) calls `sct_check` on the mutually-recursive group being
admitted. The group is the set of transparent definitions whose bodies refer to
one another (a single non-recursive definition trivially passes).

```
function sct_check(env, group):
  // group = { f_1, ..., f_m }, each f_i with parameter telescope params(f_i)
  // and body body(f_i). "Calls" are the applied occurrences of a group member.

  // (1) Build the annotated call graph.
  let G = directed multigraph on nodes { f_1, ..., f_m }
  for each f in group:
    for each call site `g ē` in body(f) where g ∈ group:
      let M = size_change_matrix(env, params(f), ē, params(g))   // §4.2
      add edge f --M--> g to G

  // (2) Idempotent closure: all composite matrices around every cycle (§4.4).
  //     Restrict to each strongly-connected component — acyclic parts cannot loop.
  let closure = idempotent_closure(G)                            // §4.4

  // (3) Accept iff every idempotent self-loop strictly decreases some parameter.
  for each (f, M) in closure with f --M--> f and M ⊙ M == M:     // M idempotent
    if not (∃ i. M[i,i] == ↓):
      return Reject(f, M)        // an infinite descent is possible — refuse
  return Accept
```

### 4.2 `size_change_matrix` — measuring one call

For a call `g ē` inside `f`, the matrix `M` has one row per parameter of `f` and
one column per argument position of `g`; `M[i,j]` compares the `j`-th call
argument `e_j` to the `i`-th caller parameter `x_i` in the **structural subterm
order**:

```
function size_change_matrix(env, callerParams, args, calleeParams):
  M := matrix(|callerParams| × |args|) filled with ?
  for i in callerParams, j in args:
    M[i,j] := sizeRel(env, x_i, e_j)
  return M

function sizeRel(env, x, e):          // x : caller parameter,  e : call argument
  e := whnf_deferδ(env, ·, e)         // expose head without unfolding the recursion
  if e is a strict subterm of x reached by ≥1 constructor field projection
       or pattern-match destructuring of x:        return ↓     // strictly smaller
  if e ≡ x (up to α),
       or e is a projection/permutation of x that is provably ≤ x in the
       subterm order and adds NO constructor:        return ↓=  // structurally ≤ x
  return ?     // EVERYTHING else: any constructor-wrapping (x ↦ c x GROWS), app, prim, cast
```

**The structural subterm order** is the well-founded order on canonical
(inductive) values: a constructor's field is strictly smaller than the
constructor (`14 §3`), transitively. `↓` is recorded when the argument is
reached from the parameter by at least one such field access (e.g. matching `x =
suc n` and calling with `n`, or `x = c ā` and calling with some `a_j`). `↓=` is
recorded **only** for the parameter itself (`e ≡ x` up to α) or a
projection/permutation that is provably `≤ x` and adds no constructor — `↓=`
means "not larger" in the order, and `compose(↓=, ↓) = ↓`, so a *single*
mis-recorded `↓=` on a **growing** argument manufactures a spurious decreasing
thread and admits a non-terminating definition (the exact failure SCT exists to
prevent). Therefore constructor-**wrapping** is never `↓=`: `x ↦ c x` *grows*
and is `?`. Everything that is not identity or a non-growing
projection/permutation — any constructor application, `app`, `prim`, `cast` — is
`?`. The conformance corpus pins this with `conversion/sct-reject-ctor-wrap` (a
re-wrapping recursion must be **rejected**).

**The size order — resolved.** The `17 §4` `(oracle)` is **pinned** (K2c
decision; rationale below), with a scoped reference-validation note:

- **Primitives are neutral.** A primitive literal (`Int`, `Bytes`, …, `14 §5`)
  has no inductive subterm structure, so no argument *built from* a primitive
  result is ever `↓` — primitive positions contribute `?` (or `↓=` only when the
  literal is passed through unchanged). This is sound: SCT never *needs* a
  primitive to decrease; treating them as non-decreasing only ever **rejects**
  more, never admits a non-terminating definition.
- **`cast` under recursion is conservatively `?`.** When a call argument is
  `cast A B e a`, the kernel does **not** attempt to prove the transport
  preserves the subterm order; it records `?`. Safe by the same argument: a
  conservative `?` can only withhold a `↓`, tightening the gate.
- **Coinduction does not arise** (`OQ-coinduction`): there are no infinite
  values whose "size" would need a dual treatment.

*Why conservative is the right default:* SCT's soundness rests on every recorded
`↓` being a *genuine* strict decrease in a well-founded order. Over-recording
`↓` would be unsound (could admit a looping definition); under-recording only
costs expressiveness (a terminating definition might be refused and need an
explicit eliminator). At the trust root we take the safe side. **(oracle —
scoped):** if Ken's reference interpreter validates a more permissive treatment
(e.g. a size-preserving `cast`), that is a *future refinement* admitting
strictly more definitions, **not** a K2c change and **not** a soundness-relevant
one.

### 4.3 Matrix composition and the idempotent closure

Entries are ordered `? < ↓= < ↓` (`?` worst, `↓` best). Composing two
consecutive call steps `M_1 : f→g` then `M_2 : g→h` yields `M_2 ⊙ M_1 : f→h`.
Two operations are involved and **must not be conflated**:

**`compose`** chains the size relation *along one thread* `i → j → k` (caller
param `i`, intermediate callee param `j`, final param `k`). It is relation
composition over the subterm order — `↓` is `>`, `↓=` is `≥`, `?` is "no known
relation" — so an `?` step **breaks** the thread (it is absorbing):

```
compose(↓,  ↓)   = ↓     // i > j > k     ⇒ i > k
compose(↓,  ↓=)  = ↓     // i > j ≥ k     ⇒ i > k
compose(↓,  ?)   = ?     // i > j , j ? k ⇒ i ? k   (NOT ↓ — the thread breaks)
compose(↓=, ↓)   = ↓     // i ≥ j > k     ⇒ i > k
compose(↓=, ↓=)  = ↓=    // i ≥ j ≥ k     ⇒ i ≥ k
compose(↓=, ?)   = ?     // i ≥ j , j ? k ⇒ i ? k
compose(?,  e)   = ?     // i ? j         ⇒ i ? k
```

**`max`** then picks the *best* (strongest) thread across all intermediates `j`
— and only here does a strict decrease dominate:

```
(M_2 ⊙ M_1)[i,k] = max over j of compose(M_1[i,j], M_2[j,k])   // max in ? < ↓= < ↓
```

So `↓` dominates **across** threads (`max`) but is never *manufactured*
**along** a thread through an unknown step: `compose(↓, ?) = ?`, not `↓`.
Recording it as `↓` would invent a spurious decreasing thread and flip a
non-terminating definition from reject to **accept** — the exact over-recording
§4.2 warns against (conformance `sct-reject-ctor-wrap-compose`).

`idempotent_closure(G)` composes edge matrices along every path within each
strongly-connected component until no new self-loop matrix appears (the set of
composite matrices is finite — entries are drawn from `{?, ↓=, ↓}` and the
dimension is fixed — so the closure terminates). A loop matrix `M` with `M ⊙ M
== M` is **idempotent**; the accept condition (§4.1 step 3) inspects exactly
these. Computing the closure only over SCCs is the standard optimization: a
definition outside any cycle can never δ-unfold into itself.

**Consequences.**

- A definition that **passes** SCT is admitted **transparent** (δ-unfoldable);
  unfolding it during conversion is guaranteed to terminate (§5).
- A definition that **fails** SCT is **rejected** as a transparent definition.
  (The elaborator MAY offer to admit it **opaque** — usable as a postulate-style
  constant that never δ-reduces — or report a totality error; policy is
  surface-level, `../30-surface/`. The *kernel* never admits a transparent
  definition it cannot certify terminating.)
- SCT is strictly more permissive than "structural recursion on one fixed
  argument": it handles permuted/lexicographic descent and mutual recursion,
  which is why it is the chosen criterion (the conformance corpus exercises all
  three, §6).
- **Scope.** SCT gates **general recursive definitions** made via δ. Recursion
  via an inductive **eliminator** (`14 §3`) is *already* structural and total
  and needs no SCT. Most surface functions elaborate to eliminators; SCT covers
  the rest.

## 5. Decidability (the payoff)

Conversion terminates on every well-typed input, so type-checking is
**decidable** — the kernel is a *checker* (always halts with yes/no), not a
semi-decision procedure. The argument has two halves that meet at the `whnf`
loop of §3.2:

1. **The core reductions are strongly normalizing.** β/Σ-β/ι/η/prim and the
   observational `Eq`/`cast` reductions terminate on well-typed terms:
   β/Σ-β/ι/prim strictly contract the term, and the `Eq`/`cast` mutual recursion
   descends on the *type* being traversed, which is a finite tree (`16 §3.3`).
   None of these can diverge.

2. **δ-unfolding is SCT-bounded.** The single branch of §3.2 that can grow a
   term is `Const(c)` unfolding. Every transparent `c` in `env` passed the **SCT
   gate (§4)** at admission, so each δ-unfolding sequence that re-enters a
   definition does so only along call paths on which some parameter strictly
   decreases in the well-founded structural order. By the **size-change
   termination theorem** (Lee, Jones & Ben-Amram, *The Size-Change Principle for
   Program Termination*, POPL 2001): if every idempotent loop of the call graph
   has a strictly-decreasing thread, then there is no infinite call sequence —
   so no infinite δ-unfolding chain. The environment is also append-only and
   acyclic (`11 §4`), so the call graph is exactly the recursion the gate
   analysed.

Combining (1) and (2): the `whnf` loop makes finitely many δ steps (each bounded
by SCT) interleaved with strongly-normalizing core reductions, so `whnf` halts;
`conv`/`convSpine` recurse only into structurally smaller subproblems (smaller
governing type, shorter spine, or a guaranteed-progressing δ retry), so they
halt too. Hence `convert` is total, and `check`/`infer` (`18 §3`) — which call
it at the mode switch — are total. This is soundness commitment `README.md §6`
(the SCT / decidability rows) and is exercised behaviorally by the §6 corpus
(`conversion/decidable-halts`); the metatheory status is tracked honestly in `18
§6`.

## 6. What the kernel checks here

A conforming kernel MUST: implement all §1 reductions and the §2 η + proof-
irrelevance rules as **type-directed conversion** (§3); decide level equality
(§3.6) and Ω proof-irrelevance; compare neutrals structurally with controlled δ
(§3.5); run the **SCT check** at admission (§4) and refuse transparent admission
of uncertified recursion; and **terminate** on every well-typed input (§5).

**Conformance.** `../../conformance/kernel/conversion/seed-conversion.md`
(authored with the validator). The corpus, per the K2c acceptance criteria,
covers six groups; each case states precise expected results (exact level /
exact reduct / accept-reject), exercises the *property* not just the obvious
case, and uses ≥2 distinct type or level variables and open terms where the
K1/K2 retro discipline applies:

1. **SCT-accept (the property, not the easy case).**
   `sct-accept-lexicographic` (a lexicographic/Ackermann-style descent admitted
   transparent and δ-reducing), `sct-accept-mutual` (a mutually-recursive pair —
   e.g. `even`/`odd` — exercising cross-edge matrix composition, §4.3), and
   `sct-accept-permuted` (a definition that recurses with permuted parameter
   order, exercising "decrease in *some* parameter, not the first").
2. **SCT-reject.** `sct-reject-loop` (`f x := f x`, no decrease),
   `sct-reject-growing` (`f n := f (suc n)`, a parameter grows), and
   `sct-reject-ctor-wrap` (a parameter re-wrapped in a constructor — `↓=` would
   be unsound here, §4.2) are all **rejected** at admission — the kernel never
   admits uncertified transparent recursion.
3. **δ-heavy convertibility terminates.** `delta-termination`: a query that
   forces substantial controlled δ-unfolding along a certified-terminating
   definition (e.g. `ackermann 3 3` converted against its numeral result)
   **halts** with the correct yes/no.
4. **Full η + proof irrelevance.** `pi-eta` (`f ≡ λx. f x` at a Π type, ≥2
   distinct type variables), `sigma-eta` (`p ≡ (p.1, p.2)`), `unit-eta` (any two
   `Unit` elements equal), `omega-pi` (any two proofs of `P : Ω` equal,
   re-tested through the unified §3 path), `omega-universe-not-pi`
   (`conv(Ω_0, Top, Bottom)` is **false** — distinct props are *not* equal as
   elements of the universe; proof irrelevance is for proofs *of* a prop, §3.3),
   and `prop-arg-skip` (a propositional spine argument skipped during structural
   comparison, §3.4).
5. **Obs conversions through the unified algorithm.** `cast-refl`
   (`cast A A refl a ≡ a`), `eq-by-type-funext` (`Eq ((x:A)→B) f g` reduces and
   conversion uses the result), and `quotient-eq` (`Eq (A/R) [a] [b] ≡ R a b`) —
   each decided through `conv`/`whnf` of §3, confirming K2c consumes the K2 obs
   reductions rather than re-deriving them.
6. **Decidability (meta-property).** `decidable-halts`: every conversion query
   in the corpus halts — a property of the whole runner, not a single case (§5).

The companion `../../conformance/kernel/judgments/seed-judgments.md` covers the
(Conv) mode-switch call and the `declare_def` SCT gate (`18 §3`, §4).
