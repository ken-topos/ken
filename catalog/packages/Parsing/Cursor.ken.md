# Parsing.Cursor

`Parsing.Cursor` is the carrier-neutral parsing floor. It exposes an explicit
operations dictionary, plain validity predicates, and a byte-structural
argument cursor.

## 1. Definition

`CursorOps` keeps element and location types explicit. Argument bytes remain
plain `Bytes`; lengths and elements come from the total `List UInt8` view.

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

fn arg_length (arg : Bytes) : Nat = bytes_nat_length arg

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

fn arg_location_origin (loc : ArgLocation) : Origin =
  match loc {
    MkArgLocation index start end ↦ ArgumentOrigin index (MkByteRange start end)
  }

lemma arg_location_origin_index_faithful
      (index : Nat) (start : Nat) (end : Nat)
    : Equal
        (Option Nat)
        (origin_argument_index (arg_location_origin (MkArgLocation index start end)))
        (Some Nat index) =
  Refl

lemma arg_location_origin_start_faithful
      (index : Nat) (start : Nat) (end : Nat)
    : Equal
        (Option Nat)
        (origin_range_start (arg_location_origin (MkArgLocation index start end)))
        (Some Nat start) =
  Refl

lemma arg_location_origin_end_faithful
      (index : Nat) (start : Nat) (end : Nat)
    : Equal
        (Option Nat)
        (origin_range_end (arg_location_origin (MkArgLocation index start end)))
        (Some Nat end) =
  Refl

data ArgCursor = MkArgCursor (List Bytes) Nat Nat

fn arg_cursor_args (cur : ArgCursor) : List Bytes =
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

fn arg_lengths_sum (args : List Bytes) : Nat =
  match args {
    Nil ↦ Zero;
    Cons arg rest ↦ cursor_nat_add (arg_length arg) (arg_lengths_sum rest)
  }

fn arg_remaining_from (args : List Bytes) (index : Nat) (offset : Nat) : Nat =
  match index {
    Zero ↦
      match args {
        Nil ↦ Zero;
        Cons arg rest ↦
          cursor_nat_add (cursor_nat_sub (arg_length arg) offset) (arg_lengths_sum rest)
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
  match nth Bytes (arg_cursor_index cur) (arg_cursor_args cur) {
    None ↦ None UInt8;
    Some arg ↦ nth UInt8 (arg_cursor_offset cur) (bytes_to_list arg)
  }

fn arg_cursor_normalize
      (fuel : Nat) (args : List Bytes) (index : Nat) (offset : Nat)
    : ArgCursor =
  match fuel {
    Zero ↦ MkArgCursor args index offset;
    Suc fuel2 ↦
      match nth Bytes index args {
        None ↦ MkArgCursor args index offset;
        Some arg ↦
          match cursor_nat_lt offset (arg_length arg) {
            True ↦ MkArgCursor args index offset;
            False ↦ arg_cursor_normalize fuel2 args (Suc index) Zero
          }
      }
  }

fn arg_cursor_start (args : List Bytes) : ArgCursor =
  arg_cursor_normalize (cursor_list_length Bytes args) args Zero Zero

fn arg_cursor_advance (cur : ArgCursor) : ArgCursor =
  arg_cursor_normalize
    (cursor_list_length Bytes (arg_cursor_args cur))
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
that computed bound, and a zero remaining count must be an end position.

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

Pass raw argument bytes directly to `arg_cursor_start`. It normalizes empty
arguments; ordinary `cursor_advance` then crosses argument boundaries while
preserving exact argument and byte positions.

## 4. Design notes

Lengths and elements are computed from the structural byte view. Repetition
fuel is derived from `arg_cursor_remaining`; no caller-supplied length or proof
is accepted.

## 5. References

None.

## 6. Trust  derivation

All declarations are transparent checked terms over landed `Bytes`, `List`,
`Option`, and equality. This package adds no axiom, primitive, or postulate.

## 7. Package  summary

Public surface: `CursorOps`, its four selectors and laws, `arg_length`,
`ArgLocation`, its faithful `arg_location_origin` injection, `ArgCursor`,
`arg_cursor_start`, and `arg_cursor_ops`.
