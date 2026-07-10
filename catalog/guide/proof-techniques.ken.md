# Proof techniques — how to actually discharge a law in Ken

Ken's laws are ordinary `Ω`-propositions and its proofs are ordinary terms
(`spec/10-kernel/16-observational.md`) — there is no separate tactic
language, `sorry`, or postponed goal. This strand is the practice of closing
those goals: choosing the right terminal move, structuring a case-split so a
hypothesis stays usable, decidable equality via `sound`/`complete`, why
`funext` needs no lemma, and the non-termination hazards a proof author
needs to see coming before the kernel rejects the definition outright.

Every example below is checked against the real elaborator; every reject
example is checked to actually fail, with the real error message quoted so a
reader recognizes it when they hit it themselves.

## Index

1. [`tt` vs. `Refl`: the two-way
   discriminator](#1-tt-vs-refl-the-two-way-discriminator) (§1.1: when
   neither closes)
2. [Induction and motive construction](#2-induction-and-motive-construction)
3. [Decidable equality: the `sound`/`complete`
   pattern](#3-decidable-equality-the-soundcomplete-pattern)
4. [`funext` is definitional](#4-funext-is-definitional)
5. [Non-termination hazards](#5-non-termination-hazards)

Shared definitions for this strand's examples. `cong` — lifting equality of
endpoints through a function — is `catalog/packages/Core/Transport.ken`'s
idiom, inlined here so this strand's examples are self-contained:

```ken
fn bool_and (a : Bool) (b : Bool) : Bool = match a { True ⇒ b ; False ⇒ False }

fn list_append (a : Type) (xs : List a) (ys : List a) : List a =
  match xs {
    Nil       ⇒ ys ;
    Cons x t  ⇒ Cons a x (list_append a t ys)
  }

fn cong (ty : Type) (ty2 : Type) (x : ty) (y : ty) (f : ty → ty2)
         (p : Equal ty x y) : Equal ty2 (f x) (f y) =
  J (λy' _. Equal ty2 (f x) (f y')) Refl p

fn bool_eq (a : Bool) (b : Bool) : Bool =
  match a { True ⇒ b ; False ⇒ match b { True ⇒ False ; False ⇒ True } }

fn not_bool (b : Bool) : Bool = match b { True ⇒ False ; False ⇒ True }
fn flip_bool (b : Bool) : Bool = match b { False ⇒ True ; True ⇒ False }
```

## 1. `tt` vs. `Refl`: the two-way discriminator

A proof's terminal step at an equation goal is **`tt` or `Refl` depending on
what the two endpoints reduce to — never a uniform choice**. Ken's
observational equality collapses two occurrences of the **same nullary
constructor** to `Top` (`spec/10-kernel/16-observational.md`, the K7 rule):
once that collapse fires, the goal is `Top`-shaped, not `Eq`-shaped, and
`Refl` — which only checks against an `Eq`-shaped goal — no longer applies.
A **neutral** (stuck) endpoint never collapses and stays `Eq`-shaped — but
`Refl` needs more than "stuck": it needs the goal's two sides to be
**convertible**, which is guaranteed when they are the *same* term (the
trivial case below), not merely when both happen to be stuck. §1.1 below
covers the case where neither applies.

`bool_and True True` reduces to `True`, so `Equal Bool (bool_and True True)
True` collapses all the way to `Top`:

```ken example
const with_tt : Equal Bool (bool_and True True) True = tt
```

This fails with `"Refl expects an `Eq`-shaped goal"` — the goal already
collapsed to `Top` before `Refl` was checked against it:

```ken reject
const with_refl : Equal Bool (bool_and True True) True = Refl
```

An abstract (neutral) variable never collapses, so the same shape flips:

```ken example
fn self_eq_refl (x : Bool) : Equal Bool (bool_and x x) (bool_and x x) = Refl
```

This fails: the goal never reduced to `Top` (`x` is abstract), so there is
nothing for `tt` (a `Top` introduction) to close:

```ken reject
fn self_eq_tt (x : Bool) : Equal Bool (bool_and x x) (bool_and x x) = tt
```

### 1.1 When neither closes: opaque primitives don't reduce under conversion

`self_eq_refl` above works because its goal's two sides are the **identical**
term (`bool_and x x` on both sides) — trivially convertible, no reduction
needed. A goal whose two sides are only *equal at runtime*, not the same
term, needs one of them to actually **reduce** to the other during
conversion — and a `declare_primitive` operation (`eq_int`, `and_bool`,
`leq_int`, …) **never unfolds under conversion, even applied to concrete
values**: the interpreter reduces it at runtime, but the kernel's
convertibility check does not. A goal built from one is **permanently
stuck**, whatever the arguments:

```ken
const five : Int = 5
```

This fails with `"Refl: the two sides of the goal are not convertible"` —
`eq_int five five` never reduces to `True` under conversion, even though
`five` is concrete and the two arguments are literally the same value:

```ken reject
const prim_eq_refl : Equal Bool (eq_int five five) True = Refl
```

This fails too, for the same reason: the goal never reduced to `Top` either:

```ken reject
const prim_eq_tt : Equal Bool (eq_int five five) True = tt
```

The honest closer is a VISIBLE postulate, the same audited-delta shape
`Int`'s own Eq/Ord instances use (§3) — the proposition is true, just not
kernel-reducible at this layer:

```ken example
const prim_eq_axiom : Equal Bool (eq_int five five) True = Axiom
```

This is not a corner case to memorize and forget: **any law whose operation
bottoms out in a primitive, applied to values that are not already literal
constants in the goal itself, needs this same honest `Axiom` treatment** —
`tt`/`Refl` are not a fallback pair that always covers a stuck goal.

**The check, in order:** reduce both endpoints. Same nullary constructor (or
a non-nullary constructor whose every component also collapses) → `Top` →
`tt`. The identical term on both sides (or otherwise genuinely convertible,
e.g. via a **transparent** `fn`'s own ι-reduction) → `Refl`. Stuck on an
**opaque primitive** with no further reduction available on either side →
neither closes — the honest move is a visible `Axiom` (§1.1). Never assume a
base case defaults to one of the three from its shape alone — check the
reduced endpoints and what kind of operation they're stuck on, every time.

## 2. Induction and motive construction

Ken has no separate induction tactic: an inductive proof is an ordinary
**structurally recursive `fn`**, SCT-checked exactly like any other
recursive definition (§5). The base case typically closes with `tt`
(constructor endpoints collapse); the step case typically needs `cong` to
lift the induction hypothesis under the outer constructor, because the two
sides of the step's goal are **neutral** (an abstract tail `t` never
reduces further) and so never collapse on their own. Below is the right unit
for `list_append`: the base case has both sides reduce to the constructor
`Nil a` → `tt`; the step case's goal is `Cons x (list_append t Nil)` vs.
`Cons x t` — neutral in the abstract tail `t`, so the recursive call's
result (the induction hypothesis) is lifted under `Cons x` by `cong`:

```ken example
fn list_right_unit (a : Type) (xs : List a)
  : Equal (List a) (list_append a xs (Nil a)) xs =
  match xs {
    Nil      ⇒ tt ;
    Cons x t ⇒ cong (List a) (List a)
                     (list_append a t (Nil a)) t
                     (λl. Cons a x l)
                     (list_right_unit a t)
  }
```

**Bind each case-split variable as its own function, before the hypothesis
that depends on it.** A law with more than one case-split parameter and a
hypothesis about both (`(x:a)(y:a)(p:P x y) → Concl`) is tempting to bind
flat — but `match x { … }` only narrows the **goal**, never a **sibling**
binder's already-fixed type, so `p` stays symbolic in `x`/`y` inside every
branch and can't be reused where a branch needs it concretely. Instead,
case-split each variable through its **own** `match`-returning-a-function
layer, and only introduce the hypothesis's `λ` *after* every relevant
variable is already concrete — exactly the shape `bool_eq_sound`/
`bool_eq_complete` use in §3, and the shape every law field in
`catalog/packages/Core/LawfulClasses.ken` follows. Once the
hypothesis's binder-time type is concrete, a branch whose hypothesis is
*already* the goal (e.g. an "impossible" combination where the hypothesis
and the goal are the same false equation) can reuse it directly — no
`absurd` detour needed in that case, and `absurd` is exactly what closes the
branches where the hypothesis genuinely is impossible (§3).

## 3. Decidable equality: the `sound`/`complete` pattern

`spec/50-stdlib/51-lawful-classes.md`'s `DecEq a` class states two directions
for a `Bool`-valued `eq`: `sound` (`eq x y = True` implies the real,
propositional `Equal a x y`) and `complete` (the reverse). Both directions
follow the §2 case-split-then-lambda shape, closing each of the four
constructor combinations with `tt` (matching endpoints) or `absurd`
(mismatched endpoints — the hypothesis, once concrete, is a false equation
between different nullary constructors, which collapses to `Bottom`, K5):

```ken example
fn bool_eq_sound (x : Bool) : (y : Bool) → IsTrue (bool_eq x y) → Equal Bool x y =
  match x {
    True  ⇒ λy. match y {
              True  ⇒ λh. tt ;
              False ⇒ λh. absurd h
            } ;
    False ⇒ λy. match y {
              True  ⇒ λh. absurd h ;
              False ⇒ λh. tt
            }
  }

fn bool_eq_complete (x : Bool) : (y : Bool) → Equal Bool x y → IsTrue (bool_eq x y) =
  match x {
    True  ⇒ λy. match y {
              True  ⇒ λp. tt ;
              False ⇒ λp. absurd p
            } ;
    False ⇒ λy. match y {
              True  ⇒ λp. absurd p ;
              False ⇒ λp. tt
            }
  }
```

An **audited-delta** carrier (a K1 primitive like `Int`, opaque to δ, with
no induction principle) cannot follow this pattern — its `sound`/`complete`
are not kernel-provable at all, and the honest spelling is the visible
`Axiom` postulate (`catalog/packages/Core/LawfulClasses.ken`'s
`Int` instance), never a proof that only looks real.

## 4. `funext` is definitional

Ken's equality at a function type **reduces to the pointwise family**:
`Equal ((x:A)→B) f g` is, definitionally, `(x:A) → Equal B (f x) (g x)`
(`spec/10-kernel/16-observational.md §2.2`, the observational-equality
computation rule for `Π`). Function extensionality therefore needs **no
axiom and no lemma** — a bare pointwise proof checks directly against a
function-equality goal, because the goal *is* that pointwise type after one
reduction step. `not_bool` and `flip_bool` below are two syntactically
different functions; proving them equal AS FUNCTIONS needs no `funext`
call, only a pointwise proof, because `Equal (Bool -> Bool) f g`
whnf-reduces to exactly that pointwise Pi type:

```ken example
const not_functions_equal : Equal (Bool → Bool) not_bool flip_bool =
  λb. match b { True ⇒ tt ; False ⇒ tt }
```

**Do not proliferate a second, function-level law field when a class's
canonical law is more natural stated pointwise** (e.g. a `Functor`'s
`map_id`): the point-free and pointwise statements are the same
proposition up to this one reduction, so pick whichever reads better at the
call site and treat the other as free.

## 5. Non-termination hazards

Every transparent (δ-unfoldable) recursive definition is admitted only if
it passes the kernel's **size-change termination (SCT)** gate at definition
time (`spec/10-kernel/17-conversion.md §4`) — this is the *sole*
termination guarantee; there is no fuel or cycle guard on δ-unfolding
itself, so a definition that slips past SCT does not merely run slowly, it
makes conversion (and therefore type-checking) **loop**. `list_right_unit`
above passes because its recursive call is on the strictly smaller tail
`t`. A self-reference with no strictly-decreasing argument is rejected,
whatever form it takes — a bare, unapplied self-reference is the minimal
case:

This fails with `"SCT: idempotent self-loop has no strictly-decreasing
parameter"`:

```ken reject
const bad : Bottom = bad
```

**Every occurrence of a recursive definition inside its own body counts**,
not only occurrences applied to arguments (`spec/10-kernel/17-conversion.md
§4.1`'s call-graph-completeness invariant) — an unapplied self-reference
passed to another function is exactly as rejected as the direct form above,
so there is no laundering route through a combinator. When a genuinely
well-founded recursion doesn't obviously decrease on a single structural
argument, restructure it around an explicit accumulator or a decreasing
index rather than reaching for a workaround — the gate's job is to reject
exactly the definitions that would make the kernel unsound if admitted.

## Design notes

- **Why no `sorry`/postponed goals exist**: every hole in a Ken proof is
  either a real term or the honest, visible `Axiom` postulate (§3) — never
  a silent gap. A law field that "will be proved later" is not merged with
  a placeholder; it is either proved now or the catalog entry's Trust
  section discloses the delta.
- **Why the restructuring discipline in §2 matters more than it looks**: it
  is the difference between a law that is provable *today* and one that
  looks stateable but silently can't close in any branch that needs a
  sibling hypothesis concretely.

## Findings

- A landed conformance fixture
  (`conformance/challenge/C1-deceq-noncanonical/unsound-deceq-decimal.ken`)
  writes `Refl Bool True` — the **applied** form, which fails with
  `UnresolvedCon` (`Refl` is checked-only, matching solely a bare
  `RCon("Refl")`, `crates/ken-elaborator/src/elab.rs`). Routed to CV as a
  fixture-fidelity bug. **Correction, via CV's own differential
  investigation:** the fix is not simply "use bare `Refl`" — bare `Refl`
  *also* fails there (`"the two sides of the goal are not convertible"`),
  and so does `tt`, because `decimalEq` bottoms out in the opaque
  primitives `eq_int`/`and_bool`, which never reduce under conversion
  (§1.1 — this Finding is exactly what §1.1 now teaches, added because of
  it). The honest witness is `Axiom`. This is the campaign's own retro loop
  working as designed: authoring against the guide surfaced a gap, and
  fixing the gap sharpened the guide itself.

## References

- Lee, Jones & Ben-Amram, *"The Size-Change Principle for Program
  Termination"*, POPL 2001 — the general theory §5's SCT gate implements;
  Ken's own gate and its call-graph-completeness invariant are stated in
  `spec/10-kernel/17-conversion.md §4`, not derived from any particular
  implementation.
- Wikipedia — [Function
  extensionality](https://en.wikipedia.org/wiki/Function_extensionality) —
  orientation on §4; Ken's own definitional treatment is
  `spec/10-kernel/16-observational.md §2.2`, distinct from the axiomatic
  treatment common in other proof assistants (Lean/Agda/Idris), where
  `funext` is a postulate applied explicitly rather than a reduction rule.

As with the surface reference strand's own compiled block (§1 there), this
entry's tangled fences run through `ken run`, so this final compiled
declaration is a nullary `proc main`:

```ken
proc main : IO Unit visits [Console] = print_line "proof techniques ok"
```
