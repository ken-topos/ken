# Identity, equality, and `J`

> Status: **K2 elaborated**. Normative. Ken's propositional equality is
> **observational equality `Eq`** (ADR 0005) -- not an inductive `Id`, not a
> cubical `Path`. This chapter is the identity *interface* (`Eq`, `refl`,
> `cast`, `J` and its computation, the derived equalities); the *machinery*
> it computes by -- `Eq`-by-type, `cast`, Omega, quotients -- is in
> `16-observational.md`.

## 1. Why observational equality

Martin-Lof type theory makes equality an inductive `Id A a b` whose
eliminator `J` **only computes on `refl`**. Cubical type theory fixes that
(via the interval) but at the cost of the largest, most
canonicity-fragile part of a kernel, and it provides univalence + higher
structure that *software* does not use.

Ken takes equality to be **observational** (`TTobs`/`CICobs`, ADR 0005):
`Eq A a b` is a **proposition computed by recursion on the type `A`**,
and a `cast` coercion transports along type-equalities and **reduces on
reflexivity**. From `cast`, `J` is derived and **reduces on any equality,
not only `refl`**, while the kernel stays small and **set-level** (UIP
holds), which is exactly what software data is. K1's required "`Id`, `J`"
is satisfied by `Eq` + the derived `J` (par. 4); `Id A a b` and `a = b`
MAY be surface synonyms for `Eq A a b`.

## 2. The equality type

**Formation.**
```
  Gamma |- A : Type l      Gamma |- a : A      Gamma |- b : A
  ───────────────────────────────────────────────────────────  (Eq-Form)
  Gamma |- Eq A a b : Omega_l
```
`Eq A a b` lands in `Omega_l` (the strict proposition universe at the
level of `A`; `16` par. 1.1), so it is **proof-irrelevant**: any two
proofs are definitionally equal (no coherence baggage; `16` par. 1.2).

**Reflexivity.** `refl a : Eq A a a`.

**Computation.** `Eq A a b` reduces *by recursion on `A`*
(`16` par. 2.2): pointwise at a Pi-type (so **funext is definitional**),
componentwise at a Sigma-type, structurally at an inductive, to the user
relation at a quotient, to mutual implication at Omega (**propext**), and
to literal equality at a primitive. On a neutral `A` it is a neutral
proposition.

`Eq` has **no primitive reduction rules of its own** -- it delegates
entirely to the type-directed computation defined in `16` par. 2.2. The
kernel's `whnf` invokes those rules when the head of the type is a known
type former; otherwise `Eq` is neutral. The algorithm for `whnf(Eq A a b)`
is: compute `whnf(A)`, then dispatch on the head former per the reduction
table in `16` par. 2.2.

## 3. Transport -- `cast` / `subst`

The operation that makes equality *do* something is **`cast`**
(`16` par. 3): transporting along a type-equality, with
`cast A A refl a ≡ a` (regularity) and `cast`-by-type computation.

```
  Gamma |- A : Type l   Gamma |- B : Type l
  Gamma |- e : Eq Type A B   Gamma |- a : A
  ───────────────────────────────────────────  (cast)
  Gamma |- cast A B e a : B
```

`cast`'s reduction rules are in `16` par. 3.2. The key properties:
- **Regularity:** `cast A A refl a ≡ a` definitionally.
- **By-type computation:** `cast` reduces by recursion on the structure of
  `A` and `B`, never inspecting the proof `e` -- it computes from the
  endpoints.
- **Neutral on stuck proofs:** when `A`, `B`, or `e` is neutral, `cast`
  is neutral.

For a family `P : A -> Type l` and `e : Eq A a b`, substitution is

```
  subst P e  :≡  cast (P a) (P b) (cong P e)  :  P a -> P b
```

`subst P (refl a) ≡ id`, and on a non-trivial equality `subst`/`cast`
reduce by recursion on `P`/the type (`16` par. 3.2) -- this is why
everything below computes.

## 4. `J` (path induction) and its computation rule

`J` is the eliminator: to prove a property `P` of "any equality out of
`a`", prove it for `refl`.

**Type.**
```
  J : (A : Type l) (a : A)
      (P : (b : A) -> Eq A a b -> Type l')
      (d : P a (refl a))
      (b : A) (e : Eq A a b)
    -> P b e
```

### 4.1 Definition (derived from `cast`)

`J` transports the base case `d` along `e` using `cast`. The construction
works as follows:

1. Form the **singleton type** `S := (b' : A) x Eq A a b'`. This is a
   Sigma-type whose elements are pairs `(b', proof that a = b')`.

2. By Sigma-Eq-by-type (`16` par. 2.2), compute the equality of the two
   singleton elements `(a, refl a)` and `(b, e)`:
   ```
   Eq S (a, refl a) (b, e)
     ⇝  Eq A a b  and  Eq (Eq A a b) (cast ... (refl a)) e
   ```
   The first conjunct is exactly `e` itself. The second conjunct holds
   definitionally by Omega proof-irrelevance (`16` par. 1.2) since
   `Eq A a b : Omega`. So the pair equality is effectively witnessed by
   `e` alone (the second component is a trivial Omega proof).

3. Apply `cong` with family `lam s. P (s.1) (s.2)` to this pair equality,
   yielding:
   ```
   pair-eq : Eq Type (P a (refl a)) (P b e)
   ```

4. Then `J` is defined as:
   ```
   J A a P d b e :≡ cast (P a (refl a)) (P b e) pair-eq d
   ```

The kernel provides `J` as a **derived operation** with the rules below;
it need not be primitive. The precise construction of `pair-eq` is a fixed
term schema (structural, not metaprogramming) that the kernel can
synthesize from `A`, `a`, `b`, `e`, and `P`.

### 4.2 Computation on `refl` (beta)

```
  J A a P d a (refl a)  ≡  d  :  P a (refl a)            (J-beta)
```

This follows from `cast`-refl (`16` par. 3.2): when `e = refl a`,
`pair-eq` reduces to `refl (P a (refl a))`, and
`cast (P a (refl a)) (P a (refl a)) (refl ...) d ≡ d`.

### 4.3 Computation on non-`refl` equality (the correction)

Because `J` is built from `cast`, when its equality argument is a
*non-`refl`* but otherwise canonical proof -- e.g. one produced by
`subst`, by a constructor's congruence, or by a quotient relation -- `J`
**reduces** by the corresponding `cast` computation (`16` par. 3.2) rather
than getting stuck.

Concretely:

- When `P` is a constant family (does not depend on the equality proof),
  `pair-eq` reduces via Eq-by-type (Omega-PI handles the second
  component), and `cast` recurses on the structure of `P` -- which is
  well-founded since `P` is a finite type expression.

- When `P` does depend on the equality proof, the `cast` still computes
  because `pair-eq` is a canonical type-equality (built from `e` via
  Eq-by-type), and `cast` reduces by type structure on `P a (refl a)` and
  `P b e`.

- In all cases, `J` on a non-`refl` canonical equality reduces to a
  constructor form (lambda, pair, constructor application) -- it never
  gets stuck at a neutral `J` node.

A conforming kernel MUST exhibit this: a conformance test
(`../../conformance/kernel/observational/j-nonrefl`) in which `J` on a
non-`refl` equality computes to a constructor form, **failing on any
kernel that only reduces `J` on `refl`.** This is achieved via
observational equality rather than cubical paths.

## 5. Derived equalities (theorems, mostly definitional)

The observational presentation makes the equalities classical MLTT must
axiomatise into **definitional or computing** facts (`16` par. 4):

- **Function extensionality (funext)** -- *definitional*: `Eq` at a
  Pi-type **is** pointwise `Eq` (`16` par. 2.2). Two functions are equal
  iff equal at every argument, with no axiom -- a major win for the
  verification layer, which constantly needs extensional function
  equality.
- **Propositional extensionality (propext)** -- *definitional*: equal
  propositions are mutually-implying (`16` par. 2.2).
- **UIP / proof irrelevance** -- *definitional*: `Eq : Omega`
  (`16` par. 1.2). Ken is set-level: there is no nontrivial
  `Eq (Eq A a b) p q`.
- **`sym`, `trans`, `cong`, `subst`** -- derivable and computing
  (`16` par. 4). Each is defined from `cast`, so each inherits `cast`'s
  computation behaviour.

## 6. No univalence (set-level, by design)

Ken has **no univalence** (`(A ≃ B) -> Eq Type A B`) and no
higher-dimensional structure -- these are cubical/HoTT features for
*mathematics*, deliberately absent (ADR 0005, `16` par. 7).
Type-equality `Eq Type A B` is **structural** (`16` par. 3): same head
former with equal parts. So you cannot transport a program across an
arbitrary *equivalence* of types; you transport across *structural*
type-equalities (which covers reindexing, parameter equalities, and the
like -- what software needs). The generic-programming-via-univalence idiom
is the one thing given up, and for set-level software it is a non-loss.

## 7. What the kernel checks here

A conforming kernel MUST:

1. Form `Eq A a b : Omega` (par. 2).
2. Compute `Eq`-by-type (`16` par. 2.2) including **definitional funext
   and propext**.
3. Provide `cast`/`subst` with **`cast`-refl** and by-type computation
   (`16` par. 3.2).
4. Provide `J` with **both** `J-beta` (on `refl`) **and** reduction on
   non-`refl` equalities via `cast` (par. 4). The definition of `J` is a
   derived operation from `cast` (par. 4.1).
5. Derive `sym`/`trans`/`cong` and **definitional UIP** (`Eq : Omega`).

The non-`refl`-`J` computation and definitional funext are required,
separately-tested behaviours.

Conformance: `../../conformance/kernel/observational/`. See also
`16` par. 9 for the full soundness-critical behaviour table and
`seed-kernel.md` for K2-tagged seed cases.
