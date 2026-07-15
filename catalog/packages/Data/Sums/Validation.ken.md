# `Validation` — accumulating independent errors

`Validation e a` is an error-or-value sum whose lawful `Applicative` instance
combines independent errors through a supplied `Semigroup e`.

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

`Result e a` is appropriate when later work depends on an earlier success: its
monadic bind stops at the first error. Validation of independent fields has a
different need. Every field can be checked, so callers benefit from receiving
all discovered errors in one pass.

`Validation e a` provides that second interpretation. It has the same broad
error-or-value shape, but its applicative application combines two `Invalid`
values with a lawful `Semigroup e` operation.

## 2. Definition

The carrier is an ordinary binary sum. Mapping preserves errors and transforms
only successful values. `validation_ap` accumulates errors from both sides in
left-to-right order.

```ken
data Validation e a = Invalid e | Valid a

fn validation_map
      (e : Type) (a : Type) (b : Type) (g : a → b) (x : Validation e a)
    : Validation e b =
  match x {
    Invalid err ↦ Invalid e b err;
    Valid value ↦ Valid e b (g value)
  }

fn validation_pure (e : Type) (a : Type) (x : a) : Validation e a = Valid e a x

fn validation_ap
      (e : Type)
      (sg : Semigroup e)
      (a : Type)
      (b : Type)
      (vf : Validation e (a → b))
      (vx : Validation e a)
    : Validation e b =
  match vf {
    Invalid left_error ↦
      match vx {
        Invalid right_error ↦ Invalid e b (sg.op left_error right_error);
        Valid value ↦ Invalid e b left_error
      };
    Valid f ↦
      match vx {
        Invalid right_error ↦ Invalid e b right_error;
        Valid value ↦ Valid e b (f value)
      }
  }
```

## 3. Using it

The example performs two independent checks. Both fail, and applicative
combination returns a single `Invalid` value containing both messages in a
`NonEmpty String`. A first-error `Result` computation would expose only the
first message.

```ken example
const name_failure : NonEmpty String =
  NonEmptyCons
    String
    (match bytes_decode (bytes_encode "name is missing") {
      Err _ ↦ "name is missing";
      Ok text ↦ text
    })
    (Nil String)

const age_failure : NonEmpty String =
  NonEmptyCons
    String
    (match bytes_decode (bytes_encode "age is invalid") {
      Err _ ↦ "age is invalid";
      Ok text ↦ text
    })
    (Nil String)

fn check_name (present : Bool) : Validation (NonEmpty String) Bool =
  match present {
    True ↦ Valid (NonEmpty String) Bool True;
    False ↦ Invalid (NonEmpty String) Bool name_failure
  }

fn check_age (adult : Bool) : Validation (NonEmpty String) Bool =
  match adult {
    True ↦ Valid (NonEmpty String) Bool True;
    False ↦ Invalid (NonEmpty String) Bool age_failure
  }

const checked_name : Validation (NonEmpty String) Bool = check_name False

const checked_age : Validation (NonEmpty String) Bool = check_age False

const checked_record : Validation (NonEmpty String) (Pair Bool Bool) =
  validation_ap
    (NonEmpty String)
    (Semigroup_instance_NonEmpty String)
    Bool
    (Pair Bool Bool)
    (validation_map
      (NonEmpty String)
      Bool
      (Bool → Pair Bool Bool)
      (mk_pair Bool Bool)
      checked_name)
    checked_age

const expected_errors : Validation (NonEmpty String) (Pair Bool Bool) =
  Invalid (NonEmpty String) (Pair Bool Bool) (nonempty_append String name_failure age_failure)

lemma same_invalid_example
      (errors : NonEmpty String)
    : Equal
        (Validation (NonEmpty String) (Pair Bool Bool))
        (Invalid (NonEmpty String) (Pair Bool Bool) errors)
        (Invalid (NonEmpty String) (Pair Bool Bool) errors) =
  cong
    (NonEmpty String)
    (Validation (NonEmpty String) (Pair Bool Bool))
    errors
    errors
    (Invalid (NonEmpty String) (Pair Bool Bool))
    Refl

lemma both_errors_accumulate
    : Equal (Validation (NonEmpty String) (Pair Bool Bool)) checked_record expected_errors =
  same_invalid_example (nonempty_append String name_failure age_failure)
```

## 4. Laws & proofs

### 4.1 Functor

Both functor laws are structural. An `Invalid` payload is preserved, and a
`Valid` payload reduces to the corresponding ordinary function equation.

```ken
proof id for validation_map
      (e : Type) (a : Type) (x : Validation e a)
    : Equal (Validation e a) (validation_map e a a (idf a) x) x =
  match x {
    Invalid err ↦ Refl;
    Valid value ↦ Refl
  }

proof fusion for validation_map
      (e : Type) (a : Type) (b : Type) (c : Type) (g : b → c) (h : a → b) (x : Validation e a)
    : Equal
        (Validation e c)
        (validation_map e a c (comp a b c g h) x)
        (validation_map e b c g (validation_map e a b h x)) =
  match x {
    Invalid err ↦ Refl;
    Valid value ↦ Refl
  }

instance Functor (Validation e) {
  map = validation_map e;
  id_law = validation_map::id e;
  fusion_law = validation_map::fusion e
}
```

### 4.2 Applicative

Identity, homomorphism, and interchange compute in each constructor case.
Composition has eight constructor combinations. Seven compute directly; when
all three inputs are invalid, the two sides contain
`(e1 <> e2) <> e3` and `e1 <> (e2 <> e3)`. The supplied semigroup law is
lifted beneath `Invalid` to close exactly that branch.

```ken
lemma validation_ap_id
      (e : Type) (sg : Semigroup e) (a : Type) (v : Validation e a)
    : Equal (Validation e a) (validation_ap e sg a a (validation_pure e (a → a) (idf a)) v) v =
  match v {
    Invalid err ↦ Refl;
    Valid value ↦ Refl
  }

lemma validation_ap_hom
      (e : Type) (sg : Semigroup e) (a : Type) (b : Type) (g : a → b) (x : a)
    : Equal
        (Validation e b)
        (validation_ap e sg a b (validation_pure e (a → b) g) (validation_pure e a x))
        (validation_pure e b (g x)) =
  Refl

lemma validation_ap_ich
      (e : Type) (sg : Semigroup e) (a : Type) (b : Type) (u : Validation e (a → b)) (y : a)
    : Equal
        (Validation e b)
        (validation_ap e sg a b u (validation_pure e a y))
        (validation_ap e sg (a → b) b (validation_pure e ((a → b) → b) (apply_to a b y)) u) =
  match u {
    Invalid err ↦ Refl;
    Valid g ↦ Refl
  }

lemma validation_ap_cmp
      (e : Type)
      (sg : Semigroup e)
      (a : Type)
      (b : Type)
      (c : Type)
      (u : Validation e (b → c))
      (v : Validation e (a → b))
      (w : Validation e a)
    : Equal
        (Validation e c)
        (validation_ap
          e
          sg
          a
          c
          (validation_ap
            e
            sg
            (a → b)
            (a → c)
            (validation_ap
              e
              sg
              (b → c)
              ((a → b) → (a → c))
              (validation_pure e ((b → c) → (a → b) → (a → c)) (compose a b c))
              u)
            v)
          w)
        (validation_ap e sg b c u (validation_ap e sg a b v w)) =
  match u {
    Invalid first ↦
      match v {
        Invalid second ↦
          match w {
            Invalid third ↦
              cong
                e
                (Validation e c)
                (sg.op (sg.op first second) third)
                (sg.op first (sg.op second third))
                (Invalid e c)
                (sg.assoc first second third);
            Valid value ↦ Refl
          };
        Valid g ↦
          match w {
            Invalid third ↦ Refl;
            Valid value ↦ Refl
          }
      };
    Valid f ↦
      match v {
        Invalid second ↦
          match w {
            Invalid third ↦ Refl;
            Valid value ↦ Refl
          };
        Valid g ↦
          match w {
            Invalid third ↦ Refl;
            Valid value ↦ Refl
          }
      }
  }

lemma validation_map_coh
      (e : Type) (sg : Semigroup e) (a : Type) (b : Type) (g : a → b) (x : Validation e a)
    : Equal
        (Validation e b)
        (functor_map_of (Validation e) (Functor_instance_Validation e) a b g x)
        (validation_ap e sg a b (validation_pure e (a → b) g) x) =
  match x {
    Invalid err ↦ Refl;
    Valid value ↦ Refl
  }

instance Applicative (Validation e) where Semigroup e {
  functor = Functor_instance_Validation e;
  pure = validation_pure e;
  ap = validation_ap e d;
  ap_id = validation_ap_id e d;
  ap_hom = validation_ap_hom e d;
  ap_ich = validation_ap_ich e d;
  ap_cmp = validation_ap_cmp e d;
  map_coh = validation_map_coh e d
}
```

## 5. Design notes

**Why there is no `Monad (Validation e)`.** Monad bind chooses the next
computation from one successful value. After an error there is no value with
which to choose that next computation, so bind must stop and keep only the
error already encountered. That is the first-error behavior of `Result`, not
independent error accumulation. This package therefore provides `Functor` and
`Applicative` deliberately, with no `Monad` instance.

**The semigroup is explicit in application.** Accumulation needs only an
associative operation, not an identity element. Keeping the requirement at
`Semigroup e` permits a structurally non-empty error collection and prevents an
empty-error `Invalid` value from being required.

## 6. References

- **Applicative functor** — Wikipedia,
  <https://en.wikipedia.org/wiki/Applicative_functor> — orientation on the
  identity, homomorphism, interchange, and composition laws.
- **Semigroup** — Wikipedia, <https://en.wikipedia.org/wiki/Semigroup> —
  orientation on the associative operation used to combine errors.

## 7. Trust & derivation

**Public API (stable names):** `Validation`/`Invalid`/`Valid`,
`validation_map`, `validation_pure`, `validation_ap`,
`Functor_instance_Validation`, and `Applicative_instance_Validation`.

**Source map:**

| Reader task | Section |
|---|---|
| Understand accumulation versus short-circuiting | [§1](#1-motivation), [§5](#5-design-notes) |
| Apply independent validations | [§2](#2-definition), [§3](#3-using-it) |
| Inspect every class law | [§4](#4-laws--proofs) |

**Derivation path from built-ins.** `Validation` is an ordinary strictly
positive inductive. Mapping, pure, and application are structural functions.
The functor laws compute by constructor reduction; the applicative composition
law consumes only the supplied semigroup associativity proof.

**`trusted_base()` delta: zero.** The package declares no primitive, opaque
constant, postulate, or `Axiom`; every class field is inhabited by an ordinary
kernel-checked term.

**Proof families.** Functor laws split on one validation. Applicative identity
and interchange split on one validation; composition splits on three and uses
`Semigroup.assoc` only in the all-invalid branch.

**Consumers.** Later decoding and command-line packages can specialize the
generic error type to a non-empty collection of their own diagnostics.

**Validation evidence.** `ken check` elaborates the tangled definition and the
two-failure checked example; the strict formatter gate checks this new catalog
source.
