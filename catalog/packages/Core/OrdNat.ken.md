# `Ord Nat` — a lawful total order on `Nat`, and its operations

`Nat` is inductive and kernel-proved, so its total order can be a real,
zero-`Axiom` `instance Ord Nat` — unlike `Int`, which is an opaque
primitive and can only postulate its laws. This entry exports that
instance plus the small set of `Nat` operations (`min`, `max`, `sub`,
`compare`) that build on it.

## Index

1. [Motivation](#1-motivation)
2. [Order and its laws](#2-order-and-its-laws)
3. [Using it](#3-using-it)
4. [Laws  proofs](#4-laws--proofs)
5. [Design notes](#5-design-notes)
6. [References](#6-references)
7. [Trust  derivation](#7-trust--derivation)

**Named reading paths**

- *Newcomer* → [Motivation](#1-motivation) → [Using it](#3-using-it)
- *Practitioner* → [Using it](#3-using-it) →
  [Laws  proofs](#4-laws--proofs)
- *Researcher* → [Laws  proofs](#4-laws--proofs) →
  [Design notes](#5-design-notes)
- *Porting from Lean/Agda* → [Design notes](#5-design-notes)

## 1. Motivation

Natural numbers have a structural total order: it is reflexive,
antisymmetric, transitive, and total. This entry exposes that order as an
`Ord Nat` instance, so generic code can use the same ordering interface for
`Nat` that it uses for other ordered carriers. It also provides the everyday
operations `min`, `max`, `sub`, and `compare`.

## 2. Order and its laws

`leq_nat` follows the constructors of `Nat`. Its reflexivity, transitivity,
and antisymmetry proofs have the same shape as the corresponding `Ord`
fields. Because `IsTrue b` is defined as `Equal Bool b True`, these proofs
fit those fields by ordinary unfolding; no conversion is needed. Each
recursive proof uses a small dependency-first island: the structural helper is
immediately followed by the original-name checked lemma it establishes.

```ken
fn leq_nat (m : Nat) (n : Nat) : Bool =
  match m {
    Zero ⇒ True ;
    Suc m2 ⇒ match n { Zero ⇒ False ; Suc n2 ⇒ leq_nat m2 n2 }
  }

lemma refl_leq_nat (x : Nat) : Equal Bool (leq_nat x x) True =
  match x { Zero ⇒ Proved ; Suc x2 ⇒ refl_leq_nat x2 }

lemma trans_leq_nat
  (x : Nat)
  : (y : Nat) -> (z : Nat) -> Equal Bool (leq_nat x y) True ->
    Equal Bool (leq_nat y z) True -> Equal Bool (leq_nat x z) True =
  match x {
    Zero ⇒ λy.λz.λp.λq. Proved ;
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

lemma antisym_leq_nat
  (x : Nat)
  : (y : Nat) -> Equal Bool (leq_nat x y) True ->
    Equal Bool (leq_nat y x) True -> Equal Nat x y =
  match x {
    Zero ⇒
      λy. match y {
        Zero ⇒ λp.λq. Proved ;
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
          (Equal Bool (leq_nat y Zero) True) Proved ;
    Suc x2 ⇒
      match y {
        Zero ⇒
          Inr (Equal Bool (leq_nat (Suc x2) Zero) True)
              (Equal Bool (leq_nat Zero (Suc x2)) True) Proved ;
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
short-circuits on its first argument, so
`or_eq_true_to_is_true_bool_or` case-splits on `p` once to compute `bool_or`'s
reduction, then discharges each side directly (the `Inl` branch needs
`cong`/`trans` to carry the equality through `bool_or`'s first-argument
position; the `Inr` branch's own case-split on `p` makes both `bool_or
True q` and `bool_or False q` reduce to a literal, so `Proved`/the hypothesis
itself close it):

```ken
lemma or_eq_true_to_is_true_bool_or
  (p : Bool) (q : Bool)
  (h : Or (Equal Bool p True) (Equal Bool q True))
  : IsTrue (bool_or p q) =
  match h {
    Inl hp ⇒
      trans Bool (bool_or p q) (bool_or True q) True
            (cong Bool Bool p True (λv. bool_or v q) hp) Proved ;
    Inr hq ⇒ match p { True ⇒ Proved ; False ⇒ hq }
  }

instance Ord Nat {
  leq     = leq_nat ;
  refl    = refl_leq_nat ;
  antisym = antisym_leq_nat ;
  trans   = trans_leq_nat ;
  total   = λx.λy. or_eq_true_to_is_true_bool_or (leq_nat x y) (leq_nat y x) (total_leq_nat x y)
}
```

`min` and `max` follow `leq_nat`'s recursion directly. `sub` is saturating
natural-number subtraction, and `compare` returns the three-way result
`OrdResult`:

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
lemma two_leq_three : IsTrue (leq_nat (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))) = Proved

const min_of_two_and_three : Nat = min (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))
const max_of_two_and_three : Nat = max (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))

const compare_two_three : OrdResult = compare (Suc (Suc Zero)) (Suc (Suc (Suc Zero)))
const compare_three_three : OrdResult =
  compare (Suc (Suc (Suc Zero))) (Suc (Suc (Suc Zero)))
```

`Ord_instance_Nat` is the dictionary value generated by the
`instance Ord Nat { ... }` declaration. Its `.leq` field is an ordinary value;
the `.total` projection below establishes an `Omega`-typed fact and is therefore
presented as a checked lemma:

```ken example
const ord_nat_leq : Bool = (Ord_instance_Nat).leq (Suc Zero) (Suc (Suc Zero))
lemma ord_nat_total : IsTrue (bool_or (leq_nat (Suc Zero) Zero) (leq_nat Zero (Suc Zero))) =
  (Ord_instance_Nat).total (Suc Zero) Zero
```

## 4. Laws  proofs

`min`/`max`/`sub` earn their place with the computation facts a caller
relies on:

```ken example
lemma min_zero_left (n : Nat) : Equal Nat (min Zero n) Zero = Proved

lemma max_zero_left (n : Nat) : Equal Nat (max Zero n) n = Refl

lemma sub_zero_right (a : Nat) : Equal Nat (sub a Zero) a = Refl
```

`min_zero_left` closes with `Proved`: `min Zero n` reduces to the literal `Zero`
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
for the overall proof STYLE (case-split to `Proved`/`absurd`), which
`or_eq_true_to_is_true_bool_or` also follows.

**Local definitions keep proof terms direct.** The order and arithmetic
operations are written here as structural definitions, so their computation
rules remain visible beside the laws that use them.

## 6. References

- **Wikipedia** — [Total order](https://en.wikipedia.org/wiki/Total_order)
  — general orientation on the reflexive/antisymmetric/transitive/total
  axioms this entry's `Ord Nat` instantiates.
- **Lean 4 core** — `Nat.le` and its `LinearOrder`/`Nat`-specific decidable
  order instances (`Init/Data/Nat/Basic.lean`, part of the Lean 4
  repository, Apache-2.0) — <https://github.com/leanprover/lean4> —
  consulted for the general shape of a structural `Nat` order (no source
  copied, `CLEAN-ROOM.md`).

## 7. Trust  derivation

1. **Public API.** `leq_nat`, `refl_leq_nat`, `trans_leq_nat`, `antisym_leq_nat`,
   `total_leq_nat`, `or_eq_true_to_is_true_bool_or`, `Ord_instance_Nat` (via
   `instance Ord Nat`), `min`, `max`, `sub`, `compare`.
2. **Source map.**

   | Task | Section |
   |---|---|
  | See the shape | [Order and its laws](#2-order-and-its-laws) |
   | Use it | [Using it](#3-using-it) |
   | Check the computation facts | [Laws  proofs](#4-laws--proofs) |
   | Why `total` needed a bridge, `refl`/`trans`/`antisym` didn't | [Design notes](#5-design-notes) |

3. **Derivation path.** `leq_nat` and its order laws are structural
   definitions and proofs over `Nat`. `or_eq_true_to_is_true_bool_or`
   converts the disjoint totality result into the Boolean form required by
   `Ord`. `min`, `max`, `sub`, and `compare` are ordinary recursive
   functions.
4. **`trusted_base()` delta.** **Zero.** Every `Ord` law field is a real,
   kernel-checked proof term. The entry introduces no `Axiom`, primitive, or
   postulate.
5. **Proof families.** Reflexivity, transitivity, and antisymmetry follow
   the structure of `Nat`; totality uses the two-case Boolean-disjunction
   bridge.
6. **Consumers.** Generic ordered algorithms can use the `Ord Nat` instance;
   direct callers can use the arithmetic and comparison operations.
7. **Validation evidence.** The catalog checks the four `Ord` fields, the
   totality bridge, the instance declaration, the zero-`Axiom` trust posture,
   and every source, example, and rejection fence.
