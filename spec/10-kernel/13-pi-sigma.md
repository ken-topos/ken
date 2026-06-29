# Dependent functions (Π) and pairs (Σ)

> Status: **K1 elaborated**. Normative. Formation, introduction, elimination,
> computation (β), and η for the two core dependent connectives. Σ here is
> **genuinely dependent** (`README.md §7`). §6 adds the K1-scoped conversion
> algorithm and subject reduction — structured for K2c (`17`) to extend with
> NbE later.

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
point**; Ken requires it.

Ken presents Σ **negatively**, by its projections, which yields a definitional η
(the Σ-η rule below). (An equivalent positive presentation with a dependent
eliminator is derivable; see `14-inductive.md §4`.)

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
giving field-update and reconstruction their expected definitional behaviour. (η
is the **record/Σ** knob; `data` declarations do not get it — `OQ-η-records`,
`14 §4`.)

## 4. Interaction with the rest of the kernel

- **Universes.** Both formation rules use predicative `max` (`12 §2`); neither
  drops a level. A `Π`/`Σ` whose codomain is a proposition lands in the strict-
  prop universe Ω (`12 §5`); impredicative `Prop` is ruled out (`OQ-Prop`).
- **Conversion.** β, the projection-β rules, and both η rules are part of
  definitional equality (`17`). η for Π and Σ is what makes conversion *typed*
  (η-expansion is driven by the type), so the conversion algorithm needs the
  type at η-points (`17 §2`).
- **Equality/transport.** `cast` in a Π- or Σ-type computes structurally
  (pushing into domain/codomain and components); the rules are in
  `16-observational.md §3`. `Eq ((x:A)×B) p q` reduces componentwise (`16 §2`),
  which the surface uses for record equality.
- **Functions out of data.** Defining a function by cases on an inductive
  argument is `elim_D` (`14`), not a Π primitive; `λ` introduces Π, `elim`
  consumes data.

## 5. What the kernel checks here

A conforming kernel MUST implement Π and Σ with: the predicative formation
rules; β and projection-β as reductions; and **both η rules** in conversion. It
MUST type `p.2` at `B[p.1/x]` (dependent second projection) and MUST reject a
non-dependent shortcut that ignores the dependency. Conformance:
`../../conformance/kernel/pi-sigma/` — includes dependent-`p.2` typing, Π-η (`f
≡ λx.f x`), Σ-η (`p ≡ (p.1,p.2)`), and a regression that a genuinely dependent
`B` (e.g. `(n : Nat) × Vec A n`) type-checks and projects correctly.

## 6. K1 conversion and subject reduction

K1 implements **basic structural conversion** — the β/η reductions of Π and Σ
(plus ι from `14` and δ-unfolding from `11 §4`) — sufficient for `check`/`infer`
to decide the K1 fragment. The full decidable conversion (lazy-WHNF NbE, the
`Eq`/`cast` equations, Ω proof irrelevance, and SCT termination gating δ) is
K2c (`17-conversion.md`). This section defines the K1 algorithm and the
extension point for K2c.

### 6.1 Reduction relation (K1 fragment)

The K1 reduction relation `t ⇝ t'` is the compatible closure of:

```
(λ (x : A). t) a          ⇝  t[a/x]                    (Π-β)
(a , b).1                 ⇝  a                         (Σ-β₁)
(a , b).2                 ⇝  b                         (Σ-β₂)
elim_D M m̄ i̅ (cₖ ā)       ⇝  mₖ ā [recursive calls]    (D-ι, 14 §3)
c                         ⇝  t   (if c : A := t in Σ)  (δ)
```

Reductions apply in any context: if `t ⇝ t'` then `t u ⇝ t' u`, `λx.t ⇝ λx.t'`,
`(t,u) ⇝ (t',u)`, etc. (compatible closure).

### 6.2 Conversion judgment (K1-scoped)

Definitional equality `Γ ⊢ a ≡ b : A` for the K1 fragment is decided by:

1. **α-equivalence**, which is syntactic identity under de Bruijn indices
   (`11 §2`): if `a` and `b` are syntactically identical (modulo α-renaming, which
   de Bruijn makes trivial) then `Γ ⊢ a ≡ b : A`.

2. **Reduction to normal form**: reduce both `a` and `b` to their normal forms
   using a leftmost-outermost reduction strategy (which terminates on K1 terms,
   `14-inductive.md §9.2`; normal forms are unique by confluence). If the two
   normal forms are equal up to α and type-directed η (step 3), then
   `Γ ⊢ a ≡ b : A`.

3. **η-expansion (type-directed).** When β-reduction alone does not identify
   the terms, the conversion algorithm inspects the type `A`:

   - **Π-η.** If `A` is `(x : A₁) → B`, compare `f` and `g` by applying both to
     a fresh variable `x` (extending the context): `Γ, x : A₁ ⊢ f x ≡ g x : B`.
     This is equivalent to reducing both sides to a common reduct if η-reduction
     (`f ⇝ λx.f x`) is included, but the expansion form avoids η-redex creation
     in the implementation.

   - **Σ-η.** If `A` is `(x : A₁) × B`, compare `p` and `q` by projecting both
     sides: `Γ ⊢ p.1 ≡ q.1 : A₁` and `Γ ⊢ p.2 ≡ q.2 : B[p.1/x]`. This is
     equivalent to surjective-pairing η-reduction.

4. **Structural congruence.** Conversion is a congruence: if `Γ ⊢ a ≡ b : A` then
   `Γ ⊢ λx.a ≡ λx.b : (x:A)→B`, `Γ ⊢ (a,c) ≡ (b,d) : (x:A)×B`, `Γ ⊢ a c ≡ b d :
   B[c/x]`, etc.

The algorithm is **syntax-directed and structurally recursive** on the type `A`
for η, and on the terms themselves for β-reduction. The K1 fragment has no
recursive definitions (acyclic δ only) and no `Eq`/`cast` equations, so
conversion terminates by structural decrease.

### 6.3 Extension point for K2c

K2c (`17-conversion.md`) replaces the K1 reduction-based conversion with
**lazy-WHNF NbE**: terms are evaluated to a normal form in a semantic domain,
and conversion at a type becomes a structural recursion over the type (with
*read-back* from the semantic domain). The K1 algorithm above is the "obvious
implementation"; the K2c algorithm is the **optimised, decidable-for-the-full-
calculus** replacement. The K1 conversion checker MUST be structured so that:

- The conversion entry point `convert : Γ → A → t → u → bool` is a standalone
  function the rest of K1 calls.
- K2c replaces its body without changing the signature or the rest of K1.

### 6.4 Subject reduction (K1 fragment)

**Theorem (K1 subject reduction).** If `Γ ⊢ t : A` (under ambient `Σ`, in the
K1 fragment — Π, Σ, universes, inductive families, and their eliminators) and
`t ⇝ t'` (by any of Π-β, Σ-β₁, Σ-β₂, D-ι, or δ), then `Γ ⊢ t' : A`.

*Proof sketch.* By induction on the typing derivation `Γ ⊢ t : A`, with case
analysis on the reduction step.

- **Π-β**: `(λ (x : A). t) a ⇝ t[a/x]`. The typing derivation ends with Π-Elim
  applied to Π-Intro. By inversion: `Γ, x : A ⊢ t : B` and `Γ ⊢ a : A`. By the
  substitution lemma (typing is closed under well-typed substitution, `11 §5`),
  `Γ ⊢ t[a/x] : B[a/x]`, which is exactly the type required.
- **Σ-β₁**: `(a , b).1 ⇝ a`. The derivation ends with Σ-Elim₁ applied to
  Σ-Intro. By inversion `Γ ⊢ a : A`. The type of the projection is `A`, so
  `Γ ⊢ a : A` matches.
- **Σ-β₂**: `(a , b).2 ⇝ b`. Similar — inversion gives `Γ ⊢ b : B[a/x]`, which
  is the projection's result type.
- **D-ι**: `elim_D M m̄ i̅ (cₖ ā) ⇝ mₖ ā [induction hypotheses]`. The typing
  derivation ends with the eliminator rule (see `14-inductive.md §7`). By
  construction of the eliminator type and the method types, the reduct has the
  required type (see `14-inductive.md §7` for the full argument).
- **δ**: `c ⇝ t` where `c : A := t` in Σ. By definition, `c` has type `A` and
  `t` was admitted at type `A`; unfolding preserves the type.

The substitution lemma — that `Γ ⊢ t[u/x] : B[u/x]` when `Γ ⊢ u : A` and
`Γ, x : A ⊢ t : B` — holds by induction on `t`'s structure (standard, `11 §5`).

The conformance test suite MUST include a property test that generates random
well-typed K1 terms, reduces them by one β/η/ι/δ step, and verifies the reduct
has the same type (up to conversion) in a fresh typing derivation.
