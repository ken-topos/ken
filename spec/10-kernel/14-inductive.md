# Inductive families

> Status: **DRAFT v0**. Normative. Declaration of inductive types, the
> strict-positivity requirement, the dependent eliminator and its ι-computation,
> and how primitive types attach. Identity is **not** a plain inductive in Ken —
> it is observational `Eq` (`15`, `16`); this chapter is the machinery `Eq`'s
> `J` and everything else reuse.

## 1. Declarations

An **inductive family** is declared in the global environment (`11 §4`):

```
data D (Δ_p) : (Δ_i) → Type ℓ where
  c₁ : (Δ₁) → D Δ_p t̄₁
  …
  cₙ : (Δₙ) → D Δ_p t̄ₙ
```

- `Δ_p` — **parameters**: fixed across the whole family and across all
  constructors (e.g. the `A` in `List A`).
- `Δ_i` — **indices**: may vary per constructor (e.g. the length in `Vec A n`).
  The family lands in `(Δ_i) → Type ℓ`.
- Each **constructor** `cₖ : (Δₖ) → D Δ_p t̄ₖ` takes arguments `Δₖ` (which may
  include recursive occurrences of `D`, subject to §2) and targets `D` at the
  *same parameters* `Δ_p` and *some* index instance `t̄ₖ`.
- `ℓ` is the family's universe level; constructor argument types must live at
  `ℓ` or below (predicativity, `12 §2`).

The kernel admits the declaration only if it passes (a) ordinary type-checking
of all constructor signatures in context `Δ_p`, (b) the **strict-positivity**
check (§2), and (c) universe-level checks. On admission the kernel adds: the
type former `D`, the constructors `cₖ`, and the **eliminator** `elim_D` (§3).

### Canonical examples (elaborated forms)

```
data Empty :            Type 0 where           -- ⊥, no constructors
data Unit  :            Type 0 where  tt : Unit -- ⊤
data Bool  :            Type 0 where  true : Bool ;  false : Bool
data Nat   :            Type 0 where  zero : Nat ;   suc : Nat → Nat
data List (A : Type ℓ): Type ℓ where  nil : List A ; cons : A → List A → List A
data Vec  (A : Type ℓ): Nat → Type ℓ where
  vnil  :                          Vec A zero
  vcons : (n : Nat) → A → Vec A n → Vec A (suc n)
```

`Vec` shows a genuine **index** (`Nat`) varying per constructor — an *indexed*
family, the feature that makes length-indexed vectors, well-typed syntax trees,
and the like expressible.

## 2. Strict positivity

To keep the logic consistent (no encoding of a fixpoint that inhabits `Empty`),
every recursive occurrence of `D` in a constructor argument MUST be **strictly
positive**: `D` may appear only as the *target* of a (possibly dependent)
function type, never to the *left* of an arrow, and never under another type
former applied to `D` in a non-positive position.

- **Allowed:** `A → List A → List A` (recursive arg `List A` is itself the
  type); `(n : Nat) → Vec A n → …` (recursive arg under a Π whose codomain is
  the recursive type); `(Nat → D) → D` (recursive occurrence strictly positive
  in the hypothesis — a branching/`W`-style argument).
- **Rejected:** `(D → Bool) → D` — `D` occurs to the left of an arrow
  (negative); admitting it would let one build a non-terminating, inconsistent
  fixpoint.

The kernel MUST run the strict-positivity check on every declaration and reject
negative occurrences. This check, plus the structural eliminator (§3), is what
guarantees the inductive fragment is total *without* needing the SCT machinery —
SCT (`17 §SCT`) is for *general* recursive definitions made via δ, not for
eliminator-based ones.

## 3. The dependent eliminator

For an inductive `D` the kernel generates one **dependent eliminator**
(induction principle) `elim_D`. It is the *only* primitive way to consume a
value of `D`; `match` and structural recursion at the surface
(`../30-surface/34-data-match.md`) elaborate to it
(`../30-surface/39-elaboration.md`).

**Shape.** Given a **motive**

```
M : (Δ_i) → D Δ_p Δ_i → Type ℓ'
```

(the result type, allowed to depend on the indices and the scrutinee — this
dependency is what makes it an *induction* principle, not just a recursor), and
one **method** `mₖ` per constructor giving the result for that constructor —
including, for each recursive argument, the **induction hypothesis** (the motive
already applied to that sub-value) — the eliminator has type:

```
elim_D : (M : (Δ_i) → D Δ_p Δ_i → Type ℓ')
       → (m₁ : ⟦method type for c₁⟧)
       → …
       → (mₙ : ⟦method type for cₙ⟧)
       → (i̅ : Δ_i) → (x : D Δ_p i̅) → M i̅ x
```

The method type for a constructor `cₖ : (Δₖ) → D Δ_p t̄ₖ` is: abstract over `Δₖ`,
add an induction hypothesis `M (…) r` for each recursive argument `r` in `Δₖ`,
and conclude `M t̄ₖ (cₖ …)`.

**Computation (ι).** On a constructor the eliminator reduces to the
corresponding method, applied to the constructor's arguments and the recursive
results:

```
elim_D M m̄ i̅ (cₖ ā)  ≡  mₖ ā [elim_D M m̄ … r  for each recursive r in ā]   (D-ι)
```

i.e. each recursive argument `r` is replaced in the method by `elim_D M m̄ … r` —
the structural recursive call. Because recursion is only ever on *structurally
smaller* sub-values, ι-reduction terminates; this is the totality of the
eliminator.

**Example (Nat).**

```
elim_Nat : (M : Nat → Type ℓ')
         → M zero
         → ((n : Nat) → M n → M (suc n))
         → (n : Nat) → M n
elim_Nat M z s zero      ≡  z
elim_Nat M z s (suc n)   ≡  s n (elim_Nat M z s n)
```

**Large elimination.** The motive may land in any `Type ℓ'`, including a
universe — so one may compute *types* by recursion on data (e.g. `if b then A
else B`, `elim_Bool`-style). Predicativity (`12`) keeps this sound; there is no
special restriction beyond the universe-level checks.

## 4. Σ as a record; relationship to Π

`Σ` (`13`) is presented natively (negatively, with η) rather than as an
inductive, because surjective-pairing η is wanted definitionally and a positive
inductive `Σ`'s η is only propositional. A *positive* inductive presentation
`data Σ' (A)(B) where pair : (a:A) → B a → Σ' A B` is **derivable** and
inter-derivable up to a path, but the kernel's primitive Σ is the negative one
(`13 §2`). Single-constructor inductives in general MAY be given η by the kernel
(definitional η for records); whether to extend definitional η to all
single-constructor inductives is **OQ-η-records** (`90-open-decisions.md`); the
DRAFT gives η to Σ (and hence records) and not to other inductives.

## 5. Primitive types

Machine types — `Int`, `Decimal`, `Float`, `Bytes`, …
(`../30-surface/35-numbers.md`, `../40-runtime/41-values.md`) — are **not**
inductive declarations (you cannot enumerate every `Int` with constructors).
They attach as **primitives** (`11 §4`):

```
Σ, (Int : Type 0 := prim …)               -- an opaque primitive type
Σ, (add : Int → Int → Int := prim …)      -- a primitive op + reduction rule
```

- A primitive type is an **opaque type constant**; it has no kernel-level
  constructors or eliminator. Its inhabitants are **literals** (introduced by
  the elaborator as opaque primitive values) and the results of primitive
  operations.
- A primitive operation carries a **registered reduction** `prim` (`41`):
  applied to literal arguments it computes a literal result *in the kernel's
  evaluator*, so `add 2 3 ≡ 5` holds definitionally and proofs can compute over
  literals. On non-literal (stuck) arguments it is a neutral term.
- Primitives are **trusted**: a wrong primitive reduction is a soundness bug, so
  the set of primitives is small, audited, and part of the kernel's trusted base
  (listed in `18 §Primitives`). This is the one place computation enters the
  kernel from outside the term language; everything else is β/ι/δ/obs.
- Equational properties of primitives that are *not* definitional (e.g.
  commutativity of `add`) are **propositions to prove**, provided as a small
  axiomatized interface or proved against a reference model, not assumed.

## 6. What the kernel checks here

A conforming kernel MUST:

1. Type-check inductive declarations and **enforce strict positivity** (§2),
   rejecting negative occurrences.
2. Generate the constructors and the **dependent** eliminator with induction
   hypotheses for recursive arguments (§3).
3. Implement **ι-reduction** of the eliminator on constructor forms (§3),
   driving structural recursion; ensure it terminates (structural decrease).
4. Permit **large elimination** under the predicative universe rules (§3).
5. Treat **primitive** types/operations as opaque constants with registered,
   audited reductions (§5), never as inductives.

Conformance: `../../conformance/kernel/inductive/` — positivity acceptance and
rejection, `elim_Nat`/`elim_Vec` ι-computation, large elimination (`elim_Bool`
into `Type`), and primitive-literal reduction (`add 2 3 ⇓ 5`).
