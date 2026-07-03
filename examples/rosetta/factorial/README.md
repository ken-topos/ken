# Factorial

Compute n! (n factorial) for a given n.

Reference: <https://rosettacode.org/wiki/Factorial>

## Status

**Working, end to end.** `ken run` prints `120` (= 5!) and exits 0.
Decimal printing (a 3-digit result) was blocked pending
`wp/RTP1-interp-sharing` — resolved (`e88ffa8`), confirmed fast.

## Implementation notes

- Implemented via structural recursion on `Nat` (the L2 `data`/`match`
  machinery). `Int` is not pattern-matchable, so Nat is the natural carrier.
- `natAdd` and `natMul` are defined from first principles (no arithmetic
  primitives on `Nat`; those live on `Int`).
- SCT termination: the recursive call `factorial m` is on `m`, a structural
  sub-term of `Suc m`.
- Decimal `Nat`->`String` conversion (`natToDecimal`), same machinery as
  `examples/rosetta/gcd/gcd.ken`.

## Oracle

`main` prints `120`.
