# Identity, paths, and `J`

> Status: **DRAFT v0**. Normative. Ken's propositional equality is the cubical
> **`Path`** type. This chapter fixes `Path`, `refl`, path application, the `J`
> eliminator and **its computation rule on non-`refl` paths**, and the derived
> equalities (funext, Σ/Π paths). The interval and the composition operators `J`
> is built from are in `16-cubical.md`; this is the identity *interface*.

## 1. Why `Path`, not an inductive `Id`

Martin-Löf type theory makes equality an inductive type `Id A a b` with a single
constructor `refl` and the eliminator `J`. That works, but `J` only *computes*
when its scrutinee is literally `refl`; on a neutral path it is stuck. The
prototype inherited exactly this limitation (`J` on non-`refl` did not reduce),
which is one of the three soundness/ergonomics gaps Ken corrects by construction
(`README.md §6`).

Ken instead takes equality to be the **cubical path type** `Path A a b`: a path
is a function out of an abstract **interval** `𝕀` (`16 §interval`) with fixed
endpoints. Because transport and composition over the interval *compute* (`16
§comp`), `J` is **derived** and **reduces on any path**, not only on `refl`.
This is the single most important design choice in the identity layer.

`K1`'s required "`Id`, `J`" is satisfied by `Path` + the derived `J` (§4); the
name `Id A a b` MAY be provided as a surface synonym for `Path A a b`.

## 2. The path type

**Formation.**
```
  Γ ⊢ A : Type ℓ      Γ ⊢ a : A      Γ ⊢ b : A
  ───────────────────────────────────────────────  (Path-Form)
  Γ ⊢ Path A a b : Type ℓ
```
`Path A a b` is the type of paths in `A` from `a` to `b`. (A heterogeneous /
dependent path `PathP (⟨i⟩ A) a b`, over a *line of types* `A`, is the primitive
form; `Path A a b :≡ PathP (⟨i⟩ A) a b` for a constant line. `PathP` is in `16
§PathP` and is needed for paths between elements of different-but-equal types;
this chapter uses the non-dependent `Path` except where noted.)

**Introduction — path abstraction.**
```
  Γ, i : 𝕀 ⊢ t : A      Γ ⊢ t[0/i] ≡ a : A      Γ ⊢ t[1/i] ≡ b : A
  ──────────────────────────────────────────────────────────  (Path-Intro)
  Γ ⊢ ⟨i⟩ t : Path A a b
```
A path is an interval-indexed term whose two endpoints are *definitionally* `a`
and `b`. The endpoint conditions are **boundary** conditions checked by
conversion.

**Reflexivity.**
```
  refl a  :≡  ⟨i⟩ a   :   Path A a a            (the constant path)
```

**Elimination — path application.**
```
  Γ ⊢ p : Path A a b      Γ ⊢ r : 𝕀
  ─────────────────────────────────────  (Path-Elim)
  Γ ⊢ p @ r : A
```
with the **definitional** boundary computations
```
  p @ 0  ≡  a        p @ 1  ≡  b        (⟨i⟩ t) @ r  ≡  t[r/i]    (Path-β)
```
So applying a path at the endpoints yields the endpoints *by computation*, and
applying an abstraction substitutes into the body. Path application at an
interior point `r` is the value "partway along" the path.

**Uniqueness (η).**
```
  Γ ⊢ p : Path A a b
  ──────────────────────────────────  (Path-η)
  Γ ⊢ p  ≡  ⟨i⟩ (p @ i)  :  Path A a b
```

## 3. Transport (the computational core)

The operation that makes paths *do* something is **transport**: a path between
types (or a type-family applied along a path) lets you move an inhabitant from
one end to the other.

```
  transp (⟨i⟩ A) 0 a   moves  a : A[0/i]   to   A[1/i]          (16 §transp)
```

For a family `P : A → Type ℓ` and a path `p : Path A a b`, transport along `p`
is

```
  transport P p  :≡  λ x. transp (⟨i⟩ P (p @ i)) 0 x   :   P a → P b
```

The defining computation: transport along `refl` is the identity, up to the
family's structure,

```
  transport P (refl a)  ≡  λ x. x        (regularity; 16 §regularity)
```

and on a *non-trivial* path `transp` reduces **by recursion on the type `A`**
(`16 §transp-by-type`) — pushing through Π, Σ, inductive families, `Path`,
`Glue`, etc. This is why everything below computes.

## 4. `J` (path induction) and its computation rule

`J` is the dependent eliminator for `Path`: to prove a property `P` of "any path
out of `a`," it suffices to prove it for `refl`.

**Type.**
```
  J : (A : Type ℓ) (a : A)
      (P : (b : A) → Path A a b → Type ℓ')
      (d : P a (refl a))
      (b : A) (p : Path A a b)
    → P b p
```

**Definition (derived from transport).** `J` transports the base case `d` along
the path `p`, viewing `(b, p)` as moving within the total space of paths out of
`a` (the based-path space, which is contractible). Concretely it is a `transp`
over the line `⟨i⟩ P (p @ i) (⟨j⟩ p @ (i ∧ j))` (a `comp`; `16 §comp`). The
kernel provides `J` as a defined operation with the computation rule below; it
need not be a separate primitive.

**Computation on `refl` (the β-rule for `J`).**
```
  J A a P d a (refl a)  ≡  d  :  P a (refl a)            (J-β)
```

**Computation on a non-`refl` path (the correction).** Because `J` is built from
`transp`/`comp`, when its path argument is a *non-`refl`* but otherwise
canonical path — e.g. a path produced by `transport`, by a constructor's
congruence, by `Glue`/univalence, or by a HIT path constructor — `J` **reduces**
by the corresponding `transp`/`comp` computation rather than getting stuck. A
conforming kernel MUST exhibit this: there is a conformance test
(`../../conformance/kernel/identity/j-nonrefl`) in which `J` applied to a
non-`refl` path computes to a constructor form, **failing on any kernel that
only reduces `J` on `refl`.** This directly encodes the corrected behaviour.

## 5. Derived equalities (theorems, not axioms)

The cubical presentation makes several equalities that classical MLTT must take
as axioms (or do without) into **provable, computing** theorems:

- **`refl`, symmetry, transitivity, congruence** — `sym p :≡ ⟨i⟩ p @ (~ i)`;
  `cong f p :≡ ⟨i⟩ f (p @ i)`; transitivity is a `comp` (`16`). All compute.
- **Function extensionality (funext).** A path in a Π-type is pointwise:
  ```
  funext : ((x : A) → Path (B x) (f x) (g x)) → Path ((x:A)→B x) f g
  funext h  :≡  ⟨i⟩ λ x. h x @ i
  ```
  funext is *definitional structure*, not an axiom — a major ergonomic win for
  the verification layer, where extensional function equality is constantly
  needed.
- **Σ-paths.** A path in `(x:A)×B` is a pair of (dependent) paths: `Path
  ((x:A)×B) p q ≃ (e : Path A p.1 q.1) × PathP (⟨i⟩ B (e@i)) p.2 q.2`. Record
  equality is therefore componentwise (`../30-surface/34-data-match.md`).
- **`transport`/`subst`.** `subst P p : P a → P b` is `transport P p`; rewriting
  along an equality is transport, and it computes.
- **`isProp`, `isSet`, h-levels.** Defined as in `12 §5` over `Path`; truncation
  levels are expressible, and `Ω` = mere-props is built here.

## 6. Univalence (stated here, built in `16`)

For types `A B : Type ℓ`, an **equivalence** `A ≃ B` (`16 §isEquiv`) yields a
**path** `ua : A ≃ B → Path (Type ℓ) A B`, and transporting along `ua e`
computes to applying the equivalence:

```
  transport (λ X. X) (ua e)  ≡  e.fun        (up to Glue computation; 16 §ua)
```

Univalence is thus a *computing* operation in Ken, not a postulate — "equal
types are interchangeable, and the interchange runs." The construction (via
`Glue`) and its precise computation are in `16-cubical.md §univalence`. The
verification layer relies on univalence + funext to make structurally-equal data
definitionally interchangeable for proof purposes.

## 7. What the kernel checks here

A conforming kernel MUST: form `Path`/`PathP`; check path-abstraction **boundary
conditions** by conversion; compute `Path-β` (endpoints and `@`-substitution)
and `Path-η`; provide `J` with **both** `J-β` (on `refl`) **and** reduction on
non-`refl` paths via `transp`/`comp` (§4); and derive funext, `sym`, `cong`,
`subst`, and the Σ/Π path characterizations. The non-`refl` `J` computation is a
required, separately-tested behaviour (§4). Conformance:
`../../conformance/kernel/identity/`.
