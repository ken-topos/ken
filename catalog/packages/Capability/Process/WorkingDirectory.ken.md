# Capability.Process.WorkingDirectory

`Capability.Process.WorkingDirectory` is the pure, byte-preserving view of the working
directory field in the landed `ProcessInput` ABI. The directory remains
`Bytes`; decoding is always an explicit choice made by a caller.

## 1. Raw working directory

Projection and replacement preserve argv and the captured environment
unchanged.

```ken
fn process_working_directory (input : ProcessInput) : Bytes =
  match input {
    MkProcessInput arguments environment working_directory ↦ working_directory
  }

fn replace_process_working_directory
      (working_directory : Bytes) (input : ProcessInput)
    : ProcessInput =
  match input {
    MkProcessInput arguments environment previous ↦
      MkProcessInput arguments environment working_directory
  }

proof round_trip for process_working_directory
      (working_directory : Bytes) (input : ProcessInput)
    : Equal Bytes
        (process_working_directory (replace_process_working_directory working_directory input))
        working_directory =
  match input {
    MkProcessInput arguments environment previous ↦ Refl
  }
```

## 2. Trust & derivation

All declarations are transparent checked terms over landed `ProcessInput` and
`Bytes`. The package declares no primitive, postulate, opaque constant, or
`Axiom`; its `trusted_base()` delta is zero.
