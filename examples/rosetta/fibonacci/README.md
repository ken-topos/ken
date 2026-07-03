# Fibonacci sequence

Compute the n-th Fibonacci number F(n), where F(0)=0, F(1)=1, F(n)=F(n-1)+F(n-2).

Reference: <https://rosettacode.org/wiki/Fibonacci_sequence>

## Status

**Working, end to end.** `ken run` prints `55` (= F(10)) and exits 0.
Decimal printing (a 2-digit result) was blocked pending
`wp/RTP1-interp-sharing` — resolved (`e88ffa8`), confirmed fast.

## Implementation notes

- Iterative linear Fibonacci (`fibStep`, two `Nat` accumulators) —
  `GAP-nested-patterns` (`Suc (Suc m)` as a match pattern triggers a
  reachability error) rules out the naive 3-case match; this is also
  O(n) instead of the naive exponential double-recursion.
- Decimal `Nat`->`String` conversion (`natToDecimal`), same machinery as
  `examples/rosetta/gcd/gcd.ken`.

## Oracle

`main` prints `55`.
