# Capability.Process.Arguments

`Capability.Process.Arguments` is the pure, byte-preserving view of the argv field in the
landed `ProcessInput` ABI. Raw arguments remain `Bytes`; decoding is always an
explicit choice made by a caller.

## 1. Raw process input

The runner's third `ProcessInput` field is the current working directory. This
package names only the argv projection and replacement operation, while the
match keeps the environment and working-directory bytes unchanged.

```ken
fn process_arguments (input : ProcessInput) : List Bytes =
  match input {
    MkProcessInput arguments environment working_directory ↦ arguments
  }

fn replace_process_arguments (arguments : List Bytes) (input : ProcessInput) : ProcessInput =
  match input {
    MkProcessInput previous environment working_directory ↦
      MkProcessInput arguments environment working_directory
  }

proof round_trip for process_arguments
      (arguments : List Bytes) (input : ProcessInput)
    : Equal
        (List Bytes)
        (process_arguments (replace_process_arguments arguments input))
        arguments =
  match input {
    MkProcessInput previous environment working_directory ↦ Refl
  }

fn process_argument_at (index : Nat) (input : ProcessInput) : Option Bytes =
  nth Bytes index (process_arguments input)
```

## 2. Arguments and locations

Parsing consumes raw argument `Bytes`. Positional lookup exposes those bytes
directly, and bounds are checked against their structural `Nat` length.

`argument_slice_location` accepts only a range whose argument exists, whose
start does not exceed its end, and whose end does not exceed the computed byte
length. The resulting location is CC3's existing `ArgLocation`.

```ken
fn argument_at (index : Nat) (arguments : List Bytes) : Option Bytes = nth Bytes index arguments

fn argument_bytes_at (index : Nat) (arguments : List Bytes) : Option Bytes =
  match argument_at index arguments {
    None ↦ None Bytes;
    Some argument ↦ Some Bytes argument
  }

fn argument_nat_leq (left : Nat) (right : Nat) : Bool =
  match left {
    Zero ↦ True;
    Suc left2 ↦
      match right {
        Zero ↦ False;
        Suc right2 ↦ argument_nat_leq left2 right2
      }
  }

fn argument_slice_location
      (index : Nat) (start : Nat) (end : Nat) (arguments : List Bytes)
    : Option ArgLocation =
  match argument_at index arguments {
    None ↦ None ArgLocation;
    Some argument ↦
      match argument_nat_leq start end {
        False ↦ None ArgLocation;
        True ↦
          match argument_nat_leq end (bytes_nat_length argument) {
            False ↦ None ArgLocation;
            True ↦ Some ArgLocation (MkArgLocation index start end)
          }
      }
  }
```

## 3. Design notes

The raw ABI projection and parsing view meet at `Bytes`, not `String`.
`ArgLocation` remains the sole argument byte-range carrier, while lengths are
computed from the total structural byte view.

## 4. Trust & derivation

All declarations are transparent checked terms over landed `ProcessInput`,
`List`, `Bytes`, and `ArgLocation`. The package declares no
primitive, postulate, opaque constant, or `Axiom`; its `trusted_base()` delta is
zero.
