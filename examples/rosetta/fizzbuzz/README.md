# FizzBuzz

For n = 1 to 20, print:
- "FizzBuzz" if n is divisible by both 3 and 5
- "Fizz" if divisible by 3
- "Buzz" if divisible by 5
- n itself otherwise

Reference: <https://rosettacode.org/wiki/FizzBuzz>

## Status

**PARTIALLY implemented — classification logic works; output blocked by gaps.**

## What works

All classification views (`Mod3`, `Mod5`, `incMod3`, `incMod5`, `mod3Step`,
`mod5Step`, `mod3`, `mod5`, `classify`) elaborate correctly. The divisibility
logic uses modular accumulator types (`Mod3`, `Mod5`) to work around two
surface limitations (see GAP-nested-patterns below). `classify n` returns the
correct `FizzTag` for any `Nat` input.

## GAP: string literals — FIXED (`37 §2.1`, VAL1-surface)

String literals now parse and elaborate. The `"Fizz"` / `"Buzz"` / `"FizzBuzz"`
literals can now appear in Ken expressions. Output is still blocked by
GAP-io-surface.

## GAP-nested-patterns: nested constructor patterns trigger ReachabilityError

A match arm pattern like `Suc (Suc Zero)` (nested constructor) causes the
elaborator's reachability checker to raise `ReachabilityError`, making
structural mod3/mod5 with explicit depth patterns impossible.

**Workaround**: define a `Mod3` (3-element) and `Mod5` (5-element) data type
as a modular accumulator. `mod3Step n acc` steps `n` times from `acc` using
`incMod3`, with only flat patterns (`Zero | Suc m` and `Zero3 | One3 | Two3`).
Avoids nested patterns and mutual recursion entirely.

## GAP: no surface print / I/O

Cannot iterate n from 1 to 20 and print each result. See `hello-world/README.md`.

## GAP: subtraction / modulo operators not in surface

`n % 3` is not a surface expression. Worked around with structural `Nat`
recursion (`mod3`/`mod5`). The workaround is correct but requires writing out
each divisor's pattern explicitly.

## GAP: no integer-range generator

`List.range` or an equivalent `[1..20]` syntax does not exist. Building the
result list requires either a `Nat`-indexed accumulator or spelling out
20 explicit `Suc (Suc ...)` Nat literals — both are impractical without a
surface range primitive.

## GAP: Bool not pattern-matchable

`Bool` is an opaque primitive (`not data Bool = True | False`), so `if
n == 0` is not expressible. Worked around with a custom `data IsZero = Zero_
| NonZero_` and an `isZero` view.

## Intended program (once gaps are resolved)

```ken
view fizzBuzz (n : Int) : String =
  if n % 15 == 0 then "FizzBuzz"
  else if n % 3 == 0 then "Fizz"
  else if n % 5 == 0 then "Buzz"
  else Int.show n

view main : IO Unit =
  let go (i : Int) : IO Unit =
    if i > 20 then return ()
    else print_line (fizzBuzz i) >> go (i + 1)
  go 1
```
