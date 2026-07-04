# Lawful constructor classes — `Semigroup`, `Monoid`, `Functor`, `Foldable`

> Status: **DRAFT v0 (CAT-1).** The first WP of the **catalog campaign**
> (`../../docs/program/06-catalog-campaign.md`) and the **pattern-setter** for
> the constructor classes every later catalog layer (collections, parsers,
> effects, traversals) leans on. Extends the `lawful-classes` discipline
> (`51`) — *ordinary Ken, laws carried as propositions and proved not
> postulated, zero `trusted_base()` delta on an inductive carrier* — from
> **value classes over `a : Type`** (`Eq`/`Ord`) to **value-level algebra**
> (`Semigroup`/`Monoid`) and the first **classes over a type constructor
> `f : Type → Type`** (`Functor`/`Foldable`). **No new kernel feature:** a
> class is a record (`../30-surface/33 §5.2`), a law is an `Ω` proposition
> (`../10-kernel/16 §1`); the one **outer-ring** addition — a higher-kinded
> class parameter — is a bounded `ken-elaborator` extension (`§6`), never a
> kernel change.

## 1. Why these lead the catalog

`Semigroup`/`Monoid` are the value-level algebra `Foldable`'s `foldMap`
consumes; `Functor`/`Foldable` are the first classes to **abstract a type
constructor**, and their law form is the one **CAT-2's `Applicative`/`Monad`
inherit verbatim**. Getting the pattern right here — the higher-kinded class
mechanism, the law-statement form, the proved-not-postulated instances — is
what makes every later tranche mechanical. This chapter is the **contract**;
the Team-Language build lands the `.ken` source (`packages/lawful-functors/`)
+ the one elaborator extension (`§6`).

`Applicative`/`Monad`/`Traversable` are **CAT-2** (fast-follow, depend on this).

## 2. `Semigroup` and `Monoid` — value-level algebra (over `a : Type`)

These are ordinary value classes, the **same shape as `Eq`/`Ord`** (`51 §2`) —
no higher-kinded machinery. The one difference from `Eq`/`Ord` is the **law
sort** (`§4`): a Semigroup/Monoid operation is `a`-valued, not `Bool`-valued,
so its laws are the kernel's **own propositional equality** `Equal a u v : Ω`
directly — *not* the `IsTrue b := Equal Bool b True` bridge `Eq`/`Ord` need for
their `Bool`-valued operations.

### 2.1 `Semigroup a` — an associative binary operation

```
class Semigroup a {
  op    : a → a → a
  assoc : (x y z : a) → Equal a (op (op x y) z) (op x (op y z))
}
```

`op` is the ergonomic `<>`/mappend. It is a **plain identifier field**, not an
infix operator token — an infix `<>` spelling is deferred sugar (`OQ-syntax`),
exactly as `36 §4.5`'s `get`/`put` name effect operations rather than minting
operators. `assoc` is a propositional equation in `Ω`.

### 2.2 `Monoid a` — a Semigroup with a two-sided identity

```
class Monoid a {
  op         : a → a → a
  mempty     : a
  assoc      : (x y z : a) → Equal a (op (op x y) z) (op x (op y z))
  left_unit  : (x : a) → Equal a (op mempty x) x
  right_unit : (x : a) → Equal a (op x mempty) x
}
```

**`Monoid` restates `op`/`assoc` rather than wiring a `Semigroup` superclass
field** — the `DecEq`-subsumes-`Eq` precedent (`51 §2.2`): the stronger class
carries the weaker one's operation + law, and the subsumption ("a `Monoid`
yields a `Semigroup` by forgetting `mempty`/`left_unit`/`right_unit`, keeping
`op`/`assoc`") is **recorded as a fact, not a `where`-constraint**. This keeps
the value classes exactly `Eq`/`Ord`-shaped, with no new kind machinery.

> **Superclass wiring is a *template* question deferred to `§5`/CAT-2.** For the
> value-level pair, restating is the grounded precedent. Whether the
> **constructor-class chain** (`Functor → Applicative → Monad`, CAT-2) should
> instead **wire** superclass fields — where the chain is deep and restatement
> costly — is pinned as a template decision (`§5`), not settled by these two.

## 3. Canonical instances — proved, zero-delta

Two carriers, chosen to exercise **both** proof styles a catalog author meets:

- **`List` append monoid** (`op = list_append`, `mempty = Nil`) — the inductive
  carrier; laws proved by **induction + congruence**.
- **`Bool` conjunction monoid** (`op = bool_and`, `mempty = True`) — the finite
  carrier; laws proved by **finite case-split**.

Every law field is a **real kernel proof** (Ω-motive `Elim`, K4/K5/K7 — the same
capabilities `51 §6`'s `Bool` instances rest on); **no `Axiom`, zero
`trusted_base()` delta**. (Grounded: the full design + the package file
elaborate through `elaborate_file`.)

### 3.1 The two-line proof grammar — induction + `cong`

An `Equal`-valued law over an inductive carrier is a **recursive `view`
returning `Equal …`**, `match`ing the carrier: the base constructor is closed
directly, the step lifts the recursive-self-call **IH** under the constructor
with `cong` (`packages/transport`, `53 §2`):

```
list_right_unit (a : Type) (xs : List a) : Equal (List a) (list_append a xs (Nil a)) xs =
  match xs {
    Nil      => tt                                         -- base
    Cons h t => cong (List a) (List a) (list_append a t (Nil a)) t
                     (Cons a h) (list_right_unit a t)       -- step: cong under Cons on the IH
  }
```

The **List append-monoid laws proved this way** are generic in the element
type: `list_assoc`, `list_left_unit` (definitional — `list_append Nil x`
ι-reduces to `x`, so `Refl`), `list_right_unit`. The `Bool` laws are the finite
analog — a full case-split, every branch closed directly.

### 3.2 The `tt`-vs-`Refl` discrimination (a load-bearing K7 subtlety)

A base/branch closes with **`tt`** or **`Refl`** depending on what its two
endpoints **reduce to** — this is not interchangeable:

- **Constructor-headed endpoints → `Top` → `tt`.** `list_right_unit`'s `Nil`
  base reduces both sides to `Nil a`; two occurrences of the **same
  constructor** observationally collapse to `Top` (`16 §8.1`, K7), so the goal
  is **no longer `Eq`-shaped** and `Refl` (which requires an `Eq`-shaped goal)
  does not apply — it is `Top`-introduced by `tt`. Likewise every `Bool` branch
  whose sides reduce to the same literal.
- **Neutral endpoints → stuck `Eq` → `Refl`.** `list_assoc`'s `Nil` base reduces
  both sides to the **neutral** `list_append a ys zs` (stuck on the free
  `ys`/`zs`); the goal stays `Eq`-shaped, closed by `Refl`.

This is the exact discrimination `lawful_classes.ken`'s `Bool` proofs document
(`51 §6`): ask what the endpoints reduce to — a constructor head (`Top`, `tt`)
or a neutral (`Eq`, `Refl`).

## 4. Law-field sorts — every law is `Ω`, no truncation (AC2)

Each class is a **structure class** (`33 §5.1`) — an `op` field is `Type`-valued
(`a → a → a` / `f a → f b`), so the record lands in `Type`, never forced to `Ω`.
The **law fields are `Ω`-valued**: a Semigroup/Monoid law is `Equal a u v` (an
equation between `a`-values), and `Equal _ _ _ : Ω` (`obs.rs`; `16`) — a
proof-irrelevant proposition. **The `51 §3` truncation catch does not fire**:
these are direct **value equations**, not a bare propositional `∨`/`∃` (whose
proof-relevant "which side / which witness" content would need `‖·‖`,
`16 §6`; [[proof-relevant-inductive-cannot-be-declared-at-omega]]). The Functor
laws (`§5`) are likewise value equations — Ω-clean, no truncation.

## 5. `Functor` and `Foldable` — classes over `f : Type → Type`

These are the first classes to abstract a **type constructor**. Two design
facts are pinned here (Architect's CAT-1 core ruling), because CAT-2 inherits
both.

### 5.1 The higher-kinded class parameter needs a bounded elaborator extension

A class quantifying over `f : Type → Type` does **not** elaborate on the landed
elaborator: the class parameter is hard-coded to `Type0` (`elab_class_decl`,
three unconditional `Term::ty(Level::Zero)` sites, ~`elab.rs:1862–1902`) and the
parser (`parse_class_decl`, `parser.rs:463`) takes a **bare ident** only, with
no `(f : K)` kind-binder path — so `class Functor f` binds `f : Type0` and a
field `map : … → f a → f b` applies a non-Π `f`, kernel-rejected. The universe
is **not** the blocker: `sort_sigma` (`check.rs`) is level-generic, so a
`Type1` structure record (Functor's `map` quantifies `(a b : Type0)`) is
admitted fine, and `List`/`Option` are real `Type0 → Type0` indformers
(`prelude.rs`) to substitute for `f`.

The fix is an **outer-ring (`ken-elaborator`-only, kernel-untouched)** extension
— **bounded to exactly four pieces** (`§6`). `class Functor (f : Type → Type)`
then elaborates as a `Type1` structure class; `instance Functor List` resolves
its head to the **bare `List` indformer** (a closed head, no free variable).

### 5.2 The Functor laws are **pointwise** — one field, no truncation

**funext is definitional in Ken's OTT** (`obs.rs`: `Eq ((x:A)→B) f g ⇝ (x:A) →
Eq (B x) (f x) (g x)`), so the function-level form `Equal (f a → f a) (map idf)
idf` **whnf-reduces to** the pointwise form — they are the *same proposition* up
to one reduction step. **Pointwise is the normal form**, so the prover's goal
*is* the stated law and every instance discharges by **direct induction on the
carrier**, no funext layer to strip:

```
-- identity law
(a : Type) → (x : f a) → Equal (f a) (map a a (idf a) x) x
-- composition (fusion), applied-pointwise
(a b c : Type) → (g : b → c) → (h : a → b) → (x : f a) →
  Equal (f c) (map a c (comp a b c g h) x) (map b c g (map a b h x))
```

Both are Ω-clean value equations (`§4`), so the truncation catch does not fire.
The point-free/categorical equation is available **for free** as a
definitionally-equal restatement — so **do not proliferate a second law field**:
**one canonical pointwise field per law.** This is the form CAT-2's Monad laws
inherit; stating them pointwise keeps that inheritance a direct induction too.
`idf`/`comp` are ordinary Ken views.

`Foldable` supplies `foldr` (and/or `foldMap` via a `Monoid`, `§2`) with the
fold laws + `Monoid` coherence; its instances (`List`/`Option`) are inductive ⇒
proved, zero-delta. *(The `foldr`-vs-`foldMap`-primary choice + the exact fold
laws are pinned with the build once the extension of `§6` lands.)*

## 6. The higher-kinded mechanism extension — bounded (AC1, hard)

The extension of `§5.1` is a **pinned CAT-1 sub-deliverable**, ratified to stay
inside this WP (not a separate frame→elaborate cycle). It is **outer-ring:
`ken-elaborator`-only, zero `ken-kernel` diff, no new `Term`/`Decl`** — the
kernel already admits the downstream shape (level-generic Σ, Π over
`Type0 → Type0`, predicative `declare_def`). It is **bounded to exactly these
four pieces** (Architect's sizing):

1. **AST** (`ClassDecl.param`) — carry an optional param kind; absent ⇒ `Type0`
   (back-compat).
2. **Parser** (`parse_class_decl`) — admit the `class C (f : K) { … }` binder
   form alongside the bare ident (`parse_type` already exists for `K`).
3. **Elaboration** (`elab_class_decl`) — replace the three hard-coded
   `Term::ty(Level::Zero)` (~`elab.rs:1862–1902`) with the elaborated param-kind
   term (default `Type0`). The ~10-line core change.
4. **Instance-side** (build-verify, likely 0 LOC) — `instance Functor List`
   resolves its head to the **bare `List` indformer** (`Type0 → Type0`);
   `head_type_name` already keys on `List`.

> **Hard AC (scope guardrail, Steward's pin — verbatim).** *The outer-ring
> extension is bounded to exactly these four pieces (AST param-kind field;
> parser `class C (f : K) { … }` binder; the ~10-line 3-site elab fix replacing
> `Term::ty(Level::Zero)` at ~L1862–1902; the instance-side head-resolution
> build-verify point) — any kernel touch, new `Term`/`Decl`, non-trivial
> instance-resolution change, or second elaborator axis re-forks to Steward
> before proceeding.* A sized outer-ring extension must not grow silently into a
> compiler change hiding inside a package WP.

### 6.1 A second, distinct surface gap — the parametric instance head (open)

Separately from `§6`'s higher-kinded **class parameter**, a **parametric
instance head** `instance Monoid (List a)` (a value class over a *parametric*
carrier) does **not** elaborate today: the parser accepts the head, but
elaboration has no binder path for the free `a` and does not generalize the
instance over it (`UnresolvedCon "a"`, grounded by probe). This is
**elaboration-side, distinct from `§6`** (which is the class param's kind;
`instance Functor List` uses a closed bare head and is unaffected). **It is
non-blocking:** the value-monoid instances bundle at **closed carriers**
(`Bool`, `List Nat`) today, and their **proofs are already generic** in the
element type (`§3.1`) — so CAT-1's ACs are met. Delivering the fully-general
`instance Monoid (List a)` is a **generality upgrade** (the `map-verified-laws`
precedent is the same shape — a parametric structure's `instance`/`where`
bundling is `(oracle)`-deferred while the parametric substance is real). Fork
open with Steward: fold into `§6`'s extension or defer as its own follow-on.

## 7. The reusable constructor-class template (AC5)

This chapter is the template CAT-2 (`Applicative`/`Monad`/`Traversable`) and
CAT-3 (collection laws) extend mechanically:

1. **A class is a record** (`33 §5.2`); **a law is `Ω`** (`16 §1`); a value law
   is a direct `Equal a u v`, a `Bool`-op law rides `IsTrue` (`51 §2`) — either
   way Ω-clean, no truncation unless a law is a genuine `∨`/`∃` (`§4`).
2. **Higher-kinded classes** (`f : Type → Type`) use the `§6` param-kind binder;
   instances resolve to the **bare indformer** head.
3. **Laws are stated pointwise** (funext-definitional ⇒ pointwise is the normal
   form, `§5.2`); **one canonical law field**, no point-free duplicate.
4. **Instances land over inductive carriers** so laws are **proved** by
   induction + `cong` (`§3.1`) — **zero `Axiom`, zero delta**; a postulate on an
   inductive carrier is a **defect** (`51 §5`), never an honest delta.
5. **The stronger class restates** the weaker's operation + law and **records**
   the subsumption as a fact (`§2.2`) — *unless* the chain is deep enough
   (CAT-2's `Functor → Applicative → Monad`) that wiring a superclass field
   wins; that is the one open template decision, deferred to CAT-2.

## 8. Derivation paths and `trusted_base()` delta (AC1)

- **The classes** are `class` declarations = record types (`33 §5.2`,
  right-nested Σ over `13 §3`), built from the kernel's `Equal`/`Ω` vocabulary
  (`15`/`16`, prelude) + the Σ/record machinery. **No new kernel former, zero
  delta.** The `§6` higher-kinded extension is **outer-ring** — still zero
  kernel diff, zero delta.
- **The instances are ZERO-DELTA — the inductive-carrier exemplar** (the path
  `51 §6`'s `Int` instances could *not* take). `List`/`Bool`/`Option` are real
  inductives with eliminators, so every ∀-law is a real kernel proof; **no
  `Axiom`, nothing enters `trusted_base()`**.
- **Reused, never re-defined** (subsume-don't-proliferate): `cong`/`sym`/`trans`
  (`packages/transport`, over `J`, zero delta); `list_append`
  (`packages/collections`, the List monoid op); `bool_and` a **transparent**
  view (not the `and_bool` primitive — a primitive never reduces on a symbolic
  argument, `51 §6`'s `bool_or` reasoning).

## 9. Acceptance

- **AC1 (kernel-untouched + zero-delta).** No `ken-kernel` diff; the `§6`
  extension is outer-ring, bounded to the four pieces (hard AC, `§6`); the
  inductive-carrier instances add **zero `trusted_base()` delta** (grep the law
  fields for `declare_postulate`/holes — absence is the guarantee).
- **AC2 (Ω-clean classes).** Each class elaborates as a record; every law field
  is an `Ω` proposition (`§4`), no accidental proof-relevance, no truncation.
- **AC3 (laws PROVED — hard, soundness).** Every canonical instance's law fields
  are **proved** (induction / case-split), **not `Axiom`**; a law-less
  "instance" is rejected (non-empty delta / re-check failure), verdict flips
  ([[kernel-backed-claim-grep-the-emission-not-the-name]]).
- **AC4 (law form).** Functor laws are **pointwise, one field** (`§5.2`) —
  funext-definitional, the form CAT-2 inherits.
- **AC5 (template).** Constructor-class template (`§7`) documented for CAT-2.
- **AC6 (examples).** An accepted use (a real `<>`/`map`/`fold` program) and a
  rejected use (a broken-unit "Monoid" / identity-breaking "Functor").
