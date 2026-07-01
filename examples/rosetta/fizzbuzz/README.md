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

`mod3`, `mod5`, and `classify` elaborate correctly. The divisibility-by-3 and
divisibility-by-5 classification via structural recursion on `Nat` is fully
expressible in the landed surface (L2 `data`/`match`, L3 `List`/`Nat`). The
`FizzTag` type encodes the four cases.

`classify n` returns the correct `FizzTag` for any `Nat` input.

## GAP: string literals not in expression grammar

Cannot produce "Fizz", "Buzz", "FizzBuzz" strings. See `hello-world/README.md`
for the full description. Same fix required.

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
