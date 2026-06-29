# Universes and the proposition classifier

> Status: **K1 elaborated**. Normative. Defines the universe hierarchy, its
> checking rules (the anchor of soundness), and level polymorphism. The
> proposition universe Ω is a reserved grammar former (see `11-syntax.md §1`);
> its typing rules, proof irrelevance, and Heyting structure are defined in K2
> (`16-observational.md`). K1 delivers §§1–4 and §6.

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

## 5. The proposition classifier Ω (K2)

`Ω` is a **reserved grammar former** (`11-syntax.md §1`, tagged `[K2]`). It is
the strict, definitionally proof-irrelevant proposition universe (`SProp`) —
the subobject classifier where `Eq` and the logic live.

K1 reserves `Ω` in the grammar (it parses and is raw-well-formed) but
implements **none** of its typing rules, proof irrelevance, or conversion
behaviour. Those are defined in K2:

- **Typing of Ω, its inhabitants, and the Heyting structure** —
  `16-observational.md §1`.
- **Definitional proof irrelevance** (any two proofs of `P : Ω` are
  definitionally equal) — `16-observational.md §1`.
- **Interaction with `Eq` and `cast`** — `Eq A a b : Ω` (`15-identity.md`),
  `16-observational.md §2`.
- **Level-polymorphic Ω** — `16-observational.md §1`.

Until K2 is delivered, the kernel's `check`/`infer` treat `Ω` as an
unrecognised former (it fails typing).

## 6. What the kernel checks here

For the universe layer specifically, a conforming kernel MUST:

1. Implement decidable level equality and the semilattice laws (§1).
2. Enforce `Type ℓ : Type (suc ℓ)` and **reject** any derivation of `Type ℓ :
   Type ℓ` (§1).
3. Apply the predicative `max` rule at Π/Σ formation (§2, `13`).
4. Check explicit level arguments at every polymorphic instantiation and
   re-verify level constraints (§4).
5. Reserve **Ω** as a grammar former (raw-well-formed, `11-syntax.md §1`).
   Ω's typing rules, definitional proof irrelevance, and Heyting structure
   are K2 (`16-observational.md`). K1's `check`/`infer` reject Ω terms as
   unrecognised.

Conformance: `../../conformance/kernel/universes/` — K1 tests: the `Type:Type`
rejection, predicative-`max` formation, level-poly instantiation. Ω conformance
tests are K2-gated.
