# Capability.Process.Environment

`Capability.Process.Environment` is the pure, byte-preserving view of the environment
field in the landed `ProcessInput` ABI. Keys and values remain `Bytes`; decoding
and lookup are explicit choices made by a caller.

## 1. Raw process environment

The environment is captured once by the runner and stored as an ordered list of
raw key/value pairs. Projection and replacement preserve argv and the working
directory unchanged.

```ken
fn process_environment (input : ProcessInput) : List (Prod Bytes Bytes) =
  match input {
    MkProcessInput arguments environment working_directory ↦ environment
  }

fn replace_process_environment
      (environment : List (Prod Bytes Bytes)) (input : ProcessInput)
    : ProcessInput =
  match input {
    MkProcessInput arguments previous working_directory ↦
      MkProcessInput arguments environment working_directory
  }

proof round_trip for process_environment
      (environment : List (Prod Bytes Bytes)) (input : ProcessInput)
    : Equal
        (List (Prod Bytes Bytes))
        (process_environment (replace_process_environment environment input))
        environment =
  match input {
    MkProcessInput arguments previous working_directory ↦ Refl
  }
```

## 2. Trust & derivation

All declarations are transparent checked terms over landed `ProcessInput`,
`List`, `Prod`, and `Bytes`. The package compares no keys and declares no
primitive, postulate, opaque constant, or `Axiom`; its `trusted_base()` delta is
zero.
