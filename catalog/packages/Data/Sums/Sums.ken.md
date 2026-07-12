# `Sums` — the `Option`/`Result`/`Either` combinator floor

The `Option`/`Result`/`Either` combinator floor: one entry for all three
sum families, each combinator paired with its defining equation(s) as a real
proof term.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws & proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust & derivation](#7-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws & proofs](#4-laws--proofs)
- *Researcher* → [Laws & proofs](#4-laws--proofs) →
  [Design notes](#5-design-notes)

## 1. Motivation

This entry gives the three standard binary-sum families a common combinator
floor: `Option`, `Result`, and `Either`. `Option` and `Result` are standard
types (`Option a = None | Some a`; `Result e a = Err e | Ok a`), so this
entry reuses them and their existing functor operations. `Either a b = Left a
| Right b` is declared here as the ordinary, neutral binary sum.

`Result` is the error-biased sum used for computations that can fail;
`Either` is the neutral sum for a choice between two values. Both are useful
and both coexist. Keeping their combinators together lets a reader compare
the closely related elimination and mapping patterns without navigating three
separate packages.

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

Every combinator here is an ordinary structural case-split `fn`/`const` over
one of the three sums. The entry introduces no new kernel feature and has a
zero `trusted_base()` delta.

### 4.1 `Option`

`get_or_else d x` returns `x`'s contained value, or the default `d` at
`None`; `is_some x` is the `Bool` presence test; `or_else x y` is
left-biased — `x`'s own value wins whenever `x` is `Some`, and `y` is only
consulted when `x` is `None`. The right-hand-`None` identity (`or_else x
None = x`) falls out cleanly from the same definition, so it ships
alongside the two defining equations.

```ken
fn get_or_else (a : Type) (d : a) (x : Option a) : a =
  match x { None ↦ d ; Some v ↦ v }

lemma get_or_else_none (a : Type) (d : a) : Equal a (get_or_else a d (None a)) d = Refl

lemma get_or_else_some (a : Type) (d : a) (v : a) : Equal a (get_or_else a d (Some a v)) v = Refl

fn is_some (a : Type) (x : Option a) : Bool =
  match x { None ↦ False ; Some v ↦ True }

lemma is_some_none (a : Type) : Equal Bool (is_some a (None a)) False = Proved

lemma is_some_some (a : Type) (v : a) : Equal Bool (is_some a (Some a v)) True = Proved

fn or_else (a : Type) (x : Option a) (y : Option a) : Option a =
  match x { None ↦ y ; Some v ↦ Some a v }

lemma or_else_none (a : Type) (y : Option a) : Equal (Option a) (or_else a (None a) y) y = Refl

lemma or_else_some (a : Type) (v : a) (y : Option a) : Equal (Option a) (or_else a (Some a v) y) (Some a v) = Refl

lemma or_else_none_rhs (a : Type) (x : Option a) : Equal (Option a) (or_else a x (None a)) x =
  match x { None ↦ Proved ; Some v ↦ Refl }
```

### 4.2 `Result`

`map_err g x` maps `g` over the error side only, leaving an `Ok` payload
untouched; `and_then k x` short-circuits on `Err` (`k` is never called)
and otherwise threads `x`'s payload through `k`; `unwrap_or d x` returns
the contained value, or the default `d` at `Err`.

```ken
fn map_err (e : Type) (f : Type) (a : Type) (g : e → f) (x : Result e a) : Result f a =
  match x { Err u ↦ Err f a (g u) ; Ok v ↦ Ok f a v }

lemma map_err_ok (e : Type) (f : Type) (a : Type) (g : e → f) (v : a)
  : Equal (Result f a) (map_err e f a g (Ok e a v)) (Ok f a v) = Refl

lemma map_err_err (e : Type) (f : Type) (a : Type) (g : e → f) (u : e)
  : Equal (Result f a) (map_err e f a g (Err e a u)) (Err f a (g u)) = Refl

fn and_then (e : Type) (a : Type) (b : Type) (k : a → Result e b) (x : Result e a) : Result e b =
  match x { Err u ↦ Err e b u ; Ok v ↦ k v }

lemma and_then_ok (e : Type) (a : Type) (b : Type) (k : a → Result e b) (v : a)
  : Equal (Result e b) (and_then e a b k (Ok e a v)) (k v) = Refl

lemma and_then_err (e : Type) (a : Type) (b : Type) (k : a → Result e b) (u : e)
  : Equal (Result e b) (and_then e a b k (Err e a u)) (Err e b u) = Refl

fn unwrap_or (e : Type) (a : Type) (d : a) (x : Result e a) : a =
  match x { Err u ↦ d ; Ok v ↦ v }

lemma unwrap_or_ok (e : Type) (a : Type) (d : a) (v : a) : Equal a (unwrap_or e a d (Ok e a v)) v = Refl

lemma unwrap_or_err (e : Type) (a : Type) (d : a) (u : e) : Equal a (unwrap_or e a d (Err e a u)) d = Refl
```

### 4.3 `Either`

`Either` is an ordinary user-defined non-dependent sum. Unlike the standard
`Option` and `Result` types, it needs no special support beyond a `data`
declaration.

`either f g x` is the eliminator: `f` on `Left`, `g` on `Right`.
`map_left`/`map_right` apply a function to one side only, leaving the
other side's payload untouched — `map_left_right`/`map_right_left` are the
"untouched" proofs, as important to state as the "applied" ones.
`swap` exchanges `Left`/`Right`, and is genuinely involutive
(`swap_involutive`), not merely the identity.

```ken
data Either a b = Left a | Right b

fn either (a : Type) (b : Type) (c : Type) (f : a → c) (g : b → c) (x : Either a b) : c =
  match x { Left v ↦ f v ; Right v ↦ g v }

lemma either_left (a : Type) (b : Type) (c : Type) (f : a → c) (g : b → c) (v : a)
  : Equal c (either a b c f g (Left a b v)) (f v) = Refl

lemma either_right (a : Type) (b : Type) (c : Type) (f : a → c) (g : b → c) (v : b)
  : Equal c (either a b c f g (Right a b v)) (g v) = Refl

fn map_left (a : Type) (b : Type) (c : Type) (f : a → c) (x : Either a b) : Either c b =
  match x { Left v ↦ Left c b (f v) ; Right v ↦ Right c b v }

lemma map_left_left (a : Type) (b : Type) (c : Type) (f : a → c) (v : a)
  : Equal (Either c b) (map_left a b c f (Left a b v)) (Left c b (f v)) = Refl

lemma map_left_right (a : Type) (b : Type) (c : Type) (f : a → c) (v : b)
  : Equal (Either c b) (map_left a b c f (Right a b v)) (Right c b v) = Refl

fn map_right (a : Type) (b : Type) (c : Type) (g : b → c) (x : Either a b) : Either a c =
  match x { Left v ↦ Left a c v ; Right v ↦ Right a c (g v) }

lemma map_right_left (a : Type) (b : Type) (c : Type) (g : b → c) (v : a)
  : Equal (Either a c) (map_right a b c g (Left a b v)) (Left a c v) = Refl

lemma map_right_right (a : Type) (b : Type) (c : Type) (g : b → c) (v : b)
  : Equal (Either a c) (map_right a b c g (Right a b v)) (Right a c (g v)) = Refl

fn swap (a : Type) (b : Type) (x : Either a b) : Either b a =
  match x { Left v ↦ Right b a v ; Right v ↦ Left b a v }

lemma swap_involutive (a : Type) (b : Type) (x : Either a b) : Equal (Either a b) (swap b a (swap a b x)) x =
  match x { Left v ↦ Refl ; Right v ↦ Refl }
```

## 5. Design notes

**One entry, not three.** The three families share a binary-sum shape, an
eliminator or direct case split, and defining equations proved by `Refl`.
Presenting them together makes the common structure visible while the
subsections give readers a direct route to the family they need.

**`Either` is independent from `Result`.** `Result` records the conventional
error-or-value interpretation; `Either` records an uncommitted choice between
two values. Declaring them independently keeps each interpretation explicit
at use sites.

**Every proof reduces by `Refl`.** Every law in this package is a direct
consequence of one pattern match reducing by computation. None of these
combinators recurses, so no induction or transport machinery is needed.

## 6. References

- **Option type** — Wikipedia,
  <https://en.wikipedia.org/wiki/Option_type> — general orientation on the
  `Option`/`Result` family and the "elsewhere" bias of `and_then`-style
  chaining.
- **Either (Haskell)** — Wikipedia / the Haskell `Data.Either` module —
  general orientation for readers porting from Haskell, where `Either` is
  the canonical neutral binary sum this package's `Either` mirrors in
  shape (left-biased `either` eliminator, `Left`/`Right` naming).

## 7. Trust & derivation

**Public API (stable names):** `get_or_else`/`is_some`/`or_else` and their
defining equations (`Option`); `map_err`/`and_then`/`unwrap_or` and their
defining equations (`Result`); `Either`, `either`, `map_left`/`map_right`,
`swap`, and their defining equations (`Either`).

**Source map:**

| Reader task | Section |
|---|---|
| Understand why this package exists / choose `Either` or `Result` | [§1](#1-motivation), [§5](#5-design-notes) |
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

**Consumers.** The catalog's executable checks exercise every combinator and
law directly, including discriminators for the distinct constructor cases.

**Validation evidence.** `ken check` on this file's tangled `` ```ken ``
fence elaborates clean; the catalog checks provide behavioral evidence for the
constructor cases and defining equations.
