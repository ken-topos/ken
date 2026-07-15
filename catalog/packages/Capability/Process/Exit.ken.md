# Capability.Process.Exit

`Capability.Process.Exit` is the application-facing policy over the landed `ExitCode` ABI.
A program chooses its status explicitly before returning to the runner; the
host never inspects the shape of an application result.

## 1. Exit values and policy

`exit_success` and `exit_failure` name both ABI choices. `exit_with` applies a
total policy supplied by the program, so every constructor of an
application-specific result is handled in ordinary Ken code.

```ken
const exit_success : ExitCode = Success

fn exit_failure (code : UInt8) : ExitCode = Failure code

fn exit_with (a : Type) (policy : a → ExitCode) (outcome : a) : ExitCode = policy outcome

fn exit_from_result
      (error : Type) (value : Type) (on_error : error → UInt8) (outcome : Result error value)
    : ExitCode =
  match outcome {
    Err problem ↦ exit_failure (on_error problem);
    Ok answer ↦ exit_success
  }
```

## 2. Design notes

There is no ambient conversion and no default status hidden in the runner.
Applications may use `exit_with` for their own result carrier or
`exit_from_result` for the common `Result` shape. Failure codes remain the
landed `UInt8`, rather than widening to `Int` or introducing another ABI type.

## 3. Trust & derivation

All four declarations are transparent checked terms over the prelude's
`ExitCode`, `UInt8`, and `Result`. This package declares no datatype, primitive,
postulate, opaque constant, or `Axiom`; its `trusted_base()` delta is zero.
