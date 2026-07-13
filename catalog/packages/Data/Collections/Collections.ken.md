# `Collections` — the derived `List`/`Nat` combinator floor + string surface

The derived `List`/`Nat` combinator floor (`spec/30-surface/37-strings-collections.md
§2.4/§2.5/§2.5.1/§4.1), the CAT-3 structural/verified-sort/projection-abstraction
slices, and the 5 derived `String` ops built on top of the real
`string_to_list_char`/`list_char_to_string` round trip. Every combinator here
is a termination-checked recursive derived definition over the real generic
eliminator — zero new kernel feature, zero `trusted_base()` delta anywhere in
this file.

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
- *Researcher* →
  [Laws  proofs](#4-laws--proofs) → [Design notes](#5-design-notes)

## 1. Motivation

`spec/30-surface/37-strings-collections.md §2.4/§2.5/§2.5.1/§4.1` (WP
`L3-strings-surface`, slice 2/2 of the string surface — slice 1,
`L3-strings-roundtrip`, landed the real `string_to_list_char`/
`list_char_to_string` round trip this package rides) needs a `List`/`Nat`
combinator floor and five derived `String` operations, stated once as
ordinary checked Ken rather than re-derived per consumer. The floor is 7
combinators, not 6: the WP's original frame assumed a landed saturating
`Nat` subtraction (`sub`) that does not exist (only signed non-saturating
`sub_int` is a primitive) — `nat_sub` is the 7th, derived in `§4.5` below.
Every combinator, law, and string op is a termination-checked recursive
`declare_def` (upgrading opaque to transparent on `sct_check` success) over
the real generic eliminator — a `match` on `List`/`Nat` lowers to a real
`Term::Elim{fam}` (`34 §3`); there is no `elim_List`/`elim_Nat` constant, and
no native interpreter primitive is added for any of these combinators
(Approach A, Architect ruling `evt_4k1yqah3yvpds`: a native primitive would
grow the tested-not-trusted reduction surface for a trivially structural
fold, a subsume-don't-proliferate violation).

## 2. Definition

`OrdResult` is the 3-way comparison result (`§2.5.1`). The landed `Ord Char`
(`catalog/packages/Core/LawfulClasses.ken.md`) is `leq`-only — no `compare`
method — and no `Ordering`/`OrdResult` type exists anywhere else on `main`
(ES2 retired the prelude's `OrdResult` as un-referenced bloat, but explicitly
sanctions a local declaration where genuinely needed — Architect ruling
`evt_1stp9sspm6ag8`, the `natCmp` precedent,
`crates/ken-elaborator/tests/val1_string_literals.rs:334`). Named
`OrdResult`, matching that precedent — never a second name (`Ordering`) for
the same concept; exported here since `compare`'s result type must be
bindable, but not re-promoted to the prelude (that would reopen ES2's
retirement — a second consumer, e.g. a verified `sort` over a richer
carrier, would trigger that subsume decision later, not this package).

The first four of the seven floor combinators follow: `list_append`
(deliberately a distinct name from the landed `Bytes`-domain `append`,
FS-effect, `crates/ken-elaborator/src/bytes.rs` — this is the pure
`List a -> List a -> List a` op and must not shadow or be shadowed by it),
`nth`, `take`, and `drop`. The remaining three (`nat_sub`, `list_eq`,
`list_compare`) are declared in `§4.5`, next to the string ops that need
them.

```ken
data OrdResult = Lt | Eq | Gt

const ord_eq : OrdResult = Eq

const ord_lt : OrdResult = Lt

const ord_gt : OrdResult = Gt

fn ord_result_leq (r : OrdResult) : Bool =
  match r {
    Lt ↦ True;
    Eq ↦ True;
    Gt ↦ False
  }

fn ord_result_dispatch2
      (c : Type)
      (ll : c)
      (le : c)
      (lg : c)
      (el : c)
      (ee : c)
      (eg : c)
      (gl : c)
      (ge : c)
      (gg : c)
      (r : OrdResult)
      (s : OrdResult)
    : c =
  match r {
    Lt ↦
      match s {
        Lt ↦ ll;
        Eq ↦ le;
        Gt ↦ lg
      };
    Eq ↦
      match s {
        Lt ↦ el;
        Eq ↦ ee;
        Gt ↦ eg
      };
    Gt ↦
      match s {
        Lt ↦ gl;
        Eq ↦ ge;
        Gt ↦ gg
      }
  }

lemma ord_result_elim
      (P : OrdResult → Prop) (r : OrdResult) (pLt : P Lt) (pEq : P Eq) (pGt : P Gt)
    : P r =
  match r {
    Lt ↦ pLt;
    Eq ↦ pEq;
    Gt ↦ pGt
  }

lemma ord_result_elim2
      (P : OrdResult → OrdResult → Prop)
      (r : OrdResult)
      (s : OrdResult)
      (pLL : P Lt Lt)
      (pLE : P Lt Eq)
      (pLG : P Lt Gt)
      (pEL : P Eq Lt)
      (pEE : P Eq Eq)
      (pEG : P Eq Gt)
      (pGL : P Gt Lt)
      (pGE : P Gt Eq)
      (pGG : P Gt Gt)
    : P r s =
  match r {
    Lt ↦
      match s {
        Lt ↦ pLL;
        Eq ↦ pLE;
        Gt ↦ pLG
      };
    Eq ↦
      match s {
        Lt ↦ pEL;
        Eq ↦ pEE;
        Gt ↦ pEG
      };
    Gt ↦
      match s {
        Lt ↦ pGL;
        Eq ↦ pGE;
        Gt ↦ pGG
      }
  }

fn pair_compare
      (a : Type)
      (b : Type)
      (cmpa : a → a → OrdResult)
      (cmpb : b → b → OrdResult)
      (x : Pair a b)
      (y : Pair a b)
    : OrdResult =
  match cmpa (pair_fst a b x) (pair_fst a b y) {
    Lt ↦ Lt;
    Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
    Gt ↦ Gt
  }

fn pair_compare_result_of (tail : OrdResult) (head : OrdResult) : OrdResult =
  match head {
    Lt ↦ Lt;
    Eq ↦ tail;
    Gt ↦ Gt
  }

proof eq for pair_compare
      (a : Type)
      (b : Type)
      (cmpa : a → a → OrdResult)
      (cmpb : b → b → OrdResult)
      (x : Pair a b)
      (y : Pair a b)
      (ha : Equal OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_eq)
      (hb : Equal OrdResult (cmpb (pair_snd a b x) (pair_snd a b y)) ord_eq)
    : Equal OrdResult (pair_compare a b cmpa cmpb x y) ord_eq =
  J
    (λr _.
      Equal
        OrdResult
        (match r {
          Lt ↦ Lt;
          Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
          Gt ↦ Gt
        })
        ord_eq)
    hb
    (sym OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_eq ha)

fn pair_compare_lt_cases_eq_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (peq : Equal OrdResult s ord_eq)
      (ptail : Equal OrdResult (cmpb sndx sndy) ord_lt)
    : Or
        (Equal OrdResult s ord_lt)
        (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_lt)) =
  Inr
    (Equal OrdResult s ord_lt)
    (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_lt))
    (and_intro (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_lt) peq ptail)

fn pair_compare_lt_cases_lt_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (plt : Equal OrdResult s ord_lt)
    : Or
        (Equal OrdResult s ord_lt)
        (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_lt)) =
  Inl
    (Equal OrdResult s ord_lt)
    (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_lt))
    plt

fn pair_compare_lt_cases_gt_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (pgt : Equal OrdResult s ord_gt)
      (plt : Equal OrdResult s ord_lt)
    : Or
        (Equal OrdResult s ord_lt)
        (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_lt)) =
  absurd (J (λr _. Equal OrdResult r ord_lt) plt pgt)

fn pair_compare_lt_cases
      (a : Type)
      (b : Type)
      (cmpa : a → a → OrdResult)
      (cmpb : b → b → OrdResult)
      (x : Pair a b)
      (y : Pair a b)
      (h : Equal OrdResult (pair_compare a b cmpa cmpb x y) ord_lt)
    : Or
        (Equal OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_lt)
        (And
          (Equal OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_eq)
          (Equal OrdResult (cmpb (pair_snd a b x) (pair_snd a b y)) ord_lt)) =
  match cmpa (pair_fst a b x) (pair_fst a b y) eqn : ha {
    Lt ↦ pair_compare_lt_cases_lt_at b cmpb (pair_snd a b x) (pair_snd a b y) Lt Proved;
    Eq ↦ pair_compare_lt_cases_eq_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Eq
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_lt)
        h
        ha);
    Gt ↦ pair_compare_lt_cases_gt_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Gt
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_lt)
        h
        ha)
  }

fn pair_compare_gt_cases_eq_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (peq : Equal OrdResult s ord_eq)
      (ptail : Equal OrdResult (cmpb sndx sndy) ord_gt)
    : Or
        (Equal OrdResult s ord_gt)
        (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_gt)) =
  Inr
    (Equal OrdResult s ord_gt)
    (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_gt))
    (and_intro (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_gt) peq ptail)

fn pair_compare_gt_cases_gt_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (pgt : Equal OrdResult s ord_gt)
    : Or
        (Equal OrdResult s ord_gt)
        (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_gt)) =
  Inl
    (Equal OrdResult s ord_gt)
    (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_gt))
    pgt

fn pair_compare_gt_cases_lt_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (plt : Equal OrdResult s ord_lt)
      (pgt : Equal OrdResult s ord_gt)
    : Or
        (Equal OrdResult s ord_gt)
        (And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_gt)) =
  absurd (J (λr _. Equal OrdResult r ord_gt) pgt plt)

fn pair_compare_gt_cases
      (a : Type)
      (b : Type)
      (cmpa : a → a → OrdResult)
      (cmpb : b → b → OrdResult)
      (x : Pair a b)
      (y : Pair a b)
      (h : Equal OrdResult (pair_compare a b cmpa cmpb x y) ord_gt)
    : Or
        (Equal OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_gt)
        (And
          (Equal OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_eq)
          (Equal OrdResult (cmpb (pair_snd a b x) (pair_snd a b y)) ord_gt)) =
  match cmpa (pair_fst a b x) (pair_fst a b y) eqn : ha {
    Lt ↦ pair_compare_gt_cases_lt_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Lt
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_gt)
        h
        ha);
    Eq ↦ pair_compare_gt_cases_eq_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Eq
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_gt)
        h
        ha);
    Gt ↦ pair_compare_gt_cases_gt_at b cmpb (pair_snd a b x) (pair_snd a b y) Gt Proved
  }

lemma pair_compare_eq_cases_eq_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (peq : Equal OrdResult s ord_eq)
      (ptail : Equal OrdResult (cmpb sndx sndy) ord_eq)
    : And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_eq) =
  and_intro (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_eq) peq ptail

lemma pair_compare_eq_cases_lt_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (plt : Equal OrdResult s ord_lt)
      (peq : Equal OrdResult s ord_eq)
    : And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_eq) =
  absurd (J (λr _. Equal OrdResult r ord_eq) peq plt)

lemma pair_compare_eq_cases_gt_at
      (b : Type)
      (cmpb : b → b → OrdResult)
      (sndx : b)
      (sndy : b)
      (s : OrdResult)
      (pgt : Equal OrdResult s ord_gt)
      (peq : Equal OrdResult s ord_eq)
    : And (Equal OrdResult s ord_eq) (Equal OrdResult (cmpb sndx sndy) ord_eq) =
  absurd (J (λr _. Equal OrdResult r ord_eq) peq pgt)

proof eq_cases for pair_compare
      (a : Type)
      (b : Type)
      (cmpa : a → a → OrdResult)
      (cmpb : b → b → OrdResult)
      (x : Pair a b)
      (y : Pair a b)
      (h : Equal OrdResult (pair_compare a b cmpa cmpb x y) ord_eq)
    : And
        (Equal OrdResult (cmpa (pair_fst a b x) (pair_fst a b y)) ord_eq)
        (Equal OrdResult (cmpb (pair_snd a b x) (pair_snd a b y)) ord_eq) =
  match cmpa (pair_fst a b x) (pair_fst a b y) eqn : ha {
    Lt ↦ pair_compare_eq_cases_lt_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Lt
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_eq)
        h
        ha);
    Eq ↦ pair_compare_eq_cases_eq_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Eq
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_eq)
        h
        ha);
    Gt ↦ pair_compare_eq_cases_gt_at
      b
      cmpb
      (pair_snd a b x)
      (pair_snd a b y)
      Gt
      Proved
      (J
        (λr _.
          Equal
            OrdResult
            (match r {
              Lt ↦ Lt;
              Eq ↦ cmpb (pair_snd a b x) (pair_snd a b y);
              Gt ↦ Gt
            })
            ord_eq)
        h
        ha)
  }

fn list_append (a : Type) (xs : List a) (ys : List a) : List a =
  match xs {
    Nil ↦ ys;
    Cons x xs2 ↦ Cons a x (list_append a xs2 ys)
  }

fn nth (a : Type) (n : Nat) (xs : List a) : Option a =
  match xs {
    Nil ↦ None a;
    Cons h t ↦
      match n {
        Zero ↦ Some a h;
        Suc m ↦ nth a m t
      }
  }

fn take (a : Type) (n : Nat) (xs : List a) : List a =
  match n {
    Zero ↦ Nil a;
    Suc m ↦
      match xs {
        Nil ↦ Nil a;
        Cons h t ↦ Cons a h (take a m t)
      }
  }

fn drop (a : Type) (n : Nat) (xs : List a) : List a =
  match n {
    Zero ↦ xs;
    Suc m ↦
      match xs {
        Nil ↦ Nil a;
        Cons h t ↦ drop a m t
      }
  }
```

## 3. Using it

This package builds up in four layers, each riding the one before: the
floor above; `§4.1`'s structural ops (`map`/`filter`/`mem`/`length`/`min`);
`§4.3`'s verified `List Bool` insertion sort, a caller-facing example of
proving a concrete instantiation of the generic `sort`/`insert` sound and
permutation-preserving; and `§4.6`'s 5 derived `String` ops
(`concat`/`slice`/`char_at`/`eq`/`compare`), which every later catalog
package that manipulates `String` values reaches for directly. Every proof
term in `§4` uses `cong`/`sym`/`trans` from
`catalog/packages/Core/Transport.ken.md`, so a consumer loads Transport
before this file.

## 4. Laws  proofs

### 4.1 CAT-3 D1 — structural list operations

Ordinary transparent recursive definitions over `List`/`Nat`; no primitive
or postulated law is added. `take_drop_decomposition`, `map_length`, and
`length_take_min` are the three proof-returning laws this slice ships. The
`filter` membership characterization is deliberately held out until its
comparator/Iff statement is pinned — no bare `Prop`-returning wrapper is
shipped for it prematurely.

```ken
fn map (a : Type) (b : Type) (f : a → b) (xs : List a) : List b =
  match xs {
    Nil ↦ Nil b;
    Cons h t ↦ Cons b (f h) (map a b f t)
  }

fn filter (a : Type) (p : a → Bool) (xs : List a) : List a =
  match xs {
    Nil ↦ Nil a;
    Cons h t ↦
      match p h {
        True ↦ Cons a h (filter a p t);
        False ↦ filter a p t
      }
  }

fn mem (a : Type) (eqf : a → a → Bool) (x : a) (xs : List a) : Bool =
  match xs {
    Nil ↦ False;
    Cons h t ↦
      match eqf x h {
        True ↦ True;
        False ↦ mem a eqf x t
      }
  }

fn length (a : Type) (xs : List a) : Nat =
  match xs {
    Nil ↦ Zero;
    Cons h t ↦ Suc (length a t)
  }

fn min (m : Nat) (n : Nat) : Nat =
  match m {
    Zero ↦ Zero;
    Suc m2 ↦
      match n {
        Zero ↦ Zero;
        Suc n2 ↦ Suc (min m2 n2)
      }
  }

lemma take_drop_decomposition
      (a : Type) (n : Nat) (xs : List a)
    : Equal (List a) (list_append a (take a n xs) (drop a n xs)) xs =
  match n {
    Zero ↦ Refl;
    Suc m ↦
      match xs {
        Nil ↦ Proved;
        Cons h t ↦ cong
          (List a)
          (List a)
          (list_append a (take a m t) (drop a m t))
          t
          (Cons a h)
          (take_drop_decomposition a m t)
      }
  }

lemma map_length
      (a : Type) (b : Type) (f : a → b) (xs : List a)
    : Equal Nat (length b (map a b f xs)) (length a xs) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦ cong Nat Nat (length b (map a b f t)) (length a t) Suc (map_length a b f t)
  }

lemma length_take_min
      (a : Type) (n : Nat) (xs : List a)
    : Equal Nat (length a (take a n xs)) (min n (length a xs)) =
  match n {
    Zero ↦ Proved;
    Suc m ↦
      match xs {
        Nil ↦ Proved;
        Cons h t ↦ cong
          Nat
          Nat
          (length a (take a m t))
          (min m (length a t))
          Suc
          (length_take_min a m t)
      }
  }
```

### 4.2 DS-4 — five more `List` combinators completing the floor

`reverse`/`zip`/`concat_map`/`range`/`foldl`, each an ordinary
structural-recursion `fn` in the same style as `§4.1` — zero new kernel
feature, zero `trusted_base()` delta. `reverse` is naive, `list_append`-based
(not an accumulator): this spelling makes the involutive proof cleanest,
needing only the standard reverse-of-snoc helper lemma below, not a separate
accumulator-invariant lemma. Both `Nil` branches of `reverse_snoc` close via
`cong _ _ (Cons a y)` over the fully-collapsed `Nil = Nil` — a `Cons`-vs-`Cons`
goal with an ABSTRACT shared element `y` does not itself collapse to bare
`Top` (the kernel's own equality-at-inductive reduction produces a
right-nested Σ pairing the stuck, `y`-abstract element equality with the
collapsed tail equality, so `Proved`/`Refl` alone both fail); lifting `Proved`
through `Cons` via `cong` is the direct, minimal proof. `zip` truncates at
the shorter list (`Nil` on either empty), NOT the length-indexed `Vec` zip:
this is ordinary non-dependent recursion carrying none of the
sibling-convoy/dependent-match capability gate that a length-indexed zip
would need — fully mechanical. `concat_map` ships with only its two
structural (`Nil`/`Cons`) equations — no bespoke length law, since that
would need a `sum` combinator not in this floor (subsume-don't-proliferate).
`range n` produces `[0, 1, .., n-1]` via a `start`-threaded helper
(`range_from`) so the recursion is structural on `n` while the contents
count up. `foldl` similarly ships with only its two structural equations —
no `foldl`/`foldr` relationship law, since `foldr` is not in this floor and
inventing one solely to state a law here would be exactly the proliferation
this package avoids elsewhere.

```ken
fn reverse (a : Type) (xs : List a) : List a =
  match xs {
    Nil ↦ Nil a;
    Cons h t ↦ list_append a (reverse a t) (Cons a h (Nil a))
  }

lemma reverse_snoc
      (a : Type) (xs : List a) (y : a)
    : Equal
        (List a)
        (reverse a (list_append a xs (Cons a y (Nil a))))
        (Cons a y (reverse a xs)) =
  match xs {
    Nil ↦ cong (List a) (List a) (Nil a) (Nil a) (Cons a y) Proved;
    Cons h t ↦ cong
      (List a)
      (List a)
      (reverse a (list_append a t (Cons a y (Nil a))))
      (Cons a y (reverse a t))
      (λw. list_append a w (Cons a h (Nil a)))
      (reverse_snoc a t y)
  }

proof involutive for reverse
      (a : Type) (xs : List a)
    : Equal (List a) (reverse a (reverse a xs)) xs =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦ trans
      (List a)
      (reverse a (reverse a (Cons a h t)))
      (Cons a h (reverse a (reverse a t)))
      (Cons a h t)
      (reverse_snoc a (reverse a t) h)
      (cong
        (List a)
        (List a)
        (reverse a (reverse a t))
        t
        (Cons a h)
        ((proof involutive for reverse) a t))
  }

lemma append_length_snoc
      (a : Type) (xs : List a) (y : a)
    : Equal Nat (length a (list_append a xs (Cons a y (Nil a)))) (Suc (length a xs)) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦ cong
      Nat
      Nat
      (length a (list_append a t (Cons a y (Nil a))))
      (Suc (length a t))
      Suc
      (append_length_snoc a t y)
  }

lemma reverse_length
      (a : Type) (xs : List a)
    : Equal Nat (length a (reverse a xs)) (length a xs) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦ trans
      Nat
      (length a (list_append a (reverse a t) (Cons a h (Nil a))))
      (Suc (length a (reverse a t)))
      (Suc (length a t))
      (append_length_snoc a (reverse a t) h)
      (cong Nat Nat (length a (reverse a t)) (length a t) Suc (reverse_length a t))
  }

fn zip (a : Type) (b : Type) (xs : List a) (ys : List b) : List (Pair a b) =
  match xs {
    Nil ↦ Nil (Pair a b);
    Cons h t ↦
      match ys {
        Nil ↦ Nil (Pair a b);
        Cons h2 t2 ↦ Cons (Pair a b) (mk_pair a b h h2) (zip a b t t2)
      }
  }

lemma zip_length
      (a : Type) (b : Type) (xs : List a) (ys : List b)
    : Equal Nat (length (Pair a b) (zip a b xs ys)) (min (length a xs) (length b ys)) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      match ys {
        Nil ↦ Proved;
        Cons h2 t2 ↦ cong
          Nat
          Nat
          (length (Pair a b) (zip a b t t2))
          (min (length a t) (length b t2))
          Suc
          (zip_length a b t t2)
      }
  }

fn concat_map (a : Type) (b : Type) (f : a → List b) (xs : List a) : List b =
  match xs {
    Nil ↦ Nil b;
    Cons h t ↦ list_append b (f h) (concat_map a b f t)
  }

fn range_from (start : Nat) (n : Nat) : List Nat =
  match n {
    Zero ↦ Nil Nat;
    Suc m ↦ Cons Nat start (range_from (Suc start) m)
  }

fn range (n : Nat) : List Nat = range_from Zero n

lemma range_from_length
      (start : Nat) (n : Nat)
    : Equal Nat (length Nat (range_from start n)) n =
  match n {
    Zero ↦ Proved;
    Suc m ↦ cong
      Nat
      Nat
      (length Nat (range_from (Suc start) m))
      m
      Suc
      (range_from_length (Suc start) m)
  }

lemma range_length (n : Nat) : Equal Nat (length Nat (range n)) n = range_from_length Zero n

fn foldl (a : Type) (b : Type) (f : b → a → b) (z : b) (xs : List a) : b =
  match xs {
    Nil ↦ z;
    Cons h t ↦ foldl a b f (f z h) t
  }
```

### 4.3 CAT-3 D2 — verified `List Bool` insertion sort

`Perm` is intentionally the package-local count/multiset equality surface —
an ordinary `Prop`-valued function over an explicit comparator, never a raw
proof-relevant inductive family — not the older prelude truncation
relation; a consumer that loads this package gets the executable
comparator-indexed form the verified `List Bool` carrier needs. `insert`/
`sort` are the ordinary transparent generic combinators; the proofs below
specialize them to `List Bool` under `bool_leq`, showing the specialized
`sort_bool` (a direct case-split implementation, not `sort` applied to
`bool_leq`) is both order-preserving (`sort_bool_sorted`) and a genuine
permutation of its input (`sort_bool_perm`, via the two count-preservation
lemmas for `True`/`False`).

```ken
fn bool_and (p : Bool) (q : Bool) : Bool =
  match p {
    True ↦ q;
    False ↦ False
  }

fn bool_leq (x : Bool) (y : Bool) : Bool =
  match x {
    False ↦ True;
    True ↦ y
  }

fn eq_from_ord (a : Type) (le : a → a → Bool) (x : a) (y : a) : Bool =
  bool_and (le x y) (le y x)

fn count (a : Type) (eqf : a → a → Bool) (x : a) (xs : List a) : Nat =
  match xs {
    Nil ↦ Zero;
    Cons h t ↦
      match eqf x h {
        True ↦ Suc (count a eqf x t);
        False ↦ count a eqf x t
      }
  }

fn Perm (a : Type) (eqf : a → a → Bool) (xs : List a) (ys : List a) : Prop =
  (x : a) → Equal Nat (count a eqf x xs) (count a eqf x ys)

fn insert (a : Type) (le : a → a → Bool) (x : a) (xs : List a) : List a =
  match xs {
    Nil ↦ Cons a x (Nil a);
    Cons h t ↦
      match le x h {
        True ↦ Cons a x (Cons a h t);
        False ↦ Cons a h (insert a le x t)
      }
  }

fn sort (a : Type) (le : a → a → Bool) (xs : List a) : List a =
  match xs {
    Nil ↦ Nil a;
    Cons h t ↦ insert a le h (sort a le t)
  }

fn bool_head_leq (x : Bool) (xs : List Bool) : Prop =
  match xs {
    Nil ↦ Top;
    Cons h t ↦ Equal Bool (bool_leq x h) True
  }

proof false for bool_head_leq (xs : List Bool) : bool_head_leq False xs =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦ Proved
  }

lemma bool_cons_sorted
      (x : Bool) (xs : List Bool)
    : is_sorted Bool bool_leq xs
      → bool_head_leq x xs
      → is_sorted Bool bool_leq (Cons Bool x xs) =
  match xs {
    Nil ↦ λh. λhb. Proved;
    Cons h t ↦ λhxs.
      λhb.
        and_intro
          (Equal Bool (bool_leq x h) True)
          (is_sorted Bool bool_leq (Cons Bool h t))
          hb
          hxs
  }

proof tail for is_sorted
      (x : Bool) (xs : List Bool)
    : is_sorted Bool bool_leq (Cons Bool x xs) → is_sorted Bool bool_leq xs =
  match xs {
    Nil ↦ λh. Proved;
    Cons h t ↦ λhCons.
      and_snd (Equal Bool (bool_leq x h) True) (is_sorted Bool bool_leq (Cons Bool h t)) hCons
  }

fn insert_true_bool (xs : List Bool) : List Bool =
  match xs {
    Nil ↦ Cons Bool True (Nil Bool);
    Cons h t ↦
      match h {
        True ↦ Cons Bool True (Cons Bool True t);
        False ↦ Cons Bool False (insert_true_bool t)
      }
  }

fn sort_bool (xs : List Bool) : List Bool =
  match xs {
    Nil ↦ Nil Bool;
    Cons h t ↦
      match h {
        False ↦ Cons Bool False (sort_bool t);
        True ↦ insert_true_bool (sort_bool t)
      }
  }

lemma sorted_insert_true_bool
      (xs : List Bool)
    : is_sorted Bool bool_leq xs → is_sorted Bool bool_leq (insert_true_bool xs) =
  match xs {
    Nil ↦ λh. Proved;
    Cons h t ↦
      match h {
        True ↦ λhxs. bool_cons_sorted True (Cons Bool True t) hxs Proved;
        False ↦ λhxs.
          bool_cons_sorted
            False
            (insert_true_bool t)
            (sorted_insert_true_bool t ((proof tail for is_sorted) False t hxs))
            ((proof false for bool_head_leq) (insert_true_bool t))
      }
  }

lemma sort_bool_sorted (xs : List Bool) : is_sorted Bool bool_leq (sort_bool xs) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      match h {
        False ↦ bool_cons_sorted
          False
          (sort_bool t)
          (sort_bool_sorted t)
          ((proof false for bool_head_leq) (sort_bool t));
        True ↦ sorted_insert_true_bool (sort_bool t) (sort_bool_sorted t)
      }
  }

lemma insert_true_bool_count_false
      (xs : List Bool)
    : Equal Nat
        (count Bool (eq_from_ord Bool bool_leq) False (insert_true_bool xs))
        (count Bool (eq_from_ord Bool bool_leq) False xs) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      match h {
        True ↦ Refl;
        False ↦ cong
          Nat
          Nat
          (count Bool (eq_from_ord Bool bool_leq) False (insert_true_bool t))
          (count Bool (eq_from_ord Bool bool_leq) False t)
          Suc
          (insert_true_bool_count_false t)
      }
  }

lemma insert_true_bool_count_true
      (xs : List Bool)
    : Equal Nat
        (count Bool (eq_from_ord Bool bool_leq) True (insert_true_bool xs))
        (Suc (count Bool (eq_from_ord Bool bool_leq) True xs)) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      match h {
        True ↦ Refl;
        False ↦ insert_true_bool_count_true t
      }
  }

lemma sort_bool_count_false
      (xs : List Bool)
    : Equal Nat
        (count Bool (eq_from_ord Bool bool_leq) False xs)
        (count Bool (eq_from_ord Bool bool_leq) False (sort_bool xs)) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      match h {
        False ↦ cong
          Nat
          Nat
          (count Bool (eq_from_ord Bool bool_leq) False t)
          (count Bool (eq_from_ord Bool bool_leq) False (sort_bool t))
          Suc
          (sort_bool_count_false t);
        True ↦ trans
          Nat
          (count Bool (eq_from_ord Bool bool_leq) False t)
          (count Bool (eq_from_ord Bool bool_leq) False (sort_bool t))
          (count Bool (eq_from_ord Bool bool_leq) False (insert_true_bool (sort_bool t)))
          (sort_bool_count_false t)
          (sym
            Nat
            (count Bool (eq_from_ord Bool bool_leq) False (insert_true_bool (sort_bool t)))
            (count Bool (eq_from_ord Bool bool_leq) False (sort_bool t))
            (insert_true_bool_count_false (sort_bool t)))
      }
  }

lemma sort_bool_count_true
      (xs : List Bool)
    : Equal Nat
        (count Bool (eq_from_ord Bool bool_leq) True xs)
        (count Bool (eq_from_ord Bool bool_leq) True (sort_bool xs)) =
  match xs {
    Nil ↦ Proved;
    Cons h t ↦
      match h {
        False ↦ sort_bool_count_true t;
        True ↦ trans
          Nat
          (Suc (count Bool (eq_from_ord Bool bool_leq) True t))
          (Suc (count Bool (eq_from_ord Bool bool_leq) True (sort_bool t)))
          (count Bool (eq_from_ord Bool bool_leq) True (insert_true_bool (sort_bool t)))
          (cong
            Nat
            Nat
            (count Bool (eq_from_ord Bool bool_leq) True t)
            (count Bool (eq_from_ord Bool bool_leq) True (sort_bool t))
            Suc
            (sort_bool_count_true t))
          (sym
            Nat
            (count Bool (eq_from_ord Bool bool_leq) True (insert_true_bool (sort_bool t)))
            (Suc (count Bool (eq_from_ord Bool bool_leq) True (sort_bool t)))
            (insert_true_bool_count_true (sort_bool t)))
      }
  }

lemma sort_bool_perm
      (xs : List Bool)
    : Perm Bool (eq_from_ord Bool bool_leq) xs (sort_bool xs) =
  match xs {
    Nil ↦
      λq.
        match q {
          False ↦ Proved;
          True ↦ Proved
        };
    Cons h t ↦
      λq.
        match q {
          False ↦ sort_bool_count_false (Cons Bool h t);
          True ↦ sort_bool_count_true (Cons Bool h t)
        }
  }
```

### 4.4 CAT-3 D3 — projection-abstraction classes

Ordinary class records over the landed right-nested Σ record machinery.
The concrete lens is intentionally over `Pair Bool Bool` only — a
polymorphic `Lens s a` / `Iso a b` and quotient-carrier views need surface
support that is not part of this slice. Every law field below closes by
`Refl`, since each concrete operation (`fst_pair_bool_bool`,
`set_fst_pair_bool_bool`, the two `Bool`-identity functions) reduces
definitionally once applied.

```ken
class View A {
  project : A → A
}

class Lens A {
  get : Pair Bool Bool → Bool;
  set : Bool → Pair Bool Bool → Pair Bool Bool;
  get_set : (a : Bool) → (s : Pair Bool Bool) → Equal Bool (get (set a s)) a;
  set_get : (s : Pair Bool Bool) → Equal (Pair Bool Bool) (set (get s) s) s;
  set_set :
    (a : Bool)
    → (b : Bool)
    → (s : Pair Bool Bool)
    → Equal (Pair Bool Bool) (set b (set a s)) (set b s)
}

class Iso A {
  to : Bool → Bool;
  from : Bool → Bool;
  to_from : (x : Bool) → Equal Bool (to (from x)) x;
  from_to : (x : Bool) → Equal Bool (from (to x)) x
}

class Representation A {
  encode : Bool → Bool;
  decode : Bool → Bool;
  roundtrip : (x : Bool) → Equal Bool (decode (encode x)) x
}

class RefinementView A {
  project : ({b : Bool | Equal Bool b True}) → Bool
}

class IndexedView A {
  project : Pair Bool Bool → Bool → Bool
}

class SetoidMorphism A {
  project : Bool → Bool;
  respects : (x : Bool) → (y : Bool) → (Equal Bool x y) → Equal Bool (project x) (project y)
}

fn id_bool (x : Bool) : Bool = x

fn fst_pair_bool_bool (p : Pair Bool Bool) : Bool = pair_fst Bool Bool p

fn set_fst_pair_bool_bool (a : Bool) (p : Pair Bool Bool) : Pair Bool Bool =
  mk_pair Bool Bool a (pair_snd Bool Bool p)

lemma fst_lens_get_set
      (a : Bool) (s : Pair Bool Bool)
    : Equal Bool (fst_pair_bool_bool (set_fst_pair_bool_bool a s)) a =
  Refl

lemma fst_lens_set_get
      (s : Pair Bool Bool)
    : Equal (Pair Bool Bool) (set_fst_pair_bool_bool (fst_pair_bool_bool s) s) s =
  Refl

proof set_set for set_fst_pair_bool_bool
      (a : Bool) (b : Bool) (s : Pair Bool Bool)
    : Equal
        (Pair Bool Bool)
        (set_fst_pair_bool_bool b (set_fst_pair_bool_bool a s))
        (set_fst_pair_bool_bool b s) =
  Refl

instance View Bool {
  project = id_bool
}

instance Lens Unit {
  get = fst_pair_bool_bool;
  set = set_fst_pair_bool_bool;
  get_set = fst_lens_get_set;
  set_get = fst_lens_set_get;
  set_set = proof set_set for set_fst_pair_bool_bool
}

fn bool_iso_to (x : Bool) : Bool = x

fn bool_iso_from (x : Bool) : Bool = x

lemma bool_iso_to_from (x : Bool) : Equal Bool (bool_iso_to (bool_iso_from x)) x = Refl

lemma bool_iso_from_to (x : Bool) : Equal Bool (bool_iso_from (bool_iso_to x)) x = Refl

instance Iso Unit {
  to = bool_iso_to;
  from = bool_iso_from;
  to_from = bool_iso_to_from;
  from_to = bool_iso_from_to
}

instance Representation Unit {
  encode = bool_iso_to;
  decode = bool_iso_from;
  roundtrip = bool_iso_from_to
}

fn true_refinement_project (x : {b : Bool | Equal Bool b True}) : Bool = x

instance RefinementView Unit {
  project = true_refinement_project
}

fn bool_pair_index_project (p : Pair Bool Bool) (ix : Bool) : Bool =
  match ix {
    False ↦ pair_fst Bool Bool p;
    True ↦ pair_snd Bool Bool p
  }

instance IndexedView Unit {
  project = bool_pair_index_project
}

proof respects for id_bool
      (x : Bool) (y : Bool)
    : Equal Bool x y → Equal Bool (id_bool x) (id_bool y) =
  λp. p

instance SetoidMorphism Unit {
  project = id_bool;
  respects = proof respects for id_bool
}
```

### 4.5 The remaining floor combinators

`nat_sub` is saturating `Nat` monus (never underflows) — `§4.6`'s `slice`
needs exactly this shape for its length computation, identical to the
landed `val1_string_literals.rs:327` `nat_sub` precedent. `list_eq` and
`list_compare` complete the seven-combinator floor from `§1`. `compare_char`
is a faithful 3-way repackaging of the landed `leqChar`/`eqChar`
(`crates/ken-elaborator/src/decimal_char.rs`, Rust-side primitives — not
catalog declarations, so their own names are untouched by this catalog's
casing convention), not a re-derivation of `Char` comparison (settled input
#4, `docs/program/wp/L3-strings-surface.md §2`): `eqChar` decides equality
directly; otherwise `Lt`/`Gt` follow from `leqChar`'s antisymmetry and
totality (both landed `Ord Char` laws, by transport from `Ord Int`).

```ken
fn nat_sub (a : Nat) (b : Nat) : Nat =
  match b {
    Zero ↦ a;
    Suc n ↦
      match a {
        Zero ↦ Zero;
        Suc m ↦ nat_sub m n
      }
  }

fn list_eq (a : Type) (eqf : a → a → Bool) (xs : List a) (ys : List a) : Bool =
  match xs {
    Nil ↦
      match ys {
        Nil ↦ True;
        Cons h t ↦ False
      };
    Cons x xs2 ↦
      match ys {
        Nil ↦ False;
        Cons y ys2 ↦
          match eqf x y {
            True ↦ list_eq a eqf xs2 ys2;
            False ↦ False
          }
      }
  }

fn list_compare (a : Type) (cmp : a → a → OrdResult) (xs : List a) (ys : List a) : OrdResult =
  match xs {
    Nil ↦
      match ys {
        Nil ↦ Eq;
        Cons h t ↦ Lt
      };
    Cons x xs2 ↦
      match ys {
        Nil ↦ Gt;
        Cons y ys2 ↦
          match cmp x y {
            Eq ↦ list_compare a cmp xs2 ys2;
            Lt ↦ Lt;
            Gt ↦ Gt
          }
      }
  }

fn compare_char (a : Char) (b : Char) : OrdResult =
  match eqChar a b {
    True ↦ Eq;
    False ↦
      match leqChar a b {
        True ↦ Lt;
        False ↦ Gt
      }
  }
```

### 4.6 The 5 derived `String` ops

Routed through the real `string_to_list_char`/`list_char_to_string` round
trip (slice 1). These ship as plain functions — `eq`/`compare` are
tested-not-trusted Boolean/decision ops, not lawful `DecEq String`/
`Ord String` instances (that transport needs a lawful `DecEq Char`, not yet
landed — a tracked follow-on; filing these as proof-carrying instances now
would over-claim the trust level). `slice` clamps by construction: `drop`
past the end yields `Nil`, `take` past the end stops at the end, and the
length `nat_sub j i` saturates at `0` when `j < i` — never an underflow,
never stuck. `char_at` is total and honest about absence — `Option Char`,
never a sentinel or a partial index. `eq` is codepoint-wise equality over
the scalar sequence, riding the landed `eqChar` — this is never
NFC-normalization equality (ADR 0010 §3: that identifies distinct scalar
sequences, so over the codepoint carrier it is non-canonical — a lawful
`DecEq` for it would inhabit `Bottom`). `compare` is 3-way, codepoint-wise
lexicographic order — the more fundamental op, subsuming `<=`/`<`/`==`
(a `leq`-only interface cannot cheaply recover a 3-way result).

```ken
fn concat (a : String) (b : String) : String =
  list_char_to_string (list_append Char (string_to_list_char a) (string_to_list_char b))

fn slice (i : Nat) (j : Nat) (s : String) : String =
  list_char_to_string (take Char (nat_sub j i) (drop Char i (string_to_list_char s)))

fn char_at (i : Nat) (s : String) : Option Char = nth Char i (string_to_list_char s)

fn eq (a : String) (b : String) : Bool =
  list_eq Char eqChar (string_to_list_char a) (string_to_list_char b)

fn compare (a : String) (b : String) : OrdResult =
  list_compare Char compare_char (string_to_list_char a) (string_to_list_char b)
```

## 5. Design notes

**Package dependency.** The CAT-3 proof terms in `§4.1`–`§4.3` use `cong`/
`sym`/`trans`, so harnesses and consumers load
`catalog/packages/Core/Transport.ken.md` before this file. The dependency is
proof-only and adds no trusted-base delta.

**SCT sound zone.** Every recursive call in this package is an applied call
whose decreasing argument is a strict subterm of a matched argument (the
`Cons` tail and/or the `Suc` predecessor) — squarely in the termination
checker's sound zone, never leaning on its unapplied-self-reference /
recursion-through-opaque-map over-accept hole.

**Deliverability honesty.** `String` is canonical with respect to `List Char`
(the `string_to_list_char`/`list_char_to_string` round trip is a bijection
on scalar sequences, ADR 0010 §2), so `DecEq String`/`Ord String` instances
are soundly deliverable later — but that transport additionally needs a
lawful `DecEq Char`, which is now landed in `Core/LawfulClasses`. Filing
`eq`/`compare` as proof-carrying instances here would still over-claim the
trust level; this package ships the functions only, honestly.

## 6. Findings

- **Kernel-reduction defect:** none.
- **Abstraction candidate:** `§4.1`'s `filter` membership characterization
  is deliberately held out until its comparator/Iff statement is pinned —
  not shipped as a premature wrapper.
- **Runtime-performance characteristic (non-blocking, forward-tracked).**
  `crates/ken-elaborator/tests/l3_strings_surface_acceptance.rs`'s
  `derived_string_ops_reduce_over_real_roundtrip` test exercises the pinned
  `slice 0 99 "abc"` equivalent-to-`"abc"` conformance case
  (`conformance/surface/collections/seed-collections.md` DS-AC3). Evaluating
  a `take`/`drop`-style structural recursion at a unary-`Nat` depth of ~99
  costs noticeably more than linear time in the current `ken-interp`
  evaluator (empirically ~O(n^3.5–4) in the recursion depth `n`, not
  exponential — a correct value, just slow: this one test takes on the
  order of a few CPU-minutes at `n = 99`, versus sub-millisecond at
  `n <= 40`). This is a pre-existing characteristic of `ken-interp`'s
  reduction strategy for deep unary-`Nat` recursion under nested `match` (no
  prior test exercised `Nat` depths anywhere near this range), **not** a bug
  introduced by this package's derived definitions (the combinators are
  correct and match the spec's mandated shapes exactly), and **not** a
  soundness concern (the interpreter is the tested-not-trusted ring — a
  wrong value, never a false proof, and the value here is correct). Flagged
  to the language-leader/Architect as a forward-tracked `ken-interp`
  performance finding; not a blocker for this package.

## 7. References

None — this entry's design is Ken-native, not consulted from an external
reference implementation.

## 8. Trust  derivation

1. **Spec / WP.** `spec/30-surface/37-strings-collections.md §2.4/§2.5/
   §2.5.1/§4.1`; WP `L3-strings-surface` (this package, slice 2/2);
   `L3-strings-roundtrip` (slice 1, the native round trip this rides).
2. **Public API.** `OrdResult`, `list_append`, `nth`, `take`, `drop`,
   `nat_sub`, `list_eq`, `list_compare` (the 7-combinator floor); `map`,
   `filter`, `mem`, `length`, `min`, `take_drop_decomposition`,
   `map_length`, `length_take_min` (CAT-3 D1); `reverse`, `reverse::involutive`,
   `zip`, `concat_map`, `range`, `foldl` and their proofs (DS-4); `count`,
   `Perm`, `insert`, `sort`, `sort_bool`, `sort_bool_sorted`,
   `sort_bool_perm` (CAT-3 D2); `View`, `Lens`, `Iso`, `Representation`,
   `RefinementView`, `IndexedView`, `SetoidMorphism` (CAT-3 D3);
   `compare_char`, `concat`, `slice`, `char_at`, `eq`, `compare` (the 5
   derived `String` ops).
3. **Source map.**

   | Task | Section |
   |---|---|
   | See the floor's first four combinators | [Definition](#2-definition) |
   | See how the layers build on each other | [Using it](#3-using-it) |
   | Structural laws, verified sort, projection classes, the string ops | [Laws  proofs](#4-laws--proofs) |
   | Package dependency, SCT sound zone, deliverability honesty | [Design notes](#5-design-notes) |

4. **Derivation path.** Every combinator, law, and string op is a
   `declare_def` (checked, upgraded opaque to transparent on `sct_check`
   success) or an ordinary `fn`; `OrdResult` is a checked `data` inductive
   (kernel-admitted by positivity), never a primitive or postulated
   declaration. No native interpreter primitive is added for any list
   combinator/law or string op (Approach A, Architect ruling
   `evt_4k1yqah3yvpds`) — deriving trivially structural folds keeps the
   audited primitive set small (subsume-don't-proliferate).
5. **`trusted_base()` delta.** **Zero.** Every proof in this package is a
   genuine, kernel-checked term; no law field is postulated anywhere.
6. **Proof families.** `§4.1`/`§4.2`: structural induction + `cong`/`trans`
   lifting the tail IH under the head constructor, the same shape
   throughout. `§4.3`: full case-split specialized to `List Bool`/`bool_leq`,
   closing by `Proved`/`Refl`/`cong`/`trans`/`sym` per branch — no postulate
   anywhere in the verified-sort slice. `§4.4`: every law field closes by
   `Refl` (each concrete operation reduces definitionally once applied, no
   case-split needed).
7. **Consumers.** `catalog/packages/Data/Collections/Map.ken` (the proved
   `Map`/`Set` BST) depends on this package's `list_append`.
   `crates/ken-elaborator/tests/cat1_lawful_functors_package.rs`,
   `ds3_sum_combinators_acceptance.rs`, `ds4_list_combinators_acceptance.rs`,
   `ds7_applicative_monad_acceptance.rs`, `ds8_traversable_acceptance.rs`,
   `either_catalog_package_acceptance.rs`, `es2_acceptance.rs`,
   `l3_strings_surface_acceptance.rs`, `map_build_acceptance.rs`, and
   `cat3_collections_package.rs` all load this package as a cross-file
   prerequisite for their own consuming packages; `crates/ken-cli/tests/rosetta.rs`
   concatenates it (after `Transport.ken.md`'s tangled source) ahead of
   several rosetta examples that reuse it per the DRY rule.
8. **Validation evidence.**
   `crates/ken-elaborator/tests/cat3_collections_package.rs` — confirms the
   CAT-3 D1/D2/D3 surface elaborates with zero `trusted_base()` delta, that
   every law is proof-returning (not a bare `Prop` wrapper) and postulates
   nothing, and that `§4.4`'s classes stay capitalized (no stray lowercase
   `View`-style declaration reintroduced).
   `crates/ken-elaborator/tests/ds4_list_combinators_acceptance.rs` —
   confirms the DS-4 combinators register as real globals and the file
   postulates nothing. `crates/ken-elaborator/tests/l3_strings_surface_acceptance.rs` —
   confirms the 5 derived `String` ops reduce over the real round trip.
