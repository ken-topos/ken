# Surface reference — the practical shape of Ken

How to actually write the pieces of a Ken program: the purity keywords,
`data`/`match`, refinement types, `class`/`instance`, effect rows, and this
guide's own literate `.ken.md` format. This strand is task-first ("how do I
write X") — for the exhaustive grammar and the normative contract, see
`../../spec/30-surface/`, chapter-linked throughout.

Every ` ```ken example ` block below elaborates against the real toolchain
(`ken run` over this file); every ` ```ken reject ` block is checked to
actually fail. Nothing here is aspirational syntax — where the spec allows a
form that no landed catalog code actually uses (implicit `{A : Type}`
parameters, most visibly), this strand says so and shows the form that is
actually in use instead.

## Index

1. [Purity keywords: `const`/`fn`/`proc`](#1-purity-keywords-constfnproc)
2. [`def`: transparent definitions](#2-def-transparent-definitions)
3. [`data` and `match`](#3-data-and-match)
4. [Refinement types](#4-refinement-types)
5. [`class` and `instance`](#5-class-and-instance)
6. [Effect rows (`visits`)](#6-effect-rows-visits)
7. [Named proof claims: `prop`, `lemma`, `proof`](#7-named-proof-claims-prop-lemma-proof)
8. [The `.ken.md` literate format](#8-the-kenmd-literate-format)

## 1. Purity keywords: `const`/`fn`/`proc`

A definition's keyword **declares its static purity** — checked against both
the signature and the body, not a comment
(`spec/30-surface/33-declarations.md §1`):

- **`const`** — a pure value, zero explicit value parameters.
- **`fn`** — a pure function, one or more explicit value parameters.
- **`proc … visits ρ`** — potentially impure/imperative; the *only* keyword
  that may carry an effect row (§6).

All three keywords in one runnable block. A literate entry's compiled fences
tangle into one module, and `ken run` executes its last definition — so this
block, being runnable, ends in a nullary `proc main` (§8 covers why a catalog
*package* entry, a library rather than a runnable file, carries none):

```ken
data Color = Red | Green | Blue

const favorite : Color = Blue

fn isRed (c : Color) : Bool =
  match c { Red ⇒ True ; Green ⇒ False ; Blue ⇒ False }

proc announce (c : Color) : IO Unit visits [Console] =
  match c {
    Red   ⇒ print_line "it's red" ;
    Green ⇒ print_line "it's green" ;
    Blue  ⇒ print_line "it's blue"
  }

proc main : IO Unit visits [Console] = announce favorite
```

**Every landed catalog type parameter is explicit**, `(a : Type)`, never the
spec's proposed implicit `{A : Type}` form — grep `catalog/packages/` and
every generic `fn`/`const`/`class` binds its type parameter as an ordinary
leading `(a : Type)` argument that the caller supplies at every call site
(see `map.ken`'s `insert (k : Type) (v : Type) (leq : …) …`). Write new
catalog code the same way: explicit type parameters, applied positionally.

The purity check is real, not advisory — an `fn` that performs an effect is
rejected, and a `proc` declared with no row that performs one is rejected
(§6 has the row-omission case). Here, `print_line` needs `[Console]`, and
`fn` carries no row at all:

```ken reject
fn announceWrong (c : Color) : IO Unit = print_line "not allowed in fn"
```

## 2. `def`: transparent definitions

`def T … = …` is a **definition**: a base type narrowed by conditions (a
refinement/`Σ`/Π-type abbreviation), or the zero-condition case — a plain
alias (`spec/30-surface/33-declarations.md §1`). `def` was spelled `type`
before `SURF-def-refinement`; `type` is now reserved, not a declaration
keyword.

The load-bearing surface fact is that `def` is **transparent** — it unfolds
by δ, so a value at the defined name is interchangeable with a value at its
underlying type, no wrap/unwrap step required. This is exactly the property
that makes `def` the right tool for a refinement alias (`{n : Int | …}`
named once, reused everywhere `PosInt` appears) instead of a `data`
wrapper: a `def`-refined value is, at the kernel, still just the underlying
type's value.

`five` below is usable directly wherever an `Int` is expected — `def` never
introduces a boundary the way an inductive `data` former does:

```ken example
def PosInt = { n : Int | Equal Bool (leq_int 0 n) True }

const five : PosInt = 5

fn addToPosInt (n : Int) (p : PosInt) : Int = add_int n p

const ten : Int = addToPosInt five five
```

A plain, zero-condition `def` is just a name for an existing type — still
transparent, so it changes nothing about how the underlying values behave:

```ken example
def Age = Int

const myAge : Age = 30

fn addYears (n : Int) (a : Age) : Int = add_int n a

const later : Int = addYears 5 myAge
```

Contrast this with `data` (§3): an inductive type former **never**
δ-unfolds, so a `data`-wrapped value is a genuinely different, opaque type
that must be pattern-matched apart before its payload is usable —
`def`'s transparency is the exception, not the default. `Box` below is a
`data` type — opaque, never δ-unfolds — so unlike the `def Age` case above, a
`Box` value is NOT usable where a plain `Int` is expected, even though it is
"just an Int" underneath:

```ken reject
data Box = MkBox Int

fn addYearsWrong (n : Int) (b : Box) : Int = add_int n b
```

## 3. `data` and `match`

`data` declares a genuine inductive type: real constructors, a real
generated eliminator, and `match` compiles to that eliminator with a
**checked exhaustiveness** requirement
(`spec/30-surface/34-data-match.md §1, §4`). Every constructor must be
covered — but a final `_`/variable arm is the sanctioned way to cover the
remaining constructors (`34 §4.1`, `§4.2`); there is no way to *skip* a
case, not a ban on wildcards.

```ken example
data Shape = Circle Int | Rectangle Int Int

fn area (s : Shape) : Int =
  match s {
    Circle r      ⇒ mul_int r r ;
    Rectangle w h ⇒ mul_int w h
  }
```

A `match` missing a constructor is a hard elaboration error, not a runtime
possibility — the surface catches it before the kernel ever sees the term.
Below, `Caution` and `Go` are left unhandled, so it rejects as non-exhaustive:

```ken reject
data TrafficLight = Stop | Caution | Go

fn isStop (t : TrafficLight) : Bool =
  match t { Stop ⇒ True }
```

`Option`/`Result` are ordinary `data` declarations built the same way — no
special-cased sum type:

```ken example
fn safeHead (a : Type) (xs : List a) : Option a =
  match xs {
    Nil      ⇒ None a ;
    Cons x _ ⇒ Some a x
  }
```

## 4. Refinement types

`{ x : A | φ }` is the type of `x : A` for which `φ x` holds. It elaborates
to `A` plus an emitted proof obligation, never a new kernel type former
(`spec/30-surface/34-data-match.md §5`) — a refined value is, at runtime,
just an `A`. This is the mechanism a `fn`'s return type uses to state a real
postcondition instead of describing it only in a comment. Below, the
refinement is the postcondition: `absInt` really does return a non-negative
Int, and `leq_int 0 y` is the same `Bool`-valued comparator the
lawful-classes package builds `Ord` on top of:

```ken example
fn absInt (x : Int) : { y : Int | Equal Bool (leq_int 0 y) True } =
  match leq_int 0 x {
    True  ⇒ x ;
    False ⇒ sub_int 0 x
  }
```

A refinement can conjoin more than one property with the prelude's `And`
(real landed idiom — `conformance/challenge/C5-verified-sort` proves a
`sort` this way: the result is a `List a` that is *both* sorted *and* a
permutation of the input, in one refinement). A function parameter can be
refined too, the mirror case of a refined result — a refined PARAMETER, here
accepting only Booleans equal to `True`, the same shape
`catalog/packages/Data/Collections/Collections.ken`'s `trueRefinementProject`
uses:

```ken example
fn projectTrue (x : { b : Bool | Equal Bool b True }) : Bool = x
```

## 5. `class` and `instance`

A `class` is an ordinary record of operations (and, when the class states
laws, of proof obligations); `instance` is an ordinary record value — no new
kernel feature, a `class` is a right-nested `Σ` exactly like `record`
(`spec/50-stdlib/51-lawful-classes.md`, and every `lawful-classes`/
`lawful-functors` package instance). Reach for a `class` only when you
genuinely need dispatch on a type; if there is exactly one carrier, an
explicit dictionary or a bare top-level `fn` is simpler and equally lawful
(the decomposition strand's §1 covers the choice).

```ken example
class Describe a {
  describe : a → String
}

instance Describe Bool {
  describe = λb. match b { True ⇒ "true" ; False ⇒ "false" }
}

fn announceIt (b : Bool) : String = (Describe_instance_Bool).describe b
```

**Referencing an instance as a value outside a `where`-resolved call** is
the synthesized global `Describe_instance_Bool` above — not `(Describe
Bool)`, which is the class **applied to its head**, i.e. the dictionary's
*type*, not a value, so projecting a field off it fails immediately:

```ken reject
const wrong : String = (Describe Bool).describe True
```

A class field's own type may itself carry a law — the shape every entry in
`catalog/packages/Core/` follows, covered in depth by the proof techniques
strand. `Eq` below elides `sym`/`trans` for brevity; see
`catalog/packages/Core/LawfulClasses.ken` for the full class:

```ken ignore
class Eq a {
  eq   : a → a → Bool ;
  refl : (x : a) → IsTrue (eq x x)
}
```

## 6. Effect rows (`visits`)

A `proc`'s effect row is part of its type: `visits [E₁, …]` names exactly
the effects its body may perform, **statically checked and transitively
inferred** — a call to an effectful function requires its effects to already
be in the caller's own row (`spec/30-surface/36-effects.md §1`).

```ken example
proc greet (name : String) : IO Unit visits [Console] =
  print_line name
```

Omitting a used effect from the row is rejected the same way an exhaustive
match omission is — the checker infers the body's real effects and compares.
Below, the body performs Console but the declared row is empty:

```ken reject
proc silentGreet (name : String) : IO Unit =
  print_line name
```

A row-polymorphic function keeps a helper's effect abstract instead of
committing it to one concrete row — write `[e]` as the row parameter
(`36 §1.5.1`'s row-variable form) when the helper's own effect should be
whatever its caller's effect happens to be, rather than hard-coding
`[Console]` into something that has nothing to do with the console.

## 7. Named proof claims: `prop`, `lemma`, `proof`

Three more declaration keywords name a proof at the surface, on top of the
`Equal`/`IsTrue` goals §4's refinement obligations already build (`33 §8`).
None of them add a new kernel declaration class or a trusted proof table —
they are elaboration vocabulary over ordinary checked terms:

- **`prop`** names a proposition family — a telescope ending in `Omega`
  (§8.1).
- **`proof <name> for <subject>`** names a checked proof term attached to an
  already-resolved subject (§8.2).
- **`lemma`** names a standalone, reusable checked theorem in the ordinary
  module namespace (§8.3).

A `prop` may carry an optional `where` block of constructor-style intro
helpers. As landed today, the elaborator only accepts a **v0 "Omega-clean
seed shape"**: every intro's conclusion must reapply the family to exactly
its own bound parameters, in order — it cannot introduce extra premises or
construct a new argument value (a real inductive relation like "list
append" needs that). Know this going in: a `where` block is not yet a
general way to define an inductive relation; for that, state the property
as a `lemma`'s result type instead (the proof-techniques strand's induction
and motive-construction section covers this) and prove it directly.

Below, `triv`'s conclusion reapplies `Trivial` to exactly its own parameters
`a` and `x` — inside the v0 seed shape — and the intro helper is addressed
through the family name, `Family.introName`:

```ken example
prop Trivial (a : Type) (x : a) : Omega where {
  triv : Trivial a x
}

const sampleInt : Int = 42

const trivialSample : Trivial Int sampleInt = Trivial.triv Int sampleInt
```

Outside the seed shape: below, `nil`'s conclusion applies `AppendsTo` to
`Nil a` and `ys`/`ys` — not to `AppendsTo`'s own three bound parameters in
order — so this rejects with `"prop intro 'nil' is outside the v0
Omega-clean seed shape"`, not a generic syntax error:

```ken reject
prop AppendsTo (a : Type) (xs : List a) (ys : List a) (zs : List a) : Omega
  where {
    nil : AppendsTo a (Nil a) ys ys
  }
```

`proof <name> for <subject>` attaches a checked proof to a subject that is
already resolved — the proof's own telescope must repeat the subject's
public call telescope exactly, and the canonical path to use it is
`subject::proof_name`:

```ken example
fn doubleIt (x : Int) : Int = add_int x x

proof trivial for doubleIt (x : Int) : Trivial Int x = Trivial.triv Int x

const attachedSample : Trivial Int sampleInt = doubleIt::trivial sampleInt
```

`lemma` is the standalone form — parameterized like a function, instantiated
by ordinary application, no attachment to a subject:

```ken example
lemma trivialAny (a : Type) (x : a) : Trivial a x = Trivial.triv a x

const lemmaSample : Trivial Int sampleInt = trivialAny Int sampleInt
```

**A `lemma` body cannot call itself.** `lemma`'s elaboration resolves its
body against a fresh scope, not the enclosing recursive group, so a
self-recursive `lemma` fails with an unresolved reference to its own name.
The working idiom is the same shape catalog code already uses for a proof
that needs induction: prove it as an ordinary recursive `fn` (or the
proof-techniques strand's induction pattern), then expose a **thin,
non-recursive `lemma` wrapper** that applies it. Below, the recursion
(structurally descending on `ys`) lives in an ordinary `fn`, gated by the
same SCT check any recursive `fn` goes through (the proof-techniques
strand's non-termination-hazards section covers the gate itself); the
`lemma` wrapper is not recursive itself — it just applies the `fn` that did
the recursion, so it resolves cleanly:

```ken example
fn trivialByList (a : Type) (x : a) (b : Type) (ys : List b) : Trivial a x =
  match ys {
    Nil      ⇒ Trivial.triv a x ;
    Cons _ t ⇒ trivialByList a x b t
  }

lemma trivialByListLemma (a : Type) (x : a) (b : Type) (ys : List b) : Trivial a x =
  trivialByList a x b ys
```

## 8. The `.ken.md` literate format

This guide (and every catalog package) is itself written in `.ken.md`: an
ordinary Markdown file whose fenced code blocks carry a checked role. Only
an exact ` ```ken ` fence tangles into the compiled module; the fence
taxonomy is `07-catalog-style-guide.md §3`:

| Fence | Tangles? | CI checks | Use |
|---|---|---|---|
| `` ```ken `` | yes | must elaborate | the component itself |
| `` ```ken ignore `` | no | none | pure illustration, e.g. a fragment |
| `` ```ken reject `` | no | asserts rejection | a pitfall kept honest |
| `` ```ken example `` | no | elaborates, not shipped | checked usage/walkthrough |

Everything above ` ```ken example ` in this strand is *checked, not
shipped* — a reader can trust it compiles without it becoming part of any
module. Use ` ```ken reject ` for a genuine negative example (something
that must fail and whose failure is worth teaching), and reserve
` ```ken ignore ` for a fragment too small to elaborate on its own (a bare
type signature, a snippet missing its surrounding declarations).

## Design notes

- **Why explicit type parameters over implicit ones**: the elaborator's
  `instance_search`/unification story for implicit-argument inference is
  still developing (the CAT-2 `class`-dictionary rulings track this); explicit
  parameters are unambiguous today and cost the caller one extra argument at
  each call site. Prefer them until the guide says otherwise.
- **Why refinements over a separate assertion mechanism**: a refinement is
  *in the type*, so every caller sees the postcondition at the call site
  without reading the body — the same reason `sort`'s result type states
  "sorted and a permutation" rather than a comment saying so.

## Findings

- **`prop`'s `where` block is v0-limited to a trivial seed shape**
  (`elab.rs::validate_seed_prop_shape`): an intro's conclusion must reapply
  the family to exactly its own bound parameters, so genuine inductive
  relations (list-append-shaped, order-shaped) can't be expressed as a
  `prop where` block yet — §7 above teaches the `lemma`-with-explicit-goal
  workaround instead. Routed as a language-surface follow-on (retro-action
  wiring, `README.md`): widening the seed shape to accept fresh premise
  binders and constructor-applied arguments is Language's call, not a
  catalog-authoring one.

## References

- Wikipedia — [Purely functional
  programming](https://en.wikipedia.org/wiki/Purely_functional_programming) —
  orientation on the pure/effectful split this strand's §1 checks
  structurally.
- Wikipedia — [Refinement type](https://en.wikipedia.org/wiki/Refinement_type) —
  orientation on §4's `{x:A|φ}` form; Ken's own encoding (carrier plus
  obligation) is stated in `spec/30-surface/34-data-match.md §5`, not
  derived from any particular implementation.
