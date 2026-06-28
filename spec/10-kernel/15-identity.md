# Identity, equality, and `J`

> Status: **DRAFT v0**. Normative. Ken's propositional equality is
> **observational equality `Eq`** (ADR 0005) — not an inductive `Id`, not a
> cubical `Path`. This chapter is the identity *interface* (`Eq`, `refl`,
> `cast`, `J` and its computation, the derived equalities); the *machinery* it
> computes by — `Eq`-by- type, `cast`, Ω, quotients — is in
> `16-observational.md`.

## 1. Why observational equality

Martin-Löf type theory makes equality an inductive `Id A a b` whose eliminator
`J` **only computes on `refl`**. Cubical type theory fixes that (via the
interval) but at the cost of the largest, most canonicity-fragile part of a
kernel, and it provides univalence + higher structure that *software* does not
use.

Ken takes equality to be **observational** (`TTobs`/`CICobs`, ADR 0005): `Eq A a
b` is a **proposition computed by recursion on the type `A`**, and a `cast`
coercion transports along type-equalities and **reduces on reflexivity**. From
`cast`, `J` is derived and **reduces on any equality, not only `refl`**, while
the kernel stays small and **set-level** (UIP holds), which is exactly what
software data is. `K1`'s required "`Id`, `J`"
is satisfied by `Eq` + the derived `J` (§4); `Id A a b` and `a = b` MAY be
surface synonyms for `Eq A a b`.

## 2. The equality type

**Formation.**
```
  Γ ⊢ A : Type ℓ      Γ ⊢ a : A      Γ ⊢ b : A
  ───────────────────────────────────────────────  (Eq-Form)
  Γ ⊢ Eq A a b : Ω
```
`Eq A a b` is a **proposition** (`Ω`, `16 §1`), so it is **proof-irrelevant**:
any two proofs are definitionally equal (no coherence baggage; `16 §1`).

**Reflexivity.** `refl a : Eq A a a`.

**Computation.** `Eq A a b` reduces *by recursion on `A`* (`16 §2`): pointwise
at a Π-type (so **funext is definitional**), componentwise at a Σ-type,
structurally at an inductive, to the user relation at a quotient, to mutual
implication at Ω (**propext**), and to literal equality at a primitive. On a
neutral `A` it is a neutral proposition.

## 3. Transport — `cast` / `subst`

The operation that makes equality *do* something is **`cast`** (`16 §3`):
transporting along a type-equality, with `cast A A refl a ≡ a` (regularity) and
`cast`-by-type computation. For a family `P : A → Type ℓ` and `e : Eq A a b`,
substitution is

```
  subst P e  :≡  cast (P a) (P b) (cong P e)  :  P a → P b
```

`subst P (refl a) ≡ id`, and on a non-trivial equality `subst`/`cast` reduce by
recursion on `P`/the type (`16 §3`) — this is why everything below computes.

## 4. `J` (path induction) and its computation rule

`J` is the eliminator: to prove a property `P` of "any equality out of `a`",
prove it for `refl`.

**Type.**
```
  J : (A : Type ℓ) (a : A)
      (P : (b : A) → Eq A a b → Type ℓ')
      (d : P a (refl a))
      (b : A) (e : Eq A a b)
    → P b e
```

**Definition (derived from `cast`).** `J` transports the base case `d` along `e`
using `cast` (`16 §3`); since `Eq` is proof-irrelevant (`16 §1`), the equality
proof itself carries no content, so `J` is determined by the endpoints. The
kernel provides `J` as a defined operation with the rules below; it need not be
primitive.

**Computation on `refl` (β).**
```
  J A a P d a (refl a)  ≡  d  :  P a (refl a)            (J-β)
```

**Computation on a non-`refl` equality (the correction).** Because `J` is built
from `cast`, when its equality argument is a *non-`refl`* but otherwise
canonical proof — e.g. one produced by `subst`, by a constructor's congruence,
or by a quotient relation — `J` **reduces** by the corresponding `cast`
computation rather than getting stuck. A conforming kernel MUST exhibit this: a
conformance test (`../../conformance/kernel/observational/j-nonrefl`) in which
`J` on a non-`refl` equality computes to a constructor form, **failing on any
kernel that only reduces `J` on `refl`.** This is achieved via observational
equality rather than cubical paths.

## 5. Derived equalities (theorems, mostly definitional)

The observational presentation makes the equalities classical MLTT must
axiomatise into **definitional or computing** facts (`16 §4`):

- **Function extensionality (funext)** — *definitional*: `Eq` at a Π-type **is**
  pointwise `Eq` (`16 §2`). Two functions are equal iff equal at every argument,
  with no axiom — a major win for the verification layer, which constantly needs
  extensional function equality.
- **Propositional extensionality (propext)** — *definitional*: equal
  propositions are mutually-implying (`16 §2`).
- **UIP / proof irrelevance** — *definitional*: `Eq : Ω` (`16 §1`). Ken is
  set-level: there is no nontrivial `Eq (Eq A a b) p q`.
- **`sym`, `trans`, `cong`, `subst`** — derivable and computing (`16 §4`).

## 6. No univalence (set-level, by design)

Ken has **no univalence** (`(A ≃ B) → Eq Type A B`) and no higher-dimensional
structure — these are cubical/HoTT features for *mathematics*, deliberately
absent (ADR 0005, `16 §7`). Type-equality `Eq Type A B` is **structural** (`16
§3`): same head former with equal parts. So you cannot transport a program
across an arbitrary *equivalence* of types; you transport across *structural*
type-equalities (which covers reindexing, parameter equalities, and the like —
what software needs). The generic-programming-via-univalence idiom is the one
thing given up, and for set-level software it is a non-loss.

## 7. What the kernel checks here

A conforming kernel MUST: form `Eq A a b : Ω`; compute `Eq`-by-type (`16 §2`)
including **definitional funext and propext**; provide `cast`/`subst` with
**`cast`-refl** and by-type computation (`16 §3`); provide `J` with **both**
`J-β` (on `refl`) **and** reduction on non-`refl` equalities via `cast` (§4);
and derive `sym`/`trans`/`cong` and **definitional UIP** (`Eq : Ω`). The
non-`refl` `J` computation and definitional funext are required,
separately-tested behaviours. Conformance:
`../../conformance/kernel/observational/`.
