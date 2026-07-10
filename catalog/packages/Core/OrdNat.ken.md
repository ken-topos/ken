# `Ord Nat` — a lawful total order on `Nat`, and its operations

`Nat` is inductive and kernel-proved, so its total order can be a real,
zero-`Axiom` `instance Ord Nat` — unlike `Int`, which is an opaque
primitive and can only postulate its laws. This entry exports that
instance plus the small set of `Nat` operations (`min`, `max`, `sub`,
`compare`) that build on it.

## Index

1. [Motivation](#1-motivation)
2. [Definition](#2-definition)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [Findings](#6-findings)
7. [References](#7-references)
8. [Trust  derivation](#8-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* → [Laws  proofs](#4-laws--proofs) →
  [Design notes](#5-design-notes)
- *Porting from Lean/Agda* → [Design notes](#5-design-notes)

## 1. Motivation

`catalog/packages/Data/Collections/Map.ken` proves a full `Nat` order
(reflexivity, antisymmetry, transitivity, totality) as a *private*,
build-internal family — real, kernel-checked proofs, but with no public
name and no connection to the landed `class Ord` (`Core/LawfulClasses.ken`)
that every other ordered carrier (`Bool`) already instantiates. That leaves
`Nat` unusable anywhere a caller wants "the" order on `Nat` polymorphically
(`where Ord a`, or a generic function parametric in an `Ord a` dictionary)
— exactly the situation `Ord Bool`'s existence already solves for `Bool`.
This entry lifts the proofs out, unchanged in substance, into a real
exported `instance Ord Nat`, and collects the day-to-day `Nat` operations
(`min`/`max`/`sub`/`compare`) that read most naturally against that order.

## 2. Definition

`leq_nat` and its four laws are unchanged from
`catalog/packages/Data/Collections/Map.ken:2571–2621` — same recursion
shape, same proof terms. `refl`/`trans`/`antisym` slot into `class Ord`'s
fields with **zero conversion code**: `IsTrue b` is *defined* as `Equal
Bool b True` (`Core/LawfulClasses.ken`), so `refl_leq_nat`/`trans_leq_nat`/
`antisym_leq_nat` — already stated as `Equal Bool … True` — satisfy the
`IsTrue`-phrased fields by ordinary definitional unfolding, no bridge
needed:

```ken
fn leq_nat (m : Nat) (n : Nat) : Bool =
  match m {
    Zero ⇒ True ;
    Suc m2 ⇒ match n { Zero ⇒ False ; Suc n2 ⇒ leq_nat m2 n2 }
  }

fn refl_leq_nat (x : Nat) : Equal Bool (leq_nat x x) True =
  match x { Zero ⇒ tt ; Suc x2 ⇒ refl_leq_nat x2 }

fn trans_leq_nat
  (x : Nat)
  : (y : Nat) -> (z : Nat) -> Equal Bool (leq_nat x y) True ->
    Equal Bool (leq_nat y z) True -> Equal Bool (leq_nat x z) True =
  match x {
    Zero ⇒ λy.λz.λp.λq. tt ;
    Suc x2 ⇒
      λy. match y {
        Zero ⇒ λz.λp.λq. absurd p ;
        Suc y2 ⇒
          λz. match z {
            Zero ⇒ λp.λq. absurd q ;
            Suc z2 ⇒ λp.λq. trans_leq_nat x2 y2 z2 p q
          }
      }
  }

fn antisym_leq_nat
  (x : Nat)
  : (y : Nat) -> Equal Bool (leq_nat x y) True ->
    Equal Bool (leq_nat y x) True -> Equal Nat x y =
  match x {
    Zero ⇒
      λy. match y {
        Zero ⇒ λp.λq. tt ;
        Suc y2 ⇒ λp.λq. absurd q
      } ;
    Suc x2 ⇒
      λy. match y {
        Zero ⇒ λp.λq. absurd p ;
        Suc y2 ⇒ λp.λq. cong Nat Nat x2 y2 Suc (antisym_leq_nat x2 y2 p q)
      }
  }

fn total_leq_nat (x : Nat) (y : Nat)
  : Or (Equal Bool (leq_nat x y) True) (Equal Bool (leq_nat y x) True) =
  match x {
    Zero ⇒
      Inl (Equal Bool (leq_nat Zero y) True)
          (Equal Bool (leq_nat y Zero) True) tt ;
    Suc x2 ⇒
      match y {
        Zero ⇒
          Inr (Equal Bool (leq_nat (Suc x2) Zero) True)
              (Equal Bool (leq_nat Zero (Suc x2)) True) tt ;
        Suc y2 ⇒
          match total_leq_nat x2 y2 {
            Inl h ⇒
              Inl (Equal Bool (leq_nat (Suc x2) (Suc y2)) True)
                  (Equal Bool (leq_nat (Suc y2) (Suc x2)) True) h ;
            Inr h ⇒
              Inr (Equal Bool (leq_nat (Suc x2) (Suc y2)) True)
                  (Equal Bool (leq_nat (Suc y2) (Suc x2)) True) h
          }
      }
  }
```

`total`'s field, `IsTrue (bool_or (leq x y) (leq y x))`, is genuinely a
different shape from `total_leq_nat`'s `Or`-of-equalities — `bool_or`
(`Core/LawfulClasses.ken`) short-circuits on its first argument, so
`or_eq_true_to_is_true_bool_or` case-splits on `p` once to compute `bool_or`'s
reduction, then discharges each side directly (the `Inl` branch needs
`cong`/`trans` to carry the equality through `bool_or`'s first-argument
position; the `Inr` branch's own case-split on `p` makes both `bool_or
True q` and `bool_or False q` reduce to a literal, so `tt`/the hypothesis
itself close it):

```ken
fn or_eq_true_to_is_true_bool_or
  (p : Bool) (q : Bool)
  (h : Or (Equal Bool p True) (Equal Bool q True))
  : IsTrue (bool_or p q) =
  match h {
    Inl hp ⇒
      trans Bool (bool_or p q) (bool_or True q) True
            (cong Bool Bool p True (λv. bool_or v q) hp) tt ;
    Inr hq ⇒ match p { True ⇒ tt ; False ⇒ hq }
  }

instance Ord Nat {
  leq     = leq_nat ;
  refl    = refl_leq_nat ;
  antisym = antisym_leq_nat ;
  trans   = trans_leq_nat ;
  total   = λx.λy. or_eq_true_to_is_true_bool_or (leq_nat x y) (leq_nat y x) (total_leq_nat x y)
}
```

`min`/`max` follow `leq_nat`'s own recursion directly; `sub` is the
existing saturating `Nat` monus (`Data/Collections/Collections.ken`,
unchanged); `compare` is new, built from `leq_nat`, and returns the
existing three-way `OrdResult` (`Data/Collections/Collections.ken`,
inlined here — self-containment, avoiding a dependency on the whole
unrelated `Collections.ken` file for one three-constructor `data`):

```ken
data OrdResult = Lt | Eq | Gt

fn min (m : Nat) (n : Nat) : Nat =
  match m {
    Zero ⇒ Zero ;
    Suc m2 ⇒ match n { Zero ⇒ Zero ; Suc n2 ⇒ Suc (min m2 n2) }
  }

fn max (m : Nat) (n : Nat) : Nat =
  match m {
    Zero ⇒ n ;
    Suc m2 ⇒ match n { Zero ⇒ m ; Suc n2 ⇒ Suc (max m2 n2) }
  }

fn sub (a : Nat) (b : Nat) : Nat =
  match b {
    Zero ⇒ a ;
    Suc n ⇒ match a { Zero ⇒ Zero ; Suc m ⇒ sub m n }
  }

fn compare (a : Nat) (b : Nat) : OrdResult =
  match leq_nat a b {
    True ⇒ match leq_nat b a { True ⇒ Eq ; False ⇒ Lt } ;
    False ⇒ Gt
  }
```

## 3. Using it

```ken example
const two_leq_three : IsTrue (leq_nat (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))) = tt

const min_of_two_and_three : Nat = min (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))
const max_of_two_and_three : Nat = max (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))

const compare_two_three : OrdResult = compare (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))
const compare_three_three : OrdResult =
  compare (Suc (Suc (Suc Zero))) (Suc (Suc (Suc Zero)))
```

`Ord_instance_Nat` is the synthesized dictionary value for `§2`'s
`instance Ord Nat { ... }`; `.leq`/`.total` project its fields directly in
VALUE position (dot-projection inside a TYPE annotation is a separate,
currently-unparseable surface gap, `§6` Finding — the type below spells
`leq_nat` directly instead, the same operation `.leq` is bound to):

```ken example
const ord_nat_leq : Bool = (Ord_instance_Nat).leq (Suc Zero) (Suc (Suc Zero))
const ord_nat_total : IsTrue (bool_or (leq_nat (Suc Zero) Zero) (leq_nat Zero (Suc Zero))) =
  (Ord_instance_Nat).total (Suc Zero) Zero
```

## 4. Laws  proofs

`min`/`max`/`sub` earn their place with the computation facts a caller
relies on:

```ken example
lemma min_zero_left (n : Nat) : Equal Nat (min Zero n) Zero = tt

lemma max_zero_left (n : Nat) : Equal Nat (max Zero n) n = Refl

lemma sub_zero_right (a : Nat) : Equal Nat (sub a Zero) a = Refl
```

`min_zero_left` closes with `tt`: `min Zero n` reduces to the literal `Zero`
regardless of `n` (both sides collapse to the same nullary constructor,
`§1` of `catalog/guide/proof-techniques.ken.md`). `max_zero_left` and
`sub_zero_right` close with `Refl`: `max Zero n`'s recursive definition and
`sub`'s own `b = Zero` branch make `n`/`a` (an abstract, stuck variable)
appear literally unchanged on the reduced side without any further
constructor-level reduction — the goal stays `Eq`-shaped, not collapsed to
`Top`.

The companion fact `sub n n = Zero` (self-subtraction) is also true, but —
unlike `sub_zero_right` — needs induction on `n` (`sub`'s own structural
recursion doesn't reduce for an ABSTRACT `n` matched against itself), so
`Refl` alone cannot close it; this entry deliberately doesn't prove that
separately-inductive law, to keep scope small, and names the gap here
rather than carrying an unproved claim:

```ken reject
lemma subSelfIsZeroWrong (n : Nat) : Equal Nat (sub n n) Zero = Refl
```

## 5. Design notes

**Why `refl`/`trans`/`antisym` needed no conversion, but `total` did.**
`class Ord`'s three "pointwise" fields are phrased directly over `IsTrue`,
which unfolds to exactly the shape `leq_nat`'s own proofs already carry
(`Equal Bool … True`) — a byproduct of `IsTrue`'s own definition, not
anything specific to `Nat`. `total`, by contrast, asks for a single
`Bool`-valued disjunction (`bool_or`) rather than a disjoint sum
(`total_leq_nat`'s `Or`) — a genuinely different STATEMENT shape (which side
holds is erased into one Bool, not carried as a tag), so a real conversion
lemma is unavoidable there regardless of carrier. `Ord Bool`'s own `total`
sidesteps this because `Bool`'s order is provable by direct case-split
down to concrete constructors without ever building an intermediate `Or` —
it isn't a template for the `Or`-to-`bool_or` bridge specifically, only
for the overall proof STYLE (case-split to `tt`/`absurd`), which
`or_eq_true_to_is_true_bool_or` also follows.

**Kept `Map.ken`'s private copy in place.** After this export, `Map.ken`'s
capstone proofs could in principle consume `Ord_instance_Nat.leq` instead
of their own private `leq_nat`, eliminating the duplication. This entry
does not attempt that: `Map.ken`'s five capstone laws are load-bearing,
previously-gate-cleared proofs (`map-verified-laws`), and re-pointing them
at a differently-sourced (if propositionally identical) `leq_nat` risks
touching definitional-equality-sensitive proof steps for a purely
cosmetic win. Filed as a follow-up (`§6`), not attempted here, per the
frame's explicit "de-dup only if safe" guardrail.

**`Collections.ken`'s `min`/`nat_sub` also stay in place**, for the
identical reason — `length_take_min`/`slice` reference them by name
in their own already-landed proofs, and this entry's `min`/`sub` are
propositionally (not referentially) the same operations. Also a named
follow-up, not a risk taken here.

## 6. Findings

- **Kernel-reduction defect:** none.
- **Abstraction candidate → Ergo/catalog follow-up:** two safe-looking
  de-duplications are deferred rather than attempted in this WP —
  `Map.ken`'s private `leq_nat`/order family could consume this entry's
  `Ord_instance_Nat`, and `Collections.ken`'s `min`/`nat_sub` could import
  this entry's copies — both behavior-preserving in principle, but each
  touches a file with its own already-landed, gate-cleared proofs
  (`Map.ken`'s capstone laws; `Collections.ken`'s `length_take_min`/
  `slice`). Worth a dedicated, narrowly-scoped follow-up WP per file
  rather than folding into this "near-mechanical" export.
- **Sugar candidate → Ergo (parser):** `(instanceValue).field` projection
  parses in VALUE position (`§2`/`§3` use it freely) but NOT inside a TYPE
  annotation — `IsTrue (bool_or ((Ord_instance_Nat).leq x y) …)` as a
  written TYPE fails with `expected RParen, found Dot`. Confirmed
  empirically (`§3`'s worked example was rewritten to spell `leq_nat`
  directly in the type position instead, the same operation `.leq` is
  bound to). Real but low-severity — the value-position path always has a
  workaround (name the underlying operation directly); worth fixing so a
  reader doesn't have to know the workaround exists.

## 7. References

- **Wikipedia** — [Total order](https://en.wikipedia.org/wiki/Total_order)
  — general orientation on the reflexive/antisymmetric/transitive/total
  axioms this entry's `Ord Nat` instantiates.
- **Lean 4 core** — `Nat.le` and its `LinearOrder`/`Nat`-specific decidable
  order instances (`Init/Data/Nat/Basic.lean`, part of the Lean 4
  repository, Apache-2.0) — <https://github.com/leanprover/lean4> —
  consulted for the general shape of a structural `Nat` order (no source
  copied, `CLEAN-ROOM.md`).

## 8. Trust  derivation

1. **Spec / WP.** `docs/program/wp/ds-2-ord-nat-export.md` (this entry's
   build WP); the order laws' contract is `class Ord`
   (`Core/LawfulClasses.ken`, `spec/50-stdlib/51-lawful-classes.md`).
2. **Public API.** `leq_nat`, `refl_leq_nat`, `trans_leq_nat`, `antisym_leq_nat`,
   `total_leq_nat`, `or_eq_true_to_is_true_bool_or`, `Ord_instance_Nat` (via
   `instance Ord Nat`), `min`, `max`, `sub`, `compare`.
3. **Source map.**

   | Task | Section |
   |---|---|
   | See the shape | [Definition](#2-definition) |
   | Use it | [Using it](#3-using-it) |
   | Check the computation facts | [Laws  proofs](#4-laws--proofs) |
   | Why `total` needed a bridge, `refl`/`trans`/`antisym` didn't | [Design notes](#5-design-notes) |

4. **Derivation path.** `leq_nat`/`refl_leq_nat`/`trans_leq_nat`/`antisym_leq_nat`/
   `total_leq_nat` — the SAME proof terms already kernel-checked in
   `Map.ken:2571–2621` (copied, not re-derived). `or_eq_true_to_is_true_bool_or` —
   this entry's own new proof, built from the landed `cong`/`trans`
   (`Core/Transport.ken`). `instance Ord Nat` — an ordinary `class`
   instance declaration (`elab_instance_decl`), the same mechanism
   `instance Ord Bool` already uses. `min`/`max`/`sub`/`compare` —
   ordinary surface `fn`, structural recursion.
5. **`trusted_base()` delta.** **Zero.** Every law field
   (`refl`/`antisym`/`trans`/`total`) is a real, kernel-checked proof
   term — no `Axiom`, no new `declare_primitive`/`declare_postulate`, no
   new `Term`/`Decl` variant. This is the acceptance bar the frame set
   (`Nat` is inductive, so this must hold, unlike `Ord Int`); confirmed by
   `crates/ken-elaborator/tests/ds2_ord_nat_acceptance.rs`'s AC (grep for
   `Axiom` in this entry's own source — zero hits — plus a direct
   `trusted_base()` set-difference check against the pre-instance
   environment).
6. **Proof families.** `refl`/`trans`/`antisym` — direct reuse (zero new
   proof steps). `total` — one new case-split proof
   (`or_eq_true_to_is_true_bool_or`), two branches, no induction beyond what
   `total_leq_nat` already supplies.
7. **Consumers.** None yet in this catalog; the two named follow-ups
   (`§5`, `§6`) are the natural next consumers (`Map.ken`,
   `Collections.ken`).
8. **Validation evidence.**
   `crates/ken-elaborator/tests/ds2_ord_nat_acceptance.rs` — the direct-
   slot probes for `refl`/`trans`/`antisym`, the `total` bridge, the full
   `instance Ord Nat` declaration, the zero-`Axiom`/`trusted_base()`
   check, and elaborating this entry's `` ```ken ``/`` ```ken example ``/
   `` ```ken reject `` fences through the literate extractor (loaded with
   `Core/Transport.ken` + `Core/LawfulClasses.ken` first — this entry
   depends on both, unlike `Core/EmptyDec.ken.md`'s fully self-contained
   pilot; `ken run` on the bare file fails for the same reason `EmptyDec`
   already found and routed — no standalone cross-package import
   mechanism, `EmptyDec.ken.md §6`, not a new finding here).
