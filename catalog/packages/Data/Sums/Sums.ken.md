# `Sums` — the `Option`/`Result`/`Either` combinator floor

The `Option`/`Result`/`Either` combinator floor: one entry for all three
L2-sum families, each combinator paired with its defining equation(s) as a
real proof term.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws & proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [Findings](#6-findings)
7. [References](#7-references)
8. [Trust & derivation](#8-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws & proofs](#4-laws--proofs)
- *Researcher* → [Laws & proofs](#4-laws--proofs) →
  [Design notes](#5-design-notes)

## 1. Motivation

DS-3 (`wp/ds-3-sum-type-combinators.md`; `Either`,
`wp/either-catalog-package.md`) needs a combinator floor for the three
binary-sum families the surface already has: `Option`, `Result`, and
`Either`. `Option`/`Result` are prelude-declared (`Option a = None | Some
a`, `Result e a = Err e | Ok a` — `Err` is first,
`crates/ken-elaborator/src/prelude.rs`) — this entry does not re-declare
either type, and reuses `option_map` + `instance Functor Option`
(`catalog/packages/Core/LawfulFunctors.ken.md`) rather than re-deriving
them. `Either a b = Left a | Right b` is declared here, at the catalog
level — unlike `Option`/`Result`, it needs zero built-in/prelude support
(an ordinary non-dependent sum), matching the spec's own model of core
data as packages (`50-stdlib/README.md:42`; judgment call L5). One entry
holds all three L2-sum families, not three separate ones, per
subsume-don't-proliferate on package count — mirroring
`catalog/packages/Data/Collections/Collections.ken.md` holding the whole
`List` floor in one entry.

`Either` is not a re-derivation of `Result`: an earlier DS-3-era spec
erratum subsumed `Either` into `Result` (isomorphic-twin proliferation —
`Either` had no declaration or user at the time). The operator's later L5
ruling reversed that, landing `Either` as a distinct declared package per
the spec's own core-data-as-packages model. `Result` remains the
error-biased sum wired into the effect system; `Either` is the neutral
one, and both coexist.

## 2. Definition

`Option`/`Result` are reused from the prelude unchanged, not re-declared
here. `Either a b = Left a | Right b` — declared in
[§4.3](#43-either), in its place among the combinators that use it — is
the one genuinely new carrier this package introduces.

## 3. Using it

Every combinator in this package is paired immediately with its defining
equation(s), stated as a real proof term — never a comment claiming the
behavior. `get_or_else`/`is_some`/`or_else` cover `Option`; `map_err`/
`and_then`/`unwrap_or` cover `Result`; `either` (the eliminator), `map_left`/
`map_right`, and `swap` cover `Either`. A caller reasoning about, say,
`and_then`'s short-circuit behavior on `Err` reaches for `and_then_err`
directly rather than re-deriving it from `and_then`'s definition. Every
proof reduces by `Refl` — each combinator is an ordinary structural
case-split, so its defining equations hold by computation alone, with no
transport machinery needed anywhere in this package.

## 4. Laws & proofs

Every combinator here is an ordinary structural case-split `fn`/`const`
over the declared sums, in the same style as the `List` floor
(`catalog/packages/Data/Collections/Collections.ken.md`). Zero new kernel
feature, zero `trusted_base()` delta.

### 4.1 `Option`

`get_or_else d x` returns `x`'s contained value, or the default `d` at
`None`; `is_some x` is the `Bool` presence test; `or_else x y` is
left-biased — `x`'s own value wins whenever `x` is `Some`, and `y` is only
consulted when `x` is `None`. The right-hand-`None` identity (`or_else x
None = x`) falls out cleanly from the same definition, so it ships
alongside the two defining equations.

```ken
fn get_or_else (a : Type) (d : a) (x : Option a) : a =
  match x { None ⇒ d ; Some v ⇒ v }

fn get_or_else_none (a : Type) (d : a) : Equal a (get_or_else a d (None a)) d = Refl

fn get_or_else_some (a : Type) (d : a) (v : a) : Equal a (get_or_else a d (Some a v)) v = Refl

fn is_some (a : Type) (x : Option a) : Bool =
  match x { None ⇒ False ; Some v ⇒ True }

const is_some_none (a : Type) : Equal Bool (is_some a (None a)) False = tt

fn is_some_some (a : Type) (v : a) : Equal Bool (is_some a (Some a v)) True = tt

fn or_else (a : Type) (x : Option a) (y : Option a) : Option a =
  match x { None ⇒ y ; Some v ⇒ Some a v }

fn or_else_none (a : Type) (y : Option a) : Equal (Option a) (or_else a (None a) y) y = Refl

fn or_else_some (a : Type) (v : a) (y : Option a) : Equal (Option a) (or_else a (Some a v) y) (Some a v) = Refl

fn or_else_none_rhs (a : Type) (x : Option a) : Equal (Option a) (or_else a x (None a)) x =
  match x { None ⇒ tt ; Some v ⇒ Refl }
```

### 4.2 `Result`

`map_err g x` maps `g` over the error side only, leaving an `Ok` payload
untouched; `and_then k x` short-circuits on `Err` (`k` is never called)
and otherwise threads `x`'s payload through `k`; `unwrap_or d x` returns
the contained value, or the default `d` at `Err`.

```ken
fn map_err (e : Type) (f : Type) (a : Type) (g : e → f) (x : Result e a) : Result f a =
  match x { Err u ⇒ Err f a (g u) ; Ok v ⇒ Ok f a v }

fn map_err_ok (e : Type) (f : Type) (a : Type) (g : e → f) (v : a)
  : Equal (Result f a) (map_err e f a g (Ok e a v)) (Ok f a v) = Refl

fn map_err_err (e : Type) (f : Type) (a : Type) (g : e → f) (u : e)
  : Equal (Result f a) (map_err e f a g (Err e a u)) (Err f a (g u)) = Refl

fn and_then (e : Type) (a : Type) (b : Type) (k : a → Result e b) (x : Result e a) : Result e b =
  match x { Err u ⇒ Err e b u ; Ok v ⇒ k v }

fn and_then_ok (e : Type) (a : Type) (b : Type) (k : a → Result e b) (v : a)
  : Equal (Result e b) (and_then e a b k (Ok e a v)) (k v) = Refl

fn and_then_err (e : Type) (a : Type) (b : Type) (k : a → Result e b) (u : e)
  : Equal (Result e b) (and_then e a b k (Err e a u)) (Err e b u) = Refl

fn unwrap_or (e : Type) (a : Type) (d : a) (x : Result e a) : a =
  match x { Err u ⇒ d ; Ok v ⇒ v }

fn unwrap_or_ok (e : Type) (a : Type) (d : a) (v : a) : Equal a (unwrap_or e a d (Ok e a v)) v = Refl

fn unwrap_or_err (e : Type) (a : Type) (d : a) (u : e) : Equal a (unwrap_or e a d (Err e a u)) d = Refl
```

### 4.3 `Either`

`Either` is a user-level catalog package, not a prelude type (judgment
call L5): an ordinary non-dependent sum needs zero built-in support,
matching the spec's own model of core data as packages
(`50-stdlib/README.md:42`). `Option`/`Result` sit in the prelude today only
as a bootstrap shortcut — a named spec-vs-implementation gap this package
does not resolve (`docs/program/wp/either-catalog-package.md`, "Named
future").

`either f g x` is the eliminator: `f` on `Left`, `g` on `Right`.
`map_left`/`map_right` apply a function to one side only, leaving the
other side's payload untouched — `map_left_right`/`map_right_left` are the
"untouched" proofs, as important to state as the "applied" ones.
`swap` exchanges `Left`/`Right`, and is genuinely involutive
(`swap_involutive`), not merely the identity.

```ken
data Either a b = Left a | Right b

fn either (a : Type) (b : Type) (c : Type) (f : a → c) (g : b → c) (x : Either a b) : c =
  match x { Left v ⇒ f v ; Right v ⇒ g v }

fn either_left (a : Type) (b : Type) (c : Type) (f : a → c) (g : b → c) (v : a)
  : Equal c (either a b c f g (Left a b v)) (f v) = Refl

fn either_right (a : Type) (b : Type) (c : Type) (f : a → c) (g : b → c) (v : b)
  : Equal c (either a b c f g (Right a b v)) (g v) = Refl

fn map_left (a : Type) (b : Type) (c : Type) (f : a → c) (x : Either a b) : Either c b =
  match x { Left v ⇒ Left c b (f v) ; Right v ⇒ Right c b v }

fn map_left_left (a : Type) (b : Type) (c : Type) (f : a → c) (v : a)
  : Equal (Either c b) (map_left a b c f (Left a b v)) (Left c b (f v)) = Refl

fn map_left_right (a : Type) (b : Type) (c : Type) (f : a → c) (v : b)
  : Equal (Either c b) (map_left a b c f (Right a b v)) (Right c b v) = Refl

fn map_right (a : Type) (b : Type) (c : Type) (g : b → c) (x : Either a b) : Either a c =
  match x { Left v ⇒ Left a c v ; Right v ⇒ Right a c (g v) }

fn map_right_left (a : Type) (b : Type) (c : Type) (g : b → c) (v : a)
  : Equal (Either a c) (map_right a b c g (Left a b v)) (Left a c v) = Refl

fn map_right_right (a : Type) (b : Type) (c : Type) (g : b → c) (v : b)
  : Equal (Either a c) (map_right a b c g (Right a b v)) (Right a c (g v)) = Refl

fn swap (a : Type) (b : Type) (x : Either a b) : Either b a =
  match x { Left v ⇒ Right b a v ; Right v ⇒ Left b a v }

fn swap_involutive (a : Type) (b : Type) (x : Either a b) : Equal (Either a b) (swap b a (swap a b x)) x =
  match x { Left v ⇒ Refl ; Right v ⇒ Refl }
```

## 5. Design notes

**One entry, not three.** DS-3's frame considered a separate package per
sum family; this entry holds all three per subsume-don't-proliferate on
package count, mirroring `Collections.ken.md`'s single-entry `List` floor.
The three families are similar enough in shape (a binary sum, an
eliminator or direct case-split, defining equations by `Refl`) that
splitting them would triple boilerplate without adding reader clarity —
the three-part `Using it`/`Laws & proofs` subsection split already gives a
reader who only cares about one family a direct path to it.

**`Either`'s independence from `Result` is a reversed ruling, not an
original design.** An earlier spec erratum folded `Either` into `Result`
as an isomorphic twin, reasoning that a package with no declaration and no
consumer yet was dead weight. The operator's later L5 ruling reversed
this once a genuine neutral-sum need (not error-biased) was identified —
`Result` keeps its effect-system wiring, `Either` is the general-purpose
sum, and the two are declared independently rather than one being defined
in terms of the other.

**Every proof reduces by `Refl`.** Unlike the `Map`/`Collections` capstone
proofs (which need `cong`/`trans`/`J` transport machinery for inductive
recursive carriers), every law in this package is a direct consequence of
one pattern match reducing by computation — there is no induction anywhere
in this file, since none of `Option`/`Result`/`Either`'s combinators
recurse.

## 6. Findings

No kernel-reduction defect, sugar candidate, or abstraction candidate
surfaced while authoring this package — every combinator and its defining
equations follow the same direct case-split-and-`Refl` shape already
established by the `List` floor, with no new proof technique needed.

## 7. References

- **Option type** — Wikipedia,
  <https://en.wikipedia.org/wiki/Option_type> — general orientation on the
  `Option`/`Result` family and the "elsewhere" bias of `and_then`-style
  chaining.
- **Either (Haskell)** — Wikipedia / the Haskell `Data.Either` module —
  general orientation for readers porting from Haskell, where `Either` is
  the canonical neutral binary sum this package's `Either` mirrors in
  shape (left-biased `either` eliminator, `Left`/`Right` naming).

## 8. Trust & derivation

**Spec catalog entry:** `docs/program/wp/ds-3-sum-type-combinators.md`
(`Option`/`Result` combinators) and `docs/program/wp/either-catalog-package.md`
(`Either`).

**Public API (stable names):** `get_or_else`/`is_some`/`or_else` and their
defining equations (`Option`); `map_err`/`and_then`/`unwrap_or` and their
defining equations (`Result`); `Either`, `either`, `map_left`/`map_right`,
`swap`, and their defining equations (`Either`).

**Source map:**

| Reader task | Section |
|---|---|
| Understand why this package exists / the `Either`-vs-`Result` history | [§1](#1-motivation), [§5](#5-design-notes) |
| Find `Either`'s raw declaration | [§2](#2-definition) |
| Find a specific combinator and its defining equation | [§4.1](#41-option)–[§4.3](#43-either) |

**Derivation path from built-ins.** `Either` is a checked `data` inductive
(kernel-admitted by positivity) — the only new carrier this package
introduces; `Option`/`Result` are reused from the prelude unchanged. Every
combinator is an ordinary `fn`/`const` over one of the three sums; no
`declare_primitive`/`declare_postulate`/`declare_opaque` appears anywhere
in this package.

**`trusted_base()` delta: zero**, confirmed both by construction (zero
postulated law fields, no primitive — every proof reduces by `Refl`) and
by this package's own acceptance tests, which assert the `trusted_base()`
set is byte-for-byte unchanged across elaborating this file.

**Proof families.** Each combinator is proved independently, by direct
case-split: state the combinator, then its defining equation(s), one per
constructor case — no shared proof machinery across combinators, and no
induction (§5).

**Consumers.** `crates/ken-elaborator/tests/ds3_sum_combinators_acceptance.rs`
(`Option`/`Result`) and
`crates/ken-elaborator/tests/either_catalog_package_acceptance.rs`
(`Either`) exercise every combinator and law directly, plus AC8
discriminators confirming each law's specific claim (not a weaker one) is
what's actually proved.

**Validation evidence.** `ken check` on this file's tangled `` ```ken ``
fence elaborates clean; the two acceptance suites above (run in CI at
merge) are this package's behavioral proof.
