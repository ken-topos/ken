# FizzBuzz

For n = 1 to 20, print:
- "FizzBuzz" if divisible by both 3 and 5
- "Fizz" if divisible by 3
- "Buzz" if divisible by 5
- n itself otherwise

Reference: <https://rosettacode.org/wiki/FizzBuzz>

## Status

**Classification logic works; output blocked by one remaining gap, see
`KNOWN-GAP.md`.** Most of this dir's originally-documented gaps are now
closed (string literals, `print_line`/Console execution, `Bool`
pattern-matching all landed since this file was first written) — the sole
remaining blocker is printing the bare decimal `n` in the non-Fizz/Buzz
case, which needs a `Nat`->`String` conversion that's currently
exponential-cost in `ken-interp` (tracked separately, fix in flight,
`wp/RTP1-interp-sharing`) and infeasible across the 1..20 range.

## What works

All classification views (`Mod3`, `Mod5`, `incMod3`, `incMod5`, `mod3Step`,
`mod5Step`, `mod3`, `mod5`, `classify`) elaborate correctly, using modular
accumulator types to work around `GAP-nested-patterns` (nested constructor
patterns like `Suc (Suc Zero)` still trigger a reachability error in match
position — unrelated to, and unaffected by, the decimal-printing gap
above). `classify n` returns the correct `FizzTag` for any `Nat` input.
