# `Empty` and `Dec` — computational falsity and decidability

`Empty` is Ken's Type-sorted false, and `Dec P` is a decidability container
that lets you *compute* whether a proposition holds and recover the proof
or refutation. Together they let ordinary code case-split on a proposition
the way it already case-splits on a `Bool`, without losing the proof.

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
  [Laws  proofs](#4-laws--proofs) → [Design notes](#5-design-notes)
- *Porting from Lean/Agda* → [Design notes](#5-design-notes)

## 1. Motivation

Ken already has two ways to state "false": the proof-irrelevant `Bottom :
Omega` (the logic's falsehood — two proofs of `Bottom` are always equal,
because propositions carry no information beyond their truth) and, for any
`Bool`-classified test, the ordinary `True`/`False` tags. Neither is enough
for *decidability*: given a proposition `P`, a decision procedure needs to
return **which** disjunct holds — a proof of `P`, or a proof that `P` is
impossible — and that answer has to be **inspectable**, so downstream code
can branch on it. A proof-irrelevant `Bottom`-based `Or` can't carry that
information (case-splitting on it can't tell you which side you're on); an
ordinary `Bool` can tell you which side, but throws the proof away.

`Empty` and `Dec` close this gap, mirroring Lean's `Decidable` and Agda's
`⊥`/`Dec` (consulted for shape only, `§7`). Every later catalog entry that
needs a real decision procedure — sorting by a decidable order, comparing
keys, checked pattern-refinement — builds on this pair.

## 2. Definition

`Empty`, `Dec`, `Yes`, `No`, and `decide` are standard names, available to
every Ken program rather than declared by this entry. `Dec` accepts a
proposition-valued parameter, a generality not yet expressible in an ordinary
surface `data` declaration. A zero-constructor data type can be declared with
the explicit-family form `data Empty : Type0 where { }`; the legacy
`data D = …` form shown below remains illustrative.

Conceptually, the standard declarations have this shape:

```ken ignore
data Empty : Type0 =

data Dec (P : Ω) : Type0 =
  Yes P
  | No (P → Empty)

fn decide (P : Ω) (d : Dec P) : Bool =
  match d { Yes p ↦ True ; No f ↦ False }
```

What this entry *does* author, as real surface code.

The general Type-sorted eliminator for `Empty` — an uninhabited type
eliminates into anything — is deliberately not named `absurd`: that
identifier is reserved checked-mode surface sugar for `Ω`-classified
`Bottom`-elimination. A declaration named `absurd` is rejected, so
`absurd_empty` is the clear, reachable name this entry uses instead:

```ken
fn absurd_empty (C : Type) (e : Empty) : C = match e {}
```

`Yes`/`No` already work directly as constructors; the lowercase `yes`/`no`
pair below is a purely ergonomic smart-constructor wrapper that reads
better at call sites and mirrors `yes`/`no` on the referenced Lean/Agda
`Decidable`/`Dec`:

```ken
fn yes (prp : Ω) (p : prp) : Dec prp = Yes prp p

fn no (prp : Ω) (f : prp → Empty) : Dec prp = No prp f
```

`DecEq` is stated here so the entry is self-contained. `dec_eq_decides`
works for any `DecEq a` instance; only the worked example needs a concrete
instance in scope:

```ken
class DecEq a {
  eq : a → a → Bool;
  sound : (x : a) → (y : a) → IsTrue (eq x y) → Equal a x y;
  complete : (x : a) → (y : a) → Equal a x y → IsTrue (eq x y)
}

fn bool_eq (a : Bool) (b : Bool) : Bool =
  match a {
    True ↦ b;
    False ↦
      match b {
        True ↦ False;
        False ↦ True
      }
  }

instance DecEq Bool {
  eq = bool_eq;
  sound = λx.match x {
    True ↦ λy.match y {
      True ↦ λp.Proved;
      False ↦ λp.absurd p
    };
    False ↦ λy.match y {
      True ↦ λp.absurd p;
      False ↦ λp.Proved
    }
  };
  complete = λx.match x {
    True ↦ λy.match y {
      True ↦ λp.Proved;
      False ↦ λp.absurd p
    };
    False ↦ λy.match y {
      True ↦ λp.absurd p;
      False ↦ λp.Proved
    }
  }
}
```

The `match e eqn: h` modifier retains the equation between a computed `Bool`
and its branch constructor, so the equation can be used as a proof directly.

`sym`/`trans` are inlined from `catalog/packages/Core/Transport.ken`
(self-containment, the same idiom `catalog/guide/proof-techniques.ken.md`
uses for `cong`) for the No-branch contradiction below:

```ken
lemma sym (ty : Type) (x : ty) (y : ty) (p : Equal ty x y) : Equal ty y x =
  J (λy' _. Equal ty y' x) Refl p

lemma trans
(ty : Type)
(x : ty)
(y : ty)
(z : ty)
(p : Equal
ty
x
y)
(q : Equal
ty
y
z) : Equal
ty
x
z =
  J (λz' _. Equal ty x z') p q
```

The bridge: any `DecEq a` instance decides propositional equality.
`d.eq x y = True` (`Inl p`, `p : Equal Bool (d.eq x y) True`) → `sound`
hands back the proof directly. `d.eq x y = False` (`Inr q`) → assuming a
proof `pxy : Equal a x y` gives `d.complete x y pxy : IsTrue (d.eq x y) =
Equal Bool (d.eq x y) True`; combined with `q` via `sym`/`trans`, that is
`Equal Bool False True`, which reduces to `Bottom`. The `absurd` sugar then
discharges it into `Empty` directly; this bridge is `Ω → Type`, not
`Empty → C`:

```ken
fn dec_eq_decides (a : Type) (d : DecEq a) (x : a) (y : a) : Dec (Equal a x y) =
  match d.eq x y eqn : h {
    True ↦
      Yes
        (Equal a x y)
        (d.sound x y h);
    False ↦
      No
        (Equal a x y)
        (λpxy.
          absurd
            (trans
              Bool
              False
              (d.eq x y)
              True
              (sym Bool (d.eq x y) False h)
              (d.complete x y pxy)))
  }
```

## 3. Using it

`DecEq_instance_Bool` is the dictionary value generated by the
`instance DecEq Bool { ... }` declaration. An instance declaration registers
a global value named after its class and carrier.

```ken example
const true_is_true : Dec (Equal Bool True True) =
  dec_eq_decides Bool DecEq_instance_Bool True True

const true_is_not_false : Dec (Equal Bool True False) =
  dec_eq_decides Bool DecEq_instance_Bool True False
```

`decide` recovers just the `Bool` tag, e.g. for an ordinary conditional:

```ken example
const true_is_true_tag : Bool = decide (Equal Bool True True) true_is_true

const true_is_not_false_tag : Bool = decide (Equal Bool True False) true_is_not_false
```

`yes`/`no` construct `Dec` values directly when you already have the proof
or refutation in hand — no `DecEq` needed:

```ken example
const any_proof_decides : Dec (Equal Bool True True) = yes (Equal Bool True True) Proved

fn refute_true_false (p : Equal Bool True False) : Empty = absurd p

const refutation_decides : Dec (Equal Bool True False) =
  no (Equal Bool True False) refute_true_false
```

`absurd_empty` — an inhabitant of `Empty`, however obtained, discharges ANY
goal:

```ken example
fn contradiction_implies_anything (e : Empty) : Bool = absurd_empty Bool e
```

## 4. Laws  proofs

`Dec`/`Empty` are containers and an eliminator, not a lawful structure with
its own class — the "laws" here are the computation facts that justify
calling `decide` an honest reflection of `Dec`'s tag, stated as `lemma`s
over the concrete `DecEq Bool` instance from `§3` (the guide's `§7` named-
proof-claims form):

```ken example
proof yes_is_true
for
decide : Equal
Bool
(decide
(Equal
Bool
True
True)
true_is_true)
True =
  Proved

proof no_is_false
for
decide : Equal
Bool
(decide
(Equal
Bool
True
False)
true_is_not_false)
False =
  Proved
```

Both close with `Proved`: `decide`/`true_is_true`/`true_is_not_false` are all closed,
fully-applied terms, so both sides reduce to the same nullary `Bool`
constructor and the equality collapses to `Top` before `Proved` is even checked
(the guide's `§1` `Proved`-vs-`Refl` discriminator) — not `Refl`, since neither
side is stuck.

## 5. Design notes

**Why not `Ω`'s `Or`, and why not a homogeneous `Sum`.** A decision
procedure must be able to *large-eliminate* on which disjunct holds — `Or`
(`Ω`-sorted) is proof-irrelevant and cannot large-eliminate into `Type`,
and a homogeneous `Sum a b : Type` cannot mix an `Ω`-sorted proof payload
with a `Type`-sorted refutation payload (Ken is non-cumulative — `Ω` does
not inject into `Type 0`). `Dec`'s two constructors fix each payload's sort
independently, which is exactly the same move the refinement type
`{x : A | φ} = (x : A) × φ` already makes (an `Ω` field on a `Type`-sorted
family, `spec/10-kernel/13-pi-sigma.md:133`) — `Dec` has precedent, not a
new capability class.

**The `DecEq Int` caveat.** `DecEq Int.sound` is `Axiom`-backed (`Int` is
an opaque primitive, no induction) — `dec_eq_decides Int (DecEq Int) x y`
type-checks and is *usable*, but its `Yes` branch's proof rides that
`Axiom`, not a kernel-checked derivation. `§3`'s worked examples
deliberately use `DecEq Bool` (an inductive carrier, honest by the
no-confusion principle — an inductive type's constructors are disjoint and
injective — with zero trusted-base delta) so the showcase is not vacuous — see
`catalog/guide/` `§1.1` for the general opaque-primitive/`Axiom` pattern.

**Zero-constructor `data` declarations** use an explicit-family form, and
**`absurd` is reserved sugar.** An explicit-family zero-constructor block declares an
independently named uninhabited type:

```ken example
data EmptyAttempt : Type where {}

fn absurd_empty_attempt (C : Type) (e : EmptyAttempt) : C = match e {}
```

The legacy `data D = …` form does not take a `:`-ascribed family type, so the
explicit-family `where { }` spelling is the usable form here.

**`absurd` is reserved.** Declaring
`fn absurd (C : Type) (e : Empty) : C = match e { }` fails:
`'absurd' collides with a reserved surface sugar identifier`.

```ken reject
fn absurd (C : Type) (e : Empty) : C = match e {}
```

## 6. References

- **Wikipedia** —
  [Decidability (logic)](https://en.wikipedia.org/wiki/Decidability_(logic))
  — general orientation on decidable propositions.
- **Lean 4 core** — `Decidable` (`Init/Prelude.lean`, part of the Lean 4
  repository, Apache-2.0) — <https://github.com/leanprover/lean4> — the
  `Decidable p := isFalse (p → False) | isTrue p` shape this entry's `Dec`
  mirrors (consulted for shape only, `CLEAN-ROOM.md`; no source copied).
- **Agda standard library** — `Relation.Nullary.Decidable`
  (`src/Relation/Nullary/Decidable.agda`, MIT) —
  <https://github.com/agda/agda-stdlib> — a second reference point for the
  same `yes`/`no`-tagged decision shape, with Agda's `⊥` as the `Type`-
  sorted false this entry's `Empty` mirrors.

## 7. Trust  derivation

1. **Public API.** `Empty`, `absurd_empty`, `Dec`, `Yes`, `No`, `decide`,
   `yes`, `no`, `dec_eq_decides`.
2. **Source map.**

   | Task | Section |
   |---|---|
   | See the shape | [Definition](#2-definition) |
   | Use it | [Using it](#3-using-it) |
   | Check the computation facts | [Laws  proofs](#4-laws--proofs) |
   | Why this shape, not `Or`/`Sum` | [Design notes](#5-design-notes) |

3. **Derivation path.** `Empty`, `Dec`, `Yes`, and `No` are standard
   inductive declarations. `decide` is an ordinary match over `Dec`, and
   `absurd_empty`, `yes`, `no`, and `dec_eq_decides` are ordinary functions.
4. **`trusted_base()` delta.** **Zero new trust category.** The standard
   inductives are kernel-checked; this entry adds no postulate or primitive.
   Instantiating `dec_eq_decides` at a carrier whose `DecEq` instance has an
   audited assumption retains that instance's declared delta.
5. **Proof families.** `dec_eq_decides` case-splits on `d.eq x y`: its true
   branch uses `sound`, and its false branch turns `complete` into a
   contradiction. No induction is required.
6. **Consumers.** Decision procedures for ordering, key comparison, and
   checked pattern refinement build on this pair.
7. **Validation evidence.** The catalog checks that `Dec` eliminates into
   `Type0`, preserves the stated trust posture, and elaborates every source,
   example, and rejection fence.
