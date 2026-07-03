# Factorial

Compute n! (n factorial) for a given n.

Reference: <https://rosettacode.org/wiki/Factorial>

## Status

**Algorithm correct; printing blocked, see `KNOWN-GAP.md`.** All
declarations elaborate; `main` evaluates to `120` (= 5!) in-process, but
does not print it via `ken run` — decimal `Nat`->`String` conversion is
confirmed infeasible at this size (the `natToDecimal` family, tracked
separately, fix in flight on `wp/RTP1-interp-sharing`).

## Implementation notes

- Implemented via structural recursion on `Nat` (the L2 `data`/`match`
  machinery). `Int` is not pattern-matchable, so Nat is the natural carrier.
- `natAdd` and `natMul` are defined from first principles (no arithmetic
  primitives on `Nat`; those live on `Int`).
- `natToInt` converts the Nat result to `Int` for human-readable output.
- Readable Nat literals (`one` through `five`) are defined incrementally.
- SCT termination: the recursive call `factorial m` is on `m`, a structural
  sub-term of `Suc m`.

## Oracle

`main` evaluates to `120`.
