# Fibonacci sequence

Compute the n-th Fibonacci number F(n), where F(0)=0, F(1)=1, F(n)=F(n-1)+F(n-2).

Reference: <https://rosettacode.org/wiki/Fibonacci_sequence>

## Status

**Working.** All declarations elaborate. `main` evaluates to `55` (= F(10)).

## Implementation notes

- Three-case `match` on `Nat`: `Zero`, `Suc Zero`, `Suc (Suc m)` covers the
  two base cases and the recursive step cleanly.
- `natAdd (fib (Suc m)) (fib m)` maps to F(n-1) + F(n-2) for `n = Suc (Suc m)`.
- SCT termination: both recursive calls are on structural sub-terms of `Suc (Suc m)`.
- Naive double recursion is exponential (F(n) calls are O(φⁿ)). For larger
  inputs, a fast-Fibonacci using `Prod Nat Nat` (pair accumulator) is the
  follow-on; deferred.

## Oracle

`main` evaluates to `55`.
