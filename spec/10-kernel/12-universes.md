# Universes and the proposition classifier

> Status: **DRAFT v0**. Normative. Defines the universe hierarchy, its checking
> rules (the anchor of soundness), level polymorphism, and the proposition
> universe Ω (the subobject classifier) the verification layer builds on.

## 1. The hierarchy

Ken has a countable hierarchy of universes

```
Type 0  :  Type 1  :  Type 2  :  …
```

written `Type ℓ` for a **level** `ℓ`. Levels are the syntactic category from
`11-syntax.md §1`:

```
ℓ ::= 0 | suc ℓ | max ℓ₁ ℓ₂ | u        -- u a level variable (§4)
```

with the evident semilattice equations (`max`
associative/commutative/idempotent, `max ℓ 0 = ℓ`, `suc (max ℓ₁ ℓ₂) = max (suc
ℓ₁) (suc ℓ₂)`). Level equality is decidable and is part of conversion
(`17-conversion.md`).

**The defining rule — and the soundness anchor:**

```
  ────────────────────────  (U-Type)
  Γ ⊢ Type ℓ : Type (suc ℓ)
```

A universe lives in the *next* universe up. There is **no** `Type ℓ : Type ℓ`,
and in particular no `Type : Type`. Admitting `Type : Type` (which is what
unchecked universes amount to) makes the system inconsistent (Girard's paradox);
a conforming kernel MUST reject it. This is soundness
commitment §5.1 of `README.md` and is tested directly (G1, G5).

## 2. Predicativity

Universe levels of the connectives are **predicative**: a quantifier landing in
`Type ℓ` may only range over things in `Type ℓ` or below. The formation rules
for Π and Σ (`13-pi-sigma.md`) take the `max` of the domain and codomain levels:

```
  Γ ⊢ A : Type ℓ₁     Γ, x : A ⊢ B : Type ℓ₂
  ───────────────────────────────────────────  (Pi-Form)
  Γ ⊢ (x : A) → B : Type (max ℓ₁ ℓ₂)
```

so a function type does **not** drop to a lower universe than its parts. This
blocks the impredicative encodings that, combined with large elimination,
threaten consistency. An **impredicative** proposition universe (à la Coq's
`Prop`, where `(x : A) → P : Prop` regardless of `A`'s level) is **ruled out**
(`OQ-Prop`): it is incompatible with canonicity, and the impredicative-`Prop`
systems are not observational. Ken is **fully predicative**. Ω (§5) is a
*primitive strict, proof-irrelevant* proposition universe (`SProp`,
`16-observational.md §1`) — **predicative**, not impredicative — introduced by
the observational foundation (ADR 0005).

## 3. Cumulativity — non-cumulative (`OQ-2` DECIDED)

Ken is **non-cumulative** (`OQ-2` decided, operator 2026-06-27): `A : Type ℓ`
does *not* automatically give `A : Type (suc ℓ)`; lifting is explicit.
Non-cumulative is simpler to specify and check and keeps a **subtyping relation
out of the trusted kernel** — consistent with the small-kernel principle, with
following Lean's (non-cumulative) kernel (`17 §3`), and with the observational,
set-level foundation (ADR 0005; OTT-style systems are non-cumulative).

Cumulativity (a subtyping `Type ℓ ≤ Type ℓ'` for `ℓ ≤ ℓ'`, propagated
structurally through Π/Σ) is ergonomic — it removes a class of "universe too
low" errors — but adds a subtyping relation to the kernel and complicates
conversion *and* inference. Ken does not pay that kernel cost. Instead the
**elaborator** (`../30-surface/39-elaboration.md`, untrusted) supplies the
ergonomics: **universe polymorphism** (§4), **typical ambiguity** (write bare
`Type`, infer a consistent level), and **inserted lifts** where genuinely
needed. So the surface cost is low while the kernel stays small — Ken's
"cleverness outside, certainty inside." (Coq is the main cumulative system, with
a heavier kernel; Lean and the observational/OTT systems are non-cumulative.)

## 4. Level polymorphism

Definitions in the global environment MAY be **level-polymorphic**: a
declaration abstracts over level variables `u₁ … uₙ`, and each *use*
instantiates them.

```
Σ, (c : ∀ u₁ … uₙ. A := t)          -- a level-polymorphic definition
c {ℓ₁ … ℓₙ}                          -- a use, with explicit level arguments
```

- Level abstraction is **only** at the level of top-level declarations, not a
  first-class `Π` over `𝕃evel`. Levels are not terms; you cannot pattern-match
  on a level or store one in a data structure. This keeps the term language and
  its metatheory unchanged by level polymorphism (it is "outside" the term
  calculus, like ML let-polymorphism).
- The kernel receives **explicit** level arguments on every polymorphic use. The
  elaborator infers them (typical ambiguity / level metavariables) and emits the
  explicit form; the kernel never guesses a level.
- Level constraints from a definition body (e.g. `max u₁ u₂ ≤ u₃`) are solved by
  the elaborator and **re-checked** by the kernel as ordinary level equalities
  at each instantiation.

## 5. The proposition classifier Ω

Ken's logic lives in a distinguished object **Ω**, the **subobject classifier**
— the "type of propositions." It is where the verification layer (`../20-`) and
the surface refinement types (`{x : A | φ x}`) take their truth values.

### 5.1 Ω is the strict proposition universe

A **proposition** is an inhabitant of **Ω**, the **primitive strict,
definitionally proof-irrelevant** proposition universe (`SProp`,
`16-observational.md §1`, ADR 0005): any two proofs of a `P : Ω` are
**definitionally equal**. So propositions are **proof-irrelevant** and **UIP**
holds — Ken is **set-level**. Equality `Eq A a b` lands in Ω (`15`, `16 §2`). Ω
is the **subobject classifier**: a predicate on `A` is a map `A → Ω` (the topos
reading), and a refinement `{x : A | φ x}` requires `φ x : Ω`. Ω is
**predicative** (§2) and is *not* the impredicative `Prop` of Coq/Lean.

> **(OQ-Prop / OQ-3) — DECIDED, revised by ADR 0005.** Impredicativity stays
> **ruled out** (incompatible with canonicity; predicative Ω). The earlier call
> (cubical-era) was "derived Ω, *propositional* proof irrelevance, no `SProp`."
> The observational foundation (ADR 0005) **supersedes** that: Ω **is** the
> strict proof-irrelevant universe (`SProp`), so proof irrelevance is now
> **definitional** — and it comes *for free* in the smaller OTT kernel (it even
> *helps* agent-generated proofs: equality goals discharge definitionally, fewer
> coherence terms to synthesise). No separate `SProp` add-on or kernel growth:
> the observational core already includes it.

### 5.2 Heyting structure (intuitionistic, not Boolean)

Ω carries the operations of a **Heyting algebra**, *not* a Boolean one:

| Connective | On Ω |
|---|---|
| truth | `⊤ : Ω` (the trivially-true proposition) |
| falsity | `⊥ : Ω` (the empty type) |
| conjunction | `φ ∧ ψ` (product of props) |
| disjunction | `φ ∨ ψ` (propositional truncation of the sum, `16 §6`) |
| implication | `φ ⇒ ψ` (function type `φ → ψ`, which is a prop when `ψ` is) |
| negation | `¬ φ :≡ φ ⇒ ⊥` |
| ∀ / ∃ | dependent product / truncated dependent sum over a type |

The defining intuitionistic facts hold and MUST NOT be "optimized" into
classical ones:

- **Excluded middle `φ ∨ ¬φ` is not assumed.** It is provable only for
  *decidable* propositions (`isDecidable φ :≡ φ ∨ ¬φ` as data) — exactly the
  boundary the verification fragment classifier exploits
  (`../20-verification/23-prover.md`).
- **Double-negation elimination `¬¬φ ⇒ φ` is not assumed.** In general `¬¬φ ≠
  φ`. The gap between `φ` and `¬¬φ` is where Ken's third truth value —
  **`unknown`** — lives at the surface (`../20-verification/24-diagnostics.md
  §3`). The kernel does not have a primitive `unknown`; it has *proofs*,
  *refutations*, and the *absence* of either, and the surface renders that
  trichotomy as proved/disproved/unknown.

Classical reasoning is available only by **explicitly assuming** an axiom (an
opaque constant `lem : (φ : Ω) → φ ∨ ¬φ`), which the kernel records as a
postulate (`11-syntax.md §4`); doing so is a deliberate, visible act, never the
default.

### 5.3 Ω and the universe levels

The base `Ω` sits in `Type 1`. Propositions about larger types form a
level-polymorphic strict-prop universe `Ω ℓ : Type (suc ℓ)` (§4). The
unqualified `Ω` means `Ω 0`. Refinement types `{x : A | φ x}`
(`../30-surface/34-data-match.md`, `../20-verification/21-spec-syntax.md`)
require `φ x : Ω ℓ` for the appropriate `ℓ`.

## 6. What the kernel checks here

For the universe layer specifically, a conforming kernel MUST:

1. Implement decidable level equality and the semilattice laws (§1).
2. Enforce `Type ℓ : Type (suc ℓ)` and **reject** any derivation of `Type ℓ :
   Type ℓ` (§1).
3. Apply the predicative `max` rule at Π/Σ formation (§2, `13`).
4. Check explicit level arguments at every polymorphic instantiation and
   re-verify level constraints (§4).
5. Provide **Ω** as a **primitive strict, proof-irrelevant** proposition
   universe (`SProp`, `16-observational.md §1`): two proofs of a `P : Ω` are
   definitionally equal (definitional proof irrelevance), and Ω is predicative.
   `⊤`, `⊥`, and the Heyting operations are defined terms over Ω + the inductive
   layer (§5).

Conformance: `../../conformance/kernel/universes/` — includes the `Type:Type`
rejection, predicative-`max` formation, level-poly instantiation, and
definitional proof irrelevance in Ω.
