# FizzBuzz

For n = 1 to 20, print:
- "FizzBuzz" if divisible by both 3 and 5
- "Fizz" if divisible by 3
- "Buzz" if divisible by 5
- n itself otherwise

Reference: <https://rosettacode.org/wiki/FizzBuzz>

## Status

**Working, end to end.** `ken run` prints the correct 20-line sequence
and exits 0. All originally-documented gaps are now closed: string
literals, `print_line`/Console execution, `Bool` pattern-matching, and
(most recently) decimal printing for the `Plain` case — that last one was
blocked pending `wp/RTP1-interp-sharing`, resolved (`e88ffa8`).

## Implementation notes

- Classification (`Mod3`/`Mod5`/`classify`) uses modular accumulator
  types to work around `GAP-nested-patterns` (nested constructor patterns
  like `Suc (Suc Zero)` still trigger a reachability error in match
  position) — unaffected by, and independent of, the printing gap.
- `fizzBuzzLoop` is a `Vis`-under-closure recursive IO combinator — the
  same shape VAL2's own `printLoop` probe confirmed SCT-acceptable
  (the recursive call sits inside `\_. fizzBuzzLoop r (Suc current)`,
  itself a constructor argument to `Vis`).
- Decimal `Nat`->`String` conversion (`natToDecimal`), same machinery as
  `examples/rosetta/gcd/gcd.ken`.

## Oracle

`main` prints the 20-line FizzBuzz sequence for 1..20.
