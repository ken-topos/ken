# Dependent functions (Π) and pairs (Σ)

> Status: **DRAFT v0**. Normative. Formation, introduction, elimination,
> computation (β/ι), and η for the two core dependent connectives. Σ here is
> **genuinely dependent** — the prototype's non-dependent Σ is corrected by
> construction (`README.md §6`).

Notation: `Γ ⊢ t : A` typing; `Γ ⊢ a ≡ b : A` definitional equality (`17`);
`t[u/x]` capture-avoiding substitution (`11 §5`). Premises above the line,
conclusion below, side conditions at right.

## 1. Dependent functions — Π

The type `(x : A) → B` classifies functions whose result *type* `B` may depend
on the argument `x`. When `B` does not mention `x` it is the ordinary arrow `A →
B`.

**Formation.**
```
  Γ ⊢ A : Type ℓ₁      Γ, x : A ⊢ B : Type ℓ₂
  ─────────────────────────────────────────────  (Π-Form)
  Γ ⊢ (x : A) → B : Type (max ℓ₁ ℓ₂)
```
Predicative `max` (`12 §2`).

**Introduction.**
```
  Γ, x : A ⊢ t : B
  ───────────────────────────────  (Π-Intro)
  Γ ⊢ λ (x : A). t : (x : A) → B
```

**Elimination (application).**
```
  Γ ⊢ f : (x : A) → B      Γ ⊢ a : A
  ─────────────────────────────────────  (Π-Elim)
  Γ ⊢ f a : B[a/x]
```

**Computation (β).**
```
  Γ ⊢ (λ (x : A). t) a  ≡  t[a/x]  :  B[a/x]            (Π-β)
```

**Uniqueness (η).**
```
  Γ ⊢ f : (x : A) → B
  ─────────────────────────────────────  (Π-η)
  Γ ⊢ f  ≡  λ (x : A). f x  :  (x : A) → B
```

η is a *definitional* equality the kernel's conversion checker MUST implement
(`17 §η`): two functions are convertible iff they agree on a fresh variable.
This makes, e.g., `λ x. f x ≡ f` hold without a proof.

## 2. Dependent pairs — Σ

The type `(x : A) × B` classifies pairs `(a, b)` where the *type* of the second
component, `B`, may depend on the first component `a`. When `B` does not mention
`x` it is the ordinary product `A × B`. **`B` mentioning `x` is the whole
point** and is what the prototype lacked; Ken requires it.

Ken presents Σ **negatively**, by its projections, which yields a definitional η
(§2.4). (An equivalent positive presentation with a dependent eliminator is
derivable; see `14-inductive.md §Σ-as-record`.)

**Formation.**
```
  Γ ⊢ A : Type ℓ₁      Γ, x : A ⊢ B : Type ℓ₂
  ─────────────────────────────────────────────  (Σ-Form)
  Γ ⊢ (x : A) × B : Type (max ℓ₁ ℓ₂)
```

**Introduction.**
```
  Γ ⊢ a : A      Γ ⊢ b : B[a/x]
  ─────────────────────────────────  (Σ-Intro)
  Γ ⊢ (a , b) : (x : A) × B
```
The second component is checked at `B[a/x]` — the dependency made explicit.

**Elimination (projections).**
```
  Γ ⊢ p : (x : A) × B                Γ ⊢ p : (x : A) × B
  ───────────────────────            ─────────────────────────  (Σ-Elim)
  Γ ⊢ p.1 : A                        Γ ⊢ p.2 : B[p.1 / x]
```
The type of `p.2` is `B` with the **first projection of the same pair**
substituted for `x`. This is well-typed precisely because Σ is dependent.

**Computation (projection β).**
```
  Γ ⊢ (a , b).1  ≡  a  :  A                              (Σ-β₁)
  Γ ⊢ (a , b).2  ≡  b  :  B[a/x]                         (Σ-β₂)
```

**Uniqueness (η).**
```
  Γ ⊢ p : (x : A) × B
  ─────────────────────────────────────  (Σ-η)
  Γ ⊢ p  ≡  (p.1 , p.2)  :  (x : A) × B
```

Surjective-pairing η is definitional and MUST be implemented by conversion. With
it, a one-field record and its projection round-trip definitionally, and Σ
behaves as a proper negative/record type.

## 3. Telescopes and n-ary forms

Iterated Π and Σ over a **telescope** `Δ = (x₁ : A₁) … (xₙ : Aₙ)` are written
`(Δ) → B` and `Σ Δ. B` and desugar right-associatively to the binary forms. The
kernel has only the binary forms; the elaborator
(`../30-surface/39-elaboration.md`) expands telescopes. Records with named
fields (`../30-surface/33-declarations.md`) elaborate to right-nested Σ with η
giving field-update and reconstruction their expected definitional behaviour.

## 4. Interaction with the rest of the kernel

- **Universes.** Both formation rules use predicative `max` (`12 §2`); neither
  drops a level. Under OQ-Prop's impredicative `Prop`, only `Π` into `Prop`
  would change (`12 §2`).
- **Conversion.** β, the projection-β rules, and both η rules are part of
  definitional equality (`17`). η for Π and Σ is what makes conversion *typed*
  (η-expansion is driven by the type), so the conversion algorithm needs the
  type at η-points (`17 §algorithm`).
- **Identity/transport.** Transport in a Π- or Σ-type computes structurally
  (`transp` pushes into domain/codomain and components); the rules are in
  `16-cubical.md §transp-by-type`. `Path ((x:A)×B) p q` is equivalent to a pair
  of paths (`15 §Σ-paths`), which the surface uses for record equality.
- **Functions out of data.** Defining a function by cases on an inductive
  argument is `elim_D` (`14`), not a Π primitive; `λ` introduces Π, `elim`
  consumes data.

## 5. What the kernel checks here

A conforming kernel MUST implement Π and Σ with: the predicative formation
rules; β and projection-β as reductions (`17`); and **both η rules** in
conversion. It MUST type `p.2` at `B[p.1/x]` (dependent second projection) and
MUST reject a non-dependent shortcut that ignores the dependency. Conformance:
`../../conformance/kernel/pi-sigma/` — includes dependent-`p.2` typing, Π-η (`f
≡ λx.f x`), Σ-η (`p ≡ (p.1,p.2)`), and a regression that a genuinely dependent
`B` (e.g. `(n : Nat) × Vec A n`) type-checks and projects correctly.
