# `NonEmpty` — lists with a structural head

`NonEmpty a` stores one `a` separately from an ordinary `List a`, so an empty
value cannot be constructed. Its append operation forms a lawful semigroup.

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

An ordinary `List a` may be empty. Some APIs instead need a sequence that is
known to contain at least one element. `NonEmpty a` makes that guarantee in its
constructor: every value has a head, followed by a possibly empty ordinary
list tail. No Boolean check or refinement proof is needed at use sites.

## 2. Definition

The carrier has exactly one constructor. The five operations expose its head
and tail, forget the structural guarantee, map every element, or append two
non-empty sequences.

The `nonempty_` prefix keeps these operations distinct from the ordinary-list
combinators in `Data.Collections` while preserving the same familiar names.

```ken
data NonEmpty a = NonEmptyCons a (List a)

fn nonempty_head (a : Type) (xs : NonEmpty a) : a =
  match xs {
    NonEmptyCons x rest ↦ x
  }

fn nonempty_tail (a : Type) (xs : NonEmpty a) : List a =
  match xs {
    NonEmptyCons x rest ↦ rest
  }

fn nonempty_to_list (a : Type) (xs : NonEmpty a) : List a =
  match xs {
    NonEmptyCons x rest ↦ Cons a x rest
  }

fn nonempty_map (a : Type) (b : Type) (f : a → b) (xs : NonEmpty a) : NonEmpty b =
  match xs {
    NonEmptyCons x rest ↦ NonEmptyCons b (f x) (map a b f rest)
  }

fn nonempty_append (a : Type) (xs : NonEmpty a) (ys : NonEmpty a) : NonEmpty a =
  match xs {
    NonEmptyCons x rest ↦
      match ys {
        NonEmptyCons y more ↦ NonEmptyCons a x (list_append a rest (Cons a y more))
      }
  }
```

## 3. Using it

Construction itself supplies the non-empty guarantee. Forgetting that
guarantee with `nonempty_to_list` always produces a `Cons`.

```ken example
const one_two : NonEmpty Nat = NonEmptyCons Nat (Suc Zero) (Cons Nat (Suc (Suc Zero)) (Nil Nat))

const first_of_one_two : Nat = nonempty_head Nat one_two

const ordinary_one_two : List Nat = nonempty_to_list Nat one_two
```

## 4. Laws & proofs

Associativity follows from ordinary list append. After exposing the three
heads, `list_append::assoc` proves the equality of the stored tails; `cong`
lifts that equality beneath the shared `NonEmptyCons` head.

```ken
proof assoc for nonempty_append
      (a : Type) (xs : NonEmpty a) (ys : NonEmpty a) (zs : NonEmpty a)
    : Equal
        (NonEmpty a)
        (nonempty_append a (nonempty_append a xs ys) zs)
        (nonempty_append a xs (nonempty_append a ys zs)) =
  match xs {
    NonEmptyCons x rest ↦
      match ys {
        NonEmptyCons y more ↦
          match zs {
            NonEmptyCons z last ↦ cong
              (List a)
              (NonEmpty a)
              (list_append a (list_append a rest (Cons a y more)) (Cons a z last))
              (list_append a rest (Cons a y (list_append a more (Cons a z last))))
              (NonEmptyCons a x)
              (list_append::assoc a rest (Cons a y more) (Cons a z last))
          }
      }
  }

instance Semigroup (NonEmpty a) {
  op = nonempty_append a;
  assoc = nonempty_append::assoc a
}
```

## 5. Design notes

**The head is structural.** Representing this carrier as a refinement of
`List` would require every consumer to transport a non-emptiness proof through
list operations. The single constructor makes the useful invariant visible to
the coverage checker and keeps `head` total by construction.

**Append keeps the left head.** The left value's head remains the result head;
its tail is followed by the complete right value. This is the usual
concatenation order, and it makes associativity reduce directly to ordinary
list-append associativity.

## 6. References

- **Non-empty list** — Wikipedia,
  <https://en.wikipedia.org/wiki/Non-empty_list> — orientation on the
  head-plus-tail representation and its total head operation.
- **Semigroup** — Wikipedia, <https://en.wikipedia.org/wiki/Semigroup> —
  orientation on the single associativity law carried by the instance.

## 7. Trust & derivation

**Public API (stable names):** `NonEmpty`/`NonEmptyCons`, `nonempty_head`,
`nonempty_tail`, `nonempty_to_list`, `nonempty_map`, `nonempty_append`, and
`Semigroup_instance_NonEmpty`.

**Source map:**

| Reader task | Section |
|---|---|
| Understand the structural guarantee | [§1](#1-motivation), [§2](#2-definition) |
| Construct or consume a value | [§3](#3-using-it) |
| Inspect the semigroup proof | [§4](#4-laws--proofs) |

**Derivation path from built-ins.** `NonEmpty` is an ordinary strictly
positive inductive. Its operations use structural matches, ordinary `List`,
and `list_append`; the semigroup law is lifted from the kernel-checked list
append associativity proof.

**`trusted_base()` delta: zero.** The package declares no primitive, opaque
constant, postulate, or `Axiom`; its only class law is inhabited by a checked
proof term.

**Proof families.** The one law is a three-value structural case split followed
by congruence over `list_append::assoc`.

**Consumers.** `Validation` uses `NonEmpty e` as the canonical independently
accumulating error carrier.

**Validation evidence.** `ken check` elaborates the tangled definition and its
checked example; the strict formatter gate checks this new catalog source.
