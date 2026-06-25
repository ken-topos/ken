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
and in particular no `Type : Type`. Reproducing `Type : Type` (as the
prototype's unchecked universes effectively did) makes the system inconsistent
(Girard's paradox); a conforming kernel MUST reject it. This is soundness
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
threaten consistency. Whether to add an **impredicative** proposition universe
(à la Coq's `Prop`, where `(x : A) → P : Prop` regardless of `A`'s level) is
**OQ-3 / OQ-Prop**: it buys convenience for the logic but complicates the
soundness argument and interacts subtly with the cubical layer. The DRAFT
baseline is **fully predicative**, with Ω a *defined* sub-universe (§5), not a
primitive impredicative sort.

## 3. Cumulativity (OQ-2)

The DRAFT baseline is **non-cumulative**: `A : Type ℓ` does *not* automatically
give `A : Type (suc ℓ)`; lifting is explicit. Non-cumulative is simpler to
specify and check and is friendlier to a small kernel.

Cumulativity (a subtyping `Type ℓ ≤ Type ℓ'` for `ℓ ≤ ℓ'`, propagated
structurally through Π/Σ) is ergonomic — it removes a class of "universe too
low" errors that the elaborator otherwise resolves by inserting lifts — but adds
a subtyping relation to the kernel and complicates conversion. **OQ-2** records
the choice; if cumulativity is rejected at the kernel, the elaborator
(`../30-surface/39-elaboration.md`) hides level bookkeeping from the programmer
via universe polymorphism (§4) and inserted lifts, so the surface cost is low
either way.

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

### 5.1 Propositions are mere propositions

A **proposition** is a type with **at most one inhabitant up to the path
equality** — a *mere proposition* (h-proposition):

```
isProp A  :≡  (x y : A) → Path A x y
```

`Ω` is the type of mere propositions at the base level:

```
Ω  :≡  (A : Type 0) × isProp A     -- a type bundled with a proof it's a prop
```

with the first projection `⟨A, _⟩ ↦ A` coercing a proposition to its underlying
type (so "having a proof `p : P`" and "`P` holds" coincide). Defining
propositions as mere (proof-irrelevant up to path) rather than as a primitive
impredicative sort keeps the kernel uniform: Ω is *derived* from the universe +
identity layers, not a new primitive. This is the HoTT presentation of the
subobject classifier and matches the topos reading "a predicate on `A` is a map
`A → Ω`."

> **(OQ-Prop / OQ-3)** Alternative: a primitive impredicative `Prop`
> (definitional proof irrelevance, impredicative quantification). Faster surface
> logic, heavier kernel and metatheory. The DRAFT uses derived Ω; revisit if the
> verification layer needs impredicativity.

### 5.2 Heyting structure (intuitionistic, not Boolean)

Ω carries the operations of a **Heyting algebra**, *not* a Boolean one:

| Connective | On Ω |
|---|---|
| truth | `⊤ : Ω` (the unit type, `isProp` trivially) |
| falsity | `⊥ : Ω` (the empty type) |
| conjunction | `φ ∧ ψ` (product of props) |
| disjunction | `φ ∨ ψ` (propositional truncation of the sum, `16-cubical.md §HIT`) |
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
  §three-region`). The kernel does not have a primitive `unknown`; it has
  *proofs*, *refutations*, and the *absence* of either, and the surface renders
  that trichotomy as proved/disproved/unknown.

Classical reasoning is available only by **explicitly assuming** an axiom (an
opaque constant `lem : (φ : Ω) → φ ∨ ¬φ`), which the kernel records as a
postulate (`11-syntax.md §4`); doing so is a deliberate, visible act, never the
default.

### 5.3 Ω and the universe levels

`Ω` as defined sits in `Type 1` (it quantifies over `Type 0`). Propositions
about larger types form `Ω ℓ :≡ (A : Type ℓ) × isProp A : Type (suc ℓ)`, a
level-polymorphic family (§4). The unqualified `Ω` means `Ω 0`. Refinement types
`{x : A | φ x}` (`../30-surface/34-data-match.md`,
`../20-verification/21-spec-syntax.md`) require `φ x : Ω ℓ` for the appropriate
`ℓ`.

## 6. What the kernel checks here

For the universe layer specifically, a conforming kernel MUST:

1. Implement decidable level equality and the semilattice laws (§1).
2. Enforce `Type ℓ : Type (suc ℓ)` and **reject** any derivation of `Type ℓ :
   Type ℓ` (§1).
3. Apply the predicative `max` rule at Π/Σ formation (§2, `13`).
4. Check explicit level arguments at every polymorphic instantiation and
   re-verify level constraints (§4).
5. Treat `isProp`, `Ω`, `⊤`, `⊥`, and the Heyting operations as **ordinary
   defined terms** over the identity/inductive/cubical layers — Ω requires no
   new kernel primitive (§5). Only if OQ-Prop selects a primitive `Prop` does
   the kernel gain a sort here.

Conformance: `../../conformance/kernel/universes/` — includes the `Type:Type`
rejection, predicative-`max` formation, level-poly instantiation, and the
`isProp`/Ω constructions.
