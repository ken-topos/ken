# Parsing.Cursor

`Parsing.Cursor` is the carrier-neutral parsing floor. It exposes an explicit
operations dictionary, plain validity predicates, and the argument cursor whose
opaque byte lengths are certified at construction.

## 1. Definition

`CursorOps` keeps element and location types explicit. `ArgBytes` mirrors the
landed CAT-5 `Source` boundary: raw `Bytes`, cached `Nat` lengths, and proofs
that the two representations agree, without an `Int → Nat` primitive.

```ken
data CursorOps c el loc = MkCursorOps (c → Nat) (c → Option el) (c → c) (c → loc)

fn cursor_remaining
      (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) (cur : c)
    : Nat =
  match ops {
    MkCursorOps remaining peek advance locate ↦ remaining cur
  }

fn cursor_peek
      (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) (cur : c)
    : Option el =
  match ops {
    MkCursorOps remaining peek advance locate ↦ peek cur
  }

fn cursor_advance (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) (cur : c) : c =
  match ops {
    MkCursorOps remaining peek advance locate ↦ advance cur
  }

fn cursor_locate
      (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) (cur : c)
    : loc =
  match ops {
    MkCursorOps remaining peek advance locate ↦ locate cur
  }

fn cursor_nat_to_int (n : Nat) : Int =
  match n {
    Zero ↦ (0 : Int);
    Suc n2 ↦ (1 : Int) + cursor_nat_to_int n2
  }

fn ArgByteLength (bs : Bytes) (n : Nat) : Prop =
  Equal Int (bytes_length bs) (cursor_nat_to_int n)

class ArgBytes {
  arg_bytes_field : Bytes;
  arg_length_field : Nat;
  arg_length_valid_field : ArgByteLength arg_bytes_field arg_length_field
}

fn arg_bytes (arg : ArgBytes) : Bytes = arg.arg_bytes_field

fn arg_length (arg : ArgBytes) : Nat = arg.arg_length_field

proof valid for arg_length (arg : ArgBytes) : ArgByteLength (arg_bytes arg) (arg_length arg) =
  arg.arg_length_valid_field

data ArgLocation = MkArgLocation Nat Nat Nat

fn arg_location_index (loc : ArgLocation) : Nat =
  match loc {
    MkArgLocation index start end ↦ index
  }

fn arg_location_start (loc : ArgLocation) : Nat =
  match loc {
    MkArgLocation index start end ↦ start
  }

fn arg_location_end (loc : ArgLocation) : Nat =
  match loc {
    MkArgLocation index start end ↦ end
  }

data ArgCursor = MkArgCursor (List ArgBytes) Nat Nat

fn arg_cursor_args (cur : ArgCursor) : List ArgBytes =
  match cur {
    MkArgCursor args index offset ↦ args
  }

fn arg_cursor_index (cur : ArgCursor) : Nat =
  match cur {
    MkArgCursor args index offset ↦ index
  }

fn arg_cursor_offset (cur : ArgCursor) : Nat =
  match cur {
    MkArgCursor args index offset ↦ offset
  }

fn cursor_nat_add (a : Nat) (b : Nat) : Nat =
  match b {
    Zero ↦ a;
    Suc b2 ↦ Suc (cursor_nat_add a b2)
  }

fn cursor_nat_sub (a : Nat) (b : Nat) : Nat =
  match b {
    Zero ↦ a;
    Suc b2 ↦
      match a {
        Zero ↦ Zero;
        Suc a2 ↦ cursor_nat_sub a2 b2
      }
  }

fn cursor_nat_lt (a : Nat) (b : Nat) : Bool =
  match b {
    Zero ↦ False;
    Suc b2 ↦
      match a {
        Zero ↦ True;
        Suc a2 ↦ cursor_nat_lt a2 b2
      }
  }

fn cursor_list_length (a : Type) (xs : List a) : Nat =
  match xs {
    Nil ↦ Zero;
    Cons x rest ↦ Suc (cursor_list_length a rest)
  }

fn arg_lengths_sum (args : List ArgBytes) : Nat =
  match args {
    Nil ↦ Zero;
    Cons arg rest ↦ cursor_nat_add (arg_length arg) (arg_lengths_sum rest)
  }

fn arg_remaining_from (args : List ArgBytes) (index : Nat) (offset : Nat) : Nat =
  match index {
    Zero ↦
      match args {
        Nil ↦ Zero;
        Cons arg rest ↦ cursor_nat_add
          (cursor_nat_sub (arg_length arg) offset)
          (arg_lengths_sum rest)
      };
    Suc index2 ↦
      match args {
        Nil ↦ Zero;
        Cons arg rest ↦ arg_remaining_from rest index2 offset
      }
  }

fn arg_cursor_remaining (cur : ArgCursor) : Nat =
  arg_remaining_from (arg_cursor_args cur) (arg_cursor_index cur) (arg_cursor_offset cur)

fn arg_cursor_peek (cur : ArgCursor) : Option UInt8 =
  match nth ArgBytes (arg_cursor_index cur) (arg_cursor_args cur) {
    None ↦ None UInt8;
    Some arg ↦ bytes_at (arg_bytes arg) (cursor_nat_to_int (arg_cursor_offset cur))
  }

fn arg_cursor_normalize
      (fuel : Nat) (args : List ArgBytes) (index : Nat) (offset : Nat)
    : ArgCursor =
  match fuel {
    Zero ↦ MkArgCursor args index offset;
    Suc fuel2 ↦
      match nth ArgBytes index args {
        None ↦ MkArgCursor args index offset;
        Some arg ↦
          match cursor_nat_lt offset (arg_length arg) {
            True ↦ MkArgCursor args index offset;
            False ↦ arg_cursor_normalize fuel2 args (Suc index) Zero
          }
      }
  }

fn arg_cursor_start (args : List ArgBytes) : ArgCursor =
  arg_cursor_normalize (cursor_list_length ArgBytes args) args Zero Zero

fn arg_cursor_advance (cur : ArgCursor) : ArgCursor =
  arg_cursor_normalize
    (cursor_list_length ArgBytes (arg_cursor_args cur))
    (arg_cursor_args cur)
    (arg_cursor_index cur)
    (Suc (arg_cursor_offset cur))

fn arg_cursor_locate (cur : ArgCursor) : ArgLocation =
  MkArgLocation (arg_cursor_index cur) (arg_cursor_offset cur) (arg_cursor_offset cur)

const arg_cursor_ops : CursorOps ArgCursor UInt8 ArgLocation =
  MkCursorOps
    ArgCursor
    UInt8
    ArgLocation
    arg_cursor_remaining
    arg_cursor_peek
    arg_cursor_advance
    arg_cursor_locate
```

## 2. Laws

The laws are predicates over an explicit dictionary. A successful peek must
have positive remaining input, advancing such a cursor must strictly reduce
that cached bound, and a zero remaining count must be an end position.

```ken
fn CursorPeekHasRemaining
      (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc)
    : Prop =
  (cur : c)
    → (value : el)
    → Equal
    (Option el)
    (cursor_peek c el loc ops cur)
    (Some el value)
    → Equal Bool
    (cursor_nat_lt Zero (cursor_remaining c el loc ops cur))
    True

fn CursorAdvanceProgress (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) : Prop =
  (cur : c)
    → (value : el)
    → Equal
    (Option el)
    (cursor_peek c el loc ops cur)
    (Some el value)
    → Equal Bool
    (cursor_nat_lt
      (cursor_remaining c el loc ops (cursor_advance c el loc ops cur))
      (cursor_remaining c el loc ops cur))
    True

fn CursorEndValid (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) : Prop =
  (cur : c)
    → Equal Nat
    (cursor_remaining c el loc ops cur)
    Zero → Equal
    (Option el)
    (cursor_peek c el loc ops cur)
    (None el)

fn CursorLaws (c : Type) (el : Type) (loc : Type) (ops : CursorOps c el loc) : Prop =
  And
    (CursorPeekHasRemaining c el loc ops)
    (And (CursorAdvanceProgress c el loc ops) (CursorEndValid c el loc ops))
```

## 3. Using it

Construct each `ArgBytes` value with raw bytes, its cached `Nat` length, and the
matching proof, then pass their list to `arg_cursor_start`. It normalizes empty
arguments; ordinary `cursor_advance` then crosses argument boundaries while
preserving exact argument and byte positions.

## 4. Design notes

The cached length is boundary evidence, not a caller budget. Repetition fuel is
derived from `arg_cursor_remaining`. The duplicated cached-length idiom is kept
local in CC3; unifying it with CAT-5 `Source` is a separate substrate decision.

## 5. References

None.

## 6. Trust  derivation

All declarations are transparent checked terms over landed `Bytes`, `List`,
`Option`, and equality. This package adds no axiom, primitive, or postulate.

## 7. Package  summary

Public surface: `CursorOps`, its four selectors and laws, `ArgBytes`,
`ArgLocation`, `ArgCursor`, `arg_cursor_start`, and `arg_cursor_ops`.
