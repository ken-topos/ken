# Greatest common divisor

Compute gcd(a, b) for two non-negative integers.

Reference: <https://rosettacode.org/wiki/Greatest_common_divisor>

## Status

**Working, end to end.** `ken run examples/rosetta/gcd/gcd.ken` prints
`4` and exits 0. Decimal printing was blocked pending
`wp/RTP1-interp-sharing` (eager-IH-computation cost in `ken-interp`'s
`elim_reduce`) — resolved (`e88ffa8`), confirmed fast.

## Implementation notes

- Subtraction-based GCD with explicit fuel (`natGcdFueled`, fuel = `a+b`)
  for SCT termination.
- `GAP-nested-patterns`: `Suc _` in `natCmp` (a nested constructor
  pattern) triggers a reachability error — worked around via a
  non-recursive `natCmpZero` helper with only flat patterns.
- Decimal `Nat`->`String` conversion (`natToDecimal`) — `ken-interp` has
  no `div_int`/`mod_int` primitive, so it goes via structural `Nat`
  peeling (`sub10`, ten chained flat matches) + a `Nat`-fuel loop.

## Oracle

`main` prints `4`.
