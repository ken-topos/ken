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
2. [`data` and `match`](#2-data-and-match)
3. [Refinement types](#3-refinement-types)
4. [`class` and `instance`](#4-class-and-instance)
5. [Effect rows (`visits`)](#5-effect-rows-visits)
6. [The `.ken.md` literate format](#6-the-kenmd-literate-format)

## 1. Purity keywords: `const`/`fn`/`proc`

A definition's keyword **declares its static purity** — checked against both
the signature and the body, not a comment
(`spec/30-surface/33-declarations.md §1`):

- **`const`** — a pure value, zero explicit value parameters.
- **`fn`** — a pure function, one or more explicit value parameters.
- **`proc … visits ρ`** — potentially impure/imperative; the *only* keyword
  that may carry an effect row (§5).

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

-- A literate entry's compiled fences tangle into one module, and `ken run`
-- executes its last definition — so a runnable file's final compiled
-- declaration is a nullary `proc main`, exactly like the CLI fixtures and
-- runnable examples. A catalog PACKAGE entry is a library, not a runnable
-- file, and carries no `proc main`.
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
(§5 has the row-omission case):

```ken reject
-- A `fn` cannot perform an effect — `print_line` needs `[Console]`, and
-- `fn` carries no row at all.
fn announceWrong (c : Color) : IO Unit = print_line "not allowed in fn"
```

## 2. `data` and `match`

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
possibility — the surface catches it before the kernel ever sees the term:

```ken reject
data TrafficLight = Stop | Caution | Go

fn isStop (t : TrafficLight) : Bool =
  match t { Stop ⇒ True }   -- Caution, Go unhandled: rejected as non-exhaustive
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

## 3. Refinement types

`{ x : A | φ }` is the type of `x : A` for which `φ x` holds. It elaborates
to `A` plus an emitted proof obligation, never a new kernel type former
(`spec/30-surface/34-data-match.md §5`) — a refined value is, at runtime,
just an `A`. This is the mechanism a `fn`'s return type uses to state a real
postcondition instead of describing it only in a comment:

```ken example
-- The refinement is the postcondition: `absInt` really does return a
-- non-negative Int. `leq_int 0 y` is the same `Bool`-valued comparator the
-- lawful-classes package builds `Ord` on top of.
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
refined too, the mirror case of a refined result:

```ken example
-- A refined PARAMETER: only Booleans equal to `True` are accepted.
-- Equivalent shape to `catalog/packages/collections/collections.ken`'s
-- `trueRefinementProject`.
fn projectTrue (x : { b : Bool | Equal Bool b True }) : Bool = x
```

## 4. `class` and `instance`

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

A class field's own type may itself carry a law — the shape every entry in
`catalog/packages/lawful-classes/` follows, covered in depth by the proof
techniques strand:

```ken ignore
class Eq a {
  eq   : a → a → Bool ;
  refl : (x : a) → IsTrue (eq x x)
  -- … `sym`, `trans` — see catalog/packages/lawful-classes/lawful_classes.ken
}
```

## 5. Effect rows (`visits`)

A `proc`'s effect row is part of its type: `visits [E₁, …]` names exactly
the effects its body may perform, **statically checked and transitively
inferred** — a call to an effectful function requires its effects to already
be in the caller's own row (`spec/30-surface/36-effects.md §1`).

```ken example
proc greet (name : String) : IO Unit visits [Console] =
  print_line name
```

Omitting a used effect from the row is rejected the same way an exhaustive
match omission is — the checker infers the body's real effects and compares:

```ken reject
proc silentGreet (name : String) : IO Unit =
  print_line name    -- performs Console but declares no row at all
```

A row-polymorphic function keeps a helper's effect abstract instead of
committing it to one concrete row — write `[e]` as the row parameter
(`36 §1.5.1`'s row-variable form) when the helper's own effect should be
whatever its caller's effect happens to be, rather than hard-coding
`[Console]` into something that has nothing to do with the console.

## 6. The `.ken.md` literate format

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

None yet — this strand is the V1 draft. As catalog entries are authored
against it, route a recurring shape the surface should sugar to Ergo, and a
gap in this strand back into the guide (retro-action wiring, `README.md`).

## References

- Wikipedia — [Purely functional
  programming](https://en.wikipedia.org/wiki/Purely_functional_programming) —
  orientation on the pure/effectful split this strand's §1 checks
  structurally.
- Wikipedia — [Refinement type](https://en.wikipedia.org/wiki/Refinement_type) —
  orientation on §3's `{x:A|φ}` form; Ken's own encoding (carrier plus
  obligation) is stated in `spec/30-surface/34-data-match.md §5`, not
  derived from any particular implementation.
