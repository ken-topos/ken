# `transport` — `subst`, `cong`, `cast`, `sym`, `trans`

Five small, non-recursive wrappers over the kernel's `J` equality eliminator
— transporting a family along an equality, lifting equality through a
function, coercing a value along a type-level equality, and the equality
algebra (`sym`/`trans`) every later proof leans on.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust  derivation](#7-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* →
  [Design notes](#5-design-notes) → [Trust  derivation](#7-trust--derivation)

## 1. Motivation

Ken has a single surface equality eliminator, `J`, but every proof that
actually needs to transport a value,
flip a hypothesis, or compose two equalities has to hand-write its own `J`
application and pick the right motive — easy to get subtly wrong (the wrong
motive shape is a common source of stuck proofs) and repetitive across the
catalog. This package names the five recurring shapes once, as ordinary
checked Ken, so later entries call `subst`/`cong`/`cast`/`sym`/`trans`
instead of re-deriving them.

## 2. Definition

Each combinator is a thin, non-recursive wrapper over the single surface
former `J`. The endpoint proof decides the motive branch; no package-local
helper or recursive proof is needed.

**Naming note.** The surface grammar treats capitalized identifiers as
constructors. A bound type or family parameter used inside an expression body
therefore uses a lowercase name, as in `class Eq a` rather than `Eq A`. These
five combinators follow that convention.

`subst` transports a `Type`-valued family along a propositional equality
(`53 §2`). The `J` motive names the family at the transported endpoint.

```ken
fn subst (ty : Type) (x : ty) (y : ty) (fam : ty → Type)
           (p : Eq ty x y) (px : fam x) : fam y =
  J (λy' _. fam y') px p
```

`cong` lifts an equality of endpoints to an equality of images under any
function. The motive lands in `Omega` (proof-irrelevant), relying on `J`'s
unconstrained codomain sort (`34 §3.4`).

```ken
lemma cong (ty : Type) (ty2 : Type) (x : ty) (y : ty) (f : ty → ty2)
          (p : Eq ty x y) : Eq ty2 (f x) (f y) =
  J (λy' _. Eq ty2 (f x) (f y')) Refl p
```

`cast` is raw type-transport: given a proof two TYPES are equal, coerce a
value from one to the other. `e : Eq Type ty ty2` is a large elimination
(the carrier of the equality is `Type` itself) — `J`'s motive here computes
a value of the ambient universe, not a `Type`-valued family.

```ken
fn cast (ty : Type) (ty2 : Type) (e : Eq Type ty ty2) (t : ty) : ty2 =
  J (λx _. x) t e
```

`sym` flips the direction of a propositional equality.

```ken
lemma sym (ty : Type) (x : ty) (y : ty) (p : Eq ty x y) : Eq ty y x =
  J (λy' _. Eq ty y' x) Refl p
```

`trans` composes two propositional equalities.

```ken
lemma trans (ty : Type) (x : ty) (y : ty) (z : ty)
           (p : Eq ty x y) (q : Eq ty y z) : Eq ty x z =
  J (λz' _. Eq ty x z') p q
```

The package also relies on two already-provided language constructs it
does not itself declare:

- **`J motive base eq`** is a surface term former, not a package export. It
  is the language's equality eliminator.
- **`Eq A a b`** is the kernel's native equality type, spelled directly by
  the surface. The standard `Equal` alias is level-fixed at `Type0`, so it
  cannot express `cast`'s equality between two `Type` values.

## 3. Using it

`sym` is useful when a hypothesis arrives in the wrong orientation, for
example before transporting a stuck `match`:

```ken example
fn stuck_of (k : Bool) : Bool = match k { True => True ; False => False }

lemma stuck_transport (k : Bool) (q : Equal Bool k True)
  : Equal Bool (stuck_of k) True =
  J (λb' _. Equal Bool (stuck_of b') True) Proved (sym Bool k True q)
```

The base case above is `Proved` (Top-introduction), not `Refl`: once `k`
substitutes to `True` the operand reduces and `Equal Bool (stuck_of True)
True` observationally collapses to `Top`, so `Proved` inhabits the result.

## 4. Laws  proofs

There is no further internal law to state here: each combinator's own body
*is* the proof that the property it names holds (`subst` — a family
transports along an equality; `sym` — equality is symmetric; `trans` —
equality composes). `sym`/`trans` composing correctly is exercised directly
by chaining two real hypotheses through both, over an abstract carrier
(genuinely stuck, so the result is a real `J`-built proof term, not a
reduction shortcut):

```ken example
lemma sym_trans_compose (ty : Type) (a : ty) (b : ty) (c : ty)
  (p : Eq ty a b) (q : Eq ty c b) : Eq ty a c =
  trans ty a b c p (sym ty c b q)
```

## 5. Design notes

**Why five names instead of one general recipe.** Each name fixes a
specific, common motive shape (`subst`'s family, `cong`'s image-under-`f`,
`cast`'s type-level carrier, `sym`/`trans`'s direct endpoint algebra) so a
caller never has to re-derive the motive by hand — the motive is exactly
where a hand-written `J` proof is easiest to get wrong.

**Why `cast` needs `Eq`, not `Equal`.** The prelude `Equal` alias is
monomorphic at `Type0`; `cast`'s equality argument relates two `Type`
values, one level up, so it must spell the kernel's native `Eq` directly.

## 6. References

- **Lean 4 core** — `Eq.mpr`/`Eq.subst`/`congrArg`
  (`Init/Prelude.lean`, part of the Lean 4 repository, Apache-2.0) —
  <https://github.com/leanprover/lean4> — the same family of `J`-derived
  transport combinators this package names (consulted for shape only,
  `CLEAN-ROOM.md`; no source copied).

## 7. Trust  derivation

1. **Public API.** `subst`, `cong`, `cast`, `sym`, `trans`.
2. **Source map.**

   | Task | Section |
   |---|---|
   | See the shape | [Definition](#2-definition) |
   | Use it | [Using it](#3-using-it) |
   | Why five names, why `cast` needs `Eq` | [Design notes](#5-design-notes) |

3. **Derivation path.** Every combinator is an ordinary use of `J`, `Eq`,
   and equality reduction. The package adds no new eliminator or reduction
   rule.
4. **`trusted_base()` delta.** **Zero.** Every public name is checked as
   ordinary Ken source and reduces through the language's equality machinery.
5. **Proof families.** None recursive — every combinator is a single
   non-recursive `J` application.
6. **Consumers.** Equality proofs throughout the catalog use these
   combinators for congruence, orientation changes, composition, and
   transport across stuck expressions.
7. **Validation evidence.** The catalog checks a direct equality-elimination
   term, a zero `trusted_base()` delta, and transport over an abstract,
   stuck match expression.
