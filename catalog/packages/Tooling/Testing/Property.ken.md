# `Tooling.Testing.Property` — deterministic finite property checks

`Tooling.Testing.Property` exercises decidable predicates over explicit finite samples and
reports the first counterexample.

## Index

- [Motivation](#motivation)
- [Definition](#definition)
- [Using it](#using-it)
- [Laws & proofs](#laws--proofs)
- [Design notes](#design-notes)
- [References](#references)
- [Trust & derivation](#trust--derivation)

## Reading paths

- **Newcomer:** Motivation → Using it
- **Practitioner:** Definition → Using it
- **Researcher:** Laws & proofs → Trust & derivation
- **Porting a property suite:** Design notes → Definition

## Motivation

A small property runner is useful even without randomness. An explicit finite
sample list makes every run deterministic, bounded, and reproducible. The
runner evaluates a `Bool` predicate and returns either success or the first
sample that falsified it.

Properties here are computations, not propositions. They test the executable
shadow of a contract without assuming or proving that contract.

## Definition

`Gen` exposes only construction from a sample list and mapping. Runner outcomes
use the ordinary error-biased `Result`: an error is the first counterexample,
while success carries `Unit`.

```ken
data Gen a = MkGen (List a)

fn gen_from_list (a : Type) (samples : List a) : Gen a = MkGen a samples

fn gen_samples (a : Type) (generator : Gen a) : List a =
  match generator {
    MkGen samples ↦ samples
  }

fn gen_map_list (a : Type) (b : Type) (f : a → b) (samples : List a) : List b =
  match samples {
    Nil ↦ Nil b;
    Cons sample rest ↦ Cons b (f sample) (gen_map_list a b f rest)
  }

fn gen_map (a : Type) (b : Type) (f : a → b) (generator : Gen a) : Gen b =
  gen_from_list b (gen_map_list a b f (gen_samples a generator))

fn check_samples (a : Type) (samples : List a) (predicate : a → Bool) : Result a Unit =
  match samples {
    Nil ↦ Ok a Unit MkUnit;
    Cons sample rest ↦
      match predicate sample {
        True ↦ check_samples a rest predicate;
        False ↦ Err a Unit sample
      }
  }

fn check (a : Type) (generator : Gen a) (predicate : a → Bool) : Result a Unit =
  check_samples a (gen_samples a generator) predicate
```

The byte generator starts from the structural `List UInt8` view and maps the
landed total conversion into `Bytes`. Its five samples include the empty input,
both boundary singletons, and two multi-byte inputs.

```ken
const byte_sample_lists : List (List UInt8) =
  Cons
    (List UInt8)
    (Nil UInt8)
    (Cons
      (List UInt8)
      (Cons UInt8 0 (Nil UInt8))
      (Cons
        (List UInt8)
        (Cons UInt8 255 (Nil UInt8))
        (Cons
          (List UInt8)
          (Cons UInt8 0 (Cons UInt8 255 (Nil UInt8)))
          (Cons
            (List UInt8)
            (Cons UInt8 1 (Cons UInt8 127 (Cons UInt8 255 (Nil UInt8))))
            (Nil (List UInt8))))))

const gen_byte_lists : Gen (List UInt8) = gen_from_list (List UInt8) byte_sample_lists

const gen_bytes : Gen Bytes = gen_map (List UInt8) Bytes list_to_bytes gen_byte_lists
```

The cursor slice contains only the operations used by the progress check. It
views a `Bytes` value structurally, peeks at the head, advances by dropping one
element, and counts the remaining elements.

```ken
data ByteCursor = MkByteCursor (List UInt8)

fn byte_cursor_start (input : Bytes) : ByteCursor = MkByteCursor (bytes_to_list input)

fn property_list_length (a : Type) (values : List a) : Nat =
  match values {
    Nil ↦ Zero;
    Cons value rest ↦ Suc (property_list_length a rest)
  }

fn byte_cursor_remaining (cursor : ByteCursor) : Nat =
  match cursor {
    MkByteCursor bytes ↦ property_list_length UInt8 bytes
  }

fn byte_cursor_peek (cursor : ByteCursor) : Option UInt8 =
  match cursor {
    MkByteCursor bytes ↦
      match bytes {
        Nil ↦ None UInt8;
        Cons byte rest ↦ Some UInt8 byte
      }
  }

fn byte_cursor_advance (cursor : ByteCursor) : ByteCursor =
  match cursor {
    MkByteCursor bytes ↦
      match bytes {
        Nil ↦ MkByteCursor (Nil UInt8);
        Cons byte rest ↦ MkByteCursor rest
      }
  }

fn byte_cursor_stuck_advance (cursor : ByteCursor) : ByteCursor = cursor

fn property_nat_lt (left : Nat) (right : Nat) : Bool =
  match right {
    Zero ↦ False;
    Suc right2 ↦
      match left {
        Zero ↦ True;
        Suc left2 ↦ property_nat_lt left2 right2
      }
  }

fn cursor_progress_with (advance : ByteCursor → ByteCursor) (input : Bytes) : Bool =
  let cursor : ByteCursor =
    byte_cursor_start input
  in
    match byte_cursor_peek cursor {
      None ↦ True;
      Some byte ↦
        property_nat_lt (byte_cursor_remaining (advance cursor)) (byte_cursor_remaining cursor)
    }

fn cursor_progress (input : Bytes) : Bool = cursor_progress_with byte_cursor_advance input

fn cursor_stuck_progress (input : Bytes) : Bool =
  cursor_progress_with byte_cursor_stuck_advance input
```

## Using it

`check` stops at the first false predicate. These helpers inspect its ordinary
`Result` value without turning a test outcome into a proof.

```ken
fn property_result_is_held (a : Type) (outcome : Result a Unit) : Bool =
  match outcome {
    Err counterexample ↦ False;
    Ok unit ↦ True
  }

fn property_uint8_eq (left : UInt8) (right : UInt8) : Bool =
  eq_int (uint8_to_int left) (uint8_to_int right)

fn property_list_uint8_eq (left : List UInt8) (right : List UInt8) : Bool =
  match left {
    Nil ↦
      match right {
        Nil ↦ True;
        Cons head tail ↦ False
      };
    Cons left_head left_tail ↦
      match right {
        Nil ↦ False;
        Cons right_head right_tail ↦
          match property_uint8_eq left_head right_head {
            True ↦ property_list_uint8_eq left_tail right_tail;
            False ↦ False
          }
      }
  }

fn property_bytes_eq (left : Bytes) (right : Bytes) : Bool =
  property_list_uint8_eq (bytes_to_list left) (bytes_to_list right)

fn property_result_failed_with
      (a : Type) (eq : a → a → Bool) (expected : a) (outcome : Result a Unit)
    : Bool =
  match outcome {
    Err counterexample ↦ eq counterexample expected;
    Ok unit ↦ False
  }

fn reject_every_byte_sample (input : Bytes) : Bool = False

const empty_byte_sample : Bytes = list_to_bytes (Nil UInt8)

const zero_byte_sample : Bytes = list_to_bytes (Cons UInt8 0 (Nil UInt8))

const first_counterexample_witness : Bool =
  property_result_failed_with
    Bytes
    property_bytes_eq
    empty_byte_sample
    (check Bytes gen_bytes reject_every_byte_sample)
```

## Laws & proofs

The real cursor check succeeds across all five generated byte strings. The
stuck-advance mutant passes the empty input and then reaches the non-empty arm,
so the runner reports the zero singleton as its first counterexample. All three
witnesses are executable `Bool` computations rather than proof terms.

```ken
const cursor_progress_witness : Bool =
  property_result_is_held Bytes (check Bytes gen_bytes cursor_progress)

const cursor_stuck_counterexample_witness : Bool =
  property_result_failed_with
    Bytes
    property_bytes_eq
    zero_byte_sample
    (check Bytes gen_bytes cursor_stuck_progress)
```

## Design notes

The deterministic representation keeps failure order stable and makes the
first counterexample reproducible. `gen_map` is the only composition operation:
binding, shrinking, random seeds, effects, and size parameters are deliberately
absent.

The cursor predicate has a live `Some` branch. Replacing its advance operation
with the identity makes that branch compute `False`, which the mutant witness
observes on the first non-empty generated value.

## References

- Claessen and Hughes, *QuickCheck: A Lightweight Tool for Random Testing of
  Haskell Programs*, ICFP 2000 — <https://doi.org/10.1145/351240.351266> —
  orientation on generator-driven property testing; this package deliberately
  uses finite deterministic samples instead of randomness.
- Wikipedia, *Property testing* — <https://en.wikipedia.org/wiki/Property_testing>
  — overview of checking general behavior over generated inputs.

## Trust & derivation

The public surface is `Gen`, `gen_from_list`, `gen_samples`, `gen_map`, `check`,
`gen_bytes`, and the three executable witnesses. Runner outcomes reuse the
prelude's `Result a Unit` rather than introducing another result carrier.
The implementation derives from the prelude's `List`, `Result`, `Unit`, `Nat`,
`Bytes`, and `UInt8` values together with the total `Bytes`/`List UInt8` view.

The declared `trusted_base()` delta is **zero**. This package introduces no
primitive, postulate, axiom, proof hole, effect, or assumed proposition. The
cursor check consumes no proof-level cursor law; it evaluates only concrete
values.
